#!/bin/sh
# lane w-fork — the A/B benchmark, interleaved.
#
# The box runs other lanes concurrently, so a single spawn-then-fork ordering
# would attribute drift in contention to the fork server. Rounds alternate
# spawn / fork / spawn / fork / ... and the load average is printed with each,
# so drift is visible rather than absorbed.
#
# usage: ab.sh <corpus-dir> <rounds>
set -e
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
LANE=$ROOT/work/w-fork
CORPUS=$1
ROUNDS=${2:-3}
WIBO_FS=<home>/code/milohax/wibo-forkserver/build/release/wibo
C2DLL=$ROOT/compilers/X360/16.00.11886.00/c2.dll
export WIBO_FS_CACHE=1

echo "corpus   : $CORPUS  ($(find "$CORPUS" -maxdepth 1 -mindepth 1 -type d -printf . | wc -c) cases)"
echo "wibo     : $WIBO_FS  ($("$WIBO_FS" --version))"
echo "both arms use the SAME wibo binary — the fork hook is inert without \$WIBO_FORK_SOCKET"
echo

r=1
while [ "$r" -le "$ROUNDS" ]; do
  echo "--- round $r  (load: $(cut -d' ' -f1-3 /proc/loadavg)) ---"
  "$LANE/driver" spawn "$CORPUS" "$WIBO_FS" "$LANE/c2forkd.exe" "$C2DLL" "spawn$r"
  sh "$LANE/forkbench.sh" "$CORPUS" "fork$r"
  r=$((r+1))
done
echo
echo "--- byte identity, every round against its own spawn round ---"
r=1
while [ "$r" -le "$ROUNDS" ]; do
  echo "round $r:"
  python3 "$LANE/compare.py" "$CORPUS" "spawn$r" "fork$r" | sed 's/^/  /'
  r=$((r+1))
done
