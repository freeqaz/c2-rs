//! **The `w-pool2` cells: the free-list TU that converts, and the seven axes
//! pinned around it** — lane `w-pool2`, board **#2590**–**#2596**.
//!
//! # What this pins that no fixture verdict can
//!
//! `fixtures/cpp/wpool2_free_list.cpp` is `src/system/utl/Pool.cpp` — the whole
//! TU, three leaves, 132 bytes — and at this tree it is a `match`. What a lane
//! can get wrong is not *that* it matches but **how wide the class is**, and the
//! seven cells of `wpool2_free_list_neg.cpp` state the boundary one axis at a
//! time:
//!
//! ```text
//!   N1  the POP's guarded arm returns a NON-ZERO literal
//!   N2  the PUSH's two stores in the OTHER order
//!   N3  the PUSH touches TWO different members
//!   N4  the PUSH carries a SECOND formal
//!   N5  the constructor's guard literal is 0, not 1
//!   N6  the constructor rounds up to EIGHT, not four
//!   N7  the walking pointer's element scale is not 1
//! ```
//!
//! # **THE CENSUS KEY DOES NOT DISCRIMINATE THESE CELLS, AND THAT IS WHY THE
//! FENCE IS PROVED SOMEWHERE ELSE**
//!
//! All seven refuse, and across seven DISTINCT clauses they report **three**
//! keys: `expr-brtrue` ×3, `expr-op-0x32` ×3 and `expr-op-0x30` ×1. Those are
//! the fall-through keys boards **#1101** and **#1416** describe — they name
//! where the *generic* walk stopped after the whole-body production declined,
//! not the clause that declined it. A test asserting seven distinct keys here
//! would assert a fiction, so this file asserts the multiset it actually has
//! (below) and the fences are graded by a different instrument.
//!
//! Three keys for seven clauses is itself the measurement, and it is the reason
//! a `_neg` file cannot be graded by reading its census rows.
//!
//! > **The multiset MOVED at Phase 1 slice C1 (lane `w-c1`, 2026-08-24), and
//! > the movement is the fall-through thesis being executed rather than
//! > argued.** It read `expr-op-0x27` ×4 + `expr-brtrue` ×3 until `0x27` — the
//! > byte-offset add — became a graded construct in `parse_expr`. **All seven
//! > cells still refuse**, and the four that named the designator step now name
//! > what was standing behind it one construct along (`0x30`, the indirect
//! > load; `0x32`). Board **#150** predicted exactly this shape at workload
//! > scale: unblocking `expr-op-0x27` *renames* far more than it converts. Four
//! > cells splitting 1/3 across two successors is the same fact at fixture
//! > scale — and note it went from two keys to three, so the file's own count
//! > is part of the assertion.
//!
//! That instrument is `work/w-pool2/neg_clauses.py`: for each cell it **mutates
//! the one shipping clause the cell is written for**, rebuilds, and re-censuses
//! — the cell must move IN CLASS and no other may. Output in
//! `work/w-pool2/NEG_CLAUSES.txt`; every patch reverted (board #1704).
//!
//! ```text
//!   N1 moved [0]  N2 moved [1]  N3 moved [2]  N4 moved [3]
//!   N5 moved [4]  N6 moved [5]  N7 moved [6]        7 of 7 EXACT
//! ```
//!
//! **Two of the seven are SWAPS rather than relaxations** — N2 and N6 make the
//! negative acceptable *and* a function of the positive fixture refuse
//! (`positive lost [2]` and `[0]`), which a blanket relaxation could not fake.
//!
//! **N4's first draft was confounded and the instrument caught it.** With a
//! named third parameter and a `(void)unused;` discard, the cell carried an
//! extra IL statement and stayed blocked with the arity clause relaxed. That is
//! `w-biquad` #2535 inside this lane's own `_neg` file, found by running the
//! probe and not by reading the cell.
//!
//! # The four must-fail mutations, RUN
//!
//! `work/w-pool2/mutations.py`, each applied to a shipping **emitter**, graded
//! by real `c2.dll` through `scripts/mode_lane.sh /O1`, and reverted. Baseline
//! `LANE-RESULT PASS … graded=341 match=167 mismatch=0`:
//!
//! ```text
//!   M1  POP parks `this` in r10, not r11              mismatch=1
//!   M2  the ctor's `rotlwi` NOT hoisted above the
//!       member-init store                             mismatch=1
//!   M3  the ctor's `twi 6` one slot earlier, ahead
//!       of the `andc`                                 mismatch=1
//!   M4  the PUSH's two stores emitted in the other
//!       order                                         mismatch=2
//! ```
//!
//! **Four for four, and each is a `mismatch` rather than a refusal** — the port
//! emits bytes and real `c2` disagrees with them. Two of the four are about the
//! constructor's SCHEDULE, which is the part of that body this lane transcribes
//! rather than derives, so it is the part most in need of a live fence.
//!
//! # Why the sources are `include_str!`-ed
//!
//! `w-fence2` §5.1's rule: a cell that re-types its subject grades a copy.
//!
//! `SKIP: toolchain absent` when there is no toolchain, like every other
//! integration test here.

