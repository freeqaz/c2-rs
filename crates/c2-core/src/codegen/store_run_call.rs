//! **Board #844 — the composition seam**: a scheduled store run as the MIDDLE
//! of a framed body.
//!
//! ```text
//!   H::H(unsigned initSize, unsigned size) { mSize = size; mCount = 0; Alloc(initSize); }
//!
//!     mflr r12 ; stw r12,-8(r1) ; std r31,-16(r1) ; stwu r1,-96(r1)   the frame
//!     li r11,0 ; stw r5,16(r3)                                        the RUN,
//!     mr r31,r3                                                       spliced
//!     stw r11,20(r3)                                                  through
//!     bl ?Alloc@H@@QAAPAUBE@@I@Z                                      the call
//!     mr r3,r31                                                       return this
//!     addi r1,r1,96 ; lwz r12,-8(r1) ; mtlr r12 ; ld r31,-16(r1) ; blr
//! ```
//!
//! # Nothing here is a new emitter
//!
//! Every part already existed and is consumed rather than re-derived (board
//! #842's lesson: *for each fact the obj states, go read whether a module
//! already produces it*). [`codegen::frame`] owns the prologue, epilogue and the
//! 96-byte frame class; [`super::calls::call_seq_text`] owns the `bl`'s
//! self-encoded displacement, the REL24 site and the block layout;
//! [`super::leaf::store::scheduled_gpr_run`] owns the run —
//! [`order::schedule`] for what goes where and [`alloc::allocate`] for which
//! register; `coff::pdata` and `coff::label` own the `.pdata` word and the
//! `$M`/`$T` labels. What board #844 was missing is **two facts**, and this file
//! is exactly those two:
//!
//!   1. a **carrier**, so the model can spell "a run *and* a call" — that is
//!      [`c2_il::CallSeq::store_run`], and the argument for putting it there is
//!      in its own doc;
//!   2. **where `mr r31,r3` goes inside the run** — §The splice.
//!
//! # The transfer is MEASURED, not assumed
//!
//! Board #866 measured over 96 cells that the leaf schedule and allocation
//! transfer unchanged into a framed body. This lane re-measured it at the
//! **workload's own** `/GR /O1 /Oi /EHsc` (board #1112 — a different population
//! from the sweep's `/Ox /GS- /c`) on the cells this emitter actually serves:
//! `work/w-seam2/grid/`, 34 accept cells each beside a **leaf control** with the
//! identical run and no call. Delete the `mr r31,r3` from the framed run and the
//! remaining text is **string-identical to the leaf's, 34 of 34**. That is why
//! this file splices into [`scheduled_gpr_run`]'s output instead of scheduling
//! again: the run is the same run, and two schedulers over one fact is
//! `GAPS.md` §6's recurring defect.
//!
//! # The splice, and its DOMAIN BOUNDARY
//!
//! `w-seam`/#867 fitted the copy's position on 24 cells and held it on an 18/18
//! fresh holdout, and **never shipped it**:
//!
//! ```text
//!   stores_before_mr = nprod - 1 + min(u, 2)
//! ```
//!
//! `nprod` is the number of distinct producers and `u` the number of stores that
//! materialise nothing. This lane scored it cell by cell against real `c2` bytes
//! and **found its boundary**:
//!
//! ```text
//!   GRID S   (36 accept cells, frozen before the first cl.exe)   32 HIT  4 MISS
//!   GRID S2  (16 cells, declared post-hoc holdout)               16 HIT  0 MISS
//!
//!   every MISS is nprod == 0 with u <= 1, where the formula is negative or off
//!   by one:   (0,0) -> -1, observed 0      (0,1) -> 0, observed 1
//!   and at nprod == 0, u >= 2 it is right:  (0,2) (0,3) (0,4) (0,5) (0,6) -> 1
//! ```
//!
//! So the rule ships **unchanged** and its domain is refused instead of patched.
//! A second clause fitted on the four cells that produced it is exactly how all
//! six refuted allocation keys got written (`w-heap` §4.1.1, boards #836/#868),
//! and `min(u, 1)` at `nprod == 0` would rest on **one** structural cell.
//! [`REFUSED_EMPTY_POOL`] is that refusal and
//! `the_mr_slot_domain_boundary_is_refused_with_its_counterexample` pins the
//! counterexample beside it, so the clause cannot be re-derived without meeting
//! the body it gets wrong.
//!
//! # `u` IS THE LEADING RUN, AND THIS FILE USED TO SAY THE OPPOSITE
//!
//! Board **#1212**. Until `w-mrslot` the paragraph here read:
//!
//! > *`u` has a second reading — [`order::layout_slots`]'s `u`, the leading run
//! > of unproduced stores in the final order (board #584) — and the two are not
//! > separated by any cell in either grid. **They cannot be**: `store_order`
//! > forbids a store whose producer has rank `j` from occupying a position below
//! > `u + j`, so the leading run is always at least `min(2, total)`, and
//! > therefore `min(leading, 2) == min(total, 2)` identically. The count is used
//! > here because it is the one the emitter can compute without re-deriving the
//! > order.*
//!
//! Every sentence of that is true **on a single-symbol run** — which is every
//! cell #867 was fitted on, every cell of its 18/18 holdout, and every cell of
//! the two grids it is talking about. It is false the moment the run has a
//! second base symbol, because the cross-symbol pin can strand an unproduced
//! store *behind* a produced one and the leading run stops there while the count
//! keeps counting. A reference bind **is** a second base symbol (board #1128),
//! so board #1199's carrier is exactly what opened the region — and four
//! `88-store-run-call` sweep cases plus 56 cross cells graded `Port=Mismatch`
//! against `w-carrier`'s first emitter, with its own 53-cell frozen grid green
//! through every one (board #1211).
//!
//! ```text
//!   H::H(unsigned a, unsigned b) { BE& lh = mListHead; mCount = 0;
//!                                  lh.mNext = (BE*)this; Reset(); }
//!   c2:   li 11,0 ; mr 31,3 ; stw 11,20(3) ; stw 3,8(3) ; bl
//!   count: the copy lands after ONE store.   leading run: after ZERO.  c2: ZERO.
//! ```
//!
//! [`save_slot`] is fed [`order::leading_unproduced`]'s answer now, carried as
//! `ScheduledRun::u_lead`. The old excuse — *"the one the emitter can compute
//! without re-deriving the order"* — was never true either:
//! [`scheduled_gpr_run`] has already asked [`order::schedule`] by the time it
//! fills the field, so the final order is in hand.
//!
//! **What the swap is worth, and what it cannot be**: `w-mrslot`'s GRID R, 145
//! cells frozen before the first `cl.exe`, 93 with an observed `mr r31,r3`,
//! every quantity read out of real `c2.dll`'s own words — the leading run is
//! **93 HIT / 0 MISS**, the count **63 / 30**. The swap is provably **inert on
//! every single-symbol run** (`order`'s own 5,460-cell enumeration), so it moves
//! no byte the port already emitted.

