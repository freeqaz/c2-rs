//! **The census/gate invariant** (roadmap #44), as a test — in **both** linkage
//! modes (roadmap #47).
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
//! # Why both modes, and what the second one was hiding
//!
//! `function_gate` takes a `fn_level_linking` flag, because `/Gy` (implied by
//! `/O1` and `/O2`, i.e. by the **entire real workload**) puts each function in
//! its own COMDAT `.text` and two of the port's refusals exist only in that
//! shape. This test used to run the `false` lane only — so it asserted the
//! invariant in the mode the fixtures capture in and said nothing at all about
//! the mode the workload compiles in. The gap scan *does* pass the real flags,
//! so the `/Gy` lane was live and ungated on every scan and merely happened to
//! read 0 there.
//!
//! Both lanes are now pinned at their recorded values with named causes. A gate
//! that lands in codegen instead of the parser fails this test in whichever mode
//! it lands in, which is the whole point.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use c2_core::codegen;
use c2_reference::Toolchain;

/// The ONE disagreement left in the fixture corpus **without** `/Gy`, with its
/// cause.
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
const KNOWN_DISAGREEMENTS_PACKED: usize = 1;

/// The disagreements under **`/Gy`** (function-level linking), which is what the
/// real workload compiles with — `/O1` and `/O2` both imply it.
///
/// It is the `/Ox` residual above **plus the pooled floating-point constants**:
/// `coff::emit_comdat_obj` does not place the `.rdata` COMDAT a pooled FP
/// constant needs, so `function_gate` refuses every W13b body in the `/Gy`
/// shape. That is a refusal, not a mis-emit, and like the one above it costs
/// **0 functions on the 878-TU workload** (no `match`-class TU there carries a
/// pooled constant). Moving *this* gate means teaching the census about a
/// whole-obj layout decision, which is `c2-core`'s seam, not the harness's —
/// recorded here so the number cannot drift, not endorsed as where it belongs.
///
/// **8 → 11 pooled-constant entries (9 → 12 total) when the FP-leaf-beside-framed
/// pair landed**, and the three new ones are *not* a new refusal: they are three
/// more W13b bodies in the fixture corpus, hitting the same standing
/// `emit_comdat_obj` limit as the eight already here. `wunw_float_neg.cpp` gains
/// one and `w28_fp_store_framed_neg.cpp` two — those two fixtures are the
/// negatives that hold the pooled-constant half of the pair, so a pooled constant
/// is precisely what they have to contain. The `causes` table below pins them by
/// name, so trading one of these for a genuinely new refusal still fails even
/// though the total would not move.
const KNOWN_DISAGREEMENTS_GY: usize = 12;

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/cpp")
}

fn work(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("c2rs-census-gate-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Cross-check one linkage mode over the whole fixture corpus.
/// Returns `(in-class functions seen, "<fixture> :: <fn> :: <refusal>" → count)`.
fn cross_check(tc: &Toolchain, gy: bool) -> (usize, BTreeMap<String, usize>) {
    let mut sources: Vec<PathBuf> = std::fs::read_dir(fixtures_dir())
        .expect("fixtures/cpp")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "cpp"))
        .collect();
    sources.sort();
    assert!(!sources.is_empty(), "no fixtures found");

    let w = work(if gy { "gy" } else { "packed" });
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
                // `/Gy`; the mode itself is read per function from `.ex`, and the
                // linkage shape is the argv fact the bundle cannot record — which
                // is exactly why it is a parameter here.
                let refusal = match gate {
                    Err(e) => Some((*e).to_string()),
                    Ok(func) => match codegen::opt_mode_of_word(f.opt_word) {
                        Err(e) => Some(e.to_string()),
                        Ok(mode) => codegen::function_gate(func, mode, gy)
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
    (in_class_total, found)
}

#[test]
fn the_census_and_the_port_agree_about_what_is_in_class() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };

    // Both lanes, at their recorded values. `/Gy` is the mode the whole 878-TU
    // workload compiles in (`/O1` implies it), so leaving it unasserted left the
    // invariant unmeasured exactly where it is load-bearing.
    for (gy, expected, causes) in [
        (
            false,
            KNOWN_DISAGREEMENTS_PACKED,
            &[("no free FP scratch register", 1usize)][..],
        ),
        (
            true,
            KNOWN_DISAGREEMENTS_GY,
            &[
                ("no free FP scratch register", 1),
                ("pooled floating-point constant under function-level linking", 11),
            ][..],
        ),
    ] {
        let (in_class_total, found) = cross_check(&tc, gy);
        let n: usize = found.values().sum();
        assert!(
            in_class_total > 0,
            "the cross-check saw no in-class functions at all (fn_level_linking={gy}) — \
             the instrument, not the port"
        );
        assert_eq!(
            n, expected,
            "census/gate disagreement changed with fn_level_linking={gy} ({in_class_total} \
             functions in class across the fixture corpus). Every entry is a function the \
             census counts and PortC2 refuses, i.e. an error term on the published coverage \
             numerator — move the gate into the IL parser (see docs/GAPS.md §6). Found:\n{}",
            found
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join("\n")
        );
        // A total is not a diagnosis. Pin the *causes* too, so a residual that
        // stays at 9 while one refusal is traded for a different one — a real
        // change, invisible to the count — still fails.
        for (cause, want) in causes {
            let got: usize = found
                .iter()
                .filter(|(k, _)| k.contains(cause))
                .map(|(_, n)| *n)
                .sum();
            assert_eq!(
                got, *want,
                "the `{cause}` refusal moved from {want} to {got} with \
                 fn_level_linking={gy}. Found:\n{}",
                found.keys().map(String::as_str).collect::<Vec<_>>().join("\n")
            );
        }
        let named: usize = causes
            .iter()
            .map(|(cause, _)| {
                found
                    .iter()
                    .filter(|(k, _)| k.contains(cause))
                    .map(|(_, n)| *n)
                    .sum::<usize>()
            })
            .sum();
        assert_eq!(
            named, n,
            "an UNNAMED census/gate refusal appeared with fn_level_linking={gy} — the \
             count still matches but the causes do not. Found:\n{}",
            found.keys().map(String::as_str).collect::<Vec<_>>().join("\n")
        );
    }
}
