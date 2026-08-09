//! **W-BIQUAD — the emitter for the null-guarded float-store diamond.**
//!
//! The reader's accept/refuse boundary and the source shape are on
//! [`c2_il::func::body::shapes::fp_store_diamond`]; this file is the words and
//! nothing else. Everything variable in them is named in
//! [`c2_il::FpStoreDiamond`]: two member-offset lists, a division run and two
//! pooled constants.
//!
//! ```text
//!   ?SetCoefficients@Biquad@DSP@@QAAXPAM@Z     .text COMDAT, 0x8C B, nrel 8
//!
//!    off  word       instruction              why it is this word
//!   ----  --------   ----------------------   -----------------------------------
//!   0x00  3d600000   lis    r11,A             B-RULE: A is used in the then-arm
//!                                             AND after the join, so its `lis`
//!                                             goes at the top of the ENTRY block,
//!                                             the earliest block dominating both.
//!                                             REFHI + PAIR.
//!   0x04  2b040000   cmplwi cr6,r4,0          the guard, UNSIGNED (pointer)
//!   0x08  c00b0000   lfs    f0,0(r11)         A's load, also in the entry: f0 is
//!                                             live across the whole diamond.
//!                                             REFLO + PAIR.
//!   0x0c  409a0024   bf     26,+0x24          `bne cr6` — the IL's `38` is
//!                                             branch-on-FALSE over an `==`, so
//!                                             the ELSE arm is the taken side and
//!                                             the then-arm falls through.
//!   0x10  3d600000   lis    r11,B             B-RULE again: B is used ONCE, in
//!                                             the then-arm, so its `lis` is that
//!                                             block's first word — NOT the
//!                                             function's. REFHI + PAIR.
//!                                             r11 again, not r10: a second block
//!                                             is a second live range.
//!   0x14  d0030010   stfs   f0,16(r3)   ┐  the then-arm's A-stores, in
//!   0x18  d003000c   stfs   f0,12(r3)   │  SOURCE order
//!   0x1c  d0030008   stfs   f0,8(r3)    │
//!   0x20  d0030004   stfs   f0,4(r3)    ┘
//!   0x24  c1ab0000   lfs    f13,0(r11)       B's load, AT THE USE — five words
//!                                             below its own `lis`. REFLO + PAIR.
//!   0x28  d1a30000   stfs   f13,0(r3)        the then-arm's B-store, always last
//!   0x2c  48000054   b      +0x54            to the JOIN, not to the epilogue
//!   0x30  c184000c   lfs    f12,12(r4)  ┐  B′-RULE: the CSE'd divisor is
//!   0x34  c1a40000   lfs    f13,0(r4)   │  loaded FIRST in every statement
//!   0x38  edad6024   fdivs  f13,f13,f12 │  of the run except the last
//!   0x3c  d1a30000   stfs   f13,0(r3)   ┘
//!   …                                        × (n − 1)
//!   0x70  c1a40014   lfs    f13,20(r4)  ┐  … and in the LAST statement the
//!   0x74  c184000c   lfs    f12,12(r4)  │  operands go in SOURCE order:
//!   0x78  edad6024   fdivs  f13,f13,f12 │  numerator, then denominator
//!   0x7c  d1a30010   stfs   f13,16(r3)  ┘
//!   0x80  d0030018   stfs   f0,24(r3)   ┐  the join, still holding A in f0 —
//!   0x84  d0030014   stfs   f0,20(r3)   ┘  which is why the else-arm may not
//!                                             touch f0
//!   0x88  4e800020   blr
//! ```
//!
//! ## The three things this file gets right that a naive emitter does not
//!
//! **1. The two `lis` are in different blocks and the rule is a DOMINATOR, not a
//! position.** `WB_CHOOSER_FINDINGS` §3.3's B-RULE has 3 entry-block witnesses
//! and 6 block-local ones, and its cell **B1** is the one that kills the reading
//! this obj invites: a constant used only in the then-arm gets its `lis` as the
//! **first word of the then-block**, not of the function. Both readings agree on
//! `?SetCoefficients`' first word and disagree on its fifth.
//!
//! **2. `lo_off` is not `hi_off + 4`.** `B`'s `lis` is at 0x10 and its `lfs` at
//! 0x24. Every graded site before this class had them adjacent, and
//! [`crate::codegen::FpConstRef`] had encoded that as arithmetic; the REFLO
//! would have landed on `stfs f0,8(r3)`.
//!
//! **3. B′-RULE's flip is on the LAST division and nowhere else**
//! (`WB_CHOOSER_FINDINGS` §4.1: runs of 2, 3, 4, 5 and 6 divisions, one flip
//! each, always last — 5 flip witnesses against 15 non-flip). Getting it
//! backwards is exactly two wrong words at the end of one arm, in an obj that
//! still links.
//!
//! **B-RULE-2 is deliberately NOT consulted.** The compare/branch separation
//! slot is `medium` at exactly three witnesses; this file transcribes the entry
//! block's word order off this class's own obj and asks no separation question,
//! so #260's warning about a clause with that history cannot bite here.

