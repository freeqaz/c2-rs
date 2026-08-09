#!/usr/bin/env python3
"""Print the full `gap --jsonl` row for each named TU.

CEILING.md §11.4 item 8: a TU conversion is priced off the GATE's own number
(`Bindings::per_record`, visible as `gate_causes` / `fn_names` / `fn_total` in
this row), never off a per-function instrument keyed on `FnCensus::emit_name`.
"""
import json
import sys

scan = sys.argv[1]
names = set(sys.argv[2:])
for line in open(scan):
    line = line.strip()
    if not line:
        continue
    try:
        r = json.loads(line)
    except Exception:
        continue
    src = r.get("src") or r.get("file") or r.get("path")
    if names and src not in names:
        continue
    print("=" * 78)
    print(src)
    for k, v in r.items():
        if k in ("src", "file", "path"):
            continue
        s = str(v)
        if len(s) > 2000:
            s = s[:2000] + " ..."
        print("  %s: %s" % (k, s))
