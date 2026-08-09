# w-readpx — PREREG

**Frozen before the first scan of this lane.** Nothing below was measured by this
lane; every prior number quoted is read out of a document already in the tree and
is cited. Committed as this lane's first commit, before any instrument exists,
per the standing rule and per #2096 (the lane-name claim goes in the same commit
as the PREREG).

Lane `w-readpx`, branch `wt-w-readpx`, board rows **#2280**–**#2299**.
Base: master tip at the time of freezing (see §0).

---

## §0 Base, and what is inherited rather than re-derived

Base commit: `0c8cf27e` — *"docs: CEILING §10 — the 2026-08-09 addendum, three
independent measurements of the same distance"*.

Inherited claims this lane will **re-derive at base rather than trust**, per
§10.27.2's own worst finding (a lane that inherits a prior identification from
its brief instead of re-deriving is the ninth instance of that failure this
week):

* `WB_READER_FINDINGS.md` §1's 48-row table (base `c34c388c`, morning of
  2026-08-08, FRONTIER **16** TUs / **59** emitted).
* `w-value` #1943's `9B` = 1,590 and `64` = 546 of 2,306.
* `WB_EH_FINDINGS.md` R1 and its 682.
* `w-inlfence` #2222's `frontier-codegen-wrong` **1 → 0**.

