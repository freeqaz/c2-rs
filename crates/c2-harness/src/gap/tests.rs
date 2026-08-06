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
        fn_eh: BTreeMap::new(),
        fn_dispatch: BTreeMap::new(),
        fn_complete: BTreeMap::new(),
        fn_prod: BTreeMap::new(),
        fn_gate_refusals: BTreeMap::new(),
        bind_checks: BTreeMap::new(),
        emit: BTreeMap::new(),
        emit_blockers: BTreeMap::new(),
        emit_witness: Vec::new(),
        fndiff: Vec::new(),
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
