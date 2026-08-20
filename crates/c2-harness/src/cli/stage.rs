//! **The stage oracle's instrument** — `c2rs stage {counts,snap,determinism,neutrality}`.
//!
//! It lives in the workspace and not in `scripts/` because its output is
//! quoted as evidence (#1406): anything whose number appears in a rung doc has
//! to run under `cargo test` or `scripts/gate.sh`.
//!
//! Every subcommand degrades with `SKIP: toolchain absent` and never panics.
//!
//! # The keys this prints, and which one is a REQUIRED ZERO
//!
//! | key | meaning |
//! |---|---|
//! | `stage-tap-obj-differs` | **REQUIRED 0.** Fixtures whose obj changed when the tap was armed. Nonzero ⇒ the oracle is grading a different compiler than the judge does ⇒ decline |
//! | `stage-tap-graded` | the denominator of the line above — published, because a required-zero over an empty population is free |
//! | `stage-sites-armed` / `stage-sites-refused` | arming, per run |
//! | `stage-snap-runs` / `stage-snap-distinct-max` / `stage-snap-unstable-tus` | determinism |
//! | `stage-snap-tuples` / `stage-snap-regions` | **content.** A structurally deterministic EMPTY snapshot passes every other criterion here trivially |
//! | `stage-snap-walk-refusals` | payload truncation. Never folded into the tuple count |
//!
//! # The standing bound
//!
//! None of these keys gates an emit and none appears in a refusal predicate.
//! `mismatch 0` remains the judge's alarm; a stage snapshot is a development
//! instrument.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use c2_obj::{ObjDiff, ObjImage};
use c2_reference::stage::{TapReport, OPT_GATED_SITES, STAGE_SITES};
use c2_reference::Toolchain;

use crate::cli::util::Scratch;
use crate::{Args, Arity, Spec};

static STAGE_SPEC: Spec = Spec::new(
    "stage",
    &[
        ("--fixtures", Arity::Value),
        ("--runs", Arity::Value),
        ("--limit", Arity::Value),
        ("--payload", Arity::Flag),
        ("--flag", Arity::Repeated),
    ],
)
.positionals(1);

/// The workload's own optimization profile — the one the 878 TUs compile at.
fn default_flags() -> Vec<String> {
    ["/O1", "/Oi", "/EHsc", "/GS-", "/c"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

fn fixture_list(args: &Args, limit: Option<usize>) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = match args.get("--fixtures") {
        Some(csv) => csv
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| {
                let p = PathBuf::from(s);
                if p.is_absolute() || p.exists() {
                    p
                } else {
                    c2_harness::fixtures_dir().join(s)
                }
            })
            .collect(),
        None => c2_harness::all_fixtures(),
    };
    v.sort();
    if let Some(n) = limit {
        v.truncate(n);
    }
    v
}

/// One capture + one (or two) tapped replays for one fixture.
struct Cell {
    name: String,
    armed: TapReport,
    /// `Some(true)` = obj identical armed vs disarmed; `Some(false)` = it
    /// moved; `None` = not graded on this path.
    neutral: Option<bool>,
    err: Option<String>,
}

