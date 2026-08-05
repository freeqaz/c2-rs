#!/bin/sh
# one.sh — run the gap scan over a single TU and dump its JSONL record.
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
root="$here/../.."
src="$1"
n=$(basename "$src" .cpp)
printf '%s\n' "$src" > "$here/one_$n.txt"
"$root/target/release/c2rs" gap \
    --list "$here/one_$n.txt" \
    --flags-file "$root/work/dc3-workload/flags.txt" \
    --cwd "${C2RS_DC3:-/home/free/code/milohax/dc3-decomp}" \
    --jsonl "$here/one_$n.jsonl" \
    --jobs 1 > "$here/one_$n.log" 2>&1
echo "== $n"
