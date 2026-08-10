#!/usr/bin/env python3
"""Four-level verdict neutrality, BY NAME and WITH DIRECTIONS.

#2667: the 878 workload TUs collapse to 841 basenames, so a basename compare
silently drops 37 rows while printing "0 MOVED". Everything here is keyed on the
full `src` path.

Level 1  the 878-TU class verdict, as a set difference by name
Level 2  the PER-TU byte triple (`fnbyte-exact` / `-differs` / `-refused`)
Level 3  the whole `gap-metric` key->value map (compared by the caller, `diff`)

    work/w-phase7b/neutral.py <base.jsonl> <tip.jsonl>
"""
import json
import sys

ACCEPT = {"match": 4, "codegen-gap": 3, "vocab-gap": 2, "port-error": 1,
          "mismatch": 0, "capture-fail": 0}


def rows(p):
    out = {}
    for line in open(p):
        r = json.loads(line)
        if "src" not in r:
            continue
        e = r.get("emit") or {}
        out[r["src"]] = (
            r["class"],
            (e.get("fnbyte-exact", 0), e.get("fnbyte-differs", 0),
             e.get("fnbyte-refused", 0)),
        )
    return out


def main():
    b, t = rows(sys.argv[1]), rows(sys.argv[2])
    print("base %d TUs, tip %d TUs; only-in-base %d, only-in-tip %d"
          % (len(b), len(t), len(set(b) - set(t)), len(set(t) - set(b))))
    moved = [(s, b[s][0], t[s][0]) for s in b if s in t and b[s][0] != t[s][0]]
    up = sum(1 for _, x, y in moved if ACCEPT[y] > ACCEPT[x])
    down = sum(1 for _, x, y in moved if ACCEPT[y] < ACCEPT[x])
    print("LEVEL 1 class verdicts: CHANGED %d  (toward acceptance %d, away %d)"
          % (len(moved), up, down))
    for s, x, y in moved[:20]:
        print("    %s  %s -> %s" % (s, x, y))
    tri = [(s, b[s][1], t[s][1]) for s in b if s in t and b[s][1] != t[s][1]]
    print("LEVEL 2 per-TU byte triples (exact,differs,refused): CHANGED %d" % len(tri))
    for s, x, y in tri[:20]:
        print("    %s  %s -> %s" % (s, x, y))
    for lbl, i in (("base", 0), ("tip", 1)):
        src = b if i == 0 else t
        se = sum(v[1][0] for v in src.values())
        sd = sum(v[1][1] for v in src.values())
        sr = sum(v[1][2] for v in src.values())
        print("  %s per-TU triple SUM: exact %d differs %d refused %d" % (lbl, se, sd, sr))


main()
