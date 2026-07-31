//! The **indirect-load leaf** and the pointer-identity leaf.
//!
//! `return p->m;` / `return *p;` / `return p[k];` at any admitted pointee
//! width, plus the pointer-valued forms that decode as the same load.
//! `finish_indirect_load{,_of}` are shared by exactly these two recognizers.
//! Consumer of [`super::designator`].

use crate::func::body::chain::straight_line_is_out_of_class;
use crate::func::body::expr::{eat_return_plumbing, BODY_SCOPE_DEPTH};
use crate::func::body::BodyShape;
use crate::func::readers::{
    eat, eat_byte, eat_int_like, eat_value_type, is_ptr4_kind, is_ptr_to_4, is_volatile_tag,
    read_token_var,
    read_type, read_varint, value_class, ValueClass, INT_TYPE,
};
use crate::func::IlOp;

use super::designator::{parse_base_member_designator, sized_ptee, sized_ptr_width};
use super::params::parse_params;

/// Try to parse an **indirect-load leaf**: a whole body that is one load through
/// a pointer, `return *p;` / `return s->m;` / `return p[k];` and nothing else.
///
/// ```text
///   B9 <base-tok> <PTR-TYPE>                     the base pointer
///   [ 33 <int-like> <off>  27 <PTR-TYPE> ]       ONE member byte-offset add, or
///   [ 33 <long>     <off>  28 00 00      ]       ONE subscript byte-offset add
///   30 <INT4|PTR4-TYPE>                          the indirect load
///   [ 2C <same class> 00 ]                       a cv-qualification strip
///   41 <same class>                              result type
///   <return plumbing, reaching the segment end>
/// ```
///
/// c2 lowers all of it to **one `lwz rD, off(rBase)`** plus the `blr`, folding the
/// offset into the displacement. Captured, one instruction each:
///
/// ```text
/// int f(int* p)                { return *p; }      -> lwz r3,0(r3)
/// int f(int a, int* p)         { return *p; }      -> lwz r3,0(r4)
/// int f(int a, int b, int* p)  { return *p; }      -> lwz r3,0(r5)
/// int f(S* s)                  { return s->d; }    -> lwz r3,16(r3)     (27, off 0x10)
/// int f(int* p)                { return p[3]; }    -> lwz r3,12(r3)     (28, off 0x0c)
/// int f(int* p)                { return p[-1]; }   -> lwz r3,-4(r3)     (off 0xfc = -4)
/// int f(int* p)                { return p[8000]; } -> lwz r3,32000(r3)
/// int C::f() const             { return b; }       -> lwz r3,4(r3)      (`this`)
/// unsigned/long/const/volatile int *                -> the same bare `lwz`
/// ```
///
/// Why every gate below is load-bearing rather than defensive — each is a
/// *captured* case where the same-looking IL lowers differently:
///
/// * **Exactly one offset add.** `p[i][j]` chains two of them and needs
///   `slwi ; add ; slwi ; lwzx`; `p[i].b` chains a `28` and a `27`.
/// * **The offset must fit the 16-bit displacement.** `p[100000]` (offset 400000)
///   is `lis r11,6 ; ori r11,r11,0x1a80 ; lwzx r3,r3,r11` instead.
/// * **The offset must be a literal.** A variable index is
///   `slwi r11,r4,2 ; lwzx r3,r11,r3` — a different instruction, an extra one, and
///   a scratch register.
/// * **The `28` payload must be exactly `00 00`.** Those two bytes are `00 00` at
///   every site captured (constant and variable indices, 1/4/8-byte elements,
///   negative indices, 2-D arrays, bitfields) and their meaning is UNKNOWN, so
///   anything else refuses.
/// * **The loaded type must be a 1-, 2-, 4- or 8-byte integer** ([`SIZED_PTEE`])
///   **or a 4-byte pointer** ([`is_ptr4_kind`]). The width picks the
///   instruction: `char *` is `lbz`, `short *` is `lhz`, `long long *` is `ld`,
///   `float *` is `lfs`, `double *` is `lfd` — all captured, all different, and
///   the FP ones are still refused. A **pointer** value is the one non-integer
///   case that lowers to the same bare `lwz` as a 4-byte integer:
///   `int* H::gpi() const { return mpi; }` is `lwz r3,0(r3) ; blr`, the same
///   scheme as the `int` getter beside it (`docs/IL_LOAD_TYPES.md` §3/§4), which
///   is why it needs no encoder.
///
///   Note the gate is on the loaded value's *own* width, never the pointee's —
///   loading a `char*` **member** is `lwz`, while loading *through* a `char*` is
///   `lbz`, and both spell `char` somewhere in the type. The two questions have
///   two predicates ([`is_ptr4_kind`] and [`is_ptr_to_4`]) for exactly that
///   reason.
/// * **Nothing may follow the load but the return.** `*p + 1` puts the load in
///   r11, and `*p * 3` is strength-reduced; see [`IlOp::LoadInd`].
/// * **A `this`-bearing function must have its `this` found**, because `this`
///   takes r3 and shifts every explicit formal up one
///   ([`parse_this_token`]).
///
/// Returns `None` — cursor untouched — for anything that is not exactly this
/// shape.
pub(crate) fn try_parse_indirect_load_leaf(seg: &[u8], start: usize, lo: usize) -> Option<BodyShape> {
    // A member inherited from a **base class** does not use the `27` offset-add
    // below; c1xx routes it through intrinsic **2117 `base-member-addr`**, which
    // computes the same address and is the single largest decode bucket in the
    // family (6.3% of blocked functions). Try that designator first — it is
    // anchored on a literal `33`, where the plain form is anchored on `B9`, so the
    // two cannot be confused.
    if let Some(shape) = try_parse_base_member_load(seg, start, lo) {
        return Some(shape);
    }
    let mut p = start;

    // The base pointer LOAD.
    if !eat_byte(seg, &mut p, 0xB9) {
        return None;
    }
    let (base_tok, w) = read_token_var(seg, p)?;
    p += w;
    let (tag, kind, _, tw) = read_type(seg, p)?;
    // …and NOT `volatile`. `int f(int x, int* volatile p){ return *p; }` homes the
    // pointer in the frame and reloads it, where this leaf emits one `lwz` —
    // `Port=Mismatch @ 8`, and pre-existing. The gate is on the base LOAD ONLY:
    // the same bit on the `27` pointee below is a pointer-to-volatile
    // (`volatile S* p`) and on the `30` it is a volatile member, and BOTH of those
    // are one `lwz` either way and stay in class. `readers::is_volatile_tag` has
    // the four-row measurement.
    if !is_ptr_to_4(tag, kind) || is_volatile_tag(tag) {
        return None;
    }
    p += tw;

    // At most ONE byte-offset add, in either of its two forms. Both push a
    // byte offset as a literal and add it to the designator; `27` re-types the
    // result and `28` does not (`docs/IL_EXPR_LAYER.md` §4).
    let mut off: i32 = 0;
    // What the byte-offset add says the pointee's width is, when it says anything.
    // `27` re-types the address and its tag carries the POINTEE width, so it is a
    // second, independent statement of the width the `30` load will announce; the
    // two are required to agree. `28` and the bare deref say nothing, and then the
    // `30` type is the only evidence.
    let mut ptee_width: Option<u8> = None;
    if *seg.get(p)? == 0x33 {
        let mut probe = p + 1;
        // The literal's own type: `86 41 74` (int) for a member offset,
        // `86 41 12` (long) for a subscript offset. Both are int-like.
        if !eat_int_like(seg, &mut probe) {
            return None;
        }
        let k = read_varint(seg, &mut probe)?;
        match *seg.get(probe)? {
            0x27 => {
                probe += 1;
                let (tag, kind, _, tw) = read_type(seg, probe)?;
                ptee_width = if is_ptr_to_4(tag, kind) {
                    Some(4)
                } else {
                    // A pointer to a 1-, 2- or 8-byte object; captured with and
                    // without the tag's const bit ([`SIZED_PTR`]).
                    Some(sized_ptr_width(tag, kind)?)
                };
                probe += tw;
            }
            0x28 => {
                // The two trailing bytes are `00 00` at every captured site and
                // are not understood; anything else refuses.
                probe += 1;
                if !eat(seg, &mut probe, &[0x00, 0x00]) {
                    return None;
                }
            }
            _ => return None,
        }
        off = k;
        p = probe;
    }
    finish_indirect_load_of(seg, p, lo, base_tok, off, ptee_width)
}

