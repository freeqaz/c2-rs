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

    /// The `$M…`/`$T…` label counter seed for this TU.
    ///
    /// Returns 0 — an unused value — when no function in the TU is framed,
    /// because then no label is emitted and `coff::plan_labels` yields `None`
    /// everywhere.
    ///
    /// **The acceptance question is not asked here.** The counter is consumed by
    /// *every* function in the TU, 1 for each class this port emits but 3 for a
    /// comparison leaf and 2 for a floating-point one, so a framed function
    /// sharing a TU with either would be mis-numbered — and that gate lives in
    /// `c2_il::IlBundle::functions`, with the TU-level gates, so the census and
    /// the emitter cannot disagree about it (roadmap #44). Same for the seed's
    /// readability. By the time `build` runs, `functions()` has established both.
    ///
    /// The `None` arm is therefore unreachable and still refuses rather than
    /// defaulting: a guessed `$M` number is a wrong-bytes obj that links, and a
    /// two-valued answer to "did I find the counter?" is how three of this
    /// project's mis-emits happened (`docs/GAPS.md` §6).
    fn frame_label_counter(il: &IlBundle, funcs: &[c2_il::IlFunction]) -> Result<u32, BackendError> {
        if !funcs.iter().any(|f| f.is_framed()) {
            return Ok(0);
        }
        il.label_counter().ok_or_else(|| {
            BackendError::NotImplemented(
                "framed function but no readable `.gl` label counter (the u32 at \
                 .gl offset 7, behind the `11 02 06 '1j2' 01` header): the $M/$T \
                 label numbers are seeded from it and must never be guessed"
                    .to_string(),
            )
        })
    }

    /// The order the TU's functions are emitted in — **not** the `.ex` order.
    ///
    /// The rule and its measurements live on [`coff::plan_text_order`]; this
    /// assembles the reference set it consumes, which is the half that needs the
    /// IL:
    ///
    /// * [`c2_il::IlFunction::callees`] — every name the body branches to;
    /// * [`c2_il::IlFunction::eh_unwind_callees`] — the `26` unwind action's
    ///   base destructor, which **emits nothing**: no `bl`, no relocation, no
    ///   symbol. Its own doc comment says it "contributes nothing to `callees`,
    ///   which is what the emitter reads", and that was true of the *emitter*
    ///   and false of the *scheduler*. `struct B{B();~B();int x;}; struct D:B
    ///   {D();}; D::D(){} B::~B(){}` is byte-exact only if this half is included
    ///   — with `callees()` alone the port emits `??0D` first and the obj has
    ///   `??1B` first.
    ///
    /// A data symbol ([`c2_il::IlFunction::data_sym`]) is deliberately **not** a
    /// reference here: it names a `.data`/`.bss` object, never a function, and
    /// the TU-level gate already refuses a bundle that defines one.
    fn plan_emit_order(funcs: &[c2_il::IlFunction]) -> Result<Vec<usize>, BackendError> {
        let names: Vec<&str> = funcs.iter().map(|f| f.mangled_name.as_str()).collect();
        let refs: Vec<Vec<&str>> = funcs
            .iter()
            .map(|f| {
                f.callees()
                    .chain(f.eh_unwind_callees.iter().map(|s| s.as_str()))
                    .collect()
            })
            .collect();
        coff::plan_text_order(&names, &refs).ok_or_else(|| {
            BackendError::NotImplemented(
                "the TU's functions reference each other in a CYCLE (mutual \
                 recursion): c2 folds the recursion and its emission order is \
                 not the dependency order — three probes in `work/w-order/p/g*.cpp` \
                 disagree with every single tie-break rule, so this refuses \
                 rather than picks one"
                    .to_string(),
            )
        })
    }

    /// This function's leading label-counter surcharge — [`c2_il::IlFunction::label_lead`]
    /// **less the `/EHsc` eh-bare slot when the unwind target is defined here
    /// AND its body is empty**.
    ///
    /// `IlFunction::eh_bare`'s table charges an empty base-delegating constructor
    /// `+1` at `/EHsc`. Every probe behind that table was a single-purpose TU in
    /// which the base destructor was an *undefined* external, so the table holds
    /// the axis this corrects fixed. When the `26` unwind action's target is
    /// **defined in this TU with a bare `blr` body**, c2 has nothing to run on
    /// the unwind path and the slot is not charged.
    ///
    /// Measured seed-free and in-TU against `int a0(int a){return ga(a)+1;}` as
    /// the leading anchor (stride 4 packed), reading `first(P) - first(a0)`.
    /// `work/w-order/p/h*.cpp`; `D` derives from `B`, so `??0D`'s unwind action
    /// names `??1B`:
    ///
    /// ```text
    ///   probe                                no /EH  /EHsc  lead   ??1B is
    ///   h0  a0 ; D::D                            4      5     1    not defined here
    ///   h1  a0 ; C::~C{} ; D::D                  5      6     1    "  (C is unrelated)
    ///   h2  a0 ; C::~C{} ; E::~E{} ; D::D        6      7     1    "
    ///   h3  a0 ; z(int) ; D::D                   5      6     1    "
    ///   h5  a0 ; B::~B{} ; D::D                  5      5     0    defined, EMPTY
    ///   h6  a0 ; C::~C{} ; B::~B{} ; D::D        6      6     0    defined, EMPTY
    ///   h8  a0 ; B::~B{} ; D::D ; F::F         5,4    5,4   0,0    defined, EMPTY
    ///   hf  a0 ; B::~B{gh();} ; D::D             5      6     1    defined, NOT empty
    ///   hg  a0 ; M::~M{} ; D::D  (D:M:Bd)        5      7     1    defined, DELEGATES
    /// ```
    ///
    /// **`hf` and `hg` are why the predicate is not just "defined here".** Both
    /// define the target and both still pay: `hf`'s body is a tail call, `hg`'s
    /// is the delegating `b ??1Bd`. Only a body that emits a bare `blr`
    /// suppresses it. Reading the first table alone gives a rule that fits
    /// `h5`/`h6`/`h8`/`h9` and turns five byte-exact objs into mismatches — it
    /// was written that way first and the `/EHsc` gate lanes caught it.
    ///
    /// **It is per FUNCTION, and `h9` separates that inside one obj**: with
    /// `??1B` defined-empty and `??1G` not defined, `D::D` (base `B`) takes lead
    /// **0** and `H::H` (base `G`) takes lead **1**, five slots apart in the same
    /// symbol table. `ha` swaps their source order and both leads follow their
    /// own class, not their position. A per-TU reading fits `h5`/`h6`/`h8`
    /// exactly as well and is wrong.
    ///
    /// **A destructor's own delegation target does not do this.** `hd`: `M::~M`
    /// (eh-bare, delegating to an undefined `??1Bd`) beside an unrelated
    /// locally-defined `Q::~Q` keeps its stride of 2. Only the constructor's
    /// unwind action is involved, which is why this reads `eh_unwind_callees`
    /// and not [`c2_il::IlFunction::callees`]. (`hc` — a destructor whose
    /// delegation target is itself defined here — is a different shape entirely:
    /// c2 inlines it, and the port already refuses.)
    ///
    /// A **mixed** unwind list — some targets defined-and-empty and some not —
    /// refuses: every witness has exactly one, and splitting the slot between
    /// them is not a measurement anyone has.
    fn label_lead_of(
        f: &c2_il::IlFunction,
        funcs: &[c2_il::IlFunction],
    ) -> Result<u32, BackendError> {
        let lead = f.label_lead();
        if !f.eh_bare || f.eh_unwind_callees.is_empty() {
            return Ok(lead);
        }
        let local_empty = |n: &String| {
            funcs
                .iter()
                .any(|g| &g.mangled_name == n && g.empty_body)
        };
        let here = f.eh_unwind_callees.iter().filter(|n| local_empty(n)).count();
        if here == 0 {
            return Ok(lead);
        }
        if here < f.eh_unwind_callees.len() {
            return Err(BackendError::NotImplemented(
                "an eh-bare constructor whose unwind actions name BOTH a \
                 locally-defined empty destructor and one that is not: the \
                 `/EHsc` label slot is charged for the second kind and not for \
                 the first, and no probe measures a body that is both"
                    .to_string(),
            ));
        }
        Ok(lead - 1)
    }

    /// Build the obj for `il`, embedding `obj_name` as S_OBJNAME. Handles one
    /// or more straight-line int add-chain functions in a single TU (each is
    /// selected + placed in a shared `.text`; see [`codegen::select_text`] and
    /// [`coff::emit_obj`]).
    pub fn build(&self, il: &IlBundle, obj_name: &str) -> Result<ObjImage, BackendError> {
        // **W-R1c — the `??__E` dynamic-initializer TU (board #158).**
        //
        // Tried before `functions()` because it is a *whole-TU* shape, not a
        // function class: the TU's only function is a compiler-emitted thunk
        // whose three arguments are two relocated addresses and a literal, and
        // `functions()` correctly refuses it (its data symbol is defined here,
        // which `Bindings::resolve_data` will not resolve and must not start
        // resolving — see `IlBundle::dyninit_tu`).
        //
        // Every gate lives in the recognizer, and this arm adds exactly one more
        // that cannot live there because `string_comdat_name` is in this crate:
        // **the computed `??_C@…` name must be one `.gl` actually spells.**
        if let Some(tu) = il.dyninit_tu() {
            if let Some(obj) = Self::build_dyninit(&tu, obj_name) {
                return Ok(ObjImage::new(obj));
            }
            return Err(BackendError::NotImplemented(
                "a `??__E` dynamic-initializer TU outside the measured class: \
                 the literal's COMDAT name, the object's alignment, or the \
                 thunk's schedule is one this port has not been graded on"
                    .to_string(),
            ));
        }

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
            // One bit of the word is NOT a mode: `0x0100` says the function is a
            // constructor or a destructor ([`c2_il::OPT_WORD_SPECIAL_MEMBER`],
            // measured one flag and one function kind at a time). It is masked off
            // before the whole-word compare, so a destructor's word reads as the
            // mode it actually is — otherwise every constructor and destructor in
            // the corpus is a `codegen-gap` however ordinary its body, which is
            // what kept `A::~A() {}` (a bare `blr`, decoded as `EmptyBody`) out of
            // the emitter. Every other bit is still required to match a word this
            // port was verified against.
            let m = codegen::opt_mode_of_word(*w)
                .map_err(|e| BackendError::NotImplemented(format!("{e} at function {i}")))?;
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

        // W-UNW-1: any framed function in the TU makes the obj carry `.pdata`
        // unwind records and the `$M…`/`$T…` compiler labels, whose numbers come
        // from a counter seeded in `.gl` and advanced once per function. Both
        // emitters model that now (it used to be a third emitter hardcoded to one
        // fixture), but the counter only advances by the measured stride for the
        // function classes it was measured over — so a framed TU is admitted only
        // when every function in it is one of those.
        let label_counter = match Self::frame_label_counter(il, &funcs) {
            Ok(c) => c,
            Err(e) => return Err(e),
        };

        // **The emission ORDER is not the `.ex` order** (board row X-d). c2
        // emits a function only once every function it references *and defines*
        // has been emitted; the port used to emit `.ex` order flat, which is a
        // silent wrong-bytes obj whenever the source defines a callee after its
        // caller. `coff::plan_text_order` carries the rule, the measurements and
        // the refusal; this is where the reference set is assembled, because it
        // is the only place that sees the IL.
        //
        // The set is `callees()` **plus `eh_unwind_callees`**, and the second
        // half is the one that cannot be recovered from the obj: a constructor's
        // `26` unwind action names the base destructor and emits no `bl`, no
        // relocation and no symbol for it, yet it orders the two functions.
        let order = Self::plan_emit_order(&funcs)?;

        // **The `/EHsc` eh-bare surcharge is not paid when the unwind target is
        // DEFINED here** — the second wrong-bytes emit this lane found, and it is
        // independent of the order above (it fires on `c7_dtor_first_src.cpp`,
        // whose emission order is already right). See [`Self::label_lead_of`].
        let leads = funcs
            .iter()
            .map(|f| Self::label_lead_of(f, &funcs))
            .collect::<Result<Vec<u32>, BackendError>>()?;

        // Under function-level linking every function gets its own COMDAT
        // `.text` section, so the texts are kept separate rather than packed.
        // The order rule is the same one — measured at `/O1` too, where it
        // decides the section table itself and not just offsets within `.text`.
        if self.fn_level_linking {
            let mut texts: Vec<Vec<u8>> = Vec::with_capacity(funcs.len());
            let mut placed: Vec<coff::Function> = Vec::with_capacity(funcs.len());
            for &fi in &order {
                let f = &funcs[fi];
                let mut frame: Option<coff::Frame> = None;
                let (text, calls) = match codegen::select_function(f, mode)? {
                    // A framed non-leaf call gets its own `.text` COMDAT like
                    // any other function, plus a `.pdata` COMDAT associated to
                    // it (W-UNW-1). `Selected::Framed` carries no bytes for the
                    // same reason `Selected::Tail` carries an incomplete text:
                    // the branch word encodes its own `.text` offset, so only
                    // the caller — which knows where the function lands — can
                    // finish it. Under `/Gy` that offset is 0, because each
                    // function starts its own section.
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
                    // A Class A many-call body: the same frame and `.pdata`, with
                    // one REL24 site per call instead of one per function.
                    codegen::Selected::Seq { setups, tail } => {
                        let seq = f.call_seq.as_ref().expect("Seq implies call_seq");
                        // **W10** — the guard, when there is one. Resolved
                        // through `seq_guard_emit` on both emission paths, so
                        // the packed and COMDAT writers cannot disagree about a
                        // branch sense.
                        let guard = seq
                            .guard
                            .as_ref()
                            .map(codegen::seq_guard_emit)
                            .transpose()?;
                        // **W11** — the guarded early returns, resolved through
                        // the same `seq_early_emit` on both emission paths for
                        // the same reason: the packed and COMDAT writers must
                        // not disagree about a branch sense or a block layout.
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
                    // **W8 — a two-arm conditional tail call.** Two REL24 sites,
                    // one per arm, in block order; the conditional branch
                    // between them carries its own displacement and NO
                    // relocation (`docs/CFG_SHAPE.md` §3.3). Under `/Gy` the
                    // function starts at offset 0 of its own COMDAT, so each
                    // tail branch's word is `-(its offset within this text)`.
                    codegen::Selected::CondPair(parts) => {
                        let cp = f.cond_pair.as_ref().expect("CondPair implies cond_pair");
                        let mut t = parts.text;
                        let mut calls = Vec::with_capacity(2);
                        for (off, callee) in parts.branch_offsets.iter().zip([
                            cp.then_arm.callee.as_str(),
                            cp.else_arm.callee.as_str(),
                        ]) {
                            let w = codegen::encode_tail_branch(*off);
                            t[*off as usize..*off as usize + 4].copy_from_slice(&w);
                            calls.push(coff::Call { reloc_offset: *off, callee });
                        }
                        (t, calls)
                    }
                    // Each function's text starts at offset 0 of its own COMDAT
                    // section, so the branch offset is just the setup's length.
                    codegen::Selected::Tail(mut t) => {
                        let branch_off = t.len() as u32;
                        t.extend_from_slice(&codegen::encode_tail_branch(branch_off));
                        let callee = f.tail_call.as_deref().expect("Tail implies tail_call");
                        (t, vec![coff::Call { reloc_offset: branch_off, callee }])
                    }
                    codegen::Selected::Float { text, .. } => (text, Vec::new()),
                    codegen::Selected::Plain(t) => (t, Vec::new()),
                };
                // Under `/Gy` each function starts at offset 0 of its own COMDAT.
                let data_refs = data_refs_of(f, &text, 0)?;
                placed.push(coff::Function { name: &f.mangled_name, text_offset: 0, calls, is_float: f.touches_floating_point(), fp_refs: Vec::new(), data_refs, frame, label_lead: leads[fi] });
                texts.push(text);
            }
            return Ok(ObjImage::new(coff::emit_comdat_obj(
                obj_name,
                &placed,
                &texts,
                label_counter,
            )));
        }

        // Select each function's .text, recording each function's byte offset.
        // Functions start at an **8-byte-aligned** offset within .text (the
        // section is ALIGN_8): c2 zero-pads between functions to the next
        // 8-byte boundary, but does NOT pad the tail of .text. The first
        // function is at 0 (already aligned). Verified: mvp_sub's three 12-byte
        // functions land at 0x0 / 0x10 / 0x20 with 4 zero bytes between.
        let mut text: Vec<u8> = Vec::new();
        let mut placed: Vec<coff::Function> = Vec::with_capacity(funcs.len());
        for &fi in &order {
            let f = &funcs[fi];
            while text.len() % 8 != 0 {
                text.push(0);
            }
            let off = text.len() as u32;
            let mut frame: Option<coff::Frame> = None;
            let (calls, fp_refs) = match codegen::select_function(f, mode)? {
                // A framed non-leaf call: the fixed 0x24-byte frame, plus a
                // `.pdata` record and two `$M` labels (W-UNW-1). Packed, the
                // `bl` displacement is `-(its own .text offset)`, so the body
                // has to be built at `off` — the same reason `Selected::Tail`
                // hands back an unfinished text. Emitting it at a hardcoded 0
                // was a live wrong-bytes emit for any framed function that is
                // not first in the section.
                codegen::Selected::Framed { setup } => {
                    let fc = f.framed_call.as_ref().expect("Framed implies framed_call");
                    let body = codegen::framed_call_text(
                        &setup,
                        fc.add_k,
                        off,
                        codegen::FrameLayout::default(),
                    )?;
                    frame = Some(coff::Frame {
                        prolog_len: body.prolog_len,
                        func_len: body.text.len() as u32,
                    });
                    text.extend_from_slice(&body.text);
                    (
                        vec![coff::Call {
                            reloc_offset: body.bl_offset,
                            callee: &fc.callee,
                        }],
                        Vec::new(),
                    )
                }
                // A Class A many-call body, built at `off` for the same reason:
                // every `bl` word encodes its own `.text` offset.
                codegen::Selected::Seq { setups, tail } => {
                    let seq = f.call_seq.as_ref().expect("Seq implies call_seq");
                    // **W10** — same resolver as the `/Gy` path above. The
                    // conditional branch and the intra-section `b` are both
                    // self-relative, so unlike every `bl` beside them they are
                    // independent of where the function lands.
                    let guard = seq
                        .guard
                        .as_ref()
                        .map(codegen::seq_guard_emit)
                        .transpose()?;
                    // **W11** — same resolver as the `/Gy` path above. The
                    // guards' `bc` and the arms' intra-section `b` are both
                    // self-relative, so unlike every `bl` beside them they are
                    // independent of where the function lands.
                    let early = seq
                        .early
                        .iter()
                        .map(codegen::seq_early_emit)
                        .collect::<Result<Vec<_>, _>>()?;
                    let body = codegen::call_seq_text(
                        &setups,
                        &tail,
                        off,
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
                    text.extend_from_slice(&body.text);
                    (
                        body.bl_offsets
                            .iter()
                            .zip(&seq.calls)
                            .map(|(o, c)| coff::Call {
                                reloc_offset: *o,
                                callee: c.callee.as_str(),
                            })
                            .collect(),
                        Vec::new(),
                    )
                }
                // **W8 — a two-arm conditional tail call**, packed: each `b`
                // encodes its own whole-`.text` offset, so the two words are
                // rebased onto `off` exactly as the single tail call's is. The
                // `bc` between them is untouched — its displacement is
                // self-relative and therefore independent of where the function
                // lands, which is the whole of `docs/CFG_SHAPE.md` §3.3's
                // "two encodings, one opcode".
                codegen::Selected::CondPair(parts) => {
                    let cp = f.cond_pair.as_ref().expect("CondPair implies cond_pair");
                    let mut body = parts.text;
                    let mut calls = Vec::with_capacity(2);
                    for (rel_off, callee) in parts.branch_offsets.iter().zip([
                        cp.then_arm.callee.as_str(),
                        cp.else_arm.callee.as_str(),
                    ]) {
                        let abs = off + rel_off;
                        let w = codegen::encode_tail_branch(abs);
                        body[*rel_off as usize..*rel_off as usize + 4].copy_from_slice(&w);
                        calls.push(coff::Call { reloc_offset: abs, callee });
                    }
                    text.extend_from_slice(&body);
                    (calls, Vec::new())
                }
                // Tail call. A void bare call (an empty setup) is a single
                // `b <callee>` (REL24) at this offset; an integer or multi-argument
                // tail call first puts the arguments in place, then branches (the
                // branch, not the function start, is the reloc site).
                codegen::Selected::Tail(setup) => {
                    let branch_off = off + setup.len() as u32;
                    text.extend_from_slice(&setup);
                    text.extend_from_slice(&codegen::encode_tail_branch(branch_off));
                    let callee = f.tail_call.as_ref().expect("Tail implies tail_call");
                    (
                        vec![coff::Call {
                            reloc_offset: branch_off,
                            callee,
                        }],
                        Vec::new(),
                    )
                }
                // W13a/W13b: an FP leaf has its own register model entirely (pool
                // [f0, f13..f1], result f1, no accumulator collapse). Each pooled
                // constant's reference site is rebased onto the whole `.text`.
                codegen::Selected::Float { text: body, consts } => {
                    text.extend_from_slice(&body);
                    (
                        Vec::new(),
                        consts
                            .into_iter()
                            .map(|r| codegen::FpConstRef {
                                hi_off: r.hi_off + off,
                                ..r
                            })
                            .collect(),
                    )
                }
                codegen::Selected::Plain(body) => {
                    text.extend_from_slice(&body);
                    (Vec::new(), Vec::new())
                }
            };
            let data_refs = data_refs_of(f, &text[off as usize..], off)?;
            placed.push(coff::Function {
                name: &f.mangled_name,
                text_offset: off,
                calls,
                is_float: f.touches_floating_point(),
                fp_refs,
                data_refs,
                frame,
                label_lead: leads[fi],
            });
        }

        let bytes = coff::emit_obj(obj_name, &placed, &text, label_counter);
        Ok(ObjImage::new(bytes))
    }

    /// **W-R1c — the join between the decode and the obj shape.**
    ///
    /// `emit_dyninit_obj` and the `??__E` decode were both built by lane w-r1 and
    /// neither had a caller; this is it. Everything here is a translation of
    /// values the recognizer already read out of the IL, plus **one gate that
    /// has to live in this crate**: the `/GF` fence.
    ///
    /// `None` => the caller reports `NotImplemented`, which is the honest answer
    /// for a TU whose obj this port has not been graded on.
    fn build_dyninit(tu: &c2_il::DynInitTu, obj_name: &str) -> Option<Vec<u8>> {
        let literal = coff::StringLiteral { bytes: &tu.literal };
        // **The `/GF` fence** — the single most likely way this class ships wrong
        // bytes, and the reason it is a check and not a comment.
        //
        // `/GF` is implied by `/O1` and `/O2` but NOT by `/Ox`
        // (`docs/OBJ_DYNINIT_SHAPE.md` §4.3). Without it the literal is a
        // **non-COMDAT `$SG<n>` `.rdata` placed BEFORE `.text`**, with 5
        // relocations instead of 9 — a different obj that this emitter does not
        // build. MEASURED: `fixtures/cpp/il_dyninit_static.cpp` at `/Ox` still
        // carries `abc\0` in `.in` but carries **no `??_C@` record in `.gl`**, so
        // trusting `.in` alone would convert that fixture to the wrong shape.
        //
        // Requiring the computed name to be one `.gl` spells refuses it on
        // structure rather than on a flag test, which is what keeps the gate
        // honest when a future lane changes how flags reach the port.
        let name = coff::string_comdat_name(&tu.literal)?;
        if !tu.literal_comdat_names.contains(&name) {
            return None;
        }
        let body = codegen::dyninit_thunk_text(tu.trailing_literal_arg)?;
        let thunk = coff::DynInitThunk {
            name: &tu.thunk_name,
            text: &body.text,
            calls: vec![coff::Call {
                reloc_offset: body.branch,
                callee: &tu.ctor,
            }],
            data_refs: vec![
                coff::DataRef {
                    hi_off: body.literal_hi,
                    lo_off: body.literal_lo,
                    name: &name,
                },
                coff::DataRef {
                    hi_off: body.object_hi,
                    lo_off: body.object_lo,
                    name: &tu.object_symbol,
                },
            ],
        };
        let object = coff::BssObject {
            symbol: &tu.object_symbol,
            size: tu.object_size,
            natural_align: tu.object_align,
            external: tu.object_external,
            initializer_symbol: &tu.initializer_symbol,
        };
        coff::emit_dyninit_obj(obj_name, &thunk, Some(&literal), &object)
    }
}


/// **WR1 — the `.text` offset of a body's data-symbol address reference, checked
/// rather than assumed.**
///
/// `codegen::sym_slots_text` hoists `lis r11,sym@ha` to the **first word** of the
/// body, so the REFHI site is the function's own start and the REFLO site is four
/// bytes later. This re-derives that from the bytes instead of trusting it: a
/// future schedule that puts anything ahead of the `lis` would otherwise relocate
/// the wrong instruction, and every relocation would still resolve — the silent
/// wrong-bytes shape `docs/GAPS.md` §6 keeps recording.
///
/// `None` when the body carries no data symbol. `Err` when it carries one and the
/// first word is not the expected `lis`.
fn data_refs_of<'a>(
    f: &'a c2_il::IlFunction,
    text: &[u8],
    base: u32,
) -> Result<Vec<coff::DataRef<'a>>, BackendError> {
    let Some(name) = f.data_sym.as_deref() else {
        return Ok(Vec::new());
    };
    let lis = codegen::encode_addis(codegen::SCRATCH_REG, 0, 0);
    if text.len() < 8 || text[..4] != lis {
        return Err(BackendError::NotImplemented(
            "a data-symbol address whose `lis` is not this body's first word: the \
             relocation site is derived from that position"
                .to_string(),
        ));
    }
    // The low half: the unique `addi rD,r11,0` among the setup words. Derived by
    // search rather than by `hi_off + 4`, because the two halves are **not**
    // adjacent when a higher argument slot carries a literal — `gsp(&gI, 7)` puts
    // the `li r4,7` between them (`coff::DataRef`). It is unambiguous: the only
    // other instructions this class emits are the `lis` (an `addis`), `li rD,k`
    // (an `addi` whose RA is **0**, not 11) and the tail branch.
    let mut lo: Option<u32> = None;
    for (i, w) in text.chunks_exact(4).enumerate().skip(1) {
        if codegen::ARG_REGS
            .iter()
            .any(|&d| w == codegen::encode_addi(d, codegen::SCRATCH_REG, 0))
        {
            if lo.is_some() {
                return Err(BackendError::NotImplemented(
                    "two low-half `addi`s against the address scratch in one body"
                        .to_string(),
                ));
            }
            lo = Some(base + 4 * i as u32);
        }
    }
    let Some(lo_off) = lo else {
        return Err(BackendError::NotImplemented(
            "a data-symbol address with no `addi rD,r11,0` low half".to_string(),
        ));
    };
    Ok(vec![coff::DataRef { hi_off: base, lo_off, name }])
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

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // #137 — the PORTABLE pin for the OTHER half of WR1's second ordering rule.
    //
    // `coff.rs` writes the REFLO record at `DataRef::lo_off`; this file is what
    // *computes* `lo_off`, and computing it as `hi_off + 4` is the wrong-bytes
    // emit WR1 recorded. Both halves need a pin: with the derivation forced to
    // `base + 4`, `cargo test --workspace` read **571 passed / 0 failed** in
    // BOTH lanes and only `scripts/gate.sh` went red (10 of 12 lanes).
    // `docs/ROADMAP.md` §9.12.
    // -----------------------------------------------------------------------

    /// The `p4` body, in bytes: `lis r11,0 · li r4,7 · addi r3,r11,0 · b`.
    /// Built through the real encoders, so a change to any of them moves the
    /// input with the code rather than leaving a stale literal behind.
    fn p4_text() -> Vec<u8> {
        let mut t = codegen::encode_addis(codegen::SCRATCH_REG, 0, 0).to_vec();
        t.extend_from_slice(&codegen::encode_addi(codegen::ARG_REGS[1], 0, 7));
        t.extend_from_slice(&codegen::encode_addi(codegen::ARG_REGS[0], codegen::SCRATCH_REG, 0));
        t.extend_from_slice(&codegen::encode_tail_branch(12));
        t
    }

    fn sym_func(name: &str) -> c2_il::IlFunction {
        let mut f = codegen::testutil::func_with(Vec::new(), Vec::new());
        f.mangled_name = name.into();
        f.data_sym = Some("?gI@@3HA".into());
        f
    }

    /// **#137 rule 2, derivation half — `lo_off` is SEARCHED, not `hi_off + 4`.**
    ///
    /// MEASURED (`work/wr1/probes/p4.cpp`): `void a7(){ gsp(&gI, 7); }` is
    /// `lis r11 · li r4,7 · addi r3,r11,0 · b`, so the low half is at **+8** and
    /// the literal's `li` occupies +4. Every relocation still resolves if the
    /// quad is emitted adjacent, which is why this was a silent wrong-bytes emit
    /// and not a link error.
    #[test]
    fn the_low_half_offset_is_found_in_the_body_not_assumed_four_past_the_lis() {
        let text = p4_text();
        let f = sym_func("?a7@@YAXXZ");

        // (a) The fixture property, over the INPUT: the body must be long enough
        // for +4 and +8 to be different words, and the word at +4 must NOT be
        // the low-half `addi` — otherwise `hi_off + 4` would be right here and
        // the assertion below could not fail.
        let lo_half = codegen::encode_addi(codegen::ARG_REGS[0], codegen::SCRATCH_REG, 0);
        assert_eq!(text.len(), 16, "(a) the discriminating body is 4 words");
        assert_ne!(
            &text[4..8],
            &lo_half[..],
            "(a2) the word at hi_off+4 must be the literal's `li`, not the \
             low-half `addi` — otherwise this body does not discriminate"
        );

        // (b) Exactly one `DataRef`, pinned before it is indexed.
        let refs = data_refs_of(&f, &text, 0).expect("in class");
        assert_eq!(refs.len(), 1, "(b) expected one DataRef, got {}", refs.len());

        // (c) REFHI at the hoisted `lis`.
        assert_eq!(refs[0].hi_off, 0, "(c) hi_off is not the body's first word");

        // (d) **The rule.** REFLO at the `addi`'s own offset, 8 — not at 4.
        assert_eq!(
            refs[0].lo_off, 8,
            "(d) lo_off must be the low-half `addi`'s own offset 8, not \
             hi_off+4 = 4 — the quad's halves are NOT adjacent"
        );

        // (e) And it tracks a non-zero base, so a packed TU's second function
        // does not get the first one's offsets.
        let rebased = data_refs_of(&f, &text, 0x40).expect("in class");
        assert_eq!(
            (rebased[0].hi_off, rebased[0].lo_off),
            (0x40, 0x48),
            "(e) both halves must be rebased by the function's .text offset"
        );
    }

    /// The derivation **refuses** rather than guessing when the body is not the
    /// shape it reads. Registered here because a search that silently returns
    /// the first plausible word is the same silent-wrong-bytes shape the `+4`
    /// was: `docs/GAPS.md` §6.
    #[test]
    fn the_low_half_search_refuses_a_body_it_cannot_read() {
        let f = sym_func("?a7@@YAXXZ");
        // No `lis` first: the REFHI site would be a different instruction.
        let mut no_lis = vec![0u8; 4];
        no_lis.extend_from_slice(&p4_text()[4..]);
        assert!(
            data_refs_of(&f, &no_lis, 0).is_err(),
            "(h) a body whose first word is not the `lis` must be refused"
        );
        // A `lis` and no low half at all.
        let mut no_lo = codegen::encode_addis(codegen::SCRATCH_REG, 0, 0).to_vec();
        no_lo.extend_from_slice(&codegen::encode_tail_branch(4));
        assert!(
            data_refs_of(&f, &no_lo, 0).is_err(),
            "(i) a body with no `addi rD,r11,0` low half must be refused"
        );
        // Two low halves: ambiguous, and the search must say so rather than
        // taking the first.
        let mut two = codegen::encode_addis(codegen::SCRATCH_REG, 0, 0).to_vec();
        two.extend_from_slice(&codegen::encode_addi(codegen::ARG_REGS[0], codegen::SCRATCH_REG, 0));
        two.extend_from_slice(&codegen::encode_addi(codegen::ARG_REGS[1], codegen::SCRATCH_REG, 0));
        two.extend_from_slice(&codegen::encode_tail_branch(12));
        assert!(
            data_refs_of(&f, &two, 0).is_err(),
            "(j) two low-half `addi`s in one body must be refused, not resolved \
             to the first"
        );
        // …and a function with no data symbol yields no DataRef at all.
        let mut plain = codegen::testutil::func_with(Vec::new(), Vec::new());
        plain.data_sym = None;
        assert_eq!(
            data_refs_of(&plain, &p4_text(), 0).expect("no data symbol is fine").len(),
            0,
            "(k) a body with no data symbol must yield no DataRef"
        );
    }

    // -----------------------------------------------------------------------
    // The `/EHsc` eh-bare label slot. `PortC2::label_lead_of` carries the
    // measurement table; these pin the PREDICATE, because the rule that fits
    // half the table ("the target is defined here") turned five byte-exact objs
    // into mismatches before the `/EHsc` gate lanes rejected it.
    // -----------------------------------------------------------------------

    /// An eh-bare constructor whose unwind target is `??1B`.
    fn eh_ctor(target: &str) -> c2_il::IlFunction {
        let mut f = codegen::testutil::func_with(Vec::new(), Vec::new());
        f.mangled_name = "??0D@@QAA@XZ".into();
        f.data_sym = None;
        f.eh_bare = true;
        f.eh_unwind_callees = vec![target.to_string()];
        f
    }

    /// A destructor body, empty or not.
    fn dtor(name: &str, empty: bool) -> c2_il::IlFunction {
        let mut f = codegen::testutil::func_with(Vec::new(), Vec::new());
        f.mangled_name = name.into();
        f.data_sym = None;
        f.empty_body = empty;
        if !empty {
            f.tail_call = Some("?gh@@YAXXZ".into());
        }
        f
    }

    #[test]
    fn eh_bare_lead_is_charged_when_the_unwind_target_is_external() {
        // h0/h1/h3: `??1B` is not defined in this TU at all.
        let ctor = eh_ctor("??1B@@QAA@XZ");
        let funcs = vec![dtor("??1C@@QAA@XZ", true), ctor];
        assert_eq!(
            PortC2::label_lead_of(&funcs[1], &funcs).unwrap(),
            1,
            "an unrelated empty destructor must not suppress the slot"
        );
    }

    #[test]
    fn eh_bare_lead_is_suppressed_by_a_local_empty_unwind_target() {
        // h5/h6/h8/h9.
        let funcs = vec![dtor("??1B@@QAA@XZ", true), eh_ctor("??1B@@QAA@XZ")];
        assert_eq!(PortC2::label_lead_of(&funcs[1], &funcs).unwrap(), 0);
    }

    #[test]
    fn a_local_but_non_empty_unwind_target_still_pays() {
        // hf and hg — the two probes that separate "defined here" from
        // "defined here and empty". Without them the predicate above is wrong
        // and every `M::~M(){}`-style TU regresses.
        let funcs = vec![dtor("??1B@@QAA@XZ", false), eh_ctor("??1B@@QAA@XZ")];
        assert_eq!(
            PortC2::label_lead_of(&funcs[1], &funcs).unwrap(),
            1,
            "hf/hg: a defined target with a real body still charges the slot"
        );
    }

    #[test]
    fn a_function_that_is_not_eh_bare_is_untouched() {
        let mut f = codegen::testutil::func_with(Vec::new(), Vec::new());
        f.mangled_name = "??0D@@QAA@XZ".into();
        f.data_sym = None;
        let funcs = vec![dtor("??1B@@QAA@XZ", true), f];
        assert_eq!(PortC2::label_lead_of(&funcs[1], &funcs).unwrap(), 0);
    }

    #[test]
    fn a_mixed_unwind_list_refuses() {
        let mut ctor = eh_ctor("??1B@@QAA@XZ");
        ctor.eh_unwind_callees.push("??1G@@QAA@XZ".into());
        let funcs = vec![dtor("??1B@@QAA@XZ", true), ctor];
        assert!(
            PortC2::label_lead_of(&funcs[1], &funcs).is_err(),
            "one local-empty target and one external is unmeasured and must refuse"
        );
    }
}
