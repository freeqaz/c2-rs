# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# **MORE THAN ONE SOURCE FILE** (lane w-shapes; `scripts/sweep_shapes.py` row
# `#include`, ZERO before this file).
#
# Every one of the 14,817 generated cases is a single self-contained `.cpp` with
# no `#include` and no `#line`, so **every function in the corpus has always come
# from file index 0** and the IL's file record has never been anything but the
# one the compiler opened. Real translation units are almost entirely the other
# thing: the dc3 workload's 878 TUs are headers.
#
# The corpus was written that way for a good reason — the toolchain under
# `compilers/` ships `cl.exe` and no `include/` directory, so `#include <…>` has
# nothing to open. There are two ways to a second file record anyway, and this
# fragment uses both: `#include __FILE__` behind a guard (a genuine second file,
# opened by the preprocessor) and `#line N "name"` (a synthetic one). They are
# different rows because only the first is a real file.
#
# ---- the measurement that says this row is not decoration ----------------------
#
#     int f(int a){return a+1;}                      ->  Port=Match
#
#     #line 100 "other.cpp"
#     int f(int a){return a+1;}                      ->  Port=NotImplemented
#
#     $ c2rs census line1.cpp
#     0/1 functions in class
#       [  0] GAP module-end-0x4F  cflow-straight+expr-modeled  107 B  ?f@@YAHH@Z
#           … 54 01 54 00 >4f< 02 e3 09 4f 01 65 4d
#
# **One line that changes no code flips the verdict.** The body is byte-identical
# and still named, and the bundle now carries a second `4f` module record that
# `crates/c2-il` stops on. `44-member-source-lines.py` and
# `46-source-line-collisions.py` vary line NUMBERS inside one file; nothing in
# the corpus has ever varied the FILE, so `module-end-0x4F` has never been
# reachable and the refusal has never been graded.
#
# ---- the axes ------------------------------------------------------------------
#
#   A. MECHANISM — `#include __FILE__` (real) | `#line N "name"` (synthetic) |
#      `#line N` with no name (renumber only, no new file). The third is the
#      control that separates "a new file" from "a discontinuity in the numbers".
#   B. WHERE the other file's content sits — entirely before the first function,
#      between two functions, after all of them.
#   C. HOW MANY functions come from each file — 1/0, 0/1, 1/1, 2/1, 1/2. The file
#      record is per-module, and a grid where every file contributes one function
#      cannot separate the record from the function.
#   D. THE `#line` NUMBER relative to the current one — smaller, equal, larger,
#      and far larger. Line positions are deltas in the IL, so a backwards jump
#      is a different encoding from a forwards one, not the same one negated.
#   E. THE FILE NAME'S LENGTH — the name is written into the record; short and
#      long names are different record lengths.
#   F. A `#line` INSIDE a function body, between two statements — the record
#      lands in the middle of a body rather than at a module boundary.
#   G. A CROSS-FILE REFERENCE — a function defined in the included half and
#      called from the including half, so `63-emit-order.py`'s dependency walk
#      has to order two functions that came from different files.
#   H. THE CONTROLS — the identical TUs with no `#include` and no `#line`.
#      Several are `Port=Match` today, which is the whole reason the flip above
#      is readable.
#
# `#include __FILE__` behind a guard opens the case file exactly twice and
# terminates; nothing is written to disk and no header is needed. Ordering
# matters: a `#line` that renames the file must come AFTER any
# `#include __FILE__`, or the include names a file that does not exist.

LEAF = "int %s(int a){return a+%d;}\n"


def selfinc(inner, outer):
    """A TU whose `inner` half is preprocessed as a second file."""
    return ("#ifndef W_SHAPES_SELF\n#define W_SHAPES_SELF\n#include __FILE__\n"
            + outer + "#else\n" + inner + "#endif\n")


