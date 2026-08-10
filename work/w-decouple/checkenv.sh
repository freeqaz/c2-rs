#!/bin/sh
# Print the resolved lane environment, so the stamp in the rung is a reading
# rather than a claim.
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
. "$here/env.sh"
echo "C2RS_COMPILERS=$C2RS_COMPILERS"
ls "$C2RS_COMPILERS/X360"
echo "C2RS_WIBO=$C2RS_WIBO"
ls -l "$C2RS_WIBO" | sed 's/ [^ ]*$//'
echo "C2RS_DC3=$C2RS_DC3"
git -C "$C2RS_DC3" rev-parse HEAD
echo "dc3 tracked-tree dirty lines: $(git -C "$C2RS_DC3" status --porcelain --untracked-files=no | wc -l)"
echo "WD_FILES=$WD_FILES  lines: $(wc -l < "$WD_FILES")"
echo "WD_FLAGS=$WD_FLAGS"
cat "$WD_FLAGS"
