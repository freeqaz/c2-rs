//! **W8 — the two-arm conditional tail call**: the port's first branch.
//!
//! `docs/CFG_SHAPE.md` §4 specifies one function byte for byte and enumerates
//! ten decisions its emitter makes. This file is those ten decisions, and the
//! §4.1 byte string is the test.
//!
//! ```text
//!   ?MemFree@NUISPEECH@@YAXPAX0K@Z          .text COMDAT, 0x24 B, nrel 2
//!     0000  7c 8b 23 78   mr      r11,r4        entry: v2 parked, both arms need it
//!     0004  2b 03 00 00   cmplwi  cr6,r3,0      v1 == 0, UNSIGNED (pointer operand)
//!     0008  40 9a 00 10   bne     cr6,+16       -> 0x18, the else entry
//!     000c  7c a4 2b 78   mr      r4,r5         then: arg2 = ul
//!     0010  7d 63 5b 78   mr      r3,r11        then: arg1 = v2
//!     0014  4b ff ff ec   b       XMemFree      REL24 @0x14
//!     0018  7d 65 5b 78   mr      r5,r11        else: arg3 = v2
//!     001c  38 80 00 00   li      r4,0          else: arg2 = 0
//!     0020  4b ff ff e0   b       RtlFreeHeap   REL24 @0x20
//! ```
//!
//! ## The two things this file gets right that a naive emitter does not
//!
//! **1. There are two `b` encodings and they are the same opcode** (§3.3, board
//! #191). The `bc` at 0x08 carries its **true self-relative displacement** and
//! takes **no relocation**. The two `b`s at 0x14 and 0x20 carry a
//! **section-start-relative placeholder** (`−k`, where `k` is the branch's own
//! `.text` offset) and each takes a `REL24`. A fixup pass that treats every `b`
//! alike corrupts one of the two. Here they are simply different functions:
//! [`encode_bc`] and [`encode_tail_branch`].
//!
//! **2. The epilogue is never materialized** (§4.2 item 7). Both `3A → epilogue`
//! in the IL become tail calls, so no edge reaches the epilogue and no block is
//! emitted for it. That is *not* the same as §3.6's rule that an **unreachable**
//! epilogue block *is* emitted — `?b_if`, `?b_and` and `?b_or` each end in a
//! dead `4e800020` that no path reaches. The difference is whether the epilogue
//! is a label some `3A` still names as a *block* or whether every path left the
//! function before it. In this class every path leaves. A shape where one arm
//! falls through to the epilogue is fold band 2 (a `bclr`) and is out of class.
//!
//! ## What the branch displacement does NOT need
//!
//! A fixup list. `docs/CFG_SHAPE.md` §6.2 item B says one is required "even for
//! §4's single-branch minimal instance", because `38`/`3A` carry no direction so
//! the target's offset is unknown when the branch is emitted. That is true of a
//! *general* lowering and it is **not** true here: the class has exactly three
//! blocks in a fixed order, so the `bc`'s displacement is `4 * (then_steps + 2)`
//! — a function of a length this emitter already has. The fixup pass arrives
//! with the first shape that has a *variable* block order; building it now would
//! be a mechanism with no fact behind it. The range check §6.2 item D asks for is
//! still here, inside [`encode_bc`], because a truncated `BD` is a
//! legal-looking branch to the wrong place.

use c2_il::{CondPlan, CondStep, CondTailPair, IlFunction, Rel};

use crate::BackendError;
use crate::codegen::encode::{
    cr_bi, encode_bc, encode_cmplwi, encode_cmpwi, encode_mr, BO_FALSE, BO_TRUE, CR_BIT_EQ,
    CR_BIT_GT, CR_BIT_LT, CR_COMPARE,
};
use crate::codegen::encode::{encode_addi, encode_rlwinm};
use crate::codegen::select::out_of_class;

/// The bytes of a [`CondTailPair`] body, minus the two tail branches, plus where
/// they go.
///
/// The two `b` words are the caller's because each encodes **its own `.text`
/// offset** — the same reason [`crate::codegen::Selected::Tail`] hands back an
/// unfinished text. Everything else, including the `bc`'s displacement, is
/// offset-independent and is finished here.
pub struct CondPairParts {
    /// The whole body with a **zero word** at each tail-branch site.
    pub text: Vec<u8>,
    /// Offsets, within `text`, of the then-arm's `b` and then the else-arm's.
    /// Block order, so the lower offset is the then-arm's — which is also the
    /// order [`c2_il::IlFunction::callees`] yields the two callees in.
    pub branch_offsets: [u32; 2],
}

