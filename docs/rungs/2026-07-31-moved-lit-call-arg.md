# WLB — `g2(b, 7)`: the moved formal beside the literal, and the hoist

    Tag:       WLB
    Slug:      moved-lit-call-arg
    Date:      2026-07-31
    Fixtures:  wlb_moved_formal.cpp wlb_moved_formal_neg.cpp
    Census:    696,551 → 697,251 (28.29 % → 28.31 %), +700
    Record:    this file, and `docs/IL_CALL_IN_EXPR.md` §26.7

WLA's residue, decomposed rather than inherited. It refused 733 functions under
`call-arg-lit-permuted` and labelled that a **ceiling, not a yield**. One re-key
of the same refusal site says what the 733 are:

| functions | shape |
|---:|---|
| **699** | two slots, the literal in slot 1, slot 0 wanting a formal not in r3 |
| 34 | four or more slots, every formal moving to a lower slot |
| **0** | anything else |

95 % of the row is one shape, it is the **only** list two slots can take once a
formal is out of place, and both of its cells are captured.

## What it admits, and what it refuses

Admits exactly two argument slots: a literal in slot 1, and slot 0 wanting a
formal that arrives in some other argument register. Read off
`work/WLA/probe/p2.cpp`, `/O1 /GS- /c`:

```text
  void f(int a,int b)       { g2(b, 7); }   7c832378 mr r3,r4 · 38800007 li 4,7
  void f(int a,int b,int c) { g2(c, 7); }   38800007 li 4,7   · 7ca32b78 mr r3,r5
```

**The order is not fixed, and that is the whole rung.** c2's default is highest
destination first — which puts the `li` in front — and it **hoists** the move
ahead of the `li` exactly when the `li`'s destination is the register holding the
value the move reads. That is the same hoist/trail rule the callee-saved copies
already follow (`call_seq_parts`), recognised rather than discovered. The
deciding variable is one boolean and **both of its values are witnessed**, which
is what makes two slots a complete cell rather than a sample.

Refuses everything else under `call-arg-lit-permuted` — **34 functions**, down
from 733 — and the reason the bound is at two slots is measured, not cautious:

```text
  void f(int a,int b,int c) { g3(c, b, 7); }  mr r3,r5 · li r5,7
  void f(int a,int b,int c) { g3(b, c, 7); }  mr r3,r4 · mr r4,r5 · li r5,7
  void f(int a,int b,int c) { g3(c, a, 7); }  mr r11,r5 · mr r4,r3
                                              · li r5,7 · mr r3,r11
```

The first two follow the hoist; the third — one formal moving **up** while
another moves **down** — breaks through r11 and emits the `li` *inside* the walk.
Any rule fitted to the first two mis-emits the third, so all three stay refused.

## Estimate vs outcome

> **Estimate: +699, biased LOW by at most 1.** The re-key measured this exact
> population (`xperm-cycle-n2` 699 `:eof`) and the implementation gates on
> exactly it; the one `:mid` body of the same shape is the only thing that could
> push it up.

**Outcome +700.** The low bias was that `:mid` body and its population was
exactly 1. 699 TUs gained, the largest single gain is 2, and `call-arg-lit-permuted`
went 733 → 34 and 2 → 1 with no other key moving by one function.

## Gate evidence

| lane | result |
|---|---|
| `cargo test --workspace` | **523 pass, 0 fail** |
| `c2rs bench` | **201 pass, 0 fail, 0 error** |
| `scripts/gate.sh --jobs 4` | **12/12 PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT**, 2,412 fixture-verdicts, 0 mismatch in every lane |
| `scripts/expr_sweep.sh` | **13,880 cases, 0 mismatches**; the new fragment is 72 cases, **43 of them grade `Match`** |
| 878-TU workload scan | 878 rows graded, `fn_total` 2,462,571, match 6 / capture-fail 7, **mismatch 0**, census **696,551 → 697,251**, **census/gate disagreement 0**, 0 TUs changed class, **0 functions lost** |
| fixtures, `c2rs census` | `wlb_moved_formal.cpp` **24/24**, whole obj byte-exact; `wlb_moved_formal_neg.cpp` **0/14**, `Port=NotImplemented` |

## Found and not taken

1. **The three-slot family, 34 functions**, all four-slot-or-wider shifts on this
   workload. Its lowering is *partly* captured (two of three arrangements follow
   the hoist) and the third needs the r11 walk with the `li` interleaved. At 34
   functions it is not worth its own grading; recorded with the captures so the
   next reader does not re-derive them.
2. **`callseq-multiarg-lit`, still 0 on this workload** — the framed call's
   literal, unchanged by this rung.
3. The `call-arg-*` family is now **34 functions in total** on the 878-TU
   workload, from 5,544 two rungs ago. Whatever ranks next in the call seam, it
   is not an argument-list question.
