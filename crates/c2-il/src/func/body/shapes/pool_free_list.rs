//! **W-POOL2 — the intrusive free-list PUSH and POP, the two guarded leaves of
//! `src/system/utl/Pool.cpp`.**
//!
//! ```cpp
//!   void *Pool::Alloc() {                 void Pool::Free(void *v) {
//!       void *ptr = mFree;                    if (!v) { return; }
//!       if (!ptr) return nullptr;             *(void **)v = mFree;
//!       mFree = *(char **)ptr;                mFree = (char *)v;
//!       return ptr;                       }
//!   }
//! ```
//!
//! Both are six or seven words, both are leaves, and both fold their guard to a
//! **conditional return** — `bclr 12,26`, `docs/CFG_SHAPE.md` §3.5's fold
//! band 2:
//!
//! ```text
//!   ?Alloc@Pool@@QAAPAXXZ   28 B          ?Free@Pool@@QAAXPAX@Z   24 B
//!     mr     r11,r3                         cmplwi cr6,r4,0
//!     lwz    r3,0(r3)                       bclr   12,26
//!     cmplwi cr6,r3,0                       lwz    r11,0(r3)
//!     bclr   12,26                          stw    r11,0(r4)
//!     lwz    r10,0(r3)                      stw    r4,0(r3)
//!     stw    r10,0(r11)                     blr
//!     blr
//! ```
//!
//! # Board #187 is NOT settled here, and the class is drawn so it cannot be
//!
//! §3.5's band-1 ↔ band-2 boundary is an **OPEN and DECLINED c2 cost model**
//! (board **#187**, and **#2564** for these two functions specifically). Nothing
//! in this file reads a discriminator, and nothing in it needs one: band 1 is
//! *a branchless arithmetic select*, and §3.5's own statement of its
//! precondition is **"both arms are constants … cheap to build from a 0/1 or
//! 0/−1 mask"**. Every body this class admits has a guarded arm that computes
//! **nothing at all** (a bare `return`, or a `return 0` whose value is already
//! in the return register) and a fall-out arm that is a **store sequence** —
//! there is no pair of constants to select between, so band 1 is unreachable
//! *by the class's own precondition* rather than by a fitted rule.
//!
//! §3.5's table carries four rows in that sub-family — `?a_store`
//! (`*p=1` / nothing), `?f_eqvoid` (two stores / nothing), `?Pool::Alloc` and
//! `?Pool::Free` — and all four are `bclr`. Band 3 is excluded for the reason
//! §3.5 gives for it: it is reached "when neither arm can be the
//! fall-through-plus-conditional-return", which requires an arm ending in a
//! transfer that is not the epilogue, or a join. This class requires the guarded
//! arm to be exactly the epilogue jump and admits no join.
//!
//! # `/O1` only, and the gate is in the PARSER (#1638, #1710)
//!
//! At `/Ox` this TU is packed into one `.text` and **`?Alloc` stops folding**:
//! it emits `cmplwi cr6,r3,0 ; bf 26,+8 ; blr ; lwz ; stw ; blr` — band **3**,
//! two `blr`s, seven words — while `?Free` stays band 2. Captured on this lane's
//! own obj (`work/w-pool2/ref/PoolOx.obj`), not inferred. So the mode word is
//! asked **before any body byte is read**, which is board #1638's remedy and
//! the reason `codegen::ptr_walk_loop`'s emitter-only clause is a standing
//! carry item.
//!
//! # What it refuses, and why each refusal is a measurement and not caution
//!
//! * **A guarded arm that computes anything.** The `return 0` of `?Alloc` is
//!   admitted only as the literal `0`: it emits **no instruction**, because the
//!   scrutinee is already in r3 and is 0 on that edge. A different literal is a
//!   different body — `work/w-pool2/probe/` grades the `return (void*)1` cell,
//!   which emits an extra `li r3,1` and a real branch — and is refused here.
//! * **A store run that is not exactly this permutation.** PUSH stores the
//!   member into `*v` and then `v` into the member; POP loads the member into
//!   the local and then `*local` into the member. Both are pinned statement by
//!   statement against the base token, so a body that stores through a second
//!   object, or in the other order, is out of class. The two-statement run is
//!   the very population `leaf_store::collect_store_run` refuses at
//!   `value_is_load` (board **#2563**) — **and this file does not widen that
//!   clause.** It reads the whole body instead, exactly as `w-biquad` #2531
//!   established for the designator layer: the grammar already exists, what was
//!   missing was a production that reads this shape *through* it.
//! * **A member offset that differs between the two designators** of one body.
//!   Both must name the same member; a body touching two members is a different
//!   register plan with no capture.
//! * **A `volatile` pointer anywhere.** `readers::is_volatile_tag` at the
//!   operand LOAD is `GAPS.md` §6's thirteenth instance: c2 homes a volatile
//!   parameter in the frame, so the body is not a leaf at all.

