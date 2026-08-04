# Pre-registration — lane `w-afail`, why does factor A fail on 843 of 871 TUs?

Committed **before** any measurement in this lane. This is a **census, not a
fix**: the deliverable is a bucketed answer to *"for the TUs where factor A is
false, why?"*, and the failure mode to guard against is the one this project has
now paid for four times (board #150) — **producing a histogram that reads as a
work queue when bucket size does not predict TU yield.**

## The question

`docs/ROADMAP.md` §10.19/§10.21 factor a byte-exact obj as
**`A ∧ B ∧ C ∧ (D ∨ E)`**. `A` (`emit-set-ceiling-gate`,
`crates/c2-harness/src/gap.rs:1958`) is the predicate

> the bundle's `.ex` segment count on the **gate** splitter (`4F 1F`,
> `IlBundle::ex_segment_count`) **equals** the reference obj's `.text*` COMDAT
> leader count (`emit-emitted`).

A is the binding constraint of the five and **has never been censused**. There
is a detailed per-function blocked-shape census; there is nothing equivalent for
the emit *set*. Every codegen lane this session worked inside the 25 TUs where A
already holds, so the total remaining capacity of all current work is 17 TUs
against a ceiling of 871.

## Provenance, recorded now

| | |
|---|---|
| c2-rs branch | `wt-w-afail`, based on `master` = `d72432c` |
| dc3-decomp HEAD **before** the run | `940d07dcb0960964ad61aa5f025658f993eb46b2` |
| dc3-decomp HEAD **after** the run | recorded in the findings; **if it moved, that is a finding, not a footnote** |
| workload | `work/dc3-workload/files.txt` (878), `flags.txt` (`/O1 /Oi /EHsc /GR …`) |

The corpus moved 40+ commits inside one 30-minute window earlier today and shifted
an emitted-census denominator by +75 with no code change. **Every incumbent below
is re-measured at my own HEAD before it is used**, including the factor counts.

## What I looked at before writing this

Disclosed so the predictions are not secretly post-hoc:

* `gap.rs` steps 1e/1g/1h — how A, B, C, D, E are computed. A is a **count
  equality**, so its failure is a signed integer per TU, and the whole census is
  a decomposition of that integer.
* `docs/STATUS.md`'s generated block: per-function census denominator
  **2,463,393** bodies, emitted-function census denominator **178,975**, over 878
  TUs. That is ~2,806 IL bodies against ~204 emitted COMDATs per TU — a **13.8×**
  ratio — which is the basis of P1 and is disclosed as such.
* A 6-TU smoke run to confirm the toolchain resolves from the worktree. No
  factor counts were read from it.

## The predicate, frozen

For each **graded** TU (`class != capture-fail`) I record:

| symbol | source |
|---|---|
| `n` | `emit["emit-gate-segments"]` — gate-anchored `.ex` segments |
| `c` | `emit["emit-emitted"]` — `.text*` COMDAT leaders |
| `t` | `fn_total` — LO-anchored census rows |

`A ⇔ n == c`. The decomposition is the identity

```
n − c  =  (n − t)                    …  splitter
        + (t − rows_bound)           …  rows the binding did not join to an emitted COMDAT
        − (c − rows_bound)           …  emitted COMDATs no census row claims
```

and the **new, additive** instrument keys (gap.rs step 1e, names only, no
existing count touched) partition `t` exactly:

* `afail-row-emitted` — row has an `emit_name` that IS a `.text` COMDAT leader
* `afail-row-not-emitted` (+ `|<mangling_class>`) — row has an `emit_name` that
  is **not** emitted: a body c2 was handed and **discarded**
* `afail-row-unnamed` — the `.gl` binding gave the row no name at all:
  **instrument-limited**, not a compiler fact

and the emitted side is already partitioned by the existing
`emit-residue-generated` / `emit-residue-unbound|<class>` /
`emit-unbound-has-record` / `emit-unbound-no-record|<class>` keys.

**Bucket key for an A-failing TU** = `(dir, dominant)` where `dir ∈ {surplus
(n>c), deficit (n<c)}` and `dominant` is the largest single contributor to
`|n−c|` among `{splitter, rows-not-emitted, rows-unnamed, comdats-generated,
comdats-unbound}`. At most **10** buckets by construction.

**A count is only evidence about the predicate that produced it.** If I change
this key mid-run, every number before the change is discarded and re-measured,
and the report says so.

## Known-answer controls

> **C0 (accounting).** `afail-row-emitted + afail-row-not-emitted +
> afail-row-unnamed == fn_total` on **871 / 871** graded TUs — **exact, zero
> breaks**. If C0 fails, nothing else in this lane is interpretable and every
> number below is reported as "not measurable".

> **C1 (A reproduced off-instrument).** The count of TUs with `n == c` computed
> in Python from the JSONL equals the `emit-set-ceiling-gate` total the Rust
> prints. **Exact.** This is the guard against the failure that has bitten three
> instruments today: two probes agreeing because a flag was silently dropped.

> **C2 (incumbents).** Re-measured at my HEAD, the factorization block prints
> **A 28 / B 338 / C 114 / D 8 / E 2 / A∧B∧C 25 / graded 871 / match 8 /
> capture-fail 7**. Registered as a *prediction about the corpus*, not as an
> assumption: if any of these moves, the movement is the first reported finding
> and every downstream number is quoted at the new value.

## The predictions

### P1 — one direction, or two?

