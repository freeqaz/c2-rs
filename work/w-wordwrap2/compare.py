#!/usr/bin/env python3
"""w-wordwrap2 — the four-level neutrality comparator.

Both comparators ASSERT the two runs cover the same key set before comparing
anything: a run that dropped rows silently is the failure mode `docs/STATUS.md`
trap 5 names, and a diff over an intersection prints `0 MOVED` for it.

The key is the WHOLE path and never the basename (#2667): 878 workload TUs
collapse to 841 basenames, so a collapsed comparison drops 37 rows and still
says `0 MOVED`.
"""
import json
import sys


def rows(path):
    out = {}
    for line in open(path):
        line = line.strip()
        if not line:
            continue
        d = json.loads(line)
        if d.get("record") == "provenance":
            continue
        src = d.get("src")
        if src is None:
            continue
        if src in out:
            raise SystemExit(f"DUPLICATE KEY {src} in {path} — the key is not a key")
        out[src] = d
    return out


def verdicts(path):
    return {k: v.get("class") for k, v in rows(path).items()}


def triples(path):
    """Per-TU (exact, differs, refused) out of the row's own `emit` map.

    The counters live under `emit` with `fnbyte-` keys, one map per TU. A `+1`
    in one TU and a `-1` in another sum to zero in the totals and move no
    verdict at all, which is what `w-wordwrap` §6.2 measured and why this is
    read per row rather than off the aggregate.
    """
    t = {}
    for k, v in rows(path).items():
        e = v.get("emit") or {}
        got = tuple(e.get(f"fnbyte-{n}") for n in ("exact", "differs", "refused"))
        if any(x is not None for x in got):
            t[k] = tuple(x or 0 for x in got)
    return t


def metrics(path):
    """`gap-metric <key> <value>` lines out of a scan LOG."""
    m = {}
    for line in open(path, errors="replace"):
        s = line.strip()
        if s.startswith("gap-metric "):
            parts = s.split(None, 2)
            if len(parts) == 3:
                m[parts[1]] = parts[2]
    return m


def cmp_map(label, a, b, unit):
    if set(a) != set(b):
        only_a = sorted(set(a) - set(b))
        only_b = sorted(set(b) - set(a))
        print(f"  {label}: KEY SETS DIFFER — {len(only_a)} only in base, {len(only_b)} only in tip")
        for k in only_a[:12]:
            print(f"      base only: {k}")
        for k in only_b[:12]:
            print(f"      tip  only: {k}")
    keys = sorted(set(a) & set(b))
    moved = [(k, a[k], b[k]) for k in keys if a[k] != b[k]]
    print(f"  {label}: {len(keys)} rows compared, {len(moved)} MOVED ({unit})")
    for k, x, y in moved:
        print(f"      {k}   {x} -> {y}")
    return moved


def hist(d):
    h = {}
    for v in d.values():
        h[v] = h.get(v, 0) + 1
    return dict(sorted(h.items()))


def main():
    mode = sys.argv[1]
    base, tip = sys.argv[2], sys.argv[3]
    print(f"== {mode}")
    if mode == "metrics":
        cmp_map("gap-metric keys", metrics(base), metrics(tip), "key -> value")
        return
    vb, vt = verdicts(base), verdicts(tip)
    print(f"  base histogram {hist(vb)}")
    print(f"  tip  histogram {hist(vt)}")
    cmp_map("verdicts BY FULL PATH", vb, vt, "class")
    tb, tt = triples(base), triples(tip)
    if tb or tt:
        cmp_map("byte triples (exact, differs, refused)", tb, tt, "fnbyte")
        for name, d in (("base", tb), ("tip", tt)):
            tot = [sum(x[i] or 0 for x in d.values()) for i in range(3)]
            print(f"  {name} totals: exact {tot[0]}  differs {tot[1]}  refused {tot[2]}")


main()