use c2_il::IlFunction;

use super::encode::mop_mr;
use super::leaf::store::scheduled_gpr_run;
use super::mop::{ops_to_bytes, Ops};
use super::select::out_of_class;
use crate::BackendError;

/// **The refusal that board #866's refutation earned.** A store whose value is a
/// formal the call keeps alive: the run is NOT the leaf's run there.
///
/// ```text
///   void P::lf(unsigned a, unsigned b) { m0=0; m1=b; m2=a; }        LEAF
///       li 11,0 ; stw 5,4(3) ; stw 4,8(3) ; stw 11,0(3) ; blr
///   P::P(unsigned a, unsigned b) { m0=0; m1=b; m2=a; Alloc(a); }    FRAMED
///       li 11,0 ; stw 4,8(3) ; stw 5,4(3) ; mr 31,3 ; stw 11,0(3) ; bl
/// ```
///
/// `a` is the call's argument and is live until the `bl`; `b` dies at its store.
/// The two unproduced stores **swap**, and the same body with a *nullary* callee
/// does not swap. That is two right words in the wrong order — an obj that
/// links, and exactly the failure class board #232 is.
///
/// **This is a REFUSAL and not a model.** The hoist rule that would emit these
/// correctly is visible in `work/w-seam2/grid3/` and is deliberately not fitted:
/// it would rest on the four cells that refuted the lane, which is how all six
/// refuted allocation keys got written.
///
/// # This is the BACKSTOP; the gate itself is the reader's
///
/// `c2_il`'s `try_parse_store_run_call` applies the same predicate, so the
/// census cannot count a body `PortC2` refuses — `census_gate.rs` is that
/// invariant and it caught this exact over-claim when the refusal lived here
/// alone. Kept here as well because a parser that widened past its witness must
/// come out as a gap and not as bytes, and because the two locators state the
/// same fact from the two sides that have to agree.
///
/// **It deliberately over-refuses one measured shape**: `work/w-seam2/grid3/p3`
/// stores the live argument FIRST in source order, where the hoist is a no-op
/// and the framed run is the leaf's. It is refused anyway — the gate is
/// syntactic, and a gate that reasoned about whether the hoist would be visible
/// is the model this refusal exists to avoid.
pub const LIVE_ARG_STORED: &str =
    "a store run before a call that stores a formal the call keeps alive: the \
     run's order is NOT the leaf's there (board #866 is refuted in general), and \
     the framed schedule for it is unmodeled";

