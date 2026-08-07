#!/bin/sh
# run_cells.sh — compile and grade GRID-W against real c2 under wibo.
#
# Lane w-target measurement tooling. The cell sources are relative to the repo
# root, so `--cwd` is the repo root and not the dc3 tree: every path the scan
# hands `cl.exe` has to resolve from there.
#
# Usage:  work/w-target/run_cells.sh [outdir-tag]
set -eu
root="$(cd "$(dirname "$0")/../.." && pwd)"
tag="${1:-cells}"
cd "$root"
ls work/w-target/cells/*.cpp > "work/w-target/$tag.txt"
./target/release/c2rs gap \
    --list "work/w-target/$tag.txt" \
    --flags-file work/w-target/flags.txt \
    --cwd "$root" \
    --jobs 8 \
    --jsonl "work/w-target/$tag.jsonl" > "work/w-target/$tag.log" 2>&1
echo "exit=$?"
grep -E '^ *(match|mismatch|codegen-gap|vocab-gap|capture-fail) ' "work/w-target/$tag.log" || true
