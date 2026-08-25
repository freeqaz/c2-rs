# w-decodereach — the DECODE REACH instrument: 98.25 % is a FRAMING claim and the reading I1 is funded to move is 11.39 %

    Tag:       w-decodereach
    Slug:      w-decodereach
    Date:      2026-08-25
    Kind:      instrument rung
    Outcome:   instrument
    Fixtures:  none — instrument rung: the `decode-reach-*` family
               (`crates/c2-harness/src/gap/decode.rs`) + its metrics and printed block
    Census:    +0 — no acceptance predicate moved; `crates/c2-il` is READ, never written
    Base:      master `5a013b8f4` (the `w-unfuse` merge) — rebased; first round was `5db186426`
    Prereg:    `docs/rungs/_2026-08-25-w-decodereach-prereg.md`, the FIRST commit on
               `wt-w-decodereach` (`83b551585`)
    Board:     #3561–#3566
    Spec:      `docs/DECISIONS_2026-08-22.md` decision 13 (owner — the general decode,
               row 4a(i) / I1) · `w-joint3` §9 item 4 · `ROADMAP_SLICING_2026-08-21.md` §5 S0
    Workload:  878 TUs, stamp `15a64d92f197+42949672950+42949672950`, read before AND
               after every arm and **held** across all four scans

---

## 1. RESULT

Decision 13 funded row **4a(i)** at **15–45 engineer-months as a lower bound** and named
the failure mode in the same breath, from 4a's own risk column: *an instrument with no
emit-path consumer, `#3336` at program scale, and unlike `#3336` there is no contrast
case to catch it.* This lane is the consumer. It found the failure mode already present,
in the number a general-decode lane would most naturally have been measured on.

### Result 1 — **98.25 % is a FRAMING claim; the reading I1 is funded to move is 11.39 %**

`GapReport::cflow_decoded_totals` has printed *"2,375,390 of 2,417,794 bodies decoded end
to end (98.2 %)"* as **prose** since long before this lane, collected by nothing and
diffable by nothing. It is **FRAME reach**: the statement-layer walk landed on the segment
tail. **MODEL reach** — *and every operand the walk stepped over was in the decoder's
modeled vocabulary* (`control_flow.rs:474`'s `off_class`, *"set the moment a stepped-over
operand token is outside the modeled class"*) — is a different number:

| strength | bodies | of 2,417,794 | bytes | of 646,881,206 |
|---|---:|---:|---:|---:|
| **FRAME** reach | **2,375,390** | **98.25 %** | 622,484,114 | **96.23 %** |
| **MODEL** reach | **275,295** | **11.39 %** | 35,404,955 | **5.47 %** |
| stopped | 41,657 | 1.72 % | — | — |
| no body to decode | 747 | 0.03 % | — | — |

**8.6× apart in bodies, 17.6× by byte.** A lane measured on frame reach starts its 15–45
month row at 98 % with 1.75 points of headroom; measured on model reach it starts at 11 %.
Both are true. Only one is a progress signal for a general decode, and the tree published
only the other one. Board **#3561**.

### Result 2 — `admitted ⊆ reached` **holds exactly, 707,728 of 707,728, and had never been asked**

`gap/mod.rs:186` states the reason the question was unanswerable: *"The cross-tab is
emitted only for a decoded body. An undecoded one contributes just the bare `cf-…` key —
crossing 'we could not read this body's control flow' with a blocker would be a product of
two ignorances."* So **`cf-*|IN-CLASS` does not exist as a row**, and *"is there a body the
port ACCEPTS WHOLE that the general decode STOPS inside?"* had no answer in the shipped
keys.

    decode-reach-admitted             707,728
    decode-reach-admitted-reached     707,728
    decode-reach-admitted-not-reached       0

The prereg registered **p = 0.55 that this is nonzero** and named it *"the one genuinely at
risk"*. It missed, and the miss is the useful direction: the two independent walkers — 35
whole-function grammars, and the statement-layer scanner — agree perfectly on the accepted
population. Board **#3562**.

### Result 3 — **reach is not the binding constraint on byte-exactness anywhere the judge can speak**

`decode-reach-verified` is the byte judge's own word (`FnByte::Exact` — bytes **and**
relocations) asked of the bodies the decode reached. It reads **35,912**, which is
**numerically identical to `fnbyte-exact`**. So every function real c2's bytes agree with is
frame-reached, 100 %, and **that key carries no information by itself** — said here rather
than left for a reader to discover.

