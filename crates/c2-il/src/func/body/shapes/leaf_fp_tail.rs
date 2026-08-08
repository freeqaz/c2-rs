//! **The floating-point tail call** — `return g(x1, …, xn);` and `g(x1, …, xn);`
//! where at least one argument is an FP formal, the whole body, no frame.
//!
//! Two rungs, one recognizer, because they differ only in how many times the
//! argument production repeats: W31 is one FP argument and W34
//! (`docs/rungs/2026-07-31-fp-multiarg.md`) is two or more, where the setup
//! becomes a permutation of the FP file broken through **f0**.
//!
//! The load-bearing restriction of the second is **no GPR argument moves** —
//! not "every argument is floating-point", which is where W31's handoff put it
//! and which is one step short. c2 *interleaves* the two files' move sequences
//! (`docs/CODEGEN_FP_ARGS.md` §1.1) on a rule no per-file solver reproduces, but
//! a marshalling with no GPR moves in it has nothing to interleave with, and the
//! capture confirms the FP half is then byte-identical to the pure-FP one
//! (§1.2.1). So a general-purpose argument is admitted here exactly when it is
//! already in the register the call wants, which costs no instruction at all —
//! and refused the moment it is not.
//!
//! This is the FP twin of [`super::calls::parse_call_shape`]'s integer tail
//! call, and it is a separate recognizer for one reason: the *argument* grammar
//! differs. `parse_expr`'s operand vocabulary is the width-4 integer/pointer one
//! and cannot spell an FP value at all — every body in this class blocks at its
//! `B9`'s type under `expr-load-type-8645`/`-8885`. The call **head** is shared
//! (`calls::eat_call_head`), because a second copy of the head decode is the
//! drift `docs/GAPS.md` §6 instance #9 records — and because the `call-conv`
//! byte lives in it, which matters more here than for the integer form: a
//! varargs callee places an FP argument in a GPR pair *as well as* in the FP
//! file, and a head decode without that gate emits half of it.
//!
//! Everything below is bytes out of a real obj (`cl.exe` 16.00.11886.00 under
//! wibo, `/O1 /GS- /c`), captured with `scripts/gt_capture.sh` and read with
//! `scripts/gt_dump.py`. The register rules themselves are **not** re-derived
//! here: they are `docs/CODEGEN_FP_ARGS.md` §0/§1, reached through the one
//! locator [`crate::func::sy::SyView::arg_classes`].
//!
//! ```text
//!   float  a1(float a)                     { return g1f(a); }  48000000  b g1f
//!   float  a2(float a, float b)            { return g1f(b); }  fc201090  fmr f1,f2
//!   float  a3(float a, float b, float c)   { return g1f(c); }  fc201890  fmr f1,f3
//!   float  a4(int k, float b)              { return g1f(b); }  (nothing)  b g1f
//!   float  a5(float a, int k, float b)     { return g1f(b); }  fc201090  fmr f1,f2
//!   double a8(float a, double b)           { return g1d(b); }  fc201090  fmr f1,f2
//!   int    b1(float a, float b)            { return gif(b); }  fc201090  fmr f1,f2
//!   void   b2(float a, float b)            { gvf(b); }         fc201090  fmr f1,f2
//!   float  C::m(float a, float b) const    { return g1f(b); }  fc201090  fmr f1,f2
//! ```
//!
//! `a4`/`a5` are the discriminators for the numbering and `C::m` for `this`:
//! the FP file counts the **FP parameters alone**, and `this` — which takes r3 —
//! displaces nothing in it, so the member function is byte-identical to its free
//! twin. `b1`/`b2` say the *result* class is not this rung's business: nothing is
//! emitted for it either way.
//!
//! ## The two conversions, which are not symmetric
//!
//! ```text
//!   double w1(float a)            { return g1d(a); }  48000000  b g1d       NOTHING
//!   double w2(float a, float b)   { return g1d(b); }  fc201090  fmr f1,f2   just the move
//!   float  n1(double a)           { return g1f(a); }  fc200818  frsp f1,f1
//!   float  n2(double a, double b) { return g1f(b); }  fc201018  frsp f1,f2  <- fused
//! ```
//!
//! `float`→`double` at the argument boundary is **free** (an FPR already holds
//! double), and `double`→`float` is a real `frsp` — `docs/CODEGEN_FP_ARGS.md` §2.
//! `n2` is the row worth having: the narrowing does **not** decompose into
//! `fmr f1,f2 ; frsp f1,f1`; c2 fuses the move into the `frsp`'s operands. A port
//! that emitted the move first would be wrong by one instruction on every
//! narrowing call from anything but f1.
//!
//! ## What refuses, and why each one is a refusal rather than a gap
//!
//! Each of these was captured and emits something this rung does not model:
//!
//! ```text
//!   float  r2(double a) { return g1d(a); }   FRAMED: bl ?g1d ; frsp f1,f1 ; epilogue
//!   float  x1(int a)    { return g1f(a); }   FRAMED: extsw/std/lfd/fcfid/frsp ; bl
//!   int    x2(float a)  { return g1i(a); }   FRAMED: fctiwz/stfd/lwz ; bl
//!   float  c1(float a, float b) { return g1f(a + b); }   fadds f1,f1,f2 ; b
//!   float  l1()         { return g1f(1.5f); }            lis/lfs through .rdata ; b
//! ```
//!
//! * a conversion applied to the **result** (`4C 2C <TYPE> 00 41`) is refused
//!   whichever direction it goes. `r1` (`double r1(float a){ return g1f(a); }`)
//!   really is a bare `b` — the widening is as free there as it is at the
//!   argument — but its narrowing twin `r2` is a whole frame, and the two are
//!   told apart only by comparing the `2C` target against the CALL token's
//!   return TYPE. That is one more field than this rung reads; §"Found and not
//!   taken" in the rung doc sizes it.
//! * a conversion **across** the register files (`int`↔`float`) is a frame in
//!   both directions — `fcfid` and `fctiwz` both round-trip through the stack —
//!   so it is refused by requiring the LOAD's type to be FP *and* the `.sy`
//!   formal to be `ArgClass::Fp`.
//! * a **computed** argument (`c1`) is emittable in principle — it is the W13
//!   float leaf's own selector with `f1` as the destination — but it is a
//!   different lowering with its own contraction and constant gates, so it is a
//!   rung and not a clause.
//! * an FP **literal** argument (`l1`) costs an `.rdata` COMDAT, a REFHI/REFLO
//!   pair and a GPR, and is refused under `/Gy` by codegen anyway.

