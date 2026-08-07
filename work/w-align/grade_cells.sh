#!/bin/sh
# grade_cells.sh — run the frozen cells through the DIFFERENTIAL (real c2.dll
# under wibo + byte-exact obj compare), at a chosen profile.
#
#   grade_cells.sh <tag> [<cl flags...>]
#
# Default profile is the WORKLOAD's own (board #1112). `c2rs gap` is used rather
# than `c2rs diff` for exactly that reason: `diff` takes no `--flags-file` and
# hardcodes `/Ox /GS- /c`.
#
# Its own run directory per invocation — board #1045.
set -eu
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"
lane=work/w-align
tag="$1"; shift || true
d="$lane/grade/$tag"
rm -rf "$d"; mkdir -p "$d"
if [ $# -gt 0 ]; then echo "$*" > "$d/flags.txt"; else cp "$lane/flags.txt" "$d/flags.txt"; fi
: > "$d/list.txt"
for f in "$lane"/cells/*.cpp; do
    printf 'z:%s\n' "$(printf '%s' "$root/$f" | tr '/' '\\')" >> "$d/list.txt"
done
./target/release/c2rs gap --list "$d/list.txt" --flags-file "$d/flags.txt" \
    --jobs 8 --no-cache --jsonl "$d/cells.jsonl" > "$d/gap.log" 2>&1 || true
echo "profile: $(cat "$d/flags.txt")"
python3 - "$d/cells.jsonl" <<'PY'
import json, sys, os
for line in open(sys.argv[1]):
    r = json.loads(line)
    tu = os.path.basename(r.get("tu") or r.get("file") or "?")
    print(f"  {tu:34s} {r.get('verdict','?'):14s} {r.get('blocker','') or ''}")
PY
grep -E '^ *(match|mismatch|codegen-gap|vocab-gap|port-error|capture-fail) ' "$d/gap.log" || true
