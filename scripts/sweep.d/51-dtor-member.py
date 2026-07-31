# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the driver; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace. A fragment cannot see
# or touch another fragment's counter (the `n`-shadowing trap that silently
# overwrote 1,233 already-written cases is unrepresentable here), and the driver
# fails if a fragment emits zero cases.


def cases(emit):
    # ---- generated empty destructors: the MEMBER sub-object form ---------------------
    # The second generated destructor (`docs/IL_CALL_IN_EXPR.md` §14.3, §15): no
    # destructible base, exactly one destructible **member**, receiver = `this + k`
    # through a plain `27` add with no class-layout intrinsic. It is one production with
    # the base form above and differs in one literal — the member's byte offset — which
    # is also the entire codegen difference: nothing at 0, one `addi r3,r3,k` otherwise.
    #
    # So the axis to sweep is the OFFSET, and it has to be swept against everything that
    # could move it independently: the padding that produces it, the member's own size
    # and alignment, cv-qualification on the member (which moves the receiver's TYPE tag
    # from `86` to `A6`), and the number of members (which moves `5E <n>` and turns the
    # whole lowering into a frame). A fixture set cannot separate "the offset is the
    # literal `k`" from "the offset is the member's index" or "…is always 4" without the
    # cross product, and c2's own switch from one `addi` to `addis`+`addi` at the signed
    # 16-bit edge is only visible if the sweep crosses it.
    MEMS = (('MemI', '~MemI(); int a;'),          # 4 bytes
            ('MemD', '~MemD(); double a;'),       # 8, aligned 8
            ('MemC', '~MemC(); char a;'),         # 1, aligned 1
            ('MemB', '~MemB(); char b[100];'),    # large, aligned 1
            ('MemE', '~MemE();'))                 # empty: 1 byte, still destructible
    # Leading padding, chosen to land the member at 0 and at a spread of offsets on both
    # sides of every alignment rule, plus both sides of the `addi` immediate boundary.
    PADS = ('', 'char p0;', 'char p0, p1;', 'int p0;', 'int p0, p1;', 'double p0;',
            'char p0[3];', 'char p0[7];', 'char p0[32760];', 'char p0[32764];',
            'char p0[32765];', 'char p0[32768];', 'char p0[40000];', 'char p0[65536];')
    for mn, mbody in MEMS:
        for pad in PADS:
            emit("struct %s { %s };\nstruct D { ~D(); %s %s m; };\nD::~D() {}\n"
                     % (mn, mbody, pad, mn))
            # The same member `const` and `volatile`: the receiver's TYPE tag picks up the
            # cv bits, and `ValueClass::Ptr4` admits four tag spellings on the claim that
            # they are all the same pointer. Only sweeping them tests that claim.
            for q in ('const', 'volatile'):
                emit("struct %s { %s };\nstruct D { ~D(); %s %s %s m; };\nD::~D() {}\n"
                         % (mn, mbody, pad, q, mn))
    # A NON-destructible base contributing only to the member's offset: the base's own
    # size is the offset, so this is the same rule reached a different way.
    for bdata in ('int b;', 'double b;', 'char b[3];', 'char b[32764];', 'char b[32768];'):
        emit("struct M{~M();int a;};\nstruct B{%s};\nstruct D:B{~D();M m;};\nD::~D(){}\n" % bdata)
    # A member sub-object that itself has a member sub-object: two generated destructors
    # in one TU, each destroying one thing, at independent offsets.
    for pad in ('', 'int p;', 'double p;'):
        emit("struct M{~M();int a;};\nstruct I{~I();%s M m;};\nstruct O{~O();%s I i;};\n"
                 "I::~I(){}\nO::~O(){}\n" % (pad, pad))
    # A member whose own destructor is VIRTUAL. Destroying a member sub-object of known
    # type is still DIRECT dispatch (`99`, not `67`/`9A`), so this must emit a bare
    # branch to `??1…@@UAA@XZ` — the licence to branch comes from the bind, not from the
    # callee, and that is exactly the kind of claim a sweep is for.
    for pad in ('', 'int p;'):
        emit("struct V{virtual ~V();int a;};\nstruct D{~D();%s V m;};\nD::~D(){}\n" % pad)
    # The definition's SOURCE LINE again, for this receiver: the closing brace's
    # `4F 01 <line>` marker lands inside the return plumbing, and line 70's marker is the
    # known-bad formals anchor `4F 01 46`.
    for line in range(64, 77):
        pad = '\n'.join('// pad %d' % k for k in range(1, line - 2))
        emit("struct M{~M();int a;};\nstruct D{~D();int q; M m;};\n" + pad
                 + "\nD::~D()\n{\n}\n")

    # The refusing neighbours. Each is one production or one payload byte from the
    # accepted shape and each costs instructions the bare branch does not emit, so a
    # MISMATCH here is the alarm and NotImplemented is the right answer.
    for src in (
        # Two bases: two calls, the second at a nonzero adjust, and `5E 02 21`.
        "struct M1{~M1();int a;};struct M2{~M2();int b;};\nstruct D:M1,M2{~D();};\nD::~D(){}\n",
        # A destructible MEMBER as well as a base — two calls again.
        "struct M1{~M1();int a;};struct M2{~M2();int b;};\nstruct D:M1{~D();M2 m;};\nD::~D(){}\n",
        # TWO destructible members. `5E 02`, two statements, and the reference emits a
        # FRAME: `or r31,r3,r3`, two `bl`s in REVERSE declaration order, `or r3,r31,r31`
        # between them, because `this` is live across the first call. These are the 574
        # bodies §14.3 measured as lost to the offset split, and they are lost for a real
        # reason — grammar-complete with both offsets, codegen-complete under neither.
        # Swept over the offset pair, because "the first member is at 0" is the one case
        # where a single-branch lowering would look plausible.
        "struct M1{~M1();int a;};struct M2{~M2();int b;};\nstruct D{~D();M1 m;M2 n;};\nD::~D(){}\n",
        "struct M1{~M1();int a;};struct M2{~M2();int b;};\nstruct D{~D();int q;M1 m;M2 n;};\nD::~D(){}\n",
        "struct M1{~M1();int a;};\nstruct D{~D();M1 m,n;};\nD::~D(){}\n",
        "struct M1{~M1();int a;};\nstruct D{~D();M1 m,n,o;};\nD::~D(){}\n",
        # An ARRAY of destructible members: a destruct LOOP plus the `??_I` helper, and it
        # blocks on a different opcode entirely (`5C` in an unexpected place).
        "struct M1{~M1();int a;};\nstruct D{~D();M1 m[2];};\nD::~D(){}\n",
        "struct M1{~M1();int a;};\nstruct D{~D();M1 m[3];};\nD::~D(){}\n",
        "struct M1{~M1();int a;};\nstruct D{~D();int q;M1 m[3];};\nD::~D(){}\n",
        # A member with NO destructor: nothing to destroy, so the body is empty.
        "struct M1{int a;};\nstruct D{~D();M1 m;};\nD::~D(){}\n",
        "struct M1{int a;};\nstruct D{~D();int q;M1 m;};\nD::~D(){}\n",
        # A member POINTER and a member REFERENCE to a destructible type: neither is a
        # sub-object, so neither is destroyed.
        "struct M1{~M1();int a;};\nstruct D{~D();M1* m;};\nD::~D(){}\n",
        "struct M1{~M1();int a;};\nstruct D{~D();M1& m;D(M1&);};\nD::~D(){}\n",
        # The member's destructor DEFINED in this TU: c2 may inline it rather than branch.
        "struct M1{~M1(){}int a;};\nstruct D{~D();M1 m;};\nD::~D(){}\n",
        "struct M1{~M1(){}int a;};\nstruct D{~D();int q;M1 m;};\nD::~D(){}\n",
        # A destructible member and a real statement in the body: two calls.
        "void h();\nstruct M1{~M1();int a;};\nstruct D{~D();int q;M1 m;};\nD::~D(){h();}\n",
        # A VIRTUAL destructor on the enclosing class: `??_E`/`??_G` thunks appear and the
        # body is no longer the only function emitted.
        "struct M1{~M1();int a;};\nstruct D{virtual ~D();int q;M1 m;};\nD::~D(){}\n",
        # The member sits inside a VIRTUAL base: intrinsic 2116 through a vbtable.
        "struct M1{~M1();int a;};\nstruct V{~V();M1 m;};\nstruct D:virtual V{~D();};\nD::~D(){}\n",
        # A destructible member of a TEMPLATE class, and a template member: the `.ex`
        # segment of an instantiation ends `47 54 01 54 00 4D`, which every shape refuses
        # on the module framing alone (`docs/IL_CALL_IN_EXPR.md` §13.5).
        "struct M1{~M1();int a;};\ntemplate<class T> struct D{~D();T q;M1 m;};\n"
        "template<class T> D<T>::~D(){}\ntemplate struct D<int>;\n",
        # A CONSTRUCTOR of the same class: same `0x0100` optimization-word bit, and it
        # calls the member's constructor rather than its destructor.
        "struct M1{M1();~M1();int a;};\nstruct D{D();~D();int q;M1 m;};\nD::D(){}\n",
        # A real statement in the body.
        "void h();\nstruct M1{~M1();int a;};\nstruct D:M1{~D();};\nD::~D(){h();}\n",
        # A VIRTUAL destructor: opcode `67`/`9A` dispatch, plus the `??_E`/`??_G` thunks.
        "struct M1{virtual ~M1();int a;};\nstruct D:M1{virtual ~D();};\nD::~D(){}\n",
        # A VIRTUAL base: intrinsic 2116 through a vbtable, not 2113.
        "struct V{~V();int v;};\nstruct D:virtual V{~D();};\nD::~D(){}\n",
        # The base destructor DEFINED in this TU: c2 may inline it rather than branch.
        "struct M1{~M1(){}int a;};\nstruct D:M1{~D();};\nD::~D(){}\n",
        # A base with NO destructor: nothing to delegate to.
        "struct M1{int a;};\nstruct D:M1{~D();};\nD::~D(){}\n",
        # A destructor with nothing at all to destroy: `EmptyBody`, a bare `blr`.
        "struct D{~D();int a;};\nD::~D(){}\n",
        # CONSTRUCTORS. They carry the same `0x0100` optimization-word bit that this rung
        # started masking off, so admitting that bit put them in front of the emitter too.
        "struct M1{M1();int a;};\nstruct D:M1{D();};\nD::D(){}\n",
        "struct D{D();int a;};\nD::D(){}\n",
        "struct D{D(int);int a;};\nD::D(int v){a=v;}\n",
        "struct D{D(const D&);int a;};\nD::D(const D& o){a=o.a;}\n",
        "struct M1{M1();int a;};\nstruct M2{M2();int b;};\nstruct D:M1,M2{D();};\nD::D(){}\n",
    ):
        emit(src)
