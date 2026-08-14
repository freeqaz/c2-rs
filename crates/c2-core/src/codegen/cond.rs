//! The condition-code model — `docs/CFG_SHAPE.md` §6.2 item **E**, the
//! **producer** side.
//!
//! > **E. A condition-code model with two producers.** The IR must distinguish
//! > *"compare X against Y into cr6"* from *"this arithmetic instruction's
//! > record form sets cr0"*, because §3.2 shows c2 branches on both and the `BI`
//! > field differs (26 vs 2). A model with a single implicit condition register
//! > emits wrong bytes for every decrement-and-test loop.
//!
//! # What was already here, and what was missing
//!
//! [`super::block_ir::Terminator::Bc`] (item **A**, board **#3072**) already
//! carries a raw `(BO, BI)` pair, so the *consumer* side of the distinction was
//! **carried** — `BI = 26` reads cr6, `BI = 2` reads cr0. What no type could say
//! was **which instruction wrote it**. That is a property of a block's
//! straight-line instruction run, and `BasicBlock::body` is where such a run
//! first became a thing that exists. This module is that property.
//!
//! # The fact that shapes the type, found in this crate's own shipped bytes
//!
//! §6.2 item E's sentence pairs a producer with a field — *compare → cr6*,
//! *record form → cr0* — and the second half is an architectural certainty
//! while **the first half is not a rule at all**. Two classes in this crate are
//! byte-exact against real `c2.dll` and emit an explicit compare into **cr0**:
//!
//! * [`super::close_call_chain`] — `RESULT_CRF = 0`: *"`cmplwi` with no explicit
//!   field is `cr0`, and that is not a spelling difference: the guard's word is
//!   `2b030000` and these are `28030000`"*;
//! * [`super::alloc_init_or_fail`] — `CR_MIDDLE = 0`: the middle of three tests
//!   reads cr0 where the outer two read cr6, and *"nothing in the source
//!   distinguishes them"*.
//!
//! So a model that mapped `Compare ⇒ cr6` would emit `2b03…`/`409a…` where those
//! objs carry `2803…`/`4082…` — the same two-byte, still-disassembles failure
//! class board **#188** names, arrived at from the other side. Hence
//! [`CondProducer::Compare`] **carries** its `crf`: an explicit compare *names*
//! its field, in its own `BF` bits, and the model reads it rather than assuming
//! it. [`CondProducer::RecordForm`] carries nothing, because a record form
//! cannot name a field: it writes cr0 or it is not a record form.
//!
//! # One reader, replacing six
//!
//! Before this module the crate held **six** private readers of "which CR field
//! does this producer write" — `encode::CR_COMPARE` (6), two separate
//! `const CR_RECORD: u8 = 0` (in [`super::ptr_walk_loop`] and
//! [`super::ptr_walk_chain_loop`]), `close_call_chain::{GUARD_CRF, RESULT_CRF}`,
//! `alloc_init_or_fail::CR_MIDDLE`, `guard_ret_chain::GUARD_CRF` — plus two
//! sites spelling the fact as a bare `cr_bi(0, …)` with no name at all. Two
//! encodings of one fact is the shape `docs/GAPS.md` §6 keeps recording. The two
//! exact duplicates (`CR_RECORD`) are gone, replaced by
//! [`CondProducer::RecordForm`]; `encode::CR_COMPARE` stays exactly where it is
//! and is **used** by [`CondProducer::compare`], never restated.
//!
//! # What this module does NOT do
//!
//! It does not choose a producer, schedule one, or decide where a compare goes:
//! that is instruction selection, and `CFG_SHAPE.md` §6.3 plus lane
//! `w-dagorder`'s finding (item **F** is a cycle-driven list scheduler) put it
//! out of scope. It emits **no bytes at all** — every word it looks at was
//! produced by an encoder in [`super::encode`] that is already byte-graded by a
//! shipped lowering. It is a *reader* of an instruction run and a *speller* of
//! the `(BO, BI)` pair a branch needs, and nothing else.

use super::encode::{cr_bi, CR_COMPARE};
use super::select::out_of_class;
use crate::BackendError;

