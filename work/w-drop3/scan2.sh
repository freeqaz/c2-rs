#!/bin/sh
set -e
here=$(cd "$(dirname "$0")/../.." && pwd)
sib=$(cd "$here/../../.." 2>/dev/null && pwd || echo "$here/..")
export C2RS_WIBO=${C2RS_WIBO:-$sib/wibo/build/wibo}
export C2RS_GAP_CACHE=${C2RS_GAP_CACHE:-$sib/c2-rs/work/capture-cache}
./target/release/c2rs gap --list work/dc3-workload/files.txt \
  --flags-file work/dc3-workload/flags.txt --cwd "${C2RS_DC3:-$sib/dc3-decomp}" \
  --jobs 12 --jsonl work/w-drop3/rows.jsonl > work/w-drop3/scan-jsonl.txt 2>&1
echo done
