# PREREG — lane `w-corpushealth`

**Frozen as this branch's first commit, before any measurement of the predicted
quantities.** Probability-form, no discount factor. Base: master `071d2d47`.

Kind: **characterization lane** (`docs/rungs/README.md` § "Lane kinds" 3).
Fixtures: none. Census: +0. Predicted reach: **0 TUs, 0 fnbyte-exact**.
Seams: `docs/` and `work/` only. `crates/`, `fixtures/`, `scripts/`
**byte-identical at both ends** — this is a revert-everything lane, so
`graded tree` identical at both ends applies and is recorded.

---

## 0. The question

The project owner's hypothesis, in his words:

> *"It's possible that c2-rs fails on some of our bytes because they are
> invalid. Not sure if that is an issue or not but maybe worth looking into."*

The workload is **not** Dance Central 3's original source. It is
`../dc3-decomp`, an in-progress decompilation under heavy active development.
The hypothesis to test: **some fraction of the port's refusals are properties
of an immature corpus, not gaps in the port.**

### The three readings of "invalid", separated before measuring

1. **Invalid to c2** — *impossible by construction* for the 870 graded TUs.
   Real `c2.dll` compiled every one of them and the differential replays that
   compile byte-exactly; their IL is valid c2 input **by definition**. This
   lane will state that and spend nothing further on it. The one place c2
   itself refuses is the **8 `capture-fail` TUs**, and those are the live
   example of the hypothesis in its *source-defect* form (`BinkIntegration.cpp`
   at `C2065` — the decomp's own source does not compile).
2. **Semantically wrong but well-formed** — the likely real case, and the
   lane's primary target. Placeholder bodies, stubs, `__assume`/`abort` shells,
   wrong types, hand-written scaffolding: all compile, all produce IL that is
   **unrepresentative** of the shipped game.
3. **Malformed or truncated containers** — captured `.ex`/`.gl` bytes actually
   damaged. Tested directly, not assumed.

## 1. The enumeration rule (published so the measurement is reproducible)

The decomp carries its own progress signal in two independent artifacts. Both
are read-only inputs; **nothing in `../dc3-decomp` is written by this lane.**

* **`build/373307D9/report.json`** — objdiff-cli `4.2.3`, relocation mode
  `functionRelocDiffs=name_check` (read from its own `provenance` block, not
  assumed). 2,224 units. Each unit carries
  `metadata.source_path` (e.g. `src/keygen_xbox.cpp`) — **the exact string
  space of `work/dc3-workload/files.txt`**, which is what makes the join
  possible at all — plus `metadata.complete`, `metadata.auto_generated`,
  `metadata.progress_categories`, and a per-function list with
  `match_percent_normalized`.
* **`decomp.db`** (sqlite) — `functions` table keyed by mangled `symbol`, with
  `unit` (= source path), `verdict` (`COMPLETE` / `AT_LIMIT` / NULL),
  `excluded`, `exclusion_reason`, `is_stub`, `reachable_100`,
  `match_percent_normalized`.

**FINISHED is defined as `match_percent_normalized == 100`** — the decomp's own
canonical ruler (`docs/PROGRESS_METRICS.md`: *"Authorable normalized %"*,
91.21 %). Rationale, stated before measuring: a function whose reconstructed
source reproduces the target's bytes up to register permutation is IL the
original build **demonstrably** could have produced. A refusal on such a
function is a **real port gap**. A refusal on a function the decomp has *not*
matched is **eligible** to be a corpus artifact — eligible, not proven, and
that asymmetry is deliberate: the lane measures an **upper bound** on the
hypothesis and says so.

The c2-rs side is one `c2rs gap` scan with `--jsonl`; the per-TU `emit` map
already carries `fnbyte-exact`, `fnbyte-refused-parse`, `fnbyte-differs` and
the denominator, so **no `crates/` change is needed or permitted**.

Three quantities, per TU `i`, joined on source path:

* `R_i` = `fnbyte-refused-parse` (c2-rs)
* `E_i` = the TU's fnbyte denominator (c2-rs emitted functions)
* `U_i` = objdiff functions in the unit with `match_percent_normalized < 100`

and the two Fréchet bounds on the per-function overlap, because the two
populations are counted per-TU and cannot be joined by symbol:

* **tight upper bound** `Σ min(R_i, U_i) / Σ R_i`
* **lower bound** `Σ max(0, R_i + U_i − E_i) / Σ R_i`

**#3249 compliance.** `fnbyte-*` is a reading of (commit × capture-cache state
× untracked workload). The base is re-read **immediately before** any
comparison, in one uninterrupted block, on one binary; the scan's provenance
block's `workload_head` and the cache hit/miss line are recorded beside every
figure. Any effect under ~10 bodies is **unattributable and named as such**,
never adjusted.

