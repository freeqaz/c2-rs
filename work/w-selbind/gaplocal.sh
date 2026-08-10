#!/bin/sh
# `c2rs gap` on a LOCAL cpp (cwd = the repo), so `gate_cause` is readable for a
# probe cell as well as for a workload TU.
#
#     sh work/w-selbind/gaplocal.sh <tag> <path-relative-to-repo> [flags-file] [binary]
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
. "$here/env.sh"
tag="$1"
src="$2"
flags="${3:-}"
bin="${4:-$repo/target/release/c2rs}"
printf '%s\n' "$src" > "$here/loc_$tag.txt"
if [ -n "$flags" ]; then
    "$bin" gap --list "$here/loc_$tag.txt" --flags-file "$flags" --cwd "$repo" \
        --jsonl "$here/loc_$tag.jsonl" > "$here/loc_$tag.log" 2>&1 || true
else
    "$bin" gap --list "$here/loc_$tag.txt" --cwd "$repo" \
        --jsonl "$here/loc_$tag.jsonl" > "$here/loc_$tag.log" 2>&1 || true
fi
grep -oE '"gate_cause":[^,]*|"gate_causes":\[[^]]*\]|"class":"[^"]*"|"fn_total":[0-9]*|"fn_names":[0-9]*|"gl_body_starts":\[[^]]*\]' \
    "$here/loc_$tag.jsonl" || true
