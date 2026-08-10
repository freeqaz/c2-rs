//! The `/EHsc` **scope-object** obj — a single function holding one destructible
//! local across a call, and the unwind funclet c2 emits for it.
//!
//! This is a **whole-TU emitter**, like [`super::emit_dyninit_obj`] and for the
//! same reason: the shape is not expressible as a `codegen::Selected`. Two facts
//! put it outside every per-function path in this crate.
//!
//! * **The function is TWO code regions in ONE `.text` COMDAT.** `main` occupies
//!   `entry .. entry+len`, and `__unwind$N` — a second entry point with its own
//!   prologue, its own `.pdata` record and its own `$M` pair — follows it.
//!   `Selected` is one-body-one-plan throughout and `coff::plan_labels` mints
//!   three labels for a framed function where this obj carries **ten**.
//! * **The function symbol's `Value` is not 0.** An 8-byte `{__CxxFrameHandler,
//!   __ehfuncinfo$<name>}` ADDR32 prefix sits at `.text+0`, so every consumer of
//!   *the function starts at 0* in `coff` is wrong here
//!   (`docs/EH_CRITICAL_PATH.md` §1).
//!
//! Everything below is a **transcription of one measured class**, in the sense
//! `codegen::ptr_walk_loop` and `codegen::if_call_join` are: it reproduces the
//! objs it was graded against and refuses everything else. The gate is in
//! [`c2_il::IlBundle::eh_scope_tu`]; this module's own `None` arms are the
//! second fence, so a widening of the reader cannot silently reach an untested
//! layout here.
//!
//! # Provenance
//!
//! `src/Main.cpp` of the dc3 workload at its own flags, plus the six probe cells
//! of `work/w-main2/probe/` (`m0`…`m5`, four distinct `.gl` label seeds). The
//! record layout is `docs/EH_RECORDS.md` and `docs/whitebox/WB_EH_FINDINGS.md`;
//! the label arithmetic is `work/w-main2/LABELS.md`.
//!
//! ```text
//!   .text   124 B, one COMDAT, 6 relocations, TWO code regions
//!   .pdata    8 B — the FUNCLET's record        (reverse region order)
//!   .pdata    8 B — the body's record, bit 31 SET
//!   .rdata   64 B — __unwindtable$ | __ehfuncinfo$ | the ip-to-state array
//! ```

use super::*;
use crate::codegen::encode::*;
use crate::codegen::frame::{
    FrameLayout, FRAME_LR_LOAD, FRAME_MFLR_R12, FRAME_MTLR_R12,
};

/// `.rdata` characteristics for the EH table group: CNT_INIT_DATA | COMDAT |
/// ALIGN_8 | MEM_READ. Numerically the same word as [`CH_PDATA_COMDAT`], and
/// deliberately spelled separately — they are two sections with two alignments
/// that happen to agree, and a lane that changes one must not silently change
/// the other.
pub(crate) const CH_RDATA_EH: u32 = 0x4040_1040;

/// `__ehfuncinfo$`'s leading magic. The only occurrence of this immediate in
/// `c2.dll` (board **#1869**).
pub(crate) const EH_MAGIC: u32 = 0x1993_0522;

/// `_s_FuncInfo` is **TEN** dwords, not the nine board #1869 and
/// `WB_EH_FINDINGS.md` §3.1 record.
///
/// The obj's own arithmetic forces it and it is not a matter of taste: the EH
/// `.rdata` is 64 bytes, `__unwindtable$main` occupies `+0x00..+0x08` and
/// `$T<n>` — the ip-to-state array, two 8-byte entries — starts at `+0x30`. The
/// record between them is `0x30 − 0x08 = 0x28` = 40 bytes = 10 dwords. The
/// tenth is zero on every cell measured here, which is exactly why counting the
/// *populated* fields gives nine.
pub(crate) const EH_FUNCINFO_DWORDS: u32 = 10;

/// One ip-to-state entry: `{ ip, state }`, both 4 bytes, the `ip` carrying an
/// ADDR32 relocation against a `$M` label.
pub(crate) const EH_IP2STATE_ENTRY: u32 = 8;

/// One `__unwindtable$` entry: `{ toState, action }`, the `action` carrying an
/// ADDR32 relocation against the `__unwind$N` funclet.
pub(crate) const EH_UNWIND_ENTRY: u32 = 8;

/// The `EHFlags` word `c2` writes at `__ehfuncinfo$ + 0x20` for a `/EHsc`
/// compilation. 1 on every cell measured.
const EH_FLAGS_EHS: u32 = 1;

/// The register the funclet reconstructs its frame pointer from. The unwinder
/// enters `__unwind$N` with the *establisher frame* in `r12`, which is why the
/// funclet's first instruction is `addi r31, r12, −F` and not a load.
const FUNCLET_ESTABLISHER_REG: u8 = 12;

