use super::chain::{
    additive_chain_canonical, has_repeated_leaf, leaves_ascending, substitute, MAX_SUBST_OPS,
};
use super::expr::{
    eat_return_plumbing, eat_scopes, formals_marker, intrinsic_selector, parse_expr, parse_formals,
    BODY_SCOPE_DEPTH,
};
use super::{blk, Block, BodyShape};
use crate::func::readers::{
    eat, eat_byte, eat_int_like, eat_opt_stmt_marker, is_int4_type, is_ptr_to_4,
    read_token_var, read_type, read_varint, DOUBLE_LIT_TYPE, DOUBLE_TYPE, FLOAT_LIT_TYPE,
    FLOAT_TYPE, INT_TYPE, UINT_TYPE,
};
use crate::func::{CompareLeaf, IlOp, Rel};

/// Try to parse a body that is a **list of assignment statements** followed by a
/// returned expression:
///
/// ```text
///   body := ( `4F 01 <line>`* assign )* `4F 01 <line>`* expr(→41) <return int>
///   assign := `26 <dst>` expr(→32) `32 <TYPE>` `4B`
/// ```
///
/// `26 <dst>` pushes the destination, `32 <TYPE>` stores it (and yields the value,
/// which `4B` then discards). The `<TYPE>` is the destination's own — a conversion
/// is always a separate visible `2C`, so an int-like type here means no conversion.
///
/// **These bodies need no stores at all.** c2 register-allocates locals and
/// coalesces the copies, so the whole class collapses to the expression that
/// actually reaches the `return`. Captured:
///
/// ```text
///   int x; x = a; return x;              -> blr            (x is already r3)
///   int x = a; int y = x; return y;      -> blr
///   a = a + 1; return a;                 -> addi r3,r3,1
///   int x = 0; x = a + 1; return x;      -> addi r3,r3,1    (the x = 0 is dead)
///   a = 7; return a;                     -> li r3,7
/// ```
///
/// So this resolves the statement list by substitution and hands codegen the
/// resulting straight-line expression, which is exactly what the reference emits.
///
/// The destination must be a **formal** (positively, from the `2D` list) or an
/// automatic `int` **local** (positively, from `.sy` — see [`crate::func::sy`]).
///
/// An earlier version asked whether `.gl` named the destination and refused if so.
/// That looked sound and was not: a file-scope `static int sv` appears there as
/// `$sv`, whose leading `$` `gl_symbol_index` does not accept as an identifier, so
/// the token looked local and the store was silently dropped. Absence from a symbol
/// table proves nothing — it only says the table did not happen to name it.
///
/// `.sy` replaces that absence test with a membership one. Globals, `extern`
/// declarations and file-scope statics appear in `.sy` not at all, and a
/// function-scope `static` appears as a differently-tagged record, so a token
/// found in the locals section is a register-resident value and folding its store
/// into the expression that reads it is what c2 itself does — measured: for
/// `int f(int a){int x=a+1;int y=x+2;return y;}` and `int f(int a){return
/// (a+1)+2;}` the reference objs differ only in the source-filename bytes.
pub(crate) fn try_parse_assign_body_detail(
    seg: &[u8],
    start: usize,
    lo: usize,
    locals: &[u32],
    mut depth: usize,
) -> Result<BodyShape, Block> {
    let mut p = start;
    let mut env: Vec<(u32, Vec<IlOp>)> = Vec::new();
    // Read once for the per-destination check. A body with no formals marker is not
    // rejected here — the destination check below refuses it anyway, and deferring
    // lets the right-hand side report its own reason first, which is what makes the
    // census name the innermost unmodeled construct rather than this outer gate.
    let formals = parse_formals(seg, lo).unwrap_or_default();
    loop {
        // Brace scopes open and close *between* statements, so they are consumed at
        // the boundary rather than being a statement of their own.
        eat_scopes(seg, &mut p, &mut depth)?;
        if *seg.get(p).ok_or(blk(seg, p, "assign-stmt"))? != 0x26 {
            break;
        }
        let mut probe = p + 1;
        let (dst, w) =
            read_token_var(seg, probe).ok_or(blk(seg, probe, "assign-dst-tok"))?;
        probe += w;
        // `BD` here means this `26` was a callee push, not a destination. The caller
        // dispatched on the FIRST one, so reaching this means the right-hand side is
        // itself a call: `int z = g(a); …`. When the very next thing is a return of
        // that same local the whole body is a tail call, which `parse_call_shape`
        // already handles given the bound token — so hand it over rather than refuse.
        //
        // Only valid as the FIRST statement: with `env` non-empty, earlier
        // assignments have already been folded away and would be lost.
        // The right-hand side is a call when it opens with its own `26 <callee>`
        // followed by the `BD` CALL opcode — two tokens along from the destination,
        // not one.
        let rhs_is_call = *seg.get(probe).ok_or(blk(seg, probe, "assign-op"))? == 0x26
            && match read_token_var(seg, probe + 1) {
                Some((_, cw)) => seg.get(probe + 1 + cw) == Some(&0xBD),
                None => false,
            };
        if rhs_is_call {
            if env.is_empty() {
                let mut q = probe;
                return parse_call_shape(seg, &mut q, lo, Some(dst));
            }
            return Err(blk(seg, probe, "assign-rhs-call"));
        }
        p = probe;
        let rhs = parse_expr(seg, &mut p, 0x32)?;
        // A store to any memory object (a global, or a file-scope `static`) is a
        // real write with a relocation, and treating it as a register copy silently
        // drops it. Testing *absence* from `.gl` failed exactly there: a
        // `static int sv` is in `.gl` as `$sv`, whose leading `$`
        // `gl_symbol_index` does not accept as an identifier, so the token looked
        // local and `static int sv; int f(int a){ sv = a; return a; }` mis-emitted.
        // Found by probing the de-conflated census, not by a fixture.
        //
        // A local is admitted only on `.sy`'s positive evidence, and only when
        // `.sy` said plain `int`, never address-taken, and bound its block 1:1 to
        // the `.ex` segments. Everything `.sy` cannot vouch for — a global, a
        // file-scope or function-scope static, a qualified or non-`int` local, a
        // local whose address escapes — leaves this list empty and refuses here,
        // exactly as before.
        if !formals.contains(&dst) && !locals.contains(&dst) {
            return Err(Block { ctx: "assign-dst-not-formal", byte: None, off: probe, aux: 0 });
        }
        if !eat_byte(seg, &mut p, 0x32) || !eat_int_like(seg, &mut p) {
            return Err(blk(seg, p, "assign-store-type"));
        }
        // `4B` ends an expression statement and discards the yielded value. A
        // body that *uses* it (`x = y = a`) does not have one here and refuses.
        if !eat_byte(seg, &mut p, 0x4B) {
            return Err(blk(seg, p, "assign-stmt-end"));
        }
        let rhs = substitute(&rhs, &env)
            .ok_or(Block { ctx: "assign-subst-overflow", byte: None, off: p, aux: 0 })?;
        // Re-assigning shadows the previous definition, which is how a dead store
        // disappears: only the last definition can reach the return.
        env.retain(|(t, _)| *t != dst);
        env.push((dst, rhs));
        if env.len() > MAX_SUBST_OPS {
            return Err(Block { ctx: "assign-too-many-locals", byte: None, off: p, aux: 0 });
        }
    }
    eat_scopes(seg, &mut p, &mut depth)?;
    let ret = parse_expr(seg, &mut p, 0x41)?;
    let ret = substitute(&ret, &env)
        .ok_or(Block { ctx: "assign-subst-overflow", byte: None, off: p, aux: 0 })?;
    eat_return_plumbing(seg, &mut p, true, depth)?;
    let params = parse_params(seg, lo)?;
    // After substitution every remaining LOAD must be a parameter. Anything else
    // is a read of something this class cannot account for — an uninitialized
    // local, a global, or a token from a construct not modeled here.
    if !ret.iter().all(|o| match o {
        IlOp::Load(t) => params.contains(t),
        _ => true,
    }) {
        return Err(Block { ctx: "assign-ret-nonformal", byte: None, off: p, aux: 0 });
    }
    // Substitution is a *source* of repeated leaves even when the written source
    // has none: `int x = a; x = x + x;` substitutes to `a + a`, which c2 emits as
    // `slwi r3,r3,1`. This gate is what keeps that from being wrong bytes.
    if has_repeated_leaf(&ret) {
        return Err(Block { ctx: "assign-repeated-leaf", byte: None, off: p, aux: 0 });
    }
    // Substitution reorders too: `int x = b; return x + a;` resolves to `b + a`.
    if !leaves_ascending(&ret, &params) || !additive_chain_canonical(&ret) {
        return Err(Block { ctx: "assign-noncanonical-order", byte: None, off: p, aux: 0 });
    }
    Ok(BodyShape::StraightLine { params, ops: ret })
}