use c2_il::FpStoreDiamond;
#[cfg(test)]
use c2_il::FpDiamondConstStore;

use crate::BackendError;
use crate::codegen::encode::{
    encode_addis, encode_b_intra, encode_bc, encode_blr, encode_cmplwi, encode_fdiv, encode_lfs,
    encode_stfs,
};
use crate::codegen::leaf::float::FpConstRef;
use crate::codegen::select::out_of_class;

/// `bf 26` — branch if condition-register bit 26 (`cr6`'s EQ) is FALSE, i.e.
/// `bne cr6`. `BO = 4` (branch if the bit is clear, no counter), `BI = 26`.
const BO_FALSE: u8 = 4;
const BI_CR6_EQ: u8 = 26;

/// The FPR the entry-hoisted pool `A` lives in for the whole body. It is read by
/// the then-arm AND the join, which straddle the branch, so it may not be reused
/// by the else-arm.
const FPR_A: u8 = 0;
/// The FPR the block-local pool `B` is loaded into, and the division run's
/// numerator/result.
const FPR_B: u8 = 13;
/// The division run's divisor.
const FPR_DEN: u8 = 12;
/// The addressing scratch both `lis` take. `WB_CHOOSER_FINDINGS` §3.3: r11
/// first, then r10 — and r10 is never reached here, because the two pools are in
/// two different blocks and each is one live range of its own.
const ADDR_GPR: u8 = 11;

/// The word count of the emitted body, so the branch displacements can be
/// resolved before the words are laid down. Kept as one arithmetic expression
/// with the emission below rather than counted twice: a displacement computed
/// from a different formula than the one the layout obeys is the defect
/// `docs/CFG_SHAPE.md` §3.3 records.
fn block_lengths(d: &FpStoreDiamond) -> (u32, u32, u32) {
    // entry: lis · cmplwi · lfs · bc
    let entry = 4 * 4;
    // then: lis · (n−1) stfs · lfs · stfs · b
    let then = 4 * (1 + (d.then_stores.len() as u32 - 1) + 1 + 1 + 1);
    // else: 4 words per division
    let els = 4 * 4 * d.divs.len() as u32;
    (entry, then, els)
}

