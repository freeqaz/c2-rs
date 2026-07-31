# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the driver; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace. A fragment cannot see
# or touch another fragment's counter, and the driver fails if a fragment emits
# zero cases.
#
# ---- WCB: `return a->m() == b->n();` — two calls, a value live across one ---
#
# The first **Class B** production in the port: the first call's result survives
# the second `bl`, so the body carries two `std`/`ld` pairs and a 112-byte frame.
# `20-compare.py` sweeps the comparison LEAF (`a <rel> k`, one register against a
# literal); `72-member-call.py` and `73-framed-member-call.py` sweep the member
# call with **one** call in the body. Nothing before this crossed the two, and
# three of the axes below cannot be reached from either:
#
#   * **THE CALL ORDER, which c2 chooses and the source does not.** The two calls
#     are emitted in ascending order of the *receiver's IL token*, so the source's
#     left operand may be emitted first or second and the spine's `subf` operands
#     swap accordingly. That is invisible in every body where the two receivers
#     appear in ascending order already, which is what any hand-written fixture
#     looks like. The grid therefore takes **every ordered pair** of receivers out
#     of a formal list, in **both** source orders, so half the cells are the
#     reordering ones — and crosses that with a leading `int` formal, which moves
#     the receivers' registers without moving their tokens.
#   * **`this`, which has parameter index 0 and the HIGHEST token.** c1xx numbers
#     the implicit receiver after the declared formals, so a member function whose
#     body compares `m()` against `a->m()` orders the calls the opposite way from
#     what its parameter list suggests. This is the cell that separates "ascending
#     token" from "ascending parameter index", and `GAPS.md` §6 records four
#     separate defects where a formal's INDEX and its REGISTER were the same
#     number in every fixture. Both source orders, at three arities.
#   * **THE SAME RECEIVER TWICE.** `p->m() == p->n()` still saves two GPRs — the
#     receiver *and* the first result — and its two calls have equal tokens, which
#     is the one cell where the ordering rule has to fall back to IL order. Both
#     method orders, at several formal positions.
#
# and three more that vary something changing no operator and no shape, which is
# the class of axis that found six live mis-emits last session:
#
#   * **the result's TYPE.** `bool`, `int` and `unsigned` are the *same bytes* and
#     three different IL spellings — `bool` carries no `2C` convert and annotates
#     `41 <int1>`, the others convert and annotate `41 <int4>`. `char`, `short`,
#     `float`, `double` and a pointer result must refuse.
#   * **the RELATION.** Only `==` is in class; `!=` is three more words and the
#     four order relations are five-word sign-sum spines with two `bool`-sensitive
#     cells. Every relation is swept so the boundary is graded and not assumed.
#   * **cv-qualification and the receiver's `2C` conversion**, the axes
#     `34-volatile-formal.py` and `73-framed-member-call.py` found live mis-emits
#     on, now with two receivers in one body instead of one.

DECL = (
    'struct E20 { int a, b, c, d, e; };\n'
    'struct U {\n'
    '  int m() const; int n() const; int o() const;\n'
    '  unsigned mu() const; char mc() const; short ms() const;\n'
    '  bool mb() const; float mf() const; double md() const;\n'
    '  E20* mp() const; int ma(int) const; int m2(int, int) const;\n'
    '};\n'
)

RELS = ('==', '!=', '<', '<=', '>', '>=')


