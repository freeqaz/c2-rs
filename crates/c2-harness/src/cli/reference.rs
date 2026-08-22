//! The reference-oracle subcommands: `capture`, `compile`, `selftest`,
//! `replay`, `replay-c1`, `diff` and `bench`. Every one of them drives the real
//! toolchain and reports a byte verdict; none of them shares state with the rest.

use std::path::PathBuf;
use std::process::ExitCode;

use c2_core::PortC2;
use c2_harness::toolchain_gate::{toolchain_ready, Cap};
use c2_harness::provenance::Provenance;
use c2_harness::{
    all_fixtures, c1_replay_check, differential_cached, oracle_selftest, C1ReplayReport,
    DiffReport, PortStatus, SelfTestOutcome, SelfTestReport,
};
use c2_il::IL_SUFFIXES;
use c2_obj::{ObjDiff, ObjImage};

use crate::{Args, Arity, Spec};
use crate::cli::util::{first_line, require_cpp, Scratch, CPP_PROFILE_REQUIRES};

static CAPTURE_SPEC: Spec = Spec::new(
    "capture",
    &[
        // `--keep-il DIR` retains the captured bundle for byte inspection — the
        // same affordance `compile --keep-obj` gives for the reference obj, and
        // the only way to design a fixture around a *record-level* `.gl` shape
        // (which name separator introduces a run, where a record's framing
        // starts) without guessing. Gitignored scratch only.
        ("--keep-il", Arity::Value),
        // `--flags-file` / `--cwd`: without them every `.gl` captured for
        // analysis was taken at the `/Ox /GS- /c` default while the obj it was
        // read against had been compiled at the workload's `/O1 /Oi /EHsc /GR …`
        // — and `/Ox` does not imply `/GF`, which is exactly the skew
        // `gl_string_comdat_names` exists to catch.
        ("--flags-file", Arity::Value),
        ("--cwd", Arity::Value),
    ],
)
.requires(CPP_PROFILE_REQUIRES);

