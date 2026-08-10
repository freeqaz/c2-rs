#!/bin/sh
# Reference obj + symbol dump for a LOCAL cpp at `/Ox /GS- /c` (the `c2rs diff`
# default), so a sweep case can be read the way `CEILING.md` §11.4 item 3 says.
#
#     sh work/w-selbind/reflocal.sh <tag> <path-relative-to-repo>
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
. "$here/env.sh"
tag="$1"
src="$2"
mkdir -p "$here/ref"
"$repo/target/release/c2rs" compile "$src" --keep-obj "$here/ref/$tag.obj" \
    > "$here/ref/$tag.log" 2>&1
python3 "$repo/scripts/gt_dump.py" "$here/ref/$tag.obj"
