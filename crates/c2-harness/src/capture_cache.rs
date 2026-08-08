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
//! **The root is absolutised the moment it enters the cache, and that is a
//! correctness property, not tidiness** (board #1388). It used to be
//! *canonicalized into the key* but stored *as spelled*, and the two disagreed
//! on exactly one path: a HIT. `capture_reference_with` calls `absolute()` on
//! the directory it is handed, so a MISS bakes — and returns — an absolute
//! `-Fo` whatever the spelling; but `read_entry` served
//! `self.root.join(key)/out.obj` verbatim, so under `--cache <relative dir>` the
//! port was handed a *relative* `S_OBJNAME` (`to_wibo_path` passes a relative
//! path through unchanged, without even the `Z:` prefix) while the cached
//! reference obj carried the absolute one. The differential then compared two
//! objs that differ only in the path each records about itself and reported
//! **`mismatch`** on a byte-exact TU — a false ALARM, in the instrument every
//! ladder on this project runs. It could not fire on a first run, only on the
//! second, which is the one a reader trusts more.
//!
//! Because the key already carried the *canonicalized* root, the two spellings
//! addressed **one** entry, so the wrong answer was reproducible and stable
//! rather than intermittent. Absolutising `root` itself makes the served path
//! and the keyed path the same string by construction; the key is unchanged, so
//! entries written before this fix stay valid.
//!
//! **And a served entry is checked against its own provenance.** `meta.txt`
//! records the absolute path the capture was actually made at
//! ([`CAPTURE_PATH_KEY`]), and an entry whose recorded path is not the path it
//! is being served from is refused — counted as [`CacheStats::foreign`], named
//! in the report, and re-captured. That is the standing rule for this cache: a
//! stale or foreign entry must be a MISS or a loud refusal, never a silent wrong
//! verdict. It is expected to read **0** forever; a non-zero is either a moved
//! cache or a regression in the absolutisation above, and both want saying out
//! loud rather than paying for silently in re-captures.
//!
//! Captured IL is **never** committed: the default root is `<repo>/work/`, which
//! `.gitignore` covers, and the entries are `_CL_*` / `*.obj` besides.

use std::io;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use c2_il::{IlBundle, IL_SUFFIXES};
use c2_obj::{ObjDiff, ObjImage};
use c2_reference::{CapturedReference, Toolchain};

use crate::provenance::GitInfo;

/// On-disk format version. Bump on any layout/key change — old entries then key
/// differently and are simply never read again.
const CACHE_FORMAT: &str = "c2rs-capture-cache/v1";

/// `meta.txt` line recording the **absolute path this entry's `out.obj` was
/// captured at**. Written since board #1388; absent on older entries, which are
/// still served (the key already pins the cache root, so their path is
/// derivable — the line exists to make that derivation *checkable* rather than
/// assumed, and to keep it checkable if the derivation is ever changed again).
///
/// Deliberately NOT part of the key material: adding it there would cold-start
/// every existing entry — ~450 s of CPU per scan, paid by every concurrent lane
/// — to re-derive bytes that are already correct.
pub const CAPTURE_PATH_KEY: &str = "objpath ";

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
    /// Entries REFUSED because their recorded capture path
    /// ([`CAPTURE_PATH_KEY`]) is not the path they would be served from. The
    /// obj embeds its own `-Fo`, so such bytes are not the bytes c2 would emit
    /// here; they are re-captured rather than served. **Expected to be 0** —
    /// see the module docs on board #1388.
    pub foreign: usize,
    /// One line per refused entry: `<src>: <recorded> != <serving>`.
    pub foreign_detail: Vec<String>,
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
}

