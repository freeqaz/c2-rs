#!/bin/sh
# sched.sh — the ORDERING grid. Reads REAL c2's emission only; the port is not
# consulted and does not need to be, because the question is what c2 schedules,
# not what we can already parse.
set -eu
root=<repo>/.claude/worktrees/agent-a90821e906953b0fd
for f in "$@"; do
    printf '########## %s\n' "$f"
    sh "$root/work/w-dclass/b-0x27/refobj_probe.sh" "$f" >/dev/null
    b="$(basename "$f" .cpp)"
    python3 "$root/scripts/gt_dump.py" \
        "$root/work/w-dclass/b-0x27/p/$b.obj" 2>&1 \
        | sed -n '/-- .text/,/-- symbols/p' | grep -vE '^-- symbols'
done
