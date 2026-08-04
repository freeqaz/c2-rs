# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# **LINKAGE, COMDAT AND THE EMIT-SET** (lane w-shapes; `scripts/sweep_shapes.py`
# rows `static (internal) function`, `inline keyword`, `anonymous namespace`,
# `operator overload` and `main()` — all five ZERO before this file).
#
# The five rows are one fragment because they are one question: **given the IL,
# which functions end up in the obj and in which section.** Every generated case
# in the corpus answers it the same way — one `extern`, decorated, non-COMDAT
# function per `.ex` segment — so the corpus has never once separated the port's
# rule from that coincidence.
#
# ---- the measurement that makes this the highest-value row ---------------------
#
#     $ cat r1.cpp
#     static int u(int a){return a+2;}
#     int f(int a){return a+1;}
#
#     $ c2rs census r1.cpp
#     r1.cpp -> 2/2 functions in class
#       [  0] ok  straight-line  cflow-straight+expr-modeled  eh-none   99 B  (unnamed)
#       [  1] ok  straight-line  cflow-straight+expr-modeled  eh-none  107 B  (unnamed)
#
#     $ c2rs diff r1.cpp
#     ReferenceReplay=ByteExact (ref=853B)  Port=NotImplemented
#
# **The obj has ONE function.** c2 drops an unreferenced internal-linkage
# function, and the census — whose verdict is per *function*, over the `.ex`
# segments — says both bodies are in the port's codegen class. The port refuses
# today, and the reason it refuses is not the drop: it is that the `.gl` name
# binding comes back `(unnamed)`, which is an accident of a different subsystem.
# `docs/STATUS.md` trap 2 is exactly this shape — *a per-function census claim
# for a never-emitted body can never be graded* — and factor A (`.ex` segments ==
# `.text` COMDATs) is exactly this count. A widening that taught the binder to
# name those two segments would turn a clean refusal into a two-COMDAT obj
# against c2's one, which is #232's direction.
#
# The same measurement, one line different, goes the other way:
#
#     namespace { int u(int a){return a+2;} }
#     int f(int a){return a+1;}          ->  903 B, .text, **Port=Match**
#
# An unreferenced *anonymous-namespace* function is NOT dropped. So "internal
# linkage" is not the predicate, and a fragment carrying only `static` would have
# fitted the wrong rule.
#
# ---- and the section shape moves under the same source edit --------------------
#
# MEASURED, `scripts/gt_capture.sh /Ox /GS- /c`, non-boilerplate sections only:
#
#     static int u(int a){return a+2;}  int f(int a){return u(a)+1;}   .text
#     inline int u(int a){return a+2;}  int f(int a){return u(a)+1;}   .text .text
#     static int u…  int (*p)(int)=u;   int f(int a){return a+1;}      .data .text
#     inline int u…  int (*p)(int)=u;   int f(int a){return a+1;}      .data .text .text
#     namespace { int g; }              int f(int a){return a+1;}      .bss  .text
#
# `static` and `inline` produce **identical IL classes, identical census
# verdicts and identical names**, and different objs: `inline` mints a second
# `.text` COMDAT. A port that decided from its own class model alone gets one of
# those two wrong whichever way it decides.
#
# ---- the axes ------------------------------------------------------------------
#
#   A. SPECIFIER on the extra function — `extern` (the control) | `static` |
#      `inline` | `static inline` | `__forceinline` | `__declspec(noinline)
#      static` | anonymous namespace | `extern "C"` | an in-class-defined member
#      (implicitly inline, implicitly COMDAT).
#   B. REFERENCED-NESS — not referenced | tail-called | called from a frame |
#      address taken through a namespace-scope pointer | called only by another
#      function that is itself dropped. This is the axis that decides the DROP,
#      and only the `extern` rows are drop-immune.
#   C. THE EMIT-SET ARITY — a TU with 1, 2, 3, 4 defined functions of which 0, 1,
#      2, 3 are dropped. Factor A is a COUNT equality; a grid whose every row
#      drops zero or one cannot see an off-by-one in it.
#   D. POSITION — the specifier-carrying function first, last, and wedged between
#      two ordinary ones. `63-emit-order.py` established that `.text` order is a
#      dependency walk; a dropped function is a node that must leave the walk.
#   E. NAME SHAPE — decorated (`?f@@YAHH@Z`) | undecorated (`extern "C"`, `main`)
#      | operator (`??H`, `??A`, `??R`, `??4`, `??6`, `??8`) | anonymous-namespace
#      (`?u@?A0x…@@`, whose length varies with a path hash). `main` and
#      `extern "C"` both come back `(unnamed)` from the `.gl` binder today while
#      being 1/1 in class, which is the same latent shape as the drop rows.
#   F. OPERATOR ARITY AND KIND — member vs free, unary vs binary, by-value vs by
#      reference, and the ones with a non-`int` result. `operator=` returning
#      `S&` is **`Port=Match` today** (measured), so this axis has a live
#      positive and is not a wall of refusals.
#   G. THE CONTROLS — the same TUs with every function `extern` and ordinary.
#      Four of them are `Port=Match` today. Without them a fix that refused any
#      TU containing the token `static`, `inline` or `operator` would pass this
#      fragment while losing shapes the port emits correctly right now.
#
# `NotImplemented` is the contract on most rows. A `Port=Mismatch` here means the
# port emitted a function c2 did not, or put one in the wrong section.

