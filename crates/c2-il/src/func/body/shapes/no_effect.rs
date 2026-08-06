//! **The dead-temporary call body** — a body whose whole content is one call
//! plus the materialization of a temporary nothing else ever reads.
//!
//! # What this is for, and what it is NOT
//!
//! This is **not** a body shape the port emits. Nothing here reaches
//! [`super::super::BodyShape`], `select_function` or the COFF writer, and
//! [`super::super::parse_segment`] is byte-for-byte unchanged by it: a body this
//! module recognizes is still **parse-refused**, still counts in
//! `fnbyte-refused`, and still refuses its whole TU in
//! [`crate::IlBundle::functions`].
//!
//! What it produces is one fact, for one consumer: **mechanism E's fixpoint**
//! (`c2_core::elide::TuEmptyCallees`). E fires when a tail call's callee
//! *reduces to nothing*, and until now the only way to establish that was
//! [`crate::IlFunction::empty_body`] — a body that **decodes empty**. Board
//! **#980**'s 370 workload functions are the case where the callee's body is not
//! empty and still emits nothing, and lane `w-seq` (board #966/#971) measured
//! that every one of them is blocked on this production.
//!
//! # The shape, and where each byte came from
//!
//! ```cpp
//!   struct true_tag {};                                   // an empty tag type
//!   template <class I> inline void aux(I, I, const true_tag&) {}
//!   template <class I, class T> inline void dr(I f, I l, T*) {
//!       aux(f, l, true_tag());                            // <- THIS BODY
//!   }
//! ```
//!
//! `true_tag()` is value-initialized into a stack temporary — which c1xx spells
//! as an **intrinsic `memset`** (selector 173, `docs/IL_INTRINSIC_CALL.md` §1) —
//! and the temporary's address is then passed by reference. The call's callee
//! `aux` has an empty body, so c2 emits nothing for the callee, nothing for the
//! temporary, and the whole function is one `4e800020`.
//!
//! This is STLport's `__destroy_range` (`stl/_construct.h:172`) with the names
//! shortened, and it is the callee of every one of board #980's 370
//! `??$_Destroy_Range@…` differs.
//!
//! ```text
//!   4C 4F 11 53                                the body marker and its scope
//!   26 <callee-tok>                            the callee symbol push
//!   BD <ret TYPE> 00 <varint fn-type-id>       the CALL token
//!     33 86 41 74 80 AD 00 00 00               the intrinsic selector, 173
//!     40 <TYPE>                                the intrinsic-call token
//!       33 86 41 74 <align> 55 86 41 74        the alignment hint
//!       33 86 41 74 <count> 55 86 41 74        the byte count
//!       33 86 41 74 <fill>  55 86 41 74        the fill byte
//!       9B <TYPE> <temp-tok> 2C <TYPE> <v>     the destination — a TEMPORARY
//!       55 <TYPE>
//!     4C                                       the intrinsic's apply
//!     9B <TYPE> <temp-tok> 2C <TYPE> <v> 44    THE SAME temporary, by reference
//!     55 <TYPE>
//!     ( B9 <formal-tok> <TYPE> 55 <TYPE> )*    the formals, as they are
//!     ( 33 <TYPE> <lit>    55 <TYPE> )*        literals, as they are
//!   4C 4B                                      apply, and discard the result
//!   <the return plumbing, to the segment end>
//! ```
//!
//! # Why this is sound, stated as the four things that make it so
//!
//! 1. **The walk is TOTAL over the segment.** It starts at the body marker and
//!    ends by requiring [`eat_return_plumbing`]'s fail-closed terminal, so every
//!    byte of the body is accounted for by a production above. "The temporary is
//!    read nowhere else" is therefore not a search — there *is* nowhere else.
//! 2. **The answer is conditional, and the condition is the callee.** This
//!    returns the callee's *token*, never a verdict. The caller resolves it and
//!    asks E's own fixpoint whether that callee reduces to nothing; a body whose
//!    callee does not is not admitted. So the rule is keyed on the **callee's
//!    decoded IL**, which is board **#950**'s standing requirement — the
//!    relocation observable reads "nothing happened" on a self-recursive body
//!    that is plainly not nothing, and nothing here reads a relocation.
//! 3. **The temporary is a `9B` temp bind, twice, with the same token.** The
//!    destination of the write and the argument passed are required to be
//!    literally the same token; a body that memsets one slot and passes another
//!    is refused rather than assumed equivalent.
//! 4. **A cycle is still never admitted.** This module says nothing about
//!    cycles; it hands a *link* to the least fixpoint, which admits a name only
//!    on a `false → true` transition and never seeds one. `elide.rs`'s
//!    `a_cycle_is_not_elided_and_terminates` is unchanged and still passes.
//!
//! # What it deliberately does NOT accept
//!
//! * **A real `memset`.** The production for `memset(p, 0, n)` on a *pointer*
//!   carries a `void*` result type and its destination is an operand stream, not
//!   a `9B` temp bind. c2 lowers that one to `b <memset>` with a REL24
//!   (`docs/IL_CAST_CONVERT.md` §1.3) — the opposite of nothing.
//! * **An argument that is anything else** — a computed expression, a data
//!   symbol's address, a nested call, a second temporary bound by a different
//!   token. The argument vocabulary is a closed list of three forms and anything
//!   outside it refuses the whole body.
//! * **A call whose result is used.** `4C` must be followed by `4B`, the
//!   discard.
//! * **A `float`/`double` result**, for [`CallRet::discarded`]'s reason: the TU
//!   acquires `_fltused` and the obj grows a symbol.

