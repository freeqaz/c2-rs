//! **The per-op value, carrying c2's own opcode number — and the one general
//! composition that turns it into a word.**
//!
//! This is Phase 0 slice **S1**'s general layer
//! (`docs/ROADMAP_SLICING_2026-08-21.md` §5). Before it,
//! [`super::encode`] was 85 hand-written functions, each holding its own copy
//! of a primary opcode and an extended opcode as literals — 85 independent
//! black-box re-derivations of two tables c2 states plainly. After it, every
//! one of those functions is a **[`MachineOp`] constructor** and there is
//! exactly one place that composes a word.
//!
//! # What is READ and what is not
//!
//! **Read-before-probe** (`docs/WHITEBOX_LEVERAGE_2026-08-21.md`): nothing here
//! is fitted. The base words in [`OPCODES`] and the form numbers beside them are
//! transcribed from `docs/whitebox/ref/ENCODE_OPCODES.txt`, dumped from the
//! pinned image by `docs/whitebox/scripts/dump_opcode_tables.py` (base-word
//! table `0x10c3a578`, encode-form table `0x10c39b18`, arm jump table
//! `0x10bfae2d`). The field placements in [`place`] are transcribed from
//! `docs/whitebox/ref/P_ENCODE.md` §5, read arm by arm — 79/79 — by lane
//! `w-read-r2`.
//!
//! **`docs/whitebox/DISCLOSURE.md` carries the provenance rows — and it did
//! not, for four days, while this sentence said it did.** The hole was found
//! by `scripts/provenance_census.py` on its first real run (board **#3632**),
//! reported and deliberately not repaired by the lane that found it, and
//! filed by lane `w-disclose` as three rows (**#3642**): **`W-MOP-1`** the 85
//! opcode numbers and the table extent, **`W-MOP-2`** the 85 transcribed rows
//! of `0x10c3a578` / `0x10c39b18` / `0x10b1b260`, **`W-MOP-3`** the field
//! placements. They are the ledger's **first rows whose `Adopted into` is on
//! the emit path**; every `W-MID-*` and `W-STAGETAP-*` row above them is
//! instrument-only by its own text. `W-MID-1` and `W-MID-2` still hold — they
//! adopt these tables' *addresses, strides and index origin* into
//! `c2-reference/tests/middle_interfaces.rs`, and both say *"no table entry is
//! copied"*, which stays true there and is false here. That is the difference
//! the three new rows exist to record.
//!
//! **`P_ENCODE.md` §9's bound is respected and is the reason this module stops
//! where it does.** That spec is a total function *from a finished machine
//! tuple*, and **building the tuple is read R5, which is unstarted**. So the
//! slots below are the **port's own** operands, named for c2's slots because
//! that is what the form rules read — they are **not** a claim to reproduce
//! c2's tuple construction. A general lowering of the tuple stream is a
//! different lane.
//!
//! # Why a table and a composition, rather than 85 literals
//!
//! Board **#3379** measured the two derivations against each other: the port's
//! 89 accumulated words, evaluated at every operand zero, agree with c2's
//! `base_word[op]` on **82 of 89**, with **zero** disagreements in a primary
//! opcode or an extended opcode, and all seven residuals a **field the port
//! bakes that c2's arm supplies**. This module keeps that measurement live
//! rather than as a one-off: [`base_word`] is now the port's *only* source of a
//! primary opcode, so the two derivations can no longer drift apart silently,
//! and the seven residuals are visible as exactly what they are — ops whose
//! constructor passes a constant into a slot.
//!
//! # The decision surface
//!
//! `docs/GOAL_DECISION_2026-08-21.md` § AMENDED and `docs/rungs/README.md`'s
//! DECISION-SURFACE CLAUSE require a general layer to ship its arbitrary
//! choices as **named, enumerable parameters whose default reproduces c2
//! byte-exactly**, not as baked constants. Here that is [`EncodeParams`]: the
//! base-word table and the per-form [`FieldPlan`] are data, and
//! [`EncodeParams::C2`] is the default that every emit uses.
//!
//! **Grading is at the default and nowhere else.** A non-default
//! [`EncodeParams`] is a legal *instrument* state and licenses no emit — it is
//! what a permuter searches and what a mutation control perturbs. Board
//! **#3379**'s four hand-edited mask mutations (`D` field 16→12 bits, `RB` 5→4,
//! drop `RA`, `SPR` unsplit) are all expressible as [`FieldPlan`] values now;
//! see [`EncodeParams::with_field_width`].

use crate::BackendError;

// ---- c2's opcode number ----------------------------------------------------

/// **c2's own opcode number** — the index into the base-word table at
/// `0x10c3a578`, as dumped to `docs/whitebox/ref/ENCODE_OPCODES.txt`.
///
/// A newtype rather than a bare `u16` because the number space is easy to
/// confuse with three neighbours the same file already carries: the **form**
/// number (`0..=108`), the PPC **primary** opcode (`0..=63`) and the PPC
/// **extended** opcode (`0..=1023`). `add` is opcode `0x0001`, form `49`,
/// primary `31`, extended `266`; four different small integers for one
/// instruction, and only the first indexes the table.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct C2Op(pub u16);

/// c2's **encode-form** number — the index into the form table at
/// `0x10c39b18`, which selects the arm that places the operand fields.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Form(pub u16);

/// One row of c2's base-word table, plus the form that composes it.
///
/// `mnemonic` is carried for diagnostics and for the agreement instrument; it
/// is c2's own spelling out of the stride-12 mnemonic table, not the port's.
#[derive(Clone, Copy, Debug)]
pub struct OpRow {
    pub op: C2Op,
    pub mnemonic: &'static str,
    /// The word before any operand field is placed. **READ**, never derived.
    pub base: u32,
    pub form: Form,
}

/// Named opcode numbers for every instruction this port emits.
///
/// The numbers are c2's, read from the pinned image. They are grouped by the
/// form that consumes them, because the form — not the mnemonic — is what
/// decides where a register lands, and form 39's placement is (per
/// `P_ENCODE.md` §5.1) *"the single most safety-critical fact on this page."*
pub mod op {
    //! PROV-BLOCK[R] DISCLOSURE `W-MOP-1` — every opcode NUMBER below is c2's
    //! own **0-based index** into the mnemonic table `0x10b1b260`, which is the
    //! same index the base-word table `0x10c3a578` and the encode-form table
    //! `0x10c39b18` are read with. Transcribed from
    //! `docs/whitebox/ref/ENCODE_OPCODES.txt`, which
    //! `docs/whitebox/scripts/dump_opcode_tables.py` dumps from the pinned image
    //! (sha256 `c80981c0…a66258`). Lane `w-read-r2`, read **R2**.
    //!
    //! **`W-MID-1` is the neighbouring row and is NOT this one.** It adopts the
    //! table's *address, stride, index origin and `_last` sentinel* into
    //! `c2-reference/tests/middle_interfaces.rs`, and says in its own text *"no
    //! table entry is copied"* — true there, and false here: the 85 constants
    //! below are 85 of the table's positions, written out as literals, on the
    //! **emit path** (`base_word` is the port's only source of a primary
    //! opcode). The row that says so is `W-MOP-1`, filed by lane `w-disclose`
    //! (board **#3642**) after `scripts/provenance_census.py` found the hole
    //! (**#3632**).
    //!
    //! **Verified against the image, not against the transcription**:
    //! `work/w-disclose/verify_rows.py` check D re-dumps `c2.dll` live and
    //! agrees with all 85 rows on mnemonic, base word and form.
    use super::C2Op;

    // form 49 — XO with RT/RA/RB
    pub const ADD: C2Op = C2Op(0x0001);
    pub const ADDE: C2Op = C2Op(0x0007);
    pub const SUBF: C2Op = C2Op(0x0181);
    pub const SUBFC: C2Op = C2Op(0x0183);
    pub const SUBFE: C2Op = C2Op(0x0187);
    pub const MULLW: C2Op = C2Op(0x0111);
    pub const DIVW: C2Op = C2Op(0x004B);
    pub const DIVWU: C2Op = C2Op(0x004F);

    // form 22 / 23 — A-form floating point
    pub const FADD: C2Op = C2Op(0x0063);
    pub const FADDS: C2Op = C2Op(0x0065);
    pub const FSUB: C2Op = C2Op(0x009D);
    pub const FSUBS: C2Op = C2Op(0x009F);
    pub const FDIV: C2Op = C2Op(0x0073);
    pub const FDIVS: C2Op = C2Op(0x0075);
    pub const FMUL: C2Op = C2Op(0x0081);
    pub const FMULS: C2Op = C2Op(0x0083);

    // form 25 — FRT/FRB
    pub const FMR: C2Op = C2Op(0x007B);
    pub const FRSP: C2Op = C2Op(0x0093);

    /// form 36 — c2 has a **dedicated opcode** for the `or rA,rS,rS` idiom, so
    /// the port no longer spells `mr` as an `or` with a repeated operand.
    pub const MR: C2Op = C2Op(0x0272);

    // form 39 — the destination is the RA field
    pub const AND: C2Op = C2Op(0x0019);
    pub const ANDC: C2Op = C2Op(0x001B);
    pub const OR: C2Op = C2Op(0x011D);
    pub const OR_RC: C2Op = C2Op(0x011E);
    pub const ORC: C2Op = C2Op(0x011F);
    pub const EQV: C2Op = C2Op(0x0059);
    pub const XOR: C2Op = C2Op(0x026A);
    pub const SLW: C2Op = C2Op(0x013F);
    pub const SRW: C2Op = C2Op(0x014B);
    pub const SRAW: C2Op = C2Op(0x0145);

    // form 47 — RT/RA only
    pub const ADDZE: C2Op = C2Op(0x0015);
    pub const SUBFZE: C2Op = C2Op(0x0192);
    pub const NEG: C2Op = C2Op(0x0117);

    // form 38 — source in RS, destination in RA
    pub const EXTSB: C2Op = C2Op(0x005B);
    pub const EXTSB_RC: C2Op = C2Op(0x005C);
    pub const EXTSH: C2Op = C2Op(0x005D);
    pub const CNTLZW: C2Op = C2Op(0x0032);

