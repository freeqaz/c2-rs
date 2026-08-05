# w-alloc PREREG — the register allocation, declared before the grid exists

    Tag:       w-alloc-prereg
    Slug:      w-alloc-prereg
    Date:      2026-08-05
    Fixtures:  none — a prereg admits nothing; it fixes what will be measured
    Record:    docs/rungs/_2026-08-05-w-alloc.md (the findings, written after)

`w-sched` shipped **SCHED** and named exactly one thing it does not cover:

> **SCHED is exact on 320/320 alloc-clean cells and 0/184 conflicted ones.**
> … The honest arity: SCHED is complete for the IL; the allocation is a SECOND
> INPUT and it is open.

This lane attacks that second input. Everything below is fixed **before**
`grid.py` is written, in the shape `w-sched` used, because that shape is the
only reason its holdout number means anything.

---

## 1. Reconnaissance — the discovery set, declared as such

Two probe compiles were run **before** this document, at the workload's own
flags through the real `cl.exe`/`c2.dll`, and the rule in §3 was read off them.
They are the **discovery set** and they are named here so that no later score
can quietly count them as evidence:

* `work/w-alloc/recon.py` — 13 functions: the four killer cells transcribed in
  `crates/c2-il/src/func/body/shapes/leaf_store.rs` (`K1`–`K4`), the
  all-distinct-literal ladder `C1`–`C5`, and four formal-pressure probes
  `P0`–`P3`.
* `work/w-alloc/recon2.py` — 23 functions: pool extent (`N4`,`N6`,`N8`),
  formal pressure to exhaustion (`F6`,`F7`,`F8`), the CSE/multi-use family
  `M1`–`MD`, shared-address `A1`,`A2`, and producer-kind `K1`,`K2`.

**36 functions total. Every rule score in the findings doc reports the
discovery set separately from FIT and from HOLDOUT.**

## 2. What the recon already establishes (not a hypothesis — measured)

1. The scratch pool is **the free volatile registers taken highest-first,
   `r11`, `r10`, `r9`, `r8`, …** — `r12` is never used. `recon2.K2`
   (`u+1,u+2,u+3,u+4`) reaches **`r8`**, so the pool is not the three registers
   `leaf_store.rs` recorded.
2. **Live-in formals are subtracted from the pool.** With 6 formals (`r4`–`r9`)
   the third producer lands in **`r5`** — a formal register freed by an
   already-emitted store — not in `r9`. With 8 formals it goes to **`r30`/`r31`
   with a save/restore pair**.
3. **Register REUSE is exactly what `w-sched`'s `conflicted()` predicate
   detects.** In `C3` (`1,2,3,4`) the fourth `li` retakes `r11` after
   `stw r11,0(r3)` frees it; `conflicted()` then sees a store reading `r11`
   that is not among that producer's consumers and returns true. The 184
   conflicted cells are therefore **pool-pressure cells**, not a separate
   phenomenon.
4. **`w-sched`'s grid could not see this.** Its signature is
   `(M* p, M* q, unsigned f0..f5)`, so `f4`→`r9` and `f5`→`r10` **collide with
   the pool from below**, and no cell in it has more than three producers or
   has every store produced. Both regimes are new territory here.

## 3. H1 — ALLOC, the rule this lane will be scored on

> Enumerate the distinct value-producers of a straight-line store run in source
> order of first use. A producer is **SHARED** if two or more stores consume it,
> **SIMPLE** if exactly one does. Assign the pool registers **descending**
> (`r11`, `r10`, `r9`, `r8`, …) to the producers taken in this sequence:
>
> 1. the **SHARED** producers in **REVERSE** source order, then
> 2. the **SIMPLE** producers in **source** order.

Equivalently, with `m` shared producers: the shared ones take `r(12-m)…r11`
ascending in source order, and the simple ones take `r(11-m)` downward.

**H1 has zero free parameters.** It is not a graph-colouring, not a linear
scan, and not a use-count priority — §5's search will say which of those
classes it is *not* in.

**Scope.** H1 is stated for runs in which every producer is simultaneously
live, i.e. **no register is reused**. The reuse regime is H2.

### 3.1 H1 must reproduce the four killer cells — predicted here, in advance

| cell | values | shared | m | H1 predicts | `leaf_store.rs` records |
|---|---|---|---:|---|---|
| `B4` | `1,2,3,1` | `1` | 1 | `1`→r11 `2`→r10 `3`→r9 | `1`→r11 `2`→r10 `3`→r9 |
| `B7` | `1,2,3,2,1` | `1,2` | 2 | `1`→r10 `2`→r11 `3`→r9 | `1`→r10 `2`→r11 `3`→r9 |
| `A1` | `1,2,1,2` | `1,2` | 2 | `1`→r10 `2`→r11 | `1`→r10 `2`→r11 |
| `B6` | `1,1,2,2,2` | `1,2` | 2 | `1`→r10 `2`→r11 | `1`→r10 `2`→r11 |

All four are in the **discovery set**. They are listed so that a later rule
which fails any of them is visibly disqualified, per the brief's constraint
that the four refuted rules are constraints and not noise.

## 4. H2, H3, H4 — declared, and allowed to fail

