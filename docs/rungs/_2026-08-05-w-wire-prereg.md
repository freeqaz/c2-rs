# PREREG — lane `w-wire`: wire the store-run models into the EMITTER

Fixtures: none — this is the prereg. The rung doc that scores it
(`2026-08-05-w-wire.md`) carries the fixtures.

Registered at master `2ed0030`, before any line under `crates/` was changed.
Board items taken: **#640**, **#641**, **#642**, **#643**.

---

## §1 The observation

Six lanes have modelled the store-run emission floor and **every one shipped as
a guard that REFUSES**:

| model | locator | holdout |
|---|---|---|
| SCHED | `codegen::schedule` | 105/105 |
| ALLOC | `codegen::alloc::allocate` | 250/250 |
| ORDER | `codegen::order::store_order` | 561/561 |
| SYMORDER | `codegen::order::store_order`, multi-symbol | 1501/1501 |
| LAYOUT (#602) | `codegen::order::layout_slots` | 24,891/24,891 |
| PIN (#601) | `codegen::order::walk`'s clause | 7,589/7,589, model-free |

`codegen::order::schedule` — the function that assembles all of them into an
emitted sequence — **has no caller under `crates/`**. `leaf::store` emits
source order and calls `producers_lead` only to decline.

**This lane makes the emitter USE them.**

## §2 The hypotheses, with the predictions registered before the run

### H1 — `/O1` and `/Ox` agree on the store-run models · board #641

*Registered in `work/w-wire/mode_probe.py`'s docstring before the run.*

**Every prior grid was compiled at the WORKLOAD's flags**, `/O1 /Oi /EHsc`
(`work/w-alloc/alloc_lib.py:26`, `work/w-order2/`, `docs/ALLOC.md:231`,
`docs/ORDER.md:313`). The **fixture gate runs `/Ox`**, and `leaf::store`
already carries a mode split for the *load*-valued arm (`gpr_scratch`). Nobody
has ever asked whether ALLOC/ORDER hold at `/Ox`.

**Prediction: they agree on ≥ 15 of the 18 probe cases.** If they do not, the
widening is gated on `OptMode::O1` and the rung says so.

**RESULT — 18 of 18 SAME**, model-free (the two modes' emitted permutations
compared to each other, never to the model). Recorded in §5 below. **No mode
gate is needed.**

### H2 — the models predict every one of those 18 cases

Hand-derivation only; no code consulted the model when the probe ran.

**Prediction: ≥ 16 of 18.** **RESULT: 18 of 18**, worked through in §5.

### H3 — the emitter rewrite is INERT on the class the parser admits today

The new path replaces `hoisted_lit` + source order with `order::schedule` +
`alloc::allocate` for **value-simple GPR runs** (every group a
`[Load(base), Lit|Load(formal), StoreInd]`). On what the parser admits today —
an all-formal run, and an all-**same**-literal run — this must produce
**byte-identical** text.

**Prediction:** workspace tests green; `gate.sh` PASS; the 878-TU scan
identical in every gap-metric; **FBM unchanged to 5 decimal places**.

*Falsifiable in a second way, and this is the one that matters:* `schedule`
must never return `None` on a run the parser admits, or the rewrite converts a
live accept into a refusal. **Proved by enumeration in a test**, not by
argument.

### H4 — the parser widening converts functions, not TUs · board #642

Two widenings of `try_parse_store_run`, each behind a **syntactic** gate that
is strictly inside every model's exact region:

* **W1** — a run of **≥ 2 distinct literal values**.
* **W2** — a run that **mixes** literal values with formal ones.

Both gated on: single base token · ≤ 3 distinct literals ·
`3 + params ≤ 11` and pool ≥ #distinct. With one base token every producer's
`nsw` is identically **0**, so `MAX_SYMBOL_CROSSINGS` is satisfied by
construction rather than by check — the domain gate #621 refused to weaken.

**Predictions (the numbers this lane is scored on):**

| metric | baseline at `2ed0030` | predicted after |
|---|---:|---|
| per-function census | 706,557 | **+8,000 … +30,000** |
| emitted-function census | 38,460 | **+400 … +3,000** |
| **FBM** | **0.16254** | **+0.002 … +0.015** |
| **`fnbyte-differs`** | **0** | **0 — this is the ALARM** |
| `fnbyte-partial` | 9,375 | +0 … +500 |
| **TU match** | **9** | **9** |
| `mismatch` | 0 | 0 |
| `codegen-gap` | 0 | 0 |
| `vocab-gap` | 862 | 862 |
| census/gate disagreement | 0 | **0** |
| factor A/B/C/D/E | 28/338/169/9/2 | unchanged except **D 9 → 9…12** |
| FRONTIER | 18 | 18 |

**TU match is predicted to NOT move, and that is the honest prediction.**
`xboxheap` is blocked on a parse chain ≥ 2 operators deep (#622); the frontier
is priced at ≥ 6 independent refusals per TU (#269). Six lanes' honest zeros
are why these models are trustworthy, and a seventh is the expected outcome.
**Expect the function number to move and the TU number not to.**

### H5 — the correction to this lane's own brief

The brief states baseline **FBM 0.16251, 29,084 exact, 9,374 partial**. Measured
at `2ed0030`: **0.16254, 29,085 exact, 9,375 partial** — one behind, matching
`docs/STATUS.md`'s generated block. **The brief's figures are superseded, and
all deltas below are against the measured baseline.**

## §3 The invariant this diff must satisfy

> *A change in a model's answer can add a refusal, but can never turn a refusal
> into an accept, outside the region that model is exact on.*

Stated per-change, because they are **not** the same claim:

* **The emitter rewrite (#640) is additive-REFUSAL.** `schedule`/`allocate`
  returning `None` becomes `out_of_class`. It cannot accept anything the parser
  did not already hand it.
* **The parser widening (#642) is additive-ACCEPT, and it is said plainly
  rather than blurred into the guard's property** — the same distinction
  `w-frame2` drew for `schedule`. It admits runs that previously refused. Its
  safety does **not** come from the guards; it comes from the syntactic gate
  landing strictly inside a region measured exact against real `c2` bytes, plus
  constructed counterexamples at each boundary (§4).

## §4 Constructed counterexamples — built to FAIL, ahead of shipping

Each is a cell one step outside the gate, compiled by real `c2` and required to
be **refused** rather than answered:

1. **4 distinct literals** — `{a=1;b=2;c=3;d=4}`. Outside
   `MAX_MODELLED_PRODUCERS`; c2 reuses a freed register (#541).
2. **Two base symbols with 2 distinct literals** — `{s->a=1; t->b=2;}`. Inside
   `MAX_MULTISYM_PRODUCERS` but **outside this lane's parser gate**; must refuse.
3. **The pool boundary** — 8 formals, 2 distinct literals. `pool_floor` is 11,
   one register free, two wanted.
4. **A wide literal beside a narrow one** — `{a=100000; b=1;}`; `lis`+`ori` is
   two words for one producer and the layout indexes producers, not words.
   *Registered prediction: NOT a boundary — this is in domain and must be
   answered with the pair kept whole.*
5. **`x_split`'s mask** (`nsw = 3`) — already refused by `layout_slots`; assert
   the parser gate refuses it *first*, so the refusal does not depend on the
   model being consulted.

### §4.1 RESULT — counterexample 4 FIRED, and prediction 4 above is WRONG

**Corrected on the page rather than replaced.** The prediction in item 4 is a
**refutation of a premise this lane was given**, and it is the single most
valuable result here. `work/w-wire/boundary_probe.py`, real `c2`, identical at
`/O1` and `/Ox`:

```text
  { a=100000; b=1;      }   lis r11 ; li r10 ; ori r11 ; stw r10,4(r3) ; stw r11,0(r3)
  { a=100000; b=200000; }   lis r11 ; lis r10 ; ori r11 ; ori r10 ; stw r11,0 ; stw r10,4
```

Two independent failures at once:

1. **A producer is not one contiguous instruction.** c2 *interleaves* the
   halves of two wide loads (`lis lis ori ori`), so `layout_slots` — which
   places producers by index — cannot express the sequence at all.
2. **`store_order` is REFUTED on the first cell**: c2 emits stores `[1, 0]`
   where the model says source order. Every ORDER/ALLOC grid used single-word
   `li` values, so a **two-word producer is outside the population the models
   were measured on** — and nothing in `docs/ORDER.md` or `docs/ALLOC.md` says
   so, because nobody had asked.

Had the widening shipped without this probe it would have been a **live wrong
emit** — board #232's exact shape. `scheduled_gpr_run_text` now refuses any run
with **more than one producer** where any literal needs more than one word;
a run whose *only* producer is wide is unaffected and stays in class, which the
test asserts in the same breath so the gate cannot over-refuse.

### §4.2 RESULT — the `/Ox` agreement is a property of the DOMAIN, not of store runs

H1 read **18 of 18** inside the modelled region. The boundary probe compiled
`{a=1;b=2;c=3;d=4}` — four producers, board #541 — and the two modes
**DISAGREE**:

```text
  /O1   li r11 ; li r10 ; stw r11,0 ; li r9 ; li r11 ; stw r10,4 ; stw r9,8 ; stw r11,12
  /Ox   li r11 ; li r10 ; li r9 ; stw r11,0 ; li r8 ; stw r10,4 ; stw r9,8 ; stw r8,12
```

`/O1` **reuses r11** after its store frees it; `/Ox` takes a fresh **r8**. So
H1's headline must be quoted with its scope: *the modes agree everywhere the
port emits, and are known to differ one step outside it.* That is a stronger
reason to keep `MAX_MODELLED_PRODUCERS = 3` than the one #541 recorded.

The pool boundary (item 3) is a measured regime too: with 8 formals c2 emits
`li r11 ; li r10 ; …`, **reusing r10 — a formal's own register** — which is the
liveness model `docs/ALLOC.md` names as open. The port refuses.

## §5 H1/H2 result — the mode probe, model-free

`python3 work/w-wire/mode_probe.py`, 18 cases, `/O1` vs `/Ox`:

```
MODE AGREEMENT: 18 of 18 cases
```

Selected cells, with the model's hand-derived answer beside the observation:

| case | source | observed (both modes) | ORDER/ALLOC/LAYOUT |
|---|---|---|---|
| `A5` | `1,2,1,2` | `li r10 ; li r11 ; S0@r10 S4@r11 S8@r10 S12@r11` | source order; constants tie count-2 → **REVERSE** first-use |
| `A6` | `1,1,2,2,2` | `li r11 ; li r10 ; S8@r11 S0@r10 S4@r10 S12@r11 S16@r11` | store order **[2,0,1,3,4]**, not source |
| `A8` | `1,2,2` | `li r11 ; li r10 ; S4@r11 S0@r10 S8@r11` | store order **[1,0,2]** |
| `M5` | `1,2,u,v` | `li r11 ; S8@r4 ; li r10 ; S12@r5 ; S0@r11 S4@r10` | `check("01..", "P0 S2 P1 S3 S0 S1")` |
| `M7` | `u,1,2,v` | `li r11 ; S0@r4 ; li r10 ; S12@r5 ; S4@r11 S8@r10` | order `[0,3,1,2]`, `u=2`, producers at slots 0,1 |

`M5`/`M7` are the load-bearing ones: the producers are **interleaved** with the
stores, which is `layout_slots` and nothing else, and the store order is a
permutation no source-order emitter can reach.

## §6 An incidental finding · board #643

`/O1` implies function-level linking on this compiler and **`/Ox` does not**:
the same probe TU comes back `PROC NEAR ; X, COMDAT` at `/O1` and
`PROC NEAR ; X` at `/Ox`. Every lane's listing parser requires the `, COMDAT`
suffix (`work/w-alloc/alloc_lib.py:63`), so **any `/Ox` listing parses to zero
functions** in the inherited tooling. Caught here only because `compile_cod`
raises on `0 PROC` — the "absence reads as success" mitigation (STATUS trap 5)
firing exactly as designed. `work/w-wire/wirelib.py` makes the suffix optional.

## §7 Method

One model per commit, gated on the full suite at each step, so a red result
names the commit. `scripts/gate.sh --jobs 6` expected **PASS 18/18, 4,500
verdicts** — registered here **before** the run, and it grows by exactly 18 per
fixture added.
