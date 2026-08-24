# PREREG — `w-permute`, S1c (i)'s LAST producer: the argument-marshalling unit

    Tag:       w-permute
    Date:      2026-08-24
    Kind:      construct
    Base:      0b00b6c14
    Rows:      #3491–#3495 (eighth-wave ledger, `docs/BOARD.md` end of file)
    Status:    REGISTERED BEFORE ANY `crates/` EDIT

**Nothing in `crates/` has moved when this file is committed.** Every figure in
§1 was read from the pristine base tree, read-only, before this file existed.
`gate.sh` content-hashes `crates fixtures scripts` and not `docs/`; the **armed
suite** reads `docs/rungs/`, so nothing under `docs/` moves while that is in
flight (`w-s1bc` §5.1, #3048). This file's name begins with `_`, so
`rung_registry.rs` and `gen_rung_index.sh` both skip it and `INDEX.md` does not
move.

---

## 0. What this lane is asked to complete

`w-s1c3` finished every producer `w-s1c2` §6.2 priced and **declined**
`permute_args_text`, re-pricing it by reading (`docs/rungs/2026-08-24-w-s1c3.md`
§6.1, board **#3471**). This lane takes that unit — the last producer surface of
Phase 0 slice **S1c (i)**.

The coordinator's brief states its citations are unverified and names
enumerations to perform. §1 performs them.

---

## 1. THE BRIEF'S COUNTS, VERIFIED BEFORE USE

Every function extent below is `fn` head to the matching column-0 `}`, computed
mechanically, not counted by eye.

| claim (`w-s1c3` §6.1 / board #3471) | verified at `0b00b6c14` | verdict |
|---|---|---|
| `permute_args_text` `calls.rs:1616`, 3 lines | `1616–1618` = **3** | **HOLDS** |
| `one_moved_formal_text` `calls.rs:1675`, 27 | `1675–1701` = **27** | **HOLDS** |
| `sym_slots_text` `calls.rs:1783`, 82 | `1783–1864` = **82** | **HOLDS** |
| `lit_insert_shift_text` `calls.rs:1886`, 49 | `1886–1934` = **49** | **HOLDS** |
| `lit_slots_text` `calls.rs:1936`, 37 | `1936–1972` = **37** | **HOLDS** |
| `permute_args_parts` `calls.rs:1981`, 144 | `1981–2124` = **144** | **HOLDS** |
| **total 342** | **342** | **HOLDS to the line** |
| the unit is **six** functions, not `w-s1c2` §6.2's four | six, and the two extra are the two named | **HOLDS** |
| `permute_args_parts` has a second production caller, `call_seq_parts` at `calls.rs:1360`, which splices its text | `calls.rs:1360` is `permute_args_parts(slots)?`, inside `call_seq_parts` (`1176–1564`) | **HOLDS** |
| no byte-position obstruction: the pair's second `Vec<u8>` is a register list | every `writes.push` in the six pushes a register (`1853`, `1929`, `1932`, `1969`, `2118`, `2122`); no byte offsets, no slicing, no back-patching in any of the six | **HOLDS** |
| `X_ops` producers = 19 at base | `git grep -hoE 'fn [a-z_0-9]+_ops\(' HEAD -- 'crates/c2-core/src/codegen/*' \| sort -u \| wc -l` = **19** | **HOLDS** |
| `mop_*` = 85/85 | `crates/c2-core/src/codegen/encode.rs` = **85**; `reach.rs` carries a 86th, `mop_in_form`, which is the form dispatcher and not an instruction twin | **HOLDS**, with the 86th named |

### 1.1 TWO CITATIONS THAT DO NOT RESOLVE AS WRITTEN — both minor, both recorded

Neither changes the price. They are recorded because #3471's own lesson is that
a figure can be right while the thing behind it is not.

1. **`w-s1c3` §6.1: *"all sharing one `Result<(Vec<u8>, Vec<u8>), BackendError>`
   shape"* — FOUR of the six share it, not six.**

   | function | actual return type |
   |---|---|
   | `permute_args_parts` | `Result<(Vec<u8>, Vec<u8>), BackendError>` |
   | `sym_slots_text` | `Result<(Vec<u8>, Vec<u8>), BackendError>` |
   | `lit_slots_text` | `Result<(Vec<u8>, Vec<u8>), BackendError>` |
   | `one_moved_formal_text` | `Result<(Vec<u8>, Vec<u8>), BackendError>` |
   | **`permute_args_text`** | **`Result<Vec<u8>, BackendError>`** — the pair's first element only |
   | **`lit_insert_shift_text`** | **`Result<Option<(Vec<u8>, Vec<u8>)>, BackendError>`** — `Ok(None)` means *"not this shape"* |

   The prereg that lane wrote says the same thing, so this is a compression
   introduced when §6.1 was written from §1.1, not a misreading. It matters
   here only because the conversion has to preserve **three** signatures, not
   one, and `Ok(None)` is a control-flow value the split must not flatten.

2. **`w-s1c3` §6.1 cites `calls.rs:1974` for `writes.push(ARG_REGS[dst])`.**
   `calls.rs:1974` is the **doc line** `/// [permute_args_text] plus **the
   registers its moves write**…`. The nearest `writes.push(ARG_REGS[dst])` is
   **`calls.rs:1929`**, in `lit_insert_shift_text`. That lane's own prereg cites
   1974 correctly, as *"1974's doc"*; the rung compressed the doc cite and the
   code cite into one parenthesis. **The claim is true; the line number points
   at the sentence, not the statement.**

### 1.2 THE SHARED MACHINERY THIS UNIT NEEDS — a stated NULL, verified

`w-s1c3`'s hardest piece was `reach::direct_op`: a gate returning `[u8; 4]` was
the single obstruction. **This unit needs no new machinery.** The six use
exactly three encoders and all three already have `mop_*` twins:

| encoder | twin |
|---|---|
| `encode_addi` (`encode.rs:280`) | `mop_addi` (`encode.rs:291`) |
| `encode_addis` (`encode.rs:297`) | `mop_addis` (`encode.rs:308`) |
| `encode_mr` (`encode.rs:1549`) | `mop_mr` (`encode.rs:1560`) |

No branch, no displacement, no relocation word is composed inside the six — the
`lis`/`addi` symbol halves are emitted with a **zero** immediate and patched by
the linker, and the caller registers REFHI/REFLO against the function's own
start (`lib.rs:1393`). Registered here so that "no machinery was needed" is a
read result and not a silence.

---

## 2. THE REGISTERED SUBSET — named BEFORE measuring

### 2.1 COMMITTED — this lane lands these or says why not

* **T1 — all six functions become op-stream producers**, in the `X_text` (thin
  `ops_to_bytes` wrapper) + `X_ops` (producer) shape of `w-s1c2` §1.3 and
  `w-s1c3` §1.1, with `fp_permute_args_text`/`fp_permute_args_ops`
  (`leaf/float.rs:500`) as the in-repo precedent for this exact function family.

* **T2 — the wrapper policy, decided in advance so #3428 cannot happen by
  drift.** A `_text` wrapper is kept **only where it has a caller**, and a
  wrapper whose only callers are tests is `#[cfg(test)]` from the commit that
  creates it — not from a later lane reading a build warning:

  | function | production caller after the split | policy |
  |---|---|---|
  | `permute_args_text` | `select.rs:476` | **keep**, thin wrapper, `pub` |
  | `permute_args_parts` | `call_seq_parts` (`calls.rs:1360`) | **keep**, thin wrapper — this is the "render at the splice" option §6.1 named |
  | `sym_slots_text` | none; **9 test call sites** (`calls.rs:3212, 3290, 3292, 3293, 3295, 3308, 3354, 3540, 3612`) | **keep as `#[cfg(test)]`**, with the reason on it, exactly as `formal_slots` (`calls.rs:1631`) already is in this file |
  | `lit_slots_text` | none, no test caller | **rename** to `_ops`, no wrapper |
  | `one_moved_formal_text` | none, no test caller | **rename** to `_ops`, no wrapper |
  | `lit_insert_shift_text` | none, no test caller | **rename** to `_ops`, no wrapper |

* **T3 — the class records go with their arms (#3469).** The six carry the
  largest transcribed-capture blocks in `calls.rs` — WLA/WLB
  (`one_moved_formal_text`), WR1 + W-ADJUST's eleven `.cod` cells
  (`sym_slots_text`), W-VSNPRNC (`lit_insert_shift_text`), and the cycle rule
  plus the length-4 grid (`permute_args_parts`). Every one of them describes
  **the arms below it**. Checklist item, once per converted producer: does the
  record still sit above the code it describes? Records about the **encoder** or
  the **word** are not moved (`w-s1c3` §8.1's exclusion).

* **T4 — required-zero, measured at both ends with the denominator printed**:
  `gate.sh --jobs 4 --require-graded` count table and the 878-TU `c2rs gap`
  scan's `gap-metric` keys, base and tip, `C2RS_COMPILERS`/`C2RS_WIBO` set
  explicitly on **both** arms (#3470).

* **T5 — at least five executed mutations**, each verified applied and
  non-no-op by going **RED**, tree restored green between (§7).

* **T6 — the COST CLAUSE axis** (#3336, §3), measured with the committed
  `scripts/cost_arms.py` at a **balanced** `--rounds`, with a null arm, and its
  refusal on an unbalanced `--rounds` **watched failing** before the real run is
  trusted (`CLAUDE.md`: watch a `--check` fail on deliberately broken input).

### 2.2 DECLINED IN ADVANCE — named so that not doing them is a decision

* **`fp_permute_args_ops` is already done** (`w-s1c3`) and is not touched.
* **`park_call_with_literals` / `seq_entry_park`** (`calls.rs:714`, `:559`) are
  the W-MMIO park, are *not* among the six, and are **not** converted here.
  They are `call_seq_parts`' own surface and pricing them is a different lane.
* **`call_seq_parts` itself** (389 lines) is **not** converted. This lane
  renders at its splice, which is the option #3471 explicitly left open.
* **S1c (ii)** — `Plain` through `block_ir` — not attempted, as for `w-s1c3`.
* **`ptr_walk_chain_loop.rs:465`'s `unused import: Mul`** — a standing warning
  older than `w-s1c2`, `w-s1c3` and this lane. **Two lanes have recorded it as
  not theirs and `w-s1c3` §6.4 item 3 said the third should just fix it.** This
  lane is the third. It is fixed, **in its own focused commit**, attributed to
  no conversion. Registered here so the fix is a decision and not collateral.

### 2.3 THE DECISION SURFACE — a stated NULL, and the points NAMED anyway

The DECISION-SURFACE CLAUSE (`docs/rungs/README.md`) requires a general layer to
ship its arbitrary choices as named parameters. **This lane exposes no new
parameter, for `w-s1c3` §1.4's reason**: a producer conversion is a change of
*representation*, the same decisions one rendering step later, and a knob added
here would be a fake surface. The real surface is S1a's `EncodeParams`.

**But this unit is the first converted producer whose decisions are the
clause's own examples** — allocation order and scheduling tie-breaks — so they
are named here for the lane that does expose them, rather than left implicit:

1. **The cycle-break scratch register.** `SCRATCH_REG` (r11) in
   `permute_args_parts`; c2 uses r10 as the *second* one in the shape this
   class refuses (`calls.rs:2085–2100`'s length-4 grid note).
2. **The cycle-walk direction.** "Lowest destination filled from the temp
   **last**", walking backwards from `lowest` — the unique clobber-free order
   *given* the choice of which destination the temp serves.
3. **The literal walk's direction.** `lit_slots_text` and `sym_slots_text` both
   walk **descending destination**; a chain link's walk is the opposite
   (`one_moved_formal_text`'s doc).
4. **The address's position in the walk.** `sym_slots_text` puts the `addi`
   after **exactly one** word of the walk — W-ADJUST, and the rule this
   replaced (*"the `addi` goes LAST"*) was a live wrong-bytes emit.
5. **The hoist predicate.** `one_moved_formal_text`'s `src == ARG_REGS[1]`.

All five are *measured* rules with cited captures, not fitted constants, so
none of them owes a read pointer under
`whitebox/READ_PLAN_2026-08-21.md` §2. What they are is exactly the set a
permuter would search, and #3471's re-pricing is what made them reachable.

---

## 3. THE FAILURE AXIS — named before the lane starts (COST CLAUSE, #3336)

A required-zero byte delta is silent about everything that is not a byte. Two
axes on which this rung **can** fail with every byte identical:

* **Port throughput.** `scripts/cost_arms.py`, three arms including a
  byte-identical null, balanced rotation, per-fixture minimum over rounds,
  paired, sign split published beside the mean. §4.
* **The write set.** The six return *two* values and only the first is bytes.
  The second — the registers the moves write — is read by `call_seq_parts`'
  callee-saved interleaving (`c2_il`'s `plan_saved_gprs`). A conversion that
  rendered the bytes correctly and dropped a `writes.push` would move bytes only
  in the Class B shapes that consult it. **This is named as an axis because the
  identity diff reaches it only through whichever gate lanes exercise Class B**,
  and a lane that did not name it would not know whether it had been tested.
  M-W (§7) is the mutation that proves this axis live.

---

## 4. THE COST PROTOCOL — registered, including the refusal test

* `scripts/cost_arms.py --arm base=… --arm nulldup=… --arm tip=… --rounds 9`.
* **Before the real run**, `--rounds 8` over three arms is invoked and the
  script is required to **REFUSE**. If it does not refuse, the refusal is not a
  control and #3468's finding is not being enforced — that is reported as a
  defect in the committed instrument, not worked around.
* The null arm's own reading is published as a **per-run certificate**, never as
  a constant of the hardware (#3468).
* **Both the mean and the sign split** are reported for every arm.

---

## 5. THE STOP CONDITION

Decision 5 as sharpened by #3426 and #3470: both binaries **pinned**, run
**back to back against one corpus**, `C2RS_COMPILERS` and `C2RS_WIBO` set
explicitly on both arms, and per arm the log asserted to carry no
`SKIP: toolchain absent`, exit 0, and a **nonzero key count**.

The workload stamp is `../dc3-decomp`'s HEAD at 12 chars. Read at base:
**`a29f559d0790`** — the value `w-s1c3` used, unmoved. It is read before and
after each arm and a move voids the pair.

**One addition to the protocol, registered here.** The stamp is the HEAD
**only**. `../dc3-decomp` is presently **DIRTY** (`src/system/obj/DataFile.cpp`
modified, tracked), and a dirty tracked file can change content without moving
the stamp by one bit. `GitInfo::probe` already computes the dirty flag
(`provenance.rs:150`) and `dirty_label()` prints it, but **no rung has ever
quoted it beside the stamp**. This lane records `git status --porcelain -uno` of
the workload before and after each arm as well as the HEAD. It is cheap and it
is strictly stronger than the stamp.

---

## 6. CONTROLS THIS PROJECT DOES NOT HAVE — restated, plus what is new here

Carried from `w-s1c3` §8.3 because they are still true, not because restating
them is free:

1. A producer conversion is **required-zero by construction** —
   `ops_to_bytes` is concatenated encoder output (`mop.rs::ops_tests`) — so a
   green gate is weak evidence about these conversions specifically. What it
   *can* catch is an op dropped or reordered.
2. `mod incumbent` proves **preservation**, never correctness.
3. `fnbyte-refused-parse` is ~70 % of the function population, so no identity
   diff sees a defect in a shape the parser refuses.
4. `hatch-red` is `REFUSED` at both ends (#1406).
5. A required-zero **byte** delta is silent about everything that is not a byte.
6. **`cargo`'s dead-code warning is a control this project reads late.** It
   fired twice in `w-s1c3` (§1.2, §8.2). §2.1's T2 policy is an attempt to make
   it fire **zero** times here by deciding the wrapper policy up front rather
   than discovering it from a warning.
7. **NEW, and it is the one this lane adds: a two-valued producer's SECOND value
   has no byte instrument at all.** §3.

---

## 7. THE MUTATIONS — registered before the conversion, so a substitution is visible

Each is applied to the real tree, verified applied by `git diff`, run, restored,
and the tree re-verified green between. A mutation that comes back **green** is
not evidence; only **RED** proves the function changed (`w-s1c2` §2.2's SPR-9
no-op). If a registered one turns out to be a genuine no-op it is **replaced and
the replacement recorded**, never reported as a control gap.

| # | mutation | the defect it models |
|---|---|---|
| **M1** | `permute_args_ops`' cycle walk emits `mr reg(lowest), SCRATCH` **first** instead of last | the cycle break undone — the one rule the whole class exists to encode |
| **M2** | `sym_slots_ops` emits the address `addi` **after the whole walk** | exactly the W-ADJUST defect that was a live wrong-bytes emit on mainline before `wadjust` |
| **M3** | `lit_slots_ops` walks the literal slots **ascending** | a schedule reversal no byte-*count* check can see |
| **M4** | `lit_insert_shift_ops` emits its `li` **before** the moves | the same, at the W-VSNPRNC arm, on the `Ok(None)`-carrying signature |
| **M5** | `one_moved_formal_ops` **inverts** the hoist predicate `src == ARG_REGS[1]` | a two-cell rule read backwards, where both cells encode as legal instructions |
| **M-W** | `sym_slots_ops` drops one `writes.push` while emitting the identical bytes | §3's second axis — a **write-set-only** defect, invisible to any byte compare of this producer's own output |

---

## 8. PREDICTIONS — registered, scored in the rung

| # | prediction | scoring rule |
|---|---|---|
| **P1** | identity diff **0 lines** over the count-bearing gate rows and over all `gap-metric` keys, with the denominator printed at both ends | HIT only if both denominators are nonzero and equal at the two ends |
| **P2** | the stop condition does not fire; `fnbyte-exact` reads **35,904** at stamp `a29f559d0790` | MISS if the figure differs at an unmoved stamp — which would refute #3426 from the other side |
| **P3** | the tip's cost lands **outside** the null's CI and **below +1.0 %** | MISS if outside that band, or if the null's sign split falls outside 45–55 % (in which case the run cannot answer and a second is required) |
| **P4** | at least one **#3469-class documentation-placement defect** is found among the six — a transcribed-capture record stranded on a wrapper | MISS if the T3 checklist finds none |
| **P5** | **no test assertion is changed by this lane.** Every test in `calls.rs` asserts through `permute_args_text` or `sym_slots_text`, both of which keep their byte signature under T2 | MISS if any `assert*` in `mod tests` has to be edited to keep the suite green |
| **P6** | `X_ops` producers go **19 → 25** | MISS at any other number; six new producers, no wrapper counted twice |
| **P7** | `cargo` emits **zero new** dead-code warnings at the tip, and the one pre-existing warning (`ptr_walk_chain_loop.rs:465`) is gone because §2.2 fixes it | MISS if any new warning appears |

---

## 9. WHAT WOULD MAKE THIS LANE SAY `FAILED`

* Any byte moves and is not traced to a defect in the conversion and fixed.
* The gate does not reach a graded verdict at either end (a `SKIP` arm is a
  void run, not a pass — #3470).
* The six do not all convert and the remainder is not priced by reading.

`declined` is **not** an acceptable outcome for this item here: #3471 already
spent a lane's read budget on the price, and the price came back `~1 session,
VERIFIED`. A second decline would be the ranking measuring itself.
