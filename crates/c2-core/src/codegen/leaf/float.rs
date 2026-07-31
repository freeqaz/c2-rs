//! The floating-point leaf: FP arithmetic chains, the `fmr` register move,
//! and the constant pool. See `docs/CODEGEN_W13_FLOAT.md` and
//! `docs/CODEGEN_FP_ARGS.md` — the FP argument register file is numbered over
//! the FP parameters ALONE, which is a different fact from the positional
//! index and was a live mis-emit until W27 separated them.

use c2_il::{IlFunction, IlOp};
use crate::BackendError;
use crate::codegen::encode::{
    encode_addis,
    encode_blr,
    encode_fadd,
    encode_fdiv,
    encode_fmr,
    encode_fmul,
    encode_frsp,
    encode_fsub,
    encode_lfs,
};
use crate::codegen::select::{SCRATCH_REG, out_of_class};

/// FP scratch pool, in allocation order: `f0` first, then descending from `f13`,
/// wrapping. Deliberately NOT the integer shape — `f0` is allocatable and comes
/// first, and the result register `f1` is last.
const FP_POOL: [u8; 14] = [0, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1];

/// FP return register.
const FP_RET: u8 = 1;

/// A **floating-point constant reference site** produced by [`float_leaf_text`].
///
/// c2 never materializes an FP constant with immediates — there is no FP
/// equivalent of `li`. It pools the value into its own `.rdata` COMDAT and loads
/// it through a two-instruction high/low address pair:
///
/// ```text
/// addis r11,r0,0     <- REFHI(__real@…) + PAIR
/// lfs   f0,0(r11)    <- REFLO(__real@…) + PAIR
/// ```
///
/// Both immediates are emitted as 0; the linker patches them. `hi_off` is the
/// `addis` byte offset **relative to the start of this function's text** — the
/// caller rebases it by the function's `.text` offset. The `lfs`/`lfd` always
/// immediately follows, so the REFLO site is `hi_off + 4`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FpConstRef {
    /// The constant's value as raw IEEE-754 **binary64** bits (as the IL carries
    /// it), regardless of the reference width.
    pub bits: u64,
    /// True for a `double` (8-byte `.rdata`, `lfd`); false for `float`.
    pub double: bool,
    /// Byte offset of the `addis` within this function's text.
    pub hi_off: u32,
}

