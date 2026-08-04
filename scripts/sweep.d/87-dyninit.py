# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# **THE `??__E` DYNAMIC INITIALIZER** (lane w-shapes; obj-level row —
# `scripts/sweep_shapes.py --objs` reports `.CRT$XCU` and `.text$yc` among the
# section names *the workload has and the corpus cannot produce*).
#
# This is not one of `sweep_shapes.py`'s 56 SOURCE markers; it is one of the
# obj-level ones, and it is the one with the most direct claim on the payoff
# metric. `docs/STATUS.md`: TU match had been flat at 6 for the project's entire
# history and moved to 8 on 2026-08-04, by a **whole-TU `??__E` dynamic-initializer
# recognizer** (`IlBundle::dyninit_tu`) — not by widening the per-function class.
# The census is +0 across that change, so **the per-function census structurally
# cannot see this emitter**, and neither could any generated case: the corpus has
# never contained a dynamically initialized namespace-scope object.
#
# So the only live whole-TU emitter in the port is graded by exactly two files of
# the dc3 workload (`TomCryptLicense.cpp`, `ZlibLicense.cpp`) and by nothing
# enumerated. `c2rs gap` prints `2 x a ??__E dynamic-initializer TU outside the
# measured class` on every scan, which is the population that emitter is one step
# from, and no instrument varies it.
#
# MEASURED (`scripts/gt_capture.sh /Ox /GS- /c`, non-boilerplate sections):
#
#     struct S{S();int a;};      S g;          .text$yc .bss .CRT$XCU
#     struct S{S();int a;};      S g; S h;     .text$yc .text$yc .bss .CRT$XCU
#     int c();                   int g = c();  .text$yc .pdata .bss .CRT$XCU
#     int c();          int g = c(); int h = c();
#                                  .text$yc .pdata .text$yc .pdata .bss .CRT$XCU
#     struct S{S();~S();int a;}; S g;
#                        .text$yc .pdata .text$yd .bss .CRT$XCU
#     struct S{S();~S();int a;}; S g[3];
#         .text .pdata .text .pdata .text$yc .pdata .text$yd .pdata .bss .CRT$XCU
#     const char* g = "hi";                    .rdata .data     <- NOT dynamic
#     int gi; int* gp = &gi;                   .bss .data       <- NOT dynamic
#
# **One `.text$yc` per initialized object and one `.CRT$XCU` for the TU**, so the
# two counts move independently and a single-object grid cannot separate them.
# The last two lines are the reason this needs controls: they look like dynamic
# initialization in the source and are not — a `const char*` to a literal and an
# address-of are constant expressions, resolved by a relocation into `.data`, with
# no `??__E` anywhere.
#
# ---- the axes ------------------------------------------------------------------
#
#   A. WHAT MAKES IT DYNAMIC — a class constructor | a call | a call with an
#      argument | a non-constant expression over another global. And the three
#      near misses that are NOT dynamic: a literal's address, an object's
#      address, an arithmetic constant.
#   B. THE OBJECT COUNT — 1, 2, 3, 5 dynamically initialized objects. `.text$yc`
#      is per object and `.CRT$XCU` is per TU.
#   C. THE DESTRUCTOR — absent | present. Present adds an `atexit` thunk in
#      `.text$yd`, which is a second generated function and a second section.
#   D. LINKAGE — `extern` | `static` | anonymous namespace. `65-linkage-comdat.py`
#      showed an unreferenced internal function is dropped and an
#      anonymous-namespace one is not; a dynamically initialized object is
#      referenced by its own initializer and by nothing else.
#   E. THE ARRAY FORMS — `S g[3]` mints `__ehvec_ctor`/`__ehvec_dtor` COMDATs on
#      top of everything else: four generated functions from one declaration.
#   F. BESIDE STATIC DATA AND FUNCTIONS — `.data`/`.bss` objects that need no
#      initializer, and ordinary functions the port matches, in both orders. This
#      is the cross `64-data-only-tu.py` (no functions) and `66-static-local.py`
#      (data from inside a body) cannot produce between them.
#   G. THE CONTROLS — the statically initialized twin of every row. A rule that
#      keyed on "the TU declares a class with a constructor" would pass this
#      fragment's positives and lose the six workload TUs whose obj is the bare
#      shell.
#
# `NotImplemented` is the contract on nearly every row; `dyninit_tu` accepts a
# narrow whole-TU class and a `Port=Mismatch` here means it accepted one it should
# not have.

LEAF = "int %s(int a){return a+%d;}\n"

CLS = "struct S{S();int a;};\n"
CLS_D = "struct S{S();~S();int a;};\n"
CLS_A = "struct S{S();S(int);int a;};\n"


