#!/bin/sh
# Build a gate.sh MUTANT that has the #3835 defect back, and prove the selftest
# reddens on it. "Before you trust your own check, watch it fail" — #3787 is the
# case where a checker printed the defect, printed CLEAN, and exited 0.
#
# The mutation deletes the tree-moved block from `decide` and NOTHING else, so
# the mutant is byte-for-byte the pre-fix behaviour: the epilogue comparison is
# still there, it still exits 1, and the headline still says PASS.
#
# It is written to `target/`, which is gitignored AND outside GRADED_DIRS
# (crates fixtures scripts) — so building it does not move the tree of any gate
# running in this worktree, and `repo_root` still resolves to the real repo
# because gate.sh derives it as `dirname $0/..`.
set -eu
root=$(cd "$(dirname "$0")/../.." && pwd)
mut="$root/target/wg_mutant_gate.sh"
mkdir -p "$root/target"

awk '
    /^    if \[ "\$\{gate_tree_moved:-0\}" -eq 1 \]; then$/ { skip = 1 }
    skip == 0 { print }
    skip == 1 && /^    fi$/ { skip = 0 }
' "$root/scripts/gate.sh" > "$mut"
chmod +x "$mut"

cut=$(( $(grep -c . "$root/scripts/gate.sh") - $(grep -c . "$mut") ))
echo "mutant: $cut non-blank lines removed from decide()'s tree-moved block"
# Match the EMITTING line, not the string: the selftest cases quote the same
# headline as a `saw` pattern and must survive the mutation — they are what has
# to go red.
if grep -q 'echo "GATE: FAIL (TREE MOVED UNDER THIS RUN)' "$mut"; then
    echo "MUTATION DID NOT APPLY — decide() still prints the headline." >&2
    exit 2
fi
if ! grep -q "saw    'GATE: FAIL (TREE MOVED UNDER THIS RUN)'" "$mut"; then
    echo "MUTATION ATE THE SELFTEST CASES — nothing would be left to go red." >&2
    exit 2
fi
if ! grep -q 'THE TREE MOVED UNDER THIS RUN' "$mut"; then
    echo "MUTATION WENT TOO FAR — the epilogue block went with it, so the" >&2
    echo "  mutant is not the pre-fix gate and the demonstration is void." >&2
    exit 2
fi
echo "mutant retains the epilogue comparison (pre-fix behaviour, exit 1 + PASS headline)"
echo "$mut"
