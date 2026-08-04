# PHASE7_VALIDATION — the out-of-sample gate for the fitted emit predicate (#161)

    Lane:      w-emitpred, 2026-08-02 (relaunched 2026-08-04, twice)
    Prereg:    rungs/_2026-08-02-w-emitpred-prereg.md — committed at `fdacb84`,
               before any cell was compiled or any held-out truth read. Floors,
               incumbents, populations and verdict rules are all there and are
               not restated here in any way that could drift from them.
    Status:    IN PROGRESS. Every results section below is either filled with
               measured numbers or marked PENDING; a PENDING section is not a
               result (§9.18.8: absence reads as success unless forbidden —
               this line forbids it).
    Toolchain: `compilers/X360/16.00.11886.00` under wibo, flags
               `/O1 /Oi /EHsc /GS- /c` (workload flags; `/Ox` evidence is
               non-transferable by standing rule).

**What this doc decides:** whether #161 — the emit predicate fitted black-box
on 172 designed cells with zero violations (`PHASE7_PLAN.md` §2) — survives
contact with anything it was not fitted on, and whether it may ever ship into
R3. In-sample 360/360 has already burned this project once; the number that
decides is the one that cannot be revised.

---

## THE ANSWER, IN ONE SENTENCE

**#161 is not cleared.** A fitted predicate that misses its own pre-registered
violation interval **on the high side** — V6 = **5**, or **4** under the most
hostile defensible reading, against a registered point of 1 and an interval of
**[0, 3]**, and a **lower bound** either way — while carrying a demonstrated
false-positive class that touches **36.7 % of the workload's TUs**, is not a
model anything should ship on.

**The robustness is the point:** the headline survives its own author's most
hostile reading (§6d). That is a stronger claim than any single number.

**Wounded, not killed** — and the distinction is load-bearing, so it is stated
with what backs it rather than as a hedge:

* *Why not killed.* 75 of 94 graded objects matched. Both attacks that could
  have made §2 categorically the wrong **kind** of model failed to land (§3a).
  Two independent readers derived **9 of 10** of the contested cells
  symbol-for-symbol identically (§6d) — §2's text is reproducibly applicable.
  And **c2 is consistent**; every inconsistency found is in §2's *prose* (§6c).
* *Why not cleared.* §2 is wrong about what a virtual call uses and about what
  triggers the vtable; **R1 is internally inconsistent on its face**, provable
  without compiling anything (§6b); and the two families that matter most are
  guardable only from **source**, which is not the channel R3 consumes (§3f).

The repair is bounded and identified. The predicate as **written** is not
shippable, and no number in this document should be quoted as if it were.

---

## 0. Provenance, and two warnings that invalidate naive comparison

### 0a. Four dc3 revs are in play. No cross-rev comparison is valid.

| rev | where it is recorded | what was measured at it |
|---|---|---|
| `13b583df` | `PHASE7_PLAN.md` session | the 172-cell fit, the 871-TU census |
| `51fb5b73` | this lane's prereg (`fdacb84`) | the frozen DEV/HELDOUT draw |
| `fbf097a5` | `pipeline/glgraph.py` docstring witness | the `.gl` edge-record witness |
| **`9ad5c4c8`** | **`../dc3-decomp` HEAD as of 2026-08-04** | anything measured from here on |

Every number in this doc names the rev it was taken at. A figure quoted from
one rev against another is not evidence, and the prereg says so.

### 0b. The branch was rebased. Pre-registration hashes have moved.

The lane's whole Part-2 discipline rests on *predictions committed before the
first compile*. That ordering is intact — the rebase preserved commit order —
but the hashes cited inside `axes1/RESULTS.md` and `axes2/RESULTS.md` no longer
resolve. The mapping, recorded once, here:

| cited in the results files | after rebase onto `cfd972c` | what it pins |
|---|---|---|
| `a5f355c` | **`fdacb84`** | the prereg |
| `0723c8d` | **`2c25adf`** | this doc's skeleton, before any number existed |
| `3401ffb` | **`b163020`** | axes1 predictions (A1/A5/A6/A7/A8) |
| `7e49cc3` | **`9c074ed`** | axes1 cell tree (59 `.cpp` / 53 `.h` / 43 `spec.json`) |
| `bac2372` | **`4418022`** | axes2 predictions + all 35 cells |

`git log --format=%ci` on those five confirms each predates the first compile
of the axis it governs. **The ordering claim is verifiable; the hashes as
printed in the results files are not.** Do not "fix" the results files — they
are dated records.

---

## 1. The gate, in one table

| part | population | discipline |
|---|---|---|
| Held-out PROC-set prediction (D1(b)) | 20 real workload TUs, seed-161 draw, frozen in the prereg | predictions committed as a git object **before** any truth artifact is read; one shot; no re-fitting |
| Structural axes (the ones 172 cells could not vary) | 9 axes, ≥4 designed cells each | per-cell predictions hand-derived from §2's text and committed **before** the axis's first compile; violations count only after an independent re-derivation |
| Warning-channel cross-check (D1(a)) | DEV TUs + probe cells | reported, not gated; attributes Part-1 misses to a half |

Incumbents registered as controls (never a bare threshold): **never-emit**
(~93 % per-body accuracy on this workload) and **emit-everything** (the port's
current behaviour). The predicate must beat both on the same universe by
≥ 2.0 pp, or it is refuted as a model regardless of anything else.

## 2. Held-out gate — result

**PENDING, and the reason is not "not yet reached".** Predictions commit: _not
made_. Truth read: _not permitted_ — the quarantine of the 20 HELDOUT TUs is
**still in force** and has been honoured throughout.

