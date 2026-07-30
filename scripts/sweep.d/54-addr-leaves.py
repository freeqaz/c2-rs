# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the driver; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace. A fragment cannot see
# or touch another fragment's counter (the `n`-shadowing trap that silently
# overwrote 1,233 already-written cases is unrepresentable here), and the driver
# fails if a fragment emits zero cases.


def cases(emit):
    # ---- address leaves: `return &s->m;` at both designators -------------------------
    # `docs/IL_CALL_IN_EXPR.md` §19. The newest accepted class, and the one whose gate
    # is *loosest by design*: the member's own type never reaches the emitted `addi`,
    # so the address path admits every pointer TYPE where the load path beside it picks
    # `lbz`/`lhz`/`lwz`/`ld` from exactly that field. Two productions therefore share
    # one designator decoder and disagree about what it may carry — which is precisely
    # the shape of the bug `docs/GAPS.md` §6 keeps recording, and only the cross product
    # separates "the width does not matter for an address" from "the width was never
    # varied in the fixtures".
    #
    # The axes: designator (plain `27` add vs intrinsic 2117) x member width x offset
    # (including 0, which emits NOTHING, and both sides of the signed 16-bit edge) x
    # base argument position (r3 vs r4 vs r5 — the `addi`'s rA field) x cv-qualification
    # x the `28` subscript add x array decay.
    ADDR_S = (
        "struct S1 { char a, b, c, d; };\n"
        "struct S2 { short a, b; };\n"
        "struct S4 { int a, b, c, d; };\n"
        "struct S8 { double a; int b; };\n"
        "struct SA { int h; int arr[4]; };\n"
        "struct A4 { int a0, a1; };\n"
        "struct B4 { int b0, b1, b2; };\n"
        "struct D4 : A4, B4 { int d; };\n"
        "struct AR { int t[4]; };\n"
        "struct DR : B4, AR { };\n"
    )
    # The plain designator: member x cv x argument position. `&r.m` through a reference
    # is the same production reached from a different source spelling.
    for st, mem, ty in (('S1','a','char'), ('S1','d','char'), ('S2','a','short'),
                        ('S2','b','short'), ('S4','a','int'), ('S4','d','int'),
                        ('S8','a','double'), ('S8','b','int')):
        for q in ('', 'const ', 'volatile '):
            emit(ADDR_S + "%s%s* f(%s%s* p) { return &p->%s; }\n" % (q, ty, q, st, mem))
            emit(ADDR_S + "%s%s* f(%s%s& r) { return &r.%s; }\n" % (q, ty, q, st, mem))
        emit(ADDR_S + "%s* f(int x, %s* p) { return &p->%s; }\n" % (ty, st, mem))
        emit(ADDR_S + "%s* f(int x, int y, %s* p) { return &p->%s; }\n" % (ty, st, mem))
        emit(ADDR_S + "void* f(%s* p) { return &p->%s; }\n" % (st, mem))
    # The subscript add, at every index including the ones that make the total zero, and
    # the bare array (a `2C` decay). Two adds in a row must FOLD, where the load leaf
    # beside them admits only one.
    for ix in ('0', '1', '3', '-1'):
        emit(ADDR_S + "int* f(SA* p) { return &p->arr[%s]; }\n" % ix)
        emit(ADDR_S + "int* f(int x, SA* p) { return &p->arr[%s]; }\n" % ix)
    emit(ADDR_S + "int* f(SA* p) { return p->arr; }\n")
    emit(ADDR_S + "int* f(int x, SA* p) { return p->arr; }\n")
    # The signed-16-bit edge, from both sides and at both designators. 32764 is one
    # `addi`; 32768 is `addis`+`addi` and must refuse.
    for pad in ('32756', '32760', '32764', '32765', '32768', '40000'):
        emit("struct P { char pad[%s]; int t; };\nint* f(P* p) { return &p->t; }\n" % pad)
        emit("struct BP { char pad[%s]; };\nstruct DP : BP { int t; };\n"
                 "int* f(DP* p) { return &p->t; }\n" % pad)
    # The intrinsic-2117 designator: every member of a two-base derived class, so the
    # two literals are exercised at (0,0), (nonzero,0), (0,nonzero) and (nonzero,nonzero)
    # — the only cross that separates a SUM from "whichever one is nonzero".
    for mem in ('a0', 'a1', 'b0', 'b1', 'b2', 'd'):
        emit(ADDR_S + "int* f(D4* p) { return &p->%s; }\n" % mem)
        emit(ADDR_S + "int* f(int x, D4* p) { return &p->%s; }\n" % mem)
        emit(ADDR_S + "const int* f(const D4* p) { return &p->%s; }\n" % mem)
        emit(ADDR_S + "void* f(D4* p) { return &p->%s; }\n" % mem)
        # …and the LOAD of the same member, which shares the designator decoder and
        # must keep picking its instruction from the width the address path ignores.
        emit(ADDR_S + "int f(D4* p) { return p->%s; }\n" % mem)
    # The same through `this`, const and non-const, plus a second inheritance step
    # (class descriptor `66 03` rather than `66 02`).
    for mem in ('a0', 'b1'):
        emit(ADDR_S + "struct C : D4 { int* g(); const int* gc() const; };\n"
                 "int* C::g() { return &%s; }\n" % mem)
        emit(ADDR_S + "struct C : D4 { const int* gc() const; };\n"
                 "const int* C::gc() const { return &%s; }\n" % mem)
    # An inherited ARRAY member: the `28` add lands AFTER the intrinsic rather than
    # after a `B9`, which is the one ordering the plain form never produces.
    for ix in ('0', '1', '3'):
        emit(ADDR_S + "int* f(DR* p) { return &p->t[%s]; }\n" % ix)
    emit(ADDR_S + "int* f(DR* p) { return p->t; }\n")
    # Inherited members of every width — the axis the address path deliberately drops
    # and the load path must not.
    ADDR_W = ("struct BW { int b0, b1; };\n"
              "struct W { char wc; short ws; int wi; long long wl; float wf; double wd; };\n"
              "struct DW : BW, W { };\n")
    for mem, ty in (('wc','char'), ('ws','short'), ('wi','int'),
                    ('wl','long long'), ('wf','float'), ('wd','double')):
        emit(ADDR_W + "%s* f(DW* p) { return &p->%s; }\n" % (ty, mem))
        emit(ADDR_W + "const %s* f(const DW* p) { return &p->%s; }\n" % (ty, mem))
        emit(ADDR_W + "void* f(DW* p) { return &p->%s; }\n" % mem)
        emit(ADDR_W + "%s f(DW* p) { return p->%s; }\n" % (ty, mem))
        emit(ADDR_W + "int f(DW* p) { return (int)p->%s; }\n" % mem)
        emit(ADDR_W + "void f(DW* p, %s v) { p->%s = v; }\n" % (ty, mem))
    # The refusing neighbours. Each is one token from an accepted shape and each costs
    # an instruction the single `addi` does not: a MISMATCH here is the alarm.
    ADDR_V = ("struct VA { int v0, v1; };\n"
              "struct VD : virtual VA { int d2; };\n")
    for src in (
        # A VIRTUAL base: intrinsic 2118, a vbtable indirection, not a constant offset.
        ADDR_V + "int* f(VD* p) { return &p->v1; }\n",
        ADDR_V + "int f(VD* p) { return p->v1; }\n",
        ADDR_V + "int* f(VD* p) { return &p->d2; }\n",
        # A variable index: the offset is not a literal at all.
        ADDR_S + "int* f(SA* p, int i) { return &p->arr[i]; }\n",
        ADDR_S + "int* f(DR* p, int i) { return &p->t[i]; }\n",
        # The address of a GLOBAL's member: a relocation pair, not an argument register.
        ADDR_S + "S4 g;\nint* f() { return &g.b; }\n",
        ADDR_S + "D4 g;\nint* f() { return &g.b1; }\n",
        # The address CONVERTED to an integer, and pointer arithmetic on the result.
        ADDR_S + "int f(S4* p) { return (int)&p->b; }\n",
        ADDR_S + "int* f(S4* p) { return &p->b + 1; }\n",
        ADDR_S + "int* f(S4* p, int i) { return &p->b + i; }\n",
        # A second statement: the production must reach the end of the segment.
        ADDR_S + "int* f(S4* p, int* q) { *q = 1; return &p->b; }\n",
        ADDR_S + "int* f(S4* p) { int* r = &p->b; return r; }\n",
        # A member of a member, and a member of a base of a member.
        ADDR_S + "struct O { int h; S4 s; };\nint* f(O* p) { return &p->s.b; }\n",
        ADDR_S + "struct O { int h; D4 d; };\nint* f(O* p) { return &p->d.b1; }\n",
        # The address of the object itself, and of a base sub-object (an upcast, which
        # is intrinsic 2114 and null-guarded — an `addi` AND a branch).
        ADDR_S + "D4* f(D4* p) { return p; }\n",
        ADDR_S + "B4* f(D4* p) { return p; }\n",
        ADDR_S + "A4* f(D4* p) { return p; }\n",
        # A member function POINTER's address, and a reference-typed member.
        ADDR_S + "struct FP { int h; void (*f)(); };\nvoid (**g(FP* p))() { return &p->f; }\n",
        # A bitfield: not addressable in C++, but the neighbouring plain member is, and
        # the layout the bitfield forces is what makes the offsets interesting.
        "struct BF { int a : 3; int b : 5; int c; };\nint* f(BF* p) { return &p->c; }\n",
    ):
        emit(src)
