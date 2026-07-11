//! K3a — the length-consistent IL edit primitive, gated against the live oracle.
//!
//! This is the [`differential.rs`](differential) sibling for **edits**: it does
//! not just *read* a captured bundle, it applies an [`c2_il::IlModel`] length
//! edit, writes the edited bundle, replays it through standalone c2, and asserts
//! the resulting obj is **byte-exact** (timestamp-normalized) to a native capture
//! of the equivalent-source program. Each P0.6a length-plasticity experiment is
//! reproduced here as a first-class *edit* (not a byte-poke probe):
//!
//!   * **A** — varint widen (same value, +4 B), single/last fn ⇒ byte-exact to
//!     the unedited baseline (semantic no-op; no `.gl` bookkeeping).
//!   * **D + C** — widen a NON-last function (`.gl` offset re-emit obligated):
//!     the API's re-emit yields a byte-exact baseline obj (**D**), while the same
//!     `.ex` with a STALE `.gl` (re-emit skipped) crashes c2 to no obj (**C**) —
//!     proving the offset re-emit the edit performs is load-bearing.
//!   * **E** — grow by inserting a `+k` term (`a+5` → `(a+5)+5`, +6 B) ⇒
//!     byte-exact to a direct `a+5+5` capture (c2 re-folds `5+5` → `addi …,10`).
//!   * **F** — shrink by deleting a term (`a+b+c` → `a+b`, −7 B) ⇒ byte-exact to
//!     a direct `a+b` capture.
//!
//! Every replay is bounded by [`Toolchain::replay_within`] with a TIMEOUT
//! (P0.6a proved a `.gl`/`.ex` function-set mismatch can hang c2, not crash) — a
//! timeout is a test failure with a clear message, never a hang. Toolchain-gated:
//! skips cleanly (never fails) when the toolchain / strace / mingw are absent.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use c2_il::{ExToken, IlModel};
use c2_obj::{ObjDiff, ObjImage};
use c2_reference::{CapturedReference, Toolchain};

/// Replay budget — well above a normal ~1 s replay, tight enough to turn a
/// function-set-mismatch hang (P0.6a G) into a prompt failure.
const TIMEOUT: Duration = Duration::from_secs(60);

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
        "c2rs-edit-{tag}-{}-{}-{}",
        std::process::id(),
        nanos,
        n
    ));
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// The toolchain + replay prerequisites, or `None` to skip cleanly.
fn ready() -> Option<Toolchain> {
    let tc = Toolchain::locate()?;
    if !tc.has_strace() {
        eprintln!("SKIP: strace absent");
        return None;
    }
    if !tc.has_mingw() {
        eprintln!("SKIP: i686-w64-mingw32-gcc absent");
        return None;
    }
    Some(tc)
}

/// A [`CapturedReference`] identical to `base` but carrying `bundle` (the edited
/// IL). Its argv (`-f`, backend flags) and base name are reused verbatim; the
/// replay swaps only `-il`/`-Fo`.
fn with_bundle(base: &CapturedReference, bundle: c2_il::IlBundle) -> CapturedReference {
    CapturedReference {
        bundle,
        ..base.clone()
    }
}

/// The token index of the first `Lit` in function `fn_index`.
fn first_lit(model: &IlModel, fn_index: usize) -> usize {
    model
        .function_tokens(fn_index)
        .unwrap()
        .iter()
        .position(|t| matches!(t, ExToken::Lit { .. }))
        .expect("a literal in this function")
}

// ---- A: varint widen, single fn — byte-exact to the unedited baseline --------

