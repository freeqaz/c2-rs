//! `c2-harness` — the differential benchmark.
//!
//! Central question: **does this C++ (via its captured IL) compile to the same
//! `.obj` under the port as under the reference toolchain?** i.e.
//! `port(capture_il(cpp)) == c2(cpp)` on timestamp-normalized bytes.
//!
//! Reference side (P0.1) is **real and byte-exact**: [`differential`] captures
//! the pipeline obj + IL bundle, replays the bundle through standalone c2, and
//! proves the replay reproduces the pipeline obj byte-for-byte before reporting
//! the port status. The native port ([`c2_core::PortC2`]) is byte-exact on the
//! MVP function class and returns `NotImplemented` outside it.
//!
//! The [`oracle_selftest`] ([`oracle_selftest`]) — determinism (compile twice,
//! normalized-equal) AND capture stability (capture twice, `.ex`-equal) — also
//! still runs green against the real toolchain.

use std::path::{Path, PathBuf};

use c2_core::{Backend, BackendError};
use c2_obj::{ObjDiff, ObjImage};
use c2_reference::Toolchain;

pub mod capture_cache;
pub mod corpus;
pub mod gap;
pub mod perf;
pub mod prefilter;
pub mod provenance;
pub mod retrieval;
pub mod search;

pub(crate) use corpus::jstr;

/// Port-side status, evaluated only after the reference replay is byte-exact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PortStatus {
    /// The bundle is outside the ported function class (the port declined it).
    NotImplemented(String),
    /// port(IL) matched the reference obj byte-exact (timestamp zeroed).
    Match,
    /// port(IL) differed; `first_offset` is the first diverging normalized byte.
    Mismatch { first_offset: usize },
}

/// Outcome of the full differential for one translation unit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiffReport {
    /// `Toolchain::locate()` returned `None`.
    ToolchainAbsent,
    /// The replay path is unavailable (strace/mingw absent) — clean skip.
    Skipped(String),
    /// Reference capture/replay could not run (I/O / toolchain error).
    ReferenceError(String),
    /// The standalone-c2 replay did NOT reproduce the pipeline obj — a real
    /// failure of the P0.1 mechanism (should never happen on the fixtures).
    ReferenceReplayMismatch {
        first_offset: usize,
        ref_len: usize,
        replay_len: usize,
    },
    /// The standalone-c2 replay reproduced the pipeline obj **byte-exact**; the
    /// port side is then evaluated and reported in `port`.
    ReferenceReplayByteExact {
        ref_len: usize,
        replay_len: usize,
        port: PortStatus,
    },
}

/// Run the full differential for `cpp`.
///
/// 1. [`Toolchain::capture_reference`] — one `/Bd` compile under `strace` that
///    runs c2 for real (the pipeline **reference obj**) while keeping the
///    `_CL_*` IL bundle and echoing the exact c2 argv.
/// 2. [`Toolchain::replay`] — write the bundle back out and re-run standalone
///    c2 on it (to the *same* `/Fo` path), then compare to the reference obj.
///    On the fixtures this is byte-exact — that is the P0.1 proof.
/// 3. Only if the replay is byte-exact, compile the bundle with `port` and
///    report its status: [`PortStatus::Match`] on an in-class TU (e.g. an int
///    add-chain), [`PortStatus::NotImplemented`] when the bundle is outside the
///    ported class.
///
/// Degrades to [`DiffReport::Skipped`] when `strace`/mingw are absent.
pub fn differential(
    cpp: &Path,
    reference: &Toolchain,
    port: &dyn Backend,
    work: &Path,
) -> DiffReport {
    if !reference.has_strace() {
        return DiffReport::Skipped("strace absent (needed to keep the IL bundle)".into());
    }
    if !reference.has_mingw() {
        return DiffReport::Skipped(
            "i686-w64-mingw32-gcc absent (needed to build the c2host stub)".into(),
        );
    }

    // 1. Capture the pipeline reference obj + IL bundle + exact c2 argv.
    let captured = match reference.capture_reference(cpp, &work.join("cap")) {
        Ok(c) => c,
        Err(e) => return DiffReport::ReferenceError(format!("capture_reference failed: {e}")),
    };

    // 2. Replay the bundle through standalone c2, to the SAME /Fo path as the
    //    reference so the embedded path string matches (ref bytes already read
    //    into memory as captured.ref_obj).
    let ref_obj_path = captured.ref_obj_path.clone();
    let replay_obj = match reference.replay(&captured, &work.join("replay_il"), &ref_obj_path) {
        Ok(o) => o,
        Err(e) => return DiffReport::ReferenceError(format!("replay failed: {e}")),
    };
    let ref_len = captured.ref_obj.len();
    let replay_len = replay_obj.len();
    if let ObjDiff::Differs { first_offset, .. } = ObjImage::diff(&captured.ref_obj, &replay_obj) {
        return DiffReport::ReferenceReplayMismatch {
            first_offset,
            ref_len,
            replay_len,
        };
    }

    // 3. Reference replay is byte-exact. Now evaluate the port. Thread the
    //    reference's exact `-Fo` output-path string (its wibo `Z:\…` form) into
    //    the port so the embedded S_OBJNAME matches — MSVC bakes that path into
    //    the obj, so it is a required emitter input, not a bundle fact.
    let obj_name = c2_reference::to_wibo_path(&captured.ref_obj_path);
    let port = match port.compile_to(&captured.bundle, &obj_name) {
        Ok(o) => match ObjImage::diff(&captured.ref_obj, &o) {
            ObjDiff::Identical => PortStatus::Match,
            ObjDiff::Differs { first_offset, .. } => PortStatus::Mismatch { first_offset },
        },
        Err(BackendError::NotImplemented(msg)) => PortStatus::NotImplemented(msg),
        Err(e) => PortStatus::NotImplemented(format!("{e}")),
    };

    DiffReport::ReferenceReplayByteExact {
        ref_len,
        replay_len,
        port,
    }
}

