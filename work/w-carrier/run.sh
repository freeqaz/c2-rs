#!/bin/sh
# run.sh — grade every frozen w-carrier GRID cell, ONE DIRECTORY PER CELL (#1045).
#
# Copied from `work/w-f23/run.sh` rather than transcribed, so the profile cannot
# drift. Two independent instruments, both at the WORKLOAD's own flags (board
# #1112 — at the harness default /Ox a refusal on this checklist reads as PAID
# when it is genuinely unpaid):
#
#   * `c2rs gap`    — the whole-TU differential against real `c2.dll` under
#                     wibo. THE SOLE JUDGE.
#   * `c2rs census` — the class verdict and the first-refusal key.
#
# Usage:  sh work/w-carrier/run.sh <tag> [cell ...]
set -eu
repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
grid="$repo_root/work/w-carrier/${GRID:-grid}"
c2rs="$repo_root/target/release/c2rs"

tag="${1:?usage: run.sh <tag> [cell ...]}"
shift || true
cells="${*:-$(cd "$grid" && ls)}"

for cell in $cells; do
    d="$grid/$cell"
    src="$d/$cell.cpp"
    [ -f "$src" ] || { echo "$cell: NO SOURCE"; continue; }
    printf '== %s\n' "$cell"

    # RELATIVE path from the repo root. An ABSOLUTE one reaches cl.exe under
    # wibo untranslated ("D8003 missing source filename"), the capture fails, and
    # a grep for the verdict prints nothing — which reads exactly like a clean
    # run. Hence the explicit NO-VERDICT line.
    rel="work/w-carrier/${GRID:-grid}/$cell/$cell.cpp"
    ( cd "$repo_root" && "$c2rs" census "$rel" \
        --flags-file work/dc3-workload/flags.txt ) > "$d/census.$tag.txt" 2>&1 || true
    grep -E 'functions in class' "$d/census.$tag.txt" \
        || echo "  NO-VERDICT (census did not grade — read $d/census.$tag.txt)"
    grep -E '^ +[0-9]+ x [a-z]' "$d/census.$tag.txt" | head -3 || true

    printf '%s\n' "$rel" > "$d/list.txt"
    "$c2rs" gap --list "$d/list.txt" --flags-file "$repo_root/work/dc3-workload/flags.txt" \
        --cwd "$repo_root" --jobs 1 > "$d/gap.$tag.txt" 2>&1 || true
    grep -E '^  \[1/1\]' "$d/gap.$tag.txt" | head -1 \
        || echo "  NO-DIFFERENTIAL (gap did not grade — read $d/gap.$tag.txt)"
done
