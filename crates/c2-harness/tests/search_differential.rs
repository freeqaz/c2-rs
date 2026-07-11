//! T-A — the IL-space search prototype, gated against the live oracle.
//!
//! Proves the inversion loop **closes byte-exact** end-to-end on the MVP class:
//! build a solvable instance from a fixture (solution IL = its parse, target obj
//! = its replay), perturb the solution by a single K3a edit inside the move set,
//! then hill-climb back — every candidate judged by a REAL standalone-c2 replay —
//! and assert the climber recovers an IL whose obj is byte-exact
//! (timestamp-normalized) to the target.
//!
//! Toolchain-gated: skips cleanly (never fails) when the toolchain / strace /
//! mingw are absent, exactly like `edit_differential.rs`.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use c2_harness::search::{self, beam_search, Budget, MoveSet, Perturb, ReplayScorer};
use c2_il::{ExToken, IlModel};
use c2_obj::{ObjDiff, ObjImage};
use c2_reference::{CapturedReference, Toolchain};

const TIMEOUT: Duration = Duration::from_secs(60);
static COUNTER: AtomicU64 = AtomicU64::new(0);

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("fixtures/cpp")
        .join(name)
}

fn work(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let d = std::env::temp_dir().join(format!(
        "c2rs-search-{tag}-{}-{}-{}",
        std::process::id(),
        nanos,
        n
    ));
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// The toolchain + replay prerequisites, or `None` to skip cleanly.
fn ready() -> Option<Toolchain> {
    let tc = Toolchain::locate()?;
    if !tc.has_strace() {
        eprintln!("SKIP: strace absent");
        return None;
    }
    if !tc.has_mingw() {
        eprintln!("SKIP: i686-w64-mingw32-gcc absent");
        return None;
    }
    Some(tc)
}

/// A single d=1 add-term instance on `a+5` recovers byte-exact: seed `(a+5)+a`
/// (an inserted redundant term), climbed back to the `a+5` target obj by a real
/// c2-judged delete move.
#[test]
fn search_solves_addterm_d1_byte_exact() {
    let Some(tc) = ready() else { return };
    let w = work("addterm");

    let r = search::solve_instance(
        &tc,
        &fixture("mvp_edit_addk.cpp"),
        Perturb::AddTerm,
        1,
        &MoveSet::default(),
        &Budget::default(),
        &w,
        TIMEOUT,
    );

    assert!(r.error.is_none(), "instance errored: {:?}", r.error);
    let outcome = r.outcome.expect("add-term has a site on a+5");
    assert!(
        outcome.solved,
        "the inversion loop must close byte-exact: {outcome:?}"
    );
    // Recovery of a single inserted term is one accepted move (a delete).
    assert!(outcome.steps >= 1, "a real recovery took at least one move");
    // Every judgement was a real compile; the seed + neighborhood were bounded.
    assert!(outcome.compiles >= 1);

    std::fs::remove_dir_all(&w).ok();
}

/// The d=2 two-move descent — the instruction-aware-gradient rung. Seed
/// `((a+5)+a)+a` (two redundant `+a` terms inserted), climbed back to the `a+5`
/// target obj by TWO real c2-judged delete moves. This was the 0/6 stall under
/// the flat `.text`-word gradient (deleting one term did not raise the ratio
/// above the seed → LocalOptimum before the second delete); the instruction-aware
/// gradient (`insn_text_similarity`) grades the partial recovery strictly higher,
/// so greedy descent now reaches the byte-exact basin. Light: ONE d=2 instance
/// (the full `c2rs search eval --d 2` sweep is deferred until CPU frees).
#[test]
fn search_solves_addterm_d2_byte_exact() {
    let Some(tc) = ready() else { return };
    let w = work("addterm-d2");

    let r = search::solve_instance(
        &tc,
        &fixture("mvp_edit_addk.cpp"),
        Perturb::AddTerm,
        2,
        &MoveSet::default(),
        &Budget::default(),
        &w,
        TIMEOUT,
    );

    assert!(r.error.is_none(), "instance errored: {:?}", r.error);
    let outcome = r.outcome.expect("add-term d=2 has a site on a+5");
    assert!(
        outcome.solved,
        "the instruction-aware gradient must break the d=2 stall byte-exact: {outcome:?}"
    );
    // A genuine two-move descent: two redundant terms → two accepted deletes.
    assert!(
        outcome.steps >= 2,
        "d=2 recovery is a multi-step descent, got {} step(s)",
        outcome.steps
    );

    std::fs::remove_dir_all(&w).ok();
}

/// A d=1 literal-value nudge recovers byte-exact: perturb `a+5`'s literal to
/// `a+8`, climb back to `a+5` via a value move (the flat-gradient enumeration
/// case). Uses the last/only-function regime (`mvp_edit_addk`) so no `.gl`
/// bookkeeping beyond what the API discharges is needed.
#[test]
fn search_solves_litnudge_d1_byte_exact() {
    let Some(tc) = ready() else { return };
    let w = work("litnudge");

    let r = search::solve_instance(
        &tc,
        &fixture("mvp_edit_addk.cpp"),
        Perturb::LitNudge,
        1,
        &MoveSet::default(),
        &Budget::default(),
        &w,
        TIMEOUT,
    );

    assert!(r.error.is_none(), "instance errored: {:?}", r.error);
    let outcome = r.outcome.expect("lit-nudge has a site on a+5");
    assert!(
        outcome.solved,
        "literal-value recovery must close byte-exact: {outcome:?}"
    );

    std::fs::remove_dir_all(&w).ok();
}

/// First leaf-pair MUL site in `toks` (index of the `Mul` whose two immediate
/// operands are distinct single-token leaves) — the reorder move's target.
fn leaf_mul_site(toks: &[ExToken]) -> Option<usize> {
    let is_leaf = |t: &ExToken| {
        matches!(
            t,
            ExToken::Load(_) | ExToken::FloatLoad(_) | ExToken::Lit { .. }
        )
    };
    (2..toks.len()).find(|&i| {
        matches!(toks[i], ExToken::Mul)
            && is_leaf(&toks[i - 2])
            && is_leaf(&toks[i - 1])
            && toks[i - 2] != toks[i - 1]
    })
}

/// **Gate 4 — the byte-exact solve on a constructed float-MUL-reorder instance.**
///
/// Two IL bodies differing ONLY by MUL operand order (`x*y*z` vs `y*x*z`), both
/// compiled by real c2. Ordering-2 = the captured `mvp_fmul3` (`a*b*c`); ordering-1
/// = its inner MUL's two `FloatLoad` leaves swapped through the K3a splice. Seed the
/// search from ordering-1's IL with ordering-2's obj as the target, enable the
/// reorder move, and confirm the search reaches ordering-2's obj **byte-exact**
/// (`ObjImage::diff == Identical` on the timestamp-normalized objs).
///
/// **Live finding recorded by this test:** c2 **canonicalizes** commutative
/// multiply — ordering-1 replays to an obj that is *already* byte-exact to
/// ordering-2's (asserted directly below). The commutative reorder is therefore
/// obj-neutral on this toolchain (like a widened literal, P0.6a A): the byte-exact
/// terminal — the sole judge — fires on the seed itself, so the search closes at
/// step 0. The reorder move is still proven to *emit* ordering-2 exactly (asserted
/// via the neighborhood), i.e. it is a correct, guarded commutative primitive; it
/// just has no obj gap to bridge on this leaf-swap class. See
/// `docs/plans/il-witness/MUL_REORDER_MOVE.md`.
#[test]
fn search_solves_float_mul_reorder_byte_exact() {
    let Some(tc) = ready() else { return };
    let w = work("mul-reorder");

    // Ordering-2 (target): capture `a*b*c`, parse, render its obj to the fixed -Fo.
    let base = tc
        .capture_reference(&fixture("mvp_fmul3.cpp"), &w.join("cap"))
        .expect("capture mvp_fmul3");
    let solution = IlModel::parse(&base.bundle).expect("parse float-mul IL");
    let toks = solution.function_tokens(0).expect("token-addressable");
    let mi = leaf_mul_site(&toks).expect("a leaf-pair MUL site (a*b*c inner MUL)");

    let search_dir = w.join("search");
    let fo = search_dir.join("cand.obj");
    let target = tc
        .replay_within(&base, &w.join("tgt_il"), &fo, TIMEOUT)
        .expect("render ordering-2 (a*b*c) obj");

    // Ordering-1 (seed): swap the inner MUL's two FloatLoad leaves.
    let mut seed = solution.clone();
    seed.splice_function_tokens(0, mi - 2..mi, vec![toks[mi - 1].clone(), toks[mi - 2].clone()])
        .expect("swap the two MUL leaves");
    assert_ne!(
        seed.encode().get("ex"),
        solution.encode().get("ex"),
        "the reorder must change the seed `.ex`"
    );

    // Live obj-neutrality: ordering-1's own replay is already byte-exact to
    // ordering-2 (c2 canonicalizes commutative MUL). Replay to the SAME -Fo so the
    // embedded S_OBJNAME matches; compare on the normalized objs.
    let seed_cap = CapturedReference {
        bundle: seed.encode(),
        ..base.clone()
    };
    let seed_obj = tc
        .replay_within(&seed_cap, &w.join("seed_il"), &fo, TIMEOUT)
        .expect("render ordering-1 (b*a*c) obj");
    assert_eq!(
        ObjImage::diff(&seed_obj, &target),
        ObjDiff::Identical,
        "c2 must canonicalize the commutative MUL reorder (obj-neutral)"
    );

    // The reorder move emits ordering-2 exactly (the move IS a correct commutative
    // primitive, even though the obj gap it would bridge is nil here).
    let regenerates_ordering2 = MoveSet::default()
        .with_mul_reorder()
        .neighbors(&seed)
        .iter()
        .any(|(_l, cand)| cand.encode().get("ex") == solution.encode().get("ex"));
    assert!(
        regenerates_ordering2,
        "the reorder move must emit ordering-2's IL as a neighbor of ordering-1"
    );

    // The search closes byte-exact (the sole judge — full timestamp-normalized
    // ObjImage::diff Identical — fires; here on the seed itself, canonicalization).
    let mut scorer = ReplayScorer::new(&tc, &base, target, search_dir.clone(), TIMEOUT);
    let outcome = beam_search(
        &seed,
        &MoveSet::default().with_mul_reorder(),
        &mut scorer,
        &Budget::default(),
    );
    assert!(
        outcome.solved,
        "the reorder instance must close byte-exact: {outcome:?}"
    );

    std::fs::remove_dir_all(&w).ok();
}