use super::calls::eat_call_head;
use super::super::expr::{
    eat_return_plumbing, eat_scopes, intrinsic_selector, parse_formals, BODY_SCOPE_DEPTH,
};
use crate::func::readers::{eat_byte, eat_opt_stmt_marker, read_token_var, read_type, read_varint, INT_TYPE};

/// The intrinsic selector id for `memset` (`docs/IL_INTRINSIC_CALL.md` §3).
/// Named here rather than spelled `173` at the site, because the *number* is
/// what the whole production hangs on.
const INTRINSIC_MEMSET: i32 = 173;

/// **The recognizer.** `Some(callee_token)` when this segment's body emits
/// nothing **provided** the callee that token names reduces to nothing.
///
/// `None` — always, for every other body — and the cursor is not shared with any
/// other parse, so this can never change what [`super::super::parse_segment`]
/// accepts or which census key a refusal reports.
pub(crate) fn no_effect_call(seg: &[u8]) -> Option<u32> {
    let lo = crate::func::body_start(seg)?;
    // The formals list, so an argument load can be required to name one. A
    // segment whose formals region does not parse is refused rather than read
    // with an empty list — an empty list would silently turn every load into a
    // non-formal and change *which* bodies are refused for *which* reason.
    let formals = parse_formals(seg, lo).ok()?;

    let mut p = crate::func::ops_start(seg, lo);
    if !eat_byte(seg, &mut p, 0x53) {
        return None;
    }
    let mut depth = BODY_SCOPE_DEPTH;
    eat_scopes(seg, &mut p, &mut depth).ok()?;
    eat_opt_stmt_marker(seg, &mut p);

    let (callee_tok, ret) = eat_call_head(seg, &mut p).ok()?;
    // A discarded `float`/`double` result drags `_fltused` into the TU. The same
    // gate every other discarded-call site applies, through the same predicate.
    ret.discarded(seg, p).ok()?;

    // ---- the argument region: a closed vocabulary of three forms ------------
    let mut temps = 0usize;
    loop {
        match *seg.get(p)? {
            0x4C => {
                p += 1;
                break;
            }
            0x9B => return None, // a temp bind that is not preceded by its memset
            0x33 => {
                // Either the intrinsic selector (the temporary) or a plain
                // literal push. `intrinsic_selector` requires the `40` to follow,
                // so the two cannot be confused.
                match intrinsic_selector(seg, p) {
                    Some(INTRINSIC_MEMSET) => {
                        eat_dead_temp_arg(seg, &mut p)?;
                        temps += 1;
                    }
                    Some(_) => return None,
                    None => eat_lit_push(seg, &mut p)?,
                }
            }
            0xB9 => eat_formal_push(seg, &mut p, &formals)?,
            _ => return None,
        }
    }
    // The whole point of the shape is the temporary; a body with none of them is
    // an ordinary void call and belongs to the shapes that already parse it.
    if temps == 0 {
        return None;
    }
    // The result is DISCARDED. Without this a value-consuming call would be read
    // as emitting nothing while its result is still wanted.
    if !eat_byte(seg, &mut p, 0x4B) {
        return None;
    }
    // The fail-closed terminal: this must reach the end of the segment, which is
    // what makes the walk total and the "read nowhere else" claim structural.
    eat_return_plumbing(seg, &mut p, false, depth).ok()?;
    Some(callee_tok)
}

