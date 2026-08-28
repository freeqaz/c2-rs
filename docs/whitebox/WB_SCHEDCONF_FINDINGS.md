# WB_SCHEDCONF — FINDINGS for read R7 (the scheduler, `[R]` → `[O]`)

> **PROVENANCE — DISASSEMBLY-DERIVED.** Every address is an absolute VA in
> `compilers/X360/16.00.11886.00/c2.dll`, sha256
> `c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`, verified
> by this lane before its prereg was written. See [`DISCLOSURE.md`](DISCLOSURE.md).
> Whitebox analysis is authorized and encouraged (`CLAUDE.md`, owner, 2026-08-17).

**Lane** `w-read-r7` · **kind** characterization · **Fixtures** none ·
**Census** +0 · **reach** 0 · **`crates/` bytes 0** ·
**Board** #3433–#3436 · prereg [`WB_SCHEDCONF_PREREG.md`](WB_SCHEDCONF_PREREG.md),
frozen as the branch's first commit `af966da13` before any measurement.

---

## 0. THE VERDICT, IN THE WORDS THE BRIEF ASKED FOR

R7 was to promote the scheduler model from `[R]` to `[O]` by **confronting it
with the live tap**, and to say whether the model **survived** or was
**refuted**. The answer is not one word, because the model has two halves and
they came out differently. Both are stated plainly rather than averaged.

> ### THE STRUCTURAL MODEL **SURVIVED THE TAP**, on 1,461 graded cells.
>
> The region-finder rule reproduces the observed region partition on
> **1,461 of 1,461** graded pairs across 60 fixtures, 100.00 %, at every region
> length from 1 to 14. The edge-latency mechanism reproduces **10 of 10** of
> `P_DAG.md` §5's published latencies from the raw image bytes. The `/Od`
> negative control and the `stage counts` positive control are both green.

> ### THE ORDER MODEL WAS **NOT CONFRONTED AT ALL**, and that is the finding.
>
> Not because the model failed, and not because the tap failed. Because
> **c2's scheduler does not reorder anything on this corpus.** The final
> schedule — run 4, the one that fixes emitted instruction order — changes the
> order on **3 of 357 functions (0.84 %)**. The prereg registered, in advance
> (§6.3 and decline criterion 4), that if the reordered subset were trivially
> small, P4 would be reported **UNGRADED with the fraction published** and no
> agreement rate quoted. It is trivially small, and that is what is reported.

> ### AND THE 13,104-CONFIGURATION SEARCH **COULD NOT HAVE SUCCEEDED**.
>
> Four independently sufficient structural reasons (§5), and **one of them
> binds on the residual population exactly**: every producer→store edge in a
> store run addresses matrix cell `(1,8)`, **the only cell of the 121 whose
> value is not a latency but a runtime branch** — 5 to the address operand,
> 2 to the data operand. The search ranged over *one scalar latency in 1..6*.
> On the residual it needed two, simultaneously, on edges out of the same
> producer. This is the R4 shape: the null was **structurally guaranteed**.

---

## 1. Scorecard

| # | prediction | result |
|---|---|---|
| P1.1 | weight table accounted as a total function, 7/7 | **HIT**, and better — the table is **8** entries, not 7 (§2.1) |
| P1.2 | ≥1 live non-negative-weight term absent from `P_DAG` §3 | **HIT** — `w[6]`, and `w[7]` besides |
| P2.1 | fanout unmasked before its shift ⇒ aliases symdest | **HIT** (§2.2) |
| P2.2 | fanout ≥ 4 reachable on the graded population | **UNGRADED** — the tap does not expose `node+0x26` (§7.3) |
| P3.1 | region rule ≥ 95 % over ≥ 100 regions, ≥ 2 populations | **PARTIAL** — 100.00 % on 1,461, but **one** population (§3.3) |
| P3.2 | the `0x50` cap never binds | **HIT** — 0 of 1,461; longest region 14 (§3.2) |
| P4.1 | ≥ 100 machine-extracted regions graded for order | **UNGRADED** (§4) |
| P4.2 | order-agreement rate falls below 87.5 % | **UNGRADED** (§4) |
| P4.3 | disagreements form ≤ 4 named families | **UNGRADED** (§4) |
| P4.4 | one verdict word, survived or refuted | **HIT** — §0, split by half and named |
| P5.1 | the 13,104 search could not have succeeded; ≥ 3 reasons | **HIT** — 4 reasons (§5.1) |
| P5.2 | ≥ 1 reason binds on the two-producer residual | **HIT** — reason 2, verified (§5.2) |
| P5.3 | the 1,048,576 search judged separately | **HIT** (§5.3) |
| P6.1 | four tables extracted, sha256-fenced, re-runnable | **HIT** |
| P6.2 | matrix cell width derived, not assumed | **HIT, by going RED** (§2.3) |
| P6.3 | all 9 `P_DAG` §5 latencies reproduce | **HIT** — 10/10 (§2.4) |
| P6.4 | ≥1 apparent discrimination collapses under stratification | **HIT** (§4.2) |
| P7.1 | reach 0, census +0, `crates/` 0 bytes | **HIT** |
| P7.2 | `DISCLOSURE.md` gains 0 rows | **HIT** |

