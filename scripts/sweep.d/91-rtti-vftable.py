# Sweep fragment — see scripts/expr_sweep.sh for the contract.
#
# **RTTI, `.rdata$r`, AND THE VFTABLE THAT MINTS IT** (lane w-gr, task #40;
# `scripts/sweep_shapes.py` rows `dynamic_cast`, `typeid`, `virtual inheritance`
# and `vftable-emitting ctor/dtor`, all ZERO before this file).
#
# `.rdata$r` is the **last** workload section name the generated corpus could not
# produce: **24,163 sections across 676 of 871 objs**
# (`work/w-bss/census/sections.jsonl`), and factor C's largest single step,
# 169 -> 590 (`docs/STATUS.md`). The workload's own flag string
# (`work/dc3-workload/flags.txt`) reads
#
#     /nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc /I …
#
# and until 2026-08-04 **not one of the lane registry's 12 lanes passed `/GR`**.
# That is the `/EHsc` hole verbatim, one flag over — and the `/EHsc` version of it
# produced board **#263**, a live wrong emit at the workload's own flags that
# every standing instrument was blind to because nothing graded the axis.
#
# ---- the two facts this fragment is built on, both MEASURED -------------------
#
# **1. `/GR` is NOT the default for this compiler.** MSVC's documented host
# default is `/GR` on; this `cl.exe` (X360 16.00.11886.00, under wibo) behaves as
# `/GR-` unless `/GR` is passed. Measured, non-boilerplate sections:
#
#     struct B{virtual ~B();virtual int f();int b;}; B::~B(){} int B::f(){…}
#       /O1 /Oi /EHsc         .text .rdata .text .pdata .text
#       /O1 /Oi /EHsc /GR     .text .rdata .rdata$r .data .rdata$r .rdata$r
#                             .rdata$r .text .pdata .text
#       /O1 /Oi /EHsc /GR-    (identical to the first line)
#
# So **this fragment alone cannot close the gap** and neither can the lane alone:
# the corpus needs RTTI *shapes* and the registry needs a `/GR` *lane*. w-gr adds
# both in one commit, and `scripts/lanes.txt` is where the second half lives.
#
# **2. The trigger is the VFTABLE, not the cast.** A 15-shape grid at
# `/O1 /Oi /EHsc /GR` (lane w-gr §1.2):
#
#     polymorphic class, DESTRUCTOR defined here      -> .rdata$r  (4 records)
#     polymorphic class, pure virtual + dtor here     -> .rdata$r
#     polymorphic class, only `int B::f(){…}` here    -> none
#     base+derived, both f() defined, no ctor/dtor    -> none
#     dynamic_cast<D*> / <D&> / <void*>               -> none  (obj is
#                                                       content-identical with
#                                                       and without /GR)
#     typeid(*p)                                      -> none
#     multiple / virtual inheritance, no ctor/dtor    -> none
#
# The vftable is emitted in the TU that generates a **constructor or destructor
# body**, because that body is what writes the vfptr; the RTTI records hang off
# the vftable. `dynamic_cast` and `typeid` reference `??_R0` type descriptors,
# which land in **`.data`** (they hold a mutable name buffer), so they move
# `.data` and `.text` and leave `.rdata$r` empty.
#
# **A fragment written from the obvious mental model — "RTTI means `dynamic_cast`
# and `typeid`" — would have produced ZERO `.rdata$r` cases and read as success.**
# That is `docs/STATUS.md` trap 5 with a plausible cover story, and it is why the
# axis order below puts the emission trigger first and the casts fourth.
#
# ---- the axes are STRUCTURAL --------------------------------------------------
#
#   A. VFTABLE-EMISSION TRIGGER — dtor defined here | ctor defined here | both |
#      neither (the control that produces no RTTI at all) | a ctor forced by a
#      local object | an out-of-line member that constructs a temporary. This is
#      the axis that decides whether `.rdata$r` exists.
#   B. HIERARCHY SHAPE — standalone | single inheritance depth 1, 2, 3 | multiple
#      inheritance with 2 and 3 bases | MI where one base is NOT polymorphic |
#      virtual inheritance | a diamond. This decides the LENGTH of the `??_R2`
#      base-class array and the count of `??_R1` descriptors, i.e. how many
#      `.rdata$r` sections the obj has and how big each is. A one-class grid
#      cannot separate "the array exists" from "the array has the right arity" —
#      the mistake `63-emit-order.py` and `64-data-only-tu.py` both recorded.
#   C. POLYMORPHIC CLASSES PER TU — 1, 2, 3. Each mints its own `??_R4`/`??_R3`/
#      `??_R2`/`??_R1` block, so this is what turns 4 `.rdata$r` sections into 8
#      and 12 and what a section-ORDER model has to get right.
#   D. RTTI-CONSUMING EXPRESSIONS — `dynamic_cast` to pointer, to reference, to
#      `void*`; `typeid`. Measured to mint no `.rdata$r` on their own, and kept
#      anyway: crossed with A they are the workload's actual joint shape, and
#      "this construct moves the `.data` set and not the `.rdata$r` set" is a
#      claim that needs cases behind it rather than a paragraph.
#   E. TU POSITION — the vftable-emitting ctor/dtor alone, before, after and
#      between functions the port matches. `63-emit-order.py` established that
#      `.text` is emitted in a dependency walk; the RTTI block is a run of
#      sections with **no `.text` at all** wedged into that walk, and w-sect's
#      board #276 was exactly a TU whose non-`.text` content the writer mis-shaped.
#   F. THE CONTROLS — every shape with `virtual` removed, and every polymorphic
#      class with its ctor/dtor definition removed. Without them a fix that
#      refused any TU containing the word `virtual` would pass this fragment
#      while losing the `93-virtual-byval` shapes the port grades today.
#
# ---- what must NOT happen ------------------------------------------------------
#
# `Port=NotImplemented` is the contract on nearly every row. A **`Port=Mismatch`
# is the alarm**: at the sweep's own `/Ox /GS- /c` these TUs carry no `.rdata$r`
# at all — `/GR` is off there — so they reduce to *plain polymorphic classes with
# an emitted vftable*, which is a `.rdata` COMDAT of function pointers in a
# section name the writer already has (`PORT_WRITER_SECTIONS`). That is board
# **#276**'s shape exactly: a name the writer can spell over content it cannot
# produce.
#
# Every case must COMPILE, at `/Ox /GS- /c` and at every lane in
# `scripts/lanes.txt`, or the sweep's `ungraded` baseline moves. Two constructs
# are deliberately absent for that reason, both measured: `typeid(a)==typeid(b)`
# needs a real `<typeinfo>` (`error C2676`, and the sweep sets no include path),
# and `#include <typeinfo>` itself is `fatal error C1034`. `class type_info;` as a
# forward declaration is enough for `typeid` to be *written*, and that is what is
# used below.