/// **The instruction that wrote the condition bits a branch reads** — §3.2's
/// two producers, and no third.
///
/// c2 branches on both, and the branch word differs in `BI` (26 vs 2) with
/// nothing else to tell them apart, which is why this is a modelled
/// distinction and not a comment.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CondProducer {
    /// An **explicit compare** — `cmpw`/`cmpwi`/`cmplw`/`cmplwi`.
    ///
    /// It **names** the field it writes, in its own `BF` bits, so the field is
    /// carried here rather than assumed: §3.2 measures cr6 in every cell it
    /// covers, and two byte-exact classes in this crate compare into cr0. See
    /// the module header.
    Compare { crf: u8 },
    /// A **record form** — `addic.`, `mr.`, `extsb.`, `rlwinm.`: an arithmetic
    /// or logical instruction with `Rc = 1`.
    ///
    /// It writes **cr0** and can write nothing else — there is no field to
    /// carry. c2 branches on it with no intervening compare (`?c_do`, `?c_callloop`,
    /// `?d_break`: `addic. r11,r11,-1` then `4082fff8`).
    RecordForm,
}

/// **cr0** — the condition-register field a record form writes, and the field a
/// compare written with no explicit field encodes.
///
/// One constant, because it is one architectural field reached two ways, and
/// this project has already paid for spelling it twice: `ptr_walk_loop` and
/// `ptr_walk_chain_loop` each carried a private `const CR_RECORD: u8 = 0`, and
/// **`ptr_walk_loop` used its copy for a `cmplwi` as well as for a `mr.`** — the
/// same number standing for two different producers under a name that claims
/// only one of them. Board **#188** is the defect the name exists to prevent: a
/// lowering that hard-codes `4*6 + bit` emits `409a…` where the obj has `4082…`
/// for every decrement-and-test loop.
///
/// It is deliberately **not** named `CR_RECORD`. A record form's field and a
/// fieldless compare's field are the same register, and a name that says
/// "record" makes the second use look like a mistake — which is how it read in
/// `ptr_walk_loop`'s entry guard for the whole of that class's life.
pub const CR0: u8 = 0;

impl CondProducer {
    /// The compare producer of §3.2 — the one that writes
    /// [`CR_COMPARE`](super::encode::CR_COMPARE), cr6.
    ///
    /// Delegates to `encode::CR_COMPARE` rather than restating `6`: that
    /// constant is the existing reader of "the field an explicit compare
    /// feeding a branch writes", and its doc carries the `?b_ifn` measurement
    /// (three compares in one body, all cr6, reused rather than allocated).
    pub fn compare() -> Self {
        CondProducer::Compare { crf: CR_COMPARE }
    }

    /// A compare into a **named** field — the form the two cr0-comparing
    /// classes need. `compare_into(CR_COMPARE)` and [`Self::compare`] are the
    /// same value.
    pub fn compare_into(crf: u8) -> Self {
        CondProducer::Compare { crf }
    }

    /// The condition-register field this producer wrote.
    pub fn crf(self) -> u8 {
        match self {
            CondProducer::Compare { crf } => crf,
            CondProducer::RecordForm => CR0,
        }
    }

    /// What this producer is, for a refusal that has to name it. Diagnostic
    /// only — a refusal that cannot say *which* producer it is about is one
    /// somebody has to re-derive.
    pub fn what(self) -> &'static str {
        match self {
            CondProducer::Compare { .. } => "an explicit compare",
            CondProducer::RecordForm => "a record form",
        }
    }
}

/// **A condition a branch reads**: a producer, plus which of that producer's
/// four bits and in which sense.
///
/// `bo` and `bit` are the raw PowerPC fields — `BO_TRUE`/`BO_FALSE` and
/// `CR_BIT_LT`/`GT`/`EQ` from [`super::encode`] spell them, and
/// [`super::cond_tail::branch_sense`] is the **existing** reader of "which
/// `(BO, bit)` pair an IL relation becomes". That predicate is used here and
/// deliberately not restated: it returns `(BO, bit)` and documents *"the caller
/// adds the CR field"* — this type is the caller it was waiting for.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Cond {
    producer: CondProducer,
    bo: u8,
    bit: u8,
}

impl Cond {
    /// A condition read from `producer`, testing `bit` in the sense `bo`.
    pub fn new(producer: CondProducer, bo: u8, bit: u8) -> Self {
        Cond { producer, bo, bit }
    }

    /// A condition from an explicit compare into cr6 — §3.2's measured default.
    pub fn compare(bo: u8, bit: u8) -> Self {
        Cond::new(CondProducer::compare(), bo, bit)
    }

