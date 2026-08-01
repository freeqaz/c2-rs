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
use c2_harness::prefilter;
use c2_harness::provenance::Provenance;
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
        "census" => cmd_census(rest),
        "bench" => cmd_bench(),
        "perf" => cmd_perf(rest),
        "perf-scale" => cmd_perf_scale(rest),
        "corpus" => cmd_corpus(rest),
        "gap" => cmd_gap(rest),
        "listing" => cmd_listing(rest),
        "listing-scan" => cmd_listing_scan(rest),
        "prefilter" => cmd_prefilter(rest),
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
         \x20 c2rs census <cpp>         P2b: per-function in-class / blocking-feature verdict\n\
         \x20 c2rs bench                selftest across all fixtures/cpp/*.cpp\n\
         \x20 c2rs perf [opts]          IL-bundle->obj latency: native port vs standalone c2\n\
         \x20 c2rs perf-scale [opts]    IL-bundle->obj throughput vs concurrency (port vs c2)\n\
         \x20 c2rs corpus gen [opts]    P1.2: generate a (source,IL,obj) triple corpus\n\
         \x20 c2rs corpus sample [dir]  write the portable synthetic sample corpus\n\
         \x20 c2rs corpus stats <dir>   summarize a corpus manifest\n\
         \x20 c2rs gap [opts]           real-workload gap scan: classify every TU, rank the blockers\n\
         \x20 c2rs listing <cpp> [opts] board #132: capture c2's own .cod assembly listing beside the obj\n\
         \x20 c2rs listing-scan [opts]  boards #134/#136: /QXSTALLS demand + the .cod census reconcile\n\
         \x20 c2rs prefilter [opts]     reject-only pre-filter seam: one JSON verdict for one candidate TU\n\
         \x20 c2rs retrieve index <dir> P1.3: obj-retrieval structure of a corpus\n\
         \x20 c2rs retrieve eval <dir>  P1.3: obj->IL retrieval baseline, recall@k\n\
         \x20 c2rs search solve <cpp>   T-A: solve one d=1 instance from a fixture, byte-exact\n\
         \x20 c2rs search eval [opts]   T-A: IL-space solve-rate over fixtures\n\
         \x20 c2rs search from-retrieval <corpus-dir>  T-A: from-unrelated-seed (P1.3-seeded) solve-rate\n\
         \n\
         perf options: --port-iters N --ref-iters N --fixtures a.cpp,b.cpp\n\
         census: c2rs census <cpp> — per-function in-class/blocked verdict (P2b)\n\
         perf-scale options: --fixture X.cpp --conc 1,2,4,8 --port-secs F --ref-secs F --csv PATH\n\
         corpus gen options: --seed N --count N --out DIR --timeout SECS\n\
         gap options: --list FILE --flags-file FILE [--cwd DIR] [--limit N] [--jobs N]\n\
         \x20            [--replay-every N] [--jsonl PATH] (see scripts/gen_dc3_workload.sh)\n\
         \x20            [--cache DIR | --no-cache] [--validate-cache N]\n\
         \x20            captures are cached content-addressed (source bytes + flags +\n\
         \x20            toolchain + workload git identity, never mtimes) under\n\
         \x20            work/capture-cache or C2RS_GAP_CACHE; --validate-cache N\n\
         \x20            re-captures every Nth hit and byte-compares it.\n\
         listing options: [--qxstalls] [--out PATH] [--flag F ...]  (default flags /O1 /Oi /EHsc /GS- /c)\n\
         listing-scan options: --list FILE --flags-file FILE [--cwd DIR] [--limit N] [--jobs N]\n\
         \x20                    [--qxstalls] [--jsonl PATH] [--work DIR]\n\
         prefilter options: --source ARG (--flag F ... | --flags-file FILE) [--cwd DIR]\n\
         \x20                 [--emit-obj PATH] [--compare-obj PATH] [--obj-name Z:\\\\...] [--work DIR]\n\
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

/// `c2rs census <cpp>` — **P2b, single TU**: capture the bundle and print the
/// per-function verdict (modeled shape, or the first blocking feature).
///
/// The whole-TU verdict (`c2rs diff`) is all-or-nothing by design — the port
/// emits a complete obj or nothing — so it says only "NotImplemented" for a TU
/// where 99 of 100 functions are in class. This is the per-function view used
/// while developing a widening step: run it before and after, watch specific
/// functions move from a blocking feature to a shape.
/// Hex-dump a census blocking window, bracketing the byte that blocked the
/// parse: `b9 8b 0a >86< 43 9d 20`. The bracket is what makes the dump usable
/// without counting columns.
fn hexdump_marked(bytes: &[u8], mark: usize) -> String {
    let mut s = String::with_capacity(bytes.len() * 3 + 2);
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 {
            s.push(' ');
        }
        if i == mark {
            s.push_str(&format!(">{b:02x}<"));
        } else {
            s.push_str(&format!("{b:02x}"));
        }
    }
    s
}

fn cmd_census(rest: &[String]) -> ExitCode {
    let Some(cpp) = require_cpp(rest) else {
        return ExitCode::from(2);
    };
    // Optional real-project capture (same inputs as `c2rs gap`), so a census can
    // be taken of an actual workload TU and not just an include-free fixture.
    let mut flags_file: Option<PathBuf> = None;
    let mut cwd: Option<PathBuf> = None;
    let mut keep_il: Option<PathBuf> = None;
    let mut it = rest[1..].iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            // Keep the captured bundle for grammar work (gitignored scratch).
            "--keep-il" => match it.next() {
                Some(v) => keep_il = Some(PathBuf::from(v)),
                None => {
                    eprintln!("--keep-il needs a value");
                    return ExitCode::from(2);
                }
            },
            "--flags-file" => match it.next() {
                Some(v) => flags_file = Some(PathBuf::from(v)),
                None => {
                    eprintln!("--flags-file needs a value");
                    return ExitCode::from(2);
                }
            },
            "--cwd" => match it.next() {
                Some(v) => cwd = Some(PathBuf::from(v)),
                None => {
                    eprintln!("--cwd needs a value");
                    return ExitCode::from(2);
                }
            },
            other => {
                eprintln!("unknown census option: {other}");
                return ExitCode::from(2);
            }
        }
    }
    let Some(tc) = located() else {
        return ExitCode::SUCCESS;
    };
    let w = scratch("census");
    // Two of the port's per-function refusals are `/Gy`-only, so the cross-check
    // below has to see the same flag the emitter would. The default capture is
    // `/Ox`, which does not imply it; a `--flags-file` may.
    let mut gy = false;
    let captured = match &flags_file {
        None => tc.capture_il(&cpp, &w),
        Some(ff) => {
            let flags: Vec<String> = match std::fs::read_to_string(ff) {
                Ok(t) => t
                    .lines()
                    .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
                    .flat_map(|l| l.split_whitespace().map(String::from))
                    .collect(),
                Err(e) => {
                    eprintln!("cannot read --flags-file {}: {e}", ff.display());
                    let _ = std::fs::remove_dir_all(&w);
                    return ExitCode::FAILURE;
                }
            };
            gy = c2_core::PortC2::flags_imply_function_level_linking(&flags);
            tc.capture_reference_with(&cpp.to_string_lossy(), &w, &flags, cwd.as_deref())
                .map(|c| c.bundle)
        }
    };
    let bundle = match captured {
        Ok(b) => b,
        Err(e) => {
            eprintln!("capture failed: {e}");
            let _ = std::fs::remove_dir_all(&w);
            return ExitCode::FAILURE;
        }
    };
    if let Some(dir) = &keep_il {
        if let Err(e) = std::fs::create_dir_all(dir) {
            eprintln!("cannot create --keep-il {}: {e}", dir.display());
        } else {
            for suffix in IL_SUFFIXES {
                if let Some(bytes) = bundle.get(suffix) {
                    let p = dir.join(format!("{}.{suffix}", bundle.base_name));
                    if let Err(e) = std::fs::write(&p, bytes) {
                        eprintln!("cannot write {}: {e}", p.display());
                    }
                }
            }
            println!("kept IL bundle in {}", dir.display());
        }
    }
    let Some(rows) = bundle.census_functions() else {
        eprintln!("census unavailable: bundle is missing .ex/.gl");
        let _ = std::fs::remove_dir_all(&w);
        return ExitCode::FAILURE;
    };
    // The census/gate cross-check, per TU (roadmap #44): a function the census
    // calls in class that `PortC2`'s own selector refuses. `c2rs census` is the
    // instrument a widening step is developed against, so it has to show this —
    // `int f(int a,int b,int c){ return a + b*c; }` read `1/1 in class` beside a
    // `Port=NotImplemented` for as long as the disagreement existed.
    let mut gate_hist: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    for (f, gate) in &rows {
        if !f.verdict.in_class() {
            continue;
        }
        let key = match gate {
            Err(e) => Some((*e).to_string()),
            Ok(func) => match c2_core::codegen::opt_mode_of_word(f.opt_word) {
                Err(e) => Some(e.to_string()),
                Ok(mode) => c2_core::codegen::function_gate(func, mode, gy)
                    .err()
                    .map(|e| e.to_string()),
            },
        };
        if let Some(k) = key {
            *gate_hist.entry(k).or_insert(0) += 1;
        }
    }
    // The `.gl` binding invariants (D14), per TU: what every generated empty
    // destructor resolved to, and whether any token is claimed twice. The oracle
    // cannot grade a correspondence, so these are printed where a widening step is
    // developed, not only in the scan aggregate (`docs/GAPS.md` §6).
    let mut dtor_callees: Vec<(String, String)> = Vec::new();
    for (f, gate) in &rows {
        if f.verdict.key().starts_with("empty-dtor") {
            if let Ok(func) = gate {
                dtor_callees.push((
                    f.name.clone().unwrap_or_else(|| format!("#{}", f.index)),
                    func.tail_call.clone().unwrap_or_default(),
                ));
            }
        }
    }
    let (gl_dropped, gl_conflicts) = bundle
        .get("gl")
        .map(|g| c2_il::gl_symbol_conflicts(g))
        .unwrap_or((0, 0));
    let census: Vec<c2_il::FnCensus> = rows.into_iter().map(|(c, _)| c).collect();
    let in_class = census.iter().filter(|f| f.verdict.in_class()).count();
    println!(
        "{} -> {}/{} functions in class",
        cpp.display(),
        in_class,
        census.len()
    );
    if gl_dropped > 0 {
        println!(
            "  .gl ambiguous tokens dropped: {gl_dropped} ({gl_conflicts} involving a mangled \
             name — that count must be 0)"
        );
    }
    if !dtor_callees.is_empty() {
        let bad = dtor_callees
            .iter()
            .filter(|(_, c)| c2_harness::gap::dtor_callee_class(c) == "other")
            .count();
        println!(
            "  generated empty destructors: {} bound, {bad} to a NON-destructor",
            dtor_callees.len()
        );
        for (f, c) in dtor_callees.iter().take(12) {
            println!("    {f} -> {c}");
        }
    }
    let mut hist: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    // One representative blocking-site hexdump per feature, so a big TU reports
    // each distinct gap once instead of thousands of times.
    let mut sample: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    // The control-flow axis, over EVERY function including the in-class ones —
    // they are the control group, and every one of them must read
    // `cflow-straight`, because every shape the port accepts is a single basic
    // block. A `cflow-loop` among them would indict the measure.
    let mut cflow_hist: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    // The EH axis beside it, likewise over every function. Here the in-class
    // rows are more than a control group: the three `empty-dtor-*` shapes ARE
    // the cheap side of `docs/EH_RECORDS.md` §6's boundary, so one of them
    // reading anything but `eh-bare` indicts the axis.
    let mut eh_hist: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    // Per-function lines are only readable for a small TU; a real one has
    // thousands of functions and the histogram is the useful view.
    let list_each = census.len() <= 64;
    for f in &census {
        let mark = if f.verdict.in_class() { "ok " } else { "GAP" };
        if list_each {
            // Both census axes on one line. A control-flow fixture is graded on
            // the pair: it must refuse (the first column) AND its shape must be
            // decoded (the second), and a single column can show only one of
            // those. `c2rs gap` prints the same second axis aggregated.
            println!(
                "  [{:>3}] {mark} {:<24} {:<26} {:<11} ({:<12}) {:>6} B  {}",
                f.index,
                f.verdict.key(),
                f.cflow,
                f.eh,
                f.eh_stmt,
                f.seg_len,
                f.name.as_deref().unwrap_or("(unnamed)")
            );
        }
        *cflow_hist.entry(f.cflow.clone()).or_insert(0) += 1;
        *eh_hist.entry(f.eh.clone()).or_insert(0) += 1;
        if !f.verdict.in_class() {
            *hist.entry(f.verdict.key()).or_insert(0) += 1;
            sample
                .entry(f.verdict.key())
                .or_insert_with(|| hexdump_marked(&f.hex, f.hex_mark));
        }
    }
    if !hist.is_empty() {
        let mut v: Vec<_> = hist.into_iter().collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        println!("  blocking features (\">\" marks the byte that blocked the parse):");
        for (feature, count) in v.iter().take(24) {
            println!("    {count:>6} x {feature}");
            if let Some(h) = sample.get(feature) {
                println!("             {h}");
            }
        }
        if v.len() > 24 {
            println!("    … and {} more distinct features", v.len() - 24);
        }
    }
    {
        let mut v: Vec<_> = cflow_hist.into_iter().collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        let decoded: usize = v.iter().filter(|(k, _)| k.starts_with("cflow-")).map(|(_, n)| n).sum();
        println!(
            "  control-flow class (decode-only): {decoded}/{} bodies decoded end to end",
            census.len()
        );
        for (class, count) in v.iter().take(16) {
            println!("    {count:>6} x {class}");
        }
    }
    {
        let mut v: Vec<_> = eh_hist.into_iter().collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        let need: usize = v
            .iter()
            .filter(|(k, _)| k.as_str() == "eh-state1")
            .map(|(_, n)| n)
            .sum();
        println!(
            "  EH class (maxState, decode-only): {need}/{} bodies have maxState >= 1 and need the \
             whole EH record",
            census.len()
        );
        for (class, count) in v.iter() {
            println!("    {count:>6} x {class}");
        }
    }
    if !gate_hist.is_empty() {
        let n: usize = gate_hist.values().sum();
        println!(
            "  census/gate DISAGREEMENT: {n} of the {in_class} in class are refused by PortC2:"
        );
        let mut v: Vec<_> = gate_hist.into_iter().collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        for (reason, count) in v.iter().take(12) {
            println!("    {count:>6} x {reason}");
        }
    }
    let _ = std::fs::remove_dir_all(&w);
    ExitCode::SUCCESS
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
    // `--keep-obj PATH` retains the reference obj for byte classification (the
    // CONST/DERIVED analysis every widening step starts from). Gitignored
    // scratch only — objs are never committed.
    let keep_obj: Option<PathBuf> = rest
        .iter()
        .position(|a| a == "--keep-obj")
        .and_then(|i| rest.get(i + 1))
        .map(PathBuf::from);
    // Optional real-project compile (same inputs as `c2rs gap`), so the
    // reference obj for a workload TU can be classified, not just a fixture's.
    let flags_file: Option<PathBuf> = rest
        .iter()
        .position(|a| a == "--flags-file")
        .and_then(|i| rest.get(i + 1))
        .map(PathBuf::from);
    let cwd: Option<PathBuf> = rest
        .iter()
        .position(|a| a == "--cwd")
        .and_then(|i| rest.get(i + 1))
        .map(PathBuf::from);
    let w = scratch("compile");
    let out = w.join("out.obj");
    if let Some(ff) = &flags_file {
        let flags: Vec<String> = match std::fs::read_to_string(ff) {
            Ok(t) => t
                .lines()
                .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
                .flat_map(|l| l.split_whitespace().map(String::from))
                .collect(),
            Err(e) => {
                eprintln!("cannot read --flags-file {}: {e}", ff.display());
                return ExitCode::FAILURE;
            }
        };
        let res = tc.capture_reference_with(&cpp.to_string_lossy(), &w, &flags, cwd.as_deref());
        return match res {
            Ok(c) => {
                println!(
                    "compiled {} -> {} bytes (project flags)",
                    cpp.display(),
                    c.ref_obj.len()
                );
                if let Some(dest) = &keep_obj {
                    if let Some(p) = dest.parent() {
                        let _ = std::fs::create_dir_all(p);
                    }
                    let _ = std::fs::write(dest, c.ref_obj.as_bytes());
                    println!("  kept reference obj at {}", dest.display());
                }
                let _ = std::fs::remove_dir_all(&w);
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("compile failed: {e}");
                let _ = std::fs::remove_dir_all(&w);
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
            if let Some(dest) = &keep_obj {
                if let Some(parent) = dest.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                match std::fs::write(dest, obj.as_bytes()) {
                    Ok(()) => println!("  kept reference obj at {}", dest.display()),
                    Err(e) => eprintln!("  cannot write {}: {e}", dest.display()),
                }
            }
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
    // The oracle self-test IS the correctness benchmark, so it names the oracle
    // it ran against (roadmap #48): a stale wibo turns this seam's verdicts over
    // without changing any other number in the report.
    print!("{}", Provenance::collect(&tc, None).render());
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

/// **Board #132 — the listing seam.** Capture one TU and print (or write) c2's
/// own `.cod` assembly listing beside the obj the differential grades.
///
/// The listing is a **decode aid, never a gate**: the obj byte-compare remains
/// the sole judge of the port.
fn cmd_listing(rest: &[String]) -> ExitCode {
    let mut cpp: Option<PathBuf> = None;
    let mut out: Option<PathBuf> = None;
    let mut qxstalls = false;
    let mut flags: Vec<String> = Vec::new();
    let mut it = rest.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--qxstalls" => qxstalls = true,
            "--out" => match it.next() {
                Some(v) => out = Some(PathBuf::from(v)),
                None => {
                    eprintln!("--out needs a value");
                    return ExitCode::from(2);
                }
            },
            "--flag" => match it.next() {
                Some(v) => flags.push(v.clone()),
                None => {
                    eprintln!("--flag needs a value");
                    return ExitCode::from(2);
                }
            },
            other if cpp.is_none() && !other.starts_with("--") => cpp = Some(PathBuf::from(other)),
            other => {
                eprintln!("unknown listing option: {other}");
                return ExitCode::from(2);
            }
        }
    }
    let Some(cpp) = cpp else {
        eprintln!(
            "usage: c2rs listing <cpp> [--qxstalls] [--out PATH] [--flag F ...]\n\
             default flags: /O1 /Oi /EHsc /GS- /c"
        );
        return ExitCode::from(2);
    };
    let Some(tc) = located() else {
        return ExitCode::SUCCESS;
    };
    if !tc.has_strace() {
        println!("SKIP: strace absent (needed to keep the IL bundle)");
        return ExitCode::SUCCESS;
    }
    if flags.is_empty() {
        flags = ["/O1", "/Oi", "/EHsc", "/GS-", "/c"]
            .iter()
            .map(|s| s.to_string())
            .collect();
    }
    let w = scratch("listing");
    let src = c2_reference::to_wibo_path(&cpp);
    let code = match tc.capture_listing_with(&src, &w, &flags, None, qxstalls) {
        Ok((captured, cod)) => {
            let listing = c2_reference::cod::CodListing::parse(&cod);
            let emitted = captured
                .ref_obj
                .text_comdat_functions()
                .map(|v| v.len())
                .unwrap_or(0);
            println!(
                "{} -> obj={}B  cod={}B  {} PROC / {} .text COMDAT / {} PUBLIC{}",
                cpp.display(),
                captured.ref_obj.len(),
                cod.len(),
                listing.functions.len(),
                emitted,
                listing.publics.len(),
                if qxstalls { "  [/QXSTALLS]" } else { "" },
            );
            match &out {
                Some(p) => match std::fs::write(p, cod.as_bytes()) {
                    Ok(()) => {
                        println!("  listing written to {}", p.display());
                        ExitCode::SUCCESS
                    }
                    Err(e) => {
                        eprintln!("cannot write {}: {e}", p.display());
                        ExitCode::FAILURE
                    }
                },
                None => {
                    print!("{cod}");
                    ExitCode::SUCCESS
                }
            }
        }
        Err(e) => {
            eprintln!("listing capture failed: {e}");
            ExitCode::FAILURE
        }
    };
    let _ = std::fs::remove_dir_all(&w);
    code
}

/// **Boards #134 and #136** — the population scan over the listing seam.
fn cmd_listing_scan(rest: &[String]) -> ExitCode {
    let mut list_file: Option<PathBuf> = None;
    let mut flags_file: Option<PathBuf> = None;
    let mut cwd: Option<PathBuf> = None;
    let mut limit: Option<usize> = None;
    let mut jobs: usize = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    let mut jsonl: Option<PathBuf> = None;
    let mut work: Option<PathBuf> = None;
    let mut qxstalls = false;

    let mut it = rest.iter();
    while let Some(a) = it.next() {
        let mut val = |name: &str| -> Option<String> {
            match it.next() {
                Some(v) => Some(v.clone()),
                None => {
                    eprintln!("{name} needs a value");
                    None
                }
            }
        };
        match a.as_str() {
            "--qxstalls" => qxstalls = true,
            "--list" => match val("--list") {
                Some(v) => list_file = Some(PathBuf::from(v)),
                None => return ExitCode::from(2),
            },
            "--flags-file" => match val("--flags-file") {
                Some(v) => flags_file = Some(PathBuf::from(v)),
                None => return ExitCode::from(2),
            },
            "--cwd" => match val("--cwd") {
                Some(v) => cwd = Some(PathBuf::from(v)),
                None => return ExitCode::from(2),
            },
            "--limit" => match val("--limit").and_then(|v| v.parse().ok()) {
                Some(v) => limit = Some(v),
                None => return ExitCode::from(2),
            },
            "--jobs" => match val("--jobs").and_then(|v| v.parse().ok()) {
                Some(v) => jobs = v,
                None => return ExitCode::from(2),
            },
            "--jsonl" => match val("--jsonl") {
                Some(v) => jsonl = Some(PathBuf::from(v)),
                None => return ExitCode::from(2),
            },
            "--work" => match val("--work") {
                Some(v) => work = Some(PathBuf::from(v)),
                None => return ExitCode::from(2),
            },
            other => {
                eprintln!("unknown listing-scan option: {other}");
                return ExitCode::from(2);
            }
        }
    }
    let (Some(list_file), Some(flags_file)) = (list_file, flags_file) else {
        eprintln!(
            "usage: c2rs listing-scan --list FILE --flags-file FILE [--cwd DIR] \
             [--limit N] [--jobs N] [--qxstalls] [--jsonl PATH] [--work DIR]"
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
    let Some(tc) = located() else {
        return ExitCode::SUCCESS;
    };
    if !tc.has_strace() {
        println!("SKIP: strace absent (needed to keep the IL bundle)");
        return ExitCode::SUCCESS;
    }

    let cfg = c2_harness::listing::ListingScanConfig {
        sources,
        flags,
        cwd,
        limit,
        jobs,
        work: work.unwrap_or_else(|| scratch("listing-scan")),
        qxstalls,
        jsonl,
    };
    let total_hint = cfg.limit.unwrap_or(cfg.sources.len());
    eprintln!(
        "listing-scan: {} TUs, {} jobs, /QXSTALLS {}",
        total_hint,
        cfg.jobs,
        if cfg.qxstalls { "ON" } else { "OFF (control)" }
    );
    let report = match c2_harness::listing::listing_scan(&tc, &cfg, &|n, total, r| {
        if n % 25 == 0 || n == total {
            eprintln!("  [{n}/{total}] {}{}", r.src, if r.error.is_empty() { "" } else { "  CAPTURE-FAIL" });
        }
    }) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("listing scan failed: {e}");
            return ExitCode::FAILURE;
        }
    };

    let pct = |a: usize, b: usize| if b == 0 { 0.0 } else { 100.0 * a as f64 / b as f64 };

    println!("\nLISTING SCAN — {} TUs captured, {} failed", report.captured, report.failed);

    // ---- #136 -------------------------------------------------------------
    let (procs, comdats, dupes, cod_only, obj_only) = report.reconcile_totals();
    let (agree, be_total) = report.byte_exact_agreement();
    let (in_class, emitted, unbound) = report.emitted_census();
    println!("\n#136  THE TWO SOURCES, RECONCILED");
    println!("  .cod PROC set          {procs}");
    println!("  obj .text COMDAT set   {comdats}");
    println!(
        "  invariant 1 injectivity     duplicate PROC names: {dupes}  ({})",
        if dupes == 0 { "PASS" } else { "FAIL" }
    );
    println!(
        "  invariant 2 totality        cod-only {cod_only}, obj-only {obj_only}  ({})",
        if cod_only == 0 && obj_only == 0 { "PASS" } else { "residue printed below" }
    );
    println!(
        "  invariant 3 byte-exact TUs  {agree}/{be_total} reconcile exactly  ({})",
        if agree == be_total { "PASS" } else { "FAIL" }
    );
    let err_terms = cod_only + obj_only + dupes;
    println!(
        "  ERROR TERM on the emitted census: {err_terms} of {comdats} emitted \
         functions = {:.4} pp",
        pct(err_terms, comdats)
    );
    println!(
        "  this scan's emitted census: {in_class}/{emitted} in class ({:.2}%), \
         {unbound} unbound residue",
        pct(in_class, emitted)
    );
    let residue = report.residue_classes();
    if !residue.is_empty() {
        println!("  residue by mangling class:");
        for (k, n) in residue.iter().take(15) {
            println!("    {n:8}  {k}");
        }
    }

    // ---- #134 -------------------------------------------------------------
    let (bs, bt, ics, ict) = report.stall_totals();
    let blhs: usize = report.tus.iter().map(|t| t.blocked_lhs).sum();
    let ilhs: usize = report.tus.iter().map(|t| t.in_class_lhs).sum();
    println!("\n#134  /QXSTALLS SCHEDULING DEMAND (emitted-function units)");
    println!(
        "  BLOCKED  emitted: {bs}/{bt} carry a stall annotation  ({:.2}%)   \
         load-hit-store: {blhs} ({:.2}%)",
        pct(bs, bt),
        pct(blhs, bt)
    );
    println!(
        "  IN-CLASS emitted: {ics}/{ict} carry a stall annotation  ({:.2}%)   \
         load-hit-store: {ilhs} ({:.2}%)   <- THE CONTROL",
        pct(ics, ict),
        pct(ilhs, ict)
    );
    println!(
        "  discrimination: blocked − in-class = {:+.2} pp",
        pct(bs, bt) - pct(ics, ict)
    );
    // The size confound, measured. A blocked function is typically far longer
    // than an in-class one, and a longer body has more chances to stall — so the
    // headline gap has to survive being read inside a size bucket or it is a
    // statement about length.
    println!("  size-stratified (the confound: blocked bodies are longer):");
    println!(
        "    bucket        blocked stalled/total          in-class stalled/total                blocked LHS   in-class LHS"
    );
    for (b, bstall, btot, istall, itot, blhs_b, ilhs_b) in report.size_strata() {
        println!(
            "    {b:<12}  {bstall:>7}/{btot:<7} ({:>6.2}%)   {istall:>7}/{itot:<7} ({:>6.2}%)                {blhs_b:>6} ({:>5.2}%)  {ilhs_b:>6} ({:>5.2}%)",
            pct(bstall, btot),
            pct(istall, itot),
            pct(blhs_b, btot),
            pct(ilhs_b, itot),
        );
    }
    if !qxstalls {
        println!(
            "  (run WITHOUT --qxstalls: both rows must read 0 — that is the \
             negative control for the annotation reader)"
        );
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

// ---------------------------------------------------------------------------
// gap — real-workload gap scan
// ---------------------------------------------------------------------------

use c2_harness::gap::{gap_scan, GapConfig, TuClass};

/// `c2rs gap --list FILE --flags-file FILE [--cwd DIR] …` — scan real TUs,
/// classify each (capture-fail / vocab-gap / codegen-gap / mismatch / match),
/// and rank the blockers. Exit is non-zero only on a *correctness* signal
/// (`mismatch` TUs or a replay-soundness divergence) or a harness error —
/// gaps themselves are the expected measurement, not a failure.
fn cmd_gap(rest: &[String]) -> ExitCode {
    let mut list_file: Option<PathBuf> = None;
    let mut flags_file: Option<PathBuf> = None;
    let mut cwd: Option<PathBuf> = None;
    let mut limit: Option<usize> = None;
    let mut jobs: usize = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    let mut replay_every: usize = 0;
    let mut jsonl: Option<PathBuf> = None;
    let mut work: Option<PathBuf> = None;
    // Capture cache: ON by default (roadmap #15). The key is content-addressed
    // (source bytes + flags + toolchain + workload-tree identity), never mtimes;
    // `--no-cache` bypasses it and `--validate-cache N` re-captures every Nth
    // hit and byte-compares. Default root is under the gitignored `work/`.
    let mut cache: Option<PathBuf> = Some(
        std::env::var_os("C2RS_GAP_CACHE")
            .map(PathBuf::from)
            .unwrap_or_else(|| c2_harness::provenance::repo_root().join("work/capture-cache")),
    );
    let mut validate_cache: usize = 0;

    let mut it = rest.iter();
    while let Some(a) = it.next() {
        let mut val = |name: &str| -> Option<String> {
            match it.next() {
                Some(v) => Some(v.clone()),
                None => {
                    eprintln!("{name} needs a value");
                    None
                }
            }
        };
        match a.as_str() {
            "--list" => match val("--list") {
                Some(v) => list_file = Some(PathBuf::from(v)),
                None => return ExitCode::from(2),
            },
            "--flags-file" => match val("--flags-file") {
                Some(v) => flags_file = Some(PathBuf::from(v)),
                None => return ExitCode::from(2),
            },
            "--cwd" => match val("--cwd") {
                Some(v) => cwd = Some(PathBuf::from(v)),
                None => return ExitCode::from(2),
            },
            "--limit" => match val("--limit").and_then(|v| v.parse().ok()) {
                Some(v) => limit = Some(v),
                None => return ExitCode::from(2),
            },
            "--jobs" => match val("--jobs").and_then(|v| v.parse().ok()) {
                Some(v) => jobs = v,
                None => return ExitCode::from(2),
            },
            "--replay-every" => match val("--replay-every").and_then(|v| v.parse().ok()) {
                Some(v) => replay_every = v,
                None => return ExitCode::from(2),
            },
            "--jsonl" => match val("--jsonl") {
                Some(v) => jsonl = Some(PathBuf::from(v)),
                None => return ExitCode::from(2),
            },
            "--work" => match val("--work") {
                Some(v) => work = Some(PathBuf::from(v)),
                None => return ExitCode::from(2),
            },
            "--cache" => match val("--cache") {
                Some(v) => cache = Some(PathBuf::from(v)),
                None => return ExitCode::from(2),
            },
            "--no-cache" => cache = None,
            "--validate-cache" => match val("--validate-cache").and_then(|v| v.parse().ok()) {
                Some(v) => validate_cache = v,
                None => return ExitCode::from(2),
            },
            other => {
                eprintln!("unknown gap option: {other}");
                return ExitCode::from(2);
            }
        }
    }

    let (Some(list_file), Some(flags_file)) = (list_file, flags_file) else {
        eprintln!(
            "usage: c2rs gap --list FILE --flags-file FILE [--cwd DIR] [--limit N] \
             [--jobs N] [--replay-every N] [--jsonl PATH] [--work DIR]\n\
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

    let Some(tc) = located() else {
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

    let cfg = GapConfig {
        sources,
        flags,
        cwd,
        limit,
        jobs,
        replay_every,
        jsonl,
        work: work.unwrap_or_else(|| scratch("gap")),
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
            for (key, count) in cflow
                .iter()
                .filter(|(k, _)| k.contains('|') && !k.starts_with("cflow-straight"))
                .take(12)
            {
                println!("    {count:>7}           {key}");
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

/// `c2rs prefilter` — the reject-only pre-filter seam (see
/// [`c2_harness::prefilter`] for the contract that binds callers).
///
/// Prints exactly one line of JSON on stdout and exits 0 for every well-formed
/// verdict, including `not_implemented`. Exit 2 means "you called me wrong" —
/// a caller must treat that as a hard error, never as a verdict.
fn cmd_prefilter(rest: &[String]) -> ExitCode {
    let mut source: Option<String> = None;
    let mut flags: Vec<String> = Vec::new();
    let mut flags_file: Option<PathBuf> = None;
    let mut cwd: Option<PathBuf> = None;
    let mut emit_obj: Option<PathBuf> = None;
    let mut compare_obj: Option<PathBuf> = None;
    let mut obj_name: Option<String> = None;
    let mut work: Option<PathBuf> = None;

    let mut it = rest.iter();
    while let Some(a) = it.next() {
        let mut val = |name: &str| -> Option<String> {
            match it.next() {
                Some(v) => Some(v.clone()),
                None => {
                    eprintln!("{name} needs a value");
                    None
                }
            }
        };
        match a.as_str() {
            "--source" => match val("--source") {
                Some(v) => source = Some(v),
                None => return ExitCode::from(2),
            },
            "--flag" => match val("--flag") {
                Some(v) => flags.push(v),
                None => return ExitCode::from(2),
            },
            "--flags-file" => match val("--flags-file") {
                Some(v) => flags_file = Some(PathBuf::from(v)),
                None => return ExitCode::from(2),
            },
            "--cwd" => match val("--cwd") {
                Some(v) => cwd = Some(PathBuf::from(v)),
                None => return ExitCode::from(2),
            },
            "--emit-obj" => match val("--emit-obj") {
                Some(v) => emit_obj = Some(PathBuf::from(v)),
                None => return ExitCode::from(2),
            },
            "--compare-obj" => match val("--compare-obj") {
                Some(v) => compare_obj = Some(PathBuf::from(v)),
                None => return ExitCode::from(2),
            },
            "--obj-name" => match val("--obj-name") {
                Some(v) => obj_name = Some(v),
                None => return ExitCode::from(2),
            },
            "--work" => match val("--work") {
                Some(v) => work = Some(PathBuf::from(v)),
                None => return ExitCode::from(2),
            },
            "--schema" => {
                println!("{}", prefilter::SCHEMA);
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("unknown prefilter option: {other}");
                return ExitCode::from(2);
            }
        }
    }

    let Some(source) = source else {
        eprintln!(
            "usage: c2rs prefilter --source ARG (--flag F ... | --flags-file FILE) [--cwd DIR]\n\
             \x20                    [--emit-obj PATH] [--compare-obj PATH] [--obj-name Z:\\...]\n\
             \x20                    [--work DIR] | --schema\n\
             Prints one line of JSON; exit 0 = verdict, exit 2 = usage error.\n\
             Only verdict=\"reject\" licenses skipping a real compile."
        );
        return ExitCode::from(2);
    };

    if let Some(p) = &flags_file {
        match std::fs::read_to_string(p) {
            Ok(text) => {
                for line in text.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    flags.extend(line.split_whitespace().map(String::from));
                }
            }
            Err(e) => {
                eprintln!("cannot read --flags-file {}: {e}", p.display());
                return ExitCode::from(2);
            }
        }
    }
    if flags.is_empty() {
        eprintln!("prefilter needs the TU's real compile flags (--flag / --flags-file)");
        return ExitCode::from(2);
    }

    let work = work.unwrap_or_else(|| scratch("prefilter"));
    let owned_work = work.clone();
    let req = prefilter::Request {
        source,
        flags,
        cwd,
        emit_obj,
        compare_obj,
        obj_name,
        work,
    };
    let out = prefilter::run(Toolchain::locate().as_ref(), &req);
    println!("{}", out.to_json());
    // Captured IL bundles are large and this runs per candidate; the JSON (and
    // the emitted obj, which lives wherever the caller asked) is the record.
    let _ = std::fs::remove_dir_all(&owned_work);
    ExitCode::SUCCESS
}