/// **Board #1212's refusal, LIFTED — kept as a name so the correction has a
/// place to be recorded and so a reviewer meets the mechanism before the code.**
///
/// `w-carrier` shipped this string rather than the fix, and it was right to:
/// the correction changes a rule that governs **every** #844 body, and at the
/// time it would have rested on the four cells that refuted that lane. It rests
/// on GRID R now — 93 cells with an observed copy, 30 of which separate the two
/// readings, `0` MISS for the leading run and `30` for the count.
///
/// The string is retained (rather than deleted with the clause) because
/// `a_bind_carrying_run_emits_the_leading_run_slot_not_the_count` quotes it as
/// the thing that was refused, and a refusal whose text is gone cannot be shown
/// to have been the same refusal.
pub const BIND_IN_A_COMPOSITION_WAS: &str =
    "a store run before a call that carries a reference bind: the copy's slot \
     rule (board #867) is fed the COUNT of unproduced stores, which equals \
     board #584's leading run only on a SINGLE-symbol run, and a bind is a \
     second base symbol (board #1128)";

/// The refusal this lane's own grid earned. Named so the census key and the test
/// cannot drift apart.
pub const REFUSED_EMPTY_POOL: &str =
    "a store-run-before-a-call whose run materialises nothing and stores at most \
     one formal: the copy's slot rule (board #867) is negative or off by one \
     there, and the correcting clause would rest on one structural cell";

/// **Where `mr rSaved,r3` goes**, as a count of STORES emitted before it.
///
/// Board #867's rule, unchanged in FORM, with its domain stated rather than
/// clamped. `None` is a refusal, not a zero — see the module doc: the two cells
/// outside the domain are the ones the formula gets *wrong*, and clamping a
/// wrong answer to a plausible one is how a wrong-bytes emit looks from the
/// inside.
///
/// # `u_lead` is board #584's LEADING RUN and NOT the count — board #1212
///
/// The parameter's name is load-bearing. It is
/// [`order::leading_unproduced`]'s answer: the leading run of unproduced stores
/// **in the final order**, capped at [`order::HEAD_SLOTS_MAX`]. It used to be
/// the plain count, the module doc argued the two are one number, and that
/// argument is false on a multi-symbol run.
///
/// **The domain refusal is unaffected by the correction, and that is a fact
/// rather than a hope**: at `nprod == 0` every store is unproduced, so no
/// produced store can precede one, so the leading run equals `min(count, 2)`
/// and [`REFUSED_EMPTY_POOL`] refuses exactly the same bodies under both
/// readings. `the_empty_pool_refusal_is_the_same_under_both_readings`
/// enumerates it.
pub fn save_slot(nprod: usize, u_lead: usize) -> Option<usize> {
    if nprod == 0 && u_lead < 2 {
        return None;
    }
    // `nprod >= 1`, or `nprod == 0` with `u_lead >= 2` where `min(u,2) == 2 >= 1`.
    // Both make the expression non-negative, which is why it is computed in
    // `usize` only after the guard above rather than in `isize` with a clamp.
    Some(nprod + u_lead.min(2) - 1)
}

/// The first call's setup for a #844 composition: **the whole store run, with
/// the callee-saved copy spliced in.**
///
/// Returned as `setups[0]` of a [`super::Selected::Seq`], so
/// [`super::calls::call_seq_text`] finishes the body with the frame it already
/// owns and the `bl` whose displacement it already encodes from the function's
/// `.text` offset. That reuse is the point: this composition introduces **no new
/// obj shape**, so `.pdata`, the `$M`/`$T` label stride, the REL24 site and the
/// `/Gy` COMDAT association are all the framed sequence's, already graded.
pub fn store_run_prefix_text(
    params: &[u32],
    prefix: &c2_il::StoreRunPrefix,
    saved_reg: u8,
) -> Result<Vec<u8>, BackendError> {
    Ok(ops_to_bytes(&store_run_prefix_ops(params, prefix, saved_reg)?))
}

