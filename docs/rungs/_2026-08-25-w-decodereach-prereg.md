# PREREG — `w-decodereach`: the DECODE REACH instrument

    Tag:       w-decodereach
    Date:      2026-08-25
    Kind:      instrument rung
    Base:      5db186426 (c2-rs master, clean) — `w-unfuse` had committed nothing at prereg time
    Rows:      #3561–#3566 (reserved at dispatch; minted in the commit that uses them)
    Fixtures:  none — instrument rung: the `decode-reach-*` family in `crates/c2-harness`
    Census:    +0 (no acceptance predicate moves; `IlBundle::functions` is not touched
               and `crates/c2-il` is READ, never written)
    Reach:     +0 emitted functions, +0 TUs — required, and it is not this lane's grade
    Funded by: `docs/DECISIONS_2026-08-22.md` decision 13 (owner — the general decode,
               row 4a(i) / I1)
    Status:    REGISTERED BEFORE ANY `decode-reach-*` NUMBER EXISTED

**Failure axis this instrument can fail on even with every byte identical**
(the construct-rung cost clause, `docs/rungs/README.md`, board **#3336**):
**the discriminating-cell count.** An instrument that grades nothing, or that
grades only the population the admission gate already accepts, has abstained
rather than passed. §5.4 states the number that must be printed and the
threshold below which this lane reports **FAILED**.

---

## 0. The question, stated so it can come back "no"

Decision 13 funded row **4a(i)** — a general op-level IL decode — at **15–45
engineer-months as a lower bound** for 4a as a whole, and named the failure
mode in the same breath:

> without this row a step-5 lane's only progress signal is snapshot parity — an
> instrument with **no emit-path consumer**, i.e. **`#3336` at program scale**,
> and unlike `#3336` there is no contrast case to catch it.

**The question this lane answers: how many bodies does the general decode
REACH, and is what it reaches right?**

Two halves, and the second is the one nothing in the tree asks today.

---

## 1. DENOMINATORS AS BELIEVED NOW — before this file existed

Read out of the tree, not remembered. **Every one is expected to move**, because
the `dc3-decomp` workload has moved seven times inside a single lane (#3428) and
the last full statement of the decode figure is **25 days old**.

| quantity | believed | source, and its staleness |
|---|---:|---|
| IL function bodies in the workload | **2,417,794** | `docs/STATUS.md` generated block, tree `b814d1db2` |
| … in the port's accepted class | **707,728** (29.27 %) | same |
| emitted functions (c2's own `.text` COMDAT leaders) | **162,205** | same — FBM's denominator |
| … in class | **39,369** (24.27 %) | same |
| **bodies decoded end to end** (the statement-layer scanner) | **2,318,605 of 2,462,571 = 94.2 %** | `docs/IL_DECODE_REACH.md` §5, **2026-07-31**, a different workload tree and 25 days stale |
| the decode CEILING under the sink family | **93,990 of 120,456 = 78.0 %** | `IL_DECODE_REACH.md` §12.2 / **#3130** — a **different denominator** (120,456), and it is not the same quantity as the row above |
| FBM `exact` / `differs` / `reloc-differs` | 35,912 / 1,968 / 531 | `STATUS.md` block |
| S0 blind reach | **388 of 113,557 = 0.342 %** | `docs/rungs/2026-08-22-s0-blind-reach.md` §1 |

**The two decode figures above are not comparable and this lane will not sum
them.** 94.2 % is over ~2.46 M bodies; 78.0 % is over 120,456. A table that
quoted one under the other's name is this repo's most repeated defect
(`w-joint3` §9 item 1, the 785-vs-833 note). Both are re-derived on this lane's
own tree, with the workload stamp beside them, or neither is quoted.

---

## 2. WHAT THIS IS NOT — the two duplication hazards, named before building

The brief names one explicitly: *"if your design turns out to be S0's with a new
name, say so and stop."* There are **two** candidates, not one, and the second
is the closer call.

### 2.1 It is not `S0` (`gap::blind`)

| | `S0` / `gap::blind` | this lane |
|---|---|---|
| population | the **113,557 parse-REFUSED** emitted functions | **every** body in the workload (~2.4 M), refused and accepted alike |
| what it varies | an **admission** gate (`Relax`, symbol resolution) | **nothing** — it observes the decode that ships |
| what it grades | the **bytes** a relaxed lowering produces | whether the **decode framing** reached the segment tail, and what the byte judge says about the bodies it reached |
| its own verdict on itself | *"S0 has not yet asked §5's question"* — 99.66 % never reached the lowering | this lane asks the reach question directly and reports reach as its headline |

They share a doctrine (`FUNCTION_BYTE_MATCH.md` §0) and no code, no population
and no key.

### 2.2 It is not `GapReport::cflow_decoded_totals()` — and this is the near miss

**`cflow_decoded_totals()` already computes a reach number** (`gap/report.rs:273`)
and `cli/gap.rs:679` prints it as **prose**. If this lane shipped that number
under a new key it would be a rename, and the honest thing would be to say so
and stop. Three things make it not a rename, and each is a thing the tree
cannot answer today:

1. **The reach number has never been GRADED.** `IL_DECODE_REACH.md` §5's own
   ⚠ banner is the proof: 183 bodies were published as *"honest refusals, not
   desyncs"* and **all 183 were desyncs**. A walk that desynchronizes and
   happens to land on the tail counts as *reached* and nothing looks. This
   lane grades the reached population against the byte judge's own verdicts
   (§3.3) and against a second, independent walker (§3.2).
2. **The containment `admitted ⊆ reached` has never been asked, and the data
   to ask it is deliberately not collected.** `gap/mod.rs:186` : *"The
   cross-tab is emitted only for a decoded body. An undecoded one contributes
   just the bare `cf-…` key."* So `cf-*|IN-CLASS` **does not exist as a row**,
   and the question *"is there a body the port ACCEPTS whose general decode
   STOPPED?"* is unasked and unanswerable from the shipped keys.
3. **It is prose, not a `gap-metric` key.** A number `scripts/status.sh` cannot
   collect and a diff cannot compare is not a progress signal for a 15–45
   month row.

**If, having built it, the containment reads 0 AND the byte-judge cross adds no
cell the shipped keys already carry, this lane reports its instrument as a
rename and says so in those words.** That clause is frozen here, before any
number exists.

---

## 3. WHAT IS BUILT — `crates/c2-harness/src/gap/decode.rs`

A new module in the crate this lane owns. `crates/c2-il` is **read** and never
written (`w-unfuse` owns those sites). Its own `TuResult` map, never a row of
an existing one — `gap/mod.rs`'s *"two maps cannot collide"* rule, which that
file records learning **twice**, the second time at a published number that
**more than doubled** with nothing in the diff to explain it.

### 3.1 The partition and the three denominators

Published as **containment, never as a ratio** (`w-objplan`'s lesson, #3356):

```
    observable   ⊇   reached   ⊇   verified
```

| key | is |
|---|---|
| `decode-reach-observable` | every census row the scan saw — **the denominator** |
| `decode-reach-nobody` | rows with no body to scan (`cf-no-body`); published, **never folded** |
| `decode-reach-reached` | the general decode walked to the segment tail |
| `decode-reach-stopped` | it did not |
| `decode-reach-verified` | reached **and** graded right by §3.2/§3.3 |

…and the **byte-weighted twin of every one** (`-bytes-*`, weighted by
`FnCensus::seg_len`), because a body count and a byte count are two
denominators and a reach that is 94 % of bodies may be a different fraction of
the IL. Neither is quoted without the other.

**A first-blocker key is not a distance (#3131).** The stopping opcode is
published as `decode-reach-stop|<key>` and is labelled, in the printed block, a
**first-blocker key that is not a distance and not a ranking** — sorted by key
NAME, never by mass (`#3505`, bound five times; `w-joint3`'s TSV precedent).
The byte-weighted reach above is this lane's distance, and its limit is stated
rather than hidden: a **per-body prefix** distance needs the stop OFFSET, which
`FnCensus` does not publish, and that field is in `crates/c2-il` — a peer's
surface. It is named as owed, not smuggled.

### 3.2 GRADE 1 — the containment control, over an input this lane did not author

**Every body the incumbent parser ACCEPTS must be REACHED by the general
decode.** Two independent walkers over the same bytes: admission parses a whole
segment under one of 35 whole-function grammars; the general decode walks the
same segment with its own vocabulary. If the decode stops inside a body
admission consumed whole, the decode is wrong about that body.

* `decode-reach-admitted` / `decode-reach-admitted-reached` — with sizes, so a
  containment claim cannot be made by an empty set (`w-objplan`: *"seed ⊆
  emitted on 853 of 854 TUs"* was **739 empty seeds**).
* `decode-reach-admitted-not-reached` — the violations. **This is not a
  known-answer-0 control**; §4 registers a real probability that it is
  nonzero, and a nonzero reading is a **finding about the decode**, not an
  alarm about the instrument.

**The input is authored by real c2's front end and by the incumbent parser —
neither of them by this lane.** That is the brief's requirement, and it is
`w-permeasure`'s lesson: a re-derivation control was **100.0000 % green through
three input defects** because it graded the lens while every defect was in the
input.

### 3.3 GRADE 2 — the byte judge's own verdict, crossed with reach

Over FBM's denominator (c2's own emitted COMDAT leaders), the 2×N cross
`decode-reach-emit|<reached|stopped>|<FnByte key>`. This is the **emit-path
consumer** 4a's risk column says an I1 instrument must have: reach cannot rise
without this table saying whether anything the judge can see moved.

It answers *"is what it reaches right"* in the judge's units **where the judge
can speak**, and publishes the denominator where it cannot — the `refused`
column, which is expected to be the overwhelming majority and must be printed
as a number rather than left out.

### 3.4 Named, settable parameters (`GOAL_DECISION_2026-08-21.md` § AMENDED)

* `C2RS_DECODE_REACH` = `on` (default) | `off`. Off is a legal instrument
  state and licenses nothing.
* `C2RS_DECODE_REACH_POP` = `all` (default) | `emitted` | `admitted` — which
  population the printed headline is denominated in. **The keys are emitted for
  all three regardless**; the parameter selects the sentence, never the
  measurement, so a run cannot narrow its own denominator.

An unparseable value is **refused loudly** (`std::process::exit`), never
silently defaulted — S0's rule, and the reason is the same: a scan that quietly
measured a different population would publish a number against the wrong
denominator.

---

## 4. PREDICTIONS — frozen before the first run

| # | prediction | P |
|---|---|---:|
| P1 | `decode-reach-observable` equals the scan's own summed `fn_total` over graded TUs, filed in the same walk (never by subtracting two published totals — #1464), and `decode-reach-partition-broken` is **0** | 0.90 |
| P2 | body-count reach lands in **[90 %, 97 %]** — i.e. `IL_DECODE_REACH.md`'s 94.2 % survives 25 days and a workload-tree move to within 3 points | 0.75 |
| P3 | **byte-weighted reach is LOWER than body-count reach** by ≥ 2 points — big bodies stop more often | 0.60 |
| P4 | **`decode-reach-admitted-not-reached` > 0** — the two walkers disagree somewhere on 707 k accepted bodies | 0.55 |
| P5 | …and if it is > 0, it is **< 5 % of `decode-reach-admitted`** | 0.70 |
| P6 | reach over the **emitted** population is **lower** than over all bodies | 0.65 |
| P7 | among emitted functions the judge calls `fnbyte-exact`, reach is **100 %** (every byte-exact body is reached) | 0.70 |
| P8 | among **reached** emitted functions, the `fnbyte-exact` rate is **< 40 %** — reach is not correctness, stated as a number | 0.80 |
| P9 | required-zero holds: **0 of N** pre-existing `gap-metric` lines differ, and the 21 count-bearing gate rows are identical line for line | 0.90 |
| P10 | the negative control of §5.3 (`C2RS_CFRESIDUE_ADMIT` moves reach by **exactly 0**) holds | 0.85 |

**Registered bias.** S0 went six for six and said so: *"six-for-six is a signal
that the thresholds were set too safely."* P2, P5 and P8 are the loose ones
here and are named as such in advance; **P4 at 0.55 is the one genuinely at
risk**, and it is the one the lane is for.

---

## 5. CONTROLS, AND THE PROOF THAT EACH CAN FAIL

An instrument's controls are worthless until one has been watched to go red.
**Every control below ships with an executed mutation that turns it red**, in a
unit test, run under `cargo test` — not an argument that it could.

### 5.1 Partition — `decode-reach-partition-broken`, known answer 0
Buckets sum to `observable`. **Proof of failure**: a test that drops one bucket
from a synthetic row set and asserts the counter fires.

### 5.2 Population — `decode-reach-population-broken`, known answer 0
`observable` is compared against the count the **same walk** filed, in the same
TU iteration. **Proof of failure**: a test that feeds a mismatched pair and
asserts the counter fires with its direction and size (never existence alone —
an aggregate cannot distinguish `+1400/-27` from `+1373/-0`).

### 5.3 The negative control on the key extraction — `C2RS_CFRESIDUE_ADMIT`
That variable moves the `+expr-modeled` **residue** suffix and **cannot move
the decode's Ok/Err**, by construction (`control_flow.rs:645` — decode-only,
*"cannot move an obj byte in either direction"*). So a run under a non-empty
admit set must move `decode-reach-reached` by **exactly 0**. **It can fail**:
keying reach off the whole `cflow` string instead of its prefix turns it red,
and the test does exactly that mutation and asserts red.

### 5.4 THE POSITIVE CHECK — the discriminating-cell count
*"Absence reads as success. The fix that generalizes is a POSITIVE check."*

`decode-reach-graded` — the number of cells that discriminate — is printed
first, always, including as a zero, and **a zero prints `NO-RESULT` loudly**.
Beside it, the separation that makes the headline a reach number rather than an
admission number:

```
    reached AND NOT admitted   ← the discriminating cells
```

**Registered threshold, frozen here:** if `reached ∧ ¬admitted` is **< 1×**
`admitted`, this instrument is not measuring reach and the lane says so. If it
is **0**, the lane reports **FAILED** in that word.

### 5.5 The one this lane did NOT author the input for
§3.2's containment (input: the incumbent parser's admissions over real c2's own
IL) and §3.3's cross (input: real c2's own COMDAT bytes). Neither input is
produced by anything this lane wrote.

---

## 6. THE DECLINE FLOOR, AND WHAT WOULD FALSIFY THE LANE'S OWN CLAIM

**This instrument claims to measure REACH rather than ADMISSION. It is
falsified by any of:**

1. `reached ∧ ¬admitted` = 0, or < 1× `admitted` (§5.4) — the reach set is the
   admission set wearing a new name.
2. `decode-reach-reached` moving when a knob moves **only** admission, or
   `fn_in_class` / the census numerator / `match` moving when this instrument
   is switched on (`C2RS_DECODE_REACH=off` → `on` must move **zero** of them).
3. The reach number turning out to be computed *from* a verdict the admission
   gate produced. (It is not: `FnCensus::cflow` is documented decode-only and
   *"nothing reads this field except the report"* — but the claim is checked by
   a test that pins reach and admission apart on one row, not asserted.)

**Decline floor — the lane reports `FAILED`, in that word, if:**

* `decode-reach-graded` is 0, or no scan of the workload completes; or
* §2.2's rename clause fires (containment 0 **and** the byte-judge cross carries
  no cell the shipped keys already have); or
* the required-zero identity diff is non-empty on a pre-existing key.

**A nonzero `decode-reach-admitted-not-reached` is NOT a decline condition.**
It is the most valuable thing this lane could find.

---

## 7. THE `#1406` TENSION, RESOLVED EXPLICITLY

`#1406`: *anything whose output is quoted as evidence must run under
`cargo test` or `scripts/gate.sh`.* FBM §0: **never in `scripts/gate.sh`.**

**Resolution, and it is the same one FBM and S0 already ship under:** the
instrument's **controls** run under `cargo test` — the partition, the
population, the key-extraction mutation and the reach/admission separation are
all unit tests with executed mutations, and `#1406` is satisfied by them. The
instrument's **numbers** are produced by `c2rs gap`, published as namespaced
`gap-metric decode-reach-*` lines, and **gate nothing**: no `decode-reach-*`
key appears in `scripts/gate.sh`, in any refusal predicate, or in any accept
path. The byte judge is unchanged and remains real `c2.dll` under wibo.

`#1406` binds the *grading of the instrument*; FBM §0 binds the *instrument's
grading of the port*. They are not in conflict once the two are separated, and
this section is the separation being written down rather than assumed.

---

## 8. FENCES, AND THE ORDERING OWED TO `w-unfuse`

* **Owned:** `crates/c2-harness`. **Read-only:** `crates/c2-il` (`w-unfuse`).
  **Not touched:** `scripts/`, `crates/c2-harness/tests/` (`w-guard`) — if a
  test file is needed there this lane **STOPS and reports** rather than editing
  a peer's surface. Unit tests live in `src/gap/decode.rs`'s own `mod tests`,
  which is inside the fence.
* **`w-unfuse` merges first.** Every number here is taken on today's **fused**
  surface. If `w-unfuse` lands, this lane rebases and **re-measures**; a number
  taken before its landing is a number about a different surface, and this
  file says so in advance so that no later reader has to guess which.
* **Required-zero:** the 21 count-bearing gate rows, identity-diffed against
  the base per `work/coordinator/gatebase/HOWTO_DIFF.md`, **and the diff proven
  able to fail** before its zero is trusted.
