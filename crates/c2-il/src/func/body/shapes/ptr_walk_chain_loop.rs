//! **The body-parameterized pointer-walk accumulate loop** — the first
//! recognizer here that consumes a *statement list of unbounded length* and
//! hands the emitter an **operation list** rather than a fixed set of scalars.
//!
//! ```c
//!   int P(const char* s) {
//!       int r = K0;
//!       while (*s) { int c = *s; r = r <op> x; r = r <op> x; … s++; }
//!       return r;
//!   }
//! ```
//!
//! # What is different about this shape, and why it needed a new production
//!
//! [`super::ptr_walk_loop`] recognizes one function. Its accumulate is consumed
//! by literal `eat_byte(0x04)` / `eat(&[0x04, 0x02])` / `eat_byte(0x06)` calls
//! at fixed cursor positions — a byte-pattern match, and its own rung says so.
//! There is no place in [`crate::func::PtrWalkModLoop`] to put a second chain
//! step, because the struct has no list.
//!
//! This production **loops**: it eats `r = r <op> x;` statements until the
//! pointer increment arrives, and every one it eats becomes a
//! [`crate::func::ChainOp`]. The emitted body's length, the induction load's
//! slot, the record form's slot and every register field are then *computed*
//! from that list by `c2_core::codegen::ptr_walk_chain_loop`. Nothing about the
//! emitted shape is transcribed.
//!
//! # The IL is a TOP-TEST loop; the rotation is `c2`'s, not the front end's
//!
//! This is worth stating because [`super::ptr_walk_loop`]'s grammar is the
//! *rotated* one and the two look like they should agree. They do not, and the
//! difference is the source construct rather than the compiler:
//!
//! ```text
//!   for (u = …; *u; u++)      3A Ltest · 29 Lincr · <incr> · 29 Ltest · <test>
//!                             · 38 Lexit · <body> · 3A Lincr
//!                             -- c1xx rotates it: the IL jumps OVER the
//!                                increment into the test
//!
//!   while (*s) { … s++; }     29 Ltest · <test> · 38 Lexit · <body> · <incr>
//!                             · 3A Ltest
//!                             -- NO rotation in the IL at all
//! ```
//!
//! Yet `c2` emits the same rotated machine shape for both — a peeled load, an
//! entry guard, then a body ending in a backward `bc`. So for this class the
//! **rotation is the port's own work**, which is the sense in which this is a
//! lowering and not a transcription: nothing in the IL tells the emitter where
//! the peel goes.
//!
//! # The accepted class, and what each refusal cost
//!
//! Every clause below is a **positive** check. Captured counterexamples, each
//! from `work/w-varloop/probe.py` against real `c2`:
//!
//! * **exactly one formal**, the walked pointer, at slot 0 — w-hash measured
//!   that moving the pointer off slot 0 re-plans the whole block (the
//!   accumulator coalesces into r3, the guard changes form), and w-sched2 §3.4
//!   found the same mechanism arriving *inside* the body as the register pool
//!   shifting under unmoved roles;
//! * the loop test is a **bare truth test** on the dereference — no `2C`
//!   widening, no `33 <int> 00`, no relational opcode. `*s != 0` is a different
//!   IL production;
//! * the character is bound to its **own automatic local** (`int c = *s;`) and
//!   the chain reads that local;
//! * the chain's operators are [`crate::func::ChainOpKind`]'s four, whose
//!   omissions are measured — `&` selects `andi.`, which writes cr0 and makes
//!   `c2` demote the record form and add an explicit `cmpwi`, a different and
//!   longer block;
//! * every literal inside its opcode's immediate field, so **one step is one
//!   instruction**. This is what keeps board #644 outside the class by
//!   construction rather than by inspection: a producer split across a
//!   `lis`/`ori` pair cannot be built from an admitted step;
//! * **at least one step reads the character**, so `pv` is defined. w-sched2's
//!   reconstruction refuses the rest with the reason printed and so does this;
//! * a stride of exactly **1** — `lbzu rN,1(rU)` folds precisely this increment
//!   into the addressing mode.
//!
//! # The label counter
//!
//! Identical status to [`super::ptr_walk_loop`]: a loop leaf charges the
//! compiler-label counter and *which* of the four charges cannot be read off
//! the emitted bytes, so `label_slots` returns `None` — the three-valued gate's
//! *refuse* answer. A TU pairing one of these with a framed function is
//! rejected; one without is admitted. Board #746.