    /// A condition from a record form. cr0, and the `BI` follows.
    pub fn record_form(bo: u8, bit: u8) -> Self {
        Cond::new(CondProducer::RecordForm, bo, bit)
    }

    /// Which instruction wrote it.
    pub fn producer(self) -> CondProducer {
        self.producer
    }

    /// The `BO` field — the sense.
    pub fn bo(self) -> u8 {
        self.bo
    }

    /// The bit within the field: `CR_BIT_LT` / `GT` / `EQ`.
    pub fn bit(self) -> u8 {
        self.bit
    }

    /// The condition-register field, from the **producer**. This is the whole
    /// point of the type.
    pub fn crf(self) -> u8 {
        self.producer.crf()
    }

    /// The `BI` field — `4*crf + bit`, **via [`cr_bi`]**, which is the existing
    /// reader of that arithmetic and stays the only one.
    pub fn bi(self) -> u8 {
        cr_bi(self.crf(), self.bit)
    }
}

/// The `BO` bit that means **"ignore the condition register"**.
///
/// PowerPC `BO` bit 0 (`0x10`). It is set in `BO = 20` ([`BO_ALWAYS`], what
/// makes a `bclr` a plain `blr`) and in `BO = 16` ([`BO_DNZ`], the CTR loop of
/// §3.7, which tests CTR and no CR bit); it is clear in `BO = 12`/`BO = 4`
/// ([`BO_TRUE`]/[`BO_FALSE`]), the two §3.1 tabulates for a branch on a
/// compare. A branch with it set reads **no** condition register, so asking
/// which field such a branch reads has no answer rather than the answer `BI/4`.
///
/// [`BO_ALWAYS`]: super::encode::BO_ALWAYS
/// [`BO_DNZ`]: super::encode::BO_DNZ
/// [`BO_TRUE`]: super::encode::BO_TRUE
/// [`BO_FALSE`]: super::encode::BO_FALSE
pub const BO_IGNORES_CR: u8 = 0x10;

/// The condition-register **field a branch reads**, or `None` if it reads none.
///
/// `BI = 4*crf + bit` (§3.1), so the field is `BI >> 2` — but only for a `BO`
/// that actually consults the CR. `blr` is `bclr` at `BO = 20, BI = 0` and
/// `BI >> 2 = 0` there is an artefact, not a claim that a return reads cr0.
pub fn bc_reads_crf(bo: u8, bi: u8) -> Option<u8> {
    if bo & BO_IGNORES_CR != 0 {
        None
    } else {
        Some(bi >> 2)
    }
}

/// What one instruction word does to the condition register.
///
/// **Four-valued on purpose.** [`Unmodelled`](Self::Unmodelled) is a distinct
/// answer from [`Untouched`](Self::Untouched): "this model does not know what
/// that word does" must never be read as "that word writes no CR field", which
/// is the absence-is-not-evidence rule applied to a decoder. The vocabulary
/// recognised is **exactly what [`super::encode`] emits** — decoding a
/// production is not licence to emit it (`IL_STMT_GRAMMAR.md` §14.2), and the
/// converse discipline applies here: claiming to decode a word this crate never
/// produces would be a claim with no witness behind it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CrEffect {
    /// An explicit compare, writing the field it names in its `BF` bits.
    Compare { crf: u8 },
    /// A record form (`Rc = 1`), writing cr0.
    Record,
    /// Recognised, and writes no condition-register field.
    Untouched,
    /// **Not in this model's vocabulary.** A scan that reaches one of these has
    /// no answer, and says so.
    Unmodelled,
}

/// Primary opcodes this crate emits that write **no** condition-register field
/// and carry no `Rc` bit.
///
/// Read off [`super::encode`]'s own encoders, one entry per `(N << 26)` it
/// builds — `twi` 3, `mulli` 7, `subfic` 8, `addic` 12, `addi` 14, `addis` 15,
/// `rlwimi` 20, `ori` 24, `xori` 26, the loads and stores (32 `lwz`, 34 `lbz`,
/// 35 `lbzu`, 36 `stw`, 37 `stwu`, 38 `stb`, 40 `lhz`, 44 `sth`, 45 `sthu`,
/// 58 `ld`, 62 `std`/`stdu`) and the FP loads and stores (48/50 `lfs`/`lfd`,
/// 52/53/54 `stfs`/`stfsu`/`stfd`).
///
/// **58 and 62 are the reason this is a list and not a rule.** They are DS-form:
/// their low two bits are an extended opcode, so `stdu` sets bit 31 while having
/// nothing to do with `Rc`. A decoder that tested bit 31 across the board would
/// call `stdu` a record form and hand a `bclr` cr0 that nothing wrote.
const SILENT_PRIMARIES: [u32; 25] = [
    3, 7, 8, 12, 14, 15, 20, 24, 26, 32, 34, 35, 36, 37, 38, 40, 44, 45, 48, 50, 52, 53, 54, 58, 62,
];

