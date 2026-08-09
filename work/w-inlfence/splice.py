#!/usr/bin/env python3
"""w-inlfence — the SPLICE and ELIDE populations, base against tip.

Both mechanisms model an inline the port has ALREADY graded, and both require a
callee this TU defines. If the fence were over-broad on them, these counters
would fall. Only the aggregate keys are summed — the per-function
`fnbyte-splice-fn|…|<mangled name>` rows are one key each and would print a
symbol table, not a measurement.
"""
import collections
import json
import sys

KEEP = ("fnbyte-elided", "fnbyte-elided-exact", "fnbyte-spliced",
        "fnbyte-spliced-exact", "fnbyte-tu-empty-callees")

for path, tag in ((sys.argv[1], "BASE"), (sys.argv[2], "TIP")):
    c = collections.Counter()
    for line in open(path):
        d = json.loads(line)
        if d.get("record") == "provenance":
            continue
        for k, v in (d.get("emit") or {}).items():
            if k in KEEP or (k.startswith("fnbyte-spliced|") or k.startswith("fnbyte-spliced-reloc|")):
                c[k] += v
    print(tag)
    for k in sorted(c):
        print("   %9d  %s" % (c[k], k))
