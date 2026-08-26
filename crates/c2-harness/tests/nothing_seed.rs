//! **GRID-N — a REFUSED body that SEEDS mechanism E**, against real `c2`.
//!
//! Board **#1053**, lane `w-seed`. `w-memset` read the destroy loop, fired it
//! 4,198 more times and converted **zero**, then stopped at one line:
//! `c2_core::elide::Reduction`'s *"a refused body contributes a link and never a
//! seed"*. The chain's last level, `??$__destroy_aux@…`, is `p->~T()` on a class
//! with a trivial destructor — an `int` literal, a `void` literal, a bind and a
//! discard, **with no call in it at all** — so `NoEffectCall(&str)` has no
//! spelling for it and no chain through it could close.
//!
//! `c2_il::…::no_effect::no_effect_nothing` reads that body, decode-only, and
//! `Reduction::NoEffectNothing` **seeds** the least fixpoint with it.
//! `parse_segment` is byte-for-byte unchanged, the row stays `FnVerdict::Blocked`
//! and `fnbyte-refused`, and `IlBundle::functions` still refuses its whole TU.
//!
//! # A SEED is a strictly stronger claim than a LINK, so the conditions are RE-ASKED
//!
//! A link says *"nothing, **provided** that callee does nothing"* and the fixpoint
//! adjudicates. A seed says *"nothing, unconditionally"*, with nothing downstream
//! to catch it. Every condition `elide.rs` imposes therefore gets a cell of its
//! own here rather than being inherited from `w-empty`'s and `w-fix`'s grids:
//! same-TU (`n07`), the data symbol (`n08`), the cycle (`n06`), mechanism I
//! (`n05`), and the refusal that keeps E away from an indirect site (`n04`).
//!
//! Every cell is compiled at the workload's flags **and again at `/Ob0`**, graded
//! per **call edge** through `grade_one` — the same function the 878-TU scan runs
//! — with **the caller's whole `.text` and its relocation count printed beside
//! every verdict** (board #950: a self-branch takes no relocation, so the
//! relocation observable reads `E` on a body that is plainly not nothing).
//!
//! The sources are frozen: `work/w-seed/cells/`, `sha256` in
//! `work/w-seed/CELLS.sha256`, committed **before** this lane ran `cl.exe` once,
//! and pulled in below with `include_str!` so this file grades the frozen bytes
//! and not a transcription of them.
//!
//! # What each test is FOR
//!
//! | test | the claim, and what going red means |
//! |---|---|
//! | `a_nothing_body_seeds_and_its_caller_collapses` | n01 — THE POSITIVE. The seed exists and reaches one level. The `/Ob0` row is load-bearing: a caller that is a `blr` at `/O1` and a call at `/Ob0` is mechanism **I** and this reader would have to be withdrawn |
//! | `a_seed_propagates_up_a_chain` | n02 — the seed is a seed and not a special case at depth 1. Three links above it, every edge |
//! | `the_workload_five_level_chain_closes` | n03 — the shape the 223 conversions actually are, loop and dead temporary included. `l09` in `destroy_loop_elision.rs` is the same source with the opposite pre-#1053 assertion |
//! | `a_body_refused_for_another_reason_never_seeds` | n04 — `body-0x67`. #1056 and #921: that refusal is what keeps E safe from an INDIRECT call site, and admitting one is board #232's shape |
//! | `mechanism_i_above_a_seed_does_not_propagate` | n05 — #954's trap. Only `/Ob0` separates I from E, so the cell is graded at both |
//! | `a_cycle_of_nothing_bodies_is_never_admitted` | n06 — PREREG §0.3 compiled. A seed carries **no link**, so a cycle member cannot seed; if this goes red the termination argument is gone and the round ceiling is all that is left |
//! | `an_external_nothing_body_keeps_its_relocation` | n07 — the same-TU condition, re-asked of the seed |
//! | `a_caller_that_materializes_data_is_not_elided` | n08 — condition 3, re-asked of the seed |
//! | `a_body_that_keeps_bytes_never_seeds` | n09 — one source character from n01: a NON-trivial destructor makes `p->~T()` a real call |
//! | `two_nothing_statements_are_refused` | n10 — "emits nothing" is a property of the WHOLE body, and the walk is total. A match declined on purpose |
//! | `self_recursion_through_a_nothing_statement_is_refused` | n11 — #950, graded on the bytes and not on the relocation count |

