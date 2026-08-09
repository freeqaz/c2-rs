#!/usr/bin/env python3
"""w-memfit — read the ALIGNMENT HINT BYTE out of the IL, per pointee type.

GRID-F establishes, on 44 cells graded by real `c2.dll`, that the divisor is
`min(8, alignof(pointee))`: F-CLAMP 44/44, F-TYPE 39/44 (it misses the five
`__declspec(align(16))` cells in the 48..80 band), F-ELEM 36/44 and F-PROV
34/44.  That is a statement about the OBSERVABLE.  It leaves one fork open:

  (a) `c1xx` writes 8 into the hint byte for a 16-aligned pointee, and
      wb-memcpy's `align = max(1, BYTE[node+0x38])` is exactly right; or
  (b) `c1xx` writes 16 and `c2` clamps it somewhere the reading did not name.

The two differ in the IL, which is a file this repo can capture.  wb §5.3's
method: capture cells that differ only in the pointee type, diff the `.ex`,
and the hint bytes are the positions that move.  Three captures there differed
in exactly three bytes, two of them taking the values 01 / 04 / 08.

Usage:  hint.py <workdir>
"""

import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(ROOT, os.pardir, os.pardir))

# (tag, pointee spelling, the divisor GRID-F measured for it)
CASES = [
    ("c",   "char",   1),
    ("i",   "int",    4),
    ("d",   "double", 8),
    ("p1",  "P1",     1),   # #pragma pack(1) struct { double a[8]; }
    ("a16", "A16",    8),   # __declspec(align(16)) struct { char c[16]; }
]

SRC = """// w-memfit hint probe %s
#pragma pack(push, 1)
struct P1 { double a[8]; };
#pragma pack(pop)
__declspec(align(16)) struct A16 { char c[16]; };
extern "C" void *memcpy(void *, const void *, unsigned int);
void f(%s *d, const %s *s) { memcpy(d, s, 96); }
"""


def main():
    out = os.path.abspath(sys.argv[1])
    os.makedirs(out, exist_ok=True)
    flags = os.path.join(REPO, "work/dc3-workload/flags.txt")
    c2rs = os.path.join(REPO, "target/release/c2rs")
    ex = {}
    for tag, ty, _div in CASES:
        name = "h_%s.cpp" % tag
        open(os.path.join(out, name), "w").write(SRC % (tag, ty, ty))
        ild = os.path.join(out, "il_" + tag)
        os.makedirs(ild, exist_ok=True)
        subprocess.run([c2rs, "capture", name, "--keep-il", ild,
                        "--flags-file", flags, "--cwd", out],
                       capture_output=True, text=True, cwd=out, check=False)
        cand = [f for f in os.listdir(ild) if f.endswith(".ex")]
        if not cand:
            print("   %-4s NO .ex captured (%s)" % (tag, os.listdir(ild)[:4]))
            continue
        ex[tag] = open(os.path.join(ild, cand[0]), "rb").read()
        print("   %-4s .ex %d bytes" % (tag, len(ex[tag])))

    # All five sources are byte-identical except for the pointee spelling, so
    # every differing .ex position is a consequence of the type.  The hint is
    # the position whose VALUE tracks the divisor GRID-F measured.
    print()
    base = "d"
    if base not in ex:
        return 1
    n = min(len(v) for v in ex.values())
    diffs = [i for i in range(n)
             if len({ex[t][i] for t in ex}) > 1]
    print("   .ex positions that differ across the five pointee types: %d"
          % len(diffs))
    print("   %-6s %s" % ("off", "  ".join("%-4s" % t for t in ex)))
    for i in diffs[:40]:
        print("   %-6d %s" % (i, "  ".join("%-4s" % ("%02x" % ex[t][i])
                                           for t in ex)))
    print()
    print("   GRID-F's measured divisor, for comparison:")
    print("   %-6s %s" % ("div", "  ".join("%-4d" % d for t, _y, d in CASES
                                           if t in ex)))
    return 0


if __name__ == "__main__":
    sys.exit(main())
