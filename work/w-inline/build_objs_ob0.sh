#!/bin/sh
# build_objs_ob0.sh — build_objs.sh's twin, through refobj_ob0.sh.
# Reuses the SAME index.txt numbering so the two trees pair by name.
#
# Usage: build_objs_ob0.sh <index.txt> <outdir> [jobs]
set -eu
repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
idx="$1"
outdir="$2"
jobs="${3:-8}"
mkdir -p "$outdir"
cat "$idx" | tr '\t' '\n' | xargs -P "$jobs" -n 2 sh -c '
    "$0/work/w-inline/refobj_ob0.sh" "$2" "$0/'"$outdir"'/$1.obj" >/dev/null 2>&1 \
        || echo "COMPILE-FAIL $2" >&2
' "$repo_root"
echo "built: $(ls "$outdir" | grep -c '\.obj$') of $(wc -l < "$idx")"
