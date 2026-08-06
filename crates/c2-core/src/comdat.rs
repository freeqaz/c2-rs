//! **The port's per-function `/Gy` entry point** — one function's complete
//! `.text` COMDAT body, plus everything the obj writer needs to place it.
//!
//! # Why this module exists (board #322)
//!
//! `docs/FUNCTION_BYTE_MATCH.md` §3.1 records a **blind spot in the project's
//! standing per-function alarm**. FBM grades the port's output against the
//! reference obj's own COMDAT bytes and `fnbyte-differs 0` is the alarm; but
//! the harness could only ask [`crate::codegen::select_function`], whose
//! `Selected::{Tail, Framed, Seq, CondPair}` variants hand back a *fragment* —
//! the words a branch would occupy are missing, because a branch word encodes
//! its own `.text` offset and only the caller knows where the function lands.
//! The harness therefore declined to compare bytes for **9,375 functions**
//! (`partial by shape: tail 7098 · seq 2150 · framed 123 · cond-pair 4`), and a
//! wrong emit in any of them read as `differs 0`.
//!
//! **The decline reason is a statement about the PACKED emitter, and FBM's
//! denominator is the `/Gy` COMDAT population.** Under function-level linking
//! every function starts at offset **0** of its own section, so the offset the
//! harness "cannot know" is a constant. [`PortC2::build`]'s `fn_level_linking`
//! branch has always composed these bodies completely; it just did so inline,
//! where nothing but the whole-TU emitter could reach it.
//!
//! [`PortC2::build`]: crate::PortC2::build
//!
//! # The one rule this module exists to enforce
//!
//! > **There is ONE composition, and both callers run it.**
//!
//! [`PortC2::build`]'s `/Gy` branch calls [`comdat_function_body`]; so does
//! `c2-harness`'s FBM measurement. A second copy in the harness could drift
//! from the emitter and the instrument would grade a fiction — an alarm that is
//! green about code the port does not emit is worse than the blind one it
//! replaced. The same argument [`crate::codegen::function_gate`] carries for
//! the accept/refuse boundary, one level down: **one fact, one locator.**
//!
//! # What is NOT here
//!
//! * The **packed** (non-`/Gy`) composition, which rebases every branch onto a
//!   real `.text` offset. It stays inline in `build`, and it is not what FBM's
//!   denominator counts.
//! * Anything TU-wide: the emission order, the compiler-label counter, the
//!   `/EHsc` label lead, the symbol table. Those are properties of the obj, not
//!   of a function body, and none of them changes one `.text` byte.
//! * A pooled floating-point constant, which the `/Gy` path **refuses**
//!   (`docs/OBJ_GY_SHAPES.md` §2 — the per-function `.rdata` COMDATs interleave,
//!   and several constants from one function append in reverse reference order).
//!   The refusal is returned as an ordinary [`BackendError::NotImplemented`],
//!   the same one `build` returned inline before this module existed.

use crate::codegen::{self, OptMode};
use crate::coff;
use crate::{data_refs_of, BackendError};
use c2_il::IlFunction;

/// One function's complete `/Gy` COMDAT body and its obj-side attachments.
///
/// `text` is the whole `.text` COMDAT payload — every word, including the
/// branches whose absence from [`crate::codegen::Selected`] is the reason this
/// module exists. Byte-for-byte what [`crate::PortC2::build`] puts in the obj
/// under function-level linking, because `build` gets it from here.
pub struct ComdatBody<'a> {
    /// The complete `.text` COMDAT bytes for this function.
    pub text: Vec<u8>,
    /// Every REL24 site, at an offset **within this function's own section**.
    pub calls: Vec<coff::Call<'a>>,
    /// `Some` iff this function establishes a stack frame (drives `.pdata`).
    pub frame: Option<coff::Frame>,
    /// Named-data-symbol address references, offsets within this section.
    pub data_refs: Vec<coff::DataRef<'a>>,
}

