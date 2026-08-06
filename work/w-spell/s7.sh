#!/bin/sh
# s7.sh — PREREG §4 (S7): re-run GRID S at the brief's `/O1 /GS- /c` instead of
# the workload's own `/O1 /Oi /EHsc /GR`, and compare the WINNER cell for cell.
#
# If the two disagree anywhere, every allocation figure on this project's record
# is flag-conditional and that is the finding.
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
python3 "$here/spellgrid.py" --jobs 8 --flags "/O1 /GS- /c" \
    > "$here/spellgrid_alt.out" 2>&1
python3 "$here/s7cmp.py" > "$here/s7.out" 2>&1
cat "$here/s7.out"
