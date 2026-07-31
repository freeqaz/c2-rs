# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# ---- W36: the member call as a whole body -----------------------------------
#
# `p->m(a…);` is `m(p, a…)` on this ABI: the receiver is argument **zero**, in
# r3, and the port lowers the whole thing through the tail call it already had.
# So the one thing that can go wrong is the *slot arithmetic* — which register
# each value has to end up in once `this` occupies the first one — and the way
# to find a slot-arithmetic bug is to enumerate the slots rather than to add one
# more hand-picked case (`docs/GAPS.md` §6 instance #10: a rule right at cycle
# length 2 and 3 and wrong at 4 was found by the complete grid, not by a probe).
#
# The axes here are the ones a hand-written corpus does not vary, and each of
# them is a place where "the receiver is just another argument" could be false:
#
#  * **the COMPLETE permutation grid at each arity.** Every permutation of the
#    explicit arguments crossed with every position of the receiver in the
#    caller's own formals list. `this` makes each cycle one element longer than
#    the equivalent free-function call, so a call the free grid graded as a
#    3-cycle is a 4-cycle here — which is exactly the region `permute_args_text`
#    is measured WRONG in, and the gate has to move with the receiver.
#  * **the caller being a member function itself**, so `this` occupies r3 and
#    the receiver is some *other* pointer. Every fixture that has ever exercised
#    a member call had the receiver as formal 0, where the argument's index and
#    its register are the same number — the single most repeated defect in this
#    project (`GAPS.md` §6 instances #4, #5, #6, #8).
#  * **cv-qualification of the receiver, the pointer and the method**, crossed.
#    It changes no operator and no shape and it does change the TYPE tags the
#    operand gate and the `99` bind read; it is the axis §6's thirteenth live
#    mis-emit hid behind.
#  * **the callee's RETURN type**, discarded and returned, across every width
#    and class. The value is in the right register either way, so a rule that
#    keyed the tail branch on the result type would agree on `void` and `int`
#    (which is every fixture) and disagree on `char`, `long long`, `float` and a
#    struct — where c1xx stops spelling the production at all.
#  * **the receiver appearing again in the argument list** (`o->m(o)`), which is
#    a repeated leaf across the `this` slot rather than inside one argument.
#  * **an argument list that is a strict subset of the caller's formals**, which
#    is instance #5's panic (`call-arg-outer-formal`) with the receiver added.
#  * **the callee's own arity up to nine argument registers**, the boundary
#    where `this` pushes the last explicit argument off the register file.


