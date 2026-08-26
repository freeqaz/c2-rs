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
//! header or not — changes every key. When there is no workload tree to probe —
//! either the caller passed `None`, or the directory is not a checkout — the
//! token is `no-git` and the key falls back to the source file's own bytes.
//!
//! **Nothing prints a warning about that, and this file used to claim
//! otherwise.** A `header_closure_warning()` accessor sat here from the cache's
//! first commit with **no caller in `src/` or `tests/`**, and this paragraph
//! advertised it as *"says so out loud"* — a guarantee no run has ever made.
//! Deleted rather than wired (review §2.4, lane `w-refrev`), because wiring it
//! would have fired on the wrong caller: `c2rs diff` passes `workload_dir =
//! None` **deliberately** (its inputs are the self-contained `fixtures/cpp/*.cpp`,
//! which include no project headers, so their own bytes ARE the closure), and it
//! would therefore have printed a staleness warning on every fixture diff. The
//! caller for which the closure is load-bearing is `c2rs gap`, which passes the
//! workload `--cwd` and gets the full `tree`/`tree-dirty` binding above. A run
//! over a workload that is genuinely not a checkout is a run whose key cannot
//! see header edits: use `--no-cache`, or `--validate-cache N`.
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
//! Captured IL is **never** committed, but the reason changed in v2 and the old
//! one is now false. It used to be "the default root is `<repo>/work/`, which
//! `.gitignore` covers". The default root is no longer inside any checkout —
//! see [`default_cache_root`] — so the guarantee is now structural rather than
//! rule-based: there is nothing to ignore because there is nothing in the tree.
//! A root pointed back into a repo with `--cache` or `$C2RS_GAP_CACHE` is an
//! operator choice and carries the operator's `.gitignore` obligation with it.
//!
//! # v2: one entry is three inodes, not nine
//!
//! An entry was `key.bin`, `meta.txt`, `out.obj` and five `_CL_*` IL streams —
//! eight files plus the directory. It is now the directory, `out.obj`, and one
//! [`c2_il::cachefmt`] `entry.bin` holding the other seven. `out.obj` cannot
//! join them: it is the `-Fo` target and its own path is baked into it, which is
//! the same fact that makes this cache directory the capture directory. The
//! motivation, the format and why the fold is a strict strengthening of the old
//! completion marker are in that module's docs.

use std::io;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use c2_il::IL_SUFFIXES;
use c2_obj::{ObjDiff, ObjImage};
use c2_reference::{CapturedReference, Toolchain};

use crate::provenance::GitInfo;

/// On-disk format version, recorded in the `meta` text. Bump on any layout/key
/// change — old entries then key differently and are simply never read again.
///
/// v2 is the fold: seven small files became one [`c2_il::cachefmt`] blob. There
/// is no migration and cannot be, because a v1 `out.obj` has the *old root's*
/// path baked into `S_OBJNAME`; rewriting the container cannot rewrite the obj
/// bytes, so a "migrated" entry would be [`EntryRead::Foreign`] by construction
/// and suppressing that check to make migration work re-creates board #1388
/// exactly. v1 entries are invalidated, not converted — and since v2 also moved
/// the root, they are invalidated by a `cache-root` change no predicate could
/// have matched anyway.
/// PROV[N] not load-bearing — the port's own capture-cache format tag. Nothing here is derived from `c2.dll` or from its output; a wrong value invalidates a cache entry, it does not move a graded byte.
const CACHE_FORMAT: &str = "c2rs-capture-cache/v2";

/// The one file a v2 entry's metadata, key and IL streams live in.
///
/// `pub` for the same reason [`LOCK_DIR`] is: consumers that must name it — the
/// validator test, `c2rs cache show`, `scripts/gt_frame_class.py` — should test
/// against one exported name rather than each hard-coding a string. Decode it
/// with [`c2_il::decode_entry`]; do not parse it by hand.
/// PROV[N] not load-bearing — the port's own cache entry filename. Nothing here is derived from `c2.dll` or from its output; a wrong value invalidates a cache entry, it does not move a graded byte.
pub const ENTRY_BLOB: &str = "entry.bin";

/// Prefix for the write-then-rename temporary. **Must not start with `_CL_`**:
/// `capture_reference_with` sweeps `_CL_*` out of its work dir before compiling
/// and `find_bundle_base` matches `_CL_*ex`, so such a name would be deleted
/// mid-write or mistaken for an IL stream.
/// PROV[N] not load-bearing — the port's own temp-file prefix, chosen so `find_bundle_base`'s `_CL_*ex` glob cannot match it. Nothing here is derived from `c2.dll` or from its output; a wrong value invalidates a cache entry, it does not move a graded byte.
const TMP_PREFIX: &str = ".entry.tmp.";

/// `meta.txt` line recording the **absolute path this entry's `out.obj` was
/// captured at**. Written since board #1388; absent on older entries, which are
/// still served (the key already pins the cache root, so their path is
/// derivable — the line exists to make that derivation *checkable* rather than
/// assumed, and to keep it checkable if the derivation is ever changed again).
///
/// Deliberately NOT part of the key material: adding it there would cold-start
/// every existing entry — ~450 s of CPU per scan, paid by every concurrent lane
/// — to re-derive bytes that are already correct.
/// PROV[N] not load-bearing — a field tag in the port's own cache key. Nothing here is derived from `c2.dll` or from its output; a wrong value invalidates a cache entry, it does not move a graded byte.
pub const CAPTURE_PATH_KEY: &str = "objpath ";

/// 128-bit content digest — see [`c2_il::cachefmt::digest128`].
///
/// Re-exported rather than reimplemented: the blob header stores one of these
/// and the cache key *is* one, so the two must be the same function or an entry
/// keyed by the harness could be validated against a different digest by the
/// codec. Kept as a re-export at this path because it is the spelling every
/// caller here already uses, and moving it was not the point of the change.
pub use c2_il::cachefmt::digest128;

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

