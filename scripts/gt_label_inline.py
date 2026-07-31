#!/usr/bin/env python3
"""gt_label_inline.py — what does the label counter charge for an INLINED body?

`docs/LABEL_COUNTER.md` §4 recorded +3 / +8 / +13 slots for 1 / 2 / 3 inlined
sites of one static framed callee ("+5 per site after the first"), on three data
points, one callee class, with the first site's 3 unexplained — and it nearly
got recorded as quadratic because at three sites the body crossed into Class C
and the `__savegprlr_29` pair's +2 was hiding inside the delta.

This script settles it. It reuses `scripts/gt_label_stride.py`'s seed-free
instrument verbatim (three anchor framed functions around the probe, `base`
measured in-object by the a1→a2 control) and adds four things:

  1. a **family baseline**. Every family is swept N = 0,1,2,… sites of the SAME
     body, and the charge is `stride(N) - stride(0)` — a difference against a
     probe that differs only in the inlining, not against a generic framed
     function. §4's numbers were differences against a *generic* framed 5, which
     is where its unexplained constant came from.
  2. a **hand-inlined control** on every row: the identical body written out at
     the call site, with no callee function anywhere in the TU. `dhand` is the
     whole point — when it is non-zero while P's `.text` bytes are identical
     (`TEXT-IDENTICAL`), the charge is bookkeeping about the inline record and
     buys no code at all.
  3. the class crossing held **out** of the delta, in the open. `hcost` is 2 ×
     the `__save*`/`__rest*` widths P gained since its own N=0 row and `adj =
     charge - hcost`; the subtraction is printed, never applied in prose. Rows
     whose class moved are tagged `CLASS+`. The sites chain (`s = f(s);`) so
     only one value is live across each call and the class does not drift.
  4. a **self-falsifying fit**. `slope` is taken from the N=1 row alone and
     `pred = N*slope`; `resid` is what falsifies linearity. A family with any
     non-zero residual prints `NON-LINEAR` in its summary. Nothing is fitted to
     the whole sweep, so the sweep can refute it.

Usage:
    scripts/gt_label_inline.py [--mode '/O1 /GS- /c'] [--max N] [family ...]
    scripts/gt_label_inline.py --list

Env: C2RS_WIBO / C2RS_COMPILERS as for scripts/gt_capture.sh.
Exit status is 0 if every row's in-object control held; it says nothing about
whether a prediction held — read the table.
"""

import hashlib
import os
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from gt_label_stride import (  # noqa: E402
    ANCHORS, ANCHOR_DECL, capture, groups, minted, prologue_class,
)

FLOAT_LEAD = "float ldl(float a, float b){ return a*b; }"


def src_of(decls, leads, probe):
    parts = [ANCHOR_DECL]
    if decls:
        parts.append(decls)
    parts += leads
    parts.append(ANCHORS[0])
    parts.append(probe)
    parts.append(ANCHORS[1])
    parts.append(ANCHORS[2])
    return "\n".join(parts) + "\n"


# P must be FRAMED at N=0, or the family baseline is a leaf (stride 1) and every
# charge inherits a bogus +4. `gs(a)+a` keeps `a` live across the call, so N=0 is
# a plain Class-B framed function, stride 5.
INT_HEAD = "int P(int a){ int s=gs(a)+a;"
INT_TAIL = "return s; }"
GS = "int gs(int);"


class Family:
    """One body, emitted N times two ways: through a callee, and written out."""

    def __init__(self, name, decls, callee, site, hand,
                 head=INT_HEAD, tail=INT_TAIL, leads=(), note="",
                 always_lead=False):
        self.name, self.decls, self.callee = name, decls, callee
        self.site, self.hand = site, hand
        self.head, self.tail = head, tail
        self.leads, self.note = list(leads), note
        # `always_lead` keeps the callee's definition in the hand variant too —
        # needed when P references it for a reason other than calling it (its
        # address, say), or the control simply will not compile.
        self.always_lead = always_lead

    def source(self, n, variant):
        leads = list(self.leads)
        if variant == "inl" or self.always_lead:
            leads.append(self.callee)
        body = " ".join([self.site if variant == "inl" else self.hand] * n)
        probe = "%s %s %s" % (self.head, body, self.tail)
        return src_of(self.decls, leads, probe)


# `s=gs(s);` ahead of each site is an opaque barrier: it stops the optimizer
# folding chained sites together, and it is present in BOTH variants, so it
# cancels out of `dhand` and out of the slope.
BAR = "s=gs(s); "

