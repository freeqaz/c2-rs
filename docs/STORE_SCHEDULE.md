# STORE_SCHEDULE — where c2 puts a value-producing instruction, and why the stores move

This is the project's single most-refuted axis. Before this document there were
**twelve candidate rules from four lanes**, each killed by the next cell:

| lane | rules | killed by |
|---|---|---|
| `w-pair` §4 | six *placement* rules (R1, R2, R3, H3, H4, H5) | `C0`, `C1`, `C3`, `C7`, `D2`, `D3`, `D5`, `E1`, `E5`, `F2` |
| `crates/c2-il/.../leaf_store.rs` | four *allocation* rules | one cell each |
| `w-dclass`/B | **F4a** (fits 6/6, deliberately not shipped) | `o7`, `xboxheap` |
| `w-dclass`/B | **F4b** — declared UNFITTED | — |
| `w-conv` | *"unpriceable — diverges at instruction 0 on ORDER"* | — |

**All twelve are now derived consequences of two rules and one constant**, and
every killer cell above is reproduced. The implementation is
`crates/c2-core/src/codegen/schedule.rs`; the grid is `work/w-sched/`
(gitignored — the generator scripts are committed, the `.cod`/`.obj` are not).

---

## 1. SCHED

> **Rule 1 — store order.** Walk the source statements in order and emit the
> earliest store that is *allowed*. A store whose value needs a new instruction
> to materialise it — a **produced** store — may not occupy store position 0 or
> 1: it may not be the first or the second store of the run. Stores through
> different base **symbols** may not be reordered past each other. If every
> remaining store is blocked, source order wins.
>
> **Rule 2 — producer placement.** The producers, in source order, are inserted
> immediately *before* the stores at store positions 0, 1, 2, … — one producer
> per store slot, from the top of the block.

That is the whole rule. The only free constant is the **2** in rule 1.

### 1.1 Worked: `xboxheap.cpp`'s constructor, the FRONTIER's branch-free TU

```
mSize     = size;                 S0   formal            base h
mFreeHead = this;                 S1   `this`            base h
mCount    = 0;                    S2   produced  (P0)    base h
mUsedHead = this;                 S3   `this`            base h
auto& listHead = mListHead;
listHead.mNext = &listHead;       S4   produced  (P1)    base l
listHead.mPrev = &listHead;       S5   produced  (P1)    base l
```

Rule 1: `S0` and `S1` are unproduced and take store positions 0 and 1. `S2` is
produced and store position 2 is now allowed, so it takes it; `S3`, `S4`, `S5`
follow. **Store order is source order.** Rule 2: `P0` goes before store position
0, `P1` before store position 1.

```
P0 S0 P1 S1 S2 S3 S4 S5
li r10,0 ; stw r5,16(r3) ; addi r11,r3,8 ; stw r3,0(r3) ; stw r10,20(r3) ;
                                           stw r3,4(r3) ; stw r11,8(r3) ; stw r11,12(r3)
```

Read off the real obj at the workload's own flags, that is exactly what `c2`
emits — with one further instruction, `mr r31,r3`, between `S2` and `S3`. **That
`mr` is the live-range save of `this` across the trailing call, not a store
producer**, and counting it as one is the entire reason `w-dclass`/B recorded
`xboxheap` as *"a third regime at 0, 2, 5"*. Its two store-value producers are
at **0 and 2**, the ordinary two-producer slots. See §5.

### 1.2 Worked: `o7`, the cell F4b declared unfitted

`a=x; b=1; c=2; d=3; e=y` — three distinct producers at statements 1, 2, 3.

Rule 1: positions 0 and 1 may not hold a produced store, so they go to `a` and
`e`; then `b`, `c`, `d`. Store order `a e b c d`. Rule 2: the three producers go
before store positions 0, 1, 2.

```
P1 S0 P2 S4 P3 S1 S2 S3
li r11,1 ; stw a ; li r10,2 ; stw e ; li r9,3 ; stw b ; stw c ; stw d
```

which is `o7` instruction for instruction. F4a predicted producers at 0, 1, 2
and was refuted; the producers are at 0, 2, 4 because **rule 2 interleaves them
with stores**, one apiece.

---

## 2. What SCHED is not — and this is measured, not asserted

**It is not a machine scheduler.** The producer's *kind* is completely
irrelevant to the permutation. `li` (an immediate), `addi` from the store's own
base, `addi` from a different base, `addi` from a formal, `rlwinm`, and
**`mulli` — several cycles slower on this part** — all yield the byte-identical
order at every consumer position. A latency model cannot be indifferent to
`mulli`.

