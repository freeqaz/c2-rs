//! `c2-core` — the clean-room native port of the MSVC Xbox 360 PPC backend
//! `c2.dll`. [`PortC2`] emits a **byte-exact** `.obj` for the MVP function class
//! (straight-line integer add-chain leaves, tail calls, and a single framed
//! non-leaf call) and returns [`BackendError::NotImplemented`] outside it —
//! that boundary is the open gate. The other value here is the shape: the
//! [`Backend`] trait every compiler (the port, and the real toolchain used as
//! an oracle) implements.
//!
//! Doctrine: the correctness criterion is **I/O equivalence**, not source
//! fidelity — for every IL bundle, `port(IL) == c2(IL)` byte-exact with the
//! COFF timestamp zeroed. The real c2 under wibo is the sole differential
//! judge (see the `c2-reference` crate).

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
    /// Whether the compile requested **function-level linking** (`/Gy`, which
    /// `/O1` and `/O2` imply). See [`PortC2::with_function_level_linking`].
    fn_level_linking: bool,
}

impl PortC2 {
    /// Construct with the `-Fo` output-path string to embed (S_OBJNAME).
    pub fn new(obj_name: impl Into<String>) -> Self {
        PortC2 {
            obj_name: obj_name.into(),
            fn_level_linking: false,
        }
    }

    /// Declare that the compile used **function-level linking** (`/Gy`).
    ///
    /// This is not a cosmetic option: it changes the obj's *shape*. Without it
    /// c2 packs every function into one `.text`; with it each function gets its
    /// own COMDAT `.text` section (characteristics `0x60401020` rather than
    /// `0x60400020`), with the section count, section symbols and aux records
    /// all following. So the same IL bundle legitimately produces two different
    /// objs depending on an argv flag the bundle does not record.
    ///
    /// That matters more than it sounds: **`/O1` and `/O2` imply `/Gy`**, and
    /// the dc3 workload compiles with `/O1`, while every fixture here uses
    /// `/Ox` — which does not. The port therefore cannot emit for a real
    /// workload TU on the strength of having matched the fixtures, and it must
    /// be *told*, because the IL alone cannot say. Found by the differential:
    /// `system/utl/Spew.cpp` decoded to two empty functions, and the port
    /// emitted a 5-section packed obj against the reference's 6-section
    /// per-function-COMDAT one.
    ///
    /// COMDAT emission is not implemented, so setting this makes the port
    /// refuse rather than mis-emit.
    pub fn with_function_level_linking(mut self, yes: bool) -> Self {
        self.fn_level_linking = yes;
        self
    }