/// Select `.text` for a **W13a/W13b floating-point leaf**: a straight-line chain
/// over float (or double) *parameters* and pooled constants, with no conversions.
///
/// Register model, which differs from the integer one in every particular
/// (docs/CODEGEN_W13_FLOAT.md §2):
/// * parameters occupy `f1…f13` in float-parameter order; the result is `f1`;
/// * temporaries come from a rotating cursor over [`FP_POOL`] — `f0` first, then
///   down from `f13` — skipping registers that still hold a live value;
/// * an FP `+` chain does **not** collapse to a single accumulator the way the
///   integer one does.
///
/// Verified: `float fmul3(float a,float b,float c){return a*b*c;}` selects
/// `fmuls f0,f1,f2 ; fmuls f1,f0,f3 ; blr`.
///
/// Returns the text plus one [`FpConstRef`] per constant **reference site**, in
/// emission order; the caller pools them into `.rdata` COMDATs and turns each
/// into a REFHI/PAIR/REFLO/PAIR relocation quad.
pub fn float_leaf_text(
    func: &IlFunction,
    double: bool,
) -> Result<(Vec<u8>, Vec<FpConstRef>), BackendError> {
    if func.params.len() > 13 {
        return Err(out_of_class(
            "more than 13 FP parameters: the 14th is stack-homed; out of class",
        ));
    }
    // Parameter n → f(n+1).
    let reg_of = |tok: u32| -> Option<u8> {
        func.params
            .iter()
            .position(|&t| t == tok)
            .map(|i| (i + 1) as u8)
    };

    // Which ops appear — A5/A7 gating happens in the IL parser, but the mix is
    // re-checked here because a contraction mis-emit is silent.
    let has_mul = func.ops.iter().any(|o| matches!(o, IlOp::Mul));
    let has_addsub = func
        .ops
        .iter()
        .any(|o| matches!(o, IlOp::Add | IlOp::Sub));
    if has_mul && has_addsub {
        return Err(out_of_class(
            "FP expression mixes `*` with `+`/`-`: c2 contracts these to \
             fmadds/fmsubs/fnmsubs, which is not modeled; out of class",
        ));
    }

    // Evaluate the postfix stream over a stack of physical FP registers.
    let n_ops = func
        .ops
        .iter()
        .filter(|o| !matches!(o, IlOp::Load(_) | IlOp::Lit(_) | IlOp::FpLit { .. }))
        .count();
    let mut emitted = 0usize;
    let mut cursor = 0usize;
    let mut live: Vec<u8> = (1..=func.params.len() as u8).collect();
    let mut stack: Vec<u8> = Vec::new();
    let mut text: Vec<u8> = Vec::new();
    let mut consts: Vec<FpConstRef> = Vec::new();
    // Address GPRs for constant loads come off the integer scratch cursor,
    // descending from r11 exactly as the integer selector's do.
    let mut next_addr_gpr: u8 = SCRATCH_REG;

    // Pull the next free FP register off the rotating pool cursor.
    let take_fp = |cursor: &mut usize, live: &[u8]| -> Result<u8, BackendError> {
        for _ in 0..FP_POOL.len() {
            let cand = FP_POOL[*cursor % FP_POOL.len()];
            *cursor += 1;
            if !live.contains(&cand) {
                return Ok(cand);
            }
        }
        Err(out_of_class(
            "no free FP scratch register (would spill f31/f30)",
        ))
    };

    // W13b gate. With **one** pooled constant the address setup and the load sit
    // adjacently, immediately before the use — verified byte-exact on six
    // distinct bodies (`w13b_fconst`, and `ka`/`kb`/`kc`/`kd`/`ke`/`kdiv` in
    // `w13b_fpool`). With two, c2 stops doing that: it hoists *every* `addis`
    // into the function prologue as a group, then schedules each `lfs` at its
    // first use and recycles the FP register once a constant dies. See the `p1`
    // and `p5` captures in `docs/CODEGEN_W13_FLOAT.md` §5.3 — `p5` loads its
    // second constant back into `f0`. That scheduler is not modeled, and the
    // REFLO site stops being `hi_off + 4`, so refuse rather than mis-emit.
    let n_consts = func
        .ops
        .iter()
        .filter(|o| matches!(o, IlOp::FpLit { .. }))
        .count();
    if n_consts > 1 {
        return Err(out_of_class(
            "more than one pooled FP constant in one body: c2 hoists the `addis` \
             address setup into the prologue and schedules the loads at first \
             use; that scheduler is not modeled; out of class",
        ));
    }
    // A constant divisor does not survive as a division: **c2** — not c1xx —
    // strength-reduces it to a reciprocal multiply (`a/3.0f/7.0f` reaches the
    // backend with both literals and leaves it having pooled `__real@3d430c31`,
    // i.e. 1/21, and emitted one `fmuls`). That is the whole reason this gate
    // exists: the IL still holds the division, so seeing one here is expected,
    // and lowering it as `fdivs` would be the mis-emit.
    if n_consts > 0 && func.ops.iter().any(|o| matches!(o, IlOp::Div)) {
        return Err(out_of_class(
            "FP division involving a pooled constant: c2 strength-reduces a \
             constant divisor to a reciprocal multiply; out of class",
        ));
    }
    // Constants claim their FP register **before** any interior temporary does,
    // in IL order. Verified by `ke` (`a*2.0f*b*3.0f`, folded to `(a*b)*6.0f`):
    // c2 emits `fmuls f13,f1,f2` and puts the constant in `f0`, so the constant
    // took pool slot 0 even though the multiply is emitted first.
    let mut const_fp: Vec<u8> = Vec::new();
    for _ in 0..n_consts {
        let r = take_fp(&mut cursor, &live)?;
        live.push(r);
        const_fp.push(r);
    }
    let mut next_const = 0usize;

    for op in &func.ops {
        match op {
            IlOp::Load(tok) => {
                let r = reg_of(*tok).ok_or_else(|| {
                    out_of_class("FP LOAD of a token that is not a parameter")
                })?;
                stack.push(r);
            }
            IlOp::Lit(_) => {
                return Err(out_of_class(
                    "integer literal in an FP expression implies a conversion; \
                     out of class",
                ))
            }
            // W13b: a pooled constant. `addis rA,r0,0` + `lfs/lfd fD,0(rA)`,
            // with both immediates left 0 for the REFHI/REFLO relocations.
            IlOp::FpLit { bits, double: lit_double } => {
                if *lit_double != double {
                    return Err(out_of_class(
                        "FP constant width differs from the expression width \
                         (implies a conversion); out of class",
                    ));
                }
                // A `float` constant must survive the binary64 → binary32
                // narrowing exactly, or the pooled 4 bytes would not be the
                // value c2 pooled.
                if !double {
                    let v = f64::from_bits(*bits);
                    if f64::from(v as f32).to_bits() != *bits {
                        return Err(out_of_class(
                            "float constant is not exactly representable in \
                             binary32; out of class",
                        ));
                    }
                }
                let gpr = next_addr_gpr;
                if gpr < 9 {
                    return Err(out_of_class(
                        "FP constant pool needs more address registers than the \
                         characterized descending range r11..r9; out of class",
                    ));
                }
                next_addr_gpr = gpr - 1;
                // Pre-assigned above; `live` already reflects it.
                let fd = const_fp[next_const];
                next_const += 1;
                consts.push(FpConstRef {
                    bits: *bits,
                    double,
                    hi_off: text.len() as u32,
                });
                text.extend_from_slice(&encode_addis(gpr, 0, 0));
                text.extend_from_slice(&encode_lfs(double, fd, gpr, 0));
                stack.push(fd);
            }
            binop => {
                let rhs = stack
                    .pop()
                    .ok_or_else(|| out_of_class("FP binary op: empty stack (rhs)"))?;
                let lhs = stack
                    .pop()
                    .ok_or_else(|| out_of_class("FP binary op: empty stack (lhs)"))?;
                emitted += 1;
                // The final value lands in f1; earlier ones take the next free
                // pool slot, skipping anything still live.
                let dest = if emitted == n_ops {
                    FP_RET
                } else {
                    take_fp(&mut cursor, &live)?
                };
                // Both sources die here unless they are still-live parameters.
                for s in [lhs, rhs] {
                    if s as usize > func.params.len() || s == 0 {
                        live.retain(|&x| x != s);
                    }
                }
                match binop {
                    IlOp::Add => text.extend_from_slice(&encode_fadd(double, dest, lhs, rhs)),
                    // Source order, NOT the integer reversal.
                    IlOp::Sub => text.extend_from_slice(&encode_fsub(double, dest, lhs, rhs)),
                    IlOp::Mul => text.extend_from_slice(&encode_fmul(double, dest, lhs, rhs)),
                    IlOp::Div => text.extend_from_slice(&encode_fdiv(double, dest, lhs, rhs)),
                    IlOp::Load(_)
                    | IlOp::Lit(_)
                    | IlOp::FpLit { .. }
                    | IlOp::LoadInd { .. }
                    | IlOp::LoadIndSized { .. }
                    | IlOp::AddrOf { .. }
                    | IlOp::StoreInd { .. }
                    | IlOp::StoreIndFp { .. } => {
                        unreachable!("not a binary op")
                    }
                }
                if dest != FP_RET {
                    live.push(dest);
                }
                stack.push(dest);
            }
        }
    }
    match stack.as_slice() {
        // Every binary op targets `FP_RET` when it is the last one, so a value
        // sitting anywhere else means the body is a bare `return <param>` whose
        // parameter is not the first — `float f(float a, float b){ return b; }`,
        // which c2 emits as `fmr f1,f2`. Emitting nothing there is wrong bytes,
        // and it *was*: this branch is the second lock on it, matching the one
        // [`select_text`] has carried for the integer identity since that class
        // was written. The parser refuses the shape first (`try_parse_float_leaf`
        // requires every formal to be an FP operand of the body), so nothing
        // should reach here.
        // A bare `return <FP parameter>` whose parameter is not the first FP one:
        // one `fmr` into the result register. `float f(float a, float b)
        // { return b; }` is `fmr f1,f2 ; blr` (captured, `fc201090`), and this
        // branch used to emit **nothing** at all — `GAPS.md` §6's seventh live
        // wrong-bytes emit, the integer identity's `straight_line_out_of_class_ctx`
        // gate missing from the other register file.
        //
        // Reachable only through the parameter list this shape now carries in
        // FP-register order; nothing else can leave a value outside `FP_RET`,
        // because every binary op targets it when it is the last one.
        [r] if *r != FP_RET => {
            text.extend_from_slice(&encode_fmr(FP_RET, *r));
        }
        [_] => {}
        _ => {
            return Err(out_of_class(
                "FP expression did not reduce to a single value; out of class",
            ))
        }
    }
    text.extend_from_slice(&encode_blr());
    Ok((text, consts))
}

