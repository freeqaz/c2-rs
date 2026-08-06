#!/bin/sh
# scrub.sh — strip this machine's absolute paths out of a captured run log.
#
# Lane w-fix measurement tooling. Read-only with respect to `crates/`.
#
# CLAUDE.md forbids committing absolute machine paths, and every long-running
# harness here prints them: `gate.sh` names its worktree and its run dir, cargo
# names `/home/<user>/…` on every compile line. The logs are worth committing —
# they are the evidence a rung's §9 quotes — so they are rewritten rather than
# left out.
#
# Usage:  scrub.sh <in> <out>
set -eu
in="$1"; out="$2"
root="$(cd "$(dirname "$0")/../.." && pwd)"
home="${HOME:-/home/$(id -un)}"
sed -e "s#$root#<worktree>#g" \
    -e "s#$home#<home>#g" \
    -e "s#/home/[A-Za-z0-9_.-]*#<home>#g" \
    -e "s#/tmp/c2rs-gate-[0-9]*#<gate-run>#g" \
    "$in" > "$out"
echo "scrubbed $(grep -c '' "$out") lines -> $out"
grep -n "/home/" "$out" && { echo "STILL CONTAINS AN ABSOLUTE PATH"; exit 1; } || true
