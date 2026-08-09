#!/usr/bin/env python3
"""w-memfit GRID-G — the IL carries TWO alignment hints.  WHICH ONE DIVIDES?

w-memcpy §2 recorded, off `?mmioGetInfo`'s own IL and on no board row, that
the memcpy argument region carries **two** alignment hints (`01` and `04`)
ahead of the size literal.  wb-memcpy §5.3 identified them positionally, at
`01` / `04` / `08` for `char*` / `int*` / `double*`, and `work/w-memfit/hint.py`
reproduces both bytes at `.ex` offsets 2733 and 2742 and adds two values
neither lane had: `01` for a `#pragma pack(1)` struct of doubles and `10` for
a `__declspec(align(16))` one.

**Every cell in every grid so far gives the two hints the SAME value**, because
every cell's two operands have the same pointee type.  So none of the 668 cells
graded to this point can say whether the divisor is the destination's hint, the
source's hint, the smaller of the two or the larger.  Four rules, one number,
and a port has to pick one.  This grid picks it.

  G-DST   divisor = min(8, hint of the DESTINATION operand)
  G-SRC   divisor = min(8, hint of the SOURCE operand)
  G-MIN   divisor = min of the two clamped hints
  G-MAX   divisor = max of the two clamped hints

`G-DST` and `G-MAX` agree on every cell where the destination is the more
aligned operand and disagree nowhere else, so the grid crosses BOTH orders of
every pair — that is what makes the four separable rather than three.

The `a32` family is the clamp's own control: `__declspec(align(32))` writes
`20` into the hint byte, and every rival here clamps at 8, so a family that
came back `inline` above `size/8 > 5` would refute the clamp rather than the
choice of operand.

Usage:  gridg.py gen <outdir> | run <outdir> <root> | score <outdir>
"""

import hashlib
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gridf import Obj_run_helper  # noqa: E402  (shared compile+read-back)

T = 5

# (tag, C spelling, the hint byte measured by work/w-memfit/hint.py, clamped)
TYPES = [("c", "char", 1), ("i", "int", 4), ("d", "double", 8),
         ("a32", "A32", 8)]

# ordered pairs (dst, src); both orders of every mixed pair, plus the a32
# control against char.
PAIRS = [("d", "c"), ("c", "d"), ("d", "i"), ("i", "d"),
         ("i", "c"), ("c", "i"), ("a32", "c"), ("c", "a32")]

SIZES = [8, 16, 20, 24, 32, 40, 48]

HDR = """// w-memfit GRID-G cell %s
// %s
__declspec(align(32)) struct A32 { char c[32]; };
extern "C" void *memcpy(void *, const void *, unsigned int);
"""

RIVALS = ["G-DST", "G-SRC", "G-MIN", "G-MAX"]


def divisors(ad, asrc):
    return {"G-DST": ad, "G-SRC": asrc,
            "G-MIN": min(ad, asrc), "G-MAX": max(ad, asrc)}


def verdict_for(divisor, size):
    if size == 0:
        return "none"
    return "inline" if size // max(1, divisor) <= T else "call"


def build_cells():
    align = {t: a for t, _c, a in TYPES}
    ctype = {t: c for t, c, _a in TYPES}
    cells = []
    for dst, src in PAIRS:
        for size in SIZES:
            name = "g_%s_%s_n%d" % (dst, src, size)
            meta = dict(dst=dst, src=src, size=size)
            divs = divisors(align[dst], align[src])
            body = ("void f(%s *d, const %s *s) { memcpy(d, s, %d); }\n"
                    % (ctype[dst], ctype[src], size))
            cells.append(dict(
                name=name, dst=dst, src=src, size=size, div=divs,
                pred={r: verdict_for(d, size) for r, d in divs.items()},
                src_text=HDR % (name, json.dumps(meta, sort_keys=True)) + body))
    return cells


def gen(outdir):
    cells = build_cells()
    os.makedirs(outdir, exist_ok=True)
    worst = None
    for i, a in enumerate(RIVALS):
        for b in RIVALS[i + 1:]:
            d = sum(1 for c in cells if c["pred"][a] != c["pred"][b])
            print("   %-6s vs %-6s separated on %2d cells" % (a, b, d))
            worst = d if worst is None else min(worst, d)
    assert worst >= 4, "some rival pair is separated on only %d cells" % worst
    # Both orders of every mixed pair must be present, or G-DST and G-MAX
    # cannot be told apart and the grid would report a rule it cannot see.
    for a, b in PAIRS:
        assert (b, a) in PAIRS, "pair %s/%s is not crossed both ways" % (a, b)
    h = hashlib.sha256()
    for c in sorted(cells, key=lambda c: c["name"]):
        open(os.path.join(outdir, c["name"] + ".cpp"), "w").write(c["src_text"])
        h.update(c["src_text"].encode())
    man = [{k: v for k, v in c.items() if k != "src_text"} for c in cells]
    json.dump(man, open(os.path.join(outdir, "manifest.json"), "w"), indent=1)
    print("cells        %d" % len(cells))
    print("pairs        %s" % PAIRS)
    print("rivals       %s" % RIVALS)
    print("min pairwise separation %d" % worst)
    print("sha256       %s" % h.hexdigest())


def score(outdir):
    man = json.load(open(os.path.join(outdir, "manifest.json")))
    mea = {r["name"]: r for r in
           json.load(open(os.path.join(outdir, "measured.json")))}
    print("== GRID-G: %d cells ==" % len(man))
    for r in RIVALS:
        h = sum(1 for c in man if c["pred"][r] == mea[c["name"]]["verdict"])
        print("   %-6s %2d/%d" % (r, h, len(man)))
    print()
    print("   dst src size  measured   %s" % "  ".join(RIVALS))
    for c in man:
        v = mea[c["name"]]["verdict"]
        marks = "  ".join("%-7s" % ("%s%s" % (c["pred"][r],
                                              "" if c["pred"][r] == v else "*"))
                          for r in RIVALS)
        print("   %-3s %-3s %4d  %-8s  %s  [%dB]"
              % (c["dst"], c["src"], c["size"], v, marks,
                 mea[c["name"]]["nbytes"]))
    print("\n   * = this rival is WRONG on that cell")


if __name__ == "__main__":
    if sys.argv[1] == "gen":
        gen(sys.argv[2])
    elif sys.argv[1] == "run":
        Obj_run_helper(sys.argv[2], sys.argv[3])
    else:
        score(sys.argv[2])
