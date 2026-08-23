//! **W-BDNZ — the counted-`for` accumulate loop's lowering.** `wb-loop`'s
//! passes 1 and 2 — the rotated pre-test guard and the `mtctr`/`bdnz`
//! conversion — and **not** pass 3.
//!
//! ```text
//!     mr     r11, r3            the bound, moved off the return register
//!     li     r3, INIT           the accumulator, coalesced INTO it
//!     cmp{w,lw}i cr6, r11, 0    PASS 1: the rotated pre-test, in cr6
//!     bclr   {4,25 | 12,26}     …realised as a CONDITIONAL RETURN
//!     mtctr  r11                PASS 2: the trip count
//!     <OP>   r3, r3, r4         the body: one compound assignment
//!     bdnz   .-4                PASS 2: the latch
//!     blr
//! ```
//!
//! # This is the first lowering here derived from a READING rather than a
//! transcription — and the distinction is narrower than it sounds
//!
//! [`super::ptr_walk_loop`] and [`super::static_scan_loop`] are transcriptions
//! of one workload function each, and say so. This class comes from
//! `docs/whitebox/WB_LOOP_FINDINGS.md`'s reading of c2's own three-pass
//! counted-loop lowering, which is why it has a *general* shape (seven
//! accumulate opcodes × two signednesses × every `simm16` init) instead of two
//! immediate fields.
//!
//! **What it does NOT have is a register allocator, a scheduler or a CFG
//! builder.** `r11` and `r3` are here because real `c2` put them here, in every
//! cell of `work/w-bdnz/probe/L3.obj` and `L4.obj`, and the module that would
//! *derive* them does not exist — `super::frontier_bytes`' header spends a page
//! refusing to pretend otherwise and this file inherits that refusal whole. The
//! reading bought the **boundary**: which loops are in the class and which are
//! not, with a measured cell on each side. It did not buy an allocator.
//!
//! # The three free fields, and what each decides
//!
//! | field | decides | outside its range |
//! |---|---|---|
//! | [`CountedAccumLoop::acc_init`] | the `li r3,INIT` immediate | `lis`/`ori`, **with the guard compare interleaved between them** |
//! | [`CountedAccumLoop::op`] | one word, injectively | `+=` deletes the loop; `/=` is a six-word spine with two traps |
//! | [`CountedAccumLoop::counter_unsigned`] | `cmplwi`+`bclr 12,26` vs `cmpwi`+`bclr 4,25` | — it is a two-valued field and both values are graded |
//!
//! Everything else in the eight words is fixed, so **a body that would need a
//! different word must be refused by the READER**, never bent to fit here.
//! That is `static_scan_loop`'s standard and the fence in
//! `c2_il::…::shapes::counted_accum_loop` is what does the work.
//!
//! # Why there is no forward-branch arm
//!
//! `wb-loop` §3's realisation clause: the guard becomes a conditional return
//! *when the loop is the function's tail and the fall-through is the epilogue*,
//! and a forward `bf 25,.+N` otherwise. Both are the same rule. This class
//! requires the loop to be the tail (reader clause 10, cell `n_after`), so only
//! the `bclr` arm can be reached — and the forward arm is **absent rather than
//! unreachable-but-present**, because an arm no cell grades is an arm that will
//! be wrong when something finally reaches it (board #1148's shape).
//!
//! # The label counter
//!
//! `c2_il::IlFunction::label_slots` returns `None` for this shape, and for a
//! reason this lane MEASURED rather than inherited: the charge is
//! **mode-dependent** (+7 at `/O1`, +8 at `/Ox`, over `leaf-none`) and
//! `label_slots` has no mode parameter. `work/w-bdnz/LABEL_LEAD.md` has the
//! eight-row table and the two separating controls.

use super::encode::{
    BO_FALSE, BO_TRUE, CR_BIT_EQ, CR_BIT_GT, CR_COMPARE, cr_bi, mop_addi, mop_and,
    mop_bclr, mop_bdnz, mop_blr, mop_cmplwi, mop_cmpwi, mop_mr, mop_mtctr,
    mop_mullw, mop_or, mop_slw, mop_sraw, mop_subf, mop_xor,
};
use super::mop::{ops_to_bytes, MachineOp, Ops};
use crate::BackendError;
use c2_il::{CountedAccumLoop, CountedAccumOp, IlFunction};

