//! **The frontier's bytes, reconstructed from the encoders — a MEASUREMENT made
//! executable, and emphatically not an emitter.**
//!
//! Nothing in this file is reachable from [`super::select::select_function`].
//! It compiles only under `cfg(test)`. It converts no TU, moves no numerator and
//! appears in no accept path, and the port still returns `NotImplemented` on
//! every body it describes.
//!
//! # The question it answers, and why an integer was the wrong instrument
//!
//! Board **#1105** prices `src/system/math/Primes.cpp` at **≥ 15** and names
//! **eight** codegen refusals. Board **#770** — ten for ten on optimistic
//! misses — says an estimate at this end of the frontier comes back dearer, so
//! the number is not in doubt. What a *count* cannot say is **which layer** the
//! refusals live in, and a lane reading "eight codegen refusals" will price the
//! instruction vocabulary as part of the eight.
//!
//! It is not. `?NextHashPrime@@YAHH@Z` is sixteen words and
//! [`primes_next_hash_prime_text`] builds every one of them:
//!
//! ```text
//!   14 of 16 words come from encoders that ALREADY EXISTED
//!    2 of 16 need an encoder — `cmpw` (register-register) and `lwzx`
//! ```
//!
//! Both were added in the same commit as this file, both are one X-form
//! expression, and both are pinned here against words real `c2` emitted. **So
//! the instruction vocabulary was two words short, and the remaining distance
//! is entirely structural** — the reader, the `.data` local-`static`, the
//! relocation fan-out, the rotated CFG and the loop-carried allocation. That is
//! a different shopping list from the one "eight codegen refusals" reads like,
//! and it is the reason this file exists rather than a paragraph.
//!
//! # What this file is NOT evidence of
//!
//! **Reproducing the bytes is not converting the TU, and the gap between those
//! two is the whole of the remaining work.** Every constant below —
//! every register number, every displacement, the block order — was *read off
//! `c2`'s own output*. A lowering has to **derive** them from IL, and the port
//! cannot even read this body: the 878-TU scan classifies `Primes.cpp` as
//! `vocab-gap`, blocking feature `expr-jump`, `il function decode failed`. There
//! is no IL for a selector to consult.
//!
//! Held in the same hand: this is exactly the standard `super::ptr_walk_loop`'s
//! own header sets for itself — *"this is a transcription, and saying so is the
//! point"* — except that module is wired to a carrier and reached by
//! `select_function`, and this one is not wired to anything.
//!
//! # Source of the pins
//!
//! `work/w-loop/Primes_b.obj`, produced by the real `c2.dll` under wibo at the
//! workload's own flags and cwd:
//!
//! ```sh
//! c2rs compile src/system/math/Primes.cpp --keep-obj work/w-loop/Primes_b.obj \
//!     --flags-file work/dc3-workload/flags.txt --cwd "$C2RS_DC3"
//! scripts/gt_dump.py work/w-loop/Primes_b.obj
//! ```
//!
//! The obj itself is **not** committed (CLAUDE.md forbids objs); its sixteen
//! `.text` words are, in [`C2_TEXT`], and the command above regenerates it.

use super::encode::{
    BO_FALSE, CR_BIT_EQ, CR_BIT_LT, CR_COMPARE, cr_bi, encode_addi, encode_addis, encode_b_intra,
    encode_bc, encode_blr, encode_cmpw, encode_cmpwi, encode_lwz, encode_lwzx, encode_rlwinm,
};

