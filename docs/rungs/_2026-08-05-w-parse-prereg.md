# w-parse — PREREGISTRATION

    Tag:       w-parse-prereg
    Slug:      w-parse-prereg
    Date:      2026-08-05
    Fixtures:  none — this is a preregistration. It admits no shape, moves no
               accept/refuse boundary and emits no obj byte. Any fixture this
               lane ships is recorded in the findings doc.
    Census:    unmeasured at this commit — a prereg changes no `crates/` file.
    Record:    this file, committed BEFORE any grid of this lane exists.
    Lane:      w-parse, worktree `wt-w-parse` off master **`51176eb`**.
    Ships:     this file.

---

## 0. What I was sent to do, and what I will actually test

Convert `src/xdk/nuispeech/xboxheap.cpp` — the FRONTIER's only branch-free TU —
and resolve board **#564** if the work reaches it.

The brief states the TU is blocked on **three parse facts and nothing else**.
I do not believe that clause and I am registering the disbelief **before**
measuring, because it is the load-bearing claim of the assignment: `ORDER` and
`SCHED` and `ALLOC` are all published as **refusing** `xboxheap` (it is out of
`order.rs`'s domain by the multi-symbol clause), so at least one *ordering*
fact must also be resolved before any emitter can produce its bytes. A parse
that admits a shape the emitter then orders wrongly is a **wrong obj**, which
is the one outcome the correctness rule forbids.

## 1. The exhaustive negative, run FIRST

`work/w-order2/grid.py` regenerates 822 cells in 0.36 s. Before building any
new grid I will re-score the **existing** 822 against two rival rank
definitions:

* **R_count** — `ORDER` as shipped: rank = (use count **descending**, first-use
  source index ascending).
* **R_fu** — rank = first-use source index ascending, **and nothing else**.

`R_fu` is the rule that (by hand, before this file was written) reproduces
`xboxheap`'s entire emitted instruction order — both the store order *and* the
producer emission order — where `R_count` gets both wrong. So `R_fu` is not a
free parameter fitted to a residual; it is the alternative the target names.

**R1.** `R_fu` is **REFUTED** on the existing single-symbol grid: it scores
strictly worse than `R_count`, and its misses concentrate on cells where a
producer of **lower use count** is first used **earlier**. Point estimate:
`R_fu` misses **between 20 and 200** of the 822.

*(If R1 fails — if `R_fu` also scores 822/822 — then `ORDER`'s rank has a free
parameter its own grid never separated, `#564` dissolves, and the finding is
much larger than the TU. I register both outcomes as informative and will
report whichever happens.)*

## 2. The axis of #564, and the artifact I expect to find

`ORDER.md` §5 and `order.rs` name the axis as **the number of base symbols**:
eight single-symbol cells of `xboxheap`'s statement shape emit the **rank-0**
producer first; `xboxheap`, with two symbols, emits the **count-1** producer
first.

`work/w-order2/grid.py` builds producer values from `KINDS = {L, A, I, S}` and
**every cell uses ONE kind for every producer of that cell** (tier 5 crosses
the kind but never mixes two kinds inside a cell). `xboxheap` **mixes** them:
its producers are a constant (`li r10,0`) and a base-derived address
(`addi r11,r3,8`). So the eight "controls" differ from `xboxheap` on **two**
named axes, not one, and only one of them was declared.

**R2.** The real axis is the **producer KIND mixture**, not the base-symbol
count. Prediction: a **single-symbol** cell of `xboxheap`'s statement shape
whose two producers are a constant and a base-derived address will emit them in
`xboxheap`'s order (constant first), not in rank order. Confidence: this is the
one I most expect to be wrong, because `w-pair`'s H4/H5 died claiming exactly
this class of axis and the answer turned out to be the symbol after all.

**R3.** The converse control: a **two-symbol** cell whose two producers are
both constants will emit them in **rank** order (count-2 producer first), i.e.
the symbol count alone changes nothing.

R2 and R3 are the two halves of one 2×2, and **the 2×2 is the deliverable**
whichever way it falls: `ORDER.md` has one cell of it filled in.

## 3. The `mr rN,r3` live-range save — `n = 1` today

`STORE_SCHEDULE.md` §5 records `xboxheap`'s `mr r31,r3` between `S2` and `S3`
as a hypothesis at `n = 1`.

**R4.** Its slot is a function of the **store schedule**, not of the source: I
predict it lands immediately after the last store slot that the *unproduced*
head occupies — i.e. at the same place a third producer's slot would be — and
that a grid crossing (number of producers) × (number of unproduced stores) ×
(a call with `this` live across it) will show it moving with the schedule and
not with the source statement index. Point estimate: ≥ 12 cells, and the rule
holds on all of them or I ship nothing.

## 4. The three parse facts, re-derived

**R5.** `w-dclass`/B's F1/F2/F3 reproduce **exactly** against this tree:
F1 a literal-valued store mixed into a store run, F2 a member's address as a
stored value, F3 a call after a store run. Each alone takes a byte-exact base
out of class; the census key is `expr-op-0x27` for all three.

**R6.** F1/F2/F3 are **NOT sufficient** to convert the TU. Naming the
additional facts before measuring them, so the count cannot drift:

1. the producer **rank** (§2) — `order.rs` refuses this run today;
2. the `mr r31,r3` slot (§3);
3. a **post-call `mr r3,r31`** — `framed_call_text`'s post-op vocabulary is
   `addi r3,r3,k` only;
4. a framed body whose **prologue is followed by a store run** — `Selected::
   Framed` emits `prologue · setup · bl · post-op · epilogue` with no
   representation for anything between the prologue and the setup.

Point estimate: **4 additional facts**, so the TU prices at **7**, and board
**#269**'s decline clause (≥ 4 independent refusals) fires unless the ordering
facts resolve into *derived consequences* of rules that already exist.

**R7.** TU match at the end of this lane is in **[9, 10]**. `9` is the honest
floor and I expect it; `10` requires every one of R2/R3/R4 to resolve *and* an
emitter to be built and gated.

## 5. Method commitments

* Every grid this lane builds writes a **holdout** partition decided by the
  generator, by a rule stated **here** (§6), into a file the fitter **raises**
  on opening. The raise is a positive check, not a convention.
* The rule is frozen at a **named commit** before the holdout is scored, and
  the commit is quoted in the findings doc.
* Every widening of a parser ships with a **positive guard on the emitter
  side** that asks the model and **refuses when it disagrees** — board #232 was
  a parser widening that turned a clean refusal into a live wrong emit for 255
  commits, and this lane widens a parser.
* Misses stay on the page.

## 6. The holdout rule, declared before any generator exists

A cell goes to **holdout** iff any of:

1. `md5(cell-id)`'s first hex digit is in `{0,1,2,3,4,5}` — a ~37.5 % random
   partition, decided by the generator and by nothing else;
2. the statement run is **longer than 5** statements;
3. the cell has **three or more distinct producers**;
4. the cell mixes **three or more producer kinds**.

Clauses 2–4 are deliberate *shape* holdouts: they hold out the shapes a rule
fitted on short two-producer runs is most likely to be wrong about, which is
the failure `order.py`'s own §4.1 correction records.

`xboxheap` itself is in **neither** partition — it is the external cell, scored
separately and always reported.

## 7. What would make me stop

If the ordering facts leave a free parameter that my own grid cannot validate
out-of-sample, I stop and report rather than ship. A mechanical holdout against
a frozen commit is out-of-sample validation and does not need the one-shot
gate; a rule that only fits does.