/// Per-file result of a front-end (P-F0.1) bundle byte-compare.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct C1FileCompare {
    /// IL suffix (`"ex"`, `"gl"`, `"sy"`, `"in"`, `"db"`).
    pub suffix: String,
    /// Captured (pipeline front-end) file length.
    pub cap_len: usize,
    /// Standalone-c1 replay file length.
    pub replay_len: usize,
    /// True iff the two files are byte-identical.
    pub identical: bool,
    /// First diverging byte offset when not identical.
    pub first_offset: Option<usize>,
}

/// Outcome of the front-end replay proof (P-F0.1) for one translation unit:
/// does driving `c1xx.dll` standalone reproduce the captured IL bundle?
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum C1ReplayReport {
    /// `Toolchain::locate()` returned `None`.
    ToolchainAbsent,
    /// The c1-replay path is unavailable (mingw / c1xx.dll absent) — clean skip.
    Skipped(String),
    /// Front-end capture/replay could not run (I/O / toolchain error).
    ReferenceError(String),
    /// The front end was replayed; `files` holds the per-suffix byte-compare.
    Replayed {
        /// Captured bundle base name (e.g. `_CL_fbdd6cfa`).
        base: String,
        files: Vec<C1FileCompare>,
    },
}

impl C1ReplayReport {
    /// True iff every present IL file reproduced byte-for-byte.
    pub fn all_identical(&self) -> bool {
        matches!(self, C1ReplayReport::Replayed { files, .. } if files.iter().all(|f| f.identical))
    }
}

/// **P-F0.1 front-end replay proof.** Capture the IL bundle (one `/Bd /d2nop`
/// compile), then reproduce it by driving `c1xx.dll` *alone* through `c1host` to
/// a **fresh** `-il` base, and compare the 5 files byte-for-byte.
///
/// Byte-equality means the front-end replay oracle is real — the precondition
/// for any future `port_c1(source) == c1(source)` claim, exactly as [`differential`]'s
/// byte-exact reference replay is for the back end. Needs `i686-w64-mingw32-gcc`
/// (builds `c1host`) and `c1xx.dll`; degrades to [`C1ReplayReport::Skipped`]
/// otherwise.
pub fn c1_replay_check(cpp: &Path, reference: &Toolchain, work: &Path) -> C1ReplayReport {
    if !reference.has_mingw() {
        return C1ReplayReport::Skipped(
            "i686-w64-mingw32-gcc absent (needed to build the c1host stub)".into(),
        );
    }
    if !reference.has_c1xx() {
        return C1ReplayReport::Skipped("c1xx.dll absent (front end not located)".into());
    }

    let captured = match reference.capture_c1_reference(cpp, &work.join("cap")) {
        Ok(c) => c,
        Err(e) => return C1ReplayReport::ReferenceError(format!("capture_c1 failed: {e}")),
    };
    let replay = match reference.replay_c1(&captured, &work.join("replay_bundle")) {
        Ok(b) => b,
        Err(e) => return C1ReplayReport::ReferenceError(format!("replay_c1 failed: {e}")),
    };

    let mut files = Vec::new();
    for suffix in c2_il::IL_SUFFIXES {
        let cap = captured.bundle.get(suffix).unwrap_or(&[]);
        let rep = replay.get(suffix).unwrap_or(&[]);
        let identical = cap == rep;
        let first_offset = if identical {
            None
        } else {
            cap.iter()
                .zip(rep.iter())
                .position(|(a, b)| a != b)
                .or(Some(cap.len().min(rep.len())))
        };
        files.push(C1FileCompare {
            suffix: suffix.to_string(),
            cap_len: cap.len(),
            replay_len: rep.len(),
            identical,
            first_offset,
        });
    }
    C1ReplayReport::Replayed {
        base: captured.base_name,
        files,
    }
}

