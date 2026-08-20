//! **IR0's three-way differential — the spine of lane `ir0`.**
//!
//! For every captured fixture bundle, three separately-written implementations
//! of each `.ex` segmentation must agree on **offsets and byte ranges**, not
//! merely on counts:
//!
//! 1. the IR0 view (`c2_il::Ir0::frame_ex(..).gate_segments()` /
//!    `.body_segments()`),
//! 2. the incumbent (`c2_il::ex_segments_gate()` /
//!    `c2_il::ex_segments_body()`, the thin public wrappers over
//!    `func::bundle::split_functions_at` / `split_function_bodies_at`),
//! 3. an **independently hand-written reference in this file**, transcribed
//!    from the incumbents' DOC BLOCKS rather than from their code.
//!
//! # Why the third one exists, and why it must stay
//!
//! Board **#3288**: derive every published count a second, differently-built
//! way and diff them. Two implementations would be a tautology the moment the
//! incumbent is folded into the view — at that point (1) and (2) are the same
//! function called twice, and a test comparing them proves nothing at all. The
//! reference below is the only side of the comparison that stays independent,
//! so **it must never be rewritten by reading `stream/ex.rs`.** It is a
//! transcription of the prose at `func/bundle.rs:468–496` and `:605–609`:
//!
//! * gate: *"Split the `.ex` stream at every `4F 1F` function-start marker"*,
//!   left to right, a match consuming the marker.
//! * body: *"anchored on the `LO` marker"* (`4C 4F 11`); *"Each segment starts
//!   at the `4F 1F` immediately preceding its `LO` … and runs to the next
//!   segment's start"*; *"Two bodies sharing one preceding `4F 1F` would
//!   collide; the later one then starts at its own `LO`"*; plus the
//!   *"second, strictly-additive pass over the `4F 1F` regions that hold no
//!   `4C 4F 11`"* for the grammar-gated bare `4C`.
//!
//! # Environment, not exit code (#3219/#3231)
//!
//! A fresh `git worktree add` has **no `compilers/`** — it is gitignored and
//! does not follow a worktree — so every capture-based test skips, and cargo
//! swallows the SKIP line for a passing test. A registered RED then reads GREEN
//! with a clean suite and the right exit code. So this file **asserts a
//! non-zero checked count** and prints it, and the skip path is the only way
//! out.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use c2_harness::all_fixtures;
use c2_il::{Ir0, Ir0Framing, RecordKind};
use c2_reference::Toolchain;

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn work(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let d = std::env::temp_dir().join(format!(
        "c2rs-ir0-{tag}-{}-{}-{}",
        std::process::id(),
        nanos,
        n
    ));
    std::fs::create_dir_all(&d).unwrap();
    d
}

// ---------------------------------------------------------------------------
// THE INDEPENDENT REFERENCE. Transcribed from the doc blocks; never from the
// code it grades. Written in a deliberately different style — plain index
// loops, no `memchr`, no `partition_point`, no shared helper with either of
// the other two implementations — so that a shared bug has to be invented
// twice to survive.
// ---------------------------------------------------------------------------

/// *"Split the `.ex` stream at every `4F 1F` function-start marker."*
fn ref_gate_offsets(ex: &[u8]) -> Vec<usize> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while i + 1 < ex.len() {
        if ex[i] == 0x4F && ex[i + 1] == 0x1F {
            out.push(i);
            i += 2;
        } else {
            i += 1;
        }
    }
    out
}

/// Offsets → segments, each running to the next offset or to end of file.
fn spans<'a>(ex: &'a [u8], offs: &[usize]) -> Vec<&'a [u8]> {
    let mut out = Vec::with_capacity(offs.len());
    for k in 0..offs.len() {
        let start = offs[k];
        let end = if k + 1 < offs.len() {
            offs[k + 1]
        } else {
            ex.len()
        };
        out.push(&ex[start..if end > start { end } else { start }]);
    }
    out
}

/// Every `4C 4F 11` body marker, left to right, a match consuming three bytes.
fn ref_lo_offsets(ex: &[u8]) -> Vec<usize> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while i + 2 < ex.len() {
        if ex[i] == 0x4C && ex[i + 1] == 0x4F && ex[i + 2] == 0x11 {
            out.push(i);
            i += 3;
        } else {
            i += 1;
        }
    }
    out
}

