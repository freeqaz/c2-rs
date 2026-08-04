# Pre-registration — lane `w-bss2`, the §5-on-real-objs rung

Committed **before** the grading run. Lane `w-bss` scored 6 right / 4 wrong on its
own prereg and observed that the two wrong predictions with **no registered
alternative** were exactly the two that would have broken a writer silently. So
every prediction below carries a named rival, and each one carries a decline
floor.

Everything is graded at the workload's flags,
`/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc` + the project's `/I` set. The
judge is the real `c2.dll` under wibo; no expected obj is constructed anywhere.

## What is being graded

`docs/OBJ_DATA_BSS_SHAPE.md` §5 — the walk order (A1, A2) and the allocator (A3)
— was fitted and scored on a **designed probe grid only**. §8.8 names that as the
lane's largest gap. The grading set here is the **real workload objs**: the 871
that compile, of which **158 carry at least one non-COMDAT `.data`/`.bss` section
with two or more defined symbols** (121 such `.bss`, 70 such `.data`). Those 191
sections are the held-out set — §5 was fitted on none of them, and none of their
sources were looked at before this file was committed.

Object size, alignment, linkage and declaration order come from the IL `.gl`
records (frame located this session, committed as mechanism in the preceding
commit); offsets and section sizes come from `work/w-bss/census/sections.jsonl`,
which was extracted from real objs by the previous lane.

---

## R0 — the parser control, which gates everything else

> **R0.** For every **COMDAT** `.data`/`.bss` section in the workload whose one
> defined symbol also has a `.gl` data record, the size read out of the `.gl`
> equals the section's `SizeOfRawData`.
>
> Registered: **≥ 95 %** exact.
>
> *Rival R0′*: the `.gl` size field is a *type* size, not the object's, and
> disagrees systematically on arrays or on class types with padding.
>
> **Decline floor: < 90 % ⇒ the parser is not trustworthy and R1/R2 are not
> reported as scores at all**, only as "not measurable this lane".

## R1 — the `.bss` allocator on real objs

> **R1.** For each of the 121 non-COMDAT multi-symbol `.bss` sections: walk the
> objects in **`.gl` file order** (A1, eager) and allocate with **A3**
> (`align = max(t, 1 if n<2 else 4 if n<64 else 8)`, bump cursor, alignment
> padding becomes a hole, lowest-addressed fitting hole reused). Every defined
> symbol's offset and the section's `SizeOfRawData` are reproduced.
>
> Registered: **≥ 70 % of sections exact** — the probe rate was 14/18 = 78 %,
> and real TUs are more mixed-size than the probe grid, so I register the lower
> end. Registered further: **failures concentrate in mixed-size sections**, i.e.
> sections whose objects share one size are ≥ 95 % exact.
>
> *Rival R1′*: **no hole reuse** scores at least as well as hole reuse on real
> objs. On probes the margin was only 14 vs 12; if R1′ wins, §5.4's hole-reuse
> clause is a probe artefact and the two hand-worked examples in §5.4 are
> unrepresentative.
>
> *Rival R1″*: the walk is not `.gl` file order at all on real TUs but
> **ascending `.gl` id** (declaration order), i.e. `.bss` behaves like `.data`
> and the probe grid's permutation was an artefact of one-object-per-line
> sources.
>
> **Decline floor: < 30 % exact for the best variant ⇒ §5.4 is downgraded from
> "rule" to "probe-only conjecture" in the document, in those words.**

## R2 — the `.data` allocator on real objs

> **R2.** For each of the 70 non-COMDAT multi-symbol `.data` sections: walk in
> **ascending `.gl` record id** (A2, declaration order) and allocate with A3.
> Every offset and the section size reproduced.
>
> Registered: **≥ 70 % exact**, and **strictly better than the `.gl`-file-order
> walk** (rival R2′). The probe rate for `.data` was 12/14 = 86 %.
>
> *Rival R2′*: `.data` uses the `.gl` file order like `.bss` does, and §5.3 is
> wrong — the probe cells happened to declare in an order that coincided.
>
> **Decline floor: if R2 and R2′ score within 5 percentage points of each
> other on the real set, the discrimination is declared not achieved** and A2
> stays a probe-only rule.

## R3 — the eager/deferred split, on real objs

> **R3.** An object is **deferred** exactly when the `.gl` carries a companion
> record named `$<identifier>$initializer$`. In every non-COMDAT `.bss` that
> mixes the two, every deferred object gets a **strictly higher** address than
> every eager one, and the deferred block's ascending-address order is the
> **reverse** of its `.gl` file order.
>
> Registered: **zero counterexamples** on the real set. This is a stronger form
> of the claim than §5.2 tested, because §5.2's mixed cell was one probe.
>
> *Rival R3′*: the two groups **do** interleave in some real TU, i.e. the
> partition is not a partition of addresses but only of walk positions.
>
> **Decline floor: any interleaving counterexample ⇒ A1's "never interleave"
> clause is refuted and must be restated as measured.**

---

## R4 — mixed-size allocation (§8.1), the skip-and-retry walk

`§5.5` records two verbatim counterexamples where an object *later* in the walk
is allocated *earlier*, and both "look like the walk yields to alignment". Two
readings are registered, and the grid below is designed to separate them. **The
grid is compiled only after this file is committed.**

