# w-seam — PREREG

    Lane:      w-seam, worktree off master `ec535b0`
    Board:     #866–#875 reserved for this lane
    Subject:   board **#844** — `codegen::leaf::store` is leaf-only by
               construction, and the frontier TU `src/xdk/nuispeech/xboxheap.cpp`
               needs its scheduled GPR store run in the MIDDLE of a framed body.
    Committed: BEFORE any probe file exists. Every grid below is declared here
               by partition and by verdict rule; the scripts are written after
               this file lands.

---

## 0. The baseline, read off a scan taken before this file was written

`work/w-seam/baseline_scan.txt`, 878 TUs at the workload's own flags:

```text
  match 10 · mismatch 0 · codegen-gap 0 · vocab-gap 861 · port-error 0 · capture-fail 7
  FRONTIER 17 · A 28 (LO 27) · B 338 · C 169 · D 10 · E 2
  B∧C 151 · A∧B∧C 27 · frontier-if-A 139 · emit-predicate-worth 124
  FBM 0.16654 · fnbyte-exact 29802 · fnbyte-partial 9375 · **fnbyte-differs 0**
  62 `gap-metric` lines, frozen at `work/w-seam/baseline_metrics.txt`
```

That file is the identity check the end of the lane is compared against, by
`diff`, not by reading a summary.

---

## 1. What this lane is asked, and what it can lose

The mission has three separable questions and they are registered separately so
that a hit on one cannot launder a miss on another.

1. **TRANSFER** — do `order::schedule` and `alloc::allocate`, both fitted
   entirely on **leaf** bodies, still hold when the same store run sits inside a
   **framed** body with one call?
2. **BRACKET** — where does the `mr r31,r3` copy sit relative to the run, and is
   its slot derivable from anything the port already models?
3. **THE LIFT** — may `codegen::alloc`'s refusal of a constant/register-derived
   mixed run be narrowed for the sub-case *clause 1 decides with no tie*?

---

## 2. GRID T — the transfer grid

**Declared before the file exists.** One function per TU, compiled through
`work/w-frame/refobj.sh` at the workload's own `/O1 /Oi /EHsc /GR` flags and
disassembled with `scripts/gt_dump.py`.

Each **configuration** is a store run. Each configuration is emitted in **four
body kinds**:

| kind | source | what it adds |
|---|---|---|
| **L** | `void f(S* s, int u, int v){ <run> }` | the leaf — the control |
| **P** | `void f(S* s, int u, int v){ <run> gx(); }` | a frame + a trailing call |
| **Q** | `void f(S* s, int u, int v){ gx(); <run> }` | the run AFTER the call |
| **R** | `S* f(S* s, int u, int v){ <run> gx(); return s; }` | `this` live across the call — the `mr r31,r3` bracket |

The twelve configurations, fixed here:

```text
  C1  two FORMAL stores, no producer at all       s->f0=u; s->f1=v;
  C2  one constant, 1 use                          s->f0=7;
  C3  one constant, 2 uses                         s->f0=7; s->f1=7;
  C4  two constants, 2 and 1 uses                  7,7 then 9
  C5  three constants, 1 use each                  7, 9, 11
  C6  the XBOXHEAP configuration: (int)&q at 2 uses beside li 0 at 1 use
  C7  mixed, reg 1 use  vs const 1 use
  C8  mixed, reg 1 use  vs const 3 uses
  C9  formal + constant mixed run                  s->f0=u; s->f1=7; s->f2=7;
  C10 one constant, 4 uses                         a long pure run
  C11 two constants, source-interleaved            7, 9, 7, 9
  C12 register-derived only, (int)&q at 3 uses, no constant
```

