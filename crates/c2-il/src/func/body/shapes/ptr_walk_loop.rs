//! **The pointer-walk accumulate loop** — `?HashString@@YAHPBDH@Z`'s shape, and
//! the first body class in this parser with a **back edge**.
//!
//! ```c
//!   int P(const char *str, int i) {
//!       int ret = K0;
//!       for (unsigned char *u = (unsigned char *)str; *u != 0; u++)
//!           ret = (*u + ret * K) % i;
//!       return ret;
//!   }
//! ```
//!
//! # Why this class is drawn as narrowly as it is
//!
//! It is drawn around **one workload function** on purpose. `w-tu1`'s technique
//! — commit to a TU, build a byte-exact base, add one construct at a time and
//! grade each against real `c2` before the next — is the only one that has ever
//! produced a conversion here, and the reason it works is that every cell inside
//! the class is *measured*, not deduced.
//!
//! What is **inside** the class, each axis graded over its own cross product by
//! `work/w-hash/hashgrid.py` against real `c2`:
//!
//! * the multiplier `K` — any `mulli`-eligible positive literal (fits `simm16`,
//!   is not a power of two, not `0`, not `1`);
//! * the accumulator's initial literal `K0` — any `simm16`.
//!
//! Those two are the emitter's only free fields, so they are graded over their
//! **cross product** and not two rows through the origin:
//! `work/w-hash/crossgrade.py` compiles **49** `(K0, K)` cells with real `c2`
//! and every one comes back `match`, beside **30** must-refuse cells (`K` a
//! power of two / `0` / `1` / `-1` / negative / above `simm16`, and ten
//! structural variants) which all come back `vocab-gap`, **0 mismatches**.
//!
//! # Three spellings I registered as in-class that are NOT, and why
//!
//! An earlier draft of this comment claimed the sentinel spelling, the
//! accumulate's operand order and the pointer's source spelling were "the same
//! IL". **That was wrong, measured, and is corrected here rather than beside
//! the right claim.** `c2` emits byte-identical `.text` for all three — which is
//! exactly why the assumption was easy to make and why it had to be graded — but
//! each is a *different IL production*, so this recognizer refuses them:
//!
//! ```text
//!   *u  instead of  *u != 0
//!       … 30 82 12 20 · 38 <exit>          the test branches on the RAW byte:
//!                                          no `2C` widening, no `33 <int> 00`,
//!                                          no `20` NE opcode at all
//!   ret*K + *u  instead of  *u + ret*K
//!       … b9 <ret> · 33 <int> K · 04 · b9 <u> · 30 · 2C · 02 · …
//!                                          the postfix stream really is
//!                                          reordered; the deref is the ADD's
//!                                          right operand, not its left
//!   const unsigned char* with no cast
//!       … b9 <str> <ptr> 32 <ptr> 4B       no `2C` in the initializer at all,
//!       … 30 a2 12 80 20                   and the element TYPE is the
//!                                          const-qualified `A2 12` with a wide
//!                                          id, not the plain `82 12`
//! ```
//!
//! Each is a small, well-defined widening and none is taken here: a second test
//! form, a second operand order and a second element type are three separate
//! productions, each owing its own graded cross product, and the refusal
//! direction is the safe one.
//!
//! What is **outside**, and every one of these has a measured counterexample
//! rather than a conservative guess (`docs/rungs/*w-hash*` §4):
//!
//! * **the pointer formal anywhere but slot 0**, or the divisor anywhere but
//!   slot 1, or a third formal. `swap`, `divfirst` and `p3` each re-plan the
//!   register assignment: the accumulator coalesces into `r3` and the guard
//!   becomes a `beqlr` instead of a forward `bc` plus a closing `mr r3,rA`.
//!   That is a different **block plan**, not a different register field;
//! * **a pointer formal used directly** as the induction variable, with no
//!   local copy (`nocast`) — same re-plan;
//! * **`K` a power of two** (`rlwinm`, and the trap moves), **`K == 1`**,
//!   **`K == 0`**, **`K` negative** (`c2` rewrites `x + a*-3` as `x - a*3`),
//!   **`K` above `simm16`** (`lis`/`ori`/`mullw`);
//! * **an unsigned or literal divisor** — both are a *different spine*
//!   (`divwu` + one `twi`; `li` + `divw` + `mulli` + `subf`), measured and
//!   recorded in the rung, and deliberately not shipped here;
//! * **`/` in place of `%`**, a **stride other than 1**, a **wider element
//!   type**, an accumulator init outside `simm16`, and `*u > 0` in place of
//!   `*u != 0` — each graded as a must-refuse cell in
//!   `work/w-hash/crossgrade.py`.
//!
//! Each of those refusals costs coverage and the cost is a number, not an
//! argument: the class is exactly what has been graded.
//!
//! # The label counter, and why this shape returns `None` from `label_slots`
//!
//! `w-loop` measured that a **leaf** loop charges the compiler-label counter
//! `+1..+4` exactly as a framed one does, and that the charge is unobservable in
//! a TU with no framed function (34 of 34, control 17 of 17) because `$M`/`$T`
//! short names are the only channel to the obj and `plan_labels` mints them for
//! framed functions alone.
//!
//! So this shape is **safe alone and unmeasured in company**. `label_slots`
//! returns `None` for it, which is the three-valued gate's *refuse* answer:
//! `IlBundle::functions` then rejects any TU that pairs one of these with a
//! framed function, and admits one that does not. That is board **#746**'s
//! placement, and it needed the caller this shape provides — the objection
//! `w-loop` raised against building it early was that a permissive `resolve`
//! with no back-edge variant to call it would be an ungraded code path by
//! construction.

