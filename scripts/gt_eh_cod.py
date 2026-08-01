#!/usr/bin/env python3
"""gt_eh_cod.py — boards #133 and #138: the EH record layout and the label-number
gaps, read off c2's own `.cod` assembly listing.

`docs/EH_RECORDS.md` §8.3 decoded the EH `.rdata` from **obj bytes**. The
listing seam (board #132) makes c2 print the same records with **field
boundaries and its own symbol names**, so this script is a *second,
name-carrying source* for a layout already derived — the #136 relationship
(ROADMAP §9.9.3). The byte model is therefore a control that can go red, which a
blank page would not have been.

Two jobs:

  #133  Decode every EH-owned data COMDAT into NAMED fields and check
        TOTALITY — every datum claimed by exactly one field, with any residue
        PRINTED rather than counted. Graded on shapes the layout was not fitted
        on, and cross-checked against §8.3's byte-derived `FuncInfo`.

  #138  §9.12 measured the inter-stage label gaps at 2..11 and 0..3 and refused
        to model them. This measures what governs them, by holding the EH shape
        fixed and varying ONE `LABEL_COUNTER.md` §1.1 surcharge at a time —
        including a surcharge that is measured to cost ZERO (a string literal),
        which is the cell that can refute the model.

Usage:
    scripts/gt_eh_cod.py gen                # write both probe corpora
    scripts/gt_eh_cod.py scan [--jobs N]    # capture .cod for every (probe,mode)
    scripts/gt_eh_cod.py records [probe]    # #133: the decoded records
    scripts/gt_eh_cod.py totality           # #133: A1, the residue, printed
    scripts/gt_eh_cod.py predict            # #133: A2, held-out cells scored
    scripts/gt_eh_cod.py gaps               # #138: the gap table
    scripts/gt_eh_cod.py grade              # everything, scored

Everything lands under `work/weh/` (gitignored). std-lib only; this is tooling,
outside the std-only Rust workspace.
"""

import json
import os
import re
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
WORK = os.path.join(ROOT, "work", "weh")
PROBES = os.path.join(WORK, "probes")
CODS = os.path.join(WORK, "cod")
ROWS = os.path.join(WORK, "rows.jsonl")
C2RS = os.path.join(ROOT, "target", "release", "c2rs")

MODES = {
    "EHsc-O1": ["/O1", "/Oi", "/EHsc", "/GS-", "/c"],
    "EHa-O1": ["/O1", "/Oi", "/EHa", "/GS-", "/c"],
    "EHsc-O2": ["/O2", "/EHsc", "/GS-", "/c"],
    "EHsc-Ox": ["/Ox", "/EHsc", "/GS-", "/c"],
}

# ---------------------------------------------------------------------------
# #133 corpus. The axes are STRUCTURAL COUNTS, per ROADMAP §9.13.1 consequence
# 2: a generated axis is only as good as the axes it varies, and a block that
# varied argument VALUES but never ARITY hid a mis-emit. So: number of try
# blocks, nesting depth, catches per try, destructible objects, functions per
# TU — not just the contents of one try.
#
# `fit` = looked at while the layout was written. Everything else is HELD OUT
# and its A2 prediction is registered in PREDICT below before capture.
# ---------------------------------------------------------------------------
COMMON = """
struct T { T(); ~T(); int v; };
struct E1 { int a; };
struct E2 { int b; };
struct V { V(); V(const V&); ~V(); int c; };
int mk(int);
int use(int);
"""

EH_SRC = {
    # ---- FIT ---------------------------------------------------------------
    "fit_1try2catch_dtor_loop": ("fit", """
int f(int n) {
  T t; int s = 0;
  try { s = mk(n); } catch (int e) { s = e; } catch (...) { s = -1; }
  for (int i = 0; i < n; ++i) s += use(i);
  return s + t.v;
}
"""),
    "fit_nested2": ("fit", """
int f(int n) {
  T a; int s = 0;
  try {
    T b;
    try { s = mk(n); }
    catch (E1& e) { s = e.a; }
    catch (const E2& e) { s = e.b; }
    s += b.v;
  } catch (int e) { s = e; }
  catch (...) { s = -2; }
  return s + a.v;
}
"""),

    # ---- HELD OUT: the destructible-object axis, no try at all -------------
    "h_dtor1": ("held", """
int f(int n) { T a; return mk(n) + a.v; }
"""),
    "h_dtor2": ("held", """
int f(int n) { T a; T b; return mk(n) + a.v + b.v; }
"""),
    "h_dtor3": ("held", """
int f(int n) { T a; T b; T c; return mk(n) + a.v + b.v + c.v; }
"""),

    # ---- HELD OUT: the try-block-COUNT axis, sequential (not nested) -------
    "h_try1": ("held", """
int f(int n) {
  int s = 0;
  try { s = mk(n); } catch (int e) { s = e; }
  return s;
}
"""),
    "h_try2seq": ("held", """
int f(int n) {
  int s = 0;
  try { s = mk(n); } catch (int e) { s = e; }
  try { s += mk(n + 1); } catch (int e) { s -= e; }
  return s;
}
"""),
    "h_try3seq": ("held", """
int f(int n) {
  int s = 0;
  try { s = mk(n); } catch (int e) { s = e; }
  try { s += mk(n + 1); } catch (int e) { s -= e; }
  try { s += mk(n + 2); } catch (int e) { s ^= e; }
  return s;
}
"""),

    # ---- HELD OUT: the nesting-DEPTH axis ---------------------------------
    "h_nest3": ("held", """
int f(int n) {
  int s = 0;
  try {
    try {
      try { s = mk(n); } catch (E1& e) { s = e.a; }
    } catch (E2& e) { s = e.b; }
  } catch (int e) { s = e; }
  return s;
}
"""),

    # ---- HELD OUT: the catches-per-try axis -------------------------------
    "h_catch4": ("held", """
int f(int n) {
  int s = 0;
  try { s = mk(n); }
  catch (V v) { s = v.c; }
  catch (const E1& e) { s = e.a; }
  catch (E2& e) { s = e.b; }
  catch (...) { s = -1; }
  return s;
}
"""),
    "h_catch_ptr": ("held", """
int f(int n) {
  int s = 0;
  try { s = mk(n); } catch (const char* volatile p) { s = p ? 1 : 0; }
  return s;
}
"""),

    # ---- HELD OUT: functions-per-TU axis -----------------------------------
    "h_2fn": ("held", """
int f(int n) { int s = 0; try { s = mk(n); } catch (int e) { s = e; } return s; }
int g(int n) { int s = 0; try { s = mk(n); } catch (E1& e) { s = e.a; } return s; }
"""),
    "h_3fn_mixed": ("held", """
int p(int x) { return x + 1; }
int f(int n) { T a; return mk(n) + a.v; }
int g(int n) { int s = 0; try { s = mk(n); } catch (...) { s = -1; } return s; }
"""),

    # ---- HELD OUT: dtor AND try together, and rethrow ---------------------
    "h_dtor2_try2catch": ("held", """
int f(int n) {
  T a; T b; int s = 0;
  try { s = mk(n); } catch (int e) { s = e; } catch (...) { s = -1; }
  return s + a.v + b.v;
}
"""),
    "h_rethrow": ("held", """
int f(int n) {
  int s = 0;
  try { s = mk(n); } catch (int e) { if (e > 0) throw; s = e; }
  return s;
}
"""),
}