#[test]
fn edit_a_widen_single_fn_byte_exact() {
    let Some(tc) = ready() else { return };
    let w = work("A");

    // Capture `int addk(int a){ return a+5; }` and replay the UNEDITED bundle to
    // a fixed obj path — that is the baseline the widen must reproduce.
    let base = tc
        .capture_reference(&fixture("mvp_edit_addk.cpp"), &w.join("cap"))
        .expect("capture addk");
    let fixed = w.join("fixed.obj");
    let baseline = tc
        .replay_within(&base, &w.join("base_il"), &fixed, TIMEOUT)
        .expect("baseline replay");

    // Widen the only literal (same value 5, 1-byte -> `80`+LE32, +4 B).
    let mut model = IlModel::parse(&base.bundle).expect("parse");
    let idx = first_lit(&model, 0);
    let report = model.set_literal_wide(0, idx, true).expect("widen");
    assert_eq!(report.byte_delta, 4);

    let edited = tc
        .replay_within(&with_bundle(&base, model.encode()), &w.join("edit_il"), &fixed, TIMEOUT)
        .expect("edited replay (should compile: pure length pad)");

    assert_byte_exact(&edited, &baseline, "A widen == baseline");
    std::fs::remove_dir_all(&w).ok();
}

// ---- D + C: widen a NON-last fn — the `.gl` offset re-emit is load-bearing ----

#[test]
fn edit_d_widen_nonlast_reemit_gl_and_c_stale_negative() {
    let Some(tc) = ready() else { return };
    let w = work("DC");

    // mvp_lit is a 3-function TU (addk `a+5`, subk `a-5`, konst `42`); editing
    // fn0 shifts fns 1,2, so the `.gl` body-start offsets MUST be re-emitted.
    let base = tc
        .capture_reference(&fixture("mvp_lit.cpp"), &w.join("cap"))
        .expect("capture mvp_lit");
    let fixed = w.join("fixed.obj");
    let baseline = tc
        .replay_within(&base, &w.join("base_il"), &fixed, TIMEOUT)
        .expect("baseline replay");

    let mut model = IlModel::parse(&base.bundle).expect("parse");
    let gl_before = model.gl_body_start_offsets();
    assert_eq!(gl_before.len(), 3, "mvp_lit has three typed .gl offsets");
    let idx = first_lit(&model, 0);
    let report = model.set_literal_wide(0, idx, true).expect("widen fn0");
    assert_eq!(report.byte_delta, 4);
    // fn0 unchanged; fns 1,2 each bumped +4 (the P0.6a experiment-D re-emit).
    assert_eq!(
        model.gl_body_start_offsets(),
        vec![gl_before[0], gl_before[1] + 4, gl_before[2] + 4]
    );

    // D (positive): the API re-emitted `.gl` → byte-exact baseline.
    let good_bundle = model.encode();
    let edited = tc
        .replay_within(&with_bundle(&base, good_bundle.clone()), &w.join("good_il"), &fixed, TIMEOUT)
        .expect("D: edited replay with re-emitted .gl");
    assert_byte_exact(&edited, &baseline, "D widen-nonlast (.gl re-emit) == baseline");

    // C (negative): the SAME edited `.ex`, but a STALE `.gl` (re-emit skipped) —
    // c2 seeks a downstream function at the old offset and SIGSEGVs (no obj).
    let mut stale = good_bundle;
    stale.set("gl", base.bundle.get("gl").expect("gl").to_vec());
    let neg = tc.replay_within(&with_bundle(&base, stale), &w.join("stale_il"), &fixed, TIMEOUT);
    // The re-emit is load-bearing: skipping it must break byte-exactness — either
    // c2 crashes/hangs to no obj (P0.6a C SIGSEGV), or it produces a DIFFERENT
    // (wrong) obj. Never the baseline. (The API path above IS byte-exact.)
    match neg {
        Err(e) => eprintln!("C: stale .gl failed to compile (as P0.6a C): {e}"),
        Ok(obj) => {
            assert!(
                !matches!(ObjImage::diff(&obj, &baseline), ObjDiff::Identical),
                "C: a stale .gl reproduced the baseline obj — the .gl re-emit would NOT be load-bearing"
            );
            eprintln!("C: stale .gl compiled but to a NON-baseline (wrong) obj — re-emit still load-bearing");
        }
    }
    std::fs::remove_dir_all(&w).ok();
}

// ---- E: grow by inserting a `+k` term — byte-exact to the direct capture ------

