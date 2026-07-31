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

    # === ROUND 16: hold-outs for the three rules rounds 13-15 arrived at.
    #
    #  (a) SWITCH. The arm ladder came in at 7/8/9/10/11 for 2..6 arms, so the
    #      switch is an ordinary depth-scaled E feature with E = arms + 2 — not
    #      an affine term of its own like the loop. What is the +2? These rows
    #      ask whether the `default` arm is counted because it is WRITTEN or
    #      because the front end always has one, and whether a shared case
    #      label counts once or twice.
    #  (b) IF/ELSE. `ctor-if` overshot by exactly 2 = 2 * (depth 2), which is
    #      one extra E unit in a body whose only difference from `cf-if` is an
    #      explicit `else`. Test it at depth 1 where the scaling cannot hide.
    #  (c) DESTRUCTORS. `dtor-direct` = 6 says a destructor is an ordinary
    #      depth-1 instance with E = 0, same as a constructor — so the 16 is
    #      not in the destructor. A single rule fits all four measured rows:
    #      **a function that owns any destructible local pays E += 2, once,
    #      not per object**, and P pays nothing because P is not an inline
    #      instance. dtor 4+2 ->6, +5 ctor, +5 dtor = 16; dtor-only 3+(1+2)=6,
    #      +5 = 11; dtor-2obj 3+(2+2)=7, +10, +10 = 27; dtor-empty = 16.
    #      Three rows now hold it out.
    Family("sw-ctx-expr", GS,
           "static int swe(int a){ switch(a){ case 1: return gs(a);"
           " case 2: return a+2; case 7: return a+7; case 8: return a+8;"
           " default: return 0; } }",
           "s=swe(s)+1;",
           "{switch(s){ case 1: s=gs(s); break; case 2: s=s+2; break;"
           " case 7: s=s+7; break; case 8: s=s+8; break; default: s=0; }"
           " s=s+1;}",
           note="5 arms, result used in an EXPRESSION at depth 1  PRED 11"
                " (10 + the flat multi-exit +1)"),
    Family("sw-nodefault", GS,
           "static int swn(int a){ int r=0; switch(a){ case 1: r=gs(a); break;"
           " case 2: r=a+2; break; case 7: r=a+7; break; } return r; }",
           "s=swn(s);",
           "{int r=0; switch(s){ case 1: r=gs(s); break; case 2: r=s+2; break;"
           " case 7: r=s+7; break; } s=r;}",
           note="3 arms and NO default, 1 local, 1 exit   PRED 9 (3 + [3+2+1])"
                " if `default` counts only when written"),
    Family("sw-withdefault", GS,
           "static int swy(int a){ int r=0; switch(a){ case 1: r=gs(a); break;"
           " case 2: r=a+2; break; case 7: r=a+7; break; default: r=0; }"
           " return r; }",
           "s=swy(s);",
           "{int r=0; switch(s){ case 1: r=gs(s); break; case 2: r=s+2; break;"
           " case 7: r=s+7; break; default: r=0; } s=r;}",
           note="the SAME body plus a written default      PRED 10 — the pair"
                " with sw-nodefault is the discriminator, not either alone"),
    Family("sw-fall", GS,
           "static int swf(int a){ switch(a){ case 1: case 2: return gs(a);"
           " case 7: return a+7; case 8: return a+8; default: return 0; } }",
           "s=swf(s);",
           "switch(s){ case 1: case 2: s=gs(s); break; case 7: s=s+7; break;"
           " case 8: s=s+8; break; default: s=0; }",
           note="5 case LABELS sharing 4 statement groups  PRED 10 if labels"
                " are what is counted / 9 if groups are"),
    Family("d2-sw2", GS,
           "static int sr1(int a){ switch(a){ case 1: return gs(a);"
           " default: return 0; } }\n"
           "static int sr2(int a){ return sr1(a)+1; }",
           "s=sr2(s);",
           "{switch(s){ case 1: s=gs(s); break; default: s=0; } s=s+1;}",
           note="a 2-arm switch at DEPTH 2   PRED 17 (3 + [5+2*4] + 1), which"
                " is d2-switch's 23 less 2*(6-4) arms"),
    Family("cf-else", GS,
           "static int lfe2(int a){ if (a > 0) return gs(a); else return a+1; }",
           "s=lfe2(s);", "if (s > 0) s=gs(s); else s=s+1;",
           note="an explicit ELSE, two returns   PRED 5 if `else` is its own E"
                " unit / 4 if not (cf-if, the same code without `else`, is 4)"),
    Family("cf-else-assign", GS,
           "static int lfg(int a){ int r; if (a > 0) r=gs(a); else r=a+1;"
           " return r; }",
           "s=lfg(s);", "{int r; if (s>0) r=gs(s); else r=s+1; s=r;}",
           note="if/else assigning a local, ONE exit   PRED 6 (1 local + if +"
                " else) / 5 if `else` does not count"),
    Family("dtor-direct-only", GS,
           "struct DP { int v; ~DP(){ gs(v); } };",
           "{DP d; d.v=gs(s)+s; s=d.v;}", "{int dv=gs(s)+s; s=dv; gs(dv);}",
           note="P owns a dtor-only object — dtor at DEPTH 1 and P, not being"
                " an inline instance, pays no scope-exit E   PRED 3"),
    Family("dtor-3obj", GS,
           "struct D3 { int v; D3(int a){ v = gs(a)+a; } ~D3(){ gs(v); } };\n"
           "static int ld3(int a){ D3 p(a); D3 q(a); D3 r(a);"
           " return p.v+q.v+r.v; }",
           "s=ld3(s);",
           "{int pv=gs(s)+s; int qv=gs(s)+s; int rv=gs(s)+s; s=pv+qv+rv;"
           " gs(rv); gs(qv); gs(pv);}",
           always_lead=True,
           note="THREE objects   PRED 38 ([3+3+2] + 3*5 + 3*5); 42 if the"
                " scope-exit charge were per OBJECT rather than per function"),
    Family("dtor-body-loc", GS,
           "struct DL { int v; DL(int a){ v = gs(a)+a; }"
           " ~DL(){ int t=gs(v); gs(t); } };\n"
           "static int ldl2(int a){ DL d(a); return d.v; }",
           "s=ldl2(s);", "{int dv=gs(s)+s; int t=gs(dv); gs(t); s=dv;}",
           always_lead=True,
           note="the DESTRUCTOR BODY declares one local   PRED 18 (16 + 2*1),"
                " i.e. the destructor's own E scales with ITS depth"),
    Family("d2-dtor", GS,
           "struct DQ { int v; DQ(int a){ v = gs(a)+a; } ~DQ(){ gs(v); } };\n"
           "static int ldq(int a){ DQ d(a); return d.v; }\n"
           "static int ldr(int a){ return ldq(a)+1; }",
           "s=ldr(s);", "{int dv=gs(s)+s; gs(dv); s=dv+1;}",
           always_lead=True,
           note="the destructible object one level DEEPER  PRED 28"
                " (3 + [5+2*(1+2)] + 7 + 7)"),
    Family("ptr-mixed", "int gs(int); extern int gv;",
           "static void lpx(int* o, int* p, int a){ *o = gs(a); *p = a+1; }",
           "lpx(&s, &gv, s);", "{int q=s; s=gs(q); gv=q+1;}",
           note="one arg &<local>, one arg &<global>   PRED 4 — the +1 fires"
                " once if ANY argument points at automatic storage"),
    Family("ptr-2global", "int gs(int); extern int gv; extern int gw;",
           "static void lpy(int* o, int* p, int a){ *o = gs(a); *p = a+1; }",
           BAR + "lpy(&gv, &gw, s);", BAR + "{gv = gs(s); gw = s+1;}",
           note="BOTH args &<global>                    PRED 3"),
    Family("d2-ptr-auto", GS,
           "static void pa1(int* o, int a){ *o = gs(a)+a; }\n"
           "static int pa2(int a){ int t=a; pa1(&t, a); return t+1; }",
           "s=pa2(s);", "{int t=s; t=gs(s)+s; s=t+1;}",
           note="the automatic-storage pointer argument at DEPTH 2   PRED 11"
                " (4 + [5+2*1]) — is the +1 depth-scaled like any E feature?"),

    # === ROUND 17: "points at automatic storage" is not the rule either.
    #     `struct-ref` binds a `const SR&` to a LOCAL STRUCT of P and costs 3,
    #     while `ref-const-read` binds a `const int&` to a LOCAL SCALAR of P
    #     and costs 4 — same storage class, same constness, same read-only
    #     use. The one thing that separates them is that a struct is already
    #     in memory and a scalar is in a register, so the +1 would be "a
    #     scalar had to be given an address". Both of these probes point at
    #     automatic storage that is ALREADY addressable, so the two readings
    #     disagree about them: 3 for "a scalar left a register", 4 for
    #     "automatic storage".
    Family("ptr-arrelem", GS,
           "static void lar(int* o, int a){ *o = gs(a)+a; }",
           "lar(&arr[0], s); s+=arr[0];", "{arr[0] = gs(s)+s;} s+=arr[0];",
           head="int P(int a){ int arr[4]; arr[0]=a; int s=gs(a)+a;",
           note="&<element of a LOCAL ARRAY>   PRED 3 scalar-left-a-register"
                " / 4 automatic-storage"),
    Family("ref-member", GS,
           "struct RM { int x, y; };\n"
           "static void lrm(int& o, int a){ o = gs(a)+a; }",
           "lrm(ob.x, s); s+=ob.x;", "{ob.x = gs(s)+s;} s+=ob.x;",
           head="int P(int a){ RM ob; ob.x=a; ob.y=a; int s=gs(a)+a;",
           always_lead=True,
           note="int& bound to a MEMBER of a local struct   PRED 3 / 4, same"
                " two readings"),

    # === ROUND 18: THE DEPTH LADDER. Every one of the three rules rounds
    #     13-17 arrived at is exact at depth 1 and thin or wrong above it: the
    #     switch has two depth-2 rows and no depth-3 row, and the scope-exit
    #     and addressability rules have one depth-2 row EACH and both MISS
    #     (`d2-dtor` law 28 / measured 27, `d2-ptr-auto` law 11 / measured 9).
    #     Real TUs inline several levels deep and are made of ctors, dtors and
    #     switches, so this is where the law will actually be used.
    #
    #     Every PRED below was written down and committed BEFORE the capture.
    #     The two misses are NOT fitted away: the corrections these rows test
    #     are derived from HELD-OUT cells (`d2-ctor` / `d3-ctor` carry no
    #     contested term at all and pin the ctor tree's depth arithmetic on
    #     their own), and each row states its rivals so the run discriminates
    #     rather than confirms.
    #
    #  (a) CTOR TREE, uncontested. §6.9 says a constructor is itself an inline
    #      instance, so `lq1` at depth d puts CQ::CQ at depth d+1. Nothing
    #      about scope-exit or addressability enters these two rows, which is
    #      the point: they are the control that lets `d2-dtor` minus `d2-ctor`
    #      read the scope-exit term OFF THE MEASUREMENT instead of off the law.
    Family("d2-ctor", GS,
           "struct CQ { int v; CQ(int a){ v = gs(a)+a; } };\n"
           "static int lq1(int a){ CQ c(a); return c.v; }\n"
           "static int lq2(int a){ return lq1(a)+1; }",
           "s=lq2(s);", "s=gs(s)+s+1;",
           always_lead=True,
           note="the constructed object at DEPTH 2   PRED 17"
                " (3 + [5+2*1] + 7) — no contested term; the CONTROL for"
                " d2-dtor"),
    Family("d3-ctor", GS,
           "struct CR { int v; CR(int a){ v = gs(a)+a; } };\n"
           "static int lr1(int a){ CR c(a); return c.v; }\n"
           "static int lr2(int a){ return lr1(a)+1; }\n"
           "static int lr3(int a){ return lr2(a)+2; }",
           "s=lr3(s);", "s=gs(s)+s+3;",
           always_lead=True,
           note="the constructed object at DEPTH 3   PRED 27"
                " (3 + 5 + [7+3*1] + 9) — the CONTROL for d3-dtor"),
    #  (b) SCOPE-EXIT at depth. The law's word is `E += 2`, i.e. a term worth
    #      2*d at depth d. That is exact on eight depth-1-owner rows and
    #      misses by 1 on the one depth-2 row. An affine S(d) through (1,2)
    #      and the value d2-dtor implies at d=2 is S(d) = d+1; `d3-dtor` is
    #      then a genuine extrapolation and `d2-dtor-only` / `d2-dtor-2obj`
    #      are independent depth-2 cells that test whether the 3 is really S
    #      and not something peculiar to d2-dtor's shape.
    Family("d2-dtor-only", GS,
           "struct DY { int v; ~DY(){ gs(v); } };\n"
           "static int ldy(int a){ DY d; d.v=gs(a)+a; return d.v; }\n"
           "static int ldw(int a){ return ldy(a)+1; }",
           "s=ldw(s);", "{int dv=gs(s)+s; gs(dv); s=dv+1;}",
           always_lead=True,
           note="a dtor-only object at DEPTH 2   PRED 21 if scope-exit is"
                " E+=2 (3 + [5+2*(1+2)] + 7) / 20 if it is d+1 / 19 if flat"),
    Family("d2-dtor-2obj", GS,
           "struct DW { int v; DW(int a){ v = gs(a)+a; } ~DW(){ gs(v); } };\n"
           "static int ldu(int a){ DW p(a); DW q(a); return p.v+q.v; }\n"
           "static int ldv(int a){ return ldu(a)+1; }",
           "s=ldv(s);",
           "{int pv=gs(s)+s; int qv=gs(s)+s; int rr=pv+qv; gs(qv); gs(pv);"
           " s=rr+1;}",
           always_lead=True,
           note="TWO destructible objects at DEPTH 2   PRED 44 if scope-exit"
                " is E+=2 once (3 + [5+2*(2+2)] + 2*7 + 2*7) / 43 if d+1 /"
                " 42 if a flat +2 / 48 if it is per-OBJECT and scaled"),
    Family("d3-dtor", GS,
           "struct DX { int v; DX(int a){ v = gs(a)+a; } ~DX(){ gs(v); } };\n"
           "static int lx1(int a){ DX d(a); return d.v; }\n"
           "static int lx2(int a){ return lx1(a)+1; }\n"
           "static int lx3(int a){ return lx2(a)+2; }",
           "s=lx3(s);", "{int dv=gs(s)+s; gs(dv); s=dv+3;}",
           always_lead=True,
           note="the destructible object at DEPTH 3   PRED 42 if scope-exit"
                " is E+=2 (3 + 5 + [7+3*(1+2)] + 9 + 9) / 40 if d+1 / 38 if"
                " flat  <== the extrapolation, held out from the d+1 fit"),
    #  (c) SWITCH at depth 3. E(switch) = groups + 2, depth-scaled, was fitted
    #      on the depth-1 arm ladder and confirmed at depth 2 at TWO group
    #      counts (d2-sw2 17, d2-switch 23). Depth 3 at the same two group
    #      counts is pure extrapolation: if the term were affine in d with a
    #      non-zero intercept the two depth-2 rows would already have caught
    #      it, so what these test is that nothing new appears below depth 2.
    Family("d3-switch", GS,
           "static int st1(int a){ switch(a){ case 1: return gs(a);"
           " case 2: return a+2; case 7: return a+7; case 8: return a+8;"
           " default: return 0; } }\n"
           "static int st2(int a){ return st1(a)+1; }\n"
           "static int st3(int a){ return st2(a)+2; }",
           "s=st3(s);",
           "{switch(s){ case 1: s=gs(s); break; case 2: s=s+2; break;"
           " case 7: s=s+7; break; case 8: s=s+8; break; default: s=0; }"
           " s=s+3;}",
           note="the 5-arm switch at DEPTH 3   PRED 37 (3 + 5 + [7+3*7] + 1)"),
    Family("d3-sw2", GS,
           "static int sn1(int a){ switch(a){ case 1: return gs(a);"
           " default: return 0; } }\n"
           "static int sn2(int a){ return sn1(a)+1; }\n"
           "static int sn3(int a){ return sn2(a)+2; }",
           "s=sn3(s);",
           "{switch(s){ case 1: s=gs(s); break; default: s=0; } s=s+3;}",
           note="a 2-arm switch at DEPTH 3      PRED 28 (3 + 5 + [7+3*4] + 1)"
                " — the group slope at depth 3 is (37-28)/(5-2) = 3 = d"),
    Family("d2-sw-void", GS,
           "static void sv1(int a){ switch(a){ case 1: gs(a); break;"
           " case 2: gs(a+2); break; case 7: gs(a+7); break;"
           " case 8: gs(a+8); break; default: gs(0); } }\n"
           "static int sv2(int a){ sv1(a); return a+1; }",
           "s=sv2(s);",
           "{switch(s){ case 1: gs(s); break; case 2: gs(s+2); break;"
           " case 7: gs(s+7); break; case 8: gs(s+8); break; default: gs(0); }"
           " s=s+1;}",
           note="the same 5 arms, VOID, at DEPTH 2   PRED 22 (3 + [5+2*7]) —"
                " d2-switch's 23 less the FLAT multi-exit +1, which is the"
                " row that says the +1 is still flat two levels down"),
    Family("d2-sw-1exit", GS,
           "static int se1(int a){ int r; switch(a){ case 1: r=gs(a); break;"
           " case 2: r=a+2; break; case 7: r=a+7; break; case 8: r=a+8; break;"
           " default: r=0; } return r; }\n"
           "static int se2(int a){ return se1(a)+1; }",
           "s=se2(s);",
           "{int r; switch(s){ case 1: r=gs(s); break; case 2: r=s+2; break;"
           " case 7: r=s+7; break; case 8: r=s+8; break; default: r=0; }"
           " s=r+1;}",
           note="5 arms, ONE exit through a local, at DEPTH 2   PRED 24"
                " (3 + [5+2*(7+1)]) — an ordinary E feature stacking on the"
                " switch term at depth 2, and no multi-exit temp"),
    #  (d) ADDRESSABILITY at depth. `d2-ptr-auto` says the +1 does not fire at
    #      all at depth 2 — 9, not 11, and 9 decomposes as 4 + 5 with E(pa1)
    #      = 0 exactly. Two readings survive that, and they differ on
    #      `d2-ptr-p`: R1 "the +1 only ever fires at depth 1" and R2 "it fires
    #      wherever the pointee is an automatic of a REAL function", pa1's
    #      pointee being a local of the inlined pa2 rather than of P.
    Family("d2-ptr-p", GS,
           "static void pb1(int* o, int a){ *o = gs(a)+a; }\n"
           "static int pb2(int* o, int a){ pb1(o, a); return a+1; }",
           "s=pb2(&t, s); s+=t;", "{t = gs(s)+s;} s=s+1; s+=t;",
           head="int P(int a){ int t=a; int s=gs(a)+a;", tail="return s+t; }",
           note="P's OWN scalar automatic addressed and handed down to DEPTH 2"
                "   PRED 9 by R1 (the +1 fires only at depth 1) / 11 by R2"
                " (it fires wherever the pointee is a real automatic)"),
    Family("d3-ptr-auto", GS,
           "static void pc1(int* o, int a){ *o = gs(a)+a; }\n"
           "static int pc2(int a){ int t=a; pc1(&t, a); return t+1; }\n"
           "static int pc3(int a){ return pc2(a)+2; }",
           "s=pc3(s);", "{int t=s; t=gs(s)+s; s=t+3;}",
           note="the automatic-address argument at DEPTH 3   PRED 17 if the +1"
                " never fires below depth 1 (3 + 5 + [7+3*0]) / 20 if it fires"
                " depth-scaled there"),
    Family("d2-ptr-glob", "int gs(int); extern int gv;",
           "static void pd1(int* o, int a){ *o = gs(a)+a; }\n"
           "static int pd2(int a){ pd1(&gv, a); return gv+1; }",
           "s=pd2(s);", "{gv = gs(s)+s;} s=gv+1;",
           note="the same shape at DEPTH 2 with the pointee a GLOBAL   PRED 8"
                " (3 + 5) under every reading — the control that isolates"
                " d2-ptr-auto's local `t` from its pointer argument"),

    # === ROUND 19: `d2-ptr-p` came in at 8 and BOTH pre-registered rivals (9
    #     and 11) missed. 8 decomposes as 3 + 5 with E = 0 on both instances,
    #     so `pb2` pays nothing even though it sits at DEPTH 1 and is handed
    #     `&t` where `t` is P's own scalar automatic. The shipped wording —
    #     "a callee HANDED the address of a scalar automatic" — is therefore
    #     wrong twice over: it over-fires here and it over-fires at depth 2.
    #     The one thing pb2 does not do is USE the pointee: it forwards `o` to
    #     pb1 and returns `a+1`. `ptr-use-d1` is the same tree with the
    #     depth-1 instance reading `*o` as well, which is the only difference.
    Family("ptr-use-d1", GS,
           "static void pe1(int* o, int a){ *o = gs(a)+a; }\n"
           "static int pe2(int* o, int a){ pe1(o, a); return *o + 1; }",
           "s=pe2(&t, s); s+=t;", "{t = gs(s)+s;} s=t+1; s+=t;",
           head="int P(int a){ int t=a; int s=gs(a)+a;", tail="return s+t; }",
           note="the depth-1 instance USES the pointee as well as forwarding"
                " it   PRED 9 if the trigger is a load/store through the"
                " address AT DEPTH 1 (d2-ptr-p's 8 plus that +1) / 8 if the"
                " trigger also requires the address not to escape deeper"),

    # === ROUND 20: `ptr-use-d1` is 8 too, so USING the pointee at depth 1 is
    #     not the trigger either. A two-deep tree with a pointer costs exactly
    #     what a two-deep tree without one costs (d2-ptr-p 8, ptr-use-d1 8,
    #     d2-ptr-glob 8, nest2 8) while a one-deep tree with a pointer costs
    #     one more than a one-deep tree without (ptr-param 4, nest1 3). Two
    #     readings survive all sixteen rows and they differ on ONE shape:
    #
    #       G  the +1 fires when the DEEPEST use of the address is at depth 1
    #       I  the +1 fires when the using depth-1 instance is a LEAF of the
    #          expansion tree — i.e. it has no inlined callee of its own,
    #          whether or not that callee touches the address
    #
    #     `ptr-use-nest` is a depth-1 instance that uses the pointee and
    #     inlines a callee that never sees it. G says the deepest use is still
    #     depth 1, so +1; I says the instance is no longer a leaf, so 0.
    Family("ptr-use-nest", GS,
           "static int pf0(int a){ return gs(a)+a; }\n"
           "static void pf1(int* o, int a){ *o = pf0(a); }",
           "pf1(&t, s); s+=t;", "{t = gs(s)+s;} s+=t;",
           head="int P(int a){ int t=a; int s=gs(a)+a;", tail="return s+t; }",
           note="the depth-1 instance uses the pointee AND inlines a callee"
                " that never sees it   PRED 9 by G (the deepest USE is still"
                " at depth 1) / 8 by I (the instance is no longer a LEAF of"
                " the expansion tree)"),

    # === ROUND 21: `ptr-use-nest` is 8, so reading I stands — the +1 fires
    #     only when the site's whole expansion is ONE instance deep. The last
    #     thing that decides whether it ever fires in a real TU is scope: is
    #     "one instance deep" a property of the SITE's tree, or of P? This
    #     site has both — a two-deep tree that never touches the address, and
    #     a one-deep pointer tree beside it.
    Family("ptr-sibling", GS,
           "static int pg0(int a){ return gs(a)+a; }\n"
           "static int pg1(int a){ return pg0(a)+1; }\n"
           "static void pg2(int* o, int a){ *o = gs(a)+a; }",
           "s=pg1(s); pg2(&t, s); s+=t;",
           "s=gs(s)+s+1; {t = gs(s)+s;} s+=t;",
           head="int P(int a){ int t=a; int s=gs(a)+a;", tail="return s+t; }",
           note="a two-deep tree and a one-deep POINTER tree at the same site"
                "   PRED 12 if the +1 is per-TREE (8 + 4) / 11 if a nested"
                " instance anywhere in P kills it (8 + 3)"),
    Family("ptr-sibling-rev", GS,
           "static int ph0(int a){ return gs(a)+a; }\n"
           "static int ph1(int a){ return ph0(a)+1; }\n"
           "static void ph2(int* o, int a){ *o = gs(a)+a; }",
           "ph2(&t, s); s+=t; s=ph1(s);",
           "{t = gs(s)+s;} s+=t; s=gs(s)+s+1;",
           head="int P(int a){ int t=a; int s=gs(a)+a;", tail="return s+t; }",
           note="the same two trees with the POINTER SITE FIRST   PRED 11 if"
                " the kill is order-independent, i.e. a property of P and not"
                " of what the front end has already expanded"),

    # === ROUND 22: every depth-2 and depth-3 row above varies exactly ONE
    #     feature of the callee, and law L' assumes the features ADD inside
    #     the d*E product. A real TU's callee has several at once. These two
    #     hold the depth fixed at 2 and combine — the second one combining the
    #     freshly-corrected scope-exit term with ordinary E features, which is
    #     the row that would catch `d+1` being an artefact of a body that had
    #     nothing else in it.
    Family("d2-mix", GS,
           "static int mx1(int a){ int t=gs(a); int u=t+a; int r;"
           " if (u > 0) r=u; else r=u+1; return r; }\n"
           "static int mx2(int a){ return mx1(a)+1; }",
           "s=mx2(s);",
           "{int t=gs(s); int u=t+s; int r; if (u>0) r=u; else r=u+1;"
           " s=r+1;}",
           note="3 locals + an if + an else at DEPTH 2, one exit   PRED 18"
                " (3 + [5 + 2*5]) if E features simply ADD two levels down"),
    Family("d2-dtor-if", GS,
           "struct DI { int v; DI(int a){ v = gs(a)+a; } ~DI(){ gs(v); } };\n"
           "static int ldi(int a){ DI d(a); int r; if (a>0) r=d.v;"
           " else r=d.v+1; return r; }\n"
           "static int ldj(int a){ return ldi(a)+1; }",
           "s=ldj(s);",
           "{int dv=gs(s)+s; int r; if (s>0) r=dv; else r=dv+1; gs(dv);"
           " s=r+1;}",
           always_lead=True,
           note="a destructible object AND an if/else at DEPTH 2   PRED 33"
                " (3 + [5 + 2*(2 locals + if + else) + S(2)=3] + 7 + 7) —"
                " the corrected scope-exit term stacking on ordinary E"
                " features; 34 if scope-exit were still E += 2"),

    # === ROUND 23: §6.11's two `NOT MODELLED` rows, attacked with the ladder.
    #     Both are single depth-1 readings where several decompositions reach
    #     the measured number and nothing at depth 1 can separate them — which
    #     is precisely what DEPTH separates, because an E unit is worth d and
    #     a flat term is worth 1 however deep it sits. That is the same lever
    #     §6.3 used to pin the multi-exit temp as flat rather than scaled.
    #
    #     `ctor-noloc` = 10 needs the wrapper to cost 5 at depth 1, and
    #         A: E = 2                 B: E = 1 + a flat 1     C: E = 0 + flat 2
    #     all give 3+2 / 3+1+1 / 3+0+2 = 5. At depth 2 they separate cleanly.
    #
    #     `struct-ret` = 5 needs the callee to cost 5 with one declared local,
    #         A: E = 2 (the hidden return slot counts alongside the local)
    #         B: E = 1 and a flat +1 for returning a struct by value
    #     which are 4+1 either way at depth 1 and 9 against 8 at depth 2.
    #
    #     Registered prediction: A for both, because law L' has been additive
    #     in E everywhere and each flat term it does have (the multi-exit
    #     result temp) had a structural reason to be flat. Stated bias: I have
    #     been wrong about exactly this kind of "surely it just adds" twice in
    #     this file already (d2-lp-for, d2-ptr-p), so A is the reading to
    #     doubt, not the one to assume.
    Family("d2-ctor-noloc", GS,
           "struct CO { int v; CO(int a){ v = gs(a)+a; } };\n"
           "static int lco(int a){ return CO(a).v; }\n"
           "static int lcp(int a){ return lco(a)+1; }",
           "s=lcp(s);", "s=gs(s)+s+1;",
           always_lead=True,
           note="ctor-noloc's unnamed temporary at DEPTH 2   PRED 19 if the"
                " wrapper's 2 are both E units / 18 if one is E and one flat"
                " / 17 if both are flat"),
    Family("d2-struct-ret", GS,
           "struct SS { int x, y; };\n"
           "static SS sr1(int a){ SS r; r.x=gs(a); r.y=a; return r; }\n"
           "static int sr2(int a){ SS q = sr1(a); return q.x+q.y+1; }",
           "s=sr2(s);", "{SS q; q.x=gs(s); q.y=s; s=q.x+q.y+1;}",
           always_lead=True,
           note="a by-value struct return at DEPTH 2   PRED 13 if the hidden"
                " return slot is a second E unit (4 + [5+2*2]) / 12 if it is"
                " a flat +1 (4 + [5+2*1] + 1)"),

    # === ROUND 24: `d2-struct-ret` = 13 confirmed E = 2 — one parameter, two
    #     depths, a rival refuted, so that row is now tested rather than
    #     fitted. `d2-ctor-noloc` = 18 refuted the registered reading A and
    #     leaves B: the unnamed temporary is ONE E unit and there is ONE flat
    #     unit beside it. But that is two parameters solved from two cells
    #     (d1: a + b = 2, d2: 2a + b = 3), so it is EXACTLY DETERMINED and
    #     nothing has been tested. Depth 3 is the first cell with a residual:
    #     a = 1, b = 1 flat predicts 3 + 5 + [7+3*1] + 1 + 9.
    Family("d3-ctor-noloc", GS,
           "struct CP { int v; CP(int a){ v = gs(a)+a; } };\n"
           "static int lcq(int a){ return CP(a).v; }\n"
           "static int lcr(int a){ return lcq(a)+1; }\n"
           "static int lcs(int a){ return lcr(a)+2; }",
           "s=lcs(s);", "s=gs(s)+s+3;",
           always_lead=True,
           note="ctor-noloc's unnamed temporary at DEPTH 3   PRED 28 if the"
                " temporary is 1 E unit plus 1 FLAT unit (3 + 5 + [7+3*1] +"
                " 1 + 9) / 30 if the second unit scales with depth after all"),

    # === ROUND 25: §6.9's constructor tree was measured entirely on standalone
    #     structs, and the single most common constructor in a DC3 TU is one
    #     that runs a BASE-CLASS constructor first. If a base ctor is an
    #     ordinary inline instance one level below the derived ctor — which is
    #     what §6.9's "a constructor is itself an inlined function" implies —
    #     then a depth-1 wrapper owning a derived object builds a THREE-deep
    #     tree, and this is the shape where getting the depth wrong is easiest
    #     (§6.9's own trap, in the direction that made `ctor` read as 9).
    Family("ctor-base", GS,
           "struct BB { int b; BB(int a){ b = gs(a); } };\n"
           "struct DD2 : BB { int v; DD2(int a) : BB(a) { v = a+1; } };\n"
           "static int lbc(int a){ DD2 d(a); return d.v + d.b; }",
           "s=lbc(s);", "{int b=gs(s); int v=s+1; s=v+b;}",
           always_lead=True,
           note="a derived ctor running a BASE ctor   PRED 16 if the base ctor"
                " is an ordinary instance one level below the derived one"
                " (4 + [5] + [7]) / 11 if it folds into the derived ctor"),

    # === ROUND 26: law L' now has TWO terms that sit outside the d*E product
    #     — §6.6's affine loop term and §6.12's scope-exit `d + 1` — and they
    #     have never appeared in the SAME instance. Nothing measured so far
    #     says they add; they were fitted on disjoint bodies. A DC3 constructor
    #     that loops over its members is an ordinary sight, so this is the
    #     cheapest large hole left in the law.
    Family("dtor-loop", GS,
           "struct DS { int v; DS(int a){ v = gs(a)+a; } ~DS(){ gs(v); } };\n"
           "static int lds(int a){ DS d(a); int t=0;"
           " for(int i=0;i<a;i++) t+=gs(i); return t+d.v; }",
           "s=lds(s);",
           "{int dv=gs(s)+s; int t=0; for(int i=0;i<s;i++) t+=gs(i);"
           " s=t+dv; gs(dv);}",
           always_lead=True,
           note="a destructible object AND a for loop in ONE instance"
                "   PRED 23 if the two non-E terms simply ADD"
                " ([3 + 3 locals + for(1)=5 + S(1)=2] + 5 + 5); 21 if the"
                " scope-exit term is not paid when a loop is present"),
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
    # round 16 — hold-outs for the rules rounds 13-15 arrived at
    "sw-ctx-expr": None, "sw-nodefault": None, "sw-withdefault": None,
    "sw-fall": None, "d2-sw2": None,
    "cf-else": None, "cf-else-assign": None,
    "dtor-direct-only": None, "dtor-3obj": None, "dtor-body-loc": None,
    "d2-dtor": None,
    "ptr-mixed": None, "ptr-2global": None, "d2-ptr-auto": None,
    # round 17
    "ptr-arrelem": None, "ref-member": None,
}