/// Emit the `.text` for a **W-BIQUAD float-store diamond**, plus one
/// [`FpConstRef`] per constant *reference site*.
///
/// The class is a **leaf** — no prologue, no `.pdata`, no label triple — so
/// unlike every other branch-emitting class in this crate there is no `base_off`
/// parameter: both of its branches are `bc`/`b` with true self-relative
/// displacements and neither takes a relocation (`docs/CFG_SHAPE.md` §3.3, board
/// #191). The only relocations are the two REFHI/REFLO quads.
pub fn fp_store_diamond_text(
    d: &FpStoreDiamond,
) -> Result<(Vec<u8>, Vec<FpConstRef>), BackendError> {
    // The reader established all of this; the emitter restates it as a backstop
    // rather than trusting the parser, which is the discipline
    // `codegen::store_run_call` states and board #844 is the reason for.
    if d.params.len() != 2 {
        return Err(out_of_class(
            "an FP store diamond whose formals are not exactly `this` and one pointer",
        ));
    }
    if d.then_stores.len() < 2 || d.divs.len() < 2 || d.join_stores.is_empty() {
        return Err(out_of_class(
            "an FP store diamond outside the measured arm sizes (>=2 then-stores, \
             >=2 divisions, >=1 join store)",
        ));
    }
    let a_bits = d.join_stores[0].bits;
    let b_bits = d.then_stores[d.then_stores.len() - 1].bits;
    if a_bits == b_bits {
        return Err(out_of_class(
            "an FP store diamond with ONE pooled constant: the two-block `lis` \
             placement this class transcribes has no witness at one pool",
        ));
    }
    if d.then_stores[..d.then_stores.len() - 1].iter().any(|s| s.bits != a_bits)
        || d.join_stores.iter().any(|s| s.bits != a_bits)
    {
        return Err(out_of_class(
            "an FP store diamond with more than two distinct pooled constants",
        ));
    }
    let den = d.divs[0].den;
    if d.divs.iter().any(|x| x.den != den) {
        return Err(out_of_class(
            "an FP store diamond whose division run does not share one divisor: \
             there is no CSE reload for B'-RULE to order",
        ));
    }
    // `this` is r3 and the guarded pointer r4 — positional, and the reader has
    // already required both to occupy one register each.
    let this_reg: u8 = 3;
    let src_reg: u8 = 4;

    let disp16 = |v: i32, what: &str| -> Result<i16, BackendError> {
        i16::try_from(v).map_err(|_| {
            out_of_class(&format!("{what} does not fit a signed 16-bit displacement"))
        })
    };

    let (entry_len, then_len, else_len) = block_lengths(d);
    let mut text: Vec<u8> = Vec::new();
    let mut consts: Vec<FpConstRef> = Vec::new();

    // ---- the ENTRY block ---------------------------------------------------
    let a_hi = text.len() as u32;
    text.extend_from_slice(&encode_addis(ADDR_GPR, 0, 0));
    text.extend_from_slice(&encode_cmplwi(6, src_reg, 0));
    let a_lo = text.len() as u32;
    text.extend_from_slice(&encode_lfs(false, FPR_A, ADDR_GPR, 0));
    consts.push(FpConstRef { bits: a_bits, double: false, hi_off: a_hi, lo_off: a_lo });
    // The conditional branch's displacement is self-relative and takes no
    // relocation: it names the else arm, which starts one whole then-block below
    // the branch word itself.
    let to_else = (entry_len - text.len() as u32) + then_len;
    text.extend_from_slice(
        &encode_bc(BO_FALSE, BI_CR6_EQ, to_else as i32)
            .ok_or_else(|| out_of_class("the guard branch does not fit a `bc` displacement"))?,
    );

    // ---- the THEN arm ------------------------------------------------------
    let b_hi = text.len() as u32;
    text.extend_from_slice(&encode_addis(ADDR_GPR, 0, 0));
    let (b_store, a_stores) = d
        .then_stores
        .split_last()
        .expect("checked non-empty above");
    for s in a_stores {
        text.extend_from_slice(&encode_stfs(
            false,
            FPR_A,
            this_reg,
            disp16(s.off, "a then-arm member offset")?,
        ));
    }
    let b_lo = text.len() as u32;
    text.extend_from_slice(&encode_lfs(false, FPR_B, ADDR_GPR, 0));
    consts.push(FpConstRef { bits: b_bits, double: false, hi_off: b_hi, lo_off: b_lo });
    text.extend_from_slice(&encode_stfs(
        false,
        FPR_B,
        this_reg,
        disp16(b_store.off, "the then-arm's block-local member offset")?,
    ));
    // The join is one whole else-block below this word.
    let to_join = (entry_len + then_len - text.len() as u32) + else_len;
    text.extend_from_slice(
        &encode_b_intra(to_join as i32)
            .ok_or_else(|| out_of_class("the join branch does not fit a `b` displacement"))?,
    );

    // ---- the ELSE arm: the CSE'd division run ------------------------------
    let last = d.divs.len() - 1;
    for (i, x) in d.divs.iter().enumerate() {
        let num_w = encode_lfs(false, FPR_B, src_reg, disp16(x.num, "a numerator offset")?);
        let den_w = encode_lfs(false, FPR_DEN, src_reg, disp16(x.den, "the divisor offset")?);
        // B′-RULE. Every statement but the last loads the reload FIRST; the last
        // one — the reload's final use — loads in source order.
        if i == last {
            text.extend_from_slice(&num_w);
            text.extend_from_slice(&den_w);
        } else {
            text.extend_from_slice(&den_w);
            text.extend_from_slice(&num_w);
        }
        text.extend_from_slice(&encode_fdiv(false, FPR_B, FPR_B, FPR_DEN));
        text.extend_from_slice(&encode_stfs(
            false,
            FPR_B,
            this_reg,
            disp16(x.off, "a division destination offset")?,
        ));
    }

    // ---- the JOIN ----------------------------------------------------------
    debug_assert_eq!(
        text.len() as u32,
        entry_len + then_len + else_len,
        "the block lengths and the emission disagree — every displacement above \
         is computed from the former and obeyed by the latter"
    );
    for s in &d.join_stores {
        text.extend_from_slice(&encode_stfs(
            false,
            FPR_A,
            this_reg,
            disp16(s.off, "a join member offset")?,
        ));
    }
    text.extend_from_slice(&encode_blr());

    Ok((text, consts))
}

