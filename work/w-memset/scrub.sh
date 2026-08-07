#!/bin/sh
# scrub.sh — replace this box's absolute paths in the committed lane evidence.
#
# CLAUDE.md forbids committing `/home/<user>/…`. The scripts derive the worktree
# from their own location and carry none; the captured logs are rewritten to the
# placeholders `<worktree>`, `<milohax>` and `<home>`.
set -eu
WT=$(cd "$(dirname "$0")/../.." && pwd)
MX=$(cd "$WT/../../.." && pwd)
HM=$(cd "$HOME" && pwd)
cd "$WT"
for f in work/w-memset/*.txt; do
    [ -f "$f" ] || continue
    sed -i -e "s#$WT#<worktree>#g" -e "s#$MX#<milohax>#g" -e "s#$HM#<home>#g" "$f"
done
if grep -l "$HM" work/w-memset/*.txt 2>/dev/null; then
    echo "STILL ABSOLUTE — above"
    exit 1
fi
echo "no absolute path left in work/w-memset/*.txt"
