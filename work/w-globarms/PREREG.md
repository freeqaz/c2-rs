# PREREG — lane `w-globarms`, wave 19 L4

    Lane:     w-globarms
    Brief:    docs/ADOPTION_BRIEF_2026-08-29.md §L4, board #3808–#3813
    Owns:     docs/whitebox/ref/P_GLOBREGS.md, docs/whitebox/WB_GLOBARMS_*,
              docs/whitebox/grids/w-globarms/, work/w-globarms/, this lane's
              new scripts, board rows #3808–#3813, its rung
    Kind:     characterization lane (docs/rungs/README.md § "Lane kinds")
    Reach:    predicted 0. `git diff master..HEAD -- crates/` MUST be empty at
              the tip. No gate.sh row (#3691). No `ported` numerator for
              globregs or regalloc (decision 21 §4, #3505).
    Base:     master 12d3c0558, branch wt-w-globarms

**Committed before the image was opened and before any deciding cell was
compiled.** This file is the frozen prediction set; anything registered later
goes in a numbered `PREREG_ADDENDUM.md` with its own commit, and the score is
reported **by tier, never pooled** (`w-globobj` §4's rule).

---

## 0. WHAT I HAD ALREADY READ WHEN I WROTE THIS — stated so no prediction below is scored as blind when it is not

Honesty about the starting state matters more than the hit rate. Before writing
this file I read, all of them **committed repo artifacts**, none of them the
image:

* `docs/whitebox/ref/P_GLOBREGS.md` in full, including §3's gate-A table.
* `docs/whitebox/labels/globregs/10b550e5.txt` — the **committed objdump +
  Ghidra listing** of `FUN_10b550e5`, which is where §3's table comes from.
  **So every statement below about what the arms *are* is a re-reading of a
  committed listing, not a blind prediction, and is scored as `READ` rather
  than as a hit.** Only the rows marked `BLIND` are scored.
* `docs/whitebox/WB_GLOBOBJ_FINDINGS.md`, `work/w-globobj/MARKS.tsv`,
  `docs/rungs/2026-08-28-w-globobj.md`.
* `docs/whitebox/ref/P_REGALLOC.md` (read-only; this lane may not edit it).
* grep hits for "symbol kind" across `docs/whitebox/` — which yielded exactly
  one attributed value: **kind 1 = a physical register**
  (`WB_LIVE_FINDINGS.md`:254).

**Not opened yet:** `compilers/X360/16.00.11886.00/c2.dll` beyond the four
committed `labels/globregs/*.txt` listings. **Not compiled yet:** any cell.

---

## 1. THE TWELVE ARMS, as §3's table carries them — `READ`, not predicted

Numbering used by this lane throughout. `kind` is the byte at `sym+0x04`.

| arm | addr | test | on TRUE | tail |
|---|---|---|---|---|
| **A1** | `0x10b5511a` | `kind == 0x10` | → `0x10b55295` | **skip, NO reject tail** |
| **A2** | `0x10b55125` | *(none — unconditional)* | `sym+0x40 &= ~1` | — |
| **A3** | `0x10b55129` | `sym+0x08 != sym` | → `0x10b55295` | **skip, NO reject tail** |
| **A4** | `0x10b55134` | `kind == 3` | → `0x10b551b3` (A11) | dispatch |
| **A5** | `0x10b55138` | `kind < 3` | → `0x10b552b8` | **REJECT + tail** |
| **A6** | `0x10b5513e` | `kind ∈ {4,5}` | → `0x10b551a8` | eligible; `+0x05 & 2` ⇒ set flag |
| **A7** | `0x10b55142` | `kind == 6` | → `0x10b552b8` | **REJECT + tail** |
| **A8** | `0x10b5514a` | `kind ∈ {7,8}` | → `0x10b551ae` | eligible; flag **always** set |
| **A9** | `0x10b5514e` | `kind ∉ {10}` | → `0x10b552b8` | **REJECT + tail** (kind 9, 11…) |
| **A10** | `0x10b55156`–`0x10b5516b` | kind 10 needs `(*(sym))[0x37] & 0x400` set **and** `& 0x200000` clear | else → `0x10b552b8` | sub-symbol path |
| **A11** | `0x10b551b3` | kind 3 needs `sym+0x14 == 0` | else → `0x10b552b8` | **REJECT + tail** |
| **A12** | `0x10b551bc` | kind 3 needs `sym+0x07 & 0x40` clear | else → `0x10b552b8` | then `sym+0x06 &= ~2` |

**Reject tail `0x10b552b8`:** `DAT_10c2e454++`, then `+0x34 = +0x38 = 0` on
every sub-symbol. **A1 and A3 do not run it.**

### 1.1 Two things the committed listing shows that §3 does not state — `READ`, and both are page amendments this lane owes

* **`R-A`: kind 10 never reaches gate B.** The A10 path runs
  `0x10b55171`–`0x10b551a3` and jumps to `0x10b55295`, so it never touches
  `0x10b551ca`/`0x10b551d4`, which is the `FUN_10bd7d24` type gate. **Gate B
  applies to kinds 3, 4, 5, 7, 8 and to nothing else.** §3 presents gate A and
  gate B as sequential for every symbol; for kind 10 they are alternatives, and
  `t+0x20 == 4` is kind 10's *substitute* for gate B, not an addition to it.
* **`R-B`: the `DAT_10c2e3ec` side set at `0x10b5520d` is FP-specific.**
  `0x10b551e6`–`0x10b5520b` admits a symbol to it only when the **type word's
  top nibble is 5** on the leader or the sub-symbol (`and cx,0xf000` /
  `cmp cx,0x5000`), which §9 already calls the FPR nibble. §3 says
  `DAT_10c2e2cf` "only adds the index to a side bitset"; it does not say the
  bitset is the FP one.

Both are graded `CONFIRM`/`REFUTE` against the image in §6.

---

## 2. THE PREDICTIONS — the kind enum (all `BLIND`)

The read this lane owes is *what value of `kind` a source construct produces*,
because without it no arm has a witness. Method registered in advance:
enumerate every **write** site of the `sym+0x04` byte in the pinned image,
attribute each to its containing function via `docs/whitebox/c2_functions.tsv`,
and take the enum from the writers rather than from the readers.

| # | prediction | p | grading |
|---|---|---:|---|
| **K1** | kind **1** is a physical register — `WB_LIVE_FINDINGS.md`:254 re-derives from a write site, not only from a read site | 0.80 | write site found and attributed |
| **K2** | at least one of kinds **4,5,7,8** is the ordinary **auto local**, i.e. the thing `pc_int` promotes | 0.90 | attribution |
| **K3** | kind **10** is an **aggregate / a symbol with a member list**, and A10's sub-symbol walk is *why* `w-globobj` found aggregates promoted member-wise | 0.65 | attribution |
| **K4** | kind **0x10** is a dead/placeholder slot — that is why A1 skips it without even charging the diagnostic counter | 0.50 | attribution |
| **K5** | kind **3** is the **externally-visible / statically-allocated** symbol (a global or a function-static), and A11+A12 are why one is never enregistered | 0.45 | attribution |
| **K6** | **at least 8 of the 11 distinct kind values `0..10` + `0x10` get an attributed writer** | 0.55 | count |

**K6 is the ceiling on the read half and it is registered on purpose.** If the
enum cannot be attributed, most arms have no witness and the honest outcome is
a classification, not a conversion.

---

## 3. THE PREDICTIONS — the obj cells (all `BLIND`)

Readout: **`w-globobj`'s frame-traffic rule**, re-implemented in this lane's own
grader rather than imported, and **cross-checked against `grade_globobj.py` on
the same dumps** as a control (§5 C4). A promoted local needs no stack slot;
the prologue's own saves sit before the `stwu` and are excluded by
construction.

Every cell is compiled at **both** `/O1` (mode W) and `/Ox` (mode X).

| # | cell | construct | predicted | p |
|---|---|---|---|---:|
| **O1** | `ka_int` | plain `int` local | **PROMOTED** (positive control) | 0.97 |
| **O2** | `ka_vol` | `volatile int` local | **MEMORY** (negative control) | 0.95 |
| **O3** | `ka_fstatic` | file-scope `static int` | **MEMORY** | 0.90 |
| **O4** | `ka_extern` | `extern int` global | **MEMORY** | 0.93 |
| **O5** | `ka_formal` | `int` by-value parameter | **PROMOTED** | 0.90 |
| **O6** | `ka_ref` | `int&` parameter (the reference itself) | **PROMOTED** | 0.85 |
| **O7** | `ka_this` | `this` in a non-static member function | **PROMOTED** | 0.80 |
| **O8** | `ka_escape` | `int` whose address escapes | **MEMORY** | 0.95 |
| **O9** | `ka_lstatic` | function-scope `static int` | **MEMORY** | 0.92 |
| **O10** | `ka_arrelem` | one element of a local `int[4]`, others untouched | **PROMOTED** | 0.70 |
| **O11** | `ka_bitfield` | a `struct` bit-field member, member-wise | **PROMOTED** | 0.45 |
| **O12** | `ka_longlong` | `long long` local on a 32-bit-register target | **PROMOTED** (both halves) | 0.75 |

O1/O2/O8/O9 deliberately **replicate** `w-globobj` cells. A replication that
disagrees is a finding about one of the two graders and is reported as such.

---

## 4. THE PREDICTION THAT MATTERS MOST — the CONSTR/UNCOMP classification, registered BEFORE any cell

**The rule that binds this lane, inherited verbatim from `w-globobj` §3 and
board `#3505`:**

> **`CONSTR` is a claim about the corpus, not about my index.** Before an arm
> is filed unobservable I must state **the two obj bodies that would have to
> differ**, and why they cannot exist. If I cannot state them, it is
> `UNCOMP` — merely uncompiled — however unlikely a cell looks.

**The structural reason most reject arms are expected to be `CONSTR`, stated
now so it is not invented after the fact:** A5, A7, A9, A11 and A12 all branch
to the *same* address `0x10b552b8` and produce the *same* state — `+0x34 = 0`
on every sub-symbol, plus one increment of a counter. §3 already carries the
identical argument for gate-A-vs-gate-B ordering and `w-globobj` filed it
`CONSTR`. **An obj cannot say which arm rejected a symbol; it can only say that
the symbol was rejected.**

| arm | predicted bucket | p | if `CONSTR`, the two bodies that cannot differ |
|---|---|---:|---|
| A1 | `CONSTR` | 0.75 | a body where a kind-`0x10` slot is promoted vs one where it is skipped — kind `0x10` is not source-selectable and the skip and the reject produce identical output |
| A2 | `CONSTR` | 0.85 | `sym+0x40` bit 0 is cleared for **every** symbol unconditionally; a body in which it is not cleared does not exist |
| A3 | `CONSTR` | 0.70 | a non-leader promoted vs skipped — the leader carries the colour either way |
| A4 | `OBS` | 0.55 | dispatch, not a decision; graded through A11/A12's constructs |
| A5 | `CONSTR` | 0.70 | kinds 0/1/2 vs any other rejecting kind: identical obj |
| A6 | `OBS` | 0.80 | — |
| A7 | `CONSTR` | 0.75 | kind 6 vs any other rejecting kind: identical obj |
| A8 | `OBS` | 0.70 | — |
| A9 | `CONSTR` | 0.75 | kind 9 vs any other rejecting kind: identical obj |
| A10 | `OBS` | 0.65 | — |
| A11 | `OBS` | 0.50 | — |
| A12 | `UNCOMP` | 0.45 | `sym+0x07 & 0x40` is a flag whose source-level cause is unknown to this lane |

**The registered ceiling: at most 5 of the 12 arms convert `[R]` → `[O]`, and
at least 6 are `CONSTR`.** Registering a ceiling before the deciding cell is
what made `w-regcells`'s negative result credible, and it is registered here
for the same reason. **If more than 5 convert, this prereg is wrong and the
lane says so in those words.**

### 4.1 The one route that would break the `CONSTR` wall, and it is graded

The reject tail's only *other* effect is `DAT_10c2e454++` at `0x10b552b8` — a
per-function count of rejected symbols. **If any reader of `DAT_10c2e454`
reaches an emitted artifact** (a diagnostic, a `/FAsc` listing line, a
`.debug$S` record, a heuristic threshold that changes codegen), then the skip
arms A1/A3 and the reject arms A5/A7/A9 **do** separate in an obj and half of
§4's table is wrong.

> **G1 `BLIND`, p = 0.20 — `DAT_10c2e454` has no reader that reaches an
> emitted artifact.** Graded by enumerating its references in the pinned image.
> This is the single prediction whose refutation would most change the lane's
> conclusion, which is why it is registered with its own number.

---

## 5. CONTROLS — each must be watched **RED** before any verdict from it is quoted (`#3336`)

| # | control | what going RED looks like |
|---|---|---|
| **C1** | **positive/negative pair fires**: `ka_int` PROMOTED and `ka_vol` MEMORY at both profiles | if either flips, the readout is dead and every promotion verdict is discarded |
| **C2** | **planted defect: the arm decoder reads the wrong displacement** — decode `cmp al, imm` at `+1` instead of `+0` | the "12 arms re-derive from the image" assertion must FAIL |
| **C3** | **planted defect: the frame-traffic scan includes the prologue saves** | `ka_int` must flip to MEMORY, i.e. C1 must fail |
| **C4** | **cross-grader agreement**: `grade_globarms.py` and `grade_globobj.py --promote` must return the **same** verdict on the same dump for the replicated cells | a disagreement is a finding about one grader, published either way |
| **C5** | **premise test**: a cell with no `stwu` frame scores `U`, enters no numerator and no denominator | absence must not read as a verdict |
| **C6** | **image-absent honesty**: with the image unresolvable the grader must print `(image absent — N assertions skipped)` and **exit 2**, never a silent green — `w-globobj` §2.6's third defect, where two planted defects reported green because their controls silently skipped | grep-for-FAIL must not be able to read a skip as a pass |

**Mutation-control hygiene, from brief §5**: after restoring a mutated file,
`touch` it and verify the restore actually rebuilt, because `cp`/`mv` preserves
the older mtime and cargo silently runs the mutated binary.

---

## 6. THE READ PREDICTIONS graded against the image

| # | claim | source | p |
|---|---|---|---:|
| **R-A** | kind 10 never reaches gate B | `READ` from the committed listing; graded `CONFIRM` against the image | 0.95 |
| **R-B** | the `DAT_10c2e3ec` side set admits only type-nibble-5 (FP) symbols | `READ`; graded against the image | 0.90 |
| **R-C** `BLIND` | `FUN_10b550e5` is the **only** function in the image that switches on `sym+0x04` with this 12-arm shape — i.e. the arm structure is not shared with a sibling phase | | 0.40 |
| **R-D** `BLIND` | `sym+0x04` in the globregs record and `sym+0x04` in the `.gl` record of `DISCLOSURE W-STAGETAP-6` (*"→ a NUL-terminated `char *`"*) are **two different record types**, not a contradiction | | 0.85 |

---

## 7. WHAT EACH OUTCOME LICENSES — the state change, registered in advance

* **≥ 1 arm converts with a named witness** → `[R]` → `[O]` on that arm's row
  in `P_GLOBREGS` §3, the witness cell named inline, and the arm's row in
  `WB_GLOBARMS_FINDINGS.md` §1. Licenses **no** `crates/` change and **no**
  numerator.
* **An arm is filed `CONSTR`** → it stays `[R]` **permanently and correctly**,
  and §10.1's family table gains a row. Licenses no further obj work on it.
* **An arm is filed `UNCOMP`** → it stays `[R]`, and §10.2 gains a row **naming
  the cell that would decide it**. It is explicitly *not* a claim that no cell
  exists.
* **G1 refuted** (a reader of `DAT_10c2e454` reaches an artifact) → §4's
  `CONSTR` predictions for A1/A3/A5/A7/A9 are withdrawn *before* being
  published, and the lane says the prereg was wrong.
* **The kind enum cannot be attributed (K6 fails)** → the lane reports the arms
  read and classified with **zero** conversions and says `FAILED` on the
  conversion deliverable in those words, per the brief's rule that a lane
  producing none of its deliverable says FAILED rather than a compound
  headline.
* **No outcome licenses a `ported` numerator for `[globregs]` or
  `[regalloc]`.** Decision 21 §4, `#3505`. If the read makes a numerator look
  obviously definable, that is a finding handed to the owner, not a licence.

---

## 8. THE MERGE HAZARD, registered so the coordinator does not have to discover it

`crates/c2-harness/src/subsys.rs`'s `the_mark_census_reproduces` pins
`P_GLOBREGS`'s `(read, obj, inferred)` mark triple. It is at `(49, 21, 4)` on
`master` after `w-globobj`. **Every `[O]` this lane adds reddens it, and this
lane is barred from `crates/`.** This lane will **not** edit the pin
(`#3748` — a re-bless belongs in a diff a reviewer reads) and will **report the
new triple in its rung** so the merge is one verified line. `w-globobj` hit
this exactly and handled it this way.
