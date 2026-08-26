//! **The standing fence-site census** — `w-mutcensus` **F4**, taken at last.
//!
//! # The item
//!
//! `w-mutcensus` measured **X = 30 of 63** `crates/c2-il` fence sites with no
//! test that can fail on them, and then recorded, twice, that its own
//! enumeration had gone **stale inside the lane's own wall clock**: peer
//! `w-fence163` landed a fence-key constant (`DATA_SYM_STRLIT_FENCED`) while the
//! campaign was running, and peer `w-npos` rewrote four of the five files it
//! enumerates. Its F4 asked for the cheap standing version — *"a gate row that
//! compares that count against a checked-in expectation and fails when a fence
//! lands without the census being re-scored"* — and could not land it, because
//! that lane's success criterion was a required-zero byte delta.
//! `w-calleeguard` landed the F4 shape for **one** dispatch
//! (`tests/callee_unresolved_sites.rs`) and recorded that both of F4's blockers
//! had expired. This is the general version.
//!
//! # Why it PARSES instead of grepping, and what that immediately caught
//!
//! Both prior enumerations of `func/body/mod.rs`' fence-key constants used the
//! pattern `pub(crate) const [A-Z_]*: &str`. **That character class excludes a
//! digit**, so both silently missed `PTR_WALK_LOOP_NOT_O1` and
//! `PTR_WALK_CHAIN_LOOP_NOT_O1` — the `_O1` suffix is the whole of it. That is
//! why `docs/rungs/2026-08-18-calleeguard.md` §4.2 reports **18** census fence
//! keys over **22** raise sites where this file measures **20** over **24**: the
//! difference is exactly those two keys at one raise site each, and the two
//! readings reconcile to the digit. Board **#3269**'s rule — *a lane that finds
//! an unexpected delta owes a measurement before it owes a cause* — is why that
//! sentence is a reconciliation and not an accusation.
//!
//! # What it counts, and why it is keyed on the KEY STRING
//!
//! One row per **census fence key**, and the row's identity is the **published
//! key string** — `"store-run-bind-group-shape"` — never the constant name
//! `STORE_RUN_BIND_GROUP_SHAPE`. That keying is `w-guards`' rule applied to a
//! counting test: *a guard on the constant passes a mutation that renames the
//! constant and its uses while the published key moves.* Both directions are
//! demonstrated in `docs/rungs/2026-08-18-deadsites.md` §6 — renaming the
//! constant everywhere leaves this test GREEN (nothing observable moved) and
//! moving the string turns it RED.
//!
//! Each row carries **two** numbers:
//!
//! * **raises** — mentions in production code that produce the key;
//! * **reads** — mentions in a comparison (`== NAME`, `!= NAME`), which consume
//!   the key rather than raising it. There is exactly one in the tree today
//!   (`OPT_MODE`, in `Block::key`'s renderer) and it is a row rather than an
//!   exclusion, because a rule with a silent exception is a rule nobody can
//!   re-derive.
//!
//! # Why a per-key table and not one integer
//!
//! F4 asked for "a count". A single integer is satisfied by any change that adds
//! one site and removes another — precisely the shape of a refactor that moves a
//! fence from a guarded raise site to an unguarded one. `w-deadsites`'
//! mutant `MC1` is that change, made on purpose: it moves one site from
//! `store-run-bind-multi-producer` to `store-run-bind-mixed-kind-alloc`, leaving
//! the total at 24, and a count-shaped census cannot see it.
//!
//! # How to respond when this test fails
//!
//! It is **not** a bug in this test. A row moved because somebody changed a
//! fence. In one commit: score the new or moved site, land a rung that says so,
//! and update the row here naming that rung.
//!
//! Portable: reads source text only, no toolchain, no capture.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The repository root, from this crate's manifest directory
/// (`crates/c2-harness/../..`), exactly as the rest of the harness resolves it.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/c2-harness/../.. is the repo root")
        .to_path_buf()
}

