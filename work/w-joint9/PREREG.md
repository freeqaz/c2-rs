# w-joint (ninth wave, 2026-08-24) — PREREG

> **TAG COLLISION, recorded here because a citation cannot see it.** A lane
> called **`w-joint`** already exists: 2026-08-04 (`docs/rungs/_2026-08-04-w-joint-prereg.md`,
> `_2026-08-04-w-joint-findings.md`) plus `w-joint2` (2026-08-05), with **24
> references in `docs/BOARD.md`**. That lane measured the emit-set data+code
> fixpoint and has nothing to do with this one. This lane's scratch is
> therefore `work/w-joint9/` (its files are still on disk in `work/w-joint/`
> and are NOT touched), and its rung is dated `2026-08-24-w-joint.md`. Anyone
> citing "`w-joint`" without a date is citing two different lanes.

Committed as this lane's **first commit**, before any measurement is run.
Registered per `CLAUDE.md` "Method discipline" and decision 11 / board **#3505**.

Lane: `w-joint`, worktree `wt-w-joint`, base `67f276409`.
Reserved board rows: **#3506–#3511**.

---

## 0. THE QUESTION, restated in the units it will be answered in

> Does **blocker closure COMPOSE**? For the TUs nearest to matching, what is
> the **complete** blocker set of each blocked function — not the head key —
> and is there any TU whose full set lies within a bounded set of constructs?

The deliverable is **subset structure + a composition verdict**. It is
explicitly **NOT** a ranking; the rule against dispatching off a blocked-key
size ranking has bound five times (`#3505`), and this lane may not add a sixth.

---

## 1. THE DENOMINATOR — stated before the run, with what it counts

`docs/STATUS.md`'s generated block reads, at tree `b814d1db2` / workload
`a29f559d0`:

    TU distance to match, blocked functions | <=0: 17, <=1: 20, <=10: 29, <=100: 35, <=1000: 268
    878-TU dc3 workload scan               | match 25, mismatch 0, codegen-gap 0, vocab-gap 845, capture-fail 8

**`<=0: 17` and `match: 25` cannot both be naive readings of "how far is this
TU from matching".** The code says why, and the denominator is written against
the code rather than against the doc row:

* `GapReport::near_match_tus(k)` (`crates/c2-harness/src/gap/report.rs:465`)
  keeps TUs with `r.fn_total - r.fn_in_class <= k`, excluding `CaptureFail`
  and `fn_total == 0`.
* `fn_total` / `fn_in_class` come from `IlBundle::census_functions()`
  (`crates/c2-harness/src/gap/scan.rs:394-397`), which splits `.ex` at the
  census marker (`LO_MARKER = 4C 4F 11`) and classifies each segment with
  `FnVerdict`.
* `match` is decided by the **differential** — the port's whole obj against
  real c2's — and the port consumes `IlBundle::functions()`, which splits at
  the **gate marker `4F 1F`**. `gap/scan.rs:1334-1348` records that these are
  two different splitters.
* **#3364** already measured the consequence in the other instrument: three
  TUs `match` byte-exact while FBM *refuses* their body, because
  `codegen::select_function` and `IlBundle::functions()` are different routes
  and **neither dominates**.

**So this lane's denominator is stated as two sets, never one number:**

| name | definition | route |
|---|---|---|
| `D_census(k)` | TUs with `fn_total - fn_in_class <= k`, `class != CaptureFail`, `fn_total > 0` | census splitter `4C 4F 11`, `FnVerdict` |
| `D_match` | TUs the differential graded `Match` | gate splitter `4F 1F`, whole-obj byte compare |

Every count published by this lane names which of the two it is over.
**`D_census(0)` is NOT "TUs one step from matching" and this lane will not
call it that until the cross tabulation says it may.**

The lane's own population — the "nearest TUs" it ladders — is defined as

> **`NEAR = D_census(10) \ D_match`**, i.e. TUs with between 1 and 10 blocked
> census bodies that the differential does **not** already grade `Match`.