/// The tail shared by both indirect-load designators (the plain `27`/`28` offset
/// add and the intrinsic-2117 `base-member-addr` form): the `30` load, an optional
/// cv strip, the result type, the return plumbing, and binding the base pointer to
/// its argument register.
///
/// Factored out rather than duplicated because the two designators compute the
/// *same* address by different routes and must lower identically; two copies of
/// this tail would be two places for the `lwz` displacement bound to drift.
fn finish_indirect_load(
    seg: &[u8],
    p: usize,
    lo: usize,
    base_tok: u32,
    off: i32,
) -> Option<BodyShape> {
    finish_indirect_load_of(seg, p, lo, base_tok, off, Some(4))
}

/// [`finish_indirect_load`] with the pointee width the *designator* announced, if
/// it announced one (`Some(4)` = "a 4-byte object, and nothing else will do").
///
/// T3 widens the load type from "a 4-byte integer" to any [`SIZED_PTEE`] scalar,
/// which is where the two designators stop being interchangeable: the plain `27`
/// form re-types the address with the pointee's width and so *knows* it, while the
/// intrinsic-2117 base-member form was captured only over 4-byte members and pins
/// itself to 4 by passing `Some(4)`.
pub(crate) fn finish_indirect_load_of(
    seg: &[u8],
    mut p: usize,
    lo: usize,
    base_tok: u32,
    off: i32,
    ptee_width: Option<u8>,
) -> Option<BodyShape> {
    if !(-0x8000..=0x7FFF).contains(&off) {
        return None;
    }

    // The indirect load itself. The loaded value is either a 4-byte integer or a
    // 4-byte **pointer** — the two cases c2 lowers with the identical `lwz`
    // (`docs/IL_LOAD_TYPES.md` §3/§4: `g_pi` is `lwz r3,24(r3)`, byte-identical
    // in scheme to the in-class `g_i`), which is why this rung needs no encoder.
    if !eat_byte(seg, &mut p, 0x30) {
        return None;
    }
    let (tag, kind, _, tw) = read_type(seg, p)?;
    // Width 4 — a 4-byte integer or a 4-byte **pointer**, the two classes c2
    // lowers with the identical bare `lwz` (`docs/IL_LOAD_TYPES.md` §3/§4: the
    // pointer member `g_pi` is `lwz r3,24(r3)`, byte-identical in scheme to the
    // in-class `g_i`), which is why the pointer half needs no encoder.
    let load_op = if let Some(loaded) = value_class(tag, kind) {
        if matches!(ptee_width, Some(w) if w != 4) {
            return None;
        }
        p += tw;

        // An optional cv-qualification strip, in the loaded value's own class:
        // provably free over a 4-byte integer source (see [`is_int4_type`]) and
        // provably free over a pointer one (`2C ptr→ptr` emits nothing —
        // `void* f(H* p){return p;}` is a bare `blr`). The trailing varint must
        // be the `00` observed at all 14,098 aligned sites.
        //
        // The two classes are kept apart rather than merged into one "either"
        // test. A cross-class `2C` — pointer source, int target, or the reverse
        // — is a *reinterpret* this port has never probed, and the neighbouring
        // shape that would look identical under the wrong rule is the one that
        // matters here: an address-adjusting up/downcast also produces a pointer
        // from a pointer, and it costs an `addi`. It never comes through `2C`
        // (it is intrinsic 2113/2114/2115), but the way to keep that true is to
        // admit only the conversions each class was measured to make free.
        if *seg.get(p)? == 0x2C {
            let mut probe = p + 1;
            if !eat_value_type(seg, &mut probe, loaded) || !eat_byte(seg, &mut probe, 0x00) {
                return None;
            }
            p = probe;
        }

        // Result type, in the same class as the load.
        if !eat_byte(seg, &mut p, 0x41) || !eat_value_type(seg, &mut p, loaded) {
            return None;
        }
        IlOp::LoadInd { off }
    } else {
        let (width, signed) = sized_ptee(tag, kind)?;
        if matches!(ptee_width, Some(w) if w != width) {
            return None;
        }
        // `ld` is DS-form: the displacement's low two bits are the form's, so an
        // offset that is not a multiple of 4 cannot be encoded at all. Natural
        // alignment makes one unreachable through a struct member, so this gate has
        // no witness — which is exactly why it refuses instead of masking.
        if width == 8 && off % 4 != 0 {
            return None;
        }
        p += tw;

        // The optional conversion. Two — and only two — targets are captured over a
        // narrow load, and they are not the same thing:
        //
        //  * the **same** width and signedness (a cv strip: `30 a2 11 93 20`
        //    `2c 82 11 70 00`) emits nothing, exactly as at width 4;
        //  * **exactly `int`** (`2c 86 41 74 00`) is a real widening. Free for an
        //    unsigned pointee (`lbz`/`lhz` zero-extend), one extra `extsb` for a
        //    signed byte, and **mode-dependent** for a signed halfword — `/O1`
        //    emits `lha r3` where `/Ox` and `/O2` emit `lhz r11 ; extsh r3,r11`
        //    (measured, both ways, `docs/IL_LOAD_TYPES.md` §3 states only the `/Ox`
        //    form). This lowering path has no optimization-mode parameter, so the
        //    signed-halfword widening is refused rather than guessed at.
        //
        // Anything else — `unsigned int` as the target (whose emit is measured
        // identical to `int`'s, but whose (source × target) matrix is not),
        // a widening of a `long long`, a narrowing, a change of signedness at the
        // same width — refuses. Each would need its own capture set.
        let mut sext = false;
        let mut int_result = false;
        if *seg.get(p)? == 0x2C {
            let mut probe = p + 1;
            let (t2, k2, _, tw2) = read_type(seg, probe)?;
            if eat(seg, &mut probe, &INT_TYPE) {
                if width != 1 && !(width == 2 && !signed) {
                    return None;
                }
                sext = signed;
                int_result = true;
            } else if sized_ptee(t2, k2) == Some((width, signed)) {
                probe += tw2;
            } else {
                return None;
            }
            if !eat_byte(seg, &mut probe, 0x00) {
                return None;
            }
            p = probe;
        }

        // Result type: the value's type after the conversion, stated again. Every
        // capture agrees byte for byte, so this is required rather than skipped.
        if !eat_byte(seg, &mut p, 0x41) {
            return None;
        }
        if int_result {
            if !eat(seg, &mut p, &INT_TYPE) {
                return None;
            }
        } else {
            let (t3, k3, _, tw3) = read_type(seg, p)?;
            if sized_ptee(t3, k3) != Some((width, signed)) {
                return None;
            }
            p += tw3;
        }
        IlOp::LoadIndSized { off, width, sext }
    };
    // The shared plumbing, which must reach the segment end.
    eat_return_plumbing(seg, &mut p, false, BODY_SCOPE_DEPTH).ok()?;

    let params = parse_params(seg, lo).ok()?;
    let ix = params.iter().position(|&t| t == base_tok)?;
    // Past the eighth argument the value is stack-homed, which needs a frame.
    if ix >= 8 {
        return None;
    }
    Some(BodyShape::IndirectLoad {
        params,
        ops: vec![IlOp::Load(base_tok), load_op],
    })
}

