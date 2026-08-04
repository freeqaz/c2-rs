//! **W11 — guarded EARLY RETURNS ahead of a framed call sequence: the port's
//! first intra-section `b` and its first real label→offset map.**
//!
//! ```cpp
//!   int f(int a, int b) {
//!       if (a != 0) return 5;
//!       if (b != 0) return 11;
//!       v0();
//!       return 0;
//!   }
//! ```
//!
//! ## Why this shape and not another
//!
//! `work/w-conv/PREREG.md` §1 prices all 17 FRONTIER TUs off their own
//! disassembly at the workload's flags. The minimum is **6** independent
//! refusals and the cheapest framed-and-branching one is **9**, so no TU is a
//! target and the rung is chosen by *construct* instead. Ranked that way, over
//! the same 17 objs:
//!
//! | missing mechanism | FRONTIER TUs |
//! |---|---:|
//! | a real label→offset map (≥ 2 transfers, ≥ 1 shared target) | **14** |
//! | the intra-section unconditional `b` (board #191) | **10** |
//!
//! Both are the same rung and the port had emitted neither. Board #191 had been
//! open since w-cfg, and W10 closed the only route that had been tried — the
//! `else` arm — because the *one* source shape producing an intra-section `b`
//! also produces `/Ox`'s join duplication, whose threshold is a c2 cost model
//! bracketed by one cell either side.
//!
//! **The guarded early return is a second route and it is not that shape.** Its
//! `b` targets the **epilogue**, not a join block.
//!
//! ## The two layouts, measured — and it is not a cost model
//!
//! ```text
//!   /O1  (what the dc3 workload compiles)   /Ox and /O2
//!     mflr/stw/stwu   Class A frame           mflr/stw/stwu
//!     cmpwi cr6,r3,0                          cmpwi cr6,r3,0
//!     bt    26,+12                            bt    26,+24
//!     li    r3,5                              li    r3,5
//!     b     +12       <- 48000...             addi/lwz/mtlr/blr  <- epilogue COPIED
//!     bl    ?v0                               bl    ?v0
//!     li    r3,0                              li    r3,0
//!     addi/lwz/mtlr/blr                       addi/lwz/mtlr/blr
//! ```
//!
//! This **is** board row X-b's mode split — `/Ox` and `/O2` tail-duplicate where
//! `/O1` shares behind a `b` — and it refutes `docs/OPT_MODE.md`'s claim, restated
//! in [`crate`]'s consumer `c2_core::codegen::OptMode`, that the modes *"differ in
//! exactly one rule … only a register field"*. They differ in **block
//! structure**.
//!
//! **But there is no threshold here to fit.** W10's declined quantity was *how
//! many bytes a join is worth duplicating*, and it varied with join length. The
//! block duplicated here is the **epilogue**, whose length is a constant of the
//! frame class, and `/Ox` copies it in **every** measured cell: guard counts 1,
//! 2 and 3; six relations; both signednesses; trailing-call counts 1–4;
//! scrutinee at formals 0, 1, 2 and 3; literals `0`, `5`, `11`, `22`, `-1`,
//! `4660`, `32767`. Two measured layouts, one per mode, ≥ 8 witnesses each
//! (`work/w-conv/p/probe1.cpp`, `probe2.cpp`, `probe3.cpp` at `/O1`, `/O2`,
//! `/Ox`).
//!
//! ## The void form is a DIFFERENT branch sense, and that is measured too
//!
//! `void w1(int a){ if (a != 0) return; v0(); v1(); }` has an **empty** arm, so
//! c2 deletes the block and points the branch straight at the epilogue with the
//! relation *itself* rather than its negation:
//!
//! ```text
//!   value arm:  cmpwi cr6,r3,0 ; bt 26,+12 ; li r3,5 ; b -> EPI   (negated)
//!   void  arm:  cmpwi cr6,r3,0 ; bf 26,+12 -> EPI                 (NOT negated)
//! ```
//!
//! That is `work/w-cross/PREREG.md` §1's **empty-arm inversion**, found in
//! `src/system/negate_test.cpp`, reproduced in the smallest body that has it.
//! It composes: `w2` (two void guards) emits two `bf`, both to the epilogue.
//! And the void form is **byte-identical at `/O1` and `/Ox`** — with no arm
//! there is nothing to duplicate — which is its own control on the mode split
//! above.
//!
//! ## What it refuses, each with the measurement beside it
//!
//! * **Two exits producing the same value.** c2 **merges the arms**:
//!   `if(a) return 5; if(b) return 5; …` emits one arm and branches *backwards*
//!   into it with the sense inverted (`409afff4  bf 26,-12`), and a guard
//!   returning the sequence's own literal loses its arm entirely
//!   (`work/w-conv/p/probe2.cpp::m2`, `m0`, at `/O1` **and** `/Ox`). The merge
//!   also costs a **sixth** compiler-label slot where every in-class cell costs
//!   five, so a lane that admitted it unnoticed would emit six wrong bytes in
//!   the symbol table as well as a wrong block.
//! * **A guard placed after a call.** `int e7(int a){ v0(); if(a) return 5;
//!   return 0; }` keeps `a` in **r31** across the call and then folds the whole
//!   `if` branchlessly (`subfic ; li ; subfe ; and`). Callee-saved plus a fold,
//!   both refused elsewhere.
//! * **An arm containing a call** (`probe2::ac`). One cell per mode; refused
//!   under the same three-witness rule W10 used for the entry-block park.
//! * **A W10 guarded call in the same body** (`probe3::x6`). c2 composes them
//!   and the emitter would too, but two block plans in one body is a second
//!   production, not a wider one.
//! * **Everything W10 refuses** — the entry-block park, anything callee-saved,
//!   and a sequence that is really a tail call — inherited by construction,
//!   because the rest of the body is handed to the **same**
//!   [`super::calls::parse_call_sequence_from`] loop rather than to a copy.
//!   That last one is not hypothetical: `void w3(int a,int b,int c){ if(a)
//!   return; if(b) return; if(c) return; v0(); }` is **not framed at all** —
//!   three `bclr` folds and a tail `b`, 32 B, no `.pdata`.

