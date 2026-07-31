# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the loader; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace. A fragment cannot see
# or touch another fragment's counter, and the loader fails if a fragment emits
# zero cases.


def cases(emit):
    # ---- Class A many-calls: the call SEQUENCE, in a TU of its own -------------------
    # The three accepted `call-sequence*` families — void tail, value tail, literal
    # tail — had **no single-function case anywhere in this corpus** until this
    # fragment. `scripts/cross_sweep.py` found that by asking the port for its own
    # list of accepted shape families and looking for a case of each: 81-fp-beside-
    # framed is the only fragment that reaches this class and every one of its TUs
    # carries a second function, so the class was only ever graded *beside* something.
    #
    # That is not a cosmetic hole. This is the class that made `docs/GAPS.md` §6 #12
    # reachable at all — the framed shape that had to share a TU with an FP store
    # before the label-counter bug could exist — and #13 then showed its repair was
    # wrong one row further out, because a per-function method was holding a per-TU
    # quantity and `n = 1` cannot separate the two formulations. So the axes here are
    # the ones that move that counter and the frame it sits in: how many calls, what
    # each call passes, what the body does after the last one, and whether a formal
    # has to survive a `bl` (Class B, which takes callee-saved GPRs and is refused).
    DECLS = ('void q0();\nvoid q1(int);\nvoid q2(int,int);\n'
             'int g0();\nint g1(int);\nint g2(int,int);\n')

    # 1. The tail forms, at each call count. `void` tail, value tail (the last call's
    #    result IS the result) and literal tail (`return 5;` after the last `bl`) are
    #    three census families and three different post-call spines.
    for ncalls in (2, 3, 4):
        voids = ' '.join('q0();' for _ in range(ncalls))
        emit(DECLS + "void F() { %s }\n" % voids)
        emit(DECLS + "int F() { %s return 5; }\n" % voids)
        emit(DECLS + "int F() { %s return 0; }\n" % voids)
        emit(DECLS + "int F() { %s return -1; }\n" % voids)
        emit(DECLS + "int F() { %s return 32767; }\n" % voids)
        # value tail: n-1 statement calls then `return g0();`
        lead = ' '.join('q0();' for _ in range(ncalls - 1))
        emit(DECLS + "int F() { %s return g0(); }\n" % lead)
        # …and a value tail whose earlier calls also return values that are dropped
        emit(DECLS + "int F() { %s return g0(); }\n"
             % ' '.join('g0();' for _ in range(ncalls - 1)))

    # 2. The ARGUMENT axis, per call. A call that passes nothing, a literal, the same
    #    formal twice, or two formals in either order sets up different registers
    #    before each `bl` — and a formal read by the LAST call has to survive the
    #    first, which is the Class A / Class B boundary.
    ARGS = ('', 'a', 'b', '1', '-1', 'a, b', 'b, a', 'a, a')
    for first in ARGS:
        for second in ARGS:
            def call(args):
                if args == '':
                    return 'q0();'
                return 'q%d(%s);' % (len(args.split(',')), args)
            emit(DECLS + "void F(int a, int b) { %s %s }\n"
                 % (call(first), call(second)))
            emit(DECLS + "int F(int a, int b) { %s %s return 7; }\n"
                 % (call(first), call(second)))

    # 3. The value tail with a computed result and with an argument — `return g1(a);`
    #    after a call is the shape where `a` is live across the first `bl`.
    for lead in ('q0();', 'q1(a);', 'q1(b);', 'q2(a, b);', 'g0();', 'g1(a);'):
        emit(DECLS + "int F(int a, int b) { %s return g1(a); }\n" % lead)
        emit(DECLS + "int F(int a, int b) { %s return g1(b); }\n" % lead)
        emit(DECLS + "int F(int a, int b) { %s return g0(); }\n" % lead)
        emit(DECLS + "int F(int a, int b) { %s return g0() + 1; }\n" % lead)
        emit(DECLS + "int F(int a, int b) { %s return a; }\n" % lead)

    # 4. The LABEL-COUNTER axis this class shares with the framed call: a sequence
    #    body owns `$M`/`$T` labels, so a neighbour whose stride the port models
    #    wrongly gives it label numbers that link and are wrong (§6 #12). Swept at
    #    both orders and with the neighbour on both sides, against leaves of every
    #    stride the port emits — including the floating-point ones, which are the
    #    pair that produced the bug.
    NEIGHBOURS = (
        'int L(int a) { return a + 1; }',            # stride 1
        'int L(int a, int b) { return b; }',         # stride 1, register move
        'int L(int *p) { return *p; }',              # stride 1, indirect load
        'struct LS { int i; float f; double d; };\nvoid L(LS *s, int v) { s->i = v; }',
        'struct LS { int i; float f; double d; };\nvoid L(LS *s, float v) { s->f = v; }',
        'struct LS { int i; float f; double d; };\nvoid L(LS *s, double v) { s->d = v; }',
        'float L(float a, float b) { return a * b; }',
        'double L(double a, double b) { return a + b; }',
        'float L(float a, float b) { return b; }',
        'int L(int x) { return x < 0; }',            # comparison leaf
        'void L() {}',                               # empty body
    )
    SEQS = ('void F() { q0(); q0(); }',
            'int F() { q0(); q0(); return 5; }',
            'int F() { q0(); return g0(); }')
    for n in NEIGHBOURS:
        for s in SEQS:
            emit(DECLS + n + '\n' + s + '\n')
            emit(DECLS + s + '\n' + n + '\n')
            # …and with a stride-1 integer leaf between them, so an error in the
            # counter cannot be absorbed by an adjacent one.
            emit(DECLS + n + '\nint M(int a) { return a + 2; }\n' + s + '\n')

    # 5. TWO sequence bodies in one TU. `label_slots` was a per-function method
    #    carrying a per-TU quantity, and at one such function the wrong rule and the
    #    right one are indistinguishable (§6 #13) — only n >= 2 separates them.
    for a in SEQS:
        for b in SEQS:
            emit(DECLS + a.replace(' F(', ' F1(') + '\n'
                 + b.replace(' F(', ' F2(') + '\n')
    for a in SEQS:
        emit(DECLS + '\n'.join(a.replace(' F(', ' F%d(' % i) for i in range(3)) + '\n')

    # 6. The refusing neighbours. Class B (a formal read after a call must survive it
    #    in a callee-saved GPR), a call through a pointer, a call whose result feeds
    #    another call, and a body with a real statement between the calls. Each costs
    #    instructions the accepted spine does not emit, so a MISMATCH here is the
    #    alarm and NotImplemented is the right answer.
    for src in (
        "void F(int a, int b) { q1(a); q1(b); }",
        "void F(int a) { q1(a); q1(a); }",
        "int F(int a) { q1(a); return a; }",
        "int F(int a) { q1(a); return a + 1; }",
        "int F(int a, int b) { q1(a); return g1(b); }",
        "int F() { return g1(g0()); }",
        "int F() { q0(); return g0() + g0(); }",
        "void F(void (*p)()) { p(); p(); }",
        "int F(int a) { int x = g1(a); q0(); return x; }",
        "void F(int a) { q0(); if (a) q0(); }",
        "int F(int a) { q0(); q0(); return a ? 1 : 2; }",
        "void F(int *p) { q1(*p); q1(*p); }",
        "int F() { q0(); q0(); return g0(); }",
    ):
        emit(DECLS + src + '\n')