Inherited and explicitly **cited, not re-derived** (a concurrent lane owns it):
`src/Main.cpp`'s blocker chain is `w-main`'s (#2260–#2279). If this lane's tables
name Main.cpp blockers, they are cited to that lane.

---

## §1 The arithmetic this lane expects to walk into

Read off the tree, not measured here:

* `WB_READER_FINDINGS.md` §1: FRONTIER 16 TUs, 59 emitted = 10 exact + 1 wrong +
  0 codegen-refused + **48 reader-refused** + 0 ungraded, at `c34c388c`.
* `CEILING.md` §10 (today): `a-and-b-and-c` **27**, `match` **18**, `frontier`
  **9**.
* `w-inlfence` (§10.29): *"across nine frontier TUs and 51 emitted functions"*,
  and `frontier-codegen-wrong` **1 → 0**.

So the frontier lost **7 TUs and 8 emitted functions** between `c34c388c` and
now. **A TU cannot convert while it holds a reader-refused function** — TU match
is a conjunction over the whole obj, and a `fnbyte-refused-parse` function is
not byte-exact. Therefore the 8 that left were, at `c34c388c`, in the `exact`
column, and the reader column should be **unchanged at 48**.

### P1 — the band on deliverable 1

| # | prediction | p |
|---|---|---:|
| **P1.1** | `frontier-codegen-reader` at this tip is in **[44, 50]** | 0.85 |
| **P1.2** | it is **exactly 48** | 0.55 |
| **P1.3** | it is **49** — because `w-inlfence`'s fence files `?supershuffle@@YAXPAD@Z` as a **parse** decline, so the function `frontier-codegen-wrong` gave up lands in the reader column rather than vanishing | 0.35 |
| **P1.4** | `frontier-codegen-exact` at this tip is **2** (10 − 8) | 0.50 |
| **P1.5** | the number of **distinct frontier TUs** carrying at least one reader-refused function is **9** — i.e. every remaining frontier TU has one | 0.70 |
| **P1.6** | **no** function on `WB_READER_FINDINGS.md` §1's 48-row list has been recovered by any of the six conversions or five reader-widening lanes that landed since; the set difference base→tip on the reader column is **0 recovered** | 0.65 |

**P1.6 is the deliverable-1 headline and it is registered as a HIT for the
morning lane, not for this one.** If it holds, the eleven landings moved the
frontier's reader column by zero and the interesting content of this lane is
entirely in §2–§5.

### P2 — the top-3 keys and their rough shares

Registered from `WB_READER_FINDINGS.md` §1 without re-scanning:

| # | prediction | p |
|---|---|---:|
| **P2.1** | rank 1 is `expr-cmp-eq`, holding **9–13** of the column (≈ 23 %) | 0.60 |
| **P2.2** | rank 2 is `expr-jump`, holding **8–11** (≈ 21 %) | 0.55 |
| **P2.3** | rank 3 is one of `assign-store-type-8643` / `expr-op-0x27`, holding **4** (≈ 8 %) | 0.60 |
| **P2.4** | the top-3 hold **≥ 22** of the column | 0.70 |
| **P2.5** | the number of **distinct** first-blocker keys on the column is **≥ 15** | 0.75 |
| **P2.6** | `src/keygen_xbox.cpp` is still the single largest contributor and still contributes **≥ 15** across **≥ 8** distinct keys | 0.60 |

### P3 — the named residues, sized at this tip

| # | prediction | p |
|---|---|---:|
| **P3.1** | the walker's `9B` re-derives within ±15 % of **1,590** emitted, and `9B`+`64` is still **≥ 90 %** of `eat_call_value`'s emitted decline population | 0.55 |
| **P3.2** | `expr-op-0x27`'s **whole-workload** emitted column is **< 500** | 0.70 |
| **P3.3** | `expr-op-0x28`'s whole-workload emitted column is **< 200** | 0.65 |
| **P3.4** | `WB_EH_FINDINGS.md` R1's 682 re-derives within ±10 % | 0.50 |
| **P3.5** | at least one key **not named in the brief** enters the top 5 of the frontier's reader column or of the whole-workload reader-refusal ranking | 0.55 |

### P4 — artifact discipline (the ninth, tenth… instance test)

Eight rankings in a row were artifacts (#150, #441, #1535, #2000, #2020, #2022,
#2031, #2246). This lane registers, before scanning, that it expects a ninth:

| # | prediction | p |
|---|---|---:|
| **P4.1** | at least one of this lane's own top-3 whole-workload candidate keys collapses under **one** of {TU replication, template replication, distinct-construct count} to **< 25 %** of its headline body count | 0.75 |
| **P4.2** | the **demangled-STEM** column (the test #2246 says the `bodies == TUs` test is structurally blind to) catches **≥ 1** candidate that the TU-replication test alone would have passed | 0.55 |
| **P4.3** | the frontier's 48 do **not** collapse — they are ≤ 48 functions in ≤ 9 TUs, so replication cannot inflate them, and their distinct-name count is **≥ 40** | 0.80 |

### P5 — the byte-verdict cross-check (#2080–#2082, #2220–#2227)

| # | prediction | p |
|---|---|---:|
| **P5.1** | for **≥ 1** candidate rung, the functions behind it are demonstrably **not** `fnbyte-exact` where an analogous class is already admitted — i.e. the rung would buy census and lose bytes, w-fltret's shape | 0.70 |
| **P5.2** | **no** candidate this lane ranks has a population that is already majority `fnbyte-exact` | 0.85 |
| **P5.3** | the `callee-defined-in-tu` fence is **fail-open** on ≥ 90 % of every candidate population this lane prices, so the byte verdict for those populations is **unknowable at this tip** and this lane will have to say so rather than predict a delta | 0.60 |

---

## §2 THE REGISTERED CALL ON DELIVERABLE 5

Deliverable 5 asks for the ranked next reader rungs **by predicted `fnbyte-exact`
delta and by TUs converted**, or the honest negative. The brief says the negative
is a live possibility. Registered, in probability form, **mutually exclusive and
exhaustive**:

| outcome | p |
|---|---:|
| **(A)** This lane finds **no reader rung that converts a TU** and **no reader rung with a positive predicted `fnbyte-exact` delta** — the double negative | **0.45** |
| **(B)** A reader rung with a **positive predicted `fnbyte-exact` delta** but **0 TUs converted** | **0.30** |
| **(C)** A reader rung that converts **exactly 1 TU** (and that TU is **not** `src/Main.cpp`, which is `w-main`'s) | **0.15** |
| **(D)** A reader rung that converts **≥ 2 TUs** | **0.05** |
| **(E)** Something the four categories above do not cover, named when it happens | **0.05** |

Supporting sub-calls:

| # | prediction | p |
|---|---|---:|
| **P6.1** | the top-ranked candidate's predicted `fnbyte-exact` delta is **0** | 0.60 |
| **P6.2** | **every** candidate whose population this lane can size has predicted `fnbyte-exact` delta **0 or unknowable** | 0.50 |
| **P6.3** | the largest *census* candidate and the largest *predicted-`fnbyte-exact`* candidate are **different rungs** — a tenth instance of the ranking lesson, on a new column | 0.55 |
| **P6.4** | at least one candidate must be declined on the **inliner** (§10.29's precondition) rather than on its own size | 0.45 |

---

## §3 Decline clauses — what makes this lane report a null rather than a number

* **D1.** If the scratch instrument's per-function print does not sum to the
  published `frontier-codegen-*` block exactly, the table is reported as
  **BROKEN** and no ranking is published off it. Columns are asserted to sum
  (deliverable 1's requirement), not eyeballed.
* **D2.** If a candidate's population cannot be crossed with `fnbyte-*`, its
  `fnbyte-exact` delta is published as **UNKNOWABLE**, never as 0 and never as
  the census number. #2095: *a conversion count is not a result unless it is
  crossed with the oracle.*
* **D3.** No rung is recommended on a body column. #2020 — the body ranking and
  the emitted ranking disagreed by 13×.
* **D4.** No candidate is ranked without **≥ 3 actual blocking bodies read** for
  its key. Counting is not reading.
* **D5.** If `cargo test --workspace --release` differs from master, the lane
  reports the difference before any finding.
* **D6.** The scratch instrument is reverted before the gate and its diff quoted
  in the rung. Nothing under `crates/` is committed.
* **D7.** Every non-scratch change is committed **before** the scratch revert
  (#2087 — `git checkout -- crates/` ate an uncommitted repair in w-fltret).

---

## §4 What this lane will NOT claim

* It will not claim the reader's residue is *decodable* or *not decodable* —
  `WB_READER_FINDINGS.md` §0 already measured that the port's width grammar
  covers 48 of 48 and that admitting the whole relational family and the whole
  control-flow vocabulary each moves the column 48 → 48. This lane prices
  **admission**, not width.
* It will not propose a `DISCLOSURE.md` row.
* It will not re-derive `src/Main.cpp`'s chain.
* It will not treat `mismatch 0` as evidence of anything (STATUS trap 1).