# ---------------------------------------------------------------------------
# #133 / A2 — registered BEFORE capture. Per held-out probe, predicted from the
# SOURCE alone: nTryBlocks, nIPMapEntries, maxState, the number of __catchsym$
# arrays, the HandlerType count in each (as a sorted list), and the number of
# funclets. `None` = declined rather than guessed; a declined cell scores 0 and
# is reported as declined, never as a pass.
# ---------------------------------------------------------------------------
PREDICT = {
    #                    nTry  maxState  nIP  nCatchsym  handlers   funclets
    "h_dtor1":          dict(nTryBlocks=0, maxState=1, nIPMapEntries=2,
                             nCatchsym=0, handlers=[], nFunclets=1),
    "h_dtor2":          dict(nTryBlocks=0, maxState=2, nIPMapEntries=4,
                             nCatchsym=0, handlers=[], nFunclets=2),
    "h_dtor3":          dict(nTryBlocks=0, maxState=3, nIPMapEntries=6,
                             nCatchsym=0, handlers=[], nFunclets=3),
    "h_try1":           dict(nTryBlocks=1, maxState=1, nIPMapEntries=None,
                             nCatchsym=1, handlers=[1], nFunclets=1),
    "h_try2seq":        dict(nTryBlocks=2, maxState=2, nIPMapEntries=None,
                             nCatchsym=2, handlers=[1, 1], nFunclets=2),
    "h_try3seq":        dict(nTryBlocks=3, maxState=3, nIPMapEntries=None,
                             nCatchsym=3, handlers=[1, 1, 1], nFunclets=3),
    "h_nest3":          dict(nTryBlocks=3, maxState=3, nIPMapEntries=None,
                             nCatchsym=3, handlers=[1, 1, 1], nFunclets=3),
    "h_catch4":         dict(nTryBlocks=1, maxState=2, nIPMapEntries=None,
                             nCatchsym=1, handlers=[4], nFunclets=4),
    "h_catch_ptr":      dict(nTryBlocks=1, maxState=1, nIPMapEntries=None,
                             nCatchsym=1, handlers=[1], nFunclets=1),
    "h_2fn":            dict(nTryBlocks=1, maxState=1, nIPMapEntries=None,
                             nCatchsym=1, handlers=[1], nFunclets=1),
    "h_3fn_mixed":      dict(nTryBlocks=0, maxState=1, nIPMapEntries=2,
                             nCatchsym=0, handlers=[], nFunclets=1),
    "h_dtor2_try2catch": dict(nTryBlocks=1, maxState=4, nIPMapEntries=None,
                              nCatchsym=1, handlers=[2], nFunclets=4),
    "h_rethrow":        dict(nTryBlocks=1, maxState=1, nIPMapEntries=None,
                             nCatchsym=1, handlers=[1], nFunclets=1),
}
# For multi-function TUs the prediction above is for the FIRST EH function in
# `.text` order (`h_2fn`'s `f`, `h_3fn_mixed`'s `f`).

# ---------------------------------------------------------------------------
# ROUND 2. Every A2 miss above was `maxState`, and all of them in one direction:
# a try block is worth **2** states, not 1. Corrected law, FITTED on round 1:
#
#     maxState = (destructible objects) + 2 x (lexical try blocks)
#
# It is exact on all 13 round-1 cells — which is worth nothing on its own,
# because it was derived from them. These five shapes are NEW, their predictions
# are registered here before capture, and they are the only cells the law is
# graded on. `z_deep4` is the one designed to break it: four levels of nesting,
# where "2 per try" and "2 per nesting level" stop agreeing with anything the
# round-1 corpus could separate.
# ---------------------------------------------------------------------------
Z_SRC = {
    "z_nest2_dtor2": """
int f(int n) {
  T a; T b; int s = 0;
  try { try { s = mk(n); } catch (E1& e) { s = e.a; } }
  catch (int e) { s = e; }
  return s + a.v + b.v;
}
""",                                   # 2 dtors + 2 try  -> 2 + 4 = 6
    "z_try4seq": """
int f(int n) {
  int s = 0;
  try { s = mk(n); } catch (int e) { s = e; }
  try { s += mk(n+1); } catch (int e) { s -= e; }
  try { s += mk(n+2); } catch (int e) { s ^= e; }
  try { s += mk(n+3); } catch (int e) { s |= e; }
  return s;
}
""",                                   # 0 dtors + 4 try  -> 0 + 8 = 8
    "z_try1catch4_dtor3": """
int f(int n) {
  T a; T b; T c; int s = 0;
  try { s = mk(n); }
  catch (V v) { s = v.c; } catch (const E1& e) { s = e.a; }
  catch (E2& e) { s = e.b; } catch (...) { s = -1; }
  return s + a.v + b.v + c.v;
}
""",                                   # 3 dtors + 1 try  -> 3 + 2 = 5
    "z_dtor5": """
int f(int n) {
  T a; T b; T c; T d; T e;
  return mk(n) + a.v + b.v + c.v + d.v + e.v;
}
""",                                   # 5 dtors + 0 try  -> 5 + 0 = 5
    "z_deep4": """
int f(int n) {
  int s = 0;
  try { try { try { try { s = mk(n); }
    catch (E1& e) { s = e.a; } } catch (E2& e) { s = e.b; } }
    catch (int e) { s = e; } } catch (...) { s = -1; }
  return s;
}
""",                                   # 0 dtors + 4 try  -> 0 + 8 = 8
}

