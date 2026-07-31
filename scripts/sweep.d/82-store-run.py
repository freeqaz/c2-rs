# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the driver; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace. A fragment cannot see
# or touch another fragment's counter, and the driver fails if a fragment emits
# zero cases.
#
# ---- W37: the store RUN, and the free return that ends it -------------------
#
# `80-store-leaf.py` sweeps ONE store against every width, displacement,
# designator and register. This fragment sweeps what only a *sequence* can vary,
# which is a different set of axes and is where a store run can go wrong:
#
#   * the ORDER of the statements against the order of the offsets — c2 emits
#     source order, and the only way to tell that apart from offset order is a
#     run whose two orders disagree;
#   * the WIDTH of each statement crossed with its neighbour's, including across
#     the register FILE (a `stfs` between two `stw`s), which no single-store case
#     can reach;
#   * cv-qualification and `#pragma pack(4)` **per statement independently** —
#     `docs/GAPS.md` §6's third live mis-emit was a packed 8-byte member behind a
#     4-byte tag folding an `lwz` at the wrong offset, because the tag carries
#     ALIGNMENT and the kind carries SIZE. Neither changes an operator or a
#     shape, so nothing else varies them;
#   * the TAIL — void, `return *this`, `return this`, `return <first formal>`,
#     a constructor's implicit result, and the refused `return <other formal>` —
#     crossed with the run length, because the tail is chosen by the source
#     construct and the run length by the body;
#   * the base register slot crossed with the run length, which is the axis
#     `GAPS.md` §6 instance #8 hid behind (every framed case had one parameter,
#     so an index and a register were the same number everywhere);
#   * SOURCE LINES between the statements and brace scopes around them —
#     instance #1's axis, and the statement-boundary walk is new code here.