fn run_cell(
    tc: &Toolchain,
    cpp: &Path,
    flags: &[String],
    payload: bool,
    grade_neutrality: bool,
) -> Cell {
    let name = cpp
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| cpp.display().to_string());
    let w = Scratch::new("stage");
    let abs = match cpp.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            return Cell { name, armed: TapReport::default(), neutral: None, err: Some(e.to_string()) }
        }
    };
    let captured = match tc.capture_reference_with(
        &c2_reference::to_wibo_path(&abs),
        &w.path().join("cap"),
        flags,
        None,
    ) {
        Ok(c) => c,
        Err(e) => {
            return Cell { name, armed: TapReport::default(), neutral: None, err: Some(format!("capture: {e}")) }
        }
    };
    // BOTH legs write to the reference's own /Fo path: c2 embeds the output
    // path in the obj, so replaying elsewhere changes the obj's LENGTH and the
    // comparison would grade the path string rather than the compiler.
    let out = captured.ref_obj_path.clone();
    let il = w.path().join("il");

    let mut neutral = None;
    if grade_neutrality {
        match tc.replay_tapped_with(&captured, &il, &out, &[], payload) {
            Ok((disarmed, rep0)) => {
                if !rep0.lines.is_empty() {
                    return Cell {
                        name,
                        armed: TapReport::default(),
                        neutral: None,
                        err: Some("the DISARMED leg printed stage-tap output".into()),
                    };
                }
                match tc.replay_tapped_with(&captured, &il, &out, STAGE_SITES, payload) {
                    Ok((armed_obj, rep)) => {
                        neutral = Some(ObjImage::diff(&disarmed, &armed_obj) == ObjDiff::Identical);
                        return Cell { name, armed: rep, neutral, err: None };
                    }
                    Err(e) => {
                        return Cell { name, armed: TapReport::default(), neutral: None, err: Some(format!("armed replay: {e}")) }
                    }
                }
            }
            Err(e) => {
                return Cell { name, armed: TapReport::default(), neutral: None, err: Some(format!("disarmed replay: {e}")) }
            }
        }
    }
    match tc.replay_tapped_with(&captured, &il, &out, STAGE_SITES, payload) {
        Ok((_obj, rep)) => Cell { name, armed: rep, neutral, err: None },
        Err(e) => Cell { name, armed: TapReport::default(), neutral: None, err: Some(format!("armed replay: {e}")) },
    }
}

pub(crate) fn cmd_stage(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&STAGE_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let sub = args.first().unwrap_or("").to_string();
    if !["counts", "snap", "determinism", "neutrality"].contains(&sub.as_str()) {
        eprintln!(
            "usage: c2rs stage <counts|snap|determinism|neutrality> [--fixtures a.cpp,b.cpp] \
             [--limit N] [--runs N] [--payload] [--flag F ...]\n\
             default flags: /O1 /Oi /EHsc /GS- /c (the workload's own profile)"
        );
        return ExitCode::from(2);
    }
    let limit = match args.num::<usize>("--limit") {
        Ok(v) => v,
        Err(c) => return c,
    };
    let runs = match args.num::<usize>("--runs") {
        Ok(v) => v.unwrap_or(5),
        Err(c) => return c,
    };
    let mut flags: Vec<String> = args.all("--flag").into_iter().map(String::from).collect();
    if flags.is_empty() {
        flags = default_flags();
    }
    let payload = args.has("--payload");
    let Some(tc) = args.toolchain() else {
        return ExitCode::SUCCESS;
    };
    if !tc.has_strace() {
        println!("SKIP: strace absent (needed to keep the IL bundle)");
        return ExitCode::SUCCESS;
    }
    if !tc.has_mingw() {
        println!("SKIP: i686-w64-mingw32-gcc absent (needed to build c2host)");
        return ExitCode::SUCCESS;
    }
    let fixtures = fixture_list(&args, limit);
    match sub.as_str() {
        "neutrality" => cmd_neutrality(&tc, &fixtures, &flags, payload),
        "counts" => cmd_counts(&tc, &fixtures, &flags),
        "snap" => cmd_snap(&tc, &fixtures, &flags),
        "determinism" => cmd_determinism(&tc, &fixtures, &flags, payload, runs),
        _ => unreachable!(),
    }
}