    /// True iff `flags` imply function-level linking: `/Gy` explicitly, or
    /// `/O1`/`/O2`, which include it. (`/Ox` does not.)
    pub fn flags_imply_function_level_linking<S: AsRef<str>>(flags: &[S]) -> bool {
        flags.iter().any(|f| {
            let f = f.as_ref();
            f.eq_ignore_ascii_case("/Gy")
                || f.eq_ignore_ascii_case("-Gy")
                || f.eq_ignore_ascii_case("/O1")
                || f.eq_ignore_ascii_case("-O1")
                || f.eq_ignore_ascii_case("/O2")
                || f.eq_ignore_ascii_case("-O2")
        })
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

        // R1: a TU that defines no functions. Its obj is the fixed four-section
        // shell with no `.text` at all, so it never reaches instruction
        // selection. `functions()` only returns an empty vec for a bundle whose
        // `.ex` positively declares an empty module (see `il::is_empty_module`).
        if funcs.is_empty() {
            return Ok(ObjImage::new(coff::emit_empty_obj(obj_name)));
        }

        // Which optimization mode to emit. `.ex` records it per function, so this
        // is read, never inferred from argv — and a TU that mixes modes (a
        // `#pragma optimize` mid-file) is refused rather than emitted under
        // whichever one happened to come first.
        //
        // Two modes are implemented, and they differ in one rule: a chain
        // intermediate whose predecessor is dead goes to a fresh descending
        // register under `/Ox` and to r11 under `/O1`. Anything else — `/Od`,
        // `#pragma optimize("", off)`, an unreadable prefix — refuses.
        //
        // The stakes, reproduced in `docs/OPT_MODE.md`: `int chain4(int a,int b,
        // int c,int d){return a*b*c*d;}` was `match` at `/Ox` and `mismatch` at
        // `/O1` before the mode was read at all. The whole dc3 workload compiles
        // `/O1`.
        //
        // Checked after the empty-module case on purpose: a TU with no functions
        // has no `4F 1F` segment to carry a word, and its obj is mode-independent.
        let words = il.opt_words().unwrap_or_default();
        let mut mode: Option<codegen::OptMode> = None;
        for (i, w) in words.iter().enumerate() {
            let m = match w {
                Some(v) if *v == c2_il::OPT_WORD_OX => codegen::OptMode::Ox,
                Some(v) if *v == c2_il::OPT_WORD_O1 => codegen::OptMode::O1,
                other => {
                    return Err(BackendError::NotImplemented(format!(
                        "opt-mode {} at function {i}: only {:08x} (/Ox, /O2) and \
                         {:08x} (/O1) are implemented{}. See docs/OPT_MODE.md.",
                        match other {
                            Some(v) => format!("{v:08x}"),
                            None => "unreadable".to_string(),
                        },
                        c2_il::OPT_WORD_OX,
                        c2_il::OPT_WORD_O1,
                        match other {
                            Some(0x0080_0005) => " — that is /Od",
                            Some(0x0080_0004) => " — that is #pragma optimize(\"\", off)",
                            _ => "",
                        },
                    )))
                }
            };
            match mode {
                None => mode = Some(m),
                Some(prev) if prev == m => {}
                Some(_) => {
                    return Err(BackendError::NotImplemented(
                        "mixed optimization modes in one TU (a `#pragma optimize` \
                         between functions): the per-function shape is modeled but \
                         emitting two modes into one obj is not characterized"
                            .to_string(),
                    ))
                }
            }
        }
        let mode = mode.unwrap_or(codegen::OptMode::Ox);

        // W4b2: a single-function TU whose body is a framed non-leaf call
        // (`return g(a) + k`) takes the dedicated 6-section path — it needs a
        // `.pdata` unwind section and the compiler label symbols, which the
        // straight-line/tail-call 5-section emitter does not model.
        //
        // Guarded on `!fn_level_linking`: this shortcut used to run *ahead* of the
        // `/Gy` branch, so a single-function framed TU under `/Gy` took the packed
        // 6-section path and the refusal inside that branch was unreachable. It
        // mis-emitted `mvp_framed.cpp` (divergence at obj offset 217), invisible
        // until `scripts/mode_lane.sh` compiled the fixtures with `/Gy` for the
        // first time. A dead guard that reads as live is worse than no guard.
        if funcs.len() == 1 && !self.fn_level_linking {
            if let Some(fc) = &funcs[0].framed_call {
                let text = codegen::framed_call_text(fc.add_k);
                let bytes =
                    coff::emit_framed_obj(obj_name, &funcs[0].mangled_name, &fc.callee, &text);
                return Ok(ObjImage::new(bytes));
            }
        }

        // Under function-level linking every function gets its own COMDAT
        // `.text` section, so the texts are kept separate rather than packed.
        if self.fn_level_linking {
            let mut texts: Vec<Vec<u8>> = Vec::with_capacity(funcs.len());
            let mut placed: Vec<coff::Function> = Vec::with_capacity(funcs.len());
            for f in &funcs {
                // The framed non-leaf path owns its whole 6-section obj shape
                // (it needs `.pdata`), which is not modeled per-COMDAT.
                if f.framed_call.is_some() {
                    return Err(BackendError::NotImplemented(
                        "framed non-leaf call under function-level linking (/Gy): \
                         the .pdata shape is not modeled per COMDAT section"
                            .to_string(),
                    ));
                }
                let (text, call) = if let Some(callee) = &f.tail_call {
                    // Each function's text starts at offset 0 of its own COMDAT
                    // section, so the branch offset is just the setup's length.
                    let (t, reloc) = if let Some(sources) = &f.arg_sources {
                        let mut t = codegen::permute_args_text(sources)?;
                        let branch_off = t.len() as u32;
                        t.extend_from_slice(&codegen::encode_tail_branch(branch_off));
                        (t, branch_off)
                    } else {
                        codegen::int_tail_call_text(f, 0, mode)?
                    };
                    (t, Some(coff::Call { reloc_offset: reloc, callee: callee.as_str() }))
                } else if f.empty_body {
                    (codegen::encode_blr().to_vec(), None)
                } else if let Some(t) = codegen::indirect_load_text(f) {
                    (t?, None)
                } else if let Some(cmp) = &f.compare {
                    (codegen::compare_leaf_text(cmp, mode)?, None)
                } else if let Some(double) = f.float_leaf {
                    let (t, consts) = codegen::float_leaf_text(f, double)?;
                    // A pooled FP constant still refuses under `/Gy`. Its section
                    // placement *is* now characterized — each `.rdata` COMDAT sits
                    // immediately after the `.text` of the function that first
                    // references it — but `docs/OBJ_GY_SHAPES.md` §2 also found that
                    // several constants introduced by ONE function are appended in
                    // **reverse** first-reference order, and a per-reference-site
                    // appender would emit them forwards. Every relocation still
                    // resolves either way, so that is a silent wrong-bytes shape
                    // rather than a crash, and it is not worth opening on one
                    // ordering probe.
                    if !consts.is_empty() {
                        return Err(BackendError::NotImplemented(
                            "pooled floating-point constant under function-level \
                             linking (/Gy): sections interleave per first-referencing \
                             function, but several constants from one function are \
                             appended in reverse reference order and that is not yet \
                             modeled"
                                .to_string(),
                        ));
                    }
                    (t, None)
                } else {
                    (codegen::select_text(f, mode)?, None)
                };
                placed.push(coff::Function { name: &f.mangled_name, text_offset: 0, call, is_float: f.float_leaf.is_some(), fp_refs: Vec::new() });
                texts.push(text);
            }
            return Ok(ObjImage::new(coff::emit_comdat_obj(obj_name, &placed, &texts)));
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
            // An empty body is a bare `blr` — no expression to select.
            if f.empty_body {
                text.extend_from_slice(&codegen::encode_blr());
                placed.push(coff::Function { name: &f.mangled_name, text_offset: off, call: None, is_float: f.float_leaf.is_some(), fp_refs: Vec::new() });
                continue;
            }
            // W13a: an FP leaf has its own register model entirely (pool
            // [f0, f13..f1], result f1, no accumulator collapse).
            if let Some(double) = f.float_leaf {
                let (body, consts) = codegen::float_leaf_text(f, double)?;
                text.extend_from_slice(&body);
                // W13b: rebase each constant reference site onto the whole .text.
                let fp_refs = consts
                    .into_iter()
                    .map(|r| codegen::FpConstRef { hi_off: r.hi_off + off, ..r })
                    .collect();
                placed.push(coff::Function { name: &f.mangled_name, text_offset: off, call: None, is_float: true, fp_refs });
                continue;
            }
            // An indirect-load leaf (`return *p;` / `return s->m;`) is a single
            // `lwz` + `blr`, recognized by an exact two-op stream rather than
            // reaching the affine selector — see `codegen::indirect_load_text`.
            if let Some(body) = codegen::indirect_load_text(f) {
                text.extend_from_slice(&body?);
                placed.push(coff::Function { name: &f.mangled_name, text_offset: off, call: None, is_float: false, fp_refs: Vec::new() });
                continue;
            }
            // W6: a comparison leaf lowers to its own branchless spine rather
            // than through the operand-stack selector.
            if let Some(cmp) = &f.compare {
                text.extend_from_slice(&codegen::compare_leaf_text(cmp, mode)?);
                placed.push(coff::Function { name: &f.mangled_name, text_offset: off, call: None, is_float: f.float_leaf.is_some(), fp_refs: Vec::new() });
                continue;
            }
            let call = if let Some(callee) = &f.tail_call {
                // Tail call. A void bare call (`ops` empty) is a single
                // `b <callee>` (REL24) at this offset; an integer tail call
                // (`ops` = the argument sub-expression) first computes the
                // argument into r3, then branches (the branch, not the function
                // start, is the reloc site).
                let reloc_offset = if let Some(sources) = &f.arg_sources {
                    // Multi-argument tail call: the parameters are already in
                    // r3.., so the setup is a register permutation (empty when the
                    // call passes them straight through), then the branch.
                    let moves = codegen::permute_args_text(sources)?;
                    let branch_off = off + moves.len() as u32;
                    text.extend_from_slice(&moves);
                    text.extend_from_slice(&codegen::encode_tail_branch(branch_off));
                    branch_off
                } else if f.ops.is_empty() {
                    text.extend_from_slice(&codegen::encode_tail_branch(off));
                    off
                } else {
                    let (body, branch_off) = codegen::int_tail_call_text(f, off, mode)?;
                    text.extend_from_slice(&body);
                    branch_off
                };
                Some(coff::Call {
                    reloc_offset,
                    callee,
                })
            } else {
                text.extend_from_slice(&codegen::select_text(f, mode)?);
                None
            };
            placed.push(coff::Function {
                name: &f.mangled_name,
                text_offset: off,
                call,
                is_float: f.float_leaf.is_some(),
                fp_refs: Vec::new(),
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
