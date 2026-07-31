//! The assignment statement body — `s->m = v;` and its neighbours as a
//! *statement* rather than an expression, before the leaf recognizers see it.

use crate::func::body::chain::{
    canonical_chain_for_codegen, has_repeated_leaf, ChainReject,
    straight_line_out_of_class_ctx, substitute, MAX_SUBST_OPS,
};
use crate::func::body::expr::{eat_return_plumbing, eat_scopes, parse_expr, parse_formals};
use crate::func::body::mcall;
use crate::func::body::{blk, Block, BodyShape};
use crate::func::readers::{eat_byte, eat_int_like, read_token_var};
use crate::func::IlOp;

use super::calls::parse_call_shape;
use super::params::parse_params;

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
/// Both sites that bind one ask, and the refusal is raised LAST, at the `26`
/// that pushed it — see [`dst_not_formal`], which carries what deferring it
/// measured (WAE, `docs/rungs/2026-07-31-assign-eof.md`).
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
    // The `26` push of the FIRST statement whose destination this class will not
    // fold, held until the rest of the body has had its say. See `dst_not_formal`
    // below for why it is deferred rather than refused on the spot.
    let mut deferred: Option<usize> = None;
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
                // The destination test applies HERE TOO, and used to not: this
                // route hands `dst` to the call shape as a bound token without
                // ever asking what `dst` is, so `extern int gv; int f(int a){ gv =
                // g(a); return gv; }` censused in class as an `int-tail-call`
                // with the store to `gv` folded away — the exact defect `.sy` was
                // built to stop, at the one site that did not consult it. It was
                // never a live mis-emit, but only because `IlBundle::functions`
                // refuses any TU whose `.gl` carries an unclaimed data symbol
                // (`fixtures/cpp/il_gl_data_symbol.cpp`), and a function-level
                // class must not be sound only by a translation-unit accounting
                // rule about something else.
                //
                // AFTER the call parse, not before, for the reason the main path
                // defers as well — a check placed before it would relabel every
                // body this production already refuses for its own reasons.
                // MEASURED: 14,454 workload functions reach this branch with a
                // destination the class will not fold, and **0** of them parse;
                // refusing them up front would invent a 14,454-row census bucket
                // naming none of its own contents.
                let shape = parse_call_shape(seg, &mut q, lo, Some(dst))?;
                if !formals.contains(&dst) && !locals.contains(&dst) {
                    return Err(dst_not_formal(seg, p));
                }
                return Ok(shape);
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
        // local whose address escapes — leaves this list empty and refuses, as
        // before. **Recorded, not raised** — see [`dst_not_formal`].
        if !formals.contains(&dst) && !locals.contains(&dst) && deferred.is_none() {
            deferred = Some(start_of_stmt);
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
            .ok_or(Block::refuse(seg, p, "assign-subst-overflow"))?;
        // Re-assigning shadows the previous definition, which is how a dead store
        // disappears: only the last definition can reach the return.
        env.retain(|(t, _)| *t != dst);
        env.push((dst, rhs));
        if env.len() > MAX_SUBST_OPS {
            return Err(Block::refuse(seg, p, "assign-too-many-locals"));
        }
    }
    eat_scopes(seg, &mut p, &mut depth)?;
    let ret = parse_expr(seg, &mut p, 0x41)?;
    let ret = substitute(&ret, &env)
        .ok_or(Block::refuse(seg, p, "assign-subst-overflow"))?;
    eat_return_plumbing(seg, &mut p, true, depth)?;
    let params = parse_params(seg, lo)?;
    // After substitution every remaining LOAD must be a parameter. Anything else
    // is a read of something this class cannot account for — an uninitialized
    // local, a global, or a token from a construct not modeled here.
    if !ret.iter().all(|o| match o {
        IlOp::Load(t) => params.contains(t),
        _ => true,
    }) {
        return Err(Block::refuse(seg, p, "assign-ret-nonformal"));
    }
    // Substitution is a *source* of repeated leaves even when the written source
    // has none: `int x = a; x = x + x;` substitutes to `a + a`, which c2 emits as
    // `slwi r3,r3,1`. This gate is what keeps that from being wrong bytes.
    if has_repeated_leaf(&ret) {
        return Err(Block::refuse(seg, p, "assign-repeated-leaf"));
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
        return Err(Block::refuse(seg, p, ctx));
    }
    // …and the **same canonicalization**, which this producer used to skip.
    //
    // Substitution reorders (`int x = b; return x + a;` resolves to `b + a`) and it
    // also *creates* pending-immediate chains the written source does not show
    // (`int x = a+1; return x+b;` resolves to `[a, 1, Add, b, Add]`). Only the
    // pre-canonicalizer fallback checks ran here, so a stream the straight-line
    // producer canonicalizes into `add r11,r3,r4 ; addi r3,r11,1` reached the
    // selector in source order and was refused — the census counted it and the
    // port declined it. One fact, one locator: the decision is shared, not copied.
    let ret = match canonical_chain_for_codegen(&ret, &params) {
        Ok(c) => c,
        // Both pre-canonicalizer refusals keep this producer's published key.
        Err(ChainReject::Order) | Err(ChainReject::Additive) => {
            return Err(Block::refuse(seg, p, "assign-noncanonical-order"))
        }
        Err(ChainReject::Affine) => return Err(Block::refuse(seg, p, "assign-affine-pending-imm")),
    };
    // …and LAST, the destination. Everything above reports first, so this key names
    // a function only when the destination really is the one thing left.
    if let Some(off) = deferred {
        return Err(dst_not_formal(seg, off));
    }
    Ok(BodyShape::StraightLine { params, ops: ret })
}