use crate::func::body::expr::{eat_return_plumbing, parse_formals};
use crate::func::body::{blk, Block, BodyShape};
use crate::func::readers::{
    eat, eat_byte, eat_int_like, eat_opt_stmt_marker, read_token_var, read_type, read_varint,
    is_int1u_type, is_ptr4_kind,
};
use crate::func::PtrWalkModLoop;

/// The scope depth the body opens at, mirrored from `expr::BODY_SCOPE_DEPTH`.
/// PROV[N] derived — mirrored from `expr::BODY_SCOPE_DEPTH`.
const BODY_SCOPE_DEPTH: usize = 2;

/// `mulli` is the whole multiply, measured over 38 constants × both
/// commutations at `/O1` (`work/w-hash/mulgrid.py`): `k` fits `simm16`, is not
/// `0`, not `±1`, and is not `±2^n`. Outside that set `c2` strength-reduces
/// (`rlwinm`, `neg`+`rlwinm`) or materializes (`lis`/`ori`/`mullw`), which is a
/// different instruction count and therefore a different block.
///
/// Positive by construction: the predicate says which `k` this shape *accepts*,
/// and everything else refuses.
pub(crate) fn is_mulli_literal(k: i32) -> bool {
    if !(-0x8000..=0x7FFF).contains(&k) || k == 0 || k == 1 || k == -1 {
        return false;
    }
    let mag = (k as i64).unsigned_abs();
    mag & (mag - 1) != 0
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

/// Consume `B9 <tok> <TYPE>` for a **width-4 integer** operand and return the token.
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

/// Consume a TYPE naming a width-4 **pointer**.
fn eat_ptr_type(seg: &[u8], p: &mut usize, what: &'static str) -> Result<u32, Block> {
    match read_type(seg, *p) {
        Some((tag, kind, id, w)) if is_ptr4_kind(tag, kind) => {
            *p += w;
            Ok(id)
        }
        _ => Err(blk(seg, *p, what)),
    }
}

/// Consume `B9 <ptrTok> <ptr TYPE> · 30 <one-byte-unsigned TYPE> · 2C <int TYPE>
/// <varint>` — the `(int)*u` operand, three tokens, and refuse anything else.
///
/// The element type is required to be the **one-byte unsigned** class
/// (`is_int1u_type`), not merely "some type": the emitted instruction is `lbz` /
/// `lbzu`, and a `short` or `int` element is `lhz`/`lwz` with a different stride
/// in the update form. A class that admitted the type without checking its width
/// would emit a byte load for a word walk — the shape of GAPS §6's repeated
/// one-field-two-facts defect.
fn eat_deref_char(
    seg: &[u8],
    p: &mut usize,
    ptr_tok: u32,
    ptr_type_id: u32,
    what: &'static str,
) -> Result<(), Block> {
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
        Some((tag, kind, _, w)) if is_int1u_type(tag, kind) => *p += w,
        _ => return Err(blk(seg, *p, what)),
    }
    // `2C <TYPE> <varint>` — the widening to `int`. Its own opcode in this IL,
    // never implicit.
    if !eat_byte(seg, p, 0x2C) {
        return Err(blk(seg, *p, what));
    }
    if !eat_int_like(seg, p) {
        return Err(blk(seg, *p, what));
    }
    read_varint(seg, p).ok_or(blk(seg, *p, what))?;
    Ok(())
}

