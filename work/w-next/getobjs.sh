#!/bin/sh
# getobjs.sh — capture reference objs for the candidate TUs at the workload flags.
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
root="$here/../.."
mkdir -p "$here/objs"
for f in "$@"; do
    n=$(basename "$f" .cpp)
    if "$root/work/w-frame/refobj.sh" "$f" "$here/objs/$n.obj" >/dev/null 2>&1; then
        echo "OK   $n  $(stat -c%s "$here/objs/$n.obj") bytes"
    else
        echo "FAIL $n"
    fi
done
