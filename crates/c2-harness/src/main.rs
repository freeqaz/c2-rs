//! `c2rs` — CLI over the differential harness. std only (no clap): args are
//! parsed by hand. Every subcommand degrades to "SKIP: toolchain absent" when
//! `Toolchain::locate()` is `None` — it never panics on a missing toolchain.
//!
//! Subcommands:
//!   capture <cpp>       capture IL, print the 5 file sizes
//!   compile <cpp>       reference obj, print size + timestamp
//!   selftest [<cpp>...] oracle self-test over the given TUs (or all fixtures)
//!   replay <cpp>        P0.1: capture + standalone-c2 replay, print byte-match
//!   replay-c1 <cpp>     P-F0.1: capture + standalone-c1 (front-end) replay, per-file byte verdict
//!   diff <cpp>          full differential (ReferenceReplay=ByteExact, Port=Match|NotImplemented)
//!   bench               selftest across all fixtures/cpp/*.cpp, summary counts
//!   perf                IL-bundle->obj latency: native port vs standalone c2
//!   perf-scale          IL-bundle->obj throughput vs concurrency (port vs c2)
//!   corpus <sub>        P1.2 corpus generator (gen / sample / stats)

use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use c2_core::PortC2;
use c2_harness::corpus::{self, CorpusConfig};
use c2_harness::retrieval;
use c2_harness::{
    all_fixtures, c1_replay_check, differential, oracle_selftest, C1ReplayReport, DiffReport,
    PortStatus, SelfTestOutcome, SelfTestReport,
};
use c2_il::IL_SUFFIXES;
use c2_obj::{ObjDiff, ObjImage};
use c2_reference::Toolchain;

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn scratch(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let d = std::env::temp_dir().join(format!("c2rs-cli-{tag}-{}-{}-{}", std::process::id(), nanos, n));
    let _ = std::fs::create_dir_all(&d);
    d
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args.first().map(String::as_str).unwrap_or("help");
    let rest = &args[args.len().min(1)..];

    match cmd {
        "capture" => cmd_capture(rest),
        "compile" => cmd_compile(rest),
        "selftest" => cmd_selftest(rest),
        "replay" => cmd_replay(rest),
        "replay-c1" => cmd_replay_c1(rest),
        "diff" => cmd_diff(rest),
        "bench" => cmd_bench(),
        "perf" => cmd_perf(rest),
        "perf-scale" => cmd_perf_scale(rest),
        "corpus" => cmd_corpus(rest),
        "retrieve" => cmd_retrieve(rest),
        "search" => cmd_search(rest),
        "help" | "-h" | "--help" => {
            print_usage();
            ExitCode::SUCCESS
        }
        other => {
            eprintln!("unknown subcommand: {other}\n");
            print_usage();
            ExitCode::from(2)
        }
    }
}

fn print_usage() {
    eprintln!(
        "c2rs — differential harness for the c2.dll native port\n\
         \n\
         USAGE:\n\
         \x20 c2rs capture <cpp>        capture IL, print the 5 file sizes\n\
         \x20 c2rs compile <cpp>        reference obj, print size + timestamp\n\
         \x20 c2rs selftest [<cpp>...]  oracle self-test (determinism + capture stability)\n\
         \x20 c2rs replay <cpp>         P0.1: capture + standalone-c2 replay, byte-match verdict\n\
         \x20 c2rs replay-c1 <cpp>      P-F0.1: capture + standalone-c1 (front-end) replay, per-file byte verdict\n\
         \x20 c2rs diff <cpp>           full differential (ReferenceReplay=ByteExact, Port=Match|NotImplemented)\n\
         \x20 c2rs bench                selftest across all fixtures/cpp/*.cpp\n\
         \x20 c2rs perf [opts]          IL-bundle->obj latency: native port vs standalone c2\n\
         \x20 c2rs perf-scale [opts]    IL-bundle->obj throughput vs concurrency (port vs c2)\n\
         \x20 c2rs corpus gen [opts]    P1.2: generate a (source,IL,obj) triple corpus\n\
         \x20 c2rs corpus sample [dir]  write the portable synthetic sample corpus\n\
         \x20 c2rs corpus stats <dir>   summarize a corpus manifest\n\
         \x20 c2rs retrieve index <dir> P1.3: obj-retrieval structure of a corpus\n\
         \x20 c2rs retrieve eval <dir>  P1.3: obj->IL retrieval baseline, recall@k\n\
         \x20 c2rs search solve <cpp>   T-A: solve one d=1 instance from a fixture, byte-exact\n\
         \x20 c2rs search eval [opts]   T-A: IL-space solve-rate over fixtures\n\
         \x20 c2rs search from-retrieval <corpus-dir>  T-A: from-unrelated-seed (P1.3-seeded) solve-rate\n\
         \n\
         perf options: --port-iters N --ref-iters N --fixtures a.cpp,b.cpp\n\
         perf-scale options: --fixture X.cpp --conc 1,2,4,8 --port-secs F --ref-secs F --csv PATH\n\
         corpus gen options: --seed N --count N --out DIR --timeout SECS\n\
         retrieve eval options: --split held-out|loo --query-div N --k 1,5,10\n\
         search options: --d 1|2|3 --moves full|length --steps N --compiles N --beam K --timeout SECS\n\
         \n\
         Toolchain: compilers/ via scripts/fetch_compilers.sh (or C2RS_COMPILERS /\n\
         C2RS_CL_EXE / C2RS_C2_DLL / C2RS_C1XX_DLL), wibo via C2RS_WIBO, sibling\n\
         ../wibo build, or PATH. Absent toolchain -> clean SKIP."
    );
}

