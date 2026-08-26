//! **W-XTEA3 — the XTEA round loop.** `?Encipher@XTEABlockEncrypter@@AAA_K_KPAI@Z`,
//! one hundred and sixteen bytes / twenty-nine words, the largest of the three
//! bodies still blocking `src/system/utl/EncryptXTEA.cpp` after `w-xtea2`, and
//! the one `w-xtea` #2339 priced at `≥ 9` mechanisms.
//!
//! ```cpp
//!   unsigned long long XTEABlockEncrypter::Encipher(unsigned long long nonce,
//!                                                   unsigned int *key) {
//!       unsigned long v1 = nonce & 0xFFFFFFFF;
//!       unsigned long v2 = nonce >> 32;
//!       unsigned int  sum = 0;
//!       for (int i = 0; i < 4; i++) {
//!           v1 += (v2 + (v2 << 4 ^ v2 >> 5)) ^ sum + key[sum & 3];
//!           sum += 0x9E3779B9;
//!           v2 += (v1 + (v1 << 4 ^ v1 >> 5)) ^ sum + key[(sum >> 11) & 3];
//!       }
//!       return ((unsigned long long)v2 << 32) | ((unsigned long long)v1 & 0xFFFFFFFF);
//!   }
//! ```
//!
//! # Why this is a TRANSCRIPTION and says so
//!
//! Twenty of the twenty-nine words are the loop body, software-pipelined: the
//! `addis`/`addi` pair that materialises `0x9E3779B9` is split **around** an
//! `xor` that does not depend on it, and the second half-round's index
//! computation (`rlwinm r7,r11,23,28,29`) is hoisted above the first half's
//! last use of r11. No pass in this port derives that order, and `docs/CEILING.md`
//! §10 is explicit that the project's demonstrated rate is one-function
//! transcriptions with measured reach 1 apiece. This class is one of those. Its
//! honesty comes from the fences, not from a schedule model: **everything that
//! is not one of the two measured parameters is required to be exactly what the
//! four compiled cells carry.**
//!
//! # The four cells, and the two things they let move
//!
//! `work/w-xtea3/probe/enc.cpp`, real `c2.dll` under wibo at `/O1 /Oi`:
//!
//! ```text
//!   Encipher       the target, verbatim                    116 B  (accepted)
//!   Encipher8      the same with EIGHT rounds              116 B
//!                  — byte-identical except `li r8,8` for `li r8,4`. So the trip
//!                    count is a parameter and NOTHING else moves with it.
//!   EncipherSwap   the returned halves exchanged           116 B
//!                  — byte-identical except the LAST TWO words:
//!                      clrldi r3,r9,32 · rldimi r3,r10,32,0
//!                    against the target's
//!                      clrldi r3,r10,32 · rldimi r3,r9,32,0
//!                  So the return pair's order is the second parameter, and it
//!                  reaches exactly two register fields.
//!   EncipherNoSum  the `sum` update REMOVED                 84 B
//!                  — a different body throughout: the key load is HOISTED out
//!                    of the loop (`lwz r9,0(r5)` in the prologue), the trip
//!                    count register moves, and the schedule collapses to
//!                    sixteen words. Out of class, and the cell that says the
//!                    round body is not composable from its halves.
//! ```
//!
//! # `/Ox` is a different function, not a different schedule
//!
//! At `/Ox /GS- /c` the same source emits **1,352 bytes** with a
//! `__savegprlr_28` frame, six relocations and the loop fully unrolled. The mode
//! gate below is in the PARSER (board #1638) and is the only thing between this
//! class and a body eleven times its size.
//!
//! # The label channel
//!
//! A leaf that carries a back edge: `work/w-xtea2/LABGRID.txt`'s `x-encipher`
//! row measures stride **3** at `/O1` against `ctl-leaf`'s **1**, in
//! `LABEL_COUNTER.md` §7.6's in-the-middle form with `base` re-measured in every
//! obj. `coff::plan_labels` charges `label_lead + 1` for a non-framed function,
//! so the lead is **2** and no arm in `plan_labels` moves — see
//! [`crate::func::IlFunction::label_lead`].

