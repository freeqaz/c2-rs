# w-itemf-price — item F priced: 7 steps, 17 lanes, and a buy of ZERO on every population the goal is written in

    Tag:       w-itemf-price
    Slug:      itemfprice
    Date:      2026-08-15
    Kind:      characterization
    Outcome:   built
    Fixtures:  none — characterization: what does CFG_SHAPE.md §6.2 item F cost, decomposed into steps a construct rung could take, each with its own fail-closed boundary, and what does it buy in each named population?
    Census:    unchanged → unchanged, +0
    Record:    docs/whitebox/WB_ITEMF_FINDINGS.md; prereg frozen in
               docs/whitebox/WB_ITEMF_PREREG.md at `5fe20768`, BEFORE the first
               grep of the export and before the first measurement of anything
               in this repository. Board rows drafted UNNUMBERED and then
               **MINTED `#3165`–`#3170`** on the coordinator's instruction;
               next free **`#3171`**, peers `w-stmt5` and `w-json` still in
               flight (§9).

**`Outcome: built`.** The lane preregistered a step count, a total ceiling and a
buy, and landed all three with the bias direction stated in writing and a
registered anti-inflation check that **fired twice**. It built nothing:
`git diff master..HEAD -- crates fixtures scripts` is **empty**.

---

## 1. The one-paragraph answer

**Item F costs 17 lanes as a ceiling with no discount factor, decomposes into 7
steps, and buys 0 on the 878-TU workload scan, 0 on the 381×18 fixture gate, 0
on `c2rs perf`'s `/Ox` gate and 0 on the frontier.** Two things make that number
different from the one in circulation. First, **the reading rule is four for
four**: item F's *title* and its *enforcing cells* quantify over different sets,
neither contains the other, and the set the title names is the one where the
price is **not**. Second, **the scheduler is not the expensive half** — F0 is 8
of the 17 and the allocator half, quoted since #3057 at *"~30 lines plus the
textbook"*, is **9**. That was registered the other way round (P2.4) and it is a
**MISS**.

For calibration: `CEILING.md` §6.2 puts the last five conversions at **~17
landed rungs each**. Item F prices at one historical conversion and converts
nothing.

## 2. What it admits, and what it refuses

**Admits nothing.** No `crates/` change, no `DISCLOSURE.md` row, no function
class, no fixture. The only new evidence this lane produced is **two decompiled
functions** from the flat export (`0x10b7e6af`, `0x10b7dc51`), **two call-graph
queries** (`0x10b36f7e` has 7 callers; `0x10b31c9a` has 1) and **four counts
measured in this repository** (165 distinct refusal strings / 30
register-grounded / 17 sites over 5 independent variables / 22 files hard-coding
a physical register).

**Refuses to claim**: that its 17 is a bound on difficulty rather than on the
*shape* of the work; that §4.3's hypothesis about two preregistered negative
results is anything but a hypothesis; and that reading `0x10b7dd2c` /
`0x10b7ddff` / `0x10b7de4a` — priced as one sub-lane on the strength of their
*position* alone — would leave F0 at 8. If any of the three is large, F0 is
larger.

## 3. The reading rule, applied before a single step was priced

