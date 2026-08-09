#!/bin/sh
# w-callprice — the workspace test totals, summed the way every rung quotes them.
set -eu
here=$(cd "$(dirname "$0")/../.." && pwd)
cd "$here"
cargo test --workspace --release 2>&1 | tee work/w-callprice/tests_raw.txt |
    grep -E '^test result' |
    awk '{p+=$4; f+=$6; n++} END {printf "%d passed, %d failed, %d targets\n", p, f, n}'
