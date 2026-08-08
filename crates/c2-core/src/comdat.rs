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
use crate::elide::drops_tail_call;
use crate::splice::{splice_body, TuContext};
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
        codegen::Selected::IfCallJoin => "if-call-join",
        codegen::Selected::GuardChainSharedTail => "guard-chain-shared-tail",
        codegen::Selected::AllocInitOrFail => "alloc-init-or-fail",
        codegen::Selected::OsfHandleGuard => "osf-handle-guard",
        codegen::Selected::XlrcCreateGuard => "xlrc-create-guard",
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
    /// **W-DATA** — data objects this function DEFINES, with their REFHI/REFLO
    /// sites at offsets within this section. See [`coff::DataDef`].
    pub data_defs: Vec<coff::DataDef<'a>>,
    /// **W-XLR** — undefined externals whose symbol records go AFTER the `$T`
    /// label, in emission order. See [`coff::Function::helper_externals`]; empty
    /// for every shape but the Class C frame.
    pub helper_externals: Vec<&'a str>,
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
    tu: &TuContext<'a>,
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
/// `tu` is the bundle's own facts — mechanism E's callees
/// ([`crate::elide::TuEmptyCallees`], reached through [`TuContext`]'s `Deref`)
/// and mechanism I's splice sources ([`crate::splice`]). It is a required
/// parameter rather than an `Option` for the reason those modules' docs give:
/// the two mechanisms are the facts in this composition that are *not*
/// properties of one function, and a caller that forgot to supply them would
/// silently emit a call c2 does not emit. Pass [`TuContext::none`] to state "no
/// bundle, therefore neither mechanism" out loud.
pub fn comdat_body_from_selected<'a>(
    f: &'a IlFunction,
    selected: codegen::Selected,
    mode: OptMode,
    tu: &TuContext<'a>,
) -> Result<ComdatBody<'a>, ComdatDecline> {
    body_of(f, selected, mode, tu, true)
}