/// Try to parse a **W13a floating-point leaf**: a straight-line chain over
/// float (or double) *parameters* only.
///
/// ```text
///   ( B9 <tok> <FT> | <op> )+     LOADs and binary ops, all of one FP type
///   41 <FT>                       result type, the SAME FP type
///   <return plumbing>
/// ```
///
/// The gate list is from `docs/CODEGEN_W13_FLOAT.md` §6 and every item is a
/// case where a naive selector emits *wrong* bytes rather than merely running
/// out of range:
///
/// * **No literal.** Every FP constant costs an `.rdata` COMDAT, a REFHI/REFLO
///   relocation pair and a GPR — that is W13b.
/// * **No `2C` convert**, and no mixing of float with double: a mixed-width
///   expression evaluates in double and may need an `frsp`.
/// * **No `*` under `+`/`-`.** Contraction to `fmadds`/`fmsubs`/`fnmsubs` is
///   *mandatory* in c2, so emitting the two separate instructions would be a
///   silent mis-emit. Approximated conservatively here by rejecting any chain
///   that contains both a `Mul` and an `Add`/`Sub`.
/// * **No repeated leaf.** `a + a` is algebraically rewritten to `a * 2.0f`,
///   which is a constant and therefore `.rdata` again.
/// * **No `0x59` marker.** It tracks source parenthesisation and is the only
///   thing distinguishing product shapes c2 flattens from those it does not;
///   its meaning is unknown, so its presence rejects.
pub(crate) fn try_parse_float_leaf(seg: &[u8], start: usize, lo: usize) -> Option<BodyShape> {
    let mut p = start;
    // The operand type is fixed by the first LOAD and every later one must match.
    if *seg.get(p)? != 0xB9 {
        return None;
    }
    let double = {
        let mut probe = p + 1;
        let (_, w) = read_token_var(seg, probe)?;
        probe += w;
        if seg.get(probe..probe + 3)? == FLOAT_TYPE {
            false
        } else if seg.get(probe..probe + 3)? == DOUBLE_TYPE {
            true
        } else {
            return None;
        }
    };
    let fty = if double { DOUBLE_TYPE } else { FLOAT_TYPE };

    let mut ops: Vec<IlOp> = Vec::new();
    loop {
        match *seg.get(p)? {
            0xB9 => {
                p += 1;
                let (tok, w) = read_token_var(seg, p)?;
                p += w;
                if seg.get(p..p + 3)? != fty {
                    return None; // mixed width, or a non-FP operand
                }
                p += 3;
                ops.push(IlOp::Load(tok));
            }
            0x02 => {
                p += 1;
                ops.push(IlOp::Add);
            }
            0x03 => {
                p += 1;
                ops.push(IlOp::Sub);
            }
            0x04 => {
                p += 1;
                ops.push(IlOp::Mul);
            }
            0x05 => {
                p += 1;
                ops.push(IlOp::Div);
            }
            // W13b: a floating-point literal.
            //
            //   33 <lit-TYPE> <8 bytes: IEEE binary64, little-endian> <u16 size>
            //
            // The payload is a binary64 pattern even for a `float` (already
            // rounded to binary32 precision), and the u16 trailer is the operand
            // *width* — 4 for float, 8 for double — which must agree with the
            // literal tag. Verified byte-for-byte against a live capture of
            // `float k_add(float a){return a + 1.0f;}`:
            //   33 86 4a 40 00 00 00 00 00 00 f0 3f 04 00
            0x33 => {
                p += 1;
                let lty = seg.get(p..p + 3)?;
                let lit_double = if lty == FLOAT_LIT_TYPE {
                    false
                } else if lty == DOUBLE_LIT_TYPE {
                    true
                } else {
                    return None; // an integer (or other) literal: out of class
                };
                // A literal of the other width implies a conversion.
                if lit_double != double {
                    return None;
                }
                p += 3;
                let raw: [u8; 8] = seg.get(p..p + 8)?.try_into().ok()?;
                p += 8;
                let size = u16::from_le_bytes(seg.get(p..p + 2)?.try_into().ok()?);
                p += 2;
                if size as usize != if double { 8 } else { 4 } {
                    return None;
                }
                ops.push(IlOp::FpLit {
                    bits: u64::from_le_bytes(raw),
                    double,
                });
            }
            0x41 => break,
            // 0x2C convert, 0x59 paren marker, 0x08 neg and every other byte
            // reject — see the gate list above.
            _ => return None,
        }
    }
    // Result type must be the same FP type.
    p += 1;
    if seg.get(p..p + 3)? != fty {
        return None;
    }
    p += 3;
    eat_return_plumbing(seg, &mut p, false, BODY_SCOPE_DEPTH).ok()?;

    // A `*` mixed with `+`/`-` contracts; reject rather than emit two
    // instructions where c2 emits one.
    let has_mul = ops.iter().any(|o| matches!(o, IlOp::Mul));
    let has_addsub = ops.iter().any(|o| matches!(o, IlOp::Add | IlOp::Sub));
    if has_mul && has_addsub {
        return None;
    }

    // ---- W13b constant gates ------------------------------------------------
    //
    // These live here, in the parser, rather than in codegen so that the census
    // and the emission gate cannot disagree about what is in class.
    //
    // c2 — not c1xx — evaluates floating-point constants, so the IL still holds
    // every literal the source wrote and the backend is free to fold, reassociate
    // and strength-reduce them. Three captured behaviours the port does not
    // model, each of which would be a silent mis-emit:
    let lits: Vec<(u64, bool)> = ops
        .iter()
        .filter_map(|o| match o {
            IlOp::FpLit { bits, double } => Some((*bits, *double)),
            _ => None,
        })
        .collect();
    if !lits.is_empty() {
        // (1) Two or more literals: c2 folds them where it can (`a*2.0f*b*3.0f`
        //     becomes `(a*b)*6.0f`), and where it cannot it hoists every `addis`
        //     into a prologue group and schedules the loads at first use. Either
        //     way the one-constant lowering is wrong. See `w13b_fpool.cpp`.
        if lits.len() > 1 {
            return None;
        }
        // (2) A constant divisor becomes a reciprocal multiply: `a/2.0f` emits
        //     `fmuls` against `__real@3f000000`, and `a/3.0f/7.0f` collapses to
        //     one `fmuls` by 1/21 — a value that is not even exactly
        //     representable, so this is a numeric transform, not a rewrite.
        if ops.iter().any(|o| matches!(o, IlOp::Div)) {
            return None;
        }
        let (bits, lit_double) = lits[0];
        let v = f64::from_bits(bits);
        // (3) An identity operand disappears entirely — `a + 0.0f`, `a - 0.0f`
        //     and `a * 1.0f` each compile to a bare `blr`, with no constant
        //     pooled at all. (`a * 0.0f` is *not* folded: it really does load
        //     zero and multiply.) Refuse when the literal is an identity for any
        //     operator in the body; slight over-refusal beats emitting three
        //     instructions where c2 emits none.
        if v == 0.0 && has_addsub {
            return None;
        }
        if v == 1.0 && has_mul {
            return None;
        }
        // (4) A `float` literal is carried as a binary64 pattern already rounded
        //     to binary32. If it does not narrow exactly, the four bytes we would
        //     pool are not the ones c2 pooled.
        if !lit_double && f64::from(v as f32).to_bits() != bits {
            return None;
        }
    }
    // FP chains are canonicalized by register exactly as integer ones are: `b + a`
    // and `b * a` emit the operands in ascending order, and every permutation of
    // `a + b + c` emits one stream. The port evaluated source order, so all of those
    // were mis-emits until the generated sweep found them.
    //
    // Division is tighter still. One division as the *only* operator is byte-exact
    // (`a / b`, `b / a` — it is non-commutative, so order is preserved), but two
    // divisions (`a / b / c`) or a division mixed with anything else (`a + b / c`)
    // are not what the serial model emits. Both refuse.
    let n_binops = ops
        .iter()
        .filter(|o| !matches!(o, IlOp::Load(_) | IlOp::Lit(_) | IlOp::FpLit { .. }))
        .count();
    if ops.iter().any(|o| matches!(o, IlOp::Div)) && n_binops != 1 {
        return None;
    }
    // A repeated leaf can trigger algebraic rewriting into a constant.
    let mut seen: Vec<u32> = Vec::new();
    for o in &ops {
        if let IlOp::Load(t) = o {
            if seen.contains(t) {
                return None;
            }
            seen.push(*t);
        }
    }
    let params = parse_params(seg, lo).ok()?;
    if params.len() > 13 || !seen.iter().all(|t| params.contains(t)) {
        return None;
    }
    // c2 canonicalizes a chain containing a **commutative** operator by register,
    // exactly as it does an integer one, so such a chain must already be written in
    // ascending order. A chain with only non-commutative operators is left alone —
    // `b - a` and `b / a` really do emit their operands in source order, and gating
    // them would refuse bodies that are byte-exact today.
    let has_commutative = ops
        .iter()
        .any(|o| matches!(o, IlOp::Add | IlOp::Mul));
    if has_commutative && !leaves_ascending(&ops, &params) {
        return None;
    }
    Some(BodyShape::FloatLeaf { params, ops, double })
}

/// The member-function `this` token, when this segment's pre-body region binds
/// one: `53 53 26 <fn> B9 <this> <TYPE> 99 <TYPE> 00 46`.
///
/// `this` is **not** in the `2D` formals list, and it occupies r3 — so every
/// explicit formal of a member function is one register higher than
/// [`parse_formals`]'s index implies. Captured, and it is a live off-by-one trap
/// for anything that maps formals to registers:
///
/// ```text
/// int C::g(int* q) const        { return *q; }   -> lwz r3,0(r4)   q is r4, not r3
/// int C::i(int v, int* q) const { return *q; }   -> lwz r3,0(r5)   q is r5, not r4
/// int D::s(int* q)              { return *q; }   -> lwz r3,0(r3)   static: no `this`
/// ```
///
/// Located against the **one** formals-marker anchor
/// ([`super::expr::formals_marker`]): the pre-body region is `26 <fn-tok>` followed
/// either by nothing or by exactly one `this` group, and whichever it is must land
/// *exactly* on that marker.
///
/// Both outcomes are established positively, and that is the point. This used to
/// return a bare `Option<u32>` and anchor on the first `0x46` byte in the segment,
/// so a `None` meant "no `this`" and "could not tell" alike — and the first `0x46`
/// is the known-bad anchor `parse_formals` documents, because a function on source
/// line 70 carries the line marker `4F 01 46`. A member function there reported no
/// `this`, every explicit formal shifted one register down, and
/// `int C::gp(int* q) const { return *q; }` emitted `lwz r3,0(r3)` where the
/// reference has `lwz r3,0(r4)` — a wrong-bytes emit inside an accepted class,
/// found by review and pinned by `fixtures/cpp/il_this_line70.cpp`.
///
/// Note that `99`'s trailing field is a one-byte varint while the visually
/// similar `9B`'s is a whole `read_token_var`; see `docs/IL_EXPR_LAYER.md` §7.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ThisBinding {
    /// The pre-body region runs straight from the function token to the formals
    /// marker: a free function or a `static` member, `this` in no register.
    Absent,
    /// A member function; the token occupies r3 and shifts every formal up one.
    Bound(u32),
}