/// Locate the toolchain or print the standard skip line. Returns `None` (and the
/// caller should exit SUCCESS) when absent.
fn located() -> Option<Toolchain> {
    match Toolchain::locate() {
        Some(tc) => Some(tc),
        None => {
            println!("SKIP: toolchain absent");
            None
        }
    }
}

fn require_cpp(rest: &[String]) -> Option<PathBuf> {
    match rest.first() {
        Some(p) => Some(PathBuf::from(p)),
        None => {
            eprintln!("error: expected a <cpp> path");
            None
        }
    }
}

fn cmd_capture(rest: &[String]) -> ExitCode {
    let Some(cpp) = require_cpp(rest) else {
        return ExitCode::from(2);
    };
    let Some(tc) = located() else {
        return ExitCode::SUCCESS;
    };
    let w = scratch("capture");
    match tc.capture_il(&cpp, &w) {
        Ok(bundle) => {
            println!("captured IL bundle {} from {}", bundle.base_name, cpp.display());
            for suffix in IL_SUFFIXES {
                let size = bundle.get(suffix).map(|b| b.len()).unwrap_or(0);
                let present = if bundle.get(suffix).is_some() { "ok" } else { "MISSING" };
                println!("  .{suffix:<2}  {size:>7} B  {present}");
            }
            let _ = std::fs::remove_dir_all(&w);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("capture failed: {e}");
            let _ = std::fs::remove_dir_all(&w);
            ExitCode::FAILURE
        }
    }
}

