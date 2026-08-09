#!/bin/sh
# `hatch-red` at the EXACT BASE TREE.
#
# Board #1406 (HATCH-DRIFT) and #1322 (HATCH-STALE) both say the same thing:
# reproduce the refusal at your own base before attributing it to yourself. The
# gate reports HATCH-STALE at the tip; this runs the same instrument over a
# temporary commit of `crates/` at the merge-base, with every file this lane
# adds removed, then resets back.
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
cd "$repo"
BASE=5831a092
TIP="$(git rev-parse HEAD)"
echo "tip $TIP  base $BASE"
trap 'git reset --hard "$TIP" >/dev/null 2>&1; echo "restored to $TIP"' EXIT INT TERM

git checkout "$BASE" -- crates
rm -f crates/c2-harness/tests/pool2_cells.rs \
      crates/c2-il/src/func/body/shapes/pool_free_list.rs \
      crates/c2-il/src/func/body/shapes/pool_ctor_chain.rs \
      crates/c2-core/src/codegen/pool_free_list.rs \
      crates/c2-core/src/codegen/pool_ctor_chain.rs
git add -A crates
git -c user.name=tmp -c user.email=tmp@local commit -q -m "TEMPORARY base tree" || true
echo "diff vs base over crates (must be 0 lines):"
git diff "$BASE" --stat -- crates | tail -2
python3 work/w-hatch/hatch_red.py 2>&1 | tail -25
echo "final crates/ diff: $(git status --porcelain -- crates | wc -l) lines"
