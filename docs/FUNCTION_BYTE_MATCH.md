# FUNCTION BYTE MATCH — the judge's own question, asked per function

**Status: adopted 2026-08-05 (lane `w-fuzzy`). Printed by every `c2rs gap`
scan, machine-readable as `gap-metric fnbyte-match`, collected into
`docs/STATUS.md` by `scripts/status.sh`.**

This document defines **FBM**, the second continuous instrument on this project,
and states its relationship to the first (`PROGRESS_METRIC.md`'s progress mass)
and to `../objdiff`'s fuzzy match. It also records the premise of
`PROGRESS_METRIC.md` that this lane **refuted**, and the defect FBM's own
known-answer control found on its first corpus run.

---

## 0. The separation rule (read this even if you read nothing else)

> **FBM is a PROGRESS instrument and never a correctness criterion. The real
> `c2` under wibo plus the byte-exact whole-obj compare is the SOLE judge of the
> port (`CLAUDE.md`). A scan whose FBM reads 0.9 and whose `mismatch` count
> reads 1 is a FAILING scan.**

Structurally, exactly as the progress mass is separated:

* **It never appears in `scripts/gate.sh`** and must never be added there.
* **It prints in its own block**, under its own disclaimer, apart from the class
  table that carries `match`/`mismatch`.
* **Its keys are namespaced** (`fnbyte-*`).
* **It licenses no emit.** FBM going up cannot be a reason to accept a shape;
  the only thing that accepts a shape is the differential.
* **It is unrepresentable over an empty scan** — `NO-RESULT`, no key, never
  100 % over zero functions.

## 1. The question it answers

`docs/STATUS.md`'s headline is **TU match**, currently **8 of 878**. A TU
matches only if *every byte of the whole obj* matches, so the verdict is a
conjunction over hundreds of functions and it moves only when a TU's **last**
defect closes. The natural follow-up — *we are 8/878 exact, but how close is the
other 870?* — had no instrument.

`PROGRESS_METRIC.md` answered the ranking question (**which lane moved more**)
with the progress mass, `P = mean(a, b, c, f)`. Three of its four terms are
obj-side preconditions (A, B, C); the fourth, `f`, is the emitted census — a
**parse-time acceptance claim**. `STATUS.md` trap 2 states the consequence
plainly: *a per-function census claim for a never-emitted body can never be
graded*, and its recorded precedent for green-and-wrong in exactly that
direction is the `.sy` positional relaxation (census +2,981, mismatch 0, wrong
on 62 % of bindings).

**FBM is the term trap 2 was missing.** It asks the judge's own question — *are
the bytes identical to real c2's?* — of every function c2 emitted, and it asks
it whether or not the surrounding TU will ever match.

## 2. The definition

Over one `c2rs gap` scan of the workload:

```
FBM = (exact + whole_tu) / denominator
```

| term | is |
|---|---|
| `denominator` | every `.text` COMDAT leader in **c2's** objs, over the graded TUs — the same population `emit-emitted` counts |
| `exact` | those whose port body, from `codegen::select_function`, is **byte-identical** to the reference COMDAT's raw bytes |
| `whole_tu` | those on a TU the differential graded `match` that the per-function route did not credit — see §5 |

`select_function` is the port's *own* per-function route: the same decision
procedure `PortC2::build` runs, already public, and already run per function by
the census/gate cross-check (`crates/c2-harness/src/gap/scan.rs` step 1c). FBM
re-uses it rather than writing a second one.

### 2.1 The partition

The walk is **denominator-driven** — it iterates the reference obj's COMDAT
leaders and asks what the port has for each — so nothing is dropped from the
denominator to flatter a ratio. Every emitted function lands in exactly one
bucket, and every bucket is printed:

| bucket | meaning | credited |
|---|---|---|
| `fnbyte-exact` | complete port body, bytes identical | **yes** |
| `fnbyte-differs` | complete port body, bytes differ | no |
| `fnbyte-partial` | the port selected, but the body is finished by the COFF emitter | no |
| `fnbyte-refused` | the port refuses this function | no |
| `fnbyte-unbound` | no census row binds this symbol, or two do | no |
| `fnbyte-nobytes` | the COMDAT's raw data did not decode | no |

`fnbyte-denominator` is their sum and the identity is checked on every scan
(`fnbyte-partition-broken`, known answer 0) rather than assumed. A TU whose obj
does not decode at all contributes to **neither** numerator nor denominator, and
is counted separately (`fnbyte-obj-unreadable`).

## 3. THE ANTI-GAMING PROPERTY, stated precisely

> **The denominator is a function of `c2`'s output alone, and the numerator is
> the judge's own predicate. No input to FBM rewards emitting anything that is
> not byte-identical to `c2`.**

Three consequences, each unit-tested:

1. **A wrong body scores exactly what a refusal scores: zero.** There is no
   partial credit anywhere in FBM. This is the property that disqualified an
   objdiff-style similarity score for this project (`PROGRESS_METRIC.md` §2.2):
   a partial-credit metric pays *more* for a nearly-right wrong emit than for
   the honest refusal it replaced, and board #232's repair was exactly that
   transition in the good direction. Test:
   `a_wrong_body_scores_exactly_what_a_refusal_scores`.
2. **Refusing cannot shrink the denominator**, because the denominator is
   counted off the reference obj. A port that emits nothing scores `0/N` and
   never `N/N`. Test: `a_port_that_emits_nothing_scores_zero`.
3. **Nothing graded ⇒ no number.** `fn_byte_match()` returns `None`, the
   `gap-metric` key is absent, and the printed block says `NO-RESULT`. objdiff's
   own `calc_fuzzy_match_percent` returns **100.0** when `total_code == 0`
   (`objdiff-core/src/bindings/report.rs:249-250`); that is the bug, not the
   baseline. Test: `fnbyte_is_unrepresentable_over_zero_emitted_functions`.

### 3.1 The one lever, and why it is left unpulled

The honest gaming vector is not in the port — it is in the **instrument**.
`Selected::{Tail, Framed, Seq, CondPair}` and a `Float` with pooled constants
hand back a body whose remaining words the COFF emitter finishes, because those
words encode their own `.text` offset. The harness *could* append them itself,
and **9,374 functions would move from `fnbyte-partial` into `fnbyte-exact` with
zero change to the port.**

So the reconstruction is not done here. Closing that class needs a per-function
entry point in `c2-core`, which is the crate that owns the fact (board #322).
Until then **FBM is a floor**: it under-reports the port and never over-reports
it, and `fnbyte-partial` is printed beside the ratio — and is a *required* part
of the `STATUS.md` row, enforced by a must-fail mutation in
`scripts/status.sh --check` — so the size of the under-report is never a rumour.

## 4. What `PROGRESS_METRIC.md` got right, and the one thing it got wrong

`PROGRESS_METRIC.md` §2 read `../objdiff` (MIT/Apache-2.0) from the sibling
checkout and made three claims about transferring its fuzzy match. Two survive
and one does not.

**Accepted — §2.2, partial credit inverts the correctness rule.** FBM has no
partial credit. The objdiff-shaped number (positionally-equal instruction words)
*is* computed, but only over the `differs` class — bodies the port produced and
got wrong — printed as a forensic aid and aggregated into no headline, exactly
the scope §2 says is legitimate.

**Accepted — §2, `total_code == 0 → 100.0`.** Inverted here as `None`.

**REFUTED — §2.1, "a fuzzy match over c2-rs is undefined on 863 TUs and reads
exactly 100.0 on the other 8".** The measurement is correct and the conclusion
does not survive the change of unit. `PortC2::build` refuses a **whole TU** when
any one of its functions is out of class, so *whole-obj* similarity is indeed
undefined on 99.1 % of TUs. But `codegen::select_function` answers **per
function**, and the reference obj's per-COMDAT bytes are already extractable
(`ObjImage::text_comdat_functions_with_bytes`, added for the listing seam). The
port's output is therefore defined on **38,458 of 178,975 emitted functions
(21.5 %)**, and 29,084 of those can be graded against c2's own bytes today.

The refutation matters for a reason beyond bookkeeping. §2.1's argument was that
output similarity "carries zero bits of information today"; at function
granularity it carries the bits trap 2 says the census cannot: **75.6 % of the
progress mass's `f` numerator is now graded by the oracle rather than claimed by
the parser, and the graded part is 100 % correct** (`fnbyte-differs 0`).

## 5. The known-answer control, and the defect it found on its first run

The oracle cannot grade a per-function correspondence in general. On a TU it
graded `match` it already has: a byte-exact obj means **every** function in it
is byte-identical to c2's. So the control is:

> On a `match` TU, the per-function route may legitimately have *no* body — but
> it may never produce a **different** one. `fnbyte-match-tu-differs` must be 0.

Written first as the stronger claim *"every emitted function on a `match` TU is
`exact` or `partial`"*, the control **read 2 where it must read 0** on its first
corpus run, and the two were `src/system/synth/tomcrypt/TomCryptLicense.cpp` and
`src/system/zlib/ZlibLicense.cpp` — the two `??__E` dynamic-initializer TUs.

The finding is real and it is about the instrument, not the port. `PortC2::build`
has **one acceptance route that is not per-function**: it tries
`IlBundle::dyninit_tu()` *before* `functions()`, and on those two TUs it emits
byte-exact through a whole-TU path `select_function` never sees. FBM as first
written credited them zero. (This is the same asymmetry board #179 found in the
factor model, where D — the per-function reading — had to gain E, the whole-TU
disjunct.)

The fix was not to relax the control. The judge's own verdict supersedes the
instrument's route: on a `match` TU every emitted function *is* byte-exact, so
those functions are credited as `whole_tu`, at the hardest possible bar (the TU
must already match), reported as their own line, and never folded into `exact` —
because the two numerators are graded by different routes and a reader must be
able to tell them apart. The control keeps its teeth in the direction that can
still indict the port: a per-function body that *differs* from a certified one
means `select_function` and the COFF emitter disagree.

**This is board #302's shape one level down** and worth stating as a rule: an
instrument built on the port's per-function route will under-count by exactly
the output of every whole-TU route, silently, and the only thing that catches it
is a control anchored on the oracle's own verdicts.

## 6. Measured value

Workload scan, 878 TUs, `capture-fail 7 / graded 871`, tree `64f4754`
(rebased onto `463796d`), workload `fe1b5b3`:

| | |
|---|---|
| **FBM** | **(29,084 + 2) / 178,975 = 0.16251** |
| exact (per-function route) | 29,084 (16.25 %) |
| whole-TU (oracle-certified) | 2 |
| **differs** | **0** |
| partial (FBM's under-report) | 9,374 (5.24 %) — `tail` 7,098 · `seq` 2,150 · `framed` 123 · `cond-pair` 3 |
| refused | 131,292 (73.36 %) |
| unbound | 9,225 (5.15 %) |
| no-bytes / obj-unreadable / partition breaks | 0 / 0 / 0 |
| controls: match-TU differs · census/gate disagree on emitted | 0 · 0 |
| per-TU FBM over 865 TUs with emitted functions | ≥100 %: 4 · ≥90 %: 4 · ≥50 %: 10 · ≥10 %: 699 |

Two identities worth keeping:

* `exact + partial = 29,084 + 9,374 = 38,458 = emit-in-class`, the progress
  mass's `f` numerator, to the digit. FBM grades exactly that population and
  splits it into what the oracle has certified (29,084) and what the instrument
  cannot yet reconstruct (9,374). The `differs` column being 0 is the finding:
  **not one of the census's emitted in-class claims is wrong where it can be
  checked.**
* `fnbyte-tus-full 4` against `match 8`: four of the eight matching TUs have no
  emitted function at all (their objs carry no `.text` COMDAT), so they are
  excluded from the per-TU distribution rather than counted as `0/0`. Same
  exclusion `near_match_tus` makes, for the same reason — never-measured is not
  nearly-done.

## 7. What FBM does NOT measure — its traps, in the STATUS tradition

1. **FBM = 1.0 would not mean the port is done.** A `.text` COMDAT's raw bytes
   are a *subset* of the obj: relocations, the symbol table, the section table
   and every non-`.text` section are outside FBM entirely. It is a **necessary**
   condition, like A/B/C, not a sufficient one. Only the whole-obj compare is
   sufficient, and only it is the judge.
2. **FBM is a floor by construction** (§3.1). Do not read a low FBM as "the port
   lowers little"; read `fnbyte-partial` beside it. And when board #322 lands, a
   jump of up to 9,374 will be an *instrument* change, not port progress — it
   must be reported as one.
3. **`unbound` is an instrument limit wearing a port's clothes.** 9,225 emitted
   symbols bind to no census row; those are the emitted-census residue
   (`GAPS.md` §8) under a second name. A binding repair moves FBM without the
   codegen moving. Read it beside `emit-residue-*`, which already splits that
   population into "compiler-generated, no IL body" and "unexplained".
4. **It says nothing about never-emitted bodies.** 92.8 % of IL bodies are never
   emitted by c2; FBM's denominator excludes them by design, exactly as the
   emitted census does. It is therefore blind to the per-function census's
   largest population — deliberately, because that is the population the oracle
   cannot grade at all.
5. **It is workload-denominated.** Warranty and instrument work — the sweep, the
   mode cross, corpus growth — move it by zero, exactly as they move the
   progress mass by zero (`PROGRESS_METRIC.md` trap 3). Quote the gate's own
   verdict count for that axis.
6. **A `mismatch` TU's functions are still counted by the per-function route.**
   Unlike the progress mass, FBM does not zero a mismatching TU: its emitted
   functions are graded individually, and a function whose bytes are right is
   credited even though the obj as a whole is wrong. That is intentional — FBM
   is not measuring TUs — but it means FBM alone cannot tell you a scan was
   clean. `mismatch` is the alarm; FBM is not.

## 8. Relationship to the progress mass — use both, and read the terms

They are not substitutes and neither subsumes the other.

| | progress mass | FBM |
|---|---|---|
| unit | TUs (three terms) + emitted functions (one) | emitted functions |
| graded by | the reference obj's shape (A/B/C) + the IL parser (`f`) | the reference obj's **bytes** |
| moves on | reachability days *and* codegen days | codegen days, and binding repairs |
| wrong emit | zeroes the whole TU | scores 0 for that function |
| blind to | codegen inside an already-reachable TU | section shape, binding, emit-set — everything that is not a `.text` body |

Rule of thumb: **P says how much of the necessary structure is discharged; FBM
says how much of c2's actual code the port can already write.** A day that moves
one and not the other is a normal day, and the point of having both is that the
day is legible either way.

## 9. Operational notes

* Printed by `c2rs gap` after the PROGRESS MASS block, before `GAP-METRICS`;
  implementation `crates/c2-harness/src/gap/fnbytes.rs` (the measurement) and
  `GapReport::fn_byte_match` (the aggregation), both pure over `results` and
  unit-tested without a toolchain.
* Machine-readable keys: `fnbyte-match`, `fnbyte-exact`, `fnbyte-whole-tu`,
  `fnbyte-denominator`, `fnbyte-differs`, `fnbyte-partial`, `fnbyte-refused`,
  `fnbyte-unbound`, `fnbyte-partition-broken`, `fnbyte-census-disagree`,
  `fnbyte-match-tu-differs`, `fnbyte-tus`, `fnbyte-tus-full`. Keys are an
  interface; **absence means NO-RESULT**, never 0 and never 1.
* Collected into `docs/STATUS.md` by `scripts/status.sh` as four rows, with two
  must-fail mutations in `--check`: the ratio must never render without its
  denominator, and the partition must never render without `fnbyte-partial`.
* Sources borrowed from `../objdiff` are cited in the module header of
  `fnbytes.rs`, following the pattern `crates/c2-obj/src/reloc.rs` set for the
  hand-ported `IMAGE_REL_PPC_*` table. No code was copied; the Patience
  alignment was read and deliberately **not** ported, because a row-shifted body
  is exactly the case this project must not award credit for.