use std::collections::BTreeMap;
use std::path::PathBuf;

use c2_reference::Toolchain;

/// The workload's own profile minus the `/I` paths a standalone cell cannot
/// use. **`/O1`, deliberately**: both classes are `/O1`-only, and `/O1` implies
/// `/Gy`, which is the regime the 878-TU scan lives in and the one
/// `Pool.obj`'s three COMDAT `.text` sections belong to.
/// PROV[N] not load-bearing — a MEASUREMENT PROFILE, named under [N] in DISCLOSURE: the compiler flag list this cell is captured and graded at. It selects which behaviour is observed; it is not a value read from c2.
const FLAGS: [&str; 8] = c2_harness::testsupport::WORKLOAD_FLAGS;

// PROV[N] not load-bearing — a FIXTURE SOURCE string. It is INPUT to c2, graded by the byte judge against real c2's output; nothing about its value is derived from `c2.dll`. DISCLOSURE names fixture material under [N].
const POSITIVE: &str = include_str!("../../../fixtures/cpp/wpool2_free_list.cpp");
// PROV[N] not load-bearing — a FIXTURE SOURCE string. It is INPUT to c2, graded by the byte judge against real c2's output; nothing about its value is derived from `c2.dll`. DISCLOSURE names fixture material under [N].
const NEGATIVE: &str = include_str!("../../../fixtures/cpp/wpool2_free_list_neg.cpp");

fn work(tag: &str) -> PathBuf {
    c2_harness::testsupport::scratch_dir("pool2cell", tag)
}

/// Capture one source at [`FLAGS`] and return `(its per-function census, its
/// `DecodeCauses`)`. Both come off the **same** capture, so a claim about the
/// census and a claim about the gate cause cannot be about two compilations.
fn cell(
    tc: &Toolchain,
    tag: &str,
    body: &str,
) -> (Vec<c2_il::func::FnCensus>, c2_il::func::DecodeCauses) {
    let dir = work(tag);
    let cpp = dir.join(format!("{tag}.cpp"));
    std::fs::write(&cpp, body).unwrap();
    let flags: Vec<String> = FLAGS.iter().map(|s| s.to_string()).collect();
    let src = c2_reference::to_wibo_path(&cpp);
    let cap = tc
        .capture_reference_with(&src, &dir, &flags, None)
        .unwrap_or_else(|e| panic!("cell `{tag}`: capture failed: {e}"));
    let census = cap
        .bundle
        .function_census()
        .unwrap_or_else(|| panic!("cell `{tag}`: no `.ex` census"));
    (census, cap.bundle.decode_causes())
}

/// **The whole TU is in class, all three bodies, and the TU-level gate stops
/// nowhere** — which is the difference between "three bodies were admitted" and
/// "the obj can be built", and only the second converts anything.
#[test]
fn the_free_list_tu_is_in_class_end_to_end() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let (rows, causes) = cell(&tc, "pos", POSITIVE);
    assert_eq!(rows.len(), 3, "Pool.cpp is three functions: {rows:?}");
    let keys: Vec<String> = rows.iter().map(|r| r.verdict.key()).collect();
    assert_eq!(
        keys,
        vec!["pool-ctor-chain", "pool-free-list", "pool-free-list"],
        "…in the workload's own COMDAT order: ctor, POP, PUSH: {rows:?}"
    );
    assert!(
        rows.iter().all(|r| r.verdict.in_class()),
        "every body in class: {rows:?}"
    );
    // The constructor is the one body here with a back edge, and the two guards
    // are the `cflow-if-1`s. Asserted so a future edit that collapsed the two
    // classes into one would have to say so.
    let mut cflow: Vec<&str> = rows.iter().map(|r| r.cflow.as_str()).collect();
    cflow.sort_unstable();
    assert_eq!(cflow, vec!["cflow-if-1", "cflow-if-1", "cflow-loop"]);
    assert_eq!(causes.first, None, "the TU stops at no gate: {causes:?}");
    assert!(causes.decodes, "…and it decodes: {causes:?}");
    assert_eq!(causes.bodies_out_of_class, 0, "…none out of class: {causes:?}");
}

