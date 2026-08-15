#!/bin/sh
# w-fencea — one mutant at a time: apply, rebuild, grade against REAL c2.dll at
# /O1, print the verdicts, revert. A guard nobody has seen fire is a guard
# nobody has tested.
#
#   work/w-fencea/mutate.sh <name> <file> <sed-expr>
set -eu
R="$(cd "$(dirname "$0")/../.." && pwd)"
name="$1"; file="$2"; expr="$3"
cp "$R/$file" "$R/$file.orig"
sed -i "$expr" "$R/$file"
if cmp -s "$R/$file" "$R/$file.orig"; then
    echo "MUTANT $name: NOT APPLIED — the sed matched nothing"
    rm -f "$R/$file.orig"; exit 2
fi
echo "=== MUTANT $name  ($file) ==="
if ! (cd "$R" && cargo build --release -p c2-harness >/dev/null 2>&1); then
    echo "  BUILD-RED (the mutant does not compile — a red, and the cheapest kind)"
    mv "$R/$file.orig" "$R/$file"; exit 0
fi
# The two `cl.exe` lists are REGENERATED here rather than committed: they hold
# `z:\\home\\…` machine paths, and this repo does not track those.
for f in "$R"/work/w-fencea/cells/*.cpp; do
    printf 'z:%s\n' "$(printf '%s' "$f" | tr '/' '\\')"
done > "$R/work/w-fencea/cells_list.txt"
# the cells (real-obj oracles, n = 0..3 plus four plain-leaf controls) …
(cd "$R" && ./target/release/c2rs gap --list work/w-fencea/cells_list.txt \
    --flags-file work/w-fencea/cells_flags.txt --jobs 4 2>&1 \
    | sed -n 's|^ *\[[0-9]*/8\] *\([a-z-]*\) .*\\\([a-z_0-9]*\)\.cpp|  cell \2 = \1|p')
# … and the four tracked fixtures this lane's classes own.
for f in whash_ptr_walk_loop whash_loop_then_framed wvl_chain3 wvl_chain6_same wxtea3_encrypt_loop; do
    printf 'z:%s\n' "$(printf '%s' "$R/fixtures/cpp/$f.cpp" | tr '/' '\\')"
done > "$R/work/w-fencea/fx_list.txt"
(cd "$R" && ./target/release/c2rs gap --list work/w-fencea/fx_list.txt \
    --flags-file work/w-fencea/cells_flags.txt --jobs 4 2>&1 \
    | sed -n 's|^ *\[[0-9]*/5] *\([a-z-]*\) .*\\\([a-z_0-9]*\)\.cpp|  fixture \2 = \1|p')
mv "$R/$file.orig" "$R/$file"
(cd "$R" && cargo build --release -p c2-harness >/dev/null 2>&1)