use super::super::expr::{eat_fn_tail, eat_return_head, eat_scopes, BODY_SCOPE_DEPTH};
use super::super::BodyShape;
use super::designator::eat_offset_adds;
use super::params::parse_params;
use crate::func::bundle::{opt_word_at, opt_word_mode, OptWordMode};
use crate::func::readers::{
    eat, eat_byte, eat_opt_stmt_marker, is_ptr4_kind, is_volatile_tag, read_token_var,
    read_type, read_varint,
};
use crate::func::{PoolFreeList, PoolFreeListOp};

/// Consume a width-4 POINTER operand type, refusing `volatile`.
fn eat_ptr4(seg: &[u8], p: &mut usize) -> Option<()> {
    let (tag, kind, _, w) = read_type(seg, *p)?;
    if !is_ptr4_kind(tag, kind) || is_volatile_tag(tag) {
        return None;
    }
    *p += w;
    Some(())
}

/// `B9 <tok> <PTR>` — a pointer value in a register. Returns the token.
fn eat_ptr_load(seg: &[u8], p: &mut usize) -> Option<u32> {
    if !eat_byte(seg, p, 0xB9) {
        return None;
    }
    let (tok, w) = read_token_var(seg, *p)?;
    *p += w;
    eat_ptr4(seg, p)?;
    Some(tok)
}

/// `2C <PTR> 00` — a pointer→pointer reinterpret, which emits nothing.
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

/// The member designator `B9 <this> <PTR> · 33 <int> k 27 <PTR>` — the same
/// offset-add walk [`super::designator::walk_offset_adds`] has consumed since
/// `w-34`, reached through the base pointer rather than through the 2117
/// intrinsic. Returns the byte offset.
///
/// The re-type is **required**: a bare `B9 <this> <PTR>` with no offset chain is
/// a plain pointer value and not a designator, and `eat_offset_adds` returns
/// `Some((0, None))` for it — admitting that would let a body naming `this`
/// itself through the member position.
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

/// Close scopes from `depth` down to `target`, each optionally preceded by its
/// own `4F 01 <line>` marker. The marker skip lives INSIDE the loop for
/// `w-biquad` #2535's reason: a body written with its braces on their own lines
/// carries one marker per close, and a recognizer that skips only once before
/// the run refuses the semantically identical body written on one line.
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

/// `39 <L>` — brTRUE past the guarded arm. Returns the label token.
///
/// The guard is spelled with **no comparison at all**: `if (!p)` over a pointer
/// arrives as a bare LOAD and a `39`, where `if (p == 0)` over an `int` arrives
/// as `B9 · 33 <k> · <rel> · 38`. That is why this class consumes a `39` and
/// every `if`-shaped production in the ladder above consumes a `38`, and it is
/// what makes the ordering between them free.
fn eat_guard_brtrue(seg: &[u8], p: &mut usize) -> Option<u32> {
    if !eat_byte(seg, p, 0x39) {
        return None;
    }
    let (tok, w) = read_token_var(seg, *p)?;
    *p += w;
    Some(tok)
}

/// Try the free-list PUSH and POP. Non-committal in the house style: a cursor
/// copy and an `Option`, so a body that is not this production keeps its own
/// first-blocker census key.
///
/// `depth` is the scope depth at `start` — after `parse_segment_shape` has eaten
/// the body's `53` and every further scope, which for the PUSH form has already
/// consumed the `if`'s own `53` and for the POP form has not (its first
/// statement is an assignment).
pub(crate) fn try_parse_pool_free_list(
    seg: &[u8],
    start: usize,
    lo: usize,
    depth: usize,
) -> Option<BodyShape> {
    // **The optimization word FIRST, before one body byte** — board #1638's
    // remedy and #1710's second instance. §0.5 of this lane's PREREG has the
    // `/Ox` obj that makes it necessary.
    if opt_word_mode(opt_word_at(seg)) != Some(OptWordMode::O1) {
        return None;
    }
    let params = parse_params(seg, lo).ok()?;
    let this_tok = *params.first()?;
    match seg.get(start)? {
        0xB9 => parse_push(seg, start, depth, &params, this_tok),
        0x26 => parse_pop(seg, start, depth, &params, this_tok),
        _ => None,
    }
}

