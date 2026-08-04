#!/bin/sh
# $1 = source path relative to dc3 root
# lane root: override with C2RS_LANE_ROOT; default = three levels up from this script
W="${C2RS_LANE_ROOT:-$(cd "$(dirname "$0")/../../.." && pwd)}"
DC3="${C2RS_DC3_SRC:?set C2RS_DC3_SRC to the dc3-decomp source tree}"
name=$(printf '%s' "$1" | tr '/' '_')
out="$W/work/w-bss/census/objs/$name.obj"
[ -s "$out" ] && exit 0
cd "$DC3" || exit 1
"$W/target/release/c2rs" compile "$1" --cwd "$DC3" --flags-file "$W/work/dc3-workload/flags.txt" --keep-obj "$out" >/dev/null 2>"$W/work/w-bss/census/objs/$name.err"
if [ -s "$out" ]; then rm -f "$W/work/w-bss/census/objs/$name.err"; else echo "FAIL $1"; fi