fn cmd_compile(rest: &[String]) -> ExitCode {
    let Some(cpp) = require_cpp(rest) else {
        return ExitCode::from(2);
    };
    let Some(tc) = located() else {
        return ExitCode::SUCCESS;
    };
    let w = scratch("compile");
    let out = w.join("out.obj");
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
            let _ = std::fs::remove_dir_all(&w);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("compile failed: {e}");
            let _ = std::fs::remove_dir_all(&w);
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

fn first_line(s: &str) -> &str {
    s.lines().next().unwrap_or(s)
}

fn cmd_selftest(rest: &[String]) -> ExitCode {
    let Some(tc) = located() else {
        return ExitCode::SUCCESS;
    };
    let targets: Vec<PathBuf> = if rest.is_empty() {
        all_fixtures()
    } else {
        rest.iter().map(PathBuf::from).collect()
    };
    if targets.is_empty() {
        eprintln!("no fixtures found");
        return ExitCode::FAILURE;
    }
    let mut all_pass = true;
    println!("oracle self-test (determinism + capture stability):");
    for cpp in &targets {
        let w = scratch("selftest");
        let report = oracle_selftest(cpp, &tc, &w);
        all_pass &= report.passed();
        println!("{}", selftest_row(&report));
        let _ = std::fs::remove_dir_all(&w);
    }
    if all_pass {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn cmd_replay(rest: &[String]) -> ExitCode {
    let Some(cpp) = require_cpp(rest) else {
        return ExitCode::from(2);
    };
    let Some(tc) = located() else {
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
    let w = scratch("replay");
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
    let _ = std::fs::remove_dir_all(&w);
    code
}

/// P-F0.1: capture the IL bundle, then reproduce it by driving `c1xx.dll` alone
/// (the front-end analogue of `replay`). Prints a per-file byte verdict; exits
/// non-zero only when a present file failed to reproduce byte-for-byte (a real
/// failure of the front-end replay oracle) or the capture/replay errored.
fn cmd_replay_c1(rest: &[String]) -> ExitCode {
    let Some(cpp) = require_cpp(rest) else {
        return ExitCode::from(2);
    };
    let Some(tc) = located() else {
        return ExitCode::SUCCESS;
    };
    if !tc.has_mingw() {
        println!("SKIP: i686-w64-mingw32-gcc absent (needed to build the c1host stub)");
        return ExitCode::SUCCESS;
    }
    if !tc.has_c1xx() {
        println!("SKIP: c1xx.dll absent (front end not located)");
        return ExitCode::SUCCESS;
    }
    let w = scratch("replay-c1");
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
    let _ = std::fs::remove_dir_all(&w);
    code
}

fn cmd_diff(rest: &[String]) -> ExitCode {
    let Some(cpp) = require_cpp(rest) else {
        return ExitCode::from(2);
    };
    let Some(tc) = located() else {
        return ExitCode::SUCCESS;
    };
    let w = scratch("diff");
    let port = PortC2::default();
    let report = differential(&cpp, &tc, &port, &w);
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
    println!("{} -> {}", cpp.display(), line);
    let _ = std::fs::remove_dir_all(&w);
    // A byte-exact reference replay is the pass condition here; the port may be
    // Match or NotImplemented depending on the TU. Treat both (and clean skips)
    // as success for scripting — only a reference-side failure is non-zero.
    match &report {
        DiffReport::ReferenceReplayMismatch { .. } | DiffReport::ReferenceError(_) => {
            ExitCode::FAILURE
        }
        _ => ExitCode::SUCCESS,
    }
}

fn cmd_bench() -> ExitCode {
    let Some(tc) = located() else {
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
        let w = scratch("bench");
        let report = oracle_selftest(cpp, &tc, &w);
        match &report.outcome {
            SelfTestOutcome::Pass { .. } => pass += 1,
            SelfTestOutcome::Error(_) => err += 1,
            _ => fail += 1,
        }
        println!("{}", selftest_row(&report));
        let _ = std::fs::remove_dir_all(&w);
    }
    println!("\nsummary: {pass} pass, {fail} fail, {err} error (of {})", targets.len());
    if fail == 0 && err == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

// ---------------------------------------------------------------------------
// perf — angle-H latency: native port vs standalone c2 (IL bundle -> obj)
// ---------------------------------------------------------------------------

use c2_harness::perf::{self, fmt_dur, PerfConfig, PortPerf};

fn cmd_perf(rest: &[String]) -> ExitCode {
    let Some(tc) = located() else {
        return ExitCode::SUCCESS;
    };
    if !tc.has_strace() || !tc.has_mingw() {
        println!("SKIP: strace / i686-w64-mingw32-gcc absent (needed for standalone-c2 replay)");
        return ExitCode::SUCCESS;
    }

    let mut cfg = PerfConfig::default();
    if let Some(v) = opt(rest, "--port-iters").and_then(|s| s.parse().ok()) {
        cfg.port_iters = v;
    }
    if let Some(v) = opt(rest, "--ref-iters").and_then(|s| s.parse().ok()) {
        cfg.ref_iters = v;
    }
    let targets: Vec<PathBuf> = match opt(rest, "--fixtures") {
        Some(list) => list
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| {
                let p = PathBuf::from(s);
                if p.exists() {
                    p
                } else {
                    c2_harness::fixtures_dir().join(s)
                }
            })
            .collect(),
        None => all_fixtures(),
    };
    if targets.is_empty() {
        eprintln!("no fixtures to benchmark");
        return ExitCode::FAILURE;
    }

    println!(
        "perf: IL-bundle -> obj latency, native port vs standalone c2 (reference)\n\
         \x20 {} fixture(s), port_iters={}, ref_iters={}   (both produce the SAME obj)\n",
        targets.len(),
        cfg.port_iters,
        cfg.ref_iters,
    );
    println!(
        "  {:<28} {:>7}  {:>13}  {:>13}  {:>11}  {}",
        "fixture", "obj", "ref median", "port median", "speedup", "port"
    );

    let mut rows = Vec::new();
    let mut errors = 0usize;
    for cpp in &targets {
        let name = cpp
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| cpp.display().to_string());
        let w = scratch("perf");
        match perf::bench_fixture(&tc, cpp, &cfg, &w) {
            Ok(r) => {
                let (port_med, speedup, status) = match r.port {
                    PortPerf::Match { median, .. } => (
                        fmt_dur(median),
                        r.speedup()
                            .map(|s| format!("{s:.0}x"))
                            .unwrap_or_else(|| "-".into()),
                        "Match".to_string(),
                    ),
                    PortPerf::NotImplemented => {
                        ("-".into(), "-".into(), "NotImplemented".to_string())
                    }
                    PortPerf::Mismatch { first_offset } => {
                        ("-".into(), "-".into(), format!("Mismatch@{first_offset}"))
                    }
                };
                // The P0.1 invariant should always hold; flag it loudly if not.
                let flag = if r.ref_exact { "" } else { "  [!ref-replay-inexact]" };
                println!(
                    "  {:<28} {:>6}B  {:>13}  {:>13}  {:>11}  {}{}",
                    name,
                    r.obj_len,
                    fmt_dur(r.ref_median),
                    port_med,
                    speedup,
                    status,
                    flag,
                );
                rows.push(r);
            }
            Err(e) => {
                println!("  {name:<28} ERROR {}", first_line(&e.to_string()));
                errors += 1;
            }
        }
        let _ = std::fs::remove_dir_all(&w);
    }

    let report = perf::PerfReport {
        rows,
        port_iters: cfg.port_iters,
        ref_iters: cfg.ref_iters,
    };
    let (matched, mismatched, ni) = report.tally();
    let ref_inexact = report.rows.iter().filter(|r| !r.ref_exact).count();
    println!(
        "\nsummary: {matched} port Match, {mismatched} mismatch, {ni} not-implemented (of {})",
        report.rows.len()
    );
    match report.geomean_speedup() {
        Some(g) => println!(
            "  geomean speedup over the {matched} matched fixture(s): {g:.0}x faster than standalone c2"
        ),
        None => println!("  no matched fixtures — no speedup to report"),
    }
    // Convention (as in `diff`): the reference is the sole judge, so a port
    // Match/Mismatch/NotImplemented is per-TU reporting, not a harness failure.
    // Only a capture/replay error or a broken P0.1 replay (ref-replay-inexact)
    // is a hard failure of the benchmark itself.
    if errors > 0 || ref_inexact > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn default_concurrencies() -> Vec<usize> {
    // Powers of two up to the machine's parallelism (capped at 32 for the graph).
    let max = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(8)
        .min(32);
    let mut v = Vec::new();
    let mut c = 1;
    while c <= max {
        v.push(c);
        c *= 2;
    }
    if *v.last().unwrap_or(&0) != max {
        v.push(max);
    }
    v
}

fn cmd_perf_scale(rest: &[String]) -> ExitCode {
    let Some(tc) = located() else {
        return ExitCode::SUCCESS;
    };
    if !tc.has_strace() || !tc.has_mingw() {
        println!("SKIP: strace / i686-w64-mingw32-gcc absent (needed for standalone-c2 replay)");
        return ExitCode::SUCCESS;
    }

    let fixture = opt(rest, "--fixture")
        .map(|s| {
            let p = PathBuf::from(s);
            if p.exists() {
                p
            } else {
                c2_harness::fixtures_dir().join(s)
            }
        })
        .unwrap_or_else(|| c2_harness::fixtures_dir().join("mvp_add3.cpp"));

    let mut cfg = perf::ScaleConfig::default();
    cfg.concurrencies = match opt(rest, "--conc") {
        Some(list) => list
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .filter(|&c: &usize| c >= 1)
            .collect(),
        None => default_concurrencies(),
    };
    if cfg.concurrencies.is_empty() {
        eprintln!("no valid --conc values");
        return ExitCode::from(2);
    }
    if let Some(v) = opt(rest, "--port-secs").and_then(|s| s.parse().ok()) {
        cfg.port_secs = v;
    }
    if let Some(v) = opt(rest, "--ref-secs").and_then(|s| s.parse().ok()) {
        cfg.ref_secs = v;
    }

    let name = fixture
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| fixture.display().to_string());
    println!(
        "perf-scale: throughput (objs/sec) vs concurrency on {name}\n\
         \x20 concurrencies={:?}  port_secs={}  ref_secs={}\n",
        cfg.concurrencies, cfg.port_secs, cfg.ref_secs
    );

    let w = scratch("perf-scale");
    let (points, obj_len) = match perf::scale_measure(&tc, &fixture, &cfg, &w) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("perf-scale failed: {}", first_line(&e.to_string()));
            let _ = std::fs::remove_dir_all(&w);
            return ExitCode::FAILURE;
        }
    };
    let _ = std::fs::remove_dir_all(&w);

    println!("  obj size: {obj_len} B (both sides produce this exact obj)\n");
    println!(
        "  {:>5}  {:>16}  {:>16}  {:>10}",
        "conc", "port objs/sec", "c2 objs/sec", "speedup"
    );
    for p in &points {
        println!(
            "  {:>5}  {:>16.0}  {:>16.1}  {:>9.0}x",
            p.concurrency,
            p.port_ops,
            p.ref_ops,
            p.speedup()
        );
    }

    // Emit CSV for the README plot when asked.
    if let Some(path) = opt(rest, "--csv") {
        let mut csv = String::from("concurrency,port_ops_per_sec,ref_ops_per_sec\n");
        for p in &points {
            csv.push_str(&format!("{},{:.3},{:.3}\n", p.concurrency, p.port_ops, p.ref_ops));
        }
        match std::fs::write(path, csv) {
            Ok(()) => println!("\nwrote CSV: {path}"),
            Err(e) => {
                eprintln!("could not write CSV {path}: {e}");
                return ExitCode::FAILURE;
            }
        }
    }
    ExitCode::SUCCESS
}

fn cmd_corpus(rest: &[String]) -> ExitCode {
    let sub = rest.first().map(String::as_str).unwrap_or("");
    let rest = &rest[rest.len().min(1)..];
    match sub {
        "gen" => cmd_corpus_gen(rest),
        "sample" => cmd_corpus_sample(rest),
        "stats" => cmd_corpus_stats(rest),
        _ => {
            eprintln!("usage: c2rs corpus <gen|sample|stats> [opts]");
            ExitCode::from(2)
        }
    }
}

/// Parse `--key value` pairs (and a leading positional) from `rest`.
fn opt<'a>(rest: &'a [String], key: &str) -> Option<&'a str> {
    rest.iter()
        .position(|a| a == key)
        .and_then(|i| rest.get(i + 1))
        .map(String::as_str)
}

