# Step 5, re-priced per stage — `w-restim`, 2026-08-21

The re-estimation `docs/ARCHITECTURE_PROPOSAL_2026-08-20.md` §6 promised
(*"re-estimate after the stage oracle and after the plan manifest"*) and
`docs/ARCH_REVIEW_2026-08-21.md` consequence 3 dispatched. Every number below
is derived from a log committed under `work/w-restim/`; predictions were frozen
in `work/w-restim/PREREG.md` before any probe ran.

**Read every forward cost figure here as a LOWER BOUND.** `docs/CEILING.md` §5:
the project's pre-registered estimates miss **optimistic roughly 5:1**, and the
misses are specifically on *forward cost*, not on measurement. That calibration
is **applied** below (each row carries a raw figure and a ×5 lower bound), not
merely cited — which was one of the review's amendments.

---

## 0. The headline, and it is a split verdict

| question | answer | changed by |
|---|---|---|
| Is COLOR gradeable **against real c2**? | **YES.** Its output is the operand's symbol pointer being rewritten from a candidate record to a physical-register record, and the register number is directly readable | probe A |
| Is the final schedule (run 4) gradeable **against real c2**? | **YES.** It moves the list on 149 of 2,946 functions, and that was observed nowhere before | probe B |
| Can a **PORT** be graded per-stage against those observations? | **NO**, and not by a margin — the two have no common coordinate. c2 cuts regions in *tuple index at a pre-lowering phase*; on four byte-exact functions c2 has 19–22 tuples where the port emits 4–5 instructions | probe C |

So step 5's grading premise fails **in one direction only**, and that is the
whole re-pricing. Reading c2's per-stage behaviour is now **cheap and
mechanised**. Grading a ported pass per-stage still requires reproducing c2's
region decomposition and its tuple coordinate system — arch review finding 1's
last bullet, confirmed, and finding 4 (a)/(c) turned into a measured ratio.

**The consequence for the plan is not "step 5 is impossible". It is that
per-stage observability buys CHARACTERIZATION, not a per-stage differential
grade** — and characterization was already the expensive half (`CEILING` §6.1
item F0 is 8 of item F's 17 lanes and is exactly *"the order that decided the
registers does not appear in the obj"*).

---

## 1. The per-stage table — measured, 384 fixtures, 2,946 function-pairs per bracket

`work/w-restim/probeAB_all_fixtures.log`, at the eight-site table with the
operand walk and the whole-function walk on. Every bracket is *the same
traversal read at two adjacent phases*; SPINE is opcode/category/flags/cc,
FULL adds the operand and symbol/candidate records.

| stage | bracket | spine moves | operand-only moves | total | gradeable vs real c2 |
|---|---|---:|---:|---:|---|
| scheduler run 1 | `sched1`→`globregs` | 453 (15.4%) | 35 (1.2%) | **488 (16.6%)** | YES |
| **globregs** | `globregs`→`sched2` | 2,766 (93.9%) | 36 (1.2%) | **2,802 (95.1%)** | YES |
| scheduler run 2 | `sched2`→`color` | 171 (5.8%) | 4 (0.1%) | **175 (5.9%)** | YES |
| **COLOR** | `color`→`sched3` | 130 (4.4%) | **771 (26.2%)** | **901 (30.6%)** | **YES — 85.6% of its visible footprint is operand-only and was invisible before this lane** |
| run 3 **+ the lowering band** | `sched3`→`sched0` | 2,946 (100%) | 0 | **2,946 (100%)** | YES but **NOT SEPARABLE** — no tap site exists between them |
| **final schedule (run 4)** | `sched0`→`after0` | 118 (4.0%) | 31 (1.1%) | **149 (5.1%)** | **YES — newly observable, site `0x10b7e701`** |

Two readings that are NOT in this table and would be wrong:

* **"COLOR is a no-op on 69.4% of functions"** is what the IDENTICAL column
  says, and it is a fact about the *fixtures*, not about the allocator: a
  function whose every value already sits in its arrival register has nothing
  for the allocator to write.
* **`#3323`'s "IDENTICAL 7/7 · offsets COLOR writes: NONE"** is unmoved and
  reproduces exactly at this tip — on `il_call_perm.cpp`, where COLOR really is
  a no-op on all seven functions. It does not generalise, and the sweep is how
  we know: 105 of the 292 fixtures with an aligned raw window name at least one
  offset (`work/w-restim/snap_all_fixtures_verdicts.log`).

### 1.1 What COLOR's write actually IS

Before (`color` site, i.e. entering the allocator) and after (`sched3` site),
`add3.cpp` fn1, verbatim from `work/w-restim/probeA_add3.log`:

    BEFORE: 00000272 0d 01 04 | OP D 0 01 1004 01 00000004 unread 00000004 | OP S 0 01 1004 02 00000002 none 00000002
    AFTER : 00000272 0d 01 04 | OP D 0 01 1004 01 00000004 unread 00000004 | OP S 0 01 1004 01 00000004 unread 00000004

The source operand's symbol goes from **kind `02`** (a candidate, id 2, assigned
descriptor `none`) to **kind `01`** (a physical register record, value `4`).
Register `4` is `r3` on the `0x10b181c0` table in the `n = r+1` encoding. On the
last differing row of the same function, candidates `{2,3,4}` become registers
`{4,5,0xc}` = `r3, r4, r11`.

