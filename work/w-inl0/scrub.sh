#!/bin/sh
# scrub.sh — remove absolute machine paths from anything this lane commits.
#
# `CLAUDE.md`: no `/home/<user>/…` in ANY committed file. Toolchain and tree
# locations are env-driven by design, so a committed extract names the env var
# and never the box.
set -eu
for f in "$@"; do
    sed -i \
        -e 's|/home/free/code/milohax/c2-rs/\.claude/worktrees/[A-Za-z0-9-]*|<worktree>|g' \
        -e 's|/home/free/code/milohax|<milohax>|g' \
        -e 's|/home/free|<home>|g' \
        "$f"
done
