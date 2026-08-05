#!/usr/bin/env python3
"""Insert this lane's board rows after the last existing row. Idempotent."""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
board = os.path.join(REPO, "docs/BOARD.md")
rows = open(os.path.join(HERE, "board_rows.md")).read().rstrip("\n")

s = open(board).read()
if "**796**<sub>w-varloop</sub>" in s:
    print("already present; nothing to do")
    sys.exit(0)
anchor = "| **795**<sub>w-sched2</sub>"
i = s.index(anchor)
j = s.index("\n", i)
s = s[:j + 1] + rows + "\n" + s[j + 1:]
open(board, "w").write(s)
print("inserted %d rows" % (rows.count("\n| **") + 1))
