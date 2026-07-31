#!/usr/bin/env python3
"""gt_inline_decline.py — WHICH inlines does the front end refuse, and on what axis?

`docs/LABEL_COUNTER.md` §6 measures what an inlined site *costs* the label
counter (law L'), and its own closing paragraph names the gap this script
exists for:

    Law L' predicts the declined tree exactly, so the modelling job is real
    and tractable; what does not exist yet is any rule for PREDICTING THE
    DECLINE ITSELF.

That gap is load-bearing rather than academic. `crates/c2-il/src/func/bundle.rs`
refuses any TU where a callee is also defined, because c2 may inline it; the
first rung that relaxes that gate has to know WHICH expansion tree it is
counting labels for, and today nothing can tell it.

WHY A SECOND INSTRUMENT
-----------------------
`gt_label_inline.py` detects a decline with `INLINE-DECLINED?`: P's `.text`
grew by much less than the hand-written control's did. That is a *proxy*, it
is one-sided, and §6.14 records it being widened from `<` to `<=` after it
missed the cheapest possible decline. It also cannot say **which level** of a
two-level tree was refused, or **how many** of the N sites were.

This script reads the answer directly out of the obj instead. An inlined call
leaves no trace in P's relocation table; a call the front end declined leaves
exactly one `bl` against the callee's symbol. So

    * the reloc COUNT for a callee = the number of sites it was NOT inlined at,
      with per-site resolution, from ONE capture;
    * the reloc NAMES = which instance of a two-level tree survived, which is
      the depth evidence §6.4 says only `.text` growth carries — carried here
      by a symbol name rather than by an inequality on byte counts;
    * `bl gs` counts cross-check the whole tree: a fully inlined N-site sweep
      of a body containing c calls holds N*c of them.

Every `--family` row prints BOTH detectors' verdicts side by side and tags
`<== DETECTORS DISAGREE`, so a disagreement is a printed row and not a memory.

WHAT IT MEASURES
----------------
Ladders. Each ladder holds a base tree fixed and varies exactly ONE feature
(k arithmetic statements / k calls / k dead locals / k live locals / …), and
for each rung reports `Nfull` — the largest N at which every site was inlined
— separately at `/O1` and `/Ox`. A budget that is a threshold on some monotone
size shows up as `Nfull` falling monotonically along the ladder; where it does
not, that is the finding and the row prints it.

Usage:
    scripts/gt_inline_decline.py [--mode '/O1 /GS- /c'] [--max N] [ladder ...]
    scripts/gt_inline_decline.py --list
    scripts/gt_inline_decline.py --family NAME ...   # gt_label_inline families
    scripts/gt_inline_decline.py --cases             # the categorical refusals
    scripts/gt_inline_decline.py --max N --kmax K

Env: C2RS_WIBO / C2RS_COMPILERS as for scripts/gt_capture.sh.
Exit status is 0 if every capture succeeded; it says nothing about whether a
prediction held — read the table.
"""

import os
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from gt_label_stride import capture, groups  # noqa: E402
from gt_label_inline import (  # noqa: E402
    FAMILIES, GS, INT_HEAD, INT_TAIL, src_of,
)

# --------------------------------------------------------------------------
# the readout
# --------------------------------------------------------------------------
def extent(o, g):
    """One function's OWN byte range inside its `.text` section.

    THIS IS NOT COSMETIC. `/O1` implies `/Gy`, so every function gets its own
    `.text` COMDAT and "the section" and "the function" are the same bytes.
    **`/Ox` does not imply `/Gy`**: at `/Ox` this compiler packs every function
    of the TU into ONE `.text`, so anything that reads `len(section)` at `/Ox`
    is reading P *plus the callees plus all three anchors*. A detector built on
    that number is polluted at N=0->1 by the callee's own first emission — the
    largest single term in the sweep — which is exactly where a decline has to
    be caught. See the header: this is why the older `dtext` detector reports
    `noinline`, the family whose entire purpose is to be declined, as clean at
    `/Ox` N=1.
    """
    sec = o.sections[g["sec"] - 1]
    starts = sorted({s["value"] for s in o.symbols
                     if s["sec"] == g["sec"] and s["type"] == 0x0020
                     and s["sc"] in (2, 3)})
    me = None
    for s in o.symbols:
        if s["name"] == g["name"] and s["sec"] == g["sec"]:
            me = s["value"]
            break
    if me is None:
        return sec, 0, sec["rawsize"]
    nxt = [v for v in starts if v > me]
    return sec, me, (nxt[0] if nxt else sec["rawsize"])


def read(o):
    """P's OWN relocation targets and every emitted function's OWN size."""
    gs_ = groups(o)

    def find(sfx):
        for g in gs_:
            if g["name"].startswith("?" + sfx + "@@") or g["name"] == sfx:
                return g
        return None

    P = find("P")
    if P is None:
        return {"error": "no P among %s" % [g["name"] for g in gs_]}
    sec, lo, hi = extent(o, P)
    rel = {}
    for va, symidx, ty in o.relocs(sec):
        if not (lo <= va < hi):
            continue                      # another function's call, at /Ox
        s = o.sym_by_index(symidx)
        if s is None:
            continue
        rel[s["name"]] = rel.get(s["name"], 0) + 1
    emitted = {}
    for g in gs_:
        if g is P:
            continue
        _s, glo, ghi = extent(o, g)
        emitted[g["name"]] = ghi - glo
    return {"rel": rel, "tsize": hi - lo, "emit": emitted}


