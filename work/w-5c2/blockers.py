#!/usr/bin/env python3
"""blockers.py — sum `fn_blockers` / `emit_blockers` over a gap-scan JSONL.

    python3 work/w-5c2/blockers.py <scan.jsonl> [--tus] [--grep SUBSTR]

Prints the provenance `binary_sha` first, because a two-scan counterfactual of
two DIFFERENT binaries is not a counterfactual (w-5c2 PREREG D1).
"""
import json, sys, collections

def load(path):
    rows = [json.loads(l) for l in open(path) if l.strip()]
    prov = rows[0] if rows and rows[0].get("record") == "provenance" else {}
    return prov, [r for r in rows if not r.get("record")]

def tally(rows, field):
    n = collections.Counter()
    tus = collections.Counter()
    for r in rows:
        for k, v in (r.get(field) or {}).items():
            n[k] += v
            tus[k] += 1
    return n, tus

if __name__ == "__main__":
    a = sys.argv[1:]
    path = a[0]
    field = "fn_blockers"
    if "--emit" in a:
        field = "emit_blockers"
    grep = a[a.index("--grep") + 1] if "--grep" in a else None
    prov, rows = load(path)
    print("binary_sha", prov.get("binary_sha"), "c2rs_head", prov.get("c2rs_head"))
    n, tus = tally(rows, field)
    print(f"{field}: {len(n)} keys, sum {sum(n.values())}")
    cls = collections.Counter(r["class"] for r in rows)
    print("TU classes:", dict(sorted(cls.items())))
    for k, v in n.most_common():
        if grep and grep not in k:
            continue
        print(f"  {v:>9}  {tus[k]:>4} TUs  {k}")
