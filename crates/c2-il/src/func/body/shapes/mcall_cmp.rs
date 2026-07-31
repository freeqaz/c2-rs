//! **WCB — `return a->m() == b->n();`**: two member calls in one expression,
//! compared for equality. The first call's result is live across the second
//! `bl`, so the body is **Class B** — 1 or 2 saved GPRs, `std`/`ld` inline.
//!
//! ```text
//!   26 <m1> B9 <r1> <ptr4> [2C…] 99 <ptr4> 00  BD <ret> 00 <id>  4C
//!   26 <m2> B9 <r2> <ptr4> [2C…] 99 <ptr4> 00  BD <ret> 00 <id>  4C
//!   1F                      the `==` operator (`Rel::Eq`)
//!   [ 2C <int4> 00 ]        an `int`-typed result converts; a `bool` one does not
//!   41 <TYPE> …             returned
//! ```
//!
//! ## Why this is a rung and `calls-2plus` did not say so
//!
//! `docs/CMP_PRODUCES_A_VALUE.md` built the comparison half of this row, measured
//! it at **+0**, and reverted — because the row is not `p->m() <rel> k`, it is a
//! **comparator**, and 89.8 % of it has two calls with the first result live
//! across the second. That is this file. The row it unblocks is
//! `expr-call-in-expr-recv-load-then-cmp-eq-and-type-int1-whole2`, **6,001
//! functions on the 878-TU workload, 6,000 of them `calls-2plus`**.
//!
//! ## The three things that are measured here and are not guessable
//!
//! **1. c2 chooses the call order, and it is neither source order nor
//! evaluation order.** The two calls are emitted in the order c1xx **numbered
//! their receivers** — see [`alloc_rank`], which is a parameter-position rule and
//! deliberately not a token-value one. The source's left operand may be either
//! one. Twelve
//! grid cells fix it (`work/WCB/probe/p5.cpp`, every ordered pair of three
//! pointer formals in both source orders, with and without a leading `int`
//! formal), and the refuter that separates "ascending token" from "ascending
//! parameter index" is a **member function**:
//!
//! ```text
//!   bool V::e1(const V* a) const { return m() == a->m(); }
//!     mr r31,r3 ; mr r3,r4 ; bl ?m ; mr r30,r3 ; mr r3,r31 ; bl ?m ; …
//! ```
//!
//! `this` is r3 and parameter index 0, yet `a`'s call goes **first** — because
//! c1xx numbers `this` *after* the declared formals. A model that ordered by
//! parameter index (which is what [`super::super::chain::leaves_ascending`] does,
//! correctly, for its own operands) emits both moves backwards here. So the rank
//! is computed with `this` moved to the end, and the source's left/right roles
//! are carried separately, in [`SeqTail::CmpEq::lhs_first`].
//!
//! **2. The spine is a register-register family the port has never emitted.**
//! `docs/CMP_PRODUCES_A_VALUE.md` reading 4: where the literal comparison is
//! `addi r11,a,-k`, this one is `subf r11,<lhs>,<rhs>` over two registers. The
//! two words after it are the *same* `== 0` fold the comparison leaf already
//! emits, with the same `/O1` temp collapse — so they are imported from
//! `c2_core::codegen::leaf::compare` rather than re-spelled.
//!
//! **3. The `bool` result is NOT a different spine here.** Reading 1 of that
//! document warns that a `bool` result changes the bytes — it does, for signed
//! `>=`/`<=` against a non-zero literal, two of 24 cells. `==` is not one of
//! them: `int c3(…)` and `bool e1(…)` over the same two calls are **byte
//! identical** (`work/WCB/probe/p7.cpp`), differing only in whether the IL
//! carries the `2C <int4> 00` convert. That is why this rung admits both and why
//! it admits **only `==`** — the relations that would walk into those cells are
//! refused by name.
//!
//! ## What is refused, each with its own census key
//!
//! * any relation but `==` (`mcall-cmp-rel`) — `!=` is three more words and
//!   **0** functions on the workload in this shape; `<`/`<=`/`>`/`>=` are the
//!   five-word sign-sum spines and the two `bool` cells above. Measured: 828
//!   functions across `cmp-gt` and `cmp-lt`, against 6,001 for `==`;
//! * either call taking an explicit argument (`mcall-cmp-args`) — the
//!   marshalling would interleave with the callee-saved move, and which of the
//!   two is hoisted is a rule `shapes/calls.rs`'s `plan_saved_gprs` refuses to
//!   guess;
//! * a receiver that is not one of this function's own formals
//!   (`call-arg-nonformal`, the shared key) — a global or a local is a load, not
//!   a register move;
//! * more than eight formals (`framed-arg-over-eight-formals`, the shared key).

