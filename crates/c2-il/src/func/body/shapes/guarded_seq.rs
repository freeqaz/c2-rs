//! **W10 — a guarded call inside a framed call sequence: the FRAMED × BRANCHING
//! cell.**
//!
//! `work/w-frame/RANKING.md` §4 measured the one thing the port had never done.
//! Over the 105 functions it emits byte-exact: **28 are framed, 2 branch, and
//! ZERO are both.** Its entire branching capability was two *leaf* bodies from
//! W8's `cond_tail`, and **10 of the 17 FRONTIER TUs need the product**. This
//! file is the IL half of that cell.
//!
//! ## The shape, and why it is this one
//!
//! ```cpp
//!   void f(int a)         { if (a != 0) g(); h(); }
//! ```
//!
//! It is `mvp_call_seq.cpp`'s shipped `void two(){ v0(); v1(); }` with a compare
//! and a branch inserted, and its obj is 44 bytes:
//!
//! ```text
//!   7d8802a6  mflr  r12          the shipped Class A 96-byte frame
//!   9181fff8  stw   r12,-8(r1)
//!   9421ffa0  stwu  r1,-96(r1)
//!   2f030000  cmpwi cr6,r3,0     <- the guard, between the prologue and the
//!   419a0008  bt    26,+8           sequence; NEGATION of the IL relation
//!   4bffffed  bl    ?g            the GUARDED call
//!   4bffffe9  bl    ?h            the sequence continues
//!   38210060  addi  r1,r1,96
//!   8181fff8  lwz   r12,-8(r1)
//!   7d8803a6  mtlr  r12
//!   4e800020  blr
//! ```
//!
//! ## The `else` arm was built, graded, and REMOVED — read this before adding it
//!
//! An `else` arm makes the then-block end in the **intra-section unconditional
//! `b`** of board **#191**, and this lane built it, ran it against the real
//! `c2`, and took it back out. The reason is a finding, not a bug:
//!
//! > **`/Ox` and `/O2` TAIL-DUPLICATE the join block and the whole epilogue;
//! > `/O1` shares them behind one `b`.** `void e(int a){ if(a) v0(); else
//! > v1(); v2(); }` is **52 B with a `48000008`** at `/O1` and **68 B with no
//! > `b` at all** at `/Ox` and `/O2`, the join's `bl` and all four epilogue
//! > words appearing twice.
//!
//! That **refutes `docs/OPT_MODE.md`'s standing claim** — restated in
//! `c2_core::codegen::OptMode`'s own doc — that the two modes *"differ in
//! exactly one rule … never a different opcode, never a different operand
//! order, only a register field"*. Here they differ in **block structure**.
//!
//! And the duplication has a **size threshold this lane did not crack**. At
//! `/Ox`, with the join at one call it duplicates (68 B); at **two** and at
//! **three** calls it emits `/O1`'s shared `b` (`work/w-cross/p/probe5.cpp`,
//! `j1`/`j2`/`j3`). So the boundary is bracketed by exactly one cell either
//! side, on a quantity — how many bytes are worth duplicating — that is a c2
//! cost model. `docs/CFG_SHAPE.md` §3.5 declined the fold cost model for
//! precisely this reason, and `work/w-cross/PREREG.md` §3.2 clause 2 forbids
//! fitting it here. **A shape whose layout the rules mis-handle must come out as
//! a gap, never as a plausible-looking wrong block order.**
//!
//! So the `3A <join>` is refused by name below, `encode_b_intra` was deleted
//! rather than left as an ungraded encoder (w-frame row **F-c**: a rung adding a
//! code path with no coverage under the graded profile is adding a first
//! witness), and board #191 stays open — with a far sharper characterization
//! than it had.
//!
//! ## What it refuses, and why each refusal is a measurement
//!
//! Every one of these is a shape lane w-cross compiled with the real toolchain
//! at the workload's own flags (`work/w-cross/p/probe{2,3}.cpp`) and declined
//! **by name** rather than approximated. Its own pre-registration
//! (`work/w-cross/PREREG.md` §3.2 clause 2) forbids fitting an allocation or
//! schedule rule with fewer than three witnesses.
//!
//! * **A guarded arm needing an entry-block park.** `probe2 s4`
//!   (`if(a) a2(b,a); v1();`) hoists `mr r11,r3` **and** `mr r3,r4` above the
//!   compare and leaves `mr r4,r11` in the arm; `probe3 P0` parks in **r10**
//!   because r11 is taken, and leaves `mr r4,r5` in the arm. The rule that
//!   separates "hoisted" from "left in the arm" fits three cells and is tested
//!   by none, so a guarded arm here takes **at most one argument** and the
//!   entry block is empty by construction.
//! * **Anything callee-saved.** `probe3 P2`/`S0`/`S1` put a formal in r31 and
//!   then the compare reads **r31** in `P2` and **r3** in `S0`/`S1`, because
//!   `P2`'s entry block also clobbers r3. One rule, four witnesses — and still
//!   refused, because it composes with the park rule above, which is refused.
//!   `saved` must be empty and the tail must save no call result.
//! * **Both arms calling the same callee.** Board **#193**: c2 tail-merges the
//!   two `bl` sites and inverts the layout.
//! * **A guard with no unguarded call after it.** `void f(int a){ if(a) g(); }`
//!   is not framed at all — it is fold band 2 (`bclr`) plus a tail call
//!   (`probe2 e0`: `cmpwi ; bnelr cr6 ; b ?v0 ; blr`, 16 B, no `.pdata`).
//!   Emitting a framed body there is a wrong-bytes obj, not a gap.
//! * **A second guard.** One `if` per body. Every two-guard witness needs a
//!   callee-saved formal (`probe3 L3`), which the clause above already refuses;
//!   admitting it here would be a class with no reachable instance.
//!
//! ## What it does NOT need, and why that is stated
//!
//! **A fixup list.** `docs/CFG_SHAPE.md` §6.2 item B asks for one; W8's
//! `cond_tail` declined it because its block order is fixed, and so is this
//! one's — one branch, one target, and a displacement that is a function of a
//! setup length the emitter already has. The first shape that needs a real
//! label→offset map is `src/system/negate_test.cpp`, whose two branches share a
//! target, and that TU is declined at **nine** independent refusals
//! (`work/w-cross/PREREG.md` §1).
//!
//! **A `coff.rs` edit.** The compiler-label counter stride was measured before
//! this file was written, holding the body shape fixed and varying only the
//! branch count (`probe3` `L0`…`L4`, 0/1/2/4 branch targets): the stride is **5**
//! throughout, the same as a straight-line framed body. So a guarded sequence's
//! `label_lead` is 0 and the counter is untouched.

