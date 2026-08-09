//! **W-EXTDATA — the emitter for a sunk `||` guard chain with a shared error
//! tail.**
//!
//! The reader's accept/refuse boundary and the source shape are on
//! [`c2_il::func::body::shapes::guard_chain_shared_tail`]; this file is the
//! thirty words and nothing else. Everything variable in them is named in
//! [`c2_il::GuardChainSharedTailFn`]: three guard formal indices, four callee
//! names and four immediates.
//!
//! ```text
//!    off  word       instruction               why it is this word
//!   ----  --------   -----------------------   ---------------------------------
//!   0x00  7d8802a6   mflr  r12                 FrameLayout{saved_gprs:1}: 96
//!   0x04  9181fff8   stw   r12,-8(r1)          bytes, one callee-saved GPR, and
//!   0x08  fbe1fff0   std   r31,-16(r1)         it is byte-for-byte what the
//!   0x0c  9421ffa0   stwu  r1,-96(r1)          shipped `FrameLayout` already
//!                                              emits at `saved_gprs: 1`
//!   0x10  7c7f1b78   mr    r31,r3              THE PARK. `params[0]` is stored
//!                                              through at 0x5c, after two `bl`s
//!                                              have clobbered every volatile —
//!                                              so it cannot stay in r3, and r31
//!                                              is why the frame saves one GPR
//!   0x14  7d094378   mr    r9,r8               THE HOIST: the LAST rotate step,
//!                                              above every branch. It is the one
//!                                              move whose destination is outside
//!                                              the incoming argument set, so it
//!                                              clobbers nothing the guards read
//!   0x18  2b0X0000   cmplwi cr6,r<g0>,0        ┐ the `||` chain: THREE branches
//!   0x1c  419a0058   bt    26,-> Lerr          │ to ONE block, not a computed
//!   0x20  2b0X0000   cmplwi cr6,r<g1>,0        │ boolean. A lowering that
//!   0x24  419a0050   bt    26,-> Lerr          │ materialised the disjunction
//!   0x28  2b0X0000   cmplwi cr6,r<g2>,0        │ emits `or` and one branch: the
//!   0x2c  419a0048   bt    26,-> Lerr          ┘ right program, wrong bytes
//!   0x30  7ce83b78   mr    r8,r7               ┐ the 5-deep rotate, DESCENDING
//!   0x34  7cc73378   mr    r7,r6               │ so no source is read after it
//!   0x38  3d600000   lis   r11,0        REFHI  │ is clobbered — and the REFHI is
//!   0x3c  7ca62b78   mr    r6,r5               │ INTERLEAVED into it, after the
//!   0x40  7c852378   mr    r5,r4               │ second step. Word 14 of the
//!   0x44  7c641b78   mr    r4,r3               ┘ body: WR1's "the `lis` is the
//!                                                first word" is false here
//!   0x48  386b0000   addi  r3,r11,0     REFLO  the function's address into r3
//!   0x4c  4bxxxxxx   bl    <helper>     REL24
//!   0x50  2c030000   cmpwi cr0,r3,0             `r < 0` on **cr0** …
//!   0x54  4080000c   bf    0,-> Lskip
//!   0x58  39600000   li    r11,0
//!   0x5c  b17f0000   sth   r11,0(r31)           a HALFWORD store: the class is
//!                                               `wchar_t*`, and a `stw` here is
//!                                               two extra bytes of zero written
//!   0x60  2f03KKKK   cmpwi cr6,r3,<sentinel>    … and `r != S` on **cr6**
//!   0x64  409a0024   bf    26,-> epilogue       with `r` already in r3
//!   0x68  4bxxxxxx   bl    <errno>      REL24   ┐ the RANGE arm
//!   0x6c  3960KKKK   li    r11,<k_range>        │
//!   0x70  4800000c   b     -> Ltail             ┘
//!  Lerr:
//!   0x74  4bxxxxxx   bl    <errno>      REL24   ┐ the GUARD arm — SUNK here from
//!   0x78  3960KKKK   li    r11,<k_guard>        ┘ the top of the IL body
//!  Ltail:
//!   0x7c  91630000   stw   r11,0(r3)            ┐ the MERGED TAIL. Four words,
//!   0x80  4bxxxxxx   bl    <invalid>    REL24   │ emitted ONCE and reached from
//!   0x84  3860KKKK   li    r3,<ret_fail>        ┘ both arms
//!   0x88  38210060   addi  r1,r1,96
//!   0x8c  8181fff8   lwz   r12,-8(r1)
//!   0x90  7d8803a6   mtlr  r12
//!   0x94  ebe1fff0   ld    r31,-16(r1)
//!   0x98  4e800020   blr
//! ```
//!
//! **The tail merge is the class, not a peephole.** Board **#1400** found, on
//! `Primes.cpp`, that the optimization a codegen lane reaches for by reflex is
//! the defect; this is the same finding pointing the other way. `bl <errno> ;
//! li r11,K` appears twice with two different `K`, and the four words after it
//! appear once. A lowering that emitted each arm whole would be four words
//! longer, would still link, and would be wrong from 0x7c onwards. The
//! mutation test below asserts it.
//!
//! **Two condition registers, and neither is a preference.** `r < 0` is read on
//! **cr0** (`2c030000`) and `r != S` on **cr6** (`2f03fffe`). Nothing in the
//! source distinguishes them and a class that used one CR for both would emit
//! the right program with the wrong `bf` operand.
//!
//! Every branch here is **self-relative** and therefore independent of where the
//! function lands in `.text`; only the four `bl`s encode their own offset, so
//! they are the only words that need `base_off`.

