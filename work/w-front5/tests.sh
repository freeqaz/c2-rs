#!/bin/sh
# The workspace suite at the lane's exact tree, with the toolchain env this
# worktree needs. `#2262` requires `--release --no-fail-fast`; `hatch-red`
# refuses on PRE-EXISTING failures, so this is run at the base tree before
# anything is attributed to the lane.
#
#     work/w-front5/tests.sh
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
. "$here/env.sh"
cd "$repo"
cargo test --workspace --release --no-fail-fast > "$here/tests.log" 2>&1 || true
grep -E '^test result' "$here/tests.log" | tail -60
echo "--- totals ---"
awk '/^test result/ { for (i = 1; i <= NF; i++) {
        if ($i == "passed;") p += $(i-1);
        if ($i == "failed;") f += $(i-1);
     } n++ }
     END { printf "%d passed, %d failed, %d targets\n", p, f, n }' "$here/tests.log"
