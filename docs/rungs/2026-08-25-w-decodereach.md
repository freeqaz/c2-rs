# w-decodereach — the DECODE REACH instrument: 98.25 % is a FRAMING claim and the reading I1 is funded to move is 11.39 %

    Tag:       w-decodereach
    Slug:      w-decodereach
    Date:      2026-08-25
    Kind:      instrument rung
    Outcome:   instrument
    Fixtures:  none — instrument rung: the `decode-reach-*` family
               (`crates/c2-harness/src/gap/decode.rs`) + its metrics and printed block
    Census:    +0 — no acceptance predicate moved; `crates/c2-il` is READ, never written
    Base:      master `5db186426`
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

*(filled from the run; the verdict LINE is quoted, never the exit code — `gate.sh` prints
`GATE: REFUSED (DIRTY crates/)` and **exits 0**.)*

See §8.1 below.

---

## 9. WHAT THIS DOES NOT SAY

* **The reach number is a property of the STATEMENT-LAYER decoder**, recorded as
  `decode-reach-decoder|statement-layer` on every scan. It is the closest thing in the tree
  to 4a(i)'s general op-level decode and it is **not** that decode: it frames and
  classifies, it does not hand a consumable op-level structure to anything. **When
  `w-unfuse` or a later I1 slice replaces the seam, frame reach is expected to DROP before
  it climbs.** A drop across that boundary is a change of instrument, not a regression, and
  only the recorded decoder identity can tell the two apart. Board **#3566**.
* **`w-unfuse` had committed nothing when this lane measured.** Every number here is taken
  on today's **fused** surface, which the prereg said in advance. The instrument rebases
  over `w-unfuse` and **re-measures**; it does not carry these numbers across.
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

## 10. FOUND AND NOT TAKEN

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
