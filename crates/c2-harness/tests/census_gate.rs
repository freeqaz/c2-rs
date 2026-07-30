//! **The census/gate invariant** (roadmap #44), as a test.
//!
//! Acceptance is supposed to live in the IL parser precisely so that
//! `IlBundle::function_census` — the public coverage numerator — and `PortC2`
//! cannot disagree about what is in class. They did: `int f(int a,int b,int c)
//! { return a + b*c; }` censused in class and the port returned
//! `NotImplemented`, because a `*` after the first operator was gated in codegen
//! where the census could not see it (`docs/IL_CALL_IN_EXPR.md` §24.7). On the
//! 878-TU workload that over-claim measured **9,230 functions, 2.24 % of the
//! numerator**, and none of it was the §24.7 shape.
//!
//! The gates have been moved, and this test is what keeps them moved: it runs
//! `PortC2`'s own per-function selector over **every function the census calls
//! in class**, across the whole fixture corpus, and requires the disagreement to
//! stay at its recorded value. `docs/GAPS.md` §6 states the general form — a
//! diagnostic that runs outside the parser needs a population whose answer is
//! already known, and this is that population: every in-class function, whose
//! answer must be "accepted".
//!
//! Toolchain-gated: skips cleanly (never fails) when `Toolchain::locate()` is
//! `None`, per the CLAUDE.md hard constraint.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use c2_core::codegen;
use c2_reference::Toolchain;

/// The ONE disagreement left in the fixture corpus, with its cause.
///
/// `w13_fscratch.cpp`'s `fm13` — thirteen `float` parameters and twelve
/// multiplies — is refused by `float_leaf_text`'s FP scratch allocator, which
/// never retires a parameter from its live set, so thirteen parameters leave
/// exactly one free pool slot and the second temporary has nowhere to go. It is
/// a refusal, not a mis-emit, and it costs **0 functions on the 878-TU
/// workload**; moving it would mean lifting the whole FP register allocator into
/// the IL crate, which is a byte-visible refactor and is written up as a handoff
/// rather than done here.
///
/// The number is asserted rather than allow-listed away so that it cannot grow
/// quietly: a new gate landing in codegen instead of the parser fails this test.
const KNOWN_DISAGREEMENTS: usize = 1;

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/cpp")
}

fn work(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("c2rs-census-gate-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn the_census_and_the_port_agree_about_what_is_in_class() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    let mut sources: Vec<PathBuf> = std::fs::read_dir(fixtures_dir())
        .expect("fixtures/cpp")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "cpp"))
        .collect();
    sources.sort();
    assert!(!sources.is_empty(), "no fixtures found");

    let w = work("all");
    let mut found: BTreeMap<String, usize> = BTreeMap::new();
    let mut in_class_total = 0usize;
    for (i, cpp) in sources.iter().enumerate() {
        let dir = w.join(format!("f{i:04}"));
        let _ = std::fs::create_dir_all(&dir);
        let Ok(bundle) = tc.capture_il(cpp, &dir) else {
            // A fixture the front end declines is not this test's business.
            let _ = std::fs::remove_dir_all(&dir);
            continue;
        };
        if let Some(rows) = bundle.census_functions() {
            for (f, gate) in &rows {
                if !f.verdict.in_class() {
                    continue;
                }
                in_class_total += 1;
                // Fixtures capture at the default `/Ox`, which does not imply
                // `/Gy`; the mode itself is read per function from `.ex`.
                let refusal = match gate {
                    Err(e) => Some((*e).to_string()),
                    Ok(func) => match codegen::opt_mode_of_word(f.opt_word) {
                        Err(e) => Some(e.to_string()),
                        Ok(mode) => codegen::function_gate(func, mode, false)
                            .err()
                            .map(|e| e.to_string()),
                    },
                };
                if let Some(r) = refusal {
                    let name = f.name.clone().unwrap_or_else(|| format!("#{}", f.index));
                    *found
                        .entry(format!(
                            "{} :: {name} :: {r}",
                            cpp.file_name().unwrap().to_string_lossy()
                        ))
                        .or_insert(0) += 1;
                }
            }
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
    let _ = std::fs::remove_dir_all(&w);

    let n: usize = found.values().sum();
    assert!(
        in_class_total > 0,
        "the cross-check saw no in-class functions at all — the instrument, not the port"
    );
    assert_eq!(
        n, KNOWN_DISAGREEMENTS,
        "census/gate disagreement changed ({in_class_total} functions in class across the \
         fixture corpus). Every entry is a function the census counts and PortC2 refuses, \
         i.e. an error term on the published coverage numerator — move the gate into the \
         IL parser (see docs/GAPS.md §6). Found:\n{}",
        found
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join("\n")
    );
}