/// **G1 at scale.** The required-zero.
fn cmd_neutrality(tc: &Toolchain, fixtures: &[PathBuf], flags: &[String], payload: bool) -> ExitCode {
    let mut graded = 0usize;
    let mut differs = 0usize;
    let mut errs = 0usize;
    let mut armed_total = 0usize;
    let mut refused_total = 0usize;
    println!(
        "stage neutrality: {} fixtures, payload={}, flags {}",
        fixtures.len(),
        if payload { "on" } else { "off" },
        flags.join(" ")
    );
    for (i, f) in fixtures.iter().enumerate() {
        let c = run_cell(tc, f, flags, payload, true);
        match (&c.err, c.neutral) {
            (Some(e), _) => {
                errs += 1;
                println!("  [{}/{}] ERR    {}  {}", i + 1, fixtures.len(), c.name, e);
            }
            (None, Some(true)) => {
                graded += 1;
                armed_total += c.armed.armed.len();
                refused_total += c.armed.refused.len();
                println!(
                    "  [{}/{}] SAME   {}  hits={} regions={} tuples={}",
                    i + 1,
                    fixtures.len(),
                    c.name,
                    c.armed.total_hits(),
                    c.armed.regions,
                    c.armed.tuples.len()
                );
            }
            (None, Some(false)) => {
                graded += 1;
                differs += 1;
                armed_total += c.armed.armed.len();
                refused_total += c.armed.refused.len();
                println!(
                    "  [{}/{}] DIFFERS {}  <-- THE TAP MOVED THE OBJ",
                    i + 1,
                    fixtures.len(),
                    c.name
                );
            }
            (None, None) => {
                errs += 1;
                println!("  [{}/{}] UNGRADED {}", i + 1, fixtures.len(), c.name);
            }
        }
    }
    println!();
    println!("  gap-metric stage-tap-obj-differs {differs}");
    println!("  gap-metric stage-tap-graded {graded}");
    println!("  gap-metric stage-tap-errors {errs}");
    println!("  gap-metric stage-sites-armed {armed_total}");
    println!("  gap-metric stage-sites-refused {refused_total}");
    if differs == 0 && graded > 0 {
        println!(
            "\n  G1 HOLDS over {graded} graded fixtures: the armed obj is byte-identical to the\n  \
             disarmed one. A required-zero over an EMPTY population would be free, which is why\n  \
             the denominator is printed beside it."
        );
    } else if graded == 0 {
        println!("\n  G1 IS VACUOUS: nothing was graded. This is NOT a pass.");
    } else {
        println!("\n  G1 FAILS on {differs} of {graded}. The oracle is grading a different compiler.");
    }
    ExitCode::SUCCESS
}

/// Per-site hit histogram — and the second derivation of P_DAG §1's
/// "four scheduler runs per function".
fn cmd_counts(tc: &Toolchain, fixtures: &[PathBuf], flags: &[String]) -> ExitCode {
    let mut hist: BTreeMap<String, u64> = BTreeMap::new();
    let mut ok = 0usize;
    println!("stage counts: {} fixtures, flags {}", fixtures.len(), flags.join(" "));
    for f in fixtures {
        let c = run_cell(tc, f, flags, false, false);
        if let Some(e) = &c.err {
            println!("  ERR  {}  {}", c.name, e);
            continue;
        }
        if !c.armed.armed_ok() {
            println!("  UNARMED  {}", c.name);
            continue;
        }
        ok += 1;
        let mut row = String::new();
        for s in STAGE_SITES {
            let n = c.armed.hits_at(s);
            *hist.entry((*s).to_string()).or_default() += n;
            row.push_str(&format!(" {s}={n}"));
        }
        println!("  {}{}", c.name, row);
    }
    println!();
    for s in STAGE_SITES {
        println!("  gap-metric stage-hits-{s} {}", hist.get(*s).copied().unwrap_or(0));
    }
    println!("  gap-metric stage-counts-tus {ok}");
    // SECOND DERIVATION (#3288) of a published whitebox count: P_DAG.md §1 says
    // three mode-1 scheduler runs come from 0x10b7dc51 and one mode-0 run from
    // 0x10b7df57, per function. The three in-band sites must therefore agree
    // with each other exactly, and `color` and `globregs` must equal them.
    let s1 = hist.get("sched1").copied().unwrap_or(0);
    let s2 = hist.get("sched2").copied().unwrap_or(0);
    let s3 = hist.get("sched3").copied().unwrap_or(0);
    let s0 = hist.get("sched0").copied().unwrap_or(0);
    let color = hist.get("color").copied().unwrap_or(0);
    let globregs = hist.get("globregs").copied().unwrap_or(0);
    println!(
        "\n  SECOND DERIVATION of P_DAG.md §1 (\"four runs per function, three from 0x10b7dc51\n  \
         and one from 0x10b7df57\"): sched1={s1} sched2={s2} sched3={s3} sched0={s0}\n  \
         globregs={globregs} color={color}\n  \
         in-band three equal: {}    mode-0 equals them: {}    color==globregs==in-band: {}",
        s1 == s2 && s2 == s3,
        s0 == s1,
        color == globregs && color == s1
    );
    ExitCode::SUCCESS
}

