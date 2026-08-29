use crate::func::IlOp;

/// The largest substituted operand stream accepted, so that a chain of
/// assignments each doubling the previous cannot blow up.
/// PROV[F] a substitution budget, chosen so a doubling chain cannot blow up. Nothing measured 32 as c2's own bound; it is this port's cap on its own work, and a real chain above it is refused. Fitted, failing closed.
pub(crate) const MAX_SUBST_OPS: usize = 32;

/// Integer argument registers r3..r10. A ninth parameter is stack-homed, which
/// needs a frame; mirrors `c2_core::codegen::ARG_REGS`.
/// PROV[S] the PowerPC EABI's eight integer argument registers r3..r10. Fixed by the ABI, not by c2; a ninth parameter is stack-homed by the same rule. Mirrors `c2_core::codegen::ARG_REGS`.
const ARG_REG_COUNT: usize = 8;

/// The census `ctx` of a straight-line body that parses cleanly but `select_text`
/// would decline anyway, or `None` when the body is in class.
///
/// These gates used to live in codegen, and that broke a stated invariant: the
/// convention is that acceptance is decided in the parser so `function_census` and
/// `PortC2::build` cannot disagree about what is in class. While they sat in
/// codegen every shape below parsed as `straight-line` and was *counted*, then was
/// refused at emission — so the census numerator included functions the port cannot
/// emit. Fail-closed either way, but the census is the public claim and its
/// histogram is the widening order, so an inflated numerator is a real defect
/// rather than a cosmetic one.
///
/// **Each clause has its own key.** The five refusals below are five different
/// lowerings — a register move, a `lis`/`ori`, a strength-reduced multiply, a
/// `subfic`, a stack frame — and reporting them as one `expr-out-of-class` bucket
/// is the `GAPS.md` §6 conflation failure in miniature: the row was 46,200
/// functions naming none of its own contents, and a row that cannot be
/// decomposed cannot be ranked. The bare-LOAD clause is split further, because
/// "the value is in the wrong argument register" (one `mr`) and "the value is not
/// an argument at all" (a global, a spilled local — no local lowering at all) are
/// the two halves that a single `params.first() != Some(t)` test hides.
///
/// Named rather than inlined so the test can assert the same predicate the parser
/// uses; the previous "census agrees with the gate" test compared `parse_segment`
/// with `parse_segment_detail`, which is `.ok()` of it, and so could not fail.
pub(crate) fn straight_line_out_of_class_ctx(
    ops: &[IlOp],
    params: &[u32],
) -> Option<&'static str> {
    // More than eight integer parameters: the ninth is stack-homed.
    if params.len() > ARG_REG_COUNT {
        return Some("expr-out-of-class-formals9");
    }
    // A bare `return <token>` whose token is **not a formal**: a global, a `.sy`
    // local, a token from a construct this class does not model. Nothing here
    // says where the value is, so nothing can be emitted for it.
    //
    // A bare *formal* is fine at any index. `return a;` emits nothing (it is
    // already in r3) and `return b;` is one `mr r3,rN` — see
    // [`c2_core::codegen::select_text`]'s finalize, and `w18_reg_move.cpp` for
    // the byte grading across every argument slot, both scalar classes and both
    // pointer spellings.
    if let [IlOp::Load(t)] = ops {
        if !params.contains(t) {
            return Some("expr-out-of-class-bare-nonformal");
        }
    }
    // A bare wide NEGATIVE constant: the `lis`+`ori` pair covers non-negative only.
    if let [IlOp::Lit(k)] = ops {
        if *k < -0x8000 {
            return Some("expr-out-of-class-wide-neg-lit");
        }
    }
    // Multiply by a constant strength-reduces to shifts and adds, and `const - reg`
    // needs a `subfic`. The chain is left-associative, so an operator's right-hand
    // operand is the leaf just before it, and a leading `Lit` is the only way a
    // literal reaches an operator's left.
    for (i, op) in ops.iter().enumerate() {
        let rhs_lit = matches!(ops.get(i.wrapping_sub(1)), Some(IlOp::Lit(_)));
        let lhs_lit = matches!(ops.first(), Some(IlOp::Lit(_)));
        match op {
            // `i == 2`, not `i == 1`. The op stream is **postfix**, so the first
            // operator of a two-leaf chain sits at index 2 (`[Lit, Load, Mul]`) —
            // which is exactly what the `Sub` arm below has always tested. At
            // `i == 1` the clause could never fire, so `return 3 * a;` censused
            // IN CLASS while the port returned `NotImplemented`: a census/gate
            // DISAGREEMENT, live, and the commuted form of the one case the rule
            // was derived from. Nothing caught it because the test table below
            // held `[Load, Lit, Mul]` and never the commutation, and because the
            // disagreement invariant is only evaluated on the `gap` path — which
            // reads 0 on the workload (no `3 * a` survives to this class there)
            // and was never run over the generated sweep corpus, where 144 cases
            // are this bug.
            IlOp::Mul if rhs_lit || (i == 2 && lhs_lit) => {
                return Some("expr-out-of-class-mul-by-lit")
            }
            IlOp::Sub if i == 2 && lhs_lit => return Some("expr-out-of-class-lit-minus-reg"),
            _ => {}
        }
    }
    // **A chain that MIXES the bitwise/shift six with the arithmetic three is
    // refused, and the reason is a live mis-emit this lane found in its own
    // change before shipping it** (`lane w-build`).
    //
    // `select_text`'s intermediate-register rule is `chain_has_add`: at `/Ox`,
    // if the chain contains any addition every intermediate reuses r11,
    // otherwise they descend r11, r10, r9. That rule was enumerated over 11,664
    // four-leaf ARITHMETIC chains and it is right about all of them. It is
    // **wrong** the moment a bitwise operator is in the chain — measured at
    // `/Ox`, `work/w-build/probe/bits3-Ox.cod`:
    //
    // ```text
    //   (a & b) + c + d    and  r11,r3,r4 ; add  r11,r11,r5 ; add  r3,r11,r6
    //   (a + b) & c & d    add  r11,r3,r4 ; and  r10,r11,r5 ; and  r3,r10,r6
    // ```
    //
    // Both chains contain an addition. The first collapses to r11 as the rule
    // says; **the second descends anyway.** So `chain_has_add` is not the rule —
    // the ten probed cells fit "an intermediate goes to r11 when its CONSUMER is
    // an `add`, and takes the next descending scratch otherwise", which also
    // reproduces every arithmetic cell `il_accum4.cpp` records. That is a
    // hypothesis over ten witnesses against a rule enumerated over 11,664, and
    // replacing the second with the first is not this rung's work.
    //
    // What IS this rung's work is not shipping the wrong bytes in the meantime.
    // A **pure** bitwise/shift chain is unaffected: it contains no addition, so
    // `chain_has_add` is false and the port descends exactly as c2 does — probed
    // at four and five leaves, at `/O1` and at `/Ox`. Mixing is what breaks, so
    // mixing refuses, under its own key so the cost is a number.
    //
    // The cells this costs are known and were all measured correct:
    // `(a>>b)+c`, `(a&b)+c`, `(a+b)&c`, `(a*b)^c`, `(a-b)<<c` at three leaves
    // (one intermediate, where every rule coincides) and `(a&b)+c+d`,
    // `(a&b)-c-d`, `(a-b)&c&d`, `(a>>b)+c+d` at four.
    if ops.iter().any(|o| o.is_bitwise_or_shift())
        && ops
            .iter()
            .any(|o| matches!(o, IlOp::Add | IlOp::Sub | IlOp::Mul))
    {
        return Some("expr-out-of-class-bitwise-mixed-arith");
    }
    // An expression the affine selector cannot walk over one scratch, and that is
    // not the one deeper shape that is characterized ([`chain_form`]). This clause
    // is the repair for `docs/IL_CALL_IN_EXPR.md` §24.7: `return a + b*c;` reaches
    // operand-stack depth 3 (a, b, c all live before the `*` fires), so it needs a
    // second scratch — and it censused in class for as long as the rule lived only
    // in codegen.
    if chain_form(ops, params).is_none() {
        return Some("expr-out-of-class-tree-depth");
    }
    None
}

/// [`straight_line_out_of_class_ctx`] as a predicate, for the call sites and tests
/// that only need the yes/no.
pub(crate) fn straight_line_is_out_of_class(ops: &[IlOp], params: &[u32]) -> bool {
    straight_line_out_of_class_ctx(ops, params).is_some()
}

/// True if any operand token is loaded more than once.
///
/// A repeated leaf licenses c2's algebraic rewriter, and it takes the licence:
/// `a + a` does **not** become `add r3,r3,r3`, it becomes `rlwinm r3,r3,1,0,30`
/// (`slwi r3,r3,1`) — byte-identical to what it emits for `a * 2`. So the operand
/// stream stops being a faithful description of the instructions.
///
/// This was a live mis-emit in the straight-line integer class, not a hypothetical:
/// `return a + a;` and `return a + b + a;` both produced wrong bytes, and had done
/// since that class was written, because no fixture used a parameter twice. The FP
/// leaf parser has had the equivalent gate from the start (see
/// [`try_parse_float_leaf`]); the integer path never got one.
///
/// Refusing is the conservative move: the rewrite set is not characterized (only
/// the `x + x` case is captured), so admitting any of it would be guessing.
pub(crate) fn has_repeated_leaf(ops: &[IlOp]) -> bool {
    let mut seen: Vec<u32> = Vec::new();
    for op in ops {
        if let IlOp::Load(t) = op {
            if seen.contains(t) {
                return true;
            }
            seen.push(*t);
        }
    }
    false
}

