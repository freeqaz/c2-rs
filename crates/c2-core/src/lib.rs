//! `c2-core` — the clean-room native port of the MSVC Xbox 360 PPC backend
//! `c2.dll`. **This is a STUB.** No compiler pass is ported yet; the value here
//! is the shape: the [`Backend`] trait every compiler (the port, and the real
//! toolchain used as an oracle) implements, and the [`PortC2`] placeholder.
//!
//! Doctrine (il-witness angle H): the correctness criterion is **I/O
//! equivalence**, not source fidelity — for every IL bundle,
//! `port(IL) == c2(IL)` byte-exact with the COFF timestamp zeroed. The real c2
//! under wibo is the sole differential judge (see the `c2-reference` crate).
//!
//! Roadmap: `docs/plans/il-witness/03_ROADMAP.md`, T-E native-port track.

pub use c2_il::IlBundle;
pub use c2_obj::ObjImage;

pub mod passes;

use std::fmt;

/// Error type for a [`Backend::compile`].
#[derive(Debug)]
pub enum BackendError {
    /// The backend (or a required mechanism) is a deliberate stub today.
    NotImplemented(String),
    /// Underlying I/O failure.
    Io(std::io::Error),
    /// A named compiler pass failed.
    Pass { pass: String, msg: String },
}

impl fmt::Display for BackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BackendError::NotImplemented(msg) => write!(f, "not implemented: {msg}"),
            BackendError::Io(e) => write!(f, "io error: {e}"),
            BackendError::Pass { pass, msg } => write!(f, "pass `{pass}` failed: {msg}"),
        }
    }
}

impl std::error::Error for BackendError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            BackendError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for BackendError {
    fn from(e: std::io::Error) -> Self {
        BackendError::Io(e)
    }
}

/// A compiler backend: something that turns an IL bundle into a COFF `.obj`.
///
/// Implemented by both the native port ([`PortC2`]) and — as the P0.1 replay
/// seam — the real toolchain wrapper in `c2-reference`. The harness compares
/// their outputs on normalized bytes.
pub trait Backend {
    /// Compile an IL bundle to a COFF `.obj`. The timestamp is not required to
    /// match — the harness normalizes it away before comparing.
    fn compile(&self, il: &IlBundle) -> Result<ObjImage, BackendError>;

    /// Short stable identifier for this backend (used in reports).
    fn name(&self) -> &str;
}

/// The native port — **STUB**. Every pass is unported; `compile` always errors.
pub struct PortC2;

impl Backend for PortC2 {
    fn compile(&self, _il: &IlBundle) -> Result<ObjImage, BackendError> {
        Err(BackendError::NotImplemented(
            "PortC2 backend is a stub: no c2.dll passes are ported yet. \
             See docs/plans/il-witness/03_ROADMAP.md (T-E native-port track) \
             and c2-core::passes for the pass order and first-port targets."
                .to_string(),
        ))
    }

    fn name(&self) -> &str {
        "port-c2"
    }
}
