# PREREG — lane `w-submetric` (the per-subsystem metric framework)

**Committed before the first deliverable measurement.** Charter: decision 15,
`docs/DECISIONS_2026-08-22.md` §"Decision 15" / board `#3616`. Board rows
reserved for this lane: **`#3617`–`#3622`**. Base: master `6c753ead0`, branch
`wt-w-submetric`, worktree `.claude/worktrees/w-submetric`.

Predictions below are **never edited after this commit**. They are graded in
`docs/rungs/2026-08-26-w-submetric.md` §"Prereg grade", and a MISS is said in
that word.

---

## 0. DISCLOSURE — what I had already read when this file was written

The protocol says prereg-before-measurement. Orientation happened first, so
the following were already on screen and are **not** predictions. They are
recorded here so a reader can tell a prediction from a thing I had seen.

* `SUBSYS.md` §1's ten rows and its `entries / band` column.
* The `**Coverage:**` line of each of the nine `P_*.md` pages that has one
  (`P_LABEL.md` has none; its numbers are in its title and §0).
* Band function counts recomputed from `docs/whitebox/ref/FUNCS.tsv`:
  coff 120 (inclusive), section 102+35=137 (inclusive), regalloc 70
  (half-open; **71 inclusive**), dag 48+13=61, inline 93 (either), encode 14
  (either), eh 47 (either), symbol band `0x10b28a9b`–`0x10b28d6f` = **1**
  function against `SUBSYS.md`'s cell of `27 / 5`.
* `FUNCS.tsv`'s `subsys` column populations: inline 350, section 327,
  regalloc 230, coff 129, eh 127, dag 83, symbol 5 — a **TU-level**
  attribution (`build_ref.py::TU_PAGE`), a different population from the
  band, and no column at all for `globregs`, `encode`, `label`.
* `P_ENCODE.md` §8.1 (82 of 89 port `encode_*` base words identical to c2's
  table) and §8.2 (`words 634,457 explained 630,548 = 99.3839 %`, strict
  masks; 99.8060 % under the generous second pass, which the page itself says
  must not be quoted as the stronger one).
