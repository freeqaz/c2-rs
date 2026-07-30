# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# `cases(emit)` is called once by the driver; `emit(src)` writes one .cpp case,
# named and counted under THIS fragment's own namespace. A fragment cannot see
# or touch another fragment's counter (the `n`-shadowing trap that silently
# overwrote 1,233 already-written cases is unrepresentable here), and the driver
# fails if a fragment emits zero cases.


def cases(emit):
    # ---- W19: the CONSTRUCTOR EPILOGUE, `return this` after the RETURN ---------------
    # The cross product the hand-written fixture cannot be: every member layout against
    # every parameter list. A constructor's body carries a value expression between the
    # `29` RETURN and the function tail, and the claim this rung makes is that it costs
    # no instruction at all — so these must grade `Match`, not merely refuse. Loop
    # variables are named for what they hold; `n` is the generator's own file counter
    # and rebinding it silently overwrites already-written cases (docs/GAPS.md §6).
    ctor_members = [
        '',
        'int m;',
        'int m, n2;',
        'double d;',
        'float f2;',
        'int arr[4];',
        'char c;',
        'void *vp; const char *cp;',
        'long long ll;',
    ]
    ctor_params = [
        '',
        'int a',
        'int a, int b',
        'int a, int b, int c, int d',
        'float x',
        'double x',
        'const char *s',
        'int a, float x',
        'float x, int a',
        'int a, int b, int c, int d, int e, int g2, int h, int i',
    ]
    for mi, mem in enumerate(ctor_members):
        for pi, plist in enumerate(ctor_params):
            emit("struct S%d_%d { %s S%d_%d(%s); };\nS%d_%d::S%d_%d(%s) {}\n"
                     % (mi, pi, mem, mi, pi, plist, mi, pi, mi, pi, plist))
    # The copy constructor, an 8-byte by-value aggregate parameter, and the shapes a
    # per-class loop cannot reach.
    for extra in (
        "struct CpA { int m; CpA(); CpA(const CpA &); };\nCpA::CpA() {}\nCpA::CpA(const CpA &o) {}\n",
        "struct CpB { double d; CpB(); CpB(const CpB &); };\nCpB::CpB() {}\nCpB::CpB(const CpB &o) {}\n",
        "struct PairS { int x, y; };\nstruct AgA { int m; AgA(PairS); };\nAgA::AgA(PairS v) {}\n",
        "struct PairS { int x, y; };\nstruct AgB { int m; AgB(PairS, int); };\nAgB::AgB(PairS v, int b) {}\n",
        "struct RefA { int m; RefA(int &); };\nRefA::RefA(int &r) {}\n",
        "struct PtrA { int m; PtrA(int *, int *); };\nPtrA::PtrA(int *p, int *q) {}\n",
        # several byte-identical bodies in ONE translation unit: the locality tell
        "struct L1 { int m; L1(); };\nstruct L2 { int m; L2(); };\nstruct L3 { int m; L3(); };\n"
        "L1::L1() {}\nL2::L2() {}\nL3::L3() {}\n",
        # the same, interleaved with the empty bodies that have NO epilogue
        "struct L4 { int m; L4(); };\nvoid e1() {}\nL4::L4() {}\nvoid e2() {}\n",
        # a nested class and a class in a namespace: the mangled name changes, the body
        # does not
        "struct Out { struct In { int m; In(); }; };\nOut::In::In() {}\n",
        "namespace ns { struct NsA { int m; NsA(); }; }\nns::NsA::NsA() {}\n",
        # a destructor with an empty body: the control, it has no epilogue
        "struct DtA { int m; ~DtA(); };\nDtA::~DtA() {}\n",
        # an empty member function and an empty static member: the other controls
        "struct MfA { int m; void v() const; static void s(); };\nvoid MfA::v() const {}\n"
        "void MfA::s() {}\n",
    ) + tuple(
        # every argument slot r4..r10 filled ahead of nothing being read, at each arity
        "struct Ar%d { int m; Ar%d(%s); };\nAr%d::Ar%d(%s) {}\n"
        % (k, k, ', '.join('int a%d' % j for j in range(k)),
           k, k, ', '.join('int a%d' % j for j in range(k)))
        for k in range(1, 9)
    ):
        emit(extra)
    # The refusing NEIGHBOURS of this gate. A call spills `this` to a nonvolatile and
    # restores it (`mr r31,r3` … `mr r3,r31`), which is the frame axis; a store through
    # `this` is the `27` designator. Both must stay `NotImplemented`, and a MISMATCH
    # here is the alarm this block exists to raise.
    for neighbour in (
        "struct NB { int b; NB(); };\nstruct ND : NB { ND(); };\nND::ND() {}\n",
        "void sfx();\nstruct NC { int m; NC(); };\nNC::NC() { sfx(); }\n",
        "struct NM { int m; NM(); };\nstruct NH { NM sub; NH(); };\nNH::NH() {}\n",
        "struct NS1 { int m; NS1(int); };\nNS1::NS1(int a) { m = a; }\n",
        "struct NS2 { int m; NS2(int); };\nNS2::NS2(int a) : m(a) {}\n",
        "struct NS3 { int m, n2; NS3(int, int); };\nNS3::NS3(int a, int b) : m(a), n2(b) {}\n",
        "struct NV { int m; virtual void f(); NV(); };\nNV::NV() {}\n",
        "struct NVi { int m; virtual void f(); NVi(); };\nvoid NVi::f() {}\nNVi::NVi() {}\n",
        # a returned object that is NOT `this`: the epilogue names another token
        "struct NRv { int m; NRv(); };\nNRv mk();\nNRv mk() { NRv v; return v; }\n",
        # a constructor that returns early: the body is not empty, it branches
        "struct NBr { int m; NBr(int); };\nNBr::NBr(int a) { if (a) return; }\n",
        # virtual inheritance: the epilogue is there, the body installs a vbtable
        "struct VB { int b; VB(); };\nstruct VD : virtual VB { VD(); };\nVD::VD() {}\n",
    ):
        emit(neighbour)
