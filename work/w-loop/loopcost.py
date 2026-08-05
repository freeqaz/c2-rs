#!/usr/bin/env python3
"""loopcost.py — what does a LEAF loop cost the compiler-label counter, and is
that cost observable in the obj at all?

Lane **w-loop**. Control: `work/w-loop/PREREG.md`, committed at `0fa469f`
before this file existed.

# Why this is not `work/w-label/cflabels.py` again

`cflabels.py` measured 33 control-flow cells and every one of its probes is a
**framed Class-A body** — the probe calls `gp` so that `extra` (the slots taken
before the probe's own `$M` pair) is defined. That is the right construction for
its question and the wrong population for this one: **every loop function on the
codegen frontier is a leaf.** `Sort.cpp`, `Primes.cpp` and
`IPP_basicmath_xbox.cpp` between them carry six loop functions, no `.pdata`, and
not one `$M` symbol. A framed loop's charge does not price them.

So this script asks two questions `cflabels.py` structurally cannot:

**Q1 (the stride).** What is `stride(P)` when `P` is a **leaf** whose body is a
loop? `gt_label_stride.py`'s `leaf-int` row reads **1**. Does a loop move it?
Reused verbatim from the shipped instrument, so the anchor control, the group
walker and the `minted` counter are the same code that produced
`LABEL_COUNTER.md` §1 — a copy would be a second instrument to keep honest.

**Q2 (the observability).** In a TU whose functions are **all leaves**, does the
obj contain any `$M`/`$T` symbol at all? `coff::plan_labels` mints a triple only
for a function with a `frame`, so if c2 agrees, the counter's *value* never
reaches such an obj and `labels.rs` invariant 4's stated justification — *"the
obj would carry a wrong `$M`"* — is vacuous there. Q2 is run on a **different TU
construction** from Q1 (no framed anchors at all), which is the whole point: Q1
needs framed anchors to read the counter, Q2 needs their absence to prove it is
unreadable.

    work/w-loop/loopcost.py                        # both, /O1 (the workload's mode)
    work/w-loop/loopcost.py --mode '/Ox /GS- /c'   # packed
    work/w-loop/loopcost.py --q1 / --q2            # one half
    work/w-loop/loopcost.py --dis <probe>          # disassemble one leaf probe

Exit status is non-zero only if a *control* failed (an anchor pair disagreeing
with the measured base, or a Q2 control TU that should mint labels and does
not). It is never non-zero because a prediction failed — the table is the
result.
"""

import os
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, os.path.join(REPO, "scripts"))
# Explicit-path imports: ten lane directories carry a `search.py`/`model.py` and
# a bare `import` resolves by `sys.path` order. These two are repo `scripts/`
# modules and are unique, but the habit is the point.
import gt_label_stride as G  # noqa: E402
from gt_dump import Obj  # noqa: E402