`STATUS.md` predicts `|D_census(10)| = 29`; `|NEAR|` is unknown before the run
and is itself a registered prediction (D4).

**Corpus, pinned and quoted beside every number** (`scripts/scan_pair.sh`'s
stamp form: `HEAD+porcelain-cksum+content-cksum`, read before and after every
arm):

    ../dc3-decomp   15a64d92f197+42949672950+42949672950   (clean tree)

Toolchain, exported explicitly on **both** arms of every pair (`#3470`):

    C2RS_COMPILERS=/home/free/code/milohax/c2-rs/compilers
    C2RS_WIBO=/home/free/code/milohax/wibo/build/wibo

An arm printing `SKIP: toolchain absent` exits 0 and grades nothing; every
arm's **key/TU denominator prints beside its numerator** and a zero
denominator is a VOID, not an agreement.

---

## 2. THE METHOD — a ladder, and its declared reach

`w-loo`'s standing instruction and `#3131`: **the port stops at the first
refusal BY DESIGN**, so `FnVerdict::Blocked(Block)` carries exactly one key
however many constructs a body needs. A head-key histogram is not a distance
and this lane may not build one.

The ladder is built **by lifting the clause in a scratch tree**, per the
prescription:

1. `work/w-joint9/scratch/` — a copy of the committed tree, gitignored,
   **never** the committed tree itself. The lifted state never ships.
2. A **named, enumerated lift set**. Each lift is a specific clause deleted or
   bypassed in `crates/c2-il/src/func/census.rs`, keyed by the census key it
   raises. The clauses that are mechanically liftable are the **post-parse
   gates** — the arms of the `match shape_to_function(...)` chain at
   `census.rs:1427-1670` plus the `fn-varargs` name gate at `census.rs:937` —
   and the shipped `Relax { sym_names: true }` switch (`census.rs:79-130`),
   which lifts the `callee-unresolved-*` / `data-sym-*` family.
3. Rung *k* re-runs `c2rs census <cpp>` (per-function verdicts, measured at
   **0.04 s/TU** warm) over `NEAR` with lifts `{L_1..L_k}` applied and records
   the **new head key of every function slot**. The per-slot sequence of head
   keys across rungs **is** that function's (enumerated prefix of its)
   complete blocker set.
4. Termination: a slot whose head key is a **parse-layer** refusal
   (`parse_segment_detail`'s `Err(b)`, e.g. `expr-op-0xNN`,
   `expr-load-type-XXXX`, `call-ref-cflow-jump`) is **UNLIFTABLE** by this
   ladder and is reported as such, with its terminal key, never as "closed".

**The ladder's reach is declared UP FRONT and is not total.** Parse-layer
constructs cannot be lifted without writing the decoder they name; this lane
does not write one. Where the ladder cannot climb, two **already-shipped**
multi-blocker instruments are harvested instead, and both are read, not
invented:

* `Complete` (`crates/c2-il/src/func/body/mod.rs:1512`) — `complete-whole:*`
  says the named construct finishes the body (**set size exactly 1** at the
  grammar layer); `complete-more:*` says something is behind it (**>=2**);
  `complete-none` says the key carries **no signal** (unknown).
* `mcall`'s greedy grant chain (`crates/c2-il/src/func/body/mcall.rs:993-1050`,
  `MAX_ADMIT = 4`) — for the `26`-in-expression family it already reports how
  many constructs it took to finish a body, i.e. an exact complete-set size in
  1..4, `-more` above that.

**The lower-bound argument, stated here so it cannot be presented later as a
post-hoc rescue.** For a TU `T`, the union of the head keys of its blocked
functions is a **lower bound** on the set of constructs whose closure `T`
requires. If that lower bound already exceeds a candidate bounded set, `T` is
unreachable by that set **whether or not closure composes** — a marginal-free
argument, and the one shape of answer the zeros-do-not-compose objection
cannot touch.

---

## 3. PREDICTIONS — registered before the run, scored in the rung

Probabilities are the lane's own priors. **Misses are the valuable part.**

