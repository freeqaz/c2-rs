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
//! | `stage-tap-obj-differs` | **REQUIRED 0.** Objs whose bytes changed when the tap was armed. Nonzero ⇒ the oracle is grading a different compiler than the judge does ⇒ decline |
//! | `stage-tap-armed-and-fired` | **THE DENOMINATOR OF THE LINE ABOVE.** Objs where every requested site armed, none refused, and at least one detour executed. Byte-identity anywhere else is free |
//! | `stage-tap-armed-nofire` | armed, but c2 never reached a site (a TU with no function body). Reported, never counted as evidence |
//! | `stage-tap-unarmed` | runs where nothing was patched. **Any nonzero value makes the verdict VACUOUS** |
//! | `stage-tap-graded` | objs that produced an obj comparison at all — a superset of `armed-and-fired`, kept for continuity |
//! | `stage-tap-hits` / `stage-sites-armed` / `stage-sites-refused` | the positive evidence the verdict is conditioned on |
//! | `stage-tap-walk-refusals` | payload truncation seen during the neutrality campaign. A quoted zero has to come from a log that could have carried a one |
//! | `stage-snap-runs` / `stage-snap-distinct-max` / `stage-snap-unstable-tus` | determinism |
//! | `stage-snap-tuples` / `stage-snap-regions` | **content.** A structurally deterministic EMPTY snapshot passes every other criterion here trivially |
//! | `stage-snap-walk-refusals` | payload truncation on a `snap` run. Never folded into the tuple count |
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

use c2_harness::toolchain_gate::{toolchain_ready, Cap};

use crate::cli::util::Scratch;
use crate::{Args, Arity, Spec};

// PROV[N] not load-bearing — a CLI argument specification for this crate's own `c2rs` binary. Nothing in it is derived from `c2.dll`; a wrong value changes a usage message or a parse, never a graded byte.
static STAGE_SPEC: Spec = Spec::new(
    "stage",
    &[
        ("--fixtures", Arity::Value),
        ("--runs", Arity::Value),
        ("--limit", Arity::Value),
        ("--payload", Arity::Flag),
        ("--raw", Arity::Value),
        ("--list", Arity::Value),
        ("--flags-file", Arity::Value),
        ("--cwd", Arity::Value),
        ("--flag", Arity::Repeated),
        ("--force-slide", Arity::Value),
    ],
)
.requires(&[("--cwd", "--flags-file")])
.positionals(1);

/// The workload's own optimization profile — the one the 878 TUs compile at.
fn default_flags() -> Vec<String> {
    ["/O1", "/Oi", "/EHsc", "/GS-", "/c"]
        .iter()
        .map(|s| s.to_string())
        .collect()
}

