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
use crate::elide::{drops_tail_call, TuEmptyCallees};
use crate::{data_refs_of, BackendError};
use c2_il::IlFunction;

/// **Why one function has no `/Gy` body**, split by *which* stage declined.
///
/// The three are not interchangeable and an instrument that merged them would
/// mis-file its own population: the selector's refusal is the port's accept
/// boundary (`fnbyte-refused`), the shape decline is a `/Gy`-only composition
/// limit that the packed path does not have, and the data-reference decline is
/// a body the selector *did* lower whose relocation site cannot be derived from
/// it — so the obj is refused even though the `.text` bytes exist.
#[derive(Debug)]
pub enum ComdatDecline {
    /// [`crate::codegen::select_function`] refused the function outright.
    Selector(BackendError),
    /// The selector produced a body, but the `/Gy` composition has no model for
    /// this shape's obj (today: a pooled floating-point constant).
    Shape(BackendError),
    /// The body exists, but [`crate::data_refs_of`] cannot locate the
    /// data-symbol relocation halves inside it.
    DataRef(BackendError),
}

impl ComdatDecline {
    /// The underlying refusal, for a caller that only needs to propagate it.
    pub fn into_error(self) -> BackendError {
        match self {
            ComdatDecline::Selector(e) | ComdatDecline::Shape(e) | ComdatDecline::DataRef(e) => e,
        }
    }
}

impl From<ComdatDecline> for BackendError {
    fn from(d: ComdatDecline) -> BackendError {
        d.into_error()
    }
}

/// The [`crate::codegen::Selected`] variant's stable tag, for a diagnostic that
/// wants to say *which shape* it is looking at.
///
/// Deliberately a free function here rather than a method on `Selected`:
/// `crates/c2-core/src/codegen/select.rs` holds the accept/refuse boundary and
/// this lane leaves that file untouched. The strings are an interface —
/// `fnbyte-partial|tail` and friends are printed by `c2rs gap` and quoted in
/// `docs/FUNCTION_BYTE_MATCH.md` — so they must not be renamed casually.
pub fn selected_tag(s: &codegen::Selected) -> &'static str {
    match s {
        codegen::Selected::Plain(_) => "plain",
        codegen::Selected::Tail(_) => "tail",
        codegen::Selected::Float { consts, .. } if consts.is_empty() => "float",
        codegen::Selected::Float { .. } => "float-const",
        codegen::Selected::Framed { .. } => "framed",
        codegen::Selected::Seq { .. } => "seq",
        codegen::Selected::CondPair(_) => "cond-pair",
    }
}

/// One function's complete `/Gy` COMDAT body and its obj-side attachments.
///
/// `text` is the whole `.text` COMDAT payload — every word, including the
/// branches whose absence from [`crate::codegen::Selected`] is the reason this
/// module exists. Byte-for-byte what [`crate::PortC2::build`] puts in the obj
/// under function-level linking, because `build` gets it from here.
pub struct ComdatBody<'a> {
    /// Which [`crate::codegen::Selected`] shape produced this body.
    pub shape: &'static str,
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
/// `Err` is the port's honest refusal for this function, tagged with the stage
/// that produced it — see [`ComdatDecline`]. `build` propagates all three
/// identically; the FBM instrument files them in three different buckets, which
/// is the whole reason the distinction is in the type.
pub fn comdat_function_body<'a>(
    f: &'a IlFunction,
    mode: OptMode,
    tu: &TuEmptyCallees,
) -> Result<ComdatBody<'a>, ComdatDecline> {
    let selected = codegen::select_function(f, mode).map_err(ComdatDecline::Selector)?;
    comdat_body_from_selected(f, selected, mode, tu)
}

/// [`comdat_function_body`] with the selection already made — the entry point a
/// diagnostic uses when it needs the shape tag *and* the body, without running
/// the ordered dispatch twice.
///
/// `mode` is still required: `call_seq_text` reads it for the W10/W11 block
/// structure, which is the one place the two optimization modes differ by more
/// than a register field (`codegen::OptMode`'s own doc).
///
/// `tu` is the bundle's **same-TU empty-bodied callees**
/// ([`crate::elide::TuEmptyCallees`]). It is a required parameter rather than an
/// `Option` for the reason that module's docs give: mechanism E is the one fact
/// in this composition that is *not* a property of one function, and a caller
/// that forgot to supply it would silently emit a call c2 does not emit. Pass
/// [`TuEmptyCallees::none`] to state "no bundle, therefore no elision" out loud.
pub fn comdat_body_from_selected<'a>(
    f: &'a IlFunction,
    selected: codegen::Selected,
    mode: OptMode,
    tu: &TuEmptyCallees,
) -> Result<ComdatBody<'a>, ComdatDecline> {
    let shape = selected_tag(&selected);
    let mut frame: Option<coff::Frame> = None;
    let (text, calls) = match selected {
        // A framed non-leaf call gets its own `.text` COMDAT like any other
        // function, plus a `.pdata` COMDAT associated to it (W-UNW-1).
        // `Selected::Framed` carries no bytes for the same reason
        // `Selected::Tail` carries an incomplete text: the branch word encodes
        // its own `.text` offset, so only the caller — which knows where the
        // function lands — can finish it. Under `/Gy` that offset is 0, because
        // each function starts its own section.
        codegen::Selected::Framed { setup } => {
            let fc = f.framed_call.as_ref().expect("Framed implies framed_call");
            let body =
                codegen::framed_call_text(&setup, fc.add_k, 0, codegen::FrameLayout::default())
                    .map_err(ComdatDecline::Shape)?;
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
                .transpose()
                .map_err(ComdatDecline::Shape)?;
            // **W11** — the guarded early returns, resolved through the same
            // `seq_early_emit` on both emission paths for the same reason: the
            // packed and COMDAT writers must not disagree about a branch sense
            // or a block layout.
            let early = seq
                .early
                .iter()
                .map(codegen::seq_early_emit)
                .collect::<Result<Vec<_>, _>>()
                .map_err(ComdatDecline::Shape)?;
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
            )
            .map_err(ComdatDecline::Shape)?;
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
            return Err(ComdatDecline::Shape(BackendError::NotImplemented(
                "pooled floating-point constant under function-level \
                 linking (/Gy): sections interleave per first-referencing \
                 function, but several constants from one function are \
                 appended in reverse reference order and that is not yet \
                 modeled"
                    .to_string(),
            )))
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
        // **MECHANISM E — the call c2 does not emit** (`crate::elide`,
        // `docs/INLINE_PREDICATE.md` §1). A tail call whose callee is defined in
        // this same bundle with an EMPTY body leaves no branch, no REL24 and no
        // external symbol: c2's whole body for the caller is one `blr`, and the
        // argument setup goes with the call. Measured on 30 graded cells against
        // real c2 at the workload's own flags AND with `/Ob0` appended — the
        // second compilation is what separates this from inline expansion, which
        // is NOT modeled here and must not be.
        //
        // Asked before the ordinary `Tail` arm rather than inside it, because
        // the two produce different bodies from the same selection and the
        // adjacency is the whole rule: `Selected::Tail`'s bytes are the setup,
        // and E discards them.
        codegen::Selected::Tail(_) if drops_tail_call(f, tu) => {
            (codegen::encode_blr().to_vec(), Vec::new())
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
    let data_refs = data_refs_of(f, &text, 0).map_err(ComdatDecline::DataRef)?;
    Ok(ComdatBody {
        shape,
        text,
        calls,
        frame,
        data_refs,
    })
}
