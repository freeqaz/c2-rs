//! The comparison leaf: relation × signedness × literal, including both i16
//! boundaries. See `docs/CODEGEN_W6_COMPARE.md` and `docs/CODEGEN_W6_O1.md`.

use crate::BackendError;
use crate::codegen::encode::{
    encode_adde,
    encode_addi,
    encode_addic,
    encode_addze,
    encode_andc,
    encode_blr,
    encode_clrlwi31,
    encode_cntlzw,
    encode_eqv,
    encode_neg,
    encode_orc,
    encode_rlwinm,
    encode_srawi,
    encode_srwi31,
    encode_subfc,
    encode_subfe,
    encode_subfic,
    encode_subfze,
    encode_xori,
};
use crate::codegen::select::{OptMode, RET_REG, out_of_class};

/// **"Is the difference already in r11 zero?"** — the two words that turn a
/// difference into a 0/1 in r3, with the `/O1` temp collapse.
///
/// One locator, two consumers. The comparison *leaf* reaches it after
/// `addi r11,a,-k` ([`compare_leaf_text`]'s `Rel::Eq` arm); the **two-call
/// comparator** reaches it after `subf r11,<lhs>,<rhs>`, which is the
/// register-register spine family `docs/CMP_PRODUCES_A_VALUE.md` reading 4
/// names. The instructions after the difference are byte-identical in both, in
/// all four modes:
///
/// ```text
///   /Ox, /O2, packed   cntlzw r10,r11 ; rlwinm r3,r10,27,31,31
///   /O1                cntlzw r11,r11 ; rlwinm r3,r11,27,31,31
/// ```
///
/// `/O1` collapses the temp because the `cntlzw` is the difference's last use —
/// the same rule, stated the same way, that every arm of the leaf spine carries.
/// Copying these two words into the second consumer instead of importing them is
/// exactly the one-fact-two-implementations drift `docs/GAPS.md` §6 records, and
/// the mode axis is the field that would have silently diverged.
pub fn eq_zero_of_difference_in_r11(mode: OptMode) -> Vec<u8> {
    let d = if mode == OptMode::O1 { 11 } else { 10 };
    let mut t = Vec::with_capacity(8);
    t.extend_from_slice(&encode_cntlzw(d, 11));
    t.extend_from_slice(&encode_rlwinm(RET_REG, d, 27, 31, 31));
    t
}

