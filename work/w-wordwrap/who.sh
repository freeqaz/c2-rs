#!/bin/sh
# Which functions of which workload TUs the new class took, by NAME — the census
# row and the emitted symbol, from the port's own accounting rather than from a
# grep over source.
#
#     work/w-wordwrap/who.sh src/a.cpp src/b.cpp …
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
for f in "$@"; do
    printf '== %s\n' "$f"
    "$here/cenw.sh" "$f" 2>/dev/null | grep -F "global-store-leaf" || echo "   (none)"
done