mod cellgrade;

use c2_harness::gap::fnbytes::FnByte;
use c2_reference::Toolchain;

use cellgrade::{grade_cell, hex, row, row_anchor, row_opt, work, Rows, BLR};

// The FROZEN grid — `include_str!` and not a copy, so this test grades the bytes
// whose `sha256` was committed before the first `cl.exe`
// (`work/w-seed/CELLS.sha256`).
// PROV[N] not load-bearing — a FIXTURE SOURCE string. It is INPUT to c2, graded by the byte judge against real c2's output; nothing about its value is derived from `c2.dll`. DISCLOSURE names fixture material under [N].
const N01: &str = include_str!("../../../work/w-seed/cells/n01.cpp");
// PROV[N] not load-bearing — a FIXTURE SOURCE string. It is INPUT to c2, graded by the byte judge against real c2's output; nothing about its value is derived from `c2.dll`. DISCLOSURE names fixture material under [N].
const N02: &str = include_str!("../../../work/w-seed/cells/n02.cpp");
// PROV[N] not load-bearing — a FIXTURE SOURCE string. It is INPUT to c2, graded by the byte judge against real c2's output; nothing about its value is derived from `c2.dll`. DISCLOSURE names fixture material under [N].
const N03: &str = include_str!("../../../work/w-seed/cells/n03.cpp");
// PROV[N] not load-bearing — a FIXTURE SOURCE string. It is INPUT to c2, graded by the byte judge against real c2's output; nothing about its value is derived from `c2.dll`. DISCLOSURE names fixture material under [N].
const N04: &str = include_str!("../../../work/w-seed/cells/n04.cpp");
// PROV[N] not load-bearing — a FIXTURE SOURCE string. It is INPUT to c2, graded by the byte judge against real c2's output; nothing about its value is derived from `c2.dll`. DISCLOSURE names fixture material under [N].
const N05: &str = include_str!("../../../work/w-seed/cells/n05.cpp");
// PROV[N] not load-bearing — a FIXTURE SOURCE string. It is INPUT to c2, graded by the byte judge against real c2's output; nothing about its value is derived from `c2.dll`. DISCLOSURE names fixture material under [N].
const N06: &str = include_str!("../../../work/w-seed/cells/n06.cpp");
// PROV[N] not load-bearing — a FIXTURE SOURCE string. It is INPUT to c2, graded by the byte judge against real c2's output; nothing about its value is derived from `c2.dll`. DISCLOSURE names fixture material under [N].
const N07: &str = include_str!("../../../work/w-seed/cells/n07.cpp");
// PROV[N] not load-bearing — a FIXTURE SOURCE string. It is INPUT to c2, graded by the byte judge against real c2's output; nothing about its value is derived from `c2.dll`. DISCLOSURE names fixture material under [N].
const N08: &str = include_str!("../../../work/w-seed/cells/n08.cpp");
// PROV[N] not load-bearing — a FIXTURE SOURCE string. It is INPUT to c2, graded by the byte judge against real c2's output; nothing about its value is derived from `c2.dll`. DISCLOSURE names fixture material under [N].
const N09: &str = include_str!("../../../work/w-seed/cells/n09.cpp");
// PROV[N] not load-bearing — a FIXTURE SOURCE string. It is INPUT to c2, graded by the byte judge against real c2's output; nothing about its value is derived from `c2.dll`. DISCLOSURE names fixture material under [N].
const N10: &str = include_str!("../../../work/w-seed/cells/n10.cpp");
// PROV[N] not load-bearing — a FIXTURE SOURCE string. It is INPUT to c2, graded by the byte judge against real c2's output; nothing about its value is derived from `c2.dll`. DISCLOSURE names fixture material under [N].
const N11: &str = include_str!("../../../work/w-seed/cells/n11.cpp");