# ---------------------------------------------------------------------------
# LAW_BOOK — the same law L', stated against the BOOKKEEPING slope (the inline
# record alone) instead of the marginal (the inline record PLUS whatever §1.1
# surcharge P owes for the code it ends up containing).
#
# Rounds 13-17 are recorded here rather than in LAW because their hand controls
# are not zero, and for `switch` that difference is the whole story: a SECOND
# written-out switch costs P 4 whether or not anything was inlined, which is
# why §6.7 first read `switch-body` as "10 at N=1, 14 marginal, not even
# uniform in N". The inline record is 10, flat, from N=1 to N=5. Where the hand
# control is 0 the two dicts mean exactly the same thing.
#
# The three rules these entries encode, all fitted and then held out:
#
#   SWITCH.  An ordinary depth-scaled E feature, NOT an affine term of its own
#            like the loop:  E(switch) = (statement groups) + 2, where a
#            `default` counts as a group only when it is WRITTEN and case
#            labels sharing one group count once. Measured 2..6 groups at
#            depth 1 (7/8/9/10/11) and 2 and 5 groups at depth 2 (17/23).
#
#   IF/ELSE. An explicit `else` is its own E unit. `cf-if` (if + fallthrough)
#            is 4 and `cf-else` (the same code with `else`) is 5.
#
#   SCOPE-EXIT. A function that owns any local with a non-trivial destructor
#            pays, ONCE — not per object — a term worth **d + 1** at its own
#            depth d. NOT an E unit: an E unit would be worth 2*d, which is
#            the same 2 at depth 1 and one too many at depth 2. Constructors
#            and destructors are otherwise ordinary inline instances with
#            E = 0, exactly like any other callee, and P itself pays nothing
#            because P is not an inline instance.  (ROUND 18 — see below.)
#
#   POINTER/REFERENCE. +1 once per depth-1 instance handed the address of a
#            SCALAR AUTOMATIC variable — the thing that has to leave a
#            register to acquire an address — **and only when P's ENTIRE
#            expansion is flat**, i.e. P contains no inline instance below
#            depth 1 at all. Not "an argument that needed a temp" (§6.7
#            predicted from that and both predictions inverted), not
#            "automatic storage" (a local array element, a local struct member
#            and a whole local struct by `const&` are already addressable and
#            all cost 0, as do a global and a function-static), and — ROUND
#            19-21 — not "handed at depth 1", not "used at depth 1", not
#            "the deepest use is at depth 1". One nested inline anywhere in P,
#            even at an unrelated call site that never touches the address,
#            removes the +1 entirely.
#
# ROUND 18-21 REWROTE THE LAST TWO OF THOSE FOUR RULES. The two live
# refutations this block used to carry — `d2-dtor` (law 28, measured 27) and
# `d2-ptr-auto` (law 11, measured 9) — are now consequences of the corrected
# rules rather than outstanding misses. They were NOT fitted away: the
# corrections are derived from held-out cells (`d2-ctor` = 17 carries no
# contested term and pins the scope-exit value at depth 2 by subtraction;
# `d3-dtor` = 40 is the extrapolation; `ptr-use-d1`, `ptr-use-nest`,
# `ptr-sibling` each refuted a pre-registered rival for the pointer rule), and
# the retired wordings are kept in SUPERSEDED below so their refutations
# re-run on every invocation instead of being remembered.
# ---------------------------------------------------------------------------
LAW_BOOK = {
    # --- switch: E = groups + 2, depth-scaled, multi-exit +1 as usual -------
    "sw-arms2": 7, "sw-arms3": 8, "sw-arms4": 9, "switch-body": 10,
    "sw-arms6": 11, "sw-dense": 10, "sw-void": 10, "sw-1exit": 11,
    "sw-nodefault": 9, "sw-withdefault": 10, "sw-fall": 9, "sw-ctx-expr": 11,
    "d2-switch": 23, "d2-sw2": 17,
    # --- an explicit `else` is an E unit ------------------------------------
    "cf-else": 5, "cf-else-assign": 6,
    # --- ctors/dtors are ordinary instances; the OWNER pays +2 once ---------
    "ctor": 9, "ctor-direct": 3, "ctor-loc": 11, "ctor-if": 13,
    "ctor-init": 9, "ctor-2mem": 9,
    "dtor": 16, "dtor-empty": 16, "dtor-only": 11, "dtor-2obj": 27,
    "dtor-3obj": 38, "dtor-body-loc": 18, "dtor-direct": 6,
    "dtor-direct-only": 3,
    "d2-dtor": 27,          # 3 + [5 + 2*1 + S(2)=3] + 7 + 7   (S(d) = d+1)
    # --- the pointer/reference +1 is ADDRESSABILITY, and only in a FLAT P ---
    "ref-param": 4, "ptr-param": 4, "ptr-already": 4, "ref-const-read": 4,
    "ptr-2args": 4, "ptr-mixed": 4,
    "ptr-global": 3, "ref-global": 3, "ptr-static-local": 3, "ptr-2global": 3,
    "ptr-arrelem": 3, "ref-member": 3, "struct-ref": 3, "struct-param": 3,
    "d2-ptr-auto": 9,       # P is not flat, so the +1 does not fire: 4 + 5
    # --- ROUND 18-21, THE DEPTH LADDER -------------------------------------
    # Every one of these was committed with law L's word BEFORE its capture
    # (see the git history of this file); the values below are the corrected
    # law, and the pre-registered predictions each row was graded against are
    # in its family note and in SUPERSEDED.
    #   uncontested controls — exact as first written:
    "d2-ctor": 17, "d3-ctor": 27,
    #   switch, exact as first written at depth 3 and at two group counts:
    "d3-switch": 37, "d3-sw2": 28, "d2-sw-void": 22, "d2-sw-1exit": 24,
    #   scope-exit S(d) = d+1, all three refuted the E+=2 wording:
    "d2-dtor-only": 20, "d2-dtor-2obj": 43, "d3-dtor": 40,
    #   addressability: fires only when P's whole expansion is flat:
    "d2-ptr-p": 8, "d3-ptr-auto": 17, "d2-ptr-glob": 8,
    "ptr-use-d1": 8, "ptr-use-nest": 8,
    "ptr-sibling": 11, "ptr-sibling-rev": 11,
    # --- ROUND 22: additivity at depth 2, committed before capture ----------
    "d2-mix": 18, "d2-dtor-if": 33,
    # --- ROUND 23: the registered reading (A) for §6.11's two NOT MODELLED
    # rows. A miss prints, which is the point — these are the two shapes the
    # document has refused to put a number on, and a wrong guess here is
    # worse than the blank they currently have.
    "d2-ctor-noloc": 18, "d2-struct-ret": 13, "struct-ret": 5,
    # --- ROUND 24: the first cell with a residual for ctor-noloc ------------
    "d3-ctor-noloc": 28, "ctor-noloc": 10,
    # --- ROUND 25 ----------------------------------------------------------
    "ctor-base": 16,
    # --- ROUND 26 ----------------------------------------------------------
    # NB the UNITS. This family is the first with BOTH a loop and a non-zero
    # hand control, so the two dicts genuinely differ on it: the marginal is
    # 23 (§6.6's `for` term is 3d+2 = 5, a marginal figure) and the inline
    # record is 21 (§6.11's split of that term into 3d = 3 plus a +2 that is
    # P's own §1.1 surcharge for containing a loop at all — which the hand
    # control measures here, independently, at exactly +2/site). LAW_BOOK is
    # graded on the record, so 21. Both readings agree; the registered PRED
    # of 23 was in marginal units and the measured marginal is 23.
    "dtor-loop": 21,
}

