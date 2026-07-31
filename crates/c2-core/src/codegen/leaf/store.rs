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
    let reg_of = |tok: u32| -> Option<u8> {
        func.params
            .iter()
            .position(|&t| t == tok)
            .filter(|&i| i < ARG_REGS.len())
            .map(|i| ARG_REGS[i])
    };
    // The **run** is the general case and the single store is its length-1
    // instance, so there is one walk rather than two — `GAPS.md` §6's "one fact,
    // one locator" in the emitter, matching the parser's own single
    // `parse_store_stmt`. `c2_il::try_parse_store_leaf` and
    // `try_parse_store_run` are the only producers of `StoreInd`/`StoreIndFp`,
    // and both emit exact groups, so a residue that does not match a group is a
    // stream this function must refuse rather than ignore.
    // **At least one group.** The single-store version pattern-matched the whole
    // `ops` slice EXACTLY; a loop matches a *prefix*, and the empty prefix matches
    // everything — so an `ops`-less shape whose data lives in another field
    // (`IlFunction::compare`) walked straight past the loop and came out as a bare
    // `blr`. That is a live wrong-bytes emit created by this rung and caught by the
    // one fixture that puts a compare leaf in the same TU as a store
    // (`w29_fp_contract.cpp`, `Port=Mismatch @ 8`); `w25_store_leaf.cpp` has no
    // compare and was green over it. `GAPS.md` §6: turning an exact match into a
    // prefix match adds the empty case, and the empty case is never the one the
    // rung is about.
    if func.ops.is_empty() {
        return None;
    }
    let mut rest = func.ops.as_slice();
    let mut text = Vec::with_capacity(16);
    let mut written: Vec<(u32, i32, u8)> = Vec::new();
    while !rest.is_empty() {
        match rest {
            // The **floating-point** store, `void f(S* s, float v){ s->f = v; }` —
            // one `stfs`/`stfd`. Two ops rather than three: the value's register is
            // already resolved, because the FP argument file is numbered over the FP
            // parameters alone and only the IL layer has the `.sy` view that says which
            // parameters those are ([`c2_il::IlOp::StoreIndFp`]). The base is the ordinary
            // GPR argument, and its index *is* its register number even with FP formals in
            // the list — an FP parameter fills no GPR but still consumes its slot, so the
            // two effects cancel exactly (`docs/ABI_EDGES.md` §2, and the capture
            // `void s_arg2(int x, S* s, float v){ s->f = v; }` → `stfs f1,4(r4)`).
            [IlOp::Load(b), IlOp::StoreIndFp { off, double, src }, tail @ ..] => {
                let d = match i16::try_from(*off) {
                    Ok(d) => d,
                    Err(_) => {
                        return Some(Err(out_of_class(
                            "FP store offset exceeds a 16-bit displacement",
                        )))
                    }
                };
                let Some(base) = reg_of(*b) else {
                    return Some(Err(out_of_class(
                        "FP store whose base is not a register argument",
                    )));
                };
                text.extend_from_slice(&encode_stfs(*double, *src, base, d));
                written.push((*b, *off, if *double { 8 } else { 4 }));
                rest = tail;
            }
            [IlOp::Load(b), v @ (IlOp::Load(_) | IlOp::Lit(_)), IlOp::StoreInd { off, width }, tail @ ..] =>
            {
                let d = match i16::try_from(*off) {
                    Ok(d) => d,
                    // The parser gates this; if it ever changed, refuse rather than truncate.
                    Err(_) => {
                        return Some(Err(out_of_class(
                            "store offset exceeds a 16-bit displacement",
                        )))
                    }
                };
                let Some(base) = reg_of(*b) else {
                    return Some(Err(out_of_class(
                        "store whose base is not a register argument",
                    )));
                };
                let src = match v {
                    IlOp::Load(t) => match reg_of(*t) {
                        Some(r) => r,
                        None => {
                            return Some(Err(out_of_class(
                                "store whose value is not a register argument",
                            )))
                        }
                    },
                    // **A literal is only lowered for a run of ONE**, and this
                    // is the second lock on the rule the parser states: c2
                    // hoists the `li`s out of a multi-store sequence, allocates
                    // them r11/r10/r9 descending, CSEs equal ones and *reorders
                    // the stores* around them (`docs/IL_STORE_LEAF.md` §11).
                    // Restated here rather than trusted, because a census/gate
                    // disagreement in either direction is what `GAPS.md` §6's
                    // "one fact, two locators" bullet exists to prevent.
                    IlOp::Lit(_) if !(rest.len() == 3 && text.is_empty()) => {
                        return Some(Err(out_of_class(
                            "literal value inside a multi-store run",
                        )))
                    }
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
                written.push((*b, *off, *width));
                rest = tail;
            }
            // Not a store stream at all: leave the ordinary selector's behaviour
            // unchanged. Past the first group it IS one, and a residue that does
            // not parse as a group refuses instead of being dropped.
            _ if written.is_empty() => return None,
            _ => return Some(Err(out_of_class("store run with an unmodeled residue"))),
        }
    }
    // **No two stores may write overlapping bytes of the same base**, the
    // emitter's copy of the parser's dead-store gate: `{ s->a=u; s->a=w; }` is a
    // *single* `stw r5,0(r3)` in the reference, so emitting both is wrong bytes.
    for (i, a) in written.iter().enumerate() {
        for b in &written[i + 1..] {
            if a.0 == b.0
                && a.1 < b.1 + i32::from(b.2)
                && b.1 < a.1 + i32::from(a.2)
            {
                return Some(Err(out_of_class(
                    "two stores overlapping the same base object",
                )));
            }
        }
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


    /// W37: the RUN — one store per group, in source order, plus the emitter's
    /// own restatement of the two gates the parser draws. Every expected word is
    /// transcribed from the reference obj of `work/w37/probe/p1.cpp`.
    #[test]
    fn store_run_text_is_one_store_per_statement_in_source_order() {
        let mut f = IlFunction {
            mangled_name: "?s2@@YAXPAUS@@HH@Z".into(),
            source_path: None,
            params: vec![0x0101, 0x0201, 0x0301],
            ops: vec![
                IlOp::Load(0x0101),
                IlOp::Load(0x0201),
                IlOp::StoreInd { off: 0, width: 4 },
                IlOp::Load(0x0101),
                IlOp::Load(0x0301),
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
            vec![
                0x90, 0x83, 0x00, 0x00, // stw r4,0(r3)
                0x90, 0xA3, 0x00, 0x04, // stw r5,4(r3)
                0x4E, 0x80, 0x00, 0x20, // blr
            ],
            "?s2@@YAXPAUS@@HH@Z"
        );
        // SOURCE order, not offset order — the one thing an ascending run cannot
        // distinguish, and the reason `?s2r@@YAXPAUS@@HH@Z` is in the probe.
        f.ops = vec![
            IlOp::Load(0x0101),
            IlOp::Load(0x0301),
            IlOp::StoreInd { off: 4, width: 4 },
            IlOp::Load(0x0101),
            IlOp::Load(0x0201),
            IlOp::StoreInd { off: 0, width: 4 },
        ];
        assert_eq!(
            store_leaf_text(&f).unwrap().unwrap(),
            vec![
                0x90, 0xA3, 0x00, 0x04, // stw r5,4(r3)
                0x90, 0x83, 0x00, 0x00, // stw r4,0(r3)
                0x4E, 0x80, 0x00, 0x20,
            ],
            "?s2r@@YAXPAUS@@HH@Z"
        );
        // The widths are per statement, and the FP group is two ops rather than
        // three (`?s2f@@YAXPAUS@@MN@Z`, the other register file).
        f.ops = vec![
            IlOp::Load(0x0101),
            IlOp::StoreIndFp { off: 16, double: false, src: 1 },
            IlOp::Load(0x0101),
            IlOp::StoreIndFp { off: 24, double: true, src: 2 },
        ];
        assert_eq!(
            store_leaf_text(&f).unwrap().unwrap(),
            vec![
                0xD0, 0x23, 0x00, 0x10, // stfs f1,16(r3)
                0xD8, 0x43, 0x00, 0x18, // stfd f2,24(r3)
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
        // **The emitter's own copy of the two gates.** A literal inside a run and
        // two stores overlapping one base are both wrong bytes rather than gaps,
        // so codegen restates them: `GAPS.md` §6's "one fact, two locators", with
        // the parser as the first lock and this as the second.
        f.ops = vec![
            IlOp::Load(0x0101),
            IlOp::Load(0x0201),
            IlOp::StoreInd { off: 0, width: 4 },
            IlOp::Load(0x0101),
            IlOp::Lit(2),
            IlOp::StoreInd { off: 4, width: 4 },
        ];
        assert!(store_leaf_text(&f).unwrap().is_err(), "literal inside a run");
        f.ops = vec![
            IlOp::Load(0x0101),
            IlOp::Load(0x0201),
            IlOp::StoreInd { off: 0, width: 4 },
            IlOp::Load(0x0101),
            IlOp::Load(0x0301),
            IlOp::StoreInd { off: 2, width: 4 },
        ];
        assert!(store_leaf_text(&f).unwrap().is_err(), "overlapping stores");
        // …and a run of ONE with a literal is unaffected: that is the store
        // leaf's own captured `li r11,k ; stw r11`.
        f.ops = vec![
            IlOp::Load(0x0101),
            IlOp::Lit(7),
            IlOp::StoreInd { off: 0, width: 4 },
        ];
        assert_eq!(
            store_leaf_text(&f).unwrap().unwrap(),
            vec![0x39, 0x60, 0x00, 0x07, 0x91, 0x63, 0x00, 0x00, 0x4E, 0x80, 0x00, 0x20],
            "li r11,7 ; stw r11,0(r3) ; blr"
        );
        // **An `ops`-less function must return `None`, not a bare `blr`.** The
        // single-store version matched the whole slice exactly; a loop matches a
        // PREFIX, and the empty prefix matches everything. That turned every
        // comparison leaf in a store-bearing TU into four bytes of `blr` —
        // `w29_fp_contract.cpp`, `Port=Mismatch @ 8` — and `w25_store_leaf.cpp`,
        // which has no compare, was green over it.
        f.ops = Vec::new();
        assert!(store_leaf_text(&f).is_none(), "an ops-less shape is not a store");
    }
}
