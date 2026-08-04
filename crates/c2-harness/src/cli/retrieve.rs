//! P1.3 retrieval leaves: `retrieve index` and `retrieve eval`. The `retrieve`
//! group dispatcher stays in `main.rs`.

use std::process::ExitCode;

use c2_harness::retrieval;

use crate::{Args, Arity, Spec};

// ---------------------------------------------------------------------------
// P1.3 retrieval baseline
// ---------------------------------------------------------------------------

/// `--k 1,5,10`. A bad element used to vanish through `filter_map(parse().ok())`
/// and, if every element was bad, the whole option fell back to the default —
/// so `--k 1,x,10` silently evaluated recall at two cutoffs, not three.
fn parse_ks(args: &Args) -> Result<Vec<usize>, ExitCode> {
    let Some(s) = args.get("--k") else {
        return Ok(vec![1, 5, 10]);
    };
    let mut v = Vec::new();
    for tok in s.split(',').map(str::trim).filter(|t| !t.is_empty()) {
        match tok.parse::<usize>() {
            Ok(k) => v.push(k),
            Err(_) => {
                eprintln!("{}: --k expects integers, got {tok:?}", args.cmd());
                return Err(ExitCode::from(2));
            }
        }
    }
    if v.is_empty() {
        eprintln!("{}: --k names no cutoffs", args.cmd());
        return Err(ExitCode::from(2));
    }
    Ok(v)
}

static RETRIEVE_INDEX_SPEC: Spec = Spec::new("retrieve index", &[]);

pub(crate) fn cmd_retrieve_index(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&RETRIEVE_INDEX_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let Some(dir) = args.path_positional() else {
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

static RETRIEVE_EVAL_SPEC: Spec = Spec::new(
    "retrieve eval",
    &[
        ("--k", Arity::Value),
        ("--split", Arity::Value),
        ("--query-div", Arity::Value),
    ],
);

pub(crate) fn cmd_retrieve_eval(rest: &[String]) -> ExitCode {
    let args = match Args::parse(&RETRIEVE_EVAL_SPEC, rest) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let Some(dir) = args.path_positional() else {
        eprintln!("usage: c2rs retrieve eval <corpus-dir> [--split held-out|loo] [--query-div N] [--k 1,5,10]");
        return ExitCode::from(2);
    };
    let ks = match parse_ks(&args) {
        Ok(v) => v,
        Err(c) => return c,
    };
    // `_ =>` used to swallow every unrecognised spelling, so `--split heldout`
    // ran leave-one-out's OPPOSITE and reported "held-out" as if asked.
    let split = match args.one_of("--split", &["held-out", "loo"]) {
        Ok(v) => v.unwrap_or("held-out"),
        Err(c) => return c,
    };
    if split == "loo" && args.get("--query-div").is_some() {
        eprintln!("retrieve eval: --query-div has no effect under --split loo; refusing rather than dropping it silently");
        return ExitCode::from(2);
    }
    let query_div: u64 = match args.num("--query-div") {
        Ok(v) => v.unwrap_or(5),
        Err(c) => return c,
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
