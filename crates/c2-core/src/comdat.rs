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
        codegen::Selected::MemcpyTail(_) => "memcpy-tail",
        codegen::Selected::Float { consts, .. } if consts.is_empty() => "float",
        codegen::Selected::FpStoreDiamond { .. } => "fp-store-diamond",
        codegen::Selected::CtorForwardCall => "ctor-forward-call",
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
        codegen::Selected::XteaEncryptLoop => "xtea-encrypt-loop",
        codegen::Selected::JsonUtf8Copy => "json-utf8-copy",
    }
}

/// **W-BIQUAD** — the GPR the forwarding constructor parks `this` in. Named
/// here as well as in the emitter because the callee-footprint gate below has to
/// ask about it, and the two must be one number.
const PARK_GPR: u8 = 10;

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
    /// **W-BIQUAD** — pooled floating-point constant reference sites, in
    /// EMISSION order, at offsets within this section. The writer reverses that
    /// order when it mints the `.rdata` COMDATs a function introduces
    /// (`docs/OBJ_GY_SHAPES.md` §2.4 rule 3), so this list must stay in the
    /// order the words were laid down and not be pre-sorted here.
    pub fp_refs: Vec<crate::codegen::FpConstRef>,
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
/// [`INLINE_DECLINE_BYTES`] bytes.
///
/// **W-FENCE2 raised that bound from `splice::INLINE_UNBOUNDED_BYTES` (64) to
/// [`INLINE_DECLINE_BYTES`] (128), and the change of meaning is the point.** At
/// 64 the test read *"the port can PROVE c2 expands this"* — the categorical
/// accept region, used as a refusal. At 128 it reads *"the port cannot prove c2
/// KEPT this"*, and the difference is the **mixed band**: GRID-W measures 64–95
/// emitted bytes as 146 calls c2 kept against 570 it inlined, which is a region
/// no rule may answer in either direction. `splice.rs`'s S7 is **unchanged** —
/// the port still only *performs* an expansion inside the categorical accept
/// region, and this constant is not that one.
///
/// # Why a mis-prediction here cannot cost a byte
///
/// `docs/whitebox/WB_INLINE_FINDINGS.md` §7 offers only **decline** rules and
/// says the accept side is not offered, because *"a mis-predicted accept is a
/// wrong obj"*. That warning is about a lane that would **perform** the inline.
/// Here the prediction drives a **refusal**: firing where c2 in fact kept the
/// call makes the port decline a function it would have got right — which costs
/// reach and cannot produce a wrong emit. The hazard is inverted, and this is
/// the one place the size question is safe to get wrong.
///
/// # Why it is not in the parser, and what the parser DOES ask
///
/// The parser cannot ask *this* question, because whether the port still emits
/// the call is decided *after* mechanism E (the callee reduces to nothing, so
/// the call is dropped) and mechanism I (the body is replaced by the callee's,
/// so the REL24 becomes the callee's own). A parser clause would fire on both of
/// those and un-ship them. Measured on the 878-TU workload: the coarse
/// parser-shaped form costs **1,074** byte-exact functions; this one costs
/// **0**. `work/w-inlfence2/crossing.md`.
///
/// What the parser asks instead — since W-FENCE2 — is whether the *facts this
/// seam needs* are available at all: `IlBundle::functions` hands a TU on only
/// when every locally-defined callee has PLAIN EXTERNAL linkage (not `static`,
/// whose ceiling F1 puts three times higher; not `inline`/`__forceinline`, which
/// F4 measured bypassing every size test) and every segment is at `/O1` (the
/// mode the bound is measured at). The two halves are total together:
/// `Bindings::per_record` is 1:1 with the `.ex` segments, so every locally
/// defined callee is one of the TU's own functions and `PortC2::build` lowers
/// all of them — a callee the port cannot lower fails the whole TU before an obj
/// exists.
///
/// # What it deliberately does NOT refuse
///
/// A callee this TU defines that the port **cannot lower**. 1,081 byte-exact
/// functions call one, and every one of them is a callee of 65–308 emitted
/// bytes, which is the class c2 **keeps the call to** (`WB_INLINE_FINDINGS` F1;
/// GRID-W re-measures it as 955 kept and 0 inlined above 80 B). Refusing those
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
                 <= {} bytes, so the port cannot prove c2 kept this call. See \
                 c2_core::comdat::fenced_inlined_callee.",
                call.callee, INLINE_DECLINE_BYTES,
            ))));
        }
    }
    Ok(())
}

