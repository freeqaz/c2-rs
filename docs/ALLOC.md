# ALLOC — which register c2 gives each producer of a store run

`docs/STORE_SCHEDULE.md` §4 settles the **order** of a store run and names what
it does not cover:

> Predicting *which* register the allocator picks is a separate, open problem —
> `leaf_store.rs` already records four refuted rules for it and this document
> adds no fifth. A caller that cannot show the allocation is clean must refuse.

This is that problem. The implementation is `crates/c2-core/src/codegen/alloc.rs`;
the grid is `work/w-alloc/` (gitignored — the generator scripts are committed,
the `.cod`/`.obj` are not).

**All four refuted rules are derived consequences of one rule**, and every
killer cell is reproduced.

| lane | rules | killed by |
|---|---|---|
| `leaf_store.rs` | use count by `A1` | `{a=1;b=2;c=1;d=2}` |
| `leaf_store.rs` | live-range length by `A2` | — |
| `leaf_store.rs` | last-use by `B6` | `{a=1;b=1;c=2;d=2;e=2}` |
| `leaf_store.rs` | first-use by `B4`/`B7` | `{a=1;b=2;c=3;d=2;e=1}` |

---

## 1. ALLOC

> Enumerate the run's distinct value-producers. Order them by
>
> 1. **use count, descending** — the number of stores that consume the value;
> 2. on a tie, **register-derived** producers before **constant** ones;
> 3. on a tie within the register-derived, **source order**;
> 4. on a tie within the constants, **REVERSE source order**;
>
> and hand out the pool registers **descending** — `r11`, `r10`, `r9`, `r8`, …
> — in that order.
>
> The pool is the **free volatile registers taken highest-first, minus those
> holding live-in formals**. `r12` is never used.

A producer is **CONSTANT** when its materialisation reads no register (`li`,
`lis`+`ori`) and **REGISTER-DERIVED** otherwise (`addi`, `rlwinm`, …). Both are
read off the IL.

That is the whole rule and it has **no free parameters**.

### 1.1 Worked: the four cells that refuted four rules

`leaf_store.rs` records these as *"four allocation rules were fitted to those
and each is refuted by one of the others"*. Under ALLOC they are one rule:

```text
  {a=1;b=2;c=3;d=1}      counts 2,1,1   -> 1:r11  2:r10  3:r9
  {a=1;b=2;c=3;d=2;e=1}  counts 2,2,1   -> 1:r10  2:r11  3:r9
  {a=1;b=2;c=1;d=2}      counts 2,2     -> 1:r10  2:r11
  {a=1;b=1;c=2;d=2;e=2}  counts 2,3     -> 1:r10  2:r11
```

Row 1: value `1` has the strictly greatest count, so clause 1 alone puts it in
`r11`; `2` and `3` tie at count 1, and a **count-1 tie runs forward**, so they
take `r10` and `r9` in source order. Rows 2 and 3: `1` and `2` tie at count 2,
and a **count-≥2 tie among constants runs BACKWARD**, so the *second* one takes
`r11`. Row 4: no tie — `2` is used three times and takes `r11` outright, which
is the same answer for a different reason, and that coincidence is exactly why
fitting either "first-use" or "last-use" to rows 1 and 4 alone looked like it
worked.

### 1.2 The sign flip is the point

Clauses 3 and 4 carry **opposite signs inside one sort**. That is not a
stylistic choice in how the rule is written — it is why the rule is *not a
priority function*, and it is the reason four lanes searching priority
functions could not find it.

---

## 2. What ALLOC is not — measured, not asserted

A preregistered exhaustive search (`work/w-alloc/search.py`, declared in
`docs/rungs/_2026-08-05-w-alloc-prereg.md` §5 before the grid existed) covers
the class every textbook answer lives in:

```
4 scan directions x 3 assignment points x 2 pool walks
  x 2,184 lexicographic keys over 7 base features (signed)
  = 52,416 configurations
```

It tops out at

```
179 of 236 fit cells   —   and the residual is EXACTLY the tie tier
```

with **0 misses at every count where no two producers tie** and every miss in a
cell where two producers share a use count. No member of that family can carry
a tiebreak whose sign depends on the value being tied on, so none of them can
express clauses 3–4. This is the same shape as `w-sched`'s 13,104-configuration
result, and as there the residual is worth more than the score.

---

## 3. The pool, and where it goes when it runs out

`leaf_store.rs` recorded the pool as *"r11/r10/r9 descending"*. **It is not
three registers.** Measured:

| formals live | producers | pool actually used |
|---|---|---|
| 1 (`this`) | 4 distinct `addi` | `r11 r10 r9` **`r8`** |
| 6 (`r4`–`r9`) | 3 | `r11 r10` **`r5`** — a formal register freed by an emitted store |
| 7 (`r4`–`r10`) | 2 | `r11` **`r4`** |
| 7 | 2 shared | **`r3`** and `r11` — the base pointer itself, after its last use |
| 8 (one stacked) | 3 | **`r31`, `r30`** with a `std`/`ld` save-restore pair, plus `r4` |

`r12` is never allocated in any probed cell (board **#543** — recorded, not
explained).

