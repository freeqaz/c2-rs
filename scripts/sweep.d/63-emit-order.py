# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# **The `.text` EMISSION ORDER** (board row X-d). c2 does not emit a TU's
# functions in `.ex` order: it emits a function only once every function it
# references *and defines* has been emitted. The port emitted `.ex` order flat,
# which is a silent wrong-bytes obj — it links, every relocation resolves, and
# only a byte compare says anything. Six live `Port=Mismatch` reproducers sat on
# master and **nothing generated the shape**, which is exactly how #232 survived
# 241 commits one layer up.
#
# So the axis this fragment sweeps is **structural, not value-shaped**. This
# project's grids have varied *values* exhaustively three times and missed on
# arity, register position and structural counts; here the values (types,
# literals, member offsets) are held fixed at `int` on purpose and the cross is
# over the shape of the TU:
#
#   A. the REFERENCE KIND that creates the ordering edge —
#        none | a framed `bl` | a tail branch | base-ctor delegation |
#        a `26` unwind action that emits NO BYTES AT ALL | two edges
#   B. the callee's POSITION relative to its caller — before, after, and after
#      with an unrelated function wedged between
#   C. the NUMBER of functions in the TU — 2, 3, 4
#   D. the position of the UNRELATED function — first, middle, last
#   E. CHAIN DEPTH — one link or two (a single deferral pass gets depth 2 wrong)
#   F. FAN-IN — two callers of one callee (their relative order must be kept)
#   G. base vs derived vs unrelated class, implicit vs explicit destructor
#   H. CYCLES — self-recursion, a 2-cycle, a 3-cycle. c2 folds the recursion and
#      no worklist reproduces what it emits, so these must REFUSE.
#
# Axis A's fifth value is the one no obj can show: `struct B{B();~B();int x;};
# struct D:B{D();}; D::D(){} B::~B(){}` has no `bl`, no relocation and no symbol
# for `??1B` anywhere, and c2 still emits it first. A sweep built from what the
# obj references would generate every other row here and miss that one.
#
# NotImplemented is fine and expected on most rows — the point is that a
# MISMATCH here is an alarm and there was no instrument to raise it.

# A leaf with no edges at all, for wedging between the interesting ones.
LEAF = "int %s(int a){return a+%d;}\n"


