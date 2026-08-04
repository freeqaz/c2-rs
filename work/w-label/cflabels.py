#!/usr/bin/env python3
"""cflabels.py — what does CONTROL FLOW cost the compiler-label counter?

`docs/LABEL_COUNTER.md` §4 records four control-flow readings and calls three of
them "measured, not modelled":

    an if/ternary that BRANCHES on a relational   +0
    `for` / `while`                               +2   (nested +4)
    `do/while`                                    +1   <- "so 'per loop' is
                                                          already wrong"
    `switch` (8 dense arms, no jump table)        +0

and W11 (`docs/rungs/2026-08-04-w-conv.md` §4) added a fifth from the other end:
the **exit-value merge** — two guards returning the same literal, which c2 emits
as one arm with a **backward** branch into it — moves the counter where every
in-class early-return cell does not.

Those five readings were taken by five different lanes for five different
reasons and have never been put on one axis. This script puts them on one axis:
it is `scripts/gt_label_stride.py`'s seed-free construction (three in-TU
anchors, every number a difference inside one obj) with a probe list that varies
**only the control-flow shape** and holds the function class fixed.

It imports the shipped instrument rather than copying it, so the anchor control,
the group walker and the `minted` counter are the same code that produced
`LABEL_COUNTER.md` §1 — a copy would be a second instrument to keep honest.

    work/w-label/cflabels.py                       # /O1 (the workload's mode)
    work/w-label/cflabels.py --mode '/Ox /GS- /c'  # packed
    work/w-label/cflabels.py --list

Exit status is non-zero only if a *control* failed (an anchor pair disagreeing
with the measured base), never because a prediction did.
"""

import os
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, os.path.join(REPO, "scripts"))
import gt_label_stride as G  # noqa: E402

DECL = "int gp(int);"