/// True if the operand LOADs appear in **strictly ascending parameter order** —
/// i.e. in ascending register order, since parameter `i` arrives in `r(3+i)`.
///
/// c2 does not evaluate a commutative chain in source order; it **canonicalizes
/// and reassociates** it by register. Every permutation of `a + b + c` — all five
/// of them — emits exactly `add r11,r3,r4 ; add r3,r11,r5`, and `b + a` emits the
/// same `add r3,r3,r4` as `a + b`. Mixed chains are reassociated too: `a + b - c`
/// and `b - c + a` both emit `subf r11,r5,r3 ; add r3,r11,r4`, which is `(a-c)+b`
/// — neither source grouping.
///
/// The port evaluates in source order, so it emitted numerically-correct but
/// byte-wrong code for every non-canonical chain. A generated differential sweep
/// over 600 integer expressions found ~20 of these, all in the straight-line class
/// that had been "byte-exact" since the MVP.
///
/// This gate is deliberately a **refusal, not a canonicalization**. The rewrite
/// rule is only partly characterized: the additive form looks like "start at the
/// lowest positive term, apply the negative terms in ascending order, then add the
/// remaining positives", but that is inferred from ten captures and implementing it
/// wrong would put the mis-emit straight back. Refusing is exact; a canonicalizer
/// needs its own capture matrix first (docs/GAPS.md).
///
/// Strictly ascending also implies no repeated leaf, so this subsumes
/// [`has_repeated_leaf`]; both are kept because they refuse for different reasons
/// and the census buckets should say which.
/// Rewrite a serial arithmetic chain into **c2's canonical order**, or return
/// `None` if the stream is not a shape this understands.
///
/// c2 does not evaluate a chain left to right. For an additive chain it treats the
/// whole thing as a sum of signed terms and emits, in order: the lowest-numbered
/// positive register, then every negative register ascending, then the remaining
/// positive registers ascending, then the folded literal. For a multiplicative
/// chain it simply sorts ascending. Captured:
///
/// ```text
///   a+b+c, a+c+b, b+a+c, b+c+a, c+b+a  ->  add r11,r3,r4 ; add r3,r11,r5
///   a*c*b                              ->  mullw r11,r3,r4 ; mullw r3,r11,r5
///   a + b - c   and   b - c + a        ->  subf r11,r5,r3 ; add r3,r11,r4
///   a - c - b                          ->  subf r11,r4,r3 ; subf r3,r5,r11
///   a + b - 1                          ->  add r11,r3,r4 ; addi r3,r11,-1
/// ```
///
/// Canonicalizing here rather than refusing is what makes every *permutation* of a
/// chain emit, instead of the one in six that happened to be written in register
/// order. It is done in the parser, not codegen, because the ordering key is the
/// parameter index — which is the register number — and because the census then sees
/// exactly what the emitter will.
///
/// Only a **serial** chain is handled: `leaf (leaf op)*`, all operators from one
/// family. A tree (`(a+b)*(c+d)`) is left untouched for `try_select_depth2_tree`,
/// and a mixed `*` with `+`/`-` is left to be refused downstream.
pub(crate) fn canonicalize_chain(ops: &[IlOp], params: &[u32]) -> Option<Vec<IlOp>> {
    // Recognize `leaf (leaf op)*` and split into signed terms.
    if ops.len() < 3 || ops.len() % 2 == 0 {
        return None;
    }
    let is_leaf = |o: &IlOp| matches!(o, IlOp::Load(_) | IlOp::Lit(_));
    if !is_leaf(&ops[0]) {
        return None;
    }
    let mut terms: Vec<(bool, IlOp)> = Vec::with_capacity(ops.len() / 2 + 1);
    terms.push((true, ops[0])); // (positive?, leaf)
    let mut mul = false;
    let mut addsub = false;
    let mut i = 1;
    while i + 1 < ops.len() + 1 && i + 1 <= ops.len() {
        if i + 1 > ops.len() - 1 {
            break;
        }
        let (leaf, op) = (ops[i], ops[i + 1]);
        if !is_leaf(&leaf) {
            return None;
        }
        match op {
            IlOp::Add => {
                addsub = true;
                terms.push((true, leaf));
            }
            IlOp::Sub => {
                addsub = true;
                terms.push((false, leaf));
            }
            IlOp::Mul => {
                mul = true;
                terms.push((true, leaf));
            }
            _ => return None,
        }
        i += 2;
    }
    if i != ops.len() || (mul && addsub) {
        return None;
    }
    // Order registers by parameter index; a non-parameter token is not orderable.
    let key = |o: &IlOp| match o {
        IlOp::Load(t) => params.iter().position(|p| p == t),
        _ => None,
    };
    if terms
        .iter()
        .any(|(_, l)| matches!(l, IlOp::Load(_)) && key(l).is_none())
    {
        return None;
    }
    // **The acceptance region of a rewrite rule must be a subset of the region that
    // was actually enumerated.** This rule was inferred from captures and is
    // validated by `scripts/expr_sweep.sh`, which enumerates chains of up to four
    // leaves; accepting longer ones would be emitting on extrapolation, which is
    // precisely how the per-chain accumulator bug survived (two rules that coincide
    // on short inputs). The multiplicative path is separately bounded by codegen's
    // r9 scratch floor, but the additive path is not bounded by anything — its
    // accumulator is r11 forever — so the bound has to be here.
    //
    // Raising this requires extending the sweep first, not the other way round.
    // PROV[F] a sweep bound on the additive path, which its own comment says "is not bounded by anything" otherwise. Fitted to the shapes measured; a longer real sweep is the off-sample case.
    const MAX_SWEPT_TERMS: usize = 4;
    if terms.len() > MAX_SWEPT_TERMS {
        return None;
    }

    if mul {
        // A multiplicative chain: registers ascending. A literal factor
        // strength-reduces (shift/add), which is not modeled, so refuse those.
        if terms.iter().any(|(_, l)| matches!(l, IlOp::Lit(_))) {
            return None;
        }
        let mut regs: Vec<IlOp> = terms.iter().map(|(_, l)| *l).collect();
        regs.sort_by_key(|l| key(l));
        let mut out = Vec::with_capacity(ops.len());
        out.push(regs[0]);
        for r in &regs[1..] {
            out.push(*r);
            out.push(IlOp::Mul);
        }
        return Some(out);
    }

    // Additive chain. Fold the literals into one constant; order the registers.
    let mut k: i32 = 0;
    let mut pos: Vec<IlOp> = Vec::with_capacity(terms.len());
    let mut neg: Vec<IlOp> = Vec::with_capacity(terms.len());
    for (positive, leaf) in &terms {
        match leaf {
            IlOp::Lit(v) => {
                k = if *positive {
                    k.checked_add(*v)?
                } else {
                    k.checked_sub(*v)?
                }
            }
            IlOp::Load(_) => {
                if *positive {
                    pos.push(*leaf)
                } else {
                    neg.push(*leaf)
                }
            }
            _ => return None,
        }
    }
    // Needs a positive register to start from: `k - a` is a `subfic` shape that the
    // selector does not model.
    if pos.is_empty() {
        return None;
    }
    pos.sort_by_key(|l| key(l));
    neg.sort_by_key(|l| key(l));
    let mut out = Vec::with_capacity(ops.len() + 2);
    out.push(pos[0]);
    for n in &neg {
        out.push(*n);
        out.push(IlOp::Sub);
    }
    for p in &pos[1..] {
        out.push(*p);
        out.push(IlOp::Add);
    }
    if k != 0 {
        // `i32::MIN.abs()` panics in debug (and wraps in release), so refuse rather
        // than rely on a downstream checked-arithmetic catch. Reachable: the literal
        // varint has a 4-byte escape form, so `a + b + (-2147483648)` is encodable.
        let mag = k.checked_abs()?;
        out.push(IlOp::Lit(mag));
        out.push(if k > 0 { IlOp::Add } else { IlOp::Sub });
    }
    Some(out)
}

