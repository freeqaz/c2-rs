//! **The positive control for the `fence-blocks-exact` counter** — lane
//! `w-fencecount`.
//!
//! `GapReport::fence_blocks` answers CLAUDE.md's two-sided fence-pricing
//! question per named fence: how many TUs does this fence hold out of `match`
//! **alone**, and how many of those already hold nothing but byte-exact emitted
//! bodies. On the 878-TU workload every one of those cells reads **0** — the one
//! TU that ever had the shape (`src/xdk/LIBCMT/vsnprnc.cpp`,
//! `docs/rungs/2026-08-09-w-vsnprnc.md` §1) was converted by `w-fence2` when it
//! narrowed the fence (board #2470), and all 845 TUs held today carry two or
//! more decode causes.
//!
//! **So the workload cannot be this counter's control.** A counter whose only
//! positive reading has been paid off reads exactly like a counter that is not
//! wired up, which is this repo's most-recorded defect
//! (`docs/STATUS.md` trap 5, ~15 instances). The control is
//! `fixtures/cpp/wfcnt_fence_holds_exact.cpp`, run through the **real** gap
//! pipeline — capture with real `c2.dll` under wibo, the port, the byte judge —
//! and every premise of the reading is asserted separately:
//!
//! 1. the run **graded something**, and it is the fixture, by name;
//! 2. its class is `vocab-gap`;
//! 3. its decode causes are exactly `[locally-defined-callee]` (sole-blocker);
//! 4. the per-function judge grades `fnbyte-exact == fnbyte-denominator == 2`
//!    (the price's expensive half — and, since c2 inlines this callee, also the
//!    proof that the port's mechanism-I splice reproduced c2's own bytes);
//! 5. `fence_blocks()` reads `sole 1 / exact 1 / bodies 2 / first 0`;
//! 6. the two published `gap-metric` keys carry the same two numbers.
//!
//! Each has its own failure message, and the premises are asserted **before**
//! the reading they support: an early guard that fires for an unrelated reason
//! must not be able to make the later assertions unreachable, which is the
//! lane-registry defect (`ROADMAP.md` §9.18.8) this ordering exists to avoid.
//!
//! `SKIP: toolchain absent` when there is no toolchain, like every other
//! integration test here — never a panic.

use std::path::PathBuf;

use c2_harness::gap::{gap_scan, GapConfig, TuClass};
use c2_reference::Toolchain;

/// The `/O1` profile, the same one `tests/gate_cause.rs` uses and for the same
/// reason: `/O1` implies `/Gy`, which is the regime the 878-TU scan lives in and
/// the only mode where the fence's exemption is live at all (board #1638). At
/// `/Ox` this fixture is `NotImplemented` and the cell would grade a different
/// question.
const FLAGS: [&str; 8] = c2_harness::testsupport::WORKLOAD_FLAGS;

/// The fixture, relative to the repo root — which is also the scan's `cwd`, so
/// the string below is both the source argument and the `src` the report keys
/// on.
const FIXTURE: &str = "fixtures/cpp/wfcnt_fence_holds_exact.cpp";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

