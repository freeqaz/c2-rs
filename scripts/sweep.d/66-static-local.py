# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# **FUNCTION-LOCAL STATIC STORAGE** (lane w-shapes; `scripts/sweep_shapes.py` row
# `static local variable`, ZERO before this file).
#
# `64-data-only-tu.py` closed the TU that has data and **no** functions. This is
# the other side of the same seam: a TU that has a function **and** data, where
# the data is minted by the *body* rather than by a namespace-scope declaration.
# It is the only construct in C++ that makes `.bss`, `.data`, a guard variable
# and an `atexit` thunk appear because of something written *inside* a function,
# and the port's data writer — rewritten twice this week (`w-r1c` added `.bss`,
# `.CRT$XCU` and `.text$yc`; `w-sect` added the data-only arm) — has never been
# shown a single case of it.
#
# MEASURED here (`scripts/gt_capture.sh`, `/Ox /GS- /c`, section names out of the
# COFF header):
#
#     int f(int a){static int k;k=k+a;return k;}      .text .bss
#     int f(int a){static int k=5;return k+a;}        .text .data
#     int f(int a){static int k;static int j=2;…}     .text .bss .data
#     int g; int f(int a){static int k;…return k+g;}  .bss .text .bss   <- ORDER
#     int c(); int f(int a){static int k=c();…}       .text .bss .bss .pdata
#     struct S{S();~S();int a;};
#     int f(int a){static S s;return a+1;}            .text .bss .bss .pdata .text$yd
#
# Four things in that table are new to the corpus and three are load-bearing:
#
#   * **`.text$yd`** — the `atexit` thunk for a static whose type has a
#     destructor. It is **646 sections across 243 objs** of the dc3 workload and
#     is factor C's third ladder step (`docs/STATUS.md`: `.rdata$r` 590 ->
#     `.text$yd` 804). Six characters of source produce one.
#   * **`.bss` on BOTH SIDES of `.text`** — a namespace-scope object and a static
#     local in the same TU do not share a section and are not adjacent. Any walk
#     that assumes "the data sections come first" is wrong here, and
#     `64-data-only-tu.py` cannot produce the shape because it writes no
#     functions.
#   * **two `.data` sections, not one**, for two static locals in two different
#     functions — the sections are per-object, not per-TU.
#   * a **guard variable**: dynamic initialization mints a second `.bss` object
#     that never appears in the source at all.
#
# ---- the axes are STRUCTURAL ---------------------------------------------------
#
# Initializer *values* are held at `1`, `2`, `5` throughout. Three grids on this
# project varied values exhaustively and missed on arity, register position and
# structural counts; the cross here is over the shape:
#
#   A. INITIALIZER KIND — absent (`.bss`) | constant zero (`.bss`) | constant
#      non-zero (`.data`) | dynamic, i.e. a call (`.bss` guard + `.bss`) |
#      aggregate. This is the axis that picks the SECTION.
#   B. TYPE — `int`, `char`, `short`, `double`, a pointer, `int[4]`, `char[8]`, a
#      POD struct, a class with a constructor, a class with a destructor. The
#      last two are the ones that mint a guard and a `.text$yd` thunk.
#   C. COUNT AND PLACEMENT — 1, 2, 3 statics in one function; one static each in
#      two functions; statics of both initializer kinds in the same function.
#      `64-data-only-tu.py` §A found the writer's class bound is `<= 2` objects
#      per section, so the rows on both sides of 2 are what say the bound is real.
#   D. USE — read | written | read-and-written | address taken | returned by
#      pointer. An address-taken static is the only row that forces a relocation
#      into `.text` naming a `.bss` symbol.
#   E. SCOPE — function body | inside an `if` | inside a `for` | a class member
#      function | a function that also has automatic locals.
#   F. NAMESPACE-SCOPE DATA BESIDE IT — none | `.bss` | `.data` | both. This is
#      the section-ORDER row, and it is the cross `64-data-only-tu.py` and this
#      file only produce together.
#   G. THE FUNCTION'S OWN CLASS — a leaf the port matches today, a tail call, a
#      framed call. A static local must not change the verdict for a reason the
#      body owns.
#   H. THE CONTROLS — the identical function with an AUTOMATIC local, and with a
#      `static const` integer (which c2 folds away and gives NO section at all,
#      measured: 845 B, `.text` only — the same obj as a function with no local).
#      Without them a fix that refused every TU containing the token `static`
#      would pass this fragment.
#
# `NotImplemented` is the contract on nearly every row. A `Port=Mismatch` means
# the writer placed an obj whose `.bss`/`.data`/`.text$yd` it did not model.

LEAF = "int %s(int a){return a+%d;}\n"