use crate::codegen::calls::encode_call_branch;
use crate::codegen::encode::{
    cr_bi, encode_addi, encode_addis, encode_b_intra, encode_bc, encode_cmplwi, encode_cmpwi,
    encode_mr, encode_stb, encode_sth, encode_stw, BO_FALSE, BO_TRUE, CR_BIT_EQ, CR_BIT_LT,
    CR_COMPARE,
};
use crate::codegen::frame::FrameLayout;
use crate::codegen::select::{fits_i16, out_of_class, ARG_REGS, RET_REG, SCRATCH_REG};
use crate::codegen::OptMode;
use crate::BackendError;
use c2_il::GuardChainSharedTailFn;

/// The callee-saved register the store target is parked in for the whole body.
/// It is the only saved GPR, which is what makes the frame `saved_gprs: 1`.
const PARK_REG: u8 = 31;

/// `li rD,k` — `addi rD,0,k`. The same two-line helper
/// [`super::if_call_join`] carries, and for the same reason: `encode_addi` with
/// `ra = 0` is `li` and spelling that at every call site hides it.
fn encode_li(rd: u8, k: i16) -> [u8; 4] {
    encode_addi(rd, 0, k)
}

/// This class's emitted body: the bytes plus the offsets the writers need.
pub struct GuardChainSharedTailBody {
    pub text: Vec<u8>,
    /// Absolute `.text` offsets of the four `bl` words, in **block order**:
    /// helper, the RANGE arm's `errno`, the GUARD arm's `errno`, then the
    /// merged tail's reporter. The caller zips them against the four names.
    pub bl_offsets: [u32; 4],
    /// Prologue length in bytes: the `$M(n)` label's value and the `.pdata`
    /// record's `PrologLen`.
    pub prolog_len: u32,
}