/// The bound's register: `r11`, the first the allocator hands out.
///
/// `WB_REGALLOC_FINDINGS.md` §3.4's order (`r11, r10, …`) predicts it and every
/// converted cell in `wb-loop`'s 36-cell grid confirms it — but this constant is
/// **read off this class's own objs**, not adopted from that reading, which is
/// why the class carries no DISCLOSURE row.
const R_BOUND: u8 = 11;
/// The accumulator's register: `r3`. It is the return register, and the
/// accumulator coalesces into it precisely because the loop is the function's
/// tail — which is reader clause 10, and cell `n_swap` is what happens when the
/// bound occupies it instead.
const R_ACC: u8 = 3;
/// The accumulate operand's register: `r4`, formal slot 1 arriving in place.
const R_OPERAND: u8 = 4;

/// The one word the body performs. Injective by construction — see the
/// `#[test]` at the bottom, which is the pin that two opcodes cannot collapse
/// onto one encoding.
fn body_op(op: CountedAccumOp) -> MachineOp {
    match op {
        // `subf rD,rA,rB` computes `rB - rA`, so `s = s - k` is
        // `subf r3, r4, r3` — the operands are in the order that reads WRONG
        // and is right. c2's own word is `7c641850`.
        CountedAccumOp::Sub => mop_subf(R_ACC, R_OPERAND, R_ACC),
        CountedAccumOp::Mul => mop_mullw(R_ACC, R_ACC, R_OPERAND),
        CountedAccumOp::And => mop_and(R_ACC, R_ACC, R_OPERAND),
        CountedAccumOp::Or => mop_or(R_ACC, R_ACC, R_OPERAND),
        CountedAccumOp::Xor => mop_xor(R_ACC, R_ACC, R_OPERAND),
        CountedAccumOp::Shl => mop_slw(R_ACC, R_ACC, R_OPERAND),
        // **`sraw` and not `srw`** — the arithmetic shift, because the reader
        // requires a SIGNED accumulator. An unsigned one is `srw`, refuses at
        // the reader, and has no arm here on purpose.
        CountedAccumOp::Sar => mop_sraw(R_ACC, R_ACC, R_OPERAND),
    }
}

/// The rendered form of [`body_op`], kept because the injectivity pin below is
/// a statement about **words**: two opcodes must not collapse onto one
/// encoding, which is a property of the composition and not of the op value.
fn body_word(op: CountedAccumOp) -> [u8; 4] {
    body_op(op).word()
}

/// The eight words for `l`.
///
/// Returns `None` never in practice — the one `?` is the `bdnz`'s displacement,
/// which is `-4` by construction — but it is kept rather than `expect`ed so a
/// future edit to the block layout produces a refusal and not a panic.
pub(crate) fn counted_accum_loop_words(l: &CountedAccumLoop) -> Option<Vec<u8>> {
    Some(ops_to_bytes(&counted_accum_loop_ops(l)?))
}