* Board `#3534`'s byte-ownership figures, which this lane **cites and does
  not re-measure** (decision 15's own instruction).
* `work/w-bss/census/sections.jsonl` — 871 records, real-c2 section order per
  workload TU, `.prov` stamp `2026-08-04T10:06:18Z`, corpus
  `../dc3-decomp` `940d07dcb0960964ad61aa5f025658f993eb46b2`, dirty false.

Everything below this line was **not** measured when it was written.

---

## 1. Predictions

Stated with a probability and a **bias direction in writing**, per
`docs/rungs/README.md`.

| # | prediction | p | bias direction |
|---|---|---:|---|
| **P1** | The **`ported` numerator** of strength 1 ("sites the port implements") is a **named residue for ≥ 8 of the 10** subsystems, because no per-site port↔image map exists in the tree and building one collides with `w-provenance`'s owned quantity | 0.85 | I expect the framework to be **more residue than decision 15's prose implies**; if I am wrong it is because a page-side mapping exists that I did not find |
| **P2** | **`agreement`** is measured for **≤ 3 of 10**; the inliner prints `pending: w-inlmetric` and never a number | 0.80 | biased toward residue; the encoder is the one I expect to land |
| **P3** | **`exercised` in the per-SITE sense is measurable for 0 of 10.** No address-level trace of `c2.dll` over the 878-TU workload exists in this tree; nothing counts which of `P_INLINE`'s 93 functions the workload entered | 0.90 | I expect to have to publish a labelled **output proxy** instead, and to have to say in the doc that a proxy is not the site count |
| **P4** | An explicitly-labelled **workload-output proxy** (not a site count) is available for **≥ 3** subsystems from `sections.jsonl` | 0.70 | — |
| **P5** | **≥ 2 of the 10** `SUBSYS.md` §1 `entries / band` cells are in **different units** from their own page's coverage line, so the cell cannot be read as a fraction | 0.75 | disclosed above: I have already seen two candidates (`P_SYMBOL` `27 / 5`, `P_ENCODE` `14 / 14` vs 79/79 arms). The prediction is that a systematic pass finds **at least one more I have not seen** | 
| **P6** | Every band denominator a page states **reproduces exactly** from `FUNCS.tsv` under one of {inclusive, half-open} — 10 of 10 reproduce, 0 unreproducible | 0.65 | biased optimistic; `globregs` and `label` have no band at all and may have to be declared band-less rather than counted |
| **P7** | The **band** denominator and the **TU-level** (`FUNCS.tsv subsys`) denominator for the same subsystem differ by **> 2×** on ≥ 3 subsystems, so "the subsystem's enumerable sites" is not a single number | 0.90 | already seen on four; the prediction is that the framework must publish **both** or be quoting a ratio whose denominator is a choice |
| **P8** | **Required-zero holds**: `scripts/gate_identity_diff.sh` reports `0 lines over 21 rows` base→tip | 0.90 | the instrument is a new subcommand, not a change to any emit or grading path |
| **P9** | The positive control goes **RED on demand** on both fabrications (a dropped subsystem row; a wrong denominator), watched, before the doc is committed | 0.95 | — |

## 2. Decline floor, fixed now

**The lane DECLINES and reports its outcome as `declined` (not `instrument`)
if any of these is true at the end:**

1. Fewer than **8 of the 10** rows can carry at least one *measured* number in
   strength 1 (band denominator recomputed on this tree **and** a read count
   verified verbatim against its page).
2. The positive control cannot be made to go red — a control never seen
   failing is decoration (`#3336`).
3. `gate_identity_diff.sh` reports any moved row.
4. The instrument would have to enter `scripts/gate.sh`'s **verdict** to be
   runnable. `FUNCTION_BYTE_MATCH.md` §0 is not negotiable and the lane fails
   rather than trades it.

**If every strength on every row comes out a residue, that is a `declined`
with a finding, not an `instrument`** — a table of ten rows of "no
differential exists" is prose with a border, and shipping it as a scoreboard
would be the failure this repo calls decoration.

## 3. What this lane will NOT do, decided in advance

* **Not re-measure byte-ownership.** `#3534` measured it 2026-08-25; the
  framework cites it with its tree (`a8593651b`) and date. "Check the board
  before dispatching."
* **Not build a provenance-marker counter** — `w-provenance` owns that
  quantity this wave (decision 15 § Concurrency fences: *owned surfaces
  include predicates, keys and facts*).
* **Not read `w-inlmetric`'s worktree** or wait on it. The inliner's
  `agreement` cell prints `pending`.
* **Not widen emission, not touch the admitted set, not add a key to
  `gate.sh`'s verdict.**
* **Not edit any `P_*.md` page.** If a page's own number is wrong, the
  instrument records the disagreement in its own doc and the rung; the page is
  amended by whoever owns it.

## 4. The keys, namespaced before they exist

`subsys-<subsystem>-<strength>-<term>`. They are **progress instruments under
`FUNCTION_BYTE_MATCH.md` §0**, adopting its five properties verbatim: never in
`gate.sh`'s verdict; their own block under their own disclaimer; namespaced;
**licensing no emit**; `NO-RESULT` rather than a ratio over zero.

## 5. #1406 placement, decided before the code exists

`#1406` binds any instrument whose output is quoted as evidence to run under
`cargo test` or `scripts/gate.sh`. §0 forbids the second. The resolution is
`decode-reach`'s: the instrument's **logic and its controls live in `crates/`
and run under `cargo test --workspace`**, which `gate.sh` runs as a row; the
**verdict** it contributes to is `cargo test`'s, never the differential's.
The rendered doc is regenerated by a `scripts/subsys_*` runner that shells the
same code, so there is one producer.
