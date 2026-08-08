//! **W-EXTDATA — a `||` guard chain whose block is SUNK to the end of the
//! function and TAIL-MERGED with a second error block, around one call that
//! takes the ADDRESS OF A FUNCTION as its first argument.**
//!
//! ```cpp
//!   int f(P0 a0, P1 a1, P2 a2, P3 a3, P4 a4, P5 a5) {
//!       int r;
//!       if (a2 == 0 || a0 == 0 || a1 == 0) {          // three tests, ONE block
//!           *e() = K_GUARD;  v();  return R_FAIL;
//!       }
//!       r = h(&g, a0, a1, a2, a3, a4, a5);            // SEVEN arguments
//!       if (r < 0)  *a0 = 0;                          // a HALFWORD store
//!       if (r != S) return r;
//!       *e() = K_RANGE;  v();  return R_FAIL;         // the SAME two calls
//!   }
//! ```
//!
//! This is `src/xdk/LIBCMT/vswprnc.cpp`'s `_vswprintf_s_l`, a FRONTIER TU with
//! exactly one emitted function — so the TU converts on this class or on none.
//!
//! ## Why a TRANSCRIPTION and not a general `cflow-if-n` lowering
//!
//! The same argument [`super::if_call_join`] makes, and it is
//! `docs/ARCHITECTURE_SEAMS.md` §7's: a general control-flow lowering forces a
//! block IR plus a value merge at the join, sequenced with the frame/liveness
//! spine, and that restructure has never been sized. What ships here is thirty
//! words of one named function class, `/O1` only, `NotImplemented` outside.
//!
//! **Accepting this shape is not a claim about `cflow-if-n` as a class.** It
//! takes ONE of the frontier's `cflow-if-n` functions.
//!
//! ## What the reference emits, and the five things about it a lowering gets wrong
//!
//! Read off the real obj at the workload's own flags
//! (`work/w-extdata/ref/vswprnc/dis.txt`, committed) and decoded token by token
//! in `work/w-extdata/VSWPRNC_BODY.md`:
//!
//! ```text
//!   mflr/stw/std r31/stwu -96   FrameLayout{saved_gprs:1} — byte for byte
//!   mr    r31,r3           (1) PARK the store target: it is read AFTER two calls
//!   mr    r9,r8            (2) the LAST rotate step, HOISTED above every branch
//!   cmplwi cr6,r5,0        (3) ┐ three guards, one target — the `||` chain is
//!   bt    26,->Lerr            │ THREE branches, not a computed boolean, and
//!   cmplwi cr6,r3,0            │ the tested formals are 2, 0, 1 IN THAT ORDER
//!   bt    26,->Lerr            │
//!   cmplwi cr6,r4,0            │
//!   bt    26,->Lerr            ┘
//!   mr    r8,r7            (4) ┐ the 5-deep rotate, descending …
//!   mr    r7,r6                │
//!   lis   r11,0            (5) │ … with the REFHI INTERLEAVED into it, at word
//!   mr    r6,r5                │ 14 of the body — WR1's "first word" is false
//!   mr    r5,r4                │ here by thirteen words
//!   mr    r4,r3                ┘
//!   addi  r3,r11,0         (6) the REFLO
//!   bl    <helper>             REL24
//!   cmpwi cr0,r3,0         (7) `r < 0` on **cr0** …
//!   bf    0,->Lskip
//!   li    r11,0
//!   sth   r11,0(r31)       (8) a HALFWORD store through the parked formal
//!   cmpwi cr6,r3,S         (9) … and `r != S` on **cr6**
//!   bf    26,->epilogue        with the result already in r3
//!   bl    <errno>              REL24   ┐ the RANGE arm
//!   li    r11,K_RANGE                  │
//!   b     ->Ltail        (10)          ┘
//!  Lerr:
//!   bl    <errno>              REL24   ┐ the GUARD arm — and the `||` target,
//!   li    r11,K_GUARD                  ┘ SUNK here from the top of the body
//!  Ltail:
//!   stw   r11,0(r3)            ┐
//!   bl    <invalid>            │ the MERGED TAIL both error arms share
//!   li    r3,R_FAIL            ┘
//!   addi/lwz/mtlr/ld r31/blr
//! ```
//!
//! 1. **The `||` chain is three branches to one block.** A lowering that
//!    materialised the disjunction would emit `or` and one branch: the right
//!    program, and every displacement after word 7 wrong.
//! 2. **c2 SINKS the guard block to the END.** In the IL it is `L47`,
//!    *textually before* the call; in `.text` it is the second-to-last block.
//!    IL order is not block order here, so nothing may be derived from it.
//! 3. **The two error arms are TAIL-MERGED on four words**, the range arm
//!    reaching the shared tail through the `b` at (10). They differ in exactly
//!    one `li` immediate. This is board **#1400**'s finding pointing the other
//!    way: there the sharing looked like a peephole and deleting it was the
//!    defect; here the sharing *is* the class and duplicating it would be four
//!    words long and still link.
//! 4. **Two condition registers.** `r < 0` reads **cr0** and `r != S` reads
//!    **cr6**. Nothing in the source distinguishes them; a class that used one CR
//!    for both emits the right program and the wrong `bf` operand.
//! 5. **The first argument is a FUNCTION'S ADDRESS**, so its symbol is
//!    `Type 0x0020` and its relocation is a REFHI/REFLO — not a REL24, and not
//!    the `Type 0x0000` every WR1 data symbol has had. It is the one argument
//!    that is a `26` designator where the other six are `B9` reads.
//!
//! ## The fence
//!
//! Every clause below is required literally, and each names the measurement
//! behind it rather than a preference.
//!
//! * **`/O1` only, asked FIRST, in the PARSER.** Board **#1638** has fired
//!   twice — a mode clause that lives only in the emitter makes the census count
//!   bodies in class that `PortC2` refuses. `census_gate.rs` is the cross-check.
//! * **Exactly SIX formals**, and this is the clause that keeps the class a
//!   transcription. The rotate is 5 steps below the guards and the `lis` sits
//!   after the second of them; with one witness there is no way to tell "after
//!   the second" from "three before the last", so the arity is pinned and a
//!   seven-formal body is refused rather than guessed at.
//! * **The call takes `formals` in reverse, then the function address.** Seven
//!   arguments, `params[5] … params[0]` in stream order and the `26 <fn>` last:
//!   that is what makes the rotate exactly this permutation.
//! * **Both error arms call the SAME two functions in the same order.** The tail
//!   merge is only legal because they do; two different callees are two tails.
//! * **The `< 0` test and the `!= S` test name the CALL RESULT**, the local the
//!   call stored to — not a formal, and not each other's operand.
//! * **The halfword store's base is `params[0]`**, the formal parked in r31.
//! * **Every label distinct.** Two aliasing labels are one block, and every
//!   displacement after the alias would be right for a program this is not.

