//! **WCH — the chained member call as a whole body**: `p->a()->b()` and
//! `return p->a()->b();`, where each call's result is the next call's receiver.
//! **WCL** added an argument on a later link — `p->a()->b(k)` — which is the
//! same recognizer and a different frame class.
//!
//! ```text
//!   26 <m_outer> … 26 <m_inner>   the method symbols, stacked LIFO
//!   B9 <recv> <TYPE ptr4> [2C…]   the innermost receiver     — `eat_receiver_this`
//!   99 <TYPE ptr4> 00             …bound as its argument zero
//!   BD <ret ptr4> 00 <id> (<arg> 55 <T>)* 4C    the innermost call
//!   ( 99 <TYPE ptr4> 00           the chain link: bind the RESULT as `this`
//!     BD <ret> 00 <id> (<arg> 55 <T>)* 4C )+   …and call the next method out
//!     4B | 41 <T>                 statement end, or the result is returned
//! ```
//!
//! ## The row, and why it is Class A
//!
//! `expr-call-in-expr-chained-whole` is **12,479 functions on the 878-TU
//! workload — the largest `-whole` row on the board** — and it is **100 %
//! `calls-2plus` and 100 % `cflow-straight`**. WCB's caution applies exactly
//! here: `calls-2plus` is not a frame class. Read off the reference obj
//! (`work/WCH/probe/p1.cpp`, `/O1 /GS- /c`):
//!
//! ```text
//!   int  c_ret (O* p) { return p->Next()->gi(); }            36 B, F = 96
//!     mflr r12 ; stw r12,-8(r1) ; stwu r1,-96(r1)
//!     bl ?Next ; bl ?gi
//!     addi r1,r1,96 ; lwz r12,-8(r1) ; mtlr r12 ; blr
//!   void c_void(O* p) { p->Next()->vv(); }                   36 B — the same
//!   int  c3    (O* p) { return p->Self()->Next()->gi(); }     40 B — one more `bl`
//!   int  c4    (O* p) { return p->Self()->Self()->Next()->gi(); }  44 B
//! ```
//!
//! **Nothing is saved.** Each call's result goes straight into r3, which is
//! where the next call's `this` belongs, so no value is ever live across a `bl`.
//! That is [`BodyShape::CallSeq`] with `saved` empty — Class A — and
//! `c2_core::codegen::call_seq_text` with N empty setups and an empty tail, which
//! has shipped since #35 rung 1. **This rung adds no codegen at all**; the whole
//! of it is the recognizer.
//!
//! Chain **depth is free** for the same reason: `call_seq_text` takes a setup per
//! call and the probe's 3- and 4-link rows are the 2-link one with more `bl`s.
//!
//! ## Arguments — WCL: the innermost link and every later one, and they differ
//!
//! The innermost call marshals out of the argument registers with nothing
//! clobbered yet, so `this` appends to its argument list as slot 0 and the whole
//! thing goes through [`tail_call_shape`] — the identical trick
//! [`super::mcall_tail`] plays, and the identical permutation:
//!
//! ```text
//!   int c_ai3(O* p,int j,int k) { return p->NextB(k, j)->gi(); }   48 B
//!     mr r11,r5 ; mr r5,r4 ; mr r4,r11 ; bl ?NextB ; bl ?gi
//! ```
//!
//! A **later** link is a different lowering in both of its cells, and both are
//! measured (`work/WCL/probe/p1.cpp`, `/O1 /GS- /c`):
//!
//! ```text
//!   int c_ao(O* p,int k) { return p->Next()->gia(k); }        52 B — CLASS B
//!     … std r31,-16(r1) ; stwu ; mr r31,r4 ; bl ?Next ; mr r4,r31 ; bl ?gia
//!   int c_al(O* p)      { return p->Next()->gia(7); }         40 B — CLASS A
//!     … bl ?Next ; li r4,7 ; bl ?gia
//! ```
//!
//! **Three facts, none of which follows from the others.**
//!
//! 1. **The argument goes to r4.** Slot 0 is the receiver the previous `bl` just
//!    left in r3, so the explicit arguments start at slot 1
//!    (`crate::LINK_FIRST_SLOT`). Putting them at slot 0 instead is 156 of the
//!    sweep fragment's 183 cases wrong.
//! 2. **The formal cell is Class B and the literal cell is not.** A formal is
//!    live across the previous `bl` and takes a callee-saved GPR;
//!    [`plan_saved_gprs`] computes that correctly and always did — but only once
//!    it is shown the link's arguments, which is one line and the difference
//!    between a `std r31` prologue and a wrong one. A literal costs no register,
//!    so an all-literal link keeps WCH's three-word prologue.
//! 3. **The emission order is ASCENDING, which is the opposite of every other
//!    call's in the port.** That one is `c2_core::codegen::calls`'s, where the
//!    free-function captures sit beside these; reusing the shipped
//!    `moves_descending` here is 72 of 183 wrong.
//!
//! The argument list itself is read by [`link_arg_slots`], not by
//! [`tail_call_shape`]: the two disagree about the slot base, about whether a
//! permutation is possible, about a repeated argument, and about the emission
//! order, so sharing the locator would be sharing a name with a rule.
//!
//! ## What else is refused, each with its own census key
//!
//! * an innermost receiver that is not a plain `B9 <formal>` load — a chain over
//!   a global (`gO.a()->b()`), a dereference or a sub-object is a different
//!   *designator* with its own lowering, and each already has its own census name
//!   inside `expr-call-in-expr-chained-*`. Declined non-committally, so those
//!   bodies keep the key that names them;
//! * a receiver that is not one of this function's own formals
//!   (`call-arg-nonformal`, the shared key) — `this` is `params[0]`, so a member
//!   function's implicit receiver passes here exactly like a declared one;
//! * more than eight formals (`callseq-over-eight-formals`, the shared key);
//! * a post-op on the result, a comparison, a conversion or a second statement —
//!   anything that is not the `4B` / `41` end. Those are the `-then-…` siblings
//!   and they keep their own keys;
//! * a **computed** link argument (`mcall-chain-link-arg-computed`), one that is
//!   not a formal (`-nonformal`), a literal wider than the `addi` immediate
//!   (`-lit-wide`), and a slot list that reaches past r10 (`-overflow`) — the
//!   four gates [`link_arg_slots`] declares, each with a case in
//!   `fixtures/cpp/wcl_chain_link_arg_neg.cpp`;
//! * three live formals, which is the `__savegprlr_29` helper class
//!   (`callseq-three-plus-saved`, the shared key), and a permuted or computed
//!   INNERMOST call beside a save (`callseq-saved-with-first-call-setup`).

