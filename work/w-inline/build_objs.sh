#!/bin/sh
# build_objs.sh — compile every TU of a frozen sample list to a reference obj at
# the WORKLOAD's own flags, in parallel, and write an index mapping obj -> src.
#
# Lane w-inline measurement tooling. Read-only with respect to `crates/`.
# Wraps `work/w-frame/refobj.sh` (board #195: `c2rs compile` hardcodes /Ox).
#
# Usage: build_objs.sh <list.txt> <outdir> [jobs]
set -eu

repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
list="$1"
outdir="$2"
jobs="${3:-8}"

mkdir -p "$outdir"
: > "$outdir/index.txt"

i=0
while IFS= read -r f; do
    [ -n "$f" ] || continue
    i=$((i + 1))
    n=$(printf '%04d' "$i")
    printf '%s\t%s\n' "$n" "$f" >> "$outdir/index.txt"
    printf '%s\t%s\n' "$n" "$f"
done < "$list" | xargs -P "$jobs" -n 2 sh -c '
    "$0/work/w-frame/refobj.sh" "$2" "$0/'"$outdir"'/$1.obj" >/dev/null 2>&1 \
        || echo "COMPILE-FAIL $2" >&2
' "$repo_root"

# `i` above is lost with the subshell the pipeline creates, so the denominator is
# re-derived from the index the loop actually wrote — never from a variable that
# may not have crossed the pipe.
echo "built: $(ls "$outdir" | grep -c '\.obj$') of $(wc -l < "$outdir/index.txt")"
