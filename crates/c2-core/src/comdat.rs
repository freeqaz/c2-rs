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
    /// **W-INLFENCE** — the composed body emits a `REL24` against a callee this
    /// TU **defines** and whose own lowered body the port can see is small
    /// enough that c2 expands it. The port would be emitting a call c2 does not.
    ///
    /// A **fourth** post-lowering stage, and the reason it is post-lowering is
    /// [`ComdatDecline::DataRef`]'s: the question is asked of the *composed
    /// body's relocation sites*, which do not exist until after lowering, so
    /// there is nothing for a parser clause to test. Asking it in the parser is
    /// strictly worse and was measured — see [`fenced_inlined_callee`].
    InlinedCallee(BackendError),
}

impl ComdatDecline {
    /// The underlying refusal, for a caller that only needs to propagate it.
    pub fn into_error(self) -> BackendError {
        match self {
            ComdatDecline::Selector(e)
            | ComdatDecline::Shape(e)
            | ComdatDecline::DataRef(e)
            | ComdatDecline::InlinedCallee(e) => e,
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
        codegen::Selected::GuardRetChain => "guard-ret-chain",
        codegen::Selected::XlrcCreateGuard => "xlrc-create-guard",
        codegen::Selected::JsonUtf8Copy => "json-utf8-copy",
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
    let body = body_of(f, selected, mode, tu, true)?;
    fenced_inlined_callee(f, &body, mode, tu)?;
    Ok(body)
}

/// **W-INLFENCE — refuse a body that emits a call c2 does not emit.**
///
/// `docs/INLINE_PREDICATE.md`'s title is the whole of it: *"when c2 does not
/// emit the call the IL contains"*. [`crate::splice`] is the half that
/// **performs** the expansion when it can prove the whole body is nothing but
/// the call. This is the other half: when the port cannot perform it but can
/// still prove c2 **would**, the port must return `NotImplemented` rather than
/// emit a `bl` c2 replaced with the callee's body. `CLAUDE.md`'s cardinal rule,
/// and board #232's shape — a refusal that became a wrong emit.
///
/// # The predicate
///
/// The composed body relocates against a name **this TU defines**, and the port
/// can lower that callee, and the callee's own lowered `/Gy` body is at most
/// [`crate::splice::INLINE_UNBOUNDED_BYTES`] bytes.
///
/// **No new constant is minted.** That is `w-splice`'s S7 bound —
/// `INLINE_PREDICATE.md` §2's `N_max` is UNBOUNDED at `index <= 64` in *both*
/// linkage classes and `index <= s`, so a callee whose emitted body is at most
/// 64 bytes is inlined at every site whatever its linkage, its parameter count,
/// its `inline` keyword or the model's unreadable `leaf` bit. `splice.rs` reads
/// that claim as *"the port MAY expand this"*; this reads the identical claim as
/// *"the port MUST NOT emit a call to this"*. One constant, two consequences.
///
/// # Why a mis-prediction here cannot cost a byte
///
/// `docs/whitebox/WB_INLINE_FINDINGS.md` §7 offers only **decline** rules and
/// says the accept side is not offered, because *"a mis-predicted accept is a
/// wrong obj"*. That warning is about a lane that would **perform** the inline.
/// Here the accept prediction drives a **refusal**: predicting "c2 inlines this"
/// when c2 did not makes the port decline a function it would have got right —
/// which costs reach and cannot produce a wrong emit. The hazard is inverted,
/// and this is the one place the accept side is safe to consult.
///
/// # Why it is not in the parser
///
/// `IlBundle::functions` already carries the parser-side form — *any* callee
/// this TU defines refuses the **whole TU** — and it stays exactly as it is.
/// The parser cannot ask this question per function, because whether the port
/// still emits the call is decided *after* mechanism E (the callee reduces to
/// nothing, so the call is dropped) and mechanism I (the body is replaced by
/// the callee's, so the REL24 becomes the callee's own). A parser clause would
/// fire on both of those and un-ship them. Measured on the 878-TU workload: the
/// coarse parser-shaped form costs **1,074** byte-exact functions; this one
/// costs **0**. `work/w-inlfence2/crossing.md`.
///
/// # What it deliberately does NOT refuse
///
/// A callee this TU defines that the port **cannot lower**. 1,081 byte-exact
/// functions call one, and every one of them is a callee of 65–308 emitted
/// bytes, which is the class c2 **keeps the call to** (`WB_INLINE_FINDINGS` F1;
/// re-measured here at 1,071 right against 7 wrong above ~80 B). Refusing those
/// would be the coarse fence and D2's decline clause forbids it.
fn fenced_inlined_callee<'a>(
    f: &'a IlFunction,
    body: &ComdatBody<'a>,
    mode: OptMode,
    tu: &TuContext<'a>,
) -> Result<(), ComdatDecline> {
    for call in &body.calls {
        if callee_is_one_c2_expands(f, call.callee, mode, tu) {
            return Err(ComdatDecline::InlinedCallee(codegen::out_of_class(&format!(
                "inlined-callee {}: this TU defines it and its lowered body is \
                 <= {} bytes, so c2 expands it and emits no call here. See \
                 c2_core::comdat::fenced_inlined_callee.",
                call.callee,
                crate::splice::INLINE_UNBOUNDED_BYTES,
            ))));
        }
    }
    Ok(())
}

/// Can the port PROVE c2 expands `name` at a site in this TU?
///
/// Every `false` is a "not proven", never a "proven not" — the fence's whole
/// contract is that it fires only on what it can establish, and leaves the rest
/// exactly where it was.
fn callee_is_one_c2_expands<'a>(
    caller: &'a IlFunction,
    name: &str,
    mode: OptMode,
    tu: &TuContext<'a>,
) -> bool {
    // **S7's varargs half, read off the mangled name** exactly as
    // `splice_body_why` reads it: `N_max = 0` categorically for a varargs
    // callee (`INLINE_PREDICATE.md` §6.18.5, and `WB_INLINE_FINDINGS` F5 on 6
    // compiled cells). c2 keeps the call, so the port's is right.
    if name.ends_with("ZZ") {
        return false;
    }
    let Some((g, opt_word)) = tu.definition(name) else {
        // Not defined here (an external), or defined TWICE — `TuContext`
        // refuses an ambiguous name rather than resolving it to the first, and
        // this fence inherits that refusal rather than guessing which body's
        // size to measure.
        return false;
    };
    // **`__declspec(noinline)` is never inlined, and the port can now SEE it.**
    //
    // `c2_il::func::gl::FN_FLAG_INLINABLE` is the `.gl` function record bit
    // board **#1039** filed as undecoded and `w-inlfence2` **#2155** named as
    // this fence's missing input — *"the missing input is not definedness"*.
    // That rung's answer was the callee's SIZE; this is the second one, and it
    // is the one this TU's own reference obj turns on: `mmioClose`'s
    // `bl mmioFlush` survives at `/O1 /Oi /EHsc /GR` while eight cells of
    // `work/w-mmioclose/probe/inl.cpp` — the same shape without the attribute,
    // including a callee defined BELOW its caller and a `static` one — are all
    // expanded. Size does not separate them: `mmioFlush` is 8 bytes.
    //
    // **Asked before the size, deliberately.** `WB_INLINE_FINDINGS` §2.1's
    // candidacy ceiling is 128 instructions and §7's licensed narrowings are all
    // decline rules keyed on size; `noinline` is a **legality** fact
    // (`0x10b5c06b`, *"requires bit 6 of `[sym+0x4c]`"*), and legality is
    // checked before profitability in c2 too. Putting it after the size test
    // would give the same answer and read as if size were the primary fact.
    //
    // `None` (unasked) and `Some(true)` both fall through to exactly the
    // behaviour this function had before the bit existed.
    if g.inlinable == Some(false) {
        return false;
    }
    // **Direct recursion is never inlined** — `WB_INLINE_FINDINGS` F5, and
    // `INLINE_PREDICATE.md` §4 grades `recurse` 336/336 declined by c2 as well.
    // Compared by ADDRESS and not by name: `IlFunction::mangled_name` is the
    // positional binding, which #918 measured disagreeing with the per-record
    // one on 74,955 workload rows. A false negative here only fails to skip,
    // which costs reach and never a byte.
    if std::ptr::eq(g, caller) {
        return false;
    }
    // The callee's own mode, falling back to the caller's when the caller does
    // not track one per function — the convention `TuContext::of_named`
    // documents and `splice_body_why` already follows.
    let m = match opt_word {
        Some(w) => match codegen::opt_mode_of_word(Some(w)) {
            Ok(m) => m,
            Err(_) => return false,
        },
        None => mode,
    };
    let Ok(sel) = codegen::select_function(g, m) else {
        return false;
    };
    // **`body_of` and not `comdat_body_from_selected`** — the unfenced
    // composition. Two reasons, and both are load-bearing:
    //
    // 1. **Termination.** `A` calls `B` and `B` calls `A`, both small, is a
    //    legal TU; a fenced probe would ask the same question back and forth
    //    forever. This asks one level and stops, so there is no recursion to
    //    bound and no cycle set to carry.
    // 2. **It is the right question.** What is wanted is the callee's emitted
    //    SIZE. Whether the callee is itself refused for *its* callees says
    //    nothing about how big it is.
    let Ok(gb) = body_of(g, sel, m, tu, true) else {
        return false;
    };
    // An empty body is mechanism **E**'s (the callee reduces to nothing), not
    // this one's, and `elide.rs` has already had its say by the time the caller
    // was composed — a caller whose call E dropped has no `REL24` for this fence
    // to see. Excluded here for the same reason S7 excludes it: one function,
    // one rule.
    !gb.text.is_empty() && gb.text.len() <= crate::splice::INLINE_UNBOUNDED_BYTES
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
        // **W-IFN — the guard chain with a materialised common epilogue.** ONE
        // REL24 site and NO data reference at all. Its callee is not read out of
        // the IL: the copy arrives as an intrinsic selector with no `.gl`
        // record, so the name is minted here from the emitter's constant — the
        // only class in this match whose callee is not an `IlFunction` field.
        codegen::Selected::GuardRetChain => {
            let g = f
                .guard_ret_chain
                .as_ref()
                .expect("GuardRetChain implies guard_ret_chain");
            let body = codegen::guard_ret_chain::guard_ret_chain_text(g, 0, mode)
                .map_err(ComdatDecline::Shape)?;
            frame = Some(coff::Frame {
                prolog_len: body.prolog_len,
                func_len: body.text.len() as u32,
            });
            let calls = vec![coff::Call {
                reloc_offset: body.bl_offset,
                callee: codegen::guard_ret_chain::MEMCPY_NAME,
            }];
            // **The name goes on `helper_externals` and not into the callee
            // region**, which is the one thing about this class that is not
            // shared with its four framed neighbours. Measured on
            // `work/w-ifn/probe/lab_z.cpp`: `memcpy` lands AFTER the first
            // user's `$T2587`, where the IL-named `?gz@@YAHH@Z` in the same obj
            // sits BETWEEN its function's two `$M`s. A minted external and an
            // IL-named one are two placements, and the writer's `known` test
            // then gives the second user no second symbol — which is what the
            // same cell's `sub2` shows.
            helper_externals = vec![codegen::guard_ret_chain::MEMCPY_NAME];
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
        // **W-JSON — the UTF-16 → UTF-8 copy loop.** TWO REL24 sites and NO
        // IL-named callee at all: both relocations are the frame's
        // `__savegprlr_28`/`__restgprlr_28` pair, minted here from the layout,
        // and their two symbols go to `helper_externals` so the writer places
        // them after the `$T` label. With no ordinary callee the per-function
        // callee region is EMPTY, which is a symbol-table cell W-XLR's
        // two-callee witness does not cover.
        codegen::Selected::JsonUtf8Copy => {
            let g = f.json_utf8_copy.as_ref().expect("JsonUtf8Copy implies json_utf8_copy");
            let body = codegen::json_utf8_copy::json_utf8_copy_text(g, 0, mode)
                .map_err(ComdatDecline::Shape)?;
            frame = Some(coff::Frame {
                prolog_len: body.prolog_len,
                func_len: body.text.len() as u32,
            });
            let fr = codegen::json_utf8_copy::json_frame();
            let (Some(save), Some(rest)) = (fr.save_gpr_helper_name(), fr.rest_gpr_helper_name())
            else {
                return Err(ComdatDecline::Shape(crate::BackendError::NotImplemented(
                    "json-utf8-copy: no `__savegprlr_N` name for this layout".to_string(),
                )));
            };
            // Reverse first-reference over the two helper sites, the same rule
            // `introduced_externals` applies: the save is the prologue's word
            // and the restore is the function's last, so the restore's symbol is
            // the earlier record.
            helper_externals = vec![rest, save];
            let calls = vec![
                coff::Call { reloc_offset: body.bl_offsets[0], callee: save },
                coff::Call { reloc_offset: body.bl_offsets[1], callee: rest },
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

#[cfg(test)]
mod inlfence_tests {
    //! **W-INLFENCE — the fence in both directions.**
    //!
    //! One positive cell and SIX negative ones, and every negative cell is
    //! declined by a **different clause** of [`callee_is_one_c2_expands`] /
    //! [`fenced_inlined_callee`]. That is the `_neg`-cell discipline `w-bdnz`
    //! paid for twice: two negative cells refused by the same earlier clause
    //! are one cell, and the later clause is untested while the table says it
    //! is covered.
    //!
    //! Each negative additionally asserts the `REL24` **survives**, so no cell
    //! can pass by the body having no call at all — the confound that would
    //! make every one of them vacuous.

    use super::*;
    use crate::codegen::select_function;
    use crate::codegen::testutil::func_with;
    use c2_il::IlOp;

    /// `int g(int a) { return a + 1; }` — 8 emitted bytes, well under the 64
    /// the fence tests against. Same shape as `splice.rs`'s own `leaf`.
    fn leaf(name: &str) -> IlFunction {
        let mut f = func_with(vec![0xE309], vec![IlOp::Load(0xE309), IlOp::Lit(1), IlOp::Add]);
        f.mangled_name = name.into();
        f.data_syms.clear();
        f
    }

    /// A leaf whose lowered body is deliberately **over** 64 bytes.
    ///
    /// `a + a + a + …`, and the repetition is not laziness. The first version
    /// of this helper was `a + 1 + 2 + … + 20` and lowered to **8 bytes** — the
    /// port folds the constant chain, which is `WB_INLINE_FINDINGS` §3.1's own
    /// GRID-I v1 failure verbatim (*"c2 folds the whole chain to two words at
    /// every k, so the size axis did not occur: 159 cells that all measured the
    /// same 28-byte callee"*). There is no constant here to fold. `n2` asserts
    /// the resulting size rather than trusting this comment.
    fn big_leaf(name: &str) -> IlFunction {
        let mut ops = vec![IlOp::Load(0xE309)];
        for _ in 0..20 {
            ops.push(IlOp::Load(0xE309));
            ops.push(IlOp::Add);
        }
        let mut f = func_with(vec![0xE309], ops);
        f.mangled_name = name.into();
        f.data_syms.clear();
        f
    }

    /// `int f(int a) { return g(a + 1); }` — a tail call **with an argument
    /// setup**, which is what makes `splice` decline (S3) so the port really
    /// does emit the `bl`. Without the setup, mechanism I takes the body and
    /// there is no relocation for this fence to see — that is cell N5.
    fn caller_with_setup(name: &str, callee: &str) -> IlFunction {
        let mut f = func_with(vec![0xE309], vec![IlOp::Load(0xE309), IlOp::Lit(1), IlOp::Add]);
        f.mangled_name = name.into();
        f.data_syms.clear();
        f.tail_call = Some(callee.into());
        f
    }

    /// The whole composition, selector included — so a cell whose *callee* the
    /// selector refuses (N6) gets an `Err` here rather than a panic. Before this
    /// took the selector's own refusal, `n6` unwrapped it and died on the line
    /// that was supposed to be its precondition.
    fn compose<'a>(funcs: &'a [IlFunction], i: usize) -> Result<ComdatBody<'a>, ComdatDecline> {
        let tu = TuContext::of(funcs);
        let sel = select_function(&funcs[i], OptMode::O1).map_err(ComdatDecline::Selector)?;
        comdat_body_from_selected(&funcs[i], sel, OptMode::O1, &tu)
    }

    fn is_fenced(r: &Result<ComdatBody<'_>, ComdatDecline>) -> bool {
        matches!(r, Err(ComdatDecline::InlinedCallee(_)))
    }

    /// **P — the positive cell.** The port would emit `b ?g` where c2 emits
    /// `?g`'s own body, because `?g` is defined here and is 8 bytes.
    #[test]
    fn p_a_call_to_a_small_same_tu_callee_is_refused() {
        let funcs = vec![leaf("?g@@YAHH@Z"), caller_with_setup("?f@@YAHH@Z", "?g@@YAHH@Z")];
        let g = compose(&funcs, 0).expect("the leaf lowers");
        assert!(
            g.text.len() <= crate::splice::INLINE_UNBOUNDED_BYTES,
            "the positive cell needs a callee UNDER the bound; got {} bytes",
            g.text.len()
        );
        assert!(
            is_fenced(&compose(&funcs, 1)),
            "THE PORT EMITTED A CALL c2 REPLACES WITH THE CALLEE'S BODY. Board \
             #232's shape, and CLAUDE.md's rule is that outside its class the \
             port returns NotImplemented"
        );
    }

    /// **N1 — `tu.definition` is `None`.** A true external: c2 has no body to
    /// expand and the port's branch is right. The clause that keeps every
    /// ordinary tail call in class.
    #[test]
    fn n1_an_external_callee_is_untouched() {
        let funcs = vec![caller_with_setup("?f@@YAHH@Z", "?ext@@YAHH@Z")];
        let body = compose(&funcs, 0).expect("an external callee is not this fence's business");
        assert_eq!(
            body.calls.len(),
            1,
            "the REL24 must SURVIVE — a cell that passes because the body has \
             no call at all tests nothing"
        );
        assert_eq!(body.calls[0].callee, "?ext@@YAHH@Z");
    }

    /// **N2 — the SIZE clause.** Defined here, lowerable, and over the bound,
    /// so c2 keeps the call. On the workload 1,071 such callers are byte-EXACT
    /// today and 7 are not (`work/w-inlfence2/crossing.md` §2) — which is why
    /// refusing them is the coarse fence decline clause D2 rejected.
    #[test]
    fn n2_a_large_same_tu_callee_is_untouched() {
        let funcs = vec![big_leaf("?g@@YAHH@Z"), caller_with_setup("?f@@YAHH@Z", "?g@@YAHH@Z")];
        let g = compose(&funcs, 0).expect("the big leaf lowers");
        assert!(
            g.text.len() > crate::splice::INLINE_UNBOUNDED_BYTES,
            "N2 IS CONFOUNDED: its callee is {} bytes, inside the bound, so the \
             cell would be decided by the clause the positive one fires on",
            g.text.len()
        );
        let body = compose(&funcs, 1).expect("c2 keeps a call to a callee this large");
        assert_eq!(body.calls.len(), 1, "the REL24 must SURVIVE");
    }

    /// **N3 — the VARARGS clause**, read off the mangled `ZZ` exactly as
    /// `splice_body_why` reads it. `N_max = 0` categorically
    /// (`INLINE_PREDICATE.md` §6.18.5, `WB_INLINE_FINDINGS` F5, 6 cells).
    #[test]
    fn n3_a_varargs_callee_is_untouched() {
        let funcs = vec![leaf("?g@@YAHHZZ"), caller_with_setup("?f@@YAHH@Z", "?g@@YAHHZZ")];
        let g = compose(&funcs, 0).expect("the leaf lowers");
        assert!(
            g.text.len() <= crate::splice::INLINE_UNBOUNDED_BYTES,
            "N3 IS CONFOUNDED: the size clause would decide it too"
        );
        let body = compose(&funcs, 1).expect("c2 never inlines a varargs callee");
        assert_eq!(body.calls.len(), 1, "the REL24 must SURVIVE");
    }

    /// **N4 — DIRECT RECURSION**, compared by ADDRESS and not by name.
    /// `INLINE_PREDICATE.md` §4 grades `recurse` 336/336 declined by c2.
    #[test]
    fn n4_direct_recursion_is_untouched() {
        let funcs = vec![caller_with_setup("?f@@YAHH@Z", "?f@@YAHH@Z")];
        let body = compose(&funcs, 0).expect("c2 never inlines direct recursion");
        assert_eq!(body.calls.len(), 1, "the REL24 must SURVIVE");
        assert_eq!(body.calls[0].callee, "?f@@YAHH@Z");
    }

    /// **N5 — MECHANISM I already handled it.** With no argument setup the
    /// splice fires, the caller's body **is** the callee's, and no `REL24`
    /// against the callee exists for the fence to see.
    ///
    /// This is the cell that says the fence is in the right PLACE: a
    /// parser-side clause fires here on the IL's call token and un-ships
    /// `w-splice`.
    #[test]
    fn n5_a_body_mechanism_i_replaced_has_no_call_to_fence() {
        let mut caller = func_with(vec![0xE309], Vec::new());
        caller.mangled_name = "?f@@YAHH@Z".into();
        caller.data_syms.clear();
        caller.tail_call = Some("?g@@YAHH@Z".into());
        let funcs = vec![leaf("?g@@YAHH@Z"), caller];
        let body = compose(&funcs, 1).expect("the splice fires and there is no call left");
        assert!(
            body.calls.is_empty(),
            "N5 IS CONFOUNDED: the splice did not fire, so this cell is really \
             the positive one"
        );
        let g = compose(&funcs, 0).expect("the leaf lowers");
        assert_eq!(body.text, g.text, "SPLICE-0: the caller's body IS the callee's");
    }

    /// **N6 — defined here and the port CANNOT LOWER IT.** The fence fires only
    /// on what it can PROVE, so an unlowerable callee leaves the call alone.
    ///
    /// The clause that carries the whole residue: 1,081 byte-exact functions on
    /// the workload call one, and every one is a callee of 65-308 emitted
    /// bytes — the class c2 keeps the call to.
    #[test]
    fn n6_an_unlowerable_same_tu_callee_is_untouched() {
        let mut bad = func_with(vec![0xE309], vec![IlOp::Load(0xE309), IlOp::Div]);
        bad.mangled_name = "?g@@YAHH@Z".into();
        bad.data_syms.clear();
        let funcs = vec![bad, caller_with_setup("?f@@YAHH@Z", "?g@@YAHH@Z")];
        assert!(
            compose(&funcs, 0).is_err(),
            "N6 IS CONFOUNDED: the port CAN lower this callee, so the cell is \
             really testing the size clause"
        );
        let body = compose(&funcs, 1)
            .expect("an unlowerable callee is a size the port cannot see, and it must not guess");
        assert_eq!(body.calls.len(), 1, "the REL24 must SURVIVE");
    }

    /// **THE MUST-FAIL PAIR.** One caller, two callees differing in exactly one
    /// field — the callee's emitted size — must land on opposite sides. If both
    /// landed together the fence would be reading something else.
    #[test]
    fn the_size_is_the_only_field_that_moves_the_verdict() {
        let small = vec![leaf("?g@@YAHH@Z"), caller_with_setup("?f@@YAHH@Z", "?g@@YAHH@Z")];
        let big = vec![big_leaf("?g@@YAHH@Z"), caller_with_setup("?f@@YAHH@Z", "?g@@YAHH@Z")];
        assert!(is_fenced(&compose(&small, 1)), "small callee: c2 expands it");
        assert!(!is_fenced(&compose(&big, 1)), "large callee: c2 keeps the call");
    }

    /// **N7 — `__declspec(noinline)`, the clause this lane added.**
    ///
    /// Byte for byte cell `P`'s TU, with one bit of the callee's `.gl`
    /// attribute cleared. c2 does **not** expand a `noinline` callee, so the
    /// port's `bl` is right and the fence must not fire — and the pair is what
    /// makes that a measurement rather than a claim, because the ONLY
    /// difference between the two cells is `IlFunction::inlinable`.
    ///
    /// The shape is `mmio.cpp`'s own: `mmioClose` calls `mmioFlush`, defined in
    /// that TU, eight bytes long, `__declspec(noinline)` — and the reference obj
    /// keeps the `bl`. `work/w-mmioclose/probe/inl.cpp` is the control from the
    /// other side: eight cells of the same shape WITHOUT the attribute and c2
    /// expands seven of them, so size does not separate this pair.
    #[test]
    fn n7_a_noinline_same_tu_callee_is_untouched() {
        let mut g = leaf("?g@@YAHH@Z");
        g.inlinable = Some(false);
        let funcs = vec![g, caller_with_setup("?f@@YAHH@Z", "?g@@YAHH@Z")];
        let body = compose(&funcs, 1).expect("c2 keeps the call, so the port may emit it");
        assert_eq!(
            body.calls.len(),
            1,
            "the REL24 must SURVIVE — a cell that passes because the body has \
             no call at all tests nothing"
        );
        assert_eq!(body.calls[0].callee, "?g@@YAHH@Z");
    }

    /// **N7's must-fail mutation, and it is the pair `P` cannot supply on its
    /// own.** `Some(true)` and `None` are the two values that are NOT the
    /// attribute, and both must land exactly where the fence has always put
    /// them. A clause written `!= Some(true)` would pass N7 and fail here.
    #[test]
    fn n7_only_some_false_moves_the_fence() {
        for flag in [None, Some(true)] {
            let mut g = leaf("?g@@YAHH@Z");
            g.inlinable = flag;
            let funcs = vec![g, caller_with_setup("?f@@YAHH@Z", "?g@@YAHH@Z")];
            assert!(
                is_fenced(&compose(&funcs, 1)),
                "inlinable = {flag:?} must leave the fence exactly where it was: \
                 None is UNASKED and Some(true) is a positive permission, and \
                 neither is `__declspec(noinline)`"
            );
        }
    }
}