> **R4-A (primary).** The walk is a **single pass with deferral**: an object
> whose alignment would force the cursor to skip bytes is **passed over** and
> retried after the following object, once, at each step; the pass-over is what
> produces §5.5's inversions. Formally: at each step, among the objects not yet
> placed, take the **first in walk order that needs no cursor padding**; if none
> does, take the first in walk order and pad.
>
> **R4-B (rival).** There is no deferral. The apparent inversion is the
> **hole-reuse rule with a different hole policy** — specifically, holes are
> searched **best-fit** rather than lowest-address, or the cursor padding is
> *not* turned into a hole while an explicit `align` gap is.
>
> **R4-C (rival).** Neither: the walk order for mixed-size objects is not the
> `.gl` order at all but the `.gl` order **stably sorted by descending
> alignment** (a classic "largest alignment first" packer).

Grid: **20 fresh random mixed-size cells** at a seed committed here —
`20260804` is the seed w-bss already used, so this lane uses **`20260805`** and
draws `k ∈ [4,9]` objects from the same 11-type table in `alloc.py`, plus the
**two verbatim §5.5 cells re-expressed as sources** as controls.

> Registered: **R4-A reproduces ≥ 18 of the 20 cells, and both §5.5 controls.**
> Registered secondary: **R4-A ⊇ A3**, i.e. every cell A3 already gets right,
> R4-A also gets right (no regression on uniform-size cells).
>
> **Decline floor: if the best of R4-A/B/C is < 16/20, §8.1 stays open and the
> document states the boundary instead — "exact for uniform sizes, and here is
> the precise class where it is not".** That is the outcome the brief asks for
> if closure is not reached, and it is registered as an acceptable result, not a
> failure.

## R5 — Rule Y2 (deferred `.bss` symbol order), held out

Y2 says: the *symbol-table* order of a deferred `.bss`'s symbols is the `.gl`
record order regardless of linkage. It was fitted on two cells (all-static and
all-extern dyninit) and §8.3 records it as having no held-out confirmation.

> **R5.** On cells Y2 was **not** fitted on — (a) a `.bss` mixing deferred
> EXTERNAL and deferred STATIC objects in one section, (b) N = 3, 5, 7, 9
> deferred objects, (c) a TU mixing eager and deferred where both kinds carry
> both linkages — the deferred symbols appear in the symbol table in **`.gl`
> record order**, and in particular the **mixed-linkage deferred** cell does
> **not** split externals from statics the way Y1 does for eager objects.
>
> *Rival R5′*: deferred objects obey the **same** two-block shape as Y1 (all
> EXTERNAL first in reverse `.gl`, then all STATIC in declaration order), and
> the two fitted cells could not see it because each had only one linkage. This
> is the reading a writer would get wrong silently, so it is named.
>
> Registered: **R5 right, R5′ wrong**, on all of (a), (b), (c).
>
> **Decline floor: if the mixed-linkage deferred cell matches neither, Y2 is
> restated as "measured only for single-linkage deferred sections" and §8.3
> stays open.**

## R6 — `.tls$` walk order (§8.4)

> **R6.** `.tls$` uses the **same** rules as `.bss`/`.data`: initialized and
> uninitialized thread-locals share one section, walked in **`.gl` file order**,
> allocated by A3.
>
> *Rival R6′*: `.tls$` is walked in **declaration order** (like `.data`) for all
> of its objects, initialized or not — i.e. the section's *kind* (initialized vs
> not) picks the walk, and `.tls$` being a single merged section makes it
> uniformly declaration-ordered.
>
> *Rival R6″*: the initialized and uninitialized thread-locals form two blocks
> within the one section, each with its own walk.
>
> Registered: **R6**, on a grid of 6 cells (uninit-only, init-only, mixed, all
> at N = 6 with names whose `.gl` order differs from declaration order, plus a
> mixed-size cell and a `static` cell).
>
> **Decline floor: if no member of {R6, R6′, R6″} matches all six cells, §8.4
> stays open with the measured orders transcribed.**

---

## Registered bias

Lane `w-bss` registered *"I expect `.data`/`.bss` to be boringly regular … which
makes me likely to under-vary"* and it was the correct worry. **My** bias is the
opposite and I register it: I arrive with a document that already reads as a
near-complete spec, and the incentive is to **confirm** it rather than to find
where it breaks on real TUs. The mitigations are structural, not intentions:

* the grading set is real workload objs, chosen **before** looking at any of
  their sources, and its size (191 sections) is fixed here so it cannot be
  trimmed to the cells that pass;
* every rule has a named rival that is scored on the same data;
* R1 and R2 register a **rate**, not "it works", so a 60 % result is a
  registered miss and not a rounding-up;
* R4's decline floor explicitly registers *"state the boundary"* as an
  acceptable outcome, which removes the incentive to force a closure.

A second bias worth naming: the `.gl` parser is **mine**, written this session,
and a parser bug and an allocator error produce the same symptom. R0 exists for
exactly that reason and gates the rest; and R1's rival R1″ is there because a
size-field misread would tend to look like a walk-order failure.