# (name, decls, leads, probe, note)
#
# Every probe below is a **framed Class-A** body — one call to `gp`, result
# consumed — so `extra` and `stride` are both defined and the only thing moving
# between rows is the control flow. The `cf-none` row is the base.
PROBES = [
    # ---- base -------------------------------------------------------------
    ("cf-none", DECL, [],
     "int P(int a){ return gp(a)+1; }",
     "BASE: framed Class A, no control flow at all"),

    # ---- forward-only branches -------------------------------------------
    ("cf-if", DECL, [],
     "int P(int a){ if (a) return 5; return gp(a); }",
     "one guarded early return, value arm (W11's in-class shape, n=1)"),
    ("cf-if2", DECL, [],
     "int P(int a,int b){ if (a) return 5; if (b) return 11; gp(a); return 0; }",
     "TWO guards, DISTINCT literals -- W11 in-class, two `b` to the epilogue"),
    ("cf-if3", DECL, [],
     "int P(int a,int b,int c){ if (a) return 5; if (b) return 11;"
     " if (c) return 22; gp(a); return 0; }",
     "THREE guards, distinct literals -- three `b` naming ONE target"),
    ("cf-ifelse", DECL, [],
     "int P(int a){ int r; if (a) r = gp(a); else r = gp(a+1); return r+1; }",
     "if/else with a join block"),
    ("cf-ifelse-val", DECL, [],
     "int P(int a){ gp(a); if (a) return 5; else return 11; }",
     "if/else, both arms return a literal"),
    ("cf-void-guard", DECL, [],
     "void P(int a){ if (a) return; gp(a); }",
     "the EMPTY arm -- W11's branch-sense inversion, `bf` straight to the epilogue"),

    # ---- the merge: the same literal from two exits ------------------------
    ("cf-merge2", DECL, [],
     "int P(int a,int b){ if (a) return 5; if (b) return 5; gp(a); return 0; }",
     "TWO guards, SAME literal -- c2 merges the arms and branches BACKWARD  <== THE PROBE"),
    ("cf-merge3", DECL, [],
     "int P(int a,int b,int c){ if (a) return 5; if (b) return 5;"
     " if (c) return 5; gp(a); return 0; }",
     "THREE guards, one literal: is the merge charge per merge or per arm?"),
    ("cf-merge-tail", DECL, [],
     "int P(int a,int b){ if (a) return 0; if (b) return 11; gp(a); return 0; }",
     "a guard returning the SEQUENCE's own literal -- the arm disappears entirely"),
    ("cf-merge-mixed", DECL, [],
     "int P(int a,int b,int c){ if (a) return 5; if (b) return 11;"
     " if (c) return 5; gp(a); return 0; }",
     "three guards, literals 5/11/5: one merged PAIR among three arms"),

    # ---- explicit backward / forward transfers -----------------------------
    ("cf-goto-fwd", DECL, [],
     "int P(int a){ int r = 1; if (a) goto out; r = gp(a); out: return r; }",
     "an explicit FORWARD goto"),
    ("cf-goto-back", DECL, [],
     "int P(int a){ int r = 0; top: r += gp(a); if (--a) goto top; return r; }",
     "an explicit BACKWARD goto -- a loop written without a loop keyword"),

    # ---- loops -------------------------------------------------------------
    ("cf-dowhile", DECL, [],
     "int P(int a){ int r = 0; do { r += gp(a); } while (--a); return r; }",
     "do/while -- LABEL_COUNTER.md §4 says +1"),
    ("cf-while", DECL, [],
     "int P(int a){ int r = 0; while (a) { r += gp(a); --a; } return r; }",
     "while -- §4 says +2"),
    ("cf-for", DECL, [],
     "int P(int a){ int r = 0; for (int i = 0; i < a; i++) r += gp(i); return r; }",
     "for -- §4 says +2"),
    ("cf-for-break", DECL, [],
     "int P(int a){ int r = 0; for (int i = 0; i < a; i++)"
     " { r += gp(i); if (r > 100) break; } return r; }",
     "for + break: does an extra exit edge cost a slot?"),
    ("cf-for-continue", DECL, [],
     "int P(int a){ int r = 0; for (int i = 0; i < a; i++)"
     " { if (i == 3) continue; r += gp(i); } return r; }",
     "for + continue: does an extra back edge cost a slot?"),
    ("cf-forever", DECL, [],
     "int P(int a){ int r = 0; for (;;) { r += gp(a); if (--a == 0) break; }"
     " return r; }",
     "for(;;) + break -- one back edge, one exit, NO entry test"),
    ("cf-for2", DECL, [],
     "int P(int a){ int r = 0; for (int i = 0; i < a; i++) r += gp(i);"
     " for (int j = 0; j < a; j++) r += gp(j); return r; }",
     "TWO sequential for loops -- is the charge per loop?"),
    ("cf-fornest", DECL, [],
     "int P(int a){ int r = 0; for (int i = 0; i < a; i++)"
     " for (int j = 0; j < a; j++) r += gp(j); return r; }",
     "nested for -- §4 says +4"),
    ("cf-dowhile2", DECL, [],
     "int P(int a){ int r = 0; do { r += gp(a); } while (--a);"
     " do { r += gp(r); } while (--a); return r; }",
     "TWO sequential do/whiles"),

    # ---- switch ------------------------------------------------------------
    ("cf-switch-dense", DECL, [],
     "int P(int a){ switch (a) { case 0: return gp(1); case 1: return gp(2);"
     " case 2: return gp(3); case 3: return gp(4); case 4: return gp(5);"
     " case 5: return gp(6); case 6: return gp(7); case 7: return gp(8);"
     " default: return 0; } }",
     "8 DENSE arms -- does a jump table appear, and does it cost slots?"),
    ("cf-switch-sparse", DECL, [],
     "int P(int a){ switch (a) { case 1: return gp(1); case 90: return gp(2);"
     " case 7000: return gp(3); default: return 0; } }",
     "3 SPARSE arms -- a compare chain, not a table"),
]

