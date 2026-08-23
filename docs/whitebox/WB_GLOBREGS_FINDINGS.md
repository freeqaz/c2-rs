# WB_GLOBREGS — read **R4**: the tie key is `cand+0x44`, it is **written**, and the read plan's entry point is not where the answer lives

> **PROVENANCE — DISASSEMBLY-DERIVED.** Every address is an absolute VA in
> `compilers/X360/16.00.11886.00/c2.dll`, sha256
> `c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`, verified
> by this lane against the repo copy **before any address was read** and
> re-verified by `docs/whitebox/scripts/dump_globregs.py`, which refuses to run
> against any other digest. Legend: [`ref/README.md`](ref/README.md) §2.

Lane `w-read-r4`, 2026-08-23. Prereg:
[`WB_GLOBREGS_PREREG.md`](WB_GLOBREGS_PREREG.md) (committed at **`7330fed41`**,
the **first** commit on the branch). Spec:
[`READ_PLAN_2026-08-21.md`](READ_PLAN_2026-08-21.md) §3 row R4 and §5.2.
Funded by [`../DECISIONS_2026-08-22.md`](../DECISIONS_2026-08-22.md) decision 6,
board **#3410**. The spec page is
[`ref/P_GLOBREGS.md`](ref/P_GLOBREGS.md); raw listings for all 13 functions,
objdump **and** Ghidra, are under [`labels/globregs/`](labels/globregs/).

---

## 0. The answer to the question R4 inherited

`READ_PLAN` §5.2, after R1: *"Consequence 3 loses its premise, so the ten
refuted alloc keys have no explanation on this mechanism — the question passes
to R4."*

> **They now HAVE an explanation, and it is not the one anybody was looking
> for.** The allocator's tie key is **`cand+0x44`**, and `+0x44` is **written**:
> one origination site, **`0x10b55fac`** in **`FUN_10b55eae`**, storing a
> **tuple-visit ordinal** — a counter zeroed once per function at `0x10b55eb7`
> and incremented once per real tuple at `0x10b55f77`. So the tie tier is a
> **sort on a program-position quantity**, not the hash-bucket walk
> `ref/P_REGALLOC.md` §4 consequence 3 describes; the bucket walk is only the
> **third** tier, reached when two candidates tie on `+0x0c` *and* on `+0x44`.
>
> Every one of the ten refuted keys is a function of a **source-level property
> of a variable**. None is that ordinal, and three structural reasons make it
> impossible for any of them to be — including that **a variable is not a
> candidate**: a symbol with *k* live-range versions mints *k* candidates, so
> a one-candidate-per-variable key is wrong *in kind*, not mis-fitted.
> `ref/P_GLOBREGS.md` §8.

And a second answer nobody asked for, which is the sharper one:

> **The 52,416-configuration null was structurally guaranteed.** That search
> (`alloc.rs:29-36`) ranged over **priority functions** — candidates for
> `cand+0x0c` — and its residual is *exactly the tie tier*. The tie tier is not
> a priority function at all: it is an ordinal stamped by a **different
> phase, in a different function**, before any priority is accumulated. **No
> member of that family could have expressed it.** The null is therefore not
> evidence about priority functions and should stop being cited as such.

---

## 1. Two dispatch corrections, and the address that did verify

**The brief's address verifies.** `functions.tsv:936` — `0x10b55732`, size
**1676**, 2 params, 1 caller, 18 callees. Entry, size and the read plan's
"1,676 B" all reproduce exactly. **This is the first coordinator-supplied
address in this wave that needed no correction**, and it was checked and
recorded in the prereg (§0.1) before the body was opened.

**But the entry is one hop off the deliverable, and the deliverable is not what
the row says it is.** Both were registered as scorable predictions:

| `READ_PLAN` §3 row R4 says | what is true |
|---|---|
| *"`FUN_10b55732` … globregs mint/merge"* | it is the **renamer**. It holds **no direct call** to the mint `0x10b54d32` — 18 callees, none of them it. Candidate ids are assigned in **`FUN_10b55dbe`** (240 B), called by the same driver 40 bytes later at `0x10b577f2`. ⚠ **Prior-art correction to this lane's own first draft:** it claimed no document had named `0x10b55dbe`. **Two had** — `WB_LIVE_FINDINGS.md` lists it among the candidate creators, and R1's own table (`WB_CANDID_FINDINGS.md` §2) has `0x10b55e66` in `FUN_10b55dbe` as mint call site #1 of seven. **What is new here is not the address but the ORDER of its walk**, which R1 explicitly deferred: *"which of the seven sites it reaches, and in what order, is read R4"*. |
| *"the candidate mint order … **the missing input to the already-read comparator**"* | **the missing input was `cand+0x44`**, not the mint order. The mint order reaches the comparator only at the third tier. The row's framing sent the lane to the right subsystem for the wrong reason. |

Neither is a defect in the *funding* decision — R4 was the right read and it
did answer §5.2 — but a lane pricing Phase 1 off the row's sentence would have
read the wrong 1,676 bytes and missed the 1,468 that matter.

---

## 2. The prereg scorecard

Graded against [`WB_GLOBREGS_PREREG.md`](WB_GLOBREGS_PREREG.md) as committed at
`7330fed41`. **Misses are reported as misses and are not smoothed.**
**9 HIT · 8 MISS · 2 PARTIAL · 1 UNGRADED**, over 20 graded predictions, plus **P5.3** — the lane's own meta-condition — **satisfied**.

> **A tally this lane got wrong about itself first.** The first draft said
> *"11 HIT · 6 MISS · 3 UNGRADED"*. Recounted row by row: HIT = P1.1, P1.2,
> P2.1, P3.1, P3.3, P4.1–P4.4 (**9**); MISS = P1.3, P1.4, P1.5, P2.3, P2.4,
> P3.2, P3.4, P5.2 (**8**); PARTIAL = P2.2 (2 of 3 clauses) and P5.1 (HIT in
> substance, MISS in mechanism) (**2**); UNGRADED = P2.5 (**1**). The corrected
> tally is **worse** for the lane — two more misses and two fewer hits — which
> is the direction that matters: a prereg whose scorecard is rounded in the
> lane's favour is not a prereg. The per-row grades below were right all
> along; only the summary was wrong.

### P1 — is this the right function, and what shape is it?

| # | prediction | grade | evidence |
|---|---|---|---|
| **P1.1** | no direct call to `0x10b54d32`; the mint is in a callee or the caller's later phases | **HIT** | `calls.tsv` lists 18 callees, none the mint; the instruction stream agrees. The mint is a **sibling**, `0x10b55dbe` — the prediction said "callees or the caller's later phases" and it is the latter |
| **P1.2** | the outer loop iterates a **block list**, not a symbol table or a hash | **HIT, with a caveat the prediction did not anticipate** | the main loop (`LAB_10b5588e`) walks blocks. **But the body opens with a symbol-arena walk** at `0x10b557f9`, and the *mint's* outer loop — the one that actually matters — **is** a symbol-arena walk. The prediction was right about the function and wrong about the subsystem |
| **P1.3** | the inner tuple loop runs **forward** | **MISS** | it steps `T->[0x10]`, which [`ref/P_DAG.md`](ref/P_DAG.md):113 already reads as **prev**. The prereg said a backward walk "is a MISS and is separately reportable, because it inverts every order prediction downstream" — and it did exactly that (§3) |
| **P1.4** | blocks in **layout/linear** order via a single `next`-style chain, not RPO/DFS/worklist | **HIT on the chain, MISS on the direction** | it is a single chain, no worklist and no numbering array — but it starts at the list header's `+0x04` end and steps `B->[0x04]`, i.e. **backward**. Scored **MISS**: "layout order" was the claim and the walk is reverse layout order |
| **P1.5** | 2 params: a function/procedure record + a mode or class selector | **MISS** | `param_1` is the **first block** (`(*(proc+8))->[0x04]`, set up at `0x10b577c1`–`0x10b577c8`), `param_2` is the **starting index base** threaded from `FUN_10bfd665`. Neither is a mode selector; there is no per-class invocation |

