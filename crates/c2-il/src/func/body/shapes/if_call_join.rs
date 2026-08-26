//! **W-CFG1 — the two-armed `if`/`else` whose arms are CALLS and whose join is a
//! real block.** The port's first `cflow-if-n` body, and its first intra-section
//! `b` that targets a **join** rather than the epilogue.
//!
//! ```cpp
//!   const Node* f(Blend b, void *clip, float t) {
//!       const Node* n = 0;
//!       if (b >= K1) {
//!           if (b == K1) { n = 0; }          // or  !(b != K1)
//!           else {
//!               if (b >= K2) n = g_hi(clip, t);
//!               else         n = g_lo(clip, t);
//!           }
//!       }
//!       return n;
//!   }
//! ```
//!
//! This is `src/system/negate_test.cpp` — a FRONTIER TU, both of whose emitted
//! functions are this shape and whose two `.text` COMDATs are **byte-identical
//! to one another** despite the two source spellings of the middle test. So the
//! TU converts on one class or on none; there is no partial credit on it, which
//! is why the recognizer admits both spellings and nothing else.
//!
//! ## Why a TRANSCRIPTION and not a general `if-n` lowering
//!
//! `docs/ARCHITECTURE_SEAMS.md` §7 says a general control-flow lowering forces a
//! block IR plus a value merge at the join, sequenced with the frame/liveness
//! spine, and that the restructure has never been sized. Board **#411**,
//! **#269** and **#257** each hand-counted this one TU independently at 9–10
//! independent unmodeled constructs. The precedent for taking a body of that
//! price *without* the restructure is
//! [`super::ptr_walk_loop`]: one named function class, `/O1` only,
//! `NotImplemented` outside, graded byte-exact by the oracle. This file is that,
//! for the `cflow-if-n` axis.
//!
//! **The consequence is stated rather than hidden: accepting this shape is not a
//! claim about `cflow-if-n` as a class.** Twenty-one of the frontier's forty-
//! eight blocked functions are `cflow-loop` and eleven are `cflow-if-n`; this
//! recognizer takes **two** of them.
//!
//! ## What the reference emits, and the five things about it a lowering gets wrong
//!
//! Read off the real obj at the workload's own flags, and narrated by c2's own
//! `/FAsc` listing (`work/w-cfgclass/p/probe1.cod`) — the listing names every
//! block, which is how the block *plan* below is ground truth and not inference:
//!
//! ```text
//!   mflr/stw/stwu -96      the shipped Class A frame
//!   mr    r10,r3           (1) the scrutinee is EVICTED out of r3 in the entry
//!   mr    r3,r4            (2) the arms' shared argument is HOISTED above every
//!                              branch — W10 pins the opposite for a guarded
//!                              call, whose setup stays INSIDE the arm
//!   li    r11,0            (3) the result HOME is r11, initialised in the entry
//!   cmpwi cr6,r10,K1
//!   blt   cr6,$LN1         (4) ONE compare, read at LT ...
//!   beq   cr6,$LN1         (5) ... and again at EQ — CSE'd across two source
//!                              `if`s, and both arms of it exit to the SAME block
//!   cmpwi cr6,r10,K2
//!   blt   cr6,$LN2
//!   bl    <hi>
//!   b     $LN8             (6) the intra-section `b` to a JOIN block
//!   bl    <lo>
//!   $LN8: mr r11,r3        (7) the join stores the arms' shared result to the
//!   $LN1: mr r3,r11            home, and the next word reads it straight back
//!   addi/lwz/mtlr/blr
//! ```
//!
//! Two of those are the reason this is a transcription. **(5)** has no
//! representation in `Selected` — the shipped guard emitter emits one compare per
//! guard, and a body reaching it would emit `cmpwi` twice, which is the right
//! program and the wrong bytes. **(7)** is a round-trip through r11 that is
//! semantically `mr r3,r3`; every peephole a codegen lane would reach for by
//! reflex deletes it, and deleting it is the defect (board **#1400**'s finding
//! on `Primes.cpp`, reproduced one TU over).
//!
//! The middle `if`'s arm is **`n = 0` where `n` is already 0**, so c2 deletes the
//! block and points the branch straight at the exit with the relation *itself*
//! rather than its negation. That is W11's empty-arm inversion
//! ([`super::early_return`]) arriving from the other side, and it is why the
//! `==` spelling (`1F` + `38`) and the `!=` spelling (`20` + `39`) emit the same
//! word: both name the same successor.
//!
//! ## The fence
//!
//! Everything below is required literally, and each clause names the measurement
//! that put it there rather than a preference:
//!
//! * **Exactly three formals — `(int-like, ptr4, float)` — and the calls take
//!   the last two.** The park (`mr r10,r3`) and the hoist (`mr r3,r4`) are a
//!   register assignment for this arity in this order; nothing here has been
//!   graded at another.
//! * **Both arms call with the SAME argument list.** That is what makes the
//!   hoist legal; two different argument lists put a setup back inside each arm
//!   and the block plan changes.
//! * **The dead middle store is `<acc> = <the same literal the entry stored>`.**
//!   If the two literals differ the middle block is not empty, c2 emits it, and
//!   the branch senses invert back.
//! * **Both compare literals inside `simm16`** — `cmpwi` has one immediate field.
//! * **`/O1` only.** `/Ox` and `/O2` tail-duplicate a join block on a threshold
//!   W10 bracketed and did not fit (board row X-b); refusing them is the same
//!   clause [`super::ptr_walk_loop`] carries.
//! * **The accumulator is an automatic local `.sy` knows about**, not a
//!   file-scope object — folding a real memory store into `li r11,0` would drop
//!   the store.

