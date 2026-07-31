//! **W36 — the member call as a whole body**: `p->m(a…);` and `return p->m(a…);`
//! where `p` is a plain pointer-valued formal.
//!
//! ```text
//!   26 <method>              push the method symbol      the callee
//!   B9 <recv> <TYPE ptr4>    the receiver value          `this`
//!   99 <TYPE ptr4> 00        bind it as argument zero
//!   BD <ret TYPE> 00 <id>    the CALL token
//!   ( <operands> 55 <T> )*   the explicit arguments, rightmost first
//!   4C                       apply
//!   4B | 41 <T>              statement end, or the result is returned
//! ```
//!
//! **The whole production is the existing tail call with one extra argument
//! slot.** `p->m(x)` is `m(p, x)` on this ABI — `this` is argument zero, in r3 —
//! so the emission is a register permutation over the formals plus `b <method>`,
//! which is exactly [`super::calls::tail_call_shape`]'s job and needs **no
//! codegen at all**: the receiver is appended to the argument list as slot 0 and
//! everything downstream (the identity case that emits nothing, the single
//! `mr r3,rN`, the permutation walk with its measured 3-cycle limit, the
//! `.gl` callee resolution, `.pdata`, `/Gy`) is the code that already grades.
//!
//! ## Why this row was invisible
//!
//! `expr-op-0x99` was **280,283 functions, 11.4 % of everything blocked and the
//! largest single key on the board** — and it was never a missing token. The body
//! dispatch consumes a statement-head `26 <method>` as an assignment
//! *destination* (the byte after it is the receiver's `B9`, not a `BD`), the
//! assignment parser hands the rest to `parse_expr`, and `parse_expr` reads the
//! receiver as an ordinary LOAD and stops on the `99` under its generic `expr`
//! fall-through. So the construct was filed as an **opcode** while the identical
//! production one byte different — `x = p->m();` — reached
//! [`crate::func::body::mcall::classify`] and was filed as a member call all
//! along. `GAPS.md` §6's unstable-*attribution* hazard, in the form that costs
//! coverage rather than correctness: the row carried no whole-body-completeness
//! bit at all, so no ranking taken from it could see what was complete behind it.
//!
//! [`crate::func::body::mcall::reanchor_chain`] now repairs the anchor, which
//! de-conflates the row 1:1 into the `expr-call-in-expr-recv-*` family and prints
//! its `-whole` counts. This file takes the largest sub-shape those counts name.
//!
//! ## What is refused, and why each refusal is a *measurement*
//!
//! Everything the [`super::calls`] tail call already refuses (a computed
//! argument in a multi-argument call, a non-formal argument, a duplicated one, a
//! multi-cycle or >3-cycle permutation, a non-cdecl convention, more than eight
//! arguments) is refused here through the **same** locator, under the same census
//! keys — there is no second copy of any of it. On top of that:
//!
//! * a receiver that is not a plain `B9 <tok> <ptr4 TYPE>` — a member
//!   (`p->q.m()`), a dereference, a named object, another call's result, an
//!   adjusted base (`intrinsic 2113`) or a chain. Each is a *different* receiver
//!   production with its own lowering, and the census already names them
//!   (`expr-call-in-expr-recv-field`, `-recv-deref`, `-recv-object`,
//!   `-recv-call`, `-recv-intrinsic-this-adjust`, `-chained`);
//! * a **non-zero `99` bind offset**. `docs/IL_EXPR_LAYER.md` §7 records that
//!   field as UNKNOWN and zero at every observed site, and a field that never
//!   varied is indistinguishable from a constant (`GAPS.md` §6) — so it is
//!   required literally and its exceptions get their own key rather than being
//!   skipped;
//! * a body that does not **end** at this call: a second statement after it is
//!   the Class A statement-call sequence with a member call in it, which is a
//!   further rung and is refused by name here rather than routed into a
//!   production that has never been graded with a receiver argument.

use crate::func::body::expr::{eat_return_plumbing, eat_scopes};
use crate::func::body::{blk, Block, BodyShape};
use crate::func::readers::{
    eat_byte, eat_operand_type, is_ptr4_kind, read_token_var, read_type, read_varint, ValueClass,
};
use crate::func::IlOp;

use super::calls::{
    eat_call_args, eat_call_token, eat_callee_push, tail_call_shape, MAX_REGISTER_FORMALS,
};
use super::params::parse_params;

