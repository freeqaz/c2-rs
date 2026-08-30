# w-emitprice — one of the five is an emit change, three are one missing reader, and the ranking is an artifact

    Tag:       w-emitprice
    Slug:      w-emitprice
    Date:      2026-08-29
    Kind:      characterization
    Outcome:   built
    Fixtures:  none — characterization: what do C7, C9, C10, C11 and C12 cost,
               two-sided — what does adopting each buy, and what does the
               standing refusal cost today
    Census:    707728/2417794 → 707728/2417794 (29.27% → 29.27%), +0
    Reach:     PREDICTED 0, REALIZED 0 — registered in prereg before the image
               was opened. This lane changed no compiled file, so the byte
               delta is 0 BY CONSTRUCTION and is the floor, not the grade.
    Record:    `docs/whitebox/WB_EMITPRICE_FINDINGS.md` (the five prices with
               their derivations); prereg `work/w-emitprice/PREREG.md`,
               committed at `314f2696f` BEFORE the image was opened; four
               instruments with their outputs beside them —
               `work/w-emitprice/f20.py` (who writes `[sym+0x20]`),
               `c7_price.py` (C7 re-read over 168 committed cells),
               `attr_twins.py` (the four-cell `ATTR` family),
               `c9_bit23.py` (the favour-speed bit's value)

Charter: `docs/WAVE21_BRIEF_2026-08-29.md` §2 L2. Dispatched at master
`1d52f8902`. Board **#3856**–**#3862**.

---

## What it admits, and what it refuses

**Admits — the price of each of the five, in the units the goal is written in.**

| clause | adopting it buys | the refusal costs today | verdict |
|---|---|---|---|
| **C7** | **negative**: 3 wrong emits at `/O1` raw, **24** through the naive 4 B/word unit; the only translation scoring 0/0 at `/O1` has a converter fitted to the bracket the incumbents were fitted to, so the value carries no new information | **0 byte-exact functions** on the workload (`comdat.rs`'s own `1,074 / 0`), and the incumbent pair is wrong-emit-free on all 82 `/O1` cells | **DECLINE**, priced negative |
| **C9** | **0** bytes — byte-neutral *by construction*, because bit 23 of the option word is **0 at `/O1`**; high characterization value | **0** — the arm cannot fire at `/O1`, and `/Ox` TUs are refused at the gate | **ADOPTABLE**, blocker `none` |
| **C10** | **0** bytes, three independent reasons; high characterization value | **0** — the population is refused upstream, `[O]` | **RECLASSIFY** |
| **C11** | **UNKNOWN** — refuse-side, so ≤ 0 in match/fnbyte and ≥ 0 in warranty, over a population nothing can measure | **unmeasurable** | **RECLASSIFY** |
| **C12** | **UNKNOWN**, bounded at n = 6 observed `ATTR` values, 0 carrying either bit | **unmeasurable** | **RECLASSIFY** |

**The one-line result: exactly ONE of the five is an emit change, and its price
is negative.** The other four are not emit changes at all — one is adoptable
today and three are blocked on a single missing reader.

**Refuses.** Five things this lane could have claimed and does not:

1. **A ranking.** Its own registered check A1 fired: four rows price at zero, so
   the deliverable is a **partition into three classes** and no order is
   asserted over the tie. `#3505` is seven for seven.
2. **A number for C11 or C12.** The honest price is *"unknown until a reader
   exists that exercises them"* and it is published in those words, under the
   brief's own licence. n = 6 with 0 hits is a bound and is labelled as one.
3. **That the IL value of `[sym+0x20]` survives to the legality test.** Four
   register-sourced writes, 60 advanced-base sites and the block-copy class
   remain undecidable by this lane's instrument, and — unlike the `+0x50` case
   `w-instrcount` closed — block copies are **not** empty at this displacement.
   The claim made is *"no enumerated pass sets C11's four bits"*, which is
   weaker and is what the evidence supports.
4. **Any edit to `CLAUSES.tsv` or `P_INLINE.md`.** `w-budget` owns both this
   wave (`#3814`). Five proposed `blocker`/`read` corrections are recorded on
   this lane's own page **with their evidence**, exactly as `w-instrcount` did.
5. **A recommendation to spend.** This lane produces a price; the decision is
   the coordinator's.

**And it corrects its own charter.** The brief says of the five that *"they are
`absent` but derivable; what stops them is that each is an emit change needing a
two-sided price"*. That is right about C7 and **wrong about the other four**,
and finding out which is the whole content of the pricing.

## Estimate vs outcome

Registered before any measurement (`PREREG` §3), scored honestly: **3 of 5
confirmed, 2 real misses.**

* **P1 (C7 negative) CONFIRMED**, and refined — the errors are one-directional
  per seam and it is their *consequence* that flips.
* **P2 (C9 unknown, blocked on a read) MISS.** The read was already taken; what
  was missing was joining `0x10b8238d`'s formula to the port's own
  `OPT_WORD_*` constants. C9 is settled and adoptable.
* **P3 (C10 misclassified, byte-neutral by construction) CONFIRMED**, by three
  mechanisms rather than the one predicted.
* **P4 (C11 not derivable) MISS.** `[sym+0x20]` is IL-borne, `0x10b9be68`, from
  the same varint reader as `ATTR`.
* **P5 (C12 warranty-shaped) CONFIRMED, amended** — its population is
  unmeasurable, and C11/C12 turned out to be one predicate.

**Both misses predicted a missing link that was present, and both corrections
made the row CHEAPER.** The bias is toward over-estimating the distance to a
fact this repo has already read — `#3846`'s direction, one page over.

**A5 does not fire**: a 5-of-5 would have been the flag to re-examine.

> **Before pricing this as codegen, run `CEILING.md` §11.4.** Not applicable —
> this is a characterization lane with predicted reach 0. It is recorded because
> the template asks and a blank row reads as an oversight.

## The axis on which this lane could have FAILED

A characterization lane owes the same thing a construct rung owes: a way to come
out wrong. Three, and each was live:

1. **The five prices could have been five zeros with no structure** — a "nobody
   can price these" result. They were not: C7 came out negative with a number,
   C9 came out adoptable, and the remaining three collapsed onto one named
   missing reader.
2. **The ranking could have been published as a ranking.** A1 was registered in
   advance precisely so that a tie would be *reported* rather than ordered, and
   it fired.
3. **`attr_twins.py`'s controls could have gone unwatched.** They did not, and
   **one went RED for real**: the first framing walk anchored on *"the first
   `0x80`"*, stopped inside the fixed run, and read an identical `0x5480` for
   all four cells. All four controls fired and the script refused to print a
   verdict. That is `#3336` working, and it is recorded rather than tidied.

## Gate evidence

| lane | result |
|---|---|
| `C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release --no-fail-fast` | **62 targets · 2,014 passed · 1 failed.** The one failure is `rung_index_is_generated_and_current`, which brief §4 states **WILL** be red at every lane tip and is not this lane's to fix (`INDEX.md` is regenerated at the merge). Every other target is green — **and the attribution was measured, not assumed**; see below. |
| `c2rs selftest` | **PASS on all 214 printed rows, 0 FAIL, 0 SKIP** — verified **NOT** `SKIP: toolchain absent` **before any measurement was taken** (brief §5) |
| `scripts/gate.sh --jobs 16 --require-graded` | **unqualified `GATE: PASS`.** `lanes: 18 in the registry — 18 PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT`; **7,056 fixture-verdicts**; sweep **19,542 of 19,638** graded, **0 mismatch**; cross **91,900 of 92,288** cells graded, **0 mismatch**; `hatch-red` 14/14, `ladder-red` 5/5; debug lane 18/18 at **0 panics** |
| 878-TU workload scan | not re-run: this lane changed no compiled file, so every input to it is byte-identical to the base tree's |
| fixtures, `c2rs census` | +0 — no fixture added, no census cell moved |

**The one red is ATTRIBUTED by a control, not by the brief's say-so.** "Expected
red" is the shape of excuse that hides a second failure behind a first, so it
was measured: with `docs/rungs/2026-08-29-w-emitprice.md` moved aside, the
target reads **`ok. 4 passed; 0 failed`**; with it restored, **`3 passed;
1 failed`**. So the red is *exactly and only* this lane's missing `INDEX.md`
row, the other three assertions of that target — including
`rung_docs_claim_their_tag_slug_and_fixtures_exactly_once`, which is the one
that validates this rung's own header block — are green **over this rung**, and
nothing else is hiding underneath. The file was restored and `touch`ed (brief
§5's mtime trap).

**`#3835`'s hazard did not bite this run, and it was checked rather than
assumed.** The gate prints its graded-tree hash twice and nothing compares
them (that is `w-gatehash`'s commission this wave). Both printings on this run
read **`c1eb31f530bd`** — line 16 and line 112 of the transcript — so the tree
that was graded at the start is the tree that was graded at the end. This lane
committed only under `docs/` and `work/`, and the hash covers
`crates fixtures scripts`, so the two ends agreeing is expected rather than
lucky; it is recorded because the check costs one `grep` and the alternative is
a transcript that looks authoritative over two different trees.

**Byte delta 0 and reach 0 are BY CONSTRUCTION**, not by measurement: `git diff
--stat 1d52f8902..HEAD -- crates/` is empty. `rung_index_is_generated_and_current`
is RED at this tip and that is expected — `INDEX.md` is regenerated at the merge
(brief §4).

## Found and not taken

Ranked, sized, frame axis applied. Full detail in
[`../whitebox/WB_EMITPRICE_FINDINGS.md`](../whitebox/WB_EMITPRICE_FINDINGS.md) §8.

1. **The `.gl` symbol-record decoder** — the single missing link behind C10,
   C11 and C12. Both ends are now known: `ATTR`'s continuation
   (`0x10c1f91b`, two-or-four bytes, bit 15 the flag, `[O]` on four cells) and
   the `+0x40`/`+0x20` region that precedes `gl_function_attrs`' framing anchor.
   **It is a reader, not an emit change**, and buying it for C10 alone leaves
   two-thirds of its value unspent.
2. **C9's adoption itself** — the cheapest row on the page, byte-neutral by
   construction, with a real input already in `crates/`.
3. **`FUN_10b82338` is the option word's whole fan-out** — five globals from one
   per-function word, of which this repo has named one.
4. **`[sym+0x4c]` bit 12 gates a sub-record decode** (`0x10b9bf99`) and is set
   on all four cells, so any decoder that ignores it desynchronises.
5. **`[sym+0x20]` bit 9 gates an extra field** (`0x10b9be6b`) — *not* C12's
   `0x200`, which is on `+0x4c`. Same bit number, different field.
6. **`and eax,0xfffffffb` at `0x10b9bf75`** force-clears `ATTR` bit 2 on this
   arm and, per `FUNCS.tsv`'s own label, not on another. Two arms disagreeing
   about a bit of a field three clauses test.