use super::super::expr::parse_formals;
use super::super::{blk, BodyShape, Block};
use super::calls::eat_call_token;
use super::params::parse_params;
use crate::func::readers::{
    eat_byte, eat_opt_stmt_marker, is_fp_type, is_int4_type, is_ptr_to_4, read_token_var,
    read_type, read_varint,
};
use crate::func::bundle::{opt_word_at, opt_word_mode, OptWordMode};
use crate::func::IfCallJoin;

/// `26 <tok>` — the destination symbol push. Returns the token.
fn eat_designator(seg: &[u8], p: &mut usize, what: &'static str) -> Result<u32, Block> {
    if !eat_byte(seg, p, 0x26) {
        return Err(blk(seg, *p, what));
    }
    let (tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, what))?;
    *p += w;
    Ok(tok)
}

/// Consume a TYPE naming a width-4 **pointer** and return its pointee id.
fn eat_ptr4(seg: &[u8], p: &mut usize, what: &'static str) -> Result<u32, Block> {
    match read_type(seg, *p) {
        Some((tag, kind, id, w)) if is_ptr_to_4(tag, kind) => {
            *p += w;
            Ok(id)
        }
        _ => Err(blk(seg, *p, what)),
    }
}

/// Consume a TYPE naming a width-4 **signed integer**.
fn eat_int4(seg: &[u8], p: &mut usize, what: &'static str) -> Result<(), Block> {
    match read_type(seg, *p) {
        Some((tag, kind, _, w)) if is_int4_type(tag, kind) => {
            *p += w;
            Ok(())
        }
        _ => Err(blk(seg, *p, what)),
    }
}

/// Consume `54 <k>`, requiring the exact depth `k`.
///
/// The depths are pinned rather than merely decoded because they are the only
/// place the *bracing* of the source shows up in this stream, and a differently
/// braced body is a different block plan. `docs/IL_STMT_GRAMMAR.md` §1: `k` is
/// the number of scopes still open after the pop.
fn eat_close(seg: &[u8], p: &mut usize, k: u8, what: &'static str) -> Result<(), Block> {
    if !eat_byte(seg, p, 0x54) || !eat_byte(seg, p, k) {
        return Err(blk(seg, *p, what));
    }
    Ok(())
}

/// Consume `29 <tok>` — a label definition — and return the label token.
fn eat_label(seg: &[u8], p: &mut usize, what: &'static str) -> Result<u32, Block> {
    if !eat_byte(seg, p, 0x29) {
        return Err(blk(seg, *p, what));
    }
    let (tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, what))?;
    *p += w;
    Ok(tok)
}

/// Consume `<op> <tok>` for a transfer opcode and return the target label.
fn eat_transfer(seg: &[u8], p: &mut usize, op: u8, what: &'static str) -> Result<u32, Block> {
    if !eat_byte(seg, p, op) {
        return Err(blk(seg, *p, what));
    }
    let (tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, what))?;
    *p += w;
    Ok(tok)
}