/// True if a `+`/`-` chain's source order already *is* c2's canonical order.
///
/// c2 does not evaluate an additive chain left to right. It treats it as a sum of
/// signed terms and emits the **negative** terms first, starting from the lowest
/// positive term, then adds the remaining positives. Captured:
///
/// ```text
///   a + b - c   ->  subf r11,r5,r3 ; add r3,r11,r4     i.e. (a - c) + b
///   b - c + a   ->  subf r11,r5,r3 ; add r3,r11,r4     the same bytes
///   a - c - b   ->  subf r11,r4,r3 ; subf r3,r5,r11    i.e. (a - b) - c
/// ```
///
/// So source order coincides with c2's only when every register subtraction comes
/// *before* every register addition. `a - b + c` and `a - b - c` satisfy that and
/// are byte-exact; `a + b - c` does not, and was a mis-emit — the port computed
/// `(a+b)-c` where c2 computes `(a-c)+b`.
///
/// A subtraction of a **literal** does not count: it folds into the running `addi`
/// immediate rather than emitting an instruction, so `a + b - 1` is fine.
///
/// The chain is left-associative, so in the postfix stream each operator's
/// right-hand operand is the leaf immediately preceding it — which is all the
/// context needed to tell a register operand from a folded literal.
pub(crate) fn additive_chain_canonical(ops: &[IlOp]) -> bool {
    // Depth bound. Even a chain already in canonical order — needing no rewrite, and
    // so taking the pre-canonicalizer path — mis-emits once it is long enough: with
    // three or more register subtractions followed by an addition, c2's intermediate
    // allocation diverges again. Measured: every 4-leaf chain is byte-exact (11,664
    // enumerated), `a - b - c - d` is byte-exact, and `a - b - c - d + e` is not.
    //
    // Nothing else bounded this. The multiplicative path stops at codegen's r9
    // scratch floor, but an additive chain's accumulator is r11 forever, so a chain
    // of any length was accepted on extrapolation from short ones — the same shape as
    // the per-chain accumulator bug. Pure additions and pure multiplications are left
    // alone (5-leaf forms of both are verified); the bound applies only where a
    // subtraction is present, which is where the divergence was found.
    //
    // Raising it requires extending the sweep first.
    // PROV[F] the name is the citation: the count of leaves this composition was VERIFIED on. A fifth leaf is precisely the case the verification could not see.
    const MAX_VERIFIED_LEAVES_WITH_SUB: usize = 4;
    let has_sub = ops.iter().any(|o| matches!(o, IlOp::Sub));
    if has_sub {
        let leaves = ops
            .iter()
            .filter(|o| matches!(o, IlOp::Load(_) | IlOp::Lit(_)))
            .count();
        if leaves > MAX_VERIFIED_LEAVES_WITH_SUB {
            return false;
        }
    }
    let mut reg_add_seen = false;
    for (i, op) in ops.iter().enumerate() {
        let rhs_is_reg = matches!(ops.get(i.wrapping_sub(1)), Some(IlOp::Load(_)));
        match op {
            IlOp::Add if rhs_is_reg => reg_add_seen = true,
            IlOp::Sub if rhs_is_reg && reg_add_seen => return false,
            _ => {}
        }
    }
    true
}

/// One operand of the **affine** selector, as `c2_core::codegen::straightline`
/// models it: either a folded constant, or a register plus a *pending* immediate
/// that has not been materialized into an instruction yet.
///
/// The register identity is deliberately absent. The only thing this reproduction
/// has to decide is whether an operator meets an operand whose immediate is still
/// pending, and that is a property of the offset alone.
#[derive(Clone, Copy)]
enum AffOperand {
    Imm(i32),
    /// A register with `off` still owed to it (`off == 0` means "a bare register").
    RegOff(i32),
}

/// Can `select_text`'s **affine** selector lower this serial chain?
///
/// This is a faithful reproduction of `c2_core::codegen::straightline::combine`,
/// and it exists because the depth test in [`chain_form`] is the WRONG predicate
/// for the gate. `chain_form` simulates a generic two-deep operand *stack*, which
/// is a strictly wider class than the affine accumulator codegen actually
/// implements: the affine model carries a register plus one pending immediate and
/// has no way to materialize that immediate before a reg-reg operator fires. So
/// every stream that still owes a constant when it reaches an `add`/`subf`/`mullw`
/// is inside `chain_form`'s class and outside codegen's — which is a census/gate
/// disagreement by construction, and was one:
///
/// ```text
///   int f(int a,int b){ return (a+1)*b; }        [a, 1, Add, b, Mul]
/// ```
///
/// censused `straight-line` and `PortC2` returned `NotImplemented`, naming it
/// "multiply by a constant strength-reduces" — which it is not; the multiplier is
/// a register. The message was the catch-all arm speaking for a case it does not
/// describe, and that misattribution is why the row was never read as a gap.
///
/// **Only the serial form.** A [`ChainForm::Depth2Tree`] goes to
/// `try_select_depth2_tree`, a different emitter over four bare formals with no
/// literal anywhere, so the affine model does not apply to it and this returns
/// `true` without looking. A stream that is neither has already been refused
/// upstream by the `expr-out-of-class-tree-depth` clause.
///
/// Refusing is the whole content: nothing here admits a shape codegen would
/// decline, because every `false` below mirrors an `Err` arm of `combine`. The
/// converse — a shape codegen accepts and this refuses — would be an in-class
/// LOSS, so the two are kept as one reading of one rule rather than two rules
/// that agree on the cases anyone thought to check.
pub(crate) fn affine_serial_ok(ops: &[IlOp], params: &[u32]) -> bool {
    if chain_form(ops, params) != Some(ChainForm::Serial) {
        return true;
    }
    let mut stack: Vec<AffOperand> = Vec::with_capacity(4);
    for op in ops {
        match op {
            IlOp::Load(_) => stack.push(AffOperand::RegOff(0)),
            IlOp::Lit(k) => stack.push(AffOperand::Imm(*k)),
            IlOp::Add
            | IlOp::Sub
            | IlOp::Mul
            | IlOp::And
            | IlOp::Or
            | IlOp::Xor
            | IlOp::Shl
            | IlOp::ShrS
            | IlOp::ShrU => {
                let (Some(rhs), Some(lhs)) = (stack.pop(), stack.pop()) else {
                    return false;
                };
                use AffOperand::{Imm, RegOff};
                let folded = match (op, lhs, rhs) {
                    // Constant folding, and the immediate folds that emit nothing.
                    (IlOp::Add, Imm(a), Imm(b)) => Imm(match a.checked_add(b) {
                        Some(v) => v,
                        None => return false,
                    }),
                    (IlOp::Add, RegOff(off), Imm(k)) | (IlOp::Add, Imm(k), RegOff(off)) => {
                        RegOff(match off.checked_add(k) {
                            Some(v) => v,
                            None => return false,
                        })
                    }
                    (IlOp::Sub, Imm(a), Imm(b)) => Imm(match a.checked_sub(b) {
                        Some(v) => v,
                        None => return false,
                    }),
                    (IlOp::Sub, RegOff(off), Imm(k)) => RegOff(match off.checked_sub(k) {
                        Some(v) => v,
                        None => return false,
                    }),
                    // The reg-reg instructions, which require BOTH operands to be
                    // bare registers. This is the clause the whole function is for.
                    (IlOp::Add | IlOp::Sub | IlOp::Mul, RegOff(0), RegOff(0)) => RegOff(0),
                    // The bitwise/shift six, register-register and nothing else
                    // (`lane w-build`). **This clause must mirror `combine`'s
                    // exactly** — it is the parser-side simulation of it, and the
                    // whole reason it exists is that a census claiming a body the
                    // selector then refuses is a disagreement, not a gap. There is
                    // deliberately NO immediate-folding clause for these to sit
                    // beside the `Add`/`Sub` ones above: `a & 5` is not `a & 0`
                    // plus an offset, it is an `andi.`, and the fold that is free
                    // for `+` does not exist here.
                    (
                        IlOp::And | IlOp::Or | IlOp::Xor | IlOp::Shl | IlOp::ShrS | IlOp::ShrU,
                        RegOff(0),
                        RegOff(0),
                    ) => RegOff(0),
                    // `const - reg` needs a `subfic`; `reg * const` strength-reduces;
                    // a bitwise or shift operand that is not a bare register selects
                    // its instruction by the immediate's VALUE; and a reg-reg
                    // operator with a pending immediate on either side has no
                    // lowering in this model at all.
                    _ => return false,
                };
                stack.push(folded);
            }
            // Every other op has its own refusal upstream and no affine meaning.
            _ => return false,
        }
    }
    // The chain must reduce to exactly one value; a pending offset on it is fine,
    // because finalize materializes it as the last `addi` into r3.
    matches!(stack.as_slice(), [AffOperand::RegOff(_)] | [AffOperand::Imm(_)])
}

/// Why a chain cannot be handed to the selector — the three refusals
/// [`canonical_chain_for_codegen`] can return.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ChainReject {
    /// The operand LOADs are not in c2's canonical register order.
    Order,
    /// A register subtraction follows a register addition (c2 reassociates).
    Additive,
    /// The affine selector cannot lower it — see [`affine_serial_ok`].
    Affine,
    /// The chain's intermediate REGISTERS are not determined by the rule
    /// `select_text` implements — see [`intermediate_alloc_determined`].
    Alloc,
}

