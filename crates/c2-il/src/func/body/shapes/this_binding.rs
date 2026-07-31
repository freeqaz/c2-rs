//! **The `this` binding** — how a member function's implicit first argument
//! appears in `.ex`, and what it binds to.
//!
//! The line-70 lesson lives here (`fixtures/cpp/il_this_line70.cpp`,
//! `docs/GAPS.md` §6 instances #1 and #2): `this` is *not* simply "token
//! whatever, argument zero". Two separate live mis-emits came from reading the
//! group around it as if it were an ordinary formal. Every consumer that needs
//! argument zero goes through [`ThisBinding`] rather than re-deriving it.

use crate::func::body::expr::formals_marker;
use crate::func::readers::{read_token_var, read_type};


/// The member-function `this` token, when this segment's pre-body region binds
/// one: `53 53 26 <fn> B9 <this> <TYPE> 99 <TYPE> 00 46`.
///
/// `this` is **not** in the `2D` formals list, and it occupies r3 — so every
/// explicit formal of a member function is one register higher than
/// [`parse_formals`]'s index implies. Captured, and it is a live off-by-one trap
/// for anything that maps formals to registers:
///
/// ```text
/// int C::g(int* q) const        { return *q; }   -> lwz r3,0(r4)   q is r4, not r3
/// int C::i(int v, int* q) const { return *q; }   -> lwz r3,0(r5)   q is r5, not r4
/// int D::s(int* q)              { return *q; }   -> lwz r3,0(r3)   static: no `this`
/// ```
///
/// Located against the **one** formals-marker anchor
/// ([`super::expr::formals_marker`]): the pre-body region is `26 <fn-tok>` followed
/// either by nothing or by exactly one `this` group, and whichever it is must land
/// *exactly* on that marker.
///
/// Both outcomes are established positively, and that is the point. This used to
/// return a bare `Option<u32>` and anchor on the first `0x46` byte in the segment,
/// so a `None` meant "no `this`" and "could not tell" alike — and the first `0x46`
/// is the known-bad anchor `parse_formals` documents, because a function on source
/// line 70 carries the line marker `4F 01 46`. A member function there reported no
/// `this`, every explicit formal shifted one register down, and
/// `int C::gp(int* q) const { return *q; }` emitted `lwz r3,0(r3)` where the
/// reference has `lwz r3,0(r4)` — a wrong-bytes emit inside an accepted class,
/// found by review and pinned by `fixtures/cpp/il_this_line70.cpp`.
///
/// Note that `99`'s trailing field is a one-byte varint while the visually
/// similar `9B`'s is a whole `read_token_var`; see `docs/IL_EXPR_LAYER.md` §7.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ThisBinding {
    /// The pre-body region runs straight from the function token to the formals
    /// marker: a free function or a `static` member, `this` in no register.
    Absent,
    /// A member function; the token occupies r3 and shifts every formal up one.
    Bound(u32),
}

/// `None` means **undetermined**, and the caller must refuse — never "absent".
pub(crate) fn parse_this_token(seg: &[u8], lo: usize) -> Option<ThisBinding> {
    let f = formals_marker(seg, lo)?;
    let mut found: Option<ThisBinding> = None;
    for q in 0..f {
        if seg[q] != 0x26 {
            continue;
        }
        let mut p = q + 1;
        let (_fn_tok, w) = match read_token_var(seg, p) {
            Some(x) => x,
            None => continue,
        };
        p += w;
        let binding = if p == f {
            ThisBinding::Absent
        } else {
            match read_this_group(seg, p) {
                Some((tok, end)) if end == f => ThisBinding::Bound(tok),
                _ => continue,
            }
        };
        // A second candidate landing on the marker means the region is not
        // determined by these bytes. Refuse rather than prefer one.
        if found.is_some() {
            return None;
        }
        found = Some(binding);
    }
    found
}

/// One `B9 <tok> <TYPE> 99 <TYPE> 00` group — the `this` push — returning its
/// token and the offset just past it.
fn read_this_group(seg: &[u8], at: usize) -> Option<(u32, usize)> {
    let mut p = at;
    if *seg.get(p)? != 0xB9 {
        return None;
    }
    p += 1;
    let (tok, w) = read_token_var(seg, p)?;
    p += w;
    let (_, _, _, tw) = read_type(seg, p)?;
    p += tw;
    if *seg.get(p)? != 0x99 {
        return None;
    }
    p += 1;
    let (_, _, _, tw) = read_type(seg, p)?;
    p += tw;
    if *seg.get(p)? != 0x00 {
        return None;
    }
    Some((tok, p + 1))
}