PREDICT2 = {
    "z_nest2_dtor2":      dict(maxState=6, nTryBlocks=2),
    "z_try4seq":          dict(maxState=8, nTryBlocks=4),
    "z_try1catch4_dtor3": dict(maxState=5, nTryBlocks=1),
    "z_dtor5":            dict(maxState=5, nTryBlocks=0),
    "z_deep4":            dict(maxState=8, nTryBlocks=4),
}

# ---------------------------------------------------------------------------
# #138 corpus. ONE axis moves per probe against `g_base`. The EH shape is held
# fixed at "one destructible local, no try" — `EH_RECORDS.md` §9.8's own shape,
# where its model `G = 4 + Sum(minting surcharges)` is claimed exact on 25/27.
# ---------------------------------------------------------------------------
GAP_COMMON = """
struct T { T(); ~T(); int v; };
int mk(int);
int use(int);
int mk2(int, int);
"""

GAP_SRC = {
    # the baseline: 1 destructible local, no try.
    "g_base": """
int f(int n) { T a; return mk(n) + a.v; }
""",
    # +_fltused: the first FP-touching function in the TU.
    "g_flt": """
int f(int n) { T a; float x = (float)n; return mk((int)(x + x)) + a.v; }
""",
    # +a newly pooled FP constant (distinct (bits,width)).
    "g_pool": """
int f(int n) { T a; float x = (float)n * 2.5f; return mk((int)x) + a.v; }
""",
    # +a loop. LABEL_COUNTER.md §2.1: costs slots and MINTS NOTHING.
    "g_loop": """
int f(int n) { T a; int s = 0; for (int i = 0; i < n; ++i) s += use(i); return s + a.v; }
""",
    # +a signed relational over two call results. §1.1: +2, MINTS NOTHING.
    "g_cmprr": """
int f(int n) { T a; int s = (mk(n) < use(n)) ? 1 : 0; return s + a.v; }
""",
    # THE CONTROL. A string literal makes an .rdata COMDAT and a `??_C@` symbol
    # and costs ZERO slots (§2.1). If G moves here, "G is the §1.1 surcharge
    # block" is REFUTED. This is the cell that can go red.
    "g_str": """
const char* gp;
int f(int n) { T a; gp = "hello"; return mk(n) + a.v; }
""",
    # +a second distinct gprlr width, introduced by a SECOND function.
    "g_gpr2": """
int wide(int a,int b,int c,int d,int e,int f2,int g,int h) { return mk2(a+b+c+d, e+f2+g+h); }
int f(int n) { T a; return mk(n) + a.v; }
""",
    # led: an EH function already in the TU, so __CxxFrameHandler is paid.
    "g_led": """
int lead(int n) { T z; return mk(n) + z.v; }
int f(int n) { T a; return mk(n) + a.v; }
""",
    # phases that emit nothing: an unreferenced static, which c2 discards
    # (ROADMAP §9.5's `globally unreferenced` disjunct).
    "g_static": """
static int dead(int x) { return x * 3 + 1; }
int f(int n) { T a; return mk(n) + a.v; }
""",
    "g_static4": """
static int d1(int x) { return x * 3 + 1; }
static int d2(int x) { return x * 5 + 2; }
static int d3(int x) { return x * 7 + 3; }
static int d4(int x) { return x * 9 + 4; }
int f(int n) { T a; return mk(n) + a.v; }
""",
    # the try/catch shape, to test whether G behaves as it does with no try.
    "g_try": """
int f(int n) { T a; int s = 0; try { s = mk(n); } catch (int e) { s = e; } return s + a.v; }
""",
}

# k in-TU callees, EH shape fixed. NOTE (measured, not assumed): these are NOT
# inlined away — c2 emits every one as its own COMDAT. So this ladder moves TWO
# things at once: the number of preceding emitted functions AND the number of
# calls inside `f`. It is kept because it is what was run first, and the two
# ladders below are the separation it needed. ROADMAP §9.13.1 consequence 2.
for _k in range(0, 5):
    _calls = " + ".join("inl%d(n)" % _i for _i in range(_k)) or "0"
    _defs = "\n".join("static int inl%d(int x) { return x + %d; }" % (_i, _i)
                      for _i in range(_k))
    GAP_SRC["g_inl%d" % _k] = """
%s
int f(int n) { T a; return mk(n) + a.v + (%s); }
""" % (_defs, _calls)

# LADDER A — k preceding emitted LEAF functions that `f` does NOT call. `f` is
# byte-identical to `g_base`'s. Isolates "another emitted function in the TU".
for _k in range(0, 5):
    _defs = "\n".join("int lead%d(int x) { return x + %d; }" % (_i, _i)
                      for _i in range(_k))
    GAP_SRC["x_lead%d" % _k] = """
%s
int f(int n) { T a; return mk(n) + a.v; }
""" % _defs

# LADDER B — k extra calls inside `f` to functions DECLARED ELSEWHERE, so the TU
# gains no function. Isolates "another call in the EH body".
for _k in range(0, 5):
    _decls = "\n".join("int ext%d(int);" % _i for _i in range(_k))
    _calls = " + ".join("ext%d(n)" % _i for _i in range(_k)) or "0"
    GAP_SRC["x_call%d" % _k] = """
%s
int f(int n) { T a; return mk(n) + a.v + (%s); }
""" % (_decls, _calls)

# LADDER A' — THE DISCRIMINATOR for ladder A's -2. The same leaf functions, but
# AFTER `f` in .text order. If G stays 4 the charge is "the FIRST emitted
# function in the TU pays it"; if G drops to 2 the charge is "the TU has more
# than one function". Ladder A alone cannot tell these apart — it is the control
# run where the discrepancy cannot appear (ROADMAP §9.1, twelfth instance).
for _k in range(1, 5):
    _defs = "\n".join("int trail%d(int x) { return x + %d; }" % (_i, _i)
                      for _i in range(_k))
    GAP_SRC["x_trail%d" % _k] = """
int f(int n) { T a; return mk(n) + a.v; }
%s
""" % _defs

# LADDER D — the brief's "labels consumed by bodies INLINED AWAY". `g_inl*`
# above does NOT test this: c2 emitted every one of its callees as its own
# COMDAT, which was measured rather than assumed. `__forceinline` is the axis
# that actually removes the body, and the `PROC` count is asserted in the report
# so a ladder that silently emits its callees again cannot be read as a result.
for _k in range(0, 5):
    _calls = " + ".join("fi%d(n)" % _i for _i in range(_k)) or "0"
    _defs = "\n".join(
        "__forceinline int fi%d(int x) { return x + %d; }" % (_i, _i)
        for _i in range(_k))
    GAP_SRC["x_fi%d" % _k] = """
%s
int f(int n) { T a; return mk(n) + a.v + (%s); }
""" % (_defs, _calls)

