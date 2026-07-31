//! **WCH — the chained member call as a whole body**: `p->a()->b()` and
//! `return p->a()->b();`, where each call's result is the next call's receiver.
//!
//! ```text
//!   26 <m_outer> … 26 <m_inner>   the method symbols, stacked LIFO
//!   B9 <recv> <TYPE ptr4> [2C…]   the innermost receiver     — `eat_receiver_this`
//!   99 <TYPE ptr4> 00             …bound as its argument zero
//!   BD <ret ptr4> 00 <id> (<arg> 55 <T>)* 4C    the innermost call
//!   ( 99 <TYPE ptr4> 00           the chain link: bind the RESULT as `this`
//!     BD <ret> 00 <id> 4C )+      …and call the next method out
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
//! ## Arguments: the innermost link is free and every later one is not
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
//! A **later** link is a different lowering in both of its cells, and both were
//! measured rather than assumed:
//!
//! ```text
//!   int c_ao(O* p,int k) { return p->Next()->gia(k); }        52 B — CLASS B
//!     … std r31,-16(r1) ; stwu ; mr r31,r4 ; bl ?Next ; mr r4,r31 ; bl ?gia
//!   int c_al(O* p)      { return p->Next()->gia(7); }         40 B — `li r4,7`
//!     … bl ?Next ; li r4,7 ; bl ?gia
//! ```
//!
//! The formal case is Class B *and* needs the save/marshalling interleave
//! `super::calls::plan_saved_gprs` refuses by name. The literal case is Class A
//! but writes **r4**, and `select_text` — the one argument-setup locator, which
//! `call_seq_parts` calls for every setup — computes into r3 and only r3. So both
//! are refused under one key, `mcall-chain-link-args`, with their cost measured
//! rather than argued.
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
//!   (`chained-then-type-ptr-and-op-more`, 15,049) and they keep their own keys.

use crate::func::body::expr::{eat_return_plumbing, eat_scopes};
use crate::func::body::{Block, BodyShape, SeqCall, SeqTail};
use crate::func::readers::eat_byte;
use crate::func::IlOp;

use super::calls::{
    eat_call_args, eat_call_token, eat_callee_push, plan_saved_gprs, tail_call_shape,
    MAX_REGISTER_FORMALS,
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
    let mut link_args: Vec<Vec<IlOp>> = Vec::new();
    for _ in 0..methods.len() - 1 {
        eat_this_bind(seg, &mut p).map_err(|_| None)?;
        ret = eat_call_token(seg, &mut p).map_err(|_| None)?;
        link_args.extend(eat_call_args(seg, &mut p).map_err(|_| None)?);
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

    // An argument on any link but the innermost — **12,092 functions on the
    // 878-TU workload, more than this rung's own row**, so the two cells are
    // reported under two keys and not one. Neither is this emitter (the module
    // header has both objs), and they are not the same distance from it:
    //
    // * `mcall-chain-link-arg-lit` — every such argument is a bare literal. The
    //   body stays **Class A** and the whole gap is one `li r4,k`: a per-call
    //   *first argument slot* in `call_seq_parts`, because slot 0 is the `this`
    //   already in r3 and the explicit arguments start at r4.
    // * `mcall-chain-link-args` — anything else. A formal is live across the
    //   previous `bl`, which is Class B **and** needs the save/marshalling
    //   interleave `plan_saved_gprs` refuses by name.
    //
    // Split on the way in rather than after somebody asks which half the row is
    // (`docs/GAPS.md` §6), because the two repairs have nothing in common.
    if !link_args.is_empty() {
        let ctx = if link_args.iter().all(|a| matches!(a.as_slice(), [IlOp::Lit(_)])) {
            "mcall-chain-link-arg-lit"
        } else {
            "mcall-chain-link-args"
        };
        return Err(Some(Block { ctx, byte: None, off: p, aux: 0 }));
    }
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
    calls.push(SeqCall { callee_tok: methods[methods.len() - 1], arg_ops, arg_sources });
    for &m in methods[..methods.len() - 1].iter().rev() {
        // Every later call's `this` arrives in r3 as the previous call's result,
        // so its setup is EMPTY — which is what makes the body Class A and is the
        // whole reason this row is cheap.
        calls.push(SeqCall { callee_tok: m, arg_ops: Vec::new(), arg_sources: None });
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
    use crate::func::body::{parse_segment, parse_segment_detail, BodyShape};
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

    #[test]
    fn an_argument_on_a_later_link_refuses_by_name_in_the_parser() {
        // The link's argument region is `4C` (empty) in the accepted segment.
        // Splicing one literal argument in front of it — `33 <int> 7 55 <int>` —
        // is `p->Next()->gia(7)`, which c2 emits as `li r4,7` between the two
        // `bl`s: Class A, but into r4, which `select_text` cannot spell. It must
        // refuse, and refuse with a name, so the census cannot claim it.
        let at = link_bind(MC_CHAIN_RET);
        let close = at
            + MC_CHAIN_RET[at..]
                .iter()
                .position(|&b| b == 0x4C)
                .expect("the link's apply");
        let mut seg = MC_CHAIN_RET.to_vec();
        seg.splice(
            close..close,
            [0x33, 0x86, 0x41, 0x74, 0x07, 0x55, 0x86, 0x41, 0x74],
        );
        assert_eq!(parse_segment(&seg, NO_LOCALS), None);
        assert_eq!(
            parse_segment_detail(&seg, NO_LOCALS).unwrap_err().ctx,
            "mcall-chain-link-args"
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
