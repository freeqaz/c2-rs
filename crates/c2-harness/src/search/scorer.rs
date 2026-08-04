use std::path::{Path, PathBuf};
use std::time::Duration;

use c2_il::IlModel;
use c2_obj::{ObjDiff, ObjImage};
use c2_reference::{CapturedReference, Toolchain};

use super::engine::{Judged, Scorer};
use super::similarity::{insn_text_similarity, insn_text_similarity_perfn};

// ===========================================================================
// ReplayScorer — the real c2 judge
// ===========================================================================

/// Judges candidates by a REAL standalone-c2 replay through the toolchain, to a
/// FIXED `-Fo` path (so the embedded `S_OBJNAME` matches the target and a
/// byte-exact terminal is achievable), bounded by a timeout (a replay
/// crash/timeout is a clean [`Judged::Reject`], per P0.6c). This is the sole
/// judge on the real path — no simulated scoring.
pub struct ReplayScorer<'a> {
    tc: &'a Toolchain,
    base: &'a CapturedReference,
    target: ObjImage,
    scratch: PathBuf,
    /// The FIXED `-Fo` path every replay (target render + all candidates) uses,
    /// so the embedded `S_OBJNAME` matches and a byte-exact terminal is possible.
    pub fo: PathBuf,
    timeout: Duration,
    counter: usize,
    compiles: usize,
    /// When `Some(nfns)` (and `nfns > 1`), score the fuzzy gradient with the
    /// per-function-decomposed similarity ([`insn_text_similarity_perfn`]) so a
    /// correct edit to one function of a multi-function target is not masked by
    /// its intact siblings (the whole-`.text` plateau). `None` = the whole-`.text`
    /// gradient. The TERMINAL (byte-exact) is identical either way.
    per_fn: Option<usize>,
}

impl<'a> ReplayScorer<'a> {
    /// `base` supplies the captured c2 argv (its `-il`/`-Fo` are swapped per
    /// replay); `target` is the obj to reach byte-exact; `scratch` is a private
    /// work dir (candidate bundles + the fixed `-Fo` obj land under it). The
    /// fixed `-Fo` is [`ReplayScorer::fo`] — render the target to it (see
    /// [`solve_instance`]) so target and candidates share the embedded path.
    pub fn new(
        tc: &'a Toolchain,
        base: &'a CapturedReference,
        target: ObjImage,
        scratch: PathBuf,
        timeout: Duration,
    ) -> Self {
        let fo = scratch.join("cand.obj");
        ReplayScorer {
            tc,
            base,
            target,
            scratch,
            fo,
            timeout,
            counter: 0,
            compiles: 0,
            per_fn: None,
        }
    }

    /// The fixed `-Fo` path candidates and the target both replay to.
    pub fn fo_path(&self) -> &Path {
        &self.fo
    }

    /// Switch the fuzzy gradient to the per-function-decomposed similarity for a
    /// multi-function target (`nfns > 1`); `nfns <= 1` leaves the whole-`.text`
    /// gradient (a single function has nothing to decompose). The terminal check
    /// is unchanged. Returns `self` for chaining.
    pub fn per_function(&mut self, nfns: usize) -> &mut Self {
        self.per_fn = if nfns > 1 { Some(nfns) } else { None };
        self
    }
}

impl<'a> Scorer for ReplayScorer<'a> {
    fn judge(&mut self, model: &IlModel) -> Judged {
        self.compiles += 1;
        self.counter += 1;
        let cap = CapturedReference {
            bundle: model.encode(),
            ..self.base.clone()
        };
        let dir = self.scratch.join(format!("cand{}", self.counter));
        let verdict = match self
            .tc
            .replay_within(&cap, &dir, &self.fo, self.timeout)
        {
            Ok(obj) => {
                if matches!(ObjImage::diff(&obj, &self.target), ObjDiff::Identical) {
                    Judged::ByteExact
                } else {
                    // Instruction-aware gradient (never a terminal — see
                    // `insn_text_similarity`'s reconciliation note). The byte-exact
                    // terminal above is the sole success; this only ranks moves.
                    // For a multi-function target the per-function decomposition
                    // keeps the edited function's progress from being masked by
                    // intact siblings (the whole-`.text` plateau).
                    let fuzzy = match self.per_fn {
                        Some(nfns) => insn_text_similarity_perfn(&obj, &self.target, nfns),
                        None => insn_text_similarity(&obj, &self.target),
                    };
                    Judged::Fuzzy(fuzzy)
                }
            }
            Err(_) => Judged::Reject, // crash / timeout / no obj — skip cleanly
        };
        let _ = std::fs::remove_dir_all(&dir);
        verdict
    }

    fn compiles(&self) -> usize {
        self.compiles
    }
}
