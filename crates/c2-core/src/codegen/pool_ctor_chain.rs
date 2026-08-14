//! **W-POOL2 — the free-list constructor's chain build**, twenty words.
//!
//! ```text
//!   ??0Pool@@QAA@HPAXH@Z    80 B     this r3 · size r4 · base r5 · total r6
//!     54ca083e  rotlwi 10,6,1        the divide's overflow helper, HOISTED
//!     90a30000  stw    5,0(3)          above the member-init store
//!     39640003  addi   11,4,3        (size + 3)
//!     392affff  addi   9,10,-1
//!     556b003a  rlwinm 11,11,0,0,29  ... & ~3      -> stride
//!     7d465bd6  divw   10,6,11       total / stride -> count
//!     7d694878  andc   9,11,9
//!     0ccb0000  twi    6,11,0        trap: stride == 0
//!     0ca9ffff  twi    5,9,-1        trap: INT_MIN / -1
//!     2f0a0001  cmpwi  cr6,10,1      the SOURCE's own `if (count > 1)`
//!     4099001c  bf     25,.+28
//!     394affff  addi   10,10,-1      n = count - 1
//!     7d4903a6  mtctr  10            ... and n IS the trip count
//!     7d4b2a14  add    10,11,5       next = ptr + stride       <- loop top
//!     91450000  stw    10,0(5)       *(char**)ptr = next
//!     7d455378  mr     5,10          ptr = next
//!     4200fff4  bdnz   .-12
//!     39600000  li     11,0
//!     91650000  stw    11,0(5)       *(char**)ptr = 0
//!     4e800020  blr
//! ```
//!
//! Every word is off this lane's own capture (`work/w-pool2/ref/Pool.obj`,
//! `scripts/gt_dump.py`) at the workload's flags.
//!
//! # THIS IS A TRANSCRIPTION, and the schedule is the part that is not derived
//!
//! `codegen::div_mod_leaf` transcribes the five signed-divide words as a
//! **contiguous constant body at r11/r3/r4**. Here the identical five —
//! `rotlwi` / `addi −1` / `andc` / `twi 6` / `twi 5` — sit at **r10/r9/r11 and
//! are split across four unrelated instructions**: the member-init `stw`, the
//! `addi +3`, the `rlwinm` mask and the `divw` itself. That is c2's scheduler
//! and its allocator, and this file models neither; it reproduces one body.
//! `div_mod_leaf`'s own header says it in as many words — *"There is no
//! scheduler here and no register allocator"* — and it is the honest standing
//! for this file too. A body that is not exactly
//! [`c2_il::func::body::shapes::pool_ctor_chain`]'s statement list is refused,
//! never scheduled.
//!
//! The `twi 6` **placement rule** `w-divmod` measured over 161 cells is
//! deliberately not consulted: its hoisting clause needs the dividend to be
//! produced inside the division's own block, and here `total` is live-in while
//! `stride` is produced in the same block — a third regime, on the far side of
//! both of that rule's branches. Reading it here would be applying a
//! description of c2's scheduler outside the population it was fitted on.
//!
//! # The register plan, and what is derived from what
//!
//! * `this` r3, `size` r4, `base` r5, `total` r6 — the ABI, from the slot
//!   indices.
//! * `stride` **r11**, `count` **r10**, the overflow temp **r9** —
//!   `WB_REGALLOC_FINDINGS` §3.4's descending scratch order, and it holds here.
//! * `ptr` reuses **r5**, the `base` formal's own register, and `next` reuses
//!   **r10** once `count` is dead in `ctr`. Both are live-range reuse, both are
//!   transcribed.
//!
//! # The two things `WB_LOOP_FINDINGS` §9 says a loop class owes, and does not here
//!
//! The trip-count arithmetic (§9 item 4, unread for a **non-unit** step) is not
//! owed: the step is `--n`, and `n` *is* the trip count. The rotated zero-trip
//! guard (§7.7 rule 1) is not owed either: `cmpwi cr6,10,1 ; bf 25` is the
//! source's own `if (count > 1)`, read out of the IL's `24` / `38 <label>`.
//! What is used is `mtctr`/`bdnz` itself, `wb-loop` #1900.
//!
//! # `/O1` only
//!
//! `/Ox` is **twenty-one** words with an extra `mr r11,r5` and the plan
//! r9/r10/r8/r7/r11 — `work/w-pool2/ref/PoolOx.obj`. Asked in the parser
//! (#1638) and again here.

use c2_il::PoolCtorChain;

use crate::codegen::encode::{
    cr_bi, encode_add, encode_addi, encode_andc, encode_bdnz, encode_blr,
    encode_cmpwi, encode_divw, encode_mr, encode_mtctr, encode_rlwinm, encode_stw, encode_twi,
    BO_FALSE, CR_BIT_GT, CR_COMPARE,
};
use crate::codegen::select::{out_of_class, OptMode};
use crate::BackendError;
use crate::codegen::labels::Form;
use crate::codegen::reach;