LEAF = "int %s(int a){return a+%d;}\n"

# (tag, prefix, needs-wrapping-in-a-namespace-block)
SPECIFIERS = (
    ("ext",        "",                                False),
    ("sta",        "static ",                         False),
    ("inl",        "inline ",                         False),
    ("stainl",     "static inline ",                  False),
    ("force",      "__forceinline ",                  False),
    ("noinl",      "__declspec(noinline) static ",    False),
    ("noinlext",   "__declspec(noinline) ",           False),
    ("anon",       "",                                True),
    ("cext",       'extern "C" ',                     False),
)


def spec_def(tag, prefix, wrap, name, k):
    body = "%sint %s(int a){return a+%d;}\n" % (prefix, name, k)
    return "namespace { %s }\n" % body.strip() if wrap else body


def cases(emit):
    # ---- A x B: every specifier x every way of being referenced --------------
    #
    # `u` is the specifier-carrying function; `f` is an ordinary one the port
    # matches on its own. Five reference kinds, including two that keep `u` alive
    # without a call instruction.
    for tag, prefix, wrap in SPECIFIERS:
        u = spec_def(tag, prefix, wrap, "u", 2)
        # not referenced at all — `static`/`inline` are DROPPED here and the
        # others are not.
        emit(u + "int f(int a){return a+1;}\n")
        emit("int f(int a){return a+1;}\n" + u)
        # tail-called
        emit(u + "int f(int a){return u(a);}\n")
        # called from a frame
        emit(u + "int f(int a){return u(a)+1;}\n")
        # called twice — one edge, two call sites
        emit(u + "int f(int a){return u(a)+u(a+1);}\n")
        # address taken through namespace-scope data: no call instruction
        # anywhere, and the reference lives in `.data` with a relocation.
        emit(u + "int (*p)(int)=u;\nint f(int a){return a+1;}\n")
        emit(u + "int (*p)(int)=u;\nint f(int a){return u(a);}\n")
        # referenced only by a function that is ITSELF dropped: the drop has to
        # be a fixpoint, not one pass.
        v = spec_def(tag, prefix, wrap, "v", 3)
        emit(u + v.replace("return a+3;", "return u(a)+3;") + "int f(int a){return a+1;}\n")
        # wedged between two ordinary functions (axis D)
        emit(LEAF % ("y", 4) + u + LEAF % ("z", 1))
        emit(LEAF % ("y", 4) + u.replace("a+2", "y(a)+2") + LEAF % ("z", 1))

    # ---- C: THE EMIT-SET ARITY — 1..4 defined, 0..3 dropped ------------------
    #
    # Factor A is `.ex segments == .text COMDATs`. These rows walk the difference
    # from 0 to 3 while holding every body in the port's straight-line class, so
    # the only thing varying is the COUNT.
    for ndrop in (0, 1, 2, 3):
        dropped = "".join("static int d%d(int a){return a+%d;}\n" % (i, i + 5)
                          for i in range(ndrop))
        emit(dropped + "int f(int a){return a+1;}\n")
        emit("int f(int a){return a+1;}\n" + dropped)
        emit(dropped + "int f(int a){return a+1;}\n" + LEAF % ("z", 1))
        emit(LEAF % ("z", 1) + dropped + "int f(int a){return a+1;}\n")
        # the same count, kept alive: the drop count goes to zero without the
        # source shape changing anywhere else.
        kept = dropped.replace("static ", "")
        emit(kept + "int f(int a){return a+1;}\n")
        emit(kept + "int f(int a){return a+1;}\n" + LEAF % ("z", 1))
        # `inline` instead of `static`, same count — same IL class, different
        # section shape when they survive.
        inl = dropped.replace("static ", "inline ")
        emit(inl + "int f(int a){return a+1;}\n")
        emit(inl + "int f(int a){return " + "+".join("d%d(a)" % i for i in range(ndrop))
             + ("+a+1;}\n" if ndrop else "a+1;}\n"))

    # ---- A: the IN-CLASS-DEFINED MEMBER — implicitly inline, implicitly COMDAT
    #
    # The commonest COMDAT in real C++ and the corpus has never written one. Both
    # referenced and not, and beside an out-of-line member which is not COMDAT.
    emit("struct C{ int m(int a){return a+1;} };\nint f(int a){return a+1;}\n")
    emit("struct C{ int m(int a){return a+1;} };\n"
         "int f(int a){ C c; return c.m(a); }\n")
    emit("struct C{ int m(int a){return a+1;} int n(int); };\n"
         "int C::n(int a){return a+2;}\n")
    emit("struct C{ int m(int a){return a+1;} int n(int); };\n"
         "int C::n(int a){return m(a)+1;}\n")
    emit("struct C{ int m(int a){return a+1;} int n(int); };\n"
         "int C::n(int a){return a+2;}\nint f(int a){ C c; return c.m(a); }\n")
    emit("struct C{ static int s(int a){return a+1;} };\nint f(int a){return C::s(a);}\n")
    emit("struct C{ static int s(int a){return a+1;} };\nint f(int a){return a+1;}\n")

    # ---- E: NAME SHAPE — `main`, `extern \"C\"`, and both beside ordinary ------
    #
    # `main` is 1/1 in class and `(unnamed)`, measured. It is the only function
    # in C++ whose symbol c2 writes undecorated without being asked.
    for src in (
        "int main(){return 0;}\n",
        "int main(int a){return a+1;}\n",
        "int main(int argc, char** argv){return argc+1;}\n",
        "int main(){return 0;}\n" + LEAF % ("z", 1),
        LEAF % ("z", 1) + "int main(){return 0;}\n",
        "int main(int a){return a+1;}\nint f(int b){return b+1;}\n",
        'extern "C" int f(int a){return a+1;}\n',
        'extern "C" int u(int a){return a+2;}\nint f(int a){return a+1;}\n',
        'extern "C" int u(int a){return a+2;}\nint f(int a){return u(a);}\n',
        'extern "C" { int u(int a){return a+2;} int v(int a){return a+3;} }\n',
        'extern "C" int u(int a){return a+2;}\n' + LEAF % ("z", 1) + "int f(int a){return a+1;}\n",
        # `extern "C"` DATA beside an ordinary function.
        'extern "C" int gv;\nint f(int a){return gv+a;}\n',
    ):
        emit(src)

    # ---- E/F: OPERATOR OVERLOADS — member and free, every arity --------------
    #
    # `??4` (`operator=`) returning `S&` is `Port=Match` today; the rest refuse.
    # The point of the axis is the SYMBOL, not the body: every one of these has a
    # body the port already knows and a name it has never seen.
    MEMBER_OPS = (
        ("int operator+(int y)",     "return a+y;",        "??H"),
        ("int operator-(int y)",     "return a-y;",        "??G"),
        ("int operator*(int y)",     "return a*y;",        "??D"),
        ("int operator==(int y)",    "return a==y;",       "??8"),
        ("int operator<(int y)",     "return a<y;",        "??M"),
        ("int operator<<(int y)",    "return a+y;",        "??6"),
        ("int operator>>(int y)",    "return a-y;",        "??5"),
        ("int operator()(int y)",    "return a+y;",        "??R"),
        ("int operator[](int y)",    "return a+y;",        "??A"),
        ("int operator!()",          "return a==0;",       "??7"),
        ("int operator++()",         "return ++a;",        "??E"),
        ("S& operator=(int y)",      "a=y;return *this;",  "??4"),
        ("S* operator->()",          "return this;",       "??C"),
    )
    for sig, body, _mangled in MEMBER_OPS:
        outofline = sig.replace("operator", "S::operator", 1) + "{" + body + "}\n"
        emit("struct S{int a;%s;};\n%s" % (sig, outofline))
        # …beside a function the port matches, so the operator's symbol is not
        # the only one in the obj.
        emit("struct S{int a;%s;};\n%s%s" % (sig, outofline, LEAF % ("z", 1)))
        # …and defined IN CLASS, which makes the same operator a COMDAT.
        emit("struct S{int a;%s{%s}};\nint f(int a){return a+1;}\n" % (sig, body))
    # free operators: by reference and by pointer-free value, unary and binary.
    for src in (
        "struct S{int a;};\nint operator+(S& x,int y){return x.a+y;}\n",
        "struct S{int a;};\nint operator-(S& x,int y){return x.a-y;}\n",
        "struct S{int a;};\nint operator!(S& x){return x.a==0;}\n",
        "struct S{int a;};\nint operator<<(S& x,int y){return x.a+y;}\n",
        "struct S{int a;};\nint operator+(S& x,int y){return x.a+y;}\n" + LEAF % ("z", 1),
        "struct S{int a;};\nint operator+(S x,int y){return x.a+y;}\n",
        "struct S{int a;};\nstatic int operator+(S& x,int y){return x.a+y;}\n"
        "int f(S& s){return s+1;}\n",
        "struct S{int a;};\ninline int operator+(S& x,int y){return x.a+y;}\n"
        "int f(S& s){return s+1;}\n",
        "struct S{int a;};\ninline int operator+(S& x,int y){return x.a+y;}\n"
        "int f(int a){return a+1;}\n",
    ):
        emit(src)

    # ---- A: ANONYMOUS NAMESPACE — functions, data, and nesting ---------------
    #
    # The mangled name carries a path-derived hash, so its LENGTH is not a
    # constant the string table can be fitted to. An unreferenced one survives
    # (measured `Port=Match`), which is what separates it from `static`.
    for src in (
        "namespace { int u(int a){return a+2;} }\nint f(int a){return a+1;}\n",
        "namespace { int u(int a){return a+2;} }\nint f(int a){return u(a);}\n",
        "namespace { int u(int a){return a+2;} }\nint f(int a){return u(a)+1;}\n",
        "namespace { int g; }\nint f(int a){return a+1;}\n",
        "namespace { int g; }\nint f(int a){return g+a;}\n",
        "namespace { int g = 1; }\nint f(int a){return g+a;}\n",
        "namespace { int u(int a){return a+2;} int g; }\nint f(int a){return u(a)+g;}\n",
        "namespace N { namespace { int u(int a){return a+2;} } }\n"
        "int f(int a){return N::u(a);}\n",
        "namespace { namespace M { int u(int a){return a+2;} } }\n"
        "int f(int a){return M::u(a);}\n",
        "namespace { struct S{int a;}; }\nint f(int a){return a+1;}\n",
        "namespace { int u(int a){return a+2;} }\n" + LEAF % ("z", 1)
        + "int f(int a){return a+1;}\n",
    ):
        emit(src)

    # ---- G: THE CONTROLS ----------------------------------------------------
    #
    # Every one of these is a row above with the specifier removed. Four are
    # `Port=Match` on this tip (measured: rows 1, 2, 3 and the `operator=`
    # member), so a blanket refusal keyed on a token fails here.
    for src in (
        "int f(int a){return a+1;}\n",
        "int u(int a){return a+2;}\nint f(int a){return a+1;}\n",
        "int f(int a){return a+1;}\nint u(int a){return a+2;}\n",
        LEAF % ("y", 4) + LEAF % ("z", 1),
        LEAF % ("y", 4) + "int f(int a){return a+1;}\n" + LEAF % ("z", 1),
        "int u(int a){return a+2;}\nint (*p)(int)=u;\nint f(int a){return a+1;}\n",
        "struct S{int a;S& assign(int y);};\nS& S::assign(int y){a=y;return *this;}\n",
        "struct C{ int n(int); };\nint C::n(int a){return a+2;}\n",
        "int g;\nint f(int a){return g+a;}\n",
        "int g = 1;\nint f(int a){return g+a;}\n",
    ):
        emit(src)
