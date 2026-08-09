#!/usr/bin/env python3
"""w-fence2 — the per-(TU, symbol) FBM sets, base vs tip, BY NAME.

Deliverable 3: w-fltret's 444 admissions were all byte-WRONG, and a narrowed
fence must not re-admit one. `--class444` filters to the shape those 444 are,
`call-sequence-value-fp` / `seq`, and reports the population both ends.
"""
import json
import sys

def rows(path):
    out = {}
    for line in open(path):
        r = json.loads(line)
        key = (r.get("tu") or r.get("src"), r.get("sym") or r.get("symbol"))
        out[key] = r
    return out

a, b = rows(sys.argv[1]), rows(sys.argv[2])
print(f"fnbyte-differs rows: base {len(a)}, tip {len(b)}")
only_a = sorted(k for k in a if k not in b)
only_b = sorted(k for k in b if k not in a)
print(f"  resolved (in base, not in tip): {len(only_a)}")
for k in only_a[:40]:
    print(f"    RESOLVED {k[0]} :: {k[1]}")
print(f"  new (in tip, not in base): {len(only_b)}")
for k in only_b[:40]:
    print(f"    NEW      {k[0]} :: {k[1]}")
