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
//! [`ReferenceC2`] wraps a toolchain as a [`Backend`], but its `compile` is the
//! **P0.1 replay seam** — feeding a captured IL bundle back through c2 alone is
//! UNPROVEN and intentionally returns `NotImplemented`.
//!
//! [wibo]: https://github.com/decompals/wibo

use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

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
    /// `c2.dll` back-end (the thing being ported; used by the replay seam).
    pub c2_dll: PathBuf,
    /// dc3-decomp checkout root — optional context.
    pub dc3_root: PathBuf,
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
        };
        // Required for any real work. wibo_debug / dc3_root are optional.
        if tc.wibo.exists() && tc.cl_exe.exists() && tc.c2_dll.exists() {
            Some(tc)
        } else {
            None
        }
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

/// The real c2 as a [`Backend`] — the **P0.1 replay seam**.
///
/// Its [`Backend::compile`] would feed a *captured IL bundle* back through
/// `c2.dll` alone to obtain an `.obj` without re-running the front-end. That is
/// the P0.1 research gate and is intentionally NOT implemented (see the comment
/// block on `compile`).
pub struct ReferenceC2<'a>(pub &'a Toolchain);

impl<'a> ReferenceC2<'a> {
    pub fn new(tc: &'a Toolchain) -> Self {
        ReferenceC2(tc)
    }
}

impl<'a> Backend for ReferenceC2<'a> {
    fn compile(&self, _il: &IlBundle) -> Result<ObjImage, BackendError> {
        // ============================================================
        // P0.1 REPLAY SEAM — UNPROVEN, NOT IMPLEMENTED. DO NOT FAKE.
        // ------------------------------------------------------------
        // Goal: feed an (unmodified) captured IL bundle back through c2.dll
        // ALONE and get a byte-identical .obj, without re-running c1xx. First
        // success validates the whole harness (determinism check doubles as
        // harness validation) and unlocks angles that key off IL injection.
        //
        // Two candidate mechanisms, cleanest first (03_ROADMAP P0.1):
        //
        //   (a) InvokeCompilerPass host-stub. A small host EXE, run under wibo,
        //       LoadLibrary("c2.dll") + GetProcAddress("InvokeCompilerPass"
        //       ~VA 0x10BEBFFD) and call it with the reconstructed `-il <base>`
        //       argv that c1xx normally passes — pointing at our bundle dir.
        //
        //   (b) Full-pipeline pause-and-swap. Breakpoint/ptrace between c1xx
        //       writing the _CL_* files and c2 opening them; substitute our
        //       bundle files in place; resume.
        //
        // Neither has ever been executed. Until one works, this returns
        // NotImplemented so the harness reports `ReferenceReplayUnproven`
        // rather than pretending.
        // ============================================================
        Err(BackendError::NotImplemented(
            "standalone c2 IL-replay is the P0.1 research gate — never executed. \
             Candidate mechanisms (InvokeCompilerPass host-stub; pause-and-swap) \
             are documented in the source and docs/plans/il-witness/03_ROADMAP.md."
                .to_string(),
        ))
    }

    fn name(&self) -> &str {
        // Name carries "reference" so the harness can distinguish a replay-seam
        // NotImplemented from a port NotImplemented.
        "reference-c2-replay (P0.1 unproven)"
    }
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
