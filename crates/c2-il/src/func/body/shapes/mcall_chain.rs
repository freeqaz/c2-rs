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
use crate::func::body::{prod_tag, Block, BodyShape, SeqCall, SeqTail};
use crate::func::readers::{
    eat_byte, is_fp_type, is_ptr4_kind, read_type, value_class, ValueClass,
};
use crate::func::IlOp;

use super::calls::{
    eat_call_args, eat_call_token, eat_callee_push, link_arg_slots, plan_saved_gprs,
    seq_call_arg_sources, tail_call_shape, MAX_REGISTER_FORMALS,
};
use super::designator::{eat_offset_adds, sized_ptee};
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
        // The loop condition already matched the `26`, so the only decline here is
        // a method-symbol token the varint reader cannot spell.
        methods.push(
            eat_callee_push(seg, &mut p)
                .map_err(|_| prod_tag("chain-method-symbol-token-unreadable"))?,
        );
        if methods.len() > MAX_CHAIN_LINKS {
            return Err(prod_tag("chain-more-methods-than-the-link-bound"));
        }
    }
    // The caller only enters here on a second `26`, so this holds by construction.
    debug_assert!(methods.len() >= 2, "a chain has two or more stacked methods");

    // The innermost receiver and its `99` bind, through the SAME locator the
    // one-link member call uses — which is where the `volatile` gate and the
    // optional pointer conversion live, and nowhere else.
    let recv_tok = eat_receiver_this(seg, &mut p).map_err(|b| {
        super::mcall_tail::recv_prod_tag(super::mcall_tail::RecvArm::Chain, seg, &b)
    })?;
    let mut ret = eat_call_token(seg, &mut p)
        .map_err(|_| prod_tag("chain-no-cdecl-call-token-after-the-receiver"))?;
    let mut inner_args = eat_call_args(seg, &mut p)
        .map_err(|_| prod_tag("chain-innermost-argument-not-in-the-operand-vocabulary"))?;
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
        // The link that is NOT a link: `x = p->m();` opens with the same two-symbol
        // head run and reaches here with a `32` store where the `99` bind must be.
        // That population is the whole reason this entry point has to stay
        // non-committal, so it is the one tag site whose name has to say which
        // byte disagreed rather than "the chain declined".
        eat_this_bind(seg, &mut p)
            .map_err(|_| prod_tag("chain-link-does-not-bind-the-previous-result"))?;
        ret = eat_call_token(seg, &mut p)
            .map_err(|_| prod_tag("chain-link-has-no-cdecl-call-token"))?;
        raw_links.push(
            eat_call_args(seg, &mut p)
                .map_err(|_| prod_tag("chain-link-argument-not-in-the-operand-vocabulary"))?,
        );
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
        ret.discarded(seg, p).map_err(Some)?;
        // A brace scope closes **between** the statement end and the return branch
        // — same reasoning, and same call site, as the one-link form's.
        eat_scopes(seg, &mut p, &mut depth)
            .map_err(|_| prod_tag("chain-void-brace-scopes-do-not-close"))?;
        eat_return_plumbing(seg, &mut p, false, depth)
            .map_err(|_| prod_tag("chain-void-body-does-not-end-at-the-call"))?;
        SeqTail::Void
    } else if seg.get(p) == Some(&0x41) {
        eat_return_plumbing(seg, &mut p, true, depth)
            .map_err(|_| prod_tag("chain-returned-body-does-not-end-at-the-call"))?;
        // The last call's value IS the result, with no post-op — the same tail
        // `int f(){ g1(); return g2(); }` produces, which is no instruction at all.
        SeqTail::CallValue { add_k: 0 }
    } else if matches!(seg.get(p), Some(&0x33) | Some(&0x30)) {
        // **WCO** — a designator step on the chain's pointer result. One
        // instruction or none; see [`chain_result_designator`].
        //
        // Anchored on **either** byte. `*p->a()->b()` is a bare `30` with no
        // offset add in front of it and emits the same `lwz r3,0(r3)` as
        // `p->a()->b()->m` at offset 0 (measured, `e_deref` and `e_off0` in
        // `work/WCO/probe/p6.cpp`); anchoring on `33` alone would have been a
        // private limit inside a recognizer that already handles the general
        // case, which is the defect §6n's item 1 records six times over.
        chain_result_designator(seg, &mut p, depth)?
    } else {
        return Err(prod_tag("chain-body-does-not-end-at-the-call"));
    };

    // ---- from here the body parses to the end of the segment, so every refusal
    // is a codegen-class one over a COMPLETE body and gets its own key. ----

    let params = parse_params(seg, lo).map_err(Some)?;
    // Past the eighth formal a parameter is stack-homed and its setup is
    // `lwz r3,<slot>(r1)`, not a register move. The refusal is on the whole formals
    // LIST because that is the predicate `select_text` raises — the same reasoning
    // and the same key as the Class A statement sequence this body becomes.
    if params.len() > MAX_REGISTER_FORMALS {
        return Err(Some(Block::refuse(seg, p, "callseq-over-eight-formals")));
    }
    // The innermost call's arguments — the receiver plus any explicit ones —
    // validated and normalized through the ONE locator every other call shape
    // uses, exactly as `parse_call_sequence` does it. `call-arg-nonformal`,
    // the permutation-cycle bound and the computed-argument rules all arrive with
    // it rather than being restated here.
    let (arg_ops, arg_sources) =
        match tail_call_shape(seg, inner_args, params.clone(), methods[methods.len() - 1], p)
            .map_err(Some)?
        {
            BodyShape::VoidTailCall { .. } => (Vec::new(), None),
            BodyShape::IntTailCall { arg_ops, .. } => (arg_ops, None),
            // WLA's literal slot goes through `seq_call_arg_sources`, which
            // refuses it: a chain's innermost call is FRAMED, and the `li`'s
            // interleaving with the callee-saved copies is uncaptured there.
            // `callseq-multiarg-lit` is the shared key.
            BodyShape::MultiArgTailCall { arg_sources, .. } => (
                Vec::new(),
                Some(seq_call_arg_sources(seg, p, arg_sources).map_err(Some)?),
            ),
            // `tail_call_shape` returns exactly those three.
            _ => return Err(Some(Block::refuse(seg, p, "callseq-arg-shape"))),
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
            link_args: Some(link_arg_slots(seg, args, &params, p).map_err(Some)?),
        });
    }
    // Class A by construction — no later call reads a formal, so this returns
    // empty. Asked anyway, through the one locator, rather than asserting it: the
    // rule is the same one the statement sequence runs, and a private restatement
    // of "this is Class A" is exactly the shape of drift `GAPS.md` §6 records.
    let saved = plan_saved_gprs(seg, &params, &calls, 0, p).map_err(Some)?;
    Ok(BodyShape::CallSeq { params, calls, tail, saved })
}