fn cmd_corpus_gen(rest: &[String]) -> ExitCode {
    let mut cfg = CorpusConfig::default();
    if let Some(v) = opt(rest, "--seed") {
        cfg.seed = v.parse().unwrap_or(cfg.seed);
    }
    if let Some(v) = opt(rest, "--count") {
        cfg.count = v.parse().unwrap_or(cfg.count);
    }
    if let Some(v) = opt(rest, "--timeout") {
        if let Ok(s) = v.parse::<u64>() {
            cfg.timeout = Duration::from_secs(s);
        }
    }
    let out = opt(rest, "--out")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("corpus"));

    let Some(tc) = located() else {
        return ExitCode::SUCCESS;
    };
    if !tc.has_strace() {
        println!("SKIP: strace absent (needed to keep the IL bundle)");
        return ExitCode::SUCCESS;
    }
    println!(
        "corpus gen: seed={} count={} timeout={}s -> {}",
        cfg.seed,
        cfg.count,
        cfg.timeout.as_secs(),
        out.display()
    );
    match corpus::generate(&out, &tc, &cfg) {
        Ok(s) => {
            println!(
                "  {} ok, {} codec_fail, {} timeout, {} error ({} distinct sources, {} rows)",
                s.ok,
                s.codec_fail,
                s.timeout,
                s.error,
                s.distinct_sources,
                s.total()
            );
            if s.ok == 0 {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(e) => {
            eprintln!("corpus gen failed: {e}");
            ExitCode::FAILURE
        }
    }
}

fn cmd_corpus_sample(rest: &[String]) -> ExitCode {
    let out = rest
        .first()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("crates/c2-harness/tests/corpus_sample"));
    match corpus::write_synthetic_sample(&out) {
        Ok(s) => {
            println!("wrote synthetic sample: {} triples -> {}", s.ok, out.display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("corpus sample failed: {e}");
            ExitCode::FAILURE
        }
    }
}

// ---------------------------------------------------------------------------
// P1.3 retrieval baseline
// ---------------------------------------------------------------------------

fn cmd_retrieve(rest: &[String]) -> ExitCode {
    let sub = rest.first().map(String::as_str).unwrap_or("");
    let rest = &rest[rest.len().min(1)..];
    match sub {
        "eval" => cmd_retrieve_eval(rest),
        "index" => cmd_retrieve_index(rest),
        _ => {
            eprintln!("usage: c2rs retrieve <eval|index> <corpus-dir> [opts]");
            ExitCode::from(2)
        }
    }
}

fn parse_ks(rest: &[String]) -> Vec<usize> {
    opt(rest, "--k")
        .map(|s| s.split(',').filter_map(|t| t.trim().parse().ok()).collect::<Vec<usize>>())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| vec![1, 5, 10])
}