def cases(emit):
    S = ('struct S { int a; int b; int c; int d; int e; int f; int g; '
         'char h; short i; long long j; int* k; float x; double y; };')

    # 1. Run length 1..8, every base slot 0..7. A run of N is N stores in source
    #    order out of consecutive argument registers; past the eighth formal the
    #    base is stack-homed and the whole body must refuse.
    MEM = ['a', 'b', 'c', 'd', 'e', 'f', 'g']
    for n in range(1, 8):
        for slot in range(0, 9):
            pre = ''.join('int p%d, ' % j for j in range(slot))
            args = ', '.join('int v%d' % j for j in range(n))
            body = ' '.join('s->%s = v%d;' % (MEM[j], j) for j in range(n))
            emit('%s\nvoid f(%sS* s, %s) { %s }\n' % (S, pre, args, body))

    # 2. Statement order against offset order. Ascending, descending, and every
    #    permutation of three — the ONE axis that separates "source order" from
    #    "offset order", and they agree on every ascending case.
    import itertools as _it
    for perm in _it.permutations(range(3)):
        body = ' '.join('s->%s = v%d;' % (MEM[i], i) for i in perm)
        emit('%s\nvoid f(S* s, int v0, int v1, int v2) { %s }\n' % (S, body))
    # …and the same three members written out of order through TWO bases, which
    # may alias at run time so c2 must keep every store.
    for perm in _it.permutations(range(3)):
        body = ' '.join('%s->%s = v%d;' % ('st'[i % 2], MEM[i], i) for i in perm)
        emit('%s\nvoid f(S* s, S* t, int v0, int v1, int v2) { %s }\n' % (S, body))

    # 3. Widths crossed with each other, in both orders, INCLUDING across the
    #    register file. A run of (int, float) and one of (float, int) exercise
    #    `stw`+`stfs` and `stfs`+`stw`, and the FP argument file is numbered over
    #    the FP parameters alone — so the second FP store's register depends on
    #    how many FP parameters precede it, which only a run can vary.
    WIDTHS = [('int', 'a'), ('char', 'h'), ('short', 'i'), ('long long', 'j'),
              ('int*', 'k'), ('float', 'x'), ('double', 'y')]
    for (t0, m0) in WIDTHS:
        for (t1, m1) in WIDTHS:
            if m0 == m1:
                continue
            emit('%s\nvoid f(S* s, %s v0, %s v1) { s->%s = v0; s->%s = v1; }\n'
                 % (S, t0, t1, m0, m1))
    # …and every ordered triple over the two files, which is where a run can put
    # an FP store between two GPR ones and shift nothing it should shift.
    for t in _it.permutations([('int', 'a'), ('float', 'x'), ('double', 'y'),
                               ('short', 'i')], 3):
        args = ', '.join('%s v%d' % (t[j][0], j) for j in range(3))
        body = ' '.join('s->%s = v%d;' % (t[j][1], j) for j in range(3))
        emit('%s\nvoid f(S* s, %s) { %s }\n' % (S, args, body))

    # 3b. PURE-file runs of three and four at MIXED widths, both files. This is
    #     the region the mixed-file gate leans on: a run of 3+ is admitted only
    #     when every statement uses one register file, so "source order holds for
    #     a pure run" must be graded past length 2 rather than assumed. Probed
    #     exhaustively once at 1,500 cases (every ordered selection of 3–5 from
    #     five FP and six GPR members, all byte-exact); this is the standing
    #     slice of it.
    Z = ('struct Z { float fa; double db; float fc; double dd; '
         'int ia; char ch; short sh; long long ll; };')
    ZFP = [('float', 'fa'), ('double', 'db'), ('float', 'fc'), ('double', 'dd')]
    ZG = [('int', 'ia'), ('char', 'ch'), ('short', 'sh'), ('long long', 'll')]
    for pool in (ZFP, ZG):
        for k in (3, 4):
            for sel in _it.permutations(pool, k):
                args = ', '.join('%s v%d' % (sel[j][0], j) for j in range(k))
                body = ' '.join('s->%s = v%d;' % (sel[j][1], j) for j in range(k))
                emit('%s\nvoid f(Z* s, %s) { %s }\n' % (Z, args, body))

    # 4. cv-qualification at every position of the run, independently. It changes
    #    no operator and no shape and it DOES change the tags this parser reads.
    CV = ['', 'const ', 'volatile ']
    for q0 in CV:
        for q1 in CV:
            for qb in ['', 'const ']:
                emit('struct Q { int a; int b; };\n'
                     'void f(Q* %ss, %sint v0, %sint v1) { s->a = v0; s->b = v1; }\n'
                     % (qb, q0, q1))
    # …and a volatile MEMBER at each position, which is a memory object on the
    # destination side rather than on the value side.
    emit('struct QV { volatile int a; int b; };\n'
         'void f(QV* s, int u, int v) { s->a = u; s->b = v; }\n')
    emit('struct QV2 { int a; volatile int b; };\n'
         'void f(QV2* s, int u, int v) { s->a = u; s->b = v; }\n')
    # …and a volatile BASE, which c2 homes in the frame (the thirteenth live
    # mis-emit's position).
    emit('struct QB { int a; int b; };\n'
         'void f(QB* volatile s, int u, int v) { s->a = u; s->b = v; }\n')

    # 5. `#pragma pack(4)`, at every run length and with the wide member at every
    #    position. The tag carries ALIGNMENT and the kind carries SIZE, so a
    #    packed `long long` behind a 4-byte tag is the one case where reading the
    #    wrong byte lands the store at the wrong offset — and a RUN multiplies
    #    the offsets that follow it.
    PACKED = ('#pragma pack(4)\n'
              'struct P { char c; int i; long long l; double d; short s; int t; };\n'
              '#pragma pack()\n')
    PMEM = [('char', 'c'), ('int', 'i'), ('long long', 'l'), ('double', 'd'),
            ('short', 's'), ('int', 't')]
    for i in range(len(PMEM)):
        for j in range(len(PMEM)):
            if i == j:
                continue
            emit('%svoid f(P* p, %s v0, %s v1) { p->%s = v0; p->%s = v1; }\n'
                 % (PACKED, PMEM[i][0], PMEM[j][0], PMEM[i][1], PMEM[j][1]))
    emit('%svoid f(P* p, char a, int b, long long c, double d, short e)\n'
         '{ p->c = a; p->i = b; p->l = c; p->d = d; p->s = e; }\n' % PACKED)

    # 6. The TAIL, crossed with the run length. Six spellings: void, `return
    #    *this`, `return this`, `return <first formal>`, a constructor's implicit
    #    result (which sits AFTER the `29` rather than ahead of the `3A`), and
    #    `return <a later formal>`, which is a register move and must refuse.
    T = 'struct T { int a; int b; int c; '
    for n in range(1, 4):
        sets = ' '.join('%s = v%d;' % (MEM[j], j) for j in range(n))
        args = ', '.join('int v%d' % j for j in range(n))
        decl = T + 'T& r(%s); T* p(%s); void v(%s); int k(%s); };\n' % (
            args, args, args, args)
        emit('%sT& T::r(%s) { %s return *this; }\n' % (decl, args, sets))
        emit('%sT* T::p(%s) { %s return this; }\n' % (decl, args, sets))
        emit('%svoid T::v(%s) { %s }\n' % (decl, args, sets))
        emit('%sint T::k(%s) { %s return v0; }\n' % (decl, args, sets))
    # A free function returning its first formal, and one returning a later one.
    for n in range(1, 4):
        sets = ' '.join('s->%s = v%d;' % (MEM[j], j) for j in range(n))
        args = ', '.join('int v%d' % j for j in range(n))
        emit('%sS* f(S* s, %s) { %s return s; }\n' % (S, args, sets))
        emit('%sS* f(int k, S* s, %s) { %s return s; }\n' % (S, args, sets))
        emit('%sint f(S* s, %s) { %s return v0; }\n' % (S, args, sets))
        emit('%sint f(int k, S* s, %s) { %s return k; }\n' % (S, args, sets))
    # Constructors: an assignment body, a member initializer list, and the two
    # mixed — the same IL and four ways of writing it.
    for n in range(1, 4):
        args = ', '.join('int v%d' % j for j in range(n))
        sets = ' '.join('%s = v%d;' % (MEM[j], j) for j in range(n))
        init = ', '.join('%s(v%d)' % (MEM[j], j) for j in range(n))
        decl = 'struct K { int a; int b; int c; K(%s); };\n' % args
        emit('%sK::K(%s) { %s }\n' % (decl, args, sets))
        emit('%sK::K(%s) : %s { }\n' % (decl, args, init))
        if n > 1:
            head = '%s(v0)' % MEM[0]
            rest = ' '.join('%s = v%d;' % (MEM[j], j) for j in range(1, n))
            emit('%sK::K(%s) : %s { %s }\n' % (decl, args, head, rest))

    # 7. SOURCE LINES and brace scopes between the statements. The statement
    #    boundary walk is new code, and a line marker is exactly the kind of
    #    field that is invisible until a probe puts the statements on separate
    #    lines (`GAPS.md` §6 instance #1 was a line-70 marker read as a token).
    emit('%svoid f(S* s, int u, int v) {\n  s->a = u;\n  s->b = v;\n}\n' % S)
    emit('%svoid f(S* s, int u, int v) {\n\n  s->a = u;\n\n\n  s->b = v;\n\n}\n' % S)
    emit('%svoid f(S* s, int u, int v) { s->a = u; { s->b = v; } }\n' % S)
    emit('%svoid f(S* s, int u, int v) { { s->a = u; } s->b = v; }\n' % S)
    emit('%svoid f(S* s, int u, int v) { { s->a = u; s->b = v; } }\n' % S)
    emit('%svoid f(S* s, int u, int v) { { s->a = u; } { s->b = v; } }\n' % S)
    emit('%svoid f(S* s, int u, int v) {\n  { \n    s->a = u;\n  }\n'
         '  {\n    s->b = v;\n  }\n}\n' % S)
    emit('%svoid f(S* s, int u, int v) { { { s->a = u; s->b = v; } } }\n' % S)
    # …and the same, with the run at source line 70, where the first `0x46` is
    # the line marker's payload rather than the `this` group.
    emit('struct L70 { int a; int b; void m(int u, int v); };\n' + '\n' * 62 +
         'void L70::m(int u, int v) { a = u; b = v; }\n')

    # 8. The REFUSED value forms, one per rule, at run length 2 and 3. These emit
    #    nothing, so they cannot mismatch — what they grade is that the gate is
    #    still closed after a later rung widens something near it, which is the
    #    failure mode a negative fixture has silently had before.
    REFUSE = [
        's->a = 1; s->b = 2;',                    # two literals: `li` hoisting
        's->a = u; s->b = 2;',                    # one literal: still hoisted
        's->a = 1; s->b = u;',                    # and the stores get REORDERED
        's->a = u; s->a = v;',                    # dead store: c2 emits ONE
        's->a = u; s->b = u + v;',                # computed value
        's->a = u; s->b = o->a;',                 # value is an indirect load
        's->a = u; s->b += v;',                   # compound assign, a `0x19`
        's->a = u; s->b = -v;',                   # unary minus
        's->a = u; g(v);',                        # a call in the run
        's->a = u; if (v) s->b = v;',             # a branch in the run
    ]
    for body in REFUSE:
        emit('struct O { int a; int b; };\n%s\nvoid g(int);\n'
             'void f(S* s, O* o, int u, int v) { %s }\n' % (S, body))

    # 9. A store run beside every OTHER thing a TU can carry, in both orders —
    #    the `_fltused` marker and the per-TU label counter are decided by the
    #    whole translation unit, and an FP store inside a run is a new way to
    #    make a TU FP-touching. `cross_sweep.sh` pairs the accepted families; this
    #    pins the orderings a run specifically adds.
    RUN_GPR = 'void R(S* s, int u, int v) { s->a = u; s->b = v; }\n'
    RUN_FP = 'void RF(S* s, float u, int v) { s->x = u; s->b = v; }\n'
    RUN_THIS = ('struct RT { int a; int b; RT& r(int u, int v); };\n'
                'RT& RT::r(int u, int v) { a = u; b = v; return *this; }\n')
    OTHERS = [
        'int A(int x) { return x + 1; }\n',
        'float C(float x, float y) { return x * y; }\n',
        'void g(int);\nvoid B(int x) { g(x); }\n',
        'int h(int);\nint D(int x) { return h(x) + 1; }\n',
    ]
    for run in (RUN_GPR, RUN_FP, RUN_THIS):
        for other in OTHERS:
            emit('%s\n%s%s' % (S, run, other))
            emit('%s\n%s%s' % (S, other, run))
    emit('%s\n%s%s' % (S, RUN_GPR, RUN_FP))
    emit('%s\n%s%s' % (S, RUN_FP, RUN_GPR))