# ---------------------------------------------------------------------------
# #138 ROUND 2 — the decomposition, graded on COMBINATIONS it was not fitted on.
# Fitted on the single-axis ladders above:
#
#   G = 2  +  2 x [f is the FIRST emitted function in the TU]
#          +  Sum( f's own LABEL_COUNTER.md §1.1-style surcharges )
#
#   measured terms:  string literal +0 · discarded static +0 · signed relational
#   over two call results +2 · _fltused + a newly pooled FP constant +3 ·
#   a loop +4 · one-or-more extra calls to functions declared elsewhere +2
#   (FLAT in the count) · try/catch +3 · EACH BODY INLINED INTO f +3
#
# A single-axis ladder cannot say whether the terms ADD. These five do, and
# their predictions are registered here before capture.
# ---------------------------------------------------------------------------
Y_SRC = {
    # not first (1 leading fn) + a loop + 2 inlined  ->  2 + 0 + 4 + 6 = 12
    "y_loop_fi2_led": ("""
int lead0(int x) { return x + 1; }
__forceinline int fa(int x) { return x + 2; }
__forceinline int fb(int x) { return x + 3; }
int f(int n) { T a; int s = 0; for (int i = 0; i < n; ++i) s += use(i);
               return s + a.v + fa(n) + fb(n); }
""", 12),
    # not first (2 inlined lead) + signed relational  ->  2 + 0 + 6 + 2 = 10
    "y_cmprr_fi2": ("""
__forceinline int fa(int x) { return x + 2; }
__forceinline int fb(int x) { return x + 3; }
int f(int n) { T a; int s = (mk(n) < use(n)) ? 1 : 0;
               return s + a.v + fa(n) + fb(n); }
""", 10),
    # FIRST + a string literal (+0) + a loop (+4)     ->  2 + 2 + 0 + 4 = 8
    "y_str_loop": ("""
const char* gp;
int f(int n) { T a; int s = 0; gp = "hello";
               for (int i = 0; i < n; ++i) s += use(i); return s + a.v; }
""", 8),
    # not first + 2 extra calls to functions declared elsewhere -> 2 + 0 + 2 = 4
    "y_lead_call2": ("""
int e0(int); int e1(int);
int lead0(int x) { return x + 1; }
int f(int n) { T a; return mk(n) + a.v + e0(n) + e1(n); }
""", 4),
    # not first + _fltused + a pooled FP constant     ->  2 + 0 + 3 = 5
    "y_pool_led": ("""
int lead0(int x) { return x + 1; }
int f(int n) { T a; float x = (float)n * 2.5f; return mk((int)x) + a.v; }
""", 5),
}

# LADDER C — k preceding DISCARDED statics. Nothing of theirs reaches the obj.
for _k in (0, 1, 2, 4, 8):
    _defs = "\n".join("static int dead%d(int x) { return x * %d + 1; }"
                      % (_i, _i + 3) for _i in range(_k))
    GAP_SRC["x_dead%d" % _k] = """
%s
int f(int n) { T a; return mk(n) + a.v; }
""" % _defs


def gen():
    os.makedirs(PROBES, exist_ok=True)
    n = 0
    for name, (_g, src) in EH_SRC.items():
        with open(os.path.join(PROBES, name + ".cpp"), "w") as fh:
            fh.write(COMMON.lstrip() + src.lstrip())
        n += 1
    for name, src in GAP_SRC.items():
        with open(os.path.join(PROBES, name + ".cpp"), "w") as fh:
            fh.write(GAP_COMMON.lstrip() + src.lstrip())
        n += 1
    for name, src in Z_SRC.items():
        with open(os.path.join(PROBES, name + ".cpp"), "w") as fh:
            fh.write(COMMON.lstrip() + src.lstrip())
        n += 1
    for name, (src, _g) in Y_SRC.items():
        with open(os.path.join(PROBES, name + ".cpp"), "w") as fh:
            fh.write(COMMON.lstrip() + src.lstrip())
        n += 1
    print("wrote %d probes to %s" % (n, PROBES))


# ---------------------------------------------------------------------------
# parse
# ---------------------------------------------------------------------------
RE_PROC = re.compile(r"^(\S+)\s+PROC\s+NEAR")
RE_ENDP = re.compile(r"^(\S+)\s+ENDP")
RE_SEG = re.compile(r"^(\S+)\s+SEGMENT")
RE_SEG_DOT = re.compile(r"^\s+(\.[A-Za-z$0-9_]+)\s*$")
RE_INSN = re.compile(r"^\s+([0-9a-f]{5,8})\s+([0-9a-f]{8})\s")
# a data definition that OPENS a named record:  `SYM DD  value`
RE_NAMED_DATUM = re.compile(r"^(\S+)\s+(DD|DW|DB|DQ)\s+(.*)$")
# a continuation datum:  `\tDD\tvalue`
RE_CONT_DATUM = re.compile(r"^\s+(DD|DW|DB|DQ)\s+(.*)$")
RE_ORG = re.compile(r"^\s+ORG\s+\$\+(\d+)\s*$")
# `DD 2 DUP(00H)` — c2 RUN-LENGTH ENCODES a run of equal data. Missing this
# silently SHORTENS a record: `__ehfuncinfo$` prints 8 tokens for its 9 dwords
# whenever `nTryBlocks` and `pTryBlockMap` are both 0, which is every function
# with a destructor and no try. Every field after it then reads one slot early —
# a plausible decode, with `pIPtoStateMap` landing on `nIPMapEntries`.
RE_DUP = re.compile(r"^(\d+)\s+DUP\s*\((.*)\)\s*$")


def expand(tok):
    """One listing operand -> the list of data it actually stands for."""
    t = tok.strip().rstrip(",")
    m = RE_DUP.match(t)
    if m:
        return [m.group(2).strip()] * int(m.group(1))
    return [t]
# every label DEFINITION that carries a number from c2's TU-wide counter
RE_NUMBERED_LABEL = re.compile(r"^(\$M|\$T)(\d+):")
RE_FUNCLET = re.compile(r"^(__catch\$|__unwind\$|__tryend\$)(\d+):")
# a numbered label that is DEFINED as data (`$T2606\tDD\t?f@@YAHH@Z`)
RE_DATA_LABEL = re.compile(r"^(\$M|\$T)(\d+)\s+DD\s+(\S+)")
# per-function local labels, a DIFFERENT and small number space
RE_LOCAL_LABEL = re.compile(r"^(\$LN|\$LL|\$L)(\d+)@(\S+):")


