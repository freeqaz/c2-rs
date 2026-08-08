# PREREG — lane `w-disagree`

Frozen **before the first line of code**, at base `04727f37` (master, tip of the
`w-midrun` merge). Nothing below is edited after a cell compiles; outcomes go in
the rung doc's scorecard beside the registration.

The rung is a **correctness defect in an instrument**, not a widening: board
**#1283**'s residue of four `census/gate DISAGREEMENT` cells, and the reason the
standing test could not see the 42 they came from.

---

## 0. Prior art — what was searched, and what it found

`grep -ril 'census/gate' docs/ scripts/ crates/` (34 hits) **and separately** a
row scan of `docs/BOARD.md` for the *topic* rather than the phrasing
(`discriminating`, `vacuous`, `population it ran over`, `absence read as
success`), because a board row does not contain the words a topic grep uses.

| prior art | what it already does | why this lane is not it |
|---|---|---|
| `crates/c2-harness/tests/census_gate.rs` | the standing test. Two linkage lanes, per-cause pinning, `in_class_total > 0` | **its population is the 286 fixtures** — the instrument this lane repairs |
| `scripts/sweep_mode.sh` | runs the **generated corpus** through `c2rs gap` at two `/O1` profiles and ratchets `census/gate DISAGREEMENT` against `C2RS_SWEEP_MODE_MAX_DISAGREE=3` | **not a row of `scripts/gate.sh`** — nothing runs it. Its baseline `3` was measured 2026-08-04 and five days of widening have landed since. It has a vacuity guard on *graded* and **none on the disagreement's own population** |
| `scripts/sweep_shapes.py --check` | the exact POSITIVE shape this brief asks for, one level up: per-marker case counts over the generated corpus, `--check` fails on a zero row | markers are a *source-text* proxy. It cannot see whether the census and the port disagreed about a case, which is this lane's quantity. **It is the model to copy, and #1140 is the caution: `pure virtual` read 166 over a population of 14 real ones** |
| board **#299** / **#1077** | absence read as success in `gate.sh`; closed by `--require-graded`, a positive demand | a statement about the *gate driver*, not about the agreement check's population |
| board **#1236** | a guard that has never been seen to fire does not work (`w-self2b` broke #1135 with the script written to prevent it) | why §4 of this prereg exists at all |
| board **#1140** (`w-gen`) | a green control green over a population it is not measuring, in `sweep_shapes.py`'s own rows | the closest match on the board. Same disease, different instrument |

**No open board row already owns this work.** #1283 is the residue and names no
instrument; #1275 is the finding. `#1304`–`#1313` are this lane's numbers.

---

## 1. What the four survivors are

`work/w-midrun/grid/{t_dl,t_dc,x_dl,x_dc}`, reproduced at base at the workload's
own `/GR /O1 /Oi /EHsc` before this file was written — all four read
`1/1 functions in class` and `census/gate DISAGREEMENT: 1`, with the **same**
emitter refusal:

> `a store run with an interior address BESIDE another producer: beside a
> literal that is the mixed-kind run codegen::alloc refuses (boards
> #836/#868/#1134); beside a second address the allocator answers but nothing
> has measured it`

They are **two different families under one message**:

* `t_dl` / `t_dc` — **twop**: two *distinct* interior addresses, `&mBlk` and
  `&mAlt`. Single-kind. `alloc::allocate` answers it; nothing has measured it.
* `x_dl` / `x_dc` — **mix**: one interior address beside a literal. This is the
  **mixed-kind allocation rule** — boards #836, #868, #1134, #1265 — which is
  peer lane `w-lineage`'s key (`alloc.rs`, the roots carrier,
  `STORE_RUN_BIND_MIXED_KIND`).

The over-claim is in the **direct** spelling only. `bind_run_ops` already
refuses both families in the bind spelling
(`STORE_RUN_BIND_MIXED_KIND` / `STORE_RUN_BIND_ADDR_PRODUCER`); the direct
spelling's run acceptance in `crates/c2-il/src/func/body/shapes/leaf_store.rs`
admits an `AddrOf`-valued group **unconditionally**, on a comment that says
`c2_core` will decline it because it is a four-op group — an argument
`w-midrun` retired when it taught `parse_simple_gpr_run` to read exactly that
group.

---

## 2. PREDICTIONS — the residue

**PRED-R1 — 2 of 4 close, 2 are named and left open.** `t_dl` and `t_dc` close
by a **reader** refusal in the direct spelling, mirroring the emitter clause
`w-midrun` already shipped in `scheduled_gpr_run`. `x_dl` and `x_dc` do **not**
close: refusing them puts a fresh reader refusal directly in front of the family
`w-lineage` is widening, and a reader refusal makes the peer's emitter rule
**unreachable** — board **#1291**'s shape exactly (*"the published cause is a
refusal that can never fire"*). Naming them is the outcome, not the fallback.

**PRED-R2 — closing the twop pair costs 0 on the workload.** `census/gate
disagreement` on the 878-TU scan is 0 at both ends and TU match stays **10**.
Registered as a *loss* direction: if `match` moves at all, the refusal is wider
than the emitter's and the lane is wrong.

**PRED-R3 — the direct-spelling refusal must not narrow the in-domain class.**
`w-midrun`'s GRID M is 76 in-domain cells byte-exact. All 76 carry **one**
interior address and no literal, so a clause keyed on *"two or more distinct
addresses"* cannot touch them. If any GRID M `dom` cell moves, revert.