use super::super::expr::parse_formals;
use super::super::{blk, BodyShape, Block};
use super::calls::eat_call_token;
use super::params::parse_params;
use crate::func::bundle::{opt_word_at, opt_word_mode, OptWordMode};
use crate::func::readers::{
    eat_byte, eat_opt_stmt_marker, is_int4_type, is_ptr_to_4, read_token_var, read_type,
    read_varint,
};
use crate::func::GuardChainSharedTail;

/// The number of formals this class is graded on. See the fence: the `lis`'s
/// position inside the rotate cannot be separated from the arity at n = 1.
const FORMALS: usize = 6;

/// Consume any TYPE and discard it.
///
/// Used where the type is a *pointer whose pointee this class never touches* —
/// the formals' own types, which reach the emitted body only as "one word in an
/// argument register". Where a type decides an instruction (the `int` of the
/// result, the halfword of the store) it is pinned instead.
fn eat_any_type(seg: &[u8], p: &mut usize, what: &'static str) -> Result<(u8, u8, u32), Block> {
    match read_type(seg, *p) {
        Some((tag, kind, id, w)) => {
            *p += w;
            Ok((tag, kind, id))
        }
        None => Err(blk(seg, *p, what)),
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

/// `26 <tok>` — a symbol push. Returns the token.
fn eat_designator(seg: &[u8], p: &mut usize, what: &'static str) -> Result<u32, Block> {
    if !eat_byte(seg, p, 0x26) {
        return Err(blk(seg, *p, what));
    }
    let (tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, what))?;
    *p += w;
    Ok(tok)
}

/// True for a TYPE whose zero-compare c2 emits as **`cmplwi`** — an unsigned
/// width-4 integer or a width-4 pointer.
///
/// **This is the clause that keeps the three guards `cmplwi` and not `cmpwi`.**
/// The emitter has one `cmplwi` per guard and no way to vary it, so a SIGNED
/// formal tested against 0 would be the right program with three wrong words —
/// board #1706's rule (anything the emitter cannot vary must be refused by the
/// reader) on a third axis. `is_int4_type` cannot serve: it admits both signs
/// deliberately (its own doc lists `86 42 22` unsigned long beside `86 41 74`
/// int), so the low nibble is read here — 1 signed, 2 unsigned, 3 pointer.
fn is_unsigned_or_ptr4(tag: u8, kind: u8) -> bool {
    is_ptr_to_4(tag, kind) || (is_int4_type(tag, kind) && (kind & 0x0F) == 0x2)
}

/// True for a TYPE that is a **signed** width-4 integer — the one whose
/// zero-compare is `cmpwi`.
fn is_signed_int4(tag: u8, kind: u8) -> bool {
    is_int4_type(tag, kind) && (kind & 0x0F) == 0x1
}

/// `B9 <tok> <TYPE>` — a value read. Returns the token and the type's two
/// discriminating bytes, because for this class the type decides an instruction.
fn eat_load(seg: &[u8], p: &mut usize, what: &'static str) -> Result<(u32, u8, u8), Block> {
    if !eat_byte(seg, p, 0xB9) {
        return Err(blk(seg, *p, what));
    }
    let (tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, what))?;
    *p += w;
    let (tag, kind, _) = eat_any_type(seg, p, what)?;
    Ok((tok, tag, kind))
}

