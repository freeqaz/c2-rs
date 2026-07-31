# W-UNW-1 — per-function `.pdata`, and the label counter

    Tag:       W-UNW-1
    Slug:      unwind-pdata
    Date:      2026-07-30
    Fixtures:  wunw_framed_pair.cpp wunw_leaf_then_framed.cpp wunw_mixed_order.cpp wunw_two_leaves_framed.cpp wunw_float_neg.cpp
    Census:    unchanged (a codegen rung, not a decode one)
    Record:    docs/ROADMAP.md §6e; docs/OBJ_FORMAT_MVP.md §7

The prerequisite for #35 (general non-leaf lowering): byte-exact unwind data
for framed functions, in both sectioning modes, with the label counter derived
rather than pinned. The X360 record is 8 bytes — `BeginAddress` plus a packed
`PrologLen[7:0] | FuncLen[29:8] | ThirtyTwoBit[30] | ExceptionFlag[31]`, both
big-endian, with **no `.xdata` and no unwind-code array**. That is not the x64
shape and was established from c2's own output rather than assumed from it.

The label counter is the fact this rung exported to every later one: every
function in the TU consumes label slots, an FP-touching function 2 and an
integer leaf 1, so a function *ahead* of a framed one with the wrong stride is
six wrong bytes in an obj that still links. Swept by
`scripts/sweep.d/81-fp-beside-framed.py`, which exists because the cross
product is the only place that mis-emit lives.