fn cmd_retrieve_index(rest: &[String]) -> ExitCode {
    let Some(dir) = rest.first().filter(|s| !s.starts_with("--")).map(PathBuf::from) else {
        eprintln!("usage: c2rs retrieve index <corpus-dir>");
        return ExitCode::from(2);
    };
    let items = match retrieval::load_items(&dir) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("could not load corpus: {e}");
            return ExitCode::FAILURE;
        }
    };
    if items.is_empty() {
        eprintln!("no `ok` triples under {}", dir.display());
        return ExitCode::FAILURE;
    }
    // Behavioral (.text), strict full-obj, and source collision structure.
    let text_classes = class_sizes(items.iter().map(|i| i.text_key.clone()));
    let full_classes = class_sizes(items.iter().map(|i| i.full_key.clone()));
    let src_classes = class_sizes(items.iter().map(|i| i.src_key.clone()));
    let n = items.len();
    let in_multi = |m: &std::collections::BTreeMap<String, usize>| {
        m.values().filter(|&&c| c > 1).sum::<usize>()
    };
    println!("retrieval index: {} ok triples from {}", n, dir.display());
    println!(
        "  distinct sources     : {:>5}   ({} rows in a shared-source class)",
        src_classes.len(),
        in_multi(&src_classes)
    );
    println!(
        "  distinct .text (code): {:>5}   ({} rows / {:.1}% in a code-collision class \u{2265}2)",
        text_classes.len(),
        in_multi(&text_classes),
        in_multi(&text_classes) as f64 / n as f64 * 100.0
    );
    println!(
        "  distinct obj_sha_norm: {:>5}   ({} rows in a full-obj class \u{2265}2; path-polluted)",
        full_classes.len(),
        in_multi(&full_classes)
    );
    let biggest = text_classes.values().copied().max().unwrap_or(0);
    println!("  largest code class   : {biggest} rows");
    println!(
        "  feature              : 256-bin L1-normalized .text byte histogram, cosine NN"
    );
    ExitCode::SUCCESS
}

