# WB_SCHEDCHK — FINDINGS for lane `w-sched` (decision 21's R7 row, re-dispatched)

> **PROVENANCE — DISASSEMBLY-DERIVED.** Every address is an absolute VA in
> `compilers/X360/16.00.11886.00/c2.dll`, sha256
> `c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`, verified
> by `dump_sched_tables.py`'s own digest check in this lane's run. See
> [`DISCLOSURE.md`](DISCLOSURE.md). Whitebox analysis is authorized and
> encouraged (`CLAUDE.md`, owner, 2026-08-17).

**Lane** `w-sched` · **kind** characterization · **Fixtures** none ·
**Census** +0 · **reach** 0 · **`crates/` bytes 0** ·
**`DISCLOSURE.md` rows 0** · **`gate.sh` rows added 0** (`#3691`) ·
**Board** #3725–#3730 · prereg
[`WB_SCHEDCHK_PREREG.md`](WB_SCHEDCHK_PREREG.md), frozen as the branch's first
commit **`7d75d471f`**, before the tap was run and before a byte of the
scheduler band was read on this branch.

---

## 0. THE VERDICT

> ### **R7 WAS ALREADY DISCHARGED. THIS LANE IS A RE-DISPATCH OF A SPENT READ.**
>
> `docs/rungs/2026-08-23-w-read-r7.md`, outcome `built`, board **#3433**–**#3436**,
> five days before decision 21 funded it again. Its brief was word for word
> this one's. Registered as P0.1 **before** any measurement and **HIT**.

> ### WHAT WAS GENUINELY UNRUN, AND IS THIS LANE'S DELIVERABLE
>
> R7 asked the discriminability question **in aggregate** — *"a simulator that
> returns its input scores 98.9 %"* — and answered it in prose. Nobody asked it
> **clause by clause**. Asked that way it has a different answer, and the
> difference is not a refinement:
>
> **`WB_SCHEDCONF`'s 1,461/1,461 pins 4 of the region rule's 9 clauses. Five
> are unconfirmable, and the most-fired clause in the whole rule is one of
> them — it fires on 1,428 of 2,889 walks and on ZERO of the 1,461 that grade.**

> ### AND `WB_SCHEDCONF` §3.4, `P_DAG` §6 AND BOARD `#3434` ARE WRONG ABOUT IT
>
> All three say the head special case *"fires on **1,121** of the 1,461 graded
> pairs — the single most common path, and it was undocumented."* **The 1,121
> belongs to a different clause** — it is `grade_regions.py`'s `excl-0x17/0x30f`
> **exit** count, the region *terminator* at `0x10be5d8b`. The head case at
> `0x10be5d55` is **not an exit**, has **no row in that histogram**, and its
> firing count had never been printed by any instrument in this repo. It is
> printed here. It is **0** on the graded population.

---

## 1. Scorecard against the frozen prereg