/// `B9 <scrut> <enum/int TYPE> · 2C <int4 TYPE> <varint> · 33 <int4 TYPE> <K>` —
/// the scrutinee test's left operand and its literal, which every one of the
/// three tests in this body repeats verbatim.
///
/// The `2C` is the enum→int widening and is **required**: it is what makes the
/// three tests share one `cmpwi`, because they are three reads of one converted
/// value. A body whose tests convert differently is not this body.
fn eat_scrutinee_and_literal(
    seg: &[u8],
    p: &mut usize,
    scrut: u32,
    what: &'static str,
) -> Result<i32, Block> {
    if !eat_byte(seg, p, 0xB9) {
        return Err(blk(seg, *p, what));
    }
    let (tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, what))?;
    *p += w;
    if tok != scrut {
        return Err(blk(seg, *p, "ifjoin-test-not-the-scrutinee"));
    }
    eat_int4(seg, p, "ifjoin-test-scrut-type")?;
    // **The `2C` widening is OPTIONAL, and which way it goes is a source fact
    // with no instruction behind it.** An `enum` scrutinee carries an explicit
    // `2C <int4 TYPE> <varint>` — the enum→int conversion, which is a
    // reinterpret between two width-4 signed integers and emits nothing — and a
    // plain `int` scrutinee carries none. Requiring it refused every non-enum
    // spelling of this body (`fixtures/cpp/wcfg1_if_call_join.cpp` p2/p3/p4);
    // requiring its absence would refuse the workload's own. Both are admitted
    // and BOTH ARE GRADED: p0/p1 are the enum form, p2/p3/p4 the int form, and
    // the oracle compares all five.
    //
    // What is NOT optional is that all three tests spell it the same way — the
    // three reads are one converted value, which is why one `cmpwi` serves them,
    // and this function is called for each test with the same `scrut`.
    if eat_byte(seg, p, 0x2C) {
        eat_int4(seg, p, "ifjoin-test-widen-type")?;
        read_varint(seg, p).ok_or(blk(seg, *p, "ifjoin-test-widen-varint"))?;
    }
    if !eat_byte(seg, p, 0x33) {
        return Err(blk(seg, *p, "ifjoin-test-lit"));
    }
    eat_int4(seg, p, "ifjoin-test-lit-type")?;
    let k = read_varint(seg, p).ok_or(blk(seg, *p, "ifjoin-test-lit-varint"))?;
    if !(-0x8000..=0x7FFF).contains(&k) {
        // `cmpwi cr6,rA,K` has one 16-bit signed immediate field. A wider
        // literal is a `lis`/`ori` pair and a different block.
        return Err(blk(seg, *p, "ifjoin-test-lit-wide"));
    }
    Ok(k)
}

/// `26 <acc> · 33 <ptr4 TYPE> <K> · 32 <the same ptr4 TYPE> · 4B` — the
/// accumulator's literal store. Returns the literal.
fn eat_acc_literal_store(
    seg: &[u8],
    p: &mut usize,
    acc: u32,
    acc_type: u32,
    what: &'static str,
) -> Result<i32, Block> {
    if eat_designator(seg, p, what)? != acc {
        return Err(blk(seg, *p, "ifjoin-store-not-the-acc"));
    }
    if !eat_byte(seg, p, 0x33) {
        return Err(blk(seg, *p, what));
    }
    if eat_ptr4(seg, p, what)? != acc_type {
        return Err(blk(seg, *p, "ifjoin-store-lit-type"));
    }
    let k = read_varint(seg, p).ok_or(blk(seg, *p, what))?;
    if !eat_byte(seg, p, 0x32) {
        return Err(blk(seg, *p, what));
    }
    if eat_ptr4(seg, p, what)? != acc_type {
        return Err(blk(seg, *p, "ifjoin-store-type"));
    }
    if !eat_byte(seg, p, 0x4B) {
        return Err(blk(seg, *p, what));
    }
    Ok(k)
}