/// `void Pool::Free(void *v) { if (!v) { return; } *(void**)v = mFree; mFree = (char*)v; }`
fn parse_push(
    seg: &[u8],
    start: usize,
    depth: usize,
    params: &[u32],
    this_tok: u32,
) -> Option<BodyShape> {
    // `this` and exactly one pointer formal. The count is pinned because it is
    // what makes the formal's slot index a register number.
    if params.len() != 2 {
        return None;
    }
    let v_tok = params[1];
    // The `if`'s own scope has already been eaten by `parse_segment_shape`, so a
    // body at the plain body depth never had one and is not this shape.
    if depth <= BODY_SCOPE_DEPTH {
        return None;
    }
    let mut p = start;

    // ---- the guard: `if (!v)` --------------------------------------------
    if eat_ptr_load(seg, &mut p)? != v_tok {
        return None;
    }
    let skip = eat_guard_brtrue(seg, &mut p)?;

    // ---- the guarded arm: a bare `return`, and NOTHING else ---------------
    let mut d = depth;
    eat_scopes(seg, &mut p, &mut d).ok()?;
    if d <= depth {
        return None;
    }
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x3A) {
        return None;
    }
    let (epi, w) = read_token_var(seg, p)?;
    p += w;
    eat_closes_to(seg, &mut p, &mut d, depth)?;

    // ---- the join, then the `if` statement's own scope close ---------------
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x29) {
        return None;
    }
    let (lbl, w) = read_token_var(seg, p)?;
    p += w;
    if lbl != skip {
        return None;
    }
    let mut d = depth;
    eat_closes_to(seg, &mut p, &mut d, BODY_SCOPE_DEPTH)?;

    // ---- `*(void**)v = this->m;` ------------------------------------------
    eat_opt_stmt_marker(seg, &mut p);
    if eat_ptr_load(seg, &mut p)? != v_tok {
        return None;
    }
    eat_ptr_cast(seg, &mut p)?;
    let off = eat_member_designator(seg, &mut p, this_tok)?;
    // The member's value: a dereference of the designator, then the reinterpret
    // to the stored type.
    if !eat_byte(seg, &mut p, 0x30) {
        return None;
    }
    eat_ptr4(seg, &mut p)?;
    eat_ptr_cast(seg, &mut p)?;
    if !eat_byte(seg, &mut p, 0x32) {
        return None;
    }
    eat_ptr4(seg, &mut p)?;
    if !eat_byte(seg, &mut p, 0x4B) {
        return None;
    }

    // ---- `this->m = (char*)v;` --------------------------------------------
    eat_opt_stmt_marker(seg, &mut p);
    if eat_member_designator(seg, &mut p, this_tok)? != off {
        return None;
    }
    if eat_ptr_load(seg, &mut p)? != v_tok {
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

    // ---- the epilogue, and it must be the label the guarded arm named -----
    // The `return`'s own `4F 01 <line>` marker. `eat_return_head` does not eat
    // one — it is entered from `parse_segment_shape` only where the dispatch
    // already consumed it — so this shape owns it, exactly as
    // `cond_tail::eat_arm_return` owns the one before its `3A`.
    eat_opt_stmt_marker(seg, &mut p);
    let tail = p;
    eat_return_head(seg, &mut p, false, BODY_SCOPE_DEPTH).ok()?;
    if !names_label(seg, tail, epi) {
        return None;
    }
    eat_fn_tail(seg, &mut p).ok()?;

    Some(BodyShape::PoolFreeList(PoolFreeList {
        params: params.to_vec(),
        op: PoolFreeListOp::Push,
        off,
    }))
}

