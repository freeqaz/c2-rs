#!/bin/sh
# sweeps.sh -- sweep fragments 88 and 89 with their per-case port split, at ONE
# end.  Adapted from work/w-mixkind/ends.sh; the binary is the one built in this
# tree at this end, never carried (#1205).
set -eu
tag="${1:?usage: sweeps.sh <base|tip>}"
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"
sib() { d="$root"; while [ "$d" != "/" ]; do [ -d "$d/../$1" ] && { (cd "$d/../$1" && pwd); return 0; }; d="$(dirname "$d")"; done; return 1; }
dc3="${C2RS_DC3:-$(sib dc3-decomp)}"
for frag in 88-store-run-call 89-store-run-live-arg; do
    n=$(printf '%s' "$frag" | cut -c1-2)
    out="$here/sweep$n-$tag.d"
    rm -rf "$out"; mkdir -p "$out"
    C2RS_SWEEP_ONLY="$frag" C2RS_DC3="$dc3" C2RS_SWEEP_JOBS=6 \
        sh "$root/scripts/expr_sweep.sh" "$out" > "$here/sweep${n}_$tag.txt" 2>&1 || true
    tail -1 "$here/sweep${n}_$tag.txt"
    sh "$here/tally.sh" "$out" 6 > "$here/tally${n}_$tag.out" 2>&1 || true
    tail -1 "$here/tally${n}_$tag.out"
done
