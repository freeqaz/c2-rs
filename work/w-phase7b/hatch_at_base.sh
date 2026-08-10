#!/bin/sh
# Reproduce `hatch-red`'s verdict at THIS LANE'S EXACT MERGE-BASE.
#
# The gate reports `HATCH-RED REFUSED / HATCH-STALE`. That is a property of the
# TREE, not of this lane, but "not this lane's" has to be MEASURED (board #1406,
# and `hatch-red` refuses on pre-existing failures rather than attributing them).
#
# Everything this lane authored is committed before this runs, so the temporary
# commit + `git reset --hard` below cannot lose work — which is the only reason
# it is safe. #2668: `hatch_red.py` discards uncommitted `crates/` edits while
# printing "final crates/ diff: EMPTY".
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
base="$1"
cd "$repo"
if [ -n "$(git status --porcelain -- crates fixtures)" ]; then
    echo "REFUSING: crates/ or fixtures/ is dirty. Commit first." >&2
    exit 2
fi
tip="$(git rev-parse HEAD)"
git checkout "$base" -- crates fixtures
git commit -q -m "TEMPORARY: crates+fixtures at the merge-base for the hatch-red counterfactual" \
    -- crates fixtures || true
echo "-- crates/fixtures diff against $base (must be 0 lines) --"
git diff "$base" --stat -- crates fixtures
python3 work/w-hatch/hatch_red.py > "$here/HATCH_RED_AT_BASE.txt" 2>&1 || true
git reset -q --hard "$tip"
echo "-- restored to $tip; diff against it (must be empty) --"
git diff "$tip" --stat
tail -20 "$here/HATCH_RED_AT_BASE.txt"
