//! **W-XTEA3 — the two-element 64-bit member run whose addend is a
//! zero-extended 32-bit formal.** `?SetNonce@XTEABlockEncrypter@@QAAXPB_KI@Z`,
//! thirty-two bytes, one of the three bodies still blocking
//! `src/system/utl/EncryptXTEA.cpp` after `w-xtea2`.
//!
//! ```cpp
//!   void XTEABlockEncrypter::SetNonce(const unsigned long long *nonce,
//!                                     unsigned int shift) {
//!       mNonce[0] = nonce[0] + shift;
//!       mNonce[1] = nonce[1] + shift;
//!   }
//! ```
//!
//! ```text
//!   ?SetNonce@XTEABlockEncrypter@@QAAXPB_KI@Z   .text COMDAT, 0x20 B, nrel 0
//!     0000  e9440000  ld     r10,0(r4)      nonce[0]
//!     0004  78ab0020  clrldi r11,r5,32      the zero-extended shift, ONCE
//!     0008  7d4a5a14  add    r10,r10,r11
//!     000c  f9430000  std    r10,0(r3)      mNonce[0]
//!     0010  e9440008  ld     r10,8(r4)      nonce[1]
//!     0014  7d6a5a14  add    r11,r10,r11    r11's LAST use, so it is the target
//!     0018  f9630008  std    r11,8(r3)      mNonce[1]
//!     001c  4e800020  blr
//! ```
//!
//! # The six cells that fix every word, compiled by this lane
//!
//! `work/w-xtea3/probe/nonce.cpp`, real `c2.dll` under wibo at `/O1 /Oi`. Every
//! clause below is a reading off one of them, and each is the `_neg` cell of the
//! clause it separates.
//!
//! ```text
//!   SetNonce      the target, verbatim                     32 B  (accepted)
//!   SetNonceRev   `shift + nonce[i]` — IDENTICAL 32 bytes, but the IL puts the
//!                 addend's `B9` BEFORE the source load, so this recognizer
//!                 refuses it. Refusing a body whose bytes we would get right is
//!                 free; admitting it on an unwitnessed operand order is not.
//!   SetNonce1     ONE element                              20 B
//!                   ld r11 · clrldi r10 · add 11,10,11 · std r11 · blr
//!                 — **the two scratch registers are SWAPPED**. So the plan is
//!                 not "load into r10", it is a fact about a run of exactly two,
//!                 and a one-element body emitted by this class would be four
//!                 wrong registers.
//!   SetNonce3     THREE statements                          48 B
//!                 the first two are byte-identical to the target and the third
//!                 re-loads `nonce[0]` into r11 — so the class's fence is on the
//!                 statement COUNT and the prefix is stable.
//!   SetNonceU64   an `unsigned long long` addend            28 B
//!                   ld r11 · add 11,11,5 · std r11 · ld r11 · add 11,11,5 · std
//!                 — **no `clrldi` at all** and a third register plan. The
//!                 addend's 4-byte-ness is what mints the `clrldi`, and it is
//!                 checked at the `2C` conversion rather than assumed.
//!   EncOff        the member at offset 8 rather than 0      32 B
//!                   … std r10,8(r3) … std r11,16(r3)
//!                 — the destination and source offsets move INDEPENDENTLY, so
//!                 both are carried and neither is derived from the other.
//! ```
//!
//! # `/Ox` is a different register plan, not a different length
//!
//! At `/Ox /GS- /c` the same source emits
//!
//! ```text
//!   ld r10,0(r4) · clrldi r11,r5,32 · add r10,r10,r11 · std r10,0(r3)
//!   ld r10,8(r4) · add  r9,r10,r11 · std  r9,8(r3) · blr
//! ```
//!
//! — the second `add` targets **r9**, and `std` reads r9. Eight words at both
//! modes, six of them identical, and two that are not. So the mode gate below is
//! load-bearing: a class admitted at `/Ox` would emit two wrong registers into
//! an obj that still links, which is board #263's shape. Board #1638 puts the
//! gate in the PARSER, where the census can see it too.

