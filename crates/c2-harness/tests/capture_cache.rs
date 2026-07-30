//! **The capture cache's validator, proven** (roadmap #15).
//!
//! A cache is an instrument that answers questions without doing the work, so
//! the only interesting property is whether a *wrong* cache is detectable. This
//! test deliberately poisons an entry — one flipped byte in the cached `.ex`,
//! and separately one flipped byte in the cached obj — and requires the
//! bypass-and-compare path to name it. It also proves the ordinary hit is
//! byte-identical to the capture that filled it, which is the claim the warm
//! scan rests on.
//!
//! Toolchain-gated: skips cleanly (never fails) when `Toolchain::locate()` is
//! `None` or `strace` is absent, per the CLAUDE.md hard constraint.

use std::path::{Path, PathBuf};

use c2_harness::capture_cache::{compare_captures, CacheOutcome, CaptureCache, CaptureDiff};
use c2_reference::Toolchain;

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/cpp/add3.cpp")
}

fn work(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!(
        "c2rs-cachetest-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// The single cache entry directory under `root` (there is exactly one).
fn only_entry(root: &Path) -> PathBuf {
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(root)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_dir())
        .collect();
    assert_eq!(dirs.len(), 1, "expected exactly one cache entry in {root:?}");
    dirs.pop().unwrap()
}

/// Flip one byte in the middle of `path`.
fn poison_file(path: &Path) {
    let mut bytes = std::fs::read(path).unwrap();
    assert!(bytes.len() > 16, "{path:?} too small to poison");
    let i = bytes.len() / 2;
    bytes[i] ^= 0xff;
    std::fs::write(path, bytes).unwrap();
}

#[test]
fn a_hit_is_byte_identical_and_a_poisoned_entry_is_caught() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    if !tc.has_strace() {
        eprintln!("SKIP: strace absent");
        return;
    }
    // NOT a skip: this fixture is tracked in the repo, so its absence is a bug
    // in the test, not an environment fact. A skip here would have made the
    // whole validator proof silently vacuous — which it briefly was.
    let cpp = fixture();
    assert!(cpp.exists(), "tracked fixture missing: {}", cpp.display());
    let base = work("validator");
    let root = base.join("cache");
    let fallback = base.join("work");
    std::fs::create_dir_all(&fallback).unwrap();
    let src_arg = c2_reference::to_wibo_path(&cpp.canonicalize().unwrap());
    let flags: Vec<String> = ["/Ox", "/GS-", "/c"].iter().map(|s| s.to_string()).collect();

    // 1. Cold: a miss that fills the entry.
    let cache = CaptureCache::new(root.clone(), &tc, None, 0).unwrap();
    let (cold, outcome) = cache.capture(&tc, &src_arg, &flags, None, &fallback);
    let cold = cold.expect("cold capture");
    assert_eq!(outcome, CacheOutcome::Miss);
    assert_eq!(cache.stats().misses, 1);

    // 2. Warm: a hit, byte-identical to the capture that filled it. This is the
    //    claim every warm scan rests on, so it is asserted field-by-field rather
    //    than by "the verdict was the same".
    let cache = CaptureCache::new(root.clone(), &tc, None, 0).unwrap();
    let (warm, outcome) = cache.capture(&tc, &src_arg, &flags, None, &fallback);
    let warm = warm.expect("warm capture");
    assert_eq!(outcome, CacheOutcome::Hit);
    assert_eq!(
        compare_captures(&cold, &warm),
        CaptureDiff::Identical,
        "a cache hit differed from the capture that filled it — a hit is a REPLAY of \
         bytes on disk, so not even the timestamp may move"
    );
    assert_eq!(cold.ref_obj_path, warm.ref_obj_path);

    // 3. The validator against an INTACT entry: re-captures and agrees. This is
    //    also the standing proof that a capture at a fixed output path is
    //    reproducible at all — without it, "poisoned" would be unfalsifiable.
    let cache = CaptureCache::new(root.clone(), &tc, None, 1).unwrap();
    let (_, outcome) = cache.capture(&tc, &src_arg, &flags, None, &fallback);
    assert_eq!(
        outcome,
        CacheOutcome::Validated,
        "re-capturing an intact entry in place did not reproduce it: {:?}",
        cache.stats().poison_detail
    );
    assert_eq!(cache.stats().poisoned, 0);

    // 4. Poison the cached `.ex` — the file a corrupt cache would silently move
    //    every census number with, while the differential stayed green.
    let entry = only_entry(&root);
    let ex = std::fs::read_dir(&entry)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .find(|p| {
            p.file_name()
                .map(|n| {
                    let n = n.to_string_lossy();
                    n.starts_with("_CL_") && n.ends_with("ex")
                })
                .unwrap_or(false)
        })
        .expect("cached .ex");
    poison_file(&ex);

    let cache = CaptureCache::new(root.clone(), &tc, None, 1).unwrap();
    let (_, outcome) = cache.capture(&tc, &src_arg, &flags, None, &fallback);
    assert_eq!(
        outcome,
        CacheOutcome::Poisoned,
        "a flipped byte in the cached .ex was served as a clean hit"
    );
    let st = cache.stats();
    assert_eq!(st.poisoned, 1);
    assert!(
        st.poison_detail[0].contains(".ex differs at offset"),
        "the validator did not name the field that differed: {:?}",
        st.poison_detail
    );

    // 5. …and the validator self-heals: the re-capture overwrote the entry, so
    //    the next validation is clean again.
    let cache = CaptureCache::new(root.clone(), &tc, None, 1).unwrap();
    let (_, outcome) = cache.capture(&tc, &src_arg, &flags, None, &fallback);
    assert_eq!(outcome, CacheOutcome::Validated);

    // 6. Poison the cached obj instead — the other half of the entry.
    poison_file(&entry.join("out.obj"));
    let cache = CaptureCache::new(root.clone(), &tc, None, 1).unwrap();
    let (_, outcome) = cache.capture(&tc, &src_arg, &flags, None, &fallback);
    assert_eq!(outcome, CacheOutcome::Poisoned);
    assert!(
        cache.stats().poison_detail[0].contains("reference obj differs at normalized offset"),
        "{:?}",
        cache.stats().poison_detail
    );

    let _ = std::fs::remove_dir_all(&base);
}

