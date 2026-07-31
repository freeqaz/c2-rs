//! **The single-argument floating-point tail call** — `return g(x);` and
//! `g(x);` where `x` is an FP formal, the whole body, no frame.
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
use crate::func::sy::{ArgClass, SyView};

use super::calls::eat_call_head;
use super::this_binding::parse_this_token;

/// The most FP parameters a body in this class may declare. Past `f13` a
/// floating-point formal is stack-homed and reading it is an `lfs`/`lfd` off r1,
/// not a register move — the same boundary the W13 leaf and the FP store draw,
/// and for the same reason (`docs/CODEGEN_FP_ARGS.md` §5: the 14-parameter
/// capture frames and spills, so it is not a leaf at all).
const MAX_FP_FORMALS: usize = 13;

/// Try to parse a **single-argument floating-point tail call**, positioned at the
/// `26 <callee>` symbol push that opens the body.
///
/// ```text
///   26 <callee> BD <ret TYPE> 00 <fn-type-id>     the shared call head
///   B9 <tok> <FP TYPE>                            the argument: one bare formal LOAD
///   [ 2C <FP TYPE> 00 ]                           …optionally converted, FP→FP
///   55 <FP TYPE>                                  the callee's formal type
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
    let callee_tok = eat_call_head(seg, &mut p).ok()?;

    // ---- the argument region ------------------------------------------------
    // Exactly ONE argument, and it is a bare LOAD. A second `B9` before the `4C`
    // is the multi-argument case: two register files whose move sequences
    // interleave on a schedule no per-file solver reproduces
    // (`docs/CODEGEN_FP_ARGS.md` §1.1), refused here by declining.
    if !eat_byte(seg, &mut p, 0xB9) {
        return None;
    }
    let (arg_tok, w) = read_token_var(seg, p)?;
    p += w;
    let src_double = eat_fp_type(seg, &mut p)?;
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
    if !eat_byte(seg, &mut p, 0x4C) {
        return None;
    }

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
    parse_this_token(seg, lo)?;
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
    // The argument must be one of the FP formals — not a local, not a global, not
    // a formal in the *other* file (which would be an `int`→`float` conversion,
    // and that is a frame). Its index here is its FP register number minus one.
    params.iter().position(|&t| t == arg_tok)?;
    // `.ex` says this value is a `float`/`double`; `.sy` says the formal is one.
    // They are two channels on one fact and a disagreement is a misread record,
    // never a width to guess at — the same all-or-nothing discipline `arg_classes`
    // itself applies.
    match classes[formals.iter().position(|&t| t == arg_tok)?] {
        ArgClass::Fp { double } if double == src_double => {}
        _ => return None,
    }
    Some(BodyShape::FpTailCall {
        params,
        arg_tok,
        // `double`→`float` at the boundary is a real `frsp`, and it is **fused**
        // with the register move rather than following one (`n2` above).
        narrowing: src_double && !want_double,
        callee_tok,
    })
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
        SyView { locals: &[], formals: Formals::Declared(formals) }
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
        let unknown = SyView { locals: &[], formals: Formals::Undetermined };
        assert_eq!(parse_segment(FP_TAIL_GAP, unknown), None);
        assert_eq!(
            parse_segment_detail(FP_TAIL_GAP, unknown).unwrap_err().feature(),
            "param-width-undetermined:eof"
        );
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
}
