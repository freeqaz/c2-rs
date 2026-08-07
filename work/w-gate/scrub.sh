#!/bin/sh
# Scrub absolute machine paths out of this lane's committed logs.
#
# CLAUDE.md forbids committing `/home/<user>/…`; the gate prints its repo root and
# its run tree in every header, and `toolchain_hint` prints four more. Rewritten
# to `<worktree>` / `<milohax>` / `<home>`, longest prefix first so a shorter one
# cannot eat a longer one's replacement.
set -eu
repo="$(cd "$(dirname "$0")/../.." && pwd)"
parent="$(cd "$repo/../../.." && pwd)"   # .../c2-rs  (worktree is <repo>/.claude/worktrees/<n>)
milohax="$(cd "$parent/.." && pwd)"

for f in "$@"; do
    [ -f "$f" ] || { echo "scrub: no such file: $f" >&2; exit 1; }
    sed -i \
        -e "s|$repo|<worktree>|g" \
        -e "s|$parent|<c2-rs>|g" \
        -e "s|$milohax|<milohax>|g" \
        -e "s|$HOME|<home>|g" \
        "$f"
done

for f in "$@"; do
    if grep -q '/home/' "$f"; then
        echo "scrub: FAILED — $f still carries a /home/ path:" >&2
        grep -m3 -o '/home/[^ ")]*' "$f" >&2
        exit 1
    fi
done
echo "scrub: $# file(s) clean of absolute machine paths"
