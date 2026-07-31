# WCB — `return a->m() == b->n();`, the port's first Class B function

    Tag:       WCB
    Slug:      cmp-two-calls
    Date:      2026-07-31
    Fixtures:  wcb_cmp_two_calls.cpp wcb_cmp_two_calls_neg.cpp
    Census:    655245 → 661245 (26.61 % → 26.85 %), +6,000
    Record:    this file; the row's decomposition is `docs/CMP_PRODUCES_A_VALUE.md`

Two member calls in one expression, compared for equality. The **first call's
result is live across the second `bl`**, so the body saves two GPRs inline — the
first Class B function this port emits, and the class `gt_frame_class.py` counts
at 30,497 in c2's own output while the port converted **2**.

`docs/CMP_PRODUCES_A_VALUE.md` built the comparison half of this row yesterday,
measured it at **+0**, and reverted it — because the row is not
`p->m() <rel> k`, it is a **comparator**, and 90.7 % of it has a value live
across a call. This is that half.

## What it admits, and what it refuses

```text
  26 <m1> B9 <r1> <ptr4> [2C…] 99 <ptr4> 00  BD <ret> 00 <id>  4C
  26 <m2> B9 <r2> <ptr4> [2C…] 99 <ptr4> 00  BD <ret> 00 <id>  4C
  1F                      the `==` operator
  [ 2C <int4> 00 ]        an `int`/`unsigned` result converts; a `bool` one does not
  41 <TYPE> …             returned
```

```text
  bool f(const U* p, const U* q) { return p->m() == q->n(); }     76 B, F = 112
    mflr r12 ; stw r12,-8(r1) ; std r30,-24(r1) ; std r31,-16(r1) ; stwu r1,-112(r1)
    mr r31,r4 ; bl ?m ; mr r30,r3 ; mr r3,r31 ; bl ?n
    subf r11,r30,r3 ; cntlzw r10,r11 ; rlwinm r3,r10,27,31,31
    addi r1,r1,112 ; lwz r12,-8(r1) ; mtlr r12 ; ld r30 ; ld r31 ; blr
```

**Three things are measured here and none of them is guessable from the source.**

### 1. c2 chooses the call order, and it is neither source nor evaluation order

The two calls are emitted in the order c1xx **numbered their receivers**. The
source's left operand may be emitted first or second, and only the spine's two
`subf` operands record which was which — so `lhs_first` and the call order are
two independent facts on the shape. Twelve grid cells fix it
(`work/WCB/probe/p5.cpp`: every ordered pair of three pointer formals, both
source orders, with and without a leading `int` formal).

**The refuter for "ascending parameter index" is `this`.** `parse_params` puts
it at index 0 because that is the register it occupies; c1xx numbers it **after**
every declared formal:

```text
  bool H::q(const U* a) const { return m() == a->m(); }
    mr r31,r3 ; mr r3,r4 ; bl ?m@U ; mr r30,r3 ; mr r3,r31 ; bl ?m@H
```

`this` is r3 *and* `params[0]`, and `a`'s call still goes first — with the saves
**hoisted** in front of the marshalling, the arm `plan_saved_gprs` predicts for a
save whose source the marshalling overwrites, and the only shape in this family
that reaches it. `docs/GAPS.md` §6's recurring form, one field carrying two
facts: "which register" and "which symbol number".

**The rule went wrong twice before it was right, and each wrong version is
recorded because each is a live hazard for the next widening.**

* **Ordering by the token VALUE was a wrong-bytes emit.** `read_token_var`'s
  two-byte form is little-endian, so consecutive tokens `0x09FF, 0x0A00` come
  back as `0xFF09, 0x000A` and the order **inverts at every low-byte wrap**. It
  passed 287 sweep cases, 19 fixture functions, four mode lanes and the workload
  scan, and it was `Port=Mismatch @ offset 8` on the first segment transcribed
  into a unit test — because that TU happened to straddle a boundary. Section 12
  of the sweep fragment now sweeps 264 consecutive symbol counts and reproduces
  **exactly 2** failing alignments per wrap, which is what a boundary rule should
  cost.
* **Refusing the four-byte token form cost 5,971 of the 6,000.** The first repair
  ordered by an "allocation key" that inverted the two-byte encoding and returned
  `None` for the wide form, whose layout has no captured witness. A real
  translation unit declares tens of thousands of symbols, so **essentially every
  workload function in this row has wide tokens**: census fell from 661,245 back
  to 655,274. The order is knowable without decoding either form, and
  `alloc_rank` is that: a parameter-position rule with `this` ranked last.