// [`ValueClass`], [`value_class`] and [`eat_value_type`] used to live here.
// They moved to `readers.rs` when `parse_expr` needed the same three (D12,
// `docs/IL_CALL_IN_EXPR.md` §24) — one fact, one locator, exactly as
// `is_ptr4_kind` moved in §21.3. Nothing about them changed.


/// Try to parse a **pointer-identity leaf**: a whole body that is one pointer
/// returned unchanged — `return p;`, `return this;`, and pointer-to-pointer
/// casts of either.
///
/// ```text
///   B9 <tok> <PTR4-TYPE>            the value
///   [ 2C <PTR4-TYPE> 00 ]           a cv-strip / ptr→ptr cast, emitting nothing
///   41 <PTR4-TYPE>                  result type
///   <return plumbing, reaching the segment end>
/// ```
///
/// c2 emits **no instruction at all** for the value: the pointer is already in
/// its incoming argument register, so the body is a bare `blr` — exactly what
/// the integer identity `int f(int a){ return a; }` already produces, which is
/// why this hands back the existing [`BodyShape::StraightLine`] lowering and
/// needs no codegen. MEASURED (`docs/IL_LOAD_TYPES.md` §3, probe `p10`):
/// `void* f(H* p){ return p; }` is one `blr`, and a `2C` from pointer to pointer
/// emits nothing.
///
/// Three real translation units (headtracker, MemMgr, Sorting) carry 16 bodies
/// of exactly this grammar, all four of the accepted tag spellings among them.
///
/// The gates, and the neighbour each one separates:
///
/// * **No offset add.** `B9 <ptr> 33 <int> <k> 27 <ptr> 2C <ptr> 00 41 <ptr>` —
///   the same production *minus the `30`* — is `return &s->m;`, and it emits an
///   `addi`. It occurs: 7 of the 40 pointer-shaped bodies in the three TUs
///   scanned are this, and admitting them as identities would emit a bare `blr`
///   where c2 emits `addi r3,r3,12`. So the identity leaf is anchored on the
///   `B9` *immediately* followed by the result, and the offset-add form has no
///   path into it.
/// * **The value must be a formal or `this`, bound positively.** An
///   `Undetermined` `this` refuses (the line-70 rule).
/// * **The result must already be in r3.** `S* f(int a, S* s){ return s; }` is
///   `mr r3,r4`, a real instruction — refused by the shared
///   [`straight_line_is_out_of_class`], the same predicate the arithmetic path
///   uses, rather than by a second copy of the rule.
/// * **Pointer *literals* are elsewhere.** `return 0;` typed as a pointer is a
///   `33` LITERAL (census `expr-lit-type-8643xx`) and needs an `li`; this
///   production is anchored on `B9` and cannot reach it.
pub(crate) fn try_parse_ptr_identity_leaf(seg: &[u8], start: usize, lo: usize) -> Option<BodyShape> {
    let mut p = start;
    if !eat_byte(seg, &mut p, 0xB9) {
        return None;
    }
    let (tok, w) = read_token_var(seg, p)?;
    p += w;
    let (tag, kind, _, tw) = read_type(seg, p)?;
    // …and NOT `volatile`: the identity of a volatile pointer formal is a frame,
    // not a bare `blr` (`readers::is_volatile_tag`).
    if !is_ptr4_kind(tag, kind) || is_volatile_tag(tag) {
        return None;
    }
    p += tw;

    if *seg.get(p)? == 0x2C {
        let mut probe = p + 1;
        if !eat_value_type(seg, &mut probe, ValueClass::Ptr4) || !eat_byte(seg, &mut probe, 0x00) {
            return None;
        }
        p = probe;
    }

    if !eat_byte(seg, &mut p, 0x41) || !eat_value_type(seg, &mut p, ValueClass::Ptr4) {
        return None;
    }
    eat_return_plumbing(seg, &mut p, false, BODY_SCOPE_DEPTH).ok()?;

    let params = parse_params(seg, lo).ok()?;
    let ix = params.iter().position(|&t| t == tok)?;
    // Past the eighth argument the value is stack-homed, which needs a frame.
    if ix >= 8 {
        return None;
    }
    let ops = vec![IlOp::Load(tok)];
    if straight_line_is_out_of_class(&ops, &params) {
        return None;
    }
    Some(BodyShape::StraightLine { params, ops })
}