/// `None` means **undetermined**, and the caller must refuse — never "absent".
fn parse_this_token(seg: &[u8], lo: usize) -> Option<ThisBinding> {
    let f = formals_marker(seg, lo)?;
    let mut found: Option<ThisBinding> = None;
    for q in 0..f {
        if seg[q] != 0x26 {
            continue;
        }
        let mut p = q + 1;
        let (_fn_tok, w) = match read_token_var(seg, p) {
            Some(x) => x,
            None => continue,
        };
        p += w;
        let binding = if p == f {
            ThisBinding::Absent
        } else {
            match read_this_group(seg, p) {
                Some((tok, end)) if end == f => ThisBinding::Bound(tok),
                _ => continue,
            }
        };
        // A second candidate landing on the marker means the region is not
        // determined by these bytes. Refuse rather than prefer one.
        if found.is_some() {
            return None;
        }
        found = Some(binding);
    }
    found
}

/// One `B9 <tok> <TYPE> 99 <TYPE> 00` group — the `this` push — returning its
/// token and the offset just past it.
fn read_this_group(seg: &[u8], at: usize) -> Option<(u32, usize)> {
    let mut p = at;
    if *seg.get(p)? != 0xB9 {
        return None;
    }
    p += 1;
    let (tok, w) = read_token_var(seg, p)?;
    p += w;
    let (_, _, _, tw) = read_type(seg, p)?;
    p += tw;
    if *seg.get(p)? != 0x99 {
        return None;
    }
    p += 1;
    let (_, _, _, tw) = read_type(seg, p)?;
    p += tw;
    if *seg.get(p)? != 0x00 {
        return None;
    }
    Some((tok, p + 1))
}

/// The function's **argument registers in order**: `this` when the pre-body region
/// binds one, then the `2D` formals.
///
/// Every shape that maps a token to an argument register must use this rather than
/// [`parse_formals`], and that this needed saying is the bug. `parse_this_token`
/// existed and exactly one shape consulted it, so a non-static member function with
/// a *straight-line* body mapped its first explicit formal to r3 — the register
/// `this` occupies. `struct S8 { int a; int m(int x) const; };
/// int S8::m(int x) const { return x + 1; }` emitted `Port=Mismatch @ offset 537`:
/// `addi r3,r3,1` where the reference has `addi r3,r4,1`.
///
/// That is the same defect as the line-70 `this` bug — one fact with more than one
/// locator — and it survived that fix because the fix went where the bug had been
/// found rather than everywhere the fact was used. Found by an adversarial reviewer
/// probing an unrelated change.
///
/// An undetermined `this` binding **refuses**; it never silently means "absent".
pub(crate) fn parse_params(seg: &[u8], lo: usize) -> Result<Vec<u32>, Block> {
    let formals = parse_formals(seg, lo)?;
    match parse_this_token(seg, lo) {
        Some(ThisBinding::Absent) => Ok(formals),
        Some(ThisBinding::Bound(this_tok)) => {
            let mut v = Vec::with_capacity(formals.len() + 1);
            v.push(this_tok);
            v.extend_from_slice(&formals);
            Ok(v)
        }
        None => Err(Block { ctx: "this-undetermined", byte: None, off: lo, aux: 0 }),
    }
}

/// Every LOAD in a call-argument operand stream must name a **formal**.
///
/// The multi-argument path established this positively from the start
/// (`call-arg-nonformal`); the three single-argument paths did not, so
/// `int gi; int g(int); int u1() { return g(gi); }` — a global as the argument —
/// **parsed as an in-class integer tail call**. Codegen then refused it, so no wrong
/// bytes were ever emitted, but the census counted it as in class while the gate did
/// not, which breaks the invariant this repo is built on: acceptance lives in the IL
/// parser precisely so the census and the gate cannot disagree about what is
/// accepted. A census that over-reports is a broken instrument, and the widening
/// order is chosen from it.
///
/// Found by an independent characterization agent probing the bucket, not by any
/// fixture — the corpus had no call whose argument was a global.
fn arg_loads_are_formals(arg_ops: &[IlOp], params: &[u32]) -> bool {
    arg_ops.iter().all(|o| match o {
        IlOp::Load(t) => params.contains(t),
        _ => true,
    })
}

/// Try to parse an **indirect-load leaf**: a whole body that is one load through
/// a pointer, `return *p;` / `return s->m;` / `return p[k];` and nothing else.
///
/// ```text
///   B9 <base-tok> <PTR-TYPE>                     the base pointer
///   [ 33 <int-like> <off>  27 <PTR-TYPE> ]       ONE member byte-offset add, or
///   [ 33 <long>     <off>  28 00 00      ]       ONE subscript byte-offset add
///   30 <INT4-TYPE>                               the indirect load
///   [ 2C <int-like> 00 ]                         a cv-qualification strip
///   41 <int-like>                                result type
///   <return plumbing, reaching the segment end>
/// ```
///
/// c2 lowers all of it to **one `lwz rD, off(rBase)`** plus the `blr`, folding the
/// offset into the displacement. Captured, one instruction each:
///
/// ```text
/// int f(int* p)                { return *p; }      -> lwz r3,0(r3)
/// int f(int a, int* p)         { return *p; }      -> lwz r3,0(r4)
/// int f(int a, int b, int* p)  { return *p; }      -> lwz r3,0(r5)
/// int f(S* s)                  { return s->d; }    -> lwz r3,16(r3)     (27, off 0x10)
/// int f(int* p)                { return p[3]; }    -> lwz r3,12(r3)     (28, off 0x0c)
/// int f(int* p)                { return p[-1]; }   -> lwz r3,-4(r3)     (off 0xfc = -4)
/// int f(int* p)                { return p[8000]; } -> lwz r3,32000(r3)
/// int C::f() const             { return b; }       -> lwz r3,4(r3)      (`this`)
/// unsigned/long/const/volatile int *                -> the same bare `lwz`
/// ```
///
/// Why every gate below is load-bearing rather than defensive — each is a
/// *captured* case where the same-looking IL lowers differently:
///
/// * **Exactly one offset add.** `p[i][j]` chains two of them and needs
///   `slwi ; add ; slwi ; lwzx`; `p[i].b` chains a `28` and a `27`.
/// * **The offset must fit the 16-bit displacement.** `p[100000]` (offset 400000)
///   is `lis r11,6 ; ori r11,r11,0x1a80 ; lwzx r3,r3,r11` instead.
/// * **The offset must be a literal.** A variable index is
///   `slwi r11,r4,2 ; lwzx r3,r11,r3` — a different instruction, an extra one, and
///   a scratch register.
/// * **The `28` payload must be exactly `00 00`.** Those two bytes are `00 00` at
///   every site captured (constant and variable indices, 1/4/8-byte elements,
///   negative indices, 2-D arrays, bitfields) and their meaning is UNKNOWN, so
///   anything else refuses.
/// * **The loaded type must be a 1-, 2-, 4- or 8-byte integer** ([`SIZED_PTEE`]):
///   `char *` is `lbz`, `short *` is `lhz`, `long long *` is `ld`, `float *` is
///   `lfs`, `double *` is `lfd` — all captured, all different instructions, and
///   the FP ones are still refused.
/// * **Nothing may follow the load but the return.** `*p + 1` puts the load in
///   r11, and `*p * 3` is strength-reduced; see [`IlOp::LoadInd`].
/// * **A `this`-bearing function must have its `this` found**, because `this`
///   takes r3 and shifts every explicit formal up one
///   ([`parse_this_token`]).
///
/// Returns `None` — cursor untouched — for anything that is not exactly this
/// shape.
pub(crate) fn try_parse_indirect_load_leaf(seg: &[u8], start: usize, lo: usize) -> Option<BodyShape> {
    // A member inherited from a **base class** does not use the `27` offset-add
    // below; c1xx routes it through intrinsic **2117 `base-member-addr`**, which
    // computes the same address and is the single largest decode bucket in the
    // family (6.3% of blocked functions). Try that designator first — it is
    // anchored on a literal `33`, where the plain form is anchored on `B9`, so the
    // two cannot be confused.
    if let Some(shape) = try_parse_base_member_load(seg, start, lo) {
        return Some(shape);
    }
    let mut p = start;

    // The base pointer LOAD.
    if !eat_byte(seg, &mut p, 0xB9) {
        return None;
    }
    let (base_tok, w) = read_token_var(seg, p)?;
    p += w;
    let (tag, kind, _, tw) = read_type(seg, p)?;
    if !is_ptr_to_4(tag, kind) {
        return None;
    }
    p += tw;

    // At most ONE byte-offset add, in either of its two forms. Both push a
    // byte offset as a literal and add it to the designator; `27` re-types the
    // result and `28` does not (`docs/IL_EXPR_LAYER.md` §4).
    let mut off: i32 = 0;
    // What the byte-offset add says the pointee's width is, when it says anything.
    // `27` re-types the address and its tag carries the POINTEE width, so it is a
    // second, independent statement of the width the `30` load will announce; the
    // two are required to agree. `28` and the bare deref say nothing, and then the
    // `30` type is the only evidence.
    let mut ptee_width: Option<u8> = None;
    if *seg.get(p)? == 0x33 {
        let mut probe = p + 1;
        // The literal's own type: `86 41 74` (int) for a member offset,
        // `86 41 12` (long) for a subscript offset. Both are int-like.
        if !eat_int_like(seg, &mut probe) {
            return None;
        }
        let k = read_varint(seg, &mut probe)?;
        match *seg.get(probe)? {
            0x27 => {
                probe += 1;
                let (tag, kind, _, tw) = read_type(seg, probe)?;
                ptee_width = if is_ptr_to_4(tag, kind) {
                    Some(4)
                } else {
                    // A pointer to a 1-, 2- or 8-byte object; captured with and
                    // without the tag's const bit ([`SIZED_PTR`]).
                    Some(sized_ptr_width(tag, kind)?)
                };
                probe += tw;
            }
            0x28 => {
                // The two trailing bytes are `00 00` at every captured site and
                // are not understood; anything else refuses.
                probe += 1;
                if !eat(seg, &mut probe, &[0x00, 0x00]) {
                    return None;
                }
            }
            _ => return None,
        }
        off = k;
        p = probe;
    }
    finish_indirect_load_of(seg, p, lo, base_tok, off, ptee_width)
}

