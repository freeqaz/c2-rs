//! The **integer divide / modulo leaf**: `return a / b;` and `return a % b;`
//! over two formals, signed and unsigned.
//!
//! ```text
//!   B9 <tok_a> <T>      LOAD the dividend      T ∈ {int, unsigned}
//!   B9 <tok_b> <T>      LOAD the divisor       the SAME T
//!   05 | 06             DIV | MOD
//!   41 <T>              result type            the SAME T again
//!   <return plumbing>
//! ```
//!
//! Captured for all four cells (`work/w-divmod/probe/`, read out of
//! `c2rs census`'s own blocking hexdump):
//!
//! ```text
//!   int P(int a,int b){return a%b;}
//!     4c 4f 11 53 b9 e3 09 86 41 74 b9 e4 09 86 41 74 >06< 41 86 41 74 3a …
//!   unsigned P(unsigned a,unsigned b){return a%b;}
//!     4c 4f 11 53 b9 e3 09 86 42 75 b9 e4 09 86 42 75 >06< 41 86 42 75 3a …
//! ```
//!
//! # This is a lift, not a widening
//!
//! `codegen::ptr_walk_loop` already emits the signed-`%` spine, but only as
//! eight words welded into a twenty-word loop transcription. `w-hash` §9.1
//! recorded the reason it could not be lifted: it had seen **two distinct
//! `twi 6` placements** and could not name the discriminator, and a leaf
//! lowering owes an answer.
//!
//! **The answer is measured and it is not shipped, because this class does not
//! need it.** Over 161 cells graded against real `c2.dll`
//! (`work/w-divmod/twigrid.py`, `rootgrid.py`, `blockgrid.py`):
//!
//! > `twi 6` is emitted immediately after the **first instruction of the
//! > division's own basic block that is neither a multiply (`mulli`/`mullw`)
//! > nor a register-amount shift (`slw`/`srw`/`sraw`)** — provided the dividend
//! > is produced by an instruction *in that block*, the divisor is *live-in* to
//! > that block, and that block is *not a loop body*. Otherwise it stays inside
//! > the spine, in the fixed slot the spine's own schedule gives it.
//!
//! Every body this file accepts has **both operands live-in**, so the hoist
//! clause is false by construction and the placement is a **constant**. The
//! rule above is a measurement this class is deliberately on the far side of;
//! nothing here reads it, and a shape that could reach the other regime is
//! refused rather than scheduled.
//!
//! # The refusals, each with a measured counterexample
//!
//! Not one of these is caution. Each names a cell where real `c2` emits
//! something this file cannot:
//!
//! * **a computed operand.** `(a+1)%b` moves `twi 6` to the block's second slot
//!   (`twigrid` `dvd-add1`) and `a%(b+1)` leaves it in the spine but re-plans
//!   the registers (`dvs-add1`). Two different bodies, neither this one.
//! * **a constant divisor.** A non-zero literal emits **no trap at all** —
//!   `a%7` is `addi ; divw ; mulli ; subf` — a power of two becomes
//!   `srawi ; addze ; rlwinm ; subf`, `%1` and `%-1` collapse to a single
//!   `addi`, `/-1` is a bare `neg`, and a literal **zero** emits no division
//!   and an unconditional `twi 7, r0, 0`. Twenty-four graded literal cells,
//!   seven distinct bodies, none of them this one.
//! * **the operands in the other order** (`b%a`), a third formal, or either
//!   operand appearing twice.
//! * **any width but 4**. `short`/`signed char`/`unsigned char` bracket the
//!   spine with `extsh`/`extsb`/`rlwinm` **and move the traps adjacent**
//!   (`… andc subf twi twi extsb`, a placement the two-formal `int` cell never
//!   shows); `long long` is a `divd`/`tdi` spine.
//! * **`long` and `unsigned long`.** `docs/IL_TYPE_TAGS.md` §3.1 records that
//!   `c2` emits byte-identical code for them, and this file still refuses them,
//!   because *this lane did not grade them*. `eat_int_like` would have taken
//!   all four spellings for free; the two triples are matched literally
//!   instead, so the accepted set is exactly the graded set.
//!
//! The type triple is required to be the **same** in all three positions rather
//! than merely int-like in each. A mixed-width body carries a conversion the
//! grammar below would not consume anyway, but stating it as an equality makes
//! the gate one fact instead of three independent ones.

use crate::func::body::expr::{eat_return_plumbing, BODY_SCOPE_DEPTH};
use crate::func::body::BodyShape;
use crate::func::readers::{eat, eat_byte, read_token_var, INT_TYPE, UINT_TYPE};
use crate::func::DivModLeaf;

use super::params::parse_params;

/// IL opcode byte for integer DIVIDE.
const IL_DIV: u8 = 0x05;
/// IL opcode byte for integer MODULO. The same byte
/// [`super::ptr_walk_loop`] consumes inside the loop body.
const IL_MOD: u8 = 0x06;