def cases(emit):
    HDR = ("struct Obj { int i;\n"
           "  void v0(); void v1(int); void v2(int,int); void v3(int,int,int);\n"
           "  void v4(int,int,int,int); void v5(int,int,int,int,int);\n"
           "  void v6(int,int,int,int,int,int); void v7(int,int,int,int,int,int,int);\n"
           "  void v8(int,int,int,int,int,int,int,int);\n"
           "  void vp(Obj*); void vpp(Obj*,Obj*);\n"
           "  int g0() const; int g1(int) const; Obj* gp();\n"
           "};\n")

    def perms(xs):
        if not xs:
            yield ()
            return
        for i, x in enumerate(xs):
            for rest in perms(xs[:i] + xs[i + 1:]):
                yield (x,) + rest

    NAMES = "abcde"

    # ---- the complete permutation grid, receiver at every formal position ----
    # arity `n` explicit arguments, all `n!` orders, and the receiver placed at
    # each of the `n+1` positions in the caller's formals. `this` occupies the
    # slot the first formal would have, so the resulting permutation is over
    # `n+1` registers and the gate (single cycle, length <= 3) has to be read
    # over that, not over `n`.
    for n in (1, 2, 3):
        args = NAMES[:n]
        for order in perms(tuple(args)):
            for recv_at in range(n + 1):
                formals = ["int %s" % c for c in args]
                formals.insert(recv_at, "Obj* o")
                emit("%svoid f(%s) { o->v%d(%s); }\n"
                     % (HDR, ", ".join(formals), n, ", ".join(order)))

    # ---- the caller is a MEMBER function: `this` in r3, receiver elsewhere ---
    # The receiver's index in the formals list and its argument register differ
    # by one here and by zero everywhere else, which is the distinction every
    # previous instance of this defect turned on.
    for n in (0, 1, 2, 3):
        args = NAMES[:n]
        for order in perms(tuple(args)):
            for recv_at in range(n + 1):
                formals = ["int %s" % c for c in args]
                formals.insert(recv_at, "Obj* o")
                sig = ", ".join(formals)
                emit("%sstruct Host { int h; void m(%s); };\n"
                     "void Host::m(%s) { o->v%d(%s); }\n"
                     % (HDR, sig, sig, n, ", ".join(order)))

    # ---- the receiver is `this` itself --------------------------------------
    for n in (0, 1, 2, 3):
        args = NAMES[:n]
        for order in perms(tuple(args)):
            decl = ", ".join("int %s" % c for c in args)
            emit("struct S { int s;\n"
                 "  void t0(); void t1(int); void t2(int,int); void t3(int,int,int);\n"
                 "  void go(%s);\n"
                 "};\n"
                 "void S::go(%s) { t%d(%s); }\n"
                 % (decl, decl, n, ", ".join(order)))

    # ---- cv-qualification, crossed ------------------------------------------
    # The pointee's cv, the pointer's own cv and the method's cv are three
    # independent spellings that each move a TYPE tag and no instruction.
    for ptee in ("", "const "):
        for ptr in ("", "const "):
            for meth, call in (("", "v0"), (" const", "c0")):
                emit("struct Obj { int i; void v0(); void c0()%s; };\n"
                     "void f(%sObj* %so) { o->%s(); }\n"
                     % (meth, ptee, ptr, call))
    # …and on a member-function caller, where the receiver is not r3.
    for ptee in ("", "const "):
        for ptr in ("", "const "):
            emit("struct Obj { int i; void v0(); };\n"
                 "struct Host { int h; void m(int k, %sObj* %so); };\n"
                 "void Host::m(int k, %sObj* %so) { o->v0(); }\n"
                 % (ptee, ptr, ptee, ptr))

    # ---- the callee's RETURN type, discarded and returned --------------------
    RETS = ("void", "int", "unsigned", "long", "char", "signed char",
            "unsigned char", "short", "unsigned short", "long long", "bool",
            "float", "double", "int*", "const char*")
    for ty in RETS:
        emit("struct Obj { int i; %s r(); };\nvoid f(Obj* o) { o->r(); }\n" % ty)
        if ty != "void":
            emit("struct Obj { int i; %s r(); };\n%s f(Obj* o) { return o->r(); }\n"
                 % (ty, ty))
    # a struct returned BY VALUE: c1xx spells it with a `9B` temporary and a
    # hidden buffer pointer in r3, which is the one shape where "the receiver is
    # argument zero" would be false.
    emit("struct Val { int a, b; };\n"
         "struct Obj { int i; Val r(); };\n"
         "void f(Obj* o) { o->r(); }\n")
    emit("struct Val { int a, b; };\n"
         "struct Obj { int i; Val r(); };\n"
         "Val f(Obj* o) { return o->r(); }\n")

    # ---- POINTER arguments beside the pointer receiver ----------------------
    # Two and three pointers over the same register file, every order — the
    # receiver is not distinguishable from them by type, only by position.
    for order in perms(("o", "q")):
        emit("struct Obj { int i; void vp(Obj*); void vpp(Obj*,Obj*); };\n"
             "void f(Obj* o, Obj* q) { %s->vp(%s); }\n" % order)
    for order in perms(("o", "q", "r")):
        emit("struct Obj { int i; void vpp(Obj*,Obj*); };\n"
             "void f(Obj* o, Obj* q, Obj* r) { %s->vpp(%s, %s); }\n" % order)

    # ---- the receiver passed again as an argument ---------------------------
    emit("struct Obj { int i; void vp(Obj*); };\n"
         "void f(Obj* o) { o->vp(o); }\n")
    emit("struct Obj { int i; void vpp(Obj*,Obj*); };\n"
         "void f(Obj* o, Obj* q) { o->vpp(q, o); }\n")

    # ---- a STRICT SUBSET of the caller's formals ----------------------------
    # `arg_sources` indexes the formals while the permutation walk treats it as
    # a permutation of the argument slots; the two lists are the same length
    # only when the call passes every formal. That mismatch was a panic
    # (`GAPS.md` §6 instance #5) and the receiver adds a slot to one side of it.
    for n in (2, 3, 4):
        args = NAMES[:n]
        for skip in range(n):
            passed = [c for k, c in enumerate(args) if k != skip]
            emit("%svoid f(Obj* o, %s) { o->v%d(%s); }\n"
                 % (HDR, ", ".join("int %s" % c for c in args),
                    n - 1, ", ".join(passed)))

    # ---- the argument-register boundary -------------------------------------
    # `this` plus n explicit arguments; at n = 8 the ninth value is stack-homed
    # and the setup is a store, not a move.
    for n in (4, 5, 6, 7, 8):
        args = ["a%d" % k for k in range(n)]
        emit("%svoid f(Obj* o, %s) { o->v%d(%s); }\n"
             % (HDR, ", ".join("int %s" % c for c in args), n, ", ".join(args)))

    # ---- literal and computed arguments -------------------------------------
    # A literal is not a bare formal LOAD, so the multi-argument path refuses it;
    # the sweep is what says the refusal is a refusal and not a wrong emit.
    for expr in ("7", "0", "-1", "k", "k + 1", "k * 4", "k - 3"):
        emit("struct Obj { int i; void v1(int); int g1(int) const; };\n"
             "void f(Obj* o, int k) { o->v1(%s); }\n" % expr)
        emit("struct Obj { int i; void v1(int); int g1(int) const; };\n"
             "int f(Obj* o, int k) { return o->g1(%s); }\n" % expr)

    # ---- brace scopes around the statement ----------------------------------
    # The inner close sits between the `4B` and the return branch, which no
    # other member-call case reaches.
    emit("struct Obj { int i; void v0(); };\n"
         "void f(Obj* o) { { o->v0(); } }\n")
    emit("struct Obj { int i; void v0(); };\n"
         "void f(Obj* o) { { { o->v0(); } } }\n")
    emit("struct Obj { int i; int g0() const; };\n"
         "int f(Obj* o) { { return o->g0(); } }\n")

    # ---- receivers this rung must REFUSE, beside the ones it takes -----------
    # Each is a different receiver production; the sweep grades that they emit
    # nothing rather than something wrong.
    emit("struct Obj { int i; void v0(); };\n"
         "struct W { int w; Obj* o; Obj em; };\n"
         "void f(W* w) { w->o->v0(); }\n")
    emit("struct Obj { int i; void v0(); };\n"
         "struct W { int w; Obj* o; Obj em; };\n"
         "void f(W* w) { w->em.v0(); }\n")
    emit("struct Obj { int i; void v0(); Obj* nxt(); };\n"
         "void f(Obj* o) { o->nxt()->v0(); }\n")
    emit("struct Obj { int i; void v0(); };\nextern Obj g;\n"
         "void f() { g.v0(); }\n")
    emit("struct B { void bm(); };\nstruct D : B { int d; };\n"
         "void f(D* d) { d->bm(); }\n")
    emit("struct V { virtual void vf(); };\n"
         "void f(V* v) { v->vf(); }\n")
