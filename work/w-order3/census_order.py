#!/usr/bin/env python3
"""census_order.py — does the 878-TU workload contain this rung's shape at all?

Read-only, over `work/w-bss/census/sections.jsonl` (871 objs, each carrying its
full section `order` array and every `.bss`/`.data` symbol with its storage
class). That file already answers this rung's question directly, so the answer
is "cannot" or "can", never "did not look".

Prints, from runs:
  1. how many objs put a non-COMDAT `.bss` BEFORE `.XBLD$W:C2`  (this rung's
     section move);
  2. how many put one AFTER every code group (the third slot cell `O04` found);
  3. how many have a `.bss` holding at least one STATIC symbol, split by slot;
  4. how many of all the above are FUNCTIONLESS objs (no `.text`), which is the
     only class `coff::data::emit_data_obj` serves.
"""
import json
import sys

PATH = sys.argv[1] if len(sys.argv) > 1 else "work/w-bss/census/sections.jsonl"

n = 0
before_c2 = []
between = []
after_code = []
static_bss = []
functionless = 0
functionless_bss_static = []

for line in open(PATH):
    r = json.loads(line)
    n += 1
    order = r["order"]
    try:
        i_c2 = order.index(".XBLD$W:C2")
        i_c1 = order.index(".XBLD$W:C1")
    except ValueError:
        print(f"  NO WATERMARK PAIR: {r['src']}")
        continue
    has_text = any(s.startswith(".text") for s in order)
    if not has_text:
        functionless += 1
    for b in r.get("bss", []):
        if b.get("comdat"):
            continue
        idx = b["idx"] - 1          # census `idx` is 1-based over `order`
        has_static = any(s.get("sc") == 3 for s in b.get("syms", []))
        if idx < i_c2:
            slot = "before-C2"
            before_c2.append((r["src"], has_static))
        elif idx < i_c1:
            slot = "between"
            between.append((r["src"], has_static))
        else:
            slot = "after-C1"
            after_code.append((r["src"], has_static))
        if has_static:
            static_bss.append((r["src"], slot))
            if not has_text:
                functionless_bss_static.append((r["src"], slot))

print(f"objs                                   {n}")
print(f"functionless objs (no .text at all)    {functionless}")
print()
print(f"non-COMDAT .bss BEFORE .XBLD$W:C2      {len(before_c2)}   <- this rung's move")
print(f"non-COMDAT .bss BETWEEN the watermarks {len(between)}   <- Rule S1")
print(f"non-COMDAT .bss AFTER .XBLD$W:C1       {len(after_code)}   <- the deferred/third slot")
print()
print(f".bss holding >=1 STATIC symbol         {len(static_bss)}")
from collections import Counter
print("   by slot: " + str(dict(Counter(s for _, s in static_bss))))
print(f"   of those, in a FUNCTIONLESS obj     {len(functionless_bss_static)}")
if before_c2:
    print("   before-C2 objs (first 20):")
    for src, hs in before_c2[:20]:
        print(f"     {src}  static={hs}")
