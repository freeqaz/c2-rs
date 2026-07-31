# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the driver; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace. A fragment cannot see
# or touch another fragment's counter, and the driver fails if a fragment emits
# zero cases.
#
# ---- WSL: a store whose VALUE is an indirect LOAD ---------------------------
#
# `80-store-leaf.py` sweeps one store's DESTINATION and `82-store-run.py` sweeps
# what a sequence adds. This fragment sweeps the axes that only appear once the
# stored value is itself read out of memory, and there are three of them that
# nothing else in the sweep can reach:
#
#   * the **scratch register**, which is the whole `/O1` / `/Ox` split for this
#     shape. `/O1` reuses r11 (f0) for every statement; `/Ox` descends r11, r10,
#     r9, … (f0, f13, f12, …) and then STOPS descending and starts skipping and
#     wrapping once it reaches a parameter's register. So run length crossed with
#     parameter count is a real axis here where it was cosmetic before, and every
#     case is compiled in both modes;
#   * the two register files counted INDEPENDENTLY inside one run — a `lfs f0`
#     between two `lwz r11`/`r10` pairs, which no pure-file run can produce;
#   * the value's designator, which is a second full designator in a position
#     that never had one. Its offset run, its `2C` cv strip, its widths and its
#     intrinsic-2117 spelling are all crossed against the DESTINATION's, so a
#     rule that reads one of the two where it meant the other shows up.
#
# It also carries the axis that found this rung's own boundary: cv-qualification
# on the SOURCE, which changes no operator and no shape and is the difference
# between a copy assignment parsing and not parsing.


