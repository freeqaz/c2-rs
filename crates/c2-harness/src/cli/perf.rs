//! Angle H: `perf` (per-obj latency) and `perf-scale` (throughput vs
//! concurrency), the two measurement subcommands.

use std::path::PathBuf;
use std::process::ExitCode;

use c2_harness::all_fixtures;
use c2_harness::toolchain_gate::{toolchain_ready, Cap};

use crate::{Args, Arity, Spec};
use crate::cli::util::{first_line, Scratch};

// ---------------------------------------------------------------------------
// perf — angle-H latency: native port vs standalone c2 (IL bundle -> obj)
// ---------------------------------------------------------------------------

use c2_harness::perf::{self, fmt_dur, PerfConfig, PortPerf};

static PERF_SPEC: Spec = Spec::new(
    "perf",
    &[
        ("--port-iters", Arity::Value),
        ("--ref-iters", Arity::Value),
        ("--fixtures", Arity::Value),
    ],
)
.positionals(0);

pub(crate) fn cmd_perf(rest: &[String]) -> ExitCode {
    // Parse and validate FIRST. This handler used to call `located()` as its
    // opening statement, so on a machine with no compilers `c2rs perf --typo`
    // exited **0** with `SKIP: toolchain absent` and the typo was never
    // reported. That is the ordering half of the class, and it is now
    // inexpressible: `args.toolchain()` needs an `args`.
    let args = match Args::parse(&PERF_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let mut cfg = PerfConfig::default();
    // `.parse().ok()` turned a typo into the default in silence. `num` refuses.
    match args.num::<usize>("--port-iters") {
        Ok(Some(v)) => cfg.port_iters = v,
        Ok(None) => {}
        Err(c) => return c,
    }
    match args.num::<usize>("--ref-iters") {
        Ok(Some(v)) => cfg.ref_iters = v,
        Ok(None) => {}
        Err(c) => return c,
    }
    let Some(tc) = args.toolchain() else {
        return ExitCode::SUCCESS;
    };
    if let Some(code) = toolchain_ready(
        &tc,
        &[Cap::Strace, Cap::Mingw],
        "needed for standalone-c2 replay",
    ) {
        return code;
    }
    let targets: Vec<PathBuf> = match args.get("--fixtures") {
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
        let w = Scratch::new("perf");
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

static PERF_SCALE_SPEC: Spec = Spec::new(
    "perf-scale",
    &[
        ("--fixture", Arity::Value),
        ("--conc", Arity::Value),
        ("--port-secs", Arity::Value),
        ("--ref-secs", Arity::Value),
        ("--csv", Arity::Value),
    ],
)
.positionals(0);

pub(crate) fn cmd_perf_scale(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&PERF_SCALE_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let fixture = args.get("--fixture")
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
    // `filter_map(parse().ok())` SILENTLY DROPPED a bad element: `--conc 1,x,4`
    // ran `[1, 4]` and printed `concurrencies=[1, 4]` as if that were what was
    // asked for. Only an all-bad list reached the refusal below, so the failure
    // was invisible exactly when it was partial.
    cfg.concurrencies = match args.get("--conc") {
        Some(list) => {
            let mut v: Vec<usize> = Vec::new();
            for tok in list.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                match tok.parse::<usize>() {
                    Ok(c) if c >= 1 => v.push(c),
                    _ => {
                        eprintln!("perf-scale: --conc expects positive integers, got {tok:?}");
                        return ExitCode::from(2);
                    }
                }
            }
            v
        }
        None => default_concurrencies(),
    };
    if cfg.concurrencies.is_empty() {
        eprintln!("no valid --conc values");
        return ExitCode::from(2);
    }
    match args.num::<f64>("--port-secs") {
        Ok(Some(v)) => cfg.port_secs = v,
        Ok(None) => {}
        Err(c) => return c,
    }
    match args.num::<f64>("--ref-secs") {
        Ok(Some(v)) => cfg.ref_secs = v,
        Ok(None) => {}
        Err(c) => return c,
    }
    let Some(tc) = args.toolchain() else {
        return ExitCode::SUCCESS;
    };
    if let Some(code) = toolchain_ready(
        &tc,
        &[Cap::Strace, Cap::Mingw],
        "needed for standalone-c2 replay",
    ) {
        return code;
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

    let w = Scratch::new("perf-scale");
    let (points, obj_len) = match perf::scale_measure(&tc, &fixture, &cfg, &w) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("perf-scale failed: {}", first_line(&e.to_string()));
            return ExitCode::FAILURE;
        }
    };

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
    if let Some(path) = args.get("--csv") {
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
