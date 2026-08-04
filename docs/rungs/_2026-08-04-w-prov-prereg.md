# Pre-registration — lane `w-prov`: census provenance, and the path-bound join

Frozen **before any measurement this lane makes**. Everything below names an
**incumbent**, not a bare threshold, because a count is only evidence about the
predicate that produced it.

Lane `w-repro` (landed as `31b210a`) established that the workload censuses are
graded against a **moving corpus with no provenance record**. This lane makes the
fixes it proposed and answers the two questions it left open.

---

## 0. Incumbents

Handed to this lane, and re-measured on the merged tree (`31b210a`) *before* this
document was written — that re-gate is Part 1 and is reported, not predicted.

| quantity | incumbent |
|---|---|
| `scripts/gate.sh --jobs 6` | 12/12 PASS, 2,592 verdicts, 0 mismatch |
| `cargo test --workspace --release` | 677 passed, **0 FAILED**, 25 targets |
| oracle self-test | 216 PASS, 0 FAIL |
| `cargo build` | 0 warnings |
| TU match / mismatch / capture-fail | 8 / 0 / 7 |
| A / B / C / D / E · FRONTIER | 28 / 338 / 114 / 8 / 2 · 17 |
| **per-function census** | **706402/2463318** |
| **emitted census** | **38457/178972** |
| `.gl` binding records | 1515163 records, 39294 row-conflicts, 732 name-conflicts |
| `grade.py` `.bss` / `.data` (same path) | 110/117 · 68/68 |
| `grade.py` at a MOVED path (w-repro §5) | 87/93 · 53/53 — population −20 % |

**Instrument trap, carried forward:** a failing test target *aborts* the run, so
a low passed-count is not a regression of that size. **Compare the FAILED count,
never the passed count.** Perf geomean wobbles 586–689× on a byte-identical
binary; it is wall-clock and not a signal.

---

## 1. Known-answer control — KAC

**KAC.** An immediately repeated `c2rs gap` scan of the *same* live corpus at the
*same* path reproduces the merged-tree run exactly: per-function census, emitted
census, match, mismatch, capture-fail, A/B/C/D/E and FRONTIER all identical.

If KAC fails, every prediction below is void and the lane reports an unstable
instrument instead of a corpus finding. This control exists because w-repro's
whole result rests on the scan being deterministic at a fixed corpus, and this
lane must not inherit that as an assumption on a *different* instrument (the Rust
gap scan, not the Python `.gl` census).

**KAC-2 (provenance stamp, no toolchain).** `prov.py`'s self-check reproduces a
stamp from a fixture directory with a known sha256 and a known HEAD, and the
consumer check accepts a matched pair and rejects each of the six mismatch
classes. Runs with no compilers present; a stamp checker that cannot fail is the
`STATUS.md` trap-5 shape and must be shown to fail.

---

## 2. The moved census — attribution

The merged-tree re-gate measured **706552/2463393** and **38455/178975** against
incumbents of **706402/2463318** and **38457/178972**. `crates/` was not touched
by the merge (docs only), and `docs/STATUS.md`'s block was collected at `eb4017c`
(09:08) while this scan ran at 09:46. `../dc3-decomp` merged `979488c2` at
09:14:38 and `940d07dc` at 09:15:17, i.e. **between** the two.

**P1.** A `c2rs gap` scan against a frozen `git archive dd9a4bdc src` tree
restores the census to **exactly 706402/2463318** and the emitted census to
**exactly 38457/178972**, with match 8 and mismatch 0 unchanged. The whole delta
is corpus.

* **P1′ (rival).** It restores neither exactly — the delta has a non-corpus
  component, and this lane has to find it before claiming corpus drift.
* **P1″ (rival).** It restores one and not the other.

**Registered confound.** The frozen tree is at a *different path*, and w-repro
§5 showed the path is a live variable for the `.gl` **name join**. P2 controls
for it.

**P2.** A scan against a frozen `git archive 940d07dc src` tree — identical
content to the live corpus, different directory — reproduces the live numbers
**706552/2463393** and **38455/178975** exactly. The function census is
path-neutral, so P1's test is clean.

* **P2′ (rival).** It does not — the function census is path-sensitive too,
  which would mean w-repro's path effect is *not* confined to the name join and
  is a larger finding than that lane recorded.

---

## 3. `sections.jsonl` — the unrecorded corpus version

`work/w-bss/census/sections.jsonl` is committed, was built at 07:21, and carries
no record of which dc3 commit produced it. Every landed `.data`/`.bss` number is
graded against it.