/// *"the grammar-gated bare `4C` where it does not"* — the statement /
/// result-ref / formals prefix walk, transcribed from `bare_lo_after_prefix`'s
/// doc: from a `53 53`, consume `53` (1 byte) and `26` (3 bytes) until `46`,
/// then the formals `2D` triples, and the bare `4C` must be followed by `53`.
fn ref_bare_body_start(seg: &[u8]) -> Option<usize> {
    let mut anchor = 0usize;
    while anchor + 1 < seg.len() {
        if seg[anchor] == 0x53 && seg[anchor + 1] == 0x53 {
            let mut p = anchor + 2;
            let ok = loop {
                match seg.get(p) {
                    None => break false,
                    Some(0x53) => p += 1,
                    Some(0x26) => p += 3,
                    Some(0x46) => break true,
                    Some(_) => break false,
                }
            };
            if ok {
                p += 1;
                while seg.get(p) == Some(&0x2D) {
                    p += 3;
                }
                if seg.get(p) == Some(&0x4C) && seg.get(p + 1) == Some(&0x53) {
                    return Some(p);
                }
            }
        }
        anchor += 1;
    }
    None
}

/// The census segmentation, from the doc block at `func/bundle.rs:468–496`.
fn ref_body_offsets(ex: &[u8]) -> Vec<usize> {
    let starts = ref_gate_offsets(ex);
    let mut los = ref_lo_offsets(ex);

    // *"a second, strictly-additive pass over the `4F 1F` regions that hold no
    // `4C 4F 11`"*. Linear containment test rather than a binary search, on
    // purpose: a different way of asking the same question.
    let mut extra = Vec::new();
    for (k, &s) in starts.iter().enumerate() {
        let e = if k + 1 < starts.len() {
            starts[k + 1]
        } else {
            ex.len()
        };
        if los.iter().any(|&l| l >= s && l < e) {
            continue;
        }
        if let Some(p) = ref_bare_body_start(&ex[s..e]) {
            extra.push(s + p);
        }
    }
    los.extend(extra);
    los.sort_unstable();

    // *"Each segment starts at the `4F 1F` immediately preceding its `LO`"*,
    // and *"Two bodies sharing one preceding `4F 1F` … the later one then
    // starts at its own `LO`"*. Linear scan for the preceding start.
    let mut out: Vec<usize> = Vec::with_capacity(los.len());
    for &lo in &los {
        let mut cand = lo;
        for &s in &starts {
            if s < lo {
                cand = s;
            } else {
                break;
            }
        }
        if out.last() == Some(&cand) {
            cand = lo;
        }
        out.push(cand);
    }
    out
}

// ---------------------------------------------------------------------------

/// Compare all three, on offsets AND byte ranges, for one `.ex` stream.
/// Returns `(gate segments, body segments)` so the caller can assert the two
/// views stay distinct on at least one input.
fn three_way(name: &str, ex: &[u8]) -> (usize, usize) {
    let f = Ir0::frame_ex(ex);

    // I1 ∧ I2 — the framing is total and re-serializes byte-identically.
    f.verify()
        .unwrap_or_else(|e| panic!("{name}: IR0 framing not total: {e:?}"));

    // And the framing accounts for every byte, split into framed and opaque.
    let (framed, opaque) = f.byte_split();
    assert_eq!(
        framed + opaque,
        ex.len(),
        "{name}: IR0 byte accounting broken ({framed} framed + {opaque} opaque != {} bytes)",
        ex.len()
    );

    // --- gate, three ways
    let (v_off, v_seg) = f.gate_segments();
    let (i_off, i_seg) = c2_il::ex_segments_gate(ex);
    let r_off = ref_gate_offsets(ex);
    let r_seg = spans(ex, &r_off);
    assert_eq!(v_off, i_off, "{name}: gate offsets, IR0 view vs incumbent");
    assert_eq!(v_off, r_off, "{name}: gate offsets, IR0 view vs REFERENCE");
    assert_eq!(v_seg, i_seg, "{name}: gate segments, IR0 view vs incumbent");
    assert_eq!(v_seg, r_seg, "{name}: gate segments, IR0 view vs REFERENCE");

    // --- body, three ways
    let (bv_off, bv_seg) = f.body_segments();
    let (bi_off, bi_seg) = c2_il::ex_segments_body(ex);
    let br_off = ref_body_offsets(ex);
    let br_seg = spans(ex, &br_off);
    assert_eq!(bv_off, bi_off, "{name}: body offsets, IR0 view vs incumbent");
    assert_eq!(bv_off, br_off, "{name}: body offsets, IR0 view vs REFERENCE");
    assert_eq!(bv_seg, bi_seg, "{name}: body segments, IR0 view vs incumbent");
    assert_eq!(bv_seg, br_seg, "{name}: body segments, IR0 view vs REFERENCE");

    (v_off.len(), bv_off.len())
}

