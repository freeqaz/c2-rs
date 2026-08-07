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
//! `u` has a second reading — [`order::layout_slots`]'s `u`, the **leading run**
//! of unproduced stores in the *final* order (board #584) — and the two are not
//! separated by any cell in either grid. They cannot be: `store_order` forbids a
//! store whose producer has rank `j` from occupying a position below `u + j`, so
//! the leading run is always at least `min(2, total)`, and therefore
//! `min(leading, 2) == min(total, 2)` identically. The count is used here
//! because it is the one the emitter can compute without re-deriving the order.

use c2_il::IlFunction;

use super::encode::encode_mr;
use super::leaf::store::scheduled_gpr_run;
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

/// The refusal this lane's own grid earned. Named so the census key and the test
/// cannot drift apart.
pub const REFUSED_EMPTY_POOL: &str =
    "a store-run-before-a-call whose run materialises nothing and stores at most \
     one formal: the copy's slot rule (board #867) is negative or off by one \
     there, and the correcting clause would rest on one structural cell";

/// **Where `mr rSaved,r3` goes**, as a count of STORES emitted before it.
///
/// Board #867's rule, unchanged, with its domain stated rather than clamped.
/// `None` is a refusal, not a zero — see the module doc: the two cells outside
/// the domain are the ones the formula gets *wrong*, and clamping a wrong
/// answer to a plausible one is how a wrong-bytes emit looks from the inside.
pub fn save_slot(nprod: usize, nsw: usize) -> Option<usize> {
    if nprod == 0 && nsw < 2 {
        return None;
    }
    // `nprod >= 1`, or `nprod == 0` with `nsw >= 2` where `min(nsw,2) == 2 >= 1`.
    // Both make the expression non-negative, which is why it is computed in
    // `usize` only after the guard above rather than in `isize` with a clamp.
    Some(nprod + nsw.min(2) - 1)
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
        .any(|o| matches!(o, c2_il::IlOp::Load(t) if live.contains(&t)))
    {
        return Err(out_of_class(LIVE_ARG_STORED));
    }
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
    // two right words in the wrong place. `fits_i16` is `emit_load_imm`'s own
    // predicate, shared rather than restated, exactly as the leaf shares it.
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

    let Some(slot) = save_slot(run.nprod, run.nsw) else {
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

    let mut text = Vec::with_capacity(4 * (run.slots.len() + 1));
    let mut placed = false;
    let mut seen = 0usize;
    for (is_store, words) in &run.slots {
        // **Before the store at index `slot`**, so a producer that sits at the
        // same boundary is emitted first — which is what the bytes say: every
        // `PM…` cell in GRID S (`sa_pL1_w0_c0`: `li r11,0 ; mr r31,r3 ;
        // stw r11,20(r3)`) has the producer ahead of the copy at slot 0.
        if *is_store && seen == slot && !placed {
            text.extend_from_slice(&encode_mr(saved_reg, super::select::RET_REG));
            placed = true;
        }
        if *is_store {
            seen += 1;
        }
        text.extend_from_slice(words);
    }
    if !placed {
        // `slot == nstores`: the copy trails the whole run. Reached by every
        // `nprod - 1 + min(u,2) == nstores` cell, e.g. GRID S's `sa_p0_w1_*`
        // family's neighbours; emitted here rather than special-cased above so
        // the loop has one placement rule.
        text.extend_from_slice(&encode_mr(saved_reg, super::select::RET_REG));
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
    if !c.arg_ops.is_empty() || c.arg_sources.is_some() || c.link_args.is_some() {
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
