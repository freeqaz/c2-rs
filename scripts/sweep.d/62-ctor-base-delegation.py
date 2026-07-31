# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# WEC: the **empty constructor that delegates to one base sub-object**, the
# mirror of `50-dtor-base.py`'s production and the one that is NOT a leaf —
# an MSVC constructor hands `this` back in r3, so `this` is live across the base
# constructor's `bl` and c2 frames the body (`mr r31,r3 ; bl ; mr r3,r31`).
#
# The axis worth sweeping is the **argument region**, because that is where this
# shape's only non-structural gate lives: the explicit arguments must be the
# leading formals, in order, with identical types, and the region is in REVERSE
# source order. Reading it forwards is byte-exact on every one-argument case and
# wrong on every two-argument one — this fragment is what separates them, and a
# hand-written corpus of one-argument probes did not.


def cases(emit):
    # ---- the base's shape, and whether it has a destructor --------------------
    # The destructor is the `/EHsc` half of the census key: with one, the body
    # carries a `26 <base dtor>` unwind action that emits NOTHING, a `5C` and a
    # `5D`, and (at `/EHsc`) one extra label-counter slot. Without one, none of
    # that is present and the `.text` is byte-identical.
    BASES = (('B0', '', ''), ('B1', 'int b0;', ''), ('B4', 'int b0,b1,b2,b3;', ''),
             ('Bd', 'int b0;', '~Bd();'), ('Bd8', 'double b0; char b1;', '~Bd8();'))
    DMEMS = ('', 'int d;', 'double d;', 'char d;', 'int d0,d1,d2;')
    for bn, bdata, bdtor in BASES:
        for dmem in DMEMS:
            emit("struct %s { %s(); %s %s };\nstruct D : %s { D(); %s };\nD::D() {}\n"
                 % (bn, bn, bdtor, bdata, bn, dmem))
            # Two inheritance levels: the delegation is still ONE step, so the
            # class-pair descriptor must still be `66 02`.
            emit("struct %s { %s(); %s %s };\nstruct M : %s { M(); %s };\n"
                 "struct D : M { D(); %s };\nD::D() {}\n"
                 % (bn, bn, bdtor, bdata, bn, dmem, dmem))

    # ---- the ARGUMENT region: arity, order, and which formals are forwarded ---
    # `ARGS` is (the base's parameter list, the derived ctor's parameter list,
    # the forwarding expression). The identity rows must be byte-exact; the rest
    # must refuse rather than mis-emit, which is what this lane grades.
    TYPES = ('int', 'unsigned', 'char', 'short', 'void*', 'const char*')
    for t in TYPES:
        # one forwarded argument, and one with a trailing formal the call ignores
        emit("struct B { B(%s); int x; };\nstruct D : B { D(%s a); };\n"
             "D::D(%s a) : B(a) {}\n" % (t, t, t))
        emit("struct B { B(%s); int x; };\nstruct D : B { D(%s a, int z); };\n"
             "D::D(%s a, int z) : B(a) {}\n" % (t, t, t))
        # a formal the constructor never uses at all
        emit("struct B { B(); int x; };\nstruct D : B { D(%s a); };\n"
             "D::D(%s a) {}\n" % (t, t))
    # arity 2..5, forwarded in order (the identity) — the rows that fail if the
    # argument region is read in source order instead of stream order.
    for n in range(2, 6):
        ps = ", ".join("int a%d" % k for k in range(n))
        fs = ", ".join("a%d" % k for k in range(n))
        ts = ", ".join("int" for _ in range(n))
        emit("struct B { B(%s); int x; };\nstruct D : B { D(%s); };\n"
             "D::D(%s) : B(%s) {}\n" % (ts, ps, ps, fs))
        # …and the same list forwarded REVERSED, which needs a permutation and
        # must refuse: beside a callee-saved copy c2 breaks the cycle through
        # the callee-saved register rather than r11.
        rf = ", ".join("a%d" % k for k in reversed(range(n)))
        emit("struct B { B(%s); int x; };\nstruct D : B { D(%s); };\n"
             "D::D(%s) : B(%s) {}\n" % (ts, ps, ps, rf))
        # …and a rotation by one, which is a single cycle rather than a swap.
        rot = ", ".join("a%d" % ((k + 1) % n) for k in range(n))
        emit("struct B { B(%s); int x; };\nstruct D : B { D(%s); };\n"
             "D::D(%s) : B(%s) {}\n" % (ts, ps, ps, rot))
    # a mixed register file: the FP formals sit in f1.. and the integers in r4..,
    # so an "identity" that is only positional would put them in the wrong file.
    for ps, fs in (("int a, float f", "a, f"), ("float f, int a", "f, a"),
                   ("double d, int a", "d, a"), ("int a, double d, int b", "a, d, b")):
        ts = ", ".join(p.rsplit(" ", 1)[0] for p in ps.split(", "))
        emit("struct B { B(%s); int x; };\nstruct D : B { D(%s); };\n"
             "D::D(%s) : B(%s) {}\n" % (ts, ps, ps, fs))
    # a literal argument (`li r4,k`), and a widening conversion — both refuse.
    for arg in ("3", "0", "-1", "(long long)a", "(double)a", "a + 1"):
        emit("struct B { B(long long); int x; };\nstruct D : B { D(int a); };\n"
             "D::D(int a) : B(%s) {}\n" % arg)

    # ---- neighbours that must NOT be admitted --------------------------------
    # A polymorphic derived class (the base moves off offset 0 and the vfptr
    # store is a second statement), a second destructible base, and a member
    # initializer beside the base.
    emit("struct B { B(); ~B(); int x; };\nstruct D : B { D(); virtual void v(); };\n"
         "D::D() {}\n")
    emit("struct B { B(); ~B(); int x; };\nstruct M { M(); ~M(); int y; };\n"
         "struct D : B, M { D(); };\nD::D() {}\n")
    emit("struct B { B(); ~B(); int x; };\nstruct D : B { D(); int m; };\n"
         "D::D() : m(0) {}\n")

    # ---- the definition's SOURCE LINE ----------------------------------------
    # `this` is bound from the pre-body region and the closing brace's own
    # `4F 01 <line>` marker lands inside the return plumbing — line 70's marker
    # is `4F 01 46`, the known-bad formals anchor. Same reason `50-dtor-base.py`
    # sweeps it, and this shape reads `this` twice as often.
    for line in range(64, 77):
        pad = '\n'.join('// pad %d' % k for k in range(1, line - 2))
        emit("struct B { B(); ~B(); int x; };\nstruct D : B { D(); int y; };\n" + pad
             + "\nD::D()\n{\n}\n")