/// Outcome of the oracle self-test for one translation unit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelfTestOutcome {
    /// Determinism + capture stability both held.
    Pass { obj_len: usize, ex_len: usize },
    /// Two compiles of the same source differed after normalization.
    DeterminismFail {
        first_offset: usize,
        len_a: usize,
        len_b: usize,
    },
    /// Two captures of the same source produced different `.ex` bytes.
    CaptureUnstable { ex_len_a: usize, ex_len_b: usize },
    /// A reference invocation failed (stderr captured in the message).
    Error(String),
}

/// Full self-test report for one translation unit.
#[derive(Clone, Debug)]
pub struct SelfTestReport {
    pub cpp: PathBuf,
    pub outcome: SelfTestOutcome,
}

impl SelfTestReport {
    pub fn passed(&self) -> bool {
        matches!(self.outcome, SelfTestOutcome::Pass { .. })
    }
}

/// The oracle self-test: the part of the benchmark that runs green today.
///
/// 1. **Determinism** — compile `cpp` twice; the normalized objs must be equal.
/// 2. **Capture stability** — capture the IL twice (into separate dirs); the
///    `.ex` bytes must be equal.
///
/// Together these establish that the reference oracle is a stable ground truth,
/// which is the precondition for trusting any future `port(IL) == c2(IL)` claim.
pub fn oracle_selftest(cpp: &Path, reference: &Toolchain, work: &Path) -> SelfTestReport {
    let outcome = run_selftest(cpp, reference, work);
    SelfTestReport {
        cpp: cpp.to_path_buf(),
        outcome,
    }
}

fn run_selftest(cpp: &Path, reference: &Toolchain, work: &Path) -> SelfTestOutcome {
    // --- determinism ---
    // NOTE: MSVC embeds the /Fo output path in the COFF (near the C1/C2 version
    // strings), so the obj is a deterministic function of (source, output-path).
    // Both compiles therefore use the SAME output path — differing paths would
    // be a spurious "difference" that is really just the embedded filename.
    let det = work.join("det.obj");
    let a = match reference.compile_obj(cpp, &det) {
        Ok(o) => o,
        Err(e) => return SelfTestOutcome::Error(format!("compile #1 failed: {e}")),
    };
    let b = match reference.compile_obj(cpp, &det) {
        Ok(o) => o,
        Err(e) => return SelfTestOutcome::Error(format!("compile #2 failed: {e}")),
    };
    if let ObjDiff::Differs {
        first_offset,
        a_len,
        b_len,
    } = ObjImage::diff(&a, &b)
    {
        return SelfTestOutcome::DeterminismFail {
            first_offset,
            len_a: a_len,
            len_b: b_len,
        };
    }

    // --- capture stability ---
    let ca = match reference.capture_il(cpp, &work.join("cap_a")) {
        Ok(bundle) => bundle,
        Err(e) => return SelfTestOutcome::Error(format!("capture #1 failed: {e}")),
    };
    let cb = match reference.capture_il(cpp, &work.join("cap_b")) {
        Ok(bundle) => bundle,
        Err(e) => return SelfTestOutcome::Error(format!("capture #2 failed: {e}")),
    };
    let ex_a = ca.ex().unwrap_or(&[]);
    let ex_b = cb.ex().unwrap_or(&[]);
    if ex_a != ex_b {
        return SelfTestOutcome::CaptureUnstable {
            ex_len_a: ex_a.len(),
            ex_len_b: ex_b.len(),
        };
    }

    SelfTestOutcome::Pass {
        obj_len: a.len(),
        ex_len: ex_a.len(),
    }
}

/// Discover the bundled fixture translation units (`fixtures/cpp/*.cpp`).
pub fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("fixtures/cpp")
}

/// Every `*.cpp` under [`fixtures_dir`], sorted by name.
pub fn all_fixtures() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(fixtures_dir()) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) == Some("cpp") {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_fixtures_finds_the_bundled_cpp() {
        let names: Vec<String> = all_fixtures()
            .iter()
            .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
            .collect();
        assert!(names.contains(&"add3.cpp".to_string()), "got: {names:?}");
        assert!(names.contains(&"il_bool_materialization.cpp".to_string()));
        assert!(names.contains(&"il_call_return.cpp".to_string()));
    }
}
