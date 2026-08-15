# w-three — PREREG

**Frozen and committed as this lane's FIRST commit, before any `crates/` change
and before any pricing probe.** Base master **`202bfc3f`**; the lane's numeric
base for every handed-down figure is master **`55933035`** and every figure below
carries the commit it was measured at, per the house rule `w-vocabgap` §0
established after nine handed-down figures were caught wrong in one wave.

The lane's question: **price `src/system/decomp_pch.cpp`,
`src/system/math/vec.cpp` and `src/system/os/NetworkSocket.cpp` — the only three
TUs in the 878-TU workload that are reader-clear and not already `match`
(#3184) and the only three carrying no emitted blocker key at all (#3191) — and
convert any that a CLOSED recognizer plus a SERIES licenses.**

---

## 0. What was already measured when this file was frozen — named, not hidden

**One measurement precedes this prereg and it is the lane's required base-end
control**, which has to be run at both ends regardless of what the lane decides:
the 878-TU workload scan at `202bfc3f` (`work/w-three/base.{log,jsonl,tsv,keys}`),
plus the three TUs' rows read out of its JSONL. Naming it is the doctrine
(`STATUS.md` trap 5). The predictions in §2 are **genuinely forward** — none of
the probes they score had been run when this file was committed.

Reproduced at `202bfc3f`, digit-for-digit against the figures the brief pins at
`55933035`:

| key | brief (at `55933035`) | measured here (at `202bfc3f`) |
|---|---:|---:|
| anchored `gap-metric` keys | **394** | **394** |
| `match` (population: **878-TU workload scan**) | 25 | **25** |
| `mismatch` | 0 | **0** |
| `codegen-gap` | 0 | **0** |
| `vocab-gap` | 845 | **845** |
| `capture-fail` / `graded` | 8 / 870 | **8 / 870** |
| `frontier` | 2 | **2** |
| `fnbyte-exact` / `-denominator` | 35,734 / 162,049 | **35,734 / 162,049** |
| `fnbyte-refused-parse` / `-codegen` | 113,612 / 949 | **113,612 / 949** |

**Zero handed-down figures disagree.** That is itself worth recording, because
this wave has caught nine.

The three TUs, from the same scan (`--factors-tsv` letters + the per-TU JSONL):

| TU | letters | `gate_cause` | `gate_causes` | `gl_body_starts` | `selective_bind` |
|---|---|---|---|---|---|
| `decomp_pch.cpp` | `-BCD-` | `gl-stop-26-introduced` | +`body-out-of-class` | 559 / 1242 | 0 named / 1242 · 872 · 206 |
| `vec.cpp` | `-BCD-` | `gl-stop-26-introduced` | +`body-out-of-class` | 373 / 811 | 0 named / 811 · 488 · 219 |
| `NetworkSocket.cpp` | `--C--` | `gl-stop-26-introduced` | +`body-out-of-class` | 120 / 240 | 0 named / 240 · 294 · 31 |

---

## 1. The register, per TU — decision and probability

**Ceiling with NO discount factor**, as the brief requires: each row states the
*most* a conversion of that TU could move each metric, not an expectation.

| TU | registered decision | p | ceiling if converted |
|---|---|---:|---|
| `src/system/decomp_pch.cpp` | **DECLINE** | **0.90** | workload `match` **+1** (25 → 26) |
| `src/system/math/vec.cpp` | **DECLINE** | **0.94** | workload `match` **+1** |
| `src/system/os/NetworkSocket.cpp` | **DECLINE** | **0.88** | workload `match` **+1** |
| **all three converted** | — | **≤ 0.02** | workload `match` **+3** (25 → 28) |

`decomp_pch.cpp` carries the lowest decline probability of the three because it
is the only one of them whose reference obj has **zero `.text` COMDATs**
(`emit-emitted 0`, measured §0) — i.e. the only one needing **no codegen at
all**, which is the shape factor **E**'s whole-TU recognizers already convert
three TUs through (`Main.cpp`, `TomCryptLicense.cpp`, `ZlibLicense.cpp`, #3185).

## 1.1 Registered metric deltas, **naming the population every time** (#3125)

`match` has three meanings. All three are registered separately.

| metric | population | base (at `202bfc3f`) | registered delta | p |
|---|---|---:|---|---:|
| `match` | **878-TU dc3 workload scan** | **25** | **+0** | **0.90** |
| fixture-gate verdicts | **`gate.sh`, 381×18 fixture gate** | **6,858** | **+0** | 0.95 |
| `mismatch` | every population, everywhere | **0** | **+0 — MUST be 0** | 0.99 |
| `fnbyte-exact` | 878-TU workload scan | **35,734** | **+0** | 0.92 |
| `cargo test --workspace --release` | portable lane | **1,643 passed / 42 targets** | **+0 / +0** | 0.85 |
| anchored `gap-metric` keys | the scan's own key space | **394** (not 370, not 372) | **+0** | 0.95 |

**A `mismatch` anywhere is an alarm that outranks every other result in this
document.** A wrong emit is strictly worse than a gap.

## 1.2 What licenses a conversion here, and what does not

Adopted from the brief and from the two waves of correction behind it:

* A **closed recognizer whose residual shapes are excluded BY CONSTRUCTION**,
  **plus a SERIES**. Not a fitted rule (`w-fenceb`'s R1′ failed its hold-out 5
  of 15, #3127) and **not a single obj reading** (`w-slots` read the charge out
  of one fixture's own obj, the objs read **3**, and the series is **`2n+1`** —
  shipping the cell would have been a wrong obj, #3147).
* **Reading one cell gives a number right for that cell and wrong as a rule.**
  Registered as a live risk on this lane specifically, because every quantity I
  have on these three TUs is a **single-TU cell**.

---

## 2. Genuinely forward predictions — the probes, and none had been run

| id | prediction | p |
|---|---|---:|
| **F1** | The three TUs' refusal is **ONE mechanism in triplicate**, not three: all three stop at `gl-stop-26-introduced` and no repair of it alone decodes any of them | **0.92** |
| **F2** | `NetworkSocket.cpp`'s "2 of 4 bodies already exact" (#3184, w-871 found-and-not-taken **#1**) is a **function count that inverts when re-read in bytes** — the exact fraction in bytes is **< 25 %** of its emitted `.text` | **0.80** |
| **F3** | `decomp_pch.cpp`'s reference obj has **0 `.text` COMDATs**, so it needs **no codegen**, and its terminal is therefore a **section/COMDAT emitter** question — i.e. **peer `w-section`'s**, not mine | **0.75** |
| **F4** | Each of the three needs **≥ 4 independent mechanisms**, counted as distinct refusals the port would have to pay in series | **0.85** |
| **F5** | At least one of the three has a residual that is **strictly outside this lane's permitted files** (`crates/c2-il`, `crates/c2-core/src/codegen/`) | **0.80** |
| **F6** | `c2rs census` on each of the three reports `0 < in-class < total` — i.e. the census is not the refusing party | 0.70 |
| **F7** | The three TUs' `emit_blockers = {}` (#3191, *"invisible to the instrument the entire widening order is built on"*) has a **mechanical cause**: the gate refuses the whole TU **upstream of** the emitted-function loop, so `{}` means *"never asked"*, not *"nothing blocks"* | **0.88** |
| **F8** | No `gap-metric` key other than the ones I deliberately move differs between my base and tip scans | 0.93 |
| **F9** | A **perfect factor A alone** converts none of the three (this is #2782 re-checked on a third TU it never covered, `NetworkSocket.cpp`) | 0.90 |

**Bias direction, registered in writing.** This lane is dispatched with *"this is
the closest thing to an available conversion anywhere on the board"*, so its
optimistic direction is **toward converting**. The repo's calibration
(`CEILING.md` §5: optimism dominates ~5:1, specifically on forward cost) says my
mechanism counts read as **lower bounds** and my decline probabilities read as
**too low**. The counter-risk is a **manufactured decline** — inflating a price
to justify not building — so §3 registers the anti-inflation check.

### 2.1 The anti-inflation check, registered before it runs

For **every pair of mechanisms** I count against a TU, I must ask *"what varies
between these two refusals?"* and collapse the pair if nothing does
(`w-871` §8's rule, which fired twice there and took numbers **out** of the
total). Registered expectation: **at least one collapse fires** (p = 0.65). If
none fires I will say so rather than omit it.

---

## 3. The mutants, with colours registered BEFORE any run

`w-bind16` had **3 of 4 come back green against registered red**, and read its
first RED off a stale `INDEX.md` fired by its own uncommitted doc — the
**flattering** direction — and discarded the run. Colours frozen here.

| # | mutant | registered colour |
|---|---|---|
| **M1** | a **comment-only** edit to the pricing probe | **GREEN** (byte-identical output) |
| **M2** | **THE POSITIVE CONTROL** — run the ladder on a `match` TU. It must report `decodes = true`, `gate_cause = None`, `causes = []` — i.e. **visibly different from the three** | **FIRES** (must differ; a probe that reports the same thing for a `match` TU as for a `vocab-gap` TU is measuring nothing) |
| **M3** | run the ladder on a **frontier** TU (`wordwrap.cpp` / `keygen_xbox.cpp`) — reader-blocked, `A∧B∧C` true | **FIRES** (a third, distinct profile) |
| **M4** | feed the probe a **truncated** scan stream (< 800 TU rows) | **REFUSE** — never "0 blockers" |
| **M5** | ask the probe for a **TU that does not exist** in the stream | **REFUSE** |
| **M6** | delete one `emit` key the probe depends on from every row | **REFUSE** |
| **M7** | **the `emit_blockers = {}` reading** — assert the three TUs' empty map is `never asked` and not `nothing blocks`, by exhibiting a TU with a **non-empty** map and the **same** class | **FIRES** (discriminating cells printed) |

**Absence is not success.** Every check is positive on content, and every count
of discriminating cells is printed beside the number it qualifies.

---

## 4. Advance declines — registered so they cannot be claimed later

| id | declined in advance | why |
|---|---|---|
| **N1** | **`crates/c2-core/src/coff/`** — any change, of any size | **OFF-LIMITS**: peer `w-section` is in it, single-occupancy, on the `.rdata`/`.data` COMDAT section emitter `w-bind16` (#3196) identified as the reachable head's terminal. If one of my three TUs terminates there I **stop and report** |
| **N2** | **`crates/c2-harness/src/gap/`** — peer `w-guards` owns it | not mine |
| **N3** | Any **narrowing, shadowing or redefinition** of a shared predicate — `LabelMap` has **13 production clients**; `codegen::labels` remains the **single reader** of a pending intra-section branch site | three semantic collisions here with no textual conflict |
| **N4** | Any **fitted** rule — a recognizer tuned to the residual shapes of one TU's own obj | `w-fenceb` R1′ (5 of 15 hold-out, #3127) and `w-slots` (`2n+1`, not `3n`, #3147) |
| **N5** | Any **ranking** of the three, and any dispatch off one | ρ ≈ +0.047, 0-for-4 (#3135); four for four on *"the ranking instrument measures itself"* |
| **N6** | Re-opening **codegen saturation** (`codegen-gap` 0 over all 878), **item F** (buys zero in all four populations, #3170) or **§6.2 completion** | measured, not a route |

---

## 5. Hygiene, registered

* Scratch in `work/w-three/`, never `/tmp`. **No `git add -f`** — `.gitignore`
  matches both `*.obj` and `/work`, so the `-f` that lands text evidence
  silences the artifact rule in the same stroke (#3156, 19 objs got in that
  way). Every file added with an **explicit pathspec**, never a directory.
* **Never glob or recursively walk** `work/capture-cache` or
  `.claude/worktrees` — two kernel OOM kills.
* Gates run as a **SINGLE WRITER in the foreground of one job**, and no
  `crates/` file is patched while a gate is in flight (#3075, #3117, #3128).
* Board rows drafted **UNNUMBERED**; the coordinator serializes. Next free is
  **#3200** with two peers in flight — **not minted by me**.
* `docs/rungs/INDEX.md` **regenerated** by `scripts/gen_rung_index.sh` on any
  conflict, never hand-merged.