use super::super::expr::{eat_return_plumbing, parse_formals};
use super::super::{blk, BodyShape, Block};
use super::designator::{eat_offset_adds, sized_ptee};
use super::guard_ret_chain::eat_load;
use super::params::parse_params;
use crate::func::bundle::{opt_word_at, opt_word_mode, OptWordMode};
use crate::func::readers::{
    eat_byte, eat_opt_stmt_marker, read_token_var, read_type, read_varint, value_class,
};

/// The element stride of the run, in bytes: the class stores 8-byte scalars and
/// the two cells that move (`SetNonce`, `EncOff::SetNonce`) both step by 8.
/// PROV[O] the run's 8-byte element stride, read off both moving cells (`SetNonce`, `EncOff::SetNonce`).
pub(crate) const ELEM: i32 = 8;

/// How many statements the class is. **Not a parameter**: `SetNonce1` and
/// `SetNonce3` above are two different register plans, so the number is part of
/// what was measured rather than something the recognizer counts and passes on.
/// PROV[O] how many statements the class is — and its own doc insists it is "**Not a parameter**": `SetNonce1` and `SetNonce3` are two different register plans, so two is part of what was measured rather than something the recognizer counts and passes on.
pub(crate) const RUN_LEN: usize = 2;

/// One `dst[i] = src[i] + (u64)addend;` statement, decoded.
struct Elem {
    dst_off: i32,
    src_off: i32,
}

/// `B9 <tok> <PTR>` then the `33 … 27/28` offset-add run — the address half of
/// either side of the assignment. Returns the token and the summed offset.
fn eat_addr(seg: &[u8], p: &mut usize, what: &'static str) -> Result<(u32, i32), Block> {
    let (tok, is_ptr) = eat_load(seg, p, what)?;
    if !is_ptr {
        return Err(blk(seg, *p, "nonce-addr-not-a-pointer"));
    }
    let off = eat_offset_adds(seg, p).ok_or(blk(seg, *p, "nonce-addr-designator"))?.0;
    // `ld`/`std` take a signed 16-bit DS displacement whose low two bits are
    // zero. A wider member offset is an `addis`/`addi` pair this class has no
    // cell for, and a misaligned one is not a `DS` form at all.
    if !(0..=0x7FF8).contains(&off) || off % 4 != 0 {
        return Err(blk(seg, *p, "nonce-addr-offset-outside-the-ds-displacement"));
    }
    Ok((tok, off))
}

/// `B9 <tok> <4-BYTE TYPE> · 2C <8-BYTE TYPE> 00` — the addend **and its
/// widening, as ONE clause**, which is the whole reason there is a `clrldi`.
///
/// **The two halves are deliberately not two clauses.** They state one fact —
/// *the addend is a 4-byte value zero-extended to 8* — and cell `SetNonceU64`
/// (an `unsigned long long` addend, which is neither 4-byte nor widened) is
/// refused by **either** of them alone. `w-xtea2` #2665 measured exactly that
/// shape: a cell fenced by several clauses grades NONE of them, because deleting
/// one leaves the others refusing it, and the repair is MERGING rather than
/// adding cells. `work/w-xtea3/MUTATIONS.md` M2 came back `vocab-gap` twice —
/// once against the split clauses and once against a mutation that broke only
/// half of the merged one — before the whole conjunction was deleted at once.
fn eat_addend(seg: &[u8], p: &mut usize) -> Result<u32, Block> {
    if !eat_byte(seg, p, 0xB9) {
        return Err(blk(seg, *p, "nonce-addend"));
    }
    let (tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, "nonce-addend"))?;
    *p += w;
    let (tag, kind, _, tw) = read_type(seg, *p).ok_or(blk(seg, *p, "nonce-addend-type"))?;
    let four = value_class(tag, kind).is_some();
    *p += tw;
    // The `2C` widening, read non-committally so that the conjunction below is
    // ONE refusal and not two.
    let widened = eat_widen8(seg, p, "nonce-addend-widening").is_ok();
    if !(four && widened) {
        return Err(blk(seg, *p, "nonce-addend-is-not-a-4-byte-value-widened-to-eight"));
    }
    Ok(tok)
}

