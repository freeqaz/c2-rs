//! `c2-harness::capture_cache` — **content-addressed cache of reference
//! captures** (roadmap #15).
//!
//! # What it is for
//!
//! The 878-TU gap scan runs `cl.exe` under wibo under strace once per TU, for
//! the sole purpose of getting back an IL bundle and c2's own obj. That is the
//! whole cost of the scan (~450 s of CPU, ~36 s wall at `--jobs 16`) and it is
//! *pure*: the same source bytes, the same flags and the same toolchain produce
//! the same capture. So cache it, and the measuring instrument stops being the
//! reason nobody re-measures.
//!
//! # The two hard requirements, and how each is met
//!
//! **(a) The staleness hazard is closed by construction, not by convention.**
//! The key is a hash over *inputs*, never over mtimes:
//!
//! | key component | why it is in there |
//! |---|---|
//! | source file **contents** | the obvious one; a rewritten TU is a different input |
//! | the source *argument* string | it is baked verbatim into `.gl` and `.debug$S` |
//! | the exact flag string | `/O1` vs `/Ox` is a different compiler |
//! | the compile `cwd` | relative includes and the baked paths resolve against it |
//! | **`cl.exe` + `c1xx.dll` + `c2.dll` contents** | the oracle itself |
//! | the wibo **version string** | the loader changed our results before ([`Toolchain::wibo_stale`]) |
//! | the workload tree's **git identity** | see below — this is the header closure |
//!
//! The interesting one is the last. A TU's capture depends on hundreds of
//! headers that the source hash cannot see, so hashing only the `.cpp` would
//! leave a header edit invisible — the exact shape of staleness bug this cache
//! is supposed to be immune to. The key therefore includes the workload tree's
//! `git rev-parse HEAD` **and** a digest of `git status --porcelain` plus the
//! contents of every path it lists, so any tracked edit anywhere in the tree —
//! header or not — changes every key. When the tree is not a checkout the token
//! is `no-git` and [`CaptureCache::header_closure_warning`] says so out loud;
//! the cache still works, but the report admits which guard is missing rather
//! than implying one.
//!
//! Hash collisions are closed the same way — by construction rather than by
//! arithmetic. The key is 128 bits (two independent FNV-1a-64 passes), which
//! over ~10^4 entries is a collision probability around 10^-31; but the entry
//! *also* stores the full key material verbatim in `key.bin` and a hit is only
//! served when those bytes compare equal. A collision therefore degrades to a
//! miss, not to a wrong answer, and the odds above are a curiosity rather than a
//! load-bearing claim.
//!
//! **(b) A bypass-and-compare validator, so a poisoned cache is detectable.**
//! `--validate-cache N` re-captures every Nth cache **hit** through the real
//! toolchain and byte-compares the fresh bundle, obj and c2 argv against what
//! the cache served. `--no-cache` bypasses the cache entirely. A disagreement is
//! reported per entry and is a hard failure signal for the scan — a cache that
//! is trusted without a sampling check is exactly the instrument failure this
//! rung exists to prevent.
//!
//! # Why the cache directory *is* the capture directory
//!
//! c2 bakes its `-Fo` path into the obj (`S_OBJNAME` in `.debug$S`), so a
//! capture is only reproducible at a **fixed output path**. Each entry is
//! therefore captured directly into `<cache>/<key>/`, and a hit hands back
//! `ref_obj_path = <cache>/<key>/out.obj` — the same path the bytes were made
//! at. That is what makes the validator a straight byte compare instead of a
//! path-normalizing one, and it is why entries are never renamed into place.
//! (The port is handed the same path via `S_OBJNAME`, so both sides of the
//! differential still agree; only the *absolute* path differs from an uncached
//! run, and it differs on both sides identically.)
//!
//! Captured IL is **never** committed: the default root is `<repo>/work/`, which
//! `.gitignore` covers, and the entries are `_CL_*` / `*.obj` besides.

use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use c2_il::{IlBundle, IL_SUFFIXES};
use c2_obj::{ObjDiff, ObjImage};
use c2_reference::{CapturedReference, Toolchain};

use crate::provenance::GitInfo;

/// On-disk format version. Bump on any layout/key change — old entries then key
/// differently and are simply never read again.
const CACHE_FORMAT: &str = "c2rs-capture-cache/v1";

