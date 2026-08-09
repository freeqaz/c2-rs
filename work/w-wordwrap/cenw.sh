#!/bin/sh
# Per-function census of one WORKLOAD TU at the workload's own flags and cwd —
# `cen.sh` is for lane-local probes and fixtures, which are compiled without the
# eight `/I` roots.
#
#     work/w-wordwrap/cenw.sh src/system/rndobj/wordwrap.cpp
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
. "$here/env.sh"
exec "$repo/target/release/c2rs" census "$@" \
    --flags-file "$C2RS_MAIN/work/dc3-workload/flags.txt" \
    --cwd "$C2RS_DC3"
