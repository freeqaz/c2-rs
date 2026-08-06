#!/bin/sh
# scrub.sh — remove absolute machine paths from anything this lane commits.
#
# `CLAUDE.md`: no `/home/<user>/…` in ANY committed file. Toolchain and tree
# locations are env-driven by design, so a committed extract names the env var
# and never the box.
set -eu
for f in "$@"; do
    # The patterns are USER-AGNOSTIC on purpose: a scrubber that spells one
    # box's home directory is itself the thing it exists to remove.
    sed -i -E \
        -e 's|/home/[a-z][a-z0-9_-]*/[A-Za-z0-9_./-]*/\.claude/worktrees/[A-Za-z0-9-]+|<worktree>|g' \
        -e 's|/home/[a-z][a-z0-9_-]*/code/milohax|<milohax>|g' \
        -e 's|/home/[a-z][a-z0-9_-]*|<home>|g' \
        "$f"
done
