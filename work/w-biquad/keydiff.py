#!/usr/bin/env python3
"""w-biquad — every `gap-metric` key, compared as a key -> value MAP.

Not a `diff`: a diff of two scan logs is dominated by per-TU progress lines and
would bury the one thing that matters — a key that VANISHED. Prints vanished,
appeared and changed, each with both values.

    keydiff.py base.out tip.out
"""
import sys

PREFIX = "gap-metric "


def load(path):
    out = {}
    for line in open(path):
        s = line.strip()
        if not s.startswith(PREFIX):
            continue
        rest = s[len(PREFIX):]
        k, _, v = rest.rpartition(" ")
        out[k] = v
    return out


def main():
    a, b = load(sys.argv[1]), load(sys.argv[2])
    vanished = sorted(set(a) - set(b))
    appeared = sorted(set(b) - set(a))
    changed = sorted(k for k in set(a) & set(b) if a[k] != b[k])
    print(f"base {len(a)} keys, tip {len(b)} keys")
    print(f"vanished {len(vanished)}")
    for k in vanished:
        print(f"   - {k} = {a[k]}")
    print(f"appeared {len(appeared)}")
    for k in appeared:
        print(f"   + {k} = {b[k]}")
    print(f"changed  {len(changed)}")
    for k in changed:
        print(f"   ~ {k}: {a[k]} -> {b[k]}")


main()
