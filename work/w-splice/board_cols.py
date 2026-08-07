#!/usr/bin/env python3
"""board_cols.py — fold w-splice's rows to the Done section's 4-column shape.

Lane w-splice scratch. `docs/BOARD.md`'s Done section header is
`| # | item | number | where settled |` — four columns. The rows this lane
inserted carried a fifth ("why it matters"), which some older rows also do but
the immediate neighbours (#957-#965) do not. Rather than leave a ragged table,
the fifth column is folded into the fourth behind an em dash, which loses
nothing and renders.
"""

import re

p = "docs/BOARD.md"
out = []
n = 0
for line in open(p):
    m = re.match(r"^\| \*\*(98[6-9]|99[0-5])\*\*<sub>w-splice</sub> \|", line)
    if m:
        cells = re.split(r"(?<!\\)\|", line.rstrip("\n"))
        # ['', num, item, number, where, why, '']
        if len(cells) == 7:
            cells = cells[:4] + [cells[4].rstrip() + " —" + cells[5]] + cells[6:]
            line = "|".join(cells) + "\n"
            n += 1
    out.append(line)
open(p, "w").writelines(out)
print("folded %d rows" % n)
