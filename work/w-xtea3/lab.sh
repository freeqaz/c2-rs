#!/bin/sh
# Run this lane's label grid with the toolchain env resolved the same way every
# other script here does.
#
#     work/w-xtea3/lab.sh [probe ...]
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
. "$here/env.sh"
exec python3 "$here/labgrid.py" "$@"
