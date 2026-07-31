# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# ---- W37: the SOURCE LINE as an axis, across every byte it can collide with --
#
# The line marker is `4F 01 <varint line>`, and its payload is the one field in
# the pre-body region whose value is chosen by the *programmer* rather than by
# the compiler. That makes it the cheapest possible source of a byte that looks
# like an opcode, and this project has already paid for it once:
# `GAPS.md` §6's **first** live wrong-bytes emit is a member function on source
# line **70**, whose marker reads `4F 01 46` — and `0x46` is the formals marker.
# The `this` lookup anchored on a bare first-`0x46` search, found that one, and
# every formal dropped a register.
#
# What exists today is `44-member-source-lines.py`, which sweeps lines 66..74 for
# one member shape. That is a window around the bug that was found, which is
# exactly the shape `GAPS.md` §6 warns about under "a rule fitted to the shapes
# the corpus happened to contain": the axis was swept where the defect was known
# to be, not across its range. Two whole regions are unswept:
#
#  * **every OTHER byte value that means something structural.** `0x4C` is `LO`,
#    `0x53` opens a scope, `0x54` closes one, `0x29` is RETURN, `0x3A` is the
#    branch, `0x41` is the result-type annotation, `0x2D` is a formals entry,
#    `0x26` is a symbol push, `0x4F` is the marker's own opcode, `0x30`/`0x32`
#    are the indirect load and store, `0xB9`/`0xBD` are LOAD and CALL. Each is a
#    line number, each is reachable by moving a definition down the file, and
#    none of them has ever been compiled. A parser that scans for any of those
#    bytes rather than parsing to them has the line-70 defect in a second place.
#  * **the varint WIDTH boundary at 127/128.** `eat_fn_tail`'s own comment
#    records that the module-end marker's payload "is four bytes longer past line
#    127", so the marker changes width mid-corpus. Every fixture in this repo is
#    a short file: nothing has ever put an accepted function past line 127, so no
#    accepted shape has ever been graded with a multi-byte line marker anywhere
#    in its segment. A fixed-width skip over that field is invisible until it is
#    not (`GAPS.md` §6: "a field the port skips is indistinguishable from a field
#    that is always the same").
#
# This varies **no operator and no shape** — the emitted code is byte-identical
# whatever line it sits on, which is the whole point: every case here has a known
# right answer, so a mismatch is unambiguous. It is crossed with the accepted
# shape families rather than run on one, because the anchor that broke is
# consulted by the params reader that *every* shape goes through, and §6's
# instance #2 is precisely "fixed in the one shape where the bug had been found".


def cases(emit):
    # Byte values that mean something structural in `.ex`, as line numbers. The
    # comment on each is what the byte is when the parser meets it elsewhere.
    COLLIDING = [
        0x26,  # 38  symbol push / assignment destination
        0x29,  # 41  RETURN
        0x2D,  # 45  one formals-list entry
        0x30,  # 48  indirect load
        0x32,  # 50  store
        0x33,  # 51  literal
        0x3A,  # 58  unconditional branch
        0x41,  # 65  result-type annotation
        0x46,  # 70  the formals marker — the known live mis-emit
        0x47,  # 71  function-tail opener
        0x4B,  # 75  statement end
        0x4C,  # 76  LO / call apply
        0x4D,  # 77  module end
        0x4F,  # 79  the marker opcode itself
        0x53,  # 83  scope open
        0x54,  # 84  scope close
        0x55,  # 85  call-argument terminator
        0x99,  # 153 member bind
        0xB9,  # 185 LOAD
        0xBD,  # 189 CALL
    ]
    # …and the varint width boundary, which nothing in the corpus crosses.
    BOUNDARY = [126, 127, 128, 129, 130, 255, 256]

    # One representative of each accepted family, written so that the *first*
    # definition lands on the swept line. Each is a shape the port emits bytes
    # for, so a wrong anchor shows up as a MISMATCH rather than as silence.
    #
    # Every one takes at least TWO formals and uses the second, because that is
    # what an empty or short formals list actually corrupts: `leaves_ascending`
    # skips tokens it does not recognize as formals, so a body whose formals
    # vanished bypasses the ordering gate silently rather than refusing.
    SHAPES = [
        # straight-line int chain, member function — the line-70 shape
        ("struct C { int m; int gp(int a, int b) const; };\n",
         "int C::gp(int a, int b) const { return a + b; }\n"),
        # …and the free-function twin, which the original fixture does not have
        ("", "int f_sl(int a, int b) { return a + b; }\n"),
        # indirect load through the SECOND formal
        ("", "int f_ld(int a, int *q) { return *q; }\n"),
        # address of a sub-object
        ("struct S { int a; int b; };\n",
         "int *f_ad(int k, S *s) { return &s->b; }\n"),
        # store leaf
        ("struct S { int a; int b; };\n",
         "void f_st(S *s, int v) { s->b = v; }\n"),
        # tail call, second formal as the argument
        ("int g(int);\n", "int f_tc(int a, int b) { return g(b); }\n"),
        # member call as a whole body (W36)
        ("struct Obj { int i; void v1(int); };\n",
         "void f_mc(Obj *o, int k) { o->v1(k); }\n"),
        # a framed non-leaf call
        ("int g(int);\n", "int f_fr(int a) { return g(a) + 1; }\n"),
        # comparison leaf
        ("", "int f_cm(int a, int b) { return a < b; }\n"),
        # float leaf — the other register file
        ("", "float f_fp(float a, float b) { return a * b; }\n"),
    ]

    def at_line(head, body, line):
        # `head` sits at the top; the body's own first line is padded down to
        # `line`. One `// pad` per line, so the count is exact and readable in the
        # generated case.
        used = head.count("\n")
        pad = "\n".join("// pad %d" % i for i in range(1, line - used))
        return head + pad + "\n" + body

    for line in COLLIDING:
        for head, body in SHAPES:
            emit(at_line(head, body, line))

    # The width boundary needs fewer shapes but MORE than one function in the TU,
    # because the interesting case is a second function whose markers are already
    # multi-byte when the first one's were not — the same "two facts sharing one
    # field" shape as the label counter (`GAPS.md` §6 #12/#13).
    for line in BOUNDARY:
        for head, body in SHAPES[:5]:
            src = at_line(head, body, line)
            emit(src)
            emit(src + "int f_tail(int a, int b) { return a - b; }\n")

    # A function whose marker crosses the boundary *between* two accepted ones, so
    # the multi-byte field sits in the middle of the segment run rather than at
    # its start or end.
    head = "int g(int);\n"
    pad = "\n".join("// pad %d" % i for i in range(1, 120))
    emit(head + "int f_a(int a, int b) { return a + b; }\n" + pad + "\n"
         "int f_b(int a, int b) { return g(b); }\n"
         "int f_c(int a, int b) { return a - b; }\n")
