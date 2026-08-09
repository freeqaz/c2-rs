#!/usr/bin/env python3
"""w-inlfence — the FBM verdicts on `src/keygen_xbox.cpp`, base against tip.

The claim being checked is the one that matters for an over-broadness test: the
function this fence takes back must be one the ORACLE grades WRONG, never a
byte-exact one.
"""
import json
import sys

for path, tag in ((sys.argv[1], "BASE"), (sys.argv[2], "TIP")):
    for line in open(path):
        d = json.loads(line)
        if d.get("record") == "provenance":
            continue
        if d["src"] != "src/keygen_xbox.cpp":
            continue
        rows = {k: v for k, v in (d.get("emit") or {}).items() if k.startswith("fnbyte-shape|")}
        print(tag, sorted(rows.items()))