What carries information is the pair beside it. **`decode-reach-verified-modeled` is
12,772**, so **23,140 of 35,912 (64.4 %) of the byte-exact population are bodies the decode
does NOT model.**

> **A body the judge calls exact that the decode does not model is a body whose bytes were
> not bought by modelling it.**

### Result 4 — reaching a body buys nothing on its own: **24.00 %**

The emit-path consumer, over c2's own emitted COMDAT leaders, with the denominator the
judge cannot speak on printed rather than left out:

| | count |
|---|---:|
| `reached\|fnbyte-exact` | **35,912** |
| `reached\|fnbyte-differs` | 1,968 |
| `reached\|fnbyte-reloc-differs` | 531 |
| `reached\|fnbyte-partial` | 10 |
| `reached\|fnbyte-refused` | **111,198** |
| `stopped\|fnbyte-refused` | **3,280** |
| `nobody\|fnbyte-refused` | 51 |
| **bound total** | **152,950** |
| `decode-reach-emit-unbound` | **9,255** |
| **the whole emitted census** | **162,205** |

Of the **149,619** emitted bodies the decode frame-reaches, **24.00 %** are byte-exact. And
**every one of the 3,280 emitted bodies the decode could not frame is `fnbyte-refused`** —
the port has never emitted a body the general decode could not walk. Board **#3563**.

### Result 5 — the knob control is a **pair**, and both halves were measured

`C2RS_CFRESIDUE_ADMIT` admits `off_class` arms: it moves the `+expr-modeled` suffix and
**cannot** move the walk's `Ok`/`Err` (`control_flow.rs:645` — decode-only, *"cannot move an
obj byte in either direction"*). A third full 878-TU scan at
`C2RS_CFRESIDUE_ADMIT=load-type,lit-type,store-type,deref,bind,temp`:

| key | base | knob | delta |
|---|---:|---:|---:|
| `decode-reach-reached` (FRAME) | 2,375,390 | 2,375,390 | **+0** ← the negative control |
| `decode-reach-modeled` (MODEL) | 275,295 | **608,734** | **+333,439** ← the positive one |
| `match` · `mismatch` · `fnbyte-exact` | 25 · 0 · 35,912 | 25 · 0 · 35,912 | unmoved |

**A pin that shows only that a parameter is inert is equally consistent with the parameter
being wired to nothing at all** — S0's `the_relaxation_is_actually_wired_to_something`. This
is the first control in the tree carrying both halves on one knob in one run. Board
**#3564**.

---

## 2. THE THREE DENOMINATORS, AND THE SEPARATION

