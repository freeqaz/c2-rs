//! `c2rs census` — P2b, the per-function in-class / blocking-feature verdict
//! for a single TU.

use std::process::ExitCode;

use c2_il::IL_SUFFIXES;

use crate::cli::util::{require_cpp, scratch, CPP_PROFILE_REQUIRES};
use crate::{Args, Arity, Spec};

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

static CENSUS_SPEC: Spec = Spec::new(
    "census",
    &[
        ("--keep-il", Arity::Value),
        ("--flags-file", Arity::Value),
        ("--cwd", Arity::Value),
    ],
)
.requires(CPP_PROFILE_REQUIRES);

pub(crate) fn cmd_census(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&CENSUS_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let Some(cpp) = require_cpp(&args) else {
        return ExitCode::from(2);
    };
    // Optional real-project capture (same inputs as `c2rs gap`), so a census can
    // be taken of an actual workload TU and not just an include-free fixture.
    // Keep the captured bundle for grammar work (gitignored scratch).
    let keep_il = args.path("--keep-il");
    let flags_file = args.path("--flags-file");
    let cwd = args.path("--cwd");
    // The profile is read and validated BEFORE the toolchain is located — the
    // ordering `capture` and `compile` already had and this command did not.
    // `census` read its `--flags-file` *inside* the post-`located()` capture
    // block, so `census x.cpp --flags-file /nonexistent` exited **0** with
    // `SKIP: toolchain absent` on a machine with no compilers, which is exactly
    // where the portable test lane runs. A usage error the binary never reports
    // is a usage error no test can pin.
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
        // `capture` and `compile` both refuse this; `census` did not, so an
        // all-comment flags file silently fell back to `cl.exe`'s own defaults
        // and the `/Gy`-dependent cross-check below was reported against a
        // profile nobody named.
        eprintln!("--flags-file names no flags; refusing to census at an unknown profile");
        return ExitCode::from(2);
    }
    let Some(tc) = args.toolchain() else {
        return ExitCode::SUCCESS;
    };
    let w = scratch("census");
    // Two of the port's per-function refusals are `/Gy`-only, so the cross-check
    // below has to see the same flag the emitter would. The default capture is
    // `/Ox`, which does not imply it; a `--flags-file` may.
    let mut gy = false;
    // Print the profile that was actually used, always — the affordance
    // `capture` and `compile` have and this command lacked, which is why its
    // dropped `--cwd` had no terminal signal at all.
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
    let captured = match &flags_file {
        None => tc.capture_il(&cpp, &w),
        Some(_) => {
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
    // they are the control group. Every shape the port accepts is a single basic
    // block **except `ptr-walk-mod-loop`** (lane `w-hash`), so a `cflow-loop`
    // under any other in-class key still indicts the measure and under that one
    // is the expected reading.
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
