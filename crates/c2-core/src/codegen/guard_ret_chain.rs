//! **W-IFN — the emitter for a framed guard chain whose arms are `return K`
//! and whose spine is a block copy.**
//!
//! The reader's accept/refuse boundary and the source shapes are on
//! [`c2_il::func::body::shapes::guard_ret_chain`]; this file is the twenty-one
//! or twenty-seven words and nothing else. Everything variable in them is
//! named in [`c2_il::GuardRetChain`]: two guard formals, two guard literals,
//! the copy length and — for sub-shape **S** — two member offsets.
//!
//! Two sub-shapes, both from `src/xdk/nuispeech/mmio.cpp`, the frontier's top
//! byte-fraction row. **They are TRANSCRIBED per sub-shape and not unified**,
//! which is `w-blockir` board #2306's lesson taken rather than re-learned: that
//! lane registered one rule for its walker and one for its park, and the answer
//! was three constants both times. Two witnesses are not a rule.
//!
//! # Sub-shape G — `mmioGetInfo`, 84 bytes
//!
//! ```c
//!   R f(A a, B b, C c) { if (a == 0) return K0;
//!                        if (b == 0) return K1;
//!                        memcpy(b, a, N);        // dst = formal 1, src = formal 0
//!                        return 0; }
//! ```
//!
//! ```text
//!    off  word       instruction             why it is this word
//!   ----  --------   ---------------------   -----------------------------------
//!   0x00  7d8802a6   mflr  r12               FrameLayout{saved_gprs:0}: 96 bytes,
//!   0x04  9181fff8   stw   r12,-8(r1)        no callee-saved GPR — nothing in
//!   0x08  9421ffa0   stwu  r1,-96(r1)        this body outlives the `bl`
//!   0x0c  7c6b1b78   mr    r11,r3            THE PARK, and it is a SWAP: the copy
//!   0x10  7c832378   mr    r3,r4             wants dst in r3 and src in r4, and
//!                                            they arrive the other way round, so
//!                                            r11 is the scratch the swap needs
//!   0x14  2b0b0000   cmplwi cr6,r11,0        guard 0, on the formal the park
//!   0x18  409a000c   bf    26,-> next        moved to r11
//!   0x1c  38600005   li    r3,<K0>           the arm, IN SOURCE ORDER — measured,
//!   0x20  48000024   b     -> epilogue       `work/w-ifn/probe/blkorder.cpp`
//!   0x24  2b030000   cmplwi cr6,r3,0         guard 1, on the formal now in r3
//!   0x28  409a000c   bf    26,-> next
//!   0x2c  3860000b   li    r3,<K1>
//!   0x30  48000014   b     -> epilogue
//!   0x34  38a00048   li    r5,<N>            the third argument
//!   0x38  7d645b78   mr    r4,r11            the second — out of the swap scratch
//!   0x3c  4bffffc5   bl    memcpy   REL24    the first is already in r3
//!   0x40  38600000   li    r3,0
//!   0x44  38210060   addi  r1,r1,96          the MATERIALISED COMMON EPILOGUE:
//!   0x48  8181fff8   lwz   r12,-8(r1)        one block, reached from three
//!   0x4c  7d8803a6   mtlr  r12               places, and the thing
//!   0x50  4e800020   blr                     `Selected::Framed` has no
//!                                            representation for
//! ```
//!
//! # Sub-shape S — `mmioSetInfo`, 108 bytes
//!
//! ```c
//!   R f(A a, B b, C c) { if (a == 0) return K0;
//!                        if (b == 0) return K1;
//!                        memcpy(a, b, N);        // dst = formal 0, src = formal 1
//!                        M *m = (M *)a;
//!                        if (m->hi < m->lo) m->hi = m->lo;
//!                        return 0; }
//! ```
//!
//! The park is a **different plan, not a parameterisation of G's**: here the
//! destination is read again *after* the `bl`, so it cannot stay in a volatile,
//! and the frame grows one callee-saved GPR to hold it. The arguments already
//! arrive in the right registers, so there is no swap and no scratch.
//!
//! ```text
//!   0x00  7d8802a6   mflr  r12               FrameLayout{saved_gprs:1}: still 96
//!   0x04  9181fff8   stw   r12,-8(r1)        bytes — one save slot fits under
//!   0x08  fbe1fff0   std   r31,-16(r1)       the 16-byte rounding
//!   0x0c  9421ffa0   stwu  r1,-96(r1)
//!   0x10  7c7f1b78   mr    r31,r3            THE PARK: dst survives the call
//!   0x14  2b030000   cmplwi cr6,r3,0         guard 0 reads r3 itself — the park
//!   0x18  409a000c   bf    26,-> next        copied it, it did not move it
//!   0x1c  38600005   li    r3,<K0>
//!   0x20  48000038   b     -> epilogue
//!   0x24  2b040000   cmplwi cr6,r4,0         guard 1 reads r4, unparked
//!   0x28  409a000c   bf    26,-> next
//!   0x2c  3860000b   li    r3,<K1>
//!   0x30  48000028   b     -> epilogue
//!   0x34  38a00048   li    r5,<N>
//!   0x38  7fe3fb78   mr    r3,r31            dst back into r3; src is untouched
//!   0x3c  4bffffc5   bl    memcpy   REL24    in r4
//!   0x40  817f001c   lwz   r11,<lo>(r31)     the clamp, on the SECOND relational
//!   0x44  815f0020   lwz   r10,<hi>(r31)     regime: `cmplw` on two LOADED
//!   0x48  7f0a5840   cmplw cr6,r10,r11       values, and `bf 24` — the LT bit,
//!   0x4c  40980008   bf    24,-> skip        not the EQ bit every guard uses
//!   0x50  917f0020   stw   r11,<hi>(r31)
//!   0x54  38600000   li    r3,0
//!   0x58  38210060   addi  r1,r1,96
//!   0x5c  8181fff8   lwz   r12,-8(r1)
//!   0x60  7d8803a6   mtlr  r12
//!   0x64  ebe1fff0   ld    r31,-16(r1)
//!   0x68  4e800020   blr
//! ```
//!
//! # Three things that are measurements and not choices
//!
//! * **The guard compare is `cmplwi`, unsigned, because the operand is a
//!   POINTER.** The same source with an `int` formal emits `cmpwi`
//!   (`work/w-ifn/probe/blkorder.cpp` cell `b1`, `2f030000` against this
//!   class's `2b030000`). The reader pins the operand's type for that reason,
//!   and a class that reached for one compare form throughout would be right
//!   about the program and wrong about one word per guard.
//! * **The block order is SOURCE order and nothing is sunk.** Nine cells,
//!   `work/w-ifn/probe/blkorder.cpp`: a four-call arm stays where it is written
//!   and inverting the guard's sense inverts the emitted order with it. The one
//!   exception measured — a `||`-chained guard's shared arm IS sunk, and its
//!   branch flips to `bt` — is [`super::guard_chain_shared_tail`]'s shape and
//!   is refused here.
//! * **`memcpy` becomes a CALL, and the boundary is a step.** 25 cells at
//!   `/O1 /Oi` (`work/w-ifn/probe/mcpy.cpp`): `n <= 5` expands to loads and
//!   stores, every `n >= 6` is `bl memcpy`. The reader's accepted window is
//!   `6..=32767` and it refuses below the step rather than emitting a call c2
//!   would have expanded.
//!
//! Every branch here except the one `bl` is **self-relative** and therefore
//! independent of where the function lands in `.text`; only the `bl` encodes
//! its own offset, so it is the only word that needs `base_off`.

