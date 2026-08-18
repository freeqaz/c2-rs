#!/usr/bin/env bash
# `w-witness7` — the gate and the 878-TU scan, logged, one tag per end.
#
#   corpus.sh <tag>
#
# The suite is `campaign.sh`'s job (it needs to bracket a patch); this runs the
# two things a lane reports at BOTH ends and nothing else.
#
# The dc3 tree is resolved relative to the MAIN repo, never absolutely
# (CLAUDE.md: no absolute machine paths in the tree) — in a worktree the repo
# root is the worktree, so the sibling lookup starts from the common git dir.
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
TAG="${1:?usage: corpus.sh <tag>}"
OUT="$ROOT/work/w-witness7/logs"
mkdir -p "$OUT"
cd "$ROOT" || exit 1

export C2RS_REQUIRE_TOOLCHAIN=1
MAIN_REPO="$(cd "$(git -C "$ROOT" rev-parse --git-common-dir)/.." && pwd)"
export C2RS_DC3="${C2RS_DC3:-$MAIN_REPO/../dc3-decomp}"

echo "=== $TAG: gate ==="
t0=$SECONDS
scripts/gate.sh --jobs 16 --require-graded > "$OUT/$TAG.gate.log" 2>&1
echo "gate exit=$? wall=$((SECONDS - t0))s"

echo "=== $TAG: 878-TU scan ==="
t0=$SECONDS
./target/release/c2rs gap --list work/dc3-workload/files.txt \
    --flags-file work/dc3-workload/flags.txt --cwd "$C2RS_DC3" --jobs 16 \
    > "$OUT/$TAG.scan.log" 2>&1
echo "scan exit=$? wall=$((SECONDS - t0))s"

# The prefix-anchored key count. NEVER the naive `grep -c 'gap-metric'`, which
# reads 396 because two prose lines merely mention keys (#3269) — and which
# caught three consecutive lanes, the third inventing a cause for the +2.
# The per-lane count table, for the identity diff. Extracted by PARSING the
# block between the dashed header and the blank line (#3288: parse rather than
# grep anything you enumerate), and its LENGTH is printed so a diff of two
# EMPTY ranges cannot read as agreement (#3215).
awk '/^-{20} /{on=1;next} on&&NF==0{on=0} on' "$OUT/$TAG.gate.log" > "$OUT/$TAG.lanes.txt"
echo -n "gate lane rows: "; wc -l < "$OUT/$TAG.lanes.txt"

echo -n "gap-metric keys (anchored): "
grep -cE '^ *gap-metric \S+ \S+$' "$OUT/$TAG.scan.log"
grep -E '^ *gap-metric \S+ \S+$' "$OUT/$TAG.scan.log" | sort > "$OUT/$TAG.keys.txt"
grep -E '^(match|mismatch) |^  (match|mismatch) ' "$OUT/$TAG.scan.log" | head -4
grep -E 'gap-metric fnbyte-(exact|differs|refused-parse) ' "$OUT/$TAG.scan.log"