/// **The expectation.** `(published key string, raise sites, comparison reads)`.
///
/// Measured at master `1744ced1` by lane `w-deadsites`
/// (`docs/rungs/2026-08-18-deadsites.md`).
const EXPECTED: &[(&str, usize, usize)] = &[
    // **W-ATEND, `docs/rungs/2026-08-26-w-atend.md`** — the ADMISSION layer's
    // own refusal reason, raised by `AdmissionPolicy::Nothing` for a body the
    // decode read WHOLE. It is the one key in this table that a production scan
    // does **not** reach, because no production call site selects a non-default
    // policy; see `EXPECTED_AT_END_SITES_BY_FILE` for the partition that keeps
    // that fact visible instead of absorbed into a total.
    ("admission-declined", 1, 0),
    ("callee-defined-in-tu", 1, 0),
    ("callee-unresolved-call-sequence", 1, 0),
    ("callee-unresolved-dtor-delegation", 1, 0),
    ("callee-unresolved-framed-call", 1, 0),
    ("callee-unresolved-tail-call", 1, 0),
    ("data-sym-not-extern", 1, 0),
    ("data-sym-strlit-fenced", 2, 0),
    ("data-sym-unresolved", 1, 0),
    ("opt-mode", 1, 1),
    ("ptr-walk-chain-loop-not-o1", 1, 0),
    ("ptr-walk-loop-not-o1", 1, 0),
    ("static-scan-loop-object-out-of-class", 1, 0),
    ("store-run-bind-address-producer", 1, 0),
    ("store-run-bind-call-tail-mr-slot", 0, 0),
    // 4 -> 3: `leaf_store.rs:2456` was PROVED DEAD and DELETED by lane
    // `w-deadsites` (board #3277). Updating this row in the same commit as the
    // deletion is exactly the workflow this file's failure message prescribes,
    // and that deletion is the first thing it caught.
    ("store-run-bind-group-shape", 3, 0),
    ("store-run-bind-mixed-kind-alloc", 1, 0),
    ("store-run-bind-multi-producer", 2, 0),
    ("store-run-bind-no-emitter-carrier", 1, 0),
    ("store-run-bind-symbol-crossings", 1, 0),
    ("store-run-call-no-emitter-carrier", 1, 0),
];

/// The five **dispatch / production axis tags**. They are declared beside the
/// fence keys and read by `dispatch_site()` / `prod_site()`; none of them ever
/// reaches `Block::at_end`, so they are not fences and are excluded **by name**
/// rather than by a heuristic.
const AXIS_TAGS: &[&str] = &[
    "DISP_NOT_RUN",
    "PROD_NOT_ENTERED",
    "PROD_ENTERED_UNTAGGED",
    "PROD_ACCEPTED",
    "PROD_COMMITTED_REFUSAL",
];

/// `w-mutcensus` §2's **E1** — `refuse("<key>")` raise sites, whose key is a
/// literal rather than a constant, so the table above cannot see them. That
/// lane counted **23**; this rule reproduces it exactly.
const EXPECTED_REFUSE_SITES: usize = 23;

/// `Block::at_end(` sites in **production** code — `w-mutcensus` §2's **E3**
/// asked over the whole file including tests and doc comments and got a larger
/// raw number; this is the same population under this file's stated rule, and
/// the two are not the same measurement.
///
/// **7 → 8 by lane `w-atend`** (`docs/rungs/2026-08-26-w-atend.md`, board
/// **#3592**), which is the named follow-up board **#3556** left. Read
/// `EXPECTED_AT_END_SITES_BY_FILE` below before quoting this total: the 8 are
/// **not** one population any more.
const EXPECTED_AT_END_SITES: usize = 8;

