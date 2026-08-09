#!/bin/sh
# Grade EVERY fixture with a named binary at named flags and keep one JSON row
# per fixture, so the two binaries can be compared BY NAME rather than by a
# count. `w-biquad` #2533 is why this exists at both modes: a live wrong-bytes
# emit at `/Ox` survived the `/O1` lane, the 878-TU scan and every workspace
# test, and only the both-modes by-name scan saw it.
#
#     work/w-pool2/fixscan.sh <base|tip> <tag> <flags...>
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
which="$1"; tag="$2"; shift 2
bin="$here/c2rs-$which"
ls "$repo"/fixtures/cpp/*.cpp | sed "s#^$repo/##" > "$here/fixtures.txt"
printf '%s\n' "$*" > "$here/flags_$tag.txt"
"$bin" gap --list "$here/fixtures.txt" --flags-file "$here/flags_$tag.txt" \
    --cwd "$repo" --jsonl "$here/fix_${which}_$tag.jsonl" > "$here/fix_${which}_$tag.log" 2>&1
grep -E 'gap-metric (match|mismatch|codegen-gap|vocab-gap|port-error|capture-fail) ' \
    "$here/fix_${which}_$tag.log" | sed 's/^ *//'
