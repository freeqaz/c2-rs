#!/bin/sh
# lane w-fork — spawn-per-obj baseline: today's reference-replay path, one
# `wibo c2host.exe c2.dll c2.dll <argv>` process per compilation, sequential.
#
# usage: baseline.sh <corpus-dir> <out-suffix>
# Writes <case>/<suffix>.obj and prints "produced N of M".
set -e
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
CORPUS=$1
SUF=${2:-base}
WIBO=${C2RS_WIBO:-$ROOT/../wibo/build/release/wibo}
C2DLL=$ROOT/compilers/X360/16.00.11886.00/c2.dll
HOST=$ROOT/target/c2host/c2host.exe
export WIBO_FS_CACHE=1

m=0; n=0
for case in "$CORPUS"/*; do
  [ -f "$case/argv.txt" ] || continue
  m=$((m+1))
  # shellcheck disable=SC2046
  set -- $(cat "$case/argv.txt")
  rm -f "$case/out.obj"
  (cd "$case" && "$WIBO" "$HOST" "$C2DLL" "$C2DLL" "$@") >/dev/null 2>&1 || true
  if [ -s "$case/out.obj" ]; then
    mv "$case/out.obj" "$case/$SUF.obj"
    n=$((n+1))
  fi
done
echo "produced $n of $m"