**This answers `docs/rungs/2026-08-20-stageoracle.md` §6.1 q1 in full**, and it
corrects a reading on the way: `P_REGALLOC.md` §4.1's `+0x1c` is the candidate
**id**, not a register — the register is one further hop, and the allocator does
not write it into the candidate the operand already points at. It re-points the
operand.

---

## 2. The two unbudgeted integration prerequisites, priced as their own rows

Arch review finding 3. Neither appears in any `§5` row of the proposal, and both
are on the critical path of **any byte-judged output** from a ported pass.

| # | row | why it is a prerequisite | raw estimate | **×5 lower bound** |
|---|---|---|---:|---:|
| **I1** | **general op-level IL decode** | IR0 stops at a two-variant byte framing; `BodyShape`'s 35 whole-function grammars are simultaneously the admission gate. A ported COLOR consumes a *semantic middle* that does not exist. Probe A sharpens this: c2's own middle is a tuple list with per-tuple categories and two operand lists, and the port has neither — probe C measures the gap at **4.4–4.75× more tuples than emitted instructions** on four byte-exact functions | 1.5–4.5 eng-months (reviewer's 3–9 for the pair, split by the reviewer's own ordering) | **7.5–22.5 eng-months** |
| **I2** | **general lowering to `coff::Function`** | today a 35-arm per-shape dispatch (`comdat.rs`, 43 `Selected::` refs). Without it a ported pass's only progress signal is stage-parity — **#3336 at program scale**, an instrument with no emit-path consumer | 1.5–4.5 eng-months | **7.5–22.5 eng-months** |
| | **I1 + I2** | | **3–9 eng-months** | **15–45 eng-months** |

**Priced two-sided, per `CLAUDE.md`.** The cost of *not* doing them is not zero
and is not "step 5 is slower": it is that every step-5 lane lands an
unconsumable instrument, and the project has a measured precedent for exactly
that failure at one-lane scale (**#3336**, `ir0`: a required-zero byte delta
that held *by construction* because the tree had no production caller). At
program scale the same shape has no contrast case to catch it. The review's
prophylactic — *every step-5 lane names in its rung header which
`coff::Function` field its pass would eventually write* — is adopted here as the
minimum, and it is a smoke alarm, not a substitute.

---

## 3. Characterization cost, re-priced against what the probes now cost

The unit is a **lane** (one worktree, one rung). Anchor: `CEILING` §6.2 — the
last five TU conversions cost **~17 landed rungs each**.

| stage | characterization still owed | lanes (raw) | **×5 lower bound** |
|---|---|---:|---:|
| scheduler runs 1–4 | **0 for observability** — this lane built it. Still owed: the *rule* (priority, ready-list order, cycle model) which `P_DAG.md` §3/§5 already holds at `[R]`, now checkable against a live pre/post pair | 0 + 1 to convert `P_DAG` §3/§5 from `[R]` to `[O]` | **5** |
| **the lowering band** | **1 lane, and it is the one gap this lane leaves open.** `sched3`→`sched0` conflates scheduler run 3 with the whole lowering band; there is no site between them, so 100% of that bracket moves and none of it is attributable | 1 | **5** |
| **globregs** | `P_REGALLOC.md` §7: **F1**, `0x10b55732`'s promotion policy, **unread**. The bracket moves on 95.1% of functions, so the observable is loud — the reading is what is missing | 2 (`CEILING` §6.1's own F1 price) | **10** |
| **COLOR** | **0 for observability** — probe A. Still owed: the candidate ORDER, which `CEILING` §6.1 prices at F5 = 2 and states is **not separable from F0** | 2 | **10** |
| | | **5–6** | **25–30** |