use super::super::expr::{eat_scopes, BODY_SCOPE_DEPTH};
use super::super::{BodyShape, SeqGuardShape};
use super::calls::{eat_call_args, eat_call_head, parse_call_sequence_from};
use super::cond_tail::eat_cmp_operand_type;
use super::params::parse_params;
use crate::func::readers::{eat, eat_byte, eat_opt_stmt_marker, read_token_var, read_varint};
use crate::func::{IlOp, Rel};

/// One statement-position call whose result is discarded:
/// `26 <callee> · <CALL head> · <args> · 4C 4B`. Returns the callee token and
/// the raw argument streams, exactly as [`super::calls::parse_call_sequence`]
/// collects them, so the marshalling still has one validator.
fn eat_stmt_call(seg: &[u8], p: &mut usize) -> Option<(u32, Vec<Vec<IlOp>>)> {
    eat_opt_stmt_marker(seg, p);
    let (tok, ret) = eat_call_head(seg, p).ok()?;
    let args = eat_call_args(seg, p).ok()?;
    if !eat_byte(seg, p, 0x4B) {
        return None;
    }
    // A discarded `float`/`double` result obliges the TU to declare `_fltused`,
    // whose placement is not modeled. The gate every other discarding site asks.
    ret.discarded(seg, *p).ok()?;
    Some((tok, args))
}