# --------------------------------------------------------------------------
# SCHEDULE D — the /O1 decline, MEASURED. Not a formula, and that is the point
# --------------------------------------------------------------------------
# At `/O1` the front end's decision is ALL-OR-NOTHING per (caller, callee)
# pair — never a mixture of inlined and declined sites, over 14 ladders, ~200
# rungs, and up to 24 sites — and the number of sites it will take is a
# function of ONE variable: `s`, the callee's OWN emitted `.text` size. §6.5
# guarantees that number is in every obj for free, because c2 emits the
# callee's COMDAT whether or not it was inlined.
#
# The boundaries below are pinned to a single 4-byte step by a ladder whose
# rungs are ONE INSTRUCTION each, and reproduced with ZERO disagreements by
# thirteen other ladders that move `s` by five independent mechanisms
# (1-instruction rungs, 1-statement integer arithmetic, calls, `if`s, and
# `double` arithmetic with an FPR frame and `_fltused`), at depth 1 and depth
# 2, with and without a loop.
#
# THERE IS NO CLOSED FORM, and every rival that looked like one is in
# SUPERSEDED_D, re-refuted from each run's own numbers. In particular this
# table is NOT generated by "cost x count <= budget" for any cost: N=10 is
# refused at s=68 while N=9 is accepted at s=72, so a strictly SMALLER total
# is refused where a larger one is accepted. That is an arithmetic
# impossibility for any product model, not a poor fit.
#
# `/O1` ONLY. `/Ox` is a DIFFERENT MECHANISM — N-independent, all-or-nothing
# at every N up to 12 — not a different constant. Nothing here is inferred
# from one mode to the other.
#
#   (max s, Nfull)     s/4 = instructions
LAW_D_TABLE = [
    (64, None),        # <= 16 instr — inlined at ANY number of sites (>=24)
    (72, 9),           # 17-18
    (76, 7),           # 19
    (80, 5),           # 20
    (88, 4),           # 21-22
    (100, 3),          # 23-25
    (140, 2),          # 26-35
    (256, 1),          # 36-64
]                      # >= 260 bytes / 65 instr -> NEVER inlined, even once


def law_d(s):
    """Measured `Nfull` for a callee of `s` bytes; None means unbounded."""
    if s is None:
        return None
    for hi, n in LAW_D_TABLE:
        if s <= hi:
            return n
    return 0


