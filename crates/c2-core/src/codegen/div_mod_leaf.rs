//! **The integer divide/modulo leaf** — `return a / b;` and `return a % b;`
//! over two formals, signed and unsigned, at both optimization modes.
//!
//! Eight constant bodies (4 shapes × 2 modes), three to nine words, **no free
//! fields**. Every word below was read off real `c2.dll`'s output with
//! `work/w-divmod/twigrid.py --dis`, which decodes each instruction itself and
//! cross-checks the decode against `llvm-mc` — so the register fields and the
//! two trap `TO` fields are transcribed, not inferred from a mnemonic.
//!
//! ```text
//!   signed %                          signed /
//!     rotlwi r11,r3,1                   rotlwi r11,r3,1
//!     divw   r10,r3,r4                  divw   r3,r3,r4
//!     addi   r11,r11,-1                 addi   r11,r11,-1
//!     mullw  r10,r10,r4                 twi    6,r4,0
//!     andc   r11,r4,r11                 andc   r11,r4,r11
//!     twi    6,r4,0                     twi    5,r11,-1
//!     subf   r3,r10,r3                  blr
//!     twi    5,r11,-1
//!     blr
//!
//!   unsigned %                        unsigned /
//!     divwu  r11,r3,r4                  divwu  r3,r3,r4
//!     twi    6,r4,0                     twi    6,r4,0
//!     mullw  r11,r11,r4                 blr
//!     subf   r3,r11,r3
//!     blr
//! ```
//!
//! # This is a transcription, and the mode axis is why that has to be said
//!
//! There is no scheduler here and no register allocator. What makes that
//! visible rather than a claim is `/Ox`: it emits the **same mnemonics in the
//! same order** and a *different register assignment* — `mullw r8,r10,r4` where
//! `/O1` has `mullw r10,r10,r4`, `andc r7,r4,r9` where `/O1` has
//! `andc r11,r4,r11`. Three of the four shapes move; **`unsigned /` is
//! byte-identical across the modes**, which is the control that says the
//! difference is real allocation and not a capture artefact.
//!
//! That is `docs/OPT_MODE.md`'s register-field reading holding *for this
//! class* — and it is worth being precise that this does not un-refute it.
//! `codegen::ptr_walk_loop` records the strongest counterexample: at `/Ox` the
//! same loop is a different *body*, twenty-one words against twenty. The modes
//! differ in a register field for a single-block leaf and in the body once
//! there is more than one block. Both readings are true of different
//! populations, which is why this file carries **both** mode tables instead of
//! refusing outside `/O1` as the loop does.
//!
//! # The `twi 6` placement, and why this file does not implement the rule
//!
//! `w-hash` §9.1 left the placement as the open residual blocking exactly this
//! lowering: it had seen two placements and no discriminator.
//! `work/w-divmod/` graded 161 cells and found one:
//!
//! > `twi 6` is emitted immediately after the **first instruction of the
//! > division's own basic block that is neither a multiply (`mulli`/`mullw`)
//! > nor a register-amount shift (`slw`/`srw`/`sraw`)** — provided the dividend
//! > is produced in that block, the divisor is live-in to it, and the block is
//! > not a loop body. Otherwise it stays in the spine.
//!
//! **Nothing in this file reads that rule.** Every body here has both operands
//! live-in, so the hoisting clause is false by construction and the placement
//! is one of the constants tabulated above. A shape that could reach the other
//! regime — any computed operand — is refused by the recognizer, not scheduled.
//! That is deliberate: the rule is supported by 161 cells and is still a
//! description of `c2`'s scheduler rather than a model of it, and this project
//! has refuted ten placement rules that were fitted to less.
//!
//! # The must-fail mutation, RUN
//!
//! A guard nobody has seen fail is not known to work. Swapping the two lines
//! marked `MUTATION ANCHOR` below — moving `twi 6` one slot later, after the
//! `subf`, which is *exactly* the placement the `short`/`char` widths do show
//! (`… andc subf twi twi extsb`) and therefore the most plausible wrong answer
//! available — turns `fixtures/cpp/wdivmod_leaf.cpp` from `Port=Match` into a
//! live **`Port=Mismatch @ offset 556`** against real `c2.dll` under wibo, and
//! fails `the_four_o1_bodies_are_reproduced_word_for_word` and its `/Ox` twin.
//! Run, not described; reverted, and the anchor is left in place so the next
//! reader can re-run it.