/// Close scopes from `depth` down to (but not including) `target`, each
/// optionally preceded by its own line marker. The twin of
/// [`super::cond_tail`]'s, restated rather than shared because that one is
/// private to a shape whose grammar this file deliberately does not import.
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

/// A guarded arm carries **at most one argument**, so the entry block is empty
/// and no park is ever needed. See the module doc: the hoisting rule that would
/// be required otherwise fits three probe cells and is tested by none.
fn arm_arity_in_class(args: &[Vec<IlOp>]) -> bool {
    args.len() <= 1
}

/// Try to parse a **guarded call sequence**.
///
/// Non-committal in the house style: works on a copy of the cursor and returns
/// `None` with no side effects, so a body that is not this production keeps its
/// own first-blocker census key.
///
/// `depth` is the scope depth at `start` — after `parse_segment_shape` has eaten
/// the body's `53` and any further scopes. For a body whose first statement is
/// the `if`, that already includes the `if` statement's own `53`.
pub(crate) fn try_parse_guarded_seq(
    seg: &[u8],
    start: usize,
    lo: usize,
    depth: usize,
) -> Option<BodyShape> {
    // The `if` statement must have opened a scope of its own, or the `54` that
    // closes it below cannot be there and this is a different production.
    if depth <= BODY_SCOPE_DEPTH {
        return None;
    }
    let mut p = start;
    let params = parse_params(seg, lo).ok()?;
    if params.is_empty() {
        return None;
    }

    // ---- the condition: the SAME grammar `cond_tail` reads -----------------
    //
    // `B9 <formal> <TYPE> · 33 <TYPE> <k> · <rel>`, with the literal restated in
    // the operand's own type. Read through `eat_cmp_operand_type`, which admits
    // only the five spellings with a witness, because the spelling is what says
    // *signed* — i.e. `2f……` (`cmpwi`) against `2b……` (`cmplwi`).
    if !eat_byte(seg, &mut p, 0xB9) {
        return None;
    }
    let (cmp_tok, w) = read_token_var(seg, p)?;
    p += w;
    let signed = eat_cmp_operand_type(seg, &mut p)?;
    let cmp_param = params.iter().position(|q| *q == cmp_tok)?;
    if !eat_byte(seg, &mut p, 0x33) {
        return None;
    }
    let mut q = p;
    if eat_cmp_operand_type(seg, &mut q)? != signed {
        return None;
    }
    p = q;
    let k = read_varint(seg, &mut p)?;
    // `cmpwi`/`cmplwi` take a 16-bit immediate; a wider literal is `lis`+`ori`
    // into a scratch and then the register-register form, which has no capture.
    if signed && !(-0x8000..=0x7FFF).contains(&k) {
        return None;
    }
    if !signed && !(0..=0xFFFF).contains(&k) {
        return None;
    }
    let rel = Rel::from_opcode(*seg.get(p)?)?;
    p += 1;

    // ---- `38 <L>` — brFALSE to the else entry (or to the join) -------------
    if !eat_byte(seg, &mut p, 0x38) {
        return None;
    }
    let (else_label, w) = read_token_var(seg, p)?;
    p += w;

    // ---- the then-arm: one scope, one statement call, close ----------------
    let mut d = depth;
    eat_scopes(seg, &mut p, &mut d).ok()?;
    if d <= depth {
        // The `38` opened no block of its own. Some other production.
        return None;
    }
    let (then_callee, then_args) = eat_stmt_call(seg, &mut p)?;
    if !arm_arity_in_class(&then_args) {
        return None;
    }
    eat_closes_to(seg, &mut p, &mut d, depth)?;

    // ---- `3A <join>` — the jump over an ELSE arm, and the class boundary ---
    //
    // Emitted AFTER the then-clause's `54`, in the `if` statement's own scope
    // (`docs/CFG_SHAPE.md` §2.3). Its presence says the `if` has an `else`, and
    // **that is refused**: the emitted block layout is MODE-DEPENDENT. `/O1`
    // shares the join behind one intra-section `b` (52 B); `/Ox` and `/O2`
    // tail-duplicate the join block and the whole epilogue (68 B, no `b` at
    // all). The module doc has the bytes and the bracket — at `/Ox` the
    // duplication happens with a one-call join and stops at two, so the
    // threshold is a c2 cost model with one cell either side of it.
    //
    // Refused **here**, in the parser, rather than in the emitter: the census
    // and the gate must agree about what is in class, and a mode-dependent
    // acceptance boundary is exactly the disagreement `docs/GAPS.md` §6 records
    // (the IL parser cannot see the optimization word).
    eat_opt_stmt_marker(seg, &mut p);
    if seg.get(p) == Some(&0x3A) {
        return None;
    }

    // ---- `29 <L>` — the label the `38` named --------------------------------
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x29) {
        return None;
    }
    let (lbl, w) = read_token_var(seg, p)?;
    p += w;
    if lbl != else_label {
        // A `29` that is not the one the `38` named is a different control-flow
        // graph — a nested `if`, a `goto` target, an `||` join. Out of class.
        return None;
    }

    let raw: Vec<(u32, Vec<Vec<IlOp>>)> = vec![(then_callee, then_args)];

    // ---- the `if` statement's own scope closes ------------------------------
    let mut d = depth;
    eat_closes_to(seg, &mut p, &mut d, BODY_SCOPE_DEPTH)?;
    if d != BODY_SCOPE_DEPTH {
        return None;
    }

    // ---- and the rest of the body is an ordinary call sequence -------------
    //
    // Handed to the SAME loop `parse_call_sequence` runs, with the guarded
    // call(s) as its prefix — not a copy of it. The tail forms, the
    // `MAX_SEQ_CALLS` bound, the `plan_saved_gprs` allocation and the
    // one-call-and-void-tail tail-call escape are all that function's, so the
    // guard cannot drift from the sequence it guards.
    let guard = SeqGuardShape { cmp_param, rel, signed, k };
    match parse_call_sequence_from(seg, &mut p, lo, raw, Some(guard), Vec::new()) {
        Ok(shape) => Some(shape),
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::func::body::SeqTail;

    /// `void A(int a) { if (a != 0) v0(); v1(); }` — cell A, the minimal
    /// framed-and-branching body, transcribed from the `.ex` `cl.exe`
    /// 16.00.11886.00 produced for `work/w-cross/p/probe4.cpp`. The slice is the
    /// whole function segment, `4F 1F` to the next, so it carries the
    /// per-function optimization word, the `46` formals list and the body.
    const CELL_A: &[u8] = &[
        0x4f, 0x1f, 0x80, 0x05, 0x00, 0xa0, 0x00, 0x4f, 0x20, 0x80, 0xfe, 0x00, 0x4f, 0x33, 0x0d,
        0x66, 0x12, 0x1c, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0b, 0x0b, 0x03, 0x0f, 0x10, 0x18,
        0x01, 0x00, 0x0e, 0x6c, 0x12, 0x38, 0x1d, 0x42, 0x45, 0x0e, 0x06, 0x01, 0x01, 0x01, 0x0d,
        0x08, 0x00, 0x0f, 0x4f, 0x02, 0x20, 0x00, 0x4f, 0x01, 0x04, 0x53, 0x53, 0x26, 0xea, 0x09,
        0x46, 0x2d, 0xe9, 0x09, 0x4c, 0x4f, 0x11, 0x53, 0x53, 0xb9, 0xe9, 0x09, 0x86, 0x41, 0x74,
        0x33, 0x86, 0x41, 0x74, 0x00, 0x20, 0x38, 0xec, 0x09, 0x53, 0x26, 0xe4, 0x09, 0xbd, 0x82,
        0x07, 0x03, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, 0x4c, 0x4b, 0x54, 0x04, 0x29, 0xec, 0x09,
        0x54, 0x03, 0x26, 0xe6, 0x09, 0xbd, 0x82, 0x07, 0x03, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00,
        0x4c, 0x4b, 0x3a, 0xeb, 0x09, 0x54, 0x02, 0x29, 0xeb, 0x09, 0x4f, 0x12, 0x47, 0x54, 0x01,
        0x54, 0x00,
    ];

    /// `void B(int a) { if (a != 0) v0(); else v1(); v2(); }` — the `else`
    /// form, which must be **REFUSED**. The delta against `CELL_A` is a
    /// `3A <join>` after the then-clause's `54` and a second `29`, exactly as
    /// `docs/CFG_SHAPE.md` §2.3 describes an `if`/`else`.
    ///
    /// Its obj is 52 B at `/O1` and **68 B at `/Ox` and `/O2`**, which is the
    /// whole reason it is refused; see the module doc.
    const CELL_B: &[u8] = &[
        0x4f, 0x1f, 0x80, 0x05, 0x00, 0xa0, 0x00, 0x4f, 0x20, 0x80, 0xfe, 0x00, 0x4f, 0x33, 0x0d,
        0x66, 0x12, 0x1c, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0b, 0x0b, 0x03, 0x0f, 0x10, 0x18,
        0x01, 0x00, 0x0e, 0x6c, 0x12, 0x38, 0x1d, 0x42, 0x45, 0x0e, 0x06, 0x01, 0x01, 0x01, 0x0d,
        0x08, 0x00, 0x0f, 0x4f, 0x02, 0x20, 0x00, 0x4f, 0x01, 0x05, 0x53, 0x53, 0x26, 0xee, 0x09,
        0x46, 0x2d, 0xed, 0x09, 0x4c, 0x4f, 0x11, 0x53, 0x53, 0xb9, 0xed, 0x09, 0x86, 0x41, 0x74,
        0x33, 0x86, 0x41, 0x74, 0x00, 0x20, 0x38, 0xf0, 0x09, 0x53, 0x26, 0xe4, 0x09, 0xbd, 0x82,
        0x07, 0x03, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, 0x4c, 0x4b, 0x54, 0x04, 0x3a, 0xf1, 0x09,
        0x29, 0xf0, 0x09, 0x53, 0x26, 0xe6, 0x09, 0xbd, 0x82, 0x07, 0x03, 0x00, 0x80, 0x01, 0x10,
        0x00, 0x00, 0x4c, 0x4b, 0x54, 0x04, 0x29, 0xf1, 0x09, 0x54, 0x03, 0x26, 0xe8, 0x09, 0xbd,
        0x82, 0x07, 0x03, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, 0x4c, 0x4b, 0x3a, 0xef, 0x09, 0x54,
        0x02, 0x29, 0xef, 0x09, 0x4f, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
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

    fn parse(seg: &[u8]) -> BodyShape {
        let (p, lo, depth) = at_body(seg);
        try_parse_guarded_seq(seg, p, lo, depth).expect("in class")
    }

    #[test]
    fn the_minimal_cell_is_a_two_call_sequence_with_a_one_armed_guard() {
        let BodyShape::CallSeq { calls, tail, saved, guard, .. } = parse(CELL_A) else {
            panic!("wrong shape")
        };
        assert_eq!(calls.len(), 2, "the guarded call plus the one after it");
        assert_eq!(tail, SeqTail::Void);
        assert!(saved.is_empty(), "Class A — nothing survives a call");
        let g = guard.expect("guarded");
        assert_eq!(g.cmp_param, 0);
        assert_eq!(g.rel, Rel::Ne);
        assert!(g.signed, "`int` operand -> cmpwi, not cmplwi");
        assert_eq!(g.k, 0);
    }

    /// **The `else` form is refused, and the byte that refuses it is the
    /// `3A <join>`.**
    ///
    /// This is not a taste boundary. `void e(int a){ if(a) v0(); else v1();
    /// v2(); }` is 52 B with an intra-section `48000008` at `/O1` and **68 B
    /// with no `b` at all** at `/Ox` and `/O2`, where c2 tail-duplicates the
    /// join block and all four epilogue words into both arms — measured
    /// against the real toolchain, `work/w-cross/p/probe5.cpp`. Emitting one
    /// layout for both modes is a wrong-bytes obj in six of `scripts/lanes.txt`'s
    /// twelve lanes.
    ///
    /// The assertion is on the **shape**, not on a census key, because the
    /// refusal has to be invisible to the emitter: a body that reaches
    /// `call_seq_text` with an else arm would get a displacement computed for a
    /// block that is not there.
    #[test]
    fn an_else_arm_is_refused_because_its_layout_is_mode_dependent() {
        let (p, lo, depth) = at_body(CELL_B);
        assert!(
            try_parse_guarded_seq(CELL_B, p, lo, depth).is_none(),
            "the `3A <join>` must refuse: /Ox and /O2 tail-duplicate the join"
        );
    }

    /// **The control on that refusal**, without which it would pass just as
    /// well if the recognizer refused every `CELL_B`-shaped byte for some
    /// unrelated reason — which is how a negative test goes green while
    /// refusing everything.
    ///
    /// Excise `CELL_B`'s whole `else` clause — from the `3A <join>` through the
    /// `29 <join>` that closes it — and nothing else. What is left is
    /// `if (a != 0) v0(); v2();`, `CELL_A`'s shape at `CELL_B`'s tokens, and it
    /// must parse **with a guard**. So the refusal is caused by the else clause
    /// and by nothing else in the body.
    #[test]
    fn the_else_refusal_is_caused_by_the_else_clause_and_nothing_else() {
        let mut seg = CELL_B.to_vec();
        let jump = seg
            .windows(3)
            .position(|w| w == [0x3a, 0xf1, 0x09])
            .expect("the jump over the else arm");
        // The else ENTRY `29 f0 09` is the label the `38` names and stays; the
        // else BODY runs from just after it to the end of the join label.
        let entry = seg
            .windows(3)
            .position(|w| w == [0x29, 0xf0, 0x09])
            .expect("the else entry");
        let join = seg
            .windows(3)
            .position(|w| w == [0x29, 0xf1, 0x09])
            .expect("the join label");
        assert!(jump < entry && entry < join, "jump, else entry, join label");
        // Later range first, so the earlier index stays valid.
        seg.drain(entry + 3..join + 3);
        seg.drain(jump..jump + 3);
        let (p, lo, depth) = at_body(&seg);
        let shape = try_parse_guarded_seq(&seg, p, lo, depth);
        assert!(
            matches!(&shape, Some(BodyShape::CallSeq { guard: Some(_), .. })),
            "excising only the else clause must leave a guarded sequence, got {shape:?}"
        );
        let Some(BodyShape::CallSeq { calls, .. }) = shape else { unreachable!() };
        assert_eq!(calls.len(), 2, "the guarded v0() and the trailing v2()");
    }

    /// A guard with **no unguarded call after it** is not this shape and is not
    /// framed at all: `void f(int a){ if(a) v0(); }` is fold band 2 plus a tail
    /// call (`probe2 e0`: `cmpwi ; bnelr cr6 ; b ?v0 ; blr`, 16 B, no `.pdata`).
    /// Built by deleting cell A's trailing call statement.
    #[test]
    fn a_guard_with_nothing_after_it_is_refused() {
        // `54 03` closes the if scope; the trailing call runs from there to the
        // `3a eb 09` return jump. Splice it out.
        let mut seg = CELL_A.to_vec();
        let close = seg
            .windows(2)
            .position(|w| w == [0x54, 0x03])
            .expect("the if scope's close");
        let ret = seg
            .windows(3)
            .position(|w| w == [0x3a, 0xeb, 0x09])
            .expect("the return jump");
        seg.drain(close + 2..ret);
        let (p, lo, depth) = at_body(&seg);
        // It must not come back as a *guarded sequence*. (`parse_call_sequence`
        // routes a lone statement call with a void tail to the tail-call
        // production, so the guard has nowhere to live.)
        assert!(match try_parse_guarded_seq(&seg, p, lo, depth) {
            None => true,
            Some(BodyShape::CallSeq { guard, .. }) => guard.is_none(),
            Some(_) => true,
        });
    }
}
