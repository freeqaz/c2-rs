#!/usr/bin/env python3
"""external.py — lane w-sched. Score the FROZEN rule on cells it was never
fitted to: w-dclass/B §3.4's ten, w-pair §4's twenty, and `xboxheap`.

Each entry is (specs, base symbols, [(machine base reg, member offset)] in
SOURCE order). The third field is needed because several of the published
structs are not written in declaration order, and `f3`/`f4` write the same two
offsets through two different pointers.
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from sched_lib import parse_cod, classify  # noqa: E402
from model import predict, conflicted  # noqa: E402

W = os.path.dirname(os.path.abspath(__file__))
R3, R4 = "r3", "r4"


def k(*offs, reg=R3):
    return [(reg, o) for o in offs]


CELLS = {
    # --- w-dclass/B §3.4 — struct O {a0 b4 c8 d12 e16} -------------------
    "o1":  (["F", "L0", "F", "F"], "pppp", k(0, 4, 8, 12)),
    "o2":  (["F", "F", "F", "L0"], "pppp", k(0, 4, 8, 12)),
    "o3":  (["L0", "F", "F", "F"], "pppp", k(0, 4, 8, 12)),
    "o4":  (["F", "L0", "L0", "F"], "pppp", k(0, 4, 8, 12)),
    "o5":  (["F", "L0", "L1", "F"], "pppp", k(0, 4, 8, 12)),
    "o6":  (["F", "F", "L0", "F", "F"], "ppppp", k(0, 4, 8, 12, 16)),
    "o7":  (["F", "L1", "L2", "L3", "F"], "ppppp", k(0, 4, 8, 12, 16)),
    "o8":  (["F", "F", "F", "L7", "F"], "ppppp", k(0, 4, 8, 12, 16)),
    # struct X {fh0 uh4 sz8 ct12}; source order sz, fh, uh, ct
    "w1":  (["F", "T", "T", "L0"], "pppp", k(8, 0, 4, 12)),
    # --- w-pair §4 — struct S8 {a0 b4 c8 ...} -----------------------------
    "c0":  (["F", "F", "F"], "ppp", k(0, 4, 8)),
    "c1":  (["F", "L0"], "pp", k(0, 4)),
    "c2f": (["F", "L0", "F"], "ppp", k(0, 4, 8)),
    "d1":  (["L0", "F"], "pp", k(0, 4)),
    "d2":  (["L0", "F", "F"], "ppp", k(0, 4, 8)),
    "d3":  (["L0", "F", "F", "F"], "pppp", k(0, 4, 8, 12)),
    "d7":  (["F", "L0", "F", "F"], "pppp", k(0, 4, 8, 12)),
    "d8":  (["L0", "F", "F", "F", "F"], "ppppp", k(0, 4, 8, 12, 16)),
    "c7":  (["F"] * 6 + ["L0"], "p" * 7, k(0, 4, 8, 12, 16, 20, 24)),
    "c8":  (["L0"] + ["F"] * 6, "p" * 7, k(0, 4, 8, 12, 16, 20, 24)),
    "d6":  (["I", "F", "F", "F"], "pppp", k(0, 4, 8, 12)),
    "e5":  (["L1", "L2", "F", "F"], "pppp", k(0, 4, 8, 12)),
    # struct H {fh0 uh4 lh8(n8,p12) sz16 ct20}
    "c5":  (["A", "A"], "ll", k(8, 12)),
    "d5":  (["A", "F", "F", "A"], "lhhl", k(8, 16, 20, 12)),
    "e3":  (["A", "F", "F", "T", "T", "A"], "lhhhhl", k(8, 16, 20, 0, 4, 12)),
    "e1":  (["A", "F", "F", "A"], "ghhg",
            [(R4, 0), (R3, 16), (R3, 20), (R4, 4)]),
    "e2":  (["A", "F", "F", "A"], "hhhh", k(0, 16, 20, 4)),
    "f1":  (["A", "F", "F", "A"], "lbbl",
            [(R3, 8), (R4, 16), (R4, 20), (R3, 12)]),
    "f2":  (["A", "F", "F", "A"], "laal",
            [(R4, 8), (R3, 16), (R3, 20), (R4, 12)]),
    "f3":  (["L0", "F", "F", "F"], "aabb",
            [(R3, 16), (R3, 20), (R4, 16), (R4, 20)]),
    "f4":  (["L0", "F", "F", "F"], "aabb",
            [(R4, 16), (R4, 20), (R3, 16), (R3, 20)]),
    "c3":  (["F", "T", "L0", "T", "A", "A"], "hhhhll", k(16, 0, 20, 4, 8, 12)),
    "c9":  (["F", "F", "T", "T", "A", "A"], "hhhhll", k(16, 20, 0, 4, 8, 12)),
}

# `xboxheap`'s constructor, read off the REAL obj for
# src/xdk/nuispeech/xboxheap.cpp at the workload's flags (rung
# _2026-08-05-w-dclass-b-0x27.md §3.1, reproduced here). The 4 prologue
# instructions, the `mr r31,r3` live-range save, the `bl` and the epilogue are
# not store-schedule instructions; §4 of the findings prices `mr r31,r3`
# separately.
XBOXHEAP = (["F", "T", "L0", "T", "A", "A"], "hhhhll")
XBOXHEAP_EMITTED = "P2 S0 P4 S1 S2 S3 S4 S5"


def emitted_tokens(ann, keys):
    idx = {kk: i for i, kk in enumerate(keys)}
    out = []
    for pos, d in enumerate(ann):
        if d["role"] == "store":
            out.append("S%d" % idx[(d["base"], d["off"])])
        elif d["mn"] == "blr":
            continue
        else:
            dst = d.get("dst")
            tgt = None
            for e in ann[pos + 1:]:
                if e["role"] == "store" and e.get("src") == dst:
                    tgt = idx[(e["base"], e["off"])]
                    break
            out.append("P%d" % tgt)
    return " ".join(out)


def main():
    fns = parse_cod(open(os.path.join(W, "control.cod")).read())
    tot = ok = conf = 0
    for cid, (specs, base, keys) in CELLS.items():
        ann = classify(fns[cid])
        emitted = emitted_tokens(ann, keys)
        pred = " ".join(predict(specs, list(base)))
        c = conflicted(ann)
        tot += 1
        conf += c
        good = emitted == pred
        ok += good
        print("  %-4s %-6s %-4s pred %-30s got %-30s"
              % (cid, "CONFL" if c else "", "OK" if good else "MISS",
                 pred, emitted))
    print("\nexternal cells %d | EXACT %d | conflicted %d | MISSES %d"
          % (tot, ok, conf, tot - ok))
    xp = " ".join(predict(XBOXHEAP[0], list(XBOXHEAP[1])))
    print("\nxboxheap  pred %s\n          got  %s   -> %s"
          % (xp, XBOXHEAP_EMITTED,
             "EXACT" if xp == XBOXHEAP_EMITTED else "MISS"))


if __name__ == "__main__":
    main()
