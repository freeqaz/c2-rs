#!/bin/sh
# Compile ONE probe source with real cl.exe/c2.dll under wibo at the workload's
# own optimization profile and dump the obj.
#
#     work/w-xtea3/probe.sh probe/mcpytail.cpp [extra flags...]
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
. "$here/env.sh"
src="$1"
shift
[ $# -gt 0 ] || set -- /O1 /Oi /GS- /c
obj="$("$repo/scripts/gt_capture.sh" "$here/$src" "$@")"
python3 "$repo/scripts/gt_dump.py" "$obj"
