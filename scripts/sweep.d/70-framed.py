# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the driver; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace. A fragment cannot see
# or touch another fragment's counter (the `n`-shadowing trap that silently
# overwrote 1,233 already-written cases is unrepresentable here), and the driver
# fails if a fragment emits zero cases.


def cases(emit):
    # ---- W-UNW-1: framed functions, `.pdata`, and the compiler label counter --------
    # The framed class was single-function-per-TU for its whole life, so nothing here
    # had ever been swept: the `.pdata` record, its relocation, the `$M`/`$T` label
    # numbers and — the part that actually broke — the `bl` displacement, which
    # follows `disp = −(own .text offset)` and was hardcoded to the offset of the one
    # body the port could emit.
    #
    # Two axes, and the cross product is the point. **Position** decides the `bl`
    # word and the `.text` offset of both `$M` labels; **the kinds of the preceding
    # functions** decide the label numbers, because the counter is consumed by every
    # function whether or not it emits a label. A sweep over position alone with one
    # leaf kind would grade neither.
    FRAMED_LEAFMATES = [
        'int L%d(int a) { return a + %d; }',
        'int L%d(int a) { return a - %d; }',
        'int L%d(int a) { return a; }',
        'int L%d(int a, int b) { return b; }',
        'int L%d(int a, int b) { return a + b; }',
        'int L%d(int a) { return g(a); }',
        'int L%d(int *p) { return *p; }',
        'struct S%d { int m; };\nint L%d(S%d *p) { return p->m; }',
    ]
    def leafmate(kind, ix):
        t = FRAMED_LEAFMATES[kind]
        if t.startswith('struct'):
            return t % (ix, ix, ix)
        return t % ((ix, ix + 1) if '%d;' in t or t.count('%d') == 2 else (ix,))
    def framed_fn(ix, k, callee):
        return 'int F%d(int a) { return %s(a) + %d; }' % (ix, callee, k)

    # 1. Runs of framed functions on their own: 1..3 of them, shared and distinct
    #    callees, and `+k` values that move only the `addi` immediate.
    for count in (1, 2, 3):
        for distinct in (0, 1):
            callees = ['g%d' % j if distinct else 'g' for j in range(count)]
            decls = ''.join('int %s(int);\n' % c for c in sorted(set(callees)))
            body = '\n'.join(framed_fn(j, j + 1, callees[j]) for j in range(count))
            emit(decls + body + '\n')

    # 2. One framed function at every position among 0..3 leafmates of one kind.
    for kind in range(len(FRAMED_LEAFMATES)):
        for mates in range(4):
            for pos in range(mates + 1):
                parts = []
                for j in range(mates + 1):
                    parts.append(framed_fn(j, 1, 'g') if j == pos else leafmate(kind, j))
                emit('int g(int);\n' + '\n'.join(parts) + '\n')

    # 3. Two framed functions with leafmates of MIXED kinds between and around them —
    #    the shape where a per-kind counter stride error and a position error can
    #    cancel in one arrangement and not another.
    for k1 in range(len(FRAMED_LEAFMATES)):
        for k2 in range(len(FRAMED_LEAFMATES)):
            for layout in ('FLFL', 'LFLF', 'FLLF', 'LFFL'):
                parts = []
                leaf_kinds = [k1, k2]
                li = 0
                for j, ch in enumerate(layout):
                    if ch == 'F':
                        parts.append(framed_fn(j, j + 1, 'g'))
                    else:
                        parts.append(leafmate(leaf_kinds[li % 2], j))
                        li += 1
                emit('int g(int);\n' + '\n'.join(parts) + '\n')

    # 4. The neighbours whose LABEL STRIDE decides the framed function's `$M`
    #    numbers. The counter is advanced by every function in the TU whether or not
    #    it emits a label, so a neighbour with a stride the emitter models wrongly
    #    gives the framed function `$M` numbers that link and are wrong.
    #
    #    The stride is 1 for every class the port emits EXCEPT the comparison leaf,
    #    which is 1 or 3 by relation, and the floating-point leaf, which is 2 (4 or 6
    #    with pooled constants). Both lists are swept: the first must MATCH, the
    #    second must refuse. A mismatch in either is the gate having a hole, and a
    #    *refusal* in the first list is the gate over-refusing — cheaper, but it is
    #    what this axis was added to measure.
    FRAMED_STRIDE1 = [
        'int R(int x) { return x < 0; }',
        'int R(int x) { return x >= 0; }',
        'int R(int x) { return x == 0; }',
        'int R(int x) { return x != 0; }',
        'int R(int x) { return x == 5; }',
        'int R(int x) { return x != -5; }',
        'int R(int x) { return x == 32767; }',
        'int R(unsigned x) { return x < 5u; }',
        'int R(unsigned x) { return x >= 5u; }',
        'int R(unsigned x) { return x > 5u; }',
        'int R(unsigned x) { return x <= 5u; }',
    ]
    FRAMED_REFUSERS = [
        'float R(float x, float y) { return x * y; }',
        'double R(double x, double y) { return x + y; }',
        'float R(float x) { return x * 2.5f; }',
        'int R(int x, int y) { return x < y; }',
        'int R(int x, int y) { return x >= y; }',
        'int R(int x) { return x < 5; }',
        'int R(int x) { return x > 0; }',
        'int R(int x) { return x <= 0; }',
        'int R(int x, int y) { return x == y; }',
    ]
    for r in FRAMED_REFUSERS + FRAMED_STRIDE1:
        emit('int g(int);\n%s\nint F(int a) { return g(a) + 1; }\n' % r)
        emit('int g(int);\nint F(int a) { return g(a) + 1; }\n%s\n' % r)
        emit('int g(int);\nint F1(int a) { return g(a) + 1; }\n%s\n'
                 'int F2(int a) { return g(a) + 2; }\n' % r)

    # 5. THE FRAMED CALL'S ARGUMENT REGISTER — the axis every case above holds
    #    fixed. `framed_fn` is `int F(int a) { return g(a) + 1; }`: one parameter,
    #    necessarily in r3, so the argument's *index* and its *register* were the
    #    same number in all 363 framed cases and in every framed fixture. c2 emits
    #    `or r3,rN,rN` when they differ and the port emitted nothing — a live
    #    wrong-bytes emit found 2026-07-30 by compiling the neighbours rather than
    #    by any instrument (`docs/GAPS.md` §6). Two things shift the register: the
    #    argument's position among the formals, and the ABI footprint of whatever
    #    precedes it — including a leading `float`/`double`/`long long`, which take
    #    a GPR slot each on this ABI even though they are passed elsewhere.
    for nf in range(1, 6):
        ps = ', '.join('int p%d' % i for i in range(nf))
        for i in range(nf):
            emit('int g(int);\nint F(%s) { return g(p%d) + %d; }\n' % (ps, i, i + 1))
            # …and with a leaf ahead of it, so the `bl` displacement and the label
            # counter move at the same time as the argument register.
            emit('int g(int);\nint L(int a) { return a + 1; }\n'
                     'int F(%s) { return g(p%d) + %d; }\n' % (ps, i, i + 1))
    #    Past the eighth formal the argument is stack-homed (`lwz r3,180(r1)`), which
    #    the register-move model cannot express and which the constant-body emitter
    #    used to answer with no instruction at all. Refused; a MISMATCH here is that
    #    gate having a hole.
    for nf in (8, 9, 10):
        ps = ', '.join('int p%d' % i for i in range(nf))
        for i in (0, nf - 1):
            emit('int g(int);\nint F(%s) { return g(p%d) + 1; }\n' % (ps, i))
    FRAMED_ARG_LEADERS = ['float x', 'double x', 'long long x', 'int *x', 'char x',
                          'short x', 'unsigned x', 'float x, float y', 'int *x, int *y']
    for lead in FRAMED_ARG_LEADERS:
        emit('int g(int);\nint F(%s, int a) { return g(a) + 1; }\n' % lead)
        emit('int g(int);\nint F(%s, int a, int b) { return g(b) + 1; }\n' % lead)
    # Member functions: `this` is r3, so every formal is shifted by one.
    for nf in range(1, 4):
        ps = ', '.join('int p%d' % i for i in range(nf))
        for i in range(nf):
            emit('int g(int);\nstruct S { int m; int F(%s); };\n'
                     'int S::F(%s) { return g(p%d) + %d; }\n' % (ps, ps, i, i + 1))