---

## 3. PREDICTIONS — the repaired instrument

The population becomes the **generated sweep corpus** (`scripts/sweep_gen.py`,
**19,556** cases at base) in addition to the 286 fixtures, and the check prints a
**discriminating-cell count** — in-class functions on which the port's gate ran
to its own verdict, i.e. the cells in which a disagreement *can* appear — broken
down by census shape key, with a floor on the total and on the number of
distinct keys.

**PRED-I1 — the fixture lane's discriminating count is the interesting small
number.** Registered: **between 400 and 1,200**, and its distinct-shape-key
count **≤ 30**. Direction of error: I expect to be **high** on the key count.

**PRED-I2 — the wide lane is at least 10× the fixture lane in discriminating
cells.** Registered: **≥ 6,000** discriminating cells and **≥ 25** distinct
shape keys over the 19,556 cases. If it is not at least 10×, the widening is a
cost with no evidence behind it and this lane says so.

**PRED-I3 — THE ONE I WANT TO BE WRONG ABOUT. The wide corpus surfaces at least
one disagreement family the fixture corpus cannot contain.** Concretely: the
packed lane reads **3** on the wide corpus (`sweep_mode.sh`'s carried baseline,
the `70-framed-03{49,50,51}` trio, which that script records as reproducing at
`/Ox` too) where the fixture lane reads **1**. Direction of error: **UNDER**.
That baseline was taken 2026-08-04 at `/O1`, before nine merges, and the packed
lane here is `/Ox`. **Registering "0 new disagreements" would be the wrong
prediction to make here and is not made.**

**PRED-I4 — the `/Gy` lane is much larger than the packed lane on the wide
corpus, and almost all of it is one shipped refusal.** `function_gate`'s pooled
floating-point constant clause refuses every FP body carrying a constant under
`/Gy`, and `scripts/sweep.d/3*-fp-*.py` is over a thousand cases. Registered:
**≥ 150**, dominated by `pooled floating-point constant`. That is not a finding
— it is a standing refusal meeting a wide corpus for the first time — and the
lane must say so rather than bank it.

**PRED-I5 — `mismatch` stays 0.** The instrument grades no bytes; it compares
two verdicts. If any wider run this lane makes surfaces a live **wrong emit**,
that outranks the whole rung and is reported immediately.

---

## 4. THE MUTATION — registered before it is run

A guard nobody has seen fire is not a guard (board **#1236**). Three mutations,
each chosen to **hold every earlier assertion's quantity fixed** so the assertion
under test is driven directly — the lane-registry trap where a count floor fired
first and the `/EH` and `/Oi` assertions never executed (GAPS §7).

| # | mutation | must fire | quantities held fixed |
|---|---|---|---|
| M1 | add one refusal to `codegen::function_gate` for a shape the census accepts in quantity | the **disagreement** assertion | captured, in-class, discriminating, shape-key count all unchanged — `function_gate` still runs on every cell |
| M2 | make the corpus for one population empty | the **population-is-empty** assertion | — (this is the first guard; it must fire *before* the count floor, and with its own message) |
| M3 | make the port refuse before `select_function` is reached for every cell | the **discriminating-count** floor | captured unchanged; the disagreement count is *also* wrong, so M3 is the case that proves the ORDER is right — the discriminating floor must be the message, not the disagreement |

**Registered requirement: three distinct failure messages.** If two mutations
produce the same message, the assertions are not separable and the instrument is
one check wearing three names.

---

## 5. DECLINE FLOOR

* **Runtime.** If the wide lane costs more than **10 minutes** wall at this
  box's parallelism inside `cargo test --workspace --release`, it ships as a
  **strided sample with the stride printed in the assertion message** and the
  full corpus behind an env knob. If even a strided sample exceeds **30
  minutes**, the wide lane is declined and the rung ships the fixture lane's
  discriminating counts alone, saying so.
* **Fencing.** If closing `t_dl`/`t_dc` requires touching `alloc.rs`, the roots
  carrier, `STORE_RUN_BIND_MIXED_KIND`, or `control_flow.rs`, the residue is
  **declined and named**. `crates/c2-core/src/codegen/coff.rs` is never opened.
* **Erasure.** If the direct-spelling refusal changes the *meaning* of any
  shared predicate rather than adding a clause beside it, stop and enumerate
  every reader first (the `FnByte::Exact` hazard, `w-relo`).
* **Alarm.** A live wrong emit outranks everything here. Stop and report.

---

## 6. Shared surfaces, named in advance

Touched or read: `leaf_store.rs`'s direct-spelling run acceptance,
`FnVerdict::key`, `codegen::function_gate`, `codegen::select_function`,
`Toolchain::capture_il`, `IlBundle::census_functions`, `scripts/sweep_gen.py`.

**Widened, never narrowed.** The new census key is a *new name*; no existing key
changes meaning. `STORE_RUN_BIND_MIXED_KIND`, `STORE_RUN_BIND_ADDR_PRODUCER`,
`alloc::Root`, `alloc::ProducerRoots`, `Producer`, `ProducerKind` are **not
touched**. `work/w-splice/peerkeys.py` is run at both ends and any vanished key
family is reported.
