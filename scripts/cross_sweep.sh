#!/bin/sh
# The CROSS-PRODUCT LANE — grade every accepted shape family beside every other.
#
# `scripts/expr_sweep.sh` sweeps one axis at a time: each fragment varies its own
# parameters inside one shape family. That corpus grows ADDITIVELY, and there is a
# whole class of defect it structurally cannot contain — the one where two facts
# share a field until some *other* function in the same translation unit pulls them
# apart. `docs/GAPS.md` §6 #12 is the instance: an FP-store rung and a many-call
# framed rung were each fully green, and the MERGE mis-emitted, because the
# compiler-label counter is a per-TU quantity that was being read from a
# per-function method. Neither branch's corpus could hold the case. #13 then showed
# the repair was also wrong one row further out, for the same reason at n = 1.
#
# The rule those two wrote down — "a merge of two independently-green branches is a
# new corpus, and the shapes only it contains have never been graded by anyone" —
# was being applied by hand. This lane applies it mechanically: it asks the PORT
# which shape families it accepts, discovers a representative TU of each by
# grading the whole sweep corpus, and then compiles every ordered pair of them,
# both orders, in packed and /Gy and /O1 and /O2.
#
# Usage:  scripts/cross_sweep.sh [workdir]
#         C2RS_CROSS_REPS=1 scripts/cross_sweep.sh    # quicker: 1 rep per family
#         C2RS_JOBS=32 scripts/cross_sweep.sh
#
# A MISMATCH is an ALARM: the port emitted bytes for a COMBINATION and they were
# wrong. Exits non-zero on a mismatch, and on a declared family that no fragment
# can supply a representative for (that is a hole in the sweep corpus, and a lane
# that quietly skipped it would be claiming coverage it does not have).
#
# Needs the toolchain (see CLAUDE.md); without it the lane says SKIP and exits 0.
set -eu

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
# ABSOLUTE. A relative workdir used to reach the driver as-is and die with a bare
# `KeyError`; it is also the shape that yields `z:work\…` paths `cl.exe` cannot
# open, so every case capture-fails and every count parsed out of the report reads
# zero and passes. Resolved here as well as in the driver — both entry points.
work="${1:-/tmp/c2rs-cross-sweep}"
mkdir -p "$work"
work="$(cd "$work" && pwd)"

# Build unconditionally and hand the driver a RUN-PRIVATE COPY of the binary.
# `scripts/harness_bin.sh` has the reasoning; the short version is that this lane
# exists to make merge grading trustworthy, so it is the last place that should be
# able to grade today's cases with yesterday's code — and the last place that
# should die because someone ran `cargo build` in another window.
. "$repo_root/scripts/harness_bin.sh"
pin_harness "$repo_root" "$work"
C2RS_BIN="$C2RS_PINNED"
export C2RS_BIN

exec python3 "$repo_root/scripts/cross_sweep.py" "$work"
