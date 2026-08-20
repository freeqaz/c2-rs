use super::classify::normalize_cl_error;
use super::*;

#[test]
fn cl_error_normalization_extracts_code() {
    let blob = "capture failed\n  stdout:\n    x.cpp\n    src/x.h(12): fatal error C1083: Cannot open include file: 'foo.h': No such file\n";
    let (key, detail) = normalize_cl_error(blob);
    assert_eq!(key, "C1083");
    assert!(detail.contains("foo.h"));
}

#[test]
fn cl_error_normalization_survives_codeless_blobs() {
    let (key, _) = normalize_cl_error("wibo: something exploded\n");
    assert_eq!(key, "wibo: something exploded");
}

fn mk_report(results: Vec<TuResult>) -> GapReport {
    GapReport {
        results,
        provenance: None,
        cache: crate::capture_cache::CacheStats::default(),
    }
}

fn mk(reason: &str) -> TuResult {
    TuResult {
        src: "s".into(),
        class: TuClass::CodegenGap,
        reason: reason.into(),
        detail: String::new(),
        ex_len: 0,
        fn_names: 0,
        replay_ok: None,
        fn_total: 0,
        fn_in_class: 0,
        fn_blockers: BTreeMap::new(),
        fn_frames: BTreeMap::new(),
        fn_cflow: BTreeMap::new(),
        fn_cflow_off: Default::default(),
        fn_cfg_admit: Default::default(),
        fn_eh: BTreeMap::new(),
        fn_dispatch: BTreeMap::new(),
        fn_complete: BTreeMap::new(),
        fn_prod: BTreeMap::new(),
        fn_gate_refusals: BTreeMap::new(),
        bind_checks: BTreeMap::new(),
        gate_cause: None,
        gate_causes: Vec::new(),
        gl_body_starts: None,
        selective_bind: None,
        emit: BTreeMap::new(),
        emit_blockers: BTreeMap::new(),
        emit_witness: Vec::new(),
        fndiff: Vec::new(),
        plan: Default::default(),
    }
}

#[test]
fn report_ranks_reasons_by_count() {
    let rep = mk_report(vec![mk("b"), mk("a"), mk("b")]);
    assert_eq!(
        rep.top_reasons(TuClass::CodegenGap),
        vec![("b".to_string(), 2), ("a".to_string(), 1)]
    );
    assert_eq!(rep.count(TuClass::Match), 0);
}

/// **The witness list ranks by frequency across TUs, and its per-bucket
/// symbol total is the number the scan's own counter must equal** (board
/// #159). Both halves matter: a list ranked per TU would name the largest
/// TU's symbols rather than the workload's, and a list whose total does not
/// reconcile with `emit-unbound-*` is a second measurement of the residue —
/// which is exactly the defect `ROADMAP.md` §10.14 records.
#[test]
fn the_witness_list_ranks_across_tus_and_reconciles_with_its_counter() {
    let row = |bucket: &str, name: &str, in_gl: bool| WitnessRow {
        bucket: bucket.into(),
        name: name.into(),
        in_gl_runs: in_gl,
        in_gl_index: in_gl,
    };
    let ord = "emit-unbound-no-record|ordinary";
    let spc = "emit-unbound-no-record|special-generated";
    let mut a = mk("x");
    a.src = "a.cpp".into();
    a.emit_witness = vec![row(ord, "?rare@@YAXXZ", false), row(ord, "?common@@YAXXZ", true)];
    a.emit.insert(ord.into(), 2);
    let mut b = mk("x");
    b.src = "b.cpp".into();
    b.emit_witness = vec![row(ord, "?common@@YAXXZ", true), row(spc, "??_7C@6B@", false)];
    b.emit.insert(ord.into(), 1);
    b.emit.insert(spc.into(), 1);

    let rep = mk_report(vec![a, b]);
    let buckets = witness_buckets(&rep.results);
    assert_eq!(buckets.len(), 2, "one entry per bucket that collected a row");
    let o = &buckets[0];
    assert_eq!(o.bucket, ord, "buckets rank by symbol count, largest first");
    assert_eq!((o.symbols, o.tus, o.names.len(), o.in_gl_runs, o.in_gl_index), (3, 2, 2, 2, 2));
    assert_eq!(
        o.names[0],
        ("?common@@YAXXZ".to_string(), 2, 2, "a.cpp".to_string()),
        "the name seen in two TUs outranks the name seen once, and carries an example TU"
    );
    assert_eq!(o.names[1].1, 1);
    assert_eq!(buckets[1].symbols, 1);

    // The reconciliation the report prints: rows summed per bucket equal the
    // counter the same loop incremented. This is the check §10.14's reader
    // could not have passed, because it had no counter to reconcile against.
    for b in &buckets {
        assert_eq!(
            b.symbols,
            rep.emit_total(&b.bucket),
            "{}: witness rows must equal the scan's own counter",
            b.bucket
        );
    }
    let rows: usize = buckets.iter().map(|b| b.symbols).sum();
    assert_eq!(rows, 4, "every row lands in exactly one bucket");
}

/// A TU whose emitted census is spelled out: `emitted` symbols, of which
/// `bound` bound, `in_class` in class, and `gen`/`other` in the two residue
/// buckets.
fn mk_emit(
    class: TuClass,
    emitted: usize,
    bound: usize,
    in_class: usize,
    gen: usize,
    other: usize,
) -> TuResult {
    let mut r = mk("x");
    r.class = class;
    r.fn_total = emitted;
    r.fn_in_class = in_class;
    for (k, n) in [
        ("emit-emitted", emitted),
        ("emit-bound", bound),
        ("emit-in-class", in_class),
        ("emit-residue-generated", gen),
        ("emit-residue-unbound", other),
    ] {
        if n > 0 {
            r.emit.insert(k.into(), n);
        }
    }
    r
}

/// The read-out and its residue aggregate across TUs, and the denominator is
/// the emitted count — never reduced by what failed to bind, which would
/// inflate the ratio.
#[test]
fn the_emitted_census_aggregates_and_keeps_its_denominator_whole() {
    let a = mk_emit(TuClass::VocabGap, 100, 90, 20, 4, 6);
    let b = mk_emit(TuClass::VocabGap, 50, 45, 5, 1, 4);
    let rep = mk_report(vec![a, b]);
    assert_eq!(
        rep.emit_coverage(),
        (25, 150),
        "the read-out is in-class over EMITTED, not over bound"
    );
    assert_eq!(
        rep.emit_residue(),
        (5, 10),
        "the residue splits into generated-with-no-body and unexplained"
    );
    assert_eq!(
        rep.emit_total("emit-bound") + rep.emit_residue().0 + rep.emit_residue().1,
        150,
        "bound + residue must account for every emitted symbol"
    );
}

/// GROUND TRUTH, and its NEGATIVE CONTROL. On a byte-exact TU the oracle has
/// already graded the whole symbol table, so the binding's answer there is
/// checkable: every emitted symbol must bind to an in-class row.
///
/// The guard's quantity — one `match` TU with 40 emitted functions — is held
/// FIXED across the two halves; only how many of them the binding claimed
/// moves. Without that, the second half could pass by the TU no longer being
/// a `match` at all, and the assertion under test would never run.
#[test]
fn a_match_tu_whose_emitted_symbols_do_not_all_bind_is_a_binding_defect() {
    let good = mk_emit(TuClass::Match, 40, 40, 40, 0, 0);
    let rep = mk_report(vec![good, mk_emit(TuClass::VocabGap, 100, 50, 10, 20, 30)]);
    assert_eq!(rep.count(TuClass::Match), 1, "control: one byte-exact TU");
    assert_eq!(
        rep.emit_match_tu_residue(),
        0,
        "control: a byte-exact TU with every symbol bound and in class reads 0"
    );

    let bad = mk_emit(TuClass::Match, 40, 37, 37, 0, 3);
    let rep = mk_report(vec![bad, mk_emit(TuClass::VocabGap, 100, 50, 10, 20, 30)]);
    assert_eq!(
        rep.count(TuClass::Match),
        1,
        "the mutation must not change the number of byte-exact TUs — otherwise \
         this control tests the class filter, not the binding"
    );
    assert_eq!(
        rep.emit_match_tu_residue(),
        3,
        "three emitted symbols the port provably emitted correctly did not bind \
         to an in-class row: the binding is wrong there, and it must say so"
    );
}

/// The near-match table is the payoff metric's leading indicator, and a
/// `capture-fail` TU must not appear in it: it has no census, so its distance
/// of 0 means "never measured", not "nearly done".
#[test]
fn the_near_match_table_excludes_the_tus_that_were_never_measured() {
    let mut near = mk_emit(TuClass::VocabGap, 10, 10, 9, 0, 0);
    near.src = "near.cpp".into();
    near.fn_total = 10;
    near.fn_in_class = 9;
    let mut far = mk_emit(TuClass::VocabGap, 500, 400, 10, 0, 0);
    far.src = "far.cpp".into();
    far.fn_total = 500;
    far.fn_in_class = 10;
    let mut unmeasured = mk("c1083");
    unmeasured.class = TuClass::CaptureFail;
    unmeasured.src = "never-captured.cpp".into();
    let rep = mk_report(vec![near, far, unmeasured]);
    let got: Vec<&str> = rep.near_match_tus(100).iter().map(|r| r.src.as_str()).collect();
    assert_eq!(
        got,
        vec!["near.cpp"],
        "only the measured TU within 100 blocked functions may appear"
    );
}

/// The two distances measure different populations and must be allowed to
/// disagree — the whole reason for publishing both. Modelled on the real
/// `src/system/math/Rand2.cpp`: 13 `.ex` bodies, 5 in class (8 blocked
/// bodies), but only 2 emitted functions of which 1 is in class, so **2**
/// by the measure the goal is written in. A leading indicator that ranked
/// this TU at 8 while another at 8-blocked-bodies-and-8-blocked-emitted also
/// read 8 is ranking two very different amounts of work the same.
#[test]
fn the_two_distances_are_different_populations_and_may_disagree() {
    let mut rand2 = mk_emit(TuClass::VocabGap, 2, 2, 1, 0, 0);
    rand2.src = "Rand2.cpp".into();
    rand2.fn_total = 13;
    rand2.fn_in_class = 5;
    let mut even = mk_emit(TuClass::VocabGap, 9, 9, 1, 0, 0);
    even.src = "even.cpp".into();
    even.fn_total = 9;
    even.fn_in_class = 1;
    let rep = mk_report(vec![rand2, even]);

    let by_body: Vec<&str> = rep.near_match_tus(8).iter().map(|r| r.src.as_str()).collect();
    assert_eq!(
        by_body,
        vec!["Rand2.cpp", "even.cpp"],
        "by blocked BODIES both TUs are 8 away and the measure cannot tell them apart"
    );
    let by_emit: Vec<&str> = rep
        .near_match_tus_emitted(2)
        .iter()
        .map(|r| r.src.as_str())
        .collect();
    assert_eq!(
        by_emit,
        vec!["Rand2.cpp"],
        "by blocked EMITTED functions Rand2 is 2 away and the other is 8 — if this \
         ever equals the body measure, one of the two is not reading what it says"
    );
}

/// The emit-set ceiling, and the control that makes it a measurement.
///
/// `PortC2` emits one `.text` COMDAT per `.ex` function segment and has no
/// emit-set model, so a TU whose segment count differs from its obj's
/// COMDAT-leader count cannot be byte-exact however good its codegen is.
/// The invariant that keeps that reading honest is that **no matching TU may
/// violate it** — a byte-exact obj cannot carry a different number of
/// `.text` COMDATs than the port wrote. The mutation below is exactly that
/// violation and it must be counted, otherwise the ceiling is an argument
/// rather than a control.
#[test]
fn the_emit_set_ceiling_is_bounded_by_an_invariant_that_can_go_red() {
    // A matching TU: 2 bodies, 2 emitted COMDATs, both in class.
    let mut ok = mk_emit(TuClass::Match, 2, 2, 2, 0, 0);
    ok.src = "Spew.cpp".into();
    ok.fn_total = 2;
    ok.fn_in_class = 2;
    // Reachable but not there yet: counts agree, one body still blocked.
    let mut near = mk_emit(TuClass::VocabGap, 1, 1, 0, 0, 0);
    near.src = "xboxheap.cpp".into();
    near.fn_total = 1;
    near.fn_in_class = 0;
    // UNREACHABLE: 802 `.ex` bodies against 2 emitted COMDATs. Every emitted
    // function is already in class, so BOTH distance measures call it near;
    // the port would still write 802 sections against c2's 2.
    let mut vec_cpp = mk_emit(TuClass::VocabGap, 2, 2, 2, 0, 0);
    vec_cpp.src = "vec.cpp".into();
    vec_cpp.fn_total = 802;
    vec_cpp.fn_in_class = 237;
    let rep = mk_report(vec![ok, near, vec_cpp]);

    let reach: Vec<&str> = rep
        .emit_set_reachable_tus()
        .iter()
        .map(|r| r.src.as_str())
        .collect();
    assert_eq!(
        reach,
        vec!["Spew.cpp", "xboxheap.cpp"],
        "vec.cpp has zero blocked EMITTED functions and is still unreachable — \
         that is the point of the ceiling"
    );
    assert_eq!(
        rep.emit_set_violations(),
        0,
        "a matching TU whose counts disagree would mean fn_total and emit-emitted \
         are not counting what the ceiling says they count"
    );

    // The control: make a MATCHING TU violate it. If this does not go red the
    // invariant cannot see the defect it exists for (#145).
    let mut bad = mk_emit(TuClass::Match, 2, 2, 2, 0, 0);
    bad.src = "Spew.cpp".into();
    bad.fn_total = 5;
    bad.fn_in_class = 2;
    let rep = mk_report(vec![bad]);
    assert_eq!(
        rep.count(TuClass::Match),
        1,
        "the mutation must not change the number of byte-exact TUs — otherwise this \
         control tests the class filter, not the emit-set reading"
    );
    assert_eq!(
        rep.emit_set_violations(),
        1,
        "a byte-exact obj with 5 `.ex` segments and 2 `.text` COMDATs is impossible; \
         the invariant must say so"
    );
}

/// A TU with the five Phase 7 factors set explicitly, through the same keys
/// `scan_one` writes. `e` is factor E, whole-TU acceptance (board #179).
fn mk_factors(
    class: TuClass,
    src: &str,
    a: bool,
    b: bool,
    c: bool,
    d: bool,
    e: bool,
) -> TuResult {
    let mut r = mk("x");
    r.class = class;
    r.src = src.into();
    // `emit-gate-segments-known` and `emit-emitted` are the populations the
    // factors are defined over; a TU missing them is UNMEASURED, not false.
    r.emit.insert("emit-gate-segments-known".into(), 1);
    r.emit.insert("emit-emitted".into(), 0);
    r.emit.insert("emit-sec-readable".into(), 1);
    for (k, on) in [
        ("emit-set-ceiling-gate", a),
        ("emit-set-ceiling-today", b),
        ("emit-sec-reachable", c),
        ("emit-class-complete", d),
        ("emit-whole-tu-any", e),
    ] {
        if on {
            r.emit.insert(k.into(), 1);
        }
    }
    r
}

/// **The factorization is a JOINT, and the joint is not the product of its
/// marginals** (`ROADMAP.md` §8.6 — the standing rule this report had no tool
/// for until the per-row dump, and now has one for at TU level).
///
/// The four TUs below give marginals A = B = C = D = 3 of 4, which multiplied
/// against 4 TUs would "predict" ≈1.3 — and the measured joint is **0**,
/// because each TU fails a different factor. A report that printed only the
/// four counts would let a reader do that multiplication and be wrong in the
/// flattering direction.
#[test]
fn the_factorization_is_a_joint_and_not_a_product_of_marginals() {
    let rep = mk_report(vec![
        mk_factors(TuClass::VocabGap, "a.cpp", false, true, true, true, false),
        mk_factors(TuClass::VocabGap, "b.cpp", true, false, true, true, false),
        mk_factors(TuClass::VocabGap, "c.cpp", true, true, false, true, false),
        mk_factors(TuClass::VocabGap, "d.cpp", true, true, true, false, false),
    ]);
    let [a, b, c, d, e, _a_lo, bc, _abc, abcd, joint] = rep.factor_counts();
    assert_eq!([a, b, c, d], [3, 3, 3, 3], "each marginal is 3 of 4");
    assert_eq!(e, 0, "no whole-TU recognizer fires on any of these");
    assert_eq!(bc, 2, "B and C jointly is measured per TU, not B*C/n");
    assert_eq!(
        abcd, 0,
        "no TU satisfies all four — the joint can be 0 while every marginal \
         is 3/4, which is the whole reason this is measured and not multiplied"
    );
    assert_eq!(joint, 0, "and E adds nothing when no recognizer fires");
    assert!(rep.factor_all_tus().is_empty());
}

/// **PROGRESS MASS reproduces the hand computation and publishes its
/// inputs** (`docs/PROGRESS_METRIC.md`). Four graded TUs with known factor
/// bits and a known emitted census; the value must be the mean of the four
/// fractions, and the `gap-metric` key must carry the same digits.
#[test]
fn progress_mass_matches_the_hand_computation() {
    let mut t1 = mk_factors(TuClass::VocabGap, "t1.cpp", true, true, true, false, false);
    t1.emit.insert("emit-emitted".into(), 4);
    t1.emit.insert("emit-in-class".into(), 2);
    let mut t2 = mk_factors(TuClass::VocabGap, "t2.cpp", false, true, true, false, false);
    t2.emit.insert("emit-emitted".into(), 6);
    t2.emit.insert("emit-in-class".into(), 1);
    let t3 = mk_factors(TuClass::VocabGap, "t3.cpp", false, false, false, false, false);
    let mut cf = mk("x");
    cf.class = TuClass::CaptureFail; // never graded, in no denominator
    let rep = mk_report(vec![t1, t2, t3, cf]);

    let p = rep.progress_mass().expect("three graded TUs with emitted fns");
    assert_eq!((p.graded, p.a, p.b, p.c), (3, 1, 2, 2));
    assert_eq!((p.emitted_in_class, p.emitted_total), (3, 10));
    assert_eq!(p.mismatch_zeroed, 0);
    let expect = (1.0 / 3.0 + 2.0 / 3.0 + 2.0 / 3.0 + 3.0 / 10.0) / 4.0;
    assert!((p.value - expect).abs() < 1e-12, "P is the mean of the four fractions");
    let m: BTreeMap<&str, String> = rep.metrics().into_iter().collect();
    assert_eq!(m["progress-mass"], format!("{expect:.5}"));
    assert_eq!(m["progress-emitted-in-class"], "3");
    assert_eq!(m["progress-emitted-total"], "10");
}

/// **A wrong emit always scores strictly below the refusal it replaced** —
/// the structural guard against the metric paying lanes to emit
/// *something*. The quantity under mutation is ONE TU's class, refusal →
/// `mismatch`, with every count on the TU held fixed; the numerators must
/// drop, both denominators must hold, and P must strictly decrease.
///
/// This is the property board #232 makes non-negotiable: the `26`-separator
/// widening turned a clean refusal into a wrong emit, and any metric on
/// which that transition scores upward would have *rewarded* the defect.
#[test]
fn a_wrong_emit_scores_strictly_below_the_refusal_it_replaced() {
    let build = |cls: TuClass| {
        let mut x = mk_factors(cls, "x.cpp", true, true, true, false, false);
        x.emit.insert("emit-emitted".into(), 4);
        x.emit.insert("emit-in-class".into(), 2);
        let mut y = mk_factors(TuClass::VocabGap, "y.cpp", false, true, false, false, false);
        y.emit.insert("emit-emitted".into(), 6);
        y.emit.insert("emit-in-class".into(), 1);
        mk_report(vec![x, y])
    };
    let refuse = build(TuClass::VocabGap).progress_mass().unwrap();
    let wrong = build(TuClass::Mismatch).progress_mass().unwrap();
    assert_eq!(refuse.graded, wrong.graded, "a mismatch is still graded");
    assert_eq!(
        refuse.emitted_total, wrong.emitted_total,
        "the f denominator never shrinks on a mismatch — zeroing must cost"
    );
    assert_eq!(wrong.mismatch_zeroed, 1, "and the zeroing is printed, not silent");
    assert_eq!(
        (wrong.a, wrong.b, wrong.c, wrong.emitted_in_class),
        (0, 1, 0, 1),
        "every one of the mismatch TU's contributions is gone from the numerators"
    );
    assert!(
        wrong.value < refuse.value,
        "refusal {} must strictly outscore the same TU emitting wrong bytes {}",
        refuse.value,
        wrong.value
    );
}

/// **A progress number over an empty scan is unrepresentable.** objdiff's
/// own `Measures::calc_fuzzy_match_percent` returns 100.0 when
/// `total_code == 0`; fifteen recorded instances on this project say
/// absence read as success is the standing failure mode. `progress_mass`
/// must return `None` — and the `gap-metric` key must be absent, never 0,
/// never 100 — both for a scan of nothing and for a scan where nothing
/// captured.
#[test]
fn progress_mass_is_unrepresentable_over_an_empty_scan() {
    for rep in [mk_report(vec![]), {
        let mut cf = mk("x");
        cf.class = TuClass::CaptureFail;
        mk_report(vec![cf])
    }] {
        assert!(rep.progress_mass().is_none());
        let m: BTreeMap<&str, String> = rep.metrics().into_iter().collect();
        assert!(
            !m.contains_key("progress-mass"),
            "no key at all: a collector must read NO-RESULT, not a number"
        );
    }
    // And a graded scan whose TUs emitted nothing has no f denominator —
    // also None, not a division-by-zero and not a flattering 3-term mean.
    let rep = mk_report(vec![mk_factors(
        TuClass::VocabGap,
        "t.cpp",
        true,
        true,
        true,
        false,
        false,
    )]);
    assert!(rep.progress_mass().is_none());
}

/// **The fifth term is a DISJUNCT on D, and both directions of that are
/// checked here** (board #179).
///
/// `d.cpp` fails D and passes E; `e.cpp` passes D and fails E. Both are in
/// the model's joint `A∧B∧C∧(D∨E)` and only one is in §10.19's original
/// `A∧B∧C∧D`. If E were ever re-implemented as a *widening of D* the two
/// numbers would collapse into one and this test would say so.
#[test]
fn the_fifth_term_is_a_disjunct_on_d_and_the_old_conjunction_is_still_measured() {
    let rep = mk_report(vec![
        mk_factors(TuClass::Match, "whole-tu.cpp", true, true, true, false, true),
        mk_factors(TuClass::Match, "per-fn.cpp", true, true, true, true, false),
        // A/B/C hold, neither acceptance path takes it: in neither joint.
        mk_factors(TuClass::VocabGap, "neither.cpp", true, true, true, false, false),
    ]);
    let [_a, _b, _c, d, e, _a_lo, _bc, abc, abcd, joint] = rep.factor_counts();
    assert_eq!((d, e), (1, 1), "one TU per acceptance path");
    assert_eq!(abc, 3, "A∧B∧C is unaffected by the fifth term");
    assert_eq!(abcd, 1, "§10.19's conjunction still misses the whole-TU TU");
    assert_eq!(joint, 2, "the disjunction picks up both acceptance paths");
    assert_eq!(rep.factor_abcd_tus(), vec!["per-fn.cpp"]);
    assert_eq!(rep.factor_all_tus(), vec!["whole-tu.cpp", "per-fn.cpp"]);
}

/// **The registry is the fifth term's whole population, and it must be
/// well-formed**: non-empty (an empty one makes E identically false and the
/// control green for the wrong reason) and with distinct names (two entries
/// sharing a name would collide on the `emit-whole-tu|<name>` key and report
/// one marginal for two recognizers).
#[test]
fn the_whole_tu_registry_is_non_empty_and_its_names_are_distinct() {
    assert!(
        !WHOLE_TU_RECOGNIZERS.is_empty(),
        "an empty registry makes E identically false, which would make the D-or-E \
         control pass by measuring nothing"
    );
    let names: std::collections::BTreeSet<&str> =
        WHOLE_TU_RECOGNIZERS.iter().map(|(n, _)| *n).collect();
    assert_eq!(
        names.len(),
        WHOLE_TU_RECOGNIZERS.len(),
        "two entries with one name share a key and report one marginal for two \
         recognizers"
    );
    assert!(names.iter().all(|n| !n.is_empty()));
}

/// **The known-answer control**, and it must be able to go red. A/B/C/`D∨E`
/// are *necessary* conditions for a byte-exact obj, so a `match` TU outside
/// one means the term is not necessary and every bound drawn from it is
/// void. For **C** this is also the only executable check on
/// [`PORT_WRITER_SECTIONS`]: a matching obj is the port's own output, so a
/// name missing from that list surfaces here.
///
/// The guard's quantity — one `match` TU — is held fixed across both halves,
/// so the second half cannot pass by the TU ceasing to be a `match`.
#[test]
fn a_matching_tu_outside_any_factor_is_a_red_control() {
    let ok = mk_factors(TuClass::Match, "Spew.cpp", true, true, true, true, false);
    let rep = mk_report(vec![
        ok,
        mk_factors(TuClass::VocabGap, "z.cpp", false, false, false, false, false),
    ]);
    assert_eq!(rep.factor_control_on_match_tus(), ([0, 0, 0, 0, 1, 0], 1));
    assert_eq!(rep.factor_all_tus(), vec!["Spew.cpp"]);

    // The mutation: the same matching TU, now carrying a section the writer
    // cannot emit. That is impossible — the port wrote that obj — so it must
    // be counted, and against factor C specifically.
    let bad = mk_factors(TuClass::Match, "Spew.cpp", true, true, false, true, false);
    let rep = mk_report(vec![bad]);
    assert_eq!(
        rep.count(TuClass::Match),
        1,
        "the mutation must not change the number of byte-exact TUs — otherwise \
         this control tests the class filter, not the factor"
    );
    assert_eq!(
        rep.factor_control_on_match_tus(),
        ([0, 0, 1, 0, 1, 0], 1),
        "a byte-exact obj outside the port writer's section vocabulary is \
         impossible; C must say so, and name itself"
    );
}

/// **The fifth term's degradation guard, executable** (board #179, prereg
/// clause 6: *a green control is not evidence unless the red case is
/// demonstrable*).
///
/// The scenario is exactly the one that happened on 2026-08-04 and the one
/// that will happen again: a **new whole-TU emit path lands in `PortC2`, the
/// differential grades its TU byte-exact, and nobody adds it to
/// [`WHOLE_TU_RECOGNIZERS`]**. Such a TU is a `match` with D false and E
/// false, and the `D∨E` column must go red and stay red until the registry
/// is taught the path.
///
/// This is the *only* guard that E is complete, and it is empirical rather
/// than static — `gap.rs` cannot enumerate `c2-core`'s match arms, and a test
/// asserting `decodes() == functions().is_some() || <registry>` would pass
/// vacuously on every bundle that exercises no new path. Nothing here claims
/// otherwise.
///
/// The three-way contrast is the content: a per-function match (D), a
/// registered whole-TU match (E) and an unregistered one (neither) sit in the
/// same report, and only the third is counted.
#[test]
fn the_control_goes_red_for_an_unregistered_whole_tu_path() {
    // Green: both acceptance paths accounted for.
    let rep = mk_report(vec![
        mk_factors(TuClass::Match, "per-fn.cpp", true, true, true, true, false),
        mk_factors(TuClass::Match, "dyninit.cpp", true, true, true, false, true),
    ]);
    let (bad, n) = rep.factor_control_on_match_tus();
    assert_eq!(n, 2);
    assert_eq!(
        bad[5], 0,
        "with both paths modelled the necessary term holds on every match"
    );
    assert_eq!(
        (bad[3], bad[4]),
        (1, 1),
        "and each disjunct is individually violated — which is why neither is \
         the necessary term"
    );
    assert_eq!(rep.factor_all_tus(), vec!["per-fn.cpp", "dyninit.cpp"]);

    // RED: a third TU the port emitted byte-exact through a path no registry
    // entry models. The population of matches is deliberately grown rather
    // than mutated, so the two green TUs stay green and the red column
    // cannot be an artefact of the class filter.
    let rep = mk_report(vec![
        mk_factors(TuClass::Match, "per-fn.cpp", true, true, true, true, false),
        mk_factors(TuClass::Match, "dyninit.cpp", true, true, true, false, true),
        mk_factors(TuClass::Match, "unregistered.cpp", true, true, true, false, false),
    ]);
    let (bad, n) = rep.factor_control_on_match_tus();
    assert_eq!(n, 3, "the mutation adds a match, it does not reclassify one");
    assert_eq!(
        bad[5], 1,
        "an emit path outside the registry must void the bound — this is the \
         only thing keeping E from being a rubber stamp"
    );
    assert_eq!(
        [bad[0], bad[1], bad[2]],
        [0, 0, 0],
        "A/B/C are unaffected, so the red names the term that is actually wrong"
    );
    assert!(
        !rep.factor_all_tus().contains(&"unregistered.cpp"),
        "and the joint stops being the match set, which is the printed alarm"
    );
}

