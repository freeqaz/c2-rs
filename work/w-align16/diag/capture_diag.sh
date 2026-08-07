#!/bin/sh
# capture_diag.sh — reference obj + IL for each POST-HOC diagnostic cell.
#
# These cells are NOT part of the frozen prereg grid. They exist to localize the
# live mismatch the frozen grid found on `A11`, and they were written AFTER that
# mismatch was seen. They carry their own `SHA256SUMS` so the localization is
# reproducible, and the rung says plainly that they are post-hoc.
#
# One directory per cell (board #1045). Profile: the workload's own (#1112).
set -eu
root="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$root"
lane=work/w-align16/diag
( cd "$lane/cells" && sha256sum -c SHA256SUMS ) > "$lane/manifest_check.txt" 2>&1 || {
    echo "MANIFEST FAILED — a diagnostic cell changed" >&2
    exit 1
}
rm -rf "$lane/caps"
mkdir -p "$lane/caps"
for f in "$lane"/cells/*.cpp; do
    n="$(basename "$f" .cpp)"
    d="$lane/caps/$n"
    mkdir -p "$d"
    ./target/release/c2rs capture "$f" --keep-il "$d" --flags-file work/w-align16/flags.txt \
        > "$d/capture.txt" 2>&1 || echo "CAPTURE-FAIL $n"
    ./target/release/c2rs compile "$f" --keep-obj "$d/ref.obj" --flags-file work/w-align16/flags.txt \
        > "$d/compile.txt" 2>&1 || echo "COMPILE-FAIL $n"
    printf '%s ' "$n"
done
echo