# The readings this round retired, re-refuted from each run's own measurement
# rather than remembered — same discipline as gt_label_inline's SUPERSEDED.
SUPERSEDED_D = [
    # Exact on every cell at N <= 6, where the sweep's own cap hid the
    # counter-examples; dead at s=68 (says >=12, measures 9) and s=72 (says
    # 10, measures 9). Pre-registered in work/gt-inline-decline/ESTIMATE-
    # round28d.txt and killed by the first out-of-sample capture.
    ("(N-1)*(s-64) < 80",
     lambda s: None if s is None or s <= 64 else 1 + 79 // (s - 64)),
    # The reading I would have written down first, killed at N <= 6.
    ("N*(s-64) < 80  ('the first copy is not free')",
     lambda s: None if s is None or s <= 64 else 79 // (s - 64)),
]


def demangle_ish(name):
    """`?lsa@@YAHH@Z` -> `lsa`. Good enough to match a probe's own callees."""
    if name.startswith("?") and "@@" in name:
        return name[1:name.index("@@")]
    return name


def declined(rel, watch):
    """{callee -> how many sites kept a `bl`}, over the names we planted."""
    out = {}
    for nm, n in rel.items():
        d = demangle_ish(nm)
        # a ctor mangles as ?0CL@@... — its class name is what we planted
        for w in watch:
            if d == w or d.endswith(w) or w in nm:
                out[w] = out.get(w, 0) + n
    return out


# --------------------------------------------------------------------------
# ladders — one base tree, one varied feature
# --------------------------------------------------------------------------
def stmts_arith(k, var="v"):
    """k statements, 0 new locals, 0 calls.

    `v = v*3+K;` was the first spelling and it is USELESS: a chain of affine
    ops constant-folds to one multiply-add, so the callee's `.text` did not
    move by a single byte across k = 0..8 and neither did anything else. Each
    rung is a distinct shift-xor instead, which is not affine and cannot fold
    into its neighbour. Verified by `in.text` growing monotonically in k.
    """
    return " ".join("%s^=%s>>%d;" % (var, var, i + 2) for i in range(k))


def stmts_call(k, var="v"):
    """k statements, 0 new locals, k calls."""
    return " ".join("%s+=gs(%s+%d);" % (var, var, i + 1) for i in range(k))


def stmts_deadloc(k):
    """k DECLARED LOCALS THAT GENERATE NO CODE — `loc1-dead`'s spelling.

    Law L' charges one of these a full `E` unit (`loc1-dead` = 4 against
    `loc0` = 3). If the decline axis is the same axis as the charge, this
    ladder moves the threshold as fast as any other; if it is code, it does
    not move it at all. That is the sharpest single question here.
    """
    return " ".join("int dd%d=%d;" % (i, i + 5) for i in range(k))


def stmts_liveloc(k, var="v"):
    """k statements AND k locals, each used — `stmts_arith` plus a name."""
    parts = []
    src = var
    for i in range(k):
        parts.append("int xx%d=%s^(%s>>%d);" % (i, src, src, i + 2))
        src = "xx%d" % i
    if k:
        parts.append("%s=%s;" % (var, src))
    return " ".join(parts)


def stmts_fine(k, var="v"):
    """k statements of ONE instruction each — 4-byte resolution on `s`.

    `^= a` and `+= a` alternate so no two adjacent rungs can be folded or
    reassociated into one another, which is what killed the first spelling
    of `stmts_arith`.
    """
    return " ".join("%s%s=a;" % (var, "^" if i % 2 == 0 else "+")
                    for i in range(k))


def stmts_dbl(k, var="v"):
    """k statements of DOUBLE arithmetic — same instruction count, totally
    different opcodes, an FPR prologue and `_fltused`. `x*x` is not affine
    so the chain cannot fold."""
    if not k:
        return ""
    parts = ["double xd=(double)a;"]
    parts += ["xd=xd*xd+(double)a;" for _ in range(k)]
    parts.append("%s+=(int)xd;" % var)
    return " ".join(parts)


def stmts_if(k, var="v"):
    """k `if` statements — branches and basic blocks, which law L' charges an
    `E` unit each."""
    return " ".join("if(a>%d) %s+=%d;" % (i + 1, var, i + 3) for i in range(k))


LOOP = "for(int i=0;i<a;i++) v+=gs(i);"
LOOP_LEAF = "for(int i=0;i<a;i++) v+=i*3;"


def tree_d2(inner_extra, loop=LOOP):
    """The `d2-loop-asctor` tree — §6.14's row that SURVIVES at /Ox to N=6.

    static int in2(int a){ int v=0; <loop> <extra> return v; }
    static int ou2(int a){ int c=in2(a); return c; }
    """
    leads = ("static int in2(int a){ int v=0; %s %s return v; }\n"
             "static int ou2(int a){ int c=in2(a); return c; }"
             % (loop, inner_extra))
    return leads, "s=ou2(s);", ["ou2", "in2"]


def tree_d1(inner_extra, loop=LOOP):
    """The same body at depth 1 — no wrapper."""
    leads = ("static int in1(int a){ int v=0; %s %s return v; }"
             % (loop, inner_extra))
    return leads, "s=in1(s);", ["in1"]


def tree_d1_noloop(inner_extra):
    leads = ("static int in1(int a){ int v=gs(a)+a; %s return v; }"
             % inner_extra)
    return leads, "s=in1(s);", ["in1"]


def tree_d2_noloop(inner_extra):
    leads = ("static int in2(int a){ int v=gs(a)+a; %s return v; }\n"
             "static int ou2(int a){ int c=in2(a); return c; }"
             % inner_extra)
    return leads, "s=ou2(s);", ["ou2", "in2"]


def tree_ctor(inner_extra, loop=LOOP):
    """§6.14's DC3 shape: the loop lives in a CONSTRUCTOR."""
    leads = ("struct CL { int v; CL(int a){ v=0; %s %s } };\n"
             "static int lcl(int a){ CL c(a); return c.v; }"
             % (loop, inner_extra))
    return leads, "s=lcl(s);", ["lcl", "CL"]


LADDERS = {}


def ladder(name, tree, gen, kmax, note):
    LADDERS[name] = (tree, gen, kmax, note)


# --- the four feature ladders, on the tree that SURVIVES at /Ox -----------
ladder("d2-arith", tree_d2, stmts_arith, 8,
       "d2 loop tree + k arithmetic statements (0 locals, 0 calls)")
ladder("d2-call", tree_d2, stmts_call, 6,
       "d2 loop tree + k CALLS")
ladder("d2-deadloc", tree_d2, stmts_deadloc, 8,
       "d2 loop tree + k DEAD locals (a full E unit each, ~no code)")
ladder("d2-liveloc", tree_d2, stmts_liveloc, 8,
       "d2 loop tree + k live locals (= d2-arith + k names)")
# --- the same four one level up, to separate depth from size --------------
ladder("d1-arith", tree_d1, stmts_arith, 10,
       "depth-1 loop callee + k arithmetic statements")
ladder("d1-call", tree_d1, stmts_call, 8,
       "depth-1 loop callee + k CALLS")
ladder("d1-deadloc", tree_d1, stmts_deadloc, 10,
       "depth-1 loop callee + k DEAD locals")
# --- and with no loop at all, to ask whether the loop is special ----------
ladder("d2-noloop-arith", tree_d2_noloop, stmts_arith, 14,
       "d2 tree, NO loop, + k arithmetic statements")
ladder("d2-noloop-call", tree_d2_noloop, stmts_call, 10,
       "d2 tree, NO loop, + k CALLS")
ladder("d1-noloop-arith", tree_d1_noloop, stmts_arith, 16,
       "depth-1, NO loop, + k arithmetic statements")
ladder("d1-noloop-call", tree_d1_noloop, stmts_call, 12,
       "depth-1, NO loop, + k CALLS")
# --- the DC3 shape itself -------------------------------------------------
# --- HELD OUT: move `s` by a DIFFERENT mechanism ---------------------------
# The eleven ladders above all grow `s` by appending statements to one body,
# so "Nfull is a function of s" could be "Nfull is a function of whatever my
# rungs move". These three move it another way.
ladder("d1-fine", tree_d1_noloop, stmts_fine, 64,
       "HELD OUT: 1-INSTRUCTION rungs — 4-byte resolution on the boundaries")
ladder("d1-dbl", tree_d1_noloop, stmts_dbl, 14,
       "HELD OUT: DOUBLE arithmetic — different opcodes, FPR frame, _fltused")
ladder("d1-if", tree_d1_noloop, stmts_if, 14,
       "HELD OUT: k `if` statements — branches and basic blocks")
ladder("ctor-arith", tree_ctor, stmts_arith, 4,
       "§6.14's ctor+loop+call shape + k arithmetic statements")
ladder("ctor-leaf-arith", lambda e: tree_ctor(e, LOOP_LEAF), stmts_arith, 6,
       "the ctor whose loop makes NO call + k arithmetic statements")
ladder("d2-leaf-arith", lambda e: tree_d2(e, LOOP_LEAF), stmts_arith, 8,
       "d2 tree whose loop makes NO call + k arithmetic statements")


# --------------------------------------------------------------------------
# CASES — one-off trees for the CATEGORICAL refusal, which is not a budget.
#
# §6.14 measured `ctor-loop` (a `for` with a call, inside a constructor)
# declined FROM THE FIRST SITE in both modes, while `ctor-loop-leaf` (same
# ctor, loop makes no call) and `d2-loop-asctor` (same tree, plain function
# where the ctor was) both inline to N=6, and concluded "it is the
# conjunction" of ctor and call-in-loop.
#
# That conclusion has a confound it never tested. The constructor stores to
# `this->v` INSIDE the loop; `d2-loop-asctor` accumulates into a LOCAL. So
# "constructor" and "a store through a pointer inside a loop" are perfectly
# aliased in §6.14's design, and the second is the one an optimiser would
# actually care about — it is the aliasing question, not a C++ one. These
# cases break the alias in both directions: a ctor that accumulates into a
# local, and a plain free function that stores through a pointer in a loop.
# --------------------------------------------------------------------------
CASES = []


def case(name, leads, site, watch, note):
    CASES.append((name, leads, site, watch, note))


case("ctor-loop-call",
     "struct C1 { int v; C1(int a){ v=0;"
     " for(int i=0;i<a;i++) v+=gs(i); } };\n"
     "static int f1(int a){ C1 c(a); return c.v; }",
     "s=f1(s);", ["f1", "C1"],
     "BASELINE = §6.14's ctor-loop: ctor, loop, call, store to this->v")
case("ctor-loop-leaf",
     "struct C2 { int v; C2(int a){ v=0;"
     " for(int i=0;i<a;i++) v+=i*3; } };\n"
     "static int f2(int a){ C2 c(a); return c.v; }",
     "s=f2(s);", ["f2", "C2"],
     "…the same ctor, loop makes NO call (§6.14 control: inlines)")
case("ctor-loop-local",
     "struct C3 { int v; C3(int a){ int t=0;"
     " for(int i=0;i<a;i++) t+=gs(i); v=t; } };\n"
     "static int f3(int a){ C3 c(a); return c.v; }",
     "s=f3(s);", ["f3", "C3"],
     "THE PROBE: ctor + loop + call, accumulating into a LOCAL and storing"
     " to the member ONCE. Breaks the ctor / store-in-loop alias one way")
case("method-loop-call",
     "struct C4 { int v; void fill(int a){ v=0;"
     " for(int i=0;i<a;i++) v+=gs(i); } };\n"
     "static int f4(int a){ C4 c; c.fill(a); return c.v; }",
     "s=f4(s);", ["f4", "fill"],
     "THE PROBE: the identical body as a MEMBER FUNCTION, not a ctor."
     " Breaks the alias the other way")
case("method-loop-local",
     "struct C5 { int v; void fill(int a){ int t=0;"
     " for(int i=0;i<a;i++) t+=gs(i); v=t; } };\n"
     "static int f5(int a){ C5 c; c.fill(a); return c.v; }",
     "s=f5(s);", ["f5", "fill"],
     "…member function accumulating into a local — the 2x2's fourth cell")
case("ptr-loop-call",
     "static void f6a(int *p,int a){ *p=0;"
     " for(int i=0;i<a;i++) *p+=gs(i); }\n"
     "static int f6(int a){ int r; f6a(&r,a); return r; }",
     "s=f6(s);", ["f6", "f6a"],
     "NO CLASS AT ALL: a free function storing THROUGH A POINTER inside a"
     " loop whose body calls. If this is refused, C++ is irrelevant")
case("ptr-loop-local",
     "static void f7a(int *p,int a){ int t=0;"
     " for(int i=0;i<a;i++) t+=gs(i); *p=t; }\n"
     "static int f7(int a){ int r; f7a(&r,a); return r; }",
     "s=f7(s);", ["f7", "f7a"],
     "…the same free function accumulating into a local — the control")
case("ctor-call-noloop",
     "struct C8 { int v; C8(int a){ v=gs(a)+a; } };\n"
     "static int f8(int a){ C8 c(a); return c.v; }",
     "s=f8(s);", ["f8", "C8"],
     "a ctor with a call and NO loop (§6.9 measured this inlining)")
case("ctor-loop-nocall-store",
     "struct C9 { int v; C9(int a){ v=0;"
     " for(int i=0;i<a;i++) v+=i*3; } };\n"
     "static int f9(int a){ C9 c(a); return c.v; }",
     "s=f9(s);", ["f9", "C9"],
     "store to this->v in a loop, loop makes NO call (= ctor-loop-leaf)")
case("ctor-loop-while",
     "struct CA { int v; CA(int a){ v=0; int i=a;"
     " while(i>0){ v+=gs(i); i--; } } };\n"
     "static int fa(int a){ CA c(a); return c.v; }",
     "s=fa(s);", ["fa", "CA"],
     "the refused conjunction with a `while` instead of a `for`")
case("glob-loop-call",
     "static int gv;\n"
     "static void fba(int a){ gv=0; for(int i=0;i<a;i++) gv+=gs(i); }\n"
     "static int fb(int a){ fba(a); return gv; }",
     "s=fb(s);", ["fb", "fba"],
     "store to a STATIC GLOBAL inside the calling loop — same aliasing"
     " shape, no pointer parameter")
case("member-noloop-store",
     "struct CC { int v; CC(int a){ v=gs(a); v+=gs(a+1); v+=gs(a+2); } };\n"
     "static int fc(int a){ CC c(a); return c.v; }",
     "s=fc(s);", ["fc", "CC"],
     "three stores to this->v with calls between them, NO loop")

# --- BATCH 2: separate "a ctor with a loop" from "a store to memory inside
#     a loop", and pin the /O1 constructor rule that batch 1 turned up.
case("ctor-2store-call",
     "struct D1 { int v; D1(int a){ v=gs(a); v+=gs(a+1); } };\n"
     "static int g1(int a){ D1 c(a); return c.v; }",
     "s=g1(s);", ["g1", "D1"],
     "/O1 PRED DECLINED: exactly TWO stores to a member with a CALL between")
case("ctor-2store-nocall",
     "struct D2 { int v; D2(int a){ v=a*3; v+=a^7; } };\n"
     "static int g2(int a){ D2 c(a); return c.v; }",
     "s=g2(s);", ["g2", "D2"],
     "/O1 PRED INLINED: two stores, NO call between them")
case("ctor-1store-2call",
     "struct D3 { int v; D3(int a){ v=gs(a)+gs(a+1); } };\n"
     "static int g3(int a){ D3 c(a); return c.v; }",
     "s=g3(s);", ["g3", "D3"],
     "/O1 PRED INLINED: ONE store, two calls — calls alone are not it")
case("ctor-2mem-call",
     "struct D4 { int v,w; D4(int a){ v=gs(a); w=gs(a+1); } };\n"
     "static int g4(int a){ D4 c(a); return c.v+c.w; }",
     "s=g4(s);", ["g4", "D4"],
     "two stores to DIFFERENT members with a call between — same member or"
     " any member?")
case("method-2store-call",
     "struct D5 { int v; void set(int a){ v=gs(a); v+=gs(a+1); } };\n"
     "static int g5(int a){ D5 c; c.set(a); return c.v; }",
     "s=g5(s);", ["g5", "set"],
     "the SAME body as a member function — /O1 PRED INLINED if the rule is"
     " constructor-specific")
case("ptr-2store-call",
     "static void g6a(int *p,int a){ *p=gs(a); *p+=gs(a+1); }\n"
     "static int g6(int a){ int r; g6a(&r,a); return r; }",
     "s=g6(s);", ["g6", "g6a"],
     "…and as a free function through a pointer — /O1 PRED INLINED")
case("ctor-loop-nostore",
     "struct D7 { int v; D7(int a){ int t=0;"
     " for(int i=0;i<a;i++) t+=i*3; v=t; } };\n"
     "static int g7(int a){ D7 c(a); return c.v; }",
     "s=g7(s);", ["g7", "D7"],
     "/Ox PRED DECLINED: a ctor with a loop that neither calls nor stores to"
     " memory — tests 'any loop in a ctor' at /Ox")
case("method-loop-nostore",
     "struct D8 { int v; void set(int a){ int t=0;"
     " for(int i=0;i<a;i++) t+=i*3; v=t; } };\n"
     "static int g8(int a){ D8 c; c.set(a); return c.v; }",
     "s=g8(s);", ["g8", "set"],
     "…the identical body as a member function — /Ox PRED INLINED")
case("ptr-store-noloop",
     "static void g9a(int *p,int a){ *p=gs(a); *p+=gs(a+1); *p+=gs(a+2); }\n"
     "static int g9(int a){ int r; g9a(&r,a); return r; }",
     "s=g9(s);", ["g9", "g9a"],
     "/Ox PRED INLINED: stores through a pointer with calls between, but NO"
     " loop — tests whether the /Ox trigger needs the loop")
case("ptr-loop-store-nocall",
     "static void gaa(int *p,int a){ *p=0;"
     " for(int i=0;i<a;i++) *p+=i*3; }\n"
     "static int ga2(int a){ int r; gaa(&r,a); return r; }",
     "s=ga2(s);", ["ga2", "gaa"],
     "/Ox PRED DECLINED: store through a pointer inside a loop, loop makes NO"
     " call")


def run_cases(mode, wd, nmax, want):
    print("    %-22s %-5s %-5s %-6s %-22s %s"
          % ("case", "Nfull", "Ndir", "P@N=1", "callee .text (s)",
             "declined per N (1..%d)" % nmax))
    print("    `s` is what the /O1 SIZE budget is a function of, so a row with")
    print("    Nfull=0 at a small `s` is a CATEGORICAL refusal, not the budget.")
    bad = 0
    for name, leads, site, watch, note in CASES:
        if want and name not in want:
            continue
        per_n, nfull, ndir, ptext, sizes = [], 0, 0, None, ""
        dsz = None
        for n in range(1, nmax + 1):
            body = " ".join([site] * n)
            src = src_of(GS, [leads],
                         "%s %s %s" % (INT_HEAD, body, INT_TAIL))
            o = capture(src, mode, wd, "c_%s_%d" % (name.replace("-", "_"), n))
            if o is None:
                per_n.append("!")
                bad += 1
                continue
            r = read(o)
            if "error" in r:
                per_n.append("!")
                bad += 1
                continue
            d = declined(r["rel"], watch)
            if n == 1:
                ptext = r["tsize"]
                dsz = size_of(r["emit"], watch[0])
                sizes = ",".join("%s=%d" % (demangle_ish(nm)[:6], sz)
                                 for nm, sz in sorted(r["emit"].items())
                                 if not demangle_ish(nm).startswith("a"))
            # the DIRECT callee alone — LAW D is a claim about ONE
            # (caller, callee) pair, so a deeper refusal must not be folded in
            ddir = {w: c for w, c in d.items() if w == watch[0]}
            nd = sum(ddir.values())
            if nd == 0 and ndir == n - 1:
                ndir = n
            tot = sum(d.values())
            per_n.append("." if tot == 0
                         else "".join("%s%d" % (w[:3], c)
                                      for w, c in sorted(d.items())))
            if tot == 0 and nfull == n - 1:
                nfull = n
        v = grade_d(ndir, dsz, nmax, "/O1" in mode)
        inner = "" if nfull == ndir else "   INNER-DECLINED (a different pair)"
        print("    %-22s %-5d %-5d %-6s %-22s %s"
              % (name, nfull, ndir, ptext, sizes, " ".join(per_n)))
        print("    %-22s        %s%s" % ("", v, inner))
        print("    %-22s        %s" % ("", note))
    return bad


def sib_source(na, nb, ka, kb):
    """Two DIFFERENT callees in one P — the subject `sba` and a sibling `sbb`.

    Everything else in this file puts exactly one callee in P, so nothing
    else here can say whether the front end's limit for one (caller, callee)
    pair moves when the caller has already absorbed an unrelated expansion.
    §6.12's `ptr-sibling` is the standing warning that a property of P's
    WHOLE expansion can reach across call sites.
    """
    leads = ("static int sba(int a){ int v=gs(a)+a; %s return v; }\n"
             "static int sbb(int a){ int v=gs(a)+a; %s return v; }"
             % (stmts_fine(ka), stmts_fine(kb)))
    body = " ".join(["s=sba(s);"] * na + ["s=sbb(s);"] * nb)
    return (src_of(GS, [leads], "%s %s %s" % (INT_HEAD, body, INT_TAIL)),
            ["sba", "sbb"])


def run_sibling(mode, wd):
    o1 = "/O1" in mode
    print("=== sibling   does P's EXISTING expansion move the limit?")
    print("    subject `sba` is sized to sit exactly at its schedule limit;")
    print("    `sbb` is an unrelated callee at nB sites in the same P.")
    print("    %-3s %-3s %-7s %-7s %-7s  %s"
          % ("nA", "nB", "s(sba)", "s(sbb)", "P.text", "declined"))
    bad = 0
    for ka, kb, grid in ((8, 8, [(5, 0), (5, 1), (5, 2), (5, 3), (5, 5),
                                 (6, 0), (6, 1)]),
                         (8, 40, [(5, 0), (5, 1), (5, 2), (4, 2)])):
        for na, nb in grid:
            src, watch = sib_source(na, nb, ka, kb)
            o = capture(src, mode, wd, "sib_%d_%d_%d_%d" % (ka, kb, na, nb))
            if o is None:
                print("    capture failed")
                bad += 1
                continue
            r = read(o)
            if "error" in r:
                bad += 1
                continue
            d = declined(r["rel"], watch)
            sa, sb = size_of(r["emit"], "sba"), size_of(r["emit"], "sbb")
            print("    %-3d %-3d %-7s %-7s %-7s  %s"
                  % (na, nb, sa, sb, r["tsize"],
                     ", ".join("%s*%d" % (w, c) for w, c in sorted(d.items()))
                     or "- (everything inlined)"))
        print()
    if o1:
        print("    SCHEDULE D is a per-PAIR claim: sba at s=80 takes 5 sites,")
        print("    sbb at s=80 takes 5 and at s=208 takes 1, INDEPENDENTLY.")
        print("    A row where sba is declined at nA<=5 refutes that.")
    return bad


def ladder_source(tree, gen, k, n):
    leads, site, watch = tree(gen(k))
    body = " ".join([site] * n)
    probe = "%s %s %s" % (INT_HEAD, body, INT_TAIL)
    return src_of(GS, [leads], probe), watch


# --------------------------------------------------------------------------
def sweep_cell(src, watch, mode, wd, tag):
    o = capture(src, mode, wd, tag)
    if o is None:
        return None
    r = read(o)
    if "error" in r:
        return None
    r["declined"] = declined(r["rel"], watch)
    return r


def size_of(emit, want):
    """The emitted `.text` size of one planted callee, by (rough) name."""
    for nm, sz in emit.items():
        d = demangle_ish(nm)
        if d == want or d.endswith(want) or want in nm:
            return sz
    return None


def grade_d(nfull, s, nmax, o1):
    """SCHEDULE D's verdict for one rung, plus the retired readings."""
    if not o1:
        return "sched D: /Ox is a DIFFERENT MECHANISM — not graded here"
    p = law_d(s)
    if p is None:
        return ("sched D: unbounded (s<=%d)%s"
                % (LAW_D_TABLE[0][0],
                   "" if nfull >= nmax else "   <== *** REFUTES SCHEDULE D ***"))
    got = min(p, nmax)
    if got == nfull:
        v = "sched D %d OK" % p
    else:
        v = "sched D %s vs %d   <== *** REFUTES SCHEDULE D ***" % (p, nfull)
    for label, fn in SUPERSEDED_D:
        r = fn(s)
        r = nmax if r is None else min(r, nmax)
        if got == nfull and r != nfull:
            v += "   [retired '%s' said %d]" % (label, r)
    return v


def run_ladder(name, mode, wd, nmax):
    tree, gen, kmax, note = LADDERS[name]
    o1 = "/O1" in mode
    print("=== %-18s %s" % (name, note))
    print("    %-3s %-5s %-6s %-6s %-6s  %-26s %s"
          % ("k", "Nfull", "s(dir)", "s(in)", "P@N=1", "declined per N (1..%d)"
             % nmax, "SCHED D"))
    bad, out, refuted = 0, [], 0
    for k in range(kmax + 1):
        per_n, nfull, ptext = [], 0, None
        dsz = isz = None
        watch = tree(gen(k))[2]
        for n in range(1, nmax + 1):
            src, watch = ladder_source(tree, gen, k, n)
            r = sweep_cell(src, watch, mode, wd,
                           "%s_k%d_n%d" % (name.replace("-", "_"), k, n))
            if r is None:
                per_n.append("!")
                bad += 1
                continue
            if n == 1:
                ptext = r["tsize"]
                dsz = size_of(r["emit"], watch[0])
                isz = size_of(r["emit"], watch[-1])
            d = r["declined"]
            tot = sum(d.values())
            per_n.append("." if tot == 0
                         else "".join("%s%d" % (w[0], c) for w, c in
                                      sorted(d.items())))
            if tot == 0 and nfull == n - 1:
                nfull = n
        v = grade_d(nfull, dsz, nmax, o1)
        if "REFUTES" in v:
            refuted += 1
        print("    %-3d %-5d %-6s %-6s %-6s  %-26s %s"
              % (k, nfull, dsz, isz, ptext, " ".join(per_n), v))
        out.append((k, nfull, dsz, isz, ptext))
    print()
    return bad, refuted


# --------------------------------------------------------------------------
# the existing gt_label_inline families, read with the new detector
# --------------------------------------------------------------------------
def direct_callee(site):
    """The identifier P actually calls at its site — `s=lsb(s);` -> `lsb`."""
    out, cur = [], ""
    for ch in site:
        if ch.isalnum() or ch == "_":
            cur += ch
        else:
            if ch == "(" and cur and not cur.isdigit():
                out.append(cur)
            cur = ""
    # the barrier `gs` is present in both variants and is never the subject
    return [c for c in out if c not in ("gs", "gt", "gu", "ga")]


def run_family(fam, mode, wd, nmax):
    """One gt_label_inline family, read with the relocation detector.

    Grading these against SCHEDULE D is a HELD-OUT test: the schedule was
    fitted entirely on `int` ladders, and these are the destructor,
    constructor and depth-3 shapes §6.9-§6.14 were built on.
    """
    print("=== %-18s %s" % (fam.name, fam.note[:66]))
    print("    %-3s %-7s %-7s %-7s  %-26s %s"
          % ("N", "P.text", "dtext", "hand-dt", "declined (reloc)", "verdicts"))
    bad = 0
    want = direct_callee(fam.site)
    prev = prevh = None
    ndir, dsz = 0, None
    for n in range(0, nmax + 1):
        o = capture(fam.source(n, "inl"), mode, wd,
                    "f_%s_i_%d" % (fam.name.replace("-", "_"), n))
        oh = capture(fam.source(n, "hand"), mode, wd,
                     "f_%s_h_%d" % (fam.name.replace("-", "_"), n))
        if o is None or oh is None:
            print("    %-3d capture failed" % n)
            bad += 1
            continue
        r, rh = read(o), read(oh)
        if "error" in r or "error" in rh:
            bad += 1
            continue
        # everything P relocates against that is NOT the opaque barrier or a
        # runtime helper is a call the front end did not inline
        keep = {nm: c for nm, c in r["rel"].items()
                if not nm.startswith(("gs", "gt", "gu", "ga", "__",
                                      "_fltused", "$", ".", "?a0@", "?a1@",
                                      "?a2@"))
                and demangle_ish(nm) not in ("gs", "gt", "gu", "ga")}
        if n == 1 and want:
            dsz = size_of(r["emit"], want[0])
        if n >= 1:
            nd = sum(c for nm, c in keep.items()
                     if want and (demangle_ish(nm) == want[0]
                                  or want[0] in nm))
            if nd == 0 and ndir == n - 1:
                ndir = n
        dt = None if prev is None else r["tsize"] - prev
        hdt = None if prevh is None else rh["tsize"] - prevh
        old = ""
        if n >= 1 and dt is not None:
            if dt <= 0 or (hdt and hdt > 0 and dt * 2 <= hdt):
                old = "OLD:DECLINED?"
        new_ = "NEW:declined" if keep else "NEW:all-inlined"
        agree = "" if (bool(keep) == bool(old)) else "   <== DETECTORS DISAGREE"
        print("    %-3d %-7d %-7s %-7s  %-26s %s %s%s"
              % (n, r["tsize"], dt, hdt,
                 ",".join("%s*%d" % (demangle_ish(k)[:14], v)
                          for k, v in sorted(keep.items())) or "-",
                 old or "OLD:ok", new_, agree))
        prev, prevh = r["tsize"], rh["tsize"]
    print("    -> direct callee %s  s=%s   Ndirect=%d   %s"
          % (want[0] if want else "?", dsz, ndir,
             grade_d(ndir, dsz, nmax, "/O1" in mode)))
    print()
    return bad


def main(argv):
    if "--list" in argv:
        for k, (_, _, kmax, note) in sorted(LADDERS.items()):
            print("%-20s k=0..%-3d %s" % (k, kmax, note))
        return 0
    mode, nmax = "/O1 /GS- /c", 6
    fams = []
    if "--mode" in argv:
        i = argv.index("--mode"); mode = argv[i + 1]; del argv[i:i + 2]
    if "--max" in argv:
        i = argv.index("--max"); nmax = int(argv[i + 1]); del argv[i:i + 2]
    if "--kmax" in argv:
        i = argv.index("--kmax"); kcap = int(argv[i + 1]); del argv[i:i + 2]
        for nm in list(LADDERS):
            t, g, km, nt = LADDERS[nm]
            LADDERS[nm] = (t, g, min(km, kcap), nt)
    while "--family" in argv:
        i = argv.index("--family"); fams.append(argv[i + 1]); del argv[i:i + 2]
    want = [a for a in argv[1:] if not a.startswith("--")]

    print("mode: %s   N = call sites of the SAME body, chained" % mode)
    print("  Nfull  = the largest N at which EVERY site was inlined.")
    print("           0 means the front end declined from the FIRST site.")
    print("  the per-N column is the number of `bl`s P kept against each")
    print("  planted callee: `.` = none, i.e. the whole tree was inlined.")
    print("  in.text/ou.text = the callee COMDATs' own sizes (§6.5: they are")
    print("           emitted whether or not they were inlined), a free")
    print("           code-size proxy for the front end's own cost estimate.")
    print()
    wd = tempfile.mkdtemp(prefix="gtdec")
    bad = 0
    if "--sibling" in argv:
        bad += run_sibling(mode, wd)
        print("captures failed: %d" % bad)
        return 1 if bad else 0
    if "--cases" in argv:
        bad += run_cases(mode, wd, nmax, want)
        print("captures failed: %d" % bad)
        return 1 if bad else 0
    if fams:
        byname = {f.name: f for f in FAMILIES}
        for f in fams:
            if f not in byname:
                print("no such family: %s" % f)
                continue
            bad += run_family(byname[f], mode, wd, nmax)
    else:
        for k in sorted(LADDERS):
            if want and k not in want:
                continue
            b, _ = run_ladder(k, mode, wd, nmax)
            bad += b
    print("captures failed: %d" % bad)
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