FAMILIES = [
    # === the class §4 measured, with a baseline and a control ===============
    Family("framed", GS,
           "static int lst(int a){ return gs(a)+a; }",
           "s=lst(s);", "s=gs(s)+s;",
           note="static FRAMED callee, plain-variable argument (the §4 class)"),
    # === is the per-site charge the callee's own base leaking through? ======
    # A leaf's standalone stride is 1 and a framed function's is 5, so
    # CALLEE-BASE-LEAK predicts leaf < framed. Same call shape both rows.
    Family("leaf", GS,
           "static int llf(int a){ return a*3+1; }",
           BAR + "s=llf(s);", BAR + "s=s*3+1;",
           note="static LEAF callee, plain argument  <== CALLEE-BASE-LEAK test"),
    Family("leaf-big", GS,
           "static int llb(int a){ int t=a*3+1; t^=t>>3; t*=5; t^=t>>7;"
           " return t+a; }",
           BAR + "s=llb(s);",
           BAR + "{int t=s*3+1; t^=t>>3; t*=5; t^=t>>7; s=t+s;}",
           note="static leaf, a BIG body — 4 statements instead of 1"),
    Family("trivial", GS,
           "static int ltr(int a){ return a+1; }",
           BAR + "s=ltr(s);", BAR + "s=s+1;",
           note="the smallest possible callee body"),
    # === the ARGUMENT shape, against a fixed callee =========================
    Family("arg-plain", GS,
           "static int lap(int a){ return gs(a)+a; }",
           "s=lap(s);", "s=gs(s)+s;",
           note="argument is a plain local"),
    Family("arg-expr", GS,
           "static int lae(int a){ return gs(a)+a; }",
           "s=lae(s+1);", "{int t=s+1; s=gs(t)+t;}",
           note="argument is an expression (s+1)"),
    Family("arg-call", GS,
           "static int lac(int a){ return gs(a)+a; }",
           "s=lac(gs(s));", "{int t=gs(s); s=gs(t)+t;}",
           note="argument is itself a call"),
    Family("arg-const", GS,
           "static int lak(int a){ return gs(a)+a; }",
           BAR + "s+=lak(7);", BAR + "s+=gs(7)+7;",
           note="argument is a constant"),
    # === the PARAMETER count ================================================
    Family("param0", GS,
           "static int lp0(){ return gs(3)+4; }",
           BAR + "s+=lp0();", BAR + "s+=gs(3)+4;",
           note="callee takes NO parameters"),
    Family("param1", GS,
           "static int lp1(int a){ return gs(a)+a; }",
           "s=lp1(s);", "s=gs(s)+s;",
           note="callee takes ONE parameter"),
    Family("param2", GS,
           "static int lp2(int a,int b){ return gs(a)+b; }",
           "s=lp2(s,s);", "s=gs(s)+s;",
           note="callee takes TWO parameters, both the same plain local"),
    Family("param3", GS,
           "static int lp3(int a,int b,int c){ return gs(a)+b+c; }",
           "s=lp3(s,s,s);", "s=gs(s)+s+s;",
           note="callee takes THREE parameters"),
    # === the BODY shape =====================================================
    Family("body-local", GS,
           "static int lbl(int a){ int t=gs(a); return t+a; }",
           "s=lbl(s);", "{int t=gs(s); s=t+s;}",
           note="callee body declares a local"),
    Family("body-2call", "int gs(int); int gt(int);",
           "static int lb2(int a){ return gs(a)+gt(a)+a; }",
           "s=lb2(s);", "s=gs(s)+gt(s)+s;",
           note="callee body makes TWO calls"),
    Family("body-3call", "int gs(int); int gt(int); int gu(int);",
           "static int lb3(int a){ return gs(a)+gt(a)+gu(a)+a; }",
           "s=lb3(s);", "s=gs(s)+gt(s)+gu(s)+s;",
           note="callee body makes THREE calls"),
    Family("body-if", GS,
           "static int lbi(int a){ if (a > 0) return gs(a); return a+1; }",
           "s=lbi(s);", "if (s > 0) s=gs(s); else s=s+1;",
           note="callee body BRANCHES (two returns)"),
    Family("body-loop", GS,
           "static int lbo(int a){ int t=0; for(int i=0;i<a;i++) t+=gs(i);"
           " return t; }",
           "s=lbo(s);", "{int t=0; for(int i=0;i<s;i++) t+=gs(i); s=t;}",
           note="callee body contains a LOOP"),
    Family("body-void", GS,
           "static void lbv(int a){ gs(a); }",
           BAR + "lbv(s);", BAR + "gs(s);",
           note="callee returns void"),
    # === a callee that introduces a symbol P does not already have ==========
    Family("newsym", "int gz(int); int gs(int);",
           "static int lns(int a){ return gs(a)+a; }",
           "s=lns(s);", "s=gs(s)+s;",
           head="int P(int a){ int s=gz(a)+a;",
           note="P calls gz only; the inlined body introduces gs, new to P"),
    Family("samesym", GS,
           "static int lss(int a){ return gs(a)+a; }",
           "s=lss(s);", "s=gs(s)+s;",
           note="the inlined body's callee gs is one P already names"),
    # === FP and pooled constants ============================================
    Family("fp", "double gd(double);",
           "static double lfp(double a){ return gd(a)*a; }",
           "s=lfp(s);", "s=gd(s)*s;",
           head="double P(double a){ double s=gd(a)*a;", tail="return s; }",
           leads=[FLOAT_LEAD],
           note="FP-touching framed callee, _fltused charged to the lead"),
    Family("const", "float gf(float);",
           "static float lcf(float a){ return gf(a)*2.5f; }",
           "s=lcf(s);", "s=gf(s)*2.5f;",
           head="float P(float a){ float s=gf(a)*a;", tail="return s; }",
           leads=[FLOAT_LEAD],
           note="inlined body pools ONE new .rdata constant (2.5f) — +2 ONCE?"),
    # === separating "the static is in the TU" from "the static was inlined" ==
    Family("noinline", GS,
           "__declspec(noinline) static int lni(int a){ return gs(a)+a; }",
           "s=lni(s);", "s=gs(s)+s;",
           note="the callee is NOT inlined — real calls to an emitted static"),
    Family("extern-inline", GS,
           "inline int lei(int a){ return gs(a)+a; }",
           "s=lei(s);", "s=gs(s)+s;",
           note="an `inline` free function rather than a `static` one"),
    Family("addr-taken", "int gs(int); int gcall(int(*)(int));",
           "static int lat(int a){ return gs(a)+a; }",
           "s=lat(s);", "s=gs(s)+s;",
           head="int P(int a){ int s=gs(a)+gcall(lat)+a;",
           always_lead=True,
           note="the callee is inlined AND its address is taken"),
    # === nesting: an inline inside an inline ================================
    Family("nested", GS,
           "static int lin(int a){ return gs(a)+a; }\n"
           "static int lout(int a){ return lin(a)+lin(a+100); }",
           "s=lout(s);", "s=gs(s)+s+gs(s+100)+(s+100);",
           note="lout inlines lin TWICE, and P inlines lout N times"),

    # === ROUND 2: decomposing the per-site constant =========================
    # --- does the charge go BELOW 3? ---------------------------------------
    Family("ret-const", GS,
           "static int lrk(int a){ return 7; }",
           BAR + "s+=lrk(s);", BAR + "s+=7;",
           note="callee body is a constant — is there an inline that costs <3?"),
    Family("ret-param", GS,
           "static int lrp(int a){ return a; }",
           BAR + "s+=lrp(s);", BAR + "s+=s;",
           note="callee body is the identity"),
    # --- LOCALS, holding control flow and argument shape fixed --------------
    Family("loc0", GS,
           "static int lc0(int a){ return gs(a)+a; }",
           "s=lc0(s);", "s=gs(s)+s;",
           note="callee declares NO locals"),
    Family("loc1", GS,
           "static int lc1(int a){ int t=gs(a); return t+a; }",
           "s=lc1(s);", "{int t=gs(s); s=t+s;}",
           note="callee declares ONE local"),
    Family("loc2", GS,
           "static int lc2(int a){ int t=gs(a); int u=t+a; return u+1; }",
           "s=lc2(s);", "{int t=gs(s); int u=t+s; s=u+1;}",
           note="callee declares TWO locals"),
    Family("loc3", GS,
           "static int lc3(int a){ int t=gs(a); int u=t+a; int v=u*2;"
           " return v+1; }",
           "s=lc3(s);", "{int t=gs(s); int u=t+s; int v=u*2; s=v+1;}",
           note="callee declares THREE locals"),
    Family("loc1-dead", GS,
           "static int lcd(int a){ int t=5; return gs(a)+a; }",
           "s=lcd(s);", "{int t=5; s=gs(s)+s; (void)t;}",
           note="callee declares ONE local that generates NO code"),
    Family("loc1-block", GS,
           "static int lcb(int a){ { int t=gs(a); a=t+a; } return a; }",
           "s=lcb(s);", "{{int t=gs(s); s=t+s;}}",
           note="the local lives in a NESTED lexical block"),
    # --- CONTROL FLOW inside the callee -------------------------------------
    Family("cf-if", GS,
           "static int lf1(int a){ if (a > 0) return gs(a); return a+1; }",
           "s=lf1(s);", "if (s > 0) s=gs(s); else s=s+1;",
           note="ONE if with two returns"),
    Family("cf-if2", GS,
           "static int lf2(int a){ if (a > 0) return gs(a);"
           " if (a < -9) return a-1; return a+1; }",
           "s=lf2(s);",
           "if (s > 0) s=gs(s); else if (s < -9) s=s-1; else s=s+1;",
           note="TWO ifs"),
    Family("cf-tern", GS,
           "static int lf3(int a){ return a > 0 ? gs(a) : a+1; }",
           "s=lf3(s);", "s = s > 0 ? gs(s) : s+1;",
           note="a ternary rather than two returns"),
    Family("cf-void-if", GS,
           "static void lf4(int a){ if (a > 0) gs(a); }",
           BAR + "lf4(s);", BAR + "if (s > 0) gs(s);",
           note="void callee with one if — no result temp needed"),
    # --- LOOPS inside the callee --------------------------------------------
    Family("lp-for", GS,
           "static int lo1(int a){ int t=0; for(int i=0;i<a;i++) t+=gs(i);"
           " return t; }",
           "s=lo1(s);", "{int t=0; for(int i=0;i<s;i++) t+=gs(i); s=t;}",
           note="callee body is a for loop with a call"),
    Family("lp-for-leaf", GS,
           "static int lo2(int a){ int t=0; for(int i=0;i<a;i++) t+=i*3;"
           " return t; }",
           "s=lo2(s);", "{int t=0; for(int i=0;i<s;i++) t+=i*3; s=t;}",
           note="callee body is a for loop with NO call"),
    Family("lp-while", GS,
           "static int lo3(int a){ int t=0; while(a>0){ t+=gs(a); a--; }"
           " return t; }",
           "s=lo3(s);", "{int t=0; int b=s; while(b>0){ t+=gs(b); b--; } s=t;}",
           note="callee body is a while loop"),
    Family("lp-do", GS,
           "static int lo4(int a){ int t=0; do { t+=gs(a); a--; } while(a>0);"
           " return t; }",
           "s=lo4(s);",
           "{int t=0; int b=s; do { t+=gs(b); b--; } while(b>0); s=t;}",
           note="callee body is a do/while loop"),
    Family("lp-two", GS,
           "static int lo5(int a){ int t=0; for(int i=0;i<a;i++) t+=gs(i);"
           " for(int j=0;j<a;j++) t+=gs(j); return t; }",
           "s=lo5(s);",
           "{int t=0; for(int i=0;i<s;i++) t+=gs(i);"
           " for(int j=0;j<s;j++) t+=gs(j); s=t;}",
           note="callee body has TWO sequential loops"),
    # --- NESTING: the composition law ---------------------------------------
    Family("nest1", GS,
           "static int n1(int a){ return gs(a)+a; }",
           "s=n1(s);", "s=gs(s)+s;",
           note="depth 1 (the reference)"),
    Family("nest2", GS,
           "static int n1(int a){ return gs(a)+a; }\n"
           "static int n2(int a){ return n1(a)+1; }",
           "s=n2(s);", "s=gs(s)+s+1;",
           note="depth 2: n2 inlines n1 ONCE"),
    Family("nest3", GS,
           "static int n1(int a){ return gs(a)+a; }\n"
           "static int n2(int a){ return n1(a)+1; }\n"
           "static int n3(int a){ return n2(a)+2; }",
           "s=n3(s);", "s=gs(s)+s+3;",
           note="depth 3: n3 inlines n2 inlines n1"),
    Family("fan2", GS,
           "static int f1(int a){ return gs(a)+a; }\n"
           "static int f2(int a){ return f1(a)+f1(a+100); }",
           "s=f2(s);", "s=gs(s)+s+gs(s+100)+(s+100);",
           note="depth 2 with FANOUT 2: f2 inlines f1 twice"),
    # --- is the charge paid by a LEAF caller too? ---------------------------
    Family("leafP", GS,
           "static int lq(int a){ return a*3+1; }",
           "s=lq(s); s=s^1;", "s=s*3+1; s=s^1;",
           head="int P(int a){ int s=a;", tail="return s; }",
           note="P is a LEAF that inlines a leaf — does a leaf pay too?"),

    # === ROUND 3: the model above, tested on shapes it was NOT fitted to =====
    # The `PRED` in each note was written down BEFORE the capture (see the run
    # log in docs/LABEL_COUNTER.md §4.3). A row whose measurement differs from
    # its PRED is the refutation, printed rather than remembered.
    Family("loc4", GS,
           "static int lc4(int a){ int t=gs(a); int u=t+a; int v=u*2;"
           " int w=v^3; return w+1; }",
           "s=lc4(s);", "{int t=gs(s); int u=t+s; int v=u*2; int w=v^3;"
           " s=w+1;}",
           note="FOUR locals                                   PRED 7"),
    Family("loc5", GS,
           "static int lc5(int a){ int t=gs(a); int u=t+a; int v=u*2;"
           " int w=v^3; int x=w-1; return x+1; }",
           "s=lc5(s);", "{int t=gs(s); int u=t+s; int v=u*2; int w=v^3;"
           " int x=w-1; s=x+1;}",
           note="FIVE locals                                   PRED 8"),
    Family("blk1", GS,
           "static int lk1(int a){ { a = gs(a)+a; } return a; }",
           "s=lk1(s);", "{{ s = gs(s)+s; }}",
           note="ONE nested block, NO local                    PRED 4"),
    Family("blk2", GS,
           "static int lk2(int a){ { { a = gs(a)+a; } } return a; }",
           "s=lk2(s);", "{{{ s = gs(s)+s; }}}",
           note="TWO nested blocks, NO local                   PRED 5"),
    Family("hold-2loc-if", GS,
           "static int lh1(int a){ int t=gs(a); int u=t+a;"
           " if (u > 0) return u; return u+1; }",
           "s=lh1(s);", "{int t=gs(s); int u=t+s; if (u>0) s=u; else s=u+1;}",
           note="HOLD-OUT: 2 locals + 1 if                     PRED 6"),
    Family("hold-3loc-2if", GS,
           "static int lh2(int a){ int t=gs(a); int u=t+a; int v=u*2;"
           " if (v > 0) return v; if (v < -9) return v-1; return v+1; }",
           "s=lh2(s);",
           "{int t=gs(s); int u=t+s; int v=u*2;"
           " if (v>0) s=v; else if (v<-9) s=v-1; else s=v+1;}",
           note="HOLD-OUT: 3 locals + 2 ifs                    PRED 8"),
    Family("hold-loc-argexpr", GS,
           "static int lh3(int a){ int t=gs(a); return t+a; }",
           "s=lh3(s+1);", "{int q=s+1; int t=gs(q); s=t+q;}",
           note="HOLD-OUT: 1 local + expression argument       PRED 5"),
    Family("hold-dbl-loc", "int gs(int); double gd(double);",
           "static int lh4(int a){ double t=gd((double)a); return (int)t+a; }",
           "s=lh4(s);", "{double t=gd((double)s); s=(int)t+s;}",
           leads=[FLOAT_LEAD],
           note="HOLD-OUT: the local is a double, not an int   PRED 4"),
    Family("hold-2in1decl", GS,
           "static int lh5(int a){ int t=gs(a), u=a+1; return t+u; }",
           "s=lh5(s);", "{int t=gs(s), u=s+1; s=t+u;}",
           note="HOLD-OUT: TWO names in ONE declaration        PRED 5"),
    # === nesting depth, pushed far enough to separate the two laws ==========
    Family("nest4", GS,
           "static int m1(int a){ return gs(a)+a; }\n"
           "static int m2(int a){ return m1(a)+1; }\n"
           "static int m3(int a){ return m2(a)+2; }\n"
           "static int m4(int a){ return m3(a)+3; }",
           "s=m4(s);", "s=gs(s)+s+6;",
           note="depth 4   quadratic PRED 24 / additive PRED 13"),
    Family("nest5", GS,
           "static int m1(int a){ return gs(a)+a; }\n"
           "static int m2(int a){ return m1(a)+1; }\n"
           "static int m3(int a){ return m2(a)+2; }\n"
           "static int m4(int a){ return m3(a)+3; }\n"
           "static int m5(int a){ return m4(a)+4; }",
           "s=m5(s);", "s=gs(s)+s+10;",
           note="depth 5   quadratic PRED 35 / additive PRED 18"),
    Family("nest6", GS,
           "static int m1(int a){ return gs(a)+a; }\n"
           "static int m2(int a){ return m1(a)+1; }\n"
           "static int m3(int a){ return m2(a)+2; }\n"
           "static int m4(int a){ return m3(a)+3; }\n"
           "static int m5(int a){ return m4(a)+4; }\n"
           "static int m6(int a){ return m5(a)+5; }",
           "s=m6(s);", "s=gs(s)+s+15;",
           note="depth 6   quadratic PRED 48 / additive PRED 23"),
    # === fanout at depth 2 ==================================================
    Family("fan1", GS,
           "static int g1(int a){ return gs(a)+a; }\n"
           "static int q1(int a){ return g1(a)+1; }",
           "s=q1(s);", "s=gs(s)+s+1;",
           note="depth 2, fanout 1 (= nest2)                   PRED 8"),
    Family("fan3", GS,
           "static int g1(int a){ return gs(a)+a; }\n"
           "static int q3(int a){ return g1(a)+g1(a+100)+g1(a+200); }",
           "s=q3(s);",
           "s=gs(s)+s+gs(s+100)+(s+100)+gs(s+200)+(s+200);",
           note="depth 2, fanout 3                             PRED 22"),

    # === ROUND 4: is blk1's +1 the block, or the assignment to the param? ====
    Family("parammod", GS,
           "static int lpm(int a){ a = gs(a)+a; return a; }",
           "s=lpm(s);", "s=gs(s)+s;",
           note="ASSIGNS to its parameter, no block          PRED 4"),
    Family("blk-nomod", GS,
           "static int lbn(int a){ { gs(a); } return a+1; }",
           BAR + "s+=lbn(s);", BAR + "{ gs(s); } s+=s+1;",
           note="a block, does NOT assign to the parameter   PRED 3"),
    # === ROUND 4: do a callee's features scale with its nesting DEPTH? =======
    Family("d2-outer-loc", GS,
           "static int o1(int a){ return gs(a)+a; }\n"
           "static int o2(int a){ int t=o1(a); return t+1; }",
           "s=o2(s);", "{int t=gs(s)+s; s=t+1;}",
           note="depth 2, the OUTER callee has 1 local  PRED 9 (the control)"),
    Family("d2-inner-loc", GS,
           "static int i1(int a){ int t=gs(a); return t+a; }\n"
           "static int i2(int a){ return i1(a)+1; }",
           "s=i2(s);", "{int t=gs(s); s=t+s+1;}",
           note="depth 2, the INNER callee has 1 local  PRED 10 by L / 9 if flat"),
    Family("d2-inner-if", GS,
           "static int j1(int a){ if (a>0) return gs(a); return a+1; }\n"
           "static int j2(int a){ return j1(a)+1; }",
           "s=j2(s);", "{if (s>0) s=gs(s); else s=s+1; s=s+1;}",
           note="depth 2, the INNER callee has 1 if    PRED 10 by L / 9 if flat"),
    Family("d3-inner-loc", GS,
           "static int k1(int a){ int t=gs(a); return t+a; }\n"
           "static int k2(int a){ return k1(a)+1; }\n"
           "static int k3(int a){ return k2(a)+2; }",
           "s=k3(s);", "{int t=gs(s); s=t+s+3;}",
           note="depth 3, the INNERMOST has 1 local    PRED 18 by L / 16 if flat"),

    # === ROUND 5: d2-inner-if came in at 11 against law L's 10. The one
    #     structural difference from d2-inner-loc (which hit 10 exactly) is that
    #     the inner callee has TWO returns, so the inliner cannot substitute its
    #     result expression and must materialise a result variable. If that is
    #     it, the charge should follow the USE CONTEXT, not the depth. ========
    Family("ctx-stmt-1ret", GS,
           "static int c1r(int a){ return gs(a)+a; }",
           "s=c1r(s);", "s=gs(s)+s;",
           note="ONE return, result assigned straight to s   PRED 3 (control)"),
    Family("ctx-expr-1ret", GS,
           "static int c1e(int a){ return gs(a)+a; }",
           "s=c1e(s)+1;", "s=gs(s)+s+1;",
           note="ONE return, result used in an expression    PRED 3"),
    Family("ctx-stmt-2ret", GS,
           "static int c2r(int a){ if (a>0) return gs(a); return a+1; }",
           "s=c2r(s);", "if (s>0) s=gs(s); else s=s+1;",
           note="TWO returns, assigned straight to s         PRED 4 (control)"),
    Family("ctx-expr-2ret", GS,
           "static int c2e(int a){ if (a>0) return gs(a); return a+1; }",
           "s=c2e(s)+1;", "{if (s>0) s=gs(s); else s=s+1;} s=s+1;",
           note="TWO returns, result used in an expression   PRED 5"),
    Family("d2-inner-2ret-tail", GS,
           "static int t1(int a){ if (a>0) return gs(a); return a+1; }\n"
           "static int t2(int a){ return t1(a); }",
           "s=t2(s);", "if (s>0) s=gs(s); else s=s+1;",
           note="depth 2, inner has 2 returns, outer returns it DIRECTLY"
                "  PRED 10"),

    # === ROUND 6: WHERE is the multi-exit callee's result temp charged? ======
    # d2-inner-2ret-tail = 11 against law L's 10. The +1 is consistent with a
    # result variable charged at the CALLER's depth (1) rather than the callee's
    # (2). At depth 3 the two readings separate by a whole slot: caller-depth
    # gives 20, callee-depth 21, no-temp 18. A void inner callee needs no result
    # at all, so it should fall back to L exactly.
    Family("d3-inner-if", GS,
           "static int y1(int a){ if (a>0) return gs(a); return a+1; }\n"
           "static int y2(int a){ return y1(a)+1; }\n"
           "static int y3(int a){ return y2(a)+2; }",
           "s=y3(s);", "{if (s>0) s=gs(s); else s=s+1;} s=s+3;",
           note="depth 3, innermost has 2 returns"
                "   PRED 20 caller-depth / 21 callee-depth / 18 no temp"),
    Family("d2-inner-void-if", GS,
           "static void v1(int a){ if (a>0) gs(a); }\n"
           "static int v2(int a){ v1(a); return a+1; }",
           "s=v2(s);", "{if (s>0) gs(s);} s=s+1;",
           note="depth 2, inner is VOID with one if — no result to materialise"
                "  PRED 10"),

    # === ROUND 7: final hold-outs for the result-temp rule, plus the shape a
    #     real C++ workload actually inlines (a member function). ============
    Family("d3-mid-if", GS,
           "static int w1(int a){ return gs(a)+a; }\n"
           "static int w2(int a){ if (a>0) return w1(a); return a+1; }\n"
           "static int w3(int a){ return w2(a)+2; }",
           "s=w3(s);", "{if (s>0) s=gs(s)+s; else s=s+1;} s=s+2;",
           note="depth 3, the MIDDLE callee has 2 returns    PRED 18"),
    Family("d3-two-if", GS,
           "static int z1(int a){ if (a>9) return gs(a); return a+1; }\n"
           "static int z2(int a){ if (a>0) return z1(a); return a+1; }\n"
           "static int z3(int a){ return z2(a)+2; }",
           "s=z3(s);",
           "{if (s>0) { if (s>9) s=gs(s); else s=s+1; } else s=s+1;} s=s+2;",
           note="depth 3, TWO multi-exit callees             PRED 22"),
    Family("method", "int gs(int);",
           "struct S { int v; int m(int a){ return gs(a)+v; } };",
           "s=ob.m(s);", "s=gs(s)+ob.v;",
           head="int P(int a){ S ob; ob.v=a; int s=gs(a)+a;", always_lead=True,
           note="a C++ MEMBER function — the shape real TUs inline  PRED 3"),
    Family("method-loc", "int gs(int);",
           "struct T { int v; int m(int a){ int t=gs(a); return t+v; } };",
           "s=ob.m(s);", "{int t=gs(s); s=t+ob.v;}",
           head="int P(int a){ T ob; ob.v=a; int s=gs(a)+a;", always_lead=True,
           note="a member function with ONE local                    PRED 4"),

    # === ROUND 8: loops. Law L' does not cover them; these are the probes that
    #     decide whether a loop is one more depth-scaled E feature or its own
    #     flat charge. See docs/LABEL_COUNTER.md §6.6. =======================
    Family("lp-min", GS,
           "static int lm1(int a){ for(int i=0;i<a;i++) gs(i); return a; }",
           "s=lm1(s);", "{for(int i=0;i<s;i++) gs(i);}",
           note="a for loop, ONE local (the induction variable)   PRED 9"),
    Family("lp-min-outer", GS,
           "static int lm2(int a){ int i; for(i=0;i<a;i++) gs(i); return a; }",
           "s=lm2(s);", "{int i; for(i=0;i<s;i++) gs(i);}",
           note="same, induction variable declared OUTSIDE the for PRED 9"),
    Family("lp-inf", GS,
           "static int lm3(int a){ for(;;){ if (gs(a)) break; } return a; }",
           "s=lm3(s);", "for(;;){ if (gs(s)) break; }",
           note="for(;;) with a break — 0 locals, 1 if            PRED 9"),
    Family("lp-nested", GS,
           "static int lm4(int a){ int t=0; for(int i=0;i<a;i++)"
           " for(int j=0;j<a;j++) t+=gs(j); return t; }",
           "s=lm4(s);",
           "{int t=0; for(int i=0;i<s;i++) for(int j=0;j<s;j++) t+=gs(j);"
           " s=t;}",
           note="TWO nested for loops, 3 locals                   PRED 16"),
    Family("d2-lp-for", GS,
           "static int lm5(int a){ int t=0; for(int i=0;i<a;i++) t+=gs(i);"
           " return t; }\n"
           "static int lm6(int a){ return lm5(a)+1; }",
           "s=lm6(s);",
           "{int t=0; for(int i=0;i<s;i++) t+=gs(i); s=t+1;}",
           note="an lp-for body at DEPTH 2   PRED 22 scaled / 17 flat"),

    # === ROUND 9: the loop is neither flat nor depth-scaled. Solving the two
    #     depths gives `for` = 3*depth + 2; depth 3 is the hold-out. ==========
    Family("d3-lp-for", GS,
           "static int q5(int a){ int t=0; for(int i=0;i<a;i++) t+=gs(i);"
           " return t; }\n"
           "static int q6(int a){ return q5(a)+1; }\n"
           "static int q7(int a){ return q6(a)+2; }",
           "s=q7(s);",
           "{int t=0; for(int i=0;i<s;i++) t+=gs(i); s=t+3;}",
           note="an lp-for body at DEPTH 3         PRED 32 by for=3*depth+2"),
    Family("d2-lp-while", GS,
           "static int r5(int a){ int t=0; while(a>0){ t+=gs(a); a--; }"
           " return t; }\n"
           "static int r6(int a){ return r5(a)+1; }",
           "s=r6(s);",
           "{int t=0; int b=s; while(b>0){ t+=gs(b); b--; } s=t+1;}",
           note="a while body at DEPTH 2                     read, no PRED"),
    Family("d2-lp-do", GS,
           "static int u5(int a){ int t=0; do { t+=gs(a); a--; } while(a>0);"
           " return t; }\n"
           "static int u6(int a){ return u5(a)+1; }",
           "s=u6(s);",
           "{int t=0; int b=s; do { t+=gs(b); b--; } while(b>0); s=t+1;}",
           note="a do/while body at DEPTH 2                  read, no PRED"),

    Family("d3-lp-while", GS,
           "static int x5(int a){ int t=0; while(a>0){ t+=gs(a); a--; }"
           " return t; }\n"
           "static int x6(int a){ return x5(a)+1; }\n"
           "static int x7(int a){ return x6(a)+2; }",
           "s=x7(s);",
           "{int t=0; int b=s; while(b>0){ t+=gs(b); b--; } s=t+3;}",
           note="a while body at DEPTH 3       PRED 27 by while=depth+3"),
    Family("d3-lp-do", GS,
           "static int y5(int a){ int t=0; do { t+=gs(a); a--; } while(a>0);"
           " return t; }\n"
           "static int y6(int a){ return y5(a)+1; }\n"
           "static int y7(int a){ return y6(a)+2; }",
           "s=y7(s);",
           "{int t=0; int b=s; do { t+=gs(b); b--; } while(b>0); s=t+3;}",
           note="a do/while body at DEPTH 3    PRED 29 by do=2*depth+2"),

    # === ROUND 11: the C++ shapes a real workload TU is actually made of. Law
    #     L' was fitted entirely on int scalars, so these are all hold-outs. ==
    Family("struct-param", GS,
           "struct SP { int x, y; };\n"
           "static int lsp(SP v){ return gs(v.x)+v.y; }",
           "s=lsp(o);", "s=gs(o.x)+o.y;",
           head="int P(int a){ SP o; o.x=a; o.y=a+1; int s=gs(a)+a;",
           always_lead=True,
           note="callee takes a 2-int struct BY VALUE            PRED 4"),
    Family("struct-ref", GS,
           "struct SR { int x, y; };\n"
           "static int lsr(const SR& v){ return gs(v.x)+v.y; }",
           "s=lsr(o);", "s=gs(o.x)+o.y;",
           head="int P(int a){ SR o; o.x=a; o.y=a+1; int s=gs(a)+a;",
           always_lead=True,
           note="callee takes a const reference to it            PRED 3"),
    Family("struct-ret", GS,
           "struct ST { int x, y; };\n"
           "static ST lst2(int a){ ST r; r.x=gs(a); r.y=a; return r; }",
           "{ST q=lst2(s); s=q.x+q.y;}",
           "{ST q; q.x=gs(s); q.y=s; s=q.x+q.y;}",
           always_lead=True,
           note="callee RETURNS a 2-int struct by value          PRED 4"),
    Family("ref-param", GS,
           "static void lrf(int& o, int a){ o = gs(a)+a; }",
           "lrf(s, s);", "s = gs(s)+s;",
           note="callee takes int& and writes through it         PRED 3"),
    Family("ptr-param", GS,
           "static void lpt(int* o, int a){ *o = gs(a)+a; }",
           "lpt(&s, s);", "s = gs(s)+s;",
           note="callee takes int* and writes through it         PRED 3"),
    Family("switch-body", GS,
           "static int lsw(int a){ switch(a){ case 1: return gs(a);"
           " case 2: return a+2; case 7: return a+7; case 8: return a+8;"
           " default: return 0; } }",
           "s=lsw(s);",
           "switch(s){ case 1: s=gs(s); break; case 2: s=s+2; break;"
           " case 7: s=s+7; break; case 8: s=s+8; break; default: s=0; }",
           note="callee body is a 5-arm switch (§4: switch = +0)  PRED 3"),
    Family("ctor", GS,
           "struct CT { int v; CT(int a){ v = gs(a)+a; } };\n"
           "static int lct(int a){ CT c(a); return c.v; }",
           "s=lct(s);", "s=gs(s)+s;",
           always_lead=True,
           note="the callee constructs an object (ctor inlined)   PRED 4"),
    Family("dtor", GS,
           "struct DT { int v; DT(int a){ v = gs(a)+a; } ~DT(){ gs(v); } };\n"
           "static int ldt(int a){ DT d(a); return d.v; }",
           "s=ldt(s);", "{int v = gs(s)+s; gs(v); s=v;}",
           always_lead=True,
           note="…and it has a destructor too                     PRED 4"),

    # === ROUND 12: ref-param and ptr-param both came in at 4 against a PRED of
    #     3. The post-hoc reading is that binding a reference / taking an
    #     address materialises the argument, so it is an "argument that is not
    #     already a plain lvalue" and law L' already charges +1 for it. That is
    #     a reconciliation, not a prediction — so predict FROM it and check. ==
    Family("ptr-already", GS,
           "static void lpa(int* o, int a){ *o = gs(a)+a; }",
           "lpa(q, s);", "s = gs(s)+s;",
           head="int P(int a){ int s=gs(a)+a; int* q=&s;",
           note="the pointer argument is ALREADY a pointer variable   PRED 3"),
    Family("ptr-global", "int gs(int); extern int gv;",
           "static void lpg(int* o, int a){ *o = gs(a)+a; }",
           BAR + "lpg(&gv, s);", BAR + "gv = gs(s)+s;",
           note="the pointer argument is &<a global>                  PRED 4"),

    # === ROUND 13: the SWITCH, decomposed. `switch-body` is recorded in §6.7
    #     as "10 at N=1, 14 marginal — not even uniform in N". It is uniform:
    #     `dhand` is +10 per site flat to N=5 and the 4 that separates 14 from
    #     10 is what a SECOND written-out switch costs P, measured on the same
    #     row by the hand control. So the inline record for a 5-arm switch is
    #     10, and law L' at depth 1 needs E = 7 to reach it. Two readings of 7:
    #     one E unit per case ARM plus one for the switch construct plus the
    #     +1 flat multi-exit temp (5+1, +1 = 7), or a slope that is not 1. The
    #     arm ladder separates them: slope-1 predicts 7/8/9/(10)/11, slope-2
    #     predicts 4/6/8/(10)/12. Bias: I expect slope 1 and expect to be wrong
    #     about the intercept, because §6.6 already caught a construct (the
    #     loop) whose charge is affine in something other than its features.
    Family("sw-arms2", GS,
           "static int sw2(int a){ switch(a){ case 1: return gs(a);"
           " default: return 0; } }",
           "s=sw2(s);",
           "switch(s){ case 1: s=gs(s); break; default: s=0; }",
           note="a 2-arm switch              PRED 7 slope-1 / 4 slope-2"),
    Family("sw-arms3", GS,
           "static int sw3(int a){ switch(a){ case 1: return gs(a);"
           " case 2: return a+2; default: return 0; } }",
           "s=sw3(s);",
           "switch(s){ case 1: s=gs(s); break; case 2: s=s+2; break;"
           " default: s=0; }",
           note="a 3-arm switch              PRED 8 slope-1 / 6 slope-2"),
    Family("sw-arms4", GS,
           "static int sw4(int a){ switch(a){ case 1: return gs(a);"
           " case 2: return a+2; case 7: return a+7; default: return 0; } }",
           "s=sw4(s);",
           "switch(s){ case 1: s=gs(s); break; case 2: s=s+2; break;"
           " case 7: s=s+7; break; default: s=0; }",
           note="a 4-arm switch              PRED 9 slope-1 / 8 slope-2"),
    Family("sw-arms6", GS,
           "static int sw6(int a){ switch(a){ case 1: return gs(a);"
           " case 2: return a+2; case 7: return a+7; case 8: return a+8;"
           " case 9: return a+9; default: return 0; } }",
           "s=sw6(s);",
           "switch(s){ case 1: s=gs(s); break; case 2: s=s+2; break;"
           " case 7: s=s+7; break; case 8: s=s+8; break; case 9: s=s+9; break;"
           " default: s=0; }",
           note="a 6-arm switch              PRED 11 slope-1 / 12 slope-2"),
    Family("sw-dense", GS,
           "static int swd(int a){ switch(a){ case 0: return gs(a);"
           " case 1: return a+1; case 2: return a+2; case 3: return a+3;"
           " default: return 0; } }",
           "s=swd(s);",
           "switch(s){ case 0: s=gs(s); break; case 1: s=s+1; break;"
           " case 2: s=s+2; break; case 3: s=s+3; break; default: s=0; }",
           note="5 arms, CONTIGUOUS values (jump table)  PRED 10 (= sparse)"),
    Family("sw-void", GS,
           "static void swv(int a){ switch(a){ case 1: gs(a); break;"
           " case 2: gs(a+2); break; case 7: gs(a+7); break;"
           " case 8: gs(a+8); break; default: gs(0); } }",
           BAR + "swv(s);",
           BAR + "switch(s){ case 1: gs(s); break; case 2: gs(s+2); break;"
           " case 7: gs(s+7); break; case 8: gs(s+8); break;"
           " default: gs(0); }",
           note="the same 5 arms, VOID — no result temp   PRED 9 (10 less the"
                " flat multi-exit +1)"),
    Family("sw-1exit", GS,
           "static int sw1(int a){ int r; switch(a){ case 1: r=gs(a); break;"
           " case 2: r=a+2; break; case 7: r=a+7; break; case 8: r=a+8; break;"
           " default: r=0; } return r; }",
           "s=sw1(s);",
           "{int r; switch(s){ case 1: r=gs(s); break; case 2: r=s+2; break;"
           " case 7: r=s+7; break; case 8: r=s+8; break; default: r=0; }"
           " s=r;}",
           note="5 arms, ONE exit through a local  PRED 10 (10 +1 local -1"
                " multi-exit)"),
    Family("d2-switch", GS,
           "static int sq1(int a){ switch(a){ case 1: return gs(a);"
           " case 2: return a+2; case 7: return a+7; case 8: return a+8;"
           " default: return 0; } }\n"
           "static int sq2(int a){ return sq1(a)+1; }",
           "s=sq2(s);",
           "{switch(s){ case 1: s=gs(s); break; case 2: s=s+2; break;"
           " case 7: s=s+7; break; case 8: s=s+8; break; default: s=0; }"
           " s=s+1;}",
           note="the 5-arm switch at DEPTH 2  PRED 21 if the switch is an E"
                " feature (3 + [5+2*6] + 1); LOWER if it is affine like a loop"),

    # === ROUND 14: the CONSTRUCTOR is itself an inlined function, so `ctor`'s
    #     expansion tree is TWO instances deep and §6.7 graded it against a
    #     depth-1 prediction. Law L' on the tree the front end actually builds:
    #     lct at depth 1 with one declared local (`CT c`) = 3 + 1*1 = 4, plus
    #     CT::CT at depth 2 with E=0 = 2*2+1 = 5. Total 9 — which is what §6.7
    #     measured. `dtor` needs 16 and the same reading gives 4 + 5 + 5 = 14,
    #     so the destructor instance costs 7, i.e. E(~DT) = 1. These probes
    #     test the reading instead of asserting it; `ctor-direct` is the one
    #     that decides it, because it puts the constructor at depth 1 where the
    #     law's own depth term is directly readable.
    Family("ctor-direct", GS,
           "struct CD { int v; CD(int a){ v = gs(a)+a; } };",
           "{CD c(s); s=c.v;}", "{int cv = gs(s)+s; s=cv;}",
           note="P constructs the object ITSELF: ctor at DEPTH 1     PRED 3"),
    Family("ctor-noloc", GS,
           "struct CN { int v; CN(int a){ v = gs(a)+a; } };\n"
           "static int lcn(int a){ return CN(a).v; }",
           "s=lcn(s);", "s=gs(s)+s;",
           always_lead=True,
           note="the wrapper declares NO named local  PRED 8 (3 + 5); 9 if a"
                " temporary counts as a local"),
    Family("ctor-loc", GS,
           "struct CL { int v; CL(int a){ int t=gs(a); v=t+a; } };\n"
           "static int lcl(int a){ CL c(a); return c.v; }",
           "s=lcl(s);", "{int t=gs(s); s=t+s;}",
           always_lead=True,
           note="the CTOR BODY declares one local     PRED 11 (4 + [5+2*1])"),
    Family("ctor-if", GS,
           "struct CI { int v; CI(int a){ if (a>0) v=gs(a); else v=a+1; } };\n"
           "static int lci(int a){ CI c(a); return c.v; }",
           "s=lci(s);", "{int cv; if (s>0) cv=gs(s); else cv=s+1; s=cv;}",
           always_lead=True,
           note="the CTOR BODY has one if             PRED 11 (4 + [5+2*1])"),
    Family("ctor-init", GS,
           "struct CJ { int v; CJ(int a) : v(gs(a)+a) {} };\n"
           "static int lcj(int a){ CJ c(a); return c.v; }",
           "s=lcj(s);", "s=gs(s)+s;",
           always_lead=True,
           note="member-init list, not an assignment  PRED 9 (= ctor)"),
    Family("ctor-2mem", GS,
           "struct CM { int v, w; CM(int a){ v=gs(a); w=a+1; } };\n"
           "static int lcm(int a){ CM c(a); return c.v+c.w; }",
           "s=lcm(s);", "{int cv=gs(s); int cw=s+1; s=cv+cw;}",
           always_lead=True,
           note="the ctor assigns TWO members         PRED 9 (members are not"
                " locals)"),
    Family("dtor-direct", GS,
           "struct DD { int v; DD(int a){ v = gs(a)+a; } ~DD(){ gs(v); } };",
           "{DD d(s); s=d.v;}", "{int dv = gs(s)+s; s=dv; gs(dv);}",
           note="P declares the object ITSELF: ctor AND dtor at DEPTH 1"
                "   PRED 7 (3 + [3+1]) if E(~)=1, 6 if E(~)=0"),
    Family("dtor-only", GS,
           "struct DZ { int v; ~DZ(){ gs(v); } };\n"
           "static int ldz(int a){ DZ d; d.v=gs(a)+a; return d.v; }",
           "s=ldz(s);", "{int dv=gs(s)+s; s=dv; gs(dv);}",
           always_lead=True,
           note="a dtor and NO user ctor   PRED 11 (4 + [5+2*1]) if E(~)=1,"
                " 9 if E(~)=0"),
    Family("dtor-empty", GS,
           "struct DE { int v; DE(int a){ v = gs(a)+a; } ~DE(){} };\n"
           "static int lde(int a){ DE d(a); return d.v; }",
           "s=lde(s);", "s=gs(s)+s;",
           always_lead=True,
           note="the destructor body is EMPTY   PRED 16 (§6.1: the charge is"
                " about the expansion, not the code); 14 if the +2 was the"
                " body; 9 if a trivial dtor is not an instance at all"),
    Family("dtor-2obj", GS,
           "struct D2 { int v; D2(int a){ v = gs(a)+a; } ~D2(){ gs(v); } };\n"
           "static int ld2(int a){ D2 p(a); D2 q(a); return p.v+q.v; }",
           "s=ld2(s);",
           "{int pv=gs(s)+s; int qv=gs(s)+s; s=pv+qv; gs(qv); gs(pv);}",
           always_lead=True,
           note="TWO objects: 2 ctor + 2 dtor instances   PRED 29"
                " ([3+2] + 5 + 5 + 7 + 7)"),

    # === ROUND 15: §6.7 turned the `int&`/`int*` reconciliation into two
    #     predictions and both inverted; the only thing separating the 4s from
    #     the 3s was that the 4s point INTO A LOCAL OF P and the 3 points at a
    #     global. That is a story about storage class OR about lexical
    #     locality, and the two are separable. A function-static has a global's
    #     storage and a local's scope: storage-class predicts 3, locality
    #     predicts 4. `ref-2args` / `ptr-2args` separately decide whether the
    #     +1 is per ARGUMENT or once per callee.
    Family("ref-global", "int gs(int); extern int gv;",
           "static void lrg(int& o, int a){ o = gs(a)+a; }",
           BAR + "lrg(gv, s);", BAR + "gv = gs(s)+s;",
           note="int& bound to a GLOBAL       PRED 3 (mirrors ptr-global)"),
    Family("ref-const-read", GS,
           "static int lrr(const int& o, int a){ return gs(a)+o; }",
           "s=lrr(s, s);", "s=gs(s)+s;",
           note="const int& only READ, never written   PRED 4 if the +1 is the"
                " binding, 3 if it is the write-through"),
    Family("ptr-static-local", GS,
           "static void lps(int* o, int a){ *o = gs(a)+a; }",
           BAR + "lps(&sv, s); s+=sv;", BAR + "sv = gs(s)+s; s+=sv;",
           head="int P(int a){ static int sv; int s=gs(a)+a;",
           note="&<a function-static>: a global's storage, a local's scope"
                "   PRED 3 storage-class / 4 locality"),
    Family("ptr-2args", GS,
           "static void lq2(int* o, int* p, int a){ *o = gs(a); *p = a+1; }",
           "lq2(&s, &t, s);", "{int q=s; s=gs(q); t=q+1;}",
           head="int P(int a){ int t=a; int s=gs(a)+a;", tail="return s+t; }",
           note="TWO int* args, both &<local of P>   PRED 5 per-argument / 4"
                " once per callee"),
]

