#!/bin/sh
# Regenerate `docs/STATUS.md`'s metric block at this lane's tree, with the
# worktree's toolchain env. NEVER hand-edit that block (CLAUDE.md).
#
#     work/w-decouple/status.sh
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
. "$here/env.sh"
cd "$repo"
# `work/dc3-workload` is gitignored and lives in the MAIN checkout, so a
# worktree needs C2RS_WORKLOAD pointed at it — otherwise the workload rows
# render NO-RESULT rather than a number (which is the collector working).
C2RS_WORKLOAD="$(dirname "$WD_FILES")" sh scripts/status.sh --write > "$here/status.log" 2>&1 || true
tail -20 "$here/status.log"
