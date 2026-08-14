# w-fenceb — PREREG

**Frozen and committed BEFORE the first `crates/` change and before the first
`cl.exe` this lane runs for a measurement.** (The one `cl.exe` that ran ahead of
this file is `scripts/setup_worktree.sh`'s own toolchain-liveness check — it
compiles `fixtures/cpp/w5_chain.cpp` and prints `4/4`, and no number from it
appears anywhere in this lane. Said rather than left implicit.)

Lane **w-fenceb**, base `ac3cdd8c`, branch `wt-w-fenceb`.

The commission is two tasks: **score `R1′`**, which `w-backedge` filed as
*"the next lane's prereg, not this lane's result"* because it was fitted after
its hold-out was opened; and **price lifting fence B** two-sided.

---

## §0 — the grids, frozen by CONTENT HASH

```text
  grid1.tsv   3dd6e18f2b857875a9b11ee873137a6c1d0c5f9cd6a3cce1dfbf7e52120a62cd
  grid2.tsv   e1e2a5a2623479b472ba10a80eb8a6deb8deeb4daaae11e004de3059a96d1e54
  grid3.tsv   96cb9bea0aa0879b603552be734842c1d7455526ea6e346c5ce3268d0c621863
```

`grid1`/`grid2` are `w-backedge`'s two grids, **copied byte-identically** into
this lane (their sha256 reproduces its `PREREG.md` §0 exactly, which is the
check that the copy is a copy). They are this lane's **fitting set** and nothing
scored on them counts as a hold-out result — `R1′` was fitted to their
residuals.

`grid3` is **this lane's hold-out**, written before any of it was compiled.

### grid3's design, and why it can separate what grid1 could not

`w-backedge` §4: *"No grid1 cell has two backward references to one target — the
fitting set was structurally incapable of separating per-reference from
per-target."* grid3 is the cross

```text
   {while, for, do/while}  ×  {continue 0,1,2}  ×  {break 0,1,2}
```

thinned to 16 informative cells, plus a 4-cell named-`goto` family and 3
must-be-zero controls. **23 cells; 8 reproduce a cell `w-backedge` already
compiled** (`w-c0b0`≈`a-while`, `w-c1b1`≈`h-while-brk-cont`, `f-c0b0`≈`a-for`,
`f-c1b0`≈`h-continue`, `d-c0b0`≈`a-dowhile`, `g-back1`≈`a-goto-back`,
`z-none`≈`a-none`, `z-if3`≈`e-if3`) and are **reproduction controls, excluded
from every held-out score**. **15 cells have never been compiled by anyone.**

---

## §1 — the features, defined on the IL before anything is read

All from the `.ex` statement stream of the probe's own segment, cut at the
`4F 12 47` function tail, epilogue label excluded — `w-backedge`'s
`labelil.py::ex_cflow`, unchanged — plus one new `.sy` reading.

* **backward reference** — a `38`/`39`/`3A <tok>` whose `<tok>` was already
  defined by an earlier `29 <tok>`.
* **backward target** — a distinct `<tok>` with ≥ 1 backward reference.
  `uncond(t)` iff at least one of `t`'s backward references is a `3A`.
* **named(t)** — `t`'s `.sy` declaration record uses the **named** form
  `03 <k> <tok> 00 <name> 00 <b> <b>` rather than the 2-byte form. That is a
  source-level `goto` label. *(This assumes the `.sy` declaration token space
  and the `.ex` label token space coincide; **claim C0 registers that
  assumption and it may fail.**)*
