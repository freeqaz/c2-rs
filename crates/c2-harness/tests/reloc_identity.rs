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
//! | `?f@@YAXXZ` | *(refused)* | `b ?ext` | **`Refused`** since 2026-08-09 — the inline fence; it read `RelocDiffers` between `w-relo` and `w-inlfence2`, and `Exact` before both |
//! | `?g@@YAXXZ` | `b ?ext` | `b ?ext` | **`Exact`** — the inverse control |
//! | `?anchor@@YAXXZ` | `b ?ext_anchor` | `b ?ext_anchor` | **`Exact`** — the anchor |
//!
//! # 2026-08-09, lane `w-inlfence2` — the known answer became a REPAIR
//!
//! `?f`'s `RelocDiffers` was a **measured wrong emit**: the port claimed a body
//! and the relocation in it named the wrong function.
//! `c2_core::comdat::fenced_inlined_callee` now proves c2 expands the same-TU
//! 4-byte `?g` and refuses the caller instead. `s12` is the canonical reproducer
//! of that family — **858 of the workload's 861 `fnbyte-reloc-differs` bodies
//! relocate against a name their own TU defines** (`work/w-inlfence2/crossing.md`
//! §1), and the fence removes 329 of them.
//!
//! Refused and RelocDiffers score the same **zero** under FBM, so no credit
//! moved. What moved is whether the claim was true.
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
use std::sync::atomic::{AtomicUsize, Ordering};

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

/// Per-CALL scratch counter. See [`work`].
static WORK_SEQ: AtomicUsize = AtomicUsize::new(0);

