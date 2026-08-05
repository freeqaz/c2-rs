#!/usr/bin/env python3
"""gapgrid.py — the use-count HOLDOUT for the mixed-kind store run.

`allocgrid.py` produced two cells that bracket a crossover:

    diff-reg1-const2   addi 1 use, li 2 uses  ->  addi r11   (clause 1 refuted)
    diff-reg1-const3   addi 1 use, li 3 uses  ->  li   r11   (clause 2 refuted)

and the rule that fits BOTH is *"a constant needs a use-count advantage of >= 2
to outrank a register-derived producer"*.  That is a two-cell fit on one side of
one threshold, which is exactly how **P3** died (`floor((N-1)/2)` fit three
published cells and failed at N = 5).

THE PREDICTION IS REGISTERED HERE, IN THE FILE THAT TESTS IT, BEFORE THE RUN:

    reg gets r11  <=>  const_uses - reg_uses <= 1

Scored over the full (reg, const) grid for 1..4 x 1..4.  **Only 2 of those 16
cells were used to form the rule**; the other 14 are holdout.  A single miss
refutes it, and a miss is what this file exists to be able to report.

Anchor control: `g1x1` must reproduce `addi r11 / li r10`.
"""

import os
import subprocess
import sys
import concurrent.futures as cf

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
REFOBJ = os.path.join(ROOT, "work", "w-frame", "refobj.sh")
DUMP = os.path.join(ROOT, "scripts", "gt_dump.py")


def _sib(name):
    d = ROOT
    while d != os.path.dirname(d):
        cand = os.path.join(os.path.dirname(d), name)
        if os.path.isdir(cand):
            return cand
        d = os.path.dirname(d)
    return None


DC3 = os.environ.get("C2RS_DC3") or _sib("dc3-decomp")

# 12 int fields at 0..44, then the inner L at 48.  Wide enough that no cell
# runs out of distinct store targets.
HEAD = """\
struct L%(t)s { int a; int b; int c; int d; };
struct S%(t)s {
    int f0; int f1; int f2; int f3; int f4; int f5;
    int f6; int f7; int f8; int f9; int fa; int fb;
    L%(t)s inner;
};
void g%(t)s(S%(t)s* s, int u, int v) {
    L%(t)s& q = s->inner;
%(body)s
}
"""

# The register-derived producer `&q` is consumed as a STORED VALUE (never as the
# base — c2 folds `&q + k` into `s + 48 + k`), which is exactly how `xboxheap`
# consumes its `addi`.  The constant is consumed as a stored value too, so the
# two differ ONLY in kind.
REG_SLOTS = ["q.a", "q.b", "q.c", "q.d"]
CONST_SLOTS = ["s->f0", "s->f1", "s->f2", "s->f3"]


def body(reg_uses, const_uses):
    out = []
    for i in range(const_uses):
        out.append("    %s = 7;" % CONST_SLOTS[i])
    for i in range(reg_uses):
        out.append("    %s = (int)&q;" % REG_SLOTS[i])
    return "\n".join(out)


def build():
    c = {}
    for r in range(1, 5):
        for k in range(1, 5):
            name = "g%dx%d" % (r, k)
            t = name
            c[name] = HEAD % dict(t=t, body=body(r, k))
    return c


def words(obj):
    out = subprocess.run([sys.executable, DUMP, obj, "--text-only"],
                         capture_output=True, text=True).stdout
    res = []
    for line in out.splitlines():
        p = line.split()
        if len(p) >= 3 and len(p[1]) == 8:
            res.append(" ".join(p[2:]).split(";")[0].strip())
    return res


def run(a):
    name, src, out = a
    cpp = os.path.join(out, name + ".cpp")
    open(cpp, "w").write(src)
    obj = os.path.join(out, name + ".obj")
    r = subprocess.run([REFOBJ, os.path.relpath(cpp, DC3), obj],
                       capture_output=True, text=True,
                       env=dict(os.environ, C2RS_DC3=DC3))
    if r.returncode != 0 or not os.path.exists(obj):
        return name, None
    return name, words(obj)


def reg_of(dis):
    """`addi 11, 3, 48` -> 11 ; `li 10, 7` -> 10."""
    return int(dis.split()[1].rstrip(","))


FIT_CELLS = {"g1x2", "g1x3"}   # the two the rule was formed on


def main():
    out = os.path.join(HERE, "gapgrid")
    os.makedirs(out, exist_ok=True)
    cells = build()
    with cf.ThreadPoolExecutor(max_workers=8) as ex:
        res = dict(ex.map(run, [(n, s, out) for n, s in sorted(cells.items())]))

    print("  cell   reg const | addi  li  | winner   predicted  | verdict  partition")
    hit = miss = skip = 0
    for r in range(1, 5):
        for k in range(1, 5):
            name = "g%dx%d" % (r, k)
            w = res.get(name)
            if not w:
                print("  %-6s FAIL" % name)
                continue
            addi = [d for d in w if d.startswith("addi") and ", 3," in d]
            li = [d for d in w if d.startswith("li ")]
            if len(addi) != 1 or len(li) != 1:
                print("  %-6s  %d    %d   | OUT OF REGIME: addi=%d li=%d  (%s)"
                      % (name, r, k, len(addi), len(li), " ".join(w[:6])))
                skip += 1
                continue
            ra, rl = reg_of(addi[0]), reg_of(li[0])
            winner = "reg" if ra > rl else "const"
            pred = "reg" if (k - r) <= 1 else "const"
            ok = winner == pred
            hit += ok
            miss += not ok
            print("  %-6s  %d    %d   | r%-3d r%-3d | %-7s %-10s | %-8s %s"
                  % (name, r, k, ra, rl, winner, pred,
                     "HIT" if ok else "**MISS**",
                     "fit" if name in FIT_CELLS else "HOLDOUT"))
    ho = [n for n in res if n not in FIT_CELLS]
    print("\n  %d hit, %d MISS, %d out of regime, of %d cells "
          "(%d holdout, %d fit)" % (hit, miss, skip, len(res), len(ho), len(FIT_CELLS)))
    print("  RULE: reg takes r11  <=>  const_uses - reg_uses <= 1")
    if miss:
        print("  ==> REFUTED on %d cell(s). The bracket stands; the rule does not." % miss)
    else:
        print("  ==> HOLDS on every graded cell. Still a THRESHOLD rule over one "
              "axis at 1..4 — not licensed outside it.")


if __name__ == "__main__":
    main()
