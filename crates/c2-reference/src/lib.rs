//! `c2-reference` — drives the **real** MSVC toolchain (`cl.exe` + `c2.dll`)
//! under [wibo] as the differential oracle. Mechanics ported from
//! `dc3-decomp/tools/compiler_trace/invoker.py` (path conversion, base command)
//! and `dc3-decomp/msvc-src/tools/il_parser.py` (`capture_il`, the `-il` scrape,
//! the 5-file bundle layout).
//!
//! wibo is a PE **loader**, not an emulator: it runs the Windows x86 `cl.exe`
//! natively and maps `Z:\` to host `/`. Two operations matter here:
//!
//! * [`Toolchain::compile_obj`] — a normal `/Ox /GS- /c` compile → real `.obj`.
//! * [`Toolchain::capture_il`] — a `/Bd /d2nop` compile that makes c2 abort
//!   *before* it deletes the temp `_CL_*` IL bundle, so the 5 files survive.
//!
//! # P0.1 standalone-c2 replay — IMPLEMENTED and byte-exact-verified
//!
//! Feeding an **unmodified** captured IL bundle back through `c2.dll` *alone*
//! (no front-end) reproduces the pipeline `.obj` **byte-for-byte** — verified
//! RAW-identical (COFF `TimeDateStamp` included) on all three bundled fixtures.
//!
//! > **Measured correction, 2026-07-30.** That raw identity is real but it is
//! > measured **back-to-back within one second**, and the timestamp is *not*
//! > pinned by wibo as this doc used to say: the 878 objs of one cold gap scan
//! > carry **58 distinct `TimeDateStamp` values**, monotone across the scan's
//! > 5-minute window and readable as ordinary Unix seconds. Two captures of the
//! > same TU minutes apart therefore differ in bytes 4..8 and **nowhere else**
//! > (51 of 51 sampled re-captures, `c2rs gap --validate-cache`). Nothing in the
//! > project's criterion moves — it zeroes those four bytes by definition — but
//! > "raw-identical" is a claim about two runs in the same second, not about the
//! > pipeline being deterministic in that field.
//!
//! The mechanism is the [`c2host`](../c2host) x86 stub that
//! `LoadLibraryA`s `c2.dll` and calls its stdcall export `_InvokeCompilerPass@12`
//! with the reconstructed `-il <base> … -Fo <obj>` argv:
//!
//! * [`Toolchain::capture_reference`] — one `/Bd` compile under `strace` (with
//!   `unlink` inject-to-no-op) that runs c2 *for real* (producing the reference
//!   obj) **and** keeps the `_CL_*` bundle, echoing the exact c2 argv.
//! * [`Toolchain::replay`] — writes the captured bundle back out and re-runs
//!   `c2.dll` on it through `c2host`, swapping only `-il`/`-Fo`.
//!
//! [`ReferenceC2`] wraps a toolchain as a [`Backend`]: its `compile` now
//! **really** drives standalone c2 on an IL bundle (a fixed DC3 argv template),
//! so it is a pure function of the bundle. Byte-equality to a *specific* pipeline
//! obj additionally requires matching `-f`/`-Fo`/backend flags — which is why the
//! differential self-check uses [`Toolchain::replay`] with the *captured* argv
//! rather than this default template.
//!
//! The replay path additionally needs `strace` (to keep the bundle) and
//! `i686-w64-mingw32-gcc` (to build `c2host`); [`Toolchain::has_strace`] /
//! [`Toolchain::has_mingw`] guard it so callers skip cleanly when either is
//! absent. The core toolchain ([`Toolchain::locate`]) does not depend on them.
//!
//! # P-F0.1 standalone-c1 (front-end) replay — IMPLEMENTED and byte-exact
//!
//! The same trick runs one stage earlier. Driving `c1xx.dll` (the C++ *front
//! end*) alone through the [`c1host`](../c1host) stub reproduces the captured
//! `_CL_*` IL **bundle byte-for-byte** on the fixtures — the front-end analogue
//! of the P0.1 back-end proof. [`Toolchain::capture_c1_reference`] keeps the
//! bundle + echoes the exact c1xx argv (no `strace` needed — the front end
//! finishes before c2 aborts), and [`Toolchain::replay_c1`] re-runs the front
//! end to a fresh `-il` base. This needs only `i686-w64-mingw32-gcc`
//! ([`Toolchain::has_mingw`]) and `c1xx.dll` ([`Toolchain::has_c1xx`]).
//!
//! [wibo]: https://github.com/decompals/wibo

use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use c2_core::{Backend, BackendError, IlBundle, ObjImage};

/// Repo root = this crate's `CARGO_MANIFEST_DIR` joined `../..`
/// (`.../c2-rs/crates/c2-reference` → `.../c2-rs`). All default toolchain paths
/// are resolved relative to it, so nothing absolute is baked into source.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Value of env var `var`, or `root/default_rel` if unset.
fn env_or(root: &Path, var: &str, default_rel: &str) -> PathBuf {
    match std::env::var_os(var) {
        Some(v) => PathBuf::from(v),
        None => root.join(default_rel),
    }
}

/// The oldest wibo release known to run the capture path correctly.
///
/// Two wibo behaviours the harness depends on landed in `1.0.1-23`:
/// `WIBO_KEEP_TEMP` (without it the reaper deletes the `_CL_*` quintet the
/// moment the `/d2nop` guest dies — i.e. exactly our product) and the
/// `lstrcpynA` shim that keeps c2 alive on large TUs. An older loader does not
/// fail loudly: it turns the gap scan's replay column from `36 checked / 0
/// diverged` into `36/30` while the census and the mismatch count stay
/// byte-identical — a **fake correctness alarm on the oracle seam**. That is
/// why [`Toolchain::wibo_stale`] exists and why the scan prints its resolution.
pub const WIBO_KNOWN_GOOD: &str = "1.0.1-23";

/// Toolchain version dir inside a compilers root — the layout of the decomp.dev
/// compilers archive (`https://files.decomp.dev/compilers_<tag>.zip`), which
/// unzips to `X360/16.00.11886.00/{cl.exe, c1xx.dll, c2.dll, ...}`.
const X360_TOOLCHAIN_REL: &str = "X360/16.00.11886.00";

/// Resolve the compilers root directory. Precedence:
///
/// 1. `C2RS_COMPILERS` env (taken verbatim, even if the dir is missing — an
///    explicit override should fail loudly, not silently fall back);
/// 2. `<repo>/compilers` if it contains the X360 toolchain dir (populate it
///    with `scripts/fetch_compilers.sh`);
/// 3. `../dc3-decomp/build/compilers` if *it* contains the toolchain dir
///    (compat with the original sibling-repo layout);
/// 4. `<repo>/compilers` as the fallthrough, so "toolchain absent" messages
///    point at the canonical place to put it.
fn compilers_root(root: &Path) -> PathBuf {
    if let Some(v) = std::env::var_os("C2RS_COMPILERS") {
        return PathBuf::from(v);
    }
    let local = root.join("compilers");
    if local.join(X360_TOOLCHAIN_REL).is_dir() {
        return local;
    }
    let sibling = root.join("../dc3-decomp/build/compilers");
    if sibling.join(X360_TOOLCHAIN_REL).is_dir() {
        return sibling;
    }
    local
}