The corroborating negative is preregistered and was run first: a search over
**13,104 list-scheduler configurations** (forward and backward × producer→
consumer latency 1..6 × a lexicographic priority key built from up to three
signed features drawn from {natural index, statement index, is-producer,
is-store, depth, height}, with memory dependences in the DAG) tops out at

```
89 of 146 fit cells   —   and the residual is EXACTLY the two-producer tier, 0 of 48
```

Rule 2 is an *insertion* rule, not a priority function, so no member of that
family can express it. Every one-producer tier is at or near 100 % in the same
search, which is why the residual is worth more than the score.

---

## 3. The axis `w-pair` mistook for a register superstition

`w-pair` §4 ends on `F1`/`F2` — the same statement structure with the two
pointer parameters exchanged, both emitting gap 1 — and concludes that every
surviving rule *"had to be stated in terms of a specific register number, which
is the signature of fitting a machine scheduler"*.

The axis is not the register. It is the **base symbol of the store's address
expression**. Two stores that may alias are never reordered, and every cell that
killed H3/H4/H5 has two symbols in it:

| cell | symbols | order | why |
|---|---|---|---|
| `E2` | `h h h h` | reordered, gap 3 | one symbol — free |
| `E1` | `g h h g` | source order, gap 1 | two symbols — pinned |
| `F1` | `l b b l` | source order, gap 1 | two symbols — pinned |
| `F2` | `l a a l` | source order, gap 1 | two symbols — pinned |
| `D5` | `l h h l` | source order, gap 1 | two symbols through **one machine register** — a bound reference `B& l = h->lh` is its own symbol |

`D5` is the discriminating cell: `r3` is the base register of all four stores
and the order is still pinned, because `l` and `h` are different *symbols* in
the IL and the address folding to `8(r3)` happens later. Tier 6 of the grid
isolates the axis directly — the same cell with the store destinations split
`pppp` / `pqqp` / `ppqq` / `qppq` gives reordered / pinned / pinned-by-one /
pinned, exactly as the symbol count predicts, with the producer's own base held
fixed.

---

## 4. The part SCHED does NOT cover: register allocation

When the allocator hands a producer a register that is still the data source of
a store the schedule has not emitted, the resulting write-after-read
anti-dependence perturbs the order. Over the 504-cell grid:

| distinct producers | allocation clean | allocation conflicts |
|---:|---:|---:|
| 0 | 6 | **0** |
| 1 | 148 | **0** |
| 2 | 109 | 10 |
| 3 | 54 | 67 |
| 4 | 3 | 107 |

* **SCHED is exact on 320 of 320 clean cells and on 0 of 184 conflicted ones.**
  The partition is decided from the register assignment alone, before the order
  is looked at, so it cannot be tuned to make a miss look explained.
* With **0 or 1 producers it never happens** — 154 of 154.
* `xboxheap` has two producers and is clean.

Predicting *which* register the allocator picks is a separate, open problem —
`leaf_store.rs` already records four refuted rules for it and this document adds
no fifth. A caller that cannot show the allocation is clean must refuse.

> ### ✔ SOLVED 2026-08-05 by lane `w-alloc` — see `docs/ALLOC.md`
>
> **The allocation is `docs/ALLOC.md` §1** and all four of `leaf_store.rs`'s
> refuted rules are derived consequences of it: sort the producers by **use
> count descending**, tie to register-derived before constant, tie within the
> register-derived by source order and within the constants by **reverse**
> source order, then hand out r11, r10, r9 … descending. 236/236 on its fit
> partition, 250/257 on a preregistered holdout with **0 wrong**, 6/6 on the
> killer cells.
>
> **Two corrections to the table above and one to §1, all measured:**
>
> 1. **The 184 "conflicted" cells are POOL-PRESSURE cells.** `conflicted()`
>    detects **register reuse** — in `{a=1;b=2;c=3;d=4}` the fourth `li`
>    retakes `r11` after `stw r11` frees it. The table's shape (conflicts
>    starting at two producers, 107 of 110 at four) is partly an **artifact of
>    this grid's own signature**: `(M* p, M* q, unsigned f0..f5)` puts
>    `f4`→`r9` and `f5`→`r10` *inside* the pool, so the pool is eaten from
>    below. **Board #541** — the reuse choice itself is still open.
> 2. **The pool is not `r11/r10/r9`.** It is the free volatile registers taken
>    highest-first, reaching `r8`, then registers freed by an emitted store
>    (`r5`, `r4`, and even `r3`), then `r30`/`r31` with a save/restore.
>    `r12` is never used — **board #543**.
> 3. **Rule 2 needs a scope condition** — see **board #542** and
>    `docs/ALLOC.md` §5. "One producer per store slot" holds only while there
>    are unproduced stores to slot against: with `u = min(2, #unproduced)` head
>    slots, producers fill those one apiece and every **remaining** producer is
>    emitted contiguously before slot `u`. `{a=1;b=2;c=3;}` is `P P P S S S`,
>    not `P S P S P S`. This grid always had ≥3 formals and ≤3 producers, so it
>    never ran out of slots. **`schedule.rs` is deliberately unchanged** — on
>    every shape the parser admits today the refined and shipped rules agree.
>
> What remains open is the store **order** when *every* store of the run is
> produced, a regime this grid never contained — **board #544**.