use crate::func::body::expr::eat_return_plumbing;
use crate::func::body::{Block, BodyShape, SeqCall, SeqTail};
use crate::func::readers::{eat_byte, eat_int_like, eat_value_type, ValueClass};
use crate::func::{IlOp, Rel};

use super::calls::{
    arg_loads_are_formals, eat_call_args, eat_call_token, eat_callee_push, plan_saved_gprs,
    MAX_REGISTER_FORMALS,
};
use super::mcall_tail::eat_receiver_this;
use super::params::parse_params;
use super::this_binding::{parse_this_token, ThisBinding};

/// The `==` operand-stream opcode. Spelled here as a [`Rel`] round-trip rather
/// than as a bare byte so the one table in [`Rel::from_opcode`] stays the only
/// place a relational byte is named.
fn eat_relation(seg: &[u8], p: &mut usize) -> Option<Rel> {
    let rel = Rel::from_opcode(*seg.get(*p)?)?;
    *p += 1;
    Some(rel)
}

/// Parse the second member call and the `==` that consumes both results.
///
/// Entered from [`super::mcall_tail::try_parse_member_tail_call`] with the first
/// call already decoded and the cursor at the byte after its `4C`. `Err(None)`
/// keeps the non-committal contract: the caller falls through and the body keeps
/// its de-conflated `expr-call-in-expr-recv-…` census key.
pub(crate) fn try_parse_member_cmp_calls(
    seg: &[u8],
    at: usize,
    lo: usize,
    depth: usize,
    first_args: &[Vec<IlOp>],
    callee1: u32,
    recv1: u32,
) -> Result<BodyShape, Option<Block>> {
    let mut p = at;
    // The second member call, read through the SAME locators the tail form uses.
    // A second copy of the receiver decode is the drift `docs/GAPS.md` §6
    // instance #9 records — in particular the `volatile` gate, which lives in
    // `eat_receiver_this`'s operand-type read and nowhere else.
    let callee2 = eat_callee_push(seg, &mut p).map_err(|_| None)?;
    let recv2 = eat_receiver_this(seg, &mut p).map_err(|_| None)?;
    let ret2 = eat_call_token(seg, &mut p).map_err(|_| None)?;
    let args2 = eat_call_args(seg, &mut p).map_err(|_| None)?;

    // The relation. Not this production unless a relational byte stands here —
    // an arithmetic post-op (`02`/`03`) over two calls is a *different* spine
    // (`add r3,r30,r3`, one word) and belongs to whichever rung takes it.
    let Some(rel) = eat_relation(seg, &mut p) else {
        return Err(None);
    };
    // **The result's class, and the annotation that has to restate it.** The
    // comparison produces a `bool`; an `int`/`unsigned`-typed result converts it
    // back with `2C <int4> 00` and annotates `41 <int4>`, a `bool` one carries
    // neither. The *bytes are the same* either way — `work/WCB/probe/p7.cpp`'s
    // `c3` (`int`) and `p3.cpp`'s `a1` (`bool`) are the same 76 bytes — so the
    // class is consumed and discarded rather than carried into the shape.
    //
    // It is still **required to agree**, for the reason
    // `parse_segment_shape`'s own one-byte-unsigned arm gives: a `bool` value
    // annotated `int` is the `rlwinm` mask c2 emits for a widening, and
    // admitting it as a no-op would be wrong bytes rather than a gap. The two
    // arms mirror that site's, including which one calls
    // [`eat_return_plumbing`] with `has_result_type` — the shared `41` gate is
    // `eat_int_like_or_ptr4` and is deliberately not widened to one-byte types.
    let converted = seg.get(p) == Some(&0x2C);
    if converted {
        let mut q = p + 1;
        if !(eat_int_like(seg, &mut q) && eat_byte(seg, &mut q, 0x00)) {
            return Err(None);
        }
        p = q;
    }
    // The body must END here: the comparison's value is what the function
    // returns. Anything else (a branch on it — `-and-branch-more`, 9,490
    // functions of this family) needs basic blocks.
    if converted {
        eat_return_plumbing(seg, &mut p, true, depth).map_err(|_| None)?;
    } else {
        if !(eat_byte(seg, &mut p, 0x41) && eat_value_type(seg, &mut p, ValueClass::Int1u)) {
            return Err(None);
        }
        eat_return_plumbing(seg, &mut p, false, depth).map_err(|_| None)?;
    }

    // ---- from here the body parses to the end of the segment, so every refusal
    // is a codegen-class one over a COMPLETE body and gets its own key
    // (`docs/GAPS.md` §6: give a new gate a key on the way in). ----

    // Only `==`. The other five relations are the sign-sum spines, and two of
    // their 24 cells change bytes with a `bool` result
    // (`docs/CMP_PRODUCES_A_VALUE.md` reading 1) — a spine borrowed from the
    // comparison leaf would be two words short there, with `.pdata FuncLen` and
    // both `$M` values wrong to match. Measured cost on the 878-TU workload:
    // 828 functions across `cmp-gt` and `cmp-lt`, against 6,001 for `==`.
    if rel != Rel::Eq {
        return Err(Some(Block { ctx: "mcall-cmp-rel", byte: None, off: p, aux: 0 }));
    }
    // A `float`/`double` result would oblige the TU to carry `_fltused`, which
    // is the `call-ret-fp` refusal one production out — asked through the shared
    // [`CallRet`] so this position cannot drift from the others.
    ret2.discarded(p).map_err(Some)?;
    // Both calls must be NULLARY. `this` is the only argument either one carries;
    // an explicit argument marshals into r4… beside the callee-saved move, and
    // which of the two is hoisted is exactly what `plan_saved_gprs` refuses to
    // guess (`callseq-saved-with-first-call-setup`, 11 of 17 probes wrong for the
    // model that assumed it).
    if first_args.len() != 1 || !args2.is_empty() {
        return Err(Some(Block { ctx: "mcall-cmp-args", byte: None, off: p, aux: 0 }));
    }

    let params = parse_params(seg, lo).map_err(Some)?;
    // Both receivers must be this function's own formals — the emission is a
    // register move, and a global or a local would be a load. `this` is in the
    // list at index 0 (`params::parse_params`), so a member function's implicit
    // receiver passes here exactly like a declared one.
    let recv_ops = [IlOp::Load(recv1), IlOp::Load(recv2)];
    if !arg_loads_are_formals(&recv_ops, &params) {
        return Err(Some(Block { ctx: "call-arg-nonformal", byte: None, off: p, aux: 0 }));
    }
    // Past the eighth formal a parameter is stack-homed and reading it is
    // `lwz r3,<slot>(r1)`. The refusal is on the whole formals LIST, not on a
    // receiver's index, because that is the predicate `select_text` raises —
    // the same reasoning and the same key as every other framed shape.
    if params.len() > MAX_REGISTER_FORMALS {
        return Err(Some(Block {
            ctx: "framed-arg-over-eight-formals",
            byte: None,
            off: p,
            aux: 0,
        }));
    }

    // **The call order.** c2 emits the two calls in the order c1xx **numbered
    // their receivers**, and that is a fact about the parameter list, not about
    // the source: see [`alloc_rank`] and the module header's grid. A tie (the
    // same receiver twice, `p->m() == p->n()`) keeps the IL order.
    let member = match parse_this_token(seg, lo) {
        Some(ThisBinding::Absent) => false,
        Some(ThisBinding::Bound(_)) => true,
        // `parse_params` above already refused an undetermined binding.
        None => return Err(Some(Block { ctx: "this-undetermined", byte: None, off: p, aux: 0 })),
    };
    let (Some(r1), Some(r2)) = (
        alloc_rank(&params, member, recv1),
        alloc_rank(&params, member, recv2),
    ) else {
        // Unreachable: `arg_loads_are_formals` just proved both are in `params`.
        return Err(Some(Block { ctx: "mcall-cmp-recv-rank", byte: None, off: p, aux: 0 }));
    };
    let lhs_first = r1 <= r2;
    let (c0, c1) = if lhs_first {
        ((callee1, recv1), (callee2, recv2))
    } else {
        ((callee2, recv2), (callee1, recv1))
    };
    let calls = vec![
        SeqCall { callee_tok: c0.0, arg_ops: vec![IlOp::Load(c0.1)], arg_sources: None },
        SeqCall { callee_tok: c1.0, arg_ops: vec![IlOp::Load(c1.1)], arg_sources: None },
    ];
    // The saved FORMALS, through the one locator the Class A/B statement
    // sequence already uses — the second call's receiver is read after the first
    // `bl`, so it is exactly what that rule returns. `extra_saved = 1` is the
    // first call's *result*, which is live across the second `bl` and takes the
    // next descending register; it is passed in so the `MAX_INLINE_SAVED_GPRS`
    // gate is applied to the TOTAL and a body that would need `__savegprlr_29`
    // refuses there rather than mis-emitting a Class C prologue.
    let saved = plan_saved_gprs(&params, &calls, 1, p).map_err(Some)?;
    Ok(BodyShape::CallSeq { params, calls, tail: SeqTail::CmpEq { lhs_first }, saved })
}

