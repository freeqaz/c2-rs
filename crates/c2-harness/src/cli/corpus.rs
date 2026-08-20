//! P1.2 corpus generator leaves: `corpus gen`, `corpus sample`, `corpus stats`.
//! The `corpus` group dispatcher itself stays in `main.rs` beside the top-level
//! dispatch it belongs to.

use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

use c2_harness::corpus::{self, CorpusConfig};
use c2_harness::toolchain_gate::{toolchain_ready, Cap};

use crate::{Args, Arity, Spec};

static CORPUS_GEN_SPEC: Spec = Spec::new(
    "corpus gen",
    &[
        ("--seed", Arity::Value),
        ("--count", Arity::Value),
        ("--timeout", Arity::Value),
        ("--out", Arity::Value),
    ],
)
.positionals(0);

pub(crate) fn cmd_corpus_gen(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&CORPUS_GEN_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let mut cfg = CorpusConfig::default();
    // `parse().unwrap_or(cfg.seed)` made `--seed abc` run at the DEFAULT seed
    // while the operator believed a seed had been set — a reproducibility claim
    // resting on a value the CLI discarded.
    match args.num("--seed") {
        Ok(Some(v)) => cfg.seed = v,
        Ok(None) => {}
        Err(c) => return c,
    }
    match args.num("--count") {
        Ok(Some(v)) => cfg.count = v,
        Ok(None) => {}
        Err(c) => return c,
    }
    match args.num::<u64>("--timeout") {
        Ok(Some(v)) => cfg.timeout = Duration::from_secs(v),
        Ok(None) => {}
        Err(c) => return c,
    }
    let out = args.path("--out").unwrap_or_else(|| PathBuf::from("corpus"));

    let Some(tc) = args.toolchain() else {
        return ExitCode::SUCCESS;
    };
    if let Some(code) = toolchain_ready(&tc, &[Cap::Strace], "needed to keep the IL bundle") {
        return code;
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

/// One optional output directory, no options. `rest.first()` was taken verbatim,
/// so `c2rs corpus sample --out /tmp/x` wrote the sample into a directory
/// literally named `--out`.
static CORPUS_SAMPLE_SPEC: Spec = Spec::new("corpus sample", &[]);

pub(crate) fn cmd_corpus_sample(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&CORPUS_SAMPLE_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let out = args
        .path_positional()
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

static CORPUS_STATS_SPEC: Spec = Spec::new("corpus stats", &[]);

pub(crate) fn cmd_corpus_stats(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&CORPUS_STATS_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let Some(dir) = args.path_positional() else {
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
