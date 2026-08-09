#!/bin/sh
# Per-function census of one source at a chosen mode.
#
#     work/w-xtea3/cen.sh o1 fixtures/cpp/wxtea2_memcpy_tail.cpp
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
. "$here/env.sh"
mode="$1"
shift
exec "$repo/target/release/c2rs" census "$@" --flags-file "$here/flags_$mode.txt"
