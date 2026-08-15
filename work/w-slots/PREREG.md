# w-slots — PREREG

**Frozen and committed BEFORE the first `crates/` change and before the first
`cl.exe` of this lane.** Base: master `ba23e8c3`, branch `wt-w-slots`.

Brief: `w-fenceb` (`docs/rungs/2026-08-14-fenceb.md` §6 item 2) lifted one arm of
Fence B (board **#746**, `IlFunction::label_slots -> None`) and left the
remaining arms as "a one-fixture question with a scripted method
(`work/w-fenceb/mutate.sh`) — **except `counted_accum_loop`, which must NOT be
taken**".

---

## 0. The arms, and the exclusion

`crates/c2-il/src/func/mod.rs::label_slots` has **four** `None` arms after
`w-fenceb`:

| arm | line | brief's status |
|---|---|---|
| `ptr_walk_chain_loop` | 4344 | ARM B, open |
| `counted_accum_loop` | 4404 | **FORBIDDEN** |
| `float_walk_loop` | 4419 | ARM A, open |
| `pool_ctor_chain` | 4443 | ARM C, open |

**`counted_accum_loop` is NOT TAKEN and is not probed.** The brief's reason is
that its charge moves `+7 -> +8` between `/O1` and `/Ox` **on a class that
accepts both modes**, so the unreachability argument that licensed `w-fenceb` is
absent. This lane has already confirmed the premise textually and will state it
as a read-only confirmation, not as a probe:
`crates/c2-il/src/func/body/shapes/counted_accum_loop.rs:233-235` admits
`Some(OptWordMode::O1) | Some(OptWordMode::Ox)`. **Both modes.** No mutation, no
grid, no lift. If any measurement in this lane appears to license it, this lane
STOPS and reports rather than proceeds.

---

## 1. The test each arm must pass — `w-fenceb`'s, restated

`w-fenceb`'s lift was **not** licensed by a rule (`R1'` scored 5 of 15 held out
and the missing term is the loop KIND, which no backward-branch feature vector
contains). It was licensed by three properties of one class. Each arm below is
scored against exactly those three:

1. **CLOSED RECOGNIZER** — every residual shape the grid3 hold-out exposed
   (`while` vs `for`, `break`, `continue`, named `goto`) is excluded **by
   construction**, not by argument.
2. **UNREACHABLE OUTSIDE THE MEASURED MODE** — because `label_slots` has no mode
   parameter and `wb-label` §7.6 forbids giving it one, a mode-dependent charge
   is only safe if the arm cannot be reached at the other modes.
3. **CHARGE READ OFF AN OBJ, NOT FITTED** — the charge comes out of a tracked
   fixture's own reference obj, and a mutation of it reddens against real
   `c2.dll` while a separating control stays green.

**If any answer is "no", the arm is DECLINED and the decline is priced.** A
priced decline is a good outcome (`w-pool`->`w-pool2`, `w-xtea`->`xtea2`->`xtea3`,
`w-backedge`->`w-fenceb`).

---

## 2. Per-arm registration

### ARM A — `float_walk_loop`. **EXPECT TO LIFT. P = 0.55**

Read-only findings that put it first (all textual, pre-`cl.exe`):

* **Property 2 is STRONGER here than in `w-fenceb`.** The `/O1` mode gate is in
  the **READER**, `crates/c2-il/src/func/body/shapes/float_walk_loop.rs:263`
  (`fwalk-opt-mode`), *before any body byte*. So `self.float_walk_loop.is_some()`
  **implies** `/O1` inside the same crate as `label_slots`. `w-fenceb`'s
  `ptr_walk_loop` relied on a **cross-crate** coupling
  (`codegen::ptr_walk_loop::select_function` refusing `mode != O1`) which it had
  to pin with a differential test. Here there is no cross-crate coupling to pin.
* **Property 1**: the recognizer requires a mandatory guard `if (n == 0) return;`
  and one `for` loop over an unsigned counter, 3-or-4 distinct formals, one body
  statement. **No `break`, no `continue`, no `goto`, no `while`, no `do/while`.**
* **The held-out fixture EXISTS and was built to be a must-fail cell.**
  `fixtures/cpp/wblockir_float_walk_then_framed_neg.cpp` puts the **loop first**
  and the framed `z9` second, precisely so a wrong charge is a live
  `Port=Mismatch` rather than a no-op (its own header records that the first
  spelling had the order the other way and "was a cell that could not fail").
  Separating control: `fixtures/cpp/wblockir_float_walk.cpp` (same loops, no
  framed function -> mints no labels -> must stay `match` under every mutant,
  board #742).

**The one thing that is NOT yet known, and it is the falsifier**: the class has
degrees of freedom `w-fenceb`'s did not — **3 shapes** (`Compound`, `Scalar`,
`Binary`) x **2 ops** (`Add`, `Mul`). `label_slots` returns one number for the
whole class, so the charge must be **invariant across them**.

* **A1 (P = 0.65)** — the lead is the **same integer** for all four bodies in
  `wblockir_float_walk.cpp` (`Add_InPlace` A/+=, `MulConstant_InPlace` B/*=,
  `Mul_InPlace` A/*=, `Mul` C/binary), each measured as `[loop, z9 framed]` in
  `w-json` counterfactual form at `/O1` against real `c2.dll`.
  **If A1 MISSES, ARM A IS DECLINED** — a per-shape charge is a rule fitted to
  which shape, and `label_slots` has no shape parameter either.
* **A2 (P = 0.50)** — that integer is **2**, i.e. the same lead `ptr_walk_loop`
  takes. Registered so a different number is a recorded surprise and not a
  silent adjustment. **A2 missing does NOT decline the arm** — the obj is the
  judge and the number is read, not predicted.
* **A3 (P = 0.85)** — with the measured lead installed,
  `wblockir_float_walk_then_framed_neg.cpp` goes `NotImplemented`/`vocab-gap`
  -> `match` at `/O1`, and `wblockir_float_walk.cpp` stays `match`.
* **A4 (P = 0.90)** — **at least 3 mutants go red** on the converted fixture
  (lead 0, lead+1, lead-1 or lead+2), with the separating control **green under
  every one**. A mutant that reddens both is measuring something else.
* **A5 (P = 0.75)** — `docs/LABEL_COUNTER.md` has no correct published row for
  this class, or has one that disagrees with the obj. **If a published number
  disagrees with the obj, the obj wins and this lane says so** (`w-fenceb`
  settled §4.2.1's `+3` as a live wrong emit exactly this way, #3126).

### ARM B — `ptr_walk_chain_loop`. **EXPECT TO DECLINE. P(decline) = 0.80**

* **B1 (P = 0.85) — the price of keeping this arm is ZERO tracked fixtures.**
  The arm's own doc comment (`mod.rs:4345`) cites a MUST-FAIL MUTATION against
  `fixtures/cpp/wvl_chain_then_framed.cpp` — and **that file does not exist and
  has never existed in this repo's history** (`git log --all --diff-filter=A`
  finds 0 adds). No tracked fixture pairs this class with a framed function, so
  the `None` holds nothing out. **A shipped must-fail claim that no cell can
  grade is a finding in its own right**, and it is reported whether or not the
  arm is taken.
* **B2 (P = 0.70) — property 1 fails on the loop KIND.** The class is spelled
  **`while (*s) { ... s++; }`** (`ptr_walk_chain_loop.rs` module doc, and
  `fixtures/cpp/wvl_chain3.cpp`), where the lifted `ptr_walk_loop` is spelled
  **`for (u = ...; *u; u++)`** (`whash_ptr_walk_loop.cpp:28`). `w-fenceb` §2.3's
  five witnesses say a `while` charges **more** than the `for` of the same shape
  and the kind is a term no feature vector holds. **So `ptr_walk_loop`'s 2 does
  NOT transfer**, and any number here must be measured on a fixture that does
  not exist.
* **B3 (P = 0.60) — property 3 has an extra unknown `w-fenceb` did not face**:
  the body is of **unbounded length** (`MAX_CHAIN = 10`), so the charge would
  have to be shown invariant in the chain length `M` as well. Reading it off one
  obj does not close it.
* **B4 (P = 0.25)** — this lane builds `wvl_chain_then_framed.cpp` anyway to
  settle B1/B2/B3 against the oracle. **Registered as OPTIONAL and contingent on
  ARM A finishing first.** Creating a fixture in order to convert it is close to
  circular; it is worth doing only to make the stale must-fail claim gradeable.

### ARM C — `pool_ctor_chain`. **EXPECT TO DECLINE. P(decline) = 0.85**

* **C1 (P = 0.85) — price of keeping is ZERO tracked fixtures AND zero workload
  TUs.** No `_then_framed` cell exists for this class, and `Pool.obj` carries
  **zero** `$M`/`$T` symbols in its whole 20-symbol table (all three functions
  are leaves), so the counter never reaches the target TU's obj. The arm already
  records this.
* **C2 (P = 0.75) — property 1 fails on the loop kind, and worse than ARM B.**
  The class is spelled **`do { ... } while (--n);`**, which is the exact member
  of `w-loop`'s four-charge confound: `do/while` +1, `for(;;)`+`break` +3, a
  backward `goto` +1, **all three emitting the identical 24 bytes**. The class
  additionally carries a **source-level `if (count > 1)` guard** that is read,
  not synthesized. Two control structures, one of them the confounded kind.
* **C3 (P = 0.90)** — property 2 HOLDS (reader-level `/O1` gate at
  `pool_ctor_chain.rs:221`), so the decline is on properties 1 and 3, not on
  mode. Registered so the decline names the right reason.

### ARM D — `counted_accum_loop`. **NOT TAKEN. P(take) = 0.00**

* **D1 (P = 0.95)** — the exclusion's premise reproduces in the source: the
  reader admits **both** `/O1` and `/Ox`. Confirmed read-only, no probe.

---

## 3. Registered outcome numbers — **CEILING, NO DISCOUNT**, population named

The ceiling is ARM A converting and ARMs B/C/D declining.

| quantity | **population (the denominator)** | base | **registered ceiling** |
|---|---|---|---|
| `mismatch` | every gate lane, sweep, cross, and the 878-TU scan | **0** | **0. NON-NEGOTIABLE.** A wrong emit is strictly worse than a gap |
| fixture-gate `match` | **381 fixtures x 18 mode lanes** (`gate.sh`) | per-lane, quoted from this lane's own base run | **+1 on the six `/O1`-family lanes ONLY** (`O1`, `O1-EHsc`, `O1-Oi`, `O1-Oi-EHsc`, `O1-Oi-GR`, `O1-Oi-EHsc-GR`); **+0 on the other twelve** |
| `c2rs perf` `Match` | **381 fixtures at the `/Ox` DEFAULT profile** | **150** | **+0.** This class refuses at `/Ox` in the reader. Registering a `/Ox` number for an `/O1`-only class was `w-fenceb`'s own estimation error and this lane will not repeat it |
| **878-TU workload `match`** | **878 dc3 TUs** (`c2rs gap`) | **25** | **+0 -> 25.** `wblockir_float_walk_then_framed_neg.cpp` is a **FIXTURE, not a workload TU**. The workload's own `IPP_basicmath_xbox.cpp` holds **no framed function**, so this gate never refused it and lifting the gate cannot admit it. **This lane will not claim a 26th TU match.** |
| `fnbyte-exact` | 878-TU scan | **35734** | **+0** |
| census | `c2rs census` | unchanged | **+0** — no new function class is admitted; a TU-level GATE moves and the census does not see it |
| workspace tests | `cargo test --workspace --release --no-fail-fast` | **1602 passed / 42 targets** | **<= 1608 / 42.** Target count quoted because a dropped target means an earlier target failed |
| `board_audit.sh` | — | all-zero | all-zero |
| `rung_registry` | — | 2/2 | 2/2 |
| `graded tree` | `crates fixtures scripts` | master **`e89e9b9be058`, 730 files** | quoted at **both ends** of the gate run |

**THREE FIGURES IN THIS REPO ARE CALLED `match`** (board **#3125**) and they move
independently. Every number this lane reports names its population in the same
breath. Three population conflations in three waves were each caught by a lane
checking rather than spending.

---

## 4. Falsifiers, registered in advance

Registered so that a null is loud rather than silent.

* **F1 — the shape-invariance falsifier.** If the four `wblockir_float_walk.cpp`
  bodies do not all force the **same** lead, ARM A is declined on the spot and
  the per-shape spread is published. This is the analogue of `w-fenceb` §5.3's
  `while`/`for` collision and it is the single most likely way ARM A dies.
* **F2 — the separating-control falsifier.** If `wblockir_float_walk.cpp` ever
  goes red under a mutant, the mutant is measuring the emitter and not the
  charge, and no red counts as evidence for the charge.
* **F3 — the mode falsifier.** If the same source's framed `$M` is measured at
  `/O1` and at `/Ox` and the reader is nonetheless found to admit the class at
  `/Ox`, ARM A is declined for `counted_accum_loop`'s reason.
* **F4 — the second-layer falsifier.** `IlBundle::functions`' gate is
  `label_slots(false)? != label_lead() + 1`, and `coff::plan_labels` advances
  exactly **1** for a non-framed function. So a bare `Some(k)` for the measured
  `k` produces a **refusal**, not a match. The lift must therefore be
  `label_lead() += K` with the `None` arm deleted, exactly as `w-fenceb` did, and
  **zero files under `crates/c2-core/src/coff/` may be opened.** If the
  conversion requires touching `coff/`, ARM A is declined as out of this lane's
  ownership.
* **F5 — the peer-collision falsifier.** `label_slots` has more than one
  consumer (`func/diag.rs:456`, `func/bundle.rs:2087`, plus shape tests). This
  lane **does not narrow, shadow or redefine** the predicate; it deletes one
  refusal arm and adds one term to `label_lead`. If a change would require
  altering the predicate's meaning for another consumer, it is not made.

---

## 5. Ownership

Owned by this lane: `crates/c2-il`'s `label_slots` surface (`func/mod.rs`) and
`crates/c2-il/src/func/body/shapes/`. `crates/c2-core/src/codegen/labels.rs` may
be touched only if unavoidable and only via the coordinator. Peer `w-layout` owns
`crates/c2-core/src/codegen/` **except** `labels.rs`; peer `w-loo` is
docs/`work/`-only. **`crates/c2-core/src/coff/` is OFF LIMITS** — `w-fenceb`
completed its lift without opening it (F4).

Scratch lives in `work/w-slots/`, never `/tmp` for anything load-bearing. No bulk
scan output is `git add -f`'d.
