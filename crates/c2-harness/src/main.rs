//! `c2rs` — CLI over the differential harness. std only (no clap): args are
//! parsed by hand. Every subcommand degrades to "SKIP: toolchain absent" when
//! `Toolchain::locate()` is `None` — it never panics on a missing toolchain.
//!
//! Subcommands:
//!   capture <cpp>       capture IL, print the 5 file sizes
//!   compile <cpp>       reference obj, print size + timestamp
//!   selftest [<cpp>...] oracle self-test over the given TUs (or all fixtures)
//!   replay <cpp>        P0.1: capture + standalone-c2 replay, print byte-match
//!   diff <cpp>          full differential (ReferenceReplay=ByteExact, Port=NotImplemented)
//!   bench               selftest across all fixtures/cpp/*.cpp, summary counts
//!   corpus <sub>        P1.2 corpus generator (gen / sample / stats)

use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use c2_core::PortC2;
use c2_harness::corpus::{self, CorpusConfig};
use c2_harness::retrieval;
use c2_harness::{
    all_fixtures, differential, oracle_selftest, DiffReport, PortStatus, SelfTestOutcome,
    SelfTestReport,
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
        "diff" => cmd_diff(rest),
        "bench" => cmd_bench(),
        "corpus" => cmd_corpus(rest),
        "retrieve" => cmd_retrieve(rest),
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
         \x20 c2rs diff <cpp>           full differential (ReferenceReplay=ByteExact, Port=NotImplemented)\n\
         \x20 c2rs bench                selftest across all fixtures/cpp/*.cpp\n\
         \x20 c2rs corpus gen [opts]    P1.2: generate a (source,IL,obj) triple corpus\n\
         \x20 c2rs corpus sample [dir]  write the portable synthetic sample corpus\n\
         \x20 c2rs corpus stats <dir>   summarize a corpus manifest\n\
         \x20 c2rs retrieve index <dir> P1.3: obj-retrieval structure of a corpus\n\
         \x20 c2rs retrieve eval <dir>  P1.3: obj->IL retrieval baseline, recall@k\n\
         \n\
         corpus gen options: --seed N --count N --out DIR --timeout SECS\n\
         retrieve eval options: --split held-out|loo --query-div N --k 1,5,10\n\
         \n\
         Toolchain is located via C2RS_WIBO / C2RS_CL_EXE / C2RS_C2_DLL / C2RS_WIBO_DEBUG\n\
         / C2RS_DC3_ROOT (relative-to-repo defaults). Absent toolchain -> clean SKIP."
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
    // A byte-exact reference replay with the port still a stub is the expected
    // state today; treat it (and clean skips) as success for scripting.
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