/// Where per-key lockfiles live, relative to the cache root. A subdirectory
/// rather than `<root>/<key>.lock`, so that the root keeps holding nothing but
/// 32-hex entry directories (the GC and every `ls | wc -l` census depend on
/// that) and so that the *entry* directory keeps holding nothing but the
/// capture — `capture_reference_with` sweeps `_CL_*` out of its work dir and
/// points `TMP`/`TEMP` at it, and this seam is not the place to find out
/// which stray file some future revision decides to read.
/// It is `pub` so that the one invariant it breaks — "every child of the cache
/// root is a 32-hex entry" — has a single name that consumers can test against
/// instead of each hard-coding a string. Any GC over the root must skip it: the
/// files inside are live cross-process locks, and deleting one on age grounds
/// silently un-guards a key.
pub const LOCK_DIR: &str = ".locks";

/// Give up waiting and proceed unguarded. Nothing downstream blocks on the
/// lock, so the failure mode is today's behaviour, not a wedged scan.
const LOCK_WAIT_MAX: Duration = Duration::from_secs(600);

/// Break a lock whose holder is gone. A capture is one `cl.exe` invocation
/// (~1–2 s); 30 minutes is ~1000× headroom, and without it a single SIGKILL
/// would poison one key of the cache permanently.
const LOCK_STALE_AFTER: Duration = Duration::from_secs(1800);

/// A cross-**process** mutex for one cache key, held for the length of one
/// `capture()`.
///
/// The lock this replaces was `Mutex<HashMap<String, Arc<Mutex<()>>>>` — an
/// in-process structure guarding a *filesystem* resource. That was sound only
/// while no two processes could compute the same key, which was true only by
/// accident: every lane's cache root differed, and `cache-root` is in the key.
/// It is not true of `scripts/gate.sh --jobs N`, which runs N `c2rs gap`
/// processes against one root, and it stops being true everywhere the moment
/// lanes share a root via `C2RS_GAP_CACHE`. Two processes on one key both
/// capture into `<root>/<key>/`, interleaving two `cl.exe` invocations' output
/// files — a torn `out.obj` read back as a hit is a *false* mismatch, which is
/// an ALARM pointing at the port while the port is fine.
///
/// `create_new` is `O_CREAT|O_EXCL`, atomic on every filesystem this runs on,
/// and it guards threads and processes alike — so the `HashMap` is not merely
/// replaced, it is subsumed.
struct KeyLock {
    path: PathBuf,
}

impl KeyLock {
    /// Acquire, or return `None` to proceed unguarded (fail-open).
    ///
    /// Caching is an optimisation and correctness is not: every error path here
    /// degrades to exactly what the code did before this type existed.
    fn acquire(root: &Path, key: &str) -> Option<KeyLock> {
        let dir = root.join(LOCK_DIR);
        std::fs::create_dir_all(&dir).ok()?;
        let path = dir.join(key);
        let deadline = Instant::now() + LOCK_WAIT_MAX;
        loop {
            match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(mut f) => {
                    // Owner's pid, so a wedged lock names its holder.
                    let _ = writeln!(f, "{}", std::process::id());
                    return Some(KeyLock { path });
                }
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                    if Self::is_stale(&path) {
                        let _ = std::fs::remove_file(&path);
                        continue;
                    }
                    if Instant::now() >= deadline {
                        return None;
                    }
                    std::thread::sleep(Duration::from_millis(20));
                }
                Err(_) => return None,
            }
        }
    }

    fn is_stale(path: &Path) -> bool {
        std::fs::metadata(path)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.elapsed().ok())
            .is_some_and(|age| age > LOCK_STALE_AFTER)
    }
}