Three for three going in (#3114 item G, #3119 item D, #3151 fence A); **four for
four coming out.**

**Item F's title** — *"Values live across block boundaries — the real cost."*
**Item F's mechanism**, read off `WB_LIVE_FINDINGS.md` §2/§6.1 and its addresses
— *a candidate whose live range spans a physical def or a clobber-set operand,
in the instruction order the allocator is handed.* **That contains no block.**
The block enters only through the liveness *transfer function* — how the range
is **computed** — never through any **decision**. There is no rule anywhere in
c2's allocator that reads *"this value crosses a block boundary"*.

A witness on each side, so the claim is not symmetric hand-waving:

* **mechanism ∖ title** — `wbl_v3` (#3054): `r11` holds **three different
  values inside one straight-line block** with no call between them. The
  strongest refutation of the port's incumbent register model is entirely
  *intra*-block. So is the whole 492-cell evidence base of `codegen::alloc`.
* **title ∖ mechanism** — `codegen::fp_store_diamond`'s `FPR_A = f0`: *"read by
  the then-arm AND the join, which straddle the branch"*, held in a **volatile**
  FPR, no copy, no callee-saved register, **byte-exact today**. A value crossing
  a block boundary with nothing clobbering it costs the port the intra-block
  *"don't reuse"* rule and nothing more.

**`WB_LIVE_FINDINGS.md` §6.2 wrote this item's refutation inside the section
written to support it** — *"**Nothing is special about the entry block** — the
copy lands wherever the arrival is"* — and nobody read it as one. That is the
generalizable part: the disease #3151 named is not that prose gets written
carelessly, it is that **nothing in this repository compares a doc's quantifier
with the mechanism three files away**, so a correct sentence can sit inside a
document for two days without anyone noticing it contradicts the heading.

### 3.1 The evidence base is three cells and item F buys zero new bytes on all three

| cell | today | item F adds |
|---|---|---|
| `?MemFree`, `v2` r4 → **r11** in the entry block | **SHIPS BYTE-EXACT** — `codegen::cond_tail`, `mr r11,r4` at offset `0x0000`, graded against §4.1's 36 published bytes | nothing emitted; the `11` becomes derived |
| `?d_join`, `b` in **r31** across the call | **OUT OF THE ACCEPTED CLASS** — §3.4.1 calls it downstream of code motion; §8/§6.3 put *"arms ending in the same call"* outside everything specified | nothing; it needs the pass §6.3 forbids |
| `?b_if2`/`?b_ifn`, formals in **r31/r30**, framed | **SHIPS BYTE-EXACT** in this shape — `codegen::if_call_join` plus three `PARK_REG = 31` classes | nothing emitted; the `31` becomes derived |

And **cell 3 needs no block boundary at all**: `wb-live` grades it on
`wbl_x1`/`x2`/`x5`, which are straight-line bodies with calls. `wbl_x4` is the
only multi-block cell of the four and it had to be constructed specially.

### 3.2 §6.2 item F and §6.3 bullet 1 are in CONTRADICTION

§6.3: *"**No code motion.** §3.4.1's hoist and tail-merge are recorded as a
limit on the accepted class … **not as a pass to build**."* Item F's cell 2 **is**
§3.4.1, and #3099 put addresses on both halves: `0x10b3b167` is the tail-merge /
cross-jump, `0x10b3b41b` is the head-merge / hoist. **Item F cannot be built to
its own cells without building the pass §6.3 declines.** Nothing in the
repository records this; `CFG_SHAPE.md` §6.2's item F now carries a dated box
that does.

## 4. The finding nobody had composed: four order-changing stages after allocation

Read whole out of the export. `FUN_10b7e6af` @ **`0x10b7e6af`** orders the
per-function pipeline; `FUN_10b7dc51` @ **`0x10b7dc51`** ends with the register
allocator:

```
0x10b7dc51 : 0x10b38099 · sched(mode 1) · globregs 0x10b57633 · sched(mode 1)
             · ***** REGISTER ALLOCATOR 0x10b31c9a *****  · sched(mode 1)
0x10b7dd2c · 0x10b7ddff · 0x10b7de4a          the lowering band
0x10b7ded5 -> 0x10b3c6e5(·,2) -> 0x10b3c2cc    the FIVE block mergers
0x10b7df57 -> 0x10be6382(·,0)                  the FINAL schedule
```

`0x10b31c9a` has **exactly one caller** and it is `0x10b7dc51`. So:

> **The instruction order the allocator sees is the output of schedule pass 2.
> The order in the obj is that plus pass 3, plus a three-pass lowering band, plus
> five block mergers, plus the final mode-0 schedule. The order that decided the
> registers does not appear in the object file.**

Both halves were published — `WB_DAGORDER_FINDINGS.md` §1 places the mode-1
passes around the allocator, `WB_DAGCLIENTS_FINDINGS.md` places the mergers
before the final schedule — and **neither states that all of them are downstream
of `0x10b31c9a`**. It also sharpens `WB_LIVE_FINDINGS.md` §9.2: the order item F
needs is **earlier** than the lowering band, not part of it.

**A hypothesis, labelled and not banked.** `codegen::alloc`'s preregistered
**52,416**-configuration allocator search tops out at 179/236 with residual
exactly the tie tier; `codegen::schedule`'s **13,104**-configuration list-scheduler
search tops out at 89/146 with residual exactly the two-producer tier. Both were
fitted against **emitted** bytes. The composition above is a candidate mechanism
for both residuals at once. **This lane built no cell that could falsify it**
and does not claim it.

## 5. The steps and the price

Full text with fail-closed boundaries in
[`WB_ITEMF_FINDINGS.md`](../whitebox/WB_ITEMF_FINDINGS.md) §5–§6.

| step | buildable today? | lanes |
|---|---|---:|
| **F0** the allocator's input order + the four stages after it | **no** — needs a **tuple-level IR below item A** (item A's `BasicBlock` carries its run as *bytes*; a DAG needs kinds, operands, def/use, symbol destinations) | **8** |
| **F1** the candidate set, globregs `0x10b55732` | **no** — promotion policy uncharacterized | **2** |
| **F2** liveness (backward fixpoint ∩ availability) | **yes** — item A, `BodyLayout`'s placement order, `FinishedBody::start_of`, nine clients | **1** |
| **F4** allowed-set narrowing | **half** — the call case is black-box re-derivable; the **non-call physical def has no obj cell** | **2** |
| **F5** the colouring **and the candidate order** | **no** — `0x10b31c9a`'s worklist order is unread | **2** |
| **F6** arrival copy + save set | **yes** (acyclic) | **1** |
| **F7** the fence that would ship F2/F4/F5/F6 without F0 | **yes**, and it admits nothing new | **1** |
| | | **17** |

**Ceiling, no discount factor.** Five of six times a discount was applied on
this project it was the error.

**F5 is the correction that matters.** The register order `[r11 … r3, r31 … r14]`
is settled and cheap. *Which candidate is coloured first* is not:
`WB_LIVE_FINDINGS.md` §10 records `wbl_x2` as unexplained and the driver's
worklist order as **unread**, and this repository has already attacked the same
unknown from the black-box side and been **refuted** — `codegen::alloc`'s clauses
1–4 *are* a fitted candidate order, **clause 2 is refuted** (#836, `w-alloc2`, 7
of 56 fresh-holdout cells), and clauses 3/4 *"carry opposite signs inside one
sort, which is why the rule is not a priority function"*. **`codegen::alloc`'s
sort key and `0x10b31c9a`'s worklist order are the same unknown from two sides
and nobody has connected them.** The step called *"~30 lines"* is a
characterization lane plus a construct rung.

### 5.1 The refusal count, and the trap firing twice

`crates/c2-core/src/` carries **165 distinct** `out_of_class` strings, **30**
register- or liveness-grounded. Asking *"what varies between these refusals?"*:

* **Item F lifts 5 independent refusals over 17 sites** — V-ARRIVAL (7 sites),
  V-CS (6), V-PLAN (2), V-POOL (1), V-FPPRESSURE (1).
* **V-PERM (3 sites) it does NOT lift**, and the refusal string says why:
  *"c2 breaks the cycle through the callee-saved register instead of r11, **which
  is not characterized**"*.
* **V-ARITY is FIVE refusal messages reading ONE variable** — a call argument, a
  chain link's argument, a literal argument, a data symbol's address and a
  parked permutation, each *"past the eight register slots"*. Counting those as
  five is exactly the error the brief warned about, and it is measurable here.
* The registered anti-inflation check (**P2.3**) also fired on the
  decomposition: availability started as its own step and **collapsed into F2**,
  because no cell of the 25 separates them. **Both collapses came out of the
  total**; without them the number would have been 18 and the refusal count 10.

### 5.2 The re-expression base nobody had counted

**22 files** in `crates/c2-core/src/codegen/` hard-code at least one physical
register number. A construct rung for item F is graded by a **required-zero byte
delta**, so all 22 must come back byte-identical with the constant replaced by a
colouring result — plus `codegen::alloc`'s **492** exact cells (236 fit + 250
holdout + 6 killer).