/// **The seven negatives all refuse, and their keys are a MULTISET rather than
/// seven distinct values** — asserted as what it is, because the alternative is
/// a test that claims a discrimination the census cannot make (#1101, #1416).
///
/// The per-cell fences are graded by `work/w-pool2/neg_clauses.py`; see the
/// module header for its seven-of-seven result.
#[test]
fn every_negative_cell_refuses_and_the_keys_are_fall_throughs() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let (rows, _) = cell(&tc, "neg", NEGATIVE);
    assert_eq!(rows.len(), 7, "seven cells: {rows:?}");
    assert!(
        rows.iter().all(|r| !r.verdict.in_class()),
        "every cell must refuse — a `_neg` that is in class is inert: {rows:?}"
    );
    let mut hist: BTreeMap<String, usize> = BTreeMap::new();
    for r in &rows {
        *hist.entry(r.verdict.key()).or_default() += 1;
    }
    assert_eq!(
        hist,
        BTreeMap::from([
            ("expr-brtrue".to_string(), 3),
            // C1 (`w-c1`): these four read `expr-op-0x27` ×4 until the
            // byte-offset add became a graded construct. They still refuse; the
            // key is now the successor. See the module header.
            ("expr-op-0x30".to_string(), 1),
            ("expr-op-0x32".to_string(), 3),
        ]),
        "the keys are FALL-THROUGHS and this file says so rather than \
         pretending to seven distinct keys — seven clauses, THREE keys: {rows:?}"
    );
    // …and none of them is the acceptance key of either shipped class, which is
    // the one thing the census CAN say here: no cell was admitted by accident.
    assert!(
        rows.iter()
            .all(|r| !matches!(r.verdict.key().as_str(), "pool-free-list" | "pool-ctor-chain")),
        "no cell may be admitted by either class: {rows:?}"
    );
}

/// **The binding predicate passes on both files** — `CEILING.md` §11.4 item 8,
/// checked rather than assumed, and in the direction that matters here: these
/// cells BIND, so the refusals above really are the reader's and not a gate
/// standing in front of them.
#[test]
fn both_pool2_files_bind_their_records_before_any_body_is_looked_at() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    for (tag, body, segs) in [("bind-pos", POSITIVE, 3), ("bind-neg", NEGATIVE, 7)] {
        let (_, c) = cell(&tc, tag, body);
        assert_eq!(c.segments, segs, "cell `{tag}`: {segs} `.ex` bodies: {c:?}");
        for stop in [
            c2_il::func::cause::BIND_COUNT,
            c2_il::func::cause::BIND_OFFSET,
            c2_il::func::cause::GL_26_INTRODUCED,
            c2_il::func::cause::GL_NAME_NOT_MANGLED,
            c2_il::func::cause::DRECTVE,
        ] {
            assert!(
                !c.causes.contains(&stop),
                "cell `{tag}`: the gate must not stop at `{stop}` — every refusal \
                 claimed here is the BODY's: {c:?}"
            );
        }
    }
}

/// **The mode gate is in the PARSER, not only in the emitter** — board #1638's
/// remedy and #1710's second instance, asserted rather than trusted.
///
/// At `/Ox` this TU is a *different obj*: the constructor is twenty-one words
/// with the register plan r9/r10/r8/r7, and `Alloc` stops folding its guard to
/// a `bclr` altogether (`work/w-pool2/ref/PoolOx.obj`). If the `/O1` clause
/// lived only in `codegen`, the census would count three functions in class
/// that `PortC2` refuses — which is exactly the disagreement `census_gate.rs`
/// exists to catch and which #1710 recorded `static_scan_loop` shipping.
#[test]
fn neither_class_is_in_class_at_ox() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let dir = work("ox");
    let cpp = dir.join("ox.cpp");
    std::fs::write(&cpp, POSITIVE).unwrap();
    let flags: Vec<String> = ["/nologo", "/c", "/GR", "/Ox", "/Oi", "/EHsc"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let src = c2_reference::to_wibo_path(&cpp);
    let cap = tc
        .capture_reference_with(&src, &dir, &flags, None)
        .expect("capture at /Ox");
    let rows = cap.bundle.function_census().expect("census at /Ox");
    assert_eq!(rows.len(), 3, "three bodies at /Ox too: {rows:?}");
    assert!(
        rows.iter().all(|r| !r.verdict.in_class()),
        "the PARSER must refuse this TU at /Ox — a `/O1` clause in the emitter \
         alone makes the census over-claim (#1638): {rows:?}"
    );
}