    // form 51 — D-form signed immediate
    pub const ADDI: C2Op = C2Op(0x000B);
    pub const ADDIC: C2Op = C2Op(0x000C);
    pub const ADDIC_RC: C2Op = C2Op(0x000D);
    pub const ADDIS: C2Op = C2Op(0x000E);
    pub const MULLI: C2Op = C2Op(0x0110);
    pub const SUBFIC: C2Op = C2Op(0x018B);

    // form 43 — logical immediate, ORed unmasked
    pub const ORI: C2Op = C2Op(0x0121);
    pub const XORI: C2Op = C2Op(0x026C);

    // form 41 / 42 / 56 — shift and rotate immediates
    pub const SRAWI: C2Op = C2Op(0x0147);
    pub const RLWINM: C2Op = C2Op(0x0133);
    pub const RLWINM_RC: C2Op = C2Op(0x0134);
    pub const RLWIMI: C2Op = C2Op(0x0131);

    // form 68 — the 64-bit rotates, with two split immediate fields
    pub const RLDICL: C2Op = C2Op(0x012B);
    pub const RLDIMI: C2Op = C2Op(0x012F);

    // forms 21 / 45 / 46 — D-form loads
    pub const LWZ: C2Op = C2Op(0x00D6);
    pub const LBZ: C2Op = C2Op(0x00A3);
    pub const LBZU: C2Op = C2Op(0x00A4);
    pub const LHZ: C2Op = C2Op(0x00BA);
    pub const LD: C2Op = C2Op(0x00A7);
    pub const LFS: C2Op = C2Op(0x00B1);
    pub const LFD: C2Op = C2Op(0x00AD);

    // forms 27 / 58 / 71 — D-form stores
    pub const STW: C2Op = C2Op(0x017A);
    pub const STWU: C2Op = C2Op(0x017E);
    pub const STB: C2Op = C2Op(0x014D);
    pub const STH: C2Op = C2Op(0x0162);
    pub const STHU: C2Op = C2Op(0x0164);
    pub const STD: C2Op = C2Op(0x0151);
    pub const STDU: C2Op = C2Op(0x0155);
    pub const STFS: C2Op = C2Op(0x015E);
    pub const STFSU: C2Op = C2Op(0x015F);
    pub const STFD: C2Op = C2Op(0x0159);

    // forms 26 / 50 / 28 / 61 — indexed memory
    pub const LWZX: C2Op = C2Op(0x00D9);
    pub const LHZX: C2Op = C2Op(0x00BD);
    pub const LFSX: C2Op = C2Op(0x00B4);
    pub const STDX: C2Op = C2Op(0x0157);
    pub const STFSX: C2Op = C2Op(0x0161);

    // form 62 — the split SPR field
    pub const MTSPR: C2Op = C2Op(0x00F8);

    // forms 1 / 4 / 5 / 6 / 55 — branches
    pub const B: C2Op = C2Op(0x001F);
    pub const BC: C2Op = C2Op(0x0021);
    pub const BCLR: C2Op = C2Op(0x0027);
    pub const BCCTR: C2Op = C2Op(0x0023);
    pub const BLR: C2Op = C2Op(0x0285);
    pub const BDNZ: C2Op = C2Op(0x0288);
    /// `bctrl` is its **own opcode** on form 55, not a `bcctr` with `LK` set —
    /// its base word already carries the link bit (`4c000421`). The port used
    /// to compose it from a primary, a `BO`, an extended opcode and a literal
    /// `1`; four facts where c2 has one row.
    pub const BCTRL: C2Op = C2Op(0x002A);

    // forms 14 / 15 / 16 — compares
    pub const CMP: C2Op = C2Op(0x002D);
    pub const CMPI: C2Op = C2Op(0x002E);
    pub const CMPL: C2Op = C2Op(0x002F);
    pub const CMPLI: C2Op = C2Op(0x0030);

    // form 64 — trap immediate
    pub const TWI: C2Op = C2Op(0x019D);
}

/// **c2's base-word table, for the opcodes this port emits.** READ from the
/// pinned image; see the module doc and `docs/whitebox/DISCLOSURE.md`.
///
/// This is a **subset** of c2's 660 rows and says so: the port emits 85
/// distinct opcodes and the other 575 are not transcribed. That is deliberate —
/// a row here is a claim the port makes about a word it emits, and copying rows
/// the port never uses would put 575 unexercised claims behind the same green
/// test as the 85 exercised ones (`STATUS.md` trap 0: a green control is a
/// statement about the population it ran over).
///
/// *(**The counts above read 71 and 589 from this file's first commit
/// `227b90dd7` until 2026-08-26**, while the table has had 85 rows throughout —
/// and `EncodeParams::row`'s own comment below has said 575 correctly the whole
/// time. Corrected by lane `w-disclose` while filing the row this constant was
/// missing, board **#3643**; comment-only, no value moved and none could.)*
/// PROV[R] DISCLOSURE `W-MOP-2` — 85 whole ROWS of c2's tables, copied as
/// source literals: the base word from `0x10c3a578`, the encode-form number
/// from `0x10c39b18`, the mnemonic from `0x10b1b260`, transcribed row by row
/// from `docs/whitebox/ref/ENCODE_OPCODES.txt` (which `verify_rows.py` check E
/// reproduces byte-identically from the pinned image). **A subset by design**:
/// 85 of c2's 660 rows. `W-MID-2` adopts the two table ADDRESSES and their
/// strides into `middle_interfaces.rs` and says no entry is copied there; this
/// row is the entries, here, on the emit path.
pub static OPCODES: &[OpRow] = &[
    row(op::ADD, "add", 0x7c00_0214, 49),
    row(op::ADDE, "adde", 0x7c00_0114, 49),
    row(op::SUBF, "subf", 0x7c00_0050, 49),
    row(op::SUBFC, "subfc", 0x7c00_0010, 49),
    row(op::SUBFE, "subfe", 0x7c00_0110, 49),
    row(op::MULLW, "mullw", 0x7c00_01d6, 49),
    row(op::DIVW, "divw", 0x7c00_03d6, 49),
    row(op::DIVWU, "divwu", 0x7c00_0396, 49),
    row(op::FADD, "fadd", 0xfc00_002a, 22),
    row(op::FADDS, "fadds", 0xec00_002a, 22),
    row(op::FSUB, "fsub", 0xfc00_0028, 22),
    row(op::FSUBS, "fsubs", 0xec00_0028, 22),
    row(op::FDIV, "fdiv", 0xfc00_0024, 22),
    row(op::FDIVS, "fdivs", 0xec00_0024, 22),
    row(op::FMUL, "fmul", 0xfc00_0032, 23),
    row(op::FMULS, "fmuls", 0xec00_0032, 23),
    row(op::FMR, "fmr", 0xfc00_0090, 25),
    row(op::FRSP, "frsp", 0xfc00_0018, 25),
    row(op::MR, "mr", 0x7c00_0378, 36),
    row(op::AND, "and", 0x7c00_0038, 39),
    row(op::ANDC, "andc", 0x7c00_0078, 39),
    row(op::OR, "or", 0x7c00_0378, 39),
    row(op::OR_RC, "or.", 0x7c00_0379, 39),
    row(op::ORC, "orc", 0x7c00_0338, 39),
    row(op::EQV, "eqv", 0x7c00_0238, 39),
    row(op::XOR, "xor", 0x7c00_0278, 39),
    row(op::SLW, "slw", 0x7c00_0030, 39),
    row(op::SRW, "srw", 0x7c00_0430, 39),
    row(op::SRAW, "sraw", 0x7c00_0630, 39),
    row(op::ADDZE, "addze", 0x7c00_0194, 47),
    row(op::SUBFZE, "subfze", 0x7c00_0190, 47),
    row(op::NEG, "neg", 0x7c00_00d0, 47),
    row(op::EXTSB, "extsb", 0x7c00_0774, 38),
    row(op::EXTSB_RC, "extsb.", 0x7c00_0775, 38),
    row(op::EXTSH, "extsh", 0x7c00_0734, 38),
    row(op::CNTLZW, "cntlzw", 0x7c00_0034, 38),
    row(op::ADDI, "addi", 0x3800_0000, 51),
    row(op::ADDIC, "addic", 0x3000_0000, 51),
    row(op::ADDIC_RC, "addic.", 0x3400_0000, 51),
    row(op::ADDIS, "addis", 0x3c00_0000, 51),
    row(op::MULLI, "mulli", 0x1c00_0000, 51),
    row(op::SUBFIC, "subfic", 0x2000_0000, 51),
    row(op::ORI, "ori", 0x6000_0000, 43),
    row(op::XORI, "xori", 0x6800_0000, 43),
    row(op::SRAWI, "srawi", 0x7c00_0670, 41),
    row(op::RLWINM, "rlwinm", 0x5400_0000, 42),
    row(op::RLWINM_RC, "rlwinm.", 0x5400_0001, 42),
    row(op::RLWIMI, "rlwimi", 0x5000_0000, 56),
    row(op::RLDICL, "rldicl", 0x7800_0000, 68),
    row(op::RLDIMI, "rldimi", 0x7800_000c, 68),
    row(op::LWZ, "lwz", 0x8000_0000, 45),
    row(op::LBZ, "lbz", 0x8800_0000, 45),
    row(op::LBZU, "lbzu", 0x8c00_0000, 45),
    row(op::LHZ, "lhz", 0xa000_0000, 45),
    row(op::LD, "ld", 0xe800_0000, 46),
    row(op::LFS, "lfs", 0xc000_0000, 21),
    row(op::LFD, "lfd", 0xc800_0000, 21),
    row(op::STW, "stw", 0x9000_0000, 58),
    row(op::STWU, "stwu", 0x9400_0000, 58),
    row(op::STB, "stb", 0x9800_0000, 58),
    row(op::STH, "sth", 0xb000_0000, 58),
    row(op::STHU, "sthu", 0xb400_0000, 58),
    row(op::STD, "std", 0xf800_0000, 71),
    row(op::STDU, "stdu", 0xf800_0001, 71),
    row(op::STFS, "stfs", 0xd000_0000, 27),
    row(op::STFSU, "stfsu", 0xd400_0000, 27),
    row(op::STFD, "stfd", 0xd800_0000, 27),
    row(op::LWZX, "lwzx", 0x7c00_002e, 50),
    row(op::LHZX, "lhzx", 0x7c00_022e, 50),
    row(op::LFSX, "lfsx", 0x7c00_042e, 26),
    row(op::STDX, "stdx", 0x7c00_012a, 61),
    row(op::STFSX, "stfsx", 0x7c00_052e, 28),
    row(op::MTSPR, "mtspr", 0x7c00_03a6, 62),
    row(op::B, "b", 0x4800_0000, 6),
    row(op::BC, "bc", 0x4000_0000, 5),
    row(op::BCLR, "bclr", 0x4c00_0020, 4),
    row(op::BCCTR, "bcctr", 0x4c00_0420, 4),
    row(op::BLR, "blr", 0x4c00_0020, 55),
    row(op::BCTRL, "bctrl", 0x4c00_0421, 55),
    row(op::BDNZ, "bdnz", 0x4000_0000, 1),
    row(op::CMP, "cmp", 0x7c00_0000, 14),
    row(op::CMPI, "cmpi", 0x2c00_0000, 15),
    row(op::CMPL, "cmpl", 0x7c00_0040, 14),
    row(op::CMPLI, "cmpli", 0x2800_0000, 16),
    row(op::TWI, "twi", 0x0c00_0000, 64),
];