def cases(emit):
    # ---- A x C: `#include __FILE__`, every split of the functions ------------
    F = "int f(int a){return a+1;}\n"
    G = "int g(int a){return a+2;}\n"
    emit(selfinc("", F))                       # 1 / 0
    emit(selfinc(F, ""))                       # 0 / 1
    emit(selfinc(G, F))                        # 1 / 1
    emit(selfinc(F, G))                        # 1 / 1, the other way round
    emit(selfinc(G + LEAF % ("z", 1), F))      # 1 / 2
    emit(selfinc(G, F + LEAF % ("z", 1)))      # 2 / 1
    emit(selfinc(G + LEAF % ("z", 1), F + LEAF % ("y", 4)))   # 2 / 2
    # data, not functions, from the other file.
    emit(selfinc("int gv;\n", "int f(int a){return gv+a;}\n"))
    emit(selfinc("int gv = 1;\n", "int f(int a){return gv+a;}\n"))
    emit(selfinc("struct S{int a;};\n", "int f(S* s){return s->a+1;}\n"))
    # declarations only from the other file — a real header's commonest shape.
    emit(selfinc("int q(int);\n", "int f(int a){return q(a);}\n"))
    emit(selfinc("struct S{int a;int m(int);};\n", "int S::m(int a){return this->a+a;}\n"))

    # ---- G: a CROSS-FILE reference, both directions --------------------------
    #
    # `63-emit-order.py` established that `.text` order is a dependency walk over
    # the functions the TU defines. Here the two ends of the edge come from
    # different files.
    emit(selfinc("int g(int a){return a+2;}\n", "int f(int a){return g(a);}\n"))
    emit(selfinc("int g(int a){return a+2;}\n", "int f(int a){return g(a)+1;}\n"))
    emit(selfinc("int g(int);\n", "int f(int a){return g(a);}\nint g(int a){return a+2;}\n"))
    emit(selfinc("struct B{B();~B();int x;};\nstruct D:B{D();};\n", "D::D(){}\nB::~B(){}\n"))
    emit(selfinc("int g(int a){return a+2;}\n" + LEAF % ("z", 1),
                 "int f(int a){return g(a)+z(a);}\n"))

    # ---- A x B x D x E: `#line`, every position and every jump ---------------
    #
    # Names of three lengths, because the name is written into the record.
    NAMES = ('"h.h"', '"other.cpp"',
             '"a_rather_long_header_name_for_the_record.h"')
    for name in NAMES:
        for n in (1, 2, 100, 30000):
            emit("#line %d %s\nint f(int a){return a+1;}\n" % (n, name))
        # between two functions, and after both.
        emit("int f(int a){return a+1;}\n#line 200 %s\nint g(int a){return a+2;}\n" % name)
        emit("int f(int a){return a+1;}\nint g(int a){return a+2;}\n#line 200 %s\n" % name)
        # two switches: back to the original name is a THIRD record, not a
        # cancellation of the second.
        emit("#line 100 %s\nint f(int a){return a+1;}\n#line 300 \"back.cpp\"\n"
             "int g(int a){return a+2;}\n" % name)
    # a BACKWARDS jump, an exact repeat, and a forwards one, each after a
    # function so the delta has a base to be relative to.
    for n in (1, 2, 3, 500):
        emit("int f(int a){return a+1;}\n#line %d \"h.h\"\nint g(int a){return a+2;}\n" % n)
    # renumber with NO file name — the control that separates a new FILE from a
    # discontinuity in the numbers.
    for n in (1, 100, 30000):
        emit("#line %d\nint f(int a){return a+1;}\n" % n)
        emit("int f(int a){return a+1;}\n#line %d\nint g(int a){return a+2;}\n" % n)

    # ---- F: a `#line` INSIDE a body -----------------------------------------
    #
    # The record lands between two statements rather than at a module boundary,
    # which is a different place in the bundle entirely.
    emit("int f(int a){\nint b = a + 1;\n#line 100 \"h.h\"\nreturn b + 1;\n}\n")
    emit("int f(int a){\n#line 100 \"h.h\"\nint b = a + 1;\nreturn b + 1;\n}\n")
    emit("int f(int a){\nint b = a + 1;\n#line 100\nreturn b + 1;\n}\n")
    emit("int q(int);\nint f(int a){\nint b = q(a);\n#line 100 \"h.h\"\nreturn b + 1;\n}\n")
    emit("int f(int a){\nint b = a + 1;\n#line 100 \"h.h\"\nint c = b + 2;\n"
         "#line 200 \"i.h\"\nreturn c + 3;\n}\n")

    # ---- A: both mechanisms in one TU ---------------------------------------
    #
    # The `#line` must come after the `#include __FILE__`: it renames `__FILE__`
    # and the include would then name a file that does not exist.
    emit(selfinc("int g(int a){return a+2;}\n",
                 "#line 100 \"h.h\"\nint f(int a){return g(a);}\n"))
    emit(selfinc("#line 50 \"inner.h\"\nint g(int a){return a+2;}\n",
                 "int f(int a){return g(a);}\n"))

    # ---- H: THE CONTROLS ----------------------------------------------------
    #
    # Every one is a row above with the directive deleted. `int f(int a){return
    # a+1;}` is `Port=Match`; the same TU with one `#line` in front of it is
    # `Port=NotImplemented` with blocker `module-end-0x4F`, and that pair is what
    # this fragment exists to hold.
    for src in (
        "int f(int a){return a+1;}\n",
        "int f(int a){return a+1;}\nint g(int a){return a+2;}\n",
        "int f(int a){return a+1;}\nint g(int a){return a+2;}\n" + LEAF % ("z", 1),
        "int q(int);\nint f(int a){return q(a);}\n",
        "int gv;\nint f(int a){return gv+a;}\n",
        "struct S{int a;};\nint f(S* s){return s->a+1;}\n",
        "struct S{int a;int m(int);};\nint S::m(int a){return this->a+a;}\n",
        "int g(int a){return a+2;}\nint f(int a){return g(a);}\n",
        "struct B{B();~B();int x;};\nstruct D:B{D();};\nD::D(){}\nB::~B(){}\n",
        "int f(int a){\nint b = a + 1;\nreturn b + 1;\n}\n",
    ):
        emit(src)