/// `26 <acc> · 26 <callee> · BD … · B9 <f> <float> 55 <float> · B9 <p> <ptr4>
/// 55 <ptr4> · 4C · 32 <ptr4> · 4B` — one arm's whole statement.
///
/// Returns the callee token. The argument region is required to name **exactly**
/// the two formals the caller passes in, in the stream order c2 emits them
/// (the FLOAT first — `.ex` lists a call's arguments in reverse source order,
/// which this recognizer matches rather than reorders).
fn eat_arm_call(
    seg: &[u8],
    p: &mut usize,
    acc: u32,
    acc_type: u32,
    fp_formal: u32,
    ptr_formal: u32,
    what: &'static str,
) -> Result<u32, Block> {
    if eat_designator(seg, p, what)? != acc {
        return Err(blk(seg, *p, "ifjoin-arm-not-the-acc"));
    }
    let callee = eat_designator(seg, p, "ifjoin-arm-callee")?;
    eat_call_token(seg, p)?;
    // Argument 1 in stream order: the FLOAT, which travels in the other register
    // file and therefore emits no setup word at all.
    if !eat_byte(seg, p, 0xB9) {
        return Err(blk(seg, *p, "ifjoin-arm-arg0"));
    }
    let (t0, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, "ifjoin-arm-arg0-tok"))?;
    *p += w;
    if t0 != fp_formal {
        return Err(blk(seg, *p, "ifjoin-arm-arg0-not-formal2"));
    }
    eat_float4(seg, p, "ifjoin-arm-arg0-type")?;
    if !eat_byte(seg, p, 0x55) {
        return Err(blk(seg, *p, "ifjoin-arm-arg0-sep"));
    }
    eat_float4(seg, p, "ifjoin-arm-arg0-septype")?;
    // Argument 2 in stream order: the POINTER, and it is the one word the entry
    // block hoisted.
    if !eat_byte(seg, p, 0xB9) {
        return Err(blk(seg, *p, "ifjoin-arm-arg1"));
    }
    let (t1, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, "ifjoin-arm-arg1-tok"))?;
    *p += w;
    if t1 != ptr_formal {
        return Err(blk(seg, *p, "ifjoin-arm-arg1-not-formal1"));
    }
    let a1 = eat_ptr4(seg, p, "ifjoin-arm-arg1-type")?;
    if !eat_byte(seg, p, 0x55) {
        return Err(blk(seg, *p, "ifjoin-arm-arg1-sep"));
    }
    if eat_ptr4(seg, p, "ifjoin-arm-arg1-septype")? != a1 {
        return Err(blk(seg, *p, "ifjoin-arm-arg1-septype"));
    }
    if !eat_byte(seg, p, 0x4C) {
        return Err(blk(seg, *p, "ifjoin-arm-arglist-close"));
    }
    if !eat_byte(seg, p, 0x32) {
        return Err(blk(seg, *p, "ifjoin-arm-store"));
    }
    if eat_ptr4(seg, p, "ifjoin-arm-store-type")? != acc_type {
        return Err(blk(seg, *p, "ifjoin-arm-store-type"));
    }
    if !eat_byte(seg, p, 0x4B) {
        return Err(blk(seg, *p, "ifjoin-arm-end"));
    }
    Ok(callee)
}

/// Consume a TYPE naming a **4-byte real** (`float`, never `double`).
///
/// A `double` argument is the same register file and a different narrowing rule,
/// and no cell of this class has been graded on one.
fn eat_float4(seg: &[u8], p: &mut usize, what: &'static str) -> Result<(), Block> {
    match read_type(seg, *p) {
        Some((tag, kind, _, w)) if is_fp_type(tag, kind) == Some(false) => {
            *p += w;
            Ok(())
        }
        _ => Err(blk(seg, *p, what)),
    }
}

