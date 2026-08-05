//! The comparison leaf: relation × signedness × literal.
//! See `docs/CODEGEN_W6_COMPARE.md`.

use crate::func::body::expr::{eat_return_plumbing, BODY_SCOPE_DEPTH};
use crate::func::body::BodyShape;
use crate::func::readers::{eat, eat_byte, read_token_var, read_varint, INT_TYPE, UINT_TYPE};
use crate::func::{CompareLeaf, Rel};

use super::params::parse_params;

/// Try to parse a **W6 comparison leaf** body: `return <formal> <rel> <k>;`.
///
/// ```text
///   B9 <tok> <T>        LOAD the formal          T ∈ {int, unsigned}
///   33 <T> <varint>     LITERAL k, same type T
///   <rel>               1F|20|21|22|23|24
///   2C <R> 00           convert bool → R         R ∈ {int, unsigned}
///   41 <R>              result type
///   <return plumbing>
/// ```
///
/// Fail-closed specifics that are load-bearing rather than incidental:
///
/// * The two operand types must be **equal**. c1xx always inserts a conversion
///   first, so a mismatch has never been observed; rejecting it is a cheap
///   assertion, not a dropped feature.
/// * The `2C` convert is accepted **only here**, directly over a comparison
///   result. The identical token over a narrow-integer LOAD is a real
///   `extsb`/`extsh` sign-extension, so a blanket "`2C` is free" rule would
///   silently drop those instructions.
/// * The parse must reach the segment end via the shared return plumbing, so a
///   trailing statement, a second comparison, or an arithmetic post-op (e.g.
///   `return (a > 7) + 1;`, which retargets the spine's last instruction) all
///   reject the whole function.
///
/// Returns `None` — leaving the caller's cursor untouched — for anything that is
/// not exactly this shape.
pub(crate) fn try_parse_compare(seg: &[u8], start: usize, lo: usize) -> Option<BodyShape> {
    let mut p = start;

    // LOAD <formal> <T>
    if !eat_byte(seg, &mut p, 0xB9) {
        return None;
    }
    let (param, w) = read_token_var(seg, p)?;
    p += w;
    let signed = if eat(seg, &mut p, &INT_TYPE) {
        true
    } else if eat(seg, &mut p, &UINT_TYPE) {
        false
    } else {
        return None;
    };
    let operand_type = if signed { INT_TYPE } else { UINT_TYPE };

    // LITERAL k, of the SAME type as the loaded operand.
    if !eat_byte(seg, &mut p, 0x33) || !eat(seg, &mut p, &operand_type) {
        return None;
    }
    let k = read_varint(seg, &mut p)?;

    // The relational opcode.
    let rel = Rel::from_opcode(*seg.get(p)?)?;
    p += 1;

    // `2C <R> 00` — convert the bool result to the return type.
    if !eat_byte(seg, &mut p, 0x2C) {
        return None;
    }
    let ret_is_int = if eat(seg, &mut p, &INT_TYPE) {
        true
    } else if eat(seg, &mut p, &UINT_TYPE) {
        false
    } else {
        return None;
    };
    if !eat_byte(seg, &mut p, 0x00) {
        return None;
    }

    // Result type + the shared return plumbing, which must reach the segment end.
    let ret_type = if ret_is_int { INT_TYPE } else { UINT_TYPE };
    if !eat_byte(seg, &mut p, 0x41) || !eat(seg, &mut p, &ret_type) {
        return None;
    }
    // Result type already consumed above, so `has_result_type` is false here.
    eat_return_plumbing(seg, &mut p, false, BODY_SCOPE_DEPTH).ok()?;

    // The compared value must be the function's FIRST formal: the spine reads it
    // from r3, and nothing here models a register move.
    let params = parse_params(seg, lo).ok()?;
    if params.first() != Some(&param) || params.len() != 1 {
        return None;
    }

    // Gates moved here from `compare_leaf_text`, so the census counts only what the
    // emitter can emit — through [`CompareLeaf::out_of_class_ctx`], which is the
    // one locator both sides share. Two of these three clauses used to be spelled
    // out again right here, and the third (a large UNSIGNED literal under
    // `==`/`!=`) was in codegen only, so `int f(unsigned a){ return a ==
    // 4294967295u; }` censused in class and the port refused it.
    let cmp = CompareLeaf { param, rel, signed, k };
    if cmp.out_of_class_ctx().is_some() {
        return None;
    }
    Some(BodyShape::Compare(cmp))
}

