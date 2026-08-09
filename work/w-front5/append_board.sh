#!/bin/sh
# Append this lane's board rows and its unminted-remainder note to BOARD.md.
# BOARD.md is hand-maintained (no header block generates it), so the rows live
# in `work/w-front5/board_rows.md` and this script is the one writer, which
# keeps the committed rows and the source file from drifting.
#
#     work/w-front5/append_board.sh
set -eu
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
board="$repo/docs/BOARD.md"

if grep -q '^| \*\*2620\*\*' "$board"; then
    echo "already appended"
    exit 0
fi

{
    echo
    cat "$here/board_rows.md"
    echo
    cat <<'NOTE'
> **`#2633`–`#2659` are minted by nobody and are FREE.** Lane `w-front5` was
> allocated `#2620`–`#2659` and used thirteen (`#2620`–`#2632`). The unused
> twenty-seven are recorded as explicitly unminted rather than left to be
> inferred from a gap.
NOTE
} >> "$board"
echo "appended; BOARD.md is now $(wc -l < "$board") lines"
