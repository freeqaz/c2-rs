# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the driver; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace. A fragment cannot see
# or touch another fragment's counter (the `n`-shadowing trap that silently
# overwrote 1,233 already-written cases is unrepresentable here), and the driver
# fails if a fragment emits zero cases.


def cases(emit):
    # ---- indirect loads: deref, member, subscript ------------------------------------
    # The newest accepted class and, until now, entirely unswept. Two of its gates rest
    # on fields whose *meaning* is unproven — the `28` subscript payload is `00 00` at
    # every captured site with no known semantics, and the `2C` cv strip is treated as
    # free on the same "always observed" basis (docs/GAPS.md §6: a field the port skips
    # is indistinguishable from a field that is always the same). A co-varying semantic
    # — a scaled rather than byte index, a qualification strip that is not free — would
    # pass those gates and emit. Only the cross product separates that from safety.
    STRUCTS = (
        "struct S1 { char a, b, c, d; };\n"
        "struct S2 { short a, b; };\n"
        "struct S4 { int a, b, c, d; };\n"
        "struct S8 { double a; int b; };\n"
        "struct A4 { int a0, a1; };\n"
        "struct B4 { int b0, b1, b2; };\n"
        "struct D4 : A4, B4 { int d; };\n"
    )
    # Element size x index: the axis that would expose a scaled-vs-byte index rule.
    for ty in ('int', 'unsigned', 'long', 'char', 'short', 'float', 'double', 'int*'):
        for ix in ('0', '1', '3', '-1', '-4', '8191', '8192', '100000'):
            emit("int f(%s* p) { return (int)p[%s]; }\n" % (ty, ix))
            emit("%s f(%s* p) { return p[%s]; }\n" % (ty, ty, ix))
    # Member offsets across widths, and the same member reached by `.` and `->`.
    for st, mem in (('S1','a'),('S1','d'),('S2','a'),('S2','b'),('S4','a'),('S4','d'),
                    ('S8','a'),('S8','b')):
        emit(STRUCTS + "int f(%s* p) { return (int)p->%s; }\n" % (st, mem))
        emit(STRUCTS + "int f(%s& r) { return (int)r.%s; }\n" % (st, mem))
    # cv-qualification on the pointee: the axis the `2C` strip claims is free.
    for q in ('', 'const ', 'volatile ', 'const volatile '):
        for ty in ('int', 'unsigned', 'char', 'short'):
            emit("int f(%s%s* p) { return (int)*p; }\n" % (q, ty))
            emit("int f(%s%s* p) { return (int)p[2]; }\n" % (q, ty))
    # Inherited members: the two literals of intrinsic 2117 must ADD, and only a member
    # at a nonzero offset inside a base at a nonzero offset separates that from
    # "whichever is nonzero".
    for mem in ('a0', 'a1', 'b0', 'b1', 'b2', 'd'):
        emit(STRUCTS + "int f(D4* p) { return p->%s; }\n" % mem)
    # Two adds chained, which must refuse rather than fold to one.
    emit(STRUCTS + "int f(S4* p) { return p[2].c; }\n")
    emit(STRUCTS + "int f(int** p) { return *p[1]; }\n")