/// The destination refusal, raised at the `26` push it is about — **after** the
/// whole body has had its say (WAE).
///
/// # Why it is deferred, and what the immediate version measured
///
/// The gate itself is right and does not move: a destination this class cannot
/// prove register-resident must not have its store folded away. What moved is
/// *when it speaks*. Refused on the spot, it fired on the FIRST assignment
/// statement — before the remaining statements, the returned expression, the
/// return plumbing and the four post-substitution gates had been parsed at all —
/// so it named every body it stopped, whatever that body's real problem was.
///
/// MEASURED on the 878-TU dc3 workload (2,462,571 functions), by deferring exactly
/// as this function now does and reading the redistribution:
///
/// ```text
///   assign-dst-not-formal      13,887  ->  0        census 691,744 unchanged, +0
///     8,221  assign-store-type-0x86     the very next line: a 4-byte non-integer
///     1,906  assign-store-type-0x82     a 1-byte store
///     1,364  expr-jump                  a goto / break / loop exit
///       830  expr-op-0x27
///       789  expr-call-in-expr-recv-load-then-call-op-0x64
///       468  expr-call-in-expr-recv-load-then-call-data-addr-and-deref-load-more
///       232  expr-call-in-expr-recv-intrinsic-this-adjust-then-intrinsic-call
///        77  … 20 further keys
/// ```
///
/// **Not one function in the workload has the destination as its only blocker.**
/// Lifting the gate entirely converts 0; lifting it *and* the store-type check
/// below it converts 0 as well (the 8,221 then land on `expr-jump` / `expr-op-0x60`
/// / `expr-op-0x10`). The row was the 20th-largest on the board and its realizable
/// worth is exactly zero — `docs/rungs/2026-07-31-assign-eof.md` is the write-up.
///
/// This is the same discipline the formals-marker check at the head of
/// [`try_parse_assign_body_detail`] already follows, and for the same stated
/// reason: an outer gate that refuses early makes the census name *it* instead of
/// the innermost unmodeled construct, and the census's histogram IS the widening
/// order.
///
/// # The byte, and the `:eof` that was not one
///
/// The block carries `byte: Some(0x26)` at the destination push. The previous
/// version passed `byte: None` at a mid-segment offset, and
/// [`crate::func::body::Block::feature`] renders any `byte: None` block as
/// `<ctx>:eof` — so the key read `assign-dst-not-formal:eof` on a refusal that had
/// not reached the end of anything. That suffix is load-bearing when it is true
/// (it means the parse ran out of segment, so nothing can be hiding behind the
/// refusal) and this key claimed it falsely: 4,466 of the 13,887 rows were
/// `cflow-loop` bodies. A rung was ranked and scheduled on the strength of it.
/// `0x26` is the opcode the gate is actually about, it is one key rather than a
/// shard per right-hand side, and `hex[hex_mark]` now points at the push.
fn dst_not_formal(seg: &[u8], off: usize) -> Block {
    Block { ctx: "assign-dst-not-formal", byte: Some(0x26), off, seg_len: seg.len(), aux: 0 }
}
