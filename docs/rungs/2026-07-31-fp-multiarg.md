# W34 — the multi-argument floating-point tail call

    Tag:       W34
    Slug:      fp-multiarg
    Date:      2026-07-31
    Fixtures:  w34_fp_multi.cpp w34_fp_multi_neg.cpp
    Census:    549,148 → 575,284 (22.30 % → 23.36 %), +26,136
    Record:    docs/CODEGEN_FP_ARGS.md §1.2 and §5 (the item this rung takes)

`return g(x1, …, xn);` and `g(x1, …, xn);` — the whole body, no frame, where the
floating-point arguments are a permutation of the FP file and **no
general-purpose argument moves**. It is the other half of the family W31 opened
(`docs/rungs/2026-07-31-fp-tail.md`), the half that section §5 had ranked at
26,136 and blocked on "the two files' interleaved move schedule".

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
was the honest uncertainty rather than the size:

* **smaller** — I refuse any call whose *other* register file has to move.
* **larger** — `arg_sources` here indexes the **FP file**, not the formals list.
  `this` is outside that file entirely and a non-FP formal does not occupy a slot
  in it, so filter (2)'s two largest exclusions — the member function and the
  leading `int`/pointer formal — are free to me. Also larger at (5): the gate is
  the number of **local minima**, not the cycle length, so unimodal 4- and
  5-cycles are in class.

Guess: all-FP-argument at roughly 45 % of 26,136 ≈ 11,800, plus an unbounded
add-back for the member functions and non-FP-leading formals that filter (2)
removed and this recognizer restores. **+14,000, biased LOW**, and the term I
could not bound was the add-back.

## Outcome, and the estimate against it

**+26,136 — LOW by 12,136, i.e. 1.87× the estimate.** The direction was called
correctly and the magnitude was not, for the second rung running.

The rung shipped in two steps and both are worth reporting, because the second
was a decision *taken on the measurement* rather than planned:

| step | class | gain |
|---|---|---:|
| as scoped — **every argument floating-point** | +17,424 | LOW by 1.24× |
| widened on the counterfactual — **no GPR argument moves** | **+26,136** | LOW by 1.87× |

**+26,136 is exactly `CODEGEN_FP_ARGS.md` §6's lax counterfactual figure**, to
the function. That section hedged its 85,231 as "an upper bound, and the strict
rung is smaller by an unmeasured amount"; W31 resolved its half at 98.4 % of the
bound and this half comes in at 100.0 %. The residue really is the identity: on
this workload every multi-argument FP tail call already passes every gate the
strict rung adds.

## What it admits, and what it refuses

The rule is `docs/CODEGEN_FP_ARGS.md` §0 — **two numberings, neither of them the
formal's index** — consumed through the one locator `sy::arg_classes` and its two
readers `fp_reg_of` / `gpr_reg_of`. The FP *sources* are numbered over the FP
formals alone; the FP *destinations* over the FP arguments alone; the GPR
numbering counts an FP parameter's slot even though it fills no register.

```text
  float id2(float a,float b)          { return g2f(a,b); }   (nothing)  b g2f
  float sw2(float a,float b)          { return g2f(b,a); }   fmr f0,f2 ; fmr f2,f1 ; fmr f1,f0
  float rt3(float a,float b,float c)  { return g3f(b,c,a); } fmr f0,f2 ; fmr f2,f3 ; fmr f3,f1 ; fmr f1,f0
  float u4 (a,b,c,d)                  { return g4f(b,c,d,a); }
                                        fmr f0,f2 ; fmr f2,f3 ; fmr f3,f4 ; fmr f4,f1 ; fmr f1,f0
  void  mx7(int a,int b,float c,float d) { gviiff(a,b,d,c); }
                                        fmr f0,f2 ; fmr f2,f1 ; fmr f1,f0   — the GPRs cost nothing
  float C::m(float a,float b) const   { return g2f(b,a); }   the same three words
```

`C::m` is the case the **integer** multi-argument rung cannot reach at all: its
`arg_sources` indexes a formals list with `this` at index 0, so every member
function with two or more arguments trips `call-arg-outer-formal`. Indexing the
FP file instead makes it free.

Refused, each because a capture shows it emits something else:

| refused | what c2 emits |
|---|---|
| a **GPR argument that moves** | the two files' moves **interleave** — `int f(int a,int b,float c,float d){ return gif2(b,a,d,c); }` is `fmr f0,f2 ; mr r11,r4 ; mr r4,r3 ; fmr f2,f1 ; mr r3,r11 ; fmr f1,f0` (§1.1) |
| a permutation with **two local minima** | a second scratch, **f13**, and then the two chains interleave in an order that is open in both files |
| a **narrowing** anywhere in the list | `double a,b,c → g3(float,float,float)(b,c,a)` is `fmr f0,f2 ; fmr f13,f3 ; frsp f3,f1 ; frsp f1,f0 ; frsp f2,f13` — **five moves and two scratches** where the same permutation without the conversion is four and one. One type change, a different lowering |
| a value passed **twice** | `g2f(a,a)` is a plain copy `fmr f2,f1`, a copy graph and not a permutation |
| a source **outside** the destination range | `float f(float a,float b,float c){ return g2f(b,c); }` is a shift, `fmr f1,f2 ; fmr f2,f3` |
| a **computed** argument, an FP **literal**, a **result** conversion | W31's own refusals in the multi-argument position |