/// True when `select_text`'s intermediate-register rule provably agrees with c2
/// on this chain. **A live wrong-bytes emit is behind this**, found by `lane
/// w-build` on master `4d6aa58` with nothing of that lane's own in the tree.
///
/// ```text
///   int f(int a,int b,int c,int d) { return (a + b) * c * d; }
///
///   c2   /Ox   add r11,r3,r4 ; mullw r10,r11,r5 ; mullw r3,r10,r6
///   port /Ox   add r11,r3,r4 ; mullw r11,r11,r5 ; mullw r3,r11,r6
///                                     ^^^  Port=Mismatch @ offset 541
/// ```
///
/// `select_text` decides the whole chain at once: at `/Ox`, if it contains any
/// addition every intermediate reuses r11, otherwise they descend r11, r10, r9.
/// That rule came from an enumeration of **11,664 four-leaf chains** and is right
/// about every one of them — and `(a + b) * c * d` contains an addition and
/// descends anyway.
///
/// **The standing sweep cannot see this.** `scripts/sweep.d/10-int-chains.py`
/// enumerates `l1 o1 l2 o2 l3` — three leaves, so **one** intermediate, and with
/// one intermediate every candidate rule gives r11. The 11,664-case four-leaf
/// enumeration was a one-off. `scripts/sweep.d/12-alloc-depth.py` is the fragment
/// that closes the hole.
///
/// ## What the 23 measured cells say, and why this REFUSES rather than fixes
///
/// Every cell in `work/w-build/probe/alloc-Ox.cod`, `bits3-Ox.cod` and
/// `il_accum4.cpp` fits one rule: **an intermediate goes to r11 when its
/// CONSUMER is an `add`, and takes the next descending scratch otherwise.** It
/// reproduces the two `chain_has_add` misses and every case `chain_has_add` gets
/// right, including `(a & b) - c - d + e`, whose allocation is r11, r10, r11 —
/// which no whole-chain rule can produce at all.
///
/// Twenty-three witnesses is not eleven thousand. Replacing a rule that survived
/// an enumeration with one that has not faced it is how the per-chain
/// accumulator bug got in the first time (two rules that coincide on short
/// inputs). So the divergent region **refuses**, the hypothesis is written down,
/// and the fragment that would validate it ships alongside.
///
/// ## The predicate, derived from the mechanism rather than fitted
///
/// The two rules classify intermediate `k` by different things — the port by
/// "does the chain contain an add", c2 by "is `op[k+1]` an add" — so they can
/// only disagree where an add's result feeds a **non**-add. With fewer than
/// three operators there is nothing to disagree about: the sole intermediate is
/// r11 under both (the descending sequence *starts* at r11). Hence: three or
/// more operators, and an `Add` immediately followed by something that is not
/// one.
///
/// Checked against every cell above — it refuses `(a+b)*c*d` and `(a+b)*c*d*e`
/// and accepts `a+b+c+d`, `a-b-c-d`, `a-c-d+b`, `a-d+b+c`, `a-b-c+d`,
/// `(a*b)+c+d`, `(a*b)-c-d`, `a*b*c*d` and `(a+b)*c`, each re-graded against
/// real c2 after the guard went in and each still `Port=Match`.
///
/// **`(a+1)*b` was in that accepted list and is CORRECTED out of it**: it is
/// `NotImplemented` — but on master `4d6aa58` too, under
/// `expr-affine-pending-imm`, because `(a+1)` leaves a pending immediate that
/// `Mul` has no lowering for. This predicate does return `true` for it (two
/// operators, nothing to disagree about); the refusal is a different, older
/// gate, and claiming it as this one's acceptance would have credited this
/// guard with a decision it does not make.
pub(crate) fn intermediate_alloc_determined(ops: &[IlOp]) -> bool {
    let opers: Vec<IlOp> = ops.iter().copied().filter(|o| o.is_binary_int()).collect();
    if opers.len() < 3 {
        return true;
    }
    !opers
        .windows(2)
        .any(|w| w[0] == IlOp::Add && w[1] != IlOp::Add)
}

/// **The one canonicalize-or-refuse decision**, for both producers of a
/// [`BodyShape::StraightLine`](crate::func::body::BodyShape).
///
/// There are two: the plain expression body (`crates/c2-il/src/func/body/mod.rs`)
/// and the assignment/locals body (`shapes/assign.rs`), which resolves its
/// statement list by substitution and hands over the resulting chain. They must
/// hand codegen the *same* stream for the same expression, and they did not:
/// `mod.rs` called [`canonicalize_chain`] and `assign.rs` only ran the
/// pre-canonicalizer fallback checks, so
///
/// ```text
///   int f(int a,int b){ return a+1+b; }             -> Match
///   int f(int a,int b){ int x=a+1; return x+b; }    -> NotImplemented
/// ```
///
/// for one and the same resolved stream `[a, 1, Add, b, Add]`. The first is
/// canonicalized to `[a, b, Add, 1, Add]` — `add r11,r3,r4 ; addi r3,r11,1` — and
/// is byte-exact; the second reached the selector in source order and hit the
/// pending-immediate refusal. That is not a missing emitter, it is a producer that
/// did not call the one that already exists.
///
/// So the decision moves here, both producers call it, and each maps the rejection
/// onto its own census key (the keys differ by producer and are published).
pub(crate) fn canonical_chain_for_codegen(
    ops: &[IlOp],
    params: &[u32],
) -> Result<Vec<IlOp>, ChainReject> {
    let out = match canonicalize_chain(ops, params) {
        Some(c) => c,
        None => {
            if !leaves_ascending(ops, params) {
                return Err(ChainReject::Order);
            }
            if !additive_chain_canonical(ops) {
                return Err(ChainReject::Additive);
            }
            ops.to_vec()
        }
    };
    // LAST, and on the CANONICALIZED stream — which is the stream codegen sees.
    // Run before canonicalization it would refuse `return a+1+b;`, a shape the
    // port emits byte-exactly, so the order here is load-bearing.
    if !affine_serial_ok(&out, params) {
        return Err(ChainReject::Affine);
    }
    // ALSO on the canonicalized stream, and for the same reason: the
    // intermediate registers are allocated over the stream codegen sees, not the
    // one the source spelled. `a + b - c - d` reaches this as `a - c - d + b`,
    // whose trailing `Add` has no successor and is therefore fine, while its
    // source order would have looked divergent.
    if !intermediate_alloc_determined(&out) {
        return Err(ChainReject::Alloc);
    }
    Ok(out)
}

pub(crate) fn leaves_ascending(ops: &[IlOp], params: &[u32]) -> bool {
    let mut last: Option<usize> = None;
    for op in ops {
        if let IlOp::Load(t) = op {
            // A LOAD whose token is not a formal is **refused**, not skipped. The
            // gate orders by parameter index, so an unorderable operand means it
            // cannot do its job — and skipping was a real hole: `parse_formals`
            // used to anchor on the first `0x46` before `LO`, which a source-line
            // marker's payload (`4F 01 46`, a function on line 70) or the
            // per-function header region can supply, and it then returned an
            // *empty* formals list instead of failing. Any body that hit that
            // bypassed this gate entirely. The anchoring is fixed, but the gate
            // must fail closed regardless rather than depend on it.
            let Some(ix) = params.iter().position(|p| p == t) else {
                return false;
            };
            if let Some(prev) = last {
                if ix <= prev {
                    return false;
                }
            }
            last = Some(ix);
        }
    }
    true
}

