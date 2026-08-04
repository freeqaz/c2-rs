# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# **`union`** (lane w-shapes; `scripts/sweep_shapes.py` row `union`, ZERO before
# this file).
#
# The cheapest of the twelve zero rows and the one with the least to say, which
# is why it is written last and why this header is short. It is here because it
# is the only aggregate whose members share an OFFSET: every `struct` in the
# corpus assigns each member a distinct one, so no case has ever asked the
# decoder for two names that resolve to the same address, and no case has ever
# asked the data writer for a section whose object is larger than the member that
# initialises it.
#
# It is not a wall of refusals: `union U{int i;char c[4];}; int f(U* u){return
# u->i;}` is **`Port=Match`** today, as are the nested and in-struct forms, so
# the axis has live positives and the controls below can be read against them.
#
# ---- the axes ------------------------------------------------------------------
#
#   A. WHERE the union lives — a pointer parameter | a by-value parameter | a
#      return value | an automatic local | a namespace-scope object (`.bss`) |
#      an initialised namespace-scope object (`.data`) | a member of a struct |
#      an anonymous member | an array of unions.
#   B. MEMBER TYPES — same width (`int`/`int`), punned (`int`/`float`),
#      mixed width (`int`/`long long`, so the union is bigger than its first
#      member), an array member, a struct member, a nested union.
#   C. ACCESS — read the first member | read a later one | write one and read
#      another | take the address of one | index an array member.
#   D. SIZE AND ALIGNMENT — 1, 4, 8 and 64 bytes, at the `.bss`/`.data` writer's
#      own promotion steps (`64-data-only-tu.py` §E: `align = max(natural, 1 if
#      n<2 else 4 if n<64 else 8)`). A union takes the MAX alignment of its
#      members and the MAX size, and those are two different members.
#   E. THE STRUCT CONTROL — every namespace-scope and by-value row is emitted a
#      second time with `union` replaced by `struct`. That is a one-word edit
#      whose only effect is the member offsets and the object's size, so a
#      verdict that differs across the pair is the keyword's and a verdict that
#      differs on both is not.
#   F. a union with a member function and with a constructor — a union is a class
#      type, and `U::get()` is a member function on one.

LEAF = "int %s(int a){return a+%d;}\n"

# (tag, member list, first member's NAME, (cast, member A), (cast, member B))
# `cast` is what an `int`-returning function needs in front of the access; the
# member spellings are kept separate from it so a write target is always
# available and never has a cast glued to it.
BODIES = (
    ("ii",   "int i;int j;",                  "i", ("", "i"),      ("", "j")),
    ("if_",  "int i;float f;",                "i", ("", "i"),      ("(int)", "f")),
    ("ic",   "int i;char c[4];",              "i", ("", "i"),      ("(int)", "c[1]")),
    ("ill",  "int i;long long l;",            "i", ("", "i"),      ("(int)", "l")),
    ("cd",   "char c;double d;",              "c", ("(int)", "c"), ("(int)", "d")),
    ("is",   "int i;struct{int a;int b;}s;",  "i", ("", "i"),      ("", "s.b")),
    ("iu",   "int i;union{int x;float y;}u;", "i", ("", "i"),      ("", "u.x")),
    ("ia64", "int i;char big[64];",           "i", ("", "i"),      ("(int)", "big[63]")),
    ("c1",   "char c;",                       "c", ("(int)", "c"), ("(int)", "c")),
)


