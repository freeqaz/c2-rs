DRAFT for `docs/ROADMAP.md` §9.15 — written by lane `w-eh`, to be landed by the
coordinator. Nothing in §1–§9.14 is touched. Pre-registration:
`docs/rungs/_2026-08-01-w-eh-prereg.md`, committed at `689ba57` before the first
capture. Full record: `docs/EH_RECORDS.md` §11.

---

### 9.15 W-EH — the EH records by name, and the label gaps are the §1.1 surcharge block (2026-08-01)

Lane `w-eh`, boards **#133**, **#121**, **#138**. **Measurement and
transcription only: no port code, no `crates/` change, census moves by 0 by
construction.** That was the registered expectation and it is the outcome.

Two corrections to the brief this lane was given, both found before any
measurement and both worth recording:

* **`docs/EH_RECORDS.md` already existed** — 1,711 lines, §1–§10, derived from
  **obj bytes**. It was not this lane's to create. That is a better starting
  position than a blank page, because it makes the byte model a **control that
  can go red**: #133 becomes a second, name-carrying source for a layout already
  fitted, which is the #136 relationship (§9.9.3) rather than a transcription.
* **#121 as the brief states it is not the board item §9.2 names.** §9.2 attaches
  #121 to the EH records; the brief describes it as
  `codec::gl_offset_framed`'s over-fit (`GAPS.md` §8.2). Both were addressed and
  they are unrelated artifacts — see §9.15.2.

#### 9.15.1 #133 — the layout, from 21 shapes rather than one

`scripts/gt_eh_cod.py`, **110 listings, 110 captured** — 15 EH shapes × 4 flag
sets (`/O1 /Oi /EHsc`, `/O1 /Oi /EHa`, `/O2 /EHsc`, `/Ox /EHsc`), plus 5
held-out `maxState` shapes, 5 held-out gap combinations and 40 single-axis gap
probes. The axes are **structural counts**, which is the §9.13.1 lesson
applied rather than quoted: try blocks 0–4, nesting depth 0–4, catches per try
1–4, destructible objects 0–5, functions per TU 1–3, and every catch form
(value, `&`, `const&`, pointer, ellipsis). Two probes fitted; the rest held out
with their counts registered in the script before capture.

The full field-by-field layout is `docs/EH_RECORDS.md` §11. What is new against
the byte-derived §8.3:

* **§8.3's `FuncInfo` is confirmed 9 of 9** — no field moved, none added, still
  no `dispUnwindHelp`. The control could have gone red and did not.
* **`maxState` = (destructible objects) + 2 × (lexical `try` blocks).** A try
  block is worth **two** states. Every A2 miss was this cell and all in one
  direction. Registered and graded on **five shapes it was not fitted on**,
  including a four-deep nest and a four-block sequence — the two arrangements
  that separate "per try block" from "per nesting level" — **10 of 10 exact.**
* **Try blocks are emitted INNERMOST FIRST**, with the enclosing block's
  `tryLow..tryHigh` spanning the inner one. §8.3 never fixed that order; a table
  built in source order is wrong on every nested function.
* **The 8-byte pad is printed, not inferred.** §8.3 *proved* the 9-dword
  `FuncInfo` from two symbol offsets; the listing emits a literal `ORG $+4`.
  Both pad values occur (0 on 13 probes, 4 on 50).
* **`/EHa` is accepted, and it scopes two of §8.3's constants to `/EHsc`.**
  `EHFlags` is `01H` under `/EHsc` and **`00H`** under `/EHa`; the `catch(...)`
  `adjectives` is `040H` under `/EHsc` and **`00H`** under `/EHa`. Every other
  record, field and count is identical between the modes on all 21 probes. §8.3
  measured "1 on all 21" and "`0x40` ellipsis" in the only mode it ran.

