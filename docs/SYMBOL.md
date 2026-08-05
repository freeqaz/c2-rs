# SYMBOL — a store run through more than one base symbol

`docs/ORDER.md` settles a store run through **one** base symbol and refuses
every other run rather than guess. `docs/STORE_SCHEDULE.md` §3 names the axis —
the base **symbol** of the store's address expression, not the machine
register — and `w-parse` proved it is the reference **BIND** (board #580:
`p->e.eK = v` and `E& l = p->e; l.eK = v` write the same bytes through the same
base register at the same displacement, and **100 of 566 pairs emit different
orders**).

This is what the multi-symbol regime *is*. The implementation is
`crates/c2-core/src/codegen/order.rs`; the grid is `work/w-sym/` (gitignored —
the generator scripts are committed, the `.cod`/`.obj` are not).

**A run's emission is three separable facts.** Keeping them apart is the whole
method here, because two of them are now closed and the third is not, and every
earlier attempt scored them as one number.

| | fact | status |
|---|---|---|
| 1 | the **cross-symbol pin** — which symbol each slot holds | **exact**, model-free, board #601 |
| 2 | the **store order** — the permutation inside each symbol group | **exact to 2 producers**, board #600, shipped |
| 3 | the **producer emission order** | **a case split**, board #582, answered but not unified |
| 4 | the **layout** — which slots the producers sit before | **open**, board #602 |

---

## 1. The pin, and it is not a modelling choice

> **THE PIN.** Two stores through different base symbols are never reordered
> past each other. Equivalently: the sequence of base symbols in the emitted
> store order is **identical** to the sequence in source order.

Measured with no rule involved at all, by reading the emitted symbol pattern
off the listing and comparing it to the source: **7,589 of 7,589 cells, 0
violations** — 3,206 fit, 4,367 holdout (which includes the whole three-symbol
tier) and 16 external, across 1, 2 and 3 symbols, 1 to 6 producers, runs of 2
to 7 statements.

This is `STORE_SCHEDULE.md` §3's may-alias clause, promoted from a modelling
assumption to a measurement, and it is what makes the rest tractable: **the
store order can only permute *within* a symbol group**, so the multi-symbol
problem is the composition of per-group permutations and nothing else.

`order.rs`'s `the_emitted_symbol_pattern_is_always_the_source_pattern` test
enumerates every run of 2..5 statements over 4 producer alphabets × 2 symbols
and asserts the pin on each — a positive check with a printed count.

## 2. The store order — one walk covers both regimes

> **SYMORDER-U.** Rank the run's distinct producers globally by
> **(use count descending, first-use ascending)**. A store's rank `j` is its
> producer's position in that order **restricted to the producers of the
> store's own base symbol**. Walk the source statements and emit the earliest
> store that is *allowed*: a store of rank `j` may not occupy position
> `< u + j`, and no store may be emitted past a still-unemitted store of a
> different symbol.
>
> **`u` is the largest value in `0 … min(2, #unproduced)` for which no slot
> ever runs out of allowed stores.**

That last clause is the whole change, and it *deletes* rather than adds.
`w-sched` rule 1 ended with *"if every remaining store is blocked, source order
wins"*; `w-order2` deleted that fallback for one symbol by observing the floors
drop with `u`. `w-parse`'s SYMORDER carried it back, because with two symbols
the pin really can block every store — the unproduced stores that were supposed
to fill the head are pinned behind produced ones. **Lower `u` instead of
relaxing a floor** and the fallback disappears again.

### 2.1 It reduces to `ORDER` exactly, and that is proven by construction

`order.rs` has **one** walk. With one symbol the pin clause is vacuous, the
group rank is the global rank, and the largest affordable `u` is
`min(2, #unproduced)` — `order.rs`'s own enumerating test still walks 5,460
single-symbol runs to confirm the relaxation never fires there. Every cell
`ORDER` was fitted on goes through the new code and comes out unchanged; the
reduction is not a claim beside the code, it *is* the code.

### 2.2 What it is worth

| population | SRC | IGNORE (`ORDER`) | SYMORDER (`w-parse`) | PSYM | **SYMORDER-U** |
|---|---:|---:|---:|---:|---:|
| fit, all 3,206 | 68.5 % | 50.7 % | 89.1 % | 59.8 % | **98.7 %** |
| fit, multi-symbol 2,658 | 75.8 % | 40.6 % | 86.9 % | 51.5 % | **98.4 %** |
| holdout, multi-symbol 4,040 | 78.5 % | 54.5 % | 89.1 % | 66.6 % | **92.9 %** |

