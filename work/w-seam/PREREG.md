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