/// The intrinsic-**2117** (`base-member-addr`) designator: reading a member that
/// the object inherits from a non-virtual base.
///
/// `p->d` for a member declared directly in `D` uses the ordinary `27` offset-add
/// in [`try_parse_indirect_load_leaf`]. A member inherited from a base does not —
/// c1xx emits an intrinsic call whose three arguments are `(0, base_offset, p)`:
///
/// ```text
///   33 <int> 80 45 08 00 00       LITERAL 2117 — the selector, always the wide form
///   40 <ptr>                      intrinsic call, result type a pointer
///   66 <n> <n × 2-byte type ref>  argument-region header (see below)
///   55 <int>                      selector argument terminator
///   33 <int> <m>     55 <int>     arg 1 — the member's offset WITHIN THE BASE
///   33 <int> <b>     55 <int>     arg 2 — the BASE's offset within the object
///   B9 <tok> <ptr>   55 <ptr>     arg 3 — the object pointer
///   4C                            call end
/// ```
///
/// **The address is `p + m + b`** — the two literals *add*. Established by the
/// witness that separates a sum from "whichever one is nonzero", since every
/// simpler case has one of them zero:
///
/// ```text
///   struct A { int a0, a1; }; struct B { int b0, b1, b2; }; struct D : A, B {};
///   p->b2   args (8, 8)   ->  lwz r3,0x10(r3)     16 = 8 + 8   <- both nonzero
///   p->b0   args (0, 8)   ->  lwz r3,8(r3)
///   p->a1   args (4, 0)   ->  lwz r3,4(r3)
/// ```
///
/// then the identical `30`/`41`/return tail as the `27` form, which is why this
/// hands back the same [`BodyShape::IndirectLoad`] and needs no new codegen: the
/// address is `p + off`, and the load of it is `lwz rD, off(rB)` either way.
/// Verified against captures —
///
/// ```text
///   struct A { int a; }; struct B { int b; }; struct D : A, B { int d; };
///   p->a  (A at 0)   80630000   lwz r3,0(r3) ; blr      <- 2117, off 0
///   p->b  (B at 4)   80630004   lwz r3,4(r3) ; blr      <- 2117, off 4
///   p->d  (at 8)     80630008   lwz r3,8(r3) ; blr      <- the `27` form
/// ```
///
/// — so the port already emitted the third of those byte-exactly and refused the
/// first two purely on the decode.
///
/// Fail-closed specifics:
///
/// * the selector must be **exactly** 2117 in its wide five-byte form. 2113–2119
///   are seven different operations and only this one is an unguarded
///   `base + constant`; 2114/2115 add a null guard, 2116/2118 go through a
///   vbtable, 2119 is a runtime call (`docs/IL_INTRINSIC_CALL.md`).
/// * the argument-region header is `66 <n>` followed by *n* two-byte type
///   references, and is skipped **structurally** rather than matched as a
///   constant. `n` is 2 for a single inheritance step and 3 for two
///   (`struct E : D`, `D : A, B` — `66 03 89 20 83 20 80 20`), so a fixed
///   six-byte match silently refused every multi-level case. The refs themselves
///   are not decoded; nothing downstream needs them, and the *value* arguments
///   that follow are what carry the address.
/// * the two offsets are summed with `checked_add`, and the sum must fit the `lwz`
///   16-bit displacement — checked by the shared tail, so a class large enough to
///   overflow it refuses instead of wrapping.
pub(crate) fn try_parse_base_member_load(seg: &[u8], start: usize, lo: usize) -> Option<BodyShape> {
    let (off, base_tok, p) = parse_base_member_designator(seg, start, is_ptr_to_4)?;
    finish_indirect_load(seg, p, lo, base_tok, off)
}

