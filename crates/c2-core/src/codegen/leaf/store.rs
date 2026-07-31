//! The store leaf: `s->m = v;` as one `stb`/`sth`/`stw`/`std` at a folded
//! displacement. The third consumer of the sub-object designator
//! (`c2-il/src/func/body/shapes/designator.rs`); see `docs/IL_STORE_LEAF.md`.

use c2_il::{IlFunction, IlOp};
use crate::BackendError;
use crate::codegen::encode::{
    encode_blr,
    encode_stb,
    encode_std,
    encode_stfs,
    encode_sth,
    encode_stw,
};
use crate::codegen::select::{ARG_REGS, SCRATCH_REG, out_of_class};
use crate::codegen::straightline::emit_load_imm;

// `encode_std` has TWO independent witnesses and exactly one definition, in
// [`crate::codegen::encode`] with every other word encoder: the frame model
// captured it as the callee-saved GPR prologue store (`fbe1fff0` =
// `std r31,-16(r1)`), and this rung captured it as a `long long` member at
// offset 32 (`f8830020`, in `store_leaf_text`'s table below). One function with
// two independent captures beats two functions with one each — and after the
// §2.1 split a second copy would be a compile error in `encode.rs` rather than
// a duplicate 2,000 lines away, which is what happened to `encode_std` once
// already (`docs/ARCHITECTURE_SEAMS.md` §1, class 4).