use crate::codegen::calls::encode_call_branch;
use crate::codegen::encode::{
    cr_bi, encode_addi, encode_cmplw, encode_cmplwi, encode_lwz,
    encode_mr, encode_stw, BO_FALSE, CR_BIT_EQ, CR_BIT_LT, CR_COMPARE,
};
use crate::codegen::frame::FrameLayout;
use crate::codegen::select::{fits_i16, out_of_class};
use crate::codegen::OptMode;
use crate::BackendError;
use c2_il::{GuardRetChain, GuardRetSpine};
use crate::codegen::labels::Form;
use crate::codegen::reach;

/// The condition-register field every guard and the clamp read. Literal, and
/// re-confirmed on this class's own objs rather than adopted from
/// `WB_REGALLOC_FINDINGS.md` — `2b0b0000` is `cmplwi cr6,r11,0`.
const GUARD_CRF: u8 = 6;

/// The scratch the sub-shape **G** swap parks the source formal in.
const SWAP_SCRATCH: u8 = 11;

/// The callee-saved register sub-shape **S** parks the destination in. It is
/// the only saved GPR, which is what makes that frame `saved_gprs: 1`.
const PARK_REG: u8 = 31;

/// The two registers the clamp loads into, in emission order.
const CLAMP_LO_REG: u8 = 11;
const CLAMP_HI_REG: u8 = 10;

