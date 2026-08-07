//! **GRID-L — mechanism E through a refused LOOP**, against real `c2`.
//!
//! Board **#980**'s residue. Lane `w-inl0` read the dead-temporary call and
//! closed 138 of the 370; the 232 that remain on master `217d4a85` all stop at
//! the *next* level down, which `fnbyte-blr-stop2` prices at **228** under the
//! production `return-scope-close-cflow-label`. That production is STLport's
//! `__destroy_range_aux(_first, _last, __false_type)` — the overload a **class**
//! element type takes — and it is a **loop**.
//!
//! `crates/c2-il/src/func/body/shapes/no_effect.rs::no_effect_loop` reads that
//! body without accepting it, and hands the loop's callee to the same
//! `c2_core::elide::Reduction::NoEffectCall` link `w-inl0` already ships.
//! **Nothing in `crates/c2-core/` changed and `parse_segment` is byte-for-byte
//! unchanged**: a body this reader recognizes is still `FnVerdict::Blocked`,
//! still `fnbyte-refused`, and `IlBundle::functions` still refuses its whole TU.
//!
//! # The five-level chain, and where it STOPS
//!
//! Read out of `src/lazer/meta_ham/CharacterProvider.cpp` with
//! `c2rs census --fn`, and reproduced by `l01`:
//!
//! | # | function | census key |
//! |---|---|---|
//! | 1 | `??$_Destroy_Range@PAVSymbol@@…` | **in class** — the differ |
//! | 2 | `??$__destroy_range@…` | `expr-intrinsic-memset` — read by `w-inl0` |
//! | 3 | `??$__destroy_range_aux@…` | `return-scope-close-cflow-label` — **read here** |
//! | 4 | `??$_Destroy@…` | `expr-intrinsic-memset` — read by `w-inl0` |
//! | 5 | `??$__destroy_aux@…` | `expr-lit-type-8207` — **the STOP** |
//!
//! Level 5 is `p->~T()` on a class with a trivial destructor: an `int` literal, a
//! `void` literal, a bind and a discard, with **no call in it at all**. For that
//! chain to close, level 5 must **SEED** E's fixpoint.
//!
//! > **2026-08-08 — it does, and `l09`'s assertion was INVERTED in the commit
//! > that made it so** (board **#1053**, lane `w-seed`).
//! > `c2_core::elide::Reduction::NoEffectNothing` lets a refused body seed, and
//! > `l01`/`l09` are now the positive for the whole five-level chain rather than
//! > the record of where it stopped. w-memset planted the old assertion for
//! > exactly this: it went red, alone, with the message it was given
//! > (`work/w-seed/l09_red.txt`), and the other ten tests here did not move.
//! > GRID-N (`work/w-seed/cells/`) is the grid that earns the widening.
//!
//! # What each test is FOR
//!
//! | test | the claim, and what going red means |
//! |---|---|
//! | `a_loop_over_an_empty_callee_collapses_to_one_blr` | l02 — the positive. The loop is a LINK and the chain closes through it with no change to E's rule. Also the `/Ob0` row: a caller that is a `blr` at `/O1` and a call at `/Ob0` is mechanism **I** and this reader would have to be withdrawn |
//! | `the_induction_and_the_comparison_are_not_pinned_to_one_value` | l12 — stride 8 and `<` instead of stride 4 and `!=`. Registered as the prediction most likely to lose |
//! | `a_dead_temporary_call_inside_the_loop_composes` | l08 — the workload's own level 3→4 edge: the two readers share one argument walk, not two |
//! | `a_loop_whose_callee_keeps_bytes_stops_the_chain` | l03 — the callee condition, which is board #950's hazard for this rule |
//! | `a_loop_over_an_external_callee_keeps_its_relocation` | l04 — the same-TU condition, one level inside the loop |
//! | `a_second_statement_in_the_loop_body_is_refused` | l05 — "emits nothing" is a property of the whole body |
//! | `an_impure_induction_step_is_refused` | l06 — the step must be one lvalue, one literal, one operator |
//! | `a_condition_over_a_global_is_refused` | l07 — the test must read only formals, or the body materializes data |
//! | `a_loop_that_stores_is_refused` | l10 — the reader is not "any loop with a matched label set" |
//! | `a_cycle_through_a_loop_is_never_admitted` | l11 — never seeded, so never admitted, and the closure terminates |
//! | `the_pseudo_destructor_leaf_seeds_and_the_whole_chain_closes` | l01/l09 — c2 emits one `blr` for the whole chain and **so does the port now**, level by level: the loop admitted as a LINK, the leaf as a SEED, and the leaf still `parse-refused`. Was `…_is_the_residue_and_needs_a_SEED` until board #1053 |

