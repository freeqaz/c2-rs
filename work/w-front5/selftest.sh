#!/bin/sh
# The oracle self-test (determinism + capture stability) at this lane's tree.
#
#     work/w-front5/selftest.sh
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
. "$here/env.sh"
cd "$repo"
"$here/c2rs-tip" selftest > "$here/selftest.log" 2>&1 || true
tail -6 "$here/selftest.log"
