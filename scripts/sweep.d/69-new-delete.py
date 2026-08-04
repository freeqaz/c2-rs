# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# **`new` AND `delete` — the compiler-generated deleting destructors** (lane
# w-shapes; `scripts/sweep_shapes.py` row `new / delete`, ZERO before this file).
#
# Board **#232** was an *implicit* destructor crossed with the packed `.text`
# path: a function that appears in no source line, that the generator had never
# written, and that survived 255 commits as a live wrong emit. `63-emit-order.py`
# closed the implicit-sibling-destructor case of it. **`new` and `delete` are the
# same family one level up**: they mint `??_G` (scalar deleting destructor),
# `??_E` (vector deleting destructor) and the `__ehvec_*` helpers, none of which
# is written anywhere in the source and each of which gets its own COMDAT.
#
# MEASURED (`scripts/gt_capture.sh /Ox /GS- /c`, non-boilerplate sections):
#
#     struct S{S();~S();int a;};  S* f(){return new S;}
#         1147 B   .text .pdata
#     struct S{S();~S();int a;};  S* f(){return new S[4];}
#         2576 B   .text .pdata .text .pdata .text .pdata .text .pdata
#     struct S{S();~S();int a;};  void f(S*p){delete p;}
#         1499 B   .text .pdata .text .pdata
#     struct S{S();~S();int a;};  void f(S*p){delete[] p;}
#         2138 B   .text .pdata .text .pdata .text .pdata
#     struct S{S();virtual ~S();int a;};  void f(S*p){delete p;}
#          882 B   .text                       <- ONE function, no helper at all
#
# **Four COMDATs from a five-token expression, and the virtual case has none.**
# The last line is the row that makes this a structural axis rather than a
# vocabulary one: a `delete` through a virtual destructor is an indirect call and
# generates nothing, while the identical source with `virtual` removed generates
# two extra functions. Any rule keyed on "the TU says `delete`" is wrong on one
# of those two, and nothing in the corpus contains either.
#
# ---- the axes ------------------------------------------------------------------
#
#   A. ALLOCATION FORM — `new T` | `new T(args)` | `new T[4]` | `new T[n]` |
#      none. The array forms are where `??_E` and `__ehvec_ctor` come from, and
#      the runtime-length form differs from the constant-length one.
#   B. TYPE — `int` | a POD struct | a class with a constructor | a class with a
#      destructor | both | a virtual destructor | a base with a virtual
#      destructor | a class with a member that has a destructor. This axis picks
#      HOW MANY generated functions exist, from zero to four.
#   C. DEALLOCATION FORM — none | `delete p` | `delete[] p` | both in one TU |
#      `delete` of a base pointer. The scalar and vector deleting destructors are
#      distinct symbols and a TU can want one, the other, or both.
#   D. CLASS-SCOPE `operator new` / `operator delete` — declared in the class, and
#      declared at namespace scope. The call target changes name and the
#      generated helpers do not.
#   E. TU SHAPE — the allocating function alone, before, after and between
#      functions the port matches. `63-emit-order.py` established the `.text`
#      order is a dependency walk; the generated helpers are nodes in that walk
#      that no source line names.
#   F. THE CONTROLS — the identical TUs with `new`/`delete` replaced by a call to
#      an ordinary allocator, and with the destructor removed. Several are the
#      port's own accepted class, so a fix that refused any TU mentioning a
#      destructor would lose them.
#
# Every class here is declared and not defined: the sweep compiles `/c` only.
# `NotImplemented` is the contract; a `Port=Mismatch` means the writer emitted an
# obj without placing the helpers c2 generated.

LEAF = "int %s(int a){return a+%d;}\n"

# (tag, class declaration, how many generated helpers it can imply)
CLASSES = (
    ("pod",     "struct S{int a;};\n"),
    ("ctor",    "struct S{S();int a;};\n"),
    ("ctorarg", "struct S{S();S(int);int a;};\n"),
    ("dtor",    "struct S{~S();int a;};\n"),
    ("both",    "struct S{S();~S();int a;};\n"),
    ("vdtor",   "struct S{S();virtual ~S();int a;};\n"),
    ("vbase",   "struct B{B();virtual ~B();int b;};\nstruct S:B{S();~S();int a;};\n"),
    ("member",  "struct M{M();~M();int m;};\nstruct S{S();M m;int a;};\n"),
    ("nested",  "struct M{M();~M();int m;};\nstruct S{S();~S();M m;int a;};\n"),
)


