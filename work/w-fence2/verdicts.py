#!/usr/bin/env python3
"""w-fence2 — the per-TU verdict SET, base vs tip, BY NAME with the direction of
every move stated. A count can hide one TU lost and one gained.
"""
import json
import sys

RANK = {"match": 0, "codegen-gap": 1, "vocab-gap": 2, "port-error": 3,
        "capture-fail": 4, "mismatch": 9}


def verdicts(path):
    out = {}
    for line in open(path):
        r = json.loads(line)
        if r.get("record") == "provenance":
            continue
        out[r["src"]] = r["class"]
    return out


a, b = verdicts(sys.argv[1]), verdicts(sys.argv[2])
only_a = sorted(set(a) - set(b))
only_b = sorted(set(b) - set(a))
moved = [(s, a[s], b[s]) for s in sorted(set(a) & set(b)) if a[s] != b[s]]
print(f"base {len(a)} TUs, tip {len(b)} TUs; only-in-base {len(only_a)}, "
      f"only-in-tip {len(only_b)}, CHANGED {len(moved)}")
for s in only_a:
    print(f"  ONLY-IN-BASE {s} = {a[s]}")
for s in only_b:
    print(f"  ONLY-IN-TIP  {s} = {b[s]}")
toward, away = 0, 0
for s, x, y in moved:
    d = "TOWARD-ACCEPTANCE" if RANK[y] < RANK[x] else "AWAY-FROM-ACCEPTANCE"
    if RANK[y] < RANK[x]:
        toward += 1
    else:
        away += 1
    print(f"  {d:20} {s}: {x} -> {y}")
print(f"DIRECTIONS: {toward} toward acceptance, {away} away from it")
for cls in sorted(set(a.values()) | set(b.values()), key=lambda c: RANK.get(c, 5)):
    print(f"  {cls:14} {sum(1 for v in a.values() if v == cls):4} -> "
          f"{sum(1 for v in b.values() if v == cls):4}")
