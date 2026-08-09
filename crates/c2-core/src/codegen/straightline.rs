//! The straight-line integer selector: one unit, deliberately.
//!
//! `select_text`, `combine`, the `Plan`/`Base`/`Operand` machinery and
//! `try_select_depth2_tree` are internally coupled — the r11→r9 cursor and the
//! depth-2 tree rule live and die together, and the `OptMode` split (a chain
//! intermediate whose predecessor is already dead) is one rule spanning all of
//! them. Splitting further would spread one fact across files.

use c2_il::{IlFunction, IlOp};
use crate::BackendError;
use crate::codegen::encode::{
    encode_add,
    encode_addi,
    encode_addis,
    encode_and,
    encode_blr,
    encode_mr,
    encode_mullw,
    encode_or,
    encode_ori,
    encode_slw,
    encode_sraw,
    encode_srw,
    encode_subf,
    encode_xor,
};
use crate::codegen::select::{
    ARG_REGS,
    OptMode,
    RET_REG,
    SCRATCH_REG,
    fits_i16,
    out_of_class,
};

/// Emit `dest = reg + k` as one `addi` (16-bit) or an `addis`+`addi` pair for a
/// wide immediate. The pair splits `k` into a sign-compensated high half and a
/// sign-extended low half: `lo = (i16)k`, `hi = (k − lo) >> 16` (so the `addi`'s
/// sign extension is absorbed). Verified: `a+70000` → `addis r3,r3,1 ; addi
/// r3,r3,4464`; `a-70000` → `addis r3,r3,-1 ; addi r3,r3,-4464`.
fn emit_add_imm(text: &mut Vec<u8>, dest: u8, reg: u8, k: i32) {
    if fits_i16(k) {
        text.extend_from_slice(&encode_addi(dest, reg, k as i16));
    } else {
        let lo = (k & 0xFFFF) as u16 as i16;
        let hi = ((k - lo as i32) >> 16) as i16;
        text.extend_from_slice(&encode_addis(dest, reg, hi));
        text.extend_from_slice(&encode_addi(dest, dest, lo));
    }
}

/// Emit a constant load `dest = k`: `li` (`addi dest,r0,k`) for a 16-bit value,
/// else the `lis`+`ori` idiom (`addis dest,r0,hi ; ori dest,dest,lo`, unsigned
/// halves). Verified: `return 70000` → `addis r3,r0,1 ; ori r3,r3,4464`.
///
/// **…and `lis` ALONE when the low half is zero, which was a live wrong-bytes
/// emit for as long as this function has existed** (found 2026-08-01 by the WLR
/// sweep fragment's value axis, and reproduced on the pre-WLR tree — case
/// `s->a = 65536;` is a run of ONE and therefore nothing to do with that rung).
/// c2 does not emit an `ori` by `0`. MEASURED, `/O1` and `/Ox` identical:
///
/// ```text
///   s->a = 65536;      3d600001                    lis r11,1
///   s->a = 131072;     3d600002                    lis r11,2
///   s->a = 65535;      3d600000 616bffff           lis r11,0 ; ori r11,r11,65535
///   s->a = 100000;     3d600001 616b86a0           lis r11,1 ; ori r11,r11,34464
///   s->a = 2147483647; 3d607fff 616bffff           lis r11,32767 ; ori …
///   s->a = -65536;     3d60ffff                    lis r11,-1   <- still REFUSED below
/// ```
///
/// The high half is emitted even when it is zero (`65535` is `lis r11,0` and an
/// `ori`), so the elision is one-sided and keyed on the LOW half alone. The
/// negative wide case stays refused rather than being widened alongside: c2
/// emits a bare `lis r11,-1` for `-65536`, which this could serve, but `-70000`
/// is unwitnessed here and a fail-closed refusal is not a bug.
pub(crate) fn emit_load_imm(text: &mut Vec<u8>, dest: u8, k: i32) -> Result<(), BackendError> {
    if fits_i16(k) {
        text.extend_from_slice(&encode_addi(dest, 0, k as i16));
    } else if k >= 0 {
        let hi = ((k >> 16) & 0xFFFF) as i16;
        let lo = (k & 0xFFFF) as u16;
        text.extend_from_slice(&encode_addis(dest, 0, hi));
        if lo != 0 {
            text.extend_from_slice(&encode_ori(dest, dest, lo));
        }
    } else {
        return Err(out_of_class("negative wide constant load not yet modeled"));
    }
    Ok(())
}

/// The base of an affine selection-stack value: either a concrete physical
/// register (a loaded parameter) or `Prev` — the running result of the most
/// recent emitted reg-reg instruction. `Prev` resolves to the scratch register
/// r11: any reg-reg result that is *read again* is by construction not the final
/// instruction (the final one lands in r3), so every consumed intermediate lives
/// in r11 (the single-scratch serial-chain invariant).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Base {
    Phys(u8),
    Prev,
}

/// An operand on the selection stack. c2 constant-folds a chain of immediate
/// additions/subtractions (`a + 5 + 5` → `a + 10`, one `addi`), so a value is
/// modeled **affinely** as an optional base register plus a pending immediate
/// offset — the offset accumulates for free and is materialized as a single
/// `addi` (or `addis`+`addi`) only when the value is finalized.
#[derive(Clone, Copy, Debug)]
enum Operand {
    /// A pure integer literal (no register component), not yet materialized.
    Imm(i32),
    /// `base + off`: a register value plus a folded constant offset (`off == 0`
    /// is a bare register). The offset materializes lazily; a reg-reg op
    /// requires `off == 0` (a pending offset there is out of the serial-chain
    /// class → fail closed).
    RegOff { base: Base, off: i32 },
}

/// One planned emission, in evaluation order. The **destination** register is
/// resolved by position at emit time — the last plan entry targets the return
/// register r3, every earlier one the scratch r11 — so folding that removes an
/// emission automatically re-targets the survivor (e.g. the single folded
/// `addi r3,r3,10` for `a + 5 + 5`) without a separate counter.
#[derive(Clone, Copy, Debug)]
enum PlanOp {
    /// A binary op over unresolved operand *bases*; `Sub` keeps its load-bearing
    /// operand order. Bases stay symbolic until emission because `Base::Prev`
    /// resolves to whichever register the previous result was placed in, and
    /// that is no longer always r11 (see [`select_text`]'s allocator).
    Bin { op: IlOp, lhs: Base, rhs: Base },
    /// Materialize a pending offset: `dest = src + k` (`addi`, or `addis`+`addi`
    /// when wide). The final flush of an affine `reg + off` value.
    AddImm { src: Base, k: i32 },
    /// Materialize a bare constant return: `dest = k` (`li`, or `lis`+`ori`).
    LoadImm { k: i32 },
    /// Materialize a bare `return <param>` whose parameter is not the first:
    /// `dest = src`, one `mr` (`or dest,src,src`). Only ever the last entry, so
    /// `dest` is r3.
    RegMove { src: u8 },
}