/// FNV-1a 64, hand-rolled (the workspace is std-only by hard constraint).
fn fnv1a64(seed: u64, bytes: &[u8]) -> u64 {
    let mut h = seed;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01B3);
    }
    h
}

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;

/// 128-bit content digest: FNV-1a-64 forward, and a second FNV-1a-64 over the
/// reversed bytes seeded with the first. Two passes with different orders make
/// the pair far harder to collide jointly than either alone; the entry's stored
/// key material is what actually rules out a wrong hit (see the module docs).
pub fn digest128(bytes: &[u8]) -> String {
    let h1 = fnv1a64(FNV_OFFSET, bytes);
    let rev: Vec<u8> = bytes.iter().rev().copied().collect();
    let h2 = fnv1a64(h1 ^ 0x9E37_79B9_7F4A_7C15, &rev);
    format!("{h1:016x}{h2:016x}")
}

/// Hash a file's contents; `None` when it cannot be read.
fn file_digest(p: &Path) -> Option<String> {
    let bytes = std::fs::read(p).ok()?;
    Some(format!("{}:{}", bytes.len(), digest128(&bytes)))
}

/// Convert a `Z:\…` wibo path back to a host path so a fixture's `src_arg` can
/// be content-hashed. Relative arguments pass through and are resolved against
/// the compile `cwd`.
fn host_source_path(src_arg: &str, cwd: Option<&Path>) -> PathBuf {
    let s = src_arg.trim();
    if let Some(rest) = s.strip_prefix("Z:").or_else(|| s.strip_prefix("z:")) {
        return PathBuf::from(rest.replace('\\', "/"));
    }
    let p = PathBuf::from(s);
    match (p.is_absolute(), cwd) {
        (false, Some(dir)) => dir.join(p),
        _ => p,
    }
}

/// What the cache did for one TU. Reported in aggregate; deliberately **not**
/// recorded per JSONL row, so a cold scan's rows and a warm scan's rows stay
/// byte-comparable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CacheOutcome {
    /// Cache disabled (`--no-cache`), or this TU had no cacheable key.
    Bypassed,
    /// Served from disk.
    Hit,
    /// Captured for real and stored.
    Miss,
    /// Served from disk, then re-captured and byte-compared: identical.
    Validated,
    /// Served from disk, then re-captured and byte-compared: **different**.
    Poisoned,
}

/// Aggregate cache statistics for one scan.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CacheStats {
    pub hits: usize,
    pub misses: usize,
    pub bypassed: usize,
    pub validated: usize,
    pub poisoned: usize,
    /// Of the `validated`, how many agreed only after zeroing the COFF
    /// `TimeDateStamp` — i.e. the re-capture ran in a later second. Reported so
    /// the normalization stays visible rather than becoming a silent tolerance.
    pub timestamp_only: usize,
    /// One line per poisoned entry: `<src>: <what differed>`.
    pub poison_detail: Vec<String>,
}

/// The cache.
pub struct CaptureCache {
    root: PathBuf,
    /// Identity of everything that is not the source file: toolchain + tree.
    context: String,
    /// True iff the workload tree's git identity is part of `context` (the
    /// header-closure guard). False = the tree is not a checkout.
    header_closure: bool,
    validate_every: usize,
    seq: AtomicUsize,
    stats: Mutex<CacheStats>,
    /// One lock per in-flight key: two TUs with identical inputs must not
    /// capture into the same directory at once.
    locks: Mutex<HashMap<String, Arc<Mutex<()>>>>,
}