/// Located real toolchain. All paths are host paths (wibo takes a host path for
/// `cl.exe`; only *source*/*output* arguments get `Z:\` conversion).
#[derive(Clone, Debug)]
pub struct Toolchain {
    /// wibo release loader (64-bit).
    pub wibo: PathBuf,
    /// wibo debug loader (32-bit) — optional; used by later replay probes.
    pub wibo_debug: PathBuf,
    /// `cl.exe` driver.
    pub cl_exe: PathBuf,
    /// `c2.dll` back-end (the thing being ported; driven by the replay path).
    pub c2_dll: PathBuf,
    /// `c2host.c` source (repo `c2host/c2host.c`) — built on demand for replay.
    pub c2host_src: PathBuf,
    /// Built `c2host.exe` cache path (gitignored; env `C2RS_C2HOST`, default
    /// `<repo>/target/c2host/c2host.exe`). Never committed.
    pub c2host_exe: PathBuf,
    /// `c1xx.dll` C++ **front end** — driven by the standalone-c1 replay path
    /// (P-F0.1). Sibling of `c2.dll` in the toolchain dir.
    pub c1xx_dll: PathBuf,
    /// `c1host.c` source (repo `c1host/c1host.c`) — built on demand for c1 replay.
    pub c1host_src: PathBuf,
    /// Built `c1host.exe` cache path (gitignored; env `C2RS_C1HOST`, default
    /// `<repo>/target/c1host/c1host.exe`). Never committed.
    pub c1host_exe: PathBuf,
    /// `strace` binary, if found on `PATH` — REQUIRED for the capture path
    /// (its `unlink` inject keeps the `_CL_*` bundle from being deleted).
    pub strace: Option<PathBuf>,
    /// `i686-w64-mingw32-gcc`, if found on `PATH` — builds the x86 `c2host` stub.
    pub mingw: Option<PathBuf>,
}

/// Find an executable `name` on `PATH` (mirrors a minimal `which`). Returns the
/// first `PATH` entry that contains an existing `name`.
fn find_on_path(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let cand = dir.join(name);
        if cand.exists() {
            return Some(cand);
        }
    }
    None
}

impl Toolchain {
    /// Locate the toolchain from env overrides (`C2RS_WIBO`, `C2RS_WIBO_DEBUG`,
    /// `C2RS_COMPILERS`, `C2RS_CL_EXE`, `C2RS_C2_DLL`, `C2RS_C1XX_DLL`) with
    /// relative-to-repo-root defaults. The compiler binaries default to
    /// `<compilers root>/X360/16.00.11886.00/` (see [`compilers_root`]); wibo
    /// defaults to the sibling `../wibo` build tree, falling back to `wibo` on
    /// `PATH`. Returns `None` if any *required* path (wibo, cl.exe, c2.dll) is
    /// missing, so callers can skip cleanly when the toolchain is absent.
    pub fn locate() -> Option<Toolchain> {
        let root = repo_root();
        let toolchain_dir = compilers_root(&root).join(X360_TOOLCHAIN_REL);
        let wibo = match std::env::var_os("C2RS_WIBO") {
            Some(v) => PathBuf::from(v),
            None => {
                let sibling = root.join("../wibo/build/release/wibo");
                if sibling.exists() {
                    sibling
                } else {
                    find_on_path("wibo").unwrap_or(sibling)
                }
            }
        };
        let tc = Toolchain {
            wibo,
            wibo_debug: env_or(&root, "C2RS_WIBO_DEBUG", "../wibo/build/debug/wibo"),
            cl_exe: match std::env::var_os("C2RS_CL_EXE") {
                Some(v) => PathBuf::from(v),
                None => toolchain_dir.join("cl.exe"),
            },
            c2_dll: match std::env::var_os("C2RS_C2_DLL") {
                Some(v) => PathBuf::from(v),
                None => toolchain_dir.join("c2.dll"),
            },
            c2host_src: root.join("c2host/c2host.c"),
            c2host_exe: env_or(&root, "C2RS_C2HOST", "target/c2host/c2host.exe"),
            c1xx_dll: match std::env::var_os("C2RS_C1XX_DLL") {
                Some(v) => PathBuf::from(v),
                None => toolchain_dir.join("c1xx.dll"),
            },
            c1host_src: root.join("c1host/c1host.c"),
            c1host_exe: env_or(&root, "C2RS_C1HOST", "target/c1host/c1host.exe"),
            // strace / mingw are needed only for the replay path; their absence
            // does NOT block locate() — callers guard via has_strace/has_mingw.
            strace: std::env::var_os("C2RS_STRACE")
                .map(PathBuf::from)
                .or_else(|| find_on_path("strace")),
            mingw: std::env::var_os("C2RS_MINGW")
                .map(PathBuf::from)
                .or_else(|| find_on_path("i686-w64-mingw32-gcc")),
        };
        // Required for any real work. wibo_debug / strace / mingw are optional
        // (the last two only gate the replay path, not the core toolchain).
        if tc.wibo.exists() && tc.cl_exe.exists() && tc.c2_dll.exists() {
            Some(tc)
        } else {
            None
        }
    }

    /// Run `<wibo> --version` and return its first line verbatim, e.g.
    /// `wibo 1.0.1-23-g4a9dd6f (Linux x86_64)`. `None` when the binary cannot be
    /// executed or prints nothing — **never** an error: toolchain location is
    /// env-driven by design and an unparseable loader must not fail a scan.
    pub fn wibo_version(&self) -> Option<String> {
        let out = Command::new(&self.wibo).arg("--version").output().ok()?;
        let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
        if text.trim().is_empty() {
            text = String::from_utf8_lossy(&out.stderr).into_owned();
        }
        let line = text.lines().next()?.trim().to_string();
        if line.is_empty() {
            None
        } else {
            Some(line)
        }
    }

    /// `Some(true)` iff the resolved wibo is **older** than [`WIBO_KNOWN_GOOD`];
    /// `Some(false)` if it is that release or newer; `None` when the version
    /// string is absent or does not parse (unknown, not "fine").
    pub fn wibo_stale(&self) -> Option<bool> {
        let line = self.wibo_version()?;
        let have = parse_wibo_version(&line)?;
        let want = parse_wibo_version(WIBO_KNOWN_GOOD)?;
        Some(have < want)
    }

    /// True iff `strace` is available (required for the replay-capture path).
    pub fn has_strace(&self) -> bool {
        self.strace.as_ref().map(|p| p.exists()).unwrap_or(false)
    }

    /// True iff `i686-w64-mingw32-gcc` is available (builds the `c2host` stub).
    pub fn has_mingw(&self) -> bool {
        self.mingw.as_ref().map(|p| p.exists()).unwrap_or(false)
    }

    /// True iff `c1xx.dll` (the C++ front end) is present — required for the
    /// standalone-c1 replay path (P-F0.1).
    pub fn has_c1xx(&self) -> bool {
        self.c1xx_dll.exists()
    }