# ---------------------------------------------------------------------------
# Q1 — the leaf-loop stride.
#
# Every probe is a LEAF: no call, no saved GPR it does not spill for itself.
# `extra` is undefined for a leaf (it mints no `$M` of its own), so only
# `stride` is read. `leaf-none` is the base and must read 1 — that is
# `gt_label_stride.py`'s own `leaf-int` row, restated here so a run that
# disagrees with the shipped table is visible in this table rather than by
# cross-reference.
# ---------------------------------------------------------------------------
Q1 = [
    ("leaf-none", "int P(int a){ return a+1; }",
     "BASE: int leaf, no control flow. gt_label_stride's `leaf-int` = 1"),
    ("leaf-if", "int P(int a){ if (a) return 5; return a+1; }",
     "a forward-only branch in a leaf -- §4 says an `if` is +0"),

    # --- the three loop keywords, sentinel-tested (the §4 framed rows are
    #     +2 / +2 / +1 in this order) ---------------------------------------
    ("leaf-while", "int P(int a){ int r=0; while (a) { r=r+a; a=a-1; } return r; }",
     "while, sentinel -- §4's FRAMED row is +2"),
    ("leaf-dowhile", "int P(int a){ int r=0; do { r=r+a; a=a-1; } while (a); return r; }",
     "do/while -- §4's FRAMED row is +1, i.e. NOT the same as `while`"),
    ("leaf-for", "int P(int a){ int r=0; for (int i=0;i<a;i++) r=r+i; return r; }",
     "for, counted -- §4's FRAMED row is +2"),
    ("leaf-forever", "int P(int a){ int r=0; for (;;) { r=r+a; a=a-1; if (!a) break; } return r; }",
     "for(;;) + break -- one back edge, one exit, NO entry test (P8's risk cell)"),

    # --- the axes CFG_SHAPE.md §8.2 L2/L3/L4 leave open -------------------
    ("leaf-for-k", "int P(int a){ int r=0; for (int i=0;i<10;i++) r=r+a; return r; }",
     "COMPILE-TIME trip count -- unrolled? CTR? neither?"),
    ("leaf-for-stride", "int P(int a){ int r=0; for (int i=0;i<a;i+=3) r=r+i; return r; }",
     "stride 3 -- L3: a trip count needing (hi-lo+k-1)/k"),
    ("leaf-for-down", "int P(int a){ int r=0; for (int i=a;i>0;i--) r=r+i; return r; }",
     "counting DOWN -- the natural CTR shape"),
    ("leaf-for-break", "int P(int a){ int r=0; for (int i=0;i<a;i++){ r=r+i; if (r>100) break; } return r; }",
     "L2: a second exit. P6 says the CTR form cannot survive it"),
    ("leaf-for-cont", "int P(int a){ int r=0; for (int i=0;i<a;i++){ if (i==3) continue; r=r+i; } return r; }",
     "L4: a second back edge / the test as a join target"),
    ("leaf-for-live", "int P(int a){ int r=0; int i; for (i=0;i<a;i++) r=r+i; return r+i; }",
     "P7: the counter is READ AFTER the loop"),
    ("leaf-for2", "int P(int a){ int r=0; for (int i=0;i<a;i++) r=r+i;"
     " for (int j=0;j<a;j++) r=r+j; return r; }",
     "TWO sequential loops -- is the charge per loop?"),
    ("leaf-fornest", "int P(int a){ int r=0; for (int i=0;i<a;i++)"
     " for (int j=0;j<a;j++) r=r+j; return r; }",
     "nested -- §4's FRAMED row is +4"),
    ("leaf-goto-back", "int P(int a){ int r=0; top: r=r+a; a=a-1; if (a) goto top; return r; }",
     "an explicit BACKWARD goto -- a loop with no loop keyword"),

    # --- the two frontier shapes, reduced to a leaf probe -------------------
    ("leaf-ptrwalk", "int P(const char* s){ int r=0; for (const char* p=s; *p; p++) r=r+*p; return r; }",
     "Sort.cpp's induction shape without the modulo: a pointer walk to a sentinel"),
    ("leaf-idxload", "int P(const int* v,int n){ int r=0; for (int i=0;i<n;i++) r=r+v[i]; return r; }",
     "Primes.cpp's induction shape: an indexed load in the body"),
]

# ---------------------------------------------------------------------------
# Q2 — is a leaf loop's charge OBSERVABLE?
#
# Three TU shapes per probe body, deliberately not the Q1 construction:
#
#   leafonly   : the probe alone.                 predicted 0 labels
#   leafx3     : the probe and two more leaves.   predicted 0 labels
#   leaf+framed: the probe THEN a framed function. predicted >0 labels
#
# The third is the CONTROL and it is the shape the brief demands: it is the only
# one that can express a wrong `$M`, and the generated sweep (single-function
# TUs at /Ox) cannot produce it. If `leaf+framed` also read 0 the instrument
# would be measuring nothing and the run exits non-zero.
# ---------------------------------------------------------------------------
FRAMED_TAIL = "int gz(int);\nint z9(int a){ return gz(a)+7; }"

Q2_BODIES = [name_body[:2] for name_body in Q1]


