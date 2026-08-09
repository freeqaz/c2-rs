//! **W-POOL2 — the intrusive free-list PUSH and POP.**
//!
//! Six and seven words, both leaves, both folding their guard to a conditional
//! return. Every word below is off this lane's own capture of `Pool.obj` at the
//! workload's flags (`work/w-pool2/ref/Pool.obj`, `scripts/gt_dump.py`), not
//! from a table.
//!
//! ```text
//!   ?Free@Pool@@QAAXPAX@Z   24 B         ?Alloc@Pool@@QAAPAXXZ   28 B
//!     2b040000  cmplwi cr6,r4,0            7c6b1b78  mr     r11,r3
//!     4d9a0020  bclr   12,26               80630000  lwz    r3,0(r3)
//!     81630000  lwz    r11,0(r3)           2b030000  cmplwi cr6,r3,0
//!     91640000  stw    r11,0(r4)           4d9a0020  bclr   12,26
//!     90830000  stw    r4,0(r3)            81430000  lwz    r10,0(r3)
//!     4e800020  blr                        914b0000  stw    r10,0(r11)
//!                                          4e800020  blr
//! ```
//!
//! # The three facts these fourteen words rest on
//!
//! 1. **The guard is fold band 2** — `bclr 12,26`, a conditional return with no
//!    label and no displacement. `docs/CFG_SHAPE.md` §3.5 names both of these
//!    functions in that band, and [`c2_il::func::body::shapes::pool_free_list`]
//!    explains why board **#187**'s band-1 ↔ band-2 cost model is neither read
//!    nor needed: the class's guarded arm computes no value, so the branchless
//!    select band 1 *is* has nothing to select between.
//! 2. **The scratch order is r11 then r10** — `WB_REGALLOC_FINDINGS` §3.4, and
//!    it is what PUSH and POP disagree about. PUSH needs one scratch and takes
//!    r11; POP needs two, because `this` has to survive the load that overwrites
//!    r3, and takes r11 for the parked `this` and r10 for the link.
//! 3. **POP's `return nullptr` costs ZERO instructions.** c2 puts the popped
//!    head in **r3** — the return register — so on the guarded edge r3 already
//!    holds 0 and the literal needs no `li`. The recognizer admits only the
//!    literal `0` for exactly this reason; a different literal is a different
//!    body, with an `li` and a real `bf`, and it is refused rather than emitted
//!    wrong.
//!
//! # `/O1` only
//!
//! At `/Ox` **`?Alloc` stops folding**: `cmplwi ; bf 26,+8 ; blr ; lwz ; stw ;
//! blr` — band 3, seven words, two `blr`s. The gate is asked in the parser
//! (board #1638) *and* here, which is what makes `function_gate` and both
//! writers ask it in one place.

use c2_il::{PoolFreeList, PoolFreeListOp};

use crate::codegen::encode::{
    cr_bi, encode_bclr, encode_blr, encode_cmplwi, encode_lwz, encode_mr, encode_stw,
    BO_TRUE, CR_BIT_EQ, CR_COMPARE,
};
use crate::codegen::select::{out_of_class, OptMode};
use crate::BackendError;

/// `this`, always r3 on a member function.
const R_THIS: u8 = 3;
/// The first scratch (`WB_REGALLOC_FINDINGS` §3.4: r11, then r10, then r9).
const R_S1: u8 = 11;
/// The second scratch, needed only by POP.
const R_S2: u8 = 10;