/// Primary opcodes this crate emits whose **bit 31 is `Rc`**, so a set bit means
/// a record form writing cr0.
///
/// `rlwinm`/`rlwinm.` (21), the 64-bit rotates (30 — `rldicl`/`rldimi`, emitted
/// only with `Rc = 0`), and the big X/XO-form family at 31.
const RC_PRIMARIES: [u32; 3] = [21, 30, 31];

/// The extended opcode of `cmp` (`cmpw`/`cmpd`) — primary 31.
const XO_CMP: u32 = 0;
/// The extended opcode of `cmpl` (`cmplw`/`cmpld`) — primary 31.
const XO_CMPL: u32 = 32;

/// What a single big-endian PowerPC word does to the condition register.
///
/// The three cases that matter, and each is a measured form:
///
/// | word | effect | witness |
/// |---|---|---|
/// | `2b 03 00 00` `cmplwi cr6,r3,0` | `Compare { crf: 6 }` | `?MemFree`, §4.1 |
/// | `28 03 00 00` `cmplwi cr0,r3,0` | `Compare { crf: 0 }` | `close_call_chain`'s result test |
/// | `35 6b ff ff` `addic. r11,r11,-1` | `Record` | `?c_do`, §3.2 |
pub fn cr_effect(word: [u8; 4]) -> CrEffect {
    let w = u32::from_be_bytes(word);
    let primary = w >> 26;
    let rc = w & 1 != 0;
    // `BF` — the field an explicit compare names. Bits 6..8 of the word, which
    // is `(w >> 23) & 7`.
    let bf = ((w >> 23) & 7) as u8;
    match primary {
        // `cmpli` (`cmplwi`) and `cmpi` (`cmpwi`) — D-form compares.
        10 | 11 => CrEffect::Compare { crf: bf },
        // `addic.` — D-form, and record by definition: there is no `addic.`
        // with `Rc = 0`, that spelling is primary 12.
        13 => CrEffect::Record,
        p if RC_PRIMARIES.contains(&p) => {
            let xo = (w >> 1) & 0x3FF;
            if p == 31 && (xo == XO_CMP || xo == XO_CMPL) {
                // `cmpw`/`cmplw` — X-form compares. Bit 31 is reserved and
                // zero here, so they must be taken before the `Rc` test.
                CrEffect::Compare { crf: bf }
            } else if rc {
                CrEffect::Record
            } else {
                CrEffect::Untouched
            }
        }
        p if SILENT_PRIMARIES.contains(&p) => CrEffect::Untouched,
        // Everything else — including the branch family (16 `bc`, 18 `b`/`bl`,
        // 19 `bclr`/`bcctr`) and the FP arithmetic (59/63, whose `Rc` writes
        // **cr1**, a third field this model does not carry). A call is the
        // load-bearing one: the volatile CR fields do not survive it, so
        // calling a `bl` "untouched" would let a scan walk past it and name a
        // producer whose bits are gone.
        _ => CrEffect::Unmodelled,
    }
}

/// Where the condition a block's terminator reads was produced.
///
/// Three-valued for the same reason [`CrEffect`] is four-valued:
/// [`NotInThisBlock`](Self::NotInThisBlock) is a **positive** finding (the whole
/// run was read and none of it writes a CR field), while
/// [`Unknown`](Self::Unknown) is the absence of one. A check that treated them
/// alike would silently stop checking the day a lowering emitted a word this
/// model does not model.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CondSource {
    /// The nearest condition-register writer in this block's own instruction
    /// run, reading backwards from the terminator.
    InBlock(CondProducer),
    /// The whole run was read and **nothing in it** writes a condition-register
    /// field. The producer, if there is one, is in a predecessor block — which
    /// is legal and common (§3.4.1's `?d_join`; a guard chain's second test).
    NotInThisBlock,
    /// A word this model does not model was reached before any writer, so the
    /// question has no answer here. Never read as [`Self::NotInThisBlock`].
    Unknown,
}

