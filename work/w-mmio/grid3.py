#!/usr/bin/env python3
"""w-mmio — grid 3, the out-of-sample test of the SCAN amendment.

Grid 2 refuted the anchor clause grid 1 fitted: when the first in-cycle guard
cannot anchor, c2 does not fall back to the minimum — it goes on to the NEXT
guard.  That amendment (`R-GUARD-SCAN`) was fitted to grid 2's 15 failures, so
it is worth nothing until it survives a grid built to break it.

New structure, none of it in grid 1 or grid 2:

  H1  THREE guards, `[mid, outside, hi]` — the scan must step over BOTH a
      non-unimodal in-cycle guard AND an out-of-cycle one.
  H2  `[hi, mid]` — the reverse of grid 2's failing order, so the guard that
      rescues the scan is the LOWER of the two rather than the higher.
  H3  every one of the six orderings of a 3-cycle's own registers, as a
      three-guard list.
  H4  `[mid, hi]` and `[hi, mid]` at arity 6 and 7, in r8/r9 — grid 2's finding
      replicated outside the register range it was found in.
  H5  two guards, BOTH outside the cycle — the only shape in which no guard can
      anchor and the minimum must.
  H6  a literal slot beside a two-guard scan (the `?mmioGetInfo` shape with the
      guards permuted off their own confound).
"""

import hashlib
import itertools
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from grid import cell_source, cycles_of, rotations, run  # noqa: E402
from rule import (ARG_REG, anchor_first_in_cycle, anchor_scan, chain_from,  # noqa: E402
                  predict)


def build():
    cells = []

    def add(kind, n, pairs, guards, lit=None):
        perm = [pairs.get(i, i) for i in range(n)]
        name = "%s_n%d_p%s_g%s_l%s" % (
            kind, n, "".join(str(x) for x in perm),
            "".join(str(x) for x in guards), "n" if lit is None else str(lit))
        if any(c["name"] == name for c in cells):
            return
        cells.append(dict(name=name, kind=kind, n=n, perm=perm,
                          guard_slots=list(guards), ncalls=1, lit_slot=lit,
                          cycles=cycles_of(perm),
                          src=cell_source(name, n, perm, guards, 1, lit)))

    for n in (4, 5):
        for sub in itertools.combinations(range(n), 3):
            lo, mid, hi = sorted(sub)
            out = [s for s in range(n) if s not in sub]
            for pairs, _t in rotations(list(sub)):
                if out:
                    add("h1", n, pairs, [mid, out[0], hi])      # H1
                    add("h1", n, pairs, [hi, out[0], mid])
                add("h2", n, pairs, [hi, mid])                   # H2
                for order in itertools.permutations((lo, mid, hi)):  # H3
                    add("h3", n, pairs, list(order))
                if len(out) >= 2:
                    add("h5", n, pairs, [out[0], out[1]])        # H5

    for n in (6, 7):                                             # H4
        for sub in itertools.combinations(range(n), 3):
            if max(sub) < n - 1:
                continue
            lo, mid, hi = sorted(sub)
            for pairs, _t in rotations(list(sub)):
                add("h4", n, pairs, [mid, hi])
                add("h4", n, pairs, [hi, mid])

    for n in (4, 5):                                             # H6
        for sub in itertools.combinations(range(n), 3):
            lo, mid, hi = sorted(sub)
            free = [s for s in range(n) if s not in sub]
            if not free:
                continue
            for pairs, _t in rotations(list(sub)):
                add("h6", n, pairs, [mid, hi], free[-1])
                add("h6", n, pairs, [hi, mid], free[-1])

    return cells


def split_at(perm, anchor):
    mv = chain_from(perm, anchor)
    d = [x for x, _ in mv]
    j = 0
    while j + 1 < len(d) and d[j] < d[j + 1]:
        j += 1
    return dict(anchor=ARG_REG[anchor],
                entry=[[11, ARG_REG[anchor]]] + [list(x) for x in mv[:j]],
                call=[list(x) for x in mv[j:]])


def gen(outdir):
    cells = build()
    os.makedirs(outdir, exist_ok=True)
    kinds, sep_fit, sep_min = {}, 0, 0
    for c in cells:
        kinds[c["kind"]] = kinds.get(c["kind"], 0) + 1
        cyc = c["cycles"][0]
        c["pred"] = split_at(c["perm"],
                             anchor_scan(c["perm"], cyc, c["guard_slots"]))
        c["pred_fit"] = split_at(c["perm"],
                                 anchor_first_in_cycle(c["perm"], cyc,
                                                       c["guard_slots"]))
        c["pred_rmin"] = split_at(c["perm"], min(cyc))
        sep_fit += c["pred"] != c["pred_fit"]
        sep_min += c["pred"] != c["pred_rmin"]

    for want in ("h1", "h2", "h3", "h4", "h5", "h6"):
        assert kinds.get(want, 0) > 0, "class %r EMPTY" % want
    assert sep_fit >= 25, \
        "grid 3 separates the SCAN from grid 1's fit at only %d cells" % sep_fit
    assert sep_min >= 60, \
        "grid 3 separates the SCAN from #1414's R-MIN at only %d cells" % sep_min
    # H5 must be the class where NO guard can anchor.
    h5 = [c for c in cells if c["kind"] == "h5"]
    assert h5 and all(c["pred"]["anchor"] == ARG_REG[min(c["cycles"][0])]
                      for c in h5), "H5 must fall back to the minimum"

    for c in cells:
        open(os.path.join(outdir, c["name"] + ".cpp"), "w").write(c["src"])
    open(os.path.join(outdir, "manifest.json"), "w").write(
        json.dumps([{k: v for k, v in c.items() if k != "src"} for c in cells],
                   indent=1, sort_keys=True))
    h = hashlib.sha256()
    for c in sorted(cells, key=lambda x: x["name"]):
        h.update(c["name"].encode())
        h.update(c["src"].encode())
    print("cells            %d" % len(cells))
    print("by class         %s" % json.dumps(kinds, sort_keys=True))
    print("separates SCAN from grid 1's fit   %d cells" % sep_fit)
    print("separates SCAN from #1414's R-MIN  %d cells" % sep_min)
    print("sha256           %s" % h.hexdigest())


if __name__ == "__main__":
    if sys.argv[1] == "gen":
        gen(sys.argv[2])
    elif sys.argv[1] == "run":
        run(sys.argv[2], sys.argv[3])
