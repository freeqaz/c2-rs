# PREREG — w-latent: how many shipped surfaces can be arbitrarily wrong with nothing able to observe it?

    Tag:       w-latent
    Slug:      w-latent
    Date:      2026-08-24
    Kind:      characterization
    Fixtures:  none — predicted reach 0
    Census:    +0
    Base:      67f276409
    Board:     #3512–#3516 (ninth-wave ledger, `docs/BOARD.md` decision 11)

Committed **before** any campaign measurement. One probe was run ahead of this
file and is disclosed in §7 as an instrument cost measurement, not a result.

---

## 1. The question, and why it is not "find dead code"

Board **#3493** (`w-permute` §3.2) found that the **write set** returned by
`crates/c2-core/src/codegen/calls.rs`' marshalling producers is **latent**:
three mutations that gut or poison it — one returning all eight `ARG_REGS` —
are byte-identical across 627 `c2-core` unit tests and 391 fixtures graded
against real `c2`, because two named refusals (`calls.rs:1303`,
`callseq-multiarg-sym`) stand between the producer and the only site that
reads it. The doc asserting Class B needed it had been false since those
refusals landed.

A latent surface is **not** dead code. Dead code is unreachable and says so.
A latent surface **runs**, is **documented as load-bearing**, and is **cited by
later work as a constraint** — and every observable this project owns agrees
with it, because no observable can see it. That is the sharpest form of #3336
(*a required-zero byte delta is silent about everything that is not a byte*),
and under goal (1) it is worse than a bug: the port is meant to be an
executable, tweakable model of `c2` whose decision points are named and
settable, so **a latent decision point is a lie told to the permuter**.

## 2. THE DENOMINATOR — declared, closed, and enumerable

This lane will **not** claim to have swept 199,362 lines. It declares three
closed populations and reports a verdict for every cell of the first two.

| axis | population | size | enumerated by |
|---|---|---|---|
| **A** | every **numeric `pub const`** in `crates/c2-core` (the byte-producing crate) | **33** | `grep -rn '^\s*pub const [A-Z_0-9]*: \(u8\|u16\|u32\|i32\|usize\|i16\|u64\) = ' crates/c2-core/src` |
| **B** | every field of every **named decision-point parameter struct** the DECISION-SURFACE CLAUSE has shipped into the byte-producing crates | **9** | `EncodeParams` (3), `SeedGapModel` (3), `SeedGapInputs` (2), `PlanInputs` (1) |
| **C** | **zero-/one-reference `pub` items** in `crates/c2-core` + `crates/c2-obj` — a *reading*, reported as a reading | 131 consts + 419 fns swept | reference count of `\b<name>\b` over `crates/` |
| **D** | **the size of the surface rustc's own liveness analysis cannot check** — a claim about the *mechanism*, with its denominator, not about any one item | whole workspace | see §2.1 |

### 2.1 Axis D — `pub` exempts an item from the compiler's own dead-code check

Registered as a **hypothesis to verify, not a fact**: rustc's `dead_code` lint
stops at the crate boundary, so a `pub` item in a library crate is reachable
*by assumption* and draws no warning however many callers it has. If that holds,
then **the entire `pub` surface of every crate here is outside the one automatic
control this project has for the orphan defect** — and `#3428`/`#3469`, which
this project has caught twice from build warnings, were both non-`pub` and could
only ever have been caught because they were non-`pub`.

This lane will (i) **verify the mechanism** on this workspace rather than assert
it from memory — by confirming that a demonstrably unreferenced `pub` item
produces no warning while a non-`pub` one does — and (ii) **report the
denominator**: how many `pub const`s and `pub fn`s exist per crate, i.e. how
large the unchecked surface is. Reported as a finding **separately from any
individual item**, because the mechanism is worth more than the row.

**What this lane does NOT cover, stated up front**: `crates/c2-il` (65k lines,
the parser/census layer), `crates/c2-harness` (owned by `w-joint` this wave),
`crates/c2-reference`, every non-`pub` item, every predicate, and every
`Vec`/set body in the port. Axis C's *sweep* is whole-crate; axes A and B's
*mutations* are not. The uncovered remainder is priced in §6.

## 3. THE LADDER — what counts as an observable

A surface is **latent** iff a mutation that is semantically different changes
**no** rung of this ladder. Every rung is a real instrument this project
already grades on; none is wall-clock.

