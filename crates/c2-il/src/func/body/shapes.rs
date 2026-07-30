use super::chain::{
    additive_chain_canonical, has_repeated_leaf, leaves_ascending, straight_line_is_out_of_class,
    straight_line_out_of_class_ctx, substitute, MAX_SUBST_OPS,
};
use super::expr::{
    eat_fn_tail, eat_return_plumbing, eat_scopes, formals_marker, intrinsic_selector, parse_expr,
    parse_formals, BODY_SCOPE_DEPTH,
};
use super::mcall;
use super::{blk, Block, BodyShape, DtorSubObject, SeqCall, SeqTail};
use crate::func::readers::{
    eat, eat_byte, eat_int_like, eat_int_like_or_ptr4, eat_opt_stmt_marker, eat_value_type,
    is_ptr4_kind, is_ptr_to_4, read_token_var, read_type, read_varint, value_class,
    ValueClass, DOUBLE_LIT_TYPE, DOUBLE_TYPE, FLOAT_LIT_TYPE, FLOAT_TYPE, INT_TYPE, UINT_TYPE,
};
use crate::func::sy::{fp_reg_of, ArgClass, SyView};
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
        // `start_of_stmt` is the byte the statement opened on — the `26` this loop
        // just consumed as a destination. It may not be one: a member call in a
        // *value* position opens on its outermost method push, which is the same
        // two bytes. Only the right-hand side's own refusal can tell, so the
        // statement head is carried to [`mcall::reanchor_chain`], which puts it back
        // when (and only when) the walk and the bind count agree that the token was
        // a method. `docs/IL_CALL_IN_EXPR.md` §16.4 measured this as a 4.4×
        // undercount of chains; §18.3 is the fix.
        let start_of_stmt = p;
        p = probe;
        let rhs = match parse_expr(seg, &mut p, 0x32) {
            Ok(r) => r,
            Err(b) => return Err(mcall::reanchor_chain(seg, start_of_stmt, probe, b)),
        };
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
    // The **same** gate the straight-line path applies, at the second site that
    // produces a `StraightLine`. It was missing here, and the census therefore
    // counted bodies `select_text` refuses: `int f(int a){ int x = a; return
    // x * 3; }` substitutes to `a * 3`, censused in class, and the port returned
    // `NotImplemented` — the exact census/gate disagreement
    // `straight_line_out_of_class_ctx` was extracted from codegen to prevent,
    // reintroduced by a second producer that did not consult it. One fact, one
    // locator: the predicate is shared, not copied.
    if let Some(ctx) = straight_line_out_of_class_ctx(&ret, &params) {
        return Err(Block { ctx, byte: None, off: p, aux: 0 });
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
pub(crate) fn try_parse_float_leaf(
    seg: &[u8],
    start: usize,
    lo: usize,
    sy: SyView,
) -> Option<BodyShape> {
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
    // **The FP register file is numbered over the FP parameters ALONE**, so the
    // shape carries its parameters in *that* order and not in declaration order.
    // [`c2_core::codegen::float_leaf_text`] maps entry `n` of this list to
    // `f(n+1)`, which is the register number exactly when the list holds the FP
    // parameters and nothing else.
    //
    // This is the fifth instance of `docs/GAPS.md` §6's "two facts sharing one
    // field", and it was **live** — `float mixfp(int a, float b, float c)
    // { return b*c; }` emitted `fmuls f1,f2,f3` where c2 emits `fmuls f1,f1,f2`,
    // on mainline, with all four mode lanes and the 3,743-case sweep green. The
    // corpus had only the safe half of the pair again: not one FP fixture had a
    // parameter list that was anything but all-`float` or all-`double`.
    //
    // It was closed then by the blunt gate `params.len() != seen.len()` — every
    // formal has to be an FP operand of the body — which is correct and costs
    // **1,005 functions** on the workload (`IL_CALL_IN_EXPR.md` §23.1, MEASURED
    // by counterfactual). What replaces it is the actual numbering, read from
    // `.sy`'s type kind (`sy::ArgClass`): a non-FP formal is skipped rather than
    // refused, and an FP formal that the body never loads still advances the
    // count. `this` is prepended as a GPR — it takes r3 and displaces nothing in
    // the FP file, which the member-function capture in `docs/CODEGEN_FP_ARGS.md`
    // §1 confirms emits the identical `fmr` sequence as the free function.
    let formals = parse_formals(seg, lo).ok()?;
    let classes = sy.arg_classes(&formals).ok()?;
    let this = matches!(parse_this_token(seg, lo)?, ThisBinding::Bound(_));
    let params: Vec<u32> = formals
        .iter()
        .zip(&classes)
        .filter(|(_, c)| matches!(c, ArgClass::Fp { .. }))
        .map(|(t, _)| *t)
        .collect();
    // Mixing the widths across the *parameter list* is not the same question as
    // mixing them inside the expression (which the operand-type loop already
    // refuses): `float f(double a, float b){ return b; }` is one FP file with two
    // widths in it, and every operand this body reads is `fty`. The register
    // numbering is width-agnostic — `int t8(double a, float b, double c)` puts
    // them in f1, f2, f3 (captured) — so nothing here needs the widths to agree.
    if params.len() > 13 || !seen.iter().all(|t| params.contains(t)) {
        return None;
    }
    // **A pooled constant keeps the OLD gate**, and this is the one place the
    // widening is deliberately held back rather than taken.
    //
    // `float_leaf_text` emits a pooled constant as an `.rdata` COMDAT reached
    // through a REFHI/REFLO pair, and `codegen::function_gate` refuses that under
    // **function-level linking** (`/Gy`, which `/O1` implies) because the COMDAT
    // association is not modeled — a refusal that lives in codegen only, because
    // the linkage mode is a translation-unit flag the parser cannot see. That
    // split cost nothing while no such body was in class; widening the parameter
    // model put one in class (a member FP leaf with a constant, in
    // `src/lazer/meta_ham/HamProfile.cpp`) and the 878-TU scan's census/gate
    // disagreement went 0 → **1**, in the over-claiming direction.
    //
    // So the pooled-constant population is held **exactly** at what it was before
    // this rung: every formal must be an FP operand of the body, the gate this
    // rung otherwise replaces. It costs **1 function** on the workload (measured,
    // and the whole of the disagreement it caused), and it keeps the invariant at
    // 0 in both directions without narrowing anything the rung is actually about.
    // The real repair is to model `/Gy` `.rdata` COMDATs — `docs/CODEGEN_FP_ARGS.md`
    // §5 ranks it — after which this clause deletes itself.
    if ops.iter().any(|o| matches!(o, IlOp::FpLit { .. }))
        && (params.len() != seen.len() || this)
    {
        return None;
    }
    let _ = this;
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

/// The **constructor epilogue**: a value expression sitting between the RETURN
/// and the function tail, naming `this`.
///
/// ```text
///   … 3A <label> 54 02 29 <label>   B9 <this> <TYPE> 41 <TYPE>   4F 12 47 54 01 54 00
///                                   ^^^^^^^^^^^^^^^^^^^^^^^^^^
/// ```
///
/// Every other shape in this module puts its returned value *before* the `3A`,
/// where [`eat_return_head`]'s `has_result_type` annotation covers it. A
/// constructor does not: its statements each end on a `4B` discard, and the value
/// it returns — `this`, which MSVC constructors hand back in r3 — is written after
/// the `29`. The parse used to stop dead on that `B9`, which is the census key
/// `fn-tail-0xB9`: **29,552 functions, the largest call-free row that was named
/// but never decomposed.**
///
/// **It costs no instruction at all**, and that is measured, not assumed. `this`
/// is already in r3 on entry and a leaf body cannot have moved it, so the epilogue
/// is a no-op. Captured from the live toolchain, eight empty constructors in one
/// translation unit — varying arity, member count, member type and position in the
/// file — every one of them exactly `4E 80 00 20`:
///
/// ```text
///   struct T { int m; T(); };  T::T() {}                 -> blr
///   struct E { int m; E(int); };  E::E(int a) {}         -> blr
///   struct G { int m; G(int,int); };  G::G(int,int) {}   -> blr
/// ```
///
/// That run is also the locality tell `docs/GAPS.md` §6 asks for before a row is
/// taken: byte-identical sources in one TU emitting one sequence means the
/// decision is local, which is what the `data-addr` rung lacked.
///
/// **The leaf restriction is load-bearing and is not conservatism.** Add a call
/// and c2 stops being able to leave `this` in r3:
///
/// ```text
///   struct B { int b; B(); };  struct D : B { D(); };  D::D() {}
///     mflr r12 ; stw r12,-8(r1) ; stw r31,-16(r1) ; stwu r1,-96(r1)
///     mr r31,r3 ; bl B::B ; mr r3,r31 ; …            <- this saved and restored
/// ```
///
/// so the 832 bodies whose epilogue follows a call need the general frame and stay
/// refused (they are `calls-1` to the frame measure, §18). Only the caller decides
/// that: this recognizer is used by exactly one arm, the empty-body one.
///
/// Both fields are required **literally**, per `docs/GAPS.md` §6's rule that a
/// field which never varied is indistinguishable from a constant. The token must
/// be the one [`parse_this_token`] bound — a positive identification, never
/// "some token we could not place" — and the loaded type must be byte-identical
/// to the `41` result type. Across the 29,549 sites the real workload has, the
/// token was `this` in **every** one; requiring it means a body that returns
/// anything else refuses instead of silently emitting a constructor's bytes.
pub(crate) fn eat_ctor_this_epilogue(seg: &[u8], p: &mut usize, lo: usize) -> bool {
    let this_tok = match parse_this_token(seg, lo) {
        Some(ThisBinding::Bound(t)) => t,
        // `Absent` and `None` alike: a free function has no `this` to return, and
        // an undetermined binding must never be read as one.
        _ => return false,
    };
    let mut q = *p;
    if seg.get(q) != Some(&0xB9) {
        return false;
    }
    q += 1;
    let (tok, w) = match read_token_var(seg, q) {
        Some(x) => x,
        None => return false,
    };
    if tok != this_tok {
        return false;
    }
    q += w;
    let load_ty = match read_type(seg, q) {
        Some((_, _, _, tw)) => &seg[q..q + tw],
        None => return false,
    };
    q += load_ty.len();
    if seg.get(q) != Some(&0x41) {
        return false;
    }
    q += 1;
    let res_ty = match read_type(seg, q) {
        Some((_, _, _, tw)) => &seg[q..q + tw],
        None => return false,
    };
    if res_ty != load_ty {
        return false;
    }
    *p = q + res_ty.len();
    true
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
///   30 <INT4|PTR4-TYPE>                          the indirect load
///   [ 2C <same class> 00 ]                       a cv-qualification strip
///   41 <same class>                              result type
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
/// * **The loaded type must be a 1-, 2-, 4- or 8-byte integer** ([`SIZED_PTEE`])
///   **or a 4-byte pointer** ([`is_ptr4_kind`]). The width picks the
///   instruction: `char *` is `lbz`, `short *` is `lhz`, `long long *` is `ld`,
///   `float *` is `lfs`, `double *` is `lfd` — all captured, all different, and
///   the FP ones are still refused. A **pointer** value is the one non-integer
///   case that lowers to the same bare `lwz` as a 4-byte integer:
///   `int* H::gpi() const { return mpi; }` is `lwz r3,0(r3) ; blr`, the same
///   scheme as the `int` getter beside it (`docs/IL_LOAD_TYPES.md` §3/§4), which
///   is why it needs no encoder.
///
///   Note the gate is on the loaded value's *own* width, never the pointee's —
///   loading a `char*` **member** is `lwz`, while loading *through* a `char*` is
///   `lbz`, and both spell `char` somewhere in the type. The two questions have
///   two predicates ([`is_ptr4_kind`] and [`is_ptr_to_4`]) for exactly that
///   reason.
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

    // The indirect load itself. The loaded value is either a 4-byte integer or a
    // 4-byte **pointer** — the two cases c2 lowers with the identical `lwz`
    // (`docs/IL_LOAD_TYPES.md` §3/§4: `g_pi` is `lwz r3,24(r3)`, byte-identical
    // in scheme to the in-class `g_i`), which is why this rung needs no encoder.
    if !eat_byte(seg, &mut p, 0x30) {
        return None;
    }
    let (tag, kind, _, tw) = read_type(seg, p)?;
    // Width 4 — a 4-byte integer or a 4-byte **pointer**, the two classes c2
    // lowers with the identical bare `lwz` (`docs/IL_LOAD_TYPES.md` §3/§4: the
    // pointer member `g_pi` is `lwz r3,24(r3)`, byte-identical in scheme to the
    // in-class `g_i`), which is why the pointer half needs no encoder.
    let load_op = if let Some(loaded) = value_class(tag, kind) {
        if matches!(ptee_width, Some(w) if w != 4) {
            return None;
        }
        p += tw;

        // An optional cv-qualification strip, in the loaded value's own class:
        // provably free over a 4-byte integer source (see [`is_int4_type`]) and
        // provably free over a pointer one (`2C ptr→ptr` emits nothing —
        // `void* f(H* p){return p;}` is a bare `blr`). The trailing varint must
        // be the `00` observed at all 14,098 aligned sites.
        //
        // The two classes are kept apart rather than merged into one "either"
        // test. A cross-class `2C` — pointer source, int target, or the reverse
        // — is a *reinterpret* this port has never probed, and the neighbouring
        // shape that would look identical under the wrong rule is the one that
        // matters here: an address-adjusting up/downcast also produces a pointer
        // from a pointer, and it costs an `addi`. It never comes through `2C`
        // (it is intrinsic 2113/2114/2115), but the way to keep that true is to
        // admit only the conversions each class was measured to make free.
        if *seg.get(p)? == 0x2C {
            let mut probe = p + 1;
            if !eat_value_type(seg, &mut probe, loaded) || !eat_byte(seg, &mut probe, 0x00) {
                return None;
            }
            p = probe;
        }

        // Result type, in the same class as the load.
        if !eat_byte(seg, &mut p, 0x41) || !eat_value_type(seg, &mut p, loaded) {
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

    let params = parse_params(seg, lo).ok()?;
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

// [`ValueClass`], [`value_class`] and [`eat_value_type`] used to live here.
// They moved to `readers.rs` when `parse_expr` needed the same three (D12,
// `docs/IL_CALL_IN_EXPR.md` §24) — one fact, one locator, exactly as
// `is_ptr4_kind` moved in §21.3. Nothing about them changed.


/// Try to parse a **pointer-identity leaf**: a whole body that is one pointer
/// returned unchanged — `return p;`, `return this;`, and pointer-to-pointer
/// casts of either.
///
/// ```text
///   B9 <tok> <PTR4-TYPE>            the value
///   [ 2C <PTR4-TYPE> 00 ]           a cv-strip / ptr→ptr cast, emitting nothing
///   41 <PTR4-TYPE>                  result type
///   <return plumbing, reaching the segment end>
/// ```
///
/// c2 emits **no instruction at all** for the value: the pointer is already in
/// its incoming argument register, so the body is a bare `blr` — exactly what
/// the integer identity `int f(int a){ return a; }` already produces, which is
/// why this hands back the existing [`BodyShape::StraightLine`] lowering and
/// needs no codegen. MEASURED (`docs/IL_LOAD_TYPES.md` §3, probe `p10`):
/// `void* f(H* p){ return p; }` is one `blr`, and a `2C` from pointer to pointer
/// emits nothing.
///
/// Three real translation units (headtracker, MemMgr, Sorting) carry 16 bodies
/// of exactly this grammar, all four of the accepted tag spellings among them.
///
/// The gates, and the neighbour each one separates:
///
/// * **No offset add.** `B9 <ptr> 33 <int> <k> 27 <ptr> 2C <ptr> 00 41 <ptr>` —
///   the same production *minus the `30`* — is `return &s->m;`, and it emits an
///   `addi`. It occurs: 7 of the 40 pointer-shaped bodies in the three TUs
///   scanned are this, and admitting them as identities would emit a bare `blr`
///   where c2 emits `addi r3,r3,12`. So the identity leaf is anchored on the
///   `B9` *immediately* followed by the result, and the offset-add form has no
///   path into it.
/// * **The value must be a formal or `this`, bound positively.** An
///   `Undetermined` `this` refuses (the line-70 rule).
/// * **The result must already be in r3.** `S* f(int a, S* s){ return s; }` is
///   `mr r3,r4`, a real instruction — refused by the shared
///   [`straight_line_is_out_of_class`], the same predicate the arithmetic path
///   uses, rather than by a second copy of the rule.
/// * **Pointer *literals* are elsewhere.** `return 0;` typed as a pointer is a
///   `33` LITERAL (census `expr-lit-type-8643xx`) and needs an `li`; this
///   production is anchored on `B9` and cannot reach it.
pub(crate) fn try_parse_ptr_identity_leaf(seg: &[u8], start: usize, lo: usize) -> Option<BodyShape> {
    let mut p = start;
    if !eat_byte(seg, &mut p, 0xB9) {
        return None;
    }
    let (tok, w) = read_token_var(seg, p)?;
    p += w;
    let (tag, kind, _, tw) = read_type(seg, p)?;
    if !is_ptr4_kind(tag, kind) {
        return None;
    }
    p += tw;

    if *seg.get(p)? == 0x2C {
        let mut probe = p + 1;
        if !eat_value_type(seg, &mut probe, ValueClass::Ptr4) || !eat_byte(seg, &mut probe, 0x00) {
            return None;
        }
        p = probe;
    }

    if !eat_byte(seg, &mut p, 0x41) || !eat_value_type(seg, &mut p, ValueClass::Ptr4) {
        return None;
    }
    eat_return_plumbing(seg, &mut p, false, BODY_SCOPE_DEPTH).ok()?;

    let params = parse_params(seg, lo).ok()?;
    let ix = params.iter().position(|&t| t == tok)?;
    // Past the eighth argument the value is stack-homed, which needs a frame.
    if ix >= 8 {
        return None;
    }
    let ops = vec![IlOp::Load(tok)];
    if straight_line_is_out_of_class(&ops, &params) {
        return None;
    }
    Some(BodyShape::StraightLine { params, ops })
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
    let (off, base_tok, p) = parse_base_member_designator(seg, start, is_ptr_to_4)?;
    finish_indirect_load(seg, p, lo, base_tok, off)
}

/// The intrinsic-2117 designator alone: `(summed byte offset, object token, end)`.
///
/// Split out of [`try_parse_base_member_load`] so the two consumers of the same
/// address — the LOAD leaf (`return b;`) and the ADDRESS leaf (`return &b;`) —
/// share one decoder. `GAPS.md` §6's "one fact, one locator": a second copy is a
/// second place for the two-literal sum, the `66` descriptor walk or the header
/// bound to drift.
///
/// `ptr_ok` is the caller's rule for the three pointer TYPEs the production
/// carries (the `40` result, the object `B9`, and its `55` push), and it is a
/// *parameter* rather than a fixed predicate because the two consumers are not
/// equally constrained and merging them would change what the load path accepts:
///
/// * the LOAD path passes [`is_ptr_to_4`] — pointer to a **4-byte** object — and
///   is byte-for-byte the rule it had before this split;
/// * the ADDRESS path passes [`is_ptr_any`], because the member's width does not
///   reach the emitted instruction at all. MEASURED (`work/bma/probes/p2.cpp`):
///   the inherited `char`, `short`, `int`, `long long`, `float` and `double`
///   members each emit the identical single `addi`, and their designators carry
///   `A6 43`, `A4 43`, `A6 43`, `A6 43`, `A6 43`, `A6 43` — so the tag's width
///   nibble is not even a reliable statement of the pointee width here, which is
///   the second reason not to gate on it.
fn parse_base_member_designator(
    seg: &[u8],
    start: usize,
    ptr_ok: fn(u8, u8) -> bool,
) -> Option<(i32, u32, usize)> {
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
    if !ptr_ok(tag, kind) {
        return None;
    }
    p += tw;
    // The argument header: `66 <n>` then n type references, skipped structurally
    // so a second inheritance step (n = 3) parses like the first.
    //
    // The refs are **LEB128 ids**, not a fixed two bytes each — see
    // [`super::mcall::eat_class_descriptor`], which owns that encoding and carries
    // the witnesses. This code stepped `2 * n` and so landed inside the second ref
    // of any descriptor with a wide id, which is every large translation unit;
    // `src/App.cpp` and `src/lazer/game/Game.cpp` carry `fb 8a 01`, `ff ff 01`,
    // `d3 80 02`. The bound on `n` stays here rather than moving into the decoder,
    // because it is this shape's acceptance rule and not part of the encoding.
    let n_refs = mcall::eat_class_descriptor(seg, &mut p)?;
    if n_refs == 0 || n_refs > MAX_HEADER_REFS {
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
    if !ptr_ok(tag, kind) {
        return None;
    }
    p += tw;
    if !eat_byte(seg, &mut p, 0x55) {
        return None;
    }
    let (tag, kind, _, tw) = read_type(seg, p)?;
    if !ptr_ok(tag, kind) {
        return None;
    }
    p += tw;
    if !eat_byte(seg, &mut p, 0x4C) {
        return None;
    }
    Some((off, base_tok, p))
}

/// `(tag, kind)` of a **pointer TYPE, whatever it points at** — the rule the
/// *address* productions use, where the pointee's width never reaches the
/// emitted instruction.
///
/// The two existing pointer predicates each answer a narrower question and
/// neither fits: [`is_ptr_to_4`] demands a 4-byte pointee (it gates a `lwz`),
/// and [`is_ptr4_kind`] demands one of four exact tags (it gates a pointer
/// *value* in a register). An address leaf needs neither, because
/// `addi rD,rBase,k` is the same word for every pointee.
///
/// Spelled as a literal whitelist rather than as nibble arithmetic, for the
/// reason [`is_ptr4_kind`]'s own comment gives — and the whitelist is the cross
/// product of two axes each independently witnessed:
///
/// * the tag's **cv bits** `0x20` (const) and `0x10` (volatile), all four
///   combinations, exactly as [`is_ptr4_kind`] already admits. `0xC6` and every
///   other tag with bit `0x40` set is **refused**: `readers.rs` records that the
///   bit occurs and no probe produced it here.
/// * the tag's **width nibble**, which is 2/4/6/8. It is *not* a dependable
///   statement of the pointee width in this position and that is precisely why
///   it is admitted rather than checked: MEASURED (`work/bma/probes/p2.cpp`,
///   `p1.cpp`) `char*` carries `86 43`, `short*` carries `84 43`, and
///   `long long*`, `float*` and `double*` all carry `86 43` — while all six
///   emit the identical single `addi`. Witnessed tags are `84`, `86`, `A4`,
///   `A6`; the other twelve are the same two axes crossed and are admitted on
///   that basis, which is a HYPOTHESIS about the encoding and not a capture.
///
/// `kind` must be exactly `0x43` — width nibble 4 (the pointer's own size on
/// this target) and class nibble 3 (a **data** pointer). `0x44` (a function or
/// code pointer) is refused: no probe produced one at an address-leaf position,
/// and a code pointer is the one case where "the pointee width does not matter"
/// has not been checked.
fn is_ptr_any(tag: u8, kind: u8) -> bool {
    const PTR_TAGS: [u8; 16] = [
        0x82, 0x84, 0x86, 0x88, // plain, width nibble 2/4/6/8
        0x92, 0x94, 0x96, 0x98, // volatile
        0xA2, 0xA4, 0xA6, 0xA8, // const
        0xB2, 0xB4, 0xB6, 0xB8, // const volatile
    ];
    PTR_TAGS.contains(&tag) && kind == 0x43
}

/// Consume a run of **byte-offset adds** applied to an address, summing them.
///
/// ```text
///   33 <int-like> k   27 <PTR>        a member offset, re-typing the address
///   33 <int-like> k   28 00 00        a subscript offset, not re-typing it
/// ```
///
/// The load leaf ([`try_parse_indirect_load_leaf`]) admits **at most one** of
/// these, because a second one there means a chained subscript whose lowering
/// needs `slwi`/`lwzx`. An *address* has no such limit: every add is folded into
/// the one `addi`'s displacement, and the whole run costs nothing extra.
/// MEASURED — `int* DR::pt2()` (`&t[2]` on an inherited array) is
/// `LIT(0) 28 · LIT(8) 28` and emits `addi r3,r3,16`; `&s->arr[2]` on a plain
/// struct is `LIT(40) 27 · LIT(8) 28` and emits `addi r3,r3,48`.
///
/// The `28` payload must be exactly `00 00`, the same fail-closed rule
/// [`try_parse_indirect_load_leaf`] states: those two bytes are `00 00` at every
/// captured site and their meaning is UNKNOWN.
///
/// Returns `None` — cursor untouched — on an overflowing sum. Stops without
/// consuming at the first token that is not an offset add, which is not a
/// failure: zero adds is the legitimate `return &p->Base::m;`.
fn eat_addr_offset_adds(seg: &[u8], p: &mut usize) -> Option<i32> {
    let mut total: i32 = 0;
    loop {
        if seg.get(*p) != Some(&0x33) {
            return Some(total);
        }
        let mut probe = *p + 1;
        if !eat_int_like(seg, &mut probe) {
            return Some(total);
        }
        let k = match read_varint(seg, &mut probe) {
            Some(k) => k,
            None => return Some(total),
        };
        match seg.get(probe) {
            Some(&0x27) => {
                probe += 1;
                let (tag, kind, _, tw) = read_type(seg, probe)?;
                if !is_ptr_any(tag, kind) {
                    return Some(total);
                }
                probe += tw;
            }
            Some(&0x28) => {
                probe += 1;
                if !eat(seg, &mut probe, &[0x00, 0x00]) {
                    return Some(total);
                }
            }
            _ => return Some(total),
        }
        total = total.checked_add(k)?;
        *p = probe;
    }
}

/// Try to parse an **address leaf**: a whole body that is one sub-object
/// *address* and nothing else — `return &s->m;`, `return &p->Base::m;`,
/// `return s->arr;`, `return &p->t[2];`.
///
/// ```text
///   <designator>                       the object pointer, one of two spellings
///   ( 33 <int-like> k 27 <PTR>         byte-offset adds, any number, summed
///   | 33 <int-like> k 28 00 00 )*
///   [ 2C <PTR> 00 ]                    an array-to-pointer decay / cv strip
///   41 <PTR>                           result type: a pointer
///   <return plumbing, reaching the segment end>
/// ```
///
/// where `<designator>` is either a plain pointer LOAD `B9 <tok> <PTR4>` (a
/// formal or `this`) or the intrinsic-2117 `base-member-addr` production
/// ([`parse_base_member_designator`]), whose two literals contribute their sum
/// to the offset before the adds are applied.
///
/// **This is one instruction and the same one either way**: `addi rD, rBase, K`,
/// with `K` the total. MEASURED at the fixture profile — every word below read
/// off the reference obj (`work/bma/probes/p1.cpp`, `p2.cpp`, `p3.cpp`):
///
/// ```text
///   int* f(S* s){ return &s->b; }          addi r3,r3,4      ; blr
///   int* f(int x, S* s){ return &s->b; }   addi r3,r4,4      ; blr   <- ANY base reg
///   int* D::pb1(){ return &b1; }           addi r3,r3,12     ; blr   <- 2117, 8+4
///   int* DR::pt2(){ return &t[2]; }        addi r3,r3,16     ; blr   <- two `28`s
///   int* f(S* s){ return s->arr; }         addi r3,r3,40     ; blr   <- the decay
///   int* f(S* s){ return &s->a; }                             blr    <- K = 0
///   char*/short*/…/double* members         the identical addi             (p2 `DW`)
/// ```
///
/// and **no `.pdata` entry**: the body is a leaf and c2 emits none.
///
/// Why each gate is load-bearing — every one is a *captured* neighbour that
/// emits something else:
///
/// * **`K` must fit a signed 16-bit displacement.** `&p->t` at 32764 is one
///   `addi`; at 32768 it is **`addis r3,r3,1 ; addi r3,r3,-32768`**, two
///   instructions this shape does not emit (`work/bma/probes/p3.cpp`).
/// * **`K == 0` requires the base to be the FIRST parameter.** The address is
///   then already in r3 and the body is a bare `blr` — but from any other
///   argument register c2 emits a real `mr r3,r4` (measured, `z_r4`/`i_z_r4`).
///   That is the same boundary [`straight_line_is_out_of_class`] draws for the
///   bare-parameter identity, and it is drawn here rather than assumed.
/// * **The result must be a POINTER.** With a `30` in front of the `41` the body
///   is a *load* and emits `lwz` — [`try_parse_indirect_load_leaf`]'s shape, one
///   token away from this one. This production is anchored on the `41`
///   immediately following the adds, so a load has no path into it.
/// * **A `2C` may only convert pointer→pointer.** An array-to-pointer decay and
///   a cv strip both emit nothing (measured: `r_d`, `a_arr0`); a cross-class
///   `2C` is a reinterpret this port has never probed.
/// * **The base must be a register argument** (`params` position < 8): past the
///   eighth it is stack-homed, which needs a frame.
///
/// Returns `None` — cursor untouched — for anything that is not exactly this
/// shape.
pub(crate) fn try_parse_addr_leaf(seg: &[u8], start: usize, lo: usize) -> Option<BodyShape> {
    let mut p = start;
    // The designator. The intrinsic form is anchored on a `33` literal and the
    // plain form on a `B9`, so the two cannot be confused; the intrinsic is tried
    // first for the same reason [`try_parse_indirect_load_leaf`] tries it first.
    let (mut off, base_tok) = match parse_base_member_designator(seg, p, is_ptr_any) {
        Some((off, tok, end)) => {
            p = end;
            (off, tok)
        }
        None => {
            if !eat_byte(seg, &mut p, 0xB9) {
                return None;
            }
            let (tok, w) = read_token_var(seg, p)?;
            p += w;
            let (tag, kind, _, tw) = read_type(seg, p)?;
            // A pointer *value* in a register: the `B9` operand position, where
            // the tag carries the pointer's own width.
            if !is_ptr4_kind(tag, kind) {
                return None;
            }
            p += tw;
            (0, tok)
        }
    };
    off = off.checked_add(eat_addr_offset_adds(seg, &mut p)?)?;

    // An array-to-pointer decay or a cv strip, pointer→pointer only.
    if seg.get(p)? == &0x2C {
        let mut probe = p + 1;
        let (tag, kind, _, tw) = read_type(seg, probe)?;
        if !is_ptr_any(tag, kind) {
            return None;
        }
        probe += tw;
        if !eat_byte(seg, &mut probe, 0x00) {
            return None;
        }
        p = probe;
    }

    // The result type — a pointer, which is what separates this from every
    // arithmetic leaf.
    if !eat_byte(seg, &mut p, 0x41) {
        return None;
    }
    let (tag, kind, _, tw) = read_type(seg, p)?;
    if !is_ptr_any(tag, kind) {
        return None;
    }
    p += tw;
    eat_return_plumbing(seg, &mut p, false, BODY_SCOPE_DEPTH).ok()?;

    // The displacement bound, checked once for both designators.
    if !(-0x8000..=0x7FFF).contains(&off) {
        return None;
    }
    let params = parse_params(seg, lo).ok()?;
    let ix = params.iter().position(|&t| t == base_tok)?;
    // Past the eighth argument the value is stack-homed, which needs a frame.
    if ix >= 8 {
        return None;
    }
    // A zero offset emits nothing when the address is already in r3, and one
    // `mr r3,rN` when it is not — the same register move the arithmetic identity
    // makes, which is why this refusal is gone rather than duplicated. MEASURED:
    // `int* f(int k, S* s){ return &s->a; }` is `7c832378` (`mr r3,r4`) and
    // `S* f(int k, const S* s){ return (S*)s; }` is the same word, against
    // `38640004` (`addi r3,r4,4`) for the nonzero-offset neighbour.
    Some(BodyShape::AddrLeaf {
        params,
        ops: vec![IlOp::Load(base_tok), IlOp::AddrOf { off }],
    })
}

/// The **width and register file** of a stored value's TYPE, or `None` — which
/// is a refusal, never a guess.
///
/// One locator over the two predicates that already answer this question for the
/// *load* side, in the same order [`finish_indirect_load_of`] asks them:
/// [`value_class`] for the two 4-byte classes c2 keeps in a GPR (a 4-byte
/// integer and a pointer — the pair it lowers with one identical `stw`), then
/// [`sized_ptee`] for the captured 1-, 2- and 8-byte scalars.
///
/// Everything else refuses, and **the floating-point types are the reason this
/// is a function and not a width lookup**: `86 45 40` and `88 85 41` are 4 and 8
/// bytes wide and are stored with `stfs`/`stfd` from `f1`, not `stw`/`std` from
/// `r4` (MEASURED: `void s_f(S* s, float v){ s->f = v; }` is `d0230014`, and
/// `s_d` is `d8230018`). A width-only rule would emit `stw r4` for both — wrong
/// bytes inside an accepted class. The FP argument register is numbered over the
/// FP parameters *alone*, which is the fifth instance of `GAPS.md` §6's "two
/// facts sharing one field" and the live mis-emit `float_leaf_text`'s header
/// records; sizing that widening is a rung, not a line.
fn store_value_width(tag: u8, kind: u8) -> Option<u8> {
    if value_class(tag, kind).is_some() {
        return Some(4);
    }
    sized_ptee(tag, kind).map(|(w, _)| w)
}

/// The width of a **floating-point** stored value — 4 for `float`, 8 for
/// `double` — or `None` when the TYPE is not one.
///
/// Keyed on the kind's **class nibble** (5, "real") and the tag's width nibble,
/// the same two channels `sy::SyView::arg_classes` uses on the `.sy` side, so the
/// two layers agree about what a floating-point value is by construction rather
/// than by two independent whitelists.
fn store_fp_value_width(tag: u8, kind: u8) -> Option<u8> {
    if (kind & 0x0F) != 0x5 {
        return None;
    }
    match tag & 0x0F {
        0x6 => Some(4),
        0x8 => Some(8),
        _ => None,
    }
}

/// Try to parse a **store leaf**: a whole body that is one store into a
/// sub-object and nothing else — `void f(S* s, int v){ s->m = v; }`,
/// `void D::set(int v){ Base::m = v; }`, `void f(S* s, int v){ s->arr[2] = v; }`,
/// `void f(int* p, int v){ *p = v; }`, `void f(S* s){ s->m = 7; }`.
///
/// ```text
///   <designator>                       the object pointer, the same two spellings
///   ( 33 <int-like> k 27 <PTR>         byte-offset adds, any number, summed
///   | 33 <int-like> k 28 00 00 )*
///   [ 2C <PTR> 00 ]                    a cv strip / array-to-pointer decay
///   ( B9 <tok> <VT> | 33 <VT> <k> )    THE VALUE: a formal, or an integer literal
///   32 <VT>                            the store; its TYPE restates the value's
///   4B                                 statement end — and the body ends here
///   <return plumbing, void, reaching the segment end>
/// ```
///
/// where `<designator>` is either a plain pointer LOAD `B9 <tok> <PTR4>` or the
/// intrinsic-2117 `base-member-addr` production ([`parse_base_member_designator`]),
/// whose two literals contribute their sum to the offset before the adds — the
/// same pair of spellings [`try_parse_addr_leaf`] and
/// [`try_parse_indirect_load_leaf`] accept, reached through the same decoder.
///
/// **This is one store instruction, and the width picks it.** MEASURED at the
/// fixture profile — every word below read off the reference obj
/// (`work/lf/probes/p1.cpp`):
///
/// ```text
///   void s_a (S* s, int v)       { s->a  = v; }   90830000  stw  r4,0(r3)
///   void s_b (S* s, int v)       { s->b  = v; }   90830004  stw  r4,4(r3)
///   void s_p (S* s, void* v)     { s->p  = v; }   90830008  stw  r4,8(r3)
///   void s_c (S* s, char v)      { s->c  = v; }   9883000c  stb  r4,12(r3)
///   void s_sh(S* s, short v)     { s->s  = v; }   b083000e  sth  r4,14(r3)
///   void s_q (S* s, long long v) { s->q  = v; }   f8830020  std  r4,32(r3)
///   void s_e2(S* s, int v)       { s->arr[2] = v; } 90830030  stw  r4,48(r3)
///   void s_k (S* s)              { s->a  = 7; }   39600007 91630000  li r11,7 ; stw r11,0(r3)
///   void s_arg2(int x,S* s,int v){ s->b  = v; }   90a40004  stw  r5,4(r4)  <- ANY two regs
///   void D::sb1(int v)           { b1 = v; }      90830004  stw  r4,4(r3)  <- 2117, 0+4
/// ```
///
/// and **no `.pdata` entry**: the body is a leaf, exactly like the load and
/// address leaves beside it.
///
/// Why each gate is load-bearing — every one is a *captured* neighbour that
/// emits something else:
///
/// * **The value must be a GPR-class scalar** ([`store_value_width`]). A `float`
///   or `double` member is `stfs`/`stfd` from the FP file and the FP argument
///   number is not the parameter index.
/// * **No conversion on the value.** `void M::setb(bool v){ m0 = v; }` (an `int`
///   member, a `bool` parameter) carries a `2C 86 41 74 00` and emits
///   `548b063e ; 91630000` — `clrlwi r11,r4,24 ; stw r11,0(r3)` — a real mask
///   through the scratch register. The production admits a `2C` only on the
///   *address*, pointer→pointer, where it is free.
/// * **The stored TYPE must restate the value's `<tag><kind>`.** They are
///   byte-identical at every captured site, and requiring it is what makes a
///   misaligned read fail closed instead of picking a plausible width.
/// * **`K` must fit a signed 16-bit displacement**, and a `width == 8` store's
///   `K` must be a multiple of 4 (`std` is DS-form and cannot encode the low two
///   bits) — the same two bounds the load leaf draws.
/// * **Both the base and the value must be register arguments** (`params`
///   position < 8): past the eighth they are stack-homed, which needs a frame.
///
/// Returns `None` — cursor untouched — for anything that is not exactly this
/// shape.
pub(crate) fn try_parse_store_leaf(
    seg: &[u8],
    start: usize,
    lo: usize,
    sy: SyView,
) -> Option<BodyShape> {
    let mut p = start;
    // The designator. The intrinsic form is anchored on a `33` literal and the
    // plain form on a `B9`, so the two cannot be confused; the intrinsic is tried
    // first for the same reason the load and address leaves try it first.
    let (mut off, base_tok) = match parse_base_member_designator(seg, p, is_ptr_any) {
        Some((off, tok, end)) => {
            p = end;
            (off, tok)
        }
        None => {
            if !eat_byte(seg, &mut p, 0xB9) {
                return None;
            }
            let (tok, w) = read_token_var(seg, p)?;
            p += w;
            let (tag, kind, _, tw) = read_type(seg, p)?;
            // A pointer *value* in a register: the `B9` operand position, where
            // the tag carries the pointer's own width.
            if !is_ptr4_kind(tag, kind) {
                return None;
            }
            p += tw;
            (0, tok)
        }
    };
    off = off.checked_add(eat_addr_offset_adds(seg, &mut p)?)?;

    // A cv strip or an array-to-pointer decay applied to the ADDRESS, which emits
    // nothing (`void f(S* s, int v){ *(int*)s = v; }` is a bare `stw r4,0(r3)`).
    // Pointer→pointer only: a cross-class `2C` here is a reinterpret this port has
    // never probed.
    if seg.get(p)? == &0x2C {
        let mut probe = p + 1;
        let (tag, kind, _, tw) = read_type(seg, probe)?;
        if !is_ptr_any(tag, kind) {
            return None;
        }
        probe += tw;
        if !eat_byte(seg, &mut probe, 0x00) {
            return None;
        }
        p = probe;
    }

    // THE VALUE — a bare formal or an integer literal, and nothing computed. A
    // computed value lands in the scratch register first (`s->m = a + b` is
    // `add r11,r3,r4 ; stw r11`), which is a different instruction count and has
    // no capture behind it here.
    let (value_op, mut value_tag, mut value_kind) = match *seg.get(p)? {
        0xB9 => {
            let mut probe = p + 1;
            let (tok, w) = read_token_var(seg, probe)?;
            probe += w;
            let (tag, kind, _, tw) = read_type(seg, probe)?;
            probe += tw;
            p = probe;
            (IlOp::Load(tok), tag, kind)
        }
        0x33 => {
            let mut probe = p + 1;
            let (tag, kind, _, tw) = read_type(seg, probe)?;
            probe += tw;
            let k = read_varint(seg, &mut probe)?;
            p = probe;
            (IlOp::Lit(k), tag, kind)
        }
        _ => return None,
    };
    // A **floating-point** stored value is `stfs`/`stfd` out of the FP argument
    // file, so it takes the whole rest of this production down a parallel path:
    // its register is not the formal's index, and the `2C` rules below are the
    // GPR classes'. MEASURED (`docs/CODEGEN_FP_ARGS.md` §3):
    //
    //     void s_f (S* s, float v)      { s->f = v; }        d0230004  stfs f1,4(r3)
    //     void s_d (S* s, double v)     { s->d = v; }        d8230008  stfd f1,8(r3)
    //     void s_two(S* s,float u,float v){ s->f = v; }      d0430004  stfs f2,4(r3)
    //
    // Sized before it was built, by counterfactual over the 878-TU workload:
    // **7,984 functions**, all `calls-0`.
    let fp_width = store_fp_value_width(value_tag, value_kind);
    if let Some(w) = fp_width {
        return finish_fp_store_leaf(seg, p, lo, base_tok, value_op, value_tag, value_kind, off, w, sy);
    }
    let width = store_value_width(value_tag, value_kind)?;

    // A class-preserving conversion of the VALUE — `void f(S* s, S* v){ s->p = v; }`
    // converts `S*` to `void*` on the way in and emits nothing (`90830008`, the same
    // bare `stw` as the unconverted neighbour). Admitted only in the two 4-byte
    // classes [`eat_value_type`] was byte-graded on since the getter rungs, and
    // **only** there: over a narrow value a `2C` is a real instruction —
    // `void M::setb(bool v){ m0 = v; }` (an `int` member, a `bool` parameter) emits
    // `clrlwi r11,r4,24 ; stw r11,0(r3)` — so `width != 4` refuses rather than
    // silently dropping the mask.
    if seg.get(p) == Some(&0x2C) {
        let cls = value_class(value_tag, value_kind)?;
        let mut probe = p + 1;
        let (t2, k2, _, _) = read_type(seg, probe)?;
        if !eat_value_type(seg, &mut probe, cls) || !eat_byte(seg, &mut probe, 0x00) {
            return None;
        }
        value_tag = t2;
        value_kind = k2;
        p = probe;
    }

    // The store, whose TYPE restates the value's.
    if !eat_byte(seg, &mut p, 0x32) {
        return None;
    }
    let (tag, kind, _, tw) = read_type(seg, p)?;
    if (tag, kind) != (value_tag, value_kind) {
        return None;
    }
    p += tw;
    // The statement end. A store yields its value and `4B` discards it; a body
    // that goes on to use it is not this shape.
    if !eat_byte(seg, &mut p, 0x4B) {
        return None;
    }
    eat_opt_stmt_marker(seg, &mut p);
    eat_return_plumbing(seg, &mut p, false, BODY_SCOPE_DEPTH).ok()?;

    if !(-0x8000..=0x7FFF).contains(&off) {
        return None;
    }
    // `std` is DS-form: the displacement's low two bits are the form's, so an
    // offset that is not a multiple of 4 cannot be encoded at all. Natural
    // alignment makes one unreachable through a struct member, so this gate has
    // no witness — which is exactly why it refuses instead of masking.
    if width == 8 && off % 4 != 0 {
        return None;
    }
    let params = parse_params(seg, lo).ok()?;
    let bix = params.iter().position(|&t| t == base_tok)?;
    // Past the eighth argument the value is stack-homed, which needs a frame.
    if bix >= 8 {
        return None;
    }
    match value_op {
        IlOp::Load(vtok) => {
            let vix = params.iter().position(|&t| t == vtok)?;
            // Past the eighth argument the value is stack-homed, which needs a frame.
            if vix >= 8 {
                return None;
            }
        }
        // A wide **negative** constant. `emit_load_imm`'s `lis`+`ori` pair covers
        // non-negative values only, and the straight-line class already refuses
        // this in the PARSER (`expr-out-of-class-wide-neg-lit`,
        // `chain::straight_line_out_of_class_ctx`). Restating the bound here rather
        // than letting codegen refuse it is the census/gate invariant: the same
        // literal reached two shapes and only one of them gated it, so
        // `void f(S* s){ s->a = -70000; }` censused in class while `PortC2`
        // returned `NotImplemented` — the `GAPS.md` §6 "one fact, two locators"
        // failure, caught by probing the new production's own boundary.
        IlOp::Lit(k) if k < -0x8000 => return None,
        _ => {}
    }
    Some(BodyShape::StoreLeaf {
        params,
        ops: vec![IlOp::Load(base_tok), value_op, IlOp::StoreInd { off, width }],
    })
}

/// The tail of [`try_parse_store_leaf`] for a **floating-point** stored value.
///
/// Split out rather than branched inline because almost every gate differs: the
/// value's register comes from the FP file, the conversion rules are the FP ones,
/// and a literal is a pooled `.rdata` COMDAT rather than an `li`.
///
/// What is REFUSED here, each because a capture shows it emits something else:
///
/// * **A conversion on the value.** `void s_narrow(S* s, double v){ s->f = v; }`
///   is `frsp f0,f1 ; stfs f0,4(r3)` — a real instruction through the FP scratch
///   register. Its free twin `void s_widen(S* s, float v){ s->d = v; }` is a bare
///   `stfd f1,8(r3)`, so the asymmetry is c2's own and not the C standard's; both
///   are refused, because admitting the free one means deciding the direction from
///   two type triples and only the narrowing one has been captured at more than
///   one offset. A rung, sized in `docs/CODEGEN_FP_ARGS.md` §5.
/// * **A literal value.** `void s_lit(S* s){ s->f = 1.5f; }` is
///   `lis r11 ; lfs f0,0(r11) ; stfs f0,4(r3)` with a REFHI/REFLO pair into an
///   `.rdata` COMDAT — the W13b constant machinery, which `codegen::function_gate`
///   refuses under `/Gy` anyway.
/// * **A value that is not a formal**, and a formal whose FP register the `.sy`
///   argument classes cannot determine.
#[allow(clippy::too_many_arguments)]
fn finish_fp_store_leaf(
    seg: &[u8],
    mut p: usize,
    lo: usize,
    base_tok: u32,
    value_op: IlOp,
    value_tag: u8,
    value_kind: u8,
    off: i32,
    width: u8,
    sy: SyView,
) -> Option<BodyShape> {
    // No conversion, and no pooled constant.
    if seg.get(p) == Some(&0x2C) {
        return None;
    }
    let IlOp::Load(vtok) = value_op else {
        return None;
    };
    // The store, whose TYPE restates the value's — the same literal requirement
    // the GPR path makes, and for the same reason.
    if !eat_byte(seg, &mut p, 0x32) {
        return None;
    }
    let (tag, kind, _, tw) = read_type(seg, p)?;
    if (tag, kind) != (value_tag, value_kind) {
        return None;
    }
    p += tw;
    if !eat_byte(seg, &mut p, 0x4B) {
        return None;
    }
    eat_opt_stmt_marker(seg, &mut p);
    eat_return_plumbing(seg, &mut p, false, BODY_SCOPE_DEPTH).ok()?;

    if !(-0x8000..=0x7FFF).contains(&off) {
        return None;
    }
    // `stfs`/`stfd` are both plain D-form — unlike `std`, which is DS-form and
    // cannot encode a displacement that is not a multiple of 4. So there is no
    // alignment gate here, and the absence is a measured difference between the
    // two paths rather than an omission (`d8230008` is `stfd f1,8(r3)`, primary
    // 54, with all sixteen displacement bits its own).
    let params = parse_params(seg, lo).ok()?;
    let bix = params.iter().position(|&t| t == base_tok)?;
    if bix >= 8 {
        return None;
    }
    // The value's FP register, resolved HERE — the one site that knows both the
    // formals order (`.ex`) and each formal's register file (`.sy`).
    let formals = parse_formals(seg, lo).ok()?;
    let classes = sy.arg_classes(&formals).ok()?;
    let fix = formals.iter().position(|&t| t == vtok)?;
    let src = fp_reg_of(&classes, fix)?;
    if src > 13 {
        // Past f13 the argument is stack-homed, which needs a frame.
        return None;
    }
    // The value's declared width and the stored width must be the same fact. They
    // are, at every capture, because a conversion is a visible `2C` that is
    // refused above — so a disagreement means a misread type, not a construct.
    if matches!(classes.get(fix), Some(ArgClass::Fp { double }) if *double != (width == 8)) {
        return None;
    }
    Some(BodyShape::StoreLeaf {
        params,
        ops: vec![
            IlOp::Load(base_tok),
            IlOp::StoreIndFp { off, double: width == 8, src },
        ],
    })
}

/// Try to parse the **compiler-generated empty destructor** that does nothing but
/// destroy **one** sub-object — the largest coherent sub-shape of the
/// `expr-call-in-expr` bucket (`docs/IL_CALL_IN_EXPR.md` §5, §15).
///
/// There are **two** such destructors and they differ only in how the sub-object's
/// address is spelled. `docs/IL_CALL_IN_EXPR.md` §14.3 separated them; §5 had seen
/// only the first:
///
/// * a **base** sub-object, whose address comes from the `this`-adjust intrinsic
///   2113 and whose adjustment this shape requires to be 0
///   (`RECV-BASE` below, D1);
/// * a **member** sub-object, whose address is a plain `27` byte-offset add of a
///   literal `k` onto `this` — no intrinsic anywhere, and `k` may be zero (the
///   member is first in the layout, so the address arithmetic emits nothing) or
///   nonzero (one `addi r3,r3,k`).
///
/// ```text
///   33 <int> 0                     the leading literal (role UNKNOWN — see below)
///   26 <method-tok>                the SUB-OBJECT destructor, pushed first
///   <RECV-BASE | RECV-MEMBER>      the receiver — one of:
///
///   RECV-BASE:
///     33 <int> 2113  40 <PTR4>     intrinsic `this-adjust`, pointer result
///     66 02 <2 LEB128 type refs>   the class-pair descriptor
///     55 <int>                     selector argument terminator
///     33 <int> 0     55 <int>      the adjust OFFSET — required to be 0
///     B9 <this> <PTR4>  55 <PTR4>  the object pointer
///     4C                           -> the adjusted receiver
///
///   RECV-MEMBER:
///     B9 <this> <PTR4>             the object pointer
///     33 <int> k                   the member's byte offset within the object
///     27 <PTR4>                    byte-offset add -> the member's address
///
///   2C <PTR4> 00                   cv strip
///   99 <PTR4> 00                   member bind (a `99` bind is DIRECT dispatch)
///   BD <void> 00 <fn-type-id>      the CALL, void result, cdecl
///   4C                             ZERO explicit arguments (`this` is not one)
///   5C <int> 01                    opaque statement trailer
///   4B                             statement end
///   3A <lbl> 54 02 29 <lbl>        the return plumbing's branch/close/return
///   5E 01 21                       opaque sub-object trailer
///   4B
///   <function tail, reaching the segment end>
/// ```
///
/// **Why it needs at most one instruction.** A `99` bind is direct dispatch by
/// construction (virtual dispatch is opcode `67` with a `9A` bind —
/// `docs/IL_CALL_IN_EXPR.md` §3), so the call is a direct branch; the call has no
/// result; and nothing follows it. So the whole function is the sub-object's
/// address in r3 followed by a tail branch, and the address is `this` (already in
/// r3, zero instructions) plus a constant. MEASURED at the workload's own
/// `/O1 /Oi /EHsc` for the base form and at the fixture profile for the member
/// forms (`work/rf/probes/p3.cpp`, `q4`, `q7`, `q8`):
///
/// ```text
///   struct B1{~B1();int x;};  struct D1:B1{~D1();int y;};  D1::~D1(){}
///   ??1D1@@QAA@XZ:       48000000  b ??1B1@@QAA@XZ         base, adjust 0
///
///   struct MemA{~MemA();int a;};
///   struct HasMem { ~HasMem();  MemA m; };        HasMem::~HasMem() {}
///   ??1HasMem@@QAA@XZ:   4bfffff0  b ??1MemA@@QAA@XZ       member at 0
///
///   struct HasMem4{ ~HasMem4(); int pad; MemA m; };  HasMem4::~HasMem4() {}
///   ??1HasMem4@@QAA@XZ:  38630004  addi r3,r3,4            member at 4
///                        4bffffe4  b ??1MemA@@QAA@XZ
/// ```
///
/// The `addi` is not a new emitter: the adjust is handed to codegen as the
/// argument-setup operand stream `[Load(this), Lit(k), Add]`, which is what
/// `return g(a + k)` already lowers through (`int_tail_call_text`), so the one new
/// instruction in this shape is emitted by code that four mode lanes and the
/// expression sweep have been grading since the MVP.
///
/// **`k` must fit a signed 16-bit `addi`.** MEASURED: a member at offset 40,000
/// (`work/rf/probes/q3.cpp`, `struct Big{~Big(); char pad[40000]; MemA m;}`) emits
/// **two** instructions, `addis r3,r3,1 ; addi r3,r3,-25536`, which is a second
/// production with one witness. It is refused, and `whole_body_is_one_value` counts
/// that body as complete, so the `-whole` census figure is an upper bound over this
/// gate too.
///
/// **Why this lands in `expr-call-in-expr` at all**: the body opens on the `33`
/// literal, so the straight-line arm runs `parse_expr`, pushes `Lit(0)` and stops
/// on the `26`. The very same production reached through a plain base-method call
/// (`p->Bm()`, no leading literal) opens on the `26`, is dispatched to the
/// assignment parser and files under `expr-intrinsic-this-adjust` — one
/// production split across two census buckets by one leading byte.
///
/// **The two opaque trailers.** `5C <int> <f>` and `5E <n> <g>` are undecoded, and
/// two of those three payload fields **vary** — which is the only reason this
/// grammar is worth writing down rather than transcribing:
///
/// * **`<n>` counts destroyed sub-objects.** MEASURED,
///   `struct N1 : M1, M2 { ~N1(); };` (two bases, each with a destructor) emits
///   *two* member-call statements — the second with a nonzero adjust offset,
///   needing an `addi` — and closes with `5E 02 21` rather than `5E 01 21`.
///   Requiring `01` is therefore a real discriminator against the shape this
///   lowering would get wrong, and it says structurally what the grammar says.
///   MEASURED again for the member form, and it is the gate that matters most
///   there: `struct Two { ~Two(); MemA m; MemB n; };` (two destructible members,
///   at offsets 0 and 4) carries `5E 02 31` and **two** statements each with its
///   own leading `33 <int> 0` literal, and the reference does *not* emit two
///   branches — it emits a **frame**, `or r31,r3,r3`, and the two `bl`s in
///   **reverse declaration order** (`work/rf/probes/q1.cpp`):
///
///   ```text
///     ??1Two@@QAA@XZ: mflr/stw/stwu … ; or r31,r3,r3 ; addi r3,r3,4
///                     bl ??1MemB@@QAA@XZ ; or r3,r31,r31 ; bl ??1MemA@@QAA@XZ
///                     addi r1,r1,96 ; lwz r12,-8(r1) ; mtlr r12 ; … ; blr
///   ```
///
///   `this` is live across the first call, so that shape needs a frame, a
///   callee-saved register and a call order this rung does not model. It is the
///   shape `docs/IL_CALL_IN_EXPR.md` §14.3 measured as 574 bodies "lost" to the
///   offset split, and the loss is real rather than an artifact: those bodies are
///   *grammar*-complete with both offsets admitted and *codegen*-complete under
///   neither.
/// * **`<f>` and `<g>` carry an exception-handling bit, and they co-vary.**
///   MEASURED by isolating one flag at a time over
///   `{/Od, /O1, /Ox} × {—, /Oi, /GS-, /GR, /EHsc, /EHa}`: **`/EH…` clears bit
///   `0x10` in both**, and nothing else in that matrix moves either byte. So the
///   fixture profile (`/Ox`, no `/EH`) gives `5C … 11` / `5E 01 31` and the dc3
///   workload profile (`/O1 /Oi /EHsc`) gives `5C … 01` / `5E 01 21`, and the
///   reference emits the **same four bytes** for both (checked at `/Ox`,
///   `/Ox /EHsc` and `/O1`). Both pairs are admitted, as a two-entry table of
///   measured values with the bit required to agree between them — not as a
///   skipped field. Had this been pinned to the one profile that was probed
///   first, the shape would have refused the entire workload or the entire
///   fixture lane depending on which one that was.
///
/// What the bit *means* is still UNKNOWN, and a third value refuses. The
/// separating probe for `<f>`'s low nibble would be a destructor of a class with
/// a virtual base, where MSVC's vbase-destruct flag should move it; not tested.
///
/// Each remaining gate, with the neighbour it separates (all MEASURED at
/// `/O1 /Oi /EHsc` against the live 16.00.11886.00 toolchain):
///
/// * **Selector exactly 2113 in its wide form** (base form only). 2113–2119 are
///   seven different operations and only this one is an unguarded adjust; a
///   *virtual* destructor goes through 2117/2116 and a whole different body
///   (`struct N3 : M4 { virtual ~N3(); };`, which blocks as
///   `expr-intrinsic-base-member-addr` and must keep doing so).
/// * **The base form's adjust offset must be 0.** A base at a nonzero offset is
///   reached only by a multi-base destructor, which has two calls and fails the
///   skeleton first, so there is no single-call witness for it. The *member* form's
///   offset is admitted nonzero because there is one — several, above — and the
///   two are separate literals in separate productions, so the base gate is not
///   loosened by the member one.
/// * **The descriptor must be exactly `66 02` + two refs** (base form only). Every
///   witness — a direct base, a two-level chain (`D4 : B4 : G4`), an empty base, a
///   multi-base class — carries `02`, because a destructor delegates exactly one
///   inheritance step. A field that never varied is required literally rather
///   than skipped structurally on the assumption that the shape keeps repeating.
/// * **The member form's offset must be non-negative and fit `addi`.** Zero is the
///   commonest case and emits nothing. A negative adjust has no witness at all —
///   only a virtual-base thunk would plausibly produce one, and that is a
///   different production — so it fails closed rather than being sign-extended on
///   the assumption that `addi` would do the right thing.
/// * **The member form admits `27` only, not `28 00 00`.** Both are byte-offset
///   adds and D2's classifier accepts either, but every captured generated
///   destructor carries the typed `27`; `28` is the subscript spelling
///   (`docs/IL_EXPR_LAYER.md` §4) and has no witness here.
/// * **The call must be `void`, cdecl, and carry ZERO explicit arguments.** The
///   receiver rides the operand stack into the `99`; a `55`-terminated argument
///   here would be a different callee.
/// * **The receiver must be the function's `this`, positively bound, and there
///   must be no other formal.** That is what puts the branch target's `this` in
///   r3 with no register move. `parse_params` refuses an undetermined `this`
///   (the line-70 rule).
/// * **The parse must reach the segment end.** A destructor with a real statement
///   (`N2::~N2() { h(); }`) has a second `26` where the plumbing must begin, and a
///   class with a destructible base *and* a destructible member (`N4 : M5 { M6 m; }`)
///   has a second member-call statement; both refuse, and both really do emit a
///   frame and two `bl`s.
///
/// Returns `None` — cursor untouched — for anything that is not exactly this
/// shape.
pub(crate) fn try_parse_empty_dtor_delegation(
    seg: &[u8],
    start: usize,
    lo: usize,
    depth: usize,
) -> Option<BodyShape> {
    /// The `void` result TYPE, required literally: this shape's whole licence to
    /// emit a bare branch is that there is no result to place.
    const VOID_TYPE: [u8; 3] = [0x82, 0x07, 0x03];
    /// The measured `(statement-trailer flag, sub-object-trailer flag)` pairs:
    /// `/EH…` clears bit `0x10` in both, and they always agree. Anything else
    /// refuses. See the doc comment.
    const TRAILER_FLAGS: [(u8, u8); 2] = [(0x11, 0x31), (0x01, 0x21)];

    let mut p = start;
    // The leading literal. Its role is UNKNOWN; it is required to be int-typed
    // and exactly zero, which is what every witness carries.
    if !eat_byte(seg, &mut p, 0x33) || !eat(seg, &mut p, &INT_TYPE) {
        return None;
    }
    if read_varint(seg, &mut p)? != 0 {
        return None;
    }
    // The sub-object destructor's symbol, pushed before its receiver.
    if !eat_byte(seg, &mut p, 0x26) {
        return None;
    }
    let (callee_tok, w) = read_token_var(seg, p)?;
    p += w;
    // The receiver: the base form's intrinsic frame, or the member form's plain
    // byte-offset add. Tried base-first and each on its own cursor copy, because
    // the two open on different bytes (`33` vs `B9`) and neither may leave the
    // cursor moved for the other.
    let save = p;
    let (recv_tok, adjust, sub_object) = match eat_dtor_base_receiver(seg, &mut p) {
        Some(tok) => (tok, 0, DtorSubObject::Base),
        None => {
            p = save;
            let (tok, k) = eat_dtor_member_receiver(seg, &mut p)?;
            (tok, k, DtorSubObject::Member)
        }
    };
    // The cv strip on the receiver, then the member bind. A `2C`
    // pointer→pointer emits nothing (`docs/IL_LOAD_TYPES.md` §3), and a `99`
    // bind is direct dispatch.
    if !eat_byte(seg, &mut p, 0x2C)
        || !eat_value_type(seg, &mut p, ValueClass::Ptr4)
        || !eat_byte(seg, &mut p, 0x00)
    {
        return None;
    }
    if !eat_byte(seg, &mut p, 0x99)
        || !eat_value_type(seg, &mut p, ValueClass::Ptr4)
        || !eat_byte(seg, &mut p, 0x00)
    {
        return None;
    }
    // The CALL: void result, cdecl, then the per-TU function-type id (decoded
    // only to find the token's end — it does not name the callee).
    if !eat_byte(seg, &mut p, 0xBD) || !eat(seg, &mut p, &VOID_TYPE) || !eat_byte(seg, &mut p, 0x00)
    {
        return None;
    }
    read_varint(seg, &mut p)?;
    // Zero explicit arguments, then the two opaque trailers and the statement end.
    if !eat_byte(seg, &mut p, 0x4C) {
        return None;
    }
    if !eat(seg, &mut p, &[0x5C]) || !eat(seg, &mut p, &INT_TYPE) {
        return None;
    }
    let stmt_flag = *seg.get(p)?;
    let (_, want_subobject) = TRAILER_FLAGS.iter().copied().find(|&(f, _)| f == stmt_flag)?;
    p += 1;
    if !eat_byte(seg, &mut p, 0x4B) {
        return None;
    }
    // The return plumbing, with the sub-object trailer wedged between the `29`
    // return and the function tail — which is why this cannot call
    // `eat_return_plumbing` and shares only [`eat_fn_tail`] with it.
    let mut depth = depth;
    eat_scopes(seg, &mut p, &mut depth).ok()?;
    if !eat_byte(seg, &mut p, 0x3A) {
        return None;
    }
    let (label, w) = read_token_var(seg, p)?;
    p += w;
    for d in (BODY_SCOPE_DEPTH..=depth).rev() {
        eat_opt_stmt_marker(seg, &mut p);
        if !eat(seg, &mut p, &[0x54, d as u8]) {
            return None;
        }
    }
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x29) {
        return None;
    }
    let (back, w) = read_token_var(seg, p)?;
    p += w;
    // The branch and the return name the same label at every witness. Required,
    // for the same reason as everything else here: it is what the bytes say.
    if back != label {
        return None;
    }
    // `5E <n> <g>`: exactly one destroyed sub-object, and its EH bit must agree
    // with the statement trailer's.
    if !eat(seg, &mut p, &[0x5E, 0x01, want_subobject]) || !eat_byte(seg, &mut p, 0x4B) {
        return None;
    }
    eat_fn_tail(seg, &mut p).ok()?;

    // `this` in r3, no explicit formals, and the receiver IS that `this`.
    let params = parse_params(seg, lo).ok()?;
    if params.as_slice() != [recv_tok] {
        return None;
    }
    Some(BodyShape::EmptyDtorDelegation { callee_tok, this_tok: recv_tok, adjust, sub_object })
}

/// The **base** sub-object's receiver: the `this`-adjust intrinsic 2113 at
/// adjustment 0, whose result is the base's address. Returns the object-pointer
/// token. See [`try_parse_empty_dtor_delegation`]'s `RECV-BASE`.
fn eat_dtor_base_receiver(seg: &[u8], p: &mut usize) -> Option<u32> {
    /// `33 <int> 80 41 08 00 00` — selector 2113 `this-adjust`, wide form.
    const SELECTOR_2113: [u8; 5] = [0x80, 0x41, 0x08, 0x00, 0x00];

    // The `this`-adjust intrinsic, whose result is the receiver.
    if !eat_byte(seg, p, 0x33)
        || !eat(seg, p, &INT_TYPE)
        || !eat(seg, p, &SELECTOR_2113)
        || !eat_byte(seg, p, 0x40)
    {
        return None;
    }
    let (tag, kind, _, tw) = read_type(seg, *p)?;
    if !is_ptr4_kind(tag, kind) {
        return None;
    }
    *p += tw;
    // The class-pair descriptor: exactly two type references, whose ids are
    // **LEB128** and not a fixed two bytes each ([`super::mcall::eat_class_descriptor`]).
    //
    // Stepping four bytes here is what made this shape refuse bodies that are its
    // own skeleton byte for byte, in every translation unit large enough to have
    // wide type ids — `src/App.cpp` carries one. It was found by a residue: the D2
    // split first spread 17,757 functions over 197 `op-0xNN` buckets, and flat over
    // the byte range is the signature of reading a payload as vocabulary.
    let n_refs = mcall::eat_class_descriptor(seg, p)?;
    if n_refs != 2 {
        return None;
    }
    if !eat_byte(seg, p, 0x55) || !eat(seg, p, &INT_TYPE) {
        return None;
    }
    // The adjust offset — zero, or this needs an `addi`. Unlike the member form's
    // offset this one stays pinned at zero: the only nonzero-adjust base is the
    // second base of a multi-base destructor, which has two calls and no
    // single-branch witness.
    if !eat_byte(seg, p, 0x33) || !eat(seg, p, &INT_TYPE) {
        return None;
    }
    if read_varint(seg, p)? != 0 {
        return None;
    }
    if !eat_byte(seg, p, 0x55) || !eat(seg, p, &INT_TYPE) {
        return None;
    }
    // The object pointer.
    if !eat_byte(seg, p, 0xB9) {
        return None;
    }
    let (recv_tok, w) = read_token_var(seg, *p)?;
    *p += w;
    if !eat_value_type(seg, p, ValueClass::Ptr4) {
        return None;
    }
    if !eat_byte(seg, p, 0x55) || !eat_value_type(seg, p, ValueClass::Ptr4) {
        return None;
    }
    if !eat_byte(seg, p, 0x4C) {
        return None;
    }
    Some(recv_tok)
}

/// The **member** sub-object's receiver: `this` plus a literal byte offset through
/// a plain `27` add, with no intrinsic anywhere. Returns
/// `(object-pointer token, offset)`. See [`try_parse_empty_dtor_delegation`]'s
/// `RECV-MEMBER`.
///
/// The offset is required to be non-negative and to fit a signed 16-bit `addi`,
/// which is the whole codegen difference between this receiver and the base one —
/// and the boundary is measured, not assumed: a member at offset 40,000 emits
/// `addis r3,r3,1 ; addi r3,r3,-25536` (`work/rf/probes/q3.cpp`).
fn eat_dtor_member_receiver(seg: &[u8], p: &mut usize) -> Option<(u32, i32)> {
    // The object pointer. `this` is `A6`-tagged in a destructor and `86`-tagged
    // through a non-const path; `ValueClass::Ptr4` admits both and refuses the
    // width-8 and aggregate spellings.
    if !eat_byte(seg, p, 0xB9) {
        return None;
    }
    let (recv_tok, w) = read_token_var(seg, *p)?;
    *p += w;
    if !eat_value_type(seg, p, ValueClass::Ptr4) {
        return None;
    }
    // The member's byte offset within the object, as an int literal.
    if !eat_byte(seg, p, 0x33) || !eat(seg, p, &INT_TYPE) {
        return None;
    }
    let adjust = read_varint(seg, p)?;
    if adjust < 0 || i16::try_from(adjust).is_err() {
        return None;
    }
    // `27 <PTR4>` — the typed byte-offset add. Not `28 00 00`: that spelling has
    // no witness in this production.
    if !eat_byte(seg, p, 0x27) || !eat_value_type(seg, p, ValueClass::Ptr4) {
        return None;
    }
    Some((recv_tok, adjust))
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
    // emitter can emit — through [`CompareLeaf::out_of_class_ctx`], which is the
    // one locator both sides share. Two of these three clauses used to be spelled
    // out again right here, and the third (a large UNSIGNED literal under
    // `==`/`!=`) was in codegen only, so `int f(unsigned a){ return a ==
    // 4294967295u; }` censused in class and the port refused it.
    let cmp = CompareLeaf { param, rel, signed, k };
    if cmp.out_of_class_ctx().is_some() {
        return None;
    }
    Some(BodyShape::Compare(cmp))
}

/// The non-trivial cycles of the argument permutation `sources`, as
/// `(count, longest)`. `sources[i]` is the formal index argument slot `i` wants,
/// so a fixed point is a value already in place.
///
/// `sources` must already have been proved to index inside itself
/// ([`tail_call_shape`]'s `call-arg-outer-formal` gate); this walk indexes `seen`
/// with an entry, so an out-of-range one **panics** rather than refusing. It did:
/// see that gate's comment.
fn permutation_cycles(sources: &[usize]) -> (usize, usize) {
    let n = sources.len();
    let mut seen = vec![false; n];
    let mut cycles = 0usize;
    let mut longest = 0usize;
    for start in 0..n {
        if seen[start] || sources[start] == start {
            seen[start] = true;
            continue;
        }
        let mut at = start;
        let mut len = 0usize;
        while !seen[at] {
            seen[at] = true;
            len += 1;
            at = sources[at];
        }
        cycles += 1;
        longest = longest.max(len);
    }
    (cycles, longest)
}

/// The longest argument-permutation cycle `c2_core::codegen::permute_args_text`
/// has been **verified** to lower, measured over complete grids rather than
/// sampled: all 24 permutations of a four-argument call and all 84 single cycles
/// of length 2–5 inside a five-argument one.
///
/// ```text
///   cycle length 2    0 mismatch / 10 cases
///   cycle length 3    0 mismatch / 20
///   cycle length 4   10 mismatch / 30
///   cycle length 5   16 mismatch / 24
/// ```
///
/// Past three, c2 does not use the minimal single-temp walk the port emits. It
/// hoists a **second** save into r10 and writes the destinations in a different
/// order — `int f(int a,int b,int c,int d){ return a4(c,d,b,a); }` is
///
/// ```text
///   7cab2b78  mr r11,r5      7cca3378  mr r10,r6
///   7c661b78  mr r6,r3       7c852378  mr r5,r4
///   7d445378  mr r4,r10      7d635b78  mr r3,r11      six moves, two temps
/// ```
///
/// against the port's five-move single-temp walk — a **live wrong-bytes emit on
/// mainline** (`Port=Mismatch @ 8`), independent of any framed shape. Twenty of
/// the thirty four-cycles happen to agree with the minimal walk and ten do not,
/// so "it worked on the fixtures" was luck of the sample: `il_call_perm.cpp` and
/// `il_call_multi.cpp` between them hold no cycle longer than three.
///
/// The order c2 actually picks past three is **not characterized** — the six
/// four-cycles split four/two on a property the grid describes but does not
/// explain — so the boundary is drawn at the measured edge rather than fitted.
pub(crate) const MAX_VERIFIED_PERM_CYCLE: usize = 3;

/// **One locator for "are these call arguments a tail call this port can emit?"**
/// — the validation and the shape construction for `return g(…)` in every
/// position it appears: the direct form, the bound-to-a-local form
/// (`int z = g(…); return z;`), and the single statement call that is a whole
/// body (`void f(int a){ g(a); }`, which c2 lowers to a bare `b g`).
///
/// It exists because those paths carried **two copies** of the checks and the
/// copies had drifted apart in both directions — each copy was missing a gate the
/// other had, and each omission was live:
///
/// * **A wrong-bytes emit.** `int f(int a,int b){ int z = g(b + a); return z; }`
///   emitted `add r3,r4,r3` against the reference's `add r3,r3,r4`: c2
///   canonicalizes the leaves of a commutative argument expression, so `g(a+b)`
///   and `g(b+a)` are the **same** obj. The direct form `return g(b + a);`
///   refuses on [`leaves_ascending`] and always has; the bound-to-a-local copy
///   never asked. `Port=Match` for `a+b`, `Port=Mismatch @ 537` for `b+a`, from
///   two lines of C++ that differ by one transposition.
/// * **A panic.** `int f(int a,int b,int c){ int z = g2(a, c); return z; }` took
///   `c2rs census` down with `index out of bounds: the len is 2 but the index is
///   2` — [`permutation_cycles`] indexed its `seen` array with a *formal* index
///   past the argument count. The direct form got the `call-arg-outer-formal`
///   gate when that was found (`docs/GAPS.md` §6); this copy did not, and the CLI
///   must degrade cleanly, never panic.
///
/// Same family as every other entry in `docs/GAPS.md` §6: one fact, two
/// implementations, and the corpus only ever exercised the fixed one.
///
/// `args` is the argument list in **stream order** (reverse source order, so slot
/// `i` is `args[len-1-i]`); `params` is the formals list with a member function's
/// `this` at index 0; `off` is the segment offset a refusal reports.
fn tail_call_shape(
    args: Vec<Vec<IlOp>>,
    params: Vec<u32>,
    callee_tok: u32,
    off: usize,
) -> Result<BodyShape, Block> {
    let refuse = |ctx: &'static str| Block { ctx, byte: None, off, aux: 0 };
    // No arguments at all: the bare `b <callee>`.
    if args.is_empty() {
        return Ok(BodyShape::VoidTailCall { callee_tok });
    }
    if args.len() > 1 {
        // Two or more arguments: only the pure-permutation shape is modeled. Every
        // argument must be a bare parameter LOAD — a computed argument would need
        // its own register and interacts with the permutation temp in ways no
        // capture covers yet.
        let mut arg_sources = Vec::with_capacity(args.len());
        for slot in 0..args.len() {
            let ops = &args[args.len() - 1 - slot];
            let tok = match ops.as_slice() {
                [IlOp::Load(t)] => *t,
                _ => return Err(refuse("call-arg-computed")),
            };
            match params.iter().position(|&t| t == tok) {
                Some(ix) => arg_sources.push(ix),
                // An argument that is not one of this function's formals (a local,
                // a global, a nested call result).
                None => return Err(refuse("call-arg-nonformal")),
            }
        }
        // **An argument that is a formal beyond the argument count.** `arg_sources`
        // indexes the *formals* list while everything below treats it as a
        // permutation of the *argument* slots, and the two lists are only the same
        // length when the call passes every formal. `int f(int a,int b,int c){
        // return g(a,c); }` gives sources `[0, 2]` over two slots: not a
        // permutation but a move out of a register the call does not otherwise
        // touch, which `permute_args_text` has no case for — and it indexed
        // [`permutation_cycles`]'s `seen` array out of bounds, i.e. **panicked**.
        if arg_sources.iter().any(|&ix| ix >= arg_sources.len()) {
            return Err(refuse("call-arg-outer-formal"));
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
                return Err(refuse("call-arg-duplicated"));
            }
        }
        let (cycles, longest) = permutation_cycles(&arg_sources);
        if cycles > 1 {
            return Err(refuse("call-arg-multicycle"));
        }
        // Past a three-element cycle c2 stops using the minimal single-temp walk
        // and hoists a second save into r10 — a live wrong-bytes emit, measured
        // over the complete 4- and 5-argument grids ([`MAX_VERIFIED_PERM_CYCLE`]).
        if longest > MAX_VERIFIED_PERM_CYCLE {
            return Err(refuse("call-arg-long-cycle"));
        }
        return Ok(BodyShape::MultiArgTailCall { params, arg_sources, callee_tok });
    }
    let arg_ops = args.into_iter().next().expect("exactly one argument");
    // The single call argument is an ordinary operand stream, so it is subject to
    // the same rewriter: `g(a + a)` is not `add` + branch.
    if has_repeated_leaf(&arg_ops) {
        return Err(refuse("call-arg-repeated-leaf"));
    }
    // And to the same reassociation: `g(b + a)` is not the source order either —
    // c2 canonicalizes the leaves and emits `add r3,r3,r4` for both orders. The
    // gate is vacuous for a single leaf (one leaf cannot be out of order), which is
    // why it asks the load count first.
    let n_loads = arg_ops.iter().filter(|o| matches!(o, IlOp::Load(_))).count();
    if n_loads > 1 && !leaves_ascending(&arg_ops, &params) {
        return Err(refuse("call-arg-noncanonical-order"));
    }
    if !additive_chain_canonical(&arg_ops) {
        return Err(refuse("call-arg-noncanonical-order"));
    }
    if !arg_loads_are_formals(&arg_ops, &params) {
        return Err(refuse("call-arg-nonformal"));
    }
    // The argument is computed into r3 by `c2_core::codegen::select_text`, the
    // same selector a straight-line leaf's body goes through, so it is subject to
    // **exactly the same** out-of-class rules — and those lived only in codegen for
    // this position. Measured: `int f(int a){ return g(a * 5); }` censuses 1/1 and
    // the port returns `NotImplemented` (a constant multiply strength-reduces to
    // shifts and adds), on mainline, in both directions of every fixture lane. A
    // census that over-claims is a broken instrument and the widening order is
    // chosen from it, so the predicate is asked here instead of there.
    //
    // Zero functions on the 878-TU workload, which is why the scan's disagreement
    // counter never saw it: it took a generated probe of the class's neighbours.
    if let Some(ctx) = straight_line_out_of_class_ctx(&arg_ops, &params) {
        return Err(Block { ctx, byte: None, off, aux: 0 });
    }
    Ok(BodyShape::IntTailCall { params, arg_ops, callee_tok })
}

/// Consume one **call header** — `26 <callee-tok> BD <ret TYPE> <conv> <varint
/// fn-type-id>` — and return the callee token.
///
/// Split out of [`parse_call_shape`] byte for byte so the statement-call sequence
/// ([`parse_call_sequence`]) reads the second and later calls through the same
/// decoder rather than a copy of it. Every refusal key is unchanged.
fn eat_call_head(seg: &[u8], p: &mut usize) -> Result<u32, Block> {
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
                aux: id as u64,
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
    Ok(callee_tok)
}

/// Consume a call's **argument region** — `( expr 55 <TYPE> )* 4C` — and return
/// one operand stream per argument, in stream order.
///
/// Split out of [`parse_call_shape`] byte for byte, for the same reason
/// [`eat_call_head`] is. Every refusal key is unchanged.
fn eat_call_args(seg: &[u8], p: &mut usize) -> Result<Vec<Vec<IlOp>>, Block> {
    let mut args: Vec<Vec<IlOp>> = Vec::new();
    loop {
        if eat_byte(seg, p, 0x4C) {
            break;
        }
        let ops = parse_expr(seg, p, 0x55)?;
        // `55 <TYPE>` carries the **formal's declared type**, and it is widened in
        // step with the operand positions: a call whose argument is a pointer
        // spells it here as well as at the `B9` (`… B9 p 86 43 f4 08 · 55 86 43
        // f4 08 · 4C`, captured from `int h1(int*); int f(int* p){return h1(p);}`),
        // so admitting one without the other admits no real call site at all —
        // measured: widening only `parse_expr` moved 1,013,468 functions between
        // census keys and gained exactly **0**. The argument is in a register
        // either way; this position is an annotation, not a lowering choice.
        if !eat_byte(seg, p, 0x55) || eat_int_like_or_ptr4(seg, p).is_none() {
            // an argument whose terminator or formal type we do not model
            return Err(blk(seg, *p, "call-end"));
        }
        args.push(ops);
        if args.len() > 8 {
            // Past the eighth the arguments are stack-homed, which needs a frame.
            return Err(Block { ctx: "call-args-overflow", byte: None, off: *p, aux: 0 });
        }
    }
    Ok(args)
}

/// The most formals a body this port emits may declare: past the eighth a
/// parameter is stack-homed and reading it is `lwz rD,<slot>(r1)`, not a register
/// move, which [`crate`]'s consumer `c2_core::codegen::select_text` refuses. Kept
/// in the parser so the census and the gate cannot disagree about it (the
/// under-claiming direction of `docs/GAPS.md` §6).
const MAX_REGISTER_FORMALS: usize = 8;

/// Parse a call shape (already positioned at the `26 <tok>` function ref): the
/// bare terminal void call, an integer tail call `return g(<arg>)` (passthrough
/// or arg-setup, plus the `g(a)+0` identity fold), the framed
/// `return g(a) + k` (k ≠ 0), or — the moment a call's result is *discarded* and
/// the body carries on — the Class A statement-call sequence
/// ([`parse_call_sequence`]). See [`parse_segment`] for the grammar; fail-closed
/// at every step. `lo` locates the formals for the arg-setup.
pub(crate) fn parse_call_shape(
    seg: &[u8],
    p: &mut usize,
    lo: usize,
    bound_to: Option<u32>,
) -> Result<BodyShape, Block> {
    let callee_tok = eat_call_head(seg, p)?;

    // VOID terminal tail call: the `4C 4B` void call-end immediately follows the
    // CALL token (no argument setup, no consumed value), then only return
    // plumbing (no result type).
    //
    // `g(); g();` and `g(); return a+1;` used to fail right here — a second `26`
    // call or a `B9` statement stands where the return plumbing must. The first of
    // those is now the Class A sequence below; the return-plumbing attempt is
    // therefore made on a **copy** of the cursor, so a body that really is the
    // single terminal call still takes this arm and still emits the bare `b g`.
    if eat(seg, p, &[0x4C, 0x4B]) {
        let mut q = *p;
        if eat_return_plumbing(seg, &mut q, false, BODY_SCOPE_DEPTH).is_ok() {
            *p = q;
            return Ok(BodyShape::VoidTailCall { callee_tok });
        }
        if bound_to.is_none() {
            return parse_call_sequence(seg, p, lo, callee_tok, Vec::new());
        }
        // Preserve the original refusal for the bound-to-a-local production,
        // which has no statement-sequence form.
        eat_return_plumbing(seg, p, false, BODY_SCOPE_DEPTH)?;
        unreachable!("the plumbing parse just failed on the same cursor");
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
    let mut args = eat_call_args(seg, p)?;
    // A call whose result is **discarded** (`4B` where the value would be
    // consumed): either the whole body — `void f(int a){ g(a); }`, which c2 tail-
    // calls exactly like the zero-argument form above — or the first statement of
    // a Class A sequence.
    if seg.get(*p) == Some(&0x4B) && bound_to.is_none() {
        *p += 1;
        let mut q = *p;
        if eat_return_plumbing(seg, &mut q, false, BODY_SCOPE_DEPTH).is_ok() {
            *p = q;
            let params = parse_params(seg, lo)?;
            return tail_call_shape(args, params, callee_tok, *p);
        }
        return parse_call_sequence(seg, p, lo, callee_tok, args);
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
        // The SAME validator the direct `return g(…)` form uses. This branch used
        // to carry its own copy, which was missing two of its gates — one wrong
        // byte and one panic; see [`tail_call_shape`].
        return tail_call_shape(args, params, callee_tok, *p);
    }
    if args.len() > 1 {
        // Two or more arguments: only the pure-permutation shape is modeled, and
        // only as a tail call — validated through the one locator
        // ([`tail_call_shape`]) the bound-to-a-local form and the statement-call
        // form also use.
        let params = parse_params(seg, lo)?;
        let shape = tail_call_shape(args, params, callee_tok, *p)?;
        // Only a terminal tail call: a post-op would consume the result and need
        // the framed path, which does not model multi-argument setup.
        if seg.get(*p) != Some(&0x41) {
            return Err(Block { ctx: "call-multiarg-postop", byte: None, off: *p, aux: 0 });
        }
        eat_return_plumbing(seg, p, true, BODY_SCOPE_DEPTH)?;
        return Ok(shape);
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
        return tail_call_shape(vec![arg_ops], params, callee_tok, *p);
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
        return tail_call_shape(vec![arg_ops], params, callee_tok, *p);
    }
    // A genuine `+ k` (k ≠ 0) is a framed non-leaf call — but the 6-section
    // framed path models only a **bare passthrough argument** (`g(a) + k`), not
    // arg-setup. `g(a+1) + 1` (a computed argument AND a framed post-op) is out
    // of class → reject (fail closed), never a mis-emitted framed obj.
    // The framed path takes a bare passthrough LOAD, which must still be a formal:
    // `int gi; g(gi) + 1` is a global read, not an argument already in r3.
    if matches!(arg_ops.as_slice(), [IlOp::Load(_)]) {
        let params = parse_params(seg, lo)?;
        if !arg_loads_are_formals(&arg_ops, &params) {
            return Err(Block { ctx: "call-arg-nonformal", byte: None, off: *p, aux: 0 });
        }
        // Past the eighth formal the value is stack-homed and its argument setup
        // is `lwz r3,<slot>(r1)`, not a register move — measured:
        // `int f(int a,…,int i){ return g(i) + 1; }` is `lwz r3,180(r1)`, and the
        // constant-body emitter used to emit *nothing* there.
        //
        // The refusal is the whole formals LIST, not just an argument past the
        // eighth, because that is the predicate `select_text` — which computes
        // this setup — actually raises. Refusing on the argument's index alone
        // would put the two out of step and re-open the census/gate disagreement
        // in the under-claiming direction (`docs/GAPS.md` §6). It is more
        // conservative than the ABI requires: `int f(int a,…,int i){ return g(a)
        // + 1; }` has its argument in r3 and would emit the plain body. Sized on
        // the 878-TU workload: **zero** functions, numerator unchanged either
        // way.
        if params.len() > MAX_REGISTER_FORMALS {
            return Err(Block { ctx: "framed-arg-over-eight-formals", byte: None, off: *p, aux: 0 });
        }
        // The formals list is carried, not dropped: the argument is *a* formal
        // but not necessarily the one already in r3, and c2 emits `or r3,rN,rN`
        // when it is not. Dropping the list here is how that word went missing
        // — see `c2_core::codegen::framed_call_text`.
        return Ok(BodyShape::FramedCall { add_k: k, callee_tok, params, arg_ops });
    }
    Err(Block { ctx: "framed-computed-arg", byte: None, off: *p, aux: 0 })
}

/// Parse the **Class A statement-call sequence** (`docs/GAPS.md` #35 step 2,
/// rung 1), positioned just past the first call's discarding `4B`.
///
/// ```text
///   seq  := stmt_call+ tail
///   stmt_call := <call head> <args> `4B`
///   tail := <void return plumbing>                          void body
///          | <call head> <args> [`33` <int> k `02`] <plumbing(result)>
///                                                           the last call's value
///          | `33` <int> k <plumbing(result)>                 `return <literal>;`
/// ```
///
/// Everything here is measured against real objs; the shapes and their bytes are
/// on [`BodyShape::CallSeq`]. Three facts this production turns on, each pinned by
/// a capture rather than assumed:
///
/// * **A single statement call with nothing after it is a TAIL call**
///   (`void f(int a){ g(a); }` → a bare `b ?g`, 5 sections, no frame), so the
///   caller tries the return plumbing before entering here and this function is
///   only ever reached with more body to parse. Emitting a frame for it would be
///   a mis-emit, not a gap.
/// * **The last call of a framed body is NOT tail-called.** `int f(){ g1();
///   return g2(); }` ends `bl ?g2 ; addi r1,r1,96 ; … ; blr`. The transform is off
///   once the function is framed.
/// * **Class A means no formal is read after the first call.** The first call's
///   arguments are evaluated before its `bl`, so a formal used only there dies
///   with it; a formal read by any later statement has to survive a call and c2
///   puts it in `r31` with a `std`/`ld` pair — Class B, a later rung, refused here
///   by name.
fn parse_call_sequence(
    seg: &[u8],
    p: &mut usize,
    lo: usize,
    first_callee: u32,
    first_args: Vec<Vec<IlOp>>,
) -> Result<BodyShape, Block> {
    let params = parse_params(seg, lo)?;
    // Past the eighth formal a parameter is stack-homed and `select_text` — which
    // computes every one of these calls' argument setups — refuses. Raised here so
    // the census cannot claim a body the gate declines (`docs/GAPS.md` §6, the
    // under-claiming direction).
    if params.len() > MAX_REGISTER_FORMALS {
        return Err(Block { ctx: "callseq-over-eight-formals", byte: None, off: *p, aux: 0 });
    }
    let mut raw: Vec<(u32, Vec<Vec<IlOp>>)> = vec![(first_callee, first_args)];
    let tail;
    loop {
        eat_opt_stmt_marker(seg, p);
        // (1) The body ends here: void return plumbing.
        {
            let mut q = *p;
            if eat_return_plumbing(seg, &mut q, false, BODY_SCOPE_DEPTH).is_ok() {
                *p = q;
                tail = SeqTail::Void;
                break;
            }
        }
        // (1b) …the same, written with an explicit `return;`. c2 records the
        // fallthrough as a SECOND `3A <label>` branch *to the same label* the
        // return plumbing then uses, and emits nothing for it: the two objs are
        // **byte-identical** (1090 B each, compared whole with the source path
        // held fixed and the timestamp zeroed).
        //
        // Requiring the two labels to MATCH is the whole gate. A real early
        // return branches somewhere else, and admitting that would drop a control
        // transfer on the floor — the difference between a no-op and a mis-emit is
        // exactly this token compare.
        if seg.get(*p) == Some(&0x3A) {
            if let Some((first, w)) = read_token_var(seg, *p + 1) {
                let mut q = *p + 1 + w;
                let same = seg.get(q) == Some(&0x3A)
                    && read_token_var(seg, q + 1).is_some_and(|(t, _)| t == first);
                if same && eat_return_plumbing(seg, &mut q, false, BODY_SCOPE_DEPTH).is_ok() {
                    *p = q;
                    tail = SeqTail::Void;
                    break;
                }
            }
        }
        // (2) `return <literal>;` — one `li r3,k` after the last `bl`. A literal is
        // the ONLY expression tail this rung admits: any operand read after a call
        // is a value live across it, which is Class B.
        if seg.get(*p) == Some(&0x33) {
            let mut q = *p;
            let k = (eat_byte(seg, &mut q, 0x33) && eat(seg, &mut q, &INT_TYPE))
                .then(|| read_varint(seg, &mut q))
                .flatten()
                .ok_or(Block { ctx: "callseq-tail-lit", byte: None, off: *p, aux: 0 })?;
            eat_return_plumbing(seg, &mut q, true, BODY_SCOPE_DEPTH)
                .map_err(|_| Block { ctx: "callseq-tail-lit", byte: None, off: *p, aux: 0 })?;
            // `li rD,k` carries a signed-16-bit immediate; a wider one is
            // `lis`+`ori` and is not modeled here.
            if !(-0x8000..=0x7FFF).contains(&k) {
                return Err(Block { ctx: "callseq-tail-lit-wide", byte: None, off: *p, aux: 0 });
            }
            *p = q;
            tail = SeqTail::Lit(k);
            break;
        }
        // (3) Another call. Either a statement (`4B`, result discarded) or the
        // value the body returns.
        let tok = eat_call_head(seg, p)?;
        let args = eat_call_args(seg, p)?;
        if eat_byte(seg, p, 0x4B) {
            raw.push((tok, args));
            if raw.len() > MAX_SEQ_CALLS {
                return Err(Block { ctx: "callseq-too-long", byte: None, off: *p, aux: 0 });
            }
            continue;
        }
        // The value call. `41` = the result is returned as is; `33 <int> k 02` =
        // returned plus a literal — the same post-op the single framed call
        // carries, and the same `addi r3,r3,k`.
        let add_k = if seg.get(*p) == Some(&0x41) {
            0
        } else {
            if !eat_byte(seg, p, 0x33) || !eat(seg, p, &INT_TYPE) {
                return Err(blk(seg, *p, "callseq-postop"));
            }
            let k = read_varint(seg, p).ok_or(blk(seg, *p, "callseq-postop-varint"))?;
            if !eat_byte(seg, p, 0x02) {
                // non-ADD post-op → non-commutative / strength-reduced
                return Err(blk(seg, *p, "callseq-postop-op"));
            }
            if !(-0x8000..=0x7FFF).contains(&k) {
                return Err(Block { ctx: "callseq-postop-wide", byte: None, off: *p, aux: 0 });
            }
            k
        };
        eat_return_plumbing(seg, p, true, BODY_SCOPE_DEPTH)?;
        raw.push((tok, args));
        tail = SeqTail::CallValue { add_k };
        break;
    }

    // A single call whose result is discarded and with nothing after it is a
    // TAIL call, not a framed body — but the caller already checked that before
    // entering, so reaching it here would mean the grammar drifted.
    debug_assert!(
        raw.len() > 1 || !matches!(tail, SeqTail::Void),
        "a lone statement call with a void tail is the tail-call shape"
    );

    // Validate and normalize every call's arguments through the ONE locator every
    // other call shape uses, so the marshalling has a single implementation.
    let mut calls: Vec<SeqCall> = Vec::with_capacity(raw.len());
    for (i, (callee_tok, args)) in raw.into_iter().enumerate() {
        let (arg_ops, arg_sources) =
            match tail_call_shape(args, params.clone(), callee_tok, *p)? {
                BodyShape::VoidTailCall { .. } => (Vec::new(), None),
                BodyShape::IntTailCall { arg_ops, .. } => (arg_ops, None),
                BodyShape::MultiArgTailCall { arg_sources, .. } => (Vec::new(), Some(arg_sources)),
                // `tail_call_shape` returns exactly those three.
                _ => return Err(Block { ctx: "callseq-arg-shape", byte: None, off: *p, aux: 0 }),
            };
        // **The Class A boundary.** Reading a formal after the first call means
        // that value had to survive a `bl`, which c2 answers with a callee-saved
        // register: `void f(int a,int b){ g1(a); g2(b); }` is
        // `std r31,-16(r1) … mr r31,r4 … mr r3,r31 …` — one saved GPR, a 5-word
        // prologue and an 11-word epilogue. That is Class B and it is a later
        // rung; refuse it by name rather than emit the Class A frame.
        if i > 0
            && (arg_sources.is_some()
                || arg_ops.iter().any(|o| matches!(o, IlOp::Load(_))))
        {
            return Err(Block {
                ctx: "callseq-value-live-across-call",
                byte: None,
                off: *p,
                aux: 0,
            });
        }
        calls.push(SeqCall { callee_tok, arg_ops, arg_sources });
    }
    Ok(BodyShape::CallSeq { params, calls, tail })
}

/// A bound on the statement calls one body may carry, so a corrupt stream cannot
/// make the parser build an unbounded list. Far above anything measured (the
/// widest probe is four) and far below anything a real body reaches before some
/// other production refuses it.
const MAX_SEQ_CALLS: usize = 64;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::func::body::{parse_segment, parse_segment_detail};
    use crate::func::bundle::LO_MARKER;
    use crate::func::readers::find_subslice;
    use crate::func::sy::{Formals, SyView};
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

    /// A two-argument tail call that passes formals 0 and 2 of three must **refuse**,
    /// and above all must not take the process down.
    ///
    /// The permutation analysis sizes its `seen[]` by the argument count and indexes
    /// it with a *formal* index, so `int f(int a,int b,int c){ return g(a,c); }`
    /// panicked with `index out of bounds: the len is 2 but the index is 2` — on
    /// mainline, from `c2rs census`, on two lines of ordinary C++. The 878-TU
    /// workload never reached it because those bodies block earlier on their operand
    /// types, which is exactly why nothing caught it: a scan that is green is green
    /// only on the IL it saw.
    #[test]
    fn a_call_argument_from_a_formal_beyond_the_argument_count_refuses_and_does_not_panic() {
        let b = parse_segment_detail(ARG2_OUTER_FORMAL, NO_LOCALS).unwrap_err();
        assert_eq!(b.ctx, "call-arg-outer-formal");
        assert_eq!(b.feature(), "call-arg-outer-formal:eof");
        assert_eq!(parse_segment(ARG2_OUTER_FORMAL, NO_LOCALS), None);
    }

    /// The control for the refusal above: the same shape passing formals 0 and 1 —
    /// a real permutation of the argument slots — stays in class. The guard must
    /// cost nothing that was already accepted.
    #[test]
    fn a_two_argument_tail_call_over_the_leading_formals_is_still_in_class() {
        let mut inner = ARG2_OUTER_FORMAL.to_vec();
        // The `2D` formals list is in REVERSE source order and `parse_formals`
        // un-reverses it, so `E6` is `a` (index 0), `E7` is `b` and `E8` is `c`
        // (index 2) — and the argument stream is reverse source order too, so
        // `g(a,c)` pushes `c` then `a`. Rebinding the FIRST push from `c` to `b`
        // turns it into `g(a,b)`: sources `[0, 1]`, a permutation of the two
        // argument slots.
        let at = inner
            .windows(3)
            .position(|w| w == [0xB9, 0xE8, 0x09])
            .expect("the first argument push");
        inner[at + 1] = 0xE7;
        assert!(
            matches!(
                parse_segment(&inner, NO_LOCALS),
                Some(BodyShape::MultiArgTailCall { .. })
            ),
            "formals 0 and 1 are a permutation and must stay accepted"
        );
    }

    /// The **call-bound-to-a-local** form of both refusals above, which carried
    /// its own copy of the argument validation and was missing a gate at each of
    /// the two points. One locator now ([`tail_call_shape`]); this test is the
    /// pair that separates "the production refuses" from "the leaf order
    /// refuses".
    ///
    /// * `int z = g(b + a); return z;` was a **wrong-bytes emit** — c2
    ///   canonicalizes a commutative argument's leaves, so it emits the same
    ///   `add r3,r3,r4 ; b ?g` as `g(a + b)` and the port emitted `add r3,r4,r3`
    ///   (`c2rs diff`: `Port=Mismatch @ 537`).
    /// * `int z = g2(a, c); return z;` **panicked** `c2rs census`.
    ///
    /// The canonical-order control must stay in class, so the fix costs nothing
    /// that was already accepted.
    #[test]
    fn a_call_bound_to_a_local_gets_the_same_argument_gates_as_the_direct_form() {
        // The destination `z` is an automatic `int` local, which is what makes the
        // production reachable at all (`.sy` membership, not absence from `.gl`).
        let zc: [u32; 1] = [0xE909];
        let zo: [u32; 1] = [0xEB09];
        let view = |l: &'static [u32]| SyView {
            locals: l,
            formals: Formals::AllOneRegisterByConstruction,
        };
        let zc: &'static [u32] = Box::leak(Box::new(zc));
        let zo: &'static [u32] = Box::leak(Box::new(zo));
        // The wrong-bytes half: non-canonical leaves refuse …
        let b = parse_segment_detail(BOUND_ARG_NONCANON, view(zc)).unwrap_err();
        assert_eq!(b.ctx, "call-arg-noncanonical-order");
        // … and the canonical control is still an in-class integer tail call.
        assert!(
            matches!(
                parse_segment(BOUND_ARG_CANON, view(zc)),
                Some(BodyShape::IntTailCall { .. })
            ),
            "`int z = g(a + b); return z;` is byte-exact and must stay in class"
        );
        // The panic half: a formal past the argument count refuses, in the
        // parser, without indexing anything out of bounds.
        let b = parse_segment_detail(BOUND_ARG2_OUTER_FORMAL, view(zo)).unwrap_err();
        assert_eq!(b.ctx, "call-arg-outer-formal");
        assert_eq!(parse_segment(BOUND_ARG2_OUTER_FORMAL, view(zo)), None);
    }

    /// **Class A many-calls**, positive and negative, on segments transcribed from
    /// live captures. The three facts the production turns on are each one
    /// assertion here, because each is a shape c2 lowers *differently* from its
    /// neighbour:
    ///
    /// * a lone statement call is a TAIL call, not a framed body;
    /// * two statement calls are a framed body whose last call is `bl`, not `b`;
    /// * one statement call plus anything after it is already framed.
    #[test]
    fn class_a_many_calls_decode_and_the_lone_statement_call_stays_a_tail_call() {
        // Two statement calls: framed, Class A, nothing saved.
        let Some(BodyShape::CallSeq { calls, tail, params }) =
            parse_segment(SEQ_TWO_VOID, NO_LOCALS)
        else {
            panic!("`g1(a); g2();` is the Class A many-call shape");
        };
        assert_eq!(params, vec![0xE609]);
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].arg_ops, vec![IlOp::Load(0xE609)]);
        assert!(calls[1].arg_ops.is_empty(), "the second call takes no argument");
        assert_eq!(tail, SeqTail::Void);

        // One statement call and a literal return — framed on ONE call.
        let Some(BodyShape::CallSeq { calls, tail, .. }) =
            parse_segment(SEQ_ONE_THEN_LIT, NO_LOCALS)
        else {
            panic!("`g1(a); return 5;` is framed");
        };
        assert_eq!(calls.len(), 1);
        assert_eq!(tail, SeqTail::Lit(5));

        // The last call's value, bare and with the `+k` post-op.
        assert!(matches!(
            parse_segment(SEQ_CALL_VALUE, NO_LOCALS),
            Some(BodyShape::CallSeq { tail: SeqTail::CallValue { add_k: 0 }, .. })
        ));
        assert!(matches!(
            parse_segment(SEQ_CALL_VALUE_PLUSK, NO_LOCALS),
            Some(BodyShape::CallSeq { tail: SeqTail::CallValue { add_k: 1 }, .. })
        ));

        // A lone statement call is a TAIL call. Emitting the Class A frame for it
        // would be a wrong-bytes emit, not a gap: c2 gives it a bare `b ?g1` and
        // no `.pdata` at all.
        assert!(
            matches!(
                parse_segment(SEQ_LONE_STMT_CALL, NO_LOCALS),
                Some(BodyShape::IntTailCall { .. })
            ),
            "a lone statement call is `b ?g1`, a 5-section leaf"
        );

        // The Class A boundary: a formal read after the first call needs r31.
        let b = parse_segment_detail(SEQ_LIVE_ACROSS, NO_LOCALS).unwrap_err();
        assert_eq!(b.ctx, "callseq-value-live-across-call");
        assert_eq!(parse_segment(SEQ_LIVE_ACROSS, NO_LOCALS), None);
    }

    /// W26: `bool` / `unsigned char` as a value class — free inside the class,
    /// and a real `rlwinm` on the way out of it.
    #[test]
    fn bool_value_class_is_free_inside_and_refuses_the_widening() {
        assert_eq!(
            parse_segment(BOOL_LIT, NO_LOCALS),
            Some(BodyShape::StraightLine {
                params: vec![],
                ops: vec![IlOp::Lit(0)],
            })
        );
        assert_eq!(
            parse_segment(BOOL_ID, NO_LOCALS),
            Some(BodyShape::StraightLine {
                params: vec![0xE409],
                ops: vec![IlOp::Load(0xE409)],
            })
        );
        // The conversion OUT of the class is `clrlwi r3,r3,24`, and it arrives as
        // the same `2C … 00` that is free between the two width-4 classes. It must
        // refuse in the PARSER, under a key that names the target.
        assert_eq!(parse_segment(BOOL_WIDEN_NEG, NO_LOCALS), None);
        assert_eq!(
            parse_segment_detail(BOOL_WIDEN_NEG, NO_LOCALS)
                .unwrap_err()
                .feature(),
            "expr-convert-target-8641"
        );
    }

    /// W25: the store leaf, from whole captured segments — both designators, the
    /// widths that pick the opcode, the literal value, and the FP refusal.
    #[test]
    fn store_leaf_decodes_both_designators_and_refuses_a_float_value() {
        assert_eq!(
            parse_segment(STORE_MEMBER, NO_LOCALS),
            Some(BodyShape::StoreLeaf {
                params: vec![0xF909, 0xFA09],
                ops: vec![
                    IlOp::Load(0xF909),
                    IlOp::Load(0xFA09),
                    IlOp::StoreInd { off: 4, width: 4 },
                ],
            })
        );
        // The width comes from the STORED type, not from the designator's pointer
        // tag — the two agree for an `int` member and this is where they part.
        assert_eq!(
            parse_segment(STORE_NARROW, NO_LOCALS),
            Some(BodyShape::StoreLeaf {
                params: vec![0x010A, 0x020A],
                ops: vec![
                    IlOp::Load(0x010A),
                    IlOp::Load(0x020A),
                    IlOp::StoreInd { off: 12, width: 1 },
                ],
            })
        );
        assert_eq!(
            parse_segment(STORE_LIT, NO_LOCALS),
            Some(BodyShape::StoreLeaf {
                params: vec![0x210A],
                ops: vec![
                    IlOp::Load(0x210A),
                    IlOp::Lit(7),
                    IlOp::StoreInd { off: 0, width: 4 },
                ],
            })
        );
        // The intrinsic-2117 designator reaches the same address by a different
        // route and must produce the byte-identical op stream.
        assert_eq!(
            parse_segment(STORE_BASE_MEMBER, NO_LOCALS),
            Some(BodyShape::StoreLeaf {
                params: vec![0x610A, 0x620A],
                ops: vec![
                    IlOp::Load(0x610A),
                    IlOp::Load(0x620A),
                    IlOp::StoreInd { off: 4, width: 4 },
                ],
            })
        );
        // …and the neighbour that emits `stfs f1` must refuse, in the parser.
        assert_eq!(parse_segment(STORE_FLOAT_NEG, NO_LOCALS), None);
        assert_eq!(
            parse_segment_detail(STORE_FLOAT_NEG, NO_LOCALS)
                .unwrap_err()
                .feature(),
            "expr-op-0x27"
        );
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
        // A pointer pointee (`int **`) emits the same `lwz` and is now ACCEPTED
        // — rung 1 of `docs/IL_LOAD_TYPES.md` §6. It used to be refused (and was
        // recorded as such in `il_expr_load_neg.cpp`) purely because the type
        // gate said "4-byte integer"; the instruction was never the reason. The
        // `41` result has to move with the `30` load: the tail requires the two
        // to be in the same value class, so a half-spliced segment refuses.
        assert!(matches!(
            bad(&[(13, 0x43), (17, 0x43)]),
            Some(BodyShape::IndirectLoad { .. })
        ));
        assert_eq!(bad(&[(13, 0x43)]), None, "ptr load, int result: cross-class");
        // …and tag `C6` is refused even with both positions moved. `readers.rs`
        // records that bit 0x40 occurs and no probe produced it, so it is
        // undetermined, not a cv bit.
        assert_eq!(bad(&[(12, 0xC6), (13, 0x43), (16, 0xC6), (17, 0x43)]), None);

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

    // ---- T1: pointer-valued leaves (rung 1 of docs/IL_LOAD_TYPES.md §6) -----
    //
    // Every segment below is a **whole** captured `.ex` function segment from the
    // live 16.00.11886.00 toolchain, `4F 1F` header included — not a suffix. The
    // pre-body region matters here (it is where `this` is bound and where the
    // line-70 mis-emit lived), so trimming to the `LO` marker would leave exactly
    // the region these cases exist to exercise untested.
    //
    // Positives are from `fixtures/cpp/w12_ptr_leaf.cpp`, negatives from
    // `fixtures/cpp/w12_ptr_leaf_neg.cpp`; both are graded byte-exact against
    // real `c2` by `c2rs diff`, so these tests pin the *decode* of segments whose
    // *emission* the differential already judges.

    /// `C* C::self_np() { return this; }` — the identity leaf through a
    /// non-const `this` (`A6 43` base, `2C` strip to `86 43`). Emits a bare `blr`.
    const PTR_IDENT_THIS: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x59, 0x53, 0x53, 0x26, 0xF9, 0x09,
        0xB9, 0x02, 0x0A, 0xA6, 0x43, 0x81, 0x20, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00, 0x46, // this
        0x4C, 0x4F, 0x11, 0x53, // LO SS
        0xB9, 0x02, 0x0A, 0xA6, 0x43, 0x81, 0x20, // LOAD this (const ptr)
        0x2C, 0x86, 0x43, 0x82, 0x20, 0x00, // ptr -> ptr, emits nothing
        0x41, 0x86, 0x43, 0x82, 0x20, // result: C*
        0x3A, 0x03, 0x0A, 0x54, 0x02, 0x29, 0x03, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `S* id_p(S* s) { return s; }` — the identity leaf over a plain formal, with
    /// no `2C` at all (`86 43` throughout).
    const PTR_IDENT_FORMAL: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x6D, 0x53, 0x53, 0x26, 0x2A, 0x0A,
        0x46, 0x2D, 0x29, 0x0A, // formals: s
        0x4C, 0x4F, 0x11, 0x53, 0xB9, 0x29, 0x0A, 0x86, 0x43, 0x98, 0x20, // LOAD s
        0x41, 0x86, 0x43, 0x98, 0x20, // result: S*
        0x3A, 0x2B, 0x0A, 0x54, 0x02, 0x29, 0x2B, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `int* gp_i(H* h) { return h->mpi; }` — the pointer-valued getter:
    /// `30 86 43 F4 08` (an `int*` value) where the accepted class used to demand
    /// a 4-byte integer. Same `lwz r3,0(r3)`.
    const PTR_GETTER: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x61, 0x53, 0x53, 0x26, 0x11, 0x0A,
        0x46, 0x2D, 0x10, 0x0A, // formals: h
        0x4C, 0x4F, 0x11, 0x53, 0xB9, 0x10, 0x0A, 0x86, 0x43, 0x8C, 0x20, // LOAD h
        0x33, 0x86, 0x41, 0x74, 0x00, 0x27, 0x86, 0x43, 0x91, 0x20, // + offsetof == 0
        0x30, 0x86, 0x43, 0xF4, 0x08, // indirect load -> int*
        0x41, 0x86, 0x43, 0xF4, 0x08, // result: int*
        0x3A, 0x12, 0x0A, 0x54, 0x02, 0x29, 0x12, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `int* gc_p(const C* c) { return c->mp; }` — the same getter with the `2C`
    /// strip, and the one that shows where the `const` lands: the **base** is
    /// `86 43` (the pointer is not const, its pointee is) while the **loaded**
    /// type is `A6 43` and the `2C` unqualifies it.
    const PTR_GETTER_CV: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x68, 0x53, 0x53, 0x26, 0x21, 0x0A,
        0x46, 0x2D, 0x20, 0x0A, // formals: c
        0x4C, 0x4F, 0x11, 0x53, 0xB9, 0x20, 0x0A, 0x86, 0x43, 0x86, 0x20, // LOAD c
        0x33, 0x86, 0x41, 0x74, 0x00, 0x27, 0x86, 0x43, 0x96, 0x20,
        0x30, 0xA6, 0x43, 0x95, 0x20, // indirect load -> int* const
        0x2C, 0x86, 0x43, 0xF4, 0x08, 0x00, // strip -> int*
        0x41, 0x86, 0x43, 0xF4, 0x08, 0x3A, 0x22, 0x0A, 0x54, 0x02, 0x29, 0x22, 0x0A, 0x4F, 0x12,
        0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `int* n_addr_of(S* s) { return &s->b; }` — **the** discriminating
    /// negative: the getter's production *minus the `30`*. Emits `addi r3,r3,4`.
    const PTR_ADDR_OF: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x64, 0x53, 0x53, 0x26, 0x17, 0x0A,
        0x46, 0x2D, 0x16, 0x0A, 0x4C, 0x4F, 0x11, 0x53, 0xB9, 0x16, 0x0A, 0x86, 0x43, 0x81, 0x20,
        0x33, 0x86, 0x41, 0x74, 0x04, 0x27, 0x86, 0x43, 0x84, 0x20, // + 4, NO `30`
        0x41, 0x86, 0x43, 0xF4, 0x08, 0x3A, 0x18, 0x0A, 0x54, 0x02, 0x29, 0x18, 0x0A, 0x4F, 0x12,
        0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `int* n_deref2(int*** ppp) { return **ppp; }` — two `30` loads, two `lwz`.
    const PTR_DEREF2: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x66, 0x53, 0x53, 0x26, 0x1A, 0x0A,
        0x46, 0x2D, 0x19, 0x0A, 0x4C, 0x4F, 0x11, 0x53, 0xB9, 0x19, 0x0A, 0x86, 0x43, 0x85, 0x20,
        0x30, 0x86, 0x43, 0x84, 0x20, // load 1
        0x30, 0x86, 0x43, 0xF4, 0x08, // load 2 -> refuse
        0x41, 0x86, 0x43, 0xF4, 0x08, 0x3A, 0x1B, 0x0A, 0x54, 0x02, 0x29, 0x1B, 0x0A, 0x4F, 0x12,
        0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `S* n_mr(int a, S* s) { return s; }` — the identity of the *second*
    /// formal, which is `mr r3,r4` and not free.
    const PTR_IDENT_R4: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x6F, 0x53, 0x53, 0x26, 0x2A, 0x0A,
        0x46, 0x2D, 0x29, 0x0A, 0x2D, 0x28, 0x0A, // formals: s, a  ->  params [a, s]
        0x4C, 0x4F, 0x11, 0x53, 0xB9, 0x29, 0x0A, 0x86, 0x43, 0x81, 0x20, // LOAD s (r4)
        0x41, 0x86, 0x43, 0x81, 0x20, 0x3A, 0x2B, 0x0A, 0x54, 0x02, 0x29, 0x2B, 0x0A, 0x4F, 0x12,
        0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    #[test]
    fn ptr_getter_leaf_decodes_as_the_same_indirect_load_an_int_getter_does() {
        // The whole point of the rung: the BodyShape is unchanged, so
        // `indirect_load_text` (which consumes no type) emits the same `lwz`.
        assert_eq!(
            parse_segment(PTR_GETTER, NO_LOCALS),
            Some(BodyShape::IndirectLoad {
                params: vec![0x100A],
                ops: vec![IlOp::Load(0x100A), IlOp::LoadInd { off: 0 }],
            })
        );
        assert_eq!(
            parse_segment(PTR_GETTER_CV, NO_LOCALS),
            Some(BodyShape::IndirectLoad {
                params: vec![0x200A],
                ops: vec![IlOp::Load(0x200A), IlOp::LoadInd { off: 0 }],
            }),
            "the A6-tagged load plus its 2C strip"
        );
    }

    #[test]
    fn ptr_identity_leaf_binds_this_and_a_formal_alike() {
        // `return this;` — the token comes from the pre-body `B9 … 99 … 00`
        // group, not from the `2D` formals list, and it is r3.
        assert_eq!(
            parse_segment(PTR_IDENT_THIS, NO_LOCALS),
            Some(BodyShape::StraightLine {
                params: vec![0x020A],
                ops: vec![IlOp::Load(0x020A)],
            })
        );
        // `return s;` — same production, token from the formals list.
        assert_eq!(
            parse_segment(PTR_IDENT_FORMAL, NO_LOCALS),
            Some(BodyShape::StraightLine {
                params: vec![0x290A],
                ops: vec![IlOp::Load(0x290A)],
            })
        );
    }

    #[test]
    fn ptr_leaves_refuse_the_shapes_that_cost_an_instruction() {
        assert_eq!(parse_segment(PTR_DEREF2, NO_LOCALS), None, "**ppp is two lwz");
        // …and it is reported by the census rather than silently, so the
        // instrument and the gate agree that it is out of class.
        assert!(parse_segment_detail(PTR_DEREF2, NO_LOCALS).is_err());

        // `return s;` from r4 is `mr r3,r4`, which W18 now emits. It comes out
        // as the identity `StraightLine` — the *same* one-op stream a first-
        // argument identity produces, with the register decided by the token's
        // position in `params` and nowhere else, which is what lets one arm in
        // `select_text` serve both. `w18_reg_move.cpp` grades the bytes.
        assert_eq!(
            parse_segment(PTR_IDENT_R4, NO_LOCALS),
            Some(BodyShape::StraightLine {
                params: vec![0x280A, 0x290A],
                ops: vec![IlOp::Load(0x290A)],
            }),
            "return s (r4) is one mr"
        );
    }

    #[test]
    fn the_offset_add_without_a_load_is_an_address_and_emits_an_addi() {
        // `&s->b` is the getter minus the `30`, and it is the case that decides
        // whether the *identity* recognizer may skip an optional offset add: it
        // may not, because this emits `addi r3,r3,4` where the identity emits
        // nothing. It used to be a pure refusal (`w12_ptr_leaf_neg.cpp`'s
        // `n_addr_of`); it is now its own production with its own lowering, and
        // the discrimination that mattered is unchanged — it must NOT come out
        // as a `StraightLine` identity.
        assert_eq!(
            parse_segment(PTR_ADDR_OF, NO_LOCALS),
            Some(BodyShape::AddrLeaf {
                params: vec![0x160A],
                ops: vec![IlOp::Load(0x160A), IlOp::AddrOf { off: 4 }],
            }),
            "&s->b is an address leaf at offset 4, not an identity"
        );
        // The neighbour one token along in the other direction: with the `30`
        // back it is a LOAD, and the two must not be interchangeable.
        assert!(
            matches!(parse_segment(PTR_GETTER, NO_LOCALS), Some(BodyShape::IndirectLoad { .. })),
            "a `30` in front of the `41` is still a load"
        );
    }

    #[test]
    fn an_address_leaf_refuses_what_it_cannot_emit_in_one_addi() {
        // Every case below is `PTR_ADDR_OF` with ONE field changed, so each
        // isolates a single gate. The shared prefix ends at the offset literal
        // `04` at index 76 and the `27` type that follows it.
        let base = PTR_ADDR_OF.to_vec();
        assert_eq!(base[79], 0x04, "the offset literal moved");
        assert_eq!(&base[80..85], &[0x27, 0x86, 0x43, 0x84, 0x20], "the `27` add moved");

        // A zero offset emits NOTHING, which is only correct because the address
        // is already in r3 — and here the base IS the first formal, so it is
        // accepted and its op stream records the zero.
        let mut zero = base.clone();
        zero[79] = 0x00;
        assert_eq!(
            parse_segment(&zero, NO_LOCALS),
            Some(BodyShape::AddrLeaf {
                params: vec![0x160A],
                ops: vec![IlOp::Load(0x160A), IlOp::AddrOf { off: 0 }],
            })
        );

        // The `27` re-type must be a POINTER. An int-typed add here would be
        // integer arithmetic on a pointer, which c2 scales.
        let mut nonptr = base.clone();
        nonptr[81] = 0x86;
        nonptr[82] = 0x41;
        nonptr[83] = 0x74;
        assert_eq!(parse_segment(&nonptr, NO_LOCALS), None, "a non-pointer `27`");

        // The `41` result must be a pointer too: an int result means the address
        // was converted, and that conversion is unprobed.
        assert_eq!(&base[85..90], &[0x41, 0x86, 0x43, 0xF4, 0x08], "the `41` moved");
        let mut intres = base.clone();
        intres.splice(85..90, [0x41, 0x86, 0x41, 0x74].iter().copied());
        assert_eq!(parse_segment(&intres, NO_LOCALS), None, "an int result type");
    }

    #[test]
    fn the_any_pointee_pointer_gate_is_a_literal_whitelist() {
        // The address path admits every pointee width where the load path picks
        // its instruction from exactly that field — because `addi` is the same
        // word for all of them (MEASURED, `work/bma/probes/p2.cpp`). The tag is
        // still a whitelist: `0x80 | cv | width`, and the four tags with bit
        // `0x40` set are refused for the reason [`is_ptr4_kind`] gives.
        for tag in [0x82u8, 0x84, 0x86, 0x88, 0x92, 0x94, 0x96, 0x98, 0xA2, 0xA4, 0xA6, 0xA8,
                    0xB2, 0xB4, 0xB6, 0xB8]
        {
            assert!(is_ptr_any(tag, 0x43), "tag {tag:#02X}");
        }
        for tag in [0xC2u8, 0xC6, 0xD6, 0xE6, 0xF6, 0x80, 0x81, 0x8A, 0x7F] {
            assert!(!is_ptr_any(tag, 0x43), "tag {tag:#02X} is undetermined");
        }
        // Kind `0x44` — a function/code pointer — is refused here even though
        // [`is_ptr4_kind`] admits it as a loaded *value*: no probe produced one
        // at an address position, and "the pointee width does not matter" has
        // not been checked for code.
        for kind in [0x44u8, 0x41, 0x42, 0x45, 0x46, 0x47, 0x33, 0x53, 0x83] {
            assert!(!is_ptr_any(0x86, kind), "kind {kind:#02X}");
        }
    }

    #[test]
    fn the_ptr4_type_gate_is_a_literal_whitelist_on_both_bytes() {
        // Tags: `0x80 | cv | width-4`, with cv ⊆ {const 0x20, volatile 0x10}.
        for tag in [0x86u8, 0x96, 0xA6, 0xB6] {
            assert!(is_ptr4_kind(tag, 0x43), "tag {tag:#02X} data pointer");
            assert!(is_ptr4_kind(tag, 0x44), "tag {tag:#02X} function pointer");
        }
        // `0xC6` — bit 0x40 — is reported by `readers.rs` as occurring and was
        // produced by none of the `IL_LOAD_TYPES.md` probes. A field that never
        // varied across the probes is indistinguishable from a constant, so it
        // is required literally and refuses. Same for `0xD6`/`0xE6`/`0xF6`.
        for tag in [0xC6u8, 0xD6, 0xE6, 0xF6] {
            assert!(!is_ptr4_kind(tag, 0x43), "tag {tag:#02X} is undetermined");
        }
        // Other widths are other instructions: an 8-byte pointer does not exist
        // on this target and a 1/2-byte one is the `27` pointee-width spelling,
        // which is a different question ([`is_ptr_to_4`]).
        for tag in [0x82u8, 0x84, 0x88, 0xA2, 0xA8] {
            assert!(!is_ptr4_kind(tag, 0x43), "tag {tag:#02X} is not a 4-byte value");
        }
        // Kinds: only 0x43/0x44. Aggregates (class 6), reals (5), void (7) and
        // the integers are all excluded here — the integers have their own
        // predicate, and the rest are T2/T3 and later rungs.
        for kind in [0x41u8, 0x42, 0x45, 0x46, 0x47, 0x33, 0x53, 0x83, 0x84] {
            assert!(!is_ptr4_kind(0x86, kind), "kind {kind:#02X}");
        }
        // The two classes the leaf tail accepts are disjoint, which is what lets
        // `2C` and `41` be required to agree with the `30`.
        assert_eq!(value_class(0x86, 0x43), Some(ValueClass::Ptr4));
        assert_eq!(value_class(0x86, 0x41), Some(ValueClass::Int4));
        assert_eq!(value_class(0x86, 0x45), None, "float is not in either class");
    }

    #[test]
    fn a_cross_class_2c_over_a_pointer_load_refuses() {
        // `30 <ptr>` followed by `2C <int> 00` is a pointer→int reinterpret. It
        // may well be free, but it is unprobed, and the class-agreement rule is
        // what keeps an address-*adjusting* conversion from ever being admitted
        // as a free one. Splice the int target into the accepted cv-strip
        // getter, changing nothing else.
        let lo = find_subslice(PTR_GETTER_CV, &LO_MARKER).unwrap();
        let at = PTR_GETTER_CV[lo..]
            .windows(6)
            .position(|w| w == [0x2C, 0x86, 0x43, 0xF4, 0x08, 0x00])
            .expect("the 2C strip")
            + lo;
        let mut s = PTR_GETTER_CV[..at].to_vec();
        s.extend_from_slice(&[0x2C, 0x86, 0x41, 0x74, 0x00]); // -> int
        s.extend_from_slice(&PTR_GETTER_CV[at + 6..]);
        assert_eq!(parse_segment(&s, NO_LOCALS), None);
        // Control: the same splice back to the captured pointer target parses,
        // so the assertion above is about the class and not about the splice.
        let mut ok = PTR_GETTER_CV[..at].to_vec();
        ok.extend_from_slice(&[0x2C, 0x86, 0x43, 0xF4, 0x08, 0x00]);
        ok.extend_from_slice(&PTR_GETTER_CV[at + 6..]);
        assert!(parse_segment(&ok, NO_LOCALS).is_some());
    }

    // ---- the generated empty destructor (D1) --------------------------------

    /// Splice a replacement for the first occurrence of `find` in `DTOR_DELEGATE`,
    /// leaving every other byte alone. Every negative below is one such edit, so
    /// each asserts about exactly one field.
    fn dtor_with(find: &[u8], repl: &[u8]) -> Vec<u8> {
        let at = DTOR_DELEGATE
            .windows(find.len())
            .position(|w| w == find)
            .expect("the field being edited");
        let mut v = DTOR_DELEGATE[..at].to_vec();
        v.extend_from_slice(repl);
        v.extend_from_slice(&DTOR_DELEGATE[at + find.len()..]);
        v
    }

    #[test]
    fn the_generated_empty_destructor_parses_under_both_trailer_flags() {
        // The same source captured twice, at the workload's flags and at the
        // fixtures'. The two differ only in the trailers' `0x10` bit and the
        // reference emits the same four bytes for both.
        for (seg, label) in [(DTOR_DELEGATE, "/O1 /Oi /EHsc"), (DTOR_DELEGATE_NOEH, "/Ox")] {
            assert_eq!(
                parse_segment(seg, NO_LOCALS),
                Some(BodyShape::EmptyDtorDelegation {
                    callee_tok: 0xE409,
                    this_tok: 0xFC09,
                    adjust: 0,
                    sub_object: DtorSubObject::Base
                }),
                "{label}"
            );
        }
    }

    // ---- the generated empty destructor, MEMBER form ------------------------

    /// Splice a replacement for the first occurrence of `find` in one of the member
    /// segments, leaving every other byte alone.
    fn mem_dtor_with(seg: &[u8], find: &[u8], repl: &[u8]) -> Vec<u8> {
        let at = seg
            .windows(find.len())
            .position(|w| w == find)
            .expect("the field being edited");
        let mut v = seg[..at].to_vec();
        v.extend_from_slice(repl);
        v.extend_from_slice(&seg[at + find.len()..]);
        v
    }

    #[test]
    fn the_member_destructor_parses_at_both_offsets() {
        // The two productions differ by exactly one literal, and that literal is
        // the whole codegen difference: nothing at 0, one `addi r3,r3,4` at 4.
        // MEASURED, `work/rf/probes/p3.cpp`:
        //   ??1HasMem@@QAA@XZ:   b ??1MemA@@QAA@XZ
        //   ??1HasMem4@@QAA@XZ:  addi r3,r3,4 ; b ??1MemA@@QAA@XZ
        assert_eq!(
            parse_segment(DTOR_MEMBER_OFF0, NO_LOCALS),
            Some(BodyShape::EmptyDtorDelegation {
                callee_tok: 0xE409,
                this_tok: 0x090A,
                adjust: 0,
                sub_object: DtorSubObject::Member
            }),
            "member at offset 0"
        );
        assert_eq!(
            parse_segment(DTOR_MEMBER_OFF4, NO_LOCALS),
            Some(BodyShape::EmptyDtorDelegation {
                callee_tok: 0xE409,
                this_tok: 0x0C0A,
                adjust: 4,
                sub_object: DtorSubObject::Member
            }),
            "member at offset 4"
        );
    }

    #[test]
    fn the_member_offset_must_fit_one_addi() {
        // MEASURED at the boundary (`work/rf/probes/k32764.cpp` / `k32768.cpp`,
        // `char pad[k]` before the member): 32,764 emits `addi r3,r3,32764` and
        // 32,768 emits **two** instructions, `addis r3,r3,1 ; addi r3,r3,-32768`.
        // The gate is therefore at the signed-16-bit edge and not at a round number,
        // and the escape spelling of the literal (`80` + 4 LE bytes) is what carries
        // a value that wide.
        let lit = |k: i32| {
            let b = k.to_le_bytes();
            vec![0x33, 0x86, 0x41, 0x74, 0x80, b[0], b[1], b[2], b[3], 0x27]
        };
        let find = [0x33, 0x86, 0x41, 0x74, 0x04, 0x27];
        for k in [8i32, 32_764, 32_767] {
            let seg = mem_dtor_with(DTOR_MEMBER_OFF4, &find, &lit(k));
            assert!(
                matches!(
                    parse_segment(&seg, NO_LOCALS),
                    Some(BodyShape::EmptyDtorDelegation { adjust, .. }) if adjust == k
                ),
                "offset {k} fits one addi"
            );
        }
        for k in [32_768i32, 65_536, -4] {
            let seg = mem_dtor_with(DTOR_MEMBER_OFF4, &find, &lit(k));
            assert_eq!(parse_segment(&seg, NO_LOCALS), None, "offset {k} does not");
        }
    }

    #[test]
    fn two_destroyed_members_in_one_body_refuse() {
        // The gate that matters most for the member form. MEASURED,
        // `work/rf/probes/q1.cpp` (`struct Two { ~Two(); MemA m; MemB n; };`): two
        // statements, each with its own leading `33 <int> 0` literal, `5E 02 31`,
        // and the reference emits a FRAME — `or r31,r3,r3`, two `bl`s in reverse
        // declaration order, `or r3,r31,r31` between them — because `this` is live
        // across the first call. Admitting it as one branch would be a wrong-bytes
        // emit, so both `5E 01` and reaching the segment end must refuse it.
        assert_eq!(
            parse_segment(
                &mem_dtor_with(DTOR_MEMBER_OFF0, &[0x5E, 0x01, 0x31], &[0x5E, 0x02, 0x31]),
                NO_LOCALS
            ),
            None,
            "two destroyed sub-objects"
        );
        assert_eq!(
            parse_segment(
                &mem_dtor_with(DTOR_MEMBER_OFF0, &[0x3A, 0x0A, 0x0A], &[0x26, 0xE4, 0x09]),
                NO_LOCALS
            ),
            None,
            "a second statement where the plumbing must begin"
        );
    }

    #[test]
    fn the_member_receiver_must_be_this_and_must_be_an_offset_add() {
        // The lowering puts the address in r3 with at most an `addi`, which is only
        // right because the base of the add is the incoming `this`.
        let mut seg = DTOR_MEMBER_OFF0.to_vec();
        let at = find_subslice(&seg, &LO_MARKER).unwrap();
        let recv = seg[at..]
            .windows(7)
            .position(|w| w == [0xB9, 0x09, 0x0A, 0xA6, 0x43, 0x81, 0x20])
            .expect("the object pointer")
            + at;
        seg[recv + 1] = 0xF7; // a token no `2D` entry and no `this` group names
        assert_eq!(parse_segment(&seg, NO_LOCALS), None, "a receiver that is not this");
        // `28 00 00` is the other byte-offset add. D2's classifier accepts either,
        // but this production has no `28` witness, so it fails closed rather than
        // being admitted on the assumption that the two spell the same thing.
        assert_eq!(
            parse_segment(
                &mem_dtor_with(
                    DTOR_MEMBER_OFF0,
                    &[0x27, 0xA6, 0x43, 0x8A, 0x20],
                    &[0x28, 0x00, 0x00]
                ),
                NO_LOCALS
            ),
            None,
            "the untyped `28` offset add"
        );
    }

    #[test]
    fn the_member_form_keeps_the_base_forms_gates() {
        // The two receivers are alternatives in one shape, so a gate loosened for
        // one must not leak into the other. The base form's adjust offset stays
        // pinned at 0 (`a_nonzero_base_adjust_refuses`), and the member form's
        // leading literal stays pinned at 0 here — it is the byte that tells this
        // production apart from the 2117 `base-member-addr` designator, which opens
        // on the same `33` and carries the selector as its payload.
        let mut seg = DTOR_MEMBER_OFF0.to_vec();
        let lo = find_subslice(&seg, &LO_MARKER).unwrap();
        seg[lo + 7] = 0x01; // the leading LIT's varint
        assert_eq!(parse_segment(&seg, NO_LOCALS), None);
    }

    #[test]
    fn the_trailer_flags_must_agree_and_a_third_value_refuses() {
        // The two flags co-vary across every witness. A mixed pair is not a
        // capture this port has ever seen, so it fails closed rather than being
        // read as "the bit does not matter".
        assert_eq!(
            parse_segment(&dtor_with(&[0x5E, 0x01, 0x21], &[0x5E, 0x01, 0x31]), NO_LOCALS),
            None,
            "EH bit clear in 5C, set in 5E"
        );
        // And an unmeasured flag value refuses outright.
        assert_eq!(
            parse_segment(&dtor_with(&[0x5C, 0x86, 0x41, 0x74, 0x01], &[0x5C, 0x86, 0x41, 0x74, 0x21]), NO_LOCALS),
            None,
            "an unmeasured statement-trailer flag"
        );
    }

    #[test]
    fn two_destroyed_subobjects_refuse() {
        // `5E <n> …` counts destroyed sub-objects, MEASURED: a two-base
        // destructor emits `5E 02 21` and two calls, the second at a nonzero
        // adjust. Requiring `01` is the gate that keeps this lowering — one bare
        // branch — away from that shape. This is the one payload field whose
        // variation is understood, so it is the one that must be pinned.
        assert_eq!(
            parse_segment(&dtor_with(&[0x5E, 0x01, 0x21], &[0x5E, 0x02, 0x21]), NO_LOCALS),
            None
        );
    }

    #[test]
    fn a_nonzero_base_adjust_refuses() {
        // A base at a nonzero offset costs a real `addi r3,r3,k` before the
        // branch. The adjust literal is the second `33 86 41 74 00` in the body;
        // the first is the leading literal, so edit through the `55` that follows.
        let seg = dtor_with(
            &[0x33, 0x86, 0x41, 0x74, 0x00, 0x55, 0x86, 0x41, 0x74, 0xB9],
            &[0x33, 0x86, 0x41, 0x74, 0x04, 0x55, 0x86, 0x41, 0x74, 0xB9],
        );
        assert_eq!(parse_segment(&seg, NO_LOCALS), None);
    }

    #[test]
    fn a_different_layout_intrinsic_refuses() {
        // 2113 is the UNguarded adjust. 2114 (`base-upcast`) is null-guarded and
        // lowers to five instructions with a control-flow split; the whole family
        // differs, so the selector is required exactly.
        let seg = dtor_with(
            &[0x80, 0x41, 0x08, 0x00, 0x00],
            &[0x80, 0x42, 0x08, 0x00, 0x00],
        );
        assert_eq!(parse_segment(&seg, NO_LOCALS), None);
    }

    #[test]
    fn a_receiver_that_is_not_this_refuses() {
        // The lowering is a bare branch precisely because `this` is already in r3.
        // Rebind the intrinsic's object-pointer argument to a token the pre-body
        // region does not bind, leaving the `this` group itself intact.
        let at = DTOR_DELEGATE
            .windows(12)
            .position(|w| w == [0xB9, 0xFC, 0x09, 0xA6, 0x43, 0x81, 0x20, 0x55, 0xA6, 0x43, 0x81, 0x20])
            .expect("the object-pointer argument");
        let mut seg = DTOR_DELEGATE.to_vec();
        seg[at + 1] = 0xF7; // a token no `2D` entry and no `this` group names
        assert_eq!(parse_segment(&seg, NO_LOCALS), None);
    }

    #[test]
    fn the_leading_literal_must_be_zero_and_int_typed() {
        // Its role is UNKNOWN, so a different value is a body this grammar has no
        // witness for. (The 2117 `base-member-addr` designator is anchored on the
        // same `33` and is told apart by exactly this payload.)
        let mut seg = DTOR_DELEGATE.to_vec();
        let lo = find_subslice(&seg, &LO_MARKER).unwrap();
        seg[lo + 7] = 0x01; // the leading LIT's varint
        assert_eq!(parse_segment(&seg, NO_LOCALS), None);
    }

    #[test]
    fn a_second_statement_and_a_short_segment_both_refuse() {
        // A destructor with a real statement, or with a destructible member, has a
        // second `26` where the return plumbing must begin — and really does emit
        // two branches and a frame.
        assert_eq!(
            parse_segment(&dtor_with(&[0x3A, 0xFD, 0x09], &[0x26, 0xE4, 0x09]), NO_LOCALS),
            None,
            "a second statement"
        );
        // And the parse must reach the segment end, which is the fail-closed
        // terminal every accepted shape shares.
        let cut = DTOR_DELEGATE.len() - 7; // drop the `47 54 01 54 00` fn tail
        assert_eq!(parse_segment(&DTOR_DELEGATE[..cut], NO_LOCALS), None);
    }

}