**The residue, named, because a correspondence graded on totality needs one.**
`__catchsym$F$k` — the `$k` suffix is **NOT MODELLED**. It is a `STATIC` symbol
whose name reaches the obj string table, so a wrong `$k` is a wrong-bytes obj.
On a sequential-try ladder the first `$k` equals `maxState` and the rest ascend;
**`h_catch4` refutes that as a law** (`maxState` 2, `$k` 6), and `h_2fn` shows it
is **per function** — two functions in one TU both get `$2`. Phase 5 needs this
and does not have it. Also open: `nIPMapEntries` for try shapes (§9.7 already
refuted the no-try rule there, and this lane **declined** those nine cells rather
than guessing, scoring them zero), and `adjectives` `0x02`.

**Totality, and why the headline number is not the evidence.** Every datum
claimed by a named field: **598/598 fitted, 2,436/2,436 held out, residue 0.**
That is exactly the shape this project reads as success when it is absence — and
here the failure mode is concrete. c2 **run-length-encodes**: `DD 2 DUP(00H)`
carries `nTryBlocks` *and* `pTryBlockMap` in one operand. The first version of
this instrument read `__ehfuncinfo$` as **8 dwords, residue 0, every field
claimed**, with `pIPtoStateMap` decoded onto `nIPMapEntries`.

So totality is graded beside an **arity** check that predicts each record's
length from a count field in a *different* record: **332/332 consistent.** Three
falsifications:

| mutation | totality | arity |
|---|---|---|
| the `DUP` expansion removed — the bug that really happened | **residue 0, SILENT** | **16 red**, `FuncInfo got 8 want 9` |
| `FuncInfo` truncated to 8 named fields | residue 8 / 60 | — |
| `HandlerType` read as 5 dwords (x86's `copyFunction`) | residue 36 / 240 | — |

**The first row is the finding.** The mutation that actually occurred is
invisible to the residue metric. *A totality count cannot see a short read* —
it needs a length predicted from somewhere else.

#### 9.15.2 #121 — NOT settled, and the number in `GAPS.md` §8.2 is 38, not 34

**Verdict: the listing does not settle #121, and it cannot.** `.cod` is an
artifact of c2's **output**; `codec::gl_offset_framed` frames records in c2's
**`.gl` input** bundle. §9.5 already refuted the existence of any IL dump with a
positive control, and nothing in 110 listings names a `.gl` offset.

That statement is unfalsifiable on its own, so it was given a number. The `.cod`
names every **emitted** function and #136 proved that set equals the obj COMDAT
set exactly — so the listing *can* adjudicate the emitted subset, and only that.
On `src/App.cpp`, where the over-fit bites: **158 emitted functions against
6,069 framed records = 2.6 %.** The listing is silent on the other 97.4 %,
because they are bodies that never reach an obj. Registered ≤ 5 %; measured
2.6 %. (The 158 is `GAPS.md` §8.2's figure carried through #136's proven
identity, not re-measured here.)

**Re-measuring the three figures rather than quoting them changed one of them.**
Directly over the cached `.gl`/`.ex` for `src/App.cpp` (`.gl` 1,512,566 B, `.ex`
2,552,214 B):

| `GAPS.md` §8.2 | re-measured |
|---|---|
| loosened predicate finds **6,069** | **6,069** — exact |
| of which **6,068** land on a `4F 1F` start | **6,068** — exact (the one miss is `0x0B0004F5`, far past the end of `.ex`) |
| shipped predicate finds **34** | **38** |

**34 is not the framing count.** The framing predicate hits **38**; the reader's
32-byte name bound then drops 4 as `records_nameless`, leaving 34.
`GAPS.md` line 2592 says "the gate's *reader* finds 34" and is correct; the
doc comment at `crates/c2-il/src/func/bind.rs:84` says "the gate's **framing**
therefore finds 34" and is **wrong by 4**. Two further corrections fall out:

* only **31** of the 34 pass `looks_mangled`, and `gl_defined_names` is
  all-or-nothing — it returns empty on the *first* framed hit with no nearby or
  non-mangled name. So `Bindings::per_record` binds **0** functions on
  `App.cpp`, not 34. "34" is *framed records the reader could name*, not
  *records that bind*.
* the `.ex` carries **9,196** `4F 1F` markers, not 9,033 — a different quantity
  (the census anchors on the `LO` marker) and easy to conflate. The loosening
  recovers 6,069 / 9,196 = **66.0 %**.

**So #121 stands open and needs a different instrument.** The over-fit is real
and confirmed at the two figures that matter; the listing is not its remedy.
`crates/c2-il` was not touched — lane `w-rerank` owns it — and the correction
above is a doc-comment fix for whoever does.

#### 9.15.3 #138 — the gaps are the §1.1 surcharge block, and they ARE additive

§9.12 measured `last funclet → first EH-state $M` at **2–11** and `state table
$T → first triple` at **0–3**, and refused to model them. The refusal was
correct. The reason is now measured, and it is **not** what the brief's three
candidates proposed.

**The leading registered hypothesis was wrong and it was cheap to kill.** C1
predicted ≥ 90 % of the gap slots would turn out to be labels the §9.12 parser
never read, under a prefix like `$LN`. Those labels **do** exist —
`$LN12@f`, `$LL3@f` — and they are a **separate, small, per-function** space
(observed 1..17) with no relation to the TU counter (25xx). **0 % of the gap
slots are named anywhere in the listing.** REFUTED.

**What governs them.** Holding the EH shape fixed at one destructible local and
moving **one axis per probe**:

> **G = 2 + 2 × [`f` is the FIRST emitted function in the TU] + Σ(`f`'s own
> `LABEL_COUNTER.md` §1.1-style surcharges)**

| axis moved | ΔG | note |
|---|---:|---|
| a **string literal** (an `.rdata` COMDAT + a `??_C@` symbol) | **+0** | **THE CONTROL.** §2.1 measures it at 0 slots; if G had moved, the model was dead |
| k **discarded** unreferenced statics, k = 1,2,4,8 | **+0** | 5 cells |
| a signed relational over two call results | **+2** | §1.1's exact integer, and it mints nothing |
| `_fltused` + a newly pooled FP constant | **+3** | §1.1's 1 + 2 |
| a loop | **+4** | not in §1.1's table |
| ≥ 1 extra call to a function declared elsewhere | **+2**, **flat** in k = 1..4 | |
| a try/catch instead of a bare destructor | **+3** | |
| **each body inlined into `f`** | **+3**, exactly linear | |
| a preceding emitted function | **−2** | see below |

**The −2 needed a discriminator and got one.** A ladder of k preceding emitted
leaf functions drops G from 4 to 2 and then **saturates** — which is consistent
with both "the first emitted function in the TU pays 2" and "a TU with more than
one function is different". The same leaf functions placed **after** `f`
(`x_trail1..4`) leave G at **4**. So the charge is paid by the **first emitted
function**, and ladder A alone was a control run where the discrepancy could not
appear.

**Graded on combinations it was not fitted on.** Five probes combining terms
(loop + 2 inlined + led; relational + 2 inlined; string + loop; led + 2 external
calls; led + pooled constant), predictions registered before capture: **5 of 5
exact.** The terms **add**.

**The answer to the brief's three candidates, separately:**

1. **Per-TU vs per-function counter resets — refuted.** The counter is monotone
   across every function boundary and no number is reused. Registered as
   expected-inert and it was.
2. **Labels consumed by bodies inlined away — CONFIRMED, at +3 each, and this
   nearly went unmeasured.** The first ladder used `static` callees and the
   second `__forceinline`, and **c2 emitted every one of them as its own
   COMDAT** — checked by `PROC` count rather than assumed. Both ladders moved
   *two* axes at once (bodies inlined into `f`, and functions added to the TU).
   The isolated term comes from contrasting them against the ladder where `f`
   does **not** call the leading functions: `x_fi_k − x_lead_k` = 3, 6, 9, 12.
3. **Labels allocated by phases that emit nothing — refuted *at G*, confirmed
   *elsewhere*, and the distinction is the point.** Discarded statics move G by
   **0** on all five cells. They do consume the counter: each one advances the
   TU's first label by exactly **3**, outside the block G measures. So "labels
   that reach no obj" is real and measurable — it is simply not the mechanism
   behind the inter-stage gaps.

**So: are the gaps predictable?** The honest answer is a third branch the
pre-registration did not offer. **G is governed by an additive law whose terms
are measured integers, and it predicts held-out combinations 5/5.** It is not a
compiler mystery, and §9.12's "not predictable from the shape" is precisely
right — G is not a function of the **EH shape** at all. It is the ordinary
`LABEL_COUNTER.md` §1.1 surcharge block, which §2.2 already established is
allocated **ahead of** a function's own `$M` pair (`extra == stride − base` on
all 21 framed rows). In a non-EH function that block sits before the pair and
nobody called it a gap; in an EH function the funclet labels are allocated first,
so the same block becomes **visible between the funclets and the ip2state `$M`s**.
§9.12 measured it across TUs whose surcharge content differed and correctly read
the spread as unmodelled.