# (declaration, an expression that uses it) — the type axis, B.
TYPES = (
    ("static int k;",            "k"),
    ("static int k = 0;",        "k"),
    ("static int k = 5;",        "k"),
    ("static char k;",           "(int)k"),
    ("static char k = 3;",       "(int)k"),
    ("static short k;",          "(int)k"),
    ("static short k = 3;",      "(int)k"),
    ("static double k;",         "(int)k"),
    ("static double k = 1.5;",   "(int)k"),
    ("static float k = 1.5f;",   "(int)k"),
    ("static int* k;",           "(k?1:0)"),
    ("static int k[4];",         "k[1]"),
    ("static int k[4] = {1,2,3,4};", "k[1]"),
    ("static char k[8];",        "(int)k[1]"),
    ("static char k[64];",       "(int)k[1]"),   # over the align-8 promotion step
)


def cases(emit):
    # ---- A x B x D: one static, every type, every use ------------------------
    for decl, use in TYPES:
        emit("int f(int a){ %s return %s + a; }\n" % (decl, use))          # read
        if "[" not in decl and "*" not in decl:
            name = "k"
            emit("int f(int a){ %s %s = (%s)a; return a+1; }\n"
                 % (decl, name, decl.split()[1].rstrip(";")))              # write
            emit("int f(int a){ %s %s = %s + (%s)a; return %s; }\n"
                 % (decl, name, name, decl.split()[1].rstrip(";"), use))   # rmw

    # address taken, and returned by pointer — the rows that force a relocation
    # in `.text` naming a `.bss`/`.data` symbol.
    emit("int* f(){ static int k; return &k; }\n")
    emit("int* f(){ static int k = 5; return &k; }\n")
    emit("void g(int*);\nint f(int a){ static int k; g(&k); return a+1; }\n")
    emit("void g(int*);\nint f(int a){ static int k = 5; g(&k); return a+1; }\n")
    emit("int f(int a){ static int k[4]; return k[a]; }\n")
    emit("int* f(){ static int k[4]; return k; }\n")

    # ---- A: DYNAMIC initialization — the guard variable ----------------------
    #
    # A static initialized by a call gets a `.bss` guard object that appears
    # nowhere in the source, plus a `.pdata` entry for the now-framed function.
    emit("int c();\nint f(int a){ static int k = c(); return k + a; }\n")
    emit("int c();\nint f(int a){ static int k = c(); static int j = 2; return k+j+a; }\n")
    emit("int c();\nint f(int a){ static int j = 2; static int k = c(); return k+j+a; }\n")
    emit("int c();\nint f(int a){ static int k = c(); static int j = c(); return k+j+a; }\n")
    emit("int c(int);\nint f(int a){ static int k = c(a); return k + a; }\n")
    emit("static int q(int a){return a+1;}\n"
         "int f(int a){ static int k = q(1); return k + a; }\n")

    # ---- B: CLASS-TYPED statics — a constructor, then a destructor -----------
    #
    # The constructor case mints the guard; adding a destructor additionally
    # mints the `atexit` thunk in `.text$yd`, a section the corpus has otherwise
    # never produced. Both classes are declared and not defined: `/c` only.
    emit("struct S{S();int a;};\nint f(int a){ static S s; return a + s.a; }\n")
    emit("struct S{S();S(int);int a;};\nint f(int a){ static S s(1); return a + s.a; }\n")
    emit("struct S{S();~S();int a;};\nint f(int a){ static S s; return a + s.a; }\n")
    emit("struct S{S();~S();int a;};\nint f(int a){ static S s; return a + 1; }\n")
    emit("struct S{S();~S();int a;};\nint f(int a){ static S s; static S t; return a + s.a + t.a; }\n")
    emit("struct S{S();~S();int a;};\nstruct T{T();int b;};\n"
         "int f(int a){ static S s; static T t; return a + s.a + t.b; }\n")
    emit("struct S{S();~S();int a;};\nstruct T{T();int b;};\n"
         "int f(int a){ static T t; static S s; return a + s.a + t.b; }\n")
    emit("struct S{~S();int a;};\nint f(int a){ static S s; return a + s.a; }\n")
    # POD aggregate — no guard, straight into `.data`.
    emit("struct P{int a;int b;};\nint f(int a){ static P p = {1,2}; return a + p.a; }\n")
    emit("struct P{int a;int b;};\nint f(int a){ static P p; return a + p.a; }\n")
    emit("struct P{char c;int a;};\nint f(int a){ static P p = {1,2}; return a + p.a; }\n")

    # ---- C: COUNT — 1, 2, 3 in one function; one each in two functions -------
    #
    # Two statics of the SAME initializer kind share a section; two of different
    # kinds do not. Two functions with one static each produce two `.data`
    # sections and not one (measured), so "per TU" is the wrong unit.
    emit("int f(int a){ static int k; return k+a; }\n")
    emit("int f(int a){ static int k; static int j; return k+j+a; }\n")
    emit("int f(int a){ static int k; static int j; static int i; return k+j+i+a; }\n")
    emit("int f(int a){ static int k=1; static int j=2; return k+j+a; }\n")
    emit("int f(int a){ static int k=1; static int j=2; static int i=3; return k+j+i+a; }\n")
    emit("int f(int a){ static int k; static int j=2; return k+j+a; }\n")
    emit("int f(int a){ static int j=2; static int k; return k+j+a; }\n")
    emit("int f(int a){ static int k=1; return k+a; }\n"
         "int g(int a){ static int j=2; return j+a; }\n")
    emit("int f(int a){ static int k; return k+a; }\nint g(int a){ static int j; return j+a; }\n")
    emit("int f(int a){ static int k; return k+a; }\nint g(int a){ static int j=2; return j+a; }\n")
    emit("int f(int a){ static int k; return k+a; }\n" + LEAF % ("z", 1))
    emit(LEAF % ("z", 1) + "int f(int a){ static int k; return k+a; }\n")
    emit(LEAF % ("y", 2) + "int f(int a){ static int k; return k+a; }\n" + LEAF % ("z", 1))
    # mixed types, so the section's alignment is not a function of one object.
    emit("int f(int a){ static char c; static double d; return (int)c+(int)d+a; }\n")
    emit("int f(int a){ static double d; static char c; return (int)c+(int)d+a; }\n")
    emit("int f(int a){ static char c=1; static double d=2.0; return (int)c+(int)d+a; }\n")
    emit("int f(int a){ static double d=2.0; static char c=1; return (int)c+(int)d+a; }\n")

    # ---- E: SCOPE — nested blocks, loops, member functions -------------------
    #
    # A static in a nested scope has the same storage and a different
    # construction point; in a loop body it is constructed once.
    emit("int f(int a){ if(a){ static int k; k=k+1; return k; } return 0; }\n")
    emit("int f(int a){ if(a){ static int k=1; return k+a; } return 0; }\n")
    emit("int f(int a){ for(int i=0;i<a;i++){ static int k; k=k+i; } return a; }\n")
    emit("int c();\nint f(int a){ if(a){ static int k=c(); return k; } return 0; }\n")
    emit("int f(int a){ { static int k; k=k+1; } { static int j; j=j+1; return j; } }\n")
    emit("struct C{int m(int);};\nint C::m(int a){ static int k; k=k+a; return k; }\n")
    emit("struct C{int m(int);int n(int);};\n"
         "int C::m(int a){ static int k; k=k+a; return k; }\n"
         "int C::n(int a){ static int j=2; return j+a; }\n")
    emit("int f(int a){ int t = a+1; static int k; k=k+t; return k; }\n")
    emit("int f(int a){ static int k; int t = a+1; k=k+t; return k; }\n")

    # ---- F: NAMESPACE-SCOPE DATA BESIDE IT — the section-ORDER row -----------
    #
    # Measured: `int g; int f(int a){static int k;…}` gives
    # `.XBLD$W .bss .XBLD$W .text .bss` — two `.bss` sections with `.text`
    # BETWEEN them. Neither `64-data-only-tu.py` (no functions) nor any other
    # fragment (no static locals) can produce that.
    for outer in ("int g;\n", "int g = 1;\n", "int g;\nint h = 1;\n",
                  "static int g;\n", "static int g = 1;\n"):
        emit(outer + "int f(int a){ static int k; k=k+a; return k+g; }\n")
        emit(outer + "int f(int a){ static int k=1; return k+a+g; }\n")
        emit(outer + "int f(int a){ static int k; return k+g+a; }\n" + LEAF % ("z", 1))
    emit("int g;\nint f(int a){ static int k; return k+g+a; }\nint h;\n")
    emit("int g = 1;\nint f(int a){ static int k = 2; return k+g+a; }\nint h = 3;\n")

    # ---- G: THE FUNCTION'S OWN CLASS ----------------------------------------
    #
    # A leaf, a tail call and a framed call, each carrying the same static. The
    # static must not be the thing that decides the verdict for a body the port
    # would otherwise refuse for its own reasons — and it must not rescue one it
    # would otherwise accept.
    emit("int f(int a){ static int k; return k+a; }\n")
    emit("int q(int);\nint f(int a){ static int k; return q(k+a); }\n")
    emit("int q(int);\nint f(int a){ static int k; return q(a)+k; }\n")
    emit("int q(int);\nint f(int a){ static int k=1; return q(a)+k; }\n")
    emit("int q(int);\nint f(int a){ static int k; k = q(a); return k; }\n")

    # ---- H: THE CONTROLS ----------------------------------------------------
    #
    # `static const int` is folded and produces NO section: measured 845 B,
    # `.text` only, byte-identical in size to `int f(int a){return a+1;}`. A rule
    # that keys on the token rather than on the storage gets these wrong.
    for src in (
        "int f(int a){ static const int k = 5; return k + a; }\n",
        "int f(int a){ static const int k = 0; return k + a; }\n",
        "int f(int a){ const int k = 5; return k + a; }\n",
        "int f(int a){ int k = 5; return k + a; }\n",
        "int f(int a){ int k; k = a + 1; return k; }\n",
        "int f(int a){ int k[4]; k[0] = a; return k[0]; }\n",
        "int f(int a){ return a + 1; }\n",
        LEAF % ("z", 1) + "int f(int a){ return a + 1; }\n",
        # `static` on the FUNCTION, not on a local — a different row entirely,
        # and `65-linkage-comdat.py` owns it. Here to keep the two separable.
        "static int f(int a){ return a + 1; }\nint g(int a){ return f(a); }\n",
    ):
        emit(src)
