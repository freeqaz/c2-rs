#!/bin/sh
# capture_cells.sh — capture IL + reference obj for every frozen w-align cell.
#
# ONE DIRECTORY PER CELL (board #1045: raced captures into a shared directory
# fabricated a finding that would have reversed a decline). Cells run one at a
# time into `caps/<cell>/`, and the manifest is re-verified first so a cell that
# changed after the freeze cannot be measured by accident.
#
# Profile: the WORKLOAD's own flags (board #1112), from `work/w-align/flags.txt`.
set -eu
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"
lane=work/w-align
flags="$lane/flags.txt"

( cd "$lane/cells" && sha256sum -c SHA256SUMS ) > "$lane/manifest_check.txt" 2>&1 || {
    echo "MANIFEST FAILED — a frozen cell changed; refusing to capture" >&2
    exit 1
}
echo "manifest: $(grep -c ': OK$' "$lane/manifest_check.txt") of 23 cells verified"

rm -rf "$lane/caps"
mkdir -p "$lane/caps"
for f in "$lane"/cells/*.cpp; do
    n="$(basename "$f" .cpp)"
    d="$lane/caps/$n"
    mkdir -p "$d"
    ./target/release/c2rs capture "$f" --keep-il "$d" --flags-file "$flags" \
        > "$d/capture.txt" 2>&1 || echo "CAPTURE-FAIL $n"
    ./target/release/c2rs compile "$f" --keep-obj "$d/ref.obj" --flags-file "$flags" \
        > "$d/compile.txt" 2>&1 || echo "COMPILE-FAIL $n"
    printf '%s ' "$n"
done
echo
echo "done"