/// **THE PARTITION, AND IT IS THE POINT OF THE 7 → 8 RATHER THAN A DECORATION.**
///
/// The rule this file used to state about `Block::at_end(` — *"every one
/// renders a `:eof` key a scan reports, so one landing unscored is a published
/// key nothing measured"* — was true of all 7 and is **not** true of all 8.
///
/// * `crates/c2-il/src/func/census.rs` — **7**, the post-parse gates. A
///   production scan reaches every one of their keys, and a peer sized the
///   population: board **#3582** measured `decode-reach-grammar-not-admitted`
///   at **4,001** bodies over the 878-TU workload under five `:eof` keys.
/// * `crates/c2-il/src/func/body/decode.rs` — **1**, `ADMISSION_DECLINED`,
///   raised only under `AdmissionPolicy::Nothing`. **No production call site
///   selects a non-default policy**, so no scan reaches it: it is an
///   *instrument state* in the sense of `docs/rungs/README.md` § Lane kinds,
///   THE DECISION-SURFACE CLAUSE.
///
/// Bumping the bare total to `8` would have hidden exactly the hazard #3556
/// correctly identified — a published key nothing measures — inside a number
/// that still looked like one population. Keying the expectation on the FILE is
/// the same move this file's header already makes for the per-key table
/// (*"a single integer is satisfied by any change that adds one site and
/// removes another"*), one axis over: a refactor that moved a fence from
/// `census.rs` into `decode.rs` would leave the total at 8 and be invisible.
///
/// **The scoring of the instrument-only site**, which is what this file's own
/// failure message demands before a row moves: three tests in
/// `crates/c2-il/src/func/body/decode.rs` fail if it moves —
/// `the_admission_layer_owns_a_reason_only_where_it_alone_refused` pins the
/// rendered key `admission-declined:eof`, its offset and its completeness;
/// `the_yes_no_and_the_emitting_form_cannot_disagree_under_any_policy` sweeps
/// every policy; `all_is_complete_and_indexed` refuses a variant that is
/// indexed but unlisted. **The day a production caller does select a
/// non-default policy, this row moves back into the reachable column** — and
/// that caller owes its own two-sided price for widening the decision surface
/// into production.
const EXPECTED_AT_END_SITES_BY_FILE: &[(&str, usize)] = &[
    ("crates/c2-il/src/func/body/decode.rs", 1),
    ("crates/c2-il/src/func/census.rs", 7),
];

/// How many of [`EXPECTED_AT_END_SITES`] are reachable only under a
/// **non-default** parameter — i.e. are instrument states rather than sites a
/// scan's key histogram can ever show. Stated as its own number so that
/// "how many published keys does nothing measure" is a *quantity* on this page
/// and not a paragraph.
const EXPECTED_AT_END_SITES_INSTRUMENT_ONLY: usize = 1;

// ---------------------------------------------------------------------------
// The enumeration rule, spelled out in code because a rule in prose drifts.
// ---------------------------------------------------------------------------

/// Every `.rs` file under `crates/c2-il/src`, sorted.
fn c2_il_sources() -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![repo_root().join("crates/c2-il/src")];
    while let Some(d) = stack.pop() {
        let rd = std::fs::read_dir(&d).unwrap_or_else(|e| panic!("read_dir {}: {e}", d.display()));
        for ent in rd {
            let p = ent.expect("dir entry").path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|e| e == "rs") {
                out.push(p);
            }
        }
    }
    out.sort();
    assert!(
        out.len() >= 40,
        "crates/c2-il/src has {} .rs files — the walk found almost nothing, \
         which would make every count below zero and this whole file vacuous",
        out.len()
    );
    out
}

