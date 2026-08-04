# w-joint — PRE-REGISTRATION

    Lane:    w-joint, 2026-08-04, worktree `wt-w-joint` off master `78d29e6`
    Ships:   NOTHING under `crates/`.  No fixture, no codegen, no widening,
             no `DISCLOSURE.md` row.
    Object:  (1) EXTEND the truth capture to record DEFINED DATA SYMBOLS —
                 w-skip §8 item 4, named as this lane's first task;
             (2) build and grade the JOINT DATA+CODE FIXPOINT that w-skip's
                 owner-emitted split (10/10 vs 0/10) says the emit set is.

**This file is committed BEFORE the corpus-wide measurement.**  §9 discloses,
by name and by number, everything I had already looked at when I wrote it.

---

## 0. The incumbents — per axis, so an improvement is distinguishable from a regression

Three lanes have measured the same quantity on the same 850 TUs.  **w-refs still
holds the best F1 and the best per-TU exact count, and two successive lanes have
gone backwards against it while improving recall.**

| model | precision | recall | **F1** | **per-TU exact** | roots |
|---|---:|---:|---:|---:|---|
| **`RGL`** w-refs | **1.00000** | 0.74307 | **0.85260** | **132 / 850** | `Seed` |
| `INIT` w-mark | 0.27289 | **0.95991** | 0.42496 | 34 / 850 | `Seed ∪ I` |
| `SKIP` w-skip | 0.36420 | 0.83732 | 0.50761 | 34 / 850 | `Seed ∪ I_skip` |
| emit-everything | 0.11577 | 1.00000 | 0.20752 | — | `U` |

**The bar this lane must clear on F1 is 0.85260, not 0.50761.**  Base rate
`|E|/|U|` = **0.11577**; that is the precision of predicting every function and
it is the denominator of the coincidence calibration.

---

## 1. What is being built, and why it is a fixpoint rather than a filter

w-skip established, through real `c2.dll` on two TUs, that an initializer
contributes roots **only when the owning DATA symbol is itself emitted** —
10/10 against 0/10, with `+0x20 = 0x1c01` in **both** arms, which refutes any
flag-based reading.  It also established (§2) that the channel does **not** need
to be ordered: `0x10b98e26` has exactly one caller chain and runs before the
compile loop, reading only fields the `.gl` reader wrote.  Its correction to the
brief is the sentence this design has to answer:

> statically-readable is not the same as statically-expressible.

The predicate is readable without running codegen; it is not *expressible* as a
root set over functions, because the thing it tests — the owner's emission — is
part of the answer.  So:

    CODE nodes  U    gate-clean tag-0x0E `.gl` names                w-refs
    DATA nodes  W    kind-1 `.gl` names that own an `in` record     w-skip

    cc  f -> RGL(f)   function targets ONLY.  w-skip T-e: `0x10b27f3c` keeps an
                      edge only for a tag-0x0E target, so there is **no
                      code->data edge at all**
    dc  d -> f        an `02` initializer node of d naming a function
    dd  d -> d'       an `02` node of d naming another data symbol

    Rc = Seed         { f : flags4c & 0x20 }                        w-roots
    Rd = ???          THE FITTED PARAMETER

**Because there is no cc-edge into the data half, `Rd` carries the entire data
side.**  That is the whole of the fitting and it is declared in one place
(`work/w-joint/joint.py`), as a frozen enumeration, not as a tweakable knob.

### 1a. The fitted parameters, named, and what varies across them

