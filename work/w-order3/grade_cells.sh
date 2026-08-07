#!/bin/sh
# grade_cells.sh — run the frozen cells through the DIFFERENTIAL (real c2.dll
# under wibo + byte-exact obj compare), at a chosen profile.
#
#   grade_cells.sh <tag> [<cl flags...>]
#
# `c2rs gap` and not `c2rs diff`, because only `gap` takes `--flags-file` and
# the workload's profile is the one the answer has to hold at (board #1112).
# Its own run directory per invocation (board #1045).
set -eu
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"
lane=work/w-order3
tag="$1"; shift 2>/dev/null || true
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
n = {}
for line in open(sys.argv[1]):
    r = json.loads(line)
    if "src" not in r:
        continue          # header record (binary / toolchain provenance)
    tu = os.path.basename((r.get("src") or "?").replace("\\", "/"))
    cls = r.get("class", "?")
    n[cls] = n.get(cls, 0) + 1
    print(f"  {tu:34s} {cls:14s} {(r.get('reason') or '')[:70]}")
print("  " + " ".join(f"{k}={v}" for k, v in sorted(n.items())))
PY