/// **Where c1xx numbered this receiver**, as a sort key over `params` — which is
/// what decides which of the two calls c2 emits first.
///
/// For a free function it is the parameter index. For a **member** function it is
/// not: `parse_params` puts `this` at index 0 because that is the register it
/// occupies, and c1xx numbers it **after** every declared formal. Both facts are
/// true and they are different orderings, which is `docs/GAPS.md` §6's recurring
/// shape — one field carrying two facts — with the two facts being "which
/// register" and "which symbol number".
///
/// ```text
///   bool H::q(const U* a) const { return m() == a->m(); }
///     mr r31,r3 ; mr r3,r4 ; bl ?m@U ; mr r30,r3 ; mr r3,r31 ; bl ?m@H
/// ```
///
/// `this` is r3 and `params[0]`, and `a`'s call still goes first.
///
/// **This is deliberately a parameter-position rule and not a token-value one.**
/// The first version of it compared the tokens [`crate::func::readers::read_token_var`]
/// returns, which was wrong twice over. It is not monotone — the two-byte form is
/// little-endian, so consecutive tokens `0x09FF, 0x0A00` come back as `0xFF09,
/// 0x000A` and the order inverts at every low-byte wrap; that was a live
/// `Port=Mismatch @ offset 8`, and the sweep axis added for it reproduces exactly
/// two failing alignments in every 264. And refusing the four-byte form, whose
/// layout has no captured witness, cost **5,971 of this rung's 6,000 functions**:
/// a real translation unit declares tens of thousands of symbols, so its tokens
/// are essentially all wide. The allocation order is knowable without decoding
/// either form.
fn alloc_rank(params: &[u32], member: bool, tok: u32) -> Option<usize> {
    let ix = params.iter().position(|&t| t == tok)?;
    Some(match (member, ix) {
        (false, i) => i,
        // `this`: numbered after every declared formal.
        (true, 0) => params.len(),
        (true, i) => i - 1,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::func::body::{parse_segment, parse_segment_detail, SeqTail};
    use crate::func::test_fixtures::NO_LOCALS;

    /// `bool t1(const U* p, const U* q) { return p->m() == q->n(); }` — the base
    /// shape, transcribed verbatim from a live-toolchain capture
    /// (`c2rs census … --keep-il`) rather than hand-assembled. The two receivers
    /// are formals 0 and 1 and the source's left operand is the one c2 calls
    /// first, so `lhs_first` is true and nothing is reordered.
    const MC_CMP_PLAIN: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x02, 0x53, 0x53, 0x26, 0xEE, 0x09,
        0x46, 0x2D, 0xED, 0x09, 0x2D, 0xEC, 0x09, 0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, 0xB9,
        0xEC, 0x09, 0x86, 0x43, 0x82, 0x20, 0x99, 0x86, 0x43, 0x85, 0x20, 0x00, 0xBD, 0x86, 0x41,
        0x74, 0x00, 0x80, 0x05, 0x10, 0x00, 0x00, 0x4C, 0x26, 0xE5, 0x09, 0xB9, 0xED, 0x09, 0x86,
        0x43, 0x82, 0x20, 0x99, 0x86, 0x43, 0x85, 0x20, 0x00, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80,
        0x05, 0x10, 0x00, 0x00, 0x4C, 0x1F, 0x41, 0x82, 0x12, 0x30, 0x3A, 0xEF, 0x09, 0x54, 0x02,
        0x29, 0xEF, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];
    /// `bool t2(const U* p, const U* q) { return q->n() == p->m(); }` — the SAME
    /// two calls with the source operands swapped. c2 emits `p`'s call first
    /// either way (ascending receiver rank), so this segment must decode to the
    /// same two calls in the same order with `lhs_first` FALSE. That pair is the
    /// whole ordering claim: a recognizer that carried the IL order into the
    /// shape passes the first test and fails this one.
    const MC_CMP_REORDERED: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x03, 0x53, 0x53, 0x26, 0xF2, 0x09,
        0x46, 0x2D, 0xF1, 0x09, 0x2D, 0xF0, 0x09, 0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE5, 0x09, 0xB9,
        0xF1, 0x09, 0x86, 0x43, 0x82, 0x20, 0x99, 0x86, 0x43, 0x85, 0x20, 0x00, 0xBD, 0x86, 0x41,
        0x74, 0x00, 0x80, 0x05, 0x10, 0x00, 0x00, 0x4C, 0x26, 0xE4, 0x09, 0xB9, 0xF0, 0x09, 0x86,
        0x43, 0x82, 0x20, 0x99, 0x86, 0x43, 0x85, 0x20, 0x00, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80,
        0x05, 0x10, 0x00, 0x00, 0x4C, 0x1F, 0x41, 0x82, 0x12, 0x30, 0x3A, 0xF3, 0x09, 0x54, 0x02,
        0x29, 0xF3, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];
    /// `bool H::q(const U* a) const { return m() == a->m(); }` — the refuter.
    /// `this` is parameter index 0 (and r3) but its token `0x0A00` is HIGHER than
    /// `a`'s `0x09FE`, because c1xx numbers the implicit receiver after the
    /// declared formals. So the calls are emitted `a` first, `this` second —
    /// the opposite of what ordering by parameter index gives — and the source's
    /// left operand (`this`) is the second call, `lhs_first` false.
    const MC_CMP_THIS: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x05, 0x53, 0x53, 0x26, 0xF7, 0x09,
        0xB9, 0x00, 0x0A, 0xA6, 0x43, 0x8A, 0x20, 0x99, 0x86, 0x43, 0x8D, 0x20, 0x00, 0x46, 0x2D,
        0xFE, 0x09, 0x4C, 0x4F, 0x11, 0x53, 0x26, 0xF5, 0x09, 0xB9, 0x00, 0x0A, 0xA6, 0x43, 0x8A,
        0x20, 0x99, 0x86, 0x43, 0x8B, 0x20, 0x00, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x0B, 0x10,
        0x00, 0x00, 0x4C, 0x26, 0xE4, 0x09, 0xB9, 0xFE, 0x09, 0x86, 0x43, 0x82, 0x20, 0x99, 0x86,
        0x43, 0x85, 0x20, 0x00, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x05, 0x10, 0x00, 0x00, 0x4C,
        0x1F, 0x41, 0x82, 0x12, 0x30, 0x3A, 0x01, 0x0A, 0x54, 0x02, 0x29, 0x01, 0x0A, 0x4F, 0x12,
        0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x06, 0x4D,
    ];

    /// The base shape decodes to a **Class B** call sequence: two calls in
    /// emission order, one saved formal (the second call's receiver), and the
    /// `CmpEq` tail whose `saved_gprs()` is therefore 2 — the saved formal plus
    /// the first call's result.
    #[test]
    fn two_member_calls_compared_for_equality_are_a_class_b_sequence() {
        let Some(BodyShape::CallSeq { params, calls, tail, saved }) =
            parse_segment(MC_CMP_PLAIN, NO_LOCALS)
        else {
            panic!("`return p->m() == q->n();` is the two-call comparator");
        };
        // The tokens as `read_token_var` returns them (`EC 09` -> 0xEC09).
        assert_eq!(params, vec![0xEC09, 0xED09], "p then q, un-reversed from `2D`");
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].arg_ops, vec![IlOp::Load(0xEC09)], "p's call is first");
        assert_eq!(calls[1].arg_ops, vec![IlOp::Load(0xED09)]);
        assert_ne!(calls[0].callee_tok, calls[1].callee_tok, "?m and ?n");
        assert_eq!(tail, SeqTail::CmpEq { lhs_first: true });
        assert_eq!(saved, vec![1], "q survives the first `bl` and takes r31");
    }

    /// **The ordering claim**, as the pair that separates it from "IL order".
    /// Swapping the source operands changes `lhs_first` and NOTHING else: the
    /// same two calls, in the same order, with the same saved formal. The emitted
    /// difference is the spine's two `subf` operands, four words later.
    #[test]
    fn the_source_operand_order_moves_only_the_spine_not_the_calls() {
        let Some(BodyShape::CallSeq { calls, tail, saved, .. }) =
            parse_segment(MC_CMP_REORDERED, NO_LOCALS)
        else {
            panic!("`return q->n() == p->m();` is the same production");
        };
        // `p` is formal 0 in this segment too (token `F0 09` against q's `F1 09`).
        assert_eq!(calls[0].arg_ops, vec![IlOp::Load(0xF009)], "p's call is STILL first");
        assert_eq!(calls[1].arg_ops, vec![IlOp::Load(0xF109)]);
        assert_eq!(tail, SeqTail::CmpEq { lhs_first: false }, "…and it is now the RHS");
        assert_eq!(saved, vec![1]);
    }

    /// **`this` is the cell that refutes ordering by parameter index.** It has
    /// index 0 and the highest token, so the declared formal's call goes first
    /// and the save is `mr r31,r3` — the hoisted arm, the only one this family
    /// reaches. Captured: `mr r31,r3 ; mr r3,r4 ; bl ?m ; mr r30,r3 ; mr r3,r31 ;
    /// bl ?m`.
    #[test]
    fn this_has_parameter_index_zero_and_the_highest_token_so_its_call_goes_last() {
        let Some(BodyShape::CallSeq { params, calls, tail, saved }) =
            parse_segment(MC_CMP_THIS, NO_LOCALS)
        else {
            panic!("a member function comparing `m()` against `a->m()`");
        };
        // **And these two tokens straddle a low-byte wrap**, which is why this
        // segment is the one kept: `a` is `FE 09` and `this` is `00 0A`, i.e.
        // allocation values 0x09FE and 0x0A00 — consecutive — but
        // `read_token_var` returns 0xFE09 and 0x000A, which order the *other*
        // way. The first version of the rule compared those raw values and was
        // `Port=Mismatch @ offset 8` on exactly this segment; the rank rule does
        // not look at them at all.
        assert_eq!(params, vec![0x000A, 0xFE09], "`this` at index 0, `a` at 1");
        assert_eq!(calls[0].arg_ops, vec![IlOp::Load(0xFE09)], "`a`'s call is first");
        assert_eq!(calls[1].arg_ops, vec![IlOp::Load(0x000A)], "`this`'s call is second");
        assert_eq!(tail, SeqTail::CmpEq { lhs_first: false });
        assert_eq!(saved, vec![0], "`this` is what survives the first `bl`");
    }

    /// Every relation but `==` refuses, **by name and in the parser**, so the
    /// census and the gate cannot disagree about them. Written as a mutation of
    /// the accepted segment so the only thing that varies between the rows is the
    /// one operator byte — the axis a hand-written fixture would have one point
    /// on.
    #[test]
    fn only_equality_is_admitted_and_the_others_refuse_by_name() {
        let at = MC_CMP_PLAIN
            .windows(5)
            .position(|w| w[0] == 0x1F && w[1] == 0x41 && w[2] == 0x82)
            .expect("the `==` operator byte");
        assert!(matches!(
            parse_segment(MC_CMP_PLAIN, NO_LOCALS),
            Some(BodyShape::CallSeq { tail: SeqTail::CmpEq { .. }, .. })
        ));
        for (op, label) in [
            (0x20u8, "!="),
            (0x21, "<="),
            (0x22, "<"),
            (0x23, ">="),
            (0x24, ">"),
        ] {
            let mut seg = MC_CMP_PLAIN.to_vec();
            seg[at] = op;
            assert_eq!(parse_segment(&seg, NO_LOCALS), None, "{label} must refuse");
            assert_eq!(
                parse_segment_detail(&seg, NO_LOCALS).unwrap_err().ctx,
                "mcall-cmp-rel",
                "{label} must refuse by name, in the parser"
            );
        }
    }

    /// **The allocation rank**, as the table the whole rung's call order rests on.
    /// A free function's is its parameter index; a member function's `this` is
    /// `params[0]` and ranks LAST, because c1xx numbers the implicit receiver
    /// after every declared formal.
    #[test]
    fn this_ranks_after_every_declared_formal_although_it_is_parameter_zero() {
        let p3 = [0xA0u32, 0xA1, 0xA2];
        // Free function: rank is the index.
        assert_eq!(alloc_rank(&p3, false, 0xA0), Some(0));
        assert_eq!(alloc_rank(&p3, false, 0xA1), Some(1));
        assert_eq!(alloc_rank(&p3, false, 0xA2), Some(2));
        // Member function: `this` is params[0] and ranks last; the declared
        // formals shift down by one and keep their relative order.
        assert_eq!(alloc_rank(&p3, true, 0xA0), Some(3), "`this` is numbered last");
        assert_eq!(alloc_rank(&p3, true, 0xA1), Some(0));
        assert_eq!(alloc_rank(&p3, true, 0xA2), Some(1));
        // A token that is not a formal has no rank — the call site has already
        // refused it (`call-arg-nonformal`), and this is the second lock.
        assert_eq!(alloc_rank(&p3, true, 0xFF), None);
    }

    /// The **total** saved-GPR count is what the helper-class gate has to see.
    /// `saved` holds one formal; the tail holds the first call's result; the
    /// prologue must `std` two registers. A consumer reading `saved.len()` gets 1
    /// and emits a Class A prologue with a Class B body.
    #[test]
    fn the_tail_s_saved_call_result_counts_toward_the_gpr_total() {
        let Some(BodyShape::CallSeq { saved, tail, .. }) =
            parse_segment(MC_CMP_PLAIN, NO_LOCALS)
        else {
            panic!("the two-call comparator");
        };
        assert_eq!(saved.len(), 1, "one saved FORMAL");
        assert_eq!(tail, SeqTail::CmpEq { lhs_first: true });
        // …plus the first call's result, which is the PUBLIC shape's business:
        // `bundle.rs` maps this body onto `c2_il::CallSeq`, and `saved_gprs()` is
        // the one place the two are summed.
        use crate::func::SeqTail as PubTail;
        let seq = crate::func::CallSeq {
            calls: Vec::new(),
            tail: PubTail::CmpEq { lhs_first: true },
            saved: vec![1],
        };
        assert_eq!(seq.saved_gprs(), 2);
        // Class A/B statement sequences are unchanged: their tails save nothing.
        assert!(!PubTail::Void.saves_a_call_result());
        assert!(!PubTail::CallValue { add_k: 3 }.saves_a_call_result());
        assert!(!PubTail::Lit(3).saves_a_call_result());
    }
}