/// **The recognizer.** `start` is the first byte after the body's own `53` and
/// any leading line marker; `lo` is the `4C 4F 11` body marker.
///
/// Non-committal in the sense every sibling production here is: it works on its
/// own cursor and returns `Err` on the first byte that is not its grammar, so a
/// body that declines still reports its dispatch arm's blocker and no census key
/// moves.
pub(crate) fn try_parse_if_call_join(
    seg: &[u8],
    start: usize,
    lo: usize,
    locals: &[u32],
    ptr_locals: &[u32],
) -> Result<BodyShape, Block> {
    // **THE MODE GATE LIVES HERE, IN THE PARSER — not in the emitter.**
    //
    // `/Ox` and `/O2` tail-duplicate a join block rather than sharing it behind
    // a `b`, so this class's twenty words are a `/O1` body and nothing else. The
    // emitter re-asserts that (`codegen::if_call_join`), but a gate that lived
    // ONLY there would make the census count these bodies in class while
    // `PortC2` refused them — an error term on the published coverage numerator,
    // which is exactly what `crates/c2-harness/tests/census_gate.rs` fails on and
    // what `docs/GAPS.md` §6 says to do about it: *move the gate into the IL
    // parser*. Asked FIRST, before any body byte is read, so the refusal cannot
    // depend on how far the walk got.
    //
    // `codegen::ptr_walk_loop` carries the same `/O1`-only clause and does NOT
    // have this half; it does not trip the cross-check today only because no
    // fixture puts its shape in front of the packed lane's profile. That is an
    // absence of evidence, and this comment is here so the next lane to touch
    // that class knows the fix is one call.
    if opt_word_mode(opt_word_at(seg)) != Some(OptWordMode::O1) {
        return Err(blk(seg, start, "ifjoin-not-o1"));
    }
    let params = parse_params(seg, lo)?;
    // **Exactly three formals**, and `parse_params` rather than `parse_formals`
    // so a non-static member function's `this` is counted: the park and the
    // hoist are a register assignment over the argument registers, and a shape
    // reading `params[1]` as r4 when r3 is `this` would emit the wrong `mr`.
    if params.len() != 3 || parse_formals(seg, lo)?.len() != 3 {
        return Err(blk(seg, start, "ifjoin-formals-not-3"));
    }
    let (scrut, ptr_formal, fp_formal) = (params[0], params[1], params[2]);

    let mut p = start;

    // ---- the entry block: `acc = K0` --------------------------------------
    let acc = eat_designator(seg, &mut p, "ifjoin-acc-designator")?;
    // **`ptr_locals`, not `locals`**, and the difference is the whole point of
    // there being two lists: `.sy` classifies an automatic width-4 data pointer
    // whose address is never taken into `ptr_locals` and an integer local into
    // `locals`, and this accumulator is the former. Requiring membership in both
    // — which is what a reader copying [`super::ptr_walk_loop`]'s accumulator
    // clause verbatim writes — refuses every real instance of this class, since
    // no token is in both lists.
    let _ = locals;
    if !ptr_locals.contains(&acc) {
        // An automatic width-4 data pointer `.sy` knows about, whose address is
        // never taken. A file-scope pointer would make this a real memory store
        // and folding it into `li r11,0` would drop it.
        return Err(blk(seg, p, "ifjoin-acc-not-a-ptr-local"));
    }
    if !eat_byte(seg, &mut p, 0x33) {
        return Err(blk(seg, p, "ifjoin-acc-init-lit"));
    }
    let acc_type = eat_ptr4(seg, &mut p, "ifjoin-acc-init-type")?;
    let acc_init = read_varint(seg, &mut p).ok_or(blk(seg, p, "ifjoin-acc-init-varint"))?;
    if !(-0x8000..=0x7FFF).contains(&acc_init) {
        return Err(blk(seg, p, "ifjoin-acc-init-wide"));
    }
    if !eat_byte(seg, &mut p, 0x32) {
        return Err(blk(seg, p, "ifjoin-acc-init-store"));
    }
    if eat_ptr4(seg, &mut p, "ifjoin-acc-init-storetype")? != acc_type {
        return Err(blk(seg, p, "ifjoin-acc-init-storetype"));
    }
    if !eat_byte(seg, &mut p, 0x4B) {
        return Err(blk(seg, p, "ifjoin-acc-init-end"));
    }

    // ---- the OUTER test: `if (s >= K1)`, exiting to Lexit ------------------
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "ifjoin-outer-scope"));
    }
    let k1 = eat_scrutinee_and_literal(seg, &mut p, scrut, "ifjoin-outer-test")?;
    // `23` is the relational this class was measured on and the only one it
    // takes. A different relation is a different branch bit AND a different
    // successor for the middle test, which shares this compare.
    if !eat_byte(seg, &mut p, 0x23) {
        return Err(blk(seg, p, "ifjoin-outer-rel-not-ge"));
    }
    let l_exit = eat_transfer(seg, &mut p, 0x38, "ifjoin-outer-branch")?;
    if !eat_byte(seg, &mut p, 0x53) || !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "ifjoin-outer-body-scopes"));
    }

    // ---- the MIDDLE test, in either spelling -------------------------------
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "ifjoin-mid-scope"));
    }
    let k1b = eat_scrutinee_and_literal(seg, &mut p, scrut, "ifjoin-mid-test")?;
    if k1b != k1 {
        // The whole reason one `cmpwi` serves two tests. Two different literals
        // are two compares and a different block plan.
        return Err(blk(seg, p, "ifjoin-mid-lit-differs"));
    }
    // `1F` (`==`) branches on FALSE, `20` (`!=`) branches on TRUE, and both name
    // the same successor — the `else` arm. c2 emits the same word for both,
    // because it deletes the empty then-block and inverts the sense once.
    let l_else1 = match seg.get(p) {
        Some(0x1F) => {
            p += 1;
            eat_transfer(seg, &mut p, 0x38, "ifjoin-mid-branch-eq")?
        }
        Some(0x20) => {
            p += 1;
            eat_transfer(seg, &mut p, 0x39, "ifjoin-mid-branch-ne")?
        }
        _ => return Err(blk(seg, p, "ifjoin-mid-rel")),
    };
    if !eat_byte(seg, &mut p, 0x53) || !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "ifjoin-mid-body-scopes"));
    }

    // ---- the DEAD arm: `acc = K0` again, which is why c2 emits no block -----
    eat_opt_stmt_marker(seg, &mut p);
    let dead = eat_acc_literal_store(seg, &mut p, acc, acc_type, "ifjoin-dead-store")?;
    if dead != acc_init {
        return Err(blk(seg, p, "ifjoin-dead-store-differs"));
    }
    eat_opt_stmt_marker(seg, &mut p);
    eat_close(seg, &mut p, 0x08, "ifjoin-dead-close-8")?;
    eat_close(seg, &mut p, 0x07, "ifjoin-dead-close-7")?;
    let l_join2 = eat_transfer(seg, &mut p, 0x3A, "ifjoin-dead-jump")?;
    if eat_label(seg, &mut p, "ifjoin-else1-label")? != l_else1 {
        return Err(blk(seg, p, "ifjoin-else1-label"));
    }
    if !eat_byte(seg, &mut p, 0x53) || !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "ifjoin-else1-scopes"));
    }

    // ---- the INNER test: `if (s >= K2)` ------------------------------------
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "ifjoin-inner-scope"));
    }
    let k2 = eat_scrutinee_and_literal(seg, &mut p, scrut, "ifjoin-inner-test")?;
    if !eat_byte(seg, &mut p, 0x23) {
        return Err(blk(seg, p, "ifjoin-inner-rel-not-ge"));
    }
    let l_else2 = eat_transfer(seg, &mut p, 0x38, "ifjoin-inner-branch")?;
    if !eat_byte(seg, &mut p, 0x53) || !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "ifjoin-inner-body-scopes"));
    }

    // ---- arm HI: `acc = hi(clip, t)` ---------------------------------------
    eat_opt_stmt_marker(seg, &mut p);
    let callee_hi = eat_arm_call(
        seg, &mut p, acc, acc_type, fp_formal, ptr_formal, "ifjoin-hi-arm",
    )?;
    eat_opt_stmt_marker(seg, &mut p);
    eat_close(seg, &mut p, 0x0B, "ifjoin-hi-close-b")?;
    eat_close(seg, &mut p, 0x0A, "ifjoin-hi-close-a")?;
    let l_join1 = eat_transfer(seg, &mut p, 0x3A, "ifjoin-hi-jump")?;
    if eat_label(seg, &mut p, "ifjoin-else2-label")? != l_else2 {
        return Err(blk(seg, p, "ifjoin-else2-label"));
    }
    if !eat_byte(seg, &mut p, 0x53) || !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "ifjoin-else2-scopes"));
    }

    // ---- arm LO: `acc = lo(clip, t)` ---------------------------------------
    eat_opt_stmt_marker(seg, &mut p);
    let callee_lo = eat_arm_call(
        seg, &mut p, acc, acc_type, fp_formal, ptr_formal, "ifjoin-lo-arm",
    )?;
    eat_opt_stmt_marker(seg, &mut p);
    eat_close(seg, &mut p, 0x0B, "ifjoin-lo-close-b")?;
    eat_close(seg, &mut p, 0x0A, "ifjoin-lo-close-a")?;

    // ---- the JOIN, and the wind-down of every scope ------------------------
    if eat_label(seg, &mut p, "ifjoin-join1-label")? != l_join1 {
        return Err(blk(seg, p, "ifjoin-join1-label"));
    }
    eat_close(seg, &mut p, 0x09, "ifjoin-join1-close")?;
    eat_opt_stmt_marker(seg, &mut p);
    eat_close(seg, &mut p, 0x08, "ifjoin-wind-8")?;
    eat_close(seg, &mut p, 0x07, "ifjoin-wind-7")?;
    if eat_label(seg, &mut p, "ifjoin-join2-label")? != l_join2 {
        return Err(blk(seg, p, "ifjoin-join2-label"));
    }
    eat_close(seg, &mut p, 0x06, "ifjoin-wind-6")?;
    eat_opt_stmt_marker(seg, &mut p);
    eat_close(seg, &mut p, 0x05, "ifjoin-wind-5")?;
    eat_opt_stmt_marker(seg, &mut p);
    eat_close(seg, &mut p, 0x04, "ifjoin-wind-4")?;
    if eat_label(seg, &mut p, "ifjoin-exit-label")? != l_exit {
        return Err(blk(seg, p, "ifjoin-exit-label"));
    }
    eat_close(seg, &mut p, 0x03, "ifjoin-wind-3")?;

    // ---- `return acc` ------------------------------------------------------
    if !eat_byte(seg, &mut p, 0xB9) {
        return Err(blk(seg, p, "ifjoin-result-load"));
    }
    let (t, w) = read_token_var(seg, p).ok_or(blk(seg, p, "ifjoin-result-tok"))?;
    p += w;
    if t != acc {
        return Err(blk(seg, p, "ifjoin-result-not-the-acc"));
    }
    if eat_ptr4(seg, &mut p, "ifjoin-result-type")? != acc_type {
        return Err(blk(seg, p, "ifjoin-result-type"));
    }
    if !eat_byte(seg, &mut p, 0x41) {
        return Err(blk(seg, p, "ifjoin-result-annotation"));
    }
    if eat_ptr4(seg, &mut p, "ifjoin-result-anntype")? != acc_type {
        return Err(blk(seg, p, "ifjoin-result-anntype"));
    }
    let l_epi = eat_transfer(seg, &mut p, 0x3A, "ifjoin-result-jump")?;
    eat_opt_stmt_marker(seg, &mut p);
    eat_close(seg, &mut p, 0x02, "ifjoin-wind-2")?;
    if eat_label(seg, &mut p, "ifjoin-epilogue-label")? != l_epi {
        return Err(blk(seg, p, "ifjoin-epilogue-label"));
    }
    // The function tail. Landing exactly on it is the whole acceptance claim:
    // a walk that ends anywhere else consumed a byte it did not understand.
    // PROV[O] the seven-byte `.ex` function tail, read off captures. See `alloc_init_or_fail::FN_TAIL`.
    const FN_TAIL: [u8; 7] = [0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00];
    if seg.get(p..p + FN_TAIL.len()) != Some(&FN_TAIL[..]) {
        return Err(blk(seg, p, "ifjoin-not-the-function-tail"));
    }
    // Every label in the body must be distinct: two of them aliasing would make
    // two different successors one block and the emitted branch displacements
    // would be right for a program this is not.
    let labels = [l_exit, l_else1, l_else2, l_join1, l_join2, l_epi];
    for i in 0..labels.len() {
        for j in i + 1..labels.len() {
            if labels[i] == labels[j] {
                return Err(blk(seg, p, "ifjoin-labels-alias"));
            }
        }
    }

    Ok(BodyShape::IfCallJoin(IfCallJoin {
        params,
        k1,
        k2,
        acc_init,
        callee_hi_tok: callee_hi,
        callee_lo_tok: callee_lo,
    }))
}
