# axes1 — RESULTS (Phase 2): A1, A5, A6, A7, A8 against `PHASE7_PLAN.md` §2

Agent `axes1`, lane `w-emitpred`, worktree `wt-w-emitpred`.

**Pre-registration record.** Predictions committed at **`3401ffb`**
(`PREDICTIONS.md`, `gencells.py`, `leaders.py`, `runcells.py`); the full
sources-only cell tree committed at **`7e49cc3`** (59 `.cpp` / 53 `.h` /
43 `spec.json`). Both precede the first axis-cell compile. Every predicted set
graded below was fixed at `3401ffb`; nothing in this file edits a prediction.

**Flags:** `/O1 /Oi /EHsc /GS- /c` (+ `/I.`, see instrument note I1), X360
`16.00.11886.00` `cl.exe` under wibo. **Ground truth:** the obj's code-section
COMDAT leader symbol set. **Cross-check:** `/FAsc` `.cod` PROC name sets agree
with the obj leader sets **59/59, zero disagreements** (names only; no
instruction byte was read).

**In one sentence, and it is the sentence that should be quoted from this
file:** across 59 objects the evidence supports *"§2 is missing one specific
distinction — what a virtual call actually uses"*, **not** *"§2 is broadly
fitted"* — the two shape attacks that could have made it categorically the
wrong kind of model (per-process root sets, a missing pragma-root clause) both
came back **negative**, which narrows the problem rather than widening it. Do
not let §2's one dramatic violation be generalized into a verdict the other 56
graded objects contradict.

---

## 1. Verdict counts — five-way, reported separately, never aggregated

| axis | cells | objs graded | MATCH | VIOLATION | AMBIGUOUS | INSTRUMENT-FAIL | axis verdict |
|---|---:|---:|---:|---:|---:|---:|---|
| **A1** header-inclusion depth | 8 | 8 | 8 | 0 | 0 | 0 | **CLEAN** |
| **A5** linkage crossings | 9 | 9 | 8 | 0 | 1 | 0 | **CLEAN, statement defect** |
| **A6** multi-TU shared header | 8 | 18 | 17 | **1** | 0 | 0 | **BROKEN** |
| **A7** pragma roots | 10 | 10 | 10 | 0 | 0 | 0 | **CLEAN** |
| **A8** PCH `/Yc` `/Yu` | 8 | 14 | 13 | **1** | 0 | 0 | **BROKEN** |
| **total** | **43** | **59** | **56** | **2** | **1** | **0** | **2 of 5 axes broken** |

These are counts of graded objects, not a score. A predicate that survives 56
objects is not thereby 95 % correct: the two violations are each a *class*, and
both classes are common in real C++.

Contribution to the prereg's **V6** (axes of 9 with ≥ 1 confirmed violation):
**+2 from my five** (A6, A8). V6's registered point estimate was 1, interval
[0, 3] — my half of the axis set alone lands at the top of the point estimate
and inside the interval.

---

## 2. VIOLATION 1 — A6/a8c7 class: a virtual call does not keep the virtual

**Cell:** `a6c5_shared_vtable_one_tu_constructs`, obj `tu2.obj`.
**Predicted** `{?anchor2@@YAHH@Z, ?v@C@@UAAHH@Z}` — **observed**
`{?anchor2@@YAHH@Z}`. Missing: **`?v@C@@UAAHH@Z`**.

**Clause contradicted — §2 Propagation, first sentence:**

> *F is added if an already-kept definition ODR-uses it — **a call anywhere in
> the pre-optimization body** (including statically dead branches and `catch`
> handlers), an address-take, or a data initializer.*

`tu2.cpp` is `extern C* pc; int anchor2(int x){ return pc->v(x) + sink(x); }`.
`anchor2` is a root (R1) and is kept; its pre-optimization body contains a call
to `C::v`, whose definition is available in the shared header. §2 as stated adds
`v`. c2 does not emit it.

The rest of the cell matched: `w`, `??_G`, `??1C`, `??0C` are all correctly
absent from `tu2.obj` (no kept constructor ⇒ the vtable rule does not fire), and
`tu1.obj` — which constructs `C` — matched its 6-name prediction exactly. So the
**vtable rule V is confirmed and the per-TU scope S is confirmed**; the defect
is isolated to P1's treatment of virtual dispatch.

### The mechanism, pinned by four post-hoc diagnostic probes

Post-hoc, clearly outside the graded 43, in `detect/mech/`:

| probe | call site (no constructor of `C` kept in the TU) | `C::v` emitted? |
|---|---|---|
| f1 | `pc->v(x)` — virtual dispatch | **no** |
| f2 | `pc->nv(x)` — non-virtual member | n/a; **`?nv@C@@QAAHH@Z` emitted** |
| f3 | `pc->C::v(x)` — qualified, devirtualized | **yes** |
| f4 | `PM g_pm = &C::v;` — pointer-to-member in a kept data initializer | **no** — instead the vcall thunk **`??_9C@@$B3AA`** is emitted |

f2 vs f1 shows the failure is not "member calls don't propagate". f3 vs f1
shows it is not about the function, the class, or the header: the *same*
definition, called with the *same* syntax minus dispatch, is kept.

**The corrected statement §2 needs:** *a virtual call ODR-uses the vtable slot,
not the definition.* A virtual function's definition is kept only by (i) the
vtable rule via a kept constructor, or (ii) a qualified/devirtualized call.
Taking `&C::v` keeps a `??_9` vcall thunk, not `v`.

### Why this one matters most

It is a **false-positive** class — §2 predicts Emit where truth is Skip. The
prereg gates fail-closed R3 on **V3 micro-precision ≥ 0.95**, and a name
predicted Emit that is not emitted is exactly the "silent wrong obj" V3 exists
to prevent. Virtual dispatch through a base pointer is not an edge case in the
dc3 workload (UI, rendering, `?CanSelect@UIListProvider@@UBA_NH@Z` in §5-D3 is
itself a `UBA` virtual). Any TU that virtual-calls into a class it never
constructs will be over-predicted.

### Detectability from c1xx-side observables — YES, but not by either
### channel §2 nominates

Both of §2's nominated channels **fail** on this cell:

* **The `.gl` name table does not discriminate.** `tu2.obj`'s bundle contains
  `?v@C@@UAAHH@Z`, `?w@C@@UAAHH@Z`, `??_GC@@UAAPAXI@Z`, `??1C@@UAA@XZ`,
  `??_7C@@6B@` — all present, none emitted. Consistent with §2's own framing of
  `.gl` as a necessary condition / fail-closed upper bound, but it means `.gl`
  buys nothing here.
* **The C4505/C4514 warning channel is silent about `v`.** `/Wall` on `tu2.cpp`
  reports only `C4514: 'C::C' : unreferenced inline function has been removed`.
  Attribution, and it is exact: `v`'s body is present in the `.ex` (3602 bytes)
  and its name in the `.gl`, so **c1xx did not remove `v` — c2 dropped it.**
  The front-end warning *channel* is therefore correct: it reports what c1xx
  removed, and c1xx removed nothing here.

  **[Lead correction, applied at recovery — the original wording of this
  bullet was wrong in a way this project's standing rules forbid.]** It read
  "lane B's precision-1.00/recall-0.928 figure is not impeached". That is true
  about the *channel* and misleading about the *predicate*, which is what #161
  is. The honest form: **lane B's precision 1.00 holds conditional on a
  population that excludes virtual-slot ODR-use, and the size of that
  exclusion was unmeasured at the time this file was written.** A false-positive
  class that sits *outside the measurement* while being *common in the
  workload* is exactly what a headline precision of 1.00 conceals; reporting
  the headline unqualified would have let an unmeasured excluded class read as
  success. The exclusion is measured in `../MAGNITUDE.md`; the number there,
  not this bullet, is what bears on V3.

**The detector that does work is syntactic and source-side**, demonstrated on
f1/f2/f3: classify each call site as *virtual dispatch* vs *qualified call* vs
*member-pointer take*. R3 can implement this as a fail-closed guard —
`Unknown ⇒ refuse` on any TU containing a virtual dispatch to a class with no
kept constructor — without any new instrument.

---

## 3. VIOLATION 2 — A8: `/Yu` moves an external definition out of the TU

**Cell:** `a8c5_extern_def_in_pch`, obj `user.obj`.
**Predicted** `{?ea@@YAHH@Z, ?anchoru@@YAHH@Z}` — **observed**
`{?anchoru@@YAHH@Z}`. Missing: **`?ea@@YAHH@Z`**.

**Clause contradicted — §2 Roots (1), applied per-TU (scope S):**

> *Roots: (1) every definition with external non-COMDAT linkage — plain extern,
> `extern "C"`, **any out-of-line definition** …*

`pchb.h` defines `int ea(int x){ return x*3+1; }` — external, non-inline,
out-of-line. Both TUs `#include "pchb.h"`. In the `/Yc` TU it is a root and is
emitted (`pchgen.obj` matched its 2-name prediction). In the `/Yu` TU the same
textual definition is **not** a root and is not emitted.

**Controls, all three run:**

| compile of the identical `user.cpp` | `?ea@@YAHH@Z` emitted? |
|---|---|
| `/Yupchb.h` (the cell) | **no** |
| no PCH, plain textual `#include` (`detect/a8c5_nopch/`) | **yes** |
| `/Ycpchb.h` (the generator TU, `pchgen.obj`) | **yes** |

So the cause is precompilation, not the source.

### Mechanism and detectability — YES, and by §2's own nominated channel

IL captured with the repo's own recipe (`strace -e inject=unlink,unlinkat:retval=0`,
as in `capture_triple`), `.gl` read with the separator-aware extractor
(`glnames.py`, never raw `strings`):

