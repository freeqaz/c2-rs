#!/usr/bin/env python3
"""w-inlfence — base vs tip on EVERY published `gap-metric` key.

Verdict neutrality level 2: the commission requires *all* gap-metric keys
accounted, not the four the lane cares about. This prints the whole key set,
partitioned into MOVED / UNCHANGED / BASE-ONLY / TIP-ONLY, so a key that
appeared or vanished cannot hide inside "unchanged".

Usage: python3 work/w-inlfence/metricdiff.py <base.fnd.out> <tip.fnd.out>
"""
import sys


def metrics(path):
    m = {}
    for line in open(path, errors="replace"):
        t = line.strip()
        if not t.startswith("gap-metric "):
            continue
        rest = t[len("gap-metric "):]
        k, _, v = rest.rpartition(" ")
        if k:
            m[k] = v
    return m


def main():
    base, tip = metrics(sys.argv[1]), metrics(sys.argv[2])
    keys = sorted(set(base) | set(tip))
    moved, same, only_b, only_t = [], [], [], []
    for k in keys:
        b, t = base.get(k), tip.get(k)
        if b is None:
            only_t.append((k, t))
        elif t is None:
            only_b.append((k, b))
        elif b != t:
            moved.append((k, b, t))
        else:
            same.append(k)
    print(f"KEYS  base {len(base)}  tip {len(tip)}  union {len(keys)}")
    print(f"MOVED {len(moved)} · UNCHANGED {len(same)} · "
          f"BASE-ONLY {len(only_b)} · TIP-ONLY {len(only_t)}")
    print("\n-- MOVED --")
    for k, b, t in moved:
        try:
            d = f"{int(t) - int(b):+d}"
        except ValueError:
            d = ""
        print(f"  {k:52s} {b:>12s} -> {t:>12s}  {d}")
    print("\n-- TIP-ONLY (a key the fence minted) --")
    for k, t in only_t:
        print(f"  {k:52s} {t:>12s}")
    print("\n-- BASE-ONLY (a key that VANISHED — must be empty) --")
    for k, b in only_b:
        print(f"  {k:52s} {b:>12s}")


if __name__ == "__main__":
    main()