use crate::func::body::expr::{eat_return_plumbing, parse_formals};
use crate::func::body::{blk, Block, BodyShape};
use crate::func::readers::{
    eat, eat_byte, eat_int_like, eat_opt_stmt_marker, is_ptr4_kind, read_token_var, read_type,
    read_varint,
};
use crate::func::{ChainOp, ChainOpKind, ChainRhs, PtrWalkChainLoop};

/// The scope depth the body opens at, mirrored from `expr::BODY_SCOPE_DEPTH`.
const BODY_SCOPE_DEPTH: usize = 2;

/// The largest chain this production will build.
///
/// **A bound, not a rule.** w-sched2's axis reaches eight steps and this lane's
/// grid grades to ten; past that nothing has been measured, and an emitter that
/// silently kept going would be claiming a length no oracle has seen. It also
/// bounds the parse against a hostile stream. The emitted body is
/// `M + 2` words and every branch in it is checked against its field width
/// anyway, so this constant can only ever be the *first* refusal, never the
/// only one.
const MAX_CHAIN: usize = 10;

/// The **one-byte signed** element class — `char` / `signed char` under a
/// `const` pointer, spelled `A2 11 <id>`.
///
/// Required as the literal pair for [`is_ptr4_kind`]'s reason: the tag carries
/// the alignment class and the kind's high nibble the size, and both say 1
/// here, so the pair double-checks the read. `82 11` (non-`const`) is
/// deliberately absent — every cell graded for this rung walks a
/// `const char*`, and a tag that never varied is indistinguishable from a
/// constant (`docs/GAPS.md` §6).
fn is_const_int1s_type(tag: u8, kind: u8) -> bool {
    (tag, kind) == (0xA2, 0x11)
}

/// The **one-byte unsigned** element class under a `const` pointer — `A2 12`.
/// Selects the `mr.` / `cmplwi` record forms; see
/// [`crate::func::PtrWalkChainLoop::elem_unsigned`].
fn is_const_int1u_type(tag: u8, kind: u8) -> bool {
    (tag, kind) == (0xA2, 0x12)
}

/// Consume `26 <tok>` and return the token.
fn eat_designator(seg: &[u8], p: &mut usize, what: &'static str) -> Result<u32, Block> {
    if !eat_byte(seg, p, 0x26) {
        return Err(blk(seg, *p, what));
    }
    let (tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, what))?;
    *p += w;
    Ok(tok)
}

/// Consume `B9 <tok> <TYPE>` for a width-4 integer operand and return the token.
fn eat_int_load(seg: &[u8], p: &mut usize, what: &'static str) -> Result<u32, Block> {
    if !eat_byte(seg, p, 0xB9) {
        return Err(blk(seg, *p, what));
    }
    let (tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, what))?;
    *p += w;
    if !eat_int_like(seg, p) {
        return Err(blk(seg, *p, what));
    }
    Ok(tok)
}

/// Consume a TYPE naming a width-4 **pointer** and return its id.
fn eat_ptr_type(seg: &[u8], p: &mut usize, what: &'static str) -> Result<u32, Block> {
    match read_type(seg, *p) {
        Some((tag, kind, id, w)) if is_ptr4_kind(tag, kind) => {
            *p += w;
            Ok(id)
        }
        _ => Err(blk(seg, *p, what)),
    }
}

/// Consume `B9 <ptrTok> <ptr TYPE> · 30 <elem TYPE>` — the dereference — and
/// return the element's signedness.
///
/// The element TYPE is required to be one of the two **one-byte** classes, not
/// merely "some type": the emitted instruction is `lbz`/`lbzu`, and a `short`
/// or `int` element would be `lhz`/`lwz` with a different stride in the update
/// form. Admitting the type without checking its width is `docs/GAPS.md` §6's
/// one-field-two-facts defect.
fn eat_deref(
    seg: &[u8],
    p: &mut usize,
    ptr_tok: u32,
    ptr_type_id: u32,
    what: &'static str,
) -> Result<bool, Block> {
    if !eat_byte(seg, p, 0xB9) {
        return Err(blk(seg, *p, what));
    }
    let (tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, what))?;
    *p += w;
    if tok != ptr_tok {
        return Err(blk(seg, *p, what));
    }
    if eat_ptr_type(seg, p, what)? != ptr_type_id {
        return Err(blk(seg, *p, what));
    }
    if !eat_byte(seg, p, 0x30) {
        return Err(blk(seg, *p, what));
    }
    match read_type(seg, *p) {
        Some((tag, kind, _, w)) if is_const_int1s_type(tag, kind) => {
            *p += w;
            Ok(false)
        }
        Some((tag, kind, _, w)) if is_const_int1u_type(tag, kind) => {
            *p += w;
            Ok(true)
        }
        _ => Err(blk(seg, *p, what)),
    }
}