def cases(emit):
    # ---- A x B x C: one edge, every reference kind, every position ----------
    #
    # Each entry is (callee definition, caller definition, extra declarations).
    # The caller references the callee; nothing else in the TU does.
    EDGES = (
        # a bare void tail branch: `b ?g`
        ("void g(){}\n", "void f(){g();}\n", "void g();\n"),
        # an integer tail call with an argument
        ("int q(int a){return a+1;}\n", "int f(int a){return q(a);}\n", "int q(int);\n"),
        # a framed non-leaf call: `bl ?q` inside the 96-byte frame
        ("int q(int a){return a+1;}\n", "int f(int a){return q(a)+1;}\n", "int q(int);\n"),
        # two calls to the SAME callee — one edge, not two
        ("void g(){}\n", "void f(){g();g();}\n", "void g();\n"),
        # two calls to two callees, only one of them local
        ("void g(){}\n", "void f(){g();h();}\n", "void g();void h();\n"),
        # base-constructor delegation: the `bl` is to the base's CONSTRUCTOR
        ("B::B(){}\n", "D::D(){}\n", "struct B{B();int x;};\nstruct D:B{D();};\n"),
        # base-destructor delegation: a bare `b ??1B`
        ("B::~B(){}\n", "D::~D(){}\n", "struct B{~B();int x;};\nstruct D:B{~D();};\n"),
        # THE UNWIND-ACTION EDGE. `??0D` branches to `??0B`, which is NOT defined
        # here; its only reference to `??1B` is the `26` unwind action, which
        # emits no instruction, no relocation and no symbol. The obj cannot show
        # this edge and c2 orders on it anyway.
        ("B::~B(){}\n", "D::D(){}\n", "struct B{B();~B();int x;};\nstruct D:B{D();};\n"),
        # the same edge with a non-empty callee body, so the callee is framed too
        ("void h();\nB::~B(){h();}\n", "D::D(){}\n",
         "struct B{B();~B();int x;};\nstruct D:B{D();};\n"),
        # X-d itself: the edge crosses two inheritance levels
        ("M::~M(){}\n", "D::D(){}\n",
         "struct Bd{Bd();~Bd();int b0;};\nstruct M:Bd{M();~M();};\nstruct D:M{D();};\n"),
    )
    for callee, caller, decls in EDGES:
        # C = 2, both orders. The callee-first row is the control: it must stay
        # put, and it is what made the defect look like it did not exist.
        emit(decls + callee + caller)
        emit(decls + caller + callee)
        # C = 3, the unrelated leaf in each of the three positions (axis D),
        # for each of the two orders of the pair.
        for lead, mid, tail in (
            (LEAF % ("z", 1), "", ""),
            ("", LEAF % ("z", 1), ""),
            ("", "", LEAF % ("z", 1)),
        ):
            emit(decls + lead + callee + mid + caller + tail)
            emit(decls + lead + caller + mid + callee + tail)
        # C = 4: two unrelated leaves bracketing the pair, both orders.
        emit(decls + (LEAF % ("z", 1)) + caller + (LEAF % ("y", 2)) + callee)
        emit(decls + (LEAF % ("z", 1)) + callee + (LEAF % ("y", 2)) + caller)

    # ---- E: CHAIN DEPTH. One deferral pass produces `c, a, b` for a->b->c and
    # the oracle produces `c, b, a`; a DFS from each root produces something
    # else again. Every permutation of the three definitions is swept, because
    # the result must be the same for all six.
    CHAIN = {
        "a": "void a(){b();}\n",
        "b": "void b(){c();}\n",
        "c": "void c(){}\n",
    }
    DECL = "void a();void b();void c();\n"
    for p in (
        "abc", "acb", "bac", "bca", "cab", "cba",
    ):
        emit(DECL + "".join(CHAIN[k] for k in p))
        # …and the same chain with an unrelated leaf at the front and the back,
        # which is where a "defer the caller to the end" rule diverges.
        emit(DECL + (LEAF % ("z", 1)) + "".join(CHAIN[k] for k in p))
        emit(DECL + "".join(CHAIN[k] for k in p) + (LEAF % ("z", 1)))

    # A four-deep chain: three passes are not enough either.
    DEEP = {
        "a": "void a(){b();}\n",
        "b": "void b(){c();}\n",
        "c": "void c(){d();}\n",
        "d": "void d(){}\n",
    }
    for p in ("abcd", "dcba", "badc", "cdab", "adbc", "bdac"):
        emit("void a();void b();void c();void d();\n"
             + "".join(DEEP[k] for k in p))

    # ---- F: FAN-IN and FAN-OUT. Two callers of one callee keep their relative
    # `.ex` order behind it; one caller of two callees keeps theirs ahead of it.
    emit("void h();\nvoid f(){h();}\nvoid g(){h();}\nvoid h(){}\n")
    emit("void h();\nvoid h(){}\nvoid f(){h();}\nvoid g(){h();}\n")
    emit("void h();\nvoid f(){h();}\nvoid h(){}\nvoid g(){h();}\n")
    emit("void g();void h();\nvoid f(){g();h();}\nvoid g(){}\nvoid h(){}\n")
    emit("void g();void h();\nvoid f(){g();h();}\nvoid h(){}\nvoid g(){}\n")
    emit("void g();void h();\nvoid g(){}\nvoid f(){g();h();}\nvoid h(){}\n")
    # Two independent pairs, interleaved: the two edges must not couple.
    emit("void g();void k();\nvoid f(){g();}\nvoid j(){k();}\nvoid g(){}\nvoid k(){}\n")
    emit("void g();void k();\nvoid f(){g();}\nvoid k(){}\nvoid j(){k();}\nvoid g(){}\n")

    # ---- G: the class shape around the unwind edge, and the CONTROLS. --------
    # Every row here has the same two function definitions and differs only in
    # whether an ordering edge exists at all. A fragment that only generated the
    # positive rows would pass with a rule that reorders unconditionally.
    CONTROLS = (
        # NO edge: `D`'s base has no destructor, so `??0D` carries no `26` and
        # `??1B` is an unrelated function. MUST stay in source order.
        "struct B{B();~B();int x;};\nstruct C{C();int y;};\nstruct D:C{D();};\n"
        "D::D(){}\nB::~B(){}\n",
        # the same with no constructor on the unrelated class either
        "struct B{~B();int x;};\nstruct C{C();int y;};\nstruct D:C{D();};\n"
        "D::D(){}\nB::~B(){}\n",
        # two classes sharing an UNDEFINED base: neither references the other
        "struct B{B();~B();int x;};\nstruct D:B{D();};\nstruct E:B{~E();};\n"
        "D::D(){}\nE::~E(){}\n",
        # ctor and dtor of the SAME class, base undefined: no edge either way
        "struct B{B();~B();int x;};\nstruct D:B{D();~D();};\nD::D(){}\nD::~D(){}\n",
        "struct B{B();~B();int x;};\nstruct D:B{D();~D();};\nD::~D(){}\nD::D(){}\n",
    )
    for src in CONTROLS:
        emit(src)

    # The IMPLICIT sibling destructor — the shape the generator had never
    # written, and the one the whole row came from. `M` declares no destructor,
    # so c2 generates `??1M` and (packed or not) gives it its own COMDAT. It must
    # REFUSE; a mismatch here is #232 returning.
    for dmem in ("", "int d;", "double d;"):
        emit("struct Bd{Bd();~Bd();int b0;};\nstruct M:Bd{M();};\n"
             "struct D:M{D();%s};\nD::D(){}\n" % dmem)
        emit("struct Bd{Bd();~Bd();int b0;};\nstruct M:Bd{M();};\n"
             "struct D:M{D();%s};\nD::D(){}\nint z(int a){return a+1;}\n" % dmem)

    # ---- H: CYCLES. c2 folds mutual recursion — `a->b->c->a` written `a,b,c`
    # comes out `b,a,c`, and `a<->b` beside a leaf comes out unpermuted with the
    # leaf LAST, which no dependency worklist can produce. The port refuses; a
    # MISMATCH on any of these is a tie-break rule someone fitted.
    emit("void f(int n){if(n)f(n-1);}\nint z(int a){return a+1;}\n")
    emit("int z(int a){return a+1;}\nvoid f(int n){if(n)f(n-1);}\n")
    emit("void b();\nvoid a(){b();}\nvoid b(){a();}\n")
    emit("void b();\nvoid a(){b();}\nvoid b(){a();}\nint z(int q){return q+1;}\n")
    emit("void b();\nint z(int q){return q+1;}\nvoid a(){b();}\nvoid b(){a();}\n")
    emit("void b();void c();\nvoid a(){b();}\nvoid b(){c();}\nvoid c(){a();}\n")
    emit("void b();void c();\nvoid a(){b();}\nvoid b(){c();}\nvoid c(){b();}\n")
    emit("void b();void c();\nvoid c(){b();}\nvoid b(){c();}\nvoid a(){b();}\n")