def q2_src(shape, probe):
    leaves = ["int q1(int a){ return a*3; }", "int q2(int a){ return a-9; }"]
    if shape == "leafonly":
        return probe + "\n"
    if shape == "leafx3":
        return "\n".join([probe] + leaves) + "\n"
    if shape == "leaf_framed":
        return "\n".join([FRAMED_TAIL.split("\n")[0], probe,
                          FRAMED_TAIL.split("\n")[1]]) + "\n"
    raise AssertionError(shape)


def label_syms(o):
    """Every `$M`/`$T` short name in the obj, in symbol-table order."""
    out = []
    for s in o.symbols:
        n = s["name"]
        if (n.startswith("$M") or n.startswith("$T")) and n[2:].isdigit():
            out.append((n, s["sc"]))
    return out


def text_sections(o):
    return [i for i, s in enumerate(o.sections) if s["name"] == ".text"]


def back_edges(o):
    """Every backward intra-section branch word in every `.text`, as
    (section index, offset, word). A branch is backward when its own signed
    displacement is negative; op 16 (`bc`) and op 18 (`b`) are the only two
    forms `CFG_SHAPE.md` §3.1 records, and an external `b` never has a negative
    displacement large enough to be confused with one *and* carries a
    relocation, which is checked rather than assumed."""
    out = []
    for i, s in enumerate(o.sections):
        if s["name"] != ".text":
            continue
        raw = o.d[s["rawptr"]:s["rawptr"] + s["rawsize"]]
        relocs = set(int.from_bytes(o.d[s["relptr"] + 10 * r:s["relptr"] + 10 * r + 4], "little")
                     for r in range(s["nrel"]))
        for off in range(0, len(raw) - 3, 4):
            w = int.from_bytes(raw[off:off + 4], "big")
            op = w >> 26
            if op == 16:                        # bc
                bd = w & 0xFFFC
                if bd & 0x8000:
                    bd -= 0x10000
                if bd < 0 and off not in relocs:
                    out.append((i, off, w, bd, "bc"))
            elif op == 18:                      # b
                li = w & 0x03FFFFFC
                if li & 0x02000000:
                    li -= 0x04000000
                if li < 0 and off not in relocs:
                    out.append((i, off, w, li, "b"))
    return out


def ctr_ops(o):
    """`mtctr rS` (`0x7C0903A6 | rS<<21`) and `bdnz` (op 16, BO=16) counts."""
    mt = bd = 0
    for i, s in enumerate(o.sections):
        if s["name"] != ".text":
            continue
        raw = o.d[s["rawptr"]:s["rawptr"] + s["rawsize"]]
        for off in range(0, len(raw) - 3, 4):
            w = int.from_bytes(raw[off:off + 4], "big")
            if (w & 0xFC1FFFFF) == 0x7C0903A6:
                mt += 1
            if (w >> 26) == 16 and ((w >> 21) & 0x1F) == 16:
                bd += 1
    return mt, bd


def run_q1(mode, wd):
    print("== Q1  the LEAF-loop stride, seed-free, /O1 anchors ==")
    print("   construction: gt_label_stride.py (a0 . P . a1 . a2), P a LEAF so `extra` is undefined")
    print("   control      = the measured anchor base (5 under /Gy, 4 packed); a row whose")
    print("                  control is neither is an instrument failure, not a reading.")
    print()
    print("%-18s %7s %8s %8s  %s" % ("probe", "stride", "control", "minted", "note"))
    bad = 0
    rows = {}
    for name, probe, note in Q1:
        row = G.run(name, "", [], probe, note, mode, wd)
        if row is None or "error" in row:
            print("%-18s  FAILED %s" % (name, (row or {}).get("error", "capture")))
            bad += 1
            continue
        if row["control"] not in (4, 5):
            bad += 1
        if row["framed"]:
            note = "!! NOT A LEAF (it minted its own $M) -- " + note
        rows[name] = row
        print("%-18s %7d %8d %8d  %s"
              % (name, row["stride"], row["control"], row["minted"], note))
    base = rows.get("leaf-none", {}).get("stride")
    if base is not None:
        print()
        print("  surcharge = stride - stride(leaf-none) = stride - %d" % base)
        for name in rows:
            print("    %-18s %+d" % (name, rows[name]["stride"] - base))
    return bad, rows