/// **The one place that decides WHERE the cache lives**, given an explicit
/// `--cache` (already applied by the caller) and the environment.
///
/// The rule was written twice — `cli/gap.rs` and `cli/reference.rs` carried the
/// same eight lines — which is the "one rule, two implementations" shape
/// [`capture_via`] below exists to close, on the same path. Two spellings of one
/// root is how a lane ends up grading against a cache nobody realised was
/// separate.
///
/// Resolution order, first hit wins:
///
/// 1. `--cache DIR` — applied by the caller before this is reached.
/// 2. `$C2RS_GAP_CACHE`
/// 3. `$XDG_CACHE_HOME/c2rs/capture` (ignored unless absolute, per the spec)
/// 4. `$HOME/.cache/c2rs/capture`
/// 5. **Error.** Never a silent fallback.
///
/// # Why this is no longer under the repo
///
/// It was `<main repo>/work/capture-cache`, and at 22.6 M entries × 9 inodes
/// that put ~203 M inodes *inside the checkout*. Every `find`, `du`, `rg` or
/// `**` glob rooted at the repo then walks them: twice an OOM kill of the whole
/// machine, once an hour-long wedge at load 37.8. No amount of care in this
/// codebase prevents that, because the traversals come from outside it — a
/// shell, an editor's indexer, another agent's `du -sh .`. Moving the root out
/// of the tree is the only change that makes the footgun structurally
/// unreachable rather than merely discouraged.
///
/// Falling back to the repo when `$HOME` is unset would resurrect it silently
/// and invisibly, which is why step 5 is an error. Callers on the capture path
/// degrade to no-cache and say so; a cache is a speedup, never a grading input.
///
/// **Do not point this at `$TMPDIR`.** `/tmp` here is a 47 GB tmpfs that dies on
/// reboot and competes with `scripts/gate.sh` for the same 1 M inodes its disk
/// preflight guards — a cold cache is an hour of `cl.exe`, and losing it to a
/// reboot is the cheap half of the problem.
///
/// `main_repo_root` is no longer consulted: `~/.cache/c2rs` collapses *clones*
/// as well as worktrees, which is more sharing than before. That is the intent —
/// the entries are keyed by toolchain digests, tree token and cache root, so a
/// wrong hit is refused rather than served — but it does mean `rm -rf
/// ~/.cache/c2rs` is now the blast radius for every clone on the box.
pub fn default_cache_root() -> io::Result<PathBuf> {
    // An env var set to the empty string is a broken export, not a request to
    // cache at the process's cwd.
    let from_env = |k: &str| {
        std::env::var_os(k)
            .map(PathBuf::from)
            .filter(|p| !p.as_os_str().is_empty())
    };
    if let Some(p) = from_env("C2RS_GAP_CACHE") {
        return Ok(p);
    }
    if let Some(p) = from_env("XDG_CACHE_HOME").filter(|p| p.is_absolute()) {
        return Ok(p.join("c2rs/capture"));
    }
    if let Some(p) = from_env("HOME") {
        return Ok(p.join(".cache/c2rs/capture"));
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "no capture-cache root: set $C2RS_GAP_CACHE, $XDG_CACHE_HOME or $HOME, or pass \
         --cache DIR. There is deliberately no fallback inside the repo — a cache root \
         under the checkout is what put 203 M inodes in the path of every `find` and `du`",
    ))
}

/// **The one place that decides "cache or straight to the toolchain".**
///
/// The rule is two lines long and it was written twice — `lib.rs`'s
/// `differential_cached` (the fixture gate) and `gap/scan.rs`'s `scan_one` (the
/// workload scan) each had their own `match cache { Some(c) => c.capture(…),
/// None => tc.capture_reference_with(…) }`. Both sit on the capture path the
/// judge runs through, and "one rule, two implementations" on that path is the
/// shape `docs/GAPS.md` §6 keeps recording (mis-emit #11's class): the two arms
/// drift, and the drift is invisible because each arm is separately green.
/// Review §2.4/§2.1, lane `w-refrev`.
///
/// Semantics are exactly what both call sites had:
///
/// * with a cache, [`CaptureCache::capture`] owns the key, the lock, the
///   validator and the counters, and `work` is its *fallback* capture dir (an
///   unkeyable TU is captured there and counted `bypassed`);
/// * without one, the capture goes straight to `work` and the outcome word is
///   [`CacheOutcome::Bypassed`] — which is what `differential_cached` already
///   reported for a cache-less run and what `scan_one` discarded.
///
/// The outcome is returned in both arms rather than only in the cached one, so
/// a caller that ignores it (the scan) does so *visibly*, at its own call site.
pub fn capture_via(
    cache: Option<&CaptureCache>,
    tc: &Toolchain,
    src_arg: &str,
    flags: &[String],
    cwd: Option<&Path>,
    work: &Path,
) -> (io::Result<CapturedReference>, CacheOutcome) {
    match cache {
        Some(c) => c.capture(tc, src_arg, flags, cwd, work),
        None => (
            tc.capture_reference_with(src_arg, work, flags, cwd),
            CacheOutcome::Bypassed,
        ),
    }
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
    /// Entries captured but not recorded, because completing them failed
    /// (ENOSPC, a read-only root, a permissions change under a running scan).
    /// Counted because the symptom is otherwise invisible: a cache that can
    /// never write presents as a permanent 100 % miss rate, which looks exactly
    /// like a cold run and stays that way forever.
    pub write_failed: usize,
    /// One line per failed completion: `<src>: <io error>`.
    pub write_failed_detail: Vec<String>,
}

/// The cache.
pub struct CaptureCache {
    root: PathBuf,
    /// Identity of everything that is not the source file: toolchain + tree.
    ///
    /// Whether the header closure is live is readable *here* — `context` carries
    /// `tree <token>`, and `no-git` is the token for "no checkout to bind to".
    /// It is deliberately not also a `bool` field: the one accessor that read
    /// such a field had no caller and is gone (module docs above).
    context: String,
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
/// PROV[N] not load-bearing — the port's own lock directory name. Nothing here is derived from `c2.dll` or from its output; a wrong value invalidates a cache entry, it does not move a graded byte.
pub const LOCK_DIR: &str = ".locks";

/// Give up waiting and proceed unguarded. Nothing downstream blocks on the
/// lock, so the failure mode is today's behaviour, not a wedged scan.
/// PROV[N] not load-bearing — a wait ceiling; its own doc says nothing downstream blocks on the lock, so the failure mode is today's behaviour. Nothing here is derived from `c2.dll` or from its output; a wrong value invalidates a cache entry, it does not move a graded byte.
const LOCK_WAIT_MAX: Duration = Duration::from_secs(600);

/// Break a lock whose holder is gone. A capture is one `cl.exe` invocation
/// (~1–2 s); 30 minutes is ~1000× headroom, and without it a single SIGKILL
/// would poison one key of the cache permanently.
/// PROV[N] not load-bearing — a staleness ceiling with ~1000x headroom over the measured hold time. Nothing here is derived from `c2.dll` or from its output; a wrong value invalidates a cache entry, it does not move a graded byte.
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

