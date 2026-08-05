//! The comparison leaf: relation × signedness × literal, including both i16
//! boundaries. See `docs/CODEGEN_W6_COMPARE.md` and `docs/CODEGEN_W6_O1.md`.

use crate::BackendError;
use crate::codegen::encode::{
    encode_adde,
    encode_addi,
    encode_addic,
    encode_addis,
    encode_addze,
    encode_andc,
    encode_blr,
    encode_clrlwi31,
    encode_cntlzw,
    encode_eqv,
    encode_neg,
    encode_mr,
    encode_orc,
    encode_rlwimi,
    encode_rlwinm,
    encode_srawi,
    encode_srwi31,
    encode_subf,
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

/// **WCR — the two-call comparator's spine**: `return a->m() <rel> b->n();`
/// lowered to a 0/1 in r3, with both operands in registers.
///
/// `first` is the callee-saved register holding the **first** call's result;
/// the second call's is still in r3. `lhs_first` says which of those two is the
/// source's *left* operand — c2 orders the calls by the order c1xx numbered
/// their receivers, so the operand roles and the emission order are two
/// independent facts (`docs/rungs/2026-07-31-cmp-two-calls.md`).
///
/// ## The three spines, read off reference objs rather than derived
///
/// `scripts/gt_cmp_rr.py`, all four modes, leaf and framed in one TU. With
/// `p->m()` in r30 and `q->n()` in r3 (`/O1` register numbers):
///
/// ```text
///   ==   sub   r11,r3,r30 ; cntlzw r11,r11 ; rlwinm r3,r11,27,31,31
///   >    subfc r11,r30,r3 ; eqv r10,r30,r3 ; srwi r11,r10,31 ; addze r11,r11
///                         ; clrlwi r3,r11,31
///   > u  subfc r11,r30,r3 ; subfe r11,r11,r11 ; clrlwi r3,r11,31
/// ```
///
/// **`<` is `>` with the two operands exchanged and nothing else** — the same
/// relationship [`compare_leaf_text`]'s `Rel::Lt` arm has to its `Rel::Gt` arm,
/// so it is one spine here rather than two, and the exchange composes with
/// `lhs_first`'s.
///
/// **This is the register-register family `docs/CMP_PRODUCES_A_VALUE.md`
/// reading 4 names, and it is not the leaf spine with the `li` deleted.** The
/// leaf materializes its literal into r11 first, so its temporaries start at
/// r10; here nothing occupies r11, so every `/Ox` temp is **one number higher**
/// (`subfc` into r11, `eqv` into r10, …) and the `/O1` collapse lands
/// differently too — the leaf's `eqv` is r11's last use and takes r11, while
/// here it is the *`srwi`* that does. Both sets are transcribed, neither is
/// computed from the other.
///
/// **The result type does not enter any of this.** Reading 1 of that document
/// records a `bool` result costing two extra words for signed `>=`/`<=` against
/// a non-zero literal; over two call results the same two relations diverge (and
/// are refused in the IL parser for exactly that reason), while `>`, `<`, `==`
/// and `!=` are byte-identical in `int`, `bool` and `unsigned` in all four modes.
pub fn cmp_of_two_call_results(
    cmp: c2_il::SeqCmp,
    lhs_first: bool,
    first: Option<u8>,
    mode: OptMode,
) -> Result<Vec<u8>, BackendError> {
    let first = first.ok_or_else(|| {
        out_of_class("a comparison of two call results needs a callee-saved register for the first")
    })?;
    let (lhs, rhs) = if lhs_first { (first, RET_REG) } else { (RET_REG, first) };
    let o1 = mode == OptMode::O1;
    let mut t: Vec<u8> = Vec::with_capacity(24);
    match cmp {
        // The difference, then "is it zero" — the two words shared with the
        // comparison leaf's `==` arm.
        c2_il::SeqCmp::Eq => {
            t.extend_from_slice(&encode_subf(11, lhs, rhs));
            t.extend_from_slice(&eq_zero_of_difference_in_r11(mode));
        }
        // `a > b` (and `a < b`, which is `b > a`). The `subfc` result is dead —
        // only its carry is read — but its register number is byte-visible.
        c2_il::SeqCmp::Order { greater, signed } => {
            let (a, b) = if greater { (lhs, rhs) } else { (rhs, lhs) };
            t.extend_from_slice(&encode_subfc(11, a, b));
            if signed {
                // /O1 collapses the two temps *after* the `eqv` onto r11; /Ox
                // keeps allocating descending. The `eqv` itself takes r10 in
                // both, which is what makes this not a one-line substitution of
                // the leaf's `(11,11,11)` / `(9,8,7)` triple.
                let (e, f, g) = if o1 { (10, 11, 11) } else { (10, 9, 8) };
                t.extend_from_slice(&encode_eqv(e, a, b));
                t.extend_from_slice(&encode_srwi31(f, e));
                t.extend_from_slice(&encode_addze(g, f));
                t.extend_from_slice(&encode_clrlwi31(RET_REG, g));
            } else {
                // The don't-care `subfe` source, same numbers as the leaf's
                // unsigned `>` arm — there the `subfic` occupies r11, here the
                // `subfc` does, so the descending allocation is in step.
                let (d, src) = if o1 { (11, 11) } else { (9, 10) };
                t.extend_from_slice(&encode_subfe(d, src, src));
                t.extend_from_slice(&encode_clrlwi31(RET_REG, d));
            }
        }
    }
    Ok(t)
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

/// **W43** — `return ((unsigned)(P != 0) << SH) | C;`, six words, no frame.
///
/// ```text
///   addic  r11,r3,-1              the `!= 0` fold, identical to W6's
///   lis    rC,C>>16                 (Rel::Ne, _) arm — signed and unsigned
///   subfe  rS,r11,r3                alike
///   rlwimi rC,rS,SH,0,31-SH
///   mr     r3,rC
///   blr
/// ```
///
/// **The `lis` sits BETWEEN the `addic` and the `subfe`** — the constant is
/// materialized into the gap between the carry producer and its consumer. That
/// is a schedule, and it is the one c2 emits in every cell of the region
/// `c2_il::shift_or_rlwimi` admits.
///
/// **The register is the incumbent `/O1` rule and not a new one.** `docs/
/// CODEGEN_W6_O1.md`: a temp whose defining instruction makes the **last use**
/// of the value in r11 takes r11 rather than a fresh descending number. The
/// `subfe` is the last use of the `addic` result, so `rS = r11` at `/O1`; at
/// `/Ox` the descending counter has spent r11 on the `addic` and r10 on the
/// `lis`, so it hands out r9. Same opcodes, same order, same immediates —
/// register fields only, exactly as the 108-cell W6 matrix behaves.
pub fn cmp_shift_or_text(
    cso: &c2_il::CmpShiftOr,
    mode: OptMode,
) -> Result<Vec<u8>, BackendError> {
    // The one locator the census also runs, so the two cannot disagree about
    // which `(SH, C)` pairs are in class.
    let (mb, me) = c2_il::shift_or_rlwimi(cso.sh, cso.c).ok_or_else(|| {
        out_of_class(
            "a shift-or whose constant or shift is outside the measured rlwimi \
             region (c2 emits slwi+oris or slwi+ori there); out of class",
        )
    })?;
    // `lis rC,hi` is `addis rC,0,hi`. The high half is taken as a raw 16-bit
    // pattern and reinterpreted, because `addis`'s immediate field is signed and
    // a constant such as `0x9abc0000` has bit 15 set.
    let hi = (cso.c >> 16) as u16 as i16;
    let const_reg = 10u8;
    let sub_reg = if mode == OptMode::O1 { 11 } else { 9 };
    let a = RET_REG;
    let mut t: Vec<u8> = Vec::with_capacity(24);
    t.extend_from_slice(&encode_addic(11, a, -1));
    t.extend_from_slice(&encode_addis(const_reg, 0, hi));
    t.extend_from_slice(&encode_subfe(sub_reg, 11, a));
    t.extend_from_slice(&encode_rlwimi(const_reg, sub_reg, cso.sh, mb, me));
    t.extend_from_slice(&encode_mr(RET_REG, const_reg));
    t.extend_from_slice(&encode_blr());
    Ok(t)
}

#[cfg(test)]
mod w43_tests {
    use super::*;

    /// `?GetXAllocAttributes@NUISPEECH@@YAKH@Z`, transcribed from the real obj
    /// this lane compiled at the workload's own flags
    /// (`work/w-tu1/ref/xboxmem.obj`, `.text` #5, 24 B, nrel 0).
    const GETXALLOC_O1: [u32; 6] = [
        0x3163ffff, // addic  r11,r3,-1
        0x3d40249b, // lis    r10,0x249b
        0x7d6b1910, // subfe  r11,r11,r3
        0x516af002, // rlwimi r10,r11,30,0,1
        0x7d435378, // mr     r3,r10
        0x4e800020, // blr
    ];
    /// The same source at `/Ox`, from `work/w-tu1/p/v1.cpp`. **Only the `subfe`
    /// destination and the `rlwimi` source differ** — `docs/CODEGEN_W6_O1.md`'s
    /// incumbent rule, not a new one.
    const GETXALLOC_OX: [u32; 6] =
        [0x3163ffff, 0x3d40249b, 0x7d2b1910, 0x512af002, 0x7d435378, 0x4e800020];

    fn words(v: &[u8]) -> Vec<u32> {
        v.chunks(4).map(|c| u32::from_be_bytes([c[0], c[1], c[2], c[3]])).collect()
    }

    #[test]
    fn w43_emits_getxallocattributes_in_both_modes() {
        let cso = c2_il::CmpShiftOr { param: 0x0f61, signed: true, sh: 30, c: 0x249b_0000 };
        assert_eq!(words(&cmp_shift_or_text(&cso, OptMode::O1).unwrap()), GETXALLOC_O1);
        assert_eq!(words(&cmp_shift_or_text(&cso, OptMode::Ox).unwrap()), GETXALLOC_OX);
    }

    /// **`addis`'s immediate is SIGNED, and the class can never hand it a
    /// negative one.** `SH > msb(C)` with `SH <= 31` forces `msb(C) <= 30`, so
    /// `C`'s bit 31 — which is bit 15 of the half `lis` carries — is always
    /// clear. The `as u16 as i16` in the emitter is therefore a no-op *inside
    /// the class*, and it is kept as a total conversion rather than removed,
    /// because a widening of `shift_or_rlwimi` that admitted `msb(C) == 31`
    /// would otherwise turn a sign flip into wrong bytes silently.
    ///
    /// This test is the check that the invariant holds, not that the conversion
    /// fires: every `C` the predicate admits has a non-negative high half.
    #[test]
    fn w43_the_class_never_hands_addis_a_negative_immediate() {
        let mut admitted = 0;
        for c in [0x4000_0000u32, 0x249b_0000, 0x1000_0000, 0x0800_0000, 0x0003_0000,
                  0x0001_0000, 0x7fff_0000, 0xffff_0000, 0x0002_0000, 0x1234_0000,
                  0x9abc_0000, 0x8000_0000] {
            for sh in 1u8..=31 {
                if c2_il::shift_or_rlwimi(sh, c).is_none() {
                    continue;
                }
                admitted += 1;
                assert!((c >> 16) as i16 >= 0, "c={c:#x} sh={sh} would sign-flip `lis`");
            }
        }
        // A positive count, not a status: an empty loop would pass vacuously.
        assert_eq!(admitted, 57, "the admitted region over these twelve constants");
    }

    /// The selection predicate, at both edges of the region it claims and on
    /// both rows of the 288-cell grid it deliberately does not explain.
    #[test]
    fn w43_the_selection_region_is_exactly_what_the_grid_supports() {
        use c2_il::shift_or_rlwimi as f;
        // Inside: `SH > msb(C)`, low half zero. The mask is always `0..31-SH`.
        assert_eq!(f(30, 0x249b_0000), Some((0, 1)));
        assert_eq!(f(31, 0x249b_0000), Some((0, 0)));
        assert_eq!(f(17, 0x0001_0000), Some((0, 14)));
        // The boundary column itself is OUT — c2 emits `slwi` + `oris` there.
        assert_eq!(f(29, 0x249b_0000), None);
        assert_eq!(f(28, 0x249b_0000), None);
        // A non-zero low half: `lis` alone cannot make the constant.
        assert_eq!(f(30, 0x0000_0004), None);
        assert_eq!(f(20, 0x0000_ffff), None);
        // The two unexplained rows, both excluded rather than explained.
        assert_eq!(f(30, 0x8000_0000), None, "msb 31, so the region is empty here");
        assert_eq!(f(16, 0x0003_0000), None, "c2 crosses at 16; the region starts at 18");
        // Degenerate shifts and a zero constant.
        assert_eq!(f(0, 0x249b_0000), None);
        assert_eq!(f(32, 0x249b_0000), None);
        assert_eq!(f(30, 0), None);
        // The emitter refuses everything the predicate does — one locator, two
        // callers, which is what keeps the census and the gate from disagreeing.
        for (sh, c) in [(29u8, 0x249b_0000u32), (30, 4), (30, 0x8000_0000), (0, 0x1_0000)] {
            let cso = c2_il::CmpShiftOr { param: 1, signed: true, sh, c };
            assert!(cmp_shift_or_text(&cso, OptMode::O1).is_err(), "sh={sh} c={c:#x}");
        }
    }
}
