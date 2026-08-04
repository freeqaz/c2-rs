# w-factors — pre-registration

Written and committed **before any measurement of the registered quantities**.
Base `39dcfb7` (`git log -1` checked in the worktree `wt-w-factors`: master's
tip, not a stale ref).

Lane premise: `docs/ROADMAP.md` §10.19 factored Phase 7 into four predicates over
the 871 graded TUs — **A** (`.ex` segments == obj `.text` COMDATs), **B** (every
emitted symbol binds), **C** (obj section set ⊆ what the port's COFF writer can
emit), **D** (every emitted COMDAT inside the port's codegen class) — and
reported **A∧B∧C∧D = 6, exactly the observed match set**. That factorization is
now the project's front-page planning model (`docs/STATUS.md`), and **two of its
four factors are not computed by any instrument**: A and B are `gap.rs` keys, C
and D exist only in a one-off analysis by the coordinator and the w-phase7plan
lane. A planning model that no scan recomputes cannot regress-detect, and the
next person to ask "where are we" re-derives it by hand — which is exactly how
§10.14 went wrong.

So this lane makes **C and D first-class reported keys of `gap.rs`** and prints
the factorization on every scan. It builds no model and converts no TU.

## Declared bias, and its direction

**Confirmatory, and strongly so.** The numbers I am setting out to reproduce are
already published, already on the front page, and were already cross-checked
once by the coordinator against a second reader. The failure mode available to
me is therefore *not* a surprising result — it is **shaping the predicate until
it prints 84 and 6**. Nothing about "obj section set ⊆ the port's writer set"
pins down, in advance, whether `.text$yc` is "a `.text`" or its own name, whether
a section the writer emits *conditionally* counts as writable, or whether a TU
whose obj does not decode is in C or out. Each of those choices moves the number,
and I would notice a choice that moved it *away* from 84 much faster than one
that moved it toward 84.

Two mitigations, both registered before the first run:

1. **The predicates are fixed in this document, in prose, before the code is
   written** — including the writer set (a literal list of the seven `name: ".…"`
   sites in `crates/c2-core/src/coff.rs`, read as source text, not as a fitted
   set), the treatment of unreadable objs (**out** of C and out of D, fail
   closed), and the treatment of a TU that emits nothing (**in** D, vacuously —
   §10.19 says so explicitly: "6 of those emit nothing").
2. **The first measurement is the recorded one.** If the first full scan
   disagrees with 84 / 13 / 6, this document's numbers are what it is scored
   against; the prediction does not get revised and re-run.

**Second bias: toward "the coordinator was right".** A lane whose whole job is to
re-derive someone else's number is under pressure to agree, and the cheapest way
to agree is to build the predicate out of the same intermediate the original used
rather than from the port's own writer. Registered here so that a disagreement is
a *result* and not a bug to be fixed until it goes away.

## What is being built

Inside `crates/c2-harness/src/gap.rs` (the lane's seam) plus one shared reader in
`crates/c2-obj`:

* `ObjImage::section_names()` — the obj's section-name list, decoded by the
  **same** name-decoding code path `text_comdat_entries` already uses (the `/NNN`
  string-table indirection included). Extracted into a shared helper rather than
  written a second time: §10.14 is the record of what a second implementation of
  a rule the harness already owns costs. Fail-closed (`None`) on the same
  conditions.
* `emit-sec-name|<name>` — one per **distinct** name per TU, so the aggregated
  row is "objs carrying this section". The vocabulary census.
* `emit-sec-extra|<name>` — the names outside the writer set. The ladder's input.
* `emit-sec-reachable` — **factor C**, per TU.
* `emit-sec-unreadable` — objs whose section headers did not decode. A count,
  printed always, so C's population can never be short in silence.
* `emit-codegen-class` — **factor D**, per TU: `emit-in-class == emit-emitted`,
  built from the keys the harness already computes, so the codegen-class
  predicate stays `FnVerdict::in_class()` and is not re-derived here.
* A printed factorization block: |A| (both anchors), |B|, |C|, |D|, **B∧C**, and
  **A∧B∧C∧D** with the TU names when it is small, plus the greedy section ladder.

**Every new key is a pure addition.** No existing `emit-*` key, no class
predicate, and no ceiling is touched. The §10.18 trap — a variable with two
consumers changed for one of them — is the named hazard for this file, so the new
block reads `captured.ref_obj` directly and shares no variable with step 1g or
step 3.

## Registered estimates

| # | claim | point | interval | what would refute it |
|---|---|---:|---|---|
| **E1** | **the control.** Every pre-existing `emit-*` key, TU match / mismatch / vocab-gap / codegen-gap / capture-fail, both ceilings and the census, identical base → tip | **identical** | exact | **any** move ⇒ reported before anything else, and the lane's numbers are void until it is explained. A pure-addition lane that moves an existing number did something it was not supposed to |
| **E2** | distinct section names across the workload | **13** | [11, 16] | a different count ⇒ my reader and the coordinator's disagree; I name the extra/missing names *with their obj counts* and say which reader is wrong, rather than adjusting to 13 |
| **E3** | the per-name obj counts reproduce `.XBLD$W`/`.debug$S`/`.drectve` 871, `.text` 863, `.pdata` 849, `.rdata` 847, `.data` 754, `.bss` 690, `.rdata$r` 676, `.text$yd` 243, `.CRT$XCU` 126, `.text$yc` 126, `.xdata$x` 67 | **all 13 exact** | ±0 on each | any row off by ≥1 ⇒ published as a disagreement with both numbers, not silently re-stated |
| **E4** | **factor C** | **84** | [70, 95] | outside the interval ⇒ the predicate I wrote is not the predicate §10.19 measured; I say which of the two is wrong and on which TUs |
| **E5** | **factor D** | **8**, of which **6** emit nothing | [4, 15] | a materially different D ⇒ either the vacuous-truth convention differs or `emit-in-class == emit-emitted` is not the codegen-class predicate the lane used |
| **E6** | **A∧B∧C∧D** | **6** — and **exactly** the observed match set, by name | exact | any other set ⇒ either §10.19 is wrong or my implementation is; I name the TUs and say which |
| **E7** | **B∧C** | **82** | [75, 90] | outside ⇒ the plan's near-term joint ceiling is not 82 and PHASE7_PLAN §1's arithmetic needs restating |
| **E8** | the greedy ladder reproduces `.data` 109, `.rdata$r` 172, `.bss` 574, `.text$yd` 698, `.xdata$x` 745, `.CRT$XCU` 745, `.text$yc` 871 — **same order, same values** | **YES** | — | a different *order* with the same endpoint ⇒ greedy has ties and I report both; a different *endpoint* ⇒ the vocabulary is not closed by seven names and the "C is finite and short" claim on the front page needs weakening |
| **E9** | `.CRT$XCU` adds **0** at its position because it never appears without `.text$yc` (both in exactly 126 objs) | **YES** | — | a nonzero gain ⇒ the co-occurrence claim is wrong and the ladder is eight steps, not seven |
| **E10** | the ordering claim: **C < B** by a factor ≥ 3 (the whole point of §10.19 — section shape is tighter than the emit-set model) | **YES** | — | C ≥ B ⇒ the front page's "C = 84 is 4× tighter than B = 324" is refuted and `docs/STATUS.md`'s newest section is wrong |
| **E11** | A∧B∧C∧D restricted to `match` TUs is **6 of 6** — every byte-exact TU satisfies all four factors | **6/6** | exact | a matching TU outside any factor ⇒ that factor is not a **necessary** condition, which is the only thing that makes it a ceiling. This is the known-answer check |

### The known-answer checks, stated in advance

* **E11 is the one that can go red on the port's own output.** A `match` TU's obj
  *is* the port's obj, so its section set is by construction a set the writer can
  emit. A matching TU outside C means the writer set I declared is **too small**,
  and the check fires without needing anyone to re-read `coff.rs`.
* **E1** is the §10.18 guard: the class counts and every `emit-*` key are
  compared base → tip on the same `../dc3-decomp` tree state, because §10.18's
  provenance note says a number from one tree state is not comparable to one from
  another.
* `emit-sec-unreadable` has a known answer of **0** (`emit-obj-unreadable` is 0
  on this workload today, and both readers fail closed on the same conditions).
  If they disagree, one of the two walks is wrong.

## What I will conclude if the numbers do not reproduce

Explicitly, so that "reproduced" cannot be reached by adjustment:

* **E2/E3 miss** (vocabulary): the two readers disagree about what a section
  *name* is — almost certainly the `/NNN` long-name indirection or a `$`-suffix
  convention. I publish both readers' rows side by side and name the mechanism. I
  do **not** change my reader to match theirs without a stated reason that is
  independent of the target number.
* **E4 miss** (C): I dump the per-TU disagreement from the scan's JSONL and name
  the TUs. If the difference is the unreadable-obj or empty-obj convention, that
  is a *definition* disagreement and I report C under both conventions.
* **E6 miss** (A∧B∧C∧D ≠ the match set): this is the load-bearing one. §10.19's
  headline is that four independently-derived predicates reproduce the match set
  on the nose; if the reproduction fails, the factorization is **not** the
  planning model it is being used as, and I say so on the front of the report
  with the TU names on both sides of the difference.
* **E8 miss** (ladder): a different endpoint is the serious case — it would mean
  the section vocabulary is not closed by seven additions, and `STATUS.md`'s "C
  is the one factor that is finite" needs the weaker statement that it is finite
  but longer.

**A refutation is a deliverable here.** The one thing this lane must not produce
is a re-statement of 84 and 6 that agrees because it was built to.

## Scope, stated so it cannot expand

* No model, no widening, no codegen. TU match is expected to be **6** at both
  ends of this lane.
* `crates/c2-core/` and `crates/c2-il/` belong to lane **w-r1** and are not
  touched — including the writer's section list, which is *read as source text*
  and mirrored into `gap.rs` with a comment naming its home. The right home for
  that list is a published constant in `c2-core`; minting one is a
  cross-seam change and is filed for the coordinator instead of taken here.
* `docs/ROADMAP.md`, `docs/BOARD.md`, `docs/STATUS.md`, `docs/PHASE7_PLAN.md` are
  the coordinator's; this lane writes only its own rung doc and this prereg.