def cases(emit):
    # ---- A x B: what makes it dynamic, x how many objects --------------------
    for decl, obj in (
        (CLS,   "S %s;\n"),
        (CLS_D, "S %s;\n"),
        (CLS_A, "S %s(1);\n"),
        ("int c();\n", "int %s = c();\n"),
        ("int c(int);\n", "int %s = c(1);\n"),
        ("double c();\n", "double %s = c();\n"),
    ):
        for n in (1, 2, 3, 5):
            names = ["g%d" % i for i in range(n)]
            emit(decl + "".join(obj % x for x in names))
            # …beside a function the port matches, in both orders.
            emit(decl + "".join(obj % x for x in names) + LEAF % ("z", 1))
            emit(decl + LEAF % ("z", 1) + "".join(obj % x for x in names))

    # a global initialized from ANOTHER global — the initializer's order matters
    # and the reference is to a symbol in the same TU.
    emit("int g = 1;\nint h = g + 1;\n")
    emit("int c();\nint g = c();\nint h = g + 1;\n")
    emit("int c();\nint g = c();\nint h = c() + g;\n")
    emit(CLS + "S g;\nint h = g.a;\n")

    # ---- C: THE DESTRUCTOR — the `atexit` thunk -----------------------------
    for src in (
        CLS + "S g;\n",
        CLS_D + "S g;\n",
        CLS_D + "S g;\nS h;\n",
        CLS_D + "S g;\n" + LEAF % ("z", 1),
        "struct S{~S();int a;};\nS g;\n",
        CLS + "struct T{T();~T();int b;};\nS g;\nT h;\n",
        CLS + "struct T{T();~T();int b;};\nT h;\nS g;\n",
    ):
        emit(src)

    # ---- D: LINKAGE ---------------------------------------------------------
    for pre, post in (("static ", ""), ("", ""),
                      ("namespace { ", " }")):
        for decl, o in ((CLS, "S g;"), (CLS_D, "S g;"), ("int c();\n", "int g = c();")):
            if pre.startswith("namespace"):
                emit(decl + "namespace { %s }\n" % o)
                emit(decl + "namespace { %s }\n" % o + LEAF % ("z", 1))
            else:
                emit(decl + pre + o + "\n" + post)
                emit(decl + pre + o + "\n" + post + LEAF % ("z", 1))

    # ---- E: THE ARRAY FORMS — four generated functions from one declaration --
    for src in (
        CLS + "S g[2];\n",
        CLS + "S g[3];\n",
        CLS_D + "S g[3];\n",
        CLS_D + "S g[1];\n",
        CLS_D + "S g[3];\n" + LEAF % ("z", 1),
        CLS_D + "S g[3];\nS h;\n",
        CLS_D + "S g;\nS h[3];\n",
        "struct M{M();~M();int m;};\nstruct S{S();~S();M m;int a;};\nS g;\n",
        "struct M{M();~M();int m;};\nstruct S{S();~S();M m[2];int a;};\nS g;\n",
    ):
        emit(src)

    # ---- F: BESIDE STATIC DATA AND FUNCTIONS --------------------------------
    #
    # The dynamically initialized object needs `.bss`, the static ones need
    # `.bss`/`.data`, and they are different sections walked in different orders
    # (`docs/OBJ_DATA_BSS_SHAPE.md` §5.2/§5.3). Both declaration orders, because
    # `.data` walks declaration order and `.bss` walks `.gl` order.
    for pre, post in (("int gi;\n", ""), ("int gi = 1;\n", ""),
                      ("", "int gi;\n"), ("", "int gi = 1;\n"),
                      ("int gi;\n", "int gj = 1;\n")):
        emit(pre + CLS + "S g;\n" + post)
        emit(pre + CLS_D + "S g;\n" + post)
        emit(pre + "int c();\nint g = c();\n" + post)
        emit(pre + CLS + "S g;\n" + post + LEAF % ("z", 1))
    # a dynamically initialized object beside a function with a STATIC LOCAL —
    # two guards, two `.bss` objects, one from each mechanism.
    emit(CLS + "S g;\nint f(int a){ static int k; k=k+a; return k; }\n")
    emit(CLS + "int f(int a){ static int k; k=k+a; return k; }\nS g;\n")
    emit("int c();\nint g = c();\nint f(int a){ static int k = c(); return k+a; }\n")

    # ---- G: THE CONTROLS ----------------------------------------------------
    #
    # Every one of these LOOKS like dynamic initialization and is not: a literal's
    # address, an object's address and an arithmetic constant are all constant
    # expressions, resolved by a relocation into `.data` with no `??__E` and no
    # `.CRT$XCU` anywhere. Measured: `const char* g = "hi";` -> `.rdata .data`.
    for src in (
        'const char* g = "hi";\n',
        'const char* g = "hi";\nconst char* h = "there";\n',
        "int gi;\nint* gp = &gi;\n",
        "int gi = 1;\nint* gp = &gi;\n",
        "int g = 1 + 2;\n",
        "double gd = 1.0;\n",
        'double gd = 1.0;\nconst char* gs = "x";\n',
        # a POD with an aggregate initializer: a class-typed object with NO
        # constructor is statically initialized.
        "struct P{int a;int b;};\nP g = {1,2};\n",
        "struct P{int a;int b;};\nP g;\n",
        # the class is DECLARED with a constructor and no object is defined.
        CLS,
        CLS + LEAF % ("z", 1),
        CLS_D + "int f(int a){return a+1;}\n",
        # the bare four-section shell and an ordinary function, which the port
        # matches today — a blanket refusal keyed on "declares a constructor"
        # loses these.
        "int f(int a){return a+1;}\n",
        "int gi;\n",
        "int gi = 1;\n",
    ):
        emit(src)
