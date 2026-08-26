//! **W-XTEA3 — the framed XTEA block loop.**
//! `?Encrypt@XTEABlockEncrypter@@QAAXPBUXTEABlock@@PAU2@@Z`, ninety-six bytes /
//! twenty-four words, the **last** blocked body of
//! `src/system/utl/EncryptXTEA.cpp`.
//!
//! ```cpp
//!   void XTEABlockEncrypter::Encrypt(const XTEABlock *in, XTEABlock *out) {
//!       unsigned int *key = mKey;
//!       unsigned long offset = (char *)out - (char *)in;
//!       for (int i = 0; i < 2; i++) {
//!           *(unsigned long long *)(offset + (char *)in) =
//!               *(unsigned long long *)in ^ Encipher(mNonce[i], key);
//!           mNonce[i] += 1;
//!           in = (const XTEABlock *)((char *)in + 8);
//!       }
//!   }
//! ```
//!
//! # What the three cells let move, and it is exactly three immediates
//!
//! `work/w-xtea3/probe/encrypt.cpp`, real `c2.dll` under wibo at `/O1 /Oi`:
//!
//! ```text
//!   Encrypt          the target, verbatim           96 B  (accepted)
//!   Encrypt4         FOUR trips instead of two      96 B
//!                    — byte-identical except `li r29,4` for `li r29,2`.
//!   EncOff::Encrypt  mNonce at 16, mKey at 32       96 B
//!                    — byte-identical except
//!                        addi r27,r3,32   for  addi r27,r3,16
//!                        addi r30,r3,8    for  addi r30,r3,-8
//!                      i.e. the key member's offset and the nonce member's
//!                      offset MINUS ONE ELEMENT, which is the biased base the
//!                      `stdu` post-increments.
//! ```
//!
//! Three immediates in twenty-four words, and every other field — including the
//! six callee-saved registers, the 144-byte frame and the `addic.`/`bf 2` back
//! edge that replaces CTR because the loop makes a call — is fixed.
//!
//! # The obj obligations this class owes, which the two leaves before it do not
//!
//! * a **frame**: `saved_gprs 6` (r26–r31) → `__savegprlr_26`/`__restgprlr_26`,
//!   and `FrameLayout`'s own rule gives `align16(80 + 8·7) = 144`, which is the
//!   obj's `stwu r1,-144(r1)`. No term is fitted;
//! * **three REL24 sites**: the two frame helpers and the same-TU `?Encipher`,
//!   whose symbol is DEFINED in this obj (`w-fence2`'s seam);
//! * a **`.pdata` record** and a `$M`/`$M`/`$T` triple;
//! * **four label slots before its own triple** —
//!   `work/w-xtea2/LABGRID.txt`'s `x-encrypt` row, `extra 4`, of which two are
//!   the `__savegprlr_26`/`__restgprlr_26` pair (`minted 7` against
//!   `ctl-plain`'s 5) and two the framed `for`. See
//!   [`crate::func::IlFunction::label_lead`].
//!
//! # `/Ox` is a different function
//!
//! The parser's mode gate (board #1638) is the same one the two leaves carry and
//! for the same reason: at `/Ox` c2 inlines `?Encipher` into this body — cell
//! `x-encrypt` against `x-encrypt-alone` in `work/w-xtea2/LABGRID.txt` reads a
//! label stride of 41 with the callee defined and 8 without it, a difference of
//! thirty-three that is the inliner and not the counter.

use super::super::expr::parse_formals;
use super::super::{blk, BodyShape, Block};
use super::params::parse_params;
use super::xtea_round_loop::{eat_assign_end, eat_convert, eat_lit, eat_load4, eat_push, eat_ty,
                             eat_tok};
use crate::func::bundle::{opt_word_at, opt_word_mode, OptWordMode};
use crate::func::readers::{eat, eat_byte, eat_opt_stmt_marker, read_type};