/// **S1c (i): [`store_run_prefix_text`] as an op stream**, reachable by a
/// caller — and the composition seam is now an op-level seam end to end.
///
/// It moves in the same commit as `leaf::store` because it is the *other*
/// consumer of [`ScheduledRun::slots`](super::leaf::store::ScheduledRun), which
/// exists in slot form precisely so the run is scheduled once and the two
/// consumers differ only in what they do with it. A slots field that one
/// consumer read as ops while the other read as bytes would be that seam split
/// in half.
///
/// **The splice's own argument is unchanged and is now enforced by the type.**
/// *"a producer is one slot, so the copy can never split a `lis`/`ori` pair"* —
/// the loop below places `mr rSaved,r3` at a slot boundary, and a slot boundary
/// was always an instruction boundary. It is one now in the representation too.
pub fn store_run_prefix_ops(
    params: &[u32],
    prefix: &c2_il::StoreRunPrefix,
    saved_reg: u8,
) -> Result<Ops, BackendError> {
    // **THE TRANSFER GATE — board #866 is not true in general, and this is where
    // that costs.** See [`LIVE_ARG_STORED`] and the module doc: a store whose
    // value is a formal the call keeps alive is scheduled differently in the
    // framed body than in the leaf, and the difference is a SWAP of two stores
    // — an obj that links, with two right words in the wrong order.
    //
    // Asked BEFORE the scheduler, because the scheduler's answer is the leaf's
    // and the leaf's answer is the wrong one here. The receiver (slot 0) is
    // exempt: `this` is the store base, it is copied to r31 regardless, and
    // storing it transfers on every measured cell.
    let live: Vec<u32> = params
        .iter()
        .take(prefix.live_args)
        .skip(1)
        .copied()
        .collect();
    if prefix
        .ops
        .iter()
        .any(|o| matches!(o, c2_il::IlOp::Load(t) if live.contains(t)))
    {
        return Err(out_of_class(LIVE_ARG_STORED));
    }
    // **THE REFUSAL THAT USED TO STAND HERE IS LIFTED, AND WHAT REPLACED IT IS
    // ONE ARGUMENT — see [`BIND_IN_A_COMPOSITION_WAS`] and the module doc.**
    // `w-carrier` refused every `BoundAddr`-carrying composition rather than
    // change the `u` fed to [`save_slot`], because the change governs every #844
    // body and at the time it would have rested on the four cells that refuted
    // that lane. It rests on GRID R now: 93 cells with an observed copy, 30 of
    // them separating the two readings, and the leading run is 93/93 where the
    // count is 63/93.
    //
    // There is deliberately **no `BoundAddr` clause left in this function**. A
    // second gate keyed on the carrier would be a gate that refuses nothing once
    // the reader admits the family (board #1175), and the one thing a bind
    // actually changes — that the run has a second base symbol, so #584's two
    // readings of `u` part company — is answered where `u` is computed and not
    // by asking whether an op is a bind.
    let run_ops = &prefix.ops;
    // **The same scheduler the leaf asks, not a second one.** `None` here means
    // the stream is not a value-simple GPR run at all — an `AddrOf` value (board
    // #1160's F2 four-op group), an FP group, a load-valued group — and it is a
    // REFUSAL, never a fall-through to `store_leaf_text`'s source-order walk.
    // Falling through would emit a run c2 schedules differently *and* drop the
    // `bl`, which is both halves of board #232 at once.
    let Some(run) = scheduled_gpr_run(params, run_ops) else {
        return Err(out_of_class(
            "a store run before a call whose values are not all a formal or a \
             literal: the schedule and the allocation are fitted on that \
             vocabulary and neither is asked outside it",
        ));
    };
    let run = run?;

    // **A MULTI-WORD LITERAL IS REFUSED HERE EVEN WHEN IT IS ALONE**, which is
    // one level stricter than the leaf's own rule — and it is measured, not
    // cautious. `scheduled_gpr_run` refuses a wide literal only *beside another
    // producer*, because c2 interleaves the halves of two wide loads; a run whose
    // ONLY producer is wide is one live range with nothing to interleave with and
    // is in class as a leaf. In a framed body the `mr r31,r3` is the second thing
    // that can land between the halves, and it does:
    //
    // ```text
    //   WL::WL(unsigned a, unsigned b) { m1 = 70000; m2 = b; p0 = this; Alloc(a); }
    //     lis 11,1 ; stw 5,8(3) ; stw 3,0(3) ; mr 31,3 ; ori 11,11,4464 ; stw 11,4(3)
    // ```
    //
    // The splice below places the copy BETWEEN slots and a producer is one slot,
    // so it can never split a pair — it would emit `lis ; ori` contiguous and be
    // two right words in the wrong place. `fits_i16` is `emit_load_imm_ops`'s
    // own predicate, shared rather than restated, exactly as the leaf shares it.
    if run_ops
        .iter()
        .any(|o| matches!(o, c2_il::IlOp::Lit(k) if !super::select::fits_i16(*k)))
    {
        return Err(out_of_class(
            "a store run before a call with a multi-word literal: the `mr r31,r3` \
             lands BETWEEN the `lis` and the `ori`, and a producer is one slot to \
             the splice",
        ));
    }

    let Some(slot) = save_slot(run.nprod, run.u_lead) else {
        return Err(out_of_class(REFUSED_EMPTY_POOL));
    };
    // A slot past the end of the run is not reachable from `save_slot`'s own
    // arithmetic for any run this file accepts — `nprod + min(u,2) - 1` is at
    // most the store count whenever there is at least one store — but a
    // refusal beats a silent append at the wrong end if a later widening
    // changes either input.
    let nstores = run.slots.iter().filter(|(is_store, _)| *is_store).count();
    if slot > nstores {
        return Err(out_of_class(
            "the callee-saved copy's slot lands past the end of the store run",
        ));
    }

    let mut text: Ops = Vec::with_capacity(run.slots.len() + 1);
    let mut placed = false;
    let mut seen = 0usize;
    for (is_store, words) in &run.slots {
        // **Before the store at index `slot`**, so a producer that sits at the
        // same boundary is emitted first — which is what the bytes say: every
        // `PM…` cell in GRID S (`sa_pL1_w0_c0`: `li r11,0 ; mr r31,r3 ;
        // stw r11,20(r3)`) has the producer ahead of the copy at slot 0.
        if *is_store && seen == slot && !placed {
            text.push(mop_mr(saved_reg, super::select::RET_REG));
            placed = true;
        }
        if *is_store {
            seen += 1;
        }
        text.extend(words.iter().copied());
    }
    if !placed {
        // `slot == nstores`: the copy trails the whole run. Reached by every
        // `nprod - 1 + min(u,2) == nstores` cell, e.g. GRID S's `sa_p0_w1_*`
        // family's neighbours; emitted here rather than special-cased above so
        // the loop has one placement rule.
        text.push(mop_mr(saved_reg, super::select::RET_REG));
    }
    Ok(text)
}