**Probe soundness (#3219/#3231).** A fresh worktree has no `compilers/` and
capture-based work then silently skips. The control is **pinned by name**: the
scan must report the **same 8 `capture-fail` TUs by name** as the registered
baseline and a nonzero cache-hit count over 870 TUs, and the run's wall
duration must be recorded. A reading taken in an environment where that control
did not execute is **void, not provisional** — discarded, re-run, log kept.

## 2. Pre-freeze disclosures (measured before this file was written)

Stated so they cannot be re-sold as findings:

* `work/dc3-workload/files.txt` is **878** lines; **12** begin `src/xdk/`.
* `report.json` has **2,224** units and unit `metadata.source_path` exists.
* `decomp.db` has the columns named in §1.
* `docs/PROGRESS_METRICS.md` headlines: authorable normalized **91.21 %**
  (29,383 / 32,213), **2,830** functions remaining at norm < 100, **416 / 967**
  authorable units complete. `docs/STATE_OF_THE_DECOMP.md` headlines
  COMPLETE 29,655 + AT_LIMIT 3,628 over 33,560 non-excluded rows.
* `dc3-decomp` head at lane start: **`ccd4c8036`** — the same head `#3238`
  measured under.

## 3. Registered predictions

Probability that each holds, judged now, before measuring.

| # | prediction | p |
|---|---|---|
| **H1** | ≥ 90 % of the 878 workload TUs resolve to a `report.json` unit by `source_path` | 0.75 |
| **H2** | Share of the `fnbyte-refused-parse` mass sitting in units the decomp does **not** mark `metadata.complete`: point estimate **57 %**, interval **45–70 %** | 0.55 |
| **H3** | **PRIMARY.** The tight upper bound `Σ min(R_i, U_i) / Σ R_i` is **< 5 %** | 0.80 |
| **H3b** | …and its point value is under **3 %** | 0.6 |
| **H4** | Of the 844 `vocab-gap` TUs, **≥ 300** sit in units the decomp marks complete — i.e. their refusals cannot be corpus artifact at the unit level. Point **370**, interval **300–450** | 0.55 |
| **H5** | **Zero** captured `.ex`/`.gl` containers in a ≥ 40-TU sample are malformed or truncated (reading 3 is a clean negative) | 0.85 |
| **H6** | `decomp.db` `is_stub = 1` rows number **< 500** and occupy **< 100** distinct units | 0.5 |
| **H7** | **THE LANE'S ANSWER.** Corpus immaturity is **negligible** as an explanation of the refusal population — the honest headline is a clean negative, not a material discount on `vocab-gap 844` / `fnbyte-refused-parse 113,447` | 0.70 |
| **H8** | At least one *distinct, nameable* corpus-artifact population **is** found and is worth reporting even though H7 holds (e.g. the 8 `capture-fail` TUs, auto-generated units, or a stub cluster) | 0.8 |
| **H9** | The lane's headline number moves `match` / `mismatch` / `codegen-gap` / `vocab-gap` by **exactly 0** and `fnbyte-*` by at most ±2 (#3249's floor) | 0.9 |

### Invalidation

* If the join rate (H1) is under 60 %, the per-function bounds are **void** and
  the lane reports the join failure as its result rather than a fraction.
* If the base re-read disagrees with the registered baseline on
  `match` / `mismatch` / `vocab-gap` / `capture-fail`, or the capture-fail
  **names** differ, every capture-derived figure in this lane is **void**.
* An `is_stub` / verdict column that is stale relative to `report.json` is
  reported as stale rather than quoted; `report.json` is the build measurement
  and wins on any disagreement (the decomp's own rule).

## 4. What this lane will NOT do

* Not write to `../dc3-decomp` — read-only, it is a peer project with its own
  agents.
* Not touch `crates/c2-harness/src/gap/tests.rs` or
  `crates/c2-harness/tests/` (peer `w-calleeguard`), nor `docs/whitebox/`
  (peer `w-c2map`).
* Not propose dropping TUs from the workload. A refusal that *is* a corpus
  artifact still has to be priced two-sided before anything is fenced
  (`CLAUDE.md`).
* Not hedge. A clean negative is the second-best outcome and will be stated in
  those words.
