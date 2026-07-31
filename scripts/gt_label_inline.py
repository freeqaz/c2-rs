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
    "body-void": 3, "body-loop": None,
    "newsym": 3, "samesym": 3, "fp": 3, "const": 3,
    "noinline": 0, "extern-inline": 3, "addr-taken": 3, "nested": 15,
    "ret-const": 3, "ret-param": 3,
    "loc0": 3, "loc1": 4, "loc2": 5, "loc3": 6, "loc4": 7, "loc5": 8,
    "loc1-dead": 4, "loc1-block": 5, "blk1": 4, "blk2": 4,
    "blk-nomod": 3, "parammod": 4,
    "cf-if": 4, "cf-if2": 5, "cf-tern": 7, "cf-void-if": 4,
    "lp-for": None, "lp-for-leaf": None, "lp-while": None, "lp-do": None,
    "lp-two": None,
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
        for variant in ("inl", "hand"):
            r = measure(source_fn(n, variant), mode, workdir,
                        "%s_%s_%d" % (name.replace("-", "_"), variant, n))
            if "error" in r:
                print("    %-3d %-4s  %s" % (n, variant, r["error"]))
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
            if variant == "inl" and n > 1 and dtext <= 0:
                flags.append("INLINE-REFUSED?")
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
    want = LAW.get(name, "?")
    if want is None:
        verdict = "law: NOT MODELLED"
    elif want == "?":
        verdict = "law: no entry"
    elif want == kinc:
        verdict = "law %d OK" % want
    else:
        verdict = "law %d  <== *** REFUTES LAW L' ***" % want
    shape = ("LINEAR to N=%d" % nmax if lin
             else "one-off %+d at N=1, linear after" % oneoff if oneoff
             else "*** NON-LINEAR ***")
    print("    -> %s/site marginal (%s at N=1), %s;  hand control %s/%s;  %s%s"
          % (kinc, ki, shape, kh, khinc, verdict,
             "  (inliner refused — see dtext)" if refused else ""))
    print()
    return bad, (want not in (None, "?") and want != kinc)


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
