//! The **store leaf**: `s->m = v;`, integer and floating-point.
//! Third consumer of [`super::designator`]; see `docs/IL_STORE_LEAF.md`.

use crate::func::body::expr::{eat_return_plumbing, parse_formals, BODY_SCOPE_DEPTH};
use crate::func::body::BodyShape;
use crate::func::readers::{
    eat_byte, eat_opt_stmt_marker, eat_value_type, is_ptr4_kind, is_volatile_tag, read_token_var,
    read_type,
    read_varint, value_class,
};
use crate::func::sy::{fp_reg_of, ArgClass, SyView};
use crate::func::IlOp;

use super::designator::{
    eat_addr_offset_adds, is_ptr_any, parse_base_member_designator, store_fp_value_width,
    store_value_width,
};
use super::params::parse_params;

/// Try to parse a **store leaf**: a whole body that is one store into a
/// sub-object and nothing else — `void f(S* s, int v){ s->m = v; }`,
/// `void D::set(int v){ Base::m = v; }`, `void f(S* s, int v){ s->arr[2] = v; }`,
/// `void f(int* p, int v){ *p = v; }`, `void f(S* s){ s->m = 7; }`.
///
/// ```text
///   <designator>                       the object pointer, the same two spellings
///   ( 33 <int-like> k 27 <PTR>         byte-offset adds, any number, summed
///   | 33 <int-like> k 28 00 00 )*
///   [ 2C <PTR> 00 ]                    a cv strip / array-to-pointer decay
///   ( B9 <tok> <VT> | 33 <VT> <k> )    THE VALUE: a formal, or an integer literal
///   32 <VT>                            the store; its TYPE restates the value's
///   4B                                 statement end — and the body ends here
///   <return plumbing, void, reaching the segment end>
/// ```
///
/// where `<designator>` is either a plain pointer LOAD `B9 <tok> <PTR4>` or the
/// intrinsic-2117 `base-member-addr` production ([`parse_base_member_designator`]),
/// whose two literals contribute their sum to the offset before the adds — the
/// same pair of spellings [`try_parse_addr_leaf`] and
/// [`try_parse_indirect_load_leaf`] accept, reached through the same decoder.
///
/// **This is one store instruction, and the width picks it.** MEASURED at the
/// fixture profile — every word below read off the reference obj
/// (`work/lf/probes/p1.cpp`):
///
/// ```text
///   void s_a (S* s, int v)       { s->a  = v; }   90830000  stw  r4,0(r3)
///   void s_b (S* s, int v)       { s->b  = v; }   90830004  stw  r4,4(r3)
///   void s_p (S* s, void* v)     { s->p  = v; }   90830008  stw  r4,8(r3)
///   void s_c (S* s, char v)      { s->c  = v; }   9883000c  stb  r4,12(r3)
///   void s_sh(S* s, short v)     { s->s  = v; }   b083000e  sth  r4,14(r3)
///   void s_q (S* s, long long v) { s->q  = v; }   f8830020  std  r4,32(r3)
///   void s_e2(S* s, int v)       { s->arr[2] = v; } 90830030  stw  r4,48(r3)
///   void s_k (S* s)              { s->a  = 7; }   39600007 91630000  li r11,7 ; stw r11,0(r3)
///   void s_arg2(int x,S* s,int v){ s->b  = v; }   90a40004  stw  r5,4(r4)  <- ANY two regs
///   void D::sb1(int v)           { b1 = v; }      90830004  stw  r4,4(r3)  <- 2117, 0+4
/// ```
///
/// and **no `.pdata` entry**: the body is a leaf, exactly like the load and
/// address leaves beside it.
///
/// Why each gate is load-bearing — every one is a *captured* neighbour that
/// emits something else:
///
/// * **The value must be a GPR-class scalar** ([`store_value_width`]). A `float`
///   or `double` member is `stfs`/`stfd` from the FP file and the FP argument
///   number is not the parameter index.
/// * **No conversion on the value.** `void M::setb(bool v){ m0 = v; }` (an `int`
///   member, a `bool` parameter) carries a `2C 86 41 74 00` and emits
///   `548b063e ; 91630000` — `clrlwi r11,r4,24 ; stw r11,0(r3)` — a real mask
///   through the scratch register. The production admits a `2C` only on the
///   *address*, pointer→pointer, where it is free.
/// * **The stored TYPE must restate the value's `<tag><kind>`.** They are
///   byte-identical at every captured site, and requiring it is what makes a
///   misaligned read fail closed instead of picking a plausible width.
/// * **`K` must fit a signed 16-bit displacement**, and a `width == 8` store's
///   `K` must be a multiple of 4 (`std` is DS-form and cannot encode the low two
///   bits) — the same two bounds the load leaf draws.
/// * **Both the base and the value must be register arguments** (`params`
///   position < 8): past the eighth they are stack-homed, which needs a frame.
///
/// Returns `None` — cursor untouched — for anything that is not exactly this
/// shape.
pub(crate) fn try_parse_store_leaf(
    seg: &[u8],
    start: usize,
    lo: usize,
    sy: SyView,
) -> Option<BodyShape> {
    let mut p = start;
    // The designator. The intrinsic form is anchored on a `33` literal and the
    // plain form on a `B9`, so the two cannot be confused; the intrinsic is tried
    // first for the same reason the load and address leaves try it first.
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

    // A cv strip or an array-to-pointer decay applied to the ADDRESS, which emits
    // nothing (`void f(S* s, int v){ *(int*)s = v; }` is a bare `stw r4,0(r3)`).
    // Pointer→pointer only: a cross-class `2C` here is a reinterpret this port has
    // never probed.
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

    // THE VALUE — a bare formal or an integer literal, and nothing computed. A
    // computed value lands in the scratch register first (`s->m = a + b` is
    // `add r11,r3,r4 ; stw r11`), which is a different instruction count and has
    // no capture behind it here.
    let (value_op, mut value_tag, mut value_kind) = match *seg.get(p)? {
        0xB9 => {
            let mut probe = p + 1;
            let (tok, w) = read_token_var(seg, probe)?;
            probe += w;
            let (tag, kind, _, tw) = read_type(seg, probe)?;
            probe += tw;
            p = probe;
            (IlOp::Load(tok), tag, kind)
        }
        0x33 => {
            let mut probe = p + 1;
            let (tag, kind, _, tw) = read_type(seg, probe)?;
            probe += tw;
            let k = read_varint(seg, &mut probe)?;
            p = probe;
            (IlOp::Lit(k), tag, kind)
        }
        _ => return None,
    };
    // A **floating-point** stored value is `stfs`/`stfd` out of the FP argument
    // file, so it takes the whole rest of this production down a parallel path:
    // its register is not the formal's index, and the `2C` rules below are the
    // GPR classes'. MEASURED (`docs/CODEGEN_FP_ARGS.md` §3):
    //
    //     void s_f (S* s, float v)      { s->f = v; }        d0230004  stfs f1,4(r3)
    //     void s_d (S* s, double v)     { s->d = v; }        d8230008  stfd f1,8(r3)
    //     void s_two(S* s,float u,float v){ s->f = v; }      d0430004  stfs f2,4(r3)
    //
    // Sized before it was built, by counterfactual over the 878-TU workload:
    // **7,984 functions**, all `calls-0`.
    let fp_width = store_fp_value_width(value_tag, value_kind);
    if let Some(w) = fp_width {
        return finish_fp_store_leaf(seg, p, lo, base_tok, value_op, value_tag, value_kind, off, w, sy);
    }
    let width = store_value_width(value_tag, value_kind)?;

    // A class-preserving conversion of the VALUE — `void f(S* s, S* v){ s->p = v; }`
    // converts `S*` to `void*` on the way in and emits nothing (`90830008`, the same
    // bare `stw` as the unconverted neighbour). Admitted only in the two 4-byte
    // classes [`eat_value_type`] was byte-graded on since the getter rungs, and
    // **only** there: over a narrow value a `2C` is a real instruction —
    // `void M::setb(bool v){ m0 = v; }` (an `int` member, a `bool` parameter) emits
    // `clrlwi r11,r4,24 ; stw r11,0(r3)` — so `width != 4` refuses rather than
    // silently dropping the mask.
    if seg.get(p) == Some(&0x2C) {
        let cls = value_class(value_tag, value_kind)?;
        let mut probe = p + 1;
        let (t2, k2, _, _) = read_type(seg, probe)?;
        if !eat_value_type(seg, &mut probe, cls) || !eat_byte(seg, &mut probe, 0x00) {
            return None;
        }
        value_tag = t2;
        value_kind = k2;
        p = probe;
    }

    // The store, whose TYPE restates the value's.
    if !eat_byte(seg, &mut p, 0x32) {
        return None;
    }
    let (tag, kind, _, tw) = read_type(seg, p)?;
    if (tag, kind) != (value_tag, value_kind) {
        return None;
    }
    p += tw;
    // The statement end. A store yields its value and `4B` discards it; a body
    // that goes on to use it is not this shape.
    if !eat_byte(seg, &mut p, 0x4B) {
        return None;
    }
    eat_opt_stmt_marker(seg, &mut p);
    eat_return_plumbing(seg, &mut p, false, BODY_SCOPE_DEPTH).ok()?;

    if !(-0x8000..=0x7FFF).contains(&off) {
        return None;
    }
    // `std` is DS-form: the displacement's low two bits are the form's, so an
    // offset that is not a multiple of 4 cannot be encoded at all. Natural
    // alignment makes one unreachable through a struct member, so this gate has
    // no witness — which is exactly why it refuses instead of masking.
    if width == 8 && off % 4 != 0 {
        return None;
    }
    let params = parse_params(seg, lo).ok()?;
    let bix = params.iter().position(|&t| t == base_tok)?;
    // Past the eighth argument the value is stack-homed, which needs a frame.
    if bix >= 8 {
        return None;
    }
    match value_op {
        IlOp::Load(vtok) => {
            let vix = params.iter().position(|&t| t == vtok)?;
            // Past the eighth argument the value is stack-homed, which needs a frame.
            if vix >= 8 {
                return None;
            }
        }
        // A wide **negative** constant. `emit_load_imm`'s `lis`+`ori` pair covers
        // non-negative values only, and the straight-line class already refuses
        // this in the PARSER (`expr-out-of-class-wide-neg-lit`,
        // `chain::straight_line_out_of_class_ctx`). Restating the bound here rather
        // than letting codegen refuse it is the census/gate invariant: the same
        // literal reached two shapes and only one of them gated it, so
        // `void f(S* s){ s->a = -70000; }` censused in class while `PortC2`
        // returned `NotImplemented` — the `GAPS.md` §6 "one fact, two locators"
        // failure, caught by probing the new production's own boundary.
        IlOp::Lit(k) if k < -0x8000 => return None,
        _ => {}
    }
    Some(BodyShape::StoreLeaf {
        params,
        ops: vec![IlOp::Load(base_tok), value_op, IlOp::StoreInd { off, width }],
    })
}