/// Select `.text` for a **W6 comparison leaf** (`return a <rel> k;`), returning
/// the spine plus its trailing `blr`.
///
/// c2 materializes these branchlessly — it emits no `cmpw`/`cmplw` at all for
/// this shape — using carry-bit and bit-extraction idioms. Every sequence below
/// is transcribed from a live capture (`docs/CODEGEN_W6_COMPARE.md` §3–§4) and
/// each instruction word is re-encoded from its fields here.
///
/// **The `k == 0` folds are dispatched first and are not optional.** c2 does not
/// run a zero literal through the general spine; it folds, sometimes to a
/// shorter sequence and sometimes to a constant. Two of the six fixture
/// functions land in that table, and emitting the general spine for them would
/// be a wrong-length, wrong-bytes mis-emit. This mirrors the `g(a) + 0` identity
/// fold in W4b2-vi: a zero operand changes the *shape*, not just an immediate.
///
/// Temporaries are allocated descending from r11 in emission order, one physical
/// register per temp with no reuse — including two kinds of slot consumed by
/// values that are never read (a `subfe u,v,v` don't-care source, and a
/// `subfc`/`subfic` destination whose only live output is the carry). Those
/// register numbers are byte-visible, so they must be allocated, not elided.
///
/// Outside this leaf class c2's allocator is demonstrably richer (it reuses dead
/// registers, and it schedules — numbering order is not emission order), so this
/// function accepts exactly the characterized shapes and returns
/// `NotImplemented` for the rest.
pub fn compare_leaf_text(
    cmp: &c2_il::CompareLeaf,
    mode: OptMode,
) -> Result<Vec<u8>, BackendError> {
    use c2_il::Rel;
    // The relational spines below are `/Ox` shapes. `/O1` reallocates them — 14 of
    // `w6_rel_k`'s 19 leaves differ in their register fields (never in an opcode) —
    // and unlike the chain allocator the rule has not been enumerated, so this
    // refuses rather than emitting `/Ox` registers. `docs/OPT_MODE.md` §4.1.
    // `/O1` emits the SAME spines — same opcodes, operand order, immediates and
    // schedule — and reallocates only the temporaries: a temp whose defining
    // instruction makes the last use of the value in r11 is written to r11 instead
    // of taking a fresh descending number. 34 of the 108 matrix cells are therefore
    // byte-identical and the other 74 differ only in register fields.
    // `docs/CODEGEN_W6_O1.md` has the full side-by-side table; each arm below names
    // its own substitution, because which temps can collapse depends on what is
    // still live at that point in that spine.
    let o1 = mode == OptMode::O1;
    let mut t: Vec<u8> = Vec::with_capacity(28);
    let a = RET_REG; // the compared formal is the first argument, r3

    if cmp.k == 0 {
        // ---- mandatory `k == 0` folds (W6 doc §4.6) ----
        match (cmp.rel, cmp.signed) {
            // `a == 0` — same bytes signed and unsigned.
            (Rel::Eq, _) => {
                t.extend_from_slice(&encode_cntlzw(11, a));
                t.extend_from_slice(&encode_rlwinm(RET_REG, 11, 27, 31, 31));
            }
            // `a != 0` — same bytes signed and unsigned. `~(x-1) == -x`, so the
            // register terms cancel and r3 is exactly the carry.
            (Rel::Ne, _) => {
                t.extend_from_slice(&encode_addic(11, a, -1));
                t.extend_from_slice(&encode_subfe(RET_REG, 11, a));
            }
            // signed `a > 0` → (-a) & ~a, sign bit.
            (Rel::Gt, true) => {
                // /O1: the `andc` is the last use of the `neg` result, so it takes
                // r11. Ten of the twelve zero folds are mode-identical; this and
                // `<=` are the two that are not, for exactly that reason.
                let d = if o1 { 11 } else { 10 };
                t.extend_from_slice(&encode_neg(11, a));
                t.extend_from_slice(&encode_andc(d, 11, a));
                t.extend_from_slice(&encode_srwi31(RET_REG, d));
            }
            // unsigned `a > 0` is exactly `a != 0`.
            (Rel::Gt, false) => {
                t.extend_from_slice(&encode_addic(11, a, -1));
                t.extend_from_slice(&encode_subfe(RET_REG, 11, a));
            }
            // signed `a < 0` is just the sign bit.
            (Rel::Lt, true) => t.extend_from_slice(&encode_srwi31(RET_REG, a)),
            // unsigned `a < 0` is constant false.
            (Rel::Lt, false) => t.extend_from_slice(&encode_addi(RET_REG, 0, 0)),
            // signed `a <= 0` → a | ~(-a), sign bit.
            (Rel::Le, true) => {
                // /O1: as for `>` above — the `orc` consumes the dying `neg`.
                let d = if o1 { 11 } else { 10 };
                t.extend_from_slice(&encode_neg(11, a));
                t.extend_from_slice(&encode_orc(d, a, 11));
                t.extend_from_slice(&encode_srwi31(RET_REG, d));
            }
            // unsigned `a <= 0` is exactly `a == 0`.
            (Rel::Le, false) => {
                t.extend_from_slice(&encode_cntlzw(11, a));
                t.extend_from_slice(&encode_rlwinm(RET_REG, 11, 27, 31, 31));
            }
            // signed `a >= 0` → !sign.
            (Rel::Ge, true) => {
                t.extend_from_slice(&encode_srwi31(11, a));
                t.extend_from_slice(&encode_xori(RET_REG, 11, 1));
            }
            // unsigned `a >= 0` is constant true.
            (Rel::Ge, false) => t.extend_from_slice(&encode_addi(RET_REG, 0, 1)),
        }
        t.extend_from_slice(&encode_blr());
        return Ok(t);
    }

    // ---- general spines, non-zero literal ----
    let k16 = i16::try_from(cmp.k).map_err(|_| {
        out_of_class(
            "comparison against a wide literal needs lis+ori materialization and \
             the extra temp slot it consumes; not characterized",
        )
    })?;
    // Only the `==`/`!=` spines form `a - k` as `addi r11,a,-k`, so only they need
    // `-k` to fit the immediate — and at `k == i16::MIN` it does not, because
    // negating it overflows. The port emitted a wrong immediate there.
    //
    // Scoped to those two relations deliberately: `<`, `<=`, `>` and `>=` reach
    // spines that never negate `k` and are correct at the boundary. `w6_rel_k.cpp`
    // tests `a <= -32768` and passes, which is exactly why the bug survived — that
    // fixture probes every relation, and both i16 boundaries, but never a
    // vulnerable relation *at* a boundary. A generated sweep over the cross product
    // found it at once.
    let negatable = k16.checked_neg().is_some();
    let needs_negation = matches!(cmp.rel, Rel::Eq | Rel::Ne);
    // **Two different immediate-eligibility predicates are in play, and they are
    // not interchangeable.** The carry spines (`<`, `<=`, `>`, `>=`) gate on raw
    // SIMM16 encodability, so `a > 4294967291u` is a legitimate
    // `subfic r11,r3,-5`. The `==`/`!=` difference spines gate on the literal's
    // **unsigned value** lying in `[0, 32767]`; against a large unsigned c2
    // materializes the constant and subtracts instead, one instruction more.
    //
    // Sharing one predicate was a live wrong-bytes emit in **both** modes:
    // `int f(unsigned a){return a == 4294967295u;}` and its `!=`, `-5` and
    // `4294967291u` siblings each came out 4 bytes short of the reference
    // (divergence at obj offset 8). Four of the 108 cells of the comparison
    // matrix, and none of them reachable from `w6_rel_k.cpp` or from
    // `scripts/expr_sweep.sh`, whose unsigned literals are all small — found only
    // by enumerating the matrix `docs/CODEGEN_W6_O1.md` tabulates.
    //
    // Refused rather than lowered: the materialize-and-subtract form is the wide
    // -literal path, which is uncharacterized for its own reasons (and where `/Ox`
    // does not even start allocating at r11 — see that doc's asymmetry list).
    if needs_negation && !cmp.signed && cmp.k < 0 {
        return Err(out_of_class(
            "`==`/`!=` against an unsigned literal above 32767: the difference \
             spine's `addi a,-k` is only used when the literal's UNSIGNED value \
             fits the immediate, and c2 materializes the constant instead; the \
             carry spines' raw-SIMM16 rule does not apply here",
        ));
    }
    if needs_negation && !negatable {
        return Err(out_of_class(
            "`==`/`!=` against i16::MIN: the difference spine needs `addi a,-k`, and \
             -(-32768) does not fit the immediate; out of class",
        ));
    }

    match (cmp.rel, cmp.signed) {
        // `a == k` → difference, then "is it zero".
        (Rel::Eq, _) => {
            // /O1: the `cntlzw` is the difference's last use, so it lands in r11.
            // Both words are [`eq_zero_of_difference_in_r11`]'s, shared with the
            // two-call comparator, whose difference is a `subf` instead.
            t.extend_from_slice(&encode_addi(11, a, -k16));
            t.extend_from_slice(&eq_zero_of_difference_in_r11(mode));
        }
        // `a != k` → the `!= 0` spine applied to the difference.
        (Rel::Ne, _) => {
            t.extend_from_slice(&encode_addi(11, a, -k16));
            t.extend_from_slice(&encode_addic(10, 11, -1));
            t.extend_from_slice(&encode_subfe(RET_REG, 10, 11));
        }
        // unsigned `a > k`: CA of `k - a` is `a <= k`, so the answer is !CA.
        // `subfe r9,r10,r10` reads r10, which is never defined — the register
        // terms cancel so the value is a don't-care, but the register NUMBER is
        // byte-visible and must be reproduced.
        (Rel::Gt, false) => {
            // /O1 names the don't-care `subfe` source r11 as well as its dest, so
            // unlike /Ox it reads a *defined* (if dead) register here.
            let (d, src) = if o1 { (11, 11) } else { (9, 10) };
            t.extend_from_slice(&encode_subfic(11, a, k16));
            t.extend_from_slice(&encode_subfe(d, src, src));
            t.extend_from_slice(&encode_clrlwi31(RET_REG, d));
        }
        // signed `a > k`: the 5-instruction spine. p = a (the greater side),
        // q = k. The final clrlwi exists solely to kill the `2` case.
        (Rel::Gt, true) => {
            // /O1: the `subfc` dest stays fresh (r11 is still live for the `eqv`),
            // but the `eqv` is r11's last use and every temp from there on collapses
            // onto it.
            let (e, f, g) = if o1 { (11, 11, 11) } else { (9, 8, 7) };
            t.extend_from_slice(&encode_addi(11, 0, k16)); // li r11,k
            t.extend_from_slice(&encode_subfc(10, a, 11)); // r10 dead; CA is the point
            t.extend_from_slice(&encode_eqv(e, a, 11));
            t.extend_from_slice(&encode_srwi31(f, e));
            t.extend_from_slice(&encode_addze(g, f));
            t.extend_from_slice(&encode_clrlwi31(RET_REG, g));
        }
        // signed `a < k`: the signed `>` spine with the two operand roles
        // swapped, and *only* that — the register numbers, the instruction count
        // and the order are all identical. Both differing words are the ones that
        // read `a` and `r11`: `subfc r10,r11,r3` (not `r3,r11`) and
        // `eqv r9,r11,r3` (not `r3,r11`). `eqv` is commutative, so the swap is
        // invisible in the *value* and visible only in the bytes.
        (Rel::Lt, true) => {
            // /O1: same collapse as signed `>`; only the two swapped operand
            // roles distinguish this spine from it.
            let (e, f, g) = if o1 { (11, 11, 11) } else { (9, 8, 7) };
            t.extend_from_slice(&encode_addi(11, 0, k16)); // li r11,k
            t.extend_from_slice(&encode_subfc(10, 11, a)); // r10 dead; CA is the point
            t.extend_from_slice(&encode_eqv(e, 11, a));
            t.extend_from_slice(&encode_srwi31(f, e));
            t.extend_from_slice(&encode_addze(g, f));
            t.extend_from_slice(&encode_clrlwi31(RET_REG, g));
        }
        // unsigned `a < k`. Unlike unsigned `>`, the literal cannot ride in the
        // `subfic` immediate: the borrow wanted here is the one out of `a - k`,
        // and `subfic` only computes `SIMM - rA`. So `k` is materialized and the
        // spine is four instructions rather than three — which shifts every
        // later register down one (`subfe r8,r9,r9`, not `r9,r10,r10`).
        (Rel::Lt, false) => {
            // /O1: here the `subfc` IS r11's last use (no `eqv` follows), so its
            // dead dest collapses onto r11 — the opposite of the signed spines
            // above, and the clearest evidence that the rule is about consumption
            // rather than about the instruction's kind.
            let (c, d, src) = if o1 { (11, 11, 11) } else { (10, 8, 9) };
            t.extend_from_slice(&encode_addi(11, 0, k16)); // li r11,k
            t.extend_from_slice(&encode_subfc(c, 11, a)); // dead; CA is the point
            t.extend_from_slice(&encode_subfe(d, src, src)); // terms cancel
            t.extend_from_slice(&encode_clrlwi31(RET_REG, d));
        }
        // signed `a >= k`. Two sign terms plus the unsigned borrow, summed by one
        // `adde`: `srawi` broadcasts the sign of the *left* side of the `>=` as
        // 0/−1, `rlwinm ...,1,31,31` takes the sign of the *right* side as 0/1,
        // and `subfc` contributes CA = unsigned(left) >= unsigned(right).
        // The two shifts are emitted in **source** order (`a` before `k`), so
        // they take r10 and r9 in that order — which is why `<=` below, whose
        // left side is the literal, emits them the other way round.
        (Rel::Ge, true) => {
            // /O1: only the `subfc` moves — it is r11's last use, and the two sign
            // temps must both stay live for the `adde`.
            let d = if o1 { 11 } else { 8 };
            t.extend_from_slice(&encode_addi(11, 0, k16)); // li r11,k
            t.extend_from_slice(&encode_srawi(10, a, 31)); // sign(a) as 0/-1
            t.extend_from_slice(&encode_srwi31(9, 11)); // sign(k) as 0/1
            t.extend_from_slice(&encode_subfc(d, 11, a)); // dead; CA is the point
            t.extend_from_slice(&encode_adde(RET_REG, 9, 10));
        }
        // signed `a <= k` is `k >= a`, so the roles invert: the 0/1 shift now
        // applies to `a` and the 0/−1 one to `k`. Emission still follows source
        // order, so `rlwinm` (on `a`) comes first and takes r10.
        (Rel::Le, true) => {
            // /O1: as for `>=` — only the `subfc` dest collapses.
            let d = if o1 { 11 } else { 8 };
            t.extend_from_slice(&encode_addi(11, 0, k16)); // li r11,k
            t.extend_from_slice(&encode_srwi31(10, a)); // sign(a) as 0/1
            t.extend_from_slice(&encode_srawi(9, 11, 31)); // sign(k) as 0/-1
            t.extend_from_slice(&encode_subfc(d, a, 11)); // dead; CA is the point
            t.extend_from_slice(&encode_adde(RET_REG, 10, 9));
        }
        // unsigned `a >= k`: CA out of `a - k` *is* the answer, so all that is
        // left is to materialize it. `subfze rD,rA` computes `~rA + CA`, so
        // against a preloaded −1 it yields CA alone. Note `subfc` writes its
        // (dead) difference back over r11 rather than taking a fresh register.
        (Rel::Ge, false) => {
            t.extend_from_slice(&encode_addi(11, 0, k16)); // li r11,k
            t.extend_from_slice(&encode_addi(10, 0, -1)); // li r10,-1
            t.extend_from_slice(&encode_subfc(11, 11, a)); // r11 reused; CA is the point
            t.extend_from_slice(&encode_subfze(RET_REG, 10));
        }
        // unsigned `a <= k` is the one shape where the literal *can* ride in the
        // `subfic` immediate — the borrow wanted is the one out of `k - a`. So no
        // `li r11,k`, three instructions, and the −1 is emitted **first** even
        // though it takes the lower register number.
        (Rel::Le, false) => {
            t.extend_from_slice(&encode_addi(10, 0, -1)); // li r10,-1
            t.extend_from_slice(&encode_subfic(11, a, k16)); // r11 dead; CA is the point
            t.extend_from_slice(&encode_subfze(RET_REG, 10));
        }
    }
    t.extend_from_slice(&encode_blr());
    Ok(t)
}

