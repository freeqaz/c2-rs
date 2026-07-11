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

        // Sanity: the .ex carried decoded body tokens and the .gl carried one
        // typed body-start offset per function (cross-checked against .ex).
        assert!(
            !model.ex_tokens().is_empty(),
            "{name}: expected decoded .ex tokens"
        );
        let noff = model.gl_body_start_offsets().len();
        assert!(noff >= 1, "{name}: expected >=1 typed .gl offset, got {noff}");

        checked += 1;
        std::fs::remove_dir_all(&w).ok();
    }

    assert!(checked >= 15, "expected the full fixture spread, ran {checked}");
    eprintln!("K1 round-trip: {checked} fixture bundles byte-identical");
}
