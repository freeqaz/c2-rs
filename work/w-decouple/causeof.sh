#!/bin/sh
# The gate's own cause set for ONE cell, at a NAMED binary and a NAMED mode —
# so "which clause refused this" is a reading and not an inference from
# `Port=NotImplemented`.
#
#     work/w-decouple/causeof.sh <base|tip> <cell.cpp> <o1|ox>
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
. "$here/env.sh"
which="$1"
src="$2"
mode="${3:-ox}"
case "$mode" in
    o1) flags="/O1 /GS- /c" ;;
    ox) flags="/Ox /GS- /c" ;;
esac
printf '%s\n' "$flags" > "$here/one_flags.txt"
printf '%s\n' "$src" > "$here/one_list.txt"
"$here/c2rs-$which" gap --list "$here/one_list.txt" --flags-file "$here/one_flags.txt" \
    --cwd "$repo" --jsonl "$here/one.jsonl" > "$here/one.log" 2>&1
python3 "$here/rowfields.py" "$here/one.jsonl"