/// `void *Pool::Alloc() { void *p = mFree; if (!p) return nullptr; mFree = *(char**)p; return p; }`
fn parse_pop(
    seg: &[u8],
    start: usize,
    depth: usize,
    params: &[u32],
    this_tok: u32,
) -> Option<BodyShape> {
    // `this` alone. A formal would occupy r4 and change nothing about this body,
    // which is exactly why it is refused: nothing here graded one.
    if params.len() != 1 {
        return None;
    }
    if depth != BODY_SCOPE_DEPTH {
        return None;
    }
    let mut p = start;

    // ---- `void *local = this->m;` -----------------------------------------
    if !eat_byte(seg, &mut p, 0x26) {
        return None;
    }
    let (local, w) = read_token_var(seg, p)?;
    p += w;
    // A local that shadows a formal token would make the guard's scrutinee
    // ambiguous. `parse_params` returns the formals; `local` must not be one.
    if params.contains(&local) {
        return None;
    }
    let off = eat_member_designator(seg, &mut p, this_tok)?;
    if !eat_byte(seg, &mut p, 0x30) {
        return None;
    }
    eat_ptr4(seg, &mut p)?;
    eat_ptr_cast(seg, &mut p)?;
    if !eat_byte(seg, &mut p, 0x32) {
        return None;
    }
    eat_ptr4(seg, &mut p)?;
    if !eat_byte(seg, &mut p, 0x4B) {
        return None;
    }

    // ---- the guard: `if (!local)` -----------------------------------------
    let mut d = depth;
    eat_scopes(seg, &mut p, &mut d).ok()?;
    if d <= depth {
        return None;
    }
    if eat_ptr_load(seg, &mut p)? != local {
        return None;
    }
    let skip = eat_guard_brtrue(seg, &mut p)?;

    // ---- the guarded arm: `return 0`, and the literal must be ZERO --------
    let mut d2 = d;
    eat_scopes(seg, &mut p, &mut d2).ok()?;
    if d2 <= d {
        return None;
    }
    if !eat_byte(seg, &mut p, 0x33) {
        return None;
    }
    eat_ptr4(seg, &mut p)?;
    // **The whole reason this arm emits nothing.** The scrutinee is already in
    // r3 and the guard proves it is 0 on this edge, so `return 0` and
    // `return local` are the same instruction sequence — the empty one. A
    // non-zero literal is a different body with an extra `li` and a real
    // branch, and it is refused here rather than emitted wrong.
    if read_varint(seg, &mut p)? != 0 {
        return None;
    }
    if !eat_byte(seg, &mut p, 0x41) {
        return None;
    }
    eat_ptr4(seg, &mut p)?;
    if !eat_byte(seg, &mut p, 0x3A) {
        return None;
    }
    let (epi, w) = read_token_var(seg, p)?;
    p += w;
    eat_closes_to(seg, &mut p, &mut d2, d)?;

    // ---- the join, then the `if` statement's own scope close ---------------
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x29) {
        return None;
    }
    let (lbl, w) = read_token_var(seg, p)?;
    p += w;
    if lbl != skip {
        return None;
    }
    let mut d = d;
    eat_closes_to(seg, &mut p, &mut d, BODY_SCOPE_DEPTH)?;

    // ---- `this->m = *(char**)local;` --------------------------------------
    eat_opt_stmt_marker(seg, &mut p);
    if eat_member_designator(seg, &mut p, this_tok)? != off {
        return None;
    }
    if eat_ptr_load(seg, &mut p)? != local {
        return None;
    }
    eat_ptr_cast(seg, &mut p)?;
    if !eat_byte(seg, &mut p, 0x30) {
        return None;
    }
    eat_ptr4(seg, &mut p)?;
    if !eat_byte(seg, &mut p, 0x32) {
        return None;
    }
    eat_ptr4(seg, &mut p)?;
    if !eat_byte(seg, &mut p, 0x4B) {
        return None;
    }

    // ---- `return local;` ---------------------------------------------------
    eat_opt_stmt_marker(seg, &mut p);
    if eat_ptr_load(seg, &mut p)? != local {
        return None;
    }
    let tail = p;
    eat_return_head(seg, &mut p, true, BODY_SCOPE_DEPTH).ok()?;
    // `eat_return_head` consumed `41 <T> 3A <tok>`; the epilogue label is the
    // token after the `41 <T>`, and it must be the one the guarded arm jumped to
    // or the two returns leave by different exits.
    let mut q = tail;
    if !(eat_byte(seg, &mut q, 0x41) && eat_ptr4(seg, &mut q).is_some()) {
        return None;
    }
    if !names_label(seg, q, epi) {
        return None;
    }
    eat_fn_tail(seg, &mut p).ok()?;

    Some(BodyShape::PoolFreeList(PoolFreeList {
        params: params.to_vec(),
        op: PoolFreeListOp::Pop,
        off,
    }))
}

/// True when the `3A <tok>` at `at` names `want`. Read rather than re-consumed,
/// so [`eat_return_head`] stays the single owner of the return plumbing's
/// grammar and this file only asks which label it named.
fn names_label(seg: &[u8], at: usize, want: u32) -> bool {
    let mut q = at;
    if !eat_byte(seg, &mut q, 0x3A) {
        return false;
    }
    matches!(read_token_var(seg, q), Some((tok, _)) if tok == want)
}
