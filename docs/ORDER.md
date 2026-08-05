# ORDER — the store order of a run when the head slots are contested

`docs/STORE_SCHEDULE.md` settles the order of a store run whenever *some* store
of the run needs no producer, and names what it does not cover. `docs/ALLOC.md`
§6 states it as board **#544**:

> Rule 1 says what may **not** sit in the two head slots. It is silent on what
> **fills** them when every store of the run is produced … the full emitted
> sequence is exact on **198 of 236 fit (83.9 %), 219 of 250 holdout (87.6 %)**
> and the residual is structured but not single-family.

This is that residual. The implementation is
`crates/c2-core/src/codegen/order.rs`; the grid is `work/w-order2/`
(gitignored — the generator scripts are committed, the `.cod`/`.obj` are not).

**The residual was exactly two families plus a five-cell measurement
artifact**, and one rule with one constant covers both — a constant that is
already `w-sched`'s.

> ### ⚠ THE POPULATION THIS RULE WAS FITTED ON — board #644, added by `w-wire`
>
> **Every producer in every grid behind this document is a SINGLE-WORD `li`.**
> That restriction was never stated here, and it is not implied by anything the
> rule says. It is real:
>
> ```text
>   { a=100000; b=1; }   lis r11 ; li r10 ; ori r11 ; stw r10,4(r3) ; stw r11,0(r3)
> ```
>
> — real `c2`, identical at `/O1` and `/Ox` (`work/w-wire/boundary_probe.py`).
> Two things break at once. The `lis`/`ori` pair is **SPLIT** by the other
> producer, so a producer is not one contiguous instruction and `layout_slots`
> — which places producers by *index* — cannot express the sequence at all; and
> the **store order is `[1, 0]`** where §1's walk says source order. A second
> cell, `{a=100000; b=200000;}`, comes back `lis lis ori ori`, confirming that
> c2 interleaves the halves rather than emitting either pair whole.
>
> A run whose **only** producer is wide is unaffected — one live range, nothing
> to interleave with — and `{a=100000;b=100000;}` is `lis ; ori ; stw ; stw`, a
> cell the parser already admitted.
>
> `leaf::store` therefore refuses **more than one producer where any literal
> needs more than one word**. Anyone re-fitting or widening the rule below owes
> a grid that contains wide values; this one does not.

---

## 1. ORDER

> Rank the run's distinct value-producers by
> **(use count descending, first-use source index ascending)**.
> Let **`u = min(2, number of unproduced stores)`**.
>
> * **Store order.** A store whose producer has rank `j` may not occupy store
>   position `< u + j`; an unproduced store is never blocked. Walk the source
>   statements in order and emit the earliest **allowed** store.
> * **Producer emission order** is the rank order.
> * **Layout.** The first `u` producers go one apiece immediately before store
>   slots `0 … u-1`; every remaining producer is emitted **contiguously**
>   immediately before store slot `u`.

**One wording correction, 2026-08-05, lane `w-parse`, and it is a no-op here.**
The layout's `u` is the length of the **leading run of unproduced stores in the
FINAL store order**, capped at 2 — not `min(2, #unproduced)`. On this grid the
two are the same number, because the floors put the unproduced stores in the
head; on a run through more than one base symbol they are not, and
`min(2, #unproduced)` puts a producer before a slot that holds a *produced*
store. Measured: switching to the leading-run reading changes **nothing** on
all 809 single-symbol cells and is what the `mr rN,r3` rule
(`STORE_SCHEDULE.md` §5) needs. Board **#584**.

**And it was a no-op only in prose until 2026-08-05, lane `w-frame2`.**
`order::schedule` was still computing the layout's `u` with `head_slots`, i.e.
`min(2, #unproduced)`, four lanes after the correction above was written. On one
symbol that is genuinely a no-op — `order.rs` now *enumerates* every run of
length 1..=6 over ≤ 3 producers and asserts the two readings agree — but on the
multi-symbol runs `#600` opened, it is **62.90 %** against the leading run's
**98.59 %**. A correction that lands in the doc and not in the code is a
correction that has not landed. Board **#621**.

**The layout clause above is right only on a gated domain**, and the gate is
board **#620**: let `nsw` be the number of symbol-group transitions in the final
store order up to and including a producer's first consumption. At `nsw ≤ 2` the
clause is exact (30,271 fit / 24,891 holdout / 54 external, 0 wrong); past it,
the producer lands one slot later than the clause says. On **one** symbol `nsw`
is 0 for every producer, so the gate is vacuous here and this section is
unchanged where it was fitted. `docs/SYMBOL.md` §4.1.