use crate::func::body::expr::{eat_return_plumbing, parse_formals, BODY_SCOPE_DEPTH};
use crate::func::body::BodyShape;
use crate::func::readers::{
    eat_byte, eat_fp_type, eat_int_like_or_ptr4, read_token_var,
};
use crate::func::sy::{fp_reg_of, gpr_reg_of, ArgClass, SyView};

use super::calls::eat_call_head;
use super::this_binding::{parse_this_token, ThisBinding};

/// The most FP parameters a body in this class may declare. Past `f13` a
/// floating-point formal is stack-homed and reading it is an `lfs`/`lfd` off r1,
/// not a register move — the same boundary the W13 leaf and the FP store draw,
/// and for the same reason (`docs/CODEGEN_FP_ARGS.md` §5: the 14-parameter
/// capture frames and spills, so it is not a leaf at all).
const MAX_FP_FORMALS: usize = 13;

/// The most arguments a call in this class may carry. Past the eighth an
/// argument is stack-homed and needs a frame — the same boundary
/// `super::calls`' `MAX_REGISTER_FORMALS` draws on the formals side.
const MAX_ARGS: usize = 8;

/// One parsed call argument: its token, and which register file its `.ex` TYPE
/// puts it in. `Some((src_double, want_double))` is a floating-point value with
/// the widths on either side of an optional boundary conversion; `None` is a
/// width-4 int-like or pointer value, which this production admits only when it
/// is already in the register the call wants.
struct Arg {
    tok: u32,
    fp: Option<(bool, bool)>,
}