| metric | registered floor | measured |
|---|---|---|
| micro-F1 | ship ≥ 0.80; refuted < 0.50 | PENDING |
| micro-accuracy vs both incumbents (same universe) | ≥ +2.0 pp over each | PENDING |
| micro-precision | ship ≥ 0.95 | PENDING |
| per-TU exact sets (of 20) | reported only | PENDING |
| F1 excl. synthesized-name families | attribution only | PENDING |

**Why it is unrun, stated plainly.** The lane ran a third child, agent
`pipeline`, to build the Part-1 prediction pipeline on the 8 DEV TUs. It was
killed mid-flight by a box-wide OOM before it wrote any report, and its work
survived only because it had written scripts to disk (recovered and committed
at `83503cf`; see §7). The pipeline is therefore built but never frozen, and
no prediction was ever committed. **The held-out population is unspent and
remains usable**, which is the one good consequence of the crash.

**Standing question this lane must not answer by drift:** Part 2 has now found
the predicate defective on multiple axes (§3). Running Part 1 would measure a
predicate already known to be broken. That is a reason to reconsider the
*order* of work, not a reason to skip the gate and not a reason to relax it.
The prereg's one-shot rule stands; if a future lane runs Part 1, it runs it
once, against predictions committed first, on these 20 TUs and no others.

## 3. Structural axes — results

All 9 axes compiled and graded. Predictions were committed per axis before that
axis's first compile (§0b). **Counts are reported five ways and never
aggregated into a pass rate** — a predicate that survives 94 objects is not
thereby 94/94-correct, because each violation is a *class*, not an instance.

| axis | cells | objs | MATCH | VIOLATION | AMBIGUOUS | INSTR-FAIL | agent |
|---|---:|---:|---:|---:|---:|---:|---|
| A1 header inclusion depth | 8 | 8 | 8 | 0 | 0 | 0 | axes1 |
| A2 template instantiation | 9 | 9 | 7 | 1 | 1 | 0 | axes2 |
| A3 virtual/multiple inheritance, thunks | 8 | 8 | 1 | 4 | 3 | 0 | axes2 |
| A4 anonymous namespaces | 9 | 9 | 6 | 1 | 2 | 0 | axes2 |
| A5 static/inline/extern "C" crossings | 9 | 9 | 8 | 0 | 1 | 0 | axes1 |
| A6 multi-TU shared header | 8 | 17 | 16 | 1 | 0 | 0 | axes1 |
| A7 pragma-created roots | 10 | 10 | 10 | 0 | 0 | 0 | axes1 |
| A8 PCH `/Yc` `/Yu` | 8 | 15 | 14 | 1 | 0 | 0 | axes1 |
| A9 vtable kept without kept ctor (D6) | 9 | 9 | 5 | 4 | 0 | 0 | axes2 |
| **total** | **78** | **94** | **75** | **12** | **7** | **0** |  |

**The 12 were CANDIDATE violations; guard 1 has now ruled on 10 of them** (§6d).
Per the prereg a violation scores only after an independent re-derivation by an
agent that has seen neither the truth nor the first agent's prediction.
**Outcome: five demoted, V6 = 5 — or 4 under the most hostile defensible
reading — against a registered interval of [0, 3].** Six further cells
(`a2_04`, `a3_01`, `a3_02`, `a3_08`, `a4_05`, `a4_06`) await **guard 2** and
could take V6 to 7; **A3 and A4's verdicts are not final until they land**
(§6e).

Five of the demotions are **not** clean passes: they regrade to **`MATCH on
§2's domain + GAP`** — §2's rule held where it applies, *and* c2 emits code
COMDATs §2's text cannot produce at all (§6g). **The demotions do not rescue
§2; they relocate half of axes2's findings from "§2 is wrong" to "§2 is
silent."**

### 3a″. One correction to axes1's per-axis object split

