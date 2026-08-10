#!/usr/bin/env python3
"""FULL per-TU row equality between two scans — every field of every row, not a
chosen subset, so "0 moved" cannot be an artifact of which columns were looked
at.  Also prints the per-TU BYTE TRIPLE (fnbyte exact / differs / refused) for
any TU whose row differs at all.
"""
import json
import sys

SKIP = {"detail"}          # carries the binary's own path in provenance-ish text


def rows(path):
    out = {}
    for line in open(path):
        d = json.loads(line)
        if d.get("record") == "provenance":
            continue
        out[d["src"]] = d
    return out


a, b = rows(sys.argv[1]), rows(sys.argv[2])
assert set(a) == set(b), "TU sets differ"
diff = []
for s in a:
    ka = {k: v for k, v in a[s].items() if k not in SKIP}
    kb = {k: v for k, v in b[s].items() if k not in SKIP}
    if ka != kb:
        fields = sorted(k for k in set(ka) | set(kb) if ka.get(k) != kb.get(k))
        diff.append((s, fields))
print(f"{len(a)} TUs compared on EVERY field except {sorted(SKIP)}")
print(f"rows that differ anywhere: {len(diff)}")
for s, f in diff[:20]:
    print(f"   {s}: {f}")
    for k in f:
        print(f"      base {a[s].get(k)!r}")
        print(f"      tip  {b[s].get(k)!r}")

# the `detail` string too, with the one known path-bearing difference named
dd = [s for s in a if a[s].get("detail") != b[s].get("detail")]
print(f"rows whose `detail` string differs: {len(dd)}")
for s in dd[:5]:
    print(f"   {s}\n      base {a[s]['detail']}\n      tip  {b[s]['detail']}")