use super::super::expr::{eat_fn_tail, parse_formals};
use super::super::{blk, BodyShape, Block};
use super::designator::sized_ptee;
use super::params::parse_params;
use crate::func::bundle::{opt_word_at, opt_word_mode, OptWordMode};
use crate::func::readers::{eat, eat_byte, eat_opt_stmt_marker, read_token_var, read_type,
                           value_class};

/// The `<<` shift the round applies to the live half. Fixed: no cell separates
/// it from the schedule, and the `slwi r6,rX,4` word encodes it.
/// PROV[S] XTEA's published round structure (Needham & Wheeler, 1997) shifts the live half left by 4. This is a constant of the SOURCE PROGRAM being compiled, not of `c2.dll`; the `slwi r6,rX,4` word is c2 encoding somebody else's algorithm.
const SHL_K: i64 = 4;
/// The `>>` shift. Fixed for [`SHL_K`]'s reason (`srwi r7,rX,5`).
/// PROV[S] XTEA's published right shift of 5 (`srwi r7,rX,5`). See [`SHL_K`].
const SHR_K: i64 = 5;
/// The second half-round's extra `>>` on `sum` before the mask. Fixed: the
/// `rlwinm r7,r11,23,28,29` word folds it with the mask and the scale, and
/// `23 == 32 - 11 + 2`.
/// PROV[S] XTEA's published `sum >> 11` in the second half-round. The `rlwinm r7,r11,23,28,29` word folds it with the mask and the scale, and `23 == 32 - 11 + 2`.
const SUM_SHR_K: i64 = 11;
/// The key index mask. Fixed: it is `mb`/`me` = 28/29 in both `rlwinm`s.
/// PROV[S] XTEA's published four-word key index mask, `& 3`. It is `mb`/`me` = 28/29 in both `rlwinm`s.
const KEY_MASK: i64 = 3;
/// The key element's byte scale — `unsigned int`.
/// PROV[S] `sizeof(unsigned int)` is 4 on this ABI. A language/ABI fact, not a c2 one.
const KEY_SCALE: i64 = 4;
/// The round constant. Carried rather than fixed (the `addis`/`addi` immediates
/// are computed from it), but its LOW half must be below 0x8000: above that the
/// `addis` immediate needs the borrow adjustment no cell here witnesses.
/// Sign-extended, because [`eat_lit`] returns the 4-byte escape's payload the
/// way a signed `int` literal reads it and `0x9E3779B9` has bit 31 set.
/// PROV[S] XTEA's published round constant `0x9E3779B9` (the golden-ratio derivative). Its sign-extension here is a property of how `eat_lit` reads a 4-byte escape, and the low-half bound the doc names is a c2 fact about `addis` borrow adjustment — but the VALUE is Needham and Wheeler's.
pub(crate) const DELTA: i64 = 0x9E37_79B9u32 as i32 as i64;

/// The loop's own `i` bound, and the `li r8,N` immediate. The one thing cell
/// `Encipher8` moves.
fn trip_count_ok(n: i64) -> bool {
    (1..=0x7FFF).contains(&n)
}

/// `<op> <tok>` for the one-token statement bytes — `26` push, `29` label,
/// `3A` jump, `38` branch-if-true, `B9` load.
pub(crate) fn eat_tok(seg: &[u8], p: &mut usize, op: u8, what: &'static str) -> Result<u32, Block> {
    if !eat_byte(seg, p, op) {
        return Err(blk(seg, *p, what));
    }
    let (tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, what))?;
    *p += w;
    Ok(tok)
}

/// A whole TYPE, returned as `(tag, kind)`.
pub(crate) fn eat_ty(seg: &[u8], p: &mut usize, what: &'static str) -> Result<(u8, u8), Block> {
    let (tag, kind, _, w) = read_type(seg, *p).ok_or(blk(seg, *p, what))?;
    *p += w;
    Ok((tag, kind))
}