const fn row(op: C2Op, mnemonic: &'static str, base: u32, form: u16) -> OpRow {
    OpRow { op, mnemonic, base, form: Form(form) }
}

/// The largest c2 opcode number, `0x294` (`vmr128`) — the extent of the
/// base-word table read at `0x10c3a578`.
/// PROV[R] DISCLOSURE `W-MOP-1` — `0x294` is `_last`(`0x295`) minus one. The
/// `_last` sentinel index is what `W-MID-1` adopts into `middle_interfaces.rs`;
/// `W-MOP-1` is the same fact adopted here, beside the 85 table positions.
/// `verify_rows.py` check D asserts this equals a live dump's own extent.
const MAX_C2_OPCODE: usize = 0x294;

/// **`opcode -> 1 + index into [`OPCODES`]`, or 0 for an opcode this port does
/// not emit.** Built at COMPILE TIME from `OPCODES`, so it cannot drift from
/// it: there is no second list to keep in sync, only a derivation.
/// PROV[N] carries no independent fact — derived from [`OPCODES`] at compile
/// time by [`build_opcode_index`], so its provenance IS `OPCODES`'.
static OPCODE_INDEX: [u8; MAX_C2_OPCODE + 1] = build_opcode_index();

const fn build_opcode_index() -> [u8; MAX_C2_OPCODE + 1] {
    let mut ix = [0u8; MAX_C2_OPCODE + 1];
    let mut i = 0;
    while i < OPCODES.len() {
        let op = OPCODES[i].op.0 as usize;
        // A `const` panic here is a compile error, which is the right place for
        // "someone added a row whose opcode is out of the table's extent" and
        // for "someone added an 86th row past what a u8 index can name".
        assert!(op <= MAX_C2_OPCODE, "opcode past the base-word table's extent");
        assert!(i < 255, "OPCODES outgrew a u8 index");
        ix[op] = (i + 1) as u8;
        i += 1;
    }
    ix
}

// ---- the per-op value ------------------------------------------------------

/// **One machine instruction, as c2's encoder sees it**: an opcode number plus
/// the operand slots the form arm reads.
///
/// The slot names are c2's, from `P_ENCODE.md` §5's notation — `S` is the
/// **destination** operand (`t+0x2c`), `D0` the first source (`t+0x28`), `D1`
/// the second, and `D2`/`D3` the third and fourth where a form takes them
/// (only `rlwinm`/`rlwimi`, which read `SH`, `MB` and `ME` as three successive
/// operands' immediates).
///
/// A slot holds either a hardware register number — c2 reaches it as
/// `reg(x) = [[x+0x1c]+0x28]`, this port already has it — or an immediate. The
/// form decides which, and a slot a form does not read is ignored rather than
/// refused, because c2's arms read exactly the slots they name and no others
/// (`P_ENCODE.md` §5, PREREG P3.2, *"a HIT with zero exceptions"*).
///
/// **`disp` is separate from the slots on purpose.** It is the only field that
/// is signed and wider than a register, and folding it into a slot would make
/// every register slot `i32` to serve one case.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MachineOp {
    pub op: C2Op,
    pub s: u32,
    pub d0: u32,
    pub d1: u32,
    pub d2: u32,
    pub d3: u32,
    pub disp: i32,
}

impl MachineOp {
    /// A value with every slot zero. Chained `.s()`/`.d0()`/… fill what the
    /// form reads.
    #[inline]
    pub const fn new(op: C2Op) -> Self {
        MachineOp { op, s: 0, d0: 0, d1: 0, d2: 0, d3: 0, disp: 0 }
    }

    #[inline]
    pub const fn s(mut self, v: u8) -> Self {
        self.s = v as u32;
        self
    }
    #[inline]
    pub const fn d0(mut self, v: u8) -> Self {
        self.d0 = v as u32;
        self
    }
    #[inline]
    pub const fn d1(mut self, v: u8) -> Self {
        self.d1 = v as u32;
        self
    }
    #[inline]
    pub const fn d2(mut self, v: u8) -> Self {
        self.d2 = v as u32;
        self
    }
    #[inline]
    pub const fn d3(mut self, v: u8) -> Self {
        self.d3 = v as u32;
        self
    }
    /// A raw immediate into a slot, unnarrowed — `SPR` numbers, `UI` fields and
    /// the 6-bit `SH`/`MB` of the doubleword rotates all exceed `u8`'s role as
    /// a register number, so they take this rather than [`MachineOp::d1`].
    #[inline]
    pub const fn imm_d1(mut self, v: u32) -> Self {
        self.d1 = v;
        self
    }
    #[inline]
    pub const fn imm_d2(mut self, v: u32) -> Self {
        self.d2 = v;
        self
    }
    #[inline]
    pub const fn disp(mut self, v: i32) -> Self {
        self.disp = v;
        self
    }

    /// Compose the word at the **default** parameters — the only composition an
    /// emit may use.
    ///
    /// Panics only if the opcode is not in [`OPCODES`], which is a programming
    /// error in this crate rather than an input condition: every caller is a
    /// constructor in [`super::encode`] naming a `const` from [`op`]. The
    /// fallible form is [`encode_op`], for callers that build an op from data.
    #[inline]
    pub fn word(self) -> [u8; 4] {
        match encode_op(&self, &EncodeParams::C2) {
            Ok(w) => w.to_be_bytes(),
            Err(_) => unreachable!("every encode.rs constructor names an OPCODES row"),
        }
    }
}

// ---- the decision surface --------------------------------------------------

/// Which operand slot a field draws from.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Slot {
    S,
    D0,
    D1,
    D2,
    D3,
    /// The signed displacement, not a slot.
    Disp,
    /// The displacement **in words** — `disp >> 2`.
    ///
    /// A separate slot rather than a `>> 2` folded into a shift, because that
    /// is literally how c2 writes it: `P_ENCODE.md` §5.3 gives form 5 as
    /// `((([D0+0x18][0x18] − pc) >> 2) & 0x3fff) << 2` and form 2 as
    /// `LI = (((target − pc) >> 2) & 0xffffff) << 2`. The arithmetic shift is
    /// load-bearing for a **backward** branch and a logical mask over the raw
    /// byte displacement is not the same function — it only coincides while the
    /// displacement is a multiple of 4, which is a caller precondition rather
    /// than a property of the field.
    DispWord,
}

/// One placed field: take [`Slot`], keep `width` low bits, shift left by
/// `shift`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Field {
    pub slot: Slot,
    pub shift: u32,
    pub width: u32,
}

const fn f(slot: Slot, shift: u32, width: u32) -> Field {
    Field { slot, shift, width }
}

/// How one c2 **form** places its operands.
///
/// `fixed` is the constant a form's arm ORs in without reading any operand —
/// `P_ENCODE.md` §5.3's form 55 is exactly one of these (*"`or ebx,0x2800000`
/// — `BO = 20`, no operand read at all"*), and it is where three of board
/// **#3379**'s seven residuals live.
#[derive(Clone, Copy, Debug)]
pub struct FieldPlan {
    fields: [Field; MAX_FIELDS],
    n: usize,
    pub fixed: u32,
}

/// The widest form this port emits is `rlwinm`/`rlwimi` at five placed fields.
/// PROV[N] a capacity bound on the port's own [`FieldPlan`] representation, not
/// a fact about c2. It is wide enough for every form `P_ENCODE.md` §5 read; a
/// wider form would be a compile error here, which is the intended failure.
pub const MAX_FIELDS: usize = 5;

/// A padding entry for the unused tail of a [`FieldPlan`]'s array. Never read:
/// [`FieldPlan::fields`] slices to `n` first.
/// PROV[N] a padding sentinel for the unused tail of a [`FieldPlan`] array.
/// Never read — [`FieldPlan::fields`] slices to `n` first.
const NONE_FIELD: Field = Field { slot: Slot::S, shift: 0, width: 0 };

impl FieldPlan {
    /// The placed fields, in the order the arm ORs them.
    pub fn fields(&self) -> &[Field] {
        &self.fields[..self.n]
    }
}

const fn fp(list: [Field; MAX_FIELDS], n: usize, fixed: u32) -> FieldPlan {
    FieldPlan { fields: list, n, fixed }
}
const fn fp0(fixed: u32) -> FieldPlan {
    fp([NONE_FIELD; MAX_FIELDS], 0, fixed)
}
const fn fp1(a: Field) -> FieldPlan {
    fp([a, NONE_FIELD, NONE_FIELD, NONE_FIELD, NONE_FIELD], 1, 0)
}
const fn fp1x(a: Field, fixed: u32) -> FieldPlan {
    fp([a, NONE_FIELD, NONE_FIELD, NONE_FIELD, NONE_FIELD], 1, fixed)
}
const fn fp2(a: Field, b: Field) -> FieldPlan {
    fp([a, b, NONE_FIELD, NONE_FIELD, NONE_FIELD], 2, 0)
}
const fn fp3(a: Field, b: Field, c: Field) -> FieldPlan {
    fp([a, b, c, NONE_FIELD, NONE_FIELD], 3, 0)
}
const fn fp5(a: Field, b: Field, c: Field, d: Field, e: Field) -> FieldPlan {
    fp([a, b, c, d, e], 5, 0)
}

