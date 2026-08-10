#!/bin/sh
# fixall.sh — every fixture, at `/O1` and `/Ox`, through BOTH binaries.
#
# The list is regenerated HERE, after this lane's last fixture landed, and
# `wc -l`-checked — regenerating it earlier is the slip that cost two lanes on
# 2026-08-09 and would grade a corpus this rung's own fixtures are absent from.
#
# Lane w-main2.
set -eu
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"

ls fixtures/cpp/*.cpp > work/w-main2/fixall.txt
echo "fixtures in the list: $(wc -l < work/w-main2/fixall.txt)"

printf '/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc\n' > work/w-main2/mode_o1.txt
printf '/nologo /wd4355 /wd4164 /c /GR /Ox /Oi /EHsc\n' > work/w-main2/mode_ox.txt

for bin in work/w-main2/c2rs-base work/w-main2/c2rs-tip; do
    for m in o1 ox; do
        out="work/w-main2/fixall-$(basename "$bin")-$m.log"
        "$bin" gap --list work/w-main2/fixall.txt \
            --flags-file "work/w-main2/mode_$m.txt" --jobs 6 > "$out" 2>&1 || true
        printf '%-28s %-3s  %s\n' "$(basename "$bin")" "$m" \
            "$(grep -oE 'gap-metric (match|mismatch) [0-9]+' "$out" | tr '\n' ' ')"
    done
done
