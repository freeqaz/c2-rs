//! The floating-point leaf: FP arithmetic chains, the register move, and the
//! pooled constant. See `docs/CODEGEN_W13_FLOAT.md`.

use crate::func::body::chain::leaves_ascending;
use crate::func::body::expr::{eat_return_plumbing, parse_formals, BODY_SCOPE_DEPTH};
use crate::func::body::BodyShape;
use crate::func::readers::{
    read_token_var, DOUBLE_LIT_TYPE, DOUBLE_TYPE, FLOAT_LIT_TYPE, FLOAT_TYPE,
};
use crate::func::sy::{ArgClass, SyView};
use crate::func::IlOp;

use super::this_binding::{ThisBinding, parse_this_token};

/// Try to parse a **W13a floating-point leaf**: a straight-line chain over
/// float (or double) *parameters* only.
///
/// ```text
///   ( B9 <tok> <FT> | <op> )+     LOADs and binary ops, all of one FP type
///   41 <FT>                       result type, the SAME FP type
///   <return plumbing>
/// ```
///
/// The gate list is from `docs/CODEGEN_W13_FLOAT.md` §6 and every item is a
/// case where a naive selector emits *wrong* bytes rather than merely running
/// out of range:
///
/// * **No literal.** Every FP constant costs an `.rdata` COMDAT, a REFHI/REFLO
///   relocation pair and a GPR — that is W13b.
/// * **No `2C` convert**, and no mixing of float with double: a mixed-width
///   expression evaluates in double and may need an `frsp`.
/// * **No `*` under `+`/`-`.** Contraction to `fmadds`/`fmsubs`/`fnmsubs` is
///   *mandatory* in c2, so emitting the two separate instructions would be a
///   silent mis-emit. Approximated conservatively here by rejecting any chain
///   that contains both a `Mul` and an `Add`/`Sub`.
/// * **No repeated leaf.** `a + a` is algebraically rewritten to `a * 2.0f`,
///   which is a constant and therefore `.rdata` again.
/// * **No `0x59` marker.** It tracks source parenthesisation and is the only
///   thing distinguishing product shapes c2 flattens from those it does not;
///   its meaning is unknown, so its presence rejects.
pub(crate) fn try_parse_float_leaf(
    seg: &[u8],
    start: usize,
    lo: usize,
    sy: SyView,
) -> Option<BodyShape> {
    let mut p = start;
    // The operand type is fixed by the first LOAD and every later one must match.
    if *seg.get(p)? != 0xB9 {
        return None;
    }
    let double = {
        let mut probe = p + 1;
        let (_, w) = read_token_var(seg, probe)?;
        probe += w;
        if seg.get(probe..probe + 3)? == FLOAT_TYPE {
            false
        } else if seg.get(probe..probe + 3)? == DOUBLE_TYPE {
            true
        } else {
            return None;
        }
    };
    let fty = if double { DOUBLE_TYPE } else { FLOAT_TYPE };

    let mut ops: Vec<IlOp> = Vec::new();
    loop {
        match *seg.get(p)? {
            0xB9 => {
                p += 1;
                let (tok, w) = read_token_var(seg, p)?;
                p += w;
                if seg.get(p..p + 3)? != fty {
                    return None; // mixed width, or a non-FP operand
                }
                p += 3;
                ops.push(IlOp::Load(tok));
            }
            0x02 => {
                p += 1;
                ops.push(IlOp::Add);
            }
            0x03 => {
                p += 1;
                ops.push(IlOp::Sub);
            }
            0x04 => {
                p += 1;
                ops.push(IlOp::Mul);
            }
            0x05 => {
                p += 1;
                ops.push(IlOp::Div);
            }
            // W13b: a floating-point literal.
            //
            //   33 <lit-TYPE> <8 bytes: IEEE binary64, little-endian> <u16 size>
            //
            // The payload is a binary64 pattern even for a `float` (already
            // rounded to binary32 precision), and the u16 trailer is the operand
            // *width* — 4 for float, 8 for double — which must agree with the
            // literal tag. Verified byte-for-byte against a live capture of
            // `float k_add(float a){return a + 1.0f;}`:
            //   33 86 4a 40 00 00 00 00 00 00 f0 3f 04 00
            0x33 => {
                p += 1;
                let lty = seg.get(p..p + 3)?;
                let lit_double = if lty == FLOAT_LIT_TYPE {
                    false
                } else if lty == DOUBLE_LIT_TYPE {
                    true
                } else {
                    return None; // an integer (or other) literal: out of class
                };
                // A literal of the other width implies a conversion.
                if lit_double != double {
                    return None;
                }
                p += 3;
                let raw: [u8; 8] = seg.get(p..p + 8)?.try_into().ok()?;
                p += 8;
                let size = u16::from_le_bytes(seg.get(p..p + 2)?.try_into().ok()?);
                p += 2;
                if size as usize != if double { 8 } else { 4 } {
                    return None;
                }
                ops.push(IlOp::FpLit {
                    bits: u64::from_le_bytes(raw),
                    double,
                });
            }
            0x41 => break,
            // 0x2C convert, 0x59 paren marker, 0x08 neg and every other byte
            // reject — see the gate list above.
            _ => return None,
        }
    }
    // Result type must be the same FP type.
    p += 1;
    if seg.get(p..p + 3)? != fty {
        return None;
    }
    p += 3;
    eat_return_plumbing(seg, &mut p, false, BODY_SCOPE_DEPTH).ok()?;

    // ~~A `*` mixed with `+`/`-` contracts; reject rather than emit two
    // instructions where c2 emits one.~~
    //
    // **STRUCK 2026-08-29, lane `w-fmadd`, board `#3792`.** The contraction is
    // modeled: `c2_core::codegen::leaf::float`'s `node_plan` defers every
    // multiply and folds it into its parent `+`/`-` as `fmadd`/`fmsub`/
    // `fnmsub`, from a read of c2's own form-24 arm at `0x10bfa49a`. The gate
    // stayed for as long as this comment says it did because the *encoder* had
    // no form-24 field plan either, so there was nothing to lower to.
    //
    // **What the widening does NOT relax** — each of these keeps refusing the
    // contracted shapes too, and together they are why "the mix is in class"
    // is not "any mix is in class":
    //
    // * the ascending-leaf gate below (`b*a + c` reaches c2 as `fmadds
    //   f1,f1,f2,f3`, i.e. the *canonicalized* pair, and this port does not
    //   model FP canonicalization);
    // * the `0x59` parenthesis marker, which rejects `(a+b)*c + d` and
    //   `a*(b*c + d)` at the top of the op loop;
    // * the repeated-leaf gate;
    // * a pooled constant inside a contracted expression, refused in codegen
    //   where the width and the scheduling question live.
    let has_mul = ops.iter().any(|o| matches!(o, IlOp::Mul));
    let has_addsub = ops.iter().any(|o| matches!(o, IlOp::Add | IlOp::Sub));
    // **What DOES still refuse the mix, and it refuses HERE rather than in
    // codegen.** `super::super::chain::fp_contract_instructions` is the whole
    // contraction rule; it returns `Err` for the two op-stream shapes the
    // lowering declines — a `+` chain c2 reassociates, and a product on both
    // sides of a node that cannot contract. The first draft of this lane put
    // that refusal in `float_leaf_text` alone, and `census_gate.rs` went red
    // saying so: the census counted 2 of 19,549 generated bodies in class that
    // `PortC2` then refused, which is a coverage over-claim by exactly 2.
    // `docs/GAPS.md` §6 — acceptance belongs in the parser.
    if crate::func::body::chain::fp_contract_instructions(&ops).is_err() {
        return None;
    }

    // ---- W13b constant gates ------------------------------------------------
    //
    // These live here, in the parser, rather than in codegen so that the census
    // and the emission gate cannot disagree about what is in class.
    //
    // c2 — not c1xx — evaluates floating-point constants, so the IL still holds
    // every literal the source wrote and the backend is free to fold, reassociate
    // and strength-reduce them. Three captured behaviours the port does not
    // model, each of which would be a silent mis-emit:
    let lits: Vec<(u64, bool)> = ops
        .iter()
        .filter_map(|o| match o {
            IlOp::FpLit { bits, double } => Some((*bits, *double)),
            _ => None,
        })
        .collect();
    if !lits.is_empty() {
        // (1) Two or more literals: c2 folds them where it can (`a*2.0f*b*3.0f`
        //     becomes `(a*b)*6.0f`), and where it cannot it hoists every `addis`
        //     into a prologue group and schedules the loads at first use. Either
        //     way the one-constant lowering is wrong. See `w13b_fpool.cpp`.
        if lits.len() > 1 {
            return None;
        }
        // (2) A constant divisor becomes a reciprocal multiply: `a/2.0f` emits
        //     `fmuls` against `__real@3f000000`, and `a/3.0f/7.0f` collapses to
        //     one `fmuls` by 1/21 — a value that is not even exactly
        //     representable, so this is a numeric transform, not a rewrite.
        if ops.iter().any(|o| matches!(o, IlOp::Div)) {
            return None;
        }
        let (bits, lit_double) = lits[0];
        let v = f64::from_bits(bits);
        // (3) An identity operand disappears entirely — `a + 0.0f`, `a - 0.0f`
        //     and `a * 1.0f` each compile to a bare `blr`, with no constant
        //     pooled at all. (`a * 0.0f` is *not* folded: it really does load
        //     zero and multiply.) Refuse when the literal is an identity for any
        //     operator in the body; slight over-refusal beats emitting three
        //     instructions where c2 emits none.
        if v == 0.0 && has_addsub {
            return None;
        }
        if v == 1.0 && has_mul {
            return None;
        }
        // (4) A `float` literal is carried as a binary64 pattern already rounded
        //     to binary32. If it does not narrow exactly, the four bytes we would
        //     pool are not the ones c2 pooled.
        if !lit_double && f64::from(v as f32).to_bits() != bits {
            return None;
        }
    }
    // FP chains are canonicalized by register exactly as integer ones are: `b + a`
    // and `b * a` emit the operands in ascending order, and every permutation of
    // `a + b + c` emits one stream. The port evaluated source order, so all of those
    // were mis-emits until the generated sweep found them.
    //
    // Division is tighter still. One division as the *only* operator is byte-exact
    // (`a / b`, `b / a` — it is non-commutative, so order is preserved), but two
    // divisions (`a / b / c`) or a division mixed with anything else (`a + b / c`)
    // are not what the serial model emits. Both refuse.
    let n_binops = ops
        .iter()
        .filter(|o| !matches!(o, IlOp::Load(_) | IlOp::Lit(_) | IlOp::FpLit { .. }))
        .count();
    if ops.iter().any(|o| matches!(o, IlOp::Div)) && n_binops != 1 {
        return None;
    }
    // A repeated leaf can trigger algebraic rewriting into a constant.
    let mut seen: Vec<u32> = Vec::new();
    for o in &ops {
        if let IlOp::Load(t) = o {
            if seen.contains(t) {
                return None;
            }
            seen.push(*t);
        }
    }
    // **The FP register file is numbered over the FP parameters ALONE**, so the
    // shape carries its parameters in *that* order and not in declaration order.
    // [`c2_core::codegen::float_leaf_text`] maps entry `n` of this list to
    // `f(n+1)`, which is the register number exactly when the list holds the FP
    // parameters and nothing else.
    //
    // This is the fifth instance of `docs/GAPS.md` §6's "two facts sharing one
    // field", and it was **live** — `float mixfp(int a, float b, float c)
    // { return b*c; }` emitted `fmuls f1,f2,f3` where c2 emits `fmuls f1,f1,f2`,
    // on mainline, with all four mode lanes and the 3,743-case sweep green. The
    // corpus had only the safe half of the pair again: not one FP fixture had a
    // parameter list that was anything but all-`float` or all-`double`.
    //
    // It was closed then by the blunt gate `params.len() != seen.len()` — every
    // formal has to be an FP operand of the body — which is correct and costs
    // **1,005 functions** on the workload (`IL_CALL_IN_EXPR.md` §23.1, MEASURED
    // by counterfactual). What replaces it is the actual numbering, read from
    // `.sy`'s type kind (`sy::ArgClass`): a non-FP formal is skipped rather than
    // refused, and an FP formal that the body never loads still advances the
    // count. `this` is prepended as a GPR — it takes r3 and displaces nothing in
    // the FP file, which the member-function capture in `docs/CODEGEN_FP_ARGS.md`
    // §1 confirms emits the identical `fmr` sequence as the free function.
    let formals = parse_formals(seg, lo).ok()?;
    let classes = sy.arg_classes(&formals).ok()?;
    let this = matches!(parse_this_token(seg, lo)?, ThisBinding::Bound(_));
    let params: Vec<u32> = formals
        .iter()
        .zip(&classes)
        .filter(|(_, c)| matches!(c, ArgClass::Fp { .. }))
        .map(|(t, _)| *t)
        .collect();
    // Mixing the widths across the *parameter list* is not the same question as
    // mixing them inside the expression (which the operand-type loop already
    // refuses): `float f(double a, float b){ return b; }` is one FP file with two
    // widths in it, and every operand this body reads is `fty`. The register
    // numbering is width-agnostic — `int t8(double a, float b, double c)` puts
    // them in f1, f2, f3 (captured) — so nothing here needs the widths to agree.
    if params.len() > 13 || !seen.iter().all(|t| params.contains(t)) {
        return None;
    }
    // **A pooled constant keeps the OLD gate**, and this is the one place the
    // widening is deliberately held back rather than taken.
    //
    // `float_leaf_text` emits a pooled constant as an `.rdata` COMDAT reached
    // through a REFHI/REFLO pair, and `codegen::function_gate` refuses that under
    // **function-level linking** (`/Gy`, which `/O1` implies) because the COMDAT
    // association is not modeled — a refusal that lives in codegen only, because
    // the linkage mode is a translation-unit flag the parser cannot see. That
    // split cost nothing while no such body was in class; widening the parameter
    // model put one in class (a member FP leaf with a constant, in
    // `src/lazer/meta_ham/HamProfile.cpp`) and the 878-TU scan's census/gate
    // disagreement went 0 → **1**, in the over-claiming direction.
    //
    // So the pooled-constant population is held **exactly** at what it was before
    // this rung: every formal must be an FP operand of the body, the gate this
    // rung otherwise replaces. It costs **1 function** on the workload (measured,
    // and the whole of the disagreement it caused), and it keeps the invariant at
    // 0 in both directions without narrowing anything the rung is actually about.
    // The real repair is to model `/Gy` `.rdata` COMDATs — `docs/CODEGEN_FP_ARGS.md`
    // §5 ranks it — after which this clause deletes itself.
    if ops.iter().any(|o| matches!(o, IlOp::FpLit { .. }))
        && (params.len() != seen.len() || this)
    {
        return None;
    }
    let _ = this;
    // c2 canonicalizes a chain containing a **commutative** operator by register,
    // exactly as it does an integer one, so such a chain must already be written in
    // ascending order. A chain with only non-commutative operators is left alone —
    // `b - a` and `b / a` really do emit their operands in source order, and gating
    // them would refuse bodies that are byte-exact today.
    let has_commutative = ops
        .iter()
        .any(|o| matches!(o, IlOp::Add | IlOp::Mul));
    if has_commutative && !leaves_ascending(&ops, &params) {
        return None;
    }
    Some(BodyShape::FloatLeaf { params, ops, double })
}
