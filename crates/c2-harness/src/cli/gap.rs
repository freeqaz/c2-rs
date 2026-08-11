//! `c2rs gap` — the real-workload gap scan: classify every TU, rank the
//! blockers. One command, and the largest of them.

use std::path::PathBuf;
use std::process::ExitCode;

use c2_harness::provenance::Provenance;

use crate::{Args, Arity, Spec};
use crate::cli::util::Scratch;

// ---------------------------------------------------------------------------
// gap — real-workload gap scan
// ---------------------------------------------------------------------------

use c2_harness::gap::{gap_scan, GapConfig, TuClass};

/// `c2rs gap --list FILE --flags-file FILE [--cwd DIR] …` — scan real TUs,
/// classify each (capture-fail / vocab-gap / codegen-gap / mismatch / match),
/// and rank the blockers. Exit is non-zero only on a *correctness* signal
/// (`mismatch` TUs or a replay-soundness divergence) or a harness error —
/// gaps themselves are the expected measurement, not a failure.
static GAP_SPEC: Spec = Spec {
    cmd: "gap",
    opts: &[
        ("--list", Arity::Value),
        ("--flags-file", Arity::Value),
        ("--cwd", Arity::Value),
        ("--limit", Arity::Value),
        ("--jobs", Arity::Value),
        ("--replay-every", Arity::Value),
        ("--jsonl", Arity::Value),
        ("--fnbyte-diff-jsonl", Arity::Value),
        ("--factors-tsv", Arity::Value),
        ("--work", Arity::Value),
        ("--cache", Arity::Value),
        ("--no-cache", Arity::Flag),
        ("--validate-cache", Arity::Value),
    ],
    requires: &[],
    max_positionals: 0,
};

