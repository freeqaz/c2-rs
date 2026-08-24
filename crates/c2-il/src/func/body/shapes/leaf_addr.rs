//! The **address leaf**: `return &s->m;` at both designators, which the
//! backend emits as one `addi`. Consumer of [`super::designator`].

use crate::func::body::expr::{eat_return_plumbing, BODY_SCOPE_DEPTH};
use crate::func::body::BodyShape;
use crate::func::readers::{eat_byte, is_ptr4_kind, is_volatile_tag, read_token_var, read_type};
use crate::func::IlOp;

use super::designator::{eat_addr_offset_adds, is_ptr_any, parse_base_member_designator};
use super::params::parse_params;

/// Try to parse an **address leaf**: a whole body that is one sub-object
/// *address* and nothing else — `return &s->m;`, `return &p->Base::m;`,
/// `return s->arr;`, `return &p->t[2];`.
///
/// ```text
///   <designator>                       the object pointer, one of two spellings
///   ( 33 <int-like> k 27 <PTR>         byte-offset adds, any number, summed
///   | 33 <int-like> k 28 00 00 )*
///   [ 2C <PTR> 00 ]                    an array-to-pointer decay / cv strip
///   41 <PTR>                           result type: a pointer
///   <return plumbing, reaching the segment end>
/// ```
///
/// where `<designator>` is either a plain pointer LOAD `B9 <tok> <PTR4>` (a
/// formal or `this`) or the intrinsic-2117 `base-member-addr` production
/// ([`parse_base_member_designator`]), whose two literals contribute their sum
/// to the offset before the adds are applied.
///
/// **This is one instruction and the same one either way**: `addi rD, rBase, K`,
/// with `K` the total. MEASURED at the fixture profile — every word below read
/// off the reference obj (`work/bma/probes/p1.cpp`, `p2.cpp`, `p3.cpp`):
///
/// ```text
///   int* f(S* s){ return &s->b; }          addi r3,r3,4      ; blr
///   int* f(int x, S* s){ return &s->b; }   addi r3,r4,4      ; blr   <- ANY base reg
///   int* D::pb1(){ return &b1; }           addi r3,r3,12     ; blr   <- 2117, 8+4
///   int* DR::pt2(){ return &t[2]; }        addi r3,r3,16     ; blr   <- two `28`s
///   int* f(S* s){ return s->arr; }         addi r3,r3,40     ; blr   <- the decay
///   int* f(S* s){ return &s->a; }                             blr    <- K = 0
///   char*/short*/…/double* members         the identical addi             (p2 `DW`)
/// ```
///
/// and **no `.pdata` entry**: the body is a leaf and c2 emits none.
///
/// Why each gate is load-bearing — every one is a *captured* neighbour that
/// emits something else:
///
/// * **`K` must fit a signed 16-bit displacement.** `&p->t` at 32764 is one
///   `addi`; at 32768 it is **`addis r3,r3,1 ; addi r3,r3,-32768`**, two
///   instructions this shape does not emit (`work/bma/probes/p3.cpp`).
/// * **`K == 0` requires the base to be the FIRST parameter.** The address is
///   then already in r3 and the body is a bare `blr` — but from any other
///   argument register c2 emits a real `mr r3,r4` (measured, `z_r4`/`i_z_r4`).
///   That is the same boundary [`straight_line_is_out_of_class`] draws for the
///   bare-parameter identity, and it is drawn here rather than assumed.
/// * **The result must be a POINTER.** With a `30` in front of the `41` the body
///   is a *load* and emits `lwz` — [`try_parse_indirect_load_leaf`]'s shape, one
///   token away from this one. This production is anchored on the `41`
///   immediately following the adds, so a load has no path into it.
/// * **A `2C` may only convert pointer→pointer.** An array-to-pointer decay and
///   a cv strip both emit nothing (measured: `r_d`, `a_arr0`); a cross-class
///   `2C` is a reinterpret this port has never probed.
/// * **The base must be a register argument** (`params` position < 8): past the
///   eighth it is stack-homed, which needs a frame.
///
/// Returns `None` — cursor untouched — for anything that is not exactly this
/// shape.
pub(crate) fn try_parse_addr_leaf(seg: &[u8], start: usize, lo: usize) -> Option<BodyShape> {
    let mut p = start;
    // The designator. The intrinsic form is anchored on a `33` literal and the
    // plain form on a `B9`, so the two cannot be confused; the intrinsic is tried
    // first for the same reason [`try_parse_indirect_load_leaf`] tries it first.
    let (mut off, base_tok) = match parse_base_member_designator(seg, p, is_ptr_any) {
        Some((off, tok, end)) => {
            p = end;
            (off, tok)
        }
        None => {
            if !eat_byte(seg, &mut p, 0xB9) {
                return None;
            }
            let (tok, w) = read_token_var(seg, p)?;
            p += w;
            let (tag, kind, _, tw) = read_type(seg, p)?;
            // A pointer *value* in a register: the `B9` operand position, where
            // the tag carries the pointer's own width.
            // …and NOT `volatile`. A volatile pointer formal is a memory
            // object: c2 homes it in the frame and reloads it, so this leaf is a
            // whole framed body. See `readers::is_volatile_tag` — the thirteenth
            // live wrong-bytes emit, and the position is load-bearing (the same
            // bit at the `27`/`30` designator positions is free).
            if !is_ptr4_kind(tag, kind) || is_volatile_tag(tag) {
                return None;
            }
            p += tw;
            (0, tok)
        }
    };
    off = off.checked_add(eat_addr_offset_adds(seg, &mut p)?)?;

    // An array-to-pointer decay or a cv strip, pointer→pointer only.
    if seg.get(p)? == &0x2C {
        let mut probe = p + 1;
        let (tag, kind, _, tw) = read_type(seg, probe)?;
        if !is_ptr_any(tag, kind) {
            return None;
        }
        probe += tw;
        if !eat_byte(seg, &mut probe, 0x00) {
            return None;
        }
        p = probe;
    }

    // The result type — a pointer, which is what separates this from every
    // arithmetic leaf.
    if !eat_byte(seg, &mut p, 0x41) {
        return None;
    }
    let (tag, kind, _, tw) = read_type(seg, p)?;
    if !is_ptr_any(tag, kind) {
        return None;
    }
    p += tw;
    eat_return_plumbing(seg, &mut p, false, BODY_SCOPE_DEPTH).ok()?;

    // The displacement bound, checked once for both designators.
    if !(-0x8000..=0x7FFF).contains(&off) {
        return None;
    }
    let params = parse_params(seg, lo).ok()?;
    let ix = params.iter().position(|&t| t == base_tok)?;
    // Past the eighth argument the value is stack-homed, which needs a frame.
    if ix >= 8 {
        return None;
    }
    // A zero offset emits nothing when the address is already in r3, and one
    // `mr r3,rN` when it is not — the same register move the arithmetic identity
    // makes, which is why this refusal is gone rather than duplicated. MEASURED:
    // `int* f(int k, S* s){ return &s->a; }` is `7c832378` (`mr r3,r4`) and
    // `S* f(int k, const S* s){ return (S*)s; }` is the same word, against
    // `38640004` (`addi r3,r4,4`) for the nonzero-offset neighbour.
    Some(BodyShape::AddrLeaf {
        params,
        ops: vec![IlOp::Load(base_tok), IlOp::AddrOf { off }],
    })
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
    fn the_offset_add_without_a_load_is_an_address_and_emits_an_addi() {
        // `&s->b` is the getter minus the `30`, and it is the case that decides
        // whether the *identity* recognizer may skip an optional offset add: it
        // may not, because this emits `addi r3,r3,4` where the identity emits
        // nothing. It used to be a pure refusal (`w12_ptr_leaf_neg.cpp`'s
        // `n_addr_of`); it is now its own production with its own lowering, and
        // the discrimination that mattered is unchanged — it must NOT come out
        // as a `StraightLine` identity.
        assert_eq!(
            parse_segment(PTR_ADDR_OF, NO_LOCALS),
            Some(BodyShape::AddrLeaf {
                params: vec![0x160A],
                ops: vec![IlOp::Load(0x160A), IlOp::AddrOf { off: 4 }],
            }),
            "&s->b is an address leaf at offset 4, not an identity"
        );
        // The neighbour one token along in the other direction: with the `30`
        // back it is a LOAD, and the two must not be interchangeable.
        assert!(
            matches!(parse_segment(PTR_GETTER, NO_LOCALS), Some(BodyShape::IndirectLoad { .. })),
            "a `30` in front of the `41` is still a load"
        );
    }

    #[test]
    fn an_address_leaf_refuses_what_it_cannot_emit_in_one_addi() {
        // Every case below is `PTR_ADDR_OF` with ONE field changed, so each
        // isolates a single gate. The shared prefix ends at the offset literal
        // `04` at index 76 and the `27` type that follows it.
        let base = PTR_ADDR_OF.to_vec();
        assert_eq!(base[79], 0x04, "the offset literal moved");
        assert_eq!(&base[80..85], &[0x27, 0x86, 0x43, 0x84, 0x20], "the `27` add moved");

        // A zero offset emits NOTHING, which is only correct because the address
        // is already in r3 — and here the base IS the first formal, so it is
        // accepted and its op stream records the zero.
        let mut zero = base.clone();
        zero[79] = 0x00;
        assert_eq!(
            parse_segment(&zero, NO_LOCALS),
            Some(BodyShape::AddrLeaf {
                params: vec![0x160A],
                ops: vec![IlOp::Load(0x160A), IlOp::AddrOf { off: 0 }],
            })
        );

        // The `27` re-type must be a POINTER. An int-typed add here would be
        // integer arithmetic on a pointer, which c2 scales.
        //
        // **This cell is the fence Phase 1 slice C1 kept**, and it is the one
        // that matters: `expr::parse_expr`'s promoted `0x27` arm gates on the
        // same [`super::designator::is_ptr_any`] whitelist this leaf does, so
        // the body is refused by *both* productions rather than falling through
        // the leaf into a general walk that would have taken it.
        let mut nonptr = base.clone();
        nonptr[81] = 0x86;
        nonptr[82] = 0x41;
        nonptr[83] = 0x74;
        assert_eq!(parse_segment(&nonptr, NO_LOCALS), None, "a non-pointer `27`");

        // **The `41` int-result cell CHANGED at C1 (lane `w-c1`), and this is
        // board #403's other predicted red — the one that is a finding.**
        //
        // This used to assert `None` with the comment *"an int result means the
        // address was converted, and that conversion is unprobed"*, and #403
        // recorded that the axis had never been probed: 11 probes over 5 axes,
        // none of them the result type. **It is probed now.** MEASURED,
        // `fixtures/cpp/wc1_offadd.cpp` (and `work/w-c1/probes/p2.cpp` before
        // it), `/Ox /GS- /c`, byte-judged against real c2:
        //
        // ```text
        //   int addr_cast(T* t) { return (int)&t->s.b; }   38630008 4e800020
        //   int addr_one (T* t) { return (int)&t->y;   }   38630010 4e800020
        //   int addr_zero(T* t) { return (int)&t->x;   }            4e800020
        // ```
        //
        // — the same one `addi` the pointer-returning form emits, because the
        // conversion is a width-4 ptr4→int4 reinterpret, which c2 pays nothing
        // for (board **#700**, graded over 31 cells and the whole 3×3 of
        // [`crate::func::readers::ValueClass`] pairs). So the refusal was
        // conservatism, not a rule, and the `Port=Match` on those bodies is what
        // retires it.
        //
        // **The ADDRESS LEAF still refuses it** — that is this test's subject and
        // it is asserted below. What changed is which production takes the body:
        // the general expression walk now folds the designator run into
        // `[Load, Lit(4), Add]`, one `addi` after `codegen::fold`.
        assert_eq!(&base[85..90], &[0x41, 0x86, 0x43, 0xF4, 0x08], "the `41` moved");
        let mut intres = base.clone();
        intres.splice(85..90, [0x41, 0x86, 0x41, 0x74].iter().copied());
        assert_eq!(
            parse_segment(&intres, NO_LOCALS),
            Some(BodyShape::StraightLine {
                params: vec![0x160A],
                ops: vec![IlOp::Load(0x160A), IlOp::Lit(4), IlOp::Add],
            }),
            "an int result type is a straight-line address computation, NOT an address leaf"
        );
        assert!(
            !matches!(parse_segment(&intres, NO_LOCALS), Some(BodyShape::AddrLeaf { .. })),
            "the ADDRESS LEAF still refuses it — this test's subject is unchanged"
        );
        // And with the C1 decision point turned off the pre-C1 parser is exactly
        // restored. Not asserted here — `off_add_admitted` reads its environment
        // through a process-wide `OnceLock`, so a test cannot flip it — see
        // `docs/rungs/2026-08-24-w-c1.md` §"the counterfactual".
    }

}