    /// Build `c2host.exe` from `c2host/c2host.c` into the gitignored cache if it
    /// is missing or older than the source. Returns the exe path.
    ///
    /// Build command (x86 Windows PE, runs under wibo):
    /// `i686-w64-mingw32-gcc -static -static-libgcc -O2 -o <exe> <src>`.
    /// Errors clearly if mingw is absent or the source is missing.
    pub fn ensure_c2host(&self) -> io::Result<PathBuf> {
        let mingw = self.mingw.clone().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "i686-w64-mingw32-gcc not found on PATH (or C2RS_MINGW) — cannot \
                 build the c2host x86 stub required for standalone-c2 replay",
            )
        })?;
        if !self.c2host_src.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("c2host source missing: {}", self.c2host_src.display()),
            ));
        }
        let needs_build = match (
            std::fs::metadata(&self.c2host_exe).and_then(|m| m.modified()),
            std::fs::metadata(&self.c2host_src).and_then(|m| m.modified()),
        ) {
            (Ok(exe_t), Ok(src_t)) => exe_t < src_t, // rebuild if stale
            _ => true,                               // exe missing → build
        };
        if needs_build {
            build_host_stub(&mingw, &self.c2host_src, &self.c2host_exe, "c2host")?;
        }
        // c2 resolves `<host-exe-dir>/1033/clui.dll` for diagnostics; without
        // it, any TU that triggers a warning dies with `fatal error C1510`.
        self.ensure_clui_beside(&self.c2host_exe, &self.c2_dll)?;
        Ok(self.c2host_exe.clone())
    }

    /// Build `c1host.exe` from `c1host/c1host.c` into the gitignored cache if it
    /// is missing or older than the source, **and** ensure the `1033` resources
    /// symlink sits next to it (c1xx resolves `<host-exe-dir>/1033/clui.dll` via
    /// `GetModuleFileNameW(NULL)` — a missing `1033` makes it abort silently).
    /// Returns the exe path.
    ///
    /// Same x86 build recipe as [`Toolchain::ensure_c2host`]; c1host converts its
    /// argv to UTF-16 internally, so no `-municode` is needed.
    pub fn ensure_c1host(&self) -> io::Result<PathBuf> {
        let mingw = self.mingw.clone().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "i686-w64-mingw32-gcc not found on PATH (or C2RS_MINGW) — cannot \
                 build the c1host x86 stub required for standalone-c1 replay",
            )
        })?;
        if !self.c1host_src.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("c1host source missing: {}", self.c1host_src.display()),
            ));
        }
        let needs_build = match (
            std::fs::metadata(&self.c1host_exe).and_then(|m| m.modified()),
            std::fs::metadata(&self.c1host_src).and_then(|m| m.modified()),
        ) {
            (Ok(exe_t), Ok(src_t)) => exe_t < src_t,
            _ => true,
        };
        if needs_build {
            build_host_stub(&mingw, &self.c1host_src, &self.c1host_exe, "c1host")?;
        }
        self.ensure_c1_resources()?;
        Ok(self.c1host_exe.clone())
    }

    /// Ensure `<c1host_exe dir>/1033` points at the toolchain's `1033` resources
    /// dir (holding `clui.dll`). c1xx locates its diagnostics resources relative
    /// to the running exe, which under wibo is `c1host.exe`.
    fn ensure_c1_resources(&self) -> io::Result<()> {
        self.ensure_clui_beside(&self.c1host_exe, &self.c1xx_dll)
    }

    /// Symlink the toolchain's `1033/` resources dir (holding `clui.dll`)
    /// beside `host_exe`. The compiler DLLs resolve `<host-exe-dir>/1033/
    /// clui.dll` via `GetModuleFileName(NULL)` the moment they need to *print
    /// a diagnostic* — without it, c1xx silently aborts and c2 dies with
    /// `fatal error C1510` on the first TU that triggers any warning.
    fn ensure_clui_beside(&self, host_exe: &Path, dll: &Path) -> io::Result<()> {
        let exe_dir = host_exe
            .parent()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "host exe has no parent"))?;
        let src_1033 = dll
            .parent()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "compiler dll has no parent"))?
            .join("1033");
        if !src_1033.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("toolchain resources dir missing: {}", src_1033.display()),
            ));
        }
        let link = exe_dir.join("1033");
        let src_abs = absolute(&src_1033)?;
        // Recreate only if absent or pointing elsewhere (idempotent, no churn).
        let ok = std::fs::read_link(&link)
            .ok()
            .map(|t| t == src_abs)
            .unwrap_or(false);
        if !ok {
            // `remove` + `symlink` is NOT safe with a concurrent reader: between
            // the two calls there is no `1033` at all, and c2/c1xx answer a
            // missing one with `fatal error C1510` — and a *second* process in
            // the same window gets `EEXIST` from its own `symlink` and errors
            // out. Build the link under a private name and `rename` it into
            // place instead: rename over an existing path is atomic, so every
            // concurrent reader sees either the old link or the new one.
            let tmp = exe_dir.join(format!(
                ".1033.{}.{}.tmp",
                std::process::id(),
                SCRATCH_COUNTER.fetch_add(1, Ordering::Relaxed)
            ));
            let _ = std::fs::remove_file(&tmp);
            symlink_dir(&src_abs, &tmp)?;
            if let Err(e) = std::fs::rename(&tmp, &link) {
                let _ = std::fs::remove_file(&tmp);
                // A real directory (rather than a symlink) at `link` refuses the
                // rename; so does a racing peer on some filesystems. Either is
                // fine as long as what is there now resolves to the resources.
                if !link.join("clui.dll").exists() {
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    /// Normal compile: `/Ox /GS- /c` → a real `.obj`. Returns its bytes.
    ///
    /// Success is detected by presence of the output file (exit code is checked
    /// as a secondary signal). On failure the exact stderr is included in the
    /// error — never papered over.
    pub fn compile_obj(&self, cpp: &Path, out_obj: &Path) -> io::Result<ObjImage> {
        if let Some(parent) = out_obj.parent() {
            std::fs::create_dir_all(parent)?;
        }
        // Stale output from a previous run would mask a failure — remove it.
        let _ = std::fs::remove_file(out_obj);

        let z_src = to_wibo_path(&absolute(cpp)?);
        let z_obj = to_wibo_path(&absolute(out_obj)?);

        let output = Command::new(&self.wibo)
            .arg(&self.cl_exe)
            .arg("/Ox")
            .arg("/GS-")
            .arg("/c")
            .arg(format!("/Fo{z_obj}"))
            .arg(&z_src)
            .env("WIBO_FS_CACHE", "1")
            .output()?;

        if !out_obj.exists() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "cl.exe produced no object at {}\n  status: {}\n  stdout:\n{}\n  stderr:\n{}",
                    out_obj.display(),
                    output.status,
                    indent(&String::from_utf8_lossy(&output.stdout)),
                    indent(&String::from_utf8_lossy(&output.stderr)),
                ),
            ));
        }
        Ok(ObjImage::new(std::fs::read(out_obj)?))
    }

    /// Build-faithful IL capture. Runs `/Bd /d2nop /Ox /GS- /c` so c2 aborts
    /// before deleting the temp `_CL_*` bundle. `TMP`/`TEMP` are pointed at
    /// `work_dir` so the bundle lands there deterministically.
    ///
    /// The `/Bd /d2nop` compile exits **non-zero** (c2 aborted) — that is
    /// SUCCESS. Success is detected by presence of the `_CL_*ex` file, not the
    /// exit code. The bundle base is scraped from the `-il <...>_CL_<hash>`
    /// token in the compiler's stdout/stderr.
    pub fn capture_il(&self, cpp: &Path, work_dir: &Path) -> io::Result<IlBundle> {
        let z_src = to_wibo_path(&absolute(cpp)?);
        let flags: Vec<String> = ["/Ox", "/GS-", "/c"].iter().map(|s| s.to_string()).collect();
        self.capture_il_with(&z_src, work_dir, &flags, None)
    }

    /// [`Toolchain::capture_il`] generalized to an arbitrary compile profile —
    /// the seam for **real-workload front-end-only capture**: a project TU
    /// needs the project's own `/O1 /Oi /EHsc /I…` flags and a cwd inside the
    /// project so relative includes resolve.
    ///
    /// * `src_arg` is passed to `cl.exe` verbatim (a `Z:\…` path, or a path
    ///   relative to `cwd` — relative is build-faithful: it is what gets baked
    ///   into the `.gl` file and `.debug$S`).
    /// * `flags` replace the default `/Ox /GS- /c` (they should include `/c`);
    ///   `/Bd /d2nop` are always prepended by this method.
    /// * `cwd` is the working directory for the compile, if given.
    ///
    /// Unlike [`Toolchain::capture_reference_with`] this does **not** need
    /// `strace` and does **not** produce a reference obj: c2 is nop'd out, so
    /// the cost is the driver + front end only. That is what makes it usable as
    /// a per-candidate step rather than a second full compile.
    pub fn capture_il_with(
        &self,
        src_arg: &str,
        work_dir: &Path,
        flags: &[String],
        cwd: Option<&Path>,
    ) -> io::Result<IlBundle> {
        std::fs::create_dir_all(work_dir)?;
        let work_abs = absolute(work_dir)?;

        let z_obj = to_wibo_path(&work_abs.join("il_capture.obj"));

        let mut cmd = Command::new(&self.wibo);
        cmd.arg(&self.cl_exe).arg("/Bd").arg("/d2nop");
        for f in flags {
            cmd.arg(f);
        }
        cmd.arg(format!("/Fo{z_obj}"))
            .arg(src_arg)
            // Land the _CL_* bundle in our private work dir, deterministically.
            .env("TMP", &work_abs)
            .env("TEMP", &work_abs)
            .env("WIBO_FS_CACHE", "1")
            // wibo >= 1.0.1-23 reaps guest FILE_ATTRIBUTE_TEMPORARY files
            // (the _CL_* quintet) when the guest dies without cleaning up -
            // which is exactly how /d2nop capture works. The `_CL_*ex` bundle
            // IS our product; opt out of the reaper.
            .env("WIBO_KEEP_TEMP", "1");
        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }
        let output = cmd.output()?;
        // NOTE: non-zero exit is expected (c2 aborted on /d2nop). Do NOT treat
        // it as failure — the IL files are the success signal.

        let mut combined = String::from_utf8_lossy(&output.stdout).into_owned();
        combined.push('\n');
        combined.push_str(&String::from_utf8_lossy(&output.stderr));

        let (scraped_dir, base) = scrape_il_base(&combined, &work_abs).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "could not find `-il ..._CL_<hash>` in compiler output\n  status: {}\n  stdout:\n{}\n  stderr:\n{}",
                    output.status,
                    indent(&String::from_utf8_lossy(&output.stdout)),
                    indent(&String::from_utf8_lossy(&output.stderr)),
                ),
            )
        })?;

        // Prefer the scraped directory; fall back to the work dir (older wibo
        // builds emitted a bare `_CL_<hash>` with files in CWD/tmp).
        let mut bundle = IlBundle::load_from_dir(&scraped_dir, &base)?;
        if bundle.ex().map(|b| b.is_empty()).unwrap_or(true) {
            bundle = IlBundle::load_from_dir(&work_abs, &base)?;
        }

        match bundle.ex() {
            Some(ex) if !ex.is_empty() => Ok(bundle),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "IL capture produced no `{base}ex` file (searched {} and {})",
                    scraped_dir.display(),
                    work_abs.display()
                ),
            )),
        }
    }

    /// **P0.1 capture.** Run one `/Bd /Ox /GS- /c` pipeline compile under
    /// `strace` (with `unlink`/`unlinkat` injected to return 0) so c2 runs *for
    /// real* — producing the reference `.obj` at `<work_dir>/out.obj` — **and**
    /// the `_CL_*` IL bundle survives (cl would otherwise delete it). `/Bd`
    /// echoes the exact c1xx→c2 argv, which we capture and reconstruct for the
    /// replay.
    ///
    /// Returns everything the replay needs: the surviving bundle, its base name,
    /// the c2 argv tokens (verbatim, after the `…c2.dll` path token), the
    /// reference obj bytes, and the reference obj path.
    ///
    /// Requires `strace` — check [`Toolchain::has_strace`] first, or this errors.
    pub fn capture_reference(
        &self,
        cpp: &Path,
        work_dir: &Path,
    ) -> io::Result<CapturedReference> {
        let z_src = to_wibo_path(&absolute(cpp)?);
        let flags: Vec<String> =
            ["/Ox", "/GS-", "/c"].iter().map(|s| s.to_string()).collect();
        self.capture_reference_with(&z_src, work_dir, &flags, None)
    }

    /// [`Toolchain::capture_reference`] generalized to an arbitrary compile
    /// profile — the seam for **real-workload** capture (gap scans over actual
    /// project TUs, which need the project's own `/O1 /EHsc /I…` flags and a
    /// cwd inside the project so relative includes resolve).
    ///
    /// * `src_arg` is passed to `cl.exe` verbatim (a `Z:\…` path, or a path
    ///   relative to `cwd` — relative is build-faithful: it is what gets baked
    ///   into the `.gl` file and `.debug$S`).
    /// * `flags` replace the default `/Ox /GS- /c` (they should include `/c`).
    /// * `cwd` is the working directory for the compile, if given.
    ///
    /// The `/Bd` echo, strace `unlink` inject, TMP redirection, and bundle
    /// scrape are identical to the fixture path.
    pub fn capture_reference_with(
        &self,
        src_arg: &str,
        work_dir: &Path,
        flags: &[String],
        cwd: Option<&Path>,
    ) -> io::Result<CapturedReference> {
        let strace = self.strace.clone().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "strace not found on PATH (or C2RS_STRACE) — required to keep the \
                 _CL_* IL bundle alive during a real c2 compile",
            )
        })?;
        std::fs::create_dir_all(work_dir)?;
        let work_abs = absolute(work_dir)?;
        let out_obj = work_abs.join("out.obj");
        let _ = std::fs::remove_file(&out_obj);
        // Clear stale bundles so find_bundle_base cannot pick up a previous
        // capture in a reused work dir.
        for entry in std::fs::read_dir(&work_abs)? {
            let entry = entry?;
            if entry.file_name().to_string_lossy().starts_with("_CL_") {
                let _ = std::fs::remove_file(entry.path());
            }
        }

        let z_obj = to_wibo_path(&out_obj);

        // strace -f -e trace=unlink,unlinkat -e inject=unlink,unlinkat:retval=0
        //   -o /dev/null  <wibo> <cl.exe> /Bd <flags…> /Fo<z_obj> <src_arg>
        let mut cmd = Command::new(&strace);
        cmd.arg("-f")
            .arg("-e")
            .arg("trace=unlink,unlinkat")
            .arg("-e")
            .arg("inject=unlink,unlinkat:retval=0")
            .arg("-o")
            .arg("/dev/null")
            .arg(&self.wibo)
            .arg(&self.cl_exe)
            .arg("/Bd")
            .args(flags)
            .arg(format!("/Fo{z_obj}"))
            .arg(src_arg)
            .env("TMP", &work_abs)
            .env("TEMP", &work_abs)
            .env("WIBO_FS_CACHE", "1");
        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }
        let output = cmd.output()?;

        if !out_obj.exists() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "capture_reference produced no obj at {}\n  status: {}\n  stdout:\n{}\n  stderr:\n{}",
                    out_obj.display(),
                    output.status,
                    indent(&String::from_utf8_lossy(&output.stdout)),
                    indent(&String::from_utf8_lossy(&output.stderr)),
                ),
            ));
        }
        let ref_obj = ObjImage::new(std::fs::read(&out_obj)?);

        let mut combined = String::from_utf8_lossy(&output.stdout).into_owned();
        combined.push('\n');
        combined.push_str(&String::from_utf8_lossy(&output.stderr));

        let c2_argv = parse_c2_argv(&combined).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "could not find the `…c2.dll -il …` argv echo in compiler output\n  stdout:\n{}\n  stderr:\n{}",
                    indent(&String::from_utf8_lossy(&output.stdout)),
                    indent(&String::from_utf8_lossy(&output.stderr)),
                ),
            )
        })?;

        let base_name = find_bundle_base(&work_abs)?.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "no surviving `_CL_*ex` bundle in {}",
                    work_abs.display()
                ),
            )
        })?;
        let bundle = IlBundle::load_from_dir(&work_abs, &base_name)?;
        if bundle.ex().map(|b| b.is_empty()).unwrap_or(true) {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("captured bundle {base_name} has an empty/absent .ex"),
            ));
        }

        Ok(CapturedReference {
            bundle,
            base_name,
            c2_argv,
            ref_obj,
            ref_obj_path: out_obj,
        })
    }

    /// **P0.1 replay.** Write `captured.bundle` back out into `bundle_dir` and
    /// re-run `c2.dll` alone on it through `c2host`, reconstructing the captured
    /// c2 argv but swapping ONLY:
    /// * `-il <val>` → `-il <Z: path to our re-written bundle base>`, and
    /// * `-Fo<val>`  → `-Fo<Z: path to `out_obj`>`.
    ///
    /// Everything else (`-f <src>`, all `-Q*`/`-G*` flags, `-Bd`, `-Og`, `-Ob2`)
    /// is kept verbatim. Returns the produced obj.
    ///
    /// For an apples-to-apples byte compare against a captured reference, pass
    /// `out_obj = &captured.ref_obj_path` so the embedded `/Fo` string is
    /// identical (the caller reads `captured.ref_obj` into memory first, then
    /// this overwrites that path). Requires `c2host` (mingw) — builds it on
    /// demand via [`Toolchain::ensure_c2host`].
    pub fn replay(
        &self,
        captured: &CapturedReference,
        bundle_dir: &Path,
        out_obj: &Path,
    ) -> io::Result<ObjImage> {
        let (mut cmd, out_abs) = self.build_replay_command(captured, bundle_dir, out_obj)?;
        let output = cmd.output()?;

        // An empty obj means c2 opened the output then died mid-pass (e.g. a
        // missing wibo import) — that is a crash, not a product.
        if !out_abs.exists() || std::fs::metadata(&out_abs)?.len() == 0 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "replay produced no (or an empty) obj at {}\n  status: {}\n  stdout:\n{}\n  stderr:\n{}",
                    out_abs.display(),
                    output.status,
                    indent(&String::from_utf8_lossy(&output.stdout)),
                    indent(&String::from_utf8_lossy(&output.stderr)),
                ),
            ));
        }
        Ok(ObjImage::new(std::fs::read(&out_abs)?))
    }

    /// **Timeout-bounded replay.** Same as [`Toolchain::replay`], but kills the
    /// c2 process and returns [`io::ErrorKind::TimedOut`] if it does not finish
    /// within `timeout`. Required for K3-edit replays: P0.6a proved a malformed
    /// `.gl`/`.ex` function-set can make c2 **hang** rather than crash — a bounded
    /// replay turns that hang into a clean, reportable failure. A SIGSEGV / abort
    /// (e.g. a stale `.gl` offset) leaves no obj and returns an `Other` error.
    pub fn replay_within(
        &self,
        captured: &CapturedReference,
        bundle_dir: &Path,
        out_obj: &Path,
        timeout: Duration,
    ) -> io::Result<ObjImage> {
        let (mut cmd, out_abs) = self.build_replay_command(captured, bundle_dir, out_obj)?;
        cmd.stdout(Stdio::null()).stderr(Stdio::null());
        let mut child = cmd.spawn()?;
        let start = Instant::now();
        loop {
            match child.try_wait()? {
                Some(_status) => break,
                None => {
                    if start.elapsed() >= timeout {
                        let _ = child.kill();
                        let _ = child.wait();
                        return Err(io::Error::new(
                            io::ErrorKind::TimedOut,
                            format!(
                                "standalone-c2 replay exceeded {timeout:?} (likely a \
                                 .gl/.ex function-set mismatch hang — P0.6a G)"
                            ),
                        ));
                    }
                    std::thread::sleep(Duration::from_millis(20));
                }
            }
        }
        if !out_abs.exists() || std::fs::metadata(&out_abs)?.len() == 0 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "replay produced no (or an empty) obj at {} (c2 crashed/aborted — \
                     e.g. a stale .gl offset SIGSEGV, P0.6a C)",
                    out_abs.display()
                ),
            ));
        }
        Ok(ObjImage::new(std::fs::read(&out_abs)?))
    }

    /// Build the `wibo c2host c2.dll c2.dll <argv…>` [`Command`] for a replay,
    /// writing `captured.bundle` to `bundle_dir` and reconstructing the captured
    /// c2 argv with only `-il` / `-Fo` swapped. Shared by [`Toolchain::replay`]
    /// and [`Toolchain::replay_within`]. Returns the command and the absolute
    /// output-obj path (already removed if it existed).
    fn build_replay_command(
        &self,
        captured: &CapturedReference,
        bundle_dir: &Path,
        out_obj: &Path,
    ) -> io::Result<(Command, PathBuf)> {
        let c2host = self.ensure_c2host()?;

        std::fs::create_dir_all(bundle_dir)?;
        let bundle_dir_abs = absolute(bundle_dir)?;
        captured
            .bundle
            .write_to_dir(&bundle_dir_abs, &captured.base_name)?;
        let base = bundle_dir_abs.join(&captured.base_name);
        let z_il = to_wibo_path(&base);

        if let Some(parent) = out_obj.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let _ = std::fs::remove_file(out_obj);
        let out_abs = absolute(out_obj)?;
        let z_fo = to_wibo_path(&out_abs);

        // Reconstruct the c2 argv, swapping -il and -Fo only.
        let mut argv: Vec<String> = Vec::with_capacity(captured.c2_argv.len());
        let mut i = 0;
        while i < captured.c2_argv.len() {
            let t = &captured.c2_argv[i];
            if t == "-il" {
                argv.push("-il".to_string());
                argv.push(z_il.clone());
                i += 2; // skip the flag and its (old) value
                continue;
            }
            if let Some(stripped) = t.strip_prefix("-Fo") {
                let _ = stripped;
                argv.push(format!("-Fo{z_fo}"));
                i += 1;
                continue;
            }
            argv.push(t.clone());
            i += 1;
        }

        // wibo c2host <c2.dll (LoadLibrary)> <c2.dll (argv0)> <reconstructed argv…>
        let mut cmd = Command::new(&self.wibo);
        cmd.arg(&c2host).arg(&self.c2_dll).arg(&self.c2_dll);
        for a in &argv {
            cmd.arg(a);
        }
        cmd.env("WIBO_FS_CACHE", "1").current_dir(&bundle_dir_abs);
        Ok((cmd, out_abs))
    }

    /// **P-F0.1 front-end capture.** One `/Bd /d2nop /Ox /GS- /c` compile: c2
    /// aborts (`/d2nop`) *before* the `_CL_*` IL bundle is deleted, so the
    /// front-end output survives, and `/Bd` echoes the exact `c1xx.dll` argv.
    /// Unlike the c2 [`capture_reference`](Toolchain::capture_reference) this
    /// needs **no `strace`**: the front end finishes writing the bundle before c2
    /// runs, and we do not need c2 to produce an obj here.
    ///
    /// Returns the surviving bundle, its base name, and the verbatim c1xx argv
    /// tokens (everything after the `…c1xx.dll` path token). `TMP`/`TEMP` point
    /// at `work_dir` so the bundle lands there deterministically.
    pub fn capture_c1_reference(&self, cpp: &Path, work_dir: &Path) -> io::Result<CapturedC1> {
        std::fs::create_dir_all(work_dir)?;
        let work_abs = absolute(work_dir)?;

        let z_src = to_wibo_path(&absolute(cpp)?);
        let z_obj = to_wibo_path(&work_abs.join("il_capture.obj"));

        let output = Command::new(&self.wibo)
            .arg(&self.cl_exe)
            .arg("/Bd")
            .arg("/d2nop")
            .arg("/Ox")
            .arg("/GS-")
            .arg("/c")
            .arg(format!("/Fo{z_obj}"))
            .arg(&z_src)
            .env("TMP", &work_abs)
            .env("TEMP", &work_abs)
            .env("WIBO_FS_CACHE", "1")
            // See capture_il: the surviving _CL_*ex file is the product, so
            // wibo's temp reaper (>= 1.0.1-23) must be disabled here.
            .env("WIBO_KEEP_TEMP", "1")
            .output()?;
        // Non-zero exit is expected (c2 aborted on /d2nop) — the surviving
        // `_CL_*ex` file, not the exit code, is the success signal.

        let mut combined = String::from_utf8_lossy(&output.stdout).into_owned();
        combined.push('\n');
        combined.push_str(&String::from_utf8_lossy(&output.stderr));

        let c1_argv = parse_c1_argv(&combined).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "could not find the `…c1xx.dll -il …` argv echo in compiler output\n  status: {}\n  stdout:\n{}\n  stderr:\n{}",
                    output.status,
                    indent(&String::from_utf8_lossy(&output.stdout)),
                    indent(&String::from_utf8_lossy(&output.stderr)),
                ),
            )
        })?;

        let base_name = find_bundle_base(&work_abs)?.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("no surviving `_CL_*ex` bundle in {}", work_abs.display()),
            )
        })?;
        let bundle = IlBundle::load_from_dir(&work_abs, &base_name)?;
        if bundle.ex().map(|b| b.is_empty()).unwrap_or(true) {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("captured bundle {base_name} has an empty/absent .ex"),
            ));
        }

        Ok(CapturedC1 {
            bundle,
            base_name,
            c1_argv,
        })
    }

    /// **P-F0.1 front-end replay.** Drive `c1xx.dll` *alone* through `c1host` on
    /// the captured source, reconstructing the captured c1xx argv but swapping
    /// the IL-output base (`-il`) to a **fresh** base under `out_bundle_dir` (and
    /// `-Fo` to a scratch path — neither affects the bundle bytes). Everything
    /// else — crucially `-f <src>`, whose lowercased path the front end bakes
    /// into `.gl` — is kept verbatim. Runs with cwd = the toolchain dir so the
    /// sibling runtime DLLs (`msvcp100.dll`/`TLBREF.dll`/…) resolve.
    ///
    /// Returns the freshly written bundle. Comparing it byte-for-byte to the
    /// captured bundle is the P-F0.1 proof that the front-end replay oracle is
    /// real. Requires `c1host` (mingw) — built on demand via
    /// [`Toolchain::ensure_c1host`].
    pub fn replay_c1(&self, captured: &CapturedC1, out_bundle_dir: &Path) -> io::Result<IlBundle> {
        let c1host = absolute(&self.ensure_c1host()?)?;

        std::fs::create_dir_all(out_bundle_dir)?;
        let out_abs = absolute(out_bundle_dir)?;
        let replay_base = "_CL_replay";
        // Clear any stale replay bundle so a failed run can't masquerade as one.
        for suffix in ["ex", "gl", "sy", "in", "db"] {
            let _ = std::fs::remove_file(out_abs.join(format!("{replay_base}{suffix}")));
        }
        let z_il = to_wibo_path(&out_abs.join(replay_base));
        let z_fo = to_wibo_path(&out_abs.join("replay.obj"));

        // Reconstruct the c1xx argv, swapping only -il and -Fo.
        let mut argv: Vec<String> = Vec::with_capacity(captured.c1_argv.len());
        let mut i = 0;
        while i < captured.c1_argv.len() {
            let t = &captured.c1_argv[i];
            if t == "-il" {
                argv.push("-il".to_string());
                argv.push(z_il.clone());
                i += 2;
                continue;
            }
            if t.starts_with("-Fo") {
                argv.push(format!("-Fo{z_fo}"));
                i += 1;
                continue;
            }
            argv.push(t.clone());
            i += 1;
        }

        let wibo = absolute(&self.wibo)?;
        let c1xx = absolute(&self.c1xx_dll)?;
        let toolchain_dir = c1xx
            .parent()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "c1xx_dll has no parent"))?
            .to_path_buf();

        // wibo c1host <c1xx.dll (LoadLibrary)> <c1xx.dll (argv0)> <reconstructed argv…>
        let mut cmd = Command::new(&wibo);
        cmd.arg(&c1host).arg(&c1xx).arg(&c1xx);
        for a in &argv {
            cmd.arg(a);
        }
        let output = cmd
            .env("WIBO_FS_CACHE", "1")
            .current_dir(&toolchain_dir)
            .output()?;

        let bundle = IlBundle::load_from_dir(&out_abs, replay_base)?;
        match bundle.ex() {
            Some(ex) if !ex.is_empty() => Ok(bundle),
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "standalone-c1 replay produced no `{replay_base}ex` in {}\n  status: {}\n  stderr:\n{}",
                    out_abs.display(),
                    output.status,
                    indent(&String::from_utf8_lossy(&output.stderr)),
                ),
            )),
        }
    }
}