**11 HIT · 1 PARTIAL · 5 UNGRADED · 0 MISS.** The five UNGRADED are four
clauses of P4 plus P2.2, all for the same reason — the population has no
variance to grade against — and they are reported as UNGRADED rather than
smoothed into partials.

**Calibration.** The prereg's confidences scored: the two rows registered as
*most likely to fail* (P1.2 at 0.55, P2.1 at 0.5) both **hit**; the row
registered at 0.6 for a specific cell width (P6.2) was **wrong about the
mechanism in the informative direction**; and the row registered at 0.6 for
P4.1 was the one that could not be graded at all. The lane's confidence was
**too low on the reads and far too high on the measurements** — a bias worth
carrying forward, since it is the same direction R4 reported.

---

## 2. The machine model, read (P1, P2, P6)

Instrument: [`scripts/dump_sched_tables.py`](scripts/dump_sched_tables.py) —
reads the pinned image directly, verifies the sha256 and refuses otherwise, and
transcribes `FUN_10c1c1d4` guard by guard with each VA in a comment.

### 2.1 The priority function is SIX terms and the weight table has EIGHT entries

`P_DAG.md` §2.1 records seven shorts at `0x10c3bf9c` and §3 publishes a
three-term formula. Both are short. The pass `FUN_10be5df6` reads the table
**through a pointer** at `0x10c6fe14` and uses six entries; a seventh is read by
the dynamic-bonus function; one is never read.

| entry | value | term | read at | status |
|---|---:|---|---|---|
| `w[0]` | `-1` | `node+0x4e` bit 1 | `0x10be5ed6` | **DEAD** — a 0/1 term `>> 1` is 0 |
| `w[1]` | `13` | height `node+0x48` | `0x10be5eec` | live — `P_DAG` §3's height |
| `w[2]` | `8` | fanout `node+0x26` | `0x10be5f35` | live — `P_DAG` §3's fanout |
| `w[3]` | `-1` | `node+0x4e` bit 0 | `0x10be5f03` | **DEAD** |
| `w[4]` | `-2` | — | — | **never read by the priority pass** |
| `w[5]` | `10` | `node+0x4e` bit 2 | `0x10be5f1b` | live — `P_DAG` §3's "symdest" |
| `w[6]` | `0` | `node+0x4e` bit 3 | `0x10be5f88` | **live, worth 1, gated** — absent from `P_DAG` §3 |
| `w[7]` | `0` | dynamic unit bonus 0..7 | `0x10c1bbf6` | **live** — past `P_DAG`'s "7 shorts" |

`w[6]`'s term is added only when the node's tuple type nibble is `0x5000`
(`0x10be5f6c`). `w[7]`'s bonus is `7` when the unit is free, `0` when its
reservation exceeds 12, else `(6 − reservation) >> 1` (`0x10c1bbd8`).

**Two terms are computed and then multiplied by a disabled weight**, which is
the more interesting half:

* **bit 0 is CRITICAL-PATH MEMBERSHIP.** Seeded on the region head at
  `0x10be5e5b` and propagated at `0x10be5ec3` to every successor achieving the
  node's height exactly (`height == succ.height + latency + 1`). c2 computes
  the critical path and weights it `-1`.
* **bit 1** is set by `FUN_10c1bd6f` (`0x10c1bdad`) over the pred edges of the
  region's last node. Weight `-1`. Also discarded.

**Bits 2 and 3 are assigned once, at node creation** (`FUN_10b327cd`,
`0x10b3280a` and `0x10b32820`, the MSVC bitfield-assign idiom), by
`FUN_10b32516` / `FUN_10b324f9`: bit 2 iff the operand chain at `tuple+0x28`
holds a record whose kind byte is `2` or `6`; bit 3 iff the chain at
`tuple+0x2c` does. Both gated on `tuple+0x9` bit 0.

> **A naming caveat, reported rather than resolved.** The repo's own record
> (`WB_MIDDLE_INTERFACES.md:182-183`) calls `tuple+0x28` the **source** side and
> `+0x2c` the **destination** side. The `<<10` term — the one `P_DAG` §3 names
> **"has-symbol-dest"** — reads `+0x28`, the *source* chain. The **mechanism is
> exact**; the **name looks backwards**. It is not corrected here because the
> operand kind values `2` and `6` have no naming table anywhere in this repo,
> and a lane that renamed the term on that basis would be guessing.

### 2.2 The three "keys" are summed into one word and are not separable (P2.1)

```
10be5f39:  0f b7 46 26     movzx eax, WORD PTR [esi+0x26]   ; fanout, 16-bit
10be5f41:  d3 e0           shl   eax, cl                     ; cl = w[2] = 8
```

No mask, no clamp, no saturate. The terms are then **added**
(`0x10be5f4c`–`0x10be5f53`), not concatenated. Consequences, in the arithmetic:

* `fanout = 4` contributes `1<<10` — **exactly what the bit-2 flag contributes**;
* `fanout ≥ 32` carries into the height field at `<<13`.

So `P_DAG` §3's presentation of *(height, fanout, symdest)* as three separable
keys is true only while `fanout ≤ 3`. **This matters directly to `order.rs`**
(§5.3).

### 2.3 P6.2 — the instrument-lies check went RED, and was right to