**`PSYM` is a refuted preregistered prediction** (R3). Scheduling each symbol
group *independently* by `ORDER` — its own `u`, its own positions — and merging
under the pin was this lane's declared favourite for the store order and it is
**far worse than the global reading**: 51.5 % against 86.9 %. The floors are
counted in **global** slots, not group-local ones. That is measured, and it is
the opposite of what §1's group *rank* does.

### 2.3 The domain, and it is stated because it was measured

| population | cells | store order | wrong |
|---|---:|---:|---:|
| fit, ≤ 2 producers | 1,867 | **1,867** | **0** |
| holdout, ≤ 2 producers, single-kind | 1,501 | **1,501** | **0** |
| external (`xboxheap`'s word, 4 symbol splits × 4 kind pairs) | 16 | **16** | **0** |
| holdout, ≤ 2 producers, **mixed-kind** | 144 | 132 | 12 |
| fit, 3 producers, multi-symbol | 1,212 | 1,170 (96.5 %) | 42 |
| holdout, 3 producers, multi-symbol | 1,441 | 1,274 (88.4 %) | 167 |

`order.rs` refuses past **two** producers through more than one symbol. The
three-producer residual is spread over six `(counts, unproduced, spanning)`
shapes with no dominant family — 42 cells, the largest bucket 14 — so there is
nothing to fit and nothing is fitted.

**Every one of the 12 in-domain holdout misses mixes producer KINDS** (a
constant with an `addi`), which is board #581's population and was held out
wholesale by the preregistered partition. `Stmt` cannot express the kind and
`leaf_store.rs` only ever builds constant producers, so the domain holds today
by construction — board **#603** records that a parser widening to
address-valued producers invalidates it.

## 3. The producer emission order — board #582, answered as a CASE SPLIT

> **One symbol** → the **rank** order (`ORDER` #561).
> **More than one** → the order of **first consumption in the final store
> order** (`w-alloc`'s rule).

Both halves are exact in the shipped domain — 1,867/1,867 fit, 1,645/1,645
holdout (mixed-kind cells **included**, so unlike the store order the producer
order is kind-independent), 16/16 external.

**It is a case split and this lane could not unify it.** That is the finding,
not a shortcut:

* `xboxheap`'s statement word emits the **count-2** producer first through one
  symbol and the **count-1** producer first through two — same statements, same
  producers, same counts, same registers. Both are in the tests.
* The preregistered exhaustive negative covered the class every "sort the
  producers on their own features" answer lives in: a lexicographic key of up
  to 3 signed features over 10 — `{group rank (min / max over the producer's
  groups), global rank position, use count, first use, first consumption,
  index of the first group, number of symbols spanned, last use, count within
  the first group}` — **8,420 configurations**, scored on FIT conditional on
  the observed store order. It tops out at **2,580 of 2,604 (99.1 %)** and the
  winner is `+first consumption`; **no member covers the single-symbol side**,
  where the rank is exact and first consumption is wrong.
* **The residual named the mechanism, and the mechanism then failed too.** All
  24 misses are three-producer cells, and 16 of them emit the *rank* order —
  the shape where two producers share a symbol group and their rank disagrees
  with their consumption. That is an **insertion/merge**, not a sort, which is
  the fourth consecutive lane whose answer was outside the class it searched.
  Four merge rules were built from it (`QUEUE-G`, `QUEUE-L`, `SYMMERGE`,
  `SYMMERGE-L`, `model.py`). The best, `SYMMERGE` — each producer belongs to
  the group of its first consumption, within a group the restricted global rank
  is a precedence constraint, and otherwise the earliest consumption wins —
  reaches **96.4 % fit / 93.1 % holdout** and **is beaten by the plain case
  split**. It dies on `x_split`, where two producers live entirely inside one
  symbol group and still come out in consumption order.

**`SYMRANK` — this lane's own preregistered central hypothesis — is refuted.**
Emitting producers by `(group rank ascending, first use ascending)` reduces to
the rank order on one symbol *by construction* and gives `xboxheap` the right
answer, and it scores **82.8 % fit / 71.7 % holdout**, barely above the plain
rank order it was meant to repair. Its `grank = min over groups` clause for
spanning producers (prereg R6) is **retired, not scored**: the rule that used
it is gone, so the prediction has no population left.

## 4. The layout — the one that is still open

Given the store order and the producer order, **where** do the producers sit?
`ORDER`'s layout with #584's leading-run `u` — the first `u` producers one
apiece before slots `0 … u-1`, the rest contiguously before slot `u` — is

| population | cells | layout exact |
|---|---:|---:|
| fit, ≤ 2 producers | 1,867 | 1,866 |
| holdout, ≤ 2 producers | 1,645 | 1,644 |
| **external** | 16 | **12** |
| fit, 3 producers, multi-symbol | 1,212 | 998 (82.3 %) |

and the four external misses are **one family**, `x_split`:

```
 syms 0,0,0,0,1,1   P0 · stw m0 · P1 · stw m1 · stw m2 · stw m3 · stw e4 · stw e5
 syms 0,0,1,0,1,1   P0 · stw m0 · stw m1 · P1 · stw e2 · stw m3 · stw e4 · stw e5
```

Same statements, same store order, same producer order, same registers; one
store moved to the other symbol and **the second producer lands one slot
later**. The single in-domain fit miss, `b0001_xvxvvv_T_100101`, has the same
signature. That is board **#602**, and it is why `schedule()` — which returns
the full interleaved sequence — **still refuses a multi-symbol run** while
`store_order` and `is_source_order` now answer. A caller needs the layout, so
no multi-symbol emitter can be built on this yet.

## 5. Arity — three symbols need no new constant

The **whole three-symbol tier was holdout** by preregistered clause 3, so it
was never available to fit against. At ≤ 2 producers it is **406 of 406** on
the store order, the producer order *and* the layout. `order.rs`'s
`three_symbols_need_no_new_constant` carries three of those cells.

## 6. The register pool — #543 answered, #541 still open

Two lanes declined board #541/#543 because their probe signatures were capped
below the pool. `work/w-sym/`'s tier G uses `(M* p, M* q)` and nothing else, so
`r5 … r11` are all free, with 4, 5 and 6 distinct producers. Measured on 24
cells:

* the pool descends **`r11, r10, r9, r8, r7`** — `r7` is reached whenever five
  producers are simultaneously live, and this is the first direct measurement
  past `r9`;
* **`r12` is never used**, 0 of 24 — #543's record confirmed;
* a register freed by an already-emitted store **re-enters** the pool, and
  #541's `li`-vs-`addi` disagreement — recorded in `alloc.rs` from **two**
  probes — reproduces on **all 12 pairs**: with four producers and four stores,
  `li` retakes `r11` where `addi` takes a fresh `r8`.

`alloc.rs` is **unchanged**: its doc already states the descent and the `r12`
skip, and its `MAX_MODELLED_PRODUCERS = 3` refusal is exactly where the
disagreement starts. This lane corroborates the shipped code at n = 24 and
extends the observed floor to `r7`; **the reuse rule itself is still #541** and
nothing was fitted to it.

## 7. Reproducing it

```sh
python3 work/w-sym/grid.py         # 7,589 cells -> fit / holdout / external
python3 work/w-sym/truth.py        # the MODEL-FREE facts, incl. the pin
python3 work/w-sym/raise_check.py  # the holdout raise, demonstrated
python3 work/w-sym/search.py       # the 8,420-configuration sort search (FIT)
python3 work/w-sym/resid.py        # that search's residual, cell by cell
python3 work/w-sym/model.py        # 6 store-order x 7 producer-order rules
python3 work/w-sym/layout.py       # the layout alone
python3 work/w-sym/arity.py        # every component by (producers, symbols)
python3 work/w-sym/arity.py --external   # xboxheap's word, both ways
python3 work/w-sym/arity.py --holdout    # only after the freeze
```

The holdout partition was declared in
`docs/rungs/_2026-08-05-w-sym-prereg.md` §6 **before** any generator of this
lane existed, is applied by `symlib.held_out` inside the generator, and
`symlib.read_rows` **raises** on any path containing `holdout` —
`raise_check.py` demonstrates the raise on four spellings and then loads the
fit table, so an absent raise fails loudly instead of passing quietly. The rule
was frozen at commit **`3f4716a`** and the holdout opened once, after.

Everything needs the toolchain (wibo + `compilers/`) and compiles at the
**workload's** flags, read from `work/dc3-workload/flags.txt` rather than
transcribed. The measurement seam is `cl /FAsc`: 7,589 probe functions in one
compile, 2.9 seconds.

**Import by explicit file path.** `work/w-alloc/`, `work/w-order2/`,
`work/w-parse/` and now `work/w-sym/` all carry a `model.py` and a `search.py`;
a bare `import search` resolves by `sys.path` order and surfaces as a missing
attribute, not a wrong module.
