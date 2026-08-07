#!/bin/sh
# dump_refs.sh — gt_dump every FRONTIER reference obj, one file per TU.
set -eu
repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
out_root="$repo_root/work/w-front2/ref"

while read -r src; do
    [ -n "$src" ] || continue
    key="$(printf '%s' "$src" | tr '/' '_')"
    d="$out_root/$key"
    [ -f "$d/out.obj" ] || { echo "MISSING	$src"; continue; }
    python3 "$repo_root/scripts/gt_dump.py" "$d/out.obj" > "$d/dis.txt" 2>&1
    echo "$(wc -l < "$d/dis.txt")	$src"
done < "$repo_root/work/w-front2/tus.txt"
