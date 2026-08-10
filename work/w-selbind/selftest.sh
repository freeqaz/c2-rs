#!/bin/sh
# `c2rs selftest` under this lane's toolchain env, log kept.
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
. "$here/env.sh"
cd "$repo"
"$repo/target/release/c2rs" selftest > "$here/selftest.log" 2>&1 || true
tail -4 "$here/selftest.log"
