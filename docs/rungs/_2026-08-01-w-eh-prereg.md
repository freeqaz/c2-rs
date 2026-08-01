# Pre-registration — lane `w-eh`, boards #133 / #121 / #138 (2026-08-01)

Written and committed **before** the first `.cod` of this lane was captured and
before any probe corpus was generated. Everything below is graded in
`docs/rungs/_draft-roadmap-9.15.md`.

Two facts were known at registration time and are *not* findings of this lane:

* `docs/EH_RECORDS.md` **already exists** — 1,711 lines, §1–§10, derived from
  **obj bytes** (GT-EH, GT-IP2STATE, EHMS). The brief said the file was mine to
  create outright; it is not. So #133 is not a transcription onto a blank page,
  it is a **second, name-carrying source for a layout already derived from
  bytes** — exactly the #136 relationship (§9.9.3). That makes the byte model a
  control that can go red, which a blank page would not have been.
* §9.12's `scripts/gt_label_cod.py` already captures 20 shapes × 4 modes and
  parses `$M` / `$T` / funclet labels. This lane extends the instrument rather
  than building one.

---

## Item 1 — #133, the EH record layout from `.cod`

The shapes are varied over **structural counts**, not contents (§9.13.1
consequence 2: a generated axis is only as good as the axes it varies; a 70-case
block that varied argument *values* but never **arity** hid a mis-emit). The
registered axes are: number of try blocks (0,1,2,3), nesting depth (0,1,2),
catch clauses per try (1..4), destructible objects in scope (0..3), functions
per TU (1..3), catch-by-value vs by-reference vs ellipsis, and `/EHsc` vs `/EHa`
vs `/O1` vs `/O2` vs `/Ox`.

* **A1 — totality, with a named printed residue.** Every `DD`/`DB`/`DW`/`DQ`
  datum in every EH-owned data COMDAT of every probe is claimed by exactly one
  named field of the layout. Predicted residue **0 words on fitted shapes**;
  on held-out shapes predicted **0**, and any non-zero residue must be **named
  and printed**, never summarised as a count. *Refuted by* an unclaimed word
  this lane cannot name.
* **A2 — prediction on shapes not fitted.** For each held-out probe, predict
  from the **source** alone, before reading its `.cod`: `nTryBlocks`,
  `nIPMapEntries`, `maxState`, the number of `__catchsym$` arrays, the number of
  `HandlerType` entries in each, and the number of funclets. Predicted **≥ 85 %**
  of those cells exact. *Refuted by* < 60 %.
* **A3 — the control that can go red: the `.cod` must agree with §8.3's
  byte-derived `FuncInfo`.** The nine dwords at +0x00..+0x20, their order, and
  the 36-byte size. Predicted **9/9 agreement**. *Refuted by* any disagreement —
  and a disagreement is the more valuable outcome, because one of the two
  sources is then wrong and the `.cod` names its fields where the bytes do not.
* **A4 — `/EHa`.** Predicted: c2 **accepts** `/EHa`, and `EHFlags` (+0x20) is
  **not 1** under it, against §8.3's "1 on all 21" under `/EHsc`. *Refuted by*
  `/EHa` rejected, or `EHFlags == 1` under both — in which case §8.3's claim is
  correctly scoped to *both* modes and this axis is inert, which must be said.
* **A5 — `adjectives` scales with the catch clause, not the type.** The same
  caught type by value / by `const&` / by `&` gives three different
  `adjectives` in one probe. Predicted `0x00` / `0x09` / `0x08` per §8.3.
* **A6 — the structural-count law.** Predicted, from the source:
  `nTryBlocks` == the number of **lexical `try` statements**, one `__catchsym$`
  array per try block, and `maxState` strictly increasing with (destructible
  objects + try blocks). *Refuted by* any probe where a count is not a function
  of the registered structural axis — and nesting is registered as the axis most
  likely to refute it.

**Registered up front as the honest bound**: #133 moves the census by **0 by
construction**. It is Phase-5 groundwork. No rung is claimed and none may be
manufactured from it.

## Item 1b — #121, `codec::gl_offset_framed`

* **B1 — the verdict.** Predicted **NOT SETTLED**, and not settled *in
  principle*: `.cod` is c2's **output** listing, `gl_offset_framed` frames
  records in c2's **`.gl` input** bundle, and §9.5 already refuted the existence
  of any IL dump with a positive control. *Refuted by* any EH-record or listing
  datum that names a `.gl` record offset.
