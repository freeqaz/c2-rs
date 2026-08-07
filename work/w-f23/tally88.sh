#!/bin/sh
# tally88.sh — the PORT's own verdict split over the 1,576 `88-store-run-call`
# cases, which `expr_sweep.sh` deliberately does not record.
#
# `mismatches=0` alone does not distinguish RIGHT from SILENT: a widening that
# converts `Port=NotImplemented` into `Port=Match` and one that converts nothing
# print the same sweep line. w-gen §5 published the baseline split
# (44 Match / 1,532 NotImplemented) for exactly this comparison.
#
# Usage:  sh work/w-f23/tally88.sh <case-dir> <out>
set -eu
repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
c2rs="${C2RS_BIN:-$repo_root/target/release/c2rs}"
dir="${1:?usage: tally88.sh <case-dir> <out>}"
out="${2:?usage: tally88.sh <case-dir> <out>}"
: > "$out"
ls "$dir"/88-store-run-call-*.cpp | xargs -P 16 -I{} sh -c \
    'printf "%s %s\n" "$("'"$c2rs"'" diff "$1" 2>&1 | tail -1 | grep -oE "Port=[A-Za-z]+" || echo Port=NONE)" "$1"' _ {} \
    >> "$out"
awk '{print $1}' "$out" | sort | uniq -c
