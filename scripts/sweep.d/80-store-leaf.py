# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the driver; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace. A fragment cannot see
# or touch another fragment's counter (the `n`-shadowing trap that silently
# overwrote 1,233 already-written cases is unrepresentable here), and the driver
# fails if a fragment emits zero cases.


def cases(emit):
    # ---- W25: the STORE leaf ---------------------------------------------------
    # `s->m = v;` is one `stb`/`sth`/`stw`/`std` at a folded displacement, and the
    # axes that pick it are (stored width) x (displacement) x (which designator) x
    # (which two registers). The hand-written fixture crosses them once each; this
    # crosses them against each other, which is the only thing that separates
    # "the width comes from the stored TYPE" from "the width comes from the
    # designator's pointer tag" — the two agree on every `int` member.
    STORE_MEMBERS = [
        ('int', 'mi'), ('unsigned', 'mu'), ('char', 'mc'), ('signed char', 'msc'),
        ('unsigned char', 'muc'), ('bool', 'mb'), ('short', 'msh'),
        ('unsigned short', 'mush'), ('long long', 'mll'),
        ('unsigned long long', 'mull'), ('void*', 'mp'),
    ]
    STORE_STRUCT = ('struct S { int pad0; ' +
                    ' '.join('%s %s;' % (t, n) for t, n in STORE_MEMBERS) +
                    ' int arr[4]; };')

    # 1. every stored width, at both a zero-ish and a later offset, through a plain
    #    designator — a member and its `arr[k]` neighbour.
    for t, nm in STORE_MEMBERS:
        emit('%s\nvoid f(S* s, %s v) { s->%s = v; }\n' % (STORE_STRUCT, t, nm))
        emit('%s\nvoid f(S* s) { s->%s = (%s)3; }\n' % (STORE_STRUCT, nm, t)
                 if t != 'void*' else
                 '%s\nvoid f(S* s) { s->mp = 0; }\n' % STORE_STRUCT)

    # 2. the register cross product: the base at slots 0..7 and the value right
    #    after it. `stw r5,4(r4)` moves BOTH fields at once.
    for slot in range(8):
        pre = ''.join('int p%d, ' % j for j in range(slot))
        emit('%s\nvoid f(%sS* s, int v) { s->mi = v; }\n' % (STORE_STRUCT, pre))
        emit('%s\nvoid f(%sS* s, char v) { s->mc = v; }\n' % (STORE_STRUCT, pre))

    # 3. literal values, across the `li` / `lis`+`ori` boundary and both signs.
    for k in ('0', '1', '7', '-1', '-3', '32767', '-32768', '32768', '70000', '65535', '-70000'):
        emit('%s\nvoid f(S* s) { s->mi = %s; }\n' % (STORE_STRUCT, k))
        emit('%s\nvoid f(S* s) { s->mc = (char)%s; }\n' % (STORE_STRUCT, k))

    # 4. the subscript add run, cv-qualified bases, the bare deref and the cast.
    for body in ('s->arr[0] = v', 's->arr[1] = v', 's->arr[3] = v', '*(int*)s = v'):
        emit('%s\nvoid f(S* s, int v) { %s; }\n' % (STORE_STRUCT, body))
    emit('%s\nvoid f(int* p, int v) { *p = v; }\n' % STORE_STRUCT)
    emit('%s\nvoid f(int* p, int v) { p[2] = v; }\n' % STORE_STRUCT)
    emit('%s\nvoid f(char* p, char v) { p[3] = v; }\n' % STORE_STRUCT)
    emit('%s\nvoid f(long long* p, long long v) { p[1] = v; }\n' % STORE_STRUCT)

    # 5. the intrinsic-2117 designator: an inherited member, one and two inheritance
    #    steps, every width, and a member function where `this` is in r3.
    BASE_DECL = ('struct A { int a0; int a1; };\n'
                 'struct B { int b0; char bc; short bs; long long bl; };\n'
                 'struct D : A, B { int d; };\n'
                 'struct E : D { int e; };\n')
    for mem, t in (('b0', 'int'), ('bc', 'char'), ('bs', 'short'), ('bl', 'long long')):
        emit('%svoid f(D* p, %s v) { p->%s = v; }\n' % (BASE_DECL, t, mem))
        emit('%svoid f(E* p, %s v) { p->%s = v; }\n' % (BASE_DECL, t, mem))
        emit('%svoid f(int x, D* p, %s v) { p->%s = v; }\n' % (BASE_DECL, t, mem))
        emit('%sstruct M : A, B { void s(%s v); };\nvoid M::s(%s v) { %s = v; }\n'
                 % (BASE_DECL, t, t, mem))
    emit('%svoid f(D* p) { p->b0 = 9; }\n' % BASE_DECL)
    emit('%svoid f(D* p) { p->bc = (char)9; }\n' % BASE_DECL)

    # 6. a store beside each accepted neighbour, so the three consumers of the one
    #    designator cannot start swallowing each other.
    for mate in ('int g(S* s) { return s->mi; }',
                 'int* g(S* s) { return &s->mi; }',
                 'S* g(S* s) { return s; }',
                 'char g(S* s) { return s->mc; }',
                 'int g(int a, int b) { return a + b; }'):
        emit('%s\n%s\nvoid f(S* s, int v) { s->mi = v; }\n' % (STORE_STRUCT, mate))
        emit('%s\nvoid f(S* s, int v) { s->mi = v; }\n%s\n' % (STORE_STRUCT, mate))

    # 7. the REFUSING neighbours. Every one emits something the store production
    #    does not; a MISMATCH here is the gate having a hole, not a gap.
    STORE_REFUSERS = [
        'struct F { float f; double d; };\nvoid f(F* s, float v) { s->f = v; }',
        'struct F { float f; double d; };\nvoid f(F* s, double v) { s->d = v; }',
        'struct F { float f; double d; };\nvoid f(F* s) { s->f = 1.5f; }',
        'struct I { int i; char c; };\nvoid f(I* s, bool v) { s->i = v; }',
        'struct I { int i; char c; };\nvoid f(I* s, int v) { s->c = (char)v; }',
        'struct I { int i; char c; };\nvoid f(I* s, char v) { s->i = v; }',
        'struct I { int i; char c; };\nvoid f(I* s, int x, int y) { s->i = x + y; }',
        'struct I { int i; char c; };\nvoid f(I* s, int x) { s->i = x * 3; }',
        'struct I { int i; int j; };\nvoid f(I* s, int v) { s->i = v; s->j = v; }',
        'struct I { int i; int j; };\nint f(I* s, int v) { return s->i = v; }',
        'struct I { int i; int a[4]; };\nvoid f(I* s, int k, int v) { s->a[k] = v; }',
        'struct I { int i; int j; };\nvoid f(I* d, I* s) { d->i = s->i; }',
        'struct I { int i; int j; };\nvoid f(I* d, I s) { *d = s; }',
        'int gv;\nvoid f(int v) { gv = v; }',
    ]
    for r in STORE_REFUSERS:
        emit('%s\n' % r)
        emit('%s\nint h(int a) { return a + 1; }\n' % r)

    # 8. W28: the FLOATING-POINT store leaf, `stfs`/`stfd` out of the FP argument
    #    file. Two things are being swept at once and they need a cross product:
    #    the value's FP register (which counts FP parameters alone) and the base
    #    pointer's GPR (which counts SLOTS, so an FP parameter advances it while
    #    filling no register). Each rule alone gets half the cases right.
    FPS_STRUCT = ('struct FS { int i; float f; double d; float a[4]; char c; '
                  'float g; double h; };\n')
    for vty, mem in (('float', 'f'), ('double', 'd'), ('float', 'g'),
                     ('double', 'h'), ('float', 'a[2]')):
        # the value at every FP-file position, behind every mix of leading formals
        for lead in ('', 'int k, ', 'int k, int l, ', 'float u, ', 'double u, ',
                     'float u, int k, ', 'int k, float u, ', 'char c, ', 'int* p, '):
            emit('%svoid f(FS* s, %s%s v) { s->%s = v; }\n'
                     % (FPS_STRUCT, lead, vty, mem))
        # …and with the struct pointer NOT first, which moves the base GPR
        emit('%svoid f(int k, FS* s, %s v) { s->%s = v; }\n'
                 % (FPS_STRUCT, vty, mem))
        emit('%svoid f(%s v, FS* s) { s->%s = v; }\n' % (FPS_STRUCT, vty, mem))
        emit('%svoid f(float u, FS* s, %s v) { s->%s = v; }\n'
                 % (FPS_STRUCT, vty, mem))
        emit('%svoid f(int j, float u, FS* s, %s v) { s->%s = v; }\n'
                 % (FPS_STRUCT, vty, mem))
    # Through a bare pointer and through an inherited base member (intrinsic 2117).
    for pty in ('float', 'double'):
        emit('void f(%s* p, %s v) { *p = v; }\n' % (pty, pty))
        emit('void f(%s* p, %s v) { p[3] = v; }\n' % (pty, pty))
        emit('void f(int k, %s* p, %s v) { *p = v; }\n' % (pty, pty))
        emit('struct BB { %s b; };\nstruct DD : BB { int d; };\n'
                 'void f(DD* p, %s v) { p->b = v; }\n' % (pty, pty))
    # Member functions: `this` takes r3, so the base is implicit and the FP file
    # must be unaffected by it.
    for vty in ('float', 'double'):
        emit('struct MM { %s m; void s(%s v); };\nvoid MM::s(%s v) { m = v; }\n'
                 % (vty, vty, vty))
        emit('struct MM { %s m; void s(int k, %s v); };\n'
                 'void MM::s(int k, %s v) { m = v; }\n' % (vty, vty, vty))
        emit('struct MM { %s m; void s(%s u, %s v); };\n'
                 'void MM::s(%s u, %s v) { m = v; }\n' % (vty, vty, vty, vty, vty))
    # The REFUSERS on the FP path: a conversion in either direction (the narrowing
    # one is a real `frsp` through f0, the widening one is free and is refused
    # anyway), a pooled literal, and a computed value.
    FP_STORE_REFUSERS = [
        'struct FS { float f; double d; };\nvoid f(FS* s, double v) { s->f = v; }',
        'struct FS { float f; double d; };\nvoid f(FS* s, float v) { s->d = v; }',
        'struct FS { float f; double d; };\nvoid f(FS* s, int v) { s->f = (float)v; }',
        'struct FS { int i; float f; };\nvoid f(FS* s, float v) { s->i = (int)v; }',
        'struct FS { float f; double d; };\nvoid f(FS* s) { s->f = 2.25f; }',
        'struct FS { float f; double d; };\nvoid f(FS* s) { s->d = 2.25; }',
        'struct FS { float f; double d; };\nvoid f(FS* s, float u, float v) { s->f = u + v; }',
        'struct FS { float f; double d; };\nvoid f(FS* s, float v) { s->f = -v; }',
        'struct FS { float f; double d; };\nvoid f(FS* s, float v) { s->f = v; s->d = v; }',
    ]
    for r in FP_STORE_REFUSERS:
        emit('%s\n' % r)
        emit('%s\nint h(int a) { return a + 1; }\n' % r)
    # `_fltused` placement: a TU that touches FP carries the marker after the FIRST
    # FP-touching function's symbol group, and an FP STORE counts as touching. The
    # port tied that to "is a W13 arithmetic leaf" and emitted every FP-store obj one
    # symbol short. Only a MIXED translation unit separates the two rules, so the
    # ordering is swept rather than assumed.
    FPS_MIX = ('struct OS { int i; float f; };\n',
               'int A(int x) { return x + 1; }\n',
               'void B(OS* s, float v) { s->f = v; }\n',
               'float C(float x, float y) { return x * y; }\n',
               'int D(int x) { return x + 2; }\n',
               'void E(OS* s, double v) { s->i = (int)v; }\n')
    import itertools as _it
    for order in _it.permutations('ABCD'):
        body = FPS_MIX[0] + ''.join(FPS_MIX['ABCDE'.index(c) + 1] for c in order)
        emit(body)
