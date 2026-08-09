#!/bin/sh
# Wait for this lane's gate run to finish, with a DEADLINE, and report the
# timeout as a distinct outcome from success.
#
# The predicate is a PID handed in by the caller, never a `pgrep -f` pattern:
# every string in a Bash tool call is present in that shell's own argv, so a
# pattern watcher matches itself and spins forever.
set -eu
pid="$1"
for i in $(seq 1 180); do          # 180 * 20s = 1h ceiling
    kill -0 "$pid" 2>/dev/null || { echo "GATE PROCESS EXITED"; exit 0; }
    sleep 20
done
echo "TIMEOUT after 1h — the gate process is still alive"
exit 1
