//! K1 codec round-trip gate over the **full captured-fixture spread**.
//!
//! Toolchain-guarded like `differential.rs`: skips cleanly (never fails) when
//! `Toolchain::locate()` is `None`. For every `fixtures/cpp/*.cpp`, capture the
//! real IL bundle through the harness and assert
//! `IlModel::encode(IlModel::parse(bundle)) == bundle` byte-for-byte — the K1
//! invariant, exercised against live-toolchain output rather than committed
//! bytes (the captured `.gl` embeds the host source path, so the bundles are not
//! committable; the always-on portable round-trip lane lives in
//! `c2-il::codec`'s unit tests over hand-built bundles).
//!
//! The spread covers every shape the codec must handle: single-function
//! arithmetic (`mvp_add3`), multi-function TUs (`mvp_two`, `mvp_lit`, `mvp_sub`),
//! wide literals (`mvp_wide`), the void tail call (`mvp_call`), the framed call
//! (`mvp_framed`), and the nine out-of-class call shapes — none of which the
//! codec needs to *decode*, only losslessly re-encode.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use c2_harness::all_fixtures;
use c2_il::IlModel;
use c2_reference::Toolchain;

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn work(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let d = std::env::temp_dir().join(format!(
        "c2rs-ilrt-{tag}-{}-{}-{}",
        std::process::id(),
        nanos,
        n
    ));
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn roundtrip_all_fixtures_byte_identical() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    let fixtures = all_fixtures();
    assert!(!fixtures.is_empty(), "no fixtures found");

    let mut checked = 0usize;
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

        // Parse must succeed (fail-closed: parse itself verifies re-encode).
        let model = IlModel::parse(&bundle)
            .unwrap_or_else(|e| panic!("codec parse refused {name}: {e}"));

        // And the encoded bundle is byte-identical, file for file.
        let back = model.encode();
        assert_eq!(
            back.base_name, bundle.base_name,
            "base name changed for {name}"
        );
        assert_eq!(
            back.files.keys().collect::<Vec<_>>(),
            bundle.files.keys().collect::<Vec<_>>(),
            "file set changed for {name}"
        );
        for (suffix, orig) in &bundle.files {
            let got = back.get(suffix).unwrap_or(&[]);
            assert!(
                got == orig.as_slice(),
                "{name}: .{suffix} not byte-identical (orig {} B, re-encoded {} B)",
                orig.len(),
                got.len(),
            );
        }

        // Sanity: the .gl carried EXACTLY one structurally-identified body-start
        // offset per function — the `== function_count` invariant K3 relies on
        // (K2a strengthened this from the earlier `>= 1`, now that offsets are
        // located by record framing and gated 1:1/in-order against the `.ex`
        // `4F 1F` markers). This 1:1 claim holds for every fixture.
        let noff = model.gl_body_start_offsets().len();
        let nfns = model.ex_function_count();
        assert_eq!(
            noff, nfns,
            "{name}: typed .gl offsets ({noff}) must be 1:1 with .ex functions ({nfns})"
        );

        // The body-token claim is conditional on the TU actually having bodies.
        // `mvp_empty.cpp` (R1) defines no functions: the front end still emits a
        // full five-file bundle, but its `.ex` carries no function bodies at
        // all, so "decoded body tokens" is legitimately zero. The round-trip
        // itself is still gated byte-for-byte above — this only scopes the
        // structural expectation, it does not relax the codec's invariant.
        if c2_il::is_empty_module(bundle.ex().unwrap_or(&[])) {
            assert_eq!(nfns, 0, "{name}: empty module must decode to 0 functions");
            assert!(
                model.ex_tokens().is_empty(),
                "{name}: empty module must decode to 0 body tokens"
            );
        } else if has_lo_anchored_body(bundle.ex().unwrap_or(&[])) {
            assert!(
                !model.ex_tokens().is_empty(),
                "{name}: expected decoded .ex tokens"
            );
            assert!(nfns >= 1, "{name}: expected >=1 function, got {nfns}");
        } else {
            // **The #158 class, named rather than left as a hole.** A dynamic
            // initializer (`il_dyninit_static.cpp`) is not an empty module — it
            // has a `4F 1F` function start, and a `.gl` record binds a name to
            // it — but its body opens `4C 53` where a source function opens
            // `4C 4F 11`, and the codec's model splits bodies on the three-byte
            // `4C 4F 11`. So it decodes to zero tokens while carrying a
            // function.
            //
            // The precondition above used to be `is_empty_module`, which asks a
            // question about the `4F 1F` marker and was being used to predict
            // the behaviour of an `LO`-anchored split. That is ROADMAP §10.11's
            // defect exactly — a count is only evidence about the predicate that
            // produced it — reached here by a test rather than by prose.
            //
            // Scoped, not relaxed: the byte-for-byte round-trip above still
            // gates this fixture, and the exception is closed on both sides.
            assert!(
                model.ex_tokens().is_empty(),
                "{name}: a body with no `4C 4F 11` must decode to 0 tokens, or the \
                 split found something this branch does not describe"
            );
            assert!(
                nfns >= 1,
                "{name}: has no LO-anchored body and is not an empty module, so it \
                 must still carry a `4F 1F` function start; got {nfns}"
            );
        }

        checked += 1;
        std::fs::remove_dir_all(&w).ok();
    }

    assert!(checked >= 15, "expected the full fixture spread, ran {checked}");
    eprintln!("K1 round-trip: {checked} fixture bundles byte-identical");
}

/// True iff `.ex` carries at least one `4C 4F 11` — the marker the codec's
/// model and `c2_il`'s census splitter both anchor bodies on.
///
/// Deliberately **not** `is_empty_module`, which asks about `4F 1F` as well and
/// therefore answers a different question (ROADMAP §10.11). Duplicated as three
/// bytes here rather than exported: this test asserts a fact about the split,
/// and reading the constant from the crate under test would let a change to it
/// move the assertion silently.
fn has_lo_anchored_body(ex: &[u8]) -> bool {
    ex.windows(3).any(|w| w == [0x4C, 0x4F, 0x11])
}
