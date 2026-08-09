#!/bin/sh
# The BYTE judge over this lane's own fixtures, at one mode.
#
# The class this lane ships CANNOT reach TU `match` — its object is a
# non-COMDAT `.bss` the writer refuses by name — so the fixture verdict column
# says nothing about it and `fnbyte-exact` is the whole grading. This prints
# that column per fixture, from real c2's own obj: bytes AND all four
# relocation records.
#
#     work/w-wordwrap/fxbyte.sh "/O1 /Oi" o1oi fixtures/cpp/wwrap_gstore.cpp …
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
. "$here/env.sh"
mode="$1"
tag="$2"
shift 2
work="$here/out/fxbyte-$tag"
mkdir -p "$work"
echo "$mode /GS- /c" > "$work/flags.txt"
: > "$work/list.txt"
for f in "$@"; do
    printf 'z:%s\n' "$(printf '%s' "$repo/$f" | tr '/' '\\')" >> "$work/list.txt"
done
"$repo/target/release/c2rs" gap --list "$work/list.txt" --flags-file "$work/flags.txt" \
    --jobs 4 --no-cache --jsonl "$work/scan.jsonl" > "$work/report.txt" 2>&1
python3 - "$work/scan.jsonl" "$tag" <<'PY'
import json, sys, os
for line in open(sys.argv[1]):
    d = json.loads(line)
    if "emit" not in d:
        continue
    e = d["emit"]
    print(
        f"{sys.argv[2]:6s} {os.path.basename(d['src'].replace(chr(92), '/')):34s}"
        f" {d['class']:11s}"
        f" fnbyte den {e.get('fnbyte-denominator', 0)}"
        f"  exact {e.get('fnbyte-exact', 0)}"
        f"  differs {e.get('fnbyte-differs', 0)}"
        f"  refused {e.get('fnbyte-refused', 0)}"
    )
PY
