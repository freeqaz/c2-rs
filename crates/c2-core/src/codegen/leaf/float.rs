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
/// caller rebases it by the function's `.text` offset.
///
/// **`lo_off` is a separate field and not `hi_off + 4`.** It was that constant
/// until `w-biquad`, on the honest ground that every graded site had the load
/// immediately after the `addis`. `Biquad.cpp`'s second pool separates them:
/// B-RULE puts the `lis` at the **top of the dominating block** and the `lfs`
/// **at the use**, and in `?SetCoefficients` those are five words apart
/// (`lis` at 0x10, `lfs` at 0x24, four unrelated `stfs` between them). The
/// arithmetic form would have emitted the REFLO against the third of those
/// stores. `DataRef` had carried the two offsets separately since W-R1 for the
/// same reason, which is the shape of `docs/GAPS.md` §6: one fact with two
/// locators that disagree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FpConstRef {
    /// The constant's value as raw IEEE-754 **binary64** bits (as the IL carries
    /// it), regardless of the reference width.
    pub bits: u64,
    /// True for a `double` (8-byte `.rdata`, `lfd`); false for `float`.
    pub double: bool,
    /// Byte offset of the `addis` within this function's text.
    pub hi_off: u32,
    /// Byte offset of the `lfs`/`lfd` within this function's text — the REFLO
    /// site. Equal to `hi_off + 4` for every straight-line float leaf.
    pub lo_off: u32,
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
                    // The straight-line leaf emits the load immediately after
                    // the `addis`, so the two are adjacent here — stated, not
                    // assumed globally (see the type's doc).
                    lo_off: text.len() as u32 + 4,
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
                    // The BITWISE/SHIFT six have no floating-point form at all
                    // — `a & b` over `float` does not exist in C++ — and
                    // `parse_expr`'s FP path never produces one, so this is
                    // spelled out beside the other non-binary ops rather than
                    // swept into a wildcard. `lane w-build`.
                    IlOp::And
                    | IlOp::Or
                    | IlOp::Xor
                    | IlOp::Shl
                    | IlOp::ShrS
                    | IlOp::ShrU
                    | IlOp::SymAddr(_)
                    | IlOp::Load(_)
                    | IlOp::Lit(_)
                    | IlOp::FpLit { .. }
                    | IlOp::LoadInd { .. }
                    | IlOp::LoadIndSized { .. }
                    | IlOp::LoadIndFp { .. }
                    | IlOp::AddrOf { .. }
                    // **Board #1199's carrier.** A bound reference is a store
                    // run's operand; it has no floating-point form and
                    // `parse_expr`'s FP path never produces one. Named rather
                    // than swept into a wildcard, for the same reason every
                    // neighbour above is.
                    | IlOp::BoundAddr { .. }
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
/// branch encodes its own `.text` offset (`Terminator::TailCall`).
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

/// **WFL — the chain result read through into the FP file**: the one instruction
/// a `CallSeq` ending in [`c2_il::SeqTail::CallLoadFp`] emits after its last
/// `bl`, in exactly the position the integer tail's `lwz` occupies.
///
/// ```text
///   float  m    lfs f1,off(r3)     c0230004
///   double m    lfd f1,off(r3)     c8230010
/// ```
///
/// `double` is the **loaded** width, and that is the whole of the width rule:
/// `lfs` loads *and converts*, so a `float` member returned as a `double` is
/// byte-identical to the unpromoted body and the opcode still follows the member
/// (measured, `work/WFL/probe/p1.cpp`; the IL parser is where the reverse
/// direction is refused).
///
/// This lives here rather than in `codegen/calls.rs` for the reason
/// [`fp_tail_call_text`] does: what it decides is the **FP register file** — that
/// the destination is [`FP_RET`] and not r3 — and the one thing it shares with
/// the call sequence is the frame, which it does not emit. The base register is
/// the caller's, passed in, because the tail's whole premise is that the last
/// call left the pointer in the integer return register.
///
/// **It does not fold at displacement 0**, exactly as the integer `CallLoad` does
/// not: `*(r3 + 0)` is a memory read that has to happen. The address form of the
/// same designator *is* `SeqTail::CallValue`, folds at 0, and never reaches here.
pub fn chain_result_fp_load_text(base: u8, off: i32, double: bool) -> Result<Vec<u8>, BackendError> {
    // D-form, both widths — `lfd` is primary 50 and NOT the DS-form the integer
    // `ld` is, so there is no low-two-bits constraint on the displacement and no
    // alignment gate to mirror from the load leaf. The 16-bit bound is the IL
    // parser's (`mcall-chain-tail-off-wide`); this is the backstop.
    let d = i16::try_from(off).map_err(|_| {
        out_of_class("call-sequence FP tail load offset exceeds a 16-bit displacement")
    })?;
    Ok(encode_lfs(double, FP_RET, base, d).to_vec())
}