/// Try to parse a **floating-point tail call**, positioned at the `26 <callee>`
/// symbol push that opens the body. One argument is [`BodyShape::FpTailCall`]
/// (W31) and two or more are [`BodyShape::FpMultiArgTailCall`] (W34).
///
/// ```text
///   26 <callee> BD <ret TYPE> 00 <fn-type-id>     the shared call head
///   (                                             one or more arguments,
///     B9 <tok> <TYPE>                               each a bare formal LOAD
///     [ 2C <FP TYPE> 00 ]                           …FP only, converted FP→FP
///     55 <TYPE>                                     the callee's formal type
///   )+                                            …in REVERSE source order
///   4C                                            end of the argument region
///   ( 4B <void plumbing>                          the result is discarded
///   | 41 <scalar TYPE> <void plumbing> )          …or it is the return value
/// ```
///
/// Non-committal, like every other recognizer in this directory: it works on a
/// copy of the cursor and returns `None` with no side effects, so a body that
/// declines still reports its own blocker through
/// [`super::calls::parse_call_shape`] and no census key moves.
///
/// The result annotation is consumed **here** rather than through
/// [`eat_return_plumbing`]'s own `41` gate, because that gate is
/// [`eat_int_like_or_ptr4`] — deliberately not widened to the FP classes
/// (`docs/ROADMAP.md` §6d) — and this production has to admit `41 <float>` for
/// `float f(float a){ return g1f(a); }`. Nothing is *emitted* for the annotation
/// in any case: the callee's return value is already in the register the caller's
/// own return uses, whichever file it is in. It is still gated to the scalar
/// classes rather than skipped, so a by-value aggregate return — which is an
/// sret pointer and a different ABI — fails closed.
pub(crate) fn try_parse_fp_tail_call(
    seg: &[u8],
    start: usize,
    lo: usize,
    sy: SyView,
) -> Option<BodyShape> {
    let mut p = start;
    // The FP tail call is the one shape whose own body IS floating point, so a
    // real result here is expected rather than refused: `fp_tail` marks the
    // function and `_fltused` follows from that.
    let (callee_tok, _ret) = eat_call_head(seg, &mut p).ok()?;

    // ---- the argument region ------------------------------------------------
    // `( B9 <tok> <TYPE> [ 2C <FP TYPE> 00 ] 55 <TYPE> )+ 4C`, arguments in
    // **reverse source order** — rightmost first, the same convention
    // [`super::calls::parse_call_shape`] anchors on `parse_formals`' reversal.
    // Every argument is a bare formal LOAD; a computed one is `float_leaf_text`'s
    // selector in argument position and a different lowering (W31's rung doc).
    //
    // A **GPR** argument is admitted here and carries no conversion: it is
    // checked below to be already in the register the call wants, so it emits
    // nothing. That is what keeps this production out of the two-file
    // interleaving question — a schedule with no GPR moves in it has nothing to
    // interleave, and the capture confirms the FP half is then byte-identical to
    // the pure-FP one (`docs/CODEGEN_FP_ARGS.md` §1.2).
    let mut stream: Vec<Arg> = Vec::new();
    while !eat_byte(seg, &mut p, 0x4C) {
        if !eat_byte(seg, &mut p, 0xB9) {
            return None;
        }
        let (arg_tok, w) = read_token_var(seg, p)?;
        p += w;
        let mut q = p;
        let arg = if let Some(src_double) = eat_fp_type(seg, &mut q) {
            p = q;
            // An optional FP→FP conversion of the argument. `2C <TYPE target> <varint 0>`,
            // the same production `parse_expr` admits for the class-preserving integer
            // case — required to be literally `00` for the same reason: a field that never
            // varied across the captures is indistinguishable from a constant.
            let mut want_double = src_double;
            if eat_byte(seg, &mut p, 0x2C) {
                want_double = eat_fp_type(seg, &mut p)?;
                if !eat_byte(seg, &mut p, 0x00) {
                    return None;
                }
            }
            // `55 <TYPE>` carries the **callee's declared formal type**, and it must agree
            // in width with whatever the conversion (or its absence) left on the stack.
            // Compared by decoded width and not byte-for-byte, because the two positions
            // are free to differ in cv-qualification and a `const float` parameter emits
            // the identical instruction.
            if !eat_byte(seg, &mut p, 0x55) {
                return None;
            }
            if eat_fp_type(seg, &mut p)? != want_double {
                return None;
            }
            Arg { tok: arg_tok, fp: Some((src_double, want_double)) }
        } else {
            // A general-purpose argument: the same width-4 int-like/pointer
            // vocabulary `parse_expr` spells, and **no `2C` at all** — a
            // conversion in this file is an `extsb`/`rlwinm`/`extsw`, i.e. an
            // instruction, and this production's whole claim about GPR arguments
            // is that they cost none.
            eat_int_like_or_ptr4(seg, &mut p)?;
            if !eat_byte(seg, &mut p, 0x55) {
                return None;
            }
            eat_int_like_or_ptr4(seg, &mut p)?;
            Arg { tok: arg_tok, fp: None }
        };
        stream.push(arg);
        // Past the eighth an argument is stack-homed, which needs a frame; the
        // `4C` gate above is the real terminator and this only keeps a malformed
        // segment from being walked forever.
        if stream.len() > MAX_ARGS {
            return None;
        }
    }
    if stream.is_empty() {
        return None; // the bare `b <callee>`; `parse_call_shape` owns it
    }
    // Source order: slot `i` is `stream[len - 1 - i]`.
    stream.reverse();

    // ---- the tail: the result is discarded, or it is the return value --------
    if eat_byte(seg, &mut p, 0x4B) {
        // `void f(float a){ g(a); }`. A single statement call with nothing after
        // it IS a tail call — measured, and the same fact `parse_call_shape`
        // turns on: emitting a frame for it would be a mis-emit, not a gap.
        eat_return_plumbing(seg, &mut p, false, BODY_SCOPE_DEPTH).ok()?;
    } else {
        if !eat_byte(seg, &mut p, 0x41) {
            // Anything else here is a post-op on the result — most importantly a
            // `2C`, the result conversion whose narrowing half is a whole frame.
            return None;
        }
        // The result annotation: an int-like or 4-byte pointer value, or an FP
        // one. Nothing is emitted for it; the gate exists so an unmodeled return
        // class fails closed rather than being branched over.
        if eat_int_like_or_ptr4(seg, &mut p).is_none() && eat_fp_type(seg, &mut p).is_none() {
            return None;
        }
        eat_return_plumbing(seg, &mut p, false, BODY_SCOPE_DEPTH).ok()?;
    }

    // ---- the FP register file ------------------------------------------------
    // Read through the ONE locator, `arg_classes`, which is all-or-nothing over
    // the whole formals list: an FP parameter changes the numbering of every
    // parameter after it in both files, so one formal of unknown class makes every
    // later one unknown too.
    let formals = parse_formals(seg, lo).ok()?;
    let classes = sy.arg_classes(&formals).ok()?;
    // `this` is not in the FP file at all — it takes r3 and displaces nothing
    // (`C::m` above is byte-identical to its free twin) — but an *undetermined*
    // `this` binding still refuses, exactly as `params::parse_params` does. It
    // never silently means "absent" (`docs/GAPS.md` §6 instances #1/#2), and a
    // segment whose pre-body region cannot be read is one whose formals list is
    // not established either.
    //
    // It is also read for a second fact: the base of the caller's own **GPR**
    // argument numbering — r3 for a free function and r4 for a member, because
    // `this` takes r3 and shifts every explicit formal up one (`sy::gpr_reg_of`).
    // One read, two consumers, and they are two consequences of the same binding
    // rather than two facts sharing a name.
    let gpr_base: u8 = match parse_this_token(seg, lo)? {
        ThisBinding::Absent => 3,
        ThisBinding::Bound(_) => 4,
    };
    // The FP parameters, in FP-file order: entry `n` is `f(n+1)`.
    let params: Vec<u32> = formals
        .iter()
        .zip(&classes)
        .filter(|(_, c)| matches!(c, ArgClass::Fp { .. }))
        .map(|(t, _)| *t)
        .collect();
    if params.len() > MAX_FP_FORMALS {
        return None;
    }
    // Walk the arguments in call order, sorting them into the two files.
    //
    // **A GPR argument is admitted only when it does not move.** Its destination
    // is `r(2 + slot)` where `slot` is its 1-based position in the *call* — FP
    // arguments consume a slot there too — and its source is `r(base + ix)` in
    // the caller's own numbering. When those agree the argument costs nothing,
    // and a marshalling with no GPR moves in it has nothing for the FP moves to
    // interleave with: the capture shows the FP half is then byte-identical to
    // the pure-FP permutation (`docs/CODEGEN_FP_ARGS.md` §1.2). When they
    // disagree — `int f(int a,int b,float c,float d){ return g(a,c,b,d); }` is
    // one `mr r5,r4` — the body declines, and the rung doc prices it.
    let mut arg_sources = Vec::new();
    let mut fp_stream = Vec::new();
    for (slot, arg) in stream.iter().enumerate() {
        let ix = formals.iter().position(|&t| t == arg.tok)?;
        match (arg.fp, classes[ix]) {
            // `.ex` says this value is a `float`/`double`; `.sy` says the formal is one.
            // They are two channels on one fact and a disagreement is a misread record,
            // never a width to guess at — the same all-or-nothing discipline `arg_classes`
            // itself applies.
            (Some((src_double, want_double)), ArgClass::Fp { double }) if double == src_double => {
                arg_sources.push(fp_reg_of(&classes, ix)? as usize - 1);
                fp_stream.push((arg.tok, src_double, want_double));
            }
            (None, ArgClass::Gpr) => {
                let src = gpr_reg_of(&classes, ix, gpr_base)?;
                let dst = u8::try_from(2 + slot + 1).ok()?;
                if src != dst {
                    return None;
                }
            }
            // A conversion **across** the register files (`int`↔`float`), or a
            // `.ex`/`.sy` disagreement about the width. Both are frames, and
            // neither is ever a width to guess at.
            _ => return None,
        }
    }
    // No floating-point argument at all: `parse_call_shape`'s integer productions
    // own that body, and this recognizer must not take it — its own file has
    // nothing to say and it would lose the integer path's argument grammar.
    if fp_stream.is_empty() {
        return None;
    }

    if fp_stream.len() == 1 {
        let (arg_tok, src_double, want_double) = fp_stream[0];
        return Some(BodyShape::FpTailCall {
            params,
            arg_tok,
            // `double`→`float` at the boundary is a real `frsp`, and it is **fused**
            // with the register move rather than following one (`n2` above).
            narrowing: src_double && !want_double,
            callee_tok,
        });
    }
    let stream = fp_stream;

    // ---- two or more arguments: the FP file's permutation ---------------------
    // `docs/CODEGEN_FP_ARGS.md` §1.2. Everything below is a gate the complete
    // n = 2…5 grid (`scripts/gt_fpperm.py --pure --model`) puts a boundary on;
    // each one is a shape c2 emits differently, never conservatism.
    //
    // **No narrowing anywhere in the list.** `double`→`float` inside a
    // permutation is fused into whichever move writes the destination — but with
    // *every* argument of a 3-cycle narrowing, c2 changes the schedule outright
    // and parks a second value:
    //
    // ```text
    //   f(double a,b,c) -> g3(double,double,double)(b,c,a)
    //       fmr f0,f2 ; fmr f2,f3 ; fmr f3,f1 ; fmr f1,f0
    //   f(double a,b,c) -> g3(float,float,float)(b,c,a)
    //       fmr f0,f2 ; fmr f13,f3 ; frsp f3,f1 ; frsp f1,f0 ; frsp f2,f13
    // ```
    //
    // Five moves and two scratches against four and one, from a type change
    // alone. The rule that produces it is not characterized, and W31 measured the
    // single-argument `frsp`'s census value at **0**, so the whole conversion is
    // refused here rather than modeled from the cases that happen to fuse.
    if stream.iter().any(|&(_, src, want)| src && !want) {
        return None;
    }
    // A value passed twice is not a permutation at all — it is a copy graph, and
    // `float f(float a,float b){ return g2f(a,a); }` is the single word
    // `fmr f2,f1` with no scratch anywhere in it. The GPR file refuses the same
    // shape (`call-arg-duplicated`), where it emits a *dead* move through the
    // temp instead; the two files do not even agree on what a duplicate costs.
    for i in 0..arg_sources.len() {
        if arg_sources[..i].contains(&arg_sources[i]) {
            return None;
        }
    }
    // **A source FP register above the destination count.**
    // `float f(float a,float b,float c){ return g2(b,c); }` wants f1←f2, f2←f3 —
    // a shift out of a register the call does not otherwise write, not a
    // permutation of the destinations. The GPR file refuses the same shape
    // (`call-arg-outer-formal`), where it was also a panic.
    if arg_sources.iter().any(|&ix| ix >= arg_sources.len()) {
        return None;
    }
    // **At most one local minimum**, i.e. at most one scratch — none at all for
    // the identity, which is every value already in place and emits nothing.
    // Measured over the complete n = 2…5 grid: with one scratch the emission is fully determined and the model is
    // exact on every cell of n = 2…5; with two it is not, and the residue is the
    // same independent-chain interleaving `docs/CODEGEN_ARG_PERM.md` §2.1 leaves
    // open in the other file (26 of 120 cells at n = 5, in both files).
    if fp_perm_local_minima(&arg_sources) > 1 {
        return None;
    }
    Some(BodyShape::FpMultiArgTailCall { params, arg_sources, callee_tok })
}