/// `33 <TYPE> <varint> 55 <TYPE>` — a literal standing as a whole argument.
fn eat_lit_push(seg: &[u8], p: &mut usize) -> Option<()> {
    let mut q = *p;
    if !eat_byte(seg, &mut q, 0x33) {
        return None;
    }
    let (_, _, _, w) = read_type(seg, q)?;
    q += w;
    read_varint(seg, &mut q)?;
    eat_push(seg, &mut q)?;
    *p = q;
    Some(())
}

/// `B9 <tok> <TYPE> 55 <TYPE>` — one of this function's own formals, as it is.
fn eat_formal_push(seg: &[u8], p: &mut usize, formals: &[u32]) -> Option<()> {
    let mut q = *p;
    if !eat_byte(seg, &mut q, 0xB9) {
        return None;
    }
    let (tok, w) = read_token_var(seg, q)?;
    q += w;
    if !formals.contains(&tok) {
        return None;
    }
    let (_, _, _, w) = read_type(seg, q)?;
    q += w;
    eat_push(seg, &mut q)?;
    *p = q;
    Some(())
}

/// `55 <TYPE>` — the argument push that closes every argument form.
fn eat_push(seg: &[u8], p: &mut usize) -> Option<()> {
    let mut q = *p;
    if !eat_byte(seg, &mut q, 0x55) {
        return None;
    }
    let (_, _, _, w) = read_type(seg, q)?;
    *p = q + w;
    Some(())
}

/// `9B <TYPE> <tok> 2C <TYPE> <varint>` — a **temporary**, converted to the type
/// the use wants. Returns the temporary's token.
///
/// `0x9B` is the temp bind (`super::mcall_tail`'s `temp_bind` axis); `0x2C` is
/// the CONVERT, whose trailing varint is decoded only to find its end.
fn eat_temp_addr(seg: &[u8], p: &mut usize) -> Option<u32> {
    let mut q = *p;
    if !eat_byte(seg, &mut q, 0x9B) {
        return None;
    }
    let (_, _, _, w) = read_type(seg, q)?;
    q += w;
    let (tok, w) = read_token_var(seg, q)?;
    q += w;
    if !eat_byte(seg, &mut q, 0x2C) {
        return None;
    }
    let (_, _, _, w) = read_type(seg, q)?;
    q += w;
    read_varint(seg, &mut q)?;
    *p = q;
    Some(tok)
}

