#!/bin/sh
# census_refs.sh — the PORT's own first-refusal key, per function, per FRONTIER TU,
# at the workload's own flags.
#
# This is the instrument that says where `c2-il` stops reading, which is a
# different question from "what instruction does c2 emit that the port cannot".
# Both are needed: the census key is a LOWER bound on the refusals (it names only
# the FIRST stop — board #1091), the disassembly is the upper one.
set -eu
repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
out_root="$repo_root/work/w-front2/ref"
: "${C2RS_DC3:?set C2RS_DC3}"

while read -r src; do
    [ -n "$src" ] || continue
    key="$(printf '%s' "$src" | tr '/' '_')"
    d="$out_root/$key"
    mkdir -p "$d"
    ( cd "$C2RS_DC3" && "$repo_root/target/release/c2rs" census "$src" \
        --flags-file "$repo_root/work/dc3-workload/flags.txt" ) \
        > "$d/census.txt" 2>&1 || true
    echo "== $src"
    grep -E 'functions in class|^ *[0-9]+ ' "$d/census.txt" | head -30
done < "$repo_root/work/w-front2/tus.txt"
