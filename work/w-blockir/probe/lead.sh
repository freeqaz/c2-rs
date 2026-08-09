#!/bin/sh
# lead.sh — measure this class's COMPILER-LABEL LEAD by counterfactual.
#
# w-json's form: one TU per cell, each `[<subject>, framed]`, and the lead is the
# difference between the framed function's `$M` number in the cell and in the
# `leaf-none` control. `docs/LABEL_COUNTER.md`'s published surcharges have now
# been measured wrong by three separate lanes and are mode-dependent, so this
# lane measures rather than quotes.
#
# Read-only with respect to `crates/`. Usage: lead.sh
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo_root="$(cd "$here/../../.." && pwd)"

for f in lead_ctl lead_a lead_b lead_c; do
    for m in o1 ox; do
        if [ "$m" = o1 ]; then
            set -- /nologo /c /GR /O1 /Oi /EHsc
        else
            set -- /nologo /c /Ox /GS-
        fi
        sh "$here/cc.sh" "$here/$f.cpp" "$here/${f}_$m" "$@" >/dev/null 2>&1 || true
        labels="$(python3 "$repo_root/scripts/gt_dump.py" "$here/${f}_$m.obj" --no-disasm 2>/dev/null |
            grep -oE '[$][MT][0-9]+' | tr '\n' ' ')"
        echo "$f $m: $labels"
    done
done
