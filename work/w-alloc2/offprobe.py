#!/usr/bin/env python3
"""offprobe.py — the displacement confound, isolated.

`freshgrid.py` and `opgrid.py` disagree on ONE cell shape and the disagreement
is real, not a grader bug:

    freshgrid F4-shift-r2k1   li 11,7 ; slwi 10,4,3 ; stw 11,0 ; stw 10,64 ; stw 10,68
    opgrid    H-shift-2v1     slwi 11,4,3 ; li 10,7 ; stw 11,4 ; stw 11,8 ; stw 10,0

Same source order (the constant first), same use counts (constant 1, shift 2),
same formals, same producer spelling.  **Both the SCHEDULE and the ALLOCATION
differ.**  The only thing that differs in the input is the displacement of the
shift's two stores: 64/68 against 4/8.

If displacement moves the allocation, it is a confound in w-next's 24 fitted
cells too — those put the constant at `s->f0`(0) and the register-derived value
at `q.*`, and `q` sits at 48 in `gapgrid.py`.  So this probe is a check on the
inherited grid as much as on mine.

One axis, everything else pinned: constant 1 use at offset 0, a 2-use producer
at (`lo`, `lo+4`) for lo in 4, 8, 16, 32, 44, 64, 128.  Both the producer's
register AND the emitted order are printed, because the two grids differ in
both and reporting only one would hide half the effect.

SHIPS NOTHING.
"""

import os
import re
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

# A flat array of ints, so a store offset is chosen by INDEX and nothing else
# about the type changes with it.
SRC = """\
struct S%(t)s { int f[64]; };
void g%(t)s(S%(t)s* s, int u, int v) {
%(body)s
}
"""

CONST = re.compile(r"^li\s+(\d+),\s*7$")
PRODS = {
    "shift": (re.compile(r"^(?:slwi|rlwinm)\s+(\d+),\s*4,"), "u << 3"),
    "add": (re.compile(r"^add\s+(\d+),\s*4,\s*5$"), "u + v"),
}
OFFS = [4, 8, 16, 32, 44, 64, 128, 252]


def dis(obj):
    out = subprocess.run([sys.executable, DUMP, obj, "--text-only"],
                         capture_output=True, text=True).stdout
    res = []
    for line in out.splitlines():
        p = line.split()
        if len(p) >= 3 and len(p[1]) == 8:
            res.append(" ".join(p[2:]).split(";")[0].strip())
    return res


def run_cell(a):
    name, body, out = a
    t = name.replace("-", "_")
    cpp = os.path.join(out, name + ".cpp")
    open(cpp, "w").write(SRC % dict(t=t, body=body))
    obj = os.path.join(out, name + ".obj")
    r = subprocess.run([REFOBJ, os.path.relpath(cpp, DC3), obj],
                       capture_output=True, text=True,
                       env=dict(os.environ, C2RS_DC3=DC3))
    if r.returncode != 0 or not os.path.exists(obj):
        return name, None
    return name, dis(obj)


def one(words, rx):
    h = {int(m.group(1)) for m in (rx.match(w) for w in words) if m}
    return h.pop() if len(h) == 1 else None


def main():
    jobs = 8
    if "--jobs" in sys.argv:
        jobs = int(sys.argv[sys.argv.index("--jobs") + 1])
    out = os.path.join(HERE, "offprobe")
    os.makedirs(out, exist_ok=True)

    cells = {}
    for tag, (_, expr) in sorted(PRODS.items()):
        for lo in OFFS:
            i = lo // 4
            cells["O-%s-%d" % (tag, lo)] = (
                "    s->f[0] = 7;\n"
                "    s->f[%d] = %s;\n    s->f[%d] = %s;" % (i, expr, i + 1, expr))

    with cf.ThreadPoolExecutor(max_workers=jobs) as ex:
        res = dict(ex.map(run_cell,
                          [(n, b, out) for n, b in sorted(cells.items())]))

    print("  %-16s | %-7s %-7s | %-6s | %s"
          % ("cell", "prod", "const", "winner", "emitted order"))
    print("  " + "-" * 88)
    graded = oor = 0
    seen = {}
    for name in sorted(res, key=lambda n: (n.split("-")[1], int(n.split("-")[2]))):
        w = res[name]
        tag = name.split("-")[1]
        if w is None:
            print("  %-16s | COMPILE FAILED" % name)
            continue
        pr, cr = one(w, PRODS[tag][0]), one(w, CONST)
        if pr is None or cr is None:
            print("  %-16s | OUT OF REGIME" % name)
            oor += 1
            continue
        graded += 1
        win = "prod" if pr > cr else "const"
        seen.setdefault(tag, []).append((int(name.split("-")[2]), win))
        order = " ".join(x.split()[0] for x in w if not x.startswith("blr"))
        print("  %-16s | r%-6d r%-6d | %-6s | %s" % (name, pr, cr, win, order))

    print("\n  GRADED %d of %d | out-of-regime %d" % (graded, len(cells), oor))
    print("\n  DOES DISPLACEMENT MOVE THE ALLOCATION?")
    for tag in sorted(seen):
        wins = seen[tag]
        vals = {w for _, w in wins}
        flip = [o for o, w in wins]
        print("    %-6s %s  ->  %s" % (
            tag, " ".join("%d:%s" % (o, w) for o, w in wins),
            "CONSTANT across displacement" if len(vals) == 1
            else "**IT MOVES** — displacement is a live axis (offsets %s)"
                 % (flip,)))


if __name__ == "__main__":
    main()
