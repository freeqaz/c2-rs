#!/usr/bin/env python3
"""The FOURTH neutrality level: the per-TU byte-verdict TRIPLE, by full path.

`neutral.py` compares the per-TU *class*; the aggregate `gap-metric` map
compares totals. Neither can say *"not one function byte-verdict outside the
target moved in either direction"* — a `+1` somewhere and a `-1` somewhere else
sum to zero in the map and move no class at all. This compares
`(fnbyte-exact, fnbyte-differs, fnbyte-refused)` per TU, keyed on the WHOLE
path (#2667: 878 workload TUs collapse to 841 basenames).

    work/w-wordwrap/triples.py <a.jsonl> <b.jsonl> [label]
"""
import json
import sys


def load(p):
    d = {}
    for line in open(p):
        r = json.loads(line)
        if r.get("record") == "provenance":
            continue
        src = r["src"]
        if src[:2].lower() == "z:":
            src = src.replace("\\", "/").rsplit("/", 1)[-1]
        e = r.get("emit", {})
        assert src not in d, "duplicate key %r — the run is not per-TU" % src
        d[src] = (
            e.get("fnbyte-exact", 0),
            e.get("fnbyte-differs", 0),
            e.get("fnbyte-refused", 0),
        )
    return d


def main(argv):
    a, b = load(argv[0]), load(argv[1])
    label = argv[2] if len(argv) > 2 else ""
    if set(a) != set(b):
        print("%s KEY SETS DIFFER: only-a %d only-b %d"
              % (label, len(set(a) - set(b)), len(set(b) - set(a))))
        for k in sorted(set(a) ^ set(b)):
            print("   ", k)
        return 1
    moved = [(k, a[k], b[k]) for k in sorted(a) if a[k] != b[k]]
    print("%s %d rows, %d TRIPLES MOVED" % (label, len(a), len(moved)))
    for k, x, y in moved:
        print("    %-52s %s -> %s" % (k, x, y))
    tot = lambda d, i: sum(v[i] for v in d.values())  # noqa: E731
    for i, n in enumerate(("exact", "differs", "refused")):
        print("    total %-8s %8d -> %8d" % (n, tot(a, i), tot(b, i)))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