fn class_sizes<I: Iterator<Item = String>>(keys: I) -> std::collections::BTreeMap<String, usize> {
    let mut m = std::collections::BTreeMap::new();
    for k in keys {
        *m.entry(k).or_insert(0) += 1;
    }
    m
}

fn cmd_retrieve_eval(rest: &[String]) -> ExitCode {
    let Some(dir) = rest.first().filter(|s| !s.starts_with("--")).map(PathBuf::from) else {
        eprintln!("usage: c2rs retrieve eval <corpus-dir> [--split held-out|loo] [--query-div N] [--k 1,5,10]");
        return ExitCode::from(2);
    };
    let ks = parse_ks(rest);
    let split = opt(rest, "--split").unwrap_or("held-out");
    let query_div: u64 = opt(rest, "--query-div").and_then(|s| s.parse().ok()).unwrap_or(5);

    let items = match retrieval::load_items(&dir) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("could not load corpus: {e}");
            return ExitCode::FAILURE;
        }
    };
    if items.is_empty() {
        eprintln!("no `ok` triples under {}", dir.display());
        return ExitCode::FAILURE;
    }

    let (report, mode) = match split {
        "loo" => (retrieval::evaluate(&items, &items, &ks, true), "leave-one-out".to_string()),
        _ => {
            let (q, idx) = retrieval::split_held_out(items, query_div);
            let m = format!("held-out (query = 1/{query_div} by sha256(id))");
            (retrieval::evaluate(&q, &idx, &ks, false), m)
        }
    };

    println!("P1.3 retrieval baseline — {}", dir.display());
    println!("  split : {mode}");
    println!("  query : {}   index: {}", report.n_query, report.n_index);
    println!(
        "  answerable queries: obj-text(.text)={}  obj-full(sha_norm)={}  exact-source={}",
        report.answerable_text, report.answerable_full, report.answerable_exact
    );
    println!("  recall@k (fraction of queries with a correct IL in top-k):");
    println!("    k    obj-equiv(.text)   obj-full(strict)   exact-source     random(.text)");
    for row in &report.rows {
        println!(
            "    {:<4} {:>10.2}%      {:>10.2}%      {:>10.2}%     {:>10.4}%",
            row.k,
            row.obj_text * 100.0,
            row.obj_full * 100.0,
            row.exact * 100.0,
            row.random_text * 100.0,
        );
    }
    ExitCode::SUCCESS
}

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

fn cmd_search(rest: &[String]) -> ExitCode {
    let sub = rest.first().map(String::as_str).unwrap_or("");
    let rest = &rest[rest.len().min(1)..];
    match sub {
        "solve" => cmd_search_solve(rest),
        "eval" => cmd_search_eval(rest),
        "from-retrieval" => cmd_search_from_retrieval(rest),
        "from-lifter" => cmd_search_from_lifter(rest),
        _ => {
            eprintln!("usage: c2rs search <solve <cpp>|eval|from-retrieval <corpus-dir>|from-lifter <corpus-dir> --gens <jsonl>> [--d 1] [--moves full|length] [--steps N] [--compiles N] [--beam K] [--timeout SECS]");
            ExitCode::from(2)
        }
    }
}