pub(crate) fn cmd_capture(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&CAPTURE_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let Some(cpp) = require_cpp(&args) else {
        return ExitCode::from(2);
    };
    let keep_il = args.path("--keep-il");
    let flags_file = args.path("--flags-file");
    let cwd = args.path("--cwd");
    // The profile is read and validated BEFORE the toolchain is located, so a
    // malformed invocation is reported as one on a machine with no compilers at
    // all. Only the capture itself needs the toolchain, and that still degrades
    // to a clean exit 0.
    //
    // No `--flags-file` keeps the default byte-for-byte: `capture_il` is still
    // the call, with `CAPTURE_IL_DEFAULT_FLAGS`. This is a widening.
    let flags: Vec<String> = match &flags_file {
        None => Vec::new(),
        Some(ff) => match std::fs::read_to_string(ff) {
            Ok(t) => t
                .lines()
                .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
                .flat_map(|l| l.split_whitespace().map(String::from))
                .collect(),
            Err(e) => {
                eprintln!("cannot read --flags-file {}: {e}", ff.display());
                return ExitCode::FAILURE;
            }
        },
    };
    if flags_file.is_some() && flags.is_empty() {
        // An empty profile would silently fall back to `cl.exe`'s own defaults —
        // the dropped-flag failure mode again, one layer down.
        eprintln!("--flags-file names no flags; refusing to capture at an unknown profile");
        return ExitCode::from(2);
    }
    let Some(tc) = args.toolchain() else {
        return ExitCode::SUCCESS;
    };
    let w = Scratch::new("capture");
    let captured = match &flags_file {
        None => tc.capture_il(&cpp, &w),
        Some(_) => tc.capture_il_flags(&cpp, &w, &flags, cwd.as_deref()),
    };
    match captured {
        Ok(bundle) => {
            println!("captured IL bundle {} from {}", bundle.base_name, cpp.display());
            // Print the profile that was actually used, always. A flag that is
            // dropped in silence is indistinguishable from a flag that had no
            // effect, and this line is what tells the two apart at the terminal.
            match &flags_file {
                None => println!(
                    "  profile: {} (default — NOT the workload's; /Ox does not imply /GF)",
                    c2_reference::CAPTURE_IL_DEFAULT_FLAGS.join(" ")
                ),
                Some(ff) => println!("  profile: {} (from {})", flags.join(" "), ff.display()),
            }
            if let Some(d) = &cwd {
                println!("  cwd:     {}", d.display());
            }
            for suffix in IL_SUFFIXES {
                let size = bundle.get(suffix).map(|b| b.len()).unwrap_or(0);
                let present = if bundle.get(suffix).is_some() { "ok" } else { "MISSING" };
                println!("  .{suffix:<2}  {size:>7} B  {present}");
            }
            if let Some(dir) = &keep_il {
                let _ = std::fs::create_dir_all(dir);
                for suffix in IL_SUFFIXES {
                    if let Some(bytes) = bundle.get(suffix) {
                        let p = dir.join(format!("{}.{suffix}", bundle.base_name));
                        match std::fs::write(&p, bytes) {
                            Ok(()) => println!("  kept {}", p.display()),
                            Err(e) => eprintln!("  keep-il {} failed: {e}", p.display()),
                        }
                    }
                }
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("capture failed: {e}");
            ExitCode::FAILURE
        }
    }
}

static COMPILE_SPEC: Spec = Spec::new(
    "compile",
    &[
        // `--keep-obj PATH` retains the reference obj for byte classification
        // (the CONST/DERIVED analysis every widening step starts from).
        ("--keep-obj", Arity::Value),
        // Optional real-project compile (same inputs as `c2rs gap`), so the
        // reference obj for a workload TU can be classified, not just a
        // fixture's.
        ("--flags-file", Arity::Value),
        ("--cwd", Arity::Value),
    ],
)
.requires(CPP_PROFILE_REQUIRES);

pub(crate) fn cmd_compile(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&COMPILE_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let Some(cpp) = require_cpp(&args) else {
        return ExitCode::from(2);
    };
    let keep_obj = args.path("--keep-obj");
    let flags_file = args.path("--flags-file");
    let cwd = args.path("--cwd");
    // The profile is read and validated BEFORE `located()`, so a malformed
    // invocation is reported as one on a machine with no compilers at all —
    // which is what lets `tests/cli_flags.rs` catch this class without a
    // toolchain. A *valid* invocation still exits 0 with `SKIP: toolchain
    // absent`.
    let flags: Vec<String> = match &flags_file {
        None => Vec::new(),
        Some(ff) => match std::fs::read_to_string(ff) {
            Ok(t) => t
                .lines()
                .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
                .flat_map(|l| l.split_whitespace().map(String::from))
                .collect(),
            Err(e) => {
                eprintln!("cannot read --flags-file {}: {e}", ff.display());
                return ExitCode::FAILURE;
            }
        },
    };
    if flags_file.is_some() && flags.is_empty() {
        // An empty profile would silently fall back to `cl.exe`'s own defaults —
        // the dropped-profile failure mode one layer down, and the reason
        // `cmd_capture` refuses it too.
        eprintln!("--flags-file names no flags; refusing to compile at an unknown profile");
        return ExitCode::from(2);
    }
    // The `--cwd`-without-`--flags-file` refusal is no longer written here: it
    // is a `requires` edge on the spec, shared by `capture`, `census` and
    // `compile`. Three commands had the same dangling option and only one
    // refused it, which is what a rule expressed in prose rather than in one
    // place gets you.
    let Some(tc) = args.toolchain() else {
        return ExitCode::SUCCESS;
    };
    let w = Scratch::new("compile");
    let out = w.join("out.obj");
    if let Some(ff) = &flags_file {
        let res = tc.capture_reference_with(&cpp.to_string_lossy(), &w, &flags, cwd.as_deref());
        return match res {
            Ok(c) => {
                println!(
                    "compiled {} -> {} bytes (project flags)",
                    cpp.display(),
                    c.ref_obj.len()
                );
                // Print the profile that was actually used, always. A flag
                // dropped in silence is indistinguishable at the terminal from a
                // flag that had no effect, and this line is what tells them
                // apart.
                println!("  profile: {} (from {})", flags.join(" "), ff.display());
                if let Some(d) = &cwd {
                    println!("  cwd:     {}", d.display());
                }
                if let Some(dest) = &keep_obj {
                    if let Some(p) = dest.parent() {
                        let _ = std::fs::create_dir_all(p);
                    }
                    let _ = std::fs::write(dest, c.ref_obj.as_bytes());
                    println!("  kept reference obj at {}", dest.display());
                }
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("compile failed: {e}");
                ExitCode::FAILURE
            }
        };
    }
    match tc.compile_obj(&cpp, &out) {
        Ok(obj) => {
            let ts = obj
                .timestamp()
                .map(|t| format!("0x{t:08x}"))
                .unwrap_or_else(|| "<none>".to_string());
            println!(
                "compiled {} -> {} bytes, TimeDateStamp={}",
                cpp.display(),
                obj.len(),
                ts
            );
            // `Toolchain::compile_obj` hard-codes the same three flags
            // `capture_il` does, which is why the published constant is what is
            // printed here rather than a second literal — one place names them.
            println!(
                "  profile: {} (default — NOT the workload's; /Ox does not imply /GF)",
                c2_reference::CAPTURE_IL_DEFAULT_FLAGS.join(" ")
            );
            if let Some(dest) = &keep_obj {
                if let Some(parent) = dest.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                match std::fs::write(dest, obj.as_bytes()) {
                    Ok(()) => println!("  kept reference obj at {}", dest.display()),
                    Err(e) => eprintln!("  cannot write {}: {e}", dest.display()),
                }
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("compile failed: {e}");
            ExitCode::FAILURE
        }
    }
}

fn selftest_row(r: &SelfTestReport) -> String {
    let name = r
        .cpp
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| r.cpp.display().to_string());
    let detail = match &r.outcome {
        SelfTestOutcome::Pass { obj_len, ex_len } => {
            format!("PASS   obj={obj_len}B ex={ex_len}B")
        }
        SelfTestOutcome::DeterminismFail {
            first_offset,
            len_a,
            len_b,
        } => format!("FAIL   determinism @off {first_offset} (len {len_a} vs {len_b})"),
        SelfTestOutcome::CaptureUnstable { ex_len_a, ex_len_b } => {
            format!("FAIL   capture-unstable (.ex {ex_len_a}B vs {ex_len_b}B)")
        }
        SelfTestOutcome::Error(msg) => format!("ERROR  {}", first_line(msg)),
    };
    format!("  {name:<34} {detail}")
}

/// `selftest` takes any number of `<cpp>` positionals and no options. It used to
/// map `rest` wholesale to fixture paths, so `c2rs selftest --flags-file f.txt`
/// looked for two "fixtures" named `--flags-file` and `f.txt` and failed as a
/// missing file — exit 1, not a usage error.
static SELFTEST_SPEC: Spec = Spec::new("selftest", &[]).positionals(usize::MAX);

pub(crate) fn cmd_selftest(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&SELFTEST_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let Some(tc) = args.toolchain() else {
        return ExitCode::SUCCESS;
    };
    let targets: Vec<PathBuf> = if args.positionals().is_empty() {
        all_fixtures()
    } else {
        args.positionals().iter().map(PathBuf::from).collect()
    };
    if targets.is_empty() {
        eprintln!("no fixtures found");
        return ExitCode::FAILURE;
    }
    let mut all_pass = true;
    // The oracle self-test IS the correctness benchmark, so it names the oracle
    // it ran against (roadmap #48): a stale wibo turns this seam's verdicts over
    // without changing any other number in the report.
    print!("{}", Provenance::collect(&tc, None).render());
    println!("oracle self-test (determinism + capture stability):");
    for cpp in &targets {
        let w = Scratch::new("selftest");
        let report = oracle_selftest(cpp, &tc, &w);
        all_pass &= report.passed();
        println!("{}", selftest_row(&report));
    }
    if all_pass {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// One `<cpp>` and no options. `rest[1..]` used to be discarded without a word,
/// so `c2rs replay <cpp> --flags-file work/dc3-workload/flags.txt` compiled at the
/// `/Ox` default and said nothing — the documented *"`replay` does not take
/// `--flags-file`"* meant "accepts and ignores it", which is the class.
static REPLAY_SPEC: Spec = Spec::new("replay", &[]);

pub(crate) fn cmd_replay(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&REPLAY_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let Some(cpp) = require_cpp(&args) else {
        return ExitCode::from(2);
    };
    let Some(tc) = args.toolchain() else {
        return ExitCode::SUCCESS;
    };
    if let Some(code) = toolchain_ready(&tc, &[Cap::Strace], "needed to keep the IL bundle") {
        return code;
    }
    if let Some(code) = toolchain_ready(&tc, &[Cap::Mingw], "needed to build c2host") {
        return code;
    }
    let w = Scratch::new("replay");
    let out = (|| {
        let captured = tc.capture_reference(&cpp, &w.join("cap"))?;
        // Replay to the SAME /Fo path as the reference for an exact byte compare.
        let ref_path = captured.ref_obj_path.clone();
        let replay = tc.replay(&captured, &w.join("replay_il"), &ref_path)?;
        Ok::<_, std::io::Error>((captured, replay))
    })();
    let code = match out {
        Ok((captured, replay)) => {
            let raw = captured.ref_obj.as_bytes() == replay.as_bytes();
            let norm = matches!(
                ObjImage::diff(&captured.ref_obj, &replay),
                ObjDiff::Identical
            );
            println!(
                "{} -> ref={}B replay={}B  raw_identical={raw}  normalized_identical={norm}",
                cpp.display(),
                captured.ref_obj.len(),
                replay.len(),
            );
            if norm {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(e) => {
            eprintln!("replay failed: {e}");
            ExitCode::FAILURE
        }
    };
    code
}

/// P-F0.1: capture the IL bundle, then reproduce it by driving `c1xx.dll` alone
/// (the front-end analogue of `replay`). Prints a per-file byte verdict; exits
/// non-zero only when a present file failed to reproduce byte-for-byte (a real
/// failure of the front-end replay oracle) or the capture/replay errored.
/// One `<cpp>` and no options. `rest[1..]` used to be discarded without a word,
/// so `c2rs replay-c1 <cpp> --flags-file work/dc3-workload/flags.txt` compiled at the
/// `/Ox` default and said nothing — the documented *"`replay-c1` does not take
/// `--flags-file`"* meant "accepts and ignores it", which is the class.
static REPLAY_C1_SPEC: Spec = Spec::new("replay-c1", &[]);

pub(crate) fn cmd_replay_c1(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&REPLAY_C1_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let Some(cpp) = require_cpp(&args) else {
        return ExitCode::from(2);
    };
    let Some(tc) = args.toolchain() else {
        return ExitCode::SUCCESS;
    };
    if let Some(code) = toolchain_ready(&tc, &[Cap::Mingw], "needed to build the c1host stub") {
        return code;
    }
    if let Some(code) = toolchain_ready(&tc, &[Cap::C1xx], "front end not located") {
        return code;
    }
    let w = Scratch::new("replay-c1");
    let report = c1_replay_check(&cpp, &tc, &w);
    let code = match &report {
        C1ReplayReport::ToolchainAbsent => {
            println!("SKIP: toolchain absent");
            ExitCode::SUCCESS
        }
        C1ReplayReport::Skipped(msg) => {
            println!("SKIP: {}", first_line(msg));
            ExitCode::SUCCESS
        }
        C1ReplayReport::ReferenceError(msg) => {
            eprintln!("c1 replay error: {}", first_line(msg));
            ExitCode::FAILURE
        }
        C1ReplayReport::Replayed { base, files } => {
            let all = report.all_identical();
            println!(
                "{} -> front-end bundle {base}  {}",
                cpp.display(),
                if all { "REPRODUCED byte-exact" } else { "DIVERGED" }
            );
            for f in files {
                let verdict = if f.identical {
                    "identical".to_string()
                } else {
                    format!(
                        "DIFFERS @ {} (cap={}B replay={}B)",
                        f.first_offset.unwrap_or(0),
                        f.cap_len,
                        f.replay_len
                    )
                };
                println!("  .{:<2}  {:>7} B  {verdict}", f.suffix, f.cap_len);
            }
            if all {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
    };
    code
}

/// One `<cpp>` and no options. `rest[1..]` used to be discarded without a word,
/// so `c2rs diff <cpp> --flags-file work/dc3-workload/flags.txt` compiled at the
/// `/Ox` default and said nothing — the documented *"`diff` does not take
/// `--flags-file`"* meant "accepts and ignores it", which is the class.
/// # The capture cache on `diff` (lane `w-gateperf`, 2026-08-18)
///
/// `--cache DIR` / `--no-cache` / `--validate-cache N`, spelled exactly as
/// `c2rs gap` spells them and defaulting the same way — one vocabulary for one
/// cache, because two spellings of one knob is how a gate row ends up with a
/// cache nobody realised was on. Both now resolve through the single
/// [`c2_harness::capture_cache::default_cache_root`], which since 2026-08-22
/// resolves **outside any checkout**; this doc used to name
/// `<main-repo>/work/capture-cache` and that is no longer where it lives.
///
/// **Why it is ON by default here.** `scripts/expr_sweep.sh` spawns one of
/// these per generated case, 19,556 of them per merge gate, and 75 % of each
/// one is a `cl.exe` process tree re-run over source bytes that have not
/// changed. `gap` has been served from this cache since 2026-08-04 and defaults
/// it on; `diff` defaulting it *off* would mean the fast path exists and no
/// standing instrument takes it. See [`c2_harness::differential_cached`] for
/// what is and is not cached — the replay and the port are not.
static DIFF_SPEC: Spec = Spec::new(
    "diff",
    &[
        ("--cache", Arity::Value),
        ("--no-cache", Arity::Flag),
        ("--validate-cache", Arity::Value),
    ],
);

pub(crate) fn cmd_diff(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&DIFF_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let Some(cpp) = require_cpp(&args) else {
        return ExitCode::from(2);
    };
    // Same two refusals `gap` makes, for the same reasons: contradictory knobs
    // are refused rather than silently ordered, and a validator with nothing to
    // validate is refused rather than reported as having run.
    if args.has("--no-cache") && args.get("--cache").is_some() {
        eprintln!("diff: --cache and --no-cache contradict each other; give one");
        return ExitCode::from(2);
    }
    let validate_cache: usize = match args.num("--validate-cache") {
        Ok(v) => v.unwrap_or(0),
        Err(c) => return c,
    };
    if args.has("--no-cache") && validate_cache > 0 {
        eprintln!(
            "diff: --validate-cache has nothing to validate with --no-cache; refusing rather \
             than reporting a validation that cannot run"
        );
        return ExitCode::from(2);
    }
    let Some(tc) = args.toolchain() else {
        return ExitCode::SUCCESS;
    };
    let w = Scratch::new("diff");
    let port = PortC2::default();

    // Build the cache. A cache that cannot be constructed is a MISSING SPEEDUP,
    // never a missing grading: fall back to the uncached path, say so in the
    // outcome word (`cache=off`), and grade exactly what this command has
    // always graded.
    let cache_root: Option<std::path::PathBuf> = if args.has("--no-cache") {
        None
    } else {
        match args.path("--cache") {
            Some(p) => Some(p),
            None => match c2_harness::capture_cache::default_cache_root() {
                Ok(p) => Some(p),
                Err(e) => {
                    eprintln!("diff: no capture cache ({e}); capturing for real");
                    None
                }
            },
        }
    };
    let cache = match &cache_root {
        None => None,
        Some(root) => match c2_harness::capture_cache::CaptureCache::new(
            root.clone(),
            &tc,
            None,
            validate_cache,
        ) {
            Ok(c) => Some(c),
            Err(e) => {
                eprintln!("diff: capture cache unavailable ({e}); capturing for real");
                None
            }
        },
    };
    let (report, outcome) = differential_cached(&cpp, &tc, &port, &w, cache.as_ref());
    // `foreign` is a REFUSED entry, not an outcome — the call reports it as a
    // miss and records the refusal in the stats. Surface it, because "the cache
    // held entries it would not serve" is exactly the kind of thing that gets
    // paid for silently in re-captures.
    let stats = cache.as_ref().map(|c| c.stats());
    let cache_word = match (&cache, outcome) {
        (None, _) if cache_root.is_none() => "off",
        (None, _) => "off",
        (Some(_), c2_harness::capture_cache::CacheOutcome::Hit) => "hit",
        (Some(_), c2_harness::capture_cache::CacheOutcome::Miss) => {
            if stats.as_ref().map(|s| s.foreign).unwrap_or(0) > 0 {
                "foreign"
            } else {
                "miss"
            }
        }
        (Some(_), c2_harness::capture_cache::CacheOutcome::Validated) => "validated",
        (Some(_), c2_harness::capture_cache::CacheOutcome::Poisoned) => "poisoned",
        (Some(_), c2_harness::capture_cache::CacheOutcome::Bypassed) => "bypass",
    };
    if let Some(s) = &stats {
        for d in s.poison_detail.iter().chain(s.foreign_detail.iter()) {
            eprintln!("diff: CACHE {d}");
        }
    }
    let line = match &report {
        DiffReport::ToolchainAbsent => "ToolchainAbsent".to_string(),
        DiffReport::Skipped(msg) => format!("SKIP: {}", first_line(msg)),
        DiffReport::ReferenceError(msg) => format!("ReferenceError: {}", first_line(msg)),
        DiffReport::ReferenceReplayMismatch {
            first_offset,
            ref_len,
            replay_len,
        } => format!(
            "ReferenceReplay=MISMATCH @ offset {first_offset} (ref={ref_len}B replay={replay_len}B)"
        ),
        DiffReport::ReferenceReplayByteExact {
            ref_len,
            replay_len,
            port,
        } => {
            let port_str = match port {
                PortStatus::NotImplemented(_) => "NotImplemented".to_string(),
                PortStatus::Match => "Match".to_string(),
                PortStatus::Mismatch { first_offset } => {
                    format!("Mismatch @ offset {first_offset}")
                }
            };
            format!(
                "ReferenceReplay=ByteExact (ref={ref_len}B replay={replay_len}B)  Port={port_str}"
            )
        }
    };
    // The cache word is APPENDED to the verdict line, never printed on its own.
    // `scripts/expr_sweep.sh` classifies `$(c2rs diff … 2>&1 | tail -1)`, so a
    // separate line would either be swallowed or would become the line the
    // classifier reads. A suffix keeps every existing `*"Port=Match"*` arm
    // matching and gives the driver something to count.
    println!("{} -> {}  cache={}", cpp.display(), line, cache_word);
    // A poisoned or refused entry means the cache handed back bytes that are
    // not what this toolchain produces at this path. That is an instrument
    // failure in the oracle's own supply line, so it FAILS rather than being
    // reported and passed over.
    if cache_word == "poisoned" || cache_word == "foreign" {
        return ExitCode::FAILURE;
    }
    // A byte-exact reference replay is the pass condition, and the port may be
    // Match or NotImplemented depending on the TU — both, and clean skips, are
    // success for scripting.
    //
    // `Port=Mismatch` is NOT. The doctrine is that a mismatch is an alarm rather
    // than a gap: the port emitted bytes and they were wrong. This is the per-rung
    // acceptance gate, so the alarm needs an exit code and not just a line of
    // stdout that a `tail -1` may or may not be read by a human.
    match &report {
        DiffReport::ReferenceReplayMismatch { .. } | DiffReport::ReferenceError(_) => {
            ExitCode::FAILURE
        }
        DiffReport::ReferenceReplayByteExact { port: PortStatus::Mismatch { .. }, .. } => {
            ExitCode::FAILURE
        }
        _ => ExitCode::SUCCESS,
    }
}

/// `bench` takes nothing. The dispatcher used to call it as `cmd_bench()`, so
/// every argument after `bench` was discarded **by the dispatcher**, one level
/// above any handler that could have refused it — the same class, at the only
/// site where the handler never even saw the arguments.
static BENCH_SPEC: Spec = Spec::new("bench", &[]).positionals(0);

pub(crate) fn cmd_bench(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&BENCH_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let Some(tc) = args.toolchain() else {
        return ExitCode::SUCCESS;
    };
    let targets = all_fixtures();
    if targets.is_empty() {
        eprintln!("no fixtures found under {}", c2_harness::fixtures_dir().display());
        return ExitCode::FAILURE;
    }
    println!("bench: oracle self-test across {} fixture(s)", targets.len());
    let (mut pass, mut fail, mut err) = (0u32, 0u32, 0u32);
    for cpp in &targets {
        let w = Scratch::new("bench");
        let report = oracle_selftest(cpp, &tc, &w);
        match &report.outcome {
            SelfTestOutcome::Pass { .. } => pass += 1,
            SelfTestOutcome::Error(_) => err += 1,
            _ => fail += 1,
        }
        println!("{}", selftest_row(&report));
    }
    println!("\nsummary: {pass} pass, {fail} fail, {err} error (of {})", targets.len());
    if fail == 0 && err == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
