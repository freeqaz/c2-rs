//! **The standing fence-site census** — `w-mutcensus` **F4**, taken at last.
//!
//! # The item
//!
//! `w-mutcensus` measured **X = 30 of 63** `crates/c2-il` fence sites with no
//! test that can fail on them, and then recorded, twice, that its own
//! enumeration had gone **stale inside the lane's own wall clock**: peer
//! `w-fence163` landed a twentieth fence-key constant (`DATA_SYM_STRLIT_FENCED`)
//! while the campaign was running, and peer `w-npos` rewrote four of the five
//! files it enumerates. Its F4 asked for the cheap standing version — *"a gate
//! row that compares that count against a checked-in expectation and fails when
//! a fence lands without the census being re-scored"* — and could not land it,
//! because that lane's success criterion was a required-zero byte delta.
//! `w-calleeguard` landed the F4 shape for **one** dispatch
//! (`tests/callee_unresolved_sites.rs`) and recorded that both of F4's blockers
//! had expired. This is the general version.
//!
//! # What it counts, and why it is keyed on the KEY STRING
//!
//! One row per **census fence key**, and the row's identity is the **published
//! key string** — `"store-run-bind-group-shape"` — not the constant name
//! `STORE_RUN_BIND_GROUP_SHAPE`. That is the whole point of the keying, and it
//! is `w-guards`' rule applied to a counting test: *a guard on the constant
//! passes a mutation that renames the constant and its uses while the published
//! key moves.* Here the direction that matters is the mirror of it — renaming
//! the constant alone must **not** fail (nothing observable moved), while
//! moving the string a scan reports **must**.
//!
//! # Why a per-key table and not one integer
//!
//! F4 asked for "a count". A single integer is satisfied by any change that
//! adds one site and removes another, which is precisely the shape of a
//! refactor that quietly moves a fence from a guarded raise site to an
//! unguarded one. The expectation here is a **table**: every fence key with the
//! number of raise sites it has, plus the two textual populations
//! `w-mutcensus` §2 enumerated (`refuse("…")` literal-key sites, `Block::at_end(`
//! sites). A move shows up as two changed rows and the failure message names
//! both.
//!
//! # How to respond when this test fails
//!
//! It is **not** a bug in this test. A row moved because somebody changed a
//! fence. The response is:
//!
//! 1. Score the new or moved site — is it guarded? `w-mutcensus`' method is one
//!    registered mutation per site against the workspace suite.
//! 2. Land a rung that says so.
//! 3. Update the row here **in the same commit**, with the rung named.
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

/// **The expectation.** `(published key string, raise sites)`.
///
/// Measured at master `1744ced1` by lane `w-deadsites`, and independently
/// cross-checked against `docs/rungs/2026-08-18-calleeguard.md` §4.2's
/// hand-counted table at `44794fa4`, which reports the same distribution:
/// one key at 0 sites, fourteen at 1, two at 2, one at 4.
const EXPECTED: &[(&str, usize)] = &[
    ("callee-defined-in-tu", 1),
    ("callee-unresolved-call-sequence", 1),
    ("callee-unresolved-dtor-delegation", 1),
    ("callee-unresolved-framed-call", 1),
    ("callee-unresolved-tail-call", 1),
    ("data-sym-not-extern", 1),
    ("data-sym-strlit-fenced", 2),
    ("data-sym-unresolved", 1),
    ("opt-mode", 1),
    ("static-scan-loop-object-out-of-class", 1),
    ("store-run-bind-address-producer", 1),
    ("store-run-bind-call-tail-mr-slot", 0),
    ("store-run-bind-group-shape", 4),
    ("store-run-bind-mixed-kind-alloc", 1),
    ("store-run-bind-multi-producer", 2),
    ("store-run-bind-no-emitter-carrier", 1),
    ("store-run-bind-symbol-crossings", 1),
    ("store-run-call-no-emitter-carrier", 1),
];

/// The five **dispatch / production axis tags**. They are declared beside the
/// fence keys and read by `dispatch_site()` / `prod_site()`; none of them ever
/// reaches `Block::at_end`, so they are not fences and are excluded by name
/// rather than by a heuristic.
const AXIS_TAGS: &[&str] = &[
    "DISP_NOT_RUN",
    "PROD_NOT_ENTERED",
    "PROD_ENTERED_UNTAGGED",
    "PROD_ACCEPTED",
    "PROD_COMMITTED_REFUSAL",
];

/// `w-mutcensus` §2's **E1** — `refuse("<key>")` raise sites, whose key is a
/// literal rather than a constant, so the table above cannot see them.
const EXPECTED_REFUSE_SITES: usize = 23;