/// **W43** — `return ((unsigned)(P != 0) << SH) | C;`.
///
/// ```text
///   B9 <tok> <T>          LOAD the formal            T ∈ {int, unsigned}
///   33 <T> 00             LITERAL 0, same type
///   20                    cmp-ne
///   2C <U> 00             convert bool -> U          the shift's operand type
///   33 <int-like> SH      the shift count
///   09                    shl
///   33 <int-like> C       the constant (usually the `80` wide escape)
///   0C                    bit-or
///   [2C <R> 00]           the return width conversion, optional
///   41 <R>                result type
///   <return plumbing>
/// ```
///
/// Tried **after** [`try_parse_compare`], which cannot take this body: it
/// requires `2C <R> 00` immediately followed by `41`, and here a `33` follows
/// the convert. Non-committal, like every other recognizer in this module.
///
/// The class gates, each with its measurement:
///
/// * **`!=` against 0 only.** That is `CompareLeaf`'s `(Rel::Ne, _)` fold —
///   `addic`+`subfe`, two words, identical signed and unsigned. Every other
///   relation is a different spine of a different length, and the fold below
///   assumes the compared value ends in one register with r11 dead.
/// * **One formal, and it is the compared one.** The spine reads r3. With the
///   formal in any other slot c2 puts the constant in **r3** and drops the `mr`
///   entirely — 5 measured cells, all in `w43_cmp_shift_or_neg.cpp`.
/// * **[`crate::func::shift_or_rlwimi`]** decides `SH` and `C`; its doc carries
///   the 288-cell grid and says plainly which region it declines to claim.
pub(crate) fn try_parse_cmp_shift_or(seg: &[u8], start: usize, lo: usize) -> Option<BodyShape> {
    let mut p = start;

    if !eat_byte(seg, &mut p, 0xB9) {
        return None;
    }
    let (param, w) = read_token_var(seg, p)?;
    p += w;
    let signed = if eat(seg, &mut p, &INT_TYPE) {
        true
    } else if eat(seg, &mut p, &UINT_TYPE) {
        false
    } else {
        return None;
    };
    let operand_type = if signed { INT_TYPE } else { UINT_TYPE };

    // `33 <T> 0` then `20` — `!= 0`, and nothing else.
    if !eat_byte(seg, &mut p, 0x33) || !eat(seg, &mut p, &operand_type) {
        return None;
    }
    if read_varint(seg, &mut p)? != 0 {
        return None;
    }
    if Rel::from_opcode(*seg.get(p)?)? != Rel::Ne {
        return None;
    }
    p += 1;

    // `2C <U> 00` — the bool widened to the shift's operand type.
    if !eat_byte(seg, &mut p, 0x2C) {
        return None;
    }
    if !crate::func::readers::eat_int_like(seg, &mut p) || !eat_byte(seg, &mut p, 0x00) {
        return None;
    }

    // `33 <T> SH` `09` — the shift.
    if !eat_byte(seg, &mut p, 0x33) || !crate::func::readers::eat_int_like(seg, &mut p) {
        return None;
    }
    let sh = read_varint(seg, &mut p)?;
    if !eat_byte(seg, &mut p, 0x09) {
        return None;
    }

    // `33 <T> C` `0C` — the constant and the OR.
    if !eat_byte(seg, &mut p, 0x33) || !crate::func::readers::eat_int_like(seg, &mut p) {
        return None;
    }
    let c = read_varint(seg, &mut p)?;
    if !eat_byte(seg, &mut p, 0x0C) {
        return None;
    }

    // The return width conversion is optional — `?GetXAllocAttributes` returns
    // `unsigned long` from an `unsigned int` expression and gets one; a body
    // whose types already agree does not.
    if eat_byte(seg, &mut p, 0x2C) {
        if !crate::func::readers::eat_int_like(seg, &mut p) || !eat_byte(seg, &mut p, 0x00) {
            return None;
        }
    }
    if !eat_byte(seg, &mut p, 0x41) || !crate::func::readers::eat_int_like(seg, &mut p) {
        return None;
    }
    eat_return_plumbing(seg, &mut p, false, BODY_SCOPE_DEPTH).ok()?;

    let params = parse_params(seg, lo).ok()?;
    if params.first() != Some(&param) || params.len() != 1 {
        return None;
    }

    let sh = u8::try_from(sh).ok()?;
    let c = u32::try_from(c).ok()?;
    // The one locator both the census and the emitter run.
    crate::func::shift_or_rlwimi(sh, c)?;
    Some(BodyShape::CmpShiftOr(crate::func::CmpShiftOr { param, signed, sh, c }))
}
