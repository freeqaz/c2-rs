#!/usr/bin/env python3
"""w-inlfence2 — where w-fltret's 444 land, as a SET intersection.

`reach.py` establishes that the shipped fence removes 0 of the 444. This says
WHY, by crossing the 444 with the `localcallee` arm measured at the fenced tip:

  * `localcallee` — the port's own composed body still emits a REL24 against a
    name this TU defines. The fence saw them and could not PROVE the callee is
    small, because the port cannot lower it (`TuContext::definition` is `None`
    for a name whose IL the parser refused).
  * `nolocal` — the body relocates against nothing this TU defines, so no
    inline question arises at all.

Two totals cannot answer this. A set can.

Usage:
  python3 work/w-inlfence2/r2arm.py <pre.jsonl> <base.jsonl> <witness.fnd.err>
"""
import json
import sys
from collections import Counter


def load_jsonl(path):
    s = set()
    for line in open(path, errors="replace"):
        line = line.strip()
        if not line:
            continue
        try:
            r = json.loads(line)
        except ValueError:
            continue
        s.add((r["tu"], r["sym"]))
    return s


def load_witness(path):
    arms = {}
    for line in open(path, errors="replace"):
        if not line.startswith("XLOCAL\t"):
            continue
        parts = line.rstrip("\n").split("\t")
        if len(parts) != 5:
            continue
        _, tu, sym, bucket, arm = parts
        arms[(tu, sym)] = (bucket, arm)
    return arms


def main():
    pre, base, arms = load_jsonl(sys.argv[1]), load_jsonl(sys.argv[2]), load_witness(sys.argv[3])
    r2 = base - pre
    old = base & pre
    print(f"R2 (w-fltret's increment) = {len(r2)}")
    print(f"the base population        = {len(old)}")
    print(f"witness rows at the fenced tip = {len(arms)}")

    for label, s in (("R2", r2), ("BASE-2111", old)):
        c = Counter()
        for k in s:
            c[arms.get(k, ("<no-witness>", "<no-witness>"))] += 1
        print(f"\n-- {label}: (bucket at the FENCED tip, fence arm) --")
        for k, n in sorted(c.items(), key=lambda kv: -kv[1]):
            print(f"    {n:6d}  {k[0]:22s} {k[1]}")


if __name__ == "__main__":
    main()
