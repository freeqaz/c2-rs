#!/usr/bin/env python3
"""insrows.py — insert this lane's board rows in NUMERIC POSITION, failing hard.

Board number ranges have collided twice this wave, so this refuses rather than
skips: it re-derives every `| **N**` id already in the file, asserts none of the
rows being inserted is among them, asserts the rows are contiguous and sorted,
and asserts the insertion point is immediately after the largest existing id
below the range.  Any violation exits non-zero and writes nothing.

    usage: insrows.py <BOARD.md> <rows.md>
"""
import re
import sys

ID = re.compile(r"^\| \*\*(\d+)\*\*")


def main():
    boardp, rowsp = sys.argv[1], sys.argv[2]
    board = open(boardp).read().split("\n")
    rows = [l for l in open(rowsp).read().split("\n") if l.strip()]

    new = []
    for l in rows:
        m = ID.match(l)
        if not m:
            raise SystemExit("row does not start with | **N**: %.60s" % l)
        new.append(int(m.group(1)))
    if new != sorted(new) or new != list(range(new[0], new[0] + len(new))):
        raise SystemExit("rows are not a contiguous ascending range: %s" % new)

    have = {}
    for i, l in enumerate(board):
        m = ID.match(l)
        if m:
            have.setdefault(int(m.group(1)), []).append(i)
    dupes = [n for n in new if n in have]
    if dupes:
        raise SystemExit("COLLISION — these ids already exist: %s" % dupes)

    below = [n for n in have if n < new[0]]
    if not below:
        raise SystemExit("no existing row below %d" % new[0])
    anchor = max(below)
    at = max(have[anchor]) + 1
    above = [n for n in have if n > new[-1]]
    print("anchor row **%d** at line %d ; inserting %d rows after it ; "
          "%d ids above the range stay put" % (anchor, at, len(new), len(above)))

    out = board[:at] + rows + board[at:]
    open(boardp, "w").write("\n".join(out))
    print("wrote %s (%d -> %d lines)" % (boardp, len(board), len(out)))


if __name__ == "__main__":
    main()