# Two-and-more DISTINCT callees, one site each — the per-site vs per-callee
# split. The sweep parameter is "how many distinct callees", each used once.
DISTINCT_CALLEES = [
    "static int ld%d(int a){ return gs(a+%d)+a; }" % (i, i + 1)
    for i in range(10)
]


def distinct_source(n, variant):
    leads = DISTINCT_CALLEES[:n] if variant == "inl" else []
    if variant == "inl":
        body = " ".join("s=ld%d(s);" % i for i in range(n))
    else:
        body = " ".join("s=gs(s+%d)+s;" % (i + 1) for i in range(n))
    return src_of(GS, leads, "%s %s %s" % (INT_HEAD, body, INT_TAIL))



# ---------------------------------------------------------------------------
# LAW L' — the model, written as data so the script refutes it rather than the
# reader remembering it. `law[f]` is the per-site charge predicted for family f:
#
#   For every inline instance I in the expansion tree (P's own call sites are
#   depth 1; a site inside a depth-d body is depth d+1):
#
#         cost(I) = (2*depth(I) + 1)  +  depth(I) * E(I)
#
#   where E(I) counts, in that callee's body: each declared local (any type, a
#   dead one counts, two names in one declaration count two), each `if`, each
#   parameter the body assigns to, and each argument at that call site that is
#   not already a plain lvalue. Plus **+1 flat, at any depth**, per multi-exit
#   callee whose result must be materialised — i.e. unless it is `void` or its
#   result is assigned straight to a variable at depth 1.
#
#   A LOOP is not an E feature: it does not scale with depth at rate 1, and the
#   three forms do not share a slope even though two of them agree at depth 1.
#   Each loop in I adds, OUTSIDE the d*E product,
#
#         for       3*d + 2        (5, 8, 11 at d = 1, 2, 3)
#         while       d + 3        (4, 5,  6)
#         do/while  2*d + 2        (4, 6,  8)
#
#   `while` and `do/while` both cost 4 at depth 1 and diverge at depth 2, which
#   is exactly the merge a depth-1-only capture set would have made.
#
#   On top of that P still pays its OWN §1.1 surcharges for the code it ends up
#   containing (`cf-tern`'s law of 7 is 5 of bookkeeping plus the 2 the hand
#   control independently measures for a materialised signed relational).
#
# `None` means "measured, deliberately NOT modelled" — the loops.
# ---------------------------------------------------------------------------
LAW = {
    "framed": 3, "leaf": 3, "leaf-big": 4, "trivial": 3,
    "arg-plain": 3, "arg-expr": 4, "arg-call": 4, "arg-const": 3,
    "param0": 3, "param1": 3, "param2": 3, "param3": 3,
    "body-local": 4, "body-2call": 3, "body-3call": 3, "body-if": 4,
    "body-void": 3, "body-loop": 10,
    "newsym": 3, "samesym": 3, "fp": 3, "const": 3,
    "noinline": 0, "extern-inline": 3, "addr-taken": 3, "nested": 15,
    "ret-const": 3, "ret-param": 3,
    "loc0": 3, "loc1": 4, "loc2": 5, "loc3": 6, "loc4": 7, "loc5": 8,
    "loc1-dead": 4, "loc1-block": 5, "blk1": 4, "blk2": 4,
    "blk-nomod": 3, "parammod": 4,
    "cf-if": 4, "cf-if2": 5, "cf-tern": 7, "cf-void-if": 4,
    "lp-for": 10, "lp-for-leaf": 10, "lp-while": 9, "lp-do": 9,
    "lp-min": 9, "lp-min-outer": 9, "lp-inf": 9, "lp-nested": 16,
    # lp-two is 16 at N=1 and then the inliner refuses, so its MARGINAL slope is
    # 0 and there is nothing for the law to be checked against. Left unmodelled
    # on purpose: the refusal, not the counter, is what that row measures.
    "lp-two": None,
    "d2-lp-for": 20, "d3-lp-for": 32,
    "d2-lp-while": 17, "d3-lp-while": 27,
    "d2-lp-do": 18, "d3-lp-do": 29,
    "hold-2loc-if": 6, "hold-3loc-2if": 8, "hold-loc-argexpr": 5,
    "hold-dbl-loc": 4, "hold-2in1decl": 5,
    "nest1": 3, "nest2": 8, "nest3": 15, "nest4": 24, "nest5": 35,
    "nest6": 48, "fan1": 8, "fan2": 15, "fan3": 22,
    "d2-outer-loc": 9, "d2-inner-loc": 10, "d2-inner-if": 11,
    "d3-inner-loc": 18, "d3-inner-if": 19, "d2-inner-void-if": 10,
    "d3-mid-if": 18, "d3-two-if": 22,
    "ctx-stmt-1ret": 3, "ctx-expr-1ret": 3, "ctx-stmt-2ret": 4,
    "ctx-expr-2ret": 5, "d2-inner-2ret-tail": 11,
    "method": 3, "method-loc": 4,
    "leafP": 3, "distinct": 3,
    # round 11 — filled in from the capture, see docs §6.8
    "struct-param": None, "struct-ref": None, "struct-ret": None,
    "ref-param": None, "ptr-param": None, "switch-body": None,
    "ctor": None, "dtor": None,
    "ptr-already": None, "ptr-global": None,
    # rounds 13-15 — hold-outs, graded by the run; see docs §6.9-§6.11
    "sw-arms2": None, "sw-arms3": None, "sw-arms4": None, "sw-arms6": None,
    "sw-dense": None, "sw-void": None, "sw-1exit": None, "d2-switch": None,
    "ctor-direct": None, "ctor-noloc": None, "ctor-loc": None,
    "ctor-if": None, "ctor-init": None, "ctor-2mem": None,
    "dtor-direct": None, "dtor-only": None, "dtor-empty": None,
    "dtor-2obj": None,
    "ref-global": None, "ref-const-read": None, "ptr-static-local": None,
    "ptr-2args": None,
}