| variant | `Rd` | fitted? |
|---|---|---|
| **`ORACLE`** | `D(t)`, the extended truth's defined-symbol set | **NOT A MODEL — a CEILING.** It consumes truth and is labelled as such everywhere |
| `ORACLE_LOOSE` | `ORACLE` plus every owner this decoder cannot name | the blind-spot sensitivity |
| `ALL` | every owner | w-mark's unfiltered reading, as the fixpoint's degenerate case |
| `NONE` | `{}` | the floor; the fixpoint must degenerate **exactly** to `P_RGL` |
| `F20_400` `F20_80` `F20_480` `F20_4000` `F20_60_20` `F20_1000` `F20_2000` | `(f20 & m) ? v` on the kind-1 flag word | **fitted**: the mask is the parameter. Every mask is transcribed from a named instruction (w-skip §1a/§1b) rather than searched |
| `TAG_01` `TAG_02` | the `.gl` record tag | fitted |
| `SC_STATIC` | the kind-1 storage-class byte at `0x10b9b9ee` | fitted |

**What varies across them is exactly one thing: which owners are roots.**  The
edges, the seed, the closure operator, the name binding and the truth reader are
w-roots'/w-refs'/w-skip's as landed and are recomputed in the same pass (KA-A).

### 1b. The ORACLE is a ceiling, and I will not quote it as a model

`ORACLE` reads `D(t)` out of the obj.  It answers exactly one question —
***given a perfect data half, is the code half solved?*** — and its number is a
bound on every joint fixpoint of this shape, never a predicate.  Nothing may be
shipped, scheduled or costed from it.  **If `ORACLE` is high and every static
`Rd` is low, the lane's result is a RELOCATION of the problem, not a solution**,
and that is how it will be written.

---

## 2. The instrument, and why it needs its own grade

**The oracle cannot grade a correspondence.**  The compiler judges obj bytes; it
cannot tell you whether census row *R* is symbol *S*.  So the extended truth is
graded on invariants of its own, and the deliverable is the grade, not the file:

| # | invariant | how it can go RED |
|---|---|---|
| **INJ** | within one obj a defined name defines exactly one entity | two entities claim a name; counted **and named** |
| **TOT** | every symbol-table ENTITY lands in exactly one bucket | residue > 0, **printed with its names on every run**. Residue 0 is *not* a control on its own (STATUS trap 4) |
| **AR** | **arity**: `A1` `sum(1+naux) == NumberOfSymbols`; `A2` aux count `== nsym − entities`; `A3` every long name resolves in the string table, with its bytes counted | a reader that mis-walks aux records leaves TOT silent at residue 0 and takes A1 red. **Residue counts entities; arity counts their contents** — the `DUP`-expansion precedent |
| **AGREE** | recomputed code COMDAT leaders **==** w-emit's independently captured `truth/<slug>.txt` | the one place the oracle already graded the symbol table |
| **KA-DUP** | the cache holds several entries per TU at the same dc3 rev; their classification must be identical | if they disagreed, reading the cache instead of re-running `cl` is unsound |
| **KA-IL** | the cache entry's `gl` bytes **==** w-emit's independently captured `gl` | ties the IL and the truth to one c2 invocation |

**Could AGREE have gone red in the most likely failure mode?**  Yes.  The most
likely failure is that the cached obj is not the obj a plain `cl` run makes —
the capture invocation adds `-il`/`-typedil` and writes through a different
`-Fo` path.  AGREE compares against 850 objs produced by a *separate* plain
`cl` run in a different lane, at a different wibo, so a capture-path artifact
lands on it directly.

---

## 3. Registered numbers — POINT and INTERVAL are separate, and the decline clauses key on the MEASURED VALUE

The point is where my belief is.  The interval is what I would not be shocked
by.  **Every decline clause below keys on the measured value against the
incumbent, never on the interval** — conflating those cost an earlier lane half
its recorded miss.

### 3a. The instrument

