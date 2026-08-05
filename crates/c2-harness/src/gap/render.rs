//! The scan's own printed report of the factor model. Split out of `gap.rs`
//! unchanged; see [`super`] for the module docs.

use super::fnbytes::byte_fraction_exact;
use super::{GapReport, TuClass, PORT_WRITER_SECTIONS, WHOLE_TU_RECOGNIZERS};

/// **The known-answer control on the byte-fraction ranker** (board **#501**).
///
/// A `match` TU is byte-identical to `c2`'s obj, so the port demonstrably
/// produced a body for **every** `.text` byte in it: its byte fraction must read
/// **100 %**. That is the one population where the answer is known in advance,
/// and checking it is what stops the ranker from being a number nobody can
/// falsify — the numerator could silently stop crediting a whole `Selected`
/// variant and every frontier row would go on printing a plausible small
/// percentage.
///
/// **Printed as a count, never as a status** (`docs/GAPS.md` §7, STATUS trap 5):
/// `N of M matched TUs at 100 %`, with every shortfall named and its bytes
/// given. A run that graded zero matched TUs prints `0 of 0` and says so rather
/// than printing nothing.
///
/// **A shortfall here is not automatically a ranker defect.** The scan's own
/// emitted-census line already reports `ground truth VIOLATED: 2 emitted symbols
/// on the byte-exact TUs did not bind to an in-class row` — those land in
/// [`super::fnbytes::FnByte::Unbound`], which the numerator deliberately does not
/// credit, so they subtract here too. The control is read for its *movement*:
/// the named set is the thing to compare across trees.
fn render_byte_fraction_control(report: &GapReport) {
    let matched = report
        .results
        .iter()
        .filter(|r| r.class == TuClass::Match)
        .count();
    // Computed in `factors.rs` beside the ranking it controls, so the printed
    // block and the `gap-metric` keys cannot drift apart.
    let (full, nodenom, short) = report.byte_fraction_control();
    let unexplained = short.iter().filter(|(e, ..)| !*e).count();
    println!(
        "\x20 BYTE-FRACTION CONTROL (board #501) — a `match` TU is byte-identical to c2's obj, so \
         the port produced a body for every `.text` byte in it and its fraction MUST read 100%. \
         {full} of {matched} matched TUs do; {} fall short; {nodenom} have no `.text` denominator \
         (a TU that defines data and no functions — board #276's shape — which is a 100%-free \
         zero, not a failure).",
        short.len()
    );
    if short.is_empty() {
        println!("\x20   no shortfall. (A count, not a status: the {full} is the evidence.)");
    } else {
        println!(
            "\x20   SHORTFALL, by name, each classified — {} explained by factor E (whole-TU \
             emitter, per-function path blind by construction), {unexplained} UNEXPLAINED. \
             **The unexplained count is the one to watch; it must be 0.** Compare the SET across \
             trees, not its size. The FRONTIER is unaffected either way: it is defined as \
             `A and B and C and not (D or E)`, so no factor-E TU can appear in the ranking above.",
            short.len() - unexplained
        );
        for (e, r, n, d) in &short {
            println!(
                "\x20     {} ({n}/{d} bytes, {:.1}%)  {}",
                r.src,
                100.0 * *n as f64 / *d as f64,
                if *e {
                    "[factor E — a WHOLE-TU recognizer emitted this; the per-function path \
                     cannot see it. EXPECTED.]"
                } else {
                    "[NOT factor E — UNEXPLAINED, and the ranker's numerator is the first \
                     suspect.]"
                }
            );
        }
    }
}