> **P1.** Of the A-failing graded TUs, **≥ 95 %** have `n > c` — the IL carries
> more function bodies than c2 emits.
>
> *Rival P1′:* the split is materially two-sided (> 5 % with `n < c`). Then
> A-failure is **two** mechanisms, and the deficit side — c2 emitting a COMDAT
> with no `.ex` segment behind it, i.e. a symbol the port must **synthesize** —
> is the harder of the two and gets its own ranking.

### P2 — how many buckets, and how concentrated?

> **P2.** **≤ 4** buckets cover **≥ 90 %** of the A-failing TUs, and the **top
> bucket exceeds 500 TUs**.
>
> *Rival P2′:* no bucket exceeds 200 TUs and the tail is long. That answer —
> "A fails for many distinct reasons and there is no lever" — is delivered as
> plainly as a positive one; it is the result that would most change the
> roadmap.

### P3 — is the top bucket a compiler fact or an instrument limit?

> **P3.** The dominant bucket is **`surplus / rows-not-emitted`** — bodies the
> front end produced and c2 chose not to emit — covering **≥ 400 of the ~843**
> A-failing TUs. On that answer, A's failure is **one understood-but-unmodelled
> mechanism** (Phase 7's emit-set selection) at massive scale.
>
> *Rival P3′:* **`rows-unnamed` dominates.** Then the census is bounded by the
> `.gl` binding's name coverage, not by the compiler: we would not be measuring
> why c2 declined to emit a body, only that our instrument cannot name it. This
> rival is live — `App.cpp` reports `fn_names 3752` against `fn_total 9033`.

### P4 — the TU-weighted ranking, which is the point

> **P4.** For **every** bucket X taken alone, "if X were closed" (its
> contribution to `n−c` removed, all others held) converts **< 60** TUs to
> satisfying A. No single bucket is a lever.
>
> **P4b.** The number of A-failing TUs whose **entire** delta comes from a
> single mechanism is **≤ 150** of ~843.
>
> **P4c.** Closing the **top three** buckets jointly moves `A∧B∧C` from **25** to
> **< 120**.
>
> *Rival P4′:* some single bucket alone converts **≥ 100** TUs to A. If P4′ wins
> this lane has found the first real lever on the payoff metric since the
> `??__E` recognizer, and the report leads with it.

### P5 — are A and C one project or two?

> **P5.** `|A ∧ C| ∈ {25, 26, 27}` — essentially all of A's 28 already sit inside
> C, so **C's section-vocabulary work buys nothing for A's population**; and the
> A-failure bucket ranking computed over C-true TUs has the **same top bucket**
> as over C-false TUs. On that answer A and C are **independent projects** and
> `docs/OBJ_DATA_BSS_SHAPE.md`'s `.data`/`.bss` work does not advance A.
>
> *Rival P5′:* the rankings differ at the top. Then C-true TUs are a structurally
> different population and the two are one project after all.

### P6 — B, at lower resolution

> **P6.** B (338) is **not** a subset relation with A either: `|A ∧ B| ≥ 26`, and
> B's failures are concentrated in `emit-unbound-no-record` (the `wall` split),
> not `emit-unbound-has-record` (the repairable one) — **≥ 60 %** of B-failing
> TUs have at least one emitted symbol with **no body record at all**.

## Priced decline clause

If **P3′ wins** — `rows-unnamed` is the dominant bucket — then after **at most
two further probes** (one dumping the unnamed rows' `.gl` neighbourhood for a
sampled 20 TUs, one re-running with the binding's name-distance bound relaxed) I
**decline** to push the mechanism census further and deliver a characterized
boundary instead.

**The price, stated now so it is not discovered later:** the report will then say
that **A's failure census is instrument-bounded**, will publish the rate, will
state that **no TU-weighted mechanism ranking from this lane is defensible**, and
will name repairing `bind.rs`'s name coverage as the blocking prerequisite for
the question the lane was opened to answer. The roadmap gets **no lever** out of
this lane on that branch. I am accepting that rather than fitting a mechanism
story to rows whose names we do not have.

Symmetrically, if **P2′ wins** (no bucket over 200 TUs), the deliverable is the
sentence *"A fails for many distinct reasons and there is no single lever"*, with
the full distribution attached, and **no** bucket is promoted to a rung.

## What this lane's buckets do NOT predict — registered in advance

Board **#150**, fourth instance. Written here *before* the numbers exist so it
cannot be read as an excuse attached afterwards:

* A bucket's **TU count does not predict the work** to close it. w-pair's two
  "cheapest" frontier TUs needed an instruction scheduler and an EH subsystem;
  w-cfgimpl disassembled all five single-blocked-function frontier TUs and found
  **every one framed**, with REFHI/REFLO pairs and `__savegprlr_26`.
* A bucket's TU count **does not predict TU yield**. A is necessary and **not
  sufficient** — `A∧B∧C∧(D∨E)` is what a byte-exact obj needs, and A alone
  converts **zero** TUs. Every A-side number in the report is published beside
  its `A∧B∧C` counterpart for exactly this reason.
* The buckets are **mechanism labels on a signed count**, not a claim that one
  code change closes one bucket.

## What this lane will not do

* **Not** touch `crates/c2-il/` or `crates/c2-core/`. The only code change is
  additive read-only keys in `crates/c2-harness/src/gap.rs`; no existing count
  moves, and that is checked by re-running the incumbents.
* **Not** edit `docs/BOARD.md`, `ROADMAP.md` or `INDEX.md` — rows are proposed in
  the report. Board numbering is contended (w-pair and w-cfgimpl both proposed
  196–200; w-repro numbered from 201); I verify against the file and state what
  I assumed.
* **Not** glob or walk `work/capture-cache` — that OOM-killed this box twice.
* **Not** fix anything it measures. This is a measurement lane.
