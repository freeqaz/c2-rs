#!/usr/bin/env python3
"""w-inlfence — what moved on `src/keygen_xbox.cpp`, the one TU the fence fires on."""
import json
import sys

for path, tag in ((sys.argv[1], "BASE"), (sys.argv[2], "TIP")):
    for line in open(path):
        d = json.loads(line)
        if d.get("record") == "provenance":
            continue
        if d["src"] != "src/keygen_xbox.cpp":
            continue
        inc = {k: v for k, v in (d.get("fn_dispatch") or {}).items() if "|INCLASS|" in k}
        print(tag, "in-class", d["fn_in_class"], inc)
        print(tag, "emit-in-class", (d.get("emit") or {}).get("emit-in-class"))
        print(tag, "emit blockers", sorted((d.get("emit_blockers") or {}).items()))