| # | quantity | **point** | interval |
|---|---|---:|---|
| **T1** | **AGREE** — TUs where my code-COMDAT leaders equal w-emit's truth | **850 / 850** | [845, 850] |
| **T2** | TOT residue, entities over the corpus | **0** | [0, 500] |
| **T3** | AR: TUs failing A1 / A2 / A3 | **0 / 0 / 0** | [0, 5] each |
| **T4** | INJ: TUs with a duplicate defined name | **0** | [0, 40] |
| **T5** | KA-DUP: duplicate entries classifying identically | **40 / 40** | pass ≥ 38/40 |
| **T6** | KA-IL: TUs where the cache `gl` == w-emit `gl` | **850 / 850** | [845, 850] |
| **T7** | `\|D_data\|` summed over the corpus | **260 000** | [80 000, 600 000] |
| **T8** | `\|D_all\|` summed over the corpus | **620 000** | [250 000, 1 200 000] |

### 3b. The ceiling

| # | quantity | **point** | interval |
|---|---|---:|---|
| **M1** | `ORACLE` precision | **0.999** | [0.950, 1.000] |
| **M2** | `ORACLE` recall | **0.930** | [0.800, 0.980] |
| **M3** | **`ORACLE` F1 — the headline** | **0.955** | [0.870, 0.990] |
| **M4** | `ORACLE` per-TU exact `P == E` | **0.30** (255/850) | [0.12, 0.70] |
| **M10** | `ORACLE` F1 with `#152` excluded from both `E` and `P` | **0.965** | [0.880, 0.995] |

### 3c. The models with parameters, and the structure

| # | quantity | **point** | interval |
|---|---|---:|---|
| **M5** | `\|Rd_ORACLE\| / \|owners\|` — the fraction of owners that are emitted | **0.020** | [0.005, 0.200] |
| **M6** | `\|owners ∩ E\|` — the circularity check; an owner must not be an emitted **function** | **0** | [0, 500] |
| **M7** | **best F1 over the 12 static `Rd` variants** | **0.66** | [0.35, 0.92] |
| **M8** | `ORACLE_LOOSE` F1 − `ORACLE` F1 | **+0.002** | [−0.010, +0.050] |
| **M9** | coincidence calibration: emitted share of the marks `ORACLE` adds over `P_RGL`, as a ratio to the uniform expectation | **6.0×** | [1.5×, 12×] |
| **M11** | owner tokens this decoder cannot name, as a fraction of `in` records | **0.28** | [0.05, 0.50] |
| **M14** | `NONE` reproduces `P_RGL` **exactly** | **850 / 850** | [850, 850] |

### 3d. The mutation through the SOLE JUDGE — in directions that can fail

| # | arm | prediction | pass mark |
|---|---|---|---|
| **M12** | **replication** of w-skip's owner split on **THREE TUs it did not use**: retarget one `02` node, split by whether the owner is a defined symbol in the baseline obj | H+ APPEARS, H− does not | **H+ ≥ 4/5 per TU** and **H− ≤ 1/5 per TU** |
| **M13** | **the dd-edge arm — the one that separates a FILTER from a FIXPOINT.** Pick an owner `d'` that is **NOT** defined in the obj but **IS** named by an `02` node of an owner that **is** defined. Retarget a node of `d'` | a pure "owner ∈ D" **filter** predicts NO appearance; the **joint fixpoint** predicts APPEARANCE | ≥ 3/5 APPEARS ⇒ the fixpoint is right and the filter is wrong; ≤ 1/5 ⇒ the reverse; in between ⇒ undecided, reported as undecided |