/// Lower a **store leaf** — `void f(S* s, int v){ s->m = v; }` /
/// `void D::set(int v){ Base::m = v; }` / `void f(S* s){ s->m = 7; }` — to one
/// store instruction + `blr`, or to `li` + store + `blr` when the value is a
/// literal.
///
/// Recognized by an **exact** three-op stream `[Load(base), Load(value) | Lit(k),
/// StoreInd { off, width }]`, which `c2_il::try_parse_store_leaf` is the only
/// producer of. Returns `None` for anything else so the ordinary selector keeps
/// its behaviour unchanged, and the pattern is deliberately not a prefix match:
/// a store whose value is *computed* puts the computation in the scratch
/// register first (`s->m = a + b` is `add r11,r3,r4 ; stw r11,0(r3)`), which is
/// a different shape with no capture behind it here.
///
/// The measured lowering (`work/lf/probes/p1.cpp`, `p3.cpp`, every word read off
/// the reference obj at `/Ox /GS- /c`):
///
/// ```text
///   width 1  stb    s->c = v   (char, off 12)      9883000c
///   width 2  sth    s->s = v   (short, off 14)     b083000e
///   width 4  stw    s->a = v   (int, off 0)        90830000
///   width 8  std    s->q = v   (long long, off 32) f8830020   DS-form
///   literal         s->a = 7                       39600007 91630000   li r11,7 ; stw r11
///   literal         s->f = true  (bool)            39600001 99630000   li r11,1 ; stb r11
///   two regs        f(int x,S* s,int v){s->b=v;}   90a40004            stw r5,4(r4)
/// ```
///
/// **The literal goes through the scratch register r11, never r3.** That is the
/// same r11 rule [`indirect_load_text`] follows for a load feeding an extension,
/// and it is read off the capture rather than assumed — a `void` function's r3
/// holds nothing the ABI cares about, so `li r3,7` would have been just as
/// plausible and is not what c2 emits.
///
/// `func.params` maps both tokens to their incoming argument registers by
/// declaration order, with a member function's `this` already at index 0.
pub fn store_leaf_text(func: &IlFunction) -> Option<Result<Vec<u8>, BackendError>> {
    // The **floating-point** store, `void f(S* s, float v){ s->f = v; }` — one
    // `stfs`/`stfd` + `blr`. Two ops rather than three: the value's register is
    // already resolved, because the FP argument file is numbered over the FP
    // parameters alone and only the IL layer has the `.sy` view that says which
    // parameters those are ([`c2_il::IlOp::StoreIndFp`]). The base is the ordinary
    // GPR argument, and its index *is* its register number even with FP formals in
    // the list — an FP parameter fills no GPR but still consumes its slot, so the
    // two effects cancel exactly (`docs/ABI_EDGES.md` §2, and the capture
    // `void s_arg2(int x, S* s, float v){ s->f = v; }` → `stfs f1,4(r4)`).
    if let [IlOp::Load(b), IlOp::StoreIndFp { off, double, src }] = func.ops.as_slice() {
        let d = match i16::try_from(*off) {
            Ok(d) => d,
            Err(_) => {
                return Some(Err(out_of_class(
                    "FP store offset exceeds a 16-bit displacement",
                )))
            }
        };
        let Some(base) = func
            .params
            .iter()
            .position(|&t| t == *b)
            .filter(|&i| i < ARG_REGS.len())
            .map(|i| ARG_REGS[i])
        else {
            return Some(Err(out_of_class(
                "FP store whose base is not a register argument",
            )));
        };
        let mut text = Vec::with_capacity(8);
        text.extend_from_slice(&encode_stfs(*double, *src, base, d));
        text.extend_from_slice(&encode_blr());
        return Some(Ok(text));
    }
    let (base_tok, value, off, width) = match func.ops.as_slice() {
        [IlOp::Load(b), v @ (IlOp::Load(_) | IlOp::Lit(_)), IlOp::StoreInd { off, width }] => {
            (*b, v, *off, *width)
        }
        _ => return None,
    };
    let d = match i16::try_from(off) {
        Ok(d) => d,
        // The parser gates this; if it ever changed, refuse rather than truncate.
        Err(_) => {
            return Some(Err(out_of_class(
                "store offset exceeds a 16-bit displacement",
            )))
        }
    };
    let reg_of = |tok: u32| -> Option<u8> {
        func.params
            .iter()
            .position(|&t| t == tok)
            .filter(|&i| i < ARG_REGS.len())
            .map(|i| ARG_REGS[i])
    };
    let Some(base) = reg_of(base_tok) else {
        return Some(Err(out_of_class(
            "store whose base is not a register argument",
        )));
    };
    let mut text = Vec::with_capacity(12);
    let src = match value {
        IlOp::Load(t) => match reg_of(*t) {
            Some(r) => r,
            None => {
                return Some(Err(out_of_class(
                    "store whose value is not a register argument",
                )))
            }
        },
        IlOp::Lit(k) => {
            if let Err(e) = emit_load_imm(&mut text, SCRATCH_REG, *k) {
                return Some(Err(e));
            }
            SCRATCH_REG
        }
        _ => return None,
    };
    match width {
        1 => text.extend_from_slice(&encode_stb(src, base, d)),
        2 => text.extend_from_slice(&encode_sth(src, base, d)),
        4 => text.extend_from_slice(&encode_stw(src, base, d)),
        8 if d % 4 == 0 => text.extend_from_slice(&encode_std(src, base, d)),
        8 => {
            return Some(Err(out_of_class(
                "8-byte store whose offset is not a multiple of 4 (std is DS-form)",
            )))
        }
        _ => return Some(Err(out_of_class("store of an unmodeled width"))),
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
    fn store_leaf_text_is_one_store_and_a_blr() {
        // Every expected word transcribed from the reference obj of
        // `fixtures/cpp/w25_store_leaf.cpp` and `work/lf/probes/p1.cpp`, not
        // derived from the encoding rule.
        let mut f = IlFunction {
            mangled_name: "?s_b@@YAXPAUS@@H@Z".into(),
            source_path: None,
            params: vec![0xF509, 0xF609],
            ops: vec![
                IlOp::Load(0xF509),
                IlOp::Load(0xF609),
                IlOp::StoreInd { off: 4, width: 4 },
            ],
            tail_call: None,
            framed_call: None,
            call_seq: None,
            compare: None,
            empty_body: false,
            float_leaf: None,
            fp_tail: None,
        fp_arg_sources: None,
            arg_sources: None,
        };
        assert_eq!(
            store_leaf_text(&f).unwrap().unwrap(),
            vec![0x90, 0x83, 0x00, 0x04, 0x4E, 0x80, 0x00, 0x20],
            "stw r4,4(r3) ; blr"
        );
        // A ZERO displacement is NOT free here — the store still happens. This is
        // the exact opposite of `addr_leaf_text`, whose zero case emits nothing,
        // and the two shapes share a designator.
        f.ops[2] = IlOp::StoreInd { off: 0, width: 4 };
        assert_eq!(
            store_leaf_text(&f).unwrap().unwrap(),
            vec![0x90, 0x83, 0x00, 0x00, 0x4E, 0x80, 0x00, 0x20],
            "stw r4,0(r3) ; blr"
        );
        // The width picks the opcode, and nothing else does.
        f.ops[2] = IlOp::StoreInd { off: 12, width: 1 };
        assert_eq!(
            store_leaf_text(&f).unwrap().unwrap()[..4],
            [0x98, 0x83, 0x00, 0x0C],
            "stb r4,12(r3)"
        );
        f.ops[2] = IlOp::StoreInd { off: 14, width: 2 };
        assert_eq!(
            store_leaf_text(&f).unwrap().unwrap()[..4],
            [0xB0, 0x83, 0x00, 0x0E],
            "sth r4,14(r3)"
        );
        f.ops[2] = IlOp::StoreInd { off: 32, width: 8 };
        assert_eq!(
            store_leaf_text(&f).unwrap().unwrap()[..4],
            [0xF8, 0x83, 0x00, 0x20],
            "std r4,32(r3)"
        );
        // `std` is DS-form: an offset that is not a multiple of 4 cannot be
        // encoded at all, so it refuses rather than dropping the low two bits.
        f.ops[2] = IlOp::StoreInd { off: 30, width: 8 };
        assert!(store_leaf_text(&f).unwrap().is_err());
        // BOTH register fields move: `void f(int x, S* s, int v){ s->b = v; }` is
        // `90a40004` — value r5, base r4 — and a lowering that hardcoded either
        // would pass every two-parameter case.
        f.params = vec![0x1111, 0xF509, 0xF609];
        f.ops[2] = IlOp::StoreInd { off: 4, width: 4 };
        assert_eq!(
            store_leaf_text(&f).unwrap().unwrap(),
            vec![0x90, 0xA4, 0x00, 0x04, 0x4E, 0x80, 0x00, 0x20],
            "stw r5,4(r4) ; blr"
        );
        // A literal value goes through the SCRATCH register, never r3: measured
        // `39600007 91630000` for `void f(S* s){ s->a = 7; }`.
        f.params = vec![0xF509];
        f.ops = vec![
            IlOp::Load(0xF509),
            IlOp::Lit(7),
            IlOp::StoreInd { off: 0, width: 4 },
        ];
        assert_eq!(
            store_leaf_text(&f).unwrap().unwrap(),
            vec![
                0x39, 0x60, 0x00, 0x07, 0x91, 0x63, 0x00, 0x00, 0x4E, 0x80, 0x00, 0x20
            ],
            "li r11,7 ; stw r11,0(r3) ; blr"
        );
        // …and a wide literal is the `lis`+`ori` pair through the same register.
        f.ops[1] = IlOp::Lit(70000);
        assert_eq!(
            store_leaf_text(&f).unwrap().unwrap()[..8],
            [0x3D, 0x60, 0x00, 0x01, 0x61, 0x6B, 0x11, 0x70],
            "lis r11,1 ; ori r11,r11,4464"
        );
        // Not a store leaf at all: the ordinary selector keeps its behaviour.
        f.ops = vec![IlOp::Load(0xF509), IlOp::LoadInd { off: 4 }];
        assert!(store_leaf_text(&f).is_none());
    }

}
