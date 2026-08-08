#!/usr/bin/env python3
"""w-mmio — grid 2, the OUT-OF-SAMPLE test of R-GUARD-UNIMODAL.

Grid 1 is where the rule was *fitted*: it refuted board #1414's "always the
cycle minimum" at 14 cells, and it refuted "the guard's register, always" at 14
others.  A rule fitted to a grid and scored on the same grid is worth what
`w-clear`'s single discriminating cell was worth, so this grid is generated
from STRUCTURE GRID 1 DOES NOT CONTAIN, its predictions are committed before
the first `cl.exe`, and it is scored without refitting.

What is new here, and what each class is FOR:

  G1  3-cycle, guard on the MIDDLE register.  Grid 1 guarded only the minimum
      (`base`) and the maximum (`gtgt`), so "the guard's register" and "the
      cycle maximum" were confounded on every cell that could have separated
      them.  Half of these cells anchor at the guard and half fall back to the
      minimum, and which half is decided by clause 1's unimodality test alone.
  G2  2-cycle, guard on the HIGH register.  Grid 1's `gtgt` was k=3 only.
  G3  TWO guards on two different cycle registers, in both orders — does the
      FIRST guard anchor, or the one that can?
  G4  first guard OUTSIDE the cycle, second inside — can a later guard anchor?
  G5  guard on a non-minimum register WITH a literal slot (the `?mmioGetInfo`
      shape, moved off its own confound).
  G6  3-cycles reaching r8/r9 at arity 6 and 7 — the register range grid 1
      never left.
"""

import hashlib
import itertools
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from grid import cell_source, cycles_of, rotations, run  # noqa: E402
from rule import ARG_REG, chain_from, predict, unimodal  # noqa: E402


def build():
    cells = []

    def add(kind, n, pairs, guards, lit=None):
        perm = [pairs.get(i, i) for i in range(n)]
        name = "%s_n%d_p%s_g%s_l%s" % (
            kind, n, "".join(str(x) for x in perm),
            "".join(str(x) for x in guards), "n" if lit is None else str(lit))
        cells.append(dict(name=name, kind=kind, n=n, perm=perm,
                          guard_slots=list(guards), ncalls=1, lit_slot=lit,
                          cycles=cycles_of(perm),
                          src=cell_source(name, n, perm, guards, 1, lit)))

    # G1 — 3-cycle, guard on the MIDDLE register
    for n in (3, 4, 5):
        for sub in itertools.combinations(range(n), 3):
            mid = sorted(sub)[1]
            for pairs, _t in rotations(list(sub)):
                add("mid", n, pairs, [mid])

    # G2 — 2-cycle, guard on the HIGH register
    for n in (2, 3, 4, 5):
        for sub in itertools.combinations(range(n), 2):
            for pairs, _t in rotations(list(sub)):
                add("hi2", n, pairs, [max(sub)])

    # G3 — two guards on two different cycle registers, both orders
    for n in (3, 4, 5):
        for sub in itertools.combinations(range(n), 3):
            lo, mid, hi = sorted(sub)
            for pairs, _t in rotations(list(sub)):
                add("g2ord", n, pairs, [hi, lo])
                add("g2ord", n, pairs, [lo, hi])
                add("g2ord", n, pairs, [mid, hi])

    # G4 — first guard OUTSIDE the cycle, second inside
    for n in (4, 5):
        for sub in itertools.combinations(range(n), 3):
            outside = [s for s in range(n) if s not in sub]
            if not outside:
                continue
            for pairs, _t in rotations(list(sub)):
                add("gout2", n, pairs, [outside[0], max(sub)])

    # G5 — guard on a non-minimum register, WITH a literal slot
    for n in (3, 4):
        for sub in itertools.combinations(range(n), 2):
            free = [s for s in range(n) if s not in sub]
            if not free:
                continue
            for pairs, _t in rotations(list(sub)):
                add("lit2", n, pairs, [max(sub)], free[-1])

    # G6 — 3-cycles that reach r8/r9
    for n in (6, 7):
        for sub in itertools.combinations(range(n), 3):
            if max(sub) < n - 1:
                continue
            for pairs, _t in rotations(list(sub)):
                add("wide", n, pairs, [min(sub)])
                add("wide", n, pairs, [max(sub)])

    return cells


def gen(outdir):
    cells = build()
    os.makedirs(outdir, exist_ok=True)
    kinds = {}
    anchors = {"guard": 0, "fallback": 0}
    for c in cells:
        kinds[c["kind"]] = kinds.get(c["kind"], 0) + 1
        cyc = c["cycles"][0]
        a, e, cc = predict(c["perm"], cyc, c["guard_slots"])
        c["pred"] = dict(anchor=ARG_REG[a],
                         entry=[[11, ARG_REG[a]]] + [list(x) for x in e],
                         call=[list(x) for x in cc])
        anchors["guard" if a in c["guard_slots"] else "fallback"] += 1
        # the rival's prediction, frozen at the same time
        mn = min(cyc)
        mv = chain_from(c["perm"], mn)
        d = [x for x, _ in mv]
        j = 0
        while j + 1 < len(d) and d[j] < d[j + 1]:
            j += 1
        c["pred_rmin"] = dict(anchor=ARG_REG[mn],
                              entry=[[11, ARG_REG[mn]]] + [list(x) for x in mv[:j]],
                              call=[list(x) for x in mv[j:]])
        c["rivals_differ"] = (c["pred"] != c["pred_rmin"])

    for want in ("mid", "hi2", "g2ord", "gout2", "lit2", "wide"):
        assert kinds.get(want, 0) > 0, "class %r EMPTY" % want
    # G1 must contain BOTH sub-cases of clause 1, or it tests nothing.
    mids = [c for c in cells if c["kind"] == "mid"]
    at_guard = [c for c in mids if c["pred"]["anchor"] == ARG_REG[c["guard_slots"][0]]]
    assert 0 < len(at_guard) < len(mids), \
        "the middle-guard class must split into anchor-at-guard AND fallback"
    diff = [c for c in cells if c["rivals_differ"]]
    assert len(diff) >= 20, \
        "grid 2 separates R-GUARD-UNIMODAL from #1414 at only %d cells" % len(diff)

    for c in cells:
        open(os.path.join(outdir, c["name"] + ".cpp"), "w").write(c["src"])
    open(os.path.join(outdir, "manifest.json"), "w").write(
        json.dumps([{k: v for k, v in c.items() if k != "src"} for c in cells],
                   indent=1, sort_keys=True))
    h = hashlib.sha256()
    for c in sorted(cells, key=lambda x: x["name"]):
        h.update(c["name"].encode())
        h.update(c["src"].encode())
    print("cells             %d" % len(cells))
    print("by class          %s" % json.dumps(kinds, sort_keys=True))
    print("anchor at guard   %d   fallback to minimum %d"
          % (anchors["guard"], anchors["fallback"]))
    print("middle-guard      %d anchor at the guard, %d fall back"
          % (len(at_guard), len(mids) - len(at_guard)))
    print("rivals differ     %d cells separate R-GUARD-UNIMODAL from #1414's R-MIN"
          % len(diff))
    print("sha256            %s" % h.hexdigest())


if __name__ == "__main__":
    if sys.argv[1] == "gen":
        gen(sys.argv[2])
    elif sys.argv[1] == "run":
        run(sys.argv[2], sys.argv[3])
