# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the driver; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace. A fragment cannot see
# or touch another fragment's counter (the `n`-shadowing trap that silently
# overwrote 1,233 already-written cases is unrepresentable here), and the driver
# fails if a fragment emits zero cases.


def cases(emit):
    # ---- pointer OPERANDS: the type gate at the LOAD, LIT, argument and result slots --
    # `docs/IL_CALL_IN_EXPR.md` §21. `parse_expr` now admits a 4-byte pointer TYPE
    # wherever it admits an int-like one, and the `55` formal type and the `41` result
    # type were widened with it — without those two positions the operand widening
    # admits no real call site at all (measured: 1.2 M functions changed census key and
    # the numerator moved by exactly 0).
    #
    # The claim being swept is "a 4-byte pointer in a register is a 4-byte int in a
    # register", and the axes are the ones that could falsify it independently: the
    # POINTEE (whose width is what pointer arithmetic scales by, and which the tag
    # carries in the *other* type position), the cv-spelling (which moves the tag `86`
    # → `A6`/`96`/`B6`, and `A6` is a const-qualified POINTER — measured, not the
    # const-qualified pointee, which stays `86`), and the ARGUMENT SLOT (because
    # "already in the right register" is exactly what makes these free, and position is
    # what breaks it).
    PTR_S = "struct PS1 { char a; };\nstruct PS8 { double a; int b; };\n"
    PTEES = ('int', 'const int', 'volatile int', 'unsigned', 'long', 'char',
             'const char', 'short', 'float', 'double', 'long long', 'void',
             'int*', 'PS1', 'PS8')
    for pte in PTEES:
        ty = '%s*' % pte
        # the LOAD position, with the result staying int (only the operand is a pointer)
        emit(PTR_S + "int g(%s);\nint f(%s p){ return g(p); }\n" % (ty, ty))
        # …and with the RESULT position a pointer too
        emit(PTR_S + "%s g(%s);\n%s f(%s p){ return g(p); }\n" % (ty, ty, ty, ty))
        # the LIT position: a null pointer constant in an argument, and as a whole body
        emit(PTR_S + "int g(%s);\nint f(){ return g(0); }\n" % ty)
        emit(PTR_S + "%s f(){ return 0; }\n" % ty)
        # a const-qualified POINTER (tag `A6`), at the load and at the formal
        emit(PTR_S + "int g(%s const);\nint f(%s const p){ return g(p); }\n" % (ty, ty))
    # The four tag spellings `is_ptr4_kind` admits, as the loaded value itself, plus the
    # code-pointer kind (`44`) that shares the predicate with the data one (`43`).
    for decl, arg in (('int* p', 'p'), ('const int* p', 'p'), ('volatile int* p', 'p'),
                      ('int* const p', 'p'), ('int (*p)(int)', 'p'), ('void (*p)()', 'p'),
                      ('int (**p)(int)', 'p')):
        emit("int g1(int);\n" +
                 "int f(%s){ return (int)(long)%s; }\n" % (decl, arg))
    for cal, decl in (('int g(int (*)(int));', 'int (*p)(int)'),
                      ('int g(void (*)());', 'void (*p)()'),
                      ('int g(int**);', 'int** p'),
                      ('int g(const int*);', 'const int* p'),
                      ('int g(volatile int*);', 'volatile int* p')):
        emit("%s\nint f(%s){ return g(p); }\n" % (cal, decl))

    # Every ARGUMENT SLOT, pointer against int, at every arity the class accepts. A gate
    # written for "the pointer is the first argument" passes the one-argument case and
    # every all-pointer case, and fails only here.
    for n_args in (1, 2, 3, 4):
        for slot in range(n_args):
            tys = ['int'] * n_args
            tys[slot] = 'int*'
            params = ', '.join('%s a%d' % (t, i) for i, t in enumerate(tys))
            args = ', '.join('a%d' % i for i in range(n_args))
            emit("int g(%s);\nint f(%s){ return g(%s); }\n"
                     % (', '.join(tys), params, args))
            # the same slot taking a null pointer LITERAL instead of a passed-in value
            pars2 = ', '.join('%s a%d' % (t, i) for i, t in enumerate(tys) if i != slot)
            args2 = ', '.join('0' if i == slot else 'a%d' % i for i in range(n_args))
            emit("int g(%s);\nint f(%s){ return g(%s); }\n"
                     % (', '.join(tys), pars2 if pars2 else 'void', args2))
        # all pointers, and pointers of MIXED pointee width in one call
        allp = ', '.join(['int*'] * n_args)
        emit("int g(%s);\nint f(%s){ return g(%s); }\n"
                 % (allp, ', '.join('int* a%d' % i for i in range(n_args)),
                    ', '.join('a%d' % i for i in range(n_args))))
        mix = [('char*', 'short*', 'int*', 'double*')[i % 4] for i in range(n_args)]
        emit("int g(%s);\nint f(%s){ return g(%s); }\n"
                 % (', '.join(mix),
                    ', '.join('%s a%d' % (t, i) for i, t in enumerate(mix)),
                    ', '.join('a%d' % i for i in range(n_args))))

    # ---- the ARITHMETIC BOUNDARY, which is the whole reason the guard exists ----------
    # `p + 1` is `addi r3,r3,4` for an `int*` and `addi r3,r3,1` for a `char*`, so the
    # increment is the POINTEE's width and an add chain that used the literal would be
    # wrong bytes for every width but one. MEASURED (§21.1): c1xx pre-scales, so the IL
    # literal is already 4 — but that is a *second* rule, and until it is graded on its
    # own axis every one of these must be `NotImplemented`. The sweep is what turns
    # "must refuse" into a fact: a MISMATCH here is the alarm.
    for pte in ('char', 'short', 'int', 'long', 'long long', 'double', 'int*', 'PS8'):
        ty = '%s*' % pte
        for e in ('p + 1', 'p - 1', 'p + 3', 'p + k', 'p - k', '1 + p', 'p + 0',
                  'p + (k * 2)', 'p - (k + 1)'):
            emit(PTR_S + "%s f(%s p, int k){ return %s; }\n" % (ty, ty, e))
        # the same arithmetic in an ARGUMENT position, where a different `parse_expr`
        # call sees it, and in a RETURNED-through-a-call position
        emit(PTR_S + "int g(%s);\nint f(%s p){ return g(p + 1); }\n" % (ty, ty))
        emit(PTR_S + "int g(%s, int);\nint f(%s p, int k){ return g(p + k, k); }\n"
                 % (ty, ty))
        # pointer DIFFERENCE: the front end divides by the pointee width, which for a
        # power of two is an arithmetic shift the operand vocabulary refuses anyway —
        # so this class fails closed twice, and the sweep says so rather than assuming.
        emit(PTR_S + "int f(%s p, %s q){ return (int)(p - q); }\n" % (ty, ty))
    # A pointer and an int in one expression with the arithmetic on the INT — the guard
    # is on the whole value, so these refuse too, and that cost is measured not argued.
    for e in ('g(p, a + 1)', 'g(p, a * b)', 'g(p, 1)'):
        emit("int g(int*, int);\nint f(int* p, int a, int b){ return %s; }\n" % e)

    # ---- the refusing NEIGHBOURS of the widened gate ---------------------------------
    # Each is one token from an admitted shape and each costs an instruction the tail
    # call and the identity do not emit.
    for src in (
        # `this` reached through a cv-strip: the A6-tagged LOAD is admitted and the `2C`
        # after it is not, which is where 98.6 % of the pointer-type population went.
        "struct C { int v; int m(); };\nint gC(C*);\nint C::m(){ return gC(this); }\n",
        "struct C { int v; int m() const; };\nint gC(const C*);\nint C::m() const { return gC(this); }\n",
        # a pointer COMPARED rather than passed: a relational opcode, not an operand
        "int f(int* p){ return p != 0; }\n",
        "int f(int* p, int* q){ return p == q; }\n",
        # a pointer DEREFERENCED in an argument: a `30` load, gated separately
        "int g(int);\nint f(int* p){ return g(*p); }\n",
        # the ADDRESS of a local passed as an argument: a frame, and a `27` with no base
        "int g(int*);\nint f(int a){ return g(&a); }\n",
        # an 8-byte operand that is NOT a pointer: the width gate must still refuse
        "int g(long long);\nint f(long long a){ return g(a); }\n",
        "long long f(long long a){ return a; }\n",
        # a REFERENCE parameter, which is a pointer in the IL but is dereferenced on use
        "int g(int&);\nint f(int& r){ return g(r); }\n",
        # a pointer through a varargs callee: the calling-convention byte is `40`
        "int g(int*, ...);\nint f(int* p){ return g(p, 1); }\n",
        # a pointer to an AGGREGATE returned by value: an sret bind, not an operand
        "struct BigA { int a[8]; };\nBigA g(int*);\nBigA f(int* p){ return g(p); }\n",
        # a FLOAT beside a pointer: two register files, and only one of them is modeled
        "int g(int*, float);\nint f(int* p, float x){ return g(p, x); }\n",
        # nine pointer arguments: past the eighth the class refuses on the frame
        "int g(int*,int*,int*,int*,int*,int*,int*,int*,int*);\n"
        "int f(int* a,int* b,int* c,int* d,int* e,int* h,int* i,int* j,int* k)"
        "{ return g(a,b,c,d,e,h,i,j,k); }\n",
    ):
        emit(src)
