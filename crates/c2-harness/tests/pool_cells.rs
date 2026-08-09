//! **The three `w-pool` cells, and the one thing that separates each from the
//! last** — lane `w-pool`, board **#2562**–**#2564**.
//!
//! # What this pins that no fixture verdict can
//!
//! `src/system/utl/Pool.cpp` is one of **three** TUs in the 878-TU workload
//! whose entire `decode_causes` set is `{body-out-of-class}` (#2506) — the
//! reader is the only gate that fires, with no binding gap, no
//! writer-composition gap and no whole-obj obligation standing behind it. Its
//! reference obj carries **zero relocations, no `.pdata`, no `$M`/`$T` label
//! symbol and no `_fltused`**.
//!
//! That reads like *"one reader repair converts it"*, and it is not. The three
//! cells below are `?Free@Pool@@QAAXPAX@Z` built up one construct at a time, and
//! each step moves the verdict:
//!
//! ```text
//!   A  *(void**)v = p->mFree;                       IN CLASS  store-leaf
//!   B  A + one more store                           BLOCKED   expr-op-0x27
//!   C  B + a null guard whose arm is a bare return  BLOCKED   expr-brtrue
//! ```
//!
//! **A → B is the must-fail mutation for B's fence** and **B → C is the
//! must-fail mutation for C's**, and both endpoints ship, so neither `_neg` can
//! go inert without a test failing: if B's clause were repaired, B's key would
//! stop being `expr-op-0x27` and this file says so; if C's guard were admitted,
//! C's key would fall back to B's and the distinctness assertion fires.
//!
//! # Why the sources are `include_str!`-ed
//!
//! `w-fence2` §5.1's rule: a cell that re-types its subject grades a copy. Every
//! body below is read out of `fixtures/cpp/` so a fixture cannot drift from the
//! assertion that claims to grade it.
//!
//! `SKIP: toolchain absent` when there is no toolchain, like every other
//! integration test here.

use std::path::PathBuf;

use c2_reference::Toolchain;

/// The workload's own profile minus the `/I` paths a standalone cell cannot
/// use. **`/O1`, deliberately**: it implies `/Gy`, which is the regime the
/// 878-TU scan lives in and the one `Pool.obj`'s three COMDAT `.text` sections
/// belong to.
const FLAGS: [&str; 8] = [
    "/nologo", "/wd4355", "/wd4164", "/c", "/GR", "/O1", "/Oi", "/EHsc",
];

const CELL_A: &str = include_str!("../../../fixtures/cpp/wpool_store_leaf_member_value.cpp");
const CELL_B: &str = include_str!("../../../fixtures/cpp/wpool_store_run_member_value_neg.cpp");
const CELL_C: &str = include_str!("../../../fixtures/cpp/wpool_guard_bclr_fold_neg.cpp");

fn work(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("c2rs-poolcell-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Capture one source at [`FLAGS`] and return `(its per-function census, its
/// `DecodeCauses`)`. Both come off the **same** capture, so a claim about the
/// census key and a claim about the gate cause cannot be about two compilations.
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

/// The one census row each of these single-function cells has.
fn only(rows: &[c2_il::func::FnCensus], tag: &str) -> c2_il::func::FnCensus {
    assert_eq!(rows.len(), 1, "cell `{tag}` must be one function: {rows:?}");
    rows[0].clone()
}

/// **THE LADDER, AS THREE CELLS: each adds one construct and each moves the
/// verdict.**
///
/// This is the executable form of `w-pool`'s decline. A single refusal fixture
/// would have made `Pool.cpp` look one repair from a match; its `?Free` alone
/// owes **two** independent rungs, and its constructor owes four more that none
/// of these cells touches.
#[test]
fn the_three_pool_cells_each_move_the_verdict_by_one_construct() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };

    // A — the CONTROL. A store whose *value* is a member load, through a cast
    // base, is in class and byte-exact: `store-leaf` admits exactly this.
    let (a_rows, a_causes) = cell(&tc, "a", CELL_A);
    let a = only(&a_rows, "a");
    assert!(
        a.verdict.in_class(),
        "cell A is the control and must be IN CLASS: {:?}",
        a.verdict
    );
    assert_eq!(a.verdict.key(), "store-leaf", "…as `store-leaf`: {a:?}");
    assert!(
        a_causes.decodes && a_causes.causes.is_empty(),
        "cell A decodes, so its cause list is empty: {a_causes:?}"
    );

    // B — A plus ONE MORE STORE. Nothing else changes: same struct, same two
    // operands, same cast, still `cflow-straight`. The run refuses because
    // `leaf_store::parse_store_stmt` does not admit a member LOAD in the value
    // position of a multi-store run, and the walk stops on the byte-offset add.
    let (b_rows, b_causes) = cell(&tc, "b", CELL_B);
    let b = only(&b_rows, "b");
    assert_eq!(
        b.verdict.key(),
        "expr-op-0x27",
        "cell B must stop at the run's value clause: {b:?}"
    );
    assert_eq!(
        b.cflow, "cflow-straight",
        "…and it must have NO control flow, so the key cannot be a branch \
         fall-through (CEILING §11.4 item 5, #1416): {b:?}"
    );
    assert_eq!(
        b_causes.first,
        Some(c2_il::func::cause::BODY_DECODE),
        "the TU-level gate stops at the body, nowhere earlier: {b_causes:?}"
    );

    // C — B plus a null guard whose arm is a bare `return`. This body IS
    // `?Free@Pool@@QAAXPAX@Z`. Its key is DIFFERENT from B's, which is what says
    // the guard is a second, independent rung rather than the same one again.
    let (c_rows, c_causes) = cell(&tc, "c", CELL_C);
    let c = only(&c_rows, "c");
    assert_eq!(
        c.verdict.key(),
        "expr-brtrue",
        "cell C must stop at the guard: {c:?}"
    );
    assert_eq!(
        c.cflow, "cflow-if-1",
        "…and it is the one cell here that HAS control flow: {c:?}"
    );
    assert_eq!(
        c_causes.first,
        Some(c2_il::func::cause::BODY_DECODE),
        "still only the body: {c_causes:?}"
    );

    // The three verdicts are pairwise distinct. Stated as a set so a future edit
    // that collapses two cells into one is a failure and not a silent loss.
    let keys: std::collections::BTreeSet<String> =
        [a.verdict.key(), b.verdict.key(), c.verdict.key()].into_iter().collect();
    assert_eq!(keys.len(), 3, "three cells, three distinct verdicts: {keys:?}");
}

