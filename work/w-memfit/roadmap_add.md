
### 10.26.8 w-memfit — the memcpy contradiction is RESOLVED, and the family's target was the wrong half of a pair (2026-08-09)

Two lanes had measured c2's block-move expansion and reached opposite
conclusions, and the disagreement had already propagated: `w-park` §6.1 cites
`w-memcpy` for *"the rule was measured NOT TO EXIST"* and declined
`?mmioGetInfo` on that basis. **`w-memfit` scored `wb-memcpy`'s read decision
function against `w-memcpy`'s own frozen cells, on `w-memcpy`'s own
denominator, and it is 408 of 408** — against 182/232 for the best rival
`w-memcpy` froze, 114/232 for the id-keyed rule and 114/176 for the sub-class
GRID-M2 refuted (**#2068**). The control ran first and reproduces
`work/w-memcpy/scorem.txt` exactly, so the numbers are comparable.

**`w-memcpy`'s "no rule fits" was a rule-space limitation, and the axis was the
DIVISOR** — every one of its six rivals is a predicate on `size`, on the id, or
on constancy, and not one divides by anything. **Favor-speed, the obvious
candidate, is NOT it** and the negative was registered in advance at p = 0.90:
both grids are `/O1`-only, where `T = 5` either way, so varying it changes zero
of the 408 (**#2069**).

**The reading needed two corrections, both wrong-emit shapes, both invisible to
every cell graded before this lane.** 100 new cells in two grids, each frozen
before its own first `cl.exe`: the divisor is **clamped at 8 above** — `c1xx`
writes `0x10` for a `__declspec(align(16))` pointee and the divisor stays 8, so
the reading taken literally predicts `inline` on 5 cells that call (**#2062**)
— and it is the **`min` of the two hints**, not either operand's, so a port
keyed on the destination's emits `inline` on 18 of 56 (**#2063**). Every cell
in all 624 graded before had naturally aligned pointees and two agreeing
operands. Corrected, the rule is **724/724 over five grids**, and **no
`DISCLOSURE` row is carried** because every element of it is derivable from obj
and IL alone (**#2070**).

**`mmio.cpp` is DECLINED and its price moved the cheap way.** `w-park`'s own
ladder file, re-run unmodified at this base, reads **4/5 in class** where
`w-park` recorded 3/5 — the rung it priced as unpaid was paid by the widening
`w-park` shipped in the same commit (**#2065**). Two rungs this lane added show
the `2C` conversion is free too, and both cells compile **byte-identical to the
84-byte `?mmioGetInfo` pin already in `crates/`**, with `c2rs gap` reading
`match 1` on the one whose callee is ordinary. **`?mmioGetInfo`'s entire
remaining distance is one word in a symbol table** — and converting it moves
the TU verdict by zero, because the other two bodies re-derive at 1/5 and 3/7
(**#2066**).

**The successor ordering this leaves is one row and it is not `memcpy`.** On
the emitted column — #2020's rule — `expr-intrinsic-memset` is **3,749 over 497
TUs** and `expr-intrinsic-memcpy` is **99 over 83**: memset is **38×**, and the
pair at 3,848 is **7× the last rung this board recommended** (§10.26.7's R2 at
544). `w-memcpy` §6.1 wrote *"the sibling selector, 173, whose workload
footprint is 10× memcpy's"* and then declined the family on `memcpy`'s name;
`wb-memcpy` read both and titled itself for one; the commission named one.
**That is a ninth ranking artifact and a fourth mechanism — the wrong member of
a two-member family** (**#2067**). `memset` is not the same rung: one pointer
operand means one hint, so #2063's correction does not apply to it at all, and
its construct count is unmeasured — quoting 3,749 as a rung's population would
be #2030's error one family over.

**What is explicitly NOT recommended** is shipping the rule into `crates/`. The
decision is not the blocker; the two that remain are a reader production for
the `0x40` statement and an emitter model for a symbol c2 mints itself, and a
lowering that implements the decision without the mint converts nothing while
adding a live predicate to a fail-closed path.

**This lane ships no `crates/` change**: `git diff --stat 8dd1a577..HEAD --
crates` is empty at its tip, and the `#[test]` count is 1,355 at both ends.

[`rungs/2026-08-09-w-memfit.md`](rungs/2026-08-09-w-memfit.md).