/// **The frontier is `A∧B∧C ∧ ¬(D∨E) ∧ ¬match`**, and each of those clauses
/// is load-bearing: a byte-exact TU in the list would be work already done, a
/// TU missing A, B or C is not one widening away from anything, and — board
/// #179 — a TU some whole-TU recognizer already accepts is **not** reachable
/// by widening the per-function class, so advertising it as codegen work
/// would point the next lane at the wrong file.
#[test]
fn the_frontier_is_the_tus_whose_only_remaining_factor_is_codegen() {
    let mut near = mk_factors(TuClass::VocabGap, "near.cpp", true, true, true, false, false);
    near.emit.insert("emit-emitted".into(), 5);
    near.emit.insert("emit-in-class".into(), 4);
    let mut far = mk_factors(TuClass::VocabGap, "far.cpp", true, true, true, false, false);
    far.emit.insert("emit-emitted".into(), 11);
    far.emit.insert("emit-in-class".into(), 8);
    // E-true and not a match: the whole-TU recognizer takes it and the
    // emitter's own fence refuses it. Blocked on work that is not codegen
    // breadth, so it must NOT appear — this is the board #179 narrowing.
    let mut fenced = mk_factors(TuClass::CodegenGap, "fenced.cpp", true, true, true, false, true);
    fenced.emit.insert("emit-emitted".into(), 1);
    fenced.emit.insert("emit-in-class".into(), 0);
    let rep = mk_report(vec![
        far,
        near,
        fenced,
        // Already done — must not appear.
        mk_factors(TuClass::Match, "done.cpp", true, true, true, true, false),
        // Blocked on a factor codegen cannot move — must not appear.
        mk_factors(TuClass::VocabGap, "sections.cpp", true, true, false, false, false),
        mk_factors(TuClass::VocabGap, "emitset.cpp", false, true, true, false, false),
    ]);
    let got: Vec<(&str, usize)> = rep
        .factor_frontier()
        .into_iter()
        .map(|(r, n)| (r.src.as_str(), n))
        .collect();
    assert_eq!(
        got,
        vec![("near.cpp", 1), ("far.cpp", 3)],
        "nearest first by blocked EMITTED functions, and only the TUs where \
         per-function codegen breadth is the whole remaining distance — \
         `fenced.cpp` is one blocked function away and still excluded, because \
         widening the function class cannot reach a TU a whole-TU emitter owns"
    );
}

/// **The machine-readable block publishes the four figures that went stale,
/// and derives the projection rather than leaving it to be subtracted.**
///
/// The second assertion is the reason `factor_frontier_if_a` is a function
/// and not a note saying "add the delta to the frontier". Board #213
/// published `A∧B∧C 25 → 107` and `FRONTIER 17 → 99` as one number, `+82`,
/// because on that corpus the two deltas happened to coincide. They are
/// **not** the same quantity: they differ on any TU inside `B∧C` that fails
/// A and that some acceptance path already takes, and this fixture contains
/// one (`noa_accepted.cpp`). Had `+82` been derived rather than observed,
/// its two halves could not have drifted apart unnoticed.
#[test]
fn the_metric_block_derives_the_projection_instead_of_leaving_it_to_a_reader() {
    let rep = mk_report(vec![
        // Already byte-exact: inside A∧B∧C and inside B∧C, on no frontier.
        mk_factors(TuClass::Match, "done.cpp", true, true, true, true, false),
        // The frontier proper: A, B, C, no acceptance path.
        mk_factors(TuClass::VocabGap, "codegen.cpp", true, true, true, false, false),
        // Fails A only — reachable if the emit-set model were perfect.
        mk_factors(TuClass::VocabGap, "noa.cpp", false, true, true, false, false),
        // Fails A *and* is already accepted per-function. Inside B∧C, so it
        // counts toward the projection; outside BOTH frontiers, so the
        // frontier delta misses it. This is the divergence.
        mk_factors(TuClass::CodegenGap, "noa_accepted.cpp", false, true, true, true, false),
    ]);
    let m: BTreeMap<&str, String> = rep.metrics().into_iter().collect();
    let g = |k: &str| m.get(k).expect("stable key must be present").clone();
    assert_eq!(g("graded"), "4");
    assert_eq!(g("b-and-c"), "4", "B∧C is a per-TU joint, not a product");
    assert_eq!(g("a-and-b-and-c"), "2");
    assert_eq!(g("frontier"), "1");
    assert_eq!(g("frontier-if-a"), "2");
    assert_eq!(
        g("emit-predicate-worth"),
        "2",
        "the projection is B∧C − A∧B∧C, derived here so it cannot be \
         reassembled from two independently-stale halves"
    );
    assert_ne!(
        g("emit-predicate-worth"),
        "1",
        "and it is NOT the frontier delta (1 here): the two coincide only \
         while every accepted TU inside B∧C also satisfies A, which is a \
         fact about a corpus and not an identity"
    );
    assert_eq!(
        rep.factor_projection_divergence(),
        vec!["noa_accepted.cpp"],
        "and the divergence is reported BY NAME, so the disagreement points at \
         a file instead of at an unexplained off-by-N"
    );
    assert!(
        !m.contains_key("ladder-head"),
        "a closed/empty ladder omits the head keys rather than publishing a \
         zero — a collector reading a missing key as 0 would announce a \
         ladder that reaches C = 0 (trap 5)"
    );
}

/// **The per-TU factor membership re-derives every joint the metric block
/// publishes, and a `capture-fail` TU is ABSENT from it rather than zeroed.**
///
/// This is the property that makes the `--factors-tsv` file a publication of
/// the same measurement instead of a second one. Lane `w-emitp` had a per-TU
/// emit-set model over its own corpus and could not price it in TU reach,
/// because `|{model exact} ∩ B∧C|` needs a per-TU `B∧C` list and this report
/// only ever published the count. It declined to multiply `151 × 0.555` — the
/// move that left `B∧C` at 107 after `C` grew from 114 to 169 — and filed the
/// listing instead. If the rows and the counts could disagree, the intersection
/// taken against the rows would not be an intersection with `B∧C` at all.
///
/// The second assertion is trap 5 with the mask on: `capture-fail` TUs were
/// never measured, so `-----` for them would be a *measurement* of five false
/// predicates, and a consumer summing the rows would report every factor
/// tighter than it is.
#[test]
fn the_per_tu_factor_membership_re_derives_the_joints_it_is_counted_into() {
    let rep = mk_report(vec![
        mk_factors(TuClass::Match, "done.cpp", true, true, true, true, false),
        mk_factors(TuClass::VocabGap, "codegen.cpp", true, true, true, false, false),
        mk_factors(TuClass::VocabGap, "noa.cpp", false, true, true, false, false),
        mk_factors(TuClass::CodegenGap, "noc.cpp", true, true, false, false, false),
        // Never measured: no obj, no census. Must not appear as a row.
        mk_factors(TuClass::CaptureFail, "gone.cpp", false, false, false, false, false),
    ]);
    let rows = rep.factor_membership();
    let [_a, b, c, _d, _e, _a_lo, bc, abc, _abcd, _joint] = rep.factor_counts();

    assert_eq!(rows.len(), 4, "4 graded TUs, and `gone.cpp` is not one of them");
    assert!(
        !rows.iter().any(|(src, _, _)| *src == "gone.cpp"),
        "a capture-fail TU is ABSENT from the membership, never a `-----` row: \
         it was not measured, which is a different fact from every factor \
         being false (docs/STATUS.md trap 5)"
    );
    assert_eq!(
        rows.iter().map(|(_, _, l)| l.as_str()).collect::<Vec<_>>(),
        vec!["ABCD-", "ABC--", "-BC--", "AB---"],
        "fixed width, fixed order, `-` for a predicate that does not hold"
    );

    // The joints, re-derived from the rows alone.
    let n = |i: usize| rows.iter().filter(|(_, _, l)| l.as_bytes()[i] != b'-').count();
    assert_eq!((n(1), n(2)), (b, c), "the B and C marginals come back");
    assert_eq!(
        rows.iter()
            .filter(|(_, _, l)| l.as_bytes()[1] != b'-' && l.as_bytes()[2] != b'-')
            .count(),
        bc,
        "and B∧C is the same 3 whether counted here or in factor_counts — \
         which is the only reason another lane may intersect against these rows"
    );
    assert_eq!(
        rows.iter()
            .filter(|(_, _, l)| l.as_bytes()[..3].iter().all(|&ch| ch != b'-'))
            .count(),
        abc,
        "same for A∧B∧C"
    );
    assert_eq!((bc, abc), (3, 2), "and the hand count agrees with both");

    // The file body: header, the row count, and the class column.
    let tsv = rep.factor_tsv();
    assert!(tsv.contains("# graded-rows 4\n"));
    assert!(
        tsv.contains("done.cpp\tmatch\t1\t1\t1\t1\t0\tABCD-\n"),
        "each row carries the TuClass label too, so `∩ ¬match` and `∩ B∧C` are \
         both takeable from one file:\n{tsv}"
    );
    assert!(!tsv.contains("gone.cpp"), "and the unmeasured TU is not in the file");
}

/// An obj whose section headers did not decode is **outside** C, never
/// inside it. An empty section list would read as "carries nothing beyond
/// the writer's set", which is the flattering direction and the shape
/// §9.18.8 records twelve times.
#[test]
fn an_unreadable_obj_is_outside_factor_c_rather_than_vacuously_inside_it() {
    let mut r = mk("x");
    r.class = TuClass::VocabGap;
    r.src = "broken.cpp".into();
    r.emit.insert("emit-sec-unreadable".into(), 1);
    let rep = mk_report(vec![r]);
    assert_eq!(rep.factor_counts()[2], 0, "no section list means no C");
    assert!(rep.factor_all_tus().is_empty());
}

/// **The greedy ladder must run through a zero-gain step.** Two names that
/// only ever co-occur each score 0 alone, so a ladder that stopped on
/// no-progress would report the vocabulary as unclosable when it is one step
/// from closed — which is exactly the workload's `.CRT$XCU`/`.text$yc` pair
/// (126 objs each, never apart).
#[test]
fn the_greedy_ladder_runs_through_a_zero_gain_step() {
    let tu = |src: &str, extras: &[&str]| {
        let mut r = mk("x");
        r.class = TuClass::VocabGap;
        r.src = src.into();
        r.emit.insert("emit-sec-readable".into(), 1);
        for e in extras {
            r.emit.insert(format!("emit-sec-extra|{e}"), 1);
        }
        if extras.is_empty() {
            r.emit.insert("emit-sec-reachable".into(), 1);
        }
        r
    };
    let rep = mk_report(vec![
        tu("in.cpp", &[]),
        tu("one.cpp", &[".data"]),
        tu("two.cpp", &[".data"]),
        tu("pair1.cpp", &[".CRT$XCU", ".text$yc"]),
        tu("pair2.cpp", &[".CRT$XCU", ".text$yc"]),
    ]);
    assert_eq!(rep.factor_counts()[2], 1, "one TU is already reachable");
    assert_eq!(
        rep.section_ladder(),
        vec![
            (".data".to_string(), 3),
            (".CRT$XCU".to_string(), 3),
            (".text$yc".to_string(), 5),
        ],
        "greedy takes the +2 first, then must push through the zero-gain half \
         of the co-occurring pair to reach the whole workload"
    );
}

/// The vocabulary census counts **objs carrying a section**, not sections.
/// Under `/Gy` one obj holds one COMDAT `.text` per emitted function, so the
/// second reading would report 158 for `src/App.cpp` alone and no reader of
/// the table could tell which number it was looking at.
#[test]
fn the_section_vocabulary_counts_objs_and_not_sections() {
    let tu = |src: &str, names: &[&str]| {
        let mut r = mk("x");
        r.class = TuClass::VocabGap;
        r.src = src.into();
        r.emit.insert("emit-sec-readable".into(), 1);
        // 158 `.text` sections in this obj — one row, because the key is
        // written once per DISTINCT name per TU.
        r.emit.insert("emit-sec-count".into(), 158);
        for n in names {
            r.emit.insert(format!("emit-sec-name|{n}"), 1);
        }
        r
    };
    let rep = mk_report(vec![tu("a.cpp", &[".text", ".data"]), tu("b.cpp", &[".text"])]);
    assert_eq!(
        rep.section_vocabulary(),
        vec![(".text".to_string(), 2), (".data".to_string(), 1)],
        "two objs carry `.text` and one carries `.data`, ranked most common first"
    );
}

#[test]
fn fn_census_aggregates_across_tus() {
    // Two TUs: 10 functions each, 3 + 4 in class, blockers summed by key.
    // The point of P2b: coverage is measurable (7/20) even though NO whole
    // TU is in class, so both TUs classify as `codegen-gap` above.
    let mut a = mk("x");
    a.fn_total = 10;
    a.fn_in_class = 3;
    a.fn_blockers.insert("expr-cmp-gt".into(), 5);
    a.fn_blockers.insert("expr-shift".into(), 2);
    let mut b = mk("x");
    b.fn_total = 10;
    b.fn_in_class = 4;
    b.fn_blockers.insert("expr-cmp-gt".into(), 6);
    let rep = mk_report(vec![a, b]);
    assert_eq!(rep.fn_coverage(), (7, 20));
    assert_eq!(
        rep.fn_blocker_histogram(),
        vec![("expr-cmp-gt".to_string(), 11), ("expr-shift".to_string(), 2)]
    );
}

/// **The dispatch axes aggregate, and the residue is a NUMBER.**
///
/// The rows below are the ones a ranking reads: two dispatch arms, one of
/// which (`disp-expr`) can never reach a member-call production, and the
/// tag-coverage residue on the production axis. Each is asserted as a positive
/// count with its own message, because the way this report fails is by
/// printing a short table that looks complete.
#[test]
fn dispatch_axes_aggregate_across_tus() {
    let mut a = mk("x");
    a.fn_total = 10;
    a.fn_in_class = 1;
    // Six bodies took the expression arm; four of those are blocked on a
    // member-call construct they can never reach a member-call production
    // with, which is the whole point of the axis.
    a.fn_dispatch.insert("disp-expr".into(), 6);
    a.fn_dispatch.insert("disp-expr|BLOCKED".into(), 6);
    a.fn_dispatch
        .insert("disp-expr|BLOCKED|expr-call-in-expr-recv-field-whole".into(), 4);
    a.fn_dispatch.insert("disp-assign".into(), 4);
    a.fn_dispatch.insert("disp-assign|BLOCKED".into(), 3);
    a.fn_prod.insert("prod-not-entered".into(), 6);
    a.fn_prod.insert("prod-entered-untagged".into(), 3);
    a.fn_prod.insert("prod-accepted".into(), 1);
    let mut b = mk("x");
    b.fn_total = 5;
    b.fn_dispatch.insert("disp-expr".into(), 5);
    b.fn_dispatch.insert("disp-expr|BLOCKED".into(), 5);
    b.fn_prod.insert("prod-not-entered".into(), 3);
    b.fn_prod.insert("prod-entered-untagged".into(), 2);
    let rep = mk_report(vec![a, b]);

    let disp = rep.fn_dispatch_histogram();
    let get = |h: &[(String, usize)], k: &str| -> usize {
        h.iter().find(|(a, _)| a == k).map(|(_, n)| *n).unwrap_or(0)
    };
    assert_eq!(
        get(&disp, "disp-expr"),
        11,
        "the expression arm must sum across TUs — this is the arm that CANNOT \
         reach a member-call production, so its size is the part of a \
         member-call row no widening there can serve"
    );
    assert_eq!(
        get(&disp, "disp-expr|BLOCKED|expr-call-in-expr-recv-field-whole"),
        4,
        "the arm x census-key cross must survive aggregation: it is the only \
         row that says a member-call CONSTRUCT arrived in an arm the member-call \
         productions never see"
    );
    let prod = rep.fn_prod_histogram();
    assert_eq!(
        get(&prod, "prod-not-entered"),
        9,
        "`prod-not-entered` is a measured population and must aggregate like \
         any other row, not be suppressed as a default"
    );
    assert_eq!(
        rep.prod_untagged_residue(),
        5,
        "the tag-coverage residue must be reported as a NUMBER — it is what the \
         tag sites in mcall_*.rs have left to explain, and inferring it from \
         missing rows is the mistake this axis exists to stop"
    );
    // Both axes must sum to the same population the census counted. A short
    // count means bodies went untagged and every row above is a lower bound.
    assert_eq!(
        rep.dispatch_axis_totals(),
        (15, 15),
        "both axes must account for all 15 functions: a body takes exactly one \
         arm and reaches exactly one production state, so a short total is an \
         under-reporting instrument rather than a small population"
    );
}

/// **A scan in which nothing reached a tagged site still reports numbers.**
///
/// This is the state of the board before the 37 tag sites in
/// `body::shapes::mcall_{tail,chain,cmp}` are placed: every body that entered
/// a production lands in `prod-entered-untagged`. The residue must read as
/// that population's exact size — not as 0, and not as an empty histogram,
/// either of which would be indistinguishable from "no bodies enter a
/// production at all".
#[test]
fn an_entirely_untagged_scan_reports_its_residue_rather_than_nothing() {
    let mut a = mk("x");
    a.fn_total = 7;
    a.fn_prod.insert("prod-not-entered".into(), 3);
    a.fn_prod.insert("prod-entered-untagged".into(), 4);
    a.fn_prod.insert("prod-entered-untagged|BLOCKED".into(), 4);
    a.fn_dispatch.insert("disp-assign".into(), 4);
    a.fn_dispatch.insert("disp-expr".into(), 3);
    let rep = mk_report(vec![a]);
    assert_eq!(
        rep.prod_untagged_residue(),
        4,
        "with no tag site placed, the residue IS the whole entered population \
         and must be printed as such"
    );
    assert!(
        rep.fn_prod_histogram()
            .iter()
            .any(|(k, n)| k == "prod-entered-untagged" && *n == 4),
        "the residue must appear as a ranked row too, so a reader of the table \
         sees it beside the named sites rather than having to know it is missing"
    );
    assert_eq!(
        rep.dispatch_axis_totals(),
        (7, 7),
        "and the axes still account for every function"
    );
}

// ---------------------------------------------------------------------------
// FUNCTION BYTE MATCH (lane w-fuzzy, `docs/FUNCTION_BYTE_MATCH.md`)
// ---------------------------------------------------------------------------

use super::fnbytes::{compare_body, compare_relocs, FnByte, RelocKind};

/// A TU carrying a hand-set `fnbyte-` partition.
fn mk_fnbyte(class: TuClass, src: &str, rows: &[(&str, usize)]) -> TuResult {
    let mut r = mk("x");
    r.class = class;
    r.src = src.into();
    let mut d = 0;
    for (k, n) in rows {
        r.emit.insert((*k).to_string(), *n);
        d += *n;
    }
    r.emit.insert("fnbyte-denominator".into(), d);
    r
}

/// **The comparison is the judge's predicate and nothing else.** Equal bytes are
/// `Exact`; one flipped word is `Differs`, with the forensic triple counted but
/// — see the next test — credited nowhere.
#[test]
fn fnbyte_compare_is_byte_equality_and_the_triple_is_forensic() {
    let a = [0x38u8, 0x60, 0x00, 0x01, 0x4e, 0x80, 0x00, 0x20];
    assert_eq!(compare_body(&a, &a), FnByte::Exact);
    let mut b = a;
    b[3] = 0x02; // one immediate field differs — objdiff would score this 99 %
    assert_eq!(
        compare_body(&b, &a),
        FnByte::Differs {
            port_words: 2,
            ref_words: 2,
            equal_words: 1
        },
        "a one-immediate miss is a MISS; the equal-word count is forensic only"
    );
    // Length disagreement is representable and is not a panic.
    assert_eq!(
        compare_body(&a[..4], &a),
        FnByte::Differs {
            port_words: 1,
            ref_words: 2,
            equal_words: 1
        }
    );
    // Empty output against a real body is a MISS, never a vacuous match. This is
    // objdiff's `total_code == 0 -> 100.0` inverted, at the leaf.
    assert_eq!(
        compare_body(&[], &a),
        FnByte::Differs {
            port_words: 0,
            ref_words: 2,
            equal_words: 0
        }
    );
}

// ---------------------------------------------------------------------------
// RELOC-EQ (lane w-relo, board #884) — the compare, unit-tested with no obj and
// no toolchain. `work/w-relo/PREREG.md` §4 C5 registers this list before it
// existed.
// ---------------------------------------------------------------------------

use c2_core::comdat::{PlanTarget, TextReloc};
use c2_obj::{CodeReloc, RelocTarget};