**The verdict rule, fixed here.** For each configuration, take the disassembled
instruction list of each kind and **strip**: the prologue words
(`mflr`/`stw r12,-8(r1)`/`std r31,…`/`stwu`), the epilogue words
(`addi r1,r1,F`/`lwz r12,-8(r1)`/`mtlr`/`ld r31,…`/`blr`), the `bl`, and any
`mr r31,r3` / `mr r3,r31`. The residue is the **run text**. The configuration
**TRANSFERS** iff the run text of P, Q and R is *string-identical* to L's.

**Counters are separate and all printed**: selected / reached / graded /
transfer / no-transfer / out-of-regime. A cell that fails to compile, or whose
strip does not consume exactly the expected frame words, is **out of regime and
is never scored as a transfer** (STATUS trap 5).

### Registered predictions

* **T1 — P transfers on every configuration.** A trailing void call with no
  argument cannot reach back into a run that precedes it.
  *This can lose*: if c2 sinks or hoists across the `bl`, or re-allocates the
  pool because r11/r10 are volatile across a call, P will differ from L.
* **T2 — Q transfers on every configuration.** Registered deliberately as the
  weaker of the two: after a call the formals `u`, `v` are dead, so a run that
  consumes them cannot be the same body at all. **T2 is expected to be OUT OF
  REGIME on every configuration that stores a formal (C1, C9), and to transfer
  on the rest.** If it transfers on C1/C9 as well, the prediction is wrong in
  the direction of the port's favour and is recorded as such.
* **T3 — R transfers.** The `mr r31,r3` is registered as **additive**: it is
  inserted into the run without moving any other word.
  *This can lose, and it is the claim I most expect to lose*: `xboxheap`'s
  `mr 31,3` sits at slot 5 of a 9-word run, which is not the top and not the
  bottom, so its insertion point is already known to be interior; whether the
  words around it keep the leaf's order is exactly the open question.
* **T4 — C6, C7, C8 and C12 will be graded, not refused.** The allocator's
  refusal is the *port's*; real `c2` emits these and the grid reads its bytes.

---

## 3. GRID M — the `mr r31,r3` bracket

**Declared before the file exists.** R-kind bodies only, sweeping run length
2…8 and producer count 0…3, plus two cells where the returned pointer is a
*second* formal rather than `s`. For each, record the **index of `mr r31,r3`
within the run text**, counted in instructions from the start of the run.

### Registered hypothesis, which I expect to LOSE

* **H-mr — the `mr r31,r3` is scheduled as if it were a producer.** Concretely:
  feed `order::schedule` a statement list with one extra producer whose use
  count is 1 and whose first use is the last store, and the slot it lands in is
  the observed slot.
  **Registered prediction: H-mr is REFUTED.** `xboxheap`'s slot 5 of 9 does not
  sit at any `layout_slots` position for a 1-use producer, so the rule is
  registered as expected-to-fail and the deliverable is the *observed* slot
  table, not a fitted rule.
* **M2 — the slot is NOT constant** across run length. If it is constant, H-mr
  is doubly dead and the answer is a much simpler one.

**If H-mr is refuted and no successor rule is exact on every graded cell, no
`mr r31,r3` emitter ships.** A run that cannot place the copy is a refusal.

---

## 4. GRID A — the narrow allocation lift

**This grid is frozen to disk and committed BEFORE it is compiled.** That is a
condition of the lane, not a courtesy.

The sub-case, stated exactly: **two producers, one register-derived and one
single-word constant, with `reg.uses > const.uses` strictly** — so `alloc.rs`'s
clause 1 (use count, descending) decides with **no tie**, and no tie-break
clause, no kind bonus and no refuted key is consulted.

Cells: three register-derived **spellings** × six use-count gaps × two body
kinds.

```text
  spellings   addi-interior   (int)&q          — xboxheap's own spelling
              add             (u + v)
              slwi            (u << 3)
  gaps        (reg,const) = (2,1) (3,1) (3,2) (4,1) (4,2) (4,3)
  kinds       L (leaf)  ·  P (framed, run before a trailing void call)
  = 3 x 6 x 2 = 36 cells
```