/// **S1c (i): the same eight words as an op stream**, reachable by a caller.
pub(crate) fn counted_accum_loop_ops(l: &CountedAccumLoop) -> Option<Ops> {
    // **PASS 1 — the guard, and it is the loop's own signedness in cr6.**
    //
    // `wb-loop` §3: the pre-test compares the loop's START against its BOUND
    // with the loop's own signedness, and this class fixes START at 0. So
    // signed `0 < n` is false iff `n <= 0` — branch on **not-GT**, `bf 25` —
    // and unsigned `0 < n` is false iff `n == 0` — branch on **EQ**, `bt 26`.
    // Two of the four branch conditions §3 tabulates; the other two belong to
    // `<=` and to the record-form descending guard, both refused by the reader.
    //
    // It is emphatically **not** a "trip count > 0" test, which is the rival
    // §3 refutes on three cells and which would predict one branch condition
    // where the objs show four.
    let (guard_cmp, guard_bo, guard_bit) = if l.counter_unsigned {
        (mop_cmplwi(CR_COMPARE, R_BOUND, 0), BO_TRUE, CR_BIT_EQ)
    } else {
        (mop_cmpwi(CR_COMPARE, R_BOUND, 0), BO_FALSE, CR_BIT_GT)
    };

    let words: [MachineOp; 8] = [
        mop_mr(R_BOUND, R_ACC),
        // `li rD,SIMM` is `addi rD,0,SIMM`. The reader has already fenced
        // `acc_init` into `simm16`, so the cast cannot lose a bit.
        mop_addi(R_ACC, 0, l.acc_init as i16),
        guard_cmp,
        mop_bclr(guard_bo, cr_bi(CR_COMPARE, guard_bit)),
        // **PASS 2.** `mtctr` in the preheader and `bdnz` at the latch are the
        // two tuples `p2\ppc\lower.c`'s converter CREATES; the three it DELETES
        // (the increment, its assign, and the compare) are exactly the three
        // this emitter never writes. The loop's own counter has no register.
        mop_mtctr(R_BOUND),
        body_op(l.op),
        mop_bdnz(-4)?,
        mop_blr(),
    ];

    debug_assert_eq!(words.len(), 8);
    Some(words.to_vec())
}

/// The body for `func`, or `None` if it is not this class.
///
/// A three-valued reading of "is this the class" is deliberately not offered:
/// the only fact that decides it is `func.counted_accum_loop`, set by exactly
/// one parser production.
pub(crate) fn counted_accum_loop_text(func: &IlFunction) -> Option<Vec<u8>> {
    counted_accum_loop_words(func.counted_accum_loop()?)
}

/// The selector's arm. `mode` is the per-function optimization word.
///
/// # The mode fence is asked HERE and in the PARSER, and that is not duplication
///
/// Board **#1638**: a gate that lives only in the emitter is a fact the census
/// cannot ask, so the census counts a function in class that `PortC2` refuses.
/// The recognizer asks the optimization word first, before any body byte; this
/// clause stays because `select_function` is what `function_gate` runs.
///
/// **Both `/O1` and `/Ox` are accepted here**, which is where this class parts
/// company with its two loop siblings. It is a measurement and not a relaxation:
/// `work/w-bdnz/probe/L5ox.obj` shows `/Ox` emitting the identical eight words
/// for the signed and the unsigned cell, every cell of
/// `fixtures/cpp/wbdnz_ctr.cpp` is graded at both, and `scripts/lanes.txt`'s 18
/// mode lanes grade the crossings. `/Od` maps to no `OptMode` at all and never
/// reaches here.
pub fn counted_accum_loop_emit(
    func: &IlFunction,
    mode: super::OptMode,
) -> Result<Vec<u8>, BackendError> {
    match mode {
        super::OptMode::O1 | super::OptMode::Ox => {}
    }
    counted_accum_loop_text(func).ok_or_else(|| {
        BackendError::NotImplemented(
            "not a counted-`for` accumulate loop (the parser sets `counted_accum_loop`)"
                .to_string(),
        )
    })
}

#[cfg(test)]
mod tests {
    use crate::codegen::encode::{encode_bdnz, encode_mtctr};
    use super::*;

    fn cell(op: CountedAccumOp, acc_init: i32, counter_unsigned: bool) -> CountedAccumLoop {
        CountedAccumLoop {
            params: vec![0xE3, 0xE4],
            acc_init,
            op,
            counter_unsigned,
        }
    }

