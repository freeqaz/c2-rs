//! **W8 — the two-arm conditional tail call**, the CFG step's minimal instance.
//!
//! This is the first production in the crate that *accepts* a `38` conditional
//! branch. Everything the file below decodes was already readable by the
//! decode-only scanner in [`super::control_flow`]; what is new is that the
//! decode now reaches an emitter, and `docs/IL_STMT_GRAMMAR.md` §14.2's rule
//! applies with full force: **decoding a production is not licence to emit it.**
//! The class is drawn as narrowly as the bytes allow and every clause below says
//! which measurement draws it.
//!
//! ## The shape
//!
//! ```cpp
//!   void MemFree(void *v1, void *v2, unsigned long ul) {
//!       if (v1 == nullptr) { XMemFree(v2, ul); return; }
//!       RtlFreeHeap(v1, 0, v2);
//!   }
//! ```
//!
//! ```text
//!   B9 <formal> <TYPE>          the compared value
//!   33 <TYPE> <k>               the literal it is compared against
//!   <rel>                       1F..24
//!   38 <L>                      brFALSE -> the ELSE entry
//!     53 …                        the then-clause's own scopes
//!     <call statement>            a tail call
//!     3A <EPI>                    `return;`
//!     54 … 54 …                   close back to the if scope
//!   29 <L>                      the ELSE entry
//!   54 <d>                      close the if scope
//!   <call statement>            the second tail call
//!   3A <EPI>  54 <d-1>  29 <EPI>  <fn tail>
//! ```
//!
//! ## Why this shape and not "an `if`"
//!
//! `docs/CFG_SHAPE.md` §3.5 measures that a `cflow-if-1` body usually emits **no
//! branch at all**: six of seven leaf probes fold to a branchless arithmetic
//! select (band 1) or a `bclr` conditional return (band 2), and **both** real
//! `cflow-if-1` functions in `src/system/utl/Pool.cpp` are band-2 folds. §3.5
//! then declines the band-1/band-2 decision outright — "every fitted rule I
//! could state is consistent with the eighteen rows above and none of them is
//! *tested* by them".
//!
//! So this class is drawn to sit **inside band 3 by construction**, not by a
//! cost model. Band 2 is reached "when one successor **is** the function's
//! epilogue"; band 1 needs both arms to be constants. A body whose *both* arms
//! end in a tail call to a different external can be neither — every path leaves
//! through a `b <callee>` and the epilogue is never materialized (§4.2 item 7).
//! That is the whole content of requiring two call arms rather than one: it is a
//! **band predicate spelled as a syntactic one**, which is what §6.2 item G asks
//! for.
//!
//! ## What it refuses, and why each refusal is a measurement
//!
//! * **An arm that falls through to the epilogue** — band 2, `bclr`, not a
//!   branch (§3.5).
//! * **Both arms calling the SAME callee** — `docs/CFG_SHAPE.md` §3.4.1: c2
//!   **tail-merges** two identical `bl` sites, empties the then-block and
//!   *inverts* the layout, so block order stops being IL statement order. Board
//!   #193. A body whose arms end in the same call is outside anything §3.4
//!   specifies.
//! * **A computed argument** — the arms marshal formals and small literals
//!   only, exactly the vocabulary [`super::calls::tail_call_shape`]'s slot list
//!   already carries.
//! * **More than one formal needing the scratch register** — the park is
//!   measured at r11 and only at r11 (§4.2 item 8). A second one would descend
//!   to r10 on the register model `docs/CODEGEN_W6_COMPARE.md` §6 records as
//!   *uncharacterized*.

use super::super::expr::{eat_fn_tail, eat_scopes, BODY_SCOPE_DEPTH};
use super::super::{BodyShape, CondArmShape, CondTailPairShape, IlOp};
use super::calls::{eat_call_args, eat_call_head, LI_IMM_MAX, LI_IMM_MIN};
use super::params::parse_params;
use crate::func::readers::{
    eat, eat_byte, eat_opt_stmt_marker, is_ptr_to_4, read_token_var, read_type, read_varint,
    INT_TYPE, LONG_TYPE, UINT_TYPE, ULONG_TYPE,
};
use crate::func::{Rel, SlotArg};