/// The whole temporary: the intrinsic `memset` that initializes it, and the same
/// temporary immediately passed by reference.
///
/// The two tokens must be **equal**. A body that writes one slot and passes
/// another is a body this module does not understand, and it refuses rather than
/// treating the pair as interchangeable.
fn eat_dead_temp_arg(seg: &[u8], p: &mut usize) -> Option<()> {
    let mut q = *p;
    // `33 <int> <173>` — the selector. Already validated by the caller through
    // `intrinsic_selector`; re-walked here so this function owns its own cursor.
    if !eat_byte(seg, &mut q, 0x33) {
        return None;
    }
    if seg.get(q..q + INT_TYPE.len())? != INT_TYPE {
        return None;
    }
    q += INT_TYPE.len();
    if read_varint(seg, &mut q)? != INTRINSIC_MEMSET {
        return None;
    }
    // `40 <TYPE>` — the intrinsic-call token. It carries no trailing field
    // (`docs/IL_INTRINSIC_CALL.md` §1.1).
    if !eat_byte(seg, &mut q, 0x40) {
        return None;
    }
    // **The result type is `int`, and that is a MEASURED discriminator.** On
    // `src/lazer/game/BustAMovePanel.cpp` the byte run `33 86 41 74 80 AD 00 00
    // 00` occurs **455** times and the byte after it splits three ways: **446**
    // `40 86 41 74` — the temporary-materialization form this module reads —
    // **7** `40 86 43 83 08`, a `void *` result, which is `memset(p, 0, n)` on a
    // real pointer and which c2 lowers to `b <memset>` with a REL24, and **2**
    // that are not followed by a `40` at all (an ordinary literal 173, not a
    // selector). Requiring the int form keeps the first two apart at the token
    // rather than downstream.
    if seg.get(q..q + INT_TYPE.len())? != INT_TYPE {
        return None;
    }
    q += INT_TYPE.len();
    // The three int literals: the alignment hint c1xx adds, the byte count and
    // the fill. Their *values* are not constrained — the write lands in a
    // temporary this body's own grammar proves nothing else reads — but each
    // must be a literal, so a computed count (a real `memset` over a range)
    // cannot reach here.
    for _ in 0..3 {
        eat_lit_push(seg, &mut q)?;
    }
    // The destination, and the apply.
    let dest = eat_temp_addr(seg, &mut q)?;
    eat_push(seg, &mut q)?;
    if !eat_byte(seg, &mut q, 0x4C) {
        return None;
    }
    // The same temporary, bound (`44`, payload-free — `super::control_flow`) and
    // pushed as the reference argument.
    let again = eat_temp_addr(seg, &mut q)?;
    if again != dest {
        return None;
    }
    if !eat_byte(seg, &mut q, 0x44) {
        return None;
    }
    eat_push(seg, &mut q)?;
    *p = q;
    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `??$dr@PAHH@@YAXPAH0PAH@Z` — the whole `.ex` segment, transcribed
    /// verbatim from a live capture of `work/w-inl0/explore/e01.cpp` at the
    /// workload's own flag axes (`/GR /O1 /Oi /EHsc`), not hand-assembled. It is
    /// STLport's `__destroy_range` with the names shortened; c2's whole `.text`
    /// COMDAT for it is `4e800020`.
    const DEAD_TEMP_CALL: &[u8] = &[
        0x4F, 0x1F, 0x80, 0x05, 0x00, 0x20, 0x00, 0x4F, 0x20, 0x80, 0xFE, 0x00, 0x4F, 0x33, 0x0D,
        0x66, 0x12, 0x1C, 0x30, 0x22, 0x10, 0x01, 0x44, 0x01, 0x0B, 0x0B, 0x03, 0x0F, 0x10, 0x18,
        0x01, 0x00, 0x0E, 0x6C, 0x12, 0x38, 0x1D, 0x42, 0x45, 0x0E, 0x06, 0x01, 0x01, 0x01, 0x0D,
        0x08, 0x00, 0x0F, 0x4F, 0x02, 0x20, 0x00, 0x4F, 0x01, 0x0E, 0x53, 0x53, 0x26, 0x09, 0x0A,
        0x46, 0x2D, 0x07, 0x0A, 0x2D, 0x06, 0x0A, 0x2D, 0x05, 0x0A, 0x4C, 0x4F, 0x11, 0x53, 0x26,
        0x11, 0x0A, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x0A, 0x10, 0x00, 0x00, 0x33, 0x86, 0x41,
        0x74, 0x80, 0xAD, 0x00, 0x00, 0x00, 0x40, 0x86, 0x41, 0x74, 0x33, 0x86, 0x41, 0x74, 0x01,
        0x55, 0x86, 0x41, 0x74, 0x33, 0x86, 0x41, 0x74, 0x01, 0x55, 0x86, 0x41, 0x74, 0x33, 0x86,
        0x41, 0x74, 0x00, 0x55, 0x86, 0x41, 0x74, 0x9B, 0x82, 0x16, 0x86, 0x20, 0x0B, 0x0A, 0x2C,
        0xA6, 0x43, 0x8D, 0x20, 0x00, 0x55, 0xA6, 0x43, 0x8D, 0x20, 0x4C, 0x9B, 0x82, 0x16, 0x86,
        0x20, 0x0B, 0x0A, 0x2C, 0xA6, 0x43, 0x8D, 0x20, 0x00, 0x44, 0x55, 0x86, 0x43, 0x8E, 0x20,
        0xB9, 0x06, 0x0A, 0x86, 0x43, 0xF4, 0x08, 0x55, 0x86, 0x43, 0xF4, 0x08, 0xB9, 0x05, 0x0A,
        0x86, 0x43, 0xF4, 0x08, 0x55, 0x86, 0x43, 0xF4, 0x08, 0x4C, 0x4B, 0x3A, 0x0A, 0x0A, 0x54,
        0x02, 0x29, 0x0A, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
    ];

    /// The token the `26` push carries — the callee
    /// (`??$aux@PAH@@YAXPAH0ABUtrue_tag@@@Z`).
    const DEAD_TEMP_CALLEE: u32 = 0x110A;

    /// Locate a byte run inside the pinned segment. Every mutation below edits a
    /// field it has *found*, never an index it has counted: an offset written by
    /// hand is a second transcription of the capture and can rot against it
    /// silently.
    fn at(pat: &[u8]) -> usize {
        crate::func::readers::find_subslice(DEAD_TEMP_CALL, pat)
            .unwrap_or_else(|| panic!("pattern {pat:02x?} is not in the pinned segment"))
    }

    #[test]
    fn the_dead_temporary_call_body_is_recognized_and_names_its_callee() {
        assert_eq!(no_effect_call(DEAD_TEMP_CALL), Some(DEAD_TEMP_CALLEE));
    }

    /// **The fact this module exists to keep true**: recognizing the body does
    /// **not** accept it. `parse_segment` still refuses, so the census, the gate
    /// and `IlBundle::functions` are all unchanged — board #971 condition 4.
    #[test]
    fn recognizing_the_body_does_not_accept_it() {
        use crate::func::test_fixtures::NO_LOCALS;
        assert!(crate::func::body::parse_segment(DEAD_TEMP_CALL, NO_LOCALS).is_none());
        let b = crate::func::body::parse_segment_detail(DEAD_TEMP_CALL, NO_LOCALS).unwrap_err();
        assert_eq!(b.feature(), "expr-intrinsic-memset");
    }

    /// **The temporary must be the SAME temporary.** Repoint the argument's temp
    /// bind at a different token and the body is refused — the write and the use
    /// are not assumed to be the same slot, they are required to be.
    #[test]
    fn a_different_temporary_in_the_argument_is_refused() {
        let mut seg = DEAD_TEMP_CALL.to_vec();
        let temp = &[0x9B, 0x82, 0x16, 0x86, 0x20, 0x0B, 0x0A];
        // The SECOND temp bind — the one the argument pushes.
        let second = crate::func::readers::find_subslice(&seg[at(temp) + 1..], temp).unwrap()
            + at(temp)
            + 1;
        seg[second + 6] = 0x0C;
        assert_eq!(no_effect_call(&seg), None);
    }

    /// **A real `memset` is not this shape.** Give the intrinsic the `void*`
    /// result type the 7 pointer-`memset` sites of `BustAMovePanel.cpp` carry —
    /// the form `docs/IL_CAST_CONVERT.md` §1.3 records, which c2 lowers to
    /// `b <memset>` with a REL24 — and the recognizer declines rather than
    /// reading it as a dead temporary.
    #[test]
    fn a_pointer_result_memset_is_not_a_dead_temporary() {
        let mut seg = DEAD_TEMP_CALL.to_vec();
        let k = at(&[0x40, 0x86, 0x41, 0x74]);
        seg.splice(k..k + 4, [0x40, 0x86, 0x43, 0x83, 0x08]);
        assert_eq!(no_effect_call(&seg), None);
    }

    /// **A different intrinsic is not this shape.** 172 is `memcpy`, whose
    /// expansion is also a REL24 tail call.
    #[test]
    fn a_different_intrinsic_selector_is_refused() {
        let mut seg = DEAD_TEMP_CALL.to_vec();
        seg[at(&[0x80, 0xAD, 0x00, 0x00, 0x00, 0x40]) + 1] = 0xAC;
        assert_eq!(no_effect_call(&seg), None);
    }

    /// **The result must be discarded.** Turn the `4B` into anything else and the
    /// walk refuses instead of reading a consumed value as emitting nothing.
    #[test]
    fn a_call_whose_result_is_not_discarded_is_refused() {
        let mut seg = DEAD_TEMP_CALL.to_vec();
        let k = at(&[0x4C, 0x4B, 0x3A]);
        seg[k + 1] = 0x41;
        assert_eq!(no_effect_call(&seg), None);
    }

    /// **The walk is TOTAL** — trailing bytes after the function tail refuse.
    /// This is what makes "the temporary is read nowhere else" structural rather
    /// than a search.
    #[test]
    fn trailing_bytes_after_the_function_tail_are_refused() {
        let mut seg = DEAD_TEMP_CALL.to_vec();
        seg.push(0x26);
        seg.push(0x11);
        assert_eq!(no_effect_call(&seg), None);
    }

    /// **An ordinary void call with no temporary is not this shape** — it has its
    /// own accepted body shape and must not come through here as well.
    #[test]
    fn a_body_with_no_temporary_is_not_recognized() {
        // `4C 4F 11 53 26 <tok> BD <void> 00 <id> 4C 4B <plumbing>`.
        let seg = &[
            0x4F, 0x1F, 0x80, 0x05, 0x00, 0x20, 0x00, 0x53, 0x4C, 0x4F, 0x11, 0x53, 0x26, 0x09,
            0x0A, 0xBD, 0x82, 0x07, 0x03, 0x00, 0x80, 0x0A, 0x10, 0x00, 0x00, 0x4C, 0x4B, 0x3A,
            0x0A, 0x0A, 0x54, 0x02, 0x29, 0x0A, 0x0A, 0x4F, 0x12, 0x47, 0x54, 0x01, 0x54, 0x00,
        ];
        assert_eq!(no_effect_call(seg), None);
    }

    /// An argument that is neither a literal, a formal nor the temporary refuses
    /// the whole body — the vocabulary is closed, not a default.
    #[test]
    fn an_argument_outside_the_vocabulary_refuses_the_body() {
        let mut seg = DEAD_TEMP_CALL.to_vec();
        seg[at(&[0xB9, 0x06, 0x0A])] = 0x26; // a data-symbol push
        assert_eq!(no_effect_call(&seg), None);
    }

    /// A load that names something other than one of this function's own formals
    /// is refused: the formals list is read from the segment and consulted.
    #[test]
    fn a_load_of_a_non_formal_is_refused() {
        let mut seg = DEAD_TEMP_CALL.to_vec();
        seg[at(&[0xB9, 0x06, 0x0A]) + 1] = 0x60; // a token the formals region does not list
        assert_eq!(no_effect_call(&seg), None);
    }
}
