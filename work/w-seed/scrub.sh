#!/bin/sh
# scrub.sh — replace this machine's absolute paths in a captured log with
# placeholders, so evidence can be committed.
#
# CLAUDE.md: absolute machine paths are never committed. `cargo` prints the
# worktree root on every `Compiling` line and the toolchain root turns up in
# capture diagnostics, so a raw log is uncommittable however useful it is.
#
# Usage: work/w-seed/scrub.sh <file>...
set -eu
WT=$(cd "$(dirname "$0")/../.." && pwd)
case "$WT" in
    */.claude/worktrees/*) MAIN=$(cd "$WT/../../.." && pwd) ;;
    *)                     MAIN="$WT" ;;
esac
PARENT=$(cd "$MAIN/.." && pwd)
for f in "$@"; do
    sed -i -e "s#$WT#<worktree>#g" -e "s#$MAIN#<repo>#g" -e "s#$PARENT#<milohax>#g" \
        -e "s#$HOME#<home>#g" "$f"
    echo "scrubbed $f"
done
