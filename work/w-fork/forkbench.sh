#!/bin/sh
# lane w-fork — start the fork server, wait for its readiness sentinel with a
# deadline, run the driver against it, then shut it down and reap.
#
# usage: forkbench.sh <corpus-dir> [out-suffix]
set -e
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
LANE=$ROOT/work/w-fork
CORPUS=$1
SUF=${2:-fork}
WIBO_FS=${W_FORK_WIBO:-/home/free/code/milohax/wibo-forkserver/build/release/wibo}
C2DLL=$ROOT/compilers/X360/16.00.11886.00/c2.dll
SOCK=$LANE/forkd.sock
LOG=$LANE/forkd.log

rm -f "$SOCK" "$SOCK.ready" "$LOG"

WIBO_FORK_SOCKET=$SOCK \
WIBO_FORK_MAX_REQUESTS=${W_FORK_MAX:-200000} \
WIBO_FORK_IDLE_SECS=${W_FORK_IDLE:-60} \
WIBO_FS_CACHE=1 \
  "$WIBO_FS" "$LANE/c2forkd.exe" "$C2DLL" > "$LOG" 2>&1 &
SERVER=$!

# Bounded wait on the readiness sentinel; TIMEOUT is a distinct outcome.
ready=0
i=0
while [ "$i" -lt 300 ]; do
  if [ -f "$SOCK.ready" ]; then ready=1; break; fi
  if ! kill -0 "$SERVER" 2>/dev/null; then
    echo "fork server died before becoming ready; log:"; cat "$LOG"; exit 1
  fi
  sleep 0.1
  i=$((i+1))
done
if [ "$ready" = 0 ]; then
  echo "TIMEOUT after 30 s waiting for $SOCK.ready — server never became ready"
  kill "$SERVER" 2>/dev/null || true
  wait "$SERVER" 2>/dev/null || true
  exit 1
fi

rc=0
"$LANE/driver" fork "$CORPUS" "$SOCK" "$C2DLL" "$SUF" || rc=$?

# The driver sends an explicit shutdown request; give the server a bounded
# moment to print its rusage line, then make sure it is gone either way.
i=0
while [ "$i" -lt 100 ]; do
  kill -0 "$SERVER" 2>/dev/null || break
  sleep 0.1
  i=$((i+1))
done
if kill -0 "$SERVER" 2>/dev/null; then
  echo "server did not exit on the shutdown request after 10 s — killing"
  kill "$SERVER" 2>/dev/null || true
fi
wait "$SERVER" 2>/dev/null || true
grep -h "forkserver: served" "$LOG" || echo "NOTE: no server CPU line in $LOG"
rm -f "$SOCK" "$SOCK.ready"
exit $rc