* **H2 (REUSE).** When the pool is exhausted, the register taken is the
  **highest-numbered free one** at the point the producer is emitted, where a
  register becomes free when the last store consuming its value has been
  emitted. *Predicted to be the weaker half.* `recon2.K1` and `recon2.K2` have
  identical statement structure and disagree (`r11,r10,r11,r9` vs
  `r11,r10,r9,r8`), so H2 is at risk from the discovery set already.
* **H3 (COVERAGE).** Re-running `w-sched`'s own 504-cell grid with H1 supplying
  the registers explains the **184 conflicted cells**. Scored as: of the 184,
  how many have their full emitted token sequence — registers included —
  predicted.
* **H4 (SCHED rule 2 is scoped).** `recon.P0`
  (`{a=1;b=2;c=3;d=u}` → `P1 S3 P2 P3 S0 S1 S2`) and every all-produced cell
  refute SCHED's producer-placement rule as stated. **Predicted: SCHED rule 2
  holds only when the number of producers does not exceed the number of
  unproduced stores.** If this is right it is a correction to a shipped
  document and it will be reported as one.

## 5. The corroborating negative, preregistered and run FIRST

Before H1 is scored, `search.py` runs an exhaustive search over a declared
family of **priority-function allocators** — the class every plausible textbook
answer lives in — and reports its ceiling and, more importantly, the
**structure of its residual**:

* direction ∈ {forward over statements, backward over statements, forward over
  the scheduled order, backward over the scheduled order};
* the value is assigned a register at ∈ {first use, last use, definition};
* the pool is walked ∈ {descending from r11, ascending into r11};
* priority key = a lexicographic tuple of up to **three** signed features drawn
  from {use count, first-use index, last-use index, live-range length, is-shared,
  producer's operand count, statement index of definition}.

That is 4 × 3 × 2 × (14 signed features choose an ordered 3-tuple, 2184) =
**52,416 configurations**. `search.py` may read **`fit.tsv` only**.

**A ceiling below 100 % whose residual is structured is a full result for this
lane** — that is exactly what `w-sched`'s 13,104-configuration search bought.

## 6. THE HOLDOUT — declared now, mechanical, and unopenable by the fitter

`grid.py` partitions every cell at generation time and writes
`fit.tsv` / `holdout.tsv`. **`search.py` and `model.py` refuse to open any path
containing `holdout`** — a positive check with a printed count, not a
convention.

A cell is **HELD OUT** if any of:

1. `sha1(cid) % 4 == 0` — a mechanical quarter, no structure;
2. it is in **tier 3** (the shared/simple mixtures at n = 5) — the tier H1's
   two-clause ordering is most exposed on;
3. it has **exactly 4 distinct producers** — the pool-pressure boundary;
4. it is in **tier 6** (formal pressure) with **≥ 6 formals** — the regime
   where the pool descends into freed formal registers.

Clauses 2–4 are *structural* holdouts: they remove whole regimes, not a random
sample, so a rule that interpolates will fail them. Clause 1 is the unstructured
control.

## 7. Freeze protocol

1. `grid.py` runs; `fit.tsv` and `holdout.tsv` are written. **`holdout.tsv` is
   not read, printed, or listed.**
2. `search.py` runs on `fit.tsv`. Its ceiling and residual are recorded.
3. `model.py` is written, fitted on `fit.tsv` only, and **committed**. The
   commit SHA is quoted in the findings doc.
4. Only then is `model.py --holdout` run. Its number is reported whatever it is.

## 8. Predictions, on the page, to be graded

| # | prediction | grade |
|---|---|---|
| R1 | H1 is ≥ 95 % exact on FIT cells with no register reuse | |
| R2 | H1's HOLDOUT accuracy is within 3 pp of its FIT accuracy | |
| R3 | The priority-allocator search of §5 tops out **below** H1 | |
| R4 | The search's residual is **structured** — concentrated in cells with m ≥ 2 | |
| R5 | H2 (reuse) scores **below 80 %** and is not shipped | |
| R6 | H3: ≥ 150 of `w-sched`'s 184 conflicted cells are explained | |
| R7 | H4: SCHED rule 2 needs the scope condition; `w-sched`'s doc is corrected | |
| R8 | `xboxheap.cpp` does **not** convert from this lane alone — the `mr r31,r3` fact is a live-range question this grid does not probe | |

## 9. What will be shipped, and how

Nothing ships unless it survives §6's holdout **and** all four killer cells.
Following `w-sched`'s pattern and `#232`'s precedent, the emitter will carry a
**positive guard**: the allocator is asked for its assignment and the store-run
emitter **refuses** when the assignment is outside the proven regime, rather
than emitting a guess. A widening that reaches the guard turns into a clean
refusal, not a wrong emit.

## 10. Board items taken

* **#540** — ALLOC: the store-run register assignment (this lane).
* **#541** — the reuse/pool-pressure regime (H2), expected to stay open.
* **#542** — SCHED rule 2's scope condition (H4), a correction to
  `docs/STORE_SCHEDULE.md` §1 if R7 grades HIT.
* **#543** — `r12` is never allocated; recorded, not explained.
