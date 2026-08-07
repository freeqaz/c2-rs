#!/bin/sh
# capture_cells.sh — reference obj + IL for every frozen cell, at a profile.
#
#   capture_cells.sh [<tag>] [<cl flags...>]
#
# One directory per cell (board #1045): four parallel probes once shared a
# PID-keyed temp dir, the captures raced, and a lane published a finding that
# reversed when it was rerun.
#
# The manifest is checked FIRST. A cell that changed after the freeze makes
# every number downstream of it unciteable, so this exits rather than warns.
set -eu
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"
lane=work/w-order3
tag="${1:-workload}"; shift 2>/dev/null || true
( cd "$lane/cells" && sha256sum -c SHA256SUMS ) > "$lane/manifest_check_$tag.txt" 2>&1 || {
    echo "MANIFEST FAILED — a frozen cell changed" >&2
    exit 1
}
d="$lane/caps/$tag"
rm -rf "$d"; mkdir -p "$d"
if [ $# -gt 0 ]; then echo "$*" > "$d/flags.txt"; else cp "$lane/flags.txt" "$d/flags.txt"; fi
echo "profile: $(cat "$d/flags.txt")"
for f in "$lane"/cells/*.cpp; do
    n="$(basename "$f" .cpp)"
    c="$d/$n"
    mkdir -p "$c"
    ./target/release/c2rs capture "$f" --keep-il "$c" --flags-file "$d/flags.txt" \
        > "$c/capture.txt" 2>&1 || echo "CAPTURE-FAIL $n"
    ./target/release/c2rs compile "$f" --keep-obj "$c/ref.obj" --flags-file "$d/flags.txt" \
        > "$c/compile.txt" 2>&1 || echo "COMPILE-FAIL $n"
    printf '%s ' "$n"
done
echo
