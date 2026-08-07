#!/bin/sh
# scrub.sh — replace this box's absolute paths in this lane's committed evidence.
#
# The patterns are DERIVED, never listed: a previous lane's scrub script hard-coded
# five of this box's own paths as its search patterns, which is the same defect one
# level out. Two rules, in this order:
#
#   1. the repo/worktree root, whatever it is, becomes `<repo>`;
#   2. any remaining `/home/<user>` becomes `/home/<user>`, matched as
#      `/home/[a-z][a-z0-9_-]*` and not as a literal.
#
# BOARD #1135: never rewrite a file another process still holds open. A scrub once
# raced a backgrounded `gate.sh` that still held its `>` descriptor and punched a
# NUL hole into a PASSING gate's log — `grep` returned nothing and a waiter
# reported TIMEOUT; the mirror case, on a FAILING gate, makes `grep -q FAIL` read
# clean.
#
# The guard is **per file and by open descriptor**, not "is any gate alive". The
# coarse form is both too weak (a writer that is not a gate slips through) and too
# strong (a PEER LANE's gate, in another worktree, writing a file this script will
# never open, blocks it). Peer lanes run concurrently here by design and their
# logs must be left alone — which is the other half of #1135.
set -eu
root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"

holders() {
    # PIDs with the given absolute path open, via /proc's fd symlinks. Bounded
    # depth; /proc only, never the repo tree.
    find /proc -mindepth 3 -maxdepth 3 -path '/proc/[0-9]*/fd/*' -lname "$1" \
        2>/dev/null | cut -d/ -f3 | sort -u
}

for f in "$@"; do
    [ -f "$f" ] || { echo "no such file: $f" >&2; exit 1; }
    abs="$(cd "$(dirname "$f")" && pwd)/$(basename "$f")"
    h="$(holders "$abs")"
    if [ -n "$h" ]; then
        echo "REFUSING: $f is held open by PID(s): $h — scrub after they exit" >&2
        echo "  (board #1135: rewriting a log a writer still holds punches a NUL" >&2
        echo "   hole into it and makes every later grep of it meaningless)" >&2
        exit 2
    fi
    tmp="$f.scrub.$$"
    sed -e "s|$root|<repo>|g" \
        -e 's|/home/[a-z][a-z0-9_-]*|/home/<user>|g' "$f" > "$tmp"
    mv "$tmp" "$f"
    if LC_ALL=C grep -q -a -P '\x00' "$f"; then
        echo "FATAL: $f is not NUL-free after the rewrite (board #1135)." >&2
        exit 3
    fi
done
echo "scrubbed $# file(s); all NUL-free, none held open"
