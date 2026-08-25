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

> ### ✔ 2026-08-22 — **THE OWNER ASKED WHETHER THE JUDGE CAN CARRY A SLIDING SCORE. THE ANSWER IS NO, AND THIS SECTION IS WHY — IT IS NOW THE STANDING TEMPLATE FOR EVERY GRADIENT.**
> *`docs/WHITEBOX_LEVERAGE_2026-08-21.md` §5(a)/(b), on the owner's question,
> the same day as the goal re-ranking (`GOAL_DECISION_2026-08-21.md` §
> "AMENDED"). Propagated by lane `w-readdocs`; **nothing on this page is
> edited, re-scored or withdrawn**.*
>
> The three layers have different rules and conflating them is the failure this
> section exists to prevent:
>
> * **The GATE stays binary, and that is load-bearing.** A 90 %-matching obj
>   *shipped* is a wrong emit, and a wrong emit scores strictly below the
>   refusal it replaced ([`PROGRESS_METRIC.md`](PROGRESS_METRIC.md) §5.2). The
>   2,490-wrong-function measurement (board **#3363**) is what that rule
>   protects against. Nothing in the goal re-ranking relaxes it.
> * **The sliding score already exists — as an instrument, on this page.** FBM
>   is it, under §0's rule. The owner's question is answered by the wall, not
>   by moving it.
> * **§0 is the TEMPLATE.** Every gradient added after FBM adopts these five
>   properties verbatim — never in `gate.sh`, its own block under its own
>   disclaimer, namespaced keys, licenses no emit, `NO-RESULT` over an empty
>   scan. Two extensions are on the books: **S0 (blind reach)**, which would
>   extend the gradient to the 113,565 parse-refused functions this instrument
>   cannot attempt (`ROADMAP_SLICING_2026-08-21.md` §5), and **the one below,
>   which is not a plan but a shipped instrument.**
>
> ### ✔ 2026-08-25 — **THE FOURTH GRADIENT IS SHIPPED: `decode-reach-*`, and it is the first one that measures the DECODE rather than the OUTPUT.**
> *Lane `w-decodereach`, boards **#3561**–**#3566**, funded by
> `DECISIONS_2026-08-22.md` decision 13 (row 4a(i) / I1, the general decode).
> `crates/c2-harness/src/gap/decode.rs`. **Nothing on this page is edited,
> re-scored or withdrawn**; §0's five properties are adopted verbatim, as S0
> adopted them.*
>
> FBM and S0 both ask *"are these bytes c2's?"* — of the accepted population and
> of the refused one respectively. This one asks a question upstream of both:
> **how many bodies does the general decode reach, and is what it reaches
> right?** It publishes **three denominators as a containment**, never a ratio
> — `observable ⊇ reached ⊇ verified` — and its `verified` bucket is **this
> page's own `FnByte::Exact`**, passed in from FBM's walk rather than
> recomputed, so there is one producer of the judge's verdict.
>
> **Two things to carry off it before quoting any reach number:**
>
> * **REACH HAS TWO STRENGTHS AND THE TREE WAS PUBLISHING ONLY THE WEAK ONE.**
>   *Frame* reach (the walk landed on the segment tail) is **98.25 %**; *model*
>   reach (…and every operand was in the decoder's modeled vocabulary) is
>   **11.39 %**, and **5.47 % by byte**. `cli/gap.rs:679` has printed the first
>   as prose for weeks. **Quote them together or neither.**
> * **`decode-reach-verified` equals `fnbyte-exact` exactly (35,912), so it
>   carries no information by itself today** — every byte-exact body is reached,
>   i.e. reach is not the binding constraint on byte-exactness anywhere the
>   judge can speak. What does carry information is the pair beside it:
>   **23,140 of the 35,912 (64.4 %) are bodies the decode does NOT model.**
>
> [`rungs/2026-08-25-w-decodereach.md`](rungs/2026-08-25-w-decodereach.md).

> **The third gradient already exists and this page never linked it.**
> [`DIFF_STRUCTURE.md`](DIFF_STRUCTURE.md) / `crates/c2-harness/src/gap/fndiff.rs`
> extends the gradient *inside* each `fnbyte-differs` body: word-granular LCS
> alignment, per-substitution decoded-field classification under a
> re-encode-or-refuse decode rule, a pure-reordering bit, and relocation-site
> awareness. It runs unconditionally on the `differs` path
> (`gap/fnbytes.rs:2569`) and prints on **every** scan (`gap/render.rs:1295+`),
> and it obeys §0's rule in its own words: *"Nothing here reaches a numerator,
> appears in an accept/refuse path, or grades the port."* **Read it with its
> two traps**: its 3,195 population is at tree `0c8a185` where this tree reads
> `fnbyte-differs` **1,960** + `fnbyte-reloc-differs` **530**, and its own ⚠
> banner marks §3.2 and one row of §4 **REFUTED** by `w-drop3`'s relocation
> reading (#984–#989). **A planning doc proposed building it from scratch on
> 2026-08-21 at 1–2 wk** — board **#3369**, and the reason it is recorded here
> is that the same doc's proposed classification is *refuted* by this
> instrument's own output.

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
| `fnbyte-exact` | complete port body, bytes identical **and relocations identical** (§2.2) | **yes** |
| `fnbyte-differs` | complete port body, bytes differ | no |
| **`fnbyte-reloc-differs`** | **bytes identical, RELOCATIONS differ** — lane `w-relo`, §2.2 | no |
| **`fnbyte-reloc-unknown`** | bytes identical, the reference relocation table did not decode — **ungraded**, the counted residue | no |
| `fnbyte-partial` | the port selected, but the body is finished by the COFF emitter | no |
| `fnbyte-refused` | the port refuses this function — **split since lane `w-column` into `fnbyte-refused-parse` (the IL PARSER refused) and `fnbyte-refused-codegen` (the parser ACCEPTED and the emitter declined), with `fnbyte-refused-split-broken` as the printed identity** | no |
| `fnbyte-unbound` | no census row binds this symbol, or two do | no |
| `fnbyte-nobytes` | the COMDAT's raw data did not decode | no |

`fnbyte-denominator` is their sum and the identity is checked on every scan
(`fnbyte-partition-broken`, known answer 0) rather than assumed. A TU whose obj
does not decode at all contributes to **neither** numerator nor denominator, and
is counted separately (`fnbyte-obj-unreadable`).

## 2.1b THE CODEGEN COLUMN — which side of the port refused (lane `w-column`, boards #1473–#1475)

> **Board #1464:** *no field in `TuResult` says "the reader accepted this
> function and the emitter could not lower it" — that verdict does not exist to
> be read.* It is true of `TuResult`'s **named fields** and false of this
> module, and the correction matters because the number that looked like the
> codegen column was 100 % reader.

`grade_one` reaches the emitter **only** through `Ok(func)` — an `IlFunction`
the IL parser produced. So the stage that declined already says which side of
the port stopped:

| `Decline` | reached when | can it legitimately be nonzero? |
|---|---|---|
| **`Parse`** | the IL parser refused. `select_function` is **never called** — there is no `IlFunction` to call it with | **it is the whole population**: 130,575 of 178,977 at `85e180d4` |
| `OptMode` | the `.ex` optimization word is not one the port emits for | **no — 0 by construction.** `census_functions`' post-parse gate (b) raises `OPT_MODE` before a row can be `InClass`, and this stage is reached only for an `InClass` row |
| `Selector` | `select_function` refused a body **the parser accepted** | **no — 0 by construction**, and this is the verdict #1464 says does not exist. It counts exactly the population `TuResult::fn_gate_refusals` counts, by the same decision procedure, so it must be 0 for the same reason: anything here is the census over-claiming (board #139) |
| `GyShape` | the selector lowered it; the `/Gy` composition has no obj model | **YES.** `/Gy` is an argv flag and is not in the IL bundle at all, so no parser clause could express this refusal |
| `DataRef` | the data-symbol relocation halves are not locatable in the composed body | **YES.** The question is asked of bytes that do not exist until after lowering |
| **`InlinedCallee`** | the composed body emits a `REL24` against a name **this TU DEFINES**, and the port can lower that callee and its lowered body is at most `splice::INLINE_UNBOUNDED_BYTES` — so c2 expands it and emits no call here (`c2_core::comdat::fenced_inlined_callee`, lane `w-inlfence2`) | **YES, and it is the first one that actually is**: **1,004** on the 878-TU workload. The question is asked of the composed body's **relocation sites**, and only *after* mechanisms **E** and **I** have had their say |

**The invariant, stated once.** Board #139 requires every emitter refusal to have
a parser counterpart, or the census over-claims. That rule binds only on
refusals **the parser is able to express** — i.e. on refusals that are a function
of the IL body alone. `GyShape`, `DataRef` and `InlinedCallee` are not, so they
are legitimate and permanent; `OptMode` and `Selector` are, so they must be zero
and their zero is an **alarm that did not fire**, never a measurement that the
codegen distance is zero.

> **Until 2026-08-09 all three of the legitimate stages read 0, so the rule's
> exceptions had never been tested and the totals built on them looked like
> alarms.** `InlinedCallee` is the first non-zero one, and two published numbers
> moved with it: `fnbyte-refused-codegen` **0 → 1,004** and
> `fnbyte-census-disagree` **0 → 1,004**.
>
> **Neither was weakened.** `fnbyte-census-disagree` is now published beside
> **`fnbyte-census-disagree-expressible`** — the half board #139's rule reaches,
> whose target is still **0** and which reads 0 — plus one
> `fnbyte-census-disagree-<stage>` row per post-lowering stage. The residue is
> not an accounting convenience: it is the measured size of the emitted census's
> **over-claim**, and it was that size before the fence existed too — as bodies
> the port emitted *wrongly* instead of refusing. Lane `w-inlfence2`, board
> **#2157**; `rungs/2026-08-09-w-inlfence2.md` §8.
>
> A refusal and a wrong emit score the **same zero** under FBM, so no credit
> moved when 1,004 functions crossed between them. What moved is whether the
> port's claim about them was true.

**Consequently the useful codegen reading is not the refusal column at all.** It
is `fnbyte-differs` + `fnbyte-reloc-differs` — *the reader accepted, the emitter
lowered, and the judge says the bytes are wrong* — which reads **2,972** on the
dc3 workload while every refusal stage reads 0. `GapReport::frontier_codegen`
publishes that reading per frontier TU beside the population it cannot see
behind (board #1474).

## 2.2 RELOC-EQ — what `exact` means since 2026-08-08 (lane `w-relo`, board #884)

> **A `.text` COMDAT's raw bytes do not contain its relocations.** Two bodies
> that branch to two *different* functions are byte-identical here and are not
> the same function. `exact` therefore means bytes **and** relocations, and the
> gap trap 6 measured at **4,664** is now graded rather than counted.

For a function whose **bytes are already exact**, let `R_ref` be the reference
COMDAT's relocation records **in disk order** and `R_port` the port's own plan
in the order the `/Gy` writer emits it. They **MATCH** iff the two are equal as
**sequences**: same length, same offset, same **whole packed** type word, same
target.

| decision | and why it could have gone the other way |
|---|---|
| a **sequence**, nothing sorted here | c2's disk order against the emitter's own order; two sets equal as multisets and swapped in order produce different obj bytes |
| the `ty` word compared **whole** | the high byte carries `NEG`/`BRTAKEN`/`BRNTAKEN`/`TOCDEFN`; `REL24\|BRTAKEN` = `0x0206` is a different relocation from `REL24` |
| the target by **NAME**, never by index | symbol indices differ across objs legitimately, and the port has no obj here at all — it has names. #918's rule, one level along: the binding is `FnCensus::emit_name` |
| three target kinds as a typed enum | `Section(".rdata")` can never equal `Symbol(".rdata")`; a `PAIR`'s index field is a **displacement**, not an index (rev 6.0) |
| **fail closed** | an undecodable table, an index out of range or landing on an aux slot refuses the whole obj, and every byte-exact function in it lands in `fnbyte-reloc-unknown` — **not** a credit. Crediting an ungraded body is the blind-instrument defect this closes |

**ONE LOCATOR.** `crates/c2-core/src/coff/writer.rs`'s `/Gy` branch used to build
its relocation list inline. It now calls `comdat::text_reloc_plan`, and so does
FBM — verbatim board #880's argument for the body composition, one field along:
*a second copy in the harness could drift from the emitter, and an alarm that is
green about relocations the port does not emit is worse than the blind one it
replaced.*

**The old count stays derivable.** `fnbyte-exact-bytes` republishes the previous
`fnbyte-exact` predicate and read **36,847** at both ends of the widening —
`exact + reloc-differs + reloc-unknown`, to the digit. `fnbyte-reloc-graded +
fnbyte-reloc-unknown = fnbyte-exact-bytes` is a second positive identity with its
own broken-counter (`fnbyte-reloc-partition-broken`, known answer 0), because a
green control is a statement about the population it ran over (`STATUS.md`
trap 0) and that population has to be printed.

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

### 3.1 The one lever — CLOSED 2026-08-06, and it was pulled the safe way

> **⚠ This section stood as written below until lane `w-fnbyte` (board #322,
> #876–#885). Read the correction first: the reconstruction IS done now, it is
> done in `c2-core` rather than in the harness, and it moved `fnbyte-differs`
> from 0 to 4,711.**

**What it said.** The honest gaming vector is not in the port — it is in the
**instrument**. `Selected::{Tail, Framed, Seq, CondPair}` and a `Float` with
pooled constants hand back a body whose remaining words the COFF emitter
finishes, because those words encode their own `.text` offset. The harness
*could* append them itself, and 9,374 functions would move from `fnbyte-partial`
into `fnbyte-exact` with zero change to the port. So the reconstruction is not
done here; closing the class needs a per-function entry point in `c2-core`, the
crate that owns the fact.

**What was wrong with it, and what is true.** The decline reason is a statement
about the **packed** emitter. FBM's denominator is the **`/Gy` COMDAT**
population — `/O1` and `/O2` imply `/Gy`, and the whole workload is `/O1` — and
under function-level linking **every function starts at offset 0 of its own
section**. The `.text` offset the harness "could not know" is a constant there,
and `PortC2::build`'s `/Gy` branch had composed all four bodies completely since
W-UNW-1. It composed them **inline**, so nothing but the whole-TU emitter could
reach the code. Reachability was the obstacle; reconstruction never was.

`crates/c2-core/src/comdat.rs` is that composition, lifted verbatim, and
**`fnbytes::complete_body` calls it — never a copy.** That is the load-bearing
part of the closure: a second implementation in the harness could drift from the
emitter, and an alarm that is green about bytes the port does not emit is worse
than the blind one it replaced.

**And the projection this section implied was wrong by half.** Board #322
estimated `FBM 0.16251 → 0.21488` on the assumption that the 9,374 were exact.
Measured: **4,664 exact, 4,711 differ**, `FBM 0.16654 → 0.19259`. See §6.1.

**What is still `partial`, and why it is not this class.** A `Float` with pooled
constants, tagged `float-const`, plus any body whose `/Gy` composition the port
itself declines (tagged `<shape>-compose`). Those are the **port's** refusals,
not the harness's. On the dc3 workload the count is **0**.

FBM is still a **floor**, for §7.1's reason and not for this one: a `.text`
COMDAT's bytes are a subset of the obj. `fnbyte-partial` is still printed beside
the ratio and is still a *required* part of the `STATUS.md` row, enforced by a
must-fail mutation in `scripts/status.sh --check`; it now prints
`partial by shape: NONE` with the denominator beside it rather than vanishing,
because an absent line is how absence reads as success.

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
port's output is therefore defined on **38,458 of 178,977 emitted functions
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

> ### ⚠ 2026-08-06 — the table in §6 below is the reading of a BLIND instrument. §6.1 is current.
>
> Every figure in §6 was taken while FBM declined to grade 9,375 emitted
> functions. It is kept because the *shape* of the reading is still instructive
> and because the sentence it licensed — quoted and corrected in §6.1 — was on
> this page for a day.

### 6.3 CURRENT — after RELOC-EQ (lane `w-relo`, 2026-08-08)

Workload scan, 878 TUs, `capture-fail 7 / graded 871`, off master `22816a5`.
Both ends of the same lane; **the port's codegen is untouched** — the only thing
that changed is what the instrument grades. **84 of the 89** `gap-metric` lines
are byte-identical at both ends, 20 keys are new, and nothing outside the
`fnbyte-` namespace moved.

| | before | after |
|---|---:|---:|
| **FBM** | 0.20589 | **0.20108** |
| **exact** | 35,986 | **35,125** — *shrank, and that is the finding* |
| **NEW `fnbyte-reloc-differs`** | — | **861** |
| `-differs-target` · `-count` · `-offset` · `-type` · `-section-target` | — | **861 · 0 · 0 · 0 · 0** |
| **NEW `fnbyte-exact-bytes`** (the OLD `exact`) | — | **35,986** — recovered to the digit |
| **NEW `fnbyte-reloc-graded` / `-reloc-unknown`** | — | **35,986 / 0** |
| `fnbyte-exact-relocated` | 4,664 (ungraded) | **3,803 (graded and credited)**; `-reloc-graded-relocated` **4,664** |
| `tail` exact / reloc-differs | 5,567 / — | **4,722 / 845** |
| `seq` exact / reloc-differs | 609 / — | **593 / 16** |
| `cond-pair` · `plain` · `float` exact | 4 · 29,352 · 450 | **unchanged — none of these relocate wrongly** |
| differs · whole-TU · partial · refused · unbound · denominator | 3,195 · 2 · 0 · 130,579 · 9,217 · 178,977 | **unchanged** |
| controls: partition-broken · reloc-reach-broken · match-TU differs · **match-TU RELOC-differs** · census disagree | 0 · — · 0 · — · 0 | **0 · 0 · 0 · 0 · 0** |
| residue: `-reloc-table-unreadable` · `-reloc-index-desync` | — | **0 · 0** |
| **NEW** `fnbyte-reloc-vs-calltarget-{both, reloc-only, calltarget-only}` | — | **861 · 0 · 0** |
| `fnbyte-calltarget-*` (lane `w-drop3`'s eight keys) | graded 39,177 · **-disagree-exact 861** | **unchanged to the digit** |

**THE NUMBER IS INDEPENDENTLY REPLICATED.** Lane `w-drop3` reached **861** on the
same corpus from a *different* reader — `REL24` targets by name
(`ObjImage::text_comdat_call_targets`, board #986) against this one's every-record
compare (`ObjImage::text_comdat_relocs`) — with different port-side sources. Two
equal totals are not evidence that two readers agree, so the agreement is
published **per function**: `-both` 861, `-calltarget-only` 0 (known answer 0 — a
`REL24` target disagreement *is* a record disagreement), `-reloc-only` 0
(measured, not predicted — a data-symbol target or a type or an offset would
land there legitimately). See `rungs/2026-08-08-w-relo.md` §4.3, which also
records the erasure that merge nearly caused: `w-drop3`'s walk guards on
`FnByte::Exact`, whose meaning **this section narrowed**, and left alone it would
have printed `disagree-exact 0` with no test red and no conflict marker.

**`exact` shrank by 861 and `FBM` fell by 0.00481. That is the
instrument-widening motion and not a regression** — the same shape w-fnbyte
declared when `fnbyte-differs` went 0 → 4,711 (§7.2). `mismatch` is **0 at both
ends**: `IlBundle::functions()` refuses every TU carrying one of the 861, so no
obj was ever emitted with a wrong target. What is wrong is the **census's
claim**, and it was wrong before this scan could say so.

**Every one of the 861 is a TARGET disagreement.** Not one is a count, an
offset, a type word or a section-symbol target — so the port emits the right
*number* of relocations, at the right *offsets*, of the right *types*, naming
the wrong *function*. The families (`fnbyte-reloc-fam|…`):

| n | family | reading |
|---:|---|---|
| **529** | `tail\|target\|local->local\|blocked` | the port's own target is a body the parser refused, so whether it calls what c2 named is **not answerable here** — priced by production: `expr-call-in-expr-recv-field-off0-then-chain-bind-whole` 348 · `…-intrinsic-this-adjust-then-chain-bind-whole` 103 · `expr-ternary` 50 · two more |
| **169** | `tail\|target\|local->local\|unrelated` | edges existed and none reached c2's target |
| **73 + 69** | `tail\|target\|local->local\|chain2` / `chain1` | **`s12`'s mechanism, proven on the workload**: c2 named what the port's own callee calls, one or two steps along |
| **16** | `seq\|target\|local->extern\|chain1` | the same, with the reference naming an external |
| 6 | `comdat-only` on one side or both | the name has a COMDAT in c2's obj and no census row |

**`blocked` and `unrelated` are different answers, and the first read of this
walk merged them.** Written as "no path found", the largest family printed
`unrelated 697` and named nothing — w-seq §2's `refused:blocked` defect exactly.
A walk that could not expand a single edge now answers `blocked` and names the
production that blocked it, which is the work list.

**Largest single signature: 51 functions**, port `b ??1?$list@…@QAA@XZ` against
c2's `b ?clear@?$_List_base@…@QAAXXZ` — an STL list destructor whose body c2
expanded, leaving the branch pointing at what the destructor itself tail-calls.
`fnbyte-reloc-witnesses` is **277** distinct witness keys over the 861 pairs
(one key per `(shape, kind, counts, index, target pair, symbol)`; two TUs
emitting the same template instantiation share a key, the same aggregation
`fnbyte-differs-witnesses 1333` over 3,195 uses).

### 6.2 SUPERSEDED — after mechanism E (lane `w-empty`, 2026-08-07)

Workload scan, 878 TUs, `capture-fail 7 / graded 871`, off master `9827bcf`.
Both ends of the same lane; `IlBundle::functions()` is untouched, so the only
thing that changed is what the port composes for one `Selected` shape.

| | before | after |
|---|---:|---:|
| **FBM** | 0.19259 | **0.20026** |
| exact | 34,466 | **35,839** |
| **differs** | **4,711** | **3,338** |
| `tail` exact / differs | 4,051 / 3,047 | **5,424 / 1,674** |
| differs witnesses (distinct symbols) | 1,950 | **1,405** |
| whole-TU · partial · refused · unbound · denominator | 2 · 0 · 130,579 · 9,217 · 178,977 | **unchanged** |
| `fnbyte-exact-relocated` | 4,664 | **4,664** — the new bodies carry no relocation |
| **NEW** `fnbyte-elided` / `fnbyte-elided-exact` | — | **1,373 / 1,373** |
| **NEW** `fnbyte-name-disagree` | — | **74,955** |
| controls: partition-broken · match-TU differs · census disagree | 0 · 0 · 0 | **0 · 0 · 0** |

**1,373 functions moved `differs → exact` and ZERO moved the other way**, checked
per symbol off the scan's own witness keys rather than by subtracting two totals
(`work/w-empty/wdiff.py`) — the aggregate cannot distinguish `+1,400 / −27` from
`+1,373 / −0`, and on this lane's first attempt it was `+0 / −14`.

**Two of the new keys are controls and not progress.** `fnbyte-elided-exact`
equalling `fnbyte-elided` is the claim that every body the elision produced is
c2's; printing the pair means a future divergence is visible instead of being
arithmetic. **`fnbyte-name-disagree` is a warning to the next lane**: a census
row carries `IlFunction::mangled_name` (paired *positionally* over `.ex`
segments) and `FnCensus::emit_name` (the per-record binding this walk uses), and
they differ on **74,955** rows. A name-matched fact read through the first is
attached to the wrong function — board #918, and the reason `differs` briefly
went the wrong way. `rungs/2026-08-07-w-empty.md` §4.

### 6.1 SUPERSEDED FIGURES — after board #322 (lane `w-fnbyte`, 2026-08-06)

Workload scan, 878 TUs, `capture-fail 7 / graded 871`, tree `840ab02`, off master
`33a1867`. Both ends of the same lane, so the only thing that changed is the
instrument.

| | before | after |
|---|---:|---:|
| **FBM** | 0.16654 | **0.19259** |
| exact (per-function route) | 29,802 | **34,466** |
| whole-TU (oracle-certified) | 5 | **2** — three of the five are now credited by the route itself |
| **differs** | **0** | **4,711** |
| partial (FBM's under-report) | 9,375 | **0** |
| refused · unbound · denominator | 130,579 · 9,217 · 178,977 | **unchanged** |
| `fnbyte-exact-relocated` (trap 6) | 0 | **4,664** |
| controls: partition-broken · match-TU differs · census disagree | 0 · 0 · 0 | **0 · 0 · 0** |

**Per shape:** `tail` 4,051 exact / 3,047 differ · `seq` 609 / 1,541 ·
**`framed` 0 / 123** · `cond-pair` 4 / 0 · `plain` 29,352 / 0 · `float` 450 / 0.

**THE CORRECTION THIS PAGE OWES.** §6's last paragraph read *"the `differs`
column being 0 is the finding: **not one of the census's emitted in-class claims
is wrong where it can be checked**."* That sentence was true of the 75.6 % of
the population the instrument could see and **false of the rest**. Restated
against the whole emitted in-class population: of **39,177** claims, **34,466
are confirmed byte-exact, 4,711 are confirmed wrong, and 0 are unexamined.**

`mismatch` is **0 at both ends and stays 0** — `IlBundle::functions()` refuses
every TU carrying one of the 4,711, so no obj was ever emitted wrong. What is
wrong is the **census's claim**, which is the PROGRESS MASS's `f` numerator; and
because every one of the 4,711 is already accepted by the *per-function* gate, a
TU-level widening that admits any TU containing one ships a wrong obj. Board
**#876**–**#879**; taxonomy, hand-verified cause and the 61 signatures in
`rungs/2026-08-06-w-fnbyte.md` §5.

### 6.0 SUPERSEDED — the blind reading

Workload scan, 878 TUs, `capture-fail 7 / graded 871`, tree `64f4754`
(rebased onto `463796d`), workload `fe1b5b3`:

| | |
|---|---|
| **FBM** | **(29,084 + 2) / 178,977 = 0.16251** |
| exact (per-function route) | 29,084 (16.25 %) |
| whole-TU (oracle-certified) | 2 |
| **differs** | **0** |
| partial (FBM's under-report) | 9,374 (5.24 %) — `tail` 7,098 · `seq` 2,150 · `framed` 123 · `cond-pair` 3 |
| refused | 131,292 (73.36 %) |
| unbound | 9,217 (5.15 %) |
| no-bytes / obj-unreadable / partition breaks | 0 / 0 / 0 |
| controls: match-TU differs · census/gate disagree on emitted | 0 · 0 |
| **credited functions carrying a relocation** (§7.7) | **0** |
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

   **2026-08-06: #322 landed and the jump was +4,664, not +9,374.** It is an
   instrument change and is reported as one (§6.1) — but the trap as written
   anticipated only the *good* half. The other 4,711 became `fnbyte-differs`,
   which is not a smaller gain, it is an **alarm**. The general form is worth
   keeping: *when an instrument widens, the new population splits, and a lane
   that budgeted only for the credit will read the split as a shortfall.*
3. **`unbound` is an instrument limit wearing a port's clothes.** 9,217 emitted
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
6. **"The bytes match" is weaker than "the function matches" — measured, and
   the gap is currently EMPTY.** A `.text` COMDAT's raw data does not contain its
   relocations, so two bodies that load the address of two *different* globals
   are byte-identical here and differ in the obj. FBM's `exact` bucket therefore
   credits bodies whose relocation targets it never checked. This was raised as a
   constructed counterexample against the metric and then measured rather than
   argued: **`fnbyte-exact-relocated` = 0.** Not one of the 29,084 credited
   functions carries a relocation in c2's obj, so today `exact` *is* a full
   function-identity claim. **The counterexample is sound and its instance count
   is zero**, which is exactly the state that changes without warning — the first
   accepted shape that relocates (an address leaf over an external, a pooled
   float under `/Gy`) makes this bucket nonzero, and the number is printed on
   every scan so the change is loud rather than silent.

   > **⚠ 2026-08-06 — IT CHANGED. `fnbyte-exact-relocated` is 4,664**, and that
   > is **every single one** of the functions board #322's closure newly
   > credited (`tail` 4,051 + `seq` 609 + `cond-pair` 4). A tail call's `b` is a
   > REL24 by construction, so the whole newly-graded class relocates. `exact` is
   > therefore **no longer a full function-identity claim for 4,664 of its
   > 34,466**: two bodies calling two *different* functions are byte-identical
   > here. The counter was built for this moment and it printed the number the
   > day it happened, which is the whole point of measuring a caveat instead of
   > writing one. Board **#884**. Closing it means comparing relocation records
   > against the census's callee names — a `c2-obj` rung.
   >
   > **⚠ 2026-08-08 — THE GAP WAS EXERCISED, and it decided a rung.** Lane
   > `w-splice` shipped a mechanism that *replaces a caller's relocations with
   > its callee's* (`crates/c2-core/src/splice.rs`), so for the 723 functions it
   > moves, "the bytes are right" and "the function is right" stopped being the
   > same question. A per-symbol check against the reference obj's own
   > relocation records — `ObjImage::text_comdat_relocs`, the `c2-obj` reader
   > this paragraph asks for, board **#1025** — found **150** wrong targets in
   > the first shipped version of that rule, **77** in the second and **1** in
   > the third. **FBM scored every one of them `exact`.** Each round changed the
   > rule (#1020, #1022) and the shipped tip is 723 of 723 verified with 0
   > disagreements, `fnbyte-exact-relocated` unmoved at 4,664.
   >
   > Two things follow for this document. **The trap is not theoretical any
   > more** — it has a measured instance count and a rung that would have
   > shipped green without it. And **the reader is built**: closing #884 for the
   > whole 4,664 is now a matter of running that comparison over every credited
   > function rather than only over the ones one mechanism moves, which is what
   > a concurrent lane is doing in `gap/fnbytes.rs`. **That lane is `w-relo`
   > and it landed — the block below is its result.**

   > ### ✅ 2026-08-08 — CLOSED (lane `w-relo`, §2.2, §6.3). **861 of the 4,664
   > were WRONG**, all of them a target-symbol disagreement, and `exact` is a
   > full function-identity claim again over the population §6.3 states.
   >
   > `fnbyte-exact-relocated` is **retired into a graded number**: it still
   > prints, and it is now the *denominator of a verdict* (3,803 credited
   > functions whose every relocation record was compared) rather than the size
   > of a blind spot. The constructed counterexample had instance count 0 for one
   > day, 4,664 for two, and **861 confirmed wrong** on the third — which is what
   > it looks like when a caveat is measured instead of written.
   >
   > **The two readings compose rather than compete.** `w-splice`'s check ran
   > over the 723 functions its own rule moves and found them clean; this one
   > runs over **every** credited function and finds 861 wrong elsewhere. The
   > 723 are re-graded independently by this instrument in
   > `rungs/2026-08-08-w-relo.md` §4.4.
7. **A `mismatch` TU's functions are still counted by the per-function route.**
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
  `fnbyte-match-tu-differs`, `fnbyte-tus`, `fnbyte-tus-full`,
  `fnbyte-elided`, `fnbyte-elided-exact`, `fnbyte-name-disagree`, and — since
  lane `w-relo` — `fnbyte-reloc-differs`, `fnbyte-reloc-differs-{count,offset,
  type,target,section-target}`, `fnbyte-reloc-unknown`, `fnbyte-reloc-graded`,
  `fnbyte-reloc-graded-relocated`, `fnbyte-exact-bytes`,
  `fnbyte-reloc-partition-broken`, `fnbyte-match-tu-reloc-differs`,
  `fnbyte-reloc-table-unreadable`, `fnbyte-reloc-index-desync`,
  `fnbyte-reloc-witnesses`; and — since lane `w-column` — `fnbyte-decline-{parse,opt-mode,selector,gy-shape,data-ref}`, `fnbyte-refused-{parse,codegen}`, `fnbyte-refused-split-broken`; and — since lane `w-inlfence2` — `fnbyte-decline-inlined-callee`, `fnbyte-census-disagree-expressible` and one `fnbyte-census-disagree-<stage>` per stage that fired, `frontier-codegen-{denominator,exact,wrong,refused,reader,ungraded,measured,partition-broken}`. Keys are an interface; **absence means NO-RESULT**,
  never 0 and never 1.
* Collected into `docs/STATUS.md` by `scripts/status.sh` as four rows, with
  **three** must-fail mutations in `--check`: the ratio must never render
  without its denominator, the partition must never render without
  `fnbyte-partial`, and — since `w-relo` — the relocation verdict must never
  render without `fnbyte-reloc-unknown`, its **ungraded residue**.
  `reloc-differs 0` beside a silent residue reads as "every relocation checks
  out" when what may have happened is that none was graded: objdiff's
  `total_code == 0 → 100.0`, one field along.
* Sources borrowed from `../objdiff` are cited in the module header of
  `fnbytes.rs`, following the pattern `crates/c2-obj/src/reloc.rs` set for the
  hand-ported `IMAGE_REL_PPC_*` table. No code was copied; the Patience
  alignment was read and deliberately **not** ported, because a row-shifted body
  is exactly the case this project must not award credit for.
