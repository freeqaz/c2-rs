# WDA — the data-symbol row DECLINED, and the count the census was throwing away

    Tag:       WDA
    Slug:      dataaddr-2sym
    Date:      2026-07-31
    Fixtures:  wda_neighbours.cpp wda_dataaddr_neg.cpp
    Census:    691,744 unchanged (28.09 %) — the rung is DECLINED; it lowers nothing
    Record:    this file; docs/IL_CALL_IN_EXPR.md §17 is the characterization it rests on

`expr-call-in-expr-data-addr-2sym-then-plain-call-and-type-ptr-whole2` — **18,926
functions, 813 TUs**, the largest key on the board carrying a whole-body
completeness bit, 0.0 % of it behind the EH boundary — was dispatched as the
rung. It is **declined**, and the decline is the cheap half of the result.

The expensive half is a defect found on the way: a one-token predicate in
`mark_whole` was **measuring** a number and then discarding it, so the *second*
largest completeness-complete construct set in the whole bucket
(`recv-load-then-call-data-addr`, 10,540 functions over 828 TUs; 10,558 counting
its smaller tails) could not be told apart from a phase. The count is now in the
key, and it says the **opposite** of what the row it was copied from says.

## What it admits, and what it refuses

**Nothing changed in acceptance.** No arm was added to any `BodyShape`, no
`Blocker` gained a production, `PortC2` refuses exactly the bodies it refused
before, and the census numerator is identical to the function. This is a
census-key rung, in the sense `docs/rungs/2026-07-31-cflow-decode.md` is.

One predicate moved, in `crates/c2-il/src/func/body/mcall.rs`:

```rust
// before (D5)
let counts_syms = matches!(form, CallForm::DataAddr | CallForm::DataRead);
// after
let counts_syms = form_counts_syms
    || matches!(first, Blocker::Call(CallForm::DataAddr) | Blocker::Call(CallForm::DataRead));
```

`Fail::syms` counts a data designator wherever `eat_data_designator` succeeds
inside an open call-argument region. `eat_one_blocker_value` routes a granted
`Blocker::Call(DataAddr)` **straight back into `eat_form_value`**, so the count
was already being accumulated for bodies whose symbols arrive as the second
blocker — and then D5's predicate, which asks only about the body's own
`CallForm`, threw it on the floor. `feature` now renders the suffix next to the
construct that owns the operands: on the form where the form is a designator
(unchanged, bit for bit), on the **blocker** where it is not.

Two things make that refusal the exact shape `docs/ROADMAP.md` §6n step 1 warns
about. It **emits nothing**, so no byte compare can see it. And it **agrees with
census by construction**, so no census/gate disagreement check can either. The
only instrument that finds it is reading the recognizer against its siblings.

## Estimate vs outcome

Written in full **before any code change and before any scan**, in
`work/DA2/ESTIMATE.md` (gitignored scratch; reproduced here).

### (a) The census delta of what would ship — **0, stated as exact, not biased**

> The only work available inside my seam is decode/census. Lowering a
> data-symbol address needs `crates/c2-core/src/coff.rs` … A decode-only change
> cannot move acceptance, so the delta is 0 by construction and any nonzero delta
> would be a bug, not a gain.

**Realized: 0.** Census 691,744 / 2,462,571 = 28.09 % before and after, to the
function; blocked total 1,770,827 before and after; **10,558 functions out of
five keys and 10,558 into ten, with no other key moving by one.**

### (b) Which of §6n's six the row is — predicted **#6**, realized **#6**

> Predicted: #6, "declared/sized/named by the previous rung", with #3 as runner-up.

Correct. D5 (`docs/IL_CALL_IN_EXPR.md` §17) took this row a session ago,
characterized it by capture, shipped **no lowering deliberately**, and §17.6 (6)
files it as *"a PHASE, not a rung"*. The `-2sym` in the key I was handed **is
D5's answer**, printed in the key so the row could not be mis-ranked from its
size again. It was nonetheless dispatched from its size again, which is worth
recording: a warning in a key's name is only read if the ranking reads keys.

