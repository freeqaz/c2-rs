#!/usr/bin/env python3
"""splice0.py — SPLICE-0: is c2's body for the CALLER c2's body for the CALLEE?

Lane w-seq measurement tooling. **Read-only with respect to `crates/`.**

    splice0.py <scan.jsonl>

SPLICE-P concatenates the port's argument setup ahead of the callee's emitted
body. SPLICE-0 asks the degenerate form — *no setup at all* — of every shape
whose port body names exactly one callee, `seq` and `framed` included, because
their port bodies carry a frame the concatenation rule cannot subtract and the
question "did c2 emit the callee's own bytes here" is still well posed.

When it fails, the **first disagreeing word** is the diagnosis: a destination
register field says inlining renamed a register; a displacement field says it
folded the caller's pointer arithmetic into the callee's first memory access.
Both are printed with counts.
"""

import collections
import json
import sys


def main():
    c = collections.Counter()
    for line in open(sys.argv[1]):
        r = json.loads(line)
        if r.get("record") == "provenance" or "src" not in r:
            continue
        for k, v in (r.get("emit") or {}).items():
            if k.startswith("fnbyte-splice0|"):
                c[k.split("|", 1)[1]] += v

    byshape = collections.Counter()
    for k, n in c.items():
        byshape[(k.split("|")[0], k.split("|")[1])] += n
    den = collections.Counter()
    for (shape, verdict), n in byshape.items():
        den[shape] += n
    print("=== SPLICE-0 by shape === (denominator = single-callee differs)")
    for (shape, verdict), n in sorted(byshape.items()):
        print("  %-7s %-8s %5d of %5d  (%5.1f%%)"
              % (shape, verdict, n, den[shape], 100.0 * n / den[shape]))
    print("  TOTAL graded: %d" % sum(den.values()))

    print("\n=== SPLICE-0 FAILURE WITNESSES, top 30 ===")
    tot = sum(n for k, n in c.items() if "differs" in k)
    for k, n in sorted(
        ((k, n) for k, n in c.items() if "differs" in k), key=lambda x: -x[1]
    )[:30]:
        print("  %5d  %5.1f%%  %s" % (n, 100.0 * n / tot, k))
    if tot == 0:
        print("  (none)")


if __name__ == "__main__":
    main()
