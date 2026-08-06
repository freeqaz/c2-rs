//! **RELOC-EQ, against real `c2`** — the known-answer test and its inverse
//! control, on one compiled obj (lane `w-relo`, board **#884**).
//!
//! `docs/FUNCTION_BYTE_MATCH.md` §7.6 stated the gap as a constructed
//! counterexample and then measured it: **4,664** credited functions carried a
//! relocation FBM did not check. Lane `w-seq` compiled the reproducer — GRID-S
//! cell `s12` — and recorded the finding without being able to grade it:
//!
//! > `s12` is a CONTROL THAT PASSED AND CAUGHT SOMETHING ELSE. FBM reads
//! > `exact 3 · differs 0` on it, and the port is **wrong about `?f`**: c2 emits
//! > `b ?ext` and the port emits `b ?g`, and both are the word `48000000`.
//!
//! This file is the grading. The cell is compiled with the **real toolchain** at
//! the workload's own profile and run through the *same* `grade_one` the 878-TU
//! scan runs — never a copy — with the reference obj's own relocation records
//! handed to it.
//!
//! # Why the positive and the negative are in ONE obj
//!
//! | symbol | port emits | c2 emits | must read |
//! |---|---|---|---|
//! | `?f@@YAXXZ` | `b ?g` | `b ?ext` | **`RelocDiffers`** — the known answer |
//! | `?g@@YAXXZ` | `b ?ext` | `b ?ext` | **`Exact`** — the inverse control |
//! | `?anchor@@YAXXZ` | `b ?ext_anchor` | `b ?ext_anchor` | **`Exact`** — the anchor |
//!
//! All three bodies are the single word `48000000`. A rule that turns every
//! relocated function red would pass the first row and fail the other two, and a
//! rule that grades nothing would pass the last two and fail the first — so the
//! pair is what makes either verdict mean anything. `docs/STATUS.md` trap 0: a
//! green control is a statement about the population it ran over, and this
//! population is stated in the table above rather than implied.
//!
//! `SKIP: toolchain absent` when there is no toolchain, like every other
//! integration test here.

use std::collections::BTreeMap;
use std::path::PathBuf;

use c2_harness::gap::fnbytes::{grade_one, tu_empty_callees, FnByte, RelocKind};
use c2_reference::Toolchain;

/// The workload's own profile, minus the `/I` paths a standalone cell cannot
/// use. `/O1` implies `/Gy`, which is the regime FBM's denominator lives in.
const FLAGS: [&str; 8] = [
    "/nologo", "/wd4355", "/wd4164", "/c", "/GR", "/O1", "/Oi", "/EHsc",
];

/// **GRID-S cell `s12`, verbatim** — `work/w-seq/cells/s12_callee_calls_extern.cpp`.
/// Kept as a literal rather than read from `work/`, which is gitignored: a test
/// that silently skips because its input is not checked in is a test that
/// reports absence as success.
const S12: &str = "\
void ext();
void g() { ext(); }
void f() { g(); }

void ext_anchor();
void anchor() { ext_anchor(); }
";

fn work() -> PathBuf {
    let d = std::env::temp_dir().join(format!("c2rs-w-relo-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Grade every emitted `.text` COMDAT of one cell on the FULL identity — bytes
/// **and** relocations — through the scan's own route.
fn grade(tc: &Toolchain) -> Vec<(&'static str, FnByte, String)> {
    let dir = work();
    let cpp = dir.join("s12.cpp");
    std::fs::write(&cpp, S12).unwrap();
    let flags: Vec<String> = FLAGS.iter().map(|s| s.to_string()).collect();
    let src = c2_reference::to_wibo_path(&cpp);
    let Ok(cap) = tc.capture_reference_with(&src, &dir, &flags, None) else {
        return Vec::new();
    };
    let (Some(census), Some(entries)) = (
        cap.bundle.census_functions(),
        cap.ref_obj.text_comdat_functions_with_bytes(),
    ) else {
        return Vec::new();
    };
    let mut claim: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (i, (f, _)) in census.iter().enumerate() {
        if let Some(n) = f.emit_name.as_deref() {
            claim.entry(n).or_default().push(i);
        }
    }
    let tu = tu_empty_callees(&census);
    // Positionally paired with the COMDAT walk — both are walks over the same
    // `text_comdat_entries` list. A `None` here would mean the reference obj's
    // relocation table did not decode, and every verdict below would then read
    // `RelocUnknown` rather than a passing `Exact`.
    let rel = cap.ref_obj.text_comdat_relocs();
    assert!(
        rel.is_some(),
        "the reference obj's .text relocation table did not decode — no verdict \
         in this cell means anything, and reading that as a pass is exactly the \
         defect this lane closed"
    );
    let mut out = Vec::new();
    for (idx, (sym, bytes)) in entries.iter().enumerate() {
        let row = match claim.get(sym.as_str()).map(Vec::as_slice) {
            Some([i]) => Some(&census[*i]),
            _ => None,
        };
        let rr = rel.as_ref().and_then(|v| v.get(idx)).map(|(_, r)| r.as_slice());
        let g = grade_one(row, Some(bytes.as_slice()), &tu, rr);
        out.push((g.shape, g.verdict, sym.clone()));
    }
    out
}

fn find<'a>(
    rows: &'a [(&'static str, FnByte, String)],
    sym: &str,
) -> &'a (&'static str, FnByte, String) {
    rows.iter().find(|r| r.2 == sym).unwrap_or_else(|| {
        panic!(
            "no `{sym}` COMDAT in the reference obj — the capture produced {} \
             functions and none of them is the one this test grades: {:?}",
            rows.len(),
            rows.iter().map(|r| &r.2).collect::<Vec<_>>()
        )
    })
}

