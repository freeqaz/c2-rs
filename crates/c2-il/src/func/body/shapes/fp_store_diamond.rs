//! **W-BIQUAD — the null-guarded `if`/`else` whose arms are FLOAT MEMBER
//! STORES**, and the port's first body that pools **two** constants and places
//! their `lis` in **two different blocks**.
//!
//! ```cpp
//!   void Biquad::SetCoefficients(float *flts) {
//!       if (flts == nullptr) {
//!           coefs[4] = 0.0f; coefs[3] = 0.0f; coefs[2] = 0.0f;
//!           coefs[1] = 0.0f; coefs[0] = 1.0f;
//!       } else {
//!           coefs[0] = flts[0] / flts[3];
//!           coefs[1] = flts[1] / flts[3];
//!           coefs[2] = flts[2] / flts[3];
//!           coefs[3] = flts[4] / flts[3];
//!           coefs[4] = flts[5] / flts[3];
//!       }
//!       coefs[6] = 0.0f;
//!       coefs[5] = 0.0f;
//!   }
//! ```
//!
//! This is `src/system/synth_xbox/Biquad.cpp`, the FRONTIER TU `w-park`
//! (#1923) declined at fifteen and `w-band` (#2240–#2247) independently
//! confirmed is **strictly deeper than `mmio.cpp`**, inverting the byte-fraction
//! ranking. Its sibling is the constructor
//! ([`super::ctor_forward_call`]); the TU converts on both or on neither.
//!
//! ## The designator layer was never the blocker, and that is worth stating
//!
//! The census reports this body at **`expr-op-0x27`** once the relational and
//! branch sinks are on — the byte-offset add that re-types a designator — and
//! that key is the **#1 row on the workload at 22,412 emitted functions**. It is
//! *not* what stops this body. [`super::designator::walk_offset_adds`] has
//! consumed `33 <int-like> k · 27 <PTR>` and `33 <int-like> k · 28 00 00` since
//! `w-34`, and this recognizer reads every designator in the body through it
//! without adding one byte of grammar. `expr-op-0x27` is
//! [`super::super::expr::parse_expr`]'s **fall-through** arm: it says *"no shape
//! claimed this body"*, which is a statement about the shape ladder and not
//! about the operand stream. `w-readpx` (`WB_READER_FINDINGS` §3.3) had already
//! priced its grammar cost at NONE and ranked it a lowering with **0 TUs**;
//! `w-dclass` §6.1 measured its head worth **six functions and zero TUs**.
//!
//! **So the size of that key is not this class's worth, and no part of this file
//! may be quoted as unblocking it.**
//!
//! ## What the reference emits, and the four things a lowering gets wrong
//!
//! Read off the real obj at the workload's own flags
//! (`work/w-biquad/real.obj`), 35 words:
//!
//! ```text
//!   0x00  lis    r11,__real@00000000    REFHI      A's `lis`, in the ENTRY block
//!   0x04  cmplwi cr6,r4,0
//!   0x08  lfs    f0,0(r11)              REFLO      A's `lfs`, also in the entry
//!   0x0c  bne    cr6,+0x24  -> 0x30
//!   0x10  lis    r11,__real@3f800000    REFHI      B's `lis`, TOP OF THE ARM
//!   0x14  stfs   f0,16(r3)
//!   0x18  stfs   f0,12(r3)
//!   0x1c  stfs   f0,8(r3)
//!   0x20  stfs   f0,4(r3)
//!   0x24  lfs    f13,0(r11)             REFLO      B's `lfs`, AT THE USE
//!   0x28  stfs   f13,0(r3)
//!   0x2c  b      +0x54      -> 0x80
//!   0x30  lfs    f12,12(r4)  ┐ divisor first …
//!   0x34  lfs    f13,0(r4)   │
//!   0x38  fdivs  f13,f13,f12 │  × 4
//!   0x3c  stfs   f13,0(r3)   ┘
//!   …
//!   0x70  lfs    f13,20(r4)  ┐ … and on the LAST division the operands SWAP
//!   0x74  lfs    f12,12(r4)  │
//!   0x78  fdivs  f13,f13,f12 │
//!   0x7c  stfs   f13,16(r3)  ┘
//!   0x80  stfs   f0,24(r3)              the join, still holding A in f0
//!   0x84  stfs   f0,20(r3)
//!   0x88  blr
//! ```
//!
//! 1. **B-RULE** (`WB_CHOOSER_FINDINGS` §3.3, 3 entry-block and 6 block-local
//!    witnesses): one `lis` per pool symbol per function, at the top of the
//!    **earliest basic block that dominates every use**. `A` is used in the
//!    then-arm *and* after the join, so it dominates from the entry; `B` is used
//!    once, in the then-arm, so its `lis` is that block's first word. A port that
//!    transcribed *"the pooled `lis` is the function's first word"* from this obj
//!    alone is wrong on `WB_CHOOSER`'s cell B1.
//! 2. **The `lfs` is at the USE, not with the `lis`** — `B`'s two halves are five
//!    words apart, with four unrelated stores between them.
//! 3. **B′-RULE** (§4.1, 5 flip witnesses and 15 non-flip): a CSE'd reload is
//!    loaded **first** in every statement of the run **except the last**, where
//!    the operands go in source order. Getting this backwards is two wrong words
//!    in the last division and nothing anywhere else — the quietest possible
//!    mis-emit.
//! 4. **`f0` is live across the whole diamond.** It is loaded in the entry block
//!    and read by the join, so the else-arm may not use it; the divisions take
//!    `f13`/`f12`.
//!
//! **B-RULE-2 is NOT used here** and that is deliberate. The compare/branch
//! separation slot is `medium` at exactly three witnesses, and #260's warning
//! applies to a clause with that history: the entry block's word order is
//! transcribed from this class's own obj and no clause in this file or in
//! `c2_core::codegen::fp_store_diamond` consults a separation rule. A cell that
//! needed one would be declined, not fitted.
//!
//! ## What refuses, and why each refusal has a witness or an absence
//!
//! * **`/O1` only**, asked before any body byte is read — the mode gate lives in
//!   the parser (board #1638), so the census cannot count a body `PortC2`
//!   refuses.
//! * **Exactly two formals**, `this` and one pointer, read through
//!   [`super::params::parse_params`] so `this` is counted: a shape that read the
//!   guard formal as r3 would compare the wrong register.
//! * **Exactly two distinct constants**, with `A` in the join and `B` used
//!   exactly once as the **last** then-arm store. Three pools would need a
//!   dominator computation and a third scratch GPR, and `WB_CHOOSER`'s B5 says
//!   two pools in one block take r11 **then r10** — a third has no witness at
//!   all.
//! * **At least two divisions**, all sharing one divisor offset. With one
//!   division the B′-RULE flip is unobservable (the only statement is also the
//!   last), so a single-division body would be admitted on an ambiguity rather
//!   than on the rule.
//! * **Every store is a 4-byte real**, and every literal is a `float` literal of
//!   size 4. A `double` member is a different store instruction and a different
//!   `.rdata` size, and no cell has been graded on one.