/// The block element's byte stride — the `ld`/`stdu` displacement and the
/// `addi r31,r31,8` step. Fixed: all three cells carry 8.
/// PROV[O] the 8-byte element stride of the encrypt loop, read off the class's captures.
pub(crate) const ELEM: i64 = 8;

/// `B9 <tok> <TYPE>` where the TYPE is a pointer.
fn eat_ptr(seg: &[u8], p: &mut usize, what: &'static str) -> Result<u32, Block> {
    let tok = eat_tok(seg, p, 0xB9, what)?;
    let (tag, kind) = eat_ty(seg, p, what)?;
    if tag & 0x80 == 0 || kind & 0x0F != 0x03 {
        return Err(blk(seg, *p, "xenc-operand-is-not-a-pointer"));
    }
    Ok(tok)
}

/// `B9 <this> <PTR> · 33 <int> <NONCE_OFF> 27 <PTR> · B9 <i> <int> ·
/// 33 <T> 8 04 · 28 00 00` — the address of `mNonce[i]`, which appears twice.
fn eat_nonce_elem(
    seg: &[u8],
    p: &mut usize,
    this: u32,
    i: u32,
    what: &'static str,
) -> Result<i64, Block> {
    if eat_ptr(seg, p, what)? != this {
        return Err(blk(seg, *p, "xenc-nonce-base-is-not-the-receiver"));
    }
    let off = eat_lit(seg, p, what)?;
    if !eat_byte(seg, p, 0x27) {
        return Err(blk(seg, *p, "xenc-nonce-not-a-member-designator"));
    }
    eat_ty(seg, p, what)?;
    if eat_load4(seg, p, what)? != i {
        return Err(blk(seg, *p, "xenc-nonce-index-is-not-the-induction-variable"));
    }
    if eat_lit(seg, p, what)? != ELEM {
        return Err(blk(seg, *p, "xenc-nonce-element-is-not-eight-bytes"));
    }
    if !eat_byte(seg, p, 0x04) {
        return Err(blk(seg, *p, "xenc-nonce-index-not-a-scaling-multiply"));
    }
    if !eat(seg, p, &[0x28, 0x00, 0x00]) {
        return Err(blk(seg, *p, "xenc-nonce-not-a-subscript"));
    }
    Ok(off)
}

/// `30 <TYPE>` — an 8-byte indirect load.
fn eat_load8(seg: &[u8], p: &mut usize, what: &'static str) -> Result<(), Block> {
    if !eat_byte(seg, p, 0x30) {
        return Err(blk(seg, *p, what));
    }
    let (tag, kind, _, w) = read_type(seg, *p).ok_or(blk(seg, *p, what))?;
    if super::designator::sized_ptee(tag, kind) != Some((8, false)) {
        return Err(blk(seg, *p, "xenc-load-is-not-an-8-byte-unsigned"));
    }
    *p += w;
    Ok(())
}