/// **The named, enumerable parameters of the general composition.**
///
/// [`EncodeParams::C2`] is the default and reproduces c2 byte-exactly; it is
/// the only value any emit path uses, and the required-zero byte delta is
/// graded at it and nowhere else. Every other value is an instrument state —
/// what a permuter searches, and what a mutation control perturbs to check that
/// a green test *could* have gone red.
#[derive(Clone, Copy, Debug)]
pub struct EncodeParams {
    pub rows: &'static [OpRow],
    /// Overrides applied on top of the form rules, by `(form, field index)`.
    /// Empty in [`EncodeParams::C2`].
    pub width_override: Option<(u16, usize, u32)>,
    /// When set, the field at `(form, index)` is dropped entirely.
    pub drop_override: Option<(u16, usize)>,
}

impl EncodeParams {
    /// The default: c2's own tables and c2's own field placements.
    /// PROV[R] DISCLOSURE `W-MOP-3` — the field placements, transcribed from
    /// `docs/whitebox/ref/P_ENCODE.md` §5, read arm by arm 79/79 by lane
    /// `w-read-r2`, of which §5.1 calls form 39's *"the single most
    /// safety-critical fact on this page."* This is the DEFAULT that every emit
    /// uses, per `docs/rungs/README.md`'s decision-surface clause; the override
    /// fields beside it are instrument states and license no emit. `W-MID-3` is
    /// the same subject at a different scale — it reads **2** of the 111 arms
    /// and says so; `W-MOP-3` is **27 of the 79 distinct arms**, and that row's
    /// other half still holds: **no relocation and no label placement is
    /// adopted anywhere in this module.**
    pub const C2: EncodeParams =
        EncodeParams { rows: OPCODES, width_override: None, drop_override: None };

    /// A **mutation state** — narrow one form's field and see what stops
    /// matching. Board **#3379** ran four of these by hand-editing `encode.rs`
    /// (`D` 16→12 bits, `RB` 5→4, drop `RA`, `SPR` unsplit) and measured
    /// 99.38% → 91.40% / 92.32% / 73.49% / 95.66% over 634,457 emitted words.
    /// They are parameter values now, so the control can be re-run without a
    /// patch — and, per #3379's own lesson, **on a population big enough to
    /// notice**: its purpose-built 46-word probe could not tell a 4-bit `RB`
    /// from a 5-bit one, because no word in it used a register ≥ 16.
    pub const fn with_field_width(self, form: u16, index: usize, width: u32) -> Self {
        EncodeParams { width_override: Some((form, index, width)), ..self }
    }

    /// A mutation state: drop one form's field entirely.
    pub const fn without_field(self, form: u16, index: usize) -> Self {
        EncodeParams { drop_override: Some((form, index)), ..self }
    }

    #[inline(always)]
    fn row(&self, op: C2Op) -> Option<&'static OpRow> {
        // **O(1), because this runs once per emitted instruction.**
        //
        // It was `self.rows.iter().find(...)` — an 85-row linear scan — for
        // exactly as long as it took to measure it: lane `w-s1`'s own
        // pre-registered cost criterion (#3336) read **+10.67 % mean port time
        // per obj and +19.04 % aggregate**, with two fixtures at **4.2x** and
        // **3.9x** and no overlap at all between the arms' distributions. The
        // byte delta was zero throughout; a required-zero byte delta is silent
        // about cost, which is the whole content of #3336, and this is what it
        // was silent about.
        //
        // The dual path is a PERFORMANCE path, never a semantic one: both
        // branches return the same row, and `the_index_and_the_scan_agree_on_
        // every_opcode` checks that over the entire opcode space including the
        // 575 absent ones.
        if std::ptr::eq(self.rows.as_ptr(), OPCODES.as_ptr()) && self.rows.len() == OPCODES.len() {
            let i = *OPCODE_INDEX.get(op.0 as usize)?;
            if i == 0 {
                return None;
            }
            return Some(&OPCODES[i as usize - 1]);
        }
        self.rows.iter().find(|r| r.op == op)
    }
}

/// Look up c2's base word for an opcode, or `None` if this port does not emit
/// it. **The port's only source of a primary opcode.**
pub fn base_word(op: C2Op) -> Option<u32> {
    EncodeParams::C2.row(op).map(|r| r.base)
}

/// Look up c2's form number for an opcode.
pub fn form_of(op: C2Op) -> Option<Form> {
    EncodeParams::C2.row(op).map(|r| r.form)
}

/// Look up c2's mnemonic for an opcode.
pub fn mnemonic_of(op: C2Op) -> Option<&'static str> {
    EncodeParams::C2.row(op).map(|r| r.mnemonic)
}

// ---- the one composition ---------------------------------------------------

/// **The field plan for one c2 form**, transcribed from `P_ENCODE.md` §5.
///
/// Returns `None` for a form this port does not emit; the port's **85** opcodes
/// reach **34** of the **104** distinct form values c2's table contains
/// (`P_ENCODE.md` §3), and the 27 arms below cover **35** form numbers — the
/// extra one is form 2, which shares an arm with form 6.
///
/// *(**This line read "71 opcodes reach 24 of c2's 109 forms" from
/// `227b90dd7` until 2026-08-26.** Corrected by lane `w-disclose`, board
/// **#3643**, comment-only. See `OPCODES`' note for the same correction on the
/// row count.)*
///
/// Provenance: the marker on [`EncodeParams::C2`] above covers every plan here
/// — DISCLOSURE `W-MOP-3`. **This line deliberately carries no marker token of
/// its own**: `plan` is a *rule*, and rule marks are `w-provext`'s surface this
/// wave, so adding one here would move a peer's census number. Reported as
/// owed, not taken (board **#3646**).
#[inline(always)]
pub fn plan(form: Form) -> Option<FieldPlan> {
    use Slot::*;
    // Every arm below cites an address in `code.c` that the placement was read
    // from — **for most arms that is the jump-table arm itself, and for the
    // four memory groups it is the COMPOSER the arm calls**, which is one level
    // deeper and is where the placement actually lives (`P_ENCODE.md` §5.5: the
    // memory arms do nothing but `call <composer>; or ebx,eax`). The mapping,
    // recorded so a future reader is not left to rediscover it:
    //     0x10bf9e55 <- arm 0x10bfa667 (forms 21/45/46, D-form load)
    //     0x10bf9eb5 <- arm 0x10bfa676 (forms 27/58/71, D-form store)
    //     0x10bf9788 <- arm 0x10bfa17f (forms 26/50,    X-form load)
    //     0x10bf97c8 <- arm 0x10bfa1a1 (forms 28/61,    X-form store)
    // `work/w-disclose/verify_rows.py` check F accounts for all 32 c2 addresses
    // this file cites, so the set cannot rot unnoticed the way
    // `WB_INLINE_FINDINGS.md`'s four did for eight days (board #3626).
    let p: FieldPlan = match form.0 {
        // `10bfa456` — RT=reg(S), RA=reg(D0), RB=reg(D1). 77 opcodes, the
        // busiest integer form; form 22 is the same arm (A-form FP).
        49 | 22 => fp3(f(S, 21, 5), f(D0, 16, 5), f(D1, 11, 5)),
        // `10bfa478` — `fmul`: the multiplier is in the **C** field at bit 6,
        // not B. Reusing form 22 here silently multiplies by the wrong
        // register (`encode.rs`'s own trap note).
        23 => fp3(f(S, 21, 5), f(D0, 16, 5), f(D1, 6, 5)),
        // `10bfa4df` — `fmr`, `frsp`, `fabs`: no A field at all.
        25 => fp2(f(S, 21, 5), f(D0, 11, 5)),
        // `10bfa53b` — **the destination is the RA field.** `P_ENCODE.md` §5.1
        // names this the single most safety-critical fact on the page, and
        // `encode_logical_x`'s doc says the same from the black-box side:
        // getting it wrong yields a valid `and` with the destination and the
        // left operand exchanged.
        39 => fp3(f(D0, 21, 5), f(S, 16, 5), f(D1, 11, 5)),
        // `10bfa549` — the `or rA,rS,rS` idiom behind `mr`/`not`: c2 reads D0
        // TWICE, into RS and RB, which is why `mr` needs no second source
        // operand and why it is its own opcode rather than an `or`.
        36 => fp3(f(D0, 21, 5), f(S, 16, 5), f(D0, 11, 5)),
        // `10bfa4c8` — `neg`, `addze`, `subfze`.
        47 => fp2(f(S, 21, 5), f(D0, 16, 5)),
        // `10bfa587` — `extsb`, `cntlzw`: source in RS, destination in RA.
        38 => fp2(f(D0, 21, 5), f(S, 16, 5)),
        // `10bfa4ed` — D-form signed immediate, then §5.4's three-way.
        51 => fp3(f(S, 21, 5), f(D0, 16, 5), f(Disp, 0, 16)),
        // `10bfa56b` — logical immediate. **`imm` is ORed unmasked**, which is
        // why `UI` is 16 bits here and the value arrives through `imm_d1`.
        43 => fp3(f(D0, 21, 5), f(S, 16, 5), f(D1, 0, 16)),
        // `10bfa685` — `srawi`.
        41 => fp3(f(D0, 21, 5), f(S, 16, 5), f(D1, 11, 5)),
        // `10bfa6dc` / `10bfa719` — `rlwinm` / `rlwimi`: SH, MB and ME are each
        // a byte at `imm()` of successive operands.
        42 | 56 => fp5(
            f(D0, 21, 5),
            f(S, 16, 5),
            f(D1, 11, 5),
            f(D2, 6, 5),
            f(D3, 1, 5),
        ),
        // `10bfad76` — the 64-bit rotates. Both immediate fields are SPLIT and
        // the split is not the same shape twice: `SH[4:0]` at 11 with `SH[5]`
        // alone at bit 1, and `MB` stored **low-bit-first** as
        // `(MB & 0x1f) << 1 | (MB >> 5)` based at bit 5. A `FieldPlan` cannot
        // express either, so this form composes in code below.
        68 => fp0(0),
        // `10bf9e55` (load) / `10bf9eb5` (store) — the D-form memory composers,
        // mirror images: the load takes its register from `t+0x2c` and its
        // memory operand from `t+0x28`, the store the other way round. Both
        // return `(reg<<5 | RA)<<16 | (u16)disp`, i.e. reg at 21 and RA at 16.
        21 | 45 => fp3(f(S, 21, 5), f(D0, 16, 5), f(Disp, 0, 16)),
        27 | 58 => fp3(f(D0, 21, 5), f(S, 16, 5), f(Disp, 0, 16)),
        // The **DS**-form pair — `ld` (46) and `std`/`stdu` (71). The low two
        // bits of the 16-bit field are the form selector, not displacement, so
        // the field is 14 bits of `disp >> 2` based at bit 2. `stdu`'s selector
        // bit is already in its base word (`f8000001`), which is why this plan
        // does not carry it.
        46 => fp3(f(S, 21, 5), f(D0, 16, 5), f(DispWord, 2, 14)),
        71 => fp3(f(D0, 21, 5), f(S, 16, 5), f(DispWord, 2, 14)),
        // `10bf9788` / `10bf97c8` — the indexed pair, same mirror image.
        26 | 50 => fp3(f(S, 21, 5), f(D0, 16, 5), f(D1, 11, 5)),
        28 | 61 => fp3(f(D0, 21, 5), f(S, 16, 5), f(D1, 11, 5)),
        // `10bfa7a3` — `mtspr`. The SPR is written LOW HALF FIRST, and c2 does
        // the split in the arm rather than in the base word, which is why
        // `mtspr`'s base word is `7c0003a6` with the field zero (§8.1 residual
        // 5). `9 << 11` would be a legal-looking `mtspr` naming SPR 288.
        62 => fp3(f(S, 21, 5), f(D1, 16, 5), f(D2, 11, 5)),
        // `10bfa2a5` — `blr`/`bctr`/`bctrl`: `or ebx,0x2800000`, i.e. BO = 20
        // supplied by the ARM, no operand read at all. The port used to bake
        // this into a literal `4e800020`; it is #3379's `BO_ALWAYS` residual
        // and it is now visible as a `fixed`.
        55 => fp0(0x0280_0000),
        // `10bfa2b0` — `bclr`/`bcctr`: BO and BI only, no displacement.
        4 => fp2(f(S, 21, 5), f(D0, 16, 5)),
        // `10bfa326` — `bc`: BO, BI, then the 14-bit self-relative
        // displacement `((target − pc) >> 2 & 0x3fff) << 2`.
        5 => fp3(f(S, 21, 5), f(D0, 16, 5), f(DispWord, 2, 14)),
        // `10bfa2c2` — `bdnz`: with no condition operand the arm ORs
        // `0x2000000`, i.e. BO = 16, then the same 14-bit displacement. The
        // port used to pass `BO_DNZ` through its `bc` encoder; that is #3379's
        // `BO = 16` residual and it is a `fixed` now.
        1 => fp1x(f(DispWord, 2, 14), 0x0200_0000),
        // `10bfa263` → `10bfa26c` — `b` to a local label: the 24-bit
        // self-relative displacement, `LI = ((target − pc) >> 2 & 0xffffff) << 2`.
        6 | 2 => fp1(f(DispWord, 2, 24)),
        // `10bfa34f` / `10bfa3ba` / `10bfa415` — the compares. `crf` is a
        // 3-bit field at bit 23; the `L` bit at 21 stays 0 for the 32-bit
        // forms, which is the only kind this port emits.
        14 => fp3(f(S, 23, 3), f(D0, 16, 5), f(D1, 11, 5)),
        15 | 16 => fp3(f(S, 23, 3), f(D0, 16, 5), f(Disp, 0, 16)),
        // `10bfa801` — `twi`: TO, RA, then a signed immediate.
        64 => fp3(f(S, 21, 5), f(D0, 16, 5), f(Disp, 0, 16)),
        _ => return None,
    };
    Some(p)
}

