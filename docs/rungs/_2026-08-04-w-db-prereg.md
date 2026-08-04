# w-db — PRE-REGISTRATION

    Lane:    w-db, 2026-08-04, worktree `wt-w-db` off master `669ee6c`
    Commits: this file BEFORE the first corpus-wide measurement.
    Ships:   NOTHING under `crates/` is planned.
    Judge:   real `c2.dll` under wibo for every causal claim; the obj for truth.

w-joint relocated Phase 7's emit-set problem to one question — **which DATA
symbols does c2 define?** — and named the **`db` sub-stream** as the instrument
no lane has read. This lane reads `db`, and it registers a *second*, larger
claim that the `db` read displaced: **the emit set IS a joint fixpoint, in the
direction w-joint did not test.**

---

## 0. The correction this lane will argue, registered before it is measured

w-joint's `joint.py` docstring states, and its U-a implies:

> `cc  f -> RGL(f)` … *so there is NO code->data edge and the data half cannot
> be reached from code.*

That comes from w-skip **T-e**, which is right about the pass it names:
`0x10b27f3c` resolves `[head+0x14]` into `[head+0xc]` and keeps an edge only
when the target is a tag-`0x0E` function record. **But `[head+0xc]` is the list
`Mark` walks, and `Mark`'s `+0x4c |= 0x20` is the CODE emit bit.** Data symbols
are not emitted by `Mark` at all. They are emitted by the COFF writer's own
recursion — `0x10b28a9b`, guarded by `[sym+0x32] & 1` ("already written"),
re-entered from `0x10b28cb9` and `0x10b29057` — which is a *second* closure over
a *different* relation.

So the pruned Mark list says nothing about whether a **reference from an emitted
function to a data symbol makes that data symbol defined**. The full,
unpruned reference list `[head+0x14]` — which w-refs already decodes and then
throws away with `∩ U` — is exactly that edge.

**Registered claim:** `D` is not independently determined and is not predicted
from `.gl` in isolation (which is why w-joint's twelve static rules failed at
0.80985). `D` is the **data half of one least fixpoint** whose code half is
w-refs' closure, and the missing edge is `code -> data`.

---

## 1. THE MODEL — `JFP`, frozen here, with every fitted parameter named

