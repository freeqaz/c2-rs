#!/usr/bin/env python3
"""w-tag02 — diff two `c2rs gap` scans BY TU NAME, not by count.

`docs/STATUS.md` trap 8's lesson stated the other way round: a count that did
not move can still hide a set that did, and a count that moved says nothing
about which members moved. So this compares the per-TU verdict line by name and
prints every TU whose verdict changed, plus a `gap-metric` key-by-key diff.

Usage: python3 work/w-tag02/scandiff.py <before> <after>
"""
import re
import sys


def verdicts(path):
    out = {}
    for line in open(path, encoding="utf-8", errors="surrogateescape"):
        m = re.match(r"\s*\[\d+/\d+\]\s+(\S+)\s+(\S+)", line)
        if m:
            out[m.group(2)] = m.group(1)
    return out


def metrics(path):
    out = {}
    for line in open(path, encoding="utf-8", errors="surrogateescape"):
        m = re.match(r"\s*gap-metric (\S+) (.*)$", line.rstrip("\n"))
        if m:
            out[m.group(1)] = m.group(2)
    return out


def main():
    a, b = sys.argv[1], sys.argv[2]
    va, vb = verdicts(a), verdicts(b)
    keys = sorted(set(va) | set(vb))
    moved = [(k, va.get(k), vb.get(k)) for k in keys if va.get(k) != vb.get(k)]
    print("== per-TU verdicts")
    print("TUs before=%d after=%d  moved=%d" % (len(va), len(vb), len(moved)))
    for k, x, y in moved:
        print("  %-70s %s -> %s" % (k, x, y))
    for v in sorted(set(va.values()) | set(vb.values())):
        print("  %-14s before=%-5d after=%d" % (
            v, sum(1 for x in va.values() if x == v), sum(1 for x in vb.values() if x == v)))
    print("== gap-metric")
    ma, mb = metrics(a), metrics(b)
    mk = sorted(set(ma) | set(mb))
    n = 0
    for k in mk:
        if ma.get(k) != mb.get(k):
            print("  %-40s %s -> %s" % (k, ma.get(k), mb.get(k)))
            n += 1
    print("  keys before=%d after=%d  moved=%d  identical=%d"
          % (len(ma), len(mb), n, len(mk) - n))


if __name__ == "__main__":
    main()
