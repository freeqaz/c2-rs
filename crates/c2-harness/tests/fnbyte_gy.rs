//! **The four `/Gy` shapes are RECONSTRUCTED AND GRADED, and every word of the
//! reconstruction is load-bearing** — board #322, lane `w-fnbyte`.
//!
//! # What this test is for
//!
//! FUNCTION BYTE MATCH grades the port's per-function output against real c2's
//! own `.text` COMDAT bytes, and `fnbyte-differs 0` is the project's standing
//! per-function alarm. Until board #322 it was **structurally blind** to
//! `Selected::{Tail, Framed, Seq, CondPair}`: the harness declined to
//! reconstruct their bodies, so a wrong emit in any of them read as `differs 0`
//! over **9,375 workload functions**.
//!
//! Closing that is only worth something if two things are true, and only a test
//! against the real toolchain can say so:
//!
//! 1. **The four shapes really are graded**, per shape, with a count. An
//!    instrument that quietly stopped reconstructing one of them would go back
//!    to reading 0 differs there and nothing would say so — absence reading as
//!    success, `docs/STATUS.md` trap 5. So this asserts a positive count per
//!    shape and names the shape that is missing.
//! 2. **Every reconstructed word is actually compared.** A reconstruction that
//!    produced the right words and then compared only the first half would look
//!    identical on a clean corpus. The mutation below flips **each word of the
//!    reference body in turn** and requires the verdict to go from `Exact` to
//!    `Differs` on every single one: `flips == words`, with both numbers
//!    printed. A word the comparison does not read stays `Exact` and fails the
//!    test by count, not by status (trap 5's mitigation, stated generally).
//!
//! # The bytes are real c2's
//!
//! The reference is `Toolchain::capture_reference_with` at **`/O1 /Oi /EHsc
//! /GR /c`** — the dc3 workload's own profile, and the one that implies `/Gy`,
//! which is what puts each function in its own COMDAT and makes its `.text`
//! offset the constant 0 this whole reconstruction rests on. Fixtures capture at
//! `/Ox` by default and `/Ox` does **not** imply `/Gy`, so a test that used the
//! default profile would grade a packed obj with no `.text` COMDAT at all and
//! pass vacuously.
//!
//! Degrades to a printed `SKIP: toolchain absent` rather than failing, per
//! `CLAUDE.md`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use c2_harness::gap::fnbytes::{grade_one, tu_empty_callees, FnByte};
use c2_reference::Toolchain;

/// One fixture per shape, chosen off a `/Gy` fixture scan (lane `w-fnbyte`,
/// `work/w-fnbyte/fixture_gy.jsonl`) — the smallest fixture that produces each.
///
/// All four are required. A shape that stops appearing here fails the test with
/// its own name, which is the difference between this and a corpus total.
const SHAPE_FIXTURES: [(&str, &str); 4] = [
    ("tail", "mvp_call.cpp"),
    ("framed", "mvp_framed.cpp"),
    ("seq", "mvp_call_twice.cpp"),
    ("cond-pair", "w8_cond_tail.cpp"),
];

/// The workload's own profile. `/O1` implies `/Gy`; `/Ox` does not, and the
/// fixtures' default profile is `/Ox`.
const GY_FLAGS: [&str; 7] = [
    "/nologo", "/wd4355", "/wd4164", "/c", "/GR", "/O1", "/Oi",
];

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/cpp")
}