#[test]
fn edit_e_grow_insert_term_byte_exact() {
    let Some(tc) = ready() else { return };
    let w = work("E");

    // Base `a+5`; target `a+5+5` (c2 folds `5+5`→10). Both replay to a FIXED obj
    // path so the embedded S_OBJNAME cannot confound the compare.
    let base = tc
        .capture_reference(&fixture("mvp_edit_addk.cpp"), &w.join("cap_base"))
        .expect("capture addk");
    let tgt = tc
        .capture_reference(&fixture("mvp_edit_addk2.cpp"), &w.join("cap_tgt"))
        .expect("capture addk2");
    let fixed = w.join("fixed.obj");

    // Insert `LIT 5 ; ADD` after the body's existing ADD → postfix `(a+5)+5`.
    let mut model = IlModel::parse(&base.bundle).expect("parse");
    let toks = model.function_tokens(0).unwrap();
    let add = toks
        .iter()
        .position(|t| matches!(t, ExToken::Add))
        .expect("the a+5 ADD");
    let report = model
        .splice_function_tokens(
            0,
            add + 1..add + 1,
            vec![ExToken::Lit { value: 5, wide: false }, ExToken::Add],
        )
        .expect("insert +5 term");
    assert_eq!(report.byte_delta, 6);

    let edited = tc
        .replay_within(&with_bundle(&base, model.encode()), &w.join("edit_il"), &fixed, TIMEOUT)
        .expect("grown-IL replay");
    let direct = tc
        .replay_within(&tgt, &w.join("tgt_il"), &fixed, TIMEOUT)
        .expect("direct a+5+5 replay");

    assert_byte_exact(&edited, &direct, "E grow (a+5)+5 == direct a+5+5");
    std::fs::remove_dir_all(&w).ok();
}

// ---- F: shrink by deleting a term — byte-exact to the direct capture ----------

#[test]
fn edit_f_shrink_delete_term_byte_exact() {
    let Some(tc) = ready() else { return };
    let w = work("F");

    // Base `add3 = a+b+c`; target `add3 = a+b` (param c left unreferenced).
    let base = tc
        .capture_reference(&fixture("mvp_add3.cpp"), &w.join("cap_base"))
        .expect("capture add3");
    let tgt = tc
        .capture_reference(&fixture("mvp_edit_ab.cpp"), &w.join("cap_tgt"))
        .expect("capture ab");
    let fixed = w.join("fixed.obj");

    // Delete the last `LOAD` (that is `c`) and its following `ADD` → `a+b`.
    let mut model = IlModel::parse(&base.bundle).expect("parse");
    let toks = model.function_tokens(0).unwrap();
    let last_load = toks
        .iter()
        .rposition(|t| matches!(t, ExToken::Load(_)))
        .expect("a load");
    assert!(
        matches!(toks[last_load + 1], ExToken::Add),
        "LOAD c must be followed by ADD"
    );
    let report = model
        .splice_function_tokens(0, last_load..last_load + 2, vec![])
        .expect("delete +c term");
    assert_eq!(report.byte_delta, -7);

    let edited = tc
        .replay_within(&with_bundle(&base, model.encode()), &w.join("edit_il"), &fixed, TIMEOUT)
        .expect("shrunk-IL replay");
    let direct = tc
        .replay_within(&tgt, &w.join("tgt_il"), &fixed, TIMEOUT)
        .expect("direct a+b replay");

    assert_byte_exact(&edited, &direct, "F shrink a+b+c -> a+b == direct a+b");
    std::fs::remove_dir_all(&w).ok();
}

/// Assert two objs are byte-exact on their timestamp-normalized bytes.
fn assert_byte_exact(got: &ObjImage, want: &ObjImage, what: &str) {
    match ObjImage::diff(got, want) {
        ObjDiff::Identical => {}
        ObjDiff::Differs {
            first_offset,
            a_len,
            b_len,
        } => panic!(
            "{what}: NOT byte-exact — first diff at {first_offset} (got {a_len} B, want {b_len} B)"
        ),
    }
}