use super::super::expr::{eat_scopes, BODY_SCOPE_DEPTH};
use super::super::{BodyShape, SeqEarlyReturnShape};
use super::calls::parse_call_sequence_from;
use super::cond_tail::eat_cmp_operand_type;
use super::params::parse_params;
use crate::func::readers::{eat, eat_byte, eat_int_like, eat_opt_stmt_marker, read_token_var, read_varint};
use crate::func::Rel;

/// Close scopes from `depth` down to (but not including) `target`, each
/// optionally preceded by its own line marker.
///
/// The twin of [`super::guarded_seq`]'s, restated rather than shared for the
/// reason that file gives: the two shapes deliberately do not import each
/// other's grammar, so neither can silently widen the other.
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

/// One guard's condition: `B9 <formal> <TYPE> · 33 <TYPE> <k> · <rel>`.
///
/// The **same** grammar [`super::cond_tail`] and [`super::guarded_seq`] read,
/// through the same [`eat_cmp_operand_type`] locator — which admits only the
/// five spellings with an oracle witness, because the spelling is what says
/// *signed*, i.e. `2f……` (`cmpwi`) against `2b……` (`cmplwi`).
fn eat_condition(
    seg: &[u8],
    p: &mut usize,
    params: &[u32],
) -> Option<(usize, Rel, bool, i32)> {
    if !eat_byte(seg, p, 0xB9) {
        return None;
    }
    let (cmp_tok, w) = read_token_var(seg, *p)?;
    *p += w;
    let signed = eat_cmp_operand_type(seg, p)?;
    let cmp_param = params.iter().position(|q| *q == cmp_tok)?;
    if !eat_byte(seg, p, 0x33) {
        return None;
    }
    let mut q = *p;
    if eat_cmp_operand_type(seg, &mut q)? != signed {
        return None;
    }
    *p = q;
    let k = read_varint(seg, p)?;
    // `cmpwi`/`cmplwi` take a 16-bit immediate; a wider literal needs `lis`+`ori`
    // into a scratch and then the register-register form, which has no witness.
    if signed && !(-0x8000..=0x7FFF).contains(&k) {
        return None;
    }
    if !signed && !(0..=0xFFFF).contains(&k) {
        return None;
    }
    let rel = Rel::from_opcode(*seg.get(*p)?)?;
    *p += 1;
    Some((cmp_param, rel, signed, k))
}