| rung | command | what it can see | baseline at `67f276409` |
|---|---|---|---|
| **R1** | `cargo test -p c2-core --release --no-fail-fast` | the port's own unit tests | **627 passed / 0 failed**, 2 targets |
| **R2** | `cargo test --workspace --release --no-fail-fast` | + every integration test, incl. the **differential against real `c2`** on the fixtures | **1,866 passed / 0 failed / 1 ignored**, 54 targets, exit 0, **zero** `SKIP: toolchain absent` |
| **R3** | `sh scripts/gate.sh --jobs 4 --require-graded` | 18 mode lanes, ~7,000 fixture verdicts, sweep, cross, DEBUG profile | to be taken |
| **R4** | 878-TU workload scan | `gap-metric` keys, census keys, `fnbyte-exact` | to be taken |

**Escalation rule.** Every axis-A and axis-B cell is run at R1. A cell that goes
RED at R1 is **not-latent** and stops there — the observable that catches it is
named and the cell costs nothing further. A cell that is **GREEN at R1
escalates** to R2, and a cell green at R2 escalates to R3 (and R4 if the cell's
own shape says a census key is the only thing that could move). This spends the
expensive rungs only where a finding is possible, which is what makes a 33-cell
sweep affordable at all.

**Verdict vocabulary** — five values. **ORPHANED and LATENT are different
defects with different consequences and are reported in separate columns**
(coordinator, 2026-08-24):

* **latent** — the item is **wired into a live path**, it is **computed**, and no
  rung can see it being wrong. `#3493`'s write set. This defect can, the day a
  refusal widens, become a **wrong emit**.
* **orphaned** — the item is **not wired at all**: zero non-declaration
  references. An orphan **cannot** cause a wrong emit. It can only cause a
  **wrong belief** — and under goal (1), where the port is meant to be a
  readable model of `c2` and rungs are what the next lane cites, *a documented
  constant that governs nothing is a lie told to the next reader*. Not a lesser
  defect; a different one.