/// `B9 <tok> <TYPE>` where the TYPE is a 4-byte GPR value.
pub(crate) fn eat_load4(seg: &[u8], p: &mut usize, what: &'static str) -> Result<u32, Block> {
    let tok = eat_tok(seg, p, 0xB9, what)?;
    let (tag, kind) = eat_ty(seg, p, what)?;
    if value_class(tag, kind).is_none() {
        return Err(blk(seg, *p, "xtea-operand-is-not-a-4-byte-gpr-value"));
    }
    Ok(tok)
}

/// `33 <TYPE> <payload>` — a literal at any width. The payload is the short
/// signed byte, or `80` followed by **eight** bytes for a tag-`0x88` type and
/// **four** otherwise, which is `expr::lit_payload_step`'s rule restated because
/// `readers::read_varint` documents that it reads only the 4-byte escape.
pub(crate) fn eat_lit(seg: &[u8], p: &mut usize, what: &'static str) -> Result<i64, Block> {
    if !eat_byte(seg, p, 0x33) {
        return Err(blk(seg, *p, what));
    }
    let (tag, _) = eat_ty(seg, p, what)?;
    let marker = *seg.get(*p).ok_or(blk(seg, *p, what))?;
    if marker != 0x80 {
        *p += 1;
        return Ok(marker as i8 as i64);
    }
    let n = if tag == 0x88 { 8 } else { 4 };
    if seg.len() < *p + 1 + n {
        return Err(blk(seg, *p, what));
    }
    let mut v: u64 = 0;
    for i in 0..n {
        v |= (seg[*p + 1 + i] as u64) << (8 * i);
    }
    *p += 1 + n;
    Ok(if n == 4 { v as u32 as i32 as i64 } else { v as i64 })
}

/// `26 <tok>` — the destination push that opens an assignment statement.
pub(crate) fn eat_push(seg: &[u8], p: &mut usize, what: &'static str) -> Result<u32, Block> {
    eat_opt_stmt_marker(seg, p);
    eat_tok(seg, p, 0x26, what)
}

/// `<op> <TYPE>` then `4B` — the assignment operator and the statement end.
/// `op` is `0x32` for `=` and `0x35`/`0x0F` for the two `+=` spellings.
pub(crate) fn eat_assign_end(seg: &[u8], p: &mut usize, op: u8, what: &'static str) -> Result<(), Block> {
    if !eat_byte(seg, p, op) {
        return Err(blk(seg, *p, what));
    }
    eat_ty(seg, p, what)?;
    if !eat_byte(seg, p, 0x4B) {
        return Err(blk(seg, *p, "xtea-stmt-end"));
    }
    Ok(())
}