## The permutation rule, measured over the complete grid

`scripts/gt_fpperm.py` is the FP twin of `scripts/gt_argperm.py`: the complete
permutation grid at n = 2, 3, 4, 5 (152 cells), scored against
`docs/CODEGEN_ARG_PERM.md` §2's GPR model with the FP file's numbering
substituted and nothing else changed. **§1.1 asserted "the FP file uses f0
exactly as the GPR file uses r11, and the shapes match one for one" on two
captures. That is right about the mechanism and wrong about the order.**

```text
  readback = park (f0 then f13)          0 / 2   0 / 6   0 / 24   26 / 120
  readback = ascending local minimum     0 / 2   0 / 6   2 / 24   40 / 120
  readback = descending  (the GPR rule)  0 / 2   0 / 6   3 / 24   47 / 120
```

Three findings, none of them available before this grid:

* **the second FP scratch is `f13`.** No capture had one; §1.1 had `f0` alone,
  which is exactly the evidence that carried the GPR file's "one temp breaks the
  cycle" rule to length 3 and then failed.
* **the read-back order is the park order, not the GPR file's descending
  minimum.** The two files agree on the parks and disagree on the restores, and
  they cannot disagree below n = 4 — one scratch makes all three candidates the
  same sequence.
* **the residue is the same 26 of 120 cells at n = 5, in both files**, and it is
  the same *kind* of residue: identical instruction multisets in a different
  order, the independent-chain interleaving `docs/CODEGEN_ARG_PERM.md` §2.1
  declines to fit a fourth time. It is not fitted here either.

**Every cell with one scratch is exact — 126 of the 152, and 100 % of the
one-minimum subset.** So the gate is the number of local minima and not the cycle
length: unimodal 4- and 5-cycles are in class here where the GPR file's rung
stops at three, and their two-minima neighbours (`w34_fp_multi_neg.cpp`'s `two`
and `vall`) are refused. A length limit would have been a fit; the minimum count
is the thing that was measured.

## Where the +26,136 came from, and what it says about the rung

Decomposed by counterfactual (scratch, reverted; the refused sub-populations
routed to their own census keys, which changes no acceptance at all — every one
of these bodies was blocked already, so only the key moves and both the in-class
count and the disagreement counter are unaffected):

| sub-population | functions |
|---|---:|
| all arguments FP, **identity** permutation — a bare `b <callee>` | **17,424** |
| a GPR argument beside them, **nothing moves in either file** | **8,712** |
| all arguments FP, a **non-identity** permutation | **0** |
| a mixed call where the **GPR** file moves (with or without FP moves) | **0** |
| a **narrowing** inside a multi-argument list | **0** |
| **two local minima** (two cycles, or a cycle with a valley) | **0** |
| a **duplicated** argument | **0** |
| a source **outside** the destination range | **0** |

**The permutation solver — the whole hard part of this rung, the f0 cycle, the
local-minimum boundary, the 152-cell grid — earns 0 functions on this workload.**
Every one of the 26,136 is a forwarding shim that emits nothing at all. It is
shipped anyway, and for exactly the reason W31 shipped the `frsp` that also
earned 0: the identity and the permutation are the *same IL production* and one
`arg_sources` comparison apart, so a rung that admitted the free half without
deciding the other would have been one comparison away from a wrong-bytes emit.
It is graded by the fixtures and by 2,313 sweep cases instead of by the census.

That is now the second time this has happened and it is worth stating as a rule:
**on a real workload the value of a marshalling rung is in what it refuses to
emit, not in what it emits.** The census ranks the population; it does not rank
the risk.

## The split, and why the boundary moved

W31 recommended splitting the family at "every argument is floating-point",
because a call with no GPR arguments has no GPR moves to interleave with. That
is sound and it is what was built first (+17,424). The counterfactual then said
the real boundary is one step further out and free: **no GPR argument *moves*.**
A marshalling with no moves in the other file has nothing to interleave either,
and the capture confirms the FP half is then byte-identical to the pure-FP one:

```text
  int f(int a,int b,float c,float d) { g(a,b,d,c); }   fmr f0,f2 ; fmr f2,f1 ; fmr f1,f0
```

— the same three words `float f(float a,float b){ g(b,a); }` emits. The gate is
`gpr_reg_of(classes, ix, base) == r(2 + slot)` per GPR argument, which is §0's
rule and had **no consumer at all** until this rung (`gpr_reg_of` was dead code:
the fact was measured, documented and unused). `base` is r4 for a member
function, which is why `void C::m(int k, float a, float b) { g(k,a,b); }`
refuses — `k` is in r4 and the call wants it in r3 — and that row is in the
negative fixture because the positive fixture's member functions pass only FP
arguments and cannot see it.