    /// **THE ORACLE PIN.** These are the bytes real `c2.dll` emitted for
    /// `?op_sub@@YAHHH@Z` in `work/w-bdnz/probe/L3.obj` at the workload's own
    /// `/O1 /Oi /EHsc /GR`, transcribed here word for word.
    #[test]
    fn the_signed_sub_cell_is_the_bytes_real_c2_emitted() {
        let got = counted_accum_loop_words(&cell(CountedAccumOp::Sub, 0, false)).unwrap();
        #[rustfmt::skip]
        let c2: [u8; 32] = [
            0x7c, 0x6b, 0x1b, 0x78, // mr    r11, r3
            0x38, 0x60, 0x00, 0x00, // li    r3, 0
            0x2f, 0x0b, 0x00, 0x00, // cmpwi cr6, r11, 0
            0x4c, 0x99, 0x00, 0x20, // bclr  4, 25
            0x7d, 0x69, 0x03, 0xa6, // mtctr r11
            0x7c, 0x64, 0x18, 0x50, // subf  r3, r4, r3
            0x42, 0x00, 0xff, 0xfc, // bdnz  .-4
            0x4e, 0x80, 0x00, 0x20, // blr
        ];
        assert_eq!(got.as_slice(), &c2[..]);
    }

    /// **THE SIGNEDNESS PIN — board #1788.** `?p_uns@@YAHIH@Z` in
    /// `work/w-bdnz/probe/L5.obj`: the SAME source but for `unsigned n` /
    /// `unsigned i`, whose IL differs from the cell above in **exactly one TYPE
    /// byte**. Two words move and no others.
    #[test]
    fn the_unsigned_cell_moves_exactly_two_words_and_they_are_the_guard() {
        let signed = counted_accum_loop_words(&cell(CountedAccumOp::Sub, 0, false)).unwrap();
        let unsigned = counted_accum_loop_words(&cell(CountedAccumOp::Sub, 0, true)).unwrap();
        #[rustfmt::skip]
        let c2_uns: [u8; 32] = [
            0x7c, 0x6b, 0x1b, 0x78, // mr     r11, r3
            0x38, 0x60, 0x00, 0x00, // li     r3, 0
            0x2b, 0x0b, 0x00, 0x00, // cmplwi cr6, r11, 0
            0x4d, 0x9a, 0x00, 0x20, // bclr   12, 26
            0x7d, 0x69, 0x03, 0xa6, // mtctr  r11
            0x7c, 0x64, 0x18, 0x50, // subf   r3, r4, r3
            0x42, 0x00, 0xff, 0xfc, // bdnz   .-4
            0x4e, 0x80, 0x00, 0x20, // blr
        ];
        assert_eq!(unsigned.as_slice(), &c2_uns[..]);
        // The difference is words 2 and 3 and nothing else: a lowering that
        // ignored the type byte would be right on 30 of 32 bytes, which is
        // exactly why the reader has to carry the flag.
        let moved: Vec<usize> = (0..8).filter(|i| signed[i * 4..i * 4 + 4] != unsigned[i * 4..i * 4 + 4]).collect();
        assert_eq!(moved, vec![2, 3]);
    }

    /// The other six accumulate words, each against real `c2`'s own
    /// (`work/w-bdnz/probe/L3.obj`, cells `op_mul` … `op_shr`).
    #[test]
    fn every_accumulate_word_is_the_one_c2_emitted() {
        use CountedAccumOp::*;
        for (op, want) in [
            (Mul, [0x7c, 0x63, 0x21, 0xd6]), // mullw 3,3,4
            (And, [0x7c, 0x63, 0x20, 0x38]), // and   3,3,4
            (Or, [0x7c, 0x63, 0x23, 0x78]),  // or    3,3,4
            (Xor, [0x7c, 0x63, 0x22, 0x78]), // xor   3,3,4
            (Shl, [0x7c, 0x63, 0x20, 0x30]), // slw   3,3,4
            (Sar, [0x7c, 0x63, 0x26, 0x30]), // sraw  3,3,4
        ] {
            let t = counted_accum_loop_words(&cell(op, 1, false)).unwrap();
            assert_eq!(&t[20..24], &want[..], "the body word for {op:?}");
        }
    }

    /// The opcode → word map is **injective**: no two accumulates may collapse
    /// onto one encoding, which would silently make one of them wrong.
    #[test]
    fn the_accumulate_words_are_pairwise_distinct() {
        use CountedAccumOp::*;
        let all = [Sub, Mul, And, Or, Xor, Shl, Sar];
        for (i, a) in all.iter().enumerate() {
            for b in &all[i + 1..] {
                assert_ne!(body_word(*a), body_word(*b), "{a:?} vs {b:?}");
            }
        }
    }

