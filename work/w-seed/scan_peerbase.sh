#!/bin/sh
# scan_peerbase.sh — one 878-TU scan with the LANE-BASE binary.
#
# The base end of `work/w-splice/peerkeys.py` has to be a binary built from the
# commit this lane branched off, and it must not share a `target/` with the tip
# binary or the two builds clobber each other and the "comparison" is one binary
# run twice. `work/w-seed/peerbase/` is a `git archive` of 29dab722 with its own
# target dir.
set -eu
WT=$(cd "$(dirname "$0")/../.." && pwd)
. "$WT/work/w-seed/env.sh"
out="$1"
shift
"$WT/work/w-seed/peerbase/target/release/c2rs" gap \
    --list "$C2RS_WORKLOAD/files.txt" \
    --flags-file "$C2RS_WORKLOAD/flags.txt" \
    --cwd "$C2RS_DC3" \
    --jobs 12 "$@" > "$out.txt" 2>&1
echo "EXIT=$? -> $out.txt"