/// Try to parse the two-formal integer `/` or `%` leaf.
///
/// Non-committal in the sense the whole ladder is: it works on its own cursor
/// and returns `None` without side effects, so a body that is not exactly this
/// shape keeps whatever blocker the productions below it report — today
/// `expr-op-0x05` / `expr-op-0x06`, which is where this population is counted.
pub(crate) fn try_parse_div_mod_leaf(seg: &[u8], start: usize, lo: usize) -> Option<BodyShape> {
    let mut p = start;

    // LOAD the dividend, and fix the type triple from it.
    if !eat_byte(seg, &mut p, 0xB9) {
        return None;
    }
    let (dividend, w) = read_token_var(seg, p)?;
    p += w;
    let signed = if eat(seg, &mut p, &INT_TYPE) {
        true
    } else if eat(seg, &mut p, &UINT_TYPE) {
        false
    } else {
        return None;
    };
    let ty = if signed { INT_TYPE } else { UINT_TYPE };

    // LOAD the divisor, in the SAME type.
    if !eat_byte(seg, &mut p, 0xB9) {
        return None;
    }
    let (divisor, w) = read_token_var(seg, p)?;
    p += w;
    if !eat(seg, &mut p, &ty) {
        return None;
    }

    // The operator.
    let is_mod = match *seg.get(p)? {
        IL_MOD => true,
        IL_DIV => false,
        _ => return None,
    };
    p += 1;

    // `41 <T>` result type, the SAME T, then the plumbing — which must reach
    // the segment end, so a trailing statement or a post-op rejects the whole
    // function rather than being silently dropped.
    if !eat_byte(seg, &mut p, 0x41) || !eat(seg, &mut p, &ty) {
        return None;
    }
    eat_return_plumbing(seg, &mut p, false, BODY_SCOPE_DEPTH).ok()?;

    // **Exactly two formals, dividend in slot 0 and divisor in slot 1.** The
    // emitted spine reads r3 and r4 by name and nothing here models a register
    // move, so every other arrangement is a different body. `b%a` and a third
    // formal are both refused here, and both are graded must-refuse cells in
    // `work/w-divmod/crossgrade.py`.
    let params = parse_params(seg, lo).ok()?;
    if params.len() != 2 || params[0] != dividend || params[1] != divisor {
        return None;
    }

    Some(BodyShape::DivModLeaf(DivModLeaf { params, is_mod, signed }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The four captured bodies, byte for byte out of `c2rs census`'s blocking
    /// hexdump (`work/w-divmod/probe/`). `lo` is the segment offset the params
    /// parse reads from; these fragments are exercised through the whole-body
    /// parser in `crate::func::body::tests`, so what is asserted here is the
    /// grammar's accept/refuse boundary on the operator and type positions.
    fn seg(op: u8, ty: [u8; 3]) -> Vec<u8> {
        let mut v = vec![0xB9, 0xE3, 0x09];
        v.extend_from_slice(&ty);
        v.extend_from_slice(&[0xB9, 0xE4, 0x09]);
        v.extend_from_slice(&ty);
        v.push(op);
        v.push(0x41);
        v.extend_from_slice(&ty);
        v
    }

    /// A mismatched type triple refuses before the plumbing is even reached, so
    /// this test does not depend on the return-plumbing bytes.
    #[test]
    fn a_mixed_type_triple_refuses() {
        let mut v = vec![0xB9, 0xE3, 0x09];
        v.extend_from_slice(&INT_TYPE);
        v.extend_from_slice(&[0xB9, 0xE4, 0x09]);
        v.extend_from_slice(&UINT_TYPE);
        v.push(IL_MOD);
        assert!(try_parse_div_mod_leaf(&v, 0, 0).is_none());
    }

    /// Only `05` and `06` are operators here. `02` (ADD) and `04` (MUL) are the
    /// straight-line chain's and must fall through to it untouched.
    #[test]
    fn only_the_two_division_opcodes_are_taken() {
        for op in [0x02u8, 0x03, 0x04, 0x07, 0x09, 0x0B, 0x0C] {
            assert!(
                try_parse_div_mod_leaf(&seg(op, INT_TYPE), 0, 0).is_none(),
                "opcode {op:#04x} must not be taken by the div/mod leaf"
            );
        }
    }

    /// `long` and `unsigned long` are byte-identical to `int`/`unsigned` in the
    /// emitted code (`docs/IL_TYPE_TAGS.md` §3.1) and are refused anyway,
    /// because this lane graded neither. The assertion exists so that a later
    /// widening has to delete a test rather than relax a helper by accident.
    #[test]
    fn long_spellings_are_refused_even_though_c2_emits_the_same_bytes() {
        const LONG: [u8; 3] = [0x86, 0x41, 0x12];
        const ULONG: [u8; 3] = [0x86, 0x42, 0x22];
        for ty in [LONG, ULONG] {
            assert!(try_parse_div_mod_leaf(&seg(IL_MOD, ty), 0, 0).is_none());
        }
    }
}