/// A captured reference: the surviving IL bundle, its base name, the verbatim c2
/// argv tokens (everything after the `…c2.dll` path token), the reference obj
/// bytes (from the *real* pipeline c2 run), and the obj's on-disk path.
#[derive(Clone, Debug)]
pub struct CapturedReference {
    /// The surviving `_CL_*` IL bundle (loaded from the work dir).
    pub bundle: IlBundle,
    /// Suffix-free bundle base, e.g. `_CL_3940e956`.
    pub base_name: String,
    /// c2 argv tokens after the `…c2.dll` path token, verbatim (incl. `-il`
    /// value, all `-Q*`/`-G*` flags, `-f <src>`, `-Fo<obj>`).
    pub c2_argv: Vec<String>,
    /// Reference obj bytes — c2 ran for real during capture.
    pub ref_obj: ObjImage,
    /// Host path the reference obj was written to (`<work_dir>/out.obj`).
    pub ref_obj_path: PathBuf,
}

/// A captured **front-end** reference (P-F0.1): the surviving IL bundle (the
/// front end's output), its base name, and the verbatim `c1xx.dll` argv tokens
/// (everything after the `…c1xx.dll` path token, including `-il` value, all
/// `-D…` defines, `-f <src>`, and `-Fo<obj>`).
#[derive(Clone, Debug)]
pub struct CapturedC1 {
    /// The surviving `_CL_*` IL bundle produced by the front end.
    pub bundle: IlBundle,
    /// Suffix-free bundle base, e.g. `_CL_fbdd6cfa`.
    pub base_name: String,
    /// c1xx argv tokens after the `…c1xx.dll` path token, verbatim.
    pub c1_argv: Vec<String>,
}

