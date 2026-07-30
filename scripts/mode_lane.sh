#!/bin/sh
# Run every fixture through the differential at a chosen optimization mode.
#
# `c2rs diff` hardcodes the default `/Ox /GS- /c` capture profile, so the fixture
# suite has only ever verified the port against **`/Ox`** — while the dc3 workload
# compiles `/O1`, which emits different code for the same source
# (`docs/OPT_MODE.md`). This is the missing lane: it drives the fixtures through
# `c2rs gap`, which does take `--flags-file`, so the same corpus can be graded in
# either mode.
#
# Usage:  scripts/mode_lane.sh [/O1|/Ox|/O2|/Od] [extra cl flags...]
#         scripts/mode_lane.sh /O1
#
# `mismatch` is the alarm: it means the port emitted bytes for a mode and they were
# wrong. `codegen-gap` is the honest refusal — a shape not yet re-targeted for that
# mode. Exits non-zero on any mismatch.
set -eu

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
mode="${1:-/O1}"
[ $# -gt 0 ] && shift
c2rs="$repo_root/target/release/c2rs"

if [ ! -x "$c2rs" ]; then
    echo "building the harness first"
    (cd "$repo_root" && cargo build --release -p c2-harness)
fi

work="${C2RS_MODE_LANE_WORK:-/tmp/c2rs-mode-lane}"
mkdir -p "$work"
flags="$work/flags.txt"
list="$work/list.txt"
echo "$mode /GS- /c $*" > "$flags"

# `cl.exe` runs under wibo, so the sources have to be named as `Z:\…` paths.
: > "$list"
for f in "$repo_root"/fixtures/cpp/*.cpp; do
    printf 'z:%s\n' "$(printf '%s' "$f" | tr '/' '\\')" >> "$list"
done
total=$(wc -l < "$list")

if "$c2rs" gap --list "$list" --flags-file "$flags" --limit 1 --jobs 1 2>&1 | grep -q "SKIP"; then
    echo "SKIP: toolchain absent — the lane would be vacuous"
    exit 0
fi

echo "grading $total fixtures at $mode"
out="$work/report.txt"
"$c2rs" gap --list "$list" --flags-file "$flags" --jobs "${C2RS_JOBS:-8}" \
    --jsonl "$work/scan.jsonl" > "$out" 2>&1 || true
sed -n '/GAP REPORT/,$p' "$out"

mm=$(sed -n 's/^  mismatch  *\([0-9]*\) .*/\1/p' "$out" | head -1)
[ "${mm:-0}" -eq 0 ] || {
    echo
    echo "MISMATCH at $mode — the port emitted wrong bytes, not a gap:"
    grep -F "mismatch" "$out" | grep -v "^  mismatch" || true
    exit 1
}
