# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the driver; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace.
#
# **The multiply-add contraction** (lane `w-fmadd`, board #3792). c2 fuses every
# `*` that feeds a `+`/`-` — `docs/CODEGEN_W13_FLOAT.md` §3.3 — and which of
# `fmadd`, `fmsub` and `fnmsub` it picks depends on WHICH SIDE of the operator
# the product was written on, because `fnmsub` computes `B − A*C` rather than
# `A*C − B`. `30-fp-leaves.py` sweeps three-leaf chains and therefore covers the
# side question for `+`/`-` at depth one; what it cannot reach is the shape that
# decides the rest of the rule:
#
#   * a product on BOTH sides (`a*b + c*d`) — which one is fused and which is
#     materialised into a scratch register first;
#   * a product feeding a product feeding an add (`a*b*c + d`) — the deferred
#     product that has to be committed early;
#   * a contraction that is NOT the last instruction (`a*b + c + d`), which is
#     the case that decides whether the result lands in `f1`.
#
# All three need four leaves, so they need their own fragment.


def cases(emit):
    sig4 = "%s f(%s a, %s b, %s c, %s d) { return %s; }\n"
    sig3 = "%s f(%s a, %s b, %s c) { return %s; }\n"
    for ty in ('float', 'double'):
        t4 = (ty,) * 5
        t3 = (ty,) * 4
        # Every left-associated four-leaf chain over the three operators that
        # can contract. 27 per type; most are refused by the ascending-leaf or
        # repeated-leaf gates and that is fine — a refusal is not a failure
        # here, a MISMATCH is.
        for o1 in ('+', '-', '*'):
            for o2 in ('+', '-', '*'):
                for o3 in ('+', '-', '*'):
                    emit(sig4 % (t4 + ("a %s b %s c %s d" % (o1, o2, o3),)))
        # Two products, both orders. `a*b + c*d` and `c*d + a*b` emit the SAME
        # three words in c2; only the first is in class here (the second has
        # descending leaves), and the sweep is what keeps the second refused
        # rather than silently emitted in source order.
        for o in ('+', '-'):
            emit(sig4 % (t4 + ("a * b %s c * d" % o,)))
            emit(sig4 % (t4 + ("c * d %s a * b" % o,)))
        # The product's side, at three leaves, spelled out rather than left to
        # 30-fp-leaves' permutation list — this is the `fmsub` vs `fnmsub`
        # discriminator and it is worth naming.
        for o in ('+', '-'):
            for form in ('a * b %s c', 'c %s a * b', 'b * c %s a', 'a %s b * c'):
                emit(sig3 % (t3 + (form % o,)))
        # A product under a division: `/` never contracts, so the product must
        # be materialised as an `fmul` first.
        emit(sig3 % (t3 + ("a * b / c",)))
        emit(sig3 % (t3 + ("a / b * c",)))