fn plan(v: &[(u32, u16, &'static str)]) -> Vec<TextReloc<'static>> {
    v.iter()
        .map(|&(va, ty, n)| TextReloc {
            va,
            ty,
            target: PlanTarget::Symbol(n),
        })
        .collect()
}

fn refs(v: &[(u32, u16, &str)]) -> Vec<CodeReloc> {
    v.iter()
        .map(|&(va, ty, n)| CodeReloc {
            va,
            ty,
            target: RelocTarget::Symbol(n.to_string()),
        })
        .collect()
}

/// **`s12`'s class, at the leaf.** Two REL24s at the same offset with the same
/// type word, naming two different functions. The instruction bytes are
/// identical by construction — `48000000` either way — so this is the ONLY
/// place the disagreement is representable.
#[test]
fn a_reloc_naming_a_different_symbol_is_a_target_disagreement() {
    let p = plan(&[(0, c2_obj::IMAGE_REL_PPC_REL24, "?g@@YAXXZ")]);
    let r = refs(&[(0, c2_obj::IMAGE_REL_PPC_REL24, "?ext@@YAXXZ")]);
    assert_eq!(compare_relocs(&p, &r), Some((RelocKind::Target, 0)));
    // The inverse control, at the same leaf: the same name on both sides agrees.
    let same = refs(&[(0, c2_obj::IMAGE_REL_PPC_REL24, "?g@@YAXXZ")]);
    assert_eq!(compare_relocs(&p, &same), None);
}

/// Equal sequences agree, including the empty one — a function that relocates
/// nowhere and a reference that relocates nowhere are equal, and that must not
/// be confused with "not measured", which is a different bucket entirely.
#[test]
fn equal_reloc_sequences_agree_and_two_empty_ones_agree() {
    assert_eq!(compare_relocs(&[], &[]), None);
    let p = plan(&[
        (0, c2_obj::IMAGE_REL_PPC_REL24, "?a@@YAXXZ"),
        (8, c2_obj::IMAGE_REL_PPC_REL24, "?b@@YAXXZ"),
    ]);
    let r = refs(&[
        (0, c2_obj::IMAGE_REL_PPC_REL24, "?a@@YAXXZ"),
        (8, c2_obj::IMAGE_REL_PPC_REL24, "?b@@YAXXZ"),
    ]);
    assert_eq!(compare_relocs(&p, &r), None);
}

/// A count disagreement is decided before any record is read, and it is
/// reported in **both** directions — the port relocating more than c2 and less
/// than c2 are the same verdict and neither is a credit.
#[test]
fn a_count_disagreement_is_decided_first_and_in_both_directions() {
    let one = plan(&[(0, c2_obj::IMAGE_REL_PPC_REL24, "?a@@YAXXZ")]);
    assert_eq!(compare_relocs(&one, &[]), Some((RelocKind::Count, 0)));
    assert_eq!(
        compare_relocs(&[], &refs(&[(0, c2_obj::IMAGE_REL_PPC_REL24, "?a@@YAXXZ")])),
        Some((RelocKind::Count, 0))
    );
}

/// **The packed-word rule, inherited from `c2-obj::reloc`.** `REL24|BRTAKEN` is
/// `0x0206` and is a DIFFERENT relocation from `REL24`. A compare that masked
/// the base type would call these equal, which is the exact defect the `Reloc`
/// type is shaped to prevent — so the whole word is compared and this is the
/// test that says so.
#[test]
fn a_flag_bit_makes_two_otherwise_identical_records_differ() {
    let p = plan(&[(0, c2_obj::IMAGE_REL_PPC_REL24, "?a@@YAXXZ")]);
    let r = refs(&[(
        0,
        c2_obj::IMAGE_REL_PPC_REL24 | c2_obj::IMAGE_REL_PPC_BRTAKEN,
        "?a@@YAXXZ",
    )]);
    assert_eq!(compare_relocs(&p, &r), Some((RelocKind::Type, 0)));
}

/// The offset is compared before the type and the type before the target, so a
/// record at the wrong place reports `Offset` and not a target mismatch.
#[test]
fn a_moved_record_reports_its_offset_and_not_its_target() {
    let p = plan(&[(0, c2_obj::IMAGE_REL_PPC_REL24, "?a@@YAXXZ")]);
    let r = refs(&[(4, c2_obj::IMAGE_REL_PPC_REL24, "?b@@YAXXZ")]);
    assert_eq!(compare_relocs(&p, &r), Some((RelocKind::Offset, 0)));
}

/// **A sequence, not a multiset.** Two sets that are equal as multisets and
/// swapped in order produce different obj bytes, so the order disagreement is
/// reported rather than sorted away.
#[test]
fn an_order_swap_at_equal_offsets_is_a_disagreement() {
    let p = vec![
        TextReloc {
            va: 0,
            ty: c2_obj::IMAGE_REL_PPC_REFHI,
            target: PlanTarget::Symbol("?v@@3HA"),
        },
        TextReloc {
            va: 0,
            ty: c2_obj::IMAGE_REL_PPC_PAIR,
            target: PlanTarget::PairDisplacement(0),
        },
    ];
    let swapped = vec![
        CodeReloc {
            va: 0,
            ty: c2_obj::IMAGE_REL_PPC_PAIR,
            target: RelocTarget::PairDisplacement(0),
        },
        CodeReloc {
            va: 0,
            ty: c2_obj::IMAGE_REL_PPC_REFHI,
            target: RelocTarget::Symbol("?v@@3HA".into()),
        },
    ];
    assert_eq!(compare_relocs(&p, &swapped), Some((RelocKind::Type, 0)));
}

/// **A section-definition target is its own kind.** `Section(".rdata")` can
/// never equal `Symbol(".rdata")`, and the finding "c2 relocated against a
/// section" is not the finding "the port named the wrong function".
#[test]
fn a_section_target_never_equals_a_symbol_of_the_same_spelling() {
    let p = plan(&[(0, c2_obj::IMAGE_REL_PPC_REFHI, ".rdata")]);
    let r = vec![CodeReloc {
        va: 0,
        ty: c2_obj::IMAGE_REL_PPC_REFHI,
        target: RelocTarget::Section(".rdata".into()),
    }];
    assert_eq!(compare_relocs(&p, &r), Some((RelocKind::SectionTarget, 0)));
}

/// A `PAIR`'s index field is a DISPLACEMENT (rev 6.0), so it is compared as a
/// number — and a `PairDisplacement` never equals a `Symbol`.
#[test]
fn a_pair_displacement_is_compared_as_a_number() {
    let p = vec![TextReloc {
        va: 4,
        ty: c2_obj::IMAGE_REL_PPC_PAIR,
        target: PlanTarget::PairDisplacement(0),
    }];
    let same = vec![CodeReloc {
        va: 4,
        ty: c2_obj::IMAGE_REL_PPC_PAIR,
        target: RelocTarget::PairDisplacement(0),
    }];
    assert_eq!(compare_relocs(&p, &same), None);
    let other = vec![CodeReloc {
        va: 4,
        ty: c2_obj::IMAGE_REL_PPC_PAIR,
        target: RelocTarget::PairDisplacement(8),
    }];
    assert_eq!(compare_relocs(&p, &other), Some((RelocKind::Target, 0)));
    let asym = vec![CodeReloc {
        va: 4,
        ty: c2_obj::IMAGE_REL_PPC_PAIR,
        target: RelocTarget::Symbol("0".into()),
    }];
    assert_eq!(compare_relocs(&p, &asym), Some((RelocKind::Target, 0)));
}

/// **The bucket a relocation disagreement lands in is NOT `exact` and NOT
/// `differs`.** It is its own key, so the two failure modes never share a work
/// queue and the widening's before/after stays auditable — and `bytes_exact`
/// recovers the count this instrument credited before relocations were graded.
#[test]
fn a_reloc_differ_is_its_own_bucket_and_the_old_count_is_recoverable() {
    let v = FnByte::RelocDiffers(RelocKind::Target);
    assert_eq!(v.bare(), "fnbyte-reloc-differs");
    assert_eq!(v.key(), "fnbyte-reloc-differs|target");
    assert_ne!(v.bare(), FnByte::Exact.bare());
    assert_ne!(
        v.bare(),
        FnByte::Differs {
            port_words: 1,
            ref_words: 1,
            equal_words: 0
        }
        .bare(),
        "a wrong TARGET and a wrong BYTE are different repairs and must not merge"
    );
    // The old `fnbyte-exact` predicate, over the whole variant set.
    for x in [
        FnByte::Exact,
        FnByte::RelocDiffers(RelocKind::Target),
        FnByte::RelocUnknown,
    ] {
        assert!(x.bytes_exact(), "{x:?} is byte-exact");
    }
    for x in [
        FnByte::Differs {
            port_words: 1,
            ref_words: 1,
            equal_words: 0,
        },
        FnByte::Partial("tail-compose"),
        FnByte::Refused,
        FnByte::Unbound,
        FnByte::NoBytes,
    ] {
        assert!(!x.bytes_exact(), "{x:?} is not byte-exact");
    }
}

/// **THE ANTI-GAMING PROPERTY, as an equality.** A function the port lowers
/// WRONG scores exactly what the same function scores when the port refuses it —
/// zero — and the denominator is identical in both worlds, because it is counted
/// off `c2`'s obj.
///
/// This is the property that disqualified an objdiff-style fuzzy match for this
/// project (`docs/PROGRESS_METRIC.md` §2.2): a partial-credit score pays MORE
/// for a nearly-right wrong emit than for the honest refusal it replaced, and
/// board #232's repair was exactly that transition in the good direction. FBM
/// must be indifferent between them and strictly below `exact` for both.
#[test]
fn a_wrong_body_scores_exactly_what_a_refusal_scores() {
    let refused = mk_report(vec![mk_fnbyte(
        TuClass::VocabGap,
        "x.cpp",
        &[("fnbyte-exact", 3), ("fnbyte-refused", 7)],
    )])
    .fn_byte_match()
    .unwrap();
    let wrong = mk_report(vec![mk_fnbyte(
        TuClass::VocabGap,
        "x.cpp",
        &[("fnbyte-exact", 3), ("fnbyte-differs", 7)],
    )])
    .fn_byte_match()
    .unwrap();
    assert_eq!(
        refused.denominator, wrong.denominator,
        "the denominator is c2's output; refusing cannot shrink it"
    );
    assert_eq!(
        refused.value, wrong.value,
        "emitting seven wrong bodies buys exactly what refusing them buys: nothing"
    );
    assert_eq!(refused.value, 0.3);
    // …and both are strictly below the world where those seven are right.
    let right = mk_report(vec![mk_fnbyte(
        TuClass::VocabGap,
        "x.cpp",
        &[("fnbyte-exact", 10)],
    )])
    .fn_byte_match()
    .unwrap();
    assert!(right.value > wrong.value);
}

/// **FBM over nothing is unrepresentable.** objdiff's
/// `calc_fuzzy_match_percent` returns **100.0** when `total_code == 0`
/// (`objdiff-core/src/bindings/report.rs:249-250`); this project has recorded
/// sixteen instances of absence reading as success. `fn_byte_match` returns
/// `None`, the `gap-metric` key is absent, and the printed block says
/// `NO-RESULT`.
#[test]
fn fnbyte_is_unrepresentable_over_zero_emitted_functions() {
    for rep in [
        mk_report(vec![]),
        mk_report(vec![mk("x")]),
        // The shape that matters most: a port that emits NOTHING anywhere. A
        // naive fuzzy port would score this 100 %.
        mk_report(vec![{
            let mut cf = mk("x");
            cf.class = TuClass::CaptureFail;
            cf
        }]),
    ] {
        assert!(
            rep.fn_byte_match().is_none(),
            "no emitted function graded -> NO-RESULT, never 1.0"
        );
        let m: BTreeMap<&str, String> = rep.metrics().into_iter().collect();
        assert!(!m.contains_key("fnbyte-match"), "absence is absence");
    }
}

/// **A port that refuses everything scores 0, not 1.** The direct inversion of
/// the objdiff trap, stated over a whole scan rather than a single body.
#[test]
fn a_port_that_emits_nothing_scores_zero() {
    let rep = mk_report(vec![mk_fnbyte(
        TuClass::VocabGap,
        "x.cpp",
        &[("fnbyte-refused", 900)],
    )]);
    let f = rep.fn_byte_match().unwrap();
    assert_eq!((f.exact, f.denominator, f.value), (0, 900, 0.0));
    let m: BTreeMap<&str, String> = rep.metrics().into_iter().collect();
    assert_eq!(m["fnbyte-match"], "0.00000");
}

/// **The whole-TU override, and the control that found it.**
///
/// `PortC2::build` has one acceptance route that is not per-function — the
/// `??__E` dynamic-initializer recognizer — and on the corpus its two TUs made
/// the per-function route report `refused` for bodies the differential had
/// already graded byte-exact. On a TU the oracle graded `match`, every emitted
/// function IS byte-identical; the judge's verdict supersedes the instrument's
/// route.
///
/// The control that is NOT relaxed: a per-function body that *differs* on a
/// `match` TU means `select_function` and the COFF emitter disagree about a
/// certified body. Known answer 0.
#[test]
fn a_match_tu_credits_every_emitted_function_and_a_differing_body_is_a_control_break() {
    let rep = mk_report(vec![
        // The dyninit shape: the per-function route has nothing, the obj matches.
        mk_fnbyte(TuClass::Match, "dyninit.cpp", &[("fnbyte-refused", 1)]),
        mk_fnbyte(TuClass::VocabGap, "other.cpp", &[("fnbyte-refused", 9)]),
    ]);
    let f = rep.fn_byte_match().unwrap();
    assert_eq!(f.exact, 0, "the per-function route credited nothing");
    assert_eq!(f.whole_tu, 1, "…and the oracle's verdict credited the one");
    assert_eq!(f.value, 0.1);
    assert_eq!(f.match_tu_differs, 0);
    assert_eq!(
        rep.fn_byte_by_tu().first().map(|(s, e, d)| (*s, *e, *d)),
        Some(("dyninit.cpp", 1, 1)),
        "a byte-exact TU reads 100 % per-TU FBM whatever the route could rebuild"
    );

    // The control break: a differing body on a certified TU.
    let bad = mk_report(vec![mk_fnbyte(
        TuClass::Match,
        "dyninit.cpp",
        &[("fnbyte-differs", 1)],
    )]);
    assert_eq!(
        bad.fn_byte_match().unwrap().match_tu_differs,
        1,
        "select_function disagreeing with the emitter about a certified body \
         must be a printed count, not an absorbed one"
    );
}

/// **The partition is checked, not assumed, and the buckets travel with the
/// ratio.** A metric block that published `fnbyte-match` without
/// `fnbyte-partial` would hide the size of the instrument's own under-report.
#[test]
fn fnbyte_metrics_publish_every_bucket_beside_the_ratio() {
    let rep = mk_report(vec![mk_fnbyte(
        TuClass::VocabGap,
        "x.cpp",
        &[
            ("fnbyte-exact", 2),
            ("fnbyte-differs", 1),
            ("fnbyte-partial", 3),
            ("fnbyte-refused", 4),
            ("fnbyte-unbound", 5),
        ],
    )]);
    let f = rep.fn_byte_match().unwrap();
    assert_eq!(
        f.exact + f.differs + f.partial + f.refused + f.unbound + f.nobytes,
        f.denominator,
        "the six buckets partition the denominator"
    );
    assert_eq!(f.partition_broken, 0);
    let m: BTreeMap<&str, String> = rep.metrics().into_iter().collect();
    for k in [
        "fnbyte-match",
        "fnbyte-exact",
        "fnbyte-whole-tu",
        "fnbyte-denominator",
        "fnbyte-differs",
        "fnbyte-partial",
        "fnbyte-refused",
        "fnbyte-unbound",
        "fnbyte-partition-broken",
        "fnbyte-census-disagree",
        "fnbyte-match-tu-differs",
        "fnbyte-tus",
        "fnbyte-tus-full",
    ] {
        assert!(m.contains_key(k), "gap-metric {k} must ride with the ratio");
    }
    assert_eq!(m["fnbyte-match"], "0.13333");
}

/// **Per-TU FBM is the answer to "how close is the other 870".** Sorted
/// nearest-first, and a TU with no emitted function is excluded rather than
/// counted as 0/0 — the same exclusion `near_match_tus` makes, for the same
/// reason: never-measured is not nearly-done.
#[test]
fn per_tu_fnbyte_ranks_nearest_first_and_excludes_the_unmeasured() {
    let rep = mk_report(vec![
        mk_fnbyte(
            TuClass::VocabGap,
            "far.cpp",
            &[("fnbyte-exact", 1), ("fnbyte-refused", 9)],
        ),
        mk_fnbyte(
            TuClass::VocabGap,
            "near.cpp",
            &[("fnbyte-exact", 9), ("fnbyte-refused", 1)],
        ),
        mk_fnbyte(TuClass::VocabGap, "empty.cpp", &[]),
        {
            let mut cf = mk_fnbyte(TuClass::CaptureFail, "cf.cpp", &[("fnbyte-exact", 5)]);
            cf.class = TuClass::CaptureFail;
            cf
        },
    ]);
    let v = rep.fn_byte_by_tu();
    assert_eq!(
        v.iter().map(|(s, _, _)| *s).collect::<Vec<_>>(),
        vec!["near.cpp", "far.cpp"],
        "nearest first; no emitted functions and capture-fail are both excluded"
    );
}

// ---------------------------------------------------------------------------
// THE BYTE-FRACTION RANKER (lane w-tu3, boards #500/#501)
// ---------------------------------------------------------------------------

/// Give a `TuResult` a byte-fraction record directly, bypassing the obj walk —
/// the same trick `mk_factors` uses for the factor keys, and for the same
/// reason: the ranking, its `None` case and its ordering are pure functions of
/// these counters and must be gradeable with no toolchain.
fn with_bytes(mut r: TuResult, accepted: usize, den: usize, exact: usize) -> TuResult {
    if den == 0 {
        r.emit.insert("bytefrac-no-denominator".into(), 1);
    } else {
        r.emit.insert("bytefrac-denominator".into(), den);
        r.emit.insert("bytefrac-accepted".into(), accepted);
        r.emit.insert("bytefrac-exact".into(), exact);
    }
    r
}

/// **A TU with zero emitted bytes must NOT score 100 %.**
///
/// This is objdiff's `calc_fuzzy_match_percent` bug by name
/// (`objdiff-core/src/bindings/report.rs:249-250`: it returns `100.0` when the
/// denominator is zero), and it is this project's most-repeated defect —
/// absence read as success, recorded 16+ times — in its purest form. A ranker
/// that scored an empty TU at 100 % would put it at the head of the frontier
/// and send the next lane at a file with nothing in it.
///
/// **This test is the must-fail mutation**: make `byte_fraction` return
/// `Some((0, 0))` or `Some((1, 1))` for the empty case and it fails here.
#[test]
fn a_tu_with_no_text_bytes_has_no_byte_fraction_and_is_never_100_percent() {
    let empty = with_bytes(
        mk_factors(TuClass::VocabGap, "empty.cpp", true, true, true, false, false),
        0,
        0,
        0,
    );
    assert_eq!(
        super::fnbytes::byte_fraction(&empty),
        None,
        "no `.text` bytes is NOT a perfect score; it is the absence of a score"
    );
    // …and the absence is COUNTED, not silent. A positive check with a printed
    // count is this project's recorded fix for absence-as-success.
    assert_eq!(
        empty.emit.get("bytefrac-no-denominator").copied(),
        Some(1),
        "and the empty case is counted under its own key, so a frontier full \
         of them is visible rather than invisible"
    );
    // The zero that IS a measurement: a positive denominator with nothing
    // accepted. It must be distinguishable from the case above.
    let nothing = with_bytes(
        mk_factors(TuClass::VocabGap, "nothing.cpp", true, true, true, false, false),
        0,
        380,
        0,
    );
    assert_eq!(
        super::fnbytes::byte_fraction(&nothing),
        Some((0, 380)),
        "0 of 380 is a measured 0%, and reads differently from `n/a`"
    );
}

/// **A wrong emit must LOWER the ranker, never raise it.**
///
/// The anti-gaming property, stated as a test rather than as a paragraph. The
/// numerator credits `Exact` and `Partial` — bodies the judge has not called
/// wrong — and credits `Differs` nowhere, so a port that emitted a wrong body
/// for every function in a TU scores 0, exactly what refusing scores. There is
/// no transformation of the port that raises this number without adding bytes
/// real `c2` would have written.
#[test]
fn bytes_the_judge_called_wrong_are_credited_nowhere() {
    let mut res = mk("wrong.cpp");
    // One 100-byte function the port gets right, one 300-byte function it gets
    // WRONG. Written through the same keys `fnbytes::measure` writes.
    res.emit.insert("bytefrac-denominator".into(), 400);
    res.emit.insert("bytefrac-accepted".into(), 100);
    res.emit.insert("bytefrac-exact".into(), 100);
    res.emit.insert("bytefrac-differs".into(), 300);
    assert_eq!(
        super::fnbytes::byte_fraction(&res),
        Some((100, 400)),
        "the 300 wrong bytes are in the DENOMINATOR and not in the numerator: \
         emitting them scores exactly what refusing them scores, which is the \
         same direction board #232's repair went (it REMOVED a wrong emit)"
    );
    assert_eq!(
        super::fnbytes::byte_fraction_exact(&res),
        100,
        "and `exact` is the oracle-graded floor, quoted with the ratio so the \
         ungraded part of the numerator is never a rumour"
    );
}

/// **The ranking is by byte, and the byte order is not the function order.**
///
/// The fixture is board #465's own refutation, re-encoded: `mmio`'s eight
/// already-emitted functions are 8-byte `li r3,0 ; blr` stubs, 64 of 380 bytes,
/// so it ranks FIRST by function and low by byte; `xboxmem` is 72 of 132.
/// Both figures are `rungs/_2026-08-05-w-tu2.md` §3.1's, and the whole point of
/// this instrument is that the two orders disagree.
#[test]
fn the_byte_ranking_inverts_the_function_ranking_that_was_refuted() {
    let f = |src: &str, emitted: usize, in_class: usize, acc: usize, den: usize| {
        let mut r = mk_factors(TuClass::VocabGap, src, true, true, true, false, false);
        r.emit.insert("emit-emitted".into(), emitted);
        r.emit.insert("emit-in-class".into(), in_class);
        with_bytes(r, acc, den, acc)
    };
    let rep = mk_report(vec![
        // mmio: 8 of 11 functions already emitted (#465 ranks it FIRST), 64 of
        // 380 bytes. DECLINED by w-tu2.
        f("mmio.cpp", 11, 8, 64, 380),
        // xboxmem-shaped: 2 of 4 functions, 72 of 132 bytes. CONVERTED by w-tu1.
        f("xboxmem.cpp", 4, 2, 72, 132),
        // A TU with no `.text` at all — must rank LAST and must be printed, not
        // dropped. A TU missing from a ranking is a TU nobody will ever pick.
        f("data_only.cpp", 1, 0, 0, 0),
    ]);
    let order: Vec<&str> = rep
        .frontier_byte_ranking()
        .iter()
        .map(|(r, _)| r.src.as_str())
        .collect();
    assert_eq!(
        order,
        vec!["xboxmem.cpp", "mmio.cpp", "data_only.cpp"],
        "by BYTE, xboxmem (54.5%) outranks mmio (16.8%) — and the outcome \
         agrees. Board #465's function count ranks them the other way round"
    );
    // The function ranking, computed from the same fixture, disagrees. Asserted
    // rather than asserted-about, so the inversion is machine-checked and does
    // not rest on the prose above.
    let by_fn: Vec<&str> = {
        let mut v: Vec<(&str, f64)> = rep
            .factor_frontier()
            .iter()
            .map(|(r, _)| {
                let e = r.emit.get("emit-emitted").copied().unwrap_or(0);
                let i = r.emit.get("emit-in-class").copied().unwrap_or(0);
                (r.src.as_str(), if e == 0 { 0.0 } else { i as f64 / e as f64 })
            })
            .collect();
        v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap().then(a.0.cmp(b.0)));
        v.into_iter().map(|(s, _)| s).collect()
    };
    assert_eq!(
        by_fn[0], "mmio.cpp",
        "#465's unit puts mmio first (8/11 = 72.7% against xboxmem's 2/4 = 50%), \
         and w-tu2 spent a whole lane discovering it does not convert"
    );
    assert_ne!(
        by_fn[0], order[0],
        "THE TWO UNITS DISAGREE AT THE HEAD. That disagreement is the entire \
         content of this instrument; if it ever stops holding on this fixture \
         the fixture has been broken, not the finding"
    );
    let m: BTreeMap<&str, String> = rep.metrics().into_iter().collect();
    assert_eq!(m.get("frontier-bytefrac-top-tu").unwrap(), "xboxmem.cpp");
    assert_eq!(m.get("frontier-bytefrac-top-accepted").unwrap(), "72");
    assert_eq!(
        m.get("frontier-bytefrac-top-denominator").unwrap(),
        "132",
        "the denominator is published beside the numerator, never a bare \
         percentage — a ratio without its denominator is the shape of the bug \
         this instrument refuses"
    );
    assert_eq!(
        m.get("frontier-bytefrac-no-denominator").unwrap(),
        "1",
        "and the un-scoreable TU is COUNTED rather than quietly absent"
    );
}

/// **The known-answer control, and the one shortfall that is legitimate.**
///
/// A `match` TU is byte-identical to c2's obj, so the port produced a body for
/// every `.text` byte in it: 100 % is the known answer. A **factor E** TU is the
/// exception and the reason the control is classified rather than merely
/// counted — E is a *whole-TU* recognizer and the ranker's numerator is the
/// *per-function* path, which structurally cannot answer for one. That is the
/// same shape as the factorization's factor-D control, which `docs/STATUS.md`
/// leaves red on purpose with its explanation beside it.
///
/// **`bytefrac-control-unexplained` is the number that must be 0.**
#[test]
fn the_control_separates_the_legitimate_shortfall_from_a_broken_numerator() {
    let rep = mk_report(vec![
        // A match the per-function path fully covers: the known answer, 100%.
        with_bytes(
            mk_factors(TuClass::Match, "ok.cpp", true, true, true, true, false),
            132,
            132,
            132,
        ),
        // A match a WHOLE-TU recognizer emitted (factor E). The per-function
        // path sees nothing; expected, and it must not read as a defect.
        with_bytes(
            mk_factors(TuClass::Match, "dyninit.cpp", true, true, true, false, true),
            0,
            24,
            0,
        ),
        // A match with no `.text` at all — board #276's data-only shape. Not a
        // shortfall and NOT a 100%: it has no score.
        with_bytes(
            mk_factors(TuClass::Match, "dataonly.cpp", true, true, true, true, false),
            0,
            0,
            0,
        ),
    ]);
    let (full, nodenom, short) = rep.byte_fraction_control();
    assert_eq!((full, nodenom, short.len()), (1, 1, 1));
    assert!(
        short[0].0 && short[0].1.src == "dyninit.cpp",
        "the only shortfall is the factor-E TU, and it is CLASSIFIED as such"
    );
    let m: BTreeMap<&str, String> = rep.metrics().into_iter().collect();
    assert_eq!(m.get("bytefrac-control-unexplained").unwrap(), "0");
    assert_eq!(m.get("bytefrac-control-shortfall-explained").unwrap(), "1");

    // Now break the numerator on a NON-E match — the mutation the control
    // exists to catch — and watch the unexplained count fire.
    let broken = mk_report(vec![with_bytes(
        mk_factors(TuClass::Match, "ok.cpp", true, true, true, true, false),
        100,
        132,
        100,
    )]);
    let m2: BTreeMap<&str, String> = broken.metrics().into_iter().collect();
    assert_eq!(
        m2.get("bytefrac-control-unexplained").unwrap(),
        "1",
        "a byte-exact TU scoring under 100% with no factor-E excuse means the \
         numerator stopped crediting something, and the control says so with a \
         count rather than leaving the ranking to print plausible percentages"
    );
}

// ---------------------------------------------------------------------------
// Board #720/#721 — the CFG-reachability screen (lane w-tu4).
// ---------------------------------------------------------------------------

/// Give a TU a CFG profile: `(class, blocker key, count)` rows, which is exactly
/// the `"<cflow class>|<census key>"` cross the census writes for BLOCKED bodies.
fn with_cflow(mut r: TuResult, rows: &[(&str, &str, usize)]) -> TuResult {
    for (class, key, n) in rows {
        r.fn_cflow
            .insert(format!("{class}|{key}"), *n);
        *r.fn_cflow.entry((*class).into()).or_insert(0) += *n;
        *r.fn_blockers.entry((*key).into()).or_insert(0) += *n;
    }
    r
}

/// **The screen answers a question no byte/function/refusal count can express.**
///
/// The two TUs here have the SAME blocked-function count and the loop TU has a
/// strictly SMALLER remaining byte count — so #269, #465 and #500 all rank it
/// first or tie it — and it is the one that cannot be built, because `Selected`
/// has no variant with a backward branch. This is the live frontier's
/// `Sort.cpp` (64 B remaining after `Primes`, one blocked `cflow-loop`) against
/// a straight-line TU.
#[test]
fn the_cfg_screen_separates_a_loop_from_a_straight_line_body_at_equal_cost() {
    let loopy = with_cflow(
        with_bytes(
            mk_factors(TuClass::VocabGap, "Sort.cpp", true, true, true, false, false),
            0,
            80,
            0,
        ),
        &[("cflow-loop", "assign-store-type-8643", 1)],
    );
    let straight = with_cflow(
        with_bytes(
            mk_factors(TuClass::VocabGap, "flat.cpp", true, true, true, false, false),
            0,
            800,
            0,
        ),
        &[("cflow-straight", "expr-op-0x27", 1)],
    );
    let rep = mk_report(vec![loopy, straight]);
    let rows = rep.frontier_cfg_reachability();
    let get = |s: &str| {
        rows.iter()
            .find(|(r, _)| r.src == s)
            .map(|(_, v)| v.clone())
            .unwrap()
    };
    assert!(
        !get("Sort.cpp").is_reachable(),
        "one blocked cflow-loop function makes a TU unreachable however few \
         bytes remain — no Selected variant encodes a backward branch"
    );
    assert!(
        get("flat.cpp").is_reachable(),
        "a straight-line blocked body is a RUNG (one unmodelled token), which is \
         a different kind of thing from a missing CFG class"
    );
    assert_eq!(
        get("Sort.cpp"),
        CfgReach::NeedsClass(["cflow-loop".to_string()].into_iter().collect()),
        "the verdict NAMES the missing class, so the next lane reads the \
         mechanism off the screen instead of re-deriving it"
    );
    // And the point: the byte ranking puts the UNREACHABLE TU ahead, because
    // both units are quantities of progress and neither can see the CFG class.
    let by_byte: Vec<&str> = rep
        .frontier_byte_ranking()
        .iter()
        .map(|(r, _)| r.src.as_str())
        .collect();
    assert_eq!(
        by_byte[0], "Sort.cpp",
        "both are at 0% accepted, so #500 breaks the tie by source path and \
         lands on the TU that CANNOT BE BUILT — the screen is orthogonal to the \
         ranking, not a refinement of it"
    );
}

/// **A blocked function the census could not classify is NOT reachable.**
///
/// The live instance is `src/system/utl/Pool.cpp`: two `cflow-if-1` functions —
/// both inside the port's classes — and a constructor tagged `cf-expr-0x05`,
/// which contributes to `fn_blockers` but to no `class|key` row. Its obj is an
/// `mtctr`/`bdnz` CTR loop, so reading the TU as reachable off the two functions
/// the census *could* classify would be exactly wrong.
#[test]
fn an_unclassified_blocked_body_is_not_credited_as_reachable() {
    let mut pool = with_cflow(
        mk_factors(TuClass::VocabGap, "Pool.cpp", true, true, true, false, false),
        &[
            ("cflow-if-1", "expr-brtrue", 1),
            ("cflow-if-1", "expr-op-0x27", 1),
        ],
    );
    // The constructor: a blocker with no CFG class at all, as the census writes
    // it when it bails before the control-flow step.
    *pool.fn_blockers.entry("expr-op-0x27".into()).or_insert(0) += 1;
    pool.fn_cflow.insert("cf-expr-0x05".into(), 1);
    let rep = mk_report(vec![pool]);
    let v = rep.frontier_cfg_reachability()[0].1.clone();
    assert_eq!(
        v,
        CfgReach::Unclassified(1),
        "3 blocked functions, 2 classified: the shortfall is reported as its own \
         verdict. Folding it into `Reachable` is absence read as success, and \
         folding it into NeedsClass would invent a class nobody measured"
    );
    assert!(!v.is_reachable(), "Unclassified is never reachable");
}

/// **The known-answer control (#721): the one TU ever converted carries only
/// port CFG classes** — and an ABSENT control is not a passing one.
#[test]
fn the_cfg_control_passes_on_xboxmem_and_is_absent_rather_than_true_when_missing() {
    // Measured on the real tree: cflow-if-1 x3 (+ its cond-tail-pair cross) and
    // cflow-straight x1 (+ its cmp-shift-or cross), fn_blockers EMPTY.
    let mut xm = mk_factors(TuClass::Match, "xboxmem.cpp", true, true, true, true, false);
    for (k, n) in [
        ("cflow-if-1", 3),
        ("cflow-if-1|cond-tail-pair", 3),
        ("cflow-straight", 1),
        ("cflow-straight|cmp-shift-or", 1),
    ] {
        xm.fn_cflow.insert(k.into(), n);
    }
    let rep = mk_report(vec![xm]);
    assert_eq!(
        rep.cfg_reach_control("xboxmem.cpp"),
        Some(true),
        "the screen's class list must admit the one TU that actually converted; \
         if it does not, the list is wrong and every frontier row is suspect"
    );
    assert_eq!(
        rep.cfg_reach_control("not/in/this/scan.cpp"),
        None,
        "a control the scan never evaluated must read ABSENT, never PASS — a \
         scan over a short list would otherwise print a green control it did \
         not take"
    );
    // A matching TU carrying a loop would mean PORT_CFG_CLASSES is wrong.
    let mut bad = mk_factors(TuClass::Match, "future.cpp", true, true, true, true, false);
    bad.fn_cflow.insert("cflow-loop".into(), 1);
    assert_eq!(
        mk_report(vec![bad]).cfg_reach_control("future.cpp"),
        Some(false),
        "the control is the thing that catches PORT_CFG_CLASSES going stale when \
         a Selected variant is added — it is a hand-maintained mirror of a \
         c2-core enum and nothing in the type system ties them together"
    );
}

// ---------------------------------------------------------------------------
// Board #778 — the CFG SUB-CLASS predicate (lane w-subclass).
//
// The screen could hold only a wholesale claim, so two lanes with genuine
// PARTIAL coverage of `cflow-loop` had to over-claim or record nothing, and
// both correctly recorded nothing. These grade the mechanism that lets them
// record it — and, above all, the property that keeps it honest: NARROWER OR
// EQUAL, NEVER WIDER.
// ---------------------------------------------------------------------------

/// **A restriction NARROWS. It cannot widen — checked over every pair of a
/// grid, with a printed count rather than a spot check.**
///
/// This is the property the whole design turns on. `admits` is
/// `class == class && <sub>`, so `Keys` adds a *conjunct* to what the bare
/// string already tested; the wholesale entry must therefore admit everything
/// the restricted one does, for every `(class, key)`. Both the grid size and
/// the number of pairs actually narrowed are asserted, because a test written
/// as "no counterexample found" passes on an empty grid — absence read as
/// success, and the predicate would be inert rather than correct.
#[test]
fn a_restricted_entry_admits_a_subset_of_what_the_whole_class_admits() {
    let classes = ["cflow-loop", "cflow-if-1", "cflow-if-n", "cflow-straight"];
    let keys = [
        "ptr-walk-mod-loop",
        "expr-op-0x27",
        "assign-store-type-8643",
        "cond-tail-pair",
        "expr-cmp-eq",
    ];
    let restricted = CfgClass {
        class: "cflow-loop",
        sub: CfgSub::Keys(&["ptr-walk-mod-loop", "expr-cmp-eq"]),
    };
    let whole = CfgClass { class: "cflow-loop", sub: CfgSub::Whole };
    let (mut pairs, mut narrowed) = (0usize, 0usize);
    for c in classes {
        for k in keys {
            pairs += 1;
            assert!(
                !restricted.admits(c, k) || whole.admits(c, k),
                "{c}|{k}: the restricted entry admitted a pair the WHOLE class does \
                 not — a restriction that widens is the one direction #778 forbids"
            );
            if whole.admits(c, k) && !restricted.admits(c, k) {
                narrowed += 1;
            }
        }
    }
    assert_eq!(pairs, 20, "the grid must be the size the assertion claims");
    assert_eq!(
        narrowed, 3,
        "3 of the 5 keys land in `cflow-loop` and outside the restriction, so it \
         must be STRICTLY narrower — a `narrowed` of 0 would mean the subset \
         check above passed vacuously"
    );
}

/// **MUST-FAIL MUTATION M2 — a key that EXTENDS a listed key is not admitted.**
///
/// The natural mistake in a hand-written allow-list is `starts_with` where `==`
/// was meant, and census keys nest densely enough that it would bite at once:
/// `expr-cmp-eq` is a strict prefix of `expr-cmp-eq-and-branch-more`, and both
/// are live `cflow-loop` keys on the 878-TU workload (measured 2026-08-05: 734
/// and 7 functions). Under exact matching, a lane that measured the short one
/// claims the short one. Change `CfgSub::Keys(ks) => ks.contains(&key)` to a
/// `starts_with` scan and this test fails, naming the key wrongly admitted.
#[test]
fn a_key_extending_a_listed_key_is_not_admitted() {
    let e = CfgClass { class: "cflow-loop", sub: CfgSub::Keys(&["expr-cmp-eq"]) };
    assert!(e.admits("cflow-loop", "expr-cmp-eq"), "the listed key itself is admitted");
    let mut refused = 0;
    for intruder in [
        "expr-cmp-eq-and-branch-more",
        "expr-cmp-eq-and-op-more",
        "expr-cmp-eq-more",
    ] {
        refused += 1;
        assert!(
            !e.admits("cflow-loop", intruder),
            "`{intruder}` EXTENDS the listed key `expr-cmp-eq` and must not be \
             admitted — a sub-class is an ENUMERATION, and a prefix match would \
             grow it silently every time the census minted a neighbouring key, \
             letting a lane report coverage it never measured"
        );
    }
    assert_eq!(refused, 3, "compare a count, never a status");
    assert!(
        !e.admits("cflow-if-n", "expr-cmp-eq"),
        "the class test is a conjunct of BOTH arms; dropping it would let a \
         restriction leak across classes"
    );
}

/// **The empty restriction admits nothing** — the `⊥` bound's foundation and
/// **must-fail mutation M1**'s detector. A matcher that ignored its key
/// argument (`CfgSub::Keys(_) => true`) passes every other test here and fails
/// this one.
#[test]
fn an_empty_restriction_admits_nothing_and_a_whole_entry_admits_everything() {
    let empty = CfgClass { class: "cflow-loop", sub: CfgSub::Keys(&[]) };
    let whole = CfgClass { class: "cflow-loop", sub: CfgSub::Whole };
    let mut checked = 0;
    for k in ["ptr-walk-mod-loop", "expr-op-0x27", "", "anything"] {
        checked += 1;
        assert!(!empty.admits("cflow-loop", k), "Keys(&[]) must admit nothing, not {k}");
        assert!(whole.admits("cflow-loop", k), "Whole must admit every key in its class");
    }
    assert_eq!(checked, 4, "compare a count, never a status");
    assert!(
        empty.covers_class("cflow-loop"),
        "an empty restriction still NAMES the class — that is what makes a miss \
         render `cflow-loop!<key>` (partial) rather than `cflow-loop` (absent)"
    );
}

/// **A partial miss renders `class!key`; a wholly-absent class renders as the
/// bare class** — so the two are never confused, and `needs_class` counts both.
///
/// This is the reporting half of #778. A screen naming a partially-covered
/// class the same way it names an uncovered one over-states the refusal by
/// exactly as much as the wholesale claim over-states the coverage.
#[test]
fn a_partial_miss_names_the_key_and_a_total_miss_names_only_the_class() {
    let tu = with_cflow(
        mk_factors(TuClass::VocabGap, "loops.cpp", true, true, true, false, false),
        &[("cflow-loop", "expr-op-0x27", 1)],
    );
    // 1. Nothing names the class: the bare string, exactly as the flat
    //    `&[&str]` list produced it. This is what makes the identity
    //    measurement on the workload mean something.
    let none: [CfgClass; 0] = [];
    assert_eq!(
        GapReport::cfg_reach_with(&none, &tu),
        CfgReach::NeedsClass(["cflow-loop".to_string()].into_iter().collect()),
        "an uncovered class is named by its bare string — byte-identical to the \
         pre-#778 rendering"
    );
    // 2. An entry names the class but not this key: the partial form.
    let partial = [CfgClass {
        class: "cflow-loop",
        sub: CfgSub::Keys(&["ptr-walk-mod-loop"]),
    }];
    let v = GapReport::cfg_reach_with(&partial, &tu);
    assert_eq!(
        v,
        CfgReach::NeedsClass(["cflow-loop!expr-op-0x27".to_string()].into_iter().collect()),
        "a partially-covered class names the KEY that fell outside, so the next \
         lane reads which part is missing off the screen instead of re-deriving it"
    );
    assert!(
        v.needs_class("cflow-loop"),
        "`needs_class` must count the partial form too — a caller testing bare \
         set membership would silently stop counting this TU the day the class \
         was restricted, and the count would fall with nothing to say why"
    );
    assert!(!v.is_reachable(), "a body outside the restriction is NOT reachable");
    // 3. The key IS listed: reachable. Without this, 1 and 2 would both pass on
    //    a predicate that never admits anything.
    let hit = [CfgClass { class: "cflow-loop", sub: CfgSub::Keys(&["expr-op-0x27"]) }];
    assert!(
        GapReport::cfg_reach_with(&hit, &tu).is_reachable(),
        "a restriction admitting the TU's only blocked body makes it reachable — \
         this is the claim #778 exists to make expressible, and it is the exact \
         shape `cflow-loop restricted to {{ptr-walk-mod-loop}}` would have"
    );
}

/// **The nesting `⊥ ⊆ ENUMERATED == SHIPPED ⊆ ⊤`, on a report** — the same
/// computation the scan prints, graded here with no toolchain.
#[test]
fn the_reach_bounds_nest_and_bottom_is_empty() {
    let ok = with_cflow(
        mk_factors(TuClass::VocabGap, "flat.cpp", true, true, true, false, false),
        &[("cflow-straight", "expr-op-0x27", 1)],
    );
    let loopy = with_cflow(
        mk_factors(TuClass::VocabGap, "loop.cpp", true, true, true, false, false),
        &[("cflow-loop", "expr-op-0x27", 1)],
    );
    let rep = mk_report(vec![ok, loopy]);
    let b = rep.cfg_reach_bounds();
    assert_eq!(b.frontier, 2, "both TUs are on the frontier");
    assert!(
        b.bottom.is_empty(),
        "BOTTOM restricts every entry to NO keys, so nothing can be reachable; a \
         non-empty BOTTOM means the matcher ignores its key argument (mutation M1)"
    );
    assert_eq!(b.shipped, vec!["flat.cpp"], "the straight-line TU, and only it");
    assert_eq!(
        b.enumerated, b.shipped,
        "re-expressing every Whole entry as the enumeration of its own observed \
         keys must reproduce SHIPPED exactly — this is the live exercise of the \
         `Keys` path, without which it would be a code path no run reaches"
    );
    assert_eq!(
        b.top,
        vec!["flat.cpp", "loop.cpp"],
        "TOP admits every class the frontier mentions, so both are reachable — \
         and TOP−SHIPPED is the honest size of the screen's refusal"
    );
    assert!(
        b.enumerated_keys > 0,
        "the enumeration must have listed something; 0 keys would make the \
         ENUMERATED==SHIPPED check pass vacuously"
    );
    assert!(b.violations().is_empty(), "0 violations: {:?}", b.violations());
}

/// **The ledger reports a whole class as `n/a`, never as PASS**, and counts
/// observed against admitted keys with a denominator.
#[test]
fn the_subclass_ledger_declines_to_pass_a_claim_it_cannot_check() {
    let tu = with_cflow(
        mk_factors(TuClass::VocabGap, "flat.cpp", true, true, true, false, false),
        &[
            ("cflow-straight", "expr-op-0x27", 1),
            ("cflow-straight", "expr-cmp-eq", 1),
        ],
    );
    let led = mk_report(vec![tu]).cfg_subclass_ledger();
    assert_eq!(led.len(), 4, "one row per shipped entry, and there are four");
    let row = led.iter().find(|r| r.class == "cflow-straight").unwrap();
    assert_eq!(row.listed, None, "every shipped entry is Whole today");
    assert!(
        row.intruders.is_none(),
        "a WHOLE entry has no declaration to cross-check against, so the ledger \
         must say `n/a` — printing PASS for a check nobody took is the exact \
         absence-read-as-success this row exists to forbid"
    );
    assert_eq!(
        (row.observed_keys, row.admitted_keys),
        (2, 2),
        "a whole class admits every key observed for it, reported WITH its \
         denominator"
    );
    assert!(row.unwitnessed.is_empty(), "nothing is listed, so nothing is unwitnessed");
}

/// **The ledger's INTRUDER cross-check, graded on a restricted entry** — and
/// this is the second detector for must-fail mutation **M2**.
///
/// The cross-check recomputes the admitted set two ways: by asking
/// [`CfgClass::admits`] about every observed key, and by literal membership in
/// the declared slice. They must agree. No *shipped* entry is restricted, so on
/// the live workload the check reports `n/a` on all four rows and grades
/// nothing — an ungraded path by construction. This test is what grades it.
///
/// The three census keys here are live `cflow-loop` keys on the 878-TU
/// workload and two of them EXTEND the third. Under exact matching one key is
/// admitted and `intruders` is empty; under `starts_with` all three are
/// admitted and `intruders` names the two the entry never declared.
#[test]
fn the_ledger_cross_check_catches_a_matcher_that_admits_beyond_its_declaration() {
    let tu = with_cflow(
        mk_factors(TuClass::VocabGap, "loop.cpp", true, true, true, false, false),
        &[
            ("cflow-loop", "expr-cmp-eq", 1),
            ("cflow-loop", "expr-cmp-eq-and-branch-more", 1),
            ("cflow-loop", "expr-cmp-eq-and-op-more", 1),
        ],
    );
    let list = [CfgClass { class: "cflow-loop", sub: CfgSub::Keys(&["expr-cmp-eq"]) }];
    let led = mk_report(vec![tu]).cfg_subclass_ledger_with(&list);
    assert_eq!(led.len(), 1, "one row per entry");
    let row = &led[0];
    assert_eq!(row.listed, Some(1), "the entry declares exactly one key");
    assert_eq!(
        row.observed_keys, 3,
        "three keys are live for the class on this report — the denominator the \
         admitted count is only readable against"
    );
    assert_eq!(
        row.admitted_keys, 1,
        "exact matching admits the declared key and nothing else; a 3 here is a \
         matcher admitting beyond its declaration"
    );
    assert_eq!(
        row.intruders.as_deref(),
        Some(&[][..]),
        "the cross-check must be TAKEN (Some) and EMPTY for a restricted entry — \
         `None` would mean the ledger declined to check a claim it can check, and \
         a non-empty vector names the keys `admits` accepted that the entry never \
         declared, which is exactly what an exact→prefix slip produces"
    );
}

/// **A restricted entry's unwitnessed keys are COUNTED, not passed over.**
///
/// A listed key no scan ever sees is a claim doing nothing while still standing
/// on the page — trap 5 with the claim attached. The ledger must name it.
#[test]
fn the_ledger_counts_a_listed_key_no_scan_witnessed() {
    let tu = with_cflow(
        mk_factors(TuClass::VocabGap, "flat.cpp", true, true, true, false, false),
        &[("cflow-straight", "expr-op-0x27", 1)],
    );
    let rep = mk_report(vec![tu]);
    // The shipped list is all-Whole, so the ledger cannot exercise this on its
    // own; the property is asserted directly on the entry the ledger consults.
    let e = CfgClass {
        class: "cflow-straight",
        sub: CfgSub::Keys(&["expr-op-0x27", "a-key-no-scan-has-ever-produced"]),
    };
    let observed: Vec<&str> = rep.results[0]
        .fn_cflow
        .keys()
        .filter_map(|k| k.split_once('|'))
        .filter(|(c, _)| *c == e.class)
        .map(|(_, k)| k)
        .collect();
    let admitted = observed.iter().filter(|k| e.admits(e.class, k)).count();
    let unwitnessed = e
        .keys()
        .unwrap()
        .iter()
        .filter(|k| !observed.contains(k))
        .count();
    assert_eq!(admitted, 1, "one of the two listed keys is live on this scan");
    assert_eq!(
        unwitnessed, 1,
        "the other is UNWITNESSED and must be counted — a claim about a key no \
         corpus contains cannot be graded by any run, and silence would read as \
         success"
    );
}

/// **The metric block publishes the bracket, not just the figure.**
///
/// A reachability number quoted without the bound it sits inside is the shape
/// board #213's `+82` had: true when published, silently invalidated when a
/// dependency moved, and unrecomputable by any script.
#[test]
fn the_metrics_publish_the_cfg_bracket_and_a_zero_violation_count() {
    let loopy = with_cflow(
        mk_factors(TuClass::VocabGap, "loop.cpp", true, true, true, false, false),
        &[("cflow-loop", "expr-op-0x27", 1)],
    );
    let flat = with_cflow(
        mk_factors(TuClass::VocabGap, "flat.cpp", true, true, true, false, false),
        &[("cflow-straight", "expr-op-0x27", 1)],
    );
    let m: BTreeMap<&str, String> = mk_report(vec![loopy, flat]).metrics().into_iter().collect();
    for (k, want) in [
        ("cfg-reach-bottom", "0"),
        ("cfg-reach-enumerated", "1"),
        ("cfg-reach-shipped", "1"),
        ("cfg-reach-top", "2"),
        ("cfg-bounds-violations", "0"),
        ("cfg-subclass-entries", "4"),
        ("cfg-subclass-restricted", "0"),
        ("cfg-subclass-unwitnessed", "0"),
        ("cfg-subclass-intruders", "0"),
    ] {
        assert_eq!(
            m.get(k).map(String::as_str),
            Some(want),
            "gap-metric {k} must read {want}; a MISSING key reads NO-RESULT to the \
             collector, which is trap 5 with the mask on"
        );
    }
}

// ---------------------------------------------------------------------------
// Board #322 — the per-shape census, the witnesses, and the partition after the
// four `/Gy` shapes became gradeable (lane `w-fnbyte`)
// ---------------------------------------------------------------------------

/// **The per-shape census keeps a `differs` VISIBLE PER SHAPE.**
///
/// The blind spot board #322 closed was invisible for one reason: the alarm was
/// a corpus total and the shapes behind it were printed only for the bucket
/// that was *not* graded. A shape that goes wrong now has to show up as its own
/// row, and a shape that stops being graded loses its row rather than quietly
/// contributing zeros to a total that still reads 0.
#[test]
fn the_shape_census_reports_each_shape_and_verdict_separately() {
    let rep = mk_report(vec![mk_fnbyte(
        TuClass::VocabGap,
        "x.cpp",
        &[
            ("fnbyte-exact", 3),
            ("fnbyte-differs", 2),
            ("fnbyte-shape|tail|fnbyte-exact", 3),
            ("fnbyte-shape|framed|fnbyte-differs", 2),
        ],
    )]);
    let census = rep.fn_byte_shape_census();
    assert_eq!(
        census,
        vec![
            ("tail".to_string(), "exact".to_string(), 3),
            ("framed".to_string(), "differs".to_string(), 2),
        ],
        "the census is (shape, verdict, count), most frequent first — a corpus \
         total cannot say WHICH shape went wrong"
    );
    // …and the row survives being the only one of its shape. A `differs` of 2 in
    // a denominator of 5 is what a per-shape regression looks like before it is
    // large enough to notice in the ratio.
    assert!(census.iter().any(|(s, v, _)| s == "framed" && v == "differs"));
}

/// **A witness is a reproducer, not a count.** Each differing function is named
/// with its shape, its word counts and the first disagreeing word — and the
/// signature collapse groups identical failures so 1,950 mangled STL names
/// reporting one defect read as one row and not as 1,950.
#[test]
fn the_differ_witnesses_collapse_to_signatures_with_an_example() {
    let rep = mk_report(vec![mk_fnbyte(
        TuClass::VocabGap,
        "x.cpp",
        &[
            ("fnbyte-differs", 3),
            (
                "fnbyte-differs-fn|tail|w1/1/eq0|first@0:port=48000000,ref=4e800020|?a@@YAXXZ",
                1,
            ),
            (
                "fnbyte-differs-fn|tail|w1/1/eq0|first@0:port=48000000,ref=4e800020|?b@@YAXXZ",
                1,
            ),
            (
                "fnbyte-differs-fn|framed|w9/3/eq0|first@0:port=7d8802a6,ref=81630004|?c@@YAXXZ",
                1,
            ),
        ],
    )]);
    assert_eq!(rep.fn_byte_differ_witnesses().len(), 3, "one row per function");
    let sigs = rep.fn_byte_differ_signatures();
    assert_eq!(sigs.len(), 2, "two distinct failures, not three");
    assert_eq!(sigs[0].0, "tail|w1/1/eq0|first@0:port=48000000,ref=4e800020");
    assert_eq!(sigs[0].1, 2);
    assert!(
        sigs[0].2.starts_with("?a@@") || sigs[0].2.starts_with("?b@@"),
        "a signature carries an EXAMPLE symbol; a signature with no name is not \
         reproducible"
    );
    assert_eq!(sigs[1].1, 1);
}

/// **`partial 0` must be printed as a statement, not as an absent line.**
///
/// Before board #322 `partial by shape` was the size of the under-report and it
/// was never empty. Now it can be, and an omitted line is exactly the shape this
/// project has recorded sixteen times: absence read as success. The histogram
/// returning an empty vector is the *input* to that; the render prints the
/// positive sentence. This pins the input so the render cannot start guessing.
#[test]
fn an_empty_partial_histogram_is_empty_and_not_absent() {
    let rep = mk_report(vec![mk_fnbyte(
        TuClass::VocabGap,
        "x.cpp",
        &[("fnbyte-exact", 10)],
    )]);
    assert!(rep.fn_byte_partial_histogram().is_empty());
    let f = rep.fn_byte_match().expect("10 graded functions");
    assert_eq!(f.partial, 0);
    assert_eq!(f.denominator, 10, "the denominator is still c2's, and printed");
}

/// **The partition identity holds with the four shapes graded**, and it is the
/// same identity as before: the buckets sum to the denominator. A shape moving
/// from `partial` into `exact` or `differs` must move the *total* by zero.
#[test]
fn the_partition_identity_survives_the_shapes_becoming_gradeable() {
    // Before: 4 partial. After: 3 exact + 1 differs. Same denominator, same sum.
    let before = mk_report(vec![mk_fnbyte(
        TuClass::VocabGap,
        "x.cpp",
        &[("fnbyte-exact", 6), ("fnbyte-partial", 4)],
    )])
    .fn_byte_match()
    .unwrap();
    let after = mk_report(vec![mk_fnbyte(
        TuClass::VocabGap,
        "x.cpp",
        &[("fnbyte-exact", 9), ("fnbyte-differs", 1)],
    )])
    .fn_byte_match()
    .unwrap();
    assert_eq!(before.denominator, after.denominator, "10 either way");
    assert_eq!(
        before.exact + before.partial,
        after.exact + after.differs,
        "the population is conserved; only its grading changed"
    );
    assert_eq!(before.partition_broken, 0);
    assert_eq!(after.partition_broken, 0);
    // …and the widened instrument scores the wrong body at zero, exactly as the
    // blind one scored the ungraded one. Grading more can only ever RAISE the
    // count of known-wrong bodies, never the credit.
    assert!(after.value > before.value, "3 of the 4 were right");
    assert_eq!(after.differs, 1, "and the fourth is now an ALARM, not a blank");
}

// ---------------------------------------------------------------------------
// The control-flow counterfactual and the residue predicate's denominator.
// Boards #1343 / #1344, lane `w-cflowlabel`.
//
// Portable — no toolchain, no workload. Each test pins one claim the rung's
// numbers rest on, because the rung's whole output is a re-ranking and a
// re-ranking that no test holds is a paragraph.
// ---------------------------------------------------------------------------

/// One TU carrying a hand-built control-flow axis and emitted-census counters.
fn mk_cflow(cflow: &[(&str, usize)], emit: &[(&str, usize)]) -> TuResult {
    let mut t = mk("cflow");
    for (k, n) in cflow {
        t.fn_cflow.insert((*k).into(), *n);
    }
    for (k, n) in emit {
        t.emit.insert((*k).into(), *n);
    }
    t
}

fn mk_cfoff(cflow: &[(&str, usize)], off: &[(&str, usize)]) -> TuResult {
    let mut t = mk_cflow(cflow, &[]);
    for (k, n) in off {
        t.fn_cflow_off.insert((*k).into(), *n);
    }
    t
}

/// **The off-class DECOMPOSITION sums to the off-class TOTAL, and the two are
/// counted in the same unit** — board #1345, and trap 0 written as a test.
///
/// `w-tag02`'s `.in` identity read `1 == 2` for the life of the file because one
/// side counted TOKENS and the other counted RECORDS, and it was green the whole
/// time because the population it ran over was too small to contain the shape.
/// This identity has exactly that hazard: `cflow-residue-inclass-offclass` is a
/// sum over the `fn_cflow` cross and `cflow-offclass-accounted` is a sum over the
/// `fn_cflow_off` cross — two maps, two crosses, one population, and nothing but
/// `Scan::off_class`'s `first reason wins` rule makes them equal.
///
/// It is asserted here **and published on every scan** rather than only
/// asserted, because the tree cannot construct the workload's shapes and a green
/// unit test over four rows is a statement about four rows.
#[test]
fn the_offclass_decomposition_accounts_for_every_off_class_in_class_body() {
    let rep = mk_report(vec![
        mk_cfoff(
            &[
                ("cflow-straight|IN-CLASS", 500),
                ("cflow-straight+expr-modeled|IN-CLASS", 190),
            ],
            &[
                ("off-add|IN-CLASS", 300),
                ("intrinsic|IN-CLASS", 200),
                ("off-add|BLOCKED", 4000),
            ],
        ),
        mk_cfoff(&[], &[("bind|IN-CLASS", 0)]),
    ]);
    let (rows, accounted) = rep.cflow_offclass_reasons();
    let (_modeled, off) = rep.cflow_residue_control();
    assert_eq!(off, 500, "the total the decomposition must account for");
    assert_eq!(accounted, 500, "TOTALITY: the per-reason IN-CLASS column sums to it");
    // Sorted by the in-class column descending — the ranking a repair set would
    // be chosen from, so the order is part of the interface.
    assert_eq!(
        rows,
        vec![
            ("off-add".to_string(), 300, 4000),
            ("intrinsic".to_string(), 200, 0),
            ("bind".to_string(), 0, 0),
        ]
    );
    // …and BOTH columns are published, per reason. The BLOCKED column is what a
    // widening would ADD to the over-claim on the other side of the two-sided
    // error, so a decomposition that printed only the in-class column would rank
    // repairs by their benefit with no cost beside it — which is #1345's
    // `a bare widening publishes a second single number` in table form.
    let m: std::collections::BTreeMap<&str, String> = rep.metrics().into_iter().collect();
    for (k, want) in [
        ("cflow-offclass-accounted", "500"),
        ("cflow-offclass-off-add-inclass", "300"),
        ("cflow-offclass-off-add-blocked", "4000"),
        ("cflow-offclass-intrinsic-inclass", "200"),
        ("cflow-offclass-intrinsic-blocked", "0"),
    ] {
        assert_eq!(m.get(k).map(String::as_str), Some(want), "gap-metric {k}");
    }
    // A reason with no bodies emits no key at all rather than a 0 — absence must
    // be absence, so a collector cannot read a vanished arm as an empty one
    // (`ladder-head`'s rule, `factors.rs`).
    assert_eq!(m.get("cflow-offclass-deref-inclass"), None);
    // And the counterfactual variable is NOT set in a test process, so the run
    // is the shipped predicate and says nothing — the key is absent, not empty.
    assert_eq!(m.get("cflow-residue-admit"), None);
}

/// **The decomposition rides in its OWN map, and that is load-bearing.**
///
/// `cflow_residue_control` counts every `fn_cflow` row ending `|IN-CLASS` that
/// does not end `+expr-modeled|IN-CLASS` as off-class. Had the per-reason rows
/// gone into `fn_cflow` — the obvious place, beside the cross they decompose —
/// `off-add|IN-CLASS` would have been folded straight into the 518,991 and the
/// published number would have doubled with no merge conflict and nothing in the
/// diff to point at. This asserts the separation rather than trusting it.
#[test]
fn the_offclass_decomposition_cannot_leak_into_the_residue_total() {
    let rep = mk_cfoff(
        &[("cflow-straight|IN-CLASS", 500)],
        &[("off-add|IN-CLASS", 300), ("intrinsic|IN-CLASS", 200)],
    );
    let rep = mk_report(vec![rep]);
    assert_eq!(
        rep.cflow_residue_control(),
        (0, 500),
        "the off-class total is the `fn_cflow` cross ALONE — 500, never 1000"
    );
    assert!(
        rep.fn_cflow_histogram().iter().all(|(k, _)| k.starts_with("cflow-")),
        "no decomposition row may appear in the cflow histogram at all"
    );
}

/// **The staleness measure counts the population the port ACCEPTS, split by what
/// the residue predicate says about it.**
///
/// The whole rung turns on this number being readable at all: `CfResidue::Modeled`
/// is a hand-written mirror of the port's class, and an in-class body it calls
/// off-class is one measured unit of the mirror falling behind. The `|BLOCKED`
/// rows must not contribute — they are the population whose answer is unknown,
/// which is the reason the control is taken over the in-class one instead.
#[test]
fn residue_control_counts_only_the_in_class_population() {
    let rep = mk_report(vec![mk_cflow(
        &[
            ("cflow-straight|IN-CLASS", 500),
            ("cflow-straight+expr-modeled|IN-CLASS", 190),
            ("cflow-if-1|IN-CLASS", 4),
            ("cflow-loop|IN-CLASS", 1),
            // the blocked side of the same classes, which must be invisible here
            ("cflow-straight|BLOCKED", 1200),
            ("cflow-straight+expr-modeled|BLOCKED", 83),
            ("cflow-if-1+expr-modeled|BLOCKED", 713),
        ],
        &[],
    )]);
    let (modeled, off) = rep.cflow_residue_control();
    assert_eq!(modeled, 190, "only the in-class `+expr-modeled` rows");
    assert_eq!(off, 505, "500 + 4 + 1 in-class bodies the residue calls off-class");
    assert_eq!(
        modeled + off,
        695,
        "the two partition the in-class population and nothing else joins it"
    );
}

/// **The control that must stay green under the mutation the rung uses as
/// evidence.** `cflow-straight+expr-modeled` is by far the largest
/// `+expr-modeled` row (276,271 on the workload) and it is NOT part of the
/// counterfactual: a straight-line body has no control flow to lower, so a block
/// IR converts none of them. An implementation that tested `ends_with(
/// "+expr-modeled")` without first excluding `cflow-straight` would report the
/// rung as worth ~385× what it is, which is the exact shape of the mis-sizings
/// this axis exists to prevent.
#[test]
fn emitted_counterfactual_excludes_straight_line_bodies() {
    let rep = mk_report(vec![mk_cflow(
        &[],
        &[
            ("emit-cflow-branchy", 40),
            ("emit-cflow-branchy-modeled", 3),
        ],
    )]);
    assert_eq!(rep.cflow_emitted_counterfactual(), (40, 3));
    // …and the SCAN'S OWN predicate, called here rather than restated, so this
    // test cannot pass while `scan.rs` counts a different set. A restated copy
    // is the shape that let three lanes collide through shared semantics with
    // no git conflict.
    let branchy = super::cflow_needs_block_ir;
    assert!(!branchy("cflow-straight"));
    assert!(!branchy("cflow-straight+expr-modeled"), "THE control");
    assert!(!branchy("cf-expr-0x59"), "an UNDECODED body has no known CFG");
    for c in ["cflow-if-1", "cflow-if-2", "cflow-if-n", "cflow-loop", "cflow-switch"] {
        assert!(branchy(c), "{c} needs a block IR");
        assert!(branchy(&format!("{c}+expr-modeled")));
    }
}

/// **`branchy-modeled` is nested inside `branchy`, and reading either alone is
/// the misreading the pair exists to prevent.** A block IR must SERVE every
/// `branchy` function and CONVERTS only the `modeled` ones; quoting the first as
/// the rung's worth over-claims and quoting the second as the population
/// under-claims. The nesting is the invariant that says they are two views of
/// one set.
#[test]
fn emitted_counterfactual_is_nested() {
    let rep = mk_report(vec![
        mk_cflow(&[], &[("emit-cflow-branchy", 12), ("emit-cflow-branchy-modeled", 2)]),
        mk_cflow(&[], &[("emit-cflow-branchy", 30), ("emit-cflow-branchy-modeled", 1)]),
    ]);
    let (branchy, modeled) = rep.cflow_emitted_counterfactual();
    assert_eq!((branchy, modeled), (42, 3), "both sum across TUs");
    assert!(modeled <= branchy, "NESTING: a converted function is a served one");
}

/// **The four keys ride together or the pairing they exist to enforce is gone.**
/// `cflow-emitted-modeled` published without `cflow-residue-inclass-offclass` is
/// the "718" failure exactly: a lower bound of unknown tightness, quotable as a
/// price. `status.sh` reads a missing key as `NO-RESULT`, so absence is the one
/// outcome that must not be silent.
#[test]
fn the_counterfactual_and_its_denominator_are_published_together() {
    let rep = mk_report(vec![mk_cflow(
        &[("cflow-straight|IN-CLASS", 7), ("cflow-straight+expr-modeled|IN-CLASS", 3)],
        &[("emit-cflow-branchy", 5), ("emit-cflow-branchy-modeled", 1)],
    )]);
    let m: std::collections::BTreeMap<&str, String> = rep.metrics().into_iter().collect();
    for (k, want) in [
        ("cflow-emitted-branchy", "5"),
        ("cflow-emitted-modeled", "1"),
        ("cflow-residue-inclass-modeled", "3"),
        ("cflow-residue-inclass-offclass", "7"),
    ] {
        assert_eq!(
            m.get(k).map(String::as_str),
            Some(want),
            "gap-metric {k} must read {want}; a MISSING key reads NO-RESULT to the collector \
             and an unpaired counterfactual is what board #1344 exists to stop"
        );
    }
}

/// **The new population cross does not disturb the axis it was added beside.**
/// The class histogram is built by `!k.contains('|')` and four sessions of
/// recorded tables name its rows, so a key that leaked into it would invalidate
/// every one of them. This is the widen-never-narrow promise, asserted.
#[test]
fn population_cross_does_not_enter_the_class_histogram() {
    let rep = mk_report(vec![mk_cflow(
        &[
            ("cflow-loop", 98387),
            ("cflow-loop|IN-CLASS", 1),
            ("cflow-loop|BLOCKED", 98386),
            ("cflow-loop|body-cflow-label", 36871),
        ],
        &[],
    )]);
    let classes: Vec<(String, usize)> = rep
        .fn_cflow_histogram()
        .into_iter()
        .filter(|(k, _)| !k.contains('|'))
        .collect();
    assert_eq!(
        classes,
        vec![("cflow-loop".to_string(), 98387)],
        "exactly one class row, at the value it had before the cross was added"
    );
    // …and the population rows partition that class, which is the second thing
    // the pair is for: it makes `cflow-loop`'s 98,387 readable as "1 the port
    // accepts, 98,386 it does not" without a second scan.
    let h: std::collections::BTreeMap<String, usize> =
        rep.fn_cflow_histogram().into_iter().collect();
    assert_eq!(
        h["cflow-loop|IN-CLASS"] + h["cflow-loop|BLOCKED"],
        h["cflow-loop"],
        "ACCOUNTING: the population cross must be total over its class"
    );
}

/// **The residue predicate is not conservative, and this is the test that says
/// so.** `Modeled` neither contains nor is contained in the port's class, so the
/// counterfactual is a proxy with a TWO-SIDED error and not a lower bound. A
/// `cflow-straight` body that is `Modeled` and still blocked is refused for a
/// reason `Modeled` said was absent — the one direction checkable without
/// lowering anything. Board **#1344**.
#[test]
fn the_residue_errs_in_both_directions() {
    let rep = mk_report(vec![mk_cflow(
        &[
            ("cflow-straight|IN-CLASS", 500),
            ("cflow-straight+expr-modeled|IN-CLASS", 190),
            // …and the counterexample: `Modeled`, straight-line, still refused.
            ("cflow-straight+expr-modeled|BLOCKED", 83),
            // a BRANCHING blocked+modeled row must NOT count here — for those
            // "blocked" is exactly what the counterfactual is about, so they say
            // nothing about the predicate's accuracy.
            ("cflow-if-1+expr-modeled|BLOCKED", 713),
        ],
        &[],
    )]);
    let (modeled, off) = rep.cflow_residue_control();
    assert_eq!((modeled, off), (190, 500), "direction 1: in-class, called off-class");
    assert_eq!(
        rep.cflow_residue_overclaim(),
        83,
        "direction 2: called Modeled, refused anyway — and the 713 branching \
         bodies are NOT this, they are the counterfactual itself"
    );
    let m: std::collections::BTreeMap<&str, String> = rep.metrics().into_iter().collect();
    assert_eq!(
        m.get("cflow-residue-straight-modeled-blocked").map(String::as_str),
        Some("83"),
        "published, or the pair reads as `the residue is conservative` — which it is not"
    );
}

// ---------------------------------------------------------------------------
// THE CODEGEN COLUMN (lane `w-column`, boards #1473/#1474)
// ---------------------------------------------------------------------------

/// **A parse refusal is not a codegen refusal, and the enum is what says so.**
///
/// Board #1464 read `fnbyte-decline|selector` as a codegen column. It was not:
/// [`super::fnbytes::Decline::Selector`] was the key for BOTH the selector's
/// refusal and the IL parser's, so the published figure was 130,575 reader
/// refusals under a codegen name. The split is the repair, and
/// [`Decline::is_codegen`] is the one place the two are told apart — a consumer
/// re-deriving it from the key spelling is the drift `TuResult::fn_complete`'s
/// doc charges for.
#[test]
fn a_parse_refusal_is_not_a_codegen_refusal() {
    use super::fnbytes::Decline;
    assert!(
        !Decline::Parse.is_codegen(),
        "the parser refused: `select_function` was NEVER CALLED, so no codegen \
         question was asked and none can be — there is no IlFunction to ask it of"
    );
    for d in [
        Decline::OptMode,
        Decline::Selector,
        Decline::GyShape,
        Decline::DataRef,
    ] {
        assert!(
            d.is_codegen(),
            "{d:?} is reached only through `Ok(func)` in `grade_one`, i.e. only \
             for a body the IL parser ACCEPTED. That is the verdict #1464 says \
             does not exist."
        );
    }
}

/// **The two halves of `fnbyte-refused` sum to it, and the control is a count.**
///
/// Written through the same keys `fnbytes::measure` writes, and the identity is
/// PUBLISHED (`fnbyte-refused-split-broken`) rather than asserted only here — a
/// bucket that silently stopped being written would otherwise shrink one side
/// while the other kept printing.
#[test]
fn the_refusal_split_is_published_with_its_own_control() {
    let mut r = mk("split.cpp");
    // The whole FBM key block is gated on there being an FBM denominator at
    // all, so the fixture carries one — a workload with no emitted functions
    // publishes no FBM keys, and that absence is already a documented state
    // rather than a zero.
    r.emit.insert("fnbyte-denominator".into(), 7);
    r.emit.insert("fnbyte-refused".into(), 7);
    r.emit.insert("fnbyte-refused-parse".into(), 6);
    r.emit.insert("fnbyte-refused-codegen".into(), 1);
    let m: std::collections::BTreeMap<&str, String> =
        mk_report(vec![r]).metrics().into_iter().collect();
    assert_eq!(m.get("fnbyte-refused-parse").map(String::as_str), Some("6"));
    assert_eq!(m.get("fnbyte-refused-codegen").map(String::as_str), Some("1"));
    assert_eq!(
        m.get("fnbyte-refused-split-broken").map(String::as_str),
        Some("0"),
        "6 + 1 == 7"
    );

    let mut bad = mk("split.cpp");
    bad.emit.insert("fnbyte-denominator".into(), 7);
    bad.emit.insert("fnbyte-refused".into(), 7);
    bad.emit.insert("fnbyte-refused-parse".into(), 6);
    // …and nothing in `-codegen`: one function has fallen out of the partition.
    let m2: std::collections::BTreeMap<&str, String> =
        mk_report(vec![bad]).metrics().into_iter().collect();
    assert_eq!(
        m2.get("fnbyte-refused-split-broken").map(String::as_str),
        Some("1"),
        "the control fires on a short sum — absence reading as success is the \
         failure this key exists to close"
    );
}

/// **THE HOLE IS THE POINT: `reader` is counted, printed, and never folded into
/// the price** (board #1474).
///
/// The fixture is `src/xdk/nuispeech/mmio.cpp` as this lane measured it — 11
/// emitted functions, 8 byte-exact, 3 behind a parse refusal — plus a planted
/// wrong emit and a planted codegen decline that mmio does not have, so that
/// every field of the partition is exercised by one row and the assertions
/// below cannot pass on a struct that always returns zero.
#[test]
fn the_codegen_column_counts_the_unmeasurable_half_separately() {
    let mut r = mk_factors(TuClass::VocabGap, "mmio.cpp", true, true, true, false, false);
    r.emit.insert("fnbyte-denominator".into(), 14);
    r.emit.insert("fnbyte-exact".into(), 8);
    r.emit.insert("fnbyte-differs".into(), 1);
    r.emit.insert("fnbyte-reloc-differs".into(), 1);
    r.emit.insert("fnbyte-refused-codegen".into(), 1);
    r.emit.insert("fnbyte-refused-parse".into(), 3);
    let c = GapReport::codegen_column(&r);
    assert_eq!(c.exact, 8);
    assert_eq!(c.wrong, 2, "differs AND reloc-differs — lowered, and wrong");
    assert_eq!(c.cg_refused, 1, "reader accepted, emitter declined");
    assert_eq!(
        c.reader, 3,
        "the IL parser refused these three: no codegen question was asked and \
         NONE CAN BE. This is the hole and it is a field, not a footnote."
    );
    assert_eq!(
        c.measured(),
        3,
        "the measurable price is wrong + cg-refused and EXCLUDES the reader \
         column — folding the 3 in would price a distance nothing has measured"
    );
    assert!(!c.partition_broken(), "8 + 2 + 1 + 3 == 14");
}

/// **The partition control fires**, and it fires on the direction that matters:
/// a bucket that stopped being written makes the row SHORT, which reads as a
/// smaller frontier rather than as a broken instrument.
#[test]
fn the_codegen_partition_control_fires_on_a_short_row() {
    let mut r = mk("short.cpp");
    r.emit.insert("fnbyte-denominator".into(), 10);
    r.emit.insert("fnbyte-exact".into(), 4);
    // 6 functions unaccounted for.
    assert!(
        GapReport::codegen_column(&r).partition_broken(),
        "4 != 10 — and the row prints `**PARTITION BROKEN**` beside itself"
    );
}

/// **The six keys are emitted on an EMPTY frontier too.**
///
/// A key that appears only when the frontier is non-empty makes absence read as
/// success: a collector seeing no `frontier-codegen-*` line cannot tell "no
/// frontier" from "the block was removed". Every other control on this page is
/// emitted unconditionally for the same reason.
#[test]
fn the_codegen_column_metrics_survive_an_empty_frontier() {
    let rep = mk_report(vec![mk_factors(
        TuClass::Match,
        "done.cpp",
        true,
        true,
        true,
        false,
        false,
    )]);
    assert!(rep.frontier_codegen().is_empty(), "a `match` is not on the frontier");
    let m: std::collections::BTreeMap<&str, String> = rep.metrics().into_iter().collect();
    for k in [
        "frontier-codegen-denominator",
        "frontier-codegen-exact",
        "frontier-codegen-wrong",
        "frontier-codegen-refused",
        "frontier-codegen-reader",
        "frontier-codegen-ungraded",
        "frontier-codegen-measured",
        "frontier-codegen-partition-broken",
    ] {
        assert_eq!(
            m.get(k).map(String::as_str),
            Some("0"),
            "{k} must print as a measured 0, never vanish"
        );
    }
}

/// **A wrong emit RAISES the codegen price; it can never lower it.**
///
/// The anti-gaming property of this column, as a test. `wrong` is credited to
/// the price and `exact` is not, so a port that emitted a wrong body for a
/// function it currently refuses moves `reader` down by one and `wrong` up by
/// one — the measurable price goes UP, and the only way to lower it is to emit
/// c2's bytes. There is no transformation that shrinks this number by refusing
/// more.
#[test]
fn a_wrong_emit_raises_the_codegen_price_and_never_lowers_it() {
    let base = |wrong: usize, reader: usize, exact: usize| {
        let mut r = mk("x.cpp");
        r.emit.insert("fnbyte-denominator".into(), 5);
        r.emit.insert("fnbyte-exact".into(), exact);
        r.emit.insert("fnbyte-differs".into(), wrong);
        r.emit.insert("fnbyte-refused-parse".into(), reader);
        GapReport::codegen_column(&r)
    };
    let refusing = base(0, 3, 2);
    let wrongly_emitting = base(1, 2, 2);
    let correct = base(0, 2, 3);
    assert_eq!(refusing.measured(), 0);
    assert_eq!(
        wrongly_emitting.measured(),
        1,
        "widening the reader onto a body the port then gets WRONG moves the \
         function out of the unmeasurable column and into the priced one — the \
         price goes UP, which is the direction board #232's repair went"
    );
    assert_eq!(correct.measured(), 0, "and emitting c2's bytes is the only way back down");
    assert!(
        correct.exact > wrongly_emitting.exact,
        "…distinguished from the refusing case by `exact`, so `measured == 0` \
         is never on its own evidence of progress"
    );
}

/// **`grade_one` files a parse refusal under `Decline::Parse` — the one line
/// this whole lane rests on** (board #1473).
///
/// Written against [`super::fnbytes::grade_one`] itself and not against a
/// hand-filled count map, because every other test in this block writes
/// `fnbyte-refused-parse` directly and therefore cannot see the producer. A
/// mutation that re-files the parse refusal back under `Decline::Selector`
/// passes all of them and is caught only here: it puts 130,575 reader refusals
/// back under a codegen name, which is exactly the misreading board #1464
/// recorded.
#[test]
fn grade_one_files_a_parse_refusal_under_the_parser_and_not_the_selector() {
    use super::fnbytes::{grade_one, Decline, FnByte};
    use c2_il::{Block, FnCensus, FnVerdict, IlFunction};

    let row: (FnCensus, Result<IlFunction, &'static str>) = (
        FnCensus {
            index: 0,
            name: Some("?f@@YAXXZ".into()),
            seg_len: 8,
            verdict: FnVerdict::Blocked(Block {
                ctx: "expr",
                byte: Some(0x4F),
                off: 3,
                seg_len: 8,
                aux: 0,
            }),
            hex: Vec::new(),
            hex_mark: 0,
            cflow: "cflow-straight".into(),
            cflow_off: "",
            cfg_admit: "admit-straight",
            eh: "eh-none".into(),
            eh_stmt: String::new(),
            calls: 0,
            dispatch: "disp-expr",
            prod: "prod-not-entered",
            opt_word: Some(c2_il::OPT_WORD_O1),
            emit_name: Some("?f@@YAXXZ".into()),
            no_effect_callee: None,
            no_effect_nothing: false,
        },
        // THE POINT: the IL parser refused, so there is no `IlFunction` and
        // `select_function` is never reached.
        Err("blocked"),
    );
    let tu = super::fnbytes::tu_empty_callees(std::slice::from_ref(&row));
    let g = grade_one(Some(&row), Some(&[0x4E, 0x80, 0x00, 0x20]), &tu, Some(&[]));
    assert_eq!(g.verdict, FnByte::Refused);
    assert_eq!(g.shape, "parse-refused");
    assert_eq!(
        g.decline,
        Some(Decline::Parse),
        "unlowered: the parser refused this body, so the decline stage is \
         `Parse`. Filing it under `Selector` published 130,575 READER refusals \
         under a codegen name for two days (board #1464), and no count-map test \
         in this file can see the difference"
    );
    assert!(
        !g.decline.expect("just asserted").is_codegen(),
        "…and it is therefore counted into `fnbyte-refused-parse`, the column \
         that says a codegen question was never asked"
    );
}

// ---------------------------------------------------- w-phase7: the alias keys

/// One graded TU carrying the alias keys a scan would have written.
fn mk_alias(class: TuClass, keys: &[(&str, usize)]) -> TuResult {
    let mut t = mk("alias");
    t.class = class;
    for (k, n) in keys {
        t.emit.insert((*k).into(), *n);
    }
    t
}

/// **Every alias key rides on the report, INCLUDING the zeroes** — the two
/// alarms most of all.
///
/// `alias-weak-default-disagree` and `alias-weak-not-search-library` have known
/// answer 0, and a key whose value is 0 and a key that is absent read the same
/// way to `sed`. `docs/STATUS.md` trap 5 is that absence reads as success, and
/// an alarm that vanishes when it stops firing is that trap pointed straight at
/// the thing it was written to watch.
#[test]
fn the_alias_keys_print_their_zeroes() {
    let rep = mk_report(vec![mk_alias(TuClass::VocabGap, &[("alias-bound", 7)])]);
    let m: std::collections::BTreeMap<&str, String> = rep.metrics().into_iter().collect();
    for k in [
        "alias-tag10",
        "alias-bound",
        "alias-rt-fail",
        "alias-self",
        "alias-dup",
        "alias-dom-with-body",
        "alias-dom-emitted",
        "alias-null-m1-shape",
        "alias-null-p1-shape",
        "alias-datatu-relocs-alias",
        "alias-emit-names",
        "alias-weak-records",
        "alias-weak-predicted",
        "alias-weak-default-disagree",
        "alias-weak-unpredicted",
        "alias-weak-not-search-library",
        "alias-unrealized",
        "alias-rule-predicted",
        "alias-rule-miss",
        "alias-rule-extra",
        "alias-weak-needed-tus",
        "alias-weak-needed-in-b-and-c",
        "alias-weak-needed-in-frontier",
    ] {
        assert!(m.contains_key(k), "gap-metric {k} must print, zero or not");
    }
    assert_eq!(m.get("alias-bound").map(String::as_str), Some("7"));
    assert_eq!(m.get("alias-weak-default-disagree").map(String::as_str), Some("0"));
}

/// **`alias-weak-needed-*` is an INTERSECTION, not a product.**
///
/// `w-emitp` §5 refused to multiply `B∧C` by a per-TU exact rate and said so,
/// because that is the error that left `B∧C` stale at 107 for weeks. The keys
/// here are the shape that refusal asks for: each is computed by intersecting
/// two per-TU predicates on the same rows, so a TU that needs a weak external
/// and fails B or C contributes to `-tus` and to neither of the others.
#[test]
fn the_weak_external_need_is_intersected_per_tu_and_never_scaled() {
    // Three graded TUs. Only the first is inside `B∧C`; the third needs no
    // weak external at all.
    let bc = [("emit-sec-readable", 1)];
    let mut a = mk_alias(TuClass::VocabGap, &[("alias-rule-predicted", 3)]);
    let mut b = mk_alias(TuClass::VocabGap, &[("alias-rule-predicted", 5)]);
    let c = mk_alias(TuClass::VocabGap, &[("alias-rule-predicted", 0)]);
    for (k, n) in bc {
        a.emit.insert(k.into(), n);
    }
    b.emit.insert("emit-sec-unreadable".into(), 1);
    let rep = mk_report(vec![a, b, c]);
    let m: std::collections::BTreeMap<&str, String> = rep.metrics().into_iter().collect();
    assert_eq!(
        m.get("alias-weak-needed-tus").map(String::as_str),
        Some("2"),
        "TUs needing at least one weak external"
    );
    // The intersection is *at most* the count above and is derived from the
    // rows, never from a rate applied to it.
    let needed: usize = m["alias-weak-needed-tus"].parse().unwrap();
    let in_bc: usize = m["alias-weak-needed-in-b-and-c"].parse().unwrap();
    let in_front: usize = m["alias-weak-needed-in-frontier"].parse().unwrap();
    assert!(in_bc <= needed && in_front <= needed);
}

// ---------------------------------------------------------------------------
// The factor SETS and the join (lane `w-bcgap`, boards #1520–#1524)
// ---------------------------------------------------------------------------

use super::sets;

/// A four-TU report that exercises every clause of every set at least once.
/// The same shape the projection-divergence test uses, plus one TU that fails
/// C, so `factor-b` and `b-and-c` cannot be equal by accident.
fn mk_set_report() -> GapReport {
    mk_report(vec![
        mk_factors(TuClass::Match, "done.cpp", true, true, true, true, false),
        mk_factors(TuClass::VocabGap, "codegen.cpp", true, true, true, false, false),
        mk_factors(TuClass::VocabGap, "noa.cpp", false, true, true, false, false),
        mk_factors(TuClass::CodegenGap, "noa_accepted.cpp", false, true, true, true, false),
        mk_factors(TuClass::VocabGap, "noc.cpp", true, true, false, false, false),
        mk_factors(TuClass::CaptureFail, "gone.cpp", false, false, false, false, false),
    ])
}

/// **The offline TSV view and the live report produce the SAME rows.**
///
/// This is the property that makes `c2rs factors` a *view* of a scan rather
/// than a second implementation of the factorization that can drift from it.
/// Without it, `|somebody's set ∩ B∧C|` computed offline would be an
/// intersection with a lookalike, and the whole point of the exercise is that
/// it is an intersection with the published `B∧C`.
#[test]
fn the_tsv_view_and_the_live_report_are_the_same_rows() {
    let rep = mk_set_report();
    let live = rep.factor_rows();
    let parsed = sets::parse_factors_tsv(&rep.factor_tsv()).expect("the writer's own output");
    assert_eq!(live, parsed, "one definition, two producers — they must agree row for row");
    assert_eq!(live.len(), 5, "5 graded TUs; `gone.cpp` captured nothing and is not a row");
    assert!(
        !live.iter().any(|r| r.src == "gone.cpp"),
        "a capture-fail TU is ABSENT, never a `-----` row (docs/STATUS.md trap 5)"
    );
}

/// **Every set re-derives the count the scan publishes** — the known-answer
/// control, run against the report's own `GAP-METRICS` block rather than
/// against numbers typed into this file.
///
/// A hand-typed expectation grades the test author's arithmetic. Reading the
/// metric block grades the thing the project quotes.
#[test]
fn every_named_set_re_derives_the_gap_metric_it_is_counted_into() {
    let rep = mk_set_report();
    let rows = rep.factor_rows();
    let published: BTreeMap<String, usize> = rep
        .metrics()
        .into_iter()
        .filter_map(|(k, v)| v.parse::<usize>().ok().map(|n| (k.to_string(), n)))
        .collect();
    let checks = sets::check_metrics(&rows, &published);
    let bad: Vec<&sets::MetricCheck> = checks.iter().filter(|c| c.verdict() != "OK").collect();
    assert!(
        bad.is_empty(),
        "these sets disagree with the count the scan publishes for them: {:?}",
        bad.iter().map(|c| (c.key, c.published, c.derived)).collect::<Vec<_>>()
    );
    assert!(
        checks.len() >= 13,
        "the control must actually check something — a control that checks 0 keys and one \
         that passes look identical in a summary line ({} checked)",
        checks.len()
    );
    assert!(
        checks.iter().all(|c| c.published.is_some()),
        "no key may be ABSENT here: the report is its own source, so an absent key means \
         a set was given a `gap-metric` name that does not exist"
    );
}

/// **The reach pool is `B∧C ∖ A∧B∧C` and it is what board #213 prices**, and
/// the frontier pool is a *different* set. The two coincide only while no TU
/// inside `B∧C` fails A while already being accepted, and this fixture has one.
#[test]
fn the_two_pools_board_213_prices_are_not_the_same_set() {
    let rows = mk_set_report().factor_rows();
    let reach = sets::members(&rows, "reach-pool").expect("named set");
    let frontier = sets::members(&rows, "frontier-pool").expect("named set");
    assert_eq!(reach, vec!["noa.cpp", "noa_accepted.cpp"]);
    assert_eq!(
        frontier,
        vec!["noa.cpp"],
        "`noa_accepted.cpp` is in the reach pool and NOT the frontier pool: the port \
         already accepts its contents, so a perfect emit predicate does not add it to the \
         codegen frontier"
    );
    assert_eq!(
        sets::count(&rows, "reach-pool").unwrap()
            - sets::count(&rows, "frontier-pool").unwrap(),
        sets::count(&rows, "projection-divergence").unwrap(),
        "and the difference between the two pools IS the projection divergence"
    );
}

/// **A join that resolves nothing is reported as nothing, never as zeros.**
///
/// This is the lane's whole reason for existing. `src__App.cpp` is not a
/// hypothetical spelling: it is how `work/w-emit/truth` names its files, so a
/// lane carrying a per-TU set out of that pipeline joins on it by default and
/// gets 0 — which reads as "this model buys no reach".
#[test]
fn a_join_on_the_wrong_key_is_loud_and_names_the_normalization_it_did_not_apply() {
    let rows = mk_set_report().factor_rows();
    let cand = sets::parse_candidate("mis", "done__cpp\nnoa__cpp\n").expect("2 names");
    let jc = sets::join(&rows, &cand);
    assert!(jc.is_empty(), "0 resolved, so the caller must refuse to tabulate");
    assert_eq!(jc.unresolved.len(), 2);

    let good = sets::parse_candidate("ok", "done.cpp\nnoa.cpp\nnot_a_tu.cpp\n").expect("3");
    let jg = sets::join(&rows, &good);
    assert_eq!(jg.resolved, vec!["done.cpp", "noa.cpp"]);
    assert_eq!(jg.unresolved, vec!["not_a_tu.cpp"]);
    assert_eq!(jg.absent, 3, "3 graded rows the candidate set never mentions");
    // The hint is a diagnosis and must not be a rewrite: the rows are untouched.
    assert_eq!(rows[0].src, "done.cpp");
}

/// The hint fires on the real `__` spelling and says how many names it would
/// have recovered — and the intersection is still computed on the *unfixed*
/// key, so the hint can never silently change an answer.
#[test]
fn the_hint_counts_what_it_would_have_recovered_without_recovering_it() {
    // Paths with a separator, so the `__` → `/` probe — the real spelling
    // `work/w-emit/truth` uses for its per-TU files — is the one under test.
    let rows = mk_report(vec![
        mk_factors(TuClass::Match, "src/App.cpp", true, true, true, true, false),
        mk_factors(TuClass::VocabGap, "src/sys/Dir.cpp", true, true, true, false, false),
    ])
    .factor_rows();
    let cand = sets::parse_candidate("mis", "src/App.cpp\nsrc__sys__Dir.cpp\n").expect("2");
    let jc = sets::join(&rows, &cand);
    let h = jc.hint.expect("one name is recoverable, so a hint is owed");
    assert!(h.contains("1 of the 1 unresolved"), "it counts: {h}");
    assert!(h.contains("NOT APPLIED"), "and it says it did not do it: {h}");
    let ints = sets::intersections(&rows, &jc.resolved);
    assert_eq!(
        ints["match"], 1,
        "the intersection is over the RESOLVED name only — the hinted one is not folded in"
    );
}

/// **`|cand ∩ S|` can never exceed `|S|` or `|cand|`** — the two bounds that a
/// join bug violates in opposite directions. A key collision (the failure mode
/// a canonicalizer would introduce) shows up as the second one.
#[test]
fn an_intersection_is_bounded_by_both_of_its_operands() {
    let rows = mk_set_report().factor_rows();
    let cand = sets::parse_candidate("all", "done.cpp\ncodegen.cpp\nnoa.cpp\nnoa_accepted.cpp\nnoc.cpp\n")
        .expect("5");
    let jc = sets::join(&rows, &cand);
    assert_eq!(jc.resolved.len(), 5);
    assert_eq!(jc.absent, 0);
    let ints = sets::intersections(&rows, &jc.resolved);
    for s in sets::NAMED_SETS {
        let n = ints[s.name];
        let card = sets::count(&rows, s.name).unwrap();
        assert!(n <= card, "|cand ∩ {}| = {n} > |S| = {card}", s.name);
        assert!(n <= jc.resolved.len(), "|cand ∩ {}| = {n} > |cand|", s.name);
        assert_eq!(
            n, card,
            "and with the WHOLE population as the candidate set every intersection is the \
             set itself — the identity a join bug breaks first ({})",
            s.name
        );
    }
}

/// **A duplicate line is counted, not silently folded.** A candidate set with
/// duplicates was produced per-something-other-than-TU, and its author's "472
/// exact" may not be 472 TUs.
#[test]
fn duplicate_candidate_names_are_counted_rather_than_absorbed() {
    let c = sets::parse_candidate("d", "a.cpp\nb.cpp\na.cpp\n\n# note\na.cpp\n").expect("names");
    assert_eq!(c.lines, 4);
    assert_eq!(c.names.len(), 2);
    assert_eq!(c.duplicates, 2);
}

/// An empty candidate set is refused. It intersects everything at 0, and 0 is a
/// number a reader will publish.
#[test]
fn an_empty_candidate_set_is_refused_rather_than_intersected() {
    let e = sets::parse_candidate("e", "# only a comment\n\n").unwrap_err();
    assert!(e.contains("0 names"), "{e}");
}

/// **The TSV parser refuses a file whose columns moved, is truncated, or whose
/// two redundant encodings of the same bits disagree.**
///
/// Each of the three yields a *smaller or differently-shaped* population, and
/// every one of them would surface downstream as a smaller intersection —
/// i.e. as a weaker model, which is a plausible wrong answer rather than a
/// visible error.
#[test]
fn the_tsv_parser_refuses_a_file_it_cannot_read_positionally() {
    let good = mk_set_report().factor_tsv();

    let no_header: String =
        good.lines().filter(|l| !l.starts_with("# columns:")).collect::<Vec<_>>().join("\n");
    assert!(sets::parse_factors_tsv(&no_header).unwrap_err().contains("columns"));

    let truncated: String = good
        .lines()
        .filter(|l| !l.starts_with("codegen.cpp") && !l.starts_with("noa"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(sets::parse_factors_tsv(&truncated).unwrap_err().contains("graded-rows"));

    let flipped = good.replace("done.cpp\tmatch\t1\t1\t1\t1\t0\tABCD-", "done.cpp\tmatch\t0\t1\t1\t1\t0\tABCD-");
    let e = sets::parse_factors_tsv(&flipped).unwrap_err();
    assert!(e.contains("letters column"), "{e}");

    let only_comments = format!("{}\n# graded-rows 0\n", sets::TSV_COLUMNS);
    assert!(sets::parse_factors_tsv(&only_comments).unwrap_err().contains("0 rows"));
}

/// The column contract in `sets` and the one the writer emits are the SAME
/// string. If they drift, the parser reads bits out of the wrong columns and
/// every set below it is wrong in a way that still looks like a table.
#[test]
fn the_column_contract_is_one_string_and_not_two() {
    assert!(
        mk_set_report().factor_tsv().lines().any(|l| l == sets::TSV_COLUMNS),
        "`sets::TSV_COLUMNS` must be a line the writer actually writes"
    );
}

/// `scrape_metrics` reads only well-formed `gap-metric k v` lines, and a log
/// with none of them yields an empty map — which the CLI reports as a control
/// that checked nothing rather than as a pass.
#[test]
fn a_log_with_no_gap_metric_lines_scrapes_to_nothing_rather_than_to_agreement() {
    assert!(sets::scrape_metrics("gap scan: 878 TUs\n  match 11\n").is_empty());
    let m = sets::scrape_metrics("    gap-metric b-and-c 151\ngap-metric junk xx\n");
    assert_eq!(m.len(), 1);
    assert_eq!(m["b-and-c"], 151);
    let rows = mk_set_report().factor_rows();
    let checks = sets::check_metrics(&rows, &Default::default());
    assert!(
        checks.iter().all(|c| c.verdict() == "ABSENT"),
        "every key is ABSENT, and ABSENT is its own verdict — not silently an OK"
    );
}

/// **`frontier` and `frontier-if-a` exclude `match` TUs**, exactly as
/// `factor_frontier` does. Dropping that clause is the single easiest way to
/// make this module disagree with the scan, and it would disagree by exactly
/// the match count — a number small enough to look like rounding.
#[test]
fn the_frontier_sets_exclude_matches_the_way_the_scan_does() {
    let rep = mk_set_report();
    let rows = rep.factor_rows();
    assert_eq!(
        sets::members(&rows, "frontier").unwrap(),
        rep.factor_frontier().iter().map(|(r, _)| r.src.as_str()).collect::<Vec<_>>(),
        "the set and the scan's own ranked frontier are the same TUs"
    );
    assert_eq!(
        sets::count(&rows, "frontier-if-a").unwrap(),
        rep.factor_frontier_if_a()
    );
    assert!(!sets::members(&rows, "frontier").unwrap().contains(&"done.cpp"));
}

/// The model's joint set, by name, is the same list `factor_all_tus` returns —
/// so the claim "this set IS the match set" is checkable through either.
#[test]
fn the_joint_set_agrees_with_the_report_that_publishes_it_by_name() {
    let rep = mk_set_report();
    let rows = rep.factor_rows();
    assert_eq!(
        sets::members(&rows, "a-and-b-and-c-and-d-or-e").unwrap(),
        rep.factor_all_tus()
    );
    assert_eq!(
        sets::members(&rows, "a-and-b-and-c-and-d").unwrap(),
        rep.factor_abcd_tus()
    );
    assert_eq!(
        sets::members(&rows, "projection-divergence").unwrap(),
        rep.factor_projection_divergence()
    );
}

/// **A `match` that no acceptance path takes is still not on the frontier —
/// and this test exists because a must-fail mutation SURVIVED without it.**
///
/// Dropping `!r.is_match()` from `frontier` / `frontier-if-a` is the single
/// easiest way to make [`sets`] disagree with the scan. Mutation **M1** did
/// exactly that and **every test still passed**, because on the fixture *and on
/// the real 878-TU workload* the clause is currently **inert**: the model's
/// joint `A∧B∧C∧(D∨E)` equals the match set (11 = 11), so every match TU is
/// already excluded by `!r.accepted()` and the match clause never bites. That
/// is the same shape as a column that is zero by construction and read as a
/// measurement — found here in this lane's own code, by mutating it.
///
/// The clause is **not** redundant in general. It is guarded by a *different*
/// control — `factor_control_on_match_tus` requires `D∨E` to have 0 violations
/// over match TUs — and if that control ever went red, this clause is what
/// keeps a byte-exact TU off the list of TUs that need codegen work. So the
/// state below is constructed rather than waited for.
#[test]
fn a_match_no_acceptance_path_takes_is_still_not_on_the_frontier() {
    let rows = mk_report(vec![
        // The state `factor_control_on_match_tus` exists to forbid: byte-exact,
        // inside A∧B∧C, and outside D and E.
        mk_factors(TuClass::Match, "impossible.cpp", true, true, true, false, false),
        mk_factors(TuClass::VocabGap, "real_frontier.cpp", true, true, true, false, false),
    ])
    .factor_rows();
    assert_eq!(
        sets::members(&rows, "frontier").unwrap(),
        vec!["real_frontier.cpp"],
        "a TU that is ALREADY byte-exact must never appear on the list of TUs whose only \
         remaining blocker is codegen breadth"
    );
    assert_eq!(sets::count(&rows, "frontier-if-a").unwrap(), 1);
    assert_eq!(sets::count(&rows, "frontier-pool").unwrap(), 0);
}

// ---------------------------------------------------------------------------
// W-FENCECOUNT — the per-fence hold-out counter (`GapReport::fence_blocks`)
// ---------------------------------------------------------------------------

/// Build a held TU: `vocab-gap`, a cause list, its first blocker, and the two
/// per-TU FnByte counters the exactness read uses. Pure data — the counter, its
/// residues and its arity checks must all be gradeable with no toolchain.
fn mk_fence(src: &str, class: TuClass, causes: &[&str], first: Option<&str>, exact: usize, den: usize) -> TuResult {
    let mut r = mk("fence");
    r.src = src.into();
    r.class = class;
    r.gate_causes = causes.iter().map(|c| c.to_string()).collect();
    r.gate_cause = first.map(str::to_string);
    if den > 0 {
        r.emit.insert("fnbyte-denominator".into(), den);
        r.emit.insert("fnbyte-exact".into(), exact);
    }
    r
}

/// **The vsnprnc shape fires the counter**: a sole-cause TU whose every emitted
/// body is byte-exact lands in `sole`, `exact_tus` AND `exact_bodies` — and a
/// sole-cause TU with a ZERO denominator lands in `sole` only, because
/// exactness over zero bodies is a vacuous read (trap 5), not a conversion
/// shape.
#[test]
fn a_sole_blocked_all_exact_tu_fires_the_fence_counter_and_a_bodiless_one_does_not() {
    let lc = c2_il::func::cause::LOCAL_CALLEE;
    let fb = mk_report(vec![
        mk_fence("vsn.cpp", TuClass::VocabGap, &[lc], Some(lc), 2, 2),
        mk_fence("nobody.cpp", TuClass::VocabGap, &[lc], Some(lc), 0, 0),
        mk_fence("partial.cpp", TuClass::VocabGap, &[lc], Some(lc), 1, 2),
    ])
    .fence_blocks();
    let row = fb.per_cause[lc];
    assert_eq!(row.sole, 3, "all three TUs are sole-blocked by the inline fence");
    assert_eq!(
        row.exact_tus, 1,
        "only the all-exact TU may count as fence-blocks-exact: the bodiless TU is vacuous \
         and the partial TU is not the conversion shape"
    );
    assert_eq!(
        row.exact_bodies, 2,
        "the bodies count is the exact TU's denominator, nothing folded in from the others"
    );
    assert_eq!(row.first_of_multi, 0, "a sole cause is never also a first-of-multi");
    assert_eq!(fb.held_tus, 3, "the held population is all three");
    assert_eq!(fb.cause_firings, 3, "arity: one cause each");
}

/// **A multi-cause TU is a first-blocker row and NEVER a sole or exact one**,
/// even when every emitted body is byte-exact — a first-blocker key is not a
/// distance, and the exact counter must not launder it into one.
#[test]
fn a_multi_cause_tu_counts_as_first_blocker_only_never_as_exact() {
    let lc = c2_il::func::cause::LOCAL_CALLEE;
    let bd = c2_il::func::cause::BODY_DECODE;
    let fb = mk_report(vec![mk_fence("multi.cpp", TuClass::VocabGap, &[bd, lc], Some(bd), 4, 4)])
        .fence_blocks();
    assert_eq!(
        fb.per_cause[bd].first_of_multi, 1,
        "the first blocker (what functions() stops on) takes the first-of-multi row"
    );
    assert_eq!(
        fb.per_cause[bd].sole + fb.per_cause[bd].exact_tus,
        0,
        "a TU with two causes is not held by one cause and nothing else"
    );
    assert!(
        !fb.per_cause.contains_key(lc) || fb.per_cause[lc] == Default::default(),
        "the co-blocker gets no row at all: attributing a multi-cause TU to every cause \
         would double-count the totality identity"
    );
    assert_eq!(
        fb.per_cause[bd].exact_bodies, 0,
        "byte-exact bodies on a multi-cause TU must not reach the exact counter"
    );
}

/// **The controls are counts and each malformed row lands in exactly one** —
/// totality (`held == attributed + arity_broken`) survives adversarial input
/// rather than being an identity over well-formed rows only.
#[test]
fn fence_controls_catch_malformed_rows_residues_and_class_disagreement() {
    let lc = c2_il::func::cause::LOCAL_CALLEE;
    let bd = c2_il::func::cause::BODY_DECODE;
    let rep = mk_report(vec![
        // well-formed sole
        mk_fence("ok.cpp", TuClass::VocabGap, &[lc], Some(lc), 0, 0),
        // first blocker missing entirely -> arity_broken
        mk_fence("nofirst.cpp", TuClass::VocabGap, &[lc, bd], None, 0, 0),
        // first blocker not a member of its own list -> arity_broken
        mk_fence("wrongfirst.cpp", TuClass::VocabGap, &[lc, bd], Some("no-such-cause"), 0, 0),
        // vocab-gap with an EMPTY cause list -> the named residue (known 0 live)
        mk_fence("nocause.cpp", TuClass::VocabGap, &[], None, 0, 0),
        // decodes-but-not-match -> outside the fence family
        mk_fence("cg.cpp", TuClass::CodegenGap, &[], None, 0, 0),
        // ...and one CARRYING a cause anyway -> class_disagree
        mk_fence("cgbad.cpp", TuClass::CodegenGap, &[lc], Some(lc), 0, 0),
        // match TUs: one clean (checked), one carrying a cause (known-0 alarm)
        mk_fence("m.cpp", TuClass::Match, &[], None, 0, 0),
        mk_fence("mbad.cpp", TuClass::Match, &[lc], Some(lc), 0, 0),
        // capture-fail must be invisible to every counter
        mk_fence("cf.cpp", TuClass::CaptureFail, &[lc], Some(lc), 0, 0),
    ]);
    let fb = rep.fence_blocks();
    assert_eq!(fb.held_tus, 3, "held = the three vocab-gap TUs with causes");
    assert_eq!(fb.arity_broken, 2, "both malformed first-blocker rows are counted, not guessed");
    let attributed: usize = fb.per_cause.values().map(|c| c.sole + c.first_of_multi).sum();
    assert_eq!(
        fb.held_tus,
        attributed + fb.arity_broken,
        "totality: every held TU is attributed to exactly one row or counted arity-broken"
    );
    assert_eq!(fb.residue_no_cause, 1, "the causeless vocab-gap TU is the named residue");
    assert_eq!(fb.decodes_not_match, 2, "both decoding non-match TUs are outside the family");
    assert_eq!(fb.class_disagree, 1, "a decoding TU carrying a cause is the known-0 alarm");
    assert_eq!(fb.on_match_tu, 1, "a match TU carrying a cause is the known-0 agreement alarm");
    assert_eq!(fb.match_tus_checked, 1, "the clean match TU is positively counted as checked");
    assert_eq!(
        fb.cause_firings, 5,
        "arity counts contents: 1 (ok) + 2 + 2 from the malformed rows; nothing from \
         residues, non-vocab classes or capture-fail"
    );
}

/// **The metric keys ride with zeros included over the closed cause
/// vocabulary** — a fence key that never fires and one that was never added
/// must be different readings — and an observed cause the closed list does not
/// carry is still printed rather than dropped.
#[test]
fn fence_metric_keys_print_zeros_for_the_closed_vocabulary_and_never_drop_an_observed_cause() {
    let lc = c2_il::func::cause::LOCAL_CALLEE;
    let rep = mk_report(vec![
        mk_fence("vsn.cpp", TuClass::VocabGap, &[lc], Some(lc), 2, 2),
        mk_fence("new.cpp", TuClass::VocabGap, &["some-future-cause"], Some("some-future-cause"), 0, 0),
    ]);
    let m: BTreeMap<&str, String> = rep.metrics().into_iter().collect();
    assert_eq!(
        m["fence-blocks-exact:locally-defined-callee"], "1",
        "the inline fence's exact row must carry the firing TU"
    );
    assert_eq!(
        m["fence-blocks-exact-bodies:locally-defined-callee"], "2",
        "…and its bodies count, in body units"
    );
    assert_eq!(
        m["fence-blocks-sole:gl-stop-26-introduced"], "0",
        "a cause that never fires still prints, at zero — absence must not read as success"
    );
    assert_eq!(
        m["fence-blocks-sole:some-future-cause"], "1",
        "a cause the closed list does not carry is printed from the data, never dropped"
    );
    for k in [
        "fence-held-tus",
        "fence-cause-firings",
        "fence-residue-no-cause",
        "fence-decodes-not-match",
        "fence-class-disagree",
        "fence-on-match-tu",
        "fence-match-tus-checked",
        "fence-arity-broken",
        "fence-accounting-broken",
    ] {
        assert!(m.contains_key(k), "control key `{k}` must ride with the rows");
    }
    assert_eq!(m["fence-accounting-broken"], "0", "totality holds on this input");
    // Every closed-vocabulary cause carries all four rows.
    for cause in crate::gap::FENCE_CAUSES {
        for fam in ["sole", "exact", "exact-bodies", "first"] {
            let key = format!("fence-blocks-{fam}:{cause}");
            assert!(
                m.contains_key(key.as_str()),
                "closed-vocabulary cause `{cause}` is missing its `{fam}` row"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// w-stmt5 — the emitted cross as a SERIES, and §14.2 step 5's boundary scored
// ---------------------------------------------------------------------------

/// **The bucketer, over every shape of input it can receive**, including the two
/// it must NOT split. `cflow_series_bucket` is the scan's own function, called
/// here rather than restated — a restated copy is how two files come to hold two
/// opinions of one predicate with no git conflict between them.
#[test]
fn the_series_bucketer_never_shards_and_splits_the_residue_on_the_suffix() {
    let b = crate::gap::classify::cflow_series_bucket;
    assert_eq!(b("cflow-straight"), "straight|expr");
    assert_eq!(b("cflow-straight+expr-modeled"), "straight|modeled");
    assert_eq!(b("cflow-if-1+expr-modeled"), "if-1|modeled");
    assert_eq!(b("cflow-loop"), "loop|expr");
    // **THE anti-sharding property.** Undecoded bodies carry a `cf-*` blocker
    // key and several of those are per-TU sharded (`GAPS.md` §6). Every one of
    // them is one bucket, so the series can never grow a bucket per input.
    for k in ["cf-expr-0x59", "cf-scope-depth", "cf-no-body", "cf-tail"] {
        assert_eq!(b(k), "undecoded|-", "{k} must not shard the series");
    }
    // **`ends_with`, not `contains`** — `w-stmt5` mutant M2. `CfBody::key`
    // appends the suffix, so a `contains` test would also fire on a shape name
    // that had the string inside it and would silently move the counterfactual
    // column into the expression one.
    assert_eq!(b("cflow-+expr-modeled-tail"), "+expr-modeled-tail|expr");
}

/// **The series partitions the blocked-emitted population exactly**, and the
/// control is counted at the same site in the same unit as the buckets. A
/// totality control counted in two different units reads 0 forever and is green
/// for the wrong reason — `w-tag02`'s rule, and the reason this asserts the
/// control against the sum rather than against a recomputation.
#[test]
fn the_emitted_series_partitions_and_reproduces_the_published_pair() {
    let rep = mk_report(vec![mk_cflow(
        &[],
        &[
            ("emit-cflow-shape|straight|expr", 70),
            ("emit-cflow-shape|straight|modeled", 3),
            ("emit-cflow-shape|if-1|expr", 16),
            ("emit-cflow-shape|if-1|modeled", 4),
            ("emit-cflow-shape|switch|modeled", 5),
            ("emit-cflow-shape|undecoded|-", 2),
            // the two published keys, from the same site
            ("emit-cflow-branchy", 25),
            ("emit-cflow-branchy-modeled", 9),
        ],
    )]);
    let (rows, accounted) = rep.cflow_emitted_series();
    assert_eq!(accounted, 100, "every blocked emitted function lands in one cell");
    assert_eq!(rows.first().map(|r| r.0.as_str()), Some("straight|expr"), "sorted by size");
    // **THE SELF-CHECK the series exists to survive.** The collapsed pair must
    // be reproducible from the series by addition, or the two are measuring
    // different sets and the series is a second opinion rather than the same
    // cross. `-branchy` is everything but `straight` and `undecoded`.
    let branchy: usize = rows
        .iter()
        .filter(|(k, _)| !k.starts_with("straight") && !k.starts_with("undecoded"))
        .map(|(_, n)| *n)
        .sum();
    let modeled: usize = rows
        .iter()
        .filter(|(k, _)| k.ends_with("|modeled") && !k.starts_with("straight"))
        .map(|(_, n)| *n)
        .sum();
    assert_eq!((branchy, modeled), rep.cflow_emitted_counterfactual());
    // …and the cell the collapse throws away is not small.
    let straight_modeled: usize = rows
        .iter()
        .filter(|(k, _)| k == "straight|modeled")
        .map(|(_, n)| *n)
        .sum();
    assert!(straight_modeled > 0, "the cell `-branchy` excludes by name");
}

/// **`emit_blockers` restricted to what a reader can reach.** The full ranking
/// is over 113,612 functions and the restricted one over 3,062, and the point of
/// the method is that the two are not the same order — a lane dispatched off the
/// first is dispatched off rows the second does not contain.
#[test]
fn the_modeled_widening_order_is_a_different_order_from_the_full_one() {
    let rep = mk_report(vec![mk_cflow(
        &[],
        &[
            ("emit-cflow-modeled-key|call-arg-multi-sym:eof", 801),
            ("emit-cflow-modeled-key|data-sym-unresolved:eof", 529),
            ("emit-cflow-modeled-key|expr-brfalse", 3),
        ],
    )]);
    let (rows, accounted) = rep.cflow_emitted_modeled_keys();
    assert_eq!(accounted, 1333);
    assert_eq!(rows.first().map(|r| r.0.as_str()), Some("call-arg-multi-sym:eof"));
    // The row §14.2 step 5 is named after is ABSENT from the reachable order —
    // which is the finding, expressed as the shape of the data rather than as a
    // number that could drift.
    assert!(
        !rows.iter().any(|(k, _)| k == "body-cflow-label"),
        "a key absent from the reachable order converts nothing, whatever its rank in the full one"
    );
}

/// **The boundary's histogram and its alarm.** The alarm is the only row here
/// that is an alarm; a large `refuse-back-edge` is the fence working. The alarm
/// is emitted unconditionally and the cells only when they occur, and this
/// pins the asymmetry: an alarm that vanished must not read as an alarm that
/// fired zero times.
#[test]
fn the_step5_boundary_publishes_its_cells_and_always_publishes_its_alarm() {
    let mut t = mk("cflow");
    for (k, n) in [
        ("admit-straight|IN-CLASS", 187),
        ("admit-straight|BLOCKED", 79),
        ("refuse-back-edge|IN-CLASS", 10),
        ("refuse-back-edge|BLOCKED", 88),
    ] {
        t.fn_cfg_admit.insert(k.into(), n);
    }
    // **The boundary axis has its OWN map and this test proves the separation
    // holds**: a `cflow` row is added beside it and must not be counted here,
    // and `cflow_residue_control` must not count the boundary's rows either.
    // That collision is not hypothetical — it shipped for one commit and moved
    // `cflow-residue-inclass-offclass` from 517,425 to 1,222,684.
    t.fn_cflow.insert("cflow-straight+expr-modeled|IN-CLASS".into(), 3);
    t.fn_cflow.insert("cflow-loop|IN-CLASS".into(), 5);
    let rep = mk_report(vec![t]);
    let (rows, disagree) = rep.cfg_admit_histogram();
    assert_eq!(disagree, 0, "the consistency alarm");
    assert_eq!(rows.iter().map(|r| r.1).sum::<usize>(), 364, "the cflow rows are NOT in it");
    // …and the separation the other way: the residue control sees only its own
    // map, so the boundary's 364 rows leave it at the 5 the `cflow` map holds.
    assert_eq!(rep.cflow_residue_control(), (3, 5));
    let m: std::collections::BTreeMap<&str, String> = rep.metrics().into_iter().collect();
    assert_eq!(m.get("step5-consistency-alarms").map(String::as_str), Some("0"));
    assert_eq!(m.get("step5-accounted").map(String::as_str), Some("364"));
    // **The two-sided price is a published cell, not a footnote.** Ten bodies
    // the port already emits byte-exactly are refused by this boundary; a
    // reading of the fence that quoted only the BLOCKED column would be the
    // one-sided pricing `CLAUDE.md` bans.
    assert_eq!(m.get("step5-refuse-back-edge-IN-CLASS").map(String::as_str), Some("10"));
    // …and a scan with no disagreement still emits the alarm, which is what
    // distinguishes it from a scan where the key was dropped.
    let empty = mk_report(vec![mk_cflow(&[], &[])]);
    let m2: std::collections::BTreeMap<&str, String> = empty.metrics().into_iter().collect();
    assert_eq!(m2.get("step5-consistency-alarms").map(String::as_str), Some("0"));
}

// ---------------------------------------------------------------------------
// Lane `w-guards` — guards for the three census surfaces board #3199 measured
// as having NOTHING that could fail on them
// ---------------------------------------------------------------------------

/// **Three clauses this instrument's own ranking is built from shipped with no
/// test that could fail on them** — `docs/rungs/2026-08-16-bind.md` §8, board
/// **#3199**. `w-bind16` registered four mutants' colours before running them
/// and **three came back GREEN against a registered RED**:
///
/// | id | site | mutation | #3199 |
/// |---|---|---|---|
/// | M1 | `c2-il .../shapes/calls.rs:431` | `syms > 1` → `syms > 2` | **GREEN** |
/// | M2 | `c2-core/src/codegen/calls.rs:1815` | `count != 1` → `count > 2` | RED |
/// | M3 | `c2-il/src/func/bind.rs:886` | drop `resolve_data`'s linkage gate | **GREEN** |
/// | M4 | `c2-il/src/func/census.rs:1211` | swap `DATA_SYM_UNRESOLVED` ⇄ `DATA_SYM_LINKAGE` | **GREEN** |
///
/// **M4 is the worst placed.** The two census keys board **#3177**'s reachable
/// widening order is *built from* — `data-sym-unresolved` and
/// `data-sym-not-extern` — can be **exchanged for each other** with nothing
/// failing, in an instrument that is **machine-read**: `scan.rs` writes
/// `format!("emit-cflow-modeled-key|{}", f.verdict.key())` and
/// [`super::report::GapReport::cflow_emitted_modeled_keys`] strips that prefix
/// back off. `w-loo` measured five of six published rankings at ρ ≈ +0.047 —
/// noise (**#3135**) — and an unguarded key in a machine-read instrument is
/// exactly how that happens without anyone noticing. #3177's own closing words
/// are *"`board_audit.sh` checks anchors and numbers; nothing checks a key
/// against the population it is quoted over"*; #3199 sharpens it to **nothing
/// checks these two keys are not exchanged for each other**.
///
/// # Why the guards live here and not beside the clauses
///
/// All three mutation sites are in `crates/c2-il`, which this lane does not
/// own. `parse_segment_detail`, `DATA_SYM_UNRESOLVED`, `DATA_SYM_LINKAGE` and
/// `gl_extern_data_names` are all `pub(crate)` there and unreachable from here.
/// So the guards drive hand-built synthetic bundles through the **public**
/// `IlBundle::census_functions()` and assert on the **observable key string** —
/// [`c2_il::FnVerdict::key`], which is verbatim what `scan.rs` concatenates
/// into the instrument's key. That is the stronger form: it guards the string
/// the instrument publishes rather than the constant behind it, so a mutation
/// that renames the constant *and* its uses is still caught if the published
/// key moves.
///
/// # The cells, and why each pair discriminates
///
/// One transcript, four `.gl` variants and two body variants. Cells **A** and
/// **B** differ in **exactly one thing** — whether `.gl` names the token — and
/// that is precisely what M4 destroys. Cells **B** and **C** differ in
/// **exactly one byte** — the `.gl` linkage byte — and that is precisely what
/// M3 destroys. The **n = 1 / 2 / 3** series is the arity fence at rank 1 of
/// #3177's reachable order, **1,296 functions**, and that is what M1 weakens.
///
/// Every guard here **prints and asserts its count of discriminating cells**,
/// because absence read as success is this repo's most-recorded failure and a
/// bundle that quietly stopped producing a census row would otherwise pass.
#[cfg(test)]
mod wr1_census_key_guards {
    use super::{mk, mk_report};
    use c2_il::IlBundle;

    /// `static Licenses sLicense("system/src/tomcrypt", (Licenses::Requirement)0);`
    /// lowering to `??__EsLicense@@YAXXZ` — a **real** `c2.dll` capture at the
    /// 878-TU workload's own flags (`/nologo /c /GR /O1 /Oi /EHsc`), `4F 1F`
    /// header through `4D` module end.
    ///
    /// Transcribed from `c2-il`'s own `func::body::wr1_dyninit::TOMCRYPT_DYNINIT`
    /// rather than referenced, because that module is `#[cfg(test)]` inside a
    /// crate this lane does not own and is not reachable across the crate
    /// boundary. **It is a copy of a capture, not a second source of truth**: if
    /// `c2-il`'s readers change under it these guards break, and that is the
    /// guard working — the response is to re-derive the transcript from the
    /// capture, never to delete the test.
    ///
    /// Its body decodes to
    /// `MultiArgTailCall { arg_sources: [SymAddr(0xF909), SymAddr(0xFC09), Lit(0)] }`
    /// with callee token `0xEA09`, and its body marker is the **bare** `4C`
    /// every `??__E`/`??__F` thunk carries.
    const DYNINIT: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0x20, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00,
        0x4F, 0x33, 0x0D, 0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01,
        0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18, 0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38,
        0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D, 0x08, 0x00, 0x0F,
        0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x03, // block start, line 3
        0x53, 0x53, 0x26, 0xFA, 0x09, // SS SS, result-ref
        0x46, // formals: EMPTY
        0x4C, 0x53, // the BARE `LO`, then the body's SS
        0x26, 0xEA, 0x09, // push the callee
        0x26, 0xF9, 0x09, 0x2C, 0xA6, 0x43, 0x81, 0x20, 0x00, // &sLicense …
        0x99, 0x86, 0x43, 0x8D, 0x20, 0x00, // … bound as the receiver
        0xBD, 0xA6, 0x43, 0x81, 0x20, 0x00, 0x80, 0x07, 0x10, 0x00, 0x00, // CALL
        0x33, 0x86, 0x41, 0x83, 0x20, 0x00, // arg 2: literal 0
        0x55, 0x86, 0x41, 0x83, 0x20,
        0x26, 0xFC, 0x09, 0x2C, 0x86, 0x43, 0x85, 0x20, 0x00, // arg 1: the string
        0x55, 0x86, 0x43, 0x85, 0x20,
        0x4C, 0x4B, // void call end
        0x3A, 0xFB, 0x09, 0x54, 0x02, 0x29, 0xFB, 0x09, // return plumbing
        0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, // function tail
        0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x04, // module end
        0x53, 0x54, 0x00, 0x4D,
    ];

    /// The `arg 1` production, `26 <tok> 2C …` — one **symbol-address**
    /// argument. Sliced out by value so the n-series can substitute for it.
    const SYM_ARG_FC09: [u8; 14] = [
        0x26, 0xFC, 0x09, 0x2C, 0x86, 0x43, 0x85, 0x20, 0x00,
        0x55, 0x86, 0x43, 0x85, 0x20,
    ];
    /// The `arg 2` production, `33 … 55 …` — one **literal** argument.
    const LIT_ARG: [u8; 11] = [
        0x33, 0x86, 0x41, 0x83, 0x20, 0x00, 0x55, 0x86, 0x41, 0x83, 0x20,
    ];

    /// A `.gl` **symbol** record: `<kind 04> <token> <sep 00> <name> 00 <TYPE>`,
    /// the framing `gl_symbol_index` binds a token to a name through.
    fn sym_rec(tok: [u8; 2], name: &str) -> Vec<u8> {
        let mut v = vec![0x04, tok[0], tok[1], 0x00];
        v.extend_from_slice(name.as_bytes());
        v.push(0x00);
        v.extend_from_slice(&[0x82, 0x07, 0x04]);
        v
    }

    /// The same record **plus the type trailer `gl_extern_data_names` reads the
    /// linkage byte out of** — `80 <kind> 00 02 <linkage>`.
    ///
    /// `linkage` is the ONE byte that separates cell **B** from cell **C**.
    /// `02` is undefined-external (an address the port may reference without
    /// emitting a section); `01` is defined-here and `04` is `static`, and both
    /// cost a whole extra section (`docs/IL_CALL_IN_EXPR.md` §17.2 item 7).
    fn data_rec(tok: [u8; 2], name: &str, linkage: u8) -> Vec<u8> {
        let mut v = vec![0x04, tok[0], tok[1], 0x00];
        v.extend_from_slice(name.as_bytes());
        v.push(0x00);
        v.extend_from_slice(&[0x80, 0x00, 0x00, 0x02, linkage]);
        v
    }

    /// [`DYNINIT`] with the **bare** `4C` body marker replaced by the composed
    /// `4C 4F 11`, so `body_start_is_bare` is false and the `two_sym_thunk`
    /// exemption at `calls.rs:431` cannot fire on `syms == 2`.
    fn composed_lo(seg: &[u8]) -> Vec<u8> {
        let mut v = seg.to_vec();
        let at = v
            .windows(2)
            .position(|w| w == [0x4C, 0x53])
            .expect("the bare `4C` body marker followed by the body's SS");
        v.splice(at..at + 1, [0x4C, 0x4F, 0x11]);
        v
    }

    /// Substitute one argument production for another, once, by value.
    fn swap_arg(seg: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
        let mut v = seg.to_vec();
        let at = v
            .windows(from.len())
            .position(|w| w == from)
            .expect("the argument production to substitute for");
        v.splice(at..at + from.len(), to.iter().copied());
        v
    }

    fn bundle(seg: &[u8], gl: Vec<u8>) -> IlBundle {
        let mut b = IlBundle::new("_CL_wguards");
        let mut ex = Vec::new();
        ex.extend_from_slice(&c2_il::EX_MAGIC);
        ex.extend_from_slice(&[0x00; 8]);
        ex.extend_from_slice(seg);
        b.set("ex", ex);
        b.set("gl", gl);
        b.set("sy", Vec::new());
        b.set("in", Vec::new());
        b
    }

    /// The census key for a one-function bundle, **with the vacuity check
    /// inside it**.
    ///
    /// A synthetic bundle that stops producing a census row would make every
    /// assertion below unreachable and the whole module would go green for the
    /// reason `docs/STATUS.md` trap 5 names. So "there is exactly one row" is a
    /// *named* failure with its own message rather than an `unwrap`, and the
    /// caller can never observe a missing row as a passing key comparison.
    fn key_of(cell: &str, seg: &[u8], gl: Vec<u8>) -> String {
        let rows = bundle(seg, gl)
            .census_functions()
            .unwrap_or_else(|| panic!("cell {cell}: the bundle produced NO census at all — \
                 the guard graded 0 discriminating cells and is vacuous, which is \
                 not a pass"));
        assert_eq!(
            rows.len(),
            1,
            "cell {cell}: expected exactly ONE census row from the one-function \
             transcript, got {} — a guard whose bundle stopped segmenting grades \
             nothing and must fail loudly rather than pass",
            rows.len()
        );
        rows[0].0.verdict.key()
    }

    /// `.gl` naming only the callee: the two symbol-argument tokens are
    /// **unnamed**, so `Bindings::resolve` returns `None` for them. **Cell A.**
    fn gl_callee_only() -> Vec<u8> {
        sym_rec([0xEA, 0x09], "?ctor@@YAXXZ")
    }

    /// `.gl` naming the callee and both symbol arguments, with the first
    /// argument's record carrying `linkage`. `02` is cell **C**, `01` cell
    /// **B**; **nothing else differs between them.**
    fn gl_named(linkage: u8) -> Vec<u8> {
        let mut g = sym_rec([0xEA, 0x09], "?ctor@@YAXXZ");
        g.extend(data_rec([0xF9, 0x09], "?objA@@3HA", linkage));
        g.extend(data_rec([0xFC, 0x09], "?strB@@3HA", 0x02));
        g
    }

    const UNRESOLVED: &str = "data-sym-unresolved:eof";
    const NOT_EXTERN: &str = "data-sym-not-extern:eof";
    /// **W-FENCE163** — the key a string-literal argument takes when the narrow
    /// prefix does not admit it.
    const STRLIT_FENCED: &str = "data-sym-strlit-fenced:eof";
    /// The key the body reports when every symbol argument resolves.
    const MULTIARG: &str = "multiarg-tail-call";

    /// A real narrow (`char`) literal name and a real wide (`wchar_t`) one,
    /// **captured** from `c2.dll` at the workload profile for `d("aa")` and
    /// `d(L"aa")` (measured `d28326b4`). The mangling's `_0`/`_1` field is the
    /// width and it is the whole difference between the two cells below.
    const NARROW_LIT: &str = "??_C@_02DKCKIIND@aa?$AA@";
    const WIDE_LIT: &str = "??_C@_15KDHKKBLG@a?$AAa?$AA?$AA?$AA@";

    /// `.gl` naming the callee and both symbol arguments, with the **first**
    /// argument's record replaced by a STRING-LITERAL record: separator `25`
    /// instead of `00`, and `name` in place of `?objA@@3HA`. Everything else is
    /// `gl_named(0x01)` byte for byte, so this is cell **B** with one record
    /// substituted.
    ///
    /// Linkage stays `01` deliberately — a literal IS defined by the real obj,
    /// which is exactly why admitting its *name* cannot license emitting its
    /// *section*.
    fn gl_strlit(name: &str) -> Vec<u8> {
        let mut g = sym_rec([0xEA, 0x09], "?ctor@@YAXXZ");
        let mut lit = data_rec([0xF9, 0x09], name, 0x01);
        lit[3] = 0x25; // the string-literal name separator
        g.extend(lit);
        g.extend(data_rec([0xFC, 0x09], "?strB@@3HA", 0x02));
        g
    }

    /// **W-FENCE163's cells, beside cell B — and the recorded response to
    /// `2026-08-16-guards.md` §8.1.**
    ///
    /// §8.1 wrote an interaction row in advance: *"a prefix-gated string-literal
    /// admission would turn cell B RED"*. It did **not** — measured at
    /// `d28326b4`, all five guards in this module stayed GREEN — and the reason
    /// is the row's trigger condition, not its existence: cell B's refused name
    /// is `?objA@@3HA`, which carries no `??_C@` prefix at all, so no
    /// prefix-gated admission can reach it. An advance-written interaction row
    /// was **right to exist and wrong in its trigger**; the response it named is
    /// executed here regardless, and nothing above was weakened or deleted.
    ///
    /// Four cells, each differing from cell B in exactly one substituted `.gl`
    /// record, and **three** distinct keys — measured at `d28326b4`:
    ///
    /// | cell | first argument's record | key |
    /// |---|---|---|
    /// | **B** (above, unchanged) | `?objA@@3HA`, sep `00`, linkage `01` | `data-sym-not-extern:eof` |
    /// | **C** (above, unchanged) | `?objA@@3HA`, sep `00`, linkage `02` | `callee-unresolved-tail-call:eof` |
    /// | **B-narrow** | `??_C@_0…`, sep `25`, linkage `01` | `callee-unresolved-tail-call:eof` |
    /// | **B-wide** | `??_C@_1…`, sep `25`, linkage `01` | `data-sym-strlit-fenced:eof` |
    ///
    /// **Cell C is the yardstick and that is the whole design of this test.**
    /// This transcript is the two-symbol `??__E` thunk, so it refuses *downstream*
    /// of the data-symbol gate whatever happens at the gate — `callee-unresolved-tail-call`
    /// is not "in class", it is **"past the gate"**, and cell C is what past-the-gate
    /// looks like when the name is an honest undefined external. So the claim
    /// *"the narrow literal is admitted"* is stated as `B-narrow == C` rather
    /// than against a hard-coded in-class key, which would have been a claim
    /// about the arity fence instead. B-narrow against B is the widening itself
    /// (the same linkage byte `01`, admitted only because the name is a narrow
    /// literal); B-narrow against B-wide is the prefix gate (MF3). Every
    /// assertion is on the published key string, like every guard here, so a
    /// mutation that renames the constant is still caught.
    #[test]
    fn the_string_literal_admission_is_narrow_only_and_leaves_cell_b_alone() {
        let cell_b = key_of("B (named, linkage 01)", DYNINIT, gl_named(0x01));
        let cell_c = key_of("C (named, linkage 02)", DYNINIT, gl_named(0x02));
        let narrow = key_of("B-narrow (`??_C@_0…`, sep 25)", DYNINIT, gl_strlit(NARROW_LIT));
        let wide = key_of("B-wide (`??_C@_1…`, sep 25)", DYNINIT, gl_strlit(WIDE_LIT));

        assert_eq!(
            cell_b, NOT_EXTERN,
            "cell B is UNCHANGED by the string-literal admission and that is \
             `w-guards` §8.1's advance row scored, not skipped: its refused name \
             `?objA@@3HA` carries no `??_C@` prefix, so a prefix-gated admission \
             cannot reach it. Got `{cell_b}` — if this moved, the admission is \
             not prefix-gated and it is eating the linkage gate whole"
        );
        assert_eq!(
            narrow, cell_c,
            "the NARROW literal at linkage `01` must reach the SAME key as an \
             undefined external at linkage `02` (`{cell_c}`) — that is exactly \
             what `resolve_data`'s prefix clause does: it returns the name \
             *before* the linkage gate is consulted, so the literal is treated \
             as relocatable although the real obj defines it. Got `{narrow}`. \
             A refusal here retires the whole widening; and because this cell's \
             separator is `25`, it also fails if `gl.rs`' `NAME_SEPARATORS` \
             stops admitting the string-literal separator"
        );
        assert_ne!(
            narrow, cell_b,
            "…and the narrow literal must NOT read like cell B, whose linkage \
             byte is the same `01`. Equal keys mean the prefix clause is not \
             firing and the +163 is coming from somewhere else"
        );
        assert_eq!(
            wide, STRLIT_FENCED,
            "MF3: the WIDE literal — the same record but for the mangling's \
             width field — must keep refusing, under the fence's own key. Got \
             `{wide}`. `w-section` §3.3 measured wide **0** of 1,458: nothing has \
             graded a wide literal's emit, and widening the prefix to `??_C@` \
             admits it in silence"
        );
        assert_ne!(
            wide, cell_c,
            "…and the wide literal must not reach the past-the-gate key either, \
             or the prefix gate is admitting both widths"
        );

        // Four cells, THREE distinct keys, and the partition is named: cell B
        // alone at `data-sym-not-extern`, cell C and B-narrow together past the
        // gate, B-wide alone at the fence's key. Asserted as a count because a
        // collapse anywhere makes the pairs above vacuous.
        let distinct: std::collections::BTreeSet<&str> =
            [cell_b.as_str(), cell_c.as_str(), narrow.as_str(), wide.as_str()]
                .into_iter()
                .collect();
        assert_eq!(
            distinct.len(),
            3,
            "4 cells graded, 3 distinct keys expected, got {}: {distinct:?}",
            distinct.len()
        );
        // The in-class key is spelled here so the module's vocabulary stays
        // complete even though this transcript refuses downstream of the gate:
        // `MULTIARG` is what a ONE-symbol body reports (the M1 series' n=1 cell),
        // and no cell in this test may reach it — a literal cell that came back
        // in class would mean the two-symbol arity fence had also gone.
        assert!(
            ![cell_b.as_str(), cell_c.as_str(), narrow.as_str(), wide.as_str()]
                .contains(&MULTIARG),
            "no cell in this two-symbol transcript may report `{MULTIARG}`: \
             {distinct:?}"
        );
        // …and the three `.gl` variants must differ ONLY in the substituted
        // record, or the cells are not isolating it.
        assert_eq!(
            gl_strlit(NARROW_LIT).len() - NARROW_LIT.len(),
            gl_strlit(WIDE_LIT).len() - WIDE_LIT.len(),
            "the two literal cells' `.gl` must be identical apart from the name \
             run itself — anything else and the pair varies more than the width"
        );
    }

    /// **M4 — the two keys #3177's ranking is BUILT FROM are not
    /// interchangeable.**
    ///
    /// Registered RED by `w-bind16`, measured **GREEN**: swapping
    /// `DATA_SYM_UNRESOLVED` and `DATA_SYM_LINKAGE` at `census.rs:1211` failed
    /// nothing in 1,643 tests. This is the test that fails.
    ///
    /// Two cells differing in **one fact** — whether `.gl` names token
    /// `0xF909`. Not one cell: a single assertion cannot tell a swap from a
    /// rename, and #3147 is the standing correction that one cell gives a number
    /// right for that cell and wrong as a rule.
    #[test]
    fn the_two_data_symbol_census_keys_are_not_interchangeable() {
        let unnamed = key_of("A (token unnamed in `.gl`)", DYNINIT, gl_callee_only());
        let named_wrong_linkage = key_of("B (named, linkage 01)", DYNINIT, gl_named(0x01));

        assert_eq!(
            unnamed, UNRESOLVED,
            "M4: a data-symbol argument whose token has NO `.gl` name at all must \
             be filed under `{UNRESOLVED}`. Getting `{unnamed}` means the \
             `DATA_SYM_UNRESOLVED` arm of `census.rs`' `sym_fail` probe now \
             publishes the OTHER key — and board #3177's reachable widening \
             order is built by counting these two keys apart"
        );
        assert_eq!(
            named_wrong_linkage, NOT_EXTERN,
            "M4: a data-symbol argument whose token DOES resolve to a `.gl` name \
             but whose linkage is not undefined-external must be filed under \
             `{NOT_EXTERN}`. Getting `{named_wrong_linkage}` means the \
             `DATA_SYM_LINKAGE` arm now publishes the other key. `data-sym-*` is \
             not a name problem here — it is `IL_CALL_IN_EXPR.md` §17.2 item 7's \
             whole-extra-section refusal, and the two rows size two different \
             follow-on rungs"
        );
        assert_ne!(
            unnamed, named_wrong_linkage,
            "M4: the two cells differ in exactly ONE fact — whether `.gl` names \
             token 0xF909 — so their keys must differ. Equal keys mean the census \
             has collapsed the two populations into one row, which is a ranking \
             that cannot be read even when neither key was renamed"
        );
        // **Absence is not success**: the count of cells that actually
        // discriminated is asserted, not assumed.
        let distinct: std::collections::BTreeSet<&str> =
            [unnamed.as_str(), named_wrong_linkage.as_str()].into_iter().collect();
        assert_eq!(
            distinct.len(),
            2,
            "M4: 2 discriminating cells graded, {} distinct keys — a guard that \
             graded fewer than 2 distinct keys proves nothing about a swap",
            distinct.len()
        );
    }

    /// **M3 — `resolve_data`'s `.gl` linkage gate is load-bearing, and the ONE
    /// byte that carries it is the one that moves the key.**
    ///
    /// Registered RED by `w-bind16`, measured **GREEN**: deleting
    /// `extern_data.contains(&name)` from `bind.rs:886` entirely failed nothing.
    /// That clause is what separates *"has no name"* from *"is defined in this
    /// TU"* — §17.2 item 7's whole-extra-section refusal — and dropping it makes
    /// the port willing to reference an address it would have to emit a section
    /// for.
    ///
    /// The cells are the **same bytes** but for the `.gl` linkage byte: `01`
    /// (defined here) must refuse, `02` (undefined external) must not. This is
    /// therefore a content check, not an existence check.
    #[test]
    fn the_data_symbol_linkage_gate_is_the_one_byte_that_moves_the_key() {
        let defined_here = key_of("B (linkage 01, defined here)", DYNINIT, gl_named(0x01));
        let undef_extern = key_of("C (linkage 02, undefined extern)", DYNINIT, gl_named(0x02));

        assert_eq!(
            defined_here, NOT_EXTERN,
            "M3: `.gl` linkage `01` is a symbol this TU DEFINES. `resolve_data` \
             must refuse it, so the census files the body under `{NOT_EXTERN}`. \
             Getting `{defined_here}` means the linkage gate no longer runs and \
             a defined global now reads as an address the port may reference \
             without emitting its section"
        );
        assert_ne!(
            undef_extern, NOT_EXTERN,
            "M3: the SAME bytes with linkage `02` — an undefined external — must \
             NOT be refused by `resolve_data`, or the gate is refusing its whole \
             input and the cell above is passing for the wrong reason. This is \
             the positive control: it fires on content, and one `.gl` byte is \
             all that separates the two cells"
        );
        assert_ne!(
            undef_extern, UNRESOLVED,
            "M3: linkage `02` with a name present must not read as UNRESOLVED \
             either — that would mean `resolve` itself stopped seeing the record \
             and the linkage gate was never reached, so the cell above would be \
             green on a broken `.gl` rather than on the gate"
        );
        // 2 cells, and the gate is the only difference between them.
        assert_eq!(
            gl_named(0x01).len(),
            gl_named(0x02).len(),
            "the two cells' `.gl` must be the same LENGTH — if they are not, \
             something other than the linkage byte differs and the guard is not \
             isolating the gate"
        );
        let diffs = gl_named(0x01)
            .iter()
            .zip(gl_named(0x02).iter())
            .filter(|(a, b)| a != b)
            .count();
        assert_eq!(
            diffs, 1,
            "the two cells' `.gl` must differ in exactly ONE byte (the linkage \
             byte); they differ in {diffs}. A guard whose two cells differ in \
             more than the thing under test does not locate the clause"
        );
    }

    /// **M1 — the call-argument ARITY fence, at rank 1 of board #3177's
    /// reachable widening order, 1,296 functions.**
    ///
    /// Registered RED by `w-bind16`, measured **GREEN**: `syms > 1` could be
    /// weakened to `syms > 2` — admitting every two-symbol call — and the whole
    /// suite still passed. A fence no test can see is a fence a later lane can
    /// silently weaken with no alarm.
    ///
    /// Graded as a **SERIES, not a cell** (#3147, which `w-slots` paid for: its
    /// fixture's obj read 3 and the series was `2n+1`). n = 1 is admitted, n = 2
    /// and n = 3 refuse. `syms > 2` breaks n = 2; `syms > 3` would break n = 3;
    /// deleting the fence breaks both.
    #[test]
    fn the_call_argument_arity_fence_is_a_series_and_admits_exactly_one_symbol() {
        // n = 1 — the second symbol argument replaced by a literal.
        let n1 = key_of(
            "n=1",
            &composed_lo(&swap_arg(DYNINIT, &SYM_ARG_FC09, &LIT_ARG)),
            gl_named(0x02),
        );
        // n = 2 — the transcript as captured.
        let n2 = key_of("n=2", &composed_lo(DYNINIT), gl_named(0x02));
        // n = 3 — the literal argument replaced by a third symbol.
        let sym_f709: Vec<u8> = {
            let mut v = SYM_ARG_FC09.to_vec();
            v[1] = 0xF7;
            v
        };
        let mut gl3 = gl_named(0x02);
        gl3.extend(data_rec([0xF7, 0x09], "?strC@@3HA", 0x02));
        let n3 = key_of("n=3", &composed_lo(&swap_arg(DYNINIT, &LIT_ARG, &sym_f709)), gl3);

        assert_eq!(
            n1, "multiarg-tail-call",
            "M1 positive control: ONE symbol argument is IN CLASS. Got `{n1}`. If \
             this is not in class the fence is refusing its whole input and the \
             two refusals below prove nothing — this is the cell that makes the \
             guard a discrimination rather than a constant"
        );
        assert_eq!(
            n2, "call-arg-multi-sym:eof",
            "M1: TWO symbol arguments must refuse on the arity fence \
             (`calls.rs` `syms > 1`). Got `{n2}`. This is the rank-1 row of \
             #3177's reachable order — 1,296 functions — and weakening the \
             threshold to `syms > 2` admits every two-symbol call silently"
        );
        assert_eq!(
            n3, "call-arg-multi-sym:eof",
            "M1: THREE symbol arguments must refuse on the same fence. Got \
             `{n3}`. n=3 is here so the guard is a SERIES: it catches a \
             threshold moved to `syms > 2` at n=2 and one moved to `syms > 3` at \
             n=3, where a single cell catches only one of them"
        );
    }

    /// **The `two_sym_thunk` exemption is the fence's ONE hole, and it is
    /// exactly one body-marker byte wide** (W-R1).
    ///
    /// `calls.rs` admits `syms == 2` when `body_start_is_bare` — the bare `4C`
    /// every `??__E`/`??__F` dynamic-initializer thunk carries. That exemption
    /// has the same property M1 has: nothing failed when it moved. Guarded as a
    /// pair, because the *whole* claim is that the bare marker and the composed
    /// one give different answers on otherwise identical bytes.
    #[test]
    fn the_two_symbol_thunk_exemption_turns_on_the_bare_body_marker_alone() {
        let bare = key_of("n=2, BARE `4C`", DYNINIT, gl_named(0x02));
        let composed = key_of("n=2, composed `4C 4F 11`", &composed_lo(DYNINIT), gl_named(0x02));
        assert_ne!(
            bare, "call-arg-multi-sym:eof",
            "W-R1: a two-symbol body behind the BARE `4C` body marker is the \
             `??__E`/`??__F` thunk class and is deliberately admitted past the \
             arity fence. Getting the arity refusal means `two_sym_thunk` no \
             longer fires and the dynamic-initializer class has silently left \
             the model"
        );
        assert_eq!(
            composed, "call-arg-multi-sym:eof",
            "W-R1: the SAME bytes with the composed `4C 4F 11` marker must take \
             the arity refusal. Got `{composed}`. If both markers give the same \
             answer the exemption is not conditioned on the marker at all"
        );
        assert_eq!(
            composed_lo(DYNINIT).len(),
            DYNINIT.len() + 2,
            "the two cells must differ by exactly the two bytes the composed \
             marker adds — anything else and the pair is not isolating the marker"
        );
    }

    /// **The producer's key and the machine-read consumer's key are the same
    /// string** — the loop #3199 says nothing closes.
    ///
    /// `scan.rs` writes `format!("emit-cflow-modeled-key|{}", f.verdict.key())`
    /// and [`GapReport::cflow_emitted_modeled_keys`] strips that prefix back
    /// off. Every test in this file that touches the reachable order feeds the
    /// consumer **hand-written** key strings, so no test in the repo has ever
    /// compared the two ends: a key the census renames still reads fine to the
    /// aggregator, and the ranking that dispatched a whole wave silently
    /// changes meaning.
    ///
    /// This drives real census keys — produced by `c2-il` from the transcript
    /// above — through the real concatenation and the real strip, and asserts
    /// round-trip identity plus the counts.
    #[test]
    fn the_census_key_survives_the_round_trip_into_the_reachable_ranking() {
        let keys = [
            key_of("A", DYNINIT, gl_callee_only()),
            key_of("B", DYNINIT, gl_named(0x01)),
            key_of("n=2 composed", &composed_lo(DYNINIT), gl_named(0x02)),
        ];
        // The producer's side, spelled exactly as `scan.rs:654` spells it.
        let emit: Vec<(String, usize)> = keys
            .iter()
            .map(|k| (format!("emit-cflow-modeled-key|{k}"), 1usize))
            .collect();
        let mut t = mk("modeled-key-roundtrip");
        for (k, n) in &emit {
            t.emit.insert(k.clone(), *n);
        }
        let rep = mk_report(vec![t]);
        let (rows, accounted) = rep.cflow_emitted_modeled_keys();
        assert_eq!(
            accounted, 3,
            "3 census rows went in; the consumer accounted {accounted}. A \
             producer key the consumer's prefix no longer matches vanishes from \
             the reachable ranking without the ranking getting shorter — it just \
             counts less, silently"
        );
        let got: std::collections::BTreeSet<&str> =
            rows.iter().map(|(k, _)| k.as_str()).collect();
        let want: std::collections::BTreeSet<&str> = keys.iter().map(String::as_str).collect();
        assert_eq!(
            got, want,
            "the strings `c2-il`'s census produced and the strings \
             `cflow_emitted_modeled_keys` published must be the SAME SET. \
             produced {want:?}, published {got:?}"
        );
        assert_eq!(
            want.len(),
            3,
            "3 discriminating cells, {} distinct keys — if the three cells \
             collapse to fewer keys this round trip is vacuous",
            want.len()
        );
        // …and the two keys #3177's head is counted from are both in it, by
        // name, having come out of the census rather than out of this file.
        assert!(
            got.contains(UNRESOLVED) && got.contains(NOT_EXTERN),
            "both keys of #3177's head must reach the ranking under their own \
             names: {got:?}"
        );
    }

    /// **W-CALLEEGUARD — one witness per RAISE SITE for the whole
    /// `callee-unresolved` key family.**
    ///
    /// `w-mutcensus` (`docs/rungs/2026-08-17-mutcensus.md` §3, §4.3) measured all
    /// four sites of this family GREEN: **nothing in the 1,660-test suite could
    /// fail on any of them**, including the **default** arm that routes
    /// `callee-unresolved-tail-call` — board **#3209**'s key over **1,296**
    /// function bodies on the 878-TU workload, the single most populous refusal
    /// key there. It could be exchanged for a sibling with the suite staying
    /// green.
    ///
    /// The four sites are the arms of one `match label` in
    /// `c2-il/src/func/census.rs` (`:1308`, `:1310`, `:1313`, `:1315` at
    /// `44794fa4` — **re-located at this base**, not inherited from the census's
    /// `3835469c`, because two peers landed in `c2-il` in between). They fire
    /// when the body **parsed** but `shape_to_function` returned `None` with no
    /// data-symbol `sym_fail` pending — i.e. the call's callee token has no `.gl`
    /// name.
    ///
    /// ## Why this is one witness per SITE and not one per KEY
    ///
    /// `w-mutcensus` F2 states the mechanism behind its whole GREEN population:
    /// *"guard tests are per-KEY witness tests, so a key with k raise sites
    /// contributes k − 1 unguarded sites by construction"*. **That mechanism does
    /// not apply to this family**: `grep -rn CALLEE_UNRESOLVED_ crates/` finds
    /// **exactly one raise site per key**, so k = 1 four times over and a per-key
    /// witness *is* a per-site witness here. These four were unguarded because
    /// nobody wrote a witness, not by construction.
    ///
    /// What generalizes is the **form**, and it is deliberately a table: one row
    /// per raise site, keyed on the input that reaches *that* site. A table of
    /// site-witnesses catches a key swap at any site even when several sites
    /// share one key — which is the shape F2 says no per-key suite can reach.
    ///
    /// ## The cells
    ///
    /// Every assertion is on the **key string** (`FnVerdict::key()`), never on a
    /// constant — `2026-08-16-guards.md` §2's reason unchanged: a guard on the
    /// constant passes a mutation that renames the constant *and* its uses while
    /// the published key moves.
    ///
    /// | cell | transcript | in-class label | family key it must take |
    /// |---|---|---|---|
    /// | **F** | `int f(int a){ return g(a)+1; }` | `framed-call` | `callee-unresolved-framed-call:eof` |
    /// | **Q** | `void f(int a){ g1(a); g2(); }` | `call-sequence` | `callee-unresolved-call-sequence:eof` |
    /// | **E** | `Der::~Der() {}` | `empty-dtor-delegation` | `callee-unresolved-dtor-delegation:eof` |
    /// | **V** | `void f(){ g(); }` | `void-tail-call` | `callee-unresolved-tail-call:eof` (**default** arm) |
    /// | **M** | [`DYNINIT`] + `gl_named(0x02)` | `multiarg-tail-call` | `callee-unresolved-tail-call:eof` (**default** arm, second label) |
    mod callee_unresolved_arms {
        use super::*;

        /// The `4F 1F` segment header, byte-for-byte the one [`DYNINIT`] carries
        /// — same capture, same toolchain, same flags. The four body transcripts
        /// below begin at the `53 53 26 <fn>` statement start and need it: a body
        /// without the header still yields a census row, but it keys
        /// `formals-marker:mid` (measured), which is a *parse* refusal upstream of
        /// the arms this module exists to pin.
        const HDR: &[u8] = &[
            0x4F, 0x1F, 0x80, 0x05, 0x00, 0x20, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00,
            0x4F, 0x33, 0x0D, 0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01,
            0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18, 0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38,
            0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D, 0x08, 0x00, 0x0F,
            0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x03,
        ];

        fn wrap(body: &[u8]) -> Vec<u8> {
            let mut v = HDR.to_vec();
            v.extend_from_slice(body);
            v
        }

        // The four transcripts are **transcribed** from `c2-il`'s own
        // `func::test_fixtures` (`MVP_FRAMED`, `SEQ_TWO_VOID`, `DTOR_DELEGATE`,
        // `MVP_CALL`) rather than referenced, for the reason [`DYNINIT`] is: that
        // module is `#[cfg(test)]` inside a crate this lane does not own and is
        // unreachable across the crate boundary. **They are copies of captures,
        // not second sources of truth** — if `c2-il`'s readers change under them
        // these guards break, and that is the guard working. The response is to
        // re-derive from the capture, never to delete the test.

        /// `int f(int a){ return g(a) + 1; }` — the framed call, callee `0xE409`.
        const FRAMED: &[u8] = &[
            0x53, 0x53, 0x26, 0xE6, 0x09, 0x46, 0x2D, 0xE5, 0x09,
            0x4C, 0x4F, 0x11, 0x53, 0x26, 0xE4, 0x09, 0xBD, 0x86, 0x41, 0x74, 0x00, 0x80, 0x01,
            0x10, 0x00, 0x00, 0xB9, 0xE5, 0x09, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74, 0x4C,
            0x33, 0x86, 0x41, 0x74, 0x01, 0x02, 0x41, 0x86, 0x41, 0x74, 0x3A, 0xE7, 0x09, 0x54,
            0x02, 0x29, 0xE7, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20,
            0x00, 0x4F, 0x01, 0x08, 0x4D,
        ];
        /// `void f(int a){ g1(a); g2(); }` — two statement calls, `0xE409` first.
        const SEQ: &[u8] = &[
            0x53, 0x53, 0x26, 0xE7, 0x09, 0x46, 0x2D, 0xE6, 0x09, 0x4C, 0x4F, 0x11, 0x53,
            0x26, 0xE4, 0x09, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00,
            0xB9, 0xE6, 0x09, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74, 0x4C, 0x4B,
            0x26, 0xE5, 0x09, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x03, 0x10, 0x00, 0x00,
            0x4C, 0x4B, 0x3A, 0xE8, 0x09, 0x54, 0x02, 0x29, 0xE8, 0x09,
            0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x03,
            0x4D,
        ];
        /// `Der::~Der() {}` at the workload's own flags — the empty destructor
        /// that delegates to `??1Base`, `0xE409`.
        const DTOR: &[u8] = &[
            0x53, 0x53, 0x26, 0xF0, 0x09, 0xB9, 0xFC, 0x09, 0xA6, 0x43, 0x81, 0x20,
            0x99, 0x86, 0x43, 0x8A, 0x20, 0x00, 0x46, 0x4C, 0x4F, 0x11, 0x53,
            0x33, 0x86, 0x41, 0x74, 0x00, 0x26, 0xE4, 0x09,
            0x33, 0x86, 0x41, 0x74, 0x80, 0x41, 0x08, 0x00, 0x00,
            0x40, 0x86, 0x43, 0x8E, 0x20, 0x66, 0x02, 0x80, 0x20, 0x82, 0x20,
            0x55, 0x86, 0x41, 0x74, 0x33, 0x86, 0x41, 0x74, 0x00, 0x55, 0x86, 0x41, 0x74,
            0xB9, 0xFC, 0x09, 0xA6, 0x43, 0x81, 0x20, 0x55, 0xA6, 0x43, 0x81, 0x20, 0x4C,
            0x2C, 0xA6, 0x43, 0x84, 0x20, 0x00, 0x99, 0x86, 0x43, 0x85, 0x20, 0x00,
            0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x05, 0x10, 0x00, 0x00, 0x4C,
            0x5C, 0x86, 0x41, 0x74, 0x01, 0x4B,
            0x3A, 0xFD, 0x09, 0x54, 0x02, 0x29, 0xFD, 0x09, 0x5E, 0x01, 0x21, 0x4B,
            0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
        ];
        /// `void f(){ g(); }` — the bare void tail call, callee `0xE309`.
        const VOIDCALL: &[u8] = &[
            0x53, 0x53, 0x26, 0xE4, 0x09, 0x46, 0x4C, 0x4F, 0x11, 0x53,
            0x26, 0xE3, 0x09, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00,
            0x4C, 0x4B, 0x3A, 0xE5, 0x09, 0x54, 0x02, 0x29, 0xE5, 0x09,
            0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x07,
            0x4D,
        ];

        const K_FRAMED: &str = "callee-unresolved-framed-call:eof";
        const K_SEQ: &str = "callee-unresolved-call-sequence:eof";
        const K_DTOR: &str = "callee-unresolved-dtor-delegation:eof";
        const K_TAIL: &str = "callee-unresolved-tail-call:eof";

        /// One row per **raise site**: the transcript that reaches it, a `.gl`
        /// that names every callee it pushes, the index of the ONE byte that
        /// un-names the first callee, the in-class label the resolved cell
        /// reports, and the family key the unresolved cell must take.
        struct Arm {
            cell: &'static str,
            site: &'static str,
            seg: &'static [u8],
            gl_resolved: fn() -> Vec<u8>,
            /// Index into `gl_resolved()` of the callee token's high byte.
            token_byte: usize,
            in_class: &'static str,
            family_key: &'static str,
        }

        fn gl_framed() -> Vec<u8> {
            sym_rec([0xE4, 0x09], "?g@@YAHH@Z")
        }
        fn gl_seq() -> Vec<u8> {
            let mut g = sym_rec([0xE4, 0x09], "?g1@@YAXH@Z");
            g.extend(sym_rec([0xE5, 0x09], "?g2@@YAXXZ"));
            g
        }
        fn gl_dtor() -> Vec<u8> {
            sym_rec([0xE4, 0x09], "??1Base@@QAA@XZ")
        }
        fn gl_voidcall() -> Vec<u8> {
            sym_rec([0xE3, 0x09], "?g0@@YAXXZ")
        }

        /// The four sites, in the source order of the arms they pin.
        const ARMS: &[Arm] = &[
            Arm { cell: "F", site: "census.rs:1308 `\"framed-call\" =>`",
                  seg: FRAMED, gl_resolved: gl_framed, token_byte: 1,
                  in_class: "framed-call", family_key: K_FRAMED },
            Arm { cell: "Q", site: "census.rs:1310 `l if l.starts_with(\"call-sequence\") =>`",
                  seg: SEQ, gl_resolved: gl_seq, token_byte: 1,
                  in_class: "call-sequence", family_key: K_SEQ },
            Arm { cell: "E", site: "census.rs:1313 `l if l.starts_with(\"empty-dtor\") =>`",
                  seg: DTOR, gl_resolved: gl_dtor, token_byte: 1,
                  in_class: "empty-dtor-delegation", family_key: K_DTOR },
            Arm { cell: "V", site: "census.rs:1315 `_ =>` (the DEFAULT arm)",
                  seg: VOIDCALL, gl_resolved: gl_voidcall, token_byte: 1,
                  in_class: "void-tail-call", family_key: K_TAIL },
        ];

        /// The `.gl` of `arm`, with the first callee's token changed in **exactly
        /// one byte**, so a record of the same length still exists and binds a
        /// token the body never pushes.
        fn gl_unresolved(arm: &Arm) -> Vec<u8> {
            let mut g = (arm.gl_resolved)();
            g[arm.token_byte] ^= 0x05;
            g
        }

        /// **The site-witness table: every raise site of the family, by the input
        /// that reaches it, asserted on the published key string.**
        ///
        /// This is the test the census's `CS5`–`CS8` mutations have to fail. Each
        /// row swaps *one* arm's key; each row of this table pins *one* arm; so a
        /// swap anywhere in the `match` fails here by construction rather than
        /// incidentally.
        #[test]
        fn every_raise_site_of_the_callee_unresolved_family_has_its_own_witness() {
            let mut seen: Vec<String> = Vec::new();
            for arm in ARMS {
                let got = key_of(
                    &format!("{} (unresolved callee)", arm.cell),
                    &wrap(arm.seg),
                    gl_unresolved(arm),
                );
                assert_eq!(
                    got, arm.family_key,
                    "cell {}: the body parses as `{}` and its callee has no `.gl` \
                     name, so the census must file it under `{}` — the key raised \
                     at {}. Got `{got}`. A different key here means that arm now \
                     publishes a sibling's key, which is exactly the swap \
                     `w-mutcensus` measured GREEN on all four of these sites",
                    arm.cell, arm.in_class, arm.family_key, arm.site
                );
                seen.push(got);
            }
            assert_eq!(
                seen.len(),
                4,
                "4 raise sites enumerated, {} witnessed — a table that shrank \
                 stops guarding the sites it dropped",
                seen.len()
            );
            let distinct: std::collections::BTreeSet<&str> =
                seen.iter().map(String::as_str).collect();
            assert_eq!(
                distinct.len(),
                4,
                "the four arms must publish four DISTINCT keys; got {} — \
                 {distinct:?}. A collapse makes every equality above satisfiable \
                 by one key and the table vacuous",
                distinct.len()
            );
        }

        /// **Each cell is a discrimination the callee's name alone moves, and the
        /// two `.gl` differ in exactly ONE byte.**
        ///
        /// `w-guards`' standard: state the minimal difference between adjacent
        /// cells and *assert* it rather than trusting it. Without the resolved
        /// half, a refusal is consistent with the arm refusing its whole input and
        /// the table above would be a constant rather than a discrimination.
        #[test]
        fn each_callee_unresolved_cell_is_moved_by_one_gl_byte_and_nothing_else() {
            for arm in ARMS {
                let resolved = (arm.gl_resolved)();
                let unresolved = gl_unresolved(arm);
                assert_eq!(
                    resolved.len(),
                    unresolved.len(),
                    "cell {}: the two `.gl` must be the same LENGTH or something \
                     other than the callee's token differs",
                    arm.cell
                );
                let diffs = resolved
                    .iter()
                    .zip(unresolved.iter())
                    .filter(|(a, b)| a != b)
                    .count();
                assert_eq!(
                    diffs, 1,
                    "cell {}: the two `.gl` must differ in exactly ONE byte (the \
                     callee token's high byte); they differ in {diffs}. A pair \
                     that varies more than the thing under test does not locate \
                     the clause",
                    arm.cell
                );

                let in_class = key_of(
                    &format!("{} (callee NAMED)", arm.cell),
                    &wrap(arm.seg),
                    resolved,
                );
                assert_eq!(
                    in_class, arm.in_class,
                    "cell {} positive control: with the callee named, the SAME \
                     bytes must report the in-class shape label `{}`. Got \
                     `{in_class}`. If this is not in class the cell is refusing \
                     its whole input and its refusal above proves nothing about \
                     {}",
                    arm.cell, arm.in_class, arm.site
                );
                let refused = key_of(
                    &format!("{} (callee UNRESOLVED)", arm.cell),
                    &wrap(arm.seg),
                    gl_unresolved(arm),
                );
                assert_eq!(
                    refused, arm.family_key,
                    "cell {}: one `.gl` byte later, the same bytes must key \
                     `{}`. Got `{refused}`",
                    arm.cell, arm.family_key
                );
            }
        }

        /// **The default arm is a CATCH-ALL, and one witness cannot say so.**
        ///
        /// `census.rs:1315` is `_ =>`. A single cell reaching
        /// `callee-unresolved-tail-call` is equally consistent with the arm having
        /// been rewritten to match that cell's own label. Two cells whose
        /// **in-class labels differ** — `void-tail-call` and `multiarg-tail-call`
        /// — reaching one key is the statement that the arm is the fallthrough.
        ///
        /// This is the site `w-mutcensus` §4.3 singles out: board **#3209**'s
        /// `callee-unresolved-tail-call` over **1,296** bodies, *"the single most
        /// populous refusal key on the 878-TU workload"*, swappable with the
        /// entire suite staying green.
        #[test]
        fn the_default_arm_is_the_catch_all_reached_by_more_than_one_label() {
            let void_arm = &ARMS[3];
            let by_void = key_of("V (void tail call)", &wrap(VOIDCALL), gl_unresolved(void_arm));
            let by_multiarg = key_of("M (`w-guards`' cell C)", DYNINIT, gl_named(0x02));

            assert_eq!(
                by_void, K_TAIL,
                "the void-tail-call label falls through to `census.rs:1315`'s \
                 `_ =>` and must key `{K_TAIL}`. Got `{by_void}`"
            );
            assert_eq!(
                by_multiarg, K_TAIL,
                "and so must a SECOND, differently-labelled body — `w-guards`' \
                 cell C, a `multiarg-tail-call`. Got `{by_multiarg}`. Two labels \
                 reaching one key is what makes `_ =>` a catch-all rather than an \
                 arm keyed on one label"
            );

            // …and the two really are different labels, checked from the census
            // itself rather than asserted in prose: naming each cell's callee
            // moves it to its own in-class label.
            let void_label = key_of("V (callee NAMED)", &wrap(VOIDCALL), gl_voidcall());
            assert_eq!(void_label, "void-tail-call");
            assert_ne!(
                void_label, "multiarg-tail-call",
                "the two default-arm witnesses must carry DIFFERENT labels, or \
                 they are one witness written twice"
            );
        }
    }

}

// ---------------------------------------------------------------------------
// THE OBJECT PLAN (lane `w-objplan`) — the round-trip and the second derivation
// ---------------------------------------------------------------------------

use super::plan as objplan;

/// Build a report whose TUs exercise every verdict at least once, so no check
/// below can pass by the population being uniform.
fn mk_plan_report() -> GapReport {
    let mut exact = mk("exact.cpp");
    exact.src = "src/exact.cpp".into();
    exact.class = TuClass::Match;
    exact.plan.observable = true;
    exact.plan.verdicts.insert("emitset-members".into(), objplan::PlanVerdict::Exact);
    exact.plan.verdicts.insert("emitset-order".into(), objplan::PlanVerdict::Exact);
    exact.plan.emitset_subset = Some(true);
    exact.plan.emitset_extra = Some(0);
    exact.plan.emitset_missing = Some(0);
    exact.plan.sigs.insert("emitset-members".into(), "1:?a@@YAXXZ".into());

    let mut differs = mk("differs.cpp");
    differs.src = "src/differs.cpp".into();
    differs.class = TuClass::VocabGap;
    differs.plan.observable = true;
    differs.plan.verdicts.insert("emitset-members".into(), objplan::PlanVerdict::Differs);
    differs.plan.verdicts.insert("emitset-order".into(), objplan::PlanVerdict::Differs);
    differs.plan.emitset_subset = Some(true);
    differs.plan.emitset_extra = Some(0);
    differs.plan.emitset_missing = Some(4);
    differs.plan.sigs.insert("emitset-members".into(), "5:?b@@YAXXZ".into());

    let mut unknown = mk("unknown.cpp");
    unknown.src = "src/unknown.cpp".into();
    unknown.class = TuClass::VocabGap;
    unknown.plan.observable = true;
    unknown.plan.verdicts.insert("emitset-members".into(), objplan::PlanVerdict::Unknown);
    unknown.plan.verdicts.insert("emitset-order".into(), objplan::PlanVerdict::Unknown);
    unknown.plan.reasons.insert("emitset-members".into(), "gl-attrs-refused".into());
    unknown.plan.reasons.insert("emitset-order".into(), "gl-attrs-refused".into());
    unknown.plan.sigs.insert("emitset-members".into(), "5:?b@@YAXXZ".into());

    let mut unobs = mk("unobs.cpp");
    unobs.src = "src/unobs.cpp".into();
    unobs.class = TuClass::VocabGap;
    unobs.plan.observable = false;
    unobs.plan.verdicts.insert("emitset-members".into(), objplan::PlanVerdict::Unobservable);
    unobs.plan.verdicts.insert("emitset-order".into(), objplan::PlanVerdict::Unobservable);

    let mut gone = mk("gone.cpp");
    gone.src = "src/gone.cpp".into();
    gone.class = TuClass::CaptureFail;

    mk_report(vec![exact, differs, unknown, unobs, gone])
}

/// **The offline `--plan-tsv` view and the live report are the SAME rows.**
///
/// #3288 in its concrete form: the `plan-*` counts come from `metrics()` over
/// the live results, and the second derivation parses the file. If the two
/// producers can drift, the "second derivation" is a derivation of something
/// else and catches nothing.
#[test]
fn the_plan_tsv_view_and_the_live_report_are_the_same_rows() {
    let rep = mk_plan_report();
    let live = rep.plan_rows();
    let parsed =
        objplan::parse_plan_tsv(&objplan::plan_tsv(&live)).expect("the writer's own output");
    assert_eq!(live, parsed, "one definition, two producers — row for row");
    assert_eq!(live.len(), 4, "4 graded TUs; `gone.cpp` captured nothing and is not a row");
    assert!(
        !live.iter().any(|r| r.src == "src/gone.cpp"),
        "a capture-fail TU is ABSENT, never an `unobservable` row: it was never \
         measured, which is a different fact (docs/STATUS.md trap 5)"
    );
}

/// **Every `plan-*` count the scan publishes is re-derived from the rows, a
/// second and differently-built way, and the two agree** (#3288 — this check has
/// caught a wrong figure in every lane that has run it).
#[test]
fn every_plan_metric_is_re_derived_from_the_rows_and_agrees() {
    let rep = mk_plan_report();
    let published: BTreeMap<String, usize> = rep
        .metrics()
        .into_iter()
        .filter_map(|(k, v)| v.parse::<usize>().ok().map(|n| (k.to_string(), n)))
        .collect();
    let derived = objplan::derive_metrics(&rep.plan_rows());
    let mut checked = 0usize;
    for (k, d) in &derived {
        let p = published.get(k).copied();
        assert_eq!(
            p,
            Some(*d),
            "`{k}`: the scan publishes {p:?} and the rows re-derive {d}. A published \
             figure and its own listing must not be able to disagree."
        );
        checked += 1;
    }
    assert!(
        checked >= 10,
        "the second derivation must actually check something — a control that \
         checks 0 keys and one that passes look identical ({checked} checked)"
    );
}

/// **The four verdicts are a PARTITION of the graded TUs, per component.**
///
/// Asserted rather than reported, `sweep_shapes.py --check`'s rule: a partition
/// that is merely printed is a partition nobody notices breaking.
#[test]
fn the_verdicts_partition_the_graded_tus_for_every_component() {
    let rep = mk_plan_report();
    let rows = rep.plan_rows();
    for (i, c) in objplan::PLAN_COMPONENTS.iter().enumerate() {
        let mut n = [0usize; 4];
        for r in &rows {
            n[match r.verdicts[i] {
                objplan::PlanVerdict::Exact => 0,
                objplan::PlanVerdict::Differs => 1,
                objplan::PlanVerdict::Unknown => 2,
                objplan::PlanVerdict::Unobservable => 3,
            }] += 1;
        }
        assert_eq!(
            n.iter().sum::<usize>(),
            rows.len(),
            "component {c}: the four verdicts must sum to the graded population"
        );
        assert!(
            n.iter().all(|k| *k > 0),
            "component {c}: this fixture must exercise all four verdicts, or the \
             partition check passes on a uniform population — {n:?}"
        );
    }
}

/// `distinct` counts DISTINCT OBSERVED values, so a component whose reference
/// side is constant across the workload reads 1 and is labelled free. Two of
/// the fixture's TUs deliberately share a signature.
#[test]
fn distinct_counts_observed_values_and_not_tus() {
    let rep = mk_plan_report();
    let d = rep.plan_distinct();
    assert_eq!(
        d.get("emitset-members").copied(),
        Some(2),
        "three TUs carry a signature and two of them are identical, so the \
         distinct count is 2 and not 3"
    );
}

/// **The containment control reads 0 on a well-formed report**, and it is a
/// count rather than a status — `plan-bounds-violations` is published either
/// way, so a run that checked nothing is distinguishable from one that passed.
#[test]
fn the_plan_containment_control_reads_zero_and_is_published() {
    let rep = mk_plan_report();
    let m: BTreeMap<&str, String> = rep.metrics().into_iter().collect();
    assert_eq!(
        m.get("plan-bounds-violations").map(String::as_str),
        Some("0"),
        "the containment invariants of the instrument itself must hold"
    );
    // …and the key is PRESENT. A renamed key returns NO-RESULT from
    // `scripts/status.sh`'s collector, which is trap 5 with the mask on.
    assert!(m.contains_key("plan-observable"));
    assert!(m.contains_key("plan-emitset-members-exact"));
    assert!(m.contains_key("plan-control-diff"));
}

/// **`differs` is DERIVED, never read** (board #213): `known − exact`, computed
/// in `metrics()`, so it cannot be assembled from two independently-stale
/// halves.
#[test]
fn plan_differs_is_derived_from_known_minus_exact() {
    let rep = mk_plan_report();
    let m: BTreeMap<&str, String> = rep.metrics().into_iter().collect();
    let g = |k: &str| m[k].parse::<usize>().unwrap();
    for c in objplan::PLAN_COMPONENTS {
        let known = g(&format!("plan-{c}-known"));
        let exact = g(&format!("plan-{c}-exact"));
        let differs = g(&format!("plan-{c}-differs"));
        assert_eq!(differs, known - exact, "component {c}");
        assert!(
            exact <= known && known <= g(&format!("plan-{c}-observable")),
            "component {c}: exact ⊆ known ⊆ observable"
        );
    }
}

/// **The named control is a real constraint and its shortfalls are named.** A
/// pinned TU that is not `exact` on every shipped component must appear in the
/// shortfall list BY NAME with the component named — a count alone cannot be
/// acted on.
#[test]
fn the_named_control_reports_its_shortfall_by_name_and_component() {
    let mut r = mk("ctl");
    // Use a real pinned name so the control actually selects this row.
    r.src = objplan::control_tus()
        .into_iter()
        .next()
        .expect("the pin is non-empty")
        .to_string();
    r.class = TuClass::Match;
    r.plan.observable = true;
    r.plan.verdicts.insert("emitset-members".into(), objplan::PlanVerdict::Exact);
    r.plan.verdicts.insert("emitset-order".into(), objplan::PlanVerdict::Differs);
    r.plan.emitset_subset = Some(true);
    let src = r.src.clone();
    let rep = mk_report(vec![r]);
    let ctl = rep.plan_control();
    assert_eq!(ctl.exact_rows, 0, "one component differs, so the TU is not exact");
    assert_eq!(ctl.shortfall.len(), 1);
    assert_eq!(ctl.shortfall[0].0, src);
    assert_eq!(ctl.shortfall[0].1, "emitset-order");
    assert_eq!(ctl.shortfall[0].2, objplan::PlanVerdict::Differs);
    // …and the identity diff reports the 25 pinned TUs this one-row report does
    // not carry, in the `left` direction. A control that quietly ignored them
    // would pass on a `--limit 1` scan.
    assert_eq!(ctl.diff_left.len(), ctl.pinned - 1);
    assert!(ctl.diff_entered.is_empty());
}