**This does NOT license a cardinal `plan_labels`, and no `plan_labels` change
ships.** Two reasons, both load-bearing:

* One input is the **set of bodies c2 chose to inline**, at +3 each.
  `LABEL_COUNTER.md` §6.15.3 records that the `/O1` inline-decline schedule is
  *"generated by no formula"*, and §9.5 records that c2's strings **name** the
  emit-set predicate's disjuncts without formula-ising it. The **per-body cost is
  constant; which bodies is not predictable**. That is the precise sense in which
  the gaps are an inlining artifact — and it is a sharper statement than "they
  are unpredictable".
* Two terms are outside §1.1's measured table entirely (a loop at +4, the extra
  external call at +2 flat), and `EH_RECORDS.md` §9.8's own `G = 4 + Σmint` is
  now explained rather than repaired: its base **4 is `2 + 2`** — the true base
  plus the first-emitted-function charge — and **its 27 probes never varied
  which function came first**, so a two-term constant read as one. Its `qLOOP`
  miss (8 against an expected 6) is exactly this lane's loop term, which
  `Σmint` cannot see because a loop mints nothing (§2.1).

A wrong `$M` number is a wrong-bytes obj (§9.12.2). The rule stays **ordinal**.

#### 9.15.4 Pre-registration scores

Registered at `689ba57`, before the first capture; the `maxState` law and the
gap decomposition were each re-registered in their own commit before the
held-out round that graded them.

| | registered | measured | |
|---|---|---|---|
| A1 totality | residue 0 fitted and held out | 0 and 0 | HIT, and **near-vacuous alone** — see A1b |
| A1b arity | *(not registered — added when `DUP` was found)* | 332/332; catches what A1 cannot | — |
| A2 structural counts | ≥ 85 % exact, refuted < 60 % | **79.5 %** (62/78) | **MISS**, not refuted |
| A2′ the corrected `maxState` law | held out ≥ 85 % | **100 %** (10/10) | HIT |
| A3 `.cod` vs §8.3 `FuncInfo` | 9/9 agreement | 9/9 | HIT |
| A4 `/EHa` accepted, `EHFlags` ≠ 1 | accepted, flag moves | accepted, `01H`→`00H` | HIT, **and a second constant moved** |
| A5 `adjectives` by clause | `00`/`09`/`08` | exactly that | HIT |
| A6 structural-count law | counts are a function of the axis; `maxState` rises with (dtors + try) | `nTryBlocks` and the arrays exact; **`maxState` weighs a try DOUBLE** | **MISS** on the stated form |
| B1 #121 settled? | NOT settled, in principle | not settled | HIT |
| B2 fraction the `.cod` can adjudicate | ≤ 5 % | **2.6 %** | HIT |
| B3 re-verify 34 / 6,069 / 6,068 | all three exact | 6,069 ✓, 6,068 ✓, **34 → 38** | **MISS** |
| C1 gap slots named under an unparsed prefix | ≥ 90 %, refuted < 50 % | **0 %** | **MISS**, refuted |
| C2 counter per TU, monotone | no reset, no reuse | none | HIT (registered inert, and inert) |
| C3 inlining moves the gap | it does | **+3 per inlined body, linear** | HIT |
| C4 phases that emit nothing | moves the residue | **+0 at G, +3 each outside it** | **SPLIT** |
| C5 the verdict | branch (a) *or* branch (b) | **neither — a third branch** | **MISS** |

