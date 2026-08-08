#!/bin/sh
# w-clear — grade probe .cpp files at the WORKLOAD's own flags, via `c2rs gap`.
# $1 = repo root (absolute), rest = probe basenames without .cpp
ROOT="$1"; shift
D="$ROOT/work/w-clear/probe"
: > "$D/list.txt"
for n in "$@"; do echo "$n.cpp" >> "$D/list.txt"; done
"$ROOT/target/release/c2rs" gap --list "$D/list.txt" \
    --flags-file "$ROOT/work/dc3-workload/flags.txt" --cwd "$D" \
    --jobs 4 --no-cache --jsonl "$D/out.jsonl" > "$D/out.txt" 2>&1
python3 - "$D/out.jsonl" <<'PY'
import json,sys
for ln in open(sys.argv[1]):
    d=json.loads(ln)
    if d.get('record')=='provenance': continue
    print("%-14s %-12s in-class %s/%s  blockers=%s" % (
        d['src'], d['class'], d.get('fn_in_class'), d.get('fn_total'),
        json.dumps(d.get('fn_blockers') or {})))
PY
