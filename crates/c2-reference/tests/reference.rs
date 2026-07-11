//! Integration tests for the reference toolchain wrapper. All are guarded by a
//! runtime `Toolchain::locate()` check: if the toolchain is absent they print
//! "SKIP: toolchain absent" and return (they never fail).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use c2_il::is_ex_magic;
use c2_obj::{ObjDiff, ObjImage};
use c2_reference::Toolchain;

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("fixtures/cpp")
        .join(name)
}

fn work(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let d = std::env::temp_dir().join(format!(
        "c2rs-ref-{tag}-{}-{}-{}",
        std::process::id(),
        nanos,
        n
    ));
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// (i) compile_obj yields a non-empty obj beginning with a plausible COFF
/// machine word (Xbox 360 = POWERPCBE, 0x01F2 little-endian → bytes F2 01).
#[test]
fn compile_obj_yields_coff() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    let w = work("compile");
    let obj = tc
        .compile_obj(&fixture("add3.cpp"), &w.join("add3.obj"))
        .expect("reference compile_obj failed");
    let b = obj.as_bytes();
    assert!(!b.is_empty(), "obj is empty");
    assert!(b.len() >= 20, "obj implausibly short: {} bytes", b.len());
    assert_eq!(&b[0..2], &[0xF2, 0x01], "COFF machine word not POWERPCBE");
    std::fs::remove_dir_all(&w).ok();
}

/// (ii) determinism: compile the same fixture twice; normalized compare must be
/// Identical (only the COFF timestamp is allowed to differ).
#[test]
fn compile_obj_is_deterministic() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    let w = work("determinism");
    // Same output path both times: MSVC embeds the /Fo path in the COFF, so
    // the obj is deterministic in (source, output-path). Different paths would
    // differ only in the embedded filename, not a real nondeterminism.
    let out = w.join("det.obj");
    let a = tc
        .compile_obj(&fixture("add3.cpp"), &out)
        .expect("compile a");
    let b = tc
        .compile_obj(&fixture("add3.cpp"), &out)
        .expect("compile b");
    assert_eq!(
        ObjImage::diff(&a, &b),
        ObjDiff::Identical,
        "two compiles of the same source differ after timestamp normalization"
    );
    std::fs::remove_dir_all(&w).ok();
}

/// (iii) capture_il yields a bundle with a non-empty `.ex` passing is_ex_magic.
#[test]
fn capture_il_yields_ex_bundle() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    let w = work("capture");
    let bundle = tc
        .capture_il(&fixture("add3.cpp"), &w)
        .expect("reference capture_il failed");
    let ex = bundle.ex().expect("bundle has no .ex");
    assert!(!ex.is_empty(), ".ex is empty");
    assert!(is_ex_magic(ex), ".ex does not start with header magic 5B 80 54 0A");
    // A real capture yields all five files.
    for suffix in ["ex", "gl", "sy", "in", "db"] {
        assert!(bundle.get(suffix).is_some(), "missing {suffix} file");
    }
    std::fs::remove_dir_all(&w).ok();
}

/// (iv) capture stability: capture twice; `.ex` bytes must be identical.
#[test]
fn capture_il_is_stable() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    let wa = work("cap_a");
    let wb = work("cap_b");
    let a = tc.capture_il(&fixture("add3.cpp"), &wa).expect("capture a");
    let b = tc.capture_il(&fixture("add3.cpp"), &wb).expect("capture b");
    assert_eq!(
        a.ex().unwrap(),
        b.ex().unwrap(),
        "two IL captures of the same source produced different .ex bytes"
    );
    std::fs::remove_dir_all(&wa).ok();
    std::fs::remove_dir_all(&wb).ok();
}

/// (v) **P0.1 regression:** for every bundled fixture, `capture_reference` +
/// `replay` (to the reference's exact `/Fo` path) reproduces the pipeline obj
/// **byte-exact** (normalized compare). Guarded on `strace` + mingw; skips
/// cleanly otherwise.
#[test]
fn p0_1_replay_byte_exact() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    if !tc.has_strace() {
        eprintln!("SKIP: strace absent (needed to keep the IL bundle)");
        return;
    }
    if !tc.has_mingw() {
        eprintln!("SKIP: i686-w64-mingw32-gcc absent (needed to build c2host)");
        return;
    }
    for name in ["add3.cpp", "il_bool_materialization.cpp", "il_call_return.cpp"] {
        let w = work("p01");
        let captured = tc
            .capture_reference(&fixture(name), &w.join("cap"))
            .unwrap_or_else(|e| panic!("capture_reference failed for {name}: {e}"));
        // Replay to the SAME /Fo path as the reference (ref bytes already in
        // captured.ref_obj) so the embedded path string matches.
        let ref_path = captured.ref_obj_path.clone();
        let replay = tc
            .replay(&captured, &w.join("replay_il"), &ref_path)
            .unwrap_or_else(|e| panic!("replay failed for {name}: {e}"));
        assert_eq!(
            ObjImage::diff(&captured.ref_obj, &replay),
            ObjDiff::Identical,
            "P0.1 replay not byte-exact for {name}: ref={}B replay={}B",
            captured.ref_obj.len(),
            replay.len(),
        );
        // Sanity: both are real, non-trivial COFF objs.
        assert!(captured.ref_obj.len() > 20 && replay.len() > 20);
        std::fs::remove_dir_all(&w).ok();
    }
}