/// `29 <tok>` — a label definition.
fn eat_label(seg: &[u8], p: &mut usize, what: &'static str) -> Result<u32, Block> {
    if !eat_byte(seg, p, 0x29) {
        return Err(blk(seg, *p, what));
    }
    let (tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, what))?;
    *p += w;
    Ok(tok)
}

/// `<op> <tok>` for a transfer opcode. Returns the target label.
fn eat_transfer(seg: &[u8], p: &mut usize, op: u8, what: &'static str) -> Result<u32, Block> {
    if !eat_byte(seg, p, op) {
        return Err(blk(seg, *p, what));
    }
    let (tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, what))?;
    *p += w;
    Ok(tok)
}

/// Consume `54 <k>`, requiring the exact depth `k`.
///
/// The depths are pinned rather than merely decoded for [`super::if_call_join`]'s
/// reason: they are the only place the *bracing* of the source shows up in this
/// stream, and a differently braced body is a different block plan.
fn eat_close(seg: &[u8], p: &mut usize, k: u8, what: &'static str) -> Result<(), Block> {
    eat_opt_stmt_marker(seg, p);
    if !eat_byte(seg, p, 0x54) || !eat_byte(seg, p, k) {
        return Err(blk(seg, *p, what));
    }
    Ok(())
}

