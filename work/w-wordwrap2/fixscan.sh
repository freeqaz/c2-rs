#!/bin/sh
# w-wordwrap2 — every fixture at BOTH modes, with BOTH binaries.
#
# The list is REGENERATED on every invocation and its length printed. A cached
# list is a fixture nobody graded: this lane adds four files, and a run against
# a list written before them would print a clean diff over a set that excludes
# the whole change.
#
# The `/Ox` half is mandatory and is not a formality — `w-biquad` shipped a live
# wrong emit at offset 760 that the `/O1`-only workload scan, the `/O1` fixture
# lane and every workspace test missed.
set -eu
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
W=work/w-wordwrap2
mkdir -p "$W/fix"

: > "$W/fix/list.txt"
for f in "$ROOT"/fixtures/cpp/*.cpp; do
    printf 'z:%s\n' "$(printf '%s' "$f" | tr '/' '\\')" >> "$W/fix/list.txt"
done
echo "fixtures in the list: $(wc -l < "$W/fix/list.txt")"

printf '/O1 /Oi /GS- /c\n' > "$W/fix/flags_O1.txt"
printf '/Ox /GS- /c\n'     > "$W/fix/flags_Ox.txt"

for bin in base tip; do
    for mode in O1 Ox; do
        echo "== $bin $mode"
        "$W/c2rs-$bin" gap --list "$W/fix/list.txt" --flags-file "$W/fix/flags_$mode.txt" \
            --jobs "${C2RS_JOBS:-8}" --jsonl "$W/fix/$bin-$mode.jsonl" \
            > "$W/fix/$bin-$mode.log" 2>&1 || true
        grep -E '^  (match|mismatch|codegen-gap|vocab-gap|capture-fail|port-error) ' \
            "$W/fix/$bin-$mode.log" || echo "  NO BUCKET LINES — the run graded nothing"
    done
done