/// Parse a wibo version line into a comparable `(major, minor, patch, release)`.
///
/// Accepts the shapes wibo actually prints — `wibo 1.0.1-23-g4a9dd6f (Linux
/// x86_64)`, `wibo 1.0.1-7-g3b0f71c-dirty (Linux x86_64)`, and a bare
/// `1.0.1-23` — by taking the first whitespace token that starts with a digit
/// and contains a `.`, then reading the dotted triple and the `-N` git-describe
/// commit count that follows it. A missing `-N` reads as release 0, which sorts
/// *before* every tagged-plus-N build, which is the conservative direction: an
/// exact-tag build is treated as older than the known-good `1.0.1-23`.
///
/// Returns `None` rather than guessing when nothing parses — the caller reports
/// "unknown", never "fine".
pub fn parse_wibo_version(line: &str) -> Option<(u32, u32, u32, u32)> {
    let tok = line.split_whitespace().find(|t| {
        t.starts_with(|c: char| c.is_ascii_digit()) && t.contains('.')
    })?;
    let mut parts = tok.splitn(2, '-');
    let triple = parts.next()?;
    let rest = parts.next().unwrap_or("");
    let mut it = triple.split('.');
    let major = it.next()?.parse().ok()?;
    let minor = it.next().unwrap_or("0").parse().ok()?;
    let patch = it.next().unwrap_or("0").parse().ok()?;
    // `23-g4a9dd6f` / `7-g3b0f71c-dirty` / `` → the leading digit run.
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    let release = digits.parse().unwrap_or(0);
    Some((major, minor, patch, release))
}