/// A scratch directory that is private to **one `grade()` call**, not to the
/// process.
///
/// # The defect this closes (lane `w-gateperf`, 2026-08-18)
///
/// This used to be `c2rs-w-relo-{pid}` — one directory for the whole test
/// binary — and the three tests in this file all call [`grade`], so `cargo test`
/// ran three concurrent captures **into one directory**. That is a write-write
/// race on five files, not a slow path:
///
/// * `Toolchain::capture_reference_with` sets `TMP` and `TEMP` to the work dir,
///   so `cl.exe` writes its `_CL_*` IL bundle there — and the bundle's name is
///   hashed from the source path, which is the same `s12.cpp` for all three, so
///   **all three compilers write the same five filenames at the same time**;
/// * that same function opens with *"clear stale bundles so `find_bundle_base`
///   cannot pick up a previous capture in a reused work dir"* and deletes every
///   `_CL_*` in the directory — i.e. one test deletes another test's live
///   bundle;
/// * and all three compile to the same `/Fo out.obj`, which each of them
///   `remove_file`s first.
///
/// Because every thread compiles identical source at identical flags, most
/// interleavings produce identical bytes and pass. The ones that do not read
/// **`Unbound` on all three symbols** — a reference obj paired with someone
/// else's (or a half-written) IL bundle — and the assertion messages then say
/// *"RELOC-EQ indicts correct functions"* and *"the fence stopped firing and the
/// port is emitting a wrong relocation again"*. **A shared temp path presents as
/// a port defect.**
///
/// **MEASURED on this box, one session, same binary:** 0 failures in 18 runs at
/// load ~6-9; **6 failures in 30 runs** with an uncached `expr_sweep.sh` at 48
/// jobs running alongside (load 29-33). Peer `w-c2map2` hit 3 of 3 at load ~15
/// on a docs-only branch whose `crates/` was byte-identical to base, which is
/// the hour-of-misattribution this comment exists to prevent recurring.
///
/// The fix is **this file adopting the convention its eleven siblings already
/// have** — `differential.rs`, `reference.rs`, `listing.rs`,
/// `edit_differential.rs`, `search_differential.rs`, `fixture_profiles.rs`,
/// `il_roundtrip.rs` and `corpus.rs` all key their scratch on
/// `{tag}-{pid}-{counter}`. This file was the only one in the tree whose work
/// directory is shared by more than one test in the same process; that is what
/// makes it a bug rather than a property of the harness.
///
/// Nothing about what is graded moves: same source, same flags, same real
/// `cl.exe`, same `grade_one`, same three assertions.
fn work() -> PathBuf {
    let n = WORK_SEQ.fetch_add(1, Ordering::Relaxed);
    let d = std::env::temp_dir().join(format!("c2rs-w-relo-{}-{n}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// The property above, pinned BY NAME rather than left to the next reader to
/// re-derive from a race that only shows up under load.
///
/// A count-shaped check would not do here: two calls returning one path is
/// exactly what the old code did, and it "passed" every low-load run for as
/// long as it existed.
#[test]
fn two_grade_calls_never_share_a_scratch_directory() {
    let a = work();
    let b = work();
    assert_ne!(
        a, b,
        "two captures in one process must not share a work dir: \
         `capture_reference_with` points TMP/TEMP at it, deletes every `_CL_*` \
         in it, and writes a fixed `out.obj` — so a shared path is a write-write \
         race whose symptom is `Unbound` verdicts that read as a port defect"
    );
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

/// **THE KNOWN ANSWER, AND THE LANE THAT FIXED IT.** `?f` calls `g()`; c2
/// expands `g` and emits `b ?ext` while the port emitted `b ?g`. Both are
/// `48000000`, so the byte compare could not see it and read `Exact` for as long
/// as this instrument existed; `w-relo` widened FBM to grade the relocation and
/// this cell then read **`RelocDiffers(Target)`** — a *measured wrong emit*.
///
/// # 2026-08-09, lane `w-inlfence2` — it reads `Refused` now, and that is the repair
///
/// `?g` is defined in this TU and its lowered body is 4 bytes, so
/// `c2_core::comdat::fenced_inlined_callee` proves c2 expands it and refuses the
/// caller instead of emitting a branch c2 does not emit. **This is the whole
/// point of the fence**: `s12` is the canonical reproducer of the family, and on
/// the 878-TU workload 858 of the 861 `fnbyte-reloc-differs` bodies are the same
/// shape (`work/w-inlfence2/crossing.md` §1).
///
/// A `Refused` and a `RelocDiffers` score the **same zero** under FBM — no
/// credit moves. What moves is the truth of the claim: the port no longer says
/// "here is my body for `?f`" and then gets it wrong. `CLAUDE.md`: outside its
/// class the port returns `NotImplemented`.
///
/// The finding this cell exists to pin — *c2 branches to `?ext`* — is asserted
/// directly against the reference obj's relocation table below, so it survives
/// the port's verdict changing.
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
        FnByte::Refused,
        "`?f@@YAXXZ` must read Refused: the port would branch to `?g` where c2 \
         expands `?g` and branches to `?ext`, and the inline fence stops it. A \
         verdict of RelocDiffers(Target) here means the fence stopped firing and \
         the port is emitting a wrong relocation again; a verdict of Exact means \
         the relocation compare stopped grading"
    );
    assert_eq!(f.0, "tail", "the shape behind the verdict is the CALLER's own");
    // The `RelocKind` import stays live: it is the verdict this cell used to
    // read, and naming it here is what makes the regression direction explicit
    // rather than implied by an absence.
    assert_ne!(
        f.1,
        FnByte::RelocDiffers(RelocKind::Target),
        "the pre-fence verdict must NOT come back"
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
/// three emitted functions, **one refused by the inline fence**, two exact.
/// Trap 0 — the control above is a statement about exactly this population.
///
/// The row has read three different things and each was true at the time:
/// `exact 3 · differs 0` before `w-relo` (the blind byte compare),
/// `bytes-exact 3 · exact 2 · reloc-differs 1` after it (the wrong emit, seen),
/// and `bytes-exact 2 · exact 2 · refused 1` after `w-inlfence2` (the wrong emit,
/// **removed**). The whole sequence is kept in the assertion message because a
/// count with no history cannot say which of those three a regression is.
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
    let refused = rows.iter().filter(|r| r.1 == FnByte::Refused).count();
    println!(
        "s12: {} emitted · bytes-exact {bytes} · exact {exact} · \
         reloc-differs {reloc} · refused {refused}",
        rows.len()
    );
    assert_eq!(
        (bytes, exact, reloc, refused),
        (2, 2, 0, 1),
        "s12 has read three things and each was true when it was written: \
         `exact 3 · differs 0` before `w-relo` (the blind byte compare), \
         `bytes-exact 3 · exact 2 · reloc-differs 1` after it (the wrong emit, \
         SEEN), and `bytes-exact 2 · exact 2 · reloc-differs 0 · refused 1` \
         after `w-inlfence2` (the wrong emit, REMOVED — `?f` no longer has bytes \
         at all, which is why `bytes-exact` fell to 2). Got: {rows:?}"
    );
}
