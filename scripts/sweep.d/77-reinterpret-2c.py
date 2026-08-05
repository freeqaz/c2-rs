# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the driver; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace. A fragment cannot see
# or touch another fragment's counter, and the driver fails if a fragment emits
# zero cases.


def cases(emit):
    # ---- w-convert / board #700: the `2C` WIDTH-4 REINTERPRET -----------------
    #
    # `61-conversion-2c.py` crosses the CLASS-PRESERVING convert and nothing else;
    # its handful of cross-class lines were written when the parser refused them,
    # so they graded a refusal. This fragment is the enumeration of the cells the
    # rung actually admits, plus the ones it must keep refusing.
    #
    # **What #688 is the precedent for.** That lane's fragment first reported 0
    # mismatches against the very binary it was written to catch, because
    # unbracketed C++ precedence reassociated every case away from the shape it
    # named. So the shapes here are written with their brackets explicit, and the
    # two blocks that matter most — the SCALED pointer arithmetic reached through
    # a conversion, and `bool` on either side — are enumerated rather than
    # sampled.
    HDR = "struct S { int m; int n; };\ntypedef void (*FnPtr)();\ntypedef int myint;\nenum E { E0, E1 };\n"
    INT4 = ['int', 'unsigned', 'long', 'unsigned long', 'myint']
    PTR = ['void *', 'int *', 'char *', 'const char *', 'S *', 'S **', 'FnPtr',
           'const S *', 'volatile S *']

    # 1. THE MATRIX, whole: every integer spelling against every pointer spelling,
    #    in BOTH directions, as a one-operand body. The signed sources are the
    #    cells that could have carried an `extsw`/`rldicl` at the 32->64 boundary
    #    and do not — so they are enumerated, not represented by `unsigned`.
    for it in INT4:
        for pt in PTR:
            emit(HDR + "%s f(%s p) { return (%s)p; }\n" % (it, pt, it))
            emit(HDR + "%s f(%s a) { return (%s)a; }\n" % (pt, it, pt))

    # 2. the reinterpret in a CALL-ARGUMENT region, at every slot of every arity
    #    up to four — the position where the workload's `calls-1` half lives, and
    #    the one where a class the callee's formal disagrees with would show up.
    for k in range(1, 5):
        for j in range(k):
            formals = ', '.join('int' for _ in range(k))
            params = ', '.join(('S *p%d' % i) if i == j else ('int a%d' % i) for i in range(k))
            actual = ', '.join(('(int)p%d' % i) if i == j else ('a%d' % i) for i in range(k))
            emit(HDR + "int g%d(%s);\nint f(%s) { return g%d(%s); }\n"
                 % (k, formals, params, k, actual))
            pformals = ', '.join('void *' for _ in range(k))
            pparams = ', '.join(('int a%d' % i) if i == j else ('void *p%d' % i) for i in range(k))
            pactual = ', '.join(('(void *)a%d' % i) if i == j else ('p%d' % i) for i in range(k))
            emit(HDR + "int h%d(%s);\nint f(%s) { return h%d(%s); }\n"
                 % (k, pformals, pparams, k, pactual))

    # 3. the argument PERMUTED out of slot order, and one converted formal used
    #    twice — the two shapes that make the register move visible underneath a
    #    conversion that emits nothing.
    emit(HDR + "int g2(int, int);\nint f(S *p, int b) { return g2(b, (int)p); }\n")
    emit(HDR + "int g2(int, int);\nint f(int a, S *p) { return g2((int)p, a); }\n")
    emit(HDR + "int g2(int, int);\nint f(S *p) { return g2((int)p, (int)p); }\n")
    emit(HDR + "int g3(int, int, int);\nint f(S *p, S *q, int c) "
               "{ return g3((int)q, (int)p, c); }\n")

    # 4. `this` — the const pointer `A6 43`, and the only pointer in the language
    #    the programmer did not spell.
    for expr, ret in [('(int)this', 'int'), ('(unsigned)this', 'unsigned'),
                      ('(long)this', 'long'), ('(void *)this', 'void *')]:
        emit("struct C { int m; %s f() const; };\n%s C::f() const { return %s; }\n"
             % (ret, ret, expr))
    emit("struct C { int m; int f(int) const; };\nint g2(int, int);\n"
         "int C::f(int a) const { return g2((int)this, a); }\n")

    # 5. stacked conversions — the round trip and back, at every depth up to
    #    four. `docs/IL_CAST_CONVERT.md` §2.2(b): conversions do NOT compose
    #    token-by-token, so a chain of them is not a chain of instructions.
    for depth in ['(int)(void *)a',
                  '(unsigned)(void *)a',
                  '(int)(void *)(int)a',
                  '(int)(S *)(unsigned)(void *)a',
                  '(long)(char *)(int)a']:
        emit(HDR + "int f(int a) { return %s; }\n" % depth)
    for depth in ['(void *)(int)p', '(S *)(unsigned)p', '(void *)(long)(int)p']:
        emit(HDR + "void *f(S *p) { return %s; }\n" % depth)

    # ---- 6. THE REFUSING NEIGHBOURS -----------------------------------------
    #
    # A MISMATCH in this block is the alarm. Each case emits a real instruction
    # that a chain dropping the conversion would omit.
    #
    # 6a. SCALED pointer arithmetic reached through a CONVERSION rather than off
    #     a LOAD. `(S *)a + 1` is `addi r3,r3,8`; `(S *)a + k` is
    #     `slwi r11,r4,3 ; add`. Nothing in the corpus could build this shape
    #     before the reinterpret existed, because `saw_ptr` was only ever set by
    #     a LOAD — which is exactly the "the corpus cannot express the failure"
    #     hole #688 shipped through. Every pointee width, every literal that
    #     changes the scale, and the variable-index form that strength-reduces.
    for pt, brief in [('S *', 'S'), ('char *', 'char'), ('int *', 'int'),
                      ('S **', 'Sp'), ('const char *', 'kchar')]:
        for k in ['1', '2', '3', '7', 'k']:
            params = 'int a' if k != 'k' else 'int a, int k'
            emit(HDR + "%s f(%s) { return ((%s)a) + %s; }\n" % (pt, params, pt, k))
            emit(HDR + "%s f(%s) { return ((%s)a) - %s; }\n" % (pt, params, pt, k))

    # 6b. **NOT a refusal any more — board #701.** Arithmetic on the INT side of
    #     the conversion: c2 emits a plain `add`/`subf`/`mullw` here, because at
    #     the operator the value is an integer and there is nothing to scale.
    #     These cases were written as neighbours of the refusal, with a note that
    #     "the pointer guard is on the whole sub-expression ... an attempt to make
    #     the guard precise has a corpus waiting for it." It did, it was made
    #     precise, and these are the accepts. Left in this block, and the note
    #     left beside them, so the boundary's movement is legible.
    for op in ['+', '-', '*']:
        emit(HDR + "int f(void *p, int b) { return ((int)p) %s b; }\n" % op)
        emit(HDR + "int f(void *p, int b) { return b %s ((int)p); }\n" % op)
        emit(HDR + "int f(void *p, void *q) { return ((int)p) %s ((int)q); }\n" % op)
        emit(HDR + "void *f(int a, int b) { return (void *)(a %s b); }\n" % op)
        emit(HDR + "S *f(int a, int b) { return (S *)(a %s b); }\n" % op)

    # 6c. `bool` / `unsigned char` on either side. `unsigned u(bool b){return b;}`
    #     is `rlwinm r3,r3,0,24,31` and `(void *)b` is the SAME mask — the
    #     pointer direction is not the free one the `ValueClass` enum suggests,
    #     and this block is the one that would catch a widening that read "a
    #     value class is a value class".
    for src in ['bool', 'unsigned char']:
        for dst in ['int', 'unsigned', 'long', 'void *', 'S *', 'char *']:
            emit(HDR + "%s f(%s b) { return (%s)b; }\n" % (dst, src, dst))
    for src in ['void *', 'S *', 'int', 'unsigned']:
        for dst in ['bool', 'unsigned char']:
            emit(HDR + "%s f(%s p) { return (%s)p; }\n" % (dst, src, dst))

    # 6d. the width boundary from a pointer source, which `61-` reaches only from
    #     an int one: every narrowing and widening target against a pointer.
    for dst in ['char', 'signed char', 'unsigned char', 'short', 'unsigned short',
                'long long', 'unsigned long long', 'float', 'double']:
        emit(HDR + "%s f(S *p) { return (%s)p; }\n" % (dst, dst))
        emit(HDR + "%s f(void *p) { return (%s)p; }\n" % (dst, dst))

    # 6d-bis. THE WORKLOAD'S OWN SHAPE (board #702): a conversion applied to the
    #     RESULT of a pointer DIFFERENCE. This is where every one of the 5,712
    #     functions the reinterpret unblocks actually lands, and c2 lowers it as
    #     a subtract plus a divide by the pointee width.
    for pt in ['S *', 'char *', 'int *', 'double *', 'const char *']:
        for dst in ['int', 'unsigned', 'long']:
            emit(HDR + "%s f(%s p, %s q) { return (%s)(p - q); }\n" % (dst, pt, pt, dst))

    # 6e. a `volatile` pointer OBJECT formal — `int * volatile p` is a volatile
    #     object, so c2 homes it in the frame and reads it back. The refusal is at
    #     the operand LOAD and must not be reached through the conversion.
    emit(HDR + "int f(int * volatile p) { return (int)p; }\n")
    emit(HDR + "int f(volatile int *p) { return (int)p; }\n")
    emit(HDR + "unsigned f(S * volatile p) { return (unsigned)p; }\n")