/// **The one general composition**: `base_word[op] | fields placed by form[op]`.
///
/// This is `P_ENCODE.md`'s `encode(tuple) -> u32` restricted to the operand
/// slots this port fills, and it is the only place in the crate that turns an
/// instruction into a word.
#[inline(always)]
pub fn encode_op(m: &MachineOp, params: &EncodeParams) -> Result<u32, BackendError> {
    let row = params.row(m.op).ok_or_else(|| {
        BackendError::NotImplemented(format!(
            "opcode {:#06x} is not in the port's base-word table \
             (docs/whitebox/ref/ENCODE_OPCODES.txt has all 660; the port \
             transcribes the {} it emits)",
            m.op.0,
            params.rows.len(),
        ))
    })?;

    // The two 64-bit rotates compose in code: their SH and MB fields are split
    // in two different shapes and a (slot, shift, width) triple cannot say so.
    if row.form.0 == 68 {
        let sh = m.d1 & 0x3F;
        let mb = m.d2 & 0x3F;
        return Ok(row.base
            | ((m.d0 & 0x1F) << 21)
            | ((m.s & 0x1F) << 16)
            | ((sh & 0x1F) << 11)
            | ((((mb & 0x1F) << 1) | (mb >> 5)) << 5)
            | ((sh >> 5) << 1));
    }

    let fp = plan(row.form).ok_or_else(|| {
        BackendError::NotImplemented(format!(
            "c2 form {} (opcode {:#06x} `{}`) has no field plan in this port",
            row.form.0, m.op.0, row.mnemonic,
        ))
    })?;

    let mut word = row.base | fp.fixed;

    // **The default path carries no per-field branches.** `width_override` and
    // `drop_override` are instrument states — a permuter's search surface, not
    // something an emit ever sets — so testing them once per FIELD put two
    // `Option` compares on the port's hottest loop to serve a configuration no
    // emit uses. Hoisted, so the mutation machinery costs the default nothing.
    if params.width_override.is_none() && params.drop_override.is_none() {
        for field in fp.fields() {
            word |= (slot_of(m, field.slot) & mask_of(field.width)) << field.shift;
        }
        return Ok(word);
    }

    for (i, field) in fp.fields().iter().enumerate() {
        if params.drop_override == Some((row.form.0, i)) {
            continue;
        }
        let width = match params.width_override {
            Some((form, idx, w)) if form == row.form.0 && idx == i => w,
            _ => field.width,
        };
        word |= (slot_of(m, field.slot) & mask_of(width)) << field.shift;
    }
    Ok(word)
}

#[inline(always)]
fn slot_of(m: &MachineOp, slot: Slot) -> u32 {
    match slot {
        Slot::S => m.s,
        Slot::D0 => m.d0,
        Slot::D1 => m.d1,
        Slot::D2 => m.d2,
        Slot::D3 => m.d3,
        Slot::Disp => m.disp as u32,
        Slot::DispWord => (m.disp >> 2) as u32,
    }
}

#[inline(always)]
fn mask_of(width: u32) -> u32 {
    if width >= 32 {
        u32::MAX
    } else {
        (1u32 << width) - 1
    }
}

// ---- the agreement instrument ----------------------------------------------

/// One row of the **`after0` opcode-agreement** measurement.
///
/// `after0` is the word a constructor produces **at every operand zero**. For a
/// form that reads only operands, that word is exactly c2's base word; where it
/// is not, the difference is a field the port supplies as a constant, and the
/// set of those is the measurement.
#[derive(Clone, Copy, Debug)]
pub struct AgreementRow {
    pub op: C2Op,
    pub mnemonic: &'static str,
    /// c2's base word, READ.
    pub base: u32,
    /// The port's word at every operand zero, through the general composition.
    pub after0: u32,
    /// `after0 ^ base` — zero iff they agree.
    pub delta: u32,
}

impl AgreementRow {
    pub fn agrees(&self) -> bool {
        self.delta == 0
    }
}