/// Try the member-call body at `start` (the statement-head `26`).
///
/// `Err(None)` means **not this production** — the cursor is untouched, no census
/// key moves, and the caller falls through to the assignment parse exactly as
/// before. That is the non-committal contract every other `try_parse_*` has.
///
/// `Err(Some(b))` means **this IS the production, and it parsed to the end of the
/// segment**, but a codegen-class gate refuses it. Those refusals are reported
/// under their own keys rather than swallowed, because `GAPS.md` §6 records the
/// rule twice: *give a new gate a key on the way in, not after someone asks what it
/// cost*, and *a gate raised after the whole-body parse succeeds is free to measure,
/// because its refusals are already complete bodies*. Without this the 11,052
/// bodies the grammar measure calls complete and the argument gates refuse would sit
/// invisibly inside `expr-call-in-expr-recv-load-whole` and the rung's own residue
/// would be a rumour.
///
/// `depth` is the lexical depth the statement parse reached, so a braced body
/// (`void f(A* p){ { p->m(); } }`) closes its scopes exactly rather than being read
/// as a shorter one — the same requirement every other shape's plumbing carries.
pub(crate) fn try_parse_member_tail_call(
    seg: &[u8],
    start: usize,
    lo: usize,
    depth: usize,
) -> Result<BodyShape, Option<Block>> {
    let mut p = start;
    let callee_tok = eat_callee_push(seg, &mut p).map_err(|_| None)?;
    let recv_tok = eat_receiver_this(seg, &mut p).map_err(|_| None)?;
    let ret = eat_call_token(seg, &mut p).map_err(|_| None)?;
    let mut args = eat_call_args(seg, &mut p).map_err(|_| None)?;

    // `this` is argument slot 0, and the argument list is in **stream** order —
    // rightmost source argument first, so slot `i` is `args[len-1-i]`. The receiver
    // therefore goes on the END of the list, not the front. Getting this backwards
    // is invisible on a nullary call and on any call whose permutation happens to be
    // symmetric, which is exactly the shape of defect `GAPS.md` §6 keeps recording,
    // so `member_tail_call_puts_this_in_slot_zero` pins it against a capture where
    // the two readings differ.
    args.push(vec![IlOp::Load(recv_tok)]);

    // The body must END here. Either the result is discarded (`4B`) and the return
    // is void, or it is the returned value (`41 <TYPE>`, consumed by the plumbing).
    // Both lower to the same bare tail branch — the callee leaves its result in the
    // register the caller's own return would use — so the two arms differ only in
    // which plumbing they require.
    //
    // A body that does NOT end here (a second statement after the call — the Class A
    // sequence with a member call in it) falls through to the assignment parse and
    // keeps its de-conflated `expr-call-in-expr-recv-load-then-…` key, which already
    // names what else is in the way. Claiming it here would replace a measured
    // second-blocker key with an uninformative one.
    let mut depth = depth;
    if eat_byte(seg, &mut p, 0x4B) {
        // The result is discarded, so a `float`/`double` one still obliges the TU
        // to carry `_fltused` and the port has no model of that — see
        // [`super::calls::CallRet`] and `docs/GAPS.md` §6 instance #14.
        ret.discarded(p).map_err(Some)?;
        // A brace scope closes **between** the statement end and the return
        // branch, not after it: `void f(A* p){ { p->m(); } }` captures
        // `… 4C 4B · 54 03 · 3A <lbl> · 54 02 · 29 <lbl> …`, so the inner close
        // sits on the far side of the `3A` from the outer one and
        // `eat_return_head`'s own run — which starts after the branch — cannot
        // reach it. Consumed here at the statement boundary, exactly as
        // `try_parse_assign_body_detail` consumes it between two statements, and
        // the plumbing is then asked for the depth that is actually left.
        eat_scopes(seg, &mut p, &mut depth).map_err(|_| None)?;
        eat_return_plumbing(seg, &mut p, false, depth).map_err(|_| None)?;
    } else if seg.get(p) == Some(&0x41) {
        eat_return_plumbing(seg, &mut p, true, depth).map_err(|_| None)?;
    } else {
        return Err(None);
    }

    // From here the body is a member call that parses to the end of the segment, so
    // every remaining refusal is a **codegen-class** one over a complete body and is
    // reported under its own key.
    if args.len() > MAX_REGISTER_FORMALS {
        return Err(Some(Block { ctx: "mcall-args-overflow", byte: None, off: p, aux: 0 }));
    }
    let params = parse_params(seg, lo).map_err(Some)?;
    tail_call_shape(args, params, callee_tok, p).map_err(Some)
}