fn fixture_list(args: &Args, limit: Option<usize>) -> Vec<PathBuf> {
    // `--list FILE` is how the 26 MATCHED WORKLOAD TUs get graded by the same
    // instrument as the fixtures. Without it the neutrality claim would be a
    // claim about `fixtures/cpp` only, and the workload's TUs are an order of
    // magnitude larger and are the population the goal is written in.
    if let Some(list) = args.get("--list") {
        let base = args.path("--cwd").unwrap_or_else(|| PathBuf::from("."));
        let text = std::fs::read_to_string(list).unwrap_or_default();
        let mut v: Vec<PathBuf> = text
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(|l| base.join(l))
            .collect();
        if let Some(n) = limit {
            v.truncate(n);
        }
        return v;
    }
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

fn run_cell_in(
    tc: &Toolchain,
    cpp: &Path,
    flags: &[String],
    payload: bool,
    grade_neutrality: bool,
    raw: u32,
    cwd: Option<&Path>,
) -> Cell {
    run_cell_in_dir(tc, cpp, flags, payload, grade_neutrality, raw, cwd, None, None)
}

/// [`run_cell_in`], with the scratch directory optionally FIXED.
///
/// `reuse: Some(dir)` runs the cell in a caller-owned directory instead of a
/// freshly minted one. That is the difference between G2's two legs and it did
/// not exist before the fix round: every run used to mint its own
/// `pid+nanos+counter` dir, so the *"5 same-config runs"* leg and the *"5 runs
/// from a fresh working directory"* leg were byte-identical calls and the
/// designed contrast never ran (review finding).
#[allow(clippy::too_many_arguments)]
fn run_cell_in_dir(
    tc: &Toolchain,
    cpp: &Path,
    flags: &[String],
    payload: bool,
    grade_neutrality: bool,
    raw: u32,
    cwd: Option<&Path>,
    reuse: Option<&Path>,
    // `force_slide`: displace every site address, so the fail-closed check
    // refuses everything. THE MUTATION LEVER for this command's own
    // required-zero — it produces exactly the population where armed-vs-
    // disarmed identity is free, and the verdict has to say VACUOUS over it.
    force_slide: Option<u32>,
) -> Cell {
    let name = cpp
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| cpp.display().to_string());
    let w = Scratch::or_work(reuse.map(|p| p.to_path_buf()), "stage");
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
        cwd,
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

    // Both arms yield `(Option<ObjImage>, TapReport)`: a forced-slide run can
    // legitimately produce NO obj (a wrong slide that patched something can
    // crash c2), and that has to reach the caller as data rather than as an
    // error, or the arming evidence is lost behind an early failure.
    let armed_replay = |il: &Path, out: &Path| match force_slide {
        Some(v) => tc.replay_tapped_forced_slide(&captured, il, out, STAGE_SITES, v),
        None => tc
            .replay_tapped_raw(&captured, il, out, STAGE_SITES, payload, raw)
            .map(|(obj, rep)| (Some(obj), rep)),
    };

    let mut neutral = None;
    if grade_neutrality {
        match tc.replay_tapped_raw(&captured, &il, &out, &[], payload, raw) {
            Ok((disarmed, rep0)) => {
                if !rep0.lines.is_empty() {
                    return Cell {
                        name,
                        armed: TapReport::default(),
                        neutral: None,
                        err: Some("the DISARMED leg printed stage-tap output".into()),
                    };
                }
                match armed_replay(&il, &out) {
                    Ok((Some(armed_obj), rep)) => {
                        neutral = Some(ObjImage::diff(&disarmed, &armed_obj) == ObjDiff::Identical);
                        return Cell { name, armed: rep, neutral, err: None };
                    }
                    Ok((None, rep)) => {
                        // No obj at all: the armed leg crashed or aborted. It
                        // is NOT neutral and it is NOT graded — and the report
                        // is kept, because what armed is the interesting half.
                        return Cell {
                            name,
                            armed: rep,
                            neutral: None,
                            err: Some("the ARMED leg produced no obj".into()),
                        };
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
    match armed_replay(&il, &out) {
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
    if let Some(ff) = args.path("--flags-file") {
        match std::fs::read_to_string(&ff) {
            Ok(t) => flags = t.split_whitespace().map(String::from).collect(),
            Err(e) => {
                eprintln!("stage: cannot read --flags-file {}: {e}", ff.display());
                return ExitCode::from(2);
            }
        }
    }
    if flags.is_empty() {
        flags = default_flags();
    }
    let payload = args.has("--payload");
    // `--force-slide HEX` — THE MUTATION LEVER, kept in the shipped command
    // rather than in a throwaway patch so the demonstration is reproducible by
    // anyone reading the rung. It displaces every site address, so the
    // fail-closed check refuses all seven and NOTHING is patched: exactly the
    // population over which armed-vs-disarmed obj identity is free.
    let force_slide: Option<u32> = match args.get("--force-slide") {
        Some(h) => match u32::from_str_radix(h.trim_start_matches("0x"), 16) {
            Ok(v) => Some(v),
            Err(_) => {
                eprintln!("stage: --force-slide wants a hex number, got {h:?}");
                return ExitCode::from(2);
            }
        },
        None => None,
    };
    let raw = match args.num::<u32>("--raw") {
        Ok(v) => v.unwrap_or(0),
        Err(c) => return c,
    };
    let Some(tc) = args.toolchain() else {
        return ExitCode::SUCCESS;
    };
    // THE FUNNEL, not a sixteenth copy. `w-refrev` landed
    // `toolchain_gate::toolchain_ready` on master while this branch was in
    // review, converting fifteen hand-rolled skip blocks; this command was
    // written in parallel and would have re-opened the hole the funnel exists
    // to close — a demand honoured at fourteen sites and not the fifteenth is a
    // demand with a hole in it. Under `C2RS_REQUIRE_TOOLCHAIN` a partially
    // provisioned run now REFUSES here instead of printing SKIP and exiting 0.
    if let Some(code) = toolchain_ready(
        &tc,
        &[Cap::Strace, Cap::Mingw],
        "needed to keep the IL bundle and build c2host",
    ) {
        return code;
    }
    let fixtures = fixture_list(&args, limit);
    // The workload's `/I src/...` roots are RELATIVE, so a --list run without
    // its --cwd silently compiles 14 of 26 matched TUs into `fatal error
    // C1083` and reports them as errors. Measured, not anticipated.
    let cwd = args.path("--cwd");
    let cwd = cwd.as_deref();
    match sub.as_str() {
        "neutrality" => cmd_neutrality(&tc, &fixtures, &flags, payload, cwd, force_slide),
        "counts" => cmd_counts(&tc, &fixtures, &flags, cwd),
        "snap" => cmd_snap(&tc, &fixtures, &flags, raw, cwd),
        "determinism" => cmd_determinism(&tc, &fixtures, &flags, payload, runs, cwd),
        _ => unreachable!(),
    }
}

/// **G1 at scale.** The required-zero.
///
/// # The denominator this command publishes, and why it is not the fixture count
///
/// FIX-ROUND CORRECTION (review finding, 2026-08-20). The first version of this
/// command graded on `c.neutral` alone and printed *"G1 HOLDS over N graded
/// fixtures"*. That sentence could be printed over a population **in which not
/// one byte of c2.dll was ever patched** — a disarmed run and a disarmed run
/// produce identical objs trivially — and over objs where the sites were armed
/// but c2 never reached them. Both are byte-identity for free.
///
/// So the verdict is now conditioned on a POSITIVE check
/// ([`TapReport::armed_and_fired`]: the run must have armed every requested
/// site, refused none, and *executed* at least one detour), never on an
/// enumeration of the ways a run can be empty, and the published denominator
/// is `stage-tap-armed-and-fired`. Objs that armed but never fired are counted
/// and printed separately as `NOFIRE`; they are a real and reportable
/// population (a TU that emits no function body runs no per-function phase),
/// but they are not evidence about neutrality.
#[allow(clippy::too_many_arguments)]
fn cmd_neutrality(
    tc: &Toolchain,
    fixtures: &[PathBuf],
    flags: &[String],
    payload: bool,
    cwd: Option<&Path>,
    force_slide: Option<u32>,
) -> ExitCode {
    let mut graded = 0usize;
    let mut fired = 0usize;
    let mut nofire = 0usize;
    let mut unarmed = 0usize;
    let mut differs = 0usize;
    let mut errs = 0usize;
    let mut armed_total = 0usize;
    let mut refused_total = 0usize;
    let mut hits_total = 0u64;
    let mut walk_refusals = 0usize;
    println!(
        "stage neutrality: {} fixtures, payload={}, flags {}{}",
        fixtures.len(),
        if payload { "on" } else { "off" },
        flags.join(" "),
        match force_slide {
            Some(v) => format!(
                "\n  --force-slide {v:x}: THE SITE ADDRESSES ARE DELIBERATELY WRONG. Every site must \
                 refuse\n  and the verdict must read VACUOUS — this is the mutation, not a measurement."
            ),
            None => String::new(),
        }
    );
    for (i, f) in fixtures.iter().enumerate() {
        let c = run_cell_in_dir(tc, f, flags, payload, true, 0, cwd, None, force_slide);
        // WALK REFUSALS ARE SURFACED HERE, not only by `stage snap`. The rung
        // claimed "zero walk refusals over the whole campaign" from a log that
        // structurally could not contain the line (review finding). A count
        // that is quoted has to be printed by the command that produced it.
        for w in &c.armed.walk_refusals {
            walk_refusals += 1;
            println!("  [{}/{}] TRUNCATED {}  {w}", i + 1, fixtures.len(), c.name);
        }
        let armed_and_fired = c.armed.armed_and_fired();
        match (&c.err, c.neutral) {
            (Some(e), _) => {
                errs += 1;
                println!("  [{}/{}] ERR    {}  {}", i + 1, fixtures.len(), c.name, e);
            }
            (None, Some(_)) if !c.armed.armed_ok() => {
                // The bytes were never written (or a site refused): the obj
                // comparison below grades nothing at all, whichever way it came
                // out. This is a FAILURE of the measurement, not a data point.
                unarmed += 1;
                armed_total += c.armed.armed.len();
                refused_total += c.armed.refused.len();
                println!(
                    "  [{}/{}] UNARMED {}  armed={:?} refused={:?}",
                    i + 1,
                    fixtures.len(),
                    c.name,
                    c.armed.armed,
                    c.armed.refused
                );
            }
            (None, Some(true)) => {
                graded += 1;
                armed_total += c.armed.armed.len();
                refused_total += c.armed.refused.len();
                hits_total += c.armed.total_hits();
                if armed_and_fired {
                    fired += 1;
                } else {
                    nofire += 1;
                }
                println!(
                    "  [{}/{}] {}   {}  hits={} regions={} tuples={}",
                    i + 1,
                    fixtures.len(),
                    if armed_and_fired { "SAME  " } else { "NOFIRE" },
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
                hits_total += c.armed.total_hits();
                if armed_and_fired {
                    fired += 1;
                } else {
                    nofire += 1;
                }
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
    println!("  gap-metric stage-tap-armed-and-fired {fired}");
    println!("  gap-metric stage-tap-armed-nofire {nofire}");
    println!("  gap-metric stage-tap-unarmed {unarmed}");
    println!("  gap-metric stage-tap-graded {graded}");
    println!("  gap-metric stage-tap-errors {errs}");
    println!("  gap-metric stage-tap-hits {hits_total}");
    println!("  gap-metric stage-sites-armed {armed_total}");
    println!("  gap-metric stage-sites-refused {refused_total}");
    println!("  gap-metric stage-tap-walk-refusals {walk_refusals}");
    // THE POSITIVE CHECK. Every clause below demands that something HAPPENED;
    // none of them is satisfied by an empty run.
    let armed_ran = unarmed == 0 && refused_total == 0 && armed_total > 0 && hits_total > 0 && fired > 0;
    if !armed_ran {
        println!(
            "\n  G1 IS VACUOUS — THE TAP DID NOT ARM AND FIRE. This is NOT a pass, and the\n  \
             obj identity above is free: {unarmed} unarmed runs, {refused_total} site refusals,\n  \
             {armed_total} sites armed, {hits_total} detour hits, {fired} objs armed-and-fired."
        );
    } else if differs == 0 {
        println!(
            "\n  G1 HOLDS over {fired} objs ON WHICH THE TAP ARMED AND FIRED ({armed_total} sites\n  \
             armed, 0 refused, {hits_total} detour hits inside c2's own code): the armed obj is\n  \
             byte-identical to the disarmed one. {nofire} further objs armed but never fired —\n  \
             c2 ran no per-function phase on them — and are EXCLUDED from that denominator,\n  \
             because byte-identity is free where no detour executed."
        );
    } else {
        println!("\n  G1 FAILS on {differs} of {fired}. The oracle is grading a different compiler.");
    }
    if walk_refusals > 0 {
        println!(
            "  {walk_refusals} TRUNCATED payload(s): every tuple count from this campaign is a\n  \
             FLOOR and not a measurement."
        );
    }
    ExitCode::SUCCESS
}

/// Per-site hit histogram — and the second derivation of P_DAG §1's
/// "four scheduler runs per function".
fn cmd_counts(tc: &Toolchain, fixtures: &[PathBuf], flags: &[String], cwd: Option<&Path>) -> ExitCode {
    let mut hist: BTreeMap<String, u64> = BTreeMap::new();
    let mut ok = 0usize;
    println!("stage counts: {} fixtures, flags {}", fixtures.len(), flags.join(" "));
    for f in fixtures {
        let c = run_cell_in(tc, f, flags, false, false, 0, cwd);
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
fn cmd_snap(tc: &Toolchain, fixtures: &[PathBuf], flags: &[String], raw: u32, cwd: Option<&Path>) -> ExitCode {
    for f in fixtures {
        let c = run_cell_in(tc, f, flags, true, false, raw, cwd);
        if let Some(e) = &c.err {
            println!("ERR {} {}", c.name, e);
            continue;
        }
        println!("== {} digest={:016x}", c.name, c.armed.digest());
        print!("{}", c.armed.canonical_bytes());
        // The PRE/POST-COLOR pair, per function. `sched2` is the last
        // scheduler run before the register allocator and `sched3` the first
        // after it (P_DAG.md §1), so this pair costs nothing extra: it is the
        // region tap read at two phases.
        //
        // An EMPTY difference is a FINDING and is printed in those words: it
        // would mean the walk is reading a list COLOR does not write.
        let funcs = c.armed.blocks.iter().map(|b| b.func).max().unwrap_or(0);
        let mut differing = 0usize;
        let mut paired = 0usize;
        // Every adjacent phase pair, not only the COLOR one. Reporting only
        // the pair a lane hoped would move is how an instrument gets graded on
        // the answer it wanted: `sched1`->`sched2` brackets a SCHEDULER run and
        // `sched2`->`sched3` brackets the ALLOCATOR, so printing both says
        // which passes this observable can and cannot see.
        for (a, b, what) in [
            ("sched1", "sched2", "SCHED+GLOBREGS"),
            ("sched2", "sched3", "COLOR"),
            ("sched3", "sched0", "LOWERING"),
        ] {
            for f in 1..=funcs {
                let cat = |phase: &str| -> Vec<String> {
                    c.armed
                        .blocks_at(phase, f)
                        .into_iter()
                        .flat_map(|x| x.tuples.iter().cloned())
                        .collect()
                };
                let (before, after) = (cat(a), cat(b));
                if before.is_empty() && after.is_empty() {
                    continue;
                }
                if what == "COLOR" {
                    paired += 1;
                    if before != after {
                        differing += 1;
                    }
                }
                println!(
                    "  {what} fn{f}: {a}={} {b}={} {}",
                    before.len(),
                    after.len(),
                    if before == after { "IDENTICAL" } else { "DIFFERS" }
                );
            }
        }
        // A DISTINCT-ROW COUNT BESIDE THE PAYLOAD SIZE (arch review
        // 2026-08-21, consequence 4). The walk terminates on `next == 0`, the
        // end of the LIST, so region k's walk re-reads regions k..n and
        // `stage-snap-tuples` counts every re-read. Publishing the payload size
        // alone invites reading it as coverage; the two counts side by side
        // make the inflation a number rather than a footnote.
        let d = c.armed.distinct_rows();
        println!(
            "  gap-metric stage-snap-tuples {}\n  gap-metric stage-snap-regions {}\n  \
             gap-metric stage-snap-walk-refusals {}\n  \
             gap-metric stage-snap-tuples-distinct {}\n  \
             gap-metric stage-snap-tuple-groups {}\n  \
             gap-metric stage-snap-suffix-violations {}\n  \
             gap-metric stage-color-pairs {paired}\n  \
             gap-metric stage-color-pairs-differing {differing}",
            c.armed.tuples.len(),
            c.armed.regions,
            c.armed.walk_refusals.len(),
            d.distinct,
            d.groups,
            d.suffix_violations,
        );
        if d.rows > 0 {
            println!(
                "  PAYLOAD {} rows over {} (phase,function) groups; {} DISTINCT tuple positions \
                 ({:.1}% of the rows are suffix re-reads, inflation {:.2}x). {}",
                d.rows,
                d.groups,
                d.distinct,
                100.0 * (d.rows - d.distinct) as f64 / d.rows as f64,
                d.rows as f64 / d.distinct.max(1) as f64,
                if d.suffix_violations == 0 {
                    "The nested-suffix model holds on every block, so DISTINCT is exact."
                } else {
                    "SUFFIX MODEL VIOLATED on at least one block — DISTINCT is a FLOOR."
                },
            );
        }
        if raw > 0 {
            // WHICH BYTE OFFSETS DOES COLOR WRITE? Answered, not guessed:
            // align the sched2 and sched3 raw windows row-for-row and report
            // every offset that differs. Offsets that differ on EVERY function
            // are structural; offsets that differ on some are data.
            let mut differ_count = vec![0usize; (raw as usize) * 2];
            let mut compared = 0usize;
            let mut skipped = 0usize;
            for f in 1..=funcs {
                let b2: Vec<String> = c.armed.blocks_at("sched2", f).into_iter()
                    .flat_map(|b| b.raw.iter().cloned()).collect();
                let b3: Vec<String> = c.armed.blocks_at("sched3", f).into_iter()
                    .flat_map(|b| b.raw.iter().cloned()).collect();
                if b2.len() != b3.len() {
                    // A LENGTH MISMATCH IS NOT A ZERO. The two windows cannot be
                    // aligned row-for-row, so this function contributes nothing —
                    // and it must be counted, or the verdict below reads the
                    // resulting silence as "COLOR wrote nothing".
                    if !(b2.is_empty() && b3.is_empty()) { skipped += 1; }
                    continue;
                }
                for (x, y) in b2.iter().zip(b3.iter()) {
                    compared += 1;
                    let (xb, yb) = (x.as_bytes(), y.as_bytes());
                    for i in 0..xb.len().min(yb.len()).min(differ_count.len()) {
                        if xb[i] != yb[i] { differ_count[i] += 1; }
                    }
                }
            }
            let mut hot: Vec<String> = Vec::new();
            for byte_off in 0..(raw as usize) {
                let n = differ_count[byte_off * 2] + differ_count[byte_off * 2 + 1];
                if n > 0 { hot.push(format!("+0x{byte_off:x}({n})")); }
            }
            // GUARD THE VERDICT ON `compared > 0` (arch review 2026-08-21,
            // finding 1, "live wrong-but-green instance"). `hot` is empty both
            // when the allocator wrote nothing AND when nothing was compared,
            // and the old text asserted the first over the second. An
            // instrument that cannot tell a measured zero from an unmeasured
            // one is this project's signature defect wearing a required-zero's
            // clothes; the two cases now have different sentences.
            if compared == 0 {
                println!(
                    "  RAW WINDOW VACUOUS — 0 pairs aligned across sched2/sched3 \
                     ({skipped} function(s) skipped on a length mismatch, window {raw}B).\n                   \
                     NOTHING WAS COMPARED. This says nothing about what COLOR writes."
                );
            } else {
                println!(
                    "  RAW WINDOW {raw}B, {compared} tuple pairs aligned across sched2/sched3 \
                     ({skipped} function(s) skipped on a length mismatch).\n                   offsets COLOR writes: {}",
                    if hot.is_empty() { "NONE — the allocator wrote nothing in this window".to_string() } else { hot.join(" ") }
                );
            }
        }
        // A TRUNCATED payload makes every count above a floor — and it does
        // worse than that: at an 8 KiB arena this same fixture reports a COLOR
        // pair DIFFERING that is 0 of 7 when the payload is whole
        // (work/oracle/fixround/mutation_arena_full.log). So the banner is
        // printed before any conclusion below it can be read.
        if !c.armed.walk_refusals.is_empty() {
            println!(
                "  TRUNCATED — {} walk refusal(s): {}.\n                   Every count above is a FLOOR, and the phase comparisons below are\n                   comparing partial lists. Nothing here is a measurement.",
                c.armed.walk_refusals.len(),
                c.armed.walk_refusals.join("; ")
            );
        }
        // ------------------------------------------------------------------
        // THE FUNCTION WALK (C2RS_STAGE_FUNCWALK) — the observable the region
        // tap cannot give.
        //
        // Two things the region tap structurally cannot do, and both are what
        // arch review 2026-08-21 finding 1 turns on:
        //   * it visits every tuple ONCE, so there is no suffix inflation and
        //     a phase-to-phase comparison is over the whole function;
        //   * it exists at `after0`, i.e. AFTER run 4 — the run that fixes
        //     emitted instruction order, whose output was observed nowhere.
        //
        // Each adjacent pair is reported twice: SPINE (opcode/category/flags/
        // cc only) and FULL (with the operand and symbol records inline).
        // A pair that is IDENTICAL on the spine and DIFFERS on FULL is a pass
        // whose entire output is in the operand records — which is exactly the
        // shape the register allocator was predicted to have.
        if !c.armed.funcs.is_empty() {
            let fw_funcs = c.armed.funcs.iter().map(|f| f.func).max().unwrap_or(0);
            let phases = ["sched1", "globregs", "sched2", "color", "sched3", "sched0", "after0"];
            println!("\n  FUNCTION WALK — {} blocks over {} function(s)", c.armed.funcs.len(), fw_funcs);
            let pick = |phase: &str, f: u32| -> Option<&c2_reference::stage::FuncWalk> {
                c.armed.funcs.iter().find(|w| w.phase == phase && w.func == f)
            };
            // #3459 — WHICH function each ordinal is. `stage snap`'s own
            // comparisons below are TAP-INTERNAL (funcwalk against funcwalk,
            // funcwalk against the region walk, both at the same ordinal in the
            // same run), so the hazard #3459 names — pairing the ordinal to an
            // obj's `.text` address order — does not reach them and none of
            // these numbers moves. What DOES change is that the reader of this
            // dump can now see which function a row is about, which is the
            // difference between a report and a report you can act on.
            let idof = |f: u32| -> String {
                phases
                    .iter()
                    .find_map(|p| pick(p, f))
                    .and_then(|w| w.identity().map(|s| s.to_string()))
                    .unwrap_or_else(|| "<no identity: #3459 open>".to_string())
            };
            for f in 1..=fw_funcs {
                println!("  FW-ID fn{f} = {}", idof(f));
            }
            let named = (1..=fw_funcs)
                .filter(|f| phases.iter().any(|p| pick(p, *f).is_some_and(|w| w.identity().is_some())))
                .count();
            println!("  gap-metric stage-fw-identified {named}");

            let mut spine_diff = 0usize;
            let mut full_diff = 0usize;
            let mut ops_only = 0usize;
            let mut fw_pairs = 0usize;
            for w in phases.windows(2) {
                let (a, b) = (w[0], w[1]);
                for f in 1..=fw_funcs {
                    let (Some(x), Some(y)) = (pick(a, f), pick(b, f)) else { continue };
                    fw_pairs += 1;
                    let (sa, sb) = (x.spine(), y.spine());
                    let (ra, rb) = (x.rows(), y.rows());
                    let s_same = sa == sb;
                    let r_same = ra == rb;
                    if !s_same { spine_diff += 1; }
                    if !r_same { full_diff += 1; }
                    if s_same && !r_same { ops_only += 1; }
                    // The two legs are the SAME function only if the identity
                    // says so. `g_fn` is incremented at `sched1`; a phase pair
                    // that straddled a skipped `sched1` would otherwise be
                    // diffed as one function and reported as a spine change.
                    let same_fn = match (x.identity(), y.identity()) {
                        (Some(i), Some(j)) => i == j,
                        _ => true, // no identity: the old behaviour, honestly
                    };
                    println!(
                        "  FW {a}->{b} fn{f}: rows {}->{}  spine {}  full {}{}{}",
                        ra.len(),
                        rb.len(),
                        if s_same { "IDENTICAL" } else { "DIFFERS  " },
                        if r_same { "IDENTICAL" } else { "DIFFERS" },
                        if s_same && !r_same { "   <-- OPERAND-ONLY WRITE" } else { "" },
                        if same_fn { "" } else { "   <-- DIFFERENT FUNCTIONS (#3459)" },
                    );
                }
            }
            println!(
                "  gap-metric stage-fw-pairs {fw_pairs}\n  \
                 gap-metric stage-fw-spine-differing {spine_diff}\n  \
                 gap-metric stage-fw-full-differing {full_diff}\n  \
                 gap-metric stage-fw-operand-only {ops_only}"
            );
            // CROSS-DERIVATION (PREREG B2). The function walk and the region
            // walk are two different readings of c2's memory — one from the
            // function record through the block chain, one from the tuple
            // pointer the region finder was handed. If the record-layout
            // reading is right they must agree about which tuples exist at a
            // phase; if it is wrong this is where it shows, rather than in a
            // conclusion nobody can check.
            for phase in ["sched2", "sched3"] {
                for f in 1..=fw_funcs {
                    let Some(w) = pick(phase, f) else { continue };
                    let fw: Vec<String> = w.spine();
                    // The region walk's OWN distinct rows for this (phase, fn):
                    // the longest block, index stripped, operands stripped.
                    let region = c
                        .armed
                        .blocks_at(phase, f)
                        .into_iter()
                        .max_by_key(|b| b.tuples.len())
                        .map(|b| {
                            b.tuples
                                .iter()
                                .map(|t| {
                                    let t = t.split_once(" | ").map(|(s, _)| s).unwrap_or(t);
                                    t.split_once(' ').map(|(_, r)| r.to_string()).unwrap_or_default()
                                })
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                    if region.is_empty() {
                        continue;
                    }
                    // ORDER-SENSITIVE, deliberately — and the upgrade from a
                    // multiset containment to this is why the reading below is
                    // stated the narrow way it is. The multiset check passed
                    // 14 of 14 and was VACUOUS: it would have passed just as
                    // happily on a reversed or re-blocked walk.
                    //
                    // WHAT IT ESTABLISHES: the region walk starts inside one
                    // block and runs to `next == 0`, so its opening rows must
                    // be a SUFFIX of that block, in order. The longest such
                    // match over all blocks is the confirmation of the
                    // within-block direction — the direction is the whole
                    // finding, since the C walk runs backward down
                    // `tuple+0x10` and the reader reverses it.
                    //
                    // WHAT IT DOES NOT ESTABLISH, and this is a MEASURED
                    // limit, not a caveat: the region walk continues past the
                    // end of that block into tuples the block chain
                    // (`block+0x4`) orders EARLIER. So `block+0x4` is a
                    // traversal order, NOT the tuple list's order, and the
                    // per-phase comparison above is a comparison of the same
                    // traversal at two phases — never a claim about emitted
                    // order.
                    let mut best = (0usize, usize::MAX);
                    for (bi, blk) in w.blocks.iter().enumerate() {
                        let spine: Vec<String> = blk
                            .iter()
                            .map(|r| r.split_once(" | ").map(|(s, _)| s.to_string()).unwrap_or_else(|| r.clone()))
                            .collect();
                        let mut k = spine.len().min(region.len());
                        while k > 0 {
                            if spine[spine.len() - k..] == region[..k] {
                                break;
                            }
                            k -= 1;
                        }
                        if k > best.0 {
                            best = (k, bi);
                        }
                    }
                    println!(
                        "  FW-XDERIV {phase} fn{f}: funcwalk {} rows in {} blocks, region walk {} rows -- {}",
                        fw.len(),
                        w.blocks.len(),
                        region.len(),
                        if best.0 >= 3 {
                            format!(
                                "the region walk's first {} rows are the IN-ORDER TAIL of funcwalk block {}: \
                                 the within-block direction is CONFIRMED, and the remaining {} row(s) are \
                                 in blocks the block chain orders earlier",
                                best.0,
                                best.1,
                                region.len() - best.0
                            )
                        } else {
                            format!(
                                "NO in-order tail match of length >= 3 (best {} rows, block {}) -- THE LAYOUT READING IS WRONG\n      \
                                 funcwalk: [{}]\n      region:   [{}]",
                                best.0,
                                best.1,
                                fw.join(" / "),
                                region.join(" / "),
                            )
                        }
                    );
                }
            }
        }
        if paired > 0 && differing == 0 && c.armed.walk_refusals.is_empty() {
            println!(
                "  FINDING, not a green: the pre/post-COLOR snapshots are IDENTICAL on every\n                   function here. The walk is reading fields COLOR does not write — it shows THAT\n                   the allocator ran, not WHAT it did."
            );
        }
    }
    ExitCode::SUCCESS
}

/// **G2 and G2b.** N runs in ONE fixed scratch directory, then N more each in a
/// freshly minted one with a different `/Fo` path.
///
/// G2b is not decoration: without it, a digest is stable only because the
/// environment was, and a leaked path or pointer would sail through G2.
///
/// FIX-ROUND CORRECTION (review finding, 2026-08-20): both loops used to call
/// the same function with the same arguments, and every call minted its own
/// scratch dir — so there was no same-config leg and no contrast between the
/// halves. The published conclusion was unaffected (ten varied runs at
/// `distinct = 1` is strictly stronger than five plus five), but the experiment
/// described was not the experiment that ran. The first leg now pins one
/// directory for all `runs` iterations, so G2 and G2b differ in exactly the way
/// their names claim.
fn cmd_determinism(
    tc: &Toolchain,
    fixtures: &[PathBuf],
    flags: &[String],
    payload: bool,
    runs: usize,
    cwd: Option<&Path>,
) -> ExitCode {
    let mut unstable = 0usize;
    let mut distinct_max = 1usize;
    let mut graded = 0usize;
    let mut empty = 0usize;
    println!(
        "stage determinism: {} fixtures x {runs} runs in ONE fixed scratch dir + {runs} \
         runs each in a FRESH dir (a different /Fo path), payload={}",
        fixtures.len(),
        if payload { "on" } else { "off" }
    );
    for f in fixtures {
        let mut digests: Vec<u64> = Vec::new();
        let mut tuples = 0usize;
        let mut err: Option<String> = None;
        // G2: one directory, held fixed across all `runs` iterations. The
        // scratch dir is minted ONCE here and passed in, so the /Fo path, the
        // bundle path and the capture path are identical run to run.
        let fixed = Scratch::new("stage-g2");
        for _ in 0..runs {
            let c = run_cell_in_dir(tc, f, flags, payload, false, 0, cwd, Some(fixed.path()), None);
            if let Some(e) = c.err {
                err = Some(e);
                break;
            }
            // POSITIVE CHECK, and it must come before the digest is recorded:
            // an unarmed run has a perfectly stable digest (the empty stream
            // hashes the same every time), so "distinct = 1" over unarmed runs
            // is the vacuous green this whole lane exists not to print.
            if !c.armed.armed_and_fired() {
                err = Some(format!(
                    "did not arm and fire (armed={:?} refused={:?} hits={})",
                    c.armed.armed,
                    c.armed.refused,
                    c.armed.total_hits()
                ));
                break;
            }
            tuples = c.armed.tuples.len();
            digests.push(c.armed.digest());
        }
        // G2b: the SAME fixture again, but every run gets a FRESH scratch
        // directory and therefore a different /Fo path (Scratch::new mints a
        // pid+nanos+counter name). Any path, PID or pointer in the stream shows
        // up here and nowhere else — and it can only show up here because the
        // leg above holds the directory still.
        if err.is_none() {
            for _ in 0..runs {
                let c = run_cell_in(tc, f, flags, payload, false, 0, cwd);
                if let Some(e) = c.err {
                    err = Some(e);
                    break;
                }
                if !c.armed.armed_and_fired() {
                    err = Some("did not arm and fire (fresh-dir leg)".into());
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
