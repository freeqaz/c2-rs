# WB_SCHEDCONF — PREREG for read R7 (the scheduler, `[R]` → `[O]`)

> **PROVENANCE — DISASSEMBLY-DERIVED.** Every address below is an absolute VA
> in `compilers/X360/16.00.11886.00/c2.dll`. See
> [`DISCLOSURE.md`](DISCLOSURE.md); nothing here may enter `crates/` without a
> row there naming the address it came from. Whitebox analysis is authorized
> and encouraged (`CLAUDE.md`, project owner, 2026-08-17).

    Lane:      w-read-r7   (branch wt-w-read-r7, base master 7aa91ff3d)
    Kind:      characterization lane (docs/rungs/README.md § "Lane kinds" 3)
    Fixtures:  none
    Census:    +0
    Reach:     0, registered as a prediction
    crates/:   0 bytes, registered as a FENCE, not an expectation
    Board:     #3433–#3436 (coordinator-allocated; the next-free pointer was
               NOT read or advanced by this lane)
    Frozen:    2026-08-23, as the FIRST commit on wt-w-read-r7, before any
               instruction of the scheduler band was read by this lane and
               before any tap was run.

**Subject.** Read **R7** of the read plan
([`READ_PLAN_2026-08-21.md`](READ_PLAN_2026-08-21.md) §3 row R7 and §5; funded
by the owner 2026-08-23 — `docs/DECISIONS_2026-08-22.md` decision 7, board
**#3423**). The row's own words:

> **R7** | **Scheduler `[R]` → `[O]`** — no new reading; check the read
> priority/latency model against the live tap | F0 re-priced 8 → 4 raw;
> confronts the 13,104-config residual with c2's actual priority function |
> the list scheduler as pseudocode + the latency matrix as data | **3–5**

**This row is unlike every other row in the plan, and the difference is the
point.** R1–R6, R8 and R9 all *acquire* a fact. R7 acquires nothing: it takes
a model that is already written down and asks whether it is true. **The model
surviving and the model being refuted are both first-class results, and the
findings document will say which in those words.**

**Image.** sha256
`c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`, size
1 347 072 — **verified by this lane against the repo copy before this file was
written**, against the pin at [`ref/README.md`](ref/README.md). The flat export
at `~/ghidra-projects/export/c2/` is dated **2026-08-04**, nineteen days before
this lane; §7.5 states exactly what that costs.

---

## 0. WHAT WAS LOOKED AT BEFORE THIS FILE WAS WRITTEN

Stated exactly, because the prereg tier is worth nothing if the boundary is
vague. Before writing this file the lane read:

* the prose record: [`ref/P_DAG.md`](ref/P_DAG.md) in full (272 lines),
  `READ_PLAN_2026-08-21.md` §1–§5, `CLAUDE.md`, `docs/rungs/README.md`;
* the two fitted residuals **as source**: `crates/c2-core/src/codegen/schedule.rs`
  lines 1–140 and `crates/c2-core/src/codegen/order.rs` lines 1–160 — i.e. both
  module doc comments in full, including the two "the read that would replace
  this module's fit" blocks that name R7 by name;
* the **instrument**, as source and prose: `c2host/stagetap.c`,
  `c2host/stagetap.h`, `c2host/README.md`,
  `crates/c2-harness/src/cli/stage.rs`, `crates/c2-reference/src/stage.rs`,
  `docs/rungs/2026-08-20-stageoracle.md` §3/§6.1, the `w-restim` rung;
* `docs/whitebox/scripts/dagorder_sim.py` **by description only** (what it
  simulates, its inputs, its published 7/8 — the file itself is opened during
  the work, not before);
* house style: `WB_GLOBREGS_PREREG.md`, `ref/PREREG.md`, `WB_ENCODE_*`;
* `docs/BOARD.md` rows #541, #1823, #3067–#3071, #3242, #3257, #3259, #3423.

**No instruction of `0x10be5df6` (the priority pass), `0x10be5cea` (the
ready-list compare), `0x10be60c0` (the cycle loop) or `0x10c1bfe2` (the issue
predicate) has been read by this lane, no byte of `0x10c3bf9c`,
`0x10c3c1a8`, `0x10b202b0` or `0x10b221d0` has been extracted, and no tap has
been run.** Every prediction below is made from `P_DAG.md`'s prose and the two
Rust doc comments only.

### 0.1 Dispatch defect check — the brief's addresses, scored up front

Six lanes running, every coordinator-supplied address list has needed
correction. R7's brief supplies no *new* addresses; it supplies **pointers into
`P_DAG.md`**, and those are what get scored:

| the brief says | verified before this file was written |
|---|---|
| `ref/P_DAG.md:185` = the priority formula | ✅ the formula block is at **`:185-190`** (§3). Correct. |
| `ref/P_DAG.md:235-246` = the latency matrix | ✅ §5's latency table occupies **`:235-246`**. Correct. |
| `schedule.rs:42-46` = the 13,104-config residual | ✅ `:42-46` is *"It is **not a machine scheduler** … 13,104 list schedulers …"*. Correct. |
| `order.rs:89-95` = the 1,048,576-config residual | ⚠️ **off by ~50 lines.** `:89-95` is the `MAX_MULTISYM_PRODUCERS` / `MAX_SYMBOL_CROSSINGS` const block. The 1,048,576 search is at **`order.rs:139-146`**, and R7's second citation of it (the doc-comment pointer to this lane) is at **`order.rs:16-21`**. Corrected here, before use. |

**Three of four verify; one is wrong and is corrected rather than carried.**
Recorded here so it cannot be confused with a result produced after looking.

---

## 1. Prior art this lane must NOT re-derive

`grep -ril` over `docs/` and `scripts/` for `schedul`, and `docs/BOARD.md`
searched separately by topic, **before this file was written** — the brief's
rule, and the reason four lanes have walked into an open row. The oldest hit
(`docs/CODEGEN_W6_COMPARE.md`, 2026-07-29) was read last.

**Checked and confirmed absent: there is no `ref/P_SCHED.md` and no
`WB_SCHED_*.md`.** The scheduler's reference page is `ref/P_DAG.md` and its
findings page is `WB_DAGORDER_FINDINGS.md`. This lane therefore creates
`WB_SCHEDCONF_*` and **amends** `ref/P_DAG.md` rather than forking a rival
page — the coordinator has twice briefed a lane to write a document that
already existed, and `P_DAG.md` at 272 lines is that document here.

| already held | where |
|---|---|
| there IS a list scheduler; #1823 is refuted; driver `0x10be6382`, 4 runs/function at `/O1` | `P_DAG.md` §1, board **#3067** — `[O]` by four patched call sites |
| the four runs are gated by `DAT_10c2e2fc` **and** `[esi+0x1c] & 1`; `/Od` ⇒ zero | `P_DAG.md` §1's correction box |
| priority `= (height<<13) + (fanout<<8) + (symdest<<10)`; `height = 1 + max(succ.height + edge.latency)` | `P_DAG.md` §3 `[R]`, board **#3068** |
| ready-list compare = priority DESC (unsigned), then `node+0x44` ASC, where `node+0x44` is the original tuple index | `P_DAG.md` §2/§3 `[R]` |
| region: ≤ `0x50` tuples, ends at category ∈ {`0x12`,`0x14`,`0x19`,`0x1b`} or `0x17`-with-opcode-`0x30f`; **a call ends the region `[O]` 15/15** | `P_DAG.md` §2, board **#3069** |
| issue: width `DAT_10c3cf98` = 2 (or 4); unit 0 uncapped; else 1 insn/unit/cycle and ≤ 2 nonzero-unit insns/cycle | `P_DAG.md` §2 `[R]` |
| the 9 published edge latencies; ALU→mem-addr 5 is the largest; cmp→branch 0, other ALU→branch 2 | `P_DAG.md` §5 `[R]` + its revision box |
| `ORDER.md`'s fitted rank *(use count desc, first-use asc)* **is** this priority's second key + its tie-break; the store floor 2 **is** the ALU→store-data latency | `P_DAG.md` §4.4, board **#3070** |
| a **second author** of tuple order exists — the `factor.c` block merger `0x10b3baa8`→`0x10b3a790` | `P_DAG.md` §6, `WB_MERGER4_FINDINGS.md` |
| `dagorder_sim.py` scores the `unit` model **7/8** on 8 hand-encoded cells | `P_DAG.md` §2, `WB_DAGORDER_FINDINGS.md` |
| the tap's 8 sites; 4 are the scheduler runs, 1 is the region finder's sole call site; **none is inside the priority/cycle/issue machinery** | `c2host/stagetap.c:95-119` |
| the tap is **never a gate**; the obj byte-compare is the sole judge | `c2host/stagetap.h:26-28` |

**This lane's numerator is therefore the *confrontation*:** whether the model
above, executed, reproduces what c2 observably does; the completion of the
model where it is published incomplete; and whether the 13,104-configuration
search could have succeeded at all.

---

## 2. The grading rule

Tier **PREREG** by [`ref/PREREG.md`](ref/PREREG.md)'s ladder: committed to git
as the **first commit on `wt-w-read-r7`**, before a byte of the target was read
and before any tap was run. Each prediction is scored **HIT / MISS / PARTIAL /
UNGRADED** in `WB_SCHEDCONF_FINDINGS.md`. **Misses are reported as misses and
not smoothed**; a prediction too vague to be falsified earns nothing and is
marked UNGRADED rather than counted as a hit.

**Never a bare accuracy threshold.** A lane once registered "≥ 70 %", scored
82.5 %, and was *worse* than the 88.6 % reader it replaced. Every prediction
below therefore names **the incumbent number and the population it was
measured on**, and states how this lane's result must beat it — which for P4 is
explicitly **not** on rate.

**Denominators, declared now:**

* **model coverage** = `weight-table entries accounted for / 7` and
  `latency-matrix cells extracted / 121`;
* **region rule** = `regions whose partition matches the read rule / regions
  observed`, over ≥ 2 populations;
* **order agreement** = `regions where predicted order == observed order /
  regions graded`, reported **twice**: over all regions, and over the subset
  the scheduler actually **reordered** (§6.3 — this split is the whole ball
  game);
* **the 13,104 question** = a structural argument, graded by whether each named
  reason is *individually sufficient* and whether ≥ 1 **binds on the residual
  population** (the two-producer tier, 0 of 48).

---

## 3. THE TRAP THIS LANE IS DEFINED AGAINST

**The observable is a lossy quotient of the claim, and the loss is not
random.**

The tap reports tuple lists at **phase boundaries**. It never reports a ready
list, a cycle, a priority value, or an issue decision. Every claim about the
priority function, the latency matrix, the issue predicate, the ≤ 7 dynamic
bonus (`0x10c1bbaf`), the +15/+40 stall penalties (`0x10c1ba6f`) and the
schedule **iteration** (`0x10c1bdff`) is graded *only through its effect on a
final order*. Two different cycle models that induce the same order are
**indistinguishable to this instrument**, and an agreement is therefore
consistent with c2 running a different algorithm that happens to be
order-equivalent on the population measured.

**This is the `/QXSTALLS` shape, and this lane names its exact instance up
front.** That reading was +76.25 pp, had a live negative control, and was still
an artifact, because the annotation was nearly a function of body length while
the port's class was leaves and tail calls — stratified by exact instruction
count, the two populations were indistinguishable at every length ≥ 3. The
three instances here:

1. **Identity permutations are free hits.** A region the scheduler leaves
   alone is predicted correctly by *any* model that is order-preserving on
   easy input, including a model that does nothing. If those are pooled into
   the agreement rate the rate measures the population, not the model. §6.3.
2. **Ungraded cells are not missing at random.** The walk emits no tuple
   identity, so a permutation is matched by opcode sequence and is ambiguous
   when opcodes repeat. Ambiguity rises with region length and with opcode
   repetition — so the graded set is biased toward **short, opcode-diverse**
   regions, which is exactly where a scheduler has least to do. §6.2.
3. **Region length is the confound.** Every discrimination this lane reports
   is stratified by region length in tuples **before** it is believed, and no
   cell is quoted unless a size-matched comparison exists. §6.4.

---

## 4. Predictions

Each row: the claim, the **incumbent it must beat**, the hit/miss criterion,
and a registered confidence. Confidences are the lane's own and are scored for
calibration in the findings.

### P1 — the published priority formula is INCOMPLETE

`P_DAG.md` §2.1 records the weight table `0x10c3bf9c` as **seven** shorts
`[-1, 13, 8, -1, -2, 10, 0]`, and §3 publishes a **three**-term formula. Three
of seven weights are accounted for (`13`→height, `8`→fanout, `10`→symdest).
The remaining four — two `-1`, one `-2`, one `0` — are dismissed in one clause
covering only the two `-1`s. **A weight of `0` shifts its term by zero, i.e.
contributes the term itself at the bottom of the priority word**, which if live
is a fourth key that nothing in this repo models.

> **P1.1** — the priority pass `0x10be5df6` is shown to read the table as a
> **total function**: all **7/7** entries accounted for, each either (a) live,
> with its term named and the reading instruction's VA, or (b) dead, with the
> instruction that makes it dead.
> **Incumbent: 3 of 7 (42.9 %), `P_DAG.md` §3.** HIT = 7/7 accounted.
> PARTIAL = 4–6. MISS = ≤ 3, or the table turns out not to be 7 entries.
> *Confidence 0.7.*

> **P1.2** — **at least one term with a non-negative weight is live and is
> NOT named in `P_DAG.md` §3.** HIT = ≥ 1 such term named with its VA.
> MISS = exactly the three published terms are live, in which case **`P_DAG.md`
> §3 is VINDICATED** and this lane says so in those words — a real and
> reportable outcome, not a shortfall.
> *Confidence 0.55.* Registered as the prediction most likely to fail.

### P2 — the fanout field ALIASES the symdest field

`(fanout << 8) + (symdest << 10)` places the two terms **4 apart in the same
word**, and they are summed, not concatenated. Unless `fanout` is masked or
saturated below 4 before the shift, `fanout = 4` contributes exactly what
`symdest = 1` contributes, and `fanout ≥ 32` carries into `height`.
`P_DAG.md` §3 presents the three as separable keys and says nothing about this.

> **P2.1** — no mask/clamp is applied to the fanout term before its shift, so
> the aliasing is real in the arithmetic. HIT = the instruction stream shows
> the unmasked add. MISS = a mask, clamp or saturate exists (which would be a
> **correction to `P_DAG.md`'s formula** and is equally reportable).
> *Confidence 0.5.*
>
> **P2.2 — reachability, which is what makes it a defect rather than a
> curiosity.** Nodes with fanout ≥ 4 **occur** in the graded population.
> HIT = ≥ 1 observed, with the count and the population named.
> MISS = 0 observed over ≥ 100 regions, in which case the aliasing is real and
> **unreachable on this workload**, and this lane says that instead.
> *Confidence 0.7.* The fanout counter's storage width (`node+0x26`, bumped at
> `0x10b32113`) bounds this and is read as part of P2.

**Incumbent for P2: none — no prior statement exists.** The bar is therefore
not a number but falsifiability: a claim that cannot go red is UNGRADED.

### P3 — the region-finder rule, promoted or refuted

> **P3.1** — the observed region partition matches the read rule (length
> ≤ `0x50`; boundary tuple category ∈ {`0x12`,`0x14`,`0x19`,`0x1b`} or category
> `0x17` with opcode `0x30f`) on ≥ 95 % of observed regions.
> **Incumbent: `[O]` 15/15 — but only for the *call* clause, on 15 cells. The
> cap clause and the four categories are graded on ZERO cells.** This lane must
> beat it on **denominator and on scope**: ≥ 100 regions, ≥ 2 populations, all
> clauses. A 100 % on 15 cells is not beaten by a 100 % on 15 cells.
> HIT = ≥ 100 regions, ≥ 2 populations, ≥ 95 %. PARTIAL = ≥ 100 regions but
> < 95 %, with the residual familied. MISS = < 100 regions obtainable.
> *Confidence 0.65.*

> **P3.2 — the `0x50` cap never binds.** `P_DAG.md` §6 lists *"whether the
> region-finder's `0x50`-tuple cap ever binds in practice"* as **unmeasured**.
> Registered direction: **0 regions of exactly 80 tuples**, so the cap is
> reachable in code and unreached on this workload.
> HIT = 0 observed and ≥ 100 regions graded (the row moves from *unmeasured* to
> *measured-and-slack*). MISS = ≥ 1 region at the cap.
> *Confidence 0.8.*

### P4 — THE CONFRONTATION: does the read model reproduce c2's order?

The deliverable. A simulator executing **the read model as published** (region
finder → DAG → height fixpoint → priority → ready-list sort → cycle issue under
the issue predicate) is run on regions extracted from the live tap, and its
predicted order is compared with the order c2 actually produced.

> **P4.1 — the population beats the incumbent even if the rate does not.**
> **Incumbent: `dagorder_sim.py`, 7 of 8 hand-encoded cells (87.5 %).** Eight
> hand-encoded discovery cells is not a rate; it is eight cells, chosen by the
> lane that also chose the model. This lane must beat it on **denominator**:
> ≥ 100 machine-extracted regions, from ≥ 2 populations, none hand-encoded.
> HIT = ≥ 100 regions graded from tap output. MISS = < 100.
> *Confidence 0.6.*

> **P4.2 — the rate FALLS.** Registered explicitly and in the honest
> direction: exact-order agreement on the machine-extracted population will be
> **below 87.5 %**, because the incumbent's cells were the discovery set.
> HIT = agreement < 87.5 % on reordered regions (§6.3's denominator).
> MISS = ≥ 87.5 %, which would mean the model generalises better than its
> author's own cells and is a *stronger* result than predicted.
> *Confidence 0.75.*

> **P4.3 — the disagreements form ≤ 4 named families**, each with a mechanism
> and ≥ 1 cited region. HIT = ≤ 4 families covering ≥ 80 % of disagreements.
> PARTIAL = families named but coverage < 80 %. MISS = residual unfamilied —
> in which case the lane reports **"the model is refuted and the refutation is
> unstructured"**, which is worse than a clean refutation and is said plainly.
> *Confidence 0.5.*

> **P4.4 — the verdict word.** The findings will contain exactly one of
> **"the model SURVIVED the tap"** or **"the model was REFUTED by the tap"**,
> with its denominator on the same line. Registered now so the lane cannot
> retreat into a compound headline. *No confidence — this is a fence.*

### P5 — could the 13,104-configuration search have SUCCEEDED at all?

The brief's central question, and the R4 precedent: R4 found a
52,416-configuration null was **structurally guaranteed**, because the search
ranged over priority functions while the answer was stamped by a different
phase entirely.

> **P5.1** — **NO**, and for **≥ 3 independent structural reasons, each
> individually sufficient.**
> **Incumbent: `schedule.rs:56-61`'s own one-line assertion** — *"Rule 2 is
> not a priority function, so no member of that family can express it."* That
> sentence already claims the conclusion, so restating it earns **nothing**.
> HIT = ≥ 3 reasons the file does not give, each named with the c2 mechanism
> and address that creates it, each shown individually sufficient.
> PARTIAL = 1–2. MISS = the search family is found to *contain* c2's rule —
> i.e. the search **could** have succeeded and merely did not, which is a
> larger and more uncomfortable result and is reported as such.
> *Confidence 0.75 for NO; 0.1 that the family contains the rule.*

> **P5.2 — at least one reason BINDS on the residual population.** A reason
> that is structurally sufficient but never active on the two-producer tier
> (0 of 48) explains nothing about why *that* tier is the residual. HIT = ≥ 1
> reason shown active on the two-producer shape specifically.
> MISS = all reasons are generic.
> *Confidence 0.5.* This is the R4-shaped half and the harder half.

> **P5.3 — the 1,048,576-configuration search (`order.rs:139-146`) gets the
> same verdict, or an explicitly different one with a reason.** No silent
> generalisation from one search to the other.
> HIT = an explicit verdict for each, separately argued. MISS = one verdict
> asserted to cover both. *Confidence 0.7.*

### P6 — the latency matrix as data, and P_DAG graded against its own raw bytes

> **P6.1** — the 11×11 matrix at `0x10c3c1a8`, the class-index table
> `0x10b221d0`, the weight table `0x10c3bf9c` and the per-opcode machine table
> `0x10b202b0` (stride 12) are extracted from the **pinned DLL** by a
> committed, re-runnable, sha256-fenced script, and published as data.
> HIT = all four dumped with the fence. MISS = any not obtainable.
> *Confidence 0.85.*

> **P6.2 — the element width is NOT stated by `P_DAG.md` and a wrong guess
> produces a plausible-looking matrix.** The page says "the 11×11 edge-latency
> matrix" and never says whether a cell is a byte, short or dword. Registered
> as an **instrument-lies check**: the width is *derived*, not assumed, by
> requiring that the 9 latencies `P_DAG.md` §5 publishes in prose reproduce at
> the derived width, and by requiring `121 · width` not to overrun the next
> known datum. HIT = width derived with both checks green.
> MISS = no width reproduces §5, which **refutes either the matrix address or
> §5's numbers** and is reported as a correction to `P_DAG.md`.
> *Confidence 0.6 that the width is 1 byte; 0.9 that some width reproduces.*

> **P6.3** — all **9** prose latencies in `P_DAG.md` §5 reproduce from the raw
> bytes. HIT = 9/9. PARTIAL = 6–8, with each disagreement reported as a
> correction. MISS = ≤ 5. *Confidence 0.7.*

### P7 — reach, fence, and the null this lane must return

> **P7.1** — reach **0**: no fixture converts, the census moves **+0**, and
> **zero bytes** change under `crates/`. HIT = all three. This is a fence: if
> the confrontation implies a `crates/` change (e.g. that `schedule.rs`'s fit
> should be replaced, or that `MAX_MODELLED_PRODUCERS = 3` has a read
> provenance), it is **REPORTED as a finding for a follow-up lane and not
> edited here**. *Confidence 0.95 — the residual 0.05 is the risk that a
> `crates/` doc-comment correction looks irresistible; it is still forbidden.*

> **P7.2** — `DISCLOSURE.md` gains **0 rows**, because that file is the ledger
> of findings **adopted into `crates/`** and this lane adopts nothing. R1, R2
> and R3 each predicted a row and each correctly produced none; this lane
> registers the null up front rather than discovering it. HIT = 0 rows.
> *Confidence 0.9.*

---

## 5. Controls

### 5.1 Positive control — is this the instrument the prior lanes used?

`c2rs stage counts` publishes an invariant: the three in-band scheduler sites
fire equally and `color == globregs == in-band`. **If this lane's tap run does
not reproduce that invariant, the instrument is not the one the record was
built with and every tap-derived number here is VOID.** Run first, quoted in
the findings.

### 5.2 Negative control — `/Od` fires no scheduler

`P_DAG.md` §1: at `/Od` the optimizer gate `DAT_10c2e2fc` is reached first and
**none** of the four runs happen. So at `/Od` the `region` site must fire
**zero** times. A nonzero count means the site attribution is wrong.

> **And this control is NECESSARY, NOT SUFFICIENT, which is the whole
> `/QXSTALLS` lesson.** That reading had a live negative control and was still
> an artifact. This one tests **site attribution** — "am I watching the
> scheduler?" — and says nothing whatever about **order attribution** — "did
> the scheduler author this order?", which is the claim at issue, and which
> `P_DAG.md` §6's second author (`0x10b3baa8` → `0x10b3a790`) is specifically
> able to falsify. Registered so a green `/Od` cell is never quoted as support
> for P4.

### 5.3 Shuffle control — does the model model anything?

Feed the simulator a **randomly permuted** input order for each region and
re-score. If agreement does not collapse toward chance, the "model" is
tracking the input order rather than the schedule, and P4 is void.
Registered threshold: shuffled agreement must fall by ≥ 30 pp on the reordered
subset. Seeded and committed so it is reproducible.

### 5.4 Cross-population control

**Grids and corpora fail in opposite directions, so both run**, and the
fixtures are crossed against each other besides:

* **Population G (grid)** — a frozen `.cpp` grid, content-sha256 pinned before
  compilation (the `wb-dagorder` protocol), sweeping region shape, producer
  count, fanout, and opcode class. Grids fail by being unrepresentative.
* **Population C (corpus)** — the repo's existing `fixtures/` tree through the
  tap, unselected. Corpora fail by being unbalanced and by hiding the axis.

A claim holding on exactly one population is reported as holding on exactly
one population.

### 5.5 Holdout, declared before the model is fitted

**Population G is partitioned now, before the grid is written**: cells whose
index is `≡ 0 (mod 3)` are **HOLDOUT** and are not looked at until the model is
frozen. The generator writes the partition into a file the scorer refuses to
open until the freeze commit. This is `w-order2`'s protocol and it is adopted
verbatim.

---

## 6. Method, and the four ways it can lie

### 6.1 What the tap actually gives

Verified from `stagetap.c` before this file was written, not assumed:

* the `region` site `0x10be643e` is the region finder's **sole** call site and
  fires at **entry**, with `ecx` = the region's first tuple; with payload on it
  emits the tuple list forward down `+0x0` as
  `TU <i> <opcode> <cat> <flags> <cc>` (`stagetap.c:544-567`);
* the walk runs to end-of-**list**, not end-of-region, so region *k* re-reads
  regions *k..n*. **The region boundaries are therefore recovered as the
  differences of successive walk lengths within one `(function, phase)`** —
  `len(walk_k) − len(walk_{k+1})` is region *k*'s length, and the tuple at that
  offset is the boundary. This is the P3 instrument and it needs no new tap
  code;
* `C2RS_STAGE_FUNCWALK=1` gives the whole-function walk (`FN`/`BLK`/`FT`),
  per block, backward down `+0x10` and re-reversed by the Rust parser — so
  **within a block the tuple list order is observable**, at `sched0` (before
  run 4) and at `after0` (after it). That pair is the P4 instrument;
* `C2RS_STAGE_OPS=1` exposes the operand records, which is what makes a
  dependence DAG constructible at all.

### 6.2 The identity problem, and why its exclusions are biased

`FT`/`TU` rows carry no tuple address, so a pre/post permutation is matched by
**opcode sequence**. Where opcodes repeat the match is ambiguous. Ambiguous
regions are **UNGRADED**, never counted as agreements — and because ambiguity
grows with length and repetition, the exclusion is **not random**. The findings
publish the exclusion rate **stratified by region length**, and any agreement
rate is read against it.

### 6.3 The identity-permutation split — the single most important number

Registered now: agreement is reported **twice**.

| denominator | why |
|---|---|
| **all graded regions** | the headline anyone will quote |
| **regions the scheduler REORDERED** (observed post ≠ observed pre) | the only one that grades a model |

A region the scheduler left alone is a **free hit** for any order-preserving
model, including one that returns its input. If the reordered fraction is
small, the all-regions rate measures the population and not the model, and the
findings say so with the fraction on the same line. **This is `/QXSTALLS` in
its exact shape for this lane.**

### 6.4 Stratification, mandatory

Every table discriminating any two populations is stratified by **region length
in tuples**, and **no cell is quoted unless a size-matched comparison
exists**. Registered prediction, graded:

> **P6.4** — at least one apparent discrimination visible in the pooled
> numbers **vanishes under length stratification**. HIT = ≥ 1 such collapse
> found and reported. MISS = none found, in which case the lane reports that
> the stratification found nothing, which is also a result.
> *Confidence 0.5.*

---

## 7. WHAT THE CONTROLS ARE STRUCTURALLY INCAPABLE OF CATCHING

Registered before the work, because it is the part a findings document written
afterwards is least able to write honestly.

**7.1 Underdetermination — the ceiling on every P4 result.** Order agreement
cannot distinguish c2's priority function from **any** function that induces
the same order on the graded population. A HIT on P4 is evidence the model is
*order-equivalent to* c2 here; it is not evidence it *is* c2's algorithm. No
enlargement of the population removes this — only reading the priority pass
does, and that is `[R]` evidence of a different kind. **The findings may
therefore never say the model is "confirmed", only that it "survived".**

**7.2 The cycle model is graded through a lossy quotient.** Issue width, the
≤ 2-nonzero-unit cap, the ≤ 7 dynamic bonus (`0x10c1bbaf`), the +15/+40 stall
penalties (`0x10c1ba6f`) and the **iteration** (`0x10c1bdff`) are all observed
only through their effect on a final order. Two cycle models producing the same
order are indistinguishable. In particular **this lane cannot detect that a
schedule was iterated** — it sees the fixed point, never the passes.

**7.3 The second author.** `P_DAG.md` §6: the `factor.c` block merger
`0x10b3baa8` → `0x10b3a790` also moves tuples and is **not** a DAG client. An
order difference this lane attributes to the scheduler may be the merger's, and
a scheduler-boundary snapshot cannot separate them. Mitigation, not solution:
report agreement stratified by whether the region abuts a block boundary.
**If this lane graded only single-block leaf regions it would not detect this
at all** — so the holdout is required to contain block-boundary-adjacent
regions, and the findings state the count.

**7.4 The mid-level pass.** Runs 1–3 are mode 1 and run 4 is mode 0
(post-lowering). `P_DAG.md` §6 lists the mid-level pass's differences as
unknown. The `sched0`/`after0` pair grades **run 4 only**. Anything true of
runs 1–3 and false of run 4 is invisible here.

**7.5 Code date vs data date.** Data claims (P1, P2, P6) come from the
**pinned DLL**, sha256-fenced in the extraction script, and are sound. Code
claims inherit the flat export's **2026-08-04** date. The digest check makes it
very likely the export was built from the pinned image; it does not prove it.
Any code claim is void if it was not.

**7.6 Zero `crates/` bytes means the byte judge never sees this.** The sole
judge of the port is `port(IL) == c2(IL)` byte-exact. **Nothing this lane
produces is evidence the port would emit anything correctly**, and a survived
model is not a shipped model.

**7.7 The `0x50` cap null is a workload statement.** P3.2 predicts the cap
never binds *on the populations measured*. A grid is chosen and a corpus is
what it is; neither can show the cap is unreachable on the 878-TU workload,
still less in general.

---

## 8. What would make this lane DECLINE, named now

A decline is priced and reported, not smoothed into a partial.

1. **c2host will not build or the tap will not arm** (mingw absent, wibo stale,
   slide resolution fails) → P3, P4, P6.4 and every control in §5.1–§5.4 are
   **declined**. The lane still lands P1, P2, P5, P6.1–P6.3 — the plan's named
   deliverable is *"the list scheduler as pseudocode + the latency matrix as
   data"*, and that half needs no tap. Outcome would still be `built`.
2. **§5.1's positive control fails** → every tap number is VOID and is deleted
   rather than caveated.
3. **Fewer than 100 regions are obtainable**, or walk-refusals dominate → P3.1
   and P4.1 are declined by their own stated criteria.
4. **The reordered subset (§6.3) is empty or trivially small** → P4 is reported
   as **UNGRADED with the fraction published**, not as a high agreement rate.
   Registered explicitly because this is the failure mode most likely to
   masquerade as success.
5. **Time.** Priced 3–5 days. If P4 cannot complete, the lane lands the
   evidence-complete increment and states the coverage as a fraction of the
   named population, per the brief.

---

## 9. The confirmation probe, and whether it can go red

**Design.** After the model is frozen, score the **HOLDOUT** partition (§5.5)
plus the corpus population C, neither used in any tuning, on:

* (a) exact-order agreement on the reordered subset;
* (b) the region-partition rule, all clauses;
* (c) the 9 `P_DAG.md` §5 latencies reproduced from raw bytes at the derived
  width.

**Named failure modes.** (a) collapses vs the graded set ⇒ the model was tuned
to the graded set. (b) fails ⇒ the region rule is population-specific, not a
rule. (c) fails ⇒ the extraction width or the matrix address is wrong and every
latency number in this lane is wrong with it.

**"Would this go red if the claim were false in the most likely way?"** The
most likely way the claim is false is **§7.3**: the order on real code is
authored partly by the block merger, not the scheduler. Then the model agrees
on short, call-free, single-block regions and disagrees systematically on
regions abutting block boundaries. A holdout of only single-block leaf regions
would show a clean green and be **wrong** — the identical shape to `/QXSTALLS`.

**So the probe is only capable of failing if the holdout contains
block-boundary-adjacent regions, and the findings must publish that count
before the agreement rate.** Registered as a binding condition: if the holdout
contains **zero** such regions, the confirmation probe is reported as
**VACUOUS**, not as passed.

---

## 10. Deliverables

1. **`ref/P_DAG.md` amended in place** — the model completed where §3 is
   incomplete, each row moved `[R]` → `[O]` or `[R]` → **refuted**, with the
   witness named. No rival page is created; §1's grep established `P_DAG.md`
   is the page.
2. **The list scheduler as pseudocode** — the plan's own words — at a level a
   port could implement, with every constant addressed.
3. **The latency matrix as data** — `docs/whitebox/ref/SCHED_LATENCY.tsv` and
   the machine/weight tables, extracted by a committed sha256-fenced script
   under `docs/whitebox/scripts/`.
4. **`WB_SCHEDCONF_FINDINGS.md`** — the grade, every prediction scored, the
   verdict word of P4.4, and the §7 list re-stated as *what this lane's
   evidence cannot show*.
5. **Board rows #3433–#3436** and a rung under `docs/rungs/`.

**Scratch** lives in `work/w-read-r7/` and is gitignored. No IL, no `.obj`, no
absolute paths, no AI trailer.
