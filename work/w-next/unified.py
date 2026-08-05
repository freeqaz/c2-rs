#!/usr/bin/env python3
"""unified.py — score ONE key against every mixed-kind cell both grids produced.

`alloc.rs` states clause 1 (use count, descending) and clause 2 (register-
derived before constant) as two clauses with, in its own words, "opposite signs
inside one sort".  Over the mixed-kind run this lane measured, a SINGLE sort key
reproduces both:

    rank by   uses + (1 if kind is RegisterDerived else 0),  descending

i.e. **clause 2 behaves as a bonus worth exactly one use**, which also makes it
the tie-break clause 2 already is (a tie in `uses` becomes a win for the
register-derived producer, by +1).

Scored below against the 16 cells of `gapgrid.py` and the 8 mixed cells of
`allocgrid.py` — 24 rows, 5 of them duplicates by (reg,const), all observed.

THIS LICENSES NO EMIT.  `alloc.rs` still refuses the mixed run and this lane
ships nothing; the key is a candidate for the next lane to break, not a rule.
"""

CELLS = [
    # (name, reg_uses, const_uses, observed winner of r11)
    ("anchor",           2, 1, "reg"),
    ("diff-reg2-const1", 2, 1, "reg"),
    ("diff-reg3-const1", 3, 1, "reg"),
    ("diff-reg1-const2", 1, 2, "reg"),
    ("diff-reg1-const3", 1, 3, "const"),
    ("tie-1-1",          1, 1, "reg"),
    ("tie-2-2",          2, 2, "reg"),
    ("tie-1-1-swapped",  1, 1, "reg"),
]

GAP = {
    (1, 1): "reg",   (1, 2): "reg",   (1, 3): "const", (1, 4): "const",
    (2, 1): "reg",   (2, 2): "reg",   (2, 3): "reg",   (2, 4): "const",
    (3, 1): "reg",   (3, 2): "reg",   (3, 3): "reg",   (3, 4): "reg",
    (4, 1): "reg",   (4, 2): "reg",   (4, 3): "reg",   (4, 4): "reg",
}
for (r, k), w in sorted(GAP.items()):
    CELLS.append(("g%dx%d" % (r, k), r, k, w))

miss = 0
for name, r, k, obs in CELLS:
    pred = "reg" if (r + 1) >= k else "const"
    if pred != obs:
        miss += 1
        print("  **MISS** %-18s reg=%d const=%d observed=%s predicted=%s"
              % (name, r, k, obs, pred))

print("unified key:  uses + (register-derived ? 1 : 0), descending")
print("  %d cells scored, %d miss" % (len(CELLS), miss))
print("  clause 1 alone would MISS: %d"
      % sum(1 for n, r, k, o in CELLS
            if ("reg" if r > k else ("const" if k > r else o)) != o))
print("  clause 2 alone would MISS: %d"
      % sum(1 for n, r, k, o in CELLS if o != "reg"))
