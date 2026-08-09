#!/bin/sh
# Grade ONE fixture at a chosen mode through `c2rs gap` — `c2rs diff` hardcodes
# the `/Ox /GS- /c` fixture profile and the classes this lane ships are `/O1`.
#
#     work/w-xtea3/one.sh /O1 fixtures/cpp/wxtea2_memcpy_tail.cpp
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
. "$here/env.sh"
mode="$1"
shift
work="$here/out/one"
mkdir -p "$work"
echo "$mode /GS- /c" > "$work/flags.txt"
: > "$work/list.txt"
for f in "$@"; do
    printf 'z:%s\n' "$(printf '%s' "$repo/$f" | tr '/' '\\')" >> "$work/list.txt"
done
"$repo/target/release/c2rs" gap --list "$work/list.txt" --flags-file "$work/flags.txt" \
    --jobs 1 --no-cache --jsonl "$work/scan.jsonl" 2>&1 | sed -n '/GAP REPORT/,$p' | head -40
