#!/bin/sh
# tally.sh — the per-case Match / NotImplemented split of ONE sweep fragment.
#
# `scripts/expr_sweep.sh` prints `checked=/mismatches=/graded=/ungraded=` and
# deliberately does NOT print the Match-vs-NotImplemented split: both are
# "not an alarm", so the sweep classifies them into one arm. But the split is
# exactly what a lane must quote at BOTH ends (board #1205 — a lane that
# tallies only at its tip books conversions it did not cause), so it is
# re-derived here by re-running `c2rs diff` over the fragment's own generated
# cases, using the binary the sweep PINNED (never `target/release/c2rs`, which
# another process may republish mid-run).
#
# Usage:  sh work/w-self2b/tally.sh <sweep-outdir> <jobs> > out.txt
#
# Prints one `Port=<verdict> <file>` line per case and a final TALLY line.
set -eu
dir="${1:?usage: tally.sh <sweep-outdir> [jobs]}"
jobs="${2:-6}"
c2rs="$dir/c2rs"
[ -x "$c2rs" ] || { echo "FAIL: no pinned harness at $c2rs"; exit 1; }

tmp="$dir/.tally"
rm -rf "$tmp"; mkdir -p "$tmp"
ls "$dir"/*.cpp > "$tmp/all"
total=$(wc -l < "$tmp/all")
[ "$total" -gt 0 ] || { echo "FAIL: 0 cases in $dir"; exit 1; }

split -n "l/$jobs" "$tmp/all" "$tmp/chunk."
w=0
for c in "$tmp"/chunk.*; do
    w=$((w + 1))
    (
        while read -r f; do
            v=$("$c2rs" diff "$f" 2>&1 | tail -1)
            case "$v" in
                *"Port=Match"*)          echo "Port=Match $f" ;;
                *"Port=NotImplemented"*) echo "Port=NotImplemented $f" ;;
                *"Port=Mismatch"*)       echo "Port=Mismatch $f" ;;
                *)                       echo "Port=OTHER($v) $f" ;;
            esac
        done < "$c"
    ) > "$tmp/out.$w" &
done
wait
cat "$tmp"/out.*

m=$(cat "$tmp"/out.* | grep -c '^Port=Match ' || true)
n=$(cat "$tmp"/out.* | grep -c '^Port=NotImplemented ' || true)
x=$(cat "$tmp"/out.* | grep -c '^Port=Mismatch ' || true)
o=$(cat "$tmp"/out.* | grep -c '^Port=OTHER' || true)
seen=$(cat "$tmp"/out.* | wc -l)
echo "TALLY dir=$dir cases=$total seen=$seen Match=$m NotImplemented=$n Mismatch=$x OTHER=$o"
[ "$seen" -eq "$total" ] || { echo "FAIL: seen != cases"; exit 1; }