/// Inline-substitute every `Load(t)` for which `env` has a definition.
///
/// The stream is postfix, so splicing a multi-op definition in place of a single
/// `Load` is valid without any bracketing: `[Load(x), Lit(1), Add]` with
/// `x -> [Load(a), Lit(1), Add]` becomes `[Load(a), Lit(1), Add, Lit(1), Add]`,
/// which is `(a+1)+1`.
///
/// One pass suffices because every `env` entry is *itself* already substituted —
/// entries are recorded at definition time, in terms of parameters only. That is
/// also what makes this correct rather than merely convenient: substituting at
/// definition time captures the operand values as of that point, so a later
/// redefinition of an operand cannot leak backwards. `int x = a; a = a + 1;
/// return x;` yields `x -> [Load(a)]` and returns the *entry* `a`, which is right;
/// substituting lazily at use time would return `a + 1`, which is not.
pub(crate) fn substitute(ops: &[IlOp], env: &[(u32, Vec<IlOp>)]) -> Option<Vec<IlOp>> {
    let mut out = Vec::with_capacity(ops.len());
    for op in ops {
        match op {
            IlOp::Load(t) => match env.iter().find(|(k, _)| k == t) {
                Some((_, def)) => out.extend_from_slice(def),
                None => out.push(*op),
            },
            _ => out.push(*op),
        }
        if out.len() > MAX_SUBST_OPS {
            return None;
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::{chain_form, ChainForm};

    use super::*;

    #[test]
    fn chains_canonicalize_to_c2s_register_order() {
        let p = vec![0x10, 0x11, 0x12]; // a -> r3, b -> r4, c -> r5
        let (a, b, c) = (IlOp::Load(0x10), IlOp::Load(0x11), IlOp::Load(0x12));
        let canon = |ops: Vec<IlOp>| canonicalize_chain(&ops, &p).unwrap();

        // Every permutation of `a + b + c` collapses to the same stream, because c2
        // emits the same `add r11,r3,r4 ; add r3,r11,r5` for all five.
        let want = vec![a, b, IlOp::Add, c, IlOp::Add];
        for perm in [
            vec![a, b, IlOp::Add, c, IlOp::Add],
            vec![a, c, IlOp::Add, b, IlOp::Add],
            vec![b, a, IlOp::Add, c, IlOp::Add],
            vec![b, c, IlOp::Add, a, IlOp::Add],
            vec![c, b, IlOp::Add, a, IlOp::Add],
        ] {
            assert_eq!(canon(perm), want);
        }
        // `b + a` -> `a + b`.
        assert_eq!(canon(vec![b, a, IlOp::Add]), vec![a, b, IlOp::Add]);
        // A multiplicative chain sorts ascending.
        assert_eq!(
            canon(vec![a, c, IlOp::Mul, b, IlOp::Mul]),
            vec![a, b, IlOp::Mul, c, IlOp::Mul]
        );

        // Additive: negatives first, from the lowest positive. `a + b - c` and
        // `b - c + a` both become `(a - c) + b`.
        let want_mixed = vec![a, c, IlOp::Sub, b, IlOp::Add];
        assert_eq!(canon(vec![a, b, IlOp::Add, c, IlOp::Sub]), want_mixed);
        assert_eq!(canon(vec![b, c, IlOp::Sub, a, IlOp::Add]), want_mixed);
        // Two negatives sort ascending: `a - c - b` becomes `(a - b) - c`.
        assert_eq!(
            canon(vec![a, c, IlOp::Sub, b, IlOp::Sub]),
            vec![a, b, IlOp::Sub, c, IlOp::Sub]
        );
        // Literals fold into one constant applied last, so they never affect order.
        assert_eq!(
            canon(vec![a, b, IlOp::Add, IlOp::Lit(1), IlOp::Sub]),
            vec![a, b, IlOp::Add, IlOp::Lit(1), IlOp::Sub]
        );
        assert_eq!(
            canon(vec![a, IlOp::Lit(1), IlOp::Add, IlOp::Lit(2), IlOp::Sub]),
            vec![a, IlOp::Lit(1), IlOp::Sub]
        );

        // Shapes it must decline rather than mangle: a tree (so
        // `try_select_depth2_tree` still sees it), a `*` mixed with `+`, a
        // multiply by a constant (which strength-reduces), and a chain with no
        // positive register to start from.
        assert!(canonicalize_chain(&[a, b, IlOp::Add, c, a, IlOp::Add, IlOp::Mul], &p).is_none());
        assert!(canonicalize_chain(&[a, b, IlOp::Mul, c, IlOp::Add], &p).is_none());
        assert!(canonicalize_chain(&[a, IlOp::Lit(2), IlOp::Mul], &p).is_none());
        assert!(canonicalize_chain(&[IlOp::Lit(1), a, IlOp::Sub], &p).is_none());
    }

    #[test]
    fn each_out_of_class_clause_reports_its_own_key() {
        // One bucket naming five different lowerings is the `GAPS.md` §6
        // conflation failure; a row that cannot be decomposed cannot be ranked.
        // Measured on the 878-TU workload: the whole 46,200-function row is two
        // of these clauses and the other three are 0, which is only visible once
        // they are separate keys.
        let p = vec![0x10, 0x11]; // a -> r3, b -> r4
        let ctx = |ops: &[IlOp]| straight_line_out_of_class_ctx(ops, &p);

        // `return a;` emits nothing (already in r3) and `return b;` is one
        // `mr r3,r4` — both in class, and W18 grades their bytes.
        assert_eq!(ctx(&[IlOp::Load(0x10)]), None);
        assert_eq!(ctx(&[IlOp::Load(0x11)]), None);
        // A token that is not a formal at all: a global, a `.sy` local, a token
        // from a construct this class does not model. Kept apart from the clause
        // above because there is no lowering implied — 2,881 functions against
        // 43,319, and only one of the two is a register move.
        assert_eq!(
            ctx(&[IlOp::Load(0x99)]),
            Some("expr-out-of-class-bare-nonformal")
        );
        // The three clauses the real workload never reaches, pinned so that stays
        // a measurement rather than an assumption.
        let nine: Vec<u32> = (0..9).collect();
        assert_eq!(
            straight_line_out_of_class_ctx(&[IlOp::Load(0)], &nine),
            Some("expr-out-of-class-formals9")
        );
        assert_eq!(
            ctx(&[IlOp::Lit(-0x8001)]),
            Some("expr-out-of-class-wide-neg-lit")
        );
        assert_eq!(ctx(&[IlOp::Lit(-0x8000)]), None);
        assert_eq!(
            ctx(&[IlOp::Load(0x10), IlOp::Lit(3), IlOp::Mul]),
            Some("expr-out-of-class-mul-by-lit")
        );
        // **The commutation, which is the case that was missing.** `3 * a` is the
        // same lowering as `a * 3` and must refuse identically; it did not, because
        // the clause tested `i == 1` against a postfix stream whose operator is at
        // index 2. The pair is kept adjacent so a rule derived from one operand
        // order can never again be graded only in that order.
        assert_eq!(
            ctx(&[IlOp::Lit(3), IlOp::Load(0x10), IlOp::Mul]),
            Some("expr-out-of-class-mul-by-lit")
        );
        assert_eq!(
            ctx(&[IlOp::Lit(3), IlOp::Load(0x10), IlOp::Sub]),
            Some("expr-out-of-class-lit-minus-reg")
        );
        // `a - 3` is the *other* commutation and is IN class: it is one `addi`
        // with a negated immediate, where `3 - a` needs a `subfic`. Asserting it
        // here is what keeps the fix above from being widened into "any literal
        // beside any operator", which would refuse a class the port emits.
        assert_eq!(ctx(&[IlOp::Load(0x10), IlOp::Lit(3), IlOp::Sub]), None);
        // The predicate and the key must never disagree: one is `.is_some()` of
        // the other, and this is the test that keeps it that way.
        for ops in [
            vec![IlOp::Load(0x10)],
            vec![IlOp::Load(0x11)],
            vec![IlOp::Load(0x99)],
            vec![IlOp::Lit(-0x8001)],
            vec![IlOp::Load(0x10), IlOp::Lit(3), IlOp::Mul],
            vec![IlOp::Lit(3), IlOp::Load(0x10), IlOp::Mul],
        ] {
            assert_eq!(
                straight_line_is_out_of_class(&ops, &p),
                straight_line_out_of_class_ctx(&ops, &p).is_some()
            );
        }
    }

    #[test]
    fn reassociation_gates_separate_canonical_from_rewritten_chains() {
        // params[i] is register r(3+i), so ascending index == ascending register.
        let p = vec![0x10, 0x11, 0x12]; // a, b, c
        let ld = |t: u32| IlOp::Load(t);

        // Commutative chains are canonicalized by register: every permutation of
        // `a + b + c` emits the same bytes, so only the ascending one may be
        // accepted in source order.
        assert!(leaves_ascending(&[ld(0x10), ld(0x11), IlOp::Add], &p)); // a + b
        assert!(!leaves_ascending(&[ld(0x11), ld(0x10), IlOp::Add], &p)); // b + a
        assert!(leaves_ascending(
            &[ld(0x10), ld(0x11), IlOp::Add, ld(0x12), IlOp::Add],
            &p
        )); // a + b + c
        assert!(!leaves_ascending(
            &[ld(0x10), ld(0x12), IlOp::Add, ld(0x11), IlOp::Add],
            &p
        )); // a + c + b
        // Literals do not participate in the ordering.
        assert!(leaves_ascending(&[ld(0x10), IlOp::Lit(1), IlOp::Add, ld(0x11), IlOp::Add], &p));

        // A mixed chain is reassociated even when the operands ARE in register
        // order: c2 applies the negative terms first. `a - b + c` is already that
        // order and is byte-exact; `a + b - c` is not.
        assert!(additive_chain_canonical(&[
            ld(0x10),
            ld(0x11),
            IlOp::Sub,
            ld(0x12),
            IlOp::Add
        ])); // a - b + c
        assert!(additive_chain_canonical(&[
            ld(0x10),
            ld(0x11),
            IlOp::Sub,
            ld(0x12),
            IlOp::Sub
        ])); // a - b - c
        assert!(!additive_chain_canonical(&[
            ld(0x10),
            ld(0x11),
            IlOp::Add,
            ld(0x12),
            IlOp::Sub
        ])); // a + b - c  -> c2 emits (a - c) + b
        // Subtracting a LITERAL folds into the `addi` immediate and emits no
        // instruction, so it can never be out of order.
        assert!(additive_chain_canonical(&[
            ld(0x10),
            ld(0x11),
            IlOp::Add,
            IlOp::Lit(1),
            IlOp::Sub
        ])); // a + b - 1
    }

    #[test]
    fn repeated_leaves_are_refused_before_and_after_substitution() {
        // Written directly: `a + a`. c2 emits `slwi r3,r3,1`, not `add r3,r3,r3`,
        // so accepting this is wrong bytes rather than a missing feature.
        assert!(has_repeated_leaf(&[IlOp::Load(1), IlOp::Load(1), IlOp::Add]));
        assert!(has_repeated_leaf(&[
            IlOp::Load(1),
            IlOp::Load(2),
            IlOp::Add,
            IlOp::Load(1),
            IlOp::Add
        ]));
        // Distinct operands, and a literal reused, are both fine.
        assert!(!has_repeated_leaf(&[IlOp::Load(1), IlOp::Load(2), IlOp::Add]));
        assert!(!has_repeated_leaf(&[
            IlOp::Load(1),
            IlOp::Lit(1),
            IlOp::Add,
            IlOp::Lit(1),
            IlOp::Add
        ]));

        // Substitution CREATES repetition that the source did not have:
        // `int x = a; x = x + x;` has no repeated operand written anywhere, but
        // resolves to `a + a`. This is why the gate runs on the resolved stream.
        let env = vec![(0x100, vec![IlOp::Load(1)])];
        let resolved = substitute(&[IlOp::Load(0x100), IlOp::Load(0x100), IlOp::Add], &env).unwrap();
        assert_eq!(resolved, vec![IlOp::Load(1), IlOp::Load(1), IlOp::Add]);
        assert!(has_repeated_leaf(&resolved));
    }

    #[test]
    fn substitution_captures_operands_at_definition_time() {
        // `int x = a; a = a + 1; return x;` must return the ENTRY `a`. Recording
        // definitions already-substituted is what guarantees it: a later
        // redefinition of `a` cannot reach backwards into `x`'s definition.
        // Substituting lazily at use time would wrongly yield `a + 1`.
        let mut env: Vec<(u32, Vec<IlOp>)> = Vec::new();
        // int x = a;
        env.push((0x100, substitute(&[IlOp::Load(1)], &env).unwrap()));
        // a = a + 1;
        let rhs = substitute(&[IlOp::Load(1), IlOp::Lit(1), IlOp::Add], &env).unwrap();
        env.retain(|(t, _)| *t != 1);
        env.push((1, rhs));
        // return x;
        assert_eq!(substitute(&[IlOp::Load(0x100)], &env).unwrap(), vec![IlOp::Load(1)]);
        // return a; would instead be the incremented value.
        assert_eq!(
            substitute(&[IlOp::Load(1)], &env).unwrap(),
            vec![IlOp::Load(1), IlOp::Lit(1), IlOp::Add]
        );
    }
}

/// The two integer expression forms `c2_core::codegen::select_text` can lower.
///
/// **One locator.** The parser refuses anything this returns `None` for (so the
/// census cannot claim it), and codegen consults it to decide which of its two
/// emitters to run (so it cannot lower a shape the parser did not admit). Before
/// it existed the depth rule lived only in `select_text`, which meant
/// `int f(int a,int b,int c){ return a + b*c; }` **censused in class and was
/// refused at emission** — `docs/IL_CALL_IN_EXPR.md` §24.7, the disagreement this
/// whole change is about.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChainForm {
    /// A serial accumulator chain: the postfix walk never puts more than two
    /// values on the operand stack, so one running scratch (`r11`) suffices.
    Serial,
    /// The depth-2 tree `(x op y) root (z op w)` over four distinct formals —
    /// two scratches, `r11`/`r10`, with the `+`-root swap.
    Depth2Tree,
}

/// Which form `ops` is, or `None` when it is neither.
///
/// The `Serial` test is the *identical* simulation `select_text`'s emitter loop
/// runs: push for a leaf, pop-two-push-one for a binary operator, and the depth
/// is checked after **every** op, so three consecutive LOADs already fail. Only
/// the ops an integer straight-line body can carry are simulated; anything else
/// (an FP literal, a division, an indirect load, a sub-object address) is
/// refused by a clause of its own upstream and has no depth to speak of.
pub fn chain_form(ops: &[IlOp], params: &[u32]) -> Option<ChainForm> {
    if is_depth2_tree(ops, params) {
        return Some(ChainForm::Depth2Tree);
    }
    let mut depth = 0usize;
    for op in ops {
        match op {
            IlOp::Load(_) | IlOp::Lit(_) => depth += 1,
            // The arithmetic three plus the bitwise/shift six (`lane w-build`).
            // Read through [`IlOp::is_binary_int`] rather than spelled out, so
            // this simulation and `select_text`'s emitter loop cannot drift
            // apart — a census that claims a body the port refuses is the
            // failure `ROADMAP.md` §6d records.
            //
            // **`is_depth2_tree` above is NOT widened with them**, and that is a
            // deliberate refusal with a measurement behind it: the `+`-root
            // register swap generalizes to the bitwise roots — probed,
            // `(a&b)|(c&d)` is `and r11 ; and r10 ; or r3,r11,r10` (no swap) and
            // `(a&b)+(c&d)` is `and r10 ; and r11 ; add r3,r10,r11` (swap), so
            // the ROOT alone decides — but that is 4 cells of a 216-cell
            // root x op1 x op2 grid, where the accepted arithmetic three are 27
            // cells that were graded whole. Four witnesses do not license a
            // sixfold widening of a shape whose two known rewrites (N1 product
            // flattening, N2 additive canonicalization) were both found by
            // gridding it. A bitwise depth-2 tree therefore reaches `depth > 2`
            // here and refuses, which is where it should refuse.
            op if op.is_binary_int() => {
                if depth < 2 {
                    return None;
                }
                depth -= 1;
            }
            // Not lowerable by the affine selector at all; each has its own
            // refusal there, and none of them belongs to a serial int chain.
            _ => return None,
        }
        if depth > 2 {
            return None;
        }
    }
    (depth == 1).then_some(ChainForm::Serial)
}

/// The depth-2 tree shape: `[Load, Load, op1, Load, Load, op2, root]` over four
/// **distinct formals**, with the three negatives that are characterized as
/// rewrites rather than as this shape (`docs/CODEGEN_W5_SCRATCH.md`):
///
/// * **N1, product flattening** — a `*` root over a `*` child is re-linearized;
/// * **N2, additive canonicalization** — an additive root over an additive child
///   is reassociated into a chain;
/// * integer division, which is not modeled anywhere.
fn is_depth2_tree(ops: &[IlOp], params: &[u32]) -> bool {
    let (l0, l1, op1, l2, l3, op2, root) = match ops {
        [IlOp::Load(a), IlOp::Load(b), o1, IlOp::Load(c), IlOp::Load(d), o2, r]
            if o1.is_tree_binop() && o2.is_tree_binop() && r.is_tree_binop() =>
        {
            (*a, *b, *o1, *c, *d, *o2, *r)
        }
        _ => return false,
    };
    let toks = [l0, l1, l2, l3];
    for (i, t) in toks.iter().enumerate() {
        if toks[..i].contains(t) || !params.contains(t) {
            return false;
        }
    }
    let is_additive = |o: IlOp| matches!(o, IlOp::Add | IlOp::Sub);
    if root == IlOp::Mul && (op1 == IlOp::Mul || op2 == IlOp::Mul) {
        return false;
    }
    if is_additive(root) && (is_additive(op1) || is_additive(op2)) {
        return false;
    }
    if root == IlOp::Div || op1 == IlOp::Div || op2 == IlOp::Div {
        return false;
    }
    true
}

#[cfg(test)]
mod chain_form_tests {
    use super::*;

    fn ld(t: u32) -> IlOp {
        IlOp::Load(t)
    }

    /// The disagreement this whole change is about (`IL_CALL_IN_EXPR.md` §24.7):
    /// `return a + b*c;` puts a, b and c on the operand stack before the `*`
    /// fires, so one running scratch cannot express it. It used to parse as
    /// `straight-line` and be refused by `select_text`.
    #[test]
    fn a_multiply_after_the_first_operator_is_not_a_serial_chain() {
        let p = vec![0x10, 0x11, 0x12];
        // a + b*c  ->  L a, L b, L c, MUL, ADD
        let ops = [ld(0x10), ld(0x11), ld(0x12), IlOp::Mul, IlOp::Add];
        assert_eq!(chain_form(&ops, &p), None);
        assert_eq!(
            straight_line_out_of_class_ctx(&ops, &p),
            Some("expr-out-of-class-tree-depth")
        );
        // a*b + c  ->  L a, L b, MUL, L c, ADD  — never deeper than two.
        let ok = [ld(0x10), ld(0x11), IlOp::Mul, ld(0x12), IlOp::Add];
        assert_eq!(chain_form(&ok, &p), Some(ChainForm::Serial));
        assert_eq!(straight_line_out_of_class_ctx(&ok, &p), None);
    }

    #[test]
    fn the_depth2_tree_is_the_one_deeper_shape_and_its_rewrites_are_not() {
        let p = vec![0x10, 0x11, 0x12, 0x13];
        let tree = |o1, o2, r| [ld(0x10), ld(0x11), o1, ld(0x12), ld(0x13), o2, r];
        // (a+b)*(c+d) and (a*b)+(c*d) are the accepted roots.
        assert_eq!(
            chain_form(&tree(IlOp::Add, IlOp::Add, IlOp::Mul), &p),
            Some(ChainForm::Depth2Tree)
        );
        assert_eq!(
            chain_form(&tree(IlOp::Mul, IlOp::Mul, IlOp::Add), &p),
            Some(ChainForm::Depth2Tree)
        );
        // N1 product flattening and N2 additive canonicalization are rewrites.
        assert_eq!(chain_form(&tree(IlOp::Mul, IlOp::Mul, IlOp::Mul), &p), None);
        assert_eq!(chain_form(&tree(IlOp::Add, IlOp::Add, IlOp::Add), &p), None);
        // A leaf that is not a formal has no register, so the tree cannot be
        // emitted and the serial walk reaches depth 3.
        let q = vec![0x10, 0x11, 0x12];
        assert_eq!(chain_form(&tree(IlOp::Add, IlOp::Add, IlOp::Mul), &q), None);
    }

    /// **The pending-immediate refusal, and why the depth test could not see it.**
    ///
    /// `chain_form` calls `[a, 1, Add, b, Add]` a `Serial` chain, because a
    /// two-deep operand *stack* expresses it fine. The affine selector does not
    /// have a stack — it has a register plus one immediate it still owes — so it
    /// cannot fire a reg-reg `add` while that immediate is pending. Every stream
    /// below is inside `chain_form`'s class and outside codegen's, which is a
    /// census/gate disagreement by construction, and all four were live:
    ///
    /// ```text
    ///   int f(int a,int b){ return (a+1)*b; }        (a, b are registers!)
    ///   int f(int a,int b){ int x=a+1; return x+b; }
    /// ```
    #[test]
    fn a_pending_immediate_at_a_reg_reg_operator_is_not_affine() {
        let p = vec![0x10, 0x11, 0x12];
        let (a, b, c) = (ld(0x10), ld(0x11), ld(0x12));
        let one = IlOp::Lit(1);

        // `(a+1)+b` and `(a+1)*b` — the two the board filed, in source order.
        assert!(!affine_serial_ok(&[a, one, IlOp::Add, b, IlOp::Add], &p));
        assert!(!affine_serial_ok(&[a, one, IlOp::Add, b, IlOp::Mul], &p));
        // …and the subtracting twin, which is the same variable at a third arm.
        assert!(!affine_serial_ok(&[a, one, IlOp::Add, b, IlOp::Sub], &p));
        // `(a-1)*b` — the pending offset can be negative and it is still pending.
        assert!(!affine_serial_ok(&[a, one, IlOp::Sub, b, IlOp::Mul], &p));

        // **The canonicalized form of the same expression is accepted**, which is
        // the whole reason this gate runs after canonicalization rather than
        // before: `a+1+b` becomes `(a+b)+1`, whose immediate is owed only at the
        // very end, where finalize materializes it as one `addi` into r3.
        assert!(affine_serial_ok(&[a, b, IlOp::Add, one, IlOp::Add], &p));
        assert_eq!(
            canonical_chain_for_codegen(&[a, one, IlOp::Add, b, IlOp::Add], &p),
            Ok(vec![a, b, IlOp::Add, one, IlOp::Add])
        );
        // The multiplicative twin has no canonical form — `canonicalize_chain`
        // declines a `*` mixed with a `+` — so the pending offset survives and
        // this one is REFUSED rather than rewritten. Census over-claim, closed.
        assert_eq!(
            canonical_chain_for_codegen(&[a, one, IlOp::Add, b, IlOp::Mul], &p),
            Err(ChainReject::Affine)
        );

        // A depth-2 tree is a DIFFERENT emitter (`try_select_depth2_tree`) over
        // four bare formals, so the affine model does not apply to it and must not
        // speak for it.
        let q = vec![0x10, 0x11, 0x12, 0x13];
        let tree = [a, b, IlOp::Add, c, ld(0x13), IlOp::Add, IlOp::Mul];
        assert_eq!(chain_form(&tree, &q), Some(ChainForm::Depth2Tree));
        assert!(affine_serial_ok(&tree, &q));
    }

    /// **The UNDER-claiming direction, which nothing else tests.**
    ///
    /// A gate that refuses too much is an in-class loss, and it is invisible: the
    /// census simply reports a smaller numerator and every differential still
    /// passes, because a refused function is never graded. So the streams codegen
    /// is *known* to lower — each one taken from a byte-graded test in
    /// `c2_core::codegen::straightline` — are asserted to survive this gate.
    ///
    /// The population-level version of this check is the generated sweep: every
    /// case `canonical_chain_for_codegen` accepts is compiled and byte-compared
    /// against the real c2, and a stream wrongly admitted shows up there as a
    /// MISMATCH. This test is the fast half that names the shapes.
    #[test]
    fn every_stream_the_affine_selector_lowers_survives_the_gate() {
        let p = vec![0x10, 0x11, 0x12];
        let (a, b, c) = (ld(0x10), ld(0x11), ld(0x12));
        for (what, ops) in [
            ("return a;", vec![a]),
            ("return b;  (one mr)", vec![b]),
            ("return 42;  (li)", vec![IlOp::Lit(42)]),
            ("return a+1;  (addi)", vec![a, IlOp::Lit(1), IlOp::Add]),
            ("return a-1;  (addi neg)", vec![a, IlOp::Lit(1), IlOp::Sub]),
            (
                "return a+5+5;  (folds to one addi)",
                vec![a, IlOp::Lit(5), IlOp::Add, IlOp::Lit(5), IlOp::Add],
            ),
            (
                "return a+5-3;  (mixed immediates fold)",
                vec![a, IlOp::Lit(5), IlOp::Add, IlOp::Lit(3), IlOp::Sub],
            ),
            ("return a+b;", vec![a, b, IlOp::Add]),
            ("return a-b;", vec![a, b, IlOp::Sub]),
            ("return a*b;", vec![a, b, IlOp::Mul]),
            (
                "return a+b+c;  (add3)",
                vec![a, b, IlOp::Add, c, IlOp::Add],
            ),
            (
                "return a*b+c;  (never deeper than two)",
                vec![a, b, IlOp::Mul, c, IlOp::Add],
            ),
            (
                "return a+b+1;  (the canonical form of a+1+b)",
                vec![a, b, IlOp::Add, IlOp::Lit(1), IlOp::Add],
            ),
            (
                "return a*b*c;",
                vec![a, b, IlOp::Mul, c, IlOp::Mul],
            ),
            (
                "return a-b-c;",
                vec![a, b, IlOp::Sub, c, IlOp::Sub],
            ),
            (
                "return a+65536;  (wide addis+addi)",
                vec![a, IlOp::Lit(0x10000), IlOp::Add],
            ),
        ] {
            assert!(
                affine_serial_ok(&ops, &p),
                "the affine gate refuses `{what}`, which codegen lowers — an \
                 in-class LOSS, and one nothing else would report"
            );
            assert!(
                canonical_chain_for_codegen(&ops, &p).is_ok(),
                "the shared canonicalizer refuses `{what}`, which codegen lowers"
            );
        }
    }

    /// Both producers of a `StraightLine` must hand codegen the **same** stream
    /// for the same expression. They did not: `mod.rs` canonicalized and
    /// `assign.rs` did not, so `return a+1+b;` was byte-exact while
    /// `int x=a+1; return x+b;` — the identical resolved stream — was refused.
    #[test]
    fn the_two_straight_line_producers_canonicalize_identically() {
        let p = vec![0x10, 0x11];
        let (a, b) = (ld(0x10), ld(0x11));
        // Written flat, and reached by substituting `x -> [a, 1, Add]`.
        let written = vec![a, IlOp::Lit(1), IlOp::Add, b, IlOp::Add];
        let env = vec![(0x100, vec![a, IlOp::Lit(1), IlOp::Add])];
        let substituted = substitute(&[ld(0x100), b, IlOp::Add], &env).unwrap();
        assert_eq!(substituted, written);
        // One decision, so one answer — whichever producer got there.
        assert_eq!(
            canonical_chain_for_codegen(&written, &p),
            canonical_chain_for_codegen(&substituted, &p)
        );
        assert_eq!(
            canonical_chain_for_codegen(&substituted, &p),
            Ok(vec![a, b, IlOp::Add, IlOp::Lit(1), IlOp::Add])
        );
    }

    /// A single leaf, and a bare literal, are both serial chains of depth 1 —
    /// the shapes `return a;` and `return 7;` decode to.
    #[test]
    fn a_single_leaf_is_a_serial_chain() {
        let p = vec![0x10];
        assert_eq!(chain_form(&[ld(0x10)], &p), Some(ChainForm::Serial));
        assert_eq!(chain_form(&[IlOp::Lit(7)], &p), Some(ChainForm::Serial));
        // An unbalanced stream is not a chain at all.
        assert_eq!(chain_form(&[ld(0x10), ld(0x10)], &p), None);
        assert_eq!(chain_form(&[IlOp::Add], &p), None);
    }
}

// ===========================================================================
// The floating-point multiply-add CONTRACTION rule.
//
// **It lives in `c2-il` and not beside the emitter, and that placement was
// forced by a test.** Lane `w-fmadd` first wrote the whole rule in
// `c2_core::codegen::leaf::float`, and
// `crates/c2-harness/tests/census_gate.rs`'s
// `the_census_and_the_port_agree_over_the_generated_corpus` went red in the
// words `docs/GAPS.md` §6 uses:
//
//   A NEW census/gate refusal family appeared over the generated corpus …
//   The census counts these functions in class and PortC2 refuses them, so the
//   published coverage numerator over-claims by that many … acceptance belongs
//   in the IL parser.
//
// It was two functions over 19,549 generated cases, and the invariant is
// right: a refusal in codegen alone is a coverage over-claim. Putting the rule
// HERE lets `try_parse_float_leaf` refuse and the emitter share the same code
// rather than restate it — the two-implementations hazard board #3637 is
// about. `c2-core` depends on `c2-il`, so this direction is the only one that
// admits a single implementation.
// ===========================================================================
/// **What one binary node does once deferred products are in play.**
///
/// c2 contracts *every* `*` that feeds a `+`/`-` — mandatory, not an
/// optimisation (`docs/CODEGEN_W13_FLOAT.md` §3.3) — so a multiply is not
/// emitted when it is read; it is held as a pending `(A, C)` pair and either
/// folded into its parent or materialised as an `fmul` when the parent cannot
/// take it.
///
/// **One rule, two consumers**, which is the whole reason this is a function
/// and not two `match`es: [`fp_contract_instructions`] runs it to find out how many
/// instructions the body will emit (the last of which targets `f1`), and the
/// evaluator in [`float_leaf_text`] runs it to emit them.
/// `the_counter_and_the_emitter_agree` drives both over a generated set of op
/// streams and requires the counts to match — board **#3637**'s lesson, that
/// two rules which agree are invisible for exactly as long as they agree.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FpNodePlan {
    /// A multiply: emit nothing, push a deferred product.
    DeferProduct,
    /// `+`/`-` whose **left** operand is a product: `fmadd`/`fmsub`.
    FuseLeft,
    /// `+`/`-` whose **right** operand is a product: `fmadd`/**`fnmsub`**.
    ///
    /// The `-` case is the asymmetry that makes the side matter:
    /// `fnmsub` computes `−(A*C − B)` = `B − A*C`, so `c - a*b` is one
    /// instruction and using `fmsub` here would negate the result.
    FuseRight,
    /// Everything else: both operands materialised, one plain instruction.
    Plain,
    /// **c2 REASSOCIATES this one and the port does not model it.** See
    /// [`fp_node_plan`]'s § "the one fence".
    Refuse,
}