/// Both flag settings, in order: the workload's own, then `/Ob0`.
/// PROV[N] not load-bearing — a MEASUREMENT PROFILE, named under [N] in DISCLOSURE: the compiler flag list this cell is captured and graded at. It selects which behaviour is observed; it is not a value read from c2. The two `/Ob0` arms this seed is graded at.
const SETTINGS: [(&str, &[&str]); 2] = [("/O1", &[]), ("/O1 /Ob0", &["/Ob0"])];

/// Print every graded row's shape, verdict, relocation count and **whole
/// `.text`** — `w-fix`'s template. A verdict with no bytes beside it cannot be
/// re-read later, and #950 is the standing reason the bytes are the observable.
fn show(cell: &str, at: &str, rows: &Rows) {
    println!("--- {cell} at {at}");
    for (shape, verdict, sym, bytes, nrel) in rows {
        println!("    {shape:<14} {verdict:?} rel={nrel} [{}]  {sym}", hex(bytes));
    }
}

/// **THE NEGATIVE CELLS' ASSERTION, stated as what it means.**
///
/// The first draft of this file asserted `verdict != Exact` for the callers that
/// must not be elided, and four cells went red on a **correct** port: c2 emits a
/// surviving `48000000` for those callers and the port emits one too, so `Exact`
/// there is the right answer and not a wrong one. A negative cell that fires on
/// success is worse than no cell — it makes the next lane delete it.
///
/// What "not elided" actually means is `!reduces_to_nothing(caller)`:
/// `elide.rs`'s own `eliding_a_call_and_reducing_to_nothing_are_the_same_fact`
/// says that for a tail call with no data symbol those two are one fact, so this
/// asks the rule directly instead of inferring it from a byte compare that has
/// three other reasons to move. c2's own body is printed and checked too: if c2
/// emitted one relocation-free `blr` here there would be nothing to decline and
/// the cell would have stopped testing its axis.
fn not_elided(
    tu: &c2_core::elide::TuEmptyCallees,
    r: &(&'static str, FnByte, String, Vec<u8>, usize),
    cell: &str,
    at: &str,
) {
    assert!(
        !tu.reduces_to_nothing(&r.2),
        "{cell} at {at}: `{}` REDUCES TO NOTHING — the port elided a call this cell \
         exists to say it must keep. c2's own body is [{}] rel={}",
        r.2,
        hex(&r.3),
        r.4
    );
}

/// **c2 KEPT CODE here**, so the port declining is the *same* answer c2 gives and
/// the cell is testing a real disagreement rather than an agreement.
///
/// Paired with [`not_elided`] on the cells where the port and c2 agree that the
/// call survives (`n04`, `n06`, `n09`, `n11`). Without it, "the port did not
/// elide" would pass just as well on a cell c2 had quietly collapsed, and the axis
/// would have stopped being tested with nothing to show for it.
fn c2_kept_code(r: &(&'static str, FnByte, String, Vec<u8>, usize), cell: &str, at: &str) {
    assert!(
        !(r.3.as_slice() == BLR.as_slice() && r.4 == 0),
        "{cell} at {at}: c2 emitted one relocation-free `blr` for `{}`, so there is \
         nothing here for the port to decline and this cell has stopped testing the \
         axis it was written for",
        r.2
    );
}

/// **c2 COLLAPSED it and the port declines anyway** — a match given up on purpose.
///
/// This is the other half of the grid and it is the more interesting one. `n05`
/// and `n08` are both cells where c2 emits one relocation-free `blr` for the
/// caller and the port keeps its branch (or refuses the body outright), because
/// `elide.rs` will not claim a shape no grid graded:
///
/// * `n05` is `c19_ret_param`'s trap one level up — a mid-node that RETURNS A
///   VALUE is mechanism **I**, and at `/O1` its caller is observationally
///   identical to an E caller. A rule fitted to the bytes takes it; this one does
///   not (`elide.rs` condition 2).
/// * `n08` is `g01_data_addr_arg` — E in c2, and declined by the port's IL parser
///   before condition 3 is even reached (`elide.rs` condition 3's own doc says so
///   in as many words).
///
/// **Both were registered in `work/w-seed/PREREG.md` §2 with the opposite
/// expectation** — "c2 keeps its REL24", "an honest differ" — and both are
/// recorded as registered predictions LOST rather than rewritten.
fn declined_on_purpose(
    tu: &c2_core::elide::TuEmptyCallees,
    r: &(&'static str, FnByte, String, Vec<u8>, usize),
    cell: &str,
    at: &str,
) {
    not_elided(tu, r, cell, at);
    assert_eq!(
        (r.3.as_slice(), r.4),
        (BLR.as_slice(), 0),
        "{cell} at {at}: c2's body for `{}` is [{}] rel={}, not the one \
         relocation-free `blr` this cell measured. The cell is now a plain \
         disagreement and no longer records a match declined on purpose",
        r.2,
        hex(&r.3),
        r.4
    );
}

/// **n01 — THE POSITIVE.** A nothing-body reached directly by a tail call: no
/// loop, no dead temporary, nothing between the caller and the seed.
///
/// The `/Ob0` half is what says this is E. Mechanism E is not governed by `/Ob`
/// and mechanism I is; at `/O1` a mid-chain inline is a bare `blr` at every level
/// (`w-fix` #954), so a cell graded at one setting has not said which mechanism it
/// measured.
#[test]
fn a_nothing_body_seeds_and_its_caller_collapses() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("n01");
    for (at, extra) in SETTINGS {
        let (rows, tu) = grade_cell(&tc, &d, "n01", N01, extra);
        show("n01", at, &rows);
        let leaf = row(&rows, "??$da@", "n01");
        assert_eq!(
            (leaf.0, leaf.1),
            ("parse-refused", FnByte::Refused),
            "n01 at {at}: THE LEAF PARSES NOW. `no_effect_nothing` is decode-only \
             by construction and `IlBundle::functions` must keep refusing this TU \
             (#971 condition 4); accepting the body is a different rung"
        );
        assert!(
            tu.reduces_to_nothing(&leaf.2),
            "n01 at {at}: the leaf `{}` did NOT seed. It is refused and it names no \
             callee, so no link can reach it — the fixpoint has no bottom here",
            leaf.2
        );
        let c = row(&rows, "?use@", "n01");
        assert_eq!(
            (c.3.as_slice(), c.4),
            (BLR.as_slice(), 0),
            "n01 at {at}: c2's own body for the caller is not one `blr` with no \
             relocation — the premise this whole grid rests on has changed. Got \
             [{}] with {} relocations",
            hex(&c.3),
            c.4
        );
        assert_eq!(
            (c.0, c.1),
            ("tail", FnByte::Exact),
            "n01 at {at}: the caller is {:?}. The seed did not reach it, so \
             `Reduction::NoEffectNothing` is not being minted or is not seeding",
            c.1
        );
    }
    let _ = std::fs::remove_dir_all(&d);
}

/// **n02 — the seed PROPAGATES.** Three links above the leaf, every edge graded.
///
/// A seed that reached only its immediate caller would be a link wearing a seed's
/// name. `w-fix` graded the fixpoint at depths 1..6 and 8 over an `empty_body`
/// seed; this asks the same question with a *refused* one at the bottom.
#[test]
fn a_seed_propagates_up_a_chain() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("n02");
    for (at, extra) in SETTINGS {
        let (rows, tu) = grade_cell(&tc, &d, "n02", N02, extra);
        show("n02", at, &rows);
        row_anchor(&rows, "n02");
        let leaf = row(&rows, "??$da@", "n02");
        assert!(
            tu.reduces_to_nothing(&leaf.2),
            "n02 at {at}: the leaf did not seed, so nothing below is meaningful"
        );
        // EVERY EDGE, and never just the top one.
        for needle in ["??$d1@", "??$d2@", "?use@"] {
            let Some(r) = row_opt(&rows, needle, "n02") else {
                panic!("n02 at {at}: c2 emitted no COMDAT for `{needle}` at all");
            };
            assert_eq!(
                (r.3.as_slice(), r.4),
                (BLR.as_slice(), 0),
                "n02 at {at}: c2's body for `{}` is [{}] with {} relocations, not \
                 one relocation-free `blr` — the chain does not collapse in c2 and \
                 the port must not be graded as if it did",
                r.2,
                hex(&r.3),
                r.4
            );
            assert_eq!(
                r.1,
                FnByte::Exact,
                "n02 at {at}: `{}` is {:?}. The seed stopped propagating at this \
                 link, which is a fixpoint that takes one step and not a fixpoint",
                r.2,
                r.1
            );
            assert!(
                tu.reduces_to_nothing(&r.2),
                "n02 at {at}: `{}` was emitted correctly but is not in the closure \
                 — eliding a call and reducing to nothing must be the same fact",
                r.2
            );
        }
    }
    let _ = std::fs::remove_dir_all(&d);
}

/// **n03 — THE WORKLOAD'S OWN SHAPE.** All five levels: the dead temporary
/// (`w-inl0`), the destroy loop (`w-memset`) and the seed (this lane), composed.
///
/// This is the source of `work/w-memset/cells/l01.cpp`, and the 223 workload
/// conversions are all this shape. It is graded here as well as in
/// `destroy_loop_elision.rs` because the two files assert different things about
/// it: that one owns GRID-L's loop reader, this one owns the seed.
#[test]
fn the_workload_five_level_chain_closes() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("n03");
    for (at, extra) in SETTINGS {
        let (rows, tu) = grade_cell(&tc, &d, "n03", N03, extra);
        show("n03", at, &rows);
        let w = row(&rows, "??$destroy_range@", "n03");
        assert_eq!(
            (w.3.as_slice(), w.4),
            (BLR.as_slice(), 0),
            "n03 at {at}: c2's body for the wrapper is [{}] with {} relocations. If \
             this changed, the residue is mechanism I after all and the rung is \
             priced wrong",
            hex(&w.3),
            w.4
        );
        assert_eq!(
            (w.0, w.1),
            ("tail", FnByte::Exact),
            "n03 at {at}: the wrapper is {:?} — the five-level chain does not close",
            w.1
        );
        // The three readers, each named, so a break can be attributed to one.
        for (needle, what) in [
            ("??$destroy_aux@", "the SEED (no_effect_nothing)"),
            ("??$aux@", "the LOOP link (no_effect_loop)"),
            ("??$dr@", "the dead-temporary link (no_effect_call)"),
        ] {
            let r = row(&rows, needle, "n03");
            assert!(
                tu.reduces_to_nothing(&r.2),
                "n03 at {at}: {what} did not admit `{}`, so the chain is broken at \
                 that level and the wrapper's verdict above says nothing about it",
                r.2
            );
            assert_eq!(
                r.1,
                FnByte::Refused,
                "n03 at {at}: `{}` PARSES NOW — every one of the three readers is \
                 decode-only and this TU must stay refused (#971 condition 4)",
                r.2
            );
        }
    }
    let _ = std::fs::remove_dir_all(&d);
}

