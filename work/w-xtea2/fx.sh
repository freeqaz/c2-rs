#!/bin/sh
# `c2rs` with this lane's toolchain env and NOTHING else — for the subcommands
# (`diff`, `selftest`, `perf`) that carry their own fixture profile.
#
#     work/w-xtea2/fx.sh diff fixtures/cpp/wxtea2_memcpy_tail.cpp
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
. "$here/env.sh"
exec "$repo/target/release/c2rs" "$@"