/// The then-clause of one guarded early return.
///
/// Two forms, and the cursor tells them apart with no lookahead heuristic:
///
/// ```text
///   value:  33 <int-like> <K> · 41 <int-like> · 3A <epilogue>
///   void:                                       3A <epilogue>
/// ```
///
/// Returns `(value, epilogue_label)`. The epilogue label is returned rather than
/// checked here so the caller can require **every** guard and the body's own
/// return plumbing to name the same one — a guard branching anywhere else is a
/// control transfer this class does not model, and admitting it would drop an
/// edge on the floor.
fn eat_return_arm(seg: &[u8], p: &mut usize) -> Option<(Option<i32>, u32)> {
    eat_opt_stmt_marker(seg, p);
    let value = if seg.get(*p) == Some(&0x33) {
        let mut q = *p;
        if !eat_byte(seg, &mut q, 0x33) || !eat_int_like(seg, &mut q) {
            return None;
        }
        let k = read_varint(seg, &mut q)?;
        // The arm materializes the value with one `li r3,k`; anything wider is
        // `lis`+`ori` or an `addis` pair and has no witness in this class.
        if !(-0x8000..=0x7FFF).contains(&k) {
            return None;
        }
        // `41 <TYPE>` — RESULT. The type is read by spelling through the same
        // `eat_int_like` W30 established for the sequence's own literal tail, so
        // `unsigned`, `long`, an `enum` and a `const int` are one value class
        // here exactly as they are there.
        if !eat_byte(seg, &mut q, 0x41) || !eat_int_like(seg, &mut q) {
            return None;
        }
        *p = q;
        Some(k)
    } else {
        None
    };
    eat_opt_stmt_marker(seg, p);
    if !eat_byte(seg, p, 0x3A) {
        return None;
    }
    let (epi, w) = read_token_var(seg, *p)?;
    *p += w;
    Some((value, epi))
}

/// The label the body's **own return plumbing** defines — the epilogue.
///
/// Every early return's `3A` must name this one. Checking each guard against the
/// others is not enough: a body with a *single* guard would then define the
/// epilogue by itself and a retargeted jump would read as consistent, which is
/// a dropped control transfer emitted as a branch to the wrong block.
///
/// The end of a body is `54 <BODY_SCOPE_DEPTH> · 29 <epi> · 4F 12 47`, and that
/// three-part anchor is what this looks for — the scope close alone would be a
/// byte pattern, and the label alone occurs once per guard.
fn body_epilogue_label(seg: &[u8]) -> Option<u32> {
    let close = [0x54, BODY_SCOPE_DEPTH as u8];
    let mut found = None;
    let mut i = 0;
    while i + 2 < seg.len() {
        if seg[i..i + 2] == close && seg.get(i + 2) == Some(&0x29) {
            let mut q = i + 3;
            if let Some((tok, w)) = read_token_var(seg, q) {
                q += w;
                if seg.get(q..q + 3) == Some(&[0x4F, 0x12, 0x47][..]) {
                    found = Some(tok);
                }
            }
        }
        i += 1;
    }
    found
}