### Denominator

| id | prediction | p |
|---|---|---|
| **D1** | `D_census(0)` is **not** a subset of `D_match`: at least one TU has 0 blocked census bodies and is not graded `Match` | 0.75 |
| **D2** | `D_match` is **not** a subset of `D_census(0)`: at least 3 TUs `match` byte-exact while carrying >=1 blocked census body (#3364's shape, in this instrument) | 0.80 |
| **D3** | `|D_census(0)| = 17` reproduces exactly on this tree and this corpus stamp | 0.85 |
| **D4** | `|NEAR| = |D_census(10) \ D_match|` lies in 15..29 | 0.70 |
| **D5** | The instrument-defect test (`--jobs 8` vs `--jobs 3`) moves no distance count | 0.95 |

### Subset structure

| id | prediction | p |
|---|---|---|
| **S1** | The **median** blocked function on `NEAR` has a head key that is **parse-layer / UNLIFTABLE** — i.e. the ladder's reach is the minority of the population | 0.70 |
| **S2** | At least one TU in `NEAR` has a head-key **union of size >= 5** over its blocked functions — a lower bound already past "a handful" | 0.65 |
| **S3** | The **smallest** head-key union over any TU in `NEAR` is >= 2 (no TU is one construct away even at the lower bound) | 0.60 |
| **S4** | Over `NEAR`, the union of all head keys is **> 10 distinct constructs** — larger than Phase 1's ten slices `C1..C10` | 0.75 |
| **S5** | At least one blocked function on `NEAR` reads `complete-more:*` or an mcall `need >= 2` — direct evidence of a set of size >=2 in shipped output | 0.85 |

### Composition

| id | prediction | p |
|---|---|---|
| **C1** | The ladder finds at least one function slot whose head key **changes** between rung 0 and a later rung — a second blocker exposed, i.e. closure of the first alone did not convert it (#150's shape, measured per-slot rather than in aggregate) | 0.80 |
| **C2** | **Zero** TUs in `NEAR` reach `blocked == 0` under the full liftable set | 0.70 |
| **C3** | The lift set is **not** additive on at least one slot: the head key after lifting `{A,B}` differs from what the singleton lifts predict | 0.35 |
| **C4** | The composition verdict is reportable as one of {composes / does not compose / not decidable by this ladder}, and it will be **"the bounded set is larger than Phase 1's ten constructs"** | 0.60 |

### Controls (required-zero / loud-failure)

| id | control | required |
|---|---|---|
| **K1** | The scratch tree with **zero lifts applied** reproduces the committed tree's per-function census verdicts on `NEAR`, slot for slot | identical, or the ladder is VOID |
| **K2** | Discriminating cells: the number of function slots whose head key moves at any rung is **printed**. Zero is a **loud failure**, never a silent pass (trap 5) | printed |
| **K3** | Every published count names its denominator in the same sentence | enforced |
| **K4** | The workload stamp is read before and after every ladder run; a move VOIDs the run | enforced |
| **K5** | Trap 0, stronger form: if the effect were **total** (every lift converts every function it touches), K2 would print `all slots moved`. The control can therefore distinguish total from null and is not green by construction | argued in the rung |

---

## 4. FENCES this lane binds itself to

* **No ranking.** No ordered list of keys to work on appears in any
  deliverable. Subset structure and a composition verdict only.
* **Owns** `crates/c2-harness` (instrument + tests) and `scripts/` for the
  ladder tool. **STOP AND REPORT** before modifying `crates/c2-il`,
  `crates/c2-core`, `crates/c2-obj`, `crates/c2-reference` **in the committed
  tree**. Lifts happen in `work/w-joint9/scratch/` only.
* `scripts/cost_arms.py` belongs to `w-adjacency` — untouched.
* The ladder tool is **committed** under `scripts/`, because `w-mixed`'s and
  `w-loo`'s both died in gitignored `work/` and that is the third such loss
  (`#3451`, the ir0 cost harness). #1406: anything whose output is quoted as
  evidence runs from the repo.
* Never commit IL (`_CL_*`, `*.il`), objs, `/target`, absolute machine paths
  in `crates/` or `scripts/`, secrets. No `Co-Authored-By` / agent trailer.
* std only, zero external crates in `crates/`.
* **No timing measurement is run by this lane at all** — so nothing here can
  contaminate `w-adjacency`'s cost rotation except CPU contention, which stops
  on a coordinator HOLD.

## 5. FAILURE MODES registered in advance

1. **The ladder cannot climb at all** — every head key on `NEAR` is
   parse-layer. Then the lane reports a **priced decline** of the ladder with
   the lower-bound result standing on its own, and states what a parse-layer
   lifter would cost.
2. **The scratch build diverges from the committed tree** for a reason other
   than the lift (K1 red). VOID; report and stop.
3. **`NEAR` is empty or tiny** — then the "nearest TUs" framing is itself the
   artifact, and the rung must say so in those words.
4. **A number that is stable but about the wrong thing** (`#3483`): a
   parameter test proves reproducibility, never attribution. Every count is
   additionally checked against a *second* route (the `gap` scan's per-TU
   `fn_blockers` vs `c2rs census`'s per-function rows) before it is published.

---

## 6. THE EMPTY-RUNG RULE — registered before the first rung is run

Raised by the coordinator at dispatch and adopted verbatim as a binding rule,
because it is **sharper for a ladder than it was for the scan pair that found
it**. `#3470`: `repo_root()` is `CARGO_MANIFEST_DIR/../..` **baked at compile
time**, so a binary built in a scratch tree under `work/` resolves
`compilers/` relative to *that* tree, finds none, prints
`SKIP: toolchain absent`, **degrades cleanly as `CLAUDE.md` requires — and
exits 0.** Every stamp read is correct and the whole log is one line.

For `w-3475` an empty arm looked like a broken run. **For a ladder it does
not.** Every rung's output is a judgement of the form *"did lifting this
clause move anything?"*, and a rung that graded **nothing** produces the same
observable as a genuinely inert clause: **no movement**. That is not an
obvious error; it is a substantive and completely wrong conclusion, and it
points toward the answer this lane might be half-expecting. **A silently empty
rung would let this lane report that closure does not compose when it had
simply stopped measuring.**

So, fixed **before** meeting one:

| id | rule | consequence |
|---|---|---|
| **K6** | `C2RS_COMPILERS` and `C2RS_WIBO` are exported explicitly for **every** scratch-tree binary invocation, not only the first | enforced in `scripts/joint_ladder.py` |
| **K7** | Every rung prints its **denominator triple**: TUs graded, function slots walked, distinct head keys emitted — beside every numerator | printed per rung |
| **K8** | A rung whose denominator is **zero on any component**, or whose log contains `SKIP: toolchain absent`, or whose probe exits nonzero, is a **VOID**. The tool **refuses and exits nonzero**; the ladder is not continued and no "no movement" is recorded for that rung | hard failure, never a data point |
| **K9** | Zero `SKIP` is **proven per rung**, not assumed — `w-permute`'s pattern. The count of `SKIP` lines is printed as `skips=0`, positively | printed |
| **K10** | After every rung, `git status --porcelain` in the **committed** worktree is checked and must be clean of any `crates/` modification. The lifted state lives only in `work/w-joint9/scratch/` and its escape is checked for, not assumed | printed per rung |
| **K11** | Rung 0 (zero lifts, scratch tree) must reproduce the committed tree slot-for-slot (**K1**). This doubles as the toolchain proof for the scratch build: if the scratch binary could not see the toolchain, K1 is red rather than silently green | enforced |

**K11 is the one that makes the rest cheap**: it is a required-*identity*
control at nonzero denominator, so the failure mode `#3470` describes cannot
present as agreement.

`scripts/scan_pair.sh` exits **4** on exactly this family (a `SKIP`, a nonzero
arm exit, or a zero key count). `scripts/joint_ladder.py` adopts the same
refusal shape and the same exit code.