    /// Take the lock if it is free **right now**, or give up immediately.
    ///
    /// Deliberately not [`KeyLock::acquire`]: that waits `LOCK_WAIT_MAX` and
    /// breaks a lock older than `LOCK_STALE_AFTER`, which is right for a capture
    /// (the alternative is a key wedged forever by one SIGKILL) and wrong for
    /// the GC. A GC that breaks a lock is a GC deciding a capture is dead, which
    /// it cannot know; the safe reading of a held lock is "someone is working
    /// here, skip it" — the entry will still be there next pass.
    fn try_acquire(root: &Path, key: &str) -> Option<KeyLock> {
        let dir = root.join(LOCK_DIR);
        std::fs::create_dir_all(&dir).ok()?;
        let path = dir.join(key);
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(mut f) => {
                let _ = writeln!(f, "{}", std::process::id());
                Some(KeyLock { path })
            }
            Err(_) => None,
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
                        let wrote = write_entry(&dir, &material, &fresh);
                        let mut st = self.stats.lock().unwrap();
                        if let Err(e) = wrote {
                            st.write_failed += 1;
                            st.write_failed_detail.push(format!("{src_arg}: {e}"));
                        }
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
                        let wrote = write_entry(&dir, &material, cap);
                        let mut st = self.stats.lock().unwrap();
                        if let Err(e) = wrote {
                            st.write_failed += 1;
                            st.write_failed_detail.push(format!("{src_arg}: {e}"));
                        }
                        st.misses += 1;
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
    // Absence of the blob is the completion marker: it appears by `rename(2)`,
    // so there is no window in which a partial one is visible under this name.
    let Ok(raw) = std::fs::read(dir.join(ENTRY_BLOB)) else {
        return EntryRead::Miss;
    };
    let Ok(blob) = c2_il::decode_entry(&raw) else {
        return EntryRead::Miss;
    };
    if blob.key != material {
        return EntryRead::Miss;
    }
    let mut base_name = String::new();
    let mut c2_argv: Vec<String> = Vec::new();
    let mut recorded_path: Option<&str> = None;
    for line in blob.meta.lines() {
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
    // The provenance check (board #1388). The pre-#1388 "line absent, serve
    // anyway" arm is gone: every v2 entry carries the line, so that arm had
    // provably zero members and was dead code that read as a tested branch. A
    // format bump is the one moment retiring it is free.
    let Some(recorded) = recorded_path else {
        return EntryRead::Miss;
    };
    if Path::new(recorded) != ref_obj_path {
        return EntryRead::Foreign(format!(
            "entry records its capture at {recorded} but is being served from {} \
             — the obj embeds its own -Fo path, so those are not the bytes c2 \
             would emit here",
            ref_obj_path.display()
        ));
    }
    // The obj stays a real file at this exact path: it is the `-Fo` target and
    // the replay target, and its own path is inside its bytes.
    let Ok(obj) = std::fs::read(&ref_obj_path) else {
        return EntryRead::Miss;
    };
    if obj.is_empty() {
        return EntryRead::Miss;
    }
    let bundle = blob.bundle(&base_name);
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

/// Complete an entry: fold the key material, the metadata and the five IL
/// streams into one `entry.bin`, then remove the IL files the capture left
/// behind. `out.obj` is already on disk at the right path and stays there.
///
/// The streams are serialised from `cap`, which already holds them in memory
/// (`capture_reference_with` loads them), so nothing is read back off disk.
///
/// No `sync_all`: the cache is an optimisation and correctness never depends on
/// it, so a blob lost to a power cut is a miss and a re-capture. A blob that
/// survives *torn* is caught by `TOTAL_LEN` and the digest — which is strictly
/// stronger than the layout this replaces, where the completion marker was
/// "`meta.txt` exists" and a half-written one parsed as a Hit with a truncated
/// argv.
fn write_entry(dir: &Path, material: &[u8], cap: &CapturedReference) -> io::Result<()> {
    std::fs::create_dir_all(dir)?;
    let blob = c2_il::encode_entry(material, &encode_meta(cap), &cap.bundle);

    // NOT a `_CL_` prefix: `capture_reference_with` sweeps `_CL_*` out of its
    // work dir before compiling and `find_bundle_base` matches `_CL_*ex`, so a
    // tmp file named that way would be deleted mid-write or mistaken for IL.
    // PROV[N] not load-bearing — a monotonic tmp-name tie-breaker; scratch state. Nothing here is derived from `c2.dll` or from its output; a wrong value invalidates a cache entry, it does not move a graded byte.
    static SEQ: AtomicUsize = AtomicUsize::new(0);
    let tmp = dir.join(format!(
        "{TMP_PREFIX}{}.{}",
        std::process::id(),
        SEQ.fetch_add(1, Ordering::Relaxed)
    ));
    // Sweep any tmp a crash left between write and rename. Here rather than in
    // `gc` because this is the only place that can look inside an entry without
    // breaking the GC's "one open per entry, never recurse" bound — and the
    // directory it reads has at most a handful of names. An entry never
    // re-captured keeps its stray: one inode, against a re-scan of the root.
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            if e.file_name().to_string_lossy().starts_with(TMP_PREFIX) {
                let _ = std::fs::remove_file(e.path());
            }
        }
    }
    std::fs::write(&tmp, &blob)?;
    // Atomic within the directory: the entry appears whole or not at all.
    if let Err(e) = std::fs::rename(&tmp, dir.join(ENTRY_BLOB)) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }

    // Best-effort: the blob is authoritative from the rename onward, so an
    // orphaned `_CL_*` left by a crash here is ignored on read and swept by the
    // next capture into this directory.
    for suffix in IL_SUFFIXES {
        let _ = std::fs::remove_file(dir.join(format!("{}{suffix}", cap.base_name)));
    }
    // v1 leftovers, for the case where a v1 root is reused via `--cache`. The
    // key differs so they are never *read*, but they are two inodes each.
    for stale in ["key.bin", "meta.txt"] {
        let _ = std::fs::remove_file(dir.join(stale));
    }
    Ok(())
}

