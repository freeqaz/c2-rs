#!/bin/sh
# w-drop3 lane scratch: one 878-TU workload scan, output named by $1.
#
# Every path is env-driven with a repo-relative sibling default, per CLAUDE.md:
# no absolute machine path lives in a committed file.
#   C2RS_WIBO      wibo binary            (default ../wibo/build/wibo)
#   C2RS_GAP_CACHE shared capture cache   (default ../c2-rs/work/capture-cache)
#   C2RS_DC3       the dc3 tree           (default ../dc3-decomp)
set -e
here=$(cd "$(dirname "$0")/../.." && pwd)          # the worktree root
sib=$(cd "$here/../../.." 2>/dev/null && pwd || echo "$here/..")  # its milohax parent
export C2RS_WIBO=${C2RS_WIBO:-$sib/wibo/build/wibo}
export C2RS_GAP_CACHE=${C2RS_GAP_CACHE:-$sib/c2-rs/work/capture-cache}
out="$1"
./target/release/c2rs gap \
  --list work/dc3-workload/files.txt \
  --flags-file work/dc3-workload/flags.txt \
  --cwd "${C2RS_DC3:-$sib/dc3-decomp}" \
  --jobs "${C2RS_JOBS:-12}" \
  --fnbyte-diff-jsonl "work/w-drop3/fndiff-$out.jsonl" \
  > "work/w-drop3/scan-$out.txt" 2>&1
echo "EXIT=$? -> work/w-drop3/scan-$out.txt"
