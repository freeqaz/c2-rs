#!/usr/bin/env python3
"""addrows.py — insert lane w-rdata3's board rows #1107-#1112 at numeric position.

The Open section is anchored by #160 (the section-vocabulary row these rows
price) and is not globally sorted; the Declined section runs newest-block-first
(1067..1076 at its head). So the three OPEN rows go immediately after #160, the
row they are the price of, and the three DECLINED rows go at the head of their
section, which is both the section's own convention and numerically highest
first. Fails hard if an anchor is missing or a number is already taken.
"""
import re
import sys

P = "docs/BOARD.md"
text = open(P, encoding="utf-8").read()
lines = text.split("\n")

for n in range(1107, 1113):
    if re.search(r"^\| \*\*%d\*\*" % n, text, re.M):
        sys.exit("FATAL: #%d is already taken" % n)

OPEN_ROWS = [open("work/w-rdata3/rows/%s.md" % s, encoding="utf-8").read().rstrip("\n")
             for s in ("1109", "1110", "1112")]
DECL_ROWS = [open("work/w-rdata3/rows/%s.md" % s, encoding="utf-8").read().rstrip("\n")
             for s in ("1107", "1108", "1111")]

try:
    i160 = next(i for i, l in enumerate(lines) if l.startswith("| **160** |"))
except StopIteration:
    sys.exit("FATAL: anchor row #160 not found")
try:
    i1067 = next(i for i, l in enumerate(lines) if l.startswith("| **1067**"))
except StopIteration:
    sys.exit("FATAL: anchor row #1067 not found")

# Insert the later anchor first so the earlier index stays valid.
lines[i1067:i1067] = DECL_ROWS
lines[i160 + 1:i160 + 1] = OPEN_ROWS

open(P, "w", encoding="utf-8").write("\n".join(lines))
print("inserted 6 rows: 1107 1108 1111 (declined), 1109 1110 1112 (open)")
