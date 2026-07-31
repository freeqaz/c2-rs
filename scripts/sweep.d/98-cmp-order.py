# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the driver; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace. A fragment cannot see
# or touch another fragment's counter, and the driver fails if a fragment emits
# zero cases.
#
# ---- WCR: `return a->m() > b->n();` — the two-call comparator's ORDER relations
#
# `96-cmp-two-calls.py` sweeps the same production's `==`. This file sweeps `>`
# and `<`, the two relations `mcall-cmp-rel` measured at 760 (692 `>`, 68 `<`,
# and **0** each for `!=`, `<=`, `>=`). Only **67** of those 760 turned out to be
# integer or pointer comparisons — 693 compare `float`s and are a conditional
# branch plus a saved FPR, not a spine — so the axes below are what grades the
# rung, not the workload.
#
# Four axes exist here that `==` cannot reach, and each is swept because
# reverting its rule has to produce mismatches here or the axis is worth nothing.
# Measured, on this file's 356 cases, with each rule individually reverted:
#
#     signedness forced to `signed`                      122 mismatches
#     the `<`/`>` operand exchange dropped               118
#     the label surcharge dropped                        121
#     the surcharge moved to AFTER the `$M` triple       121
#     pointers classed as signed                          22
#
# The last two are the pair that matters most: the surcharge's SIZE and its
# PLACEMENT are two claims, and dropping it entirely and moving it four slots
# later cost the same 121 — so a run that graded only the total would have said
# nothing about where the slots go.
#
#   * **SIGNEDNESS, which is not in the operator byte.** Signed and unsigned `<`
#     are both opcode `0x22`; they lower to a five-word spine and a three-word
#     one. The only thing that separates them is the two callees' result TYPEs
#     (`86 41 74` against `86 42 75`), so every relational cell below is emitted
#     in both signednesses, and the `int`-result form of an *unsigned*
#     comparison is emitted too — the cell where the `2C` convert says `int` and
#     the operands are unsigned, i.e. where reading the wrong one of the two
#     type facts picks the wrong spine.
#   * **TWO INDEPENDENT OPERAND SWAPS THAT COMPOSE.** `<` is `>` with the spine's
#     operands exchanged; `lhs_first` is a *second* exchange, decided by c2's call
#     order rather than by the source. Getting the composition wrong turns `>`
#     into `<` in exactly the cells where both are non-trivial and nowhere else,
#     so the grid takes every ordered receiver pair in both source orders and
#     crosses it with both relations.
#   * **THE LABEL-COUNTER SURCHARGE.** A signed order comparator takes **two
#     extra counter slots ahead of its own `$M` triple** (stride 7 under `/Gy`,
#     6 packed, against 5/4 for its `==`, unsigned and arithmetic-tailed
#     siblings — `scripts/gt_cmp_rr.py --stride`). The lead moves the comparator's
#     own labels, so a single-function TU grades it; the *stride* only shows
#     against a following function, so those are swept too, with a
#     signed-relational neighbour that pays the surcharge twice.
#
#   * **THE POINTER CLASS.** Two pointers under an order relation take the
#     unsigned spine byte for byte — 66 of the rung's 67 realized functions —
#     and every pointee width lands on the same `86 43` in a result position.
#
# and the refusals, which have to be graded as refusals rather than assumed:
# `!=`/`<=`/`>=` (0 workload witnesses each, and the two order-or-equal relations
# are the ones whose bytes move with a `bool` result), a mixed-signedness or
# mixed-pointee comparison (c1xx inserts a `2C` convert on one side, in either
# position), and every operand type outside those two classes — of which the
# floating one is 693 functions.

DECL = (
    'struct E20 { int a, b, c, d, e; };\n'
    'struct U {\n'
    '  int m() const; int n() const; int o() const;\n'
    '  unsigned um() const; unsigned un() const; unsigned uo() const;\n'
    '  char mc() const; short ms() const; bool mb() const;\n'
    '  float mf() const; double md() const;\n'
    '  E20* mp() const; const void* mv() const; const char* mstr() const;\n'
    '  double* mdp() const; int* mip() const;\n'
    '  long ml() const; unsigned long mul() const;\n'
    '  int ma(int) const;\n'
    '};\n'
)

