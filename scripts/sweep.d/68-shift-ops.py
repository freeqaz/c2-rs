# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# **THE SHIFT OPERATORS** (lane w-shapes; `scripts/sweep_shapes.py` row
# `shift << >>`, ZERO before this file).
#
# 14,817 generated cases and not one `<<` or `>>`. The corpus has `+`, `-`, `*`,
# `/`, `%`, `&`, `|`, `^`, `~` — 728 cases of bitwise operators alone — and the
# two operators that on PowerPC are a *different instruction family*
# (`rlwinm`/`slw`/`srw`/`srawi`, with the 64-bit `rld*` forms above 32 bits) have
# never been written.
#
# ---- what this row is and is not ----------------------------------------------
#
# Unlike `65`/`66`/`67`, this is **not** a section-shape row and it does not
# reach a new part of the COFF writer. It is a *decode* row, and the honest
# statement of its value is that it costs an afternoon and buys a named
# refusal count:
#
#     $ c2rs census q3.cpp        # int f(int a){return a<<1;}
#       [  0] GAP expr-shl  cflow-straight  eh-none  107 B  ?f@@YAHH@Z
#       1 x expr-shl
#           … 86 41 74 01 >09< 41 86 41 74 3a …
#
# `expr-shl` is a `vocab-gap`: `crates/c2-il` stops at IL opcode `09`. So every
# row here refuses today and will keep refusing until somebody decodes `09`,
# **and that is the point** — the day the decoder learns `09`, this fragment
# becomes ~280 already-written graded cases across every width, amount and
# position instead of the four cases whoever writes it thinks of. The project's
# own rule (`docs/GAPS.md`) is that a hand-picked corpus is biased toward the
# shapes its author was thinking about; this file is the enumeration written
# *before* the widening rather than after it.
#
# ---- the axes are STRUCTURAL ---------------------------------------------------
#
#   A. OPERATOR — `<<` | `>>` | `<<=` | `>>=`. The compound forms are a different
#      IL shape (a read-modify-write against a location), not sugar.
#   B. WIDTH AND SIGNEDNESS of the shifted value — `char`, `unsigned char`,
#      `short`, `unsigned short`, `int`, `unsigned`, `long long`,
#      `unsigned long long`. `>>` on a signed value is an arithmetic shift and on
#      an unsigned one a logical shift: same operator, two instructions.
#   C. SHIFT AMOUNT — 0, 1, 2, 4, 7, 8, 15, 16, 31 as constants (0 folds away, 31
#      is the last defined amount for a 32-bit type), 32 and 63 for the 64-bit
#      rows, a variable, and a variable plus a constant. Amount-as-a-constant and
#      amount-as-a-register are different instructions, and an amount that is a
#      power of two is not special — that is what the non-power rows say.
#   D. POSITION — alone | left of `+` | right of `+` | inside a call argument |
#      compared | stored through a pointer | chained `(a<<a)>>b` | nested
#      `a << (b >> 1)`. The corpus's own history is that operand POSITION is
#      where its value-varying grids missed (`docs/rungs/2026-07-31-cmp-order.md`).
#   E. THE FOLDING CONTROLS — `1 << 3` is folded by the front end and the IL
#      contains the constant 8; it is **`Port=Match` today** (measured, 849 B).
#      `a << 0` is not folded and refuses. `a * 2`, `a * 8`, `a / 2` compile to
#      the same PowerPC shift instructions from IL that is not a shift, and they
#      refuse too — so a widening cannot be graded by "does a shift instruction
#      come out".
#   F. beside a function the port matches, so a whole-TU refusal is separable
#      from a per-function one.

LEAF = "int %s(int a){return a+%d;}\n"

# (C type, a printable tag, the widest defined shift amount)
WIDTHS = (
    ("int",                "i",  31),
    ("unsigned",           "u",  31),
    ("short",              "s",  15),
    ("unsigned short",     "us", 15),
    ("char",               "c",   7),
    ("unsigned char",      "uc",  7),
    ("long long",          "ll", 63),
    ("unsigned long long", "ull", 63),
)
AMOUNTS = (0, 1, 2, 4, 7, 8, 15, 16, 31, 32, 63)