# An ordinary function the port matches on its own, for wedging (axis E).
LEAF = "int %s(int a){return a+%d;}\n"

# ---- axis B: the hierarchy shapes -------------------------------------------
#
# `(tag, declarations, the class whose ctor/dtor is defined, its member)`. Each
# declares its virtuals and defines none of them, so `/c` needs no bodies; the
# ctor/dtor definition is appended by axis A, and it is the ONLY thing that makes
# a vftable.
HIER = (
    # standalone polymorphic class — the minimum: one ??_R1 in the base array.
    ("solo",
     "struct S{S();virtual ~S();virtual int f();int s;};\n", "S", "s"),
    # single inheritance, depth 1 / 2 / 3 — the base array grows by one each time.
    ("si1",
     "struct A{A();virtual ~A();virtual int f();int a;};\n"
     "struct S:A{S();virtual ~S();virtual int f();int s;};\n", "S", "s"),
    ("si2",
     "struct A{A();virtual ~A();virtual int f();int a;};\n"
     "struct B:A{B();virtual ~B();virtual int f();int b;};\n"
     "struct S:B{S();virtual ~S();virtual int f();int s;};\n", "S", "s"),
    ("si3",
     "struct A{A();virtual ~A();virtual int f();int a;};\n"
     "struct B:A{B();virtual ~B();virtual int f();int b;};\n"
     "struct C:B{C();virtual ~C();virtual int f();int c;};\n"
     "struct S:C{S();virtual ~S();virtual int f();int s;};\n", "S", "s"),
    # multiple inheritance: TWO vftables in one class, so two ??_R4 locators.
    ("mi2",
     "struct A{A();virtual ~A();virtual int f();int a;};\n"
     "struct B{B();virtual ~B();virtual int g();int b;};\n"
     "struct S:A,B{S();virtual ~S();virtual int f();virtual int g();int s;};\n",
     "S", "s"),
    ("mi3",
     "struct A{A();virtual ~A();virtual int f();int a;};\n"
     "struct B{B();virtual ~B();virtual int g();int b;};\n"
     "struct C{C();virtual ~C();virtual int h();int c;};\n"
     "struct S:A,B,C{S();virtual ~S();virtual int f();virtual int g();"
     "virtual int h();int s;};\n", "S", "s"),
    # MI where ONE base is not polymorphic — the base array has an entry the
    # vftable set does not.
    ("mimixed",
     "struct A{A();virtual ~A();virtual int f();int a;};\n"
     "struct P{P();int p;};\n"
     "struct S:A,P{S();virtual ~S();virtual int f();int s;};\n", "S", "s"),
    # virtual inheritance — the ??_R1 entries carry a vbase displacement.
    ("vi",
     "struct A{A();virtual ~A();virtual int f();int a;};\n"
     "struct S:virtual A{S();virtual ~S();virtual int f();int s;};\n", "S", "s"),
    # a diamond: one shared virtual base reached two ways.
    ("diamond",
     "struct A{A();virtual ~A();virtual int f();int a;};\n"
     "struct B:virtual A{B();virtual ~B();virtual int f();int b;};\n"
     "struct C:virtual A{C();virtual ~C();virtual int f();int c;};\n"
     "struct S:B,C{S();virtual ~S();virtual int f();int s;};\n", "S", "s"),
    # abstract: a pure virtual the derived class does not override here.
    ("abstract",
     "struct A{A();virtual ~A();virtual int f()=0;int a;};\n"
     "struct S:A{S();virtual ~S();virtual int f();int s;};\n", "S", "s"),
)