## 6. What it buys, in each named population (#3125)

| population | today | item F complete |
|---|---|---|
| **878-TU dc3 workload scan** | `match 25 · mismatch 0 · codegen-gap 0 · vocab-gap 845 · capture-fail 8` | **0 conversions.** `codegen-gap` is **0 over all 878** — no TU waits behind the emitter. A codegen item cannot move `match` when nothing reaches codegen |
| **381×18 fixture gate** | 150 port Match, 0 mismatch, 231 not-implemented | **0** — a construct rung is required-zero *by the grading rule*; F7's fence admits nothing new |
| **`c2rs perf` `/Ox` gate** | 451× geomean over matched fixtures | **0** — perf times an obj already proven byte-exact |
| **the frontier** | `FRONTIER` **2** by codegen breadth alone | **0** — **`frontier-codegen-refused` is 0 of 59**; the frontier's emitter refuses nothing because the reader refuses first, and 48 of 59 die in the IL parser |

**The one positive buy has no unit.** Item F would turn 22 files' transcribed
register constants into derived ones and replace a fitted sort whose clause 2 is
refuted with a mechanism. That is a **soundness** buy, and
`PROGRESS_METRIC.md` scores a transcribed-but-correct constant identically to a
derived one. **So, plainly: item F buys zero on every population the goal is
written in.**