/// Select `.text` bytes for a straight-line integer-arithmetic function
/// (`+`, `-`, `*`; no branches/calls/relocations).
///
/// Params are pre-colored to the incoming ABI argument registers by position
/// (a→r3, b→r4, c→r5, …). The postfix `LOAD`/binary-op stream is walked over an
/// operand stack of physical registers: each binary op pops rhs then lhs and
/// emits its instruction into `dest` — the **final** binary op targets the
/// return register r3, every earlier one targets the running scratch r11. A
/// trailing `blr` returns.
///
/// Operand-order handling per op (the correctness-critical part):
/// * `Add` → `add dest, lhs, rhs` — commutative, order match-neutral.
/// * `Mul` → `mullw dest, lhs, rhs` — commutative, order match-neutral.
/// * `Sub` → `subf dest, rhs, lhs` — **non-commutative**. `subf` computes
///   `rB - rA`, so realizing `lhs - rhs` requires `rA = rhs`, `rB = lhs`; this
///   is the exact reversed mapping the reference c2 emits (`a-b-c` →
///   `subf r11,r4,r3 ; subf r3,r5,r11`). A swap here would be a fuzzy-invisible
///   sign inversion (CLAUDE.md correctness boundary) — see [`encode_subf`].
/// Try to select a **depth-2 expression tree** `(a op b) root (c op d)` over
/// four distinct parameter leaves (W5 trees).
///
/// The operand stack reaches depth 3 here, so the serial-chain selector cannot
/// express it. c2 lowers it as an actual tree: left child into one scratch,
/// right child into another, then the root into r3.
///
/// ```text
///   (a+b)*(c+d)   add   r11,r3,r4 ; add   r10,r5,r6 ; mullw r3,r11,r10
///   (a*b)-(c*d)   mullw r11,r3,r4 ; mullw r10,r5,r6 ; subf  r3,r10,r11
///   (a*b)+(c*d)   mullw r10,r3,r4 ; mullw r11,r5,r6 ; add   r3,r10,r11
/// ```
///
/// Note the third line: **when the root is `+` the two children's registers are
/// swapped** relative to every other root operator. That is reproducible and
/// order-independent — `(a*b)+(c*d)` and `(c*d)+(a*b)` are byte-identical, so c2
/// canonicalizes the commutative root by parameter order and then gives the
/// first term r10. The mechanism is not understood, only characterized, which is
/// why the `+` root is accepted at *exactly* this depth and nowhere else.
///
/// Four gates, each a shape where c2 does **not** lower the source tree as a
/// tree and a post-order selector would emit plausible, wrong bytes:
///
/// * a `*` node with a `*` child collapses into one n-ary product and is
///   re-linearized into a chain — `(a*b)*(c*d)`, `(a+b)*(c*d)` and `a*(b*(c*d))`
///   all compile to the *same* chain, none of them the source pairing;
/// * a `+`/`-` node with a `+`/`-` child collects into one n-ary sum whose terms
///   are reordered (subtracted first) — `(a+b)-(c+d)` emits its leaves in the
///   order `a, c, d, b`;
/// * any immediate on an additive node (its register order is unexplained);
/// * anything but four distinct parameter leaves.
fn try_select_depth2_tree(
    func: &IlFunction,
    reg_of: &dyn Fn(u32) -> Option<u8>,
) -> Option<Vec<u8>> {
    // Which shape this is — and whether it is a shape at all — is decided by
    // `c2_il::chain_form`, the SAME predicate the IL parser gates on. The four
    // distinct-formal / N1 / N2 / division rules used to be spelled out twice,
    // here and (partly) in the parser; that is how the depth rule ended up
    // enforced only here, with the census claiming bodies the port refused.
    if c2_il::chain_form(&func.ops, &func.params) != Some(c2_il::ChainForm::Depth2Tree) {
        return None;
    }
    let (l0, l1, op1, l2, l3, op2, root) = match func.ops.as_slice() {
        [IlOp::Load(a), IlOp::Load(b), o1, IlOp::Load(c), IlOp::Load(d), o2, r] => {
            (*a, *b, *o1, *c, *d, *o2, *r)
        }
        // `chain_form` already proved the shape; this is the destructuring.
        _ => return None,
    };
    let regs: Vec<u8> = [l0, l1, l2, l3]
        .iter()
        .map(|t| reg_of(*t))
        .collect::<Option<_>>()?;

    // The `+`-root swap.
    let (left_reg, right_reg) = if root == IlOp::Add {
        (SCRATCH_REG - 1, SCRATCH_REG) // r10, r11
    } else {
        (SCRATCH_REG, SCRATCH_REG - 1) // r11, r10
    };

    let emit = |out: &mut Vec<u8>, op: IlOp, dest: u8, lhs: u8, rhs: u8| match op {
        IlOp::Add => out.extend_from_slice(&encode_add(dest, lhs, rhs)),
        IlOp::Mul => out.extend_from_slice(&encode_mullw(dest, lhs, rhs)),
        // `subf` computes rB − rA, so `lhs − rhs` needs rA=rhs, rB=lhs.
        IlOp::Sub => out.extend_from_slice(&encode_subf(dest, rhs, lhs)),
        _ => unreachable!("gated above"),
    };

    let mut text = Vec::with_capacity(16);
    // Left child first, always — only the register assignment swaps.
    emit(&mut text, op1, left_reg, regs[0], regs[1]);
    emit(&mut text, op2, right_reg, regs[2], regs[3]);
    emit(&mut text, root, RET_REG, left_reg, right_reg);
    text.extend_from_slice(&encode_blr());
    Some(text)
}