* **B2 — the control that can fail, because B1 as stated is unfalsifiable
  hand-waving otherwise.** The `.cod` names every **emitted** function, and #136
  proved that set equals the obj COMDAT set exactly. So the listing *can* grade
  the emitted subset of the framing question. Measured on a TU where the
  over-fit bites (`src/App.cpp` per `GAPS.md` §8.2: predicate finds 34, loosened
  finds 6,069, 158 emitted): predicted the `.cod` can adjudicate **≤ 5 %** of the
  6,069 records, because 158/6,069 = 2.6 %. *Refuted by* the `.cod` naming more
  than 5 % of them — which would mean the listing does reach into the IL and B1
  is wrong.
* **B3 — re-verify the over-fit itself rather than quoting it.** Re-run both
  predicates on that TU's `.gl`. Predicted **34** and **6,069**, with **6,068**
  of the loosened set landing on a `4F 1F` function start. *Refuted by* either
  count moving. (`crates/c2-il` is owned by lane `w-rerank`; this is a read-only
  re-measure in a scratch script, no edit.)

## Item 2 — #138, what governs the label-number gaps

§9.12 measured `last funclet → first EH-state $M` at **2,3,4,5,7,8,9,10,11** and
`state table $T → first triple` at **0,1,2,3**, and refused to model them.

The three candidates the brief names are registered as rivals, plus a fourth
this lane adds:

* **C1 — the fourth candidate, registered as the leading one: the gap slots are
  not missing at all, they are label numbers the §9.12 parser did not read.**
  `gt_label_cod.py`'s regexes match `$M`, `$T` and `__catch$`/`__unwind$`/
  `__tryend$` only. If c2 also prints `$LN`, `$L`, `$I`, `$B` or any other
  `$`-prefixed label from the **same counter**, those occupy the gap slots and
  the gaps are an **instrument artifact**, not a compiler unknown. Predicted
  **≥ 90 % of gap slots are named somewhere in the same `.cod`** under a prefix
  §9.12 did not parse. *Refuted by* < 50 %.
* **C2 — the counter is per TU and monotone, never per function.** Predicted:
  across every multi-function probe, allocation numbers ascend across the
  function boundary and no number is reused. *Refuted by* any reset or reuse.
  (This is the "per-TU versus per-function counter resets" candidate; it is
  cheap and it is registered as **expected to be inert** — §9.12's P9 already
  implies it. Registering an inert control as inert.)
* **C3 — labels consumed by bodies inlined away.** Probe: a TU with `k`
  identical always-inlined callees, `k = 0..4`, EH shape held fixed. Predicted:
  if C1 is false, the residual gap moves with `k`. *Refuted by* a gap flat in
  `k`.
* **C4 — labels allocated by phases that emit nothing.** Probe: add a `static`
  function that is **never referenced**, so c2 discards it (§9.5's `globally
  unreferenced` disjunct). Predicted: if the counter advances across a function
  that reaches no obj, the residue is phase allocation and is **not** predictable
  from the emitted shape.
* **C5 — the verdict registered as a disjunction, so that both branches are
  gradeable.** Either (a) C1 holds and the gaps become **fully accountable from
  the listing** — in which case a cardinal `plan_labels` becomes *possible* but
  still needs the un-printed residue to be zero, which is a separate claim I do
  **not** register as true; or (b) C1 fails and the residue is an inlining /
  phase artifact, in which case the answer is **"not predictable"** and this lane
  **ships no model**. `LABEL_COUNTER.md` §6.15.3 already calls the `/O1`
  inline-decline schedule *"generated by no formula"*; if the gaps land there,
  the brief's instruction is to say so and stop.

**Registered refusal.** Under branch (b) no stride, no formula and no
`plan_labels` change ships, however tempting a fit over the fitted shapes looks.
A wrong `$M` number is a wrong-bytes obj (§9.12.2).

## What this lane will NOT do

* No port code. Both items are measurement and transcription; §9.13's "do not
  manufacture a rung" applies and is registered.
* No edit to `docs/ROADMAP.md` (recorded add/add conflict site) and no edit
  anywhere in `crates/c2-il` (lane `w-rerank` is live there).