# ---------------------------------------------------------------------------
# SUPERSEDED — the wordings rounds 18-21 retired, kept as data so that every
# run RECOMPUTES their refutation instead of the reader recalling it. A row
# whose measurement disagrees with the retired rule prints the disagreement
# beside its verdict; a row that agreed with both is silent, because it never
# discriminated. This is the same discipline as §3's `stride == minted` tag:
# the point of a superseded model is that it is re-refuted on every run, not
# that it is deleted and the reason written down in prose somewhere.
# ---------------------------------------------------------------------------
SUPERSEDED = {
    # "the owner of a destructible local pays E += 2" — worth 2*d at depth d.
    # Exact at depth 1 (eight rows), one too many at depth 2 and two too many
    # at depth 3.
    "d2-dtor": (28, "scope-exit as E += 2"),
    "d2-dtor-only": (21, "scope-exit as E += 2"),
    "d2-dtor-2obj": (44, "scope-exit as E += 2"),
    "d3-dtor": (42, "scope-exit as E += 2"),
    # "+1 once per callee HANDED the address of a scalar automatic", scaled by
    # that callee's depth like any other E unit.
    "d2-ptr-auto": (11, "addressability as a depth-scaled E unit"),
    "d2-ptr-p": (11, "addressability as a depth-scaled E unit"),
    "d3-ptr-auto": (20, "addressability as a depth-scaled E unit"),
    # the three intermediate readings, each refuted by exactly one row
    "ptr-use-d1": (9, "addressability keyed on USE at depth 1"),
    "ptr-use-nest": (9, "addressability keyed on the DEEPEST use being depth 1"),
    "ptr-sibling": (12, "addressability scoped to the call site's own tree"),
    "d2-dtor-if": (34, "scope-exit as E += 2"),
    "d2-ctor-noloc": (19, "the unnamed temporary as TWO E units"),
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
    # A family recorded in LAW_BOOK is graded on the inline record alone; one
    # recorded in LAW is graded on the marginal, which is what every entry
    # written before round 13 means. Both are the same number wherever the hand
    # control is 0, i.e. on 72 of the original 100 families.
    if name in LAW_BOOK:
        want, got, tag = LAW_BOOK[name], book, "law(book)"
    else:
        want, got, tag = LAW.get(name, "?"), kinc, "law"
    # A LAW entry is a prediction about ONE expansion tree — the one the front
    # end builds for that source. When the front end declines an inline the tree
    # is a different tree, and the law is not being asked the question it
    # answers. Saying so is not an excuse: `INLINE-DECLINED?` is computed from
    # P's own `.text` growth against the hand control, printed on the offending
    # row, and it is what turns 10 apparent /Ox refutations into 10 rows where
    # `while`/`do` loop bodies simply were not inlined at the inner level.
    if want is None:
        verdict = "%s: NOT MODELLED" % tag
    elif want == "?":
        verdict = "%s: no entry" % tag
    elif want == got:
        verdict = "%s %d OK" % (tag, want)
    elif refused:
        verdict = "%s %d n/a — the front end declined an inline, so this is a" \
                  " DIFFERENT expansion tree" % (tag, want)
    else:
        verdict = "%s %d vs %s  <== *** REFUTES LAW L' ***" % (tag, want, got)
    # A retired wording is re-refuted here, from this run's own measurement,
    # rather than remembered. Silence means the row never discriminated.
    sup = SUPERSEDED.get(name)
    if sup is not None and got is not None and not refused and sup[0] != got:
        verdict += "   [retired '%s' said %d, measured %s]" % (sup[1], sup[0],
                                                              got)
    shape = ("LINEAR to N=%d" % nmax if lin
             else "one-off %+d at N=1, linear after" % oneoff if oneoff
             else "*** NON-LINEAR ***")
    print("    -> %s/site marginal (%s at N=1), %s;  hand control %s/%s;"
          "  bookkeeping %s/site;  %s%s"
          % (kinc, ki, shape, kh, khinc, book, verdict,
             "  (see INLINE-DECLINED? rows)" if refused else ""))
    print()
    return bad, (want not in (None, "?") and want != got and not refused)


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