* **not-latent** — a rung goes red. *Name the rung and the test.*
* **fenced** — no rung goes red, and the code **says so itself** and pins it
  (`order.rs`' `the_layout_guard_is_inert_on_what_the_parser_admits`,
  `alloc.rs:1128`'s *"`allocate` does not read this field and a test pins that
  it does not"*). Honest, and a lane that reads it is not misled.
* **test-only** — a unit test goes red but **no byte can move**. The permuter
  still cannot use it. `FRAME_STWUX` is the predicted shape: its only reference
  is a test asserting its own bytes against its own literal.
Where a cell is **both** (orphaned *and* documented as governing something) the
report gives it both columns.

**Two questions every ORPHANED row must additionally answer** (coordinator,
2026-08-24 point 3): *is the VALUE corroborated by anything?* An orphan whose
value is also unverified is a strictly worse row than one whose value is right.
So each orphan row reports (a) whether any live computation independently
arrives at the same number, and (b) whether the byte judge — a fixture, a
captured prologue — confirms the doc's claim about `c2`.

**Compile-fenced** is a fifth outcome for a cell whose mutation will not build
(`MAX_FIELDS` is an array length). It is reported, not counted as green.

## 4. NON-NO-OP IS PROVEN BEFORE A GREEN COUNTS (`w-s1c2` §2.2, `w-3475`'s three control gaps)

**A green mutation is a claim about the control before it is a claim about the
code.** Three guards, all mechanical:

1. **T4.1 — the patch is verified applied.** The patcher counts the anchor,
   **aborts unless the count is exactly 1**, and prints `git diff --stat`. A
   vacuous patch fails loudly instead of reading GREEN. (`run_mutant.sh`'s rule,
   reused.)
2. **T4.2 — the tree is verified clean before AND after.** The runner refuses to
   start on a dirty `crates/`, and re-checks `git diff --quiet -- crates/`
   after the revert. **No mutated state is ever committed**, and the rung will
   say so with the check that proves it.
3. **T4.3 — the mutation is verified SEMANTICALLY different, not just textually.**
   For axis A this is the **reference count**: a const with `refs ≥ 1` outside
   its own declaration has at least one site whose computed value provably
   changes. **A const with ZERO references is a no-op by construction and this
   lane will say exactly that** rather than book it as a mutation result — it
   is `w-s1c2`'s SPR-9 case, and the finding there is the *reading*, not the
   run. Every axis-A row prints its reference count beside its verdict.

**T4.4 — DISCRIMINATING CELLS ARE COUNTED AND A ZERO IS A LOUD FAILURE (#3454
trap 0, stronger form).** For each axis-A cell the rung prints the number of
non-declaration reference sites. A row reading `refs=0` is reported as
*latent-by-enumeration* in a separate column and is **excluded from the mutation
denominator**, so "33 mutations, N green" can never be inflated by cells that
could not have gone red.

**T4.5 — the ladder is proven able to fail.** R1 must be shown to go red on a
mutation of the same shape and the same file. §7 records that it does.

## 5. PREDICTIONS — axis A, all 33 cells, registered before the campaign

Mutation value in the third column. `refs` is the total `\b<name>\b` count over
`crates/` **including** the declaration, as measured at base.

| # | const (file:line) | value → mutation | refs | PREDICTION |
|---|---|---|---|---|
| A01 | `cond.rs:115` `CR0` = 0 | → 3 | 21 | not-latent (R1) |
| A02 | `cond.rs:238` `BO_IGNORES_CR` = 0x10 | → 0x02 | 7 | not-latent (R1) |
| A03 | `cond.rs:249` `BO_SENSE_BIT` = 0x08 | → 0x04 | 2 | not-latent (R1) |
| A04 | `comdat.rs:446` `INLINE_DECLINE_LOOP_BYTES` = 80 | → 8 | 3 | not-latent (R1) |
| A05 | `comdat.rs:495` `INLINE_DECLINE_BYTES` = 128 | → 16 | 14 | not-latent (R1) |
| A06 | `splice.rs:154` `INLINE_UNBOUNDED_BYTES` = 64 | → 8 | 8 | not-latent (R1) |
| A07 | `osf_handle_guard.rs:135` `K_SCALE` = 4 | → 8 | 5 | not-latent (R1) |
| A08 | `alloc.rs:1130` `POOL_TOP` = 11 | → 9 | 30 | not-latent (R1) |
| A09 | `alloc.rs:1134` `MAX_MODELLED_PRODUCERS` = 3 | → 1 | 12 | not-latent (R1) |
| A10 | `xtea_round_loop.rs:77` `DELTA` = 0x9E3779B9 | → 0x12345678 | 9 | not-latent (R1) |
| A11 | `schedule.rs:103` `BLOCKED_STORE_POSITIONS` = 2 | → 0 | 3 | not-latent (R1) |
| A12 | `mop.rs:506` `MAX_FIELDS` = 5 | → 6 | 4 | **compile-fenced** (array length) |
| A13 | `frame.rs:34` `FRAME_HEAD` = 80 | → 800 | **1** | **ORPHANED** — zero non-declaration references; `FrameLayout::locals_base` computes 80 from `FRAME_MIN_OUT_SLOTS` instead, and `docs/rungs/2026-08-05-w-next.md:167` already quotes the constant as a fact |
| A14 | `frame.rs:37` `FRAME_MIN_OUT_SLOTS` = 8 | → 4 | 3 | not-latent (R1) |
| A15 | `frame.rs:43` `FRAME_MAX_SAVED_NO_SPILL` = 17 | → 2 | 4 | not-latent (R1) |
| A16 | `frame.rs:48` `FRAME_PAGE` = 4096 | → 256 | 4 | **latent or test-only** — the probes need a frame > one page |
| A17 | `frame.rs:70` `FRAME_STWUX` = 0x7C21616E | → 0 | 2 | **test-only** — its one reference is a test asserting it against itself |
| A18 | `label.rs:85` `LABEL_SEED_GAP` (derived) | see B04–B08 | 19 | not-latent (R1) via B |
| A19 | `order.rs:117` `HEAD_SLOTS_MAX` = 2 | → 1 | 8 | not-latent (R1) |
| A20 | `order.rs:121` `MAX_MODELLED_PRODUCERS` = 3 | → 1 | 12 | not-latent (R1) |
| A21 | `order.rs:128` `MAX_MULTISYM_PRODUCERS` = 2 | → 1 | 8 | not-latent (R1) |
| A22 | `order.rs:136` `MAX_SYMBOL_CROSSINGS` = 2 | → 1 | 16 | not-latent (R1) |
| A23 | `encode.rs:457` `BO_DNZ` = 16 | → 8 | 20 | not-latent (R1) |
| A24 | `encode.rs:1745` `CR_COMPARE` = 6 | → 5 | 161 | not-latent (R1) |
| A25 | `encode.rs:1748` `BO_TRUE` = 12 | → 13 | 83 | not-latent (R1) |
| A26 | `encode.rs:1750` `BO_FALSE` = 4 | → 5 | 107 | not-latent (R1) |
| A27 | `encode.rs:1752` `BO_ALWAYS` = 20 | → 21 | 24 | not-latent (R1) |
| A28 | `encode.rs:1755` `CR_BIT_LT` = 0 | → 3 | 37 | not-latent (R1) |
| A29 | `encode.rs:1756` `CR_BIT_GT` = 1 | → 3 | 15 | **uncertain** — `>` may not be in any admitted class |
| A30 | `encode.rs:1757` `CR_BIT_EQ` = 2 | → 3 | 93 | not-latent (R1) |
| A31 | `encode.rs:1772` `BC_MAX_DISP` = 32764 | → 64 | 28 | **uncertain** — a range check no admitted body reaches |
| A32 | `encode.rs:1805` `B_MAX_DISP` = 0x01FFFFFC | → 64 | 19 | **uncertain** — same shape, larger bound |
| A33 | `plan/mod.rs:347` `FN_FLAG_EMIT_SEED` = 0x20 | → 0x40 | 5 | **census-only** — `emit_set_members` ships `Unknown`; the seed is a characterization value |

**Registered aggregate prediction: 3 latent or test-only (A13, A16, A17), 1
compile-fenced (A12), 1 census-only (A33), 3 uncertain (A29, A31, A32), 25
not-latent.** Scored in the rung.

## 6. PREDICTIONS — axis B, the shipped decision points

| # | cell | mutation | PREDICTION |
|---|---|---|---|
| B01 | `EncodeParams::C2.rows` | point at a truncated table | not-latent (R1) |
| B02 | `EncodeParams::width_override` handling (`mop.rs:786`) | ignore the override | fenced — `mop.rs:966` is a test that exists to catch exactly this |
| B03 | `EncodeParams::drop_override` handling (`mop.rs:783`) | ignore the override | fenced — same test |
| B04 | `SeedGapModel::READ.base` = 7 | → 6 | not-latent (R1) |
| B05 | `SeedGapModel::READ.global_optimizer` = 2 | → 3 | not-latent (R1) |
| B06 | `SeedGapModel::READ.pooled_data_phase_string` = 1 | → 40 | **LATENT** — `PORT_ADMITTED` sets the input `false`, so the coefficient is multiplied by zero on every live path. This is exactly `w-permute`'s write-set shape: the mutated program **is** extensionally different (`gap()` at `pooled=true` returns 46 instead of 10), nothing reaches it |
| B07 | `SeedGapInputs::PORT_ADMITTED.global_optimizer` = true | → false | not-latent (R1) |
| B08 | `SeedGapInputs::PORT_ADMITTED.pooled_data_phase_string` = false | → true | **not-latent** — flipping the *input* on turns the latent coefficient live and moves the gap 9 → 10. **This is the pair that proves B06 is a latency finding and not a no-op**, and it is registered as such before it runs |
| B09 | `PlanInputs::function_level_linking` | invert at the one producer | not-latent (R1) |

**B06/B08 is the lane's designed control pair.** If B06 is green and B08 is red,
the coefficient is genuinely latent while the surface it sits on is live — the
strongest form of the finding. If **both** are green the surface is dead
wholesale and the report says that instead. If **both** are red, B06 was never
latent and the prediction is a miss.

## 7. DISCLOSED: one probe run before this file

`FRAME_MIN_OUT_SLOTS: u32 = 8` → `4` was applied, built, run at R1 and reverted
before this prereg was written, to price the mutation cycle (**4.6 s warm**,
which is what makes a 33-cell campaign affordable) and to satisfy **T4.5**. It
came back **RED: 27 failed of 627**. That is A14's cell and its result is
already registered above as `not-latent`; it is disclosed here rather than
presented as a campaign result. The tree was verified clean after the revert.

## 7.1 REPORT, DO NOT REMOVE — registered before anything is found

No item this lane names will be deleted, renamed or rewired. The fence says
STOP-AND-REPORT before modifying any existing `crates/` file, `CLAUDE.md` bans
reflex cleanup, and — the substantive reason — **an orphan with a nine-line doc
may encode a read somebody did once and nobody re-derived**, which is worth
*recovering*, not discarding. Each row names the exact edit that would fix it and
its (zero) call sites, and leaves the decision to a later lane.

The rung that cites a corrected fact is **amended beside, never edited**
(`#3495` / `w-r8idiom` convention): `docs/rungs/2026-08-05-w-next.md` stays as
written and the board row carries the correction and cites it.

## 8. WHAT WOULD MAKE THIS LANE FAIL

* **FAILED** if the campaign produces no verdict table — e.g. if the ladder
  cannot be shown to discriminate, or if R2/R3 cannot be run.
* A **vacuous negative** — every cell green — is a **loud failure**, not a pass:
  it would mean R1 is not an observable at all, and §7's red is the check
  against it.
* Finding **zero latent surfaces** is a real and reportable result: it says the
  33 constants of the byte-producing crate are genuinely fenced, and the next
  lane may stop asking.