The prereg registered that `P_DAG.md` never states the matrix cell width and
that a wrong guess yields a plausible matrix, so the width would be *derived*
by requiring `P_DAG` §5's nine prose latencies to reproduce. Run as designed:

| width | reproduces |
|---|---|
| 1 | 0/9 |
| 2 | 0/9 |
| 4 | **1/9** |

**The check rejected the correct width.** Width 4 is right — `imul edx,edx,0xb`
at `0x10c1c255` and `mov eax,[edx*4+0x10c3c1a8]` at `0x10c1c25a` settle it — and
the check failed because its premise was wrong in a way no probe could expose:
**the cells are not latencies, they are tags.** `0x10c1c261` returns a cell
verbatim only when it is `> -2`; six negative tags dispatch to opcode-,
category- and edge-flag-dependent rules. The fix was not a better probe. It was
reading `FUN_10c1c1d4`. This is the clearest instance this lane produced of the
standing doctrine: **price the read that answers the question, and prefer it.**

### 2.4 The mechanism, and P6.3 = 10/10

The matrix is indexed by `CLASSTAB[opcode]` read at **stride 12** from
`0x10b221d0` (`0x10c1c234`, `0x10c1c23f`) — a per-opcode table distinct from the
machine table at `0x10b202b0`. They agree on **660 of 661** opcodes and differ
at opcode `0x292` (latency class 1, machine unit 0), so `P_DAG` §2.1's
*"`+8` class IS the unit"* is a near-identity, not an identity. Class 0
short-circuits, so **100 of the 121 cells are reachable and 21 can never be
addressed**.

Anti-deps are `0` **structurally**, by the `test BYTE PTR [ecx+0x10],0x21` gate
at `0x10c1c1e4` — not by a matrix cell.

The tag decode:

| tag | rule | at |
|---:|---|---|
| `-8` | producer opcode ∈ `{0x2d..0x30}` (cmp family) → **0**, else **2** | `0x10c1c315` |
| `-7`, `-5` | → **2** | `0x10c1c32c` |
| `-6` | consumer category `0x12` + a guard chain → **23**, else **0** | `0x10c1c2c0` |
| `-4` | → **17** | `0x10c1c2bc` |
| `-3` | → **5** | `0x10c1c2b8` |
| `-2` / `≤ -9` | **5**, or **2** when the consumer opcode ∈ `[0x14d,0x180]` **and** `edge+0x19` bit 1 is set | `0x10c1c294`–`0x10c1c2b3` |

> **The finding `P_DAG` §5 hides by flattening.** *"ALU → memory ADDRESS = 5"*
> and *"ALU → store DATA = 2"* are **not two matrix cells**. They are **one
> cell**, `(1,8) = -2` — verified to be **the only cell of the 121 holding that
> tag** — resolved at runtime by an edge flag bit. Any model that reads the
> matrix as a static latency grid gets one of the two wrong, whichever it picks.

**P6.3: all ten reproduce** from the transcription
(`dump_sched_tables.py --verify`), including both halves of the cmp/non-cmp
branch split. **`P_DAG` §5 is vindicated as a set of facts while being shown to
be lossy as a mechanism.**

Data deliverable: [`ref/SCHED_LATENCY.tsv`](ref/SCHED_LATENCY.tsv), 121 cells.

### 2.5 Address corrections to `P_DAG` §2

* **`0x10c1c25e` is not an instruction.** `P_DAG` §2 lists it as *"the ALU→branch
  cell"*. Linear disassembly from the verified entry `0x10c1c1d4` puts
  `0x10c1c25a` (`8b 04 95 a8 c1 c3 10`, 7 bytes) spanning `0x10c1c25a..0x10c1c260`,
  so `0x10c1c25e` is **mid-instruction**. The ALU→branch decision is at
  **`0x10c1c315`**.
* **Several scheduler hooks are inert in this build**, which matters to a port
  that would otherwise implement dead paths: `0x10b32533` returns 0 (25 callers,
  ICF-folded) so the veto at `0x10be6134` never rejects; `0x10c1bdc1` returns 0
  so the veto at `0x10be6155` never fires; `0x10c1bdb9` returns 1 so the pred-edge
  loop `0x10be61a9..0x10be61cd` does nothing and the scheduler is never skipped at
  `0x10be6394`; **`0x10c1bdc0` is a bare `ret`**, so the second setup call at
  `0x10be5e17` is a no-op; and the region finder's tail jump `0x10be5dab` lands on
  an identity stub.

---