/// The name c2 gives the block-copy helper. **Minted, not read**: the intrinsic
/// arrives as selector 172 on a `40` token and there is no `.gl` record for it —
/// `work/w-ifn/il/`'s capture of `mmio.cpp` has no `memcpy` string in its `.gl`
/// at all, while the obj carries it as an undefined external at symbol 19. So
/// this is the one name in this class that does not come out of the IL, and it
/// is a constant here rather than a field for exactly that reason.
pub const MEMCPY_NAME: &str = "memcpy";

/// `li rD,k` — `addi rD,0,k`. The same two-line helper
/// [`super::guard_chain_shared_tail`] and [`super::if_call_join`] carry.
fn encode_li(rd: u8, k: i16) -> [u8; 4] {
    encode_addi(rd, 0, k)
}

/// This class's emitted body: the bytes plus the offsets the writers need.
pub struct GuardRetChainBody {
    pub text: Vec<u8>,
    /// Absolute `.text` offset of the one `bl memcpy` (already includes
    /// `base_off`): the REL24 relocation site.
    pub bl_offset: u32,
    /// Prologue length in bytes: the `$M(n)` label's value and the `.pdata`
    /// record's `PrologLen`.
    pub prolog_len: u32,
}

/// The frame this class builds, per sub-shape.
///
/// `out_slots` is the widest call's argument count — `memcpy`'s three — which
/// is under the ABI floor of eight and therefore does not enter the size. It is
/// written as 3 rather than 8 so the layout says what the body does; both give
/// 96 bytes and [`FrameLayout::size`]'s own test pins the flooring.
fn frame_for(g: &GuardRetChain) -> FrameLayout {
    FrameLayout {
        locals: 0,
        out_slots: 3,
        saved_gprs: match g.spine {
            GuardRetSpine::Copy { .. } => 0,
            GuardRetSpine::CopyClamp { .. } => 1,
        },
        saved_fprs: 0,
    }
}