/// **The binding predicate PASSES on all three cells, and on `Pool.cpp` itself**
/// — `CEILING.md` §11.4 item 8, checked FIRST rather than assumed.
///
/// Two consecutive conversion lanes (`w-mmioclose` #2406, `w-vec` #2503) priced
/// codegen on a TU whose gate binds nothing, by two different mechanisms. The
/// cheap check is one line, and it is this: `bind-record-count-ne-segments` and
/// `bind-offset-ne-segment-start` are the two clauses `Bindings::per_record`
/// applies, so their **absence** from the cause list is the statement that the
/// binding succeeded.
///
/// It matters here in the opposite direction from those two lanes: these cells
/// bind, so the refusals asserted above really are the reader's.
#[test]
fn every_pool_cell_binds_its_records_before_the_body_is_looked_at() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    for (tag, body) in [("bind-a", CELL_A), ("bind-b", CELL_B), ("bind-c", CELL_C)] {
        let (_, c) = cell(&tc, tag, body);
        assert_eq!(c.segments, 1, "cell `{tag}`: one `.ex` body: {c:?}");
        for stop in [
            c2_il::func::cause::BIND_COUNT,
            c2_il::func::cause::BIND_OFFSET,
            c2_il::func::cause::GL_26_INTRODUCED,
            c2_il::func::cause::GL_NAME_NOT_MANGLED,
            c2_il::func::cause::DRECTVE,
        ] {
            assert!(
                !c.causes.contains(&stop),
                "cell `{tag}`: the gate must not stop at `{stop}` — the whole \
                 claim of this lane is that the BODY is the only gate: {c:?}"
            );
        }
    }
}

/// **The `_neg` cells must not be inert, and the proof is the cell one step
/// back.**
///
/// Six of the last ten lanes shipped a `_neg` that was inert or confounded. Here
/// each negative's fence is proven by a **shipped** neighbour that differs from
/// it by exactly one construct and lands somewhere else — so an edit that
/// silently admitted B's clause or C's guard cannot leave this file green.
///
/// Stated as an ordering rather than as three separate facts: the cells are a
/// chain, and the chain's shape is that in-class comes first and every step
/// after it is blocked at its own key.
#[test]
fn each_pool_negative_is_fenced_by_its_own_shipped_neighbour() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let (a, _) = cell(&tc, "fence-a", CELL_A);
    let (b, _) = cell(&tc, "fence-b", CELL_B);
    let (c, _) = cell(&tc, "fence-c", CELL_C);
    let (a, b, c) = (
        only(&a, "fence-a"),
        only(&b, "fence-b"),
        only(&c, "fence-c"),
    );

    assert!(a.verdict.in_class(), "A in class: {a:?}");
    assert!(!b.verdict.in_class(), "B blocked: {b:?}");
    assert!(!c.verdict.in_class(), "C blocked: {c:?}");
    assert_ne!(
        b.verdict.key(),
        c.verdict.key(),
        "B and C must be blocked at DIFFERENT keys, or C adds nothing and \
         `Pool::Free` looks one repair from a match when it owes two"
    );
    // …and the direction: adding a construct never makes a body more acceptable.
    assert!(
        b.seg_len > a.seg_len && c.seg_len > b.seg_len,
        "each cell adds IL rather than replacing it: {} -> {} -> {}",
        a.seg_len,
        b.seg_len,
        c.seg_len
    );
}

/// **The anti-drift invariant, on real captures.** `DecodeCauses::decodes` is
/// read from the real [`c2_il::IlBundle::decodes`] and `causes` from the
/// re-asked predicates; `causes.is_empty() == decodes` is what stops the
/// diagnostic from becoming a second, disagreeing gate. `c2-il` asserts it on
/// synthetic bundles and `gate_cause.rs` on `w-vec`'s three sources; this adds
/// the three shapes `w-pool` cut, which are the first cells in the corpus whose
/// only refusal is the body.
#[test]
fn the_diagnostic_agrees_with_the_gate_on_every_pool_cell() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    for (tag, body) in [("inv-a", CELL_A), ("inv-b", CELL_B), ("inv-c", CELL_C)] {
        let (_, c) = cell(&tc, tag, body);
        assert_eq!(
            c.causes.is_empty(),
            c.decodes,
            "cell `{tag}`: the diagnostic and the gate disagree: {c:?}"
        );
        assert_eq!(
            c.first.is_none(),
            c.decodes,
            "cell `{tag}`: a first cause exists iff the bundle is refused: {c:?}"
        );
    }
}