def cases(emit):
    # ---- A x B: every allocation form x every class --------------------------
    for _tag, decl in CLASSES:
        emit(decl + "S* f(){ return new S; }\n")
        emit(decl + "S* f(int n){ return new S[4]; }\n")
        emit(decl + "S* f(int n){ return new S[n]; }\n")
        emit(decl + "S* f(int n){ return new S[n+1]; }\n")
        # two allocations in one function — one set of helpers, not two.
        emit(decl + "S* f(int n){ if(n) return new S; return new S[4]; }\n")
        # …and in two functions.
        emit(decl + "S* f(){ return new S; }\nS* g(int n){ return new S[n]; }\n")
    emit("struct S{S();S(int);int a;};\nS* f(int n){ return new S(n); }\n")
    emit("struct S{S();S(int);int a;};\nS* f(int n){ return new S(1); }\n")
    emit("int* f(){ return new int; }\n")
    emit("int* f(int n){ return new int[n]; }\n")
    emit("int* f(){ return new int[4]; }\n")
    emit("int* f(int n){ return new int(n); }\n")
    emit("double* f(){ return new double; }\n")
    emit("char* f(int n){ return new char[n]; }\n")

    # ---- C: DEALLOCATION — scalar, vector, both, and through a base ----------
    for _tag, decl in CLASSES:
        emit(decl + "void f(S* p){ delete p; }\n")
        emit(decl + "void f(S* p){ delete[] p; }\n")
        emit(decl + "void f(S* p, S* q){ delete p; delete[] q; }\n")
        emit(decl + "void f(S* p, S* q){ delete[] q; delete p; }\n")
        # allocate AND free in one TU: the scalar deleting destructor is wanted
        # by `delete` and the constructor by `new`, and they are different
        # generated functions.
        emit(decl + "S* f(){ return new S; }\nvoid g(S* p){ delete p; }\n")
        emit(decl + "void g(S* p){ delete p; }\nS* f(){ return new S; }\n")
    emit("void f(int* p){ delete p; }\n")
    emit("void f(int* p){ delete[] p; }\n")
    emit("struct B{B();virtual ~B();int b;};\nstruct S:B{S();~S();int a;};\n"
         "void f(B* p){ delete p; }\n")
    emit("struct B{B();~B();int b;};\nstruct S:B{S();~S();int a;};\n"
         "void f(B* p){ delete p; }\n")
    emit("struct B{B();~B();int b;};\nstruct S:B{S();~S();int a;};\n"
         "void f(S* p){ delete p; }\n")
    # `delete` of a null-checked pointer and of an expression, so the shape is
    # not keyed on the operand being a bare parameter.
    emit("struct S{S();~S();int a;};\nvoid f(S* p){ if(p) delete p; }\n")
    emit("struct S{S();~S();int a;};\nstruct H{S* p;};\nvoid f(H* h){ delete h->p; }\n")

    # ---- D: CLASS-SCOPE AND NAMESPACE-SCOPE `operator new` -------------------
    #
    # The allocator's SYMBOL changes; the generated helpers do not. Declaring
    # `operator delete` in the class is what makes the scalar deleting destructor
    # call a member rather than `??3@YAXPAX@Z`.
    for src in (
        "struct S{ void* operator new(unsigned int); void operator delete(void*);"
        " S(); ~S(); int a; };\nS* f(){ return new S; }\n",
        "struct S{ void* operator new(unsigned int); void operator delete(void*);"
        " S(); ~S(); int a; };\nvoid f(S* p){ delete p; }\n",
        "struct S{ void* operator new[](unsigned int); void operator delete[](void*);"
        " S(); ~S(); int a; };\nS* f(int n){ return new S[n]; }\n",
        "void* operator new(unsigned int);\nvoid operator delete(void*);\n"
        "struct S{S();~S();int a;};\nS* f(){ return new S; }\n",
        "void* operator new(unsigned int, void*);\n"
        "struct S{S();int a;};\nS* f(void* m){ return new (m) S; }\n",
        "void* operator new(unsigned int, int);\n"
        "struct S{S();~S();int a;};\nS* f(int n){ return new (n) S; }\n",
    ):
        emit(src)

    # ---- E: TU SHAPE — the allocator among functions the port matches --------
    NEW = "struct S{S();~S();int a;};\nS* f(){ return new S; }\n"
    DEL = "struct S{S();~S();int a;};\nvoid f(S* p){ delete p; }\n"
    for src in (NEW, DEL):
        head, tail = src.split("\n", 1)
        emit(head + "\n" + tail + LEAF % ("z", 1))
        emit(head + "\n" + LEAF % ("z", 1) + tail)
        emit(head + "\n" + LEAF % ("y", 2) + tail + LEAF % ("z", 1))
        emit(head + "\n" + tail + LEAF % ("z", 1) + LEAF % ("y", 2))
    # the allocating function CALLS a locally defined one — the dependency edge
    # `63-emit-order.py` grades, crossed with generated helpers in the same walk.
    emit("struct S{S();~S();int a;};\n" + LEAF % ("z", 1)
         + "S* f(int a){ z(a); return new S; }\n")
    emit("struct S{S();~S();int a;};\nint z(int);\n"
         "S* f(int a){ z(a); return new S; }\n" + LEAF % ("z", 1))

    # ---- F: THE CONTROLS ----------------------------------------------------
    #
    # The same TUs with the allocation replaced by an ordinary call, and with the
    # destructor removed. The `virtual ~S()` + `delete p` row above generates NO
    # helper at all and belongs on this side of the line as much as on the other.
    for src in (
        "struct S{S();~S();int a;};\nS* g();\nS* f(){ return g(); }\n",
        "struct S{S();~S();int a;};\nvoid g(S*);\nvoid f(S* p){ g(p); }\n",
        "struct S{int a;};\nS* g();\nS* f(){ return g(); }\n",
        "struct S{S();~S();int a;};\nint f(S* p){ return p->a + 1; }\n",
        "struct S{int a;};\nint f(S* p){ return p->a + 1; }\n",
        "int f(int a){ return a + 1; }\n",
        LEAF % ("z", 1) + "int f(int a){ return a + 1; }\n",
        "void* g(unsigned int);\nvoid* f(){ return g(4); }\n",
        # a destructor written by hand, out of line: the shape the corpus DOES
        # have (`50-dtor-base.py`), kept here so the generated ones are read
        # against it.
        "struct S{S();~S();int a;};\nS::~S(){}\n",
        "struct S{S();~S();int a;};\nS::S(){}\nS::~S(){}\n",
    ):
        emit(src)