/// One half-round: `dst += (src + (src << 4 ^ src >> 5)) ^ sum + key[<idx>];`
///
/// `sum_shr` is `None` for the first half (`sum & 3`) and `Some(11)` for the
/// second (`(sum >> 11) & 3`) — the ONE byte of difference between the two, and
/// the reason they are one function rather than two transcriptions that could
/// drift.
fn eat_half_round(
    seg: &[u8],
    p: &mut usize,
    dst: u32,
    src: u32,
    sum: u32,
    key: u32,
    sum_shr: Option<i64>,
) -> Result<(), Block> {
    let d = eat_push(seg, p, "xtea-half-round-dst")?;
    if d != dst {
        return Err(blk(seg, *p, "xtea-half-round-destination-is-not-the-other-half"));
    }
    // `src`
    if eat_load4(seg, p, "xtea-half-round-src")? != src {
        return Err(blk(seg, *p, "xtea-half-round-operand-is-not-the-live-half"));
    }
    // `src << 4`
    if eat_load4(seg, p, "xtea-half-round-shl-src")? != src {
        return Err(blk(seg, *p, "xtea-half-round-operand-is-not-the-live-half"));
    }
    if eat_lit(seg, p, "xtea-half-round-shl-k")? != SHL_K {
        return Err(blk(seg, *p, "xtea-half-round-left-shift-is-not-four"));
    }
    if !eat_byte(seg, p, 0x09) {
        return Err(blk(seg, *p, "xtea-half-round-not-a-left-shift"));
    }
    // `src >> 5`
    if eat_load4(seg, p, "xtea-half-round-shr-src")? != src {
        return Err(blk(seg, *p, "xtea-half-round-operand-is-not-the-live-half"));
    }
    if eat_lit(seg, p, "xtea-half-round-shr-k")? != SHR_K {
        return Err(blk(seg, *p, "xtea-half-round-right-shift-is-not-five"));
    }
    if !eat_byte(seg, p, 0x0A) {
        return Err(blk(seg, *p, "xtea-half-round-not-a-right-shift"));
    }
    // `^` then `+`
    if !eat_byte(seg, p, 0x0D) {
        return Err(blk(seg, *p, "xtea-half-round-not-a-xor"));
    }
    if !eat_byte(seg, p, 0x02) {
        return Err(blk(seg, *p, "xtea-half-round-not-an-add"));
    }
    // `sum`
    if eat_load4(seg, p, "xtea-half-round-sum")? != sum {
        return Err(blk(seg, *p, "xtea-half-round-addend-is-not-the-round-sum"));
    }
    // `key[…]`
    if eat_load4(seg, p, "xtea-half-round-key")? != key {
        return Err(blk(seg, *p, "xtea-half-round-table-is-not-the-key-formal"));
    }
    if eat_load4(seg, p, "xtea-half-round-index")? != sum {
        return Err(blk(seg, *p, "xtea-half-round-index-is-not-the-round-sum"));
    }
    if let Some(k) = sum_shr {
        if eat_lit(seg, p, "xtea-half-round-index-shr-k")? != k {
            return Err(blk(seg, *p, "xtea-half-round-index-shift-is-not-eleven"));
        }
        if !eat_byte(seg, p, 0x0A) {
            return Err(blk(seg, *p, "xtea-half-round-index-not-a-right-shift"));
        }
    }
    if eat_lit(seg, p, "xtea-half-round-index-mask")? != KEY_MASK {
        return Err(blk(seg, *p, "xtea-half-round-index-mask-is-not-three"));
    }
    if !eat_byte(seg, p, 0x0B) {
        return Err(blk(seg, *p, "xtea-half-round-index-not-an-and"));
    }
    // **The scale is a MULTIPLY, not a designator literal.** A *variable*
    // subscript is `33 <T> <scale> 04 28 00 00` — the element size, an explicit
    // `04` MUL, then the pointer add — where a *constant* one folds the scale
    // into the literal and carries no `04` at all (`?SetNonce`'s
    // `33 86 41 12 08 28 00 00`). The two spellings differ by one byte and the
    // first draft of this recognizer read the constant one.
    if eat_lit(seg, p, "xtea-half-round-index-scale")? != KEY_SCALE {
        return Err(blk(seg, *p, "xtea-half-round-key-element-is-not-four-bytes"));
    }
    if !eat_byte(seg, p, 0x04) {
        return Err(blk(seg, *p, "xtea-half-round-index-not-a-scaling-multiply"));
    }
    if !eat(seg, p, &[0x28, 0x00, 0x00]) {
        return Err(blk(seg, *p, "xtea-half-round-not-a-subscript"));
    }
    if !eat_byte(seg, p, 0x30) {
        return Err(blk(seg, *p, "xtea-half-round-key-not-an-indirect-load"));
    }
    let (tag, kind) = eat_ty(seg, p, "xtea-half-round-key-type")?;
    if value_class(tag, kind).is_none() {
        return Err(blk(seg, *p, "xtea-half-round-key-element-is-not-a-gpr-value"));
    }
    if !eat_byte(seg, p, 0x02) {
        return Err(blk(seg, *p, "xtea-half-round-not-an-add"));
    }
    if !eat_byte(seg, p, 0x0D) {
        return Err(blk(seg, *p, "xtea-half-round-not-a-xor"));
    }
    eat_assign_end(seg, p, 0x0F, "xtea-half-round-not-a-plus-assign")
}