Published as containment, never as a ratio (#3356 / `w-objplan`: a containment claim of
*"seed ⊆ emitted on 853 of 854 TUs"* was **739 empty seeds**, undetectable until the
claimant's own size was printed beside it):

```
  ALL BODIES     observable 2,417,794  ⊇  frame 2,375,390  ⊇  model   275,295
  BY BYTE        observable 646,881,206 ⊇ frame 622,484,114 ⊇ model 35,404,955
  EMITTED ONLY   observable   152,950  ⊇  frame   149,619  ⊇  model    16,013
                                                            ⊇  VERIFIED 35,912
                                                                 …of which model 12,772
```

*(`verified` sits under `frame`, not under `model` — the two lower branches are different
predicates and the containment check asserts each against its own parent.)*

**THE DISCRIMINATING CELLS — the positive check, and the lane's own falsifier.**

    decode-reach-reached-not-admitted   1,667,662     vs   admitted 707,728   =  2.36x

The prereg froze the threshold before measuring: **if this were 0 the lane reports
`FAILED`**, and below 1× the instrument is not measuring reach. At 2.36× the headline
cannot be admission wearing a reach key's name.

**And model reach is not admission either — in BOTH directions, from keys that already
ship**: `cflow-residue-inclass-offclass` **518,956** (admitted bodies that are not modeled)
and `cflow-residue-straight-modeled-blocked` **85,806** (modeled straight-line bodies the
port refuses anyway). *Modeled* neither contains nor is contained in the accepted class.

---

## 3. ESTIMATE vs OUTCOME — all ten predictions scored

Frozen in the prereg §4 before the module existed.

| # | prediction | P | outcome | |
|---|---|---:|---|---|
| P1 | `observable` matches the sibling walk; `partition-broken` 0 | 0.90 | 0 / 0 | **HIT** |
| P2 | body reach in **[90 %, 97 %]** | 0.75 | **98.25 %** | **MISS** — high by 1.25 |
| P3 | byte reach ≥ 2 points **below** body reach | 0.60 | 96.23 % vs 98.25 % = **2.02** | **HIT**, barely |
| P4 | `admitted-not-reached` **> 0** | 0.55 | **0** | **MISS** |
| P5 | …and < 5 % of admitted | 0.70 | — | **VOID** (conditional on P4) |
| P6 | emitted reach **below** all-body reach | 0.65 | 97.82 % vs 98.25 % | **HIT** |
| P7 | every `fnbyte-exact` body is reached | 0.70 | 35,912 of 35,912 = **100 %** | **HIT** |
| P8 | exact rate among reached emitted **< 40 %** | 0.80 | **24.00 %** | **HIT** |
| P9 | required-zero: 0 pre-existing `gap-metric` lines differ | 0.90 | **0 of 498** | **HIT** |
| P10 | the knob moves frame reach by exactly 0 | 0.85 | **+0** | **HIT** |

**Seven hits, two misses, one void — and the miss that matters is not P4.**

**P2's real defect is not its band, it is its SUBJECT.** The prereg registered a range on
"reach" while the lane had not yet separated FRAME from MODEL; the number it was
registering against turned out to be the one with no headroom, and the number that matters
(**11.39 %**) was **not predicted at all**. A prereg that registers a band on a quantity it
has not yet decomposed is registering against a **name**, not a measurement. That is this
lane's registered bias for the next one, and it is a sharper failure than being 1.25 points
off.

**P4 was the one flagged at dispatch as genuinely at risk, and it missed cleanly** — which
is what a 0.55 is for. Its value is not the zero; it is that the zero makes result 2 a
*measured* containment instead of an assumed one, and it is the containment that lets the
2.36× separation be read as reach ⊋ admission rather than as two unrelated numbers.

---

## 4. THE CONTROLS, AND THE ONE THIS LANE DID NOT AUTHOR THE INPUT FOR

*"A green control is a statement about the population it ran over"* — `w-permeasure`'s
re-derivation control was **100.0000 % green through three input defects that inverted its
headline twice**, because it graded the *lens* while every defect was in the *input*.

| control | known answer | reads | can it fail? |
|---|---|---|---|
| `decode-reach-partition-broken` | 0 | **0** | **EXECUTED** — `the_partition_control_fires_when_a_bucket_stops_being_written` drops a bucket and asserts the identity goes red |
| `decode-reach-population-broken` | 0 | **0** | compares against the **sibling walk** (`gap::scan` 1b/1c's `fn_in_class + Σ fn_blockers`), not against its own loop — see §5 |
| `decode-reach-containment-broken` | 0 | **0** | asserts each of the four containments against its own parent |
| `decode-reach-incumbent-disagree` | 0 | **0** | **#3288's second derivation**: `cflow_decoded_totals` (a different map, different skip rules, code this lane did not write) reads **2,375,390**; this walk reads **2,375,390** |
| the residue-suffix mutation | — | — | **EXECUTED** — the whole-string reading is built in the test and shown to disagree, so the key-extraction control is watched going red |
| `decode-reach-graded` | **> 0** | **2,417,047** | the **positive** check: *"the run must have GRADED something"*, printed first, and a zero prints `NO-RESULT` and no number at all |

**The two whose input this lane did not author**, which is the brief's requirement:

1. **The containment (result 2).** Input: the incumbent parser's admissions over real c2's
   own IL. Neither the 707,728 admissions nor the 2.4 M bodies are produced by anything
   this lane wrote — and the *two walkers being independent* is the whole content of the
   check.
2. **The byte-judge cross (results 3–4).** Input: **real c2's own COMDAT bytes**, via
   FBM's per-symbol verdicts, which are **returned from the walk that produced them and
   never recomputed here** — one producer of the judge's verdict (`fnbytes.rs:98`'s
   *called, never copied*; #1464's *never a subtraction of two published totals*).

---

## 5. THE NEAR-MISS THAT IS THIS LANE'S OWN SUBJECT — board #3565

The first draft filed `decode-reach-population-broken` on `observable != census.len()`.
The walk increments `observable` **once per element of `census`**. **That comparison cannot
fail.** It is `#3336` verbatim — *a criterion that could not fail did not pass; it
abstained* — shipped inside the lane funded to prevent `#3336` at program scale.

It is recorded rather than quietly fixed because the near-miss is the evidence: this
failure mode is not exotic, it is **what you get by writing the obvious identity**. The
shipped control names the walk it compares against, at the site, in those words.

---

## 6. THE RENAME CLAUSE, EVALUATED

The prereg §2.2 froze this before measuring:

> If, having built it, the containment reads 0 AND the byte-judge cross adds no cell the
> shipped keys already carry, this lane reports its instrument as a rename and says so in
> those words.

**The containment DID read 0.** The clause does not fire, and the reason is stated rather
than assumed — the cross adds cells nothing in the tree carries:
`decode-reach-verified-modeled` (**12,772**), the whole `stopped|*` column (**3,280 + 51**),
and the MODEL strength (**275,295**) with its byte twin. **What is a restatement, and is
labelled as one:** `decode-reach-verified` alone, which today equals `fnbyte-exact` exactly.

**And the near-rename that the lane avoided is worth more than the clause.** The instrument
*would* have been a rename of `cflow_decoded_totals` if it had shipped frame reach alone —
same population, same predicate, a `gap-metric` key instead of a `println!`. Result 1 is
what it is because the lane went looking for the second strength after the first came back
at 98 %.

---

## 7. REQUIRED-ZERO

**Scan pair** — `scripts/scan_pair.sh`, base `work/w-decodereach/c2rs-base` (master
`5db186426`) vs tip, both arms with the toolchain resolved once and exported, workload
stamp read before and after each arm:

    STAMP HELD: 15a64d92f197+42949672950+42949672950
    IDENTITY DIFF: 27 lines over 498 keys (base) / 525 keys (tip)
    every one of the 27 is an ADDED `decode-reach-*` line
    NOT ONE pre-existing key moved — checked by filtering the diff for
    non-`decode-reach` changed lines: the filter's output is EMPTY

`match` **25** · `mismatch` **0** · `fnbyte-exact` **35,912** · `fnbyte-differs` **1,968** ·
`fnbyte-reloc-differs` **531** — all unmoved, and unmoved *as lines of the diff*, not as
numbers copied out of two reports.

**The key count moved 498 → 525 and that is a result, not noise** (#1002): a new metric is
a new metric. The diff is quoted with **both** denominators for the reason `w-s1c2` §3.2
caught live at `0 lines over 0 keys` — *"0 lines" alone is not a result.*

### 7.1 Proof that the diff can fail

*(see §8 for the gate table's own proof)*

The scan-pair harness's own exit code is the demonstration: this lane ran it **twice**, and
both runs returned **`PAIR: DIFFERS`, exit 1** — because 20 and then 27 keys were added.
A harness that reported `PAIR: IDENTICAL` over an added key would be the defect; it
reported the difference, named the key-count move, and refused to call it noise. The
required-zero claim here is therefore **not** "the tool printed identical" — it is "the tool
printed a difference, and every line of that difference is a line this lane added", which is
a stronger statement and the one the diff file supports line by line.

---

## 8. GATE EVIDENCE

**The verdict LINE is quoted, never the exit code** — `gate.sh` prints
`GATE: REFUSED (DIRTY crates/)` and **exits 0**, so a lane that read the status would
report a refusal as a pass. Every figure below is a count read off the run.

Both arms run from this worktree with `C2RS_COMPILERS`, `C2RS_WIBO` and `C2RS_DC3`
exported explicitly. The base arm is the **same worktree at detached `5db186426`**, so the
two gates differ only in the commits under test.

| | base `5db186426` | tip `c3cfc230b` |
|---|---|---|
| verdict line | `GATE: PASS (HATCH-RED REFUSED)` | `GATE: PASS (HATCH-RED REFUSED)` |
| lanes | 18/18 ran, every one graded a corpus | 18/18 ran, every one graded a corpus |
| fixture-verdicts | 7,038 | **7,038** |
| generated sweep | 19,460 graded of 19,556 | **19,460 of 19,556** |
| mode cross | 90,424 graded of 90,812 cells | **90,424 of 90,812** |
| debug lane | 18/18, 7,038 verdicts, 0 panics | **18/18, 7,038, 0 panics** |
| mismatches | **0** anywhere | **0** anywhere |

    C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release --no-fail-fast
       55 targets · 1,885 passed · 0 failed
       `#[test]` count 1,883 (base) -> 1,895 (tip), +12 — all in `gap::decode`

**The environment is asserted BY DURATION, never by "0 `SKIP` lines"** — that check is
vacuous, because libtest swallows the stdout of passing tests (#3341). `census_gate`
**70.15 s**, `cli_flags` **128.95 s**, `fixture_profiles` **52.74 s**: the toolchain was
live.

### 8.1 The 21-row identity diff, and the proof it can fail

`work/coordinator/gatebase/HOWTO_DIFF.md`'s procedure: normalise the run dir, cut to the
count-bearing columns (`LANE VERDICT graded/total match`), drop the two `n/a`-mismatch rows
(`hatch-red`, `ladder-red`). **The denominator is 21 rows — 18 mode lanes + `expr-sweep` +
`mode-cross` + `debug-lane` — asserted by enumeration and printed at both ends:**

    base rows: 21   tip rows: 21
    IDENTITY DIFF: 0 lines over 21 rows on both sides

**PROOF THE DIFF CAN FAIL, executed before its zero was trusted.** The tip table with a
single row's `match` count decremented by one (`O1 186 → 185` — the `/O1` family is where a
regression shows first) is diffed against the real one:

    1c1
    < O1 PASS 391/391 186
    ---
    > O1 PASS 391/391 185
    diff exit=1

One changed count, one reported line, non-zero exit. **The zero above is a measurement, not
a pipeline that cannot speak.**

### 8.2 `hatch-red` REFUSED is not this lane's, proven two independent ways

The headline is qualified `(HATCH-RED REFUSED)` at **both** ends, so it does not move across
this lane. Beyond that:

1. **`hatch.py` patches only `crates/c2-il/*`, and this lane changed zero bytes of it.**
   `git rev-parse <ref>:crates/c2-il` is **`1f0e99a9061d77f378d21fe9cae79ad89e20dc88` at
   the tip, at the base, and on `master`** — identical at all three. The hatch is exactly
   as stale at this tip as on master.
2. **The main repository's `work/gate_row_history.tsv` reads
   `hatch-red 24 2026-08-20 REFUSED`** — 24 consecutive runs, first seen **five days before
   this lane existed**.

**And the liveness counter under-reports from a worktree, in the direction that blames the
current lane** — this run printed *"REFUSED for 1 consecutive run(s) (first seen
2026-08-25)"*, because `work/` does not follow a `git worktree add` and the copy is fresh.
`w-joint3` §10.1 recorded this at N = 22; it is **24** now and still unfixed. A lane reading
it at face value concludes it broke the row.

### 8.3 What the gate diff does and does not prove

It is the right control for an instrument rung — the deliverable moves no byte, and 0 lines
over 21 rows is that. It is **not** evidence that the `decode-reach-*` numbers are right;
nothing in `scripts/gate.sh` grades them and, by `FUNCTION_BYTE_MATCH.md` §0, nothing there
ever may. What grades them is §4's control table, the byte compare against real c2 (§1
results 3–4), and the knob pair (§1 result 5).

---

## 9. WHAT THIS DOES NOT SAY

* **The reach number is a property of the STATEMENT-LAYER decoder**, recorded as
  `decode-reach-decoder|statement-layer` on every scan. It is the closest thing in the tree
  to 4a(i)'s general op-level decode and it is **not** that decode: it frames and
  classifies, it does not hand a consumable op-level structure to anything. **When
  `w-unfuse` or a later I1 slice replaces the seam, frame reach is expected to DROP before
  it climbs.** A drop across that boundary is a change of instrument, not a regression, and
  only the recorded decoder identity can tell the two apart. Board **#3566**.
* **`w-unfuse` had not merged when this lane measured, and the re-measure is OWED.** Every
  number here is taken on today's **fused** surface, at base `5db186426`, which the prereg
  said in advance. At the close `wt-w-unfuse` is live and unmerged — it has named the two
  halves (`Decoded` / `AdmissionPolicy`) and reports an identity diff of 0 lines with
  **2,417,794 census rows byte-identical per symbol**, the same denominator as
  `decode-reach-observable` here, which is the first cross-lane agreement either has. **The
  instrument rebases over it and RE-MEASURES; it does not carry these numbers across**, and
  `reach_of`'s body is where `Decoded` replaces the statement-layer verdict — one function,
  one locator. (`master` moved to `97cc7ce62` during this lane when **`w-ilarms`** merged;
  that lane is docs-only and cannot move a number here.)
* **A per-body PREFIX distance is not built and its reach is unknown, not zero.** It needs
  the stop OFFSET, and `FnCensus` publishes none. That field lives in `crates/c2-il`, which
  this lane READS and does not write (`w-unfuse` owns those sites). The byte-weighted reach
  is this lane's distance instead — a real second denominator, and not a substitute for a
  per-body one.
* **`decode-reach-stop|<key>` is a FIRST-BLOCKER histogram and neither a distance nor a
  ranking** (#3131: the port stops at its first refusal by design; 19 greedy rungs off such
  a histogram bought reach **0**, while three tokens no mass ranking can name were each
  worth the whole gain). It is emitted sorted by key NAME, never by mass (#3505, bound five
  times). Seven productions over 41,657 bodies:
  `cf-expr-0x00` 2 · `cf-expr-0x08` 9,641 · `cf-expr-0x59` 20,804 · `cf-expr-0x60` 9,772 ·
  `cf-expr-0xBC` 737 · `cf-offadd-type-0x86` 699 · `cf-offadd-type-0xA6` 2.
  **Do not dispatch off that list.**
* **Nothing here licenses an emit.** `decode-reach-*` appears in no `scripts/gate.sh` row,
  no refusal predicate and no accept path; the byte judge is unchanged and remains real
  `c2.dll` under wibo. A wrong emit still scores strictly below the refusal it replaced.
* **This lane did not re-derive `complete-whole:segment-end` or
  `callee-unresolved-tail-call`** and does not build on either (#3511). It uses neither.

---

## 11. SECOND ROUND — re-measured over `w-unfuse`'s split (base `5a013b8f4`)

`w-unfuse` merged. The lane rebased onto master and re-ran the pair rather than
carrying its numbers across, which §9 said it owed.

**Every pre-existing `decode-reach-*` key is identical to the digit.**

| key | before the split | after | delta |
|---|---:|---:|---:|
| `decode-reach-observable` | 2,417,794 | 2,417,794 | **0** |
| `decode-reach-reached` (FRAME) | 2,375,390 | 2,375,390 | **0** |
| `decode-reach-modeled` (MODEL) | 275,295 | **275,295** | **0** |
| `decode-reach-stopped` · `-nobody` | 41,657 · 747 | 41,657 · 747 | **0** |
| `decode-reach-admitted` | 707,728 | 707,728 | **0** |
| `decode-reach-verified` | 35,912 | 35,912 | **0** |

**It did not move, and here is what the split did and did not change.** Two
structural reasons, both checkable without trusting the run:

1. **`w-unfuse` changed zero bytes of `shapes::control_flow`**, which is this
   instrument's seam — and that module *was already a decode-only layer*, which
   is `w-unfuse`'s own **#3559** and `w-ilarms`'s finding independently.
2. **`AdmissionPolicy::RecognizedShape` is the identity on the decode result**,
   so the admitted set could not move either. `w-unfuse`'s **#3558** measured
   the same thing from the other side — 2,417,794 census rows byte-identical per
   symbol, the same denominator as `decode-reach-observable` here, which is the
   first cross-lane agreement the two lanes have.

**So the split made the two questions separately *askable*; it did not change
either answer.** That is what a prerequisite is. Board **#3580**.

**What the split DID buy this instrument is a third strength and a detector**,
off the surface it built for exactly that — see §13.

---

## 12. `ROADMAP_SLICING` §3's TWO FIGURES — one corroborated to the body, one refuted

The coordinator flagged a five-point disagreement in a consumer this lane had
not swept. It is not definitional. **One figure is right and one is wrong.**

### 12.1 The `98.2 %` is the same measurement, and its partition matches exactly

§3: *"a decode-only walker already reaches **2,362,034 of 2,404,438 bodies
(98.2%)** … only **41,657 bodies (1.73%) are undecoded**."*

| term | §3 (its corpus) | here (this corpus) | delta |
|---|---:|---:|---:|
| reached | 2,362,034 | 2,375,390 | +13,356 |
| **undecoded** | **41,657** | **41,657** | **0** |
| no body to decode | 747 (derived) | **747** | **0** |
| total | 2,404,438 | 2,417,794 | +13,356 |

**The whole corpus difference is in the `reached` term and the other two are
identical to the body.** Two walks, two trees, one answer. Same predicate.

### 12.2 The `83.5 %` is WRONG, and §3's own table refutes it

The ten constructs sum to **2,036,842** at cumulative **97.3 %**, so the table's
denominator is `2,036,842 / 0.973 = 2,093,363`. At 83.5 % that denominator would
be `0.835 × 2,404,438 = 2,007,706` — **smaller than the rows it contains**, so
the ten constructs would sum to **101.5 % of their own population**. A
first-reason partition cannot do that. 2,093,363 is **87.1 %** of 2,404,438.

Measured here, from the same `cflow-offclass-*` decomposition the table is built
from — and the identification is checked twice, not asserted:

* the rows sum to **2,100,095**, which is `reached − modeled` (2,375,390 −
  275,295) **exactly**; and
* `load-type` reproduces at **221,583**, *identical to the body* to §3's row.

| | §3 | measured |
|---|---:|---:|
| ≥1 operand outside the semantic model | ~~83.5 %~~ | **88.61 %** (2,141,752 of 2,417,047) |
| the complement, "in the semantic model" | ~~16.5 %~~ | **11.39 %** (275,295) |

**So it is a real disagreement, not two definitions — and the hole row 4a(i)
must fill is 5 points WIDER than §3 priced it, not narrower.**

### 12.3 And the complement equals MODEL reach exactly, which §3 could not have known

`decode-reach-inmodel` is **275,295** and `decode-reach-modeled` is **275,295**.
The two predicates differ *in principle* — §3's counts a body that stopped
before anything took it off-model; MODEL reach requires the walk to have reached
the tail — so their agreeing is a **measurement**: **every one of the 41,657
undecoded bodies had already left the semantic model before the walk stopped.**

The difference in principle is pinned by
`the_slicing_predicate_is_wider_than_model_reach`, so the coincidence can never
be read as an identity by a later tree where it stops holding. Board **#3581**.

### 12.4 The sweep — ENUMERATED, not grepped-and-stopped

`w-ilarms` found a banner's consumer list short by two, both on a shelf no topic
grep reaches. So the shelf was enumerated: **every live doc that prices 4a(i)/I1
at all** — `ARCHITECTURE_PROPOSAL_2026-08-20.md`, `ARCH_REVIEW_2026-08-21.md`,
`STEP5_PRICING_2026-08-21.md`, `GOAL_DECISION_2026-08-21.md`,
`DECISIONS_2026-08-22.md`, `whitebox/READ_PLAN_2026-08-21.md`,
`whitebox/WB_MIDDLE_INTERFACES.md`, `whitebox/ref/P_ILRECORD.md`,
`FUNCTION_BYTE_MATCH.md`, `IL_DECODE_REACH.md`, `ROADMAP_SLICING_2026-08-21.md`.

**Result: exactly one of them quotes a reach figure — `ROADMAP_SLICING` §3.**
The others price I1 off the *structural* claim (*"`BodyShape`'s 35 grammars are
simultaneously the admission gate"*), which is `w-unfuse`'s to amend and which
its **#3559** already has. Amended in place, inline at the struck line, per
`DOC_CONVENTIONS.md` §2 mitigation 1.

**Every other `98.2 %` in the tree is a different 98.2 %** and the enumeration is
what establishes that rather than a grep that stopped: `DIFF_STRUCTURE.md` (diff
clusters), `ROADMAP.md` ×2 (the `26` separator; virtual member functions),
`rungs/2026-08-02-w-witness.md`, `rungs/2026-08-06-w-bytes.md`. None is reach.
`rungs/2026-08-20-refrev.md`'s `2404438` is the **census** denominator in a dated
rung record and is left alone.

---

## 13. THE THIRD STRENGTH, AND THE LAYER THE SPLIT DID NOT REACH

`w-unfuse` built `IlBundle::decode_bodies()` for this instrument (**#3555**).
Consuming it adds **GRAMMAR** reach — *did the decode reach a whole-function
grammar* — and a detector.

| strength | key | bodies | of 2,417,794 |
|---|---|---:|---:|
| FRAME | `decode-reach-reached` | 2,375,390 | 98.25 % |
| **GRAMMAR** | `decode-reach-grammar` | **711,729** | **29.44 %** |
| MODEL | `decode-reach-modeled` | 275,295 | 11.39 % |

**They are not a chain, and the cells say so**: `grammar∧model` **190,865**,
`grammar-not-model` **520,864**, `model-not-grammar` **84,430**. `frame ⊇ model`
holds; grammar contains neither and is contained in neither.

### 13.1 The detector reads 4,001 on its first run — and 4,001 is the baseline

`decode-reach-grammar` **711,729** vs `decode-reach-admitted` **707,728**.
`admitted-not-grammar` is **0**; `grammar-not-admitted` is **4,001**, decomposed
in the same walk that filed it:

| census verdict | bodies |
|---|---:|
| `callee-unresolved-tail-call:eof` | 2,282 |
| `data-sym-unresolved:eof` | 1,665 |
| `data-sym-not-extern:eof` | 52 |
| `callee-defined-in-tu:eof` · `data-sym-strlit-fenced:eof` | 1 · 1 |

**Every one is `:eof`** — the parse ran to the end of the segment and the refusal
came *afterwards* — and every one is **symbol binding**. `census_functions` runs
`shape_to_function` after the admission predicate (`census.rs:957`), and that
step resolves callees and data symbols through `.gl`; `Decoded` is upstream of
it.

> **`w-unfuse` unfused the GRAMMAR layer. There is a third layer under it —
> symbol binding — still fused into the census's admission verdict, and it is
> 4,001 bodies wide.**

### 13.2 …and shipping the pairing that reads zero would have been `#3336` twice in one lane

`Decoded::is_admitted` is *defined* as `Decoded::reached_shape`, so comparing
them **cannot fail**. That pairing reads 0 forever and says nothing — the same
shape as this lane's own first population control (**#3565**), in the same lane,
for the second time. The shipped detector pairs against the **census's** verdict,
which can disagree, and does.

**So the I1 progress signal is the CHANGE in 4,001, measured against that
baseline — never its distance from 0.** A lane grading its first slice against
*"0 means nothing landed"* would have started 4,001 off.

Three of the five keys are exactly S0's relaxation population (`data-sym-*`,
`#3392`) and one is `#3511`(b)'s named catch-all. **None of the bodies is new.
What is new is that they are the residue of an incomplete unfusing rather than
ordinary blockers.** Board **#3582**.

---

## 14. FOUND AND NOT TAKEN

1. **`decode-reach-verified` is `fnbyte-exact` today.** It becomes informative the moment
   frame reach stops being 100 % of the judge-verified population — i.e. the moment a
   stronger decoder lands at the seam. Kept, labelled, and not deleted, because a key that
   is redundant *now* and load-bearing *after the seam moves* is exactly the key a later
   lane will wish had been collected on both sides.
2. **64.4 % of byte-exact bodies are not model-reached** (result 3). Nobody has asked
   whether that fraction is stable, or whether it is concentrated in one template family —
   `w-empty`'s 1,373 were all one STLport template and `w-fix`'s 143 all one tier above it
   (#925, #952). One cross-tab against the mangling class answers it.
3. **`cf-expr-0x59` is 20,804 bodies, half the stopped population**, and
   `docs/IL_DECODE_REACH.md` §7 listed it at 16,016 on 2026-07-31 as *"appears between two
   FP arithmetic ops"* — still unidentified 25 days later. **Named here, not ranked**: the
   preceding bullet's rule binds this one too.
4. **The prose reach line at `cli/gap.rs:679` is now shadowed by a key that says more.**
   Not removed — removing a line another doc may cite is a different lane's call — but a
   reader comparing the two will find `98.2 %` beside `98.25 %` and should take the second
   with its MODEL twin.