| # | prediction | result |
|---|---|---|
| P0.1 | R7 already ran; the first deliverable is a staleness banner | **HIT** |
| P2.1 | ≥ 4 clauses GREEN, and they are the four never-fired exits + the cap | **MISS — in the informative direction.** 8 GREEN, and two of them are the *most-fired* clause (§3.2) |
| P2.2 | the tap confirms a **ray** `cap ≥ k`, `k ≤ 16`, read constant ≥ 5× beyond | **HIT** — `k = 13`, slack **6.2×** (§3.3) |
| P2.3 | separate UNCONFIRMABLE-BY-CORPUS from -BY-CONSTRUCTION | **HIT**, and both classes are populated (§4) |
| M-HEAD-DROP | RED ≥ 1000 | **MISS** — GREEN. The most informative row of the grid |
| M-HEAD-ANY | RED < 100 | **HIT** — 38 |
| M-HEAD-OP | RED ≈ DROP | **MISS** — GREEN |
| M-12-EXCL / M-1B-EXCL | RED ≈ 204 / ≈ 136 | **HIT** — 204 / 136 exactly |
| M-14-EXCL / M-19-INCL / M-14-DROP / M-19-DROP | GREEN | **HIT** ×4 |
| M-12-DROP / M-1B-DROP | RED | **HIT**, and smaller than the clause's fire count (§3.4) |
| M-17-DROP / M-17-INCL | RED ≈ 1121 | **HIT** — 1,121 exactly |
| M-17-ANY | RED, conf 0.50 | **MISS** — GREEN (§3.5) |
| M-CAP-GE | GREEN | **HIT** |
| P3.1 | no DAG-node field is emitted by any tap site ⇒ no priority/latency clause is `[O]` | **HIT** (§5) |
| P3.2 | the order channel confirms one bit, witnessed by 3 of 357 | **HIT**, and the channel is now *sized*: **8 positions of 3,015** (§6) |
| P3.3 | ≥ 1 clause unfalsifiable in both senses, marked as neither | **HIT** — the head case (§3.2) |
| P3.4 | NO-CHANNEL clauses outnumber `[O]`-confirmed by > 3:1 | **HIT** — 26 vs 4, **6.5:1** (§4) |
| P4.1 | §8.2 is wrong about the record | **HIT** (§7.1) |
| P4.2 | §8.2 is wrong about the time | **HIT** (§7.2) |
| P4.3 | 0.45 a tuple→node back-pointer exists | **the 0.55 branch: it does not** — and a *different*, cheaper mechanism does (§7.3) |
| P4.4 | corrected price is a shape, > 0.5 d, no wall-clock | **HIT** (§7.4) |
| P7 | reach 0, census +0, `crates/` 0 bytes, `DISCLOSURE.md` +0, `gate.sh` +0 | **HIT** |