/// Changing an input must miss, not hit — the staleness half of the contract.
/// Uses a scratch copy of the fixture so the *contents* at a fixed path change,
/// which is precisely the case an mtime-keyed cache gets wrong.
#[test]
fn editing_the_source_at_the_same_path_misses() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    if !tc.has_strace() {
        eprintln!("SKIP: strace absent");
        return;
    }
    let src = fixture();
    assert!(src.exists(), "tracked fixture missing: {}", src.display());
    let base = work("stale");
    let root = base.join("cache");
    let fallback = base.join("work");
    std::fs::create_dir_all(&fallback).unwrap();
    let cpp = base.join("t.cpp");
    std::fs::write(&cpp, "int f(int a, int b) { return a + b; }\n").unwrap();
    let src_arg = c2_reference::to_wibo_path(&cpp.canonicalize().unwrap());
    let flags: Vec<String> = ["/Ox", "/GS-", "/c"].iter().map(|s| s.to_string()).collect();

    let cache = CaptureCache::new(root.clone(), &tc, None, 0).unwrap();
    let (first, o1) = cache.capture(&tc, &src_arg, &flags, None, &fallback);
    let first = first.expect("first capture");
    assert_eq!(o1, CacheOutcome::Miss);

    // Same path, same flags, same toolchain — different bytes.
    std::fs::write(&cpp, "int f(int a, int b) { return a - b; }\n").unwrap();
    let cache = CaptureCache::new(root.clone(), &tc, None, 0).unwrap();
    let (second, o2) = cache.capture(&tc, &src_arg, &flags, None, &fallback);
    let second = second.expect("second capture");
    assert_eq!(
        o2,
        CacheOutcome::Miss,
        "an edited source was served from cache — the key is not content-addressed"
    );
    assert!(
        matches!(compare_captures(&first, &second), CaptureDiff::Differs(_)),
        "the two captures were identical, so this test proves nothing about the key"
    );

    // A different flag set is a different key too.
    let other: Vec<String> = ["/O1", "/GS-", "/c"].iter().map(|s| s.to_string()).collect();
    let cache = CaptureCache::new(root.clone(), &tc, None, 0).unwrap();
    let (_, o3) = cache.capture(&tc, &src_arg, &other, None, &fallback);
    assert_eq!(o3, CacheOutcome::Miss, "a flag change was served from cache");

    let _ = std::fs::remove_dir_all(&base);
}