/// Parse the backtick-quoted c2 argv-echo line from `/Bd` output. Finds the line
/// mentioning both `c2.dll` and `-il`, strips the leading backtick and trailing
/// `'`, splits off everything after the `…c2.dll` path token, and returns the
/// whitespace-split tokens (empties dropped). Mirrors the parse in
/// `/tmp/p01/validate.py`.
fn parse_c2_argv(text: &str) -> Option<Vec<String>> {
    let line = text.lines().find(|ln| {
        let low = ln.to_lowercase();
        low.contains("c2.dll") && low.contains("-il")
    })?;
    let trimmed = line.trim().trim_start_matches('`').trim_end_matches('\'');
    // Everything after the first "c2.dll" occurrence (the dll path ends in it).
    let idx = trimmed.find("c2.dll")?;
    let tail = &trimmed[idx + "c2.dll".len()..];
    let tokens: Vec<String> = tail.split_whitespace().map(|s| s.to_string()).collect();
    if tokens.is_empty() {
        None
    } else {
        Some(tokens)
    }
}

/// Parse the backtick-quoted **c1xx** argv-echo line from `/Bd` output. Finds the
/// line mentioning both `c1xx.dll` and `-il`, strips the leading backtick and
/// trailing `'`, and returns the whitespace-split tokens after the `…c1xx.dll`
/// path token. Sibling of [`parse_c2_argv`] (the two `/Bd` echo lines are
/// distinguished by their dll name).
fn parse_c1_argv(text: &str) -> Option<Vec<String>> {
    let line = text.lines().find(|ln| {
        let low = ln.to_lowercase();
        low.contains("c1xx.dll") && low.contains("-il")
    })?;
    let trimmed = line.trim().trim_start_matches('`').trim_end_matches('\'');
    let idx = trimmed.find("c1xx.dll")?;
    let tail = &trimmed[idx + "c1xx.dll".len()..];
    let tokens: Vec<String> = tail.split_whitespace().map(|s| s.to_string()).collect();
    if tokens.is_empty() {
        None
    } else {
        Some(tokens)
    }
}