/// The number of **local minima** over the cycles of the destination→source map
/// `sources` — which is the number of scratch FP registers c2 parks into, and
/// therefore the boundary of the modeled class.
///
/// `sources[i]` is the FP-file index of the value destination `f(i+1)` wants, so
/// σ(i) = `sources[i]` as a permutation of the destinations. Write a cycle as the
/// cyclic sequence `c0, c1 = σ(c0), …`; `ci` is a local minimum when
/// `c(i-1) > ci < c(i+1)`, cyclically. A fixed point is not a cycle and
/// contributes nothing.
///
/// Measured, not assumed: `docs/CODEGEN_ARG_PERM.md` §2 establishes the count for
/// the GPR file over 152 cells and `docs/CODEGEN_FP_ARGS.md` §1.2 does the same
/// for the FP file. One minimum implies exactly one non-trivial cycle, so a
/// separate multi-cycle gate would be redundant.
pub(crate) fn fp_perm_local_minima(sources: &[usize]) -> usize {
    let n = sources.len();
    let mut seen = vec![false; n];
    let mut minima = 0usize;
    for start in 0..n {
        if seen[start] || sources[start] == start {
            seen[start] = true;
            continue;
        }
        let mut cycle = Vec::new();
        let mut at = start;
        while !seen[at] {
            seen[at] = true;
            cycle.push(at);
            at = sources[at];
            // An entry outside the destination range cannot close a cycle. The
            // caller has already refused that shape; this is the backstop that
            // keeps a misread record from indexing out of bounds — the GPR twin
            // of this walk *panicked* on exactly that, and the CLI must degrade
            // cleanly (`docs/GAPS.md` §6, `call-arg-outer-formal`).
            if at >= n {
                return usize::MAX;
            }
        }
        let k = cycle.len();
        for i in 0..k {
            if cycle[(i + k - 1) % k] > cycle[i] && cycle[i] < cycle[(i + 1) % k] {
                minima += 1;
            }
        }
    }
    minima
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::func::body::{parse_segment, parse_segment_detail};
    use crate::func::sy::{Formals, SyFormal};

    /// A `.sy` view declaring exactly these formals. The FP file cannot be read
    /// off `.ex` at all — `float` and `int` are both 4 bytes and the formals
    /// region carries tokens and no types — so a test that used
    /// [`Formals::AllOneRegisterByConstruction`] would grade every parameter as a
    /// GPR and this whole production would decline. Stating the classes here is
    /// what makes the numbering the thing under test.
    fn sy(formals: &'static [SyFormal]) -> SyView<'static> {
        SyView { locals: &[], ptr_locals: &[], addr_locals: &[], formals: Formals::Declared(formals) }
    }
    const FLOAT: u8 = 0x45;
    const DOUBLE: u8 = 0x85;
    const INT: u8 = 0x41;

    /// `float a5(float a, int k, float b) { return g1f(b); }` — whole captured
    /// segment from `fixtures/cpp/w31_fp_tail.cpp`, which the differential grades
    /// byte-exact at `fmr f1,f2 ; b ?g1f`.
    ///
    /// **The discriminator for the FP file.** `b` is the THIRD formal and the
    /// SECOND floating-point one, so the shape must name f2. A positional model
    /// names f3 and emits an `fmr` c2 does not — `docs/CODEGEN_FP_ARGS.md` §1,
    /// where the same confusion was two of this project's live wrong-bytes emits.
    const FP_TAIL_GAP: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x40, 0x53, 0x53, 0x26, 0xFE, 0x09,
        0x46, 0x2D, 0xFD, 0x09, 0x2D, 0xFC, 0x09, 0x2D, 0xFB, 0x09, // formals, reversed
        0x4C, 0x4F, 0x11, 0x53, //
        0x26, 0xE4, 0x09, 0xBD, 0x86, 0x45, 0x40, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, // call head
        0xB9, 0xFD, 0x09, 0x86, 0x45, 0x40, // LOAD b, float
        0x55, 0x86, 0x45, 0x40, 0x4C, // the callee's formal, end of args
        0x41, 0x86, 0x45, 0x40, // result: float
        0x3A, 0xFF, 0x09, 0x54, 0x02, 0x29, 0xFF, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `float n2(double a, double b) { return g1f(b); }` — the narrowing, whose
    /// whole emission is the single word `fc201018`, `frsp f1,f2`. The `2C` here
    /// is byte-identical in shape to the *free* widening `w2` carries; only the
    /// two types' widths tell them apart.
    const FP_TAIL_NARROW: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x49, 0x53, 0x53, 0x26, 0x1E, 0x0A,
        0x46, 0x2D, 0x1D, 0x0A, 0x2D, 0x1C, 0x0A, //
        0x4C, 0x4F, 0x11, 0x53, //
        0x26, 0xE4, 0x09, 0xBD, 0x86, 0x45, 0x40, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, //
        0xB9, 0x1D, 0x0A, 0x88, 0x85, 0x41, // LOAD b, double
        0x2C, 0x86, 0x45, 0x40, 0x00, // …converted to float: a real `frsp`
        0x55, 0x86, 0x45, 0x40, 0x4C, 0x41, 0x86, 0x45, 0x40, //
        0x3A, 0x1F, 0x0A, 0x54, 0x02, 0x29, 0x1F, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `void b2(float a, float b) { gvf(b); }` — the discarded statement call,
    /// `4C 4B` and no result annotation at all. A single statement call with
    /// nothing after it IS a tail call; emitting a frame for it would be a
    /// mis-emit, not a gap.
    const FP_TAIL_VOID: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x4C, 0x53, 0x53, 0x26, 0x26, 0x0A,
        0x46, 0x2D, 0x25, 0x0A, 0x2D, 0x24, 0x0A, //
        0x4C, 0x4F, 0x11, 0x53, //
        0x26, 0xEA, 0x09, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x05, 0x10, 0x00, 0x00, // void ret
        0xB9, 0x25, 0x0A, 0x86, 0x45, 0x40, 0x55, 0x86, 0x45, 0x40, 0x4C, //
        0x4B, // the result is DISCARDED
        0x3A, 0x27, 0x0A, 0x54, 0x02, 0x29, 0x27, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `float C::m(float a, float b) const { return g1f(b); }` — a member
    /// function, whose pre-body region binds `this`. It emits the byte-identical
    /// `fmr f1,f2` its free twin does: `this` takes r3 and displaces **nothing**
    /// in the FP file, so the FP parameter list must not count it.
    const FP_TAIL_MEMBER: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x4F, 0x53, 0x53, 0x26, 0x2B, 0x0A,
        0xB9, 0x35, 0x0A, 0xA6, 0x43, 0x88, 0x20, 0x99, 0x86, 0x43, 0x8A, 0x20, 0x00, // `this`
        0x46, 0x2D, 0x33, 0x0A, 0x2D, 0x32, 0x0A, //
        0x4C, 0x4F, 0x11, 0x53, //
        0x26, 0xE4, 0x09, 0xBD, 0x86, 0x45, 0x40, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, //
        0xB9, 0x33, 0x0A, 0x86, 0x45, 0x40, 0x55, 0x86, 0x45, 0x40, 0x4C, 0x41, 0x86, 0x45, 0x40,
        0x3A, 0x36, 0x0A, 0x54, 0x02, 0x29, 0x36, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// The FP file is numbered over the FP parameters ALONE, and the shape says
    /// so by carrying only them: `params[n]` is `f(n+1)`.
    #[test]
    fn a_non_fp_formal_between_the_fp_ones_does_not_move_the_fp_file() {
        const F: [SyFormal; 3] = [
            SyFormal { tok: 0xFB09, size: 4, kind: FLOAT },
            SyFormal { tok: 0xFC09, size: 4, kind: INT },
            SyFormal { tok: 0xFD09, size: 4, kind: FLOAT },
        ];
        assert_eq!(
            parse_segment(FP_TAIL_GAP, sy(&F)),
            Some(BodyShape::FpTailCall {
                params: vec![0xFB09, 0xFD09],
                arg_tok: 0xFD09,
                narrowing: false,
                callee_tok: 0xE409,
            })
        );
        // …and with the middle formal declared `float` instead, the SAME `.ex`
        // bytes name f3. That is the whole content of the rule: the `.ex` stream
        // cannot distinguish the two, and only `.sy` can.
        const G: [SyFormal; 3] = [
            SyFormal { tok: 0xFB09, size: 4, kind: FLOAT },
            SyFormal { tok: 0xFC09, size: 4, kind: FLOAT },
            SyFormal { tok: 0xFD09, size: 4, kind: FLOAT },
        ];
        assert_eq!(
            parse_segment(FP_TAIL_GAP, sy(&G)),
            Some(BodyShape::FpTailCall {
                params: vec![0xFB09, 0xFC09, 0xFD09],
                arg_tok: 0xFD09,
                narrowing: false,
                callee_tok: 0xE409,
            })
        );
    }

    /// The narrowing conversion is carried on the shape, not inferred from the
    /// types later: `double`→`float` at the boundary is an `frsp` and its twin
    /// `float`→`double` is nothing at all.
    #[test]
    fn the_boundary_conversion_is_recorded_in_the_direction_that_costs_an_instruction() {
        const F: [SyFormal; 2] = [
            SyFormal { tok: 0x1C0A, size: 8, kind: DOUBLE },
            SyFormal { tok: 0x1D0A, size: 8, kind: DOUBLE },
        ];
        assert_eq!(
            parse_segment(FP_TAIL_NARROW, sy(&F)),
            Some(BodyShape::FpTailCall {
                params: vec![0x1C0A, 0x1D0A],
                arg_tok: 0x1D0A,
                narrowing: true,
                callee_tok: 0xE409,
            })
        );
        // The `.sy` side must agree with the `.ex` side about the source width.
        // Declaring the same formals `float` while `.ex` LOADs them as `double`
        // is a misread record, never a width to guess at.
        const MISMATCHED: [SyFormal; 2] = [
            SyFormal { tok: 0x1C0A, size: 4, kind: FLOAT },
            SyFormal { tok: 0x1D0A, size: 4, kind: FLOAT },
        ];
        assert_eq!(parse_segment(FP_TAIL_NARROW, sy(&MISMATCHED)), None);
    }

    /// A discarded statement call is a tail call, and a member function's `this`
    /// is outside the FP file entirely.
    #[test]
    fn the_discarded_call_and_the_member_receiver_are_the_same_shape() {
        const F: [SyFormal; 2] = [
            SyFormal { tok: 0x240A, size: 4, kind: FLOAT },
            SyFormal { tok: 0x250A, size: 4, kind: FLOAT },
        ];
        assert_eq!(
            parse_segment(FP_TAIL_VOID, sy(&F)),
            Some(BodyShape::FpTailCall {
                params: vec![0x240A, 0x250A],
                arg_tok: 0x250A,
                narrowing: false,
                callee_tok: 0xEA09,
            })
        );
        const M: [SyFormal; 2] = [
            SyFormal { tok: 0x320A, size: 4, kind: FLOAT },
            SyFormal { tok: 0x330A, size: 4, kind: FLOAT },
        ];
        // `this` (token 0x350A) is neither in `params` nor counted by it.
        assert_eq!(
            parse_segment(FP_TAIL_MEMBER, sy(&M)),
            Some(BodyShape::FpTailCall {
                params: vec![0x320A, 0x330A],
                arg_tok: 0x330A,
                narrowing: false,
                callee_tok: 0xE409,
            })
        );
    }

    /// Without a `.sy` binding the FP file is not knowable at all — `float` and
    /// `int` are both four bytes and the `.ex` formals region carries tokens and
    /// no types — so the production declines and the body reports its ordinary
    /// blocker instead. Refusing rather than assuming is the whole reason
    /// `arg_classes` is all-or-nothing.
    #[test]
    fn an_undetermined_sy_declines_and_leaves_the_census_key_alone() {
        let unknown = SyView { locals: &[], ptr_locals: &[], addr_locals: &[], formals: Formals::Undetermined };
        assert_eq!(parse_segment(FP_TAIL_GAP, unknown), None);
        let b = parse_segment_detail(FP_TAIL_GAP, unknown).unwrap_err();
        // `:mid`: the widths are withheld *before* the body is parsed at all, at
        // the `LO` marker, so the whole body is still ahead of the cursor.
        assert!(b.off < b.seg_len, "the refusal is at the LO marker, with the body still ahead");
        assert_eq!(b.feature(), "param-width-undetermined:mid");
    }

    /// A `volatile` FP formal is a memory object: c2 homes it in a frame and
    /// reads it back, so the whole body is framed rather than one `fmr` and a
    /// branch. W32, and it was live — `readers::is_volatile_tag`. Retyping the
    /// LOAD alone, `86 45 40` → `96 45 40`, must flip acceptance.
    #[test]
    fn a_volatile_fp_formal_refuses_because_its_load_is_a_spill() {
        const F: [SyFormal; 3] = [
            SyFormal { tok: 0xFB09, size: 4, kind: FLOAT },
            SyFormal { tok: 0xFC09, size: 4, kind: INT },
            SyFormal { tok: 0xFD09, size: 4, kind: FLOAT },
        ];
        let mut seg = FP_TAIL_GAP.to_vec();
        let load = seg
            .windows(6)
            .position(|w| w == [0xB9, 0xFD, 0x09, 0x86, 0x45, 0x40])
            .expect("the argument LOAD");
        seg[load + 3] = 0x96;
        assert_eq!(parse_segment(&seg, sy(&F)), None);
        // …while the `const` spelling, one bit away in the same byte, is free.
        seg[load + 3] = 0xA6;
        assert!(matches!(
            parse_segment(&seg, sy(&F)),
            Some(BodyShape::FpTailCall { .. })
        ));
    }

    // ---- W34: the multi-argument FP tail call --------------------------------

    /// `float sw2(float a, int k, float b) { return g2f(b, a); }` — whole captured
    /// segment, the swap with a non-FP formal wedged between the two FP ones.
    ///
    /// **The discriminator for both numberings at once.** The *sources* are the
    /// FP file over the formals (a → f1, b → f2 — `k` occupies no FP register),
    /// and the *destinations* are the FP file over the ARGUMENTS (slot 1 → f1,
    /// slot 2 → f2). A model that used formal positions anywhere names f3 and
    /// emits a permutation of three registers where c2 emits one of two.
    ///
    /// Arguments appear in **reverse source order**, so the stream is `a` then
    /// `b` for the call `g2f(b, a)` — the same convention
    /// [`super::calls::parse_call_shape`] anchors on `parse_formals`' reversal,
    /// and reading it the other way round turns every swap into an identity.
    const FP_MULTI_SWAP: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x02, 0x53, 0x53, 0x26, 0xE9, 0x09,
        0x46, 0x2D, 0xE8, 0x09, 0x2D, 0xE7, 0x09, 0x2D, 0xE6, 0x09, // formals, reversed: a k b
        0x4C, 0x4F, 0x11, 0x53, //
        0x26, 0xE5, 0x09, 0xBD, 0x86, 0x45, 0x40, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, // call head
        0xB9, 0xE6, 0x09, 0x86, 0x45, 0x40, 0x55, 0x86, 0x45, 0x40, // arg 2 of 2: `a`
        0xB9, 0xE8, 0x09, 0x86, 0x45, 0x40, 0x55, 0x86, 0x45, 0x40, // arg 1 of 2: `b`
        0x4C, 0x41, 0x86, 0x45, 0x40, //
        0x3A, 0xEA, 0x09, 0x54, 0x02, 0x29, 0xEA, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `float rt3(float a, float b, float c) { return g3f(b, c, a); }` — whole
    /// captured segment. The 3-cycle, which c2 breaks through **f0**.
    const FP_MULTI_ROT3: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x02, 0x53, 0x53, 0x26, 0xEA, 0x09,
        0x46, 0x2D, 0xE9, 0x09, 0x2D, 0xE8, 0x09, 0x2D, 0xE7, 0x09, //
        0x4C, 0x4F, 0x11, 0x53, //
        0x26, 0xE6, 0x09, 0xBD, 0x86, 0x45, 0x40, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, //
        0xB9, 0xE7, 0x09, 0x86, 0x45, 0x40, 0x55, 0x86, 0x45, 0x40, // `a`, the LAST argument
        0xB9, 0xE9, 0x09, 0x86, 0x45, 0x40, 0x55, 0x86, 0x45, 0x40, // `c`
        0xB9, 0xE8, 0x09, 0x86, 0x45, 0x40, 0x55, 0x86, 0x45, 0x40, // `b`, the FIRST
        0x4C, 0x41, 0x86, 0x45, 0x40, //
        0x3A, 0xEB, 0x09, 0x54, 0x02, 0x29, 0xEB, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// `float C::m(float a, float b) const { return g2f(a, b); }` — whole captured
    /// segment. A member function forwarding both arguments unchanged.
    ///
    /// **The case the integer multi-argument rung cannot reach at all.** Its
    /// `arg_sources` indexes the formals list with `this` at index 0, so every
    /// member function with two or more arguments trips `call-arg-outer-formal`.
    /// `this` (token 0xF309) takes r3 and is outside the FP file entirely, so
    /// indexing that file instead makes the shape free — and the identity
    /// permutation emits **nothing**, a bare `b ?g2f`.
    const FP_MULTI_MEMBER_ID: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x03, 0x53, 0x53, 0x26, 0xE9, 0x09,
        0xB9, 0xF3, 0x09, 0xA6, 0x43, 0x82, 0x20, 0x99, 0x86, 0x43, 0x84, 0x20, 0x00, // `this`
        0x46, 0x2D, 0xF1, 0x09, 0x2D, 0xF0, 0x09, //
        0x4C, 0x4F, 0x11, 0x53, //
        0x26, 0xE5, 0x09, 0xBD, 0x86, 0x45, 0x40, 0x00, 0x80, 0x07, 0x10, 0x00, 0x00, //
        0xB9, 0xF1, 0x09, 0x86, 0x45, 0x40, 0x55, 0x86, 0x45, 0x40, //
        0xB9, 0xF0, 0x09, 0x86, 0x45, 0x40, 0x55, 0x86, 0x45, 0x40, //
        0x4C, 0x41, 0x86, 0x45, 0x40, //
        0x3A, 0xF4, 0x09, 0x54, 0x02, 0x29, 0xF4, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// Both numberings run over their own file, and a non-FP formal moves
    /// neither. Declaring the middle formal `float` instead re-numbers the
    /// SOURCES and nothing else — the same `.ex` bytes, a different permutation.
    #[test]
    fn the_destinations_are_numbered_over_the_fp_arguments_and_the_sources_over_the_fp_formals() {
        const F: [SyFormal; 3] = [
            SyFormal { tok: 0xE609, size: 4, kind: FLOAT },
            SyFormal { tok: 0xE709, size: 4, kind: INT },
            SyFormal { tok: 0xE809, size: 4, kind: FLOAT },
        ];
        assert_eq!(
            parse_segment(FP_MULTI_SWAP, sy(&F)),
            Some(BodyShape::FpMultiArgTailCall {
                params: vec![0xE609, 0xE809],
                arg_sources: vec![1, 0],
                callee_tok: 0xE509,
            })
        );
        // With the middle formal floating-point the FP file has THREE entries and
        // `b` is f3, so the same call is a permutation out of a register the
        // argument list does not otherwise write — a shift, which c2 lowers as
        // two independent moves and this rung refuses.
        const G: [SyFormal; 3] = [
            SyFormal { tok: 0xE609, size: 4, kind: FLOAT },
            SyFormal { tok: 0xE709, size: 4, kind: FLOAT },
            SyFormal { tok: 0xE809, size: 4, kind: FLOAT },
        ];
        assert_eq!(parse_segment(FP_MULTI_SWAP, sy(&G)), None);
    }

    /// The 3-cycle, and the argument stream really is reversed: read the other
    /// way round `g3f(b,c,a)` would come out as `arg_sources = [2, 0, 1]`, which
    /// is the *other* 3-cycle and four different instructions.
    #[test]
    fn the_argument_stream_is_reverse_source_order() {
        const F: [SyFormal; 3] = [
            SyFormal { tok: 0xE709, size: 4, kind: FLOAT },
            SyFormal { tok: 0xE809, size: 4, kind: FLOAT },
            SyFormal { tok: 0xE909, size: 4, kind: FLOAT },
        ];
        assert_eq!(
            parse_segment(FP_MULTI_ROT3, sy(&F)),
            Some(BodyShape::FpMultiArgTailCall {
                params: vec![0xE709, 0xE809, 0xE909],
                arg_sources: vec![1, 2, 0],
                callee_tok: 0xE609,
            })
        );
    }

    /// A member function's `this` is outside the FP file, so a two-argument
    /// forwarding call is in class here where the integer twin refuses it.
    #[test]
    fn a_member_receivers_this_does_not_enter_the_fp_argument_file() {
        const F: [SyFormal; 2] = [
            SyFormal { tok: 0xF009, size: 4, kind: FLOAT },
            SyFormal { tok: 0xF109, size: 4, kind: FLOAT },
        ];
        assert_eq!(
            parse_segment(FP_MULTI_MEMBER_ID, sy(&F)),
            Some(BodyShape::FpMultiArgTailCall {
                params: vec![0xF009, 0xF109],
                arg_sources: vec![0, 1],
                callee_tok: 0xE509,
            })
        );
    }

    /// A **GPR argument alongside the FP one refuses**, and this is the gate the
    /// whole rung rests on: c2 interleaves the two files' move sequences on a
    /// schedule no per-file solver reproduces. Retyping one formal `int` — which
    /// is what makes its argument a GPR one — must flip acceptance, and it does
    /// so through `arg_classes` alone: the `.ex` bytes are identical.
    #[test]
    fn a_gpr_argument_beside_an_fp_one_refuses_because_the_two_files_interleave() {
        const MIXED: [SyFormal; 3] = [
            SyFormal { tok: 0xE609, size: 4, kind: INT },
            SyFormal { tok: 0xE709, size: 4, kind: INT },
            SyFormal { tok: 0xE809, size: 4, kind: FLOAT },
        ];
        assert_eq!(parse_segment(FP_MULTI_SWAP, sy(&MIXED)), None);
    }

    /// The gate is the number of **local minima**, which is the number of
    /// scratch registers c2 parks into — not the cycle length. Measured over the
    /// complete n = 2…5 grid, `scripts/gt_fpperm.py`.
    #[test]
    fn the_permutation_gate_counts_local_minima_and_not_cycle_length() {
        // Fixed points and the identity: no scratch at all.
        assert_eq!(fp_perm_local_minima(&[0, 1, 2]), 0);
        // Every cycle of length <= 3 is unimodal, which is why the published
        // "one temp breaks the cycle" rule survived to three and no further.
        assert_eq!(fp_perm_local_minima(&[1, 0]), 1);
        assert_eq!(fp_perm_local_minima(&[1, 2, 0]), 1);
        assert_eq!(fp_perm_local_minima(&[2, 0, 1]), 1);
        // A 4- and a 5-cycle that ascend after their minimum stay at one scratch
        // and are in class — the length is not the boundary.
        assert_eq!(fp_perm_local_minima(&[1, 2, 3, 0]), 1);
        assert_eq!(fp_perm_local_minima(&[1, 2, 3, 4, 0]), 1);
        // `g4f(b,a,d,c)` — two disjoint 2-cycles, one minimum each.
        assert_eq!(fp_perm_local_minima(&[1, 0, 3, 2]), 2);
        // `g4f(c,d,b,a)` — ONE 4-cycle whose sequence descends then ascends, so
        // it needs two scratches even though a 4-cycle above needs one.
        assert_eq!(fp_perm_local_minima(&[2, 3, 1, 0]), 2);
        // A source outside the destination range never closes a cycle; the walk
        // reports "unmodelable" rather than indexing out of bounds.
        assert_eq!(fp_perm_local_minima(&[1, 2]), usize::MAX);
    }

    /// `void mx(int k, float a, float b) { gviff(k, b, a); }` — whole captured
    /// segment. A **GPR argument beside the FP ones**, in the position where it
    /// does not move: `k` is formal 0 so it sits in r3, and it is the call's
    /// first argument so the callee wants it in r3 too. Nothing is emitted for
    /// it, and the FP half is the byte-identical pure-FP swap
    /// (`fmr f0,f2 ; fmr f2,f1 ; fmr f1,f0`).
    ///
    /// This is the shape that makes the rung's gate "no GPR argument moves"
    /// rather than "every argument is floating-point" — a marshalling with no
    /// moves in the other file has nothing for the FP moves to interleave with.
    const FP_MULTI_MIXED: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0xA0, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x02, 0x53, 0x53, 0x26, 0xEA, 0x09,
        0x46, 0x2D, 0xE9, 0x09, 0x2D, 0xE8, 0x09, 0x2D, 0xE7, 0x09, // formals, reversed: k a b
        0x4C, 0x4F, 0x11, 0x53, //
        0x26, 0xE6, 0x09, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x01, 0x10, 0x00, 0x00, // void ret
        0xB9, 0xE8, 0x09, 0x86, 0x45, 0x40, 0x55, 0x86, 0x45, 0x40, // arg 3: `a`, float
        0xB9, 0xE9, 0x09, 0x86, 0x45, 0x40, 0x55, 0x86, 0x45, 0x40, // arg 2: `b`, float
        0xB9, 0xE7, 0x09, 0x86, 0x41, 0x74, 0x55, 0x86, 0x41, 0x74, // arg 1: `k`, INT
        0x4C, 0x4B, // the result is discarded
        0x3A, 0xEB, 0x09, 0x54, 0x02, 0x29, 0xEB, 0x09, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// The gate is **no GPR argument moves**, not "every argument is FP". `k`
    /// occupies argument slot 1 in the caller's numbering and in the call's, so
    /// it costs nothing and the body is in class with the FP file's own swap.
    #[test]
    fn a_gpr_argument_that_does_not_move_is_free_and_the_fp_half_is_unchanged() {
        const F: [SyFormal; 3] = [
            SyFormal { tok: 0xE709, size: 4, kind: INT },
            SyFormal { tok: 0xE809, size: 4, kind: FLOAT },
            SyFormal { tok: 0xE909, size: 4, kind: FLOAT },
        ];
        assert_eq!(
            parse_segment(FP_MULTI_MIXED, sy(&F)),
            Some(BodyShape::FpMultiArgTailCall {
                params: vec![0xE809, 0xE909],
                arg_sources: vec![1, 0],
                callee_tok: 0xE609,
            })
        );
        // …and the moment a GPR argument would have to MOVE, the body declines.
        // Declaring the *second* formal `int` as well leaves `b` the only FP
        // argument, so `a` becomes the call's third slot — r5 — while the
        // caller has it in r4. One `mr`, and how that move schedules against the
        // FP one is the open question this rung refuses.
        const G: [SyFormal; 3] = [
            SyFormal { tok: 0xE709, size: 4, kind: INT },
            SyFormal { tok: 0xE809, size: 4, kind: INT },
            SyFormal { tok: 0xE909, size: 4, kind: FLOAT },
        ];
        assert_eq!(parse_segment(FP_MULTI_MIXED, sy(&G)), None);
    }
}