impl CaptureCache {
    /// Build a cache rooted at `root` for this toolchain and workload tree.
    ///
    /// `validate_every` > 0 re-captures every Nth **hit** and byte-compares it
    /// (0 = never). Computing the context runs `git` a couple of times and
    /// hashes three compiler DLLs — once per scan, not once per TU.
    pub fn new(
        root: PathBuf,
        tc: &Toolchain,
        workload_dir: Option<&Path>,
        validate_every: usize,
    ) -> io::Result<CaptureCache> {
        std::fs::create_dir_all(&root)?;
        let tree = workload_dir.map(GitInfo::probe).unwrap_or_else(GitInfo::unknown);
        let header_closure = tree.head != "unknown";
        let mut context = String::new();
        context.push_str(CACHE_FORMAT);
        context.push('\n');
        for (label, path) in [
            ("cl.exe", &tc.cl_exe),
            ("c1xx.dll", &tc.c1xx_dll),
            ("c2.dll", &tc.c2_dll),
        ] {
            context.push_str(&format!(
                "tool {label} {}\n",
                file_digest(path).unwrap_or_else(|| "absent".to_string())
            ));
        }
        context.push_str(&format!(
            "wibo {}\n",
            tc.wibo_version().unwrap_or_else(|| "unknown".to_string())
        ));
        // The header closure: HEAD plus every tracked modification, by content.
        context.push_str(&format!("tree {}\n", tree.token()));
        if let Some(dir) = workload_dir {
            context.push_str(&format!("tree-dirty {}\n", dirty_digest(dir)));
        }
        // WHERE the cache lives is part of its key, because a cached reference
        // obj EMBEDS this directory's absolute path — `c2` is invoked with `-Fo`
        // into the cache dir, so the path length lands in the obj bytes. Every
        // other component above describes the *inputs*; this one describes the
        // capture, and leaving it out is what let a copied cache serve bytes
        // captured under a different path. It presented as `mismatch` on six
        // unrelated TUs — an ALARM, the highest-priority signal in this project,
        // pointing at the port while the port was fine (ref 740 B vs port 768 B,
        // diverging at offset 8: exactly the path-length delta).
        //
        // With the root in the key a relocated cache MISSES and re-captures,
        // which is slow and correct, instead of hitting and lying. `--validate-cache`
        // does name this exact case ("c2 argv differs") but is off by default, so
        // the default path was the one that lied.
        //
        // Canonicalized so that two spellings of one directory are one key rather
        // than two; `create_dir_all` above guarantees it resolves.
        context.push_str(&format!(
            "cache-root {}\n",
            root.canonicalize().unwrap_or_else(|_| root.clone()).display()
        ));
        Ok(CaptureCache {
            root,
            context,
            header_closure,
            validate_every,
            seq: AtomicUsize::new(0),
            stats: Mutex::new(CacheStats::default()),
            locks: Mutex::new(HashMap::new()),
        })
    }

    /// The cache root.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The context digest, printed in the scan header so two scans can be
    /// compared on whether they even shared a toolchain.
    pub fn context_digest(&self) -> String {
        digest128(self.context.as_bytes())
    }

    /// A warning when the header-closure guard is unavailable (the workload is
    /// not a git checkout), so the report never *implies* a guard it lacks.
    pub fn header_closure_warning(&self) -> Option<String> {
        if self.header_closure {
            None
        } else {
            Some(
                "WARNING: the workload tree is not a git checkout, so the capture cache \
                 key cannot see header edits (only the .cpp's own bytes). Re-run with \
                 --no-cache, or --validate-cache N, before trusting a warm scan."
                    .to_string(),
            )
        }
    }

    /// Snapshot of the counters.
    pub fn stats(&self) -> CacheStats {
        self.stats.lock().unwrap().clone()
    }

    /// The full key material for one TU, or `None` when the source cannot be
    /// read — in which case the TU is captured normally and never cached
    /// (fail-open on caching, never on correctness).
    fn key_material(
        &self,
        src_arg: &str,
        flags: &[String],
        cwd: Option<&Path>,
    ) -> Option<Vec<u8>> {
        let src_path = host_source_path(src_arg, cwd);
        let src_digest = file_digest(&src_path)?;
        let mut m = Vec::with_capacity(self.context.len() + 256);
        m.extend_from_slice(self.context.as_bytes());
        m.extend_from_slice(b"src-arg\x00");
        m.extend_from_slice(src_arg.as_bytes());
        m.extend_from_slice(b"\x00src-bytes\x00");
        m.extend_from_slice(src_digest.as_bytes());
        m.extend_from_slice(b"\x00cwd\x00");
        m.extend_from_slice(
            cwd.map(|d| d.display().to_string())
                .unwrap_or_default()
                .as_bytes(),
        );
        m.extend_from_slice(b"\x00flags\x00");
        for f in flags {
            m.extend_from_slice(f.as_bytes());
            m.push(0x1f);
        }
        m.push(0);
        Some(m)
    }