/// **n04 — a body refused for ANOTHER reason never seeds**, and `body-0x67` is
/// the one that matters.
///
/// It holds **5,154** no-effect chain stops on the workload, and w-memset #1056
/// records that this is a hazard rather than headroom: `body-0x67` is the
/// virtual-dispatch production, and E **does not model the call site**
/// (`INLINE_PREDICATE.md` §1.3, board #921 — `f10_virtual_ptr`). The port is safe
/// only because the parser refuses it. If a seeding reader ever admits one, E
/// becomes a wrong emit at an indirect site: board #232's shape.
#[test]
fn a_body_refused_for_another_reason_never_seeds() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("n04");
    for (at, extra) in SETTINGS {
        let (rows, tu) = grade_cell(&tc, &d, "n04", N04, extra);
        show("n04", at, &rows);
        row_anchor(&rows, "n04");
        if let Some(v) = row_opt(&rows, "?vcall@", "n04") {
            assert!(
                !tu.reduces_to_nothing(&v.2),
                "n04 at {at}: THE VIRTUAL-DISPATCH BODY `{}` WAS ADMITTED. E does \
                 not model the call site, and the only thing keeping it safe from \
                 an indirect one is this refusal (#921/#1056). PREREG §3 clause 6 \
                 stops the lane here",
                v.2
            );
        }
        if let Some(u) = row_opt(&rows, "?use@", "n04") {
            not_elided(&tu, u, "n04", at);
            c2_kept_code(u, "n04", at);
        }
    }
    let _ = std::fs::remove_dir_all(&d);
}