> ### ⛔ THIS SECTION IS AMENDED, 2026-08-28, BY LANE `w-sched` (board **#3725**–**#3727**)
>
> Amended beside, never rewritten. **§3.1's 1,461/1,461 stands, reproduced
> figure-for-figure on another tree**, as do all three of §3.4's corrections to
> `P_DAG` §2. What is corrected is what the 100.00 % is evidence *for*, and one
> number is attributed to the wrong clause:
>
> * **§3.4 bullet 2's *"This clause fires on 1,121 of the 1,461 graded pairs —
>   the single most common path"* is WRONG.** The 1,121 is §3.3's
>   `excl-0x17/0x30f` **exit** count — the terminator at `0x10be5d8b`, a
>   different clause. The head case at `0x10be5d55` is not an exit, has no row
>   in that histogram, and fires on **1,428 of 2,889 walks and 0 of the 1,461
>   graded pairs**. `1,428 − 60 last-of-fixture = 1,368 = the UNGRADED count`,
>   exactly: the clause's firing set and the graded set are disjoint, because
>   the instrument check that makes the rest of §3 trustworthy discards every
>   walk it fires on.
> * **§3.3's qualification is right in spirit and wrong in its list.** By
>   single-clause mutation, **four** clauses are pinned and **five** are not,
>   and the unconfirmable set is *not* the never-fired set — it contains the
>   most-fired clause in the rule and omits nothing that a mutation kills.
> * **§3.2's cap result is a RAY.** A cap of 13 scores 1,461/1,461; 12 goes
>   red. The tap pins `cap ≥ 13` against a read `0x50` = 80 — **6.2× slack**.
>
> Instrument: [`scripts/mutate_regions.py`](scripts/mutate_regions.py). Full
> grade: [`WB_SCHEDCHK_FINDINGS.md`](WB_SCHEDCHK_FINDINGS.md) §3.

## 3. The region rule (P3)

Instrument: [`scripts/grade_regions.py`](scripts/grade_regions.py), over
`c2rs stage snap` on 60 fixtures. Each entry to the region finder walks the
tuple list from that region's first tuple **to the end of the list**, so region
*k*'s walk is a strict suffix of region *k−1*'s and the difference of their
lengths is what region *k−1* consumed. No new tap code was needed.

**Instrument check first**: a pair is graded only when walk B is
**byte-identical to the tail of walk A**, which proves the two walks are
consecutive entries inside one run rather than two unrelated runs that happen
to shrink. **1,368 pairs fail it and are reported UNGRADED, not dropped.**

### 3.1 Result

```
60 fixtures, 2,889 region walks
GRADED 1,461   HIT 1,461   MISS 0   =  100.00%
```

Stratified by region length: **100 % at every length 1…14**, so no single
stratum carries it.

**The incumbent was `[O]` 15/15 for the *call* clause alone, with the cap and
the four boundary categories graded on zero cells.** This beats it on
denominator (1,461 vs 15) and on scope (the whole predicate).

### 3.2 The `0x50` cap does not bind (P3.2)

`P_DAG` §6 lists *"whether the region-finder's `0x50`-tuple cap ever binds in
practice"* as **unmeasured**. It is now measured: **0 of 1,461** regions reach
it and the longest observed region is **14** tuples against a cap of 80. The
row moves from *unmeasured* to **measured-and-slack** — on this population, and
§7.7 bounds that.

### 3.3 The qualification, which is the part worth reading

Only **three of the rule's seven exits ever fired**:

```
excl-0x17/0x30f  1121      incl-cat-12  204      incl-cat-1b  136
NEVER EXERCISED: cap>0x50, incl-cat-14, excl-cat-19, end-of-list
```

So 1,461/1,461 grades **three clauses, not the rule**. The other four are read
and ungraded. P3.1 is scored **PARTIAL**, not HIT, because the prereg demanded
**two populations** and this lane landed one — the grid population (§5.4 of the
prereg) was not built, and the honest consequence is a PARTIAL.

### 3.4 Three corrections to `P_DAG` §2's region row

`P_DAG` says the region *"ends at tuple category ∈ {`0x12`,`0x14`,`0x19`,`0x1b`}
or `0x17`-with-opcode-`0x30f`"*.

1. **The four categories are not alike.** `0x12`, `0x14`, `0x1b` stop
   **inclusive** (the terminator joins the region, `0x10be5d85`); `0x19` stops
   **exclusive** (`0x10be5d7f`), as does `0x17`-with-`0x30f` (`0x10be5d8b`).
   Getting this wrong is an off-by-one on **every** region boundary.
2. **There is a head special case `P_DAG` does not mention.** If the region's
   first tuple has opcode `0x30f` it is taken into the region and the scan
   starts at the second (`0x10be5d55`). **This clause fires on 1,121 of the
   1,461 graded pairs — the single most common path, and it was undocumented.**
3. **The cap is `cmp edx,0x50 / jg`** at `0x10be5d66`, a **signed** compare on a
   count starting at 0, so the body runs for `count = 0..0x50` and **up to 81**
   tuples are visited past the head. `0x50` is the constant, not the count.

---

## 4. The order confrontation (P4) — UNGRADED, and why that is the result

### 4.1 The measurement

Instrument: [`scripts/grade_reorder.py`](scripts/grade_reorder.py), plus an
independent second instrument (the tap's own function-walk verdicts under
`C2RS_STAGE_FUNCWALK=1`). **They agree.**

