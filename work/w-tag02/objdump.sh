#!/bin/bash
# objdump.sh — the `.data`/`.bss` face of every captured w-tag02 obj.
#   work/w-tag02/objdump.sh [cell...]
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
cells=("$@")
if [ ${#cells[@]} -eq 0 ]; then
    cells=($(cat work/w-tag02/grid_list.txt | sed 's/\.cpp$//'))
fi
for c in "${cells[@]}"; do
    echo "############ $c"
    python3 scripts/gt_dump.py "work/w-tag02/obj/$c.obj" --no-disasm --raw 2>&1 \
        | grep -v '^  \[' | grep -v '^-- symbols'
    python3 scripts/gt_dump.py "work/w-tag02/obj/$c.obj" --no-disasm 2>&1 \
        | grep -E '^  \[' | grep -vE '@comp\.id|\.drectve|\.debug\$S|\.XBLD\$W|__C[12]_11886'
done
