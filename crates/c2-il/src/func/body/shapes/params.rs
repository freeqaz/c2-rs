//! The formal parameter list, read once.
//!
//! Every recognizer that reports `params` gets them from [`parse_params`] —
//! the positional index of a formal and its register number are two facts that
//! come apart (see `docs/CODEGEN_FP_ARGS.md`), and a second reader would be
//! the place they silently disagree.

use crate::func::body::expr::parse_formals;
use crate::func::body::Block;

use super::this_binding::{ThisBinding, parse_this_token};

/// The function's **argument registers in order**: `this` when the pre-body region
/// binds one, then the `2D` formals.
///
/// Every shape that maps a token to an argument register must use this rather than
/// [`parse_formals`], and that this needed saying is the bug. `parse_this_token`
/// existed and exactly one shape consulted it, so a non-static member function with
/// a *straight-line* body mapped its first explicit formal to r3 — the register
/// `this` occupies. `struct S8 { int a; int m(int x) const; };
/// int S8::m(int x) const { return x + 1; }` emitted `Port=Mismatch @ offset 537`:
/// `addi r3,r3,1` where the reference has `addi r3,r4,1`.
///
/// That is the same defect as the line-70 `this` bug — one fact with more than one
/// locator — and it survived that fix because the fix went where the bug had been
/// found rather than everywhere the fact was used. Found by an adversarial reviewer
/// probing an unrelated change.
///
/// An undetermined `this` binding **refuses**; it never silently means "absent".
pub(crate) fn parse_params(seg: &[u8], lo: usize) -> Result<Vec<u32>, Block> {
    let formals = parse_formals(seg, lo)?;
    match parse_this_token(seg, lo) {
        Some(ThisBinding::Absent) => Ok(formals),
        Some(ThisBinding::Bound(this_tok)) => {
            let mut v = Vec::with_capacity(formals.len() + 1);
            v.push(this_tok);
            v.extend_from_slice(&formals);
            Ok(v)
        }
        None => Err(Block::refuse(seg, lo, "this-undetermined")),
    }
}
