//! Board #132's listing seam: `listing` (one TU's `.cod`) and `listing-scan`
//! (boards #134/#136, the population scan over it).

use std::path::PathBuf;
use std::process::ExitCode;

use crate::{Args, Arity, Spec};
use crate::cli::util::scratch;

/// **Board #132 — the listing seam.** Capture one TU and print (or write) c2's
/// own `.cod` assembly listing beside the obj the differential grades.
///
/// The listing is a **decode aid, never a gate**: the obj byte-compare remains
/// the sole judge of the port.
static LISTING_SPEC: Spec = Spec::new(
    "listing",
    &[
        ("--qxstalls", Arity::Flag),
        ("--out", Arity::Value),
        // Repeatable: `--flag /GR- --flag /Ox` builds the profile.
        ("--flag", Arity::Repeated),
    ],
);

pub(crate) fn cmd_listing(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&LISTING_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let out = args.path("--out");
    let qxstalls = args.has("--qxstalls");
    let mut flags: Vec<String> = args.all("--flag").into_iter().map(String::from).collect();
    let Some(cpp) = args.first().map(PathBuf::from) else {
        eprintln!(
            "usage: c2rs listing <cpp> [--qxstalls] [--out PATH] [--flag F ...]\n\
             default flags: /O1 /Oi /EHsc /GS- /c"
        );
        return ExitCode::from(2);
    };
    let Some(tc) = args.toolchain() else {
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
static LISTING_SCAN_SPEC: Spec = Spec::new(
    "listing-scan",
    &[
        ("--qxstalls", Arity::Flag),
        ("--list", Arity::Value),
        ("--flags-file", Arity::Value),
        ("--cwd", Arity::Value),
        ("--limit", Arity::Value),
        ("--jobs", Arity::Value),
        ("--jsonl", Arity::Value),
        ("--work", Arity::Value),
    ],
)
.positionals(0);

pub(crate) fn cmd_listing_scan(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&LISTING_SCAN_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let cwd = args.path("--cwd");
    let jsonl = args.path("--jsonl");
    let work = args.path("--work");
    let qxstalls = args.has("--qxstalls");
    // `--limit`/`--jobs` used to `return ExitCode::from(2)` on an unparseable
    // value with NO message at all — the `{name} needs a value` line only fired
    // when the token was missing entirely, so `--jobs eight` exited 2 in
    // silence. `num` names the option and the value.
    let limit: Option<usize> = match args.num("--limit") {
        Ok(v) => v,
        Err(c) => return c,
    };
    let jobs: usize = match args.num("--jobs") {
        Ok(Some(v)) => v,
        Ok(None) => std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4),
        Err(c) => return c,
    };
    let (list_file, flags_file) = (args.path("--list"), args.path("--flags-file"));
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
    let Some(tc) = args.toolchain() else {
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
