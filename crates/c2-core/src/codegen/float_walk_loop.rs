//! **W-BLOCKIR — the float array-walk counted loop's lowering**, the whole of
//! `src/system/synth_xbox/IPP_basicmath_xbox.cpp`.
//!
//! ```text
//!  A Compound  b[i] OP= a[i]        mr    r11, b            <- the park, ABOVE the guard
//!                                   cmplwi cr6, r3, 0       <- wb-loop pass 1
//!                                   bclr  12, 26            <- …as a conditional RETURN
//!                                   mtctr r3                <- wb-loop pass 2
//!                                   sub   r10, a, b         <- the base difference
//!                                   lfsx  f0, r10, r11
//!                                   lfs   f13, 0(r11)
//!                                   fOPs  f0, f0, f13
//!                                   stfs  f0, 0(r11)
//!                                   addi  r11, r11, 4
//!                                   bdnz  .-20
//!                                   blr
//!  B Scalar    b[i] OP= s           cmplwi/bclr · addi r11,b,-4 · mtctr r3
//!                                   lfs f0,4(r11) · fOPs f0,f0,f1
//!                                   stfsu f0,4(r11) · bdnz .-12 · blr
//!  C Binary    c[i] = a[i] OP b[i]  cmplwi/bclr · mr r11,b · mtctr r3
//!                                   sub r10,a,b · sub r9,c,b
//!                                   lfsx f0,r10,r11 · lfs f13,0(r11)
//!                                   fOPs f0,f0,f13 · stfsx f0,r9,r11
//!                                   addi r11,r11,4 · bdnz .-20 · blr
//! ```
//!
//! # This is a TRANSCRIPTION and saying so is the point
//!
//! [`super::ptr_walk_loop`]'s header sentence, and it applies here whole: there
//! is **no register allocator, no scheduler and no CFG builder** in this file.
//! `r11`, `r10`, `r9`, `f0`, `f13` and `f1` are here because real `c2` put them
//! here, in every one of the 28 cells of `work/w-blockir/probe/`, and the module
//! that would *derive* them does not exist.
//!
//! What the probe grid bought is the **boundary** — which loops are in the class
//! and which are not, with a compiled cell on each side — and three per-shape
//! constants it is honest about:
//!
//! | constant | witnesses | the cell that would break a wider rule |
//! |---|---|---|
//! | the walker | 6 / 4 / 4 | `c4` — three right-hand arrays walks the SECOND |
//! | the park's position | 6 / 4 / 4 | `c5`, `c6`, `d5` refute the register test |
//! | the load order | 6 | `c7`, `c8` — `-=`/`/=` swap the two loads |
//!
//! `docs/whitebox/WB_LOOP_FINDINGS.md` §4.3 says of walker selection *"In all
//! five measured cells the walker is the array whose access is emitted last,
//! which is circular. `#1767`'s rule against a two-point fit applies; not
//! claimed."* This file does not claim it either. It writes down three answers
//! and names their witness counts.
//!
//! # Where the back edge's displacement comes from
//!
//! Computed here, directly, through [`encode_bdnz`] — **not** through
//! [`super::labels`]. That map's invariant 4 refuses every backward reference,
//! and it is untouched by this class for the same reason it is untouched by
//! [`super::ptr_walk_loop`] and [`super::ptr_walk_chain_loop`]: neither carrier
//! routes through it, so the map never sees the reference
//! (`codegen/labels.rs`'s own 2026-08-08 correction).
//!
//! # What is NOT here, and each was compiled before it was declined
//!
//! * **`-=` and `/=`.** A different word *order*, not a different field: the
//!   non-commutative op pins its left operand, so the walker's own `lfs` is
//!   emitted first and the other array's `lfsx` second (cells `c7`, `c8`). An
//!   arm that substituted only the opcode would emit two loads in the wrong
//!   order, so there is no such arm — **absent rather than
//!   unreachable-but-present**, which is board #1148's shape.
//! * **A signed counter or bound** (`cmpwi`/`bclr 4,25`, cell `c9`), **`double`
//!   arrays** (`lfdx`/`lfd`/`fadd`/`stfd` and a stride of 8, cell `c11`), and
//!   **`int` arrays** (`lwzx`/`lwz`/`add`/`stw`, cell `c14`). The skeleton
//!   generalises to all three and this lane does not ship the generalisation:
//!   the reader refuses each, so no body reaches this file that would need a
//!   word it does not have.
//! * **`/Ox`.** It unrolls four times behind a `cmpwi cr6,r3,4` pre-test with a
//!   remainder loop, `lfsu`, and 688 bytes in one section
//!   (`work/w-blockir/probe/ipp_ox.dis.txt`). The mode gate is in the READER,
//!   before any body byte, because a gate that lives only here is a fact the
//!   census cannot ask (board #1638).
//!
//! # The update form is one word in one shape, and it is not `wb-loop`'s pass 3
//!
//! Shape B ends its body on `stfsu`, which is the form `wb-loop` §4.4 gridded
//! and `w-bdnz` declined **as a general rule** — four rivals, ten cells, none
//! elected, RU-H filed unfrozen. Nothing here elects one. This is a transcribed
//! word in a shape with four graded witnesses (`MulConstant_InPlace`, `c12`,
//! `d3`, `d4`), and the reader admits nothing else; the general question is left
//! exactly as open as `w-bdnz` left it.

