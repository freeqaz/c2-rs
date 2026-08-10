#!/bin/sh
# w-wordwrap2 — grade one mutation against REAL c2 over this lane's cells.
#
# The verdict that matters is `mismatch`: a mutation that admits a `_neg` cell
# emits an obj real c2 disagrees with, and the differential says so in the one
# word the correctness rule is written in.
set -eu
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
W=work/w-wordwrap2
mkdir -p "$W/mut"
tag="$1"

: > "$W/mut/list.txt"
for f in "$ROOT"/fixtures/cpp/wwrap_*.cpp; do
    printf 'z:%s\n' "$(printf '%s' "$f" | tr '/' '\\')" >> "$W/mut/list.txt"
done

cargo build --release -p c2-harness >/dev/null 2>&1
./target/release/c2rs gap --list "$W/mut/list.txt" --flags-file "$W/fix/flags_O1.txt" \
    --jobs 8 --jsonl "$W/mut/$tag.jsonl" > "$W/mut/$tag.log" 2>&1 || true
python3 - "$W/mut/$tag.jsonl" <<'PY'
import json, sys
for line in open(sys.argv[1]):
    d = json.loads(line)
    if d.get("record") == "provenance":
        continue
    s = d.get("src", "")
    print("  %-12s %s" % (d.get("class"), s.split("\\")[-1]))
PY
grep -c "panicked" "$W/mut/$tag.log" | sed 's/^/  panics: /'