/// **Measure `after0` agreement over every opcode the port emits.**
///
/// An INSTRUMENT. It gates nothing, licenses no emit, and never appears in a
/// refusal predicate — the byte judge is real `c2.dll` under wibo and this is
/// not it (`docs/FUNCTION_BYTE_MATCH.md`'s separation rule, applied one level
/// down).
///
/// **Read the denominator, not the ratio** (`STATUS.md` trap 0b, standing rule
/// 4 / board #3356): the population is the opcodes *this port emits*, which is
/// 71 of c2's 660, and a row is only as good as the constructor that fills it.
pub fn agreement(params: &EncodeParams) -> Vec<AgreementRow> {
    params
        .rows
        .iter()
        .map(|r| {
            let m = MachineOp::new(r.op);
            let after0 = encode_op(&m, params).unwrap_or(0);
            AgreementRow {
                op: r.op,
                mnemonic: r.mnemonic,
                base: r.base,
                after0,
                delta: after0 ^ r.base,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every opcode named in [`op`] has a row, every row has a field plan, and
    /// no opcode number is listed twice.
    ///
    /// The duplicate check is the load-bearing one: `OPCODES` is a linear scan,
    /// so a duplicated opcode number would silently shadow the second row and
    /// the shadowed one would never be exercised by any test that goes through
    /// [`encode_op`].
    #[test]
    fn the_table_is_total_and_has_no_duplicate_opcode() {
        let mut seen: Vec<u16> = OPCODES.iter().map(|r| r.op.0).collect();
        let before = seen.len();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(before, seen.len(), "OPCODES has a duplicate opcode number");
        for r in OPCODES {
            assert!(
                plan(r.form).is_some(),
                "opcode {:#06x} `{}` has form {} with no field plan",
                r.op.0,
                r.mnemonic,
                r.form.0
            );
        }
    }

    /// **The O(1) index and the linear scan agree on the WHOLE opcode space**,
    /// including the 575 opcodes this port does not emit and the ones past the
    /// table's extent.
    ///
    /// The fast path is guarded by a pointer comparison, so a future edit that
    /// makes `rows` a different-but-equal slice would silently take the scan —
    /// correct, and slow. This test pins the agreement; the cost criterion
    /// pins that taking the scan is expensive.
    #[test]
    fn the_index_and_the_scan_agree_on_every_opcode() {
        for n in 0..=(MAX_C2_OPCODE as u16 + 8) {
            let op = C2Op(n);
            let fast = EncodeParams::C2.row(op).map(|r| r.op);
            let slow = OPCODES.iter().find(|r| r.op == op).map(|r| r.op);
            assert_eq!(fast, slow, "index and scan disagree at opcode {n:#06x}");
        }
        let present = (0..=MAX_C2_OPCODE as u16)
            .filter(|n| EncodeParams::C2.row(C2Op(*n)).is_some())
            .count();
        assert_eq!(present, OPCODES.len(), "the index does not reach every row");
    }

    /// An opcode the port does not emit is a clean refusal, never a panic and
    /// never a wrong word. `0x0002` (`add.`) is a real c2 opcode this port has
    /// no constructor for, which makes it the honest probe.
    #[test]
    fn an_untranscribed_opcode_refuses() {
        let m = MachineOp::new(C2Op(0x0002));
        assert!(encode_op(&m, &EncodeParams::C2).is_err());
        assert!(base_word(C2Op(0x0002)).is_none());
    }

    /// **Form 39's placement, pinned against form 49's, in the portable lane.**
    ///
    /// These are the two busiest integer forms and they differ by exactly the
    /// swap that `P_ENCODE.md` §5.1 calls the most safety-critical fact on the
    /// page: form 49 puts the destination at bit 21, form 39 at bit 16. A
    /// single regression test on `or` alone could not see it, because `or`'s
    /// three registers are often equal at a call site.
    #[test]
    fn form_39_puts_the_destination_in_ra_and_form_49_does_not() {
        // and r3, r11, r5  — dest 3, lhs 11, rhs 5
        let and = MachineOp::new(op::AND).s(3).d0(11).d1(5);
        assert_eq!(encode_op(&and, &EncodeParams::C2).unwrap(), 0x7d63_2838);
        // add r3, r11, r5  — dest 3, ra 11, rb 5
        let add = MachineOp::new(op::ADD).s(3).d0(11).d1(5);
        assert_eq!(encode_op(&add, &EncodeParams::C2).unwrap(), 0x7c6b_2a14);
    }

    /// The `fixed` constants are what three of board #3379's seven residuals
    /// were, and they are now readable as data rather than as a literal word.
    #[test]
    fn form_55_supplies_bo_20_from_the_arm_not_from_the_base_word() {
        assert_eq!(base_word(op::BLR).unwrap(), 0x4c00_0020);
        assert_eq!(plan(form_of(op::BLR).unwrap()).unwrap().fixed, 0x0280_0000);
        assert!(plan(form_of(op::BLR).unwrap()).unwrap().fields().is_empty());
        assert_eq!(
            encode_op(&MachineOp::new(op::BLR), &EncodeParams::C2).unwrap(),
            0x4e80_0020
        );
    }

    /// A mutation state changes a word, and the DEFAULT does not — the control
    /// that says the parameter is wired to the composition at all.
    ///
    /// Registered in the prereg as the answer to "a criterion that cannot fail
    /// abstains rather than passes": if `with_field_width` were ignored, every
    /// agreement number this module publishes would be unfalsifiable.
    #[test]
    fn a_field_width_mutation_bites_and_the_default_does_not() {
        // lwz r3, 0x1234(r20) — RA = 20 needs its fifth bit.
        let m = MachineOp::new(op::LWZ).s(3).d0(20).disp(0x1234);
        let good = encode_op(&m, &EncodeParams::C2).unwrap();
        assert_eq!(good, 0x8074_1234);
        // Narrow form 45's RA field (index 1) from 5 bits to 4: r20 becomes r4.
        let narrowed = EncodeParams::C2.with_field_width(45, 1, 4);
        assert_eq!(encode_op(&m, &narrowed).unwrap(), 0x8064_1234);
        // Narrow the displacement to 12 bits — #3379's `D` mutation.
        let dnarrow = EncodeParams::C2.with_field_width(45, 2, 12);
        assert_eq!(encode_op(&m, &dnarrow).unwrap(), 0x8074_0234);
        // Drop RA entirely — #3379's third mutation.
        let dropped = EncodeParams::C2.without_field(45, 1);
        assert_eq!(encode_op(&m, &dropped).unwrap(), 0x8060_1234);
    }

    /// **THE `after0` OPCODE-AGREEMENT RATIO — an INSTRUMENT, never a gate.**
    ///
    /// Slice S1's third graded-by (`ROADMAP_SLICING_2026-08-21.md` §5, row S1;
    /// `DECISIONS_2026-08-22.md` decision 5). It asserts the **shape of the
    /// residual**, not a ratio floor, because a ratio floor on a
    /// characterization number is a gate wearing an instrument's clothes: it
    /// would go red the day someone adds an opcode whose arm supplies a field,
    /// which is a fact about c2 and not a regression in the port.
    ///
    /// Run it for the numbers:
    /// `cargo test -p c2-core --lib after0 -- --nocapture`.
    ///
    /// **READ THE DENOMINATOR** (standing rule 4 / board #3356). It is **85
    /// distinct c2 opcodes**, and that is *not* board #3379's **89**, which
    /// counted **encoder functions**. The two populations differ by the
    /// convenience wrappers — `encode_srwi31` and `encode_clrlwi31` are
    /// `rlwinm` with the mask baked, not opcodes — and by the `double` flag,
    /// where one function reaches two rows (`fadd`/`fadds`). Neither number is
    /// wrong; they answer different questions, and the ratio is meaningless
    /// without saying which.
    ///
    /// **The ratio reads `82 / 85`, and #3379's read `82 / 89`. The equal
    /// numerators are a COINCIDENCE and must not be reported as agreement.**
    /// The seven residuals #3379 named went to three, for two different
    /// reasons that a single count would have blurred:
    ///
    /// * `BO = 20` ×2 (`blr`, `bctrl`) and `BO = 16` (`bdnz`) **survive** —
    ///   they are genuinely fields c2's arm supplies (form 55's
    ///   `or ebx,0x2800000`, form 1's `or 0x2000000`) and the port must
    ///   reproduce them.
    /// * The **split `SPR`** dissolved for a real reason: `encode_mtctr` used
    ///   to bake `0x120` into its word and now passes SPR 9 through form 62's
    ///   split placement, so `mtspr`'s `after0` **is** its base word. One
    ///   fitted field became a read one.
    /// * `SH`/`MB`/`ME` ×3 dissolved for a **definitional** reason and bought
    ///   nothing: they were `encode_srwi31`, `encode_clrlwi31` and
    ///   `encode_clrlwi_record`, three convenience wrappers over one opcode.
    ///   `rlwinm` itself always agreed. Counting them as three residuals was
    ///   an artifact of counting functions.
    #[test]
    fn after0_agreement_with_c2s_base_word_table() {
        let rows = agreement(&EncodeParams::C2);
        let (agree, differ): (Vec<&AgreementRow>, Vec<&AgreementRow>) =
            rows.iter().partition(|r| r.agrees());
        println!(
            "\nafter0 opcode agreement: {} of {} distinct c2 opcodes the port emits",
            agree.len(),
            rows.len()
        );
        println!("the shape of M - N ({} rows):", differ.len());
        for r in &differ {
            println!(
                "  {:<8} op {:#06x} form {:<3} base {:08x}  after0 {:08x}  delta {:08x}",
                r.mnemonic,
                r.op.0,
                form_of(r.op).unwrap().0,
                r.base,
                r.after0,
                r.delta
            );
        }

        // Every residual is a `fixed` — a field c2's ARM supplies with no
        // operand read — and never a disagreement in a primary or extended
        // opcode. That is the claim worth pinning, and it is #3379's finding
        // stated as a property instead of as a count.
        for r in &differ {
            let fp = plan(form_of(r.op).unwrap()).unwrap();
            assert_ne!(fp.fixed, 0, "{} differs but its form ORs no constant", r.mnemonic);
            assert_eq!(
                r.delta, fp.fixed,
                "{}'s after0 delta is not exactly its form's fixed constant",
                r.mnemonic
            );
        }
        // …and symmetrically: an opcode whose form ORs nothing MUST agree, or
        // the transcription of its base word is wrong.
        for r in &agree {
            assert_eq!(plan(form_of(r.op).unwrap()).unwrap().fixed, 0);
        }

        // The residual set, named in advance rather than read off the run.
        let mut names: Vec<&str> = differ.iter().map(|r| r.mnemonic).collect();
        names.sort_unstable();
        assert_eq!(
            names,
            vec!["bctrl", "bdnz", "blr"],
            "the after0 residual set changed; that is a finding, not a failure \
             — re-derive it before adjusting this list"
        );
    }

    /// The mnemonics in [`OPCODES`] are c2's, so they can be checked against
    /// the committed dump rather than trusted.
    ///
    /// A transcription test, and the only thing standing between this table and
    /// a typo that would be invisible everywhere else: a wrong base word whose
    /// primary opcode happens to be right still assembles, still disassembles,
    /// and is caught by nothing above except the incumbent cross-check — which
    /// is exactly why that cross-check is kept as an INDEPENDENT derivation.
    #[test]
    fn every_transcribed_row_is_internally_consistent() {
        for r in OPCODES {
            assert!(!r.mnemonic.is_empty());
            // c2's table has no base word with bits below the primary opcode
            // set for an operand field this port fills — checked as: composing
            // at all-zero operands never loses a bit of the base.
            let after0 = encode_op(&MachineOp::new(r.op), &EncodeParams::C2).unwrap();
            assert_eq!(after0 & r.base, r.base, "{} loses base bits at after0", r.mnemonic);
        }
    }

    /// The `mtspr` split, which is the one place a plain `(slot, shift, width)`
    /// triple would have been wrong in a way that still assembles: SPR 9
    /// written unsplit names SPR 288.
    #[test]
    fn the_spr_field_is_split_low_half_first() {
        // mtctr r11 = mtspr 9, r11
        let m = MachineOp::new(op::MTSPR).s(11).imm_d1(9 & 0x1F).imm_d2(9 >> 5);
        assert_eq!(encode_op(&m, &EncodeParams::C2).unwrap(), 0x7d69_03a6);
    }
}

// ---- S1c (i): the op stream ------------------------------------------------

/// **A body under construction, as the ops it is made of rather than the bytes
/// they render to** — slice S1's `Vec<MachineOp>`.
///
/// # Why this type exists at all, when it is a `Vec`
///
/// Before S1c a producer's only output was a `Vec<u8>`: `extend_from_slice`
/// called on the result of an `encode_*`, word after word, with **no
/// intermediate representation at all**. `w-s1` §6 surveyed it and found 16 of
/// the 18 `Selected::plain` producers built bytes that way. Once the words are
/// bytes, the opcode is gone — nothing downstream can ask what instruction a
/// body contains, only what it looks like.
///
/// An `Ops` keeps [`C2Op`], **c2's own opcode number** (S1a, read R2), alive
/// until the last possible moment. That is what the goal's two named consumers
/// need (`GOAL_DECISION_2026-08-21.md` § AMENDED): a permuter searching the
/// port's decision points, and matching-pretext generation, both want the
/// decisions, not their rendering.
///
/// # What it does NOT claim
///
/// This is **not** `block_ir`, and wiring `Plain` through `block_ir` is a
/// **different construct rung** — the correction `w-s1` §6 filed against board
/// #3365, whose *"13 of the 35 shape arms already share `block_ir`"* is right
/// about the 13 but counts only **2** on a `Selected::plain` arm. Nor is it
/// c2's machine tuple: `P_ENCODE.md` §9's bound stands, the tuple stream is
/// read R5, and these are the **port's own** operands.
pub type Ops = Vec<MachineOp>;

/// Render an op stream to the bytes the two dispatchers consume.
///
/// The single boundary at which an `Ops` becomes a `Vec<u8>`. Every op composes
/// through [`MachineOp::word`], i.e. through [`encode_op`] at
/// [`EncodeParams::C2`] — **the default and nowhere else**, which is where
/// grading happens.
#[inline]
pub fn ops_to_bytes(ops: &[MachineOp]) -> Vec<u8> {
    let mut t = Vec::with_capacity(ops.len() * 4);
    for m in ops {
        t.extend_from_slice(&m.word());
    }
    t
}

#[cfg(test)]
mod ops_tests {
    use super::*;
    use crate::codegen::encode::{encode_addis, encode_blr, encode_stw};

    /// **The renderer is exactly concatenated `encode_*` output** — the
    /// property that makes every S1c producer conversion required-zero by
    /// construction rather than by measurement.
    #[test]
    fn ops_to_bytes_is_concatenated_encoder_output() {
        let ops: Ops = vec![
            crate::codegen::encode::mop_addis(11, 0, 0),
            crate::codegen::encode::mop_stw(3, 11, 0),
            crate::codegen::encode::mop_blr(),
        ];
        let mut want = Vec::new();
        want.extend_from_slice(&encode_addis(11, 0, 0));
        want.extend_from_slice(&encode_stw(3, 11, 0));
        want.extend_from_slice(&encode_blr());
        assert_eq!(ops_to_bytes(&ops), want);
    }

    /// An empty stream renders to no bytes — the live case, not a curiosity:
    /// `BodyShape::Tail` with an empty `ops` stream is a VOID tail call whose
    /// whole body is the terminator branch, and `splice.rs` reads that
    /// emptiness as a semantic stratum (SPLICE-P, 0 of 953).
    #[test]
    fn an_empty_op_stream_renders_to_no_bytes() {
        let ops: Ops = Vec::new();
        assert_eq!(ops_to_bytes(&ops), Vec::<u8>::new());
        assert!(ops_to_bytes(&ops).is_empty());
    }

    // ---- S1c (i), lane w-s1c2: the domain sweep, widened from 22 to ALL 85 --
    //
    // `w-s1bc` shipped this sweep over the 22 encoders it had converted. It is
    // the control that makes the encoder rewrite's required-zero byte delta
    // CHECKABLE rather than asserted, so leaving it at 22 while the population
    // went to 85 would have been a green control over a quarter of its
    // population — `STATUS.md` trap 0 exactly.
    //
    // The macros exist so that adding an encoder is one name in one list, and
    // so a failure names the FUNCTION (`stringify!`) rather than a line number.
    // They are `macro_rules!`, i.e. std only; nothing here adds a dependency.

    /// `mop_X(a,b,c).word() == encode_X(a,b,c)` over all 32^3 register triples.
    macro_rules! agree_rrr {
        ($($mop:ident / $enc:ident),* $(,)?) => {
            for a in 0..32u8 {
                for b in 0..32u8 {
                    for c in 0..32u8 {
                        $(assert_eq!(
                            e::$mop(a, b, c).word(), e::$enc(a, b, c),
                            concat!(stringify!($mop), " disagrees at ({},{},{})"), a, b, c
                        );)*
                    }
                }
            }
        };
    }

    /// Two registers and a 16-bit signed displacement.
    macro_rules! agree_rrd {
        ($disps:expr; $($mop:ident / $enc:ident),* $(,)?) => {
            for a in 0..32u8 {
                for b in 0..32u8 {
                    for d in $disps {
                        $(assert_eq!(
                            e::$mop(a, b, d).word(), e::$enc(a, b, d),
                            concat!(stringify!($mop), " disagrees at ({},{},{})"), a, b, d
                        );)*
                    }
                }
            }
        };
    }

    /// Two registers and a 16-bit unsigned immediate.
    macro_rules! agree_rru {
        ($($mop:ident / $enc:ident),* $(,)?) => {
            for a in 0..32u8 {
                for b in 0..32u8 {
                    for u in [0u16, 1, 0x7fff, 0x8000, 0xffff] {
                        $(assert_eq!(
                            e::$mop(a, b, u).word(), e::$enc(a, b, u),
                            concat!(stringify!($mop), " disagrees at ({},{},{})"), a, b, u
                        );)*
                    }
                }
            }
        };
    }

    /// Two registers.
    macro_rules! agree_rr {
        ($($mop:ident / $enc:ident),* $(,)?) => {
            for a in 0..32u8 {
                for b in 0..32u8 {
                    $(assert_eq!(
                        e::$mop(a, b).word(), e::$enc(a, b),
                        concat!(stringify!($mop), " disagrees at ({},{})"), a, b
                    );)*
                }
            }
        };
    }

    /// A `double` flag and three registers — the A-form FP family, where the
    /// flag selects between two OPCODE ROWS (`fadd`/`fadds`) rather than
    /// between two operand values, which is why it is swept and not fixed.
    macro_rules! agree_frrr {
        ($($mop:ident / $enc:ident),* $(,)?) => {
            for dbl in [false, true] {
                for a in 0..32u8 {
                    for b in 0..32u8 {
                        for c in 0..32u8 {
                            $(assert_eq!(
                                e::$mop(dbl, a, b, c).word(), e::$enc(dbl, a, b, c),
                                concat!(stringify!($mop), " disagrees at ({},{},{},{})"),
                                dbl, a, b, c
                            );)*
                        }
                    }
                }
            }
        };
    }

    /// **The register-register families — 26 opcodes over their whole 32^3
    /// domain.**
    ///
    /// The population deliberately mixes forms 49, 39, 36, 41, 14, 26/50 and
    /// 28/61 in one sweep, because the thing most likely to be wrong is a
    /// **field role**, not an opcode: form 49 puts the destination at bit 21
    /// and form 39 at bit 16, and `P_ENCODE.md` §5.1 calls that the single most
    /// safety-critical fact on the page. Sweeping all three registers
    /// independently is what makes the swap visible — at `a == b == c` every
    /// layout agrees.
    #[test]
    fn every_register_register_mop_agrees_with_its_encoder() {
        use crate::codegen::encode as e;
        agree_rrr!(
            mop_add / encode_add,
            mop_adde / encode_adde,
            mop_subf / encode_subf,
            mop_subfc / encode_subfc,
            mop_subfe / encode_subfe,
            mop_mullw / encode_mullw,
            mop_divw / encode_divw,
            mop_divwu / encode_divwu,
            mop_and / encode_and,
            mop_andc / encode_andc,
            mop_or / encode_or,
            mop_orc / encode_orc,
            mop_eqv / encode_eqv,
            mop_xor / encode_xor,
            mop_slw / encode_slw,
            mop_srw / encode_srw,
            mop_sraw / encode_sraw,
            mop_srawi / encode_srawi,
            mop_clrlwi_record / encode_clrlwi_record,
            mop_lwzx / encode_lwzx,
            mop_lhzx / encode_lhzx,
            mop_lfsx / encode_lfsx,
            mop_stdx / encode_stdx,
            mop_stfsx / encode_stfsx,
            mop_cmpw / encode_cmpw,
            mop_cmplw / encode_cmplw,
        );
    }

    /// **The D-form families — two registers and a signed displacement.**
    ///
    /// The displacement set spans both signs, both 16-bit boundaries and a
    /// value that is **not** a multiple of 4, which is the one the DS-form
    /// opcodes (`ld`, `std`, `ldr`) are about: their low two bits are the form
    /// selector, so a 14-bit `disp >> 2` and a 16-bit `disp` are different
    /// functions and only disagree off a word boundary.
    ///
    /// **`stdu` is swept on multiples of 4 alone**, and that is a property of
    /// the encoder rather than a gap: it carries a `debug_assert_eq!(ds & 3, 0)`
    /// that fires in the portable (debug) lane. Sweeping it off a word boundary
    /// would test the assertion, not the encoding.
    #[test]
    fn every_d_form_mop_agrees_with_its_encoder() {
        use crate::codegen::encode as e;
        const D: [i16; 8] = [0, 1, -1, 4, -4, 6, i16::MAX, i16::MIN];
        const D4: [i16; 6] = [0, 4, -4, 8, -8, -32768];
        agree_rrd!(D;
            mop_addi / encode_addi,
            mop_addic / encode_addic,
            mop_addic_record / encode_addic_record,
            mop_addis / encode_addis,
            mop_mulli / encode_mulli,
            mop_subfic / encode_subfic,
            mop_lwz / encode_lwz,
            mop_lbz / encode_lbz,
            mop_lbzu / encode_lbzu,
            mop_lhz / encode_lhz,
            mop_ld / encode_ld,
            mop_ldr / encode_ldr,
            mop_lfd / encode_lfd,
            mop_stw / encode_stw,
            mop_stwu / encode_stwu,
            mop_stb / encode_stb,
            mop_sth / encode_sth,
            mop_sthu / encode_sthu,
            mop_std / encode_std,
            mop_stfd / encode_stfd,
            mop_stfsu / encode_stfsu,
            mop_twi / encode_twi,
            mop_cmpwi / encode_cmpwi,
        );
        agree_rrd!(D4; mop_stdu / encode_stdu);
    }

    /// The logical-immediate and compare-immediate forms, whose field is
    /// **unsigned and ORed unmasked** — so the sweep includes `0xffff` and the
    /// sign-bit boundary rather than small values only.
    #[test]
    fn every_immediate_mop_agrees_with_its_encoder() {
        use crate::codegen::encode as e;
        agree_rru!(
            mop_ori / encode_ori,
            mop_xori / encode_xori,
            mop_cmplwi / encode_cmplwi,
        );
    }

    /// The two-register forms, including the four that are `rlwinm` wrappers
    /// with their masks baked (`srwi31`, `clrlwi31`) and the `mr`/`fmr` moves.
    #[test]
    fn every_two_register_mop_agrees_with_its_encoder() {
        use crate::codegen::encode as e;
        agree_rr!(
            mop_addze / encode_addze,
            mop_subfze / encode_subfze,
            mop_neg / encode_neg,
            mop_extsb / encode_extsb,
            mop_extsb_record / encode_extsb_record,
            mop_extsh / encode_extsh,
            mop_cntlzw / encode_cntlzw,
            mop_srwi31 / encode_srwi31,
            mop_clrlwi31 / encode_clrlwi31,
            mop_mr / encode_mr,
            mop_mr_record / encode_mr_record,
            mop_fmr / encode_fmr,
            mop_frsp / encode_frsp,
            mop_bclr / encode_bclr,
        );
    }

    /// The A-form floating-point family, over the `double` flag AND all three
    /// registers.
    ///
    /// `fmul` is the one worth having in a sweep rather than a spot check: it
    /// is form **23**, not form 22, because its multiplier lands in the **C**
    /// field at bit 6 and not in B at bit 11. Reusing form 22 for it multiplies
    /// by the wrong register and still assembles.
    #[test]
    fn every_fp_a_form_mop_agrees_with_its_encoder() {
        use crate::codegen::encode as e;
        agree_frrr!(
            mop_fadd / encode_fadd,
            mop_fsub / encode_fsub,
            mop_fmul / encode_fmul,
            mop_fdiv / encode_fdiv,
        );
        // The single-precision loads/stores take the same flag with a
        // displacement instead of a third register.
        for dbl in [false, true] {
            for a in 0..32u8 {
                for b in 0..32u8 {
                    for d in [0i16, 1, -1, 4, -4, i16::MAX, i16::MIN] {
                        assert_eq!(e::mop_lfs(dbl, a, b, d).word(), e::encode_lfs(dbl, a, b, d));
                        assert_eq!(e::mop_stfs(dbl, a, b, d).word(), e::encode_stfs(dbl, a, b, d));
                    }
                }
            }
        }
    }

    /// **The rotates, over their own split immediate fields.**
    ///
    /// The 32-bit rotates take `SH`, `MB` and `ME` as three successive 5-bit
    /// operands and are swept exhaustively over all three.
    ///
    /// **The 64-bit rotates are swept 0..64, not 0..32, and that is a widening
    /// rather than a restatement.** `rldicl`/`rldimi` are form 68, whose `SH`
    /// is **split** — `SH[4:0]` at bit 11 with `SH[5]` alone at bit 1 — and
    /// whose `MB` is stored low-bit-first as `(MB & 0x1f) << 1 | (MB >> 5)`.
    /// The incumbent sweep ran 0..32, so **every value it tried had bit 5
    /// clear** and neither split's high bit was ever exercised. A sweep that
    /// cannot reach the field it is checking is #3379's 46-word probe again,
    /// one form over.
    #[test]
    fn every_rotate_mop_agrees_with_its_encoder() {
        use crate::codegen::encode as e;
        for &ra in &[0u8, 3, 31] {
            for &rs in &[0u8, 4, 31] {
                for sh in 0..32u8 {
                    for mb in 0..32u8 {
                        for me in 0..32u8 {
                            assert_eq!(
                                e::mop_rlwinm(ra, rs, sh, mb, me).word(),
                                e::encode_rlwinm(ra, rs, sh, mb, me)
                            );
                            assert_eq!(
                                e::mop_rlwinm_record(ra, rs, sh, mb, me).word(),
                                e::encode_rlwinm_record(ra, rs, sh, mb, me)
                            );
                            assert_eq!(
                                e::mop_rlwimi(ra, rs, sh, mb, me).word(),
                                e::encode_rlwimi(ra, rs, sh, mb, me)
                            );
                        }
                    }
                }
                // form 68's two 6-bit split fields, over their WHOLE range.
                for sh in 0..64u8 {
                    for mb in 0..64u8 {
                        assert_eq!(
                            e::mop_rldicl(ra, rs, sh, mb).word(),
                            e::encode_rldicl(ra, rs, sh, mb),
                            "mop_rldicl disagrees at sh={sh} mb={mb}"
                        );
                        assert_eq!(
                            e::mop_rldimi(ra, rs, sh, mb).word(),
                            e::encode_rldimi(ra, rs, sh, mb),
                            "mop_rldimi disagrees at sh={sh} mb={mb}"
                        );
                    }
                }
            }
        }
    }

    /// **The fallible branch encoders — the refusal is part of the agreement.**
    ///
    /// `bdnz`, `bc` and `b_intra` return `Option`, and S1c gives them an
    /// `Option<MachineOp>` twin rather than an infallible one. The property is
    /// therefore not "the words agree" but "**the `Option`s agree**": a twin
    /// that encoded an out-of-range displacement instead of refusing it would
    /// pass a word comparison over the in-range domain and be silently wrong
    /// exactly where the range check exists to help.
    ///
    /// So the sweep spans **both sides of both limits** and every displacement
    /// that is not a multiple of 4, and asserts `None == None` as hard as it
    /// asserts the words.
    #[test]
    fn every_fallible_branch_mop_agrees_with_its_encoder_including_its_refusals() {
        use crate::codegen::encode as e;
        let disps: [i32; 20] = [
            0, 4, -4, 8, -8, 1, -1, 2, -2, 3,
            0x7ffc, -0x8000, 0x8000, -0x8004,
            0x1ff_fffc, -0x200_0000, 0x200_0000, -0x200_0004,
            i32::MAX, i32::MIN,
        ];
        let mut refused = 0usize;
        for d in disps {
            assert_eq!(
                e::mop_bdnz(d).map(|m| m.word()), e::encode_bdnz(d),
                "mop_bdnz disagrees at {d}"
            );
            assert_eq!(
                e::mop_b_intra(d).map(|m| m.word()), e::encode_b_intra(d),
                "mop_b_intra disagrees at {d}"
            );
            for bo in 0..32u8 {
                for bi in 0..32u8 {
                    assert_eq!(
                        e::mop_bc(bo, bi, d).map(|m| m.word()), e::encode_bc(bo, bi, d),
                        "mop_bc disagrees at ({bo},{bi},{d})"
                    );
                }
            }
            if e::mop_b_intra(d).is_none() {
                refused += 1;
            }
        }
        // The sweep REACHED the refusing arm — without this the test could pass
        // by never having asked a question either side could refuse, which is
        // the "green over the population it ran on" failure one level down.
        assert!(refused > 0, "no displacement in the sweep was refused");
    }

    /// The nullary and unary encoders. `mtctr` is here rather than with the
    /// two-register forms because its second field is not a register at all: it
    /// is SPR 9, written **low half first**, so an unsplit twin would name SPR
    /// 288 and still assemble.
    #[test]
    fn every_nullary_and_unary_mop_agrees_with_its_encoder() {
        use crate::codegen::encode as e;
        assert_eq!(e::mop_blr().word(), e::encode_blr());
        assert_eq!(e::mop_bctrl().word(), e::encode_bctrl());
        for a in 0..32u8 {
            assert_eq!(e::mop_mtctr(a).word(), e::encode_mtctr(a), "mop_mtctr at {a}");
        }
    }

    /// **The population control: every `encode_*` in the module has a `mop_*`
    /// twin, and the count is asserted rather than described.**
    ///
    /// Without this, the sweeps above are green over whatever subset somebody
    /// remembered to list — which is exactly how the incumbent stayed green
    /// over 22 of 85. This cannot see a twin that exists and is unswept, so it
    /// is a floor on the population and not a proof of coverage; it is checked
    /// against the module source, which is the only place the truth is.
    #[test]
    fn the_encoder_and_mop_populations_are_the_same_size() {
        let src = include_str!("encode.rs");
        let mut encoders = 0usize;
        let mut mops = 0usize;
        for line in src.lines() {
            let l = line.trim_start_matches("pub ").trim_start_matches("pub(crate) ");
            if !l.starts_with("fn ") || !line.starts_with("pub") && !line.starts_with("fn ") {
                continue;
            }
            let name = &l[3..];
            if name.starts_with("encode_") {
                encoders += 1;
            } else if name.starts_with("mop_") {
                mops += 1;
            }
        }
        assert_eq!(
            encoders, mops,
            "encode.rs has {encoders} encoders and {mops} mop_* twins — S1c (i) \
             requires one twin per encoder"
        );
        // The absolute number, so that a DROP of both is not silently green.
        assert_eq!(encoders, 85, "the encoder population moved; update the sweeps too");
    }
}
