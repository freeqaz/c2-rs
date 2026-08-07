#!/usr/bin/env python3
"""insert_rows.py — put lane w-relo's board rows in NUMERIC POSITION.

Lane tooling. `docs/BOARD.md` is the enumeration `ROADMAP.md` references and
never lists; rows are ordered by number inside the Open table, so a row appended
at the end is a row nobody finds. This inserts the rows in `board_rows.md` after
the highest-numbered row already in that table.

**It refuses to duplicate a number**, which is not decoration: this lane wrote
its rows as #996-#1005, master renumbered w-inread into exactly that range
(`b231974`, "repair the board namespace"), and the rebase would otherwise have
produced twenty rows and ten collisions. The anchor is computed rather than
hardcoded for the same reason.
"""

import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
BOARD = os.path.join(ROOT, "docs", "BOARD.md")
ROWS = os.path.join(os.path.dirname(os.path.abspath(__file__)), "board_rows.md")


def number_of(line):
    if not line.startswith("| **"):
        return None
    n = line[4:].split("*", 1)[0]
    return int(n) if n.isdigit() else None


def main():
    rows = [l for l in open(ROWS, encoding="utf-8").read().split("\n") if l.strip()]
    lines = open(BOARD, encoding="utf-8").read().split("\n")
    mine = [number_of(r) for r in rows]
    if None in mine:
        sys.exit("every row in board_rows.md must start with | **<number>**")
    have = {number_of(l) for l in lines} - {None}
    clash = sorted(set(mine) & have)
    if clash:
        sys.exit(f"already present, refusing to duplicate board numbers: {clash}")
    # Anchor on the highest-numbered row **of the `## Open` table**. BOARD.md has
    # three numbered tables (`Open`, `Declined and refuted`, `Done`) and the
    # highest number in the file is in the last of them — anchoring on that put
    # this lane's rows under `## Done`, which is a claim they do not make. The
    # section bound is therefore explicit and this refuses if it is not found.
    start = next((k for k, l in enumerate(lines) if l.strip() == "## Open"), None)
    if start is None:
        sys.exit("no `## Open` section in BOARD.md — refusing to guess a position")
    end = next(
        (k for k, l in enumerate(lines[start + 1 :], start + 1) if l.startswith("## ")),
        len(lines),
    )
    at = max(
        (k for k, l in enumerate(lines[start:end], start) if number_of(l) is not None),
        default=None,
    )
    if at is None:
        sys.exit("no numbered rows under `## Open` — refusing to guess a position")
    lines[at + 1 : at + 1] = rows
    open(BOARD, "w", encoding="utf-8").write("\n".join(lines))
    print(f"inserted {len(rows)} rows ({mine[0]}-{mine[-1]}) after line {at + 1}")


main()