/// The `meta.txt` text, as a pure function of the capture.
///
/// Split out from the I/O so a test can mutate what production actually writes
/// instead of hand-rolling a second implementation of the grammar beside it —
/// the hand-rolled copies in the in-module tests are what let the format and its
/// tests drift apart.
fn encode_meta(cap: &CapturedReference) -> String {
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
    meta
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

// ---------------------------------------------------------------------------
// Reclamation (board #3265). The cache had no GC and nothing evicts, so it grew
// to ~21.5 M entries; the 2026-08-04 cleanup was a one-off manual delete, which
// is why it grew back. What follows is that cleanup as code.
// ---------------------------------------------------------------------------

/// `src-arg\0` — the first NUL-delimited field of the per-TU key tail.
/// PROV[N] not load-bearing — a NUL-delimited field tag in the port's own cache key. Nothing here is derived from `c2.dll` or from its output; a wrong value invalidates a cache entry, it does not move a graded byte.
const SRC_ARG_TAG: &[u8] = b"src-arg\x00";
/// `\0cwd\0`.
/// PROV[N] not load-bearing — the second such field tag. Nothing here is derived from `c2.dll` or from its output; a wrong value invalidates a cache entry, it does not move a graded byte.
const CWD_TAG: &[u8] = b"\x00cwd\x00";

fn find_sub(hay: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || hay.len() < needle.len() {
        return None;
    }
    (0..=hay.len() - needle.len()).find(|&i| &hay[i..i + needle.len()] == needle)
}

/// The fields of a `key.bin` a reclamation pass needs: the shared context
/// prefix, and the two components that name the source file.
///
/// Parsing rather than re-deriving is the point — a re-derivation would need
/// this binary's own toolchain digests and workload tree, i.e. it could only
/// ever classify the generation it is standing in.
///
/// The split is unambiguous only because `context` contains no NUL (it is
/// entirely `format!`-built newline-terminated text) — `the_context_contains_no_nul`
/// holds that, since without it this parse is a guess that happens to work.
pub struct KeyFields<'a> {
    /// Everything before `src-arg\0`: format tag, tool digests, wibo, tree,
    /// tree-dirty, cache-root. Identifies the *generation*.
    pub context: &'a [u8],
    pub src_arg: &'a str,
    /// Empty when the capture passed no `--cwd`.
    pub cwd: &'a str,
}

/// Split raw key material into [`KeyFields`]. `None` if it is not key material.
pub fn parse_key_material(material: &[u8]) -> Option<KeyFields<'_>> {
    let at = find_sub(material, SRC_ARG_TAG)?;
    let context = &material[..at];
    let rest = &material[at + SRC_ARG_TAG.len()..];
    let end = rest.iter().position(|b| *b == 0)?;
    let src_arg = std::str::from_utf8(&rest[..end]).ok()?;
    let cat = find_sub(rest, CWD_TAG)?;
    let after = &rest[cat + CWD_TAG.len()..];
    let cend = after.iter().position(|b| *b == 0)?;
    let cwd = std::str::from_utf8(&after[..cend]).ok()?;
    Some(KeyFields { context, src_arg, cwd })
}

/// What a reclamation pass decided about one entry. Everything except
/// [`EntryVerdict::Unreachable`] is a KEEP.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EntryVerdict {
    /// The source file is present: this key can still be formed.
    Live,
    /// The source's directory is readable and the source is not in it, so
    /// `key_material` would return `None` and **this key can never be formed
    /// again**. The only predicate that deletes.
    Unreachable,
    /// Could not be established — an unreadable or absent parent directory, an
    /// unparseable key. **Kept**, and counted, because "unknown" must not mean
    /// "delete": an unmounted volume or a workload tree mid-`git checkout` is
    /// indistinguishable from a deleted source, and the 2026-08-04 age
    /// predicate was refuted for exactly this class of mistake.
    Unknown(String),
    /// Younger than the pass's `min_age` floor.
    TooYoung,
    /// Another process holds this key's lock.
    Locked,
}

/// Classify one entry from its key material alone.
pub fn classify_entry(material: &[u8]) -> EntryVerdict {
    let Some(k) = parse_key_material(material) else {
        return EntryVerdict::Unknown("key material did not parse".into());
    };
    let cwd = (!k.cwd.is_empty()).then(|| PathBuf::from(k.cwd));
    let src = host_source_path(k.src_arg, cwd.as_deref());
    if src.exists() {
        return EntryVerdict::Live;
    }
    // The source is absent — but so is every file on a volume that is not
    // mounted. Only a readable parent makes "absent" mean "deleted".
    match src.parent() {
        Some(p) if std::fs::read_dir(p).is_ok() => EntryVerdict::Unreachable,
        Some(p) => EntryVerdict::Unknown(format!("parent unreadable: {}", p.display())),
        None => EntryVerdict::Unknown("source path has no parent".into()),
    }
}

/// Knobs for [`gc`].
pub struct GcOptions {
    /// Delete. Default is a dry run: a reclamation pass that ran silently is
    /// indistinguishable from one that did not.
    pub apply: bool,
    /// Stop after this many entries (bounds a first `--apply`).
    pub limit: Option<usize>,
    /// Do not touch an entry younger than this.
    ///
    /// Age used to **protect**, never to evict. The distinction is the whole
    /// correction of 2026-08-04: a hit never rewrites mtime and `/home` is
    /// `noatime`, so age cannot tell a live entry from residue and an age-based
    /// *eviction* would have deleted the live gate working set.
    pub min_age: Duration,
    /// Context digests (from [`generations`]) to delete wholesale, whatever the
    /// source predicate says. Operator-supplied on purpose — a shared root
    /// legitimately holds several live generations at once, so "not the context
    /// I would compute" is not a safe predicate for the code to apply itself.
    pub drop_generations: Vec<String>,
}

impl Default for GcOptions {
    fn default() -> Self {
        GcOptions {
            apply: false,
            limit: None,
            min_age: Duration::from_secs(3600),
            drop_generations: Vec::new(),
        }
    }
}

/// Counts from one reclamation pass. Every field is printed; a number nobody
/// prints is a number nobody can check.
#[derive(Clone, Debug, Default)]
pub struct GcReport {
    pub scanned: usize,
    pub live: usize,
    pub unreachable: usize,
    pub unknown: usize,
    pub too_young: usize,
    pub locked: usize,
    pub generation_dropped: usize,
    /// Children of the root that are neither a 32-hex entry nor [`LOCK_DIR`].
    /// **Counted, never deleted** — an unrecognised name is exactly the case
    /// where a reclamation pass should not be guessing.
    pub strays: usize,
    pub stray_names: Vec<String>,
    pub deleted: usize,
    pub delete_failed: usize,
    /// One line per distinct `Unknown` reason, capped.
    pub unknown_detail: Vec<String>,
}

