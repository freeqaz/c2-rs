//! `c2-harness` — the differential benchmark.
//!
//! Central question: **does this C++ (via its captured IL) compile to the same
//! `.obj` under the port as under the reference toolchain?** i.e.
//! `port(capture_il(cpp)) == c2(cpp)` on timestamp-normalized bytes.
//!
//! Two of the three ingredients are stubs today:
//! * the native port ([`c2_core::PortC2`]) has no passes;
//! * standalone c2 IL-replay ([`c2_reference::ReferenceC2`]) is the P0.1 gate.
//!
//! So the *live-meaningful* path is the **oracle self-test** ([`oracle_selftest`]):
//! determinism (compile twice, normalized-equal) AND capture stability (capture
//! twice, `.ex`-equal). That runs green today against the real toolchain and is
//! what gives the benchmark teeth before either stub is filled in.

use std::path::{Path, PathBuf};

use c2_core::{Backend, BackendError};
use c2_obj::{ObjDiff, ObjImage};
use c2_reference::Toolchain;

/// Outcome of the full differential for one translation unit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiffReport {
    /// `Toolchain::locate()` returned `None`.
    ToolchainAbsent,
    /// The native port backend is a stub (today's normal result).
    PortNotImplemented(String),
    /// The reference replay seam (standalone c2) is the unproven P0.1 gate, or
    /// the reference toolchain could not otherwise produce the comparison obj.
    ReferenceReplayUnproven(String),
    /// port(IL) matched c2(IL) byte-exact (timestamp zeroed).
    Match,
    /// port(IL) differed from c2(IL); `first_offset` is the first diverging
    /// normalized byte.
    Mismatch { first_offset: usize },
}

/// Run the full differential for `cpp`: capture IL via the reference, compile
/// it with `port`, and compare against the reference's ground-truth obj.
///
/// `port` is any [`Backend`]. When it is [`c2_core::PortC2`] the result today is
/// [`DiffReport::PortNotImplemented`]; when it is
/// [`c2_reference::ReferenceC2`] (the replay seam) the result is
/// [`DiffReport::ReferenceReplayUnproven`]. Callers guard toolchain presence and
/// pass a located `reference`.
pub fn differential(
    cpp: &Path,
    reference: &Toolchain,
    port: &dyn Backend,
    work: &Path,
) -> DiffReport {
    // 1. Capture the IL bundle (reference front-end).
    let il = match reference.capture_il(cpp, &work.join("cap")) {
        Ok(il) => il,
        Err(e) => {
            return DiffReport::ReferenceReplayUnproven(format!(
                "reference IL capture failed: {e}"
            ))
        }
    };

    // 2. Compile it with the port under test.
    let port_obj = match port.compile(&il) {
        Ok(o) => o,
        Err(BackendError::NotImplemented(msg)) => {
            // Distinguish the replay-seam stub from the native-port stub by name.
            return if port.name().contains("reference") {
                DiffReport::ReferenceReplayUnproven(msg)
            } else {
                DiffReport::PortNotImplemented(msg)
            };
        }
        Err(e) => return DiffReport::PortNotImplemented(format!("{e}")),
    };

    // 3. Ground-truth obj from the normal pipeline.
    let ref_obj = match reference.compile_obj(cpp, &work.join("ref.obj")) {
        Ok(o) => o,
        Err(e) => {
            return DiffReport::ReferenceReplayUnproven(format!(
                "reference compile_obj failed: {e}"
            ))
        }
    };

    // 4. Compare on normalized bytes.
    match ObjImage::diff(&ref_obj, &port_obj) {
        ObjDiff::Identical => DiffReport::Match,
        ObjDiff::Differs { first_offset, .. } => DiffReport::Mismatch { first_offset },
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