/// `w-mutcensus` §2's **E3** — `Block::at_end(` sites.
const EXPECTED_AT_END_SITES: usize = 12;

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
        out.len() >= 10,
        "crates/c2-il/src has {} .rs files — the walk found nothing, which would \
         make every count below zero and this whole test vacuous",
        out.len()
    );
    out
}

/// Production lines only.
///
/// Drops, in this order:
///
/// * every `#[cfg(test)]` module — from a `#[cfg(test)]` at **column 0** to the
///   next `}` at column 0. Brace-matched at the margin rather than cut at the
///   first occurrence, because `bundle.rs` carries **three** test modules with
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
        // Strip a trailing `// …` comment so a constant named only in one
        // cannot be counted as a raise.
        let code = match t.find("//") {
            Some(ix) => &t[..ix],
            None => t,
        };
        out.push((i + 1, code.to_string()));
    }
    out
}

/// Whole-identifier occurrences of `name` in `code`.
fn ident_hits(code: &str, name: &str) -> usize {
    let b = code.as_bytes();
    let n = name.as_bytes();
    let ident = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
    let mut hits = 0;
    let mut i = 0;
    while i + n.len() <= b.len() {
        if &b[i..i + n.len()] == n
            && (i == 0 || !ident(b[i - 1]))
            && (i + n.len() == b.len() || !ident(b[i + n.len()]))
        {
            hits += 1;
            i += n.len();
        } else {
            i += 1;
        }
    }
    hits
}

/// `(constant name -> published key string)` for every `pub(crate) const … :
/// &str` in `func/body/mod.rs`, minus the five axis tags.
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
        let key = &tail[open + 1..open + 1 + close];
        if AXIS_TAGS.contains(&name) {
            continue;
        }
        out.insert(name.to_string(), key.to_string());
    }
    out
}