## Gate evidence

| lane | result |
|---|---|
| `cargo test --workspace --release` | **446 pass, 0 fail** (438 before) |
| `c2rs bench` | **165 pass, 0 fail, 0 error** (163 before) |
| `scripts/mode_lane.sh` `/Ox` / `/O1` / `/O2` / `/Ox /Gy` | **77 / 75 / 75 / 75 match, mismatch 0** in every lane (76/74/74/74 before) |
| `scripts/expr_sweep.sh` | **9,986 checked, 0 mismatches** — printed = generated = on disk (7,673 + 2,313 W34) |
| `scripts/cross_sweep.sh` | **0 mismatches**, and it discovered `fp-multiarg-tail-call` / `fp-multiarg-tail-call-perm` from `census.rs` without being told |
| 878-TU workload scan | match 6, **mismatch 0**, census **575,284 / 2,462,571 = 23.36 %**, **census/gate disagreement 0** |
| blocker histogram | **580 keys, none added or removed**; the sum of the key deltas is exactly **−26,136** over exactly two keys — `expr-load-type-8885` −13,794 and `expr-load-type-8645` −12,342. The eleventh rung running where the bucket drop equals the gain to the function |
| frame class | all 26,136 are `calls-1`; `calls-2plus` is unmoved |
| fixtures, `c2rs census` | `w34_fp_multi.cpp` **27/27**, `w34_fp_multi_neg.cpp` **0/11**; both objs `Port=Match` / `Port=NotImplemented` with `ReferenceReplay=ByteExact` |
| the permutation grid | 152 cells at n = 2…5, `scripts/gt_fpperm.py --pure --model`: **0 refutations on every one-scratch cell**, 26 two-scratch cells refuted and refused |

The label counter is untouched: this rung's stride is 1 per function, the same
as every other tail call's, plus the TU's `_fltused` — which
`touches_floating_point` now has a fourth producer for
(`IlFunction::fp_arg_sources`). `docs/LABEL_COUNTER.md` §1.1's measured surcharge
table is not extended.

`fixtures/cpp/w31_fp_tail_neg.cpp`'s `p1` row — "the two-file permutation",
which was a *single*-file one — became in class the moment this rung landed and
is now the genuine two-file case. That it had to be found by running the census
rather than by reading the file is this repo's own recurring lesson about
negative fixtures: **a refusal that quietly became an acceptance looks exactly
like a refusal.**

## Found and not taken

Ranked, sized, with the frame axis applied. This is the section the next rung
reads first.

| item | size | frame axis | what stops it |
|---|---:|---|---|
| **a multi-argument FP call whose result is CONSUMED** (`x = g(a,b);`, `return g(a,b)+1;`, a converted result) | **25,785** measured — the largest single item this rung uncovered, and the same order as the rung itself | mostly **framed**; the residue is decode-only, so the split between `calls-1` and `calls-2plus` inside it is **not** measured | it is not a tail call at all. The argument marshalling this rung ships is exactly what such a body needs in front of its `bl`, so the *argument* half is done; what is missing is the frame and the result plumbing. Whoever takes the framed FP body gets this for the cost of reusing `fp_permute_args_text` |
| a **GPR argument that moves**, beside FP ones | **0** on this workload | leaf | §1.1's interleaved schedule. Ranked here with its measurement because the number is the point: the question that blocked this whole rung for two sections **is worth nothing on this corpus**, and a future agent should not spend a day on it before re-measuring |
| a **non-identity FP permutation** | **0** | leaf | nothing — it is shipped. Recorded because it is the counterexample to "ship what the census ranks", for the second rung running |
| the **two-scratch** permutation (two cycles, or one with a valley) | **0** | leaf | the independent-chain interleaving, open in both register files (`CODEGEN_ARG_PERM.md` §2.1). The parks and the read-backs are exact; only the order between them is not |
| a **narrowing** inside a permutation | **0** | leaf | a schedule that changes with the type — five moves and two scratches where four and one would do. Would need its own grid over (cycle shape × which arguments narrow) |
| the **free result widening** (`double f(float a,float b){ return gf2(a,b); }`) | unmeasured, **< 25,785** and inside it | leaf | unchanged from W31: `eat_call_head` decodes the CALL token's return TYPE and discards it, and returning it needs a signature change in the spine-owned `shapes/calls.rs`. Still the cheapest item on this list by a wide margin |
| a **computed** FP argument in a multi-argument list | unmeasured, **0** measured for the whole-body case | leaf | `float_leaf_text`'s selector in argument position, plus its contraction and pooled-constant gates, plus the interaction with the permutation temp — which is genuinely new, because a computed argument needs a register the permutation may already be using |
| an FP formal past **f13** | 0 observed | frame | stack-homed formals |

**One document carries a number this rung supersedes.**
`docs/CODEGEN_FP_ARGS.md` §5's first row ("the FP **tail call**, multi-argument |
26,136") is now taken in full, and §1.1's "nothing here models a permutation in
both files at once" should point at §1.2 for what *is* modeled: a permutation in
the FP file beside a GPR file that does not move.