fn is_entry_name(name: &str) -> bool {
    name.len() == 32 && name.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Read an entry's stored key material. **The one format-dependent read** in the
/// reclamation path, so a container change is one function.
///
/// Deliberately not a v1/v2 dual reader: a v1 entry is unreachable at a v2 root
/// (the root is in the key), so the only way to meet one here is `--cache` at an
/// old root, where it reads as an entry the GC cannot establish anything about —
/// `unknown`, and therefore kept. Retiring a v1 tree is one `rm -rf`, not a
/// reclamation pass.
pub(crate) fn entry_key_material(dir: &Path) -> Option<Vec<u8>> {
    let raw = std::fs::read(dir.join(ENTRY_BLOB)).ok()?;
    Some(c2_il::decode_entry(&raw).ok()?.key.to_vec())
}

/// Walk `root` one level and reclaim provably-unreachable entries.
///
/// **Enumeration is bounded by construction and must stay that way.**
/// `read_dir` is a lazy `getdents64` iterator, so it holds one name at a time —
/// that is why the standing rule forbids *globs* over a cache root and not
/// `readdir`. A shell glob over this directory has twice taken the machine down
/// with the OOM killer (62 GB and 72 GB of anonymous RSS in a `zsh`).
/// So: never `collect()` the entries, never sort them, never build a `Vec` of
/// paths, and never recurse into an entry. Adding any of those reintroduces the
/// OOM at 21.5 M entries.
pub fn gc(root: &Path, opts: &GcOptions, progress: &dyn Fn(usize)) -> io::Result<GcReport> {
    let mut rep = GcReport::default();
    for ent in std::fs::read_dir(root)? {
        let Ok(ent) = ent else { continue };
        let name = ent.file_name().to_string_lossy().into_owned();
        if name == LOCK_DIR {
            continue;
        }
        // From the dirent, not a `stat` per entry.
        if !ent.file_type().map(|t| t.is_dir()).unwrap_or(false) || !is_entry_name(&name) {
            rep.strays += 1;
            if rep.stray_names.len() < 20 {
                rep.stray_names.push(name);
            }
            continue;
        }
        if let Some(n) = opts.limit {
            if rep.scanned >= n {
                break;
            }
        }
        rep.scanned += 1;
        if rep.scanned % 100_000 == 0 {
            progress(rep.scanned);
        }
        let dir = ent.path();

        let young = std::fs::metadata(&dir)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.elapsed().ok())
            .is_some_and(|age| age < opts.min_age);
        if young {
            rep.too_young += 1;
            continue;
        }

        let Some(material) = entry_key_material(&dir) else {
            rep.unknown += 1;
            note(&mut rep.unknown_detail, "no readable entry.bin");
            continue;
        };

        let drop_gen = !opts.drop_generations.is_empty()
            && parse_key_material(&material)
                .map(|k| opts.drop_generations.contains(&digest128(k.context)))
                .unwrap_or(false);

        let verdict = classify_entry(&material);
        match &verdict {
            EntryVerdict::Live => rep.live += 1,
            EntryVerdict::Unreachable => rep.unreachable += 1,
            EntryVerdict::TooYoung => rep.too_young += 1,
            EntryVerdict::Locked => rep.locked += 1,
            EntryVerdict::Unknown(why) => {
                rep.unknown += 1;
                note(&mut rep.unknown_detail, why);
            }
        }
        if verdict != EntryVerdict::Unreachable && !drop_gen {
            continue;
        }
        if drop_gen && verdict != EntryVerdict::Unreachable {
            rep.generation_dropped += 1;
        }

        // Never remove an entry someone is capturing into. The lock is not a
        // complete guard — `capture()` is fail-open, so an unguarded capture
        // leaves this free to succeed — but the residual is small: an entry
        // being captured has a source that exists, which is `Live`, and the
        // `min_age` floor covers the `--drop-generation` arm.
        let Some(_guard) = KeyLock::try_acquire(root, &name) else {
            rep.locked += 1;
            if verdict == EntryVerdict::Unreachable {
                rep.unreachable -= 1;
            }
            continue;
        };
        if !opts.apply {
            continue;
        }
        match std::fs::remove_dir_all(&dir) {
            Ok(()) => rep.deleted += 1,
            Err(_) => rep.delete_failed += 1,
        }
    }
    Ok(rep)
}

fn note(detail: &mut Vec<String>, why: &str) {
    if detail.len() < 20 && !detail.iter().any(|d| d == why) {
        detail.push(why.to_string());
    }
}

/// Stream `<src-arg>\t<entry dir>` for every entry, one line at a time.
///
/// **The supported bulk reader.** Consumers used to hand-roll this — three
/// untracked `cacheindex.py`/`cachekey.py` copies in various worktrees, each
/// with its own `digest128` and its own idea of the entry layout, and
/// `scripts/gt_frame_class.py` with a `glob(<cache>/*/out.obj)` that materialises
/// every path in one list. At 22.5 M entries that glob is the OOM this whole
/// change exists to remove. One pass here, bounded exactly like [`gc`], is what
/// they should all call instead.
///
/// `emit` is called per entry rather than returning a `Vec` for the same reason
/// [`gc`] never collects: a `Vec` of 22.5 M paths is the failure mode, not an
/// implementation detail. Returns the number emitted and the number whose blob
/// could not be read.
pub fn index(root: &Path, emit: &mut dyn FnMut(&str, &Path)) -> io::Result<(usize, usize)> {
    let (mut n, mut unreadable) = (0usize, 0usize);
    for ent in std::fs::read_dir(root)? {
        let Ok(ent) = ent else { continue };
        let name = ent.file_name().to_string_lossy().into_owned();
        if name == LOCK_DIR || !is_entry_name(&name) {
            continue;
        }
        let dir = ent.path();
        let Some(material) = entry_key_material(&dir) else {
            unreadable += 1;
            continue;
        };
        match parse_key_material(&material) {
            Some(k) => {
                emit(k.src_arg, &dir);
                n += 1;
            }
            None => unreadable += 1,
        }
    }
    Ok((n, unreadable))
}

/// Histogram the root by generation: `(context digest, entries, decoded context)`.
///
/// A shared root holds several live generations at once — fixture captures key
/// as `tree unknown`, the workload scan carries the dc3 tree, the sweep and the
/// cross bring their own — so **which** generations are dead is an operator
/// judgement, not one this code can make. This prints the evidence for it.
/// `every` samples one entry in N — the histogram costs one `open` per entry
/// read, so a full pass over 22.5 M entries is hours of random IO. A sample is
/// a *sample*: callers must print the rate beside the counts, never scale them
/// up silently.
pub fn generations(root: &Path, every: usize) -> io::Result<(Vec<(String, usize, String)>, usize)> {
    let mut seen: Vec<(String, usize, String)> = Vec::new();
    let every = every.max(1);
    let mut seq = 0usize;
    let mut read = 0usize;
    for ent in std::fs::read_dir(root)? {
        let Ok(ent) = ent else { continue };
        let name = ent.file_name().to_string_lossy().into_owned();
        if name == LOCK_DIR || !is_entry_name(&name) {
            continue;
        }
        seq += 1;
        if seq % every != 0 {
            continue;
        }
        let Some(material) = entry_key_material(&ent.path()) else {
            continue;
        };
        read += 1;
        let Some(k) = parse_key_material(&material) else {
            continue;
        };
        let d = digest128(k.context);
        match seen.iter_mut().find(|(x, _, _)| *x == d) {
            Some((_, n, _)) => *n += 1,
            None => seen.push((d, 1, String::from_utf8_lossy(k.context).into_owned())),
        }
    }
    seen.sort_by(|a, b| b.1.cmp(&a.1));
    Ok((seen, read))
}