pub(crate) fn cmd_gap(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&GAP_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let (list_file, flags_file) = (args.path("--list"), args.path("--flags-file"));
    let cwd = args.path("--cwd");
    let jsonl = args.path("--jsonl");
    // The per-DIFFERING-FUNCTION diff signature (board #976). Opt-in and a file
    // for the same reason `--factors-tsv` is; the `fndiff-*` counts it is
    // derived from print on every scan regardless.
    let fndiff_jsonl = args.path("--fnbyte-diff-jsonl");
    // The per-TU factor membership. Opt-in and a file, not stdout: see
    // `GapReport::factor_membership`.
    let factors_tsv = args.path("--factors-tsv");
    let work = args.path("--work");
    // `--limit`/`--jobs`/`--replay-every`/`--validate-cache` used to exit 2 with
    // NO message when the value did not parse. `num` names the option and echoes
    // the value it choked on.
    let limit: Option<usize> = match args.num("--limit") {
        Ok(v) => v,
        Err(c) => return c,
    };
    let jobs: usize = match args.num("--jobs") {
        Ok(Some(v)) => v,
        Ok(None) => std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4),
        Err(c) => return c,
    };
    let replay_every: usize = match args.num("--replay-every") {
        Ok(v) => v.unwrap_or(0),
        Err(c) => return c,
    };
    let validate_cache: usize = match args.num("--validate-cache") {
        Ok(v) => v.unwrap_or(0),
        Err(c) => return c,
    };
    // Capture cache: ON by default (roadmap #15). The key is content-addressed
    // (source bytes + flags + toolchain + workload-tree identity), never mtimes;
    // `--no-cache` bypasses it and `--validate-cache N` re-captures every Nth
    // hit and byte-compares. Default root is under the gitignored `work/`.
    //
    // `--cache` and `--no-cache` used to be ORDER-DEPENDENT, because each simply
    // assigned as the scan walked: `--cache X --no-cache` disabled the cache and
    // dropped X, while `--no-cache --cache X` used X. One of those two spellings
    // did something other than what it says, and which one depended on argument
    // order. They are contradictory, so refuse both spellings.
    if args.has("--no-cache") && args.get("--cache").is_some() {
        eprintln!("gap: --cache and --no-cache contradict each other; give one");
        return ExitCode::from(2);
    }
    // `--validate-cache N` re-captures every Nth *hit*, so with no cache there
    // are no hits and nothing is validated — but the report still printed
    // "validating every Nth hit" under a `DISABLED (--no-cache)` cache line.
    if args.has("--no-cache") && validate_cache > 0 {
        eprintln!(
            "gap: --validate-cache has nothing to validate with --no-cache; refusing rather \
             than reporting a validation that cannot run"
        );
        return ExitCode::from(2);
    }
    let cache: Option<PathBuf> = if args.has("--no-cache") {
        None
    } else {
        Some(args.path("--cache").unwrap_or_else(|| {
            std::env::var_os("C2RS_GAP_CACHE")
                .map(PathBuf::from)
                // `main_repo_root`, not `repo_root`: the latter is
                // CARGO_MANIFEST_DIR and so resolves to the *worktree* a lane's
                // binary was built in, which is how 50 separate caches came to
                // exist. See its doc comment for why this is resolved in code
                // rather than exported as an env var.
                .unwrap_or_else(|| {
                    c2_harness::provenance::main_repo_root().join("work/capture-cache")
                })
        }))
    };
    let (Some(list_file), Some(flags_file)) = (list_file, flags_file) else {
        eprintln!(
            "usage: c2rs gap --list FILE --flags-file FILE [--cwd DIR] [--limit N] \
             [--jobs N] [--replay-every N] [--jsonl PATH] [--fnbyte-diff-jsonl PATH] \
             [--factors-tsv PATH] \
             [--work DIR]\n\
             (generate the dc3 workload inputs with scripts/gen_dc3_workload.sh)"
        );
        return ExitCode::from(2);
    };

    let read_tokens = |p: &PathBuf, split: bool| -> std::io::Result<Vec<String>> {
        let text = std::fs::read_to_string(p)?;
        let mut out = Vec::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if split {
                out.extend(line.split_whitespace().map(String::from));
            } else {
                out.push(line.to_string());
            }
        }
        Ok(out)
    };
    let sources = match read_tokens(&list_file, false) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("cannot read --list {}: {e}", list_file.display());
            return ExitCode::FAILURE;
        }
    };
    let flags = match read_tokens(&flags_file, true) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("cannot read --flags-file {}: {e}", flags_file.display());
            return ExitCode::FAILURE;
        }
    };

    let Some(tc) = args.toolchain() else {
        return ExitCode::SUCCESS;
    };
    if !tc.has_strace() {
        println!("SKIP: strace absent (needed to keep IL bundles during capture)");
        return ExitCode::SUCCESS;
    }
    if replay_every > 0 && !tc.has_mingw() {
        println!("SKIP: i686-w64-mingw32-gcc absent (needed for --replay-every)");
        return ExitCode::SUCCESS;
    }

    // `gap_scan` mints and deletes a `tuNNNNN` subdir per TU inside this one, so
    // the container is empty by the time the scan returns and the report has
    // gone to stdout/`--jsonl`/`--factors-tsv`. Held in a binding, not inlined,
    // so it outlives `cfg` and removes the dir on every exit -- including the
    // early returns below, which is how 1,924 of these accumulated in one day.
    // A user-supplied `--work` is theirs and is left alone.
    let work = Scratch::or_work(work, "gap");
    let cfg = GapConfig {
        sources,
        flags,
        cwd,
        limit,
        jobs,
        replay_every,
        jsonl,
        fndiff_jsonl,
        factors_tsv,
        work: work.path().to_path_buf(),
        cache,
        validate_cache,
    };
    let total = cfg.limit.unwrap_or(cfg.sources.len()).min(cfg.sources.len());
    println!(
        "gap scan: {total} TUs, {} flags, jobs={}, replay-every={}",
        cfg.flags.len(),
        cfg.jobs,
        cfg.replay_every
    );
    println!(
        "  capture cache: {}{}",
        match &cfg.cache {
            Some(p) => p.display().to_string(),
            None => "DISABLED (--no-cache)".to_string(),
        },
        if cfg.validate_cache > 0 {
            format!("  (validating every {}th hit)", cfg.validate_cache)
        } else {
            String::new()
        }
    );
    // Roadmap #46/#48: name the corpus, the binary, and the loader BEFORE the
    // numbers. A moved corpus once matched on `fn_total` and a stale wibo once
    // faked a replay alarm; neither was visible in any line of the old report.
    print!("{}", Provenance::collect(&tc, cfg.cwd.as_deref()).render());

    let t0 = std::time::Instant::now();
    let report = match gap_scan(&tc, &cfg, &|n, tot, r| {
        println!(
            "  [{n}/{tot}] {:<12} {}{}",
            r.class.label(),
            r.src,
            if r.reason.is_empty() || r.class == TuClass::Match {
                String::new()
            } else {
                format!("  ({})", r.reason)
            }
        );
    }) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("gap scan failed: {e}");
            return ExitCode::FAILURE;
        }
    };
    let elapsed = t0.elapsed();

    let n = report.results.len().max(1);
    println!("\nGAP REPORT ({} TUs in {:.1}s)", report.results.len(), elapsed.as_secs_f64());
    for class in [
        TuClass::Match,
        TuClass::Mismatch,
        TuClass::CodegenGap,
        TuClass::VocabGap,
        TuClass::PortError,
        TuClass::CaptureFail,
    ] {
        let c = report.count(class);
        println!("  {:<13} {:>5}  {:>5.1}%", class.label(), c, 100.0 * c as f64 / n as f64);
    }
    let (checked, diverged) = report.replay_stats();
    if checked > 0 {
        println!("  replay soundness: {checked} checked, {diverged} diverged");
    }
    let cs = &report.cache;
    if cfg.cache.is_some() {
        println!(
            "  capture cache: {} hit, {} miss, {} uncacheable  |  validator: {} re-captured \
             and agreed ({} of them only after zeroing the COFF TimeDateStamp), {} POISONED",
            cs.hits, cs.misses, cs.bypassed, cs.validated, cs.timestamp_only, cs.poisoned
        );
        for line in cs.poison_detail.iter().take(10) {
            println!("    POISONED {line}");
        }
        if cs.poison_detail.len() > 10 {
            println!("    … and {} more", cs.poison_detail.len() - 10);
        }
        // Entries refused on provenance (board #1388). Printed as its own line,
        // and printed even when it is 0, because 0 is the expected reading and a
        // guard whose result is only ever shown when it fires is a guard nobody
        // can tell apart from one that is not wired up.
        println!(
            "  cache entries REFUSED on provenance: {} (expected 0 — an entry whose \
             recorded capture path is not where it is being served from is re-captured, \
             never served)",
            cs.foreign
        );
        for line in cs.foreign_detail.iter().take(10) {
            println!("    REFUSED {line}");
        }
        if cs.foreign_detail.len() > 10 {
            println!("    … and {} more", cs.foreign_detail.len() - 10);
        }
    }

    // P2b function-level census. The TU ladder above is all-or-nothing, so it
    // reads 0% until a whole TU comes in class; this is the fine-grained
    // numerator that actually moves per widening step, plus the ranked
    // histogram that chooses the next step (docs/ROADMAP.md §G5).
    let (in_class, fn_total) = report.fn_coverage();
    if fn_total > 0 {
        println!(
            "\n  FUNCTION CENSUS (P2b): {in_class}/{fn_total} functions in class ({:.2}%)",
            100.0 * in_class as f64 / fn_total as f64
        );
        // The census/gate cross-check (roadmap #44). The numerator above is the
        // public claim, so its error term is printed beside it on every scan
        // rather than being characterized once and forgotten: these are the
        // functions the census calls in class and `PortC2`'s own per-function
        // selector refuses. It must read 0.
        let disagree = report.fn_gate_disagreement();
        if disagree == 0 {
            println!("  census/gate disagreement: 0  (the port accepts every function above)");
        } else {
            println!(
                "  census/gate DISAGREEMENT: {disagree} ({:.2}% of the numerator) — the census \
                 OVER-CLAIMS by this much",
                100.0 * disagree as f64 / in_class.max(1) as f64
            );
            for (key, count) in report.fn_gate_histogram().iter().take(15) {
                println!("    {count:>7}  {key}");
            }
        }
        // The `.gl` binding invariants (D14). A binding decides *which* symbol a
        // token names, and a green differential cannot grade a correspondence
        // (`docs/GAPS.md` §6). These are the two facts the container settles by
        // itself, and both have a known answer of 0.
        let binds = report.bind_check_histogram();
        if !binds.is_empty() {
            let bad = report.bind_violations();
            if bad == 0 {
                println!(
                    "  .gl binding invariants: 0 violations (every generated destructor \
                     resolves to a destructor)"
                );
            } else {
                println!("  .gl binding VIOLATIONS: {bad} — a token bound to the wrong symbol");
            }
            for (key, count) in binds.iter().take(8) {
                println!("    {count:>7}  {key}");
            }
        }
        // ---- The EMITTED-function census (`docs/GAPS.md` §8) ----------------
        //
        // The headline above counts IL bodies. c2 emits about 7 % of them, and
        // for a body it never emits "in class" is a parser-only claim no byte
        // compare has ever graded or ever can. This block is the same numerator
        // restricted to functions that actually appear in an obj, and it is a
        // FLOOR: every emitted symbol the binding could not claim is printed as
        // residue, never folded into the numerator or out of the denominator.
        let (emit_in_class, emitted) = report.emit_coverage();
        if emitted > 0 {
            println!(
                "\n  EMITTED CENSUS (§8): {emit_in_class}/{emitted} emitted functions in class \
                 ({:.2}%)",
                100.0 * emit_in_class as f64 / emitted as f64
            );
            let (gen, unexplained) = report.emit_residue();
            let bound = report.emit_total("emit-bound");
            println!(
                "    bound {bound}  |  residue {}: {gen} compiler-generated (no IL body), \
                 {unexplained} unexplained  ({:.2}% of the denominator)",
                gen + unexplained,
                100.0 * (gen + unexplained) as f64 / emitted as f64
            );
            println!(
                "    ceiling if every residue symbol were in class: {} ({:.2}%)",
                emit_in_class + gen + unexplained,
                100.0 * (emit_in_class + gen + unexplained) as f64 / emitted as f64
            );
            // Ground truth. On a byte-exact TU the oracle has already graded the
            // whole symbol table, so this binding's answer there is checkable
            // rather than merely self-consistent. Known answer 0.
            let mtu = report.count(TuClass::Match);
            let mres = report.emit_match_tu_residue();
            if mres == 0 {
                println!(
                    "    ground truth: {mtu} byte-exact TUs, every emitted symbol bound to an \
                     in-class row (residue 0)"
                );
            } else {
                println!(
                    "    ground truth VIOLATED: {mres} emitted symbols on the {mtu} byte-exact \
                     TUs did not bind to an in-class row — the BINDING is wrong, not the port"
                );
            }
            let broken = report.emit_total("emit-accounting-broken");
            let unreadable = report.emit_total("emit-obj-unreadable");
            println!(
                "    binding: {} records, {} nameless, {} before the first row, {} row-conflicts, \
                 {} name-conflicts, {broken} accounting breaks, {unreadable} unreadable objs",
                report.emit_total("emit-records"),
                report.emit_total("emit-record-nameless"),
                report.emit_total("emit-record-outside"),
                report.emit_total("emit-row-conflict"),
                report.emit_total("emit-name-conflict"),
            );
            // ARITY, beside totality and never instead of it (#144). The counts
            // answer different questions: `records` is how many records the
            // FRAMING found, `record-offsets` is their contents. A change to the
            // NAME scan must move `nameless` above and leave both of these fixed
            // — which is how W-VGL's `26`-separator repair was held to being a
            // naming repair (records 1,515,160 before and after, nameless
            // 152,941 -> 420).
            println!(
                "    binding arity (#144 — residue 0 is not a control): {} record offsets \
                 against {} records, {} arity breaks (known answer 0)",
                report.emit_total("emit-record-offsets"),
                report.emit_total("emit-records"),
                report.emit_total("emit-arity-broken"),
            );
            // **The `.in` initializer reader's own report** (board #936). Same
            // shape as the binding rows above and for the same reason: a decode
            // cannot be graded by the oracle, so it is graded on its own
            // invariants — totality (`records == values + residue + conflicts`),
            // arity (`elements`, the records' contents) and injectivity
            // (`conflicts`).
            println!(
                "    .in initializers: {} records = {} accepted + {} residue, {} elements \
                 (ARITY), {} values (TOKENS, not records), {} duplicate records, {} conflicts, \
                 {} accounting breaks (known answer 0)",
                report.emit_total("in-init-records"),
                report.emit_total("in-init-accepted"),
                report.emit_total("in-init-residue"),
                report.emit_total("in-init-elements"),
                report.emit_total("in-init-values"),
                report.emit_total("in-init-duplicate-records"),
                report.emit_total("in-init-conflicts"),
                report.emit_total("in-init-accounting-broken"),
            );
            println!(
                "    .in symbol addresses (element tag 02, board #931): {} elements over {} \
                 records",
                report.emit_total("in-init-symrefs"),
                report.emit_total("in-init-records-with-symrefs"),
            );
            // **THE DENOMINATOR THE LINE ABOVE IS SILENT ABOUT — board #961.**
            // `records` counts what the `00 01`/`00 02` anchor scan reaches;
            // these count what it does not, so `records == accepted + residue`
            // can no longer read as a statement about the whole stream. The
            // three are printed beside the identity and never folded into it —
            // that is the difference between publishing a denominator and
            // widening a control until it goes green (`docs/STATUS.md` trap 0).
            println!(
                "    .in UNANCHORED (the denominator, board #961): {} records whose first \
                 element is a tag-03 blob or a tag-08 fill, {} `00 02` candidates dropped by \
                 the fail-closed arm, {} anchors with no token — none of these is in \
                 `records` OR in the residue",
                report.emit_total("in-init-unanchored"),
                report.emit_total("in-init-fail-closed"),
                report.emit_total("in-init-no-token"),
            );
            // EVERY reason, including the zeroes — a residue reason that stops
            // occurring must read `0` and not vanish (trap 5).
            //
            // **Driven from `InInitResidue::ALL`, because a hand-kept copy of
            // this list is the same trap one level down and it fired.** The six
            // names used to be spelled out here; `w-inread` added three
            // (`pointer-width`, `zero-fill`, `inline-bytes`), the reader
            // reported them, `scan.rs` aggregated them under their own keys —
            // and this loop printed the other six and no one of the three, so
            // the first 878-TU run of the widened reader showed a residue
            // histogram that silently did not sum to `in-init-residue`. A
            // reason that CANNOT be printed is worse than one that reads `0`.
            for reason in c2_il::InInitResidue::ALL.iter().map(|r| r.key()) {
                println!(
                    "      .in residue {reason:<20} {:>8}",
                    report.emit_total(&format!("in-init-residue-{reason}")),
                );
            }
            // What the unexplained residue IS. A residue reported only as a
            // number cannot be attacked and cannot be checked; these rows say
            // whether it is a population c2 synthesizes (concentrated in the
            // special-member classes) or the binding losing ordinary functions.
            let by_class: Vec<(String, usize)> = report
                .emit_histogram()
                .into_iter()
                .filter(|(k, _)| k.starts_with("emit-residue-unbound|"))
                .collect();
            for (key, n) in by_class.iter().take(10) {
                println!(
                    "      residue {:<20} {n:>7} ({:>5.1}% of the unexplained)",
                    key.trim_start_matches("emit-residue-unbound|"),
                    100.0 * *n as f64 / unexplained.max(1) as f64
                );
            }
            // The payoff metric's leading indicator, as a distribution.
            let buckets = [0usize, 1, 10, 100, 1000];
            let dist: Vec<String> = buckets
                .iter()
                .map(|b| format!("≤{b}: {}", report.near_match_tus(*b).len()))
                .collect();
            println!("    TU distance to matching (blocked functions) — {}", dist.join(", "));
            // The same distribution over the population the goal is written in.
            // Published beside the body one because they are different numbers
            // AND a different order: `Rand2.cpp` is 8 blocked bodies / 2 blocked
            // emitted, `vec.cpp` is 565 blocked bodies / 0 blocked emitted.
            let dist_e: Vec<String> = buckets
                .iter()
                .map(|b| format!("≤{b}: {}", report.near_match_tus_emitted(*b).len()))
                .collect();
            println!(
                "    TU distance to matching (blocked EMITTED functions) — {}",
                dist_e.join(", ")
            );
            // The ceiling neither distance can see: the port emits one `.text`
            // COMDAT per `.ex` function segment and has no emit-set model, so a
            // TU whose segment count differs from its obj's COMDAT-leader count
            // cannot match however good the codegen gets.
            let reach = report.emit_set_reachable_tus();
            let graded = report
                .results
                .iter()
                .filter(|r| r.class != c2_harness::gap::TuClass::CaptureFail)
                .count();
            let viol = report.emit_set_violations();
            println!(
                "    emit-set ceiling: {} of {graded} graded TUs have `.ex` segments == obj `.text` \
                 COMDATs — the most TU match can reach before ROADMAP §8.3 Phase 7 \
                 (violations among matching TUs, must be 0: {viol})",
                reach.len()
            );
            // **W-EMITSET — the ceiling on a MODEL, which is a different and
            // lower number than the ceiling on today's model-free port.**
            //
            // The line above compares two counts. A model has to reproduce the
            // reference `.text` COMDAT *set*, and it can only ever emit a COMDAT
            // for a body this bundle carries, under the name the binding gives
            // it. So the bound is per TU: does every emitted symbol bind (today),
            // and failing that does every emitted symbol at least HAVE a `.gl`
            // body record (repaired)? The gap between the two is `bind.rs` work;
            // what remains after it is a wall that needs COMDAT synthesis.
            let (c_today, c_repaired, c_wall) = (
                report.emit_total("emit-set-ceiling-today"),
                report.emit_total("emit-set-ceiling-repaired"),
                report.emit_total("emit-set-ceiling-wall"),
            );
            let (u_body, u_none) = (
                report.emit_total("emit-unbound-has-record"),
                report.emit_total("emit-unbound-no-record"),
            );
            println!(
                "    emit-set MODEL ceiling: {c_today} of {graded} TUs bind every emitted symbol \
                 today; {c_repaired} would if `bind.rs` lost none; {c_wall} carry an emitted \
                 symbol with NO `.gl` body record and are a wall for any segment-driven model"
            );
            println!(
                "      unbound emitted symbols: {u_body} have a body record (instrument defect), \
                 {u_none} have none (wall) — nesting invariant, must hold: \
                 today <= repaired ({}), repaired + wall == graded ({})",
                c_today <= c_repaired,
                c_repaired + c_wall == graded
            );

            let eh = report.emit_blocker_histogram();
            if !eh.is_empty() {
                let blocked: usize = eh.iter().map(|(_, n)| *n).sum();
                println!("    the widening order OVER EMITTED CODE ({blocked} blocked):");
                for (feature, count) in eh.iter().take(20) {
                    println!(
                        "      {count:>7} ({:>5.1}%)  {feature}",
                        100.0 * *count as f64 / blocked as f64
                    );
                }
                if eh.len() > 20 {
                    println!("      … and {} more distinct features", eh.len() - 20);
                }
            }
            // The payoff metric's leading indicator, with the emitted census
            // beside it: these are the TUs whose remaining distance is small
            // enough to be worked directly.
            let near = report.near_match_tus(100);
            if !near.is_empty() {
                println!(
                    "    TUs within 100 blocked functions of matching: {} \
                     (blocked | emitted in-class/emitted | src)",
                    near.len()
                );
                for r in near.iter().take(60) {
                    let e = r.emit.get("emit-emitted").copied().unwrap_or(0);
                    let i = r.emit.get("emit-in-class").copied().unwrap_or(0);
                    println!(
                        "      {:>5} | {i:>4}/{e:<4} | {} [{}]",
                        r.fn_total - r.fn_in_class,
                        r.src,
                        r.class.label()
                    );
                }
                if near.len() > 60 {
                    println!("      … and {} more", near.len() - 60);
                }
            }
        }

        let hist = report.fn_blocker_histogram();
        if !hist.is_empty() {
            println!("  blocking features (the widening order):");
            let blocked: usize = hist.iter().map(|(_, n)| *n).sum();
            for (feature, count) in hist.iter().take(20) {
                println!(
                    "    {count:>7} ({:>5.1}%)  {feature}",
                    100.0 * *count as f64 / blocked as f64
                );
            }
            if hist.len() > 20 {
                println!("    … and {} more distinct features", hist.len() - 20);
            }
            // **The two axes do not share a vocabulary, and nothing else says so.**
            // These keys come from `mcall`'s walk, which does not decode `0x64`
            // (the by-value return's materialize) or `0x67` (virtual dispatch); the
            // control-flow axis below does, and spells them by name. So an
            // `op-0x64` row here and a `cf-…` row there are the *same construct
            // under two vocabularies*, and a ranking that adds or compares them is
            // comparing an unnamed byte with a named production. Renaming these
            // from the statement layer would be a census key change with no
            // production behind it — the rung that widens `mcall` renames them,
            // with the 1:1 proof (`docs/IL_DECODE_REACH.md` §11.2).
            if hist.iter().any(|(k, _)| k.contains("op-0x64") || k.contains("op-0x67")) {
                println!(
                    "    NOTE: `op-0x64`/`op-0x67` above spell hex because mcall's walk does \
                     not decode them; the control-flow axis below does. Do not rank across \
                     the two axes on those rows — different vocabularies."
                );
            }
        }
        // The D6 frame axis (`docs/IL_CALL_IN_EXPR.md` §18). The blocking-feature
        // histogram above ranks by *size*; this one says whether a row's lowering
        // can be local at all, and `calls-2plus` is the half that provably cannot
        // — two calls means LR is clobbered, so the body needs a frame.
        let [c0, c1, c2p] = report.frame_class_totals();
        let seen = c0 + c1 + c2p;
        if seen > 0 {
            println!("  frame class (CALL tokens per body — decode-only, §18):");
            for (label, n) in [("calls-0", c0), ("calls-1", c1), ("calls-2plus", c2p)] {
                println!(
                    "    {n:>7} ({:>5.1}%)  {label}",
                    100.0 * n as f64 / seen as f64
                );
            }
            let frames = report.fn_frame_histogram();
            for (key, count) in frames
                .iter()
                .filter(|(k, _)| k.starts_with("calls-2plus|"))
                .take(10)
            {
                println!("    {count:>7}           {key}");
            }
        }
        // The control-flow axis (roadmap #25/#61) — DECODE ONLY, and the sizing of
        // the block-IR restructure. `cflow-*` rows are bodies whose statement layer
        // decoded end to end, so their CFG is known; `cf-*` rows are where the
        // statement-layer decoder itself stopped, which is the residue of the
        // grammar and the next decode rung's own widening order.
        let (dec, undec) = report.cflow_decoded_totals();
        let cf_seen = dec + undec;
        if cf_seen > 0 {
            println!(
                "  control-flow class (statement layer, decode-only): {dec} of {cf_seen} bodies \
                 decoded end to end ({:.1}%)",
                100.0 * dec as f64 / cf_seen as f64
            );
            let cflow = report.fn_cflow_histogram();
            for (key, count) in cflow.iter().filter(|(k, _)| !k.contains('|')).take(14) {
                println!(
                    "    {count:>7} ({:>5.1}%)  {key}",
                    100.0 * *count as f64 / cf_seen as f64
                );
            }
            // …and what each decoded shape is worth **if it were lowered**: the
            // shape crossed with what else the body is blocked on. A row whose
            // census key is an `expr-*` feature is waiting on the expression layer
            // too; the `+expr-modeled` rows are the ones waiting on control flow
            // alone, and their total is the number this restructure is worth today.
            //
            // The `|IN-CLASS` / `|BLOCKED` population cross is excluded here and
            // rendered on its own below: it is not a "what else is this waiting
            // on" row, and letting it into this list would displace real ones
            // (`cflow-loop|BLOCKED` alone outranks eight of them).
            for (key, count) in cflow
                .iter()
                .filter(|(k, _)| {
                    k.contains('|')
                        && !k.starts_with("cflow-straight")
                        && !k.ends_with("|IN-CLASS")
                        && !k.ends_with("|BLOCKED")
                })
                .take(12)
            {
                println!("    {count:>7}           {key}");
            }
            // ---- the counterfactual, and the denominator it is a fraction of --
            let (res_mod, res_off) = report.cflow_residue_control();
            let (em_branchy, em_modeled) = report.cflow_emitted_counterfactual();
            let cf_bodies: usize = cflow
                .iter()
                .filter(|(k, _)| !k.contains('|') && k.ends_with("+expr-modeled"))
                .filter(|(k, _)| !k.starts_with("cflow-straight"))
                .map(|(_, n)| *n)
                .sum();
            let ic = res_mod + res_off;
            println!(
                "    CONTROL-FLOW COUNTERFACTUAL (board #1343) — what a block IR would convert \
                 BY ITSELF: {cf_bodies} bodies, {em_modeled} of the {em_branchy} blocked \
                 EMITTED functions a block IR must serve. Neither is a bound — both are a \
                 PROXY whose two-sided error the next two lines measure. Never quote one \
                 without them."
            );
            if ic > 0 {
                println!(
                    "    RESIDUE CONTROL (board #1344) — `CfResidue::Modeled` is a hand-written \
                     mirror of the port's class and NOTHING checked it against the port. It \
                     calls {res_off} of the {ic} bodies the port ACCEPTS off-class ({:.1}%), \
                     recognising only {res_mod}. NOT a gate and NOT an error: a residue LOOSER \
                     than the emitter would over-claim, which is worse. What it IS is the \
                     counterfactual's error term, and it was assumed rather than measured for \
                     eight days.",
                    100.0 * res_off as f64 / ic as f64
                );
                println!(
                    "    …and it errs BOTH WAYS, so `lower bound` is the wrong word for the \
                     line above: {} straight-line bodies are `+expr-modeled` and the port \
                     REFUSES them anyway. `Modeled` neither contains nor is contained in the \
                     class — it is a different predicate, and the counterfactual inherits both \
                     differences.",
                    report.cflow_residue_overclaim()
                );
            }
        }
        // The EH axis (`docs/EH_RECORDS.md` §9.4, §10) — DECODE ONLY, and the
        // sizing of the EH phase. `eh-state0` is the cheap side: a destructible
        // object is live but no call crosses it, so `maxState = 0` and no handler
        // prefix, no second `.pdata`, no `.rdata`, no funclet is emitted.
        // `eh-state1` is the whole of §1–§5.
        let ehh = report.fn_eh_histogram();
        let eh_seen: usize = ehh
            .iter()
            .filter(|(k, _)| !k.contains('|'))
            .map(|(_, n)| *n)
            .sum();
        let get = |k: &str| -> usize {
            ehh.iter().find(|(a, _)| a == k).map(|(_, n)| *n).unwrap_or(0)
        };
        if eh_seen > 0 {
            let need = get("eh-state1");
            println!(
                "  EH class (maxState, decode-only): {need} of {eh_seen} bodies have maxState >= 1 \
                 and need the whole EH record ({:.1}%)",
                100.0 * need as f64 / eh_seen as f64
            );
            // Totals over EVERY function, with the blocked subtotal beside each —
            // the two are different populations and a table that prints only one
            // of them invites the reader to rank the other by mistake.
            for (key, count) in ehh.iter().filter(|(k, _)| !k.contains('|')) {
                println!(
                    "    {count:>7} ({:>5.1}%)  {key:<12}  blocked {:>7}",
                    100.0 * *count as f64 / eh_seen as f64,
                    get(&format!("{key}|BLOCKED"))
                );
            }
            // The BLOCKED cross: what a rung on this side would actually have to
            // widen. In-class rows are excluded here by construction — they are
            // printed separately below as the control group they are.
            println!("    blocked residue, by census key (accepted shapes excluded):");
            for (key, count) in ehh
                .iter()
                .filter(|(k, _)| k.contains("|BLOCKED|") && !k.starts_with("eh-none|"))
                .take(16)
            {
                println!("    {count:>7}           {key}");
            }
            // …and the control group, labelled as one. These are functions the
            // port ACCEPTS. They are here because the cheap side of the boundary
            // is exactly where the accepted `empty-dtor-*` shapes live, so one of
            // them reading anything but `eh-state0` indicts the axis.
            println!("    in-class control group (ACCEPTED functions, never a rung):");
            for (key, count) in ehh
                .iter()
                .filter(|(k, _)| k.contains("|INCLASS|") && !k.starts_with("eh-none|"))
                .take(10)
            {
                println!("    {count:>7}           {key}");
            }
            // The migration cross against the refuted statement-count axis.
            println!("    maxState x statement-count (the refuted predicate, for reconciliation):");
            for (key, count) in ehh.iter().filter(|(k, _)| k.starts_with("eh-migrate|")) {
                if key.ends_with("|eh-none") && key.starts_with("eh-migrate|eh-none") {
                    continue;
                }
                println!("    {count:>7}           {key}");
            }
        }
        // The two DISPATCH axes — DECODE ONLY, and the answer to a question no
        // census key can be asked: **which recognizer looked at this body, and
        // where inside it did the refusal happen.**
        //
        // The blocking-feature histogram above names the *construct*. It cannot
        // say whether a widening could reach the body at all: a member call that
        // is a store's right-hand side or a plain call's argument gets the same
        // `expr-call-in-expr-recv-*` key as one that is the whole body, and only
        // the last of the three ever enters a member-call production. Nor can it
        // say whether a row is a missing construct or a **private limit inside a
        // recognizer that already ships** — which has been the answer six rungs
        // running.
        // **THE GRAMMAR-COMPLETENESS AXIS** (roadmap §9.11 / §9.14) — is anything
        // hiding behind a row, or is its count directly a widening estimate?
        //
        // Printed as its own axis, and printed WHOLE, because the fact has two
        // producers that write it into two different halves of the census key
        // (`-whole`/`-more` and `:eof`/`:mid`). WR1 moved 39,967 functions from
        // one encoding to the other; every ranking table built by grepping the
        // key for `-whole` has under-counted that family by 18,931 since. The
        // residue row `complete-none` is printed rather than suppressed for the
        // same reason `prod-entered-untagged` is: an axis whose residue is
        // invisible cannot be audited, and "reported as nothing" and "measured
        // at zero" must never share a rendering.
        let comph = report.fn_complete_histogram();
        let comp_bare: Vec<_> = comph.iter().filter(|(k, _)| !k.contains('|')).collect();
        let comp_seen: usize = comp_bare.iter().map(|(_, n)| *n).sum();
        if comp_seen > 0 {
            println!(
                "  grammar completeness (decode-only): {comp_seen} of {fn_total} bodies read"
            );
            if comp_seen != fn_total {
                println!(
                    "    AXIS UNDER-REPORTS: {} bodies have no completeness reading",
                    fn_total - comp_seen.min(fn_total)
                );
            }
            let whole: usize = comph
                .iter()
                .filter(|(k, _)| k.ends_with("|BLOCKED") && k.starts_with("complete-whole"))
                .map(|(_, n)| *n)
                .sum();
            for (key, count) in &comp_bare {
                let blocked = comph
                    .iter()
                    .find(|(a, _)| *a == format!("{key}|BLOCKED"))
                    .map(|(_, n)| *n)
                    .unwrap_or(0);
                println!(
                    "    {count:>7} ({:>5.1}%)  {key:<30}  blocked {blocked:>7}",
                    100.0 * *count as f64 / comp_seen as f64
                );
            }
            // The join the roadmap actually ranks by, across BOTH producers —
            // the number §9.13 had to re-derive by hand.
            println!(
                "    grammar-COMPLETE and blocked, both producers summed: {whole}"
            );
        }

        let disph = report.fn_dispatch_histogram();
        let prodh = report.fn_prod_histogram();
        let (disp_seen, prod_seen) = report.dispatch_axis_totals();
        if disp_seen > 0 {
            // Both axes must sum to the census. Stated as an equality rather than
            // left implicit: a short count means bodies took an arm nobody tagged,
            // and the resulting table would under-report every row in it while
            // looking perfectly well formed.
            println!(
                "  body dispatch (which recognizer claimed the body, decode-only): {disp_seen} of \
                 {fn_total} bodies tagged on the dispatch axis, {prod_seen} on the production axis"
            );
            if disp_seen != fn_total || prod_seen != fn_total {
                println!(
                    "    AXIS UNDER-REPORTS: {} / {} bodies are missing a tag — every row below \
                     is a lower bound",
                    fn_total - disp_seen.min(fn_total),
                    fn_total - prod_seen.min(fn_total)
                );
            }
            // Every bare row, untruncated. A dispatch arm that is dropped for
            // being small reads as an arm no body takes, and the whole reason
            // this axis exists is that "reported as nothing" and "measured at
            // zero" had become the same rendering.
            for (key, count) in disph.iter().filter(|(k, _)| !k.contains('|')) {
                let blocked = disph
                    .iter()
                    .find(|(a, _)| *a == format!("{key}|BLOCKED"))
                    .map(|(_, n)| *n)
                    .unwrap_or(0);
                println!(
                    "    {count:>7} ({:>5.1}%)  {key:<32}  blocked {blocked:>7}",
                    100.0 * *count as f64 / disp_seen as f64
                );
            }
            // The BLOCKED cross for the arms that cannot reach a member-call
            // production at all. **This is the row set that says a member-call
            // widening cannot serve these bodies** — they are the same construct
            // in a statement position the productions never see.
            //
            // Selected by EXCLUDING the two arms that do reach a production
            // (`disp-member-call`, and `disp-assign` which is only ever arrived at
            // by falling through one), rather than by listing the arms that do
            // not: a list of names silently drops any arm added later, and an arm
            // that vanishes from this table is a population reported as zero.
            println!("    blocked residue of the arms that never enter a member-call production:");
            for (key, count) in disph
                .iter()
                .filter(|(k, _)| {
                    k.contains("|BLOCKED|")
                        && !k.starts_with("disp-assign|")
                        && !k.starts_with("disp-member-call|")
                })
                .take(24)
            {
                println!("    {count:>7}           {key}");
            }
            // The production axis. `prod-entered-untagged` is the tag-coverage
            // residue and is printed as a number on every scan: it is what the 37
            // tag sites in `body::shapes::mcall_{tail,chain,cmp}` have left to
            // explain, and inferring it from missing rows is exactly the mistake
            // this axis was built to stop.
            let residue = report.prod_untagged_residue();
            println!(
                "  member-call production first blocker (decode-only): {residue} bodies entered a \
                 production, declined, and reached NO tagged bail — the tag-coverage residue \
                 (target 0)"
            );
            for (key, count) in prodh.iter().filter(|(k, _)| !k.contains('|')) {
                let blocked = prodh
                    .iter()
                    .find(|(a, _)| *a == format!("{key}|BLOCKED"))
                    .map(|(_, n)| *n)
                    .unwrap_or(0);
                println!(
                    "    {count:>7} ({:>5.1}%)  {key:<32}  blocked {blocked:>7}",
                    100.0 * *count as f64 / prod_seen.max(1) as f64
                );
            }
            for (key, count) in prodh
                .iter()
                .filter(|(k, _)| k.contains("|BLOCKED|"))
                .take(20)
            {
                println!("    {count:>7}           {key}");
            }
        }
    }

    for (class, title) in [
        (TuClass::CaptureFail, "top capture-fail reasons"),
        (TuClass::VocabGap, "top vocab gaps"),
        (TuClass::CodegenGap, "top codegen gaps"),
        (TuClass::PortError, "top port errors"),
        (TuClass::Mismatch, "mismatches"),
    ] {
        let reasons = report.top_reasons(class);
        if reasons.is_empty() {
            continue;
        }
        println!("\n  {title}:");
        for (reason, count) in reasons.iter().take(10) {
            println!("    {count:>5} x {reason}");
        }
        if reasons.len() > 10 {
            println!("    … and {} more distinct reasons", reasons.len() - 10);
        }
    }

    // **W-VEC (#2500) — the GATE's own first refusal, over the whole
    // `vocab-gap` bucket.**
    //
    // `top vocab gaps` above prints one string, *"il function decode failed"*,
    // for 851 of 878 TUs, because that is the only `reason` the scan sets on
    // that path. This is the actionable decomposition `IlBundle::decode_causes`
    // has produced since lane `w-vocab` and that nothing in this crate called:
    // which of the eleven gates `functions()` **stops** on, and — separately —
    // how many TUs each cause fires on *anywhere*, which is what a lane owes
    // after repairing the first one.
    //
    // Printed as a HAND-VERIFIABLE pair rather than a single ranking, because
    // the two answer different questions and the difference is the whole point:
    // a repair of the top FIRST cause converts nothing if the same TUs are also
    // in the ALSO column for four more.
    let mut first: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    let mut anywhere: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for r in &report.results {
        if let Some(c) = &r.gate_cause {
            *first.entry(c.as_str()).or_insert(0) += 1;
        }
        for c in &r.gate_causes {
            *anywhere.entry(c.as_str()).or_insert(0) += 1;
        }
    }
    if !first.is_empty() {
        let total: usize = first.values().sum();
        println!(
            "\n  gate FIRST refusal, over the {total} TUs `IlBundle::decodes()` \
             rejects (c2_il::func::diag; a first cause is what a repair is \
             guaranteed to move, the ALSO column is what it would still owe):"
        );
        let mut rows: Vec<(&&str, &usize)> = first.iter().collect();
        rows.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
        for (cause, count) in rows {
            println!(
                "    {count:>5} x {cause:<34} (also fires on {} TUs total)",
                anywhere.get(*cause).copied().unwrap_or(0)
            );
        }
    }

    // **W-PHASE7B — IS THE BINDING THIS TU FAILED SATISFIABLE AT ALL?**
    //
    // Every published binding instrument is about the READER: `fn_names` is the
    // census's loose scan, `emit-bound` is `EmitBinding`, `gate_cause` is the
    // clause `gl_defined_names_framed` stopped on. Four fields have been used to
    // answer `CEILING.md` §11.4 item 8 and three were wrong, and all four have
    // the same blind spot — they report on a walk, so their answer always looks
    // like a repair address.
    //
    // `Bindings::per_record` needs the `.gl` records 1:1 with the `.ex`
    // segments. A segment whose body-start offset `.gl` does not spell **at
    // all** cannot be bound by any framing anyone writes, so this row separates
    // "the reader stopped early" from "there is nothing to stop at". It is the
    // one column on this page that can retire a repair rather than locate one.
    let mut short: Vec<(&str, usize, usize)> = Vec::new();
    let mut whole = 0usize;
    let mut absent = 0usize;
    for r in &report.results {
        match r.gl_body_starts {
            None => absent += 1,
            Some((p, t)) if p < t => short.push((r.src.as_str(), p, t)),
            Some(_) => whole += 1,
        }
    }
    if !short.is_empty() || whole > 0 {
        let segs_short: usize = short.iter().map(|&(_, p, t)| t - p).sum();
        println!(
            "\n  `.gl` BODY-START COVERAGE — can `Bindings::per_record` bind this TU at all?\n    \
             {whole:>5} TUs where `.gl` spells a body-start for EVERY `.ex` segment\n    \
             {:>5} TUs where it does not — {segs_short} segments across them can bind to NO \
             record, whatever the framing\n    \
             {absent:>5} TUs with no `.ex` or no `.gl` (the field is null, which is not zero)",
            short.len()
        );
        short.sort_by(|a, b| (b.2 - b.1).cmp(&(a.2 - a.1)).then(a.0.cmp(b.0)));
        for (src, p, t) in short.iter().take(10) {
            println!("      {p:>6} of {t:<6} {src}");
        }
        if short.len() > 10 {
            println!("      … and {} more", short.len() - 10);
        }
    }

    // **W-SELBIND — the SELECTIVE contract's denominator, printed beside the
    // coverage block because reading one as the other is the specific error this
    // lane was commissioned on.**
    //
    // The block above asks whether a segment's body-start offset is SPELLED in
    // `.gl`; its own reader is a deliberate over-count. This asks whether a `.gl`
    // record NAMES it, which is what a binding needs, and on `vec.cpp` the two
    // read 373 and 36. The join at the end is the one w-phase7b §10 item 3 left
    // open and it is the number that decides whether selectivity has a
    // denominator at all: a TU where some symbol c2 EMITTED carries no record can
    // never be bound selectively however good the accounting gets, because the
    // port would emit an obj missing that function.
    let sel_1to1 = report.bind_total("selbind-one-to-one-tus");
    let sel_sel = report.bind_total("selbind-selective-tus");
    let sel_total = report.bind_total("selbind-total-tus");
    let emit_tus = report.bind_total("selbind-emit-tus");
    if sel_1to1 + sel_sel > 0 {
        println!(
            "\n  SELECTIVE BINDING (`Bindings::selective`) — how many TUs can bind a SUBSET, \
             and is the subset SOUND?\n    \
             {sel_1to1:>5} TUs whose records are 1:1 with the segments (the incumbent contract; \
             `per_record` is this case)\n    \
             {sel_sel:>5} TUs where a record NAMES some but not all segments — the selective \
             population\n    \
             {sel_total:>5} of those pass the TOTALITY clause (every `.gl` run that could be a \
             COFF symbol name is claimed), i.e. MAY bind today\n    \
             {:>5} blocked by an unclaimed MANGLED run · {:>5} blocked by an unclaimed run that \
             FITS the 8-byte inline name field (board #1721's hole, and the shape\n          \
             `work/w-small/probe/l1_counterexample.cpp` measured as `Port=Mismatch @ offset 8`)",
            report.bind_total("selbind-blocked-mangled-tus"),
            report.bind_total("selbind-blocked-inline-fit-tus"),
        );
        // **W-FRAME783 — this block used to end with *"the difference between
        // those two is the price of the unshipped frame relaxation, and it is
        // the only thing standing between the two numbers."* The relaxation is
        // SHIPPED and the gate's number did not move, so the sentence is
        // replaced by the decomposition that says why.**
        println!(
            "    IS `emitted ⊆ named`?  {} of {} emitted symbols carry a `.gl` record the GATE \
             BINDS ({} named by a walk-free scan at the same framing, {} at the window-free one).\n      \
             TUs with any emitted symbol whose emit set is ENTIRELY named, {emit_tus} in the \
             denominator — the CEILING on a selective binding, decomposed:\n        \
             {:>5}  scan, INCUMBENT framing  (`codec::gl_offset_framed`, no walk)\n        \
             {:>5}  scan, SHIPPED framing    (#2783 relaxed + the 16 MB offset bound, no walk)\n        \
             {:>5}  scan, window-free framing (board #2783 as filed — includes 551 offsets that \
             are not `.ex` split points)\n        \
             {:>5}  the GATE's own binding walk, at the shipped framing — six stop clauses, any \
             one of which empties the whole TU\n      \
             Read the last two rows against each other: what separates the gate from the \
             instrument is the WALK, not the framing (board #2860).",
            report.bind_total("selbind-emitted-named-gate"),
            report.bind_total("selbind-emitted"),
            report.bind_total("selbind-emitted-named-scan-precise"),
            report.bind_total("selbind-emitted-named-wide"),
            report.bind_total("selbind-emit-subset-scan-narrow-tus"),
            report.bind_total("selbind-emit-subset-scan-precise-tus"),
            report.bind_total("selbind-emit-subset-wide-tus"),
            report.bind_total("selbind-emit-subset-gate-tus"),
        );
    }

    let mismatches = report.count(TuClass::Mismatch);
    if mismatches > 0 || diverged > 0 || report.cache.poisoned > 0 {
        eprintln!(
            "\nCORRECTNESS SIGNAL: {mismatches} mismatching TU(s), {diverged} replay \
             divergence(s), {} poisoned cache entr(ies)",
            report.cache.poisoned
        );
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