HELPER_PFX = ("__savegprlr_", "__restgprlr_", "__savefpr_", "__restfpr_")


def measure(src, mode, workdir, tag):
    o = capture(src, mode, workdir, tag)
    if o is None:
        return {"error": "capture failed"}
    gs_ = groups(o)

    def find(sfx):
        for g in gs_:
            if g["name"].startswith("?" + sfx + "@@") or g["name"] == sfx:
                return g
        return None
    a0, a1, a2, P = find("a0"), find("a1"), find("a2"), find("P")
    if not (a0 and a1 and a2 and P):
        return {"error": "missing group %s" % [g["name"] for g in gs_]}
    f = lambda g: min(g["labels"]) if g["labels"] else None  # noqa: E731
    base = f(a2) - f(a1)
    known = {a0["name"], a1["name"], a2["name"], P["name"]}
    text = o.raw(o.sections[P["sec"] - 1])
    return {
        "base": base,
        "stride": f(a1) - f(a0) - base,
        # slots taken BEFORE P's own $M pair — §2.2 says every surcharge is
        # pre-allocated, so `extra == stride - base` is a second control.
        "extra": (f(P) - f(a0) - base) if f(P) is not None else None,
        "framed": f(P) is not None,
        "minted": minted(P),
        "prologue": prologue_class(o, P["sec"]),
        "helpers": sorted({s for s in P["syms"] if s.startswith(HELPER_PFX)}),
        "others": [g["name"] for g in gs_ if g["name"] not in known],
        "tsize": len(text),
        "thash": hashlib.sha1(text).hexdigest()[:8],
    }