/// **THE KNOWN ANSWER.** `?f` calls `g()`; c2 expands `g` and emits `b ?ext`
/// while the port emits `b ?g`. Both are `48000000`, so the byte compare cannot
/// see it and read `Exact` for as long as this instrument existed. It must now
/// read `RelocDiffers(Target)`.
#[test]
fn the_s12_reproducer_moves_from_exact_to_a_relocation_disagreement() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let rows = grade(&tc);
    if rows.is_empty() {
        println!("SKIP: capture produced no graded function");
        return;
    }
    let f = find(&rows, "?f@@YAXXZ");
    assert_eq!(
        f.1,
        FnByte::RelocDiffers(RelocKind::Target),
        "`?f@@YAXXZ` must read RelocDiffers(Target): the port branches to `?g` \
         and c2 branches to `?ext`, and the two instruction words are equal. A \
         verdict of Exact here means the widening did not widen; a verdict of \
         Differs means it merged two different repairs into one bucket"
    );
    assert_eq!(f.0, "tail", "the shape behind the verdict");
    // The old predicate still holds — this is a BYTE-exact body, which is what
    // makes the finding a relocation finding and not a codegen one.
    assert!(
        f.1.bytes_exact(),
        "`?f`'s bytes ARE c2's; if they were not, this cell would be testing the \
         byte compare and not the relocation compare"
    );
}

/// **THE INVERSE CONTROL, on the same obj.** `?g` branches to the external it
/// really calls and so does c2; `?anchor` likewise. Both must stay `Exact` —
/// bytes *and* relocations.
///
/// Without this, "every relocated function is now red" would pass the test
/// above, and a widening that indicts everything measures nothing.
#[test]
fn a_function_whose_bytes_and_relocations_are_both_c2s_stays_exact() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let rows = grade(&tc);
    if rows.is_empty() {
        println!("SKIP: capture produced no graded function");
        return;
    }
    for sym in ["?g@@YAXXZ", "?anchor@@YAXXZ"] {
        let r = find(&rows, sym);
        assert_eq!(
            r.1,
            FnByte::Exact,
            "`{sym}` is byte-exact AND relocation-exact against c2's own obj — \
             it emits one REL24 against the external it actually calls. A red \
             verdict here means RELOC-EQ indicts correct functions, and a rule \
             that indicts everything grades nothing"
        );
    }
}

/// The cell's population, stated as a count rather than left to be inferred:
/// three emitted functions, one relocation disagreement, two exact. Trap 0 —
/// the control above is a statement about exactly this population.
#[test]
fn the_cells_population_is_three_functions_one_of_which_disagrees() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let rows = grade(&tc);
    if rows.is_empty() {
        println!("SKIP: capture produced no graded function");
        return;
    }
    let exact = rows.iter().filter(|r| r.1 == FnByte::Exact).count();
    let reloc = rows
        .iter()
        .filter(|r| matches!(r.1, FnByte::RelocDiffers(_)))
        .count();
    let bytes = rows.iter().filter(|r| r.1.bytes_exact()).count();
    println!(
        "s12: {} emitted · bytes-exact {bytes} · exact {exact} · reloc-differs {reloc}",
        rows.len()
    );
    assert_eq!(
        (bytes, exact, reloc),
        (3, 2, 1),
        "s12 read `exact 3 · differs 0` before this lane and must read \
         `bytes-exact 3 · exact 2 · reloc-differs 1` after it: {rows:?}"
    );
}