/// Read the operand TYPE of the compared value and say whether the compare is
/// **signed**, i.e. `cmpwi` rather than `cmplwi`.
///
/// Only the five spellings with a witness are admitted, deliberately. The
/// generic `is_int4_type` predicate that `eat_int_like` uses would let an
/// `enum`, a `typedef` or a `const int` through — and it cannot say whether the
/// underlying type is signed, which here is the difference between `2f……` and
/// `2b……` in the emitted word. `docs/CFG_SHAPE.md` §3.2: "the signedness comes
/// from the shared operand type triple at the comparison".
pub(crate) fn eat_cmp_operand_type(seg: &[u8], p: &mut usize) -> Option<bool> {
    if eat(seg, p, &INT_TYPE) || eat(seg, p, &LONG_TYPE) {
        return Some(true);
    }
    if eat(seg, p, &UINT_TYPE) || eat(seg, p, &ULONG_TYPE) {
        return Some(false);
    }
    // A pointer null-check is an **unsigned** compare — `?MemFree` and both
    // `Pool.cpp` functions emit `cmplwi` (§3.2's witness row).
    let (tag, kind, _, w) = read_type(seg, *p)?;
    if is_ptr_to_4(tag, kind) {
        *p += w;
        return Some(false);
    }
    None
}

/// One arm: a single call statement that transfers control out of the function.
///
/// Returns the callee token and the argument slot list, with the cursor left
/// immediately after the statement's value/void terminator and before its `3A`.
fn eat_arm_call(seg: &[u8], p: &mut usize, params: &[u32]) -> Option<(u32, Vec<SlotArg>)> {
    let (callee_tok, ret) = eat_call_head(seg, p).ok()?;
    let args = eat_call_args(seg, p).ok()?;
    // The terminator. Two spellings, both witnessed in `xboxmem.cpp`:
    //   `4B`                     a VOID statement call    (`?MemFree`)
    //   `[2C <T> 00] 41 <T>`     the call's value returned (`?MemAlloc`, `?MemSize`)
    // Nothing between them changes a single emitted byte — the value is already
    // in r3 and the `2C` is an int-width conversion — so they share this arm
    // rather than splitting the shape.
    if eat_byte(seg, p, 0x4B) {
        // A discarded `float`/`double` result obliges the TU to declare
        // `_fltused`, whose placement is not modeled. The same gate every other
        // discarding site asks.
        ret.discarded(seg, *p).ok()?;
    } else {
        if eat_byte(seg, p, 0x2C) {
            // `2C <T> 00` — the result's width conversion.
            let (_, _, _, w) = read_type(seg, *p)?;
            *p += w;
            if !eat_byte(seg, p, 0x00) {
                return None;
            }
        }
        if !eat_byte(seg, p, 0x41) {
            return None;
        }
        let (_, _, _, w) = read_type(seg, *p)?;
        *p += w;
    }
    // The slot list, under exactly the vocabulary the existing tail call
    // carries. Arguments arrive in reverse source order, so slot `i` is
    // `args[len-1-i]` — the same reversal every argument position in
    // [`super::calls`] uses.
    let mut slots = Vec::with_capacity(args.len());
    for slot in 0..args.len() {
        slots.push(match args[args.len() - 1 - slot].as_slice() {
            [IlOp::Load(t)] => SlotArg::Formal(params.iter().position(|q| q == t)?),
            [IlOp::Lit(k)] if (LI_IMM_MIN..=LI_IMM_MAX).contains(k) => SlotArg::Lit(*k),
            // A computed argument, a local, a global, a data symbol's address.
            // Each would need its own instruction inside an arm, and the
            // interaction between that instruction and the entry block's parks
            // has no capture.
            _ => return None,
        });
    }
    Some((callee_tok, slots))
}

/// `3A <EPI>` — the arm's `return`, which in this class is always folded into
/// the tail call. Returns the epilogue label token.
fn eat_arm_return(seg: &[u8], p: &mut usize) -> Option<u32> {
    eat_opt_stmt_marker(seg, p);
    if !eat_byte(seg, p, 0x3A) {
        return None;
    }
    let (tok, w) = read_token_var(seg, *p)?;
    *p += w;
    Some(tok)
}

