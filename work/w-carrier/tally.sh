#!/bin/sh
# tally.sh — the PORT's own verdict split over a named sweep fragment, which
# `expr_sweep.sh` deliberately does not record.
#
#   sh work/w-carrier/tally.sh <case-dir> <fragment-prefix> <binary> <out>
#
# `mismatches=0` alone does not distinguish RIGHT from SILENT: a widening that
# converts `Port=NotImplemented` into `Port=Match` and one that converts nothing
# print the same sweep line. Board #1205: a lane that tallies only at its TIP
# books conversions it did not cause, so this is run at BOTH ends with a base
# binary built from the lane's base commit IN THIS TREE.
#
# Generalised from `work/w-f23/tally88.sh`, which hard-codes the 88 fragment;
# board #1189 added `89-store-run-live-arg` and it must be tallied too.
set -eu
dir="${1:?usage: tally.sh <case-dir> <prefix> <binary> <out>}"
prefix="${2:?usage: tally.sh <case-dir> <prefix> <binary> <out>}"
bin="${3:?usage: tally.sh <case-dir> <prefix> <binary> <out>}"
out="${4:?usage: tally.sh <case-dir> <prefix> <binary> <out>}"
: > "$out"
ls "$dir"/"$prefix"-*.cpp | xargs -P 16 -I{} sh -c \
    'printf "%s %s\n" "$("'"$bin"'" diff "$1" 2>&1 | tail -1 | grep -oE "Port=[A-Za-z]+" || echo Port=NONE)" "$1"' _ {} \
    >> "$out"
awk '{print $1}' "$out" | sort | uniq -c