axes1's `RESULTS.md` prints **A6 = 18 objs** and **A8 = 14 objs**. Re-counted
by the lead from `grades.json`, which is machine-readable and authoritative:
**A6 = 17** (8 cells: seven 2-obj cells plus `a6c7`'s three TUs) and
**A8 = 15** (8 cells: `a8c1_yc_no_refs` is a `/Yc`-only cell with a single
`pchgen.obj`, the rest are 2-obj pairs).

The two errors are off-by-one in opposite directions, so **the 59-obj total,
the 56/2/1 verdict counts, and every violation are unaffected**. The table
above carries the corrected split; `axes1/RESULTS.md` is left as printed,
per §0b's rule that dated records are not retrofitted.

`grades.json` reproduces axes1's headline exactly: 59 records, **MATCH 56,
VIOLATION 2, AMBIGUOUS 1**, with the two violations at
`a6c5_shared_vtable_one_tu_constructs/tu2.obj` and
`a8c5_extern_def_in_pch/user.obj` — the two cells the write-up names.

### 3a′. Evidence quality is not uniform across the two agents — note before quoting the table

Re-verified by the lead directly from the raw observation files, not from the
agents' prose:

* **axes2's instrument controls all reproduce**: 35/35 cells compiled
  (`cl_rc == 0`), `IMAGE_SCN_CNT_CODE` vs `.text`-name section selection agree
  **35/35**, **zero** cells with non-COMDAT code sections, and
  `code_leaders == textname_leaders` on **35/35**. The `a9_05`/`a9_06` minimal
  pair reproduces exactly: 5 leaders vs 1, with `a9_09`'s positive control at 6.
* **But axes2 has no machine-readable grades file.** `observed.json` carries
  observations only; its five-way verdict counts exist **only in the prose of
  `RESULTS.md`**, graded against predictions that are themselves prose in
  `PREDICTIONS.md`. axes1, by contrast, committed `grades.json` alongside
  `results.json`.

So the axes2 half of the table above is **observation-verified but not
grading-verified**: the lead can confirm what c2 emitted, and cannot
mechanically confirm the predicted-vs-observed comparison without re-deriving
the predictions. That is precisely what guard 1 is doing for the 10 axes2
candidate violations (§6) — which means the guard is load-bearing here in a
second way the prereg did not anticipate, and the 6 AMBIGUOUS and 19 MATCH
cells in axes2 remain prose-only either way.

This is recorded as a limitation of the evidence, not as a doubt about the
agent. Nothing in the observations contradicts its account.

### 3a. The shape of the result, in one sentence

Across 94 objects the evidence supports *"§2 is missing specific distinctions
about what a virtual call uses and what the vtable trigger is"*, **not** *"§2
is broadly fitted"*. The two attacks that could have made it categorically the
wrong *kind* of model both came back **negative**:

* **The root set is per-TU, not per-process** — CONFIRMED, not refuted. Six A6
  cells put 2–3 TUs sharing a header into a *single* `cl.exe` invocation.
  `a6c6` (external non-COMDAT definition in the shared header, referenced by
  **neither** TU) emits `?hc@@YAHH@Z` into **both** objs; `a6c3` with the
  command-line order reversed is **obj-for-obj identical** to `a6c2`. A
  per-process root set would have emitted into the first obj only.
* **§2's root list needs no pragma clause** — CONFIRMED. All 10 A7 cells clean.
  `#pragma comment(linker, "/include:...")` on an unreferenced function creates
  **no root**; the directive reaches `.drectve` and is the *linker's* problem.
  `#pragma init_seg` moves the section and never the name set.

These are real negative results and they **narrow** the problem. Do not let the
dramatic violations be generalized into a verdict the other 75 graded objects
contradict.

### 3b. The finding that decides the lane: a virtual call ODR-uses the vtable SLOT, not the definition

Discovered by **axes1** on `a6c5 tu2` and pinned with four separating probes;
independently replicated by **axes2** on `a9_04`/`a9_06`, which ran without
sight of axes1's work.

| call site, no constructor of `C` kept in the TU | `C::v` emitted? |
|---|---|
| `pc->v(x)` — virtual dispatch | **no** |
| `pc->nv(x)` — non-virtual member | yes |
| `pc->C::v(x)` — qualified, devirtualized | **yes** |
| `&C::v` in a kept data initializer | **no** — emits the vcall thunk `??_9C@@$B3AA` instead |

`f2` vs `f1` shows this is not "member calls don't propagate". `f3` vs `f1`
shows it is not about the function, the class, or the header: the *same*
definition, called with the *same* syntax minus dispatch, is kept.

**This is a false-positive class** — §2 predicts Emit, truth is Skip — which is
exactly what the **V3 micro-precision ≥ 0.95** gate exists to stop, and virtual
dispatch into a class the TU never constructs is not an edge case in dc3.

**Evidential weight, stated so it is not double-counted:** axes1 discovered the
mechanism; axes2 replicated it. A cell that re-finds a known mechanism is
weaker evidence than the cell that found it. A6 and A9 must not be counted as
two independent refutations of one clause — that weighting question is put to
guard 1 explicitly (§6).

**Magnitude over the real workload: PENDING** (§5). This is the number that
decides whether #161 needs a clause or a replacement, and the lane does not
have it yet. It is named here as unmeasured rather than left absent.

### 3c. The precision figure, restated with its scope

axes1's original write-up said lane B's precision-1.00/recall-0.928 warning
channel "is NOT impeached; that channel is simply out of scope for this class."

That is **true about the channel and misleading about the predicate.** The
channel is correct: `/Wall` on `a6c5 tu2` is silent about `v` because c1xx did
not remove `v` — its body is in the `.ex` (3602 B) and its name in the `.gl`.
**c2 dropped it.** The channel reported accurately on what it measures.

The honest form, which is now the wording in the results file:

> **Lane B's precision 1.00 holds conditional on a population that EXCLUDES
> virtual-slot ODR-use, and the size of that exclusion is unmeasured.**

A false-positive class that sits *outside the measurement* while being *common
in the workload* is precisely what a headline precision of 1.00 conceals. This
project's standing rule is that absence must never read as success; an
unmeasured excluded class is that rule's exact shape.

### 3d. R1 admits no consistent reading — a defect in the statement, not just a cell

`a5c1` and `a5c4` are **jointly unexplainable** by any single reading of §2's
root clause R1:

* **Reading A** (the head "external non-COMDAT linkage" governs; `inline` ⇒
  COMDAT ⇒ not a root): gets `a5c1` right, `a5c4` wrong.
* **Reading B** (the dash-clause is an independent enumeration of root
  spellings): gets `a5c4` right, `a5c1` wrong.

**axes1 kept its registered grade and reported its stronger evidence
separately, rather than moving its own goalpost. That was the correct call and
it is the guard working as designed from the biased agent's own side.**

#### The Selection-byte argument, corrected — guard 1's version supersedes axes1's

axes1 argued that Selection = 2 puts `cand`/`cand2` **outside** R1. That
argument **presumes R1's biconditional** — that COMDAT-ness settles rootness —
which is the very thing in dispute.

Guard 1's reading is sharper and replaces it. The byte does not confirm that
premise, **it refutes it**: those symbols are simultaneously **COMDATs** *and*
**unreferenced-yet-emitted**, i.e. roots. Therefore:

> **R1's head is FALSE as a rootness criterion. c2 has roots that are COMDATs.**

That is a stronger and more useful finding than the one it replaces, and it is
the form that should be quoted.

The byte remains admissible only as **corroboration, never as an interpretation
of §2** (§6c) — it observes what c2 did, and letting c2's output define §2's
meaning would make §2 unfalsifiable.

Independently of the count: **a clause that cannot be read consistently is a
defect in §2 regardless of which cell breaks.**

### 3e. A8 — and why it is a genuine win for R3, not just a violation

`a8c5`: an external, non-inline, out-of-line definition inside a PCH header is
emitted in the `/Yc` TU and **not** in the `/Yu` TU. Three controls isolate
precompilation, not source (`/Yu` no, no-PCH yes, `/Yc` yes).

The IL resolves the attribution cleanly and in c2's favour: `?ea@@YAHH@Z` is
**absent from the `/Yu` TU's `.gl`**. c1xx never hands it over, so **c2's emit
behaviour is not contradicted**. What is contradicted is §2's clause *as a rule
for enumerating roots from source text* — a definition textually present in the
TU is not necessarily a root, because precompilation assigns definition
ownership to the `/Yc` TU.

**The R3 consequence, which belongs here and not buried in a cell result:**
§2's existing "`.gl` presence is a necessary condition" **already guards this
class at zero cost**. An R3 that intersects its predicted set with the `.gl`
names is *already* immune to the entire PCH class. That is the cheapest
possible fix, it uses an instrument the project already has and has already
known-answer-gated, and it costs nothing if R3 is built over the IL (which the
plan's R3 rung says it is).

### 3f. What the `.gl` channel can and cannot guard — the boundary is proven

| class | guarded by `.gl`? | proof |
|---|---|---|
| A8 PCH root-ownership | **YES**, free | `ea` absent from the `/Yu` `.gl` |
| Family 1 synthesized thunks (`??_D`, `??_E…W3…`, `??_9`) | **YES** | detector flagged 5/5, 0 false neg, 0 over-flags |
| Family 4 `??__E` fires without emission | **YES** | 1/1 |
| Family 2 vtable trigger is not "a kept constructor" | **NO — provably** | minimal pair below |
| Family 3 virtual call is not an ODR-use | **NO — provably** | minimal pair below |

The proof is a minimal pair inside axes2's own grid:

```
a9_05 .gl (minus anchor)  ==  a9_06 .gl (minus anchor)
a9_05 emits 5 functions.      a9_06 emits 1.
```

**Identical front-end name tables, emitted sets differing by four functions.**
So for the destructor/virtual-call families the discriminator is *not* in the
`.gl` names.

#### RETIRED — the discriminator IS in the IL, and it is a single byte

The conclusion this section originally drew — *"R3 must get it from source; the
demonstrated detectors are source-side greps, not the IL-side version R3 will
need"* — **is superseded by §5's measurement.** The `.ex` operand stream
distinguishes the two edge kinds directly:

    direct call / reference     26 <token>
    VIRTUAL DISPATCH            67 <vtable-byte-offset> <token>

**Known-answer gated 12/12** against real objs — including axes1's four
mechanism probes, **the actual graded violation obj `a6c5/tu2`**, and a
`p_ctor` control in which the *only* change from `f1` is a kept constructor and
the entire class disappears.

So the honest statement is now: **`.gl` names are insufficient; the `.ex` is
sufficient.** Family 3 is guardable — and better than guardable, *predictable* —
from the exact channel R3 consumes, and **R3 needs no fail-closed source-side
guard for it**. axes1's "the detector that works is syntactic and source-side"
was correct as far as its evidence went and is superseded.

Family 2 (the vtable **trigger**) is not addressed by this byte and remains
demonstrated only from source. **That gap stands.**

Union refusal rule over axes2's 35 cells: refuses **10 of 35**, and every one
refused is a violating cell — **0 false negatives, 0 over-flags** on that
population. Good in both directions, on 35 probe cells, which is not the
workload.

## 4. Warning-channel on real headers — result

**PARTIAL, and reported not gated** (prereg Part 3).

Measured so far: on `a6c5 tu2` the channel is **correctly silent** — it reports
what c1xx removed, and c1xx removed nothing. `/Wall` reported only
`C4514: 'C::C' : unreferenced inline function has been removed`. The channel's
accuracy is not in question; its **scope** is (§3c).

The full DEV-plus-probe pass registered in prereg Part 3 is **PENDING** — it
was the `pipeline` agent's work and died with it. `work/emitpred/dev/warn/`
holds captured `/Wall` stderr for all 8 DEV TUs, unanalysed.

## 5. Magnitude of the virtual-slot false-positive class over the workload

**MEASURED.** Full record: `work/emitpred/MAGNITUDE.md` (`1ac16bc`).

Measured over **850 TUs** at dc3 **`9ad5c4c8`** — 878 − **21** quarantined − 7
that produce no obj. (The quarantine is **21, not the prereg's 20**: the frozen
draw named *basenames*, and `FxSendEQ.cpp` resolves to two workload paths;
both were quarantined conservatively.)

| | count | denominator |
|---|---:|---|
| **TUs with ≥ 1 false positive of this class** | **289** | 850 — **34.0 %** |
| **false-positive name-instances** | **649** | — |
| **distinct decorated names** | **331** | — |
| total emitted code COMDATs, same 850 TUs | 174 410 | — |

Composition: **481** ordinary virtual members, **166** `??_G` scalar-deleting
destructors (the `delete p`-through-a-base-pointer form axes2 pinned on
`a9_06`), **2** provable detector artifacts — left in the count rather than
quietly deducted.

### 5a. The V3-relevant fraction — and why V3 still cannot be quoted

| | |
|---|---|
| this class as a share of the smallest possible predicted set | **0.371 %** |
| ⇒ ceiling on V3 micro-precision from this class alone | **≤ 0.99629** |
| size this class would need to drag V3 to 0.95 | **9 179 instances — 14.1×** the measured figure |

The bound **needs no root model**: every flagged name has an *emitted* referrer,
so it is definitely a kept definition and §2's Propagation clause definitely
predicts it, and it is definitely not emitted; `TP` is bounded by truth at
`|E|`. A bound that holds under *every* root model beats a point estimate under
a guessed one.

**But V3 itself remains UNMEASURED**, and no V3 number may be quoted from this
work. §2's full predicted set is not computable — Part 1's root model was never
finished (§2) — so `TP` and the non-virtual-slot part of `FP` are both unknown.
**This class does not break V3, and it is not the thing that will.**

### 5b. Verdict — A CLAUSE, not a replacement

**§2's Vtable rule is already right. Its Propagation clause is wrong about
exactly one edge kind.** The repair:

> *"a call anywhere in the pre-optimization body"* → *…**except that a virtual
> dispatch ODR-uses the vtable SLOT, not the definition**.*

The **distribution supports this reading**: broad but thin — 34.0 % of TUs
touched, **median 2** instances per affected TU, max 17; 331 names produce 649
instances with the **top 10 names producing 28.5 %**, headed by shared
header-inline virtuals and container-node deleting destructors that everything
includes. That is the shape of a **systematic construct one clause fixes
everywhere at once**, not of an accident.

### 5c. Detector error rate — four independent checks

1. **Provable artifacts: 2 / 649 = 0.31 %.** A vtable slot can hold only a
   virtual, a `??_G`/`??_E`, or a `??_9`; exactly two flagged names demangle to
   something else.
2. **Slot-offset hygiene, and neither property is used by the detector:** of 623
   flagged cells carrying a slot byte, **0** have `off % 4 ≠ 0`, and **0 of 319
   distinct names** disagree about their offset across TUs. Under a coincidence
   model, ≈ `4⁻⁶²³`.
3. **Raw `67`-edge artifact rate is 10.7 %** workload-wide — reported so the
   class rule's filtering down to 0.31 % is *visible* rather than assumed.
4. **Hand check, seeded n = 20: 20/20 confirmed**, with the dispatching call
   site located by hand in the dc3 sources for 10 of them. Stated plainly by its
   author: n = 20 buys only a **13.9 %** one-sided upper bound — checks 1–3 are
   what make the number tight.

### 5d. A defect found in the recovered pipeline, and routed around

`pipeline/model.py`'s `named_bodies` binds a name to only **66.3 %** of `.ex`
body segments, and the recovered rule that folds each unnamed segment onto the
nearest *preceding named* one is **wrong — graded correct on 1 of 14 842**
gradeable segments. The headline uses the **strict** rule instead; soundness is
argued from **`E ⊆ U` holding on 174 404 of 174 410** emitted names.

**This is why the earlier partial figure of 1 049 instances / 312 TUs is
superseded and must not be re-quoted** — it was the folded variant.

**Residual, unmeasured:** 33.7 % of segments have no recoverable owner, so the
headline is a **lower bound**. Five of the eight items the measurement names as
unmeasured (§6 of that file) can only make the true figure **larger**.

## 6. Guard 1 — independent re-derivation

**IN PROGRESS.** Per prereg guard 1, all 12 candidate violations are scored
only after a second agent, given **only the cell sources and §2's text** — not
the compiled truth, not the first agent's prediction — reproduces the
prediction. Disagreement ⇒ `AMBIGUOUS`, automatically, *regardless of which
derivation truth favours*. Resolving a disagreement by picking the reading
truth prefers is the goalpost-move the guard exists to prevent.

The guard is held to the same commit-before-look discipline as axes1 and
axes2: it writes `work/emitpred/guard1/DERIVATION.md`, the lead commits it,
and only then may it read any result artifact.

Three questions put to it, each of which changes a number in this doc:

1. **Is axes1's violation count 2 or 3?** — the `a5c4` Selection-byte question
   (§3d). Is c2's own post-hoc classification evidence about what §2 *means*,
   or only about what c2 *does*?
2. **Does R1 admit a consistent reading at all?** If not, that is a defect
   independent of the count, and it needs a minimal textual repair.
3. **V6 weighting** — A6 and A9 as two axes broken, or one mechanism found
   twice (§3b)?

### 6a. The lead leaked truth into the guard's brief — recorded as a lane defect

**This is the lane's own methodology failing, and it is recorded here because
this doc is where the gate's integrity is accounted for.**

The tasking brief the lead wrote for guard 1 stated that the measured Selection
bytes make `a5c4`'s `cand`/`cand2` both **Selection = 2**. *A COMDAT Selection
byte exists only for a COMDAT present in the obj.* That sentence therefore told
the guard that `cand` and `cand2` **are emitted** — which is exactly the truth
its independent re-derivation existed to reach unaided — in a brief whose only
purpose was to keep truth out.

**Attribution: the lead's leak, not the guard's contamination.**

The guard caught it, and its handling is the reason the exercise survives:

* disclosed it loudly and unprompted, at the top of its own frozen file;
* marked `a5c4` **CONTAMINATED-GUARD**;
* identified **`a5c1` as the load-bearing clean cell _before_ looking** at
  anything;
* **pre-committed both contested rulings inside the frozen file**, so the leak
  could not steer the outcome after the fact;
* left `cells/*/spec.json` unread — not on the quarantine list, but plausibly
  carrying expected sets, and treating an unlisted file conservatively is the
  right default when the list was written by someone who had already leaked.

**Consequence for scoring:** `a5c4`'s agreement with truth is now **weak
evidence** and is graded as such. `a5c1` is unaffected and carries R1's case.

**Why this does not sink the guard:** its strongest finding is immune to the
leak — see §6b, which is reached without compiling anything and without
reference to either contaminated cell.

### 6b. R1 is internally inconsistent on its face — no experiment required

The guard's most valuable result, and it outranks everything the 94 objects
produced on this clause:

> An **out-of-line member definition marked `inline`** is simultaneously
> *"any out-of-line definition"* — a root, by R1's list — and **COMDAT** — not
> a root, by R1's head.

No reordering of the clause fixes it. It is reached through R1's *non-member*
items, so it is **independent of `a5c1`/`a5c4` entirely**, and therefore
independent of the lead's leak.

This reframes the whole R1 question. `a5c1`/`a5c4` stop being a puzzle about
*which of two readings is right* and become **two more instances of an
intersection §2 cannot resolve under either reading**. A defect provable from
the text alone is stronger evidence than one that needed 59 objects to expose,
and it cannot be answered by re-running anything.

### 6c. Rulings the guard fixed in advance, and why each is right

* **The Selection byte is corroboration, not interpretation.** §2's authority
  over its own words is not delegated to c2's output — otherwise **§2 could
  never be violated at all**, since every disagreement would be re-read as "§2
  meant whatever c2 did", dissolving this lane's own refutations along with
  everything else. It therefore **cannot convert AMBIGUOUS into VIOLATION**.
  Its legitimate use is the *textual* argument that Reading B makes §2
  self-inconsistent.
* **V6 counts axes, not mechanisms.** Redefining a registered unit after
  discovering that two axes share a cause is a goalpost-move that happens to
  **lower** the score — and it is still an error when the direction is
  unflattering. **A6 and A9 each count.** The interpretation must then state
  both facts together: `a6c5` and `a9_04`/`a9_06`/`a9_07` are **one defect with
  one repair**, *and* independent cross-agent replication **raises** confidence
  the mechanism is real while **lowering** the number of distinct repairs §2
  needs.
* **[REACH] vs [GAP].** Carried into the final scoring. *"§2 produces a definite
  prediction and it is wrong"* (REACH) and *"§2 has no clause that could produce
  this symbol at all"* (GAP) are different failures and **must not score the
  same**. Falsifiable advance call, recorded before the guard looked: **all four
  A3 cells, if they break, break only by GAPs.**
* **`a9_05` is split from `a9_04`/`06`/`07`.** Under-inclusive vtable forcing
  and over-inclusive propagation are **opposite** defects; a merged report
  would hide one.

### 6d. V6 RULED — 5, or 4 hostile. Above the registered interval either way, and a lower bound

Guard 1 demoted **five of the twelve** candidate violations and the result still
lands outside its own pre-registration.

**V6 is reported BOTH ways, because guard 1 exposed the single interpretive
call that moved its own headline number _up_** — which is exactly what a guard
is for, and it is published rather than resolved silently:

| reading | V6 | basis |
|---|---:|---|
| **guard 1's** | **5** | the prereg's AMBIGUOUS clause triggers on **inter-agent** disagreement, and axes1's registered *primary* prediction was identical to guard 1's — so **A5 is broken** |
| **the hostile alternative** | **4** | a pre-registered **rival** reading counts as a derivation disagreement, so **A5 is unbroken**; A6, A8, A2, A9 remain |

| | value |
|---|---|
| V6 registered point estimate | **1** |
| V6 registered interval | **[0, 3]** |
| **V6 ruled** | **5**, or **4** under the most hostile defensible reading |

### The robustness is the finding — lead with it, not with either number

**Both readings exceed the registered interval [0, 3].** So the headline — *the
pre-registration missed on the high side* — is **robust to the one interpretive
call that inflates it.**

That robustness is worth more than the extra point. **A result that survives its
own author's most hostile reading is the strongest thing this lane can
publish**, and it is a stronger claim than V6 = 5 asserted flatly.

**Missed on the high side, after the most deflationary accounting the guard
could defend.** The five demotions make the result **more** credible, not less.
It is not to be softened, not to be averaged against the raw 12, and the
demotions are not the headline.

**5 is a lower bound twice over, and this travels with the number wherever it
is quoted:**

1. **Six cells remain unpromoted** for want of a frozen independent derivation
   (`a2_04`, `a3_01`, `a3_02`, `a3_08`, `a4_05`, `a4_06`) — see §6e. Their
   resolution decides whether V6 is **5 or 7**, and whether **A3 and A4** break.
2. **The guard's construction is asymmetric.** It re-derived only *claimed*
   violations. **A cell graded MATCH in error is structurally invisible to it.**
   Nothing in this exercise could have caught a false negative, so the true
   count cannot be lower than 5 and could be higher for a reason no one looked
   for.

**Reported in the same breath, because it is a real result in §2's favour and
suppressing it would be the mirror of the error this lane is guarding against:**
on **9 of 10** axes2 cells the guard's quarantined derivation is
**symbol-for-symbol identical** to axes2's, and **AMBIGUOUS fires nowhere in
that set**. Two readers, working independently from the same prose, reached the
same predictions. **§2's text is reproducibly applicable** — its defects are
specific, not a general looseness.

### 6e. The guard flagged six cells that would RAISE its own number

Guard 1 identified six cells whose promotion would push V6 from 5 to 7, and
**declined to promote them** because it had no frozen derivation for them. That
is the behaviour that makes a guard worth having, and it is credited here
explicitly.

It also **disqualifies guard 1 for those six**: its mechanical predicted-vs-
observed cross-check ran over all ten cells, so it has now seen their truth and
cannot produce an independent derivation. **`guard2` was spawned** to derive
them under the same freeze-before-look discipline, carrying guard 1's
*procedural* rules (one committed primary per cell, REACH/GAP tagging,
falsifiable advance calls) and **none of its findings**. Its brief was scrubbed
of everything entailing symbol presence — Selection bytes, sizes, counts,
section assignments — and it was asked to flag any leak it finds in the
tasking, since the lead has already made that mistake once (§6a).

**Status: IN PROGRESS.** V6 is **5 (or 4 hostile) pending guard 2**, and may become **7 (or 6)**.

### 6f. c2 is consistent — the inconsistency is purely textual

**This reframing changes what the repair is, which is why it is here and not
buried in a cell result.**

Nothing found in 94 objects plus guard 1's probes shows c2 behaving
inconsistently. Every inconsistency established by this lane is in **§2's
prose**. c2 is applying *some* rule uniformly; §2 describes that rule with words
that cannot all be true at once.

**The evidence that isolates it — the strongest single item in this lane.**
Guard 1's post-hoc probes `p1` vs `p3` are a minimal pair: **both `extern "C"`,
both `inline`, both unreferenced, differing by exactly one declaration**, and
the symbol flips. That isolates the variable instead of sampling around it,
which is what the 172 fitted cells and the 78 axis cells could not do for this
clause.

**The rule that emerges:** only the **storage-class `extern`** confers rootness
— **never the language-linkage `extern "C"`**. This explains **all 9 A5 cells,
guard 1's 3 probes, and `a4_07`** with one statement.

**Caveat, kept exactly as the guard stated it and not to be dropped when this is
quoted:** that is a **repair, not a reading**. Any rule separating storage-class
`extern` from language-linkage `extern "C"` **still contradicts R1's list**,
which enumerates `extern "C"` as a root spelling. So the finding gives §2 a
*correct* rule to adopt; it does **not** rescue §2-as-written, which remains
without a consistent reading (§6b).

### 6g. Five cells graded `MATCH on §2's domain + GAP` — two groups, and BOTH halves travel

**The lead first instructed that these be refiled as plain confirmations. That
instruction was wrong** and, followed literally, would have produced **the
mirror image of the error guard 1 had just caught** — deleting a real finding in
the direction that flatters §2, immediately after correcting a filing that
flattered the lane's thesis. Corrected in
`work/emitpred/guard1/RULING_ADDENDUM.md` (`9aa5d76`).

**The grade is two-part, and the halves have different owners:**

| group | the CONFIRMATION half — owner **#161** | the GAP half — owner **#152** |
|---|---|---|
| `a3_03`, `a3_04`, `a3_05`, `a3_06` | §2's vtable closure **held**: the **wide** reading of "every virtual of C" (inherited included), applied **per base subobject**, with the `??_G` rider — across MI, virtual bases and a diamond | c2 emits `??_DB@@QAAXXZ`, `??_DC@@QAAXXZ`, `??_DD@@QAAXXZ` and `??_ED@@W3AAPAXI@Z` — **code COMDATs §2's text has no clause capable of producing** |
| `a4_09` | §2's **pre-optimization** clause confirmed on the hardest case available: the dyninit thunk's **propagation survives the thunk's own elimination** — a `static` reachable only through a `??__E` c2 does not emit is emitted anyway | c1xx **names** that `??__E` thunk in the `.gl` and c2 then does **not** emit it; §2's root 4 gives no rule for which initializers produce one |

**Dropping the gap half would delete a coverage finding. R3 must emit those
symbols or refuse, and that is outstanding work** — it does not become
unnecessary because §2's vtable rule was vindicated on the same cells.

`a4_09` also sharpens §2's wording: *"removed" must mean **never kept by the
fixpoint**, not **absent from the output**.*

**Guard 1's line, to be carried verbatim wherever these five are quoted:**

> **The demotions do not rescue §2; they relocate half of axes2's findings from
> "§2 is wrong" to "§2 is silent."**

"Wrong" and "silent" are different defects with different repairs. **Neither is
"fine".**

## 7. Recovered instrument: the `.gl` records carry the reference graph

Found in the killed `pipeline` agent's `glgraph.py`, unreported until
`83503cf`. Each `.gl` symbol record is

    <kind> <token> <sep> <name> <fixed header ...> ( <token> <refcount> )*

so **c1xx writes, per symbol, the complete list of symbols that symbol
references, with use counts** — for *data* symbols too, so a static table of
function pointers links to everything whose address it takes, and **a vftable
links to its slots**. That list is the reference relation §2's fixpoint runs
over, available from the front-end side alone.

This matters here for two reasons:

1. It is a c1xx-side channel **§2's own text does not nominate**. The plan
   nominates the `.gl` *name table* as a necessary condition and says nothing
   about the `.gl` carrying *edges*.
2. "A vftable links to its slots" is the shape the virtual-slot class needs:
   the discriminator may be "the only `.gl` edge reaching F is through a
   vftable record whose class has no kept ctor/dtor in this TU". **That is a
   hypothesis from a docstring, not a measured fact**, and it is being tested,
   not assumed.

Carried caution, verbatim from the author: `ref_graph()` deliberately
**over-approximates** — it scans the payload for any operand token the symbol
index resolves, and a fixed header field can alias a token value. That is the
safe direction (the fixpoint must not lose an edge). **Do not tighten it
without a known-answer gate.**

## 8. Verdict

**PENDING.** It will be one of the five words fixed in the prereg
(SHIP-CANDIDATE / SURVIVES-NOT-SHIPPABLE / REFUTED-ON-REAL / DECLINE /
INSTRUMENT-FAIL), with the numbers that forced it.

What is already settled and cannot be revised by anything still outstanding:

* **#161 does not ship into R3 as stated.** Part 2 alone establishes that §2's
  text is wrong about the vtable trigger and about what a virtual call uses,
  and that **R1 is internally inconsistent on its face** (§6b) — provable from
  the text, needing no experiment and no cell. SHIP-CANDIDATE requires "Part-2
  axes all clean-or-guardable"; two families are guardable only from *source*,
  which is not the channel R3 consumes.
* **V6 = 5 — or 4 under the most hostile defensible reading — above the
  registered interval of [0, 3] either way, and a lower bound twice over**
  (§6d). The pre-registration expected the predicate to break on about one
  axis. It broke on at least four, after deliberate deflation, and **the
  headline survives its own author's most hostile reading** — which is a
  stronger claim than any single number.
* **The predicate is not thereby refuted as a model.** 75 of 94 graded objects
  matched; both categorical-shape attacks failed to land; **9 of 10** contested
  cells were derived identically by two independent readers; and **c2 itself is
  consistent** — the inconsistency is in §2's prose (§6f). Every AMBIGUOUS
  cluster is a one-line wording repair with an unambiguous direction.
* **The false-positive class is cheap for precision and expensive for
  coverage** (§5): ≤ 0.598 pp of V3 micro-precision — it **cannot alone** take
  V3 below 0.95 — but present in **36.7 %** of TUs, so a fail-closed R3 that
  refuses on the construct refuses better than one TU in three. **Neither
  framing may be quoted without the other.**

What is outstanding, and what each would change: **guard 2** (§6e) decides
whether V6 is **5 or 7** and whether **A3 and A4** break — the highest-value
remaining work; the magnitude write-up's detector error rate and distribution
(§5) complete the clause-vs-replacement case; Part 1 (§2) is unrun and its
population is **unspent**.

**On the prereg's five words:** the evidence already excludes SHIP-CANDIDATE.
It does not support REFUTED-ON-REAL either — that verdict is defined by V1/V2
on the held-out TUs, which were never run, and it must not be reached by
substituting Part-2 evidence for a Part-1 measurement. **DECLINE** ("not
validated; #161 does not ship; no patching") is where the evidence points, and
the final word waits on guard 2.

## 9. Clean-room ledger

All channels used are black-box under ROADMAP §9.8's existing blessing:
compile-and-observe probe cells, `/Wall` C4505/C4514 stderr, `/FAsc` PROC
name sets (names only), obj byte analysis of our own captures, `.gl` reads
via the separator-aware extractors. Disassembly-derived constants adopted:
**none**. If that changes it will be disclosed here per-finding.

**Two instrument notes that would each have manufactured a false result:**

* **`/I.` and A8.** Every `/Yu` compile initially failed `C1034`. `/I.` fixes
  it; `INCLUDE=.` fixes it identically. `/I.` was then applied uniformly to all
  43 axes1 cells and the suite re-run, with the paired run kept as
  `results_noI.json` as a control: **53 of 59 leader sets byte-identical, and
  the only 6 that moved are exactly the previously-failing objs going
  `None → set`.** `/I.` is inert, so **A8 is a real result, not an
  instrument-fail** — without the control it would have been reported as one
  for 6 of 14 objs.
* **Section NAME is not section CONTENTS, and it bit again.** `a7c8`'s
  `#pragma code_seg(".mytext")` yields an **empty** leader set under a
  `.text`-name-prefix reading and the **correct** one under the
  `IMAGE_SCN_CNT_CODE` characteristic. Grading by name prefix would have
  manufactured a fabricated violation on exactly 1 of 59 objs. **Both readings
  are recorded for every obj**; axes2 checked the same thing independently and
  found the two readings agree on all 35 of its cells. This is the same
  name-as-proxy-for-contents failure mode that has already cost this project
  days elsewhere.

## 10. Side findings routed to other lanes — recorded, not chased

| # | finding | routing |
|---|---|---|
| **S1** | The COMDAT Selection byte tracks **inline-ness, not storage class**: a kept plain `static` is Selection 1, a kept `static inline` is Selection **2** with storage class STATIC. §2's corroboration says "1 for strong-linkage and kept statics" — the rule the port needs is "`inline` ⇒ 2", not "static ⇒ 1". | **lane w-r1** (the writer must reproduce this byte). Written down precisely; not acted on here. |
| **S2** | `#pragma init_seg("name")` yields a **user-chosen** section name (`.mycrt$a`). §1 of the plan rests on "the workload section vocabulary is 13 names, so C is finite and enumerable" — closed over the workload as measured, **not closed by the language**. | **Already routed** to the lane correcting the section-vocabulary claims. No action here. |
| **S3** | `&C::v` in a kept data initializer emits the vcall thunk `??_9C@@$B3AA` and **not** `?v@C@@UAAHH@Z`. §2's propagation names "an address-take" as keeping F; here the address-take keeps a *different, synthesized* symbol. | **#152 synthesis territory / the A3 axis.** Reported, not claimed. |
| **S4** | PCH changes the captured `.ex` from **3029 bytes to 125** for the same source, under both `/Yc` and `/Yu`, while the objs are correct in all three cases. Where the bodies travel under PCH is **not answered**. | **Uncharacterized, and left that way deliberately.** No verdict in this doc rests on it. Flagged because any IL-side tooling that meets a PCH build will meet this. |

## 11. Artifacts

Under `work/emitpred/` (gitignored tree; text records force-added, never objs,
`.cod`, `.pch`, `_CL_*` or captured IL):

| path | what |
|---|---|
| `axes1/PREDICTIONS.md`, `axes1/cells/` | frozen predictions + 43 cells (`b163020`, `9c074ed`) |
| `axes1/RESULTS.md`, `grades.json`, `results.json`, `results_noI.json` | axes1 grading and the `/I.` control |
| `axes2/PREDICTIONS.md`, `axes2/cells/` | frozen predictions + 35 cells (`4418022`) |
| `axes2/RESULTS.md`, `observed.json`, `il_names.json`, `detectors.py` | axes2 grading and the detectability demonstration |
| `pipeline/` | recovered Part-1 tooling: `glgraph.py`, `coff.py`, `gl.py`, `il.py`, `model.py`, `names.py`, `truth.py` (`83503cf`) |
| `dev/truth/`, `dev/shim/` | DEV-TU truth sets (truth-open) and shims |
| `guard1/` | independent re-derivation (in progress) |
| `MAGNITUDE.md` | the workload measurement (in progress) |