/// Close scopes from `depth` down to (but not including) `target`, each
/// optionally preceded by its own line marker.
fn eat_closes_to(seg: &[u8], p: &mut usize, depth: &mut usize, target: usize) -> Option<()> {
    while *depth > target {
        eat_opt_stmt_marker(seg, p);
        if !eat(seg, p, &[0x54, *depth as u8]) {
            return None;
        }
        *depth -= 1;
    }
    Some(())
}

/// Try to parse the **two-arm conditional tail call**.
///
/// Non-committal in the house style: works on a copy of the cursor and returns
/// `None` with no side effects, so a body that is not this production keeps its
/// own first-blocker census key.
///
/// `depth` is the scope depth at `start`, i.e. after `parse_segment_shape` has
/// eaten the body's `53` and any further scopes — for `?MemFree` that is 3, and
/// the `54 03` that closes the `if` statement's own scope is what confirms it.
pub(crate) fn try_parse_cond_tail_pair(
    seg: &[u8],
    start: usize,
    lo: usize,
    depth: usize,
) -> Option<BodyShape> {
    let mut p = start;
    let params = parse_params(seg, lo).ok()?;
    // The formals must all be register-resident for a slot index to mean a
    // register. `parse_segment_shape` has already asserted one register each.
    if params.is_empty() {
        return None;
    }

    // ---- the condition -----------------------------------------------------
    if !eat_byte(seg, &mut p, 0xB9) {
        return None;
    }
    let (cmp_tok, w) = read_token_var(seg, p)?;
    p += w;
    let signed = eat_cmp_operand_type(seg, &mut p)?;
    let cmp_param = params.iter().position(|q| *q == cmp_tok)?;

    // The literal, spelled in the SAME type as the loaded operand. Requiring the
    // restatement is the same cheap assertion the comparison leaf makes: c1xx
    // always inserts a conversion first, so a mismatch has never been observed.
    if !eat_byte(seg, &mut p, 0x33) {
        return None;
    }
    let mut q = p;
    if eat_cmp_operand_type(seg, &mut q)? != signed {
        return None;
    }
    p = q;
    let k = read_varint(seg, &mut p)?;
    // `cmpwi`/`cmplwi` take a 16-bit immediate. A wider literal is `lis`+`ori`
    // into a scratch and then the register-register form, which has no capture.
    if !(-0x8000..=0xFFFF).contains(&k) {
        return None;
    }
    if signed && !(-0x8000..=0x7FFF).contains(&k) {
        return None;
    }
    if !signed && !(0..=0xFFFF).contains(&k) {
        return None;
    }

    let rel = Rel::from_opcode(*seg.get(p)?)?;
    p += 1;

    // ---- `38 <L>` — brFALSE to the else entry ------------------------------
    if !eat_byte(seg, &mut p, 0x38) {
        return None;
    }
    let (else_label, w) = read_token_var(seg, p)?;
    p += w;

    // ---- the then-clause ---------------------------------------------------
    let mut d = depth;
    eat_scopes(seg, &mut p, &mut d).ok()?;
    // The then-clause must open at least one scope of its own; if it did not,
    // the `38` had no block and this is some other production.
    if d <= depth {
        return None;
    }
    let (then_callee, then_slots) = eat_arm_call(seg, &mut p, &params)?;
    let epi = eat_arm_return(seg, &mut p)?;
    eat_closes_to(seg, &mut p, &mut d, depth)?;

    // ---- `29 <L>` — the else entry, then the if scope's own close ----------
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x29) {
        return None;
    }
    let (lbl, w) = read_token_var(seg, p)?;
    p += w;
    // A `29` that is not the one the `38` named is a different control-flow
    // graph — a nested `if`, a `goto` target, an `||` join. Out of class.
    if lbl != else_label {
        return None;
    }
    // The `if` statement's own scope closes here, leaving the body scope.
    let mut d = depth;
    eat_closes_to(seg, &mut p, &mut d, BODY_SCOPE_DEPTH)?;
    if d != BODY_SCOPE_DEPTH {
        return None;
    }

    // ---- the else clause ---------------------------------------------------
    let (else_callee, else_slots) = eat_arm_call(seg, &mut p, &params)?;
    if eat_arm_return(seg, &mut p)? != epi {
        return None;
    }
    // The body scope closes and the epilogue label is defined. This is
    // `eat_return_head`'s tail, restated rather than called because that helper
    // consumes the `3A` too and this shape has already eaten two of them.
    let mut d = BODY_SCOPE_DEPTH;
    eat_closes_to(seg, &mut p, &mut d, BODY_SCOPE_DEPTH - 1)?;
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x29) {
        return None;
    }
    let (lbl, w) = read_token_var(seg, p)?;
    p += w;
    if lbl != epi {
        return None;
    }
    eat_fn_tail(seg, &mut p).ok()?;

    // ---- the class gates ---------------------------------------------------
    // **Board #193 — the tail-merge.** Two arms ending in the same call is the
    // one measured refutation of "block order is IL statement order": c2 merges
    // the sites, empties the then-block and inverts the layout. Refused by name.
    if then_callee == else_callee {
        return None;
    }
    let pair = CondTailPairShape {
        params,
        cmp_param,
        rel,
        signed,
        k,
        then_arm: CondArmShape { callee_tok: then_callee, slots: then_slots },
        else_arm: CondArmShape { callee_tok: else_callee, slots: else_slots },
    };
    // The register plan is part of the class, not part of the emitter: a body
    // the plan cannot schedule must not census as in-class. `plan` is the ONE
    // locator both sides share (the same discipline `CompareLeaf::out_of_class_ctx`
    // applies to the comparison leaf).
    pair.plan()?;
    Some(BodyShape::CondTailPair(pair))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `?MemFree@NUISPEECH@@YAXPAX0K@Z` from `src/xdk/nuispeech/xboxmem.cpp`,
    /// transcribed from the `.ex` captured at the workload's own flags
    /// (`/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc /I …`, with the dc3 tree
    /// as cwd). The slice is the whole function **segment**, `4F 1F` to `4F 1F`
    /// — the split the port itself consumes — so it carries the per-function
    /// optimization word, the debug region, the `46` formals list and the body.
    ///
    /// Its body is `docs/CFG_SHAPE.md` §4.1's listing, byte for byte.
    const MEMFREE: &[u8] = &[
        0x4f, 0x1f, 0x80, 0x05, 0x00, 0x20, 0x00, 0x4f, 0x20, 0x80, 0xfe, 0x00, 0x4f, 0x33, 0x0d,
        0x66, 0x12, 0x1c, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0b, 0x0b, 0x03, 0x0f, 0x10, 0x18,
        0x01, 0x00, 0x0e, 0x6c, 0x12, 0x38, 0x1d, 0x42, 0x45, 0x0e, 0x06, 0x01, 0x01, 0x01, 0x0d,
        0x08, 0x00, 0x0f, 0x4f, 0x02, 0x20, 0x00, 0x4f, 0x01, 0x0f, 0x53, 0x53, 0x26, 0x57, 0x0f,
        0x46, 0x2d, 0x6e, 0x0f, 0x2d, 0x6d, 0x0f, 0x2d, 0x6c, 0x0f, 0x4c, 0x4f, 0x11, 0x53, 0x4f,
        0x01, 0x10, 0x53, 0xb9, 0x6c, 0x0f, 0x86, 0x43, 0x83, 0x08, 0x33, 0x86, 0x43, 0x83, 0x08,
        0x00, 0x1f, 0x38, 0x71, 0x0f, 0x53, 0x53, 0x4f, 0x01, 0x11, 0x26, 0xda, 0x0e, 0xbd, 0x82,
        0x07, 0x03, 0x00, 0x80, 0x05, 0x10, 0x00, 0x00, 0xb9, 0x6e, 0x0f, 0x86, 0x42, 0x22, 0x55,
        0x86, 0x42, 0x22, 0xb9, 0x6d, 0x0f, 0x86, 0x43, 0x83, 0x08, 0x55, 0x86, 0x43, 0x83, 0x08,
        0x4c, 0x4b, 0x4f, 0x01, 0x12, 0x3a, 0x70, 0x0f, 0x4f, 0x01, 0x13, 0x54, 0x05, 0x4f, 0x01,
        0x14, 0x54, 0x04, 0x29, 0x71, 0x0f, 0x54, 0x03, 0x26, 0xec, 0x09, 0xbd, 0x86, 0x42, 0x75,
        0x00, 0x80, 0x07, 0x10, 0x00, 0x00, 0xb9, 0x6d, 0x0f, 0x86, 0x43, 0x83, 0x08, 0x55, 0x86,
        0x43, 0x83, 0x08, 0x33, 0x86, 0x42, 0x22, 0x00, 0x55, 0x86, 0x42, 0x22, 0xb9, 0x6c, 0x0f,
        0x86, 0x43, 0x83, 0x08, 0x55, 0x86, 0x43, 0x83, 0x08, 0x4c, 0x4b, 0x4f, 0x01, 0x15, 0x3a,
        0x70, 0x0f, 0x54, 0x02, 0x29, 0x70, 0x0f, 0x4f, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// The parser's entry conditions, reproduced: the body's `53` is eaten and
    /// `eat_scopes` runs before the shape recognizer is offered the cursor.
    fn at_body(seg: &[u8]) -> (usize, usize, usize) {
        let lo = crate::func::body_start(seg).expect("LO marker");
        let mut p = crate::func::ops_start(seg, lo);
        assert!(eat_byte(seg, &mut p, 0x53));
        let mut depth = BODY_SCOPE_DEPTH;
        eat_scopes(seg, &mut p, &mut depth).expect("scopes");
        (p, lo, depth)
    }

    #[test]
    fn memfree_parses_as_a_cond_tail_pair() {
        let (p, lo, depth) = at_body(MEMFREE);
        let shape = try_parse_cond_tail_pair(MEMFREE, p, lo, depth).expect("in class");
        let BodyShape::CondTailPair(pair) = shape else {
            panic!("wrong shape")
        };
        // `if (v1 == nullptr)`, v1 the FIRST formal, an UNSIGNED compare against 0.
        assert_eq!(pair.cmp_param, 0);
        assert_eq!(pair.rel, Rel::Eq);
        assert!(!pair.signed, "a pointer null-check is unsigned (CFG_SHAPE §3.2)");
        assert_eq!(pair.k, 0);
        // `XMemFree(v2, ul)` — the second and third formals, in that order.
        assert_eq!(pair.then_arm.slots, vec![SlotArg::Formal(1), SlotArg::Formal(2)]);
        // `RtlFreeHeap(v1, 0, v2)`.
        assert_eq!(
            pair.else_arm.slots,
            vec![SlotArg::Formal(0), SlotArg::Lit(0), SlotArg::Formal(1)]
        );
        assert_ne!(pair.then_arm.callee_tok, pair.else_arm.callee_tok);
    }

    #[test]
    fn the_register_plan_matches_the_reference_obj() {
        let (p, lo, depth) = at_body(MEMFREE);
        let BodyShape::CondTailPair(pair) = try_parse_cond_tail_pair(MEMFREE, p, lo, depth).unwrap()
        else {
            panic!()
        };
        let plan = pair.plan().expect("schedulable");
        use crate::func::CondStep::{Li, Move};
        // `mr r11,r4` — v2 is wanted by BOTH arms at DIFFERENT destinations
        // (r3 in the then-arm, r5 in the else-arm), so it is parked.
        assert_eq!(plan.entry, vec![Move { dst: 11, src: 4 }]);
        // The compare reads v1, untouched in r3.
        assert_eq!(plan.cmp_reg, 3);
        // then: `mr r4,r5 ; mr r3,r11`, descending destination.
        assert_eq!(
            plan.then_steps,
            vec![Move { dst: 4, src: 5 }, Move { dst: 3, src: 11 }]
        );
        // else: `mr r5,r11 ; li r4,0`, descending destination with the literal
        // interleaved rather than grouped.
        assert_eq!(
            plan.else_steps,
            vec![Move { dst: 5, src: 11 }, Li { dst: 4, k: 0 }]
        );
    }

    #[test]
    fn a_body_whose_arms_call_the_same_callee_is_refused() {
        // Board #193: c2 tail-merges the two sites and inverts the layout. Built
        // by rewriting the else arm's callee token to the then arm's.
        let mut seg = MEMFREE.to_vec();
        // `26 ec 09` -> `26 da 0e` (the then arm's callee push).
        let at = seg
            .windows(3)
            .position(|w| w == [0x26, 0xec, 0x09])
            .expect("the else arm's callee push");
        seg[at + 1] = 0xda;
        seg[at + 2] = 0x0e;
        let (p, lo, depth) = at_body(&seg);
        assert!(try_parse_cond_tail_pair(&seg, p, lo, depth).is_none());
    }
}
