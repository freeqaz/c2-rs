#!/bin/sh
# Run the freshly-built c2rs with this lane's toolchain env and the workload's
# own flags/cwd already applied.  Everything after the subcommand is passed
# through.
#
#     work/w-xtea3/c2rs.sh census src/system/utl/EncryptXTEA.cpp
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
. "$here/env.sh"
exec "$repo/target/release/c2rs" "$@" \
    --flags-file "$C2RS_MAIN/work/dc3-workload/flags.txt" \
    --cwd "$C2RS_DC3"
