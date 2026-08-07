#!/bin/sh
# scrub.sh — replace this box's absolute paths in a committed log.
#
# Lane w-alloc3. **A SCRUBBER IS A DESTRUCTIVE TRANSFORM.** `w-splice` §12.1
# records what that costs: `sed -i` was run over `gate_tip.txt` while the gate
# was still appending to it, and the rewrite truncated everything after the
# point it had read — a run that exited 0 left a log ending mid-`mode cross`,
# indistinguishable from a hung one.
#
# So this refuses to touch a file any process still holds open, instead of
# trusting the caller to have waited.
#
# Usage:  work/w-alloc3/scrub.sh <file> [<file> …]
set -eu
root="$(cd "$(dirname "$0")/../.." && pwd)"
main="$(cd "$root/../../.." && pwd)"

for f in "$@"; do
    [ -f "$f" ] || { echo "scrub: no such file $f" >&2; exit 1; }
    if command -v fuser >/dev/null 2>&1 && fuser "$f" >/dev/null 2>&1; then
        echo "scrub: REFUSING $f — a process still holds it open" >&2
        exit 1
    fi
    sed -i -e "s|$root|<worktree>|g" -e "s|$main|<milohax>|g" \
           -e "s|/home/[a-z0-9_-]*|<home>|g" "$f"
    echo "scrubbed $f"
done