/// Scan a block's straight-line instruction run **backwards** for the producer
/// of the condition its terminator reads.
///
/// Backwards because the last writer wins: `?b_ifn` writes cr6 three times in
/// one body, *"each branch consuming its own before the next is issued"*
/// (§3.2). A forward scan would name the first.
///
/// A run whose length is not a whole number of words is [`CondSource::Unknown`]
/// — not an error, because refusing a ragged block is
/// [`super::block_ir::BodyLayout::place`]'s job and already has a test; this
/// function's contract is only that it never invents an answer.
pub fn cond_source(run: &[u8]) -> CondSource {
    if run.len() % 4 != 0 {
        return CondSource::Unknown;
    }
    for word in run.chunks_exact(4).rev() {
        let w: [u8; 4] = [word[0], word[1], word[2], word[3]];
        match cr_effect(w) {
            CrEffect::Compare { crf } => {
                return CondSource::InBlock(CondProducer::Compare { crf })
            }
            CrEffect::Record => return CondSource::InBlock(CondProducer::RecordForm),
            CrEffect::Untouched => {}
            CrEffect::Unmodelled => return CondSource::Unknown,
        }
    }
    CondSource::NotInThisBlock
}

/// The producer of the condition a branch at the **end** of `run` reads, or a
/// refusal that names the site.
///
/// The one reader of "turn a scan into a producer or refuse", so that a lowering
/// asking the question does not each write its own three-armed `match` — three
/// copies of one rule is how [`CondProducer::RecordForm`]'s field came to be
/// spelled twice in the first place. `site` appears in the refusal only.
///
/// **Both non-answers refuse**, and they refuse for different reasons that the
/// message keeps apart: a run with no writer means the producer is in a
/// predecessor block, which is legal in general but means *this* caller cannot
/// derive a field from what it holds; an unmodelled word means the scan has no
/// answer at all. Neither is grounds for assuming cr6.
pub fn producer_at(run: &[u8], site: &str) -> Result<CondProducer, BackendError> {
    match cond_source(run) {
        CondSource::InBlock(p) => Ok(p),
        CondSource::NotInThisBlock => Err(out_of_class(&format!(
            "{site}: nothing in the instruction run before this branch writes a \
             condition register, so its CR field would have to be assumed — and \
             CFG_SHAPE.md §3.2's two producers write different ones"
        ))),
        CondSource::Unknown => Err(out_of_class(&format!(
            "{site}: the instruction run before this branch reaches a word this \
             condition model does not model before it reaches a producer, so the \
             branch's CR field is unknown rather than assumed"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::encode::{
        encode_add, encode_addi, encode_addic, encode_addic_record, encode_bc, encode_bctrl,
        encode_blr, encode_clrlwi_record, encode_cmplw, encode_cmplwi, encode_cmpw, encode_cmpwi,
        encode_extsb, encode_extsb_record, encode_lbz, encode_lbzu, encode_ld, encode_lwz,
        encode_mr, encode_mr_record, encode_mulli, encode_mullw, encode_rlwinm,
        encode_rlwinm_record, encode_stb, encode_std, encode_stdu, encode_stdx, encode_stfd,
        encode_stw, encode_twi, BO_ALWAYS, BO_DNZ, BO_FALSE, BO_TRUE, CR_BIT_EQ, CR_BIT_LT,
        CR_COMPARE,
    };

    fn run(words: &[[u8; 4]]) -> Vec<u8> {
        words.iter().flatten().copied().collect()
    }

    /// **The known-answer control, from real obj bytes.** Every word in
    /// `docs/CFG_SHAPE.md` §3.2's table — the section that establishes there are
    /// two producers at all — plus `close_call_chain`'s measured `28030000`,
    /// decoded by this model and required to report the field the obj's own
    /// branch reads.
    ///
    /// Written as a table with a count so that deleting a row fails here rather
    /// than quietly narrowing what "graded" means.
    #[test]
    fn every_producer_word_section_3_2_published_decodes_to_the_field_the_obj_reads() {
        let cells: [([u8; 4], CrEffect, &str); 8] = [
            ([0x2b, 0x03, 0x00, 0x00], CrEffect::Compare { crf: 6 }, "?MemFree cmplwi cr6,r3,0"),
            ([0x2f, 0x03, 0x00, 0x00], CrEffect::Compare { crf: 6 }, "?b_ifn cmpwi cr6,r3,0"),
            ([0x2f, 0x1f, 0x00, 0x00], CrEffect::Compare { crf: 6 }, "?d_cont cmpwi cr6,r31,0"),
            ([0x7f, 0x1f, 0xe8, 0x00], CrEffect::Compare { crf: 6 }, "?c_forcall cmpw cr6,r31,r29"),
            ([0x35, 0x6b, 0xff, 0xff], CrEffect::Record, "?c_do addic. r11,r11,-1"),
            ([0x37, 0xff, 0xff, 0xff], CrEffect::Record, "?c_callloop addic. r31,r31,-1"),
            ([0x28, 0x03, 0x00, 0x00], CrEffect::Compare { crf: 0 }, "close_call_chain cmplwi cr0,r3,0"),
            ([0x7d, 0x2b, 0x4b, 0x79], CrEffect::Record, "ptr_walk_chain_loop mr. r11,r9"),
        ];
        assert_eq!(cells.len(), 8, "eight words graded, and the count is the assertion");
        for (word, want, what) in cells {
            assert_eq!(cr_effect(word), want, "{what}");
        }
        // …and the two fields really are different, which is the whole of §3.2.
        assert_ne!(cr_effect(cells[0].0), cr_effect(cells[6].0));
    }

    /// **A compare NAMES its field; a record form cannot.** The asymmetry that
    /// forces `Compare` to carry a `crf` and `RecordForm` to carry nothing.
    #[test]
    fn a_compare_carries_its_field_and_a_record_form_has_only_cr0() {
        assert_eq!(CondProducer::compare().crf(), CR_COMPARE);
        assert_eq!(CondProducer::compare().crf(), 6);
        assert_eq!(CondProducer::compare_into(CR0).crf(), 0);
        assert_eq!(CondProducer::RecordForm.crf(), CR0);
        assert_eq!(CondProducer::compare_into(CR_COMPARE), CondProducer::compare());
        // The two cr0 producers agree about the FIELD and are still different
        // producers — the distinction item E asks the IR to carry.
        assert_eq!(CondProducer::compare_into(CR0).crf(), CondProducer::RecordForm.crf());
        assert_ne!(CondProducer::compare_into(CR0), CondProducer::RecordForm);
        assert_eq!(CondProducer::RecordForm.what(), "a record form");
        assert_eq!(CondProducer::compare().what(), "an explicit compare");
    }

    /// **The `BI` a branch gets comes from the producer, and both answers are
    /// words a real obj carries.**
    ///
    /// `?MemFree`'s `409a0010` (compare, cr6) and `?c_do`'s `4082fff8` (record
    /// form, cr0) have the *same* `(BO, bit)` — branch-if-EQ-clear — and differ
    /// only in the two `BI` bytes. This asserts both, and asserts the wrong one
    /// is wrong: the hazard `CFG_SHAPE.md` §3.2 names, demonstrated rather than
    /// described.
    #[test]
    fn the_bi_comes_from_the_producer_and_both_words_are_real_obj_bytes() {
        let from_compare = Cond::compare(BO_FALSE, CR_BIT_EQ);
        let from_record = Cond::record_form(BO_FALSE, CR_BIT_EQ);
        assert_eq!(from_compare.bi(), 26);
        assert_eq!(from_record.bi(), 2);
        assert_eq!(from_compare.bo(), from_record.bo());
        assert_eq!(from_compare.bit(), from_record.bit());

        // ?MemFree at 0x08, target 0x18: `bne cr6,+16`.
        assert_eq!(
            encode_bc(from_compare.bo(), from_compare.bi(), 16),
            Some([0x40, 0x9a, 0x00, 0x10])
        );
        // ?c_do's back edge: `bne cr0,-8`.
        assert_eq!(
            encode_bc(from_record.bo(), from_record.bi(), -8),
            Some([0x40, 0x82, 0xff, 0xf8])
        );
        // The defect, spelled out: the compare's BI on the record form's branch
        // is a legal-looking branch to the same place on the wrong bit.
        assert_eq!(
            encode_bc(from_compare.bo(), from_compare.bi(), -8),
            Some([0x40, 0x9a, 0xff, 0xf8])
        );
        assert_ne!(
            encode_bc(from_compare.bo(), from_compare.bi(), -8),
            encode_bc(from_record.bo(), from_record.bi(), -8)
        );
    }

    /// A branch whose `BO` ignores the condition register reads **no** field,
    /// and `BI >> 2` there is an artefact rather than an answer.
    #[test]
    fn bo_20_and_bo_16_read_no_condition_register_and_bo_12_and_4_do() {
        assert_eq!(bc_reads_crf(BO_ALWAYS, 0), None); // blr
        assert_eq!(bc_reads_crf(BO_DNZ, 0), None); // bdnz — CTR, not CR
        assert_eq!(bc_reads_crf(BO_TRUE, 26), Some(6));
        assert_eq!(bc_reads_crf(BO_FALSE, 2), Some(0));
        assert_eq!(bc_reads_crf(BO_FALSE, 24), Some(6));
        // The artefact, named: BO=20 with BI=8 would "read cr2" on a naive
        // BI>>2, and `blr` reads nothing at all.
        assert_eq!(bc_reads_crf(BO_ALWAYS, 8), None);
    }

    /// **The backward scan finds the LAST writer.** §3.2's `?b_ifn` writes cr6
    /// three times in one body, each branch consuming its own; a forward scan
    /// would name the first.
    #[test]
    fn the_scan_reads_backwards_and_the_last_writer_wins() {
        let r = run(&[
            encode_cmpwi(CR_COMPARE, 3, 0),
            encode_mr(11, 3),
            encode_addic_record(11, 11, -1),
        ]);
        assert_eq!(cond_source(&r), CondSource::InBlock(CondProducer::RecordForm));

        let r2 = run(&[
            encode_addic_record(11, 11, -1),
            encode_mr(11, 3),
            encode_cmpwi(CR_COMPARE, 3, 0),
        ]);
        assert_eq!(
            cond_source(&r2),
            CondSource::InBlock(CondProducer::Compare { crf: 6 })
        );
    }

    /// **`NotInThisBlock` is a positive finding and `Unknown` is not.** A run of
    /// nine words that touch no condition register says so; a run with one word
    /// this model does not model says the opposite thing, and the two are
    /// asserted to differ.
    #[test]
    fn a_run_with_no_writer_says_so_and_is_not_the_same_as_unknown() {
        let silent = run(&[
            encode_mr(11, 4),
            encode_addi(3, 0, 7),
            encode_lwz(9, 3, 4),
            encode_stw(9, 3, 8),
            encode_add(10, 9, 3),
            encode_rlwinm(10, 9, 1, 0, 31),
            encode_mulli(8, 10, 127),
            encode_twi(6, 4, 0),
            encode_lbzu(10, 9, 1),
        ]);
        assert_eq!(cond_source(&silent), CondSource::NotInThisBlock);

        // A CALL is the load-bearing unmodelled word: the volatile CR fields do
        // not survive it, so walking past one and naming an earlier producer
        // would be naming bits that are gone.
        let mut with_call = run(&[encode_cmpwi(CR_COMPARE, 3, 0)]);
        with_call.extend_from_slice(&encode_bctrl());
        with_call.extend_from_slice(&encode_mr(11, 3));
        assert_eq!(cond_source(&with_call), CondSource::Unknown);
        assert_ne!(cond_source(&with_call), cond_source(&silent));

        // A ragged run has no answer either, and does not invent one.
        assert_eq!(cond_source(&[0x38, 0x60, 0x00]), CondSource::Unknown);
        // An empty run is NOT unknown: nothing in it writes a CR field.
        assert_eq!(cond_source(&[]), CondSource::NotInThisBlock);
    }

    /// **The DS-form trap.** `stdu` sets bit 31 and is not a record form; a
    /// decoder that tested bit 31 across the board would hand a `bclr` a cr0
    /// nothing wrote. `ld`/`std` are the same family with the bit clear.
    #[test]
    fn the_ds_form_stores_are_not_record_forms() {
        assert_eq!(cr_effect(encode_stdu(11, 30, 8)), CrEffect::Untouched);
        assert_eq!(u32::from_be_bytes(encode_stdu(11, 30, 8)) & 1, 1, "bit 31 IS set");
        assert_eq!(cr_effect(encode_std(11, 30, 8)), CrEffect::Untouched);
        assert_eq!(cr_effect(encode_ld(3, 3, 0)), CrEffect::Untouched);
        assert_eq!(cr_effect(encode_stdx(11, 26, 31)), CrEffect::Untouched);
    }

    /// **Every record-form encoder this crate has reports `Record`**, and its
    /// non-record twin reports `Untouched`. Five pairs, counted.
    #[test]
    fn the_record_form_encoders_report_record_and_their_twins_do_not() {
        let pairs: [([u8; 4], [u8; 4], &str); 4] = [
            (encode_mr_record(11, 9), encode_mr(11, 9), "mr."),
            (encode_extsb_record(11, 11), encode_extsb(11, 11), "extsb."),
            (encode_addic_record(29, 29, -1), encode_addic(29, 29, -1), "addic."),
            (
                encode_rlwinm_record(3, 4, 0, 31, 31),
                encode_rlwinm(3, 4, 0, 31, 31),
                "rlwinm.",
            ),
        ];
        assert_eq!(pairs.len(), 4);
        for (rc, plain, what) in pairs {
            assert_eq!(cr_effect(rc), CrEffect::Record, "{what}");
            assert_eq!(cr_effect(plain), CrEffect::Untouched, "{what} twin");
        }
        assert_eq!(cr_effect(encode_clrlwi_record(3, 4, 31)), CrEffect::Record);
    }

    /// **The four compare encoders report the field they were handed**, over
    /// both fields this crate actually emits into. Eight cells.
    #[test]
    fn the_compare_encoders_report_the_field_they_were_given() {
        let mut graded = 0;
        for crf in [CR_COMPARE, CR0] {
            for word in [
                encode_cmpwi(crf, 3, 0),
                encode_cmplwi(crf, 3, 0),
                encode_cmpw(crf, 3, 11),
                encode_cmplw(crf, 3, 11),
            ] {
                assert_eq!(cr_effect(word), CrEffect::Compare { crf });
                graded += 1;
            }
        }
        assert_eq!(graded, 8, "eight cells graded, not zero");
    }

    /// **Positive coverage: every ordinary word this crate emits is
    /// RECOGNISED.** Not "no failure found" — the count is asserted, and an
    /// `Unmodelled` here would mean a scan over a real block silently gives up.
    #[test]
    fn every_encoder_in_this_crates_ordinary_vocabulary_is_recognised() {
        let words: Vec<[u8; 4]> = vec![
            encode_add(3, 4, 5),
            encode_addi(3, 0, 1),
            encode_addic(11, 3, -1),
            encode_mr(11, 4),
            encode_mulli(8, 10, 127),
            encode_mullw(7, 7, 4),
            encode_rlwinm(10, 9, 1, 0, 31),
            encode_lwz(3, 4, 0),
            encode_lbz(11, 3, 0),
            encode_lbzu(10, 9, 1),
            encode_ld(3, 3, 0),
            encode_stw(3, 4, 0),
            encode_stb(3, 4, 0),
            encode_std(11, 30, 8),
            encode_stdu(11, 30, 8),
            encode_stdx(11, 26, 31),
            encode_stfd(1, 3, 8),
            encode_twi(6, 4, 0),
            encode_extsb(11, 11),
            encode_cmpwi(CR_COMPARE, 3, 0),
            encode_cmplwi(CR0, 3, 0),
            encode_mr_record(11, 9),
        ];
        assert_eq!(words.len(), 22, "22 words graded, and the count is the assertion");
        for w in words {
            assert_ne!(cr_effect(w), CrEffect::Unmodelled, "word {w:02x?}");
        }
        // …and the control: the branch family IS unmodelled, on purpose.
        assert_eq!(cr_effect(encode_blr()), CrEffect::Unmodelled);
        assert_eq!(cr_effect(encode_bc(BO_FALSE, 26, 16).unwrap()), CrEffect::Unmodelled);
        assert_eq!(cr_effect([0, 0, 0, 0]), CrEffect::Unmodelled);
    }

    /// The accessors report what was built, including the field coming from the
    /// producer and nowhere else.
    #[test]
    fn a_cond_reports_its_producer_bo_bit_and_derived_field() {
        let c = Cond::new(CondProducer::compare_into(CR0), BO_TRUE, CR_BIT_LT);
        assert_eq!(c.producer(), CondProducer::Compare { crf: 0 });
        assert_eq!(c.bo(), BO_TRUE);
        assert_eq!(c.bit(), CR_BIT_LT);
        assert_eq!(c.crf(), 0);
        assert_eq!(c.bi(), 0);
        let d = Cond::compare(BO_TRUE, CR_BIT_LT);
        assert_eq!(d.crf(), 6);
        assert_eq!(d.bi(), 24);
    }
}
