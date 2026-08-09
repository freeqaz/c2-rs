#!/bin/sh
# w-biquad — every fixture, at BOTH modes, base binary and tip binary, compared
# BY NAME.
#
# The list is REGENERATED here rather than reused, and its length is printed, so
# a fixture this lane added and did not scan cannot read as an unchanged count —
# which is how a fixture-level neutrality claim goes wrong.
#
#   fixscan.sh <out-stem> <binary> <mode>
set -e
OUT="work/w-biquad/$1"
BIN="$2"
MODE="$3"
: > "$OUT.list"
for f in fixtures/cpp/*.cpp; do
    printf 'z:%s\n' "$(cd "$(dirname "$0")/../.." && pwd)/$f" | tr '/' '\\' >> "$OUT.list"
done
# `tr` above also mangled the `z:` prefix's slash-free head; rewrite cleanly.
python3 - "$OUT.list" <<'PY'
import os, sys
out = sys.argv[1]
root = os.getcwd()
rows = []
for f in sorted(os.listdir("fixtures/cpp")):
    if f.endswith(".cpp"):
        rows.append("z:" + os.path.join(root, "fixtures/cpp", f).replace("/", "\\"))
open(out, "w").write("\n".join(rows) + "\n")
print(f"{len(rows)} fixtures")
PY
wc -l "$OUT.list"
echo "$MODE /GS- /c" > "$OUT.flags"
"$BIN" gap --list "$OUT.list" --flags-file "$OUT.flags" --jobs 12 \
    --jsonl "$OUT.jsonl" > "$OUT.out" 2>&1
echo "done $OUT"
