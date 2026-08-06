#!/bin/sh
# scrub.sh — replace this box's absolute paths with placeholders before any
# lane evidence is committed. `CLAUDE.md`: never commit absolute machine paths.
#
#   <worktree>  this lane's worktree
#   <milohax>   the parent checkout directory
#   <home>      anything else under the user's home
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
wt="$(cd "$here/../.." && pwd)"
mx="$(cd "$wt/../../.." && pwd)"
for f in "$@"; do
    [ -f "$f" ] || { echo "no such file: $f" >&2; continue; }
    sed -i -e "s#$wt#<worktree>#g" -e "s#$mx#<milohax>#g" -e "s#/home/[a-z0-9_-]*#<home>#g" "$f"
done
