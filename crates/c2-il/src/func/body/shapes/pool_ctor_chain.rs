//! **W-POOL2 — the free-list constructor's chain build.**
//!
//! `src/system/utl/Pool.cpp`'s `??0Pool@@QAA@HPAXH@Z`, and the deepest of that
//! TU's three bodies by every axis: 80 B, twenty words, three basic blocks, a
//! signed division with both of its traps, and a `bdnz` counted loop.
//!
//! ```cpp
//!   Pool::Pool(int i1, void *v, int i2) : mFree((char *)v) {
//!       char *ptr = (char *)v;
//!       int stride = (i1 + 3) & ~3;
//!       int count  = i2 / stride;
//!       if (count > 1) {
//!           int n = count - 1;
//!           do {
//!               char *next = ptr + stride;
//!               *(char **)ptr = next;
//!               ptr = next;
//!           } while (--n);
//!       }
//!       *(char **)ptr = 0;
//!   }
//! ```
//!
//! # The IL statement list this reads, in order
//!
//! ```text
//!   this->m = (char*)v            store through the member designator
//!   ptr     = (char*)v            26 <ptr> · B9 <v> · 2C · 32 · 4B
//!   stride  = (i1 + K) & ~K       26 <stride> · B9 <i1> · 33 K · 02 · 33 ~K · 0B
//!   count   = i2 / stride         26 <count> · B9 <i2> · B9 <stride> · 05
//!   if (count > 1)                B9 <count> · 33 1 · 24 · 38 <Lend>
//!     n = count - 1               26 <n> · B9 <count> · 33 1 · 03
//!     29 <Ltop>                   the do-while's back-edge target
//!       next = ptr + stride       26 <next> · B9 <ptr> · B9 <stride> · 33 1 · 04 · 02
//!       *(char**)ptr = next       B9 <ptr> · 2C · B9 <next> · 32
//!       ptr = next                26 <ptr> · B9 <next> · 32
//!     29 <Ltest>                  the while test's own label
//!       --n                       26 <n> · 33 1 · 10 · 39 <Ltop>
//!     29 <Lbreak>
//!   29 <Lend>
//!   *(char**)ptr = 0              B9 <ptr> · 2C · 33 0 · 32
//!   return this
//! ```
//!
//! The `33 1 · 04` inside `ptr + stride` is the pointer's **element scale**, 1
//! for `char*`, and it is required to be exactly 1: any other scale is a
//! `mulli`/`slwi` this class has no witness of.
//!
//! # WHAT THIS CLASS DOES **NOT** OWE — the two prices that came down
//!
//! `docs/whitebox/WB_LOOP_FINDINGS.md` §9 names two unread things a
//! `loop_counted` class would owe. **Neither is owed here**, and that is the
//! finding this file exists to record rather than a convenience:
//!
//! * **§9 item 4, the trip-count arithmetic.** It is unread *for a non-unit
//!   step*, which is why §9.1 says the honest first class should require
//!   `step ∈ {+1, −1}`. This loop's step is `--n`, i.e. **−1**, and its counter
//!   *is* its trip count: `mtctr` takes `n` after a single `addi -1`. There is
//!   no `srwi`/`divwu`/`addi +1` preheader to read.
//! * **§7.7 rule 1, rotation-plus-guard.** The `cmpwi cr6,r10,1 ; bf 25,+28` is
//!   **the source's own `if (count > 1)`** — it is in the IL as `24` then
//!   `38 <label>` — not a synthesized zero-trip pre-test. So the guard is
//!   *read*, not *invented*, and board #1902's rule (which decides the pre-test's
//!   operands and sense) is not consulted anywhere in this file.
//!
//! What remains is `mtctr`/`bdnz` itself (`wb-loop` #1900, obj-confirmed) over a
//! four-word body, and `w-loop` #744's own predicate holds on it: the trip count
//! is computable at entry and the body makes no call.
//!
//! # This is a TRANSCRIPTION, and the schedule is the part that is not derived
//!
//! `c2_core::codegen::pool_ctor_chain` emits twenty words whose ORDER is c2's
//! scheduler's, read off this lane's own capture of `Pool.obj`. Specifically:
//! the divide's overflow helper (`rotlwi`) is hoisted **above the member-init
//! store**, and `andc` sits **between** the `divw` and the first `twi` — the
//! five words `codegen::div_mod_leaf` transcribes as a contiguous constant body
//! at r11/r3/r4 arrive here at **r10/r9/r11, split across four unrelated
//! instructions**. `div_mod_leaf`'s own header says it plainly and it is true
//! twice over here: *"There is no scheduler here and no register allocator."*
//! A body that is not exactly this statement list is refused rather than
//! scheduled.
//!
//! # `/O1` only, and the gate is in the PARSER (#1638, #1710)
//!
//! At `/Ox` the same source is **twenty-one** words with a different register
//! plan — an extra `mr r11,r5`, and `r9`/`r10`/`r8`/`r7` where `/O1` has
//! `r10`/`r11`/`r9` — captured on this lane's own `work/w-pool2/ref/PoolOx.obj`.
//! So the mode word is asked before any body byte is read.
//!
//! # The pinned constants, each stated as pinned
//!
//! * **the alignment is 4** (`+3` and `& ~3`). The addend and the `rlwinm`
//!   MB/ME pair are a *matched* pair and this lane graded one of them; a second
//!   alignment without its own obj would be `JsonUtf8Copy`'s refused widening.
//! * **the guard literal is 1** and **the counter's initial adjustment is −1**.
//!   Both feed the `cmpwi cr6,r10,1` / `addi r10,r10,-1` pair, and moving either
//!   alone changes the loop's trip count as well as the word.
//! * **the pointer scale is 1** (`char*`).
//! * **the terminating store's value is the literal 0** — `li r11,0`.