/// **The recognizer.**
///
/// Non-committal in the sense every sibling production here is: it works on its
/// own cursor and returns `Err` without side effects, so a body that declines
/// still reports its dispatch arm's blocker and no census key moves.
pub(crate) fn try_parse_xtea_encrypt_loop(
    seg: &[u8],
    start: usize,
    lo: usize,
    _depth: usize,
) -> Result<BodyShape, Block> {
    // **THE MODE GATE LIVES HERE, IN THE PARSER** (board #1638). At `/Ox` c2
    // inlines the callee into this body.
    if opt_word_mode(opt_word_at(seg)) != Some(OptWordMode::O1) {
        return Err(blk(seg, start, "xenc-not-o1"));
    }
    let params = parse_params(seg, lo)?;
    if params.len() != 3 {
        return Err(blk(seg, start, "xenc-not-exactly-three-argument-registers"));
    }
    let formals = parse_formals(seg, lo)?;
    if formals.len() + 1 != params.len() && formals.len() != params.len() {
        return Err(blk(seg, start, "xenc-formals-do-not-account-for-params"));
    }
    let this = params[0];
    let in_p = params[1];
    let out_p = params[2];

    let mut p = start;

    // `key = mKey;`  →  `addi r27,r3,<key_off>`
    let key = eat_push(seg, &mut p, "xenc-key")?;
    if eat_ptr(seg, &mut p, "xenc-key-base")? != this {
        return Err(blk(seg, p, "xenc-key-base-is-not-the-receiver"));
    }
    let key_off = eat_lit(seg, &mut p, "xenc-key-off")?;
    if !eat_byte(seg, &mut p, 0x27) {
        return Err(blk(seg, p, "xenc-key-not-a-member-designator"));
    }
    eat_ty(seg, &mut p, "xenc-key-designator-type")?;
    eat_convert(seg, &mut p, "xenc-key-convert")?;
    eat_assign_end(seg, &mut p, 0x32, "xenc-key-store")?;

    // `offset = (char *)out - (char *)in;`  →  `sub r26,r5,r4`
    let offset = eat_push(seg, &mut p, "xenc-offset")?;
    if eat_ptr(seg, &mut p, "xenc-offset-lhs")? != out_p {
        return Err(blk(seg, p, "xenc-offset-minuend-is-not-the-out-formal"));
    }
    eat_convert(seg, &mut p, "xenc-offset-lhs-convert")?;
    if eat_ptr(seg, &mut p, "xenc-offset-rhs")? != in_p {
        return Err(blk(seg, p, "xenc-offset-subtrahend-is-not-the-in-formal"));
    }
    eat_convert(seg, &mut p, "xenc-offset-rhs-convert")?;
    if !eat_byte(seg, &mut p, 0x03) {
        return Err(blk(seg, p, "xenc-offset-not-a-subtract"));
    }
    eat_convert(seg, &mut p, "xenc-offset-convert")?;
    eat_assign_end(seg, &mut p, 0x32, "xenc-offset-store")?;

    // The rotated `for`, the same shape `xtea_round_loop` reads.
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "xenc-loop-scope"));
    }
    let i = eat_push(seg, &mut p, "xenc-i")?;
    if eat_lit(seg, &mut p, "xenc-i-init")? != 0 {
        return Err(blk(seg, p, "xenc-induction-does-not-start-at-zero"));
    }
    eat_assign_end(seg, &mut p, 0x32, "xenc-i-store")?;
    let l_test = eat_tok(seg, &mut p, 0x3A, "xenc-loop-entry-jump")?;
    let l_body = eat_tok(seg, &mut p, 0x29, "xenc-loop-body-label")?;
    if eat_push(seg, &mut p, "xenc-i-step")? != i {
        return Err(blk(seg, p, "xenc-loop-step-is-not-the-induction-variable"));
    }
    if eat_lit(seg, &mut p, "xenc-i-step-k")? != 1 {
        return Err(blk(seg, p, "xenc-loop-step-is-not-one"));
    }
    eat_assign_end(seg, &mut p, 0x35, "xenc-i-step-store")?;
    if eat_tok(seg, &mut p, 0x29, "xenc-loop-test-label")? != l_test {
        return Err(blk(seg, p, "xenc-loop-test-label-is-not-the-entry-target"));
    }
    if eat_load4(seg, &mut p, "xenc-loop-test")? != i {
        return Err(blk(seg, p, "xenc-loop-test-is-not-the-induction-variable"));
    }
    let trips = eat_lit(seg, &mut p, "xenc-loop-bound")?;
    if !(1..=0x7FFF).contains(&trips) {
        return Err(blk(seg, p, "xenc-loop-bound-outside-the-li-immediate"));
    }
    if !eat_byte(seg, &mut p, 0x22) {
        return Err(blk(seg, p, "xenc-loop-test-is-not-a-signed-less-than"));
    }
    let l_exit = eat_tok(seg, &mut p, 0x38, "xenc-loop-exit-branch")?;
    if !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "xenc-loop-body-scope"));
    }

    // -- statement 1: the store through the biased destination -----------------
    eat_opt_stmt_marker(seg, &mut p);
    eat_opt_stmt_marker(seg, &mut p);
    // `(char *)in`
    if eat_ptr(seg, &mut p, "xenc-store-base")? != in_p {
        return Err(blk(seg, p, "xenc-store-base-is-not-the-in-formal"));
    }
    eat_convert(seg, &mut p, "xenc-store-base-convert")?;
    // `+ offset`, scaled by 1
    if eat_load4(seg, &mut p, "xenc-store-offset")? != offset {
        return Err(blk(seg, p, "xenc-store-displacement-is-not-the-difference"));
    }
    if eat_lit(seg, &mut p, "xenc-store-scale")? != 1 {
        return Err(blk(seg, p, "xenc-store-displacement-is-scaled"));
    }
    if !eat_byte(seg, &mut p, 0x04) {
        return Err(blk(seg, p, "xenc-store-not-a-scaling-multiply"));
    }
    if !eat_byte(seg, &mut p, 0x02) {
        return Err(blk(seg, p, "xenc-store-not-an-add"));
    }
    eat_convert(seg, &mut p, "xenc-store-convert")?;
    // `*(unsigned long long *)in`
    if eat_ptr(seg, &mut p, "xenc-value-base")? != in_p {
        return Err(blk(seg, p, "xenc-value-base-is-not-the-in-formal"));
    }
    eat_convert(seg, &mut p, "xenc-value-convert")?;
    eat_load8(seg, &mut p, "xenc-value-load")?;
    // `Encipher(mNonce[i], key)` — the call, with the receiver bound.
    let callee_tok = eat_tok(seg, &mut p, 0x26, "xenc-callee")?;
    if eat_ptr(seg, &mut p, "xenc-call-receiver")? != this {
        return Err(blk(seg, p, "xenc-call-receiver-is-not-the-receiver"));
    }
    if !eat_byte(seg, &mut p, 0x99) {
        return Err(blk(seg, p, "xenc-call-not-a-member-bind"));
    }
    eat_ty(seg, &mut p, "xenc-call-bind-type")?;
    if !eat_byte(seg, &mut p, 0x00) {
        return Err(blk(seg, p, "xenc-call-bind-has-an-offset"));
    }
    if !eat_byte(seg, &mut p, 0xBD) {
        return Err(blk(seg, p, "xenc-not-a-call"));
    }
    eat_ty(seg, &mut p, "xenc-call-result-type")?;
    if !eat(seg, &mut p, &[0x00, 0x80, 0x05, 0x10, 0x00, 0x00]) {
        return Err(blk(seg, p, "xenc-call-header"));
    }
    // **The arguments are listed LAST FIRST**, which is the order the emitted
    // `mr r5` / `ld r4` / `mr r3` run puts them in.
    if eat_load4(seg, &mut p, "xenc-call-arg-key")? != key {
        return Err(blk(seg, p, "xenc-call-second-argument-is-not-the-key-local"));
    }
    if !eat_byte(seg, &mut p, 0x55) {
        return Err(blk(seg, p, "xenc-call-arg-sep"));
    }
    eat_ty(seg, &mut p, "xenc-call-arg-sep-type")?;
    let nonce_off = eat_nonce_elem(seg, &mut p, this, i, "xenc-call-arg-nonce")?;
    eat_load8(seg, &mut p, "xenc-call-arg-nonce-load")?;
    if !eat_byte(seg, &mut p, 0x55) {
        return Err(blk(seg, p, "xenc-call-arg-sep"));
    }
    eat_ty(seg, &mut p, "xenc-call-arg-sep-type")?;
    if !eat_byte(seg, &mut p, 0x4C) {
        return Err(blk(seg, p, "xenc-call-end"));
    }
    if !eat_byte(seg, &mut p, 0x0D) {
        return Err(blk(seg, p, "xenc-store-value-not-a-xor"));
    }
    eat_assign_end(seg, &mut p, 0x32, "xenc-store")?;

    // -- statement 2: `mNonce[i] += 1;` ---------------------------------------
    eat_opt_stmt_marker(seg, &mut p);
    if eat_nonce_elem(seg, &mut p, this, i, "xenc-bump")? != nonce_off {
        return Err(blk(seg, p, "xenc-bumped-member-is-not-the-one-the-call-read"));
    }
    if eat_lit(seg, &mut p, "xenc-bump-k")? != 1 {
        return Err(blk(seg, p, "xenc-nonce-step-is-not-one"));
    }
    eat_assign_end(seg, &mut p, 0x0F, "xenc-bump-store")?;

    // -- statement 3: `in = (const XTEABlock *)((char *)in + 8);` -------------
    if eat_push(seg, &mut p, "xenc-in-step")? != in_p {
        return Err(blk(seg, p, "xenc-stepped-pointer-is-not-the-in-formal"));
    }
    if eat_ptr(seg, &mut p, "xenc-in-step-base")? != in_p {
        return Err(blk(seg, p, "xenc-in-step-base-is-not-the-in-formal"));
    }
    eat_convert(seg, &mut p, "xenc-in-step-convert")?;
    if eat_lit(seg, &mut p, "xenc-in-step-k")? != ELEM {
        return Err(blk(seg, p, "xenc-in-step-is-not-eight-bytes"));
    }
    if !eat_byte(seg, &mut p, 0x02) {
        return Err(blk(seg, p, "xenc-in-step-not-an-add"));
    }
    eat_convert(seg, &mut p, "xenc-in-step-back-convert")?;
    eat_assign_end(seg, &mut p, 0x32, "xenc-in-step-store")?;

    // -- the loop close, the back edge, and the void return --------------------
    eat_opt_stmt_marker(seg, &mut p);
    if !eat(seg, &mut p, &[0x54, 0x04]) {
        return Err(blk(seg, p, "xenc-loop-body-close"));
    }
    if eat_tok(seg, &mut p, 0x3A, "xenc-back-edge")? != l_body {
        return Err(blk(seg, p, "xenc-back-edge-is-not-the-body-label"));
    }
    if eat_tok(seg, &mut p, 0x29, "xenc-exit-label")? != l_exit {
        return Err(blk(seg, p, "xenc-exit-label-is-not-the-branch-target"));
    }
    eat_opt_stmt_marker(seg, &mut p);
    if !eat(seg, &mut p, &[0x54, 0x03]) {
        return Err(blk(seg, p, "xenc-loop-scope-close"));
    }
    let l_ret = eat_tok(seg, &mut p, 0x3A, "xenc-return-jump")?;
    eat_opt_stmt_marker(seg, &mut p);
    if !eat(seg, &mut p, &[0x54, 0x02]) {
        return Err(blk(seg, p, "xenc-body-scope-close"));
    }
    eat_opt_stmt_marker(seg, &mut p);
    if eat_tok(seg, &mut p, 0x29, "xenc-return-label")? != l_ret {
        return Err(blk(seg, p, "xenc-return-label-is-not-the-jump-target"));
    }
    super::super::expr::eat_fn_tail(seg, &mut p)?;

    // The three immediates, each bounded by the `addi`/`li` field it lands in.
    // The nonce base is emitted BIASED — `addi r30,r3,<nonce_off - 8>` — because
    // the `stdu` post-increments it, so the biased value is what must fit.
    let biased = nonce_off - ELEM;
    if !(0..=0x7FFF).contains(&key_off) || !(-0x8000..=0x7FFF).contains(&biased) {
        return Err(blk(seg, p, "xenc-member-offset-outside-the-addi-immediate"));
    }
    // Every role must be a distinct token, so a body that reused one for two
    // cannot reach the emitter's six fixed callee-saved registers.
    let toks = [this, in_p, out_p, key, offset, i];
    for a in 0..toks.len() {
        for b in a + 1..toks.len() {
            if toks[a] == toks[b] {
                return Err(blk(seg, p, "xenc-two-roles-share-one-token"));
            }
        }
    }
    Ok(BodyShape::XteaEncryptLoop {
        params,
        callee_tok,
        key_off: key_off as i32,
        nonce_off: nonce_off as i32,
        trips: trips as i32,
    })
}
