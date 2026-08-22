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

use c2_harness::capture_cache::{
    compare_captures, CacheOutcome, CaptureCache, CaptureDiff, ENTRY_BLOB, LOCK_DIR,
};
use c2_reference::Toolchain;

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/cpp/add3.cpp")
}

fn work(tag: &str) -> PathBuf {
    c2_harness::testsupport::clean_scratch_dir("cachetest", tag)
}

/// The single cache entry directory under `root` (there is exactly one).
///
/// `LOCK_DIR` is the one non-entry child the root is allowed to have — it holds
/// the `O_EXCL` per-key lockfiles. Excluded by name rather than by "skip
/// anything dotted", so that a *new* stray directory still fails this assert
/// instead of being quietly tolerated.
fn only_entry(root: &Path) -> PathBuf {
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(root)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_dir())
        .filter(|p| p.file_name().map(|n| n != LOCK_DIR).unwrap_or(true))
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

/// Flip one byte of the cached `.ex` **inside** the entry blob: decode, mutate
/// the payload, re-encode with a correct digest.
///
/// **Do not "simplify" this to `poison_file(entry.join(ENTRY_BLOB))`.** A raw
/// byte flip fails the blob's own digest, so `read_entry` returns `Miss`, the
/// cache re-captures, and the outcome is `Miss` — not `Poisoned`. The test would
/// then fail for the wrong reason, and relaxing the assertion to accept `Miss`
/// would silently convert this project's only validator proof into a checksum
/// test. What is being proven here is that a *well-formed* entry carrying wrong
/// IL is caught by bypass-and-compare, which is the only check that can see it.
fn poison_cached_ex(entry: &Path) {
    let raw = std::fs::read(entry.join(ENTRY_BLOB)).unwrap();
    let blob = c2_il::decode_entry(&raw).expect("the entry blob must decode");
    let key = blob.key.to_vec();
    let meta = blob.meta.to_string();
    // The base name is metadata, not a section tag, so any name round-trips the
    // payloads unchanged; `read_entry` takes the real one from `meta`.
    let mut bundle = blob.bundle("_CL_poison");
    let mut ex = bundle.get("ex").expect("cached .ex").to_vec();
    assert!(ex.len() > 16, "cached .ex too small to poison");
    let i = ex.len() / 2;
    ex[i] ^= 0xff;
    bundle.set("ex", ex);
    let poisoned = c2_il::encode_entry(&key, &meta, &bundle);
    // The poisoned blob must still be *valid*, or the validator never runs.
    assert!(c2_il::decode_entry(&poisoned).is_ok());
    std::fs::write(entry.join(ENTRY_BLOB), poisoned).unwrap();
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

    // 4. The inode claim, proven against the real toolchain rather than
    //    asserted: a completed entry is `out.obj` and one blob. Everything else
    //    the capture wrote has been folded away.
    let entry = only_entry(&root);
    let mut names: Vec<String> = std::fs::read_dir(&entry)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(
        names,
        vec![ENTRY_BLOB.to_string(), "out.obj".to_string()],
        "a real capture left files behind the fold did not absorb"
    );

    // 5. Poison the cached `.ex` — the stream a corrupt cache would silently
    //    move every census number with, while the differential stayed green.
    poison_cached_ex(&entry);

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

    // 6. …and the validator self-heals: the re-capture overwrote the entry, so
    //    the next validation is clean again.
    let cache = CaptureCache::new(root.clone(), &tc, None, 1).unwrap();
    let (_, outcome) = cache.capture(&tc, &src_arg, &flags, None, &fallback);
    assert_eq!(outcome, CacheOutcome::Validated);

    // 7. Poison the cached obj instead — the other half of the entry. Still a
    //    plain byte flip: the obj is a real file outside the blob, which is the
    //    fold's one exception and worth having a test that depends on it.
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

/// `replay` OVERWRITES the obj at the path it is given, and under a cache that
/// path is the cache entry itself.
///
/// This pins the premise that makes the restore in `differential_tail`
/// load-bearing: without it a diverging replay is left behind as the "cached
/// capture" and served as a hit next run, faking a `mismatch` on a byte-exact
/// TU. `gap/scan.rs` carries the same restore for its own replay.
///
/// Deliberately pins the *premise* and not the divergence: a non-diverging
/// replay rewrites byte-identical content (bar the COFF `TimeDateStamp`, a clock
/// reading), so a test of the leak itself would only fire when the two calls
/// straddle a second boundary — a race, not a check. If this test ever fails
/// because `replay` stopped overwriting, the restore became redundant; delete it
/// deliberately rather than leaving the reason a mystery.
#[test]
fn replay_overwrites_its_output_path() {
    let Some(tc) = Toolchain::locate() else {
        eprintln!("SKIP: toolchain absent");
        return;
    };
    if !tc.has_strace() {
        eprintln!("SKIP: strace absent");
        return;
    }
    if !tc.has_mingw() {
        eprintln!("SKIP: i686-w64-mingw32-gcc absent (needed for the c2host stub)");
        return;
    }
    let cpp = fixture();
    assert!(cpp.exists(), "tracked fixture missing: {}", cpp.display());
    let base = work("replay-overwrite");
    let root = base.join("cache");
    let fallback = base.join("work");
    std::fs::create_dir_all(&fallback).unwrap();
    let src_arg = c2_reference::to_wibo_path(&cpp.canonicalize().unwrap());
    let flags: Vec<String> = ["/Ox", "/GS-", "/c"].iter().map(|s| s.to_string()).collect();

    let cache = CaptureCache::new(root.clone(), &tc, None, 0).unwrap();
    let (captured, _) = cache.capture(&tc, &src_arg, &flags, None, &fallback);
    let captured = captured.expect("capture");
    let entry_obj = captured.ref_obj_path.clone();

    // A sentinel no compiler would emit. If `replay` leaves this in place it is
    // not writing to the path it was handed, and the restore guards nothing.
    std::fs::write(&entry_obj, b"SENTINEL-NOT-AN-OBJ").unwrap();
    tc.replay(&captured, &base.join("replay_il"), &entry_obj)
        .expect("replay");
    let after = std::fs::read(&entry_obj).unwrap();
    assert_ne!(
        after, b"SENTINEL-NOT-AN-OBJ",
        "replay did not overwrite the obj at the path it was given"
    );

    // And the restore is what puts the captured bytes back — the exact call
    // `differential_tail` makes before it can return on the mismatch path.
    std::fs::write(&entry_obj, captured.ref_obj.as_bytes()).unwrap();
    assert_eq!(
        std::fs::read(&entry_obj).unwrap(),
        captured.ref_obj.as_bytes(),
        "the entry did not round-trip the captured bytes"
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