mod cellgrade;

use c2_harness::gap::fnbytes::FnByte;
use c2_reference::Toolchain;

// **The cell-grading half lives in `cellgrade`** — the ANCHOR, the TAIL PAD, the
// flag profile, `grade_cell`, `row` and `work`, verbatim and byte for byte what
// this file used to carry. Lane `w-seed` (board #1053) needed the identical
// helper for GRID-N, and `w-relo`'s merge is what a fourth private copy costs:
// two lanes wrote the same reader in different files, auto-merged with no
// conflict marker, and the duplicate walks were caught only by a compile error.
// `empty_elision.rs` and `dead_temp_elision.rs` still carry their own; see
// `cellgrade`'s module doc for why they were deliberately not migrated here.
use cellgrade::{grade_cell, row, work, BLR};

const L01: &str = include_str!("../../../work/w-memset/cells/l01.cpp");
const L02: &str = include_str!("../../../work/w-memset/cells/l02.cpp");
const L03: &str = include_str!("../../../work/w-memset/cells/l03.cpp");
const L04: &str = include_str!("../../../work/w-memset/cells/l04.cpp");
const L05: &str = include_str!("../../../work/w-memset/cells/l05.cpp");
const L06: &str = include_str!("../../../work/w-memset/cells/l06.cpp");
const L07: &str = include_str!("../../../work/w-memset/cells/l07.cpp");
const L08: &str = include_str!("../../../work/w-memset/cells/l08.cpp");
const L09: &str = include_str!("../../../work/w-memset/cells/l09.cpp");
const L10: &str = include_str!("../../../work/w-memset/cells/l10.cpp");
const L11: &str = include_str!("../../../work/w-memset/cells/l11.cpp");
const L12: &str = include_str!("../../../work/w-memset/cells/l12.cpp");


/// **l02 — THE POSITIVE.** The loop's callee is `empty_body`, so the existing
/// seed is reachable and the whole chain closes through the loop LINK alone.
///
/// The `/Ob0` half is the load-bearing one: mechanism E is not governed by `/Ob`
/// and mechanism I is, so a wrapper that is a bare `blr` at `/O1` and a call at
/// `/Ob0` would be I wearing E's clothes and this reader would have to go.
#[test]
fn a_loop_over_an_empty_callee_collapses_to_one_blr() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("l02");
    for extra in [&[] as &[&str], &["/Ob0"]] {
        let at = if extra.is_empty() { "/O1" } else { "/O1 /Ob0" };
        let (rows, tu) = grade_cell(&tc, &d, "l02", L02, extra);
        let w = row(&rows, "?destroy_range@", "l02");
        assert_eq!(
            (w.3.as_slice(), w.4),
            (BLR.as_slice(), 0),
            "l02 at {at}: c2's own body for the wrapper is not one `blr` with no \
             relocation — the premise this cell rests on has changed"
        );
        assert_eq!(
            (w.0, w.1),
            ("tail", FnByte::Exact),
            "l02 at {at}: the port must select this as a tail call and emit \
             nothing for it. A `Differs` means `no_effect_loop` stopped feeding \
             the fixpoint"
        );
        let a = row(&rows, "?aux@", "l02");
        assert_eq!(
            (a.0, a.1),
            ("parse-refused", FnByte::Refused),
            "l02 at {at}: THE LOOP PARSES NOW. This reader is decode-only by \
             construction and `IlBundle::functions` must keep refusing this TU \
             (board #971 condition 4). Accepting the body is a different rung"
        );
        assert!(
            tu.reduces_to_nothing(&a.2),
            "l02 at {at}: the fixpoint did not admit the refused LOOP `{}` — the \
             link `no_effect_loop` returns is not reaching `elide.rs`",
            a.2
        );
    }
    let _ = std::fs::remove_dir_all(&d);
}