/// **The emitted branch condition is the NEGATION of the IL relation**, because
/// the IL's `38` is brFALSE and the branch it becomes is the edge to the *else*
/// block (`docs/CFG_SHAPE.md` §1 prediction A3, RIGHT across ten cells; §4.2
/// item 2).
///
/// Returns `(BO, bit)`; the caller adds the CR field.
pub(crate) fn branch_sense(rel: Rel) -> (u8, u8) {
    match rel {
        // `==` -> branch when EQ is CLEAR, i.e. `bne`.
        Rel::Eq => (BO_FALSE, CR_BIT_EQ),
        Rel::Ne => (BO_TRUE, CR_BIT_EQ),
        // `<` -> `bge`: LT clear.
        Rel::Lt => (BO_FALSE, CR_BIT_LT),
        Rel::Ge => (BO_TRUE, CR_BIT_LT),
        // `>` -> `ble`: GT clear.
        Rel::Gt => (BO_FALSE, CR_BIT_GT),
        Rel::Le => (BO_TRUE, CR_BIT_GT),
    }
}

fn emit_steps(steps: &[CondStep], out: &mut Vec<u8>) -> Result<(), BackendError> {
    for s in steps {
        match *s {
            CondStep::Move { dst, src } => out.extend_from_slice(&encode_mr(dst, src)),
            CondStep::Li { dst, k } => {
                let k = i16::try_from(k).map_err(|_| {
                    out_of_class("a conditional arm's literal argument is wider than `li`")
                })?;
                out.extend_from_slice(&encode_addi(dst, 0, k));
            }
            // **W42** — `(formal >> k) & m`, folded to one `rlwinm` at parse
            // time by `c2_il::shift_mask_rlwinm` (70 graded cells; see its doc).
            // `dst == src` is `plan_cond_pair`'s rule 1b and is re-asserted here
            // so the two files cannot drift about which form is in class.
            CondStep::Rlwinm { dst, src, sh, mb, me } => {
                if dst != src {
                    return Err(out_of_class(
                        "an out-of-place shift-and-mask in a conditional arm: c2 \
                         homes the source into a scratch first and which scratch \
                         is uncharacterized; out of class",
                    ));
                }
                out.extend_from_slice(&encode_rlwinm(dst, src, sh, mb, me));
            }
        }
    }
    Ok(())
}

