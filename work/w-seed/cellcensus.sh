#!/bin/sh
# cellcensus.sh — `c2rs census` on ONE GRID-N cell, assembled exactly as
# `tests/cellgrade` assembles it (ANCHOR prepended, TAIL PAD appended) and at the
# same flags.
#
# The cell as it sits in `work/w-seed/cells/` is not what the grid compiles, and a
# census run on the bare file would be reading a different TU from the one the
# verdicts came from.
#
# Usage: work/w-seed/cellcensus.sh <cell> [extra c2rs census args...]
set -eu
WT=$(cd "$(dirname "$0")/../.." && pwd)
. "$WT/work/w-seed/env.sh"
cell="$1"
shift
out="$WT/work/w-seed/scratch-$cell"
mkdir -p "$out"
{
    printf '\nvoid ext_anchor();\nvoid anchor() { ext_anchor(); }\n'
    cat "$WT/work/w-seed/cells/$cell.cpp"
    cat <<'EOF'

template <class T> inline T pad5(T v) { return v; }
template <class T> inline T pad4(T v) { return pad5(v); }
template <class T> inline T pad3(T v) { return pad4(v); }
template <class T> inline T pad2(T v) { return pad3(v); }
template <class T> inline T pad1(T v) { return pad2(v); }
int pad_use(int v) { return pad1(v); }
EOF
} > "$out/$cell.cpp"
printf '%s\n' '/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc' > "$out/flags.txt"
"$WT/target/release/c2rs" census "$cell.cpp" \
    --flags-file "$out/flags.txt" --cwd "$out" "$@"