### P2 — the promotion policy (item **F1**)

| # | prediction | grade | evidence |
|---|---|---|---|
| **P2.1** | a locatable predicate, expressible over named fields with offsets | **HIT** | two gates, both in `FUN_10b550e5`: a **kind** gate (`0x10b5511a`…`0x10b551c6`, every arm addressed) and a **type** gate (`0x10b551d4` → `FUN_10bd7d24`). `ref/P_GLOBREGS.md` §3 |
| **P2.2** | excludes kind 1 (physical register), kind 3 (memory), and an address-taken/aliased flag — scored 0–3 | **2 of 3** | kind 1 **is** excluded (`0x10b55138`, kinds < 3 reject). Kind **3 is NOT excluded — it is eligible**, which is the point of the pass: globregs *promotes memory-resident locals into registers*. The alias-style clause exists and is named: kind 3 additionally needs `sym+0x14 == 0` **and** `sym+0x07 & 0x40` clear (`0x10b551b3`, `0x10b551bc`) |
| **P2.3** | at least one numeric threshold in the predicate | **MISS — and the prereg called this "a clean result"** | there is none. The policy is wholly categorical: a kind switch plus a 30-entry table lookup. **A port needs no fitted constant for F1**, which is stronger than the read plan asked for |
| **P2.4** | the policy consults a compilation-mode global | **MISS** | `DAT_10c2e2cf` is read at `0x10b551dd` but only adds an index to a side bitset; it does not gate eligibility. **The mode-dependence is at the phase level, not the symbol level** — `DAT_10c2e2fc` (#3375), including a 40,000 size bail-out |
| **P2.5** | formals are promoted, and promoted first | **UNGRADED** | they are promoted (every probe cell depends on it). "First" is not separable: §4's observable cannot distinguish mint position from tie-key position — prereg §7 item 1 |

### P3 — `cand+0x44`, the field that decides every tie

| # | prediction | grade | evidence |
|---|---|---|---|
| **P3.1** | the complete writer set is enumerable two independent ways and is **small (≤ 6)** | **HIT** | **5 sites** — 1 origination, 3 split inheritances, 1 destructor `memset`. Ghidra xrefs and a displacement scan of the objdump agree, with every hit attributed to a containing function and disqualified when the same base carries a displacement ≥ `0x48` |
| **P3.2** | `FUN_10b55732` or its subtree is among the writers | **MISS** | the writer is **`FUN_10b55eae`**, a *sibling*. The read plan's entry point does not contain the tie key at all |
| **P3.3** | `+0x44` carries a **program-order quantity**, not a cost, weight or flag | **HIT** | a tuple-visit counter: zeroed once at `0x10b55eb7`, `inc` at `0x10b55f77`, stored at `0x10b55fac`. Marked `[R]`; §4.3 states exactly how far the obj takes it |
| **P3.4** | ⚠ **the registered rival**: `+0x44` is never written outside the spiller, so the tier degenerates to the bucket walk | **MISS — and inverted** | the spiller `0x10b3032a` **never writes candidate `+0x44` at all**. Its `+0x40`/`+0x44` save-restore is over the **basic-block** record — bitset pointers, and the same base carries `+0x48`/`+0x4c`/`+0x50`, which a `0x48`-byte record cannot have. `ref/P_REGALLOC.md` §2 and §4.1 are corrected beside |

### P4 — the merge rule

| # | prediction | grade | evidence |
|---|---|---|---|
| **P4.1** | a merge candidate at a join for a value reaching from ≥2 predecessors; a union of live ranges, not a fresh unrelated value | **HIT** | `FUN_10b54c07`, `ref/P_GLOBREGS.md` §5 |
| **P4.2** | `FUN_10b54c07` is on that path (a held fact — scores as a check on the predecessor) | **HIT** | confirmed; `WB_LIVE_FINDINGS.md:278` was right |
| **P4.3** | keyed on the original symbol identity | **HIT** | via `DAT_10c400d0[i]` at `0x10b54c50` — the index→symbol table |
| **P4.4** | does a merged candidate take an input's number or a fresh one? *"'a merge happens' is not the deliverable"* | **HIT** | **both, conditionally**: it searches the symbol's existing version records for one whose bitset meets the join's phi set (`FUN_10b273d3`) and **reuses** that number if found; otherwise it mints a fresh one and increments. Address-cited in §5 |

### P5 — the headline

| # | grade |
|---|---|
| **P5.1** the affirmative branch | **HIT in substance, MISS in mechanism.** The tie key *is* a lowered-program-position quantity, so the ten keys were indeed fitting the wrong variable — that is the claim, and it holds. But the specific mechanism predicted ("mint order is a function of lowered program order") is **wrong**: mint order is symbol-arena order, and it is not the comparator's second key at all. The right answer was a field the prediction treated as a side question |
| **P5.2** the negative branch | **MISS** |
| **P5.3** exactly one of P5.1/P5.2 scores HIT, or the rung says `FAILED` | **satisfied** — P5.1 |

---

## 3. What P1.3's MISS cost, stated because the prereg said it would

The prereg registered a backward inner walk as *"a MISS and separately
reportable, because it inverts every order prediction downstream"*. It did:

* The renamer is **not forward SSA renaming**. It is a **backward live-range
  construction** — the running list is the live set, an operand encounter adds
  to it, and the arm where the remaining-use nibble reaches 0 (the definition,
  reached last going backward) removes it.
* Therefore **version 1 goes to the symbol encountered earliest in the
  *backward* walk, i.e. latest in program order.** A port modelling the
  numbering as forward-SSA produces the reversed sequence.
* And a first draft of `ref/P_GLOBREGS.md` §1 asserted steps 2 and 4 were *the
  same walk*. **They run in opposite block directions** — step 2 from the list
  header's `+0x04` end via `B->[0x04]`, step 4 from `+0x00` via `B->[0x00]`.
  The draft is corrected in place with the evidence rather than quietly fixed
  (`ref/README.md` §2.1).

---

## 4. Control C2 — the confirmation probe

Preregistered in [`WB_GLOBREGS_PREREG.md`](WB_GLOBREGS_PREREG.md) §6, committed
at `7330fed41` **before any obj was compiled**. Driver
**`scripts/globregs_c2.py`**, committed under `scripts/` and re-run from there
because #1406 binds anything whose output is quoted as evidence; it degrades to
`SKIP: toolchain absent` (exit 2). Oracle: real `cl.exe` 16.00.11886.00 under
wibo. **Every cell run at both `/O1` and `/Ox`** — `ref/P_REGALLOC.md` §5's
trap (board #3241: the two reverse on 6 of 20 cells).

**Observable**: the prologue `mr rTARGET, rARG` map, i.e. which callee-saved
register each formal lands in — a direct readout of colouring order.

```
G-pos (positive control)  : DIFFERENT -> instrument LIVE      (both modes)
G-ladder N=2..8           : a0->r31 a1->r30 a2->r29 ... a7->r24  (both modes)
G-perm  24 permutations   : 24 of 24 identical to base          (both modes)
G-block separator         : UNCHANGED in both variants          (both modes)
```

**Population: 262 resolved formal→register assignments over 62 grid objs** —
70 from the ladder (2+3+…+8 = 35 assignments per mode, 14 objs) and 192 from
the permutation grid (24 objs per mode × 4 formals) — plus 4 separator objs
(8 assignments) and 4 positive-control objs. **70 objs compiled in total.**

> **An arithmetic correction this lane made to itself, kept rather than quietly
> fixed.** A first draft of this section, of the rung and of board **#3414**
> said *"118 assignments over 66 objs"*, reached by adding 70 **assignments**
> to 48 **cells** — two different units. The corrected figures are above and
> are larger, so nothing published rested on the smaller number; it is recorded
> because a denominator reached by adding incompatible units is precisely the
> defect `STATUS.md` tells readers to check for.

### 4.1 What went green, and what that is worth

* **G-pos fired in both modes.** Without it every green below would have been
  discarded rather than published (R1's rule; `STATUS.md`'s standing trap).
* **G-perm is a real discrimination and it did the job it was built for.**
  All 24 permutations of the *use* order, with declaration order held fixed,
  give the identical map. **That refutes any "last use position" key** — under
  which the map would follow the permutation — on 48 cells. It is the cell
  that could have gone red in the most likely way the read was wrong, and it
  did not.
* **The two modes agree on every cell.** That is itself a datum: #3241's
  `/O1`↔`/Ox` reversal does **not** reach this shape, so unlike the candidate
  order the *formals-across-a-call* map is profile-stable here.

### 4.2 What went UNGRADED, and the caveat that fired

**G-block came back UNCHANGED, and it is UNGRADED rather than a refutation.**
It was built to separate the two survivors — the `+0x44` ordinal (whose counter
is not reset per block) from plain arena/declaration order — by swapping which
of two formals appears in a later block. The ordinal reading predicts the map
swaps. It did not.

**The reason is the caveat committed with the cell, in the script, before it
ran:** moving a formal's later use into a different block changes its **live
range**, hence `cand+0x0c` — the **primary** key — so the comparator very
plausibly decided the pair on priority and **never reached the `+0x44` tier**.
A cell that does not reach the tier it probes grades nothing. **The separator
remains unbuilt**, and it is named as the next instrument (§6).

### 4.3 The limit of the green, fixed in the prereg before it ran

Prereg §7 item 1 — Trap A — registered that *"the observable is a many-to-one
image of the claim"*, and it is the binding limit here:

> On every straight-line body this lane built, the map is invariably
> `a_i → r(31-i)`. **That is equally consistent with the `+0x44` ordinal and
> with plain symbol-arena mint order**, because on a straight-line body the two
> coincide. **262 assignments over 62 objs do not separate them**, and no cell in this lane does.

So: the **existence and identity** of the tie key is `[R]`, verified at the
bytes twice-sourced, and it replaces consequence 3 outright. **What the ordinal
means in program terms is `[R]` only** — it rests on an inherited reading of
`tuple+0x10` as *prev* and on a traversal this lane composed rather than
confirmed. The obj confirms the *observable*, at `[O]`, and the observable is a
quotient.

---

## 5. Corrections filed by this read

| doc | site | was | now |
|---|---|---|---|
| `ref/P_REGALLOC.md` | §4 consequence 3 | *"on an exact tie the order is a hash-bucket walk"* | the bucket walk is the **third** tier; the second is `+0x44`, written at `0x10b55fac`. Revision box added beside |
| `ref/P_REGALLOC.md` | §2, `0x10b3032a` row | *"saves and restores `+0x40` **and `+0x44`** across a split"* | that is the **basic-block** record, not the candidate. The spiller never writes `cand+0x44`; its subtree only **inherits** it |
| `ref/P_REGALLOC.md` | §4.1, the `+0x44` row | *"the unenumerated field that decides every tie"*, no writer named | writer named, value read, default proven 0 |
| `ref/P_REGALLOC.md` | §7 | *"F1, globregs.c's promotion policy at `0x10b55732`: unread"* | read — and it is in `0x10b550e5`, not `0x10b55732` |
| `READ_PLAN_2026-08-21.md` | §3 row R4, §5.2 | *"`FUN_10b55732` — globregs mint/merge"*; *"the mint order … the missing input to the comparator"* | entry point is the **renamer**; mint is `0x10b55dbe`; the missing input was `+0x44` |
| `WB_LIVE_FINDINGS.md` | `:682-686` | *"its promotion policy … is not characterized"* | characterized; dated record left as written, pointer added |
| `WB_ITEMF_FINDINGS.md` | F1, §9 item 3 | *"NOT BUILDABLE"*, *"cannot be priced"* | the policy is read; §6 re-prices the residue |

## 5.1 No `DISCLOSURE.md` row is owed

`DISCLOSURE.md` is the ledger of findings **adopted into `crates/`**. This lane
is a characterization lane: `Fixtures: none`, `Census: +0`, **zero `crates/`
bytes**, and it adopts no constant, table or rule. R1 set the precedent and
`READ_PLAN` §5.2 already records that the "three rows" the plan predicted did
not materialise and that this is correct rather than a shortfall. Flagged, not
silently satisfied.

## 5.2 What this read implies for `crates/`, reported and NOT done

Under the docs-only fence (prereg §8 D4, R3's precedent with `LABEL_SEED_GAP`):

1. **`crates/c2-core/src/codegen/alloc.rs:103-539`'s catalogue should record
   that the ten keys now have a mechanism** — the tie key is a traversal
   ordinal, not a variable property — and that the 52,416-config null was
   structurally guaranteed rather than informative. **A doc comment, not a
   behaviour change.**
2. **No new key should be fitted from this.** The ordinal is over c2's lowered,
   scheduled tuple list, which the port does not have; §6 prices that.
3. `alloc.rs:40-43` points at R4 as the read that would settle it. It did, and
   the pointer should be updated to `ref/P_GLOBREGS.md` §8.

---

## 6. What this changes about the price of item F1

`WB_ITEMF_FINDINGS.md` F1 is **"NOT BUILDABLE"** at **2 lanes**, and §9 item 3
lists the promotion policy among the things that **cannot be priced**
(*"F1's ceiling of 2 is a guess at the shape of the work, not a measurement"*).

* **The promotion policy half is now a measurement, and it is small**: a kind
  switch with eight arms and a 30-entry boolean table, no threshold, no mode
  flag. That is a table transcription, not a lane.
* **The candidate-set half is not**, and the reason has moved. It is no longer
  "we do not know which symbols become candidates" — it is that a candidate is
  a **(symbol, live-range version)** pair, and producing the versions needs the
  backward live-range walk over the **lowered tuple list**, which is
  `CEILING` §6.1 phase 0/1 output.
* **The ORDER half is now a named dependency rather than a mystery**, which is
  the real re-pricing: `+0x44` is computable by anyone who has the lowered
  tuple list in c2's traversal order, and by nobody who does not.

**This lane does not re-price F1 numerically** — `docs/rungs/README.md` and
#1406 put pricing in a lane that measures it, and the honest statement is that
one of F1's two unknowns became a table and the other became a dependency.

**The next instrument, named:** a separator cell that reaches the `+0x44` tier
**while holding `cand+0x0c` fixed**. G-block failed because it moved the live
range. The shape that would work holds every live range identical and varies
only *tuple count between the same two program points* — e.g. padding one arm
of a diamond with tuples that touch no candidate. That is a half-day and it
would take §7.1's clause from `[R]` to `[O]` or kill it.

---

## 7. What was NOT read, stated so absence does not read as coverage

* **`FUN_10b55eae`'s other ~1,300 bytes.** Only the `+0x44` stamp and the
  operand re-point were read out of it. It is the largest unread body this
  lane touched and it is now known to carry policy.
* **Which of `T->[0x28]`/`T->[0x2c]` is the def list.** The numbering does not
  depend on it (versioning is on first encounter in either); the merge
  semantics might.
* **Whether the arena serial equals symbol *creation* order.** `0x10bd3225`'s
  free-list path preserves `sym+0x1c` across recycling, so a late symbol in a
  recycled slot carries an early serial. Not measured.
* **Floating point.** Gate B's classes `0x0d`–`0x0f` (nibble 5, FPR) are
  promotable and **no cell in any grid in this repo uses floating point**
  (`ref/P_REGALLOC.md` §7). Registered as uncatchable in prereg §7 item 3 and
  it stayed uncaught.
* **The `>1024`-candidate regime**, prereg §7 item 2 — `id & 0x3ff` wraps and
  the step-4 ordinal does not. No body here is close.
* **Spill/split interaction**, prereg §7 item 4 — no cell spilled, so the three
  `+0x44` inheritance sites were read and never exercised.
* **`/Od`**, prereg §7 item 5 — the phase does not run; blind by construction.
* **The IL-record side** — which IL constructs create which symbol kinds is
  read **R5**'s subject (`FUN_10bc2d7a`). Cited here as an **open
  cross-reference**, not a claim; this lane touched nothing of R5's.