/// Emit the body.
///
/// `base_off` is the function's own offset within `.text` — zero under `/Gy`,
/// where each function is its own COMDAT. It reaches only the `bl` word.
pub fn guard_ret_chain_text(
    g: &GuardRetChain,
    base_off: u32,
    mode: OptMode,
) -> Result<GuardRetChainBody, BackendError> {
    // **`/O1` only.** The reader asks this first, before any body byte is read
    // (board #1638); this is the emitter's own copy, kept for the reason every
    // framed class here keeps its: the two must not be able to disagree
    // silently, and `select_function` is what `function_gate` runs.
    //
    // The clause is this family's — a block reached from several places
    // tail-duplicates above `/O1` on a threshold W10 bracketed and did not fit
    // (board row X-b) — and the common epilogue at the end is exactly that
    // shape. Verified on this class's own cells: at `/Ox` both bodies are
    // longer and the epilogue appears more than once.
    if mode != OptMode::O1 {
        return Err(out_of_class(
            "a guard chain with a materialised common epilogue at /Ox or /O2: \
             the shared block tail-duplicates on a threshold this port has not \
             fitted (board row X-b)",
        ));
    }
    if g.guards.len() != 2 {
        return Err(out_of_class(
            "guard-ret-chain with other than two guards: both witnesses have \
             exactly two and a third arm has never been graded",
        ));
    }

    let frame = frame_for(g);
    let mut t = frame.prologue()?;
    let prolog_len = t.len() as u32;

    // ---- the park, per sub-shape --------------------------------------------
    //
    // Two witnesses, two plans, transcribed rather than unified (module doc).
    // `guard_reg[i]` is where guard `i`'s formal is by the time its compare
    // runs — which the park decides, so the two are resolved out of one place.
    let guard_reg: [u8; 2];
    match g.spine {
        GuardRetSpine::Copy { .. } => {
            t.extend_from_slice(&encode_mr(SWAP_SCRATCH, 3));
            t.extend_from_slice(&encode_mr(3, 4));
            guard_reg = [SWAP_SCRATCH, 3];
        }
        GuardRetSpine::CopyClamp { .. } => {
            t.extend_from_slice(&encode_mr(PARK_REG, 3));
            guard_reg = [3, 4];
        }
    }

    // ---- the guard chain ----------------------------------------------------
    //
    // Each guard is four words and the arm sits INSIDE the chain, in source
    // order. The `b` to the epilogue is the only word whose displacement is not
    // known yet, so its offset is recorded and the word patched once the
    // epilogue's position is known — the same two-step every self-relative
    // forward branch in this crate uses, and not a fixup pass over a block IR.
    let mut arm_branches: Vec<usize> = Vec::with_capacity(2);
    for (i, guard) in g.guards.iter().enumerate() {
        let k = i16::try_from(guard.ret)
            .map_err(|_| out_of_class("guard-ret-chain return literal wider than simm16"))?;
        t.extend_from_slice(&encode_cmplwi(GUARD_CRF, guard_reg[i], 0));
        t.extend_from_slice(&reach::direct(
            Form::Bc { bo: BO_FALSE, bi: cr_bi(GUARD_CRF, CR_BIT_EQ) },
            12,
            "guard-ret-chain guard branch",
        )?);
        t.extend_from_slice(&encode_li(3, k));
        arm_branches.push(t.len());
        t.extend_from_slice(&[0, 0, 0, 0]);
    }

    // ---- the spine ----------------------------------------------------------
    let (dst_reg, src_reg, len, clamp) = match g.spine {
        GuardRetSpine::Copy { len, .. } => (None, Some(SWAP_SCRATCH), len, None),
        GuardRetSpine::CopyClamp { len, lo, hi, .. } => (Some(PARK_REG), None, len, Some((lo, hi))),
    };
    if !(6..=0x7FFF).contains(&len) {
        // Below the step c2 expands the copy inline (25 cells,
        // `work/w-ifn/probe/mcpy.cpp`); above `simm16` the length does not fit
        // the `li`. Both are refusals and neither is a guess.
        return Err(out_of_class(
            "guard-ret-chain copy length outside the measured call window 6..=32767",
        ));
    }
    t.extend_from_slice(&encode_li(5, len as i16));
    if let Some(r) = src_reg {
        t.extend_from_slice(&encode_mr(4, r));
    }
    if let Some(r) = dst_reg {
        t.extend_from_slice(&encode_mr(3, r));
    }
    let bl_offset = base_off + t.len() as u32;
    t.extend_from_slice(&encode_call_branch(bl_offset));

    if let Some((lo, hi)) = clamp {
        if !fits_i16(lo) || !fits_i16(hi) {
            return Err(out_of_class("guard-ret-chain clamp offset wider than simm16"));
        }
        t.extend_from_slice(&encode_lwz(CLAMP_LO_REG, PARK_REG, lo as i16));
        t.extend_from_slice(&encode_lwz(CLAMP_HI_REG, PARK_REG, hi as i16));
        // `cmplw cr6,r10,r11` then `bf 24` — bit 24 is crf6's LT bit, NOT the
        // EQ bit every guard above reads. The store runs when `hi < lo`.
        t.extend_from_slice(&encode_cmplw(GUARD_CRF, CLAMP_HI_REG, CLAMP_LO_REG));
        t.extend_from_slice(&reach::direct(
            Form::Bc { bo: BO_FALSE, bi: cr_bi(GUARD_CRF, CR_BIT_LT) },
            8,
            "guard-ret-chain clamp branch",
        )?);
        t.extend_from_slice(&encode_stw(CLAMP_LO_REG, PARK_REG, hi as i16));
    }

    t.extend_from_slice(&encode_li(3, 0));

    // ---- the materialised common epilogue -----------------------------------
    let epi = t.len();
    t.extend_from_slice(&frame.epilogue()?);

    for site in arm_branches {
        let disp = (epi - site) as i32;
        let w = reach::direct(Form::B, disp, "guard-ret-chain arm branch")?;
        t[site..site + 4].copy_from_slice(&w);
    }

    debug_assert_eq!(t.len() % 4, 0, "a body is a whole number of words");
    Ok(GuardRetChainBody {
        text: t,
        bl_offset,
        prolog_len,
    })
}

