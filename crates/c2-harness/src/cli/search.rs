//! T-A IL-space search leaves: `search solve|eval|from-retrieval|from-lifter`
//! and the option helpers the four of them share. The `search` group dispatcher
//! stays in `main.rs`.

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use c2_harness::toolchain_gate::{toolchain_ready, Cap};
use crate::{Args, Arity, Spec};
use crate::cli::util::{first_line, Scratch};

// ---------------------------------------------------------------------------
// T-A IL-space search prototype
// ---------------------------------------------------------------------------

use c2_harness::search::{self, Budget, MoveSet, Perturb};

/// The default solvable-instance roster: straight-line int fixtures that carry
/// literals and/or arithmetic terms (the move set's sites). C++-only; each is
/// captured fresh (no committed IL/obj).
const SEARCH_FIXTURES: &[&str] = &[
    "mvp_edit_addk.cpp",
    "mvp_lit.cpp",
    "mvp_wide.cpp",
    "mvp_add3.cpp",
    "mvp_sub.cpp",
    "mvp_two.cpp",
];

/// The move set. The `_ =>` arm used to map every unrecognised spelling to the
/// full set — and `search eval` then ECHOED the raw string, so `--moves lenght`
/// printed `moves=lenght` in its header while running the full moveset. A report
/// that names a configuration it did not run is worse than one that names none.
/// `one_of` is checked at parse time, so the echo and the behaviour cannot part.
fn search_moveset(args: &Args) -> Result<MoveSet, ExitCode> {
    let mut m = match args.one_of("--moves", &["full", "length"])? {
        Some("length") => MoveSet::length_only(),
        _ => MoveSet::default(),
    };
    // On the real obj-judged path, widen/narrow is obj-INVISIBLE (P0.6a A: c2
    // re-optimizes a re-widthed literal to byte-identical code), so it can never
    // reach a new obj — it only floods the beam with gradient-tied duplicate
    // models that crowd out productive (structure-changing) moves. Drop it from
    // the search moveset unless explicitly re-enabled. (The mock-scorer unit tests
    // keep it via `MoveSet::default()`, where it IS `.ex`-visible.)
    if !args.has("--keep-widen") {
        m.widen_narrow = false;
    }
    Ok(m)
}

fn search_budget(args: &Args) -> Result<Budget, ExitCode> {
    let mut b = Budget::default();
    if let Some(v) = args.num("--steps")? {
        b.max_steps = v;
    }
    if let Some(v) = args.num("--compiles")? {
        b.max_compiles = v;
    }
    if let Some(v) = args.num("--beam")? {
        b.beam_width = v;
    }
    Ok(b)
}

fn search_perturbs(args: &Args) -> Result<Vec<(Perturb, usize)>, ExitCode> {
    // The obj-changing families at d=1, plus AddTerm at d=2 (a gradient-guided
    // two-move recovery) when --d 2 is requested. WidenLit is obj-invisible on
    // the real path (P0.6a A), so it is not in the roster.
    let d: usize = args.num("--d")?.unwrap_or(1);
    let mut v = vec![
        (Perturb::AddTerm, 1),
        (Perturb::LitNudge, 1),
        (Perturb::DropTerm, 1),
    ];
    if d >= 2 {
        v.push((Perturb::AddTerm, 2));
    }
    if d >= 3 {
        v.push((Perturb::AddTerm, 3));
    }
    Ok(v)
}

fn search_timeout(args: &Args) -> Result<Duration, ExitCode> {
    Ok(Duration::from_secs(args.num("--timeout")?.unwrap_or(60)))
}

/// The options every `search` subcommand shares.
const SEARCH_COMMON: &[(&str, Arity)] = &[
    ("--moves", Arity::Value),
    ("--keep-widen", Arity::Flag),
    ("--steps", Arity::Value),
    ("--compiles", Arity::Value),
    ("--beam", Arity::Value),
    ("--timeout", Arity::Value),
];