/// `this` — the object, r3.
const R_THIS: u8 = 3;
/// The block size formal, r4.
const R_SIZE: u8 = 4;
/// The arena base formal, r5 — and the walking `ptr`, which reuses it.
const R_PTR: u8 = 5;
/// The arena size formal, r6 — the dividend.
const R_TOTAL: u8 = 6;
/// `stride`, the divisor, and the first scratch.
const R_STRIDE: u8 = 11;
/// `count`, then `next` once `count` has moved into `ctr`.
const R_COUNT: u8 = 10;
/// The divide's overflow predicate.
const R_OVF: u8 = 9;

/// `TO = 6` — traps when the divisor is zero. Named identically to
/// [`crate::codegen::div_mod_leaf`]'s, where the bit meaning is derived.
const TO_DIV_BY_ZERO: u8 = 6;
/// `TO = 5` — traps on `INT_MIN / -1`.
const TO_OVERFLOW: u8 = 5;

/// The number of words from the `bf` to its target: the four-word loop body,
/// the two words that set it up, and the `bdnz`.
const GUARD_SKIP_BYTES: i32 = 28;
/// The back edge: three words up, to `add r10,r11,r5`.
const BACK_EDGE_BYTES: i32 = -12;

pub(crate) fn pool_ctor_chain_text(
    c: &PoolCtorChain,
    mode: OptMode,
) -> Result<Vec<u8>, BackendError> {
    // Restated from the recognizer for #1638's reason: `select_function` is what
    // `function_gate` runs.
    if mode != OptMode::O1 {
        return Err(out_of_class(
            "the free-list constructor at `/Ox`: twenty-one words with a different \
             register plan, and this lane graded only the `/O1` twenty",
        ));
    }
    if c.params.len() != 4 {
        return Err(out_of_class(
            "a free-list constructor with other than `this` and three formals: every \
             register in this body is a slot index",
        ));
    }
    if c.align != 4 {
        return Err(out_of_class(
            "a round-up alignment other than 4: the `+ (align-1)` addend and the \
             `rlwinm` MB/ME pair are a MATCHED pair and only one of them was graded",
        ));
    }
    let off = i16::try_from(c.off).map_err(|_| {
        out_of_class("a free-list head member beyond the `stw` displacement")
    })?;
    // `& ~(align-1)` as a mask: clear the low `log2(align)` bits, i.e. keep bits
    // 0..=31-log2. Written as arithmetic rather than as the literal 29 so the
    // `align != 4` refusal above is the only thing standing between this file
    // and a second alignment, and it is one clause instead of a second constant.
    let me = 31 - c.align.trailing_zeros() as u8;
    let addend = i16::try_from(c.align - 1).expect("align 4");

    let mut t: Vec<u8> = Vec::with_capacity(80);
    // ---- the entry block --------------------------------------------------
    // The overflow helper first — HOISTED above the member-init store, which is
    // the single most visible thing c2's scheduler does to this body.
    t.extend_from_slice(&encode_rlwinm(R_COUNT, R_TOTAL, 1, 0, 31));
    t.extend_from_slice(&encode_stw(R_PTR, R_THIS, off));
    t.extend_from_slice(&encode_addi(R_STRIDE, R_SIZE, addend));
    t.extend_from_slice(&encode_addi(R_OVF, R_COUNT, -1));
    t.extend_from_slice(&encode_rlwinm(R_STRIDE, R_STRIDE, 0, 0, me));
    t.extend_from_slice(&encode_divw(R_COUNT, R_TOTAL, R_STRIDE));
    t.extend_from_slice(&encode_andc(R_OVF, R_STRIDE, R_OVF));
    t.extend_from_slice(&encode_twi(TO_DIV_BY_ZERO, R_STRIDE, 0));
    t.extend_from_slice(&encode_twi(TO_OVERFLOW, R_OVF, -1));

    // ---- the source's own `if (count > 1)` --------------------------------
    t.extend_from_slice(&encode_cmpwi(CR_COMPARE, R_COUNT, 1));
    t.extend_from_slice(&reach::direct(
        Form::Bc { bo: BO_FALSE, bi: cr_bi(CR_COMPARE, CR_BIT_GT) },
        GUARD_SKIP_BYTES,
        "the pool-ctor guard",
    )?);

    // ---- the preheader: `n = count - 1`, and n IS the trip count ----------
    t.extend_from_slice(&encode_addi(R_COUNT, R_COUNT, -1));
    t.extend_from_slice(&encode_mtctr(R_COUNT));

    // ---- the loop body, four words ---------------------------------------
    t.extend_from_slice(&encode_add(R_COUNT, R_STRIDE, R_PTR));
    t.extend_from_slice(&encode_stw(R_COUNT, R_PTR, 0));
    t.extend_from_slice(&encode_mr(R_PTR, R_COUNT));
    t.extend_from_slice(
        &encode_bdnz(BACK_EDGE_BYTES)
            .ok_or_else(|| out_of_class("the back edge does not fit a `bdnz`"))?,
    );

    // ---- the terminating null link ---------------------------------------
    // `li r11,0` is `addi r11,r0,0`; r11 is free again because `stride` is dead
    // once the loop is over.
    t.extend_from_slice(&encode_addi(R_STRIDE, 0, 0));
    t.extend_from_slice(&encode_stw(R_STRIDE, R_PTR, 0));
    t.extend_from_slice(&encode_blr());

    debug_assert_eq!(t.len(), 80, "the class's body length is a constant");
    Ok(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctor(off: i32) -> PoolCtorChain {
        PoolCtorChain { params: vec![1, 2, 3, 4], off, align: 4 }
    }

    fn words(t: &[u8]) -> Vec<u32> {
        t.chunks(4).map(|c| u32::from_be_bytes([c[0], c[1], c[2], c[3]])).collect()
    }

    /// `??0Pool@@QAA@HPAXH@Z`, word for word off `work/w-pool2/ref/Pool.obj`.
    /// Twenty words, and the assertion is the whole body rather than a length —
    /// a length would pass for any schedule of the same instructions, and the
    /// schedule is exactly what this file transcribes.
    #[test]
    fn the_body_is_pool_ctor_word_for_word() {
        let t = pool_ctor_chain_text(&ctor(0), OptMode::O1).unwrap();
        assert_eq!(
            words(&t),
            vec![
                0x54ca_083e, // rotlwi 10,6,1
                0x90a3_0000, // stw    5,0(3)
                0x3964_0003, // addi   11,4,3
                0x392a_ffff, // addi   9,10,-1
                0x556b_003a, // rlwinm 11,11,0,0,29
                0x7d46_5bd6, // divw   10,6,11
                0x7d69_4878, // andc   9,11,9
                0x0ccb_0000, // twi    6,11,0
                0x0ca9_ffff, // twi    5,9,-1
                0x2f0a_0001, // cmpwi  cr6,10,1
                0x4099_001c, // bf     25,+28
                0x394a_ffff, // addi   10,10,-1
                0x7d49_03a6, // mtctr  10
                0x7d4b_2a14, // add    10,11,5
                0x9145_0000, // stw    10,0(5)
                0x7d45_5378, // mr     5,10
                0x4200_fff4, // bdnz   -12
                0x3960_0000, // li     11,0
                0x9165_0000, // stw    11,0(5)
                0x4e80_0020, // blr
            ]
        );
    }

    /// The guard skips exactly the seven words between it and the null link,
    /// and the back edge lands on the loop's first word. Asserted as
    /// *arithmetic over the emitted body* rather than as the two constants, so
    /// a body length change cannot leave the displacements behind silently.
    #[test]
    fn the_two_displacements_agree_with_the_body_they_span() {
        let t = pool_ctor_chain_text(&ctor(0), OptMode::O1).unwrap();
        let w = words(&t);
        let bf_at = 10usize;
        let target = 18usize; // `stw r11,0(r5)`… no: the `li`, index 17
        assert_eq!(w[bf_at] & 0xFFFF, GUARD_SKIP_BYTES as u32);
        assert_eq!(bf_at + (GUARD_SKIP_BYTES as usize / 4), 17);
        let bdnz_at = 16usize;
        assert_eq!((w[bdnz_at] & 0xFFFF) as i16 as i32, BACK_EDGE_BYTES);
        assert_eq!(bdnz_at as i32 + BACK_EDGE_BYTES / 4, 13);
        let _ = target;
    }

    /// The member offset is the only free field on the emitted words, and it
    /// moves exactly the member-init `stw` — the loop threads the arena, not
    /// the object, so its two `stw`s must NOT move.
    #[test]
    fn the_member_offset_moves_only_the_member_init_store() {
        let base = words(&pool_ctor_chain_text(&ctor(0), OptMode::O1).unwrap());
        let moved = words(&pool_ctor_chain_text(&ctor(4), OptMode::O1).unwrap());
        for (i, (a, b)) in base.iter().zip(moved.iter()).enumerate() {
            if i == 1 {
                assert_eq!(*b, *a + 4, "the member-init store carries the offset");
            } else {
                assert_eq!(a, b, "word {i} must not move with the member offset");
            }
        }
    }

    #[test]
    fn the_class_is_o1_only() {
        assert!(pool_ctor_chain_text(&ctor(0), OptMode::O1).is_ok());
        assert!(pool_ctor_chain_text(&ctor(0), OptMode::Ox).is_err());
    }

    #[test]
    fn an_ungraded_alignment_or_arity_refuses() {
        let mut c = ctor(0);
        c.align = 8;
        assert!(pool_ctor_chain_text(&c, OptMode::O1).is_err());
        let mut c = ctor(0);
        c.params = vec![1, 2, 3];
        assert!(pool_ctor_chain_text(&c, OptMode::O1).is_err());
    }
}
