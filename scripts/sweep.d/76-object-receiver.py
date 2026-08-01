# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the driver; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace. A fragment cannot see
# or touch another fragment's counter, and the driver fails if a fragment emits
# zero cases.


def cases(emit):
    # ---- W-ADJUST: a NAMED DATA OBJECT standing as a member call's receiver ----------
    # Board #128, `docs/rungs/2026-08-01-w-adjust.md`.
    #
    # `gObj.m(a);` — the receiver designator is a data symbol's *address*, so the body
    # is W36's member call composed with WR1's relocation quad. Nothing new is emitted;
    # what is new is the composition, and that is precisely the class of change a
    # hand-written fixture grades badly: `fixtures/cpp/wadjust_obj_recv.cpp` puts the
    # symbol at slot 0 in every single case, because the receiver IS argument zero on
    # this ABI, so it cannot discriminate any rule that depends on the address's slot.
    #
    # The axes below are the ones the emitter branches on:
    #
    #   * **whether the caller's formals are already in their slots** — a member caller
    #     drops `this`, so formal `i` lands at slot `i`; a free caller shifts by one and
    #     is a permutation past a hoisted `lis`, which WR1 refuses. Both sides are swept
    #     so the boundary is graded, not assumed;
    #   * **how many words ride beside the address** (0 through 4), the count WR1's own
    #     sweep never varied and whose two-word cell was mis-emitted;
    #   * **the argument type** — `int` and `const char*`, since the census key the row
    #     was ranked under is `-then-type-ptr` and the pointer is what it names;
    #   * **the result** — discarded and returned;
    #   * **the object's linkage** — extern (in class), defined here and static (out of
    #     class, a section in the middle of the table);
    #   * **an offset off the object** (`gPair.b.m()`), which is a third instruction and
    #     must keep refusing;
    #   * **the object's mangled name length across the COFF 8-byte inline boundary**;
    #   * **the same object from several functions in one TU** — one undefined external,
    #     however many bodies reference it.
    DECLS = (
        "struct Dbg {\n"
        "  void nul(); void puti(int); void putp(const char*);\n"
        "  void two(int,int); void three(int,int,int); void four(int,int,int,int);\n"
        "  int get(); int getk(int); Dbg* self(); void put2(int*);\n"
        "};\n"
        "struct Pair { int a; Dbg b; };\n"
        "extern int gI;\n"
    )
    EXTERN = (
        "extern Dbg gDbg;\nextern Dbg gObjectWithAVeryLongMangledName;\n"
        "extern Pair gPair;\n"
    )
    DEFINED = (
        "Dbg gDbg;\nDbg gObjectWithAVeryLongMangledName;\nPair gPair;\n"
    )
    STATIC = (
        "static Dbg gDbg;\nstatic Dbg gObjectWithAVeryLongMangledName;\n"
        "static Pair gPair;\nDbg* keep(){ return &gDbg; }\n"
    )

    # ---- the literal walk beside the address, 0..4 words ----------------------------
    for call in (
        "gDbg.nul()",
        "gDbg.puti(7)",
        "gDbg.two(3, 4)",
        "gDbg.three(3, 4, 5)",
        "gDbg.four(3, 4, 5, 6)",
        "gDbg.puti(-1)",
        "gDbg.puti(32767)",
        "gDbg.two(-32768, 32767)",
    ):
        for linkage in (EXTERN, DEFINED, STATIC):
            emit(DECLS + linkage + f"void f(){{ {call}; }}\n")

    # ---- the caller's kind: member (formals in place) against free (a permutation) ---
    MEMBER = (
        "struct Fwd {\n"
        "  void f0(); void f1(int); void f2(int,int); void f3(int,int,int);\n"
        "  void p1(const char*); int r0(); int r1(int);\n"
        "  void m1(int); void m2(int,int);\n"
        "};\n"
    )
    for sig, body in (
        ("void Fwd::f0()", "gDbg.nul()"),
        ("void Fwd::f1(int a)", "gDbg.puti(a)"),
        ("void Fwd::f2(int a, int b)", "gDbg.two(a, b)"),
        ("void Fwd::f3(int a, int b, int c)", "gDbg.three(a, b, c)"),
        ("void Fwd::p1(const char* s)", "gDbg.putp(s)"),
        ("int Fwd::r0()", "return gDbg.get()"),
        ("int Fwd::r1(int k)", "return gDbg.getk(k)"),
        ("void Fwd::m1(int a)", "gDbg.two(a, 7)"),
        ("void Fwd::m2(int a, int b)", "gDbg.three(a, b, 7)"),
    ):
        emit(DECLS + EXTERN + MEMBER + f"{sig} {{ {body}; }}\n")

    # …and the free-caller twins, where the same argument has to MOVE. These must keep
    # refusing (`call-arg-sym-permuted`), and the pair is what says the discriminator is
    # the SLOT and not the caller's kind.
    for sig, body in (
        ("void f(int a)", "gDbg.puti(a)"),
        ("void f(int a, int b)", "gDbg.two(a, b)"),
        ("void f(const char* s)", "gDbg.putp(s)"),
        ("int f(int k)", "return gDbg.getk(k)"),
        # …the free callers whose formal index ALREADY equals its slot: in class.
        ("void f(const char*, int b)", "gDbg.puti(b)"),
        ("void f(int, int b, int c)", "gDbg.two(b, c)"),
        ("void f(int, int b)", "gDbg.two(b, 7)"),
    ):
        emit(DECLS + EXTERN + f"{sig} {{ {body}; }}\n")

    # ---- the pointer argument, which is what the census key names -------------------
    for body in (
        'void f(){ gDbg.putp(0); }',
        'void f(const char*, const char* t){ gDbg.putp(t); }',
        'int  f(){ return gDbg.get(); }',
        'int  f(){ return gDbg.getk(7); }',
    ):
        emit(DECLS + EXTERN + body + "\n")

    # ---- the long mangled name, and one object from several bodies -------------------
    emit(DECLS + EXTERN + "void f(){ gObjectWithAVeryLongMangledName.nul(); }\n")
    emit(
        DECLS
        + EXTERN
        + "void f(){ gDbg.nul(); }\nvoid g(){ gDbg.nul(); }\nvoid h(){ gDbg.puti(7); }\n"
    )
    emit(
        DECLS
        + EXTERN
        + "void f(){ gDbg.nul(); }\nvoid g(){ gObjectWithAVeryLongMangledName.nul(); }\n"
    )

    # ---- the neighbours that must keep refusing --------------------------------------
    for body in (
        # an offset off the object: a third instruction off the scratch
        "void f(){ gPair.b.nul(); }",
        "void f(){ gPair.b.puti(7); }",
        # a second symbol in the same call
        "void f(){ gDbg.put2(&gI); }",
        # a chain through the object: two `bl`s, a value live across the first
        "void f(){ gDbg.self()->nul(); }",
        "void f(){ gDbg.self()->puti(7); }",
        # the result consumed by a literal post-op: a framed call with an ADDRESS
        # receiver, which `framed_member_call` has no capture for
        "int f(){ return gDbg.get() + 1; }",
        "int f(){ return gDbg.getk(2) - 20; }",
        # a second statement after the call
        "void f(){ gDbg.nul(); gDbg.puti(1); }",
        # the object's address taken, rather than a method called on it
        "extern void gz(Dbg*);\nvoid f(){ gz(&gDbg); }",
        # a pointer to the object, dereferenced — the ordinary receiver, not this one
        "extern Dbg* gPtr;\nvoid f(){ gPtr->nul(); }",
    ):
        emit(DECLS + EXTERN + body + "\n")