**P3.** Regenerating it with `scripts/regen_census.sh --sections` at dc3
`940d07dc` produces a file whose sha256 **differs** from the committed one, and
the number of TU records that differ is **between 1 and 100**.

* **P3′ (rival).** Byte-identical — the committed file was already built at
  `940d07dc` and carries no stale corpus after all.
* **P3″ (rival).** More than 100 TU records differ, which would mean the
  committed file predates the corpus by considerably more than one morning.

**P4.** With `sections.jsonl` and `glcensus.jsonl` **jointly** regenerated at one
corpus and one path, `grade.py` still reports `.bss` **110/117** and `.data`
**68/68** — `docs/OBJ_DATA_BSS_SHAPE.md`'s landed numbers stand.

* **P4′ (rival).** They move. Then a landed document is wrong and this lane says
  so plainly rather than smoothing it.

---

## 4. The path-bound join — pin or path-free

**P5.** Normalizing MSVC's anonymous-namespace mangling `?A0x[0-9a-f]{8}` to a
constant on **both** sides of the join recovers the full graded population at a
moved path: `.bss` **93 → 117** and `.data` **53 → 68**, i.e. exactly the
incumbent same-path denominators.

* **P5′ (rival).** It recovers some but not all — some other path-derived name
  component exists.
* **P5″ (rival).** It over-joins: `skipped['ambiguous name match']` rises above
  its same-path incumbent, which would make a path-free join *unsafe* rather than
  merely incomplete.

**Registered in advance, so it is not a post-hoc rationalisation:** even a
**green P5 does not by itself justify adopting a path-free join**, because
w-repro §5.1 shows the path also moves `.gl` record *positions*
(`JobMgr.cpp` 1 → 7) with no anonymous namespace involved, and `.gl` file order
is `grade.py`'s winning `.bss` walk model. Restoring the denominator while the
**key** is still perturbed is `STATUS.md` trap 5 wearing the opposite mask. The
decision rule is fixed here: **pin unless P5 is green AND the `.gl`-order
perturbation is explained.**

**P6.** Over the full 871-TU census — not w-repro's 40-TU sample — the share of
`glparse.globals_in_order()` records whose first four characters contain one of
`(&'"{}#!` is **6.5 % ± 1.0 pp**.

* **P6′ (rival).** Outside that band, i.e. the 40-TU sample did not generalise.

**P7 (restatement, known answer).** `r56.py:124`'s `set`-ordering hazard is
**latent, not live**: after the fix to `(gid, index)`, output is byte-identical
to the incumbent `md5 841f037bd0a57aff974f0f0751c99a96` at `PYTHONHASHSEED` 0–4.
A change that alters the bytes would mean the hazard *was* firing and w-repro's
5-seed probe missed it.

---

## 5. Priced decline clause

**`JobMgr.cpp`'s `.gl` index moving 1 → 7 purely on `cwd`, with no anonymous
namespace and identical `gid`, size, alignment and linkage, is DECLINED after
three probes** if no single measurable variable accounts for it. The three probes
are named now so the count cannot be inflated after the fact:

1. Does the record set (`all_names`) differ between the two paths, i.e. is a
   record *added* before index 1 rather than the record *moving*?
2. Is the shift a function of the path's **length** (not its content) — same
   tree, third path, chosen to have the same length as one of the two?
3. Do the six records it jumps carry a `?A0x` name, i.e. is this the
   anonymous-namespace effect reaching a TU that has none, via a header?

**The price of declining:** `grade.py`'s winning `.bss` model (`.gl` order,
85/110 = 77.3 %) stays fitted on a key with one known, unexplained perturbation;
board **#203** stays **OPEN** rather than closing; and any future lane that moves
the corpus must re-derive this. That is stated so the decline costs something and
is not the cheap exit.

**Second decline clause.** If `regen_census.sh --sections` exceeds its 7200 s
deadline, or reports more than the expected 7 compile failures, the regeneration
is **declined** and P3/P4 go unmeasured, reported as `NO-RESULT` — not as a pass.
A run that graded nothing is a failure, not a pass (`STATUS.md` trap 5).

---

## 6. What this lane will NOT do

* **Not touch `crates/`** — lane `w-cfgimpl` owns `c2-il` and `c2-core`.
* **Not hand-edit** `docs/STATUS.md`'s generated block or `docs/rungs/INDEX.md`.
* **Not renumber** w-repro's frozen board rows to close the 196–200 gap.
* **Not adjust an incumbent** to match a number this lane measures. If a number
  moves, the move is the finding.