/// [`comdat_body_from_selected`] with the splice's **re-entry** switch exposed.
///
/// `allow_splice` is `true` for every caller but one: [`crate::splice`]'s walk
/// composes the chain's END with `false`, because it has already established —
/// by asking the predicate itself, link by link — that this body does not
/// splice. Asking again here would be the same question with a second
/// implementation, and it is the recursion that would then have no base case.
///
/// It is a boolean and not a depth counter on purpose: the depth is bounded by
/// the walk's own `seen` set and its ceiling, in `splice.rs`, where the
/// termination argument lives beside the cycle refusal it depends on.
pub(crate) fn body_of<'a>(
    f: &'a IlFunction,
    selected: codegen::Selected,
    mode: OptMode,
    tu: &TuContext<'a>,
    allow_splice: bool,
) -> Result<ComdatBody<'a>, ComdatDecline> {
    let shape = selected_tag(&selected);
    // **MECHANISM I — the call c2 replaced the caller's whole body with**
    // (`crate::splice`, `docs/INLINE_PREDICATE.md` §2, `w-seq` §4.1). A caller
    // whose emitted body is nothing but one call to a same-TU callee the port
    // lowers emits **the callee's body**: no branch, no REL24 against the
    // callee, no frame — and the callee's own relocations, at the callee's own
    // offsets.
    //
    // Asked ahead of the match, and after mechanism E inside the predicate
    // (S9), because it replaces the body of two different `Selected` variants
    // and a guard on each arm would be the same rule written twice. The `shape`
    // tag is kept as the CALLER's own selection, so `fnbyte-shape|tail|exact`
    // still counts what the selector chose and not what the composition did
    // with it.
    if allow_splice {
        if let Some(body) = splice_body(f, &selected, mode, tu)? {
            return Ok(ComdatBody { shape, ..body });
        }
    }
    let mut frame: Option<coff::Frame> = None;
    // **W-XLR** — filled by the one arm whose frame mints externals of its own.
    let mut helper_externals: Vec<&'a str> = Vec::new();
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
        // W-CFG1 — the `if`/`else`-with-a-join. Built at 0 because each
        // function is its own COMDAT here, which is what its two `bl`
        // displacements are relative to.
        // **W-EXTDATA — the sunk-`||`-guard body.** Built at 0 for the reason
        // every framed shape here is: each function is its own COMDAT under
        // `/Gy`, which is what its four `bl` displacements are relative to.
        codegen::Selected::GuardChainSharedTail => {
            let g = f
                .guard_chain_shared_tail
                .as_ref()
                .expect("GuardChainSharedTail implies guard_chain_shared_tail");
            let body =
                codegen::guard_chain_shared_tail::guard_chain_shared_tail_text(g, 0, mode)
                    .map_err(ComdatDecline::Shape)?;
            frame = Some(coff::Frame {
                prolog_len: body.prolog_len,
                func_len: body.text.len() as u32,
            });
            // Four sites, three names: `errno` is called from BOTH arms and the
            // symbol is emitted once, which is `introduced_externals`' own dedup
            // and the reason these are zipped by SITE and not by name.
            let calls = body
                .bl_offsets
                .iter()
                .zip([
                    g.helper.as_str(),
                    g.errno.as_str(),
                    g.errno.as_str(),
                    g.invalid.as_str(),
                ])
                .map(|(off, callee)| coff::Call { reloc_offset: *off, callee })
                .collect();
            (body.text, calls)
        }
        // **W-UNDNAME — the guarded allocation with a shared error store.** ONE
        // REL24 site and TWO REFHI/REFLO quads, the latter derived from the
        // emitted words by `crate::data_refs_of` below rather than declared
        // here — which is what lets the two hoist distances differ.
        codegen::Selected::AllocInitOrFail => {
            let a = f
                .alloc_init_or_fail
                .as_ref()
                .expect("AllocInitOrFail implies alloc_init_or_fail");
            let body = codegen::alloc_init_or_fail::alloc_init_or_fail_text(a, 0, mode)
                .map_err(ComdatDecline::Shape)?;
            frame = Some(coff::Frame {
                prolog_len: body.prolog_len,
                func_len: body.text.len() as u32,
            });
            let calls = vec![coff::Call {
                reloc_offset: body.bl_offset,
                callee: a.alloc.as_str(),
            }];
            (body.text, calls)
        }
        // **W-OSFINFO — the range-and-flag guarded table lookup.** TWO REL24
        // sites and TWO REFHI/REFLO quads, the latter derived from the emitted
        // words by `crate::data_refs_of` below rather than declared here — which
        // is what lets one of them be a `lwz` displacement.
        codegen::Selected::OsfHandleGuard => {
            let g = f
                .osf_handle_guard
                .as_ref()
                .expect("OsfHandleGuard implies osf_handle_guard");
            let body = codegen::osf_handle_guard::osf_handle_guard_text(g, 0, mode)
                .map_err(ComdatDecline::Shape)?;
            frame = Some(coff::Frame {
                prolog_len: body.prolog_len,
                func_len: body.text.len() as u32,
            });
            let calls = vec![
                coff::Call { reloc_offset: body.bl_offsets[0], callee: g.errno.as_str() },
                coff::Call { reloc_offset: body.bl_offsets[1], callee: g.doserrno.as_str() },
            ];
            (body.text, calls)
        }
        // **W-XLR — the two-stage create/attach guard.** FOUR REL24 sites for
        // TWO IL-named callees: the frame's `__savegprlr_26`/`__restgprlr_26`
        // pair is minted here from the layout, never read out of the IL, and its
        // two symbols are handed to `helper_externals` so the writer places them
        // after the `$T` label instead of in the callee region.
        codegen::Selected::XlrcCreateGuard => {
            let g = f
                .xlrc_create_guard
                .as_ref()
                .expect("XlrcCreateGuard implies xlrc_create_guard");
            let body = codegen::xlrc_create_guard::xlrc_create_guard_text(g, 0, mode)
                .map_err(ComdatDecline::Shape)?;
            frame = Some(coff::Frame {
                prolog_len: body.prolog_len,
                func_len: body.text.len() as u32,
            });
            let fr = codegen::xlrc_create_guard::xlrc_frame();
            let (Some(save), Some(rest)) =
                (fr.save_gpr_helper_name(), fr.rest_gpr_helper_name())
            else {
                return Err(ComdatDecline::Shape(crate::BackendError::NotImplemented(
                    "xlrc-create-guard: no `__savegprlr_N` name for this layout".to_string(),
                )));
            };
            // Reverse first-reference over the two helper sites — the save is
            // the prologue's word and the restore is the function's last, so the
            // restore's symbol is the earlier record. Derived here rather than
            // written as a literal pair, so it stays the same rule
            // `introduced_externals` applies.
            helper_externals = vec![rest, save];
            let calls = vec![
                coff::Call { reloc_offset: body.bl_offsets[0], callee: save },
                coff::Call { reloc_offset: body.bl_offsets[1], callee: g.create.as_str() },
                coff::Call { reloc_offset: body.bl_offsets[2], callee: g.attach.as_str() },
                coff::Call { reloc_offset: body.bl_offsets[3], callee: rest },
            ];
            (body.text, calls)
        }
        codegen::Selected::IfCallJoin => {
            let j = f.if_call_join.as_ref().expect("IfCallJoin implies if_call_join");
            let body = codegen::if_call_join::if_call_join_text(j, 0, mode)
                .map_err(ComdatDecline::Shape)?;
            frame = Some(coff::Frame {
                prolog_len: body.prolog_len,
                func_len: body.text.len() as u32,
            });
            let calls = vec![
                coff::Call { reloc_offset: body.bl_offsets[0], callee: j.callee_hi.as_str() },
                coff::Call { reloc_offset: body.bl_offsets[1], callee: j.callee_lo.as_str() },
            ];
            (body.text, calls)
        }
        codegen::Selected::Seq { setups, tail, park } => {
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
                .enumerate()
                .map(|(ix, e)| codegen::seq_early_emit_remapped(e, &park, ix))
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
                &park.entry,
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
        // `docs/INLINE_PREDICATE.md` §1, §1.2). A tail call whose callee is
        // defined in this same bundle by a body that **reduces to nothing**
        // leaves no branch, no REL24 and no external symbol: c2's whole body for
        // the caller is one `blr`, and the argument setup goes with the call.
        //
        // "Reduces to nothing" is a FIXPOINT, not "empty" — `void h(){}
        // void g(){h();} void f(){g();}` drops BOTH calls. Measured on 30 graded
        // cells for the one-step rule and 94 graded call edges for the closure,
        // against real c2 at the workload's own flags AND with `/Ob0` appended;
        // the second compilation is what separates this from inline expansion,
        // which is NOT modeled here and must not be — `k12_cross_i` is a chain
        // whose every caller is a bare `blr` at `/O1` and mechanism I at `/Ob0`.
        //
        // Asked before the ordinary `Tail` arm rather than inside it, because
        // the two produce different bodies from the same selection and the
        // adjacency is the whole rule: `Selected::Tail`'s bytes are the setup,
        // and E discards them.
        codegen::Selected::Tail(_) if drops_tail_call(f, tu.empty_callees()) => {
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
    let data_defs = crate::data_defs_of(f, 0).map_err(ComdatDecline::DataRef)?;
    Ok(ComdatBody {
        shape,
        text,
        calls,
        frame,
        data_refs,
        data_defs,
        helper_externals,
    })
}

/// **What one planned relocation points at**, on the port's side of the compare.
///
/// The port has no obj here and therefore no symbol table: it knows the target
/// by NAME. `PairDisplacement` is the one field that is not a name, because a
/// `PAIR` record's index slot carries a displacement (PE/COFF rev 6.0) — every
/// one the port emits is 0, since each pooled constant gets its own COMDAT.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlanTarget<'a> {
    Symbol(&'a str),
    PairDisplacement(u32),
}

/// One relocation record the `/Gy` writer will emit for a `.text` COMDAT, with
/// the target as a name rather than as this obj's symbol index.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextReloc<'a> {
    /// Offset within the function's own COMDAT section.
    pub va: u32,
    /// The packed 16-bit `Type` word, exactly as it goes on disk.
    pub ty: u16,
    pub target: PlanTarget<'a>,
}

