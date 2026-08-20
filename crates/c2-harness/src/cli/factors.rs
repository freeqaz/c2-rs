//! `c2rs factors` — the Phase-7 factor sets **as sets**: re-derive every
//! published count from the per-TU listing, and intersect that listing with
//! somebody else's per-TU set.
//!
//! Offline over a `c2rs gap --factors-tsv` file. **No toolchain, no capture, no
//! scan** — it reads a text file and prints. That is the gating answer this lane
//! owed: the listing itself stays opt-in and a file (board #352's reasoning is
//! unchanged — `gap` is also the engine under `mode_lane.sh`/`mode_cross.sh`,
//! where one row per graded TU is tens of thousands of lines per lane), and the
//! *algebra over it* costs nothing on any scan because it does not run on one.
//!
//! See [`c2_harness::gap::sets`] for why the join is a reported, refusable step
//! and not a `join(1)`.

use std::process::ExitCode;

use c2_harness::gap::sets::{
    self, check_metrics, intersections, join, parse_candidate, parse_factors_tsv, scrape_metrics,
    NAMED_SETS,
};

use crate::{Args, Arity, Spec};

static FACTORS_SPEC: Spec = Spec {
    cmd: "factors",
    opts: &[
        ("--tsv", Arity::Value),
        ("--set", Arity::Repeated),
        ("--check-metrics", Arity::Value),
        ("--list", Arity::Value),
        ("--plan-tsv", Arity::Value),
    ],
    requires: &[],
    max_positionals: 0,
};

fn usage() {
    eprintln!(
        "usage: c2rs factors --tsv PATH [--check-metrics GAPLOG] [--list SETNAME] \
         [--set NAME=PATH]...\n\
         \n\
         \x20 --tsv PATH            a `c2rs gap --factors-tsv` file: one row per GRADED TU\n\
         \x20 --check-metrics PATH  a gap scan log; compares every re-derived count against\n\
         \x20                       the `gap-metric` line the scan published (known-answer)\n\
         \x20 --list SETNAME        print the members of one set, one per line\n\
         \x20 --plan-tsv PATH       a `c2rs gap --plan-tsv` file: the OBJECT PLAN grade per\n\
         \x20                       TU. With --check-metrics, re-derives every `plan-*`\n\
         \x20                       count from the rows and diffs it against the scan's\n\
         \x20                       own published figure (#3288, the second derivation)\n\
         \x20 --set NAME=PATH       intersect a candidate per-TU set (one TU name per line,\n\
         \x20                       `#` comments) against every set below. Repeatable.\n\
         \n\
         sets: {}",
        NAMED_SETS.iter().map(|s| s.name).collect::<Vec<_>>().join(" ")
    );
}