def parse_cod(path):
    """One listing -> (a) the named data records in emission order, (b) every
    label carrying a TU-counter number, (c) the local `$LN`/`$LL` labels, which
    are a separate space, (d) the PROC list."""
    with open(path, "r", errors="replace") as fh:
        lines = fh.read().splitlines()

    records = []      # {name, seg, data:[str], pad_before:[int]}
    cur = None
    seg = None
    pad_next = []
    labels = []       # TU-counter labels
    locals_ = []      # $LN/$LL
    funcs = []
    fn = None
    fn_ix = -1
    body = None
    pending = []
    last_off = None

    def flush(off):
        for p in pending:
            p["text_off"] = off
        del pending[:]

    def close():
        nonlocal cur
        if cur is not None:
            records.append(cur)
            cur = None

    for raw in lines:
        line = raw.rstrip("\n")
        if line.startswith(";"):
            continue
        m = RE_SEG.match(line)
        if m:
            seg = m.group(1)
            close()
            continue
        m = RE_SEG_DOT.match(line)
        if m:
            seg = m.group(1)
            close()
            continue
        m = RE_PROC.match(line)
        if m:
            close()
            fn = m.group(1)
            fn_ix += 1
            funcs.append(fn)
            body = "main"
            seg = ".code"
            last_off = None
            continue
        m = RE_ENDP.match(line)
        if m:
            # gt_label_cod.py instrument defect 1: a body-end `$M` sits just
            # before ENDP and belongs to THIS function, at one past its last
            # instruction — not to the next function, which under /Gy restarts
            # its offsets at 0.
            flush((last_off + 4) if last_off is not None else None)
            fn, body, last_off = None, None, None
            continue
        if line.strip() == ".endprolog":
            continue
        m = RE_FUNCLET.match(line)
        if m:
            name = m.group(1) + m.group(2)
            body = name
            labels.append(dict(num=int(m.group(2)), kind="funclet", where="code",
                               name=name, fn=fn, fn_ix=fn_ix, body=name,
                               text_off=None))
            pending.append(labels[-1])
            continue
        m = RE_NUMBERED_LABEL.match(line)
        if m:
            labels.append(dict(num=int(m.group(2)),
                               kind={"$M": "M", "$T": "T"}[m.group(1)],
                               where="code", name=m.group(1) + m.group(2),
                               fn=fn, fn_ix=fn_ix, body=body, text_off=None))
            pending.append(labels[-1])
            continue
        m = RE_LOCAL_LABEL.match(line)
        if m:
            locals_.append(dict(prefix=m.group(1), num=int(m.group(2)),
                                fn=m.group(3)))
            continue
        m = RE_INSN.match(line)
        if m:
            last_off = int(m.group(1), 16)
            flush(last_off)
            continue
        m = RE_ORG.match(line)
        if m:
            # `ORG $+4` is ALIGNMENT PADDING between two records, not a datum of
            # the record it follows. EH_RECORDS.md §8.3 derived this pad by
            # inference from symbol offsets ("pad 0 ... and pad 4, both
            # observed"); the listing states it outright. Carried as `pad_next`
            # so it is neither counted as data nor silently dropped.
            pad_next.append(int(m.group(1)))
            continue
        m = RE_DATA_LABEL.match(line)
        if m:
            # `$T2606 DD ?f@@YAHH@Z` in .pdata, or `$T2601 DD $M2591` in the EH
            # .rdata: a numbered label that OPENS a data record. It is both.
            labels.append(dict(num=int(m.group(2)),
                               kind={"$M": "M", "$T": "T"}[m.group(1)],
                               where="data", name=m.group(1) + m.group(2),
                               fn=None, fn_ix=None, body=None,
                               describes=m.group(3).rstrip("H,"),
                               text_off=None))
            close()
            cur = dict(name=m.group(1) + m.group(2), seg=seg,
                       data=expand(m.group(3)), pad_before=list(pad_next))
            del pad_next[:]
            continue
        m = RE_NAMED_DATUM.match(line)
        if m and fn is None:
            close()
            cur = dict(name=m.group(1), seg=seg, data=expand(m.group(3)),
                       pad_before=list(pad_next))
            del pad_next[:]
            continue
        m = RE_CONT_DATUM.match(line)
        if m and cur is not None:
            cur["data"] += expand(m.group(2))
            continue
    close()
    return dict(records=records, labels=labels, locals=locals_, funcs=funcs)


# ---------------------------------------------------------------------------
# #133 — decode the named records into named FIELDS.
# ---------------------------------------------------------------------------
def n_of(tok):
    """A `DD` operand as an int when it is a literal, else None (a relocation)."""
    t = tok.strip().rstrip(",")
    if re.fullmatch(r"0[0-9a-fA-F]*H", t):
        v = int(t[:-1], 16)
        return v - (1 << 32) if v >= (1 << 31) else v
    if re.fullmatch(r"\d+", t):
        return int(t)
    return None


FUNCINFO_FIELDS = ["magic", "maxState", "pUnwindMap", "nTryBlocks",
                   "pTryBlockMap", "nIPMapEntries", "pIPtoStateMap",
                   "pESTypeList", "EHFlags"]