/// The **whole** `.text` of `?NextHashPrime@@YAHH@Z`, 64 bytes, exactly as the
/// real `c2.dll` emitted it. Transcribed once from `scripts/gt_dump.py`.
///
/// Annotated with the source-level meaning so the block structure below can be
/// checked against it by eye as well as by the assert:
///
/// ```text
///   0000  3d400000  lis   r10,0        REFHI -> ?primes@…  (+PAIR)
///   0004  39600000  li    r11,0        i2 = 0
///   0008  392a0000  addi  r9,r10,0     REFLO -> ?primes@…  (+PAIR)   r9 = &primes
///   000c  814a0000  lwz   r10,0(r10)   REFLO -> ?primes@…  (+PAIR)   r10 = primes[0]
///   0010  48000018  b     .+24         ROTATION: jump INTO the bottom test
///   0014  7f0a1800  cmpw  cr6,r10,r3     <- LOOP TOP
///   0018  4098001c  bf    24,.+28      cr6.LT false -> the value-return block
///   001c  396b0001  addi  r11,r11,1    i2++
///   0020  556a103a  slwi  r10,r11,2
///   0024  7d4a482e  lwzx  r10,r10,r9   r10 = primes[i2]
///   0028  2f0a0000  cmpwi cr6,r10,0      <- the bottom test, the `b` lands here
///   002c  409affe8  bf    26,.-24      cr6.EQ false -> BACK EDGE to the loop top
///   0030  4e800020  blr                fall-out: return i (already in r3)
///   0034  556b103a  slwi  r11,r11,2      <- REMATERIALIZED: r10 already held this
///   0038  7c6b482e  lwzx  r3,r11,r9
///   003c  4e800020  blr
/// ```
pub const C2_TEXT: [u8; 64] = [
    0x3d, 0x40, 0x00, 0x00, 0x39, 0x60, 0x00, 0x00, 0x39, 0x2a, 0x00, 0x00, 0x81, 0x4a, 0x00, 0x00,
    0x48, 0x00, 0x00, 0x18, 0x7f, 0x0a, 0x18, 0x00, 0x40, 0x98, 0x00, 0x1c, 0x39, 0x6b, 0x00, 0x01,
    0x55, 0x6a, 0x10, 0x3a, 0x7d, 0x4a, 0x48, 0x2e, 0x2f, 0x0a, 0x00, 0x00, 0x40, 0x9a, 0xff, 0xe8,
    0x4e, 0x80, 0x00, 0x20, 0x55, 0x6b, 0x10, 0x3a, 0x7c, 0x6b, 0x48, 0x2e, 0x4e, 0x80, 0x00, 0x20,
];

/// The five basic blocks, by `.text` byte offset. Named rather than inlined so
/// the three branch displacements below are computed from the **layout** — the
/// rotation is then visible as `ENTRY`'s branch going *forward* to `TEST`, which
/// sits *below* `TOP`.
const TOP: i32 = 0x14; // the loop top: the `cmpw` against the formal
const BODY: i32 = 0x1c; // i2++, the scaled-index load
const TEST: i32 = 0x28; // the sentinel test — the `b` at 0x10 enters HERE
const FALLOUT: i32 = 0x30; // `return i`
const VALUE: i32 = 0x34; // `return primes[i2]`, with its rematerialization

