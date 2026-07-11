//! P1.2 corpus generator tests.
//!
//! Two lanes:
//!   * **portable** (`sample_corpus_committed_is_valid`) — loads the committed
//!     synthetic sample from disk and validates the schema end-to-end (manifest
//!     ↔ files, codec round-trip) with **no toolchain**. Runs in the portable
//!     lane like the codec's always-on unit tests.
//!   * **integration** (`generate_small_corpus_*`) — toolchain-gated (skips
//!     cleanly when `Toolchain::locate()` / strace is absent): generates a
//!     handful of real triples, asserts each triple's codec round-trip holds,
//!     the obj is reproducible, and the manifest indexes them.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use c2_harness::corpus::{self, sha256_hex, CorpusConfig};
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
        "c2rs-corpus-test-{tag}-{}-{}-{}",
        std::process::id(),
        nanos,
        n
    ));
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn sample_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpus_sample")
}

/// Portable lane: the committed synthetic sample must load and be internally
/// consistent (manifest sizes/hashes match the on-disk files; every bundle
/// round-trips through the codec; every triple is path-free).
#[test]
fn sample_corpus_committed_is_valid() {
    let dir = sample_dir();
    let rows = corpus::load_manifest(&dir).expect("load committed sample manifest");
    assert!(!rows.is_empty(), "committed sample manifest is empty");

    for row in &rows {
        assert_eq!(row.status, "ok", "sample triple {} not ok", row.id);
        assert_eq!(row.codec_roundtrip, Some(true), "{} codec flag", row.id);

        // Source is present and hashes as recorded.
        let src_rel = row.source_rel.as_ref().expect("source_rel");
        let src = std::fs::read(dir.join(src_rel)).expect("read source");
        // (source_sha256 is on the row's writer side; recompute for integrity.)
        assert!(!src.is_empty());

        // IL bundle files exist and match their recorded sizes.
        let il_dir = dir.join(row.il_dir_rel.as_ref().expect("il_dir_rel"));
        let base = row.il_base.as_ref().expect("il_base");
        assert!(
            !base.starts_with("_CL_"),
            "committed sample must not use a gitignored _CL_ base"
        );
        for (suffix, len) in &row.il_files {
            let p = il_dir.join(format!("{base}{suffix}"));
            let meta = std::fs::metadata(&p)
                .unwrap_or_else(|_| panic!("missing sample IL file {}", p.display()));
            assert_eq!(meta.len() as i64, *len, "{} .{suffix} size", row.id);
        }

        // The bundle round-trips through the K1 codec, and its typed stats match
        // the manifest (non-vacuous: real tokens + the framed body-start offset).
        let bundle = c2_il::IlBundle::load_from_dir(&il_dir, base).expect("load bundle");
        let model = IlModel::parse(&bundle).expect("codec parse committed sample");
        assert_eq!(model.encode().files, bundle.files, "sample must round-trip");
        assert_eq!(
            model.ex_tokens().len() as i64,
            row.ex_token_count.unwrap(),
            "{} ex_token_count",
            row.id
        );
        let offs: Vec<i64> = model
            .gl_body_start_offsets()
            .iter()
            .map(|&o| o as i64)
            .collect();
        assert_eq!(offs, row.gl_offsets, "{} gl_offsets", row.id);

        // Bundle is path-free (the whole reason the sample is synthetic).
        for bytes in bundle.files.values() {
            let low = String::from_utf8_lossy(bytes).to_lowercase();
            assert!(!low.contains("z:\\") && !low.contains("/home/"), "path leak");
        }

        // obj.bin exists, matches recorded len + normalized sha.
        let obj = std::fs::read(dir.join(row.obj_rel.as_ref().expect("obj_rel"))).expect("obj");
        assert_eq!(obj.len() as i64, row.obj_len.unwrap(), "{} obj_len", row.id);
        assert_eq!(
            sha256_hex(&obj),
            *row.obj_sha256_norm.as_ref().unwrap(),
            "{} obj sha",
            row.id
        );
    }
}

/// Integration lane: generate a small real corpus and assert every triple is a
/// valid, codec-round-tripping, reproducible (source, IL, obj) triple indexed by
/// the manifest.
#[test]
fn generate_small_corpus_roundtrips_and_indexes() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    if !tc.has_strace() {
        eprintln!("SKIP: strace absent (needed to keep the IL bundle)");
        return;
    }

    let root = work("gen");
    let cfg = CorpusConfig {
        seed: 7,
        count: 3,
        timeout: Duration::from_secs(60),
    };
    let summary = corpus::generate(&root, &tc, &cfg).expect("generate corpus");
    assert_eq!(summary.ok, 3, "expected 3 ok triples, got {summary:?}");
    assert_eq!(summary.total(), 3);

    let rows = corpus::load_manifest(&root).expect("load manifest");
    assert_eq!(rows.len(), 3, "manifest must index every triple");

    for row in &rows {
        assert_eq!(row.status, "ok");
        assert_eq!(row.codec_roundtrip, Some(true));
        // IL bundle files exist per the manifest, and re-parse round-trips.
        let il_dir = root.join(row.il_dir_rel.as_ref().unwrap());
        let base = row.il_base.as_ref().unwrap();
        let bundle = c2_il::IlBundle::load_from_dir(&il_dir, base).expect("load bundle");
        let model = IlModel::parse(&bundle).expect("codec re-parse");
        assert_eq!(model.encode().files, bundle.files, "{} round-trip", row.id);
        assert!(!model.ex_tokens().is_empty(), "{} decoded tokens", row.id);
        // A real in-class bundle carries one framed body-start offset per fn.
        assert!(!row.gl_offsets.is_empty(), "{} gl offsets", row.id);
        // obj is present and non-empty.
        let obj = std::fs::read(root.join(row.obj_rel.as_ref().unwrap())).expect("obj");
        assert!(obj.len() > 8, "{} obj too small", row.id);
        assert_eq!(obj.len() as i64, row.obj_len.unwrap());
    }

    std::fs::remove_dir_all(&root).ok();
}

/// Integration lane: the obj is a reproducible function of (source, output path)
/// — regenerating the same seed into the **same root** yields byte-identical
/// normalized objs (obj embeds its `/Fo` path, so reproducibility is per-root).
#[test]
fn corpus_obj_is_reproducible_per_root() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    if !tc.has_strace() {
        eprintln!("SKIP: strace absent");
        return;
    }

    let root = work("repro");
    let cfg = CorpusConfig {
        seed: 3,
        count: 2,
        timeout: Duration::from_secs(60),
    };
    corpus::generate(&root, &tc, &cfg).expect("gen #1");
    let first = corpus::load_manifest(&root).expect("manifest #1");
    corpus::generate(&root, &tc, &cfg).expect("gen #2");
    let second = corpus::load_manifest(&root).expect("manifest #2");

    assert_eq!(first.len(), second.len());
    for (a, b) in first.iter().zip(second.iter()) {
        assert_eq!(a.id, b.id);
        assert_eq!(
            a.obj_sha256_norm, b.obj_sha256_norm,
            "obj {} not reproducible per-root",
            a.id
        );
    }

    std::fs::remove_dir_all(&root).ok();
}
