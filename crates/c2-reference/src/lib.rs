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
//! RAW-identical (COFF `TimeDateStamp` included; wibo pins it) on all three
//! bundled fixtures. The mechanism is the [`c2host`](../c2host) x86 stub that
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
//! [wibo]: https://github.com/decompals/wibo

use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

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
    /// dc3-decomp checkout root — optional context.
    pub dc3_root: PathBuf,
    /// `c2host.c` source (repo `c2host/c2host.c`) — built on demand for replay.
    pub c2host_src: PathBuf,
    /// Built `c2host.exe` cache path (gitignored; env `C2RS_C2HOST`, default
    /// `<repo>/target/c2host/c2host.exe`). Never committed.
    pub c2host_exe: PathBuf,
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
    /// `C2RS_CL_EXE`, `C2RS_C2_DLL`, `C2RS_DC3_ROOT`) with relative-to-repo-root
    /// defaults. Returns `None` if any *required* path (wibo, cl.exe, c2.dll) is
    /// missing, so callers can skip cleanly when the toolchain is absent.
    pub fn locate() -> Option<Toolchain> {
        let root = repo_root();
        let tc = Toolchain {
            wibo: env_or(&root, "C2RS_WIBO", "../wibo/build/release/wibo"),
            wibo_debug: env_or(&root, "C2RS_WIBO_DEBUG", "../wibo/build/debug/wibo"),
            cl_exe: env_or(
                &root,
                "C2RS_CL_EXE",
                "../dc3-decomp/build/compilers/X360/16.00.11886.00/cl.exe",
            ),
            c2_dll: env_or(
                &root,
                "C2RS_C2_DLL",
                "../dc3-decomp/build/compilers/X360/16.00.11886.00/c2.dll",
            ),
            dc3_root: env_or(&root, "C2RS_DC3_ROOT", "../dc3-decomp"),
            c2host_src: root.join("c2host/c2host.c"),
            c2host_exe: env_or(&root, "C2RS_C2HOST", "target/c2host/c2host.exe"),
            // strace / mingw are needed only for the replay path; their absence
            // does NOT block locate() — callers guard via has_strace/has_mingw.
            strace: std::env::var_os("C2RS_STRACE")
                .map(PathBuf::from)
                .or_else(|| find_on_path("strace")),
            mingw: std::env::var_os("C2RS_MINGW")
                .map(PathBuf::from)
                .or_else(|| find_on_path("i686-w64-mingw32-gcc")),
        };
        // Required for any real work. wibo_debug / dc3_root / strace / mingw are
        // optional (the last two only gate the replay path, not core toolchain).
        if tc.wibo.exists() && tc.cl_exe.exists() && tc.c2_dll.exists() {
            Some(tc)
        } else {
            None
        }
    }

    /// True iff `strace` is available (required for the replay-capture path).
    pub fn has_strace(&self) -> bool {
        self.strace.as_ref().map(|p| p.exists()).unwrap_or(false)
    }

    /// True iff `i686-w64-mingw32-gcc` is available (builds the `c2host` stub).
    pub fn has_mingw(&self) -> bool {
        self.mingw.as_ref().map(|p| p.exists()).unwrap_or(false)
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
            if let Some(parent) = self.c2host_exe.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let output = Command::new(&mingw)
                .arg("-static")
                .arg("-static-libgcc")
                .arg("-O2")
                .arg("-o")
                .arg(&self.c2host_exe)
                .arg(&self.c2host_src)
                .output()?;
            if !output.status.success() || !self.c2host_exe.exists() {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!(
                        "building c2host failed\n  status: {}\n  stderr:\n{}",
                        output.status,
                        indent(&String::from_utf8_lossy(&output.stderr)),
                    ),
                ));
            }
        }
        Ok(self.c2host_exe.clone())
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
            // Land the _CL_* bundle in our private work dir, deterministically.
            .env("TMP", &work_abs)
            .env("TEMP", &work_abs)
            .env("WIBO_FS_CACHE", "1")
            .output()?;
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

        let z_src = to_wibo_path(&absolute(cpp)?);
        let z_obj = to_wibo_path(&out_obj);

        // strace -f -e trace=unlink,unlinkat -e inject=unlink,unlinkat:retval=0
        //   -o /dev/null  <wibo> <cl.exe> /Bd /Ox /GS- /c /Fo<z_obj> <z_src>
        let output = Command::new(&strace)
            .arg("-f")
            .arg("-e")
            .arg("trace=unlink,unlinkat")
            .arg("-e")
            .arg("inject=unlink,unlinkat:retval=0")
            .arg("-o")
            .arg("/dev/null")
            .arg(&self.wibo)
            .arg(&self.cl_exe)
            .arg("/Bd")
            .arg("/Ox")
            .arg("/GS-")
            .arg("/c")
            .arg(format!("/Fo{z_obj}"))
            .arg(&z_src)
            .env("TMP", &work_abs)
            .env("TEMP", &work_abs)
            .env("WIBO_FS_CACHE", "1")
            .output()?;

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
        let output = cmd
            .env("WIBO_FS_CACHE", "1")
            .current_dir(&bundle_dir_abs)
            .output()?;

        if !out_abs.exists() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "replay produced no obj at {}\n  status: {}\n  stdout:\n{}\n  stderr:\n{}",
                    out_abs.display(),
                    output.status,
                    indent(&String::from_utf8_lossy(&output.stdout)),
                    indent(&String::from_utf8_lossy(&output.stderr)),
                ),
            ));
        }
        Ok(ObjImage::new(std::fs::read(&out_abs)?))
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