/// `B9 <tok> <TYPE ptr4> · 99 <TYPE ptr4> 00` — the receiver value and its bind as
/// argument zero. Returns the receiver's token.
///
/// The receiver's TYPE goes through [`eat_operand_type`] rather than a local
/// tag/kind test, so this position inherits the **`volatile` gate** with it:
/// `GAPS.md` §6's thirteenth live mis-emit was a `volatile` formal read that c2
/// homes in the frame, and it was pre-existing across seven shapes because each had
/// asked the question itself. One locator.
fn eat_receiver_this(seg: &[u8], p: &mut usize) -> Result<u32, Block> {
    if !eat_byte(seg, p, 0xB9) {
        return Err(blk(seg, *p, "mcall-recv"));
    }
    let (tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, "mcall-recv-tok"))?;
    *p += w;
    // A pointer, positively — not merely "some 4-byte operand". An `int` receiver
    // is not a receiver, and the class is what the `99` binds.
    match eat_operand_type(seg, p) {
        Some(ValueClass::Ptr4) => {}
        _ => return Err(blk(seg, *p, "mcall-recv-type")),
    }
    if !eat_byte(seg, p, 0x99) {
        return Err(blk(seg, *p, "mcall-bind"));
    }
    // The bound value's own TYPE — a pointer to the class the method belongs to.
    // Required to be a width-4 pointer for the same reason the receiver is: this is
    // the token that says the call is a *direct* member dispatch on an ordinary
    // object pointer (virtual dispatch is `67`/`9A`, a different opcode pair).
    match read_type(seg, *p) {
        Some((tag, kind, _, w)) if is_ptr4_kind(tag, kind) => *p += w,
        _ => return Err(blk(seg, *p, "mcall-bind-type")),
    }
    // The trailing field. UNKNOWN (`docs/IL_EXPR_LAYER.md` §7) and `00` at every
    // observed site, including a member function of a class with a base — so it is
    // required literally, and what that costs is a census key rather than an
    // argument.
    let save = *p;
    match read_varint(seg, p) {
        Some(0) => {}
        Some(_) => {
            *p = save;
            return Err(Block { ctx: "mcall-bind-offset", byte: None, off: save, aux: 0 });
        }
        None => return Err(blk(seg, *p, "mcall-bind-tail")),
    }
    Ok(tok)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::func::body::{parse_segment, parse_segment_detail, BodyShape};
    use crate::func::test_fixtures::*;

    /// `void mv_one(Obj *o) { o->set1(); }` — the minimal member call: the receiver
    /// is the only formal, so it is already in r3 and the whole body is `b <set1>`.
    ///
    /// Transcribed verbatim from a live-toolchain capture of
    /// `fixtures/cpp/w36_member_call.cpp` (`c2rs census … --keep-il`), not
    /// hand-assembled — the point of the production is where the receiver sits
    /// relative to the CALL token, and only a capture settles that.
    const MC_NULLARY: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x02, 0x53, 0x53, 0x26, 0xED, 0x09,
        0x46, 0x2D, 0xEC, 0x09, 0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE5, 0x09, 0xB9, 0xEC, 0x09, 0x86,
        0x43, 0x81, 0x20, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80,
        0x04, 0x10, 0x00, 0x00, 0x4C, 0x4B, 0x3A, 0xEE, 0x09, 0x54, 0x02, 0x29, 0xEE, 0x09, 0x4F,
        0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x03, 0x4D,
    ];

    /// `void mv_swap(Obj *o, int a, int b) { o->v2(b, a); }` — the case where
    /// putting `this` at the FRONT of the argument list instead of the end emits a
    /// different permutation. Verbatim capture, same TU discipline as above.
    const MC_SWAP: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x02, 0x53, 0x53, 0x26, 0xF1, 0x09,
        0x46, 0x2D, 0xF0, 0x09, 0x2D, 0xEF, 0x09, 0x2D, 0xEE, 0x09, 0x4C, 0x4F, 0x11, 0x53, 0x26,
        0xE7, 0x09, 0xB9, 0xEE, 0x09, 0x86, 0x43, 0x81, 0x20, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00,
        0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x04, 0x10, 0x00, 0x00, 0xB9, 0xEF, 0x09, 0x86, 0x41,
        0x74, 0x55, 0x86, 0x41, 0x74, 0xB9, 0xF0, 0x09, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74,
        0x4C, 0x4B, 0x3A, 0xF2, 0x09, 0x54, 0x02, 0x29, 0xF2, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01,
        0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x03, 0x4D,
    ];

    /// The offset of the `99` bind's trailing field in a captured segment: the
    /// `99`, its 4-byte TYPE, then the one-byte varint, with the `BD` after it.
    fn bind_tail(seg: &[u8]) -> usize {
        seg.windows(7)
            .position(|w| w[0] == 0x99 && w[1] == 0x86 && w[2] == 0x43 && w[6] == 0xBD)
            .expect("the 99 bind")
            + 5
    }

    /// The offset of the receiver LOAD's TYPE: `B9`, a 2-byte token, then the TYPE.
    fn recv_type(seg: &[u8]) -> usize {
        seg.windows(7)
            .position(|w| w[0] == 0xB9 && w[3] == 0x86 && w[4] == 0x43 && w[6] == 0x20)
            .expect("the receiver load")
            + 3
    }

    #[test]
    fn a_nullary_member_call_is_a_tail_call_whose_argument_is_the_receiver() {
        // One formal, already in r3, so the argument setup is empty and the emission
        // is the bare `b <method>` — the same `IntTailCall` the free-function
        // statement call `void f(int a){ g(a); }` produces.
        assert_eq!(
            parse_segment(MC_NULLARY, NO_LOCALS),
            Some(BodyShape::IntTailCall {
                params: vec![60425],
                arg_ops: vec![IlOp::Load(60425)],
                callee_tok: 58633,
            })
        );
    }

    #[test]
    fn member_tail_call_puts_this_in_slot_zero() {
        // `o->v2(b, a)` is `v2(o, b, a)`: slot 0 is the receiver, slot 1 is `b`
        // (formal 2) and slot 2 is `a` (formal 1) — a 2-cycle over r4/r5 with r3
        // already in place. The argument list is in STREAM order (rightmost source
        // argument first), so the receiver belongs on the END of it; pushing it on
        // the front would give `[1, 2, 0]`, a 3-cycle, and three wrong `mr`s.
        assert_eq!(
            parse_segment(MC_SWAP, NO_LOCALS),
            Some(BodyShape::MultiArgTailCall {
                params: vec![60937, 61193, 61449],
                arg_sources: vec![0, 2, 1],
                callee_tok: 59145,
            })
        );
    }

    #[test]
    fn the_bind_s_trailing_field_is_required_to_be_zero() {
        // UNKNOWN and `00` at every observed site (`IL_EXPR_LAYER.md` §7). A field
        // that never varied is indistinguishable from a constant, so it is required
        // rather than skipped and its exceptions get their own key.
        let at = bind_tail(MC_NULLARY);
        assert_eq!(MC_NULLARY[at], 0x00);
        let mut seg = MC_NULLARY.to_vec();
        seg[at] = 0x04;
        assert!(parse_segment(&seg, NO_LOCALS).is_none());
    }

    #[test]
    fn a_non_pointer_receiver_declines() {
        // `int` where the pointer must be. The body still refuses, and it refuses
        // through the shared operand-type locator rather than a local tag test —
        // which is also how it inherits the `volatile` gate.
        let at = recv_type(MC_NULLARY);
        let mut seg = MC_NULLARY.to_vec();
        seg.splice(at..at + 4, [0x86, 0x41, 0x74]);
        assert!(parse_segment(&seg, NO_LOCALS).is_none());
    }

    #[test]
    fn the_census_still_names_the_member_call_when_the_shape_declines() {
        // Decoding is not accepting, and a declining body must keep the
        // de-conflated `expr-call-in-expr-recv-*` key rather than falling back to
        // the opcode bucket `expr-op-0x99` this rung exists to empty. Declined here
        // by retyping an ARGUMENT to `float` (`86 45 76`), which the integer operand
        // vocabulary cannot spell — the same way the workload's 1,255
        // `recv-load-then-type-real-…` bodies decline.
        let at = MC_SWAP
            .windows(6)
            .position(|w| w[0] == 0xB9 && w[3] == 0x86 && w[4] == 0x41 && w[5] == 0x74)
            .expect("an argument load")
            + 3;
        let mut seg = MC_SWAP.to_vec();
        seg[at + 1] = 0x45;
        seg[at + 2] = 0x76;
        assert!(parse_segment(&seg, NO_LOCALS).is_none());
        let b = parse_segment_detail(&seg, NO_LOCALS).unwrap_err();
        assert!(
            b.feature().starts_with("expr-call-in-expr-recv-load"),
            "{}",
            b.feature()
        );
    }
}
