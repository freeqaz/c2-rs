#!/usr/bin/env python3
"""rivals.py — the POST-HOC scoring, kept separate from the graded column so it
cannot be mistaken for a result.

`gridz.py --grade` scores the SEVEN rules that were frozen in `pred.tsv` before
a cell was compiled. This file scores three more that could only be written
AFTER the grade, reads the two frontiers GRID Z actually measured, and states in
its own output that none of them has any standing.

    H-2Y   `d = 1` iff the STORE designator's root token is a temp BIND head
           AND it differs from the VALUE expression's root token.
           (H-2X with the asymmetry GRID Z measured. Still wrong.)

    H-2Z   H-2Y with the bonus suppressed at `ru = 1`.
           (0 wrong on GRID Z — and it has THREE conjuncts, two of them read
           off this grid. `RULE W2` was 388 of 388 and `RULE BIND` 33 of 33.
           NOT PROPOSED.)

    cu<=ru+2-on-SELF-2B   board #1221's clause, scored on the class it was
           said to fit. GRID Z reaches `(1,3)` and `(1,4)`, which no lane's
           `SELF-2B` cells did, and it is WRONG there.

**COMPILES NOTHING.** It reads `pred.tsv` for the cell specs and `grade.out` for
the objs, both committed.
"""

import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from gridz import cells, FAMS                                   # noqa: E402

GRADE = os.path.join(HERE, "grade.out")

# family -> (store root is a BIND, roots DIFFER) — read off `roots.out`, which
# DECODES the `.ex`; not off the source text and not off the obj.
FAM_IL = {
    "Z1": (False, False),
    "Z2": (True, False),
    "Z3": (True, True),
    "Z4": (True, True),
    "Z5": (False, True),
    "Z6": (True, True),
}


def objs():
    """The graded column, parsed out of the committed `grade.out`."""
    out = {}
    rx = re.compile(r"^\s+(\S+)\s+(\S+)\s+(near|far)\s+(in|CONTROL)\s+"
                    r"(\S+)\s+(prod|const|OOR.*|compile-failed)\s*(\*\*MISS\*\*|control)?\s*$")
    for line in open(GRADE):
        m = rx.match(line.rstrip("\n"))
        if m:
            out[m.group(1)] = m.group(6)
    return out


def h_2y(c):
    b, d = FAM_IL[c.fam]
    return "prod" if c.cu <= c.ru + 1 + (1 if (b and d) else 0) else "const"


def h_2z(c):
    b, d = FAM_IL[c.fam]
    bonus = 1 if (b and d and c.ru >= 2) else 0
    return "prod" if c.cu <= c.ru + 1 + bonus else "const"


def cu_le_ru2(c):
    return "prod" if c.cu <= c.ru + 2 else "const"


POST = [("H-2Y  (asymmetric)", h_2y),
        ("H-2Z  (+ ru>=2 guard)", h_2z),
        ("cu<=ru+2  (#1221)", cu_le_ru2)]


def main():
    if not os.path.exists(GRADE):
        print("  FAIL: no grade.out")
        return 1
    o = objs()
    inn = [c for c in cells() if c.in_domain and o.get(c.name) in ("prod", "const")]
    sb = [c for c in inn if c.fam in ("Z3", "Z4", "Z6")]
    print("  POST-HOC — written AFTER the grade.  NONE OF THIS HAS STANDING.")
    print("  in-domain graded cells: %d   (of which SELF-2B-like: %d)"
          % (len(inn), len(sb)))
    print("\n  rule                       right  WRONG   wrong cells")
    print("  " + "-" * 78)
    for name, fn in POST:
        bad = [c.name for c in inn if fn(c) != o[c.name]]
        print("  %-26s %5d %6d   %s"
              % (name, len(inn) - len(bad), len(bad),
                 ", ".join(bad) if bad else "-"))
    print("\n  #1221's clause scored on the SELF-2B-like families ALONE")
    bad = [c.name for c in sb if cu_le_ru2(c) != o[c.name]]
    print("  cu<=ru+2 on Z3/Z4/Z6:  right %d  WRONG %d   %s"
          % (len(sb) - len(bad), len(bad), ", ".join(bad) or "-"))

    print("\n  THE TWO FRONTIERS GRID Z MEASURED")
    pts = sorted({(c.ru, c.cu) for c in inn})
    print("      %-6s %s" % ("", "  ".join("%d/%d" % p for p in pts)))
    for f in FAMS:
        row = []
        for ru, cu in pts:
            v = o.get("%s-r%dk%d" % (f, ru, cu), "?")
            row.append({"prod": "P", "const": "c"}.get(v, "?"))
        print("      %-6s %s" % (f, "    ".join(row)))
    print("""
      A = Z1 Z2 Z5 :  prod iff cu <= ru+1
      B = Z3 Z4 Z6 :  prod iff cu <= ru+2  at ru in {2,3}
                                cu <= 2    at ru = 1

  **`cu <= ru+2` is REFUTED on fresh `SELF-2B` cells.** It fits all 22 on
  record because no lane's `SELF-2B` cells reach `(1,3)`; GRID Z does, and
  it is `const` there in all three B families, near and far.

  What GRID Z CANNOT say: whether the `ru = 1` collapse is a `ru >= 2` guard,
  a `cu <= 2*ru` cap, or a requirement that the address be live across at
  least two of its OWN stores. Those three agree everywhere this grid
  reaches. `ru = 1` at `cu = 2`, and `ru` 4-5 at `cu = ru+2` and `cu = 2*ru`,
  are what separates them, and no lane has any of those cells.""")
    return 0


if __name__ == "__main__":
    sys.exit(main())