**M13 is the arm I most expect to be wrong about**, and it is registered because
it can refute *this lane's own shape*: if `d'` never pulls anything in, the dd
edge is not real, the model is a filter and not a fixpoint, and my §1 is wrong.

---

## 4. Decline clauses — written before the numbers exist

1. **F1 < 0.87260** (the incumbent 0.85260 plus a ±2.0 pp wash band) **for the
   best MODEL (M7)** ⇒ the model half is published as a **refuted hypothesis**,
   the first line says so, and **I do not go looking for a further channel after
   the number arrives.**  Keys on the measured value.
2. **M1 < 0.95** ⇒ the ceiling is not a ceiling; the coincidence calibration
   goes in the headline paragraph and M2/M3 are published as a bound.
3. **M12 fails** (H+ < 4/5 or H− > 1/5 on any TU) ⇒ **the failure is reported
   FIRST**, before any headline, and §1's owner predicate is marked as not
   replicating.
4. **M13 lands ≤ 1/5** ⇒ the dd-edge is refuted, §1 is corrected in place to a
   FILTER rather than a fixpoint, and the correction goes above the headline.
5. **T1 < 845** ⇒ reading the capture cache is unsound; **every model number in
   this lane is withdrawn** and the lane re-runs `cl` or reports nothing.
6. **No instrument tuning after truth is read.**  The `Rd` enumeration, the
   fixpoint operator and the truth reader are frozen at this commit.  Any change
   after a corpus number exists is disclosed with both numbers.
7. **Nothing ships under `crates/`.**  `PortC2` still returns `NotImplemented`
   outside its class.
8. **`Rfloor` is not a decline key**; it is reported for comparability only.
9. **`ORACLE` is never quoted as a model**, in this lane or in the rung doc.

### 4a. The one-shot Part-1 gate — NOT spent, and NOT spendable by me alone

w-emitpred's Part-1 gate is **UNSPENT** and the 21-TU quarantine is **INTACT**;
five consecutive lanes preserved it.  This lane is the first with **fitted
parameters** (§1a), which is exactly what a held-out population catches, and the
brief says the gate belongs to it.

**I will not spend it unilaterally.**  If a static `Rd` beats the incumbent in
sample, I stop, describe the model, its in-sample numbers, its fitted parameters
and what varies across them, and **ask the coordinator**.  If the model is
refuted in sample, I do not spend it and I say so — w-skip's reason, adopted
verbatim: *a held-out set cannot improve a refutation.*

No quarantined TU will be read, captured or mutated; every mutation TU is
checked against `heldout.txt` by name before anything is written.

---

## 5. Known-answer controls

| # | control | pass mark |
|---|---|---|
| **KA-A** | reproduce all three incumbents **exactly** — `\|U\|` 1 506 586, `\|E\|` 174 417, `\|Seed\|` 14 662, RGL 129 604 / 1.00000 / 0.74307 / 0.85260 / 132, INIT 613 532 / 0.27289 / 0.95991 / 0.42496 / 34 | every figure to the digit |
| **KA-B** | the `in` terminus gate unchanged | 850/850 clean |
| **KA-INJ / KA-TOT / KA-AR / KA-AGREE / KA-DUP / KA-IL** | §2 | §3a |
| **KA-NONE** | `Rd = {}` reproduces `P_RGL` exactly on every TU | 850/850 (M14) |
| **KA-POS** | **positive check — this run GRADED something**, printed as a count of discriminating names: `P_ORACLE Δ P_RGL` and `P_ORACLE Δ P_INIT` | both > 0 |
| **KA-PROV** | dc3 HEAD before/after; wibo version | no mid-run move; wibo `1.0.1-23-g4a9dd6f` |

**Before trusting a control, ask whether it could have gone red.**  KA-NONE
could: if `data_fixpoint` leaked a root when `Rd` is empty, or if my `closure`
differed from w-refs' by one line, `NONE` would not reproduce `P_RGL` and the
whole comparison would be against a moved baseline.  KA-AGREE could: §2.
KA-A could: it has caught a moved baseline in three previous lanes.

---

## 6. What this lane will NOT measure — named in advance, so absence cannot read as success

1. **Where a data symbol's emission comes from.**  `ORACLE` conditions on it and
   does not explain it.  The `db` sub-stream — which the capture cache holds and
   **no lane has ever read** — is the obvious next instrument and is **not**
   decoded here.
2. **`0x10b3389b`** (`dag.c`, edges added during codegen) and **`0x10b9aa26`**
   (the by-name intern, roots added during codegen).  w-skip named both as the
   sources of the *ordering* requirement; neither is modelled.
3. **`#152`** — `??_G`/`??_E` deleting destructors are synthesized by c2 and
   named by no `02` node, so no initializer model of any shape reaches them.
   Stratified out and reported both ways, never repaired.