> ### ✔ SOLVED 2026-08-05 by lane `w-order2` — see `docs/ORDER.md`
>
> **#544 is closed and rule 1 is a special case of its answer.** Rank the
> producers by *(use count descending, first-use ascending)*, let
> `u = min(2, #unproduced)`, and a store whose producer has rank `j` may not
> occupy store position `< u + j`. **Rule 1 is `j = 0` with `u = 2`**, and
> `w-alloc`'s hoist is `j = 0` with `u < 2`. `479/479` on the grid above,
> `248/248` fit and **`561/561` on a preregistered holdout**, `822/822` on the
> store order alone. Rule 1's *"if every remaining store is blocked, source
> order wins"* fallback is **deleted**, not carried: with `u < 2` the floors
> drop with it and nothing is ever fully blocked.
>
> **Correction 3 (rule 2's scope, board #542) is now APPLIED** to
> `schedule.rs` — the interleaving stops when the unproduced stores run out,
> so `{a=1;b=2;c=3;}` is `P P P S S S` and `{a=1;b=2;c=3;d=f;}` is
> `P S P P S S S`. Five measured cells are in the tests. Nothing else consumes
> `schedule()`, so the change is test-visible only.
>
> **Correction 2 (the pool) is NOT applied.** It is an `alloc.rs` fact and no
> cell of `w-order2`'s grid measures the pool — its signature is capped at
> three formals precisely so that it cannot. It stays boards **#541**/**#543**.
>
> **§5's `mr r31,r3` fact is upgraded from n = 1 to a discriminated one.**
> `xboxheap` emits its producers in **first-consumption** order; **eight**
> single-symbol cells of the same statement shape emit them in **rank** order.
> The axis is the two base symbols, and `order.rs` refuses a multi-symbol run
> rather than guess. Board **#564**.

---

## 5. `mr r31,r3` — one fact, n = 1, recorded as a hypothesis

`xboxheap`'s live-range save of `this` across the trailing call sits between
`S2` and `S3` — at the slot rule 2 would give a *third* producer if that
producer's slot index were 3 rather than 2. **One instance. Not a rule.** It is
the single remaining instruction-order fact between SCHED and a byte-exact
`xboxheap`, and it needs its own grid (calls with a live range across them,
crossed against the number of store producers).

---

## 6. Reproducing it

```sh
python3 work/w-sched/control.py     # the 30 published cells, as a known-answer control
python3 work/w-sched/grid.py        # tiers 1-7  -> fit.tsv / holdout.tsv
python3 work/w-sched/grid2.py       # tier 8     -> fit2.tsv / holdout2.tsv
python3 work/w-sched/fit.py         # the 13,104-config list-scheduler search (FIT only)
python3 work/w-sched/model.py       # SCHED, scored on the fit partitions
python3 work/w-sched/external.py    # SCHED on the published cells and xboxheap
```

The holdout partition is written by the generators and the fitter refuses to
open it; `docs/rungs/_2026-08-05-w-sched-prereg.md` §4 declares the rule and
`docs/rungs/_2026-08-05-w-sched.md` §3 records the freeze commit the holdout was
scored against.

Everything needs the toolchain (wibo + `compilers/`) and compiles at the
**workload's** flags, read from `work/dc3-workload/flags.txt` rather than
transcribed. The measurement seam is `cl /FAsc`, which lets one compile carry
270 probe functions.
