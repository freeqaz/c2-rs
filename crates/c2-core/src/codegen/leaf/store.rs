//! The store leaf: `s->m = v;` as one `stb`/`sth`/`stw`/`std` at a folded
//! displacement. The third consumer of the sub-object designator
//! (`c2-il/src/func/body/shapes/designator.rs`); see `docs/IL_STORE_LEAF.md`.

use c2_il::{IlFunction, IlOp, FP_SCRATCH};
use crate::BackendError;
use crate::codegen::encode::{
    encode_blr,
    encode_lbz,
    encode_ld,
    encode_lfs,
    encode_lhz,
    encode_lwz,
    encode_stb,
    encode_std,
    encode_stfs,
    encode_sth,
    encode_stw,
};
use crate::codegen::select::{ARG_REGS, OptMode, SCRATCH_REG, out_of_class};
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
pub fn store_leaf_text(
    func: &IlFunction,
    mode: OptMode,
) -> Option<Result<Vec<u8>, BackendError>> {
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
    // **The one-value all-literal run: ONE materialization, hoisted, then the
    // stores in source order.** The parser
    // ([`c2_il::try_parse_store_run`]) admits a multi-store literal run only when
    // every statement stores the *same* value, and this is the matching emitter
    // half: the `li` (or `lis`+`ori`) is emitted once, before any store, and the
    // per-group arm below then finds `SCRATCH_REG` already loaded. Detected off
    // the whole `ops` stream rather than statement by statement, because "every
    // statement in the run" is not a property any one group can see.
    //
    // MEASURED — `{ a=9; b=9; c=9; }` is `li r11,9 ; stw ; stw ; stw` at both
    // `/O1` and `/Ox`, and `{ a=100000; b=100000; }` is `lis ; ori ; stw ; stw`
    // with the pair *whole* at the top. Neither mode descends the scratch
    // register here: there is one value, so there is one live range.
    let hoisted_lit: Option<i32> = {
        let mut k0: Option<i32> = None;
        let mut ok = !func.ops.is_empty();
        let mut walk = func.ops.as_slice();
        while ok {
            match walk {
                [] => break,
                [IlOp::Load(_), IlOp::Lit(k), IlOp::StoreInd { .. }, tail @ ..] => {
                    if *k0.get_or_insert(*k) != *k {
                        ok = false;
                    }
                    walk = tail;
                }
                _ => ok = false,
            }
        }
        // A run of ONE keeps the old in-group emission: it is the shape
        // `try_parse_store_leaf` has always produced and its bytes are already
        // graded. Only the multi-store case needs the hoist.
        if ok && func.ops.len() > 3 {
            k0
        } else {
            None
        }
    };
    if let Some(k) = hoisted_lit {
        if let Err(e) = emit_load_imm(&mut text, SCRATCH_REG, k) {
            return Some(Err(e));
        }
    }
    // How many store GROUPS have been emitted. **Not `written.len()`**: a
    // load-valued group deliberately records nothing there (see below), so using
    // the overlap list as the "did we match anything" flag would send a run of
    // load-valued stores with a trailing residue back to the ordinary selector
    // as if it were not a store at all — the `GAPS.md` §6 empty-prefix shape
    // that this file has already been bitten by once.
    let mut groups = 0usize;
    // How many LOAD-valued statements have been emitted in each register file.
    // The two files are allocated independently — MEASURED, `work/wsl/probe/p7.cpp`
    // `MX`: `{ d->f0=s->f0; d->d0=s->d0; d->f1=s->f1; d->d1=s->d1; }` is
    // `lfs f0 ; lfd f13 ; lfs f12 ; lfd f11`, one descending FP sequence over both
    // widths, while `fx3` runs an independent `r11 ; r10` beside its `f0`.
    let (mut gpr_loads, mut fp_loads) = (0usize, 0usize);
    // **The scratch register a loaded value lands in, and the ONE place the
    // `/O1` / `/Ox` split is stated for this shape.** MEASURED over
    // `work/wsl/probe/p6.cpp` — runs of 1..8 crossed with 2..6 pointer
    // parameters, both modes, plus pure-`float` and pure-`double` runs of 1..5 —
    // and `p7.cpp` for the boundary:
    //
    // ```text
    //   /O1   every statement    r11          f0
    //   /Ox   statement i        r(11 - i)    f0, then f(14 - j) for j >= 1
    // ```
    //
    // which is the same allocator `docs/OPT_MODE.md` §3.1 already records for
    // arithmetic chains: `/O1` reuses r11 because each intermediate's
    // predecessor is dead, `/Ox` gives every value its own descending register.
    // The parameter count does not enter it — `g0_5` through `g4_5` are
    // byte-identical — until the sequence reaches a register a parameter holds,
    // and **that is where this refuses** (see the bound below).
    let gpr_scratch = |i: usize| -> u8 {
        match mode {
            OptMode::O1 => SCRATCH_REG,
            OptMode::Ox => SCRATCH_REG.saturating_sub(i as u8),
        }
    };
    let fp_scratch = |j: usize| -> u8 {
        match mode {
            OptMode::O1 => FP_SCRATCH,
            OptMode::Ox if j == 0 => FP_SCRATCH,
            OptMode::Ox => 14u8.saturating_sub(j as u8),
        }
    };
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
                groups += 1;
                rest = tail;
            }
            // **A store whose VALUE is an indirect load** — `d->a = s->b;`, the
            // body of every hand-written copy constructor and copy assignment.
            // Two instructions through the scratch register and no frame; the
            // widths pick both opcodes independently, and the parser has already
            // required the two TYPEs to be byte-identical, so they agree here by
            // construction. MEASURED (`work/wsl/probe/p1.cpp`, `p2.cpp`):
            //
            // ```text
            //   d->a = s->qb   81640004 91630000   lwz r11,4(r4) ; stw r11,0(r3)
            //   d->c = s->c    89640000 99630000   lbz ; stb
            //   d->h = s->h    a1640002 b1630002   lhz ; sth
            //   d->q = s->q    e9640008 f9630008   ld  ; std      (both DS-form)
            //   d->f = s->f    c0040010 d0030010   lfs f0 ; stfs f0
            //   d->g = s->g    c8040018 d8030018   lfd f0 ; stfd f0
            // ```
            //
            // Matched **ahead** of the three-op formal-valued group and of the
            // two-op FP one: a four-op group opens with the same `Load(base)` and
            // only its third op separates it, so an earlier shorter arm would
            // shadow it. The four-op arms are unambiguous against each other
            // because `LoadInd`/`LoadIndSized` and `LoadIndFp` are disjoint.
            [IlOp::Load(b), IlOp::Load(sb), l @ (IlOp::LoadInd { .. } | IlOp::LoadIndSized { .. } | IlOp::LoadIndFp { .. }), st @ (IlOp::StoreInd { .. } | IlOp::StoreIndFp { .. }), tail @ ..] =>
            {
                let (Some(base), Some(sbase)) = (reg_of(*b), reg_of(*sb)) else {
                    return Some(Err(out_of_class(
                        "load-valued store whose base is not a register argument",
                    )));
                };
                // The LOAD half.
                let (soff, lwidth, lfp) = match l {
                    IlOp::LoadInd { off } => (*off, 4u8, false),
                    IlOp::LoadIndSized { off, width, sext } => {
                        if *sext {
                            // A widening `2C` costs an `extsb` between the two
                            // instructions; the parser refuses it, and refusing it
                            // twice is the census/gate invariant.
                            return Some(Err(out_of_class(
                                "load-valued store whose value is sign-extended",
                            )));
                        }
                        (*off, *width, false)
                    }
                    IlOp::LoadIndFp { off, double } => (*off, if *double { 8 } else { 4 }, true),
                    _ => return None,
                };
                let Ok(sd) = i16::try_from(soff) else {
                    return Some(Err(out_of_class(
                        "load-valued store whose load offset exceeds a 16-bit displacement",
                    )));
                };
                // The scratch register for THIS statement, from the file's own
                // running count. **Refused where the descending sequence would
                // reach a register a parameter holds** — that is where c2 stops
                // being a plain descent and starts skipping live registers and
                // wrapping back to r11 (MEASURED, `work/wsl/probe/p7.cpp`: `L9`
                // is `r11 … r5` then **r11** again, and `P8` — two dead `int`
                // parameters ahead of the two pointers — is `r11 … r7, r4, r3,
                // r11`). Reconstructing that needs a liveness model; the gate is
                // drawn where the evidence is a straight descent, and the parser
                // states the same bound so census and gate cannot disagree.
                let (lreg, sreg) = if lfp {
                    let r = fp_scratch(fp_loads);
                    fp_loads += 1;
                    (r, r)
                } else {
                    let r = gpr_scratch(gpr_loads);
                    if r <= 2 + func.params.len().min(8) as u8 {
                        return Some(Err(out_of_class(
                            "load-valued store run longer than the free scratch descent",
                        )));
                    }
                    gpr_loads += 1;
                    (r, r)
                };
                if lfp {
                    text.extend_from_slice(&encode_lfs(lwidth == 8, lreg, sbase, sd));
                } else {
                    match lwidth {
                        1 => text.extend_from_slice(&encode_lbz(lreg, sbase, sd)),
                        2 => text.extend_from_slice(&encode_lhz(lreg, sbase, sd)),
                        4 => text.extend_from_slice(&encode_lwz(lreg, sbase, sd)),
                        // `ld` is DS-form, exactly as `std` is.
                        8 if sd % 4 == 0 => {
                            text.extend_from_slice(&encode_ld(lreg, sbase, sd))
                        }
                        8 => {
                            return Some(Err(out_of_class(
                                "8-byte load whose offset is not a multiple of 4 (ld is DS-form)",
                            )))
                        }
                        _ => return Some(Err(out_of_class("load of an unmodeled width"))),
                    }
                }
                // The STORE half, out of the same scratch register.
                let (off, width, sfp) = match st {
                    IlOp::StoreInd { off, width } => (*off, *width, false),
                    IlOp::StoreIndFp { off, double, src } => {
                        if *src != FP_SCRATCH {
                            return Some(Err(out_of_class(
                                "load-valued FP store out of an argument register",
                            )));
                        }
                        (*off, if *double { 8 } else { 4 }, true)
                    }
                    _ => return None,
                };
                if sfp != lfp || width != lwidth {
                    return Some(Err(out_of_class(
                        "load-valued store whose two halves disagree on width or register file",
                    )));
                }
                let Ok(d) = i16::try_from(off) else {
                    return Some(Err(out_of_class(
                        "store offset exceeds a 16-bit displacement",
                    )));
                };
                if sfp {
                    text.extend_from_slice(&encode_stfs(width == 8, sreg, base, d));
                } else {
                    match width {
                        1 => text.extend_from_slice(&encode_stb(sreg, base, d)),
                        2 => text.extend_from_slice(&encode_sth(sreg, base, d)),
                        4 => text.extend_from_slice(&encode_stw(sreg, base, d)),
                        8 if d % 4 == 0 => {
                            text.extend_from_slice(&encode_std(sreg, base, d))
                        }
                        8 => {
                            return Some(Err(out_of_class(
                                "8-byte store whose offset is not a multiple of 4 (std is DS-form)",
                            )))
                        }
                        _ => return Some(Err(out_of_class("store of an unmodeled width"))),
                    }
                }
                // **Not recorded in `written`.** The dead-store elimination the
                // overlap check below models does not happen when a load sits
                // between the two stores — MEASURED,
                // `{ d->a = s->a; d->a = s->b; }` emits BOTH stores, because `s`
                // may alias `d` and the first one is observable. Feeding these
                // groups to that check would refuse a shape c2 emits in full; the
                // gate that keeps a *loaded* run safe is the parser's aliasing
                // rule (no object both loaded from and stored to), which is a
                // different fact and lives where it can see the tokens.
                groups += 1;
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
                    // The one-value run: the materialization already happened
                    // above, so the group emits the store alone.
                    IlOp::Lit(k) if hoisted_lit == Some(*k) => SCRATCH_REG,
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
                groups += 1;
                rest = tail;
            }
            // Not a store stream at all: leave the ordinary selector's behaviour
            // unchanged. Past the first group it IS one, and a residue that does
            // not parse as a group refuses instead of being dropped.
            _ if groups == 0 => return None,
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
            eh_bare: false,
            eh_unwind_callees: Vec::new(),
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
            store_leaf_text(&f, OptMode::Ox).unwrap().unwrap(),
            vec![0x90, 0x83, 0x00, 0x04, 0x4E, 0x80, 0x00, 0x20],
            "stw r4,4(r3) ; blr"
        );
        // A ZERO displacement is NOT free here — the store still happens. This is
        // the exact opposite of `addr_leaf_text`, whose zero case emits nothing,
        // and the two shapes share a designator.
        f.ops[2] = IlOp::StoreInd { off: 0, width: 4 };
        assert_eq!(
            store_leaf_text(&f, OptMode::Ox).unwrap().unwrap(),
            vec![0x90, 0x83, 0x00, 0x00, 0x4E, 0x80, 0x00, 0x20],
            "stw r4,0(r3) ; blr"
        );
        // The width picks the opcode, and nothing else does.
        f.ops[2] = IlOp::StoreInd { off: 12, width: 1 };
        assert_eq!(
            store_leaf_text(&f, OptMode::Ox).unwrap().unwrap()[..4],
            [0x98, 0x83, 0x00, 0x0C],
            "stb r4,12(r3)"
        );
        f.ops[2] = IlOp::StoreInd { off: 14, width: 2 };
        assert_eq!(
            store_leaf_text(&f, OptMode::Ox).unwrap().unwrap()[..4],
            [0xB0, 0x83, 0x00, 0x0E],
            "sth r4,14(r3)"
        );
        f.ops[2] = IlOp::StoreInd { off: 32, width: 8 };
        assert_eq!(
            store_leaf_text(&f, OptMode::Ox).unwrap().unwrap()[..4],
            [0xF8, 0x83, 0x00, 0x20],
            "std r4,32(r3)"
        );
        // `std` is DS-form: an offset that is not a multiple of 4 cannot be
        // encoded at all, so it refuses rather than dropping the low two bits.
        f.ops[2] = IlOp::StoreInd { off: 30, width: 8 };
        assert!(store_leaf_text(&f, OptMode::Ox).unwrap().is_err());
        // BOTH register fields move: `void f(int x, S* s, int v){ s->b = v; }` is
        // `90a40004` — value r5, base r4 — and a lowering that hardcoded either
        // would pass every two-parameter case.
        f.params = vec![0x1111, 0xF509, 0xF609];
        f.ops[2] = IlOp::StoreInd { off: 4, width: 4 };
        assert_eq!(
            store_leaf_text(&f, OptMode::Ox).unwrap().unwrap(),
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
            store_leaf_text(&f, OptMode::Ox).unwrap().unwrap(),
            vec![
                0x39, 0x60, 0x00, 0x07, 0x91, 0x63, 0x00, 0x00, 0x4E, 0x80, 0x00, 0x20
            ],
            "li r11,7 ; stw r11,0(r3) ; blr"
        );
        // …and a wide literal is the `lis`+`ori` pair through the same register.
        f.ops[1] = IlOp::Lit(70000);
        assert_eq!(
            store_leaf_text(&f, OptMode::Ox).unwrap().unwrap()[..8],
            [0x3D, 0x60, 0x00, 0x01, 0x61, 0x6B, 0x11, 0x70],
            "lis r11,1 ; ori r11,r11,4464"
        );
        // Not a store leaf at all: the ordinary selector keeps its behaviour.
        f.ops = vec![IlOp::Load(0xF509), IlOp::LoadInd { off: 4 }];
        assert!(store_leaf_text(&f, OptMode::Ox).is_none());
    }


    /// W37: the RUN — one store per group, in source order, plus the emitter's
    /// own restatement of the two gates the parser draws. Every expected word is
    /// transcribed from the reference obj of `work/w37/probe/p1.cpp`.
    #[test]
    fn store_run_text_is_one_store_per_statement_in_source_order() {
        let mut f = IlFunction {
            eh_bare: false,
            eh_unwind_callees: Vec::new(),
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
            store_leaf_text(&f, OptMode::Ox).unwrap().unwrap(),
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
            store_leaf_text(&f, OptMode::Ox).unwrap().unwrap(),
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
            store_leaf_text(&f, OptMode::Ox).unwrap().unwrap(),
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
        assert!(store_leaf_text(&f, OptMode::Ox).unwrap().is_err(), "literal inside a run");
        f.ops = vec![
            IlOp::Load(0x0101),
            IlOp::Load(0x0201),
            IlOp::StoreInd { off: 0, width: 4 },
            IlOp::Load(0x0101),
            IlOp::Load(0x0301),
            IlOp::StoreInd { off: 2, width: 4 },
        ];
        assert!(store_leaf_text(&f, OptMode::Ox).unwrap().is_err(), "overlapping stores");
        // …and a run of ONE with a literal is unaffected: that is the store
        // leaf's own captured `li r11,k ; stw r11`.
        f.ops = vec![
            IlOp::Load(0x0101),
            IlOp::Lit(7),
            IlOp::StoreInd { off: 0, width: 4 },
        ];
        assert_eq!(
            store_leaf_text(&f, OptMode::Ox).unwrap().unwrap(),
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
        assert!(store_leaf_text(&f, OptMode::Ox).is_none(), "an ops-less shape is not a store");
    }

    /// WSL: a store whose VALUE is an indirect load — the two-instruction pair,
    /// the widths that pick both opcodes, and the `/O1` / `/Ox` scratch split.
    /// Every expected word is transcribed from the reference obj of
    /// `work/wsl/probe/p1.cpp`, `p2.cpp` and `p6.cpp`, not derived from the
    /// encoding rule.
    #[test]
    fn load_valued_store_is_a_scratch_pair_and_the_mode_picks_the_register() {
        let mut f = IlFunction {
            eh_bare: false,
            eh_unwind_callees: Vec::new(),
            mangled_name: "?c1@@YAXPAUS@@PAUQ@@@Z".into(),
            source_path: None,
            params: vec![0x0101, 0x0201],
            ops: vec![
                IlOp::Load(0x0101),
                IlOp::Load(0x0201),
                IlOp::LoadInd { off: 4 },
                IlOp::StoreInd { off: 0, width: 4 },
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
            store_leaf_text(&f, OptMode::Ox).unwrap().unwrap(),
            vec![
                0x81, 0x64, 0x00, 0x04, // lwz r11,4(r4)
                0x91, 0x63, 0x00, 0x00, // stw r11,0(r3)
                0x4E, 0x80, 0x00, 0x20,
            ],
            "?c1@@YAXPAUS@@PAUQ@@@Z"
        );
        // The narrow and wide widths pick BOTH opcodes, and the two halves are
        // independent fields even though the parser requires them equal.
        f.ops[2] = IlOp::LoadIndSized { off: 0, width: 1, sext: false };
        f.ops[3] = IlOp::StoreInd { off: 0, width: 1 };
        assert_eq!(
            store_leaf_text(&f, OptMode::Ox).unwrap().unwrap()[..8],
            [0x89, 0x64, 0x00, 0x00, 0x99, 0x63, 0x00, 0x00],
            "lbz r11,0(r4) ; stb r11,0(r3)"
        );
        f.ops[2] = IlOp::LoadIndSized { off: 2, width: 2, sext: false };
        f.ops[3] = IlOp::StoreInd { off: 2, width: 2 };
        assert_eq!(
            store_leaf_text(&f, OptMode::Ox).unwrap().unwrap()[..8],
            [0xA1, 0x64, 0x00, 0x02, 0xB1, 0x63, 0x00, 0x02],
            "lhz r11,2(r4) ; sth r11,2(r3)"
        );
        f.ops[2] = IlOp::LoadIndSized { off: 8, width: 8, sext: false };
        f.ops[3] = IlOp::StoreInd { off: 8, width: 8 };
        assert_eq!(
            store_leaf_text(&f, OptMode::Ox).unwrap().unwrap()[..8],
            [0xE9, 0x64, 0x00, 0x08, 0xF9, 0x63, 0x00, 0x08],
            "ld r11,8(r4) ; std r11,8(r3) — both DS-form"
        );
        // The FLOATING-POINT pair lands in f0, never f1: `f1` is the first FP
        // ARGUMENT, and a loaded value is not an argument.
        f.ops[2] = IlOp::LoadIndFp { off: 16, double: false };
        f.ops[3] = IlOp::StoreIndFp { off: 16, double: false, src: FP_SCRATCH };
        assert_eq!(
            store_leaf_text(&f, OptMode::Ox).unwrap().unwrap()[..8],
            [0xC0, 0x04, 0x00, 0x10, 0xD0, 0x03, 0x00, 0x10],
            "lfs f0,16(r4) ; stfs f0,16(r3)"
        );
        f.ops[2] = IlOp::LoadIndFp { off: 24, double: true };
        f.ops[3] = IlOp::StoreIndFp { off: 24, double: true, src: FP_SCRATCH };
        assert_eq!(
            store_leaf_text(&f, OptMode::Ox).unwrap().unwrap()[..8],
            [0xC8, 0x04, 0x00, 0x18, 0xD8, 0x03, 0x00, 0x18],
            "lfd f0,24(r4) ; stfd f0,24(r3)"
        );
        // A run of THREE. `/Ox` gives each statement its own DESCENDING register
        // and `/O1` reuses r11 — the same allocator split `docs/OPT_MODE.md`
        // §3.1 records for arithmetic chains, and the one thing about this shape
        // that is not mode-independent. Transcribed from `?c3@@YAXPAUS@@0@Z` at
        // `/Ox /Gy` and at `/O1 /Gy`.
        let group = |off: i32| {
            vec![
                IlOp::Load(0x0101),
                IlOp::Load(0x0201),
                IlOp::LoadInd { off },
                IlOp::StoreInd { off, width: 4 },
            ]
        };
        f.ops = [group(0), group(4), group(8)].concat();
        assert_eq!(
            store_leaf_text(&f, OptMode::Ox).unwrap().unwrap(),
            vec![
                0x81, 0x64, 0x00, 0x00, 0x91, 0x63, 0x00, 0x00, // lwz r11 ; stw r11
                0x81, 0x44, 0x00, 0x04, 0x91, 0x43, 0x00, 0x04, // lwz r10 ; stw r10
                0x81, 0x24, 0x00, 0x08, 0x91, 0x23, 0x00, 0x08, // lwz r9  ; stw r9
                0x4E, 0x80, 0x00, 0x20,
            ],
            "/Ox descends r11, r10, r9"
        );
        assert_eq!(
            store_leaf_text(&f, OptMode::O1).unwrap().unwrap(),
            vec![
                0x81, 0x64, 0x00, 0x00, 0x91, 0x63, 0x00, 0x00,
                0x81, 0x64, 0x00, 0x04, 0x91, 0x63, 0x00, 0x04,
                0x81, 0x64, 0x00, 0x08, 0x91, 0x63, 0x00, 0x08,
                0x4E, 0x80, 0x00, 0x20,
            ],
            "/O1 reuses r11"
        );
        // The two register FILES are counted independently: an FP statement
        // between two GPR ones must not advance the GPR descent, and vice versa
        // (MEASURED, `?fx3@@YAXPAUW@@0@Z` is `r11 ; f0 ; r10`).
        f.ops = [
            group(0),
            vec![
                IlOp::Load(0x0101),
                IlOp::Load(0x0201),
                IlOp::LoadIndFp { off: 16, double: false },
                IlOp::StoreIndFp { off: 16, double: false, src: FP_SCRATCH },
            ],
            group(32),
        ]
        .concat();
        let t = store_leaf_text(&f, OptMode::Ox).unwrap().unwrap();
        assert_eq!(&t[0..4], [0x81, 0x64, 0x00, 0x00], "lwz r11");
        assert_eq!(&t[8..12], [0xC0, 0x04, 0x00, 0x10], "lfs f0");
        assert_eq!(&t[16..20], [0x81, 0x44, 0x00, 0x20], "lwz r10, not r9");
        // …and the FP descent's own second element is f13, not f1 or f12.
        f.ops = [
            vec![
                IlOp::Load(0x0101),
                IlOp::Load(0x0201),
                IlOp::LoadIndFp { off: 0, double: false },
                IlOp::StoreIndFp { off: 0, double: false, src: FP_SCRATCH },
            ],
            vec![
                IlOp::Load(0x0101),
                IlOp::Load(0x0201),
                IlOp::LoadIndFp { off: 4, double: false },
                IlOp::StoreIndFp { off: 4, double: false, src: FP_SCRATCH },
            ],
        ]
        .concat();
        assert_eq!(
            &store_leaf_text(&f, OptMode::Ox).unwrap().unwrap()[8..12],
            [0xC1, 0xA4, 0x00, 0x04],
            "lfs f13,4(r4)"
        );
        // **The descent refuses where it would reach a parameter's register.**
        // The parser draws the same bound; restating it here is the census/gate
        // invariant, and past it c2 skips live registers and wraps back to r11
        // rather than continuing down.
        f.params = vec![0x0101, 0x0201];
        f.ops = (0..8).flat_map(|i| group(i * 4)).collect();
        assert!(
            store_leaf_text(&f, OptMode::Ox).unwrap().is_err(),
            "eight statements with two parameters reaches r4"
        );
        // …and it is the PARAMETER COUNT that moves the bound, not the length.
        f.params = vec![0x0101, 0x0201, 0x0301, 0x0401];
        f.ops = (0..6).flat_map(|i| group(i * 4)).collect();
        assert!(store_leaf_text(&f, OptMode::Ox).unwrap().is_err(), "six with four");
        f.ops = (0..5).flat_map(|i| group(i * 4)).collect();
        assert!(store_leaf_text(&f, OptMode::Ox).unwrap().is_ok(), "five with four");
        // `/O1` has no bound at all, because every statement reuses r11.
        f.params = vec![0x0101, 0x0201];
        f.ops = (0..8).flat_map(|i| group(i * 4)).collect();
        assert!(store_leaf_text(&f, OptMode::O1).unwrap().is_ok());
    }
}