/// Pointee TYPEs admitted at the `30` indirect-load position **beyond** the
/// 4-byte integer class ([`is_int4_type`]), as `(tag, kind, width, signed)`.
///
/// Required **literally, as pairs**, rather than computed from the tag's
/// width nibble: the width is stated twice in a TYPE — the tag's low nibble and
/// the kind's high nibble — and demanding both is a free discriminator against a
/// misaligned read landing on a plausible-looking byte. Every pair below has a
/// capture in `fixtures/cpp/w12_narrow_getters.cpp` (see that file's header for
/// the per-case witness); a tag not listed — notably `volatile` (`92`/`94`/`98`),
/// which no probe produced — refuses rather than being assumed to behave like the
/// `const` one.
///
/// `signed` is the pointee's own signedness (kind's low nibble 1 vs 2), which
/// matters only when a `2C` widens the value to `int`: an unsigned narrow load is
/// already zero-extended by `lbz`/`lhz`, a signed one is not.
const SIZED_PTEE: &[(u8, u8, u8, bool)] = &[
    (0x82, 0x11, 1, true),  // char / signed char        `30 82 11 70` / `… 10`
    (0xA2, 0x11, 1, true),  // const char                `30 a2 11 8e 20`
    (0x82, 0x12, 1, false), // unsigned char / bool      `30 82 12 20` / `… 30`
    (0xA2, 0x12, 1, false), // const unsigned char/bool  `30 a2 12 95 20`
    (0x84, 0x21, 2, true),  // short                     `30 84 21 11`
    (0xA4, 0x21, 2, true),  // const short               `30 a4 21 99 20`
    (0x84, 0x22, 2, false), // unsigned short / wchar_t  `30 84 22 21` / `… 71`
    (0xA4, 0x22, 2, false), // const unsigned short/wchar_t `30 a4 22 9b 20`
    (0x88, 0x81, 8, true),  // long long                 `30 88 81 13`
    (0xA8, 0x81, 8, true),  // const long long           `30 a8 81 9f 20`
    (0x88, 0x82, 8, false), // unsigned long long        `30 88 82 23`
    (0xA8, 0x82, 8, false), // const unsigned long long  `30 a8 82 … 20`
];

/// `(tag, kind)` of a **pointer whose tag carries the pointee's width** — the
/// shape the `27` byte-offset-add position uses (`27 82 43 f0 08` for `char *`,
/// `27 a4 43 9a 20` for `const short *`, `27 a8 43 a0 20` for `const long long *`).
/// The tag's const bit here does **not** track the loaded type's: a *non*-const
/// member function's getter carries `27 a2 43 f0 08` over a `30 82 11 70`
/// (`D::n_c()`), so both tags are listed for each width and neither implies
/// anything about the load.
const SIZED_PTR: &[(u8, u8, u8)] = &[
    (0x82, 0x43, 1),
    (0xA2, 0x43, 1),
    (0x84, 0x43, 2),
    (0xA4, 0x43, 2),
    (0x88, 0x43, 8),
    (0xA8, 0x43, 8),
];

/// `(width, signed)` of a [`SIZED_PTEE`] pair, or `None` — which is a refusal,
/// never "assume 4".
fn sized_ptee(tag: u8, kind: u8) -> Option<(u8, bool)> {
    SIZED_PTEE
        .iter()
        .find(|&&(t, k, _, _)| t == tag && k == kind)
        .map(|&(_, _, w, s)| (w, s))
}

/// Pointee width of a [`SIZED_PTR`] pair, or `None`.
fn sized_ptr_width(tag: u8, kind: u8) -> Option<u8> {
    SIZED_PTR
        .iter()
        .find(|&&(t, k, _)| t == tag && k == kind)
        .map(|&(_, _, w)| w)
}

/// The tail shared by both indirect-load designators (the plain `27`/`28` offset
/// add and the intrinsic-2117 `base-member-addr` form): the `30` load, an optional
/// cv strip, the result type, the return plumbing, and binding the base pointer to
/// its argument register.
///
/// Factored out rather than duplicated because the two designators compute the
/// *same* address by different routes and must lower identically; two copies of
/// this tail would be two places for the `lwz` displacement bound to drift.
fn finish_indirect_load(
    seg: &[u8],
    p: usize,
    lo: usize,
    base_tok: u32,
    off: i32,
) -> Option<BodyShape> {
    finish_indirect_load_of(seg, p, lo, base_tok, off, Some(4))
}

/// [`finish_indirect_load`] with the pointee width the *designator* announced, if
/// it announced one (`Some(4)` = "a 4-byte object, and nothing else will do").
///
/// T3 widens the load type from "a 4-byte integer" to any [`SIZED_PTEE`] scalar,
/// which is where the two designators stop being interchangeable: the plain `27`
/// form re-types the address with the pointee's width and so *knows* it, while the
/// intrinsic-2117 base-member form was captured only over 4-byte members and pins
/// itself to 4 by passing `Some(4)`.
fn finish_indirect_load_of(
    seg: &[u8],
    mut p: usize,
    lo: usize,
    base_tok: u32,
    off: i32,
    ptee_width: Option<u8>,
) -> Option<BodyShape> {
    if !(-0x8000..=0x7FFF).contains(&off) {
        return None;
    }

    // The indirect load itself.
    if !eat_byte(seg, &mut p, 0x30) {
        return None;
    }
    let (tag, kind, _, tw) = read_type(seg, p)?;
    let load_op = if is_int4_type(tag, kind) {
        if matches!(ptee_width, Some(w) if w != 4) {
            return None;
        }
        p += tw;

        // An optional cv-qualification strip. Provably free over a 4-byte integer
        // source (see [`is_int4_type`]); the target must still be int-like, and the
        // trailing varint must be the `00` observed at all 14,098 aligned sites.
        if *seg.get(p)? == 0x2C {
            let mut probe = p + 1;
            if !eat_int_like(seg, &mut probe) || !eat_byte(seg, &mut probe, 0x00) {
                return None;
            }
            p = probe;
        }

        // Result type.
        if !eat_byte(seg, &mut p, 0x41) || !eat_int_like(seg, &mut p) {
            return None;
        }
        IlOp::LoadInd { off }
    } else {
        let (width, signed) = sized_ptee(tag, kind)?;
        if matches!(ptee_width, Some(w) if w != width) {
            return None;
        }
        // `ld` is DS-form: the displacement's low two bits are the form's, so an
        // offset that is not a multiple of 4 cannot be encoded at all. Natural
        // alignment makes one unreachable through a struct member, so this gate has
        // no witness — which is exactly why it refuses instead of masking.
        if width == 8 && off % 4 != 0 {
            return None;
        }
        p += tw;

        // The optional conversion. Two — and only two — targets are captured over a
        // narrow load, and they are not the same thing:
        //
        //  * the **same** width and signedness (a cv strip: `30 a2 11 93 20`
        //    `2c 82 11 70 00`) emits nothing, exactly as at width 4;
        //  * **exactly `int`** (`2c 86 41 74 00`) is a real widening. Free for an
        //    unsigned pointee (`lbz`/`lhz` zero-extend), one extra `extsb` for a
        //    signed byte, and **mode-dependent** for a signed halfword — `/O1`
        //    emits `lha r3` where `/Ox` and `/O2` emit `lhz r11 ; extsh r3,r11`
        //    (measured, both ways, `docs/IL_LOAD_TYPES.md` §3 states only the `/Ox`
        //    form). This lowering path has no optimization-mode parameter, so the
        //    signed-halfword widening is refused rather than guessed at.
        //
        // Anything else — `unsigned int` as the target (whose emit is measured
        // identical to `int`'s, but whose (source × target) matrix is not),
        // a widening of a `long long`, a narrowing, a change of signedness at the
        // same width — refuses. Each would need its own capture set.
        let mut sext = false;
        let mut int_result = false;
        if *seg.get(p)? == 0x2C {
            let mut probe = p + 1;
            let (t2, k2, _, tw2) = read_type(seg, probe)?;
            if eat(seg, &mut probe, &INT_TYPE) {
                if width != 1 && !(width == 2 && !signed) {
                    return None;
                }
                sext = signed;
                int_result = true;
            } else if sized_ptee(t2, k2) == Some((width, signed)) {
                probe += tw2;
            } else {
                return None;
            }
            if !eat_byte(seg, &mut probe, 0x00) {
                return None;
            }
            p = probe;
        }

        // Result type: the value's type after the conversion, stated again. Every
        // capture agrees byte for byte, so this is required rather than skipped.
        if !eat_byte(seg, &mut p, 0x41) {
            return None;
        }
        if int_result {
            if !eat(seg, &mut p, &INT_TYPE) {
                return None;
            }
        } else {
            let (t3, k3, _, tw3) = read_type(seg, p)?;
            if sized_ptee(t3, k3) != Some((width, signed)) {
                return None;
            }
            p += tw3;
        }
        IlOp::LoadIndSized { off, width, sext }
    };
    // The shared plumbing, which must reach the segment end.
    eat_return_plumbing(seg, &mut p, false, BODY_SCOPE_DEPTH).ok()?;

    // Bind the base to its argument register. `this` is argument 0, and when it
    // is present every explicit formal shifts up one.
    let formals = parse_formals(seg, lo).ok()?;
    // `None` is "undetermined", and refusing is the whole point: treating it as
    // "no `this`" is what mis-emitted the base register.
    let params = match parse_this_token(seg, lo)? {
        ThisBinding::Bound(this_tok) => {
            let mut v = vec![this_tok];
            v.extend_from_slice(&formals);
            v
        }
        ThisBinding::Absent => formals,
    };
    let ix = params.iter().position(|&t| t == base_tok)?;
    // Past the eighth argument the value is stack-homed, which needs a frame.
    if ix >= 8 {
        return None;
    }
    Some(BodyShape::IndirectLoad {
        params,
        ops: vec![IlOp::Load(base_tok), load_op],
    })
}