/// Production lines only, as `(1-based line number, code with any trailing
/// `//` comment removed)`.
///
/// Drops, in this order:
///
/// * every `#[cfg(test)]` module — from a `#[cfg(test)]` at **column 0** to the
///   next `}` at column 0. Matched at the margin rather than cut at the first
///   occurrence, because `bundle.rs` carries **three** test modules with
///   production code between them and a first-occurrence cut would silently
///   drop `data_tu`'s siblings;
/// * `//`-comments (including `///` and `//!`) and `/* … */` blocks;
/// * `use` statements, which mention a constant without raising it — including
///   the multi-line brace form `use crate::func::body::{ A, B, C };`.
fn production_lines(text: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut in_test_mod = false;
    let mut in_block_comment = false;
    let mut in_use = false;
    for (i, raw) in text.lines().enumerate() {
        let line = raw.trim_end();
        if in_test_mod {
            if line == "}" {
                in_test_mod = false;
            }
            continue;
        }
        if line.starts_with("#[cfg(test)]") {
            in_test_mod = true;
            continue;
        }
        let t = line.trim_start();
        if in_block_comment {
            if t.contains("*/") {
                in_block_comment = false;
            }
            continue;
        }
        if t.starts_with("/*") && !t.contains("*/") {
            in_block_comment = true;
            continue;
        }
        if t.starts_with("//") {
            continue;
        }
        if in_use {
            if t.contains(';') {
                in_use = false;
            }
            continue;
        }
        if t.starts_with("use ") || t.starts_with("pub use ") {
            if !t.contains(';') {
                in_use = true;
            }
            continue;
        }
        let code = match t.find("//") {
            Some(ix) => &t[..ix],
            None => t,
        };
        out.push((i + 1, code.trim_end().to_string()));
    }
    out
}

/// Byte offsets of whole-identifier occurrences of `name` in `code`.
fn ident_hits(code: &str, name: &str) -> Vec<usize> {
    let b = code.as_bytes();
    let n = name.as_bytes();
    let ident = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
    let mut hits = Vec::new();
    let mut i = 0;
    while i + n.len() <= b.len() {
        if &b[i..i + n.len()] == n
            && (i == 0 || !ident(b[i - 1]))
            && (i + n.len() == b.len() || !ident(b[i + n.len()]))
        {
            hits.push(i);
            i += n.len();
        } else {
            i += 1;
        }
    }
    hits
}

/// Is the occurrence at `at` a COMPARISON against the constant rather than a
/// production of it?
fn is_comparison(code: &str, at: usize, len: usize) -> bool {
    let before = code[..at].trim_end();
    let after = code[at + len..].trim_start();
    before.ends_with("==") || before.ends_with("!=") || after.starts_with("==") || after.starts_with("!=")
}

/// `(constant name -> published key string)` for every `pub(crate) const … :
/// &str` in `func/body/mod.rs`, minus the five axis tags.
///
/// Parsed, not grepped — see this file's header for the two keys a
/// `[A-Z_]`-classed grep drops.
fn fence_key_constants() -> BTreeMap<String, String> {
    let p = repo_root().join("crates/c2-il/src/func/body/mod.rs");
    let text = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    let mut out = BTreeMap::new();
    for (_, code) in production_lines(&text) {
        let Some(rest) = code.strip_prefix("pub(crate) const ") else {
            continue;
        };
        let Some((name, tail)) = rest.split_once(':') else {
            continue;
        };
        let name = name.trim();
        if !tail.trim_start().starts_with("&str") {
            continue;
        }
        let Some(open) = tail.find('"') else { continue };
        let Some(close) = tail[open + 1..].find('"') else {
            continue;
        };
        if AXIS_TAGS.contains(&name) {
            continue;
        }
        out.insert(name.to_string(), tail[open + 1..open + 1 + close].to_string());
    }
    out
}