**The three spellings are in the grid on purpose and may not be dropped.**
`w-alloc2`'s `F4-shift-r2k1` is already on record as a `(2,1)` cell where the
**constant** takes `r11` — i.e. clause 1 refuted at a strict use-count
advantage. Building this grid without `slwi` would be fitting the lift to the
spelling that survives.

### Registered prediction

* **A1 — THE LIFT FAILS.** At least one cell disagrees with *"the producer with
  strictly more uses takes `r11`"*, and the disagreement is a `slwi` cell.
  Consequently **`codegen::alloc`'s mixed refusal STANDS and `xboxheap` does not
  convert in this lane.** If A1 is wrong — if all 36 cells agree — the lift is
  still only licensed for the sub-case above, is shipped with must-fail
  mutations graded on real bytes, and is stated as fitted on 36 cells.
* **A2 — the spelling axis, not the use-count axis, is what separates.** The
  `addi-interior` and `add` rows agree with clause 1 on every gap; the `slwi`
  row does not.

**Standing instruction taken as binding**: if any cell disagrees, the refusal
stands, `xboxheap` does not convert, that is recorded, and **nothing is
patched**. Neither `w-next`'s key nor `w-alloc2`'s `H-self` is shipped under any
outcome.

---

## 5. What may ship, and under what condition

**S1 — the seam ships only if T1 (and T3, if `mr` is in it) holds on every
graded cell of GRID T.** The seam is a framed-body emission path that composes
three emitters that already exist and are already graded — `codegen::frame`'s
prologue/epilogue, `scheduled_gpr_run_text`'s plan, and `calls::call_seq_text`'s
call block — and it emits **byte-exact or `NotImplemented`**, never wrong bytes.
Reachability needs a parser production as well (a `CallSeq` with a leading
value-simple GPR store run); if the parser half does not land, the emitter half
does not ship either, because an unreachable emitter moves no byte and is not
worth the refusal surface.

**S2 — `xboxheap` does not convert.** Registered as a prediction that can lose.
`w-alloc2` prices it at 16 independent facts, of which 6 are IL-side (four
opcode steps `0x27`/`0x32`/`0x4B`/`0x4F` and a composite `-more` terminal that
is itself a lower bound), and this lane addresses at most the emit-side
composition. **TU match is predicted 10 → 10.**

**S3 — `fnbyte-differs` stays 0 and `mismatch` stays 0**, at both ends, printed
from a scan and not asserted.

**S4 — if anything ships, it ships with must-fail mutations graded on real
`c2.dll` bytes**, and each mutation's result is printed as RED with a count.
`w-alloc2` §5.1 is taken as binding: **a green 878-TU scan is not evidence about
any rule this lane measures**, because a deliberately wrong allocation key
produced 62 byte-identical `gap-metric` lines. No claim in this lane's rung may
cite the gate as support for a schedule or allocation rule.

---

## 6. The named risk

**#232's shape**: a widening that turns a clean refusal into a wrong emit, live
under a green gate. Every accept path this lane could add is an *additive
accept* and therefore carries it. The mitigation is that every accept is graded
against real `c2` on cells chosen before the answer was known, and that the
default on any disagreement is the refusal. **Zero conversions is a better
outcome than one wrong emit.**

---

## 7. Scoring

Every row above is scored in `docs/rungs/2026-08-06-w-seam.md` §2 as HIT /
PARTIAL / MISS / UNSCORED, with the measured number beside the registered one,
including the rows that lose.

---

## ADDENDUM 1 — 2026-08-06, after GRID T and BEFORE GRID M and GRID A exist

GRID T is compiled and scored (`work/w-seam/gridt.out`): **60 selected / 60
reached / 60 GRADED / 0 out-of-regime**, 48 IDENT (12 of them the leaf
controls), 6 PLAN-only, 6 DIFFER. Two registered rows moved, and both moves are
recorded here before the next grid is written.

### A1.1 — T1's cells are NOT framed, and the instrument is what says so