### 2. The spine is a register-register family the port had never emitted

`docs/CMP_PRODUCES_A_VALUE.md` reading 4: where the comparison *leaf* forms its
difference as `addi r11,a,-k`, this one is `subf r11,<lhs>,<rhs>` over two
registers. The two words after the difference are the **same** `== 0` fold, with
the same `/O1` temp collapse (`cntlzw r11,r11` against `cntlzw r10,r11`), so they
are one locator now — `codegen::leaf::compare::eq_zero_of_difference_in_r11`,
two consumers — rather than two copies that would have diverged on the mode axis.

### 3. The `bool` result is NOT a different spine *here*

Reading 1 of that document warns that a `bool` result changes the bytes, and it
does — for signed `>=`/`<=` against a non-zero literal, two of 24 cells. `==` is
not one of them: `int`, `unsigned` and `bool` results over the same two calls are
**byte-identical** and differ only in whether the IL carries the `2C <int4> 00`
convert. The annotation is still required to *restate* the value's class, for the
reason `parse_segment_shape`'s own one-byte-unsigned arm gives.

### Refusals, each with its measured cost on the 878-TU workload

| refusal | cost | why |
|---|---:|---|
| `mcall-cmp-rel` — any relation but `==` | **760** (692 `>`, 68 `<`; `!=` is **0**) | the four order relations are the five-word sign-sum spines, two of whose cells move with a `bool` result. A spine borrowed from `compare_leaf_text` is two words short there, with `.pdata FuncLen` and both `$M` wrong to match. |
| `mcall-cmp-args` — an explicit argument on either call | **0** | the marshalling interleaves with the callee-saved move and which is hoisted is what `plan_saved_gprs` refuses to guess (11 of 17 probes wrong for the model that assumed it) |
| `call-arg-nonformal` — a receiver that is not a formal | **0** | the emission is a register move; a global is a load |
| `framed-arg-over-eight-formals` | **0** | past the eighth a formal is stack-homed |
| `callseq-three-plus-saved` | **structurally unreachable** | this production is exactly two calls, so its saved count is exactly 2 by construction. `plan_saved_gprs` gained an `extra_saved` argument so the gate sees the **total** — the saved formals plus the tail's saved call result — and a body needing three registers refuses whatever the third one holds. Stated rather than fixtured: a row that never reaches the gate it claims to test is worse than none. |

`mcall-cmp-rel` is the only non-zero one, and it is the ranked next rung below.

## Estimate vs outcome

`work/WCB/ESTIMATE.md`, written before any scan, with the pre-filter named.

| | estimate | outcome | bias |
|---|---|---|---|
| **E1** the largest *reachable* Class B sub-shape | **6,800**, range 1,500–25,000 | **6,001** | inside, 1.13× high |

The one function of the 6,001 that did not convert is the row's single `calls-1`
member — `bool f(const U* p, int k){ return p->m() == k; }`, where the value live
across the call is a *formal* rather than the first call's result. It is
`CMP_PRODUCES_A_VALUE.md`'s 66-shape, it needs a literal/formal right-hand side
rather than a second call, and it is worth 1.

| **E2** realized census delta | **+2,000**, range 0–7,000 | **+6,000** | inside, **3× LOW** on the midpoint |

**The first estimate in eight to land inside its range, and it landed for a
stated reason.** The estimate named its pre-filter (`-whole` × the frame class ×
the relational grant), refused to multiply the 30,497 ceiling by anything, and
refused to borrow W41's 2.62× realization ratio across populations — the move
that cost W36 2.99× and the relational measurement two orders of magnitude. What
it got wrong was the *risk* discount: E2's midpoint was set below the row because
the shape needs five independent things at once, and four of the five turned out
to be free (the Class B frame, the register allocation, the hoist/trail
interleave and the label stride were all already built and had no reachable
caller). Only the spine and the call order were new.

The correction for the ranking table: **when a row's blocker is a frame class
whose emitter already exists, the ceiling is the estimate.** That is the same
rule ROADMAP §6n records for a counterfactual of the production being widened
(1.0002×), reached from the other direction — and it is what makes the remaining
Class B rows worth ranking on their `-whole` counts directly.

### Which of §6n's five categories this row was

**(1), a private limit inside a recognizer that already exists** — twice over.
`try_parse_member_tail_call` decoded the whole first call and then required the
body to *end* there; the second call is its next byte. And `FrameLayout`,
`call_seq_parts` and `plan_saved_gprs` had modelled Class B since the Class A
call-sequence rung and had **2 reachable callers on the workload**. The row was
not mis-described (`docs/CMP_PRODUCES_A_VALUE.md` had already corrected that) and
not unmeasurable; it was a recognizer's own boundary and an emitter with no way in.