def cases(emit):
    # 1. THE CALL ORDER. Every ordered pair of receivers out of three formals, in
    #    both source orders, with distinguishable methods so the emitted order is
    #    readable from the relocations — crossed with a leading `int` formal that
    #    shifts the registers but not the tokens.
    METHS = ('m', 'n', 'o')
    for pad in ('', 'int z, ', 'int z, int y, '):
        for i in range(3):
            for j in range(3):
                for mi in range(2):
                    for mj in range(2):
                        emit('%sbool f(%sconst U* p0, const U* p1, const U* p2)'
                             ' { return p%d->%s() == p%d->%s(); }\n'
                             % (DECL, pad, i, METHS[mi], j, METHS[mj]))

    # 2. `this`: parameter index 0, highest token. The refuter for "ascending
    #    parameter index", both source orders, three arities, and the case where
    #    `this` is not a receiver at all (so nothing saves it).
    for extra in ('', ', const U* b', ', int k, const U* b'):
        for lhs, rhs in (('m()', 'a->m()'), ('a->m()', 'm()'),
                         ('m()', 'a->n()'), ('a->n()', 'm()'),
                         ('m()', 'm()')):
            emit('%sstruct H : U { bool q(const U* a%s) const;\n };\n'
                 'bool H::q(const U* a%s) const { return %s == %s; }\n'
                 % (DECL, extra, extra, lhs, rhs))
    for extra in ('', ', int k'):
        for lhs, rhs in (('a->m()', 'b->m()'), ('b->m()', 'a->m()'),
                         ('a->m()', 'b->n()'), ('b->n()', 'a->m()')):
            emit('%sstruct H : U { bool q(const U* a, const U* b%s) const;\n };\n'
                 'bool H::q(const U* a, const U* b%s) const { return %s == %s; }\n'
                 % (DECL, extra, extra, lhs, rhs))

    # 3. THE SAME RECEIVER TWICE — equal tokens, the ordering rule's tie — at
    #    several formal positions, both method orders.
    for slot in range(0, 4):
        pre = ''.join('int q%d, ' % j for j in range(slot))
        for lhs, rhs in (('m', 'n'), ('n', 'm'), ('m', 'm')):
            emit('%sbool f(%sconst U* p) { return p->%s() == p->%s(); }\n'
                 % (DECL, pre, lhs, rhs))

    # 4. THE RESULT'S TYPE. `bool`, `int` and `unsigned` are the same bytes and
    #    three IL spellings; the narrow, floating and pointer results must refuse.
    for ret in ('bool', 'int', 'unsigned', 'long', 'char', 'short',
                'float', 'double', 'E20*'):
        for meth in ('m', 'mu'):
            emit('%s%s f(const U* p, const U* q) { return p->%s() == q->%s(); }\n'
                 % (DECL, ret, meth, meth))
    # …and the OPERANDS' types, which decide signedness and whether the operand
    # vocabulary can spell them at all.
    for meth in ('m', 'mu', 'mc', 'ms', 'mb', 'mf', 'md', 'mp'):
        for ret in ('bool', 'int'):
            emit('%s%s f(const U* p, const U* q) { return p->%s() == q->%s(); }\n'
                 % (DECL, ret, meth, meth))
            emit('%s%s f(const U* p, const U* q) { return p->%s() == q->m(); }\n'
                 % (DECL, ret, meth))

    # 5. THE RELATION, every one, crossed with the result type and the call
    #    order. Only `==` is in class and the other five must refuse — but they
    #    must refuse for BOTH orders, or the gate is order-dependent.
    for rel in RELS:
        for ret in ('bool', 'int'):
            emit('%s%s f(const U* p, const U* q) { return p->m() %s q->n(); }\n'
                 % (DECL, ret, rel))
            emit('%s%s f(const U* p, const U* q) { return q->n() %s p->m(); }\n'
                 % (DECL, ret, rel))
            emit('%s%s f(const U* p) { return p->m() %s p->n(); }\n'
                 % (DECL, ret, rel))

    # 6. THREE calls — the `__savegprlr_29` helper class, which must refuse
    #    however the third one is reached.
    for expr in ('p->m() == q->m() + r->m()',
                 'p->m() + q->m() == r->m()',
                 'p->m() == (q->m() == r->m())'):
        emit('%sbool f(const U* p, const U* q, const U* r) { return %s; }\n'
             % (DECL, expr))

    # 7. EXPLICIT ARGUMENTS on one or both calls: the marshalling would interleave
    #    with the callee-saved move and must refuse. Both sides, both orders.
    for lhs, rhs in (('ma(k)', 'm()'), ('m()', 'ma(k)'), ('ma(k)', 'ma(k)'),
                     ('m2(k, k)', 'm()'), ('m()', 'm2(j, k)')):
        emit('%sbool f(const U* p, const U* q, int j, int k)'
             ' { return p->%s == q->%s; }\n' % (DECL, lhs, rhs))

    # 8. cv-qualification and the receiver's `2C` conversion, on either receiver
    #    or both — the axes that produced live mis-emits with ONE receiver.
    RECVS = ('const U* p', 'U* p', 'const U* const p', 'volatile U* p',
             'const volatile U* p')
    for a in RECVS:
        emit('%sbool f(%s, const U* q) { return p->m() == q->m(); }\n' % (DECL, a))
        emit('%sbool f(%s, const U* q) { return q->m() == p->m(); }\n' % (DECL, a))
    CASTS = (('void* v', '((const U*)v)'), ('const U* p', 'const_cast<U*>(p)'),
             ('char* c', '((const U*)c)'))
    for parm, expr in CASTS:
        emit('%sbool f(%s, const U* q) { return %s->m() == q->m(); }\n'
             % (DECL, parm, expr))
        emit('%sbool f(%s, const U* q) { return q->m() == %s->m(); }\n'
             % (DECL, parm, expr))
        emit('%sbool f(%s) { return %s->m() == %s->n(); }\n'
             % (DECL, parm, expr, expr))

    # 9. Nine formals — one past the register file, where a receiver is stack-homed
    #    and reading it is `lwz`, not a register move.
    for n in (6, 7, 8, 9):
        pre = ''.join('int q%d, ' % j for j in range(n - 2))
        emit('%sbool f(%sconst U* p, const U* q) { return p->m() == q->m(); }\n'
             % (DECL, pre))

    # 10. Source lines and brace scopes around the whole thing — `GAPS.md` §6
    #     instance #1's axis, including a body past the one-byte line marker.
    for pad in (0, 1, 3, 70):
        nl = '\n' * pad
        emit('%s%sbool f(const U* p, const U* q) { return p->m() == q->m(); }\n'
             % (DECL, nl))
        emit('%s%sbool f(const U* p, const U* q) {\n  return p->m() == q->m();\n}\n'
             % (DECL, nl))
        emit('%s%sbool f(const U* p, const U* q) { { return p->m() == q->m(); } }\n'
             % (DECL, nl))

    # 11. A NEIGHBOUR in the same TU, which is where the `/Gy` label counter and
    #     the symbol-table order are actually graded: this shape introduces two
    #     callee externals and takes the plain framed stride of 5, and a wrong
    #     surcharge only shows against a following function.
    NEIGH = ('int g(int);\nint nb(int a) { return g(a) + 1; }\n',
             'int nb(int a) { return a + 1; }\n',
             'int g(int);\nvoid nb(int a) { g(a); g(a); }\n',
             'int nb(int a) { return a == 3; }\n',
             'int nb(int a) { return a <= 3; }\n')
    for nb in NEIGH:
        emit('%sbool f(const U* p, const U* q) { return p->m() == q->n(); }\n%s'
             % (DECL, nb))
        emit('%s%sbool f(const U* p, const U* q) { return p->m() == q->n(); }\n'
             % (DECL, nb))
        emit('%sbool f(const U* p, const U* q) { return p->m() == q->n(); }\n'
             'bool f2(const U* p, const U* q) { return q->n() == p->m(); }\n%s'
             % (DECL, nb))

    # 12. **HOW MANY SYMBOLS THE TU DECLARED BEFORE THIS FUNCTION**, an axis that
    #     appears nowhere in the source of the function under test.
    #
    #     The two calls are ordered by the order c1xx NUMBERED their receivers,
    #     and the first version of that rule compared the numbers
    #     `read_token_var` hands back. A token's two-byte form is little-endian,
    #     so those ASCEND for 255 consecutive tokens and then DROP by ~65,000: a
    #     body whose two receivers straddle a low-byte boundary ordered the
    #     opposite way from a body one symbol earlier. That was a live
    #     `Port=Mismatch @ offset 8`, and no other axis in this file can reach it
    #     — every other cell has both receivers inside one low-byte range, which
    #     is what a small TU looks like.
    #
    #     **This run is what grades the ordering rule itself.** Every cell is a
    #     member function, so reverting `alloc_rank`'s `this`-is-last arm to a
    #     plain parameter index produces **272 mismatches** here, and restoring
    #     the token-value comparison produces **2** — the two straddling
    #     alignments. A green run of this section is a statement about the rule
    #     and not only about the shape.
    #
    #     **The range is 264 and not 28, because the first attempt at this axis
    #     did not separate the rules.** Each filler declaration advances the token
    #     counter by exactly one (measured), so a run has to be longer than a full
    #     low byte to contain a boundary at all — a 28-case run moved the tokens
    #     from 0x0A11 to 0x0A2C and graded green with the ordering key
    #     deliberately reverted. The step must also stay at **one**: the two
    #     receivers are two tokens apart, so only two alignments in each 256
    #     straddle, and a coarser filler steps over them. The member-function form
    #     is used because `this` is numbered last and is therefore the receiver
    #     that ends up on the far side of a wrap.
    for k in range(264):
        fill = ''.join('  int z%d() const;\n' % j for j in range(k))
        emit('%sstruct H {\n  int m() const;\n%s  bool q(const U* a) const;\n};\n'
             'bool H::q(const U* a) const { return m() == a->m(); }\n'
             % (DECL, fill))
    # …and both source orders at a handful of widths, so the operand roles are
    # swept beside the wrap rather than only across it.
    for k in (0, 1, 2, 3):
        fill = ''.join('  int z%d() const;\n' % j for j in range(k))
        for lhs, rhs in (('m()', 'a->m()'), ('a->m()', 'm()')):
            emit('%sstruct H {\n  int m() const;\n%s  bool q(const U* a) const;\n};\n'
                 'bool H::q(const U* a) const { return %s == %s; }\n'
                 % (DECL, fill, lhs, rhs))

    # 13. **A FOUR-BYTE TOKEN.** Every other case in this file — and every case in
    #     every other fragment — declares a few dozen symbols, so all their IL
    #     tokens take the two-byte form. A real translation unit declares tens of
    #     thousands: **5,971 of this rung's 6,000 workload functions have four-byte
    #     receiver tokens**, a form no generated case had ever produced, and
    #     `read_token_var`'s width choice moves every cursor downstream of it.
    #     That asymmetry — the whole realized population on one side of a
    #     representation boundary and the whole graded corpus on the other — is
    #     what `docs/GAPS.md` §6 keeps recording, so it is swept rather than
    #     argued about.
    #
    #     The threshold is 0x8000 symbols; 33,000 method declarations clear it with
    #     margin. Three cases only: they are ~1 MB of source each, and the axis
    #     they add is the token WIDTH, which is binary.
    WIDE = 'struct P {\n%s};\n' % ''.join(
        '  int w%d() const;\n' % j for j in range(33000))
    emit('%s%sbool f(const U* p, const U* q) { return p->m() == q->n(); }\n'
         % (DECL, WIDE))
    emit('%s%sbool f(const U* p, const U* q) { return q->n() == p->m(); }\n'
         % (DECL, WIDE))
    emit('%s%sstruct H {\n  int m() const;\n  bool q(const U* a) const;\n};\n'
         'bool H::q(const U* a) const { return m() == a->m(); }\n'
         % (DECL, WIDE))