/// The intrinsic-**2117** (`base-member-addr`) designator: reading a member that
/// the object inherits from a non-virtual base.
///
/// `p->d` for a member declared directly in `D` uses the ordinary `27` offset-add
/// in [`try_parse_indirect_load_leaf`]. A member inherited from a base does not —
/// c1xx emits an intrinsic call whose three arguments are `(0, base_offset, p)`:
///
/// ```text
///   33 <int> 80 45 08 00 00       LITERAL 2117 — the selector, always the wide form
///   40 <ptr>                      intrinsic call, result type a pointer
///   66 <n> <n × 2-byte type ref>  argument-region header (see below)
///   55 <int>                      selector argument terminator
///   33 <int> <m>     55 <int>     arg 1 — the member's offset WITHIN THE BASE
///   33 <int> <b>     55 <int>     arg 2 — the BASE's offset within the object
///   B9 <tok> <ptr>   55 <ptr>     arg 3 — the object pointer
///   4C                            call end
/// ```
///
/// **The address is `p + m + b`** — the two literals *add*. Established by the
/// witness that separates a sum from "whichever one is nonzero", since every
/// simpler case has one of them zero:
///
/// ```text
///   struct A { int a0, a1; }; struct B { int b0, b1, b2; }; struct D : A, B {};
///   p->b2   args (8, 8)   ->  lwz r3,0x10(r3)     16 = 8 + 8   <- both nonzero
///   p->b0   args (0, 8)   ->  lwz r3,8(r3)
///   p->a1   args (4, 0)   ->  lwz r3,4(r3)
/// ```
///
/// then the identical `30`/`41`/return tail as the `27` form, which is why this
/// hands back the same [`BodyShape::IndirectLoad`] and needs no new codegen: the
/// address is `p + off`, and the load of it is `lwz rD, off(rB)` either way.
/// Verified against captures —
///
/// ```text
///   struct A { int a; }; struct B { int b; }; struct D : A, B { int d; };
///   p->a  (A at 0)   80630000   lwz r3,0(r3) ; blr      <- 2117, off 0
///   p->b  (B at 4)   80630004   lwz r3,4(r3) ; blr      <- 2117, off 4
///   p->d  (at 8)     80630008   lwz r3,8(r3) ; blr      <- the `27` form
/// ```
///
/// — so the port already emitted the third of those byte-exactly and refused the
/// first two purely on the decode.
///
/// Fail-closed specifics:
///
/// * the selector must be **exactly** 2117 in its wide five-byte form. 2113–2119
///   are seven different operations and only this one is an unguarded
///   `base + constant`; 2114/2115 add a null guard, 2116/2118 go through a
///   vbtable, 2119 is a runtime call (`docs/IL_INTRINSIC_CALL.md`).
/// * the argument-region header is `66 <n>` followed by *n* two-byte type
///   references, and is skipped **structurally** rather than matched as a
///   constant. `n` is 2 for a single inheritance step and 3 for two
///   (`struct E : D`, `D : A, B` — `66 03 89 20 83 20 80 20`), so a fixed
///   six-byte match silently refused every multi-level case. The refs themselves
///   are not decoded; nothing downstream needs them, and the *value* arguments
///   that follow are what carry the address.
/// * the two offsets are summed with `checked_add`, and the sum must fit the `lwz`
///   16-bit displacement — checked by the shared tail, so a class large enough to
///   overflow it refuses instead of wrapping.
fn try_parse_base_member_load(seg: &[u8], start: usize, lo: usize) -> Option<BodyShape> {
    /// `33 <int-like> 80 45 08 00 00` — the selector literal, wide form.
    const SELECTOR_2117: [u8; 5] = [0x80, 0x45, 0x08, 0x00, 0x00];
    /// Longest argument-header type list accepted. Two witnesses (`n` = 2 and 3)
    /// bound what is understood; a deeper list is refused rather than skipped on
    /// the assumption that the shape keeps repeating.
    const MAX_HEADER_REFS: u8 = 3;

    let mut p = start;
    // The selector, pushed as an int literal.
    if !eat_byte(seg, &mut p, 0x33) || !eat_int_like(seg, &mut p) || !eat(seg, &mut p, &SELECTOR_2117)
    {
        return None;
    }
    // The intrinsic-call marker; its result is the member's address.
    if !eat_byte(seg, &mut p, 0x40) {
        return None;
    }
    let (tag, kind, _, tw) = read_type(seg, p)?;
    if !is_ptr_to_4(tag, kind) {
        return None;
    }
    p += tw;
    // The argument header: `66 <n>` then n two-byte type references, skipped
    // structurally so a second inheritance step (n = 3) parses like the first.
    if !eat_byte(seg, &mut p, 0x66) {
        return None;
    }
    let n_refs = *seg.get(p)?;
    if n_refs == 0 || n_refs > MAX_HEADER_REFS {
        return None;
    }
    p += 1 + 2 * n_refs as usize;
    if p > seg.len() {
        return None;
    }
    // Each argument is `<value> 55 <its type>`.
    if !eat_byte(seg, &mut p, 0x55) || !eat_int_like(seg, &mut p) {
        return None;
    }
    // arg 1 — the member's offset within its base.
    if !eat_byte(seg, &mut p, 0x33) || !eat_int_like(seg, &mut p) {
        return None;
    }
    let member_off = read_varint(seg, &mut p)?;
    if !eat_byte(seg, &mut p, 0x55) || !eat_int_like(seg, &mut p) {
        return None;
    }
    // arg 2 — the base's offset within the object. The address is the sum.
    if !eat_byte(seg, &mut p, 0x33) || !eat_int_like(seg, &mut p) {
        return None;
    }
    let base_off = read_varint(seg, &mut p)?;
    let off = member_off.checked_add(base_off)?;
    if !eat_byte(seg, &mut p, 0x55) || !eat_int_like(seg, &mut p) {
        return None;
    }
    // arg 3 — the object pointer.
    if !eat_byte(seg, &mut p, 0xB9) {
        return None;
    }
    let (base_tok, w) = read_token_var(seg, p)?;
    p += w;
    let (tag, kind, _, tw) = read_type(seg, p)?;
    if !is_ptr_to_4(tag, kind) {
        return None;
    }
    p += tw;
    if !eat_byte(seg, &mut p, 0x55) {
        return None;
    }
    let (tag, kind, _, tw) = read_type(seg, p)?;
    if !is_ptr_to_4(tag, kind) {
        return None;
    }
    p += tw;
    if !eat_byte(seg, &mut p, 0x4C) {
        return None;
    }
    finish_indirect_load(seg, p, lo, base_tok, off)
}

/// Try to parse a **W6 comparison leaf** body: `return <formal> <rel> <k>;`.
///
/// ```text
///   B9 <tok> <T>        LOAD the formal          T ∈ {int, unsigned}
///   33 <T> <varint>     LITERAL k, same type T
///   <rel>               1F|20|21|22|23|24
///   2C <R> 00           convert bool → R         R ∈ {int, unsigned}
///   41 <R>              result type
///   <return plumbing>
/// ```
///
/// Fail-closed specifics that are load-bearing rather than incidental:
///
/// * The two operand types must be **equal**. c1xx always inserts a conversion
///   first, so a mismatch has never been observed; rejecting it is a cheap
///   assertion, not a dropped feature.
/// * The `2C` convert is accepted **only here**, directly over a comparison
///   result. The identical token over a narrow-integer LOAD is a real
///   `extsb`/`extsh` sign-extension, so a blanket "`2C` is free" rule would
///   silently drop those instructions.
/// * The parse must reach the segment end via the shared return plumbing, so a
///   trailing statement, a second comparison, or an arithmetic post-op (e.g.
///   `return (a > 7) + 1;`, which retargets the spine's last instruction) all
///   reject the whole function.
///
/// Returns `None` — leaving the caller's cursor untouched — for anything that is
/// not exactly this shape.
pub(crate) fn try_parse_compare(seg: &[u8], start: usize, lo: usize) -> Option<BodyShape> {
    let mut p = start;

    // LOAD <formal> <T>
    if !eat_byte(seg, &mut p, 0xB9) {
        return None;
    }
    let (param, w) = read_token_var(seg, p)?;
    p += w;
    let signed = if eat(seg, &mut p, &INT_TYPE) {
        true
    } else if eat(seg, &mut p, &UINT_TYPE) {
        false
    } else {
        return None;
    };
    let operand_type = if signed { INT_TYPE } else { UINT_TYPE };

    // LITERAL k, of the SAME type as the loaded operand.
    if !eat_byte(seg, &mut p, 0x33) || !eat(seg, &mut p, &operand_type) {
        return None;
    }
    let k = read_varint(seg, &mut p)?;

    // The relational opcode.
    let rel = Rel::from_opcode(*seg.get(p)?)?;
    p += 1;

    // `2C <R> 00` — convert the bool result to the return type.
    if !eat_byte(seg, &mut p, 0x2C) {
        return None;
    }
    let ret_is_int = if eat(seg, &mut p, &INT_TYPE) {
        true
    } else if eat(seg, &mut p, &UINT_TYPE) {
        false
    } else {
        return None;
    };
    if !eat_byte(seg, &mut p, 0x00) {
        return None;
    }

    // Result type + the shared return plumbing, which must reach the segment end.
    let ret_type = if ret_is_int { INT_TYPE } else { UINT_TYPE };
    if !eat_byte(seg, &mut p, 0x41) || !eat(seg, &mut p, &ret_type) {
        return None;
    }
    // Result type already consumed above, so `has_result_type` is false here.
    eat_return_plumbing(seg, &mut p, false, BODY_SCOPE_DEPTH).ok()?;

    // The compared value must be the function's FIRST formal: the spine reads it
    // from r3, and nothing here models a register move.
    let params = parse_params(seg, lo).ok()?;
    if params.first() != Some(&param) || params.len() != 1 {
        return None;
    }

    // Gates moved here from `compare_leaf_text`, so the census counts only what the
    // emitter can emit. A literal outside the signed 16-bit immediate needs
    // `lis`+`ori` and the extra temp slot that consumes; and `==`/`!=` form `a - k`
    // as `addi r11,a,-k`, so at `k == i16::MIN` the negation itself overflows.
    if i16::try_from(k).is_err() {
        return None;
    }
    if matches!(rel, Rel::Eq | Rel::Ne) && k == i32::from(i16::MIN) {
        return None;
    }
    Some(BodyShape::Compare(CompareLeaf { param, rel, signed, k }))
}