/// `30 <TYPE>` — the 8-byte indirect load. The width is read out of the TYPE
/// through the shared [`sized_ptee`] whitelist rather than off the tag's width
/// nibble, which `designator.rs` records as unreliable.
fn eat_load8(seg: &[u8], p: &mut usize) -> Result<(), Block> {
    if !eat_byte(seg, p, 0x30) {
        return Err(blk(seg, *p, "nonce-not-an-indirect-load"));
    }
    let (tag, kind, _, tw) = read_type(seg, *p).ok_or(blk(seg, *p, "nonce-load-type"))?;
    match sized_ptee(tag, kind) {
        Some((8, false)) => {}
        _ => return Err(blk(seg, *p, "nonce-load-is-not-an-8-byte-unsigned")),
    }
    *p += tw;
    Ok(())
}

/// `2C <TYPE> <varint>` — one reinterpreting conversion, required to be to an
/// 8-byte unsigned and to carry no offset of its own.
fn eat_widen8(seg: &[u8], p: &mut usize, what: &'static str) -> Result<(), Block> {
    if !eat_byte(seg, p, 0x2C) {
        return Err(blk(seg, *p, what));
    }
    let (tag, kind, _, tw) = read_type(seg, *p).ok_or(blk(seg, *p, what))?;
    match sized_ptee(tag, kind) {
        Some((8, false)) => {}
        _ => return Err(blk(seg, *p, "nonce-conversion-is-not-to-an-8-byte-unsigned")),
    }
    *p += tw;
    let k = read_varint(seg, p).ok_or(blk(seg, *p, what))?;
    if k != 0 {
        return Err(blk(seg, *p, "nonce-conversion-has-an-offset"));
    }
    Ok(())
}

/// `32 <TYPE>` — the 8-byte indirect store that closes the statement.
fn eat_store8(seg: &[u8], p: &mut usize) -> Result<(), Block> {
    if !eat_byte(seg, p, 0x32) {
        return Err(blk(seg, *p, "nonce-not-an-indirect-store"));
    }
    let (tag, kind, _, tw) = read_type(seg, *p).ok_or(blk(seg, *p, "nonce-store-type"))?;
    match sized_ptee(tag, kind) {
        Some((8, false)) => {}
        _ => return Err(blk(seg, *p, "nonce-store-is-not-an-8-byte-unsigned")),
    }
    *p += tw;
    Ok(())
}

/// One statement of the run. `addend` is the token the FIRST statement bound;
/// every later one must name the same token, which is what makes the `clrldi`
/// a common subexpression rather than a per-statement instruction.
fn eat_stmt(
    seg: &[u8],
    p: &mut usize,
    this_tok: u32,
    src_tok: u32,
    addend_tok: u32,
) -> Result<Elem, Block> {
    eat_opt_stmt_marker(seg, p);
    let (dtok, dst_off) = eat_addr(seg, p, "nonce-dst")?;
    if dtok != this_tok {
        return Err(blk(seg, *p, "nonce-destination-is-not-the-receiver"));
    }
    let (stok, src_off) = eat_addr(seg, p, "nonce-src")?;
    if stok != src_tok {
        return Err(blk(seg, *p, "nonce-source-is-not-the-source-formal"));
    }
    eat_load8(seg, p)?;
    // **The addend, and the whole reason there is a `clrldi`.** It is a 4-byte
    // value in a register and the `2C` widens it to 8 — cell `SetNonceU64`
    // carries a 64-bit addend, emits no `clrldi` and has a different register
    // plan throughout, so this is not decoration.
    let atok = eat_addend(seg, p)?;
    if atok != addend_tok {
        return Err(blk(seg, *p, "nonce-addend-is-not-the-same-token-in-every-statement"));
    }
    if !eat_byte(seg, p, 0x02) {
        return Err(blk(seg, *p, "nonce-operator-is-not-add"));
    }
    eat_widen8(seg, p, "nonce-result-widening")?;
    eat_store8(seg, p)?;
    if !eat_byte(seg, p, 0x4B) {
        return Err(blk(seg, *p, "nonce-stmt-end"));
    }
    Ok(Elem { dst_off, src_off })
}