**Register REUSE is what `w-sched`'s `conflicted()` predicate was detecting.**
In `{a=1;b=2;c=3;d=4}` the fourth `li` retakes `r11` after `stw r11,0(r3)`
frees it; `conflicted()` then sees a store reading `r11` that is not among that
producer's consumers and returns true. So `STORE_SCHEDULE.md` §4's **184
conflicted cells are pool-pressure cells**, and the table it prints —
conflicts starting at 2 producers, 107 of 110 at four — is the pool colliding
with `f4`→`r9` and `f5`→`r10` from *below*, an artifact of that grid's
`(M* p, M* q, unsigned f0..f5)` signature rather than a second phenomenon.

Past three producers the reuse choice is **open** (board **#541**): two
four-producer runs with identical statement structure disagree —
`{a=1;b=2;c=3;d=4}` reuses `r11`, while `{a=u+1;b=u+2;c=u+3;d=u+4}` takes a
fresh `r8`. `alloc.rs` refuses there.

---

## 4. The multiply is its own regime

A `mulli` producer is **never held live beside another producer**. It is
materialised one at a time, in `r11`, immediately before the stores that consume
it, with the consumers of one value emitted as a group:

```text
  {a=u*3; b=u*5;}                mulli r11 ; stw ; mulli r11 ; stw
  {a=u*3;b=u*5;c=u*5;d=u*5;e=u*3}  mulli r11 ; stw b ; stw c ; stw d ;
                                   mulli r11 ; stw a ; stw e
```

The groups come out in **count-descending order**, which is clause 1 again.
`alloc.rs` refuses a run containing a multiply rather than modelling this.

---

## 5. `docs/STORE_SCHEDULE.md` needs a scope condition — SCHED rule 2

SCHED rule 2 reads *"the producers, in source order, are inserted immediately
before the stores at store positions 0, 1, 2, … — one producer per store
slot"*. **That is right only while there are unproduced stores to slot
against.** Rule 1 keeps store positions 0 and 1 free of produced stores, so
there are `u = min(2, #unproduced)` such slots; producers fill those one
apiece, and **every remaining producer is emitted contiguously** immediately
before store slot `u`. Measured:

```text
  {a=1;b=2;}                     li r11,1 ; li r10,2 ; stw ; stw      P P S S
  {a=1;b=2;c=3;}                 li ; li ; li ; stw ; stw ; stw       P P P S S S
  {a=1;b=2;c=3;d=u}              li r11,1 ; stw r4 ; li r10,2 ;
                                 li r9,3 ; stw ; stw ; stw            P S P P S S S
```

The middle line is `P P P S S S`, not `P S P S P S`. `w-sched`'s grid always
had at least three formals and at most three producers, so it never ran out of
slots and the difference never appeared. **Board #542.**

`crates/c2-core/src/codegen/schedule.rs` is **not changed** by this lane. Its
only consumer is `is_source_order`, a guard, and on every shape the parser
admits today (an all-formal run; an all-same-literal run) the refined rule and
the shipped one give the same answer — so changing it would widen a refusal
without evidence to widen it on.

---

## 6. What is still open: the store ORDER when every store is produced

Rule 1 says what may **not** sit in the two head slots. It is silent on what
**fills** them when every store of the run is produced, a regime `w-sched`'s
grid never contained. Measured, one more store is hoisted into the head — the
first consumer of the producer with the **strictly greatest** use count, and
nothing is hoisted on a tie for the greatest:

```text
  {a=1;b=2;c=2}        counts 1,2   -> stores 1,0,2
  {a=1;b=2;c=1;d=2}    counts 2,2   -> stores 0,1,2,3   (tie: no hoist)
  {a=1;b=1;c=2;d=2;e=2} counts 2,3  -> stores 2,0,1,3,4
```

That is the same use count clause 1 sorts on, which is why the order and the
allocation were entangled and why fitting either alone kept failing.

**It is not complete.** With ALLOC supplying the registers and this hoist
supplying the order, the full emitted sequence is exact on

```
198 of 236 fit  (83.9%)      219 of 250 holdout  (87.6%)
```

and the residual is structured but not single-family: **35 of the 38 fit misses
are runs whose counts are (2,2,1) with no unproduced store**, and the holdout
adds a second family at counts (2,1) with two unproduced stores. **Nothing of
this section is shipped.** Board **#544**.

---

## 7. Reproducing it

```sh
python3 work/w-alloc/recon.py      # the discovery set, 13 probes
python3 work/w-alloc/recon2.py     # the discovery set, 23 probes
python3 work/w-alloc/grid.py       # 526 cells -> fit.tsv / holdout.tsv
python3 work/w-alloc/search.py     # the 52,416-config search (FIT only)
python3 work/w-alloc/supp.py       # the unequal-count and mixed-kind probes
python3 work/w-alloc/model.py      # ALLOC, scored on FIT
python3 work/w-alloc/external.py   # ALLOC on the four killer cells
python3 work/w-alloc/model.py --holdout
```

`search.py` **refuses** to open any path containing `holdout` — a positive check
that raises, not a convention. The holdout partition was declared in
`docs/rungs/_2026-08-05-w-alloc-prereg.md` §6 before `grid.py` was written and
scored only after ALLOC was frozen at commit `8973ffc`.

Everything needs the toolchain (wibo + `compilers/`) and compiles at the
**workload's** flags, read from `work/dc3-workload/flags.txt` rather than
transcribed. The measurement seam is `cl /FAsc`, which lets one compile carry
526 probe functions.