    /// Get a capture for one TU: from disk when the key hits, otherwise through
    /// the real toolchain (and stored). `fallback_work` is used only when the
    /// TU is not cacheable.
    ///
    /// The returned [`CapturedReference`] is *exactly* what
    /// `Toolchain::capture_reference_with` would have returned, at the same
    /// output path — see the module docs on why the cache dir is the capture
    /// dir.
    pub fn capture(
        &self,
        tc: &Toolchain,
        src_arg: &str,
        flags: &[String],
        cwd: Option<&Path>,
        fallback_work: &Path,
    ) -> (io::Result<CapturedReference>, CacheOutcome) {
        let Some(material) = self.key_material(src_arg, flags, cwd) else {
            self.stats.lock().unwrap().bypassed += 1;
            return (
                tc.capture_reference_with(src_arg, fallback_work, flags, cwd),
                CacheOutcome::Bypassed,
            );
        };
        let key = digest128(&material);
        let dir = self.root.join(&key);

        // Serialize same-key work (duplicate entries in a source list).
        let lock = {
            let mut locks = self.locks.lock().unwrap();
            locks.entry(key.clone()).or_default().clone()
        };
        let _guard = lock.lock().unwrap();

        match read_entry(&dir, &material) {
            Some(hit) => {
                let n = self.seq.fetch_add(1, Ordering::Relaxed) + 1;
                let validate = self.validate_every > 0 && n % self.validate_every == 0;
                if !validate {
                    self.stats.lock().unwrap().hits += 1;
                    return (Ok(hit), CacheOutcome::Hit);
                }
                // Bypass-and-compare: re-capture *in place* (same directory, so
                // the same `-Fo` is baked in) and compare against what the cache
                // just served. Re-capturing in place also self-heals the entry.
                match tc.capture_reference_with(src_arg, &dir, flags, cwd) {
                    Ok(fresh) => {
                        let diff = compare_captures(&hit, &fresh);
                        let _ = write_entry(&dir, &material, &fresh);
                        let mut st = self.stats.lock().unwrap();
                        st.hits += 1;
                        match diff {
                            CaptureDiff::Identical => {
                                st.validated += 1;
                                drop(st);
                                (Ok(fresh), CacheOutcome::Validated)
                            }
                            CaptureDiff::TimestampOnly => {
                                st.validated += 1;
                                st.timestamp_only += 1;
                                drop(st);
                                (Ok(fresh), CacheOutcome::Validated)
                            }
                            CaptureDiff::Differs(what) => {
                                st.poisoned += 1;
                                st.poison_detail.push(format!("{src_arg}: {what}"));
                                drop(st);
                                (Ok(fresh), CacheOutcome::Poisoned)
                            }
                        }
                    }
                    Err(e) => {
                        // The re-capture failed; the cached bytes are still what
                        // we have, and a validator that cannot run says so.
                        let mut st = self.stats.lock().unwrap();
                        st.hits += 1;
                        st.poisoned += 1;
                        st.poison_detail
                            .push(format!("{src_arg}: re-capture failed: {e}"));
                        drop(st);
                        (Ok(hit), CacheOutcome::Poisoned)
                    }
                }
            }
            None => {
                let out = tc.capture_reference_with(src_arg, &dir, flags, cwd);
                match &out {
                    Ok(cap) => {
                        let _ = write_entry(&dir, &material, cap);
                        self.stats.lock().unwrap().misses += 1;
                    }
                    Err(_) => {
                        // A capture-fail is not cached: it is usually an
                        // environment fact (a missing include root), and a
                        // cached failure would be indistinguishable from a real
                        // one on the next run.
                        let _ = std::fs::remove_dir_all(&dir);
                        self.stats.lock().unwrap().misses += 1;
                    }
                }
                (out, CacheOutcome::Miss)
            }
        }
    }
}

impl Default for CacheOutcome {
    fn default() -> Self {
        CacheOutcome::Bypassed
    }
}