def cases(emit):
    # ---- A x B x C: where it lives, what is in it, which member is read ------
    for _tag, members, first, (ca, ma), (cb, mb) in BODIES:
        U = "union U{%s};\n" % members
        # through a pointer — the shape that is Port=Match today.
        emit(U + "int f(U* u){return %su->%s;}\n" % (ca, ma))
        emit(U + "int f(U* u){return %su->%s;}\n" % (cb, mb))
        emit(U + "int f(U* u){return %su->%s + %su->%s;}\n" % (ca, ma, cb, mb))
        # write one member, read another: the punning row.
        emit(U + "int f(U* u, int a){u->%s = (a?1:0); return %su->%s;}\n"
             % (first, cb, mb))
        # an automatic local, written then read back through the other member.
        emit(U + "int f(int a){U u; u.%s = (a?1:0); return %su.%s;}\n"
             % (first, cb, mb))
        emit(U + "int f(U* u, int a){return %su->%s + a;}\n" % (ca, ma))
        # by value, and returned
        emit(U + "int f(U u){return %su.%s;}\n" % (ca, ma))
        emit(U + "U g(int);\nint f(int a){return %sg(a).%s;}\n" % (ca, ma))
        # a member of a struct, and an ARRAY of unions
        emit(U + "struct S{int a;U u;};\nint f(S* s){return %ss->u.%s;}\n" % (ca, ma))
        emit(U + "struct S{U u;int a;};\nint f(S* s){return %ss->u.%s + s->a;}\n" % (ca, ma))
        emit(U + "int f(U* u, int a){return %su[a].%s;}\n" % (ca, ma))

    # ---- A x D: NAMESPACE-SCOPE unions — the `.bss`/`.data` writer -----------
    #
    # A union's size is its largest member's and its alignment its most-aligned
    # member's, and those are two different members in every row here. Each is
    # emitted uninitialised (`.bss`) and initialised (`.data`), and each is
    # paired with the equivalent `struct` (axis E) — a one-word edit whose only
    # effect is the offsets and the size.
    for _tag, members, _first, (ca, ma), _rb in BODIES:
        for kw in ("union", "struct"):
            T = "%s U{%s};\n" % (kw, members)
            emit(T + "U g;\nint f(int a){return %sg.%s + a;}\n" % (ca, ma))
            emit(T + "U g;\nU h;\nint f(int a){return %sg.%s + %sh.%s + a;}\n"
                 % (ca, ma, ca, ma))
            emit(T + "U g;\nint gi;\nint f(int a){return %sg.%s + gi + a;}\n" % (ca, ma))
            emit(T + "U g[2];\nint f(int a){return %sg[a].%s;}\n" % (ca, ma))
            emit(T + "U g;\n")                       # no function at all
            emit(T + "U g;\nint gi = 1;\n")          # …beside initialised data
    # initialised namespace-scope unions: only the FIRST member can be
    # initialised, so the object is larger than its initialiser. That is the
    # `.data` row a struct cannot produce.
    for src in (
        "union U{int i;float f;};\nU g = {1};\nint f(int a){return g.i + a;}\n",
        "union U{int i;long long l;};\nU g = {1};\nint f(int a){return g.i + a;}\n",
        "union U{int i;char big[64];};\nU g = {1};\nint f(int a){return g.i + a;}\n",
        "union U{char c;double d;};\nU g = {1};\nint f(int a){return g.c + a;}\n",
        "union U{int i;float f;};\nU g = {1};\n",
        "union U{int i;char big[64];};\nU g = {1};\n",
        "union U{int i;float f;};\nU g = {1};\nU h;\n",
        "struct U{int i;float f;};\nU g = {1};\nint f(int a){return g.i + a;}\n",
        "struct U{int i;float f;};\nU g = {1,2.0f};\nint f(int a){return g.i + a;}\n",
    ):
        emit(src)

    # ---- A: ANONYMOUS unions -------------------------------------------------
    #
    # An anonymous union member has no name of its own, so its members are the
    # enclosing type's members at a shared offset — the only construct in C++
    # that gives one struct two names for one address.
    for src in (
        "struct S{union{int i;float f;};int b;};\nint f(S* s){return s->i;}\n",
        "struct S{union{int i;float f;};int b;};\nint f(S* s){return s->b;}\n",
        "struct S{union{int i;float f;};int b;};\nint f(S* s){return s->i + s->b;}\n",
        "struct S{int b;union{int i;float f;};};\nint f(S* s){return s->i + s->b;}\n",
        "struct S{union{int i;int j;};union{int k;int l;};};\n"
        "int f(S* s){return s->i + s->k;}\n",
        "struct S{union{struct{int a;int b;};int i;};};\nint f(S* s){return s->a + s->i;}\n",
        "int f(int a){ union{int i;float f;}; i = a; return i; }\n",
        # a namespace-scope anonymous union: its members ARE namespace-scope
        # objects, so this is `64-data-only-tu.py`'s `.bss` reached through a
        # type. (`f` is renamed: an anonymous union's members are injected into
        # the enclosing scope and would collide with the function.)
        "static union{int i;float d;};\nint fn(int a){return i + a;}\n",
        "static union{int i;float d;};\n",
    ):
        emit(src)

    # ---- F: a union with MEMBERS -------------------------------------------
    for src in (
        "union U{int i;float f;int get();};\nint U::get(){return i;}\n",
        "union U{int i;float f;int get();};\nint U::get(){return (int)f;}\n",
        "union U{int i;float f;int get();};\nint U::get(){return i;}\n" + LEAF % ("z", 1),
        "union U{int i;float f;U();};\nU::U(){i=0;}\n",
        "union U{int i;float f;U();~U();};\nU::U(){i=0;}\nU::~U(){}\n",
        "union U{int i;float f;int get();};\nint f(U* u){return u->get();}\n",
        "union U{int i;float f;static int s;};\nint U::s = 3;\nint f(){return U::s;}\n",
    ):
        emit(src)

    # ---- E/G: THE CONTROLS --------------------------------------------------
    #
    # The struct twin of every accepted union row, plus the plain shapes. Several
    # of these and several of the union rows above are `Port=Match` on this tip,
    # which is what makes the pairing readable at all.
    for src in (
        "struct U{int i;int j;};\nint f(U* u){return u->i;}\n",
        "struct U{int i;char c[4];};\nint f(U* u){return u->i;}\n",
        "struct U{int i;float f;};\nint f(U* u){return u->i;}\n",
        "struct S{int a;struct{int a;int b;}s;};\nint f(S* s){return s->s.b;}\n",
        "struct S{int a;int b;};\nint f(S* s){return s->a + s->b;}\n",
        "int f(int a){return a+1;}\n",
        "int g;\nint f(int a){return g + a;}\n",
        "int g = 1;\nint f(int a){return g + a;}\n",
        "struct U{int i;float f;};\nU g;\nint f(int a){return g.i + a;}\n",
    ):
        emit(src)