/// **The recognizer.** `start` is the first byte after the body's own `53`; `lo`
/// is the `4C 4F 11` body marker; `depth` is the lexical depth the dispatcher
/// reached.
///
/// Non-committal in the sense every sibling production here is: it works on its
/// own cursor and returns `Err` without side effects, so a body that declines
/// still reports its dispatch arm's blocker and no census key moves.
pub(crate) fn try_parse_xtea_round_loop(
    seg: &[u8],
    start: usize,
    lo: usize,
    _depth: usize,
) -> Result<BodyShape, Block> {
    // **THE MODE GATE LIVES HERE, IN THE PARSER** (board #1638). At `/Ox` this
    // source is 1,352 bytes with a `__savegprlr_28` frame and six relocations.
    if opt_word_mode(opt_word_at(seg)) != Some(OptWordMode::O1) {
        return Err(blk(seg, start, "xtea-not-o1"));
    }
    // r3 = the receiver, r4 = the 64-bit nonce, r5 = the key pointer. The
    // receiver is unused by the body and that is a fact about the emitted words:
    // the class touches r4 and r5 and leaves r3 to be the return value.
    let params = parse_params(seg, lo)?;
    if params.len() != 3 {
        return Err(blk(seg, start, "xtea-not-exactly-three-argument-registers"));
    }
    let formals = parse_formals(seg, lo)?;
    if formals.len() + 1 != params.len() && formals.len() != params.len() {
        return Err(blk(seg, start, "xtea-formals-do-not-account-for-params"));
    }
    let nonce = params[1];
    let key = params[2];

    let mut p = start;

    // `v1 = nonce & 0xFFFFFFFF;`
    let v1 = eat_push(seg, &mut p, "xtea-v1")?;
    if eat_tok(seg, &mut p, 0xB9, "xtea-v1-src")? != nonce {
        return Err(blk(seg, p, "xtea-low-half-is-not-the-nonce-formal"));
    }
    eat_ty(seg, &mut p, "xtea-v1-src-type")?;
    if eat_lit(seg, &mut p, "xtea-v1-mask")? != 0xFFFF_FFFF {
        return Err(blk(seg, p, "xtea-low-half-mask-is-not-0xffffffff"));
    }
    if !eat_byte(seg, &mut p, 0x0B) {
        return Err(blk(seg, p, "xtea-low-half-not-an-and"));
    }
    eat_convert(seg, &mut p, "xtea-v1-narrow")?;
    eat_assign_end(seg, &mut p, 0x32, "xtea-v1-store")?;

    // `v2 = nonce >> 32;`
    let v2 = eat_push(seg, &mut p, "xtea-v2")?;
    if eat_tok(seg, &mut p, 0xB9, "xtea-v2-src")? != nonce {
        return Err(blk(seg, p, "xtea-high-half-is-not-the-nonce-formal"));
    }
    eat_ty(seg, &mut p, "xtea-v2-src-type")?;
    if eat_lit(seg, &mut p, "xtea-v2-shift")? != 32 {
        return Err(blk(seg, p, "xtea-high-half-shift-is-not-thirty-two"));
    }
    if !eat_byte(seg, &mut p, 0x0A) {
        return Err(blk(seg, p, "xtea-high-half-not-a-right-shift"));
    }
    eat_convert(seg, &mut p, "xtea-v2-narrow")?;
    eat_assign_end(seg, &mut p, 0x32, "xtea-v2-store")?;

    // `sum = 0;`
    let sum = eat_push(seg, &mut p, "xtea-sum")?;
    if eat_lit(seg, &mut p, "xtea-sum-init")? != 0 {
        return Err(blk(seg, p, "xtea-sum-does-not-start-at-zero"));
    }
    eat_assign_end(seg, &mut p, 0x32, "xtea-sum-store")?;

    // The rotated `for`: `53`, `i = 0`, jump to the test, the body label, the
    // increment, the test label, `i < N`, branch out. This is `w-bdnz`'s own
    // pre-test rotation, re-read here rather than shared, because that class
    // excludes a memory reference BY NAME (#1981) and this loop has an `lwzx`.
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "xtea-loop-scope"));
    }
    let i = eat_push(seg, &mut p, "xtea-i")?;
    if eat_lit(seg, &mut p, "xtea-i-init")? != 0 {
        return Err(blk(seg, p, "xtea-induction-does-not-start-at-zero"));
    }
    eat_assign_end(seg, &mut p, 0x32, "xtea-i-store")?;
    let l_test = eat_tok(seg, &mut p, 0x3A, "xtea-loop-entry-jump")?;
    let l_body = eat_tok(seg, &mut p, 0x29, "xtea-loop-body-label")?;
    // `i += 1`
    if eat_push(seg, &mut p, "xtea-i-step")? != i {
        return Err(blk(seg, p, "xtea-loop-step-is-not-the-induction-variable"));
    }
    if eat_lit(seg, &mut p, "xtea-i-step-k")? != 1 {
        return Err(blk(seg, p, "xtea-loop-step-is-not-one"));
    }
    eat_assign_end(seg, &mut p, 0x35, "xtea-i-step-store")?;
    if eat_tok(seg, &mut p, 0x29, "xtea-loop-test-label")? != l_test {
        return Err(blk(seg, p, "xtea-loop-test-label-is-not-the-entry-target"));
    }
    if eat_load4(seg, &mut p, "xtea-loop-test")? != i {
        return Err(blk(seg, p, "xtea-loop-test-is-not-the-induction-variable"));
    }
    let trips = eat_lit(seg, &mut p, "xtea-loop-bound")?;
    if !trip_count_ok(trips) {
        return Err(blk(seg, p, "xtea-loop-bound-outside-the-li-immediate"));
    }
    if !eat_byte(seg, &mut p, 0x22) {
        return Err(blk(seg, p, "xtea-loop-test-is-not-a-signed-less-than"));
    }
    let l_exit = eat_tok(seg, &mut p, 0x38, "xtea-loop-exit-branch")?;
    if !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "xtea-loop-body-scope"));
    }

    // The two half-rounds with the `sum` update between them. Their ORDER is
    // load-bearing — cell `EncipherNoSum` is what happens without the update.
    eat_half_round(seg, &mut p, v1, v2, sum, key, None)?;
    if eat_push(seg, &mut p, "xtea-delta")? != sum {
        return Err(blk(seg, p, "xtea-round-update-is-not-the-sum"));
    }
    let delta = eat_lit(seg, &mut p, "xtea-delta-k")?;
    if delta != DELTA {
        return Err(blk(seg, p, "xtea-round-constant-is-not-the-measured-delta"));
    }
    eat_assign_end(seg, &mut p, 0x0F, "xtea-delta-store")?;
    eat_half_round(seg, &mut p, v2, v1, sum, key, Some(SUM_SHR_K))?;

    // Close the body, jump back, and land the exit label.
    eat_opt_stmt_marker(seg, &mut p);
    if !eat(seg, &mut p, &[0x54, 0x04]) {
        return Err(blk(seg, p, "xtea-loop-body-close"));
    }
    if eat_tok(seg, &mut p, 0x3A, "xtea-back-edge")? != l_body {
        return Err(blk(seg, p, "xtea-back-edge-is-not-the-body-label"));
    }
    if eat_tok(seg, &mut p, 0x29, "xtea-exit-label")? != l_exit {
        return Err(blk(seg, p, "xtea-exit-label-is-not-the-branch-target"));
    }

    // `return ((u64)hi << 32) | ((u64)lo & 0xFFFFFFFF);`
    eat_opt_stmt_marker(seg, &mut p);
    let hi = eat_load4(seg, &mut p, "xtea-return-high")?;
    eat_convert(seg, &mut p, "xtea-return-high-widen")?;
    if eat_lit(seg, &mut p, "xtea-return-shift")? != 32 {
        return Err(blk(seg, p, "xtea-return-shift-is-not-thirty-two"));
    }
    if !eat_byte(seg, &mut p, 0x09) {
        return Err(blk(seg, p, "xtea-return-not-a-left-shift"));
    }
    let lo_half = eat_load4(seg, &mut p, "xtea-return-low")?;
    eat_convert(seg, &mut p, "xtea-return-low-widen")?;
    if eat_lit(seg, &mut p, "xtea-return-mask")? != 0xFFFF_FFFF {
        return Err(blk(seg, p, "xtea-return-mask-is-not-0xffffffff"));
    }
    if !eat_byte(seg, &mut p, 0x0B) {
        return Err(blk(seg, p, "xtea-return-not-an-and"));
    }
    if !eat_byte(seg, &mut p, 0x0C) {
        return Err(blk(seg, p, "xtea-return-not-an-or"));
    }
    // **Which half goes where is the SECOND parameter** — cell `EncipherSwap`
    // exchanges exactly the two register fields of the last two words.
    let swapped = if hi == v2 && lo_half == v1 {
        false
    } else if hi == v1 && lo_half == v2 {
        true
    } else {
        return Err(blk(seg, p, "xtea-return-halves-are-not-the-two-round-variables"));
    };

    // The return plumbing, spelled here rather than through
    // [`super::super::expr::eat_return_plumbing`]. Its `41` gate is
    // `eat_int_like_or_ptr4` — a **4-byte** predicate, shared with three
    // byte-graded shapes and deliberately not widened (`ROADMAP.md` §6d) — and
    // this function returns an `unsigned long long`. So the annotation is read
    // here with the class's own 8-byte rule and the FUNCTION TAIL is taken from
    // the shared [`eat_fn_tail`], which is the part that must not drift.
    if !eat_byte(seg, &mut p, 0x41) {
        return Err(blk(seg, p, "xtea-result-type"));
    }
    let (rtag, rkind) = eat_ty(seg, &mut p, "xtea-result-type")?;
    if sized_ptee(rtag, rkind) != Some((8, false)) {
        return Err(blk(seg, p, "xtea-result-is-not-an-8-byte-unsigned"));
    }
    let l_ret = eat_tok(seg, &mut p, 0x3A, "xtea-return-jump")?;
    // The two scope closes, innermost first, each preceded by its own `4F 01
    // <line>` — `eat_return_head`'s own rule, and the depths are pinned because
    // they are the only place the body's BRACING appears in this stream.
    for d in [0x03u8, 0x02] {
        eat_opt_stmt_marker(seg, &mut p);
        if !eat(seg, &mut p, &[0x54, d]) {
            return Err(blk(seg, p, "xtea-return-scope-close"));
        }
    }
    eat_opt_stmt_marker(seg, &mut p);
    if eat_tok(seg, &mut p, 0x29, "xtea-return-label")? != l_ret {
        return Err(blk(seg, p, "xtea-return-label-is-not-the-jump-target"));
    }
    eat_fn_tail(seg, &mut p)?;

    // Every local the class knows about must be distinct, so a body that reused
    // one token for two roles cannot reach the emitter's four fixed registers.
    let toks = [v1, v2, sum, i, nonce, key];
    for a in 0..toks.len() {
        for b in a + 1..toks.len() {
            if toks[a] == toks[b] {
                return Err(blk(seg, p, "xtea-two-roles-share-one-token"));
            }
        }
    }

    Ok(BodyShape::XteaRoundLoop {
        params,
        trips: trips as i32,
        delta: delta as i32,
        swapped,
    })
}

/// `2C <TYPE> <varint>` — one reinterpreting conversion carrying no offset.
pub(crate) fn eat_convert(seg: &[u8], p: &mut usize, what: &'static str) -> Result<(), Block> {
    if !eat_byte(seg, p, 0x2C) {
        return Err(blk(seg, *p, what));
    }
    eat_ty(seg, p, what)?;
    if !eat_byte(seg, p, 0x00) {
        return Err(blk(seg, *p, "xtea-conversion-has-an-offset"));
    }
    Ok(())
}