/// Digest of every tracked modification in `dir`, **by content** — the part of
/// the key that makes a header edit invisible to no one. `git status
/// --porcelain -uno` names the changed paths; each named path's bytes go in.
/// Degrades to `no-git` when git cannot answer.
fn dirty_digest(dir: &Path) -> String {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["status", "--porcelain", "-uno"])
        .output();
    let Ok(out) = out else {
        return "no-git".to_string();
    };
    if !out.status.success() {
        return "no-git".to_string();
    }
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    let mut buf = text.clone().into_bytes();
    for line in text.lines() {
        // `XY path` (and `XY old -> new` for renames — take the last field).
        let path = line.get(3..).unwrap_or("").rsplit(" -> ").next().unwrap_or("");
        if path.is_empty() {
            continue;
        }
        buf.extend_from_slice(path.as_bytes());
        buf.push(0);
        if let Ok(bytes) = std::fs::read(dir.join(path.trim_matches('"'))) {
            buf.extend_from_slice(digest128(&bytes).as_bytes());
        } else {
            buf.extend_from_slice(b"unreadable");
        }
        buf.push(0);
    }
    digest128(&buf)
}

/// Read a complete entry from `dir`, requiring its stored key material to equal
/// `material` byte-for-byte (so a hash collision is a miss, not a wrong hit).
fn read_entry(dir: &Path, material: &[u8]) -> Option<CapturedReference> {
    let stored = std::fs::read(dir.join("key.bin")).ok()?;
    if stored != material {
        return None;
    }
    // `meta.txt` is written last, so its presence is the completion marker.
    let meta = std::fs::read_to_string(dir.join("meta.txt")).ok()?;
    let mut base_name = String::new();
    let mut c2_argv: Vec<String> = Vec::new();
    for line in meta.lines() {
        if let Some(v) = line.strip_prefix("base ") {
            base_name = v.to_string();
        } else if let Some(v) = line.strip_prefix("arg ") {
            c2_argv.push(v.to_string());
        }
    }
    if base_name.is_empty() || c2_argv.is_empty() {
        return None;
    }
    let ref_obj_path = dir.join("out.obj");
    let obj = std::fs::read(&ref_obj_path).ok()?;
    if obj.is_empty() {
        return None;
    }
    let bundle = IlBundle::load_from_dir(dir, &base_name).ok()?;
    if bundle.ex().map(<[u8]>::is_empty).unwrap_or(true) {
        return None;
    }
    Some(CapturedReference {
        bundle,
        base_name,
        c2_argv,
        ref_obj: ObjImage::new(obj),
        ref_obj_path,
    })
}

/// Complete an entry: the bundle and `out.obj` are already on disk (the capture
/// wrote them there), so this only records the key material and the metadata —
/// `meta.txt` last, as the completion marker.
fn write_entry(dir: &Path, material: &[u8], cap: &CapturedReference) -> io::Result<()> {
    std::fs::create_dir_all(dir)?;
    std::fs::write(dir.join("key.bin"), material)?;
    let mut meta = String::new();
    meta.push_str(CACHE_FORMAT);
    meta.push('\n');
    meta.push_str(&format!("base {}\n", cap.base_name));
    for a in &cap.c2_argv {
        meta.push_str(&format!("arg {a}\n"));
    }
    std::fs::write(dir.join("meta.txt"), meta)
}

/// Byte-compare two captures. `None` = identical in every field the harness
/// consumes; `Some(what)` names the first field that differs.
///
/// Deliberately field-by-field rather than "the obj matched": a poisoned `.gl`
/// with an intact obj would silently change every census number while the
/// differential stayed green, which is the failure mode a validator exists for.
/// The verdict of a bypass-and-compare check.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CaptureDiff {
    /// Every byte the harness consumes is equal.
    Identical,
    /// Equal except the COFF `TimeDateStamp` (obj bytes 4..8) — a clock reading,
    /// not a function of the inputs, and the one field the project's correctness
    /// criterion explicitly zeroes. Not poison; counted separately so it stays
    /// visible instead of becoming an unexplained tolerance.
    TimestampOnly,
    /// A real difference, named.
    Differs(String),
}