def decode(rec):
    """One named record -> (kind, list of (field, value)) and any UNCLAIMED
    residue, which is returned as data rather than as a count."""
    nm, d = rec["name"], rec["data"]
    if nm.startswith("__ehfuncinfo$"):
        fields = [(FUNCINFO_FIELDS[i] if i < len(FUNCINFO_FIELDS)
                   else "UNCLAIMED[%d]" % i, v) for i, v in enumerate(d)]
        return "FuncInfo", fields
    if nm.startswith("__unwindtable$"):
        out, i = [], 0
        while i + 1 < len(d):
            out += [("[%d].toState" % (i // 2), d[i]),
                    ("[%d].action" % (i // 2), d[i + 1])]
            i += 2
        if i < len(d):
            out.append(("UNCLAIMED[%d]" % i, d[i]))
        return "UnwindMap", out
    if nm.startswith("__catchsym$"):
        out, i = [], 0
        F = ["adjectives", "pType", "dispCatchObj", "addressOfHandler"]
        while i + 3 < len(d):
            for j, f in enumerate(F):
                out.append(("[%d].%s" % (i // 4, f), d[i + j]))
            i += 4
        while i < len(d):
            out.append(("UNCLAIMED[%d]" % i, d[i]))
            i += 1
        return "HandlerTypeArray", out
    if nm.startswith("__tryblocktable$"):
        out, i = [], 0
        F = ["tryLow", "tryHigh", "catchHigh", "nCatches", "pHandlerArray"]
        while i + 4 < len(d):
            for j, f in enumerate(F):
                out.append(("[%d].%s" % (i // 5, f), d[i + j]))
            i += 5
        while i < len(d):
            out.append(("UNCLAIMED[%d]" % i, d[i]))
            i += 1
        return "TryBlockMap", out
    if re.fullmatch(r"\$T\d+", nm) and rec["seg"] == ".rdata":
        out, i = [], 0
        while i + 1 < len(d):
            out += [("[%d].ip" % (i // 2), d[i]),
                    ("[%d].state" % (i // 2), d[i + 1])]
            i += 2
        if i < len(d):
            out.append(("UNCLAIMED[%d]" % i, d[i]))
        return "IpToStateMap", out
    if re.fullmatch(r"\$T\d+", nm) and rec["seg"] == ".pdata":
        f = [("BeginAddress", d[0])]
        f += [("unwindWord", d[1])] if len(d) > 1 else []
        f += [("UNCLAIMED[%d]" % i, v) for i, v in enumerate(d[2:], 2)]
        return "PdataEntry", f
    if nm.startswith("??_R0"):
        f = [("pVFTable", d[0])]
        f += [("spare", d[1])] if len(d) > 1 else []
        f += [("name", d[2])] if len(d) > 2 else []
        f += [("UNCLAIMED[%d]" % i, v) for i, v in enumerate(d[3:], 3)]
        return "TypeDescriptor", f
    return None, []


EH_RECORD_PREFIXES = ("__ehfuncinfo$", "__unwindtable$", "__catchsym$",
                      "__tryblocktable$", "??_R0")


def eh_records(p):
    """Every record this lane claims is part of the EH record set."""
    out = []
    for r in p["records"]:
        if r["name"].startswith(EH_RECORD_PREFIXES):
            out.append(r)
        elif re.fullmatch(r"\$T\d+", r["name"]) and r["seg"] in (".rdata", ".pdata"):
            out.append(r)
    return out


# ---------------------------------------------------------------------------
# #138 — the allocation spans and the gaps between them.
# ---------------------------------------------------------------------------
def gap_analysis(p):
    """PER FUNCTION: the funclet block, the EH-state `$M` block, the state-table
    `$T`, the triples — and the UNNAMED slots between them.

    INSTRUMENT DEFECT, found and fixed here rather than reported as a result:
    the first version aggregated over the whole TU with `min`/`max`, so a TU with
    two EH functions interleaved their blocks and printed a NEGATIVE gap
    (`g_led`: -16, -17). A negative gap is at least loud; the same aggregation on
    a TU whose second function happened to sit above the first would have printed
    a plausible positive integer. Bind every block to its function by name."""
    labels = sorted(p["labels"], key=lambda l: l["num"])
    if not labels:
        return None
    named = set(l["num"] for l in labels)
    lo, hi = labels[0]["num"], labels[-1]["num"]

    # fn -> its ip2state $T, via `__ehfuncinfo$<fn>`'s pIPtoStateMap operand.
    fn_state_t, fn_ip_ms = {}, {}
    t_records = {}
    for r in p["records"]:
        if re.fullmatch(r"\$T\d+", r["name"]) and r["seg"] == ".rdata":
            t_records[r["name"]] = r
    for r in p["records"]:
        if not r["name"].startswith("__ehfuncinfo$"):
            continue
        fn = r["name"][len("__ehfuncinfo$"):]
        if len(r["data"]) > 6:
            tname = r["data"][6].strip().rstrip(",")
            if tname in t_records:
                fn_state_t[fn] = int(tname[2:])
                fn_ip_ms[fn] = sorted(
                    int(m.group(1)) for m in
                    (re.fullmatch(r"\$M(\d+)", t.strip().rstrip(","))
                     for t in t_records[tname]["data"]) if m)

    per_fn = {}
    for fn in set(list(fn_state_t) + [l["fn"] for l in labels if l["fn"]]):
        fl = sorted(l["num"] for l in labels
                    if l["kind"] == "funclet" and l["fn"] == fn)
        ip = fn_ip_ms.get(fn, [])
        st = fn_state_t.get(fn)
        own = sorted(l["num"] for l in labels if l["fn"] == fn)
        tri = sorted(n for n in own
                     if n not in fl and n not in ip and n != st)
        e = dict(funclets=fl, ip_ms=ip, state_t=st, triple_ms=tri)
        if fl and ip:
            e["G_funclet_to_ipM"] = min(ip) - max(fl) - 1
        if st is not None and tri:
            e["G_stateT_to_triple"] = min(tri) - st - 1
        per_fn[fn] = e

    return dict(lo=lo, hi=hi, span=hi - lo + 1, n_named=len(named),
                unnamed=[n for n in range(lo, hi + 1) if n not in named],
                per_fn=per_fn, funcs=p["funcs"],
                locals=sorted(set((l["prefix"], l["num"]) for l in p["locals"])))


# ---------------------------------------------------------------------------
# scan
# ---------------------------------------------------------------------------
def one(args):
    name, mode, flags = args
    cpp = os.path.join(PROBES, name + ".cpp")
    out = os.path.join(CODS, "%s.%s.cod" % (name, mode))
    cmd = [C2RS, "listing", cpp, "--out", out]
    for f in flags:
        cmd += ["--flag", f]
    r = subprocess.run(cmd, capture_output=True, text=True, cwd=ROOT)
    if r.returncode != 0 or not os.path.exists(out):
        return dict(probe=name, mode=mode, ok=False,
                    err=(r.stderr or r.stdout).strip()[:300])
    p = parse_cod(out)
    grp = EH_SRC[name][0] if name in EH_SRC else ("held2" if name in Z_SRC else ("gapheld" if name in Y_SRC else "gap"))
    return dict(probe=name, mode=mode, ok=True, group=grp, flags=flags,
                records=p["records"], labels=p["labels"], locals=p["locals"],
                funcs=p["funcs"], gaps=gap_analysis(p))


def scan(jobs=6):
    os.makedirs(CODS, exist_ok=True)
    work = []
    for name in EH_SRC:
        for mode, flags in MODES.items():
            work.append((name, mode, flags))
    for name in GAP_SRC:
        for mode in ("EHsc-O1",):
            work.append((name, mode, MODES[mode]))
    for name in Z_SRC:
        work.append((name, "EHsc-O1", MODES["EHsc-O1"]))
    for name in Y_SRC:
        work.append((name, "EHsc-O1", MODES["EHsc-O1"]))
    with ThreadPoolExecutor(max_workers=jobs) as ex:
        rows = list(ex.map(one, work))
    with open(ROWS, "w") as fh:
        for r in rows:
            fh.write(json.dumps(r) + "\n")
    ok = sum(1 for r in rows if r["ok"])
    print("captured %d/%d listings" % (ok, len(rows)))
    for r in rows:
        if not r["ok"]:
            print("  FAIL %s %s: %s" % (r["probe"], r["mode"], r["err"]))


def load():
    with open(ROWS) as fh:
        return [json.loads(l) for l in fh if l.strip()]


# ---------------------------------------------------------------------------
# report
# ---------------------------------------------------------------------------
def cmd_records(argv):
    want = argv[0] if argv else None
    for r in load():
        if not r["ok"] or (want and r["probe"] != want):
            continue
        if r["probe"] in GAP_SRC and not want:
            continue
        print("=== %s [%s] %s" % (r["probe"], r["mode"], " ".join(r["flags"])))
        for rec in eh_records(r):
            kind, fields = decode(rec)
            print("  %-34s %-16s seg=%s  %d data" %
                  (rec["name"], kind or "?", rec["seg"], len(rec["data"])))
            for f, v in fields:
                print("      %-22s %s" % (f, v))
        print()


def cmd_arity(argv):
    """A1b — THE CHECK WITH TEETH.

    Residue alone cannot see a SHORT read: if the parser loses data, the record
    simply has fewer fields and every one of them is still claimed, so the
    residue stays 0 and the run prints success. That is not hypothetical — c2
    run-length-encodes with `DD 2 DUP(00H)` and the first version of this parser
    read `__ehfuncinfo$` as 8 dwords instead of 9, residue 0, every field
    "claimed", and `pIPtoStateMap` decoded onto `nIPMapEntries`.

    So each record's LENGTH is predicted from a count field in a DIFFERENT
    record and checked. A short or long read fails this even when residue is 0.
    """
    ok = bad = 0
    fails = []
    for r in load():
        if not r["ok"] or r["probe"] in GAP_SRC:
            continue
        by_name = {rec["name"]: rec for rec in eh_records(r)}
        for nm, rec in sorted(by_name.items()):
            if not nm.startswith("__ehfuncinfo$"):
                continue
            fn = nm[len("__ehfuncinfo$"):]
            d = rec["data"]
            checks = [("FuncInfo dwords", len(d), 9)]
            maxState = n_of(d[1]) if len(d) > 1 else None
            nTry = n_of(d[3]) if len(d) > 3 else None
            nIP = n_of(d[5]) if len(d) > 5 else None
            uw = by_name.get("__unwindtable$" + fn)
            if uw is not None and maxState is not None:
                checks.append(("UnwindMap dwords", len(uw["data"]), 2 * maxState))
            tb = by_name.get("__tryblocktable$" + fn)
            if nTry:
                checks.append(("TryBlockMap dwords",
                               len(tb["data"]) if tb else 0, 5 * nTry))
            elif tb is not None:
                checks.append(("TryBlockMap present with nTryBlocks=0",
                               len(tb["data"]), 0))
            if len(d) > 6 and nIP is not None:
                tname = d[6].strip().rstrip(",")
                t = by_name.get(tname)
                checks.append(("IpToStateMap dwords",
                               len(t["data"]) if t else -1, 2 * nIP))
            # nCatches in each try-block entry vs its HandlerType array length
            if tb is not None and nTry:
                for i in range(nTry):
                    base = 5 * i
                    if base + 4 < len(tb["data"]):
                        nC = n_of(tb["data"][base + 3])
                        arr = by_name.get(tb["data"][base + 4].strip().rstrip(","))
                        checks.append(("HandlerType[%d] dwords" % i,
                                       len(arr["data"]) if arr else -1,
                                       4 * nC if nC is not None else -1))
            for what, got, want in checks:
                if got == want:
                    ok += 1
                else:
                    bad += 1
                    fails.append((r["probe"], r["mode"], fn, what, got, want))
    print("A1b ARITY — each record's length predicted from a count field elsewhere")
    print("  %d/%d consistent, %d inconsistent" % (ok, ok + bad, bad))
    for f in fails:
        print("  FAIL %-22s %-9s %-16s %-28s got %s want %s" % f)
    return bad


def cmd_totality(argv):
    """A1 — every datum claimed by a named field, residue PRINTED."""
    tot = {"fit": [0, 0], "held": [0, 0]}
    residue = []
    for r in load():
        if not r["ok"] or r["probe"] in GAP_SRC:
            continue
        grp = r["group"]
        for rec in eh_records(r):
            kind, fields = decode(rec)
            if kind is None:
                residue.append((r["probe"], r["mode"], rec["name"],
                                "NO DECODER", rec["data"]))
                tot[grp][1] += len(rec["data"])
                continue
            for f, v in fields:
                tot[grp][1] += 1
                if f.startswith("UNCLAIMED"):
                    residue.append((r["probe"], r["mode"], rec["name"], f, v))
                else:
                    tot[grp][0] += 1
    print("A1 TOTALITY — data claimed by a named field")
    for g in ("fit", "held"):
        c, t = tot[g]
        print("  %-6s %6d / %6d claimed   residue %d"
              % (g, c, t, t - c))
    print("\nRESIDUE, printed (never summarised):")
    if not residue:
        print("  (none)")
    for probe, mode, nm, f, v in residue:
        print("  %-24s %-9s %-34s %-14s %s" % (probe, mode, nm, f, v))


def _first_eh_fn_records(r):
    """The records belonging to the FIRST EH function in .text order."""
    out = {}
    for rec in eh_records(r):
        nm = rec["name"]
        for pre in ("__ehfuncinfo$", "__unwindtable$", "__tryblocktable$"):
            if nm.startswith(pre):
                out.setdefault(pre, rec)
    out["catchsyms"] = [rec for rec in eh_records(r)
                        if rec["name"].startswith("__catchsym$")]
    return out


def observed(r):
    """The A2 cells, read out of the .cod."""
    recs = _first_eh_fn_records(r)
    fi = recs.get("__ehfuncinfo$")
    if fi is None:
        return None
    d = fi["data"]
    o = dict(maxState=n_of(d[1]) if len(d) > 1 else None,
             nTryBlocks=n_of(d[3]) if len(d) > 3 else None,
             nIPMapEntries=n_of(d[5]) if len(d) > 5 else None,
             EHFlags=n_of(d[8]) if len(d) > 8 else None)
    # restrict to the first function's own catchsym arrays
    fn = fi["name"][len("__ehfuncinfo$"):]
    mine = [c for c in recs["catchsyms"]
            if c["name"][len("__catchsym$"):].startswith(fn + "$")]
    o["nCatchsym"] = len(mine)
    o["handlers"] = sorted(len(c["data"]) // 4 for c in mine)
    o["nFunclets"] = len([l for l in r["labels"] if l["kind"] == "funclet"
                          and l["fn"] == fn])
    return o


def cmd_predict(argv):
    """A2 — held-out cells, scored against PREDICT (registered pre-capture)."""
    cells = hits = declined = 0
    print("A2 — held-out structural counts, predicted from SOURCE before capture")
    print("     graded at EHsc-O1 (the workload profile)")
    print("  %-20s %-16s %8s %8s  %s" % ("probe", "cell", "pred", "obs", ""))
    for r in load():
        if not r["ok"] or r.get("group") != "held" or r["mode"] != "EHsc-O1":
            continue
        pred = PREDICT.get(r["probe"])
        obs = observed(r)
        if pred is None or obs is None:
            print("  %-20s  NO OBSERVATION" % r["probe"])
            continue
        for k, pv in pred.items():
            ov = obs.get(k)
            cells += 1
            if pv is None:
                declined += 1
                mark = "DECLINED"
            elif pv == ov:
                hits += 1
                mark = "hit"
            else:
                mark = "MISS"
            print("  %-20s %-16s %8s %8s  %s"
                  % (r["probe"], k, pv, ov, mark))
    print("\n  %d/%d exact = %.1f %%   (%d declined, scored 0)"
          % (hits, cells, 100.0 * hits / cells if cells else 0.0, declined))


def cmd_predict2(argv):
    """The corrected maxState law, graded ONLY on shapes it was not fitted on."""
    hits = cells = 0
    print("A2 round 2 — maxState = (dtors) + 2 x (try blocks), HELD OUT")
    for r in load():
        if not r["ok"] or r["probe"] not in PREDICT2 or r["mode"] != "EHsc-O1":
            continue
        obs = observed(r)
        for k, pv in sorted(PREDICT2[r["probe"]].items()):
            ov = obs.get(k) if obs else None
            cells += 1
            good = (pv == ov)
            hits += good
            print("  %-22s %-12s pred %-4s obs %-4s %s"
                  % (r["probe"], k, pv, ov, "hit" if good else "MISS"))
    print("  %d/%d = %.1f %% on held-out shapes"
          % (hits, cells, 100.0 * hits / cells if cells else 0.0))


def cmd_gaps(argv):
    rows = [r for r in load() if r["ok"] and r["probe"] in GAP_SRC]
    rows.sort(key=lambda r: r["probe"])
    print("#138 — the label-number gaps, PER FUNCTION, one axis moved per probe")
    print("  %-12s %-24s %5s %8s %8s   %s"
          % ("probe", "function", "nfn", "G_fn>M", "G_T>tri", "TU unnamed"))
    base = None
    for r in rows:
        g = r["gaps"]
        if not g:
            print("  %-12s (no labels)" % r["probe"])
            continue
        first = True
        for fn, e in sorted(g["per_fn"].items()):
            if "G_funclet_to_ipM" not in e:
                continue
            if r["probe"] == "g_base":
                base = e["G_funclet_to_ipM"]
            print("  %-12s %-24s %5d %8s %8s   %s"
                  % (r["probe"] if first else "", fn[:24], len(g["funcs"]),
                     e.get("G_funclet_to_ipM", "-"),
                     e.get("G_stateT_to_triple", "-"),
                     g["unnamed"] if first else ""))
            first = False
    print()
    print("  DELTA of G_funclet_to_ipM against g_base (G = %s), first EH fn:" % base)
    for r in rows:
        g = r["gaps"]
        if not g:
            continue
        cand = [e for _f, e in sorted(g["per_fn"].items())
                if "G_funclet_to_ipM" in e]
        if not cand:
            continue
        v = cand[0]["G_funclet_to_ipM"]
        print("  %-12s G=%-4d delta %+d" % (r["probe"], v, v - base))
    print()
    print("  the SEPARATE local-label space (`$LN`/`$LL`), NOT the TU counter:")
    for r in rows:
        g = r["gaps"]
        if g and g["locals"]:
            print("  %-12s %s" % (r["probe"], g["locals"]))


def cmd_gapmodel(argv):
    """#138 round 2 — the decomposition on combinations it was not fitted on."""
    hits = cells = 0
    print("#138 round 2 — G predicted from the decomposition, HELD OUT")
    print("  %-18s %6s %6s  %s" % ("probe", "pred", "obs", ""))
    for r in load():
        if not r["ok"] or r["probe"] not in Y_SRC:
            continue
        want = Y_SRC[r["probe"]][1]
        g = r["gaps"]
        cand = [e for _f, e in sorted((g or {}).get("per_fn", {}).items())
                if "G_funclet_to_ipM" in e]
        got = cand[0]["G_funclet_to_ipM"] if cand else None
        cells += 1
        good = (got == want)
        hits += good
        print("  %-18s %6s %6s  %s"
              % (r["probe"], want, got, "hit" if good else "MISS"))
    print("  %d/%d = %.1f %%" % (hits, cells, 100.0 * hits / cells if cells else 0))


def cmd_grade(argv):
    cmd_totality([])
    print()
    cmd_arity([])
    print()
    cmd_predict([])
    print()
    cmd_gaps([])


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        return
    cmd, argv = sys.argv[1], sys.argv[2:]
    if cmd == "gen":
        gen()
    elif cmd == "scan":
        j = 6
        if "--jobs" in argv:
            j = int(argv[argv.index("--jobs") + 1])
        scan(j)
    elif cmd == "records":
        cmd_records(argv)
    elif cmd == "totality":
        cmd_totality(argv)
    elif cmd == "arity":
        cmd_arity(argv)
    elif cmd == "predict":
        cmd_predict(argv)
    elif cmd == "predict2":
        cmd_predict2(argv)
    elif cmd == "gapmodel":
        cmd_gapmodel(argv)
    elif cmd == "gaps":
        cmd_gaps(argv)
    elif cmd == "grade":
        cmd_grade(argv)
    else:
        print(__doc__)


if __name__ == "__main__":
    main()
