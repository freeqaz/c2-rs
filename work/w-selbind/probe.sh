#!/bin/sh
# Run a c2rs subcommand (or any command) under this lane's toolchain env.
#
#     sh work/w-selbind/probe.sh diff fixtures/cpp/mvp_empty.cpp
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
. "$here/env.sh"
exec "$repo/target/release/c2rs" "$@"