/// **WCO — one designator step on the chain's pointer RESULT**:
/// `return p->a()->b()->m;` and `return &p->a()->b()->m;`.
///
/// ```text
///   … 4C                                the outermost call's apply
///   ( 33 <int-like> k  27 <PTR>          a RUN of byte-offset adds, summed —
///   | 33 <long>     k  28 00 00 )+       [`eat_offset_adds`], the shared walk
///   [ 30 <TYPE> [ 2C <same class> 00 ] ] the indirect load, when there is one
///   41 <TYPE>                            the result type
///   <return plumbing>
/// ```
///
/// **Two cells, one instruction each, and the second was already shipped.**
/// Read off the reference obj (`work/WCO/probe/p1.cpp`, `/O1 /GS- /c`):
///
/// ```text
///   int  c_off  (O* p) { return  p->Next()->gf()->m; }   bl ; bl ; lwz  r3,4(r3)
///   int  c_off0 (O* p) { return  p->Next()->gf()->a; }   bl ; bl ; lwz  r3,0(r3)
///   int* c_addr (O* p) { return &p->Next()->gf()->m; }   bl ; bl ; addi r3,r3,4
///   int* c_addr0(O* p) { return &p->Next()->gf()->a; }   bl ; bl ;   (nothing)
/// ```
///
/// The **address** cell is [`SeqTail::CallValue`], which has shipped since #35
/// rung 1 and folds `+0` away by itself, so it is a recognizer and nothing else.
/// The **load** cell is one new tail, [`SeqTail::CallLoad`], and it does *not*
/// fold at offset 0 — the two middle rows above are the whole difference and
/// they are 4 bytes of `.text` apart.
///
/// ## The locator, and the check in both directions
///
/// The offset run is [`eat_offset_adds`], shared, **not** a private single-add
/// copy. That is not a style preference: the indirect-load *leaf* carried
/// exactly such a private copy until W35, it refused **5,161** functions the
/// address and store leaves beside it accepted, and this file would have
/// reproduced the identical defect one production over. A run folds here for the
/// same reason it folds there — `p->Next()->gf()->in.b` is `27 · 27` and emits
/// one `lwz r3,48(r3)`, and `&p->Next()->gf()->arr[2]` is `27 · 28` and emits
/// one `addi r3,r3,60` (both measured, `work/WCO/probe/p6.cpp`).
///
/// The other direction — is there a locator nobody asks? — the `30`/`41` type
/// tail below **agrees rule for rule** with
/// [`super::leaf_load::finish_indirect_load_of`], measured across the whole
/// width table in one TU (`p6.cpp`), base already in r3:
///
/// ```text
///   int / int* / nested / subscripted   lwz  r3,k(r3)      — ADMITTED here
///   char, unsigned char                 lbz  r3,k(r3)
///   short, unsigned short               lhz  r3,k(r3)
///   long long                           ld   r3,k(r3)
///   char widened to int                 lbz  r11,k(r3) ; extsb r3,r11
///   float / double                      lfs / lfd  f1,k(r3)   — a different file
/// ```
///
/// Everything but the first row is **refused by name** rather than admitted,
/// because emitting them means moving `finish_indirect_load_of`'s width/sext
/// dispatch out of the leaf and into a locator both can call — which is a rung
/// and not a line, and the table above is its ceiling handed over intact.
fn chain_result_designator(
    seg: &[u8],
    p: &mut usize,
    depth: usize,
) -> Result<SeqTail, Option<Block>> {
    // The offset run — **zero or more**, because `*p->a()->b()` has none at all
    // and is the same instruction at displacement 0.
    let (off, _last_retype) = eat_offset_adds(seg, p)
        .ok_or_else(|| prod_tag("chain-tail-designator-not-an-offset-run"))?;
    let mut q = *p;
    let load = eat_byte(seg, &mut q, 0x30);
    // The whole rest of the step, read **structurally** — the load's TYPE, an
    // optional conversion, the result type, and the return plumbing — before a
    // single class gate is asked.
    //
    // **The order matters and it is a contract, not a style.** `Err(Some(b))`
    // means "this IS the production and it parsed to the end of the segment";
    // raising a width refusal at the `30` would claim bodies that carry a whole
    // further construct after the load and replace their measured
    // `-then-…-more` key with an uninformative one. The wild capture
    // `mcall::WILD_CHAIN_AS_RECV_LOAD` is exactly that body — a chain, an
    // indirect load of a `const float`, and then *more* — and it must keep
    // `expr-call-in-expr-chained-then-deref-load-more`.
    let loaded_at = q;
    let loaded = if load {
        let (tag, kind, _, tw) =
            read_type(seg, q).ok_or_else(|| prod_tag("chain-tail-load-has-no-type"))?;
        q += tw;
        // An optional conversion on the loaded value. Consumed generically here
        // and *classified* below, for the same reason as above.
        let conv = if seg.get(q) == Some(&0x2C) {
            let (t2, k2, _, tw2) = read_type(seg, q + 1)
                .ok_or_else(|| prod_tag("chain-tail-load-convert-has-no-type"))?;
            let mut probe = q + 1 + tw2;
            if !eat_byte(seg, &mut probe, 0x00) {
                return Err(prod_tag("chain-tail-load-convert-not-terminated"));
            }
            q = probe;
            Some((t2, k2))
        } else {
            None
        };
        Some((tag, kind, conv))
    } else {
        None
    };
    // The result type, stated again. Read generically; its class is checked
    // against the load's below.
    if !eat_byte(seg, &mut q, 0x41) {
        return Err(prod_tag("chain-tail-designator-has-no-result-type"));
    }
    let (rt, rk, _, rtw) =
        read_type(seg, q).ok_or_else(|| prod_tag("chain-tail-result-type-unreadable"))?;
    q += rtw;
    // The plumbing, which must reach the segment end. Asked WITHOUT a result
    // type (`false`) because the `41` is already consumed above — letting
    // `eat_return_head` take it instead would widen the check to its
    // `eat_int_like_or_ptr4` and lose the class agreement this step needs.
    eat_return_plumbing(seg, &mut q, false, depth)
        .map_err(|_| prod_tag("chain-tail-designator-body-does-not-end-here"))?;
    *p = q;

    // ---- from here the body parses to the end of the segment, so every refusal
    // is a codegen-class one over a COMPLETE body and gets its own key. ----

    // The `lwz`/`addi` displacement field is signed 16-bit. A wider one is
    // `addis`+`addi` or an indexed load with a scratch register — a different
    // instruction count, so it refuses rather than truncating. Gated on the SUM,
    // the same boundary `finish_indirect_load_of` draws for the leaf.
    if !(-0x8000..=0x7FFF).contains(&off) {
        return Err(Some(Block::refuse(seg, *p, "mcall-chain-tail-off-wide")));
    }
    let Some((tag, kind, conv)) = loaded else {
        // **The address form.** No load: the value is the adjusted pointer, and
        // the result type must say so. `SeqTail::CallValue` is the same `addi`
        // the W41 post-op emits and folds `+0` to nothing all by itself.
        if !is_ptr4_kind(rt, rk) {
            return Err(Some(Block::refuse(seg, *p, "mcall-chain-tail-addr-class")));
        }
        return Ok(SeqTail::CallValue { add_k: off });
    };
    // **The load form.** The loaded value must be one of the two width-4 classes
    // c2 lowers with the identical bare `lwz` — a 4-byte integer or a 4-byte
    // pointer (`docs/IL_LOAD_TYPES.md` §3/§4) — asked through the SAME
    // [`value_class`] predicate the leaf asks, so the two cannot drift about what
    // "width 4" means.
    let Some(cls) = value_class(tag, kind) else {
        // **WFL — the floating-point member**, which is the whole of what
        // `mcall-chain-tail-load-class` was worth: 717 functions, measured by
        // counterfactual rather than named. `lfs`/`lfd` into **f1**, a different
        // register file and a `_fltused` obligation besides. Asked through the
        // shared [`is_fp_type`] — the volatile-refusing, two-channel one the FP
        // tail call and the FP leaf ask — and NOT through a nibble test written
        // here, which is `docs/GAPS.md` §6's "one rule, two implementations".
        if let Some(double) = is_fp_type(tag, kind) {
            return fp_chain_result_load(seg, off, double, conv, rt, rk, loaded_at);
        }
        // Named, because what each costs is a number and not an argument. The
        // narrow and 8-byte scalars are `lbz`/`lhz`/`ld`; what is left in this
        // key after the FP class moved out is `volatile float`, and anything a
        // member may be that neither [`value_class`], [`sized_ptee`] nor
        // [`is_fp_type`] can spell.
        let ctx = match sized_ptee(tag, kind) {
            Some(_) => "mcall-chain-tail-load-width",
            None => "mcall-chain-tail-load-class",
        };
        return Err(Some(Block::refuse(seg, loaded_at, ctx)));
    };
    if !matches!(cls, ValueClass::Int4 | ValueClass::Ptr4) {
        return Err(Some(Block::refuse(seg, loaded_at, "mcall-chain-tail-load-class")));
    }
    // The conversion, when there was one, must stay **inside** the loaded class:
    // `2C int→int` and `2C ptr→ptr` are cv strips and emit nothing. A
    // cross-class one is a reinterpret this port has never probed, and a
    // widening out of a narrow class is the `extsb` the width gate above already
    // refuses. This gate has no witness among the spellings a caller can write
    // (the width gate fires first on every one of them) — which is exactly why
    // it refuses rather than being skipped.
    if let Some((t2, k2)) = conv {
        if value_class(t2, k2) != Some(cls) {
            return Err(Some(Block::refuse(seg, loaded_at, "mcall-chain-tail-load-convert")));
        }
    }
    // …and the result type restates the loaded class. Required rather than
    // skipped: every capture agrees byte for byte.
    if value_class(rt, rk) != Some(cls) {
        return Err(Some(Block::refuse(seg, loaded_at, "mcall-chain-tail-load-result")));
    }
    Ok(SeqTail::CallLoad { off })
}

