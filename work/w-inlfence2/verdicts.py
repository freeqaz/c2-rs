#!/usr/bin/env python3
"""w-inlfence2 — verdict neutrality LEVEL 1: the 878 TUs BY NAME.

A count of 18 at both ends does not say the same 18. This compares the per-TU
class of every source path in both scans and prints every disagreement.

Usage: python3 work/w-inlfence2/verdicts.py <base.fnd.out> <tip.fnd.out>
"""
import re
import sys

ROW = re.compile(r"^\s*\[\s*\d+/\d+\]\s+(\S+)\s+(\S.*)$")


def verdicts(path):
    v = {}
    for line in open(path, errors="replace"):
        m = ROW.match(line.rstrip("\n"))
        if not m:
            continue
        cls, src = m.group(1), m.group(2).strip()
        v[src] = cls
    return v


def main():
    base, tip = verdicts(sys.argv[1]), verdicts(sys.argv[2])
    print(f"TUs: base {len(base)}  tip {len(tip)}")
    only_b = sorted(set(base) - set(tip))
    only_t = sorted(set(tip) - set(base))
    moved = sorted(k for k in set(base) & set(tip) if base[k] != tip[k])
    for label, rows in (("BASE-ONLY TU", only_b), ("TIP-ONLY TU", only_t)):
        print(f"{label}: {len(rows)}")
        for k in rows:
            print(f"    {k}")
    print(f"CLASS CHANGED: {len(moved)}")
    for k in moved:
        print(f"    {k}: {base[k]} -> {tip[k]}")
    mb = sorted(k for k, c in base.items() if c == "match")
    mt = sorted(k for k, c in tip.items() if c == "match")
    print(f"\nMATCH SET  base {len(mb)}  tip {len(mt)}  identical={mb == mt}")
    for k in sorted(set(mb) ^ set(mt)):
        print(f"    SYMMETRIC DIFFERENCE: {k}")
    for cls in ("mismatch", "port-error"):
        b = [k for k, c in base.items() if c == cls]
        t = [k for k, c in tip.items() if c == cls]
        print(f"{cls}: base {len(b)} tip {len(t)}  {sorted(set(b) | set(t))}")


if __name__ == "__main__":
    main()
