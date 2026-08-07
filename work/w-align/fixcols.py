#!/usr/bin/env python3
"""fixcols.py — collapse each w-align board row to its SECTION's column count.

`docs/BOARD.md`'s three tables have different shapes — Done is
`# | item | number | where settled` (4) and Open / Declined are 5 — and a row
written to the wrong shape renders with a column silently dropped. Merges the
verdict cell into the number cell rather than truncating, so nothing is lost.
"""
TARGET = {"1117": 4, "1119": 4, "1121": 4, "1118": 5, "1120": 5}
for n, want in sorted(TARGET.items()):
    p = "work/w-align/rows/%s.md" % n
    s = open(p, encoding="utf-8").read().strip()
    assert s.startswith("| ") and s.endswith(" |"), n
    cells = s[2:-2].split(" | ")
    while len(cells) > want:
        cells[2:4] = [cells[2] + " · " + cells[3]]
    assert len(cells) == want, (n, len(cells))
    open(p, "w", encoding="utf-8").write("| " + " | ".join(cells) + " |\n")
    print(n, "->", want)
