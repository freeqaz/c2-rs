//! The scan's own printed report of the factor model. Split out of `gap.rs`
//! unchanged; see [`super`] for the module docs.

use super::{GapReport, TuClass, PORT_WRITER_SECTIONS, WHOLE_TU_RECOGNIZERS};

/// **The Phase 7 factorization, printed on every scan** (`docs/ROADMAP.md`
/// §10.19 and §10.21, boards #160 and #179).
///
/// Printed from here rather than from `main.rs` for the same reason the splitter
/// block above is: the predicates live in this file, and a report assembled
/// somewhere else is a second place the definitions can drift.
///
/// **Everything here is a count.** There is no "factorization OK" line: a joint
/// that reproduced nothing would print zeros against a nonzero match count,
/// which is visible, where a status would not be (`docs/GAPS.md` §7).
pub(super) fn print_factorization(report: &GapReport) {
    let graded = report.graded().count();
    let [a, b, c, d, e, a_lo, bc, abc, abcd, joint] = report.factor_counts();
    let (bad, match_tus) = report.factor_control_on_match_tus();
    let matched: Vec<&str> = report
        .results
        .iter()
        .filter(|r| r.class == TuClass::Match)
        .map(|r| r.src.as_str())
        .collect();
    let all = report.factor_all_tus();
    let abcd_tus = report.factor_abcd_tus();
    let vocab = report.section_vocabulary();
    let unreadable = report.emit_total("emit-sec-unreadable");
    let readable = report.emit_total("emit-sec-readable");

    println!(
        "\nPHASE 7 FACTORS (ROADMAP §10.19/§10.21, boards #160/#179) — the model is \
         A and B and C and (D or E), over {graded} graded TUs\n\
         \x20 A  emit set reachable   `.ex` segments == obj `.text` COMDATs   {a:>5}  \
         (gate-anchored `4F 1F`; {a_lo} on the census's `4C 4F 11` anchor)\n\
         \x20 B  binding complete     every emitted symbol binds              {b:>5}\n\
         \x20 C  section shape        obj sections subset of the writer's {}   {c:>5}\n\
         \x20 D  per-fn acceptance    every emitted COMDAT in class           {d:>5}  \
         ({} of them emit nothing at all)\n\
         \x20 E  whole-TU acceptance  a REGISTERED whole-TU recognizer takes it {e:>3}  \
         ({} in the registry)\n\
         \x20   D and E are the two readings of ONE question — does the port have an accepted \
         route to this TU's contents. Neither is necessary alone (measured: D fails on the \
         whole-TU matches, E on the per-function ones); the DISJUNCTION is the term.\n\
         \x20 B and C jointly (the near-term ceiling, measured per TU — NOT a product of \
         marginals, §8.6): {bc}\n\
         \x20 A and B and C: {abc}   |   A and B and C and D (§10.19's original, refuted): \
         {abcd}   |   A and B and C and (D or E): {joint}   |   TUs the differential graded \
         `match`: {}\n\
         \x20 section headers: {readable} objs read, {unreadable} did not decode (outside C, \
         fail-closed)",
        PORT_WRITER_SECTIONS.len(),
        report.emit_total("emit-class-empty"),
        WHOLE_TU_RECOGNIZERS.len(),
        matched.len(),
    );
    // Per-recognizer marginals. A registry entry that never fires and one that
    // was never added are the same `|E|` and very different facts.
    for (name, n) in report.whole_tu_marginals() {
        println!("\x20   whole-TU recognizer `{name}` accepts {n} graded TU(s)");
    }
    // The refuted conjunction's own set, printed only when it differs from the
    // model's. This is the DELTA the fifth term accounts for, by name — §10.19's
    // claim is refuted by exactly these TUs and a reader should not have to
    // reconstruct which.
    if abcd_tus != all {
        let only_new: Vec<&&str> = all.iter().filter(|s| !abcd_tus.contains(s)).collect();
        println!(
            "\x20 the fifth term accounts for {} TU(s) that A and B and C and D misses: {:?}",
            only_new.len(),
            only_new
        );
    }
    // The set identity, by name. §10.19's claim is that the joint IS the match
    // set; two sets of the same size that differ by a swap would read as equal
    // if this printed only counts.
    if all == matched {
        println!(
            "\x20 the joint is EXACTLY the match set ({} TUs, by name): {}",
            all.len(),
            all.join(", ")
        );
    } else {
        let only_joint: Vec<&&str> = all.iter().filter(|s| !matched.contains(s)).collect();
        let only_match: Vec<&&str> = matched.iter().filter(|s| !all.contains(s)).collect();
        println!(
            "\x20 the joint is NOT the match set — {} in the joint only ({:?}), {} matching but \
             outside some factor ({:?}). The second list is the ALARM: every factor is meant \
             to be NECESSARY, so a matching TU outside one voids the bound it carries.",
            only_joint.len(),
            only_joint,
            only_match.len(),
            only_match,
        );
    }
    // **The known-answer control.** Split into the terms that must be zero and
    // the two diagnostics that must not be required to be — see
    // `factor_control_on_match_tus` for why moving D out of the first group is a
    // repair of the model and not a relaxation of the control.
    println!(
        "\x20 known-answer control — matching TUs failing each NECESSARY term (all must be 0, \
         over {match_tus} matching TUs): A {} B {} C {} D-or-E {}\n\
         \x20   diagnostics, NOT required to be 0 (neither disjunct is necessary alone): \
         D {} E {}",
        bad[0], bad[1], bad[2], bad[5], bad[3], bad[4]
    );
    // **W-R1c measured that D alone is not necessary; board #179 says what the
    // necessary term is instead.** §10.19's claim was that A∧B∧C∧D is exactly the
    // observed match set. That was measured when `PortC2` had exactly ONE
    // acceptance path, the per-function one, and D's proxy for "the port can emit
    // this" is the per-function census verdict. A `??__E` dynamic-initializer TU
    // is emitted by a **whole-TU** path (`IlBundle::dyninit_tu`), so its thunk is
    // byte-exact in the obj and out of class in the census at the same time —
    // both statements true, about different questions.
    //
    // The fix is a fifth term, E, disjoined onto D. It is NOT a widening of D:
    // D still reads exactly the per-function census, so the census/gate symmetry
    // `census.rs` maintains on purpose (tracked by the `census/gate disagreement`
    // line above) is untouched. And D's own violation count is still printed,
    // above, because a refutation whose evidence stops being reported is a claim
    // nobody can re-check.
    if bad[3] > 0 {
        println!(
            "\x20   D {} — the per-function census does not model the whole-TU emit path, so \
             §10.19's A∧B∧C∧D conjunction stays refuted by these TU(s). That is the finding \
             board #179 repairs with E, not a defect E hides.",
            bad[3]
        );
    }
    if bad[5] > 0 {
        println!(
            "\x20   ALARM: {} matching TU(s) are outside BOTH D and E. The port emitted \
             byte-exact bytes through a path no entry in WHOLE_TU_RECOGNIZERS models — \
             register it there, or the factorization's bound is void. This is the control \
             doing its job, not a number to tune away.",
            bad[5]
        );
    }
    // The vocabulary, in full. It is finite, and its size is the headline: it is
    // what makes C the one factor with a short route to closure.
    println!(
        "\x20 SECTION VOCABULARY — {} distinct names across the workload (objs carrying each):",
        vocab.len()
    );
    for (name, objs) in &vocab {
        let mine = if PORT_WRITER_SECTIONS.contains(&name.as_str()) {
            "writer"
        } else {
            "  ---"
        };
        println!("\x20   {objs:>5} objs  [{mine}]  {name}");
    }
    // The one ACTIONABLE row of the whole block: TUs no acceptance path covers.
    // Printed as a list with each one's distance, because a count would say "17"
    // and name nothing to work on.
    //
    // **Board #179 narrowed the membership from `¬D` to `¬(D∨E)`.** A TU a
    // whole-TU recognizer already accepts is not reachable by widening the
    // per-function class, so listing it here would advertise the wrong work.
    let frontier = report.factor_frontier();
    println!(
        "\x20 FRONTIER — {} graded TUs satisfy A and B and C, are NOT a match, and are outside \
         BOTH acceptance paths (not D, not E), so per-function codegen breadth is the whole \
         remaining distance (blocked emitted | emitted | src):",
        frontier.len()
    );
    for (r, blocked) in frontier.iter().take(40) {
        println!(
            "\x20   {blocked:>4} | {:>4} | {}",
            r.emit.get("emit-emitted").copied().unwrap_or(0),
            r.src
        );
    }
    if frontier.len() > 40 {
        println!("\x20   … and {} more", frontier.len() - 40);
    }
    // **What a perfect emit predicate is worth, stated as both of the
    // quantities board #213 conflated.** #213 published `+82` for both because
    // they coincided on that corpus; they are different questions and the
    // difference is printed by name whenever it is nonempty.
    let frontier_if_a = report.factor_frontier_if_a();
    let div = report.factor_projection_divergence();
    println!(
        "\x20 A PERFECT EMIT PREDICATE (board #213) — reachability, NOT conversions: every TU \
         below is still gated on codegen that does not exist.\n\
         \x20   as REACH:    B and C {bc} less A and B and C {abc} = +{}\n\
         \x20   as FRONTIER: frontier-if-A {frontier_if_a} less FRONTIER {} = +{}",
        bc.saturating_sub(abc),
        frontier.len(),
        frontier_if_a.saturating_sub(frontier.len()),
    );
    if div.is_empty() {
        println!(
            "\x20   the two agree ({}). They are still different questions — they coincide \
             only while no TU inside B and C fails A with an acceptance path already \
             covering it, which is a fact about this corpus.",
            bc.saturating_sub(abc)
        );
    } else {
        println!(
            "\x20   THE TWO DISAGREE by {} — #213's single `+82` is refuted on this tree. The \
             difference is exactly the TU(s) inside B and C that fail A and that the port \
             ALREADY accepts (D or E), so modelling the emit set makes them reachable \
             without adding them to the codegen frontier: {:?}",
            div.len(),
            div
        );
    }
    // …and the route: which name to teach next, by TUs brought into reach.
    let ladder = report.section_ladder();
    println!(
        "\x20 GREEDY LADDER — next section name to teach the writer, and the resulting C \
         ({} steps from {c} to {readable}):",
        ladder.len()
    );
    let mut prev = c;
    for (name, reach) in &ladder {
        println!("\x20   +{name:<12} C = {reach:>5}   (+{})", reach - prev);
        prev = *reach;
    }
    // **The same figures again, in a form a collector can take.** See
    // `GapReport::metrics` for why: C, `A∧B∧C` and the FRONTIER are printed by
    // every scan and are still hand-copied into `STATUS.md`, and all three went
    // stale twice on 2026-08-04. `B∧C` went stale by a *dependency* moving.
    // **PROGRESS MASS — printed apart from every correctness count, with the
    // disclaimer in the header rather than in a doc nobody re-reads.** The
    // byte-exact `match` line in the GAP REPORT below is the only correctness
    // number a scan produces; this block is the leading-indicator mass and can
    // NEVER substitute for it. See `GapReport::progress_mass` and
    // `docs/PROGRESS_METRIC.md` for the design and the two structural guards.
    match report.progress_mass() {
        Some(p) => {
            let g = p.graded as f64;
            println!(
                "\n\x20 PROGRESS MASS — a PROGRESS metric, NEVER a correctness signal \
                 (docs/PROGRESS_METRIC.md).\n\
                 \x20   The byte-exact differential is the SOLE judge; `match` below is the \
                 only correctness count.\n\
                 \x20   No term rewards emitted bytes: P counts proven preconditions (A, B, C \
                 against the reference obj)\n\
                 \x20   and honest acceptance (the emitted census). A TU graded `mismatch` \
                 contributes 0 to every\n\
                 \x20   numerator, so a wrong emit always scores below the refusal it \
                 replaced.\n\
                 \x20   P = mean(a, b, c, f) over {} graded TUs = {:.5}\n\
                 \x20     a  emit-set reachable (A)      {:>6}/{}  = {:.5}\n\
                 \x20     b  binding complete (B)        {:>6}/{}  = {:.5}\n\
                 \x20     c  section shape (C)           {:>6}/{}  = {:.5}\n\
                 \x20     f  emitted fns in class   {:>6}/{}  = {:.5}\n\
                 \x20   mismatch-zeroed TUs: {}",
                p.graded,
                p.value,
                p.a,
                p.graded,
                p.a as f64 / g,
                p.b,
                p.graded,
                p.b as f64 / g,
                p.c,
                p.graded,
                p.c as f64 / g,
                p.emitted_in_class,
                p.emitted_total,
                p.emitted_in_class as f64 / p.emitted_total as f64,
                p.mismatch_zeroed,
            );
        }
        None => {
            // Deliberately NOT 100 % and deliberately not silent: a scan that
            // graded nothing must make a positive claim of that fact.
            println!(
                "\n\x20 PROGRESS MASS: NO-RESULT — 0 graded TUs (or no emitted functions). \
                 A progress number over an empty scan is unrepresentable on purpose."
            );
        }
    }
    println!(
        "\n\x20 GAP-METRICS — stable `key value` pairs for scripts/status.sh; keys are an \
         interface, do not rename. The projection `emit-predicate-worth` = \
         `b-and-c` − `a-and-b-and-c` is derived HERE on purpose (board #213):"
    );
    for (k, v) in report.metrics() {
        println!("\x20   gap-metric {k} {v}");
    }
}
