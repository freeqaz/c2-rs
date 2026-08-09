#!/bin/sh
# The full merge gate with this lane's toolchain env resolved.
#
#     work/w-xtea3/gate.sh
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
. "$here/env.sh"
export C2RS_MODE_LANE_WORK="$here/out/gate"
exec "$repo/scripts/gate.sh" --jobs 4 "$@"