pub fn select_text(func: &IlFunction, mode: OptMode) -> Result<Vec<u8>, BackendError> {
    // Out-of-class, not a pass failure. As `BackendError::Pass` this landed in the
    // harness's `port-error` bucket while every other refusal in `codegen` landed in
    // `codegen-gap`, and `differential` coerced it to `NotImplemented` anyway — so
    // the two instruments classified the same function differently. The parser now
    // refuses this shape first (`straight_line_is_out_of_class`); this stays as the
    // backstop.
    if func.params.len() > ARG_REGS.len() {
        return Err(out_of_class(&format!(
            "more than {} register arguments ({}): the rest are stack-homed and need \
             a frame; out of class",
            ARG_REGS.len(),
            func.params.len()
        )));
    }

    // token -> incoming ABI register, by declaration order.
    let reg_of = |tok: u32| -> Option<u8> {
        func.params
            .iter()
            .position(|&t| t == tok)
            .map(|i| ARG_REGS[i])
    };

    // A depth-2 tree is not a serial chain and the affine selector below cannot
    // express it; try the dedicated tree shape first.
    if let Some(text) = try_select_depth2_tree(func, &reg_of) {
        return Ok(text);
    }

    // Capacity only (the ops stream bounds both): no behavior change.
    let mut stack: Vec<Operand> = Vec::with_capacity(4);
    let mut plan: Vec<PlanOp> = Vec::with_capacity(func.ops.len() / 2 + 2);

    for op in &func.ops {
        match op {
            IlOp::Load(tok) => {
                let reg = reg_of(*tok).ok_or_else(|| BackendError::Pass {
                    pass: "codegen".into(),
                    msg: format!("LOAD of unknown token 0x{tok:04X} (not a parameter)"),
                })?;
                stack.push(Operand::RegOff { base: Base::Phys(reg), off: 0 });
            }
            IlOp::Lit(k) => stack.push(Operand::Imm(*k)),
            // An FP constant only ever appears in an FP expression, which
            // `float_leaf_text` owns; reaching the integer selector means the
            // classifier disagreed with the parser.
            IlOp::FpLit { .. } => {
                return Err(out_of_class(
                    "floating-point constant in an integer expression; out of class",
                ))
            }
            // Integer division is not modeled (`divw`/`divwu`, and a constant
            // divisor strength-reduces to a multiply-high). FP division reaches
            // `float_leaf_text` instead and never gets here.
            IlOp::Div => return Err(out_of_class("integer division; out of class")),
            // WR1: a named data symbol's address only ever appears as a whole
            // CALL ARGUMENT, where it is `lis`+`addi` and a relocation quad that
            // `permute_args_parts` owns. Reaching the affine selector would mean
            // lowering it as an ordinary register operand, which it is not.
            IlOp::SymAddr(_) => {
                return Err(out_of_class(
                    "a data symbol's address feeding arithmetic; out of class",
                ))
            }
            // An indirect load only ever appears as the whole body of an
            // indirect-load leaf, which `indirect_load_text` owns. Reaching the
            // affine selector would mean lowering `*p` as if it were a register
            // operand — and c2 does not: the load lands in the scratch register
            // and the arithmetic reads it from there.
            IlOp::LoadInd { .. } | IlOp::LoadIndSized { .. }
                    | IlOp::LoadIndFp { .. } => {
                return Err(out_of_class(
                    "indirect load feeding arithmetic; out of class",
                ))
            }
            // A sub-object address only ever appears as the whole body of an
            // address leaf, which `addr_leaf_text` owns. An address that feeds an
            // integer expression would have to be converted to an integer first,
            // and no capture establishes that lowering.
            IlOp::AddrOf { .. } => {
                return Err(out_of_class(
                    "sub-object address feeding arithmetic; out of class",
                ))
            }
            // **Board #1199's carrier.** A bound reference (`auto& l = m;`) is
            // a store run's operand and nothing else: it names a store's base
            // symbol and carries the address that base is computed from. In an
            // arithmetic expression it would have to be materialised as an
            // `addi` feeding the chain, and no capture establishes that — the
            // one place it materialises at all is a store's VALUE position,
            // which is `leaf::store`'s and is refused there by name.
            IlOp::BoundAddr { .. } => {
                return Err(out_of_class(
                    "a bound reference feeding arithmetic; out of class",
                ))
            }
            // An indirect store only ever appears as the last op of a store
            // leaf, which `store_leaf_text` owns. A store is not a value at
            // all — reaching the affine selector would mean pushing one onto
            // the operand stack.
            IlOp::StoreInd { .. } | IlOp::StoreIndFp { .. } => {
                return Err(out_of_class(
                    "indirect store in an expression; out of class",
                ))
            }
            // The nine binary integer operators — the arithmetic three and the
            // bitwise/shift six (`lane w-build`). Spelled out rather than
            // written as `op if op.is_binary_int()` so the match stays
            // EXHAUSTIVE: a guard does not satisfy the exhaustiveness checker,
            // and a wildcard here would let the next `IlOp` variant reach the
            // affine selector silently. The `debug_assert` keeps the list and
            // the predicate from drifting — `chain_form`'s stack simulation
            // gates on the predicate, and the two must name one set.
            op @ (IlOp::Add
            | IlOp::Sub
            | IlOp::Mul
            | IlOp::And
            | IlOp::Or
            | IlOp::Xor
            | IlOp::Shl
            | IlOp::ShrS
            | IlOp::ShrU) => {
                debug_assert!(
                    op.is_binary_int(),
                    "the selector's binary arm and IlOp::is_binary_int disagree"
                );
                // Binary op: pop rhs then lhs.
                let rhs = stack.pop().ok_or_else(|| out_of_class("binary op: empty stack (rhs)"))?;
                let lhs = stack.pop().ok_or_else(|| out_of_class("binary op: empty stack (lhs)"))?;
                let result = combine(*op, lhs, rhs, &mut plan)?;
                stack.push(result);
            }
        }
        // Single-scratch (r11) selection is correct only for a **serial
        // accumulator chain** (operand stack depth ≤ 2: one running result +
        // one fresh operand). A tree like `(a+b)*(c+d)` reaches depth 3 and
        // needs a second scratch; emitting it with one would silently clobber
        // the first result. Reject as out-of-class rather than mis-emit.
        if stack.len() > 2 {
            return Err(out_of_class(
                "expression is not a serial accumulator chain (operand stack \
                 depth > 2 → needs more than one scratch register); outside the \
                 current straight-line class",
            ));
        }
    }

    // Finalize the single remaining value into the return register r3. A pending
    // offset (or a bare literal) becomes the last plan entry, so it materializes
    // into r3 (see [`PlanOp`] dest resolution).
    match stack.as_slice() {
        [Operand::RegOff { base, off }] => {
            if *off != 0 {
                plan.push(PlanOp::AddImm { src: *base, k: *off });
            } else {
                match base {
                    // Chain already ended in r3 (the last reg-reg op targets it),
                    // or a bare `return a` where the parameter is already in r3.
                    Base::Prev | Base::Phys(RET_REG) => {}
                    // A bare `return <param>` whose value is not in r3 — the
                    // whole body is one register move. MEASURED across every
                    // argument slot and both scalar widths (`w18_reg_move.cpp`):
                    //
                    //   int f(int a,int b)         { return b; }  7c832378 mr r3,r4
                    //   int f(int a,int b,int c)   { return c; }  7ca32b78 mr r3,r5
                    //   int C::m(int x,int y) const{ return y; }  7ca32b78 mr r3,r5
                    //   S*  f(int a, S* s)         { return s; }  7c832378 mr r3,r4
                    //   int f(…8 params…)          { return h; }  7d435378 mr r3,r10
                    //
                    // and then `blr`. The move is the same instruction for an
                    // int, an unsigned, a short, a `long long` and a pointer —
                    // one 4-byte word in one GPR, no extension anywhere — which
                    // is what lets one arm serve all of them. `this` is already
                    // at index 0 of `func.params`, so a member function's first
                    // explicit formal is r4 without a second rule.
                    //
                    // The FP file has the same shape and is NOT this arm:
                    // `float f(float a,float b){return b;}` is `fmr f1,f2`, and
                    // `float_leaf_text` refuses it because the FP-argument index
                    // cannot be derived from the positional one (see there).
                    Base::Phys(other) => {
                        plan.push(PlanOp::RegMove { src: *other });
                    }
                }
            }
        }
        [Operand::Imm(k)] => {
            // Bare constant return, e.g. `return 42;` → `li r3,k`; wide → lis+ori.
            plan.push(PlanOp::LoadImm { k: *k });
        }
        _ => {
            return Err(out_of_class(
                "expression did not reduce to a single value (malformed or out of class)",
            ))
        }
    }

    // Emit the plan. The **last** entry targets the return register r3. For the
    // earlier ones the destination depends on the op, because c2 does NOT use a
    // single scratch for every chain (verified against live captures):
    //
    //   a+b+c+d  ->  add   r11,r3,r4 ; add   r11,r11,r5 ; add   r3,r11,r6
    //   a*b*c*d  ->  mullw r11,r3,r4 ; mullw r10,r11,r5 ; mullw r3,r10,r6
    //   a-b-c-d  ->  subf  r11,r4,r3 ; subf  r10,r5,r11 ; subf  r3,r6,r10
    //
    // An additive chain collapses to one running accumulator (r11 reused), while
    // a `*`/`-` chain gives every intermediate its own register, descending from
    // r11. The two rules coincide at exactly one intermediate — which is why
    // every fixture up to `a-b-c` matched while `a-b-c-d` silently mis-emitted.
    //
    // `Base::Prev` therefore resolves to the previous entry's ACTUAL destination
    // rather than to a fixed r11; that is why plan operands stay symbolic until
    // here.
    // Every plan entry emits at most 8 bytes (wide immediates), plus the `blr`.
    let mut text: Vec<u8> = Vec::with_capacity(plan.len() * 8 + 4);
    let last = plan.len().saturating_sub(1);
    let mut next_scratch: u8 = SCRATCH_REG;
    let mut prev_reg: u8 = SCRATCH_REG;
    // The accumulator decision is made once for the WHOLE chain, not per operation.
    // If the chain contains any addition, every intermediate reuses r11 — including
    // the subtractions ahead of that addition. Only a chain with no addition at all
    // gives each intermediate its own descending register.
    //
    // Deciding per-operation instead was a mis-emit found by the generated 4-leaf
    // sweep (270 cases): `a + b - c - d` emits `subf r11,r5,r3 ; subf r11,r6,r11 ;
    // add r3,r11,r4` — r11 twice, even though both of those are subtractions —
    // against `a - b - c - d`, which really does descend
    // `subf r11 ; subf r10 ; subf r3`. The two rules coincide at one intermediate,
    // which is why every 3-leaf chain matched and only 4 leaves exposed it.
    //
    // **All of the above is the `/Ox` rule.** Under `/O1` (favour size) there is no
    // descending case at all: this plan is a serial chain, so every intermediate's
    // predecessor is dead by construction, and `/O1` reuses r11 for a dead
    // predecessor unconditionally. `a - b - c - d` is
    // `subf r11,r4,r3 ; subf r11,r5,r11 ; subf r3,r6,r11` — where `/Ox` descends
    // r11, r10, r3 — and the operator-dependence disappears with it. Enumerated
    // over all 108 three- and four-operator chains: only register fields differ,
    // never an opcode or an operand order.
    let chain_has_add = plan
        .iter()
        .any(|e| matches!(e, PlanOp::Bin { op: IlOp::Add, .. } | PlanOp::AddImm { .. }));
    for (i, entry) in plan.iter().enumerate() {
        let dest = if i == last {
            RET_REG
        } else if mode == OptMode::O1 || chain_has_add {
            SCRATCH_REG
        } else {
            match entry {
                // Unreachable while `chain_has_add` is false, but keeps the arm
                // exhaustive and the intent local.
                PlanOp::Bin { op: IlOp::Add, .. } | PlanOp::AddImm { .. } => SCRATCH_REG,
                _ => {
                    let d = next_scratch;
                    // Observed descending allocation covers r11, r10, r9 (the
                    // deepest characterized chain is `a*b*c*d*e`). Below that is
                    // extrapolation, and c2's allocator is demonstrably richer
                    // outside this class — it reuses dead registers and it
                    // schedules — so refuse rather than guess.
                    if d < 9 {
                        return Err(out_of_class(
                            "expression chain needs more scratch registers than the \
                             characterized descending range r11..r9; out of class",
                        ));
                    }
                    next_scratch = d - 1;
                    d
                }
            }
        };
        let resolve = |b: Base| -> u8 {
            match b {
                Base::Phys(r) => r,
                Base::Prev => prev_reg,
            }
        };
        match *entry {
            PlanOp::Bin { op, lhs, rhs } => {
                let (l, r) = (resolve(lhs), resolve(rhs));
                match op {
                    IlOp::Add => text.extend_from_slice(&encode_add(dest, l, r)),
                    IlOp::Mul => text.extend_from_slice(&encode_mullw(dest, l, r)),
                    // `subf` computes rB − rA, so realizing `lhs − rhs` needs
                    // rA=rhs, rB=lhs (the load-bearing reversed order — see
                    // [`encode_subf`]).
                    IlOp::Sub => text.extend_from_slice(&encode_subf(dest, r, l)),
                    // The bitwise/shift six (`lane w-build`). Every one is an
                    // X-form whose DESTINATION is the RA field, not the RT
                    // field the three above use — see [`encode_and`]. `l` is
                    // the left operand and `r` the right, and for the three
                    // shifts that order is load-bearing in the same way
                    // `encode_subf`'s is.
                    IlOp::And => text.extend_from_slice(&encode_and(dest, l, r)),
                    IlOp::Or => text.extend_from_slice(&encode_or(dest, l, r)),
                    IlOp::Xor => text.extend_from_slice(&encode_xor(dest, l, r)),
                    IlOp::Shl => text.extend_from_slice(&encode_slw(dest, l, r)),
                    IlOp::ShrS => text.extend_from_slice(&encode_sraw(dest, l, r)),
                    IlOp::ShrU => text.extend_from_slice(&encode_srw(dest, l, r)),
                    // `combine` never records a Div plan entry (it rejects
                    // first), so reaching here would be an internal error.
                    IlOp::Div
                    | IlOp::SymAddr(_)
                    | IlOp::Load(_)
                    | IlOp::Lit(_)
                    | IlOp::FpLit { .. }
                    | IlOp::LoadInd { .. }
                    | IlOp::LoadIndSized { .. }
                    | IlOp::LoadIndFp { .. }
                    | IlOp::AddrOf { .. }
                    // Board #1199's carrier, refused one layer up in the op
                    // walk and named here for the same reason its neighbours
                    // are: the list stays exhaustive so the next variant cannot
                    // reach the encoder through a wildcard.
                    | IlOp::BoundAddr { .. }
                    | IlOp::StoreInd { .. }
                    | IlOp::StoreIndFp { .. } => {
                        unreachable!("not a modeled integer binary op")
                    }
                }
            }
            PlanOp::AddImm { src, k } => emit_add_imm(&mut text, dest, resolve(src), k),
            PlanOp::LoadImm { k } => emit_load_imm(&mut text, dest, k)?,
            PlanOp::RegMove { src } => text.extend_from_slice(&encode_mr(dest, src)),
        }
        prev_reg = dest;
    }

    text.extend_from_slice(&encode_blr());
    Ok(text)
}

