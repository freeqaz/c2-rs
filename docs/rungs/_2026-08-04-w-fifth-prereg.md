# w-fifth — pre-registration

Written and committed **before any measurement of the registered quantities**.
Base `5e278f0` (master's tip, checked in the worktree `wt-w-fifth`, not a stale
ref). Seam: `crates/c2-harness/src/gap.rs` and its tests, exclusively.

Lane premise: board #179. `docs/ROADMAP.md` §10.19 factored Phase 7 into four
predicates — **A** (`.ex` segments == obj `.text` COMDATs, gate-anchored), **B**
(every emitted symbol binds), **C** (obj section set ⊆ the writer's vocabulary),
**D** (every emitted COMDAT in the port's per-function codegen class) — and
claimed `A∧B∧C∧D` is **exactly** the match set by name. §10.21 records that this
is now **refuted**: the conjunction is 6, the differential grades 8, and the
scan's known-answer control prints `D 2`. Lane w-r1c converted two TUs through a
**whole-TU** recognizer (`IlBundle::dyninit_tu`) and left the control red on
purpose rather than widen D.

This lane looks for the **fifth term**.

---

## 0. The definitions, fixed in prose, before any code

### 0.1 What the four factors were actually a factorization *of*

Reading §10.19's four predicates back, they are not four unrelated conditions.
They are four *questions the port must answer yes to* before its output can be
the reference's bytes:

| | question |
|---|---|
| **A** | do the port and the reference agree on **what set of things is emitted**? |
| **B** | can the port **name** everything in that set? |
| **C** | can the port's writer **write the containers** the obj needs? |
| **D** | does the port have an **accepted route to the contents**? |

A/B/C are properties of the obj and the binding. **D is the odd one out**: it is
not a property of the obj at all, it is a property of *the port's acceptance
machinery*. Specifically, `emit-class-complete` (`emitted == in_class`) is the
per-function census's verdict, i.e. **"`PortC2`'s per-function acceptance path
takes every COMDAT here"**.

§10.19 was measured when `PortC2::build` had exactly **one** acceptance path, so
"the port has a route to the contents" and "the per-function path accepts every
COMDAT" were the same sentence. They are not the same sentence any more.
`PortC2::build` now tries `IlBundle::dyninit_tu()` **before** `functions()`
(`crates/c2-core/src/lib.rs:204`). D is the per-function reading of question 4,
and it is now one reading of two.

### 0.2 The fifth term, defined

> **E — whole-TU acceptance.** At least one **registered whole-TU recognizer**
> accepts this bundle.

A *whole-TU recognizer* is an acceptance predicate over the entire `IlBundle`
that no per-function predicate can express, and which `PortC2::build` consults
as a distinct arm. `c2-il` has exactly one today: `IlBundle::dyninit_tu()`, the
`??__E` dynamic-initializer shape.

E is **not** hard-coded to `??__E`. It is the disjunction over an explicit,
named **registry** declared in `gap.rs`:

```
WHOLE_TU_RECOGNIZERS: [(&str, fn(&IlBundle) -> bool)]
  ("dyninit-??__E", |b| b.dyninit_tu().is_some())
```

Each entry also gets its own per-TU key (`emit-whole-tu|<name>`) so the marginal
of each recognizer is visible separately and a registry that grows cannot hide a
recognizer that never fires.

### 0.3 The model, restated

> A byte-exact obj requires **A ∧ B ∧ C ∧ (D ∨ E)**.

D and E are the two readings of question 4 — per-function acceptance and
whole-TU acceptance. Neither is necessary alone (measured: D is not, `D 2`).
**The disjunction is what is claimed necessary**, and the disjunction is what the
known-answer control will be taken over.

**This is a disjunction and not a widening of D.** The brief instructs me to test
that shape rather than assume it, so it is registered as a *prediction* in §2
(D2/D3) with the alternative — that E is true for TUs D also covers, i.e. that
the two overlap and the honest term is not a disjunction — as a named outcome.

### 0.4 Why this is not "retuning D until the control goes green"

Four separable commitments, all registered now:

1. **D's definition is byte-for-byte untouched.** `emit-class-complete` keeps
   meaning `emit-emitted == emit-in-class` from the per-function census. Nothing
   in `c2-il`'s `census.rs` is touched (it is another lane's seam anyway), so the
   scan's `census/gate disagreement: 0` line must stay 0. If it moves, the lane
   declines (§3, clause 5).
2. **D's own violation count keeps being printed**, as a number, next to the
   note that says D is not necessary. The refutation of §10.19 is a *finding*
   and stays visible; it is not absorbed.
3. **E reads a different oracle.** D reads the census's `FnVerdict::in_class()`.
   E reads `c2-il`'s whole-TU recognizer directly. They share no variable, no
   intermediate and no call — the §10.18 trap for this file is a shared variable
   with two consumers, so E is written in step 1h from `captured.bundle` and
   touches nothing 1e/1g/step 3 computes.
4. **E is symmetric with D in kind.** D is a *class-membership* predicate
   (does the per-function acceptance path take this?), evaluated without running
   the emitter. E is the same kind of predicate at whole-TU granularity (does a
   whole-TU acceptance path take this?), also evaluated without running the
   emitter. E is deliberately **not** "the port emitted it" and **not** the class
   field: either would be circular, and the model would become unfalsifiable.

   The visible consequence, registered here as an accepted cost: like D, **E is
   not sufficient.** `PortC2::build_dyninit` carries a gate that lives in
   `c2-core` and not in the recognizer — the `/GF` fence, "the computed `??_C@…`
   name must be one `.gl` spells". A TU that `dyninit_tu()` accepts and the fence
   refuses would be E-true and not a match. That would show up as an
   over-prediction in the set-identity line, and I would report it, not repair
   it. (Prediction: does not occur on this workload, which is `/O1`, so `/GF` is
   implied — §2, D4.)

### 0.5 How it degrades honestly

The requirement is: **when a new emit path lands that E does not model, the
control must go red again.**

The mechanism is that **the registry is closed and explicit**. A new whole-TU
acceptance arm added to `PortC2::build` does *not* enter
`WHOLE_TU_RECOGNIZERS` — that is a separate, deliberate edit in this file. A TU
converted by an unregistered path is therefore a `match` with D false and E
false, so the `D∨E` control column goes red and names it. That is exactly the
event that happened on 2026-08-04 with `dyninit_tu`, and the design keeps it
happening.

The rejected alternative is registered so it cannot be quietly adopted later:
**E := `bundle.decodes() && bundle.functions().is_none()`** is one line, needs no
registry, and is **wrong for this lane's purpose**. `decodes()` is defined as
`functions().is_some() || dyninit_tu().is_some()` and its doc comment says
"adding a third path means adding it here" — so a third recognizer would enter
`decodes()` and E would **silently absorb it**. It is the open-world definition
and it makes the control permanently green by construction. The registry is the
closed-world one.

I claim **no static guard** that the registry is complete. There cannot be one:
`gap.rs` cannot enumerate `c2-core`'s match arms, and a test asserting
`decodes() == functions().is_some() || registry_any()` would pass vacuously on
every bundle that exercises no new path. The guard is **empirical and is the
scan control itself**, and I will say that in the code rather than imply more.

### 0.6 What the FRONTIER means afterwards

Today: `A∧B∧C ∧ ¬D ∧ ¬match` — "the only remaining factor is codegen breadth".

Afterwards: `A∧B∧C ∧ ¬(D∨E) ∧ ¬match` — **"no acceptance path the port has
covers this TU, and per-function codegen breadth is the whole remaining
distance."**

The change is a *narrowing*: a TU that some whole-TU recognizer already accepts
but that is not a match is **not** on the codegen-breadth frontier — its blocker
is that whole-TU emitter's own fence, which is different work. Removing it is
correct, and every entering/leaving TU will be reported **by name**, because a
parallel lane (w-front) is working this list by name. Prediction: the membership
does not move (§2, D5).

---

## 1. What is being built

All inside `crates/c2-harness/src/gap.rs`:

* `WHOLE_TU_RECOGNIZERS` — the named registry (§0.2).
* per-TU keys, written in step 1h beside C and D:
  * `emit-whole-tu|<name>` — one per registered recognizer that accepts.
  * `emit-whole-tu-any` — **factor E**.
  * `emit-emit-path` — **D ∨ E**, the generalized question-4 term.
* `factors()` returns `[bool; 5]` (`[A,B,C,D,E]`); a new `emit_path()` helper is
  `f[3] || f[4]`.
* `factor_counts()` gains |E| and |A∧B∧C∧(D∨E)| alongside the existing
  |A∧B∧C∧D|, which is **kept and printed** so §10.19's original conjunction stays
  measurable.
* `factor_control_on_match_tus()` returns violations for A, B, C, D, E **and
  D∨E**, with the "all must be 0" claim attached to **A, B, C, D∨E** and D/E
  printed as diagnostics with an explicit "not required alone" label.
* `factor_frontier()` filters on `¬(D∨E)` per §0.6.
* Portable tests: the control can go red for an unregistered whole-TU path; the
  registry is non-empty and its names are distinct; the frontier excludes an
  E-true TU; D's marginal is unchanged by E's presence.

**Every new key is a pure addition.** No existing `emit-*` key is redefined, no
class predicate is touched, `PORT_WRITER_SECTIONS` is untouched, and nothing
outside `gap.rs` is edited.

---

## 2. Predictions, registered before the first scan

Incumbent on master, from the brief and to be re-measured on this lane's own
base rather than quoted: **A 28, B 338, C 114, D 8, A∧B∧C 25, A∧B∧C∧D 6,
FRONTIER 17, match 8, mismatch 0, codegen-gap 0, vocab-gap 863, capture-fail 7**.

| id | prediction |
|---|---|
| **D1** | **\|E\| = 2** on the graded population. Both are the w-r1c TUs: `src/system/synth/tomcrypt/TomCryptLicense.cpp` and `src/system/zlib/ZlibLicense.cpp`. Interval: `2 ≤ \|E\| ≤ 6`; above 6 I treat the recognizer as looser than I understood and report that as the finding. |
| **D2** | **E and D are disjoint** on this workload — no TU has both. If any TU has both, the "two readings of one question" story is still fine but the disjunction is not the *minimal* form, and I say so. |
| **D3** | **\|A∧B∧C∧(D∨E)\| = 8**, and it is **exactly the match set by name**. |
| **D4** | **No over-prediction**: no non-`match` TU satisfies `A∧B∧C∧E`. The `/GF` fence cannot fire on this workload (flags are `/O1`, which implies `/GF`). |
| **D5** | **FRONTIER 17 → 17**, membership identical by name. It can only shrink, and only by a TU that is E-true, non-match and inside A∧B∧C — which D4 predicts is empty. |
| **D6** | **Known-answer control**: `A 0 B 0 C 0 D∨E 0`, with `D 2` and `E 6` still printed as diagnostics (E is false for the 6 per-function matches). |
| **D7** | **Per-TU class-change diff = 0 of 878.** This lane adds keys and changes printed joins; it must move no TU between `match`/`mismatch`/`codegen-gap`/`vocab-gap`/`capture-fail`. Any nonzero here is the §10.18 trap firing and outranks the fifth term entirely. |
| **D8** | **A, B, C, D marginals unchanged**: 28 / 338 / 114 / 8. **A∧B∧C = 25** unchanged. |
| **D9** | `cargo test --workspace --release` 665 → ≥ 671 passed, 0 failed, 24 targets. `scripts/gate.sh --jobs 6` 12/12 PASS, 2,592 verdicts, 0 mismatch — **unchanged**, since nothing outside `gap.rs` moves. |

---

## 3. Decline floor — against the named incumbents, not a bare threshold

The lane **declines and leaves the control red** if any of:

1. **D7 fails** — any TU changes class. A predicate that moves a TU while
   claiming to only add a term is the §10.18 defect, and it outranks the result.
2. **D8 fails** — A, B, C or D moves off 28 / 338 / 114 / 8, or A∧B∧C off 25.
   E is an addition; a marginal that moves means I edited something I said I
   would not.
3. **mismatch ≠ 0**, or `match` ≠ 8, or `capture-fail` ≠ 7, or the
   `dc3-decomp` HEAD moves across the bracket. Any of these voids the scan and
   it is re-taken, not reported.
4. **D3 fails and the miss is an over-prediction** — i.e. `A∧B∧C∧(D∨E)` contains
   a non-matching TU. Then E is looser than the emit path it models and the
   honest report is "the fifth term is not the recognizer; leave the control
   red".
5. **`census/gate disagreement` ≠ 0**, or the census totals move. That is the
   symmetry w-r1c protected, and breaking it is precisely the failure this lane
   exists to avoid.
6. **The control goes green for a reason I cannot demonstrate a red case for.**
   Concretely: unless a portable test shows a `match` TU covered by *no*
   registered recognizer and outside D turning the `D∨E` column red, the green
   is not evidence and the lane declines.
7. `gate.sh` not 12/12 PASS with 2,592 verdicts and 0 mismatch, or workspace
   tests below 665 passed / any failure.

A **partial** outcome that is explicitly acceptable and not a decline: E lands,
D3 holds, but the FRONTIER moves. That is reported by name and handed to w-front;
it is a result, not a failure.

---

## 4. Declared bias, and its direction

**Confirmatory, and the brief says so.** The target numbers — match 8,
A∧B∧C∧(D∨E) = 8, control all-zero — are on the front page and in my own brief.
The available failure mode is not a surprising result; it is **shaping E until
the joint prints 8 and the control prints zeros**. Every degree of freedom in
"a whole-TU recognizer accepts this" moves that number: recognizer-level vs
emitter-level, registry vs `decodes()`, whether an unreadable obj is in or out,
whether E is a disjunct on D or a widening of it.

Second bias, specific to this lane: **toward declaring the model repaired.** A
green control is a nicer report than a red one, and the cheapest route to green
is an E broad enough to swallow anything — which is precisely the instrument
this project keeps rebuilding as a false-green (memory: "absence read as
success", eight instruments and counting).

Mitigations, registered:

1. **Every predicate is fixed in prose above, before code**, including the two
   choices that move the number most: recognizer-level rather than emitter-level
   (§0.4 item 4) and closed registry rather than `decodes()` (§0.5). Both are
   written with the rejected alternative named, so adopting the alternative later
   is a visible reversal.
2. **The first full scan is the recorded one.** If it disagrees with §2, §2 is
   what it is scored against; the predictions are not revised and re-run.
3. **A green control does not count as evidence unless the red case is
   executable** (clause 6). The control's value is entirely in its ability to
   go red, and w-r1c's is worth more red than a fitted one is worth green.
4. **The per-TU class dump is taken before and after and diffed** (D7), because
   §10.18's whole lesson is that a count is only evidence about the predicate
   that produced it — and that lane's damage was invisible in every printed
   count.

---

## 5. Provenance protocol

- `git -C ../dc3-decomp rev-parse HEAD` **before and after every scan**, both
  reported. Held at `86357b58` at lane start. If it moves, or `capture-fail ≠ 7`,
  the scan is **void** and re-taken — a mid-move checkout reports 39 capture-fail,
  drops the graded population 871 → 839 and pulls every factor down with it.
- Base and tip scans taken **inside one workload-tree state**, both with
  `--jsonl`, so the per-TU diff (D7) is over rows from the same corpus.
- The JSONL header self-records `workload_head`, `workload_dirty` and
  `c2rs_dirty`; those are checked, not assumed.
