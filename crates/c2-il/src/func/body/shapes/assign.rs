//! The assignment statement body — `s->m = v;` and its neighbours as a
//! *statement* rather than an expression, before the leaf recognizers see it.

use crate::func::body::chain::{
    additive_chain_canonical, has_repeated_leaf, leaves_ascending,
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
