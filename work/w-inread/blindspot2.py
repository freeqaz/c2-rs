#!/usr/bin/env python3
"""blindspot2.py — `work/w-emitp2/blindspot.py` re-run against the WIDENED
crate acceptance, so the blind spot it decomposed can be read forward.

The only difference from the original is one import: `crateacc` patches
`strictin._crate_verdict` before `blindspot` binds it.  `blindspot.py` itself is
not edited and not copied — prereg decline floor 3 says a shortfall is
decomposed by the same instrument, and an instrument that was rewritten for the
occasion is not the same instrument.

`blame()` still asks *which element byte would `read_elements` refuse first*, so
its rows are still upper bounds and the total is still exact for the union.

    usage: blindspot2.py <cacheidx.tsv> [jobs]
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
MAIN = os.environ.get("C2RS_LANEROOT", os.path.abspath(os.path.join(HERE, "..", "..")))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(MAIN, "work", "w-emitp2"))
import crateacc   # noqa: E402,F401  — the patch; must precede `blindspot`
import blindspot  # noqa: E402

assert blindspot.si._crate_verdict is crateacc.crate_verdict_widened, \
    "the widened acceptance did not reach blindspot — the patch order is wrong"


def blame_widened(el):
    """`blindspot.blame`, with the four kinds this lane taught the reader
    removed from the refusal list. Everything else is untouched."""
    for k, a, w in el:
        if k in (blindspot.SYM, 0x03, 0x08):
            continue
        if k != 0x01:
            return "element-tag-%02x" % k
        if a == 0x05:
            return "scalar type 05 (floating point)"
        if a not in crateacc.SCALAR_TYPES:
            return "scalar type %02x" % a
        if a in crateacc.PTR_TYPES:
            if w != crateacc.PTR_WIDTH:
                return "pointer width %d" % w
        elif w not in crateacc.INT_WIDTHS:
            return "scalar width %d" % w
    return "NONE"


blindspot.blame = blame_widened

if __name__ == "__main__":
    blindspot.main()