/// Try to parse **N guarded early returns followed by a call sequence**.
///
/// Non-committal in the house style: works on a copy of the cursor and returns
/// `None` with no side effects, so a body that is not this production keeps its
/// own first-blocker census key.
///
/// `depth` is the scope depth at `start` — after `parse_segment_shape` has eaten
/// the body's `53` and any further scopes, which for a body whose first
/// statement is the `if` already includes that `if`'s own `53`.
pub(crate) fn try_parse_early_return_seq(
    seg: &[u8],
    start: usize,
    lo: usize,
    depth: usize,
) -> Option<BodyShape> {
    // The first `if` must have opened a scope of its own, or the `54` that closes
    // it below cannot be there and this is a different production.
    if depth <= BODY_SCOPE_DEPTH {
        return None;
    }
    let mut p = start;
    let params = parse_params(seg, lo).ok()?;
    if params.is_empty() {
        return None;
    }

    // Read the body's own epilogue label FIRST, so every arm is checked against
    // the function's real exit rather than against the first arm's opinion of it.
    let body_epi = body_epilogue_label(seg)?;

    let mut early: Vec<SeqEarlyReturnShape> = Vec::new();
    let mut epilogue: Option<u32> = None;
    let mut guard_depth = depth;
    loop {
        let (cmp_param, rel, signed, k) = eat_condition(seg, &mut p, &params)?;

        // `38 <L>` — brFALSE to the skip entry.
        if !eat_byte(seg, &mut p, 0x38) {
            return None;
        }
        let (skip_label, w) = read_token_var(seg, p)?;
        p += w;

        // **W-SMALL — short-circuit `&&`.** A further condition-and-branch group
        // right here, naming the SAME skip label, is one more conjunct of *this*
        // guard rather than a new statement: `if (a != 0 && b != 0) return 5;`
        // is the single-guard IL with `b9 … 20 38 <same label>` inserted, and it
        // emits one more `cmp ; bc` at the same target with the same sense
        // (`419a` in both words, measured on `work/w-small/l2/v_and.cpp`).
        //
        // Three things make this safe to admit and each is checked, not assumed:
        // the label must be the SAME one (a different label is `||`'s shape or a
        // `goto`); there is no scope marker between the groups, so a second
        // `if` — which opens its own `53` — cannot be mistaken for a conjunct;
        // and the compiler-label counter charges **+0** for this shape
        // (`ho-and`/`ho-or`, `stride 5 / extra 0`, `docs/rungs/2026-08-04-w-label.md`
        // §2.3), so `coff::plan_labels` needs no change and the branches are all
        // FORWARD — `labels.rs` invariant 4 would refuse them otherwise.
        let mut and_conds: Vec<(usize, Rel, bool, i32)> = Vec::new();
        while seg.get(p) == Some(&0xB9) {
            let c = eat_condition(seg, &mut p, &params)?;
            if !eat_byte(seg, &mut p, 0x38) {
                return None;
            }
            let (l, w) = read_token_var(seg, p)?;
            p += w;
            if l != skip_label {
                // A conjunct that skips somewhere else is not a conjunct. `||`
                // lands here (its first branch names the ARM, not the skip), and
                // so would any `goto` out of the condition.
                return None;
            }
            and_conds.push(c);
        }

        // The then-clause opens its own scope.
        let mut d = guard_depth;
        eat_scopes(seg, &mut p, &mut d).ok()?;
        if d <= guard_depth {
            return None;
        }
        let (value, epi) = eat_return_arm(seg, &mut p)?;
        // **W-SMALL — a VOID arm with `&&` conjuncts is a DIFFERENT block plan,
        // and this refusal is a measurement rather than caution.**
        //
        // The value form is uniform: every conjunct emits `cmp ; bc` at the one
        // skip label with the one sense. The void form is not, and assuming it
        // was produced **12 live `Port=Mismatch` cells** on this lane's own grid
        // before the oracle caught it — every void cell in it and no other.
        //
        // `void P(int a,int b){ if (a!=0 && b!=0) return; v0(); v1(); }` at
        // `/Ox /GS- /c` (`work/w-small/grid/pos_c2_void.cpp`, 52 B):
        //
        // ```text
        //   000c  2f030000  cmpwi cr6,r3,0
        //   0010  419a000c  bf 26 -> 0x001c   <- the SEQUENCE, negated sense
        //   0014  2f040000  cmpwi cr6,r4,0
        //   0018  409a000c  bt 26 -> 0x0024   <- the EPILOGUE, relation itself
        //   001c  bl ?v0 ; bl ?v1
        //   0024  epilogue
        // ```
        //
        // Two targets and two senses: every conjunct but the LAST steps to the
        // sequence start carrying the negation, and only the last carries the
        // empty-arm inversion to the epilogue. The sequence start is a **third**
        // label that no production in this port mints, and what the
        // compiler-label counter charges for minting it is unmeasured — which
        // `docs/rungs/2026-08-04-w-label.md` AA-b/AA-c say cannot be recovered
        // from either the emitted obj or the `.gl` seed. So it is refused with
        // the measurement attached rather than fitted from one cell.
        //
        // This also narrows w-label's `ho-and` reading. That probe was an
        // `int gp(int)` body, so its "two branches naming one interior target"
        // is a fact about the VALUE arm; the void arm does not have it.
        if value.is_none() && !and_conds.is_empty() {
            return None;
        }
        if epi != body_epi {
            // A `goto`, a nested join, or a `return` out of an inner scope. Not
            // this class, and admitting it drops an edge.
            return None;
        }
        match epilogue {
            None => epilogue = Some(epi),
            // Every early return must leave through the SAME label. A second
            // target is a `goto` or a nested join, and this class has one exit
            // block.
            Some(e) if e == epi => {}
            Some(_) => return None,
        }
        eat_closes_to(seg, &mut p, &mut d, guard_depth)?;

        // A `3A` here — after the then-clause's `54`, in the `if`'s own scope —
        // is an **`else` arm**, which is `guarded_seq`'s refusal and stays
        // refused: `/Ox` and `/O2` tail-duplicate the join and the epilogue
        // where `/O1` shares them, on a threshold that is a c2 cost model.
        eat_opt_stmt_marker(seg, &mut p);
        if seg.get(p) == Some(&0x3A) {
            return None;
        }

        // `29 <L>` — the label the `38` named, and no other.
        eat_opt_stmt_marker(seg, &mut p);
        if !eat_byte(seg, &mut p, 0x29) {
            return None;
        }
        let (lbl, w) = read_token_var(seg, p)?;
        p += w;
        if lbl != skip_label {
            return None;
        }

        early.push(SeqEarlyReturnShape { and_conds, cmp_param, rel, signed, k, value });

        // The `if` statement's own scope closes, back to the body's.
        let mut d = guard_depth;
        eat_closes_to(seg, &mut p, &mut d, BODY_SCOPE_DEPTH)?;
        if d != BODY_SCOPE_DEPTH {
            return None;
        }

        // Another guard? Each subsequent `if` opens its own `53` that the body's
        // entry `eat_scopes` did not consume. A cursor copy, so a non-guard
        // statement leaves the sequence's cursor exactly where it was.
        let mut q = p;
        let mut nd = BODY_SCOPE_DEPTH;
        if eat_scopes(seg, &mut q, &mut nd).is_ok()
            && nd > BODY_SCOPE_DEPTH
            && seg.get(q) == Some(&0xB9)
        {
            p = q;
            guard_depth = nd;
            continue;
        }
        break;
    }

    // ---- the exit-value merge: refused, and this is where -------------------
    //
    // Measured on `work/w-conv/p/probe2.cpp::m2` and `::m0` at `/O1` and `/Ox`.
    // The parser is the right place: the census and the emission gate must agree
    // about what is in class, and the merge is visible in the IL.
    let mut seen: Vec<i32> = Vec::with_capacity(early.len() + 1);
    for e in &early {
        match e.value {
            Some(v) if seen.contains(&v) => return None,
            Some(v) => seen.push(v),
            // A void arm has no value to collide with. Mixing void and value
            // arms in one body cannot happen — the function has one return type
            // — but the loop does not assume that, and the tail check below
            // makes the two consistent anyway.
            None => {}
        }
    }
    let all_void = early.iter().all(|e| e.value.is_none());
    let any_void = early.iter().any(|e| e.value.is_none());
    if all_void != any_void {
        return None;
    }

    // ---- and the rest of the body is an ordinary call sequence -------------
    //
    // Handed to the SAME loop `parse_call_sequence` runs — not a copy — so the
    // tail forms, the `MAX_SEQ_CALLS` bound, `plan_saved_gprs` and the
    // one-call-and-a-void-tail tail-call escape are all that function's. The
    // escape matters here: `w3` (three void guards, one trailing call) is not a
    // framed body at all, and only that loop knows it.
    let shape = match parse_call_sequence_from(seg, &mut p, lo, Vec::new(), None, early) {
        Ok(shape) => shape,
        Err(_) => return None,
    };

    // The sequence's own tail must agree with the arms: a void body's arms are
    // all void, a value body's arms all produce a literal **distinct from the
    // tail's**. Checked here rather than inside the shared loop, which knows
    // nothing about early returns.
    let BodyShape::CallSeq { tail, early: ref e, .. } = shape else {
        return None;
    };
    match tail {
        super::super::SeqTail::Void if all_void => {}
        super::super::SeqTail::Lit(k) if !all_void && !e.iter().any(|r| r.value == Some(k)) => {}
        _ => return None,
    }
    Some(shape)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::func::body::SeqTail;

    /// `int a1(int a) { if (a != 0) return 5; v0(); return 0; }` — the minimal
    /// cell, transcribed from the `.ex` `cl.exe` 16.00.11886.00 produced for
    /// `work/w-conv/p/il1.cpp`. The slice is the whole function segment, `4F 1F`
    /// to the next, so it carries the per-function optimization word, the `46`
    /// formals list and the body.
    const CELL_ONE: &[u8] = &[
        0x4f, 0x1f, 0x80, 0x05, 0x00, 0xa0, 0x00, 0x4f, 0x20, 0x80, 0xfe, 0x00, 0x4f, 0x33, 0x0d,
        0x66, 0x12, 0x1c, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0b, 0x0b, 0x03, 0x0f, 0x10, 0x18,
        0x01, 0x00, 0x0e, 0x6c, 0x12, 0x38, 0x1d, 0x42, 0x45, 0x0e, 0x06, 0x01, 0x01, 0x01, 0x0d,
        0x08, 0x00, 0x0f, 0x4f, 0x02, 0x20, 0x00, 0x4f, 0x01, 0x02, 0x53, 0x53, 0x26, 0xe6, 0x09,
        0x46, 0x2d, 0xe5, 0x09, 0x4c, 0x4f, 0x11, 0x53, 0x53, 0xb9, 0xe5, 0x09, 0x86, 0x41, 0x74,
        0x33, 0x86, 0x41, 0x74, 0x00, 0x20, 0x38, 0xe8, 0x09, 0x53, 0x33, 0x86, 0x41, 0x74, 0x05,
        0x41, 0x86, 0x41, 0x74, 0x3a, 0xe7, 0x09, 0x54, 0x04, 0x29, 0xe8, 0x09, 0x54, 0x03, 0x26,
        0xe3, 0x09, 0xbd, 0x82, 0x07, 0x03, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, 0x4c, 0x4b, 0x33,
        0x86, 0x41, 0x74, 0x00, 0x41, 0x86, 0x41, 0x74, 0x3a, 0xe7, 0x09, 0x54, 0x02, 0x29, 0xe7,
        0x09, 0x4f, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
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

    fn parse(seg: &[u8]) -> Option<BodyShape> {
        let (p, lo, depth) = at_body(seg);
        try_parse_early_return_seq(seg, p, lo, depth)
    }

    #[test]
    fn the_minimal_cell_is_one_early_return_over_a_one_call_sequence() {
        let Some(BodyShape::CallSeq { calls, tail, saved, guard, early, .. }) =
            parse(CELL_ONE)
        else {
            panic!("in class")
        };
        assert_eq!(calls.len(), 1);
        assert_eq!(tail, SeqTail::Lit(0));
        assert!(saved.is_empty(), "Class A — nothing survives a call");
        assert!(guard.is_none(), "a W10 guarded CALL is a different production");
        assert_eq!(early.len(), 1);
        assert_eq!(early[0].cmp_param, 0);
        assert_eq!(early[0].rel, Rel::Ne);
        assert!(early[0].signed, "`int` operand -> cmpwi, not cmplwi");
        assert_eq!(early[0].k, 0);
        assert_eq!(early[0].value, Some(5));
        assert!(
            early[0].and_conds.is_empty(),
            "a single-test guard has no conjuncts — the `&&` field must not \
             acquire a phantom entry from the plain shape"
        );
    }

    /// **W-SMALL — `int P(int a,int b){ if (a!=0 && b!=0) return 5; v0();
    /// return 0; }`**, transcribed from the `.ex` for
    /// `work/w-small/grid/pos_c2_int.cpp`.
    ///
    /// The whole content of the shape is visible in the bytes: the conjuncts are
    /// two copies of one group naming **one** label (`38 eb 09` twice), and the
    /// arm and the `29 eb 09` that defines it occur once. Two separate `if`s
    /// would mint two labels and two arms — that production already parsed and
    /// is unchanged.
    const CELL_AND2: &[u8] = &[
        0x4f, 0x1f, 0x80, 0x05, 0x00, 0xa0, 0x00, 0x4f, 0x20, 0x80, 0xfe, 0x00, 0x4f, 0x33, 0x0d,
        0x66, 0x12, 0x1c, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0b, 0x0b, 0x03, 0x0f, 0x10, 0x18,
        0x01, 0x00, 0x0e, 0x6c, 0x12, 0x38, 0x1d, 0x42, 0x45, 0x0e, 0x06, 0x01, 0x01, 0x01, 0x0d,
        0x08, 0x00, 0x0f, 0x4f, 0x02, 0x20, 0x00, 0x4f, 0x01, 0x04, 0x53, 0x53, 0x26, 0xe9, 0x09,
        0x46, 0x2d, 0xe8, 0x09, 0x2d, 0xe7, 0x09, 0x4c, 0x4f, 0x11, 0x53, 0x53, 0xb9, 0xe7, 0x09,
        0x86, 0x41, 0x74, 0x33, 0x86, 0x41, 0x74, 0x00, 0x20, 0x38, 0xeb, 0x09, 0xb9, 0xe8, 0x09,
        0x86, 0x41, 0x74, 0x33, 0x86, 0x41, 0x74, 0x00, 0x20, 0x38, 0xeb, 0x09, 0x53, 0x33, 0x86,
        0x41, 0x74, 0x05, 0x41, 0x86, 0x41, 0x74, 0x3a, 0xea, 0x09, 0x54, 0x04, 0x29, 0xeb, 0x09,
        0x54, 0x03, 0x26, 0xe3, 0x09, 0xbd, 0x82, 0x07, 0x03, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00,
        0x4c, 0x4b, 0x33, 0x86, 0x41, 0x74, 0x00, 0x41, 0x86, 0x41, 0x74, 0x3a, 0xea, 0x09, 0x54,
        0x02, 0x29, 0xea, 0x09, 0x4f, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4f, 0x02, 0x20, 0x00,
        0x4f, 0x01, 0x05, 0x4d,
    ];

    /// The same source with a **void** arm — `void P(int a,int b){ if (a!=0 &&
    /// b!=0) return; v0(); v1(); }`, from `work/w-small/grid/neg_c2_void.cpp`.
    /// The two conjunct groups are byte-identical to `CELL_AND2`'s; only the arm
    /// differs (`53 3a ea 09 54 04` — an empty arm that is just the exit goto).
    const CELL_VOID2: &[u8] = &[
        0x4f, 0x1f, 0x80, 0x05, 0x00, 0xa0, 0x00, 0x4f, 0x20, 0x80, 0xfe, 0x00, 0x4f, 0x33, 0x0d,
        0x66, 0x12, 0x1c, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0b, 0x0b, 0x03, 0x0f, 0x10, 0x18,
        0x01, 0x00, 0x0e, 0x6c, 0x12, 0x38, 0x1d, 0x42, 0x45, 0x0e, 0x06, 0x01, 0x01, 0x01, 0x0d,
        0x08, 0x00, 0x0f, 0x4f, 0x02, 0x20, 0x00, 0x4f, 0x01, 0x04, 0x53, 0x53, 0x26, 0xe9, 0x09,
        0x46, 0x2d, 0xe8, 0x09, 0x2d, 0xe7, 0x09, 0x4c, 0x4f, 0x11, 0x53, 0x53, 0xb9, 0xe7, 0x09,
        0x86, 0x41, 0x74, 0x33, 0x86, 0x41, 0x74, 0x00, 0x20, 0x38, 0xeb, 0x09, 0xb9, 0xe8, 0x09,
        0x86, 0x41, 0x74, 0x33, 0x86, 0x41, 0x74, 0x00, 0x20, 0x38, 0xeb, 0x09, 0x53, 0x3a, 0xea,
        0x09, 0x54, 0x04, 0x29, 0xeb, 0x09, 0x54, 0x03, 0x26, 0xe3, 0x09, 0xbd, 0x82, 0x07, 0x03,
        0x00, 0x80, 0x01, 0x10, 0x00, 0x00, 0x4c, 0x4b, 0x26, 0xe4, 0x09, 0xbd, 0x82, 0x07, 0x03,
        0x00, 0x80, 0x01, 0x10, 0x00, 0x00, 0x4c, 0x4b, 0x3a, 0xea, 0x09, 0x54, 0x02, 0x29, 0xea,
        0x09, 0x4f, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00, 0x4f, 0x02, 0x20, 0x00, 0x4f, 0x01, 0x05,
        0x4d,
    ];

    /// A short-circuit `&&` is ONE guard with two conditions, not two guards.
    #[test]
    fn a_short_circuit_and_is_one_early_return_carrying_both_conditions() {
        let Some(BodyShape::CallSeq { early, calls, tail, guard, .. }) = parse(CELL_AND2) else {
            panic!("in class")
        };
        assert_eq!(calls.len(), 1);
        assert_eq!(tail, SeqTail::Lit(0));
        assert!(guard.is_none());
        // ONE early return — two would be the `if(a) …; if(b) …;` production and
        // would emit two arms and two labels.
        assert_eq!(early.len(), 1, "`a && b` is one guard, not two");
        assert_eq!(early[0].cmp_param, 0);
        assert_eq!(early[0].rel, Rel::Ne);
        assert_eq!(early[0].value, Some(5));
        // …and the second condition rides along, on the SECOND formal.
        assert_eq!(early[0].and_conds.len(), 1);
        assert_eq!(early[0].and_conds[0], (1, Rel::Ne, true, 0));
    }

    /// **The VOID arm with conjuncts is refused, and the refusal is a byte
    /// measurement.** c2 sends every conjunct but the last to the SEQUENCE with
    /// the negated sense and only the last to the epilogue with the relation
    /// itself (`work/w-small/grid/pos_c2_void.cpp` disassembly, in the parser's
    /// own comment) — two targets, two senses, and a third label whose counter
    /// cost is unmeasured. Emitting the value form's uniform loop here computes
    /// `||` where the source says `&&`; it was 12 live mismatching cells on this
    /// lane's grid before the oracle caught it.
    #[test]
    fn a_void_arm_with_and_conjuncts_is_refused() {
        assert!(
            parse(CELL_VOID2).is_none(),
            "the void `&&` is a different block plan and must not reach the emitter"
        );
        // …and the control: the same cell's conjunct groups are byte-identical to
        // the value form's, so what refuses is the ARM and nothing else.
        let a = CELL_AND2.windows(15).position(|w| w[0] == 0xb9 && w[1] == 0xe7);
        let v = CELL_VOID2.windows(15).position(|w| w[0] == 0xb9 && w[1] == 0xe7);
        let (a, v) = (a.expect("and cell"), v.expect("void cell"));
        assert_eq!(
            CELL_AND2[a..a + 30],
            CELL_VOID2[v..v + 30],
            "both cells carry the SAME two conjunct groups; only the arm differs"
        );
    }

    /// **The exit-value merge is refused, and the byte that refuses it is the
    /// arm's literal.**
    ///
    /// Rewrite the cell's early-return literal from `5` to `0` — the sequence's
    /// own tail literal — and nothing else. c2 emits a *different body* for
    /// that source (`work/w-conv/p/probe2.cpp::m0`: the arm vanishes and the
    /// branch skips the call straight to the shared `li r3,0`, 44 B against
    /// 52 B), and it costs a sixth compiler-label slot. One byte moved, one
    /// refusal, and the control below shows the byte is the cause.
    #[test]
    fn an_exit_value_that_repeats_is_refused() {
        let mut seg = CELL_ONE.to_vec();
        let arm = seg
            .windows(5)
            .position(|w| w == [0x33, 0x86, 0x41, 0x74, 0x05])
            .expect("the arm's `LIT int 5`");
        seg[arm + 4] = 0x00;
        assert!(parse(&seg).is_none(), "5 -> 0 collides with the tail's `return 0`");
    }

    /// **The control on that refusal.** Move the arm's literal to a *third*
    /// distinct value instead of to the tail's, changing exactly the same byte.
    /// It must still parse — so the refusal above is caused by the collision and
    /// not by having touched the byte at all.
    #[test]
    fn a_distinct_exit_value_still_parses() {
        let mut seg = CELL_ONE.to_vec();
        let arm = seg
            .windows(5)
            .position(|w| w == [0x33, 0x86, 0x41, 0x74, 0x05])
            .expect("the arm's `LIT int 5`");
        seg[arm + 4] = 0x0b;
        let Some(BodyShape::CallSeq { early, .. }) = parse(&seg) else {
            panic!("11 does not collide with 0")
        };
        assert_eq!(early[0].value, Some(11));
    }

    /// A body with **no** early return is not this production. Built by excising
    /// the whole `if` statement, which leaves `v0(); return 0;` — a plain
    /// sequence the older rung owns.
    #[test]
    fn a_body_with_no_guard_is_not_this_shape() {
        let mut seg = CELL_ONE.to_vec();
        let cond = seg
            .windows(3)
            .position(|w| w == [0xb9, 0xe5, 0x09])
            .expect("the condition");
        let call = seg
            .windows(3)
            .position(|w| w == [0x26, 0xe3, 0x09])
            .expect("the trailing call");
        seg.drain(cond..call);
        assert!(parse(&seg).is_none());
    }

    /// An **`else` arm** is refused, for the reason W10 measured: `/Ox` and `/O2`
    /// tail-duplicate the join block and the epilogue where `/O1` shares them
    /// behind one `b`, on a threshold that is a c2 cost model. Built by
    /// inserting a `3A <join>` where an `else` puts one — after the
    /// then-clause's `54 04`, in the `if` statement's own scope.
    #[test]
    fn an_else_arm_is_refused() {
        let mut seg = CELL_ONE.to_vec();
        let close = seg
            .windows(2)
            .position(|w| w == [0x54, 0x04])
            .expect("the then-clause's close");
        seg.splice(close + 2..close + 2, [0x3a, 0xf5, 0x09]);
        assert!(parse(&seg).is_none());
    }

    /// A guard whose `3A` names a label **other** than the one the body returns
    /// through is a `goto`, not an early return, and must refuse: admitting it
    /// would drop a control transfer on the floor. Built by retargeting the
    /// arm's jump alone, leaving the body's own return plumbing untouched.
    #[test]
    fn an_arm_that_jumps_somewhere_else_is_refused() {
        let mut seg = CELL_ONE.to_vec();
        let arm_jump = seg
            .windows(3)
            .position(|w| w == [0x3a, 0xe7, 0x09])
            .expect("the arm's jump to the epilogue");
        seg[arm_jump + 1] = 0xf5;
        assert!(parse(&seg).is_none());
    }
}
