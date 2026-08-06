# w-spell — PREREG

    Lane:      w-spell
    Branch:    w-spell, worktree off master `33a1867`
    Date:      2026-08-06
    Committed: BEFORE any probe file exists. Every grid gets a dated addendum
               in this file, committed before that grid's generator is written.

This file is the contract. Anything measured that is not registered here (or in
a dated addendum below, committed before its grid existed) is reported as
**unregistered** and cannot be scored as a hit.

---

## 0. What is already on record, and what this lane is for

Four lanes agree that no allocation key on record survives off its own cells:

| key | fit | fresh holdout |
|---|---|---|
| w-next's `uses + (register-derived ? 1 : 0)` | 24/24 | **7 wrong of 56** (w-alloc2) |
| `H-self` (bonus ~1.5 uses for a self-pointing producer) | 80/81 | **11 wrong of 72** (w-refbind) |
| clause-1-strict (strictly more uses takes `r11`) | — | **12 wrong of 36** (w-seam GRID A) |

and that the separating axis is the producer's **spelling** — the opcode c2
selects — which `ProducerKind` cannot represent. The evidence, restated from the
three rungs so this file can be scored without them:

* `addi rX,r3,K` (an interior address stored into the object it derives from)
  wins a 1-use-vs-1-use tie; `add rX,r4,r5`, `addi rX,r4,5` and `slwi` lose it
  (w-alloc2 §3.1).
* `extsh` and a rematerialised `lwz` win the same tie **in both binding modes**
  (w-refbind §6.1) — four of the eleven misses that killed H-self.