/// Fold one binary op over the affine operand stack, recording a [`PlanOp`] only
/// when a register instruction is actually needed. Immediate accumulations fold
/// for free (`a + 5 + 5` → `a + 10`, matching c2's constant folding); a reg-reg
/// op requires both operands to be bare registers (`off == 0`) — a pending
/// offset there is outside the serial-chain class and fails closed. Rejects the
/// shapes needing an instruction this class does not model (immediate multiply →
/// strength reduction; `imm - reg` → `subfic`).
fn combine(
    op: IlOp,
    lhs: Operand,
    rhs: Operand,
    plan: &mut Vec<PlanOp>,
) -> Result<Operand, BackendError> {
    use Operand::{Imm, RegOff};

    // Emit a reg-reg instruction and return its running result (r11 via `Prev`).
    let mut emit_reg_reg = |op: IlOp, a: Base, b: Base| -> Result<Operand, BackendError> {
        plan.push(PlanOp::Bin { op, lhs: a, rhs: b });
        Ok(RegOff { base: Base::Prev, off: 0 })
    };

    match (op, lhs, rhs) {
        // ---- Add (commutative) ------------------------------------------------
        (IlOp::Add, Imm(a), Imm(b)) => Ok(Imm(a
            .checked_add(b)
            .ok_or_else(|| out_of_class("constant add overflow"))?)),
        (IlOp::Add, RegOff { base, off }, Imm(k)) | (IlOp::Add, Imm(k), RegOff { base, off }) => {
            let off = off
                .checked_add(k)
                .ok_or_else(|| out_of_class("folded add-immediate overflow"))?;
            Ok(RegOff { base, off })
        }
        (IlOp::Add, RegOff { base: a, off: 0 }, RegOff { base: b, off: 0 }) => {
            emit_reg_reg(IlOp::Add, a, b)
        }
        (IlOp::Add, RegOff { .. }, RegOff { .. }) => Err(out_of_class(
            "reg+reg add with a pending immediate offset (non-serial chain); out of class",
        )),

        // ---- Sub (`lhs - rhs`, NON-commutative) -------------------------------
        (IlOp::Sub, Imm(a), Imm(b)) => Ok(Imm(a
            .checked_sub(b)
            .ok_or_else(|| out_of_class("constant sub overflow"))?)),
        // reg − imm folds by *subtracting* into the running offset (no negate,
        // no INT_MIN hazard — `emit_add_imm` handles the sign at materialization).
        (IlOp::Sub, RegOff { base, off }, Imm(k)) => {
            let off = off
                .checked_sub(k)
                .ok_or_else(|| out_of_class("folded sub-immediate overflow"))?;
            Ok(RegOff { base, off })
        }
        (IlOp::Sub, Imm(_), RegOff { .. }) => {
            Err(out_of_class("`const - reg` needs subfic; out of class"))
        }
        (IlOp::Sub, RegOff { base: a, off: 0 }, RegOff { base: b, off: 0 }) => {
            emit_reg_reg(IlOp::Sub, a, b)
        }
        (IlOp::Sub, RegOff { .. }, RegOff { .. }) => Err(out_of_class(
            "reg-reg subtract with a pending immediate offset (non-serial chain); out of class",
        )),

        // ---- Mul (commutative) ------------------------------------------------
        (IlOp::Mul, RegOff { base: a, off: 0 }, RegOff { base: b, off: 0 }) => {
            emit_reg_reg(IlOp::Mul, a, b)
        }
        // reg*const strength-reduces, and const*const is unexpected (c1xx folds).
        (IlOp::Mul, _, _) => Err(out_of_class(
            "multiply by a constant strength-reduces (shift/add); out of class",
        )),

        // ---- The bitwise/shift six — REGISTER-REGISTER ONLY (`lane w-build`) --
        //
        // `&`, `|`, `^` are commutative; `<<`, `>>` are not, and `emit_reg_reg`
        // preserves `(lhs, rhs)` for all six, so the shifts get their operands
        // in source order.
        //
        // **Every other operand form refuses**, and the arm below is one arm on
        // purpose rather than six copies of `Mul`'s: what the immediate forms
        // cost is one measured statement, not six. `IlOp::And`'s doc comment
        // carries the probe. In one sentence: `a & 1` is `clrlwi`, `a & 5` is
        // `andi.` (record-form, it writes CR0), `a & 0x12345` is a three
        // instruction materialization through **r12**, `a | 0x12345` is
        // `oris`+`ori`, and `256 >> a` materializes the literal into r11 with
        // `li` first. Three instruction families and two scratch registers
        // across one axis, selected by a predicate over the immediate's VALUE.
        // That is not a cell this rung gridded, so it is not a cell this rung
        // emits.
        (
            op @ (IlOp::And | IlOp::Or | IlOp::Xor | IlOp::Shl | IlOp::ShrS | IlOp::ShrU),
            RegOff { base: a, off: 0 },
            RegOff { base: b, off: 0 },
        ) => {
            debug_assert!(op.is_bitwise_or_shift());
            emit_reg_reg(op, a, b)
        }
        (IlOp::And | IlOp::Or | IlOp::Xor | IlOp::Shl | IlOp::ShrS | IlOp::ShrU, _, _) => Err(out_of_class(
            "a bitwise or shift operand that is not a bare register: the immediate \
             forms select an instruction by the immediate's VALUE (rlwinm mask / \
             andi. / lis+ori+and via r12) and are not gridded; out of class",
        )),

        (IlOp::Div, _, _) => Err(out_of_class("integer division; out of class")),
        (
            IlOp::Load(_)
            | IlOp::Lit(_)
            | IlOp::SymAddr(_)
            | IlOp::FpLit { .. }
            | IlOp::LoadInd { .. }
            | IlOp::LoadIndSized { .. }
                    | IlOp::LoadIndFp { .. }
            | IlOp::AddrOf { .. }
            // Board #1199's carrier — not a binary op, and refused before the
            // operand stack is ever built.
            | IlOp::BoundAddr { .. }
            | IlOp::StoreInd { .. }
            | IlOp::StoreIndFp { .. },
            _,
            _,
        ) => {
            unreachable!("not a binary op")
        }
    }
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
    fn select_text_sub_uses_reversed_operands() {
        // `a - b - c`: LOAD a, LOAD b, SUB, LOAD c, SUB. The subf operand order
        // (rA=rhs, rB=lhs) must reproduce c2's `subf r11,r4,r3 ; subf r3,r5,r11`.
        let func = IlFunction {
            eh_bare: false,
            eh_unwind_callees: Vec::new(),
            mangled_name: "?sub3@@YAHHHH@Z".into(),
            source_path: None,
            tail_call: None,
            framed_call: None,
        call_seq: None,
            cond_pair: None,
            compare: None,
            cmp_shift_or: None,
        ptr_walk_loop: None,
            if_call_join: None,
            ptr_walk_chain_loop: None,
        div_mod_leaf: None,
            empty_body: false,
            float_leaf: None,
            fp_tail: None,
        fp_arg_sources: None,
            arg_sources: None,
            data_syms: Vec::new(),
            fn_addr_sym: None,
            data_def: None,
            static_scan_loop: None,
            counted_accum_loop: None,
            guard_chain_shared_tail: None,
        alloc_init_or_fail: None,
        osf_handle_guard: None,
        guard_ret_chain: None,
        xlrc_create_guard: None,
        json_utf8_copy: None,
            params: vec![0xE309, 0xE409, 0xE509],
            ops: vec![
                IlOp::Load(0xE309),
                IlOp::Load(0xE409),
                IlOp::Sub,
                IlOp::Load(0xE509),
                IlOp::Sub,
            ],
            float_walk_loop: None,
        };
        assert_eq!(
            select_text(&func, OptMode::Ox).unwrap(),
            vec![
                0x7D, 0x64, 0x18, 0x50, // subf r11,r4,r3  (= r3-r4 = a-b)
                0x7C, 0x65, 0x58, 0x50, // subf r3,r5,r11  (= r11-r5 = (a-b)-c)
                0x4E, 0x80, 0x00, 0x20, // blr
            ]
        );
    }

    #[test]
    fn mul_chain_of_three_ops_uses_descending_scratch_registers() {
        // REGRESSION (w5_chain.cpp): `a*b*c*d`. c2 gives every intermediate of a
        // `*` chain its own register; the port used to reuse r11 and silently
        // mis-emitted. Reference `.text` (live capture):
        //   7d6321d6 mullw r11,r3,r4 ; 7d4b29d6 mullw r10,r11,r5
        //   7c6a31d6 mullw r3,r10,r6
        let f = func_with(
            vec![0xE309, 0xE409, 0xE509, 0xE609],
            vec![
                IlOp::Load(0xE309),
                IlOp::Load(0xE409),
                IlOp::Mul,
                IlOp::Load(0xE509),
                IlOp::Mul,
                IlOp::Load(0xE609),
                IlOp::Mul,
            ],
        );
        assert_eq!(
            select_text(&f, OptMode::Ox).unwrap(),
            vec![
                0x7D, 0x63, 0x21, 0xD6, // mullw r11,r3,r4
                0x7D, 0x4B, 0x29, 0xD6, // mullw r10,r11,r5
                0x7C, 0x6A, 0x31, 0xD6, // mullw r3,r10,r6
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
    }

    #[test]
    fn sub_chain_of_three_ops_descends_and_keeps_operand_order() {
        // `a-b-c-d`. Descending destinations AND the load-bearing reversed subf
        // operand order at every step. Reference:
        //   7d641850 subf r11,r4,r3 ; 7d455850 subf r10,r5,r11
        //   7c665050 subf r3,r6,r10
        let f = func_with(
            vec![0xE309, 0xE409, 0xE509, 0xE609],
            vec![
                IlOp::Load(0xE309),
                IlOp::Load(0xE409),
                IlOp::Sub,
                IlOp::Load(0xE509),
                IlOp::Sub,
                IlOp::Load(0xE609),
                IlOp::Sub,
            ],
        );
        assert_eq!(
            select_text(&f, OptMode::Ox).unwrap(),
            vec![
                0x7D, 0x64, 0x18, 0x50, // subf r11,r4,r3
                0x7D, 0x45, 0x58, 0x50, // subf r10,r5,r11
                0x7C, 0x66, 0x50, 0x50, // subf r3,r6,r10
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
    }

    #[test]
    fn add_chain_reuses_one_accumulator_register() {
        // The contrast that makes the rule non-obvious: an ADDITIVE chain
        // collapses to a single accumulator, so `a+b+c+d` keeps r11 throughout
        // where the `*`/`-` chains above descend. Reference:
        //   7d632214 add r11,r3,r4 ; 7d6b2a14 add r11,r11,r5
        //   7c6b3214 add r3,r11,r6
        let f = func_with(
            vec![0xE309, 0xE409, 0xE509, 0xE609],
            vec![
                IlOp::Load(0xE309),
                IlOp::Load(0xE409),
                IlOp::Add,
                IlOp::Load(0xE509),
                IlOp::Add,
                IlOp::Load(0xE609),
                IlOp::Add,
            ],
        );
        assert_eq!(
            select_text(&f, OptMode::Ox).unwrap(),
            vec![
                0x7D, 0x63, 0x22, 0x14, // add r11,r3,r4
                0x7D, 0x6B, 0x2A, 0x14, // add r11,r11,r5
                0x7C, 0x6B, 0x32, 0x14, // add r3,r11,r6
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
    }

    #[test]
    fn select_text_add_immediate() {
        // `a + 5` → addi r3,r3,5 ; blr
        let f = func_with(vec![0xE309], vec![IlOp::Load(0xE309), IlOp::Lit(5), IlOp::Add]);
        assert_eq!(
            select_text(&f, OptMode::Ox).unwrap(),
            vec![0x38, 0x63, 0x00, 0x05, 0x4E, 0x80, 0x00, 0x20]
        );
    }

    #[test]
    fn select_text_folds_consecutive_add_immediates() {
        // `a + 5 + 5` → the two literal adds fold to a single `addi r3,r3,10`
        // (c2 constant-folds `5 + 5` → `10`), NOT two chained addi. Verified
        // against the live obj (mvp_edit_addk2: .text = 3863000a 4e800020).
        let f = func_with(
            vec![0xE309],
            vec![
                IlOp::Load(0xE309),
                IlOp::Lit(5),
                IlOp::Add,
                IlOp::Lit(5),
                IlOp::Add,
            ],
        );
        assert_eq!(
            select_text(&f, OptMode::Ox).unwrap(),
            vec![0x38, 0x63, 0x00, 0x0A, 0x4E, 0x80, 0x00, 0x20]
        );
    }

    #[test]
    fn select_text_folds_mixed_add_sub_immediates() {
        // `a + 5 - 3` folds to `a + 2` → `addi r3,r3,2 ; blr`.
        let f = func_with(
            vec![0xE309],
            vec![
                IlOp::Load(0xE309),
                IlOp::Lit(5),
                IlOp::Add,
                IlOp::Lit(3),
                IlOp::Sub,
            ],
        );
        assert_eq!(
            select_text(&f, OptMode::Ox).unwrap(),
            vec![0x38, 0x63, 0x00, 0x02, 0x4E, 0x80, 0x00, 0x20]
        );
    }

    #[test]
    fn select_text_sub_immediate_folds_to_addi_neg() {
        // `a - 5` → addi r3,r3,-5 ; blr
        let f = func_with(vec![0xE309], vec![IlOp::Load(0xE309), IlOp::Lit(5), IlOp::Sub]);
        assert_eq!(
            select_text(&f, OptMode::Ox).unwrap(),
            vec![0x38, 0x63, 0xFF, 0xFB, 0x4E, 0x80, 0x00, 0x20]
        );
    }

    #[test]
    fn select_text_bare_constant_return_is_li() {
        // `return 42;` → addi r3,r0,42 (li) ; blr
        let f = func_with(vec![], vec![IlOp::Lit(42)]);
        assert_eq!(
            select_text(&f, OptMode::Ox).unwrap(),
            vec![0x38, 0x60, 0x00, 0x2A, 0x4E, 0x80, 0x00, 0x20]
        );
    }

    #[test]
    fn select_text_bare_non_first_parameter_is_one_mr() {
        // `return b;` is `mr r3,r4 ; blr` (`or r3,r4,r4`, opcode 31 / XO 444).
        // Measured across the whole argument file — every word here is read off
        // a reference obj, see `fixtures/cpp/w18_reg_move.cpp`.
        let p = vec![0xE309, 0xE409, 0xE509, 0xE609];
        let sel = |tok: u32| {
            select_text(&func_with(p.clone(), vec![IlOp::Load(tok)]), OptMode::Ox).unwrap()
        };
        // The first parameter is already in r3 and emits nothing at all — the
        // control that keeps this arm from firing on every identity.
        assert_eq!(sel(0xE309), vec![0x4E, 0x80, 0x00, 0x20]);
        assert_eq!(sel(0xE409), vec![0x7C, 0x83, 0x23, 0x78, 0x4E, 0x80, 0x00, 0x20]);
        assert_eq!(sel(0xE509), vec![0x7C, 0xA3, 0x2B, 0x78, 0x4E, 0x80, 0x00, 0x20]);
        assert_eq!(sel(0xE609), vec![0x7C, 0xC3, 0x33, 0x78, 0x4E, 0x80, 0x00, 0x20]);
        // The eighth argument register, r10 — the far end of the file.
        let eight: Vec<u32> = (0..8).map(|i| 0xE309 + i * 0x100).collect();
        assert_eq!(
            select_text(
                &func_with(eight.clone(), vec![IlOp::Load(eight[7])]),
                OptMode::Ox
            )
            .unwrap(),
            vec![0x7D, 0x43, 0x53, 0x78, 0x4E, 0x80, 0x00, 0x20],
            "mr r3,r10 ; blr"
        );
        // The mode does not reach this arm: there is no intermediate to allocate.
        assert_eq!(
            select_text(&func_with(p.clone(), vec![IlOp::Load(0xE409)]), OptMode::O1).unwrap(),
            sel(0xE409)
        );
        // A token that is not a parameter still fails closed — the move needs a
        // source register and there is none.
        assert!(select_text(&func_with(p, vec![IlOp::Load(0x9999)]), OptMode::Ox).is_err());
    }

    #[test]
    fn select_text_rejects_immediate_multiply() {
        // `a * 3` strength-reduces (out of class) — must reject, not mis-emit.
        let f = func_with(vec![0xE309], vec![IlOp::Load(0xE309), IlOp::Lit(3), IlOp::Mul]);
        assert!(matches!(select_text(&f, OptMode::Ox), Err(BackendError::NotImplemented(_))));
    }

    #[test]
    fn select_text_wide_add_immediate_uses_addis_addi() {
        // `a + 70000` → addis r3,r3,1 ; addi r3,r3,4464 ; blr.
        let f = func_with(vec![0xE309], vec![IlOp::Load(0xE309), IlOp::Lit(70000), IlOp::Add]);
        assert_eq!(
            select_text(&f, OptMode::Ox).unwrap(),
            vec![
                0x3C, 0x63, 0x00, 0x01, // addis r3,r3,1
                0x38, 0x63, 0x11, 0x70, // addi r3,r3,4464
                0x4E, 0x80, 0x00, 0x20, // blr
            ]
        );
    }

    #[test]
    fn select_text_wide_constant_load_uses_lis_ori() {
        // `return 70000;` → addis r3,r0,1 ; ori r3,r3,4464 ; blr.
        let f = func_with(vec![], vec![IlOp::Lit(70000)]);
        assert_eq!(
            select_text(&f, OptMode::Ox).unwrap(),
            vec![
                0x3C, 0x60, 0x00, 0x01, // addis r3,r0,1
                0x60, 0x63, 0x11, 0x70, // ori r3,r3,4464
                0x4E, 0x80, 0x00, 0x20, // blr
            ]
        );
    }

    fn tree4(op1: IlOp, op2: IlOp, root: IlOp) -> IlFunction {
        func_with(
            vec![0xE309, 0xE409, 0xE509, 0xE609],
            vec![
                IlOp::Load(0xE309),
                IlOp::Load(0xE409),
                op1,
                IlOp::Load(0xE509),
                IlOp::Load(0xE609),
                op2,
                root,
            ],
        )
    }

    #[test]
    fn depth2_tree_matches_the_reference() {
        // `(a+b)*(c+d)` — the operand stack reaches depth 3, so this is a tree
        // rather than a serial chain: left child into r11, right into r10, root
        // into r3.
        assert_eq!(
            select_text(&tree4(IlOp::Add, IlOp::Add, IlOp::Mul), OptMode::Ox).unwrap(),
            vec![
                0x7D, 0x63, 0x22, 0x14, // add   r11,r3,r4
                0x7D, 0x45, 0x32, 0x14, // add   r10,r5,r6
                0x7C, 0x6B, 0x51, 0xD6, // mullw r3,r11,r10
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
        // `(a*b)-(c*d)` — same register assignment; subf keeps its reversed
        // operand order (rA=rhs, rB=lhs).
        assert_eq!(
            select_text(&tree4(IlOp::Mul, IlOp::Mul, IlOp::Sub), OptMode::Ox).unwrap(),
            vec![
                0x7D, 0x63, 0x21, 0xD6, // mullw r11,r3,r4
                0x7D, 0x45, 0x31, 0xD6, // mullw r10,r5,r6
                0x7C, 0x6A, 0x58, 0x50, // subf  r3,r10,r11
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
    }

    #[test]
    fn depth2_tree_with_an_add_root_swaps_the_child_registers() {
        // The one exception: a `+` ROOT swaps the two children's registers
        // relative to every other root operator. `(a*b)+(c*d)` puts the left
        // child in r10 and the right in r11 — reproducible and order
        // independent, but not mechanistically understood, which is why the
        // `+` root is accepted at exactly this depth and nowhere else.
        assert_eq!(
            select_text(&tree4(IlOp::Mul, IlOp::Mul, IlOp::Add), OptMode::Ox).unwrap(),
            vec![
                0x7D, 0x43, 0x21, 0xD6, // mullw r10,r3,r4   <-- swapped
                0x7D, 0x65, 0x31, 0xD6, // mullw r11,r5,r6   <-- swapped
                0x7C, 0x6A, 0x5A, 0x14, // add   r3,r10,r11
                0x4E, 0x80, 0x00, 0x20,
            ]
        );
    }

    #[test]
    fn tree_shapes_c2_does_not_lower_as_trees_fail_closed() {
        // These are tree-shaped SOURCE that c2 re-linearizes, so a post-order
        // selector emits plausible wrong bytes rather than running out of range.
        //
        // N1: a `*` with a `*` child becomes one n-ary product — `(a*b)*(c*d)`,
        //     `(a+b)*(c*d)` and `a*(b*(c*d))` all compile to the SAME chain,
        //     none of them the source's pairing.
        for (op1, op2) in [(IlOp::Mul, IlOp::Mul), (IlOp::Add, IlOp::Mul), (IlOp::Mul, IlOp::Add)]
        {
            assert!(
                matches!(
                    select_text(&tree4(op1, op2, IlOp::Mul), OptMode::Ox),
                    Err(BackendError::NotImplemented(_))
                ),
                "N1: {op1:?} / {op2:?} under a `*` root must reject"
            );
        }
        // N2: an additive node with an additive child collects into one n-ary
        //     sum whose terms are REORDERED — `(a+b)-(c+d)` emits its leaves in
        //     the order a, c, d, b.
        for root in [IlOp::Add, IlOp::Sub] {
            for (op1, op2) in [(IlOp::Add, IlOp::Add), (IlOp::Sub, IlOp::Mul), (IlOp::Mul, IlOp::Sub)]
            {
                assert!(
                    matches!(
                        select_text(&tree4(op1, op2, root), OptMode::Ox),
                        Err(BackendError::NotImplemented(_))
                    ),
                    "N2: {op1:?} / {op2:?} under a {root:?} root must reject"
                );
            }
        }
    }

    #[test]
    fn select_text_mul_is_commutative_order() {
        // `a * b * c` → mullw r11,r3,r4 ; mullw r3,r11,r5 ; blr.
        let func = IlFunction {
            eh_bare: false,
            eh_unwind_callees: Vec::new(),
            mangled_name: "?mul3@@YAHHHH@Z".into(),
            source_path: None,
            tail_call: None,
            framed_call: None,
        call_seq: None,
            cond_pair: None,
            compare: None,
            cmp_shift_or: None,
        ptr_walk_loop: None,
            if_call_join: None,
            ptr_walk_chain_loop: None,
        div_mod_leaf: None,
            empty_body: false,
            float_leaf: None,
            fp_tail: None,
        fp_arg_sources: None,
            arg_sources: None,
            data_syms: Vec::new(),
            fn_addr_sym: None,
            data_def: None,
            static_scan_loop: None,
            counted_accum_loop: None,
            guard_chain_shared_tail: None,
        alloc_init_or_fail: None,
        osf_handle_guard: None,
        guard_ret_chain: None,
        xlrc_create_guard: None,
        json_utf8_copy: None,
            params: vec![0xE309, 0xE409, 0xE509],
            ops: vec![
                IlOp::Load(0xE309),
                IlOp::Load(0xE409),
                IlOp::Mul,
                IlOp::Load(0xE509),
                IlOp::Mul,
            ],
            float_walk_loop: None,
        };
        assert_eq!(
            select_text(&func, OptMode::Ox).unwrap(),
            vec![
                0x7D, 0x63, 0x21, 0xD6, // mullw r11,r3,r4
                0x7C, 0x6B, 0x29, 0xD6, // mullw r3,r11,r5
                0x4E, 0x80, 0x00, 0x20, // blr
            ]
        );
    }

    #[test]
    fn select_text_for_add3() {
        let func = IlFunction {
            eh_bare: false,
            eh_unwind_callees: Vec::new(),
            mangled_name: "?add3@@YAHHHH@Z".into(),
            source_path: None,
            tail_call: None,
            framed_call: None,
        call_seq: None,
            cond_pair: None,
            compare: None,
            cmp_shift_or: None,
        ptr_walk_loop: None,
            if_call_join: None,
            ptr_walk_chain_loop: None,
        div_mod_leaf: None,
            empty_body: false,
            float_leaf: None,
            fp_tail: None,
        fp_arg_sources: None,
            arg_sources: None,
            data_syms: Vec::new(),
            fn_addr_sym: None,
            data_def: None,
            static_scan_loop: None,
            counted_accum_loop: None,
            guard_chain_shared_tail: None,
        alloc_init_or_fail: None,
        osf_handle_guard: None,
        guard_ret_chain: None,
        xlrc_create_guard: None,
        json_utf8_copy: None,
            params: vec![0xE309, 0xE409, 0xE509],
            ops: vec![
                IlOp::Load(0xE309),
                IlOp::Load(0xE409),
                IlOp::Add,
                IlOp::Load(0xE509),
                IlOp::Add,
            ],
            float_walk_loop: None,
        };
        let text = select_text(&func, OptMode::Ox).unwrap();
        assert_eq!(
            text,
            vec![
                0x7D, 0x63, 0x22, 0x14, // add r11,r3,r4
                0x7C, 0x6B, 0x2A, 0x14, // add r3,r11,r5
                0x4E, 0x80, 0x00, 0x20, // blr
            ]
        );
    }

}