/// **n05 — mechanism I above a seed does not propagate.** `w-fix` #954's trap,
/// with a seed at the bottom instead of an empty body.
///
/// `mid` returns a value, so c2 does not drop the call to it — it EXPANDS it. At
/// `/O1` an I caller and an E caller are both a bare `blr` and are
/// indistinguishable; the `/Ob0` row is the entire content of this cell.
#[test]
fn mechanism_i_above_a_seed_does_not_propagate() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("n05");
    for (at, extra) in SETTINGS {
        let (rows, tu) = grade_cell(&tc, &d, "n05", N05, extra);
        show("n05", at, &rows);
        row_anchor(&rows, "n05");
        if let Some(leaf) = row_opt(&rows, "??$da@", "n05") {
            assert!(
                tu.reduces_to_nothing(&leaf.2),
                "n05 at {at}: the leaf did not seed, so this cell is not testing \
                 propagation THROUGH an inline — it is testing nothing"
            );
        }
        let m = row_opt(&rows, "??$mid@", "n05").or_else(|| row_opt(&rows, "?mid@", "n05"));
        if let Some(m) = m {
            assert!(
                !tu.reduces_to_nothing(&m.2),
                "n05 at {at}: `{}` was admitted. It RETURNS A VALUE — c2 expands it \
                 rather than dropping the call, and a fixpoint fitted to the /O1 \
                 bytes takes the whole chain and is wrong about all of it (#954)",
                m.2
            );
        }
        if let Some(u) = row_opt(&rows, "?use@", "n05") {
            declined_on_purpose(&tu, u, "n05", at);
        }
    }
    let _ = std::fs::remove_dir_all(&d);
}

