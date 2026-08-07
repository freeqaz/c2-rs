#!/usr/bin/env python3
"""addrows.py — insert lane w-align's board rows #1117-#1121 at numeric position.

Placement, per the sections' own conventions:

  Open      #1120 (the ALIGN_16 refusal, a live price) goes immediately after
            #1110, the row it is the residue of.
  Declined  #1118 (a refuted inference) at the head of its section, which runs
            newest-block-first.
  Done      #1117, #1119, #1121 at the head of the Done table.

And #1110 itself is marked CLOSED in place, with its original text struck
through and a pointer to #1117 — the convention #322 and #884 established.

Fails HARD if an anchor is missing or a number is already taken; it does not
skip. (The brief: "renumber tool must fail hard, not skip.")
"""
import re
import sys

P = "docs/BOARD.md"
text = open(P, encoding="utf-8").read()

for n in range(1117, 1122):
    if re.search(r"^\| \*\*%d\*\*" % n, text, re.M):
        sys.exit("FATAL: #%d is already taken" % n)

rows = {s: open("work/w-align/rows/%s.md" % s, encoding="utf-8").read().rstrip("\n")
        for s in ("1117", "1118", "1119", "1120", "1121")}

# ---- mark #1110 CLOSED, in place ------------------------------------------
lines = text.split("\n")
try:
    i1110 = next(i for i, l in enumerate(lines) if l.startswith("| **1110**"))
except StopIteration:
    sys.exit("FATAL: anchor row #1110 not found")
old = lines[i1110]
head, rest = old.split(" | ", 1)
item, rest2 = rest.split(" | ", 1)
lines[i1110] = (
    head
    + " | **CLOSED 2026-08-08 by lane `w-align` — see #1117, and #1119 before"
    + " quoting this row's price.** ~~"
    + item.strip()
    + "~~ | "
    + rest2
)

def index_of(prefix):
    try:
        return next(k for k, l in enumerate(lines) if l.startswith(prefix))
    except StopIteration:
        sys.exit("FATAL: anchor row %r not found" % prefix)

# Later anchors first so earlier indices stay valid. #1118 and the Done block go
# BEFORE their anchors — both sections run newest-block-first and this block is
# newer than w-rdata3's (#1107) and w-fnbyte's (#880).
i = index_of("| **880**")
lines[i:i] = [rows["1117"], rows["1119"], rows["1121"]]     # Done head
i = index_of("| **1107**")
lines[i:i] = [rows["1118"]]                                 # Declined head
i = index_of("| **1110**")
lines[i + 1:i + 1] = [rows["1120"]]                         # Open, beside its parent

open(P, "w", encoding="utf-8").write("\n".join(lines))
print("inserted 5 rows: 1117 1119 1121 (done), 1118 (declined), 1120 (open); "
      "#1110 marked CLOSED")
