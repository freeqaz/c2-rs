#!/bin/sh
# w-drop3 lane scratch: one 878-TU workload scan.
#
#   sh work/w-drop3/scan.sh <tag> [--jsonl]
#
# Writes work/w-drop3/scan-<tag>.txt, plus fndiff-<tag>.jsonl (default) or
# rows-<tag>.jsonl (--jsonl, the per-TU emit maps).
#
# Every path is env-driven with a repo-relative default, per CLAUDE.md: no
# absolute machine path lives in a committed file.
#   C2RS_WIBO      wibo binary          (default <milohax>/wibo/build/wibo)
#   C2RS_GAP_CACHE shared capture cache (default <main repo>/work/capture-cache)
#   C2RS_DC3       the dc3 tree         (default <milohax>/dc3-decomp)
#
# A worktree sits at <main repo>/.claude/worktrees/<name>, so the main repo is
# THREE levels up from it and the milohax parent is FOUR. Getting that wrong is
# silent: the toolchain simply does not resolve and the scan prints
# `SKIP: toolchain absent` and exits 0 — which is `docs/STATUS.md` trap 5, and
# this script shipped with the off-by-one before it was caught by an empty
# output file.
set -e
here=$(cd "$(dirname "$0")/../.." && pwd)     # the worktree (or repo) root
# The MAIN repo, derived from git rather than by counting "..": a worktree sits
# at <repo>/.claude/worktrees/<name>, which is three levels, and hand-counting
# it is what shipped the first version of this script broken.
repo=$(cd "$(git rev-parse --git-common-dir)/.." && pwd)
sib=$(cd "$repo/.." && pwd)                                   # its milohax parent
export C2RS_WIBO=${C2RS_WIBO:-$sib/wibo/build/wibo}
export C2RS_GAP_CACHE=${C2RS_GAP_CACHE:-$repo/work/capture-cache}
[ -x "$C2RS_WIBO" ] || { echo "no wibo at $C2RS_WIBO — set C2RS_WIBO"; exit 2; }

out="$1"
case "$2" in
  --jsonl) rows="--jsonl work/w-drop3/rows-$out.jsonl" ;;
  *)       rows="--fnbyte-diff-jsonl work/w-drop3/fndiff-$out.jsonl" ;;
esac

./target/release/c2rs gap \
  --list work/dc3-workload/files.txt \
  --flags-file work/dc3-workload/flags.txt \
  --cwd "${C2RS_DC3:-$sib/dc3-decomp}" \
  --jobs "${C2RS_JOBS:-12}" \
  $rows \
  > "work/w-drop3/scan-$out.txt" 2>&1

# A scan that resolved no toolchain exits 0 and grades nothing. Say so.
if grep -q 'SKIP: toolchain absent' "work/w-drop3/scan-$out.txt"; then
  echo "SKIP: toolchain absent — work/w-drop3/scan-$out.txt graded NOTHING"
  exit 3
fi
echo "ok -> work/w-drop3/scan-$out.txt"