#[cfg(test)]
mod tests {
    // The single `mod tests` this was split out of opened with
    // `use super::*;`; the globs keep that reach.
    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
    use crate::func::body::shapes::*;
    #[allow(unused_imports)]
    use crate::func::body::shapes::testutil::*;
    #[allow(unused_imports)]
    use crate::func::body::{parse_segment, parse_segment_detail};
    #[allow(unused_imports)]
    use crate::func::bundle::LO_MARKER;
    #[allow(unused_imports)]
    use crate::func::readers::find_subslice;
    #[allow(unused_imports)]
    use crate::func::sy::{Formals, SyView};
    #[allow(unused_imports)]
    use crate::func::test_fixtures::*;
    #[test]
    fn indirect_load_leaf_decodes_deref_member_and_subscript() {
        assert_eq!(
            parse_segment(&free_fn(IND_DEREF), NO_LOCALS),
            Some(BodyShape::IndirectLoad {
                params: vec![0xEE09],
                ops: vec![IlOp::Load(0xEE09), IlOp::LoadInd { off: 0 }],
            })
        );
        assert_eq!(
            parse_segment(&free_fn(IND_MEMBER0), NO_LOCALS),
            Some(BodyShape::IndirectLoad {
                params: vec![0xFE09],
                ops: vec![IlOp::Load(0xFE09), IlOp::LoadInd { off: 0 }],
            })
        );
        // The offset is a SIGNED short-form byte, and `-1` on an `int *` is −4
        // bytes — the scale is already applied by the front end.
        assert_eq!(
            parse_segment(&free_fn(IND_SUBSCRIPT_NEG), NO_LOCALS),
            Some(BodyShape::IndirectLoad {
                params: vec![0x100A],
                ops: vec![IlOp::Load(0x100A), IlOp::LoadInd { off: -4 }],
            })
        );
    }