use super::encode::{
    encode_addi, encode_bclr, encode_bdnz, encode_blr, encode_cmplwi, encode_fadd, encode_fmul,
    encode_lfs, encode_lfsx, encode_mr, encode_mtctr, encode_stfs, encode_stfsu, encode_stfsx,
    encode_subf, BO_TRUE, CR_BIT_EQ, CR_COMPARE, cr_bi,
};
use crate::BackendError;
use c2_il::{FloatWalkLoop, FloatWalkOp, FloatWalkShape, IlFunction};

/// The induction pointer: `r11`, the first register the allocator hands out
/// (`WB_REGALLOC_FINDINGS.md` §3.4's order predicts it) — but this constant is
/// **read off this class's own objs** and not adopted from that reading, which
/// is why the class carries no `DISCLOSURE.md` row.
const R_WALK: u8 = 11;
/// The first base difference: `r10`. Second in the same order.
const R_DIFF1: u8 = 10;
/// The second base difference: `r9`, used only by shape C's store.
const R_DIFF2: u8 = 9;
/// The bound and trip count: `r3`, formal slot 0 arriving in place.
const R_BOUND: u8 = 3;
/// The first argument register. Formal slot `i` arrives in `ARG0 + i`.
const ARG0: u8 = 3;

/// The accumulator FPR: `f0`. Note the contrast with
/// [`super::leaf::float`]'s pool — there `f0` is allocatable and comes first,
/// and here it is a fixed constant, because this class has no expression tree to
/// allocate over.
const F_ACC: u8 = 0;
/// The second operand's FPR: `f13`, the walker's own `lfs` destination.
const F_OTHER: u8 = 13;
/// The scalar formal's FPR: `f1`. Float parameters occupy `f1…f13` in
/// float-parameter order, and the reader requires the scalar to be the **only**
/// float formal and the **last** formal, so it is always the first — hence `f1`.
const F_SCALAR: u8 = 1;

/// `float` is four bytes: the induction step and shape B's displacement.
const STEP: i16 = 4;

/// The A-form word the body performs, `fD = fA OP fB` in single precision.
///
/// Injective by construction — the `#[test]` at the bottom is the pin that two
/// operations cannot collapse onto one encoding — and note the asymmetry the
/// encoders already carry: `fmuls` puts its multiplier in the **C** field where
/// `fadds` uses **B**, so this is a match rather than one call with a varying
/// XO.
fn body_word(op: FloatWalkOp, fd: u8, fa: u8, fb: u8) -> [u8; 4] {
    match op {
        FloatWalkOp::Add => encode_fadd(false, fd, fa, fb),
        FloatWalkOp::Mul => encode_fmul(false, fd, fa, fb),
    }
}

/// The guard: `cmplwi cr6, r3, 0` then `bclr 12, 26` — `wb-loop`'s pass 1,
/// realised as a **conditional return** because the loop is the function's tail
/// (reader clause 10). There is no forward-branch arm, and it is absent rather
/// than unreachable: an arm no cell grades is an arm that will be wrong when
/// something finally reaches it.
fn guard(out: &mut Vec<u8>) {
    out.extend_from_slice(&encode_cmplwi(CR_COMPARE, R_BOUND, 0));
    out.extend_from_slice(&encode_bclr(BO_TRUE, cr_bi(CR_COMPARE, CR_BIT_EQ)));
}

/// `sub rD, rA, rB` — the base difference `other − walker`.
///
/// `subf rD,rA,rB` computes `rB − rA`, so the operands read in the order that
/// looks wrong and is right; `?Add_InPlace`'s own word is `7d452050`, which is
/// `subf r10, r5, r4` = `r4 − r5` = `f1 − f2`.
fn base_diff(out: &mut Vec<u8>, rd: u8, other: u8, walker: u8) {
    out.extend_from_slice(&encode_subf(rd, walker, other));
}