/// **WFL** — the class tail of [`chain_result_designator`] when the loaded member
/// is floating point: one `lfs`/`lfd` into `f1`.
///
/// Reached only after the whole body has parsed to the end of the segment and the
/// displacement has been bounded, so every refusal here is a codegen-class one
/// over a complete body and carries its own key — the same contract the integer
/// tail's gates keep, and for the same reason (`Err(Some(b))` claims the
/// production).
///
/// The cells, read off the reference obj (`work/WFL/probe/p1.cpp`, `/O1 /GS- /c`,
/// base already in r3):
///
/// ```text
///   float  m;   return …->m;              lfs f1,k(r3)      c0230004
///   double m;   return …->m;              lfd f1,k(r3)      c8230010
///   float* p;   return *…;                lfs f1,0(r3)      — no fold at 0
///   float  m;   DOUBLE return             lfs f1,k(r3)      — byte-IDENTICAL
///   double m;   FLOAT  return             lfd f0,k(r3) ; frsp f1,f0   — REFUSED
/// ```
///
/// **The promotion is free and the narrowing is not, and neither is a guess.**
/// `lfs` loads *and converts* — the FP register holds a double either way — so a
/// `float` member returned as a `double` is the same single instruction, and the
/// emitted opcode follows the **member's** width rather than the result's. The
/// narrowing is two words *and its destination is `f0`*, which is the FP file's
/// first scratch and not the result register; it is a second production
/// (`float_leaf_text`'s pool would have to allocate it) and it refuses under its
/// own name rather than being folded in.
///
/// `lfd` is **D-form** (primary 50), unlike the integer `ld` that
/// [`super::leaf_load::finish_indirect_load_of`] guards with a 4-byte alignment
/// test: there is no DS-form displacement bit here, so an 8-byte load at an odd
/// displacement encodes fine and needs no gate. One instruction, two widths, one
/// encoder ([`c2_core::codegen::encode::encode_lfs`], which takes the width).
fn fp_chain_result_load(
    seg: &[u8],
    off: i32,
    double: bool,
    conv: Option<(u8, u8)>,
    rt: u8,
    rk: u8,
    loaded_at: usize,
) -> Result<SeqTail, Option<Block>> {
    // The conversion, when there is one. It must stay in the FP file, and within
    // it only the WIDENING direction is free.
    let value_double = match conv {
        None => double,
        Some((t2, k2)) => {
            let Some(conv_double) = is_fp_type(t2, k2) else {
                // FP → integer (`int f(){ return …->fltmember; }`) is a
                // `fctiwz`, a spill through the frame and a reload; nothing in
                // this tail's one instruction.
                return Err(Some(Block::refuse(seg, loaded_at, "mcall-chain-tail-load-fp-convert")));
            };
            if double && !conv_double {
                return Err(Some(Block::refuse(seg, loaded_at, "mcall-chain-tail-load-fp-narrow")));
            }
            conv_double
        }
    };
    // …and the result type restates the value's width after the conversion.
    // Required rather than skipped: every capture agrees byte for byte, and this
    // is the one place a promotion could be mis-read as a same-width strip.
    if is_fp_type(rt, rk) != Some(value_double) {
        return Err(Some(Block::refuse(seg, loaded_at, "mcall-chain-tail-load-fp-result")));
    }
    Ok(SeqTail::CallLoadFp { off, double })
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

    /// **Every non-committal decline out of the chain production — including the
    /// WCO designator tail — is attributed to a named site.** The corpus spans
    /// the two-link chain, the three-link one, a link with an argument, and the
    /// designator tails (offset run, deref, `lfs`/`lfd`), so a bail reachable
    /// only from inside `chain_result_designator` is swept too.
    #[test]
    fn no_decline_out_of_the_chain_production_lands_in_the_residue() {
        crate::func::body::shapes::mcall_tail::assert_no_decline_lands_in_the_residue(
            &[
                ("MC_CHAIN_RET", MC_CHAIN_RET),
                ("MC_CHAIN_THREE", MC_CHAIN_THREE),
                ("MC_LINK_ARG", MC_LINK_ARG),
                ("MC_LINK_LIT", MC_LINK_LIT),
                ("MC_TAIL_RUN", MC_TAIL_RUN),
                ("MC_TAIL_DEREF", MC_TAIL_DEREF),
                ("MC_TAIL_LFS", MC_TAIL_LFS),
                ("MC_TAIL_FP_NARROW", MC_TAIL_FP_NARROW),
            ],
            // MEASURED: 11,403 of the mutations reach the production.
            11_000,
        );
    }

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

    /// `int t_off4(O* p) { return p->Next()->gf()->m; }` — **the row**, the load
    /// cell at displacement 4. Its tail is
    /// `4C · 33 <int> 04 27 <ptr> · 30 <int> · 41 <int>`, where WCH's is a bare
    /// `41`. Transcribed verbatim from a live-toolchain capture
    /// (`work/WCO/probe/t1.cpp`, `c2rs census --keep-il`), not hand-assembled.
    const MC_TAIL_LOAD4: &[u8] = &[
    0x4F, 0x1F, 0x80, 0x05, 0x00, 0x20, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
    0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
    0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
    0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x04, 0x53, 0x53, 0x26, 0x05, 0x0A,
    0x46, 0x2D, 0x04, 0x0A, 0x4C, 0x4F, 0x11, 0x53, 0x26, 0xFC, 0x09, 0x26, 0xFB, 0x09, 0xB9,
    0x04, 0x0A, 0x86, 0x43, 0x81, 0x20, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00, 0xBD, 0x86, 0x43,
    0x81, 0x20, 0x00, 0x80, 0x04, 0x10, 0x00, 0x00, 0x4C, 0x99, 0x86, 0x43, 0x87, 0x20, 0x00,
    0xBD, 0x86, 0x43, 0x86, 0x20, 0x00, 0x80, 0x07, 0x10, 0x00, 0x00, 0x4C, 0x33, 0x86, 0x41,
    0x74, 0x04, 0x27, 0x86, 0x43, 0xF4, 0x08, 0x30, 0x86, 0x41, 0x74, 0x41, 0x86, 0x41, 0x74,
    0x3A, 0x06, 0x0A, 0x54, 0x02, 0x29, 0x06, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `int t_off0(O* p) { return p->Next()->gf()->a; }` — the same body at
    /// displacement **0**, which still emits `lwz r3,0(r3)`. Verbatim, same TU.
    const MC_TAIL_LOAD0: &[u8] = &[
    0x4F, 0x1F, 0x80, 0x05, 0x00, 0x20, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
    0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
    0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
    0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x05, 0x53, 0x53, 0x26, 0x08, 0x0A,
    0x46, 0x2D, 0x07, 0x0A, 0x4C, 0x4F, 0x11, 0x53, 0x26, 0xFC, 0x09, 0x26, 0xFB, 0x09, 0xB9,
    0x07, 0x0A, 0x86, 0x43, 0x81, 0x20, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00, 0xBD, 0x86, 0x43,
    0x81, 0x20, 0x00, 0x80, 0x04, 0x10, 0x00, 0x00, 0x4C, 0x99, 0x86, 0x43, 0x87, 0x20, 0x00,
    0xBD, 0x86, 0x43, 0x86, 0x20, 0x00, 0x80, 0x07, 0x10, 0x00, 0x00, 0x4C, 0x33, 0x86, 0x41,
    0x74, 0x00, 0x27, 0x86, 0x43, 0xF4, 0x08, 0x30, 0x86, 0x41, 0x74, 0x41, 0x86, 0x41, 0x74,
    0x3A, 0x09, 0x0A, 0x54, 0x02, 0x29, 0x09, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `int* t_addr4(O* p) { return &p->Next()->gf()->m; }` — the ADDRESS cell:
    /// the identical designator with the `30` load absent, and its `41` states a
    /// pointer. Verbatim, same TU.
    const MC_TAIL_ADDR4: &[u8] = &[
    0x4F, 0x1F, 0x80, 0x05, 0x00, 0x20, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
    0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
    0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
    0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x06, 0x53, 0x53, 0x26, 0x0B, 0x0A,
    0x46, 0x2D, 0x0A, 0x0A, 0x4C, 0x4F, 0x11, 0x53, 0x26, 0xFC, 0x09, 0x26, 0xFB, 0x09, 0xB9,
    0x0A, 0x0A, 0x86, 0x43, 0x81, 0x20, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00, 0xBD, 0x86, 0x43,
    0x81, 0x20, 0x00, 0x80, 0x04, 0x10, 0x00, 0x00, 0x4C, 0x99, 0x86, 0x43, 0x87, 0x20, 0x00,
    0xBD, 0x86, 0x43, 0x86, 0x20, 0x00, 0x80, 0x07, 0x10, 0x00, 0x00, 0x4C, 0x33, 0x86, 0x41,
    0x74, 0x04, 0x27, 0x86, 0x43, 0x8F, 0x20, 0x41, 0x86, 0x43, 0xF4, 0x08, 0x3A, 0x0C, 0x0A,
    0x54, 0x02, 0x29, 0x0C, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `int* t_addr0(O* p) { return &p->Next()->gf()->a; }` — the address cell at
    /// displacement 0, which emits **nothing at all**. Verbatim, same TU.
    const MC_TAIL_ADDR0: &[u8] = &[
    0x4F, 0x1F, 0x80, 0x05, 0x00, 0x20, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
    0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
    0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
    0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x07, 0x53, 0x53, 0x26, 0x0E, 0x0A,
    0x46, 0x2D, 0x0D, 0x0A, 0x4C, 0x4F, 0x11, 0x53, 0x26, 0xFC, 0x09, 0x26, 0xFB, 0x09, 0xB9,
    0x0D, 0x0A, 0x86, 0x43, 0x81, 0x20, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00, 0xBD, 0x86, 0x43,
    0x81, 0x20, 0x00, 0x80, 0x04, 0x10, 0x00, 0x00, 0x4C, 0x99, 0x86, 0x43, 0x87, 0x20, 0x00,
    0xBD, 0x86, 0x43, 0x86, 0x20, 0x00, 0x80, 0x07, 0x10, 0x00, 0x00, 0x4C, 0x33, 0x86, 0x41,
    0x74, 0x00, 0x27, 0x86, 0x43, 0x8F, 0x20, 0x41, 0x86, 0x43, 0xF4, 0x08, 0x3A, 0x0F, 0x0A,
    0x54, 0x02, 0x29, 0x0F, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `int t_run(O* p) { return p->Next()->gf()->in.y; }` — TWO offset adds,
    /// `27 · 27` at 16 and 4, folding into one `lwz r3,20(r3)`. The witness that
    /// this production shares [`eat_offset_adds`] rather than carrying the
    /// private single-add copy the load leaf had until W35. Verbatim, same TU.
    const MC_TAIL_RUN: &[u8] = &[
    0x4F, 0x1F, 0x80, 0x05, 0x00, 0x20, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
    0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
    0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
    0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x08, 0x53, 0x53, 0x26, 0x11, 0x0A,
    0x46, 0x2D, 0x10, 0x0A, 0x4C, 0x4F, 0x11, 0x53, 0x26, 0xFC, 0x09, 0x26, 0xFB, 0x09, 0xB9,
    0x10, 0x0A, 0x86, 0x43, 0x81, 0x20, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00, 0xBD, 0x86, 0x43,
    0x81, 0x20, 0x00, 0x80, 0x04, 0x10, 0x00, 0x00, 0x4C, 0x99, 0x86, 0x43, 0x87, 0x20, 0x00,
    0xBD, 0x86, 0x43, 0x86, 0x20, 0x00, 0x80, 0x07, 0x10, 0x00, 0x00, 0x4C, 0x33, 0x86, 0x41,
    0x74, 0x10, 0x27, 0x86, 0x43, 0x91, 0x20, 0x33, 0x86, 0x41, 0x74, 0x04, 0x27, 0x86, 0x43,
    0xF4, 0x08, 0x30, 0x86, 0x41, 0x74, 0x41, 0x86, 0x41, 0x74, 0x3A, 0x12, 0x0A, 0x54, 0x02,
    0x29, 0x12, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `int t_deref(O* p) { return *p->Next()->gpi(); }` — a bare `30` with **no**
    /// offset add in front of it, which is the same `lwz r3,0(r3)`. Verbatim,
    /// same TU.
    const MC_TAIL_DEREF: &[u8] = &[
    0x4F, 0x1F, 0x80, 0x05, 0x00, 0x20, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
    0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
    0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
    0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x09, 0x53, 0x53, 0x26, 0x14, 0x0A,
    0x46, 0x2D, 0x13, 0x0A, 0x4C, 0x4F, 0x11, 0x53, 0x26, 0xFD, 0x09, 0x26, 0xFB, 0x09, 0xB9,
    0x13, 0x0A, 0x86, 0x43, 0x81, 0x20, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00, 0xBD, 0x86, 0x43,
    0x81, 0x20, 0x00, 0x80, 0x04, 0x10, 0x00, 0x00, 0x4C, 0x99, 0x86, 0x43, 0x88, 0x20, 0x00,
    0xBD, 0x86, 0x43, 0xF4, 0x08, 0x00, 0x80, 0x08, 0x10, 0x00, 0x00, 0x4C, 0x30, 0x86, 0x41,
    0x74, 0x41, 0x86, 0x41, 0x74, 0x3A, 0x15, 0x0A, 0x54, 0x02, 0x29, 0x15, 0x0A, 0x4F, 0x12,
    0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x0A, 0x4D,
    ];

    fn tail_of(seg: &[u8]) -> SeqTail {
        let Some(BodyShape::CallSeq { tail, calls, saved, .. }) = parse_segment(seg, NO_LOCALS)
        else {
            panic!("the chained member call with a designator on its result");
        };
        assert_eq!(calls.len(), 2, "still two calls; the tail is not one");
        assert_eq!(saved, Vec::<usize>::new(), "the tail costs no callee-saved GPR");
        tail
    }

    #[test]
    fn a_designator_on_the_chain_result_is_a_load_or_an_add_and_they_differ_at_zero() {
        // **The pair the whole rung rests on.** The two forms are the same IL
        // apart from the `30`, and at displacement 0 they emit a different
        // NUMBER of instructions: `lwz r3,0(r3)` against nothing at all.
        // Merging them — treating "offset 0 emits nothing" as a property of the
        // designator rather than of the add — is four bytes of `.text` wrong on
        // every `f_off0`-shaped body, and both captures are here so the two
        // cannot be conflated by a later edit.
        assert_eq!(tail_of(MC_TAIL_LOAD4), SeqTail::CallLoad { off: 4 });
        assert_eq!(tail_of(MC_TAIL_LOAD0), SeqTail::CallLoad { off: 0 });
        assert_eq!(tail_of(MC_TAIL_ADDR4), SeqTail::CallValue { add_k: 4 });
        assert_eq!(tail_of(MC_TAIL_ADDR0), SeqTail::CallValue { add_k: 0 });
        // A bare `30` with no add at all is the load at displacement 0 — the
        // same instruction, reached by a different spelling.
        assert_eq!(tail_of(MC_TAIL_DEREF), SeqTail::CallLoad { off: 0 });
    }

    #[test]
    fn the_offset_run_folds_because_the_locator_is_shared() {
        // `->in.y` is `27` at 16 then `27` at 4, and c2 emits one
        // `lwz r3,20(r3)`. A private single-add copy — the exact defect the
        // indirect-load leaf carried until W35, worth 5,161 functions there —
        // would refuse this.
        assert_eq!(tail_of(MC_TAIL_RUN), SeqTail::CallLoad { off: 20 });
    }

    #[test]
    fn a_loaded_width_that_is_not_four_refuses_by_name() {
        // The `30`'s TYPE retyped from `int` (`86 41 74`) to a 1-byte integer
        // (`82 11 70`, captured in `wco_chain_tail_load_neg.cpp`'s `n_char`).
        // That is `lbz`, a different instruction, and the body must refuse with
        // a name rather than emit an `lwz` — the successor that widens the tail
        // needs this number, not an argument.
        let at = MC_TAIL_LOAD4
            .windows(4)
            .rposition(|w| w[0] == 0x30 && w[1] == 0x86 && w[2] == 0x41 && w[3] == 0x74)
            .expect("the indirect load");
        let mut seg = MC_TAIL_LOAD4.to_vec();
        seg.splice(at + 1..at + 4, [0x82, 0x11, 0x70]);
        // …and the `41` result type restates it, so both copies move together.
        let at2 = seg
            .windows(4)
            .rposition(|w| w[0] == 0x41 && w[1] == 0x86 && w[2] == 0x41 && w[3] == 0x74)
            .expect("the result type");
        seg.splice(at2 + 1..at2 + 4, [0x82, 0x11, 0x70]);
        assert_eq!(parse_segment(&seg, NO_LOCALS), None);
        assert_eq!(
            parse_segment_detail(&seg, NO_LOCALS).unwrap_err().ctx,
            "mcall-chain-tail-load-width"
        );
    }

    #[test]
    fn a_displacement_past_sixteen_bits_refuses_by_name() {
        // The `addi`/`lwz` immediate is signed 16-bit; 0x9C40 (40,000) needs
        // `addis`+`addi` or an indexed load with a scratch register. The gate is
        // on the SUM, the same boundary the indirect-load leaf draws.
        let at = MC_TAIL_LOAD4
            .windows(6)
            .rposition(|w| w[0] == 0x33 && w[1] == 0x86 && w[2] == 0x41 && w[3] == 0x74 && w[5] == 0x27)
            .expect("the offset add");
        let mut seg = MC_TAIL_LOAD4.to_vec();
        // `80 <LE32>` — the wide literal payload, exactly as captured for
        // `n_far` in `wco_chain_tail_load_neg.cpp` (40,000 = 0x9C40).
        seg.splice(at + 4..at + 5, [0x80, 0x40, 0x9C, 0x00, 0x00]);
        assert_eq!(parse_segment(&seg, NO_LOCALS), None);
        assert_eq!(
            parse_segment_detail(&seg, NO_LOCALS).unwrap_err().ctx,
            "mcall-chain-tail-off-wide"
        );
    }

    /// **A WILD witness that this production must NOT accept**, from
    /// `src/system/hamobj/Ham.cpp` at the workload's flags — the same segment
    /// `mcall::WILD_CHAIN_AS_RECV_LOAD` carries, kept here because WCO is what
    /// moved its census key. It is a two-link chain with an `int` argument on
    /// the outer link, followed by `30 A6 45 F3 30` — an indirect load of a
    /// `const float` member — and it parses to the end of the segment.
    ///
    /// So it IS this production. WCO refused it by name
    /// (`mcall-chain-tail-load-class`) because a `float` member is
    /// `lfs f1,k(r3)`, a different register file and a `_fltused` obligation
    /// besides; **WFL emits that instruction**, and this wild segment is the
    /// row's acceptance witness rather than its refusal one. It is kept under
    /// its original name because what makes it valuable is unchanged: it is a
    /// real body from the workload, and the `2C 86 45 40` cv strip off a
    /// `const float` is a spelling no hand-written fixture in this repo
    /// produced.
    const WILD_CHAIN_FLOAT_MEMBER: &[u8] = &[
        0x4C, 0x4F, 0x11, 0x53, 0x26, 0xF5, 0x42, 0x26, 0x6B, 0x43, 0xB9, 0x8F, 0x43, 0x86, 0x43,
        0xFE, 0x31, 0x99, 0x86, 0x43, 0xF2, 0x31, 0x00, 0xBD, 0x86, 0x43, 0xCB, 0x31, 0x00, 0x80,
        0xF2, 0x18, 0x00, 0x00, 0x4C, 0x99, 0x86, 0x43, 0xC3, 0x31, 0x00, 0xBD, 0x86, 0x43, 0xF4,
        0x30, 0x00, 0x80, 0xC3, 0x18, 0x00, 0x00, 0xB9, 0x75, 0x43, 0x86, 0x41, 0x74, 0x55, 0x86,
        0x41, 0x74, 0x4C, 0x30, 0xA6, 0x45, 0xF3, 0x30, 0x2C, 0x86, 0x45, 0x40, 0x00, 0x41, 0x86,
        0x45, 0x40, 0x3A, 0x90, 0x43, 0x54, 0x02, 0x29, 0x90, 0x43, 0x4F, 0x12, 0x47, 0x54, 0x01,
        0x54, 0x00,
    ];

    /// `float t_lfs(O* p) { return p->Next()->gf()->f; }` — **the row**: the
    /// designator step whose member is a `float`, which is one `lfs f1,4(r3)`
    /// (`c0230004`). Its tail is `4C · 33 <int> 04 27 <ptr> · 30 <FLOAT> ·
    /// 41 <FLOAT>`, where the integer form's two TYPEs are `86 41 74`.
    /// Transcribed verbatim from a live-toolchain capture
    /// (`work/WFL/probe/t1.cpp`, `c2rs census --keep-il`), not hand-assembled.
    const MC_TAIL_LFS: &[u8] = &[
    0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
    0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
    0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
    0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x06, 0x53, 0x53, 0x26, 0xF8, 0x09,
    0x46, 0x2D, 0xF7, 0x09, 0x4C, 0x4F, 0x11, 0x53, 0x26, 0xEF, 0x09, 0x26, 0xEE, 0x09, 0xB9,
    0xF7, 0x09, 0x86, 0x43, 0x81, 0x20, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00, 0xBD, 0x86, 0x43,
    0x81, 0x20, 0x00, 0x80, 0x04, 0x10, 0x00, 0x00, 0x4C, 0x99, 0x86, 0x43, 0x87, 0x20, 0x00,
    0xBD, 0x86, 0x43, 0x86, 0x20, 0x00, 0x80, 0x07, 0x10, 0x00, 0x00, 0x4C, 0x33, 0x86, 0x41,
    0x74, 0x04, 0x27, 0x86, 0x43, 0xC0, 0x08, 0x30, 0x86, 0x45, 0x40, 0x41, 0x86, 0x45, 0x40,
    0x3A, 0xF9, 0x09, 0x54, 0x02, 0x29, 0xF9, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `double t_lfd(O* p) { return p->Next()->gf()->d; }` — the same body with
    /// a `double` member: one `lfd f1,8(r3)` (`c8230008`). One encoder, one
    /// width bit, and NO alignment gate — `lfd` is D-form (primary 50), unlike
    /// the integer `ld` the load leaf has to guard. Verbatim, same TU.
    const MC_TAIL_LFD: &[u8] = &[
    0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
    0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
    0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
    0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x07, 0x53, 0x53, 0x26, 0xFB, 0x09,
    0x46, 0x2D, 0xFA, 0x09, 0x4C, 0x4F, 0x11, 0x53, 0x26, 0xEF, 0x09, 0x26, 0xEE, 0x09, 0xB9,
    0xFA, 0x09, 0x86, 0x43, 0x81, 0x20, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00, 0xBD, 0x86, 0x43,
    0x81, 0x20, 0x00, 0x80, 0x04, 0x10, 0x00, 0x00, 0x4C, 0x99, 0x86, 0x43, 0x87, 0x20, 0x00,
    0xBD, 0x86, 0x43, 0x86, 0x20, 0x00, 0x80, 0x07, 0x10, 0x00, 0x00, 0x4C, 0x33, 0x86, 0x41,
    0x74, 0x08, 0x27, 0x88, 0x43, 0xC1, 0x08, 0x30, 0x88, 0x85, 0x41, 0x41, 0x88, 0x85, 0x41,
    0x3A, 0xFC, 0x09, 0x54, 0x02, 0x29, 0xFC, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `float t_deref(O* p) { return *p->Next()->gpf(); }` — a bare `30` with
    /// no offset add at all, which is the same `lfs f1,0(r3)`. The load form
    /// does **not** fold at displacement 0; the address form does, and that
    /// pair is what makes `CallLoadFp` a variant rather than a flag.
    /// Verbatim, same TU.
    const MC_TAIL_FP_DEREF: &[u8] = &[
    0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
    0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
    0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
    0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x08, 0x53, 0x53, 0x26, 0xFE, 0x09,
    0x46, 0x2D, 0xFD, 0x09, 0x4C, 0x4F, 0x11, 0x53, 0x26, 0xF0, 0x09, 0x26, 0xEE, 0x09, 0xB9,
    0xFD, 0x09, 0x86, 0x43, 0x81, 0x20, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00, 0xBD, 0x86, 0x43,
    0x81, 0x20, 0x00, 0x80, 0x04, 0x10, 0x00, 0x00, 0x4C, 0x99, 0x86, 0x43, 0x88, 0x20, 0x00,
    0xBD, 0x86, 0x43, 0xC0, 0x08, 0x00, 0x80, 0x08, 0x10, 0x00, 0x00, 0x4C, 0x30, 0x86, 0x45,
    0x40, 0x41, 0x86, 0x45, 0x40, 0x3A, 0xFF, 0x09, 0x54, 0x02, 0x29, 0xFF, 0x09, 0x4F, 0x12,
    0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `double t_promote(O* p) { return p->Next()->gf()->f; }` — a `float`
    /// member returned as a `double`. `2C 88 85 41 00` sits between the load
    /// and the result, and the emitted instruction is **unchanged**:
    /// `lfs f1,4(r3)`, byte-identical to [`MC_TAIL_LFS`], because `lfs` loads
    /// and converts in one instruction. So the width bit follows the MEMBER
    /// and not the result, and refusing "any conversion" would have been a
    /// discount on a measured-free cell. Verbatim, same TU.
    const MC_TAIL_FP_PROMOTE: &[u8] = &[
    0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
    0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
    0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
    0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x09, 0x53, 0x53, 0x26, 0x01, 0x0A,
    0x46, 0x2D, 0x00, 0x0A, 0x4C, 0x4F, 0x11, 0x53, 0x26, 0xEF, 0x09, 0x26, 0xEE, 0x09, 0xB9,
    0x00, 0x0A, 0x86, 0x43, 0x81, 0x20, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00, 0xBD, 0x86, 0x43,
    0x81, 0x20, 0x00, 0x80, 0x04, 0x10, 0x00, 0x00, 0x4C, 0x99, 0x86, 0x43, 0x87, 0x20, 0x00,
    0xBD, 0x86, 0x43, 0x86, 0x20, 0x00, 0x80, 0x07, 0x10, 0x00, 0x00, 0x4C, 0x33, 0x86, 0x41,
    0x74, 0x04, 0x27, 0x86, 0x43, 0xC0, 0x08, 0x30, 0x86, 0x45, 0x40, 0x2C, 0x88, 0x85, 0x41,
    0x00, 0x41, 0x88, 0x85, 0x41, 0x3A, 0x02, 0x0A, 0x54, 0x02, 0x29, 0x02, 0x0A, 0x4F, 0x12,
    0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `float t_narrow(O* p) { return (float)p->Next()->gf()->d; }` — the other
    /// direction, and the one that is not free: `lfd f0,8(r3) ; frsp f1,f0`,
    /// two words whose load destination is **f0**, the FP pool's first scratch.
    /// Verbatim, same TU.
    const MC_TAIL_FP_NARROW: &[u8] = &[
    0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
    0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
    0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
    0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x0A, 0x53, 0x53, 0x26, 0x04, 0x0A,
    0x46, 0x2D, 0x03, 0x0A, 0x4C, 0x4F, 0x11, 0x53, 0x26, 0xEF, 0x09, 0x26, 0xEE, 0x09, 0xB9,
    0x03, 0x0A, 0x86, 0x43, 0x81, 0x20, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00, 0xBD, 0x86, 0x43,
    0x81, 0x20, 0x00, 0x80, 0x04, 0x10, 0x00, 0x00, 0x4C, 0x99, 0x86, 0x43, 0x87, 0x20, 0x00,
    0xBD, 0x86, 0x43, 0x86, 0x20, 0x00, 0x80, 0x07, 0x10, 0x00, 0x00, 0x4C, 0x33, 0x86, 0x41,
    0x74, 0x08, 0x27, 0x88, 0x43, 0xC1, 0x08, 0x30, 0x88, 0x85, 0x41, 0x2C, 0x86, 0x45, 0x40,
    0x00, 0x41, 0x86, 0x45, 0x40, 0x3A, 0x05, 0x0A, 0x54, 0x02, 0x29, 0x05, 0x0A, 0x4F, 0x12,
    0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `int t_toint(O* p) { return p->Next()->gf()->f; }` — out of the FP file
    /// entirely: `lfs f0 ; fctiwz f0,f0 ; stfd f0,80(r1) ; lwz r3,84(r1)`, four
    /// words and a frame slot. Verbatim, same TU.
    const MC_TAIL_FP_TOINT: &[u8] = &[
    0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
    0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
    0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
    0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x0B, 0x53, 0x53, 0x26, 0x08, 0x0A,
    0x46, 0x2D, 0x07, 0x0A, 0x4C, 0x4F, 0x11, 0x53, 0x26, 0xEF, 0x09, 0x26, 0xEE, 0x09, 0xB9,
    0x07, 0x0A, 0x86, 0x43, 0x81, 0x20, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00, 0xBD, 0x86, 0x43,
    0x81, 0x20, 0x00, 0x80, 0x04, 0x10, 0x00, 0x00, 0x4C, 0x99, 0x86, 0x43, 0x87, 0x20, 0x00,
    0xBD, 0x86, 0x43, 0x86, 0x20, 0x00, 0x80, 0x07, 0x10, 0x00, 0x00, 0x4C, 0x33, 0x86, 0x41,
    0x74, 0x04, 0x27, 0x86, 0x43, 0xC0, 0x08, 0x30, 0x86, 0x45, 0x40, 0x2C, 0x86, 0x41, 0x74,
    0x00, 0x41, 0x86, 0x41, 0x74, 0x3A, 0x09, 0x0A, 0x54, 0x02, 0x29, 0x09, 0x0A, 0x4F, 0x12,
    0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x0C, 0x4D,
    ];

    #[test]
    fn a_floating_point_member_on_the_chain_result_is_an_lfs_or_an_lfd() {
        // **The pair this rung rests on**, and the width bit is the only thing
        // between them: `lfs f1,4(r3)` = c0230004 and `lfd f1,8(r3)` = c8230008.
        assert_eq!(tail_of(MC_TAIL_LFS), SeqTail::CallLoadFp { off: 4, double: false });
        assert_eq!(tail_of(MC_TAIL_LFD), SeqTail::CallLoadFp { off: 8, double: true });
        // A bare `30` with no add is the same load at displacement 0 — it does
        // NOT fold, exactly as the integer form does not.
        assert_eq!(tail_of(MC_TAIL_FP_DEREF), SeqTail::CallLoadFp { off: 0, double: false });
    }

    #[test]
    fn the_promotion_is_free_and_the_width_follows_the_member() {
        // `double f(){ return …->floatmember; }` is byte-identical to the
        // unpromoted body, so `double` here must stay **false**: it is the
        // LOADED width. A rule that read the result type instead would emit
        // `lfd` where c2 emits `lfs` — four bytes wrong inside an accepted
        // class, which no census number would show.
        assert_eq!(tail_of(MC_TAIL_FP_PROMOTE), SeqTail::CallLoadFp { off: 4, double: false });
        assert_eq!(
            tail_of(MC_TAIL_FP_PROMOTE),
            tail_of(MC_TAIL_LFS),
            "the promotion changes no instruction"
        );
    }

    #[test]
    fn the_narrowing_and_the_integer_conversion_refuse_by_name() {
        // Two words and a scratch register (`lfd f0 ; frsp f1,f0`), against this
        // tail's one instruction into f1.
        assert_eq!(parse_segment(MC_TAIL_FP_NARROW, NO_LOCALS), None);
        assert_eq!(
            parse_segment_detail(MC_TAIL_FP_NARROW, NO_LOCALS).unwrap_err().ctx,
            "mcall-chain-tail-load-fp-narrow"
        );
        // …and leaving the FP file is `fctiwz` plus a spill through the frame.
        assert_eq!(parse_segment(MC_TAIL_FP_TOINT, NO_LOCALS), None);
        assert_eq!(
            parse_segment_detail(MC_TAIL_FP_TOINT, NO_LOCALS).unwrap_err().ctx,
            "mcall-chain-tail-load-fp-convert"
        );
    }

    #[test]
    fn a_volatile_floating_point_member_stays_refused_and_the_key_says_so() {
        // The residue of `mcall-chain-tail-load-class` after the FP class moved
        // out. c2 emits the IDENTICAL single `lfs f1,k(r3)` for a
        // `volatile float` member (measured, `c_vol` in
        // `work/WFL/probe/p4.cpp`), so this refusal costs coverage and not
        // correctness — and it is kept because the predicate asked here is the
        // SHARED [`is_fp_type`], whose volatile refusal is right at the position
        // it was written for (a `volatile float` FORMAL is a spill). Splitting
        // that locator by position is a rung in `readers.rs`, not a line here;
        // this test is what makes the choice visible instead of implicit.
        let at = MC_TAIL_LFS
            .windows(3)
            .rposition(|w| w[0] == 0x30 && w[1] == 0x86 && w[2] == 0x45)
            .expect("the indirect load");
        let mut seg = MC_TAIL_LFS.to_vec();
        seg[at + 1] = 0x96; // `86` -> `96`, the volatile tag
        assert_eq!(parse_segment(&seg, NO_LOCALS), None);
        assert_eq!(
            parse_segment_detail(&seg, NO_LOCALS).unwrap_err().ctx,
            "mcall-chain-tail-load-class"
        );
    }

    #[test]
    fn the_fp_tail_is_what_puts_fltused_in_the_obj() {
        // W36 lost a symbol by missing a shape in this producer, and a `CallSeq`
        // is integer-shaped in every field but its tail — so the failure mode
        // here is an obj one symbol short on every positive case at once, which
        // is `Port=Mismatch @ offset 12` (the COFF header's `NumberOfSymbols`)
        // and nothing a census number would show.
        let fp = crate::func::IlFunction {
            call_seq: Some(crate::func::CallSeq {
                calls: Vec::new(),
                tail: crate::func::SeqTail::CallLoadFp { off: 4, double: false },
                saved: Vec::new(),
            }),
            ..crate::func::IlFunction::base("?f@@YAMPAUO@@@Z", &None)
        };
        assert!(fp.touches_floating_point(), "the FP tail is a `_fltused` producer");
        // …and the integer sibling one line away is not.
        let int_tail = crate::func::IlFunction {
            call_seq: Some(crate::func::CallSeq {
                calls: Vec::new(),
                tail: crate::func::SeqTail::CallLoad { off: 4 },
                saved: Vec::new(),
            }),
            ..crate::func::IlFunction::base("?g@@YAHPAUO@@@Z", &None)
        };
        assert!(!int_tail.touches_floating_point());
    }

    #[test]
    fn a_wild_float_member_on_the_chain_result_is_the_fp_load() {
        // The `const float` cv strip (`30 A6 45 F3 30 · 2C 86 45 40 00`) has to
        // survive: `A6` is the const spelling and the whole reason this rung
        // asks the nibble-reading [`is_fp_type`] rather than whitelisting the
        // two bare triples — `docs/ROADMAP.md` §6d's lesson, which cost 15,924
        // functions when it was learned on the integer file.
        // **This fragment cannot be accepted whole, and WCO's version of this
        // test did not say so.** The transcription starts at the `4C` and
        // carries no `46 2D` formals marker, so `parse_params` refuses it — and
        // `parse_params` runs *after* the tail designator. The old assertion
        // passed because the tail gate short-circuited in front of it; the
        // fragment was never proving that the whole body parsed.
        //
        // What it can prove, and does, is that the DESIGNATOR now takes this
        // spelling: unmodified the refusal has moved past the tail entirely, and
        // one byte changed — the `const` tag `A6` to the `volatile` `96` — puts
        // it straight back on the tail's own key. Two runs of one fragment, and
        // the difference between them is the gate under test.
        let seg = crate::func::test_fixtures::free_fn(WILD_CHAIN_FLOAT_MEMBER);
        assert_eq!(
            parse_segment_detail(&seg, NO_LOCALS).unwrap_err().ctx,
            "formals-marker",
            "the tail is past; what stops this fragment is its own truncation"
        );
        let at = seg
            .windows(3)
            .rposition(|w| w[0] == 0x30 && w[1] == 0xA6 && w[2] == 0x45)
            .expect("the indirect load of the `const float` member");
        let mut vol = seg.clone();
        vol[at + 1] = 0x96;
        assert_eq!(
            parse_segment_detail(&vol, NO_LOCALS).unwrap_err().ctx,
            "mcall-chain-tail-load-class",
            "and the tail gate is still there, one tag byte away"
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