use super::super::expr::{eat_return_plumbing, parse_formals, BODY_SCOPE_DEPTH};
use super::super::{blk, BodyShape, Block};
use super::designator::eat_offset_adds;
use super::params::parse_params;
use crate::func::bundle::{opt_word_at, opt_word_mode, OptWordMode};
use crate::func::readers::{
    eat_byte, eat_opt_stmt_marker, is_fp_type, read_token_var, read_type, read_varint,
    FLOAT_LIT_TYPE,
};
use crate::func::{FpDiamondConstStore, FpDiamondDiv, FpStoreDiamond};

/// The lexical depth an arm's two `53`s reach, from the body's own depth.
const ARM_DEPTH: u8 = 5;

/// Consume `54 <k>`, requiring the exact depth `k`. The depths are pinned rather
/// than decoded for [`super::if_call_join::try_parse_if_call_join`]'s reason:
/// they are the only place the source's *bracing* shows up in this stream, and a
/// differently braced body is a different block plan.
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

/// Consume a TYPE naming a **4-byte real** — `float`, never `double`.
fn eat_float4(seg: &[u8], p: &mut usize, what: &'static str) -> Result<(), Block> {
    match read_type(seg, *p) {
        Some((tag, kind, _, w)) if is_fp_type(tag, kind) == Some(false) => {
            *p += w;
            Ok(())
        }
        _ => Err(blk(seg, *p, what)),
    }
}