/// **The byte-fraction ranking of the FRONTIER** (lane `w-tu3`, board **#500**).
///
/// The FRONTIER block above ranks by *blocked function count*, which is board
/// **#198**'s standing complaint and has now mis-ranked a target twice: #269
/// counted refusals and could not see what was already emitted, and **#465**
/// counted already-emitted *functions* and could not see how much of the TU they
/// were. This block is the third unit, and the only one with an outcome behind
/// it — see [`byte_fraction`] for `mmio` 72.7 % by function against 16.8 % by
/// byte, and `xboxmem` 50 % / 54.5 % with the conversion.
///
/// **Every ratio prints its denominator**, and a TU with no `.text` bytes prints
/// `n/a` under a counted reason rather than 100 %. `exact` is the
/// oracle-graded floor under `accepted`.
///
/// **This is an instrument and not a gate.** It ranks; it licenses nothing.
fn render_byte_fraction_ranking(report: &GapReport) {
    // Computed in `factors.rs`; see `GapReport::frontier_byte_ranking`.
    let rows = report.frontier_byte_ranking();
    let no_den = rows.iter().filter(|(_, f)| f.is_none()).count();
    let zero = rows.iter().filter(|(_, f)| matches!(f, Some((0, _)))).count();
    println!(
        "\x20 FRONTIER BY `.text` BYTE FRACTION (board #500) — how much of each TU's `.text` the \
         port already produces a body for, BY BYTE. Board #465 counted FUNCTIONS and was refuted \
         by the TU registered to confirm it (mmio: 72.7% by function, 16.8% by byte, DECLINED; \
         xboxmem: 50.0% by function, 54.5% by byte, CONVERTED). n = 2 outcomes — this ranks, it \
         does not license. `exact` is the oracle-graded floor under `accepted`; a TU with no \
         `.text` bytes prints n/a, NEVER 100%. {} of {} frontier TUs have NO denominator, and \
         {zero} are at EXACTLY 0% — TUs where codegen breadth has not begun:",
        no_den,
        rows.len()
    );
    // **The REMAINING column is a FOURTH unit and it is printed, not chosen
    // between** (board #505). `total − accepted` is the PowerPC the port must
    // learn to write for this TU, and on the only two cells with outcomes it
    // agrees with the fraction — `xboxmem` 60 bytes remaining and converted,
    // `mmio` 316 and declined, a 5.3x margin next to the fraction's 3.2x. On the
    // CURRENT frontier the two units DISAGREE at the head: the fraction says
    // `mmio` (16.8%, 316 B remaining) and the remainder says `Primes.cpp` (0%,
    // 64 B remaining). Neither is validated at n = 2, and this project has now
    // been wrong about the unit twice (#269, #465). Printing both is the honest
    // state; picking one on this evidence would be the third mistake.
    println!(
        "\x20    accepted/total bytes    frac   exact  remain | src   (REMAIN = total - accepted \
         = the PowerPC still to write. A FOURTH unit, printed and NOT chosen between: it agrees \
         with `frac` on both outcome cells and DISAGREES with it at this frontier's head.)"
    );
    for (r, f) in &rows {
        match f {
            Some((n, d)) => println!(
                "\x20   {n:>8}/{d:<8} bytes  {:>5.1}%  {:>5}  {:>6} | {}",
                100.0 * *n as f64 / *d as f64,
                byte_fraction_exact(r),
                d - n,
                r.src
            ),
            None => println!(
                "\x20   {:>8}/{:<8} bytes  {:>6}  {:>5}  {:>6} | {}",
                "-", 0, "n/a", "-", "n/a", r.src
            ),
        }
    }
}

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
    render_byte_fraction_ranking(report);
    render_byte_fraction_control(report);
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
    // **FUNCTION BYTE MATCH — the same judge, a finer unit.** Printed in its own
    // block, under the same disclaimer, because it is a PROGRESS instrument and
    // the `match` line below is still the only correctness count. See
    // `GapReport::fn_byte_match` and `docs/FUNCTION_BYTE_MATCH.md`.
    match report.fn_byte_match() {
        Some(f) => {
            let pct = |n: usize| 100.0 * n as f64 / f.denominator as f64;
            println!(
                "\n\x20 FUNCTION BYTE MATCH (FBM) — a PROGRESS instrument, NEVER a gate \
                 (docs/FUNCTION_BYTE_MATCH.md).\n\
                 \x20   The judge's own predicate — byte-identical to real c2 — asked per \
                 EMITTED FUNCTION instead of\n\
                 \x20   per TU. No partial credit: a wrong body scores exactly what a refusal \
                 scores, which is 0.\n\
                 \x20   The denominator is counted off c2's obj, so refusing more never \
                 shrinks it.\n\
                 \x20   FBM = ({} + {})/{} emitted functions byte-exact = {:.5}\n\
                 \x20     exact       {:>8}  ({:>5.2}%)   CREDITED — per-function route, bytes \
                 identical to c2's\n\
                 \x20     whole-TU    {:>8}  ({:>5.2}%)   CREDITED — on a TU the differential \
                 graded `match`; the judge certified the whole obj\n\
                 \x20     differs     {:>8}  ({:>5.2}%)   complete port body, bytes differ\n\
                 \x20     partial     {:>8}  ({:>5.2}%)   selected; body finished by the COFF \
                 emitter — FBM's own under-report (board #322)\n\
                 \x20     refused     {:>8}  ({:>5.2}%)   the port declines the function\n\
                 \x20     unbound     {:>8}  ({:>5.2}%)   no census row claims the symbol\n\
                 \x20     no-bytes    {:>8}  ({:>5.2}%)   COMDAT raw data did not decode\n\
                 \x20   objs unreadable (contribute NO denominator): {}   partition breaks \
                 (known answer 0): {}\n\
                 \x20   KNOWN-ANSWER CONTROL — per-function bodies that DIFFER on a TU the \
                 oracle graded `match` (must be 0): {}\n\
                 \x20   census/gate disagreement on EMITTED fns (the error term on the \
                 emitted census, target 0): {}\n\
                 \x20   NOTE: the six buckets partition the denominator by the PER-FUNCTION \
                 route alone; on a `match` TU the whole-obj verdict supersedes them.\n\
                 \x20   BYTES ARE NOT THE WHOLE FUNCTION: {} of the credited functions carry a \
                 relocation, whose\n\
                 \x20   target FBM does NOT check — a `.text` COMDAT's raw bytes do not contain \
                 its relocations.",
                f.exact,
                f.whole_tu,
                f.denominator,
                f.value,
                f.exact,
                pct(f.exact),
                f.whole_tu,
                pct(f.whole_tu),
                f.differs,
                pct(f.differs),
                f.partial,
                pct(f.partial),
                f.refused,
                pct(f.refused),
                f.unbound,
                pct(f.unbound),
                f.nobytes,
                pct(f.nobytes),
                f.obj_unreadable,
                f.partition_broken,
                f.match_tu_differs,
                f.census_disagree,
                f.exact_relocated,
            );
            let (pw, rw, ew) = f.differ_words;
            if f.differs > 0 {
                // The objdiff-shaped number, confined to the class
                // `docs/PROGRESS_METRIC.md` §2 says it is legitimate on: bodies
                // the port DID produce and got wrong. It is a forensic aid on
                // the `differs` class and is aggregated into no headline —
                // raising it by emitting more nearly-right bodies raises FBM by
                // exactly nothing.
                println!(
                    "\x20   forensic (differs class ONLY, credited nowhere): {ew} of {rw} \
                     reference words positionally equal, port wrote {pw} words"
                );
            }
            // The under-report, by shape. This is a work list, not a defect
            // list: each row is a `Selected` variant whose body only the COFF
            // emitter can finish.
            let parts = report.fn_byte_partial_histogram();
            if !parts.is_empty() {
                let rows: Vec<String> =
                    parts.iter().map(|(k, n)| format!("{k} {n}")).collect();
                println!("\x20   partial by shape: {}", rows.join(" · "));
            }
            // Per-TU FBM, nearest first — the answer to "we are 8/878 exact, how
            // close is the other 870?" stated in TUs rather than in one ratio.
            let by_tu = report.fn_byte_by_tu();
            let buckets = [1.0f64, 0.9, 0.5, 0.1];
            let dist: Vec<String> = buckets
                .iter()
                .map(|b| {
                    let n = by_tu
                        .iter()
                        .filter(|(_, e, d)| *e as f64 / *d as f64 >= *b - 1e-12)
                        .count();
                    format!("≥{:.0}%: {n}", b * 100.0)
                })
                .collect();
            println!(
                "\x20   per-TU FBM over {} TUs with emitted functions — {}",
                by_tu.len(),
                dist.join(", ")
            );
        }
        None => {
            println!(
                "\n\x20 FUNCTION BYTE MATCH: NO-RESULT — no emitted function was graded. \
                 A ratio over zero functions is unrepresentable on purpose (objdiff's \
                 calc_fuzzy_match_percent returns 100.0 here; that is the bug, not the \
                 baseline)."
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