impl Drop for KeyLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
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
        // ABSOLUTISE ONCE, HERE, and use the same value for the key and for
        // every path served out of this cache. Board #1388: the key line below
        // used to canonicalize while `self.root` kept the caller's spelling, so
        // `--cache <relative dir>` HIT the entry an absolute run had written and
        // then handed the port a *relative* `S_OBJNAME` — a false `mismatch` on
        // a byte-exact TU. One binding, so the two cannot drift apart again.
        //
        // `create_dir_all` above guarantees the path resolves; the fallback is
        // for the pathological case (a race deleting it) and keeps the cache
        // fail-open rather than turning an unreadable directory into a panic.
        let root = root.canonicalize().unwrap_or(root);
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
        // than two. `root` is ALREADY the canonical form (absolutised at the top
        // of this function), so this line is byte-identical to what it produced
        // before board #1388's fix and no existing entry is invalidated.
        context.push_str(&format!("cache-root {}\n", root.display()));
        Ok(CaptureCache {
            root,
            context,
            header_closure,
            validate_every,
            seq: AtomicUsize::new(0),
            stats: Mutex::new(CacheStats::default()),
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
        // Canonicalized, for the same reason `cache-root` is (see `new`), and it
        // is the *stronger* of the two cases: a relative `--cwd` resolves against
        // the c2rs process's own working directory, so the identical string
        // `../dc3-decomp` names a different directory in every worktree. Keying
        // over the raw spelling therefore aliases two different inputs onto one
        // key — harmless only for as long as every lane has its own cache root,
        // which is exactly the arrangement `C2RS_GAP_CACHE` removes. Canonicalizing
        // also folds the four observed spellings of one directory
        // (`/…/dc3-decomp`, `../dc3-decomp`, `/…/c2-rs/../dc3-decomp`,
        // `../../../../dc3-decomp`) into one generation instead of four.
        //
        // Sound only because the cwd's *spelling* does not reach the obj: wibo
        // hands cl.exe the resolved directory, the source argument is keyed
        // verbatim on its own line above, and `-Fo` is the (already canonical)
        // cache root. `two_spellings_of_one_cwd_capture_identical_bytes` holds
        // that claim to the real toolchain rather than to this comment.
        //
        // Falls back to the raw spelling when the path does not resolve, so an
        // unreadable cwd is a cache miss and never a panic.
        m.extend_from_slice(b"\x00cwd\x00");
        m.extend_from_slice(
            cwd.map(|d| {
                d.canonicalize()
                    .unwrap_or_else(|_| d.to_path_buf())
                    .display()
                    .to_string()
            })
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

        // Serialize same-key work — duplicate entries in one source list, and
        // (the case the in-process lock could not see) two `c2rs` processes
        // sharing a cache root. Fail-open: `None` means proceed unguarded.
        let _guard = KeyLock::acquire(&self.root, &key);

        // A refused entry is recorded BEFORE it is re-captured, so the report
        // says "this cache had entries it would not serve" rather than quietly
        // paying for them in misses. Expected to be 0; see the module docs.
        let entry = read_entry(&dir, &material);
        if let EntryRead::Foreign(what) = &entry {
            let mut st = self.stats.lock().unwrap();
            st.foreign += 1;
            st.foreign_detail.push(format!("{src_arg}: {what}"));
        }
        match entry {
            EntryRead::Hit(hit) => {
                let hit = *hit;
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
            EntryRead::Miss | EntryRead::Foreign(_) => {
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

/// What a lookup at one entry directory produced.
///
/// Three outcomes, not two, because "there is nothing here" and "there is
/// something here that must not be served" want different treatment: the first
/// is the ordinary cold path, the second is a fact about the cache that the
/// report has to carry. Both re-capture; only one of them says so.
#[derive(Debug)]
pub enum EntryRead {
    /// Nothing usable: absent, incomplete, or the stored key material differs
    /// (a hash collision, which is therefore a miss and never a wrong hit).
    Miss,
    /// An entry whose recorded capture path is not the path it is being served
    /// from. **Never served.** The reference obj embeds its own `-Fo` path
    /// (`S_OBJNAME`), so its bytes are not the bytes c2 would emit here, and
    /// serving them fakes a `mismatch` on a byte-exact TU — board #1388.
    Foreign(String),
    /// A complete entry, served at the path it was captured at.
    Hit(Box<CapturedReference>),
}

/// Read a complete entry from `dir`, requiring its stored key material to equal
/// `material` byte-for-byte (so a hash collision is a miss, not a wrong hit) and
/// its recorded capture path to be the path it is being served from.
fn read_entry(dir: &Path, material: &[u8]) -> EntryRead {
    let Ok(stored) = std::fs::read(dir.join("key.bin")) else {
        return EntryRead::Miss;
    };
    if stored != material {
        return EntryRead::Miss;
    }
    // `meta.txt` is written last, so its presence is the completion marker.
    let Ok(meta) = std::fs::read_to_string(dir.join("meta.txt")) else {
        return EntryRead::Miss;
    };
    let mut base_name = String::new();
    let mut c2_argv: Vec<String> = Vec::new();
    let mut recorded_path: Option<&str> = None;
    for line in meta.lines() {
        if let Some(v) = line.strip_prefix("base ") {
            base_name = v.to_string();
        } else if let Some(v) = line.strip_prefix("arg ") {
            c2_argv.push(v.to_string());
        } else if let Some(v) = line.strip_prefix(CAPTURE_PATH_KEY) {
            recorded_path = Some(v);
        }
    }
    if base_name.is_empty() || c2_argv.is_empty() {
        return EntryRead::Miss;
    }
    let ref_obj_path = dir.join("out.obj");
    // The provenance check. Absent on entries written before board #1388 — those
    // are served, because the cache root is in the key and the path is therefore
    // derivable; the line exists so the derivation is CHECKED rather than
    // assumed for everything written from now on.
    if let Some(recorded) = recorded_path {
        if Path::new(recorded) != ref_obj_path {
            return EntryRead::Foreign(format!(
                "entry records its capture at {recorded} but is being served from {} \
                 — the obj embeds its own -Fo path, so those are not the bytes c2 \
                 would emit here",
                ref_obj_path.display()
            ));
        }
    }
    let Ok(obj) = std::fs::read(&ref_obj_path) else {
        return EntryRead::Miss;
    };
    if obj.is_empty() {
        return EntryRead::Miss;
    }
    let Ok(bundle) = IlBundle::load_from_dir(dir, &base_name) else {
        return EntryRead::Miss;
    };
    if bundle.ex().map(<[u8]>::is_empty).unwrap_or(true) {
        return EntryRead::Miss;
    }
    EntryRead::Hit(Box::new(CapturedReference {
        bundle,
        base_name,
        c2_argv,
        ref_obj: ObjImage::new(obj),
        ref_obj_path,
    }))
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
    // The provenance line: where these bytes were actually made. `read_entry`
    // refuses to serve them from anywhere else (board #1388).
    meta.push_str(&format!(
        "{CAPTURE_PATH_KEY}{}\n",
        cap.ref_obj_path.display()
    ));
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

    /// Canonicalizing `cwd` must fold *spellings* without folding *directories*.
    ///
    /// The second half is the one that matters: before canonicalization a
    /// relative `--cwd` was keyed by its raw string, so two lanes passing
    /// `../dc3-decomp` from different worktrees produced the same key material
    /// for two genuinely different directories. That aliasing was invisible only
    /// because `cache-root` differed per lane and separated them downstream —
    /// which is precisely the separation a shared `C2RS_GAP_CACHE` removes.
    #[test]
    fn canonicalizing_the_cwd_folds_spellings_but_not_directories() {
        let base = std::env::temp_dir().join(format!("c2rs-cachecwd-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let (a, b) = (base.join("a"), base.join("b"));
        std::fs::create_dir_all(&a).unwrap();
        std::fs::create_dir_all(&b).unwrap();
        for d in [&a, &b] {
            std::fs::write(d.join("t.cpp"), b"int f(){return 1;}").unwrap();
        }
        let c = CaptureCache {
            root: base.clone(),
            context: "ctx".to_string(),
            header_closure: true,
            validate_every: 0,
            seq: AtomicUsize::new(0),
            stats: Mutex::new(CacheStats::default()),
        };
        let flags: Vec<String> = vec!["/O1".into()];
        let k = |d: PathBuf| c.key_material("t.cpp", &flags, Some(&d)).unwrap();

        // Three spellings of directory `a`, including the `..`-hop form that
        // minted its own generation in the field. One key.
        let direct = k(a.clone());
        assert_eq!(direct, k(base.join("./a")));
        assert_eq!(direct, k(base.join("b/../a")));

        // A different directory with byte-identical contents is still a
        // different key — canonicalization must not reach that far.
        assert_ne!(direct, k(b.clone()));

        // A cwd that does not resolve is not cacheable at all when the source
        // is relative — it cannot be read, so the cache fails open (unchanged
        // by canonicalization; `host_source_path` still joins the raw spelling).
        assert!(c
            .key_material("t.cpp", &flags, Some(&base.join("nonexistent")))
            .is_none());

        // …and with an absolute source the canonicalization itself falls back
        // to the raw spelling rather than panicking, so an unresolvable cwd
        // still produces a key — just never the same one as a real directory.
        let abs = a.join("t.cpp");
        let abs = abs.to_str().unwrap();
        assert_ne!(
            c.key_material(abs, &flags, Some(&a)).unwrap(),
            c.key_material(abs, &flags, Some(&base.join("nonexistent"))).unwrap()
        );
        let _ = std::fs::remove_dir_all(&base);
    }

    /// The lockfile is exclusive, self-releasing, and fail-open.
    #[test]
    fn the_key_lock_excludes_a_second_holder_and_releases_on_drop() {
        let root = std::env::temp_dir().join(format!("c2rs-cachelock-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let key = "0123456789abcdef0123456789abcdef";

        let held = KeyLock::acquire(&root, key).expect("first acquire");
        assert!(root.join(LOCK_DIR).join(key).exists());

        // A second holder must not get in while the first is live. Proven
        // against the primitive rather than against a timeout: the O_EXCL open
        // itself has to fail.
        assert!(std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(root.join(LOCK_DIR).join(key))
            .is_err());

        drop(held);
        assert!(!root.join(LOCK_DIR).join(key).exists());
        // …and the key is acquirable again afterwards.
        assert!(KeyLock::acquire(&root, key).is_some());

        // Fail-open: an unusable root yields None (proceed unguarded) rather
        // than an error the scan would have to handle.
        let not_a_dir = root.join("file");
        std::fs::write(&not_a_dir, b"x").unwrap();
        assert!(KeyLock::acquire(&not_a_dir, key).is_none());
        let _ = std::fs::remove_dir_all(&root);
    }

    /// The lock directory must not be mistaken for a cache entry.
    #[test]
    fn the_lock_directory_is_not_a_readable_entry() {
        let dir = std::env::temp_dir().join(format!("c2rs-cachelockdir-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        assert!(matches!(read_entry(&dir, b"anything"), EntryRead::Miss));
        // 32-hex is what an entry name looks like; `.locks` is not that, which
        // is what keeps the age GC's `-name '[0-9a-f]*'` filter honest.
        assert!(!LOCK_DIR.chars().all(|c| c.is_ascii_hexdigit()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The soundness precondition for canonicalizing `cwd`, held to the real
    /// toolchain: the *spelling* of the working directory must not reach the
    /// captured bytes. If it did, folding two spellings onto one key would
    /// serve bytes captured under the other spelling — the same class of silent
    /// wrong answer that leaving `cache-root` out of the key once produced.
    ///
    /// Both captures go into the *same* directory so `-Fo` is identical and the
    /// only variable is the cwd spelling. Toolchain-absent is a clean SKIP.
    #[test]
    fn two_spellings_of_one_cwd_capture_identical_bytes() {
        let Some(tc) = Toolchain::locate() else {
            eprintln!("SKIP: toolchain absent");
            return;
        };
        let base = std::env::temp_dir().join(format!("c2rs-cwdspell-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let src_dir = base.join("src");
        let work = base.join("work");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::create_dir_all(&work).unwrap();
        std::fs::write(src_dir.join("t.cpp"), b"int f(int a,int b){return a+b;}\n").unwrap();

        let flags: Vec<String> = vec!["/c".into(), "/O1".into()];
        let Ok(direct) = tc.capture_reference_with("t.cpp", &work, &flags, Some(&src_dir)) else {
            eprintln!("SKIP: reference capture unavailable");
            return;
        };
        let hop = base.join("work/../src");
        let via_hop = tc
            .capture_reference_with("t.cpp", &work, &flags, Some(&hop))
            .expect("capture via the `..`-hop spelling of the same directory");

        match compare_captures(&direct, &via_hop) {
            CaptureDiff::Identical | CaptureDiff::TimestampOnly => {}
            CaptureDiff::Differs(what) => panic!(
                "the cwd SPELLING reached the captured bytes ({what}) — canonicalizing \
                 `cwd` in key_material would then fold two different outputs onto one \
                 key. Revert that canonicalization; do not relax this test."
            ),
        }
        let _ = std::fs::remove_dir_all(&base);
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

    /// **Board #1388.** A RELATIVE `--cache` spelling must address the same
    /// entry as the absolute one *and serve it at the same absolute path*.
    ///
    /// The key half of that was already true — `cache-root` is canonicalized —
    /// and it is exactly what made the defect stable: both spellings hit ONE
    /// entry, and the relative one then handed the port
    /// `work/…/out.obj` as `S_OBJNAME` while the cached reference obj carried
    /// `Z:\…\work\…\out.obj`. Byte-exact TU, verdict `mismatch`.
    ///
    /// This asserts the two halves separately, so a future change that fixes
    /// one and breaks the other cannot pass. No capture is performed, so it
    /// needs no `strace`; `Toolchain::locate` is required only because the key
    /// hashes the compiler DLLs.
    #[test]
    fn a_relative_root_and_an_absolute_root_agree_on_key_and_on_served_path() {
        let Some(tc) = Toolchain::locate() else {
            eprintln!("SKIP: toolchain absent");
            return;
        };
        let base = std::env::temp_dir().join(format!("c2rs-cacherel-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join("cache")).unwrap();
        std::fs::write(base.join("a.cpp"), b"int f(){return 1;}").unwrap();
        let flags: Vec<String> = vec!["/O1".into(), "/c".into()];

        // The relative spelling, resolved against THIS process's cwd — which is
        // what `--cache work/x` means and what the field bug was reported with.
        // Reached via a `..`-hop so the string is genuinely non-canonical.
        let abs = base.join("cache");
        let hop = base.join("cache/../cache");

        let c_abs = CaptureCache::new(abs.clone(), &tc, Some(&base), 0).unwrap();
        let c_hop = CaptureCache::new(hop, &tc, Some(&base), 0).unwrap();

        // Half 1: one key. (True before the fix, too — this is the half that
        // made the wrong answer reproducible rather than intermittent.)
        assert_eq!(
            c_abs.key_material("a.cpp", &flags, Some(&base)).unwrap(),
            c_hop.key_material("a.cpp", &flags, Some(&base)).unwrap(),
            "two spellings of one cache directory must address one entry"
        );

        // Half 2: one SERVED path, and it is absolute. This is the half that was
        // false. `root()` is what `read_entry` joins the key onto, so it is the
        // path the port is handed as `S_OBJNAME` on every hit.
        assert!(
            c_hop.root().is_absolute(),
            "the cache root is served as {:?} — a relative root makes to_wibo_path \
             hand the port a relative S_OBJNAME on every HIT, while the cached obj \
             carries the absolute one (board #1388)",
            c_hop.root()
        );
        assert_eq!(
            c_abs.root(),
            c_hop.root(),
            "two spellings of one cache directory must serve from one path"
        );
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
        assert!(matches!(read_entry(&dir, b"material-B"), EntryRead::Miss));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// **Board #1388's guard, fired on purpose.** An entry that records a
    /// capture path other than the one it is being served from is REFUSED —
    /// never served — because the reference obj embeds that path as
    /// `S_OBJNAME` and serving it fakes a `mismatch` on a byte-exact TU.
    ///
    /// Toolchain-free: this is a property of `meta.txt`, not of `c2`.
    #[test]
    fn an_entry_recorded_at_another_path_is_refused_not_served() {
        let dir = std::env::temp_dir().join(format!("c2rs-cacheprov-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("key.bin"), b"material-A").unwrap();
        let meta = |objpath: Option<&str>| {
            let mut m = format!("{CACHE_FORMAT}\nbase _CL_1\narg -il\n");
            if let Some(p) = objpath {
                m.push_str(&format!("{CAPTURE_PATH_KEY}{p}\n"));
            }
            std::fs::write(dir.join("meta.txt"), m).unwrap();
        };

        // Recorded somewhere else -> Foreign, and the message NAMES both paths.
        meta(Some("/somewhere/else/out.obj"));
        let what = match read_entry(&dir, b"material-A") {
            EntryRead::Foreign(w) => w,
            other => panic!("a foreign entry read as {other:?} — it must be REFUSED"),
        };
        assert!(what.contains("/somewhere/else/out.obj"), "{what}");
        assert!(what.contains(&dir.join("out.obj").display().to_string()), "{what}");

        // Recorded HERE -> the provenance check passes and the read continues
        // (and then misses on the absent obj/bundle). Miss, never Foreign: this
        // is the arm that would catch the check refusing everything, which would
        // turn the cache into a permanent cold start rather than a wrong answer.
        meta(Some(&dir.join("out.obj").display().to_string()));
        assert!(
            matches!(read_entry(&dir, b"material-A"), EntryRead::Miss),
            "an entry recorded at its own path must not be refused on provenance"
        );

        // No line at all -> a pre-#1388 entry. Also not Foreign: the cache root
        // is in the key, so those entries' paths are already pinned, and
        // refusing them would cold-start every cache on this box at once.
        meta(None);
        assert!(
            matches!(read_entry(&dir, b"material-A"), EntryRead::Miss),
            "an entry written before the provenance line must not be refused"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A written entry records where it was captured, so the guard above has
    /// something to check. Without this, `write_entry` could stop emitting the
    /// line and every `read_entry` would take the pre-#1388 compatibility arm —
    /// green, and checking nothing.
    #[test]
    fn a_written_entry_records_its_own_capture_path() {
        let dir = std::env::temp_dir().join(format!("c2rs-cachewrite-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let mut b = IlBundle::new("_CL_test");
        b.set("ex", b"EX".to_vec());
        let cap = CapturedReference {
            bundle: b,
            base_name: "_CL_test".to_string(),
            c2_argv: vec!["-il".to_string()],
            ref_obj: ObjImage::new(b"OOOOOOOOOOOO".to_vec()),
            ref_obj_path: dir.join("out.obj"),
        };
        write_entry(&dir, b"material-A", &cap).unwrap();
        let meta = std::fs::read_to_string(dir.join("meta.txt")).unwrap();
        assert!(
            meta.lines().any(|l| l
                == format!("{CAPTURE_PATH_KEY}{}", dir.join("out.obj").display())),
            "write_entry did not record the capture path; meta.txt was:\n{meta}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_incomplete_entry_reads_as_a_miss() {
        // meta.txt is the completion marker: key material alone is not an entry.
        let dir = std::env::temp_dir().join(format!("c2rs-cacheinc-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("key.bin"), b"material-A").unwrap();
        assert!(matches!(read_entry(&dir, b"material-A"), EntryRead::Miss));
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
