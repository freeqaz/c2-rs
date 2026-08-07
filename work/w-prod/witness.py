#!/usr/bin/env python3
"""witness.py — the two-cell witness, and WHERE the carrier runs out.

`roots.py` decodes `P6` (`TWOBIND`) and `P7` (`CHAINBIND`) to what looks like
the same row: store root a BIND, value root a different BIND, both offset lists
equal. **And c2 answers differently** — `P6` is `prod` at all four deciding
points, `P7` is `const` at all nine.

If that is right it is worth more than the grade, because it says the carrier
this lane just built — `(root token, is-a-bind, literal list)` of both sides,
which is what `w-self2b` named and what nine dead keys were missing — is
**still not enough**, and it names what is missing rather than leaving it as a
residual.

So this prints, for both cells, the FULL decoded bind table (`tok -> (base
token, offset literals)`) beside the two roots, and asserts the comparison
rather than eyeballing it.

Reads the IL and nothing else — no obj, no disassembly, no register.
SHIPS NOTHING.
"""

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, HERE)
sys.path.insert(0, os.path.join(ROOT, "work", "w-ilx"))
import exdec                                                    # noqa: E402
from gridp import DC3, cells                                    # noqa: E402
from roots import capture                                       # noqa: E402

PAIR = ["P6-r2k4", "P7-r2k4"]
OBJ = {"P6-r2k4": "prod", "P7-r2k4": "const"}


def decode(c):
    st = capture(c.source())
    ex = st.get(".ex")
    if ex is None:
        return None
    binds, assigns = exdec.decode_body(ex)
    prod = [a for a in assigns if a["v"][0] == "addr-load"]
    if not prod:
        return None
    a = prod[0]
    return {
        "binds": binds,
        "lvalue": (a["l"][0], a["l"][0] in binds, list(a["l"][1])),
        "value": (a["v"][1], a["v"][1] in binds, list(a["v"][2])),
    }


def main():
    by = {c.name: c for c in cells()}
    got = {}
    for name in PAIR:
        d = decode(by[name])
        if d is None:
            print("  %-10s DID NOT DECODE — a counter, not a verdict" % name)
            return 1
        got[name] = d

    print("  THE WITNESS PAIR — same (ru,cu) = (2,4), same class of spelling,")
    print("  and c2 takes DIFFERENT registers.\n")
    for name in PAIR:
        c, d = by[name], got[name]
        print("  %s  (%s)   obj: %s" % (name, c.klass, OBJ[name]))
        for line in c.source().split("\n"):
            if "&" in line or "F&" in line or "F*" in line:
                print("      %s" % line.strip())
        lt, lb, ll = d["lvalue"]
        vt, vb, vl = d["value"]
        print("      CARRIER  lvalue tok 0x%04x %-6s %s" %
              (lt, "BIND" if lb else "formal", ll))
        print("               value  tok 0x%04x %-6s %s" %
              (vt, "BIND" if vb else "formal", vl))
        print("      BIND TABLE (tok -> base tok, offset literals)")
        for t in sorted(d["binds"]):
            base, lits = d["binds"][t]
            print("               0x%04x -> base 0x%04x  %s" % (t, base, lits))
        print()

    a, b = got[PAIR[0]], got[PAIR[1]]
    same_carrier = (a["lvalue"] == b["lvalue"] and a["value"] == b["value"])
    same_binds = (a["binds"] == b["binds"])

    print("  THE CARRIER'S OWN FIELDS, compared:")
    print("      lvalue root equal: %s" % (a["lvalue"] == b["lvalue"]))
    print("      value  root equal: %s" % (a["value"] == b["value"]))
    print("      => the carrier this lane built is %s on this pair"
          % ("IDENTICAL" if same_carrier else "different"))
    print("      bind TABLE equal:  %s" % same_binds)

    if same_carrier and not same_binds:
        # name the element that differs
        diffs = []
        for t in sorted(set(a["binds"]) | set(b["binds"])):
            if a["binds"].get(t) != b["binds"].get(t):
                diffs.append((t, a["binds"].get(t), b["binds"].get(t)))
        print("""
  THE RESULT, and it is the useful half of this lane:

    **The carrier is NOT ENOUGH, and here is the two-cell proof.** `P6` and
    `P7` decode to the SAME `(root token, is-a-bind, literal list)` on BOTH
    sides — which is exactly the carrier `w-self2b` named (#1231) and this lane
    built — and real c2 gives them DIFFERENT registers. No rule stated over
    those six fields can separate them, so board #1231's predicate is refuted
    on a decode and not merely on a source spelling.

    **And the difference is in the IL, one level down**: the bind's OWN BASE.""")
        for t, x, y in diffs:
            print("      tok 0x%04x   P6: base 0x%04x %s   P7: base 0x%04x %s"
                  % (t, x[0], x[1], y[0], y[1]))
        print("""
    `P6` binds `m` to the FORMAL's path; `P7` binds `m` to `k` — another bind.
    The two stores' root tokens are equal, distinct and both bind heads in both
    cells; what differs is what the store's root is itself rooted at.

    So the carrier needs a THIRD element per root — the root's own base — and
    `alloc::Root` carries `(tok, is_bind, offsets)` and not that. This is the
    same shape as the finding it repairs: #1231 replaced a per-producer FACT
    with a relation between two roots, and this replaces a relation between two
    roots with a relation over the roots' own definitions. Board #908's lesson
    a second time: not one contiguous field, and not one number either.""")
        return 0

    if same_carrier and same_binds:
        print("""
  THE STREAMS DECODE IDENTICALLY AND THE OBJS DIFFER.  That is a stronger and
  more uncomfortable result than the one this file was written for: the fact is
  NOT in the part of the `.ex` this decoder reads at all.  A successor owes a
  byte diff of the two streams before any rule is stated over either.""")
        return 0

    print("""
  The carrier DOES separate the pair, so `roots.py`'s row was read too fast and
  this file is the reason to distrust it. The refutation of #1231's predicate
  stands or falls on the row above, not on this paragraph.""")
    return 0


if __name__ == "__main__":
    if DC3 is None or not os.path.isdir(DC3):
        print("SKIP: toolchain absent (dc3 tree)")
        sys.exit(3)
    sys.exit(main())
