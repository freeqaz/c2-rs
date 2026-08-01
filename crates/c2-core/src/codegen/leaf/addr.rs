//! The address leaf: `return &s->m;` as one `addi`. The second consumer of
//! the sub-object designator (`c2-il/src/func/body/shapes/designator.rs`).

use c2_il::{IlFunction, IlOp};
use crate::BackendError;
use crate::codegen::encode::{encode_addi, encode_blr, encode_mr};
use crate::codegen::select::{ARG_REGS, RET_REG, out_of_class};

/// Lower an **address leaf** — `return &s->m;` / `return &p->Base::m;` /
/// `return s->arr;` / `return &p->t[2];` — to one `addi` + `blr`, or to a bare
/// `blr` when the offset is zero.
///
/// Recognized by an **exact** two-op stream `[Load(base), AddrOf { off }]`, which
/// `c2_il::try_parse_addr_leaf` is the only producer of. Returns `None` for
/// anything else so the ordinary selector keeps its behaviour unchanged, and the
/// pattern is deliberately not a prefix match: an address that feeds arithmetic
/// is a construct with no capture behind it.
///
/// The measured lowering (`work/bma/probes/p1.cpp`, `p2.cpp`, `p3.cpp`, every
/// word read off the reference obj at `/Ox /GS- /c`):
///
/// ```text
///   int* f(S* s){ return &s->b; }         38630004  addi r3,r3,4
///   int* f(int x, S* s){ return &s->b; }  38640004  addi r3,r4,4
///   int* D::pb1(){ return &b1; }          3863000c  addi r3,r3,12   (2117, 8+4)
///   int* f(S* s){ return &s->a; }         —                          (off 0)
/// ```
///
/// The zero-offset case emits **nothing at all**, and is only reachable with the
/// base in r3: from any other argument register c2 emits `mr r3,rN`, and the
/// parser refuses that rather than have this function guess. The `Err` below is
/// the second lock on it, not the primary one.
///
/// `func.params` maps the base token to its incoming argument register by
/// declaration order, with a member function's `this` already at index 0.
pub fn addr_leaf_text(func: &IlFunction) -> Option<Result<Vec<u8>, BackendError>> {
    let (base_tok, off) = match func.ops.as_slice() {
        [IlOp::Load(t), IlOp::AddrOf { off }] => (*t, *off),
        _ => return None,
    };
    let d = match i16::try_from(off) {
        Ok(d) => d,
        // The parser gates this; if it ever changed, refuse rather than truncate —
        // a displacement over 32767 is `addis` + `addi`, two instructions.
        Err(_) => {
            return Some(Err(out_of_class(
                "sub-object address offset exceeds a 16-bit displacement",
            )))
        }
    };
    let base = match func.params.iter().position(|&t| t == base_tok) {
        Some(i) if i < ARG_REGS.len() => ARG_REGS[i],
        _ => {
            return Some(Err(out_of_class(
                "sub-object address whose base is not a register argument",
            )))
        }
    };
    let mut text = Vec::with_capacity(8);
    if d != 0 {
        text.extend_from_slice(&encode_addi(RET_REG, base, d));
    } else if base != RET_REG {
        // A zero-offset address from a non-first argument is the same one
        // register move `select_text` makes for `return b;` — `int* f(int k,
        // S* s){ return &s->a; }` is `mr r3,r4`, measured, the same word as the
        // pointer identity beside it. Two spellings, one instruction.
        text.extend_from_slice(&encode_mr(RET_REG, base));
    }
    text.extend_from_slice(&encode_blr());
    Some(Ok(text))
}

#[cfg(test)]
mod tests {
    // The single `mod tests` this was split out of opened with
    // `use super::*;`; the glob keeps that reach.
    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
    use crate::codegen::*;
    #[allow(unused_imports)]
    use c2_il::{IlFunction, IlOp};
    #[allow(unused_imports)]
    use crate::codegen::testutil::*;
    #[test]
    fn addr_leaf_text_is_one_addi_and_a_blr() {
        // Every expected word transcribed from the reference obj of
        // `fixtures/cpp/w16_addr_leaf.cpp` and `work/bma/probes/p{1,2,3}.cpp`,
        // not derived from the encoding rule.
        let mut f = IlFunction {
            mangled_name: "?a_off4@@YAPAHPAUS@@@Z".into(),
            source_path: None,
            params: vec![0xEE09],
            ops: vec![IlOp::Load(0xEE09), IlOp::AddrOf { off: 4 }],
            tail_call: None,
            framed_call: None,
        call_seq: None,
            compare: None,
            empty_body: false,
            float_leaf: None,
            fp_tail: None,
        fp_arg_sources: None,
            arg_sources: None,
            data_sym: None,
            eh_bare: false,
            eh_unwind_callees: Vec::new(),
        };
        assert_eq!(
            addr_leaf_text(&f).unwrap().unwrap(),
            vec![0x38, 0x63, 0x00, 0x04, 0x4E, 0x80, 0x00, 0x20],
            "addi r3,r3,4 ; blr"
        );
        // A ZERO offset emits the `blr` alone — the address is already in r3.
        // A lowering that emitted `addi r3,r3,0` would be one word too long, and
        // it is the case a nonzero-only test cannot see.
        f.ops = vec![IlOp::Load(0xEE09), IlOp::AddrOf { off: 0 }];
        assert_eq!(
            addr_leaf_text(&f).unwrap().unwrap(),
            vec![0x4E, 0x80, 0x00, 0x20],
            "a zero offset emits nothing at all"
        );
        // The base is the `addi`'s rA, not a hardcoded r3.
        f.params = vec![0x1111, 0xEE09];
        f.ops = vec![IlOp::Load(0xEE09), IlOp::AddrOf { off: 4 }];
        assert_eq!(
            addr_leaf_text(&f).unwrap().unwrap(),
            vec![0x38, 0x64, 0x00, 0x04, 0x4E, 0x80, 0x00, 0x20],
            "addi r3,r4,4 ; blr"
        );
        // …and at zero offset from that same non-first base c2 emits `mr r3,r4`
        // — measured, `int* f(int k, S* s){ return &s->a; }` is `7c832378`, the
        // same word as the pointer identity beside it. The one case a
        // zero-offset-from-r3 test cannot see is precisely this one: a bare
        // `blr` here would silently return `k` instead of the address.
        f.ops = vec![IlOp::Load(0xEE09), IlOp::AddrOf { off: 0 }];
        assert_eq!(
            addr_leaf_text(&f).unwrap().unwrap(),
            vec![0x7C, 0x83, 0x23, 0x78, 0x4E, 0x80, 0x00, 0x20],
            "mr r3,r4 ; blr"
        );
        // An offset past the signed 16-bit immediate is `addis` + `addi`.
        f.params = vec![0xEE09];
        f.ops = vec![IlOp::Load(0xEE09), IlOp::AddrOf { off: 32768 }];
        assert!(addr_leaf_text(&f).unwrap().is_err(), "32768 does not fit an addi");
        f.ops = vec![IlOp::Load(0xEE09), IlOp::AddrOf { off: 32764 }];
        assert_eq!(
            addr_leaf_text(&f).unwrap().unwrap(),
            vec![0x38, 0x63, 0x7F, 0xFC, 0x4E, 0x80, 0x00, 0x20],
            "32764 still fits"
        );
        // Anything that is not EXACTLY `[Load, AddrOf]` is not this shape.
        f.ops = vec![IlOp::Load(0xEE09), IlOp::AddrOf { off: 4 }, IlOp::Lit(1), IlOp::Add];
        assert!(addr_leaf_text(&f).is_none());
    }

}
