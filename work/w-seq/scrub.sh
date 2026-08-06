#!/bin/sh
# scrub.sh — replace this box's absolute paths with placeholders, in place.
#
# Lane w-seq measurement tooling. CLAUDE.md: no `/home/<user>/…` in anything
# committed. The provenance banners and `gt_dump.py`'s obj header both print
# absolute paths, so every captured log has to go through this before `git add`.
#
# Usage:  scrub.sh <file> [<file>...]
set -eu
for f in "$@"; do
    [ -f "$f" ] || continue
    sed -i \
        -e 's#/home/[A-Za-z0-9_.-]*/code/milohax/c2-rs/\.claude/worktrees/[A-Za-z0-9_-]*#<worktree>#g' \
        -e 's#/home/[A-Za-z0-9_.-]*/code/milohax/c2-rs#<repo>#g' \
        -e 's#/home/[A-Za-z0-9_.-]*/code/milohax/dc3-decomp#<dc3>#g' \
        -e 's#/home/[A-Za-z0-9_.-]*/code/milohax/wibo#<wibo>#g' \
        -e 's#/home/[A-Za-z0-9_.-]*#<home>#g' \
        "$f"
done
grep -l '/home/' "$@" 2>/dev/null && { echo "SCRUB INCOMPLETE"; exit 1; }
exit 0