/// The frame-pointer register a scope-object function establishes. `r31` is
/// also the one saved GPR, so the two facts are one register.
const FRAME_PTR_REG: u8 = 31;

/// `r3`, and the argument registers the class passes through.
const ARG0: u8 = 3;

/// The description of one `/EHsc` scope-object TU, as the reader hands it over.
///
/// Every field is read from the IL. Nothing here is a constant of the target
/// TU: the frame size comes from the object's own `sizeof`, the argument shuffle
/// from the formal list, and the label numbers from the `.gl` counter.
pub struct EhScopeTu<'a> {
    /// The function's COFF name — `main`. EXTERNAL, `Type` 0x0020, and its
    /// `Value` is the **entry offset**, not 0.
    pub name: &'a str,
    /// The scope object's constructor, called first with the object's address
    /// in `r3` and the function's own formals shifted up one register.
    pub ctor: &'a str,
    /// The member function called while the object is live. This is the call
    /// the ip-to-state map's state-0 entry points at.
    pub member: &'a str,
    /// The destructor. Called twice — once on the normal path and once from the
    /// funclet — and both `bl` words relocate against the same symbol.
    pub dtor: &'a str,
    /// `sizeof` the scope object, in bytes. Feeds [`FrameLayout::locals`], and
    /// through it the `stwu` immediate and the object's own address.
    pub object_size: u32,
    /// How many single-GPR formals the function declares, in declaration order.
    /// Each is moved up one register to make room for `this`, highest first.
    pub formals: u32,
    /// The `.gl` compiler-label counter ([`c2_il::label_counter`]).
    pub label_counter: u32,
}

/// The ten label numbers this class mints, derived from the `.gl` seed.
///
/// Measured in `LABEL_COUNTER.md` §7.6's **in-the-middle** form — every number
/// is an offset from the TU's own seed, never a difference between two
/// compilations, because c1xx and c2 share one symbol-id space and a
/// counterfactual measures Δseed + Δcharge (`wb-label`, board #2430–#2440).
///
/// `B = seed + LABEL_SEED_GAP + 3·nfuncs` is where [`plan_labels`] says this
/// function's own block begins, and with **one** function that is `seed + 12`.
/// Against `B`, over `m0`/`m1`/`m2`/`m3`/`m4`/`m5` and `src/Main.cpp`:
///
/// ```text
///   B+3, B+4   the ip-to-state $M pair
///   B+5        the ip-to-state $T
///   B+7 … B+9  the function's own triple      <=>  label_lead = 7
///   B+10 … +12 the funclet's triple
/// ```
///
/// The funclet symbol `__unwind$` is the **one** number that is not at a fixed
/// offset from `B`: it reads `B−2` when the EH function is the TU's first and
/// `B+0` when anything precedes it, and the six cells do not separate the two
/// readings. That is why [`c2_il::IlBundle::eh_scope_tu`] refuses a TU with more
/// than one `.ex` segment — under that gate only the `B−2` branch can ever fire.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EhLabels {
    /// `__unwind$N` — the funclet's own symbol.
    pub funclet: u32,
    /// The two `$M` the ip-to-state array relocates against.
    pub state: [u32; 2],
    /// The `$T` on the ip-to-state array itself.
    pub ip2state: u32,
    /// The body's `$M` (prologue end), `$M` (end) and `$T` (its `.pdata`).
    pub body: [u32; 3],
    /// The funclet's `$M`, `$M` and `$T`.
    pub funclet_triple: [u32; 3],
}

impl EhLabels {
    /// The allocation for a TU whose **only** function is this one.
    pub fn of_single_function_tu(seed: u32) -> EhLabels {
        let b = seed + LABEL_SEED_GAP + 3;
        EhLabels {
            funclet: b - 2,
            state: [b + 3, b + 4],
            ip2state: b + 5,
            body: [b + 7, b + 8, b + 9],
            funclet_triple: [b + 10, b + 11, b + 12],
        }
    }
}

/// Where each piece of the emitted `.text` sits. Everything downstream — the
/// label values, the `.pdata` records, the relocation offsets — is read off
/// this rather than recomputed, because the two `.pdata` records and the six
/// label symbols are four independent copies of the same arithmetic and this
/// file's sibling (`writer.rs`) already carries the scar of letting a count live
/// in three places.
struct TextPlan {
    text: Vec<u8>,
    /// `.text` offset of the function's first instruction — the width of the
    /// `{__CxxFrameHandler, __ehfuncinfo$}` prefix. This is the function
    /// symbol's `Value`.
    entry: u32,
    /// Body prologue end (`$M`), body end (`$M`).
    body_prolog_end: u32,
    body_end: u32,
    /// The three `bl` sites in the body, in call order.
    call_sites: [u32; 3],
    /// Funclet start (`__unwind$`'s `Value`), its prologue end and its end.
    funclet: u32,
    funclet_prolog_end: u32,
    funclet_end: u32,
    /// The funclet's single `bl` site.
    funclet_call_site: u32,
}

