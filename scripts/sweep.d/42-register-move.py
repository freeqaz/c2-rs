# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the driver; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace. A fragment cannot see
# or touch another fragment's counter (the `n`-shadowing trap that silently
# overwrote 1,233 already-written cases is unrepresentable here), and the driver
# fails if a fragment emits zero cases.


def cases(emit):
    # ---- the REGISTER MOVE: `return <a formal that is not the first>` ---------------
    # One `mr r3,rN` and a `blr`, where N is the formal's argument slot. The axis that
    # matters is **position x value class**: the whole class rests on a formal's index
    # in the list being its argument-register number, and that identity is exactly what
    # a by-value aggregate or a stack-homed ninth argument breaks. Swept as a cross
    # product rather than one axis at a time, because the two facts coincide for every
    # scalar and only come apart when they are varied together.
    MOVE_STRUCTS = (
        "struct S { int a; int b; int arr[3]; };\n"
        "struct Pair { int x, y; };\n"
        "struct Big { int a[8]; };\n"
    )
    # Every argument slot at every arity, so the move's source register is swept over
    # the whole of r3..r10 and not only its ends.
    for nargs in range(1, 9):
        ps = ', '.join('int p%d' % i for i in range(nargs))
        for i in range(nargs):
            emit("int f(%s) { return p%d; }\n" % (ps, i))
            emit("int g(int);\nint f(%s) { return g(p%d); }\n" % (ps, i))
    # Value class x position. One GPR word is one GPR word — int, unsigned and every
    # pointer spelling share the instruction — while the narrow, wide and FP classes
    # refuse on their operand type ahead of any question about the move.
    for ty in ('int', 'unsigned', 'short', 'long long', 'char', 'bool', 'float',
               'double', 'int*', 'const int*', 'char*', 'void*', 'int**', 'S*'):
        for pos in range(3):
            ps = ', '.join(['int a%d' % k for k in range(pos)] + ['%s v' % ty])
            emit(MOVE_STRUCTS + "%s f(%s) { return v; }\n" % (ty, ps))
        emit(MOVE_STRUCTS + "%s f(%s a, %s v) { return v; }\n" % (ty, ty, ty))
    # The zero-offset sub-object address against its nonzero neighbour, at each
    # position: the pair that separates the register move from the `addi`.
    for pos in range(3):
        lead = ''.join('int a%d, ' % k for k in range(pos))
        for expr in ('&s->a', '&s->b', 's->arr', '&s->arr[1]'):
            emit(MOVE_STRUCTS + "int* f(%sS* s) { return %s; }\n" % (lead, expr))
        emit(MOVE_STRUCTS + "S* f(%sconst S* s) { return (S*)s; }\n" % lead)
        emit(MOVE_STRUCTS + "void* f(%sS* s) { return s; }\n" % lead)
    # Member functions: `this` takes r3, so the first explicit formal is r4 and every
    # later one shifts with it. The off-by-one `il_this_line70.cpp` pins.
    for nargs in range(1, 4):
        ps = ', '.join('int p%d' % i for i in range(nargs))
        for i in range(nargs):
            emit("struct C { int m(%s) const; };\n"
                     "int C::m(%s) const { return p%d; }\n" % (ps, ps, i))
    emit(MOVE_STRUCTS + "struct C { S* p(S* q) const; };\n"
             "S* C::p(S* q) const { return q; }\n")
    emit(MOVE_STRUCTS + "struct C { int* p(S* q) const; };\n"
             "int* C::p(S* q) const { return &q->a; }\n")
    # The neighbours that must NOT emit a move. A by-value aggregate wider than one
    # GPR makes the index stop being the register number — `docs/GAPS.md` §6's fourth
    # instance, and the reason the move is gated behind `.sy`'s declared widths — and
    # an 8-byte one does not, so both must be swept or the gate is untested. A ninth
    # argument is not in a register at all, and a global is not an argument.
    for agg in ('Big', 'Pair'):
        emit(MOVE_STRUCTS + "int f(%s v, int b) { return b; }\n" % agg)
        emit(MOVE_STRUCTS + "int f(int a, %s v, int b) { return b; }\n" % agg)
        emit(MOVE_STRUCTS + "%s* f(%s v, %s* p) { return p; }\n" % (agg, agg, agg))
        emit(MOVE_STRUCTS + "int g(int);\nint f(%s v, int b) { return g(b); }\n" % agg)
    emit("int f(int a,int b,int c,int d,int e,int h,int i,int j,int k)"
             "{ return k; }\n")
    emit("int gv;\nint f(int a, int b) { return gv; }\n")
    emit("static int sv;\nint f(int a, int b) { return sv; }\n")
    emit("int f(int a, int b) { return b + 1; }\n")
    emit(MOVE_STRUCTS + "S* f(int a, S* s) { return s + 1; }\n")
    emit("int f(int a, int b, int c) { return a ? b : c; }\n")