fn search_moveset(rest: &[String]) -> MoveSet {
    let mut m = match opt(rest, "--moves") {
        Some("length") => MoveSet::length_only(),
        _ => MoveSet::default(),
    };
    // On the real obj-judged path, widen/narrow is obj-INVISIBLE (P0.6a A: c2
    // re-optimizes a re-widthed literal to byte-identical code), so it can never
    // reach a new obj — it only floods the beam with gradient-tied duplicate
    // models that crowd out productive (structure-changing) moves. Drop it from
    // the search moveset unless explicitly re-enabled. (The mock-scorer unit tests
    // keep it via `MoveSet::default()`, where it IS `.ex`-visible.)
    if opt(rest, "--keep-widen").is_none() {
        m.widen_narrow = false;
    }
    m
}

fn search_budget(rest: &[String]) -> Budget {
    let mut b = Budget::default();
    if let Some(v) = opt(rest, "--steps").and_then(|s| s.parse().ok()) {
        b.max_steps = v;
    }
    if let Some(v) = opt(rest, "--compiles").and_then(|s| s.parse().ok()) {
        b.max_compiles = v;
    }
    if let Some(v) = opt(rest, "--beam").and_then(|s| s.parse().ok()) {
        b.beam_width = v;
    }
    b
}

fn search_perturbs(rest: &[String]) -> Vec<(Perturb, usize)> {
    // The obj-changing families at d=1, plus AddTerm at d=2 (a gradient-guided
    // two-move recovery) when --d 2 is requested. WidenLit is obj-invisible on
    // the real path (P0.6a A), so it is not in the roster.
    let d: usize = opt(rest, "--d").and_then(|s| s.parse().ok()).unwrap_or(1);
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
    v
}

fn search_timeout(rest: &[String]) -> Duration {
    Duration::from_secs(opt(rest, "--timeout").and_then(|s| s.parse().ok()).unwrap_or(60))
}

fn cmd_search_solve(rest: &[String]) -> ExitCode {
    let Some(cpp) = rest.first().filter(|s| !s.starts_with("--")).map(PathBuf::from) else {
        eprintln!("usage: c2rs search solve <cpp> [--moves full|length]");
        return ExitCode::from(2);
    };
    let Some(tc) = located() else {
        return ExitCode::SUCCESS;
    };
    if !tc.has_strace() || !tc.has_mingw() {
        println!("SKIP: strace / i686-w64-mingw32-gcc absent (needed for replay)");
        return ExitCode::SUCCESS;
    }
    let moves = search_moveset(rest);
    let budget = search_budget(rest);
    let timeout = search_timeout(rest);
    let w = scratch("search-solve");
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
    let _ = std::fs::remove_dir_all(&w);
    code
}