/// **l12 — the induction and the comparison are not pinned to one value.**
/// Stride 8 and `<` where `l02` has stride 4 and `!=`.
///
/// Registered in `ADDENDUM-1.md` as the prediction most likely to lose: the
/// workload only ever shows `++`/`!=` at a stride of `sizeof(T)`, so a reader
/// that had to be widened for this cell would be one with a value smuggled into
/// its grammar (#644). It did not have to be — the stride is read and not
/// constrained, and `<` is in `LOOP_CMP_OPS` **because this cell grades it**.
#[test]
fn the_induction_and_the_comparison_are_not_pinned_to_one_value() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("l12");
    let (rows, tu) = grade_cell(&tc, &d, "l12", L12, &[]);
    let a = row(&rows, "?aux@", "l12");
    assert!(
        tu.reduces_to_nothing(&a.2),
        "l12: a stride of 8 and a `<` test are not read, so the reader is keyed \
         on `l02`'s literal `4` or on its `!=` opcode — which is a value in a \
         grammar's clothing (#644)"
    );
    let w = row(&rows, "?destroy_range@", "l12");
    assert_eq!(
        (w.3.as_slice(), w.4),
        (BLR.as_slice(), 0),
        "l12: c2's body for the wrapper is not one `blr` with no relocation"
    );
    assert_eq!((w.0, w.1), ("tail", FnByte::Exact), "l12: the chain must close");
    let _ = std::fs::remove_dir_all(&d);
}

/// **l08 — the two readers COMPOSE.** The loop's single statement is not a plain
/// call but the tag-dispatch call `w-inl0` already reads, which is the workload's
/// own level 3 → level 4 edge. The argument vocabulary is shared
/// (`eat_no_effect_call_stmt`), not duplicated: `w-relo`'s merge is the reason
/// that matters — two lanes wrote the same reader in different files and
/// auto-merged into duplicate walks with no conflict marker.
#[test]
fn a_dead_temporary_call_inside_the_loop_composes() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("l08");
    let (rows, tu) = grade_cell(&tc, &d, "l08", L08, &[]);
    let a = row(&rows, "?aux@", "l08");
    assert!(
        tu.reduces_to_nothing(&a.2),
        "l08: the loop's statement is a dead-temporary call and the reader did \
         not walk it — the two shapes are not sharing one argument vocabulary"
    );
    let one = row(&rows, "?destroy_one@", "l08");
    assert!(
        tu.reduces_to_nothing(&one.2),
        "l08: the dead-temporary link below the loop is not admitted"
    );
    let w = row(&rows, "?destroy_range@", "l08");
    assert_eq!(
        (w.3.as_slice(), w.4),
        (BLR.as_slice(), 0),
        "l08: c2's body for the wrapper is not one `blr` with no relocation"
    );
    assert_eq!(
        (w.0, w.1),
        ("tail", FnByte::Exact),
        "l08: the chain must close through BOTH readers"
    );
    let _ = std::fs::remove_dir_all(&d);
}

/// **l03 — THE CALLEE CONDITION.** Give the loop's leaf a store and nothing may
/// be elided. This is board #950's hazard for this rule: the answer is keyed on
/// the callee's decoded IL, never on "no relocation appeared".
#[test]
fn a_loop_whose_callee_keeps_bytes_stops_the_chain() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("l03");
    let (rows, tu) = grade_cell(&tc, &d, "l03", L03, &[]);
    let a = row(&rows, "?aux@", "l03");
    assert!(
        !tu.reduces_to_nothing(&a.2),
        "THE CALLEE CONDITION WAS DROPPED: the loop's leaf stores to a global and \
         the fixpoint admitted `{}` anyway. Every caller of it would be emitted \
         as `blr` against a c2 body that is not one",
        a.2
    );
    let w = row(&rows, "?destroy_range@", "l03");
    assert!(
        matches!(w.1, FnByte::Differs { .. }) || w.1 == FnByte::Refused,
        "l03: the wrapper came back {:?}. Its chain does something, so an `Exact` \
         here would mean the port emitted nothing for a body that stores",
        w.1
    );
    let _ = std::fs::remove_dir_all(&d);
}