/// `B9 <tok> <TYPE>` followed by an offset-add run — a **designator** — through
/// [`super::designator::eat_offset_adds`], the crate's one locator for that
/// walk. Returns the summed byte offset.
///
/// A recognizer that parsed `27`/`28` itself here would be the reinvention this
/// module's siblings are warned about in `shapes/mod.rs`'s header, and it would
/// be a second place for the overflow check, the `28 00 00` payload rule and the
/// stop condition to drift.
fn eat_member_designator(
    seg: &[u8],
    p: &mut usize,
    base: u32,
    what: &'static str,
) -> Result<i32, Block> {
    if !eat_byte(seg, p, 0xB9) {
        return Err(blk(seg, *p, what));
    }
    let (tok, w) = read_token_var(seg, *p).ok_or(blk(seg, *p, what))?;
    *p += w;
    if tok != base {
        return Err(blk(seg, *p, what));
    }
    let (_, _, _, tw) = read_type(seg, *p).ok_or(blk(seg, *p, what))?;
    *p += tw;
    let (off, _) = eat_offset_adds(seg, p).ok_or(blk(seg, *p, what))?;
    Ok(off)
}

/// `33 <FLOAT-LIT-TYPE> <8 bytes binary64, little-endian> <u16 width>` — a
/// **float** constant, returned as the binary64 bits the IL carries.
///
/// The payload is a binary64 pattern even for a `float` (already rounded to
/// binary32 precision) and the trailing `u16` is the operand *width*, which must
/// agree with the literal tag — the same reading
/// [`super::leaf_float::try_parse_float_leaf`] uses, restated at the one site
/// that needs it rather than shared, because that reader's copy is inside a
/// postfix walk this shape does not have.
fn eat_float_literal(seg: &[u8], p: &mut usize, what: &'static str) -> Result<u64, Block> {
    if !eat_byte(seg, p, 0x33) {
        return Err(blk(seg, *p, what));
    }
    if seg.get(*p..*p + 3) != Some(&FLOAT_LIT_TYPE[..]) {
        return Err(blk(seg, *p, what));
    }
    *p += 3;
    let raw: [u8; 8] = seg
        .get(*p..*p + 8)
        .and_then(|s| s.try_into().ok())
        .ok_or(blk(seg, *p, what))?;
    *p += 8;
    let size = seg
        .get(*p..*p + 2)
        .and_then(|s| s.try_into().ok())
        .map(u16::from_le_bytes)
        .ok_or(blk(seg, *p, what))?;
    *p += 2;
    if size != 4 {
        return Err(blk(seg, *p, what));
    }
    Ok(u64::from_le_bytes(raw))
}

/// One constant store statement: `<this designator> <float literal> 32 <float> 4B`.
fn eat_const_store(
    seg: &[u8],
    p: &mut usize,
    this: u32,
    what: &'static str,
) -> Result<FpDiamondConstStore, Block> {
    eat_opt_stmt_marker(seg, p);
    let off = eat_member_designator(seg, p, this, what)?;
    let bits = eat_float_literal(seg, p, what)?;
    if !eat_byte(seg, p, 0x32) {
        return Err(blk(seg, *p, what));
    }
    eat_float4(seg, p, what)?;
    if !eat_byte(seg, p, 0x4B) {
        return Err(blk(seg, *p, what));
    }
    Ok(FpDiamondConstStore { off, bits })
}

/// One division statement:
/// `<this designator> <src designator> 30 <float> <src designator> 30 <float> 05 32 <float> 4B`.
fn eat_div_store(
    seg: &[u8],
    p: &mut usize,
    this: u32,
    src: u32,
    what: &'static str,
) -> Result<FpDiamondDiv, Block> {
    eat_opt_stmt_marker(seg, p);
    let off = eat_member_designator(seg, p, this, what)?;
    let num = eat_member_designator(seg, p, src, what)?;
    if !eat_byte(seg, p, 0x30) {
        return Err(blk(seg, *p, what));
    }
    eat_float4(seg, p, what)?;
    let den = eat_member_designator(seg, p, src, what)?;
    if !eat_byte(seg, p, 0x30) {
        return Err(blk(seg, *p, what));
    }
    eat_float4(seg, p, what)?;
    // `05` is DIV and this class takes no other operator: `fdivs` is the one
    // instruction whose operand order B′-RULE is about, and a `+`/`-`/`*` run
    // has no reload to order.
    if !eat_byte(seg, p, 0x05) {
        return Err(blk(seg, *p, what));
    }
    if !eat_byte(seg, p, 0x32) {
        return Err(blk(seg, *p, what));
    }
    eat_float4(seg, p, what)?;
    if !eat_byte(seg, p, 0x4B) {
        return Err(blk(seg, *p, what));
    }
    Ok(FpDiamondDiv { off, num, den })
}