That is the whole rule. The only free constant is the **2**, and it is rule
1's own. `MAX_SYMBOL_CROSSINGS` is a **second** 2 — measured independently off
the crossing count and *not* derived from rule 1's. Whether they are the same
constant is unmeasured, and it is written as two constants so that a lane that
moves one does not silently move the other.

### 1.1 The rank is not the register order, and the signs disagree

`ALLOC` breaks a use-count tie **among constants** by *reverse* source order.
The rank breaks it by *forward* source order. `{a=1;b=2;c=1;d=2;e=3}` puts the
value `2` in `r11` and emits its producer **second**:

```text
  li r10,1 ; li r11,2 ; li r9,3 ; stw r10 ; stw r11 ; stw r10 ; stw r11 ; stw r9
     ^ rank 0, register r10        ^ rank 1, register r11
```

Two orders over the same three producers, disagreeing in sign. That is why
fitting the order and the register separately kept failing, and why
`w-alloc`'s `predict_seq` — which emitted producers in order of first
consumption — was right on 198 of 236 and wrong in a structured way on the
rest.

### 1.2 Both published special cases are consequences

* **`w-sched` rule 1** — *"a produced store may not occupy store position 0 or
  1"* — is rank `j = 0` with `u = 2`. Every produced store has rank `≥ 0`, so
  every produced store is blocked from positions `< 2` exactly when there are
  two unproduced stores to fill them. Rule 1's *"if every remaining store is
  blocked, source order wins"* fallback disappears: with `u < 2` the floors
  drop with it, and the rank-0 producer's stores are always allowed at slot
  `u`.
* **`w-alloc`'s hoist** — *"the first consumer of the **strictly** greatest use
  count moves into the head, and nothing moves on a tie"* — is rank `j = 0`
  with `u < 2`. The rank-0 producer is the greatest-count one; on a tie the
  two tied producers take ranks 0 and 1 **in source order**, so the earliest is
  already first and nothing appears to move.

`crates/c2-core/src/codegen/order.rs`'s tests recompute both from the rule
rather than transcribing them.

---

## 2. The residual, characterised before it was fitted

Over `w-alloc`'s 526-cell grid — **both** partitions, which is this lane's
declared discovery set because that holdout has already been scored and its
families are published:

| family | counts | unproduced | cells | `w-alloc` misses |
|---|---|---:|---:|---:|
| **A** | `(2,2,1)` | 0 | 99 | **44** |
| **B** | `(2,1)` | 2 | 62 | **20** |
| artifact | any | any | 5 | **5** |

and **nothing else**: every other `(counts, unproduced)` family is 0 miss.

* **Family B was never a new regime.** Inside it the miss is exactly the 20
  cells whose produced sub-word begins with the **use-count-1** value; the 42
  beginning with the shared value are hits. `w-alloc` gated its hoist on the
  unproduced stores running out. They had not run out — the hoist fires anyway.
* **Family A is a rank effect and not a count effect.** In every family-A cell
  two producers tie at count 2 and one has count 1, and the count-1 store is
  displaced to position 2. In counts `(2,1)` with no unproduced store the
  count-1 store **stays at position 1**. Nothing that reads a store's own use
  count can separate `aba` from `abcac`.
* **The five artifact cells are kind `W`** (`lis`+`ori`). That producer is
  **two** instructions and every canon in `work/` emits one token per producer.
  They are misses of the *observation*, not of the order. Board **#562**.

---

## 3. What ORDER is not — measured, not asserted

A preregistered exhaustive search (`work/w-order2/search.py`, declared in
`docs/rungs/_2026-08-05-w-order2-prereg.md` §5 before the grid existed) covers
the class every release-time answer lives in — a threshold on the store's
**own** features:

```
2 counters (absolute store slot / produced-store index)
  x 4^9 thresholds over {unproduced} u {count 1,2,3,>=4} x {first use?}
  x 2 tiebreaks (earliest / latest source order)
  = 1,048,576 configurations
```

`w-sched` rule 1 is a member of it. It tops out at

```
196 of 250 fit cells  (78.4 %)

RESIDUAL of the best configuration:
  rank order != producers' source order    93 cells,  50 miss
  rank order == producers' source order   157 cells,   4 miss
```

