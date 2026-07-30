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

    /// The `$M…`/`$T…` label counter seed for this TU, or the reason to refuse.
    ///
    /// Returns `Ok(0)` — an unused value — when no function in the TU is framed,
    /// because then no label is emitted and `coff::plan_labels` yields `None`
    /// everywhere. Otherwise the seed must be **read**, and every function in the
    /// TU must belong to a class whose counter stride was measured, because the
    /// counter is consumed by *every* function, framed or not:
    ///
    /// | class | slots consumed |
    /// |---|---|
    /// | int leaf, empty body, tail call, indirect load, address leaf | 1 |
    /// | framed non-leaf call | 4 packed, 5 under `/Gy` |
    /// | **comparison leaf** (`a < b`) | **3** |
    /// | **floating-point leaf** | **2**, plus **2 per pooled constant** |
    ///
    /// The last two rows are the reason for the gate rather than a note: a
    /// preceding compare leaf shifts every later `$M` number by 2, and a `$M`
    /// with the wrong number is a wrong-bytes obj that still links. Measured by
    /// compiling `<class> ; int F(int a){return g(a)+1;}` and reading `F`'s
    /// labels out of the reference obj against the `.gl` seed
    /// (`docs/OBJ_GY_SHAPES.md` §3.5).
    ///
    /// It over-refuses, and by a measured amount: the comparison stride is not
    /// uniform over the relation (`a == b` and `a < 0` consume 1, `a < b` and
    /// `a >= b` consume 3) and this gate keys on the class, not the relation. On
    /// the generated sweep that is 6 of 21 framed-plus-refuser cases refused
    /// unnecessarily. The cost is a refusal, never a wrong byte, and closing it
    /// means a stride table per relation and operand type.
    fn frame_label_counter(
        il: &IlBundle,
        funcs: &[c2_il::IlFunction],
    ) -> Result<u32, BackendError> {
        if !funcs.iter().any(|f| f.framed_call.is_some()) {
            return Ok(0);
        }
        if let Some(f) = funcs.iter().find(|f| f.compare.is_some() || f.float_leaf.is_some()) {
            return Err(BackendError::NotImplemented(format!(
                "framed function in a TU that also holds `{}`, a comparison or \
                 floating-point leaf: those consume 3 and 2 compiler-label slots \
                 respectively where every class this port emits consumes 1, so \
                 every $M/$T number after them would be wrong. See \
                 docs/OBJ_GY_SHAPES.md §3.4.",
                f.mangled_name
            )));
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
            // One bit of the word is NOT a mode: `0x0100` says the function is a
            // constructor or a destructor ([`c2_il::OPT_WORD_SPECIAL_MEMBER`],
            // measured one flag and one function kind at a time). It is masked off
            // before the whole-word compare, so a destructor's word reads as the
            // mode it actually is — otherwise every constructor and destructor in
            // the corpus is a `codegen-gap` however ordinary its body, which is
            // what kept `A::~A() {}` (a bare `blr`, decoded as `EmptyBody`) out of
            // the emitter. Every other bit is still required to match a word this
            // port was verified against.
            let masked = w.map(|v| v & !c2_il::OPT_WORD_SPECIAL_MEMBER);
            let m = match &masked {
                Some(v) if *v == c2_il::OPT_WORD_OX => codegen::OptMode::Ox,
                Some(v) if *v == c2_il::OPT_WORD_O1 => codegen::OptMode::O1,
                other => {
                    // Reported as the RAW word, not the masked one: the census key
                    // has to name what is actually in the file.
                    return Err(BackendError::NotImplemented(format!(
                        "opt-mode {} at function {i}: only {:08x} (/Ox, /O2) and \
                         {:08x} (/O1) are implemented{}. See docs/OPT_MODE.md.",
                        match w {
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

        // Under function-level linking every function gets its own COMDAT
        // `.text` section, so the texts are kept separate rather than packed.
        if self.fn_level_linking {
            let mut texts: Vec<Vec<u8>> = Vec::with_capacity(funcs.len());
            let mut placed: Vec<coff::Function> = Vec::with_capacity(funcs.len());
            for f in &funcs {
                let mut frame: Option<coff::Frame> = None;
                // A framed non-leaf call gets its own `.text` COMDAT like any
                // other function, plus a `.pdata` COMDAT associated to it. The
                // `bl` is at the same offset as packed because the function
                // starts at 0 of its own section either way.
                let (text, call) = if let Some(fc) = &f.framed_call {
                    let t = codegen::framed_call_text(fc.add_k, 0);
                    frame = Some(coff::Frame {
                        prolog_len: codegen::FRAMED_PROLOG_LEN,
                        func_len: t.len() as u32,
                    });
                    (
                        t,
                        Some(coff::Call {
                            reloc_offset: codegen::FRAMED_BL_OFFSET,
                            callee: fc.callee.as_str(),
                        }),
                    )
                } else if let Some(callee) = &f.tail_call {
                    // Each function's text starts at offset 0 of its own COMDAT
                    // section, so the branch offset is just the setup's length.
                    let (t, reloc) = if let Some(sources) = &f.arg_sources {
                        let mut t = codegen::permute_args_text(sources)?;
                        let branch_off = t.len() as u32;
                        t.extend_from_slice(&codegen::encode_tail_branch(branch_off));
                        (t, branch_off)
                    } else if f.ops.is_empty() {
                        // A VOID tail call (`void f(){ g(); }`, and the generated
                        // empty destructor): no argument to compute, so the whole
                        // text is the branch at offset 0 of this COMDAT. The packed
                        // emitter below has had this case since the MVP; this branch
                        // did not, and fell into `int_tail_call_text`, which needs an
                        // operand stream and refused with "expression did not reduce
                        // to a single value" — so `mvp_call.cpp` has been a
                        // `codegen-gap` in the `/O1`, `/O2` and `/Ox /Gy` lanes for as
                        // long as those lanes have existed, for no reason but this
                        // missing arm. The workload compiles `/O1`, which implies
                        // `/Gy`, so it is the arm every real void tail call needs.
                        (codegen::encode_tail_branch(0).to_vec(), 0)
                    } else {
                        codegen::int_tail_call_text(f, 0, mode)?
                    };
                    (t, Some(coff::Call { reloc_offset: reloc, callee: callee.as_str() }))
                } else if f.empty_body {
                    (codegen::encode_blr().to_vec(), None)
                } else if let Some(t) = codegen::indirect_load_text(f) {
                    (t?, None)
                } else if let Some(t) = codegen::addr_leaf_text(f) {
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
                placed.push(coff::Function { name: &f.mangled_name, text_offset: 0, call, is_float: f.float_leaf.is_some(), fp_refs: Vec::new(), frame });
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
        for f in &funcs {
            while text.len() % 8 != 0 {
                text.push(0);
            }
            let off = text.len() as u32;
            // A framed non-leaf call: the fixed 0x24-byte frame, plus a `.pdata`
            // record and two `$M` labels. The `bl` reloc site is rebased onto
            // the shared `.text` like any other call.
            if let Some(fc) = &f.framed_call {
                let body = codegen::framed_call_text(fc.add_k, off);
                let frame = coff::Frame {
                    prolog_len: codegen::FRAMED_PROLOG_LEN,
                    func_len: body.len() as u32,
                };
                text.extend_from_slice(&body);
                placed.push(coff::Function {
                    name: &f.mangled_name,
                    text_offset: off,
                    call: Some(coff::Call {
                        reloc_offset: off + codegen::FRAMED_BL_OFFSET,
                        callee: &fc.callee,
                    }),
                    is_float: false,
                    fp_refs: Vec::new(),
                    frame: Some(frame),
                });
                continue;
            }
            // An empty body is a bare `blr` — no expression to select.
            if f.empty_body {
                text.extend_from_slice(&codegen::encode_blr());
                placed.push(coff::Function { name: &f.mangled_name, text_offset: off, call: None, is_float: f.float_leaf.is_some(), fp_refs: Vec::new(), frame: None });
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
                placed.push(coff::Function { name: &f.mangled_name, text_offset: off, call: None, is_float: true, fp_refs, frame: None });
                continue;
            }
            // An indirect-load leaf (`return *p;` / `return s->m;`) is a single
            // `lwz` + `blr`, recognized by an exact two-op stream rather than
            // reaching the affine selector — see `codegen::indirect_load_text`.
            if let Some(body) = codegen::indirect_load_text(f) {
                text.extend_from_slice(&body?);
                placed.push(coff::Function { name: &f.mangled_name, text_offset: off, call: None, is_float: false, fp_refs: Vec::new(), frame: None });
                continue;
            }
            // An address leaf (`return &s->m;`) is one `addi` + `blr`, or a bare
            // `blr` at offset 0 — the same exact-two-op recognition, one token
            // away from the load above and a different instruction.
            if let Some(body) = codegen::addr_leaf_text(f) {
                text.extend_from_slice(&body?);
                placed.push(coff::Function { name: &f.mangled_name, text_offset: off, call: None, is_float: false, fp_refs: Vec::new(), frame: None });
                continue;
            }
            // W6: a comparison leaf lowers to its own branchless spine rather
            // than through the operand-stack selector.
            if let Some(cmp) = &f.compare {
                text.extend_from_slice(&codegen::compare_leaf_text(cmp, mode)?);
                placed.push(coff::Function { name: &f.mangled_name, text_offset: off, call: None, is_float: f.float_leaf.is_some(), fp_refs: Vec::new(), frame: None });
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
                frame: None,
            });
        }

        let bytes = coff::emit_obj(obj_name, &placed, &text, label_counter);
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
