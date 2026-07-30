# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the driver; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace. A fragment cannot see
# or touch another fragment's counter (the `n`-shadowing trap that silently
# overwrote 1,233 already-written cases is unrepresentable here), and the driver
# fails if a fragment emits zero cases.


def cases(emit):
    # ---- W20: the `2C` CONVERSION in a general expression operand position -------
    #
    # `docs/IL_CALL_IN_EXPR.md` §24. A conversion whose target is the value's own
    # 4-byte class emits NOTHING, so admitting it must leave the emitted bytes of the
    # surrounding chain exactly as they were. The axis that matters — and the one no
    # leaf shape could reach — is WHERE the conversion sits relative to the operands
    # and the operator, because that is the layer the reassociation mis-emits lived
    # in. Every combination below is generated rather than picked.
    INT4 = ['int', 'unsigned', 'long', 'unsigned long']
    # 1. the whole spelling matrix, source x target, as a one-operand body.
    for src_t in INT4:
        for dst_t in INT4:
            emit("%s f(%s a) { return (%s)a; }\n" % (dst_t, src_t, dst_t))
    # 2. the conversion at each position of a two-operand chain, over every operator
    #    and both operand orders (a literal on either side included).
    for op in ['+', '-', '*']:
        for l, r in [('a', 'b'), ('b', 'a'), ('a', '3'), ('3', 'a'), ('a', 'a')]:
            emit("unsigned f(int a, int b) { return (unsigned)%s %s %s; }\n" % (l, op, r))
            emit("unsigned f(int a, int b) { return %s %s (unsigned)%s; }\n" % (l, op, r))
            emit("unsigned f(int a, int b) { return (unsigned)(%s %s %s); }\n" % (l, op, r))
    # 3. three-operand chains with the conversion at each of the four slots.
    for op1 in ['+', '-', '*']:
        for op2 in ['+', '-', '*']:
            emit("unsigned f(int a, int b, int c) { return (unsigned)a %s b %s c; }\n" % (op1, op2))
            emit("unsigned f(int a, int b, int c) { return a %s (unsigned)b %s c; }\n" % (op1, op2))
            emit("unsigned f(int a, int b, int c) { return a %s b %s (unsigned)c; }\n" % (op1, op2))
            emit("unsigned f(int a, int b, int c) { return (unsigned)(a %s b) %s c; }\n" % (op1, op2))
            emit("unsigned f(int a, int b, int c) { return (unsigned)(a %s b %s c); }\n" % (op1, op2))
    # 4. nested conversions: the round trip, and back again.
    for depth in ['(unsigned)a', '(unsigned)(int)a', '(int)(unsigned)a', '(unsigned)(int)(unsigned)a']:
        emit("unsigned f(int a) { return %s; }\n" % depth)
        emit("unsigned f(int a, int b) { return %s + b; }\n" % depth)
    # 5. the converted formal at every argument slot, at every arity — the D10
    #    register move underneath the conversion.
    for k in range(1, 9):
        params = ', '.join('int a%d' % j for j in range(k))
        emit("unsigned f(%s) { return (unsigned)a%d; }\n" % (params, k - 1))
    # 6. the conversion inside a CALL-ARGUMENT region, which is `parse_expr`'s other
    #    caller and where the workload's `calls-1` half lives.
    emit("int g1(int);\nint f(unsigned a) { return g1((int)a); }\n")
    emit("int g1(int);\nint f(int a, int b) { return g1((int)(a + b)); }\n")
    emit("int g1(int);\nint f(int a, int b) { return g1((int)a + b); }\n")
    emit("int g2(int, int);\nint f(unsigned a, unsigned b) { return g2((int)a, (int)b); }\n")
    emit("int g2(int, int);\nint f(unsigned a, int b) { return g2((int)a, b); }\n")
    emit("int g2(int, int);\nint f(int a, unsigned b) { return g2(a, (int)b); }\n")
    emit("int g3(int, int, int);\nint f(unsigned a, int b, int c) { return g3((int)a, b, c); }\n")
    emit("int g3(int, int, int);\nint f(int a, unsigned b, int c) { return g3(a, (int)b, c); }\n")
    emit("int g3(int, int, int);\nint f(int a, int b, unsigned c) { return g3(a, b, (int)c); }\n")
    # 7. the POINTER half: every pointee width against every target pointer spelling,
    #    as a tail-call argument (the workload shape) and as a whole body.
    PTEE = ['char', 'short', 'int', 'double', 'void', 'S']
    PDST = ['void *', 'const void *', 'const S *', 'S *']
    for ptee in PTEE:
        emit("struct S { int m; };\nint gv(void *);\n"
                 "int f(%s *p) { return gv((void *)p); }\n" % ptee)
        emit("struct S { int m; };\n"
                 "void *f(%s *p) { return (void *)p; }\n" % ptee)
    for dst_t in PDST:
        emit("struct S { int m; };\nint gq(%s);\n"
                 "int f(S *p) { return gq((%s)p); }\n" % (dst_t, dst_t))
        emit("struct S { int m; };\n%s f(S *p) { return (%s)p; }\n" % (dst_t, dst_t))
    # a pointer conversion at each argument slot of a multi-argument tail call
    for k in range(1, 4):
        for j in range(k):
            args = ', '.join('void *' if i == j else 'int' for i in range(k))
            params = ', '.join(('S *p%d' % i) if i == j else ('int a%d' % i) for i in range(k))
            actual = ', '.join(('(void *)p%d' % i) if i == j else ('a%d' % i) for i in range(k))
            emit("struct S { int m; };\nint gp%d(%s);\nint f(%s) { return gp%d(%s); }\n"
                     % (k, args, params, k, actual))
    # 8. a member function, where `this` is a const pointer in r3
    emit("struct C { int m; unsigned u(int a) const; };\n"
             "unsigned C::u(int a) const { return (unsigned)a; }\n")
    emit("struct C { int m; unsigned u(int a, int b) const; };\n"
             "unsigned C::u(int a, int b) const { return (unsigned)b; }\n")
    emit("struct C { int m; int c() const; };\nint gv(void *);\n"
             "int C::c() const { return gv((void *)this); }\n")
    # 9. the REFUSING neighbours. Every one of these emits an instruction the modeled
    #    chain cannot produce, or is a reinterpret that has never been graded — a
    #    MISMATCH here is the alarm this block exists to raise.
    for dst_t in ['char', 'signed char', 'unsigned char', 'short', 'unsigned short',
                  'long long', 'unsigned long long', 'float', 'double', 'bool']:
        emit("%s f(int a) { return (%s)a; }\n" % (dst_t, dst_t))
        emit("%s f(int a, int b) { return (%s)(a + b); }\n" % (dst_t, dst_t))
        emit("int g1(int);\n%s f(int a) { return (%s)g1(a); }\n" % (dst_t, dst_t))
    for src_t in ['char', 'short', 'long long', 'float', 'double']:
        emit("int f(%s a) { return (int)a; }\n" % src_t)
        emit("unsigned f(%s a) { return (unsigned)a; }\n" % src_t)
    # the cross-class reinterpret, both directions, body and argument
    emit("struct S { int m; };\nint f(S *p) { return (int)p; }\n")
    emit("struct S { int m; };\nunsigned f(S *p) { return (unsigned)p; }\n")
    emit("struct S { int m; };\nS *f(int a) { return (S *)a; }\n")
    emit("struct S { int m; };\nS *f(unsigned a) { return (S *)a; }\n")
    emit("struct S { int m; };\nint g1(int);\nint f(S *p) { return g1((int)p); }\n")
    emit("struct S { int m; };\nint f(S *p, int k) { return (int)p + k; }\n")
    # a conversion whose value then does pointer arithmetic — the §21 guard, reached
    # through a conversion rather than straight off a LOAD
    emit("struct S { int m; };\nvoid *f(S *p) { return (void *)(p + 1); }\n")
    emit("struct S { int m; };\nint f(S *p, S *q) { return (int)(p - q); }\n")
    emit("struct S { int m; };\nvoid *f(S *p, int k) { return (void *)(p + k); }\n")
    # a cv-qualified operand TYPE, which blocks at the operand and never reaches the
    # conversion — a different key, kept here so a regression cannot merge the two
    emit("int f(const int a) { return (int)a; }\n")
    emit("unsigned f(const int a, int b) { return (unsigned)(a + b); }\n")