/// Is `name` a callee at a site in this TU whose call the port **cannot prove c2
/// kept**?
///
/// Every `false` is either a positive decline proof (varargs, direct recursion,
/// a lowered body over [`INLINE_DECLINE_BYTES`]) or a "not established" — an
/// external, an ambiguous name, an unreadable mode, a callee the port cannot
/// lower. The second kind is left exactly where it was, and on the emit path it
/// is unreachable: `IlBundle::functions` hands on only TUs whose every locally
/// defined callee is one of the TU's own segments, and `PortC2::build` lowers
/// every one of them.
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
    // **W-XTEA3 — F9's licensed decline rule, and the FIRST clause here that is
    // a size rule keyed on something other than the size alone.**
    //
    // `docs/whitebox/WB_INLINE_FINDINGS.md` §7's MAY table lists it verbatim:
    // *"a **loop-bodied** callee **> 80 bytes** ⇒ never inlined at `/O1`"*, F9
    // + the anchor, **62 cells**, with the port-side use stated as *"the safe
    // decline side: the port may keep the call"*. GRID-J measures the loop
    // family's boundary at `(56,80]` against the straight-line family's
    // `(96,120]`, **identically at the workload flags and at `/O1 /GS- /c`**,
    // which is what refutes `R7-FLAGS`.
    //
    // The mechanism §5 names is why the two families differ: the ceiling is
    // applied to a pre-codegen tuple COUNT and emitted bytes are a proxy that
    // over-credits a loop by roughly 1.55, because the induction variables, the
    // compare and the branch collapse into one `bdnz`.
    //
    // **This clause widens acceptance, which is the dangerous direction**, and
    // it is bounded three ways: it is one of the five rows the whitebox lane
    // licensed for exactly this use; it fires only above a bound three times
    // tighter than the one it replaces (80 against 128), so the mixed band
    // GRID-W measures at 64–95 is untouched on the straight-line side and only
    // its top 15 bytes are entered on the loop side; and it is graded by the
    // oracle at 346 fixtures × two modes and 878 workload TUs, where a
    // mis-prediction is a `mismatch` and not a silence.
    //
    // The target is `?Encipher@XTEABlockEncrypter` at **116** emitted bytes with
    // a `bdnz` — 36 bytes above the top of the loop bracket — and
    // `EncryptXTEA.obj`'s own `?Encrypt` carries the `bl` against it, so the
    // oracle agrees on this witness.
    if g.body_has_loop() && gb.text.len() > INLINE_DECLINE_LOOP_BYTES {
        return false;
    }
    !gb.text.is_empty() && gb.text.len() <= INLINE_DECLINE_BYTES
}

/// **The emitted-body size above which a LOOP-BODIED callee is measured never
/// to be inlined at `/O1`.** `WB_INLINE_FINDINGS` F9, GRID-J's `(56,80]`
/// bracket over 56 cells plus the anchor's `(60,84]`, so **80** is the top of
/// the measured bracket and the rule is stated `> 80`.
///
/// Three times tighter than [`INLINE_DECLINE_BYTES`], which is the whole point:
/// a loop body priced by its emitted size is over-credited relative to the
/// pre-codegen count c2 actually tests.
pub const INLINE_DECLINE_LOOP_BYTES: usize = 80;