/// Raise sites per constant name, over every production line of `c2-il`.
fn raise_sites(names: &[String]) -> BTreeMap<String, Vec<String>> {
    let mut out: BTreeMap<String, Vec<String>> =
        names.iter().map(|n| (n.clone(), Vec::new())).collect();
    let root = repo_root();
    for f in c2_il_sources() {
        let text = std::fs::read_to_string(&f).unwrap_or_else(|e| panic!("{}: {e}", f.display()));
        let rel = f.strip_prefix(&root).unwrap_or(&f).display().to_string();
        for (lineno, code) in production_lines(&text) {
            // The declaration itself is not a raise.
            if code.starts_with("pub(crate) const ") {
                continue;
            }
            for name in names {
                for _ in 0..ident_hits(&code, name) {
                    out.get_mut(name).expect("seeded").push(format!("{rel}:{lineno}"));
                }
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------

#[test]
fn every_census_fence_key_has_the_number_of_raise_sites_this_repo_last_scored() {
    let consts = fence_key_constants();
    let names: Vec<String> = consts.keys().cloned().collect();
    let sites = raise_sites(&names);

    // (1) The key strings themselves. This is the binding that makes a witness
    //     asserting `"data-sym-strlit-fenced:eof"` and this counting test ONE
    //     guard rather than two that can drift apart.
    let mut declared: Vec<(String, usize)> = Vec::new();
    for (name, key) in &consts {
        declared.push((key.clone(), sites[name].len()));
    }
    declared.sort();

    let expected: Vec<(String, usize)> = EXPECTED
        .iter()
        .map(|(k, n)| ((*k).to_string(), *n))
        .collect();

    if declared != expected {
        let mut msg = String::from(
            "THE FENCE-SITE CENSUS MOVED.\n\n\
             This is not a bug in this test. A row below changed because somebody \
             changed a fence in `crates/c2-il`, and `docs/rungs/2026-08-17-mutcensus.md`'s \
             X/N is a fact about a COMMIT: it goes stale the moment a fence lands \
             (that lane recorded its own frame going stale TWICE inside its own \
             wall clock).\n\n\
             Do this, in one commit:\n  \
             1. score the new or moved site — is any test able to fail on it?\n  \
             2. land a rung that says so;\n  \
             3. update the row in `EXPECTED` here, naming that rung.\n\n\
             rows that differ (key string, raise sites):\n",
        );
        let mut all: Vec<String> = declared
            .iter()
            .map(|(k, _)| k.clone())
            .chain(expected.iter().map(|(k, _)| k.clone()))
            .collect();
        all.sort();
        all.dedup();
        for k in all {
            let got = declared.iter().find(|(a, _)| *a == k).map(|(_, n)| *n);
            let want = expected.iter().find(|(a, _)| *a == k).map(|(_, n)| *n);
            if got != want {
                msg.push_str(&format!("  {k:<40} expected {want:?}  got {got:?}\n"));
                if let Some(name) = consts.iter().find(|(_, v)| **v == k).map(|(n, _)| n) {
                    for s in &sites[name] {
                        msg.push_str(&format!("      site: {s}\n"));
                    }
                }
            }
        }
        panic!("{msg}");
    }

    // (2) The totals, restated so a reader of a failure sees the headline.
    let total: usize = declared.iter().map(|(_, n)| n).sum();
    assert_eq!(
        total, 22,
        "22 raise sites over {} census fence keys is what `w-deadsites` measured \
         at 1744ced1 and what `2026-08-18-calleeguard.md` §4.2 counted by hand \
         at 44794fa4; got {total}",
        declared.len()
    );
    assert_eq!(
        declared.len(),
        18,
        "18 census fence keys — 23 `pub(crate) const … : &str` in \
         `func/body/mod.rs` minus the 5 dispatch/production axis tags"
    );
}

#[test]
fn a_key_with_no_fence_at_all_is_still_counted_and_still_zero() {
    // `w-mutcensus` **F5**: `STORE_RUN_BIND_CALL_TAIL_RETIRED` is a fence key
    // with **zero** live raise sites, test-only since #1212's correction. It is
    // the inverse of this lane's question — a key with no fence is as invisible
    // to every instrument as a fence with no test — and the only way it stays
    // visible is by being a row here rather than an absence.
    let consts = fence_key_constants();
    let names: Vec<String> = consts.keys().cloned().collect();
    let sites = raise_sites(&names);
    let retired = "STORE_RUN_BIND_CALL_TAIL_RETIRED";
    assert!(
        consts.contains_key(retired),
        "{retired} is declared in `func/body/mod.rs` and must stay enumerated \
         even at zero sites — deleting the constant is a decision, and it should \
         fail this test rather than pass silently"
    );
    assert_eq!(
        sites[retired].len(),
        0,
        "{retired} is F5's key with no fence. If it grew a raise site, something \
         re-armed a refusal #1212 retired, and that needs a rung: {:?}",
        sites[retired]
    );
    assert_eq!(
        consts[retired], "store-run-bind-call-tail-mr-slot",
        "…and its published key string is part of the row"
    );
}

#[test]
fn the_two_textual_fence_populations_are_the_size_the_census_enumerated() {
    // `w-mutcensus` §2's E1 and E3. These do not go through a constant, so the
    // per-key table above is blind to them by construction — which is exactly
    // why that lane enumerated them separately, and why leaving them out here
    // would let a whole family land unwatched.
    let mut refuse = 0usize;
    let mut at_end = 0usize;
    let mut refuse_where: Vec<String> = Vec::new();
    let root = repo_root();
    for f in c2_il_sources() {
        let text = std::fs::read_to_string(&f).expect("read");
        let rel = f.strip_prefix(&root).unwrap_or(&f).display().to_string();
        for (lineno, code) in production_lines(&text) {
            // `refuse("…")` — the closure `calls.rs` binds to
            // `Block::refuse(seg, off, ctx)`. Counted only where the argument is
            // a literal, which is what makes the key invisible to the table.
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
        "`refuse(\"…\")` literal-key raise sites moved ({} -> {refuse}). \
         `w-mutcensus` §2 E1 counted 23, all in `func/body/shapes/calls.rs`. \
         Sites now: {refuse_where:?}",
        EXPECTED_REFUSE_SITES
    );
    assert_eq!(
        at_end, EXPECTED_AT_END_SITES,
        "`Block::at_end(` sites moved ({} -> {at_end}) — `w-mutcensus` §2 E3. \
         Every one of these renders a `:eof` key a scan reports, so one landing \
         unscored is a published key nothing measured",
        EXPECTED_AT_END_SITES
    );
}

#[test]
fn the_enumerator_itself_is_not_vacuous() {
    // STATUS trap 5, applied to this file: a walker that found nothing, a
    // stripper that dropped everything, or an `ident_hits` that never matched
    // would make all three tests above pass on an empty population. Each of the
    // three stages is asserted to have produced something, and the stripper is
    // asserted to have actually stripped.
    let files = c2_il_sources();
    assert!(files.len() >= 10, "walker: {} files", files.len());

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
    // The `#[cfg(test)]` module really is gone: `mod tests` is inside it.
    assert!(
        !prod.iter().any(|(_, c)| c.contains("mod tests")),
        "stripper left the test module in"
    );

    assert_eq!(ident_hits("Err(STORE_RUN_BIND_GROUP_SHAPE)", "STORE_RUN_BIND_GROUP_SHAPE"), 1);
    assert_eq!(ident_hits("STORE_RUN_BIND_GROUP_SHAPE_X", "STORE_RUN_BIND_GROUP_SHAPE"), 0);
    assert_eq!(ident_hits("A, A", "A"), 2);
    assert_eq!(ident_hits("", "A"), 0);
}
