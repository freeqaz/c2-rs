#!/usr/bin/env python3
"""neutral.py — verdict neutrality at level 1 (878 TUs BY NAME) and level 2
(every `gap-metric` key as a key->value MAP), with the DIRECTION of every
moved verdict.

    python3 work/w-pool/neutral.py work/w-pool/base.jsonl work/w-pool/tip.jsonl

A set comparison, not a count: two scans can agree on `match 20` with a
different twenty.  Direction is reported because "0 changed" and "3 toward
acceptance and 3 away" are the same count and not the same result.
"""
import json
import sys

# How close to accepting each class is.  Only the ORDER matters.
RANK = {"capture-fail": 0, "port-error": 1, "mismatch": 2, "vocab-gap": 3,
        "codegen-gap": 4, "match": 5}


def load(path):
    rows, metrics = {}, {}
    for line in open(path):
        r = json.loads(line)
        if r.get("record") == "provenance":
            continue
        rows[r["src"]] = r["class"]
        for k, v in (r.get("emit") or {}).items():
            metrics[k] = metrics.get(k, 0) + v
    return rows, metrics


def main(a_path, b_path):
    a, am = load(a_path)
    b, bm = load(b_path)
    rc = 0

    print("== LEVEL 1 — 878 TUs, BY NAME")
    only_a = sorted(set(a) - set(b))
    only_b = sorted(set(b) - set(a))
    changed = sorted(s for s in set(a) & set(b) if a[s] != b[s])
    print(f"   base {len(a)} TUs, tip {len(b)} TUs; "
          f"only-in-base {len(only_a)}, only-in-tip {len(only_b)}, CHANGED {len(changed)}")
    toward = away = 0
    for s in changed:
        d = RANK.get(b[s], -1) - RANK.get(a[s], -1)
        dirn = "TOWARD acceptance" if d > 0 else ("AWAY from it" if d < 0 else "sideways")
        print(f"     {s}: {a[s]} -> {b[s]}   {dirn}")
        toward += d > 0
        away += d < 0
    print(f"   DIRECTIONS: {toward} toward acceptance, {away} away from it")
    for s in only_a:
        print(f"     ONLY IN BASE: {s} ({a[s]})")
    for s in only_b:
        print(f"     ONLY IN TIP:  {s} ({b[s]})")
    if only_a or only_b or changed:
        rc = 1

    print("== LEVEL 2 — every `gap-metric` key, as a key->value MAP")
    van = sorted(set(am) - set(bm))
    new = sorted(set(bm) - set(am))
    moved = sorted(k for k in set(am) & set(bm) if am[k] != bm[k])
    print(f"   base {len(am)} keys, tip {len(bm)} keys; "
          f"vanished {len(van)}, appeared {len(new)}, CHANGED {len(moved)}")
    for k in van:
        print(f"     VANISHED: {k} = {am[k]}")
    for k in new:
        print(f"     APPEARED: {k} = {bm[k]}")
    for k in moved:
        print(f"     CHANGED:  {k}: {am[k]} -> {bm[k]}")
    if van or new or moved:
        rc = 1

    for k in ("fnbyte-exact", "fnbyte-differs", "fnbyte-refused", "fnbyte-denominator"):
        print(f"   {k}: {am.get(k)} -> {bm.get(k)}")
    print("NEUTRAL" if rc == 0 else "NOT NEUTRAL — see above")
    return rc


if __name__ == "__main__":
    sys.exit(main(sys.argv[1], sys.argv[2]))