/// Dump one fixture's canonical snapshot, with the payload on.
fn cmd_snap(tc: &Toolchain, fixtures: &[PathBuf], flags: &[String]) -> ExitCode {
    for f in fixtures {
        let c = run_cell(tc, f, flags, true, false);
        if let Some(e) = &c.err {
            println!("ERR {} {}", c.name, e);
            continue;
        }
        println!("== {} digest={:016x}", c.name, c.armed.digest());
        print!("{}", c.armed.canonical_bytes());
        println!(
            "  gap-metric stage-snap-tuples {}\n  gap-metric stage-snap-regions {}\n  \
             gap-metric stage-snap-walk-refusals {}",
            c.armed.tuples.len(),
            c.armed.regions,
            c.armed.walk_refusals.len()
        );
    }
    ExitCode::SUCCESS
}

/// **G2 and G2b.** N runs in the same configuration, then N more from a
/// different working directory with a different `/Fo` path.
///
/// G2b is not decoration: without it, a digest is stable only because the
/// environment was, and a leaked path or pointer would sail through G2.
fn cmd_determinism(
    tc: &Toolchain,
    fixtures: &[PathBuf],
    flags: &[String],
    payload: bool,
    runs: usize,
) -> ExitCode {
    let mut unstable = 0usize;
    let mut distinct_max = 1usize;
    let mut graded = 0usize;
    let mut empty = 0usize;
    println!(
        "stage determinism: {} fixtures x {runs} same-config runs + {runs} \
         different-cwd runs, payload={}",
        fixtures.len(),
        if payload { "on" } else { "off" }
    );
    for f in fixtures {
        let mut digests: Vec<u64> = Vec::new();
        let mut tuples = 0usize;
        let mut err: Option<String> = None;
        for _ in 0..runs {
            let c = run_cell(tc, f, flags, payload, false);
            if let Some(e) = c.err {
                err = Some(e);
                break;
            }
            if !c.armed.armed_ok() {
                err = Some("did not arm".into());
                break;
            }
            tuples = c.armed.tuples.len();
            digests.push(c.armed.digest());
        }
        // G2b: the SAME fixture again, but every run gets a fresh scratch
        // directory and therefore a different /Fo path (Scratch::new mints a
        // pid+nanos+counter name). Any path, PID or pointer in the stream shows
        // up here and nowhere else.
        if err.is_none() {
            for _ in 0..runs {
                let c = run_cell(tc, f, flags, payload, false);
                if let Some(e) = c.err {
                    err = Some(e);
                    break;
                }
                digests.push(c.armed.digest());
            }
        }
        let name = f.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        if let Some(e) = err {
            println!("  ERR   {name}  {e}");
            continue;
        }
        graded += 1;
        let mut uniq: Vec<u64> = digests.clone();
        uniq.sort_unstable();
        uniq.dedup();
        distinct_max = distinct_max.max(uniq.len());
        if uniq.len() > 1 {
            unstable += 1;
        }
        if tuples == 0 && payload {
            empty += 1;
        }
        println!(
            "  {name}  runs={} distinct={} tuples={} {}",
            digests.len(),
            uniq.len(),
            tuples,
            if uniq.len() == 1 { "STABLE" } else { "UNSTABLE" }
        );
    }
    println!();
    println!("  gap-metric stage-snap-runs {}", runs * 2);
    println!("  gap-metric stage-snap-distinct-max {distinct_max}");
    println!("  gap-metric stage-snap-unstable-tus {unstable}");
    println!("  gap-metric stage-snap-graded {graded}");
    if payload {
        println!("  gap-metric stage-snap-empty-payload {empty}");
        if empty > 0 {
            println!(
                "\n  {empty} of {graded} produced a DETERMINISTIC EMPTY payload. That passes every\n  \
                 other criterion here trivially and is reported as what it is, not as a green."
            );
        }
    }
    let _ = OPT_GATED_SITES;
    ExitCode::SUCCESS
}