ORDER = ('>', '<')
REFUSED = ('!=', '<=', '>=')
# (accessor triple, the result types that keep the comparison in class)
SIGNS = (('m', 'n', 'o'), ('um', 'un', 'uo'))


def cases(emit):
    # 1. **THE TWO SWAPS, COMPOSED.** Every ordered pair of receivers out of
    #    three formals, both relations, both signednesses — and, crossed in, a
    #    leading `int` formal that moves the receivers' registers without moving
    #    their tokens. Half these cells reorder the calls relative to the source,
    #    which is the half a hand-written fixture does not contain.
    for meths in SIGNS:
        for rel in ORDER:
            for pad in ('', 'int z, '):
                for i in range(3):
                    for j in range(3):
                        emit('%sbool f(%sconst U* p0, const U* p1, const U* p2)'
                             ' { return p%d->%s() %s p%d->%s(); }\n'
                             % (DECL, pad, i, meths[0], rel, j, meths[1]))

    # 2. **`this`: parameter index 0, HIGHEST token.** The refuter for "ascending
    #    parameter index", now under the order relations, where a wrong call order
    #    also swaps the spine's operands and so changes two words instead of one.
    for meths in SIGNS:
        for rel in ORDER:
            for extra in ('', ', int k'):
                for lhs, rhs in (('%s()' % meths[0], 'a->%s()' % meths[1]),
                                 ('a->%s()' % meths[1], '%s()' % meths[0]),
                                 ('%s()' % meths[0], '%s()' % meths[1])):
                    emit('%sstruct H : U { bool q(const U* a%s) const;\n };\n'
                         'bool H::q(const U* a%s) const { return %s %s %s; }\n'
                         % (DECL, extra, extra, lhs, rel, rhs))

    # 3. **THE SAME RECEIVER TWICE** — equal tokens, the ordering rule's tie, at
    #    several formal positions and in both method orders.
    for meths in SIGNS:
        for rel in ORDER:
            for slot in range(0, 3):
                pre = ''.join('int q%d, ' % j for j in range(slot))
                for lhs, rhs in ((meths[0], meths[1]), (meths[1], meths[0])):
                    emit('%sbool f(%sconst U* p) { return p->%s() %s p->%s(); }\n'
                         % (DECL, pre, lhs, rel, rhs))

    # 4. **THE RESULT TYPE, against the OPERAND type.** These are two facts and
    #    only one of them picks the spine. `bool`, `int` and `unsigned` results
    #    are the same bytes over the same operands; an `int` result over
    #    *unsigned* operands is the cell that separates them.
    for meths in SIGNS:
        for rel in ORDER:
            for ret in ('bool', 'int', 'unsigned', 'long'):
                emit('%s%s f(const U* p, const U* q) { return p->%s() %s q->%s(); }\n'
                     % (DECL, ret, meths[0], rel, meths[1]))
    # `long` / `unsigned long` operands are the same two 4-byte classes spelled
    # differently — same spines, and the type ids differ from `int`'s.
    for meth in ('ml', 'mul'):
        for rel in ORDER:
            for ret in ('bool', 'int'):
                emit('%s%s f(const U* p, const U* q) { return p->%s() %s q->%s(); }\n'
                     % (DECL, ret, meth, rel, meth))

    # 5. **MIXED SIGNEDNESS MUST REFUSE**, in both positions — c1xx puts the `2C`
    #    convert on the left operand in one and on the right in the other, so the
    #    two are different grammar cells and a gate could catch only one.
    for rel in ORDER + REFUSED:
        for ret in ('bool', 'int'):
            emit('%s%s f(const U* p, const U* q) { return p->m() %s q->un(); }\n'
                 % (DECL, ret, rel))
            emit('%s%s f(const U* p, const U* q) { return p->um() %s q->n(); }\n'
                 % (DECL, ret, rel))

    # 6. **POINTER OPERANDS TAKE THE UNSIGNED SPINE, BYTE FOR BYTE** — 66 of this
    #    rung's 67 realized functions, and the row a fixture written from
    #    "a pointer is not an integer" would have refused. Every pointee width is
    #    swept because a result-position pointer TYPE is `86 43 <pointee id>` for
    #    all of them and a predicate keyed on the tag rather than the class would
    #    split on the pointee. Both relations, both source orders.
    PTRS = ('mp', 'mv', 'mstr', 'mdp', 'mip')
    for meth in PTRS:
        for rel in ORDER:
            emit('%sbool f(const U* p, const U* q) { return p->%s() %s q->%s(); }\n'
                 % (DECL, meth, rel, meth))
            emit('%sbool f(const U* p, const U* q) { return q->%s() %s p->%s(); }\n'
                 % (DECL, meth, rel, meth))
    # …and two DIFFERENT pointer types must refuse, the same way two different
    # integer signednesses do: the convert lands on one side or the other.
    for a in PTRS[1:]:
        for rel in ORDER:
            emit('%sbool f(const U* p, const U* q) { return p->mp() %s q->%s(); }\n'
                 % (DECL, rel, a))
            emit('%sbool f(const U* p, const U* q) { return p->%s() %s q->mp(); }\n'
                 % (DECL, a, rel))
    # …and a pointer against an INTEGER, which needs an explicit cast and so
    # carries a convert of its own.
    for rel in ORDER:
        emit('%sbool f(const U* p, const U* q) { return (unsigned)p->mp() %s q->um(); }\n'
             % (DECL, rel))

    # 6b. **THE OPERAND TYPES OUTSIDE BOTH CLASSES.** Narrow, boolean and
    #     floating operands must refuse under an order relation, on either side.
    #     The floating rows are the whole of what this rung leaves behind — 693
    #     functions, `fcmpu` plus a conditional branch plus a saved FPR — so they
    #     are graded as refusals rather than assumed to be ones.
    for meth in ('mc', 'ms', 'mb', 'mf', 'md'):
        for rel in ORDER:
            emit('%sbool f(const U* p, const U* q) { return p->%s() %s q->%s(); }\n'
                 % (DECL, meth, rel, meth))
            emit('%sbool f(const U* p, const U* q) { return p->%s() %s q->m(); }\n'
                 % (DECL, meth, rel))

    # 7. **THE REFUSED RELATIONS**, both signednesses, both result types, both
    #    source orders — so the boundary is graded rather than assumed, and so a
    #    later widening of `>=`/`<=` starts from a green baseline it can revert
    #    against. `>=` and `<=` are the two whose bytes move with a `bool` result.
    for meths in SIGNS:
        for rel in REFUSED:
            for ret in ('bool', 'int'):
                emit('%s%s f(const U* p, const U* q) { return p->%s() %s q->%s(); }\n'
                     % (DECL, ret, meths[0], rel, meths[1]))
                emit('%s%s f(const U* p, const U* q) { return q->%s() %s p->%s(); }\n'
                     % (DECL, ret, meths[1], rel, meths[0]))

    # 8. **THE LABEL SURCHARGE, against a NEIGHBOUR.** The lead moves the
    #    comparator's own `$M` numbers (so sections 1–4 already grade it), but the
    #    *stride* — what every later function in the TU sees — only shows here.
    #    The neighbour set deliberately includes a framed one (its own `$M`
    #    triple), a leaf that consumes one slot, a comparison leaf that consumes
    #    three, and a second comparator so the surcharge is paid twice in one TU.
    NEIGH = ('int g(int);\nint nb(int a) { return g(a) + 1; }\n',
             'int nb(int a) { return a + 1; }\n',
             'int g(int);\nvoid nb(int a) { g(a); g(a); }\n',
             'int nb(int a) { return a == 3; }\n',
             'int nb(unsigned a) { return a < 3u; }\n')
    for meths in SIGNS:
        for rel in ORDER:
            probe = ('bool f(const U* p, const U* q) { return p->%s() %s q->%s(); }\n'
                     % (meths[0], rel, meths[1]))
            second = ('bool f2(const U* p, const U* q) { return q->%s() %s p->%s(); }\n'
                      % (meths[1], rel, meths[0]))
            for nb in NEIGH:
                emit('%s%s%s' % (DECL, probe, nb))
                emit('%s%s%s' % (DECL, nb, probe))
                emit('%s%s%s%s' % (DECL, probe, second, nb))
    # …and the two signednesses in ONE translation unit, which is the only cell
    # where a function paying the surcharge and one not paying it are adjacent
    # in both orders.
    for a, b in (('m() > q->n()', 'um() > q->un()'),
                 ('um() > q->un()', 'm() > q->n()'),
                 ('m() < q->n()', 'um() < q->un()'),
                 ('m() > q->n()', 'mp() > q->mp()'),
                 ('mp() < q->mp()', 'm() < q->n()')):
        emit('%sbool f1(const U* p, const U* q) { return p->%s; }\n'
             'bool f2(const U* p, const U* q) { return p->%s; }\n' % (DECL, a, b))

    # 9. **ARGUMENTS, ARITY AND THE THIRD CALL** — the shared refusals, re-graded
    #    under an order relation because each of them is reached through a
    #    different byte position than `==` reaches it through.
    for rel in ORDER:
        emit('%sbool f(const U* p, const U* q, int k)'
             ' { return p->ma(k) %s q->m(); }\n' % (DECL, rel))
        emit('%sbool f(const U* p, const U* q, int k)'
             ' { return p->m() %s q->ma(k); }\n' % (DECL, rel))
        emit('%sbool f(const U* p, const U* q, const U* r)'
             ' { return p->m() %s q->m() + r->m(); }\n' % (DECL, rel))
        for n in (7, 8, 9):
            pre = ''.join('int q%d, ' % j for j in range(n - 2))
            emit('%sbool f(%sconst U* p, const U* q) { return p->m() %s q->m(); }\n'
                 % (DECL, pre, rel))

    # 10. **cv-QUALIFICATION AND THE RECEIVER'S `2C`**, the axes that produced
    #     live mis-emits with one receiver, now under an order relation.
    RECVS = ('const U* p', 'U* p', 'const U* const p', 'volatile U* p')
    for rel in ORDER:
        for a in RECVS:
            emit('%sbool f(%s, const U* q) { return p->m() %s q->m(); }\n'
                 % (DECL, a, rel))
            emit('%sbool f(%s, const U* q) { return q->m() %s p->m(); }\n'
                 % (DECL, a, rel))
        emit('%sbool f(void* v, const U* q)'
             ' { return ((const U*)v)->m() %s q->m(); }\n' % (DECL, rel))

    # 11. **SOURCE LINES AND BRACE SCOPES**, including a body past the one-byte
    #     line marker — `GAPS.md` §6 instance #1's axis.
    for pad in (0, 1, 70):
        nl = '\n' * pad
        for rel in ORDER:
            emit('%s%sbool f(const U* p, const U* q) { return p->m() %s q->n(); }\n'
                 % (DECL, nl, rel))
            emit('%s%sbool f(const U* p, const U* q) {\n  { return p->um() %s q->un(); }\n}\n'
                 % (DECL, nl, rel))

    # 12. **A FOUR-BYTE RECEIVER TOKEN.** 5,971 of WCB's 6,000 realized functions
    #     had one and no generated case had ever produced one; the order relations
    #     inherit that ordering rule unchanged, so the boundary is re-graded here
    #     with the spine that has two swaps instead of one. Three cases only —
    #     they are ~1 MB of source each and the axis is binary.
    WIDE = 'struct P {\n%s};\n' % ''.join(
        '  int w%d() const;\n' % j for j in range(33000))
    emit('%s%sbool f(const U* p, const U* q) { return p->m() > q->n(); }\n'
         % (DECL, WIDE))
    emit('%s%sbool f(const U* p, const U* q) { return q->n() < p->m(); }\n'
         % (DECL, WIDE))
    emit('%s%sbool f(const U* p, const U* q) { return p->um() > q->un(); }\n'
         % (DECL, WIDE))