/// # The two things a capture is *not* reproducible in
///
/// Both were found by the validator failing on its own control case, which is
/// the outcome a control case is for.
///
/// 1. **The bundle nonce.** `cl.exe` names the IL bundle `_CL_<hex>` from a
///    per-invocation nonce, so two captures of the same TU with the same flags
///    into the same directory differ in their file names and in the `-il` value
///    of the echoed c2 argv.
/// 2. **The COFF `TimeDateStamp`.** The reference obj's bytes 4..8 are wall
///    clock. On the fixture-sized control this hid behind one-second
///    granularity — two captures a few hundred milliseconds apart agreed — and
///    it only showed up when the validator ran across the 878-TU workload, where
///    8 of 8 sampled entries "differed at offset 4". That is the *whole* reason
///    the project's own criterion is "byte-exact with the timestamp zeroed", so
///    the check applies the same normalization and reports the case as
///    [`CaptureDiff::TimestampOnly`] rather than pretending it did not happen.
///
/// Everything else is compared byte-for-byte: the five bundle streams raw (so a
/// nonce leaking into `.ex` would still be caught), the rest of the argv, and
/// the obj outside those four bytes.
pub fn compare_captures(a: &CapturedReference, b: &CapturedReference) -> CaptureDiff {
    for suffix in IL_SUFFIXES {
        let (x, y) = (a.bundle.get(suffix), b.bundle.get(suffix));
        match (x, y) {
            (Some(x), Some(y)) => {
                if x != y {
                    let off = x
                        .iter()
                        .zip(y.iter())
                        .position(|(p, q)| p != q)
                        .unwrap_or_else(|| x.len().min(y.len()));
                    return CaptureDiff::Differs(format!(
                        ".{suffix} differs at offset {off} (cached {} B, fresh {} B)",
                        x.len(),
                        y.len()
                    ));
                }
            }
            (None, None) => {}
            _ => {
                return CaptureDiff::Differs(format!(".{suffix} present on one side only"))
            }
        }
    }
    let (aa, bb) = (argv_without_il(&a.c2_argv), argv_without_il(&b.c2_argv));
    if aa != bb {
        return CaptureDiff::Differs(format!("c2 argv differs ({aa:?} vs {bb:?})"));
    }
    // The obj under the project's own criterion: identical with bytes 4..8
    // (`TimeDateStamp`) zeroed. `ObjDiff` is the same normalization the
    // differential itself grades with, so the validator cannot be stricter or
    // looser than the thing it protects.
    match ObjImage::diff(&a.ref_obj, &b.ref_obj) {
        ObjDiff::Identical => {
            if a.ref_obj.as_bytes() == b.ref_obj.as_bytes() {
                CaptureDiff::Identical
            } else {
                CaptureDiff::TimestampOnly
            }
        }
        ObjDiff::Differs {
            first_offset,
            a_len,
            b_len,
        } => CaptureDiff::Differs(format!(
            "reference obj differs at normalized offset {first_offset} \
             (cached {a_len} B, fresh {b_len} B)"
        )),
    }
}

