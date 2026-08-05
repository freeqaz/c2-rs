#!/usr/bin/env python3
"""external.py — lane w-alloc. Score ALLOC on cells that are not in the grid at
all: the four allocation rules `leaf_store.rs` records as refuted, every cell
`docs/STORE_SCHEDULE.md` publishes, and the recon/supplement probes.

The four killer cells are the brief's constraint — "take the four refuted
allocation rules as constraints your rule must satisfy, not as noise". A rule
that misses any of them is disqualified whatever it scores on the grid.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from alloc_lib import compile_cod, parse_cod, classify  # noqa: E402
from model import alloc  # noqa: E402

W = os.path.dirname(os.path.abspath(__file__))

# (name, body-values, nf, kind, EXPECTED alloc as published)
# value lists name the distinct producer per statement.
CASES = [
    # --- the four refuted-rule killer cells, leaf_store.rs:915-921 ----------
    ("B4", "1,2,3,1",     0, "L", {"1": "r11", "2": "r10", "3": "r9"}),
    ("B7", "1,2,3,2,1",   0, "L", {"1": "r10", "2": "r11", "3": "r9"}),
    ("A1", "1,2,1,2",     0, "L", {"1": "r10", "2": "r11"}),
    ("B6", "1,1,2,2,2",   0, "L", {"1": "r10", "2": "r11"}),
    # --- leaf_store.rs's own admitted/neighbour cells -----------------------
    ("N1", "1,2",         0, "L", {"1": "r11", "2": "r10"}),
    ("N2", "1,2,3",       0, "L", {"1": "r11", "2": "r10", "3": "r9"}),
]


def main():
    src = os.path.join(W, "external.cpp")
    with open(src, "w") as f:
        f.write("struct S { unsigned %s; };\n"
                % ",".join("m%X" % i for i in range(20)))
        for name, vals, nf, kind, _exp in CASES:
            vs = vals.split(",")
            body = " ".join("s->m%X = %s;" % (k, v) for k, v in enumerate(vs))
            f.write("void X%s(S* s) { %s }\n" % (name, body))
    txt = compile_cod(src, os.path.join(W, "external.cod"),
                      os.path.join(W, "external.obj"))
    fns = parse_cod(txt)

    n = ok = 0
    for name, vals, nf, kind, exp in CASES:
        fn = "X" + name
        if fn not in fns:
            raise SystemExit("FAIL: %s produced no PROC" % fn)
        obs = {}
        toks = []
        for d in classify(fns[fn]):
            if d["mn"] == "blr":
                continue
            if d["role"] == "store":
                obs.setdefault(vals.split(",")[d["off"] // 4], d["src"])
                toks.append("S%d@%s" % (d["off"] // 4, d["src"]))
            elif d.get("dst"):
                toks.append("P%s" % d["dst"])
        vs = vals.split(",")
        specs = ["V%d" % sorted(set(vs), key=vs.index).index(v) for v in vs]
        idx = sorted(set(vs), key=vs.index)
        pred_v = alloc(specs, nf, kind)
        pred = {idx[int(k[1:])]: r for k, r in pred_v.items()}
        n += 1
        agree_doc = (obs == exp)
        agree_rule = (pred == obs)
        ok += agree_rule
        print("%-4s %-14s measured=%-34s ALLOC=%-34s %s  (doc: %s)"
              % (name, vals, obs, pred,
                 "HIT " if agree_rule else "MISS",
                 "agrees" if agree_doc else "DISAGREES"))
        print("     emitted: %s" % " ".join(toks))
    print("\nALLOC on external cells: %d of %d" % (ok, n))
    if ok != n:
        raise SystemExit("DISQUALIFIED: a killer cell is not reproduced")


if __name__ == "__main__":
    main()