Every **P** cell (`<run> gx();`) came back with a **frame word count of 1** —
one `b`, no `mflr`, no `stwu`. c2 **tail-calls** a lone trailing void call even
behind a store run, so `P` measures a *tail-call* body, not a framed one. T1 is
therefore rescored against what it actually compiled, and **P2** (two trailing
calls, frame count 9) is the row that carries the framed claim. This was caught
only because the frame-word count is printed beside every verdict rather than
asserted.

### A1.2 — a NEW rule fell out of the R cells, and it is FITTED, so it gets a fresh grid

The registered hypothesis **H-mr** ("the `mr r31,r3` is scheduled as if it were
one more producer, placed by `layout_slots`") is **REFUTED** — `layout_slots`
puts producer `i` before store slot `min(i, u)`, which mispredicts `C1` and
`C4`. The twelve observed slots instead fit

> ### **`stores_before_mr = nprod − 1 + u`**

where `nprod` is the number of distinct producers in the run and `u` is the
**leading run of unproduced stores in the FINAL emitted order** (`layout_slots`'
own `u`, board #584) — both read off the same disassembly, so the rule is
stated in observables and not in a model. It fits all twelve GRID T `R` cells
**and `xboxheap.cpp`'s own body** (`nprod` 2, `u` 2 → 3 stores before the
`mr 31,3`, which is what the obj shows).

**It is fitted on the cells that produced it**, which is exactly how `P3`,
w-next's key and w-alloc2's `H-self` were born. It is therefore registered here
as a candidate, frozen, and taken to a **fresh holdout (GRID M)** that varies
`u` and `nprod` independently over cells GRID T does not contain.

* **M1 — the rule `stores_before_mr = nprod − 1 + u` HOLDS on every graded
  fresh cell.** *This can lose, and losing is the deliverable if it does.*
* **M2 — the rule is not shipped in this lane under either outcome.** No
  `mr r31,r3` emitter is written: the shape it would serve (`this` live across
  a call) needs the mixed-kind allocation GRID A is registered to leave
  refused, so an emitter for it would be unreachable. GRID M is measurement.

GRID M's cells: `R`-kind bodies only, `nprod ∈ {0,1,2,3}` crossed with a leading
run of **unproduced** stores of length `{0,1,2,3}` (formal `u`, formal `v` and
`(int)s` supply three distinct unproduced values), total stores 2…8. `u` is
**measured off the emitted order**, never assumed from the source order.

### A1.3 — T3 HOLDS where it was registered to lose

T3 ("the `mr r31,r3` is additive — it is inserted without moving any other
word") was registered as *the claim I most expect to lose*. It is **12 of 12
IDENT**: every `R` cell's run text is string-identical to its leaf's. Recorded
here, before GRID A, so the scoring cannot be read as retrofitted.

### A1.4 — GRID A is unchanged and is compiled after this addendum lands

The 36 cells of §4 are written and committed before a single one is compiled.
Prediction **A1 (the lift FAILS, and the disagreement is a `slwi` cell)** and
**A2 (the spelling axis separates, not the use-count axis)** stand as written.

---

## ADDENDUM 2 — 2026-08-06, after GRID M and GRID A, before GRID T2 and GRID M2

### A2.1 — GRID A is scored, and it is a HIT in its strongest form

**36 selected / 36 reached / 36 GRADED / 0 out-of-regime**, 24 hit, **12 MISS**.

```text
  addi-interior  12 / 0 / 0
  add            12 / 0 / 0
  slwi            0 / 12 / 0
```

Every miss is a `slwi` cell, which is **A2** exactly as registered, and `slwi`
loses at a use-count advantage of **three** (reg 4 uses against const 1) as
flatly as at one. There is therefore no threshold the lift could be narrowed
around: for that spelling the register-derived producer never takes `r11`
against a constant in this configuration. Both body kinds agree cell for cell,
so the framed context does not rescue it either.

> ### **`codegen::alloc`'s mixed refusal STANDS. `xboxheap.cpp` does not convert in this lane. Nothing is patched.** That is the standing instruction and it is taken as binding.

### A2.2 — GRID M is scored: the fitted rule dies on its own holdout, and the miss names the observable

**24 selected / 24 reached / 24 GRADED / 0 out-of-regime**, 19 hit, **5 MISS**.
Every miss has a leading unproduced run of **3** and is off by **exactly one**,
so the *shape* was right and the *observable* was wrong: `store_order`'s own
`u` is `min(2, #unproduced)`, and the raw leading run is not it. The corrected
statement is

> ### **`stores_before_mr = nprod − 1 + min(u, 2)`**

It is **fitted on GRID M's five misses** and is therefore taken to a second
fresh holdout (**GRID M2**) before it is quoted anywhere. **M2 of Addendum 1
still binds: no `mr r31,r3` emitter ships under either outcome.**

GRID M2's cells, declared here: leading unproduced runs of **4, 5 and 6** —
strictly outside every cell GRID M contains — crossed with `nprod` 0…3, plus
runs where the unproduced stores are **not** first in source (so the emitted
leading run and the source leading run differ), plus two multi-width runs.

### A2.3 — GRID T2, a fresh transfer holdout

T1/T3 were registered before GRID T compiled, so 12/12 is a holdout result and
not a fit. GRID T2 widens it anyway, on axes GRID T holds fixed:

```text
  more FORMALS (pool floor)      f(S*,int,int,int,int,int)
  MIXED WIDTHS                   char / short / int / long long stores
  TWO BASE SYMBOLS               s and a second pointer formal
  a WIDE literal (board #644)    lis/ori, whose halves split
  a run of SEVEN                 past every fitted length
  a call WITH an argument        gx(u) rather than gx()
  a NON-VOID trailing call       int r = gx(); s->f0 = 7;   (run before, result after)
```

crossed with the three kinds that carried the claim — **L**, **P2** and **R**.
Registered prediction **T5: every graded configuration still transfers at the
IDENT level**, with the `gx(u)` and non-void rows expected to be the ones that
can lose, because an argument setup competes for the same scratch pool the run
allocates from.

### A2.4 — WHY NO SEAM IS SHIPPED, decided here and not after the fact

Two blockers, both measured, and the second is the one that decides it.

1. **The allocation half is refused** (§A2.1), so `xboxheap`'s own
   configuration cannot be emitted regardless of how the frame composes.
2. **Every composition board #844 names is INVISIBLE to `fnbyte-differs`.**
   `crates/c2-harness/src/gap/fnbytes.rs` maps `Selected::{Tail, Framed, Seq,
   CondPair}` to `FnByte::Partial` by construction — a body whose missing words
   encode their own `.text` offset is one the harness *must not* reconstruct —
   and the baseline scan prints the population that already sits there:
   **`partial by shape: tail 7098 · seq 2150 · framed 123 · cond-pair 4`**, 9,375
   emitted functions accepted by the port and graded by nothing.

   A store-run-before-a-call seam adds its whole population to that bucket. The
   standing alarm this lane is instructed to watch would read **0 whether the
   seam were right or wrong**, which is board #232's shape with the alarm
   removed rather than merely unwatched. `fnbytes.rs`' own module doc already
   says the reconstruction belongs in `c2-core` behind a per-function entry
   point — **board #322** — and declines to do it in the harness because that is
   the one lever that inflates FBM without moving the port.

   > ### So **#322 is a PREREQUISITE of #844, not a neighbour of it.** Until a per-function entry point exists in `c2-core`, any seam #844 asks for is an accept path with no standing per-function grader, and this lane declines to open one.

**What ships instead**: the measurements, and `crates/` **tests only** — pinning
the transfer result and the leaf-only construction so a lane that widens this
has to come here and state what it measured. **Zero emitted bytes change**, and
that is verified by a scan rather than argued.
