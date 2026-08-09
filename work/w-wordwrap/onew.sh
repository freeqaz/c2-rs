#!/bin/sh
# Grade ONE WORKLOAD TU through `c2rs gap`, at the workload's own flags and cwd,
# and print its `gap-metric` emit rows — the byte judge for a single file while
# a class is being built.
#
#     work/w-wordwrap/onew.sh src/system/rndobj/wordwrap.cpp
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
. "$here/env.sh"
work="$here/out/onew"
mkdir -p "$work"
: > "$work/list.txt"
for f in "$@"; do
    printf '%s\n' "$f" >> "$work/list.txt"
done
"$repo/target/release/c2rs" gap --list "$work/list.txt" \
    --flags-file "$C2RS_MAIN/work/dc3-workload/flags.txt" --cwd "$C2RS_DC3" \
    --jobs 1 --jsonl "$work/scan.jsonl" > "$work/report.txt" 2>&1
sed -n '/GAP REPORT/,$p' "$work/report.txt" | head -30
python3 - "$work/scan.jsonl" <<'PY'
import json, sys
for line in open(sys.argv[1]):
    d = json.loads(line)
    e = d.get("emit", {})
    print(d["src"], d["class"], "|", d.get("gate_causes"))
    for k in sorted(e):
        if k.startswith(("fnbyte", "bytefrac", "emit-bound", "emit-records")):
            print(f"   {k} {e[k]}")
PY
