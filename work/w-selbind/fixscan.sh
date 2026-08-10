#!/bin/sh
# Grade EVERY fixture with a NAMED binary at a NAMED mode and keep one JSON row
# per fixture, so two binaries can be compared BY NAME rather than by a count.
#
# The `/Ox` half is not optional: `w-biquad` (#2533) shipped a live wrong emit
# (`Port=Mismatch` at offset 760) that the `/O1`-only workload scan, the `/O1`
# fixture lane and every workspace test all missed, and only a both-modes
# by-name fixture scan saw it.
#
#     work/w-selbind/fixscan.sh <base|tip|cf> <o1|ox>
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
. "$here/env.sh"
which="$1"
mode="$2"
case "$mode" in
    o1) flags="/O1 /GS- /c" ;;
    ox) flags="/Ox /GS- /c" ;;
    *) echo "usage: fixscan.sh <base|tip|cf> <o1|ox>" >&2; exit 2 ;;
esac
# Regenerated HERE, after the last fixture this lane authored, and `wc -l`-checked
# by the caller against `ls fixtures/cpp/*.cpp | wc -l`.
ls "$repo"/fixtures/cpp/*.cpp | sed "s#^$repo/##" > "$here/fixtures.txt"
printf '%s\n' "$flags" > "$here/flags_$mode.txt"
"$here/c2rs-$which" gap --list "$here/fixtures.txt" \
    --flags-file "$here/flags_$mode.txt" --cwd "$repo" \
    --jsonl "$here/fix_${which}_$mode.jsonl" \
    > "$here/fix_${which}_$mode.log" 2>&1
echo "fixtures listed: $(wc -l < "$here/fixtures.txt")"
grep -E 'gap-metric (match|mismatch|codegen-gap|vocab-gap|port-error|capture-fail) ' \
    "$here/fix_${which}_$mode.log" | sed 's/^ *//'