| bundle | `?ea@@YAHH@Z` in `.gl`? |
|---|---|
| `/Yu` TU | **absent** |
| no-PCH TU | present |
| `/Yc` TU | present |

**This resolves the finding's attribution cleanly, and it is good news for the
port:** c1xx never hands `ea` to c2 under `/Yu`, so **c2's emit behaviour is not
contradicted** — c2 emits exactly what its input supports. What is contradicted
is §2's clause as a rule for enumerating roots **from source text**: a
definition textually present in the TU is *not* necessarily a root, because
precompilation assigns definition ownership to the `/Yc` TU.

**Detector, demonstrated:** the `.gl` name table. `ea` is absent from the `/Yu`
bundle, so §2's existing "`.gl` presence is a necessary condition" already
guards this class at zero cost — an R3 that intersects its predicted set with
the `.gl` names is *already* immune. This is the cheapest possible fix and it
uses an instrument the project already has and has already known-answer-gated.

**Scope honesty:** this violation is against the *source-side* root
enumeration. If R3 is built over the IL (which the plan's R3 rung says it is —
"per segment"), this class costs nothing. It is recorded as a violation because
§2 as written enumerates roots over definitions, not over IL segments, and Part
1 of this lane's own gate predicts from source+headers+IL.

---

## 4. Shape findings — the predicate's SHAPE was tested and HELD

Per the coordinator's condition 4, the shape questions get their own section.
Both returned **confirmations**, not violations. Negative results, and they are
worth as much as the breaks because they close off two ways §2 could have been
categorically wrong.

### 4a. The root set is per-TU, not per-compiler-process — CONFIRMED

§2's fixpoint is stated per TU. Six cells put 2–3 TUs sharing one header into a
**single `cl.exe` invocation**, giving cross-TU leakage every opportunity:

| cell | structure | result |
|---|---|---|
| `a6c2` | tu1 refs `ca`, tu2 refs `cb`, one invocation | per-TU exact; `cc` in neither |
| `a6c3` | same, command-line order reversed | **identical to `a6c2`, obj for obj** — no order dependence |
| `a6c1` | same sources, two separate invocations | identical to `a6c2` — the control |
| `a6c4` | shared header `static`s, only tu1 refs `sa` | `sa` in tu1 only; tu2 clean |
| `a6c7` | three TUs, only the middle refs `cb` | no leakage forwards **or** backwards |
| `a6c6` | external non-COMDAT def in the shared header, **neither** TU refs it | **`?hc@@YAHH@Z` emitted in BOTH objs** |

`a6c6` is the sharpest of these: R1 fires independently in each TU of the same
compiler process. A per-process root set would have emitted `hc` into the first
obj only. §2's scope clause holds exactly.

`a6c8` adds the same result for dynamic-init roots: a header-defined internal
`static int g_v = mk(seed());` produces `??__Eg_v@@YAXXZ` **and** `?mk@@YAHH@Z`
independently in both objs.

### 4b. §2's root list has no pragma clause, and needs none — CONFIRMED

The 172 fitted cells contain no `#pragma` of any kind. All ten A7 cells matched.

* **`#pragma comment(linker, "/include:?cand@@YAHH@Z")` creates no root** —
  `a7c1` (naming an unreferenced `static`) and `a7c2` (naming an unreferenced
  `inline`) both emit the anchor only. The directive reaches `.drectve` and is
  the *linker's* problem; c2's emit set is unmoved. This was the most plausible
  missing-root category and it is not one.
* **`#pragma init_seg` moves the section, never the name set.** `a7c4/c6/c7`
  each emit exactly `a7c5`'s (no-pragma) 3-name set while the initializer
  pointer moves `.CRT$XCU → .CRT$XCC → .CRT$XCL → .mycrt$a`. §2 has no section
  term and needs none.
* `a7c8` (`code_seg`), `a7c9` (`#pragma section` + `__declspec(allocate)`
  address-take, which propagates correctly via R5→P1), `a7c3` (inert
  `comment(lib/exestr)`) all matched.

---