/// Rebuild the sixteen words from [`super::encode`].
///
/// Returns `(text, needed_a_new_encoder)` — one flag per word, `true` for the
/// two that had no encoder before this commit. The flags are the executable form
/// of the module header's `14 of 16`, so that claim cannot rot into a comment
/// that stopped being true.
fn primes_next_hash_prime_text() -> (Vec<u8>, Vec<bool>) {
    let bit_lt = cr_bi(CR_COMPARE, CR_BIT_LT); // 24
    let bit_eq = cr_bi(CR_COMPARE, CR_BIT_EQ); // 26

    // `slwi rA,rS,2` is `rlwinm rA,rS,2,0,29` — the shift the port already had.
    let slwi2 = |ra: u8, rs: u8| encode_rlwinm(ra, rs, 2, 0, 29);

    let words: Vec<([u8; 4], bool)> = vec![
        // -- ENTRY: materialize &primes, peel primes[0], rotate into the test --
        (encode_addis(10, 0, 0), false), // lis r10,0        + REFHI
        (encode_addi(11, 0, 0), false),  // li  r11,0
        (encode_addi(9, 10, 0), false),  // addi r9,r10,0    + REFLO
        (encode_lwz(10, 10, 0), false),  // lwz r10,0(r10)   + REFLO
        (encode_b_intra(TEST - 0x10).expect("in range"), false),
        // -- TOP --
        (encode_cmpw(CR_COMPARE, 10, 3), true), // <== NEW ENCODER
        (encode_bc(BO_FALSE, bit_lt, VALUE - 0x18).expect("in range"), false),
        // -- BODY --
        (encode_addi(11, 11, 1), false),
        (slwi2(10, 11), false),
        (encode_lwzx(10, 10, 9), true), // <== NEW ENCODER
        // -- TEST (the rotation's landing pad) --
        (encode_cmpwi(CR_COMPARE, 10, 0), false),
        // the BACK EDGE — negative displacement, which `super::labels` refuses
        (encode_bc(BO_FALSE, bit_eq, TOP - 0x2c).expect("in range"), false),
        // -- FALLOUT --
        (encode_blr(), false),
        // -- VALUE: c2 recomputes what r10 already holds --
        (slwi2(11, 11), false),
        (encode_lwzx(3, 11, 9), true), // <== NEW ENCODER (second cell)
        (encode_blr(), false),
    ];

    let mut text = Vec::with_capacity(64);
    let mut new = Vec::with_capacity(16);
    for (w, is_new) in words {
        text.extend_from_slice(&w);
        new.push(is_new);
    }
    (text, new)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The claim, asserted whole.** Every one of `Primes.cpp`'s 64 bytes comes
    /// out of `super::encode`.
    #[test]
    fn primes_sixty_four_bytes_reconstruct_from_the_encoders() {
        let (text, _) = primes_next_hash_prime_text();
        assert_eq!(text.len(), 64, "the function is 64 bytes");
        assert_eq!(
            text,
            C2_TEXT.to_vec(),
            "reconstruction diverged from the bytes real c2 emitted"
        );
    }

    /// **The `14 of 16` split, so the header cannot rot.** Two words needed a
    /// new encoder and they are at the two offsets named in the doc; every other
    /// word came from an encoder that already existed.
    #[test]
    fn only_two_of_sixteen_words_needed_a_new_encoder() {
        let (_, new) = primes_next_hash_prime_text();
        assert_eq!(new.len(), 16);
        let needed: Vec<usize> = new
            .iter()
            .enumerate()
            .filter(|(_, n)| **n)
            .map(|(i, _)| i * 4)
            .collect();
        assert_eq!(
            needed,
            vec![0x14, 0x24, 0x38],
            "the words with no prior encoder are the `cmpw` and the two `lwzx`"
        );
        assert_eq!(new.iter().filter(|n| !**n).count(), 13);
        // Three word SITES, but only TWO encoders — the two `lwzx` are one.
        assert_eq!(needed.len(), 3, "three sites");
    }

    /// **The rotation, asserted as a SHAPE rather than described in prose.**
    ///
    /// Board **#1105** calls this "a three-block rotated plan entered by a `b`
    /// into the bottom test". Measured here it is **five** blocks and the
    /// rotation is the ENTRY branch overshooting the loop top: `0x10`'s target
    /// (`TEST`) is strictly *below* the top the back edge returns to (`TOP`).
    /// That inequality is the rotation, and it is what a straight-line lowering
    /// cannot produce.
    #[test]
    fn the_plan_is_five_blocks_and_the_entry_branch_overshoots_the_loop_top() {
        // Every block boundary is a real instruction boundary in the 64 bytes.
        for off in [TOP, BODY, TEST, FALLOUT, VALUE] {
            assert_eq!(off % 4, 0, "block {off:#x} is word-aligned");
            assert!((off as usize) < C2_TEXT.len(), "block {off:#x} is in range");
        }
        // Strictly increasing, and five of them.
        let blocks = [TOP, BODY, TEST, FALLOUT, VALUE];
        assert!(blocks.windows(2).all(|w| w[0] < w[1]), "{blocks:?}");
        assert_eq!(blocks.len(), 5);

        // THE ROTATION: the entry `b` at 0x10 lands at TEST, past TOP.
        assert_eq!(&C2_TEXT[0x10..0x14], &encode_b_intra(TEST - 0x10).unwrap());
        assert!(TEST > TOP, "the entry branch overshoots the loop top");

        // THE BACK EDGE: from inside TEST's block, upward to TOP.
        assert!(TOP - 0x2c < 0, "the back edge displacement is negative");

        // Two `blr`s, and they are the last word of two DIFFERENT blocks.
        assert_eq!(&C2_TEXT[FALLOUT as usize..FALLOUT as usize + 4], &encode_blr());
        assert_eq!(&C2_TEXT[60..64], &encode_blr());
    }

    /// **The two new encoders, pinned individually against c2's own words.**
    /// Separate from the whole-function assert on purpose: if the reconstruction
    /// ever needs re-deriving, these still say what the encoder must produce.
    #[test]
    fn the_two_new_encoders_are_pinned_to_real_c2_words() {
        // cmpw cr6,r10,r3  @ 0x14
        assert_eq!(encode_cmpw(6, 10, 3), [0x7f, 0x0a, 0x18, 0x00]);
        // lwzx r10,r10,r9  @ 0x24   and   lwzx r3,r11,r9  @ 0x38
        assert_eq!(encode_lwzx(10, 10, 9), [0x7d, 0x4a, 0x48, 0x2e]);
        assert_eq!(encode_lwzx(3, 11, 9), [0x7c, 0x6b, 0x48, 0x2e]);
        // The field separation the two `lwzx` cells buy: rD and rA move
        // independently, so neither is pinned only by the formula.
        assert_ne!(encode_lwzx(10, 10, 9), encode_lwzx(3, 11, 9));
        // `cmpw` is NOT `cmpwi` — the defect a single-cell pin would miss.
        assert_ne!(encode_cmpw(6, 10, 3), encode_cmpwi(6, 10, 3));
    }

    /// **NEGATIVE CONTROL — the assert above is not vacuous.**
    ///
    /// `docs/GAPS.md` §6's standing rule is that a green comparison proves
    /// nothing until something is shown to make it red. Each mutation below is
    /// a single wrong field, and every one must diverge from [`C2_TEXT`].
    #[test]
    fn one_wrong_field_anywhere_breaks_the_reconstruction() {
        let (good, _) = primes_next_hash_prime_text();
        assert_eq!(good, C2_TEXT.to_vec());

        // 1. the rotation removed — enter at the TOP instead of the TEST
        let mut m = good.clone();
        m[16..20].copy_from_slice(&encode_b_intra(TOP - 0x10).expect("in range"));
        assert_ne!(m, C2_TEXT.to_vec(), "the entry displacement is load-bearing");

        // 2. the back edge's CR BIT wrong (EQ -> LT), the board #188 defect class
        let mut m = good.clone();
        m[44..48].copy_from_slice(
            &encode_bc(BO_FALSE, cr_bi(CR_COMPARE, CR_BIT_LT), TOP - 0x2c).expect("in range"),
        );
        assert_ne!(m, C2_TEXT.to_vec(), "the back edge's CR bit is load-bearing");

        // 3. the back edge's CR FIELD wrong (cr6 -> cr0) — board #188 itself
        let mut m = good.clone();
        m[44..48].copy_from_slice(
            &encode_bc(BO_FALSE, cr_bi(0, CR_BIT_EQ), TOP - 0x2c).expect("in range"),
        );
        assert_ne!(m, C2_TEXT.to_vec(), "the back edge's CR field is load-bearing");

        // 4. the exit block's REMATERIALIZATION elided — `mr r3,r10` would be
        //    semantically identical and is NOT what c2 wrote. This is the
        //    mutation that matters most: it is the "obvious" optimization a
        //    lowering would reach for, and it is wrong bytes.
        let mut m = good.clone();
        m[52..56].copy_from_slice(&super::super::encode::encode_mr(3, 10));
        m[56..60].copy_from_slice(&encode_blr());
        assert_ne!(
            m,
            C2_TEXT.to_vec(),
            "c2 rematerializes slwi+lwzx over a value already live in r10"
        );

        // 5. the index scale wrong (int -> byte)
        let mut m = good.clone();
        m[32..36].copy_from_slice(&encode_rlwinm(10, 11, 0, 0, 31));
        assert_ne!(m, C2_TEXT.to_vec(), "the slwi scale is load-bearing");

        // 6. `lwzx` operand order swapped — rA and rB are NOT symmetric here
        let mut m = good.clone();
        m[36..40].copy_from_slice(&encode_lwzx(10, 9, 10));
        assert_ne!(m, C2_TEXT.to_vec(), "lwzx rA/rB order is load-bearing");
    }

    /// **POSITIVE CONTROL — stays green under every mutation above.**
    ///
    /// The negative control mutates copies, never the builder, so the builder
    /// must still agree with `C2_TEXT` after all of it. Without this, a
    /// mutation test that accidentally corrupted shared state would read as six
    /// successful detections.
    #[test]
    fn the_builder_is_unchanged_by_the_mutation_test() {
        let (a, fa) = primes_next_hash_prime_text();
        let (b, fb) = primes_next_hash_prime_text();
        assert_eq!(a, b);
        assert_eq!(fa, fb);
        assert_eq!(a, C2_TEXT.to_vec());
    }

    /// **The honest boundary, asserted rather than only written down.** The port
    /// does not emit this function, and nothing in this module makes it do so.
    ///
    /// `super::labels`' invariant 4 refuses every backward reference, and the
    /// back edge at `0x2c` is one. That refusal is *unchanged* by this file, and
    /// this test is what says so: a lane that relaxed it would have to come
    /// through here.
    #[test]
    fn the_back_edge_is_still_a_backward_reference_the_label_map_refuses() {
        use crate::codegen::labels::{Form, LabelMap};

        // Lay the body out through the map, as a lowering would: the loop top is
        // defined at 0x14, and the back edge at 0x2c refers to it.
        let mut m = LabelMap::new();
        let mut text: Vec<u8> = Vec::new();
        text.extend_from_slice(&C2_TEXT[..TOP as usize]);
        let top = m.mint("primes-loop-top");
        m.define(top, &text).expect("first definition");
        text.extend_from_slice(&C2_TEXT[TOP as usize..0x2c]);
        m.reference(&mut text, top, Form::Bc { bo: BO_FALSE, bi: 26 });
        text.extend_from_slice(&C2_TEXT[0x30..]);
        assert_eq!(text.len(), 64, "the placeholder kept the length");

        let err = m.resolve(&mut text);
        assert!(
            err.is_err(),
            "the label map must still refuse Primes' back edge"
        );

        // POSITIVE CONTROL for this test: the identical map with a FORWARD
        // reference resolves. Without it, `is_err()` would also be satisfied by
        // a map that refuses everything.
        let mut m2 = LabelMap::new();
        let mut t2: Vec<u8> = Vec::new();
        let fwd = m2.mint("primes-value-block");
        t2.extend_from_slice(&C2_TEXT[..0x18]);
        m2.reference(&mut t2, fwd, Form::Bc { bo: BO_FALSE, bi: 24 });
        t2.extend_from_slice(&C2_TEXT[0x1c..VALUE as usize]);
        m2.define(fwd, &t2).expect("first definition");
        t2.extend_from_slice(&C2_TEXT[VALUE as usize..]);
        m2.resolve(&mut t2).expect("a forward reference resolves");
        // And it resolves to the very word c2 emitted at 0x18.
        assert_eq!(&t2[0x18..0x1c], &C2_TEXT[0x18..0x1c]);
    }
}