    /// The init literal is the `li` immediate and the `simm16` edges are the
    /// reader's fence, not this emitter's — so both edges must encode, and the
    /// word must be the one c2 emits (`init_big` reads `38607fff`, `init_neg`
    /// reads `38608000`; `work/w-bdnz/probe/L4.obj`).
    #[test]
    fn the_init_literal_is_the_li_immediate_at_both_simm16_edges() {
        for (k, want) in [
            (32767i32, [0x38u8, 0x60, 0x7f, 0xff]),
            (-32768, [0x38, 0x60, 0x80, 0x00]),
            (-1, [0x38, 0x60, 0xff, 0xff]),
            (0, [0x38, 0x60, 0x00, 0x00]),
            (1, [0x38, 0x60, 0x00, 0x01]),
        ] {
            let t = counted_accum_loop_words(&cell(CountedAccumOp::Mul, k, false)).unwrap();
            assert_eq!(&t[4..8], &want[..], "li r3,{k}");
        }
    }

    /// **`mtctr`'s split SPR field, pinned.** `9 << 11` would assemble to a
    /// legal `mtspr` naming a different register and nothing downstream would
    /// notice; the captured word says otherwise.
    #[test]
    fn mtctr_uses_the_split_spr_field() {
        assert_eq!(encode_mtctr(11), [0x7d, 0x69, 0x03, 0xa6]);
        assert_eq!(encode_mtctr(4), [0x7c, 0x89, 0x03, 0xa6]);
        assert_eq!(encode_mtctr(3), [0x7c, 0x69, 0x03, 0xa6]);
        // The naive encoding, spelled out so the test says what it is refusing.
        let naive: u32 = (31 << 26) | (11 << 21) | (9 << 11) | (467 << 1);
        assert_ne!(encode_mtctr(11), naive.to_be_bytes());
    }

    /// **The selector routes this shape at BOTH modes, and to the same bytes.**
    ///
    /// The mode arm is where this class parts company with its two loop
    /// siblings, so it is asserted rather than commented: `/Ox` emits the
    /// identical eight words (`work/w-bdnz/probe/L5ox.obj`), and a selector that
    /// refused one of the two would turn `fixtures/cpp/wbdnz_ctr.cpp` from a
    /// `match` at `/Ox` into a refusal without any byte changing.
    #[test]
    fn the_selector_routes_the_shape_at_both_modes_and_to_the_same_bytes() {
        use crate::codegen::select::{select_function, OptMode};
        let mut f = super::super::testutil::func_with(vec![0xE3, 0xE4], Vec::new());
        f.body = c2_il::BodyShape::CountedAccumLoop(cell(CountedAccumOp::Mul, 1, false));
        let want = counted_accum_loop_words(f.counted_accum_loop().unwrap()).unwrap();
        for mode in [OptMode::O1, OptMode::Ox] {
            // S1b: the retired `Selected::Plain(t)` pattern, read back through
            // the `#[cfg(test)]` view that stands in for it.
            let sel = select_function(&f, mode).expect("the selector must route this shape");
            let t = sel
                .as_plain()
                .expect("the counted loop is a Plain body, not a tail or a float");
            assert_eq!(t, want.as_slice(), "at {mode:?}");
        }
    }

    /// `bdnz` is `bc` at `BO = 16`, `BI = 0`, and the two captured
    /// displacements are the one-word and two-word body forms.
    #[test]
    fn bdnz_is_the_captured_word_at_both_measured_displacements() {
        assert_eq!(encode_bdnz(-4), Some([0x42, 0x00, 0xff, 0xfc]));
        assert_eq!(encode_bdnz(-8), Some([0x42, 0x00, 0xff, 0xf8]));
        // Out of the 14-bit reach it refuses rather than truncating.
        assert_eq!(encode_bdnz(-40000), None);
        assert_eq!(encode_bdnz(-3), None);
    }
}