## 7. What cannot be priced

Eight, named in `WB_ITEMF_FINDINGS.md` §9: `0x10b3b5fd` (reached, never fires on
either grid); M5 `0x10b3ab86` → `0x10b394f5` (entered on every optimized cell,
never reaching its inner call); the globregs promotion policy at `0x10b55732`;
the availability intersection (necessity unmeasured — its zero inside F2 is
**absence-grounded**); **the non-call physical-register def — item F's flagship
cell's own mechanism, with no obj cell in existence**; cycle-breaking in the
arrival permutation; the scheduler's **cost term** (inert on all 25 cells — it
can be priced neither up nor down); and the `/LTCG:PGI` axis (off-path for a
workload built `/O1 /EHsc /GR` — a **coverage bound, not a zero**).

**#3057's four not-blocking items are NOT re-priced here**: the interference
graph, the cost function, the spiller, a callee-saved policy. Nor is
`w-merger4`'s floor.

## 8. The gate

Docs-only; `git diff master..HEAD -- crates fixtures scripts` **empty**.

| check | result |
|---|---|
| `gate.sh --jobs 4 --require-graded`, start and end | **GATE: PASS** · `graded tree e6d4bfb38066` · **730 files** at both ends · 18/18 PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT · **6,858 fixture-verdicts** · the lane table `diff`s **IDENTICAL** between the two runs |
| expression sweep | 19,556 checked · **0 mismatches** · 19,460 graded |
| mode cross | 90,424 of 90,812 case-lane cells · **0 mismatches** |
| 878-TU scan | `match 25 · mismatch 0 · codegen-gap 0 · vocab-gap 845 · capture-fail 8` · **370 keys** — digit-identical |
| `board_audit.sh` | all-zero on all five checks |
| `rung_registry` | 2/2 |
| `cargo test --workspace --release --no-fail-fast` | **1,619 passed, 0 failed / 42 targets** |

