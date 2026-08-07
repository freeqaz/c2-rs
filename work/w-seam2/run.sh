#!/bin/sh
# run.sh — grade every frozen w-seam2 GRID S cell, ONE DIRECTORY PER CELL (#1045).
#
# Two instruments, both at the WORKLOAD's own `/GR /O1 /Oi /EHsc` and never the
# harness `/Ox` (board #1112 — at `/Ox` a refusal on this checklist reads as PAID
# when it is genuinely unpaid):
#
#   * `c2rs gap`    — the whole-TU differential against real `c2.dll` under wibo.
#                     THE SOLE JUDGE.
#   * `c2rs census` — the class verdict and the first-refusal key.
#
# Usage:  sh work/w-seam2/run.sh <tag> [cell ...]
#         <tag> names the outputs so BEFORE and AFTER sit in the same directory
#         without either overwriting the other.
#
# Every cell prints a line even when it did not grade — an explicit NO-VERDICT /
# NO-DIFFERENTIAL rather than a blank that reads exactly like a clean run
# (STATUS.md trap 5).
set -eu
repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
grid="$repo_root/work/w-seam2/grid"
c2rs="$repo_root/target/release/c2rs"

tag="${1:?usage: run.sh <tag> [cell ...]}"
shift || true
cells="${*:-$(cd "$grid" && ls)}"

for cell in $cells; do
    d="$grid/$cell"
    src="$d/$cell.cpp"
    [ -f "$src" ] || { echo "$cell: NO SOURCE"; continue; }

    # RELATIVE path from the repo root. An ABSOLUTE one reaches cl.exe under wibo
    # untranslated ("D8003 missing source filename"), the capture fails, and a
    # grep for the verdict prints nothing — which reads like a clean run.
    rel="work/w-seam2/grid/$cell/$cell.cpp"
    ( cd "$repo_root" && "$c2rs" census "$rel" \
        --flags-file work/dc3-workload/flags.txt ) > "$d/census.$tag.txt" 2>&1 || true

    printf '%s\n' "$rel" > "$d/list.txt"
    "$c2rs" gap --list "$d/list.txt" --flags-file "$repo_root/work/dc3-workload/flags.txt" \
        --cwd "$repo_root" --jobs 1 > "$d/gap.$tag.txt" 2>&1 || true

    verdict="$(grep -E '^  \[1/1\] ' "$d/gap.$tag.txt" | head -1 \
               | sed -E 's/^  \[1\/1\] +([a-z-]+) .*/\1/' || true)"
    key="$(grep -E '^ +[0-9]+ x [a-z]' "$d/census.$tag.txt" | head -1 \
           | sed -E 's/^ +[0-9]+ x //' || true)"
    inclass="$(grep -oE '[0-9]+/[0-9]+ functions in class' "$d/census.$tag.txt" | head -1 || true)"
    printf '%-28s %-22s %-28s %s\n' \
        "$cell" "${verdict:-NO-DIFFERENTIAL}" "${inclass:-NO-VERDICT}" "${key:-}"
done