    #[test]
    fn indirect_load_leaf_binds_this_as_argument_zero() {
        // `this` is not in the `2D` list, so `params` must be built from the
        // pre-body binding — otherwise the base register is unknown (or, worse,
        // an explicit formal is mapped one register low).
        let lo = find_subslice(IND_THIS_GETTER, &LO_MARKER).unwrap();
        assert_eq!(parse_this_token(IND_THIS_GETTER, lo), Some(ThisBinding::Bound(0xF809)));
        assert_eq!(
            parse_segment(&free_fn(IND_THIS_GETTER), NO_LOCALS),
            Some(BodyShape::IndirectLoad {
                params: vec![0xF809],
                ops: vec![IlOp::Load(0xF809), IlOp::LoadInd { off: 4 }],
            })
        );
    }

    /// T3: the four accepted non-4-byte pointee shapes, each from a whole captured
    /// segment, plus the two refusals that separate them from wrong bytes.
    #[test]
    fn indirect_load_leaf_decodes_narrow_and_wide_pointees() {
        let ind = |tok: u32, off: i32, width: u8, sext: bool| {
            Some(BodyShape::IndirectLoad {
                params: vec![tok],
                ops: vec![IlOp::Load(tok), IlOp::LoadIndSized { off, width, sext }],
            })
        };
        // `char g_c_c(char*)`: a signed byte with NO conversion extends nothing.
        assert_eq!(
            parse_segment(NARROW_CHAR_DEREF, NO_LOCALS),
            ind(0x0F0A, 0, 1, false)
        );
        // `int g_i_c(char*)`: the same load + `2C 86 41 74 00` — the one case that
        // pays an instruction, so `sext` is the only field that differs.
        assert_eq!(
            parse_segment(NARROW_CHAR_TO_INT, NO_LOCALS),
            ind(0x2A0A, 0, 1, true)
        );
        // `int g_i_us(unsigned short*)`: the same token over an unsigned pointee is
        // free. If the parse read the token instead of the pointee's signedness,
        // this and the previous case would be indistinguishable — and one of the two
        // objs would be wrong.
        assert_eq!(
            parse_segment(NARROW_USHORT_TO_INT, NO_LOCALS),
            ind(0x3A0A, 0, 2, false)
        );
        // `long long m_q(S*)`: offset 16 folds into the DS-form displacement.
        assert_eq!(
            parse_segment(NARROW_LL_MEMBER, NO_LOCALS),
            ind(0x4F0A, 16, 8, false)
        );
        // `char C::t_c() const`: `this` binds at index 0, and the `2C` is a cv-strip
        // (const char → char) rather than a widening, so no extension.
        assert_eq!(
            parse_segment(NARROW_CONST_CHAR_THIS, NO_LOCALS),
            ind(0x530A, 0, 1, false)
        );
        // Refused: a signed halfword widened to int is `lha r3` at `/O1` and
        // `lhz r11 ; extsh r3,r11` at `/Ox` — one shape, two lowerings, no mode
        // here to choose with.
        assert_eq!(parse_segment(NARROW_SHORT_TO_INT_REFUSED, NO_LOCALS), None);
        // Refused: a `#pragma pack(1)` 8-byte member. Its tag says align-1, and
        // reading the width from the tag would emit `lbz` for a `long long`.
        assert_eq!(parse_segment(NARROW_LL_PACKED_REFUSED, NO_LOCALS), None);
    }