# ---------------------------------------------------------------------------
# HELD OUT. These were named in `work/w-label/PREREG.md` §3.3 prediction **L6**
# and committed before they were run. L6 registers that §1.4's boundary --
# "forward-only is necessary, and inside it the only charging shapes are
# §3.4.1's code-motion ones" -- survives cells chosen to break it, and that the
# refuted "interior join" rule is not rescued by any of them.
#
# The last two are not about L6: they price the shape `calls.rs:415` refuses by
# name (a guarded call and a guarded early return in one body, which w-conv
# measured c2 as composing) so the next lane has its counter cost before it
# starts rather than after.
HELDOUT = [
    ("ho-if4", DECL, [],
     "int P(int a,int b,int c,int d){ if (a) return 5; if (b) return 11;"
     " if (c) return 22; if (d) return 33; gp(a); return 0; }",
     "FOUR forward guards, four distinct literals -- L6 says +0"),
    ("ho-if-nested", DECL, [],
     "int P(int a,int b){ if (a) { if (b) return 5; return 11; } gp(a); return 0; }",
     "NESTED forward guards -- L6 says +0"),
    ("ho-ternary", DECL, [],
     "int P(int a,int b){ return gp(a ? b : b + 1) + 1; }",
     "a ternary in an argument -- forward only, L6 says +0"),
    ("ho-and", DECL, [],
     "int P(int a,int b){ if (a && b) return 5; gp(a); return 0; }",
     "short-circuit && -- two compares, one arm, forward only, L6 says +0"),
    ("ho-or", DECL, [],
     "int P(int a,int b){ if (a || b) return 5; gp(a); return 0; }",
     "short-circuit || -- the second test is the JOIN case; L6 is at risk here"),
    ("ho-void-2guard", DECL, [],
     "void P(int a,int b){ if (a) return; if (b) return; gp(a); gp(b); }",
     "two VOID guards over two calls -- W11's in-class void shape, L6 says +0"),
    ("ho-arm-call", DECL, [],
     "int P(int a,int b){ if (a) { gp(b); return 5; } gp(a); return 0; }",
     "a guard whose ARM contains a call -- forward only, L6 says +0"),
    # --- not L6: pricing the composition `calls.rs:415` refuses --------------
    ("ho-compose", DECL, [],
     "int P(int a,int b,int c){ if (a) return 5; if (b) gp(b); gp(c); return 0; }",
     "a guarded EARLY RETURN and a guarded CALL in one body -- `calls.rs:415`"
     " refuses this and w-conv measured c2 composing it"),
    ("ho-compose2", DECL, [],
     "int P(int a,int b,int c){ if (a) return 5; if (b) return 11;"
     " if (c) gp(c); gp(a); return 0; }",
     "the same with TWO early returns ahead of the guarded call"),
]


def main(argv):
    if "--list" in argv:
        for p in PROBES + HELDOUT:
            print("%-20s %s" % (p[0], p[4]))
        return 0
    mode = "/O1 /GS- /c"
    if "--mode" in argv:
        i = argv.index("--mode")
        mode = argv[i + 1]
        del argv[i:i + 2]
    want = [a for a in argv[1:] if not a.startswith("--")]
    pool = PROBES + HELDOUT if "--heldout" in argv else PROBES
    probes = [p for p in pool if not want or p[0] in want]

    print("mode: %s" % mode)
    print("anchors: 3x plain Class-A framed; `control` is the anchor base "
          "MEASURED in this obj (5 /Gy, 4 packed)")
    print("`extra` = slots taken before P's own $M; `stride` = slots P "
          "consumes in total; base is `control`")
    print()
    print("%-18s %6s %6s %7s %8s  %s"
          % ("probe", "extra", "stride", "minted", "control", "surcharge"))
    bad = 0
    base_stride = None
    wd = tempfile.mkdtemp(prefix="cflbl")
    rows = []
    for p in probes:
        row = G.run(p[0], p[1], p[2], p[3], p[4], mode, wd)
        if row is None:
            print("%-18s  CAPTURE FAILED" % p[0]); bad += 1; continue
        if "error" in row:
            print("%-18s  %s" % (p[0], row["error"])); bad += 1; continue
        if row["control"] not in (4, 5):
            bad += 1
        if p[0] == "cf-none":
            base_stride = row["stride"]
        sur = ("+%d" % (row["stride"] - base_stride)
               if base_stride is not None else "?")
        rows.append((p[0], row, sur))
        print("%-18s %6s %6d %7d %8d  %6s   %s" % (
            p[0],
            "-" if row["extra"] is None else row["extra"],
            row["stride"], row["minted"], row["control"], sur, p[4]))
    print()
    print("controls failed: %d" % bad)
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