use c2_il::DivModLeaf;

use crate::codegen::encode::{
    encode_addi, encode_andc, encode_blr, encode_divw, encode_divwu, encode_mullw, encode_rlwinm,
    encode_subf, encode_twi,
};
use crate::codegen::select::{out_of_class, OptMode};
use crate::BackendError;

/// The dividend formal, slot 0.
const R_A: u8 = 3;
/// The divisor formal, slot 1.
const R_B: u8 = 4;

/// `TO = 6` — *equal* ∪ *unsigned less-than*, over the divisor and `0`. Traps
/// exactly when the divisor is zero; see [`encode_twi`]'s own docs, where the
/// bit meaning is derived rather than asserted.
const TO_DIV_BY_ZERO: u8 = 6;
/// `TO = 5` — *equal* ∪ *unsigned greater-than*, over `andc(divisor,
/// rotlwi(dividend,1) - 1)` and `-1`. Traps exactly on `INT_MIN / -1`.
const TO_OVERFLOW: u8 = 5;

/// Emit the whole body. No relocation, no pooled constant, no label and no
/// branch — so the caller takes it as an ordinary `Selected::Plain`.
pub(crate) fn div_mod_leaf_text(d: &DivModLeaf, mode: OptMode) -> Result<Vec<u8>, BackendError> {
    // Re-asserted here even though `try_parse_div_mod_leaf` already required
    // it: `select_function` is what `function_gate` runs, so a shape that
    // reached codegen with a different arity would be a census/gate
    // disagreement, and that counter reading 0 is the only thing keeping the
    // census honest about what the port accepts.
    if d.params.len() != 2 {
        return Err(out_of_class(
            "div/mod leaf with other than two formals: the register plan is measured at two",
        ));
    }
    let mut t: Vec<u8> = Vec::with_capacity(36);
    match (d.signed, d.is_mod) {
        // ---- unsigned `/` — three words, and byte-identical at both modes ---
        (false, false) => {
            t.extend_from_slice(&encode_divwu(R_A, R_A, R_B));
            t.extend_from_slice(&encode_twi(TO_DIV_BY_ZERO, R_B, 0));
        }
        // ---- unsigned `%` ---------------------------------------------------
        //
        // `/O1` computes the quotient in r11 and multiplies it back in place;
        // `/Ox` moves the product to r10. Same five words, one register field.
        (false, true) => {
            let q = 11;
            let p = match mode {
                OptMode::O1 => 11,
                OptMode::Ox => 10,
            };
            t.extend_from_slice(&encode_divwu(q, R_A, R_B));
            t.extend_from_slice(&encode_twi(TO_DIV_BY_ZERO, R_B, 0));
            t.extend_from_slice(&encode_mullw(p, q, R_B));
            t.extend_from_slice(&encode_subf(R_A, p, R_A));
        }
        // ---- signed `/` -----------------------------------------------------
        //
        // The quotient goes straight to r3; the predicate's three instructions
        // are the only other work, and `twi 6` sits between the `addi` and the
        // `andc` — one slot earlier than in the `%` spine, which is the
        // schedule difference `w-hash` R5 registered and lost on.
        (true, false) => {
            let ovf = match mode {
                OptMode::O1 => 11,
                OptMode::Ox => 10,
            };
            t.extend_from_slice(&encode_rlwinm(11, R_A, 1, 0, 31));
            t.extend_from_slice(&encode_divw(R_A, R_A, R_B));
            t.extend_from_slice(&encode_addi(11, 11, -1));
            t.extend_from_slice(&encode_twi(TO_DIV_BY_ZERO, R_B, 0));
            t.extend_from_slice(&encode_andc(ovf, R_B, 11));
            t.extend_from_slice(&encode_twi(TO_OVERFLOW, ovf, -1));
        }
        // ---- signed `%` — `Sort.cpp`'s own spine, standing on its own -------
        //
        // These are the eight words `codegen::ptr_walk_loop` welds into its
        // twenty, in the same order, with the leaf's own register plan.
        (true, true) => {
            // (predicate temp, quotient, product) per mode.
            let (pred, quot, prod) = match mode {
                OptMode::O1 => (11u8, 10u8, 10u8),
                OptMode::Ox => (9u8, 10u8, 8u8),
            };
            let ovf = match mode {
                OptMode::O1 => 11,
                OptMode::Ox => 7,
            };
            t.extend_from_slice(&encode_rlwinm(11, R_A, 1, 0, 31));
            t.extend_from_slice(&encode_divw(quot, R_A, R_B));
            t.extend_from_slice(&encode_addi(pred, 11, -1));
            t.extend_from_slice(&encode_mullw(prod, quot, R_B));
            t.extend_from_slice(&encode_andc(ovf, R_B, pred));
            // MUTATION ANCHOR: swap these two lines and the fixture goes from
            // `Port=Match` to a live `Port=Mismatch @ offset 556`. See the
            // module docs.
            t.extend_from_slice(&encode_twi(TO_DIV_BY_ZERO, R_B, 0));
            t.extend_from_slice(&encode_subf(R_A, prod, R_A));
            t.extend_from_slice(&encode_twi(TO_OVERFLOW, ovf, -1));
        }
    }
    t.extend_from_slice(&encode_blr());
    debug_assert_eq!(
        t.len(),
        4 * expected_words(d),
        "the class's body length is a constant of (signed, is_mod)"
    );
    Ok(t)
}