/// Find the surviving IL bundle base in `dir`: the first file named
/// `_CL_<hex>ex`, with the trailing `ex` stripped (e.g. `_CL_3940e956`).
fn find_bundle_base(dir: &Path) -> io::Result<Option<String>> {
    let mut found: Option<String> = None;
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with("_CL_") && name.ends_with("ex") {
            let base = name[..name.len() - 2].to_string();
            // Deterministic pick: smallest name if several (there is normally one).
            match &found {
                Some(prev) if prev <= &base => {}
                _ => found = Some(base),
            }
        }
    }
    Ok(found)
}

/// Convert an absolute host path to a wibo path: leading `/` → `Z:` with
/// `\`-joined components. Relative paths pass through unchanged. Single
/// backslashes — through `std::process::Command` (execve, no shell) they are
/// passed verbatim; do NOT double them. Mirrors `_to_wibo_path` in invoker.py.
pub fn to_wibo_path(p: &Path) -> String {
    let s = p.to_string_lossy();
    if s.starts_with('/') {
        format!("Z:{}", s.replace('/', "\\"))
    } else {
        s.into_owned()
    }
}

/// Make a path absolute without requiring the file itself to exist (its parent
/// must). Canonicalizes so the result is anchored at the real filesystem root,
/// which is what the wibo `Z:\` mapping needs.
fn absolute(p: &Path) -> io::Result<PathBuf> {
    if p.exists() {
        return p.canonicalize();
    }
    let file = p.file_name().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "path has no file name")
    })?;
    let parent = p.parent().filter(|p| !p.as_os_str().is_empty());
    let base = match parent {
        Some(dir) => dir.canonicalize()?,
        None => std::env::current_dir()?,
    };
    Ok(base.join(file))
}

/// Scrape `-il <prefix>_CL_<hex>` from compiler output. Returns the directory
/// the bundle landed in and the suffix-free base name (`_CL_<hex>`).
///
/// Ports the Python regex `-il\s+(\S*?)(_CL_[0-9a-f]+)` by hand (no regex
/// crate): find each `-il` followed by whitespace, take the next non-space
/// token, split it at `_CL_`, and read the trailing lowercase hex run as the
/// hash. The prefix (with `\` normalized to `/`) is the directory.
fn scrape_il_base(text: &str, fallback_dir: &Path) -> Option<(PathBuf, String)> {
    let mut search = 0usize;
    while let Some(rel) = text[search..].find("-il") {
        let idx = search + rel;
        let after = &text[idx + 3..];
        // Require whitespace directly after "-il" (so "-ilfoo" doesn't match).
        let trimmed = after.trim_start_matches([' ', '\t']);
        if trimmed.len() == after.len() {
            search = idx + 3;
            continue;
        }
        let token: String = trimmed.chars().take_while(|c| !c.is_whitespace()).collect();
        if let Some(pos) = token.find("_CL_") {
            let prefix = &token[..pos];
            let after_cl = &token[pos + 4..];
            let hash: String = after_cl
                .chars()
                .take_while(|c| matches!(c, '0'..='9' | 'a'..='f'))
                .collect();
            if !hash.is_empty() {
                let base = format!("_CL_{hash}");
                let dir = if prefix.is_empty() {
                    fallback_dir.to_path_buf()
                } else {
                    let norm = prefix.replace('\\', "/");
                    let d = norm.trim_end_matches('/');
                    if d.is_empty() {
                        PathBuf::from("/")
                    } else {
                        PathBuf::from(d)
                    }
                };
                return Some((dir, base));
            }
        }
        search = idx + 3;
    }
    None
}

fn indent(s: &str) -> String {
    s.lines().map(|l| format!("    {l}")).collect::<Vec<_>>().join("\n")
}

/// Create a directory symlink `link` → `target`. Unix-only (this harness targets
/// Linux + wibo); other platforms error clearly rather than silently miscompile.
#[cfg(unix)]
fn symlink_dir(target: &Path, link: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(not(unix))]
fn symlink_dir(_target: &Path, _link: &Path) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "standalone-c1 replay requires a Unix symlink for the 1033 resources dir",
    ))
}

/// The real c2 as a [`Backend`] — the **P0.1 replay path (IMPLEMENTED)**.
///
/// Its [`Backend::compile`] feeds an IL bundle back through `c2.dll` *alone*
/// (via the [`c2host`](../c2host) stub under wibo), using a fixed DC3 standalone
/// c2-argv template. This is "standalone c2 as a pure function of the bundle" —
/// it works even for synthesized IL with no source (c2 tolerates a bogus `-f`).
///
/// NOTE: byte-equality to a *specific* pipeline obj additionally requires
/// matching `-f`/`-Fo`/backend flags. The differential self-check therefore uses
/// [`Toolchain::replay`] with the *captured* argv (not this default template) to
/// prove reference-replay byte-exactness. This backend is the general
/// bundle→obj function used where an exact pipeline match is not the goal.
pub struct ReferenceC2<'a>(pub &'a Toolchain);

impl<'a> ReferenceC2<'a> {
    pub fn new(tc: &'a Toolchain) -> Self {
        ReferenceC2(tc)
    }
}

impl<'a> Backend for ReferenceC2<'a> {
    fn compile(&self, il: &IlBundle) -> Result<ObjImage, BackendError> {
        let tc = self.0;
        let c2host = tc.ensure_c2host().map_err(BackendError::Io)?;

        // Private scratch dir for this compile.
        let work = scratch_dir("refc2");
        let bundle_dir = work.join("il");
        let base = if il.base_name.is_empty() {
            "_CL_synth".to_string()
        } else {
            il.base_name.clone()
        };
        il.write_to_dir(&bundle_dir, &base).map_err(BackendError::Io)?;

        let base_path = absolute(&bundle_dir.join(&base)).map_err(BackendError::Io)?;
        let z_il = to_wibo_path(&base_path);
        let out = work.join("out.obj");
        let out_abs = absolute(&out).map_err(BackendError::Io)?;
        let z_fo = to_wibo_path(&out_abs);
        // Canonical bogus source path — c2 tolerates a `-f` that does not exist.
        let z_src = r"Z:\c2rs\synth.cpp";

        // Fixed DC3 standalone c2 argv template (backend flags `/Ox` expands to).
        let template: Vec<String> = vec![
            "-il".into(),
            z_il,
            "-typedil".into(),
            "-W".into(),
            "1".into(),
            "-Gs4096".into(),
            "-G604".into(),
            "-QVMX128".into(),
            "-QDD2".into(),
            "-MT".into(),
            "-Fdvc100.pdb".into(),
            "-f".into(),
            z_src.into(),
            "-Og".into(),
            "-Ob2".into(),
            format!("-Fo{z_fo}"),
        ];

        let mut cmd = Command::new(&tc.wibo);
        cmd.arg(&c2host).arg(&tc.c2_dll).arg(&tc.c2_dll);
        for a in &template {
            cmd.arg(a);
        }
        let output = cmd
            .env("WIBO_FS_CACHE", "1")
            .current_dir(&bundle_dir)
            .output()
            .map_err(BackendError::Io)?;

        if !out_abs.exists() {
            let msg = format!(
                "standalone c2 produced no obj at {}\n  status: {}\n  stderr:\n{}",
                out_abs.display(),
                output.status,
                indent(&String::from_utf8_lossy(&output.stderr)),
            );
            let _ = std::fs::remove_dir_all(&work);
            return Err(BackendError::Pass {
                pass: "standalone-c2".into(),
                msg,
            });
        }
        let obj = std::fs::read(&out_abs).map_err(BackendError::Io)?;
        let _ = std::fs::remove_dir_all(&work);
        Ok(ObjImage::new(obj))
    }