#[test]
fn ir0_three_way_differential_over_all_fixtures() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    let fixtures = all_fixtures();
    assert!(!fixtures.is_empty(), "no fixtures found");

    let mut checked = 0usize;
    let mut ex_bytes = 0usize;
    let mut ex_opaque = 0usize;
    let mut records = 0usize;
    let mut bundle_bytes = 0usize;
    let mut bundle_framed = 0usize;
    // **C1's property, at fixture scale**: the two views must remain two
    // segmentations. Counted, not asserted per fixture — most fixtures are
    // one-function TUs where they legitimately agree.
    let mut differ = 0usize;

    for cpp in &fixtures {
        let name = cpp
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let w = work("cap");
        let bundle = match tc.capture_il(cpp, &w) {
            Ok(b) => b,
            Err(e) => {
                std::fs::remove_dir_all(&w).ok();
                panic!("capture_il failed for {name}: {e}");
            }
        };

        // Whole-bundle framing: totality across every present file.
        let ir0 = Ir0::frame(&bundle);
        ir0.verify()
            .unwrap_or_else(|e| panic!("{name}: whole-bundle IR0 framing not total: {e:?}"));
        for ff in &ir0.files {
            records += ff.records.len();
            bundle_bytes += ff.bytes.len();
            bundle_framed += ff
                .records
                .iter()
                .filter(|r| r.kind != RecordKind::Opaque)
                .map(|r| r.extent.len())
                .sum::<usize>();
        }

        if let Some(ex) = bundle.ex() {
            let (g, b) = three_way(&name, ex);
            if g != b {
                differ += 1;
            }
            let f = Ir0::frame_ex(ex);
            let (_, op) = f.byte_split();
            ex_bytes += ex.len();
            ex_opaque += op;
        }
        checked += 1;
        std::fs::remove_dir_all(&w).ok();
    }

    // **The environment assertion, not the exit code.** A count of zero here is
    // the unprovisioned-worktree failure reading green.
    assert!(
        checked > 0,
        "IR0 three-way differential checked ZERO fixtures — the environment did not run"
    );
    eprintln!(
        "IR0 three-way: checked {checked} fixtures; {records} records over {bundle_bytes} bundle \
         bytes ({bundle_framed} framed, {} opaque = {:.1}%); .ex {ex_bytes} bytes, {ex_opaque} \
         opaque ({:.2}%); the two views' segment counts DIFFER on {differ} of {checked}",
        bundle_bytes - bundle_framed,
        100.0 * (bundle_bytes - bundle_framed) as f64 / bundle_bytes.max(1) as f64,
        100.0 * ex_opaque as f64 / ex_bytes.max(1) as f64,
    );
}

/// The three-way differential is only worth running if the reference can
/// actually disagree. Six hand-built streams, each exercising a clause the
/// prose names, run WITHOUT a toolchain so this file is never entirely skipped.
#[test]
fn ir0_three_way_differential_portable_cases() {
    let lo = [0x4Cu8, 0x4F, 0x11];
    let cases: Vec<(&str, Vec<u8>)> = vec![
        ("empty", vec![]),
        ("no-marker", vec![0, 1, 2, 3]),
        ("marker-at-zero", vec![0x4F, 0x1F, 0xAA]),
        ("opaque-head", vec![0x11, 0x22, 0x4F, 0x1F, 0xAA]),
        ("truncated-marker", vec![0x11, 0x4F]),
        ("overlapping-candidate", vec![0x4F, 0x4F, 0x1F, 0x00]),
        (
            "two-bodies-one-start",
            [
                &[0x4F, 0x1F, 0x00, 0x00][..],
                &lo[..],
                &[0x00, 0x00][..],
                &lo[..],
                &[0x00][..],
            ]
            .concat(),
        ),
        (
            "lo-before-any-start",
            [&lo[..], &[0x00, 0x4F, 0x1F, 0x00][..]].concat(),
        ),
        (
            "bare-4c-thunk",
            vec![0x4F, 0x1F, 0x53, 0x53, 0x46, 0x4C, 0x53, 0x00],
        ),
    ];
    let mut differ = 0usize;
    for (name, ex) in &cases {
        let (g, b) = three_way(name, ex);
        if g != b {
            differ += 1;
        }
    }
    // **C1's property, asserted**: at least one input on which the two views
    // are NOT the same segmentation. If a refactor ever makes them agree
    // everywhere, this is what goes red — and the flattering direction is
    // exactly the one that reads green everywhere else.
    assert!(
        differ > 0,
        "IR0's two views agreed on every case — the splitters have been unified, \
         which moves the census numerator everything else is differenced against"
    );
    eprintln!(
        "IR0 portable three-way: {} cases, views differ on {differ}",
        cases.len()
    );
}
