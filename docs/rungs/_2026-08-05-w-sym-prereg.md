# w-sym — PREREGISTRATION

    Tag:       w-sym-prereg
    Slug:      w-sym-prereg
    Date:      2026-08-05
    Fixtures:  none — this is a preregistration. It admits no shape, moves no
               accept/refuse boundary and emits no obj byte. Any fixture this
               lane ships is recorded in the findings doc.
    Census:    unmeasured at this commit — a prereg changes no `crates/` file.
    Record:    this file, committed BEFORE any grid of this lane exists.
    Lane:      w-sym, worktree `wt-w-sym` off master **`11bd0df`**.
    Ships:     this file.

---

## 0. What I was sent to do

Close board **#582** — the **producer emission order of a store run through
more than one base symbol** — and, if it closes, replace
`crates/c2-core/src/codegen/order.rs`'s refusal of a multi-symbol run with a
measured model.

The two candidate rules already on the page are both refuted, and they are
refuted *by the shape the item is about*, in both directions:

| | rank order (`ORDER`'s, #561) | first-consumer in the final store order (`w-alloc`'s) |
|---|---:|---:|
| single-symbol | **857 / 857** | 822 / 857 |
| **multi-symbol** | 4459 / 5053 | 4860 / 5053 |

So the answer is neither, and it must **reduce to rank order on one symbol**
(that column is exact and is not up for renegotiation).

## 1. Definitions, fixed here so no clause can drift later