/// The immediate a chain step's opcode can carry in **one** instruction.
///
/// Two ranges, because two encodings: `addi`/`mulli` take a signed 16-bit
/// field, `xori`/`ori` an unsigned one. `mulli` narrows further to the literals
/// `c2` lowers as a single `mulli` at all — [`super::ptr_walk_loop::is_mulli_literal`]
/// is the 38-constant grid that says which, and the extra `> 0` clause is its
/// own measured fact: `c2` rewrites `x + a*(-3)` as `x - a*3`, changing an
/// opcode this shape does not carry.
fn literal_fits(kind: ChainOpKind, k: i32) -> bool {
    match kind {
        ChainOpKind::Add => (-0x8000..=0x7FFF).contains(&k),
        ChainOpKind::Xor | ChainOpKind::Or => (0..=0xFFFF).contains(&k),
        ChainOpKind::Mul => super::ptr_walk_loop::is_mulli_literal(k) && k > 0,
    }
}

/// The IL operator byte → [`ChainOpKind`]. `None` for every other byte, which
/// is what refuses `&` (`0B`), `-` (`03`), `/` (`05`) and `%` (`06`).
fn chain_op_kind(b: u8) -> Option<ChainOpKind> {
    match b {
        0x02 => Some(ChainOpKind::Add),
        0x04 => Some(ChainOpKind::Mul),
        0x0C => Some(ChainOpKind::Or),
        0x0D => Some(ChainOpKind::Xor),
        _ => None,
    }
}