/// Word count per shape, stated separately from the emitter so the two have to
/// agree. `/` and `%` are **different lengths**, not one body with a flag.
fn expected_words(d: &DivModLeaf) -> usize {
    match (d.signed, d.is_mod) {
        (false, false) => 3,
        (false, true) => 5,
        (true, false) => 7,
        (true, true) => 9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn leaf(signed: bool, is_mod: bool) -> DivModLeaf {
        DivModLeaf { params: vec![0x09EA, 0x09EB], is_mod, signed }
    }

    fn words(d: &DivModLeaf, mode: OptMode) -> Vec<u32> {
        div_mod_leaf_text(d, mode)
            .unwrap()
            .chunks_exact(4)
            .map(|c| u32::from_be_bytes([c[0], c[1], c[2], c[3]]))
            .collect()
    }

    /// **The four `/O1` bodies, word for word**, transcribed from
    /// `work/w-divmod/twigrid.py --dis s-mod-var s-div-var u-mod-var u-div-var`
    /// at `/O1 /GS- /c` — the dc3 workload's own optimization mode.
    #[test]
    fn the_four_o1_bodies_are_reproduced_word_for_word() {
        assert_eq!(
            words(&leaf(true, true), OptMode::O1),
            vec![
                0x546b_083e, // rotlwi r11,r3,1
                0x7d43_23d6, // divw   r10,r3,r4
                0x396b_ffff, // addi   r11,r11,-1
                0x7d4a_21d6, // mullw  r10,r10,r4
                0x7c8b_5878, // andc   r11,r4,r11
                0x0cc4_0000, // twi    6,r4,0
                0x7c6a_1850, // subf   r3,r10,r3
                0x0cab_ffff, // twi    5,r11,-1
                0x4e80_0020, // blr
            ]
        );
        assert_eq!(
            words(&leaf(true, false), OptMode::O1),
            vec![
                0x546b_083e, // rotlwi r11,r3,1
                0x7c63_23d6, // divw   r3,r3,r4
                0x396b_ffff, // addi   r11,r11,-1
                0x0cc4_0000, // twi    6,r4,0
                0x7c8b_5878, // andc   r11,r4,r11
                0x0cab_ffff, // twi    5,r11,-1
                0x4e80_0020, // blr
            ]
        );
        assert_eq!(
            words(&leaf(false, true), OptMode::O1),
            vec![
                0x7d63_2396, // divwu r11,r3,r4
                0x0cc4_0000, // twi   6,r4,0
                0x7d6b_21d6, // mullw r11,r11,r4
                0x7c6b_1850, // subf  r3,r11,r3
                0x4e80_0020, // blr
            ]
        );
        assert_eq!(
            words(&leaf(false, false), OptMode::O1),
            vec![
                0x7c63_2396, // divwu r3,r3,r4
                0x0cc4_0000, // twi   6,r4,0
                0x4e80_0020, // blr
            ]
        );
    }

    /// **The four `/Ox` bodies**, from the same script at `/Ox /GS- /c`. Three
    /// of them differ from `/O1` in register fields only; the fourth is
    /// byte-identical, and that asymmetry is the point of asserting all four.
    #[test]
    fn the_four_ox_bodies_are_reproduced_word_for_word() {
        assert_eq!(
            words(&leaf(true, true), OptMode::Ox),
            vec![
                0x546b_083e, // rotlwi r11,r3,1
                0x7d43_23d6, // divw   r10,r3,r4
                0x392b_ffff, // addi   r9,r11,-1
                0x7d0a_21d6, // mullw  r8,r10,r4
                0x7c87_4878, // andc   r7,r4,r9
                0x0cc4_0000, // twi    6,r4,0
                0x7c68_1850, // subf   r3,r8,r3
                0x0ca7_ffff, // twi    5,r7,-1
                0x4e80_0020, // blr
            ]
        );
        assert_eq!(
            words(&leaf(true, false), OptMode::Ox),
            vec![
                0x546b_083e, // rotlwi r11,r3,1
                0x7c63_23d6, // divw   r3,r3,r4
                0x396b_ffff, // addi   r11,r11,-1
                0x0cc4_0000, // twi    6,r4,0
                0x7c8a_5878, // andc   r10,r4,r11
                0x0caa_ffff, // twi    5,r10,-1
                0x4e80_0020, // blr
            ]
        );
        assert_eq!(
            words(&leaf(false, true), OptMode::Ox),
            vec![
                0x7d63_2396, // divwu r11,r3,r4
                0x0cc4_0000, // twi   6,r4,0
                0x7d4b_21d6, // mullw r10,r11,r4
                0x7c6a_1850, // subf  r3,r10,r3
                0x4e80_0020, // blr
            ]
        );
    }

    /// **`unsigned /` is the mode control**: it is the one shape of the four
    /// whose two modes are byte-identical, measured, and a change that made the
    /// mode table uniform in either direction would fail here.
    #[test]
    fn unsigned_divide_is_byte_identical_across_the_modes_and_the_others_are_not() {
        assert_eq!(
            words(&leaf(false, false), OptMode::O1),
            words(&leaf(false, false), OptMode::Ox),
        );
        for (signed, is_mod) in [(true, true), (true, false), (false, true)] {
            let d = leaf(signed, is_mod);
            assert_ne!(
                words(&d, OptMode::O1),
                words(&d, OptMode::Ox),
                "({signed}, {is_mod}) must differ across the modes"
            );
        }
    }

    /// The four shapes are four **lengths**, and a change that folded `/` into
    /// `%` with a flag would fail here before it reached an obj.
    #[test]
    fn each_shape_has_its_own_measured_length() {
        for (signed, is_mod, n) in
            [(true, true, 9), (true, false, 7), (false, true, 5), (false, false, 3)]
        {
            for mode in [OptMode::O1, OptMode::Ox] {
                assert_eq!(words(&leaf(signed, is_mod), mode).len(), n);
            }
        }
    }

    /// The arity gate is re-asserted in codegen, so a shape that ever reached
    /// here with a different formals list refuses rather than emitting the
    /// two-formal register plan over three registers.
    #[test]
    fn a_different_arity_refuses_in_codegen_too() {
        let mut d = leaf(true, true);
        d.params = vec![0x09EA];
        assert!(div_mod_leaf_text(&d, OptMode::O1).is_err());
        d.params = vec![0x09EA, 0x09EB, 0x09EC];
        assert!(div_mod_leaf_text(&d, OptMode::O1).is_err());
    }
}
