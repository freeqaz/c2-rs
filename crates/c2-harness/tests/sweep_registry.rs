//! **The sweep-fragment registry** — a portable lane test, no toolchain needed.
//!
//! `scripts/expr_sweep.sh` is the differential sweep, and its generator half
//! (`scripts/sweep_gen.py`, one locator, also used by `scripts/cross_sweep.py`)
//! was until now exercised by **nothing in `cargo test`**. A fragment that
//! stopped emitting, or two fragments whose case files collided on a name, was
//! visible only to whoever next ran the whole sweep — which takes minutes and a
//! toolchain, and which a rung author can plausibly skip.
//!
//! The bug this guards is not hypothetical. A fragment once shadowed the
//! driver's file counter with a `for n in …`, silently rewound it, and
//! **overwrote 1,233 already-written cases**; the sweep then reported a green
//! run over the survivors (`docs/GAPS.md` §6, `docs/ARCHITECTURE_SEAMS.md`
//! §2.4). The counter is now owned by the loader and the trap is
//! unrepresentable — but the *observable symptom* of that class of bug is a
//! count that does not match what is on disk, and this test is what makes the
//! loader's own assertion of that a gate rather than a courtesy.
//!
//! What it asserts, over `scripts/sweep.d/`:
//!
//! 1. the loader runs clean and its printed per-fragment counts sum to its
//!    printed total;
//! 2. **printed = generated = on disk**, per fragment and overall — the loader
//!    fails on a mismatch and this test fails with it;
//! 3. **no fragment emits zero cases** (the counter bug's symptom);
//! 4. every fragment is named `NN-<slug>.py` and defines `cases(emit)`, so a
//!    file dropped into the directory that the loader would silently ignore is
//!    a failure instead;
//! 5. generation is **deterministic** — two runs produce byte-identical
//!    corpora. The sweep's own totals are quoted in every rung's gate table and
//!    are only comparable if that holds.
//!
//! What it deliberately does **not** assert: that every accepted shape family
//! has a fragment covering it. That question can only be answered by *compiling*
//! — the family of a case is the port's own verdict on it — and it is
//! `scripts/cross_sweep.py`'s job, which fails by name on a family no fragment
//! can supply a representative for. Asserting it here from a hardcoded list
//! would be a green light that means nothing the day the list drifts.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/c2-harness/../.. is the repo root")
        .to_path_buf()
}