#[test]
fn the_inline_fence_holds_one_all_exact_tu_and_the_counter_says_so() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let root = repo_root();
    let work = std::env::temp_dir().join(format!("c2rs-fencecount-{}", std::process::id()));
    std::fs::create_dir_all(&work).expect("scratch dir");
    let cfg = GapConfig {
        sources: vec![FIXTURE.to_string()],
        flags: FLAGS.iter().map(|s| s.to_string()).collect(),
        cwd: Some(root),
        limit: None,
        jobs: 1,
        replay_every: 0,
        jsonl: None,
        fndiff_jsonl: None,
        factors_tsv: None,
        work: work.clone(),
        // No cache: this cell must grade a capture it made, not one a previous
        // run left behind under a key it cannot see.
        cache: None,
        validate_cache: 0,
    };
    let report = gap_scan(&tc, &cfg, &|_, _, _| {}).expect("gap scan");
    let _ = std::fs::remove_dir_all(&work);

    // (1) POSITIVE ON CONTENT: the run graded a TU, and it is the fixture by
    // name. Never "no failures were observed" — an empty run must fail here.
    assert_eq!(
        report.results.len(),
        1,
        "the scan must have graded exactly one TU; it graded {} — a run that \
         grades nothing must fail this cell, not pass it silently",
        report.results.len()
    );
    let r = &report.results[0];
    assert_eq!(
        r.src, FIXTURE,
        "the graded TU must be the control fixture by name, not whatever else \
         the scan happened to reach; got `{}`",
        r.src
    );

    // (2) The class. `capture-fail` here would mean the cell graded nothing at
    // all while every later assertion still had numbers to read.
    assert_eq!(
        r.class,
        TuClass::VocabGap,
        "the control must be held OUT of `match` as `vocab-gap` (the gate \
         refused it); it graded `{}` — detail: {}",
        r.class.label(),
        r.detail
    );

    // (3) The sole-blocker premise: this TU is held by the inline fence and by
    // nothing else the diagnostic models.
    assert_eq!(
        r.gate_causes,
        vec![c2_il::func::cause::LOCAL_CALLEE.to_string()],
        "the control's decode causes must be exactly [{}] — a second cause \
         means the fixture no longer isolates the fence and the `sole` reading \
         below would be about something else; got {:?}",
        c2_il::func::cause::LOCAL_CALLEE,
        r.gate_causes
    );

    // (4) The expensive half of the price, from the oracle's own byte test:
    // every emitted body of this refused TU is byte-exact.
    let den = r.emit.get("fnbyte-denominator").copied().unwrap_or(0);
    let exact = r.emit.get("fnbyte-exact").copied().unwrap_or(0);
    assert_eq!(
        den, 2,
        "the reference obj must carry both bodies as emitted `.text` COMDATs \
         (denominator 2); got {den} — with no denominator, exactness is the \
         vacuous reading this counter refuses to make"
    );
    assert_eq!(
        exact, den,
        "every emitted body must be byte-exact against real c2 ({exact} of \
         {den}) — this is the half that makes the fence's refusal a PRICE, and \
         if c2 stopped inlining this callee (or the splice stopped reproducing \
         it) the counter would be measuring a different TU"
    );

    // (5) The counter itself.
    let fb = report.fence_blocks();
    let row = fb
        .per_cause
        .get(c2_il::func::cause::LOCAL_CALLEE)
        .copied()
        .unwrap_or_default();
    assert_eq!(
        row.sole, 1,
        "the inline fence must be counted as the SOLE blocker of one TU; got \
         {} — the whole per-cause map: {:?}",
        row.sole, fb.per_cause
    );
    assert_eq!(
        row.exact_tus, 1,
        "…and that TU must land in `fence-blocks-exact`, because all its \
         emitted bodies are byte-exact; got {}",
        row.exact_tus
    );
    assert_eq!(
        row.exact_bodies, 2,
        "…with the BODY count beside the TU count (arity: entities against \
         their contents); got {}",
        row.exact_bodies
    );
    assert_eq!(
        row.first_of_multi, 0,
        "a sole-blocked TU must never also be counted as a first-of-multi row, \
         or the totality identity double-counts it; got {}",
        row.first_of_multi
    );

    // …and the controls that ride with it, over this one-TU population.
    assert_eq!(
        (fb.held_tus, fb.cause_firings),
        (1, 1),
        "held TUs and cause firings must both read 1 on a single sole-blocked \
         TU (totality and arity); got {:?}",
        (fb.held_tus, fb.cause_firings)
    );
    assert_eq!(
        (fb.residue_no_cause, fb.arity_broken, fb.class_disagree, fb.on_match_tu),
        (0, 0, 0, 0),
        "every known-answer-0 control must read 0 on the control fixture; got \
         (residue_no_cause, arity_broken, class_disagree, on_match_tu) = {:?}",
        (fb.residue_no_cause, fb.arity_broken, fb.class_disagree, fb.on_match_tu)
    );

    // (6) The published keys — what a rung quotes and what `status.sh` could
    // collect. A reading that exists only inside the struct is not the
    // instrument.
    let m: std::collections::BTreeMap<&str, String> = report.metrics().into_iter().collect();
    assert_eq!(
        m.get("fence-blocks-exact:locally-defined-callee").map(String::as_str),
        Some("1"),
        "`gap-metric fence-blocks-exact:locally-defined-callee` must publish \
         the TU count; the fence keys present were: {:?}",
        m.keys().filter(|k| k.starts_with("fence-")).collect::<Vec<_>>()
    );
    assert_eq!(
        m.get("fence-blocks-exact-bodies:locally-defined-callee").map(String::as_str),
        Some("2"),
        "`gap-metric fence-blocks-exact-bodies:locally-defined-callee` must \
         publish the body count beside it"
    );
    assert_eq!(
        m.get("fence-blocks-sole:locally-defined-callee").map(String::as_str),
        Some("1"),
        "`gap-metric fence-blocks-sole:locally-defined-callee` must publish the \
         sole-blocker count"
    );
}