A store run is statements `S_0 … S_{n-1}`. `sym(k)` is the base **symbol** of
`S_k` (a bound reference is its own symbol — board **#580**), `prod(k)` is its
producer or `None`.

* `c(p)` — use count, the number of statements whose producer is `p`.
* `fu(p)` — first-use source index.
* **`R`** — the global rank order: producers sorted by `(c desc, fu asc)`.
  This is `ORDER`'s rank (#561).
* **group** `G_s` — the sub-sequence of statements with `sym = s`.
* **`grank(p, s)`** — the position of `p` in `R` **restricted to the producers
  that appear in group `s`**.
* **`grank(p)`** — `min over s in symbols(p) of grank(p, s)`. A producer that
  feeds stores through two symbols therefore takes the **better** of its two
  group ranks. **This is the clause I most expect to be wrong** and §6 clause 6
  does not hide it: spanning cells stay in FIT so it can be measured.

## 2. The central hypothesis, stated before any cell of this lane exists

> **SYMRANK.** Producers are emitted in the order **`(grank asc, fu asc)`**.

On a single-symbol run every producer's group is the whole run, so
`grank = R`-position and SYMRANK **is** rank order — the 857/857 column is
preserved *by construction*, not by fitting. On `xboxheap` the two groups are
`h = {S0,S1,S2,S3}` and `l = {S4,S5}`, each contributing exactly one producer,
so both producers have `grank = 0` and the `fu` tiebreak emits the count-1
producer `li r10,0` **first** — which is what `c2` does and what rank order
gets wrong.

**R1.** SYMRANK is exact on **≥ 99 %** of in-domain FIT cells conditional on
the observed store order, and **857/857** on the existing single-symbol
population. If it lands below 99 % I will report the number and ship nothing.

## 3. The exhaustive negative, run FIRST

Before SYMRANK is scored I enumerate the class every "sort the producers on
their own features" answer lives in, on **FIT only**:

```
key = a lexicographic tuple of up to 3 signed features drawn from
      { grank(p) (min over groups),   R-position of p,
        c(p),                         fu(p),
        first-consumption index in the OBSERVED store order,
        index of p's first group in source symbol order,
        number of distinct symbols p feeds,
        last-use source index,
        max grank over p's groups,
        c(p) counted within p's first group }
      = 10 features x 2 signs, up to 3 deep  =  8,000 configurations
```

SYMRANK is `(+grank, +fu)` and is a member; `ORDER`'s rank is
`(-c, +fu)` and is a member; `w-alloc`'s first-consumer rule is
`(+first-consumption)` and is a member. **The point of running it first is not
to find a winner but to see the ceiling and the residual**: three consecutive
lanes found the answer *outside* the class they searched and every time the
residual's shape named the mechanism.

**R2.** No configuration in the 8,000 beats SYMRANK on FIT. Point estimate:
the best non-SYMRANK configuration is at least 3 points below it. *(If
something beats SYMRANK, it is a **fitted** rule and not a preregistered one,
and I will label it that way and let the holdout decide.)*

The producer order is scored **conditional on the observed store order**, so
that a wrong store order cannot contaminate the number #582 asks for. The full
emitted sequence is scored separately in §4.

## 4. The store order — the other half, and it is NOT #582

`w-parse`'s SYMORDER reaches 91.9 % on multi-symbol cells. Its docstring says
the floor is `u_g + j` *counted within the store's own group*; **its code uses
the global `u` and the global position** (`work/w-parse/model.py`, `u_all` and
`q = len(out)`). So the per-symbol reading was never measured. Two rivals:

* **SO-A** — SYMORDER exactly as `w-parse` shipped it (global `u`, global
  position, group rank, cross-symbol pin).
* **SO-B / PER-SYMBOL** — each group is scheduled **independently** by `ORDER`
  (its own `u_s = min(2, unproduced in group s)`, its own positions, ranks
  taken from `R` restricted to the group), and the groups are merged so that
  the emitted **symbol pattern equals the source symbol pattern**.
* **SO-C** — SO-A with `u` taken per-group but positions global.

**R3.** SO-B beats SO-A on multi-symbol FIT cells. Point estimate: **≥ 96 %**
for SO-B against SYMORDER's 91.9 %.

**R4 (model-free, no rule involved).** The emitted symbol pattern equals the
source symbol pattern on **100 %** of cells — the cross-symbol pin of
`STORE_SCHEDULE.md` §3 is exact, not approximate. Any exception refutes a
landed doc and is the most valuable thing this lane could find.

## 5. What else this grid is built to be able to falsify

Every lane that shipped a wrong claim here shipped it from a population that
**could not contain its own counterexample** (#581; `w-sched`'s 184 conflicted
cells). The declared axes:

**R5 — arity.** Runs through **three** base symbols are covered by SYMRANK and
by the winning store-order rule with **no new constant**. The whole 3-symbol
tier is **holdout** (§6 clause 3), so this is scored out of sample or not at
all. Point estimate: within 2 points of the 2-symbol rate.

**R6 — spanning producers.** A producer feeding stores through two symbols
takes `grank = min` over its groups (§1). Point estimate: exact on ≥ 95 % of
spanning cells. I expect this to be the clause that breaks; `max`, `first
group` and `last group` are the alternatives and all four are in the §3 search.

**R7 — kind mixture (#581).** The *producer emission order* is independent of
the producer-kind mixture even though the *store permutation* is not. The whole
mixed-kind tier is **holdout** (§6 clause 4). Point estimate: SYMRANK within 2
points of its all-constant rate; the store-order rule I expect to lose more.

**R8 — the register pool (#541/#543).** Two lanes declined this because their
probe signatures were capped so they could not measure it. Mine is not: a tier
with signature `(M* p, M* q)` and up to **six** producers leaves `r5 … r11`
free. Prediction: the pool is the free volatile registers **highest first**,
`r11, r10, r9, r8, r7, r6`, **skipping `r12`**, and a register freed by an
already-emitted store re-enters the pool. Point estimate: exact on ≥ 90 % of
the tier. **If it holds I apply it with evidence; if it does not I report the
counterexample and leave #543 open.**

## 6. The holdout rule, declared before any generator exists

A cell goes to **holdout** iff any of:

1. `md5(cell-id)`'s first hex digit is in `{0,1,2,3,4,5}` — a ~37.5 % random
   partition, decided by the generator and by nothing else;
2. the run is **longer than 6** statements;
3. the cell has **three or more base symbols** (R5 — the whole arity tier);
4. the cell **mixes producer kinds** (R7 — the whole #581 population);
5. the cell has **four or more distinct producers**.

Clauses 2–5 are deliberate *shape* holdouts: FIT is two-symbol, all-constant,
≤ 3 producers, ≤ 6 statements, and every structural generalisation the rule
claims is graded outside it.

`xboxheap`'s own word is in **neither** partition — it is the external cell,
scored separately and always reported.

**Mechanics.** The generator writes both files. The fitter and the search
**raise** on any path containing `holdout`, and I will demonstrate the raise
rather than assert it. The rule is frozen at a **named commit** quoted in the
findings doc before the holdout is scored once.

## 7. What I predict about the TU, so it cannot drift

**R9.** TU match at the end of this lane is **9**. `xboxheap.cpp` re-priced to
6 and #582 is only one of its remaining facts: it also needs a post-call
`mr r3,r31`, which `framed_call_text`'s post-op vocabulary (`addi r3,r3,k`)
cannot spell, and a framed body with a store run **between the prologue and the
call**, which `Selected::Framed` has no representation for. **Closing #582 will
not convert it**, and I am saying so before I start so that a match of 9 is a
predicted outcome and not an excuse.

## 8. What I ship, and what would make me stop

* If SYMRANK **and** a store-order rule are exact on FIT and on the holdout in
  a stated domain, I replace `order.rs`'s `single_symbol` refusal with the
  model — as a **positive guard** that asks the model and refuses when it
  disagrees, inert by construction, which is why seventeen refuted rules have
  produced zero wrong objs.
* If SYMRANK holds and the store order does not, I ship **SYMRANK as the
  producer-order half only** and the refusal stays, because a caller needs
  both. #582 would then be closed and `order.rs` unchanged, and I will say that
  plainly.
* If neither holds I ship the negative, the residual's shape, and nothing else.
* If I end with a free parameter my own grid cannot validate out of sample I
  stop and report rather than ship. A mechanical holdout against a frozen
  commit is out-of-sample validation and does not need the one-shot gate.

## 9. Board numbers

This lane starts at **#600** (highest is #586; the free gaps `#303`–`#318`,
`#325`–`#339` and `#563` are left alone). Whatever it takes is listed in the
findings doc and added to `docs/BOARD.md` in the same commit.