fn work(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("c2rs-fnbyte-gy-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Grade every emitted function of one fixture at the `/Gy` profile.
/// Returns `(shape, verdict, symbol, reference bytes)` per emitted COMDAT.
fn grade_fixture(tc: &Toolchain, cpp: &Path, dir: &Path) -> Vec<(&'static str, FnByte, String, Vec<u8>)> {
    let mut flags: Vec<String> = GY_FLAGS.iter().map(|s| s.to_string()).collect();
    flags.push("/EHsc".to_string());
    let src = c2_reference::to_wibo_path(cpp);
    let Ok(cap) = tc.capture_reference_with(&src, dir, &flags, None) else {
        return Vec::new();
    };
    let Some(census) = cap.bundle.census_functions() else {
        return Vec::new();
    };
    let Some(entries) = cap.ref_obj.text_comdat_functions_with_bytes() else {
        return Vec::new();
    };
    // Same unique-binding rule the scan uses: two rows claiming one symbol is
    // `Unbound`, never a coin flip between them.
    let mut claim: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (i, (f, _)) in census.iter().enumerate() {
        if let Some(n) = f.emit_name.as_deref() {
            claim.entry(n).or_default().push(i);
        }
    }
    // The reference obj's own relocation records, positionally paired with the
    // COMDAT walk above (both are walks over `text_comdat_entries`). Passed in
    // so these cells are graded on the FULL identity — bytes and relocations —
    // exactly as the 878-TU scan grades them.
    let rel = cap.ref_obj.text_comdat_relocs();
    let mut out = Vec::new();
    for (idx, (name, bytes)) in entries.iter().enumerate() {
        let row = match claim.get(name.as_str()).map(Vec::as_slice) {
            Some([i]) => Some(&census[*i]),
            _ => None,
        };
        let rr = rel.as_ref().and_then(|v| v.get(idx)).map(|(_, r)| r.as_slice());
        let g = grade_one(row, Some(bytes.as_slice()), &tu_empty_callees(&census), rr);
        out.push((g.shape, g.verdict, name.clone(), bytes.clone()));
    }
    out
}

#[test]
fn the_four_gy_shapes_are_graded_and_every_reconstructed_word_is_compared() {
    let Some(tc) = Toolchain::locate() else {
        println!("SKIP: toolchain absent");
        return;
    };
    let w = work("shapes");
    // --- 1. every shape is graded, per shape, with a count -------------------
    let mut graded_per_shape: BTreeMap<&str, usize> = BTreeMap::new();
    // One `Exact` specimen per shape, for the mutation below.
    let mut specimen: BTreeMap<&str, (String, Vec<u8>, PathBuf)> = BTreeMap::new();
    for (shape, fixture) in SHAPE_FIXTURES {
        let cpp = fixtures_dir().join(fixture);
        let dir = w.join(fixture);
        let _ = std::fs::create_dir_all(&dir);
        let rows = grade_fixture(&tc, &cpp, &dir);
        assert!(
            !rows.is_empty(),
            "{fixture}: no emitted function was graded at all — the /Gy capture \
             produced no `.text` COMDAT, so this test would have passed vacuously"
        );
        for (s, v, name, bytes) in rows {
            if s != shape {
                continue;
            }
            // The point of the whole lane: this shape must NOT be `Partial`.
            assert!(
                !matches!(v, FnByte::Partial(_)),
                "{fixture} :: {name}: shape `{shape}` came back Partial — the \
                 reconstruction declined it, which is the blind spot board #322 \
                 closed"
            );
            *graded_per_shape.entry(shape).or_insert(0) += 1;
            if v == FnByte::Exact {
                specimen
                    .entry(shape)
                    .or_insert((name.clone(), bytes.clone(), cpp.clone()));
            }
        }
    }
    for (shape, fixture) in SHAPE_FIXTURES {
        let n = graded_per_shape.get(shape).copied().unwrap_or(0);
        println!("graded {shape}: {n} function(s) from {fixture}");
        assert!(
            n > 0,
            "shape `{shape}` was graded ZERO times (fixture {fixture}). A count of \
             zero is what a silently-dropped reconstruction looks like; it is not \
             a pass."
        );
    }

    // --- 2. THE MUTATIONS — one per shape, every word ------------------------
    //
    // For each shape's `Exact` specimen, flip one bit of each reference word in
    // turn and require `Differs` every time. A word the comparison never reads
    // survives the flip as `Exact`, and the assertion below is on the COUNT of
    // flips that went red, not on any status line.
    let mut total_flips = 0usize;
    for (shape, _) in SHAPE_FIXTURES {
        let Some((name, bytes, cpp)) = specimen.get(shape) else {
            panic!(
                "shape `{shape}` produced no byte-exact specimen to mutate — the \
                 mutation harness graded nothing for it, which is a vacuous pass"
            );
        };
        let dir = w.join(format!("mut-{shape}"));
        let _ = std::fs::create_dir_all(&dir);
        // Re-grade to recover the census row through the same route.
        let rows = grade_fixture(&tc, cpp, &dir);
        let (_, v0, _, _) = rows
            .iter()
            .find(|(_, _, n, _)| n == name)
            .expect("the specimen is still emitted");
        assert_eq!(
            *v0,
            FnByte::Exact,
            "{shape} :: {name}: the UNMUTATED control must be Exact, or the \
             mutations below prove nothing"
        );
        let words = bytes.len() / 4;
        assert!(words > 0, "{shape} :: {name}: a zero-word body cannot be mutated");
        let mut flips = 0usize;
        for i in 0..words {
            let mut m = bytes.clone();
            m[i * 4 + 3] ^= 0x01;
            // Re-run the whole grading against the mutated reference. Only the
            // reference changes; the port's body is the emitter's own output.
            let g = regrade_against(&tc, cpp, &dir, name, &m);
            if matches!(g, Some(FnByte::Differs { .. })) {
                flips += 1;
            } else {
                panic!(
                    "{shape} :: {name}: flipping word {i} of {words} left the verdict \
                     at {g:?} — that word is NOT compared, so the reconstruction is \
                     graded only in part"
                );
            }
        }
        println!("mutation {shape}: {flips} of {words} words flipped the verdict red");
        assert_eq!(
            flips, words,
            "{shape} :: {name}: {flips} of {words} reconstructed words are \
             load-bearing; every one must be"
        );
        total_flips += flips;
    }
    println!("mutations total: {total_flips} words, all RED");
    assert!(
        total_flips >= 4,
        "fewer than one mutation per shape ran ({total_flips}) — a mutation \
         harness that runs nothing passes everything"
    );
    let _ = std::fs::remove_dir_all(&w);
}

/// Re-grade one named symbol with a **substituted** reference body.
fn regrade_against(
    tc: &Toolchain,
    cpp: &Path,
    dir: &Path,
    name: &str,
    bytes: &[u8],
) -> Option<FnByte> {
    let mut flags: Vec<String> = GY_FLAGS.iter().map(|s| s.to_string()).collect();
    flags.push("/EHsc".to_string());
    let src = c2_reference::to_wibo_path(cpp);
    let cap = tc.capture_reference_with(&src, dir, &flags, None).ok()?;
    let census = cap.bundle.census_functions()?;
    let mut claim: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (i, (f, _)) in census.iter().enumerate() {
        if let Some(n) = f.emit_name.as_deref() {
            claim.entry(n).or_default().push(i);
        }
    }
    let row = match claim.get(name).map(Vec::as_slice) {
        Some([i]) => Some(&census[*i]),
        _ => None,
    };
    // Only the BYTES are substituted; the relocation records stay the reference
    // obj's own, so a mutation that fails to turn the verdict red cannot be
    // rescued by handing the compare an empty relocation set.
    let rel = cap.ref_obj.text_comdat_relocs();
    let rr = rel
        .as_ref()
        .and_then(|v| v.iter().find(|(n, _)| n == name))
        .map(|(_, r)| r.as_slice());
    Some(grade_one(row, Some(bytes), &tu_empty_callees(&census), rr).verdict)
}