/// The ADDR32 prefix c2 puts in front of an EH function's first instruction:
/// the language handler and this function's `__ehfuncinfo$`, both patched
/// entirely by relocation, so the raw bytes are zero.
const HANDLER_PREFIX_LEN: u32 = 8;

fn plan_text(tu: &EhScopeTu) -> Option<TextPlan> {
    // The body's frame: the object is the only addressed local, `r31` is the
    // only saved GPR, and the outgoing-parameter area is the ABI floor — the
    // widest call this class makes passes three GPRs, well under eight.
    // `wb-frame`'s `align16(80 + locals + 8 + 8·saved)` is unmodified here.
    let body_frame = FrameLayout {
        locals: tu.object_size,
        out_slots: 0,
        saved_gprs: 1,
        saved_fprs: 0,
    };
    if body_frame.out_of_class_ctx().is_some() {
        return None;
    }
    // The funclet's frame: no locals, no saved GPRs — it re-derives `r31` from
    // the establisher frame in `r12` instead of restoring it — and one call.
    let funclet_frame = FrameLayout::default();
    if funclet_frame.out_of_class_ctx().is_some() {
        return None;
    }
    let f = i16::try_from(body_frame.size()).ok()?;
    let g = i16::try_from(funclet_frame.size()).ok()?;
    let obj_off = i16::try_from(body_frame.locals_base()).ok()?;
    // `stw r12,−8(r1)` / `std r31,−16(r1)` are written against the CALLER's
    // `r1`, so both displacements are negative and independent of `F`.
    let lr_slot: i16 = -8;
    let gpr_slot: i16 = -16;

    let mut t: Vec<u8> = Vec::with_capacity(128);
    t.extend_from_slice(&[0u8; HANDLER_PREFIX_LEN as usize]);
    let entry = HANDLER_PREFIX_LEN;

    // ---- the body prologue: the Class B one plus the frame-pointer word ----
    //
    // `FrameLayout::prologue` emits `mflr r12 / stw r12,−8(r1) / std r31,−16(r1)
    // / stwu r1,−F(r1)`. This class inserts `addi r31,r1,−F` before the `stwu`,
    // because the object is addressed off `r31` and `r31` has to name the frame
    // *before* `r1` moves. Transcribed here rather than added to `prologue()`:
    // that method is the gate every shipped emitter runs and widening it would
    // change the bytes of every framed class at once.
    t.extend_from_slice(&FRAME_MFLR_R12.to_be_bytes());
    t.extend_from_slice(&encode_stw(12, 1, lr_slot));
    t.extend_from_slice(&encode_std(FRAME_PTR_REG, 1, gpr_slot));
    t.extend_from_slice(&encode_addi(FRAME_PTR_REG, 1, -f));
    t.extend_from_slice(&encode_stwu(1, 1, -f));
    let body_prolog_end = t.len() as u32;

    // ---- the argument shuffle -------------------------------------------
    //
    // The constructor takes `this` plus every one of the function's own
    // formals, so each formal moves up one register. HIGHEST FIRST, which is
    // what makes the sequence non-destructive without a temporary: `mr r5,r4`
    // then `mr r4,r3`. `docs/CODEGEN_ARG_PERM.md` is the general rule; this is
    // its one-place-shift instance.
    if tu.formals == 0 || tu.formals > 7 {
        return None;
    }
    for i in (0..tu.formals as u8).rev() {
        t.extend_from_slice(&encode_mr(ARG0 + i + 1, ARG0 + i));
    }

    // ---- ctor(this, …), member(this), dtor(this) ------------------------
    let mut call_sites = [0u32; 3];
    for (k, site) in call_sites.iter_mut().enumerate() {
        t.extend_from_slice(&encode_addi(ARG0, FRAME_PTR_REG, obj_off));
        *site = t.len() as u32;
        t.extend_from_slice(&crate::codegen::calls::encode_call_branch(*site));
        let _ = k;
    }

    // `return 0` — the implicit one `main` gets, spelled `li r3,0`.
    t.extend_from_slice(&encode_addi(ARG0, 0, 0));

    // ---- the body epilogue ----------------------------------------------
    //
    // `addi r1,r31,F` and not `addi r1,r1,F`: the stack pointer is restored
    // from the frame pointer, which is the epilogue half of the `addi r31`
    // above.
    t.extend_from_slice(&encode_addi(1, FRAME_PTR_REG, f));
    t.extend_from_slice(&FRAME_LR_LOAD.to_be_bytes());
    t.extend_from_slice(&FRAME_MTLR_R12.to_be_bytes());
    t.extend_from_slice(&encode_ldr(FRAME_PTR_REG, 1, gpr_slot));
    t.extend_from_slice(&encode_blr());
    let body_end = t.len() as u32;

    // ---- the funclet ------------------------------------------------------
    let funclet = body_end;
    t.extend_from_slice(&encode_addi(FRAME_PTR_REG, FUNCLET_ESTABLISHER_REG, -f));
    t.extend_from_slice(&FRAME_MFLR_R12.to_be_bytes());
    t.extend_from_slice(&encode_stw(12, 1, lr_slot));
    t.extend_from_slice(&encode_stwu(1, 1, -g));
    let funclet_prolog_end = t.len() as u32;
    t.extend_from_slice(&encode_addi(ARG0, FRAME_PTR_REG, obj_off));
    let funclet_call_site = t.len() as u32;
    t.extend_from_slice(&crate::codegen::calls::encode_call_branch(funclet_call_site));
    t.extend_from_slice(&encode_addi(1, 1, g));
    t.extend_from_slice(&FRAME_LR_LOAD.to_be_bytes());
    t.extend_from_slice(&FRAME_MTLR_R12.to_be_bytes());
    t.extend_from_slice(&encode_blr());
    let funclet_end = t.len() as u32;

    Some(TextPlan {
        text: t,
        entry,
        body_prolog_end,
        body_end,
        call_sites,
        funclet,
        funclet_prolog_end,
        funclet_end,
        funclet_call_site,
    })
}