/// The **GPR footprint** of a lowered body of this class: `{r11}`.
///
/// Named and exported because it is what
/// [`crate::codegen::ctor_forward_call`] needs and cannot derive — M-RULE
/// (`WB_CHOOSER_FINDINGS` §2.3) places a value live across a call in a register
/// the callee does not write, and for a **same-TU** callee c2 uses that callee's
/// EXACT register footprint rather than the whole volatile set. A constructor
/// forwarding to a body of this class therefore parks `this` in a volatile
/// (`mr r10,r3`) where the same constructor forwarding to an external parks it
/// in `r31` and pays a `std`/`ld` pair — measured, both cells, in
/// `work/w-biquad/probe/park_{local,extern}.cpp`.
///
/// This is a **statement about this file's emission**, one function away from
/// the words that make it true, which is the only place it can be maintained:
/// the body writes `r11` (both `lis`) and reads `r3`/`r4`, and it writes no
/// other GPR at all. `f0`, `f12` and `f13` are FPRs and no GPR park competes
/// with them.
pub const GPR_FOOTPRINT: &[u8] = &[ADDR_GPR];

/// The member offsets a body of this class stores to, in emission order — a
/// decode-only convenience for tests that want to assert the transcription
/// without re-deriving the split.
#[cfg(test)]
pub(crate) fn store_offsets(d: &FpStoreDiamond) -> Vec<i32> {
    d.then_stores
        .iter()
        .chain(d.join_stores.iter())
        .map(|s: &FpDiamondConstStore| s.off)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use c2_il::FpDiamondDiv;

    /// `Biquad.cpp`'s own shape, as `IlFunction` carries it.
    fn biquad() -> FpStoreDiamond {
        let f = |v: f32| (v as f64).to_bits();
        FpStoreDiamond {
            params: vec![1, 2],
            then_stores: vec![
                FpDiamondConstStore { off: 16, bits: f(0.0) },
                FpDiamondConstStore { off: 12, bits: f(0.0) },
                FpDiamondConstStore { off: 8, bits: f(0.0) },
                FpDiamondConstStore { off: 4, bits: f(0.0) },
                FpDiamondConstStore { off: 0, bits: f(1.0) },
            ],
            divs: vec![
                FpDiamondDiv { off: 0, num: 0, den: 12 },
                FpDiamondDiv { off: 4, num: 4, den: 12 },
                FpDiamondDiv { off: 8, num: 8, den: 12 },
                FpDiamondDiv { off: 12, num: 16, den: 12 },
                FpDiamondDiv { off: 16, num: 20, den: 12 },
            ],
            join_stores: vec![
                FpDiamondConstStore { off: 24, bits: f(0.0) },
                FpDiamondConstStore { off: 20, bits: f(0.0) },
            ],
        }
    }

    fn words(t: &[u8]) -> Vec<u32> {
        t.chunks(4).map(|w| u32::from_be_bytes(w.try_into().unwrap())).collect()
    }

    /// **The 35 words, against `work/w-biquad/real.obj` word for word.** Not a
    /// spot check: the whole `.text` COMDAT of
    /// `?SetCoefficients@Biquad@DSP@@QAAXPAM@Z`, taken off the reference obj
    /// this lane compiled with real `c2.dll` under wibo at the workload's own
    /// flags. The length is asserted separately, because emitting a body of the
    /// wrong LENGTH is the one error this class can make that still links.
    #[test]
    fn the_thirty_five_words_are_the_reference_obj() {
        let (text, _) = fp_store_diamond_text(&biquad()).expect("in class");
        assert_eq!(text.len(), 140, "the reference `.text` COMDAT is 140 bytes");
        assert_eq!(
            words(&text),
            vec![
                0x3d600000, 0x2b040000, 0xc00b0000, 0x409a0024, // entry
                0x3d600000, 0xd0030010, 0xd003000c, 0xd0030008, // then
                0xd0030004, 0xc1ab0000, 0xd1a30000, 0x48000054,
                0xc184000c, 0xc1a40000, 0xedad6024, 0xd1a30000, // else, x5
                0xc184000c, 0xc1a40004, 0xedad6024, 0xd1a30004,
                0xc184000c, 0xc1a40008, 0xedad6024, 0xd1a30008,
                0xc184000c, 0xc1a40010, 0xedad6024, 0xd1a3000c,
                0xc1a40014, 0xc184000c, 0xedad6024, 0xd1a30010, // ...the FLIP
                0xd0030018, 0xd0030014, 0x4e800020, // join
            ]
        );
    }

    /// **B′-RULE, isolated: exactly one flip and it is the last division.**
    ///
    /// Read off the emitted words rather than off the source, and asserted over
    /// runs of 2, 3, 4 and 5 — four of the five run lengths
    /// `WB_CHOOSER_FINDINGS` §4.1 compiled. A rule fitted to `Biquad.cpp`'s run
    /// of five alone would pass a test written only at five.
    #[test]
    fn the_divisor_loads_first_in_every_statement_but_the_last() {
        for n in 2..=5usize {
            let mut d = biquad();
            d.divs.truncate(n);
            let (text, _) = fp_store_diamond_text(&d).expect("in class");
            let w = words(&text);
            let (entry, then, _) = block_lengths(&d);
            let base = ((entry + then) / 4) as usize;
            let mut flips = Vec::new();
            for i in 0..n {
                // `lfs f12,…` is the divisor and `lfs f13,…` the numerator; the
                // FRT field is bits 6..11 of the word.
                let frt = (w[base + 4 * i] >> 21) & 0x1f;
                if frt == FPR_B as u32 {
                    flips.push(i);
                }
            }
            assert_eq!(flips, vec![n - 1], "run of {n}: one flip, on the last");
        }
    }

    /// **`lo_off` is NOT `hi_off + 4` for the block-local pool**, which is the
    /// whole reason [`FpConstRef`] grew the field. Five words separate B's
    /// `lis` from its `lfs`, with four unrelated `stfs` between them, and the
    /// arithmetic form would have put the REFLO on the third of those.
    #[test]
    fn the_block_local_pool_has_its_halves_five_words_apart() {
        let (_, consts) = fp_store_diamond_text(&biquad()).expect("in class");
        assert_eq!(consts.len(), 2);
        // A — the entry-hoisted pool; the `cmplwi` sits between its two halves.
        assert_eq!((consts[0].hi_off, consts[0].lo_off), (0, 8));
        // B — the block-local one.
        assert_eq!((consts[1].hi_off, consts[1].lo_off), (0x10, 0x24));
        assert_ne!(consts[1].lo_off, consts[1].hi_off + 4);
        // EMISSION order, which the writer REVERSES to get the section order.
        assert_eq!(consts[0].bits, (0.0f32 as f64).to_bits());
        assert_eq!(consts[1].bits, (1.0f32 as f64).to_bits());
    }

    /// **The branch displacements are a function of the arm sizes**, and both
    /// branches are self-relative with no relocation. Asserted over a grid
    /// rather than at `Biquad.cpp`'s one shape, because a displacement computed
    /// from a different formula than the one the layout obeys is
    /// `docs/CFG_SHAPE.md` §3.3's defect and agrees at exactly one point.
    #[test]
    fn both_branches_land_on_their_blocks_at_every_arm_size() {
        for nt in 2..=5usize {
            for nd in 2..=5usize {
                let mut d = biquad();
                d.then_stores.truncate(nt - 1);
                d.then_stores
                    .push(FpDiamondConstStore { off: 0, bits: (1.0f32 as f64).to_bits() });
                d.divs.truncate(nd);
                let (text, _) = fp_store_diamond_text(&d).expect("in class");
                let w = words(&text);
                let (entry, then, els) = block_lengths(&d);
                let bc = w[3];
                assert_eq!(bc >> 16, 0x409a, "`bne cr6` at every size");
                assert_eq!(bc & 0xffff, entry - 0x0c + then, "then={nt} div={nd}");
                let b_at = ((entry + then) / 4 - 1) as usize;
                assert_eq!(w[b_at] >> 26, 18, "primary opcode 18, AA=0 LK=0");
                assert_eq!(
                    w[b_at] & 0x03ff_fffc,
                    entry + then - 4 * b_at as u32 + els,
                    "then={nt} div={nd}"
                );
            }
        }
    }

    /// Every backstop refuses, and each names its own construct. The reader has
    /// established all of these already; the emitter restates them because a
    /// wrong emit here is silent, and a test exercising only the accepted shape
    /// would leave seven arms ungraded (board #1148).
    #[test]
    fn the_emitter_backstops_refuse_by_name() {
        let one = (1.0f32 as f64).to_bits();
        let cases: Vec<(&str, FpStoreDiamond)> = vec![
            ("formals", FpStoreDiamond { params: vec![1], ..biquad() }),
            ("one division", FpStoreDiamond { divs: biquad().divs[..1].to_vec(), ..biquad() }),
            (
                "one then-store",
                FpStoreDiamond { then_stores: biquad().then_stores[..1].to_vec(), ..biquad() },
            ),
            ("empty join", FpStoreDiamond { join_stores: Vec::new(), ..biquad() }),
            (
                "one pool",
                FpStoreDiamond {
                    join_stores: vec![FpDiamondConstStore { off: 24, bits: one }],
                    ..biquad()
                },
            ),
            (
                "divisors differ",
                FpStoreDiamond {
                    divs: vec![
                        FpDiamondDiv { off: 0, num: 0, den: 12 },
                        FpDiamondDiv { off: 4, num: 4, den: 16 },
                    ],
                    ..biquad()
                },
            ),
        ];
        for (what, d) in cases {
            assert!(fp_store_diamond_text(&d).is_err(), "{what} must refuse");
        }
        // …and the displacement bound, the one refusal about a range rather than
        // about a shape.
        let mut wide = biquad();
        wide.join_stores[0].off = 0x8000;
        assert!(fp_store_diamond_text(&wide).is_err(), "a 16-bit displacement bound");
    }

    /// The decode-only offset list, kept honest: then-stores then join-stores,
    /// in emission order.
    #[test]
    fn store_offsets_are_then_then_join() {
        assert_eq!(store_offsets(&biquad()), vec![16, 12, 8, 4, 0, 24, 20]);
    }
}