* At (bind present, reg 2 uses, const 1): `add addi srawi extsh` win;
  `slwi srwi sub and or xor neg nor` lose (w-refbind §6.2, board **#862**).
* `slwi` loses at a use-count advantage of **three** as flatly as at one
  (w-seam GRID A, 36/36 graded, 12 miss, board **#868**).
* The bind is not the axis: what matters is whether the body carries **more
  than one distinct store-base value** (w-refbind §5.2, board **#865**) — six
  discriminating rows, **no holdout**.

**This lane maps the spelling axis as a population and gives #865 its holdout.**
It is a measurement lane. Publishing the table is the deliverable even if — and
the prior four lanes say *especially if* — no rule states.

## 0.1 The INCUMBENT, registered as a rival and not as a threshold

The control any candidate must beat is **the shipped refusal in
`crates/c2-core/src/codegen/alloc.rs`**: a run mixing a constant and a
register-derived producer returns `None`, so the emitter refuses.

    incumbent:  right 0 | WRONG 0 | refused N     on every population, always.

**A refusal is never wrong.** A candidate rule that is wrong on one cell of any
population in this lane therefore **loses to the incumbent** and is not
proposed. This is registered as a rival rather than as a bare threshold because
STATUS trap 4 says a control that only checks a total is not a control: the
incumbent's WRONG column is structurally 0 and the candidate's is not, so the
two columns are the comparison and a "0 misses" line on its own is not.

## 0.2 What this lane will NOT do

* It will **not** ship any allocation key into `crates/`. Shipping is
  unauthorized unless a rule survives §5's holdout with **zero** misses **and**
  decides every recorded refutation cell of w-next / w-alloc2 / w-refbind /
  w-seam correctly — and even then what may be proposed is a **wider refusal
  boundary** test suite, with the emit decision left to the coordinator.
* It will **not** cite `gate.sh` or the 878-TU scan as support for any schedule
  or allocation fact. w-alloc2 §5.1 ran that control: a rule measurably wrong on
  20 of 81 cells left all 62 `gap-metric` lines byte-identical, because #840
  means no register-derived producer can reach `alloc::allocate` from today's
  emitter. The gate here is an **inertness check on a lane that ships nothing**.
* It will **not** fit anything on the cells of §5's holdout. If the holdout
  refutes, the miss is the deliverable and the lane STOPS there.

---

## 1. Instrument design, registered before it is written

Two prior lanes lost cells to their own graders. Both mitigations are
registered here rather than discovered again.

1. **No producer regex may name a source register.** w-refbind's `refprobe`
   scored the reference-formal cells out of regime because adding a formal moved
   `u`/`v` from `r4`/`r5` to `r5`/`r6` and the regexes were anchored on `r4`.
   The grader in this lane reads the producer's register **off its own store**:
   for a store `st* rS, DISP(rB)` at a DISP this grid assigned to the producer's
   run, `rS` is the producer's register. Nothing is anchored on `r3`, `r4` or
   `r5`, and the constant is read the same way.
2. **The mnemonic is OBSERVED, never assumed.** Board #843 has fired three times
   (`sub` not `subf`, `slwi` not `rlwinm`, and again in w-refbind's first grade
   run). This grader does not match a per-spelling regex at all: having found the
   producer's register it finds the instruction that **defines** that register
   and *records the mnemonic c2 printed*. The population table is therefore keyed
   on what c2 emitted, not on what the C++ was expected to compile to.
3. **#644 enforcement.** A register that is defined more than once in the body
   (a rematerialised `lwz`, a two-instruction `addi`+`addi` interior address) is
   **out of regime**, never a hit and never a miss. `reached`, `graded`,
   `out-of-regime` and `compile-failed` are four separate printed counters
   (STATUS trap 5).
4. **Every store in a cell gets a DISTINCT displacement**, so the emitted store
   order maps back to source statements unambiguously without any positional
   reader (#644).
5. **ORDER and ALLOC are read separately** in every cell and printed side by
   side. w-alloc2 §3.1 and w-refbind §3.2 both turn on their being separable.

---

## 2. GRID S — the population table (§ the deliverable)

**Registered before `spellgrid.py` exists.** Dimensions:

* **spelling** — the producer expression, ~16 of them, chosen to cover the
  opcode families c2's `/O1` selects for a store-run producer: interior address
  (self and cross), `addi` immediate, `add`, `sub`, `and`, `or`, `xor`, `neg`,
  `nor`, `slwi`, `srwi`, `srawi`, `extsh`, a `lwz` load, and a formal copy.
* **use counts** `(ru, cu)` — `(1,1) (2,1) (3,1) (2,2) (2,3)`. The last two are
  in because w-refbind §6.2 reports `add` winning at `(2,3)` where `slwi` loses,
  which no additive key over `ru` and `cu` can produce.
* **store-base structure** — `1base` (one pointer formal, no bind) and `2base`
  (the same body plus `L& q = s->inner;`, a bind at a non-zero displacement,
  which #865 says is one way to make a second store-base value).

Cells: 16 × 5 × 2 = **160**, all compiled at the workload's own
`/O1 /Oi /EHsc /GR` through `work/w-frame/refobj.sh` against real `c2.dll`, with
a spot-check partition at `/O1 /GS- /c` (§4).

### 2.1 Registered predictions — S1…S6

| # | claim | what would kill it |
|---|---|---|
| **S1** | **Every one of the 16 spellings resolves to a single defining instruction and the table is ≥ 80 % graded.** | fewer than 128 of 160 cells graded |
| **S2** | **The outcome is a function of (observed mnemonic, ru, cu, bases) and of nothing else this grid varies** — i.e. two cells agreeing on all four agree on the winner | any two such cells disagreeing |
| **S3** | **NO additive/linear key over `ru`, `cu` and a per-spelling constant fits the whole table.** Registered as the claim I expect to WIN, stated so it can lose: an exhaustive search over per-spelling integer bonuses `b ∈ [-8,8]` with the rule `2·ru + b > 2·cu` will leave ≥ 1 residual cell | such a key fitting every graded cell |
| **S4** | **The `1base`→`2base` move takes wins away and never gives one.** No spelling that loses at `1base` wins the same `(ru,cu)` at `2base` | one such cell |
| **S5** | **`srawi` and `srwi` — the same C-level operation at different signedness — land in DIFFERENT groups.** This is the sharpest statement that the axis is the selected instruction and not the source expression | they agree everywhere |
| **S6** | **`extsh` and `lwz` win at `(1,1)` in both base modes** — w-refbind's four non-#839 misses reproduce in this lane's instrument | either loses a `(1,1)` cell |

**S3 is the load-bearing negative.** If it loses, a linear key exists and the
next step is §5's holdout on it. If it wins, the axis is a per-opcode lookup and
the honest deliverable is the table.

## 3. What a "rule" would have to be, registered in advance

Whatever GRID S shows, a candidate rule is only worth taking to §5 if it is
stated as a **total function of observable IL/instruction features**. A rule of
the form *"look the spelling up in this table of 16"* is **not a rule** — it
cannot decide a spelling it has not seen, and the workload contains opcodes this
grid does not. So the candidate must come with a **stated principle** that
assigns an unseen mnemonic to a group, and §5 grades that principle on mnemonics
GRID S never contained. **This is registered now so that a table cannot be
promoted to a rule after the fact.**

## 4. Flag spot-check — S7

**S7:** the ten cells re-compiled at `/O1 /GS- /c` (the brief's flags) instead of
the workload's `/O1 /Oi /EHsc /GR` give the **same** winner in every cell.
Registered as expected-to-hit; a miss means every allocation figure on this
project's record is flag-conditional and that is the finding.

## 5. GRID H — the frozen holdout for whatever GRID S produces

Two-phase, w-refbind's discipline verbatim: `--freeze` writes every source and
a `holdout_pred.tsv` carrying the candidate's prediction **and each source's
sha256**, compiles nothing, and is **committed**; `--grade` re-checks every
sha256 before grading and reads the frozen prediction column rather than
recomputing it.

Axes GRID S does not vary, all of which GRID H does:

* **fresh mnemonics** — `andi`/`ori`/`xori` (immediate logicals), `subfic`,
  `extsb`, `andc`, `slw`/`srw`/`sraw` (variable shifts), `mullw`, a halfword
  and a byte load. The §3 principle must place each of these **before** any is
  compiled.
* **fresh use counts** — `(4,1)`, `(1,2)`, `(3,3)`.
* **a fresh signature** — an extra formal, so every register moves (this is the
  axis that broke w-refbind's grader; the §1 instrument is built for it).
* **fresh struct offsets** and a fresh constant value.
* **three producers** rather than two.

| # | claim |
|---|---|
| **H1** | the candidate's frozen predictions are graded on ≥ 40 never-fitted cells |
| **H2** | **the candidate MISSES at least one cell.** Registered as the outcome I expect, on the record of the three keys in §0. If it misses, it **loses to the incumbent** (§0.1) and nothing is proposed | 
| **H3** | if H2 loses — zero misses — the candidate is additionally scored on **every recorded refutation cell** of the four prior lanes, and is proposed only as a **wider refusal boundary** test suite if that too is 0 wrong |

**What would kill the candidate, stated now:** one wrong cell anywhere in GRID
H, or one wrong cell on any prior lane's recorded refutation, or an unseen
mnemonic the §3 principle places in the wrong group. Any of the three and the
lane reports a refutation and ships nothing.

## 6. GRID B — #865's holdout (schedule side, measurement only)

**#865:** *the schedule pins to source order iff the body carries more than one
distinct store-base value.* Six fitted discriminating rows, no holdout.

Frozen the same way: predictions and sha256 committed before compiling. Every
cell's source store order is **constant-run first, producer-run second**, which
is the order w-refbind found the unbound schedule breaks (it hoists the
producer's stores above the constant's), so every cell discriminates rather than
being satisfied by an order that already matched.

Axes the six fitted rows did not vary:

* **base counts 3 and 4**, not just 1 and 2;
* a base that is **derived** (`S* p = s->next;`) rather than a formal;
* **three runs across two bases where the constant and the producer SHARE a
  base** — the rival w-refbind §5.2 named explicitly and did not build:
  *"more than one distinct store-base value"* says PINNED, *"the constant's and
  the producer's stores have different bases"* says NOT;
* a bind at displacement **0** beside a genuine second base;
* two producer spellings, one from each group GRID S finds.

| # | claim |
|---|---|
| **B1** | ≥ 20 cells graded, with `reached`/`graded`/`out-of-regime` printed separately |
| **B2** | **#865 predicts every graded cell.** Registered as the claim most likely to LOSE, because it is a post-hoc description with six rows and that is exactly the standing H-self had |
| **B3** | **the 3-runs-2-bases-shared cell separates #865 from its rival**, and the rival is scored beside it |
| **B4** | nothing is shipped from GRID B under either outcome — the schedule side is measurement only, per the brief |

## 7. Warranty — W1…W3

| # | claim |
|---|---|
| **W1** | the 878-TU scan reads **match 10 · mismatch 0 · codegen-gap 0 · vocab-gap 861 · port-error 0 · capture-fail 7** and **`fnbyte-differs` 0** at BOTH ends, and all 62 `gap-metric` lines are byte-identical, checked by `diff` |
| **W2** | `scripts/gate.sh --jobs 6` is 18/18 PASS, 0 mismatch, at a tree whose `crates/ scripts/ fixtures/` diff against the gate's tree is empty |
| **W3** | `cargo test --workspace --release` is aggregated across every `test result:` line into a printed `targets=/passed=/failed=`, measured at BOTH ends, never through `tail` or `head` |

**Baseline, measured at `33a1867` before this file was committed**
(`work/w-spell/baseline_scan.txt`): match 10 · mismatch 0 · codegen-gap 0 ·
vocab-gap 861 · port-error 0 · capture-fail 7 · FRONTIER 17 · `fnbyte-differs`
0 · 62 `gap-metric` lines.

---

## 8. Addenda

Each addendum is dated and committed **before** the grid it declares exists.
