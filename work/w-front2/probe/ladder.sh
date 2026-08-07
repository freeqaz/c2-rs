#!/bin/sh
# ladder.sh — board #401's construct ladder for `xboxheap.cpp`, re-run at THIS
# master, one directory per cell (board #1045: four cells sharing one PID-keyed
# temp dir raced and fabricated a finding).
#
# Each cell is graded twice:
#   * `c2rs census --flags-file` — is the body in class, and if not, where does
#     the reader stop;
#   * `c2rs gap --list <one-line>` — the WHOLE-TU differential against real
#     `c2.dll` under wibo at the workload's own flags. That is the sole judge.
set -eu
repo_root="$(cd "$(dirname "$0")/../../.." && pwd)"
here="$repo_root/work/w-front2/probe"
c2rs="$repo_root/target/release/c2rs"

for cell in "$@"; do
    d="$here/$cell"
    [ -f "$d/$cell.cpp" ] || { echo "$cell: NO SOURCE"; continue; }
    printf '== %s\n' "$cell"
    # RELATIVE path, from the repo root. An ABSOLUTE one reaches `cl.exe` under
    # wibo untranslated and it answers `D8003 missing source filename` — the
    # capture fails, `census.txt` holds an error, and a grep for the verdict line
    # prints NOTHING, which reads exactly like a clean run. Trap 5 on this page:
    # absence reads as success unless something forbids it. Hence the explicit
    # NO-VERDICT below.
    ( cd "$repo_root" && "$c2rs" census "work/w-front2/probe/$cell/$cell.cpp" \
        --flags-file work/dc3-workload/flags.txt ) > "$d/census.txt" 2>&1 || true
    grep -E 'functions in class' "$d/census.txt" || echo "  NO-VERDICT (census did not grade — read $d/census.txt)"
    grep -E '^  \[' "$d/census.txt" | head -4 || true
    grep -A1 -E '^ +[0-9]+ x ' "$d/census.txt" | head -4 || true
    printf 'work/w-front2/probe/%s/%s.cpp\n' "$cell" "$cell" > "$d/list.txt"
    "$c2rs" gap --list "$d/list.txt" --flags-file "$repo_root/work/dc3-workload/flags.txt" \
        --cwd "$repo_root" --jobs 1 > "$d/gap.txt" 2>&1 || true
    grep -E '^  \[1/1\]|match [0-9]+' "$d/gap.txt" | head -3 || true
done