**9 of 15, with one split.** The misses carry this round:

* **C1** was the lane's *leading* hypothesis and it was refuted in the first ten
  minutes by one `grep` for `$`-prefixed labels. Killing it early is what left
  time for the ladders that actually answered #138.
* **C5** is the more interesting failure: the pre-registration offered a
  two-branch disjunction ("accountable from the listing" *or* "an inlining
  artifact, stop") and reality took a third — accountable from the **surcharge
  table**, with inlining as one unpredictable *input* rather than the whole
  story. **A disjunction registered as exhaustive was not**, which is worth more
  than either branch would have been.
* **B3** is the reason re-measuring beats quoting: `34` is a reader count, not a
  framing count, and a shipped doc comment says otherwise.
* **A2/A6** both missed on the same cell, and the miss produced the section's
  best result — a law graded 10/10 on shapes it was not fitted on. An estimate
  that is wrong in a single consistent direction is a law waiting to be written.

Two registered items are called out rather than counted as evidence: **A1**,
which is vacuous without the arity check that was *not* registered (it was added
mid-round when `DUP` was found, so it is an unregistered strengthening, not a
scored prediction); and **C2**, which was registered as expected-inert and is
inert — §9.12's P9 already implied it.

#### 9.15.5 Gate evidence

This lane wrote **no port code**: `docs/` (two files), `scripts/gt_eh_cod.py`,
and nothing under `crates/`. The gate is quoted to show it did not move, not to
claim it as evidence for anything above.

* `cargo test --workspace` — **584 passed, 0 failed, 1 ignored**. This **is**
  the merge-base count: `git diff --stat 99ed418..HEAD` is `docs/EH_RECORDS.md`,
  `docs/rungs/*`, `scripts/gt_eh_cod.py` and nothing else, so no test was added
  or removed. (§9.10's standing metric asks for the diff at both ends; here the
  diff is empty by construction, which is the honest form of it. Note the number
  is **584**, not the 579 of §9.12.5 or the 576 of §9.13.3 — those were
  different merge bases, and quoting a stale total as "unchanged" is exactly the
  §9.10 trap.)
* `c2rs selftest` — **208/208 PASS**, 0 fail, 0 skip.
* `scripts/gate.sh --jobs 6` — **GATE: PASS**, 12/12 lanes in the registry,
  **2,496 fixture-verdicts across all lanes, 0 mismatch in every one**
  (208/208 graded per lane; `/O1` 97 match, `/Ox` 99, `/Od` 1).
* Census, emitted census, and TU match are **unchanged by construction** — no
  acceptance path, no emitter and no census key was touched.
* `scripts/gt_eh_cod.py` — 110/110 listings captured across 4 flag sets.

#### 9.15.6 New board items

* **#143 — `__catchsym$F$k`, the per-function symbol ordinal.** The one piece of
  the EH record set §11 could not model, and it is a *name* that reaches the obj
  string table. Blocks a byte-exact Phase-5 emitter on any function with a try
  block. The `$LN`/`$LL`/`e$NNNN` numbers look like the same space and are the
  place to start.
* **#144 — `nIPMapEntries` for try/catch shapes.** §9.7 refuted the no-try rule;
  this lane declined to guess. `h_try1` 1, `h_try2seq` 4, `h_try3seq` 7,
  `h_nest3` 3 — not a function of any count in `FuncInfo`.
* **#145 — fix the `bind.rs:84` doc comment (38, not 34) and record that
  `Bindings::per_record` binds 0 on `App.cpp`.** One-line doc change plus a
  measured note; belongs to whoever holds `crates/c2-il` after `w-rerank`. Feeds
  #121, which is **still open** and still needs an instrument that reads the IL
  container, not the listing.
* **#146 — repair `EH_RECORDS.md` §9.8's `G = 4 + Σmint`.** The base is `2 + 2`
  (the second 2 being the first-emitted-function charge its 27 probes never
  varied), and `Σmint` should range over **all** §1.1-style surcharges, not the
  minting ones — which is what its `qLOOP` miss already was. Instrument
  correction, not a rung.
