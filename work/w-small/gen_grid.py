#!/usr/bin/env python3
"""w-small Lead 2 probe grid: short-circuit `&&` in a guarded early return.

POSITIVE cells are expected `Port=Match`; NEGATIVE cells are expected
`Port=NotImplemented`.  Structural axes are CROSSED, not varied one at a time
(board #198 / w-frame §4.5.3: a family exhaustive on the axis it varies and
blind on the one it holds fixed reads as complete).
"""
import os, sys

HDR = "void v0();\nvoid v1();\nint gi(int);\n"
POS, NEG = [], []

def p(name, body): POS.append((name, body))
def n(name, body): NEG.append((name, body))

# ---- axis: conjunct COUNT (2,3,4) x arm KIND (int, void) -------------------
p("c2_int",  "int P(int a,int b){ if (a!=0 && b!=0) return 5; v0(); return 0; }")
p("c3_int",  "int P(int a,int b,int c){ if (a!=0 && b!=0 && c!=0) return 5; v0(); return 0; }")
p("c4_int",  "int P(int a,int b,int c,int d){ if (a!=0 && b!=0 && c!=0 && d!=0) return 5; v0(); return 0; }")
n("c2_void", "void P(int a,int b){ if (a!=0 && b!=0) return; v0(); v1(); }")
n("c3_void", "void P(int a,int b,int c){ if (a!=0 && b!=0 && c!=0) return; v0(); v1(); }")

# ---- axis: RELATION, crossed with signedness ------------------------------
RELS = [("ne","a!=0 && b!=0"), ("eq","a==0 && b==0"), ("lt","a<3 && b<7"),
        ("gt","a>3 && b>7"), ("le","a<=3 && b<=7"), ("ge","a>=3 && b>=7"),
        ("mix1","a<0 && b!=0"), ("mix2","a==0 && b>=11")]
for k, c in RELS:
    p("rel_%s" % k, "int P(int a,int b){ if (%s) return 5; v0(); return 0; }" % c)
    n("relv_%s" % k, "void P(int a,int b){ if (%s) return; v0(); v1(); }" % c)
URELS = [("ult","a<3u && b<7u"), ("uge","a>=3u && b>=7u"), ("une","a!=0u && b!=0u")]
for k, c in URELS:
    p("urel_%s" % k, "int P(unsigned a,unsigned b){ if (%s) return 5; v0(); return 0; }" % c)

# ---- axis: SCRUTINEE POSITION (formals 0..3) ------------------------------
p("pos_23", "int P(int a,int b,int c,int d){ if (c!=0 && d!=0) return 5; v0(); return 0; }")
p("pos_03", "int P(int a,int b,int c,int d){ if (a!=0 && d!=0) return 5; v0(); return 0; }")
p("pos_30", "int P(int a,int b,int c,int d){ if (d!=0 && a!=0) return 5; v0(); return 0; }")
p("pos_same", "int P(int a,int b){ if (a!=0 && a!=3) return 5; v0(); return 0; }")

# ---- axis: LITERAL MAGNITUDE ---------------------------------------------
p("lit_big", "int P(int a,int b){ if (a<4660 && b>-1) return 32767; v0(); return 0; }")
p("lit_neg", "int P(int a,int b){ if (a<-3 && b>-100) return -1; v0(); return 0; }")

# ---- axis: POINTER operands (an unsigned compare) -------------------------
p("ptr", "int P(void*p,void*q){ if (p==0 && q==0) return 5; v0(); return 0; }")
n("ptrv", "void P(void*p,void*q){ if (p==0 && q==0) return; v0(); v1(); }")

# ---- axis: GUARD COUNT / COMPOSITION with plain guards --------------------
p("mix_and_then_plain", "int P(int a,int b,int c){ if (a!=0 && b!=0) return 5; if (c!=0) return 11; v0(); return 0; }")
p("mix_plain_then_and", "int P(int a,int b,int c){ if (a!=0) return 5; if (b!=0 && c!=0) return 11; v0(); return 0; }")
p("mix_and_and",        "int P(int a,int b,int c,int d){ if (a!=0 && b!=0) return 5; if (c!=0 && d!=0) return 11; v0(); return 0; }")
n("mix_void2",          "void P(int a,int b,int c){ if (a!=0 && b!=0) return; if (c!=0) return; v0(); v1(); }")

# ---- axis: TRAILING-CALL COUNT -------------------------------------------
p("tail2", "int P(int a,int b){ if (a!=0 && b!=0) return 5; v0(); v1(); return 0; }")
p("tail_val", "int P(int a,int b){ if (a!=0 && b!=0) return 5; v0(); return 11; }")

# ==== NEGATIVE: every one of these must stay NotImplemented ================
n("or2",       "int P(int a,int b){ if (a!=0 || b!=0) return 5; v0(); return 0; }")
n("or_void",   "void P(int a,int b){ if (a!=0 || b!=0) return; v0(); v1(); }")
n("or3",       "int P(int a,int b,int c){ if (a!=0 || b!=0 || c!=0) return 5; v0(); return 0; }")
n("and_or",    "int P(int a,int b,int c){ if (a!=0 && b!=0 || c!=0) return 5; v0(); return 0; }")
n("or_and",    "int P(int a,int b,int c){ if ((a!=0 || b!=0) && c!=0) return 5; v0(); return 0; }")
# same exit value in two arms -> c2 MERGES and branches BACKWARD (w-conv probe2::m2)
n("samevalue", "int P(int a,int b,int c){ if (a!=0 && b!=0) return 5; if (c!=0) return 5; v0(); return 0; }")
n("samevalue_tail", "int P(int a,int b){ if (a!=0 && b!=0) return 0; v0(); return 0; }")
# an arm containing a call, and a guard after a call
n("arm_call",  "int P(int a,int b){ if (a!=0 && b!=0) { v1(); return 5; } v0(); return 0; }")
n("after_call","int P(int a,int b){ v0(); if (a!=0 && b!=0) return 5; return 0; }")
# a conjunct that is not a formal compare
n("nonformal", "int P(int a,int b){ int t=a+b; if (t!=0 && b!=0) return 5; v0(); return 0; }")
n("call_cond", "int P(int a,int b){ if (gi(a)!=0 && b!=0) return 5; v0(); return 0; }")
# a guarded CALL (W10's production) plus an && early return: two block plans
n("compose",   "int P(int a,int b,int c){ if (a!=0 && b!=0) return 5; if (c!=0) v0(); v1(); return 0; }")
# else arm
n("else_arm",  "int P(int a,int b){ if (a!=0 && b!=0) return 5; else return 11; }")

out = sys.argv[1]
os.makedirs(out, exist_ok=True)
man = []
for kind, cells in (("POS", POS), ("NEG", NEG)):
    for name, body in cells:
        fn = os.path.join(out, "%s_%s.cpp" % (kind.lower(), name))
        open(fn, "w").write(HDR + body + "\n")
        man.append("%s %s" % (kind, fn))
open(os.path.join(out, "manifest.txt"), "w").write("\n".join(man) + "\n")
print("POS=%d NEG=%d total=%d" % (len(POS), len(NEG), len(POS) + len(NEG)))