/// The c2 argv with the `-il <base>` value dropped — that value carries the
/// per-capture `_CL_<hex>` nonce and the replay path overwrites it anyway.
fn argv_without_il(argv: &[String]) -> Vec<&str> {
    let mut out = Vec::with_capacity(argv.len());
    let mut i = 0;
    while i < argv.len() {
        if argv[i] == "-il" {
            i += 2;
            continue;
        }
        out.push(argv[i].as_str());
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn argv_normalization_drops_only_the_il_nonce() {
        let a: Vec<String> = ["-il", "/t/_CL_aaaa", "-typedil", "-f", "x.cpp"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let b: Vec<String> = ["-il", "/t/_CL_bbbb", "-typedil", "-f", "x.cpp"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(argv_without_il(&a), argv_without_il(&b));
        // …and nothing else: a changed -f is still a difference.
        let c: Vec<String> = ["-il", "/t/_CL_bbbb", "-typedil", "-f", "y.cpp"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_ne!(argv_without_il(&a), argv_without_il(&c));
    }

    #[test]
    fn digest_is_deterministic_and_input_sensitive() {
        assert_eq!(digest128(b"abc"), digest128(b"abc"));
        assert_ne!(digest128(b"abc"), digest128(b"abd"));
        // Order matters (the reverse pass is not the point — the forward one is).
        assert_ne!(digest128(b"ab"), digest128(b"ba"));
        assert_eq!(digest128(b"abc").len(), 32);
    }

    #[test]
    fn digest_separates_the_empty_and_zero_inputs() {
        assert_ne!(digest128(b""), digest128(b"\0"));
        assert_ne!(digest128(b"\0"), digest128(b"\0\0"));
    }

    #[test]
    fn wibo_source_paths_convert_back_to_host_paths() {
        assert_eq!(
            host_source_path(r"Z:\proj\x\y.cpp", None),
            PathBuf::from("/proj/x/y.cpp")
        );
        assert_eq!(
            host_source_path("src/a.cpp", Some(Path::new("/w"))),
            PathBuf::from("/w/src/a.cpp")
        );
        assert_eq!(
            host_source_path("/abs/a.cpp", Some(Path::new("/w"))),
            PathBuf::from("/abs/a.cpp")
        );
    }

    /// The key must move when *any* of its documented components moves. Written
    /// as a table so a component added later without a case here is obvious.
    #[test]
    fn key_material_separates_every_documented_component() {
        let dir = std::env::temp_dir().join(format!("c2rs-cachekey-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let src = dir.join("a.cpp");
        std::fs::write(&src, b"int f(){return 1;}").unwrap();

        let mk = |context: &str| CaptureCache {
            root: dir.clone(),
            context: context.to_string(),
            header_closure: true,
            validate_every: 0,
            seq: AtomicUsize::new(0),
            stats: Mutex::new(CacheStats::default()),
            locks: Mutex::new(HashMap::new()),
        };
        let c = mk("ctx-A");
        let flags: Vec<String> = vec!["/O1".into(), "/c".into()];
        let base = c.key_material("a.cpp", &flags, Some(&dir)).unwrap();

        // Same everything → same key.
        assert_eq!(base, c.key_material("a.cpp", &flags, Some(&dir)).unwrap());

        // Different flags.
        let other_flags: Vec<String> = vec!["/Ox".into(), "/c".into()];
        assert_ne!(base, c.key_material("a.cpp", &other_flags, Some(&dir)).unwrap());
        // Flag *joining* must not be ambiguous: ["/O1","/c"] != ["/O1/c"].
        let joined: Vec<String> = vec!["/O1/c".into()];
        assert_ne!(base, c.key_material("a.cpp", &joined, Some(&dir)).unwrap());

        // Different context (toolchain / wibo version / tree identity).
        assert_ne!(base, mk("ctx-B").key_material("a.cpp", &flags, Some(&dir)).unwrap());

        // Different cwd, *identical* source bytes: relative includes and the
        // paths baked into `.gl`/`.debug$S` resolve against it, so it is part of
        // the input even when nothing else moves.
        let dir2 = dir.join("sub");
        std::fs::create_dir_all(&dir2).unwrap();
        std::fs::write(dir2.join("a.cpp"), b"int f(){return 1;}").unwrap();
        assert_ne!(base, c.key_material("a.cpp", &flags, Some(&dir2)).unwrap());

        // A source that cannot be read is not cacheable at all — the cache
        // fails open rather than keying over a missing input.
        assert!(c.key_material("a.cpp", &flags, None).is_none());

        // Different source *contents* at the same path — the mtime-free part.
        std::fs::write(&src, b"int f(){return 2;}").unwrap();
        assert_ne!(base, c.key_material("a.cpp", &flags, Some(&dir)).unwrap());

        // Unreadable source → not cacheable (rather than a key over nothing).
        assert!(c.key_material("missing.cpp", &flags, Some(&dir)).is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Relocating the cache must MISS, not hit.
    ///
    /// The table test above builds `CaptureCache` with a hand-written `context`,
    /// so it cannot see this component at all — the root enters the key through
    /// `new()`. That is exactly how the gap survived: every documented component
    /// had a case, and the one that was undocumented had nowhere to fail.
    ///
    /// Toolchain-absent is a clean SKIP, never a failure (CLAUDE.md).
    #[test]
    fn the_cache_root_is_part_of_the_key() {
        let Some(tc) = Toolchain::locate() else {
            eprintln!("SKIP: toolchain absent");
            return;
        };
        let base = std::env::temp_dir().join(format!("c2rs-cacheroot-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let (a, b) = (base.join("A"), base.join("B"));
        let src = base.join("a.cpp");
        std::fs::create_dir_all(&base).unwrap();
        std::fs::write(&src, b"int f(){return 1;}").unwrap();

        // Same toolchain, same workload tree, same source, same flags — the only
        // thing that moved is where the capture would be written.
        let ca = CaptureCache::new(a.clone(), &tc, Some(&base), 0).unwrap();
        let cb = CaptureCache::new(b.clone(), &tc, Some(&base), 0).unwrap();
        let flags: Vec<String> = vec!["/O1".into(), "/c".into()];
        let ka = ca.key_material("a.cpp", &flags, Some(&base)).unwrap();
        let kb = cb.key_material("a.cpp", &flags, Some(&base)).unwrap();
        assert_ne!(
            ka, kb,
            "a cache at a different root must miss: the reference obj embeds the \
             capture directory's absolute path, so bytes captured under one root \
             are not the bytes c2 would emit under another"
        );

        // …and a cache re-opened at the same root still hits.
        let ca2 = CaptureCache::new(a, &tc, Some(&base), 0).unwrap();
        assert_eq!(ka, ca2.key_material("a.cpp", &flags, Some(&base)).unwrap());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn a_hash_collision_would_read_as_a_miss() {
        // read_entry demands the stored material equal the requested material,
        // so an entry written under other inputs is never served.
        let dir = std::env::temp_dir().join(format!("c2rs-cachecol-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("key.bin"), b"material-A").unwrap();
        std::fs::write(dir.join("meta.txt"), "base _CL_1\narg -il\n").unwrap();
        assert!(read_entry(&dir, b"material-B").is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_incomplete_entry_reads_as_a_miss() {
        // meta.txt is the completion marker: key material alone is not an entry.
        let dir = std::env::temp_dir().join(format!("c2rs-cacheinc-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("key.bin"), b"material-A").unwrap();
        assert!(read_entry(&dir, b"material-A").is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn compare_captures_names_the_field_that_differs() {
        let mk = |ex: &[u8], obj: &[u8], argv: Vec<String>| {
            let mut b = IlBundle::new("_CL_test");
            b.set("ex", ex.to_vec());
            CapturedReference {
                bundle: b,
                base_name: "_CL_test".to_string(),
                c2_argv: argv,
                ref_obj: ObjImage::new(obj.to_vec()),
                ref_obj_path: PathBuf::from("/tmp/out.obj"),
            }
        };
        let named = |d: CaptureDiff| match d {
            CaptureDiff::Differs(s) => s,
            other => panic!("expected a named difference, got {other:?}"),
        };
        // 12 bytes so the obj is long enough to carry a TimeDateStamp at 4..8;
        // the differing byte sits outside it.
        let a = mk(b"AAAA", b"OOOOOOOOOOOO", vec!["-il".into()]);
        assert_eq!(compare_captures(&a, &a), CaptureDiff::Identical);

        let ex_differs = mk(b"AABA", b"OOOOOOOOOOOO", vec!["-il".into()]);
        assert!(named(compare_captures(&a, &ex_differs)).starts_with(".ex differs at offset 2"));

        let obj_differs = mk(b"AAAA", b"OOOOOOOOOOOX", vec!["-il".into()]);
        assert!(named(compare_captures(&a, &obj_differs))
            .starts_with("reference obj differs at normalized offset 11"));

        // The COFF TimeDateStamp (bytes 4..8) alone is NOT poison — it is a
        // clock reading, and it is the one field the project's own criterion
        // zeroes. It is still reported as its own verdict, never as "identical".
        let stamped = mk(b"AAAA", b"OOOO\x01\x02\x03\x04OOOO", vec!["-il".into()]);
        assert_eq!(
            compare_captures(&a, &stamped),
            CaptureDiff::TimestampOnly
        );

        // The `-il` value is the per-capture `_CL_<hex>` nonce and is normalized
        // away; everything else in the argv is still compared.
        let nonce_only = mk(
            b"AAAA",
            b"OOOOOOOOOOOO",
            vec!["-il".into(), "/t/_CL_ffff".into()],
        );
        let a_nonce = mk(
            b"AAAA",
            b"OOOOOOOOOOOO",
            vec!["-il".into(), "/t/_CL_0000".into()],
        );
        assert_eq!(compare_captures(&a_nonce, &nonce_only), CaptureDiff::Identical);
        let real_argv_change = mk(
            b"AAAA",
            b"OOOOOOOOOOOO",
            vec!["-il".into(), "/t/_CL_ffff".into(), "-Ox".into()],
        );
        assert!(named(compare_captures(&a_nonce, &real_argv_change)).starts_with("c2 argv differs"));
    }
}