use super::super::expr::{eat_fn_tail, eat_scopes, BODY_SCOPE_DEPTH};
use super::super::BodyShape;
use super::designator::eat_offset_adds;
use super::params::parse_params;
use crate::func::bundle::{opt_word_at, opt_word_mode, OptWordMode};
use crate::func::readers::{
    eat, eat_byte, eat_int_like, eat_opt_stmt_marker, is_ptr4_kind, is_volatile_tag,
    read_token_var, read_type, read_varint,
};
use crate::func::PoolCtorChain;

/// The one alignment with an obj behind it. See the module header's pinned list.
const ALIGN: i32 = 4;

fn eat_ptr4(seg: &[u8], p: &mut usize) -> Option<()> {
    let (tag, kind, _, w) = read_type(seg, *p)?;
    if !is_ptr4_kind(tag, kind) || is_volatile_tag(tag) {
        return None;
    }
    *p += w;
    Some(())
}

fn eat_ptr_load(seg: &[u8], p: &mut usize) -> Option<u32> {
    if !eat_byte(seg, p, 0xB9) {
        return None;
    }
    let (tok, w) = read_token_var(seg, *p)?;
    *p += w;
    eat_ptr4(seg, p)?;
    Some(tok)
}

fn eat_int_load(seg: &[u8], p: &mut usize) -> Option<u32> {
    if !eat_byte(seg, p, 0xB9) {
        return None;
    }
    let (tok, w) = read_token_var(seg, *p)?;
    *p += w;
    if !eat_int_like(seg, p) {
        return None;
    }
    Some(tok)
}

/// `33 <int-like> k` — an integer literal in an operand position.
fn eat_int_lit(seg: &[u8], p: &mut usize) -> Option<i32> {
    if !eat_byte(seg, p, 0x33) || !eat_int_like(seg, p) {
        return None;
    }
    read_varint(seg, p)
}

fn eat_ptr_cast(seg: &[u8], p: &mut usize) -> Option<()> {
    if !eat_byte(seg, p, 0x2C) {
        return None;
    }
    eat_ptr4(seg, p)?;
    if !eat_byte(seg, p, 0x00) {
        return None;
    }
    Some(())
}