/// Select `.text` for one accepted body.
///
/// Every register below is derived from a **formal index**, never carried, so
/// the positional formal→register map lives in exactly one expression.
pub(crate) fn float_walk_loop_text(func: &IlFunction) -> Result<Vec<u8>, BackendError> {
    let l = func
        .float_walk_loop
        .as_ref()
        .ok_or(BackendError::NotImplemented("not a float array-walk loop".into()))?;
    let reg = |idx: usize| -> Result<u8, BackendError> {
        let r = ARG0 as usize + idx;
        // The reader bounds the arity at 4, so this cannot fire on an accepted
        // body; it is here because a register number computed from an index is
        // exactly the arithmetic that goes wrong silently.
        if idx == 0 || r > ARG0 as usize + 3 {
            return Err(BackendError::NotImplemented("float walk loop: formal index".into()));
        }
        Ok(r as u8)
    };
    let walker = reg(l.walker)?;
    let mut out: Vec<u8> = Vec::with_capacity(52);

    match l.shape {
        // ---- A: `b[i] OP= a[i]` -------------------------------------------
        FloatWalkShape::Compound => {
            let [other] = shape_others::<1>(l)?;
            let other = reg(other)?;
            // **The park floats ABOVE the guard here and below it in C.** Six
            // witnesses; the rule that fits all seven measured cells is the
            // number of preheader `sub`s, and the register test this lane
            // registered in advance is refuted by `c5`, `c6` and `d5`. Shipped
            // as a per-shape constant rather than as a rule (PREREG §5.2's P2).
            out.extend_from_slice(&encode_mr(R_WALK, walker));
            guard(&mut out);
            out.extend_from_slice(&encode_mtctr(R_BOUND));
            base_diff(&mut out, R_DIFF1, other, walker);
            // The commutative op loads the OTHER array first. `-=`/`/=` swap
            // these two words and are refused by the reader.
            out.extend_from_slice(&encode_lfsx(F_ACC, R_DIFF1, R_WALK));
            out.extend_from_slice(&encode_lfs(false, F_OTHER, R_WALK, 0));
            out.extend_from_slice(&body_word(l.op, F_ACC, F_ACC, F_OTHER));
            out.extend_from_slice(&encode_stfs(false, F_ACC, R_WALK, 0));
            out.extend_from_slice(&encode_addi(R_WALK, R_WALK, STEP));
        }
        // ---- B: `b[i] OP= s` ----------------------------------------------
        FloatWalkShape::Scalar => {
            let [] = shape_others::<0>(l)?;
            guard(&mut out);
            // The pre-bias, which is what makes the update form legal: every
            // access on the walking pointer is D-form here, so the write-back
            // has nothing to move out from under.
            out.extend_from_slice(&encode_addi(R_WALK, walker, -STEP));
            out.extend_from_slice(&encode_mtctr(R_BOUND));
            out.extend_from_slice(&encode_lfs(false, F_ACC, R_WALK, STEP));
            out.extend_from_slice(&body_word(l.op, F_ACC, F_ACC, F_SCALAR));
            out.extend_from_slice(&encode_stfsu(F_ACC, R_WALK, STEP));
        }
        // ---- C: `c[i] = a[i] OP b[i]` -------------------------------------
        FloatWalkShape::Binary => {
            let [src, dst] = shape_others::<2>(l)?;
            let (src, dst) = (reg(src)?, reg(dst)?);
            guard(&mut out);
            out.extend_from_slice(&encode_mr(R_WALK, walker));
            out.extend_from_slice(&encode_mtctr(R_BOUND));
            base_diff(&mut out, R_DIFF1, src, walker);
            base_diff(&mut out, R_DIFF2, dst, walker);
            out.extend_from_slice(&encode_lfsx(F_ACC, R_DIFF1, R_WALK));
            out.extend_from_slice(&encode_lfs(false, F_OTHER, R_WALK, 0));
            out.extend_from_slice(&body_word(l.op, F_ACC, F_ACC, F_OTHER));
            out.extend_from_slice(&encode_stfsx(F_ACC, R_DIFF2, R_WALK));
            out.extend_from_slice(&encode_addi(R_WALK, R_WALK, STEP));
        }
    }

    // The latch, and the one computed field in the whole file: the back edge
    // reaches the FIRST word of the loop body, which is the word after the last
    // preheader instruction.
    let body_words = match l.shape {
        FloatWalkShape::Compound => 5,
        FloatWalkShape::Scalar => 3,
        FloatWalkShape::Binary => 5,
    };
    let disp = -4 * (body_words as i32);
    let latch =
        encode_bdnz(disp).ok_or(BackendError::NotImplemented("float walk loop: back edge".into()))?;
    out.extend_from_slice(&latch);
    out.extend_from_slice(&encode_blr());
    Ok(out)
}

