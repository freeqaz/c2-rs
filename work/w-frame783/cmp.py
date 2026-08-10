#!/usr/bin/env python3
"""w-frame783 — the four-level verdict neutrality compare, with DIRECTIONS.

  level 1  every `gap-metric <key> <value>` line: added / removed / changed
  level 2  per-TU CLASS (match / mismatch / codegen-gap / vocab-gap / …)
  level 3  per-TU `gate_cause` and `gate_causes` SET
  level 4  per-TU byte triple — (fnbyte_exact, fnbyte_differs, fnbyte_refused)
           where the jsonl carries it, plus records/segments/selective_bind

    cmp.py <base.log> <tip.log> <base.jsonl> <tip.jsonl>
"""
import sys, json, re
from collections import Counter

blog, tlog, bj, tj = sys.argv[1:5]


def metrics(path):
    out = {}
    for line in open(path):
        m = re.match(r"\s*gap-metric (\S+) (.*)$", line.rstrip("\n"))
        if m:
            out[m.group(1)] = m.group(2).strip()
    return out


def rows(path):
    out = {}
    for line in open(path):
        if '"record"' in line[:14]:
            continue
        r = json.loads(line)
        out[r["src"]] = r
    return out


mb, mt = metrics(blog), metrics(tlog)
print(f"=== LEVEL 1 — gap-metric keys: base {len(mb)}  tip {len(mt)}")
added = sorted(set(mt) - set(mb))
removed = sorted(set(mb) - set(mt))
changed = sorted(k for k in set(mb) & set(mt) if mb[k] != mt[k])
print(f"    added {len(added)}   removed {len(removed)}   CHANGED VALUE {len(changed)}")
for k in added:
    print(f"      + {k} = {mt[k]}")
for k in removed:
    print(f"      - {k} = {mb[k]}")
for k in changed:
    print(f"      ~ {k}: {mb[k]}  ->  {mt[k]}")

rb, rt = rows(bj), rows(tj)
print(f"\n=== LEVEL 2 — per-TU class over {len(rb)} / {len(rt)} rows")
moved = [(s, rb[s]["class"], rt[s]["class"]) for s in rb if s in rt
         and rb[s]["class"] != rt[s]["class"]]
print(f"    TUs whose CLASS moved: {len(moved)}")
for s, a, b in moved:
    print(f"      {s}: {a} -> {b}")
print("    base class histogram:", dict(Counter(r["class"] for r in rb.values())))
print("    tip  class histogram:", dict(Counter(r["class"] for r in rt.values())))

print(f"\n=== LEVEL 3 — per-TU gate_cause / gate_causes")
c1 = [(s, rb[s].get("gate_cause"), rt[s].get("gate_cause")) for s in rb
      if s in rt and rb[s].get("gate_cause") != rt[s].get("gate_cause")]
c2 = [s for s in rb if s in rt
      and sorted(rb[s].get("gate_causes") or []) != sorted(rt[s].get("gate_causes") or [])]
print(f"    first-cause moved on {len(c1)} TUs; cause SET moved on {len(c2)} TUs")
print("    base first-cause histogram:",
      dict(Counter(r.get("gate_cause") for r in rb.values())))
print("    tip  first-cause histogram:",
      dict(Counter(r.get("gate_cause") for r in rt.values())))
for s, a, b in c1[:25]:
    print(f"      {s}: {a} -> {b}")
if len(c1) > 25:
    print(f"      … and {len(c1)-25} more")

print(f"\n=== LEVEL 4 — per-TU BYTE TRIPLE (exact, differs, refused) from `emit`")
TRIPLE = ("fnbyte-exact", "fnbyte-differs", "fnbyte-refused")


def triple(r):
    e = r.get("emit") or {}
    return tuple(e.get(k, 0) for k in TRIPLE)


diff = [s for s in rb if s in rt and triple(rb[s]) != triple(rt[s])]
tb = tuple(sum((r.get("emit") or {}).get(k, 0) for r in rb.values()) for k in TRIPLE)
tt = tuple(sum((r.get("emit") or {}).get(k, 0) for r in rt.values()) for k in TRIPLE)
print(f"    TUs carrying a triple: base "
      f"{sum(1 for r in rb.values() if r.get('emit'))}  "
      f"tip {sum(1 for r in rt.values() if r.get('emit'))}")
print(f"    TUs whose triple MOVED: {len(diff)}")
for s in diff[:25]:
    print(f"      {s}: {triple(rb[s])} -> {triple(rt[s])}")
if len(diff) > 25:
    print(f"      … and {len(diff)-25} more")
print(f"    workload totals {TRIPLE}: base {tb}  tip {tt}")
# and the whole `emit` map, key by key — the widest per-TU compare available
allk = set()
for r in list(rb.values()) + list(rt.values()):
    allk |= set((r.get("emit") or {}).keys())
moved = {}
for s in rb:
    if s not in rt:
        continue
    a, b = rb[s].get("emit") or {}, rt[s].get("emit") or {}
    for k in allk:
        if a.get(k) != b.get(k):
            moved.setdefault(k, []).append(s)
print(f"    per-TU `emit` keys tracked: {len(allk)};  keys that moved on any TU: "
      f"{len(moved)}")
for k, v in sorted(moved.items()):
    print(f"      {k}: moved on {len(v)} TUs, e.g. {v[:3]}")

print(f"\n=== LEVEL 4b — per-TU reader fields")
for f in ("gl_body_starts", "selective_bind", "fn_names", "fn_total", "fn_in_class"):
    d = [s for s in rb if s in rt and rb[s].get(f) != rt[s].get(f)]
    print(f"    {f:18s} moved on {len(d)} TUs")
    for s in d[:8]:
        print(f"        {s}: {rb[s].get(f)} -> {rt[s].get(f)}")
    if len(d) > 8:
        print(f"        … and {len(d)-8} more")
