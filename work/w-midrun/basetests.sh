#!/bin/sh
# basetests.sh — `cargo test --workspace --release` at the BASE tree, in place.
#
# The lane's own tip is committed, so the base is restored by `git checkout HEAD`
# whatever happens — the `trap` is the point, not the convenience. `fixtures/` is
# reverted TOO and the lane's new fixture is removed: a first attempt left it in
# place and the base tree's `census_gate` test failed on it, which made the base
# total unusable and is exactly the kind of "measured at the wrong tree" number
# this file exists to avoid.  `docs/` is reverted TOO for the same reason: a
# second attempt left the lane's rung doc in place and `rung_registry` failed on
# a fixture the base tree does not have, aborting the run at 21 of 36 targets.
set -u
cd "$(dirname "$0")/../.."
restore() { git checkout HEAD -- crates/ fixtures/ docs/; }
trap restore EXIT INT TERM
git checkout 503f8937 -- crates/ fixtures/ docs/
rm -f fixtures/cpp/w1274_addr_producer.cpp docs/rungs/2026-08-09-w-midrun.md
cargo test --workspace --release