/// Emit the whole body. No relocation, no pooled constant, no label and no
/// branch with a target — so the caller takes it as an ordinary
/// [`crate::codegen::Selected::Plain`].
pub(crate) fn pool_free_list_text(
    g: &PoolFreeList,
    mode: OptMode,
) -> Result<Vec<u8>, BackendError> {
    // The mode clause, restated here even though the recognizer already asked
    // it: `select_function` is what `function_gate` runs, so a body that reached
    // codegen at the other mode would be a census/gate disagreement, and that
    // counter reading 0 is the only thing keeping the census honest about what
    // the port accepts (#1638, #1710).
    if mode != OptMode::O1 {
        return Err(out_of_class(
            "the free-list guard pair at `/Ox`: `?Alloc` stops folding to `bclr` there \
             and emits a seven-word band-3 body with two `blr`s — a different body, \
             and this lane graded only the `/O1` one",
        ));
    }
    let off = i16::try_from(g.off).map_err(|_| {
        out_of_class(
            "a free-list head member beyond the `lwz`/`stw` displacement: a wide \
             offset is a `lis`+`ori` into a scratch and has no capture here",
        )
    })?;
    let mut t: Vec<u8> = Vec::with_capacity(28);
    match g.op {
        // ---- PUSH — `void Pool::Free(void *v)` ----------------------------
        PoolFreeListOp::Push => {
            if g.params.len() != 2 {
                return Err(out_of_class(
                    "a free-list PUSH with other than `this` and one formal: the \
                     formal's slot index is its register and the plan is measured at one",
                ));
            }
            // Slot 1 of a member function's argument list. Named through the
            // slot rather than written as `4`, so a class that ever admits a
            // second formal cannot silently keep this register.
            let r_v = R_THIS + 1;
            t.extend_from_slice(&encode_cmplwi(CR_COMPARE, r_v, 0));
            t.extend_from_slice(&encode_bclr(BO_TRUE, cr_bi(CR_COMPARE, CR_BIT_EQ)));
            t.extend_from_slice(&encode_lwz(R_S1, R_THIS, off));
            t.extend_from_slice(&encode_stw(R_S1, r_v, 0));
            t.extend_from_slice(&encode_stw(r_v, R_THIS, off));
        }
        // ---- POP — `void *Pool::Alloc()` ----------------------------------
        PoolFreeListOp::Pop => {
            if g.params.len() != 1 {
                return Err(out_of_class(
                    "a free-list POP with a formal: nothing here graded one, and a \
                     formal in r4 changes which register `this` is parked in",
                ));
            }
            // `this` is parked FIRST, because the very next word overwrites r3
            // with the loaded head — which is also the returned value, and is
            // why the guarded arm needs no `li`.
            t.extend_from_slice(&encode_mr(R_S1, R_THIS));
            t.extend_from_slice(&encode_lwz(R_THIS, R_THIS, off));
            t.extend_from_slice(&encode_cmplwi(CR_COMPARE, R_THIS, 0));
            t.extend_from_slice(&encode_bclr(BO_TRUE, cr_bi(CR_COMPARE, CR_BIT_EQ)));
            t.extend_from_slice(&encode_lwz(R_S2, R_THIS, 0));
            t.extend_from_slice(&encode_stw(R_S2, R_S1, off));
        }
    }
    t.extend_from_slice(&encode_blr());
    debug_assert_eq!(
        t.len(),
        4 * match g.op {
            PoolFreeListOp::Push => 6,
            PoolFreeListOp::Pop => 7,
        },
        "the class's body length is a constant of the variant"
    );
    Ok(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn push(off: i32) -> PoolFreeList {
        PoolFreeList { params: vec![1, 2], op: PoolFreeListOp::Push, off }
    }

    fn pop(off: i32) -> PoolFreeList {
        PoolFreeList { params: vec![1], op: PoolFreeListOp::Pop, off }
    }

    fn words(t: &[u8]) -> Vec<u32> {
        t.chunks(4).map(|c| u32::from_be_bytes([c[0], c[1], c[2], c[3]])).collect()
    }

    /// `?Free@Pool@@QAAXPAX@Z`, word for word off `work/w-pool2/ref/Pool.obj`.
    #[test]
    fn push_is_pool_free_word_for_word() {
        let t = pool_free_list_text(&push(0), OptMode::O1).unwrap();
        assert_eq!(
            words(&t),
            vec![0x2b04_0000, 0x4d9a_0020, 0x8163_0000, 0x9164_0000, 0x9083_0000, 0x4e80_0020]
        );
    }

    /// `?Alloc@Pool@@QAAPAXXZ`, word for word off the same obj — and the
    /// guarded `return nullptr` contributes NO word, which is the class's third
    /// fact stated as a length.
    #[test]
    fn pop_is_pool_alloc_word_for_word() {
        let t = pool_free_list_text(&pop(0), OptMode::O1).unwrap();
        assert_eq!(
            words(&t),
            vec![
                0x7c6b_1b78,
                0x8063_0000,
                0x2b03_0000,
                0x4d9a_0020,
                0x8143_0000,
                0x914b_0000,
                0x4e80_0020
            ]
        );
    }

    /// The member offset is the only free field, and it moves exactly the two
    /// displacements it should.
    #[test]
    fn the_member_offset_moves_both_displacements_and_nothing_else() {
        let base = words(&pool_free_list_text(&push(0), OptMode::O1).unwrap());
        let moved = words(&pool_free_list_text(&push(8), OptMode::O1).unwrap());
        assert_eq!(base.len(), moved.len());
        // words 2 and 4 are the `lwz r11,off(r3)` and `stw r4,off(r3)`.
        for (i, (a, b)) in base.iter().zip(moved.iter()).enumerate() {
            if i == 2 || i == 4 {
                assert_eq!(*b, *a + 8, "word {i} carries the displacement");
            } else {
                assert_eq!(a, b, "word {i} does not");
            }
        }
    }

    #[test]
    fn the_class_is_o1_only() {
        assert!(pool_free_list_text(&push(0), OptMode::O1).is_ok());
        assert!(pool_free_list_text(&push(0), OptMode::Ox).is_err());
        assert!(pool_free_list_text(&pop(0), OptMode::Ox).is_err());
    }

    #[test]
    fn an_arity_the_register_plan_was_not_measured_at_refuses() {
        let mut g = push(0);
        g.params = vec![1, 2, 3];
        assert!(pool_free_list_text(&g, OptMode::O1).is_err());
        let mut g = pop(0);
        g.params = vec![1, 2];
        assert!(pool_free_list_text(&g, OptMode::O1).is_err());
    }

    #[test]
    fn a_wide_member_offset_refuses_rather_than_truncating() {
        assert!(pool_free_list_text(&push(0x1_0000), OptMode::O1).is_err());
    }
}