fn scratch(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!(
        "c2rs-sweepreg-{tag}-{}-{:?}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Run the loader into `out`. Returns `(stdout, per-fragment counts)`, or `None`
/// when python3 is absent (the portable lane must degrade, never fail).
fn generate(out: &Path) -> Option<(String, BTreeMap<String, usize>)> {
    let root = repo_root();
    let res = Command::new("python3")
        .arg(root.join("scripts/sweep_gen.py"))
        .arg(out)
        .arg(root.join("scripts/sweep.d"))
        .output();
    let output = match res {
        Ok(o) => o,
        Err(e) => {
            println!("SKIP: cannot run python3 ({e})");
            return None;
        }
    };
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    assert!(
        output.status.success(),
        "scripts/sweep_gen.py failed:\n{stdout}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut counts = BTreeMap::new();
    for line in stdout.lines() {
        let Some(rest) = line.trim().strip_prefix("fragment ") else { continue };
        let mut it = rest.split_whitespace();
        let (Some(stem), Some(n)) = (it.next(), it.next()) else { continue };
        counts.insert(stem.to_string(), n.parse::<usize>().expect("a case count"));
    }
    Some((stdout, counts))
}

#[test]
fn every_sweep_fragment_emits_cases_and_the_counts_match_the_disk() {
    let root = repo_root();
    let frag_dir = root.join("scripts/sweep.d");
    let out = scratch("counts");
    let Some((stdout, counts)) = generate(&out) else { return };

    assert!(!counts.is_empty(), "the loader printed no per-fragment counts");

    // (3) no fragment emits zero cases.
    for (stem, n) in &counts {
        assert!(
            *n > 0,
            "fragment {stem} emitted zero cases — that is the observable symptom \
             of the counter bug (docs/ARCHITECTURE_SEAMS.md §2.4)"
        );
    }

    // (2) printed = generated = on disk, PER FRAGMENT. The loader checks the
    // total; a pair of fragments whose names overlapped could still balance out
    // across it, so the per-fragment split is checked here.
    let mut on_disk: BTreeMap<String, usize> = BTreeMap::new();
    let mut total_files = 0usize;
    for e in std::fs::read_dir(&out).expect("the generated corpus dir").flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        let Some(stem) = name.strip_suffix(".cpp") else { continue };
        total_files += 1;
        let (frag, _idx) = stem.rsplit_once('-').expect("<fragment>-NNNN.cpp");
        *on_disk.entry(frag.to_string()).or_default() += 1;
    }
    assert_eq!(
        counts, on_disk,
        "the loader's printed per-fragment counts are not what is on disk"
    );
    let printed_total: usize = counts.values().sum();
    assert_eq!(
        printed_total, total_files,
        "printed {printed_total} cases, {total_files} .cpp on disk"
    );
    assert!(
        stdout.contains(&format!("{printed_total} cases total")),
        "the loader's own total disagrees with the per-fragment counts:\n{stdout}"
    );

    // (4) every fragment is loadable by the contract, and none is silently skipped.
    let mut files: Vec<String> = std::fs::read_dir(&frag_dir)
        .expect("scripts/sweep.d")
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| !n.starts_with('_'))
        .collect();
    files.sort();
    for name in &files {
        assert!(
            name.ends_with(".py"),
            "scripts/sweep.d/{name} is not a .py fragment — the loader would \
             ignore it, and a fragment nobody runs grades nothing"
        );
        let (num, _) = name.split_once('-').unwrap_or(("", ""));
        assert!(
            !num.is_empty() && num.bytes().all(|b| b.is_ascii_digit()),
            "scripts/sweep.d/{name} is not named NN-<slug>.py — the number is \
             the axis's ordering and the slug is its identity"
        );
        let text = std::fs::read_to_string(frag_dir.join(name)).expect("fragment readable");
        assert!(
            text.contains("def cases(emit)"),
            "scripts/sweep.d/{name} defines no cases(emit) — the fragment contract \
             is in scripts/expr_sweep.sh"
        );
        assert!(
            counts.contains_key(&name[..name.len() - 3]),
            "scripts/sweep.d/{name} produced no count line — it was not loaded"
        );
    }
    assert_eq!(
        files.len(),
        counts.len(),
        "{} fragment files but {} were loaded",
        files.len(),
        counts.len()
    );

    println!(
        "sweep registry: {} fragments, {} cases, printed = generated = on disk",
        counts.len(),
        printed_total
    );
    let _ = std::fs::remove_dir_all(&out);
}

#[test]
fn sweep_generation_is_deterministic() {
    let a = scratch("det-a");
    let b = scratch("det-b");
    let Some((_, counts_a)) = generate(&a) else { return };
    let Some((_, counts_b)) = generate(&b) else { return };
    assert_eq!(counts_a, counts_b, "two runs disagree about the case counts");

    let read_all = |dir: &Path| -> BTreeMap<String, String> {
        std::fs::read_dir(dir)
            .expect("corpus dir")
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().ends_with(".cpp"))
            .map(|e| {
                (
                    e.file_name().to_string_lossy().into_owned(),
                    std::fs::read_to_string(e.path()).expect("case readable"),
                )
            })
            .collect()
    };
    let (ma, mb) = (read_all(&a), read_all(&b));
    assert_eq!(
        ma.len(),
        mb.len(),
        "two runs produced different numbers of cases"
    );
    for (name, src) in &ma {
        assert_eq!(
            mb.get(name),
            Some(src),
            "case {name} differs between two generator runs — the sweep totals \
             quoted in every rung's gate table are only comparable if generation \
             is deterministic"
        );
    }
    let _ = std::fs::remove_dir_all(&a);
    let _ = std::fs::remove_dir_all(&b);
}
