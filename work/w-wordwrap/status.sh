#!/bin/sh
# `scripts/status.sh` with this lane's toolchain env resolved.
#
# A worktree has no `./compilers`, no `../wibo` and no `../dc3-decomp`, and
# `status.sh` reads all three through `Toolchain::locate`'s relative defaults —
# so run without them it writes a STATUS block that is NO-RESULT on every
# toolchain-dependent row, which is a WORSE document than the one it replaced.
# Two such writes were made and reverted before this wrapper existed.
#
# `work/dc3-workload/` is symlinked into the worktree from the main checkout for
# the same reason (it is gitignored, so the link is never committed).
#
#     work/w-wordwrap/status.sh --write --tests-log work/w-wordwrap/tests_tip.txt
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
. "$here/env.sh"
exec sh "$repo/scripts/status.sh" "$@"