def cases(emit):
    S = ('struct S { int a; int b; int c; int d; int e; int f; int g; int h; '
         'int i2; char c8; short s16; long long q64; unsigned u32; int* p; '
         'float x; double y; };')
    GPR = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i2']

    # 1. **Run length crossed with parameter count**, in both modes. This is the
    #    axis the `/Ox` scratch descent lives on: with n parameters the descent
    #    r11, r10, … reaches r(2+n) at statement 9-n, and that is where c2 stops
    #    being a plain descent. Sweeping one past the bound on every parameter
    #    count is the only way a fitted allocator shows up as wrong bytes.
    for n in range(1, 10):
        for extra in range(0, 5):
            pre = ''.join(', S* x%d' % j for j in range(extra))
            body = ' '.join('d->%s = s->%s;' % (GPR[j], GPR[j]) for j in range(n))
            emit('%s\nvoid f(S* d, S* s%s) { %s }\n' % (S, pre, body))

    # 2. The same, in the FLOATING-POINT file, whose descent is f0 and then f13
    #    downwards — a different sequence with a different first element, so a
    #    rule that reused the GPR one would be wrong at statement 2 already.
    FPM = ['x', 'y']
    F = 'struct F { float a[8]; double b[8]; };'
    for n in range(1, 9):
        body = ' '.join('d->a[%d] = s->a[%d];' % (j, j) for j in range(n))
        emit('%s\nvoid f(F* d, F* s) { %s }\n' % (F, body))

    # 3. **The two files inside one run**, at every interleaving of a 3-run. The
    #    counters are independent, so `int,float,int` must be r11/f0/r10 and not
    #    r11/f0/f13 or r11/r10/f0 — three distinct wrong answers a single counter
    #    would give.
    MIX = ['a', 'x', 'y', 'c8', 'q64']
    import itertools as _it
    for combo in _it.product(MIX, repeat=3):
        body = ' '.join('d->%s = s->%s;' % (m, m) for m in combo)
        emit('%s\nvoid f(S* d, S* s) { %s }\n' % (S, body))

    # 4. **cv-qualification on the SOURCE**, per statement independently and at
    #    every width. A `const`/`volatile` pointee puts a `2C` between the load
    #    and the store that emits nothing — and requiring the two types to be
    #    byte-identical (which is the rule the formal-valued path draws) refuses
    #    every copy assignment in the corpus. The qualifier changes no operator
    #    and no shape, so nothing else varies it.
    WIDTHS = ['a', 'c8', 's16', 'q64', 'u32', 'p', 'x', 'y']
    for q in ('', 'const ', 'volatile ', 'const volatile '):
        for m in WIDTHS:
            emit('%s\nvoid f(S* d, %sS* s) { d->%s = s->%s; }\n' % (S, q, m, m))
        # …and mixed within a run, which is where a per-statement rule and a
        # per-body rule part company.
        emit('%s\nvoid f(S* d, %sS* s) { d->a = s->a; d->c8 = s->c8; d->y = s->y; }\n'
             % (S, q))

    # 5. The SOURCE designator crossed with the DESTINATION's: plain member,
    #    nested member, subscript, bare deref, and the intrinsic-2117 inherited
    #    member — all four spellings on each side, so a rule that read one
    #    designator where it meant the other cannot hide.
    N = ('struct In { int x; int y; };\nstruct N { int m0; In in; int arr[4]; };')
    DES = ['%s->m0', '%s->in.y', '%s->arr[2]']
    for dst in DES:
        for src in DES:
            emit('%s\nvoid f(N* d, N* s) { %s = %s; }\n'
                 % (N, dst % 'd', src % 's'))
    BM = ('struct A { int a0; int a1; };\nstruct B { int b0; int b1; };\n'
          'struct D : A, B { int d0; };')
    for dst in ('d->d0', 'd->b1', 'd->a0'):
        for src in ('s->d0', 's->b1', 's->a0'):
            emit('%s\nvoid f(D* d, D* s) { %s = %s; }\n' % (BM, dst, src))
    emit('void f(int* d, int* s) { *d = *s; }\n')
    emit('void f(int* d, int* s) { d[3] = s[5]; }\n')
    emit('void f(int* d, int* s) { *d = s[5]; }\n')
    emit('void f(int* d, int* s) { d[3] = *s; }\n')

    # 6. **The base SLOT crossed with the run length**, for both bases
    #    independently. `GAPS.md` §6 instance #8's axis: with one parameter an
    #    index and a register are the same number, and here there are two
    #    designators whose registers move separately.
    for dslot in range(0, 4):
        for sslot in range(0, 4):
            if dslot == sslot:
                continue
            slots = max(dslot, sslot) + 1
            args = ', '.join(('S* d' if j == dslot else 'S* s' if j == sslot
                              else 'int k%d' % j) for j in range(slots))
            for n in (1, 2, 3):
                body = ' '.join('d->%s = s->%s;' % (GPR[j], GPR[j]) for j in range(n))
                emit('%s\nvoid f(%s) { %s }\n' % (S, args, body))

    # 7. **The same object on both sides**, which is safe for ONE statement and
    #    is dead-store-eliminated for more than one — two opposite lowerings
    #    behind one shape.
    emit('%s\nvoid f(S* d) { d->a = d->b; }\n' % S)
    emit('%s\nvoid f(S* d) { d->a = d->b; d->b = d->a; }\n' % S)
    emit('%s\nvoid f(S* d) { d->a = d->b; d->c = d->d; }\n' % S)
    emit('%s\nvoid f(S* d, S* s) { d->a = s->a; s->b = d->b; }\n' % S)

    # 8. Values MIXED between the three kinds — loaded, formal, literal — at
    #    every position of a 3-run. c2 schedules as soon as the kinds are mixed
    #    (the load is hoisted and its store sinks), and which orders happen to
    #    survive is not a rule, so every one of them is swept.
    KIND = ['s->%s', 'v', '7']
    for combo in _it.product(range(3), repeat=3):
        body = ' '.join('d->%s = %s;' % (
            GPR[j], (KIND[combo[j]] % GPR[j]) if combo[j] == 0 else KIND[combo[j]])
            for j in range(3))
        emit('%s\nvoid f(S* d, S* s, int v) { %s }\n' % (S, body))

    # 9. Conversions on the loaded value — the widenings that cost an `extsb`,
    #    the narrowings that do not, and the cross-file ones. All refused, and
    #    the point of sweeping them is that the refusal has to hold at every
    #    (source, target) pair rather than at the one that was captured.
    CONV = ['a', 'c8', 's16', 'q64', 'u32', 'x', 'y']
    for src in CONV:
        for dst in CONV:
            if src == dst:
                continue
            emit('%s\nvoid f(S* d, S* s) { d->%s = s->%s; }\n' % (S, dst, src))

    # 10. Source LINES and brace scopes between the statements, including a run
    #     at source line 70 — instance #1's axis, and the statement-boundary walk
    #     is shared with the run production rather than copied.
    for sep in ('\n', '\n\n\n', '\n', ''):
        body = sep.join(['    d->a = s->a;', '    d->b = s->b;', '    d->c = s->c;'])
        emit('%s\nvoid f(S* d, S* s) {\n%s\n}\n' % (S, body))
    emit('%s\nvoid f(S* d, S* s) { { d->a = s->a; } { d->b = s->b; } }\n' % S)
    emit('%s\nvoid f(S* d, S* s) { { d->a = s->a; d->b = s->b; } }\n' % S)
    emit('%s\n%s\nvoid f(S* d, S* s) { d->a = s->a; d->b = s->b; }\n'
         % (S, '\n' * 60))

    # 11. **The TAIL crossed with the run length**, which is where this rung
    #     meets W38's: a copy assignment is a load-valued run with a
    #     `return *this`, and a copy constructor is one with a constructor's
    #     implicit result sitting after the `29`. Each special member is defined
    #     OUT OF LINE so it is emitted without a call site — a forcing helper
    #     would have to make a member call, which is a different rung's shape.
    DECL = 'int a0; int a1; int a2;'
    for n in (1, 2, 3):
        body = ' '.join('a%d = r.a%d;' % (j, j) for j in range(n))
        for ret, sig in (('void', 'void T::set(const T& r)'),
                         ('T&', 'T& T::asn(const T& r)'),
                         ('T*', 'T* T::pas(const T& r)')):
            tail = {'void': '', 'T&': ' return *this;', 'T*': ' return this;'}[ret]
            emit('struct T { %s\n void set(const T&); T& asn(const T&);'
                 ' T* pas(const T&); };\n%s { %s%s }\n'
                 % (DECL, sig, body, tail))
        emit('struct T { %s\n T(const T&); };\nT::T(const T& r) { %s }\n'
             % (DECL, body))

    # 12. A load-valued run beside every OTHER thing a TU can carry, in both
    #     orders — the `_fltused` marker and the per-TU label counter are decided
    #     by the whole translation unit, and an FP *load* inside a run is a new
    #     way to make a TU FP-touching that no previous family produces.
    RUNS = [
        '%s\nvoid R(S* d, S* s) { d->a = s->a; d->b = s->b; }\n' % S,
        '%s\nvoid RF(S* d, S* s) { d->x = s->x; d->y = s->y; }\n' % S,
        '%s\nvoid RM(S* d, S* s) { d->a = s->a; d->x = s->x; }\n' % S,
    ]
    OTHERS = [
        'int A(int x) { return x + 1; }\n',
        'float C(float x, float y) { return x * y; }\n',
        'void g(int);\nvoid B(int x) { g(x); }\n',
        'int h(int);\nint D(int x) { return h(x) + 1; }\n',
        'int E(int x) { return x > 3; }\n',
    ]
    for run in RUNS:
        for other in OTHERS:
            emit(run + other)
            emit(other + run)
