#!/usr/bin/env python3
"""rows_insert.py — add this lane's board rows in NUMERIC POSITION.

Board rows are placed by number, not appended (`docs/BOARD.md`'s own rule and
the lane brief's). This inserts #1197–#1206 immediately after the highest-numbered
row of the same table, and refuses hard if any of the numbers is already taken —
a renumber tool that fails soft is how two lanes end up on one number.
"""
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
BOARD = os.path.join(ROOT, "docs", "BOARD.md")

ROWS = open(os.path.join(HERE, "rows.md")).read().rstrip("\n").split("\n")


def main():
    src = open(BOARD).read()
    taken = set(int(m) for m in re.findall(r"^\| \*\*(\d+)\*\*", src, re.M))
    mine = [int(re.match(r"\| \*\*(\d+)\*\*", r).group(1)) for r in ROWS]
    clash = sorted(set(mine) & taken)
    if clash:
        raise SystemExit("REFUSING: board numbers already taken: %s" % clash)
    if len(set(mine)) != len(mine):
        raise SystemExit("REFUSING: duplicate numbers in rows.md")
    lines = src.split("\n")
    # The anchor: the LAST row line of the main table, i.e. the last line that
    # starts a numbered row before the "## Unclear" heading.
    end = next(i for i, l in enumerate(lines) if l.startswith("## Unclear"))
    last = max(i for i in range(end) if lines[i].startswith("| **"))
    out = lines[: last + 1] + ROWS + lines[last + 1 :]
    open(BOARD, "w").write("\n".join(out))
    print("inserted %d rows after line %d (%s…)"
          % (len(ROWS), last + 1, lines[last][:28]))
    return 0


if __name__ == "__main__":
    sys.exit(main())