use crate::func::body::expr::{eat_return_plumbing, eat_scopes};
use crate::func::body::{Block, BodyShape, SeqCall, SeqTail};
use crate::func::readers::eat_byte;
use crate::func::IlOp;

use super::calls::{
    eat_call_args, eat_call_token, eat_callee_push, link_arg_slots, plan_saved_gprs,
    tail_call_shape, MAX_REGISTER_FORMALS,
};
use super::mcall_tail::{eat_receiver_this, eat_this_bind};
use super::params::parse_params;

/// A bound on the links one chain may carry, so a corrupt stream cannot make the
/// parser build an unbounded list. The deepest chain in the D2 sample is four
/// (`mcall::MAX_CHAIN`, the completeness walk's own bound, which every function
/// in this row has already passed) and this is the acceptance side of it.
const MAX_CHAIN_LINKS: usize = 8;

/// Try the chained member call at `at`, which is the **second** `26` of the
/// LIFO method run — [`super::mcall_tail::try_parse_member_tail_call`] has
/// already consumed the first as `outer_callee`.
///
/// `Err(None)` means **not this production**: the cursor is untouched, no census
/// key moves, and the caller falls through to the assignment parse. That matters
/// more here than anywhere else in the family, because the two-symbol head run
/// this entry point keys on is *also* what an ordinary assignment to a member
/// call looks like (`x = p->m();` opens `26 <x> 26 <m> B9 <p> …`,
/// `mcall::PROBE_ONE_LINK_ASSIGN`). Those decline at the chain link — where a
/// `32` store stands rather than a `99` bind — and reach the assignment parser
/// exactly as before.
///
/// `Err(Some(b))` means this IS the production and it parsed to the end of the
/// segment, but a codegen-class gate refuses it; those get their own keys, per
/// `docs/GAPS.md` §6's *give a new gate a key on the way in*.
pub(crate) fn try_parse_member_chain_call(
    seg: &[u8],
    at: usize,
    lo: usize,
    depth: usize,
    outer_callee: u32,
) -> Result<BodyShape, Option<Block>> {
    let mut p = at;
    // **The rest of the method run, outermost first.** The pushes stack LIFO
    // (`mcall`'s module header), so the LAST one is the innermost method — the one
    // the receiver designator belongs to and the one c2 calls first.
    //
    // The run ends at the receiver. A `B9` load ends it by not being a `26`; a
    // named-object receiver (`gO.a()->b()`) is *itself* a `26` push and would be
    // swallowed here — which is harmless, because `eat_receiver_this` then finds no
    // `B9` and the whole production declines non-committally. That row keeps its
    // own census key rather than being claimed and refused under this one.
    let mut methods = vec![outer_callee];
    while seg.get(p) == Some(&0x26) {
        methods.push(eat_callee_push(seg, &mut p).map_err(|_| None)?);
        if methods.len() > MAX_CHAIN_LINKS {
            return Err(None);
        }
    }
    // The caller only enters here on a second `26`, so this holds by construction.
    debug_assert!(methods.len() >= 2, "a chain has two or more stacked methods");

    // The innermost receiver and its `99` bind, through the SAME locator the
    // one-link member call uses — which is where the `volatile` gate and the
    // optional pointer conversion live, and nowhere else.
    let recv_tok = eat_receiver_this(seg, &mut p).map_err(|_| None)?;
    let mut ret = eat_call_token(seg, &mut p).map_err(|_| None)?;
    let mut inner_args = eat_call_args(seg, &mut p).map_err(|_| None)?;
    // `this` is argument slot 0 and the argument list is in **stream** order —
    // rightmost source argument first — so the receiver goes on the END of it.
    // The same fact, and the same one-line consequence, as
    // `mcall_tail::member_tail_call_puts_this_in_slot_zero`.
    inner_args.push(vec![IlOp::Load(recv_tok)]);

    // **The chain links**, outward: each binds the value the previous call left on
    // the operand stack — which is r3, already where `this` belongs — and calls the
    // next method out. `methods` is outermost-first, so they are consumed in
    // reverse.
    //
    // Each link's argument region is kept **per link** rather than flattened. It
    // was flattened while every link was required to be nullary, and one list of
    // "the arguments of the links" is not a thing: slot 1 of the third link and
    // slot 1 of the fourth are different registers at different `bl`s.
    let mut raw_links: Vec<Vec<Vec<IlOp>>> = Vec::with_capacity(methods.len() - 1);
    for _ in 0..methods.len() - 1 {
        eat_this_bind(seg, &mut p).map_err(|_| None)?;
        ret = eat_call_token(seg, &mut p).map_err(|_| None)?;
        raw_links.push(eat_call_args(seg, &mut p).map_err(|_| None)?);
    }

    // The body must END here, exactly as the one-link form requires: the result is
    // discarded (`4B`, a void body) or it is the returned value (`41 <TYPE>`).
    // Anything else is a `-then-…` sibling with a construct of its own after the
    // chain, and it keeps the census key that names that construct.
    let mut depth = depth;
    let tail = if eat_byte(seg, &mut p, 0x4B) {
        // A discarded `float`/`double` result still obliges the TU to carry
        // `_fltused`, which the port has no model of — asked through the shared
        // [`super::calls::CallRet`], so this position cannot drift from the others.
        ret.discarded(p).map_err(Some)?;
        // A brace scope closes **between** the statement end and the return branch
        // — same reasoning, and same call site, as the one-link form's.
        eat_scopes(seg, &mut p, &mut depth).map_err(|_| None)?;
        eat_return_plumbing(seg, &mut p, false, depth).map_err(|_| None)?;
        SeqTail::Void
    } else if seg.get(p) == Some(&0x41) {
        eat_return_plumbing(seg, &mut p, true, depth).map_err(|_| None)?;
        // The last call's value IS the result, with no post-op — the same tail
        // `int f(){ g1(); return g2(); }` produces, which is no instruction at all.
        SeqTail::CallValue { add_k: 0 }
    } else {
        return Err(None);
    };

    // ---- from here the body parses to the end of the segment, so every refusal
    // is a codegen-class one over a COMPLETE body and gets its own key. ----

    let params = parse_params(seg, lo).map_err(Some)?;
    // Past the eighth formal a parameter is stack-homed and its setup is
    // `lwz r3,<slot>(r1)`, not a register move. The refusal is on the whole formals
    // LIST because that is the predicate `select_text` raises — the same reasoning
    // and the same key as the Class A statement sequence this body becomes.
    if params.len() > MAX_REGISTER_FORMALS {
        return Err(Some(Block { ctx: "callseq-over-eight-formals", byte: None, off: p, aux: 0 }));
    }
    // The innermost call's arguments — the receiver plus any explicit ones —
    // validated and normalized through the ONE locator every other call shape
    // uses, exactly as `parse_call_sequence` does it. `call-arg-nonformal`,
    // the permutation-cycle bound and the computed-argument rules all arrive with
    // it rather than being restated here.
    let (arg_ops, arg_sources) =
        match tail_call_shape(inner_args, params.clone(), methods[methods.len() - 1], p)
            .map_err(Some)?
        {
            BodyShape::VoidTailCall { .. } => (Vec::new(), None),
            BodyShape::IntTailCall { arg_ops, .. } => (arg_ops, None),
            BodyShape::MultiArgTailCall { arg_sources, .. } => (Vec::new(), Some(arg_sources)),
            // `tail_call_shape` returns exactly those three.
            _ => return Err(Some(Block { ctx: "callseq-arg-shape", byte: None, off: p, aux: 0 })),
        };
    let mut calls = Vec::with_capacity(methods.len());
    // The innermost call's list is COMPLETE — `this` is its slot 0 — so it is not
    // a link and goes through `tail_call_shape` above like every other call.
    calls.push(SeqCall {
        callee_tok: methods[methods.len() - 1],
        arg_ops,
        arg_sources,
        link_args: None,
    });
    // **WCL** — every later call's `this` arrives in r3 as the previous call's
    // result, so slot 0 needs no instruction and the explicit arguments start at
    // slot 1 ([`LINK_FIRST_SLOT`]). `raw_links` is in the same outward order this
    // loop walks, and a nullary link resolves to an empty list, which is the
    // WCH row: setup EMPTY, nothing saved, Class A.
    for (m, args) in methods[..methods.len() - 1].iter().rev().zip(raw_links) {
        calls.push(SeqCall {
            callee_tok: *m,
            arg_ops: Vec::new(),
            arg_sources: None,
            link_args: Some(link_arg_slots(args, &params, p).map_err(Some)?),
        });
    }
    // Class A by construction — no later call reads a formal, so this returns
    // empty. Asked anyway, through the one locator, rather than asserting it: the
    // rule is the same one the statement sequence runs, and a private restatement
    // of "this is Class A" is exactly the shape of drift `GAPS.md` §6 records.
    let saved = plan_saved_gprs(&params, &calls, 0, p).map_err(Some)?;
    Ok(BodyShape::CallSeq { params, calls, tail, saved })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::func::body::{parse_segment, parse_segment_detail, BodyShape, SlotArg};
    use crate::func::test_fixtures::NO_LOCALS;

    /// `int p_ret(O* p) { return p->Next()->gi(); }` — the row's witness,
    /// transcribed verbatim from a live-toolchain capture of
    /// `fixtures/cpp/wch_chained_call.cpp` (`c2rs census … --keep-il`) rather than
    /// hand-assembled. The head is `26 <gi> 26 <Next> B9 <p> …`: the OUTERMOST
    /// method is pushed first and the innermost last, which is the one fact a
    /// hand-written segment would get backwards.
    const MC_CHAIN_RET: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x1E, 0x53, 0x53, 0x26, 0x02, 0x0A,
        0x46, 0x2D, 0x01, 0x0A, 0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, 0x26, 0xF3, 0x09, 0xB9,
        0x01, 0x0A, 0x86, 0x43, 0x81, 0x20, 0x99, 0x86, 0x43, 0x86, 0x20, 0x00, 0xBD, 0x86, 0x43,
        0x83, 0x20, 0x00, 0x80, 0x06, 0x10, 0x00, 0x00, 0x4C, 0x99, 0x86, 0x43, 0x90, 0x20, 0x00,
        0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x10, 0x10, 0x00, 0x00, 0x4C, 0x41, 0x86, 0x41, 0x74,
        0x3A, 0x03, 0x0A, 0x54, 0x02, 0x29, 0x03, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `int p_three(O* p) { return p->Self()->Next()->gi(); }` — the same
    /// production one link deeper, and the row that separates "reverse the push
    /// run" from "swap the two". Its head is `26 <gi> 26 <Next> 26 <Self>`, so
    /// the emission order `Self, Next, gi` is the push run read backwards; a
    /// two-element swap gives `Next, Self, gi` and is byte-wrong here while being
    /// byte-right in [`MC_CHAIN_RET`]. Verbatim capture, same TU.
    const MC_CHAIN_THREE: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x2C, 0x53, 0x53, 0x26, 0x0B, 0x0A,
        0x46, 0x2D, 0x0A, 0x0A, 0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, 0x26, 0xF3, 0x09, 0x26,
        0xF9, 0x09, 0xB9, 0x0A, 0x0A, 0x86, 0x43, 0x81, 0x20, 0x99, 0x86, 0x43, 0x8B, 0x20, 0x00,
        0xBD, 0x86, 0x43, 0x81, 0x20, 0x00, 0x80, 0x0B, 0x10, 0x00, 0x00, 0x4C, 0x99, 0x86, 0x43,
        0x86, 0x20, 0x00, 0xBD, 0x86, 0x43, 0x83, 0x20, 0x00, 0x80, 0x06, 0x10, 0x00, 0x00, 0x4C,
        0x99, 0x86, 0x43, 0x90, 0x20, 0x00, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x10, 0x10, 0x00,
        0x00, 0x4C, 0x41, 0x86, 0x41, 0x74, 0x3A, 0x0C, 0x0A, 0x54, 0x02, 0x29, 0x0C, 0x0A, 0x4F,
        0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    #[test]
    fn a_two_link_chain_is_a_class_a_call_sequence_innermost_first() {
        let Some(BodyShape::CallSeq { params, calls, tail, saved }) =
            parse_segment(MC_CHAIN_RET, NO_LOCALS)
        else {
            panic!("`return p->Next()->gi();` is the chained member call");
        };
        assert_eq!(params, vec![0x010A], "the one formal, `p`");
        assert_eq!(calls.len(), 2);
        // **The emission order is the reverse of the push order.** `26 <gi>` is
        // first in the stream and `?gi` is called SECOND; a recognizer that took
        // the pushes in order emits both `bl`s to the wrong callees, and the obj
        // differs only in two REL24 symbol indices.
        assert_eq!(calls[0].callee_tok, 0xF309, "?Next — pushed last, called first");
        assert_eq!(calls[1].callee_tok, 0xE409, "?gi — pushed first, called last");
        assert_eq!(calls[0].arg_ops, vec![IlOp::Load(0x010A)], "`this` is slot 0");
        assert!(calls[1].arg_ops.is_empty(), "the link's `this` is already in r3");
        assert_eq!(tail, SeqTail::CallValue { add_k: 0 });
        // **Class A**: nothing is live across either `bl`. This is the assertion
        // the whole rung rests on — `calls-2plus` says the opposite and is not a
        // frame class (WCB).
        assert_eq!(saved, Vec::<usize>::new());
    }

    #[test]
    fn the_emission_order_is_the_push_run_reversed_not_a_swap() {
        let Some(BodyShape::CallSeq { calls, saved, .. }) =
            parse_segment(MC_CHAIN_THREE, NO_LOCALS)
        else {
            panic!("`return p->Self()->Next()->gi();` is a three-link chain");
        };
        // Pushes are `gi, Next, Self`; the calls are `Self, Next, gi`.
        assert_eq!(
            calls.iter().map(|c| c.callee_tok).collect::<Vec<_>>(),
            vec![0xF909, 0xF309, 0xE409]
        );
        assert!(calls[1].arg_ops.is_empty() && calls[2].arg_ops.is_empty());
        // Depth costs one `bl` and nothing else — still Class A at three links.
        assert_eq!(saved, Vec::<usize>::new());
    }

    /// The offset of the **chain link's** `99` bind — the second one in the
    /// segment, the one with no `B9` designator in front of it.
    fn link_bind(seg: &[u8]) -> usize {
        let first = seg
            .windows(6)
            .position(|w| w[0] == 0x99 && w[1] == 0x86 && w[2] == 0x43 && w[5] == 0x00)
            .expect("the receiver bind");
        first
            + 1
            + seg[first + 1..]
                .windows(6)
                .position(|w| w[0] == 0x99 && w[1] == 0x86 && w[2] == 0x43 && w[5] == 0x00)
                .expect("the chain link bind")
    }

    #[test]
    fn the_chain_link_binds_through_the_one_shared_locator() {
        // The link's bind is `eat_this_bind`, not a private copy — so it carries
        // the same three gates the receiver's does. Retyping the bound value to
        // `int` (`86 41 74`) says the previous call returned a non-pointer, which
        // cannot be a receiver, and the body refuses.
        let at = link_bind(MC_CHAIN_RET);
        let mut seg = MC_CHAIN_RET.to_vec();
        seg.splice(at + 1..at + 5, [0x86, 0x41, 0x74]);
        assert!(parse_segment(&seg, NO_LOCALS).is_none());
        // …and the trailing UNKNOWN field is required to be `00` here too. Both
        // mutations decline **non-committally** — the body is not yet known to be
        // complete at the bind, so the production hands back a body that keeps its
        // de-conflated `expr-call-in-expr-…` key rather than claiming it under a
        // gate of this rung's own. That is the same contract
        // `mcall_tail::eat_receiver_this` has one link in.
        let mut seg = MC_CHAIN_RET.to_vec();
        seg[at + 5] = 0x04;
        assert!(parse_segment(&seg, NO_LOCALS).is_none());
        let f = parse_segment_detail(&seg, NO_LOCALS).unwrap_err().feature();
        assert!(f.starts_with("expr-call-in-expr"), "{f}");
    }

    /// `int p_ao(O* p, int k) { return p->Next()->gia(k); }` — **the row**, the
    /// one-formal-argument link, transcribed verbatim from a live-toolchain
    /// capture (`work/WCL/probe/il1.cpp`, `c2rs census --keep-il`). Its link
    /// region is `99 <bind> BD <head> B9 <k> 55 <int> 4C`, where WCH's is a bare
    /// `4C`.
    const MC_LINK_ARG: &[u8] = &[
    0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
    0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
    0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
    0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x02, 0x53, 0x53, 0x26, 0xF2, 0x09,
    0x46, 0x2D, 0xF1, 0x09, 0x2D, 0xF0, 0x09, 0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE6, 0x09, 0x26,
    0xE4, 0x09, 0xB9, 0xF0, 0x09, 0x86, 0x43, 0x81, 0x20, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00,
    0xBD, 0x86, 0x43, 0x81, 0x20, 0x00, 0x80, 0x04, 0x10, 0x00, 0x00, 0x4C, 0x99, 0x86, 0x43,
    0x86, 0x20, 0x00, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x06, 0x10, 0x00, 0x00, 0xB9, 0xF1,
    0x09, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74, 0x4C, 0x41, 0x86, 0x41, 0x74, 0x3A, 0xF3,
    0x09, 0x54, 0x02, 0x29, 0xF3, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `int p_jk(O* p, int j, int k) { return p->Next()->gia2(j, k); }` — two
    /// formal arguments on the link. With [`MC_LINK_TWO_SWAPPED`] this is the
    /// pair that grades the **stream order** of a link's argument region: the two
    /// differ in nothing but which formal each slot wants, and reading the region
    /// forwards instead of reversed swaps the two `mr` sources in exactly one of
    /// them. Verbatim capture, same TU.
    const MC_LINK_TWO: &[u8] = &[
    0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
    0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
    0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
    0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x03, 0x53, 0x53, 0x26, 0xF7, 0x09,
    0x46, 0x2D, 0xF6, 0x09, 0x2D, 0xF5, 0x09, 0x2D, 0xF4, 0x09, 0x4C, 0x4F, 0x11, 0x53, 0x26,
    0xE9, 0x09, 0x26, 0xE4, 0x09, 0xB9, 0xF4, 0x09, 0x86, 0x43, 0x81, 0x20, 0x99, 0x86, 0x43,
    0x84, 0x20, 0x00, 0xBD, 0x86, 0x43, 0x81, 0x20, 0x00, 0x80, 0x04, 0x10, 0x00, 0x00, 0x4C,
    0x99, 0x86, 0x43, 0x88, 0x20, 0x00, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x08, 0x10, 0x00,
    0x00, 0xB9, 0xF6, 0x09, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74, 0xB9, 0xF5, 0x09, 0x86,
    0x41, 0x74, 0x55, 0x86, 0x41, 0x74, 0x4C, 0x41, 0x86, 0x41, 0x74, 0x3A, 0xF8, 0x09, 0x54,
    0x02, 0x29, 0xF8, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `int p_kj(O* p, int j, int k) { return p->Next()->gia2(k, j); }` — the
    /// same call with the two arguments transposed at the source. Verbatim.
    const MC_LINK_TWO_SWAPPED: &[u8] = &[
    0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
    0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
    0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
    0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x04, 0x53, 0x53, 0x26, 0xFC, 0x09,
    0x46, 0x2D, 0xFB, 0x09, 0x2D, 0xFA, 0x09, 0x2D, 0xF9, 0x09, 0x4C, 0x4F, 0x11, 0x53, 0x26,
    0xE9, 0x09, 0x26, 0xE4, 0x09, 0xB9, 0xF9, 0x09, 0x86, 0x43, 0x81, 0x20, 0x99, 0x86, 0x43,
    0x84, 0x20, 0x00, 0xBD, 0x86, 0x43, 0x81, 0x20, 0x00, 0x80, 0x04, 0x10, 0x00, 0x00, 0x4C,
    0x99, 0x86, 0x43, 0x88, 0x20, 0x00, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x08, 0x10, 0x00,
    0x00, 0xB9, 0xFA, 0x09, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74, 0xB9, 0xFB, 0x09, 0x86,
    0x41, 0x74, 0x55, 0x86, 0x41, 0x74, 0x4C, 0x41, 0x86, 0x41, 0x74, 0x3A, 0xFD, 0x09, 0x54,
    0x02, 0x29, 0xFD, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `int p_al(O* p) { return p->Next()->gia(7); }` — the literal cell, which
    /// stays **Class A**: a constant costs no callee-saved register, so nothing
    /// is saved and the whole body is `bl ?Next ; li r4,7 ; bl ?gia`. Verbatim.
    const MC_LINK_LIT: &[u8] = &[
    0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
    0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
    0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
    0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x05, 0x53, 0x53, 0x26, 0xFF, 0x09,
    0x46, 0x2D, 0xFE, 0x09, 0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE6, 0x09, 0x26, 0xE4, 0x09, 0xB9,
    0xFE, 0x09, 0x86, 0x43, 0x81, 0x20, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00, 0xBD, 0x86, 0x43,
    0x81, 0x20, 0x00, 0x80, 0x04, 0x10, 0x00, 0x00, 0x4C, 0x99, 0x86, 0x43, 0x86, 0x20, 0x00,
    0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x06, 0x10, 0x00, 0x00, 0x33, 0x86, 0x41, 0x74, 0x07,
    0x55, 0x86, 0x41, 0x74, 0x4C, 0x41, 0x86, 0x41, 0x74, 0x3A, 0x00, 0x0A, 0x54, 0x02, 0x29,
    0x00, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01,
    0x06, 0x4D,
    ];

    #[test]
    fn a_formal_argument_on_a_later_link_is_class_b_at_slot_one() {
        let Some(BodyShape::CallSeq { params, calls, saved, tail }) =
            parse_segment(MC_LINK_ARG, NO_LOCALS)
        else {
            panic!("`return p->Next()->gia(k);` is the chained member call");
        };
        assert_eq!(params, vec![0xF009, 0xF109], "`p` then `k`");
        assert_eq!(calls.len(), 2);
        // The innermost call is NOT a link: its argument list is complete, with
        // the receiver as slot 0, and it went through `tail_call_shape`.
        assert_eq!(calls[0].callee_tok, 0xE409, "?Next — pushed last, called first");
        assert_eq!(calls[0].arg_ops, vec![IlOp::Load(0xF009)]);
        assert_eq!(calls[0].link_args, None);
        // The link carries `k` at slot 1 — `mr r4,r31`, not `mr r3,r31`.
        assert_eq!(calls[1].callee_tok, 0xE609, "?gia");
        assert_eq!(calls[1].link_args, Some(vec![SlotArg::Formal(1)]));
        assert!(calls[1].arg_ops.is_empty() && calls[1].arg_sources.is_none());
        assert_eq!(tail, SeqTail::CallValue { add_k: 0 });
        // **Class B, and this is the whole difference from WCH.** `k` is live
        // across the first `bl`, so it takes r31 — `plan_saved_gprs` sees it only
        // because it now reads `link_args`, which is the one line that decides
        // whether this body gets a `std r31` prologue or a wrong one.
        assert_eq!(saved, vec![1]);
    }

    #[test]
    fn a_links_argument_region_is_read_in_stream_order_like_every_other() {
        // `gia2(j, k)` — slot 1 wants `j` (params[1]), slot 2 wants `k`.
        let Some(BodyShape::CallSeq { calls, saved, params, .. }) =
            parse_segment(MC_LINK_TWO, NO_LOCALS)
        else {
            panic!("two arguments on a link is the chained member call");
        };
        assert_eq!(params, vec![0xF409, 0xF509, 0xF609], "`p`, `j`, `k`");
        assert_eq!(
            calls[1].link_args,
            Some(vec![SlotArg::Formal(1), SlotArg::Formal(2)]),
            "the region is `B9 <k> · B9 <j> · 4C`; slot 1 is the LAST of it"
        );
        assert_eq!(saved, vec![1, 2], "j takes r31 and k takes r30");
        // …and the transposed source is the transposed slot list, from a segment
        // that differs only in the two `B9` tokens. Reading the region forwards
        // returns the identity on one of these two and the swap on the other,
        // which is what makes the pair a measurement rather than a restatement.
        let Some(BodyShape::CallSeq { calls, saved, .. }) =
            parse_segment(MC_LINK_TWO_SWAPPED, NO_LOCALS)
        else {
            panic!("the transposed form is the same production");
        };
        assert_eq!(
            calls[1].link_args,
            Some(vec![SlotArg::Formal(2), SlotArg::Formal(1)])
        );
        assert_eq!(saved, vec![1, 2]);
    }

    #[test]
    fn a_literal_link_argument_keeps_the_body_class_a() {
        let Some(BodyShape::CallSeq { calls, saved, .. }) =
            parse_segment(MC_LINK_LIT, NO_LOCALS)
        else {
            panic!("`return p->Next()->gia(7);` is the chained member call");
        };
        assert_eq!(calls[1].link_args, Some(vec![SlotArg::Lit(7)]));
        // A constant needs no register to survive the `bl`, so the frame keeps
        // WCH's three-word prologue and the only new word is `li r4,7`.
        assert_eq!(saved, Vec::<usize>::new());
    }

    #[test]
    fn a_computed_or_nonformal_link_argument_refuses_by_name() {
        // The accepted segment's link argument is `B9 <k> 55 <int>`. Rewriting it
        // to `k + 1` is `addi r4,r31,1` — the operand stream rebased onto the
        // callee-saved register, captured (`work/WCL/probe/p1.cpp`, `c_ac`) and
        // out of class. It must refuse, and refuse with a name.
        let at = MC_LINK_ARG
            .windows(3)
            .rposition(|w| w[0] == 0xB9 && w[1] == 0xF1 && w[2] == 0x09)
            .expect("the link's argument load");
        let end = at
            + MC_LINK_ARG[at..]
                .iter()
                .position(|&b| b == 0x55)
                .expect("the argument terminator");
        let mut seg = MC_LINK_ARG.to_vec();
        seg.splice(end..end, [0x33, 0x86, 0x41, 0x74, 0x01, 0x02]);
        assert_eq!(parse_segment(&seg, NO_LOCALS), None);
        assert_eq!(
            parse_segment_detail(&seg, NO_LOCALS).unwrap_err().ctx,
            "mcall-chain-link-arg-computed"
        );
        // An argument that is not one of this function's formals — here the
        // token is simply not in the `2D` run — cannot be in a callee-saved
        // register, because nothing put it there.
        let mut seg = MC_LINK_ARG.to_vec();
        seg[at + 1] = 0x77;
        assert_eq!(parse_segment(&seg, NO_LOCALS), None);
        assert_eq!(
            parse_segment_detail(&seg, NO_LOCALS).unwrap_err().ctx,
            "mcall-chain-link-arg-nonformal"
        );
    }

    #[test]
    fn a_chain_whose_receiver_is_not_a_load_declines_without_taking_the_key() {
        // `gO.a()->b()` — the innermost receiver is a named object, so the last
        // `26` push IS the receiver and no `B9` follows. The production must
        // decline **non-committally**: the body keeps whichever
        // `expr-call-in-expr-chained-…` key names its own receiver, rather than
        // being claimed and refused under this rung's.
        let at = MC_CHAIN_RET
            .windows(3)
            .position(|w| w[0] == 0xB9 && w[1] == 0x01 && w[2] == 0x0A)
            .expect("the receiver load");
        let mut seg = MC_CHAIN_RET.to_vec();
        seg[at] = 0x26;
        assert_eq!(parse_segment(&seg, NO_LOCALS), None);
        let ctx = parse_segment_detail(&seg, NO_LOCALS).unwrap_err().ctx;
        assert_ne!(ctx, "mcall-chain-link-args");
        assert_ne!(ctx, "callseq-over-eight-formals");
    }
}
