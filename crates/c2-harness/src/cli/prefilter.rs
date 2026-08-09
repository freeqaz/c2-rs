//! `c2rs prefilter` — the reject-only pre-filter seam: one JSON verdict for one
//! candidate TU.

use std::process::ExitCode;

use c2_harness::prefilter;

use crate::{Args, Arity, Spec};
use crate::cli::util::Scratch;

/// `c2rs prefilter` — the reject-only pre-filter seam (see
/// [`c2_harness::prefilter`] for the contract that binds callers).
///
/// Prints exactly one line of JSON on stdout and exits 0 for every well-formed
/// verdict, including `not_implemented`. Exit 2 means "you called me wrong" —
/// a caller must treat that as a hard error, never as a verdict.
static PREFILTER_SPEC: Spec = Spec {
    cmd: "prefilter",
    opts: &[
        ("--source", Arity::Value),
        ("--flag", Arity::Repeated),
        ("--flags-file", Arity::Value),
        ("--cwd", Arity::Value),
        ("--emit-obj", Arity::Value),
        ("--compare-obj", Arity::Value),
        ("--obj-name", Arity::Value),
        ("--work", Arity::Value),
        ("--schema", Arity::Flag),
    ],
    requires: &[],
    max_positionals: 0,
};

pub(crate) fn cmd_prefilter(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&PREFILTER_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    // `--schema` used to SHORT-CIRCUIT mid-parse: it printed and returned from
    // inside the option loop, so every argument after it — including an unknown
    // one — was never examined. `prefilter --schema --typo` exited 0. The whole
    // command line is parsed first now, and only then does `--schema` win.
    if args.has("--schema") {
        println!("{}", prefilter::SCHEMA);
        return ExitCode::SUCCESS;
    }
    let source = args.get("--source").map(String::from);
    let mut flags: Vec<String> = args.all("--flag").into_iter().map(String::from).collect();
    let flags_file = args.path("--flags-file");
    let cwd = args.path("--cwd");
    let emit_obj = args.path("--emit-obj");
    let compare_obj = args.path("--compare-obj");
    let obj_name = args.get("--obj-name").map(String::from);
    let work = args.path("--work");

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

    // Captured IL bundles are large and this runs per candidate; the JSON (and
    // the emitted obj, which lives wherever the caller asked) is the record, so
    // the working dir goes away. It used to go away even when the caller named
    // it: the old spelling cloned the path *after* `unwrap_or_else` and passed
    // the clone to an unconditional `remove_dir_all`, so `prefilter --work DIR`
    // deleted DIR -- a directory the harness did not create. `Scratch` removes
    // only what it minted, and drops on the two error returns above's successors
    // as well as this one.
    let work = Scratch::or_work(work, "prefilter");
    let req = prefilter::Request {
        source,
        flags,
        cwd,
        emit_obj,
        compare_obj,
        obj_name,
        work: work.path().to_path_buf(),
    };
    // `toolchain_quiet`, not `toolchain`: this command emits one line of JSON
    // and reports toolchain absence *inside* it, so a bare `SKIP:` line would
    // corrupt the output it is contracted to produce.
    let out = prefilter::run(args.toolchain_quiet().as_ref(), &req);
    println!("{}", out.to_json());
    ExitCode::SUCCESS
}