/// Lower a [`CondTailPair`]. The register schedule comes from
/// [`c2_il::plan_cond_pair`] — **the same function the IL parser gated on**, not
/// a copy of it, so the census and the emission gate cannot disagree about what
/// is in class (`docs/GAPS.md` §6 instance #9).
pub fn cond_pair_parts(
    func: &IlFunction,
    pair: &CondTailPair,
) -> Result<CondPairParts, BackendError> {
    let plan: CondPlan = c2_il::plan_cond_pair(
        func.params.len(),
        pair.cmp_param,
        &pair.then_arm.slots,
        &pair.else_arm.slots,
    )
    .ok_or_else(|| {
        out_of_class(
            "a two-arm conditional tail call whose values cannot be scheduled by \
             the measured entry-block rules: refused rather than emitted, because \
             the placement rules are fitted (docs/CFG_SHAPE.md §8.1 B3) and a \
             wrong schedule is a plausible-looking wrong branch",
        )
    })?;

    let mut text: Vec<u8> = Vec::with_capacity(0x30);
    // ---- the entry block: shuffles, then the compare --------------------
    //
    // The shuffles come FIRST, and the compare reads the value's *post-hoist*
    // location. `?mmioGetInfo` is the separating cell: `mr r11,r3 ; cmplwi
    // cr6,r11,0` — it compares r11, because r3 is about to be overwritten by the
    // other hoist. `?MemFree` is the same rule with an untouched r3.
    emit_steps(&plan.entry, &mut text)?;
    text.extend_from_slice(&if pair.signed {
        let k = i16::try_from(pair.k).map_err(|_| {
            out_of_class("a signed comparison literal wider than `cmpwi`'s immediate")
        })?;
        encode_cmpwi(CR_COMPARE, plan.cmp_reg, k)
    } else {
        let k = u16::try_from(pair.k).map_err(|_| {
            out_of_class("an unsigned comparison literal wider than `cmplwi`'s immediate")
        })?;
        encode_cmplwi(CR_COMPARE, plan.cmp_reg, k)
    });

    // ---- the branch ------------------------------------------------------
    //
    // The then-block is the FALL-THROUGH and the `bc` is the edge to the else
    // block (§3.4, ten cells consistent). Its displacement is its own width plus
    // the then-block's: the branch, the then-block's steps, and the then-block's
    // tail `b`.
    let then_words = plan.then_steps.len() as i32 + 1;
    let disp = 4 * (then_words + 1);
    let (bo, bit) = branch_sense(pair.rel);
    let bc = encode_bc(bo, cr_bi(CR_COMPARE, bit), disp).ok_or_else(|| {
        out_of_class(
            "a conditional branch past the 14-bit BD field: the expansion \
             (invert, branch over an unconditional `b`) is measured but not built",
        )
    })?;
    text.extend_from_slice(&bc);

    // ---- the then block, the else block, and the two tail branches -------
    emit_steps(&plan.then_steps, &mut text)?;
    let then_branch = text.len() as u32;
    text.extend_from_slice(&[0; 4]);
    emit_steps(&plan.else_steps, &mut text)?;
    let else_branch = text.len() as u32;
    text.extend_from_slice(&[0; 4]);

    Ok(CondPairParts {
        text,
        branch_offsets: [then_branch, else_branch],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use c2_il::{CondArm, SlotArg};

    fn memfree() -> IlFunction {
        // v1, v2, ul — three formals, tokens as captured.
        let mut f = crate::codegen::testutil::func_with(vec![0x0F6C, 0x0F6D, 0x0F6E], Vec::new());
        f.mangled_name = "?MemFree@NUISPEECH@@YAXPAX0K@Z".to_string();
        f.cond_pair = Some(CondTailPair {
            cmp_param: 0,
            rel: Rel::Eq,
            signed: false,
            k: 0,
            then_arm: CondArm {
                callee: "XMemFree".to_string(),
                slots: vec![SlotArg::Formal(1), SlotArg::Formal(2)],
            },
            else_arm: CondArm {
                callee: "RtlFreeHeap".to_string(),
                slots: vec![SlotArg::Formal(0), SlotArg::Lit(0), SlotArg::Formal(1)],
            },
        });
        f
    }

    /// **The known-answer control.** `docs/CFG_SHAPE.md` §4.1 published these
    /// thirty-six bytes — read off the real obj by lane w-cfg — before this
    /// emitter existed. The two zero words are the tail branches, which the
    /// caller fills because they encode their own `.text` offset.
    #[test]
    fn memfree_matches_the_published_bytes() {
        let f = memfree();
        let parts = cond_pair_parts(&f, f.cond_pair.as_ref().unwrap()).expect("in class");
        #[rustfmt::skip]
        let want: Vec<u8> = vec![
            0x7c, 0x8b, 0x23, 0x78, // mr     r11,r4
            0x2b, 0x03, 0x00, 0x00, // cmplwi cr6,r3,0
            0x40, 0x9a, 0x00, 0x10, // bne    cr6,+16
            0x7c, 0xa4, 0x2b, 0x78, // mr     r4,r5
            0x7d, 0x63, 0x5b, 0x78, // mr     r3,r11
            0x00, 0x00, 0x00, 0x00, // b      XMemFree      <- the caller's
            0x7d, 0x65, 0x5b, 0x78, // mr     r5,r11
            0x38, 0x80, 0x00, 0x00, // li     r4,0
            0x00, 0x00, 0x00, 0x00, // b      RtlFreeHeap   <- the caller's
        ];
        assert_eq!(parts.text, want);
        assert_eq!(parts.text.len(), 0x24);
        assert_eq!(parts.branch_offsets, [0x14, 0x20]);
    }

    /// `?MemAlloc`'s entry block hoists a **second** shuffle, because both arms
    /// want `attrs` in r4 — the then-arm passes it, the else-arm masks it in
    /// place. `?MemFree` hoists only one. §4.2 item 9's discriminator, as far as
    /// this class can express it.
    #[test]
    fn a_value_both_arms_want_in_the_same_register_is_hoisted() {
        let mut f = memfree();
        // Model `?MemAlloc`'s slot lists, minus the else-arm's `rlwinm` (which
        // is out of class — see the rung). then: g(p1,p2); else: h(p0,p2,p1).
        let pair = CondTailPair {
            cmp_param: 0,
            rel: Rel::Eq,
            signed: false,
            k: 0,
            then_arm: CondArm {
                callee: "XMemAlloc".to_string(),
                slots: vec![SlotArg::Formal(1), SlotArg::Formal(2)],
            },
            else_arm: CondArm {
                callee: "RtlAllocateHeap".to_string(),
                slots: vec![SlotArg::Formal(0), SlotArg::Formal(2), SlotArg::Formal(1)],
            },
        };
        f.cond_pair = Some(pair);
        let parts = cond_pair_parts(&f, f.cond_pair.as_ref().unwrap()).unwrap();
        // entry: `mr r11,r4` then `mr r4,r5` — descending destination.
        assert_eq!(&parts.text[..8], &[0x7c, 0x8b, 0x23, 0x78, 0x7c, 0xa4, 0x2b, 0x78]);
        // then: only `mr r3,r11` remains.
        assert_eq!(&parts.text[16..20], &[0x7d, 0x63, 0x5b, 0x78]);
    }

    /// `w9_rel_signed.cpp` / `w9_rel_unsigned.cpp` — the relation grid, against
    /// bytes read off the **real obj**.
    ///
    /// The test below this one (`the_branch_sense_negates_every_relation`)
    /// compares the port's table to *itself*, so it cannot fail for the right
    /// reason. Until lane w-frame added the W9 fixtures, that self-comparison
    /// plus `signedness_selects_the_compare_instruction` were the ONLY things
    /// holding five of `branch_sense`'s six rows and the whole `cmpwi` path:
    /// every W8 fixture tests `v1 == 0` on a **pointer**, which is `Rel::Eq`
    /// (→ `BO_FALSE` only) on an unsigned operand (→ `cmplwi` only) against the
    /// literal 0. `docs/STATUS.md` trap 5 — absence reads as success.
    ///
    /// These twelve words are transcribed from the obj `cl.exe` 16.00.11886.00
    /// emits for the two fixtures, one COMDAT of six 24-byte bodies each. They
    /// are the port's first oracle witness for `bt` (`BO=12`), for the `LT`/`GT`
    /// CR bits, for `cmpwi`, and for a **non-zero** comparison immediate.
    #[test]
    fn the_relation_grid_matches_the_real_obj_bytes() {
        // `void s_xx(void *v1, unsigned long ul, int a)`:
        //     if (a <rel> k) { g2(v1, ul); return; }
        //     h3(v1, 0, 0);
        // The scrutinee is the THIRD formal, so neither arm wants its register
        // and no entry-block park is involved — the fixture is w8's shape with
        // only the relation and the operand type moved.
        fn grid(rel: Rel, signed: bool, k: i32) -> Vec<u8> {
            let mut f =
                crate::codegen::testutil::func_with(vec![0x0F6C, 0x0F6D, 0x0F6E], Vec::new());
            f.mangled_name = "?s_eq@@YAXPAXKH@Z".to_string();
            f.cond_pair = Some(CondTailPair {
                cmp_param: 2,
                rel,
                signed,
                k,
                then_arm: CondArm {
                    callee: "?g2@@YAXPAXK@Z".to_string(),
                    slots: vec![SlotArg::Formal(0), SlotArg::Formal(1)],
                },
                else_arm: CondArm {
                    callee: "?h3@@YAXPAXK0@Z".to_string(),
                    slots: vec![SlotArg::Formal(0), SlotArg::Lit(0), SlotArg::Lit(0)],
                },
            });
            let parts = cond_pair_parts(&f, f.cond_pair.as_ref().unwrap()).expect("in class");
            // The layout is fixed for the whole grid: compare, branch, the
            // then-arm's bare tail call, the else-arm's two literals (in
            // DESCENDING destination order, r5 before r4), its tail call.
            assert_eq!(parts.text.len(), 0x18, "{rel:?}");
            assert_eq!(parts.branch_offsets, [0x08, 0x14], "{rel:?}");
            assert_eq!(&parts.text[8..12], &[0, 0, 0, 0], "{rel:?}");
            #[rustfmt::skip]
            let tail: &[u8] = &[
                0x38, 0xa0, 0x00, 0x00, // li r5,0
                0x38, 0x80, 0x00, 0x00, // li r4,0
                0x00, 0x00, 0x00, 0x00, // b  ?h3  <- the caller's
            ];
            assert_eq!(&parts.text[12..], tail, "{rel:?}");
            parts.text[..8].to_vec()
        }

        // --- signed, `cmpwi cr6,r5,0` (w9_rel_signed.cpp) --------------------
        const CMPWI: [u8; 4] = [0x2f, 0x05, 0x00, 0x00];
        for (rel, branch) in [
            (Rel::Eq, [0x40, 0x9a, 0x00, 0x08]), // bne cr6,+8   BO=4  bit EQ
            (Rel::Ne, [0x41, 0x9a, 0x00, 0x08]), // beq cr6,+8   BO=12 bit EQ  <- first `bt`
            (Rel::Lt, [0x40, 0x98, 0x00, 0x08]), // bge cr6,+8   BO=4  bit LT
            (Rel::Ge, [0x41, 0x98, 0x00, 0x08]), // blt cr6,+8   BO=12 bit LT
            (Rel::Gt, [0x40, 0x99, 0x00, 0x08]), // ble cr6,+8   BO=4  bit GT
            (Rel::Le, [0x41, 0x99, 0x00, 0x08]), // bgt cr6,+8   BO=12 bit GT
        ] {
            let mut want = CMPWI.to_vec();
            want.extend_from_slice(&branch);
            assert_eq!(grid(rel, true, 0), want, "signed {rel:?}");
        }

        // --- unsigned, `cmplwi cr6,r5,7` (w9_rel_unsigned.cpp) --------------
        // Same six branch words: the CR bit and the BO come from the relation
        // alone, and the compare's signedness moves only the compare word. The
        // immediate is 7, the first non-zero one the port has ever been graded
        // on.
        const CMPLWI: [u8; 4] = [0x2b, 0x05, 0x00, 0x07];
        for (rel, branch) in [
            (Rel::Eq, [0x40, 0x9a, 0x00, 0x08]),
            (Rel::Ne, [0x41, 0x9a, 0x00, 0x08]),
            (Rel::Lt, [0x40, 0x98, 0x00, 0x08]),
            (Rel::Ge, [0x41, 0x98, 0x00, 0x08]),
            (Rel::Gt, [0x40, 0x99, 0x00, 0x08]),
            (Rel::Le, [0x41, 0x99, 0x00, 0x08]),
        ] {
            let mut want = CMPLWI.to_vec();
            want.extend_from_slice(&branch);
            assert_eq!(grid(rel, false, 7), want, "unsigned {rel:?}");
        }
    }

    /// The branch sense is the **negation** of the IL relation, per relation.
    ///
    /// This compares the port's table to itself and is kept only as a locator:
    /// the assertion with a byte behind it is
    /// `the_relation_grid_matches_the_real_obj_bytes`, above.
    #[test]
    fn the_branch_sense_negates_every_relation() {
        assert_eq!(branch_sense(Rel::Eq), (BO_FALSE, CR_BIT_EQ)); // bne
        assert_eq!(branch_sense(Rel::Ne), (BO_TRUE, CR_BIT_EQ)); // beq
        assert_eq!(branch_sense(Rel::Lt), (BO_FALSE, CR_BIT_LT)); // bge
        assert_eq!(branch_sense(Rel::Ge), (BO_TRUE, CR_BIT_LT)); // blt
        assert_eq!(branch_sense(Rel::Gt), (BO_FALSE, CR_BIT_GT)); // ble
        assert_eq!(branch_sense(Rel::Le), (BO_TRUE, CR_BIT_GT)); // bgt
    }

    /// A signed comparison emits `cmpwi`, an unsigned one `cmplwi`, and the
    /// choice comes from the operand TYPE alone.
    #[test]
    fn signedness_selects_the_compare_instruction() {
        let mut f = memfree();
        f.cond_pair.as_mut().unwrap().signed = true;
        let parts = cond_pair_parts(&f, f.cond_pair.as_ref().unwrap()).unwrap();
        assert_eq!(&parts.text[4..8], &[0x2f, 0x03, 0x00, 0x00]);
    }
}
