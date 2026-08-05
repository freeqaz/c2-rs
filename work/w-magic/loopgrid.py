#!/usr/bin/env python3
"""loopgrid.py — does the constant-divisor map survive INSIDE A LOOP?

Lane **w-magic**. `w-divsplit` §8 measured the embedded-division population at
**4,649 of 4,674 `cflow-loop`** (99.5 %), **74.3 % `calls-1`**, and **99.8 %
carrying a pointer-difference dividend**. `kgrid.py` measures the lowering in a
straight-line leaf. That is the cheap place to measure it and the wrong place to
*claim* it: w-hash §5.1 found the `twi 6` placement differs between a
straight-line body and a loop with the same expression in it, and #763 found the
whole ptr-walk body differs between `/O1` and `/Ox`. **A map established outside
the loop and asserted inside it would be this project's twelfth refuted
placement rule.**

So this probe puts the *same* `k` inside the population's own shape — a
pointer-difference dividend, a loop, and (in half the cells) a call in the body —
and asks whether the division subsequence is the one `rule.py` generates.

Registers are NOT expected to match: inside a loop the dividend lives in a
different register and the allocator has more pressure. What is compared is the
**mnemonic sequence and the immediate fields**, which is what the map is a
statement about. That weaker comparison is stated here so nobody reads a green
row as a byte claim.

    work/w-magic/loopgrid.py
    work/w-magic/loopgrid.py --mode '/Ox /GS- /c'
"""

import os
import sys
import tempfile
from concurrent.futures import ThreadPoolExecutor

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, os.path.join(REPO, "scripts"))
sys.path.insert(0, HERE)
import gt_label_stride as G  # noqa: E402
import kgrid  # noqa: E402
import rule  # noqa: E402

# The workload's own divisors, plus the two boundaries the leaf grid found:
# 32768 (the ONLY k in the whole 32-bit axis whose materialization needs an
# `ori` in the interleaved position) and 100000 (`wide-lo`, contiguous).
K_LOOP = [20, 2, 24, 12, 6, 3, 100, 732, 4, 32768, 100000, 196608, 7]

# Four shapes. Each is the population's own, not a synthetic minimum:
#   ptrdiff  — `(p - q) / k` as an explicit int division over a struct pointer
#   ptrsub   — the C++ pointer difference itself, where `k` is sizeof(T) and the
#              division is emitted by the FRONT end, not written by the user
#   heap     — `(i - 1) / 2`, the binary-heap parent `w-divsplit` §9 names
#   call     — `ptrdiff` with a call in the loop body (74.3 % of the population)
SHAPES = {
    "ptrdiff": """
int g(int);
int P(char* p, char* q){ int t=0; while(p<q){ t += (int)(q-p)/%(k)d; p += 3; } return t; }
""",
    "heap": """
int P(int n){ int t=0; while(n>0){ n = (n-1)/%(k)d; t += n; } return t; }
""",
    "call": """
int g(int);
int P(char* p, char* q){ int t=0; while(p<q){ t += g((int)(q-p)/%(k)d); p += 3; } return t; }
""",
}


def words(o):
    out = []
    for s in o.sections:
        if s["name"] != ".text":
            continue
        raw = o.d[s["rawptr"]:s["rawptr"] + s["rawsize"]]
        for off in range(0, len(raw) - 3, 4):
            out.append(int.from_bytes(raw[off:off + 4], "big"))
    return out


def key(w):
    """(mnemonic, immediate-or-None) — the comparison unit. Register fields are
    deliberately dropped; see the module docstring."""
    m, _, f = kgrid.decode(w)
    if m in ("li", "lis", "mulli", "addi", "addis"):
        return (m, f["SIMM"])
    if m in ("ori", "oris"):
        return (m, f["UIMM"])
    if m == "srawi":
        return (m, f["SH"])
    if m == "rlwinm":
        return (m, (f["SH"], f["MB"], f["ME"]))
    return (m, None)


def expected(k, is_mod):
    """The map's answer for this `k`, as a (mnemonic, immediate) list with the
    trailing `blr` dropped — a loop body does not end in one."""
    b = rule.body(True, is_mod, k)
    return [key(w) for w in b[:-1]] if b else None


def contains(hay, needle):
    """Is `needle` a contiguous subsequence of `hay`? Returns the index or -1."""
    for i in range(len(hay) - len(needle) + 1):
        if hay[i:i + len(needle)] == needle:
            return i
    return -1


def scattered(hay, needle):
    """Is `needle` an ORDERED but possibly non-contiguous subsequence? #644 says
    a producer need not be contiguous, so a miss on `contains` and a hit here is
    the interesting middle answer, not a failure."""
    it = iter(hay)
    return all(any(x == y for y in it) for x in needle)


def main(argv):
    mode = "/O1 /GS- /c"
    if "--mode" in argv:
        i = argv.index("--mode")
        mode = argv[i + 1]
        del argv[i:i + 2]
    wd = tempfile.mkdtemp(prefix="wmagicloop")
    cells = [(sh, k) for sh in SHAPES for k in K_LOOP]
    print("mode: %s   shapes: %d   k: %d   cells: %d"
          % (mode, len(SHAPES), len(K_LOOP), len(cells)))
    print()

    def cap(c):
        sh, k = c
        src = SHAPES[sh] % {"k": k}
        return c, G.capture(src, mode, wd, "%s_%d" % (sh, k))

    with ThreadPoolExecutor(max_workers=8) as ex:
        res = list(ex.map(cap, cells))

    print("%-9s %8s %4s %-10s %s" % ("shape", "k", "wds", "verdict", "detail"))
    print("-" * 112)
    tot = exact = scat = absent = fail = 0
    for (sh, k), o in res:
        if o is None:
            print("%-9s %8d  CAPTURE FAILED" % (sh, k))
            fail += 1
            continue
        ws = words(o)
        got = [key(w) for w in ws]
        want = expected(k, False)
        tot += 1
        i = contains(got, want)
        if i >= 0:
            exact += 1
            v = "CONTIGUOUS@%d" % i
        elif scattered(got, want):
            scat += 1
            v = "SCATTERED"
        else:
            absent += 1
            v = "ABSENT"
        print("%-9s %8d %4d %-10s want[%s]"
              % (sh, k, len(ws), v,
                 " ".join(m if im is None else "%s(%s)" % (m, im)
                          for m, im in want)))
        if v in ("ABSENT", "SCATTERED"):
            print("%-9s %8s      got [%s]"
                  % ("", "",
                     " ".join(m if im is None else "%s(%s)" % (m, im)
                              for m, im in got)))
    print()
    print("cells %d · CONTIGUOUS %d · SCATTERED %d · ABSENT %d · capture-fail %d"
          % (tot, exact, scat, absent, fail))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