/// The unwind word's bit 31 — *this region has a language handler*.
///
/// `wb-eh` established (board **#1862**) that bit 31 and the
/// `{__CxxFrameHandler, __ehfuncinfo$}` prefix are **one predicate**, not two
/// facts: the same two handler arguments that decide the prefix are masked to
/// zero inside a `'T'`-opened region. So the body's record carries it and the
/// funclet's does not, and the two rivals — *"has a language handler"* and
/// *"has a prologue"* — are both refuted on that one cell (the funclet's prolog
/// field is 4).
pub(crate) const UNWIND_HAS_HANDLER: u32 = 0x8000_0000;

/// The 8-byte X360 `RUNTIME_FUNCTION` with the EH flag as an argument.
///
/// [`pdata_record`] is this with `has_handler = false`, kept as its own name so
/// no existing caller changes and so the one place that sets bit 31 is visible.
pub(crate) fn pdata_record_eh(begin_addend: u32, frame: &Frame, has_handler: bool) -> [u8; 8] {
    debug_assert_eq!(frame.func_len % 4, 0, "function length is a word multiple");
    debug_assert_eq!(frame.prolog_len % 4, 0, "prologue length is a word multiple");
    let mut unwind =
        UNWIND_THIRTY_TWO_BIT | ((frame.func_len / 4) << 8) | (frame.prolog_len / 4);
    if has_handler {
        unwind |= UNWIND_HAS_HANDLER;
    }
    let mut r = [0u8; 8];
    r[..4].copy_from_slice(&begin_addend.to_be_bytes());
    r[4..].copy_from_slice(&unwind.to_be_bytes());
    r
}

