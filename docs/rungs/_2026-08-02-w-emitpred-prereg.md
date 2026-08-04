# w-emitpred — pre-registration: the out-of-sample gate for #161

Written and committed **before any cell is compiled, before any held-out truth
is read, and before any structural-axis probe exists**. Base `39dcfb7`
(master), worktree `wt-w-emitpred`. Lane job: try to kill the fitted emit
predicate (#161, `PHASE7_PLAN.md` §2), and build the gate that decides whether
it may ever ship into R3.

Provenance, recorded now because it invalidates cross-session byte-comparison:
**`../dc3-decomp` HEAD is `51fb5b73` at this lane's start** — it has moved
again since the plan session's `13b583df`. Every scan/capture this lane runs
must record this rev; no cached number from an earlier rev is comparable.

## The predicate under test, verbatim scope

`PHASE7_PLAN.md` §2: least-fixpoint reachability from roots (strong linkage,
explicit instantiation definitions, `dllexport` closure, dynamic-initializer
thunks, kept data), ODR-use over **kept** definitions **pre-optimization**,
vtable-forced virtuals via kept constructors, no transitivity through dead
code, `sizeof` does not count. Fitted on **172 synthetic cells, zero
violations, zero out-of-sample validation**. That last clause is what this
lane exists to fix or to exploit.

## Declared bias

**Deflationary / adversarial.** The brief asks me to kill the rule and calls a
refutation the more valuable outcome; my incentive is to find breaks. Guards
against manufacturing them:

1. **A "violation" on a designed cell is scored only after an independent
   re-derivation.** A second agent, given only the cell's source and §2's
   text (not the compiled truth, not the first agent's prediction), must
   reproduce the prediction. If the two derivations disagree, the cell is
   `AMBIGUOUS` — the predicate's *statement* is too loose to apply, which is
   reported as its own (lesser) defect class, never as a violation.
2. The held-out gate is symmetric: floors are registered for both pass and
   fail, and the incumbents are controls, so I cannot move goalposts in
   either direction after seeing the score.
3. All grading is against the **obj** `.text` COMDAT leader set; the `.cod`
   PROC set is a cross-check and name/demangling source only (standing rule,
   third strike recorded in `PHASE7_PLAN.md` §6).

## Part 1 — the held-out real-TU gate (D1(b) of the plan)

### Population, drawn and frozen now

Seeded procedure, run before this commit and reproduced here exactly:
`pool = sorted(files.txt minus the 6 match TUs minus the 7 capture-fail TUs)`
(865 TUs; the match/capture-fail lists were read as *class labels only* from
`work/dc3-workload/scan-merged-20260731.jsonl`; no per-TU emit quantity was
read). `order = random.Random(161).sample(pool, len(pool))`. DEV = first 8 of
`order`; HELDOUT = next 20 of `order` skipping any TU whose emit-set
statistics are individually quoted in docs/ (the R3 14-TU target list,
App/TextFile/MeterEffect/TomCryptLicense/ZlibLicense) — none were in fact
encountered in the prefix.

**DEV (truth-open, for building the prediction pipeline — 8):**
Part.cpp, CharWeightSetter.cpp, Gen.cpp (rndobj), CharClipDriver.cpp,
FlowLabel.cpp, MoggClipMap.cpp, ShadowMap.cpp, HolmesUtl.cpp
(full paths as printed by the procedure above).

**HELDOUT (truth-quarantined — 20):**
CalibrationPanel.cpp, Compress.cpp, FIRFilter.cpp, UIListWidget.cpp,
Joypad.cpp, Keyboard_Xbox.cpp, UIListMesh.cpp, Challenges.cpp, Archive.cpp,
LocalePanel.cpp, DifficultyProvider.cpp, HolmesClient_NetSocket.cpp,
FxSendEQ.cpp, HamStoreOffer.cpp, TexProc.cpp, FileCache.cpp, PanelDir.cpp,
Option.cpp, keygen_xbox.cpp, PartyModeMgr.cpp
(paths: src/lazer/meta_ham/, src/system/…, src/keygen_xbox.cpp,
src/lazer/game/PartyModeMgr.cpp — exactly the printed list, in this order).

### Quarantine, in force from this commit until the predictions commit

For the 20 HELDOUT TUs, **no agent of this lane reads any c2-output-derived
artifact**: cached reference objs, fresh objs, `.cod`/`/FAsc` listings, any
`emit-*` scan key, any per-TU row of any scan jsonl, or any docs passage
quoting their emit statistics. **Allowed** (they are c1xx-side inputs, exactly
what an R3 model would legally consume): source text, headers, the captured
IL container (`.ex` segments, `.gl` names via the separator-aware extractor),
and compiler *front-end* diagnostics (`/Wall`/`/W4` C4505/C4514 stderr) —
provided the invocation's obj/listing outputs are discarded unread into a
quarantine directory. DEV TUs are exempt: anything may be read.

### The prediction

For each HELDOUT TU: the **exact set of function names c2 emits** (the `.text`
COMDAT leader symbol set of the reference obj at the workload's own flags),
predicted by an operationalization of §2's predicate built **only on DEV TUs
and probe cells**, frozen, then run once. The 20 predicted sets are committed
as a git object **before** any truth artifact is read. After that commit,
truth is extracted (cached objs / fresh replay at `dc3-decomp @ 51fb5b73`,
`.cod` PROC as cross-check) and scored with no further edits to predictions
or pipeline.

### Metrics

Per TU: truth `E(t)`; prediction `P(t)`; candidate universe `C(t)` = the
c1xx-side name universe the frozen pipeline defines (its definition freezes
with the pipeline, before the held-out run); `U(t) = C(t) ∪ P(t) ∪ E(t)`.
TP = `|P∩E|`, FP = `|P∖E|`, FN = `|E∖P|`, TN = `|U| − TP − FP − FN`.
All headline numbers are **micro-aggregated over the 20 TUs**.

### Incumbents, registered as controls — never a bare threshold

* **never-emit**: `P(t) = ∅`. Expected per-body accuracy on the workload is
  ~93 % (c2 emits ~7 % of IL bodies); its actual accuracy is computed **on
  the same `U(t)`** at scoring time.
* **emit-everything**: `P(t) = ` every function-body candidate (one per
  gate-anchored `.ex` segment — the port's current behaviour). Recall 1.0 by
  construction on segment-named bodies; accuracy ≈ the emit rate.

The predicate is interesting **only if it beats both on the same universe.**

### Registered gates and estimates

| # | quantity | point | interval | gate |
|---|---|---:|---|---|
| **V1** | micro-F1 of `P` vs `E` | 0.85 | [0.60, 0.97] | **F1 ≥ 0.80** required to ship; **F1 < 0.50 ⇒ REFUTED on real TUs** |
| **V2** | micro-accuracy over `U` | — | — | must exceed **both** incumbents' accuracy on the same `U` by **≥ 2.0 pp absolute**; failing this ⇒ predicate is worse than a trivial rule, REFUTED as a model regardless of F1 |
| **V3** | micro-precision | 0.93 | [0.78, 1.00] | **≥ 0.95 required to ship into fail-closed R3** (a name predicted Emit that is not emitted would be a silent wrong obj) |
| **V4** | per-TU exact-set count (of 20) | 6 | [1, 12] | reported, not gated — exactness is R3's job via `Unknown ⇒ refuse`, not the predicate's |
| **V5** | F1 recomputed excluding synthesized-name families (`??_G ??_E ??_D ??__E ??__F ??_9` prefixes) | 0.90 | [0.70, 1.00] | attribution only: if V5 ≥ 0.90 while V1 < 0.80, the misses are the #152 synthesis cap, and the verdict text must say so — but the **ship verdict still follows V1–V3 unfiltered** |

### Verdict rule, fixed now

* **SHIP-CANDIDATE** — V1 ≥ 0.80 ∧ V2 pass ∧ V3 ≥ 0.95 ∧ Part-2 axes all
  clean-or-guardable. (#161 graduates from "fitted" to "validated"; R3 may
  build on it, still fail-closed.)
* **SURVIVES, NOT SHIPPABLE** — V1 ≥ 0.80 ∧ V2 pass, V3 < 0.95: the rule is
  real but its operationalization is not safe for fail-closed emission yet.
* **REFUTED ON REAL TUs** — V1 < 0.50, or V2 fail. #161 goes to the board as
  refuted-out-of-sample; the misses are analyzed **only** to name axes, and
  any iteration happens on **new designed probe cells in a future lane, never
  on these 20 TUs** — this population is spent either way and may not be
  reused as a held-out set again.
* **DECLINE** — anything between: reported as "not validated", #161 does not
  ship, no patching.
* **INSTRUMENT-FAIL** — if the pipeline cannot be built at all on DEV (the
  warning channel dry under this toolchain, name normalization unclosable),
  that is reported as its own outcome: the gate is *unpassable with current
  instruments* and #161 remains unshipped. Not a pass, not a refutation.

**No re-fitting after seeing a miss. One shot.**

## Part 2 — structural axes the 172 cells never varied

The 172 cells crossed linkage × reference kind × TU context and varied
*values* exhaustively; the axes below are *structures* they held fixed. Each
axis gets **≥ 4 designed cells** (most get 6+), predictions hand-derived from
§2's text and written to a predictions file **before** the axis's first
compile; compiled at the workload's flags (`/O1 /Oi /EHsc /GS- /c`, X360
toolchain under wibo); graded on obj `.text` COMDAT leaders.

| axis | structure varied |
|---|---|
| A1 | header inclusion depth / same definition reached through nested includes |
| A2 | templates: implicit vs explicit instantiation, `extern template`, never-referenced members of instantiated class templates |
| A3 | virtual & multiple inheritance: adjustor thunks (`??_9`), vtordisp, vbase ctors — the vtable rule under MI |
| A4 | anonymous namespaces: nested, anon-ns class virtuals, anon-ns vs `static` crossing |
| A5 | `static` vs `inline` vs `extern "C"` vs `static inline` crossings, incl. definitions in a header included by the TU |
| A6 | multiple TUs sharing one header, references differing per TU (per-TU independence of the fixpoint) |
| A7 | `#pragma comment`, `#pragma init_seg` — pragma-created roots |
| A8 | PCH (`/Yc`/`/Yu`) — does precompilation change removal/emission |
| A9 | `dynamic_cast`/`typeid` forcing a vtable with **no kept constructor** (plan D6 — the one designed cell the plan itself says could break the ctor⇒vtable formulation) |

| # | quantity | point | interval |
|---|---|---:|---|
| **V6** | axes (of 9) with ≥ 1 confirmed violation of §2-as-stated | 1 | [0, 3] |

Scoring per axis: **any confirmed violation** (surviving guard 1's
independent re-derivation) ⇒ the axis is **BROKEN** and the predicate
as-stated is refuted on it. A broken axis blocks SHIP-CANDIDATE unless the
breaking condition is **demonstrated detectable from c1xx-side observables**
(so R3 can guard it `Unknown ⇒ refuse`) — demonstrated means a working
detector run on the violating cells, not asserted.

## Part 3 — the warning-channel cross-check (D1(a)), reported not gated

One `/Wall`-augmented pass over a DEV-plus-probe population, comparing
warned∪one-step-closure against truth, to re-measure lane B's
precision-1.00/recall-0.928 on real headers. Registered expectation:
precision stays ≥ 0.95 on real TUs (point 1.00); recall point 0.90
[0.75, 1.00]. This channel rides inside the Part-1 pipeline; its independent
numbers are evidence about *which half* of a Part-1 miss failed.

## Method constraints, fixed in advance

* Workload flags only; `/Ox` evidence non-transferable (standing).
* Ground truth = obj `.text` COMDAT leader set; listing = names/demangling
  cross-check only.
* Every count names its predicate (splitter, scanner, flags, dc3 rev).
* Worktree binaries: rebuild `target/release/c2rs` before reading any
  `emit-*` key — stale binaries return silence, not zero.
* `.gl` reads only via separator-aware extractors (`glnames.py` /
  `readers.py`, known-answer-gated), never raw `strings`.
* Scratch under `work/`; no captured IL, objs, or absolute paths committed;
  no `pgrep -f` self-matching watchers; every wait deadline-bounded.
* Subagents write only under `work/` and `docs/PHASE7_VALIDATION.md` /
  this file's scoring section; nothing under `crates/` (lane w-r1 is live
  there).

## What this lane will NOT claim

* Not that a validated predicate converts any TU. A∧B∧C∧D still stands; this
  gate is about factor A's model only.
* Not that passing on 20 TUs is proof — it is one coverage-bounded sample;
  the verdict language will say "survived N held-out TUs", never "correct".
* Not a schedule.