/// The **backstops** the parser's own gate is restated by, so that
/// `IlBundle::function_census` and the emitter cannot disagree about the class
/// silently (`docs/GAPS.md` §6 #9's shape).
///
/// Each is a fact `c2_il`'s `try_parse_store_run_call` already requires. A body
/// reaching here that fails one of them means a parser widened past its
/// witness, and the honest answer is a gap and not a guess.
pub fn gate_composition(seq: &c2_il::CallSeq) -> Result<(), BackendError> {
    if seq.calls.len() != 1 {
        return Err(out_of_class(
            "a store run before a call SEQUENCE of more than one call: the copy's \
             slot rule is measured on one call and the run-after-the-call \
             question (board #872) has no rule fitted at all",
        ));
    }
    if !matches!(seq.tail, c2_il::SeqTail::SavedFormal { param: 0 }) {
        // Board #869/#1131: only the constructor frames. The `void`,
        // `return <call>` and discarded-`int` forms are frame words **0** and
        // tail-call *behind* the run — three of the four cells that look like
        // this shape are a different body, and `work/w-seam2/grid/sr_f*` grades
        // all three.
        return Err(out_of_class(
            "a store run before a call whose body does not return the saved \
             receiver: the other three forms are frame words 0 and tail-call \
             behind the run (boards #869/#1131)",
        ));
    }
    if seq.saved != [0] {
        return Err(out_of_class(
            "a store run before a call with a saved set other than the receiver \
             alone: nothing else is measured live across this call",
        ));
    }
    if seq.guard.is_some() || !seq.early.is_empty() {
        return Err(out_of_class(
            "a store run before a GUARDED call, or beside a guarded early \
             return: two block plans and one run, and the interleaving is ungraded",
        ));
    }
    let c = &seq.calls[0];
    if !c.arg_ops.is_empty() || c.arg_slots.is_some() || c.link_args.is_some() {
        // Board #1129, the regime boundary, restated on the emitter's side. A
        // setup that writes `r3` destroys `this`, the store base switches
        // `r3 -> r31` mid-run and the setup interleaves into it — `w-seam2`'s
        // `sr_c1r3` is that cell and it is a different body, not a longer one.
        return Err(out_of_class(
            "a store run before a call with a non-empty argument setup: a setup \
             that writes r3 switches the store base mid-run (boards #870/#1129)",
        ));
    }
    Ok(())
}