/// `26 <tok>` — the destination push that opens an assignment statement.
fn eat_dest(seg: &[u8], p: &mut usize) -> Option<u32> {
    if !eat_byte(seg, p, 0x26) {
        return None;
    }
    let (tok, w) = read_token_var(seg, *p)?;
    *p += w;
    Some(tok)
}

/// `29 <tok>` — a label definition. Returns the label token.
fn eat_label(seg: &[u8], p: &mut usize) -> Option<u32> {
    eat_opt_stmt_marker(seg, p);
    if !eat_byte(seg, p, 0x29) {
        return None;
    }
    let (tok, w) = read_token_var(seg, *p)?;
    *p += w;
    Some(tok)
}

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

/// The member designator `B9 <this> <PTR> · 33 <int> k 27 <PTR>`, with the
/// re-type required. Same helper, same argument, as
/// [`super::pool_free_list::eat_member_designator`]'s.
fn eat_member_designator(seg: &[u8], p: &mut usize, this_tok: u32) -> Option<i32> {
    let mut q = *p;
    if eat_ptr_load(seg, &mut q)? != this_tok {
        return None;
    }
    let (off, retype) = eat_offset_adds(seg, &mut q)?;
    retype?;
    *p = q;
    Some(off)
}

/// Try the free-list constructor. Non-committal: a cursor copy and an `Option`,
/// so a body that is not this production keeps its own first-blocker key.
pub(crate) fn try_parse_pool_ctor_chain(
    seg: &[u8],
    start: usize,
    lo: usize,
    depth: usize,
) -> Option<BodyShape> {
    // The optimization word FIRST — board #1638, and this class's `/Ox` body is
    // twenty-one words rather than twenty.
    if opt_word_mode(opt_word_at(seg)) != Some(OptWordMode::O1) {
        return None;
    }
    if depth != BODY_SCOPE_DEPTH {
        return None;
    }
    let params = parse_params(seg, lo).ok()?;
    // `this`, the block size, the arena base and the arena size. Pinned: every
    // register in the emitted body is a slot index.
    if params.len() != 4 {
        return None;
    }
    let (this_tok, size_tok, base_tok, total_tok) =
        (params[0], params[1], params[2], params[3]);
    let mut p = start;

    // ---- `this->m = (char*)v;` — the member initializer -------------------
    eat_opt_stmt_marker(seg, &mut p);
    let off = eat_member_designator(seg, &mut p, this_tok)?;
    if eat_ptr_load(seg, &mut p)? != base_tok {
        return None;
    }
    eat_ptr_cast(seg, &mut p)?;
    if !eat_byte(seg, &mut p, 0x32) {
        return None;
    }
    eat_ptr4(seg, &mut p)?;
    if !eat_byte(seg, &mut p, 0x4B) {
        return None;
    }

    // ---- `char *ptr = (char*)v;` -------------------------------------------
    eat_opt_stmt_marker(seg, &mut p);
    let ptr = eat_dest(seg, &mut p)?;
    if params.contains(&ptr) {
        return None;
    }
    if eat_ptr_load(seg, &mut p)? != base_tok {
        return None;
    }
    eat_ptr_cast(seg, &mut p)?;
    if !eat_byte(seg, &mut p, 0x32) {
        return None;
    }
    eat_ptr4(seg, &mut p)?;
    if !eat_byte(seg, &mut p, 0x4B) {
        return None;
    }

    // ---- `int stride = (i1 + 3) & ~3;` -------------------------------------
    eat_opt_stmt_marker(seg, &mut p);
    let stride = eat_dest(seg, &mut p)?;
    if params.contains(&stride) {
        return None;
    }
    if eat_int_load(seg, &mut p)? != size_tok {
        return None;
    }
    if eat_int_lit(seg, &mut p)? != ALIGN - 1 {
        return None;
    }
    if !eat_byte(seg, &mut p, 0x02) {
        return None;
    }
    if eat_int_lit(seg, &mut p)? != !(ALIGN - 1) {
        return None;
    }
    if !eat_byte(seg, &mut p, 0x0B) {
        return None;
    }
    if !eat_byte(seg, &mut p, 0x32) || !eat_int_like(seg, &mut p) {
        return None;
    }
    if !eat_byte(seg, &mut p, 0x4B) {
        return None;
    }

    // ---- `int count = i2 / stride;` — SIGNED, hence the two traps ----------
    eat_opt_stmt_marker(seg, &mut p);
    let count = eat_dest(seg, &mut p)?;
    if params.contains(&count) {
        return None;
    }
    if eat_int_load(seg, &mut p)? != total_tok {
        return None;
    }
    if eat_int_load(seg, &mut p)? != stride {
        return None;
    }
    // `05` is DIV and `06` is MOD (`div_mod_leaf`'s own table). Only the signed
    // division's trap pair is transcribed here; `eat_int_like` above is what
    // keeps the operand type to the four-byte integer family the traps were
    // measured on.
    if !eat_byte(seg, &mut p, 0x05) {
        return None;
    }
    if !eat_byte(seg, &mut p, 0x32) || !eat_int_like(seg, &mut p) {
        return None;
    }
    if !eat_byte(seg, &mut p, 0x4B) {
        return None;
    }

    // ---- `if (count > 1)` — the SOURCE's own guard -------------------------
    let mut d = depth;
    eat_scopes(seg, &mut p, &mut d).ok()?;
    if d <= depth {
        return None;
    }
    if eat_int_load(seg, &mut p)? != count {
        return None;
    }
    if eat_int_lit(seg, &mut p)? != 1 {
        return None;
    }
    // `24` is `>`; see [`crate::func::Rel::from_opcode`]. Matched literally
    // rather than through `Rel`, because the emitted `bf 25` names cr6's GT bit
    // and no other relation was graded.
    if !eat_byte(seg, &mut p, 0x24) {
        return None;
    }
    if !eat_byte(seg, &mut p, 0x38) {
        return None;
    }
    let (l_end, w) = read_token_var(seg, p)?;
    p += w;

    // ---- `int n = count - 1;` ---------------------------------------------
    let mut d2 = d;
    eat_scopes(seg, &mut p, &mut d2).ok()?;
    if d2 <= d {
        return None;
    }
    let n = eat_dest(seg, &mut p)?;
    if params.contains(&n) || n == count {
        return None;
    }
    if eat_int_load(seg, &mut p)? != count {
        return None;
    }
    if eat_int_lit(seg, &mut p)? != 1 {
        return None;
    }
    if !eat_byte(seg, &mut p, 0x03) {
        return None;
    }
    if !eat_byte(seg, &mut p, 0x32) || !eat_int_like(seg, &mut p) {
        return None;
    }
    if !eat_byte(seg, &mut p, 0x4B) {
        return None;
    }

    // ---- the loop top ------------------------------------------------------
    let l_top = eat_label(seg, &mut p)?;
    let mut d3 = d2;
    eat_scopes(seg, &mut p, &mut d3).ok()?;
    if d3 <= d2 {
        return None;
    }

    // `char *next = ptr + stride;`
    eat_opt_stmt_marker(seg, &mut p);
    let next = eat_dest(seg, &mut p)?;
    if params.contains(&next) || next == ptr {
        return None;
    }
    if eat_ptr_load(seg, &mut p)? != ptr {
        return None;
    }
    if eat_int_load(seg, &mut p)? != stride {
        return None;
    }
    // The pointer's element scale. `char*` is 1; anything else is a multiply
    // this class has no witness of.
    if eat_int_lit(seg, &mut p)? != 1 {
        return None;
    }
    if !eat_byte(seg, &mut p, 0x04) || !eat_byte(seg, &mut p, 0x02) {
        return None;
    }
    if !eat_byte(seg, &mut p, 0x32) {
        return None;
    }
    eat_ptr4(seg, &mut p)?;
    if !eat_byte(seg, &mut p, 0x4B) {
        return None;
    }

    // `*(char**)ptr = next;`
    eat_opt_stmt_marker(seg, &mut p);
    if eat_ptr_load(seg, &mut p)? != ptr {
        return None;
    }
    eat_ptr_cast(seg, &mut p)?;
    if eat_ptr_load(seg, &mut p)? != next {
        return None;
    }
    if !eat_byte(seg, &mut p, 0x32) {
        return None;
    }
    eat_ptr4(seg, &mut p)?;
    if !eat_byte(seg, &mut p, 0x4B) {
        return None;
    }

    // `ptr = next;`
    eat_opt_stmt_marker(seg, &mut p);
    if eat_dest(seg, &mut p)? != ptr {
        return None;
    }
    if eat_ptr_load(seg, &mut p)? != next {
        return None;
    }
    if !eat_byte(seg, &mut p, 0x32) {
        return None;
    }
    eat_ptr4(seg, &mut p)?;
    if !eat_byte(seg, &mut p, 0x4B) {
        return None;
    }

    // ---- `} while (--n);` --------------------------------------------------
    eat_closes_to(seg, &mut p, &mut d3, d2)?;
    // The while test is its own label — the `continue` target — and the source
    // has no `continue`, so nothing branches to it. It is consumed, not
    // interpreted.
    eat_label(seg, &mut p)?;
    if eat_dest(seg, &mut p)? != n {
        return None;
    }
    if eat_int_lit(seg, &mut p)? != 1 {
        return None;
    }
    // `10` is the compound pre-decrement (`-= 1` whose VALUE is the test).
    if !eat_byte(seg, &mut p, 0x10) || !eat_int_like(seg, &mut p) {
        return None;
    }
    if !eat_byte(seg, &mut p, 0x39) {
        return None;
    }
    let (back, w) = read_token_var(seg, p)?;
    p += w;
    if back != l_top {
        return None;
    }
    // The loop's break target, then the `if` body's and the `if`'s own closes.
    eat_label(seg, &mut p)?;
    eat_closes_to(seg, &mut p, &mut d2, d)?;
    if eat_label(seg, &mut p)? != l_end {
        return None;
    }
    let mut d = d;
    eat_closes_to(seg, &mut p, &mut d, BODY_SCOPE_DEPTH)?;

    // ---- `*(char**)ptr = 0;` ----------------------------------------------
    eat_opt_stmt_marker(seg, &mut p);
    if eat_ptr_load(seg, &mut p)? != ptr {
        return None;
    }
    eat_ptr_cast(seg, &mut p)?;
    if !eat_byte(seg, &mut p, 0x33) {
        return None;
    }
    eat_ptr4(seg, &mut p)?;
    if read_varint(seg, &mut p)? != 0 {
        return None;
    }
    if !eat_byte(seg, &mut p, 0x32) {
        return None;
    }
    eat_ptr4(seg, &mut p)?;
    if !eat_byte(seg, &mut p, 0x4B) {
        return None;
    }

    // ---- `return this` -----------------------------------------------------
    // The constructor's epilogue: the RETURN plumbing, then `B9 <this> · 41`,
    // which emits nothing — `this` is already in r3 and no statement above moved
    // it. `eat_return_head` owns the `3A`/closes/`29` grammar; the `B9 <this>`
    // that precedes it is this shape's.
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x3A) {
        return None;
    }
    let (_, w) = read_token_var(seg, p)?;
    p += w;
    let mut d = BODY_SCOPE_DEPTH;
    eat_closes_to(seg, &mut p, &mut d, BODY_SCOPE_DEPTH - 1)?;
    eat_label(seg, &mut p)?;
    if eat_ptr_load(seg, &mut p)? != this_tok {
        return None;
    }
    if !eat_byte(seg, &mut p, 0x41) {
        return None;
    }
    eat_ptr4(seg, &mut p)?;
    eat_fn_tail(seg, &mut p).ok()?;

    Some(BodyShape::PoolCtorChain(PoolCtorChain {
        params,
        off,
        align: ALIGN,
    }))
}