pub(crate) fn cmd_factors(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&FACTORS_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let Some(tsv) = args.path("--tsv") else {
        usage();
        return ExitCode::from(2);
    };
    let text = match std::fs::read_to_string(&tsv) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("factors: cannot read --tsv {}: {e}", tsv.display());
            return ExitCode::FAILURE;
        }
    };
    let rows = match parse_factors_tsv(&text) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("factors: {} is not a usable factor listing: {e}", tsv.display());
            return ExitCode::FAILURE;
        }
    };

    println!(
        "c2rs factors — the Phase-7 A/B/C/D/E sets, as SETS\n\
         \x20 {} graded TU rows from {}\n\
         \x20 A `capture-fail` TU is NOT a row and is not a zero row: it was never measured.\n\
         \x20 Every cardinality below is taken over these rows and nothing else.",
        rows.len(),
        tsv.display()
    );

    println!("\nSET CARDINALITIES");
    for s in NAMED_SETS {
        let n = rows.iter().filter(|r| (s.pred)(r)).count();
        println!(
            "\x20 {:<26} {:>5}   {}{}",
            s.name,
            n,
            s.blurb,
            match s.metric {
                Some(k) => format!(" [gap-metric {k}]"),
                None => String::new(),
            }
        );
    }

    let mut bad = false;

    // ---- THE OBJECT PLAN's second derivation (#3288, lane `w-objplan`) -------
    //
    // `plan-*` is published by `GapReport::metrics()` over the live results.
    // This is the other producer: the same counts re-derived from the
    // `--plan-tsv` ROWS, offline, by `gap::plan::derive_metrics`. A published
    // figure and its own listing must not be able to disagree.
    if let Some(ptsv) = args.path("--plan-tsv") {
        match std::fs::read_to_string(&ptsv) {
            Err(e) => {
                eprintln!("factors: cannot read --plan-tsv {}: {e}", ptsv.display());
                return ExitCode::FAILURE;
            }
            Ok(t) => match c2_harness::gap::plan::parse_plan_tsv(&t) {
                None => {
                    // Fail-closed, like the writer: a parser that SKIPPED a
                    // malformed row would re-derive a smaller count than the
                    // scan published, and the disagreement would then be
                    // unattributable.
                    eprintln!(
                        "factors: {} is not a usable object-plan listing (a malformed row \
                         refuses the whole file rather than being skipped)",
                        ptsv.display()
                    );
                    return ExitCode::FAILURE;
                }
                Some(prows) => {
                    let derived = c2_harness::gap::plan::derive_metrics(&prows);
                    println!(
                        "\nOBJECT PLAN — {} graded TU rows from {}",
                        prows.len(),
                        ptsv.display()
                    );
                    match args.path("--check-metrics") {
                        None => {
                            for (k, v) in &derived {
                                println!("\x20 {k:<40} {v:>7}");
                            }
                            println!(
                                "\x20 (counts only — pass --check-metrics <gap log> to diff \
                                 them against what the scan PUBLISHED)"
                            );
                        }
                        Some(log) => {
                            let pub_text = std::fs::read_to_string(&log).unwrap_or_default();
                            let published = scrape_metrics(&pub_text);
                            println!(
                                "\x20 SECOND DERIVATION (#3288) — every `plan-*` count \
                                 re-derived from the rows, against the `gap-metric` line the \
                                 scan published:"
                            );
                            let (mut ok, mut dis, mut abs) = (0usize, 0usize, 0usize);
                            for (k, d) in &derived {
                                let p = published.get(k).copied();
                                let verdict = match p {
                                    None => {
                                        abs += 1;
                                        "ABSENT"
                                    }
                                    Some(v) if v == *d => {
                                        ok += 1;
                                        "OK"
                                    }
                                    Some(_) => {
                                        dis += 1;
                                        "DISAGREE"
                                    }
                                };
                                println!(
                                    "\x20 {k:<40} published {:>7}  derived {d:>7}  {verdict}",
                                    match p {
                                        Some(v) => v.to_string(),
                                        None => "-".to_string(),
                                    }
                                );
                            }
                            // **THE COVERAGE, AND ITS COMPLEMENT.** "13 OK"
                            // reads as done; the first version of this check
                            // covered 13 of 48 published keys and the omissions
                            // included the PRIMARY GRADING CRITERION. So the
                            // uncovered keys are NAMED, and a unit test asserts
                            // the list is exactly the uncovered set.
                            let uncovered = c2_harness::gap::plan::uncovered_metric_keys();
                            let published_plan = published
                                .keys()
                                .filter(|k| k.starts_with("plan-"))
                                .count();
                            println!(
                                "\x20 {ok} OK, {dis} DISAGREE, {abs} ABSENT. An ABSENT is its \
                                 own verdict and not a pass: a control that checks nothing and \
                                 a control that passes look identical in a summary line."
                            );
                            println!(
                                "\x20 COVERAGE — {} of the {published_plan} `plan-*` keys in \
                                 this log are re-derived here. The rest are NOT a silence: \
                                 they are the observe-side inventory, which is a sum over the \
                                 reference obj and is not a column in the TSV, so no parser \
                                 over it can reach them. Named: {}",
                                derived.len(),
                                uncovered.join(" ")
                            );
                            if dis > 0 || ok == 0 {
                                bad = true;
                            }
                        }
                    }
                }
            },
        }
    }

    // ---- the known-answer control -------------------------------------
    if let Some(log) = args.path("--check-metrics") {
        match std::fs::read_to_string(&log) {
            Err(e) => {
                eprintln!("factors: cannot read --check-metrics {}: {e}", log.display());
                return ExitCode::FAILURE;
            }
            Ok(t) => {
                let published = scrape_metrics(&t);
                let checks = check_metrics(&rows, &published);
                println!(
                    "\nKNOWN-ANSWER CONTROL — every re-derived count vs the `gap-metric` line \
                     the scan published, {}",
                    log.display()
                );
                if published.is_empty() {
                    println!(
                        "\x20 THE LOG CARRIES NO `gap-metric` LINES AT ALL. Every row below \
                         would read ABSENT, which is not a pass — it is a control that \
                         checked nothing."
                    );
                    bad = true;
                }
                let (mut ok, mut dis, mut abs) = (0, 0, 0);
                for c in &checks {
                    match c.verdict() {
                        "OK" => ok += 1,
                        "DISAGREE" => dis += 1,
                        _ => abs += 1,
                    }
                    println!(
                        "\x20 {:<26} published {:>6}  derived {:>6}  {}",
                        c.key,
                        c.published.map(|v| v.to_string()).unwrap_or_else(|| "-".into()),
                        c.derived,
                        c.verdict()
                    );
                }
                println!("\x20 {ok} OK, {dis} DISAGREE, {abs} ABSENT of {} keys", checks.len());
                if dis > 0 {
                    println!(
                        "\x20 A DISAGREE means the LISTING and the PUBLISHED COUNT are not the \
                         same measurement, so an intersection taken against these rows is not \
                         an intersection with the set that count names. Exit is non-zero."
                    );
                    bad = true;
                }
                if abs > 0 {
                    println!(
                        "\x20 An ABSENT key was not in the log — the control did not check it. \
                         Reported as its own verdict because a control that checks nothing and \
                         a control that passes look the same in a summary line."
                    );
                }
            }
        }
    } else {
        println!(
            "\nKNOWN-ANSWER CONTROL: NOT REQUESTED — pass `--check-metrics <gap scan log>` to \
             compare every count above against the `gap-metric` line the scan published. \
             Without it the cardinalities above are unchecked arithmetic over a file."
        );
    }

    // ---- one set's members --------------------------------------------
    if let Some(name) = args.get("--list") {
        match sets::members(&rows, name) {
            None => {
                eprintln!(
                    "factors: --list {name:?} is not a set. known sets: {}",
                    NAMED_SETS.iter().map(|s| s.name).collect::<Vec<_>>().join(" ")
                );
                return ExitCode::from(2);
            }
            Some(m) => {
                println!("\nMEMBERS OF `{name}` — {} TUs", m.len());
                for s in &m {
                    println!("{s}");
                }
                if m.is_empty() {
                    println!(
                        "\x20 (empty — and that is printed as a statement rather than left as \
                         a blank block)"
                    );
                }
            }
        }
    }

    // ---- the intersections --------------------------------------------
    let candidates = args.all("--set");
    if candidates.is_empty() {
        println!(
            "\nINTERSECTIONS: NONE REQUESTED — pass `--set NAME=PATH` with a per-TU candidate \
             set (a model's exact set, say) to price it in TU reach. `|{{model exact}} ∩ B∧C|` \
             is the number w-emitp declined to extrapolate as `151 × 0.555`."
        );
    }
    for spec in candidates {
        let Some((name, path)) = spec.split_once('=') else {
            eprintln!("factors: --set expects NAME=PATH, got {spec:?}");
            return ExitCode::from(2);
        };
        let t = match std::fs::read_to_string(path) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("factors: cannot read --set {name} at {path}: {e}");
                return ExitCode::FAILURE;
            }
        };
        let cand = match parse_candidate(name, &t) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("factors: {e}");
                return ExitCode::FAILURE;
            }
        };
        let jc = join(&rows, &cand);
        println!("\nINTERSECT `{name}` — {path}");
        println!(
            "\x20 JOIN: {} lines, {} distinct names ({} duplicate), {} RESOLVED to a graded row, \
             {} UNRESOLVED; {} graded rows are absent from the set",
            cand.lines,
            cand.names.len(),
            cand.duplicates,
            jc.resolved.len(),
            jc.unresolved.len(),
            jc.absent
        );
        if !jc.unresolved.is_empty() {
            let show: Vec<&str> =
                jc.unresolved.iter().take(5).map(String::as_str).collect();
            println!("\x20 unresolved examples: {}", show.join(" "));
            if let Some(h) = &jc.hint {
                println!("\x20 HINT: {h}");
            }
        }
        if jc.is_empty() {
            println!(
                "\x20 *** THE JOIN RESOLVED NOTHING. Every intersection below would be 0, and \
                 a table of zeros reads as \"this set buys no reach\" when what happened is \
                 that the KEY IS WRONG. Refusing to print one. ***"
            );
            bad = true;
            continue;
        }
        println!(
            "\x20 INTERSECTIONS — |{name} ∩ S|, over the {} resolved names",
            jc.resolved.len()
        );
        let ints = intersections(&rows, &jc.resolved);
        for s in NAMED_SETS {
            let n = ints.get(s.name).copied().unwrap_or(0);
            let total = rows.iter().filter(|r| (s.pred)(r)).count();
            println!("\x20   |{:<26} ∩ cand| = {:>5}  of |S| = {:>5}", s.name, n, total);
        }
        let g = |k: &str| ints.get(k).copied().unwrap_or(0);
        let card = |k: &str| sets::count(&rows, k).unwrap_or(0);
        println!(
            "\x20 REACH PRICE (board #213, generalized from a PERFECT predicate to this one)\n\
             \x20   by reach:    |cand ∩ (B∧C ∖ A∧B∧C)| = {}   of the {} #213 prices a perfect \
             emit predicate at\n\
             \x20   by frontier: |cand ∩ (frontier-if-A ∖ FRONTIER)| = {}   of {}\n\
             \x20   #213's two arithmetics are DIFFERENT quantities and both are printed; they \
             coincide only while no TU inside B∧C fails A while already being accepted.\n\
             \x20   Neither number is a schedule and neither is a conversion: a TU in the reach \
             pool still needs codegen. This prices the emit predicate ALONE.",
            g("reach-pool"),
            card("reach-pool"),
            g("frontier-pool"),
            card("frontier-pool"),
        );
        if g("reach-pool") == 0 {
            println!(
                "\x20   The reach price is ZERO and the join resolved {} names, so this is a \
                 measurement and not a broken key: every TU this set is exact on either \
                 already satisfies A or fails B or C.",
                jc.resolved.len()
            );
        }
    }

    if bad {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