The sizing was reproduced from hand-written source before any of it was
believed, per the standing rule — all four plain-call keys land exactly, and
`neg_two_sym_assert` (`a1(p,"expr",42,"file")`) reproduces §17.1's shape 1, the
`MILO_ASSERT` family that is 1,211 of the 2,730 in D5's argument-shape walk. The
row **is** what its name says. It is not #5.

### (c) The measurement — predicted, and **WRONG in direction**

> Prediction: the 10,540 row is majority 1sym. Point estimate 1sym ≥ 70 %
> (≥ 7,400 of 10,540). **Bias direction: HIGH** — i.e. I expect the realized 1sym
> share to come in *below* 70 %.

**Realized: 99.87 % (10,544 of 10,558).** The stated bias direction was
**wrong**, and the magnitude was **1.43× low** (99.87 / 70; on counts
10,538 / 7,400 = 1.42× on the single key).

The reasoning that produced the estimate was right and the hedge was wrong. The
two grounds — that §17.1's four two-symbol shapes are all *free* functions, and
that the natural member-call shape is `p->SetName("foo")` — were both correct and
sufficient. The hedge, "dc3's logging/assert macros plausibly have a member-call
flavour I cannot see from here", is a guess about the corpus imported to justify
a discount, and it is the same move this series has now been wrong about every
time it has been made. **The estimate should have been the ceiling neat.**

## The measurement

The newly-visible population, and the row it was assumed to resemble:

| population | 1sym | 2sym+ | 1sym share |
|---|---:|---:|---:|
| **form-owned** (`data-addr-…-then-plain-call…`, D5's) | 2,719 | 18,936 | **12.6 %** |
| **blocker-owned** (`…-then-call-data-addr…`, WDA's) | **10,544** | 14 | **99.87 %** |

They are near-perfect mirror images, and nothing in D5 predicts that. The
member-call population is *essentially entirely single-symbol*: 14 functions in
878 TUs pass two.

The exact partition, which is the acceptance check for a re-key:

```
  -10540  expr-call-in-expr-recv-load-then-call-data-addr-whole
  +10538  expr-call-in-expr-recv-load-then-call-data-addr-1sym-whole
      -9  …-and-off-add-whole4            ->  +9  …-2sym-and-off-add-whole4
      -2  …-recv-object-then-call-data-addr-whole  ->  +2  …-1sym-whole
      -2  …-and-type-int1-whole2          ->  +1 1sym  +1 2sym
      -2  …-and-type-real-whole3          ->  +1 1sym  +1 2sym
      -2  …-and-type-ptr-whole2           ->  +2 1sym
      -1  …-and-op-whole3                 ->  +1 2sym
  ----------------------------------------------------------------
  10,558 out, 10,558 in, net 0; every other key delta exactly 0
```

### What the `-whole` / `-whole2` / `-whole3` / `-whole4` suffix counts

Asked mid-rung, and it decides whether "one rung or three" is even the right
question. It is `need` = **the number of DISTINCT extra constructs granted past
the form** before the body parsed to its end. Not statements, not calls, not
symbols. Two controlled pairs settle it, both one source token apart:

```text
  x = uc("hi")       data-addr-1sym-then-plain-call-whole                 need 1
  x = u3(p, "cc")    data-addr-1sym-then-plain-call-and-type-ptr-whole2   need 2
```

Adding one **pointer formal** — no new statement, no new call, no new symbol —
moves the suffix. And the reverse control:

```text
  x = uc("hi")       data-addr-1sym-then-plain-call-whole                 need 1
  d1("aa","bb")      data-addr-2sym-then-plain-call-whole                 need 1
```

A whole extra string argument does **not** move it, because two designators are
one construct. That is precisely why the count needed its own bits, and it is
`the_whole_suffix_counts_granted_constructs_not_occurrences` in the test module.

It follows that the **unit of work is `{form} ∪ granted`, not the receiver
form.** Over the `-whole…` family — 99 keys, 71,767 functions — there are **93
distinct construct sets**, and `recv-load` alone spans **47** of them for 27,202
functions. Grouping that form into one rung would be grouping 47 different pieces
of work.

### The one number this rung would put at the top of the board

Ranked by construct set rather than by key, the `-whole…` family's two largest
entries are both data-symbol rows:

```
  20,586  data-addr-then-plain-call-and-type-ptr        (18,926 2sym + 1,660 1sym)
  10,540  recv-load-then-call-data-addr                 (10,538 1sym + 2 2sym)
```

**31,126 functions — 43 % of everything the completeness matcher can finish in
four constructs or fewer — is blocked on materializing a data symbol's address**,
and counterfactual C below finds **8,658 more** where the designator is the third
construct rather than the second. Together that is **39,784 of 71,767 = 55.4 %**
of the whole `-whole…` family.

**More than half of everything the grammar says is within four constructs of
complete has to put a data symbol's address in a register**, and the code that
would do it lives in `crates/c2-core/src/coff.rs`, not in the IL layer at all.
That is the ranking result of this rung, and it is not visible from any single
key: the largest one is 18,926 and it is the *unbuildable* fraction.

## One rung, or three? — the answer

Four rows were named. They are **two rungs and a phase**, and the boundary is not
where the row names put it.

| row | count | verdict |
|---|---:|---:|
| `data-addr-1sym-then-plain-call-whole` | 1,058 | rung A |
| `data-addr-1sym-then-plain-call-and-type-ptr-whole2` | 1,660 | rung A |
| `recv-load-then-call-data-addr-1sym-whole` (+ 6 tails) | 10,544 | rung B |
| `data-addr-2sym-…` (+ `recv-load-…-2sym`) | 18,950 | **phase** |

* **By grammar the three `data-addr` rows are ONE thing.** They share the
  designator production and differ only in a grant count and a symbol count,
  neither of which is a grammar distinction.
* **By codegen the split is 1sym / 2sym, and it cuts straight through them.**
  §17.3 (a): two symbols in one call are not two relocation pairs; c2 emits one
  `lis`/`addi` and derives the rest as `addi rD, rAnchor, <difference of .rdata
  pool offsets>`. Instruction selection then depends on a whole-TU pool layout the
  port's per-function `select_text` cannot see.
* **The 10,544 are a separate rung, not part of either** — they need the
  **member-call** emitter, not the plain-call one, and that emitter does not exist
  either (`expr-call-in-expr-recv-load-whole` is still 6,495 blocked functions).
  They sit behind **two** unbuilt things where rung A sits behind one.

So: **not one rung, not three. Two rungs and a phase**, and the largest of the
two rungs (10,544) was invisible until this session because its defining number
was being computed and discarded.

### Why the phase is a phase, counted rather than discounted

The estimate rule says: when the blocker is a class whose **emitter already
exists**, take the ceiling neat and count the independent refusals between
ceiling and emitter. The emitter here does **not** exist, so the refusals are
counted instead. For rung A (2,719):

| # | piece | state |
|---|---|---|
| 1 | REFHI/REFLO+PAIR quad on a data symbol | **shape exists** — byte-identical to the pooled-FP-constant relocation `coff.rs` already emits (§17.2 (1)) |
| 2 | the extern-linkage gate | **exists** — `gl_defined_names` already refuses every defined/static-global TU (§17.2 (7)) |
| 3 | the `/Ox` `.rdata` string pool + section-table insertion before `.text` | **does not exist** — `coff.rs` |
| 4 | `$SG<n>` STATIC symbols (`/Ox`) *or* `??_C@` COMDAT sections read from `.gl` (`/O1`, the workload's own profile) | **does not exist**, and these are **two different emitters** (§17.2 (4)) |
| 5 | `lis`/`addi` selection with no other argument setup | **does not exist** |
| 6 | the acceptance arm in `c2-il` | does not exist |

Three unbuilt pieces, two of them in `coff.rs`. Rung B adds the whole member-call
emitter on top. The phase adds four more, and **two of those have no derived rule
at all** — they are not unimplemented, they are unknown:

* §17.3 (b), which symbol is the anchor: *"a HYPOTHESIS with no mechanism behind
  it"*, fitted to 14 witnesses including six byte-identical functions in one TU
  that split two ways on nothing but a pool offset.
* §17.3 (c), the argument scheduler: five witnesses, *"a 'descending slot' rule
  fits four of the five and a 'gap of exactly one instruction' rule fits eleven of
  twelve. Neither is a rule."*

A rung cannot be estimated across a construct for which no rule exists. That is
§17.4's correction — *an estimate made from a grammar measure cannot see a codegen
construct that the grammar does not distinguish* — and it is why WDA ships the
measurement that makes the construct visible and stops there.

## The seam boundary, stated

Every remaining piece of all three items is in `crates/c2-core/src/coff.rs`, which
this session was explicitly barred from touching, plus a new `shapes/` arm and its
`mod.rs` dispatch, which is not in this seam either. **The rung is not merely
declined on measurement; it is not takeable from this seam at all.** Recorded here
rather than attempted.

## A second defect, fixed, and NOT part of this rung

Handed over mid-session and landed as its own commit because it is in this seam.
`chain.rs`'s multiply-by-literal gate read `i == 1 && lhs_lit` against a
**postfix** op stream whose first operator sits at index 2 — the `Sub` arm
immediately below it has always used `i == 2`. The clause could never fire, so:

```text
  return 3 * a;   census 1/1 IN CLASS,  port NotImplemented   <- DISAGREEMENT
  return a * 3;   census 0/1 refused,   port NotImplemented   <- agrees
```

Reproduced from source, fixed to `i == 2`, verified. The direction is safe: it
makes the census **stricter**, the port already refused both spellings, and no
byte moves. Its workload cost was **verified, not assumed**: a key-level diff of
the 878-TU scan before and after the fix is **0 on every one of the 722 keys**,
in-class 691,744 both sides — the corpus contains no `return <lit> * a;` that
survives to this class, which is exactly why nothing on the workload lane ever
flagged it. The test table held `[Load, Lit, Mul]` — the exact operand order the
rule was derived from — and never the commutation; both orders are now in it,
plus `a - 3` as the in-class control that keeps the fix from being widened into
"any literal beside any operator".

Method note, since it is the transferable part: this was found by running the
**generated sweep corpus through the `gap` path**. `expr_sweep` drives `c2rs diff`
and greps for `Mismatch`, so the census/gate disagreement invariant — which exists
only on the `gap` path — had never been evaluated on generated cases at all. It
reads 0 on the workload and 0 on fixtures and read **155** there.

## Gate evidence

| lane | result |
|---|---|
| `cargo test --workspace --release` | **516 pass, 0 fail** |
| `c2rs bench` | **195 pass, 0 fail, 0 error** |
| `c2rs selftest` | **195 pass, 0 fail** |
| `scripts/mode_lane.sh` `/Ox` / `/O1` / `/O2` / `/Ox /Gy` | **92 / 90 / 90 / 90 match, mismatch 0** |
| `scripts/expr_sweep.sh` (private outdir) | **checked=13,707, 0 mismatches** |
| `scripts/cross_sweep.sh` | **27,956 configurations × 4 mode lanes = 111,824 gradings, 0 mismatches** |
| 878-TU workload scan | match 6, **mismatch 0**, census **691,744 / 2,462,571 = 28.09 %**, **disagreement 0** |
| fixtures, `c2rs census` | `wda_neighbours.cpp` **6/6**, `wda_dataaddr_neg.cpp` **0/6** |

The fixture pair is the boundary stated as source rather than as a row count:
`nb_one_lit` (`x = uci(7)`) is in class and `neg_one_sym` (`x = uc("hi")`) is
1,058 workload functions, and they differ by exactly one operand token — an int
literal against a data symbol's address. `neg_recv_one_sym` / `neg_recv_two_syms`
are the pair that printed one identical key before this rung.

## Found and not taken

| item | size | what stops it |
|---|---:|---|
| **rung B — `recv-load-then-call-data-addr-1sym`** | **10,544** | the member-call emitter *and* the data-symbol emitter, both unbuilt; `coff.rs`. The largest single-construct-set rung now visible, and it did not exist as a measured quantity before this session |
| **rung A — `data-addr-1sym`** | 2,719 | items 3/4/5 of the table above, all `coff.rs`. §17.6 (3)'s figure of 2,712 is confirmed at 2,719 |
| **the 2sym phase** | 18,950 | §17.3 (a)/(b)/(c)/(d); two of the four have **no rule**, only fitted hypotheses |
| the `-and-<kind>` third construct | 8,654 in one family | still coarse: a nested call renders as `-and-call` with its receiver form dropped, so `recv-object-then-call-nested-call-and-call-whole{2,3,4}` cannot say what its third construct *is*. Counterfactual C shows **all 8,654 materialize a data symbol**, and for the 4,905 at `-whole2` — where there are exactly two grants — the third construct *is* the designator. That is precisely the fact the coarse kind hides, and the reason the count could not be safely rendered there |
| the symbol count on a `-more` body | — | deliberately unset, per §17.5: an abandoned prefix's designator count is a property of the refusal, not of the program |
| **a designator that is neither the form nor the *named* second blocker** | **8,658, of which 7,242 are 1sym** | measured by counterfactual C below — **not** 0, and the first draft of this document asserted 0 without measuring it. Left unrendered deliberately; see below |

### Counterfactual C — the residual, measured rather than asserted

The shipped predicate is keyed on the **named** second blocker being a
designator. A body can also reach one as its *third* construct, which the key
names only by coarse kind (`-and-call`). The first draft of this document
asserted that population was 0. It is not, and asserting it was the same error
this rung exists to correct — so it was measured.

Scratch build, reverted: `counts_syms = true` unconditionally in `mark_whole`,
rescan the 878 TUs, diff against the shipped tree.

```
  -4905  recv-object-then-call-nested-call-and-call-whole2
  +4903    …-call-nested-call-1sym-and-call-whole2      +1 2sym  +1 3sym+
  -2334  recv-object-then-call-nested-call-and-call-whole3
  +2333    …-1sym-and-call-whole3                       +1 2sym
  -1415  recv-object-then-call-nested-call-and-call-whole4
  +1411    …-2sym-and-call-whole4                       +4 1sym
     -3  three singleton keys                           -> 1sym / 2sym / 3sym+
  ---------------------------------------------------------------------
  8,658 out, 8,658 in, net 0; census 691,744 unchanged; disagreement 0

    1sym  7,242      2sym  1,414      3sym+  2
```

**A further 7,242 single-symbol functions are visible to the count and not
rendered.** They were **not** shipped, and the reason is that the honest
rendering position does not exist yet. The suffix currently sits on the form or
on the named second blocker — both constructs the key *names*. Here the
designator is in the third construct, which the key names only as `-and-call`,
so `recv-object-then-call-nested-call-1sym-and-call-whole2` would attach "one
symbol" to a nested call that may materialize none. That is exactly the
by-position mis-attribution `GAPS.md` §6 records and this module is built
against, and trading one discarded measurement for one misattributed one is not
a fix.

It is also **not the same rung**: those bodies need the member-call emitter, the
nested-call emitter *and* the data-symbol emitter — three unbuilt things where
rung A needs one. Rendering them would grow the 1sym pool on paper from 13,263 to
20,505 without moving a single one of them closer to being emitted.

The fix, when someone takes it, is a body-level spelling that cannot be read as
belonging to any named construct (the count has been per-body since §17.5, never
per-call), not a third positional convention. Recorded, sized, and left.

### The riskiest thing left unmeasured

**Whether the 99.87 % single-symbol share survives contact with the member-call
emitter.** Every number above is a *grammar* measure, and §17.4's correction is
precisely that a grammar measure cannot see a codegen construct the grammar does
not distinguish. The symbol count is now in the key because D5 paid for that
lesson; there is no reason to believe it is the *last* such construct hiding
inside `recv-load-then-call-data-addr`. What §17.3 (c) found for plain calls — that
argument setup is **scheduled**, in an order no rule fits — has never been looked
for on a member call, where `this` occupies r3 and every symbol address shifts one
slot. Rung B should re-run §17.3 (c)'s probe set with a receiver before it is
sized, not after.
