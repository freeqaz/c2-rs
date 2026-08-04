# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# **THE REAL EH TABLE** (lane w-shapes; `scripts/sweep_shapes.py` rows `try/catch`
# and `throw`, both ZERO before this file).
#
# `scripts/lanes.txt` crosses `/EHsc` over all six code-shape configurations and
# `scripts/mode_cross.sh` grades that cross against the generated corpus — 63,723
# cells. Every one of those `/EHsc` cells is graded through an **implicit unwind
# action**: a local with a destructor, a base-class cleanup, the `26` separator.
# **Not one of them has ever contained a `try`, a `catch` or a `throw**, because no
# fragment writes one. So the axis the registry exists to cover is covered by its
# cheapest half only.
#
# MEASURED here (`scripts/gt_capture.sh`, section names out of the COFF header):
#
#     int f(int a){ if(a) throw 1; return a+1; }
#       /Ox /GS- /c        .text .xdata$x .pdata .xdata$x .xdata$x .data
#       /O1 /Oi /EHsc      .text .xdata$x .pdata .xdata$x .xdata$x .data .text
#
#     int h(); int f(int a){ try{ return h(); } catch(int e){ return e; } }
#       /Ox /GS- /c        .text .pdata .rdata .data
#       /O1 /Oi /EHsc      .text .pdata .pdata .rdata .data
#
# `.xdata$x` is **622 sections across 67 objs** of the dc3 workload
# (`work/w-bss/census/sections.jsonl`) and is the LAST step of factor C's ladder —
# the one that takes C from 804 to 871 (`docs/STATUS.md`). Before this fragment the
# corpus could not produce a single one, so "the port refuses `.xdata$x`" was an
# assertion with no case behind it. It is now 0 cases -> the count this file emits.
#
# Note the second measured line carefully: `try/catch` and `throw` do **not**
# produce the same sections. A fragment that wrote only `throw` would leave
# `try`'s `.rdata` + `.data` pair unreached, and one that wrote only `try` would
# never emit `.xdata$x` at all. They are two rows in `sweep_shapes.py` because
# they are two shapes in the obj.
#
# ---- the axes are STRUCTURAL ---------------------------------------------------
#
# The thrown/caught VALUES are held at `1` and `a` almost everywhere; three grids
# on this project have varied values exhaustively and missed on arity, register
# position and structural counts (`63-emit-order.py`, `64-data-only-tu.py`). The
# cross is over the shape of the EH region:
#
#   A. CONSTRUCT — `throw` alone | conditional `throw` | `try`/`catch(...)` |
#      `try`/typed `catch` | rethrow | function-try-block | `try` inside `catch` |
#      nested `try`. Each is a different `maxState` and a different funclet count.
#   B. HANDLER COUNT — 1, 2, 3. The handler array is length-prefixed, so a
#      single-handler grid cannot separate "the count" from "the first entry".
#   C. NESTING DEPTH — 1, 2, 3. `maxState` is what `c2rs census` reports as the
#      EH class and every generated case in the corpus reads `maxState 0`.
#   D. THROWN TYPE — `int`, `double`, a pointer, a POD struct, a struct with a
#      destructor, a derived struct. The last two are the ones that mint
#      `__TI`/`__CT` records and a copy-constructor reference.
#   E. UNWOUND OBJECT COUNT — 0, 1, 2 destructible locals live across the
#      protected region. This is the axis that turns one `.xdata$x` into three.
#   F. TU POSITION — the EH function alone, first, last, and wedged between two
#      ordinary ones. `63-emit-order.py` showed the `.text` emission order is a
#      dependency walk; an EH function carries extra symbols and this crosses the
#      two.
#   G. THE CONTROLS — the identical TU with the EH construct removed. Without
#      them a fix that refuses any TU mentioning a class with a destructor would
#      pass this fragment while losing shapes the port matches today.
#
# ---- what must NOT happen ------------------------------------------------------
#
# `NotImplemented` is the contract here and is expected on nearly every row: the
# port's COFF writer has no `.xdata$x` and no funclet model. A `Port=Mismatch` is
# the alarm — it means the writer emitted an obj for a TU whose EH tables it
# cannot have placed.
#
# Every case must COMPILE. `/Ox` without `/EHsc` warns C4530 on a `try`, which is
# a warning and produces an obj; the sweep's `ungraded` baseline must not move.

# A destructible local: declared, never defined — `/c` only, so no link step.
DTOR = "struct %s{%s();~%s();int m;};\n"
# An ordinary function the port matches on its own, for wedging.
LEAF = "int %s(int a){return a+%d;}\n"

# (thrown expression, the declarations it needs, a catch clause that matches it)
THROWN = (
    ("1",            "",                                   "int e"),
    ("2.0",          "",                                   "double e"),
    ("(int*)0",      "",                                   "int* e"),
    ("P()",          "struct P{int a;};\n",                "P e"),
    ("P()",          "struct P{int a;};\n",                "const P& e"),
    ("Q()",          "struct Q{Q();Q(const Q&);~Q();int a;};\n", "const Q& e"),
    ("R()",          "struct P{int a;};\nstruct R:P{int b;};\n", "const P& e"),
    ("(P*)0",        "struct P{int a;};\n",                "P* e"),
)