/// The tail of [`try_parse_store_leaf`] for a **floating-point** stored value.
///
/// Split out rather than branched inline because almost every gate differs: the
/// value's register comes from the FP file, the conversion rules are the FP ones,
/// and a literal is a pooled `.rdata` COMDAT rather than an `li`.
///
/// What is REFUSED here, each because a capture shows it emits something else:
///
/// * **A conversion on the value.** `void s_narrow(S* s, double v){ s->f = v; }`
///   is `frsp f0,f1 ; stfs f0,4(r3)` — a real instruction through the FP scratch
///   register. Its free twin `void s_widen(S* s, float v){ s->d = v; }` is a bare
///   `stfd f1,8(r3)`, so the asymmetry is c2's own and not the C standard's; both
///   are refused, because admitting the free one means deciding the direction from
///   two type triples and only the narrowing one has been captured at more than
///   one offset. A rung, sized in `docs/CODEGEN_FP_ARGS.md` §5.
/// * **A literal value.** `void s_lit(S* s){ s->f = 1.5f; }` is
///   `lis r11 ; lfs f0,0(r11) ; stfs f0,4(r3)` with a REFHI/REFLO pair into an
///   `.rdata` COMDAT — the W13b constant machinery, which `codegen::function_gate`
///   refuses under `/Gy` anyway.
/// * **A value that is not a formal**, and a formal whose FP register the `.sy`
///   argument classes cannot determine.
#[allow(clippy::too_many_arguments)]
fn finish_fp_store_leaf(
    seg: &[u8],
    mut p: usize,
    lo: usize,
    base_tok: u32,
    value_op: IlOp,
    value_tag: u8,
    value_kind: u8,
    off: i32,
    width: u8,
    sy: SyView,
) -> Option<BodyShape> {
    // No conversion, and no pooled constant.
    if seg.get(p) == Some(&0x2C) {
        return None;
    }
    let IlOp::Load(vtok) = value_op else {
        return None;
    };
    // The store, whose TYPE restates the value's — the same literal requirement
    // the GPR path makes, and for the same reason.
    if !eat_byte(seg, &mut p, 0x32) {
        return None;
    }
    let (tag, kind, _, tw) = read_type(seg, p)?;
    if (tag, kind) != (value_tag, value_kind) {
        return None;
    }
    p += tw;
    if !eat_byte(seg, &mut p, 0x4B) {
        return None;
    }
    eat_opt_stmt_marker(seg, &mut p);
    eat_return_plumbing(seg, &mut p, false, BODY_SCOPE_DEPTH).ok()?;

    if !(-0x8000..=0x7FFF).contains(&off) {
        return None;
    }
    // `stfs`/`stfd` are both plain D-form — unlike `std`, which is DS-form and
    // cannot encode a displacement that is not a multiple of 4. So there is no
    // alignment gate here, and the absence is a measured difference between the
    // two paths rather than an omission (`d8230008` is `stfd f1,8(r3)`, primary
    // 54, with all sixteen displacement bits its own).
    let params = parse_params(seg, lo).ok()?;
    let bix = params.iter().position(|&t| t == base_tok)?;
    if bix >= 8 {
        return None;
    }
    // The value's FP register, resolved HERE — the one site that knows both the
    // formals order (`.ex`) and each formal's register file (`.sy`).
    let formals = parse_formals(seg, lo).ok()?;
    let classes = sy.arg_classes(&formals).ok()?;
    let fix = formals.iter().position(|&t| t == vtok)?;
    let src = fp_reg_of(&classes, fix)?;
    if src > 13 {
        // Past f13 the argument is stack-homed, which needs a frame.
        return None;
    }
    // The value's declared width and the stored width must be the same fact. They
    // are, at every capture, because a conversion is a visible `2C` that is
    // refused above — so a disagreement means a misread type, not a construct.
    if matches!(classes.get(fix), Some(ArgClass::Fp { double }) if *double != (width == 8)) {
        return None;
    }
    Some(BodyShape::StoreLeaf {
        params,
        ops: vec![
            IlOp::Load(base_tok),
            IlOp::StoreIndFp { off, double: width == 8, src },
        ],
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
    /// W25: the store leaf, from whole captured segments — both designators, the
    /// widths that pick the opcode, the literal value, and the FP refusal.
    #[test]
    fn store_leaf_decodes_both_designators_and_refuses_a_float_value() {
        assert_eq!(
            parse_segment(STORE_MEMBER, NO_LOCALS),
            Some(BodyShape::StoreLeaf {
                params: vec![0xF909, 0xFA09],
                ops: vec![
                    IlOp::Load(0xF909),
                    IlOp::Load(0xFA09),
                    IlOp::StoreInd { off: 4, width: 4 },
                ],
            })
        );
        // The width comes from the STORED type, not from the designator's pointer
        // tag — the two agree for an `int` member and this is where they part.
        assert_eq!(
            parse_segment(STORE_NARROW, NO_LOCALS),
            Some(BodyShape::StoreLeaf {
                params: vec![0x010A, 0x020A],
                ops: vec![
                    IlOp::Load(0x010A),
                    IlOp::Load(0x020A),
                    IlOp::StoreInd { off: 12, width: 1 },
                ],
            })
        );
        assert_eq!(
            parse_segment(STORE_LIT, NO_LOCALS),
            Some(BodyShape::StoreLeaf {
                params: vec![0x210A],
                ops: vec![
                    IlOp::Load(0x210A),
                    IlOp::Lit(7),
                    IlOp::StoreInd { off: 0, width: 4 },
                ],
            })
        );
        // The intrinsic-2117 designator reaches the same address by a different
        // route and must produce the byte-identical op stream.
        assert_eq!(
            parse_segment(STORE_BASE_MEMBER, NO_LOCALS),
            Some(BodyShape::StoreLeaf {
                params: vec![0x610A, 0x620A],
                ops: vec![
                    IlOp::Load(0x610A),
                    IlOp::Load(0x620A),
                    IlOp::StoreInd { off: 4, width: 4 },
                ],
            })
        );
        // …and the neighbour that emits `stfs f1` must refuse, in the parser.
        assert_eq!(parse_segment(STORE_FLOAT_NEG, NO_LOCALS), None);
        assert_eq!(
            parse_segment_detail(STORE_FLOAT_NEG, NO_LOCALS)
                .unwrap_err()
                .feature(),
            "expr-op-0x27"
        );
    }

}