/// Select the argument setup for a **single-argument floating-point tail call** —
/// everything before the `b <callee>`, which the caller appends because the
/// branch encodes its own `.text` offset (`Selected::Tail`).
///
/// `params` is the FP formals **alone**, in FP-file order, so entry `n` is
/// `f(n+1)`; `fp.arg` names which of them the call passes. The destination is
/// always `f1`: the argument region held exactly one argument and it is FP, so it
/// is the callee's first floating-point parameter.
///
/// Three cases and one instruction between them, all captured
/// (`docs/CODEGEN_FP_ARGS.md`, and the module doc of
/// `c2_il`'s `shapes::leaf_fp_tail`):
///
/// ```text
///   the value is already in f1        (nothing)   b g
///   it is in fN, same width           fmr  f1,fN
///   it is in fN, callee wants float   frsp f1,fN
/// ```
///
/// The narrowing case is **fused** and that is the row worth having:
/// `float n2(double a, double b){ return g1f(b); }` is the single word
/// `fc201018` — `frsp f1,f2` — and *not* `fmr f1,f2 ; frsp f1,f1`. It is also
/// unconditional: `frsp f1,f1` is emitted even when the source is already f1,
/// because the rounding is the point, not the move.
///
/// This lives beside [`float_leaf_text`] rather than in `codegen/calls.rs`
/// because what it decides is the FP *register file* — the same numbering, the
/// same `f(n+1)` convention, the same `.sy`-derived parameter list — and the one
/// thing it shares with the integer tail call is the branch, which it does not
/// emit.
pub fn fp_tail_call_text(
    params: &[u32],
    fp: &c2_il::FpTail,
) -> Result<Vec<u8>, BackendError> {
    if params.len() > 13 {
        return Err(out_of_class(
            "more than 13 FP parameters: the 14th is stack-homed; out of class",
        ));
    }
    // Parameter n → f(n+1), exactly as `float_leaf_text` maps it. The parser has
    // already established that the argument is one of them; a miss here is a
    // refusal rather than a guessed register, because guessing one is precisely
    // the mis-emit `docs/CODEGEN_FP_ARGS.md` §1 records.
    let src = params
        .iter()
        .position(|&t| t == fp.arg)
        .map(|i| (i + 1) as u8)
        .ok_or_else(|| {
            out_of_class("FP tail-call argument is not one of the FP parameters")
        })?;
    Ok(if fp.narrowing {
        encode_frsp(FP_RET, src).to_vec()
    } else if src == FP_RET {
        Vec::new()
    } else {
        encode_fmr(FP_RET, src).to_vec()
    })
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
    // ---- W13a floating-point leaves ----------------------------------------

    fn fpfunc(params: Vec<u32>, ops: Vec<IlOp>) -> IlFunction {
        let mut f = func_with(params, ops);
        f.float_leaf = Some(false);
        f
    }

    #[test]
    fn float_chain_matches_the_reference() {
        // `float fmul3(float a,float b,float c){ return a*b*c; }` — the live
        // capture is `ec0100b2 ec2000f2 4e800020`:
        //   fmuls f0,f1,f2   (first temp is f0 — the pool's FIRST slot)
        //   fmuls f1,f0,f3   (result forced to f1)
        // Note the multiplier sits in the C field, not B.
        let f = fpfunc(
            vec![0xE309, 0xE409, 0xE509],
            vec![
                IlOp::Load(0xE309),
                IlOp::Load(0xE409),
                IlOp::Mul,
                IlOp::Load(0xE509),
                IlOp::Mul,
            ],
        );
        let (text, consts) = float_leaf_text(&f, false).unwrap();
        assert_eq!(
            text,
            vec![
                0xEC, 0x01, 0x00, 0xB2, // fmuls f0,f1,f2
                0xEC, 0x20, 0x00, 0xF2, // fmuls f1,f0,f3
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
        assert!(consts.is_empty(), "no literals in this body");
    }

    // ---- W13b pooled floating-point constants -------------------------------

    /// IEEE binary64 bits for a value, as the IL carries an FP literal.
    fn f64bits(v: f64) -> u64 {
        v.to_bits()
    }

    #[test]
    fn fp_constant_loads_through_a_relocated_addis_lfs_pair() {
        // `float k_add(float a){ return a + 1.0f; }` — the live capture is
        // `3d600000 c00b0000 ec21002a 4e800020`:
        //   addis r11,r0,0    <- REFHI(__real@3f800000) + PAIR
        //   lfs   f0,0(r11)   <- REFLO(__real@3f800000) + PAIR
        //   fadds f1,f1,f0
        // Both immediates are 0; the linker patches them.
        let f = fpfunc(
            vec![0x09E3],
            vec![
                IlOp::Load(0x09E3),
                IlOp::FpLit { bits: f64bits(1.0), double: false },
                IlOp::Add,
            ],
        );
        let (text, consts) = float_leaf_text(&f, false).unwrap();
        assert_eq!(
            text,
            vec![
                0x3D, 0x60, 0x00, 0x00, // addis r11,r0,0
                0xC0, 0x0B, 0x00, 0x00, // lfs   f0,0(r11)
                0xEC, 0x21, 0x00, 0x2A, // fadds f1,f1,f0
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
        assert_eq!(
            consts,
            vec![FpConstRef { bits: f64bits(1.0), double: false, hi_off: 0 }]
        );
    }

    #[test]
    fn fp_constant_claims_its_register_before_any_interior_temporary() {
        // `ke` in w13b_fpool: c2 folds `a*2.0f*b*3.0f` to `(a*b)*6.0f` and emits
        //   fmuls f13,f1,f2 ; addis r11,r0,0 ; lfs f0,0(r11) ; fmuls f1,f13,f0
        // The interior temp is f13, NOT f0 — so the constant took pool slot 0
        // even though the multiply is *emitted* first. Allocating temporaries in
        // emission order instead would put the multiply in f0 and match every
        // single-op body, which is exactly why this case is pinned.
        let f = fpfunc(
            vec![0x09E3, 0x09E4],
            vec![
                IlOp::Load(0x09E3),
                IlOp::Load(0x09E4),
                IlOp::Mul,
                IlOp::FpLit { bits: f64bits(6.0), double: false },
                IlOp::Mul,
            ],
        );
        let (text, _) = float_leaf_text(&f, false).unwrap();
        assert_eq!(
            text,
            vec![
                0xED, 0xA1, 0x00, 0xB2, // fmuls f13,f1,f2
                0x3D, 0x60, 0x00, 0x00, // addis r11,r0,0
                0xC0, 0x0B, 0x00, 0x00, // lfs   f0,0(r11)
                0xEC, 0x2D, 0x00, 0x32, // fmuls f1,f13,f0
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
    }

    #[test]
    fn double_constant_uses_lfd_and_the_double_primary_opcode() {
        // `double kd(double a){ return a + 1.0; }` →
        //   addis r11,r0,0 ; lfd f0,0(r11) ; fadd f1,f1,f0
        // `lfd` is primary 50 (not 48) and `fadd` primary 63 (not 59).
        let f = {
            let mut g = fpfunc(
                vec![0x09E3],
                vec![
                    IlOp::Load(0x09E3),
                    IlOp::FpLit { bits: f64bits(1.0), double: true },
                    IlOp::Add,
                ],
            );
            g.float_leaf = Some(true);
            g
        };
        let (text, consts) = float_leaf_text(&f, true).unwrap();
        assert_eq!(
            text,
            vec![
                0x3D, 0x60, 0x00, 0x00, // addis r11,r0,0
                0xC8, 0x0B, 0x00, 0x00, // lfd   f0,0(r11)
                0xFC, 0x21, 0x00, 0x2A, // fadd  f1,f1,f0
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
        assert!(consts[0].double);
    }

    #[test]
    fn fp_constant_pool_refuses_what_it_has_not_characterized() {
        // Two constants: c2 hoists both `addis` into a prologue group and
        // schedules the loads at first use, so the REFLO site is no longer
        // `hi_off + 4`. Refuse.
        let two = fpfunc(
            vec![0x09E3, 0x09E4],
            vec![
                IlOp::Load(0x09E3),
                IlOp::FpLit { bits: f64bits(1.0), double: false },
                IlOp::Add,
                IlOp::Load(0x09E4),
                IlOp::FpLit { bits: f64bits(2.0), double: false },
                IlOp::Add,
                IlOp::Sub,
            ],
        );
        assert!(float_leaf_text(&two, false).is_err());

        // A constant divisor strength-reduces to a reciprocal multiply, so a
        // surviving Div against a literal is not something the model expects.
        let div = fpfunc(
            vec![0x09E3],
            vec![
                IlOp::Load(0x09E3),
                IlOp::FpLit { bits: f64bits(3.0), double: false },
                IlOp::Div,
            ],
        );
        assert!(float_leaf_text(&div, false).is_err());

        // A `float` literal whose binary64 pattern does not narrow exactly
        // would pool four bytes that are not the value c2 pooled.
        let inexact = fpfunc(
            vec![0x09E3],
            vec![
                IlOp::Load(0x09E3),
                IlOp::FpLit { bits: f64bits(0.1), double: false },
                IlOp::Add,
            ],
        );
        assert!(float_leaf_text(&inexact, false).is_err());

        // A width mismatch between the literal and the expression implies a
        // conversion the model does not emit.
        let mixed = fpfunc(
            vec![0x09E3],
            vec![
                IlOp::Load(0x09E3),
                IlOp::FpLit { bits: f64bits(1.0), double: true },
                IlOp::Add,
            ],
        );
        assert!(float_leaf_text(&mixed, false).is_err());
    }

    #[test]
    fn fp_subtract_uses_source_order_not_the_integer_reversal() {
        // `fsubs fD,fA,fB` computes fA − fB — the OPPOSITE of encode_subf's
        // load-bearing reversal. Reusing the integer convention here would
        // silently negate every FP subtraction, so pin the operand order.
        assert_eq!(encode_fsub(false, 1, 1, 2), [0xEC, 0x21, 0x10, 0x28]);
        assert_eq!(encode_fadd(false, 1, 1, 2), [0xEC, 0x21, 0x10, 0x2A]);
        assert_eq!(encode_fdiv(false, 1, 1, 2), [0xEC, 0x21, 0x10, 0x24]);
        // Double precision is the same fields under primary opcode 63.
        assert_eq!(encode_fadd(true, 1, 1, 2), [0xFC, 0x21, 0x10, 0x2A]);
    }

    #[test]
    fn fp_rejects_the_shapes_that_would_mis_emit() {
        // A `*` mixed with `+`/`-` CONTRACTS to fmadds/fmsubs in c2, so emitting
        // two instructions would be a silent wrong-bytes emit, not a gap.
        let mixed = fpfunc(
            vec![0xE309, 0xE409, 0xE509],
            vec![
                IlOp::Load(0xE309),
                IlOp::Load(0xE409),
                IlOp::Mul,
                IlOp::Load(0xE509),
                IlOp::Add,
            ],
        );
        assert!(matches!(
            float_leaf_text(&mixed, false),
            Err(BackendError::NotImplemented(_))
        ));
        // An FP literal needs an .rdata COMDAT plus a REFHI/REFLO pair (W13b).
        let lit = fpfunc(
            vec![0xE309],
            vec![IlOp::Load(0xE309), IlOp::Lit(1), IlOp::Mul],
        );
        assert!(matches!(
            float_leaf_text(&lit, false),
            Err(BackendError::NotImplemented(_))
        ));
    }


    /// The FP tail call's whole emission: the move that is elided, the move that
    /// is one `fmr`, and the narrowing that is one **fused** `frsp`.
    ///
    /// Every word is read off a reference obj (`fixtures/cpp/w31_fp_tail.cpp`,
    /// `/O1 /GS- /c`). `n2` is the row that matters: `frsp f1,f2` is `fc201018`,
    /// a single instruction — c2 does not emit `fmr f1,f2` and then round f1, so
    /// a port that composed the two would be wrong by one word on every
    /// narrowing call from anything but f1.
    #[test]
    fn fp_tail_call_emits_the_move_the_capture_shows() {
        let params = vec![0xE309u32, 0xE409, 0xE509];
        // `a1` — the argument is already in f1: a bare `b g1f`, no setup at all.
        assert_eq!(
            fp_tail_call_text(&params, &c2_il::FpTail { arg: 0xE309, narrowing: false }).unwrap(),
            Vec::<u8>::new()
        );
        // `a2` — from f2:  fc201090  fmr f1,f2
        assert_eq!(
            fp_tail_call_text(&params, &c2_il::FpTail { arg: 0xE409, narrowing: false }).unwrap(),
            vec![0xFC, 0x20, 0x10, 0x90]
        );
        // `a3` — from f3:  fc201890  fmr f1,f3
        assert_eq!(
            fp_tail_call_text(&params, &c2_il::FpTail { arg: 0xE509, narrowing: false }).unwrap(),
            vec![0xFC, 0x20, 0x18, 0x90]
        );
        // `n1` — narrowing from f1: still a real instruction, because the
        // ROUNDING is the point and not the move.  fc200818  frsp f1,f1
        assert_eq!(
            fp_tail_call_text(&params, &c2_il::FpTail { arg: 0xE309, narrowing: true }).unwrap(),
            vec![0xFC, 0x20, 0x08, 0x18]
        );
        // `n2` — narrowing from f2, FUSED.  fc201018  frsp f1,f2
        assert_eq!(
            fp_tail_call_text(&params, &c2_il::FpTail { arg: 0xE409, narrowing: true }).unwrap(),
            vec![0xFC, 0x20, 0x10, 0x18]
        );
        // An argument that is not one of the FP parameters is a refusal, never a
        // guessed register: guessing one is exactly the mis-emit
        // `docs/CODEGEN_FP_ARGS.md` §1 records.
        assert!(matches!(
            fp_tail_call_text(&params, &c2_il::FpTail { arg: 0xEE09, narrowing: false }),
            Err(BackendError::NotImplemented(_))
        ));
    }
}