**16 HIT · 4 MISS · 1 split.** Every MISS is a GREEN this lane predicted RED,
i.e. the tap turned out to confirm **less** than the prereg expected in every
direction it was wrong. That is a one-sided error and it is worth carrying
forward: *this lane, like R7, was too optimistic about what the measurement
could see, and never the reverse.* R7 recorded the same bias
(`WB_SCHEDCONF` §1, *"too low on the reads and far too high on the
measurements"*). **Two independent lanes, same direction, on the same
instrument.**

---

## 2. Controls, both live

**C-A — the population is R7's, pinned by NAME and COUNT.** Re-running
`grade_regions.py` on this branch's own `stage snap --limit 60`:

```
GRADED PAIRS 1461   HIT 1461   MISS 0   UNGRADED 1368
excl-0x17/0x30f 1121   incl-cat-12 204   incl-cat-1b 136
NEVER EXERCISED: cap>0x50, incl-cat-14, excl-cat-19, end-of-list
```

Every figure identical to `WB_SCHEDCONF` §3.1/§3.3. `grade_reorder.py` likewise
reproduces **456 pairs / 5 reordered / 1.10 %** and the same length
stratification. The environment is validated, so the colours below are not
void.

**C-B — a mutant that dies.** 16 RED rows across the grid and the cap sweep.
The grid is not decoration (`#3336`).

**A third check nobody registered, and it is the one that made §3.2
findable.** The head-clause census is an *arithmetic identity*, not an
estimate:

```
2,889 walks − 60 last-of-fixture      = 2,829 candidate pairs
2,829 = 1,368 UNGRADED + 1,461 GRADED
1,428 walks fire the head clause; 1,428 − 60 = 1,368 == UNGRADED, exactly
```

The head clause's firing set and the instrument's graded set are **disjoint,
and exhaustively so** — every walk that fires it is discarded by the instrument
check, and every walk the instrument keeps fails to fire it. Not a sampling
accident: a structural property of the two rules, closing to the unit.

---

## 3. The mutation grid — which clauses the 1,461/1,461 actually pins

Instrument: [`scripts/mutate_regions.py`](scripts/mutate_regions.py). One clause
of the transcribed `FUN_10be5d4b` is changed at a time; the same frozen stream
is re-scored. **GREEN = the tap cannot separate the clause from its negation.**

```
;  id              mutation                                 hit  miss  d(miss)  verdict
  M-HEAD-DROP     head special case removed                 1461     0       +0  GREEN
  M-HEAD-ANY      head taken for ANY opcode                 1423    38      +38  RED
  M-HEAD-OP       head opcode 0x30f -> 0x30e                1461     0       +0  GREEN
  M-12-EXCL       cat 0x12 inclusive -> exclusive           1257   204     +204  RED
  M-14-EXCL       cat 0x14 inclusive -> exclusive           1461     0       +0  GREEN
  M-1B-EXCL       cat 0x1b inclusive -> exclusive           1325   136     +136  RED
  M-19-INCL       cat 0x19 exclusive -> inclusive           1461     0       +0  GREEN
  M-12-DROP       cat 0x12 not a stop at all                1315   146     +146  RED
  M-14-DROP       cat 0x14 not a stop at all                1461     0       +0  GREEN
  M-1B-DROP       cat 0x1b not a stop at all                1375    86      +86  RED
  M-19-DROP       cat 0x19 not a stop at all                1461     0       +0  GREEN
  M-17-DROP       the 0x17/0x30f clause removed              340  1121    +1121  RED
  M-17-ANY        cat 0x17 stops regardless of opcode       1461     0       +0  GREEN
  M-17-INCL       0x17/0x30f exclusive -> inclusive          340  1121    +1121  RED
  M-CAP-GE        cap compare `>` -> `>=`                   1461     0       +0  GREEN
```

### 3.1 What IS pinned — four clauses, and they are pinned hard

`0x12` inclusive (204 cells), `0x1b` inclusive (136), the `0x17`/`0x30f`
exclusive terminator (1,121), and the *existence* of a head test at all
(38, from the `ANY` direction). Nothing about these is in doubt: each has a
mutant that dies on hundreds of cells, and the inclusive/exclusive polarity
that `WB_SCHEDCONF` §3.4 called *"an off-by-one on every region boundary"* is
the single best-confirmed thing in the whole scheduler model.

### 3.2 What is NOT pinned — and the biggest one is the clause the record calls the most common

**`M-HEAD-DROP` is GREEN. Deleting the head special case entirely changes not
one of the 1,461 cells.** So is `M-HEAD-OP`: the constant `0x30f` at
`0x10be5d55` can be replaced with `0x30e` — with any value — and the tap does
not notice.

The mechanism, measured rather than argued:

```
walks whose head opcode is 0x30f, over ALL 2,889 walks        1,428
walks whose head opcode is 0x30f, over the 1,461 GRADED       0
of the cat-0x17 tuples in the corpus, how many have op 0x30f  2,889 of 2,889
graded walks whose head is a stop tuple of any kind           79
```

A region that ends at the exclusive `0x17`/`0x30f` terminator leaves the next
region's head **on** that `0x30f` — so the head clause fires on exactly the
walks that begin a run's final region, and those walks' successors are the
*first* walk of the next run, which is **longer**, so `grade_regions.py`'s
instrument check (B must be a strict tail of A) discards them. The sample
`(lenA, lenB)` pairs are `(4,7)`, `(4,16)`, `(4,15)` — B longer than A in every
case.

**This is the sharpest form of "absence is not evidence" this repo has
recorded.** The instrument check is *correct* — it is what makes the other four
clauses trustworthy — and its correctness is exactly what makes this clause
unobservable. A 100.00 % that a lane worked hard to earn is silent on the
rule's most-executed branch, and the record does not say so anywhere.

### 3.3 The cap constant is a RAY, not a value

| cap | 80 | 40 | 20 | 16 | 15 | 14 | **13** | 12 | 10 | 8 | 4 | 2 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| verdict | G | G | G | G | G | G | **G** | R(2) | R(4) | R(15) | R(97) | R(355) |

**The tap confirms `cap ≥ 13`. The read constant is `0x50` = 80. Slack 6.2×.**
`WB_SCHEDCONF` §3.2 promoted this row from *unmeasured* to *measured-and-slack*
on the evidence *"0 of 1,461 regions reach it"*; that is true and it is a
statement about the corpus, not about the constant. **Any port using any cap
from 13 upward is byte-identical on every cell this repo can currently grade.**

This is `#3388`'s failure mode — a constant whose control is structurally
incapable of exercising it — in a second place. The difference, and it matters,
is that `LABEL_SEED_GAP = 9` was **fitted** and this `80` was **read**: the read
is almost certainly right, and the point is only that *the measurement does not
say so*. A port adopting 80 should cite `0x10be5d66`, never the 1,461.

### 3.4 Clause redundancy — the fire count OVERSTATES the confirmation

`incl-cat-12` fires on 204 cells, but removing `0x12` from the stop set
entirely loses only **146**: on 58 of the 204 a *different* clause stops the
scan at the same index anyway. `incl-cat-1b` fires 136 and loses **86**; 50 are
covered twice. So `grade_regions.py`'s clause histogram is an upper bound on
per-clause evidence, in both directions at once: it over-counts clauses that
are shadowed by others (`0x12`, `0x1b`) and it can report **zero** for a clause
that fires 1,428 times (§3.2).

**Clause coverage is not clause confirmation.** That sentence is the general
result and it is not confined to this rule.

### 3.5 The opcode half of the `0x17` test is unconfirmed

`M-17-ANY` — stop at category `0x17` regardless of opcode — is GREEN, because
**all 2,889 category-`0x17` tuples in the corpus carry opcode `0x30f`**. The
two-term test at `0x10be5d8b` is graded as a one-term test. If some other
opcode ever appears at category `0x17`, nothing measured here says which way
c2 goes.

---

## 4. THE CLAUSE DISCRIMINABILITY TABLE

Four verdicts, as preregistered. **`[O]`** = graded by the tap and a mutation
dies. **CORPUS** = a channel exists, this population has no separating cell.
**CONSTRUCTION** = no corpus can separate them. **NO CHANNEL** = the tap emits
no observable any negation could move; `[R]` forever by this instrument.

### 4.1 The region rule (`P_DAG` §2's region row / `0x10be5d4b`)

| clause | at | verdict | evidence |
|---|---|---|---|
| `0x12` stops INCLUSIVE | `0x10be5d72` | **`[O]`** | M-12-EXCL 204 · M-12-DROP 146 |
| `0x1b` stops INCLUSIVE | `0x10be5d83` | **`[O]`** | M-1B-EXCL 136 · M-1B-DROP 86 |
| `0x17`+`0x30f` stops EXCLUSIVE | `0x10be5d8b` | **`[O]`** | M-17-DROP / M-17-INCL, 1,121 each |
| a head test exists | `0x10be5d55` | **`[O]`**, weakly | M-HEAD-ANY, 38 |
| the head test's **opcode** (`0x30f`) | `0x10be5d55` | **CORPUS** | M-HEAD-OP GREEN; 0 of 1,461 fire |
| the head case's **effect** | `0x10be5d55` | **CORPUS** | M-HEAD-DROP GREEN; 1,428 firings, all discarded |
| the `0x17` test's **opcode** half | `0x10be5d8b` | **CORPUS** | M-17-ANY GREEN; 2,889/2,889 are `0x30f` |
| `0x14` is a stop, and INCLUSIVE | `0x10be5d76` | **CORPUS** | never fires; both mutants GREEN |
| `0x19` is a stop, and EXCLUSIVE | `0x10be5d7f` | **CORPUS** | never fires; both mutants GREEN |
| the cap's **value** `0x50` | `0x10be5d66` | **CORPUS** | ray `≥ 13`, 6.2× slack |
| the cap's **strictness** (`>` vs `>=`) | `0x10be5d66` | **CORPUS** | differs only at `count == cap`, unreached |
| the cap's **signedness** (`jg`) | `0x10be5d66` | **CONSTRUCTION** | the counter is 0-initialised and only incremented, so signed and unsigned agree on every reachable input — no corpus separates them |
| end-of-list exit | — | **CORPUS** | never fires |

**4 `[O]` · 8 CORPUS · 1 CONSTRUCTION.**

### 4.2 The priority function (`P_DAG` §3, R7's six terms)

Every row is **NO CHANNEL**. The tap's per-tuple row is
`{opcode, cat, flags, cc}` read from the **tuple** record (`stagetap.c`
`tap_walk_tuples`, `b+4`, `b[8]`, `b[9]`, `b[10]&0x1f`); the optional levers add
raw **tuple** bytes (`C2RS_STAGE_RAW`), an operand/symbol walk
(`C2RS_STAGE_OPS`) and a whole-function **tuple** walk (`C2RS_STAGE_FUNCWALK`).
**No tap site emits a single field of a DAG node**, so none of these terms has
any observable except through its effect on final tuple order — which is §6's
8 positions.

| clause | at | verdict |
|---|---|---|
| `height << 13`, `w[1] = 13` | `0x10be5eec` | NO CHANNEL |
| `fanout << 8`, unmasked, `w[2] = 8` | `0x10be5f35`/`0x10be5f39` | NO CHANNEL — and `node+0x26` itself is unobservable (R7 P2.2, still UNGRADED) |
| `bit2 << 10`, `w[5] = 10` | `0x10be5f1b` | NO CHANNEL |
| `w[0]`, `w[3]` DEAD (`-1`, term `>>1`) | `0x10be5ed6`/`0x10be5f03` | NO CHANNEL |
| bit 0 = critical-path membership | `0x10be5e5b`/`0x10be5ec3` | NO CHANNEL |
| `w[6]` worth 1, gated on typeword `0x5000` | `0x10be5f88` | NO CHANNEL |
| `w[7]` dynamic unit bonus 0..7 | `0x10c1bbf6` | NO CHANNEL |
| opcode `0x2b8` ⇒ `0xffffffff`, maximal | — | NO CHANNEL |
| the compared field is `+0x3c`, not `+0x38` | `0x10be6046` | NO CHANNEL |
| `node+0x44` truncated to 16 bits, unsigned | `0x10be5fe8`/`0x10be5fef` | NO CHANNEL |
| ready list fully re-sorted every cycle | `0x10be6046` | NO CHANNEL |
| tie-break `node+0x44` ASC | `0x10be5cea` | NO CHANNEL |

### 4.3 The latency model (`P_DAG` §5) and the cycle model

| clause | verdict | note |
|---|---|---|
| the 9 published latencies | **NO CHANNEL** *by the tap* | see the box below |
| tag dispatch; cell `(1,8)` = `-2` splits 5/2 on `edge+0x19` bit 1 | NO CHANNEL | `edge+0x19` is not observable at all |
| anti-deps 0 structurally (`test [ecx+0x10],0x21`) | NO CHANNEL | |
| `CLASSTAB` stride 12 at `0x10b221d0`, 660/661 vs the machine table | NO CHANNEL | |
| issue width `DAT_10c3cf98 ∈ {2,4}` | NO CHANNEL | |
| ≤ 2 nonzero-unit instructions per cycle | NO CHANNEL | |
| `node+0x40 <= cycle + slack(unit)` | NO CHANNEL | |
| `+15` microcode / `+40` store-forward penalties | NO CHANNEL | |
| the schedule is ITERATED (`0x10c1bdff`) | NO CHANNEL | R7 §7.2: the tap *"would see the fixed point, never the passes"* |

> ### ⛔ THE §5 HALF OF R7's HEADLINE NEVER MET THE TAP
>
> `WB_SCHEDCONF` §0 files *"the edge-latency mechanism reproduces 10 of 10 of
> `P_DAG.md` §5's published latencies from the raw image bytes"* under the
> heading **"THE STRUCTURAL MODEL SURVIVED THE TAP"**. Re-run here:
> `dump_sched_tables.py <c2.dll> --verify` takes **the image and nothing else**
> as input — no snap, no obj, no fixture — and prints `10/10`.
>
> It is a **self-consistency check between two readings of the same image**: the
> prose in §5 against the bytes at `0x10c3c1a8`/`0x10b221d0`. That is worth
> having and it is worth reproducing (it does, on this tree). It is not an
> observation, it does not promote `[R]` to `[O]` under
> [`ref/README.md`](ref/README.md)'s legend (*"confirmed against a real obj or
> `/FAsc` listing"*), and filing it under a heading with the word *tap* in it
> invites exactly that reading. **The latency model is `[R]`.**

**Totals: `[O]` 4 · CORPUS 8 · CONSTRUCTION 1 · NO CHANNEL 26.**
**NO CHANNEL outnumbers `[O]` 6.5 : 1** (P3.4 predicted > 3:1).

---

## 5. What the tap emits, exhaustively — the basis for every NO CHANNEL row

Read from `c2host/stagetap.c`, not assumed:

* **8 sites** (`g_sites`): `sched1` `0x10b7dc9f`, `globregs` `0x10b7dcb7`,
  `sched2` `0x10b7dcde`, `color` `0x10b7dcf6`, `sched3` `0x10b7dd1d`,
  `sched0` `0x10b7e00c`, **`region` `0x10be643e`**, `after0` `0x10b7e701`.
* **The only payload with a tuple pointer is `region`**, and it takes it from
  the region finder's own `ecx` argument. The six phase sites carry the
  **function record**; `after0` likewise.
* **Every emitted row is a field of a TUPLE**: `+0x4` opcode, `+0x8` category,
  `+0x9` flags, `+0xa & 0x1f` cc; optionally the first *n* raw tuple bytes, and
  optionally the operand chain at `+0x28`/`+0x2c` with its symbol records.
* **No site emits a DAG node, an edge, a cycle number, a priority, a height, a
  fanout, a unit or a reservation counter.** There is no code in `stagetap.c`
  that dereferences the DAG at all.

Consequence, stated plainly because it is the load-bearing one:
**everything in `P_DAG` §3 and §5 is observable by this instrument only through
the composed final tuple order.** §6 measures how much of that there is.

---

## 6. The order channel, SIZED

Instrument: [`scripts/grade_final_order.py`](scripts/grade_final_order.py), over
`C2RS_STAGE_FUNCWALK=1 c2rs stage snap --limit 60`, comparing `sched0` (run 4's
input) with `after0` (`0x10b7e701`, the first call after `0x10b7df57` returns).

```
functions paired 357   UNCHANGED 354   REORDERED 3   (0.84%)
excluded: 0 phase-unpaired, 0 tuple-multiset-changed

  il_intrinsic_bits.cpp   ?b_bswap64@@YA_K_K@Z      19 tuples   4 positions moved
  il_sy_locals.cpp        ?addr_taken@@YAHH@Z       16 tuples   2 positions moved
  il_sy_locals.cpp        ?mixed_addr@@YAHH@Z       21 tuples   2 positions moved

TOTAL DISCRIMINATING POSITIONS: 8, over 3,015 tuples walked.
```

R7's 3-of-357 reproduces exactly, and the three functions are the same three
(§4.1 named them by ordinal and length: fn12/19, fn8/16, fn9/21 — the names and
lengths agree). **The number R7 did not publish is the last line: 8.**

> **`0.27 % of walked tuple positions is the entire order channel of this
> instrument at the final schedule.** Two order models are separated here **only
> if they disagree on one of those 8 positions.** Not "if their agreement rates
> differ" — an agreement rate over 3,015 positions of which 3,007 are free is a
> measurement of the corpus, which is `WB_SCHEDCONF` §4.2's point stated in the
> units a follow-up lane actually needs.

Two consequences a future lane should not have to re-derive:

1. **The 12 priority clauses of §4.2 are pairwise indistinguishable on 8
   positions in any model family with more than 8 bits of freedom** — and the
   published family has at least the six weights, the tie key, the field
   choice, the truncation, the re-sort and the bonus. There is no experiment
   design over this corpus that separates them; the shortfall is not
   statistical.
2. **`WB_SCHEDCONF` §8.1's *"build a population that reorders"* is therefore
   not an optimisation, it is the precondition.** With 8 positions, a model
   with *k* free binary decisions is at best `8/k`-determined.

---

## 7. `WB_SCHEDCONF` §8's follow-up prices, checked

Checked because `w-f0price` carried §8.1 into F0's price as one of two
**UNPRICED** terms (`#3716`, sub-item 4). **This lane does not re-price F0**
(decision 21's hard limit); it checks the two figures §8 published and reports
the bearing.

### 7.1 §8.2 is wrong about the record (P4.1 HIT)

> *"Expose `node+0x26` and `node+0x38` in the tap (≈0.5 day). **Three fields in
> `tap_walk_tuples`**."*

`+0x26` and `+0x38` are **DAG node** fields. `tap_walk_tuples` walks **tuple**
records — a different structure reached by a different pointer. The two are
joined by `node+0x1c`, read at two independent sites: written at
**`0x10b327de`** (`mov [esi+0x1c],edi`, node create) and read at
**`0x10c1c1ea`**/**`0x10c1c1f1`** (edge-latency, `mov esi,[eax+0x1c]` then
`mov ebx,[esi+4]` = the tuple's opcode). The direction is **node → tuple**.
There is no field of the tuple that is `node+0x26`.

### 7.2 §8.2 is also wrong about the time (P4.2 HIT)

`tap_walk_tuples` is called from **one** place — the `region` site, at
**region-finder ENTRY** (`0x10be643e`). `schedule_run`'s body is
`find_region → build_dag → prioritise → emit_cycles`, and `build_dag`
(`0x10b328da`) calls the DAG reset `0x10b32008` at **`0x10b328e8`**, fourteen
bytes in. **At the hook, region *k*'s DAG does not exist.** No edit to
`tap_walk_tuples` can read a field of a node that has not been created.

### 7.3 A cheaper mechanism than the one §8.2 describes — and it is not a back-pointer (P4.3, the 0.55 branch)

`FUN_10b327cd` writes **nothing** into the tuple: `edi` (the tuple) is never a
store destination anywhere in its 158 bytes. **There is no tuple→node
back-pointer.** But an enumeration does not need one, because the DAG is
reachable from a global and each node names its own tuple `[R]`:

| what | where | how |
|---|---|---|
| the DAG object | `DAT_10c435e0` | set by the reset `0x10b3200a`; also read at `0x10b33153`, `0x10c1c44b` |
| node list head / tail | `[dag+0x00]` / `[dag+0x04]` | written `0x10b3315e` / `0x10b3316b` from `DAT_10c435dc` / `DAT_10c435d8` |
| node count | `[dag+0x24]` | written `0x10b33179` from `DAT_10c435c8` (bumped per node at `0x10b32860`) |
| the intra-region node link | `node+0x04` | the priority pass's reverse walk: `mov eax,[ebx+4]` `0x10be5e22`, `mov eax,[eax+4]` `0x10be5e53`, terminating on `[head+4]` at `0x10be5e56` |
| node → its tuple | `node+0x1c` | `0x10b327de` (write), `0x10c1c1ea` (read) |
| the fields §8.2 wants | `node+0x24` preds · `node+0x26` fanout · `node+0x38` priority · `node+0x3c` working · `node+0x40` earliest · `node+0x44` index · `node+0x48` height · `node+0x4a` maxlat · `node+0x4c` unit · `node+0x4e` flags | all zeroed/assigned in `FUN_10b327cd` `0x10b32844`–`0x10b3284f`, `0x10b3282b`, `0x10b32839` |

**And the existing `region` site can reach it with no new hook, off by one
region.** The DAG reset runs *inside* `build_dag`, so at region *k*'s
find_region entry `DAT_10c435e0` still describes region *k−1*, complete —
built, prioritised and scheduled. Fanout for region *k−1* is readable at the
hook that already exists.

*Caveats, because this is `[R]` and the difference matters:* prioritise's `ecx`
is identified with `DAT_10c435e0` from the matching `+0`/`+4` layout and is not
separately confirmed; and whether the node pool is recycled before the next
`find_region` is **not** established here. Both are settled by building the
walker and watching the join `node+0x1c → tuple` reproduce the tuple rows the
same site already emits — a confirmation probe that can fail, which is the only
kind worth registering.

### 7.4 The corrected shape of §8.2's price (P4.4 HIT)

Not *"three fields in `tap_walk_tuples`"*. It is: **a new walker over a second
record type, reached through a global, with a join field, a plausibility fence
per node, a bounded node cap, a canonical-stream schema extension in
`crates/c2-reference/src/stage.rs`, a determinism/neutrality re-run, and a
confirmation probe on the join** — plus, if region *k* rather than *k−1* is
wanted, a new tap site after `0x10be5df6` returns. **Strictly more than §8.2's
0.5 d.** No wall-clock figure is published here, for `WHITEBOX_LEVERAGE` §3.1's
reason and in the same spirit as `#3716`'s own refusal.

**Bearing on F0, and nothing more.** `#3716` prices F0 at *≥ 10 raw sub-lanes +
2 UNPRICED*, with §8.1's ≈1 d population build as one of the UNPRICED terms.
This lane changes neither number. What it adds: §8.1's item now has a **measured
target** — 8 discriminating positions is what the current corpus supplies, so a
population build has something to be scored against — and §8.2, the *other*
follow-up, was **under-priced at the record level**, which pushes the same
direction `#3716` already reported. **The figure stays `w-f0price`'s.**

---

## 8. WHAT THIS LANE'S EVIDENCE IS STRUCTURALLY INCAPABLE OF SHOWING

**8.1 A GREEN row is a statement about THIS instrument on THIS corpus.** It
never says the clause is false, and it never says c2 does not do it. Nine of
the thirteen region clauses are read from the disassembly and are very likely
right; what is measured here is that the tap does not *say so*.

**8.2 The mutation grid cannot reach the priority or latency model at all.**
Every row of §4.2 and §4.3 is classified by reading `stagetap.c`'s emitters and
finding no channel — an argument from the instrument's source, not a
measurement. If a tap site is added, those rows must be re-derived.

**8.3 Single-clause mutants miss interaction.** Two clauses could be jointly
pinned while neither is separately pinned, or jointly free while each looks
pinned. The grid is 15 single mutations, not the 2^13 lattice.

**8.4 The corpus is 60 fixtures.** `--limit 60` reproduces R7's population
exactly, which is the point; it is also the same 60, so nothing here is
independent evidence about *other* code. §3.3's 6.2× cap slack and §6's 8
positions are properties of these fixtures.

**8.5 `crates/` bytes 0 means the byte judge never saw any of this.** The sole
judge is `port(IL) == c2(IL)` byte-exact. Nothing here is evidence the port
would emit anything correctly, and no constant was adopted.

**8.6 The `after0` site's reading is inherited, not re-verified.** §6 depends on
`stagetap.c`'s claim that `0x10b7e701` sits after run 4 returns with the
function record in `ecx`. This lane re-read the surrounding bytes and agrees,
but the site was another lane's read and its `[R]` is inherited.

**8.7 Code date.** Data claims come from the pinned image, digest-checked in
this lane's `--verify` run. Code claims inherit the flat export's 2026-08-04
date.

---

## 9. Everything this lane corrected

| corrected | was | now |
|---|---|---|
| `WB_SCHEDCONF` §3.4 · `P_DAG` §6 box · **#3434** | the head case *"fires on 1,121 of the 1,461 graded pairs — the most common path"* | **0 of 1,461.** The 1,121 is the `excl-0x17/0x30f` **exit**. The head case fires 1,428 of 2,889 walks and its firing set is disjoint from the graded set (1,428 − 60 = 1,368 = UNGRADED, exactly) |
| `WB_SCHEDCONF` §3.3's qualification | *"1,461/1,461 grades three clauses, not the rule"*; four exits never fired | **four clauses are pinned and five are not** — and the never-fired list is the wrong list: it omits the head case (not an exit, so it has no row in the histogram) and includes clauses that a mutation *does* kill |
| `WB_SCHEDCONF` §3.2 · `P_DAG` §6 | the `0x50` cap is *measured-and-slack*, 0 of 1,461 | the tap pins a **ray `cap ≥ 13`**; `0x50` = 80 is **6.2×** past anything the corpus exercises |
| `WB_SCHEDCONF` §0 | *"the structural model survived **the tap**"*, covering §5's 10/10 | §5's 10/10 is **image-vs-image**; `dump_sched_tables.py --verify` takes no tap input. The latency model is `[R]` |
| `WB_SCHEDCONF` §8.2 | *"≈0.5 day. Three fields in `tap_walk_tuples`"* | **wrong record and wrong time**; no tuple→node back-pointer exists; the mechanism that does work is the DAG object `DAT_10c435e0` + `node+0x1c`, and it is a new walker |
| `WB_SCHEDCONF` §4 | the order confrontation is UNGRADED, input-returner scores 98.9 % | sized: **8 discriminating tuple positions of 3,015** at the final schedule, in 3 named functions |
| `READ_PLAN` §3 row **R7** | an open read, re-dispatched by decision 21 | **spent 2026-08-23**; the re-dispatch is instance N of *"check the board before dispatching"* |