Nodes are `.gl` names. Written as one relation:

    NODES
      U        gate-clean tag-0x0E `.gl` records            (w-refs `refs.scan`)
      W        names that own an `in` initializer record    (w-mark/w-skip)

    EDGES  (both taken UNRESTRICTED — this is the change)
      c->*   f  ->  every name its `.gl` reference list names, refcount != 0
                   [w-refs `reflist`, WITHOUT w-refs' `∩ U`]
      d->*   d  ->  every name an `02` node of d's `in` record names
                   [w-mark's channel, WITHOUT w-mark's `∩ U`]

    ROOTS
      Seed   { f in U : flags4c & 0x20 and not & 0x02 }     (w-roots, unchanged)

    GATE   a node not in `U` may enter the fixpoint only if it is in `W`.

    OUTPUT
      P      = live ∩ U        graded against `E`   (the code half)
      Dpred  = live ∩ W        graded against `D`   (the data half, DIRECTLY)

**The fitted parameters, and what varies across the registered variants:**

| parameter | value in `JFP` | varied by |
|---|---|---|
| code-edge target restriction | none | `JFP_URESTRICT` (restricted to `U`, = w-refs') |
| data-entry gate | `∈ W` | `JFP_UNGATED` (any `.gl` name may enter) |
| refcount-0 edges | dropped (w-refs') | `JFP_KEEPZERO` (kept) |
| root set | `Seed` only | `JFP_C1` (`Seed ∪ {__C1_*}`) |
| the `in` channel at all | present | `JFP_CODEONLY` (code edges only) |

Eight variants are scored, frozen: `RGL` (incumbent), `ORACLE` (w-joint's
ceiling), `JFP`, `JFP_UNGATED`, `JFP_URESTRICT`, `JFP_KEEPZERO`, `JFP_C1`,
`JFP_CODEONLY`. **No variant is added after truth is read.**

---

## 2. THE INCUMBENTS, per axis

| axis | incumbent | value |
|---|---|---|
| **code, micro-F1 vs `E`** | w-refs `RGL` | **0.85260** |
| **code, per-TU exact** | w-refs `RGL` | **132 / 850** |
| code, ceiling (**never a model**) | w-joint `ORACLE` | 0.97888 / 151 |
| **`D` predictor, downstream code-F1** | w-joint's best static `Rd`, `TAG_01` | **0.80985** |
| **`D` predictor graded AGAINST `D`** | **NONE EXISTS.** Established in this same pass by grading w-joint's twelve `Rd` rules against `D` directly | registered below as M9 |

The wash bar for clause 1 is **0.87260** (incumbent + 2.0 pp), w-skip's and
w-joint's bar unchanged.

---

## 3. `db` — the named instrument

Read from the binary before any corpus scan (`work/w-db/dis.sh`, reproducible):

* the five sub-stream names are contiguous at `0x10b13358` — `pch`, **`db`**,
  `sy`, `ex`, `in`, `gl` — with wide (UTF-16) twins;
* **`db` is sub-stream ordinal 4** (`push 0x4`, slot `[module+0x280]`), against
  `sy` 1, `ex` 2, `in` 3;
* it is **read at `0x10be7f41`** (`mov edx,0x10b1335c ; call 0x10b7e276`), inside
  a per-module loop at `0x10be7ef5` that is gated on **`[module+0xcd8] & 0x2000`**
  and feeds `0x10be997b` / `0x10be9892`;
* it is **written by the container writer only when `ds:0x10c40ef8 & 0x2000` or
  `ds:0x10c40ecc != 0`** (`0x10b73bb7`/`0x10b73bd3`);
* the first bytes of a real `db` show CodeView type leaves — `0x1201`
  `LF_ARGLIST`, `0x1008` `LF_PROCEDURE`, `0x1503` `LF_ARRAY`, `0x1505`
  `LF_STRUCTURE`, `0x00f1` padding.

**Registered reading: `db` is the DEBUG (CodeView type) sub-stream and it does
NOT determine `D`.** That is a null, so it is graded the way w-skip graded its
null — a value change must be shown to reach c2, and the instrument must be
shown to be capable of going red.

---

## 4. Registered numbers — POINT and INTERVAL are separate

The decline clauses key on the **interval**, except where stated.

### Instrument (`db`) and capture

| # | quantity | **point** | interval |
|---|---|---|---|
| T1 | `db` present and non-empty in the indexed entries | 850/850 | [845, 850] |
| T2 | `db` bytes per TU, median | 300 000 | [10 000, 5 000 000] |
| T3 | fraction of `D_all` names occurring as a NUL-terminated string anywhere in `db` | **0.02** | [0.00, 0.60] |
| T4 | fraction of `E` names occurring as a string in `db` | 0.02 | [0.00, 0.60] |
| T5 | CodeView leaf bytes recognised in the `db` prefix (positive check that `db` was read at all) | > 0 on 850/850 | ≥ 845 |
| T6 | KA-AGREE — my `E` == w-emit's `truth/` | 850/850 | [845, 850] |
| T7 | TOT residue, named and printed | 0 | [0, 500] |
| T8 | arity A1/A2/A3 TUs failing | 0/0/0 | [0,5] each |
| T9 | INJ conflicting definitions (w-joint measured 116 TUs / 338 names — a RED it characterised) | 338 | [0, 600] |

### The model, CODE half, graded against `E`

| # | quantity | **point** | interval |
|---|---|---|---|
| **M1** | `JFP` precision | 0.995 | [0.900, 1.000] |
| **M2** | `JFP` recall | 0.930 | [0.800, 0.980] |
| **M3** | **`JFP` micro-F1** | **0.960** | [0.860, 0.990] |
| **M4** | `JFP` per-TU exact | 145/850 = 0.171 | [0.100, 0.400] |
| M5 | `JFP_URESTRICT` F1 (code edges restricted to `U`, isolating the code→data edge) | 0.870 | [0.800, 0.960] |
| M6 | `JFP_CODEONLY` F1 (no `in` channel) | 0.900 | [0.700, 0.980] |

### The model, DATA half, graded DIRECTLY against `D` — the axis nobody has measured

Population: `W` (the `in`-owner names) per TU; positive class `D_all ∩ W`.

| # | quantity | **point** | interval |
|---|---|---|---|
| **M7** | `JFP` data precision | 0.990 | [0.850, 1.000] |
| **M8** | `JFP` data recall | 0.920 | [0.700, 0.990] |
| **M9** | **`JFP` data micro-F1** | **0.950** | [0.780, 0.995] |
| M10 | `JFP` data per-TU exact | 0.250 | [0.050, 0.600] |
| **M11** | **best of w-joint's twelve static `Rd` rules, graded against `D`** — establishing that no incumbent D-predictor exists | **0.45** | [0.10, 0.85] |
| M12 | base rate `\|D ∩ W\| / \|W\|` (w-joint: 0.05831) | 0.058 | [0.03, 0.12] |

### Calibration, stratification and controls

| # | quantity | **point** | interval |
|---|---|---|---|
| M13 | coincidence ratio, `JFP`'s new code marks over `P_RGL`, against uniform 0.03254 | 25× | [3×, 31×] |
| M14 | the same against the base rate `\|E\|/\|U\|` = 0.11577 | 7× | [1×, 9×] |
| M15 | `JFP` code F1 with `#152` excluded from both `E` and `P` | 0.975 | [0.880, 0.998] |
| M16 | `#152`'s share of `JFP`'s code residual | 0.55 | [0.20, 0.90] |
| M17 | KA-A — every incumbent reproduced to the digit (`\|U\|` 1 506 586, `\|E\|` 174 417, `\|E∩U\|` 173 907, `\|Seed\|` 14 662, `RGL` 129 604 / 1.00000 / 0.74307 / 0.85260 / 132, `ORACLE` 0.99997 / 0.95867 / 0.97888 / 151) | all | exact |
| M18 | **KA-POS** — the run GRADED something: `\|P_JFP △ P_RGL\|` printed | > 20 000 | > 0 |

### Mutations through real `c2.dll` — the directions that can fail

**MUT-DB.** Perturb the `db` stream on 3 non-quarantined TUs and replay.

| # | arm | **prediction** | pass |
|---|---|---|---|
| M19 | the obj **changes at all** (positive check that the write reached c2) | 3/3 | ≥ 2/3 |
| M20 | every section **except `.debug$*`** is byte-identical, and the defined-symbol set of the non-debug sections is unchanged | 3/3 | ≥ 2/3 |

M20 going **red** means `db` reaches the emit set; **clause 3 then puts it above
the headline.**

**MUT-CD — the code→data edge, the claim of §0.** Retarget one token of an
*emitted* function's `.gl` reference list, byte-length preserving (same `varU`
width), to an `in`-owner that is **not** defined in the baseline obj.

| # | arm | **prediction** | pass |
|---|---|---|---|
| M21 | **H+** — the function IS emitted → the retargeted data symbol becomes DEFINED | 5/5 per TU | ≥ 4/5, 3 TUs |
| M22 | **H− control** — same retarget in a function that is NOT emitted → it does not | 0/5 per TU | ≤ 1/5, 3 TUs |
| M23 | **positive control** — the baseline replay reproduces the pipeline obj's defined-symbol set | 3/3 | 3/3 |

M21 and M22 are the pair that makes the claim causal rather than correlational;
each can go red on its own, and **if H− comes back green-as-APPEARS the code→data
edge is refuted and §0 is wrong.**

---

## 5. Declared bias, and the outcome I most expect to be wrong about

I expect `JFP` to beat the incumbent on the code axis, because the disclosed
pilot (§9) already shows it doing so on three TUs. **The number I most expect to
be wrong about is M8, the data RECALL** — the pilot's data misses are a
structured family (`??_B` local-static guards, `?npos@…` static-member
constants, `__C1_11886`, `?gEaseFuncs`) and I have modelled none of them, so
recall is where a corpus will punish me. I have registered M8 at 0.920 with a
floor of 0.700 rather than hedging low.

Second: **M4, per-TU exact.** Micro-F1 can be excellent while every TU is off by
one name; the incumbent's 132 and the ceiling's 151 are 19 apart and a model with
recall 0.93 may sit under both.

---

## 6. Decline clauses

1. **If `JFP` code micro-F1 < 0.87260**, the model half is published as a
   **refuted hypothesis** in the first paragraph, and **I do not go looking for
   a further channel afterwards.** Every channel I decline is named in the
   findings' §"did NOT measure".
2. **If M7 (data precision) < 0.85**, the data half is published as an **upper
   bound**, not a model, with the coincidence calibration in w-mark's shape.
3. **If M20 goes red** — `db` changes the non-debug obj — that result goes
   **above the headline**, because it refutes §3's registered reading of the
   instrument this lane was commissioned to open.
4. **If M21/M22 fail to discriminate**, §0's correction of w-joint/w-skip is
   **withdrawn in the first paragraph** and the model is published as a
   correlation with no mechanism.
5. **No instrument tuning after truth is read.** The eight variants of §1, the
   edge definitions, the gate, the truth reader and the closure operator are
   frozen at this commit. Any change after a truth read is disclosed with its
   timestamp and the affected numbers are re-run from scratch.
6. **Nothing ships under `crates/`.** `PortC2` still returns `NotImplemented`
   outside its class; no `DISCLOSURE.md` row is owed unless that changes.
7. **`Rfloor` is not a decline key** — reported for comparability only.
8. **`ORACLE` is never quoted as a model.** It is a ceiling in every table.
9. **`db`'s null, if it is a null, is graded like w-skip's**: a no-op rewrite
   must reproduce the obj byte for byte and a value change must be shown to
   reach c2, or the null is about the instrument and is reported as such.

---

## 7. Registered before the numbers exist

* **TU match stays 8** at both ends; this lane changes no Rust.
* **`census/gate disagreement` stays 0.**
* **A high F1 is not a shippable predicate.** Order is untouched, and a right
  set in the wrong order is still a mismatch.
* **The 21-TU quarantine stays intact.** Every mutation TU is checked against
  `heldout.txt` by the script, before anything is written.

---

## 8. THE ONE-SHOT PART-1 GATE — owed, and NOT spent unilaterally

w-joint states the gate is owed by whoever first ships a model that predicts
`D`. **This lane has that model and it has fitted parameters** (§1's table), so
the gate is genuinely earned here and a held-out population is exactly what
catches the fitting.

**I will not spend it without asking the coordinator.** I will report the model,
its in-sample numbers, the fitted parameters and what varies across them, and
ask. If `JFP` is refuted in sample (clause 1), I will say so and stop, because a
held-out set cannot improve a refutation.

---

## 9. DISCLOSURE — the orienting pilot, run BEFORE this commit

Three TUs, all previously used by w-mark/w-skip/w-joint, none quarantined:
`src/system/net/HttpReq.cpp`, `src/system/utl/PoolAlloc.cpp`,
`src/system/rndobj/EventTrigger.cpp`. Scripts `work/w-db/pilot.py` and
`work/w-db/pilot2.py`, committed with this file.

**Pilot 1** (field cross-tab) established that no `.gl` kind-1 field separates
`D`: `+0x20 = 0x1c01` is 1-of-30 defined on `HttpReq`, and 59 of 68 / 86 of 106
defined data symbols (`$T<n>` temporaries) have no `.gl` record at all.

**Pilot 2** (the fixpoint) produced these numbers, and **the owner gate `∈ W`
was chosen after seeing the ungated false positives on these three TUs**
(`??_7type_info@@6B@`, `?TheDebug@@3VDebug@@A`, `?gStlAllocName@@3PBDB` — all
extern data referenced but defined elsewhere). That is the one fitted choice
made against data, it was made on three TUs, and it is disclosed here rather
than presented as a derivation:

| TU | CODE p / r / F1 | DATA p / r / F1 (ungated → gated) |
|---|---|---|
| `HttpReq.cpp` | 1.0000 / 0.9841 / 0.9920 | 0.6364/0.7778/0.7000 → **1.0000/0.7778/0.8750** |
| `PoolAlloc.cpp` | 1.0000 / 0.9610 / 0.9801 | 0.6923/0.9000/0.7826 → **1.0000/0.9000/0.9474** |
| `EventTrigger.cpp` | 1.0000 / 0.9171 / 0.9568 | 0.9361/0.9856/0.9602 → **1.0000/0.9471/0.9728** |

Per-TU exact was **False on all three, on both halves**. The registered points
in §4 are set *below* this pilot on every axis, because three TUs that three
previous lanes hand-picked are not a corpus.
