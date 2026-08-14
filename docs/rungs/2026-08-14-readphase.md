# w-readphase — the reader ladder is **≥ 3 clause rungs deep before it opens into a 615-key space**, the whole expression layer's decode ceiling is **76,041 of 120,456 emitted functions reaching the function tail**, and the thing nobody had measured is the OTHER SIDE: a decode-only widening of the standing ladder instrument **costs SEVEN TU matches and 5,949 byte-exact functions**

    Tag:       w-readphase
    Slug:      readphase
    Date:      2026-08-14
    Kind:      characterization
    Outcome:   instrument
    Fixtures:  none — characterization: *how deep is the reader's refusal
               ladder, what does a reader phase cost, and in what unit would a
               reader phase be graded offline?* This lane builds no class and
               admits nothing, so there is nothing to fixture.
    Census:    **+0**. `git diff 6f2c7c41 -- crates/` is EMPTY at this tip. Three
               scratch lifts were applied, measured and reverted; the base scan
               was re-run after the last revert and returned **370 `gap-metric`
               keys, 0 differing** against the scan taken before the first lift.
    Record:    this file. PREREG `work/w-readphase/PREREG.md`, committed at
               **`3ee6ff08`** as this lane's first commit, **before the first
               scan**. Scored in §8.
    Lane:      `w-readphase`, worktree branch `wt-w-readphase`, off master
               **`6f2c7c41`**. Master did not move under the lane.
    Ships:     nothing under `crates/`. Four analysis drivers under
               `work/w-readphase/` (`ladder.py`, `phase.py`, `greedy.py`,
               `keydiff.py`) and 25 workload scans. Board rows left
               **UNNUMBERED** for the coordinator (§9).
    Adopts:    **nothing.** No `docs/whitebox/DISCLOSURE.md` row, no constant,
               no width, no flag bit.

---

## 0. The base, and one correction to a number four lanes are quoting

Measured at `6f2c7c41` in this worktree, and re-measured identically after every
scratch lift was reverted:

| | value |
|---|---:|
| `match` · `mismatch` · `codegen-gap` · `port-error` | **25 · 0 · 0 · 0** |
| `vocab-gap` · `capture-fail` · `frontier` | **845 · 8 · 2** |
| `gap-metric` keys · jsonl rows · **graded** TUs | **370 · 878 · 870** |
| `fnbyte-exact` / `fnbyte-denominator` | **35,734 / 162,049** |
| `emit_blockers` | **615 keys summing 113,612** |
| `fn_blockers` (all IL bodies) | **635 keys summing 1,705,627** |

> ### ⚠ **`CEILING.md` §6's "130,575 of 139,792 emitted functions are blocked at the IL reader" IS STALE, and so is the denominator under it.** Today the same instrument reads **`fnbyte-refused-parse` 113,612** of **`fnbyte-denominator` 162,049** — 70.1 %. The published pair was measured at `85e180d4` against a 178,977 denominator. `STRATEGY_REVIEW_2026-08-13.md` §3 item 2, `CEILING.md` §6 and this lane's own dispatch brief all quote the stale pair. **The 93 % headline is also wrong in the same direction and by more:** the reader's share of the *refusal* population is 113,612 / (113,612 + 949) = **99.2 %**, and its share of the whole **denominator** is 70.1 %. Neither is 93 %.

The emitted widening order is likewise **615 keys summing 113,612**, not §3.1's
"648 keys summing 130,575", and its head `expr-op-0x27` reads **22,407**, not
22,373.

---

## 1. THE RESULT, up front

> ### **1. THE HEAD CLASS'S LADDER IS AT LEAST THREE CLAUSE RUNGS DEEP, AND `decode_causes`' ALL-CAUSE SET — THE INSTRUMENT BUILT TO ANSWER EXACTLY THIS — SAYS IT IS TWO.** All **819** TUs whose first blocker is `gl-stop-26-introduced` report a cause set of **arity exactly 2**, the second member being `body-out-of-class` on **819 of 819**. Lifting the clause in a scratch tree moves **2** of the 819 there. The other **817** land on **three `.gl` clauses the diagnostic could not see**: `bind-record-count-ne-segments` **518**, `gl-stop-varargs-record` **299**, `gl-stop-name-too-far` **11**. Lift those too and **three further causes appear that read 2, 1 and 0 at base** — `unclaimed-gl-symbol` **735**, `locally-defined-callee` **725**, `shape-token-unresolved` **715** — and the modal cause-set arity goes **2 → 4**.

> ### **2. AND THE PRICE OF THE WHOLE `.gl` WALK, LIFTED, IS `match` +0 AND `fnbyte-exact` −65.** Two rungs, four clauses, the entire binding walk opened: **match 25 → 25 → 25**, `mismatch` 0 throughout, and `fnbyte-exact` **35,734 → 35,669**. The head class's realized worth is not zero, it is **negative on the only metric that maps to the goal**. `w-vec`'s #2505 ("repairing it converts ZERO TUs") is confirmed and sharpened: it also *costs* 65.