**Qualified, and the qualification is stated.** Both runs print `GATE: PASS
(**HATCH-RED REFUSED**)` — `hatch.py apply` cannot hatch a worktree tree (board
**#1389**), so its eight refusals were not exercised at either end. **Identical
at both ends and not caused by this lane** (`graded tree` is master's own hash;
the `crates fixtures scripts` diff is empty), and per board **#1406** this run
does not establish what a full run establishes.

## 9. Board rows — **MINTED `#3165`–`#3170`**

Drafted `F-1`…`F-6` **unnumbered**, then minted on the coordinator's explicit
instruction after the price was reported: `F-1` → **#3165**, `F-2` → **#3166**,
`F-3` → **#3167**, `F-4` → **#3168**, `F-5` → **#3169**, `F-6` → **#3170**.
**The next free number is `#3171`, and TWO LANES ARE IN FLIGHT: `w-stmt5`
(owner of `crates/c2-il`) and `w-json` (owner of
`crates/c2-core/src/codegen/`).** The block is appended at the **bottom** of
`docs/BOARD.md` (`2026-08-14-irg.md` §8.5's ordering hazard), no predecessor
block is edited, and the free pointer was **re-read from `BOARD.md` in the same
edit that spends it** (#3117) rather than taken from the hand-off.

**"No lane is in flight" is deliberately NOT written**, because that sentence is
how four rows ended up on master with no identity this week: `board_audit.sh`
checks that no two rows claim one number and that every *cited* number has a
row, so **a row with no number at all is invisible to it by construction**.

| row | title | status | evidence |
|---|---|---|---|
| **#3165** (`F-1`) | **ITEM F'S TITLE AND ITS MECHANISM QUANTIFY OVER DIFFERENT SETS AND NEITHER CONTAINS THE OTHER — the block boundary enters only through the transfer function, never through a decision; the fourth instance of #3151's disease** | MEASURED | `wbl_v3` (mechanism ∖ title, #3054) and `codegen::fp_store_diamond`'s `FPR_A = f0` (title ∖ mechanism, byte-exact). `WB_LIVE_FINDINGS.md` §6.2's own *"nothing is special about the entry block"*. WB_ITEMF §2 |
| **#3166** (`F-2`) | **FOUR ORDER-CHANGING STAGES SIT BETWEEN THE REGISTER ALLOCATOR AND THE OBJ — the order that decided the registers does not appear in the object file** | READ, address-cited | `FUN_10b7e6af` @ `0x10b7e6af` and `FUN_10b7dc51` @ `0x10b7dc51`, whole; `0x10b31c9a` has exactly one caller. Both halves were published and neither was composed. Carries a **labelled hypothesis** for `codegen::alloc`'s 52,416-config and `codegen::schedule`'s 13,104-config residuals. WB_ITEMF §4 |
| **#3167** (`F-3`) | **§6.2 ITEM F AND §6.3 BULLET 1 ARE IN CONTRADICTION — item F cannot be built to its own cells without the "no code motion" pass §6.3 declines, and item F's evidence base is 2/3 already-shipped and 1/3 out of class** | MEASURED | `codegen::cond_tail` ships `?MemFree`; `codegen::if_call_join` + three `PARK_REG` classes ship `?b_if2`'s shape; `?d_join` is §3.4.1, whose two transforms are `0x10b3b167` and `0x10b3b41b` (#3099). WB_ITEMF §3 |
| **#3168** (`F-4`) | **`codegen::if_call_join`'s PARK RATIONALE IS REFUTED BY ITS OWN LISTING — it parks in r10, and `ARG_REGS = [3..10]`. The bytes are right; the reason is wrong** | FILED, NOT APPLIED — a `crates/` change and this is a docs lane | The comment says *"cannot stay in a volatile argument register"* and emits `mr r10,r3`; both `cmpwi cr6,r10` sit **above** both `bl`s, so the range never straddles a call. L0 gives r10 exactly: r3 clobbered by the next word, r11 taken by the accumulator home. WB_ITEMF §3 box |
| **#3169** (`F-5`) | **THE STEP QUOTED AT "~30 LINES PLUS THE TEXTBOOK" IS THE LARGER HALF: `codegen::alloc`'s FITTED SORT AND `0x10b31c9a`'s UNREAD WORKLIST ORDER ARE THE SAME UNKNOWN FROM TWO SIDES** | MEASURED / OPEN | `WB_LIVE_FINDINGS.md` §10's unexplained `wbl_x2`; `codegen::alloc` clause 2 refuted (#836) under a preregistered 52,416-configuration search. F0 prices at 8 and the allocator half at 9 — a registered **MISS** (P2.4). WB_ITEMF §5, §6.1 |
| **#3170** (`F-6`) | **ITEM F IS PRICED AT 17 LANES (ceiling, no discount) AND BUYS 0 ON EVERY NAMED POPULATION — the one positive buy has no unit in any published metric** | PRICED, and the decline is the deliverable | `codegen-gap` **0** over 878; `frontier-codegen-refused` **0 of 59**; a construct rung is required-zero by the grading rule. 5 independent refusals over 17 sites; 22 files to re-express. WB_ITEMF §6, §7 |

## 10. Found and not taken — and the ranking caveat comes FIRST

`w-loo` (#3135–#3140) measured that **five of six published rankings carry no
information** (ρ ≈ +0.047 — noise, not inversion), that **a ladder never scores
what it starts with**, and that leave-one-out's **zeros do not compose**. It
found a row pointing at something that ships and **deliberately refused to
dispatch it**. So this list is published **unranked as to value** and ordered
only by how cheaply each could be checked. **Nothing here is a dispatch
recommendation.**

1. **§4.3's hypothesis is one cheap experiment.** Whether the allocator's input
   order explains `codegen::alloc`'s and `codegen::schedule`'s twin residuals is
   testable by ablating the post-allocation stages on the pinned image and
   re-fitting — the method `w-dagclients` and `w-merger4` already own. **Not
   taken:** this lane built no cell, by design.
2. **F-4 is a one-line `crates/` comment fix.** **Not taken:** docs-only lane;
   editing it would move the `graded tree` hash this lane's prereg registers as
   unchanged.
3. **`0x10b3b5fd` has no firing witness on two grids.** A grid built to fire it
   would close one of the eight unpriceables. **Not taken, and deliberately not
   recommended:** dispatching off it would be dispatching off an **absence**,
   which is this repository's most persistent defect family (~15 instances,
   #1823 the worst).
4. **`0x10b7dd2c` / `0x10b7ddff` / `0x10b7de4a`, the lowering band**, are priced
   as one sub-lane on the strength of their *position*. Reading their sizes is a
   `grep` away and would tighten F0's 8. **Not taken:** it is F0's work, not a
   pricing lane's.