/// **THE PORT'S `.text` RELOCATION PLAN for one function**, in the order the
/// writer puts it on disk.
///
/// # One locator (board #880's rule, one field along)
///
/// [`crate::PortC2::build`]'s `/Gy` branch used to build this list inline; it
/// now calls **this**, and so does FUNCTION BYTE MATCH. The argument is the one
/// board #880 settled for the body composition: a second copy in the harness
/// could drift from the emitter, and *an alarm that is green about relocations
/// the port does not emit is worse than the blind one it replaced*. The writer
/// maps each [`PlanTarget::Symbol`] to that obj's symbol index; the ORDER, the
/// offsets and the type words come from here for both callers.
///
/// # The shape
///
/// * one `REL24` per call site — a tail call's `b`, a framed call's `bl`, or one
///   per call of a many-call body. Several sites may share one callee.
/// * WR1: one `REFHI` / `PAIR` / `REFLO` / `PAIR` quad per named-data-symbol
///   address, with the two halves at **`hi_off` and `lo_off`**, which are not
///   adjacent (`coff::DataRef`'s own doc records the wrong-bytes emit that
///   assuming `hi_off + 4` produced).
///
/// Sorted **ascending by `VirtualAddress`**, which is the order records in a
/// section carry. The sort is **stable**, so each quad keeps its
/// `REFHI`-before-`PAIR` order at equal `va`.
pub fn text_reloc_plan<'a>(
    calls: &[coff::Call<'a>],
    data_refs: &[coff::DataRef<'a>],
    data_defs: &[coff::DataDef<'a>],
) -> Vec<TextReloc<'a>> {
    let mut recs: Vec<TextReloc<'a>> = Vec::with_capacity(calls.len() + 4 * data_refs.len());
    for c in calls {
        recs.push(TextReloc {
            va: c.reloc_offset,
            ty: coff::REL_PPC_REL24,
            target: PlanTarget::Symbol(c.callee),
        });
    }
    for r in data_refs {
        recs.push(TextReloc {
            va: r.hi_off,
            ty: coff::REL_PPC_REFHI,
            target: PlanTarget::Symbol(r.name),
        });
        recs.push(TextReloc {
            va: r.hi_off,
            ty: coff::REL_PPC_PAIR,
            target: PlanTarget::PairDisplacement(0),
        });
        recs.push(TextReloc {
            va: r.lo_off,
            ty: coff::REL_PPC_REFLO,
            target: PlanTarget::Symbol(r.name),
        });
        recs.push(TextReloc {
            va: r.lo_off,
            ty: coff::REL_PPC_PAIR,
            target: PlanTarget::PairDisplacement(0),
        });
    }
    // **W-DATA — the same quad shape, fanned out 1:N.** One `REFHI`/`PAIR` at
    // the high half and one `REFLO`/`PAIR` at **each** low half, all against the
    // same symbol. MEASURED on `Primes.cpp`'s obj: `REFHI @0x00`,
    // `REFLO @0x08`, `REFLO @0x0c`, six records for one symbol.
    //
    // Written as its own loop rather than by widening the one above, because the
    // symbol this resolves against is DEFINED in this obj and `DataRef`'s is an
    // undefined external — two different symbol tables in the writer, and one
    // list searched for both is how a data symbol silently resolves against a
    // callee of the same spelling (`writer.rs`'s own note).
    for d in data_defs {
        recs.push(TextReloc {
            va: d.hi_off,
            ty: coff::REL_PPC_REFHI,
            target: PlanTarget::Symbol(d.symbol),
        });
        recs.push(TextReloc {
            va: d.hi_off,
            ty: coff::REL_PPC_PAIR,
            target: PlanTarget::PairDisplacement(0),
        });
        for &lo in &d.lo_offs {
            recs.push(TextReloc {
                va: lo,
                ty: coff::REL_PPC_REFLO,
                target: PlanTarget::Symbol(d.symbol),
            });
            recs.push(TextReloc {
                va: lo,
                ty: coff::REL_PPC_PAIR,
                target: PlanTarget::PairDisplacement(0),
            });
        }
    }
    recs.sort_by_key(|r| r.va);
    recs
}