    fn name(&self) -> &str {
        // Name carries "reference" so the harness can distinguish this from the
        // native port.
        "reference-c2-replay"
    }
}

/// Build one x86 host stub (`c2host`, `c1host`) from `src` to `exe`, **atomically**.
///
/// One implementation for both stubs, because they are the same recipe and this
/// repo has twice paid for "one rule, two implementations" (`docs/GAPS.md` §6
/// #9, #10).
///
/// The atomicity is the point, and it is a *fixed flake*, not a precaution.
/// `cargo test --workspace --release` runs the integration tests
/// multi-threaded, every one of them reaches `ensure_c2host`, and the stub is
/// stale exactly once per fresh worktree — the `.c` gets a checkout mtime and
/// the reflinked `target/` copy is older. Linking **in place** then means N
/// concurrent `i686-w64-mingw32-gcc` processes writing one output file while
/// other tests are launching it, and wibo answers a half-written file with
/// `Failed to load PE image …/c2host.exe`. Reproduced by touching the source
/// and running the differential binary at 32 test threads: **4–13 of 17 tests
/// failed** per run, six runs out of six, and the same binary was green
/// immediately afterwards because the stub was then fresh. That is the
/// "intermittent in parallel, green serially and on re-run" flake exactly.
///
/// Linking to a private sibling and `rename`-ing it into place makes the
/// publication atomic: a concurrent reader opens either the old inode or the
/// new one, never a partial. It also sidesteps `ETXTBSY` from writing an
/// executable that another process is running.
fn build_host_stub(mingw: &Path, src: &Path, exe: &Path, what: &str) -> io::Result<()> {
    let parent = exe
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "host exe has no parent"))?;
    std::fs::create_dir_all(parent)?;
    let tmp = parent.join(format!(
        ".{what}.{}.{}.tmp",
        std::process::id(),
        SCRATCH_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let output = Command::new(mingw)
        .arg("-static")
        .arg("-static-libgcc")
        .arg("-O2")
        .arg("-o")
        .arg(&tmp)
        .arg(src)
        .output()?;
    if !output.status.success() || !tmp.exists() {
        let _ = std::fs::remove_file(&tmp);
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "building {what} failed\n  status: {}\n  stderr:\n{}",
                output.status,
                indent(&String::from_utf8_lossy(&output.stderr)),
            ),
        ));
    }
    if let Err(e) = std::fs::rename(&tmp, exe) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    Ok(())
}

static SCRATCH_COUNTER: AtomicU64 = AtomicU64::new(0);

/// A unique scratch directory under the system temp dir.
fn scratch_dir(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let n = SCRATCH_COUNTER.fetch_add(1, Ordering::Relaxed);
    let d = std::env::temp_dir().join(format!(
        "c2rs-{tag}-{}-{}-{}",
        std::process::id(),
        nanos,
        n
    ));
    let _ = std::fs::create_dir_all(&d);
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_wibo_path_absolute() {
        assert_eq!(
            to_wibo_path(Path::new("/proj/x/y.cpp")),
            r"Z:\proj\x\y.cpp"
        );
        assert_eq!(to_wibo_path(Path::new("/")), r"Z:\");
    }

    #[test]
    fn to_wibo_path_relative_unchanged() {
        assert_eq!(to_wibo_path(Path::new("sub/dir/y.cpp")), "sub/dir/y.cpp");
    }

    #[test]
    fn scrape_current_wibo_form() {
        // Current build: prefix is a real tmp path with a backslash separator.
        let text = "some noise\n -il /tmp/c2probe\\_CL_5114c7e9\nmore\n";
        let (dir, base) = scrape_il_base(text, Path::new("/fallback")).unwrap();
        assert_eq!(base, "_CL_5114c7e9");
        assert_eq!(dir, PathBuf::from("/tmp/c2probe"));
    }

    #[test]
    fn scrape_bare_form_uses_fallback_dir() {
        // Older build: bare `_CL_<hash>`, no directory prefix.
        let text = "cl : -il _CL_deadbeef blah";
        let (dir, base) = scrape_il_base(text, Path::new("/fallback")).unwrap();
        assert_eq!(base, "_CL_deadbeef");
        assert_eq!(dir, PathBuf::from("/fallback"));
    }

    #[test]
    fn scrape_ignores_non_flag() {
        assert!(scrape_il_base("nothing here", Path::new("/f")).is_none());
        // "-ilfoo" is not the -il flag (no whitespace separator).
        assert!(scrape_il_base("-ilfoo_CL_1234", Path::new("/f")).is_none());
    }

    #[test]
    fn parse_c1_argv_isolates_the_c1xx_line() {
        // Both /Bd echo lines are present; parse_c1_argv must pick the c1xx one
        // and return the tokens after the dll path (starting at -zm), keeping -f.
        let text = "noise\n\
            `Z:\\tc\\c1xx.dll -zm0x11000000 -il /tmp/x\\_CL_ab12 -typedil -f Z:\\p\\a.cpp -Fo Z:\\p\\o.obj'\n\
            `Z:\\tc\\c2.dll -il /tmp/x\\_CL_ab12 -typedil -f Z:\\p\\a.cpp -FoZ:\\p\\o.obj'\n";
        let argv = parse_c1_argv(text).unwrap();
        assert_eq!(argv[0], "-zm0x11000000");
        assert_eq!(argv[1], "-il");
        assert_eq!(argv[2], "/tmp/x\\_CL_ab12");
        assert!(argv.iter().any(|t| t == "-f"));
        // The c2 line (no c1xx.dll) must not be what we matched.
        assert!(!argv.iter().any(|t| t.contains("c2.dll")));
    }

    #[test]
    fn parse_c1_argv_none_without_c1xx_line() {
        assert!(parse_c1_argv("`Z:\\tc\\c2.dll -il _CL_1 -f a.cpp'").is_none());
        assert!(parse_c1_argv("no compiler echo here").is_none());
    }

    #[test]
    fn wibo_version_parses_the_shapes_wibo_prints() {
        assert_eq!(
            parse_wibo_version("wibo 1.0.1-23-g4a9dd6f (Linux x86_64)"),
            Some((1, 0, 1, 23))
        );
        assert_eq!(
            parse_wibo_version("wibo 1.0.1-7-g3b0f71c-dirty (Linux x86_64)"),
            Some((1, 0, 1, 7))
        );
        assert_eq!(parse_wibo_version("1.0.1-23"), Some((1, 0, 1, 23)));
        assert_eq!(parse_wibo_version("wibo 2.0 (Linux)"), Some((2, 0, 0, 0)));
        assert_eq!(parse_wibo_version("no version here"), None);
    }

    #[test]
    fn wibo_version_ordering_puts_the_stale_build_first() {
        // The exact pair that faked a replay alarm: 1.0.1-7 must compare older
        // than the known-good 1.0.1-23 (a *numeric* compare — lexically "7" > "23").
        let old = parse_wibo_version("wibo 1.0.1-7-g3b0f71c-dirty (Linux x86_64)").unwrap();
        let good = parse_wibo_version(WIBO_KNOWN_GOOD).unwrap();
        assert!(old < good);
        assert!(!(parse_wibo_version("wibo 1.0.1-23-g4a9dd6f (Linux x86_64)").unwrap() < good));
        assert!(!(parse_wibo_version("wibo 1.0.2-0-gdeadbee (Linux)").unwrap() < good));
    }

    #[test]
    fn locate_is_none_when_paths_absent() {
        // Point the required vars at nonexistent paths; locate must return None.
        std::env::set_var("C2RS_WIBO", "/definitely/not/here/wibo");
        std::env::set_var("C2RS_CL_EXE", "/definitely/not/here/cl.exe");
        std::env::set_var("C2RS_C2_DLL", "/definitely/not/here/c2.dll");
        assert!(Toolchain::locate().is_none());
        std::env::remove_var("C2RS_WIBO");
        std::env::remove_var("C2RS_CL_EXE");
        std::env::remove_var("C2RS_C2_DLL");
    }
}