/// **The composition's own refusal for a function that spells the run twice.**
///
/// See [`c2_il::IlFunction::store_run_carried_twice`]. Refusing is the whole
/// point: the alternative to refusing is picking a winner between `ops` and the
/// carrier, and picking a winner is precisely board #232's defect.
pub fn gate_carrier(func: &IlFunction) -> Result<(), BackendError> {
    if func.store_run_carried_twice() {
        return Err(out_of_class(
            "a function carrying a store run in BOTH `ops` and `CallSeq::store_run`: \
             the two are alternatives in the selector's dispatch and one would be \
             silently dropped (board #232, board #844)",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **#867's rule, scored against the two grids' own numbers.** The table is
    /// `(nprod, u, observed)` transcribed from `work/w-seam2/mrslot_grid.txt` and
    /// `mrslot_grid2.txt`, which are read off real `c2.dll` objs at the
    /// workload's `/GR /O1 /Oi /EHsc` — not derived from the formula this test
    /// checks.
    #[test]
    fn the_save_slot_rule_reproduces_every_graded_cell_in_its_domain() {
        // GRID S (the frozen fit set) — the `nprod >= 1` rows.
        let grid_s = [
            (1usize, 0usize, 0usize), // sa_pL1_w0 / sa_pZ1_w0 / sa_pLu2_w0 / sa_pZ2_w0
            (1, 1, 1),                // sa_pL1_w1 / sa_pZ1_w1 / sa_pLu2_w1 / sa_pZ2_w1
            (1, 3, 2),                // sa_pL1_w3 / sa_pZ1_w3 / sa_pLu2_w3 / sa_pZ2_w3
            (2, 0, 1),                // sa_pL2_w0
            (2, 1, 2),                // sa_pL2_w1
            (2, 3, 3),                // sa_pL2_w3
        ];
        // GRID S2 (the declared post-hoc holdout) — levels GRID S holds fixed:
        // `nprod = 3` throughout, and `u = 2, 4, 5, 6` at every `nprod`.
        let grid_s2 = [
            (0usize, 2usize, 1usize), // h_np0_u2
            (0, 4, 1),                // h_np0_u4
            (0, 5, 1),                // h_np0_u5
            (0, 6, 1),                // h_np0_u6
            (1, 2, 2),                // h_np1_u2
            (1, 4, 2),                // h_np1_u4
            (1, 6, 2),                // h_np1_u6
            (2, 2, 3),                // h_np2_u2
            (2, 4, 3),                // h_np2_u4
            (3, 0, 2),                // h_np3_u0
            (3, 1, 3),                // h_np3_u1
            (3, 2, 4),                // h_np3_u2
            (3, 3, 4),                // h_np3_u3
            (3, 4, 4),                // h_np3_u4
            (1, 3, 2),                // h_ordmix   — a produced store written FIRST
            (2, 3, 3),                // h_ordmix2  — the same with two producers
        ];
        for (nprod, u, want) in grid_s.iter().chain(grid_s2.iter()) {
            assert_eq!(
                save_slot(*nprod, *u),
                Some(*want),
                "nprod={nprod} u={u}: the slot rule disagrees with a graded obj"
            );
        }
    }

    /// **The domain boundary, pinned WITH the counterexample.**
    ///
    /// `nprod == 0, u <= 1` is refused, and the reason is not caution: at
    /// `(0, 1)` board #867's formula answers **0** and real `c2` emits the copy
    /// after the store, i.e. **1**. `work/w-seam2/grid/sa_p0_w1_c0/dis.txt`:
    ///
    /// ```text
    ///   stw r5,16(r3) ; mr r31,r3 ; bl ?Alloc@H@@QAAPAUBE@@I@Z ; mr r3,r31
    /// ```
    ///
    /// A lane that clamps the formula at zero emits `mr r31,r3 ; stw r5,16(r3)`
    /// — two right words in the wrong order, an obj that still links. That is
    /// the shape of every one of the six refuted allocation keys, so the clause
    /// that would fix it is **named and not taken**: it would rest on one
    /// structural cell (`u = 1`), and `u = 0` is a run of length 0 the reader
    /// files under a different production entirely.
    /// **BOARD #1212 CLOSED IN THIS FILE — the refusal's own counterexample,
    /// asserted from the other side.** The test kept its shape and flipped its
    /// verdict, so the two states of this seam are one diff apart in the record.
    ///
    /// `w-carrier`'s first emitter composed a bind-carrying run through this
    /// seam. Four cases of `scripts/sweep.d/88-store-run-call.py` and 56 cross
    /// cells graded `Port=Mismatch`, all of them the same word in the wrong
    /// place, and that lane's own 53-cell frozen grid was green through every one
    /// (board #1211). Real `c2.dll` at the workload's own flags,
    /// `work/w-carrier/bisect/s1427.cpp`:
    ///
    /// ```text
    ///   H::H(unsigned a, unsigned b) { BE& lh = mListHead; mCount = 0;
    ///                                  lh.mNext = (BE*)this; Reset(); }
    ///   c2:        li 11,0 ; mr 31,3 ; stw 11,20(3) ; stw 3,8(3) ; bl
    ///   the port:  li 11,0 ; stw 11,20(3) ; mr 31,3 ; stw 3,8(3) ; bl   WRONG
    /// ```
    ///
    /// The copy belongs after **zero** stores. [`save_slot`] answered **one**
    /// because it was fed the COUNT of unproduced stores where the run needs
    /// #584's LEADING RUN — equal only on a single-symbol run, and a bind is a
    /// second symbol (board #1128).
    ///
    /// Both numbers stay asserted here: the emitter produces the leading-run
    /// answer now, and the count's answer is written down beside it so a future
    /// reader meets the thing that was wrong and not only the thing that is
    /// right. `w-mrslot` graded the swap over GRID R — 93 cells with an observed
    /// copy, 30 of them separating the two readings, **93 HIT / 0 MISS** against
    /// the count's 63 / 30.
    #[test]
    fn a_bind_carrying_run_emits_the_leading_run_slot_not_the_count() {
        let (this, init, size) = (0x0101u32, 0x0201u32, 0x0301u32);
        let l = 0xFB09u32;
        let prefix = c2_il::StoreRunPrefix {
            ops: vec![
                c2_il::IlOp::Load(this),
                c2_il::IlOp::Lit(0),
                c2_il::IlOp::StoreInd { off: 20, width: 4 },
                c2_il::IlOp::BoundAddr { tok: l, base: this, off: 8 },
                c2_il::IlOp::Load(this),
                c2_il::IlOp::StoreInd { off: 0, width: 4 },
            ],
            live_args: 1,
        };
        let text = store_run_prefix_text(&[this, init, size], &prefix, 31)
            .expect("s1427's shape is in class since board #1212");
        // c2's own words, in order: `li 11,0`, `mr 31,3`, `stw 11,20(3)`,
        // `stw 3,8(3)`. The copy is the SECOND word — after ZERO stores — and
        // `save_slot` fed the count would have put it third.
        assert_eq!(
            &text[4..8],
            &crate::codegen::encode::encode_mr(31, 3)[..],
            "the copy lands after ZERO stores"
        );
        // What the two readings say about this exact run, kept beside the
        // emitter so the correction cannot be silently reverted.
        assert_eq!(save_slot(1, 1), Some(1), "the COUNT reading says 1 — refuted");
        assert_eq!(save_slot(1, 0), Some(0), "the LEADING-RUN reading says 0 — c2");
        // The same run WITHOUT the bind is single-symbol, both readings agree
        // there, and it is untouched by the correction.
        let plain = c2_il::StoreRunPrefix {
            ops: vec![
                c2_il::IlOp::Load(this),
                c2_il::IlOp::Lit(0),
                c2_il::IlOp::StoreInd { off: 20, width: 4 },
                c2_il::IlOp::Load(this),
                c2_il::IlOp::Load(this),
                c2_il::IlOp::StoreInd { off: 8, width: 4 },
            ],
            live_args: 1,
        };
        assert!(store_run_prefix_text(&[this, init, size], &plain, 31).is_ok());
    }

    /// **BOARD #844's COMPOSITION WITH AN INTERIOR-ADDRESS PRODUCER — the whole
    /// prefix, word for word, in both spellings.** The rung this test belongs to
    /// widened `parse_simple_gpr_run`; this is the half of it that only the
    /// composition can show, because the `mr r31,r3` splice lands BETWEEN slots
    /// and a producer is one slot to it.
    ///
    /// Every word is read off real `c2.dll`'s own obj at the WORKLOAD's
    /// `/GR /O1 /Oi /EHsc` — `work/w-midrun/grid/m_bc_u1_f1_af/dis.txt` and
    /// `m_dc_u1_f1_af/dis.txt`, one directory per cell. The bodies are
    /// `H::H(unsigned p, unsigned q) { … Grab(p); }` with `mBlk` at 20 and `mA`
    /// at 16, so `this`/`p`/`q` are r3/r4/r5.
    ///
    /// **The two spellings put the copy in DIFFERENT slots**, and neither is a
    /// special case: the bind is a second base symbol, so the final store order
    /// differs, so `order::leading_unproduced` differs, so `save_slot` differs.
    /// One rule, two answers, both c2's.
    #[test]
    fn the_composition_emits_the_interior_address_in_both_spellings() {
        let (this, p, q) = (0x0101u32, 0x0201u32, 0x0301u32);
        let l = 0xFB09u32;
        let params = [this, p, q];

        // `m_bc_u1_f1_af` — BIND. `addi 11,3,20 ; mr 31,3 ; stw 11,20(3) ;
        // stw 5,16(3)`: two symbols, the produced store leads, leading run 0.
        let bind = c2_il::IlOp::BoundAddr { tok: l, base: this, off: 20 };
        let bound = c2_il::StoreRunPrefix {
            ops: vec![
                bind,
                bind,
                c2_il::IlOp::StoreInd { off: 0, width: 4 },
                c2_il::IlOp::Load(this),
                c2_il::IlOp::Load(q),
                c2_il::IlOp::StoreInd { off: 16, width: 4 },
            ],
            live_args: 2,
        };
        assert_eq!(
            store_run_prefix_text(&params, &bound, 31).expect("in class"),
            vec![
                0x39, 0x63, 0x00, 0x14, // addi r11,r3,20
                0x7C, 0x7F, 0x1B, 0x78, // mr   r31,r3
                0x91, 0x63, 0x00, 0x14, // stw  r11,20(r3)
                0x90, 0xA3, 0x00, 0x10, // stw  r5,16(r3)
            ],
            "m_bc_u1_f1_af"
        );

        // `m_dc_u1_f1_af` — DIRECT, the four-op group. `addi 11,3,20 ;
        // stw 5,16(3) ; mr 31,3 ; stw 11,20(3)`: ONE symbol, the unproduced
        // store leads, leading run 1, and the copy moves a slot.
        let direct = c2_il::StoreRunPrefix {
            ops: vec![
                c2_il::IlOp::Load(this),
                c2_il::IlOp::Load(this),
                c2_il::IlOp::AddrOf { off: 20 },
                c2_il::IlOp::StoreInd { off: 20, width: 4 },
                c2_il::IlOp::Load(this),
                c2_il::IlOp::Load(q),
                c2_il::IlOp::StoreInd { off: 16, width: 4 },
            ],
            live_args: 2,
        };
        assert_eq!(
            store_run_prefix_text(&params, &direct, 31).expect("in class"),
            vec![
                0x39, 0x63, 0x00, 0x14, // addi r11,r3,20
                0x90, 0xA3, 0x00, 0x10, // stw  r5,16(r3)
                0x7C, 0x7F, 0x1B, 0x78, // mr   r31,r3
                0x91, 0x63, 0x00, 0x14, // stw  r11,20(r3)
            ],
            "m_dc_u1_f1_af"
        );

        // The mixed run — `xboxheap.cpp`'s own — is SERVED here since board
        // #1297 (lane `w-lineage`), and the TU is byte-exact against real
        // `c2.dll` at the workload's own flags. The paragraph this replaces read
        // *"still REFUSED here, and it is peer lane `w-mixkind`'s rung rather
        // than an oversight of this one"*; `w-mixkind` measured the ladder and
        // declined the rule, GRID L refuted five keys at once, and what shipped
        // is a REFUSAL BOUNDARY — the mix is served only where the `d` bonus is
        // provably zero **and** the address's stores go through the bind naming
        // it, so `docs/SYMBOL.md`'s cross-symbol pin fixes the order.
        let mixed = c2_il::StoreRunPrefix {
            ops: vec![
                bind,
                bind,
                c2_il::IlOp::StoreInd { off: 0, width: 4 },
                c2_il::IlOp::Load(this),
                c2_il::IlOp::Lit(0),
                c2_il::IlOp::StoreInd { off: 16, width: 4 },
            ],
            live_args: 2,
        };
        assert_eq!(
            store_run_prefix_text(&params, &mixed, 31).expect("served since #1297"),
            vec![
                0x39, 0x63, 0x00, 0x14, // addi r11,r3,20    the ADDRESS
                0x39, 0x40, 0x00, 0x00, // li   r10,0        the LITERAL
                0x91, 0x63, 0x00, 0x14, // stw  r11,20(r3)
                0x7C, 0x7F, 0x1B, 0x78, // mr   r31,r3
                0x91, 0x43, 0x00, 0x10, // stw  r10,16(r3)
            ]
        );
    }

    /// **THE CORRECTION DOES NOT MOVE THE DOMAIN REFUSAL, and it is enumerated
    /// rather than argued.** At `nprod == 0` every store is unproduced, so no
    /// produced store can precede one, so the leading run equals
    /// `min(count, HEAD_SLOTS_MAX)` and [`REFUSED_EMPTY_POOL`] refuses exactly
    /// the bodies it refused before.
    ///
    /// Registered in `work/w-mrslot/PREREG.md` §0.1 as a checkable consequence
    /// **before the lane's first probe**, with the failure condition written
    /// down: if the swap moved this boundary the widening would be larger than
    /// the lane claimed.
    #[test]
    fn the_empty_pool_refusal_is_the_same_under_both_readings() {
        for total in 0..8usize {
            let lead = total.min(crate::codegen::order::HEAD_SLOTS_MAX);
            assert_eq!(
                save_slot(0, total).is_none(),
                save_slot(0, lead).is_none(),
                "the empty-pool refusal moved at total = {total}"
            );
        }
    }

    #[test]
    fn the_mr_slot_domain_boundary_is_refused_with_its_counterexample() {
        assert_eq!(save_slot(0, 0), None);
        assert_eq!(save_slot(0, 1), None, "the clamped formula answers 0; c2 emits 1");
        // and it is refused ONLY there — the neighbours in both directions are
        // in the domain and are the graded values.
        assert_eq!(save_slot(0, 2), Some(1));
        assert_eq!(save_slot(1, 0), Some(0));
        assert_eq!(save_slot(1, 1), Some(1));
    }
}