4. **`sy`.** Still unread.
5. **Node kind `0x14`.**  Only the stream's `0x02` byte kind is decoded.
6. **Order.** A right set in the wrong order is still a mismatch.
7. **The 21 quarantined TUs.**
8. **Whether `D(t)` is predictable at all.**  That is the question this lane
   hands on, not one it answers.

---

## 7. Restated before the numbers exist

* **TU match stays 8.**  This lane changes no Rust.
* **`census/gate disagreement` stays 0.**
* **A high recall is not a shippable predicate**, and neither is a high ceiling.
* **A ceiling is not a schedule.**  Reaching `ORACLE` requires an instrument
  nobody has built.

---

## 8. The single outcome I most expect to be wrong about

**M13, the dd-edge arm.**  I have registered it at ≥ 3/5 because §1's fixpoint
says a data symbol reached from an emitted data symbol is itself live.  If it
comes back 0/5 the honest reading is that w-skip's predicate is a **filter on a
defined set**, not a fixpoint, my §1 is wrong in its central claim, and the
brief's framing of this lane is wrong with it.  I would rather find that out
from the sole judge than argue it from a disassembly.

Second most likely: **M7**.  I expect every static `Rd` to fail, i.e. that the
best static F1 is *below* the incumbent and clause 1 fires.  I have registered
M7 at 0.66 — above the wash bar's 0.66-ish neighbourhood but below 0.87260 — so
being right costs me nothing and being *optimistic* would.

---

## 9. FULL DISCLOSURE — everything I had already seen when I wrote this

Protocol requires the prereg before the measurement; it does not require me to
pretend I built the instrument blind.  Before this commit I had:

1. **Built and run the extended truth capture on a 5-TU pilot**
   (`work/w-joint/pilot_tus.txt`, the first five rows of the index:
   `src/App.cpp`, `src/ChecksumData_xbox.cpp`, `src/Main.cpp`,
   `src/Memory_Xbox.cpp`, `src/lazer/game/BustAMovePanel.cpp`).
   Its invariants read: INJ 0, TOT residue 0, AR A1/A2/A3 0/0/0,
   **AGREE 5/5**, `|D_all|` 3 722, `|D_data|` 1 551, `|E|` 606,
   records 8 370 = entities 6 287 + aux 2 083.
2. **Run the joint scan on the same 5 TUs.**  `work/w-joint/pilot.txt`:
   `ORACLE` **1.00000 / 0.94224 / 0.97026**, `ORACLE_LOOSE`
   1.00000 / 0.94389 / 0.97114, `ALL` 0.10604 / 0.94224 / 0.19062,
   `NONE` = `RGL` 1.00000 / 0.73927 / 0.85009, best static `F20_2000`
   0.55966 / 0.75083 / **0.64130**; owners 6 539, `owners ∩ E` **0**,
   `owners ∩ D` 118, owner-unbound 1 828 of 6 539.
   **M1/M2/M3/M5/M7/M11 are therefore registered informed by a 5-TU pilot and
   are not blind**, and I have registered them *below* the pilot (M3 0.955
   against a pilot 0.970; M7 0.66 against a pilot 0.641 — above, because five
   TUs is a thin read of a maximum over twelve variants) so that a corpus that
   behaves like the pilot still costs me on some of them.
3. **One change made after the pilot**: `ORACLE_LOOSE` was added, because the
   pilot showed 28 % of owner tokens unbound and resolving that blind spot in
   only one direction is how a blind spot gets to look like a filter.  It
   **widens** the evidence against a clean result and is registered as M8.
4. **Not looked at**: any corpus-wide number, any mutation, `db`, `sy`, the
   quarantined TUs, and the residual class histogram.

`work/w-joint/pilot.txt` and `work/w-joint/pilot_idx.tsv` are committed with
this file so the disclosure is checkable rather than asserted.
