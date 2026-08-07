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
root="$(cd "$(dirname "$0")/../../.." && pwd)"   # one level deeper than the lane's own copy
cd "$root"
lane=work/w-align16/diag
tag="$1"; shift || true
d="$lane/grade/$tag"
rm -rf "$d"; mkdir -p "$d"
if [ $# -gt 0 ]; then echo "$*" > "$d/flags.txt"; else cp "work/w-align16/flags.txt" "$d/flags.txt"; fi
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
    # w-align16: the `gap` jsonl calls these `src` / `class` / `reason`, NOT
    # `tu` / `verdict` / `blocker`. w-align's copy of this printer read the wrong
    # three keys and printed `?` for every cell — the per-cell column of that
    # lane's own table came from `cellrows.py`, not from here, and this file was
    # a silent instrument nobody noticed.
    if "src" not in r:
        continue          # the header record (binary / toolchain provenance)
    tu = os.path.basename((r.get("src") or "?").replace("\\", "/"))
    print(f"  {tu:34s} {r.get('class','?'):14s} {(r.get('reason') or '')[:70]}")
PY
grep -E '^ *(match|mismatch|codegen-gap|vocab-gap|port-error|capture-fail) ' "$d/gap.log" || true