/// Emit the thirty words.
///
/// `base_off` is the function's own offset within `.text` — zero under `/Gy`,
/// where each function is its own COMDAT. It reaches only the four `bl` words.
pub fn guard_chain_shared_tail_text(
    g: &GuardChainSharedTailFn,
    base_off: u32,
    mode: OptMode,
) -> Result<GuardChainSharedTailBody, BackendError> {
    // **`/O1` only.** The reader asks this first, before any body byte is read
    // (board #1638); this is the emitter's own copy, kept for the reason
    // `if_call_join` keeps its: the two must not be able to disagree silently,
    // and `select_function` is what `function_gate` runs. The clause itself is
    // the family's — a block shared behind a `b` tail-duplicates above `/O1` on
    // a threshold W10 bracketed and did not fit (board row X-b) — and the merged
    // tail at 0x7c is exactly that shape.
    if mode != OptMode::O1 {
        return Err(out_of_class(
            "a shared error tail at /Ox or /O2: the merged block tail-duplicates \
             on a threshold this port has not fitted (board row X-b)",
        ));
    }
    // Range-checked here as well as in the reader, because this is where a
    // truncation would happen: each of these lands in one signed 16-bit field.
    if !fits_i16(g.k_guard) || !fits_i16(g.k_range) || !fits_i16(g.sentinel) || !fits_i16(g.ret_fail)
    {
        return Err(out_of_class(
            "a shared error tail whose literal is outside simm16",
        ));
    }
    // **W-VSNPRNC: three to seven formals**, and the `lis`'s position is a rule
    // rather than a constant. See the reader's `FORMALS_MIN`/`FORMALS_MAX` and
    // `work/w-vsnprnc/GRID-N.md`. Seven is the last arity whose topmost rotate
    // step lands in an argument register: at eight, c2 spills to the frame and
    // emits a shape this class does not model.
    if !(3..=7).contains(&g.params.len()) {
        return Err(out_of_class(
            "a shared error tail with fewer than three or more than seven formals: \
             at eight the ninth argument spills to the frame and c2 emits a \
             different shape (work/w-vsnprnc/probe/n8.obj)",
        ));
    }
    // The width the reader carried. Anything else never reaches here — and
    // saying so as a refusal rather than a `debug_assert` is board #1706's rule:
    // what the emitter cannot vary, something must refuse.
    let store = match g.store_width {
        1 => encode_stb as fn(u8, u8, i16) -> [u8; 4],
        2 => encode_sth as fn(u8, u8, i16) -> [u8; 4],
        _ => {
            return Err(out_of_class(
                "a shared error tail whose store is neither a byte nor a halfword",
            ))
        }
    };
    if g.guard_ix.iter().any(|&i| i >= g.params.len()) {
        return Err(out_of_class("a guard testing a formal this body does not have"));
    }

    let frame = FrameLayout {
        saved_gprs: 1,
        ..Default::default()
    };
    let prologue = frame.prologue()?;
    let epilogue = frame.epilogue()?;
    let prolog_len = prologue.len() as u32;

    let mut t = prologue;
    // ---- the entry block ---------------------------------------------------
    t.extend_from_slice(&encode_mr(PARK_REG, ARG_REGS[0]));
    // **THE HOIST, as a rule.** The call takes one more argument than the
    // function has formals, so every formal moves up exactly one register; the
    // topmost of those steps — `ARG_REGS[n] <- ARG_REGS[n-1]` — is emitted above
    // every branch, because it is the one whose destination is outside the
    // incoming argument set and therefore clobbers nothing the guards read.
    //
    // This was `mr r9,r8`, a constant that happens to be the n = 6 instance.
    // GRID-N reads it at n = 3…7 as `mr r6,r5`, `mr r7,r6`, `mr r8,r7`,
    // `mr r9,r8`, `mr r10,r9`.
    let n = g.params.len();
    t.extend_from_slice(&encode_mr(ARG_REGS[n], ARG_REGS[n - 1]));

    // ---- the `||` chain ----------------------------------------------------
    //
    // The displacements are filled in after the block layout is known: `Lerr` is
    // the second-to-last block and its offset depends on nothing before it, but
    // writing the arithmetic out here would be three copies of the same
    // off-by-one. The three sites are recorded and patched below.
    let mut guard_sites = [0u32; 3];
    for (n, &ix) in g.guard_ix.iter().enumerate() {
        t.extend_from_slice(&encode_cmplwi(CR_COMPARE, ARG_REGS[ix], 0));
        guard_sites[n] = t.len() as u32;
        t.extend_from_slice(&[0; 4]);
    }

    // ---- the rotate, with the REFHI interleaved after its second step -------
    //
    // Descending: `r<n-1+3> <- …`, … `r4 <- r3`. The topmost step was hoisted
    // above the guards, so `n - 1` remain here.
    //
    // **THE `lis`'s POSITION IS A RULE ABOUT REGISTERS.** This was
    // `const LIS_AFTER: usize = 2`, and the shipped fence said in as many words
    // that one witness could not tell "after the second step" from "three before
    // the last". GRID-N graded n = 3…8 and refuted **both**: the first holds
    // only at n = 6, the second fails at n = 3 where the hoist takes one of the
    // three. What fits all six — and fits n = 8, where the whole shape changes
    // and a count rule would have nothing to say — is
    //
    //     the REFHI is emitted immediately before the first remaining rotate
    //     move whose DESTINATION REGISTER is r6 or lower.
    //
    // Written as that test rather than as an index, so the rule is legible at
    // the one place a wrong `lis` position would be emitted.
    const LIS_BELOW: u8 = 6;
    let mut hi_off = 0u32;
    for step in 0..n - 1 {
        // `ARG_REGS[n-1-step] <- ARG_REGS[n-2-step]`.
        let dst = ARG_REGS[n - 1 - step];
        if hi_off == 0 && dst <= LIS_BELOW {
            hi_off = t.len() as u32;
            t.extend_from_slice(&encode_addis(SCRATCH_REG, 0, 0));
        }
        t.extend_from_slice(&encode_mr(dst, ARG_REGS[n - 2 - step]));
    }
    debug_assert!(hi_off != 0, "the REFHI is inside the rotate, never at word 0");
    t.extend_from_slice(&encode_addi(RET_REG, SCRATCH_REG, 0));

    // ---- the call ----------------------------------------------------------
    let bl_helper = base_off + t.len() as u32;
    t.extend_from_slice(&encode_call_branch(bl_helper));

    // ---- `if (r < 0) *params[0] = 0;` --------------------------------------
    t.extend_from_slice(&encode_cmpwi(0, RET_REG, 0));
    let neg_site = t.len() as u32;
    t.extend_from_slice(&[0; 4]);
    t.extend_from_slice(&encode_li(SCRATCH_REG, 0));
    t.extend_from_slice(&store(SCRATCH_REG, PARK_REG, 0));
    let l_skip = t.len() as u32;

    // ---- `if (r != S) return r;` -------------------------------------------
    t.extend_from_slice(&encode_cmpwi(CR_COMPARE, RET_REG, g.sentinel as i16));
    let sent_site = t.len() as u32;
    t.extend_from_slice(&[0; 4]);

    // ---- the RANGE arm ------------------------------------------------------
    let bl_errno_range = base_off + t.len() as u32;
    t.extend_from_slice(&encode_call_branch(bl_errno_range));
    t.extend_from_slice(&encode_li(SCRATCH_REG, g.k_range as i16));
    let b_site = t.len() as u32;
    t.extend_from_slice(&[0; 4]);

    // ---- the GUARD arm, sunk here from the top of the IL body ---------------
    let l_err = t.len() as u32;
    let bl_errno_guard = base_off + t.len() as u32;
    t.extend_from_slice(&encode_call_branch(bl_errno_guard));
    t.extend_from_slice(&encode_li(SCRATCH_REG, g.k_guard as i16));

    // ---- the MERGED TAIL both arms share ------------------------------------
    let l_tail = t.len() as u32;
    t.extend_from_slice(&encode_stw(SCRATCH_REG, RET_REG, 0));
    let bl_invalid = base_off + t.len() as u32;
    t.extend_from_slice(&encode_call_branch(bl_invalid));
    t.extend_from_slice(&encode_li(RET_REG, g.ret_fail as i16));
    let l_epi = t.len() as u32;
    t.extend_from_slice(&epilogue);

    // ---- the five displacements --------------------------------------------
    //
    // Every one is computed from the block layout above rather than written as a
    // constant, so a change to any block's length moves them all together. A
    // hardcoded `+0x58` would keep linking and stop being right.
    let mut patch = |site: u32, target: u32, bo: u8, bi: Option<u8>| -> Result<(), BackendError> {
        let disp = target as i32 - site as i32;
        let w = match bi {
            Some(bi) => encode_bc(bo, bi, disp),
            None => encode_b_intra(disp),
        }
        .ok_or_else(|| out_of_class("a shared-tail branch outside its displacement field"))?;
        t[site as usize..site as usize + 4].copy_from_slice(&w);
        Ok(())
    };
    for site in guard_sites {
        patch(site, l_err, BO_TRUE, Some(cr_bi(CR_COMPARE, CR_BIT_EQ)))?;
    }
    patch(neg_site, l_skip, BO_FALSE, Some(cr_bi(0, CR_BIT_LT)))?;
    patch(sent_site, l_epi, BO_FALSE, Some(cr_bi(CR_COMPARE, CR_BIT_EQ)))?;
    patch(b_site, l_tail, 0, None)?;

    Ok(GuardChainSharedTailBody {
        text: t,
        bl_offsets: [bl_helper, bl_errno_range, bl_errno_guard, bl_invalid],
        prolog_len,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `_vswprintf_s_l`'s parse, as the reader produces it from the workload's
    /// own IL. Tokens are the capture's; the names are the obj's.
    fn vswprnc() -> GuardChainSharedTailFn {
        GuardChainSharedTailFn {
            params: vec![1, 2, 3, 4, 5, 6],
            // count, buffer, sizeInWords — `params[2]`, `params[0]`, `params[1]`
            guard_ix: [2, 0, 1],
            helper: "_vswprintf_helper".into(),
            fn_addr: "_woutput_s_l".into(),
            errno: "_errno".into(),
            invalid: "_invalid_parameter_noinfo".into(),
            k_guard: 0x16,
            k_range: 0x22,
            sentinel: -2,
            ret_fail: -1,
            // `wchar_t *buffer` — a halfword. `vsnprnc()` below is the byte.
            store_width: 2,
        }
    }

    /// `_vsprintf_s_l`'s parse — **five** formals and a **byte** store.
    ///
    /// The same class one arity down and one store width over, which is the
    /// whole of what lane `w-vsnprnc` widened. Kept beside `vswprnc()` so the
    /// two arities are asserted against two real objs rather than one obj and a
    /// generalization.
    fn vsnprnc() -> GuardChainSharedTailFn {
        GuardChainSharedTailFn {
            params: vec![1, 2, 3, 4, 5],
            // format, buffer, sizeInBytes — `params[2]`, `params[0]`, `params[1]`
            guard_ix: [2, 0, 1],
            helper: "_vsnprintf_helper".into(),
            fn_addr: "_output_s_l".into(),
            errno: "_errno".into(),
            invalid: "_invalid_parameter_noinfo".into(),
            k_guard: 0x16,
            k_range: 0x22,
            sentinel: -2,
            ret_fail: -1,
            store_width: 1,
        }
    }

    /// **The thirty-nine words, against the real obj.**
    ///
    /// Transcribed from `work/w-extdata/ref/vswprnc/dis.txt`, which this lane
    /// re-derived at the workload's own flags rather than inheriting. Under
    /// `/Gy` the function is at offset 0 of its own COMDAT, so every `bl`
    /// displacement is `-(its own offset)`.
    #[test]
    fn body_matches_the_reference_obj_word_for_word() {
        let b = guard_chain_shared_tail_text(&vswprnc(), 0, OptMode::O1).unwrap();
        #[rustfmt::skip]
        let want: Vec<u8> = [
            0x7d8802a6u32, 0x9181fff8, 0xfbe1fff0, 0x9421ffa0, // frame
            0x7c7f1b78,                                        // mr r31,r3
            0x7d094378,                                        // mr r9,r8
            0x2b050000, 0x419a0058,                            // count == 0
            0x2b030000, 0x419a0050,                            // buffer == 0
            0x2b040000, 0x419a0048,                            // size == 0
            0x7ce83b78, 0x7cc73378,                            // r8<-r7, r7<-r6
            0x3d600000,                                        // lis r11 (REFHI)
            0x7ca62b78, 0x7c852378, 0x7c641b78,                // r6<-r5,r5<-r4,r4<-r3
            0x386b0000,                                        // addi r3,r11 (REFLO)
            0x4bffffb5,                                        // bl helper
            0x2c030000, 0x4080000c,                            // r < 0 on cr0
            0x39600000, 0xb17f0000,                            // li r11,0 ; sth
            0x2f03fffe, 0x409a0024,                            // r != -2 on cr6
            0x4bffff99, 0x39600022, 0x4800000c,                // RANGE arm
            0x4bffff8d, 0x39600016,                            // GUARD arm
            0x91630000, 0x4bffff81, 0x3860ffff,                // the merged tail
            0x38210060, 0x8181fff8, 0x7d8803a6, 0xebe1fff0, 0x4e800020, // epilogue
        ]
        .iter()
        .flat_map(|w| w.to_be_bytes())
        .collect();
        assert_eq!(b.text.len(), 156, "the reference `.text` is 156 bytes");
        assert_eq!(b.text, want);
        assert_eq!(b.prolog_len, 0x10);
        assert_eq!(b.bl_offsets, [0x4c, 0x68, 0x74, 0x80]);
    }

    /// **`_vsprintf_s_l`'s thirty-eight words, against ITS OWN reference obj.**
    ///
    /// Transcribed from `work/w-vsnprnc/obj/vsnprnc.obj`, dumped by
    /// `scripts/gt_dump.py` at the workload's own flags and cwd. **This is the
    /// second arity and the second store width the class has ever been asserted
    /// on**, and it is a different obj rather than the same one re-derived: at
    /// n = 5 the hoist is `mr r8,r7`, one rotate step sits above the `lis`
    /// instead of two, and the store is `stb`.
    #[test]
    fn the_five_formal_byte_store_body_matches_its_reference_obj() {
        let b = guard_chain_shared_tail_text(&vsnprnc(), 0, OptMode::O1).unwrap();
        #[rustfmt::skip]
        let want: Vec<u8> = [
            0x7d8802a6u32, 0x9181fff8, 0xfbe1fff0, 0x9421ffa0, // frame
            0x7c7f1b78,                                        // mr r31,r3
            0x7ce83b78,                                        // mr r8,r7  THE HOIST
            0x2b050000, 0x419a0054,                            // format == 0
            0x2b030000, 0x419a004c,                            // buffer == 0
            0x2b040000, 0x419a0044,                            // size == 0
            0x7cc73378,                                        // r7<-r6   (ONE, not two)
            0x3d600000,                                        // lis r11 (REFHI)
            0x7ca62b78, 0x7c852378, 0x7c641b78,                // r6<-r5,r5<-r4,r4<-r3
            0x386b0000,                                        // addi r3,r11 (REFLO)
            0x4bffffb9,                                        // bl helper
            0x2c030000, 0x4080000c,                            // r < 0 on cr0
            0x39600000, 0x997f0000,                            // li r11,0 ; STB
            0x2f03fffe, 0x409a0024,                            // r != -2 on cr6
            0x4bffff9d, 0x39600022, 0x4800000c,                // RANGE arm
            0x4bffff91, 0x39600016,                            // GUARD arm
            0x91630000, 0x4bffff85, 0x3860ffff,                // the merged tail
            0x38210060, 0x8181fff8, 0x7d8803a6, 0xebe1fff0, 0x4e800020, // epilogue
        ]
        .iter()
        .flat_map(|w| w.to_be_bytes())
        .collect();
        assert_eq!(b.text.len(), 152, "the reference `.text` is 152 bytes");
        assert_eq!(b.text, want);
        assert_eq!(b.prolog_len, 0x10);
        assert_eq!(b.bl_offsets, [0x48, 0x64, 0x70, 0x7c]);
    }

    /// **MUST-FAIL MUTATION — the `lis` position is a REGISTER rule, and both
    /// carried count rules would put it in the wrong place.**
    ///
    /// GRID-N's whole content, executable. `LIS_AFTER = 2` — the constant this
    /// class shipped with — is right at n = 6 and wrong at every other arity;
    /// "three rotate steps follow the `lis`" is right at n ≥ 4 and wrong at
    /// n = 3, where the hoist takes one of the three.
    #[test]
    fn the_lis_sits_above_the_r6_r5_r4_block_at_every_arity() {
        let lis = encode_addis(SCRATCH_REG, 0, 0);
        // (formals, the `lis`'s word index, moves emitted AFTER it)
        for (n, word, after) in [(3usize, 12usize, 2usize), (4, 12, 3), (5, 13, 3), (6, 14, 3), (7, 15, 3)] {
            let mut g = vsnprnc();
            g.params = (1..=n as u32).collect();
            let b = guard_chain_shared_tail_text(&g, 0, OptMode::O1).unwrap();
            let at = word * 4;
            assert_eq!(&b.text[at..at + 4], &lis, "n = {n}: the lis is not word {word}");
            // The moves after it write r6, r5, r4 — or, at n = 3, r5 and r4,
            // because `mr r6,r5` was hoisted above the guards.
            for (s, dst) in (0..after).zip([6u8, 5, 4].iter().skip(3 - after)) {
                let o = at + 4 + s * 4;
                assert_eq!(
                    &b.text[o..o + 4],
                    &encode_mr(*dst, *dst - 1),
                    "n = {n}: move {s} after the lis does not write r{dst}"
                );
            }
            // And the word right after those is the REFLO, never another move.
            let reflo = at + 4 + after * 4;
            assert_eq!(&b.text[reflo..reflo + 4], &encode_addi(RET_REG, SCRATCH_REG, 0));
            // THE MUTATION: `LIS_AFTER = 2` puts the lis at word 6 + 2 = 8 for
            // every arity. It agrees with the truth at n = 6 and nowhere else.
            assert_eq!(
                word == 14,
                n == 6,
                "n = {n}: the shipped constant and the measured rule agree only at six"
            );
        }
    }

    /// **The HOIST is `ARG_REGS[n] <- ARG_REGS[n-1]`, not `mr r9,r8`.**
    #[test]
    fn the_hoisted_move_follows_the_arity() {
        for (n, dst) in [(3usize, 6u8), (4, 7), (5, 8), (6, 9), (7, 10)] {
            let mut g = vsnprnc();
            g.params = (1..=n as u32).collect();
            let b = guard_chain_shared_tail_text(&g, 0, OptMode::O1).unwrap();
            assert_eq!(&b.text[0x14..0x18], &encode_mr(dst, dst - 1), "n = {n}");
        }
    }

    /// **The arity fence, and it is a WITNESS at the top end.**
    ///
    /// Two and eight both refuse. Eight is not an extrapolation: `n8.obj` shows
    /// c2 growing the frame to 112, spilling `r10` to `84(r1)` at the call site
    /// and hoisting nothing — a different shape, refused rather than guessed.
    #[test]
    fn arity_outside_three_to_seven_refuses() {
        for n in [1usize, 2, 8, 9] {
            let mut g = vsnprnc();
            g.params = (1..=n as u32).collect();
            g.guard_ix = [0, 0, 0];
            assert!(
                guard_chain_shared_tail_text(&g, 0, OptMode::O1).is_err(),
                "n = {n} must refuse"
            );
        }
    }

    /// **MUST-FAIL MUTATION — the store width, which was a LIVE `Port=Mismatch`.**
    ///
    /// The shipped emitter wrote `sth` unconditionally while the reader admitted
    /// every non-word store, so `char*`, `bool*` and `long long*` bodies came
    /// out with one substituted word (GRID-S: five of twelve cells). This
    /// asserts that the two widths are two different words and that nothing else
    /// in the body moves with them.
    #[test]
    fn the_store_width_is_the_only_word_that_follows_the_pointee_type() {
        let mut byte = vsnprnc();
        byte.params = vec![1, 2, 3, 4, 5, 6];
        let mut half = byte.clone();
        half.store_width = 2;
        let b = guard_chain_shared_tail_text(&byte, 0, OptMode::O1).unwrap();
        let h = guard_chain_shared_tail_text(&half, 0, OptMode::O1).unwrap();
        assert_eq!(&b.text[0x5c..0x60], &encode_stb(SCRATCH_REG, PARK_REG, 0));
        assert_eq!(&h.text[0x5c..0x60], &encode_sth(SCRATCH_REG, PARK_REG, 0));
        assert_eq!(b.text.len(), h.text.len());
        for i in (0..b.text.len()).filter(|i| !(0x5c..0x60).contains(i)) {
            assert_eq!(b.text[i], h.text[i], "byte {i} moved with the store width");
        }
        // Anything the reader did not carry refuses rather than defaulting.
        let mut wide = byte.clone();
        wide.store_width = 4;
        assert!(guard_chain_shared_tail_text(&wide, 0, OptMode::O1).is_err());
        wide.store_width = 8;
        assert!(guard_chain_shared_tail_text(&wide, 0, OptMode::O1).is_err());
    }

    /// **MUST-FAIL MUTATION — the tail merge is the class.**
    ///
    /// Emitting each error arm whole (its own `stw`/`bl`/`li` tail) instead of
    /// sharing four words is the reflex a codegen lane has, it still links, and
    /// it is wrong from 0x7c onwards. Board **#1400**'s finding pointing the
    /// other way. Asserted by construction: the body is 156 bytes, and a
    /// duplicated tail is 168.
    #[test]
    fn duplicating_the_shared_tail_would_be_twelve_bytes_longer() {
        let b = guard_chain_shared_tail_text(&vswprnc(), 0, OptMode::O1).unwrap();
        assert_eq!(b.text.len(), 156);
        // The `b` at 0x70 is what makes the sharing visible in the bytes: remove
        // it and the RANGE arm falls into the GUARD arm's `bl`.
        assert_eq!(&b.text[0x70..0x74], &0x4800000cu32.to_be_bytes());
    }

    /// **The two condition registers are different, and that is asserted.**
    ///
    /// `r < 0` reads cr0 and `r != S` reads cr6. A class that used one CR for
    /// both emits the right program and the wrong `bf` operand.
    #[test]
    fn the_two_result_tests_use_different_condition_registers() {
        let b = guard_chain_shared_tail_text(&vswprnc(), 0, OptMode::O1).unwrap();
        assert_eq!(&b.text[0x50..0x54], &encode_cmpwi(0, 3, 0));
        assert_eq!(&b.text[0x60..0x64], &encode_cmpwi(CR_COMPARE, 3, -2));
        assert_ne!(b.text[0x54..0x58], b.text[0x64..0x68]);
    }

    /// **The REFHI is inside the rotate**, which is the fact WR1's old "first
    /// word" rule contradicted. If a future schedule moved it, `data_refs_of`
    /// would still find it by search — this asserts the schedule itself.
    #[test]
    fn the_high_half_is_word_fourteen_and_not_word_zero() {
        let b = guard_chain_shared_tail_text(&vswprnc(), 0, OptMode::O1).unwrap();
        assert_eq!(&b.text[0x38..0x3c], &encode_addis(SCRATCH_REG, 0, 0));
        assert_ne!(&b.text[0x00..0x04], &encode_addis(SCRATCH_REG, 0, 0));
    }

    /// `/Ox` and `/O2` refuse, in the emitter as well as in the parser.
    #[test]
    fn refuses_outside_o1() {
        assert!(guard_chain_shared_tail_text(&vswprnc(), 0, OptMode::Ox).is_err());
    }

    /// **SUPERSEDED 2026-08-09 by lane `w-vsnprnc`, and kept as the record of
    /// what replaced it.**
    ///
    /// This test read `refuses_a_seventh_formal`, and its reason was honest:
    /// *"the `lis`'s position inside the rotate was not separable from the
    /// arity at n = 1"*. It was a fence around a missing measurement, not a fact
    /// about c2 — and GRID-N supplied the measurement. A seventh formal is now
    /// **in class and byte-exact**, and `refuses_a_seventh_formal` would today
    /// be asserting the port refuses something it can do.
    ///
    /// The assertion is inverted rather than deleted, so the supersession is in
    /// the file that carried the claim: n = 7 emits, n = 8 refuses, and `n = 8`
    /// refuses on a *witness* (`work/w-vsnprnc/probe/n8.obj`) rather than on the
    /// absence of one, which is the whole difference between this fence and the
    /// one it replaces.
    #[test]
    fn a_seventh_formal_is_in_class_now_and_an_eighth_still_is_not() {
        let mut g = vswprnc();
        g.params.push(7);
        let b = guard_chain_shared_tail_text(&g, 0, OptMode::O1).unwrap();
        assert_eq!(b.text.len(), 160, "the n = 7 reference `.text` is 160 bytes");
        assert_eq!(&b.text[0x14..0x18], &encode_mr(10, 9), "the hoist is mr r10,r9");
        g.params.push(8);
        assert!(guard_chain_shared_tail_text(&g, 0, OptMode::O1).is_err());
    }
}