/// **The recognizer.** `start` is the first byte after the body's own `53`; `lo`
/// is the `4C 4F 11` body marker; `depth` is the lexical depth the dispatcher
/// reached, which the return plumbing needs.
///
/// Non-committal in the sense every sibling production here is: it works on its
/// own cursor and returns `Err` without side effects, so a body that declines
/// still reports its dispatch arm's blocker and no census key moves.
pub(crate) fn try_parse_nonce_add_run(
    seg: &[u8],
    start: usize,
    lo: usize,
    depth: usize,
) -> Result<BodyShape, Block> {
    // **THE MODE GATE LIVES HERE, IN THE PARSER** (board #1638), and it is
    // load-bearing rather than cautious: see the module header's `/Ox` reading —
    // the second `add` targets r9 there and `std` reads r9.
    if opt_word_mode(opt_word_at(seg)) != Some(OptWordMode::O1) {
        return Err(blk(seg, start, "nonce-not-o1"));
    }
    // r3 = the receiver, r4 = the source pointer, r5 = the addend. Gated on
    // `params` rather than `parse_formals` for `memcpy_tail`'s reason: `this`
    // occupies an argument register and the emitter's three registers are
    // argument registers.
    let params = parse_params(seg, lo)?;
    if params.len() != 3 {
        return Err(blk(seg, start, "nonce-not-exactly-three-argument-registers"));
    }
    // …and no formal beyond them, so `parse_params`' `this` accounting is
    // checked rather than assumed (`params.rs`'s own bug: a formal mapped to the
    // register `this` occupies).
    let formals = parse_formals(seg, lo)?;
    if formals.len() + 1 != params.len() && formals.len() != params.len() {
        return Err(blk(seg, start, "nonce-formals-do-not-account-for-params"));
    }

    let mut p = start;
    // The first statement binds the three tokens; every later one must name the
    // same three. Read here rather than inside `eat_stmt` so the "same token"
    // clause below has something to compare against.
    eat_opt_stmt_marker(seg, &mut p);
    let mut probe = p;
    let (this_tok, _) = eat_addr(seg, &mut probe, "nonce-dst")?;
    let (src_tok, _) = eat_addr(seg, &mut probe, "nonce-src")?;
    eat_load8(seg, &mut probe)?;
    let addend_tok = eat_addend(seg, &mut probe)?;
    if this_tok == src_tok || this_tok == addend_tok || src_tok == addend_tok {
        return Err(blk(seg, p, "nonce-two-of-the-three-operands-are-the-same-token"));
    }
    if this_tok != params[0] || src_tok != params[1] || addend_tok != params[2] {
        return Err(blk(seg, p, "nonce-operands-are-not-already-in-the-argument-registers"));
    }

    let mut elems = Vec::with_capacity(RUN_LEN);
    for _ in 0..RUN_LEN {
        elems.push(eat_stmt(seg, &mut p, this_tok, src_tok, addend_tok)?);
    }
    // **EXACTLY two statements**, and the fence is on the count because the
    // count is what the register plan is a fact about: `SetNonce1` swaps r10 and
    // r11 and `SetNonce3` adds a third statement in a plan this class does not
    // model. A third statement here would be silently dropped, which is a
    // complete plausible wrong body — board #232's shape.
    eat_opt_stmt_marker(seg, &mut p);
    eat_return_plumbing(seg, &mut p, false, depth)?;

    // The two elements step by [`ELEM`] on BOTH sides, independently based —
    // cell `EncOff` moves the destination and not the source.
    if elems[1].dst_off != elems[0].dst_off + ELEM || elems[1].src_off != elems[0].src_off + ELEM {
        return Err(blk(seg, p, "nonce-the-two-elements-are-not-one-stride-apart"));
    }
    Ok(BodyShape::NonceAddRun {
        params,
        dst_off: elems[0].dst_off,
        src_off: elems[0].src_off,
    })
}
