#!/bin/sh
# excerpt.sh — pull the hand-verified COMDATs out of a full `gt_dump.py` dump.
#
# Lane w-seq measurement tooling. The full dumps are 1–2 MB per workload TU and
# are gitignored; the four bodies the rung quotes are a few dozen lines and are
# committed, so §4.4 can be checked without recompiling. Reproduce the dumps with
#
#   c2rs compile <tu> --keep-obj work/w-seq/caps/<name>.obj \
#        --flags-file work/dc3-workload/flags.txt --cwd <dc3>
#   scripts/gt_dump.py work/w-seq/caps/<name>.obj --text-only
#
# Usage:  excerpt.sh <dump.txt> <symbol-substring> [<symbol-substring>...]
set -eu
dump="$1"
shift
for sym in "$@"; do
    n="$(grep -n -- "$sym" "$dump" | grep '^[0-9]*:-- \.text' | head -1 | cut -d: -f1)"
    [ -n "$n" ] || { echo "NOT FOUND: $sym"; continue; }
    end="$n"
    while :; do
        end=$((end + 1))
        line="$(sed -n "${end}p" "$dump")"
        case "$line" in
            "-- .text"* | "") break ;;
        esac
    done
    sed -n "${n},$((end - 1))p" "$dump"
done