fn cmd_search_eval(rest: &[String]) -> ExitCode {
    let Some(tc) = located() else {
        return ExitCode::SUCCESS;
    };
    if !tc.has_strace() || !tc.has_mingw() {
        println!("SKIP: strace / i686-w64-mingw32-gcc absent (needed for replay)");
        return ExitCode::SUCCESS;
    }
    let moves = search_moveset(rest);
    let budget = search_budget(rest);
    let perturbs = search_perturbs(rest);
    let timeout = search_timeout(rest);
    let fixtures: Vec<PathBuf> = SEARCH_FIXTURES
        .iter()
        .map(|n| c2_harness::fixtures_dir().join(n))
        .collect();

    let w = scratch("search-eval");
    println!(
        "T-A IL-space solve-rate: {} fixtures x {} perturbation families, moves={}, budget steps={} compiles={}",
        fixtures.len(),
        perturbs.len(),
        opt(rest, "--moves").unwrap_or("full"),
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
    let _ = std::fs::remove_dir_all(&w);
    if attempted > 0 && solved == attempted {
        ExitCode::SUCCESS
    } else if solved > 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn cmd_search_from_retrieval(rest: &[String]) -> ExitCode {
    let Some(dir) = rest.first().filter(|s| !s.starts_with("--")).map(PathBuf::from) else {
        eprintln!(
            "usage: c2rs search from-retrieval <corpus-dir> [--sample N] [--multi N] [--select-seed N] [--steps N] [--compiles N] [--beam K] [--timeout SECS]"
        );
        return ExitCode::from(2);
    };
    let Some(tc) = located() else {
        return ExitCode::SUCCESS;
    };
    if !tc.has_strace() || !tc.has_mingw() {
        println!("SKIP: strace / i686-w64-mingw32-gcc absent (needed for replay)");
        return ExitCode::SUCCESS;
    }

    // Real-path moveset: drop the obj-invisible widen/narrow (P0.6a A) unless
    // explicitly kept — same rule as `search eval`.
    let moves = search_moveset(rest);
    let mut cfg = search::FromSeedConfig::default();
    if let Some(v) = opt(rest, "--sample").and_then(|s| s.parse().ok()) {
        cfg.sample = v;
    }
    if let Some(v) = opt(rest, "--multi").and_then(|s| s.parse().ok()) {
        cfg.multi = v;
    }
    if let Some(v) = opt(rest, "--select-seed").and_then(|s| s.parse().ok()) {
        cfg.select_seed = v;
    }
    if let Some(v) = opt(rest, "--steps").and_then(|s| s.parse().ok()) {
        cfg.budget.max_steps = v;
    }
    if let Some(v) = opt(rest, "--compiles").and_then(|s| s.parse().ok()) {
        cfg.budget.max_compiles = v;
    }
    if let Some(v) = opt(rest, "--beam").and_then(|s| s.parse().ok()) {
        cfg.budget.beam_width = v;
    }
    if let Some(v) = opt(rest, "--timeout").and_then(|s| s.parse().ok()) {
        cfg.timeout = Duration::from_secs(v);
    }

    let w = scratch("search-from-retrieval");
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
            let _ = std::fs::remove_dir_all(&w);
            return ExitCode::FAILURE;
        }
    };
    let _ = std::fs::remove_dir_all(&w);

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

fn cmd_search_from_lifter(rest: &[String]) -> ExitCode {
    let Some(dir) = rest.first().filter(|s| !s.starts_with("--")).map(PathBuf::from) else {
        eprintln!(
            "usage: c2rs search from-lifter <corpus-dir> --gens <jsonl> [--k K] [--limit N] [--steps N] [--compiles N] [--beam K] [--timeout SECS]"
        );
        return ExitCode::from(2);
    };
    let Some(gens_path) = opt(rest, "--gens").map(PathBuf::from) else {
        eprintln!("from-lifter: --gens <jsonl> required (rows {{\"id\",\"generations\":[...]}})");
        return ExitCode::from(2);
    };
    let Some(tc) = located() else {
        return ExitCode::SUCCESS;
    };
    if !tc.has_strace() || !tc.has_mingw() {
        println!("SKIP: strace / i686-w64-mingw32-gcc absent (needed for replay)");
        return ExitCode::SUCCESS;
    }

    let k: usize = opt(rest, "--k").and_then(|s| s.parse().ok()).unwrap_or(5);
    let limit: usize = opt(rest, "--limit").and_then(|s| s.parse().ok()).unwrap_or(0);
    let moves = search_moveset(rest);
    let mut budget = search_budget(rest);
    // Bounded per-generation defaults (many generations x targets share one CPU).
    if opt(rest, "--steps").is_none() {
        budget.max_steps = 8;
    }
    if opt(rest, "--compiles").is_none() {
        budget.max_compiles = 200;
    }
    if opt(rest, "--beam").is_none() {
        budget.beam_width = 4;
    }
    let timeout = Duration::from_secs(
        opt(rest, "--timeout").and_then(|s| s.parse().ok()).unwrap_or(25),
    );

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

    let w = scratch("search-from-lifter");
    let report =
        match search::from_lifter_eval(&tc, &dir, &gens, k, limit, &moves, &budget, timeout, &w) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("from-lifter eval failed: {e}");
                let _ = std::fs::remove_dir_all(&w);
                return ExitCode::FAILURE;
            }
        };
    let _ = std::fs::remove_dir_all(&w);

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

fn cmd_corpus_stats(rest: &[String]) -> ExitCode {
    let Some(dir) = rest.first().map(PathBuf::from) else {
        eprintln!("usage: c2rs corpus stats <dir>");
        return ExitCode::from(2);
    };
    match corpus::load_manifest(&dir) {
        Ok(rows) => {
            let ok = rows.iter().filter(|r| r.status == "ok").count();
            let rt = rows
                .iter()
                .filter(|r| r.codec_roundtrip == Some(true))
                .count();
            let toks: usize = rows.iter().filter_map(|r| r.ex_token_count).sum::<i64>() as usize;
            println!(
                "{}: {} rows, {} ok, {} codec round-trip, {} total .ex tokens",
                dir.display(),
                rows.len(),
                ok,
                rt,
                toks
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("could not read manifest: {e}");
            ExitCode::FAILURE
        }
    }
}