HDR = ("    %-3s %-4s %6s %6s %5s %5s %5s %5s %6s %6s  %-15s %s"
       % ("N", "var", "stride", "charge", "hcost", "adj", "pred", "resid",
          "minted", "dtext", "text", "flags"))


def sweep(name, source_fn, note, mode, workdir, nmax):
    """Sweep N for both variants, N outer so the two can be cross-checked.

    Cross-checking at the same N is what catches an **inliner refusal**: at
    `/O1` the front end abandons inlining once the accumulated body gets big
    enough, and when it does, the charge silently stops growing. That looks
    exactly like the counter going non-linear. `dtext` (P's `.text` growth since
    N-1) makes it visible: it collapses on the row where the refusal happened,
    and the row is tagged `INLINE-REFUSED?`.
    """
    print("=== %-14s %s" % (name, note))
    print(HDR)
    rows, bad = {}, 0
    slope, adjs, refused = {}, {}, False
    for n in range(nmax + 1):
        both = {}
        for variant in ("inl", "hand"):
            r = measure(source_fn(n, variant), mode, workdir,
                        "%s_%s_%d" % (name.replace("-", "_"), variant, n))
            if "error" not in r:
                both[variant] = r
        for variant in ("inl", "hand"):
            r = both.get(variant)
            if r is None:
                print("    %-3d %-4s  capture failed" % (n, variant))
                bad += 1
                continue
            if r["base"] not in (4, 5):
                bad += 1
            rows[(variant, n)] = r
            b = rows.get((variant, 0))
            charge = r["stride"] - b["stride"]
            newh = [h for h in r["helpers"] if h not in b["helpers"]]
            hcost = 2 * len({h.rsplit("_", 1)[-1] for h in newh})
            adj = charge - hcost
            adjs.setdefault(variant, {})[n] = adj
            if n == 1:
                slope[variant] = adj
            k = slope.get(variant)
            pred = 0 if k is None else k * n
            prev = rows.get((variant, n - 1))
            dtext = r["tsize"] - prev["tsize"] if prev else 0
            flags = []
            if newh:
                flags.append("CLASS+" + ",".join(newh))
            if not r["framed"]:
                flags.append("LEAF-P")
            elif r["extra"] != r["stride"] - r["base"]:
                flags.append("PREALLOC-BROKEN(extra=%s)" % r["extra"])
            # An inline that DID happen grows P by roughly what writing the body
            # out grows it by. Much less than that means the front end declined
            # one — possibly only an INNER level, which leaves the charge linear
            # and non-zero and is therefore invisible in every other column. At
            # `/Ox` it declines the inner level of every `while`/`do` loop body,
            # and without this check that shows up as the law being refuted.
            if variant == "inl" and n >= 1 and "hand" in both:
                hprev = rows.get(("hand", n - 1))
                hd = both["hand"]["tsize"] - hprev["tsize"] if hprev else None
                if dtext <= 0 or (hd and hd > 0 and dtext * 2 < hd):
                    flags.append("INLINE-DECLINED?(dtext %d vs hand %s)"
                                 % (dtext, hd))
                    refused = True
            other = rows.get(("hand" if variant == "inl" else "inl", n))
            if other:
                d = r["stride"] - other["stride"]
                flags.append("dhand=%+d" % (d if variant == "inl" else -d))
                flags.append("TEXT-IDENTICAL" if other["thash"] == r["thash"]
                             else "text-differs")
            if r["others"]:
                flags.append("EMIT:" + ",".join(x[:20] for x in r["others"]))
            print("    %-3d %-4s %6d %6d %5d %5d %5d %5d %6d %6d  %-15s %s"
                  % (n, variant, r["stride"], charge, hcost, adj, pred,
                     adj - pred, r["minted"], dtext,
                     "%d/%s" % (r["tsize"], r["thash"]), " ".join(flags)))
    ki, kh = slope.get("inl"), slope.get("hand")
    lin = (ki is not None
           and all(v == ki * k for k, v in adjs.get("inl", {}).items()))
    # The N=1 slope absorbs any ONE-OFF charge P pays for merely containing the
    # construct at all (at /Ox, a branch costs P +1 once — the hand control
    # shows the same +1 and then goes flat). The incremental slope between the
    # last two rows has that one-off differenced out, so it, not the N=1 slope,
    # is what LAW L' is checked against.
    ai = adjs.get("inl", {})
    kinc = (ai[nmax] - ai[nmax - 1]) if nmax >= 2 and nmax in ai else ki
    ah = adjs.get("hand", {})
    khinc = (ah[nmax] - ah[nmax - 1]) if nmax >= 2 and nmax in ah else kh
    oneoff = None if ki is None or kinc is None else ki - kinc
    # THE BOOKKEEPING SLOPE. `kinc` is everything the inlined row pays: the
    # inline record PLUS whatever §1.1 surcharge P owes for the code it now
    # contains. The hand control pays only the second, so `kinc - khinc` is the
    # inline record alone. On 72 of the 100 families khinc is 0 and the two are
    # the same number; where it is not, reading `kinc` as "the cost of inlining"
    # is a category error. `switch-body` is the case that matters: its marginal
    # 14 is 10 of bookkeeping plus 4 that a SECOND written-out switch costs P
    # whether or not anything was inlined, and the 10 is flat from N=1 to N=5
    # while the 14 is not. Printed on every row so no family can hide it.
    book = None if kinc is None or khinc is None else kinc - khinc
    want = LAW.get(name, "?")
    # A LAW entry is a prediction about ONE expansion tree — the one the front
    # end builds for that source. When the front end declines an inline the tree
    # is a different tree, and the law is not being asked the question it
    # answers. Saying so is not an excuse: `INLINE-DECLINED?` is computed from
    # P's own `.text` growth against the hand control, printed on the offending
    # row, and it is what turns 10 apparent /Ox refutations into 10 rows where
    # `while`/`do` loop bodies simply were not inlined at the inner level.
    if want is None:
        verdict = "law: NOT MODELLED"
    elif want == "?":
        verdict = "law: no entry"
    elif want == kinc:
        verdict = "law %d OK" % want
    elif refused:
        verdict = "law %d n/a — the front end declined an inline, so this is a" \
                  " DIFFERENT expansion tree" % want
    else:
        verdict = "law %d  <== *** REFUTES LAW L' ***" % want
    shape = ("LINEAR to N=%d" % nmax if lin
             else "one-off %+d at N=1, linear after" % oneoff if oneoff
             else "*** NON-LINEAR ***")
    print("    -> %s/site marginal (%s at N=1), %s;  hand control %s/%s;"
          "  bookkeeping %s/site;  %s%s"
          % (kinc, ki, shape, kh, khinc, book, verdict,
             "  (see INLINE-DECLINED? rows)" if refused else ""))
    print()
    return bad, (want not in (None, "?") and want != kinc and not refused)