/// Read the non-walker indices as a fixed-size array, refusing rather than
/// truncating when the count disagrees with the shape.
///
/// The reader builds this list and the emitter's word count depends on its
/// length, so a disagreement between the two is exactly the class of defect that
/// produces a plausible wrong obj. Checked rather than assumed.
fn shape_others<const N: usize>(l: &FloatWalkLoop) -> Result<[usize; N], BackendError> {
    if l.others.len() != N {
        return Err(BackendError::NotImplemented(
            "float walk loop: base-difference count disagrees with the shape".into(),
        ));
    }
    let mut out = [0usize; N];
    out[..N].copy_from_slice(&l.others[..N]);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(shape: FloatWalkShape, op: FloatWalkOp, walker: usize, others: Vec<usize>) -> IlFunction {
        let n = if shape == FloatWalkShape::Binary { 4 } else { 3 };
        let mut fun = crate::codegen::testutil::func_with((0..n as u32).collect(), Vec::new());
        fun.float_walk_loop = Some(FloatWalkLoop {
            params: (0..n as u32).collect(),
            shape,
            op,
            walker,
            others,
        });
        fun
    }

    /// **The four bodies of `IPP_basicmath_xbox.cpp`, word for word**, against
    /// `work/w-blockir/ref/ipp.dis.txt` — the real obj `cl.exe`
    /// 16.00.11886.00 produced under wibo at the workload's own flags.
    ///
    /// This is the whole warranty of the file: the emitter has no free field
    /// that is not pinned by one of these four, and the reader refuses
    /// everything that would need a fifth.
    #[test]
    fn the_four_ipp_bodies_reproduce_byte_for_byte() {
        // ?Add_InPlace@IPP@@YAXIPBMPAM@Z — 48 B. formals size,f1,f2; walker f2.
        let want: &[u8] = &[
            0x7c, 0xab, 0x2b, 0x78, // mr r11,r5
            0x2b, 0x03, 0x00, 0x00, // cmplwi cr6,r3,0
            0x4d, 0x9a, 0x00, 0x20, // bclr 12,26
            0x7c, 0x69, 0x03, 0xa6, // mtctr r3
            0x7d, 0x45, 0x20, 0x50, // sub r10,r4,r5
            0x7c, 0x0a, 0x5c, 0x2e, // lfsx f0,r10,r11
            0xc1, 0xab, 0x00, 0x00, // lfs f13,0(r11)
            0xec, 0x00, 0x68, 0x2a, // fadds f0,f0,f13
            0xd0, 0x0b, 0x00, 0x00, // stfs f0,0(r11)
            0x39, 0x6b, 0x00, 0x04, // addi r11,r11,4
            0x42, 0x00, 0xff, 0xec, // bdnz .-20
            0x4e, 0x80, 0x00, 0x20, // blr
        ];
        let got =
            float_walk_loop_text(&f(FloatWalkShape::Compound, FloatWalkOp::Add, 2, vec![1]))
                .unwrap();
        assert_eq!(got.len(), 48);
        assert_eq!(got, want);

        // ?Mul_InPlace@IPP@@YAXIPBMPAM@Z — the same 48 B with `fmuls`.
        let got =
            float_walk_loop_text(&f(FloatWalkShape::Compound, FloatWalkOp::Mul, 2, vec![1]))
                .unwrap();
        let mut want_mul = want.to_vec();
        want_mul[28..32].copy_from_slice(&[0xec, 0x00, 0x03, 0x72]); // fmuls f0,f0,f13
        assert_eq!(got, want_mul);

        // ?MulConstant_InPlace@IPP@@YAXIPAMM@Z — 36 B, the update form.
        let want: &[u8] = &[
            0x2b, 0x03, 0x00, 0x00, // cmplwi cr6,r3,0
            0x4d, 0x9a, 0x00, 0x20, // bclr 12,26
            0x39, 0x64, 0xff, 0xfc, // addi r11,r4,-4
            0x7c, 0x69, 0x03, 0xa6, // mtctr r3
            0xc0, 0x0b, 0x00, 0x04, // lfs f0,4(r11)
            0xec, 0x00, 0x00, 0x72, // fmuls f0,f0,f1
            0xd4, 0x0b, 0x00, 0x04, // stfsu f0,4(r11)
            0x42, 0x00, 0xff, 0xf4, // bdnz .-12
            0x4e, 0x80, 0x00, 0x20, // blr
        ];
        let got =
            float_walk_loop_text(&f(FloatWalkShape::Scalar, FloatWalkOp::Mul, 1, vec![])).unwrap();
        assert_eq!(got.len(), 36);
        assert_eq!(got, want);

        // ?Mul@IPP@@YAXIPBM0PAM@Z — 52 B. formals size,f1,f2,f3; walker f2.
        let want: &[u8] = &[
            0x2b, 0x03, 0x00, 0x00, // cmplwi cr6,r3,0
            0x4d, 0x9a, 0x00, 0x20, // bclr 12,26
            0x7c, 0xab, 0x2b, 0x78, // mr r11,r5
            0x7c, 0x69, 0x03, 0xa6, // mtctr r3
            0x7d, 0x45, 0x20, 0x50, // sub r10,r4,r5
            0x7d, 0x25, 0x30, 0x50, // sub r9,r6,r5
            0x7c, 0x0a, 0x5c, 0x2e, // lfsx f0,r10,r11
            0xc1, 0xab, 0x00, 0x00, // lfs f13,0(r11)
            0xec, 0x00, 0x03, 0x72, // fmuls f0,f0,f13
            0x7c, 0x09, 0x5d, 0x2e, // stfsx f0,r9,r11
            0x39, 0x6b, 0x00, 0x04, // addi r11,r11,4
            0x42, 0x00, 0xff, 0xec, // bdnz .-20
            0x4e, 0x80, 0x00, 0x20, // blr
        ];
        let got =
            float_walk_loop_text(&f(FloatWalkShape::Binary, FloatWalkOp::Mul, 2, vec![1, 3]))
                .unwrap();
        assert_eq!(got.len(), 52);
        assert_eq!(got, want);
    }

    /// The two operations do not collapse onto one encoding, and `fmuls` puts
    /// its operand in a different field from `fadds` — which is why the body is
    /// a `match` and not one call with a varying extended opcode.
    #[test]
    fn the_two_operations_are_injective_and_use_different_fields() {
        let a = body_word(FloatWalkOp::Add, 0, 0, 13);
        let m = body_word(FloatWalkOp::Mul, 0, 0, 13);
        assert_ne!(a, m);
        assert_eq!(a, [0xec, 0x00, 0x68, 0x2a]);
        assert_eq!(m, [0xec, 0x00, 0x03, 0x72]);
    }

    /// A carrier whose `others` list disagrees with its shape REFUSES. The two
    /// halves are built in different crates and a silent disagreement here is
    /// exactly the defect class that produces a plausible wrong obj.
    #[test]
    fn a_base_difference_count_that_disagrees_with_the_shape_refuses() {
        assert!(float_walk_loop_text(&f(
            FloatWalkShape::Compound,
            FloatWalkOp::Add,
            2,
            vec![1, 3]
        ))
        .is_err());
        assert!(
            float_walk_loop_text(&f(FloatWalkShape::Binary, FloatWalkOp::Mul, 2, vec![1])).is_err()
        );
        assert!(
            float_walk_loop_text(&f(FloatWalkShape::Scalar, FloatWalkOp::Mul, 1, vec![1])).is_err()
        );
        // …and a walker index of 0 is the BOUND's register, which no accepted
        // body can name.
        assert!(
            float_walk_loop_text(&f(FloatWalkShape::Compound, FloatWalkOp::Add, 0, vec![1]))
                .is_err()
        );
    }

    /// The back edge reaches the first word of the loop body in each shape, and
    /// the two displacements are the ones real `c2` emitted.
    #[test]
    fn the_back_edge_lands_on_the_first_body_word() {
        for (shape, others, disp_at, want) in [
            (FloatWalkShape::Compound, vec![1], 40usize, -20i32),
            (FloatWalkShape::Scalar, vec![], 28, -12),
            (FloatWalkShape::Binary, vec![1, 3], 44, -20),
        ] {
            let walker = if shape == FloatWalkShape::Scalar { 1 } else { 2 };
            let text = float_walk_loop_text(&f(shape, FloatWalkOp::Mul, walker, others)).unwrap();
            let latch = &text[disp_at..disp_at + 4];
            assert_eq!(latch, encode_bdnz(want).unwrap());
            // …and the word it names is a load, i.e. the body and not a
            // preheader instruction.
            let target = (disp_at as i32 + want) as usize;
            assert!(
                text[target] == 0x7c || text[target] == 0xc0,
                "{shape:?} back edge lands on {:#04x}",
                text[target]
            );
        }
    }
}