/// The `CR_COMPARE` re-export exists so the constant is visibly *used*: this
/// class reads two different bits of one CR field, and naming the field's
/// width beside them is what stops the next reader from assuming `bf 24` and
/// `bf 26` are two fields rather than two bits of one.
const _: u8 = CR_COMPARE;

#[cfg(test)]
mod tests {
    use super::*;
    use c2_il::{GuardRetChain, GuardRetGuard, GuardRetSpine};

    fn get_info() -> GuardRetChain {
        GuardRetChain {
            params: vec![0xc7, 0xc8, 0xc9],
            guards: vec![
                GuardRetGuard { formal: 0, ret: 5 },
                GuardRetGuard { formal: 1, ret: 11 },
            ],
            spine: GuardRetSpine::Copy {
                dst: 1,
                src: 0,
                len: 0x48,
            },
        }
    }

    fn set_info() -> GuardRetChain {
        GuardRetChain {
            params: vec![0xce, 0xcf, 0xd0],
            guards: vec![
                GuardRetGuard { formal: 0, ret: 5 },
                GuardRetGuard { formal: 1, ret: 11 },
            ],
            spine: GuardRetSpine::CopyClamp {
                dst: 0,
                src: 1,
                len: 0x48,
                lo: 0x1c,
                hi: 0x20,
            },
        }
    }

    fn words(t: &[u8]) -> Vec<u32> {
        t.chunks(4)
            .map(|c| u32::from_be_bytes([c[0], c[1], c[2], c[3]]))
            .collect()
    }

    /// **`mmioGetInfo`, word for word, against the real obj.**
    ///
    /// Every word is `work/w-ifn/ref/mmio.dump.txt`'s `.text #5`, which
    /// `cl.exe` 16.00.11886.00 produced under wibo at the workload's own flags.
    /// A test that asserted a length or a shape would pass on a body that
    /// links and is wrong; this asserts the bytes.
    #[test]
    fn sub_shape_g_is_mmiogetinfo_word_for_word() {
        let b = guard_ret_chain_text(&get_info(), 0, OptMode::O1).unwrap();
        assert_eq!(
            words(&b.text),
            vec![
                0x7d88_02a6, 0x9181_fff8, 0x9421_ffa0, // prologue, 96 B, 0 saved
                0x7c6b_1b78, 0x7c83_2378, // the swap park
                0x2b0b_0000, 0x409a_000c, 0x3860_0005, 0x4800_0024, // guard 0
                0x2b03_0000, 0x409a_000c, 0x3860_000b, 0x4800_0014, // guard 1
                0x38a0_0048, 0x7d64_5b78, 0x4bff_ffc5, // li r5,72 · mr r4,r11 · bl
                0x3860_0000, // li r3,0
                0x3821_0060, 0x8181_fff8, 0x7d88_03a6, 0x4e80_0020, // epilogue
            ]
        );
        assert_eq!(b.text.len(), 84, "the reference COMDAT is 84 bytes");
        assert_eq!(b.prolog_len, 12);
        assert_eq!(b.bl_offset, 0x3c);
    }