/// **The recognizer.** `start` is the first byte after the body's own `53` and
/// any leading scope/line markers; `lo` is the `4C 4F 11` body marker.
///
/// Non-committal in the sense every sibling production here is: it works on its
/// own cursor, and every failure returns `Err` with the caller's cursor
/// untouched, so a body that declines still reports the blocker its own
/// dispatch arm found.
pub(crate) fn try_parse_ptr_walk_chain_loop(
    seg: &[u8],
    start: usize,
    lo: usize,
    locals: &[u32],
    ptr_locals: &[u32],
) -> Result<BodyShape, Block> {
    let params = parse_formals(seg, lo)?;
    // **Exactly one formal.** Everything about the emitted block plan — that
    // the accumulator coalesces into r3, that the guard folds to `bclr`, that
    // the walked pointer lands in r10 — is measured at this arity. A second
    // formal is w-hash's `p3`/`swap` re-plan.
    if params.len() != 1 {
        return Err(blk(seg, start, "chainloop-formals-not-1"));
    }
    let src_tok = params[0];

    let mut p = start;

    // ---- statement 1: `r = K0` --------------------------------------------
    let acc_tok = eat_designator(seg, &mut p, "chainloop-acc-designator")?;
    if !locals.contains(&acc_tok) {
        // The accumulator must be an automatic local `.sy` knows about. A
        // file-scope `static int` is a memory object and folding its store away
        // would drop a real store — `assign.rs`'s membership test, for its
        // reason.
        return Err(blk(seg, p, "chainloop-acc-not-a-local"));
    }
    if !eat_byte(seg, &mut p, 0x33) || !eat_int_like(seg, &mut p) {
        return Err(blk(seg, p, "chainloop-acc-init-lit"));
    }
    let acc_init = read_varint(seg, &mut p).ok_or(blk(seg, p, "chainloop-acc-init-varint"))?;
    if !(-0x8000..=0x7FFF).contains(&acc_init) {
        // `li rA,K0` is one instruction only inside `simm16`; wider is a
        // `lis`/`ori` pair and a different preamble.
        return Err(blk(seg, p, "chainloop-acc-init-wide"));
    }
    if !eat_byte(seg, &mut p, 0x32) || !eat_int_like(seg, &mut p) || !eat_byte(seg, &mut p, 0x4B) {
        return Err(blk(seg, p, "chainloop-acc-init-store"));
    }

    // ---- the `while` scope opens ------------------------------------------
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "chainloop-while-scope"));
    }

    // ---- the TEST: `29 Ltest · <deref s> · 38 Lexit` ----------------------
    //
    // A **bare truth test**: the branch reads the loaded byte directly. There
    // is no `2C` widening and no relational opcode, which is exactly what makes
    // `while (*s)` a different production from `while (*s != 0)`
    // (`super::ptr_walk_loop`'s module docs record that pair as measured).
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x29) {
        return Err(blk(seg, p, "chainloop-test-label"));
    }
    let (l_test, w) = read_token_var(seg, p).ok_or(blk(seg, p, "chainloop-test-label-tok"))?;
    p += w;
    // The pointer is read straight from formal 0 — there is no local copy in
    // this source shape at all, so the `mr r10,r3` the port emits is its own
    // decision rather than a transcribed IL store.
    if !ptr_locals.is_empty() && ptr_locals.contains(&src_tok) {
        // A formal that `.sy` also reports as an address-taken local is not the
        // register-resident pointer this plan assumes.
        return Err(blk(seg, p, "chainloop-ptr-addr-taken"));
    }
    let save = p;
    if !eat_byte(seg, &mut p, 0xB9) {
        return Err(blk(seg, p, "chainloop-test-load"));
    }
    let (t, w) = read_token_var(seg, p).ok_or(blk(seg, p, "chainloop-test-tok"))?;
    p += w;
    if t != src_tok {
        return Err(blk(seg, p, "chainloop-test-not-formal0"));
    }
    let ptr_type_id = eat_ptr_type(seg, &mut p, "chainloop-test-ptrtype")?;
    p = save;
    let elem_unsigned = eat_deref(seg, &mut p, src_tok, ptr_type_id, "chainloop-test-deref")?;
    if !eat_byte(seg, &mut p, 0x38) {
        return Err(blk(seg, p, "chainloop-test-brfalse"));
    }
    let (l_exit, w) = read_token_var(seg, p).ok_or(blk(seg, p, "chainloop-exit-tok"))?;
    p += w;
    if l_exit == l_test {
        return Err(blk(seg, p, "chainloop-labels-alias"));
    }

    // ---- the loop body scope opens, then `int c = *s` ---------------------
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "chainloop-body-scope"));
    }
    eat_opt_stmt_marker(seg, &mut p);
    let char_tok = eat_designator(seg, &mut p, "chainloop-char-designator")?;
    if char_tok == acc_tok || !locals.contains(&char_tok) {
        return Err(blk(seg, p, "chainloop-char-not-a-local"));
    }
    // The **same** dereference as the test, on the same pointer and element
    // type. `c2` emits one load for both, which is the whole reason the peeled
    // load can serve the test and the chain at once.
    if eat_deref(seg, &mut p, src_tok, ptr_type_id, "chainloop-char-deref")? != elem_unsigned {
        return Err(blk(seg, p, "chainloop-char-elem-type"));
    }
    // `2C <int TYPE> <varint>` — the widening to `int`. Its own opcode here,
    // never implicit.
    if !eat_byte(seg, &mut p, 0x2C) {
        return Err(blk(seg, p, "chainloop-char-convert"));
    }
    if !eat_int_like(seg, &mut p) {
        return Err(blk(seg, p, "chainloop-char-convert-type"));
    }
    read_varint(seg, &mut p).ok_or(blk(seg, p, "chainloop-char-convert-varint"))?;
    if !eat_byte(seg, &mut p, 0x32) || !eat_int_like(seg, &mut p) || !eat_byte(seg, &mut p, 0x4B) {
        return Err(blk(seg, p, "chainloop-char-store"));
    }

    // ---- THE CHAIN: `r = r <op> x;` until the increment --------------------
    //
    // **This is the loop the whole shape exists for.** Each turn consumes one
    // statement and appends one `ChainOp`; the emitter's schedule, allocation
    // and body length are all functions of the list this builds. The exit
    // condition is positive — the increment's designator names the pointer —
    // so a stream that runs out, or that names something else, refuses rather
    // than terminating the chain early.
    let mut ops: Vec<ChainOp> = Vec::new();
    loop {
        eat_opt_stmt_marker(seg, &mut p);
        let save = p;
        let dest = eat_designator(seg, &mut p, "chainloop-step-designator")?;
        if dest == src_tok {
            // The pointer increment: the chain is complete. Rewind so the
            // increment is parsed by its own code below.
            p = save;
            break;
        }
        if dest != acc_tok {
            return Err(blk(seg, p, "chainloop-step-not-the-accumulator"));
        }
        if ops.len() >= MAX_CHAIN {
            return Err(blk(seg, p, "chainloop-too-long"));
        }
        // `b9 <acc> <int>` — every step reads the accumulator first. A step
        // that did not would not be a chain step at all.
        if eat_int_load(seg, &mut p, "chainloop-step-acc-load")? != acc_tok {
            return Err(blk(seg, p, "chainloop-step-acc-load"));
        }
        // The right-hand operand: a literal, or the character's local.
        let rhs = if seg.get(p) == Some(&0x33) {
            p += 1;
            if !eat_int_like(seg, &mut p) {
                return Err(blk(seg, p, "chainloop-step-lit-type"));
            }
            let k = read_varint(seg, &mut p).ok_or(blk(seg, p, "chainloop-step-lit-varint"))?;
            ChainRhs::Lit(k)
        } else {
            if eat_int_load(seg, &mut p, "chainloop-step-rhs-load")? != char_tok {
                return Err(blk(seg, p, "chainloop-step-rhs-not-the-char"));
            }
            ChainRhs::Char
        };
        let kind = seg
            .get(p)
            .copied()
            .and_then(chain_op_kind)
            .ok_or(blk(seg, p, "chainloop-step-op-not-admitted"))?;
        p += 1;
        if let ChainRhs::Lit(k) = rhs {
            if !literal_fits(kind, k) {
                // One step must be one instruction. A literal outside its
                // opcode's field is a `lis`/`ori` pair — board #644's split
                // producer — and a `mulli`-ineligible one is a strength
                // reduction; both are different bodies.
                return Err(blk(seg, p, "chainloop-step-lit-not-one-instruction"));
            }
        }
        if !eat_byte(seg, &mut p, 0x32)
            || !eat_int_like(seg, &mut p)
            || !eat_byte(seg, &mut p, 0x4B)
        {
            return Err(blk(seg, p, "chainloop-step-store"));
        }
        ops.push(ChainOp { kind, rhs });
    }
    if ops.is_empty() {
        return Err(blk(seg, p, "chainloop-empty-chain"));
    }
    // **`pv` must exist.** Every rule the emitter applies is stated in terms of
    // the last step reading the character; a chain that never reads it is
    // outside w-sched2's reconstructed class, refused there with the reason
    // printed, and refused here.
    if !ops.iter().any(|o| o.rhs == ChainRhs::Char) {
        return Err(blk(seg, p, "chainloop-char-never-read"));
    }

    // ---- the increment: `26 s · 33 <TYPE> 01 · 35 <ptr TYPE> · 4B` ---------
    eat_opt_stmt_marker(seg, &mut p);
    if eat_designator(seg, &mut p, "chainloop-incr-designator")? != src_tok {
        return Err(blk(seg, p, "chainloop-incr-not-the-pointer"));
    }
    if !eat_byte(seg, &mut p, 0x33) {
        return Err(blk(seg, p, "chainloop-incr-lit"));
    }
    // The stride literal's TYPE is the *element* type of the pointer
    // arithmetic, so it is read for its width and not classified.
    match read_type(seg, p) {
        Some((_, _, _, w)) => p += w,
        None => return Err(blk(seg, p, "chainloop-incr-lit-type")),
    }
    if read_varint(seg, &mut p) != Some(1) {
        return Err(blk(seg, p, "chainloop-incr-stride-not-1"));
    }
    if !eat_byte(seg, &mut p, 0x35) {
        return Err(blk(seg, p, "chainloop-incr-op"));
    }
    if eat_ptr_type(seg, &mut p, "chainloop-incr-type")? != ptr_type_id {
        return Err(blk(seg, p, "chainloop-incr-type"));
    }
    if !eat_byte(seg, &mut p, 0x4B) {
        return Err(blk(seg, p, "chainloop-incr-end"));
    }

    // ---- the BACK EDGE: `54 04` · `3A Ltest` ------------------------------
    eat_opt_stmt_marker(seg, &mut p);
    if !eat(seg, &mut p, &[0x54, (BODY_SCOPE_DEPTH + 2) as u8]) {
        return Err(blk(seg, p, "chainloop-body-scope-close"));
    }
    if !eat_byte(seg, &mut p, 0x3A) {
        return Err(blk(seg, p, "chainloop-back-edge"));
    }
    let (t, w) = read_token_var(seg, p).ok_or(blk(seg, p, "chainloop-back-edge-tok"))?;
    p += w;
    if t != l_test {
        return Err(blk(seg, p, "chainloop-back-edge-target"));
    }

    // ---- the EXIT: `29 Lexit` · `54 03` · `return r` ----------------------
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x29) {
        return Err(blk(seg, p, "chainloop-exit-label"));
    }
    let (t, w) = read_token_var(seg, p).ok_or(blk(seg, p, "chainloop-exit-label-tok"))?;
    p += w;
    if t != l_exit {
        return Err(blk(seg, p, "chainloop-exit-label-mismatch"));
    }
    eat_opt_stmt_marker(seg, &mut p);
    if !eat(seg, &mut p, &[0x54, (BODY_SCOPE_DEPTH + 1) as u8]) {
        return Err(blk(seg, p, "chainloop-while-scope-close"));
    }
    eat_opt_stmt_marker(seg, &mut p);
    if eat_int_load(seg, &mut p, "chainloop-return-load")? != acc_tok {
        return Err(blk(seg, p, "chainloop-return-not-the-accumulator"));
    }
    // The shared tail, and the fail-closed terminal — anything trailing rejects.
    eat_return_plumbing(seg, &mut p, true, BODY_SCOPE_DEPTH)?;

    Ok(BodyShape::PtrWalkChainLoop(PtrWalkChainLoop {
        params,
        acc_init,
        elem_unsigned,
        ops,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The literal ranges are **two**, because the opcodes are: `addi`/`mulli`
    /// carry a signed field and `xori`/`ori` an unsigned one. A single range
    /// would either refuse `r ^ 0xFFFF` (which is one `xori`) or admit
    /// `r + 40000` (which is not one `addi`).
    #[test]
    fn literal_ranges_follow_the_immediate_field_and_not_a_single_guess() {
        assert!(literal_fits(ChainOpKind::Add, 32767));
        assert!(literal_fits(ChainOpKind::Add, -32768));
        assert!(!literal_fits(ChainOpKind::Add, 32768));
        assert!(!literal_fits(ChainOpKind::Add, 40000));

        assert!(literal_fits(ChainOpKind::Xor, 0xFFFF));
        assert!(literal_fits(ChainOpKind::Or, 0xFFFF));
        assert!(!literal_fits(ChainOpKind::Xor, 0x1_0000));
        // A negative is not an unsigned field, whatever its magnitude.
        assert!(!literal_fits(ChainOpKind::Or, -1));

        // `mulli` narrows to the graded constant set and to positives.
        assert!(literal_fits(ChainOpKind::Mul, 7));
        assert!(literal_fits(ChainOpKind::Mul, 127));
        assert!(!literal_fits(ChainOpKind::Mul, 8), "a power of two is `rlwinm`");
        assert!(!literal_fits(ChainOpKind::Mul, 1), "the identity emits nothing");
        assert!(!literal_fits(ChainOpKind::Mul, -3), "`x + a*-3` becomes `x - a*3`");
    }

    /// The operator byte table is the whole of the admitted vocabulary, and the
    /// refusals are the measured ones rather than an omission.
    #[test]
    fn only_the_four_measured_operators_are_admitted() {
        assert_eq!(chain_op_kind(0x02), Some(ChainOpKind::Add));
        assert_eq!(chain_op_kind(0x04), Some(ChainOpKind::Mul));
        assert_eq!(chain_op_kind(0x0C), Some(ChainOpKind::Or));
        assert_eq!(chain_op_kind(0x0D), Some(ChainOpKind::Xor));
        // `&` is `andi.`, which writes cr0 and makes c2 restructure the block;
        // `-` is `subf`, non-commutative, which S5 does not speak for; `/` and
        // `%` belong to w-divmod's spine and to `PtrWalkModLoop`.
        for b in [0x03u8, 0x05, 0x06, 0x0B, 0x07, 0x08, 0x09, 0x0A, 0x00, 0xFF] {
            assert_eq!(chain_op_kind(b), None, "byte {b:#04x} must not be admitted");
        }
    }

    /// The two element classes and nothing else. `82 11` / `82 12` (non-`const`)
    /// are absent on purpose: no cell graded for this rung walks one.
    #[test]
    fn element_type_classes_are_the_two_const_one_byte_spellings() {
        assert!(is_const_int1s_type(0xA2, 0x11));
        assert!(is_const_int1u_type(0xA2, 0x12));
        assert!(!is_const_int1s_type(0xA2, 0x12));
        assert!(!is_const_int1u_type(0xA2, 0x11));
        assert!(!is_const_int1s_type(0x82, 0x11));
        assert!(!is_const_int1u_type(0x82, 0x12));
        // A width-4 int element would be `lwz`, not `lbzu`.
        assert!(!is_const_int1s_type(0x86, 0x41));
        assert!(!is_const_int1u_type(0x86, 0x41));
    }
}
