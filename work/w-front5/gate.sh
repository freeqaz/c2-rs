#!/bin/sh
# The full merge gate at this lane's tree, with the worktree's toolchain env.
#
#     work/w-front5/gate.sh
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
. "$here/env.sh"
cd "$repo"
sh scripts/gate.sh > "$here/gate.log" 2>&1 || true
tail -40 "$here/gate.log"
