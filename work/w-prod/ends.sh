#!/bin/sh
# ends.sh — the BOTH-ENDS measurement block for lane w-prod.
#
#   sh work/w-prod/ends.sh <tag>        tag is `base` or `tip`
#
# Runs, in order, on a harness built IN THIS TREE (#1205 — never a binary
# carried from elsewhere, and never `target/release/c2rs`, which another
# process may republish mid-run):
#
#   * sweep fragment `88-store-run-call`      and its per-case Match /
#   * sweep fragment `89-store-run-live-arg`  NotImplemented tally
#   * the 878-TU workload scan
#   * peer keys over the scan's jsonl
#
# Scratch lives under `work/w-prod/`, never `/tmp` (which is periodically
# cleaned out from under a long lane).
set -eu
tag="${1:?usage: ends.sh <base|tip>}"
here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/../.." && pwd)"

sib() {
    d="$root"
    while [ "$d" != "/" ]; do
        [ -d "$d/../$1" ] && { (cd "$d/../$1" && pwd); return 0; }
        d="$(dirname "$d")"
    done
    return 1
}
dc3="${C2RS_DC3:-$(sib dc3-decomp)}"

cargo build --release --manifest-path "$root/Cargo.toml" -p c2-harness >/dev/null

for frag in 88-store-run-call 89-store-run-live-arg; do
    n=$(printf '%s' "$frag" | cut -c1-2)
    out="$here/sweep$n-$tag.d"
    rm -rf "$out"
    mkdir -p "$out"
    C2RS_SWEEP_ONLY="$frag" C2RS_DC3="$dc3" C2RS_SWEEP_JOBS=6 \
        sh "$root/scripts/expr_sweep.sh" "$out" \
        > "$here/sweep$n"_"$tag".txt 2>&1 || true
    tail -1 "$here/sweep$n"_"$tag".txt
    sh "$here/tally.sh" "$out" 6 > "$here/tally$n"_"$tag".out 2>&1 || true
    tail -1 "$here/tally$n"_"$tag".out
done

sh "$here/scan.sh" "$tag" "$root/target/release/c2rs"
echo "ENDS $tag DONE"
