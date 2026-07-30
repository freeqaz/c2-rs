use crate::func::IlOp;

/// The largest substituted operand stream accepted, so that a chain of
/// assignments each doubling the previous cannot blow up.
pub(crate) const MAX_SUBST_OPS: usize = 32;

/// Integer argument registers r3..r10. A ninth parameter is stack-homed, which
/// needs a frame; mirrors `c2_core::codegen::ARG_REGS`.
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
    // `return b;` — a bare parameter that is not the first needs a register move.
    // `return a;` is free, since it is already in r3.
    if let [IlOp::Load(t)] = ops {
        if params.first() != Some(t) {
            return Some(if params.contains(t) {
                // In an argument register, just not r3: one `mr r3,rN`.
                "expr-out-of-class-bare-nonfirst-formal"
            } else {
                // Not an argument at all — a global, a `.sy` local, a token from
                // a construct this class does not model. No lowering is implied.
                "expr-out-of-class-bare-nonformal"
            });
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
            IlOp::Mul if rhs_lit || (i == 1 && lhs_lit) => {
                return Some("expr-out-of-class-mul-by-lit")
            }
            IlOp::Sub if i == 2 && lhs_lit => return Some("expr-out-of-class-lit-minus-reg"),
            _ => {}
        }
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

        // `return a;` — already in r3, in class.
        assert_eq!(ctx(&[IlOp::Load(0x10)]), None);
        // `return b;` — one `mr r3,r4`.
        assert_eq!(
            ctx(&[IlOp::Load(0x11)]),
            Some("expr-out-of-class-bare-nonfirst-formal")
        );
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
        assert_eq!(
            ctx(&[IlOp::Lit(3), IlOp::Load(0x10), IlOp::Sub]),
            Some("expr-out-of-class-lit-minus-reg")
        );
        // The predicate and the key must never disagree: one is `.is_some()` of
        // the other, and this is the test that keeps it that way.
        for ops in [
            vec![IlOp::Load(0x10)],
            vec![IlOp::Load(0x11)],
            vec![IlOp::Lit(-0x8001)],
            vec![IlOp::Load(0x10), IlOp::Lit(3), IlOp::Mul],
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
