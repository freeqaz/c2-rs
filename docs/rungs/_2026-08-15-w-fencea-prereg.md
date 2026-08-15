# w-fencea — PREREG, frozen before the first `crates/` change

    Lane:      w-fencea
    Kind:      construct rung
    Base:      master `1f85d14c`, branch `wt-w-fencea`
    Frozen:    this file is committed before any file under `crates/` is touched

**The question.** Board **#3144**: `LabelMap` invariant 4 — board #746's **fence
A** — was measured at *"0 conversions and 0 bytes"* by `#3089`, was correct, and
was **superseded by construction** when `w-layout` took the map's production
clients from 2 to 9. Seven of the eight residual branch sites are now behind it.
`w-slots` (**#3148**) settled the attribution: fence A is the **backward-
reference refusal in `c2-core`**, fence B is `label_slots -> None` in `c2-il`,
and lifting B discharges **0** of the 8. **This lane prices and, if licensed,
lifts fence A.**

---

## 0. The fence, cited by its ENFORCING LINE and not by its module

`docs/rungs/2026-08-15-layout.md` §7 `P19` records its own miss: *"a prereg row
naming a fence must cite the enforcing line, not the module it lives in"*. So:

* **Fence A's enforcing line** is `crates/c2-core/src/codegen/labels.rs`
  `LabelMap::resolve`, the `if target <= r.at { return Err(...) }` arm — the
  method's **invariant 4**.
* **Fence A's stated rule**, module header § "The two rules this map enforces":
  *"Every reference must be FORWARD — a backward reference is refused, and the
  refusal is a `coff/` fact, not a `codegen/` preference."*
* **Fence B's enforcing line** is `crates/c2-il/src/func/mod.rs`
  `IlFunction::label_slots`, its `if self.ptr_walk_chain_loop.is_some() { return
  None }` / `counted_accum_loop` / `pool_ctor_chain` arms. **Not this lane's.**

`w-ir-g` (#3114) and `w-item-d` (#3119) each found an item whose **sentence
misled where its title bound**. The reading that this lane registers **in
advance**, as `H1`, is that fence A splits the same way:

> **`H1` (p 0.80).** The stated rule quantifies over **bodies that emit a
> backward intra-section branch**. The enforcing line quantifies over
> **references routed through `LabelMap`**. Those are different populations, and
> four shipped byte-exact lowerings live in the gap: `ptr_walk_loop`,
> `ptr_walk_chain_loop`, `json_utf8_copy` and `xtea_encrypt_loop` each emit a
> backward `bc` through `reach::direct` / `encode_bc` and the map never sees it.
> Therefore the enforcing line does **not** enforce the stated rule, and the
> thing that actually protects the `$M` counter is `IlFunction::label_slots` /
> `label_lead` at the TU level — which `labels.rs`' own **reason 3** already
> says: *"That is where a loop rung's relaxation belongs; it is not here."*

`H1` is a claim about code and is settled by reading two files. It is registered
because if it is **false** the lane declines every arm.

## 1. What licenses a lift, restated as this lane's bar

From `#3127` (a fitted rule dies on its hold-out, 5 of 15, because the loop
**kind** is a term no backward-branch feature vector holds) and `#3147` (one
cell's obj reads a number that is right for that cell and wrong as a rule; the
**series** is the separator):

1. **A closed recognizer whose residual shapes are excluded BY CONSTRUCTION.**
2. **A series over the structural count**, not a single obj reading.
3. Anything that cannot meet both is **declined and priced**.

**No fitted loop-kind rule is used anywhere in this lane.** `kindrule.py`'s
23-of-23 is disqualified in its own docstring and is not consulted.

## 2. The construction this lane will attempt (arm A1)

Invariant 4 becomes a **per-map admission fixed at construction**, defaulting to
the refusal. The admission is not a flag and not a class name: it is a value
computed from **c2-il's own three-valued counter gate**, so the residual is
excluded by construction rather than by review.

```text
  BackEdge::of(f: &IlFunction) ->
     RefusedByTheTuGate     label_slots(false) == None
                            => IlBundle::functions refuses EVERY TU in which
                               this body's $M could be observed (board #742,
                               Q2: 34 of 34 leaf-only TUs mint zero labels)
     ChargedByPlanLabels    label_slots(false) == Some(label_lead() + 1)
                            and label_lead() >= 1
                            => coff::plan_labels ALREADY advances the surcharge
     Refused                everything else, including label_lead() == 0
                            => the wrong-$M case invariant 4 was built for
```

`LabelMap::new()` is unchanged and is `Refused`; **all nine of item A's existing
clients stay exactly as they are**. No second fixup list, no `Form` variant, no
second copy of the rule in `block_ir`.

## 3. The arms, each registered as lift-or-decline with a probability

| arm | what | registered |
|---|---|---:|
| **A1** | the fence itself, as §2 | **lift**, p 0.70 |
| **A2** | `ptr_walk_loop` — 2 sites — onto `BodyLayout` | **lift**, p 0.65 |
| **A3** | `xtea_encrypt_loop` — 1 site | **lift**, p 0.50 |
| **A4** | `ptr_walk_chain_loop` — 2 sites | **decline**, p 0.60 (variable-length chain; its `label_slots` is `None`, so it is admitted by §2's second arm, but the migration is a re-shape of a 638-line variable-arity emitter) |
| **A5** | `json_utf8_copy` — 2 sites | **decline**, p 0.65 (76 words, ten branch sites, a Class C prologue relocation and three published offsets) |
| **A6** | `pool_ctor_chain` — 1 site | **decline, NOT PRICED HERE** — its fence is the absent CTR terminator (`#3146`), not fence A. Named so the count is honest. |

**Sites discharged, registered: 3 of 8** (stretch 5 of 8; floor 0 of 8 if `H1`
is false).

## 4. The series (bar item 2), and the population it is over

The structural count is **the number of admitted-class loop functions in one
TU**, `n = 0, 1, 2, 3`, each TU closed by one framed function whose `$M` is the
readout. Instrument: `work/w-slots/lead.py`'s seed-cancelling form (**#3148**),
which subtracts each TU's **own** `.gl` counter so the seed cancels inside the
TU. The class is `ptr_walk_loop`, whose lead `w-fenceb` registered at **2** from
**one** cell.

> **`S1` (p 0.70).** The framed `$M` advances **`2n`** — the shipped
> `+ 2 * u32::from(self.ptr_walk_loop.is_some())` term is per-function-additive.
> A reading of `2n + c` for any `c != 0`, or any non-linear series, means the
> shipped charge is a **live wrong emit at `n >= 2`** and this lane reports it
> rather than shipping on top of it.

`#3147`'s lesson applied literally: the parameter `w-fenceb` did not vary is `n`,
and this lane varies it. **`S1` is NON-DECLINING for arms A2/A3** — the obj is
the judge and the number is read, not predicted — but a miss is a finding that
must be reported and routed, not adjusted to.

## 5. The metrics, with the POPULATION NAMED every time (#3125)

`match` names three quantities in this repo and they move independently.

| # | metric | population | registered |
|---|---|---|---|
| P1 | `mismatch` | 878-TU dc3 workload scan | **0**, absolutely. An alarm that outranks all work |
| P2 | `match` | **878-TU dc3 workload scan** | **25 → 25, +0.** No fixture is a workload TU and this lane claims no workload conversion |
| P3 | `match` | **381×18 fixture gate**, per lane | **+0 on all 18 lanes.** `O1` 178, `O1-EHsc` 179, `O1-Oi` 180, `O1-Oi-EHsc` 181, `O1-Oi-GR` 180, `O1-Oi-EHsc-GR` 181, `/Ox` 150, `/O2` 156, `/Od` 18 |
| P4 | `Match` | **`c2rs perf`'s `/Ox` default profile** | **150 → 150, +0** |
| P5 | `fnbyte-exact` | 878-TU scan `gap-metric` | **35,734 → 35,734, +0** |
| P6 | `gap-metric` keys | 878-TU scan | **372** keys identical digit for digit |
| P7 | per-TU verdicts | 878-TU scan | **878** lines identical, sorted |
| P8 | census | `c2rs census` | **+0** — no fixture claimed, no prefix taken |
| P9 | sweep / cross | gate | sweep **19,556 / 19,460 / 0 mismatch**; cross **90,812 / 90,424 / 0 mismatch** |
| P10 | `graded tree` | gate | identical at **both ends** of each run; base predicted `04e3500f07b7` (**730** files) |

**This is a construct rung: the success criterion is a REQUIRED-ZERO BYTE
DELTA.** A conversion would be the failure signal, not the prize. `P2`/`P3`/`P4`
are registered at **+0** and a *fall* in any of them is a result to report and
revert, never to rationalize.

## 6. Test count — CEILING WITH NO DISCOUNT

| # | registered |
|---|---|
| P11 | base **1,610 passed / 42 targets**; tip **1,610 + N**, `N` ≤ **34**, no discount factor. 0 failed, 42 targets unchanged |
| P12 | `git grep -c '#\[test\]' -- 'crates/*'` delta equals the runner's delta; the runner stays **10** behind at both ends (#3076) |

## 7. Mutants — the bar is `w-layout`'s eight / `w-item-d`'s 34 / `w-slots`' four

| # | registered |
|---|---|
| P13 | **≥ 6 mutants**, each watched go red and reverted, **≥ 3 of them reddening a REAL-OBJ oracle** (a fixture verdict against real `c2.dll`, not a unit test), with a **separating control green under each** |
| P14 | **M-A**: `BackEdge::of` returning the admission unconditionally must NOT redden anything by itself — registered as a **known-negative**, because it is the honest statement that the admission's safety is about *future* classes and not about today's bytes. If it *does* redden something the lane has found more than it registered |
| P15 | **M-B**: the migrated back-edge displacement off by one word must redden a real-obj oracle |

## 8. Hard constraints this lane binds itself to

* **`coff/` is off-limits.** Zero files under `crates/c2-core/src/coff/`.
* **`docs/LABEL_COUNTER.md` is peer `w-labeltable`'s.** Zero edits; anything
  found wrong there is **reported**, not corrected.
* **`crates/c2-il`'s `counted_accum_loop` reader is peer `w-counted`'s.**
* **`codegen::labels` stays the single reader of a pending intra-section branch
  site.** No second fixup list; `block_ir`'s
  `a_backward_branch_is_refused_by_the_label_maps_own_rule` must keep reading
  `labels.rs`' own words — it will be **re-pointed at the default map**, which
  is still `Refused`, and it must still fire.
* **No shared predicate narrowed, shadowed or redefined.** `LabelMap` has nine
  clients; the default path is byte-for-byte the one they have today.
* Board rows land **UNNUMBERED**; the coordinator serializes `#3151`+.

## 9. What this lane will NOT do

* **Item F is not mine and is not re-priced** (`CFG_SHAPE.md` §6.2, 6 of 7).
* No relaxation pass, no long-branch expansion, no `Terminator::Bdnz`, no
  `BlockOrder` second variant, no code motion, no scheduler.
* No fitted loop-kind rule, no `R1'`, no re-reading of `LABEL_COUNTER.md` as
  fact.
