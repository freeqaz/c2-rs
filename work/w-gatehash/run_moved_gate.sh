#!/bin/sh
# w-gatehash — reproduce #3835's incident deterministically.
#
# Launches a full gate and, 45 s in (well after the FIRST identity is taken and
# well before the last row finishes), creates ONE untracked, non-gitignored file
# under `crates/`. That is exactly what lane `w-globset` did by accident when it
# authored two modules in the worktree its base gate was running in: 808 files at
# the start, 810 at the end.
#
# Usage: run_moved_gate.sh <outfile> [extra gate args...]
# The probe file is removed on the way out, on every path.
set -u
out="$1"; shift
root=$(cd "$(dirname "$0")/../.." && pwd)
probe="$root/crates/w_gatehash_probe.txt"

rm -f "$probe"
( sleep 45; printf 'w-gatehash mid-run tree mutation (board #3835 reproduction)\n' > "$probe" ) &
mutator=$!

sh "$root/scripts/gate.sh" "$@" > "$out" 2>&1
echo "GATE_EXIT=$?" >> "$out"

# Never `pkill`: peers run gates from their own worktrees on this box.
wait "$mutator" 2>/dev/null
rm -f "$probe"
echo "probe removed; tree clean: $(cd "$root" && git status --porcelain crates/ | wc -l) crates/ entries" >> "$out"
