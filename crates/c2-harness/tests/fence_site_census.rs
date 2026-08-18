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
const EXPECTED_AT_END_SITES: usize = 7;

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
        (20, 23),
        "20 census fence keys over 23 raise sites is what `w-deadsites` leaves \
         at its tip — it MEASURED 24 at base 1744ced1 and deleted one, \
         `leaf_store.rs:2456`, as provably dead (board #3277). Got {} keys over \
         {raises} raises. `2026-08-18-calleeguard.md` §4.2's 18/22 is the same \
         base tree read with a `[A-Z_]`-classed grep, which drops the two `_O1` keys",
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
         {at_end}). Every one renders a `:eof` key a scan reports, so one \
         landing unscored is a published key nothing measured"
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