> ### **3. THE WHOLE EXPRESSION LAYER'S DECODE CEILING, MEASURED: 76,041 OF 120,456 BLOCKED EMITTED FUNCTIONS (63.1 %) WALK THEIR BODY END TO END TO THE FUNCTION TAIL.** Granting every one of `chain_skip_form`'s **47** pinned opcodes plus `type`, `convert` and `intrinsic` — the committed, poisoned `C2RS_SINK_CHAIN` (board #660), no `crates/` change — leaves **72** keys. **76,041** land on the function tail `4F 12`; **0** on any other `4F`; **6** on `expr-chain-short`. The residue is **44,409 over 68 keys**, and its head is a short list of *identified* constructs, not a long tail.

> ### **4. `type`, `convert` AND `intrinsic` ARE WORTH EXACTLY ZERO AT THAT CEILING — the three scans are BYTE-IDENTICAL** (`md5 87b206ef…`, three specs). `IL_STMT_GRAMMAR.md` §14.1 prices the operand TYPE gate at **3.2×** and calls it *"the step with the largest measured decode gain"*. Once the opcode width table is granted wholesale it is worth **0**, because `op:B9`'s `TokType` form consumes a LOAD's TYPE whatever that TYPE is. §14.1's own sentence — *"the entire rest of the plain operator table is worth exactly zero"* — now applies to the step §14.1 ranked first.

> ### **5. THE OTHER SIDE OF THE PRICE, AND IT IS THE FINDING THIS LANE EXISTS FOR: A DECODE-ONLY WIDENING OF THE STANDING INSTRUMENT DE-ACCEPTS.** Granting **one** scaffold token, `op:41`, moves **2,694 emitted functions out of the accepted class** — every one of them onto `expr-jump` (298 → 3,731) — buys **zero** decode distance (poison 0, no body reaches an end), and drops **`match` 25 → 24** (`src/xdk/nuispeech/mmio.cpp`) and **`fnbyte-exact` 35,734 → 33,040**. At the 22-token ladder tip it is **`match` 25 → 18** — seven TUs by name — and **`fnbyte-exact` −5,949**. The sink's own doc says it *"cannot move one obj byte even when it is ON"*; that is true of **wrong emits** and false of **removals**, and every per-TU ladder this repo has published was climbed with `op:41` in its SEED.

> ### **6. SO THE GRADING UNIT IS NOT MISSING — IT IS `fnbyte-refused-parse` WITH A REQUIRED-ZERO SIDE, AND THE PREREG PREDICTED THE OPPOSITE.** This lane registered at p = 0.75 that FBM *cannot* grade a decode-only reader phase. It grades one extremely well: it grades it **negative**. §6 states the four-column offline gate that follows.

---

## 2. LADDER A — the head class, climbed by lifting, three compiles

Method: lift the clause in this worktree, `cargo build --release`, re-scan all
878 TUs, read the first-cause histogram, repeat. Reverted after each reading;
`git status --porcelain crates/` is **0 lines** at this tip and the base scan
reproduces to **370 keys, 0 differing**.

| rung | what was lifted | first-cause histogram over the 845 held TUs | `match` | `fnbyte-exact` |
|---|---|---|---:|---:|
| **0** | — (base) | **819** `gl-stop-26-introduced` · 13 `drectve-not-boilerplate` · 11 `bind-record-count-ne-segments` · 2 `body-out-of-class` | 25 | 35,734 |
| **1** | `GlBindStop::Name26Introduced` (`gl.rs:886`) | **518** `bind-record-count-ne-segments` · **299** `gl-stop-varargs-record` · 13 `drectve` · **11** `gl-stop-name-too-far` · 4 `body-out-of-class` | 25 | **35,669** |
| **2** | …+ `Bindings::selective` clauses 3 and 4 (`bind.rs:770-776`) + `GlBindStop::VariadicRecord` (`gl.rs:787`) | **720** `body-out-of-class` · **106** `gl-stop-name-too-far` · 13 `drectve` · 5 `varargs` · 1 `shape-token-unresolved` | 25 | 35,669 |
| **3** | `body-out-of-class` — **not a clause.** An open key space: **615 keys / 113,612** emitted-blocked | — | — | — |

**Depth: ≥ 3, and rung 3 is not a rung.** The class does not "finish parsing"
and it does not hit one structural thing; it opens into a key space two orders
of magnitude wider than the clause vocabulary.

### 2.1 The diagnostic's blind spot, stated precisely — and it is not a defect

`diag.rs`'s module doc already says it: *"Some gates cannot be asked without the
answer to an earlier one… Those are reported as **not evaluated**."* What this
lane adds is the **size** of that boundary, which had never been measured:

| cause | all-cause count at base | at ladder rung 2 | factor |
|---|---:|---:|---:|
| `unclaimed-gl-symbol` | 2 | **735** | **368×** |
| `locally-defined-callee` | 1 | **725** | **725×** |
| `shape-token-unresolved` | 0 | **715** | **∞** |
| `gl-stop-name-too-far` | 0 | **107** | **∞** |
| `bind-record-count-ne-segments` | 12 | 522 | 43.5× |
| `gl-stop-varargs-record` | 0 | 308 | ∞ |

So `fence_blocks`' standing caveat — *"a first-blocker count is NOT a distance"* —
is **true of the all-cause column too**, and by up to 725×. A lane reading
`decode_causes`' arity as a depth reads **2** where the answer is **≥ 4**.

### 2.2 What varies between these refusals, asked before they were counted

Three of the four `.gl` clauses lifted read **different fields**: a name run's
preceding separator byte (`26`), a record's own flags byte (`0x40`, VARIADIC),
and a name's distance from its record (> 32 bytes). `bind-record-count-ne-segments`
+ `selective-unaccounted-emitted-body` are **one** refusal counted once, not two —
they are pushed together by construction (`diag.rs:321-325`). So the rung-1
successor set is **three** independent refusals, not four.

---

## 3. LADDER B — the class-wide sink ladder, 14 rounds over all 878 TUs

`work/w-front3/ladder.py` climbs **one TU's** ladder; `w-ladders` climbed
sixteen. This is the same committed instrument driven over the **whole
workload**, so what it produces is the depth of the **class**. Seed: the
scaffold `op:41,op:4F,op:53,op:54,op:4B,op:29,op:38,op:39,op:3A`. Each round
grants the token that opens the current head of the *emitted* widening order.

`reach` = `expr-chain-sink-poison` (walked to the segment end) +
`expr-chain-fntail`/`noform-0x4F` (walked to the function tail). `blocked` is a
**control**, not a result: it is invariant unless a body is accepted.

| # | granted | keys | **reach** | `match` | `fnbyte-exact` | new head |
|---:|---|---:|---:|---:|---:|---|
| 0 | scaffold (9 tokens) | 612 | 5,082 | **24** | 33,040 | `expr-op-0x27` (22,529) |
| 1 | `op:27` | 748 | 5,486 | 24 | 33,040 | `expr-op-0x30` (18,171) |
| 2 | `op:30` | 802 | 14,322 | 24 | 33,040 | `expr-intrinsic-this-adjust` (9,273) |
| 3 | `intrinsic` | 787 | 19,345 | 24 | 33,040 | `expr-class-descriptor` (17,825) |
| 4 | `op:66` | 788 | 19,345 | 24 | 33,040 | `expr-op-0x55` (18,557) |
| 5 | `op:55` | 736 | 15,689 | **18** | **29,785** | `expr-op-0x4C` (21,080) |
| 6 | `op:4C` | 814 | 21,278 | 18 | 29,785 | `expr-op-0x9B` (7,824) |
| 7 | `op:9B` | 818 | 21,982 | 18 | 29,785 | `expr-cmp-eq` (5,206) |
| 8 | `op:1F` | 829 | 23,852 | 18 | 29,785 | `expr-call-in-expr-data-addr-then-intrinsic-call` (4,624) |
| 9 | `op:26` | **539** | 14,479 | 18 | 29,785 | `expr-op-0x99` (13,069) |
| 10 | `op:99` | **112** | 14,479 | 18 | 29,785 | `expr-op-0xBD` (51,164) |
| 11 | `op:BD` | 122 | **40,530** | 18 | 29,785 | `expr-op-0x32` (12,419) |
| 12 | `op:32` | 119 | 41,303 | 18 | 29,785 | `expr-op-0x5C` (13,169) |
| 13 | `op:5C` | 122 | **41,762** | 18 | 29,785 | `expr-convert-no-value-0x2C` (11,933) — **EXIT** |