/// The FP file's **cycle scratch registers**, in allocation order.
///
/// `docs/CODEGEN_FP_ARGS.md` §1.1 had only `f0`, from a 2-cycle and a 3-cycle —
/// which is exactly the evidence that carried the GPR file's "one r11 breaks the
/// cycle" rule to length 3 and then failed. `f13` is the second, measured over
/// the complete n = 4 grid (`scripts/gt_fpperm.py`, §1.2). Only the first entry
/// is reachable from [`fp_permute_args_text`], which refuses a second scratch;
/// the second is here because the constant it is *not* is the point.
const FP_CYCLE_SCRATCH: u8 = 0;

/// Lower a **multi-argument floating-point tail call**'s argument permutation:
/// the `fmr`s that put each FP argument register's wanted value in place, with
/// `f0` as the single break scratch (W34).
///
/// `sources[i]` is the index, into the FP formals in FP-file order, of the value
/// that FP argument register `f(i+1)` wants — so the identity is the passthrough
/// case and emits nothing. Every argument is floating-point by construction
/// (`c2_il`'s `shapes::leaf_fp_tail`), which is what makes the destination
/// numbering `f1…fn` and what keeps the **other** register file out of it: a call
/// that moved values in both files interleaves their schedules on a rule
/// `docs/CODEGEN_FP_ARGS.md` §1.1 records as uncharacterized.
///
/// The rule, measured over the complete permutation grids at n = 2…5
/// (`scripts/gt_fpperm.py --pure --model`, `docs/CODEGEN_FP_ARGS.md` §1.2):
/// decompose the destination→source map into cycles; each **local minimum** of a
/// cycle parks its source into a scratch and reads it back at the end. This
/// function accepts **one** local minimum — one scratch — where the emission is
/// fully determined and the model is exact on every measured cell. With two, the
/// order the independent chains interleave in is open, in this file exactly as
/// it is in the GPR one (`docs/CODEGEN_ARG_PERM.md` §2.1: 26 of 120 cells at
/// n = 5, and the FP grid refutes the same 26).
///
/// Captured, and each row is a whole function body:
///
/// ```text
///   float f(float a,float b)         { return g2(a,b); }   (nothing)  b g2
///   float f(float a,float b)         { return g2(b,a); }   fmr f0,f2 ; fmr f2,f1 ; fmr f1,f0
///   float f(float a,float b,float c) { return g3(b,c,a); } fmr f0,f2 ; fmr f2,f3 ; fmr f3,f1 ; fmr f1,f0
/// ```
///
/// The primary gate is the IL parser's, so the census and the emitter cannot
/// disagree about what is in class; everything here is the backstop.
pub fn fp_permute_args_text(sources: &[usize]) -> Result<Vec<u8>, BackendError> {
    let n = sources.len();
    if n > 13 {
        return Err(out_of_class(
            "more than 13 FP arguments: the 14th is stack-homed; out of class",
        ));
    }
    for (i, s) in sources.iter().enumerate() {
        if *s >= n {
            // A source FP register the call does not otherwise write: a shift,
            // not a permutation, and the walk below would not terminate on it.
            return Err(out_of_class(
                "FP argument permutation reads a register outside the argument \
                 list; out of class",
            ));
        }
        if sources[..i].contains(s) {
            return Err(out_of_class(
                "an FP argument value is passed twice; out of class",
            ));
        }
    }

    // Cycle-decompose. `sources[i] == i` is a fixed point and needs no move.
    let mut seen = vec![false; n];
    let mut cycles: Vec<Vec<usize>> = Vec::new();
    for start in 0..n {
        if seen[start] || sources[start] == start {
            seen[start] = true;
            continue;
        }
        let mut cycle = Vec::new();
        let mut at = start;
        while !seen[at] {
            seen[at] = true;
            cycle.push(at);
            at = sources[at];
        }
        cycles.push(cycle);
    }
    if cycles.is_empty() {
        return Ok(Vec::new()); // passthrough: every value is already in place
    }
    if cycles.len() > 1 {
        return Err(out_of_class(
            "FP argument permutation has two or more disjoint cycles: c2 parks \
             one scratch per cycle (f0 then f13) and the order the chains \
             interleave in is not characterized; out of class",
        ));
    }
    let cycle = &cycles[0];
    let k = cycle.len();
    let minima: Vec<usize> = (0..k)
        .filter(|&i| cycle[(i + k - 1) % k] > cycle[i] && cycle[i] < cycle[(i + 1) % k])
        .collect();
    if minima.len() != 1 {
        return Err(out_of_class(
            "FP argument permutation has a cycle with two local minima: c2 parks \
             a second scratch (f13) and reorders the writes, and that order is \
             not characterized; out of class",
        ));
    }
    // `f(i+1)` for destination slot `i`, in both roles.
    let reg = |slot: usize| (slot + 1) as u8;
    let lowest = cycle[minima[0]];
    let mut text = Vec::new();
    text.extend_from_slice(&encode_fmr(FP_CYCLE_SCRATCH, reg(sources[lowest])));
    // Walk from the parked source back to the minimum: each step writes a
    // destination whose old value has already been consumed. With one minimum
    // this is a single chain and the order is forced.
    let mut dst = sources[lowest];
    while dst != lowest {
        text.extend_from_slice(&encode_fmr(reg(dst), reg(sources[dst])));
        dst = sources[dst];
    }
    text.extend_from_slice(&encode_fmr(reg(lowest), FP_CYCLE_SCRATCH));
    Ok(text)
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
        f.body = c2_il::BodyShape::FloatLeaf(false);
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
            vec![FpConstRef { bits: f64bits(1.0), double: false, hi_off: 0, lo_off: 4 }]
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
            g.body = c2_il::BodyShape::FloatLeaf(true);
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

    /// W34 — the multi-argument FP tail call's whole emission. Every word is read
    /// off a reference obj (`fixtures/cpp/w34_fp_multi.cpp`, `/O1 /GS- /c`) and
    /// the model is scored on the complete n = 2…5 permutation grid by
    /// `scripts/gt_fpperm.py --pure --model`.
    #[test]
    fn fp_argument_permutation_breaks_its_cycle_through_f0() {
        // The identity: every value is already in place, so a bare `b g`.
        assert_eq!(fp_permute_args_text(&[0, 1]).unwrap(), Vec::<u8>::new());
        assert_eq!(fp_permute_args_text(&[0, 1, 2]).unwrap(), Vec::<u8>::new());
        // `g2f(b,a)` — fc001090 fc400890 fc200090
        assert_eq!(
            fp_permute_args_text(&[1, 0]).unwrap(),
            vec![
                0xFC, 0x00, 0x10, 0x90, // fmr f0,f2
                0xFC, 0x40, 0x08, 0x90, // fmr f2,f1
                0xFC, 0x20, 0x00, 0x90, // fmr f1,f0
            ]
        );
        // `g3f(b,c,a)` — the 3-cycle, and it runs in the direction the cycle
        // happens to go.
        assert_eq!(
            fp_permute_args_text(&[1, 2, 0]).unwrap(),
            vec![
                0xFC, 0x00, 0x10, 0x90, // fmr f0,f2
                0xFC, 0x40, 0x18, 0x90, // fmr f2,f3
                0xFC, 0x60, 0x08, 0x90, // fmr f3,f1
                0xFC, 0x20, 0x00, 0x90, // fmr f1,f0
            ]
        );
        // `g3f(c,a,b)` — the other 3-cycle, which walks the other way.
        assert_eq!(
            fp_permute_args_text(&[2, 0, 1]).unwrap(),
            vec![
                0xFC, 0x00, 0x18, 0x90, // fmr f0,f3
                0xFC, 0x60, 0x10, 0x90, // fmr f3,f2
                0xFC, 0x40, 0x08, 0x90, // fmr f2,f1
                0xFC, 0x20, 0x00, 0x90, // fmr f1,f0
            ]
        );
        // `g3f(a,c,b)` — a fixed point beside a 2-cycle: the scratch is still f0
        // and the untouched destination emits nothing.
        assert_eq!(
            fp_permute_args_text(&[0, 2, 1]).unwrap(),
            vec![
                0xFC, 0x00, 0x18, 0x90, // fmr f0,f3
                0xFC, 0x60, 0x10, 0x90, // fmr f3,f2
                0xFC, 0x40, 0x00, 0x90, // fmr f2,f0
            ]
        );
        // **A 4-cycle, with ONE scratch.** The GPR file's rung stops at three
        // because past three c2 hoists a second temp — but that is a property of
        // the number of local minima, not of the length, and this one is
        // unimodal. `?u4@@YAMMMMM@Z` in the fixture is these five words.
        assert_eq!(
            fp_permute_args_text(&[1, 2, 3, 0]).unwrap(),
            vec![
                0xFC, 0x00, 0x10, 0x90, // fmr f0,f2
                0xFC, 0x40, 0x18, 0x90, // fmr f2,f3
                0xFC, 0x60, 0x20, 0x90, // fmr f3,f4
                0xFC, 0x80, 0x08, 0x90, // fmr f4,f1
                0xFC, 0x20, 0x00, 0x90, // fmr f1,f0
            ]
        );
    }

    /// The refusals, each because a capture shows c2 emits something else.
    #[test]
    fn fp_argument_permutation_refuses_the_two_scratch_shapes() {
        // Two disjoint 2-cycles (`g4f(b,a,d,c)`): c2 parks f0 AND f13 and then
        // has several clobber-free orders to choose between.
        assert!(matches!(
            fp_permute_args_text(&[1, 0, 3, 2]),
            Err(BackendError::NotImplemented(_))
        ));
        // One 4-cycle with a valley (`g4f(c,d,b,a)`): also two scratches, and its
        // unimodal neighbour above is one — which is why the gate counts minima.
        assert!(matches!(
            fp_permute_args_text(&[2, 3, 1, 0]),
            Err(BackendError::NotImplemented(_))
        ));
        // A value passed twice is a copy graph, not a permutation.
        assert!(matches!(
            fp_permute_args_text(&[0, 0]),
            Err(BackendError::NotImplemented(_))
        ));
        // A source outside the destination range is a shift out of a register the
        // call does not otherwise write — and the walk must refuse rather than
        // run off the end, which is how the GPR twin panicked.
        assert!(matches!(
            fp_permute_args_text(&[1, 2]),
            Err(BackendError::NotImplemented(_))
        ));
    }
}