# ---- axis A: what makes the vftable ------------------------------------------
#
# `(tag, template taking (class, member))`. `none` is the CONTROL: the identical
# hierarchy with no ctor and no dtor body, which emits no vftable and therefore
# no `.rdata$r` — the row that separates "the TU mentions `virtual`" from "the TU
# emits a vftable".
def _dtor(c, m):   return "%s::~%s(){}\n" % (c, c)
def _ctor(c, m):   return "%s::%s(){}\n" % (c, c)
def _both(c, m):   return "%s::%s(){}\n%s::~%s(){}\n" % (c, c, c, c)
def _member(c, m): return "int %s::f(){return %s;}\n" % (c, m)
def _none(c, m):   return "int q(%s* p){return p->%s;}\n" % (c, m)

TRIGGER = (
    ("dtor",   _dtor),
    ("ctor",   _ctor),
    ("both",   _both),
    ("memfn",  _member),   # a virtual DEFINED here but no ctor/dtor — no vftable
    ("none",   _none),     # CONTROL: nothing defined at all
)


def cases(emit):
    # ---- A x B: the whole cross. This is the core of the fragment. -----------
    for _htag, decls, cls, mem in HIER:
        for _ttag, trig in TRIGGER:
            emit(decls + trig(cls, mem))

    # ---- A x B x E: the same cross, wedged among functions the port matches --
    #
    # Only the two triggers that actually emit a vftable, times every hierarchy,
    # times four positions — the RTTI block is a run of non-`.text` sections and
    # the question is where the dependency walk puts it.
    for _htag, decls, cls, mem in HIER:
        for trig in (_dtor, _both):
            body = trig(cls, mem)
            emit(decls + body + LEAF % ("z", 1))
            emit(decls + LEAF % ("z", 1) + body)
            emit(decls + LEAF % ("y", 2) + body + LEAF % ("z", 1))
            emit(decls + body + LEAF % ("z", 1) + LEAF % ("y", 2))

    # ---- C: TWO and THREE polymorphic classes in one TU ----------------------
    #
    # Each emitted vftable brings its own ??_R4/??_R3/??_R2/??_R1 block, so this
    # is the arity axis for the section COUNT rather than for one record's
    # contents. Both orders, because the blocks are emitted in the order the
    # vftables are, and that is what a section-order model has to reproduce.
    P = "struct P{P();virtual ~P();virtual int f();int p;};\n"
    Q = "struct Q{Q();virtual ~Q();virtual int g();int q;};\n"
    R = "struct R{R();virtual ~R();virtual int h();int r;};\n"
    PD, QD, RD = "P::~P(){}\n", "Q::~Q(){}\n", "R::~R(){}\n"
    emit(P + Q + PD + QD)
    emit(P + Q + QD + PD)
    emit(Q + P + PD + QD)
    emit(P + Q + R + PD + QD + RD)
    emit(P + Q + R + RD + QD + PD)
    emit(P + Q + R + QD + PD + RD)
    # …with only SOME of them emitting a vftable: the RTTI set must follow the
    # definitions, not the declarations.
    emit(P + Q + PD)
    emit(P + Q + QD)
    emit(P + Q + R + QD)
    emit(P + Q + R + PD + RD)
    # …and interleaved with ordinary functions.
    emit(P + Q + PD + LEAF % ("z", 1) + QD)
    emit(P + Q + LEAF % ("z", 1) + PD + QD)
    emit(P + Q + PD + QD + LEAF % ("z", 1) + LEAF % ("y", 2))
    # a derived class whose base also emits its vftable in this TU.
    emit("struct A{A();virtual ~A();virtual int f();int a;};\n"
         "struct S:A{S();virtual ~S();virtual int f();int s;};\n"
         "A::~A(){}\nS::~S(){}\n")
    emit("struct A{A();virtual ~A();virtual int f();int a;};\n"
         "struct S:A{S();virtual ~S();virtual int f();int s;};\n"
         "S::~S(){}\nA::~A(){}\n")

    # ---- D: dynamic_cast, every target form, with and without a vftable ------
    #
    # MEASURED to mint no `.rdata$r`: the ??_R0 type descriptors go to `.data`.
    # The rows are here because the CROSS is the workload's shape, and because a
    # claim that a construct does not move a section wants cases, not a sentence.
    DC = ("struct A{A();virtual ~A();virtual int f();int a;};\n"
          "struct S:A{S();virtual ~S();virtual int f();int s;};\n")
    for tail in (
        "int q(A* p){ S* d = dynamic_cast<S*>(p); return d ? d->s : 0; }\n",
        "int q(A& p){ S& d = dynamic_cast<S&>(p); return d.s; }\n",
        "void* q(A* p){ return dynamic_cast<void*>(p); }\n",
        "A* q(S* p){ return dynamic_cast<A*>(p); }\n",
        "int q(A* p){ return dynamic_cast<S*>(p) ? 1 : 0; }\n",
        "int q(A* p, A* r){ return dynamic_cast<S*>(p) == dynamic_cast<S*>(r); }\n",
    ):
        emit(DC + tail)                          # no vftable emitted here
        emit(DC + tail + "S::~S(){}\n")          # …and with one
        emit(DC + "A::~A(){}\n" + tail)          # …the BASE's, before the cast
        emit(DC + tail + LEAF % ("z", 1))        # …beside a matched function
    # a cross-cast through multiple inheritance, and one through a virtual base:
    # both are runtime walks of the base-class array rather than a fixed offset.
    MI = ("struct A{A();virtual ~A();virtual int f();int a;};\n"
          "struct B{B();virtual ~B();virtual int g();int b;};\n"
          "struct S:A,B{S();virtual ~S();virtual int f();virtual int g();int s;};\n")
    for tail in (
        "int q(A* p){ B* d = dynamic_cast<B*>(p); return d ? d->b : 0; }\n",
        "int q(A* p){ S* d = dynamic_cast<S*>(p); return d ? d->s : 0; }\n",
        "int q(B* p){ A* d = dynamic_cast<A*>(p); return d ? d->a : 0; }\n",
    ):
        emit(MI + tail)
        emit(MI + tail + "S::~S(){}\n")
    VB = ("struct A{A();virtual ~A();virtual int f();int a;};\n"
          "struct S:virtual A{S();virtual ~S();virtual int f();int s;};\n")
    for tail in (
        "int q(A* p){ S* d = dynamic_cast<S*>(p); return d ? d->s : 0; }\n",
        "void* q(A* p){ return dynamic_cast<void*>(p); }\n",
    ):
        emit(VB + tail)
        emit(VB + tail + "S::~S(){}\n")

    # ---- D: typeid ------------------------------------------------------------
    #
    # `class type_info;` is a forward declaration and is all `typeid` needs to be
    # written. `<typeinfo>` is `fatal error C1034` here (no include path) and
    # `typeid(a)==typeid(b)` is `error C2676` without the real header; both are
    # excluded on purpose so the sweep's `ungraded` baseline does not move.
    TI = "class type_info;\n"
    TY = ("struct A{A();virtual ~A();virtual int f();int a;};\n"
          "struct S:A{S();virtual ~S();virtual int f();int s;};\n")
    for tail in (
        "const type_info& q(A* p){ return typeid(*p); }\n",          # polymorphic
        "const type_info& q(A& p){ return typeid(p); }\n",
        "const type_info& q(){ return typeid(S); }\n",               # static
        "const type_info& q(){ return typeid(int); }\n",             # built-in
        "const type_info& q(A* p, int n){ return n ? typeid(*p) : typeid(S); }\n",
    ):
        emit(TI + TY + tail)
        emit(TI + TY + tail + "S::~S(){}\n")
        emit(TI + TY + "A::~A(){}\n" + tail)
        emit(TI + TY + tail + LEAF % ("z", 1))
    # typeid of a NON-polymorphic type — resolved at compile time, no vftable
    # anywhere in the TU. This is the control for the four rows above.
    emit(TI + "struct N{int n;};\nconst type_info& q(){ return typeid(N); }\n")
    emit(TI + "struct N{int n;};\nconst type_info& q(N* p){ return typeid(*p); }\n")
    emit(TI + "const type_info& q(){ return typeid(double); }\n")
    emit(TI + "const type_info& q(){ return typeid(int*); }\n")
    # typeid AND dynamic_cast in one TU, with and without an emitted vftable.
    emit(TI + TY + "int q(A* p){ S* d=dynamic_cast<S*>(p); return d?d->s:0; }\n"
         "const type_info& r(A* p){ return typeid(*p); }\n")
    emit(TI + TY + "int q(A* p){ S* d=dynamic_cast<S*>(p); return d?d->s:0; }\n"
         "const type_info& r(A* p){ return typeid(*p); }\nS::~S(){}\n")

    # ---- A: the trigger, stated on its own ------------------------------------
    #
    # The two-line difference that decides whether the obj has an RTTI block at
    # all. Kept separate from the A x B cross so the smallest reproducer of the
    # rule is a case in its own right and not a corner of a grid.
    MIN = "struct S{S();virtual ~S();int s;};\n"
    emit(MIN + "S::~S(){}\n")                       # vftable  -> .rdata$r
    emit(MIN + "S::S(){}\n")                        # vftable  -> .rdata$r
    emit(MIN + "int q(S* p){return p->s;}\n")       # none
    emit("struct S{S();~S();int s;};\nS::~S(){}\n")  # non-virtual: no vftable
    emit("struct S{virtual int f();int s;};\nint S::f(){return s;}\n")
    emit("struct S{S();virtual int f();int s;};\nS::S(){}\n")
    emit("struct S{S();virtual int f()=0;virtual ~S();int s;};\nS::~S(){}\n")
    # a ctor that is only IMPLICITLY needed, by a local object.
    emit("struct S{virtual ~S();virtual int f();int s;};\n"
         "int q(){ S x; return x.s; }\n")
    emit("struct S{virtual int f();int s;};\nint q(){ S x; return x.s; }\n")
    emit("struct S{S();virtual ~S();virtual int f();int s;};\n"
         "int q(S* p){ S x(*p); return x.s; }\n")
    # `__declspec(novtable)` — an XDK idiom: the ctor does NOT write the vfptr,
    # so the vftable may not be emitted even though the ctor body is.
    emit("struct __declspec(novtable) S{S();virtual ~S();virtual int f();int s;};\n"
         "S::~S(){}\n")
    emit("struct __declspec(novtable) A{A();virtual int f();int a;};\n"
         "struct S:A{S();virtual ~S();virtual int f();int s;};\nS::~S(){}\n")

    # ---- F: THE CONTROLS ------------------------------------------------------
    #
    # Each is a row above with `virtual` deleted, or with the ctor/dtor definition
    # deleted. Several are `Port=Match` today. A fragment of positives only cannot
    # tell a rule from a blanket refusal — and "refuse every TU containing the
    # word `virtual`" would pass a positives-only version of this file while
    # losing what `93-virtual-byval.py` grades.
    for src in (
        "int f(int a){ return a+1; }\n",
        LEAF % ("z", 1) + "int f(int a){ return a+1; }\n",
        # non-virtual twins of the hierarchy shapes.
        "struct A{A();~A();int a;};\nstruct S:A{S();~S();int s;};\nS::~S(){}\n",
        "struct A{A();~A();int a;};\nstruct S:A{S();~S();int s;};\nA::~A(){}\n",
        "struct A{A();~A();int a;};\nstruct B{B();~B();int b;};\n"
        "struct S:A,B{S();~S();int s;};\nS::~S(){}\n",
        "struct A{A();~A();int a;};\nstruct S:virtual A{S();~S();int s;};\nS::~S(){}\n",
        "struct S{S();~S();int s;};\nS::S(){}\nS::~S(){}\n",
        # polymorphic but with NOTHING defined in the TU: declarations only.
        "struct S{S();virtual ~S();virtual int f();int s;};\n"
        "int q(S* p){ return p->s + 1; }\n",
        "struct A{A();virtual ~A();virtual int f();int a;};\n"
        "struct S:A{S();virtual ~S();virtual int f();int s;};\n"
        "int q(S* p){ return p->s + 1; }\n",
        # a virtual CALL, which needs no RTTI and no emitted vftable.
        "struct S{S();virtual ~S();virtual int f();int s;};\n"
        "int q(S* p){ return p->f(); }\n",
        "struct S{S();virtual ~S();virtual int f();int s;};\n"
        "int q(S* p){ return p->f(); }\n" + LEAF % ("z", 1),
        # a plain struct with a dtor defined here — the `50-dtor-base.py` shape,
        # kept so the RTTI rows are read against it rather than against nothing.
        "struct S{S();~S();int s;};\nS::~S(){}\n",
        "struct S{int s;};\nint q(S* p){ return p->s + 1; }\n",
    ):
        emit(src)
