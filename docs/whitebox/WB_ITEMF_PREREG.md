# WB_ITEMF — PREREG (lane `w-itemf-price`)

> Frozen and committed **before the first grep of the export** and before the
> first measurement of anything in this repository. Scored in
> [`WB_ITEMF_FINDINGS.md`](WB_ITEMF_FINDINGS.md) §7.

**Kind:** characterization lane (`rungs/README.md` § "Lane kinds", kind 3).
**Outcome word will be** `built` or `FAILED`, in that word.

**Image pinned:** `compilers/X360/16.00.11886.00/c2.dll`,
`sha256 c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`,
**verified at the top of this lane** (`C2_MAP_METHOD.md` §0).

**The commissioned question.** `CFG_SHAPE.md` §6.2 item **F** is the last of the
seven unbuilt (6 of 7 built: A, B, C, D, E, G). Its floor was established by
`w-merger4` (#3111) after three upward re-pricings (#3067, #3099, #3109).
**Price it. Do not build it.** No `crates/` change; `git diff master..HEAD --
crates fixtures scripts` must be empty.

Deliverables, fixed here so they cannot be renegotiated after the fact:

1. a decomposition of item F into steps a construct rung could take, each with
   its own **fail-closed boundary**, in §14.2's style;
2. a **price per step as a ceiling with NO discount factor**, counting
   *independent* refusals, not refusal *sites*;
3. **what item F buys, in each named population** — the 381×18 fixture gate,
   `c2rs perf`'s `/Ox` gate, the 878-TU workload scan — named every time
   (#3125);
4. **what cannot be priced**, named as such.

---

## 0. The bias direction, stated in writing

**My expected bias is UPWARD.** Every prior on this seam pushes the same way:
phase 7 has been re-priced upward three times in three days (#3067, #3099,
#3109); five of six times a discount factor was applied on this project it was
the error; and this lane's brief tells me to report a larger price if I find
one, which is a licence to inflate.

The specific failure mode that bias produces is **counting one variable read at
several thresholds as several refusals** — the error a previous ceiling made,
and the error `w-item-d` avoided when it found the range check spelled
**twenty-four** times over **one** fact. So the counter-check is registered as
a scored prediction (**P2.3**) rather than a promise: I predict that at least
one pair of steps I separate on the first pass **collapses to one** under the
question *"what varies between these two refusals?"*. If nothing collapses, that
is evidence my decomposition is inflated, and §7 will say so.

**Registered headline numbers, before any measurement:**

| register | value |
|---|---|
| **step count** | **7** |
| **total ceiling for item F complete** | **≥ 12 lanes**, of which **≥ 8** are step F0 |
| **what it buys on the 878-TU scan** | **0 conversions** |

---

## 1. P0 — method and the decline floor

| # | prediction | p | scored |
|---|---|---|---|
| P0.1 | The sha256 above matches the on-disk image, and every VA this lane cites resolves to a function in `~/ghidra-projects/export/c2/functions.tsv`. | 0.90 | — |
| P0.2 | **DECLINE FLOOR.** If this lane cannot produce a decomposition in which **≥ 3 steps carry a stated fail-closed boundary** *and* **≥ 1 step is priced against a count measured in this repository** (not quoted from a doc), it reports **FAILED** in that word and publishes no price. | — | — |
| P0.3 | No claim in the findings rests on an absence without being **labelled** as absence-grounded in the sentence that makes it. `w-dagclients` labelled three of its own; #1823 banked three and was refuted. | 0.85 | — |

## 2. P1 — the reading rule: title vs enforcing line

Item F's **title** is *"Values live across block boundaries — the real cost."*
Its **enforcing lines** are three named cells (`MemFree`'s `v2` r4→r11 entry
copy; `d_join`'s `b` in r31 across a call; `?b_if2`/`?b_ifn`'s formals in
r31/r30 across calls, framed for that reason alone) plus the paragraph that the
incumbent register model "has no notion of a value being live at a program
point".

Three for three, the prose and the code have quantified over different sets
(#3114 item G, #3119 item D, #3151 fence A). Registered predictions:

| # | prediction | p | scored |
|---|---|---|---|
| P1.1 | The title and the enforcing cells quantify over **different sets, and neither contains the other**. The set the *mechanism* ranges over is *"a candidate live at a clobber point in the lowered instruction order"* — which contains no block. | 0.70 | — |
| P1.2 | **At least one** of item F's three named cells needs **no block boundary at all** — a single-block body reproduces it — and `wb-live`'s own grid already contains such a cell. | 0.65 | — |
| P1.3 | The complement holds too: a value live **across a block boundary with no intervening clobber** costs the port nothing and is already emitted correctly by the incumbent positional model. **The title's set is where the price is NOT.** | 0.55 | — |
| P1.4 | Item F read through its cells **requires a pass `CFG_SHAPE.md` §6.3's first bullet forbids** — *"No code motion. §3.4.1's hoist and tail-merge are recorded as a limit on the accepted class … not as a pass to build"* — and #3099's block merger **is** §3.4.1's tail-merge/hoist. §6.2 and §6.3 are in **contradiction on item F**, and nothing in the repo records it. | 0.60 | — |
| P1.5 | The phrase **"the real cost"** in the title is **false as a claim about where the cost is**: the dominant cost of item F is its *prerequisite* (the lowered order), which is not item F and appears nowhere in §6.2's seven items. | 0.60 | — |

## 3. P2 — the decomposition

| # | prediction | p | scored |
|---|---|---|---|
| P2.1 | The step count lands at **7 ± 2**. Registered: **7**. | 0.60 | — |
| P2.2 | **≥ 3 steps are buildable today** (items A–E and G exist; `BodyLayout` has nine clients; fence A is lifted) and **≥ 2 need something that does not exist**. | 0.55 | — |
| P2.3 | **The anti-inflation check.** At least one pair of steps separated on the first pass **collapses to one** under *"what varies between these two refusals?"*. | 0.50 | — |
| P2.4 | The single most expensive step is **F0, the lowered instruction order**, and it prices **≥ the sum of every other step**. | 0.65 | — |
| P2.5 | At least one step is **not orderable** — it cannot be placed before or after its neighbour on evidence, because the evidence that would order it does not exist. Named as such rather than given an arbitrary position. | 0.40 | — |

## 4. P3 — the price

| # | prediction | p | scored |
|---|---|---|---|
| P3.1 | The total ceiling for item F **complete and byte-exact on arbitrary bodies** is **≥ 12 lanes**, **≥ 8** of them step F0. Ceiling, **no discount factor applied**. | 0.50 | — |
| P3.2 | The count of **independent** refusals in `crates/c2-core/src/codegen/` that item F would lift is **< 10**, and **strictly less than the count of refusal *sites***. (Item D's shape: 24 spellings of one fact.) | 0.60 | — |
| P3.3 | The number of shipped byte-exact lowerings that **hard-code at least one physical register number** is **≥ 10**. That set is the required-zero re-expression base a construct rung for item F must clear, and it is the part of the price nobody has counted. | 0.55 | — |
| P3.4 | **At least one** step's price is dominated not by the mechanism but by the **re-expression base** — i.e. the cost is in the classes that already ship, not in the new code. | 0.45 | — |

## 5. P4 — what item F buys, per named population (#3125)

| # | prediction | p | scored |
|---|---|---|---|
| P4.1 | **878-TU workload scan: 0 conversions**, from item F complete. `codegen-gap` is **0** over all 878 and `vocab-gap` is **845** — nothing reaches codegen, so a codegen item cannot move `match`. | 0.85 | — |
| P4.2 | **381×18 fixture gate: 0.** A construct rung for item F is **required-zero by definition** (board #290's pattern), so the buy on this population is zero *by the grading rule*, not by weakness. | 0.80 | — |
| P4.3 | **`c2rs perf`'s `/Ox` gate: 0.** Perf times an already-byte-exact obj; a register model changes no timing population. | 0.80 | — |
| P4.4 | There is **at least one positive buy that is not a conversion**: a currently-shipped class whose register constants item F would make **derived** instead of transcribed. If it exists it is named; if it does not, that is reported as **zero buy on every population**, which the brief says is the most useful outcome. | 0.40 | — |
| P4.5 | The **frontier** (16 TUs, `cfg-reach-shipped` 2 of `cfg-reach-top` 16) is **not** moved by item F either — the frontier's block is CFG class and reader, not registers. | 0.70 | — |

## 6. P5 — what cannot be priced

| # | prediction | p | scored |
|---|---|---|---|
| P5.1 | **≥ 3** distinct unpriceable things are named, drawn from: the grey-zone fifth merger `0x10b3ab86` → `0x10b394f5` (entered 2/2, never reaching its inner call on 13 cells); the globregs candidate-minting policy at `0x10b55732` (uncharacterized — `WB_LIVE_FINDINGS.md` §10); the availability forward fixpoint (no cell isolates it); the cost function (measured **inert** on all 25 cells — it can be priced neither up nor down). | 0.75 | — |
| P5.2 | At least one thing this lane would otherwise have priced **rests on an absence** and is labelled rather than banked. | 0.60 | — |
| P5.3 | The scheduler's **cost term** is **not** re-priced by this lane in either direction, and `wb-live`'s four not-blocking items (#3057 — interference graph, cost function, spiller, callee-saved policy) are **not** re-priced. | 0.90 | — |

## 7. P6 — landing and the gate

| # | prediction | p | scored |
|---|---|---|---|
| P6.1 | Docs-only, therefore the 878-TU scan is **digit-identical**: `match 25 · mismatch 0 · codegen-gap 0 · vocab-gap 845 · capture-fail 8`, **370 keys**. | 0.90 | — |
| P6.2 | `gate.sh --jobs 4 --require-graded` prints `graded tree e6d4bfb38066`, **730 files**, at **both** ends; `board_audit.sh` all-zero; `rung_registry` 2/2; `cargo test --workspace --release --no-fail-fast` **1,619 passed / 42 targets**. | 0.85 | — |
| P6.3 | `git diff master..HEAD -- crates fixtures scripts` is **empty**. No `DISCLOSURE.md` row, because nothing is adopted. | 0.95 | — |

---

## 8. What this lane will NOT do

Named in advance so absence never reads as coverage.

* **It will not build any part of item F.** No `crates/` change of any kind.
* **It will not dispatch or recommend off a ranking.** `w-loo` (#3135–#3140)
  measured five of six published rankings as carrying no information
  (ρ ≈ +0.047), that a ladder never scores what it starts with, and that
  leave-one-out zeros do not compose. A "found and not taken" list is a list,
  and it will be published **with its own uninformativeness stated**.
* **It will not open the Ghidra project.** Disassembly only by grep over the
  flat export at `~/ghidra-projects/export/c2/`.
* **It will not re-price phase 7's floor.** `w-merger4` established it and this
  lane takes it as given; if this lane's reading implies a *different* floor,
  that is reported as a disagreement in a dated box, not as a silent revision.
