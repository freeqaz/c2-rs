#!/usr/bin/env python3
"""w-three — THE SERIES behind the FUNCTIONS-vs-BYTES gap.

`w-slots` (#3147) was told to read a charge out of one fixture's own obj, did
exactly that, the objs read **3**, and shipping it would have been a wrong obj:
the series is `2n+1`, not `3n`. **Reading one cell gives a number right for that
cell and wrong as a rule. Only the series separates them.**

So `NetworkSocket.cpp`'s "2 of 4 bodies already exact = 50 %, but 24 of 660
bytes = 3.64 %" is ONE CELL and is not yet a finding. This computes the same
quantity over every TU in the workload that has both denominators, so the claim
can be stated as a rule or withdrawn.

REFUSES rather than reporting a null on: fewer than 800 TU rows; fewer than 20
TUs with both denominators; any TU whose byte partition the scan itself flagged
broken (`bytefrac-partition-broken`).
"""
import json
import sys


def die(msg):
    print(f"REFUSE: {msg}", file=sys.stderr)
    sys.exit(2)


def main(path):
    rows = []
    n = 0
    broken = 0
    for line in open(path):
        r = json.loads(line)
        if not r.get("src"):
            continue
        n += 1
        e = r.get("emit", {})
        broken += e.get("bytefrac-partition-broken", 0)
        fd, fe = e.get("fnbyte-denominator", 0), e.get("fnbyte-exact", 0)
        bd, be = e.get("bytefrac-denominator", 0), e.get("bytefrac-exact", 0)
        if fd and bd:
            rows.append((r["src"], r["class"], fd, fe, bd, be))
    if n < 800:
        die(f"{path}: {n} TU rows (< 800) — a truncated scan")
    if broken:
        die(f"{path}: `bytefrac-partition-broken` fired on {broken} TU(s) — the "
            f"instrument's own partition identity failed, so no ratio here is a measurement")
    if len(rows) < 20:
        die(f"{path}: only {len(rows)} TUs carry BOTH denominators — too few for a series")

    print(f"TU rows {n} · TUs with both denominators {len(rows)} · "
          f"`bytefrac-partition-broken` {broken} (the scan's own identity, must be 0)")
    print()
    # The population-level number first: the two quantities SUMMED, not averaged.
    FD = sum(r[2] for r in rows); FE = sum(r[3] for r in rows)
    BD = sum(r[4] for r in rows); BE = sum(r[5] for r in rows)
    print(f"SUMMED over the {len(rows)}: functions {FE}/{FD} = {100*FE/FD:.2f} %   "
          f"bytes {BE}/{BD} = {100*BE/BD:.2f} %   -> {(FE/FD)/(BE/BD):.2f}x apart")
    print()
    # The per-TU series, restricted to TUs where the function reading is
    # NON-TRIVIAL (partially exact) — the population the "N of M already exact"
    # sentence is ever written about.
    part = [r for r in rows if 0 < r[3] < r[2]]
    print(f"PARTIALLY-exact TUs (0 < fnbyte-exact < fnbyte-denominator): {len(part)}")
    print(f"{'TU':<52} {'fn':>9} {'bytes':>13} {'ratio':>7}")
    ser = []
    for src, cls, fd, fe, bd, be in sorted(part, key=lambda r: -(r[2])):
        fp, bp = fe / fd, (be / bd if bd else 0.0)
        rat = (fp / bp) if bp else float("inf")
        ser.append((src, cls, fp, bp, rat))
        print(f"{src:<52} {fe:>4}/{fd:<4} {be:>6}/{bd:<6} {rat:>7.1f}x")
    fin = [s for s in ser if s[4] != float("inf")]
    if fin:
        rr = sorted(s[4] for s in fin)
        print()
        print(f"THE SERIES: {len(fin)} finite ratios · min {rr[0]:.1f}x · "
              f"median {rr[len(rr)//2]:.1f}x · max {rr[-1]:.1f}x · "
              f"ALL > 1 ? {'YES' if rr[0] > 1.0 else 'NO'}")
        over = sum(1 for x in rr if x > 1.0)
        print(f"  ratios > 1 (functions FLATTER than bytes): {over} of {len(rr)}")
        print(f"  ratios <= 1 (bytes flatter than functions): {len(rr)-over} of {len(rr)}")
    inf = [s for s in ser if s[4] == float("inf")]
    if inf:
        print(f"  ratio INFINITE (some functions exact, ZERO bytes exact): {len(inf)}")
        for s in inf:
            print(f"      {s[0]}")
    print()
    print("DISCRIMINATING CELLS: "
          f"{len({round(s[4],3) for s in fin})} distinct finite ratios over {len(fin)} TUs")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        die("usage: series.py <scan.jsonl>")
    main(sys.argv[1])