The exit is honest and is not a rename: `expr-convert-no-value-0x2C` is a
**production** refusal outside `parse_expr`, which the committed sink structurally
cannot reach (`ladder.py`'s own `HATCHES` list names it, added by lane `w-one`).

**Two things this table says that no per-TU ladder could.** The key count
*collapses* at rounds 9–10 (829 → 539 → **112**) — granting `op:26` and `op:99`
absorbs seven hundred keys — and the reach *jumps* 14,479 → 40,530 on one rung
(`op:BD`, the CALL token). The widening order's mass ranking predicts neither.

---

## 4. THE CEILING — the whole expression layer, granted at once

Spec: all **47** opcodes `chain_skip_form` pins, derived from the tree by
`greedy.py:pinned_opcodes()` (which refuses `LADDER-NOWIDTHTABLE` rather than
returning an empty set), plus `type`, `convert`, `intrinsic`. **870 TUs graded.**

```
blocked emitted 120,456 over 72 keys
  reached FUNCTION TAIL (4F 12)   76,041   63.1 %
  reached SEGMENT END (poison)         0
  expr-chain-noform (other 4F)         0
  expr-chain-short                     6
  RESIDUE                         44,409   36.9 %  over 68 keys
```

### 4.1 The terminal was ambiguous and was RESOLVED, not assumed

`expr-chain-noform-0x4F` is `w-front3/ladder.py`'s `TAIL` constant, but
`chain_skip_form`'s `Line4F` refuses **every** `4F` that is not `4F 01`, and
`bundle.rs:292` records four members of the family — `4F 01`, `4F 02`, `4F 12`,
`4F 20`, `4F 33`. So the key could be *"the body was fully walked"* (`4F 12
47 54 01 54 00`, the function tail = segment end) or *"a mid-body marker this
tree has not decoded"*, and **the whole 76,041 turns on which.** A scratch split
of that arm (reverted; the tree is identity-clean) answers it positively:

```
76,041  expr-chain-fntail-0x4F      <- 4F 12, the function tail
     0  expr-chain-noform-0x4F      <- every other 4F
```

**All 76,041 are the function tail.** 76,041 is a **reach**, not a blocker.

### 4.2 The residue, and the refusals counted as INDEPENDENT rather than as keys

| refusal | keys it spans | emitted fns | what it is |
|---|---|---:|---|
| **`5D`/`5E`, the EH count trailers** | 2 | **13,158** | ONE refusal, not two: both are `<varint> <varint>` and **no `SkipForm` variant can spell that** (`chain_skip_form`'s "deliberately absent" list says so in those words). An instrument-expressiveness limit, not a missing measurement |
| **`0x64`, the by-value return materialize** | 1 | **8,000** | Identified in `IL_DECODE_REACH.md` §4 by elimination over a 27-function battery, and deliberately left out of the width table |
| **the statement layer** | 4 | **7,903** | `body-cflow-label` 2,832 · `body-0x9B` 2,213 · `return-scope-close-cflow-label` 1,814 · `body-0x67` 1,044 — outside `parse_expr` by construction |
| **the compound-assign family** | 11 | **5,269** | `0x19` 2,208 · `0x10` 2,188 · `0x36` 755 · `0x11` 58 · `0x16` 46 · `0x18` 7 · `0x12` 3 · `0x17` 2 · `0x13` 1 · `0x15` 1 · **`0x14` 1**. ONE production, eleven opcodes |
| **`0x9A`, the vtable-slot bind** | 1 | **2,674** | `IL_DECODE_REACH.md` §3's own finding, one opcode along from `0x67` |
| **`0x00` — NOT AN OPCODE** | 1 | **2,276** | The 64-bit-literal payload desync (`SkipForm::LitTypeVarint`'s doc, board #1465). An **instrument defect**, and no entry in the width table can fix a desync |
| everything else | 48 | 5,129 | 48 keys, largest 2,029 |

**Six refusals cover 39,280 of the 44,409 residue (88.5 %).** That is the reader
phase's real worklist on the emitted population, and it is short.

---

## 5. §14.2's DECODE ORDER, re-read against the tree — three of six steps are PAID, one is FALSIFIED, and the last one's RANK is an artifact

| step | §14.2's named gain | today | status |
|---|---|---|---|
| **1** line markers as varints | `body-0xAD` 15,480 + `body-0xB3` 15,782 + `body-0x4F` 26,666 | **0 · 0 · 0** | **DONE** — `readers.rs:409` is `4F 01 <read_varint>` in a loop, verbatim |
| **2** the scope stack | `body-0x53` 70,078 + `body-0x29` 14,594 + `fn-tail-0xB9` 29,552 | **0 · 0 · 0** | **DONE** — `expr.rs:141` `eat_scopes`, `53`/`54 <k>`, depth counter, `k != depth` refuses |
| **3** the statement list | `call-token-0x26` 80,284; the `assign-0x26` family | **10**; `assign-dst-not-formal` **0** | **DONE** |
| **4** compound assignment | *"a small gain (+2 bodies on Dir.cpp) but nearly free"*; *"`0x14` is unobserved and must reject"* | **45** emitted-blocked at base; **5,269** only once everything above it is granted | **STALE, AND FALSIFIED.** `expr-op-0x14` reads **1** emitted function at the ceiling. It is observed |
| **5** branches and labels | all of control flow bar `switch` | `expr-brfalse` 3,105 · `expr-brtrue` 1,941 · `body-cflow-label` 2,832 · `return-scope-close-cflow-label` 1,814 · `expr-jump` 298 = **9,990** | **PARTIAL — and it is the live one** |
| **6** the operand TYPE gate | *"the step with the largest measured decode gain"*, **3.2×** | **2,389** emitted-blocked over **22 sharded keys**; **122,736** over all bodies; **0** at the sink ceiling | **RANK IS AN ARTIFACT** |

### 5.1 Why step 6's rank inverts, and it is `GAPS.md` §6's rule twice

§14.1's whole table is measured in **bodies decoded on Dir.cpp** — the
all-bodies population. On the population the goal is written in, the type gate
is **2,389 of 113,612 (2.1 %)** against **122,736 of 1,705,627 (7.2 %)**: a
**51×** census-to-emitted gap. And its key is `expr-load-type-<per-TU id>` —
**exactly the sharded key `GAPS.md` §6 names**, *"a bucket key derived from data
the compiler allocates per input is not a stable key"*, spread over 22 shards
here. Both of §6's warnings fire on the one step §14.1 ranked first.

### 5.2 The order the emitted population implies today

Not "statements, then types". It is: **`5D`/`5E` (13,158, one new `SkipForm`) ·
`0x64` (8,000) · the statement layer (7,903) · `0x9A` (2,674)** — and *none* of
those four is a step in §14.2. §14.2 is not wrong; it is **finished** for its
first three steps and was written against a population that has since been paid
down.

---

## 6. THE GRADING UNIT for a reader phase — four columns, all offline, no obj emitted

`STRATEGY_REVIEW_2026-08-13.md` §4 lever 5 says reader work must be *"graded by
fnbyte/offline"* and prices it as *"unpriced (the largest unknown)"*. The
instrument exists and this lane's own measurement shows what it must be
**guarded with**, which is the half that was missing.

| column | key | today | rule for a phase |
|---|---|---:|---|
| **denominator** | `fnbyte-denominator` | **162,049** | c2's own `.text` COMDAT leaders. Never changes for the phase's benefit; a port that emits nothing scores `0/N` |
| **the phase's numerator** | **`fnbyte-refused-parse`** | **113,612** | **must FALL.** It is the only published key a decode-only change can move, and it does move: every lift in §2 moved it |
| **required-zero, side A** | `fnbyte-exact` | **35,734** | **must not fall.** §1 finding 5 is why: one scaffold token cost 2,694 |
| **required-zero, side B** | `match` / `mismatch` | **25 / 0** | **must not fall / must stay 0.** §1 finding 5 is why: one scaffold token cost `mmio.cpp` |

> **A reader phase PASSES iff `Δfnbyte-refused-parse < 0` **and** `Δfnbyte-exact ≥ 0` **and** `Δmatch ≥ 0` **and** `mismatch == 0`.** All four come off one `c2rs gap` scan that already runs, on 162,049 functions — *"a larger base than any whitelist entry ever had"* (§4 lever 3), and 6,482× the 25-TU base the differential grades. It is board **#290**'s construct-rung pattern one column over: three required-zeros and one column that must move.

**What counts as the first phase succeeding, stated as a number.** At the
ceiling, **76,041 of 120,456** blocked emitted functions walk to the function
tail. Applied at base that is the reader phase's honest first milestone:

    fnbyte-refused-parse  113,612  ->  <= 44,415
    fnbyte-exact          >= 35,734      match >= 25      mismatch == 0

That is a **counterfactual of the production being widened** — the 1.0002× kind
— so per ROADMAP's own rule **the ceiling IS the estimate**, with no discount.
What remains between it and the emitter is not a discount factor, it is a count
of independent refusals, and §4.2 gives them: six, covering 88.5 % of the
residue.

### 6.1 What this unit does NOT do

It does not license an emit. A body moving out of `fnbyte-refused-parse` lands in
`fnbyte-refused-codegen` (**949** today), *not* in `fnbyte-exact`, and every
`STATUS.md` trap about drivers applies unchanged. It is a **phase** gate, not a
correctness one; `mismatch` remains the alarm and real c2 remains the judge.

---

## 7. THE TWO-SIDED PRICE

**Cost of KEEPING the reader narrow**, in the units the goal is written in:

* **All 846 TUs, eventually.** `body-out-of-class` fires on **845 of 845** held
  TUs (`decode_causes`, base). Not one of them can convert while it stands, so
  the reader is a **necessary term in the entire remaining distance to 871**.
* **70.1 % of the goal's denominator is unaskable.** 113,612 of 162,049 emitted
  functions are refused by the parser against **949** by the emitter — **120 : 1**.
  Every codegen phase in `CEILING.md` §6.1 is being priced on the 30 % the
  reader lets through.
* **0 TUs today.** `codegen-gap 0`, `frontier 2`, and §2's ladder: opening the
  *entire* `.gl` binding walk converts **0**. No TU is waiting on the reader
  alone, and the head class's realized worth is **−65 `fnbyte-exact`**.

**Cost of WIDENING it**, measured this lane rather than argued:

* **`match` −7 and `fnbyte-exact` −5,949** at the 22-token ladder tip; **−1 and
  −2,694** from a *single* token that bought zero decode distance.
* The mechanism is not the poison — it is that **`parse_expr` is called by the
  shape recognizers**, so a wider expression walk pre-empts a recognizer that
  was already byte-exact. This is a hazard a real widening shares, and it is
  the mechanism behind `w-readpx`'s *"one 444-wide admission bought +0"*.

**The asymmetry is the argument.** The narrow side costs everything at 871 and
nothing today; the wide side costs measurable, named TUs the moment it is taken
carelessly. That is precisely a case for a **phase graded offline against §6's
four columns** and against nothing else — and precisely not a case for a TU lane.

---

## 8. THE PREREG, SCORED

`work/w-readphase/PREREG.md`, frozen at `3ee6ff08` before the first scan.
**6 hits, 8 misses**, and the misses carry the lane.

| id | registered | p | outcome |
|---|---|---:|---|
| **L1** | ≥ 750 of 819 → `body-out-of-class` | 0.88 | **MISS, hard** — **2**. The successor is three clauses the diagnostic could not see |
| **L1b** | the lift converts 0 TUs | 0.95 | **HIT** |
| **L1c** | `fnbyte-exact` moves by 0 | 0.85 | **MISS** — **−65**, the wrong sign |
| **L1d** | `fnbyte-refused-parse` moves by 0 | 0.85 | **MISS** — **+66** |
| **L2** | ladder depth is exactly 2 | 0.80 | **MISS** — ≥ 3, and the arity goes 2 → 4 |
| **L2b** | depth ≥ 3 | 0.15 | **HIT at p = 0.15** — scored as a calibration miss, not a win |
| **L3** | head class ≥ 600 keys summing ≥ 120,000 | 0.85 | **MISS**, both — **596** keys summing **110,100** |
| **L4** | head class's TU yield is 0 | 0.93 | **HIT** |
| **L5** | head class's `fnbyte-exact` yield is 0, as a ceiling | 0.80 | **MISS** — it is **negative**. *A ceiling with no discount was still optimistic, because the quantity's sign was not registered.* |
| **L6** | the successor is not computable without implementing the production | 0.70 | **HALF.** Wrong for **47** opcodes — the tree has a pinned width table and a committed instrument I did not know existed. Right for `5D`/`5E` (13,158, unspellable) and for `0x00` (2,276, a desync no pin can fix) |
| **S1** | ≥ 4 of §14.2's six steps are done | 0.60 | **MISS** — exactly **three** |
| **S2** | §14.2's order is stale in ≥ 1 place | 0.70 | **HIT — in three** |
| **G1** | FBM cannot grade a decode-only reader phase | 0.75 | **MISS, and INVERTED** — it grades one at **−2,694 / −1 TU**. This is the deliverable |
| **G2** | `fnbyte-refused-parse` is the key that moves | 0.65 | **HIT** |
| **P1** | the narrow side's price is non-zero | 0.55 | **HIT** |

### 8.1 The three that are worth reading

1. **L1 is the lane's method justifying itself.** The brief warned that a
   first-blocker key is not a distance. The stronger finding is that the
   **all-cause** set is not one either — it was built to be the answer to this
   question and it under-reports arity by 725× on one cause.
2. **L5 is the estimate discipline failing in the one way it was armed against.**
   The prereg said "ceiling, no discount". It did not say **which sign**, and the
   realized value was on the other side of zero from the ceiling.
3. **L6 is prior art I did not find in the two hours I spent looking for it.**
   `C2RS_SINK_CHAIN`, `C2RS_SINK_REL`, `C2RS_SINK_BRANCH` and
   `C2RS_SINK_OFF_ADD_ARG` are four env-gated ladder instruments living in
   `expr.rs` doc comments, reachable by no `docs/` grep and by no `BOARD.md`
   topic search. §9's last row is about that.

---

## 9. Board rows — **UNNUMBERED**, for the coordinator

Next free is **#3092** if `w-backedge` lands first; peers may shift it.

| # | item | worth (measured, not estimated) | defined | notes |
|---|---|---|---|---|
| **‹unnumbered A›**<sub>w-readphase</sub> | **`decode_causes`' ALL-CAUSE SET IS NOT A DEPTH EITHER — IT UNDER-REPORTS THE HEAD CLASS'S ARITY BY UP TO 725×** | **MEASURED, by lifting rather than by re-asking.** All 819 `gl-stop-26-introduced` TUs report arity **2**; lifting the clause sends **2** of 819 to the co-reported `body-out-of-class` and **817** to three clauses the diagnostic reports as *not evaluated*. Lift those and `unclaimed-gl-symbol` goes 2 → 735, `locally-defined-callee` 1 → 725, `shape-token-unresolved` 0 → 715 | 819 TUs · 3 hidden clauses · arity 2 → 4 · `match` +0 · `fnbyte-exact` **−65** | rungs/2026-08-14-readphase.md §2, §2.1 · #2505 · #3062 | `diag.rs`'s module doc **states** the boundary (*"reported as not evaluated"*); what is new is its size. The standing caveat *"a first-blocker count is NOT a distance"* should read *"neither is the cause set"* |
| **‹unnumbered B›**<sub>w-readphase</sub> | **THE POISONED SINK IS FAIL-CLOSED BUT NOT EMISSION-NEUTRAL: ONE TOKEN COSTS 2,694 BYTE-EXACT FUNCTIONS AND ONE MATCHING TU** | **MEASURED on the committed instrument, no `crates/` change.** `C2RS_SINK_CHAIN=op:41`: emitted-blocked 113,612 → 116,306, **all +2,694 absorbed by `expr-jump`** (298 → 3,731), poison **0** — zero decode distance bought — and `match` 25 → 24 (`src/xdk/nuispeech/mmio.cpp`), `fnbyte-exact` 35,734 → 33,040. At 22 tokens: `match` 25 → **18** (7 TUs by name), `fnbyte-exact` **−5,949** | 1 token · −2,694 fns · −1 TU · 22 tokens · −5,949 fns · −7 TUs | rungs/2026-08-14-readphase.md §1 finding 5, §3 · `expr.rs` `chain_sink` · #660 | The sink's doc says it *"cannot move one obj byte even when it is ON"*. True of **wrong emits**, false of **removals** — the mechanism is that `parse_expr` is called **by** the shape recognizers. `op:41` is in `ladder.py`'s SEED and SCAFFOLD, so **every published per-TU ladder was climbed through it**; the rung counts stand, any neutrality reading does not |
| **‹unnumbered C›**<sub>w-readphase</sub> | **THE WHOLE EXPRESSION LAYER'S DECODE CEILING IS 76,041 OF 120,456, AND THE RESIDUE IS SIX REFUSALS, NOT A LONG TAIL** | **MEASURED.** All 47 pinned opcodes + `type` + `convert` + `intrinsic`, 870 TUs graded: **76,041 (63.1 %)** reach the function tail `4F 12`, 0 reach any other `4F`, 6 `expr-chain-short`. Residue **44,409 over 68 keys**, of which **39,280 (88.5 %)** is six refusals: `5D`/`5E` **13,158** (one refusal — `<varint> <varint>`, unspellable by any `SkipForm`), `0x64` **8,000**, the statement layer **7,903**, the compound-assign family **5,269**, `0x9A` **2,674**, and `0x00` **2,276** which is **not an opcode** but the 64-bit-literal desync | 76,041 / 120,456 · 72 keys · residue 44,409 / 68 keys · 6 refusals = 88.5 % | rungs/2026-08-14-readphase.md §4, §4.1, §4.2 · IL_DECODE_REACH.md §3, §4 · #1465 | The terminal was **ambiguous and was resolved**, not assumed: `noform-0x4F` is `ladder.py`'s `TAIL` but `Line4F` refuses every non-`4F 01`, and `bundle.rs:292` lists five family members. A scratch split (reverted) put **76,041 of 76,041** on `4F 12` |
| **‹unnumbered D›**<sub>w-readphase</sub> | **`type`, `convert` AND `intrinsic` ARE WORTH EXACTLY ZERO AT THE CEILING — §14.1's TOP-RANKED STEP, RE-MEASURED ON THE EMITTED POPULATION** | **MEASURED: three specs, three BYTE-IDENTICAL scans** (`md5 87b206ef61865bf18645e7cd5d0986ea`). `op:B9`'s `TokType` form consumes a LOAD's TYPE whatever it is, so once the opcode table is granted the type gate is never reached. On the emitted population at base the gate is **2,389 of 113,612 (2.1 %)** over **22 sharded keys**, against **122,736 of 1,705,627 (7.2 %)** over all bodies — a **51×** census-to-emitted gap | 3 identical scans · 2,389 vs 122,736 · 51× · 22 shards | rungs/2026-08-14-readphase.md §1 finding 4, §5.1 · IL_STMT_GRAMMAR.md §14.1 · GAPS.md §6 | Both of `GAPS.md` §6's warnings fire on the one step §14.1 ranked **first**: the population is the census and not the emitter, and the key is **sharded on a per-TU-allocated type id** |
| **‹unnumbered E›**<sub>w-readphase</sub> | **`CEILING.md` §6's "130,575 OF 139,792 BLOCKED AT THE IL READER" IS STALE, AND SO IS THE "93 %" FOUR LANES CONVERGED ON** | **MEASURED at `6f2c7c41`:** `fnbyte-refused-parse` **113,612**, `fnbyte-denominator` **162,049** (the published pair is at `85e180d4` against 178,977). The emitted widening order is **615 keys summing 113,612**, not §3.1's 648/130,575, head `expr-op-0x27` **22,407** not 22,373. The reader's share is **99.2 %** of refusals and **70.1 %** of the denominator — **not 93 %** | 113,612 / 162,049 · 615 keys · 99.2 % · 70.1 % | rungs/2026-08-14-readphase.md §0 · CEILING.md §6 · STRATEGY_REVIEW §3 | Quoted stale by `CEILING.md` §6, `STRATEGY_REVIEW` §3 item 2, and this lane's own dispatch brief. **The direction of the staleness matters**: the reader's share of *refusals* is higher than 93 %, its share of the *denominator* lower |
| **‹unnumbered F›**<sub>w-readphase</sub> | **§14.2's STEP 4 IS FALSIFIED BY ITS OWN CLAUSE: `0x14` IS OBSERVED** | **MEASURED.** §14.2 step 4: *"`0x14` is unobserved and must reject."* At the sink ceiling `expr-op-0x14` reads **1** emitted function. The step's whole family is worth **45** emitted-blocked at base and **5,269** only once everything above it is granted — so it is neither *"nearly free"* in position nor closed as a vocabulary | 1 function · 45 at base · 5,269 at the ceiling · 11 opcodes = 1 production | rungs/2026-08-14-readphase.md §5, §4.2 · IL_STMT_GRAMMAR.md §14.2 | Steps **1, 2 and 3 are DONE** — every named gain bucket (`body-0xAD`/`0xB3`/`0x4F`/`0x53`/`0x29`, `fn-tail-0xB9`, `call-token-0x26` 80,284 → **10**) reads 0 or near it. The doc should say so; it currently reads as a plan |
| **‹unnumbered G›**<sub>w-readphase</sub> | **FOUR ENV-GATED LADDER INSTRUMENTS LIVE ONLY IN `expr.rs` DOC COMMENTS AND ARE REACHABLE BY NO `docs/` GREP AND NO BOARD-TOPIC SEARCH** | **OPEN — NAMED, NOT REPAIRED.** `C2RS_SINK_CHAIN` (#660), `C2RS_SINK_REL` (#420), `C2RS_SINK_BRANCH` (#440, three levels) and `C2RS_SINK_OFF_ADD_ARG` (#143) are the repo's answer to *"what is the successor of this blocker"*. **No file under `docs/` documents any of them as an instrument**; this lane registered PREREG L6 (*"the successor is not computable without implementing the production"*, p = 0.70) after ~2 h of orientation and found them by reading `expr.rs` | 4 instruments · 0 `docs/` entries · 1 prereg registered against their existence | rungs/2026-08-14-readphase.md §8.1 item 3 · `expr.rs` `chain_sink`/`rel_sink_enabled`/`branch_sink`/`off_add_sink_enabled` | The fix is one table in `docs/IL_DECODE_REACH.md` or `docs/GAPS.md` §6 naming the four, their poison discipline, and **board B's non-neutrality**. Deliberately not written here: `w-readphase` is a characterization lane and this is a docs seam a peer may hold |

---

## 10. Gate evidence

| lane | result |
|---|---|
| `cargo test --workspace --release --no-fail-fast` | see §10.1 |
| `scripts/gate.sh --jobs 4 --require-graded` | see §10.1 |
| `scripts/board_audit.sh` | see §10.1 |
| 878-TU workload scan, **base** | `match 25 · mismatch 0 · codegen-gap 0 · vocab-gap 845 · capture-fail 8 · frontier 2`, 370 keys, 878 rows / **870 graded** |
| 878-TU workload scan, **tip, after every scratch lift was reverted** | **370 keys, 0 differing**; `git status --porcelain crates/` = **0 lines** |

---

## 11. Found and not taken

1. **A new `SkipForm` for `<varint> <varint>` — 13,158 emitted functions, the
   largest single residue item, and it is an INSTRUMENT change, not a port
   change.** `chain_skip_form`'s absent-list says the EH count trailers `5D`/`5E`
   are unpinnable because *"no `SkipForm` variant can spell `<varint> <varint>`"*
   — which is a statement about the enum, and `EH_RECORDS.md` §7.1 already gives
   the width. Adding the variant is decode-only, poisoned, and would move the
   ceiling from 76,041 by up to 13,158 with **zero** risk to the byte judge.
   Not taken: this lane's `crates/` delta is required-zero.
2. **`0x00`'s 2,276 are an instrument DEFECT masquerading as a construct
   (board #1465).** The 64-bit-literal escape is 8 bytes and `read_varint`
   reads 4, so the cursor lands *inside* the payload. No pin can fix a desync.
   Sized here for the first time on the emitted population.
3. **The de-acceptance mechanism has never been measured on a REAL widening.**
   Everything in §1 finding 5 is measured through the sink. Whether a widening
   that *accepts* instead of poisoning also de-accepts is the question that
   decides §6's required-zero columns, and it needs one hatched (unpoisoned)
   run — `work/w-front3/hatch.py`, which `gate.sh` reports as `HATCH-STALE` on
   this tree (board #1389). **That is the next lane**, and it is a construct
   rung, not a TU lane.
4. **`gl-stop-name-too-far` goes 11 → 106 across ladder rung 2** — a clause whose
   population is *created* by lifting two others. Not sized further; it is the
   cleanest small instance of §2.1's masking for a lane that wants one.
5. **`op:BD` alone moves the class-wide reach 14,479 → 40,530**, the largest
   single rung in §3 by 3.5×, and the CALL token is not a step in §14.2 at all.
6. **Not taken and deliberately so: any reader widening.** The dispatch brief
   forbade it and §7 says why it would have been wrong even unforbidden.