/// **The recognizer.** `start` is the first byte after the body's own `53`, any
/// leading line marker and the `if` statement's own scope open; `lo` is the
/// `4C 4F 11` body marker.
///
/// Non-committal in the sense every sibling production here is: it works on its
/// own cursor and returns `Err` on the first byte that is not its grammar, so a
/// body that declines still reports its dispatch arm's blocker and no census key
/// moves.
pub(crate) fn try_parse_fp_store_diamond(
    seg: &[u8],
    start: usize,
    lo: usize,
) -> Result<BodyShape, Block> {
    // **THE MODE GATE LIVES HERE, IN THE PARSER** (board #1638), asked before
    // any body byte is read so the refusal cannot depend on how far the walk
    // got. At `/Ox` and `/O2` c2 tail-duplicates a join block rather than
    // sharing it behind a `b`; this class's 35 words are a `/O1` body and
    // nothing else, and a gate that lived only in the emitter would make the
    // census count a body `PortC2` refuses.
    if opt_word_mode(opt_word_at(seg)) != Some(OptWordMode::O1) {
        return Err(blk(seg, start, "fpdiamond-not-o1"));
    }
    let params = parse_params(seg, lo)?;
    // `parse_params`, not `parse_formals`, so a non-static member's `this` is
    // counted — the guard compares r4 and the stores are through r3, and a
    // reader that lost `this` would emit both against the wrong register.
    if params.len() != 2 || parse_formals(seg, lo)?.len() != 1 {
        return Err(blk(seg, start, "fpdiamond-formals-not-this-plus-1"));
    }
    let (this, src) = (params[0], params[1]);

    let mut p = start;

    // ---- the guard: `if (src == 0)`, branching FALSE to the else arm -------
    if !eat_byte(seg, &mut p, 0xB9) {
        return Err(blk(seg, p, "fpdiamond-guard-load"));
    }
    let (tok, w) = read_token_var(seg, p).ok_or(blk(seg, p, "fpdiamond-guard-tok"))?;
    p += w;
    if tok != src {
        return Err(blk(seg, p, "fpdiamond-guard-not-the-formal"));
    }
    let (gtag, gkind, gid, gw) = read_type(seg, p).ok_or(blk(seg, p, "fpdiamond-guard-type"))?;
    p += gw;
    // The null literal carries the SAME pointer type as the operand it is
    // compared against. Requiring the triple to match rather than merely
    // requiring "a literal" is what makes `flts == nullptr` distinguishable from
    // a comparison against some other pointer-typed constant.
    if !eat_byte(seg, &mut p, 0x33) {
        return Err(blk(seg, p, "fpdiamond-guard-lit"));
    }
    match read_type(seg, p) {
        Some((t, k, id, w2)) if (t, k, id) == (gtag, gkind, gid) => p += w2,
        _ => return Err(blk(seg, p, "fpdiamond-guard-lit-type")),
    }
    let k = read_varint(seg, &mut p).ok_or(blk(seg, p, "fpdiamond-guard-lit-value"))?;
    if k != 0 {
        return Err(blk(seg, p, "fpdiamond-guard-lit-not-null"));
    }
    // `1F` is `==`; `38` is branch-on-FALSE, so the emitted `bc` carries the
    // negation and names the ELSE arm, and the then-arm is the fall-through.
    if !eat_byte(seg, &mut p, 0x1F) {
        return Err(blk(seg, p, "fpdiamond-guard-rel-not-eq"));
    }
    let l_else = eat_transfer(seg, &mut p, 0x38, "fpdiamond-guard-branch")?;

    // ---- the THEN arm: constant stores through `this` ----------------------
    if !eat_byte(seg, &mut p, 0x53) || !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "fpdiamond-then-scopes"));
    }
    let mut then_stores: Vec<FpDiamondConstStore> = Vec::new();
    while seg.get(p) == Some(&0xB9) || is_marker_then(seg, p, 0xB9) {
        then_stores.push(eat_const_store(seg, &mut p, this, "fpdiamond-then-store")?);
    }
    if then_stores.len() < 2 {
        return Err(blk(seg, p, "fpdiamond-then-fewer-than-2-stores"));
    }
    eat_opt_stmt_marker(seg, &mut p);
    eat_close(seg, &mut p, ARM_DEPTH, "fpdiamond-then-close-5")?;
    eat_close(seg, &mut p, ARM_DEPTH - 1, "fpdiamond-then-close-4")?;
    let l_join = eat_transfer(seg, &mut p, 0x3A, "fpdiamond-then-jump")?;
    if eat_label(seg, &mut p, "fpdiamond-else-label")? != l_else {
        return Err(blk(seg, p, "fpdiamond-else-label"));
    }

    // ---- the ELSE arm: a CSE'd division run --------------------------------
    if !eat_byte(seg, &mut p, 0x53) || !eat_byte(seg, &mut p, 0x53) {
        return Err(blk(seg, p, "fpdiamond-else-scopes"));
    }
    let mut divs: Vec<FpDiamondDiv> = Vec::new();
    while seg.get(p) == Some(&0xB9) || is_marker_then(seg, p, 0xB9) {
        divs.push(eat_div_store(seg, &mut p, this, src, "fpdiamond-div")?);
    }
    if divs.len() < 2 {
        return Err(blk(seg, p, "fpdiamond-fewer-than-2-divisions"));
    }
    // **One divisor for the whole run** — that is what makes it a CSE run and
    // what B′-RULE is a statement about. A run whose divisors differ has no
    // reload to order and is not this body.
    if divs.iter().any(|d| d.den != divs[0].den) {
        return Err(blk(seg, p, "fpdiamond-divisors-differ"));
    }
    eat_opt_stmt_marker(seg, &mut p);
    eat_close(seg, &mut p, ARM_DEPTH, "fpdiamond-else-close-5")?;
    eat_close(seg, &mut p, ARM_DEPTH - 1, "fpdiamond-else-close-4")?;
    if eat_label(seg, &mut p, "fpdiamond-join-label")? != l_join {
        return Err(blk(seg, p, "fpdiamond-join-label"));
    }
    eat_close(seg, &mut p, ARM_DEPTH - 2, "fpdiamond-join-close-3")?;

    // ---- the JOIN: more constant stores through `this` ---------------------
    let mut join_stores: Vec<FpDiamondConstStore> = Vec::new();
    while seg.get(p) == Some(&0xB9) || is_marker_then(seg, p, 0xB9) {
        join_stores.push(eat_const_store(seg, &mut p, this, "fpdiamond-join-store")?);
    }
    if join_stores.is_empty() {
        return Err(blk(seg, p, "fpdiamond-empty-join"));
    }

    // ---- the return plumbing, which is where acceptance is decided ---------
    // Landing exactly on it is the whole acceptance claim: a walk that ends
    // anywhere else consumed a byte it did not understand.
    //
    // The marker is skipped HERE rather than inside `eat_return_head`, which
    // requires its `3A` immediately: the last statement's own `4F 01 <line>` is
    // the source line of the closing `}` and belongs to the run that just
    // ended, not to the plumbing.
    eat_opt_stmt_marker(seg, &mut p);
    eat_return_plumbing(seg, &mut p, false, BODY_SCOPE_DEPTH)?;

    if l_else == l_join {
        return Err(blk(seg, p, "fpdiamond-labels-alias"));
    }

    // ---- the two-pool shape, which is the whole of B-RULE here -------------
    //
    // `A` is the constant whose uses span the then-arm AND the join, so it
    // dominates from the entry block; `B` is used exactly once, as the LAST
    // then-arm store, so its `lis` is that block's first word. Everything the
    // emitter does with placement follows from these two sentences, so they are
    // established positively here rather than re-derived there.
    let b_bits = then_stores[then_stores.len() - 1].bits;
    let a_bits = join_stores[0].bits;
    if a_bits == b_bits {
        return Err(blk(seg, p, "fpdiamond-one-pool"));
    }
    if then_stores[..then_stores.len() - 1].iter().any(|s| s.bits != a_bits) {
        return Err(blk(seg, p, "fpdiamond-then-not-one-constant"));
    }
    if join_stores.iter().any(|s| s.bits != a_bits) {
        return Err(blk(seg, p, "fpdiamond-join-not-one-constant"));
    }

    Ok(BodyShape::FpStoreDiamond(FpStoreDiamond {
        params,
        then_stores,
        divs,
        join_stores,
    }))
}

/// True when `p` is a line marker whose next byte is `want` — the statement
/// boundary this class's runs are separated by. The run loops ask it so that a
/// marker cannot be mistaken for the end of a run.
fn is_marker_then(seg: &[u8], p: usize, want: u8) -> bool {
    let mut q = p;
    eat_opt_stmt_marker(seg, &mut q);
    q != p && seg.get(q) == Some(&want)
}