/// `constant name -> (raise sites, comparison reads)`, each as `file:line`.
fn sites(names: &[String]) -> BTreeMap<String, (Vec<String>, Vec<String>)> {
    let mut out: BTreeMap<String, (Vec<String>, Vec<String>)> = names
        .iter()
        .map(|n| (n.clone(), (Vec::new(), Vec::new())))
        .collect();
    let root = repo_root();
    for f in c2_il_sources() {
        let text = std::fs::read_to_string(&f).unwrap_or_else(|e| panic!("{}: {e}", f.display()));
        let rel = f.strip_prefix(&root).unwrap_or(&f).display().to_string();
        for (lineno, code) in production_lines(&text) {
            if code.starts_with("pub(crate) const ") {
                continue; // the declaration is not a use of any kind
            }
            for name in names {
                for at in ident_hits(&code, name) {
                    let entry = out.get_mut(name).expect("seeded");
                    let where_ = format!("{rel}:{lineno}");
                    if is_comparison(&code, at, name.len()) {
                        entry.1.push(where_);
                    } else {
                        entry.0.push(where_);
                    }
                }
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------

#[test]
fn every_census_fence_key_has_the_sites_this_repo_last_scored() {
    let consts = fence_key_constants();
    let names: Vec<String> = consts.keys().cloned().collect();
    let found = sites(&names);

    let mut declared: Vec<(String, usize, usize)> = consts
        .iter()
        .map(|(name, key)| (key.clone(), found[name].0.len(), found[name].1.len()))
        .collect();
    declared.sort();

    let expected: Vec<(String, usize, usize)> = EXPECTED
        .iter()
        .map(|(k, r, c)| ((*k).to_string(), *r, *c))
        .collect();

    if declared != expected {
        let mut msg = String::from(
            "THE FENCE-SITE CENSUS MOVED.\n\n\
             This is not a bug in this test. A row below changed because somebody \
             changed a fence in `crates/c2-il`, and \
             `docs/rungs/2026-08-17-mutcensus.md`'s X/N is a fact about a COMMIT: \
             it goes stale the moment a fence lands, and that lane recorded its own \
             frame going stale TWICE inside its own wall clock.\n\n\
             Do this, in ONE commit:\n  \
             1. score the new or moved site — can any test fail on it?\n  \
             2. land a rung that says so;\n  \
             3. update the row in `EXPECTED` here, naming that rung.\n\n\
             rows that differ — (key string, raises, comparison reads):\n",
        );
        let mut all: Vec<String> = declared
            .iter()
            .map(|(k, _, _)| k.clone())
            .chain(expected.iter().map(|(k, _, _)| k.clone()))
            .collect();
        all.sort();
        all.dedup();
        for k in all {
            let got = declared.iter().find(|(a, _, _)| *a == k).map(|(_, r, c)| (*r, *c));
            let want = expected.iter().find(|(a, _, _)| *a == k).map(|(_, r, c)| (*r, *c));
            if got != want {
                msg.push_str(&format!("  {k:<40} expected {want:?}  got {got:?}\n"));
                if let Some(name) = consts.iter().find(|(_, v)| **v == k).map(|(n, _)| n) {
                    for s in &found[name].0 {
                        msg.push_str(&format!("      raise: {s}\n"));
                    }
                    for s in &found[name].1 {
                        msg.push_str(&format!("      read:  {s}\n"));
                    }
                }
            }
        }
        panic!("{msg}");
    }

    // The headline, restated so a reader of a failure sees it without adding up
    // twenty rows.
    let raises: usize = declared.iter().map(|(_, r, _)| r).sum();
    assert_eq!(
        (declared.len(), raises),
        (21, 24),
        "21 census fence keys over 24 raise sites is what `w-atend` leaves \
         (`docs/rungs/2026-08-26-w-atend.md`, board #3592): it added exactly one \
         key at exactly one raise site, `ADMISSION_DECLINED`. Before it, \
         `w-deadsites` left 20/23 — it MEASURED 24 raises at base 1744ced1 and \
         deleted one, `leaf_store.rs:2456`, as provably dead (board #3277). Got \
         {} keys over {raises} raises. `2026-08-18-calleeguard.md` §4.2's 18/22 \
         is that older tree read with a `[A-Z_]`-classed grep, which drops the \
         two `_O1` keys",
        declared.len()
    );
}

#[test]
fn a_key_with_no_fence_at_all_is_still_counted_and_still_zero() {
    // `w-mutcensus` **F5**: `STORE_RUN_BIND_CALL_TAIL_RETIRED` is a fence key
    // with **zero** live raise sites, test-only since #1212's correction. It is
    // the inverse of `w-deadsites`' question — a key with no fence is as
    // invisible to every instrument as a fence with no test — and the only way
    // it stays visible is by being a row here rather than an absence.
    let consts = fence_key_constants();
    let names: Vec<String> = consts.keys().cloned().collect();
    let found = sites(&names);
    let retired = "STORE_RUN_BIND_CALL_TAIL_RETIRED";
    assert!(
        consts.contains_key(retired),
        "{retired} is declared in `func/body/mod.rs` and must stay enumerated \
         even at zero sites — deleting the constant is a decision, and it should \
         fail this test rather than pass silently"
    );
    assert_eq!(
        found[retired].0.len(),
        0,
        "{retired} is F5's key with no fence. If it grew a raise site, something \
         re-armed a refusal #1212 retired, and that needs a rung: {:?}",
        found[retired].0
    );
    assert_eq!(
        consts[retired], "store-run-bind-call-tail-mr-slot",
        "…and its published key string is part of the row"
    );
}

#[test]
fn the_two_textual_fence_populations_are_the_size_this_rule_measured() {
    // `w-mutcensus` §2's E1 and E3. Neither goes through a constant, so the
    // per-key table is blind to them by construction — which is why that lane
    // enumerated them separately, and why leaving them out here would let a
    // whole family land unwatched.
    let mut refuse = 0usize;
    let mut at_end = 0usize;
    let mut refuse_where: Vec<String> = Vec::new();
    let mut at_end_by_file: BTreeMap<String, usize> = BTreeMap::new();
    let root = repo_root();
    for f in c2_il_sources() {
        let text = std::fs::read_to_string(&f).expect("read");
        let rel = f.strip_prefix(&root).unwrap_or(&f).display().to_string();
        for (lineno, code) in production_lines(&text) {
            let mut i = 0;
            while let Some(ix) = code[i..].find("refuse(\"") {
                refuse += 1;
                refuse_where.push(format!("{rel}:{lineno}"));
                i += ix + 8;
            }
            let mut j = 0;
            while let Some(ix) = code[j..].find("Block::at_end(") {
                at_end += 1;
                *at_end_by_file.entry(rel.clone()).or_insert(0) += 1;
                j += ix + 14;
            }
        }
    }
    assert_eq!(
        refuse, EXPECTED_REFUSE_SITES,
        "`refuse(\"…\")` literal-key raise sites moved ({EXPECTED_REFUSE_SITES} \
         -> {refuse}). `w-mutcensus` §2 E1 counted 23, all in \
         `func/body/shapes/calls.rs`, and this rule reproduces it. Sites now: \
         {refuse_where:?}"
    );
    assert_eq!(
        at_end, EXPECTED_AT_END_SITES,
        "`Block::at_end(` production sites moved ({EXPECTED_AT_END_SITES} -> \
         {at_end}). Sites now: {at_end_by_file:?}. \
         **Read `EXPECTED_AT_END_SITES_BY_FILE` before changing this number.** \
         The rule this assertion used to state — *every one renders a `:eof` \
         key a scan reports* — was true of all 7 and is not true of all 8, and \
         the partition below is what keeps that from being hidden in a total"
    );

    // THE PARTITION. Keyed on the FILE, because a refactor that moved a fence
    // from the reachable population into the instrument-only one would leave
    // the total at 8 — the same defect this file's header describes for a
    // count-shaped per-key census, one axis over.
    let declared: Vec<(String, usize)> =
        at_end_by_file.iter().map(|(k, v)| (k.clone(), *v)).collect();
    let expected: Vec<(String, usize)> = EXPECTED_AT_END_SITES_BY_FILE
        .iter()
        .map(|(f, n)| ((*f).to_string(), *n))
        .collect();
    assert_eq!(
        declared, expected,
        "the `Block::at_end(` partition moved. This is not a bug in this test — \
         somebody moved, added or removed a post-parse fence. Score the site \
         (can any test fail on it? does a production scan reach its key, or is \
         it an instrument state?), land a rung, and update \
         `EXPECTED_AT_END_SITES_BY_FILE` in the same commit"
    );
    assert_eq!(
        declared.iter().map(|(_, n)| n).sum::<usize>(),
        at_end,
        "the partition does not sum to the total — the two counts came from one \
         walk and disagreeing is impossible unless this test is broken"
    );

    // …and the instrument-only quantity, named. `decode.rs`' site is reachable
    // only under a non-default `AdmissionPolicy`; nothing in production selects
    // one. If a production caller ever does, this number goes to 0 and the site
    // joins the reachable column — and that caller owes its own price.
    let instrument_only = declared
        .iter()
        .filter(|(f, _)| f == "crates/c2-il/src/func/body/decode.rs")
        .map(|(_, n)| *n)
        .sum::<usize>();
    assert_eq!(
        instrument_only, EXPECTED_AT_END_SITES_INSTRUMENT_ONLY,
        "the number of `Block::at_end(` sites whose key NO scan reaches moved \
         ({EXPECTED_AT_END_SITES_INSTRUMENT_ONLY} -> {instrument_only}). That \
         quantity is board #3556's hazard as a number rather than a paragraph, \
         and it must never move silently in either direction"
    );
}

#[test]
fn the_enumerator_itself_is_not_vacuous() {
    // `docs/STATUS.md` trap 5, applied to this file: a walker that found
    // nothing, a stripper that dropped everything, or an `ident_hits` that
    // never matched would make all three tests above pass over an empty
    // population, in the flattering direction, silently.
    let files = c2_il_sources();
    assert!(files.len() >= 40, "walker: {} files", files.len());

    let p = repo_root().join("crates/c2-il/src/func/body/shapes/leaf_store.rs");
    let text = std::fs::read_to_string(&p).expect("read leaf_store.rs");
    let prod = production_lines(&text);
    assert!(!prod.is_empty(), "stripper returned no production lines");
    assert!(
        prod.len() < text.lines().count(),
        "stripper dropped nothing: {} of {} lines survived, and this file \
         certainly has a `#[cfg(test)]` module and comments",
        prod.len(),
        text.lines().count()
    );
    assert!(
        !prod.iter().any(|(_, c)| c.contains("mod tests")),
        "stripper left the test module in — every count above would then include \
         test code, which is the population `w-mutcensus` explicitly excluded"
    );

    assert_eq!(ident_hits("Err(STORE_RUN_BIND_GROUP_SHAPE)", "STORE_RUN_BIND_GROUP_SHAPE").len(), 1);
    assert_eq!(ident_hits("STORE_RUN_BIND_GROUP_SHAPE_X", "STORE_RUN_BIND_GROUP_SHAPE").len(), 0);
    assert_eq!(ident_hits("A, A", "A").len(), 2);
    assert!(ident_hits("", "A").is_empty());
    assert!(is_comparison("if self.ctx == OPT_MODE {", 15, 8));
    assert!(!is_comparison("Block::at_end(seg, OPT_MODE)", 19, 8));

    // …and the two keys a `[A-Z_]`-classed grep drops are really in the table,
    // stated as an assertion so the header's claim is checked rather than told.
    let consts = fence_key_constants();
    for missed in ["PTR_WALK_LOOP_NOT_O1", "PTR_WALK_CHAIN_LOOP_NOT_O1"] {
        assert!(
            consts.contains_key(missed),
            "{missed} must be enumerated — it is the constant whose `_O1` suffix \
             a `[A-Z_]` character class silently drops, and both prior \
             enumerations of this file dropped it"
        );
    }
}
