#!/bin/sh
# dump.sh — print the `.text` words of the named GRID K cells, or of all of them.
set -eu
repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
grid="$repo_root/work/w-carrier/grid"
cells="${*:-$(cd "$grid" && ls)}"
for c in $cells; do
    printf '=== %s\n' "$c"
    if [ -f "$grid/$c/dis.txt" ]; then
        grep -E '^   [0-9a-f]{4}  ' "$grid/$c/dis.txt" || echo "   (no text)"
    else
        echo "   NO DISASM"
    fi
done