## 5. The one AMBIGUOUS — §2's root clause R1 admits no consistent reading

**Cell:** `a5c4_extern_then_inline_unref`.
**Predicted** `{?anchor@@YAHH@Z}` — **observed**
`{?anchor@@YAHH@Z, ?cand@@YAHH@Z, ?cand2@@YAHH@Z}`.

Source: `extern int cand(int x);` then `inline int cand(int x){…}`; plus
`extern inline int cand2(int x){…}`. Neither is referenced. Both are emitted.

Graded **AMBIGUOUS**, per my own pre-registration and the coordinator's binding
condition: truth matches the alternative reading I registered at `3401ffb`
("an `extern`-spelling-driven root test would emit one or both"). I am not
re-grading it upward. But three facts belong on the record:

1. **The two readings of §2's R1 are jointly refuted.** R1 reads *"every
   definition with external **non-COMDAT** linkage — plain extern,
   `extern "C"`, …"*.
   * *Reading A* (the head governs; `inline` ⇒ COMDAT ⇒ not a root): predicts
     a5c1 correctly (1 name, MATCH) but a5c4 wrongly (1 vs 3).
   * *Reading B* (the parenthetical is an independent enumeration of root
     spellings): predicts a5c4 correctly (3) but a5c1 wrongly — `a5c1`'s
     unreferenced `extern "C" inline cand` is **not** emitted, and `extern "C"`
     is in the parenthetical.

   **No single reading of §2 as written explains both cells.** That is a defect
   in the predicate's *statement*, which is exactly the lesser class the
   prereg's guard 1 defines — but it is a confirmed one, not a maybe.

2. **§2's own nominated linkage observable sides with Reading A.** §2's third
   corroboration says the COMDAT Selection byte encodes the linkage split —
   Selection 1 for strong linkage, 2 for COMDAT linkage. Measured
   (`selbytes.py`): a5c4's `?cand@@YAHH@Z` and `?cand2@@YAHH@Z` are both
   **Selection = 2 (ANY)** — c2 itself classifies them as COMDAT-linkage.
   Under §2's own observable they are outside R1, and their emission is
   unexplained by any root clause. Under Reading A this cell is a **VIOLATION**;
   I record that and still grade it AMBIGUOUS as registered. Guard 1's
   independent re-derivation should settle it.

3. The rest of A5 matched, and the matches are informative: `extern "C" inline`
   unreferenced is **not** emitted (a5c1) but is emitted when referenced as
   `cand`, undecorated (a5c2); a header-defined `extern "C"` **non-inline**
   definition is a root even unreferenced (a5c7, `hc`); the six-way header
   linkage matrix (a5c6) split exactly 3-referenced / 3-not; `extern "C" static
   inline` behaved as internal linkage and emitted `candR` undecorated (a5c9,
   the decoration I had flagged as uncertain).

**Minimal repair to §2's text:** replace R1's parenthetical with *"— plain
extern, non-inline `extern "C"`, any out-of-line definition … — where `inline`
implies COMDAT linkage **except** when the same entity also carries an `extern`
declaration or the `extern inline` spelling."* That is a hypothesis from two
cells; it is not fitted and should not ship unprobed.

---

## 6. Side findings, for other lanes

**S1 — §2's Selection-byte corroboration is narrower than stated.** §2 says
"Selection 1 (NODUPLICATES) for strong-linkage **and kept statics**, 2 (ANY)
for COMDAT-linkage". Measured here: a kept plain `static` is Selection 1
(a1c4, a5c5, a6c4) but a kept **`static inline`** is Selection **2** with
storage class STATIC (a5c3 `?candR@@YAHH@Z`, a5c6 `?hsiR@@YAHH@Z`). So the
byte tracks `inline`-ness, not the storage class. The port must reproduce this
byte; the rule it needs is "`inline` ⇒ 2" and not "static ⇒ 1".

**S2 — `#pragma init_seg("name")` puts a 14th name in the section vocabulary.**
§1 of the plan rests on "the entire workload section vocabulary is 13 names
(measured, full census), so C is finite and enumerable". `a7c7` produces
`.mycrt$a` — a *user-chosen* section name. The vocabulary is closed over the
workload as measured, but it is not closed by the language. Worth one grep of
the workload for `init_seg("` before R2 treats 13 as a closed set.

**S3 — hand-off to the A3 agent (virtual/MI axis).** Probe f4 shows
`PM g_pm = &C::v;` in a kept data initializer emits the vcall thunk
**`??_9C@@$B3AA`** and *not* `?v@C@@UAAHH@Z`. §2's propagation clause names
"an address-take" as keeping F; here the address-take keeps a *different,
synthesized* symbol. That is A3's territory (`??_9` adjustor thunks) and #152's
synthesis cap; I am reporting it, not claiming it.

