//! The floating-point leaf: FP arithmetic chains, the `fmr` register move,
//! and the constant pool. See `docs/CODEGEN_W13_FLOAT.md` and
//! `docs/CODEGEN_FP_ARGS.md` — the FP argument register file is numbered over
//! the FP parameters ALONE, which is a different fact from the positional
//! index and was a live mis-emit until W27 separated them.

use c2_il::{
    // **The contraction rule lives in `c2-il` and this file does not restate
    // it.** It is the acceptance gate `try_parse_float_leaf` runs *and* the
    // emission rule below, one implementation — see `c2_il`'s `body::chain`
    // block header for the census/gate failure that forced the placement.
    fp_contract_instructions, fp_node_plan, FpNodePlan, FpValKind, IlFunction, IlOp,
    FP_REASSOCIATED,
};
use crate::BackendError;
use crate::codegen::encode::{
    encode_addis,
    encode_blr,
    encode_fadd,
    encode_fdiv,
    encode_fmadd,
    encode_fmr,
    encode_fmsub,
    encode_fmul,
    encode_fnmsub,
    encode_fsub,
    encode_lfs,
};
use crate::codegen::select::{OptMode, SCRATCH_REG, out_of_class};

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

/// One entry of the evaluation stack: a register, or a **product c2 has not
/// committed to yet**.
#[derive(Clone, Copy, Debug)]
enum FpVal {
    /// A committed value in a register. `from_add` is [`ValKind::from_add`] —
    /// the one bit the reassociation fence reads.
    Reg { r: u8, from_add: bool },
    /// `a * c`, held in c2's own slot order: `a` is the `A` field and `c` the
    /// `C` field of whatever instruction ends up consuming it.
    Prod { a: u8, c: u8 },
}

impl FpVal {
    fn kind(self) -> FpValKind {
        match self {
            FpVal::Reg { from_add, .. } => FpValKind { prod: false, from_add },
            FpVal::Prod { .. } => FpValKind { prod: true, from_add: false },
        }
    }
}

/// **How c2 picks the next FP scratch register — and it is MODE-DEPENDENT.**
///
/// The pool and the skip-if-live rule are the same at both modes. The one
/// difference is whether the cursor **carries** across allocations:
///
/// ```text
///   float f(float a,float b,float c,float d){ return a+b+c+d; }
///   /Ox   fadds f0,f1,f2 ; fadds f13,f0,f3 ; fadds f1,f13,f4
///   /O1   fadds f0,f1,f2 ; fadds f0,f0,f3  ; fadds f1,f0,f4
/// ```
///
/// At `/O1` the temporary **dies into its own successor and is immediately
/// reused**; at `/Ox` the cursor never revisits. **Measured over six shapes and
/// three depths** — `p4`/`p5`/`p6` (four to six `+` leaves), `m4` (four `*`
/// leaves), `y5` (four `-` leaves) and `x5` (a contracted body) — in
/// `work/w-fmadd/repro/deep_O1.cod` and `deep_Ox.cod`, so it is not the
/// two-point fit `#1767` bans.
///
/// **This was a LIVE WRONG EMIT at master `12d3c0558`, on the workload's own
/// mode.** The port implemented `Carried` unconditionally, and
/// `work/dc3-workload/flags.txt` is `/O1 /Oi /EHsc`. It was invisible because
/// **no case in the corpus had ever needed two FP temporaries**: with three
/// leaves there is exactly one intermediate and the two policies agree on it.
/// `scripts/sweep.d/36-fp-contract.py`'s four-leaf chains are the first, and
/// `scripts/mode_cross.sh` reported 108 cells over 17 cases — every one of
/// which mis-emits at master too (`work/w-fmadd/repro/`).
///
/// Shipped as a named parameter rather than an `if mode ==` at the use site,
/// per `docs/GOAL_DECISION_2026-08-21.md` § AMENDED: the default reproduces c2
/// byte-exactly at each mode, and a permuter can search the other value.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FpTempPolicy {
    /// `/Ox`: the cursor carries across allocations, so a freed register is not
    /// revisited until the pool wraps.
    Carried,
    /// `/O1`: the first free register in pool order, every time — so a
    /// temporary that just died is taken again.
    FirstFree,
}

impl FpTempPolicy {
    /// The default for a mode. **This is the only place the mapping lives.**
    pub fn for_mode(mode: OptMode) -> FpTempPolicy {
        match mode {
            OptMode::Ox => FpTempPolicy::Carried,
            OptMode::O1 => FpTempPolicy::FirstFree,
        }
    }
}

/// Pull the next free FP register off the pool, under [`FpTempPolicy`].
///
/// *(A free function since 2026-08-29 — it was a closure over `cursor`/`live`,
/// which the contraction's three materialisation sites could no longer share
/// without fighting the borrow checker. Same body, same order.)*
fn take_fp(
    cursor: &mut usize,
    live: &[u8],
    policy: FpTempPolicy,
) -> Result<u8, BackendError> {
    if policy == FpTempPolicy::FirstFree {
        *cursor = 0;
    }
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
}

/// A source register dies at its use unless it is a still-live parameter.
///
/// The `r == 0` clause is not a special case for `f0`'s number: `f0` is the
/// *first* pool slot and is never a parameter (parameters start at `f1`), so a
/// value in it is always a temporary. It was written this way before this lane
/// and is left alone.
fn retire(live: &mut Vec<u8>, r: u8, nparams: usize) {
    if r as usize > nparams || r == 0 {
        live.retain(|&x| x != r);
    }
}