def cases(emit):
    # ---- A x B x C: operator x width x constant amount -----------------------
    #
    # The amount is clamped to the type's own width: shifting an `int` by 32 is
    # undefined and MSVC folds it to something the IL does not represent, which
    # would be a value row masquerading as a structural one.
    for ctype, tag, wmax in WIDTHS:
        for amt in AMOUNTS:
            if amt > wmax:
                continue
            for op in ("<<", ">>"):
                emit("int f(%s a){ return (int)(a %s %d); }\n" % (ctype, op, amt))
        # the compound forms, at three amounts: below, at and above the byte step
        for amt in (1, 7, 8):
            if amt > wmax:
                continue
            for op in ("<<=", ">>="):
                emit("int f(%s a){ %s b = a; b %s %d; return (int)b; }\n"
                     % (ctype, ctype, op, amt))
        # the amount in a REGISTER rather than an immediate
        for op in ("<<", ">>"):
            emit("int f(%s a, int n){ return (int)(a %s n); }\n" % (ctype, op))
            emit("int f(%s a, int n){ return (int)(a %s (n + 1)); }\n" % (ctype, op))

    # ---- D: POSITION in the surrounding expression ---------------------------
    #
    # Held at `int` and amount 1/3 so the only thing varying is where the shift
    # sits. `98-cmp-order.py` and `45-offset-run.py` are on record that operand
    # position is where this project's value-varying grids have missed.
    for op in ("<<", ">>"):
        emit("int f(int a){ return (a %s 1) + 1; }\n" % op)
        emit("int f(int a){ return 1 + (a %s 1); }\n" % op)
        emit("int f(int a, int b){ return (a %s 1) + b; }\n" % op)
        emit("int f(int a, int b){ return b + (a %s 1); }\n" % op)
        emit("int f(int a, int b){ return (a %s 1) + (b %s 2); }\n" % (op, op))
        emit("int f(int a, int b){ return (a %s b) + (b %s a); }\n" % (op, op))
        emit("int f(int a){ return (a %s 1) - (a %s 2); }\n" % (op, op))
        emit("int f(int a){ return (a %s 1) * 3; }\n" % op)
        emit("int f(int a){ return (a %s 1) & 7; }\n" % op)
        emit("int f(int a){ return (a & 7) %s 1; }\n" % op)
        emit("int f(int a){ return (a %s 1) == 4; }\n" % op)
        emit("int f(int a){ if (a %s 1) return 1; return 0; }\n" % op)
        emit("int q(int);\nint f(int a){ return q(a %s 1); }\n" % op)
        emit("int q(int,int);\nint f(int a){ return q(a %s 1, a %s 2); }\n" % (op, op))
        emit("void f(int* p, int a){ *p = a %s 1; }\n" % op)
        emit("int f(int* p, int a){ return p[a %s 1]; }\n" % op)
        emit("int g;\nvoid f(int a){ g = a %s 1; }\n" % op)
        # chained and nested — a single-shift decoder gets these wrong for a
        # different reason than it gets one shift wrong.
        emit("int f(int a, int b){ return (a %s 1) %s 2; }\n" % (op, op))
        emit("int f(int a, int b){ return a %s (b %s 1); }\n" % (op, op))
        emit("int f(int a, int b, int c){ return ((a %s b) %s c) %s 1; }\n" % (op, op, op))
        # beside a function the port matches (axis F)
        emit("int f(int a){ return a %s 1; }\n" % op + LEAF % ("z", 1))
        emit(LEAF % ("z", 1) + "int f(int a){ return a %s 1; }\n" % op)

    # the two operators MIXED — a decoder that handles each alone can still get
    # the pair's operand order wrong, which is exactly how `98-cmp-order` began.
    emit("int f(int a){ return (a << 3) >> 1; }\n")
    emit("int f(int a){ return (a >> 3) << 1; }\n")
    emit("int f(int a, int b){ return (a << b) >> b; }\n")
    emit("int f(int a, int b){ return (a >> b) << b; }\n")
    emit("int f(int a, int b){ return (a << 1) + (b >> 1); }\n")
    emit("int f(int a, int b){ return (a >> 1) + (b << 1); }\n")

    # ---- E: THE FOLDING CONTROLS --------------------------------------------
    #
    # `1 << 3` is folded before the IL is written and is `Port=Match` today
    # (measured 849 B). The rest reach the emitter through arithmetic that is not
    # a shift and produce shift *instructions*, so "a shift came out of the
    # emitter" is not a test for this row.
    for src in (
        "int f(int a){ return (1 << 3) + a; }\n",
        "int f(int a){ return (256 >> 5) + a; }\n",
        "int f(int a){ return 1 << 3; }\n",
        "int f(){ return 1 << 3; }\n",
        "int f(int a){ return a * 2; }\n",
        "int f(int a){ return a * 8; }\n",
        "int f(int a){ return a * 2 + 1; }\n",
        "int f(int a){ return a / 2; }\n",
        "unsigned f(unsigned a){ return a / 2; }\n",
        "unsigned f(unsigned a){ return a * 4; }\n",
        "int f(int a){ return a + a; }\n",
        "int f(int a){ return a & 7; }\n",
        "int f(int a){ return a | 8; }\n",
        "int f(int a){ return a ^ 3; }\n",
        "int f(int a){ return ~a; }\n",
        "int f(int a){ return a + 1; }\n",
        "int f(int a){ return a + 1; }\n" + LEAF % ("z", 1),
    ):
        emit(src)