/// What the evaluator knows about one stack entry, and it is exactly the two
/// bits [`fp_node_plan`] reads.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct FpValKind {
    /// A multiply whose instruction has not been emitted.
    pub prod: bool,
    /// The value came out of an `IlOp::Add` node — plain `fadd` **or** fused
    /// `fmadd`, which are the same node as far as the reassociation is
    /// concerned.
    pub from_add: bool,
}

/// The contraction rule itself.
///
/// **Left wins when both operands are products** — `a*b + c*d` and
/// `c*d + a*b` emit *the same three words* (`work/w-fmadd/probe/fma2.cod`,
/// `fma3.cod`), with `a*b` fused and `c*d` materialised into `f0` first; the
/// parser's ascending-leaf gate is what makes "left" and "lower-numbered" the
/// same choice on everything that reaches here.
///
/// # The one fence, and it was found by the sweep rather than reasoned to
///
/// `float f(float a,float b,float c,float d){ return a + b + c*d; }` was a
/// **live wrong emit** in this lane's first working draft, and
/// `scripts/sweep.d/36-fp-contract.py` caught it on its first run (2 of 4,597
/// FP cases). c2 does not lower the source tree `((a+b) + c*d)`; it
/// **reassociates the `+` chain** so the product is fused with the *first*
/// term and the rest are added afterwards:
///
/// ```text
///   a + b + c*d      fmadds f0,f3,f4,f1   ; c*d + a
///                    fadds  f1,f0,f2      ;      + b
/// ```
///
/// The rule is narrow, and every clause of it is a measured pair
/// (`work/w-fmadd/shapes_*.txt`, graded against real `c2.dll`):
///
/// | shape | fusing node | addend produced by | c2 |
/// |---|---|---|---|
/// | `a + b + c*d` | `Add` | `Add` | **reassociates** |
/// | `a - b + c + d*e` | `Add` | `Add` | **reassociates** |
/// | `a*b + c + d*e` | `Add` | `Add` (a fused one) | **reassociates** |
/// | `a - b + c*d` | `Add` | `Sub` | does not |
/// | `a - b - c + d*e` | `Add` | `Sub` | does not |
/// | `a + b - c*d` | `Sub` | `Add` | does not |
/// | `a + b + c - d*e` | `Sub` | `Add` | does not |
/// | `a*b + c*d + e` | `Add` | `Mul` | does not |
///
/// So the fence is exactly *"an `Add` fusing against an addend that came out
/// of an `Add`"*, and it is **priced two-sided** (`#1042`): what it costs is
/// the `+`-chain shapes whose product is not in the first term — every one of
/// which this lane measured as a MISMATCH before the fence went in, so the
/// refusal replaces a wrong emit rather than a green one. Modeling the
/// reassociation is a separate rule with its own read; `w13_freassoc.cpp`
/// already fences that family for the non-contracted case.
pub fn fp_node_plan(op: &IlOp, lhs: FpValKind, rhs: FpValKind) -> FpNodePlan {
    match op {
        IlOp::Mul => FpNodePlan::DeferProduct,
        IlOp::Add | IlOp::Sub if lhs.prod => {
            if matches!(op, IlOp::Add) && rhs.from_add {
                FpNodePlan::Refuse
            } else {
                FpNodePlan::FuseLeft
            }
        }
        IlOp::Add | IlOp::Sub if rhs.prod => {
            if matches!(op, IlOp::Add) && lhs.from_add {
                FpNodePlan::Refuse
            } else {
                FpNodePlan::FuseRight
            }
        }
        _ => FpNodePlan::Plain,
    }
}

