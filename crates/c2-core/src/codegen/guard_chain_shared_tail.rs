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
    encode_mr, encode_sth, encode_stw, BO_FALSE, BO_TRUE, CR_BIT_EQ, CR_BIT_LT, CR_COMPARE,
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
    // Six formals: see the reader's fence. The rotate's length and the `lis`'s
    // position inside it are both functions of this number and neither was
    // separable at n = 1, so the arity is required rather than generalized.
    if g.params.len() != 6 {
        return Err(out_of_class("a shared error tail without six formals"));
    }
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
    // The hoist. `ARG_REGS[6]` is r9 and `ARG_REGS[5]` is r8: the call takes
    // seven arguments where the function has six formals, so every formal moves
    // up exactly one register and this is the topmost step.
    t.extend_from_slice(&encode_mr(ARG_REGS[6], ARG_REGS[5]));

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
    // Descending: `r8 <- r7`, `r7 <- r6`, … `r4 <- r3`. Written as a loop over
    // the formals rather than six literal `mr`s so the register numbers come
    // from `ARG_REGS` exactly as every other class reads the ABI.
    const LIS_AFTER: usize = 2;
    let mut hi_off = 0u32;
    for step in 0..5usize {
        if step == LIS_AFTER {
            hi_off = t.len() as u32;
            t.extend_from_slice(&encode_addis(SCRATCH_REG, 0, 0));
        }
        // `ARG_REGS[5 - step] <- ARG_REGS[4 - step]`: r8<-r7, r7<-r6, r6<-r5,
        // r5<-r4, r4<-r3.
        t.extend_from_slice(&encode_mr(ARG_REGS[5 - step], ARG_REGS[4 - step]));
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
    t.extend_from_slice(&encode_sth(SCRATCH_REG, PARK_REG, 0));
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

    /// A seventh formal refuses rather than rotating one step further: the
    /// `lis`'s position inside the rotate was not separable from the arity at
    /// n = 1.
    #[test]
    fn refuses_a_seventh_formal() {
        let mut g = vswprnc();
        g.params.push(7);
        assert!(guard_chain_shared_tail_text(&g, 0, OptMode::O1).is_err());
    }
}