# Catch clauses that do NOT name the thrown type, so the handler array has
# entries the throw never selects. The count is what matters, not the match.
EXTRA_CATCH = (
    "catch(char e){return 7;}",
    "catch(long e){return 8;}",
    "catch(short e){return 9;}",
)


def cases(emit):
    # ---- A x D: THROW, every thrown type, conditional and unconditional ------
    #
    # `throw` alone is the shape that mints `.xdata$x`. It needs no `try` and no
    # handler, which is why it is a separate row from `try/catch`.
    for expr, decls, _c in THROWN:
        emit(decls + "int f(int a){ if(a) throw %s; return a+1; }\n" % expr)
        emit(decls + "int f(int a){ throw %s; }\n" % expr)
        # …beside a function the port matches on its own (axis F).
        emit(decls + "int f(int a){ if(a) throw %s; return a+1; }\n" % expr
             + LEAF % ("z", 1))
        emit(LEAF % ("z", 1)
             + decls + "int f(int a){ if(a) throw %s; return a+1; }\n" % expr)
        # …and wedged between two, so the emission-order walk has to place an
        # EH-bearing function in the middle.
        emit(LEAF % ("y", 2)
             + decls + "int f(int a){ if(a) throw %s; return a+1; }\n" % expr
             + LEAF % ("z", 1))

    # ---- A x B: TRY/CATCH, handler count 1..3 --------------------------------
    #
    # The protected region calls an EXTERNAL function: at `/O1 /EHsc` the
    # compiler proves a pure expression cannot throw and deletes the whole
    # region, so a `try` around `a+1` is a control, not an EH case. Measured:
    # `try{return a+1;}catch(...){return 0;}` compiles to the same 845-byte obj
    # as `int f(int a){return a+1;}` at `/O1 /Oi /EHsc`.
    for expr, decls, catchdecl in THROWN:
        for nextra in (0, 1, 2, 3):
            handlers = "catch(%s){return 3;}" % catchdecl
            handlers += "".join(EXTRA_CATCH[:nextra])
            emit("int h();\n" + decls
                 + "int f(int a){ try{ return h(); } %s }\n" % handlers)
        # `catch(...)` on its own, and after the typed one — the ellipsis handler
        # is encoded differently from a typed entry.
        emit("int h();\n" + decls
             + "int f(int a){ try{ return h(); } catch(...){ return 4; } }\n")
        emit("int h();\n" + decls
             + "int f(int a){ try{ return h(); } catch(%s){ return 3; }"
               " catch(...){ return 4; } }\n" % catchdecl)

    # ---- C: NESTING DEPTH 1..3, and `try` inside `catch` ---------------------
    #
    # Depth is `maxState`. Every generated case in the corpus reads `maxState 0`;
    # these are the only cases that do not.
    body = "return h();"
    for depth in (1, 2, 3):
        body = "try{ %s } catch(...){ return %d; }" % (body, depth)
        emit("int h();\nint f(int a){ %s return a; }\n" % body)
        emit("int h();\nint f(int a){ %s return a; }\n" % body + LEAF % ("z", 1))
    # a `try` in the HANDLER rather than in the body — the state transition runs
    # the other way.
    emit("int h();\nint f(int a){ try{ return h(); }"
         " catch(...){ try{ return h(); } catch(...){ return 2; } } }\n")
    # rethrow: a bare `throw;` inside a handler.
    emit("int h();\nint f(int a){ try{ return h(); } catch(...){ throw; } }\n")
    emit("int h();\nint f(int a){ try{ return h(); }"
         " catch(int e){ if(e) throw; return e; } catch(...){ throw; } }\n")
    # function-try-block — the protected region is the whole body, including the
    # parameter copies.
    emit("int h();\nint f(int a) try { return h(); } catch(...) { return 0; }\n")
    emit("struct S{S();~S();int m;};\n"
         "struct T{T();~T();S s;};\nT::T() try : s() {} catch(...) {}\n")

    # ---- E: UNWOUND OBJECT COUNT — 0, 1, 2 destructible locals ---------------
    #
    # This is the axis that turns one unwind state into several. The `26`
    # separator the corpus already has covers the ZERO-`try` half of it; these
    # rows put the same destructors inside a protected region.
    decls2 = DTOR % ("A", "A", "A") + DTOR % ("B", "B", "B")
    for nloc in (0, 1, 2):
        locs = "".join(("A x;", "B y;")[:nloc])
        emit("int h();\n" + decls2
             + "int f(int a){ %s try{ return h(); } catch(...){ return 0; } }\n" % locs)
        emit("int h();\n" + decls2
             + "int f(int a){ try{ %s return h(); } catch(...){ return 0; } }\n" % locs)
        emit("int h();\n" + decls2
             + "int f(int a){ try{ return h(); } catch(...){ %s return 0; } }\n" % locs)
        emit("int h();\n" + decls2
             + "int f(int a){ %s if(a) throw 1; return h(); }\n" % locs)
    # a destructible local in a LOOP body inside the protected region: the same
    # state entered and left repeatedly.
    emit("int h();\n" + decls2
         + "int f(int a){ try{ for(int i=0;i<a;i++){ A x; h(); } } catch(...){ return 0; } return a; }\n")

    # ---- F: TU POSITION x the ordinary functions the port already matches ----
    #
    # Two, three and four functions in the TU with the EH-bearing one in every
    # position. `63-emit-order.py` established that `.text` is emitted in a
    # dependency walk and not in `.ex` order; an EH function adds `.pdata` and
    # `.xdata$x` entries whose order is a second, independent sequence.
    EH = "int f(int a){ if(a) throw 1; return %s; }\n"
    EHT = "int f(int a){ try{ return %s; } catch(...){ return 0; } }\n"
    for eh in (EH, EHT):
        pre = "int h();\n" if eh is EHT else ""
        body, sib = eh % "a+1", (eh % "a+1").replace("int f(", "int g(")
        if eh is EHT:
            body, sib = eh % "h()", (eh % "h()").replace("int f(", "int g(")
        emit(pre + body + LEAF % ("z", 1))
        emit(pre + LEAF % ("z", 1) + body)
        emit(pre + body + LEAF % ("z", 1) + LEAF % ("y", 2))
        emit(pre + LEAF % ("z", 1) + body + LEAF % ("y", 2))
        emit(pre + LEAF % ("z", 1) + LEAF % ("y", 2) + body)
        # two EH-bearing functions — the tables must not be shared or reordered.
        emit(pre + body + sib)
        emit(pre + body + sib + LEAF % ("z", 1))
        emit(pre + LEAF % ("z", 1) + body + sib)
        # an EH function that CALLS a locally defined one — the dependency edge
        # `63-emit-order.py` grades, crossed with an EH region. Both orders, with
        # a forward declaration so the caller-first row compiles.
        emit(LEAF % ("z", 1) + eh % "z(a)")
        emit("int z(int);\n" + (eh % "z(a)") + LEAF % ("z", 1))

    # ---- D again: a thrown class with a DESTRUCTOR beside one without --------
    #
    # `__CT` records reference a copy constructor and a destructor by symbol; a
    # thrown POD references neither. Both in one TU is the cell that separates
    # "the record exists" from "the record has the right arity".
    emit("struct P{int a;};\nstruct Q{Q();Q(const Q&);~Q();int a;};\n"
         "int f(int a){ if(a) throw P(); throw Q(); }\n")
    emit("struct P{int a;};\nstruct Q{Q();Q(const Q&);~Q();int a;};\n"
         "int f(int a){ if(a) throw Q(); throw P(); }\n")
    emit("struct P{int a;};\nstruct R:P{int b;};\n"
         "int f(int a){ if(a) throw R(); throw P(); }\n")
    # the SAME type thrown twice — one record, not two.
    emit("struct P{int a;};\nint f(int a){ if(a) throw P(); throw P(); }\n")
    emit("int f(int a){ if(a) throw 1; throw 2; }\n")
    # thrown from a function that also catches.
    emit("int h();\nstruct P{int a;};\n"
         "int f(int a){ try{ return h(); } catch(...){ throw P(); } }\n")

    # ---- G: THE CONTROLS ----------------------------------------------------
    #
    # Each is a TU this fragment's rows are built from, with the EH construct
    # deleted. They carry no `.xdata$x`, several are `Port=Match` today, and a
    # fix that refused every TU containing a destructor-bearing class would lose
    # them. A fragment of positives only cannot tell a rule from a blanket
    # refusal.
    for src in (
        "int f(int a){ return a+1; }\n",
        "int h();\nint f(int a){ return h(); }\n",
        "int f(int a){ return a+1; }\n" + LEAF % ("z", 1),
        LEAF % ("z", 1) + "int f(int a){ return a+1; }\n",
        LEAF % ("y", 2) + "int f(int a){ return a+1; }\n" + LEAF % ("z", 1),
        # a destructible local WITHOUT a protected region: the implicit-unwind
        # shape the corpus already covers, kept here so this fragment's EH rows
        # are read against it rather than against nothing.
        "int h();\n" + decls2 + "int f(int a){ A x; return h(); }\n",
        "int h();\n" + decls2 + "int f(int a){ A x; B y; return h(); }\n",
        # a `try` the optimizer deletes because the region provably cannot
        # throw. Measured identical to the plain function at `/O1 /Oi /EHsc`;
        # at `/Ox` it is not. That disagreement across the mode cross is the
        # point of the row.
        "int f(int a){ try{ return a+1; } catch(...){ return 0; } }\n",
        "int f(int a){ try{ return a+1; } catch(...){ return 0; } }\n" + LEAF % ("z", 1),
        # classes declared but never thrown or caught.
        "struct P{int a;};\nstruct Q{Q();Q(const Q&);~Q();int a;};\n"
        "int f(int a){ return a+1; }\n",
    ):
        emit(src)