/// **Build one function's complete `.text` COMDAT body**, exactly as
/// [`crate::PortC2::build`] does under `/Gy` — because `build` calls this.
///
/// `Err` is the port's honest refusal for this function, in the same two
/// flavours `build` has always produced: the selector declined the shape, or
/// the `/Gy` composition declined it (a pooled FP constant), or the data-symbol
/// relocation site could not be derived from the body.
pub fn comdat_function_body<'a>(
    f: &'a IlFunction,
    mode: OptMode,
) -> Result<ComdatBody<'a>, BackendError> {
    let mut frame: Option<coff::Frame> = None;
    let (text, calls) = match codegen::select_function(f, mode)? {
        // A framed non-leaf call gets its own `.text` COMDAT like any other
        // function, plus a `.pdata` COMDAT associated to it (W-UNW-1).
        // `Selected::Framed` carries no bytes for the same reason
        // `Selected::Tail` carries an incomplete text: the branch word encodes
        // its own `.text` offset, so only the caller — which knows where the
        // function lands — can finish it. Under `/Gy` that offset is 0, because
        // each function starts its own section.
        codegen::Selected::Framed { setup } => {
            let fc = f.framed_call.as_ref().expect("Framed implies framed_call");
            let body = codegen::framed_call_text(
                &setup,
                fc.add_k,
                0,
                codegen::FrameLayout::default(),
            )?;
            frame = Some(coff::Frame {
                prolog_len: body.prolog_len,
                func_len: body.text.len() as u32,
            });
            (
                body.text,
                vec![coff::Call {
                    reloc_offset: body.bl_offset,
                    callee: fc.callee.as_str(),
                }],
            )
        }
        // A Class A many-call body: the same frame and `.pdata`, with one REL24
        // site per call instead of one per function.
        codegen::Selected::Seq { setups, tail } => {
            let seq = f.call_seq.as_ref().expect("Seq implies call_seq");
            // **W10** — the guard, when there is one. Resolved through
            // `seq_guard_emit` on both emission paths, so the packed and COMDAT
            // writers cannot disagree about a branch sense.
            let guard = seq
                .guard
                .as_ref()
                .map(codegen::seq_guard_emit)
                .transpose()?;
            // **W11** — the guarded early returns, resolved through the same
            // `seq_early_emit` on both emission paths for the same reason: the
            // packed and COMDAT writers must not disagree about a branch sense
            // or a block layout.
            let early = seq
                .early
                .iter()
                .map(codegen::seq_early_emit)
                .collect::<Result<Vec<_>, _>>()?;
            let body = codegen::call_seq_text(
                &setups,
                &tail,
                0,
                codegen::FrameLayout {
                    saved_gprs: seq.saved_gprs() as u8,
                    ..Default::default()
                },
                guard.as_ref(),
                &early,
                mode,
            )?;
            frame = Some(coff::Frame {
                prolog_len: body.prolog_len,
                func_len: body.text.len() as u32,
            });
            let calls = body
                .bl_offsets
                .iter()
                .zip(&seq.calls)
                .map(|(off, c)| coff::Call {
                    reloc_offset: *off,
                    callee: c.callee.as_str(),
                })
                .collect();
            (body.text, calls)
        }
        // A pooled FP constant still refuses under `/Gy`. Its section placement
        // *is* now characterized — each `.rdata` COMDAT sits immediately after
        // the `.text` of the function that first references it — but
        // `docs/OBJ_GY_SHAPES.md` §2 also found that several constants
        // introduced by ONE function are appended in **reverse** first-reference
        // order, and a per-reference-site appender would emit them forwards.
        // Every relocation still resolves either way, so that is a silent
        // wrong-bytes shape rather than a crash, and it is not worth opening on
        // one ordering probe.
        codegen::Selected::Float { consts, .. } if !consts.is_empty() => {
            return Err(BackendError::NotImplemented(
                "pooled floating-point constant under function-level \
                 linking (/Gy): sections interleave per first-referencing \
                 function, but several constants from one function are \
                 appended in reverse reference order and that is not yet \
                 modeled"
                    .to_string(),
            ))
        }
        // **W8 — a two-arm conditional tail call.** Two REL24 sites, one per
        // arm, in block order; the conditional branch between them carries its
        // own displacement and NO relocation (`docs/CFG_SHAPE.md` §3.3). Under
        // `/Gy` the function starts at offset 0 of its own COMDAT, so each tail
        // branch's word is `-(its offset within this text)`.
        codegen::Selected::CondPair(parts) => {
            let cp = f.cond_pair.as_ref().expect("CondPair implies cond_pair");
            let mut t = parts.text;
            let mut calls = Vec::with_capacity(2);
            for (off, callee) in parts
                .branch_offsets
                .iter()
                .zip([cp.then_arm.callee.as_str(), cp.else_arm.callee.as_str()])
            {
                let w = codegen::encode_tail_branch(*off);
                t[*off as usize..*off as usize + 4].copy_from_slice(&w);
                calls.push(coff::Call {
                    reloc_offset: *off,
                    callee,
                });
            }
            (t, calls)
        }
        // Each function's text starts at offset 0 of its own COMDAT section, so
        // the branch offset is just the setup's length.
        codegen::Selected::Tail(mut t) => {
            let branch_off = t.len() as u32;
            t.extend_from_slice(&codegen::encode_tail_branch(branch_off));
            let callee = f.tail_call.as_deref().expect("Tail implies tail_call");
            (
                t,
                vec![coff::Call {
                    reloc_offset: branch_off,
                    callee,
                }],
            )
        }
        codegen::Selected::Float { text, .. } => (text, Vec::new()),
        codegen::Selected::Plain(t) => (t, Vec::new()),
    };
    // Under `/Gy` each function starts at offset 0 of its own COMDAT.
    let data_refs = data_refs_of(f, &text, 0)?;
    Ok(ComdatBody {
        text,
        calls,
        frame,
        data_refs,
    })
}