#[cfg(test)]
mod tests {
    // The single `mod tests` this was split out of opened with
    // `use super::*;`; the glob keeps that reach.
    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
    use crate::codegen::*;
    #[allow(unused_imports)]
    use c2_il::{IlFunction, IlOp};
    #[allow(unused_imports)]
    use crate::codegen::testutil::*;
    // ---- W6 comparison spines (bytes from live captures) --------------------

    fn cmp(rel: c2_il::Rel, signed: bool, k: i32) -> Vec<u8> {
        compare_leaf_text(&c2_il::CompareLeaf { param: 0xE309, rel, signed, k }, OptMode::Ox).unwrap()
    }

    #[test]
    fn compare_zero_folds_match_the_reference() {
        use c2_il::Rel;
        // `x != 0` (unsigned) — 2 instructions, the carry trick.
        assert_eq!(
            cmp(Rel::Ne, false, 0),
            vec![0x31, 0x63, 0xFF, 0xFF, 0x7C, 0x6B, 0x19, 0x10, 0x4E, 0x80, 0x00, 0x20]
        );
        // signed `x > 0` — a FOLD, not the general 5-instruction spine.
        assert_eq!(
            cmp(Rel::Gt, true, 0),
            vec![
                0x7D, 0x63, 0x00, 0xD0, // neg r11,r3
                0x7D, 0x6A, 0x18, 0x78, // andc r10,r11,r3
                0x55, 0x43, 0x0F, 0xFE, // srwi r3,r10,31
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
        // unsigned `x < 0` folds to constant false; `x >= 0` to constant true.
        assert_eq!(cmp(Rel::Lt, false, 0)[..4], [0x38, 0x60, 0x00, 0x00]);
        assert_eq!(cmp(Rel::Ge, false, 0)[..4], [0x38, 0x60, 0x00, 0x01]);
    }

    #[test]
    fn compare_general_spines_match_the_reference() {
        use c2_il::Rel;
        // `x == 1` (3 instructions).
        assert_eq!(
            cmp(Rel::Eq, false, 1),
            vec![
                0x39, 0x63, 0xFF, 0xFF, // addi r11,r3,-1
                0x7D, 0x6A, 0x00, 0x34, // cntlzw r10,r11
                0x55, 0x43, 0xDF, 0xFE, // rlwinm r3,r10,27,31,31
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
        // `x != 1` (3 instructions).
        assert_eq!(
            cmp(Rel::Ne, false, 1),
            vec![
                0x39, 0x63, 0xFF, 0xFF, // addi r11,r3,-1
                0x31, 0x4B, 0xFF, 0xFF, // addic r10,r11,-1
                0x7C, 0x6A, 0x59, 0x10, // subfe r3,r10,r11
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
        // unsigned `x > 7` — note the `subfe r9,r10,r10` don't-care SOURCE r10,
        // which is never defined but is byte-visible and must be reproduced.
        assert_eq!(
            cmp(Rel::Gt, false, 7),
            vec![
                0x21, 0x63, 0x00, 0x07, // subfic r11,r3,7
                0x7D, 0x2A, 0x51, 0x10, // subfe r9,r10,r10
                0x55, 0x23, 0x07, 0xFE, // clrlwi r3,r9,31
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
        // signed `x > 7` — the 6-word spine.
        assert_eq!(
            cmp(Rel::Gt, true, 7),
            vec![
                0x39, 0x60, 0x00, 0x07, // li r11,7
                0x7D, 0x43, 0x58, 0x10, // subfc r10,r3,r11 (r10 dead; CA is the point)
                0x7C, 0x69, 0x5A, 0x38, // eqv r9,r3,r11
                0x55, 0x28, 0x0F, 0xFE, // srwi r8,r9,31
                0x7C, 0xE8, 0x01, 0x94, // addze r7,r8
                0x54, 0xE3, 0x07, 0xFE, // clrlwi r3,r7,31
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
    }

    #[test]
    fn compare_lt_ge_le_against_a_nonzero_literal_match_the_reference() {
        use c2_il::Rel;
        // All six captured from `int f(int a){ return a <rel> 5; }` (and the
        // `unsigned` overloads) against the live toolchain.

        // signed `a < 5` — the signed `>` spine with the two operands that read
        // `a`/`r11` swapped, and nothing else changed. `eqv` is commutative, so
        // the swap is invisible in the value and visible only here.
        assert_eq!(
            cmp(Rel::Lt, true, 5),
            vec![
                0x39, 0x60, 0x00, 0x05, // li r11,5
                0x7D, 0x4B, 0x18, 0x10, // subfc r10,r11,r3
                0x7D, 0x69, 0x1A, 0x38, // eqv r9,r11,r3
                0x55, 0x28, 0x0F, 0xFE, // srwi r8,r9,31
                0x7C, 0xE8, 0x01, 0x94, // addze r7,r8
                0x54, 0xE3, 0x07, 0xFE, // clrlwi r3,r7,31
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
        // unsigned `a < 5` — four words, not the three of unsigned `>`: the
        // literal cannot ride in a `subfic` immediate here, and materializing it
        // shifts the dead `subfe` down to r8/r9 (from r9/r10).
        assert_eq!(
            cmp(Rel::Lt, false, 5),
            vec![
                0x39, 0x60, 0x00, 0x05, // li r11,5
                0x7D, 0x4B, 0x18, 0x10, // subfc r10,r11,r3
                0x7D, 0x09, 0x49, 0x10, // subfe r8,r9,r9
                0x55, 0x03, 0x07, 0xFE, // clrlwi r3,r8,31
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
        // signed `a >= 5` — `srawi` (0/-1) on the left operand, `rlwinm …,1,31,31`
        // (0/1) on the right, plus CA, summed by one `adde`.
        assert_eq!(
            cmp(Rel::Ge, true, 5),
            vec![
                0x39, 0x60, 0x00, 0x05, // li r11,5
                0x7C, 0x6A, 0xFE, 0x70, // srawi r10,r3,31
                0x55, 0x69, 0x0F, 0xFE, // srwi r9,r11,31
                0x7D, 0x0B, 0x18, 0x10, // subfc r8,r11,r3
                0x7C, 0x69, 0x51, 0x14, // adde r3,r9,r10
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
        // signed `a <= 5` is `5 >= a`, so the two shifts swap which operand they
        // apply to — and, because emission follows source order, also swap
        // positions. Reusing the `>=` order here would be wrong bytes.
        assert_eq!(
            cmp(Rel::Le, true, 5),
            vec![
                0x39, 0x60, 0x00, 0x05, // li r11,5
                0x54, 0x6A, 0x0F, 0xFE, // srwi r10,r3,31
                0x7D, 0x69, 0xFE, 0x70, // srawi r9,r11,31
                0x7D, 0x03, 0x58, 0x10, // subfc r8,r3,r11
                0x7C, 0x6A, 0x49, 0x14, // adde r3,r10,r9
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
        // unsigned `a >= 5` — CA out of `a - 5` *is* the answer; `subfze` against
        // a preloaded -1 materializes it. `subfc` writes its dead difference back
        // over r11 instead of taking a fresh register.
        assert_eq!(
            cmp(Rel::Ge, false, 5),
            vec![
                0x39, 0x60, 0x00, 0x05, // li r11,5
                0x39, 0x40, 0xFF, 0xFF, // li r10,-1
                0x7D, 0x6B, 0x18, 0x10, // subfc r11,r11,r3
                0x7C, 0x6A, 0x01, 0x90, // subfze r3,r10
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
        // unsigned `a <= 5` — the only shape whose literal rides in the `subfic`
        // immediate, so three words; and `li r10,-1` is emitted BEFORE the
        // `subfic` even though it takes the lower register number.
        assert_eq!(
            cmp(Rel::Le, false, 5),
            vec![
                0x39, 0x40, 0xFF, 0xFF, // li r10,-1
                0x21, 0x63, 0x00, 0x05, // subfic r11,r3,5
                0x7C, 0x6A, 0x01, 0x90, // subfze r3,r10
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
    }

    #[test]
    fn compare_uncharacterized_relations_fail_closed() {
        use c2_il::Rel;
        // A wide literal needs lis+ori and the extra temp slot it consumes.
        assert!(matches!(
            compare_leaf_text(&c2_il::CompareLeaf {
                param: 0xE309,
                rel: Rel::Gt,
                signed: false,
                k: 70000,
            }, OptMode::Ox),
            Err(BackendError::NotImplemented(_))
        ));
    }

}