/// **n06 — THE CYCLE, and it is PREREG §0.3 compiled.**
///
/// `w-fix` #950 argued *"a cycle is never seeded, so it is never admitted"*, and
/// that sentence was true only because `empty_body` was the only seed. The
/// re-derivation turns on step (2): **a seeded name has no outgoing link**,
/// because the reader's vocabulary contains no call token. Both members here carry
/// the nothing-statement *and* a call, so both are refused — not "accepted with
/// something extra", simply not in the language.
///
/// If this ever goes red, the fixpoint's termination rests on the round ceiling
/// alone and `Reduction`'s four-step argument is gone.
#[test]
fn a_cycle_of_nothing_bodies_is_never_admitted() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("n06");
    for (at, extra) in SETTINGS {
        let (rows, tu) = grade_cell(&tc, &d, "n06", N06, extra);
        show("n06", at, &rows);
        row_anchor(&rows, "n06");
        for needle in ["??$a1@", "??$b1@"] {
            if let Some(r) = row_opt(&rows, needle, "n06") {
                assert!(
                    !tu.reduces_to_nothing(&r.2),
                    "n06 at {at}: A CYCLE MEMBER WAS ADMITTED — `{}`. A seed carries \
                     no link, and a cycle member always has one, so this cannot \
                     happen unless the reader started accepting a body with a call \
                     in it. `Reduction`'s step (2) is what just broke",
                    r.2
                );
            }
        }
        assert!(
            !tu.overflowed(),
            "n06 at {at}: THE ROUND CEILING FIRED. The fixpoint is no longer \
             monotone; the context now admits NOTHING, which is the safe answer and \
             not a working one"
        );
        if let Some(u) = row_opt(&rows, "?use@", "n06") {
            not_elided(&tu, u, "n06", at);
            c2_kept_code(u, "n06", at);
        }
    }
    let _ = std::fs::remove_dir_all(&d);
}

