#!/usr/bin/env python3
"""Stratified TU list for the pre-PREREG framing sweep, off this lane's own scan."""
import json, random, sys

path = sys.argv[1] if len(sys.argv) > 1 else "work/w-frame783/base.jsonl"
rows = [json.loads(l) for l in open(path) if '"record"' not in l[:14]]
if "--keys" in sys.argv:
    print(sorted(rows[0].keys()))
    sys.exit(0)

match = [r["src"] for r in rows if r["class"] == "match"]
# the reach-pool proxy: TUs where the wide framing already names everything is
# not in the jsonl, so stratify on what IS: fn_total buckets + gate_cause.
random.seed(783)
others = [r["src"] for r in rows if r["class"] != "match" and (r.get("ex_len") or 0) > 0]
small = [s for s in others if (next(r for r in rows if r["src"] == s).get("fn_total") or 0) <= 20]
mid = [s for s in others if 20 < (next(r for r in rows if r["src"] == s).get("fn_total") or 0) <= 400]
big = [s for s in others if (next(r for r in rows if r["src"] == s).get("fn_total") or 0) > 400]
pick = match + random.sample(small, min(30, len(small))) \
     + random.sample(mid, min(40, len(mid))) + random.sample(big, min(25, len(big)))
# the two named target TUs, always
for s in ("src/system/math/vec.cpp", "src/system/decomp_pch.cpp"):
    if s not in pick:
        pick.append(s)
seen = set()
for s in pick:
    if s not in seen:
        seen.add(s)
        print(s)
