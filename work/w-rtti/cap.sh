#!/bin/bash
# cap.sh — capture the frozen w-rtti grid through the REAL toolchain.
#
#   work/w-rtti/cap.sh [<flags-file>] [<out-subdir>]
#
# Defaults to the workload's own profile minus its include paths
# (`work/w-rtti/flags_probe.txt`) and writes to `work/w-rtti/obj/`.
# Every obj goes in ONE directory (the `w-ilx` rule: one directory for
# byte-diffed captures, so nothing is diffed across two source paths).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
FLAGS="${1:-$ROOT/work/w-rtti/flags_probe.txt}"
OUT="${2:-$ROOT/work/w-rtti/obj}"
C2RS="$ROOT/target/release/c2rs"

[ -x "$C2RS" ] || { echo "SKIP: $C2RS not built"; exit 0; }
mkdir -p "$OUT"

n=0
for src in $(cat "$ROOT/work/w-rtti/grid_list.txt"); do
    base="${src%.cpp}"
    # A relative source path: `cl` runs under wibo and does not take a POSIX
    # absolute path as a filename (it reads it as an unknown option and then
    # reports "missing source filename").
    (cd "$ROOT" && "$C2RS" compile "work/w-rtti/grid/$src" \
        --keep-obj "$OUT/$base.obj" --flags-file "$FLAGS" >/dev/null)
    n=$((n + 1))
done
echo "captured=$n into $OUT with $(basename "$FLAGS")"
ls "$OUT" | grep -c '\.obj$' | sed 's/^/objs=/'