/// Parse a call shape (already positioned at the `26 <tok>` function ref): the
/// bare terminal void call, an integer tail call `return g(<arg>)` (passthrough
/// or arg-setup, plus the `g(a)+0` identity fold), or the framed
/// `return g(a) + k` (k ≠ 0). See [`parse_segment`] for the grammar;
/// fail-closed at every step. `lo` locates the formals for the arg-setup.
pub(crate) fn parse_call_shape(
    seg: &[u8],
    p: &mut usize,
    lo: usize,
    bound_to: Option<u32>,
) -> Result<BodyShape, Block> {
    // 26 <tok> function/result ref.
    if !eat_byte(seg, p, 0x26) {
        return Err(blk(seg, *p, "call-ref"));
    }
    // The `26 <tok>` symbol push NAMES THE CALLEE. The CALL token that follows
    // carries only a function-*type* id, so this token is the only thing that
    // distinguishes one callee from another; it is resolved through the `.gl`
    // symbol index (see `gl_symbol_index`).
    let (callee_tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, "call-ref-tok"))?;
    *p += w;
    // The CALL token: `BD <TYPE ret> <flags> <varint fn-type-id>`. Nothing in it
    // is fixed but the `BD` — it is 8 to 13 bytes and self-delimiting field by
    // field, so it is decoded rather than matched.
    //
    // This replaces a hardcoded 6-byte "callee anchor" `00 80 01 10 00 00`,
    // which was never an anchor: it is `flags = 0` followed by the varint
    // `0x1001`, and `0x1001` is merely the first function type a single-function
    // fixture TU happens to create. True of every MVP fixture and of almost
    // nothing else — which is precisely what the `call-anchor-*` census buckets
    // were measuring.
    if !eat_byte(seg, p, 0xBD) {
        // `26 <sym>` followed by an INTRINSIC CALL rather than a `BD`. This is the
        // other half of the `0x40` production's footprint and it was the whole of
        // the `call-token-0x33` census bucket (7.4 % of blocked functions): a
        // member call whose `this` is an adjusted base pointer opens
        // `26 <method> 33 86 41 74 <2113> 40 …`, and an intrinsic result stored to
        // a symbol opens `26 <dest> 33 86 41 74 <id> 40 …`. Reported with the
        // selector so the two footprints can be summed; still `Err`, so the gate
        // is unchanged.
        if let Some(id) = intrinsic_selector(seg, *p) {
            return Err(Block {
                ctx: "call-intrinsic",
                byte: Some(0x40),
                off: *p,
                aux: id as u32,
            });
        }
        return Err(blk(seg, *p, "call-token"));
    }
    let (_, _, _, ret_w) = read_type(seg, *p).ok_or(blk(seg, *p, "call-ret-type"))?;
    *p += ret_w;
    // Calling convention: 0x00 = cdecl/stdcall, 0x04 = fastcall, 0x40 = varargs.
    // Only cdecl is in class — the others need argument-passing the port does
    // not implement, and accepting them would mis-emit rather than refuse.
    match seg.get(*p) {
        Some(0x00) => *p += 1,
        _ => return Err(blk(seg, *p, "call-conv")),
    }
    // The function-type id. NOT the callee: three different callees sharing one
    // signature produce byte-identical CALL tokens. The callee is bound from the
    // `26 <tok>` symbol push instead, so this field is decoded only to find the
    // token's end, then discarded.
    read_varint(seg, p).ok_or(blk(seg, *p, "call-fn-type-id"))?;

    // VOID terminal tail call: the `4C 4B` void call-end immediately follows the
    // CALL token (no argument setup, no consumed value), then only return
    // plumbing (no result type). `g();g();` and `g();return a+1;` fail here — a
    // second `26` call or a `B9` statement stands where the return plumbing must.
    if eat(seg, p, &[0x4C, 0x4B]) {
        eat_return_plumbing(seg, p, false, BODY_SCOPE_DEPTH)?;
        return Ok(BodyShape::VoidTailCall { callee_tok });
    }

    // INT call. The argument region is a **repetition**, not a single argument:
    //
    //     args := ( expr `55` <TYPE> )*  `4C`
    //
    // Each argument is a modeled sub-expression — a passthrough `B9 a INT`
    // (→ `[Load]`) or an arg-setup like `a + 1` (→ `[Load, Lit, Add]`) — followed
    // by `55 <TYPE>` carrying the *formal's* declared type, and the whole list is
    // terminated by `4C`. Arguments appear in **reverse source order**, rightmost
    // first (anchored on `parse_formals`, which reverses the `2D` stream so
    // `params[0]` is its last token; `fixtures/cpp/il_call_args2.cpp` holds the
    // `g2(a,b)` / `g2(b,a)` pair that separates the two readings).
    //
    // This used to accept exactly one argument, so every real call site blocked at
    // the second `B9` — the largest single census bucket.
    let mut args: Vec<Vec<IlOp>> = Vec::new();
    loop {
        if eat_byte(seg, p, 0x4C) {
            break;
        }
        let ops = parse_expr(seg, p, 0x55)?;
        if !eat_byte(seg, p, 0x55) || !eat_int_like(seg, p) {
            // an argument whose terminator or formal type we do not model
            return Err(blk(seg, *p, "call-end"));
        }
        args.push(ops);
        if args.len() > 8 {
            // Past the eighth the arguments are stack-homed, which needs a frame.
            return Err(Block { ctx: "call-args-overflow", byte: None, off: *p, aux: 0 });
        }
    }
    if args.is_empty() {
        // A zero-argument int call (`return g();`). The value-consuming shapes
        // below all assume an argument region, so refuse rather than guess.
        return Err(Block { ctx: "call-args-none", byte: None, off: *p, aux: 0 });
    }
    // A call whose result is bound to a local that is then returned immediately —
    // `int z = g(a); return z;` — is byte-identical to `return g(a);`. c2
    // register-allocates the local and coalesces the copy, so both are a bare
    // `b <callee>`; captured on the one-, two- and three-argument forms.
    //
    // This is the `expr-call-in-expr` census bucket, and after the gate migration it
    // is the largest single blocker at 12.3% of blocked functions. It needs no new
    // codegen at all — only the IL model — so it routes to the existing tail-call
    // productions rather than growing a shape of its own.
    //
    // The local never becomes a memory object here, which is why this does not
    // reopen the store question `il_stmt_static.cpp` closed: the value is returned,
    // never written anywhere, and the shape below admits nothing between the store
    // and the return.
    if let Some(dst) = bound_to {
        //  32 <TYPE> 4B          store the call result into `dst`, discard the value
        //  [4F 01 <line>]*       a line change between the two statements
        //  B9 <dst> <TYPE> 41    load it straight back and return it
        if !eat_byte(seg, p, 0x32) || !eat_int_like(seg, p) {
            return Err(blk(seg, *p, "call-bound-store"));
        }
        if !eat_byte(seg, p, 0x4B) {
            return Err(blk(seg, *p, "call-bound-stmt-end"));
        }
        eat_opt_stmt_marker(seg, p);
        if !eat_byte(seg, p, 0xB9) {
            return Err(blk(seg, *p, "call-bound-reload"));
        }
        let (back, w) =
            read_token_var(seg, *p).ok_or(blk(seg, *p, "call-bound-reload-tok"))?;
        *p += w;
        // Anything other than reading back the very token just written is a
        // different program.
        if back != dst {
            return Err(Block { ctx: "call-bound-other-token", byte: None, off: *p, aux: 0 });
        }
        if !eat_int_like(seg, p) {
            return Err(blk(seg, *p, "call-bound-reload-type"));
        }
        eat_return_plumbing(seg, p, true, BODY_SCOPE_DEPTH)?;
        let params = parse_params(seg, lo)?;
        if args.len() > 1 {
            let mut arg_sources = Vec::with_capacity(args.len());
            for slot in 0..args.len() {
                let ops = &args[args.len() - 1 - slot];
                let tok = match ops.as_slice() {
                    [IlOp::Load(t)] => *t,
                    _ => {
                        return Err(Block {
                            ctx: "call-arg-computed",
                            byte: None,
                            off: *p,
                            aux: 0,
                        })
                    }
                };
                match params.iter().position(|&t| t == tok) {
                    Some(ix) => arg_sources.push(ix),
                    None => {
                        return Err(Block {
                            ctx: "call-arg-nonformal",
                            byte: None,
                            off: *p,
                            aux: 0,
                        })
                    }
                }
            }
            for (i, src) in arg_sources.iter().enumerate() {
                if arg_sources[..i].contains(src) {
                    return Err(Block {
                        ctx: "call-arg-duplicated",
                        byte: None,
                        off: *p,
                        aux: 0,
                    });
                }
            }
            let n = arg_sources.len();
            let mut seen = vec![false; n];
            let mut cycles = 0usize;
            for start in 0..n {
                if seen[start] || arg_sources[start] == start {
                    seen[start] = true;
                    continue;
                }
                let mut at = start;
                while !seen[at] {
                    seen[at] = true;
                    at = arg_sources[at];
                }
                cycles += 1;
            }
            if cycles > 1 {
                return Err(Block { ctx: "call-arg-multicycle", byte: None, off: *p, aux: 0 });
            }
            return Ok(BodyShape::MultiArgTailCall { params, arg_sources, callee_tok });
        }
        let arg_ops = args.pop().expect("exactly one argument");
        if has_repeated_leaf(&arg_ops) {
            return Err(Block { ctx: "call-arg-repeated-leaf", byte: None, off: *p, aux: 0 });
        }
        if !additive_chain_canonical(&arg_ops) {
            return Err(Block { ctx: "call-arg-noncanonical-order", byte: None, off: *p, aux: 0 });
        }
        if !arg_loads_are_formals(&arg_ops, &params) {
            return Err(Block { ctx: "call-arg-nonformal", byte: None, off: *p, aux: 0 });
        }
        return Ok(BodyShape::IntTailCall { params, arg_ops, callee_tok });
    }
    if args.len() > 1 {
        // Two or more arguments: only the pure-permutation shape is modeled, and
        // only as a tail call. Every argument must be a bare parameter LOAD — a
        // computed argument would need its own register and interacts with the
        // permutation temp in ways no capture covers yet.
        let params = parse_params(seg, lo)?;
        let mut arg_sources = Vec::with_capacity(args.len());
        // Stream order is reverse source order, so slot `i` is stream `n-1-i`.
        for slot in 0..args.len() {
            let ops = &args[args.len() - 1 - slot];
            let tok = match ops.as_slice() {
                [IlOp::Load(t)] => *t,
                _ => {
                    return Err(Block {
                        ctx: "call-arg-computed",
                        byte: None,
                        off: *p,
                        aux: 0,
                    })
                }
            };
            match params.iter().position(|&t| t == tok) {
                Some(ix) => arg_sources.push(ix),
                // An argument that is not one of this function's formals (a local,
                // a global, a nested call result).
                None => {
                    return Err(Block { ctx: "call-arg-nonformal", byte: None, off: *p, aux: 0 })
                }
            }
        }
        // The two permutation shapes codegen cannot lower are rejected HERE rather
        // than there, so the census and the emission gate cannot disagree about
        // what is in class (the same reason the FP contraction and constant gates
        // live in this file). Both are captured in `fixtures/cpp/il_call_multi.cpp`
        // and explained at `c2_core::codegen::permute_args_text`.
        //
        // A value passed twice: c2 emits a dead `mr` through the temp, which no
        // live-value-driven solver produces.
        for (i, s) in arg_sources.iter().enumerate() {
            if arg_sources[..i].contains(s) {
                return Err(Block { ctx: "call-arg-duplicated", byte: None, off: *p, aux: 0 });
            }
        }
        // Two or more disjoint cycles: c2 hoists every save (r11, then r10) and
        // then has several clobber-free orders to choose between, which the one
        // available capture does not pin down.
        {
            let n = arg_sources.len();
            let mut seen = vec![false; n];
            let mut cycles = 0usize;
            for start in 0..n {
                if seen[start] || arg_sources[start] == start {
                    seen[start] = true;
                    continue;
                }
                let mut at = start;
                while !seen[at] {
                    seen[at] = true;
                    at = arg_sources[at];
                }
                cycles += 1;
            }
            if cycles > 1 {
                return Err(Block { ctx: "call-arg-multicycle", byte: None, off: *p, aux: 0 });
            }
        }
        // Only a terminal tail call: a post-op would consume the result and need
        // the framed path, which does not model argument setup at all.
        if seg.get(*p) != Some(&0x41) {
            return Err(Block { ctx: "call-multiarg-postop", byte: None, off: *p, aux: 0 });
        }
        eat_return_plumbing(seg, p, true, BODY_SCOPE_DEPTH)?;
        return Ok(BodyShape::MultiArgTailCall { params, arg_sources, callee_tok });
    }
    let arg_ops = args.pop().expect("exactly one argument");
    // The single call argument is an ordinary operand stream, so it is subject to
    // the same rewriter: `g(a + a)` is not `add` + branch.
    if has_repeated_leaf(&arg_ops) {
        return Err(Block { ctx: "call-arg-repeated-leaf", byte: None, off: *p, aux: 0 });
    }
    // And to the same reassociation: `g(b + a)` is not the source order either.
    //
    // "The framed-call class carries no formals" is what this comment used to say,
    // and it was FALSE. It came from `MVP_FRAMED`, a pinned segment truncated at the
    // `LO` marker: a real `int f(int a) { return g(a) + 1; }` segment carries
    // `46 2D E5 09` like every other. The fixture omitted the region and the comment
    // inferred a property of the compiler from the omission — see `docs/GAPS.md` §6,
    // a truncated fixture cannot witness the region it omits. The pinned segments now
    // carry their real `53 53 26 <fn> 46 2D <formal>` prologue.
    //
    // The ordering gate is still skipped for a single operand, because it is vacuous
    // there — one leaf cannot be out of order — not because there are no formals.
    let n_loads = arg_ops.iter().filter(|o| matches!(o, IlOp::Load(_))).count();
    if n_loads > 1 {
        let formals = parse_params(seg, lo)?;
        if !leaves_ascending(&arg_ops, &formals) {
            return Err(Block { ctx: "call-arg-noncanonical-order", byte: None, off: *p, aux: 0 });
        }
    }
    if !additive_chain_canonical(&arg_ops) {
        return Err(Block { ctx: "call-arg-noncanonical-order", byte: None, off: *p, aux: 0 });
    }

    // Post-op region. EITHER the return plumbing begins directly at its `41`
    // result-type marker (no post-op → an integer tail call `return g(<arg>)`),
    // OR exactly one literal `33 <int> k` + ADD (`return g(a) + k`, framed).
    if seg.get(*p) == Some(&0x41) {
        // No post-op → integer tail call: compute the argument into r3, then
        // `b <callee>` (5-section leaf). The int analog of the void tail call;
        // `g(a)` is a bare `b g`, `g(a+1)` prepends `addi r3,r3,1`.
        eat_return_plumbing(seg, p, true, BODY_SCOPE_DEPTH)?;
        let params = parse_params(seg, lo)?;
        if !arg_loads_are_formals(&arg_ops, &params) {
            return Err(Block { ctx: "call-arg-nonformal", byte: None, off: *p, aux: 0 });
        }
        return Ok(BodyShape::IntTailCall { params, arg_ops, callee_tok });
    }
    // Post-op `+ k`: EXACTLY one literal `33 <int> k` immediately followed by
    // ADD. A second call (`g(a)+g(1)` → `26 …`), a second literal (`g(a)+1+2` →
    // a second `33 …`), or SUB/MUL (`03`/`04`) all fail one of these `eat`s.
    if !eat_byte(seg, p, 0x33) || !eat(seg, p, &INT_TYPE) {
        return Err(blk(seg, *p, "call-postop"));
    }
    let k = read_varint(seg, p).ok_or(blk(seg, *p, "call-postop-varint"))?;
    if !eat_byte(seg, p, 0x02) {
        // non-ADD post-op → non-commutative / strength-reduced
        return Err(blk(seg, *p, "call-postop-op"));
    }
    // `k` must fit a single signed-16-bit `addi` immediate (the 0x24 frame).
    if !(-0x8000..=0x7FFF).contains(&k) {
        return Err(Block { ctx: "call-postop-wide", byte: None, off: *p, aux: 0 });
    }
    eat_return_plumbing(seg, p, true, BODY_SCOPE_DEPTH)?;

    // W4b2-vi identity fold: a net post-op of 0 is NOT a framed call. `g(a)+0`
    // == `g(a)`, and the optimizer folds it to the bare `b g` (verified: the
    // `g(a)+0` obj is byte-identical to `g(a)`'s). Route it to the integer
    // tail-call production so it takes the 5-section leaf path — never the
    // 6-section framed obj (which would mis-emit a frame the reference elides).
    if k == 0 {
        let params = parse_params(seg, lo)?;
        if !arg_loads_are_formals(&arg_ops, &params) {
            return Err(Block { ctx: "call-arg-nonformal", byte: None, off: *p, aux: 0 });
        }
        return Ok(BodyShape::IntTailCall { params, arg_ops, callee_tok });
    }
    // A genuine `+ k` (k ≠ 0) is a framed non-leaf call — but the 6-section
    // framed path models only a **bare passthrough argument** (`g(a) + k`), not
    // arg-setup. `g(a+1) + 1` (a computed argument AND a framed post-op) is out
    // of class → reject (fail closed), never a mis-emitted framed obj.
    // The framed path takes a bare passthrough LOAD, which must still be a formal:
    // `int gi; g(gi) + 1` is a global read, not an argument already in r3.
    if matches!(arg_ops.as_slice(), [IlOp::Load(_)]) {
        if !arg_loads_are_formals(&arg_ops, &parse_params(seg, lo).unwrap_or_default()) {
            return Err(Block { ctx: "call-arg-nonformal", byte: None, off: *p, aux: 0 });
        }
        return Ok(BodyShape::FramedCall { add_k: k, callee_tok });
    }
    Err(Block { ctx: "framed-computed-arg", byte: None, off: *p, aux: 0 })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::func::body::{parse_segment, parse_segment_detail};
    use crate::func::bundle::LO_MARKER;
    use crate::func::readers::find_subslice;
    use crate::func::test_fixtures::*;

    /// A call argument that is not a formal must refuse — and it must refuse in the
    /// PARSER, so the census and the gate agree about it.
    ///
    /// `int gi; int g(int); int u1() { return g(gi); }` parsed as an in-class integer
    /// tail call: the multi-argument path checked its arguments against the formals
    /// list from the start, and the three single-argument paths never did. Codegen
    /// refused it downstream, so no wrong bytes were emitted — but the census counted
    /// it in class while the gate did not, and the widening order is chosen from the
    /// census. Found by a characterization agent probing the bucket; no fixture had a
    /// call whose argument was a global.
    #[test]
    fn a_call_argument_that_is_not_a_formal_refuses_in_the_parser() {
        // `INT_TAILRET` is `return g(a);` — rebind the argument LOAD to a token that
        // is not in the `2D` formals list, changing nothing else.
        let mut global_arg = INT_TAILRET.to_vec();
        let lo = find_subslice(&global_arg, &LO_MARKER).unwrap();
        let at = global_arg[lo..]
            .windows(2)
            .position(|w| w == [0xB9, 0xE5])
            .expect("the argument LOAD")
            + lo
            + 1;
        assert_eq!(parse_segment(&free_fn(INT_TAILRET), NO_LOCALS).is_some(), true, "control");
        global_arg[at] = 0xF0; // a token no `2D` entry names
        let b = parse_segment_detail(&free_fn(&global_arg), NO_LOCALS).unwrap_err();
        assert_eq!(b.ctx, "call-arg-nonformal");
    }

    #[test]
    fn indirect_load_leaf_decodes_deref_member_and_subscript() {
        assert_eq!(
            parse_segment(&free_fn(IND_DEREF), NO_LOCALS),
            Some(BodyShape::IndirectLoad {
                params: vec![0xEE09],
                ops: vec![IlOp::Load(0xEE09), IlOp::LoadInd { off: 0 }],
            })
        );
        assert_eq!(
            parse_segment(&free_fn(IND_MEMBER0), NO_LOCALS),
            Some(BodyShape::IndirectLoad {
                params: vec![0xFE09],
                ops: vec![IlOp::Load(0xFE09), IlOp::LoadInd { off: 0 }],
            })
        );
        // The offset is a SIGNED short-form byte, and `-1` on an `int *` is −4
        // bytes — the scale is already applied by the front end.
        assert_eq!(
            parse_segment(&free_fn(IND_SUBSCRIPT_NEG), NO_LOCALS),
            Some(BodyShape::IndirectLoad {
                params: vec![0x100A],
                ops: vec![IlOp::Load(0x100A), IlOp::LoadInd { off: -4 }],
            })
        );
    }

    #[test]
    fn indirect_load_leaf_binds_this_as_argument_zero() {
        // `this` is not in the `2D` list, so `params` must be built from the
        // pre-body binding — otherwise the base register is unknown (or, worse,
        // an explicit formal is mapped one register low).
        let lo = find_subslice(IND_THIS_GETTER, &LO_MARKER).unwrap();
        assert_eq!(parse_this_token(IND_THIS_GETTER, lo), Some(ThisBinding::Bound(0xF809)));
        assert_eq!(
            parse_segment(&free_fn(IND_THIS_GETTER), NO_LOCALS),
            Some(BodyShape::IndirectLoad {
                params: vec![0xF809],
                ops: vec![IlOp::Load(0xF809), IlOp::LoadInd { off: 4 }],
            })
        );
    }

    /// T3: the four accepted non-4-byte pointee shapes, each from a whole captured
    /// segment, plus the two refusals that separate them from wrong bytes.
    #[test]
    fn indirect_load_leaf_decodes_narrow_and_wide_pointees() {
        let ind = |tok: u32, off: i32, width: u8, sext: bool| {
            Some(BodyShape::IndirectLoad {
                params: vec![tok],
                ops: vec![IlOp::Load(tok), IlOp::LoadIndSized { off, width, sext }],
            })
        };
        // `char g_c_c(char*)`: a signed byte with NO conversion extends nothing.
        assert_eq!(
            parse_segment(NARROW_CHAR_DEREF, NO_LOCALS),
            ind(0x0F0A, 0, 1, false)
        );
        // `int g_i_c(char*)`: the same load + `2C 86 41 74 00` — the one case that
        // pays an instruction, so `sext` is the only field that differs.
        assert_eq!(
            parse_segment(NARROW_CHAR_TO_INT, NO_LOCALS),
            ind(0x2A0A, 0, 1, true)
        );
        // `int g_i_us(unsigned short*)`: the same token over an unsigned pointee is
        // free. If the parse read the token instead of the pointee's signedness,
        // this and the previous case would be indistinguishable — and one of the two
        // objs would be wrong.
        assert_eq!(
            parse_segment(NARROW_USHORT_TO_INT, NO_LOCALS),
            ind(0x3A0A, 0, 2, false)
        );
        // `long long m_q(S*)`: offset 16 folds into the DS-form displacement.
        assert_eq!(
            parse_segment(NARROW_LL_MEMBER, NO_LOCALS),
            ind(0x4F0A, 16, 8, false)
        );
        // `char C::t_c() const`: `this` binds at index 0, and the `2C` is a cv-strip
        // (const char → char) rather than a widening, so no extension.
        assert_eq!(
            parse_segment(NARROW_CONST_CHAR_THIS, NO_LOCALS),
            ind(0x530A, 0, 1, false)
        );
        // Refused: a signed halfword widened to int is `lha r3` at `/O1` and
        // `lhz r11 ; extsh r3,r11` at `/Ox` — one shape, two lowerings, no mode
        // here to choose with.
        assert_eq!(parse_segment(NARROW_SHORT_TO_INT_REFUSED, NO_LOCALS), None);
        // Refused: a `#pragma pack(1)` 8-byte member. Its tag says align-1, and
        // reading the width from the tag would emit `lbz` for a `long long`.
        assert_eq!(parse_segment(NARROW_LL_PACKED_REFUSED, NO_LOCALS), None);
    }

    /// The `27` byte-offset-add type states the pointee's width, and the `30` load
    /// states it again. They must **agree**: the two are separate fields in separate
    /// productions, and a parse that trusted only one of them would accept a
    /// splice whose two halves disagree — where c2's own output cannot.
    #[test]
    fn indirect_load_leaf_requires_the_offset_type_and_the_load_to_agree_on_width() {
        // `long long m_q(S*)`, with the `27` type re-tagged to a pointer-to-1-byte
        // (`88 43 93 08` → `82 43 93 08`) and nothing else touched.
        let lo = find_subslice(NARROW_LL_MEMBER, &LO_MARKER).unwrap();
        let mut mismatched = NARROW_LL_MEMBER.to_vec();
        let at = lo + 17; // the `27` type's tag
        assert_eq!(mismatched[at - 1], 0x27, "the 27 marker");
        assert_eq!(mismatched[at], 0x88, "the pointee-width tag");
        mismatched[at] = 0x82;
        assert_eq!(parse_segment(&mismatched, NO_LOCALS), None);
        // Control: the same splice on the *load's* tag alone also refuses, so the
        // agreement is required in both directions rather than one being ignored.
        let mut load_only = NARROW_LL_MEMBER.to_vec();
        assert_eq!(load_only[lo + 21], 0x30, "the load marker");
        load_only[lo + 22] = 0x82;
        assert_eq!(parse_segment(&load_only, NO_LOCALS), None);
    }

    #[test]
    fn indirect_load_leaf_refuses_the_adjacent_shapes() {
        // Splice one field of IND_DEREF at a time. Each variant is a construct
        // whose reference codegen differs (see fixtures/cpp/il_expr_load_neg.cpp).
        //
        // Offsets are measured from the `LO` marker rather than written as absolute
        // indices: they were absolute, and prepending the real `53 53 26 <fn>`
        // prologue to the pinned segment silently shifted every one of them.
        // Offsets are relative to the `LO` marker, which is a landmark the segment
        // itself defines.
        let base = |seg: &[u8]| find_subslice(seg, &LO_MARKER).unwrap();
        let d = base(IND_DEREF);
        let bad = |patch: &[(usize, u8)]| {
            let mut s = IND_DEREF.to_vec();
            for &(i, b) in patch {
                s[d + i] = b;
            }
            parse_segment(&free_fn(&s), NO_LOCALS)
        };
        // A `char` pointee is `lbz`, not `lwz` (`30 82 11 70`) — a *different* op,
        // and since T3 a modeled one: it must come out as a width-1 load, never as
        // the 4-byte [`IlOp::LoadInd`] whose `lwz` would read three bytes too many.
        assert_eq!(
            bad(&[(12, 0x82), (13, 0x11), (14, 0x70), (16, 0x82), (17, 0x11), (18, 0x70)]),
            Some(BodyShape::IndirectLoad {
                params: vec![0xEE09],
                ops: vec![
                    IlOp::Load(0xEE09),
                    IlOp::LoadIndSized { off: 0, width: 1, sext: false },
                ],
            })
        );
        // A `float` pointee is `lfs` (`30 86 45 40`).
        assert_eq!(bad(&[(13, 0x45), (14, 0x40), (17, 0x45), (18, 0x40)]), None);
        // A pointer pointee (`int **`) emits the same word but stays refused.
        assert_eq!(bad(&[(13, 0x43)]), None);

        // Arithmetic after the load: the load lands in the scratch register, so
        // this must not reach the affine selector.
        let mut with_add = IND_DEREF[..d + 15].to_vec();
        with_add.extend_from_slice(&[0x33, 0x86, 0x41, 0x74, 0x01, 0x02]); // + 1
        with_add.extend_from_slice(&IND_DEREF[d + 15..]);
        assert_eq!(parse_segment(&free_fn(&with_add), NO_LOCALS), None);

        // A `28` payload other than `00 00` is unexplained and must refuse.
        let n = base(IND_SUBSCRIPT_NEG);
        let mut sub_bad = IND_SUBSCRIPT_NEG.to_vec();
        sub_bad[n + 17] = 0x01;
        assert_eq!(parse_segment(&free_fn(&sub_bad), NO_LOCALS), None);

        // An offset past the 16-bit displacement materializes an index register.
        let mut wide = IND_SUBSCRIPT_NEG[..n + 15].to_vec();
        wide.extend_from_slice(&[0x80, 0x80, 0x1A, 0x06, 0x00]); // 400000
        wide.extend_from_slice(&IND_SUBSCRIPT_NEG[n + 16..]);
        assert_eq!(parse_segment(&free_fn(&wide), NO_LOCALS), None);
    }
}