**Every rule in that class is blind to the rank**, because two producers can
tie on the use count and still take different ranks — and the residual says so
in its own shape. This is the third time in three lanes that the answer was
outside the class being searched, and the third time the residual named the
missing mechanism rather than the score doing it.

*(The lane's prereg R4 predicted this residual would concentrate on cells
where two producers **tie**. It concentrates on the strictly more general
`rank ≠ source order` instead — and it had to, because the preregistered
holdout clause 2 had already removed nearly every tie cell from FIT. That is a
graded miss, recorded in the findings doc.)*

---

## 4. Evidence

| population | cells | in domain | ORDER exact | **wrong** |
|---|---:|---:|---:|---:|
| discovery (`w-alloc`'s grid, both partitions) | 526 | 479 | **479** (100.0 %) | **0** |
| fit | 250 | 248 | **248** (100.0 %) | **0** |
| **holdout** | 572 | 561 | **561** (100.0 %) | **0** |
| holdout, shapes absent from discovery | — | 223 | **223** | **0** |
| store order alone, fit + holdout | 822 | 822 | **822** | **0** |
| published cells (`o7`, with registers) | 2 | 2 | **2** | **0** |

The **store order alone** column reaches past `ALLOC`'s three-producer domain:
four-producer runs, seven- and eight-statement runs, `this`-valued fillers and
the tie tier are all exact on the order even where the register is refused.

**The relaxation branch never fires** — not on any of the 1,301 measured cells,
and not on any of the 5,460 runs up to six statements over three producers and
a filler that `order.rs`'s own enumerating test walks.

The holdout partition was declared in
`docs/rungs/_2026-08-05-w-order2-prereg.md` §6 **before** `grid.py` was
written, decided by the generator, and scored only after the rule was frozen
at commit **`980e42e`**. `search.py` **refuses** to open any path containing
`holdout` — a raise, not a convention.

### 4.1 The one correction, made on FIT and recorded

The rule frozen *with the prereg* (`work/w-order2/order.py`, commit `7ee557e`,
before the grid existed) counted **produced stores already emitted** rather
than the store slot. It scores **246 of 248** on FIT and both misses are one
shape — three unproduced fillers ahead of a produced word beginning with the
lower-ranked producer. Discovery could not contain that shape: its runs are
five statements long, so three fillers leave two producers whose ranks can
never disagree with source order.

The correction replaces the counter with `u + j` and makes the rule **shorter**
— it is what lets rule 1 and the fallback be deleted rather than carried.
`order.py` is kept committed beside `order2.py` so the frozen original is on
the page.

---

## 5. What ORDER refuses, and why the refusals are not conservatism

* **More than one base symbol.** `xboxheap.cpp`'s constructor — the FRONTIER's
  only branch-free TU — stores through two symbols and emits its producers in
  **first-consumption** order:

  ```text
    li r10,0 ; stw r5,16(r3) ; addi r11,r3,8 ; stw r3,0(r3) ; …
    ^ the count-1 producer, emitted FIRST
  ```

  **Eight cells of this lane's grid have the same statement shape through one
  symbol** — `t3_011_FFFvvv`, `t3_011_vFFFvv` (fit) and six more (holdout) —
  and every one of them emits `r11`, the **rank-0** producer, first. Two
  symbols, opposite answer, and `ORDER` refuses rather than guess which.
  Board **#564**. This upgrades `STORE_SCHEDULE.md` §5's *"one instance, not a
  rule"* into a **discriminated** fact with 8 controls on one side and 1 on the
  other.

  > ### ✔ The AXIS is resolved 2026-08-05 by lane `w-parse` — and the REFUSAL stays
  >
  > Those eight controls differ from `xboxheap` on **four** axes, not one: the
  > filler identity, the **per-producer kind** (this grid uses one kind for
  > every producer of a cell), the base symbol, and whether the address
  > producer's value *is* the second symbol. Crossing all four on
  > `xboxheap`'s own word gives **36 cells whose answer depends on the symbol
  > and on nothing else** — every kind pair and every filler mixture emits the
  > same tokens at each symbol level (`docs/rungs/_2026-08-05-w-parse.md` §3).
  >
  > **And the axis is the reference BIND, not the offset range.** `p->e.eK = v`
  > and `E& l = p->e; l.eK = v` write the same bytes through the same base
  > register at the same displacement; **100 of 566 such pairs emit different
  > instruction orders**. Board **#580**.
  >
  > **The refusal is still correct.** The *store* order generalises — a store
  > of rank `j` is floored at `u + j` with `j` taken among **its own symbol
  > group's** producers, plus the cross-symbol pin, which reduces to this rule
  > exactly on all 809 single-symbol cells and reaches 91.9 % on multi-symbol
  > ones. The **producer emission order** does not: rank order is
  > **4459 / 5053** on multi-symbol cells and `w-alloc`'s first-consumer order
  > is **822 / 857** on single-symbol ones, so **both candidates are refuted**
  > and there is no rule to ship. Board **#582**.
  >
  > ### ✔ SUPERSEDED 2026-08-05 by lane `w-sym` — see `docs/SYMBOL.md`. THE REFUSAL IS LIFTED FOR THE STORE ORDER
  >
  > **The store order needed one more change and it DELETES a clause.** `u` is
  > the **largest** value in `0 … min(2, #unproduced)` for which no slot runs
  > out of allowed stores — lower `u` rather than relax a floor, which is what
  > `w-order2` did for one symbol and what SYMORDER above carried back. Worth
  > **86.9 % → 98.4 %** on multi-symbol cells, and **exact at up to two
  > producers**: 1867/1867 fit, 1501/1501 holdout, 16/16 external, 0 wrong.
  > `store_order` and `is_source_order` now **answer** a multi-symbol run
  > instead of returning `None`. Board **#600**.
  >
  > **`w-sym`'s own preregistered store-order favourite is refuted**:
  > scheduling each symbol group independently and merging under the pin
  > (`PSYM`) scores **51.5 %** against SYMORDER's 86.9 %. The floors are
  > counted in **global** slots even though the rank is taken **per group**.
  >
  > **#582 is answered as a CASE SPLIT, not a unification** — one symbol → the
  > rank order, more than one → first consumption; both exact in the shipped
  > domain, and a preregistered 8,420-configuration search over every
  > lexicographic sort key on 10 producer features shows no single sort covers
  > both sides. `schedule()` **still refuses** a multi-symbol run because the
  > **layout** does not generalise (board **#602**): the same statements with
  > one store moved to the other symbol put the second producer one slot later.
  >
  > **The cross-symbol pin is now a measurement, not an assumption** — the
  > emitted symbol pattern equals the source pattern on **7,589 of 7,589**
  > cells. Board **#601**.
* **More than three distinct producers**, matching `ALLOC`'s domain. The order
  alone is exact there (822 of 822); the register is not, and a caller needs
  both.

---

## 6. Reproducing it

```sh
python3 work/w-order2/resid.py --detail  # the residual by family (discovery)
python3 work/w-order2/fam.py 221 0       # hit vs miss inside family A
python3 work/w-order2/truth.py           # the raw observation table, model-free
python3 work/w-order2/grid.py            # 822 cells -> fit.tsv / holdout.tsv
python3 work/w-order2/search.py          # the 1,048,576-config search (FIT only)
python3 work/w-order2/resid2.py          # that search's residual, by family
python3 work/w-order2/order2.py          # ORDER on discovery and on FIT
python3 work/w-order2/model.py           # FIT
python3 work/w-order2/model.py --order   # the store order alone
python3 work/w-order2/external.py        # o7, xboxheap, rank vs first-consumer
python3 work/w-order2/model.py --holdout # only after the freeze
python3 work/w-order2/cells.py           # order.rs's test cells vs real c2
```

`cells.py` is the one to run when editing `order.rs`'s tests: it renders every
asserted cell **and cross-checks it against a grid row compiled by real c2**,
failing rather than reporting when the two disagree.

Everything needs the toolchain (wibo + `compilers/`) and compiles at the
**workload's** flags, read from `work/dc3-workload/flags.txt` rather than
transcribed. The measurement seam is `cl /FAsc`, which lets one compile carry
822 probe functions in a quarter of a second.

**A trap for the next lane:** `work/w-alloc/` and `work/w-order2/` both contain
a `model.py` and a `search.py`. A bare `import search` resolves to whichever
lane's directory is earlier on `sys.path`, and it silently resolved to
`w-alloc`'s here — reporting a missing attribute rather than a wrong module.
Everything in this lane imports by explicit file path.
