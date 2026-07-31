# W33 — the multi-argument floating-point tail call

    Tag:       W33
    Slug:      fp-multiarg
    Date:      2026-07-31
    Fixtures:  w33_fp_multi.cpp w33_fp_multi_neg.cpp
    Census:    TBD
    Record:    docs/CODEGEN_FP_ARGS.md §1.2 and §5 (the item this rung takes)

## Estimate, recorded before the scan

**Estimate: +14,000. Bias: LOW.**

The bucket is `docs/CODEGEN_FP_ARGS.md` §6's **26,136** — the bodies that parsed
as `MultiArgTailCall` under that section's lax FP-type counterfactual. W31's
lesson is that a bucket is not a sample of the construct, it is a sample of the
construct **that already passed some filter**, so this is what mine had already
been filtered by, in the order it matters:

1. every argument is a **bare formal LOAD** — no computed argument, no global,
   no local, no nested call result;
2. `call-arg-outer-formal`: no argument names a formal at index ≥ the argument
   count. That list has a member function's `this` at index 0, so the filter
   removes **every non-static member function with two or more arguments** and
   every call that passes the formals with a gap;
3. no duplicated argument;
4. **at most one** non-trivial cycle;
5. that cycle is at most **three** elements;
6. the lax type gate, which admits cross-width and cross-file `2C` conversions a
   strict rung refuses.

So what survives is: free functions forwarding a permutation of their leading
*n* formals, at least one of them floating-point.

My class differs from that bucket in **both directions**, which is why the sign
of the estimate is the honest uncertainty rather than the size:

* **smaller** — I require *every* argument to be floating-point. A call that
  also passes a GPR argument is refused, and that is the split this rung exists
  to make.
* **larger** — `arg_sources` here indexes the **FP file**, not the formals list.
  `this` is outside that file entirely and a non-FP formal does not occupy a
  slot in it, so filters (2)'s two largest exclusions — the member function and
  the leading `int`/pointer formal — are free to me. Also larger at (5): the
  gate is the number of **local minima**, not the cycle length, so unimodal 4-
  and 5-cycles are in class.

Guess: all-FP-argument at roughly 45 % of 26,136 ≈ 11,800, plus an unbounded
add-back for the member functions and non-FP-leading formals that filter (2)
removed and this recognizer restores. **+14,000, biased LOW**, and the term I
cannot bound is the add-back.

## Outcome

TBD
