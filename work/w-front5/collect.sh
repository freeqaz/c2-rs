#!/bin/sh
# Render the SMALL, committable summaries out of this lane's large scratch.
#
# The 878-TU `--jsonl` scans are ~62 MB each and are NEVER committed (the merge
# funnel dropped two 64 MB dumps from a branch this session; `.git` is at
# 703 MB). Everything a reader needs is a derived text file, produced here so
# the derivation is re-runnable rather than pasted.
#
#     work/w-front5/collect.sh
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
cd "$repo"

{
    echo "== the five FRONTIER TUs, from this lane's own base scan"
    sed -n '/^  FRONTIER — /,/^  FRONTIER BY .text. BYTE FRACTION/p' \
        "$here/base_scan.log" | head -8
    echo
    echo "== byte fraction"
    sed -n '/accepted\/total bytes/,/BYTE-FRACTION CONTROL/p' \
        "$here/base_scan.log" | head -8
    echo
    echo "== CFG reachability"
    sed -n '/blocked | src\/Main.cpp/,/^    LABEL CHANNEL/p' \
        "$here/base_scan.log" | head -7
    echo
    echo "== frontier by codegen"
    sed -n '/ den exact wrong cg-ref reader ungrade/,/^    TOTAL 40/p' \
        "$here/base_scan.log" | head -8
} > "$here/FRONTIER.txt"

{
    echo "### gate-cause histogram, 878 TUs, base"
    python3 "$here/causes.py" "$here/base_scan.jsonl" gl-stop-name-not-mangled
    echo
    echo "### the 15 gl-stop-name-not-mangled TUs, by name"
    python3 "$here/first_cause.py" "$here/base_scan.jsonl" \
        gl-stop-name-not-mangled
} > "$here/CAUSES.txt"

{
    echo "### the GATE's binding walk over each frontier TU's own .gl"
    for t in main mmio wordwrap xtea keygen; do
        gl=$(ls "$here"/il/$t/*.gl)
        ex=$(ls "$here"/il/$t/*.ex)
        echo "---------- $t"
        python3 "$here/glwalk.py" "$gl" "$ex"
    done
} > "$here/GLWALK.txt"

{
    echo "### COUNTERFACTUAL — the bound-record-name length clause removed"
    echo "### base binary 53e70e8fa9cfb8a482a2072927b615aa"
    echo "### cf   binary 829e849b2e2d3ad5a00aef577ab18e90"
    echo
    python3 "$here/keydiff.py" "$here/base_metrics.txt" "$here/cf_metrics.txt"
    echo
    python3 "$here/verdicts.py" "$here/base_scan.jsonl" "$here/cf_scan.jsonl"
} > "$here/COUNTERFACTUAL.txt"

{
    echo "### THREE-LEVEL NEUTRALITY, base -> tip"
    echo "### base and tip binaries are BIT-IDENTICAL (md5"
    echo "### 53e70e8fa9cfb8a482a2072927b615aa both), because this lane"
    echo "### ships no crates/, fixtures/ or scripts/ change. The comparisons"
    echo "### below are run anyway: 'identical binary' is a claim about the"
    echo "### build, and the verdict sets are the claim about the OUTPUT."
    echo
    echo "== workload gap-metric keys"
    python3 "$here/keydiff.py" "$here/base_metrics.txt" "$here/tip_metrics.txt"
    echo
    echo "== workload, 878 TUs by name"
    python3 "$here/verdicts.py" "$here/base_scan.jsonl" "$here/tip_scan.jsonl"
    echo
    echo "== fixtures at /O1"
    python3 "$here/verdicts.py" "$here/fix_base_o1.jsonl" "$here/fix_tip_o1.jsonl"
    echo
    echo "== fixtures at /Ox"
    python3 "$here/verdicts.py" "$here/fix_base_ox.jsonl" "$here/fix_tip_ox.jsonl"
} > "$here/NEUTRALITY.txt"

wc -l "$here/FRONTIER.txt" "$here/CAUSES.txt" "$here/GLWALK.txt" \
      "$here/COUNTERFACTUAL.txt" "$here/NEUTRALITY.txt"