/// **l04 — CONTROL, the same-TU condition one level inside the loop.** The
/// loop's callee is external, so no definition in this bundle can answer for it
/// and c2 keeps a relocation in the chain.
#[test]
fn a_loop_over_an_external_callee_keeps_its_relocation() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("l04");
    let (rows, tu) = grade_cell(&tc, &d, "l04", L04, &[]);
    let a = row(&rows, "?aux@", "l04");
    assert!(
        !tu.reduces_to_nothing(&a.2),
        "THE SAME-TU CONDITION WAS DROPPED: `{}` calls a function this TU does \
         not define and the fixpoint admitted it",
        a.2
    );
    let w = row(&rows, "?destroy_range@", "l04");
    assert!(
        w.4 > 0 || matches!(w.1, FnByte::Differs { .. }) || w.1 == FnByte::Refused,
        "l04: c2's wrapper carries {} relocations and grades {:?} — the cell is \
         supposed to keep a call to `?ext_leaf` somewhere in the chain, and if it \
         does not, this control cannot fire",
        w.4,
        w.1
    );
    let _ = std::fs::remove_dir_all(&d);
}

/// **l05 — the walk is TOTAL.** A second statement in the loop body and the
/// reader must decline: "emits nothing" is a property of the whole segment.
#[test]
fn a_second_statement_in_the_loop_body_is_refused() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("l05");
    let (rows, tu) = grade_cell(&tc, &d, "l05", L05, &[]);
    let a = row(&rows, "?aux@", "l05");
    assert!(
        !tu.reduces_to_nothing(&a.2),
        "THE TOTALITY OF THE WALK WAS DROPPED: the loop body also stores to a \
         global and `{}` was still read as emitting nothing",
        a.2
    );
    let _ = std::fs::remove_dir_all(&d);
}

/// **l06 — the induction step must be PURE.** `f = advance(f)` puts a call in the
/// increment, so the step is no longer one lvalue, one literal and one operator.
#[test]
fn an_impure_induction_step_is_refused() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("l06");
    let (rows, tu) = grade_cell(&tc, &d, "l06", L06, &[]);
    let a = row(&rows, "?aux@", "l06");
    assert!(
        !tu.reduces_to_nothing(&a.2),
        "THE INDUCTION STEP'S PURITY WAS DROPPED: `{}` advances through a CALL \
         and was still read as emitting nothing",
        a.2
    );
    let _ = std::fs::remove_dir_all(&d);
}

/// **l07 — the exit test must read only FORMALS.** A condition over a global is
/// a body that materializes a data symbol, which is `elide.rs`'s condition 3 one
/// level down (`w-fix`'s `k16`).
#[test]
fn a_condition_over_a_global_is_refused() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("l07");
    let (rows, tu) = grade_cell(&tc, &d, "l07", L07, &[]);
    let a = row(&rows, "?aux@", "l07");
    assert!(
        !tu.reduces_to_nothing(&a.2),
        "THE FORMALS TEST WAS DROPPED: `{}` compares against a GLOBAL and was \
         still read as emitting nothing — the body materializes a data symbol",
        a.2
    );
    let _ = std::fs::remove_dir_all(&d);
}

/// **l10 — CONTROL, a loop that EMITS.** The same skeleton with a store as its
/// body and no call at all. The reader is not "any loop with a matched label
/// set".
#[test]
fn a_loop_that_stores_is_refused() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("l10");
    let (rows, tu) = grade_cell(&tc, &d, "l10", L10, &[]);
    let a = row(&rows, "?aux@", "l10");
    assert_ne!(
        a.3, BLR,
        "l10: c2's own body for the storing loop IS a bare `blr`, so the cell no \
         longer tests what it is named for"
    );
    assert!(
        !tu.reduces_to_nothing(&a.2),
        "THE READER TOOK A LOOP THAT EMITS: `{}` writes through its induction \
         variable and the fixpoint admitted it as emitting nothing",
        a.2
    );
    let _ = std::fs::remove_dir_all(&d);
}