def run_q2(mode, wd):
    print()
    print("== Q2  is the charge OBSERVABLE?  labels minted per TU shape ==")
    print("   leafonly / leafx3 : predicted 0 (P1).  leaf_framed : the CONTROL, predicted > 0.")
    print("   `back` = backward intra-section branch words; `ctr` = (mtctr, bdnz).")
    print()
    print("%-18s %10s %8s %6s %8s | %s"
          % ("probe", "shape", "labels", "back", "ctr", "the label names"))
    bad = 0
    ctrl_seen = 0
    rows = {}
    for name, probe in Q2_BODIES:
        for shape in ("leafonly", "leafx3", "leaf_framed"):
            o = G.capture(q2_src(shape, probe), mode, wd,
                          ("%s_%s" % (name, shape)).replace("-", "_"))
            if o is None:
                print("%-18s %10s  CAPTURE FAILED" % (name, shape))
                bad += 1
                continue
            ls = label_syms(o)
            be = back_edges(o)
            mt, bdz = ctr_ops(o)
            rows[(name, shape)] = (len(ls), len(be), mt, bdz)
            if shape == "leaf_framed" and len(ls) > 0:
                ctrl_seen += 1
            print("%-18s %10s %8d %6d %8s | %s"
                  % (name, shape, len(ls), len(be), "(%d,%d)" % (mt, bdz),
                     " ".join(n for n, _ in ls) or "-"))
    if ctrl_seen == 0:
        print("  !! CONTROL FAILED: no `leaf_framed` TU minted a label. The instrument")
        print("     cannot distinguish `unobservable` from `not looking`.")
        bad += 1
    else:
        print()
        print("  control: %d of %d `leaf_framed` TUs minted at least one label."
              % (ctrl_seen, len(Q2_BODIES)))
    zero = sum(1 for (n, s), v in rows.items()
               if s in ("leafonly", "leafx3") and v[0] == 0)
    tot = sum(1 for (n, s) in rows if s in ("leafonly", "leafx3"))
    withback = sum(1 for (n, s), v in rows.items()
                   if s in ("leafonly", "leafx3") and v[1] > 0)
    print("  P1: %d of %d leaf-only TUs minted ZERO labels (%d of them contain a"
          " backward branch)." % (zero, tot, withback))
    return bad, rows


def run_dis(mode, wd, want):
    for name, probe, note in Q1:
        if want and name not in want:
            continue
        o = G.capture(q2_src("leafonly", probe), mode, wd,
                      ("dis_" + name).replace("-", "_"))
        print("== %s == %s" % (name, note))
        print("   %s" % probe)
        if o is None:
            print("   CAPTURE FAILED")
            continue
        path = os.path.join(wd, "dis_%s.obj" % name.replace("-", "_"))
        open(path, "wb").write(o.d)
        subprocess.run([sys.executable, os.path.join(REPO, "scripts", "gt_dump.py"), path])
        print()


def main(argv):
    mode = "/O1 /GS- /c"
    if "--mode" in argv:
        i = argv.index("--mode")
        mode = argv[i + 1]
        del argv[i:i + 2]
    want = [a for a in argv[1:] if not a.startswith("--")]
    wd = tempfile.mkdtemp(prefix="wloop")
    print("mode: %s   workdir: %s" % (mode, wd))
    print()
    bad = 0
    if "--dis" in argv:
        run_dis(mode, wd, want)
        return 0
    if "--q2" not in argv:
        b, _ = run_q1(mode, wd)
        bad += b
    if "--q1" not in argv:
        b, _ = run_q2(mode, wd)
        bad += b
    print()
    print("controls failed: %d" % bad)
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
