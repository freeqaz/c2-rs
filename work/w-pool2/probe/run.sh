#!/bin/sh
# Capture every probe cell at the workload's own optimization level and print
# its `.text` beside its neighbour's. Measurement only — nothing here is a
# fixture and nothing here is graded by the port.
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
for f in "$@"; do
    sh "$repo/scripts/gt_capture.sh" "$here/$f.cpp" /nologo /c /GR /O1 /Oi /EHsc 2>/dev/null
    echo "=== $f"
    python3 "$repo/scripts/gt_dump.py" "$here/$f.obj" | sed -n '/-- .text/,/-- symbols/p' | head -40
done