/// **W-FENCE2 — the emitted-body size above which c2 is measured never to inline
/// an EXTERNAL callee at `/O1`, with margin.** A call to a locally-defined
/// callee whose lowered body exceeds this is one the port may keep; anything at
/// or below it is refused, whether or not c2 would in fact have kept it.
///
/// # Measured, on the decline side only
///
/// `work/w-fence2/GRID-W.md` — for every IL call edge to a callee its own TU
/// defines, over the whole 878-TU dc3 workload, whether the **reference** obj's
/// caller carries a `REL24` naming the callee. 1,101 kept, 6,451 inlined, 0
/// unknown, banded by the callee's own reference `.text` size:
///
/// ```text
///    0- 63 B      0 kept   5,881 inlined
///   64- 79 B      9 kept     503 inlined     <-- MIXED
///   80- 95 B    137 kept      67 inlined     <-- MIXED, and the last inline
///   96+   B    955 kept       0 inlined
/// ```
///
/// **The largest callee c2 is measured to inline anywhere on the workload is 80
/// bytes.** The boundary is `(80, 96]`, on 7,552 sites — 23× the 320 cells
/// `WB_INLINE_FINDINGS` compiled, and tighter than every bracket it must live
/// under: F2's `(100,116]` EXTERNAL at `/O1`, GRID-J's `(96,120]`.
///
/// # Why 128 and not 96
///
/// 128 is **48 bytes / 12 words above the largest measured inline**, and it is
/// *above* both published `/O1` first-declined points (116 and 120) rather than
/// fitted between them. It is not fitted to the TU this bound was raised for
/// either: `vsnprnc.cpp`'s `_vsprintf_s_l` is 152 B, and any value in `(95,152]`
/// produces the identical result on this workload — GRID-W's port-side table has
/// no site at all between 47 B and 152 B.
///
/// # What it is NOT
///
/// **Not `splice::INLINE_UNBOUNDED_BYTES`.** That one is the *accept* region —
/// `index <= 64` is where `INLINE_PREDICATE.md` §2's `N_max` is unbounded in
/// both linkage classes — and it licenses the port to *perform* an expansion.
/// This one licenses nothing; it only decides where a refusal stops. They must
/// not be merged: raising the splice bound would make the port inline bodies c2
/// keeps, which is the wrong-obj direction.
///
/// **Not linkage-agnostic, and not mode-agnostic.** F1 puts the STATIC ceiling
/// at `(300,308]` and F1/F2 put the favour-speed ceilings at `(212,252]` and
/// `(156,164]` — all above this. The parser refuses a TU whose locally-defined
/// callee is `static`, `inline`/`__forceinline` or at any mode but `/O1`, so no
/// obj reaches this constant outside the class it was measured on.
pub const INLINE_DECLINE_BYTES: usize = 128;

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
    // **W-BIQUAD** — filled by the one arm that pools constants under `/Gy`.
    let mut fp_refs: Vec<crate::codegen::FpConstRef> = Vec::new();
    let (text, calls) = match selected {
        // A framed non-leaf call gets its own `.text` COMDAT like any other
        // function, plus a `.pdata` COMDAT associated to it (W-UNW-1).
        // `Selected::Framed` carries no bytes for the same reason
        // `Selected::Tail` carries an incomplete text: the branch word encodes
        // its own `.text` offset, so only the caller — which knows where the
        // function lands — can finish it. Under `/Gy` that offset is 0, because
        // each function starts its own section.
        // **W-BIQUAD — the forwarding constructor**, and the ONE place in this
        // crate where a body's words depend on a fact about a DIFFERENT
        // function.
        //
        // M-RULE (`WB_CHOOSER_FINDINGS` §2.3) puts a value live across a call in
        // a register the callee does not write, and for a same-TU callee it uses
        // that callee's EXACT footprint. `codegen::ctor_forward_call` therefore
        // emits `mr r10,r3` with no restore — nine words that are right only if
        // the callee writes neither r10 nor r3. Nothing in this constructor's
        // own IL says that, so it is asked here, where `tu` carries the callee's
        // definition and this crate carries its lowering.
        //
        // **The admitted set is one class**, and that is the honest size of the
        // knowledge: `fp_store_diamond::GPR_FOOTPRINT` is a statement one
        // function away from the words that make it true. Every other callee —
        // an external, an ambiguous name, a class whose footprint nobody has
        // stated — DECLINES, which is a named `codegen-gap` rather than a wrong
        // obj. `w-blockir` #2305's lesson in advance: the alternative to
        // refusing is guessing a register, and eight of the nine words would
        // still be right.
        codegen::Selected::CtorForwardCall => {
            let c = f
                .ctor_forward_call
                .as_ref()
                .expect("CtorForwardCall implies ctor_forward_call");
            let footprint = tu
                .definition(&c.callee)
                .and_then(|(g, _)| {
                    g.fp_store_diamond
                        .as_ref()
                        .map(|_| codegen::fp_store_diamond::GPR_FOOTPRINT)
                })
                .ok_or_else(|| {
                    ComdatDecline::Shape(BackendError::NotImplemented(format!(
                        "a forwarding constructor whose callee `{}` has no STATED                          GPR footprint in this port: M-RULE picks the park register                          out of the callee's exact register set for a same-TU                          callee and out of the whole volatile set otherwise, and                          the two differ in the park register, in the presence of a                          `std`/`ld` pair and in whether a restore is emitted. See                          c2_core::codegen::ctor_forward_call.",
                        c.callee,
                    )))
                })?;
            if footprint.contains(&PARK_GPR) || footprint.contains(&3) {
                return Err(ComdatDecline::Shape(BackendError::NotImplemented(
                    "a forwarding constructor whose callee writes the park                      register or r3: the volatile park and the absent restore                      are both statements that it writes neither"
                        .to_string(),
                )));
            }
            let body = codegen::ctor_forward_call::ctor_forward_call_text(0)
                .map_err(ComdatDecline::Shape)?;
            frame = Some(coff::Frame {
                prolog_len: body.prolog_len,
                func_len: body.text.len() as u32,
            });
            (
                body.text,
                vec![coff::Call {
                    reloc_offset: body.bl_offset,
                    callee: c.callee.as_str(),
                }],
            )
        }
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
        // **W-XTEA3 — the framed XTEA block loop.** THREE REL24 sites for ONE
        // IL-named callee: the frame's `__savegprlr_26`/`__restgprlr_26` pair is
        // minted here from the layout, never read out of the IL, and its two
        // symbols go on `helper_externals` so the writer places them after the
        // `$T` label rather than in the callee region. The third site is
        // `?Encipher`, which is DEFINED in this same obj — so it is an IL-named
        // callee whose symbol the writer already has.
        codegen::Selected::XteaEncryptLoop => {
            let g = f
                .xtea_encrypt_loop
                .as_ref()
                .expect("XteaEncryptLoop implies xtea_encrypt_loop");
            let body = codegen::xtea_encrypt_loop::xtea_encrypt_loop_text(g, 0, mode)
                .map_err(ComdatDecline::Shape)?;
            frame = Some(coff::Frame {
                prolog_len: body.prolog_len,
                func_len: body.text.len() as u32,
            });
            let fr = codegen::xtea_encrypt_loop::xtea_frame();
            let (Some(save), Some(rest)) =
                (fr.save_gpr_helper_name(), fr.rest_gpr_helper_name())
            else {
                return Err(ComdatDecline::Shape(crate::BackendError::NotImplemented(
                    "xtea-encrypt-loop: no `__savegprlr_N` name for this layout".to_string(),
                )));
            };
            // Reverse first-reference over the two helper sites, the same rule
            // `introduced_externals` applies: the save is the prologue's word
            // and the restore is the function's last, so the restore's symbol is
            // the earlier record.
            helper_externals = vec![rest, save];
            let calls = vec![
                coff::Call { reloc_offset: body.bl_offsets[0], callee: save },
                coff::Call { reloc_offset: body.bl_offsets[1], callee: g.callee.as_str() },
                coff::Call { reloc_offset: body.bl_offsets[2], callee: rest },
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
        // **W-XTEA2 — the whole-body `memcpy` tail branch.** The branch is the
        // ordinary tail call's, word for word; what differs is that the callee is
        // MINTED here from the emitter's constant, because the copy arrives as an
        // intrinsic selector with no `.gl` record and `f.tail_call` is `None`.
        //
        // **`helper_externals` stays EMPTY, which is the one thing this class
        // does not share with `GuardRetChain` above.** That class's user is
        // framed and `w-ifn` measured its `memcpy` landing after the `$T` label;
        // this user is a LEAF and has no `$T`. Both of this lane's obj readings
        // put the name in the CALLEE REGION instead — `work/w-xtea2/ref/xtea.dump`
        // has `[16] ?SetKey · [17] memcpy · [18] .text`, and
        // `work/w-xtea2/probe/mcpytail.obj` the same one function over — so the
        // name goes on `calls` alone and `introduced_externals` places it.
        codegen::Selected::MemcpyTail(mut t) => {
            let branch_off = t.len() as u32;
            t.extend_from_slice(&codegen::encode_tail_branch(branch_off));
            (
                t,
                vec![coff::Call {
                    reloc_offset: branch_off,
                    callee: codegen::memcpy_tail::MEMCPY_NAME,
                }],
            )
        }
        codegen::Selected::Float { text, .. } => (text, Vec::new()),
        // **W-BIQUAD — the float-store diamond.** A leaf with two branches: no
        // frame, no `.pdata`, no REL24. Its two pools travel on `fp_refs`, and
        // the emitter has already placed both halves of each — `lo_off` is NOT
        // `hi_off + 4` here (B-RULE puts one `lis` five words above its `lfs`).
        codegen::Selected::FpStoreDiamond { text, consts } => {
            fp_refs = consts;
            (text, Vec::new())
        }
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
        fp_refs,
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
    /// **W-BIQUAD — a pooled FP constant's `.rdata` symbol.** Carried as the
    /// `(bit pattern, width)` key rather than as a name because the name is a
    /// *rendering* of exactly that key
    /// ([`coff::real_symbol_name`]) and a plan holding both could disagree with
    /// itself. Consumers that need the spelling call the renderer; the writer
    /// resolves the key against its own pool table.
    FpPool { bits: u64, double: bool },
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
    fp_refs: &[crate::codegen::FpConstRef],
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
    // **W-BIQUAD — the same quad shape a third time**, against a `.rdata`
    // COMDAT this obj also defines. `lo_off` is a field and not `hi_off + 4`:
    // B-RULE puts a block-local `lis` at the top of its block and its `lfs` at
    // the use, five words apart in `?SetCoefficients`.
    for r in fp_refs {
        recs.push(TextReloc {
            va: r.hi_off,
            ty: coff::REL_PPC_REFHI,
            target: PlanTarget::FpPool { bits: r.bits, double: r.double },
        });
        recs.push(TextReloc {
            va: r.hi_off,
            ty: coff::REL_PPC_PAIR,
            target: PlanTarget::PairDisplacement(0),
        });
        recs.push(TextReloc {
            va: r.lo_off,
            ty: coff::REL_PPC_REFLO,
            target: PlanTarget::FpPool { bits: r.bits, double: r.double },
        });
        recs.push(TextReloc {
            va: r.lo_off,
            ty: coff::REL_PPC_PAIR,
            target: PlanTarget::PairDisplacement(0),
        });
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
    /// A leaf whose lowered body is over [`INLINE_DECLINE_BYTES`].
    ///
    /// **The rung count was 20 and is 40 — w-fence2, 2026-08-09.** The bound
    /// this cell has to clear moved from `splice::INLINE_UNBOUNDED_BYTES` (64)
    /// to `INLINE_DECLINE_BYTES` (128) when the fence stopped meaning *"the port
    /// can prove c2 expands this"* and started meaning *"the port cannot prove
    /// c2 kept this"*. The cell's own guard caught it — it asserts its callee is
    /// outside the bound before grading anything, which is why this was a test
    /// failure and not a silent confound.
    fn big_leaf(name: &str) -> IlFunction {
        let mut ops = vec![IlOp::Load(0xE309)];
        for _ in 0..40 {
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
            g.text.len() <= INLINE_DECLINE_BYTES,
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
            g.text.len() > INLINE_DECLINE_BYTES,
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
            g.text.len() <= INLINE_DECLINE_BYTES,
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