def main(argv):
    fams = {f.name: (f.source, f.note) for f in FAMILIES}
    fams["distinct"] = (distinct_source,
                        "N DISTINCT callees, ONE site each "
                        "<== per-site vs per-callee")
    if "--list" in argv:
        for k, (_, note) in fams.items():
            print("%-16s %s" % (k, note))
        return 0
    mode, nmax = "/O1 /GS- /c", 6
    if "--mode" in argv:
        i = argv.index("--mode"); mode = argv[i + 1]; del argv[i:i + 2]
    if "--max" in argv:
        i = argv.index("--max"); nmax = int(argv[i + 1]); del argv[i:i + 2]
    want = [a for a in argv[1:] if not a.startswith("--")]
    sel = [k for k in fams if not want or k in want]

    print("mode: %s   N = inlined call sites of the SAME body" % mode)
    print("  charge = stride(N) - stride(0) OF THE SAME FAMILY (not a generic framed 5)")
    print("  hcost  = 2 x helper widths P gained since its own N=0; adj = charge - hcost")
    print("           (a class crossing is SUBTRACTED IN THE OPEN, never in prose)")
    print("  slope  = adj at N=1 ALONE; pred = slope*N; resid = adj - pred FALSIFIES it")
    print("  law    = LAW L\'s prediction for this family, written down as data —")
    print("           a family that disagrees prints *** REFUTES LAW L\' ***")
    print("  dhand  = stride(inlined) - stride(hand-written-out); TEXT-IDENTICAL means")
    print("           P's .text bytes are equal in the two objs, so dhand bought no code")
    print("  book   = marginal(inl) - marginal(hand): the INLINE RECORD alone, with P's")
    print("           own §1.1 surcharge for the code differenced out. Read this, not")
    print("           the marginal, when asking what an inlined site costs.")
    print()
    wd = tempfile.mkdtemp(prefix="gtinl")
    bad = refuted = 0
    for k in sel:
        src_fn, note = fams[k]
        b, r = sweep(k, src_fn, note, mode, wd, nmax)
        bad += b
        refuted += bool(r)
    print("controls failed: %d   families refuting LAW L': %d" % (bad, refuted))
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