**S4 — PCH changes the IL bundle shape, uncharacterized.** For the same source,
the captured `.ex` is **3029 bytes** without PCH but **125 bytes** under both
`/Yc` and `/Yu`, while the objs are correct in all three cases. Where the
bodies travel under PCH is not answered by this lane. No verdict here rests on
it; flagged because any IL-side tooling that meets a PCH build will meet this.

---

## 7. Instrument notes

**I1 — `/Yu` needs an explicit include path on this toolchain.** Every `/Yu`
compile initially failed `fatal error C1034: <hdr>: no include path set`,
although the byte-identical `/Yc` compile of a quoted `#include` in the same
directory succeeded. `/I.` fixes it; `INCLUDE=.` fixes it identically (both
verified, same leader set). **`/I.` was then added uniformly to all 43 cells**
and the whole suite re-run; the paired run without it is kept as
`results_noI.json`. Control: **53 of 59 objs byte-identical leader sets across
the two runs, and the only 6 that changed are exactly the previously-failing
`/Yu` objs going `None →` a set.** `/I.` is inert. Without this fix A8 would
have been reported INSTRUMENT-FAIL for six of its fourteen objs; it is not, and
A8's violation is a real result, not a broken instrument.

**I2 — section NAME is not section CONTENTS, and this bit again.** `a7c8` uses
`#pragma code_seg(".mytext")`. Under a `.text`-name-prefix reading its leader
set is **empty**; under the `IMAGE_SCN_CNT_CODE` characteristic reading it is
`{?anchor@@YAHH@Z}`, which is the correct answer and the one predicted. This is
the same failure mode this project has just spent days on in another
attribution — **a section's name used as a proxy for what the section
contains**. It is one of 59 objs here, and it would have been a fabricated
"violation" (predicted 1, "observed" 0) had I graded by name prefix. Both
readings are recorded for every obj in `results.json`; `a7c8` is the only one
where they differ.

**I3 — no timeouts, no hangs.** 86 records (43 cells × obj pass + listing
pass), every invocation under a 120 s bound, longest observed 0.05 s.

---

## 8. What this does and does not say

* It does **not** say §2 is refuted overall. A1 and A7 are clean across 18
  objects, and A6's shape questions — the ones that could have made §2
  categorically the wrong kind of model — came back confirming it.
* It does say **two of my five axes are BROKEN**, and neither break was
  reachable from the 172 fitted cells: those cells contain **no header file and
  no `#pragma` at all**, so per-TU header sharing, PCH, and virtual dispatch
  into an unconstructed class were all structurally outside the grid.
* Per the prereg's Part-2 scoring, a broken axis blocks **SHIP-CANDIDATE**
  unless the breaking condition is *demonstrated* detectable. Both are:
  * **A8's** class is guarded for free by the `.gl` necessary condition §2
    already carries — demonstrated, `ea` absent from the `/Yu` `.gl`.
  * **A6's** class is **not** guarded by `.gl` (`v` is present) and **not**
    guarded by the C4505/C4514 channel (silent, correctly — c2 dropped it, not
    c1xx). It needs a new source-side syntactic guard, demonstrated on f1/f2/f3.
    Until R3 carries that guard, this class is a live precision risk against
    the V3 ≥ 0.95 gate.

---

## 9. Artifacts

All under `/home/free/code/milohax/c2-rs/.claude/worktrees/w-emitpred/work/emitpred/axes1/`:

| path | what |
|---|---|
| `PREDICTIONS.md` | the frozen predictions (commit `3401ffb`) |
| `cells/<axis>/<cell>/` | 43 cells, sources + `spec.json` (commit `7e49cc3`) |
| `gencells.py` | single generator; recreates the whole cell tree idempotently |
| `runcells.py` | Phase-2 runner (base flags + `/I.`, 120 s bound) |
| `leaders.py` | COFF reader; records **both** code-characteristic and `.text`-prefix leader sets |
| `selbytes.py` | per-COMDAT Selection byte (the §2 linkage observable) |
| `grade.py` | predicted-vs-observed grading; ALT/decoration tables as registered |
| `results.json` / `results_noI.json` | observed sets, both runs (the `/I.` control) |
| `grades.json` | per-obj verdicts |
| `detect/` | `/Wall` runs, the no-PCH control, captured IL bundles, and `detect/mech/` (probes f1–f4) |

Nothing was written outside this directory; nothing under `crates/` or `docs/`
was touched; no obj, `.cod`, `.pch`, or captured IL is staged for commit; no
`git commit` was run by me.