**What probes A and B did to F0.** `CEILING` §6.1 prices **F0 — "the order the
allocator is handed, and the four stages after it" — at 8 lanes**, the largest
single line in item F, and its justification is exactly *"the order that decided
the registers does not appear in the obj"*. That order is now **directly
readable at six phases plus after run 4**, and the register assignment is
readable beside it. F0's cost changes in KIND: from a black-box search over
obj-visible consequences to a differential read against a live trace.

**It does not go to zero, and pretending otherwise is the optimism §5 warns
about.** Two things survive:

1. Reading a trace is not deriving a rule. `P_DAG.md` §3's priority formula and
   §5's machine model are still `[R]`, and turning them `[O]` is what the lane
   above prices.
2. **Probe C's residue is F0's residue.** A rule read off c2's trace still has
   to be *implemented in the port*, and the port has no tuple, no category and
   no region — so the implementation lands in I1/I2's coordinate system, not in
   this one.

Re-priced honestly: **F0 8 → 4 lanes raw (×5 = 20)**, and the 4 that leave are
search lanes, not construction lanes. Item F's total goes **17 → 13 lanes raw
(×5 = 65)**.

---

## 4. The whole curve

| row | raw | **×5 lower bound** | on the critical path of a byte-judged output? |
|---|---:|---:|---|
| **I1** general op-level IL decode | 1.5–4.5 eng-mo | **7.5–22.5 eng-mo** | **YES** |
| **I2** general lowering to `coff::Function` | 1.5–4.5 eng-mo | **7.5–22.5 eng-mo** | **YES** |
| characterization, all stages (§3) | 5–6 lanes | **25–30 lanes** | no — it is the input to the construct rows |
| item F construct, re-priced (§3) | 13 lanes | **65 lanes** | YES |
| the lowering-band tap site | 1 lane | **5 lanes** | no |

**The critical path is INTEGRATION, not any single pass** — registered as E2 at
0.70 and it holds. I1+I2 alone are 15–45 engineer-months at the lower bound,
against which item F's 65 calibrated lanes are the *second* cost, and every
characterization lane in §3 is cheap by comparison.

**Against the proposal's raw 12–24 months for step 5**: the figure is not
obviously wrong in magnitude, but it is wrong in **composition** — it was
written for the passes, and the passes are not what dominates. E3 (registered
0.55, *"the calibrated total exceeds 12–24 months"*) is **not** cleanly
resolved by this table because the two rows are in different units (engineer-
months and lanes) and this lane declines to invent a conversion. What it does
resolve: **the 12–24 figure does not cover I1 and I2 at all**, and those two
rows alone are 15–45 engineer-months at the lower bound.

---

## 5. What this does NOT price, stated so absence is not read as coverage

* **No conversion.** Predicted reach 0, delivered 0. No fixture is claimed, no
  census number moves, `match 26 / mismatch 0` is unmoved by construction.
* **The thesis-vs-870 goal question.** `STRATEGY_REVIEW_2026-08-13.md:251` —
  *"The question is currently owned by nobody"* — is still true, and this
  document does not touch it. A cost curve is an input to that decision and
  never a substitute for it. Arch review consequence 1(c) stands.
* **IR3.** Arch review finding 4's minimal repair (give IR3 its own step in
  tuple/region coordinates) is the amendment lane's, not this one's. Probe C is
  evidence *for* it: the port→snapshot projection is undefined today, and
  probe C measures by how much.
* **A per-stage grade for a ported pass.** Probe C says NO on measured
  evidence. Any step-5 row that assumes one is mispriced, and the standing bound
  (`docs/rungs/2026-08-20-stageoracle.md` §8 — no `crates/` rule enters on
  snapshot equality) is what keeps that mistake out of the judge.