/// `SEARCH_COMMON` plus the subcommand's own, as one `const` list.
const fn search_spec(cmd: &'static str, opts: &'static [(&'static str, Arity)]) -> Spec {
    Spec { cmd, opts, requires: &[], max_positionals: 1 }
}

/// **`--d` is not here on purpose.** The top-level `search` usage advertised it
/// for every subcommand, but `solve` hardcodes `Perturb::AddTerm, 1` and never
/// calls `search_perturbs`, so `search solve <cpp> --d 3` accepted the option and
/// ran d=1. Refusing it is the honest reading of "this subcommand does not take
/// it"; the ladder lives on `search eval`.
static SEARCH_SOLVE_SPEC: Spec = search_spec("search solve", SEARCH_COMMON);

pub(crate) fn cmd_search_solve(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&SEARCH_SOLVE_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let Some(cpp) = args.path_positional() else {
        eprintln!("usage: c2rs search solve <cpp> [--moves full|length]");
        return ExitCode::from(2);
    };
    let (moves, budget, timeout) = match (|| {
        Ok((search_moveset(&args)?, search_budget(&args)?, search_timeout(&args)?))
    })() {
        Ok(v) => v,
        Err(c) => return c,
    };
    let Some(tc) = args.toolchain() else {
        return ExitCode::SUCCESS;
    };
    if let Some(code) = toolchain_ready(&tc, &[Cap::Strace, Cap::Mingw], "needed for replay") {
        return code;
    }
    let w = Scratch::new("search-solve");
    // An inserted redundant term is the cleanest obj-changing single demo.
    let r = search::solve_instance(&tc, &cpp, Perturb::AddTerm, 1, &moves, &budget, &w, timeout);
    let code = match (&r.outcome, &r.error) {
        (Some(o), _) => {
            println!(
                "{} [{}] -> solved={} steps={} compiles={} best_fuzzy={:.4} ({:?})",
                r.fixture,
                r.perturb.label(),
                o.solved,
                o.steps,
                o.compiles,
                o.best_fuzzy,
                o.reason
            );
            if !o.path.is_empty() {
                println!("  path: {}", o.path.join(" -> "));
            }
            if o.solved {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        (None, Some(e)) => {
            eprintln!("{} -> instance error: {e}", r.fixture);
            ExitCode::FAILURE
        }
        (None, None) => {
            println!("{} -> no perturbation site (skipped)", r.fixture);
            ExitCode::SUCCESS
        }
    };
    code
}

static SEARCH_EVAL_SPEC: Spec = Spec {
    cmd: "search eval",
    opts: &[
        ("--moves", Arity::Value),
        ("--keep-widen", Arity::Flag),
        ("--steps", Arity::Value),
        ("--compiles", Arity::Value),
        ("--beam", Arity::Value),
        ("--timeout", Arity::Value),
        ("--d", Arity::Value),
    ],
    requires: &[],
    max_positionals: 0,
};

pub(crate) fn cmd_search_eval(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&SEARCH_EVAL_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let (moves, budget, perturbs, timeout) = match (|| {
        Ok((
            search_moveset(&args)?,
            search_budget(&args)?,
            search_perturbs(&args)?,
            search_timeout(&args)?,
        ))
    })() {
        Ok(v) => v,
        Err(c) => return c,
    };
    let Some(tc) = args.toolchain() else {
        return ExitCode::SUCCESS;
    };
    if let Some(code) = toolchain_ready(&tc, &[Cap::Strace, Cap::Mingw], "needed for replay") {
        return code;
    }
    let fixtures: Vec<PathBuf> = SEARCH_FIXTURES
        .iter()
        .map(|n| c2_harness::fixtures_dir().join(n))
        .collect();

    let w = Scratch::new("search-eval");
    println!(
        "T-A IL-space solve-rate: {} fixtures x {} perturbation families, moves={}, budget steps={} compiles={}",
        fixtures.len(),
        perturbs.len(),
        args.get("--moves").unwrap_or("full"),
        budget.max_steps,
        budget.max_compiles,
    );
    let report = search::solve_rate(&tc, &fixtures, &perturbs, &moves, &budget, &w, timeout);
    for r in &report.instances {
        let tag = format!("{} d{}", r.perturb.label(), r.d);
        match (&r.outcome, &r.error) {
            (Some(o), _) => println!(
                "  {:<20} {:<13} solved={} steps={} compiles={:>3} fuzzy={:.4} {:?}",
                r.fixture, tag, o.solved, o.steps, o.compiles, o.best_fuzzy, o.reason
            ),
            (None, Some(e)) => {
                println!("  {:<20} {:<13} ERROR {}", r.fixture, tag, first_line(e))
            }
            (None, None) => println!("  {:<20} {:<13} skipped (no site)", r.fixture, tag),
        }
    }
    println!("\nsolve-rate by family (attempted excludes no-site/errored):");
    for ((kind, d), (attempted, solved, mean)) in report.by_family() {
        let pct = if attempted > 0 {
            solved as f64 / attempted as f64 * 100.0
        } else {
            0.0
        };
        println!(
            "  {:<10} d{}  {}/{} = {:>5.1}%   mean compiles-to-solve: {:.1}",
            kind.label(),
            d,
            solved,
            attempted,
            pct,
            mean
        );
    }
    let (attempted, solved, mean) = report.tally();
    let pct = if attempted > 0 {
        solved as f64 / attempted as f64 * 100.0
    } else {
        0.0
    };
    println!(
        "  {:<14} {}/{} = {:>5.1}%   mean compiles-to-solve: {:.1}",
        "OVERALL", solved, attempted, pct, mean
    );
    if attempted > 0 && solved == attempted {
        ExitCode::SUCCESS
    } else if solved > 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

static SEARCH_FROM_RETRIEVAL_SPEC: Spec = Spec {
    cmd: "search from-retrieval",
    opts: &[
        ("--moves", Arity::Value),
        ("--keep-widen", Arity::Flag),
        ("--steps", Arity::Value),
        ("--compiles", Arity::Value),
        ("--beam", Arity::Value),
        ("--timeout", Arity::Value),
        ("--sample", Arity::Value),
        ("--multi", Arity::Value),
        ("--select-seed", Arity::Value),
    ],
    requires: &[],
    max_positionals: 1,
};

pub(crate) fn cmd_search_from_retrieval(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&SEARCH_FROM_RETRIEVAL_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let Some(dir) = args.path_positional() else {
        eprintln!(
            "usage: c2rs search from-retrieval <corpus-dir> [--sample N] [--multi N] [--select-seed N] [--steps N] [--compiles N] [--beam K] [--timeout SECS]"
        );
        return ExitCode::from(2);
    };
    // Real-path moveset: drop the obj-invisible widen/narrow (P0.6a A) unless
    // explicitly kept — same rule as `search eval`.
    let (moves, cfg) = match (|| {
        let moves = search_moveset(&args)?;
        let mut cfg = search::FromSeedConfig::default();
        if let Some(v) = args.num("--sample")? {
            cfg.sample = v;
        }
        if let Some(v) = args.num("--multi")? {
            cfg.multi = v;
        }
        if let Some(v) = args.num("--select-seed")? {
            cfg.select_seed = v;
        }
        if let Some(v) = args.num("--steps")? {
            cfg.budget.max_steps = v;
        }
        if let Some(v) = args.num("--compiles")? {
            cfg.budget.max_compiles = v;
        }
        if let Some(v) = args.num("--beam")? {
            cfg.budget.beam_width = v;
        }
        if let Some(v) = args.num::<u64>("--timeout")? {
            cfg.timeout = Duration::from_secs(v);
        }
        Ok((moves, cfg))
    })() {
        Ok(v) => v,
        Err(c) => return c,
    };
    let Some(tc) = args.toolchain() else {
        return ExitCode::SUCCESS;
    };
    if let Some(code) = toolchain_ready(&tc, &[Cap::Strace, Cap::Mingw], "needed for replay") {
        return code;
    }

    let w = Scratch::new("search-from-retrieval");
    println!(
        "T-A from-unrelated-seed: sample={} (multi={}) select-seed={} budget steps={} compiles={} beam={} timeout={}s",
        cfg.sample,
        cfg.multi,
        cfg.select_seed,
        cfg.budget.max_steps,
        cfg.budget.max_compiles,
        cfg.budget.beam_width,
        cfg.timeout.as_secs(),
    );
    let report = match search::from_retrieval_eval(&tc, &dir, &moves, &cfg, &w) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("from-retrieval eval failed: {e}");
            return ExitCode::FAILURE;
        }
    };

    println!("  corpus items: {}", report.n_items);
    println!("  per-target (target[fns] <- seed[fns] : class):");
    for r in &report.records {
        let seed = match (&r.seed_id, r.seed_fns) {
            (Some(id), Some(f)) => format!("{id}[{f}fns]"),
            (Some(id), None) => format!("{id}[?]"),
            _ => "<none>".to_string(),
        };
        let mut line = format!(
            "    {}[{}fns] <- {:<14} {:<17}",
            r.target_id,
            r.target_fns,
            seed,
            r.class.label()
        );
        if let Some(o) = &r.outcome {
            line.push_str(&format!(
                " solved={} steps={} compiles={} fuzzy={:.4} {:?}",
                o.solved, o.steps, o.compiles, o.best_fuzzy, o.reason
            ));
        } else if !r.detail.is_empty() {
            line.push_str(&format!(" ({})", r.detail));
        }
        println!("{line}");
        if let Some(w2) = &r.outcome_wholetext {
            let p = r.outcome.as_ref().map(|o| o.solved).unwrap_or(false);
            println!(
                "        per-fn vs whole-.text: per-fn solved={} fuzzy={:.4} | whole solved={} fuzzy={:.4}",
                p,
                r.outcome.as_ref().map(|o| o.best_fuzzy).unwrap_or(0.0),
                w2.solved,
                w2.best_fuzzy,
            );
        }
    }

    println!("\n  failure taxonomy:");
    for (class, n) in report.class_counts() {
        println!("    {:<17} {}", class.label(), n);
    }
    let (searched, solved) = report.search_tally();
    let pct = if searched > 0 {
        solved as f64 / searched as f64 * 100.0
    } else {
        0.0
    };
    println!(
        "\n  HEADLINE: from-unrelated-seed solve-rate = {solved}/{searched} = {pct:.1}% (searched = in-scope, non-trivial)"
    );
    ExitCode::SUCCESS
}

static SEARCH_FROM_LIFTER_SPEC: Spec = Spec {
    cmd: "search from-lifter",
    opts: &[
        ("--moves", Arity::Value),
        ("--keep-widen", Arity::Flag),
        ("--steps", Arity::Value),
        ("--compiles", Arity::Value),
        ("--beam", Arity::Value),
        ("--timeout", Arity::Value),
        ("--gens", Arity::Value),
        ("--k", Arity::Value),
        ("--limit", Arity::Value),
    ],
    requires: &[],
    max_positionals: 1,
};

pub(crate) fn cmd_search_from_lifter(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&SEARCH_FROM_LIFTER_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let Some(dir) = args.path_positional() else {
        eprintln!(
            "usage: c2rs search from-lifter <corpus-dir> --gens <jsonl> [--k K] [--limit N] [--steps N] [--compiles N] [--beam K] [--timeout SECS]"
        );
        return ExitCode::from(2);
    };
    let Some(gens_path) = args.path("--gens") else {
        eprintln!("from-lifter: --gens <jsonl> required (rows {{\"id\",\"generations\":[...]}})");
        return ExitCode::from(2);
    };
    let (k, limit, moves, budget, timeout) = match (|| {
        let k: usize = args.num("--k")?.unwrap_or(5);
        let limit: usize = args.num("--limit")?.unwrap_or(0);
        let moves = search_moveset(&args)?;
        let mut budget = search_budget(&args)?;
        // Bounded per-generation defaults (many generations x targets share one
        // CPU). The guard is PRESENCE, and that used to matter: with
        // `.parse().ok()` swallowing a bad value, `--compiles abc` left
        // `Budget::default()`'s 400 instead of this bounded 200 — a typo doubled
        // the compile budget in silence. `num` refuses first, so presence and
        // parse-success now coincide.
        if args.get("--steps").is_none() {
            budget.max_steps = 8;
        }
        if args.get("--compiles").is_none() {
            budget.max_compiles = 200;
        }
        if args.get("--beam").is_none() {
            budget.beam_width = 4;
        }
        let timeout = Duration::from_secs(args.num::<u64>("--timeout")?.unwrap_or(25));
        Ok((k, limit, moves, budget, timeout))
    })() {
        Ok(v) => v,
        Err(c) => return c,
    };
    let Some(tc) = args.toolchain() else {
        return ExitCode::SUCCESS;
    };
    if let Some(code) = toolchain_ready(&tc, &[Cap::Strace, Cap::Mingw], "needed for replay") {
        return code;
    }

    let gens = match search::load_lifter_gens(&gens_path) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("from-lifter: reading {}: {e}", gens_path.display());
            return ExitCode::FAILURE;
        }
    };
    println!(
        "T-B lifter byte-exact eval: gens={} targets, k={}, limit={}, budget steps={} compiles={} beam={} timeout={}s",
        gens.len(),
        k,
        limit,
        budget.max_steps,
        budget.max_compiles,
        budget.beam_width,
        timeout.as_secs(),
    );

    let w = Scratch::new("search-from-lifter");
    let report =
        match search::from_lifter_eval(&tc, &dir, &gens, k, limit, &moves, &budget, timeout, &w) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("from-lifter eval failed: {e}");
                return ExitCode::FAILURE;
            }
        };

    println!("  corpus ok-rows: {}", report.n_items);
    println!("  per-target (id[fns] class : detail):");
    for r in &report.records {
        let slot = match r.solved_slot {
            Some(s) => format!("slot{s}"),
            None => "-".to_string(),
        };
        println!(
            "    {:<9}[{}fns] cap={}/{} {:<10} {:<6} {}",
            r.target_id, r.target_fns, r.captured, r.k, r.class.label(), slot, r.detail
        );
    }

    let (attempted, pass1, passk) = report.tally();
    let no_compile = report
        .records
        .iter()
        .filter(|r| r.class == search::LifterClass::NoCompile)
        .count();
    let errors = report
        .records
        .iter()
        .filter(|r| r.class == search::LifterClass::Error)
        .count();
    let p1 = if attempted > 0 {
        pass1 as f64 / attempted as f64 * 100.0
    } else {
        0.0
    };
    let pk = if attempted > 0 {
        passk as f64 / attempted as f64 * 100.0
    } else {
        0.0
    };
    println!(
        "\n  attempted(searched)={attempted} no-compile-targets={no_compile} target-errors={errors}"
    );
    println!("  P1.3 retrieval control floor: 9.6% pass@1");
    println!(
        "  HEADLINE: lifter byte-exact pass@1 = {pass1}/{attempted} = {p1:.1}%   pass@{k} = {passk}/{attempted} = {pk:.1}%"
    );
    ExitCode::SUCCESS
}