* **break-jump** — a **forward** `3A` reference at stream index `i` to a target
  defined at index `d`, such that some **backward** reference sits at index `b`
  with `i < b < d`. In words: *an unconditional forward jump that leaps over a
  back edge*, which is what a `break` is and what an `if`'s forward jump is not.
  This is the discriminator `w-backedge` §4 named (*"the `break` is an
  unconditional forward `3A` to a label defined after the back edge"*) and it is
  what keeps `e-if2`/`e-if3`/`e-if4` — forward-only at 2, 3 and 4 labels,
  charging 0 — out of the break term.

## §2 — the models, frozen

```text
  R1     (w-backedge, frozen at 83943d3e)
             charge = 2·bwd_uncond_REFS + 1·bwd_cond_REFS

  R1′    (w-backedge found-and-not-taken #1 — THE SUBJECT OF THIS LANE)
             charge = Σ over backward TARGETS t of  c(t)
             c(t) = 1                if named(t)
                  = 2                else if uncond(t)
                  = 1                otherwise

  M-TGT  = R1′  +  1 per BREAK-JUMP                 (R1′ with w-backedge's
                                                     found-and-not-taken #2)
  M-TGT-L= R1′  +  1 per backward TARGET crossed by ≥1 break-jump
                                                    (break priced per LOOP)
  M-REF  = R1 with the named correction (a named backward target contributes
           1, not 2·refs)  +  1 per BREAK-JUMP      (the per-REFERENCE rival)
  M-REF-L= M-REF with the break priced per LOOP
```

`h-tern-loop`'s materialised signed relational is **+2** and is an orthogonal,
already-published surcharge (`LABEL_COUNTER.md` §1.1). grid3 contains **no**
relational-materialising cell, deliberately, so the term never enters. Stated
rather than silently excluded.

### Why the prior on R1′ is not high, and this is registered ahead of the run

`w-backedge`'s **own published grid2** already contains a cell that R1′ + a `+1`
break term gets wrong in the direction R1 gets right. Read off its `g2_o1.tsv`
(a fitting-set reading, not a hold-out one):

```text
                    bwd_t  bwd_u  bwd_c   real   R1   R1′   M-TGT   M-REF
  h-continue          1      2      0       2     4    2      2       4
  h-while-brk-cont    1      2      0       5     4    2      3       5
```

Two cells with an **identical recorded backward-feature vector** and charges 2
and 5. So *whatever* the per-target/per-reference answer is, **one of these two
cells refutes it**, and no rule built from `(bwd_targets, bwd_uncond,
bwd_cond, break-jumps)` alone can fit both unless the break term is +1 in one
place and +3 in the other. grid3's `w-c1b0` / `f-c1b0` / `w-c0b1` / `f-c0b1`
cross is built precisely to say which half is the anomaly. **This is why R1′ is
registered at P = 0.30 and not at 0.7.**

## §3 — claims, with probabilities. These are CEILINGS with no discount factor

| # | claim | P |
|---|---|---:|
| **C0** | the `.sy` declaration token space and the `.ex` label token space coincide, so `named(t)` is readable at all — the named form is found for `g-back1`'s `top` and its token equals a backward target's | 0.60 |
| **C1** | instrument: 0 controls failed, anchors equal on ≥ 21 of 23 cells | 0.80 |
| **C2** | the count of **discriminating** cells (`bwd_refs > bwd_targets`) on grid3 is **≥ 5**. *A count of 0 is a LOUD FAILURE of the grid, not a pass of any rule* | 0.85 |
| **C3** | the 8 reproduction controls reproduce `w-backedge`'s charge exactly, 8 of 8 | 0.90 |
| **C4** | **R1′ (bare) scores ≥ 13 of 15 on grid3's no-break cells** | 0.30 |
| **C5** | R1′ (bare) **strictly beats** R1 on grid3's no-break cells | 0.55 |
| **C6** | `break` is exactly **+1** per break-jump on every break cell of at least one of the four break-bearing models | 0.45 |
| **C7** | **M-REF strictly beats M-TGT** over all 23 grid3 cells (i.e. per-REFERENCE wins and `w-backedge`'s correction #1 is BACKWARDS) | 0.45 |
| **C8** | the break term is per **statement**, not per **loop** — the `b2` cells separate them and the per-statement variant wins | 0.50 |
| **C9** | **no** model in §2 is exact on all 23 grid3 cells | 0.70 |
| **C10** | the mutation control goes red on **≥ 5 discriminating constructed cells** graded against reference-obj bytes, and the discriminating count is printed | 0.85 |
| **C11** | the `ptr_walk_loop` class's charge **differs between `/O1` and `/Ox`**, so a mode-free `Some(k)` cannot be right at both | 0.55 |
| **C12** | fence B's cost of keeping is **exactly 1 tracked fixture and 0 workload TUs**, re-measured rather than quoted | 0.75 |
| **C13** | **the lift ships and `whash_loop_then_framed.cpp` converts** | 0.35 |

## §4 — the registered outcome numbers

Base, at `ac3cdd8c`:

```text
  cargo test --workspace --release      1581 passed, 0 failed, 42 targets
  fixture port gate (c2rs perf)         150 Match, 0 mismatch, 231 not-implemented (of 381)
  878-TU workload scan                  match 25, mismatch 0, codegen-gap 0,
                                        vocab-gap 845, capture-fail 8
```

| quantity | registered prediction |
|---|---|
| `mismatch`, everywhere, every lane | **0**. This outranks every other line in this table, including C13 |
| workspace tests | **1581 → 1589, 42 targets** (a ceiling: ≤ 8 new unit tests, no target added) |
| fixture port gate `Match` | **150 → 151** if C13 holds, **150** if not |
| 878-TU workload `match` | **25 → 25**. `whash_loop_then_framed.cpp` is a *fixture*, not a workload TU. The brief's *"26th TU match"* is the **fixture-gate** number; the workload number does **not** move and this lane will not claim it does |
| `fnbyte-exact` | **+0** |
| census | **+0** — no new emit class |

## §5 — the falsifiers, named before the run

1. **R1′ is falsified** if any grid3 cell with `bwd_refs > bwd_targets` and no
   break-jump has a charge that per-reference predicts and per-target does not.
2. **The break term is falsified as a constant** if `w-c0b1` and `f-c0b1` — one
   break each, different loop kind — have different residuals over the same
   base model.
3. **The whole feature vector is falsified** (C4 falsified again, this time on
   grid3) if two grid3 cells share `(bwd_targets, bwd_uncond, bwd_cond,
   break-jumps, named)` and differ in charge.
4. **`named(t)` is falsified** if `g-sep2` (two independent named back edges)
   does not charge exactly twice `g-back1`.
5. **The lift is refused** if C11 holds — a mode-dependent charge with no mode
   parameter on `label_slots` is `counted_accum_loop`'s reading 2 verbatim, and
   it is sufficient on its own.

## §6 — the control that can go red, and the mutation

`w-ir-e`'s standard: a mutant that reddens a test built from **reference-obj
bytes**. `w-backedge`'s `mchan.py` is that control and it is reused unchanged in
construction: a two-function TU `[P (the loop leaf), F (framed)]`, so the charge
lands on `F`'s `$M` — **six bytes of the reference obj's symbol table** — and
every cell is scored against those bytes.

Mutants, and **the discriminating count is printed for each** so a vacuous
control is a loud failure (`w-backedge` printed 11, 11 and 9; matching that is
C10):

```text
  M0   the model under test          expect GREEN on most
  M1   charge + 1                    expect red
  M2   charge − 1                    expect red
  M3   charge = 0                    what `coff::plan_labels` charges TODAY —
                                     this is the shipped port's model and it
                                     MUST go red, on every cell with a charge
  M4   R1                            the previous lane's model
```

## §7 — the two-sided price of fence B, and what would license lifting it

`CLAUDE.md`: a fence is priced two-sided before it ships, and #3062's standing
counter reads `sole` 0 / `exact` 0 across all 23 causes, so the price is argued
from **constructed cells**, never read off the workload.

**Fence B** is `IlFunction::label_slots` returning `None` for the loop shapes.
The lift's mechanism is already precedented and is registered here **before it
is attempted**, so that a later reading cannot claim it was discovered by trying
things: `IlFunction::label_lead()` already carries a **non-zero lead for a LEAF
class** — `xtea_round_loop`'s `+2` — and `coff::plan_labels` charges a non-framed
function `label_lead + 1`, so **no arm in `plan_labels` changes and `coff/` is
not entered**. `IlBundle::functions`' gate is `label_slots(false)? != label_lead()
+ 1`, which a fall-through `Some(label_lead() + 1)` satisfies by construction.

The lift ships **only if all four hold**:

1. a model survives grid3, a hold-out this lane did not fit to;
2. that model is exact on `whash_loop_then_framed.cpp`'s own obj bytes;
3. the charge is the **same integer at every gate lane the class is reachable
   at** (C11 falsifies this and is on its own sufficient to refuse);
4. `mismatch` is 0 at every one of the 18 lanes.

Otherwise the outcome is **`declined`**, the price is the deliverable, and that
is a good outcome — `w-pool`→`w-pool2` and `w-xtea`→`xtea2`→`xtea3` are the
precedent that a priced decline feeds the conversion that follows it.

## §8 — corrections owed in place (#3091)

`fixtures/cpp/whash_loop_then_framed.cpp`'s header says the shape charges **+3**
and that the triple would come out *three* low; `IlFunction::label_slots`' doc
says **+3**. That fixture's own obj says the lead is **2**. Both are corrected
in this lane whatever its outcome, because the error runs in the direction that
makes the fence look **dearer to lift than it is**.