## What this refuted

* **`work/WCB/ESTIMATE.md`'s prediction 3, the one it flagged as most likely
  wrong, is refuted.** "A member statement-call sequence row exists and is larger
  than the 6,760 comparator row": `void f(V* p){ p->a(); p->b(); }` and
  `p->m() + q->m()` both land in `expr-call-in-expr-recv-load-whole`, whose
  `calls-2plus` share is **31 of 6,495**. Two hundred times smaller, not larger.
* **`calls-2plus` does not imply Class B**, and the counter-example is the
  **largest `-whole` row on the board**. `expr-call-in-expr-chained-whole` is
  **12,479 functions, 100 % `calls-2plus`** — and `int f(S* p){ return
  p->a()->b(); }` is a **three-word prologue with nothing saved**: 36 bytes,
  `bl`, `bl`, epilogue. Class A. The frame-class column was already known to
  *under*-count Class B (`CMP_PRODUCES_A_VALUE.md`'s 66); this is the first
  measurement showing it also **over**-counts, and the over-count is bigger than
  the whole comparison family. Read off the obj, not inferred.
* **Prediction 2 held**: Class B reachable before this rung was 2 of 30,497.

## Gate evidence

Corpus `dc3-decomp` at `05ca6d09` (dirty, as recorded in every provenance line).
Baseline `3a1bcf9`.

| lane | baseline | this tree |
|---|---|---|
| `cargo test --workspace --release` | 464 / 0 | **470 / 0** (six new) |
| `c2rs bench` | 176 / 0 / 0 | **178 / 0 / 0** (two new fixtures) |
| `scripts/mode_lane.sh` `/Ox` · `/O1` · `/O2` · `/Ox /Gy` | 83 · 81 · 81 · 81, 0 mismatch | **84 · 82 · 82 · 82, 0 mismatch** |
| `scripts/expr_sweep.sh` | 11,636 cases, 0 mismatches | **12,198 cases, 0 mismatches** (562 new) |
| `scripts/cross_sweep.sh` | 11,761 × 4, 0 mismatches | **14,184 × 4, 0 mismatches** |
| 878-TU scan | 655,245 / 2,462,571 (26.61 %), mismatch 0, disagreement 0 | **661,245 / 2,462,571 (26.85 %), mismatch 0, disagreement 0** |
| `--validate-cache 50 --replay-every 25` | — | **0 cache hits / 878 misses — a full COLD re-capture of all 878 TUs from the real toolchain, same numbers; replay soundness 36 checked, 0 diverged, 0 POISONED** |
| fixtures, `c2rs census` | | positive **19/19**, negative **0/23** |

**The sweep fragment separates the rules it claims to, checked in both
directions** rather than asserted. Reverting `alloc_rank`'s `this`-is-last arm to
a plain parameter index gives **272 mismatches**; ordering by raw token values
instead gives **2**, the two straddling alignments per 256. A first attempt at
the wrap axis used 28 cases and graded **green** against the deliberately wrong
rule — an axis that does not separate is worth nothing, and the only way to know
is to try it.

## The locator check, both directions

Every seam this rung touched, and what asks it:

| locator | consumers | note |
|---|---|---|
| `shapes::calls::plan_saved_gprs` | 2 (the statement sequence, this rung) | **signature changed**: `extra_saved: usize` inserted before `p`. Leaf rungs import from this file — see the merge note. |
| `shapes::mcall_tail::eat_receiver_this` | 2 (was 1) | made `pub(crate)`; the `volatile` gate lives in its operand-type read and nowhere else, which is the reason it is imported rather than re-spelled |
| `codegen::leaf::compare::eq_zero_of_difference_in_r11` | 2 (the comparison leaf, this rung) | **new**, extracted rather than copied: the two words and the `/O1` temp collapse are one fact and the mode axis is the field that would have diverged |
| `SeqTail` | 4 match sites, all exhaustive | the emitter, `bundle.rs`, the census, this rung |

**A shared locator nobody asks was created and then deleted.** The first repair
added `readers::token_alloc_key`, a general "allocation order of a token" helper.
When the rule became a parameter-position one it had zero consumers, and a
locator with no caller is the defect W38 found in the other direction — so it is
gone, and what it knew is in `alloc_rank`'s doc comment where the one consumer is.

**A private copy that refuses more than its siblings: looked for, found latent.**
`chain::leaves_ascending` orders *its* operands by parameter index and has five
consumers, so it carries exactly the `this` defect `alloc_rank` had to avoid —
`params[0]` is the implicit receiver and c1xx numbers it last. It is **not live**:
`this` can only enter a commutative chain as a pointer operand and the
pointer-arithmetic guard bars that. Measured rather than argued —
`int S::f(int a,int b) const { return b + a; }` and its `a + b` twin both census
in class and both are `Port=Match`, with `this` never a leaf. It becomes live the
moment a rung admits pointer arithmetic over a member function's formals.

## Found and not taken

Ranked, frame axis applied first, and the frame axis now means *the obj's
prologue* rather than the call count.

1. **`expr-call-in-expr-chained-whole` — 12,479 functions, and it is CLASS A.**
   `p->a()->b()`: three-word prologue, two back-to-back `bl`s, epilogue, nothing
   saved — which is exactly `call_seq_text` with two empty setups and an empty
   tail. **The largest `-whole` row on the board and the emitter for it already
   ships**; what is missing is the receiver production
   (`mcall::eat_chained_call` has a completeness matcher and no recognizer). Its
   `-then-*` siblings add 2,283 more (`chained-then-type-int1-and-type-aggregate`
   ×2 and ×3). Cheapest large rung on the board by a wide margin.
2. **`mcall-cmp-rel` — 760, this rung's own residue**, and the cheapest way to
   buy it is `>` alone (692). It needs the five-word sign-sum spine and, per
   `CMP_PRODUCES_A_VALUE.md` reading 1, a `bool`-result variant in two of its
   cells plus a `/Ox`-only schedule change — but `scripts/gt_cmp_spine.py` prints
   the whole 48-cell table in one command and the label surcharge is already
   measured (+2, taken ahead of the function's own `$M` triple, which
   `plan_labels` still has no per-function leading count for).
3. **`expr-call-in-expr-recv-object-then-call-nested-call-and-call-whole2/3/4` —
   8,654 together, all `calls-2plus`, frame class still unmeasured.** No probe
   here reproduced that exact key, but its **neighbours are Class A**: a
   `recv-object` receiver is a global, so it is re-materialized from its address
   at each use and never needs saving — `int o1(){ return g_u.h(g_v.m()); }` is
   56 bytes with a three-word prologue and nothing saved, and so is its `void`
   twin and its `p->m()`-argument variant. That lowers the prior on this row
   being Class B considerably, and it is one capture away from being known.
   **Read the prologue before ranking it either way** — that is the whole lesson
   of #1 above, where the biggest row on the board turned out to be Class A.
4. **`expr-call-in-expr-recv-load-then-call-recv-load-whole` — 3,197, all
   `calls-2plus`, and it is `p->h(q->m())`** — a member call whose **argument**
   is another member call. Not what its name suggests and not what this rung's
   author guessed (`a->m() + b->m()`, which is `recv-load-whole` and has 31
   `calls-2plus` in total); probed and read off the obj rather than inferred:

   ```text
     int n5(const W* p, const U* q) { return p->h(q->m()); }   60 B, F = 96
       std r31,-16(r1) ; stwu r1,-96(r1)
       mr r31,r3 ; mr r3,r4 ; bl ?m ; mr r4,r3 ; mr r3,r31 ; bl ?h
   ```

   **Class B with ONE saved GPR** — the outer receiver survives the inner call
   and the inner result goes straight to r4. It is this rung's shape with the
   spine replaced by a single `mr r4,r3`, so `call_seq_parts` needs the inner
   result routed to an argument register rather than to a `subf`, and nothing
   else changes. The cheapest Class B rung left.
5. **The riskiest thing left unmeasured.** Every one of the 6,000 realized
   functions lives in a TU that is `vocab-gap` — the port never emits their objs
   — so their byte evidence is 562 generated cases and 19 fixture functions, and
   nothing else. Two asymmetries inside that:
   * **`this` beside ONE declared formal is 288 cells; beside two or more it is
     8.** The rule `alloc_rank` states is "`this` ranks after *every* declared
     formal", and the plural is graded by eight cases. Everything else about the
     rule is over-covered and that clause is not.
   * **Whether `this` is numbered last is measured for ordinary member functions
     only.** A `static` member, a member of a template instantiation, a member
     with default arguments and a virtual override are all unmeasured, and each
     is a plausible place for c1xx to number differently. The wide-token axis
     (section 13) closed the analogous asymmetry — the whole realized population
     had four-byte tokens and the whole graded corpus had two-byte ones — and
     that one only got closed because somebody counted the two populations.
