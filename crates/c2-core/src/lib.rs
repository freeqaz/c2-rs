//! `c2-core` — the clean-room native port of the MSVC Xbox 360 PPC backend
//! `c2.dll`. [`PortC2`] emits a **byte-exact** `.obj` for the MVP function class
//! (straight-line integer add-chain leaves, tail calls, and a single framed
//! non-leaf call) and returns [`BackendError::NotImplemented`] outside it —
//! that boundary is the open gate. The other value here is the shape: the
//! [`Backend`] trait every compiler (the port, and the real toolchain used as
//! an oracle) implements.
//!
//! Doctrine (il-witness angle H): the correctness criterion is **I/O
//! equivalence**, not source fidelity — for every IL bundle,
//! `port(IL) == c2(IL)` byte-exact with the COFF timestamp zeroed. The real c2
//! under wibo is the sole differential judge (see the `c2-reference` crate).
//!
//! Roadmap: `docs/plans/il-witness/03_ROADMAP.md`, T-E native-port track.

pub use c2_il::IlBundle;
pub use c2_obj::ObjImage;

pub mod codegen;
pub mod coff;
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
/// Implemented by both the native port ([`PortC2`]) and — via the now-proven
/// P0.1 standalone-c2 replay — the real toolchain wrapper `ReferenceC2` in
/// `c2-reference`. The harness compares their outputs on normalized bytes.
pub trait Backend {
    /// Compile an IL bundle to a COFF `.obj`. The timestamp is not required to
    /// match — the harness normalizes it away before comparing.
    fn compile(&self, il: &IlBundle) -> Result<ObjImage, BackendError>;

    /// Compile an IL bundle to a COFF `.obj`, threading the `-Fo` **output-path
    /// string** the reference toolchain saw. MSVC embeds that path in the
    /// object (`.debug$S` S_OBJNAME), so a byte-exact match requires the port
    /// to see the *same* string — it is an emitter input, not a bundle fact.
    ///
    /// Default: ignore the name and defer to [`Backend::compile`] (correct for
    /// backends like `ReferenceC2` that fix the path themselves via replay).
    /// [`PortC2`] overrides this to embed `obj_name` verbatim.
    fn compile_to(&self, il: &IlBundle, obj_name: &str) -> Result<ObjImage, BackendError> {
        let _ = obj_name;
        self.compile(il)
    }

    /// Short stable identifier for this backend (used in reports).
    fn name(&self) -> &str;
}

/// The native port. For the MVP function class (a straight-line integer
/// add-chain leaf function, e.g. `int add3(int,int,int)`) this now emits a
/// **byte-exact** `.obj`: it parses the IL bundle
/// ([`IlBundle::mvp_function`](c2_il::IlBundle::mvp_function)), selects PPC
/// `.text` ([`codegen::select_text`]), and builds the 5-section COFF
/// ([`coff::emit_mvp_obj`]). Anything outside that class returns
/// [`BackendError::NotImplemented`].
///
/// The `-Fo` output-path string (embedded in `.debug$S` S_OBJNAME) is carried
/// on the struct so [`Backend::compile`] is self-contained; the harness's
/// differential prefers [`Backend::compile_to`] to thread the reference's exact
/// path in.
#[derive(Clone, Debug, Default)]
pub struct PortC2 {
    /// The `-Fo` output-path string to embed as S_OBJNAME (wibo `Z:\…` form).
    obj_name: String,
}

impl PortC2 {
    /// Construct with the `-Fo` output-path string to embed (S_OBJNAME).
    pub fn new(obj_name: impl Into<String>) -> Self {
        PortC2 {
            obj_name: obj_name.into(),
        }
    }

    /// Build the obj for `il`, embedding `obj_name` as S_OBJNAME. Handles one
    /// or more straight-line int add-chain functions in a single TU (each is
    /// selected + placed in a shared `.text`; see [`codegen::select_text`] and
    /// [`coff::emit_obj`]).
    pub fn build(&self, il: &IlBundle, obj_name: &str) -> Result<ObjImage, BackendError> {
        let funcs = il.functions().ok_or_else(|| {
            BackendError::NotImplemented(
                "PortC2 only handles straight-line int add-chain functions \
                 (e.g. add3, or a TU of several such); this bundle is outside \
                 that class. See c2-core::codegen and the CODEGEN spec."
                    .to_string(),
            )
        })?;

        // W4b2: a single-function TU whose body is a framed non-leaf call
        // (`return g(a) + k`) takes the dedicated 6-section path — it needs a
        // `.pdata` unwind section and the compiler label symbols, which the
        // straight-line/tail-call 5-section emitter does not model.
        if funcs.len() == 1 {
            if let Some(fc) = &funcs[0].framed_call {
                let text = codegen::framed_call_text(fc.add_k);
                let bytes =
                    coff::emit_framed_obj(obj_name, &funcs[0].mangled_name, &fc.callee, &text);
                return Ok(ObjImage::new(bytes));
            }
        }

        // Select each function's .text, recording each function's byte offset.
        // Functions start at an **8-byte-aligned** offset within .text (the
        // section is ALIGN_8): c2 zero-pads between functions to the next
        // 8-byte boundary, but does NOT pad the tail of .text. The first
        // function is at 0 (already aligned). Verified: mvp_sub's three 12-byte
        // functions land at 0x0 / 0x10 / 0x20 with 4 zero bytes between.
        let mut text: Vec<u8> = Vec::new();
        let mut placed: Vec<coff::Function> = Vec::with_capacity(funcs.len());
        for f in &funcs {
            while text.len() % 8 != 0 {
                text.push(0);
            }
            let off = text.len() as u32;
            let call = if let Some(callee) = &f.tail_call {
                // Tail call. A void bare call (`ops` empty) is a single
                // `b <callee>` (REL24) at this offset; an integer tail call
                // (`ops` = the argument sub-expression) first computes the
                // argument into r3, then branches (the branch, not the function
                // start, is the reloc site).
                let reloc_offset = if f.ops.is_empty() {
                    text.extend_from_slice(&codegen::encode_tail_branch(off));
                    off
                } else {
                    let (body, branch_off) = codegen::int_tail_call_text(f, off)?;
                    text.extend_from_slice(&body);
                    branch_off
                };
                Some(coff::Call {
                    reloc_offset,
                    callee,
                })
            } else {
                text.extend_from_slice(&codegen::select_text(f)?);
                None
            };
            placed.push(coff::Function {
                name: &f.mangled_name,
                text_offset: off,
                call,
            });
        }

        let bytes = coff::emit_obj(obj_name, &placed, &text);
        Ok(ObjImage::new(bytes))
    }
}

impl Backend for PortC2 {
    fn compile(&self, il: &IlBundle) -> Result<ObjImage, BackendError> {
        self.build(il, &self.obj_name)
    }

    fn compile_to(&self, il: &IlBundle, obj_name: &str) -> Result<ObjImage, BackendError> {
        self.build(il, obj_name)
    }

    fn name(&self) -> &str {
        "port-c2"
    }
}