    /// **`mmioSetInfo`, word for word, against the real obj** — `.text #7`.
    #[test]
    fn sub_shape_s_is_mmiosetinfo_word_for_word() {
        let b = guard_ret_chain_text(&set_info(), 0, OptMode::O1).unwrap();
        assert_eq!(
            words(&b.text),
            vec![
                0x7d88_02a6, 0x9181_fff8, 0xfbe1_fff0, 0x9421_ffa0, // prologue, 1 saved
                0x7c7f_1b78, // the park
                0x2b03_0000, 0x409a_000c, 0x3860_0005, 0x4800_0038, // guard 0
                0x2b04_0000, 0x409a_000c, 0x3860_000b, 0x4800_0028, // guard 1
                0x38a0_0048, 0x7fe3_fb78, 0x4bff_ffc5, // li r5,72 · mr r3,r31 · bl
                0x817f_001c, 0x815f_0020, 0x7f0a_5840, 0x4098_0008, 0x917f_0020, // clamp
                0x3860_0000, // li r3,0
                0x3821_0060, 0x8181_fff8, 0x7d88_03a6, 0xebe1_fff0, 0x4e80_0020, // epilogue
            ]
        );
        assert_eq!(b.text.len(), 108, "the reference COMDAT is 108 bytes");
        assert_eq!(b.prolog_len, 16);
        assert_eq!(b.bl_offset, 0x3c);
    }

    /// **The two `.pdata` words the reference obj carries, derived rather than
    /// carried.** `40001503` and `40001b04` are read out of
    /// `work/w-ifn/ref/mmio.dump.txt`; `coff::pdata::pdata_record` computes
    /// them from the two lengths this emitter returns. This is `w-blockir`'s
    /// mechanism 4 shown paid rather than asserted paid.
    #[test]
    fn the_pdata_words_fall_out_of_the_two_lengths() {
        for (g, want) in [(get_info(), 0x4000_1503u32), (set_info(), 0x4000_1b04)] {
            let b = guard_ret_chain_text(&g, 0, OptMode::O1).unwrap();
            let f = crate::coff::Frame {
                prolog_len: b.prolog_len,
                func_len: b.text.len() as u32,
            };
            let rec = crate::coff::pdata_record(0, &f);
            assert_eq!(u32::from_be_bytes([rec[4], rec[5], rec[6], rec[7]]), want);
        }
    }

    /// **The mode gate, in the emitter as well as in the parser** (board
    /// #1638). Two places, one clause, and the test is what stops them
    /// drifting.
    #[test]
    fn the_emitter_refuses_outside_o1() {
        for g in [get_info(), set_info()] {
            assert!(guard_ret_chain_text(&g, 0, OptMode::Ox).is_err());
        }
    }

    /// **The copy window is a refusal, not a clamp.** Below the measured step
    /// at `n = 6` c2 expands the copy inline, so emitting a call there would be
    /// a complete, plausible, wrong body.
    #[test]
    fn a_copy_below_the_measured_step_is_refused() {
        for n in [0, 1, 2, 3, 4, 5] {
            let mut g = get_info();
            if let GuardRetSpine::Copy { ref mut len, .. } = g.spine {
                *len = n;
            }
            assert!(
                guard_ret_chain_text(&g, 0, OptMode::O1).is_err(),
                "n = {n} expands inline and must be refused"
            );
        }
        let mut g = get_info();
        if let GuardRetSpine::Copy { ref mut len, .. } = g.spine {
            *len = 6;
        }
        assert!(guard_ret_chain_text(&g, 0, OptMode::O1).is_ok());
    }

    /// **`base_off` reaches the `bl` and nothing else.** Every other branch in
    /// this body is self-relative, so a packed layout moves exactly one word.
    #[test]
    fn only_the_bl_moves_with_base_off() {
        let a = guard_ret_chain_text(&get_info(), 0, OptMode::O1).unwrap();
        let b = guard_ret_chain_text(&get_info(), 0x100, OptMode::O1).unwrap();
        let differing: Vec<usize> = a
            .text
            .chunks(4)
            .zip(b.text.chunks(4))
            .enumerate()
            .filter(|(_, (x, y))| x != y)
            .map(|(i, _)| i * 4)
            .collect();
        assert_eq!(differing, vec![0x3c]);
        assert_eq!(b.bl_offset, 0x13c);
    }

    /// **A third guard is refused rather than extrapolated.** Both witnesses
    /// have exactly two; `work/w-ifn/probe/blkorder.cpp` cell `b3` shows what a
    /// third looks like and this class has not been graded on one.
    #[test]
    fn a_third_guard_is_refused() {
        let mut g = get_info();
        g.guards.push(GuardRetGuard { formal: 2, ret: 13 });
        assert!(guard_ret_chain_text(&g, 0, OptMode::O1).is_err());
    }
}