/// `33 <int TYPE> <varint>` — an `int` literal that has to fit `simm16`, because
/// every one of them lands in a `li`/`cmpwi` immediate field.
fn eat_int_lit(seg: &[u8], p: &mut usize, what: &'static str) -> Result<i32, Block> {
    if !eat_byte(seg, p, 0x33) {
        return Err(blk(seg, *p, what));
    }
    eat_int4(seg, p, what)?;
    let k = read_varint(seg, p).ok_or(blk(seg, *p, what))?;
    if !(-0x8000..=0x7FFF).contains(&k) {
        return Err(blk(seg, *p, "gcst-literal-wider-than-simm16"));
    }
    Ok(k)
}

/// One guard test: `B9 <formal> <T> · 33 <T> 00 · 1F · <39|38> <label>`.
///
/// Returns `(formal index, label)`. The relation is `==` and the literal is 0
/// for every one of the three: that is what makes each guard exactly one
/// `cmplwi crf,rX,0`, and a non-zero literal or a different relation is a
/// different instruction.
fn eat_guard(
    seg: &[u8],
    p: &mut usize,
    params: &[u32],
    brop: u8,
    what: &'static str,
) -> Result<(usize, u32), Block> {
    let (tok, tag, kind) = eat_load(seg, p, what)?;
    if !is_unsigned_or_ptr4(tag, kind) {
        return Err(blk(seg, *p, "gcst-guard-is-signed-so-c2-emits-cmpwi"));
    }
    let ix = params
        .iter()
        .position(|&t| t == tok)
        .ok_or(blk(seg, *p, "gcst-guard-not-a-formal"))?;
    if !eat_byte(seg, p, 0x33) {
        return Err(blk(seg, *p, what));
    }
    eat_any_type(seg, p, what)?;
    let k = read_varint(seg, p).ok_or(blk(seg, *p, what))?;
    if k != 0 {
        return Err(blk(seg, *p, "gcst-guard-literal-not-zero"));
    }
    if !eat_byte(seg, p, 0x1F) {
        return Err(blk(seg, *p, "gcst-guard-rel-not-eq"));
    }
    let l = eat_transfer(seg, p, brop, what)?;
    Ok((ix, l))
}

/// `26 <fn> · BD … · 4C · 33 <int> <k> · 32 <int> 4B` — `*<fn>() = <k>`, the
/// store through a nullary call's returned pointer. Returns `(callee, k)`.
fn eat_errno_store(seg: &[u8], p: &mut usize, what: &'static str) -> Result<(u32, i32), Block> {
    eat_opt_stmt_marker(seg, p);
    let callee = eat_designator(seg, p, what)?;
    eat_call_token(seg, p)?;
    if !eat_byte(seg, p, 0x4C) {
        return Err(blk(seg, *p, "gcst-errno-call-takes-arguments"));
    }
    let k = eat_int_lit(seg, p, what)?;
    if !eat_byte(seg, p, 0x32) {
        return Err(blk(seg, *p, what));
    }
    eat_int4(seg, p, what)?;
    if !eat_byte(seg, p, 0x4B) {
        return Err(blk(seg, *p, what));
    }
    Ok((callee, k))
}

/// `26 <fn> · BD … · 4C · 4B` — a nullary call in statement position.
fn eat_void_call(seg: &[u8], p: &mut usize, what: &'static str) -> Result<u32, Block> {
    eat_opt_stmt_marker(seg, p);
    let callee = eat_designator(seg, p, what)?;
    eat_call_token(seg, p)?;
    if !eat_byte(seg, p, 0x4C) {
        return Err(blk(seg, *p, "gcst-void-call-takes-arguments"));
    }
    if !eat_byte(seg, p, 0x4B) {
        return Err(blk(seg, *p, what));
    }
    Ok(callee)
}

/// `33 <int> <k> · 41 <int> · 3A <epilogue>` — `return <k>;`.
fn eat_return_lit(seg: &[u8], p: &mut usize, what: &'static str) -> Result<(i32, u32), Block> {
    eat_opt_stmt_marker(seg, p);
    let k = eat_int_lit(seg, p, what)?;
    if !eat_byte(seg, p, 0x41) {
        return Err(blk(seg, *p, what));
    }
    eat_int4(seg, p, what)?;
    let l = eat_transfer(seg, p, 0x3A, what)?;
    Ok((k, l))
}