    /// The `27` byte-offset-add type states the pointee's width, and the `30` load
    /// states it again. They must **agree**: the two are separate fields in separate
    /// productions, and a parse that trusted only one of them would accept a
    /// splice whose two halves disagree — where c2's own output cannot.
    #[test]
    fn indirect_load_leaf_requires_the_offset_type_and_the_load_to_agree_on_width() {
        // `long long m_q(S*)`, with the `27` type re-tagged to a pointer-to-1-byte
        // (`88 43 93 08` → `82 43 93 08`) and nothing else touched.
        let lo = find_subslice(NARROW_LL_MEMBER, &LO_MARKER).unwrap();
        let mut mismatched = NARROW_LL_MEMBER.to_vec();
        let at = lo + 17; // the `27` type's tag
        assert_eq!(mismatched[at - 1], 0x27, "the 27 marker");
        assert_eq!(mismatched[at], 0x88, "the pointee-width tag");
        mismatched[at] = 0x82;
        assert_eq!(parse_segment(&mismatched, NO_LOCALS), None);
        // Control: the same splice on the *load's* tag alone also refuses, so the
        // agreement is required in both directions rather than one being ignored.
        let mut load_only = NARROW_LL_MEMBER.to_vec();
        assert_eq!(load_only[lo + 21], 0x30, "the load marker");
        load_only[lo + 22] = 0x82;
        assert_eq!(parse_segment(&load_only, NO_LOCALS), None);
    }

    #[test]
    fn indirect_load_leaf_refuses_the_adjacent_shapes() {
        // Splice one field of IND_DEREF at a time. Each variant is a construct
        // whose reference codegen differs (see fixtures/cpp/il_expr_load_neg.cpp).
        //
        // Offsets are measured from the `LO` marker rather than written as absolute
        // indices: they were absolute, and prepending the real `53 53 26 <fn>`
        // prologue to the pinned segment silently shifted every one of them.
        // Offsets are relative to the `LO` marker, which is a landmark the segment
        // itself defines.
        let base = |seg: &[u8]| find_subslice(seg, &LO_MARKER).unwrap();
        let d = base(IND_DEREF);
        let bad = |patch: &[(usize, u8)]| {
            let mut s = IND_DEREF.to_vec();
            for &(i, b) in patch {
                s[d + i] = b;
            }
            parse_segment(&free_fn(&s), NO_LOCALS)
        };
        // A `char` pointee is `lbz`, not `lwz` (`30 82 11 70`) — a *different* op,
        // and since T3 a modeled one: it must come out as a width-1 load, never as
        // the 4-byte [`IlOp::LoadInd`] whose `lwz` would read three bytes too many.
        assert_eq!(
            bad(&[(12, 0x82), (13, 0x11), (14, 0x70), (16, 0x82), (17, 0x11), (18, 0x70)]),
            Some(BodyShape::IndirectLoad {
                params: vec![0xEE09],
                ops: vec![
                    IlOp::Load(0xEE09),
                    IlOp::LoadIndSized { off: 0, width: 1, sext: false },
                ],
            })
        );
        // A `float` pointee is `lfs` (`30 86 45 40`).
        assert_eq!(bad(&[(13, 0x45), (14, 0x40), (17, 0x45), (18, 0x40)]), None);
        // A pointer pointee (`int **`) emits the same `lwz` and is now ACCEPTED
        // — rung 1 of `docs/IL_LOAD_TYPES.md` §6. It used to be refused (and was
        // recorded as such in `il_expr_load_neg.cpp`) purely because the type
        // gate said "4-byte integer"; the instruction was never the reason. The
        // `41` result has to move with the `30` load: the tail requires the two
        // to be in the same value class, so a half-spliced segment refuses.
        assert!(matches!(
            bad(&[(13, 0x43), (17, 0x43)]),
            Some(BodyShape::IndirectLoad { .. })
        ));
        assert_eq!(bad(&[(13, 0x43)]), None, "ptr load, int result: cross-class");
        // …and tag `C6` is refused even with both positions moved. `readers.rs`
        // records that bit 0x40 occurs and no probe produced it, so it is
        // undetermined, not a cv bit.
        assert_eq!(bad(&[(12, 0xC6), (13, 0x43), (16, 0xC6), (17, 0x43)]), None);

        // Arithmetic after the load: the load lands in the scratch register, so
        // this must not reach the affine selector.
        let mut with_add = IND_DEREF[..d + 15].to_vec();
        with_add.extend_from_slice(&[0x33, 0x86, 0x41, 0x74, 0x01, 0x02]); // + 1
        with_add.extend_from_slice(&IND_DEREF[d + 15..]);
        assert_eq!(parse_segment(&free_fn(&with_add), NO_LOCALS), None);

        // A `28` payload other than `00 00` is unexplained and must refuse.
        let n = base(IND_SUBSCRIPT_NEG);
        let mut sub_bad = IND_SUBSCRIPT_NEG.to_vec();
        sub_bad[n + 17] = 0x01;
        assert_eq!(parse_segment(&free_fn(&sub_bad), NO_LOCALS), None);

        // An offset past the 16-bit displacement materializes an index register.
        let mut wide = IND_SUBSCRIPT_NEG[..n + 15].to_vec();
        wide.extend_from_slice(&[0x80, 0x80, 0x1A, 0x06, 0x00]); // 400000
        wide.extend_from_slice(&IND_SUBSCRIPT_NEG[n + 16..]);
        assert_eq!(parse_segment(&free_fn(&wide), NO_LOCALS), None);
    }