| instrument | population | reordered |
|---|---|---|
| region method | 456 run-to-run pairs | **5 — 1.10 %** |
| function walk, run 1 (`sched1→globregs`) | 357 functions | 6 — 1.68 % |
| function walk, run 2 (`sched2→color`) | 357 functions | 9 — 2.52 % |
| **function walk, run 4 (`sched0→after0`) — THE FINAL SCHEDULE** | 357 functions | **3 — 0.84 %** |
| *for contrast:* `globregs→sched2` (globregs' own effect) | 357 functions | 334 — 93.6 % |

Run 3 is confounded: `sched3→sched0` spans the lowering band and differs
357/357.

**The final schedule changes the order on three functions**, all among the
longest in the corpus: `il_intrinsic_bits.cpp` fn12 (19 tuples),
`il_sy_locals.cpp` fn8 (16), `il_sy_locals.cpp` fn9 (21).

### 4.2 Why this is UNGRADED and not a 99 %

The prereg registered, before any measurement (§6.3, decline criterion 4), that
a region the scheduler leaves alone is a **free hit for any order-preserving
model, including one that returns its input**, and that if the reordered subset
were trivially small P4 would be reported **UNGRADED with the fraction
published**. A simulator that returns its input scores **98.9 %** on the region
population and **99.2 %** on the final run. That number measures the corpus.

**This is `/QXSTALLS` in exactly the predicted shape** (P6.4 = HIT): reordering
is nearly a function of body length — 0.00 % at every function length ≤ 7,
which is 355 of the 456 pairs, rising to 28.6 % at 10 — and the corpus is
dominated by short bodies. The pooled 1.10 % and any pooled agreement rate
would both be artefacts of the length distribution. **Mechanism**: the median
region is 2 tuples and the longest is 14, so most regions are too short to
admit a permutation at all.

### 4.3 The larger result

**This repo's fixture corpus cannot validate a scheduler model in either
direction.** It contains three positive cells for the final run. A lane that
graded a scheduler model here and reported 99 % would be reporting the corpus,
and that is a live hazard, because §5's fitted searches were graded on
store-run grids that are exactly this shape.

**It also re-prices R7's own premise.** The read plan justified R7 as *"F0
re-priced 8 → 4 raw"* by confronting the read model against the tap. The
confrontation the row imagined **is not available on this corpus at any price**;
it needs a population built to reorder. That is a follow-up lane's deliverable,
named in §8.

---

## 5. Could the 13,104-configuration search have succeeded? (P5)

**No.** Four independently sufficient reasons, and the incumbent to beat is
`schedule.rs:56-61`'s own sentence — *"Rule 2 is not a priority function, so no
member of that family can express it"* — which asserts the conclusion, so
restating it earns nothing. None of the four below is in that file.

### 5.1 The four reasons

The family (`schedule.rs:56-61`): **forward/backward × latency 1..6 × a
lexicographic priority over six DAG features**.

**Reason 1 — the priority is a weighted SUM into one word, not a lexicographic
order.** §2.2: the terms are summed at fixed shifts with no mask. Lexicographic
means key *k* is consulted only when keys `1..k-1` tie; here a large enough
lower-order key **overrides** a higher-order one (`fanout ≥ 4` reaches the
bit-2 term's weight, `fanout ≥ 32` reaches height). No lexicographic order over
the same features can express that. *Sufficient.*

**Reason 2 — latency is not a scalar in `1..6`.** §2.4: it takes values
`{0,1,2,4,5,7,10,12,14,16,17,23}` and six reachable cells are **tags** resolved
at runtime from the producer opcode, the consumer opcode, the consumer category
and an edge flag bit. *Sufficient*, and it is the one that binds — §5.2.

**Reason 3 — the scheduler is cycle-driven under a resource model.** The issue
predicate `0x10c1bfe2` admits **at most two nonzero-unit instructions per
cycle** (`cmp edi,0x2` at `0x10c1c011`, independent of the width
`DAT_10c3cf98 ∈ {2,4}`) and never two on the same unit; per-unit reservation
counters at `0x10c3cfa8` tick down each cycle; selection tests
`node+0x40 <= cycle + slack`, **not** `<= cycle` (`0x10be6174`). Two nodes with
identical priorities issue in either order depending on resource state.
*Sufficient.*

**Reason 4 — the priority is not static.** `FUN_10be60c0` re-prices every node
(`0x10c1bbaf`) and **fully re-sorts the ready list** (`0x10be6046`) at the top of
**every cycle**, adding a unit-availability bonus of `0..7` that depends on
machine state at that cycle. The priority is a function of (DAG, cycle,
resource state), not of the DAG. A search over static priority functions cannot
express a time-varying one. *Sufficient.*

(A fifth, not counted because it may not bind on a short store run: the
schedule is **iterated** — `0x10c1bdff` can rewrite edge-latency slots after a
pass and force a re-schedule of the same region.)

### 5.2 The reason that BINDS on the residual (P5.2)

The residual is *"exactly the two-producer tier, 0 of 48"* — two-producer store
runs. Verified from the tables, not asserted:

| | opcode | latency class |
|---|---:|---:|
| `li` (the producer) | `0x270` | **1** (integer ALU) |
| `stw` (the store) | `0x17a` | **8** (integer load/store) |

so every producer→store edge addresses cell **`(1,8)`**, whose value is `-2`,
and **`(1,8)` is the only cell of all 121 holding that tag.** `0x17a` lies
inside the tag's `[0x14d,0x180]` consumer range (52 opcodes qualify). Therefore:

```
li -> stw, edge+0x19 bit 1 CLEAR (address operand)  ->  latency 5
li -> stw, edge+0x19 bit 1 SET   (data operand)     ->  latency 2
```

**Every cell of the residual population is built entirely out of the one edge
type in c2's machine model whose latency is not a number.** The search ranged
over a single scalar latency in `1..6`; on the residual it needed **two,
simultaneously, on edges out of the same producer**. The null was
**structurally guaranteed** — the R4 shape, reached by a different mechanism.

**And `schedule.rs`'s fitted constant is one half of that cell.** `P_DAG` §4.4
already suspected this (*"its store-floor constant 2 is the ALU→store-data
latency"*); it is now read. `BLOCKED_STORE_POSITIONS = 2` and `order.rs`'s
`HEAD_SLOTS_MAX = 2` are the **data** branch of cell `(1,8)`. The **address**
branch, 5, is nowhere in `crates/`.

### 5.3 The 1,048,576-configuration search, judged separately (P5.3)

**Also no, and for a partly different reason** — no silent generalisation.

`order.rs:139-146`'s family is **per-store release times**: 2 counters × `4^9`
thresholds × 2 tiebreaks. c2 **has no per-store release time**. It has
`node+0x40`, an earliest-start relaxed from the DAG, and the *order* comes from
the priority sort, not from release times. Reasons 1, 2 and 4 apply unchanged.

**But the more useful finding is positive.** `order.rs`'s fitted rank —
*(use count descending, first-use source index ascending)* — **is** c2's
priority restricted to one regime. `fanout` (`node+0x26`, bumped once per
successor edge at `0x10b32113`) **is** the use count; `node+0x44` **is** the
first-use/original index, and the ready-list comparator is fanout DESC then
`node+0x44` ASC once height ties. So `P_DAG` §4.4's identification is confirmed
by the read.

> **With one boundary `order.rs` does not know about.** The mapping is exact
> **only while `fanout ≤ 3`**. At `fanout ≥ 4` the term reaches the bit-2 flag's
> weight (§2.2) and the rank stops being a rank. `order.rs`'s own
> `MAX_MODELLED_PRODUCERS = 3` (board #541, *"recorded, not explained"*) **sits
> exactly at that boundary.** This lane does not claim the constant was derived
> from it — it was fitted — but the fitted value and the read boundary coincide,
> and #541 now has a candidate explanation for the first time. **Reported as a
> finding for a follow-up lane; nothing in `crates/` is edited** (§7.6, P7.1).

---

## 6. The list scheduler as pseudocode

The read plan's named deliverable. Every constant carries its address.

```
schedule_function(fn):                          # 0x10b7dc51, 0x10b7df57
  # runs 1..3 mode 1, run 4 mode 0 (post-lowering); each gated on
  # DAT_10c2e2fc (the /Og bit) AND fn[+0x1c] bit 0 for runs 1..3
  for each run: schedule_run(fn, mode)          # 0x10be6382

schedule_run(fn, mode):
  DAT_10c3d144 = mode                           # 0x10be63a5
  DAT_10c3cf98 = 4 if (mode != 0 and DAT_10c2e2d2 != 0) else 2   # 0x10c1c101
  t = first tuple
  while t:
    last = find_region(t)                       # 0x10be5d4b
    build_dag(t, last)                          # 0x10b328da
    prioritise(t, last)                         # 0x10be5df6
    emit_cycles(t, last)                        # 0x10be626c -> 0x10be60c0
    t = last->next

find_region(t):                                 # 0x10be5d4b  -- graded 1461/1461
  result = None; cur = t; count = 0
  if cur->opcode == 0x30f: result = cur; cur = cur->next      # 0x10be5d55
  while cur:
    if count > 0x50: break                                    # 0x10be5d66, SIGNED
    c = cur->cat
    if c in {0x12, 0x14, 0x1b}:      return cur               # INCLUSIVE
    if c == 0x19:                    return result            # EXCLUSIVE
    if c == 0x17 and cur->opcode == 0x30f: return result      # EXCLUSIVE
    result = cur; cur = cur->next; count += 1
  return result

edge_latency(e):                                # 0x10c1c1d4  -- 10/10 vs P_DAG §5
  if (e->kind & 0x21) == 0:            return 0              # anti-deps
  p, c = e->src->tuple->opcode, e->dst->tuple->opcode
  if not (0 < p < 0x295) or not (0 < c < 0x295): return 0
  if e->src->tuple->cat == 0x15:       return 0
  pc, cc = CLASSTAB[p], CLASSTAB[c]                          # 0x10b221d0, STRIDE 12
  if pc == 0 or cc == 0:               return 0
  v = MATRIX[pc*11 + cc]                                     # 0x10c3c1a8, i32
  if v > -2:                           return v              # a literal latency
  ... tag dispatch, §2.4 ...
  e->latency = result                                        # WORD at e+0x14
  src->max_out = max(src->max_out, result)                   # BYTE at src+0x4a

prioritise(head, tail):                         # 0x10be5df6
  for n from tail back to head:                              # reverse list order
    n->height = 1 + max over succ e of (e->dst->height + e->latency)   # u16
  mark_critical_path(head)                                   # bit 0, then discarded
  for n in region:
    W = *(short**)0x10c6fe14                                 # -> 0x10c3bf9c, set once
    n->priority = (bit1(n) >>1)            # w[0] = -1  DEAD
                + (n->height   << 13)      # w[1]
                + (n->fanout   <<  8)      # w[2]  -- UNMASKED, see §2.2
                + (bit0(n)     >>1)        # w[3] = -1  DEAD (critical path!)
                + (bit2(n)     << 10)      # w[5]
    if n->tuple->opcode == 0x2b8: n->priority = 0xffffffff    # MAXIMAL (unsigned cmp)
    elif (n->tuple->typeword & 0xf000) == 0x5000:
        n->priority += (bit3(n) << 0)      # w[6] -- ABSENT FROM P_DAG §3
    n->prio_work = n->priority                                # +0x3c

emit_cycles(head, tail):                        # 0x10be626c
  cycle = 0
  while ready list non-empty:
    for n in ready: n->prio_work = n->priority + (bonus(n, cycle) << w[7])  # 0x10c1bbaf
    resort ready by (prio_work DESC unsigned, (u16)(n+0x44) ASC)  # 0x10be6046 / 0x10be5cea
    for slot in 0 .. DAT_10c3cf98-1:
      pick the FIRST n in ready order with
          issuable(n, issued_this_cycle)                      # 0x10c1bfe2
          and n->earliest <= cycle + slack(n->unit)           # 0x10be6174, NOT <= cycle
      if none: break
      issue n; relax successors' earliest and pred counts     # 0x10be6210
    cycle += 1                                                # UNCONDITIONAL, 0x10be6365
    tick unit reservations down                               # 0x10c1b9da

issuable(n, issued):                            # 0x10c1bfe2
  if n->unit == 0:                        return 0            # free, uncapped
  if any m in issued with m->unit == n->unit:      return -1
  if count(m in issued: m->unit != 0) >= 2:        return -1  # HARD 2, not the width
  if reservation[n->unit] > 0 and n->unit <= DAT_10c3cf90: return -1
  return n->unit
```

Two comparator details `P_DAG` §3 does not carry: the priority compared is
`node+0x3c` (**the per-cycle working copy**, not `+0x38`), **unsigned**; and
`node+0x44` is **truncated to 16 bits by the caller** (`movzx …,WORD PTR [x+0x44]`
at `0x10be5fe8`/`0x10be5fef`) before an unsigned 16-bit compare.

---

## 7. WHAT THIS LANE'S EVIDENCE IS STRUCTURALLY INCAPABLE OF SHOWING

Registered in the prereg before the work; re-stated here with what actually
happened, per the brief.

**7.1 Underdetermination — unchanged, and now moot.** Order agreement could
never have distinguished c2's priority function from any order-equivalent one.
As it happens the question did not arise: §4 shows there was no order variance
to agree about. The ceiling stands for any future lane.

**7.2 The cycle model is graded through a lossy quotient — and was not graded
at all.** Issue width, the ≤2-nonzero-unit cap, the ≤7 bonus, the +15/+40 stall
penalties and the **iteration** are observable only through their effect on a
final order. **This lane produced no evidence for or against any of them.**
They are `[R]` on the strength of the disassembly and nothing more. In
particular **this lane cannot detect that a schedule was iterated** — it would
see the fixed point, never the passes.

**7.3 `node+0x26` is not observable.** The tap's `TU`/`FT` rows carry opcode,
category, flags and cc — **not fanout**. P2.2 (is `fanout ≥ 4` reachable?) is
therefore **UNGRADED**, and the aliasing of §2.2 is proven in the *arithmetic*
and unproven in *practice*. This is the single cheapest follow-up (§8).

**7.4 Run 4 only, and even that barely.** The `sched0→after0` pair grades the
final run; runs 1 and 2 are graded separately; **run 3 is confounded with
lowering and is not graded at all**. Anything true of the mid-level passes and
false of run 4 is invisible here.

**7.5 The second author was never separated out.** `P_DAG` §6's `factor.c` block
merger (`0x10b3baa8` → `0x10b3a790`) also moves tuples and is not a DAG client.
Because §4 found essentially no motion to attribute, the question of *whose*
motion it was never became live — **so this lane's green results are not
evidence that the merger is quiet**, only that the phase boundaries it measured
were.

**7.6 Zero `crates/` bytes means the byte judge never saw any of this.** The
sole judge is `port(IL) == c2(IL)` byte-exact. **Nothing here is evidence the
port would emit anything correctly.** §5.3's `MAX_MODELLED_PRODUCERS` coincidence
is a finding for a follow-up lane and was deliberately not acted on.

**7.7 §3.2's cap null and §4's identity result are workload statements.** Both
are properties of 60 fixtures / 357 functions. Neither shows the cap is
unreachable, nor that c2's scheduler is quiet, **in general** — only on this
corpus, which §4.3 argues is the wrong corpus for the question.

**7.8 Code date.** Data claims (§2, §5.2) come from the pinned image and are
sha256-fenced in the script. Code claims inherit the flat export's 2026-08-04
date; the digest check makes it very likely the export was built from the
pinned image but does not prove it.

**7.9 The confirmation probe (prereg §9) is reported VACUOUS.** The prereg bound
it: *"if the holdout contains zero block-boundary-adjacent regions, the
confirmation probe is reported as VACUOUS, not as passed."* The grid population
was not built and the order probe had no positive cells, so **the probe named in
§9 did not run.** It is recorded as vacuous rather than quietly dropped. What
*did* end in a probe capable of failing is §2.3 — a width check that **went red
on the correct answer** and drove a correction — and §3's instrument check,
which rejected 1,368 of 2,829 candidate pairs.

---

> ### ⛔ ITEM 2's PRICE IS AMENDED, 2026-08-28, BY LANE `w-sched` (board **#3730**)
>
> ***"Expose `node+0x26` and `node+0x38` in the tap (≈0.5 day). Three fields in
> `tap_walk_tuples`"*** — **wrong about the record and wrong about the time**,
> and this matters beyond the estimate because `#3716` carries item **1** into
> F0 as an UNPRICED term.
>
> * `+0x26`/`+0x38` are **DAG node** fields; `tap_walk_tuples` walks **tuple**
>   records. The join is one-way, `node+0x1c` → tuple (written `0x10b327de`,
>   read `0x10c1c1ea`). `FUN_10b327cd` stores **nothing** into the tuple in its
>   158 bytes, so **no tuple→node back-pointer exists**.
> * `tap_walk_tuples` has **one** caller, the `region` site at region-finder
>   **entry** (`0x10be643e`), and `build_dag` calls the DAG reset `0x10b32008`
>   at `0x10b328e8`. At the hook, region *k*'s DAG **does not exist yet**.
> * **A cheaper mechanism than the one this item describes does exist**, and it
>   is not a back-pointer: the DAG object at `DAT_10c435e0` carries head/tail at
>   `[+0]`/`[+4]` and a node count at `[+0x24]`, nodes chain on `+0x4`, and each
>   node names its tuple at `+0x1c`. Because the reset runs *inside*
>   `build_dag`, the **existing** region hook can read region *k−1*'s finished
>   DAG with no new site. `[R]`, with two named caveats.
> * Corrected shape, published as a shape and not a wall clock
>   (`WHITEBOX_LEVERAGE` §3.1, and `#3716`'s own refusal): a new walker over a
>   second record type reached through a global, a join field, a per-node
>   plausibility fence and cap, a canonical-stream schema extension in
>   `crates/c2-reference/src/stage.rs`, a determinism/neutrality re-run, and a
>   confirmation probe on the join. **Strictly more than 0.5 d.**
>
> **F0 is not re-priced by that lane and is not re-priced here.** The figure
> stays `#3716`'s: *≥ 10 raw sub-lanes + 2 UNPRICED*.
> [`WB_SCHEDCHK_FINDINGS.md`](WB_SCHEDCHK_FINDINGS.md) §7.

## 8. What a follow-up lane should do, priced

1. **Build a population that reorders** (≈1 day). §4 is the blocker for every
   order claim. Long straight-line regions, ≥ 8 independent chains, mixed units
   — the shapes at which §4.1's stratification shows reordering starts. Without
   this, no scheduler model in this repo can be graded, including the port's.
2. **Expose `node+0x26` and `node+0x38` in the tap** (≈0.5 day). Three fields in
   `tap_walk_tuples`. Discharges P2.2 and makes the priority function directly
   observable rather than inferred from order — which removes §7.1's ceiling.
3. **Price `MAX_MODELLED_PRODUCERS = 3` against §5.3's `fanout ≤ 3` boundary**
   (≈0.5 day). Board #541 has had no explanation since it was opened; it now has
   a candidate. A `crates/` lane, not a characterization lane.
4. **The `-2` cell's address/data split** belongs in any future store model:
   `crates/` carries the `2` and not the `5`.

---

## 9. Everything this lane corrected

| corrected | was | now |
|---|---|---|
| `P_DAG` §2.1 weight table | "7 shorts" | **8 entries**; `w[7]` live in `0x10c1bbaf` |
| `P_DAG` §3 priority | 3 terms | **6 terms**, 2 of them dead-weighted; `w[6]` absent |
| `P_DAG` §3 keys | 3 separable keys | one **summed** word; unseparable at `fanout ≥ 4` |
| `P_DAG` §3 priority field | implied `+0x38` | compared field is **`+0x3c`**, unsigned; `+0x44` truncated to 16 bits |
| `P_DAG` §5 latencies | 9 static rows | **facts confirmed 10/10**, mechanism is tag dispatch; ADDRESS/DATA are **one cell** |
| `P_DAG` §2.1 class index | "+8 class IS the unit" | a **separate** stride-12 table; 660/661 agree, `0x292` differs |
| `P_DAG` §2 `0x10c1c25e` | "the ALU→branch cell" | **mid-instruction**; the cell is `0x10c1c315` |
| `P_DAG` §2 region rule | 4 alike categories | **3 inclusive + 1 exclusive**, plus an undocumented head case firing on 1,121/1,461 |
| `P_DAG` §2 cap | "≤ `0x50` tuples" | signed `>`; **up to 81** tuples past the head |
| `P_DAG` §2 issue selection | "earliest-start ≤ cycle" | **`≤ cycle + slack`** |
| `P_DAG` §6 cap question | unmeasured | **measured, 0/1,461** |
| read plan §3 row R7 | "F0 re-priced 8 → 4" | the confrontation it assumes **is not available on this corpus** (§4.3) |
| brief citation | `order.rs:89-95` | the 1,048,576 search is at **`order.rs:139-146`** |