/// The whole error block: `*e() = K; v(); return R;`. Both arms are this, which
/// is why the tail merges — so both are parsed by ONE function and the caller
/// compares the three results rather than trusting that they agree.
fn eat_error_block(
    seg: &[u8],
    p: &mut usize,
    what: &'static str,
) -> Result<(u32, i32, u32, i32, u32), Block> {
    let (errno, k) = eat_errno_store(seg, p, what)?;
    let invalid = eat_void_call(seg, p, what)?;
    let (r, l) = eat_return_lit(seg, p, what)?;
    Ok((errno, k, invalid, r, l))
}

/// **The recognizer.** `start` is the first byte after the body's own `53`, the
/// leading line markers and the `if`'s own `53` — all eaten by `eat_scopes`, so
/// the cursor arrives on the first guard's `B9`. `lo` is the `4C 4F 11` marker.
///
/// Non-committal in the sense every sibling production here is: it works on its
/// own cursor and returns `Err` on the first byte that is not its grammar, so a
/// body that declines still reports its dispatch arm's blocker and no census key
/// moves.
pub(crate) fn try_parse_guard_chain_shared_tail(
    seg: &[u8],
    start: usize,
    lo: usize,
) -> Result<BodyShape, Block> {
    // **THE MODE GATE LIVES HERE, IN THE PARSER — not in the emitter.**
    //
    // Board **#1638**, which has now fired twice (w-cfgclass §5.3, w-data §6.5).
    // A gate that lived only in `codegen` would make the census count this body
    // in class while `PortC2` refused it — an error term on the published
    // coverage numerator, and exactly what `census_gate.rs` fails on. Asked
    // FIRST, before any body byte is read, so the refusal cannot depend on how
    // far the walk got.
    //
    // `/Ox` and `/O2` are refused rather than modeled for this class's own
    // reason as well as the family's: the tail merge at (10) is a shared block
    // behind a `b`, which is precisely the shape W10 measured tail-duplicating
    // above `/O1` (board row X-b, and `super::if_call_join` carries the same
    // clause for the same block kind).
    if opt_word_mode(opt_word_at(seg)) != Some(OptWordMode::O1) {
        return Err(blk(seg, start, "gcst-not-o1"));
    }
    // `parse_params` rather than `parse_formals` so a non-static member
    // function's `this` would be counted — the rotate is a register assignment
    // over the argument registers and a shape reading `params[0]` as r3 when r3
    // is `this` would emit the wrong `mr`. Both are asked, and required to
    // agree, so a member function is refused here rather than mis-rotated.
    let params = parse_params(seg, lo)?;
    if params.len() != FORMALS || parse_formals(seg, lo)?.len() != FORMALS {
        return Err(blk(seg, start, "gcst-formals-not-6"));
    }

    let mut p = start;

    // ---- the `||` chain: two `brtrue`s and a `brfalse` ---------------------
    //
    // This is the byte that separates this production from every neighbour in
    // its dispatch arm. `cond_tail`, `guarded_seq` and `early_return_seq` all
    // consume a `38` (brfalse) after the relation; this one consumes a `39`
    // (brtrue), because a `||` short-circuits INTO its block rather than around
    // it. None of them can reach this body and it cannot take one of theirs.
    let (g0, l_err) = eat_guard(seg, &mut p, &params, 0x39, "gcst-guard-0")?;
    let (g1, l_err1) = eat_guard(seg, &mut p, &params, 0x39, "gcst-guard-1")?;
    if l_err1 != l_err {
        return Err(blk(seg, p, "gcst-guard-1-different-block"));
    }
    let (g2, l_cont) = eat_guard(seg, &mut p, &params, 0x38, "gcst-guard-2")?;
    if l_cont == l_err {
        return Err(blk(seg, p, "gcst-guard-2-same-block"));
    }
    if g0 == g1 || g1 == g2 || g0 == g2 {
        // Three tests of two formals is two `cmplwi`s in c2's hands, not three.
        return Err(blk(seg, p, "gcst-guards-test-the-same-formal-twice"));
    }

    // ---- the GUARD arm, which `.text` sinks to the end ---------------------
    if eat_label(seg, &mut p, "gcst-err-label")? != l_err {
        return Err(blk(seg, p, "gcst-err-label"));
    }
    if !eat_byte(seg, &mut p, 0x53) || !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "gcst-err-scopes"));
    }
    let (errno_a, k_guard, invalid_a, ret_a, l_epi_a) =
        eat_error_block(seg, &mut p, "gcst-err-block")?;
    eat_close(seg, &mut p, 0x05, "gcst-err-close-5")?;
    eat_close(seg, &mut p, 0x04, "gcst-err-close-4")?;
    if eat_label(seg, &mut p, "gcst-cont-label")? != l_cont {
        return Err(blk(seg, p, "gcst-cont-label"));
    }
    eat_close(seg, &mut p, 0x03, "gcst-cont-close-3")?;

    // ---- the call: SEVEN arguments, the last of which is a function address --
    eat_opt_stmt_marker(seg, &mut p);
    let result = eat_designator(seg, &mut p, "gcst-result-designator")?;
    if params.contains(&result) {
        // The call's destination is a local, not one of the formals: the formals
        // are all live across the call as its own arguments.
        return Err(blk(seg, p, "gcst-result-is-a-formal"));
    }
    let helper = eat_designator(seg, &mut p, "gcst-helper-designator")?;
    eat_call_token(seg, &mut p)?;
    // `.ex` lists a call's arguments in REVERSE source order, so the formals
    // arrive last-to-first. Required literally: this permutation is the rotate.
    for (n, want) in params.iter().rev().enumerate() {
        let what = "gcst-call-arg-not-the-formal";
        let (tok, _, _) = eat_load(seg, &mut p, what)?;
        if tok != *want {
            return Err(blk(seg, p, what));
        }
        if !eat_byte(seg, &mut p, 0x55) {
            return Err(blk(seg, p, "gcst-call-arg-sep"));
        }
        eat_any_type(seg, &mut p, "gcst-call-arg-septype")?;
        let _ = n;
    }
    // Argument 7 in stream order, i.e. the FIRST in source order: a `26`
    // designator and not a `B9` read. This is the whole of the REFHI/REFLO in
    // the emitted body, and the reason its symbol is `Type 0x0020`.
    let fn_addr = eat_designator(seg, &mut p, "gcst-fnaddr-designator")?;
    if !eat_byte(seg, &mut p, 0x2C) {
        // The function-to-function-pointer decay. It emits no instruction, and
        // requiring it is what tells this argument from a data symbol's address.
        return Err(blk(seg, p, "gcst-fnaddr-no-decay"));
    }
    eat_any_type(seg, &mut p, "gcst-fnaddr-decay-type")?;
    read_varint(seg, &mut p).ok_or(blk(seg, p, "gcst-fnaddr-decay-varint"))?;
    if !eat_byte(seg, &mut p, 0x55) {
        return Err(blk(seg, p, "gcst-fnaddr-sep"));
    }
    eat_any_type(seg, &mut p, "gcst-fnaddr-septype")?;
    if !eat_byte(seg, &mut p, 0x4C) {
        return Err(blk(seg, p, "gcst-call-arglist-close"));
    }
    if !eat_byte(seg, &mut p, 0x32) {
        return Err(blk(seg, p, "gcst-call-store"));
    }
    eat_int4(seg, &mut p, "gcst-call-store-type")?;
    if !eat_byte(seg, &mut p, 0x4B) {
        return Err(blk(seg, p, "gcst-call-store-end"));
    }

    // ---- `if (r < 0) *a0 = 0;` ---------------------------------------------
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "gcst-neg-scope"));
    }
    let (t, tag, kind) = eat_load(seg, &mut p, "gcst-neg-test")?;
    if t != result {
        return Err(blk(seg, p, "gcst-neg-test-not-the-result"));
    }
    if !is_signed_int4(tag, kind) {
        // `r < 0` on an UNSIGNED result is a constant-false test c2 folds away;
        // the emitted `cmpwi cr0,r3,0` is a signed compare and nothing else.
        return Err(blk(seg, p, "gcst-result-is-unsigned"));
    }
    if eat_int_lit(seg, &mut p, "gcst-neg-lit")? != 0 {
        return Err(blk(seg, p, "gcst-neg-lit-not-zero"));
    }
    if !eat_byte(seg, &mut p, 0x22) {
        return Err(blk(seg, p, "gcst-neg-rel-not-lt"));
    }
    let l_skip = eat_transfer(seg, &mut p, 0x38, "gcst-neg-branch")?;
    if !eat_byte(seg, &mut p, 0x53) || !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "gcst-neg-scopes"));
    }
    // `*a0 = 0` — the base is the formal parked in r31, and the store's TYPE is
    // what makes it an `sth`. Pinned as "not the `int` the rest of this body
    // uses": a width-4 store here is a `stw` and one different word.
    eat_opt_stmt_marker(seg, &mut p);
    if eat_load(seg, &mut p, "gcst-store-base")?.0 != params[0] {
        return Err(blk(seg, p, "gcst-store-base-not-formal-0"));
    }
    if !eat_byte(seg, &mut p, 0x33) {
        return Err(blk(seg, p, "gcst-store-lit"));
    }
    let (stag, skind, sid) = eat_any_type(seg, &mut p, "gcst-store-littype")?;
    if is_int4_type(stag, skind) {
        return Err(blk(seg, p, "gcst-store-is-a-word-not-a-halfword"));
    }
    if read_varint(seg, &mut p).ok_or(blk(seg, p, "gcst-store-varint"))? != 0 {
        return Err(blk(seg, p, "gcst-store-value-not-zero"));
    }
    if !eat_byte(seg, &mut p, 0x32) {
        return Err(blk(seg, p, "gcst-store-op"));
    }
    let (ttag, tkind, tid) = eat_any_type(seg, &mut p, "gcst-store-type")?;
    if (ttag, tkind, tid) != (stag, skind, sid) {
        return Err(blk(seg, p, "gcst-store-type-differs"));
    }
    if !eat_byte(seg, &mut p, 0x4B) {
        return Err(blk(seg, p, "gcst-store-end"));
    }
    eat_close(seg, &mut p, 0x05, "gcst-neg-close-5")?;
    eat_close(seg, &mut p, 0x04, "gcst-neg-close-4")?;
    if eat_label(seg, &mut p, "gcst-skip-label")? != l_skip {
        return Err(blk(seg, p, "gcst-skip-label"));
    }
    eat_close(seg, &mut p, 0x03, "gcst-skip-close-3")?;

    // ---- `if (r != S) return r;` -------------------------------------------
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "gcst-sent-scope"));
    }
    if eat_load(seg, &mut p, "gcst-sent-test")?.0 != result {
        return Err(blk(seg, p, "gcst-sent-test-not-the-result"));
    }
    let sentinel = eat_int_lit(seg, &mut p, "gcst-sent-lit")?;
    if !eat_byte(seg, &mut p, 0x20) {
        return Err(blk(seg, p, "gcst-sent-rel-not-ne"));
    }
    let l_tail = eat_transfer(seg, &mut p, 0x38, "gcst-sent-branch")?;
    if !eat_byte(seg, &mut p, 0x53) || !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "gcst-sent-scopes"));
    }
    eat_opt_stmt_marker(seg, &mut p);
    if eat_load(seg, &mut p, "gcst-ret-result")?.0 != result {
        return Err(blk(seg, p, "gcst-ret-not-the-result"));
    }
    if !eat_byte(seg, &mut p, 0x41) {
        return Err(blk(seg, p, "gcst-ret-annotation"));
    }
    eat_int4(seg, &mut p, "gcst-ret-anntype")?;
    let l_epi_b = eat_transfer(seg, &mut p, 0x3A, "gcst-ret-jump")?;
    eat_close(seg, &mut p, 0x05, "gcst-sent-close-5")?;
    eat_close(seg, &mut p, 0x04, "gcst-sent-close-4")?;
    if eat_label(seg, &mut p, "gcst-tail-label")? != l_tail {
        return Err(blk(seg, p, "gcst-tail-label"));
    }
    eat_close(seg, &mut p, 0x03, "gcst-tail-close-3")?;

    // ---- the RANGE arm, which is the same block with a different literal ----
    let (errno_b, k_range, invalid_b, ret_b, l_epi_c) =
        eat_error_block(seg, &mut p, "gcst-range-block")?;
    // **The tail merge is only legal because the two arms agree**, and this is
    // where that is established rather than assumed. Four words are emitted once
    // and reached from both; two different callees, two different return values
    // or two different epilogue targets are two tails and a different body.
    if errno_a != errno_b {
        return Err(blk(seg, p, "gcst-arms-call-different-errno"));
    }
    if invalid_a != invalid_b {
        return Err(blk(seg, p, "gcst-arms-call-different-reporter"));
    }
    if ret_a != ret_b {
        return Err(blk(seg, p, "gcst-arms-return-different-values"));
    }
    if k_guard == k_range {
        // Equal literals would make the two arms byte-identical, and c2 would
        // have merged them completely — a shorter body this class does not emit.
        return Err(blk(seg, p, "gcst-arms-share-their-literal"));
    }

    // ---- the wind-down ------------------------------------------------------
    eat_close(seg, &mut p, 0x02, "gcst-wind-2")?;
    if eat_label(seg, &mut p, "gcst-epilogue-label")? != l_epi_a {
        return Err(blk(seg, p, "gcst-epilogue-label"));
    }
    if l_epi_b != l_epi_a || l_epi_c != l_epi_a {
        // Three `return`s, one epilogue. Two epilogues is two blocks.
        return Err(blk(seg, p, "gcst-returns-reach-different-epilogues"));
    }
    // The function tail. Landing exactly on it is the whole acceptance claim: a
    // walk that ends anywhere else consumed a byte it did not understand.
    const FN_TAIL: [u8; 7] = [0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00];
    if seg.get(p..p + FN_TAIL.len()) != Some(&FN_TAIL[..]) {
        return Err(blk(seg, p, "gcst-not-the-function-tail"));
    }

    // Every label distinct: two of them aliasing would make two different
    // successors one block, and every displacement after the alias would be
    // right for a program this is not.
    let labels = [l_err, l_cont, l_skip, l_tail, l_epi_a];
    for i in 0..labels.len() {
        for j in i + 1..labels.len() {
            if labels[i] == labels[j] {
                return Err(blk(seg, p, "gcst-labels-alias"));
            }
        }
    }
    // The four names must be four names. A body calling the same function as
    // both `errno` and `helper` relocates two sites against one symbol, which is
    // a different symbol table.
    let names = [helper, fn_addr, errno_a, invalid_a];
    for i in 0..names.len() {
        for j in i + 1..names.len() {
            if names[i] == names[j] {
                return Err(blk(seg, p, "gcst-callees-alias"));
            }
        }
    }

    Ok(BodyShape::GuardChainSharedTail(GuardChainSharedTail {
        params,
        guard_ix: [g0, g1, g2],
        helper_tok: helper,
        fn_addr_tok: fn_addr,
        errno_tok: errno_a,
        invalid_tok: invalid_a,
        k_guard,
        k_range,
        sentinel,
        ret_fail: ret_a,
    }))
}
