#!/usr/bin/env python3
"""Append this lane's board rows to docs/BOARD.md. Idempotent-checked: refuses if
#2020 is already present."""
import pathlib

root = pathlib.Path(__file__).resolve().parents[2]
board = root / "docs/BOARD.md"
rows = (root / "work/w-callprice/board_rows.md").read_text()
txt = board.read_text()
assert "**2020**<sub>w-callprice</sub>" not in txt, "already appended"
assert "> **`#2008`–`#2019` are minted by nobody and are FREE.**" in txt, "anchor"
tail = (
    "> **`#2033`–`#2039` are minted by nobody and are FREE.** Lane "
    "`w-callprice`\n> was allocated `#2020`–`#2039` and used thirteen "
    "(`#2020`–`#2032`). The unused\n> seven are recorded as explicitly unminted "
    "rather than left to be inferred\n> from a gap.\n"
)
board.write_text(txt.rstrip("\n") + "\n\n" + rows.rstrip("\n") + "\n\n" + tail)
print(f"appended {len(rows.strip().splitlines())} rows to {board.name}")