    #[test]
    fn ptr_getter_leaf_decodes_as_the_same_indirect_load_an_int_getter_does() {
        // The whole point of the rung: the BodyShape is unchanged, so
        // `indirect_load_text` (which consumes no type) emits the same `lwz`.
        assert_eq!(
            parse_segment(PTR_GETTER, NO_LOCALS),
            Some(BodyShape::IndirectLoad {
                params: vec![0x100A],
                ops: vec![IlOp::Load(0x100A), IlOp::LoadInd { off: 0 }],
            })
        );
        assert_eq!(
            parse_segment(PTR_GETTER_CV, NO_LOCALS),
            Some(BodyShape::IndirectLoad {
                params: vec![0x200A],
                ops: vec![IlOp::Load(0x200A), IlOp::LoadInd { off: 0 }],
            }),
            "the A6-tagged load plus its 2C strip"
        );
    }

    #[test]
    fn ptr_identity_leaf_binds_this_and_a_formal_alike() {
        // `return this;` — the token comes from the pre-body `B9 … 99 … 00`
        // group, not from the `2D` formals list, and it is r3.
        assert_eq!(
            parse_segment(PTR_IDENT_THIS, NO_LOCALS),
            Some(BodyShape::StraightLine {
                params: vec![0x020A],
                ops: vec![IlOp::Load(0x020A)],
            })
        );
        // `return s;` — same production, token from the formals list.
        assert_eq!(
            parse_segment(PTR_IDENT_FORMAL, NO_LOCALS),
            Some(BodyShape::StraightLine {
                params: vec![0x290A],
                ops: vec![IlOp::Load(0x290A)],
            })
        );
    }

    #[test]
    fn ptr_leaves_refuse_the_shapes_that_cost_an_instruction() {
        assert_eq!(parse_segment(PTR_DEREF2, NO_LOCALS), None, "**ppp is two lwz");
        // …and it is reported by the census rather than silently, so the
        // instrument and the gate agree that it is out of class.
        assert!(parse_segment_detail(PTR_DEREF2, NO_LOCALS).is_err());

        // `return s;` from r4 is `mr r3,r4`, which W18 now emits. It comes out
        // as the identity `StraightLine` — the *same* one-op stream a first-
        // argument identity produces, with the register decided by the token's
        // position in `params` and nowhere else, which is what lets one arm in
        // `select_text` serve both. `w18_reg_move.cpp` grades the bytes.
        assert_eq!(
            parse_segment(PTR_IDENT_R4, NO_LOCALS),
            Some(BodyShape::StraightLine {
                params: vec![0x280A, 0x290A],
                ops: vec![IlOp::Load(0x290A)],
            }),
            "return s (r4) is one mr"
        );
    }

    #[test]
    fn a_cross_class_2c_over_a_pointer_load_refuses() {
        // `30 <ptr>` followed by `2C <int> 00` is a pointer→int reinterpret. It
        // may well be free, but it is unprobed, and the class-agreement rule is
        // what keeps an address-*adjusting* conversion from ever being admitted
        // as a free one. Splice the int target into the accepted cv-strip
        // getter, changing nothing else.
        let lo = find_subslice(PTR_GETTER_CV, &LO_MARKER).unwrap();
        let at = PTR_GETTER_CV[lo..]
            .windows(6)
            .position(|w| w == [0x2C, 0x86, 0x43, 0xF4, 0x08, 0x00])
            .expect("the 2C strip")
            + lo;
        let mut s = PTR_GETTER_CV[..at].to_vec();
        s.extend_from_slice(&[0x2C, 0x86, 0x41, 0x74, 0x00]); // -> int
        s.extend_from_slice(&PTR_GETTER_CV[at + 6..]);
        assert_eq!(parse_segment(&s, NO_LOCALS), None);
        // Control: the same splice back to the captured pointer target parses,
        // so the assertion above is about the class and not about the splice.
        let mut ok = PTR_GETTER_CV[..at].to_vec();
        ok.extend_from_slice(&[0x2C, 0x86, 0x43, 0xF4, 0x08, 0x00]);
        ok.extend_from_slice(&PTR_GETTER_CV[at + 6..]);
        assert!(parse_segment(&ok, NO_LOCALS).is_some());
    }

}