/// **The recognizer.** `start` is the first byte after the body's own `53` and
/// any leading scope/line markers; `lo` is the `4C 4F 11` body marker.
///
/// Non-committal in the sense every sibling production here is: it works on its
/// own cursor and every failure returns `Err` with the cursor untouched by the
/// caller's reckoning, so a body that declines still reports the blocker its own
/// dispatch arm found.
pub(crate) fn try_parse_ptr_walk_loop(
    seg: &[u8],
    start: usize,
    lo: usize,
    locals: &[u32],
    ptr_locals: &[u32],
) -> Result<BodyShape, Block> {
    let params = parse_formals(seg, lo)?;
    // **Exactly two formals, pointer first.** Everything about the emitted block
    // plan is measured at this arity and this order; `swap`, `divfirst` and `p3`
    // each emit a different plan (module docs).
    if params.len() != 2 {
        return Err(blk(seg, start, "loop-formals-not-2"));
    }
    let (src_tok, div_tok) = (params[0], params[1]);

    let mut p = start;

    // ---- statement 1: `acc = K0` ------------------------------------------
    let acc_tok = eat_designator(seg, &mut p, "loop-acc-designator")?;
    if !locals.contains(&acc_tok) {
        // The accumulator must be an automatic local `.sy` knows about — the
        // same membership test `assign.rs` uses, and for the same reason: a
        // file-scope `static int` is a memory object and folding its store away
        // would drop a real store.
        return Err(blk(seg, p, "loop-acc-not-a-local"));
    }
    if !eat_byte(seg, &mut p, 0x33) || !eat_int_like(seg, &mut p) {
        return Err(blk(seg, p, "loop-acc-init-lit"));
    }
    let acc_init = read_varint(seg, &mut p).ok_or(blk(seg, p, "loop-acc-init-varint"))?;
    if !(-0x8000..=0x7FFF).contains(&acc_init) {
        // `li rA,K0` is one instruction only inside `simm16`; a wider literal is
        // a `lis`/`ori` pair and a different block.
        return Err(blk(seg, p, "loop-acc-init-wide"));
    }
    if !eat_byte(seg, &mut p, 0x32) || !eat_int_like(seg, &mut p) || !eat_byte(seg, &mut p, 0x4B) {
        return Err(blk(seg, p, "loop-acc-init-store"));
    }

    // ---- the `for` scope opens, then `u = (unsigned char *)str` -----------
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "loop-for-scope"));
    }
    eat_opt_stmt_marker(seg, &mut p);
    let ptr_tok = eat_designator(seg, &mut p, "loop-ptr-designator")?;
    // **Positively** an automatic width-4 data pointer whose address is never
    // taken — `.sy`'s own answer, not `.gl`'s absence. A file-scope pointer
    // would make `u = str` and `u++` real memory writes, and folding them into
    // an `mr` plus an `lbzu` would drop both.
    if ptr_tok == acc_tok || !ptr_locals.contains(&ptr_tok) {
        return Err(blk(seg, p, "loop-ptr-not-a-local"));
    }
    // `B9 <src formal> <ptr TYPE>` — the initializer reads formal 0 and nothing
    // else. A pointer arriving from anywhere but the first argument register
    // would need a different `mr`.
    if !eat_byte(seg, &mut p, 0xB9) {
        return Err(blk(seg, p, "loop-ptr-init-load"));
    }
    let (t, w) = read_token_var(seg, p).ok_or(blk(seg, p, "loop-ptr-init-tok"))?;
    p += w;
    if t != src_tok {
        return Err(blk(seg, p, "loop-ptr-init-not-formal0"));
    }
    eat_ptr_type(seg, &mut p, "loop-ptr-init-srctype")?;
    // The cast to `unsigned char *`. It is a **reinterpret** between two width-4
    // pointers, which emits nothing — the `mr` that follows is the copy, not the
    // conversion.
    if !eat_byte(seg, &mut p, 0x2C) {
        return Err(blk(seg, p, "loop-ptr-init-convert"));
    }
    let ptr_type_id = eat_ptr_type(seg, &mut p, "loop-ptr-init-dsttype")?;
    read_varint(seg, &mut p).ok_or(blk(seg, p, "loop-ptr-init-convert-varint"))?;
    if !eat_byte(seg, &mut p, 0x32) {
        return Err(blk(seg, p, "loop-ptr-init-store"));
    }
    if eat_ptr_type(seg, &mut p, "loop-ptr-init-storetype")? != ptr_type_id {
        return Err(blk(seg, p, "loop-ptr-init-storetype"));
    }
    if !eat_byte(seg, &mut p, 0x4B) {
        return Err(blk(seg, p, "loop-ptr-init-end"));
    }

    // ---- the ROTATION: `3A Ltest` · `29 Lincr` · `u += 1` · `29 Ltest` ------
    //
    // This is `w-loop`'s mechanism #9 read off the stream rather than off the
    // obj: the IL jumps **over** the increment into the test, so the increment
    // block physically precedes the test block and the source's single test site
    // is emitted twice — once as the entry guard, once as the loop-closing
    // branch.
    if !eat_byte(seg, &mut p, 0x3A) {
        return Err(blk(seg, p, "loop-entry-jump"));
    }
    let (l_test, w) = read_token_var(seg, p).ok_or(blk(seg, p, "loop-entry-jump-tok"))?;
    p += w;
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x29) {
        return Err(blk(seg, p, "loop-incr-label"));
    }
    let (l_incr, w) = read_token_var(seg, p).ok_or(blk(seg, p, "loop-incr-label-tok"))?;
    p += w;
    if l_incr == l_test {
        return Err(blk(seg, p, "loop-labels-alias"));
    }
    eat_opt_stmt_marker(seg, &mut p);
    if eat_designator(seg, &mut p, "loop-incr-designator")? != ptr_tok {
        return Err(blk(seg, p, "loop-incr-not-the-pointer"));
    }
    // `33 <TYPE> 01` — the literal 1, then `35 <ptr TYPE>`, the compound
    // add-assign. The stride must be **1**: `lbzu rN,1(rU)` folds exactly this
    // increment into the addressing mode and nothing else.
    if !eat_byte(seg, &mut p, 0x33) {
        return Err(blk(seg, p, "loop-incr-lit"));
    }
    // The stride literal's TYPE is the *element* type of the pointer arithmetic
    // (`86 41 12` in the workload's own capture), not the `int` operand type, so
    // it is read for its width and not classified.
    match read_type(seg, p) {
        Some((_, _, _, w)) => p += w,
        None => return Err(blk(seg, p, "loop-incr-lit-type")),
    }
    if read_varint(seg, &mut p) != Some(1) {
        return Err(blk(seg, p, "loop-incr-stride-not-1"));
    }
    if !eat_byte(seg, &mut p, 0x35) {
        return Err(blk(seg, p, "loop-incr-op"));
    }
    if eat_ptr_type(seg, &mut p, "loop-incr-type")? != ptr_type_id {
        return Err(blk(seg, p, "loop-incr-type"));
    }
    if !eat_byte(seg, &mut p, 0x4B) {
        return Err(blk(seg, p, "loop-incr-end"));
    }

    // ---- the TEST: `29 Ltest` · `(int)*u != 0` · `38 Lexit` ----------------
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x29) {
        return Err(blk(seg, p, "loop-test-label"));
    }
    let (t, w) = read_token_var(seg, p).ok_or(blk(seg, p, "loop-test-label-tok"))?;
    p += w;
    if t != l_test {
        return Err(blk(seg, p, "loop-test-label-mismatch"));
    }
    eat_deref_char(seg, &mut p, ptr_tok, ptr_type_id, "loop-test-deref")?;
    if !eat_byte(seg, &mut p, 0x33) || !eat_int_like(seg, &mut p) {
        return Err(blk(seg, p, "loop-test-lit"));
    }
    if read_varint(seg, &mut p) != Some(0) {
        return Err(blk(seg, p, "loop-test-not-vs-zero"));
    }
    // `20` — the NE relation. Its sibling `22` is LT and would be a different
    // branch condition; the byte is required literally.
    if !eat_byte(seg, &mut p, 0x20) {
        return Err(blk(seg, p, "loop-test-not-ne"));
    }
    if !eat_byte(seg, &mut p, 0x38) {
        return Err(blk(seg, p, "loop-test-brfalse"));
    }
    let (l_exit, w) = read_token_var(seg, p).ok_or(blk(seg, p, "loop-exit-tok"))?;
    p += w;
    if l_exit == l_test || l_exit == l_incr {
        return Err(blk(seg, p, "loop-labels-alias"));
    }

    // ---- the BODY: `acc = ((int)*u + acc * K) % div` -----------------------
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "loop-body-scope"));
    }
    eat_opt_stmt_marker(seg, &mut p);
    if eat_designator(seg, &mut p, "loop-body-designator")? != acc_tok {
        return Err(blk(seg, p, "loop-body-not-the-accumulator"));
    }
    eat_deref_char(seg, &mut p, ptr_tok, ptr_type_id, "loop-body-deref")?;
    if eat_int_load(seg, &mut p, "loop-body-acc-load")? != acc_tok {
        return Err(blk(seg, p, "loop-body-acc-load"));
    }
    if !eat_byte(seg, &mut p, 0x33) || !eat_int_like(seg, &mut p) {
        return Err(blk(seg, p, "loop-body-mul-lit"));
    }
    let mul_k = read_varint(seg, &mut p).ok_or(blk(seg, p, "loop-body-mul-varint"))?;
    // **`mul_k > 0` on top of the `mulli` predicate, and the extra clause is a
    // measured refusal rather than caution.** `c2` *does* emit one `mulli` for a
    // negative non-power-of-two — but it also rewrites `x + a*(-3)` as
    // `x - a*3`, so the accumulate's `add` becomes a `subf` and the emitted body
    // differs by an opcode this shape does not carry (`work/w-hash/hashgrid.py`
    // row `K=-3`). The predicate and the clause are separate because the
    // predicate is about the multiply and the clause is about the *fold*.
    if !is_mulli_literal(mul_k) || mul_k <= 0 {
        return Err(blk(seg, p, "loop-body-mul-not-mulli"));
    }
    // `04` MUL then `02` ADD — postfix, so the multiply binds `acc * K` and the
    // add folds `(int)*u` into it. The two bytes are required in this order:
    // swapping them is a different expression tree.
    if !eat(seg, &mut p, &[0x04, 0x02]) {
        return Err(blk(seg, p, "loop-body-not-mul-then-add"));
    }
    if eat_int_load(seg, &mut p, "loop-body-div-load")? != div_tok {
        return Err(blk(seg, p, "loop-body-div-not-formal1"));
    }
    // `06` MOD. `05` is DIV and is a different (shorter) spine — measured, not
    // shipped.
    if !eat_byte(seg, &mut p, 0x06) {
        return Err(blk(seg, p, "loop-body-not-mod"));
    }
    if !eat_byte(seg, &mut p, 0x32) || !eat_int_like(seg, &mut p) || !eat_byte(seg, &mut p, 0x4B) {
        return Err(blk(seg, p, "loop-body-store"));
    }

    // ---- the BACK EDGE: `54 04` · `3A Lincr` ------------------------------
    eat_opt_stmt_marker(seg, &mut p);
    if !eat(seg, &mut p, &[0x54, (BODY_SCOPE_DEPTH + 2) as u8]) {
        return Err(blk(seg, p, "loop-body-scope-close"));
    }
    if !eat_byte(seg, &mut p, 0x3A) {
        return Err(blk(seg, p, "loop-back-edge"));
    }
    let (t, w) = read_token_var(seg, p).ok_or(blk(seg, p, "loop-back-edge-tok"))?;
    p += w;
    if t != l_incr {
        return Err(blk(seg, p, "loop-back-edge-target"));
    }

    // ---- the EXIT: `29 Lexit` · `return acc` ------------------------------
    eat_opt_stmt_marker(seg, &mut p);
    if !eat_byte(seg, &mut p, 0x29) {
        return Err(blk(seg, p, "loop-exit-label"));
    }
    let (t, w) = read_token_var(seg, p).ok_or(blk(seg, p, "loop-exit-label-tok"))?;
    p += w;
    if t != l_exit {
        return Err(blk(seg, p, "loop-exit-label-mismatch"));
    }
    eat_opt_stmt_marker(seg, &mut p);
    if eat_int_load(seg, &mut p, "loop-return-load")? != acc_tok {
        return Err(blk(seg, p, "loop-return-not-the-accumulator"));
    }
    // The shared tail: `41 <int> · 3A <lbl> · 54 03 · 54 02 · 29 <lbl> · 4F 12 …`,
    // and the fail-closed terminal — anything trailing rejects.
    eat_return_plumbing(seg, &mut p, true, BODY_SCOPE_DEPTH + 1)?;

    Ok(BodyShape::PtrWalkModLoop(PtrWalkModLoop {
        params,
        acc_init,
        mul_k,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The `mulli` predicate, over the whole constant axis `work/w-hash/mulgrid.py`
    /// measured at `/O1`. Every accepted `k` is a cell that read a single
    /// `mulli` from real `c2`; every rejected one read something else, and the
    /// comment says which.
    #[test]
    fn mulli_literal_is_the_measured_set_and_nothing_else() {
        // graded ACCEPT — one `mulli` word
        for k in [3, 5, 6, 7, 9, 10, 15, 17, 63, 65, 100, 127, 255, 1000, 32767] {
            assert!(is_mulli_literal(k), "{k} reads one `mulli` from c2");
        }
        // graded REFUSE, with the instruction c2 emits instead
        for (k, why) in [
            (0i32, "li rD,0"),
            (1, "identity, no instruction at all"),
            (-1, "neg"),
            (2, "rlwinm"),
            (4, "rlwinm"),
            (8, "rlwinm"),
            (16, "rlwinm"),
            (64, "rlwinm"),
            (128, "rlwinm"),
            (256, "rlwinm"),
            (32768, "rlwinm"),
            (65536, "rlwinm"),
            (-2, "neg ; rlwinm"),
            (-4, "neg ; rlwinm"),
            (-8, "neg ; rlwinm"),
            (-128, "neg ; rlwinm"),
            (-32768, "neg ; rlwinm"),
            (65535, "addis ; ori ; mullw"),
            (100000, "addis ; ori ; mullw"),
            (-32769, "addis ; ori ; mullw"),
        ] {
            assert!(!is_mulli_literal(k), "c2 emits {why} for {k}, not one mulli");
        }
        // The negative `mulli` cells: c2 *does* emit one `mulli` for a negative
        // non-power-of-two, so the predicate admits them — and the loop shape
        // still refuses them, at the ACCUMULATE and under its own clause,
        // because `x + a*(-3)` is rewritten as `x - a*3` and the emitted body
        // has a `subf` where this one has an `add`. Two facts, two gates.
        assert!(is_mulli_literal(-3));
        assert!(is_mulli_literal(-7));
        assert!(is_mulli_literal(-127));
    }

    /// **W-FENCEB — the label charge, pinned where it can be seen.** This class
    /// used to return `None` from `IlFunction::label_slots` (board **#746**,
    /// fence B) and now charges a lead of **2**, so a TU pairing it with a
    /// framed function is admitted instead of refused whole. The number is not
    /// fitted to any rule: it is read out of `fixtures/cpp/whash_loop_then_framed.cpp`'s
    /// own reference obj, whose framed `?z9` sits at `$M2564` against a
    /// charge-0 base of 2562.
    ///
    /// **`docs/LABEL_COUNTER.md` §4.2.1 publishes `+3` for this shape and it is
    /// ONE HIGH** (board **#3091**). That is asserted here as a distinct value
    /// rather than mentioned, because a lead of 3 is a live `mismatch` against
    /// real `c2.dll` and the published table would licence it.
    ///
    /// Asserted at BOTH values of `fn_level_linking`: a leaf's slots do not
    /// depend on `/Gy`, so neither spelling of the question may drift.
    #[test]
    fn the_pointer_walk_loop_charges_a_lead_of_two_and_not_the_published_three() {
        let f = crate::func::IlFunction {            body: crate::func::BodyShape::PtrWalkLoop(PtrWalkModLoop {
                params: vec![0xE3, 0xE4],
                acc_init: 0,
                mul_k: 0x7F,
            }),
            ..crate::func::IlFunction::base("?HashString@@YAHPBDH@Z", &None)
        };
        assert_eq!(f.label_lead(), 2, "the obj says 2; §4.2.1's table says 3");
        assert_eq!(f.label_slots(false), Some(3));
        assert_eq!(f.label_slots(true), Some(3));
        // `IlBundle::functions`' three-valued gate is
        // `label_slots(false)? != label_lead() + 1`, and `coff::plan_labels`
        // advances exactly `label_lead + 1` for a non-framed function. Asserting
        // the RELATION and not only the number is what keeps the two halves of
        // the lift from drifting apart into six wrong bytes.
        assert_eq!(f.label_slots(false), Some(f.label_lead() + 1));
        // The separating control: the same builder without the field is an
        // ordinary leaf at lead 0, so the 2 above is this shape's answer and not
        // the builder's.
        let plain = crate::func::IlFunction::base("?g@@YAHH@Z", &None);
        assert_eq!(plain.label_lead(), 0);
        assert_eq!(plain.label_slots(false), Some(1));
    }
}
