#!/bin/sh
# Run one mode lane under a NAMED binary and keep its per-fixture verdicts.
#
#     work/w-pool2/lane.sh <base|tip> <flags...>
#
# The counterfactual form the brief requires: both binaries grade the SAME
# fixture list, which is regenerated after the last fixture this lane authored
# and `wc -l`-checked. `C2RS_BIN` is `harness_bin.sh`'s documented override, so
# the base binary is the one that was BUILT at the base and kept (#2409) — never
# a `git checkout master -- crates/` round trip.
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
which="$1"
shift
case "$which" in
    base) bin="$here/c2rs-base" ;;
    tip)  bin="$here/c2rs-tip" ;;
    *) echo "usage: lane.sh <base|tip> <flags...>" >&2; exit 2 ;;
esac
tag="$(echo "$which $*" | tr -c 'A-Za-z0-9' '_')"
C2RS_BIN="$bin" sh "$repo/scripts/mode_lane.sh" "$@" > "$here/lane_$tag.log" 2>&1 || true
tail -1 "$here/lane_$tag.log"