/// Build the 8-section `/EHsc` scope-object obj, or `None` for anything outside
/// the measured class.
pub fn emit_eh_scope_obj(obj_name: &str, tu: &EhScopeTu<'_>) -> Option<Vec<u8>> {
    let plan = plan_text(tu)?;
    let labels = EhLabels::of_single_function_tu(tu.label_counter);

    // ---- the two `.pdata` records ---------------------------------------
    //
    // The FUNCLET's first. `wb-eh` read the region cut out of `c2.dll` itself
    // (`.text` is split at every `__catch$`/`__unwind$` label) and the records
    // come out in **reverse** `.text` region order.
    let body_rec = pdata_record_eh(
        0,
        &Frame {
            prolog_len: plan.body_prolog_end - plan.entry,
            func_len: plan.body_end - plan.entry,
        },
        true,
    );
    let funclet_rec = pdata_record_eh(
        plan.funclet - plan.entry,
        &Frame {
            prolog_len: plan.funclet_prolog_end - plan.funclet,
            func_len: plan.funclet_end - plan.funclet,
        },
        false,
    );

    // ---- the EH `.rdata` -------------------------------------------------
    //
    //   +0x00  __unwindtable$<name>   one entry: { toState = −1, action }
    //   +0x08  __ehfuncinfo$<name>    ten dwords, magic first
    //   +0x30  $T<n>                  two ip-to-state entries
    //
    // `maxState` is 1: exactly one object is destructible and exactly one call
    // happens while it is live. Every address is a relocation, so every field
    // that holds one is written as zero.
    let unwind_table_at = 0u32;
    let funcinfo_at = unwind_table_at + EH_UNWIND_ENTRY;
    let ip2state_at = funcinfo_at + 4 * EH_FUNCINFO_DWORDS;
    let n_ip2state = 2u32;
    let mut rdata: Vec<u8> = Vec::with_capacity((ip2state_at + n_ip2state * EH_IP2STATE_ENTRY) as usize);
    let mut w = |v: u32| rdata.extend_from_slice(&v.to_be_bytes());
    // __unwindtable$: the one unwind action, leaving state −1 behind it.
    w(u32::MAX);
    w(0); // -> __unwind$N   (ADDR32)
    // __ehfuncinfo$: magic, maxState, pUnwindMap, nTryBlocks, pTryBlockMap,
    // nIPMapEntries, pIPtoStateMap, dispUnwindHelp, pESTypeList, EHFlags.
    w(EH_MAGIC);
    w(1);
    w(0); // -> __unwindtable$<name>   (ADDR32)
    w(0);
    w(0);
    w(n_ip2state);
    w(0); // -> $T<n>   (ADDR32)
    w(0);
    w(EH_FLAGS_EHS);
    w(0);
    // The ip-to-state array. Entry 0 opens state 0 at the member call; entry 1
    // closes it at the destructor call. Both `ip` fields are relocations.
    w(0); // -> $M(state 0)
    w(0);
    w(0); // -> $M(state −1)
    w(u32::MAX);
    debug_assert_eq!(rdata.len() as u32, ip2state_at + n_ip2state * EH_IP2STATE_ENTRY);

    // ---- sections --------------------------------------------------------
    let mut sections = shell_sections(obj_name);
    let sec_text = sections.len();
    sections.push(Section {
        name: ".text",
        characteristics: CH_TEXT_COMDAT,
        raw: std::borrow::Cow::Borrowed(&plan.text),
        checksum: 0,
        selection: COMDAT_SELECT_NODUPLICATES,
        assoc: 0,
        uninit_size: None,
    });
    let text_sec_num = (sec_text + 1) as u16;
    let sec_pdata_funclet = sections.len();
    sections.push(Section {
        name: ".pdata",
        characteristics: CH_PDATA_COMDAT,
        raw: std::borrow::Cow::Borrowed(&funclet_rec[..]),
        checksum: coff_checksum(&funclet_rec[..]),
        selection: COMDAT_SELECT_ASSOCIATIVE,
        assoc: text_sec_num,
        uninit_size: None,
    });
    let sec_pdata_body = sections.len();
    sections.push(Section {
        name: ".pdata",
        characteristics: CH_PDATA_COMDAT,
        raw: std::borrow::Cow::Borrowed(&body_rec[..]),
        checksum: coff_checksum(&body_rec[..]),
        selection: COMDAT_SELECT_ASSOCIATIVE,
        assoc: text_sec_num,
        uninit_size: None,
    });
    let sec_rdata = sections.len();
    sections.push(Section {
        name: ".rdata",
        characteristics: CH_RDATA_EH,
        raw: std::borrow::Cow::Borrowed(&rdata),
        checksum: coff_checksum(&rdata),
        selection: COMDAT_SELECT_ASSOCIATIVE,
        assoc: text_sec_num,
        uninit_size: None,
    });
    let n_sections = sections.len();

    // ---- symbol indices --------------------------------------------------
    //
    // The symbol table follows section order, and inside the `.text` group it
    // follows **descending `.text` address**: the function itself first, then
    // every label and every undefined external, an external keyed on its lowest
    // relocation site and placed ahead of the `$M` at the same address.
    // Transcribed from the reference objs; `docs/OBJ_DYNINIT_SHAPE.md` §3.1's
    // rule — *"then any undefined external first referenced by that section"* —
    // does **not** hold here, and `__CxxFrameHandler` is the counterexample: it
    // is referenced by `.text` and sits after both `.pdata` groups.
    let mut next = N_SHELL_SYMBOLS;
    next += 2; // .text section symbol + aux
    let i_func = next;
    let i_m_funclet_end = next + 1;
    let i_m_funclet_prolog = next + 2;
    let i_unwind = next + 3;
    let i_m_body_end = next + 4;
    let i_dtor = next + 5;
    let i_m_state1 = next + 6;
    let i_member = next + 7;
    let i_m_state0 = next + 8;
    let i_ctor = next + 9;
    let i_m_body_prolog = next + 10;
    next += 11;
    next += 2; // .pdata (funclet) section symbol + aux
    let i_t_funclet = next;
    next += 1;
    next += 2; // .pdata (body) section symbol + aux
    let i_t_body = next;
    next += 1;
    let i_handler = next;
    next += 1;
    next += 2; // .rdata section symbol + aux
    let i_funcinfo = next;
    let i_unwindtable = next + 1;
    let i_t_ip2state = next + 2;
    next += 3;
    let n_symbols = next;

    // ---- relocations -----------------------------------------------------
    let text_relocs: Vec<(u32, u32, u16)> = vec![
        (0, i_handler, REL_PPC_ADDR32),
        (4, i_funcinfo, REL_PPC_ADDR32),
        (plan.call_sites[0], i_ctor, REL_PPC_REL24),
        (plan.call_sites[1], i_member, REL_PPC_REL24),
        (plan.call_sites[2], i_dtor, REL_PPC_REL24),
        (plan.funclet_call_site, i_dtor, REL_PPC_REL24),
    ];
    // Both `.pdata` `BeginAddress` fields relocate against the FUNCTION symbol,
    // whose `Value` is the entry offset — which is why the funclet's record
    // carries `funclet − entry` and not `funclet`.
    let pdata_relocs: Vec<(u32, u32, u16)> = vec![(0, i_func, REL_PPC_ADDR32)];
    let rdata_relocs: Vec<(u32, u32, u16)> = vec![
        (unwind_table_at + 4, i_unwind, REL_PPC_ADDR32),
        (funcinfo_at + 8, i_unwindtable, REL_PPC_ADDR32),
        (funcinfo_at + 24, i_t_ip2state, REL_PPC_ADDR32),
        (ip2state_at, i_m_state0, REL_PPC_ADDR32),
        (ip2state_at + EH_IP2STATE_ENTRY, i_m_state1, REL_PPC_ADDR32),
    ];

    let mut relocs: Vec<Vec<(u32, u32, u16)>> = vec![Vec::new(); n_sections];
    relocs[sec_text] = text_relocs;
    relocs[sec_pdata_funclet] = pdata_relocs.clone();
    relocs[sec_pdata_body] = pdata_relocs;
    relocs[sec_rdata] = rdata_relocs;
    let n_reloc_of: Vec<u16> = relocs.iter().map(|r| r.len() as u16).collect();

    let (ptrs, reloc_ptr, ptr_symtab) = layout_sections(&sections, &n_reloc_of);

    let mut b = Buf::with_capacity(ptr_symtab + n_symbols as usize * SYMBOL_LEN + 512);
    write_coff_header(&mut b, n_sections, ptr_symtab, n_symbols);
    write_section_headers(&mut b, &sections, &ptrs, &reloc_ptr, &n_reloc_of);
    for (i, s) in sections.iter().enumerate() {
        debug_assert_eq!(b.0.len(), ptrs[i]);
        b.bytes(&s.raw);
        if !relocs[i].is_empty() {
            debug_assert_eq!(b.0.len(), reloc_ptr[i].unwrap());
            for &(va, sym, ty) in &relocs[i] {
                b.u32(va);
                b.u32(sym);
                b.u16(ty);
            }
        }
    }
    debug_assert_eq!(b.0.len(), ptr_symtab);

    // ---- the symbol table ------------------------------------------------
    let mut strtab = StringTable::new();
    emit_shell_symbols(&mut b, &mut strtab, &sections);

    let text_num = text_sec_num as i16;
    emit_section_symbol(&mut b, &sections[sec_text], text_num, n_reloc_of[sec_text]);
    // The function: EXTERNAL, FUNCTION type, and `Value` = the entry offset.
    emit_function_symbol(&mut b, &mut strtab, tu.name, text_num, plan.entry);
    let lab = |b: &mut Buf, st: &mut StringTable, prefix: char, n: u32, value: u32| {
        emit_label_symbol(b, &label_name(prefix, n), value, text_num);
        let _ = st;
    };
    lab(&mut b, &mut strtab, 'M', labels.funclet_triple[1], plan.funclet_end);
    lab(&mut b, &mut strtab, 'M', labels.funclet_triple[0], plan.funclet_prolog_end);
    // `__unwind$N` — STATIC with FUNCTION type, the second entry point.
    emit_symbol(
        &mut b,
        &mut strtab,
        &format!("__unwind${}", labels.funclet),
        plan.funclet,
        text_num,
        0x0020,
        3,
    );
    lab(&mut b, &mut strtab, 'M', labels.body[1], plan.body_end);
    emit_function_symbol(&mut b, &mut strtab, tu.dtor, 0, 0);
    lab(&mut b, &mut strtab, 'M', labels.state[1], plan.call_sites[2]);
    emit_function_symbol(&mut b, &mut strtab, tu.member, 0, 0);
    lab(&mut b, &mut strtab, 'M', labels.state[0], plan.call_sites[1]);
    emit_function_symbol(&mut b, &mut strtab, tu.ctor, 0, 0);
    lab(&mut b, &mut strtab, 'M', labels.body[0], plan.body_prolog_end);

    emit_section_symbol(
        &mut b,
        &sections[sec_pdata_funclet],
        (sec_pdata_funclet + 1) as i16,
        n_reloc_of[sec_pdata_funclet],
    );
    emit_pdata_label_symbol(
        &mut b,
        &label_name('T', labels.funclet_triple[2]),
        0,
        (sec_pdata_funclet + 1) as i16,
    );
    emit_section_symbol(
        &mut b,
        &sections[sec_pdata_body],
        (sec_pdata_body + 1) as i16,
        n_reloc_of[sec_pdata_body],
    );
    emit_pdata_label_symbol(
        &mut b,
        &label_name('T', labels.body[2]),
        0,
        (sec_pdata_body + 1) as i16,
    );
    // `__CxxFrameHandler` — the language handler, an undefined external c2
    // mints for every EH function. It is referenced by `.text` and emitted
    // here, which is the one place `emit_dyninit_obj`'s ordering rule does not
    // predict.
    emit_function_symbol(&mut b, &mut strtab, "__CxxFrameHandler", 0, 0);

    let rdata_num = (sec_rdata + 1) as i16;
    emit_section_symbol(&mut b, &sections[sec_rdata], rdata_num, n_reloc_of[sec_rdata]);
    emit_symbol(&mut b, &mut strtab, &format!("__ehfuncinfo${}", tu.name), funcinfo_at, rdata_num, 0, 3);
    emit_symbol(&mut b, &mut strtab, &format!("__unwindtable${}", tu.name), unwind_table_at, rdata_num, 0, 3);
    emit_symbol(&mut b, &mut strtab, &label_name('T', labels.ip2state), ip2state_at, rdata_num, 0, 3);

    b.bytes(&strtab.finish());
    debug_assert_eq!(i_t_ip2state + 1, n_symbols);
    let _ = (i_m_funclet_end, i_m_funclet_prolog, i_m_body_end, i_m_body_prolog, i_t_funclet, i_t_body);
    Some(b.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn main_tu() -> EhScopeTu<'static> {
        EhScopeTu {
            name: "main",
            ctor: "??0App@@QAA@HPAPAD@Z",
            member: "?Run@App@@QAAXXZ",
            dtor: "??1App@@QAA@XZ",
            object_size: 4,
            formals: 2,
            label_counter: 2575,
        }
    }

    /// The 124 `.text` bytes of `src/Main.cpp`'s obj, transcribed from
    /// `python3 scripts/gt_dump.py work/w-main2/ref/Main.obj`. Written out as
    /// words with their disassembly so a diff names the instruction.
    const MAIN_TEXT: [u32; 31] = [
        0x0000_0000, // -> __CxxFrameHandler
        0x0000_0000, // -> __ehfuncinfo$main
        0x7d88_02a6, // mflr 12
        0x9181_fff8, // stw 12,-8(1)
        0xfbe1_fff0, // std 31,-16(1)
        0x3be1_ff90, // addi 31,1,-112
        0x9421_ff90, // stwu 1,-112(1)          $M(prolog end) = 0x1c
        0x7c85_2378, // mr 5,4
        0x7c64_1b78, // mr 4,3
        0x387f_0050, // addi 3,31,80
        0x4bff_ffd9, // bl -> ??0App
        0x387f_0050, // addi 3,31,80
        0x4bff_ffd1, // bl -> ?Run              $M(state 0) = 0x30
        0x387f_0050, // addi 3,31,80
        0x4bff_ffc9, // bl -> ??1App            $M(state -1) = 0x38
        0x3860_0000, // li 3,0
        0x383f_0070, // addi 1,31,112
        0x8181_fff8, // lwz 12,-8(1)
        0x7d88_03a6, // mtlr 12
        0xebe1_fff0, // ld 31,-16(1)
        0x4e80_0020, // blr                      $M(end) = 0x54
        0x3bec_ff90, // addi 31,12,-112          __unwind$ = 0x54
        0x7d88_02a6, // mflr 12
        0x9181_fff8, // stw 12,-8(1)
        0x9421_ffa0, // stwu 1,-96(1)            $M(funclet prolog) = 0x64
        0x387f_0050, // addi 3,31,80
        0x4bff_ff99, // bl -> ??1App
        0x3821_0060, // addi 1,1,96
        0x8181_fff8, // lwz 12,-8(1)
        0x7d88_03a6, // mtlr 12
        0x4e80_0020, // blr                      $M(funclet end) = 0x7c
    ];

    #[test]
    fn the_text_is_byte_identical_to_the_reference_obj() {
        let plan = plan_text(&main_tu()).expect("in class");
        let want: Vec<u8> = MAIN_TEXT.iter().flat_map(|w| w.to_be_bytes()).collect();
        assert_eq!(plan.text.len(), 124, "two regions, 31 words");
        for (i, (g, w)) in plan.text.chunks(4).zip(want.chunks(4)).enumerate() {
            assert_eq!(g, w, "word {} at .text+0x{:x}", i, i * 4);
        }
    }

    #[test]
    fn the_layout_matches_the_reference_offsets() {
        let p = plan_text(&main_tu()).expect("in class");
        assert_eq!(p.entry, 0x08);
        assert_eq!(p.body_prolog_end, 0x1c);
        assert_eq!(p.call_sites, [0x28, 0x30, 0x38]);
        assert_eq!(p.body_end, 0x54);
        assert_eq!(p.funclet, 0x54);
        assert_eq!(p.funclet_prolog_end, 0x64);
        assert_eq!(p.funclet_call_site, 0x68);
        assert_eq!(p.funclet_end, 0x7c);
    }

    /// The two records, including the one field the incumbent `pdata_record`
    /// cannot spell: bit 31, and a non-zero `BeginAddress` addend.
    #[test]
    fn the_two_pdata_records_reproduce() {
        let p = plan_text(&main_tu()).expect("in class");
        let body = pdata_record_eh(
            0,
            &Frame { prolog_len: p.body_prolog_end - p.entry, func_len: p.body_end - p.entry },
            true,
        );
        let funclet = pdata_record_eh(
            p.funclet - p.entry,
            &Frame {
                prolog_len: p.funclet_prolog_end - p.funclet,
                func_len: p.funclet_end - p.funclet,
            },
            false,
        );
        assert_eq!(body, [0x00, 0x00, 0x00, 0x00, 0xc0, 0x00, 0x13, 0x05]);
        assert_eq!(funclet, [0x00, 0x00, 0x00, 0x4c, 0x40, 0x00, 0x0a, 0x04]);
    }

    /// `pdata_record_eh(_, _, false)` must be the incumbent record, so the two
    /// spellings cannot drift.
    #[test]
    fn the_handler_free_record_is_the_incumbent_one() {
        let f = Frame { prolog_len: 20, func_len: 76 };
        assert_eq!(pdata_record_eh(0, &f, false), pdata_record(0, &f));
        assert_eq!(pdata_record_eh(0x4c, &f, false)[..4], [0, 0, 0, 0x4c]);
    }

    /// Every one of the ten labels, against `src/Main.cpp`'s own obj.
    #[test]
    fn the_ten_labels_reproduce_at_main_cpps_seed() {
        let l = EhLabels::of_single_function_tu(2575);
        assert_eq!(l.funclet, 2585);
        assert_eq!(l.state, [2590, 2591]);
        assert_eq!(l.ip2state, 2592);
        assert_eq!(l.body, [2594, 2595, 2596]);
        assert_eq!(l.funclet_triple, [2597, 2598, 2599]);
    }

    /// …and against the two probe cells that move ONLY the seed. `m0` reads
    /// 2551 and `m1` reads 2554; both are in `work/w-main2/LABELS.md`.
    #[test]
    fn the_label_model_is_affine_in_the_seed() {
        let a = EhLabels::of_single_function_tu(2551);
        assert_eq!(a.funclet, 2561);
        assert_eq!(a.state, [2566, 2567]);
        assert_eq!(a.ip2state, 2568);
        assert_eq!(a.body, [2570, 2571, 2572]);
        assert_eq!(a.funclet_triple, [2573, 2574, 2575]);
        let b = EhLabels::of_single_function_tu(2554);
        assert_eq!(b.funclet, 2564);
        assert_eq!(b.body, [2573, 2574, 2575]);
    }

    /// The EH `.rdata` is 64 bytes exactly, and it is that only because
    /// `__ehfuncinfo$` is TEN dwords. Nine would put `$T` at `+0x2c` and the
    /// section at 60 bytes; board #1869 and `WB_EH_FINDINGS.md` §3.1 both say
    /// nine.
    #[test]
    fn the_eh_rdata_is_sixty_four_bytes_and_the_funcinfo_is_ten_dwords() {
        assert_eq!(EH_UNWIND_ENTRY + 4 * EH_FUNCINFO_DWORDS + 2 * EH_IP2STATE_ENTRY, 64);
        assert_eq!(EH_UNWIND_ENTRY + 4 * EH_FUNCINFO_DWORDS, 0x30);
    }

    /// A formal count outside the measured shift refuses rather than emitting a
    /// shuffle nothing graded.
    #[test]
    fn an_unmeasured_formal_count_refuses() {
        let mut tu = main_tu();
        tu.formals = 0;
        assert!(plan_text(&tu).is_none());
        tu.formals = 8;
        assert!(plan_text(&tu).is_none());
    }
}
