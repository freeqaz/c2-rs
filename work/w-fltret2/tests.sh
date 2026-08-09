#!/bin/sh
# w-fltret — the workspace test totals, summed from every target's result line.
# `$1` is the output stem under work/w-fltret2/.
set -eu
here=$(cd "$(dirname "$0")/../.." && pwd)
cd "$here"
stem="${1:-tests}"
cargo test --workspace --release > "work/w-fltret2/$stem.raw" 2>&1 || true
grep -E "^test result|^error|FAILED" "work/w-fltret2/$stem.raw" > "work/w-fltret2/$stem.txt" || true
python3 - "work/w-fltret2/$stem.txt" <<'PY'
import re, sys
p = f = n = 0
for line in open(sys.argv[1]):
    m = re.match(r"test result: \w+\. (\d+) passed; (\d+) failed", line)
    if m:
        p += int(m.group(1)); f += int(m.group(2)); n += 1
print(f"{p} passed, {f} failed, {n} targets")
PY