/// **n07 — CONTROL. The same-TU condition, re-asked of the SEED.**
///
/// `c22_extern_callee` graded it for the link and `w-inl0`'s `m07` for the
/// dead-temporary edge. A seed is an unconditional claim, so it gets its own cell
/// rather than inheriting either: c2 keeps its REL24 at both flag settings, and
/// the relocation count is asserted so that "the port emitted no branch" and
/// "nothing in this cell emitted anything" cannot be the same observation.
#[test]
fn an_external_nothing_body_keeps_its_relocation() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("n07");
    for (at, extra) in SETTINGS {
        let (rows, tu) = grade_cell(&tc, &d, "n07", N07, extra);
        show("n07", at, &rows);
        row_anchor(&rows, "n07");
        assert!(
            !tu.reduces_to_nothing("?da_ext@@YAXPAUS@@@Z"),
            "n07 at {at}: a callee this TU does NOT DEFINE was admitted. Dropping \
             the same-TU condition turns `c22_extern_callee` into a wrong emit"
        );
        let u = row(&rows, "?use@", "n07");
        assert!(
            u.4 > 0,
            "n07 at {at}: c2's body for the caller is [{}] with ZERO relocations — \
             it must keep its REL24 to the external, and if it does not, this cell \
             is no longer the control it was written to be",
            hex(&u.3)
        );
        assert_ne!(
            u.3.as_slice(),
            BLR.as_slice(),
            "n07 at {at}: c2 emitted one `blr` for a call to an EXTERNAL"
        );
    }
    let _ = std::fs::remove_dir_all(&d);
}

/// **n08 — condition 3, re-asked of the SEED.** The leaf is a genuine
/// nothing-body and the caller materializes a global's address to reach it.
///
/// `g01_data_addr_arg` is E in c2 and no grid ever graded an elided tail call that
/// also materializes a data symbol; `elide.rs` declines rather than letting the
/// workload be the first case, and `data_refs_of` would in any event fail to locate
/// a relocation half inside a one-word `blr`.
#[test]
fn a_caller_that_materializes_data_is_not_elided() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("n08");
    for (at, extra) in SETTINGS {
        let (rows, tu) = grade_cell(&tc, &d, "n08", N08, extra);
        show("n08", at, &rows);
        row_anchor(&rows, "n08");
        if let Some(leaf) = row_opt(&rows, "??$da@", "n08") {
            assert!(
                tu.reduces_to_nothing(&leaf.2),
                "n08 at {at}: the leaf did not seed, so the caller's refusal below \
                 has some other cause and this cell is not testing condition 3"
            );
        }
        let u = row(&rows, "?use@", "n08");
        declined_on_purpose(&tu, u, "n08", at);
    }
    let _ = std::fs::remove_dir_all(&d);
}

/// **n09 — a body that keeps BYTES never seeds.** One source character from `n01`:
/// `T2` has a NON-trivial destructor, so `p->~T()` is a real call and the front end
/// emits a different production entirely.
///
/// This is the cell that says the reader is keyed on the decoded BODY and not on
/// the spelling `p->~T()`.
#[test]
fn a_body_that_keeps_bytes_never_seeds() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("n09");
    for (at, extra) in SETTINGS {
        let (rows, tu) = grade_cell(&tc, &d, "n09", N09, extra);
        show("n09", at, &rows);
        row_anchor(&rows, "n09");
        if let Some(leaf) = row_opt(&rows, "??$da@", "n09") {
            assert!(
                !tu.reduces_to_nothing(&leaf.2),
                "n09 at {at}: `{}` was admitted. Its body CALLS `~T2()`; the reader \
                 is keyed on the decoded body and a call is not in its vocabulary. \
                 c2's own body for it is [{}] rel={}",
                leaf.2,
                hex(&leaf.3),
                leaf.4
            );
        }
        if let Some(u) = row_opt(&rows, "?use@", "n09") {
            not_elided(&tu, u, "n09", at);
            c2_kept_code(u, "n09", at);
        }
    }
    let _ = std::fs::remove_dir_all(&d);
}