/// The mutable state the contraction's three emit sites share.
struct Emit<'a> {
    text: &'a mut Vec<u8>,
    live: &'a mut Vec<u8>,
    cursor: &'a mut usize,
    emitted: &'a mut usize,
    n_ops: usize,
    nparams: usize,
    double: bool,
    policy: FpTempPolicy,
}

impl Emit<'_> {
    /// The destination for the instruction about to be emitted: `f1` for the
    /// **last** one, the next free pool slot otherwise.
    fn dest(&mut self) -> Result<u8, BackendError> {
        *self.emitted += 1;
        if *self.emitted == self.n_ops {
            Ok(FP_RET)
        } else {
            take_fp(self.cursor, self.live, self.policy)
        }
    }

    /// **Retire the dying sources and choose the destination, in the order the
    /// policy requires.**
    ///
    /// * `FirstFree` (`/O1`) retires FIRST, because c2 takes the register the
    ///   instruction itself just freed — `fadds f0,f0,f3`.
    /// * `Carried` (`/Ox`) retires LAST, which is the order the port has always
    ///   used and which `/Ox` needs.
    ///
    /// **The two orders are NOT interchangeable under `Carried`, and this lane
    /// claimed they were before a test said otherwise.** They differ exactly at
    /// pool exhaustion: `w13_fscratch::fm13` has thirteen live parameters and
    /// one free slot, so retiring the temporary before the cursor wraps lets
    /// the cursor find it and emit, where c2 (at `/Ox`) does something else.
    /// Retiring early there turned a REFUSAL into a WRONG EMIT —
    /// `census_gate.rs`'s pinned `KNOWN_DISAGREEMENTS_PACKED` went 1 → 0 and
    /// looked like an improvement, and a direct grade of `fm13` at `/Ox` showed
    /// it was not. A refusal outranks a wrong emit
    /// (`docs/PROGRESS_METRIC.md`), so the order is per-policy.
    fn retire_then_dest(&mut self, dying: &[u8]) -> Result<u8, BackendError> {
        if self.policy == FpTempPolicy::FirstFree {
            for &r in dying {
                retire(self.live, r, self.nparams);
            }
            self.dest()
        } else {
            let dest = self.dest()?;
            for &r in dying {
                retire(self.live, r, self.nparams);
            }
            Ok(dest)
        }
    }

    fn finish(&mut self, dest: u8) {
        if dest != FP_RET {
            self.live.push(dest);
        }
    }

    /// Commit a deferred product to an `fmul`, or pass a register through.
    ///
    /// This is the *only* place an `fmul` is emitted now: a `*` node no longer
    /// emits anything itself, because whether it becomes an `fmul` or the `A`/`C`
    /// half of an `fmadd` is decided by its **parent**.
    fn materialise(&mut self, v: FpVal) -> Result<u8, BackendError> {
        match v {
            FpVal::Reg { r, .. } => Ok(r),
            FpVal::Prod { a, c } => {
                let dest = self.retire_then_dest(&[a, c])?;
                self.text
                    .extend_from_slice(&encode_fmul(self.double, dest, a, c));
                self.finish(dest);
                Ok(dest)
            }
        }
    }
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
    mode: OptMode,
) -> Result<(Vec<u8>, Vec<FpConstRef>), BackendError> {
    let policy = FpTempPolicy::for_mode(mode);
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
    // ~~`has_mul && has_addsub` refused here.~~ **STRUCK 2026-08-29, lane
    // `w-fmadd`, board `#3792`.** The contraction is modeled now — see
    // [`NodePlan`] and §3.3 of `docs/CODEGEN_W13_FLOAT.md` — so the mix is in
    // class. `has_addsub` is still read below for the identity-constant gate.
    //
    // **One case of the mix stays refused, and it is not a leftover**: a pooled
    // constant *inside* a contracted expression. `float k(float a, float b)
    // { return a*b + 1.0f; }` puts the constant's `lfs` and the fused `fmadds`
    // in one body, and this port has no witness for the order in which c2
    // schedules the two — the one-constant lowering's whole warrant
    // (`FpConstRef`'s doc) is six bodies in which the `addis`/`lfs` pair sits
    // immediately before its use, and none of them is a fused one. Refusing is
    // one function-shape; guessing is a wrong emit.
    let _ = has_mul;
    if has_mul
        && has_addsub
        && func.ops.iter().any(|o| matches!(o, IlOp::FpLit { .. }))
    {
        return Err(out_of_class(
            "a pooled FP constant inside a contracted multiply-add: the order \
             in which c2 schedules the constant load against the fused \
             instruction has no witness; out of class",
        ));
    }

    // Evaluate the postfix stream over a stack of physical FP registers.
    // **The instruction count, not the binary-node count.** A contracted body
    // emits fewer instructions than it has `*`/`+` nodes, and it is the LAST
    // instruction that targets `f1`.
    let n_ops = fp_contract_instructions(&func.ops).map_err(out_of_class)?;
    let mut emitted = 0usize;
    let mut cursor = 0usize;
    let mut live: Vec<u8> = (1..=func.params.len() as u8).collect();
    let mut stack: Vec<FpVal> = Vec::new();
    let mut text: Vec<u8> = Vec::new();
    let mut consts: Vec<FpConstRef> = Vec::new();
    // Address GPRs for constant loads come off the integer scratch cursor,
    // descending from r11 exactly as the integer selector's do.
    let mut next_addr_gpr: u8 = SCRATCH_REG;

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
        let r = take_fp(&mut cursor, &live, policy)?;
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
                stack.push(FpVal::Reg { r, from_add: false });
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
                // **A deferred product must not float past this load.** The
                // constant emits two instructions of its own, and c2 emits the
                // multiply where the multiply is: `a*2.0f*b*3.0f` folds to
                // `(a*b)*6.0f` and c2 writes `fmuls f13,f1,f2` FIRST, then the
                // `addis`/`lfs` pair. Deferring past it reordered the body and
                // `fp_constant_claims_its_register_before_any_interior_temporary`
                // caught it — a pre-existing test, red on this lane's first
                // draft, which is the neighbouring-guard case the module doc
                // keeps recording.
                if stack.iter().any(|v| matches!(v, FpVal::Prod { .. })) {
                    let mut e = Emit {
                        text: &mut text,
                        live: &mut live,
                        cursor: &mut cursor,
                        emitted: &mut emitted,
                        n_ops,
                        nparams: func.params.len(),
                        double,
                        policy,
                    };
                    for i in 0..stack.len() {
                        if matches!(stack[i], FpVal::Prod { .. }) {
                            let r = e.materialise(stack[i])?;
                            stack[i] = FpVal::Reg { r, from_add: false };
                        }
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
                stack.push(FpVal::Reg { r: fd, from_add: false });
            }
            binop => {
                let rhs = stack
                    .pop()
                    .ok_or_else(|| out_of_class("FP binary op: empty stack (rhs)"))?;
                let lhs = stack
                    .pop()
                    .ok_or_else(|| out_of_class("FP binary op: empty stack (lhs)"))?;
                let plan = fp_node_plan(binop, lhs.kind(), rhs.kind());
                if plan == FpNodePlan::Refuse {
                    return Err(out_of_class(FP_REASSOCIATED));
                }
                let from_add = matches!(binop, IlOp::Add);
                let mut e = Emit {
                    text: &mut text,
                    live: &mut live,
                    cursor: &mut cursor,
                    emitted: &mut emitted,
                    n_ops,
                    nparams: func.params.len(),
                    double,
                    policy,
                };
                match (plan, binop) {
                    // A `*`: emit NOTHING and remember the pair. Whether it
                    // becomes an `fmul` or the `A`/`C` half of a fused
                    // instruction belongs to its parent, and this is the whole
                    // structural change the contraction needed.
                    (FpNodePlan::DeferProduct, _) => {
                        let a = e.materialise(lhs)?;
                        let c = e.materialise(rhs)?;
                        stack.push(FpVal::Prod { a, c });
                    }
                    // The product is the LEFT operand: `A*C ± B` directly.
                    // The right operand is materialised FIRST — that is c2's
                    // own emission order for `a*b + c*d`, which is
                    // `fmuls f0,f3,f4` and only then `fmadds f1,f1,f2,f0`
                    // (`work/w-fmadd/probe/fma2.cod`).
                    (FpNodePlan::FuseLeft, _) | (FpNodePlan::FuseRight, _) => {
                        let (prod, other) = if plan == FpNodePlan::FuseLeft {
                            (lhs, rhs)
                        } else {
                            (rhs, lhs)
                        };
                        let b = e.materialise(other)?;
                        let FpVal::Prod { a, c } = prod else {
                            unreachable!("node_plan chose a side that is not a product")
                        };
                        let dest = e.retire_then_dest(&[a, c, b])?;
                        let w = match (binop, plan) {
                            (IlOp::Add, _) => encode_fmadd(double, dest, a, c, b),
                            // `A*C − B` when the product was written on the
                            // left …
                            (IlOp::Sub, FpNodePlan::FuseLeft) => {
                                encode_fmsub(double, dest, a, c, b)
                            }
                            // … and `B − A*C` when it was on the right. One
                            // opcode apart, opposite sign; this is the branch
                            // the fail axis is about.
                            (IlOp::Sub, _) => encode_fnmsub(double, dest, a, c, b),
                            _ => unreachable!("node_plan fused a non-add/sub node"),
                        };
                        e.text.extend_from_slice(&w);
                        e.finish(dest);
                        stack.push(FpVal::Reg { r: dest, from_add });
                    }
                    (FpNodePlan::Plain, _) => {
                        let l = e.materialise(lhs)?;
                        let r = e.materialise(rhs)?;
                        // Both sources die here unless they are still-live
                        // parameters; WHEN they die relative to the
                        // destination choice is the policy's, not this site's.
                        let dest = e.retire_then_dest(&[l, r])?;
                        match binop {
                            IlOp::Add => e.text.extend_from_slice(&encode_fadd(double, dest, l, r)),
                            // Source order, NOT the integer reversal.
                            IlOp::Sub => e.text.extend_from_slice(&encode_fsub(double, dest, l, r)),
                            IlOp::Div => e.text.extend_from_slice(&encode_fdiv(double, dest, l, r)),
                            // `IlOp::Mul` cannot reach here: `fp_node_plan` maps
                            // every multiply to `DeferProduct`. Named rather
                            // than swept in, like every neighbour below.
                            IlOp::Mul
                            // The BITWISE/SHIFT six have no floating-point form
                            // at all — `a & b` over `float` does not exist in
                            // C++ — and `parse_expr`'s FP path never produces
                            // one, so this is spelled out beside the other
                            // non-binary ops rather than swept into a wildcard.
                            // `lane w-build`.
                            | IlOp::And
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
                            // **Board #1199's carrier.** A bound reference is a
                            // store run's operand; it has no floating-point form
                            // and `parse_expr`'s FP path never produces one.
                            // Named rather than swept into a wildcard, for the
                            // same reason every neighbour above is.
                            | IlOp::BoundAddr { .. }
                            | IlOp::StoreInd { .. }
                            | IlOp::StoreIndFp { .. } => {
                                unreachable!("not a plain FP binary op")
                            }
                        }
                        e.finish(dest);
                        stack.push(FpVal::Reg { r: dest, from_add });
                    }
                    (FpNodePlan::Refuse, _) => unreachable!("refused above"),
                }
            }
        }
    }
    // **A body whose ROOT is a product commits it here** — `a*b*c` ends on an
    // `fmuls` into `f1`, and the deferral would otherwise swallow it. This is
    // the one place `fp_contract_instructions`' trailing `+ 1` is spent, and the
    // two halves are pinned against each other by
    // `the_counter_and_the_emitter_agree`.
    if let Some(FpVal::Prod { .. }) = stack.last() {
        let v = stack.pop().expect("just matched");
        let mut e = Emit {
            text: &mut text,
            live: &mut live,
            cursor: &mut cursor,
            emitted: &mut emitted,
            n_ops,
            nparams: func.params.len(),
            double,
            policy,
        };
        let r = e.materialise(v)?;
        stack.push(FpVal::Reg { r, from_add: false });
    }
    let stack: Vec<u8> = stack
        .iter()
        .map(|v| match v {
            FpVal::Reg { r, .. } => *r,
            FpVal::Prod { .. } => unreachable!("root product was materialised above"),
        })
        .collect();
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
    Ok(crate::codegen::mop::ops_to_bytes(&fp_tail_call_ops(params, fp)?))
}