/// **l11 — THE CYCLE.** `aux` calls `dr2` and `dr2` calls `aux`. Nothing seeds
/// it, so the least fixpoint admits neither member; and it terminates, which is
/// the property this cell exists for (`w-fix` §3.1's round ceiling).
#[test]
fn a_cycle_through_a_loop_is_never_admitted() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("l11");
    let (rows, tu) = grade_cell(&tc, &d, "l11", L11, &[]);
    assert!(
        !tu.overflowed(),
        "l11: THE ROUND CEILING FIRED. A cycle made the closure non-monotone and \
         the context now admits nothing at all"
    );
    let a = row(&rows, "?aux@", "l11");
    let c = row(&rows, "?dr2@", "l11");
    assert!(
        !tu.reduces_to_nothing(&a.2) && !tu.reduces_to_nothing(&c.2),
        "A CYCLE WAS TREATED AS REDUCING TO NOTHING: `{}` and `{}` call each \
         other and neither is seeded",
        a.2,
        c.2
    );
    let _ = std::fs::remove_dir_all(&d);
}

/// **l01 and l09 — THE RESIDUE, CLOSED.** Board **#1053**, lane `w-seed`.
///
/// > This assertion was **inverted on 2026-08-08, and its going red was the
/// > intended signal.** `w-memset` wrote it to assert the residue *"precisely so
/// > that widening the seed set turns it red in the same commit"* (its §5), and
/// > it did: `l01 at /O1: the wrapper came back Exact. THE SEED EXISTS NOW`.
/// > The run is kept at `work/w-seed/l09_red.txt` — 10 passed, 1 failed, and the
/// > one that failed is this one. Nothing else in GRID-L moved.
///
/// Both cells are the workload's own chain for a **class** element type, five
/// levels deep, and c2 emits one `4e800020` with no relocation for the wrapper at
/// `/O1` **and** at `/Ob0` — mechanism E and not I, exactly as `w-inl0`'s `m06`
/// found. What has changed is the port: the chain bottoms out at `p->~T()` — an
/// `int` literal, a `void` literal, a bind and a discard, with **no call in it** —
/// and `c2_core::elide::Reduction::NoEffectNothing` lets that body **SEED** the
/// fixpoint instead of contributing nothing at all.
///
/// **What this test asserts now is the whole chain, level by level**, because
/// "the wrapper is `Exact`" alone would also be true if the port had elided it
/// for some entirely different reason. The loop is admitted as a LINK, the leaf
/// as a SEED, and the wrapper's own body against the judge's.
#[test]
fn the_pseudo_destructor_leaf_seeds_and_the_whole_chain_closes() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("l09");
    for (name, body) in [("l01", L01), ("l09", L09)] {
        for extra in [&[] as &[&str], &["/Ob0"]] {
            let at = if extra.is_empty() { "/O1" } else { "/O1 /Ob0" };
            let (rows, tu) = grade_cell(&tc, &d, name, body, extra);
            let w = row(&rows, "??$destroy_range@", name);
            assert_eq!(
                (w.3.as_slice(), w.4),
                (BLR.as_slice(), 0),
                "{name} at {at}: c2's body for the wrapper is not one `blr` with \
                 no relocation. If this changed the residue is mechanism I after \
                 all and the whole rung is priced wrong"
            );
            assert_eq!(
                (w.0, w.1),
                ("tail", FnByte::Exact),
                "{name} at {at}: the wrapper is {:?}. THE SEED STOPPED REACHING \
                 THE TOP — `no_effect_nothing` no longer feeds `elide.rs`, or the \
                 loop link above it stopped composing with it. Board #1053",
                w.1
            );
            // The chain, level by level, so that a wrapper which is `Exact` for
            // some other reason cannot pass as this rule working.
            let a = row(&rows, "??$aux@", name);
            assert!(
                tu.reduces_to_nothing(&a.2),
                "{name} at {at}: the LOOP `{}` is not admitted, so the wrapper's \
                 `Exact` above did not come through this chain at all",
                a.2
            );
            let leaf = row(&rows, "??$destroy_aux@", name);
            assert_eq!(
                (leaf.0, leaf.1),
                ("parse-refused", FnByte::Refused),
                "{name} at {at}: THE LEAF PARSES NOW. `no_effect_nothing` is \
                 decode-only by construction and `IlBundle::functions` must keep \
                 refusing this TU (#971 condition 4); accepting the body is a \
                 different rung"
            );
            assert!(
                tu.reduces_to_nothing(&leaf.2),
                "{name} at {at}: the leaf `{}` did not SEED. It is refused and it \
                 has no callee, so a link cannot reach it — if this is false the \
                 chain has no bottom",
                leaf.2
            );
        }
    }
    let _ = std::fs::remove_dir_all(&d);
}
