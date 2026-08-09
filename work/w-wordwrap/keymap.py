#!/usr/bin/env python3
"""Every `gap-metric` key as a key -> value MAP, compared between two runs.

`w-empty`'s rule: a total that is unchanged is not evidence, because two
opposite moves cancel. Compared per key, with VANISHED and APPEARED reported
separately from CHANGED — a key that disappears is a different finding from one
that moves.

    work/w-wordwrap/keymap.py <a_metrics.txt> <b_metrics.txt>
"""
import sys


def load(p):
    d = {}
    for line in open(p):
        line = line.strip()
        if not line.startswith("gap-metric "):
            continue
        parts = line.split()
        if len(parts) < 3:
            continue
        d[parts[1]] = " ".join(parts[2:])
    return d


def main(argv):
    a, b = load(argv[0]), load(argv[1])
    vanished = sorted(set(a) - set(b))
    appeared = sorted(set(b) - set(a))
    changed = sorted(k for k in set(a) & set(b) if a[k] != b[k])
    print("keys: %d -> %d   %d vanished, %d appeared, %d changed"
          % (len(a), len(b), len(vanished), len(appeared), len(changed)))
    for k in vanished:
        print("   VANISHED %-44s %s" % (k, a[k]))
    for k in appeared:
        print("   APPEARED %-44s %s" % (k, b[k]))
    for k in changed:
        print("   %-46s %s -> %s" % (k, a[k], b[k]))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
