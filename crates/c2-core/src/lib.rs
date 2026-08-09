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
pub mod comdat;
pub mod elide;
pub mod passes;
pub mod splice;

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
        //
        // **W-SECT — "defines no functions" was the WRONG precondition, and it
        // was a live wrong emit for eight probe shapes.** `is_empty_module` is a
        // property of `.ex` alone; `.gl` can still declare storage that costs c2
        // a fifth or sixth section, and this arm emitted four regardless:
        //
        // ```text
        //   int g = 5;                         c2 5 sections, port 4  MISMATCH
        //   char b1;                           c2 5,           port 4  MISMATCH
        //   extern const int ce = 9;  (.rdata) c2 5,           port 4  MISMATCH
        //   const char* s = "hi";              c2 6,           port 4  MISMATCH
        //   __declspec(thread) int t1; (.tls$) c2 5,           port 4  MISMATCH
        //   __declspec(selectany) int sa = 3;  c2 5,           port 4  MISMATCH
        //   char b1; char b2;                  c2 5,           port 4  MISMATCH
        //   char b1; char d1 = 1;              c2 6,           port 4  MISMATCH
        // ```
        //
        // A wrong emit is strictly worse than a refusal, and **no standing
        // instrument could see this one**: `scripts/expr_sweep.sh` generates
        // expressions and never a bare declaration, `differential.rs` names
        // three fixtures, and the 878-TU workload contains **zero** TUs whose
        // sections are the shell plus data — so `c2rs gap` read `mismatch 0`
        // over a class it cannot represent. That is `docs/STATUS.md` trap 5,
        // *absence reads as success*, in its purest form.
        //
        // `IlBundle::shell_only_tu` asks the question this arm actually needs
        // answered — *does `.gl` name anything that would have to be given a
        // section?* — and refuses conservatively on anything it cannot account
        // for. `scripts/sweep.d/64-data-only-tu.py` generates the class from now
        // on.
        if funcs.is_empty() {
            if il.shell_only_tu() {
                return Ok(ObjImage::new(coff::emit_empty_obj(obj_name)));
            }
            // **W-SECT — the `.data`/`.bss` TU (board #174).** The refusal above
            // is the floor; this is the class that was measured out of it. The
            // decode bound lives in `IlBundle::data_tu` and the layout bound —
            // at most two objects per non-COMDAT section, §8.1 — lives in
            // `coff::emit_data_obj`, so neither crate assumes the other ran.
            if let Some(tu) = il.data_tu() {
                // **The relocations travel with the bytes from here to the
                // writer** (board #931). `DataObject::bytes` already holds each
                // one's addend, so passing the bytes and dropping this vector
                // emits a `.data` that is right about its contents and wrong
                // about its addresses — board #232's direction, out of what was
                // an honest refusal until this lane. Built beside the objects,
                // in the same iteration order, so the two cannot come apart.
                let relocs: Vec<Vec<coff::DataObjReloc>> = tu
                    .objects
                    .iter()
                    .map(|o| {
                        o.relocs
                            .iter()
                            .map(|r| coff::DataObjReloc { at: r.at, target: &r.target })
                            .collect()
                    })
                    .collect();
                let objs: Vec<coff::DataObj> = tu
                    .objects
                    .iter()
                    .zip(&relocs)
                    .map(|(o, r)| coff::DataObj {
                        symbol: &o.coff_name,
                        size: o.size,
                        natural_align: o.natural_align,
                        external: o.external,
                        bytes: o.bytes.as_deref(),
                        decl_index: o.decl_index,
                        relocs: r,
                    })
                    .collect();
                // **Every object dropped means the bare shell IS the right
                // obj.** `static int za;` — uninitialized, unreferenced,
                // internal linkage — is removed by c2 entirely, so a TU of
                // nothing but those emits 720 bytes. `shell_only_tu` says no
                // here (the `.gl` *does* name storage) and it is right to;
                // `data_tu` is the reader that knows the storage went away, and
                // it did the same exhaustive accounting before saying so.
                if objs.is_empty() {
                    return Ok(ObjImage::new(coff::emit_empty_obj(obj_name)));
                }
                if let Some(obj) = coff::emit_data_obj(obj_name, &objs) {
                    return Ok(ObjImage::new(obj));
                }
            }
            return Err(BackendError::NotImplemented(
                "a TU that defines no functions but whose `.gl` names storage \
                 the bare four-section shell does not carry: an initialized or \
                 uninitialized namespace-scope object, a `const` pool, a string \
                 literal, a thread-local, or a COMDAT. Emitting the shell here \
                 is a wrong section COUNT, which mismatches at file offset 2"
                    .to_string(),
            ));
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
        // **MECHANISM E's one input** (`crate::elide`): which of this bundle's
        // own functions have empty bodies. Resolved once per TU, here, because
        // it is the only place that sees every function — the per-function
        // composition below cannot derive it and must not guess it.
        //
        // Unreachable on today's accept boundary: `IlBundle::functions()`
        // refuses any TU that defines one of its own callees, so `tu_empty` is
        // non-empty only for a TU whose empty-bodied function nobody calls.
        // Built unconditionally anyway — the FBM instrument runs the same
        // composition over the *census*, where it does fire, and one
        // composition with two contexts is how those two stay in agreement.
        // **MECHANISM I's input comes with it** (`crate::splice`): the same pass
        // also indexes every definition this bundle can splice FROM. One context,
        // two mechanisms, one name binding — `TuContext` derefs to the E half so
        // nothing about the elision changed.
        let tu_empty = splice::TuContext::of(&funcs);

        // **MECHANISM I IS REFUSED AT THE WHOLE-OBJ LEVEL, on BOTH paths.**
        //
        // A spliced body is the callee's, and the callee's is unframed — so a
        // `Selected::Seq` caller that splices loses its frame, its `.pdata`
        // record and its `$M`/`$M`/`$T` label slots. `Self::frame_label_counter`
        // is computed from the IL *above*, before any body exists, and
        // `plan_emit_order` orders on a call edge that no longer exists in the
        // emitted obj. None of that is measured, so an obj carrying one is
        // refused rather than emitted.
        //
        // Unreachable today: `IlBundle::functions()` refuses any TU that defines
        // one of its own callees, which is a strict superset of the splice's S5.
        // Written out anyway, and asked through the same `splice_callee` the
        // composition consults, so that a future narrowing of that refusal turns
        // this into a loud refusal instead of a silent wrong obj — the lockstep
        // `elide.rs`'s packed-path arm gets by construction. This one refuses
        // the `/Gy` path too, because the label counter is a TU-level fact and
        // mechanism E's single `blr` never reached it.
        for f in &funcs {
            if let Ok(sel) = codegen::select_function(f, mode) {
                if let Some(g) = splice::splice_callee(f, &sel, &tu_empty) {
                    return Err(BackendError::NotImplemented(format!(
                        "a call c2 replaces with its callee's body (mechanism I: \
                         `{}` is `{}`'s whole emitted body) inside a whole obj: \
                         the spliced caller loses its frame, and with it its \
                         `.pdata` record and its compiler-label slots, while the \
                         label counter is computed from the IL before any body \
                         exists. Modeled per COMDAT only — \
                         docs/INLINE_PREDICATE.md §2, crates/c2-core/src/splice.rs",
                        g, f.mangled_name,
                    )));
                }
            }
        }

        // `.text` section, so the texts are kept separate rather than packed.
        // The order rule is the same one — measured at `/O1` too, where it
        // decides the section table itself and not just offsets within `.text`.
        if self.fn_level_linking {
            let mut texts: Vec<Vec<u8>> = Vec::with_capacity(funcs.len());
            let mut placed: Vec<coff::Function> = Vec::with_capacity(funcs.len());
            // **The per-function COMDAT body comes from `comdat::comdat_function_body`,
            // which is the ONE composition** (board #322). It used to be this
            // loop's inline `match`, reachable only from here — so the standing
            // per-function alarm (FUNCTION BYTE MATCH) could not ask the port
            // for a `Tail`/`Framed`/`Seq`/`CondPair` body at all and declined to
            // grade 9,375 emitted functions. Lifting it changes no byte: the
            // arms below moved verbatim, and `crates/c2-core/src/comdat.rs`
            // carries the reason the harness must call this and never a copy.
            for &fi in &order {
                let f = &funcs[fi];
                let body = comdat::comdat_function_body(f, mode, &tu_empty)?;
                placed.push(coff::Function {
                    name: &f.mangled_name,
                    text_offset: 0,
                    calls: body.calls,
                    is_float: f.touches_floating_point(),
                    mints_memcpy: f.mints_memcpy(),
                    fp_refs: Vec::new(),
                    data_refs: body.data_refs,
                    data_defs: body.data_defs,
                    frame: body.frame,
                    label_lead: leads[fi],
                    helper_externals: body.helper_externals,
                });
                texts.push(body.text);
            }
            // **W-DATA — `emit_comdat_obj` is three-valued now.** `None` is the
            // honest refusal for an obj whose defined-data shape nothing graded
            // (today: more than one object, or an alignment the container cannot
            // spell). Emitting a guess would be a wrong section count at file
            // offset 2, which is the mismatch `IlBundle::functions`' own
            // unclaimed-name gate cost two objs to learn.
            let obj = coff::emit_comdat_obj(obj_name, &placed, &texts, label_counter)
                .ok_or_else(|| {
                    BackendError::NotImplemented(
                        "a `/Gy` obj whose defined COMDAT data is outside the                          measured class: every rule about its section slot, its                          alignment nibble, its aux CheckSum and its symbol                          group was read off ONE obj, and nothing separates a                          second object's placement from any ordering that                          coincides with it at n = 1"
                            .to_string(),
                    )
                })?;
            return Ok(ObjImage::new(obj));
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
                codegen::Selected::Seq { setups, tail, park } => {
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
                        .enumerate()
                        .map(|(ix, e)| codegen::seq_early_emit_remapped(e, &park, ix))
                        .collect::<Result<Vec<_>, _>>()?;
                    let body = codegen::call_seq_text(
                        &setups,
                        &tail,
                        off,
                        codegen::FrameLayout {
                            saved_gprs: seq.saved_gprs() as u8,
                            ..Default::default()
                        },
                        &park.entry,
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
                // **MECHANISM E, refused rather than modeled on the PACKED
                // path.** The `/Gy` composition emits one `blr` here
                // (`crate::elide`); the packed emitter would have to shorten the
                // function, which moves every later function's `.text` offset,
                // every branch displacement built on it and the `.pdata`
                // association numbers. Nothing measures that, so it refuses.
                //
                // Unreachable today — `IlBundle::functions()` refuses any TU
                // that defines one of its own callees, which is a superset of
                // this condition — and it is written out anyway so that the two
                // emitters cannot silently disagree about one rule if that
                // refusal is ever narrowed.
                //
                // **The lockstep is by construction and not by review**: this
                // arm calls the same `drops_tail_call` against the same
                // `tu_empty` the `/Gy` arm consults, so the fixpoint widening
                // (board #946) widened the refusal in the same commit and by
                // the same line. A copy of the predicate here would have had to
                // be found and changed; there is none.
                codegen::Selected::Tail(_) if elide::drops_tail_call(f, &tu_empty) => {
                    return Err(BackendError::NotImplemented(
                        "a call c2 does not emit (its callee is defined in \
                         this TU by a body that reduces to nothing) inside a \
                         PACKED `.text`: the elision shortens the caller, and \
                         no capture measures what that does to the following \
                         functions' offsets. Modeled under /Gy only — \
                         docs/INLINE_PREDICATE.md §1.2, crates/c2-core/src/elide.rs"
                            .to_string(),
                    ))
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
                // W-CFG1 — the `if`/`else`-with-a-join, built at `off` for the
                // same reason every framed shape here is: both `bl` words encode
                // their own `.text` offset.
                // **W-EXTDATA — the sunk-`||`-guard body**, built at `off` for
                // the same reason every framed shape here is: all four `bl`
                // words encode their own `.text` offset.
                codegen::Selected::GuardChainSharedTail => {
                    let g = f
                        .guard_chain_shared_tail
                        .as_ref()
                        .expect("GuardChainSharedTail implies guard_chain_shared_tail");
                    let body =
                        codegen::guard_chain_shared_tail::guard_chain_shared_tail_text(
                            g, off, mode,
                        )?;
                    frame = Some(coff::Frame {
                        prolog_len: body.prolog_len,
                        func_len: body.text.len() as u32,
                    });
                    text.extend_from_slice(&body.text);
                    (
                        body.bl_offsets
                            .iter()
                            .zip([
                                g.helper.as_str(),
                                g.errno.as_str(),
                                g.errno.as_str(),
                                g.invalid.as_str(),
                            ])
                            .map(|(off, callee)| coff::Call { reloc_offset: *off, callee })
                            .collect(),
                        Vec::new(),
                    )
                }
                // **W-UNDNAME — the guarded allocation with a shared error
                // store**, built at `off` for the same reason every framed
                // shape here is: the `bl` word encodes its own `.text` offset.
                codegen::Selected::AllocInitOrFail => {
                    let a = f
                        .alloc_init_or_fail
                        .as_ref()
                        .expect("AllocInitOrFail implies alloc_init_or_fail");
                    let body =
                        codegen::alloc_init_or_fail::alloc_init_or_fail_text(a, off, mode)?;
                    frame = Some(coff::Frame {
                        prolog_len: body.prolog_len,
                        func_len: body.text.len() as u32,
                    });
                    text.extend_from_slice(&body.text);
                    (
                        vec![coff::Call {
                            reloc_offset: body.bl_offset,
                            callee: &a.alloc,
                        }],
                        Vec::new(),
                    )
                }
                // **W-OSFINFO — the range-and-flag guarded table lookup**, built
                // at `off` for the same reason every framed shape here is: both
                // `bl` words encode their own `.text` offsets.
                // **W-XLR — refused in the PACKED layout, not emitted.**
                //
                // The body itself would build fine at `off`; what has no
                // measured slot is the symbol table. Every witness of the
                // `__savegprlr_N`/`__restgprlr_N` pair's placement — after the
                // `$T` label, `docs/CODEGEN_FRAMED_CALLS.md` §2.3a — is a `/Gy`
                // obj, and the packed writer has no `$T` group per function to
                // place them after. Guessing would be a wrong symbol index in an
                // obj that still links, which is `docs/GAPS.md` §6's shape.
                //
                // It costs the workload nothing: the class is `/O1` only and
                // `/O1` implies `/Gy`, so this arm is unreachable from the
                // workload and exists so the `/Ox` gate lane gets a refusal
                // instead of a guess.
                codegen::Selected::XlrcCreateGuard => {
                    return Err(BackendError::NotImplemented(
                        "the `__savegprlr_N` frame class in the PACKED (non-`/Gy`) \
                         layout: the helper pair's symbol records are witnessed \
                         only after a `$T` label, which the packed symbol table \
                         does not have"
                            .to_string(),
                    ));
                }
                // **W-JSON — refused in the PACKED layout, not emitted**, for
                // exactly W-XLR's reason: every witness of the helper pair's
                // symbol placement is a `/Gy` obj with a `$T` to put them after,
                // and the packed symbol table has none. The class is `/O1` only
                // and `/O1` implies `/Gy`, so this arm is unreachable from the
                // workload and exists so the `/Ox` gate lane gets a refusal
                // instead of a guess.
                codegen::Selected::JsonUtf8Copy => {
                    return Err(BackendError::NotImplemented(
                        "the frameless `__savegprlr_N` frame class in the PACKED \
                         (non-`/Gy`) layout: the helper pair's symbol records are \
                         witnessed only after a `$T` label, which the packed symbol \
                         table does not have"
                            .to_string(),
                    ));
                }
                codegen::Selected::OsfHandleGuard => {
                    let g = f
                        .osf_handle_guard
                        .as_ref()
                        .expect("OsfHandleGuard implies osf_handle_guard");
                    let body = codegen::osf_handle_guard::osf_handle_guard_text(g, off, mode)?;
                    frame = Some(coff::Frame {
                        prolog_len: body.prolog_len,
                        func_len: body.text.len() as u32,
                    });
                    text.extend_from_slice(&body.text);
                    (
                        vec![
                            coff::Call { reloc_offset: body.bl_offsets[0], callee: &g.errno },
                            coff::Call { reloc_offset: body.bl_offsets[1], callee: &g.doserrno },
                        ],
                        Vec::new(),
                    )
                }
                // **W-IFN — refused in the PACKED layout, for W-XLR's reason.**
                // Its one external is MINTED rather than IL-named, and c2 places
                // a minted external after the `$T` label — measured,
                // `work/w-ifn/probe/lab_z.cpp` puts `memcpy` after the first
                // user's `$T2587` while the IL-named `?gz@@YAHH@Z` sits between
                // that function's two `$M`s. The packed symbol table has no `$T`
                // and therefore no measured slot for it. Unreachable as written
                // — the class is `/O1` only and `/O1` implies `/Gy`
                // (`docs/OPT_MODE.md` §3.3) — and a named refusal rather than an
                // `unreachable!()` because an unreachable arm that becomes
                // reachable is how a guessed layout ships.
                codegen::Selected::GuardRetChain => {
                    return Err(BackendError::NotImplemented(
                        "the minted-external class in the PACKED (non-`/Gy`) \
                         layout: `memcpy` is witnessed only after a `$T` label, \
                         which the packed symbol table does not have"
                            .to_string(),
                    ));
                }
                codegen::Selected::IfCallJoin => {
                    let j = f.if_call_join.as_ref().expect("IfCallJoin implies if_call_join");
                    let body = codegen::if_call_join::if_call_join_text(j, off, mode)?;
                    frame = Some(coff::Frame {
                        prolog_len: body.prolog_len,
                        func_len: body.text.len() as u32,
                    });
                    text.extend_from_slice(&body.text);
                    (
                        vec![
                            coff::Call {
                                reloc_offset: body.bl_offsets[0],
                                callee: &j.callee_hi,
                            },
                            coff::Call {
                                reloc_offset: body.bl_offsets[1],
                                callee: &j.callee_lo,
                            },
                        ],
                        Vec::new(),
                    )
                }
                codegen::Selected::Plain(body) => {
                    text.extend_from_slice(&body);
                    (Vec::new(), Vec::new())
                }
            };
            let data_refs = data_refs_of(f, &text[off as usize..], off)?;
            // **W-DATA — the PACKED layout has no measured slot for a COMDAT
            // `.data`**, so a function that defines one refuses here rather than
            // reaching `emit_obj`. The packed section table interleaves
            // `.rdata` and `.pdata` in `.text` order (six distinct orders over
            // 240 objs, `emit_obj`'s own comment), and where a COMDAT `.data`
            // goes in that sequence is a seventh thing nobody has captured.
            //
            // It costs the workload nothing: every TU this class reaches
            // compiles at `/O1`, which implies `/Gy`, so the branch above is
            // the one that runs. This arm exists so the `/Ox` gate lane gets a
            // refusal instead of a guess.
            if f.data_def.is_some() {
                return Err(BackendError::NotImplemented(
                    "a function-local `static` in the PACKED (`/Ox`) layout:                      the COMDAT `.data`'s position relative to `.rdata` and                      `.pdata` — which interleave in `.text` order — has never                      been captured"
                        .to_string(),
                ));
            }
            placed.push(coff::Function {
                name: &f.mangled_name,
                text_offset: off,
                calls,
                is_float: f.touches_floating_point(),
                mints_memcpy: f.mints_memcpy(),
                fp_refs,
                data_refs,
                data_defs: Vec::new(),
                frame,
                label_lead: leads[fi],
                // **W-XLR — always empty on this path, by the refusal above.**
                // The packed layout has no measured slot for a symbol placed
                // after `$T`: every witness of the `__savegprlr_N` pair's
                // placement (`docs/CODEGEN_FRAMED_CALLS.md` §2.3a) is a `/Gy`
                // obj. Refused rather than guessed, exactly as the COMDAT
                // `.data` above is.
                helper_externals: Vec::new(),
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
                    is_function: false,
                },
                coff::DataRef {
                    hi_off: body.object_hi,
                    lo_off: body.object_lo,
                    name: &tu.object_symbol,
                    is_function: false,
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
/// high half cannot be located unambiguously.
///
/// # W-EXTDATA — the `lis` is no longer required to be the body's FIRST word
///
/// It was, and the clause was right for every class that had reached here: the
/// `Selected::Tail`/`Seq` setups hoist it to word 0. `_vswprintf_s_l` puts it at
/// **word 14**, interleaved into a five-deep argument rotate
/// (`work/w-extdata/VSWPRNC_BODY.md` §3), so the position rule refuses a body
/// c2 emits.
///
/// # W-UNDNAME — the pairing is POSITIONAL, and there may be more than one
///
/// W-EXTDATA replaced the position rule with a *uniqueness* rule: the high half
/// is the one `addis rT,0,0` in the body, two of them refuse. That refusal was
/// not hypothetical and it is now paid.
/// `?append@DName@@QAAXPAVDNameNode@@@Z` materializes **two** symbols
/// (`work/w-undname/UNDNAME_BODY.md` §3):
///
/// ```text
///   +0x24  lis  r11,0      REFHI ?gHeapManager      ┐ low half is an ARG_REG,
///   +0x2c  addi r3,r11,0   REFLO ?gHeapManager      ┘ 2 words below
///   +0x40  lis  r11,0      REFHI ?pairNode_vtable   ┐ low half is the SCRATCH
///   +0x4c  addi r11,r11,0  REFLO ?pairNode_vtable   ┘ ITSELF, 3 words below
/// ```
///
/// Two different hoist distances in one body, and one low half writing the very
/// register the high half lives in — so **neither** a fixed distance **nor** a
/// search restricted to `addi <ARG_REG>,r11,0` can derive both. The rule that
/// does: walk the words once; each `addis rT,0,0` **opens** a pair and the first
/// low half after it **closes** it, `rD` unconstrained. An `addis` that
/// never closes, or a low half before any high half, refuses.
///
/// # W-OSFINFO: the walk is no longer keyed on the SCRATCH register, and a low
/// # half can be a `lwz` DISPLACEMENT
///
/// `_free_osfhnd` (`src/xdk/LIBCMT/osfinfo.cpp`) breaks the walk above in two
/// independent ways, and only one of them was a missing clause:
///
/// ```text
///   +0x14  lis  r11,0       REFHI _nhandle   ┐ the low half is a **`lwz`
///   +0x18  lwz  r11,0(r11)  REFLO _nhandle   ┘ displacement**: the global's
///                                              VALUE is loaded, so nothing
///                                              takes its address and there is
///                                              no `addi` in the body at all
///   +0x28  lis  r10,0       REFHI __pioinfo  ┐ the high half is in **r10**.
///   +0x2c  slwi r9,r11,2                     │ The walk keyed its `addis`
///   +0x30  addi r10,r10,0   REFLO __pioinfo  ┘ test on `SCRATCH_REG`, so this
///                                              quad was INVISIBLE to it — not
///                                              merely unpaired
/// ```
///
/// So the `addis` test is now over **any** `rT`, the open slot carries that
/// register, and the closer must name the same one. Two closer forms are
/// admitted and they are **not** symmetric:
///
/// * `addi rD,rT,0` — never an ordinary instruction. c2 spells a register copy
///   `mr` (an `or`), so a zero-displacement `addi` off a non-zero base is always
///   a relocation low half. It therefore keeps its **canary**: one appearing
///   with no open high half above it still refuses.
/// * `lwz rD,0(rT)` — an ordinary load, all day long. This very body has one
///   that is **not** a relocation (`lwz r10,0(r11)` at `+0x50`, reading the
///   table entry's handle word). It is admitted **only while a pair on `rT` is
///   open**, and it carries no canary, because one would refuse every body that
///   loads through a base register at displacement zero.
///
/// # Which form closes a pair is decided by LOOKAHEAD, and that is what makes
/// # the widening byte-neutral BY CONSTRUCTION
///
/// If the walk simply closed on whichever form came first, a previously-accepted
/// body with a `lwz rD,0(r11)` sitting between a REFHI and its REFLO would
/// close early and relocate a different word — a silent byte change in an obj
/// that already matched. So the form is chosen when the pair **opens**: within
/// the window from the open to the next `addis` (or the end), an `addi rD,rT,0`
/// wins if one exists at all, and only otherwise is the `lwz` form considered.
///
/// **Every body the old walk accepted had an `addi` in that window** — it is
/// what closed the pair — so every one of them takes the identical word, and no
/// obj the port had ever emitted can move. That is an argument from the shape of
/// the old code rather than from a survey, and
/// `data_ref_tests::the_lwz_form_never_preempts_an_addi_form` is it as a
/// `#[test]`. It is *also* measured, per PREREG **D4**: the 878-TU scan at base
/// and tip, diffed per TU by name.
///
/// A body where both forms sit in the window still takes the `addi`, **even
/// when the `lwz` comes first** — that is precisely the case the preference
/// exists for, and it is what the old walk did.
///
/// **The discipline the original clause was written for is kept**: the sites are
/// DERIVED from the bytes and never declared by the class, so a schedule change
/// cannot silently relocate the wrong instruction.
///
/// # The names come from the parser, and the two lists must AGREE
///
/// [`c2_il::IlFunction::data_syms`] is in emission order, so pair `i` takes name
/// `i`. The counts are checked here rather than assumed: a body whose derived
/// pair count differs from its name count refuses, because the alternative is a
/// relocation against the wrong symbol — which links, and is `docs/GAPS.md` §6's
/// silent wrong-bytes shape.
///
/// # The order constraint is GONE, and that is board #1720 shipping
///
/// This function's doc used to end with a paragraph explaining that
/// `coff::writer` emits callees first and then data symbols, that the reference
/// tables are one union list in reverse first-reference order, that the two
/// agree only while every data reference precedes every call, and that
/// `check_external_order` re-imposed the difference as a refusal. **The writer
/// emits the union list now** ([`coff::Function::introduced_externals`]), so the
/// fence has nothing left to protect and is deleted rather than left standing as
/// a dead clause. GRID A is `work/w-extdata/GRID_A_RESULT.md`; the cell that
/// exercises the new arm is this very body, whose externals are
/// `data · callee · data`.
/// Which of the two low-half forms a given REFHI/REFLO pair is looking for.
///
/// Decided once, when the pair opens — see [`data_refs_of`]'s doc on why a
/// first-match walk over both forms would not be byte-neutral.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LoForm {
    /// `addi rD,rT,0` — the address is TAKEN. Every body the port emitted before
    /// W-OSFINFO uses this form and only this form.
    Addi,
    /// `lwz rD,0(rT)` — the global's VALUE is loaded and the relocation rides
    /// the load's displacement field. `_free_osfhnd`'s range bound.
    Lwz,
}

/// `addis rT,0,0` — a relocation HIGH half, for any `rT`. Returns `rT`.
///
/// The `rA = 0` and `SIMM = 0` fields are what distinguish this from an ordinary
/// `addis`: c2 writes the placeholder as zero and the linker fills it.
fn hi_half_reg(w: &[u8]) -> Option<u8> {
    (0u8..32).find(|&rt| w == codegen::encode_addis(rt, 0, 0))
}

/// `addi rD,rT,0` for the given base, any destination.
///
/// `li rD,k` cannot be mistaken for one: its `RA` field is 0, and every caller
/// here excludes `rT = 0`.
fn is_addi_lo(w: &[u8], rt: u8) -> bool {
    (0u8..32).any(|d| w == codegen::encode_addi(d, rt, 0))
}

/// `lwz rD,0(rT)` for the given base, any destination.
fn is_lwz_lo(w: &[u8], rt: u8) -> bool {
    (0u8..32).any(|d| w == codegen::encode_lwz(d, rt, 0))
}

/// Choose the low-half form for the pair opening at word `hi_i` on register
/// `rt`, by looking ahead to the next high half (or the end).
///
/// **`Addi` wins whenever one exists in the window at all.** That is the clause
/// that makes admitting the `lwz` form byte-neutral on every body the port
/// already emits: each of those closed on an `addi`, so each still does, on the
/// identical word. Only a window with no `addi` at all reaches the `lwz`.
///
/// A window holding **both**, with the `lwz` first, still takes the `addi` — and
/// that is the case the preference exists for. The old walk took the `addi`
/// there, every obj it produced is graded, and reversing it would move bytes in
/// a matching obj. Refusing instead would be no better: it would turn an
/// accepted body into a refused one, which loses a TU for a shape that is
/// already right.
///
/// So the `lwz` arm is reachable **only** from a window with no `addi` in it at
/// all, which no previously-accepted body has. The widening can therefore turn a
/// refusal into an acceptance and can do nothing else.
fn lo_form_for(words: &[&[u8]], hi_i: usize, rt: u8) -> LoForm {
    for w in words.iter().skip(hi_i + 1) {
        // The window ends at the next high half: past it a second pair is open
        // and the walk refuses anyway.
        if hi_half_reg(w).is_some() {
            break;
        }
        if is_addi_lo(w, rt) {
            return LoForm::Addi;
        }
    }
    // Either a `lwz` low half, or no closer at all — in which case the walk's
    // own "high half with no low half" clause reports it, with the offset to
    // name.
    LoForm::Lwz
}

pub(crate) fn data_refs_of<'a>(
    f: &'a c2_il::IlFunction,
    text: &[u8],
    base: u32,
) -> Result<Vec<coff::DataRef<'a>>, BackendError> {
    // The two carriers name different symbol RECORDS — a data name is
    // `Type 0x0000` and a function's address `Type 0x0020` — and nothing in the
    // derived sites tells them apart, because both are the same `lis`/`addi`
    // quad. A body setting both would need this function to decide which pair is
    // which, and no cell says. Refused loudly rather than guessed.
    if !f.data_syms.is_empty() && f.fn_addr_sym.is_some() {
        return Err(BackendError::NotImplemented(
            "a body naming BOTH a data symbol and a function's address: two \
             REFHI/REFLO quads whose `lis`es are the same word and whose symbol \
             records are not"
                .to_string(),
        ));
    }
    let (names, is_function): (Vec<&'a str>, bool) = match f.fn_addr_sym.as_deref() {
        Some(n) => (vec![n], true),
        None if f.data_syms.is_empty() => return Ok(Vec::new()),
        None => (f.data_syms.iter().map(String::as_str).collect(), false),
    };
    let words: Vec<&[u8]> = text.chunks_exact(4).collect();
    // Every register some `addis rT,0,0` in this body writes. Only these can
    // carry the `addi` canary below — an `addi rD,rT,0` off a register no high
    // half ever wrote is not a low half of anything.
    let mut hi_regs: u32 = 0;
    for w in &words {
        if let Some(rt) = hi_half_reg(w) {
            hi_regs |= 1 << rt;
        }
    }
    // One forward walk. `open` holds the `.text` word index of the `addis` whose
    // low half has not been seen yet, the register it wrote, and which of the two
    // closer forms this pair is looking for; a second `addis` before that one
    // closes is a body this derivation cannot read.
    let mut pairs: Vec<(u32, u32)> = Vec::new();
    let mut open: Option<(usize, u8, LoForm)> = None;
    for (i, w) in words.iter().enumerate() {
        let off = base + 4 * i as u32;
        if let Some(rt) = hi_half_reg(w) {
            if open.is_some() {
                return Err(BackendError::NotImplemented(
                    "a second `lis rS,sym@ha` before the first one's low half: \
                     the relocation sites are derived by a forward walk and two \
                     open high halves cannot be told apart"
                        .to_string(),
                ));
            }
            if rt == 0 {
                return Err(BackendError::NotImplemented(
                    "a `lis r0,sym@ha`: r0 reads as the literal zero in the base \
                     field of every low-half form, so the pair cannot be closed \
                     unambiguously"
                        .to_string(),
                ));
            }
            open = Some((i, rt, lo_form_for(&words, i, rt)));
            continue;
        }
        match open {
            // A pair is open: only the form chosen at the open closes it, and
            // only on the same base register.
            Some((hi_i, rt, form)) => {
                let closes = match form {
                    LoForm::Addi => is_addi_lo(w, rt),
                    LoForm::Lwz => is_lwz_lo(w, rt),
                };
                if closes {
                    pairs.push((base + 4 * hi_i as u32, off));
                    open = None;
                }
            }
            // No pair is open. An `addi rD,rT,0` off a register some `addis`
            // in this body wrote is a low half with nothing above it — the
            // canary the scratch-keyed walk carried, kept. A `lwz rD,0(rT)` is
            // NOT: an ordinary zero-displacement load, of which this very class
            // contains one.
            None => {
                if (1u8..32).any(|rt| hi_regs & (1 << rt) != 0 && is_addi_lo(w, rt)) {
                    return Err(BackendError::NotImplemented(
                        "an `addi rD,rT,0` low half with no open `lis rT,…` above it"
                            .to_string(),
                    ));
                }
            }
        }
    }
    if open.is_some() {
        return Err(BackendError::NotImplemented(
            "a `lis rS,sym@ha` high half with no `addi rD,rS,0` or `lwz rD,0(rS)` \
             low half below it"
                .to_string(),
        ));
    }
    if pairs.len() != names.len() {
        return Err(BackendError::NotImplemented(format!(
            "{} REFHI/REFLO pair(s) derived from the emitted words against {} \
             symbol name(s) from the parser: the two lists are paired by \
             position and a mismatch would relocate a site against the wrong \
             symbol",
            pairs.len(),
            names.len()
        )));
    }
    Ok(pairs
        .into_iter()
        .zip(names)
        .map(|((hi_off, lo_off), name)| coff::DataRef { hi_off, lo_off, name, is_function })
        .collect())
}

/// **W-DATA — the relocation sites for a data object this TU DEFINES**, derived
/// from the emitted words rather than declared by the class.
///
/// [`data_refs_of`]'s discipline, one fan-out wider, and for the reason that
/// function's own doc gives: a class that *declared* `hi_off = 0` and
/// `lo_offs = [8, 12]` would keep saying so after a schedule change, and every
/// relocation would still resolve — `docs/GAPS.md` §6's silent wrong-bytes
/// shape. Derived here, the offsets cannot drift from the bytes.
///
/// The derivation, and each clause is a refusal rather than a guess:
///
/// * **the high half** is the unique `addis rT,0,0` in the body, and `rT` must
///   not be `r0`. `lis` is the only way this port materializes a symbol's high
///   half, and a second one would mean two objects sharing a scratch — a
///   different allocation nobody has graded;
/// * **the low halves** are every word whose `RA` is that same `rT` and whose
///   16-bit displacement field is **0** — `addi rD,rT,0` (the base address) and
///   `lwz rD,0(rT)` (a peeled element). Both spellings occur in `Primes.cpp`;
///   the `lwzx` forms in the same body are X-form and cannot match.
///
/// **The count is not fixed at one.** `Primes.cpp` has TWO low halves against
/// one high half, which is `w-loop`'s R3, and a 1:1 derivation would emit five
/// relocation records where c2 emits six — a wrong `NumberOfRelocations` at the
/// section header, the aux record and the plan at once.
pub(crate) fn data_defs_of<'a>(
    f: &'a c2_il::IlFunction,
    base: u32,
) -> Result<Vec<coff::DataDef<'a>>, BackendError> {
    let Some(d) = f.data_def.as_ref() else {
        return Ok(Vec::new());
    };
    let text = match codegen::static_scan_loop::static_scan_loop_text(f) {
        Some(t) => t,
        None => {
            return Err(BackendError::NotImplemented(
                "a defined data object on a function whose class does not \
                 materialize its address: nothing derives the relocation sites"
                    .to_string(),
            ))
        }
    };
    let words: Vec<[u8; 4]> = text.chunks_exact(4).map(|w| [w[0], w[1], w[2], w[3]]).collect();
    let mut hi: Option<(u32, u8)> = None;
    for (i, w) in words.iter().enumerate() {
        for rt in 1u8..32 {
            if *w == codegen::encode_addis(rt, 0, 0) {
                if hi.is_some() {
                    return Err(BackendError::NotImplemented(
                        "two `lis rT,sym@ha` high halves in one body: two \
                         objects sharing an address scratch is an allocation \
                         nothing has graded"
                            .to_string(),
                    ));
                }
                hi = Some((base + 4 * i as u32, rt));
            }
        }
    }
    let Some((hi_off, rt)) = hi else {
        return Err(BackendError::NotImplemented(
            "a defined data object with no `lis rT,sym@ha` high half".to_string(),
        ));
    };
    let mut lo_offs = Vec::new();
    for (i, w) in words.iter().enumerate() {
        let at = base + 4 * i as u32;
        if at <= hi_off {
            continue;
        }
        let is_lo = (0u8..32).any(|rd| {
            *w == codegen::encode_addi(rd, rt, 0) || *w == codegen::encode_lwz(rd, rt, 0)
        });
        if is_lo {
            lo_offs.push(at);
        }
    }
    if lo_offs.is_empty() {
        return Err(BackendError::NotImplemented(
            "a defined data object whose high half has no low half".to_string(),
        ));
    }
    Ok(vec![coff::DataDef {
        symbol: &d.coff_name,
        size: d.size,
        natural_align: d.natural_align,
        bytes: &d.bytes,
        hi_off,
        lo_offs,
    }])
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
        f.data_syms = vec!["?gI@@3HA".into()];
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
        plain.data_syms.clear();
        assert_eq!(
            data_refs_of(&plain, &p4_text(), 0).expect("no data symbol is fine").len(),
            0,
            "(k) a body with no data symbol must yield no DataRef"
        );
    }

    /// **W-UNDNAME — TWO quads in one body, paired by POSITION.**
    ///
    /// The body is `?append@DName@@QAAXPAVDNameNode@@@Z`'s two hoists, reduced
    /// to the words that matter: the first pair is 2 words apart and its low
    /// half writes an `ARG_REG`; the second is 3 words apart and its low half
    /// writes the **scratch register itself**. Neither a fixed distance nor an
    /// `ARG_REG`-restricted search can derive both, which is what the previous
    /// rule was and why it refused this body by name.
    #[test]
    fn two_quads_in_one_body_pair_by_position_and_take_their_names_in_order() {
        let mut f = codegen::testutil::func_with(Vec::new(), Vec::new());
        f.mangled_name = "?append@DName@@QAAXPAVDNameNode@@@Z".into();
        f.data_syms = vec!["?gObj@@3HA".into(), "?gVt@@3PAXA".into()];
        let mut t = Vec::new();
        t.extend_from_slice(&codegen::encode_addis(codegen::SCRATCH_REG, 0, 0)); // 0x00
        t.extend_from_slice(&codegen::encode_addi(codegen::ARG_REGS[2], 0, 0)); // 0x04 li
        t.extend_from_slice(&codegen::encode_addi(codegen::ARG_REGS[0], codegen::SCRATCH_REG, 0)); // 0x08
        t.extend_from_slice(&codegen::encode_addi(codegen::ARG_REGS[1], 0, 16)); // 0x0c li
        t.extend_from_slice(&codegen::encode_addis(codegen::SCRATCH_REG, 0, 0)); // 0x10
        t.extend_from_slice(&codegen::encode_stw(30, 3, 8)); // 0x14
        t.extend_from_slice(&codegen::encode_addi(10, 0, -1)); // 0x18 li
        t.extend_from_slice(&codegen::encode_addi(codegen::SCRATCH_REG, codegen::SCRATCH_REG, 0)); // 0x1c

        let refs = data_refs_of(&f, &t, 0).expect("in class");
        assert_eq!(refs.len(), 2, "two quads");
        assert_eq!((refs[0].hi_off, refs[0].lo_off), (0x00, 0x08));
        assert_eq!((refs[1].hi_off, refs[1].lo_off), (0x10, 0x1c));
        // The names travel in EMISSION order and are paired by index.
        assert_eq!(refs[0].name, "?gObj@@3HA");
        assert_eq!(refs[1].name, "?gVt@@3PAXA");
        // The two hoist distances differ, which is the whole reason the pairing
        // cannot be a distance.
        assert_ne!(refs[0].lo_off - refs[0].hi_off, refs[1].lo_off - refs[1].hi_off);
        // …and the second low half is NOT an `ARG_REG`, which is what the old
        // search required.
        assert!(!codegen::ARG_REGS.iter().any(|&d| {
            t[0x1c..0x20] == codegen::encode_addi(d, codegen::SCRATCH_REG, 0)
        }));

        // The counts are checked: one name against two derived pairs refuses
        // rather than relocating a site against the wrong symbol.
        let mut short = f.clone();
        short.data_syms.pop();
        assert!(data_refs_of(&short, &t, 0).is_err(), "count mismatch must refuse");
    }

    /// `_free_osfhnd`'s two quads, reduced to the words that matter, plus the
    /// **decoy**: the `lwz r10,0(r11)` at `+0x50` that reads the table entry's
    /// handle word and is not a relocation of anything.
    fn osfinfo_text() -> Vec<u8> {
        let s = codegen::SCRATCH_REG;
        let mut t = Vec::new();
        t.extend_from_slice(&codegen::encode_cmpwi(codegen::CR_COMPARE, 3, 0)); // 0x00
        t.extend_from_slice(&codegen::encode_addis(s, 0, 0)); //                   0x04  REFHI
        t.extend_from_slice(&codegen::encode_lwz(s, s, 0)); //                     0x08  REFLO
        t.extend_from_slice(&codegen::encode_cmplw(codegen::CR_COMPARE, 3, s)); // 0x0c
        t.extend_from_slice(&codegen::encode_srawi(s, 3, 5)); //                   0x10
        t.extend_from_slice(&codegen::encode_addis(10, 0, 0)); //                  0x14  REFHI
        t.extend_from_slice(&codegen::encode_rlwinm(9, s, 2, 0, 29)); //           0x18
        t.extend_from_slice(&codegen::encode_addi(10, 10, 0)); //                  0x1c  REFLO
        t.extend_from_slice(&codegen::encode_lwzx(10, 9, 10)); //                  0x20
        t.extend_from_slice(&codegen::encode_add(s, 10, s)); //                    0x24
        t.extend_from_slice(&codegen::encode_lwz(10, s, 0)); //                    0x28  THE DECOY
        t.extend_from_slice(&codegen::encode_stw(10, s, 0)); //                    0x2c
        t
    }

    /// **W-OSFINFO — the walk is no longer keyed on the scratch register, and a
    /// low half can be a `lwz` DISPLACEMENT.**
    ///
    /// Two quads, and each breaks the shipped walk a different way: the first's
    /// low half is a `lwz` (no `addi` for it exists anywhere in the body) and
    /// the second's high half is in **r10**, which the `addis`-on-`SCRATCH_REG`
    /// test could not see at all.
    #[test]
    fn a_lwz_displacement_and_a_non_scratch_high_half_both_pair() {
        let mut f = codegen::testutil::func_with(Vec::new(), Vec::new());
        f.mangled_name = "_free_osfhnd".into();
        f.data_syms = vec!["_nhandle".into(), "__pioinfo".into()];
        let t = osfinfo_text();

        let refs = data_refs_of(&f, &t, 0).expect("in class");
        assert_eq!(refs.len(), 2, "two quads");
        assert_eq!((refs[0].hi_off, refs[0].lo_off), (0x04, 0x08));
        assert_eq!((refs[1].hi_off, refs[1].lo_off), (0x14, 0x1c));
        assert_eq!(refs[0].name, "_nhandle");
        assert_eq!(refs[1].name, "__pioinfo");

        // (a) The first low half really is a `lwz` and NOT any `addi rD,r11,0`,
        // so the shipped clause could not have matched it.
        assert!(!(0u8..32).any(|d| t[0x08..0x0c] == codegen::encode_addi(d, codegen::SCRATCH_REG, 0)));
        // (b) The second high half really is NOT the scratch register, so the
        // shipped `addis` test could not have opened it.
        assert_ne!(&t[0x14..0x18], &codegen::encode_addis(codegen::SCRATCH_REG, 0, 0)[..]);
    }

    /// **THE DECOY: a `lwz rD,0(rT)` with no open high half is NOT a low half**,
    /// and the same word IS one four words earlier where a pair is open.
    ///
    /// This is the asymmetry between the two closer forms stated as a test. An
    /// `addi rD,rT,0` off a relocated base is never an ordinary instruction, so
    /// it keeps its canary; a zero-displacement load is one all day long, and a
    /// canary on it would refuse every body that loads through a base register.
    #[test]
    fn a_zero_displacement_load_with_no_open_pair_is_not_a_low_half() {
        let mut f = codegen::testutil::func_with(Vec::new(), Vec::new());
        f.mangled_name = "_free_osfhnd".into();
        f.data_syms = vec!["_nhandle".into(), "__pioinfo".into()];
        let t = osfinfo_text();

        // The decoy at 0x28 and the REFLO at 0x08 are `lwz` off the SAME base
        // register at the SAME displacement — only one of them relocates, and
        // nothing in the word says which.
        assert!(is_lwz_lo(&t[0x08..0x0c], codegen::SCRATCH_REG));
        assert!(is_lwz_lo(&t[0x28..0x2c], codegen::SCRATCH_REG));
        let refs = data_refs_of(&f, &t, 0).expect("in class");
        assert!(refs.iter().all(|r| r.lo_off != 0x28), "the decoy is not a site");

        // …and the `addi` form's canary is still live: the same body with an
        // `addi rD,r11,0` where the decoy is refuses outright.
        let mut with_stray = t.clone();
        with_stray[0x28..0x2c]
            .copy_from_slice(&codegen::encode_addi(10, codegen::SCRATCH_REG, 0));
        assert!(
            data_refs_of(&f, &with_stray, 0).is_err(),
            "an `addi rD,rT,0` with no open high half must still refuse"
        );
    }

    /// **The widening is byte-neutral on the shipped population BY
    /// CONSTRUCTION**, and this is that argument as executable code rather than
    /// as a paragraph.
    ///
    /// Every body the port emitted before this change closed its pairs on an
    /// `addi`. A first-match walk over both forms would close early on any
    /// `lwz rD,0(rT)` that happened to sit between a REFHI and its REFLO — a
    /// different relocated word in an obj that already matched. The lookahead
    /// prevents it: `Addi` wins whenever one exists in the window at all.
    #[test]
    fn the_lwz_form_never_preempts_an_addi_form() {
        let s = codegen::SCRATCH_REG;
        let mut f = codegen::testutil::func_with(Vec::new(), Vec::new());
        f.mangled_name = "?w@@YAXXZ".into();
        f.data_syms = vec!["?gI@@3HA".into()];

        // `lis r11 · lwz r5,0(r11) · addi r3,r11,0` — the load FIRST.
        let mut t = codegen::encode_addis(s, 0, 0).to_vec();
        t.extend_from_slice(&codegen::encode_lwz(5, s, 0)); //     0x04
        t.extend_from_slice(&codegen::encode_addi(3, s, 0)); //    0x08
        t.extend_from_slice(&codegen::encode_blr());

        // The two candidate words are both present and both match their form…
        assert!(is_lwz_lo(&t[0x04..0x08], s));
        assert!(is_addi_lo(&t[0x08..0x0c], s));
        // …the lookahead picks `Addi` even though the load comes FIRST…
        assert_eq!(lo_form_for(&t.chunks_exact(4).collect::<Vec<_>>(), 0, s), LoForm::Addi);
        // …and the pair closes on the `addi` at 0x08, which is the word the
        // shipped walk closed on. A first-match walk over both forms would
        // relocate 0x04 instead: different bytes, in an obj that already
        // matched.
        let refs = data_refs_of(&f, &t, 0).expect("in class");
        assert_eq!((refs[0].hi_off, refs[0].lo_off), (0x00, 0x08));

        // With the load AFTER the `addi`, there is no disagreement: the `addi`
        // closes the pair and the load is an ordinary instruction. This is the
        // shape every previously-emitted obj has, and it is unchanged.
        let mut t2 = codegen::encode_addis(s, 0, 0).to_vec();
        t2.extend_from_slice(&codegen::encode_addi(3, s, 0)); //  0x04
        t2.extend_from_slice(&codegen::encode_lwz(5, s, 0)); //   0x08
        t2.extend_from_slice(&codegen::encode_blr());
        let refs = data_refs_of(&f, &t2, 0).expect("in class");
        assert_eq!((refs[0].hi_off, refs[0].lo_off), (0x00, 0x04));
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
        f.data_syms.clear();
        f.eh_bare = true;
        f.eh_unwind_callees = vec![target.to_string()];
        f
    }

    /// A destructor body, empty or not.
    fn dtor(name: &str, empty: bool) -> c2_il::IlFunction {
        let mut f = codegen::testutil::func_with(Vec::new(), Vec::new());
        f.mangled_name = name.into();
        f.data_syms.clear();
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
        f.data_syms.clear();
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