#[cfg(test)]
mod tests {
    use super::*;
    use c2_il::IlBundle;

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

    // ---- reclamation (board #3265) ------------------------------------------

    /// A scratch cache root with one hand-built entry keyed on `src`.
    fn gc_fixture(tag: &str, src: &Path) -> (PathBuf, PathBuf, Vec<u8>) {
        let root = std::env::temp_dir().join(format!("c2rs-gc-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let mut material = b"c2rs-capture-cache/v1\ntree unknown+dirty-unknown\n".to_vec();
        material.extend_from_slice(b"src-arg\x00");
        material.extend_from_slice(src.display().to_string().as_bytes());
        material.extend_from_slice(b"\x00src-bytes\x00");
        material.extend_from_slice(b"0:deadbeef");
        material.extend_from_slice(b"\x00cwd\x00");
        material.extend_from_slice(b"\x00flags\x00\x00");
        let key = digest128(&material);
        let dir = root.join(&key);
        std::fs::create_dir_all(&dir).unwrap();
        put_blob(&dir, &material, "base _CL_1\narg -il\n", &IlBundle::default());
        // Older than any `min_age` this test uses.
        (root, dir, material)
    }

    /// Write an entry blob with **production's own encoder**. Tests that want a
    /// broken entry mutate what this produced; hand-rolling a second
    /// implementation of the container beside the first is how a format and its
    /// tests drift apart.
    fn put_blob(dir: &Path, material: &[u8], meta: &str, bundle: &IlBundle) {
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(
            dir.join(ENTRY_BLOB),
            c2_il::encode_entry(material, meta, bundle),
        )
        .unwrap();
    }

    /// A **complete, servable** entry: everything `read_entry` needs for a Hit.
    /// Tests break exactly one thing against this, so a Miss is attributable.
    fn hittable_entry(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("c2rs-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("out.obj"), b"OOOOOOOOOOOO").unwrap();
        let mut b = IlBundle::new("_CL_1");
        b.set("ex", b"EXEX".to_vec());
        b.set("gl", b"GLGLGL".to_vec());
        put_blob(&dir, b"material-A", &hit_meta(&dir), &b);
        dir
    }

    /// The meta text of a `hittable_entry`, recording `dir` as its capture path.
    fn hit_meta(dir: &Path) -> String {
        format!(
            "{CACHE_FORMAT}\nbase _CL_1\narg -il\n{CAPTURE_PATH_KEY}{}\n",
            dir.join("out.obj").display()
        )
    }

    /// `min_age` protects a recent entry; a pass that must act uses 0.
    fn gc_now() -> GcOptions {
        GcOptions { apply: true, min_age: Duration::from_secs(0), ..GcOptions::default() }
    }

    /// The parse the whole reclamation path rests on. Without this the split on
    /// `src-arg\0` is a guess that happens to work.
    #[test]
    fn the_context_contains_no_nul() {
        let base = std::env::temp_dir().join(format!("c2rs-ctxnul-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();
        let Some(tc) = Toolchain::locate() else {
            eprintln!("SKIP: toolchain absent");
            return;
        };
        let c = CaptureCache::new(base.join("cache"), &tc, None, 0).unwrap();
        let material = c.key_material("Z:\\x.cpp", &["/Ox".to_string()], None);
        // No source file, so this may be `None`; when it is, assert on the
        // context the cache built for itself instead.
        let ctx = c.context.as_bytes();
        assert!(
            !ctx.contains(&0u8),
            "the cache context grew a NUL — parse_key_material splits on the first \
             `src-arg\\0` and would now find it inside the context"
        );
        if let Some(m) = material {
            let k = parse_key_material(&m).expect("key material must parse");
            assert!(!k.context.contains(&0u8));
        }
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn key_material_parses_back_to_its_src_arg_and_cwd() {
        let (root, _dir, material) = gc_fixture("parse", Path::new("/nonexistent/x.cpp"));
        let k = parse_key_material(&material).expect("parse");
        assert_eq!(k.src_arg, "/nonexistent/x.cpp");
        assert_eq!(k.cwd, "");
        assert!(k.context.starts_with(b"c2rs-capture-cache/v1\n"));
        let _ = std::fs::remove_dir_all(&root);
    }

    /// **"Unknown" must not mean "delete".** A source whose PARENT directory
    /// cannot be read is indistinguishable from a deleted source — an unmounted
    /// volume, a workload tree mid-`git checkout` — so the entry is kept. This
    /// is the row the refuted 2026-08-04 age predicate did not have.
    #[test]
    fn gc_keeps_an_entry_whose_source_parent_is_unreadable() {
        let (root, dir, _) = gc_fixture("unknown", Path::new("/nonexistent-volume/sub/x.cpp"));
        let rep = gc(&root, &gc_now(), &|_| {}).unwrap();
        assert_eq!(rep.unknown, 1, "an unreadable parent must classify as unknown");
        assert_eq!(rep.unreachable, 0);
        assert_eq!(rep.deleted, 0);
        assert!(dir.is_dir(), "gc deleted an entry it could not establish");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// The one predicate that deletes: the source's directory is readable and
    /// the source is not in it, so `key_material` returns `None` and this key
    /// can never be formed again.
    #[test]
    fn gc_deletes_only_a_provably_unreachable_entry() {
        let live = std::env::temp_dir().join(format!("c2rs-gclive-{}.cpp", std::process::id()));
        std::fs::write(&live, b"int f(){return 0;}\n").unwrap();
        let (root_l, dir_l, _) = gc_fixture("live", &live);
        let rep = gc(&root_l, &gc_now(), &|_| {}).unwrap();
        assert_eq!(rep.live, 1, "an existing source must be LIVE");
        assert_eq!(rep.deleted, 0);
        assert!(dir_l.is_dir());
        let _ = std::fs::remove_dir_all(&root_l);

        // Same shape, source removed from a directory that still reads.
        std::fs::remove_file(&live).unwrap();
        let (root_d, dir_d, _) = gc_fixture("dead", &live);
        let rep = gc(&root_d, &gc_now(), &|_| {}).unwrap();
        assert_eq!(rep.unreachable, 1);
        assert_eq!(rep.deleted, 1);
        assert!(!dir_d.exists(), "a provably-unreachable entry survived --apply");
        let _ = std::fs::remove_dir_all(&root_d);
    }

    /// A dry run is the default and must not touch anything.
    #[test]
    fn gc_dry_run_deletes_nothing() {
        let (root, dir, _) = gc_fixture("dry", Path::new("/tmp/c2rs-gc-absent-source.cpp"));
        let opts = GcOptions { min_age: Duration::from_secs(0), ..GcOptions::default() };
        let rep = gc(&root, &opts, &|_| {}).unwrap();
        assert_eq!(rep.unreachable, 1, "the predicate should still classify");
        assert_eq!(rep.deleted, 0, "a dry run deleted something");
        assert!(dir.is_dir());
        let _ = std::fs::remove_dir_all(&root);
    }

    /// A key someone is capturing into is skipped, not deleted.
    #[test]
    fn gc_skips_a_locked_key() {
        let (root, dir, _) = gc_fixture("locked", Path::new("/tmp/c2rs-gc-absent-source.cpp"));
        let key = dir.file_name().unwrap().to_string_lossy().into_owned();
        let held = KeyLock::acquire(&root, &key).expect("take the lock");
        let rep = gc(&root, &gc_now(), &|_| {}).unwrap();
        assert_eq!(rep.locked, 1, "a held key must be skipped");
        assert_eq!(rep.deleted, 0);
        assert!(dir.is_dir(), "gc deleted an entry another process holds");
        drop(held);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// `.locks` is not an entry and must never be walked as one.
    #[test]
    fn gc_never_touches_the_lock_dir() {
        let (root, _dir, _) = gc_fixture("lockdir", Path::new("/tmp/c2rs-gc-absent-source.cpp"));
        let locks = root.join(LOCK_DIR);
        std::fs::create_dir_all(&locks).unwrap();
        std::fs::write(locks.join("deadbeef"), b"1\n").unwrap();
        let rep = gc(&root, &gc_now(), &|_| {}).unwrap();
        assert_eq!(rep.strays, 0, "{LOCK_DIR} was counted as a stray");
        assert!(locks.is_dir(), "gc removed the lock directory");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// An unrecognised child is counted and KEPT — a reclamation pass should not
    /// be guessing about a name it does not know.
    #[test]
    fn gc_reports_strays_without_deleting_them() {
        let (root, _dir, _) = gc_fixture("stray", Path::new("/tmp/c2rs-gc-absent-source.cpp"));
        let stray = root.join("NOTAKEY-README");
        std::fs::write(&stray, b"hello\n").unwrap();
        let rep = gc(&root, &gc_now(), &|_| {}).unwrap();
        assert_eq!(rep.strays, 1);
        assert!(rep.stray_names.iter().any(|s| s == "NOTAKEY-README"));
        assert!(stray.exists(), "gc deleted a stray it did not recognise");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_hash_collision_would_read_as_a_miss() {
        // read_entry demands the stored material equal the requested material,
        // so an entry written under other inputs is never served. Built from a
        // fully hittable entry so the ONLY reason it can miss is the material.
        let dir = hittable_entry("cachecol");
        assert!(matches!(read_entry(&dir, b"material-A"), EntryRead::Hit(_)));
        assert!(matches!(read_entry(&dir, b"material-B"), EntryRead::Miss));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// **Board #1388's guard, fired on purpose.** An entry that records a
    /// capture path other than the one it is being served from is REFUSED —
    /// never served — because the reference obj embeds that path as
    /// `S_OBJNAME` and serving it fakes a `mismatch` on a byte-exact TU.
    ///
    /// Toolchain-free: this is a property of the `meta` section, not of `c2`.
    #[test]
    fn an_entry_recorded_at_another_path_is_refused_not_served() {
        let dir = hittable_entry("cacheprov");
        let mut b = IlBundle::new("_CL_1");
        b.set("ex", b"EXEX".to_vec());
        let meta = |objpath: Option<&str>| {
            let mut m = format!("{CACHE_FORMAT}\nbase _CL_1\narg -il\n");
            if let Some(p) = objpath {
                m.push_str(&format!("{CAPTURE_PATH_KEY}{p}\n"));
            }
            put_blob(&dir, b"material-A", &m, &b);
        };

        // Recorded somewhere else -> Foreign, and the message NAMES both paths.
        meta(Some("/somewhere/else/out.obj"));
        let what = match read_entry(&dir, b"material-A") {
            EntryRead::Foreign(w) => w,
            other => panic!("a foreign entry read as {other:?} — it must be REFUSED"),
        };
        assert!(what.contains("/somewhere/else/out.obj"), "{what}");
        assert!(what.contains(&dir.join("out.obj").display().to_string()), "{what}");

        // Recorded HERE -> served. This is the arm that would catch the check
        // refusing everything, which would turn the cache into a permanent cold
        // start rather than a wrong answer.
        meta(Some(&dir.join("out.obj").display().to_string()));
        assert!(
            matches!(read_entry(&dir, b"material-A"), EntryRead::Hit(_)),
            "an entry recorded at its own path must be served"
        );

        // **Inverted at v2.** A missing line used to mean "pre-#1388 entry,
        // serve it anyway". After the hard invalidation every entry carries the
        // line, so that arm had provably zero members — dead code that read as a
        // tested branch. A missing line is now a Miss, and this asserts it
        // against an entry that is otherwise complete.
        meta(None);
        assert!(
            matches!(read_entry(&dir, b"material-A"), EntryRead::Miss),
            "an entry with no provenance line must not be served"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A written entry records where it was captured, so the guard above has
    /// something to check. Without this, `write_entry` could stop emitting the
    /// line and every entry would read as a Miss — a permanently cold cache,
    /// which is slow rather than wrong and so goes unnoticed.
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
        std::fs::write(dir.join("out.obj"), b"OOOOOOOOOOOO").unwrap();
        write_entry(&dir, b"material-A", &cap).unwrap();
        let raw = std::fs::read(dir.join(ENTRY_BLOB)).unwrap();
        let meta = c2_il::decode_entry(&raw).expect("production wrote a decodable blob").meta;
        assert!(
            meta.lines().any(|l| l
                == format!("{CAPTURE_PATH_KEY}{}", dir.join("out.obj").display())),
            "write_entry did not record the capture path; meta was:\n{meta}"
        );
        // And it round-trips all the way back to a Hit.
        assert!(matches!(read_entry(&dir, b"material-A"), EntryRead::Hit(_)));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// **The completion marker.** v1's was "`meta.txt` exists", written last —
    /// and `fs::write` is create+truncate+write_all, not atomic, so a crash
    /// after two lines landed left a parseable `base`+`arg` pair, i.e. a Hit
    /// with a truncated argv. v2's marker is the blob's own `TOTAL_LEN` and
    /// digest, so every one of these is a Miss rather than a partial answer.
    #[test]
    fn a_torn_blob_reads_as_a_miss() {
        let dir = hittable_entry("cachetorn");
        let good = std::fs::read(dir.join(ENTRY_BLOB)).unwrap();
        assert!(
            matches!(read_entry(&dir, b"material-A"), EntryRead::Hit(_)),
            "the fixture must be servable, or these cases prove nothing"
        );

        let cases: Vec<(&str, Box<dyn Fn(&mut Vec<u8>)>)> = vec![
            ("truncated to 60%", Box::new(|b: &mut Vec<u8>| b.truncate(b.len() * 3 / 5))),
            ("truncated to the header", Box::new(|b: &mut Vec<u8>| b.truncate(20))),
            ("flipped payload byte", Box::new(|b: &mut Vec<u8>| { let n = b.len() - 1; b[n] ^= 0xFF; })),
            ("corrupt magic", Box::new(|b: &mut Vec<u8>| b[0] = b'X')),
            ("version 1", Box::new(|b: &mut Vec<u8>| b[8..12].copy_from_slice(&1u32.to_le_bytes()))),
            ("lying TOTAL_LEN", Box::new(|b: &mut Vec<u8>| {
                let n = b.len() as u64 + 8;
                b[16..24].copy_from_slice(&n.to_le_bytes());
            })),
            ("section offset past the end", Box::new(|b: &mut Vec<u8>| {
                b[c2_il::cachefmt::HEADER_LEN + 8..c2_il::cachefmt::HEADER_LEN + 16]
                    .copy_from_slice(&u64::MAX.to_le_bytes());
            })),
            ("appended slack", Box::new(|b: &mut Vec<u8>| b.push(0))),
        ];
        for (what, break_it) in cases {
            let mut bytes = good.clone();
            break_it(&mut bytes);
            std::fs::write(dir.join(ENTRY_BLOB), &bytes).unwrap();
            assert!(
                matches!(read_entry(&dir, b"material-A"), EntryRead::Miss),
                "a blob {what} must read as a Miss, not as a partial entry"
            );
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The obj is the one thing the fold cannot absorb — it is the `-Fo` target
    /// and its own path is inside its bytes. An entry with a valid blob and no
    /// obj is not servable.
    #[test]
    fn a_blob_without_its_obj_reads_as_a_miss() {
        let dir = hittable_entry("cachenoobj");
        std::fs::remove_file(dir.join("out.obj")).unwrap();
        assert!(matches!(read_entry(&dir, b"material-A"), EntryRead::Miss));
        std::fs::write(dir.join("out.obj"), b"").unwrap();
        assert!(
            matches!(read_entry(&dir, b"material-A"), EntryRead::Miss),
            "an empty obj is not a capture"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// After a crash between the rename and the IL deletes, an entry can carry
    /// both a valid blob and the `_CL_*` files it superseded. The blob is
    /// authoritative from the rename onward; the orphans must not change what is
    /// served, or a stale stream could be read back in place of a fresh one.
    #[test]
    fn an_orphan_cl_file_beside_a_valid_blob_does_not_change_the_read() {
        let dir = hittable_entry("cacheorphan");
        let before = match read_entry(&dir, b"material-A") {
            EntryRead::Hit(h) => h,
            other => panic!("fixture must be a Hit, got {other:?}"),
        };
        std::fs::write(dir.join("_CL_1ex"), b"STALE-STALE").unwrap();
        std::fs::write(dir.join("_CL_1gl"), b"STALE").unwrap();
        let after = match read_entry(&dir, b"material-A") {
            EntryRead::Hit(h) => h,
            other => panic!("orphans turned a Hit into {other:?}"),
        };
        assert_eq!(before.bundle.files, after.bundle.files);
        assert_eq!(after.bundle.get("ex"), Some(&b"EXEX"[..]));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The inode win, asserted rather than assumed: a completed entry is the
    /// directory, `out.obj` and `entry.bin` — the five IL files the capture left
    /// behind are gone.
    #[test]
    fn write_entry_folds_the_il_files_away() {
        let dir = std::env::temp_dir().join(format!("c2rs-cachefold-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let mut b = IlBundle::new("_CL_test");
        for s in IL_SUFFIXES {
            b.set(s, format!("payload-{s}").into_bytes());
            // What `cl.exe` left on disk, which `write_entry` must clear.
            std::fs::write(dir.join(format!("_CL_test{s}")), format!("payload-{s}")).unwrap();
        }
        std::fs::write(dir.join("out.obj"), b"OOOOOOOOOOOO").unwrap();
        let cap = CapturedReference {
            bundle: b.clone(),
            base_name: "_CL_test".to_string(),
            c2_argv: vec!["-il".to_string()],
            ref_obj: ObjImage::new(b"OOOOOOOOOOOO".to_vec()),
            ref_obj_path: dir.join("out.obj"),
        };
        write_entry(&dir, b"material-A", &cap).unwrap();

        let mut names: Vec<String> = std::fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        assert_eq!(names, vec![ENTRY_BLOB.to_string(), "out.obj".to_string()]);

        // …and every stream survived the fold, byte for byte.
        match read_entry(&dir, b"material-A") {
            EntryRead::Hit(h) => assert_eq!(h.bundle.files, b.files),
            other => panic!("a folded entry must still be servable, got {other:?}"),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The validator compares two captures field by field, so if the fold lost
    /// or reordered anything it consumes, `--validate-cache` would start
    /// reporting poison on a healthy cache. Round-trip through the blob and
    /// assert the verdict is still `Identical`.
    #[test]
    fn the_fold_preserves_compare_captures_semantics() {
        let dir = std::env::temp_dir().join(format!("c2rs-cachecmp-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("out.obj"), b"OOOOOOOOOOOO").unwrap();
        let mut b = IlBundle::new("_CL_1");
        for s in IL_SUFFIXES {
            b.set(s, format!("payload-{s}").into_bytes());
        }
        let original = CapturedReference {
            bundle: b,
            base_name: "_CL_1".to_string(),
            c2_argv: vec!["-il".into(), "/t/_CL_1".into(), "-Ox".into()],
            ref_obj: ObjImage::new(b"OOOOOOOOOOOO".to_vec()),
            ref_obj_path: dir.join("out.obj"),
        };
        write_entry(&dir, b"material-A", &original).unwrap();
        let round_tripped = match read_entry(&dir, b"material-A") {
            EntryRead::Hit(h) => *h,
            other => panic!("expected a Hit, got {other:?}"),
        };
        assert_eq!(
            compare_captures(&original, &round_tripped),
            CaptureDiff::Identical
        );
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