/// Is this IL op a value push rather than a binary node?
fn fp_is_leaf_op(o: &IlOp) -> bool {
    matches!(o, IlOp::Load(_) | IlOp::Lit(_) | IlOp::FpLit { .. })
}

/// **How many machine instructions the contracted body emits.**
///
/// `None` when the stream is one this lowering refuses (an underflowing stack,
/// or a node with a product on *both* sides that is not a fusable `+`/`-` —
/// the only sources of which need source parentheses, which the parser already
/// rejects, so there is no witness for the order in which c2 would materialise
/// the two).
///
/// The count matters because the **last** instruction is the one that targets
/// `f1`; with contraction the instruction count is no longer the binary-node
/// count, and reading it off the wrong one silently leaves the result in a
/// scratch register.
pub fn fp_contract_instructions(ops: &[IlOp]) -> Result<usize, &'static str> {
    let mut stack: Vec<FpValKind> = Vec::new();
    let mut n = 0usize;
    for op in ops {
        if fp_is_leaf_op(op) {
            stack.push(FpValKind::default());
            continue;
        }
        let (rhs, lhs) = match (stack.pop(), stack.pop()) {
            (Some(r), Some(l)) => (r, l),
            _ => return Err(FP_EMPTY_STACK),
        };
        let from_add = matches!(op, IlOp::Add);
        match fp_node_plan(op, lhs, rhs) {
            FpNodePlan::Refuse => return Err(FP_REASSOCIATED),
            FpNodePlan::DeferProduct => {
                if lhs.prod && rhs.prod {
                    return Err(FP_TWO_PRODUCTS);
                }
                n += usize::from(lhs.prod) + usize::from(rhs.prod);
                stack.push(FpValKind { prod: true, from_add: false });
            }
            FpNodePlan::FuseLeft => {
                n += usize::from(rhs.prod) + 1;
                stack.push(FpValKind { prod: false, from_add });
            }
            FpNodePlan::FuseRight => {
                n += usize::from(lhs.prod) + 1;
                stack.push(FpValKind { prod: false, from_add });
            }
            FpNodePlan::Plain => {
                if lhs.prod && rhs.prod {
                    return Err(FP_TWO_PRODUCTS);
                }
                n += usize::from(lhs.prod) + usize::from(rhs.prod) + 1;
                stack.push(FpValKind { prod: false, from_add });
            }
        }
    }
    // A body whose ROOT is a product materialises it: `a*b*c` ends on an
    // `fmuls` into `f1`.
    if stack.last().map(|v| v.prod) == Some(true) {
        n += 1;
    }
    Ok(n)
}

/// The three refusals [`fp_contract_instructions`] can report, spelled once so the
/// counter and the emitter cannot describe the same condition two ways.
pub const FP_REASSOCIATED: &str =
    "an FP `+` chain whose product is not its first term: c2 REASSOCIATES the \
     chain so the product is fused first (`a + b + c*d` is \
     `fmadds f0,f3,f4,f1 ; fadds f1,f0,f2`); that reassociation is not modeled; \
     out of class";
pub const FP_TWO_PRODUCTS: &str =
    "an FP expression with a product on both sides of a node that cannot \
     contract: no witness for the order c2 materialises them in; out of class";
pub const FP_EMPTY_STACK: &str = "FP binary op: empty stack; out of class";

