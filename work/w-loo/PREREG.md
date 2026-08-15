# w-loo — PREREG

Frozen as this lane's **first commit**, before the first re-scoring run.
Only `work/w-loo/base.jsonl` (the unmodified-tree control, `match 25 ·
mismatch 0 · fnbyte-exact 35,734`) has been run at freeze time; it is a
control, not a re-scoring.

Lane kind: **construct** (instrument). Predicted `crates/` byte delta: **zero**.

---

## 0. THE QUESTION

`w-read2` (#3131) found that leave-one-out marginals over the statement layer
sum to **98,039** against a total reach of **5,184** — **18.9×** — and that
`op:29` reads **0** as a rung-1 greedy grant and **5,184** as a LOO margin.
Every published ladder in this repo is a greedy head-mass climb. This lane
re-scores the published ladders through leave-one-out and reports, per ranking,
**greedy / LOO / ratio**, plus which conclusions change.

**This lane dispatches nothing off the new numbers.** The repo is 0-for-4 on
lanes dispatched off a mass ranking (MEMORY "ranking instruments measure
themselves"); a better ranking is not a licence to trust it.

---

## 1. THE DENOMINATORS — registered in advance, one row per number I will publish

**The error this repo has made three times in three waves (#3092, #3107, #3125)
is quoting a counterfactual number as a base number.** Every number this lane
publishes is registered here with its denominator and its population *before*
it is measured. **Every one of them is counterfactual except D5.**

| id | number I will publish | denominator | population is… |
|---|---|---|---|
| **D1** | expression-layer LOO margin per token | **the full-52-token expression-layer reach** (`expr-chain-fntail` under `C2RS_SINK_CHAIN=<ceiling_with.txt>`), expected ≈ 88,806 | **COUNTERFACTUAL** — functions blocked at base that a 52-token sink walks to the tail. Ships nowhere; no token of it is accepted |
| **D2** | statement-layer LOO margin per token | **5,184** = `stmt-chain-fntail` 3,684 + `rsc-chain-fntail` 1,500 under `C2RS_SINK_STMT=<ceiling>` | **COUNTERFACTUAL**, and doubly so — the statement layer cannot de-accept at all (read2 §3) |
| **D3** | Ladder B greedy per-round delta | **41,762**, Ladder B's own round-13 reach (readphase §3) | **COUNTERFACTUAL** — a scaffold+13 spec that ships nowhere. Its round-0 base is itself a 9-token scaffold, not the tree |
| **D4** | emitted widening-order mass (`expr-op-0x27` = 22,407 etc.) | **113,612** `fnbyte-refused-parse` over **615** keys, 878 TUs | **BASE** for the numerator, but the *rank* is a first-blocker rank — a key is counted only where it blocks FIRST |
| **D5** | `match` / `mismatch` / `fnbyte-exact` | **878 TUs** / **162,049** functions | **BASE — the only real one on this page.** 25 · 0 · 35,734 |

**Standing rule for this lane's prose: numerator and denominator in the same
breath, every time.** A margin is quoted as `N of <full-set reach>`, never bare.

**No number on this page is `match`-denominated. No ranking here has ever
forecast a conversion, and this lane will not claim one does.**

---

## 2. WHAT COUNTS AS "SURVIVES" — thresholds fixed now so they cannot move

Two questions, reported **separately** (a ranking whose order survives is a
different result from one whose magnitudes survive):

- **Survives by ORDER**: Spearman rank correlation **ρ ≥ 0.50** between the
  published order and the LOO order, over the items both score. Reported with
  the item count `n`; **ρ on n < 4 is not reported as a survival verdict.**
- **Survives by MAGNITUDE**: **every** scored item within a factor of **2**
  (`0.5 ≤ LOO/published ≤ 2.0`). One item outside → the ranking does not
  survive by magnitude. Items published at 0 are scored by absolute difference.
- **INVERTS** (the headline case): ρ ≤ **−0.30**, or the published #1 item has a
  LOO margin of **0** while a published-0 / unranked item has a positive one.

**Discriminating cells are printed for every re-score**: how many items *could*
have disagreed. A ranking where no cell could have moved is reported as
**VACUOUS**, not as "survives" (the precedent is a lane here whose two
"0 disagree" results were structurally vacuous).

---

## 3. THE PREDICTIONS, with probabilities. Ceiling with NO discount factor.

### 3.1 Scope

| id | registered | p |
|---|---|---:|
| **S1** | The inventory names **≥ 12** distinct published rankings across `docs/` + `docs/BOARD.md` | 0.80 |
| **S2** | **≥ 4** of them are re-scored with new scans (the rest from published numbers or declined with a stated reason) | 0.85 |
| **S3** | **≥ 3** are declined as not re-scoreable by this instrument, each with the reason named | 0.70 |

### 3.2 The expression layer — the ladder the roadmap is ranked off

| id | registered | p |
|---|---|---:|
| **E1** | Expression-layer sum-of-margins / full reach is **> 1.5×** (some conjunction structure exists) | 0.85 |
| **E2** | …and it is **< 18.9×**, i.e. the expression layer is *less* super-additive than the statement layer | 0.65 |
| **E3** | The greedy #1 of Ladder B by round-delta (`op:BD`, +26,051 of 41,762) is **not** the LOO #1 | 0.65 |
| **E4** | **≥ 1** Ladder B token with a strictly positive greedy delta has a LOO margin of **exactly 0** | 0.70 |
| **E5** | **≥ 1 SCAFFOLD token** (`41 4F 53 54 4B 29 38 39 3A` — granted free at round 0 and therefore never ranked) has a LOO margin **≥ the largest margin of the 14 ranked tokens**. *If this hits, the ladder's seed outranks everything the ladder ever chose, and no published number says so.* | 0.75 |
| **E6** | `op:27` — board **#1333**'s "the BOARD'S #1 ROW", 22,407 of 113,612 by mass — has a LOO margin **< 1,000 of the full reach** | 0.60 |
| **E7** | Ladder B **does not survive by MAGNITUDE** (§2 threshold) | 0.88 |
| **E8** | Ladder B **does not survive by ORDER** (ρ < 0.50) | 0.55 |
| **E9** | Ladder B **INVERTS** (§2 threshold) | 0.30 |
| **E10** | At least one of `op:54`, `op:3A`, `op:29` is worth the **whole** expression-layer reach at the margin, as it is on the statement layer | 0.55 |
| **E11** | Count of the 52 tokens with a **nonzero** expression-layer margin is **≥ 30** | 0.70 |

### 3.3 Survival, counted

| id | registered | p |
|---|---|---:|
| **V1** | Of the rankings re-scored with scans, **0** survive by MAGNITUDE | 0.70 |
| **V2** | Of the rankings re-scored with scans, **exactly 1** survives by ORDER | 0.40 |
| **V3** | **≥ 1** ranking survives by ORDER (i.e. not all of them break) | 0.75 |
| **V4** | The **per-TU frontier ladders** (`w-ladders` §2's net/step/**lb**) are the family that survives best — they are the only published family that already carries a LOO column (`lb`), and its published cost is only **154 → 147 net→lb, 4.5 %** | 0.65 |

### 3.4 Controls — the instrument must be shown able to fail

| id | registered | p |
|---|---|---:|
| **C1** | The statement-layer LOO **reproduces `w-read2` §5.4 exactly** — `op:54`/`op:3A`/`op:29` margin **5,184** each, sum of marginals **98,039**, **18.9×** — *even though this lane runs it COMPOSED with the chain sink in one scan*. This is simultaneously an independent test of read2 §4's **orthogonality** claim | 0.70 |
| **C2** | **MUTANT (must go RED)**: corrupting the terminal key name so the full-set reach reads 0 makes the driver **REFUSE** (non-zero exit), not report 52 margins of 0 | 0.90 |
| **C3** | **NEGATIVE CONTROL (must read exactly 0)**: a token absent from the ceiling spec has margin **0** — removing what is not there changes nothing. A nonzero here means the harness is not measuring what it says | 0.90 |
| **C4** | **MUTANT (must go RED)**: a driver that silently accepts a scan with < 800 graded TUs is detectable — the guard fires when fed a truncated stream | 0.85 |
| **C5** | Discriminating-cell count for the expression-layer re-score is **> 0** and is printed. *"Absence is not success": every check is reported positive on content* | 0.95 |

### 3.5 Required-zero (this lane is docs+`work/` only)

| id | registered | p |
|---|---|---:|
| **Z1** | `crates/` byte delta **zero**; `match` 25, `mismatch` 0, `codegen-gap` 0, `vocab-gap` 845, `capture-fail` 8, `frontier` 2 | 0.97 |
| **Z2** | `fnbyte-exact` **35,734**, `fnbyte-refused-parse` **113,612** | 0.97 |
| **Z3** | 372 `gap-metric` keys and 878 verdict lines identical at both ends | 0.95 |
| **Z4** | `cargo test --workspace --release` **1,602 / 0 / 42**, delta **+0** (this lane adds no test to `crates/`) | 0.90 |

### 3.6 What I expect to be UNABLE to do — registered so a null is not dressed up

| id | registered | p |
|---|---|---:|
| **N1** | `CEILING.md` §2.4's section-name ladder (`.rdata$r`/`.text$yd`/`.xdata$x`) is **NOT** re-scoreable by this instrument — it is a COFF-writer/factor-C ladder with no sink token — and is declined with that reason rather than force-fitted | 0.85 |
| **N2** | `CFG_SHAPE.md` §5.2's CFG-class ladder is likewise **not** token-addressable and is declined | 0.85 |
| **N3** | `LABEL_COUNTER.md`'s ladders are a **charge law**, not a reach ranking, and are out of scope | 0.80 |
| **N4** | LOO **cannot** see the `54`+`3A`+`29` conjunction from marginals alone; **subset structure must be probed separately** and this lane will run ≥ 4 explicit subset cells rather than inferring conjunction from the 18.9× alone | 0.90 |

---

## 4. WHAT LEAVE-ONE-OUT CANNOT SEE — registered BEFORE publishing any LOO number

The trap named in the brief, and it is the one this lane is most likely to fall
into: **the replacement instrument can be as wrong as the one it replaces.**

1. **LOO is a MARGINAL against a full set. It cannot see a token worth nothing
   alone and everything in a triple.** The `54`+`3A`+`29` finding is exactly
   that shape, and LOO found it only because the conjunction was *complete* in
   the full set. A conjunction whose members are all **outside** the ceiling
   spec is invisible to LOO the same way a terminator is invisible to a
   first-blocker rank.
2. **LOO's denominator is a counterfactual population** (D1/D2/D3). A margin of
   "5,184, the whole reach" is 5,184 of a number that ships nowhere and is
   **0 of 878 TUs and 0 of 35,734 fnbyte-exact**.
3. **Sum-of-marginals ≫ total diagnoses super-additivity but does not locate
   it.** 18.9× says "conjunction"; it does not say *which* conjunction. Only the
   subset cells do.
4. **LOO is symmetric in a way the goal is not.** It scores removal from a full
   set nobody will ever ship. The shipping question is *addition to the tree*,
   which is the greedy direction — so LOO is not simply "the right instrument":
   it answers the complementary question, and **neither answers the goal
   question**, which is `match`-denominated conversion.
5. **LOO is O(n) scans and therefore blind to interactions of order ≥ 2 by
   construction.** With 52 tokens there are 1,326 pairs; this lane runs a
   handful of hand-chosen cells, so **"no conjunction found" would be a
   statement about the cells run, never about the space.**
6. **A token with margin 0 is not "worthless"** — it may be perfectly
   substitutable by another token in the set (redundancy reads identically to
   irrelevance under LOO). Distinguishing them needs a leave-*two*-out, which
   this lane does not run at scale.

**No LOO number in this lane's output may be quoted without §1's denominator
attached, and none of them is evidence of a conversion.**

---

## 5. Reproduction

```sh
./work/w-loo/scan.sh base                       # the control: match 25, fnbyte-exact 35,734
python3 work/w-loo/loo2.py                      # the re-score, 53 scans, both layers in one pass
python3 work/w-loo/subsets.py                   # the conjunction cells LOO cannot see
python3 work/w-loo/rescore.py                   # greedy/LOO/ratio tables + Spearman + cells
```

`.jsonl` scan streams are **NOT tracked** (`w-readphase` committed 1.85 GB by
accident and had to `filter-branch` it out). Drivers and summaries only.