/// **S1c (i): the FP tail-call setup as an op stream.**
///
/// Zero or one op, and the **zero** case is the load-bearing one: when the
/// argument already sits in the return register there is nothing to move, and
/// this returns the EMPTY stream rather than an empty byte vector. `splice.rs`
/// reads that emptiness as a semantic stratum, so it is worth spelling as "no
/// ops" rather than "no bytes" — the same reasoning the void-tail arm carries.
pub fn fp_tail_call_ops(
    params: &[u32],
    fp: &c2_il::FpTail,
) -> Result<crate::codegen::mop::Ops, BackendError> {
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
    use crate::codegen::encode::{mop_fmr, mop_frsp};
    Ok(if fp.narrowing {
        vec![mop_frsp(FP_RET, src)]
    } else if src == FP_RET {
        Vec::new()
    } else {
        vec![mop_fmr(FP_RET, src)]
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
    Ok(crate::codegen::mop::ops_to_bytes(&fp_permute_args_ops(sources)?))
}

/// **S1c (i): the FP argument permutation as an op stream**, reachable by a
/// caller.
///
/// Every decision here is taken over `sources` and the cycle decomposition --
/// never over the text -- so the conversion is a change of what is appended and
/// of nothing else. The **passthrough** arm returns the EMPTY stream, which is
/// the same load-bearing emptiness the void-tail arm carries.
pub fn fp_permute_args_ops(sources: &[usize]) -> Result<crate::codegen::mop::Ops, BackendError> {
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
    let mut text: crate::codegen::mop::Ops = Vec::new();
    text.push(crate::codegen::encode::mop_fmr(FP_CYCLE_SCRATCH, reg(sources[lowest])));
    // Walk from the parked source back to the minimum: each step writes a
    // destination whose old value has already been consumed. With one minimum
    // this is a single chain and the order is forced.
    let mut dst = sources[lowest];
    while dst != lowest {
        text.push(crate::codegen::encode::mop_fmr(reg(dst), reg(sources[dst])));
        dst = sources[dst];
    }
    text.push(crate::codegen::encode::mop_fmr(reg(lowest), FP_CYCLE_SCRATCH));
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

    /// **The contraction, against the words real `c2.dll` emits.**
    ///
    /// Every expected word below is c2's own, out of its `/FAsc` listing —
    /// `work/w-fmadd/probe/fma.cod`, `fma2.cod` — not this port's output blessed
    /// as a baseline. `fixtures/cpp/w13c_fma.cpp` is the same claim at the obj
    /// level and is the one the byte judge grades; this test is here so a
    /// regression names the instruction rather than a 1,513-byte diff.
    #[test]
    fn the_contraction_emits_the_words_c2_emits() {
        let p = vec![0xE309u32, 0xE409, 0xE509, 0xE609];
        let ld = |i: usize| IlOp::Load(p[i]);
        // `a*b + c` -> `fmadds f1,f1,f2,f3`  (A=a, C=b, B=c: the ADDEND is the
        // B field at bit 11, not the C field at bit 6).
        let cases: &[(&str, Vec<IlOp>, Vec<u32>)] = &[
            (
                "a*b+c",
                vec![ld(0), ld(1), IlOp::Mul, ld(2), IlOp::Add],
                vec![0xEC21_18BA],
            ),
            (
                "a*b-c",
                vec![ld(0), ld(1), IlOp::Mul, ld(2), IlOp::Sub],
                vec![0xEC21_18B8],
            ),
            // The product on the RIGHT of a `-` is `fnmsubs` (`B - A*C`), one
            // opcode away from `fmsubs` and the opposite sign.
            (
                "a-b*c",
                vec![ld(0), ld(1), ld(2), IlOp::Mul, IlOp::Sub],
                vec![0xEC22_08FC],
            ),
            (
                "a+b*c",
                vec![ld(0), ld(1), ld(2), IlOp::Mul, IlOp::Add],
                vec![0xEC22_08FA],
            ),
            // Two products: the LEFT one is fused and the right is materialised
            // into f0 FIRST — c2's own emission order.
            (
                "a*b+c*d",
                vec![ld(0), ld(1), IlOp::Mul, ld(2), ld(3), IlOp::Mul, IlOp::Add],
                vec![0xEC03_0132, 0xEC21_00BA],
            ),
            // A deferred product a second `*` forces out early.
            (
                "a*b*c+d",
                vec![ld(0), ld(1), IlOp::Mul, ld(2), IlOp::Mul, ld(3), IlOp::Add],
                vec![0xEC01_00B2, 0xEC20_20FA],
            ),
            // The fused instruction is not the last one, so f1 goes to the
            // trailing `fadds` and the fusion takes a scratch register.
            (
                "a*b+c+d",
                vec![ld(0), ld(1), IlOp::Mul, ld(2), IlOp::Add, ld(3), IlOp::Add],
                vec![0xEC01_18BA, 0xEC20_202A],
            ),
        ];
        for (name, ops, want) in cases {
            let f = fpfunc(p.clone(), ops.clone());
            let (text, _) = float_leaf_text(&f, false, OptMode::Ox)
                .unwrap_or_else(|e| panic!("{name}: {e:?}"));
            let got: Vec<u32> = text
                .chunks_exact(4)
                .map(|c| u32::from_be_bytes([c[0], c[1], c[2], c[3]]))
                .collect();
            let mut expect = want.clone();
            expect.push(0x4E80_0020); // blr
            assert_eq!(
                got.iter().map(|w| format!("{w:#010x}")).collect::<Vec<_>>(),
                expect.iter().map(|w| format!("{w:#010x}")).collect::<Vec<_>>(),
                "{name}"
            );
        }
    }

    /// **The FP scratch policy is MODE-DEPENDENT, and it was a live wrong emit
    /// on the workload's own mode.**
    ///
    /// Both expected streams are c2's own, out of its `/FAsc` listing at each
    /// mode (`work/w-fmadd/repro/deep_O1.cod`, `deep_Ox.cod`). Depth 1 is
    /// deliberately included and is deliberately IDENTICAL: it is the whole
    /// corpus this project had before 2026-08-29, and it is why nothing caught
    /// the divergence.
    #[test]
    fn the_fp_scratch_policy_follows_the_optimization_mode() {
        let p = vec![0xE309u32, 0xE409, 0xE509, 0xE619];
        let ld = |i: usize| IlOp::Load(p[i]);
        // `a + b + c + d`
        let ops = vec![ld(0), ld(1), IlOp::Add, ld(2), IlOp::Add, ld(3), IlOp::Add];
        let f = fpfunc(p.clone(), ops);
        let words = |m| {
            let (t, _) = float_leaf_text(&f, false, m).unwrap();
            t.chunks_exact(4)
                .map(|c| format!("{:#010x}", u32::from_be_bytes([c[0], c[1], c[2], c[3]])))
                .collect::<Vec<_>>()
        };
        // /Ox: the cursor CARRIES — f0, then f13, then the result f1.
        assert_eq!(
            words(OptMode::Ox),
            ["0xec01102a", "0xeda0182a", "0xec2d202a", "0x4e800020"],
            "/Ox must carry the pool cursor"
        );
        // /O1: the temporary is RECYCLED — f0, f0, then f1.
        assert_eq!(
            words(OptMode::O1),
            ["0xec01102a", "0xec00182a", "0xec20202a", "0x4e800020"],
            "/O1 must reuse the register the instruction just killed"
        );
        // Depth 1 — three leaves, one temporary — is where the two AGREE, and
        // where every fixture in this project lived until this lane.
        let g = fpfunc(p[..3].to_vec(), vec![ld(0), ld(1), IlOp::Add, ld(2), IlOp::Add]);
        let w1 = |m| float_leaf_text(&g, false, m).unwrap().0;
        assert_eq!(
            w1(OptMode::Ox),
            w1(OptMode::O1),
            "a ONE-temporary body cannot separate the policies — this equality is \
             the reason the /O1 mis-emit survived, and it is asserted so that a \
             future change cannot make the two modes differ here without saying so"
        );
    }

    /// **`a + b + c*d` must REFUSE**, and this is the shape that was a live
    /// wrong emit until `scripts/sweep.d/36-fp-contract.py` caught it. c2
    /// reassociates the `+` chain (`fmadds f0,f3,f4,f1 ; fadds f1,f0,f2`) and
    /// this port does not model that.
    #[test]
    fn the_reassociated_plus_chain_refuses() {
        let p = vec![0xE309u32, 0xE409, 0xE509, 0xE609];
        let ld = |i: usize| IlOp::Load(p[i]);
        let f = fpfunc(
            p.clone(),
            vec![ld(0), ld(1), IlOp::Add, ld(2), ld(3), IlOp::Mul, IlOp::Add],
        );
        let e = float_leaf_text(&f, false, OptMode::Ox).expect_err("must refuse");
        assert!(
            format!("{e:?}").contains("REASSOCIATES"),
            "refused for the wrong reason: {e:?}"
        );
        // The neighbour that does NOT reassociate stays in class: the fusing
        // node is a `-`, so the fence must not catch it.
        let g = fpfunc(
            p.clone(),
            vec![ld(0), ld(1), IlOp::Add, ld(2), ld(3), IlOp::Mul, IlOp::Sub],
        );
        assert!(float_leaf_text(&g, false, OptMode::Ox).is_ok(), "the `-` neighbour must stay in class");
    }

    /// **E-#3637: the counter and the emitter are one rule, checked.**
    ///
    /// `fp_contract_instructions` decides which instruction targets `f1`, and the
    /// evaluator emits them; two implementations that agree are invisible for
    /// exactly as long as they agree. This drives both over every op stream of
    /// up to four leaves and requires the count to equal the number of
    /// instructions actually emitted (minus the trailing `blr`).
    #[test]
    fn the_counter_and_the_emitter_agree() {
        let p = vec![0xE309u32, 0xE409, 0xE509, 0xE609];
        let binops = [IlOp::Add, IlOp::Sub, IlOp::Mul, IlOp::Div];
        let mut checked = 0usize;
        let mut emitted_ok = 0usize;
        // Every well-formed postfix stream over 2..=4 ascending leaves.
        for n_leaves in 2..=4usize {
            let n_ops = n_leaves - 1;
            let mut idx = vec![0usize; n_ops];
            loop {
                // Build `L L op L op L op ...` — the left-associated chain —
                // and also the `L L L op op ...` right-leaning ones by varying
                // where the ops fall. Two shapes is enough to cover both the
                // FuseLeft and the FuseRight arms.
                for shape in 0..2u8 {
                    let mut ops: Vec<IlOp> = Vec::new();
                    if shape == 0 {
                        ops.push(IlOp::Load(p[0]));
                        for k in 0..n_ops {
                            ops.push(IlOp::Load(p[k + 1]));
                            ops.push(binops[idx[k]].clone());
                        }
                    } else {
                        for k in 0..n_leaves {
                            ops.push(IlOp::Load(p[k]));
                        }
                        for k in (0..n_ops).rev() {
                            ops.push(binops[idx[k]].clone());
                        }
                    }
                    checked += 1;
                    let want = fp_contract_instructions(&ops);
                    let f = fpfunc(p.clone(), ops.clone());
                    match (want, float_leaf_text(&f, false, OptMode::Ox)) {
                        (Ok(n), Ok((text, _))) => {
                            // One trailing `blr`, four bytes per instruction.
                            let got = text.len() / 4 - 1;
                            assert_eq!(
                                n, got,
                                "counter said {n}, emitter emitted {got}: {ops:?}"
                            );
                            emitted_ok += 1;
                        }
                        // The emitter may refuse for a reason the counter does
                        // not model (no free scratch register, an empty stack);
                        // the counter refusing while the emitter succeeds is
                        // the direction that would be a bug, and it is caught
                        // here.
                        (Err(_), Ok(_)) => {
                            panic!("counter refused but the emitter emitted: {ops:?}")
                        }
                        _ => {}
                    }
                }
                // odometer over the operator choices
                let mut k = 0;
                loop {
                    if k == n_ops {
                        break;
                    }
                    idx[k] += 1;
                    if idx[k] < binops.len() {
                        break;
                    }
                    idx[k] = 0;
                    k += 1;
                }
                if idx.iter().all(|&x| x == 0) {
                    break;
                }
            }
        }
        assert!(checked >= 160, "the enumeration is too small: {checked}");
        assert!(
            emitted_ok >= 40,
            "only {emitted_ok} of {checked} streams emitted — the check is \
             passing because nothing is in class"
        );
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
        let (text, consts) = float_leaf_text(&f, false, OptMode::Ox).unwrap();
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
        let (text, consts) = float_leaf_text(&f, false, OptMode::Ox).unwrap();
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
        let (text, _) = float_leaf_text(&f, false, OptMode::Ox).unwrap();
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
        let (text, consts) = float_leaf_text(&f, true, OptMode::Ox).unwrap();
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
        assert!(float_leaf_text(&two, false, OptMode::Ox).is_err());

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
        assert!(float_leaf_text(&div, false, OptMode::Ox).is_err());

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
        assert!(float_leaf_text(&inexact, false, OptMode::Ox).is_err());

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
        assert!(float_leaf_text(&mixed, false, OptMode::Ox).is_err());
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
        // ~~A `*` mixed with `+`/`-` CONTRACTS to fmadds/fmsubs in c2, so
        // emitting two instructions would be a silent wrong-bytes emit.~~
        // **STRUCK 2026-08-29, lane `w-fmadd`** — the contraction is modeled,
        // so this shape EMITS now, and it emits the single word c2 emits. The
        // assertion is inverted rather than deleted, because the thing it was
        // guarding (that this port never writes `fmuls`+`fadds` here) is still
        // exactly the failure to watch for.
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
        let (text, _) = float_leaf_text(&mixed, false, OptMode::Ox).expect("the contraction is modeled");
        assert_eq!(
            text,
            vec![0xEC, 0x21, 0x18, 0xBA, 0x4E, 0x80, 0x00, 0x20],
            "`a*b+c` must be ONE fmadds, never fmuls+fadds"
        );
        // An FP literal needs an .rdata COMDAT plus a REFHI/REFLO pair (W13b).
        let lit = fpfunc(
            vec![0xE309],
            vec![IlOp::Load(0xE309), IlOp::Lit(1), IlOp::Mul],
        );
        assert!(matches!(
            float_leaf_text(&lit, false, OptMode::Ox),
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

/// **SURFACE[float.contraction]** — the registered decision surface's domain.
///
/// `#3723`'s exact shape, and this lane is the case the board row is about.
/// What this lane adopted is a **new emit**, so a required-zero byte delta
/// cannot grade it at all; what grades it is the byte judge on
/// `fixtures/cpp/w13c_fma.cpp` and the FP sweep. **But both of those reach
/// only what the corpus reaches**, and two things here run past it:
///
/// * **the FPR grid in block 3.** Every FP body this port emits lives in
///   `f0..f13` — parameters are `f1..f13` and the scratch pool is the same
///   file — so no obj in the corpus can distinguish c2's `B` field (bit 11)
///   from its `C` field (bit 6) at a register the parameter numbering cannot
///   produce. `w-encarms`'s **C-C2** is the same failure one form over: it
///   perturbed form 54's SPR high half and **zero byte tests moved**, because
///   every SPR the port names is `< 32`. Block 3 renders the four fused words
///   at `f31/f30/f29/f28` and at `f14/f15`, where a shift or a width defect
///   is unmistakable and where nothing else in the project looks.
/// * **the rule table in block 1.** `fp_node_plan` is a total function of
///   `(op, lhs kind, rhs kind)` and the corpus exercises a handful of its
///   cells; the table renders all of them, so widening the fence — say, by
///   dropping the `from_add` test and letting `a + b + c*d` through again —
///   moves committed text whether or not any fixture covers it.
///
/// Block 2 renders [`fp_contract_instructions`], the half of the rule that decides
/// which instruction lands in `f1`. Its refusals are the streams the lowering
/// declines outright.
///
/// **Block 4 is [`FpTempPolicy`], and it is here because it is the second
/// `#3723` case this lane found and the expensive one.** The two policies
/// agree at depth 1 and diverge at every depth above it, and **every FP body
/// in this project's corpus was depth 1 until 2026-08-29** — three leaves need
/// one temporary. So the byte judge was structurally blind to the choice, and
/// the port shipped `/Ox`'s answer on `/O1`, which is the workload's own mode.
/// The block runs to depth 8 (nine leaves), which nothing generates.
///
/// *(Named `contraction_surface_rows` rather than `surface_rows` like its five
/// siblings for one mechanical reason: `codegen/leaf/mod.rs` glob-re-exports
/// this module and `codegen/mod.rs` glob-re-exports both `leaf` and `frame`, so
/// the sibling name would collide with `frame::surface_rows` and rustc's
/// `ambiguous_glob_reexports` warns. A new warning on the emit path is not
/// worth a naming convention.)*
pub fn contraction_surface_rows() -> Vec<crate::surface::Row> {
    use crate::surface::{Row, REFUSE};
    let mut rows = Vec::new();

    // -- block one: the contraction rule, every cell -------------------------
    //
    // The three operand kinds are the representable ones: a deferred product is
    // never *also* `from_add`, so `(prod, from_add) = (true, true)` is excluded
    // rather than rendered as a cell that cannot arise.
    let kinds: [(&str, FpValKind); 3] = [
        ("leaf", FpValKind { prod: false, from_add: false }),
        ("add", FpValKind { prod: false, from_add: true }),
        ("prod", FpValKind { prod: true, from_add: false }),
    ];
    for (opname, op) in
        [("add", IlOp::Add), ("sub", IlOp::Sub), ("mul", IlOp::Mul), ("div", IlOp::Div)]
    {
        for (ln, lk) in kinds {
            for (rn, rk) in kinds {
                let plan = fp_node_plan(&op, lk, rk);
                let outcome = match plan {
                    FpNodePlan::Refuse => format!("{REFUSE} c2-reassociates-the-plus-chain"),
                    FpNodePlan::DeferProduct => "defer-product".to_string(),
                    FpNodePlan::FuseLeft => match op {
                        IlOp::Add => "fuse-left fmadd".to_string(),
                        _ => "fuse-left fmsub".to_string(),
                    },
                    FpNodePlan::FuseRight => match op {
                        IlOp::Add => "fuse-right fmadd".to_string(),
                        // The asymmetry, rendered so it cannot be changed
                        // quietly: `fnmsub` computes `B - A*C`.
                        _ => "fuse-right fnmsub".to_string(),
                    },
                    FpNodePlan::Plain => "plain".to_string(),
                };
                rows.push(Row::new(format!("plan.{opname}.{ln}.{rn}"), outcome));
            }
        }
    }

    // -- block two: the instruction count, hence which one targets f1 --------
    let ld = |t: u32| IlOp::Load(t);
    let streams: [(&str, &[IlOp]); 17] = [
        ("a*b+c", &[ld(0), ld(1), IlOp::Mul, ld(2), IlOp::Add]),
        ("a*b-c", &[ld(0), ld(1), IlOp::Mul, ld(2), IlOp::Sub]),
        ("a+b*c", &[ld(0), ld(1), ld(2), IlOp::Mul, IlOp::Add]),
        ("a-b*c", &[ld(0), ld(1), ld(2), IlOp::Mul, IlOp::Sub]),
        ("a*b+c*d", &[ld(0), ld(1), IlOp::Mul, ld(2), ld(3), IlOp::Mul, IlOp::Add]),
        ("a*b-c*d", &[ld(0), ld(1), IlOp::Mul, ld(2), ld(3), IlOp::Mul, IlOp::Sub]),
        ("a*b*c+d", &[ld(0), ld(1), IlOp::Mul, ld(2), IlOp::Mul, ld(3), IlOp::Add]),
        ("a*b*c", &[ld(0), ld(1), IlOp::Mul, ld(2), IlOp::Mul]),
        ("a+b+c*d", &[ld(0), ld(1), IlOp::Add, ld(2), ld(3), IlOp::Mul, IlOp::Add]),
        ("a+b-c*d", &[ld(0), ld(1), IlOp::Add, ld(2), ld(3), IlOp::Mul, IlOp::Sub]),
        ("a-b+c*d", &[ld(0), ld(1), IlOp::Sub, ld(2), ld(3), IlOp::Mul, IlOp::Add]),
        ("a*b+c+d*e", &[
            ld(0), ld(1), IlOp::Mul, ld(2), IlOp::Add, ld(3), ld(4), IlOp::Mul, IlOp::Add,
        ]),
        // Four more the fence declines, and the last is the OTHER refusal —
        // a product on both sides of a node that cannot contract, which needs
        // source parentheses to write and therefore has no witness at all.
        ("a+b+c+d*e", &[
            ld(0), ld(1), IlOp::Add, ld(2), IlOp::Add, ld(3), ld(4), IlOp::Mul, IlOp::Add,
        ]),
        ("a-b+c+d*e", &[
            ld(0), ld(1), IlOp::Sub, ld(2), IlOp::Add, ld(3), ld(4), IlOp::Mul, IlOp::Add,
        ]),
        ("a+b+c*d+e", &[
            ld(0), ld(1), IlOp::Add, ld(2), ld(3), IlOp::Mul, IlOp::Add, ld(4), IlOp::Add,
        ]),
        ("a+b+c*d*e", &[
            ld(0), ld(1), IlOp::Add, ld(2), ld(3), IlOp::Mul, ld(4), IlOp::Mul, IlOp::Add,
        ]),
        ("(a*b)*(c*d)", &[
            ld(0), ld(1), IlOp::Mul, ld(2), ld(3), IlOp::Mul, IlOp::Mul,
        ]),
    ];
    for (name, ops) in streams {
        let outcome = match fp_contract_instructions(ops) {
            Ok(n) => format!("instructions={n}"),
            // The REASON is rendered, not just the refusal: the two are
            // different boundaries and a domain that collapsed them would
            // survive replacing one with the other.
            Err(r) if r == FP_REASSOCIATED => format!("{REFUSE} c2-reassociates"),
            Err(_) => format!("{REFUSE} product-on-both-sides"),
        };
        rows.push(Row::new(format!("count.{name}"), outcome));
    }

    // -- block three: the WORDS, at registers the corpus cannot reach --------
    //
    // `(fd, fa, fc, fb)`. The first row is the one every published site uses
    // and is here so a reader can check the domain against
    // `docs/CODEGEN_W13_FLOAT.md` §3.3 by eye; every other row names at least
    // one FPR above `f13`, which no body this port emits can produce.
    for (fd, fa, fc, fb) in [
        (1u8, 1u8, 2u8, 3u8),
        (31, 30, 29, 28),
        (0, 31, 0, 0),
        (0, 0, 31, 0),
        (0, 0, 0, 31),
        (14, 15, 16, 17),
        (28, 29, 30, 31),
        (31, 31, 31, 31),
    ] {
        for double in [false, true] {
            let w = if double { "d" } else { "s" };
            let m = format!("{fd},{fa},{fc},{fb}");
            let hex = |b: [u8; 4]| u32::from_be_bytes(b);
            rows.push(Row::new(
                format!("word.{w}.fmadd.{m}"),
                format!("{:#010x}", hex(encode_fmadd(double, fd, fa, fc, fb))),
            ));
            rows.push(Row::new(
                format!("word.{w}.fmsub.{m}"),
                format!("{:#010x}", hex(encode_fmsub(double, fd, fa, fc, fb))),
            ));
            rows.push(Row::new(
                format!("word.{w}.fnmsub.{m}"),
                format!("{:#010x}", hex(encode_fnmsub(double, fd, fa, fc, fb))),
            ));
        }
    }


    // -- block four: the MODE-DEPENDENT scratch policy, past every corpus ----
    //
    // `take_fp` under each policy, over a chain of `n` intermediates where each
    // temporary dies into its successor — the shape `a + b + … ` has. **Depth 1
    // is where every fixture in this project lived until 2026-08-29**, and the
    // two policies agree there, which is exactly why the `/O1` wrong emit
    // survived: a three-leaf body needs one temporary and one temporary cannot
    // separate `f0,f13,f12,…` from `f0,f0,f0,…`. The rows below run to depth 8,
    // which needs nine leaves and which nothing in the corpus generates.
    for (pname, policy) in [("carried", FpTempPolicy::Carried), ("firstfree", FpTempPolicy::FirstFree)]
    {
        for nparams in [2usize, 4] {
            for depth in 1..=8usize {
                let mut live: Vec<u8> = (1..=nparams as u8).collect();
                let mut cursor = 0usize;
                let mut taken: Vec<String> = Vec::new();
                let mut prev: Option<u8> = None;
                let mut failed = false;
                for _ in 0..depth {
                    // The previous temporary dies into this instruction.
                    if let Some(p) = prev {
                        retire(&mut live, p, nparams);
                    }
                    match take_fp(&mut cursor, &live, policy) {
                        Ok(r) => {
                            taken.push(format!("f{r}"));
                            live.push(r);
                            prev = Some(r);
                        }
                        Err(_) => {
                            failed = true;
                            break;
                        }
                    }
                }
                let outcome = if failed {
                    format!("{REFUSE} pool-exhausted")
                } else {
                    taken.join(" ")
                };
                rows.push(Row::new(
                    format!("temp.{pname}.p{nparams}.d{depth}"),
                    outcome,
                ));
            }
        }
    }

    rows
}