/// **n10 — TWO of the statement, and it is REFUSED.**
///
/// Two discarded pseudo-destructors emit nothing exactly as one does, so this is a
/// match declined on purpose: the shape that was graded has a single statement in
/// it, and the walk being **total** is the only thing that makes "there is nothing
/// else in this body" checkable rather than searched for.
#[test]
fn two_nothing_statements_are_refused() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("n10");
    for (at, extra) in SETTINGS {
        let (rows, tu) = grade_cell(&tc, &d, "n10", N10, extra);
        show("n10", at, &rows);
        row_anchor(&rows, "n10");
        if let Some(leaf) = row_opt(&rows, "??$da2@", "n10") {
            assert!(
                !tu.reduces_to_nothing(&leaf.2),
                "n10 at {at}: a body with TWO statements was admitted. The walk is \
                 supposed to be total — a second statement has nowhere to hide, so \
                 either the terminal is gone or the statement walk now loops"
            );
        }
    }
    let _ = std::fs::remove_dir_all(&d);
}

/// **n11 — self-recursion, graded on the BYTES.** Board #950.
///
/// `void r(){r();}` emits a self-branch that takes **no relocation at all**, so the
/// relocation observable — the one the whole E family is built on — reads "nothing
/// happened" for a body that is plainly not nothing. A grid scored by counting
/// relocations calls this E. The verdict here is read off the bytes, and the bytes
/// are printed.
#[test]
fn self_recursion_through_a_nothing_statement_is_refused() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let d = work("n11");
    for (at, extra) in SETTINGS {
        let (rows, tu) = grade_cell(&tc, &d, "n11", N11, extra);
        show("n11", at, &rows);
        row_anchor(&rows, "n11");
        if let Some(r) = row_opt(&rows, "??$r@", "n11") {
            assert!(
                !tu.reduces_to_nothing(&r.2),
                "n11 at {at}: THE SELF-RECURSIVE BODY WAS ADMITTED — `{}`, c2 body \
                 [{}] rel={}. Its relocation count is {} and that is exactly why \
                 the count is not the observable (#950)",
                r.2,
                hex(&r.3),
                r.4,
                r.4
            );
            assert_ne!(
                r.3.as_slice(),
                BLR.as_slice(),
                "n11 at {at}: c2 emitted one `blr` for a self-recursive body — the \
                 premise of this cell has changed and its verdict means nothing"
            );
            // **BOARD #950, WITNESSED.** c2's body for `??$r@…` is one branch word
            // and it takes **ZERO** relocations, because the target is the function
            // itself. So the relocation observable — "no REL24, therefore nothing
            // was emitted" — reads `E` here on a body that plainly is not nothing.
            // Asserted rather than remarked on, so the day it stops being true the
            // grid says so instead of quietly losing its sharpest cell.
            assert_eq!(
                (r.3.len(), r.4),
                (4, 0),
                "n11 at {at}: the self-branch is [{}] with {} relocations. This cell \
                 exists because that count is 0 for a body that emits real code; if \
                 it is not, #950's hazard has changed shape",
                hex(&r.3),
                r.4
            );
        }
        if let Some(u) = row_opt(&rows, "?use@", "n11") {
            not_elided(&tu, u, "n11", at);
            c2_kept_code(u, "n11", at);
        }
    }
    let _ = std::fs::remove_dir_all(&d);
}
