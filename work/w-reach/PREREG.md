# w-reach — PRE-REGISTRATION

    Lane:   w-reach, 2026-08-04, worktree `wt-w-reach` off master `f351a32`
    Task:   price the tag-0x10 ALIAS channel in TU-REACH terms, by MEASURING
            `|{model-exact} ∩ B∧C|` for four models — never by scaling.
    Ships:  `crates/c2-harness/` only — the per-TU factor membership that
            `gap.rs` does not publish today.  No `crates/c2-core`, no
            `crates/c2-il`, no `scripts/`, no `fixtures/`.

**Committed BEFORE any intersection is computed.** The rule this lane exists to
honour is the one w-emitp refused to break: `B∧C` sat published at **107** for
weeks because it had been measured at `C = 114` and then scaled/assumed forward.
It is **151**. Multiplying `151 × 0.555` to price the alias channel would be the
same error one level up, so every number below is registered as a *point and an
interval* and then measured.

---

## 0. What is already known, and is NOT re-derived here

From `docs/rungs/_2026-08-04-w-emitp-findings.md` §2.2 / §5, over **850** TUs:

| model | kind | micro-F1 | per-TU exact / 850 |
|---|---|---:|---:|
| `RGL` / `JFP` | incumbent model | 0.85260 / 0.92655 | 132 |
| `ORACLE` | **ceiling** (conditions on `D`) | 0.97888 | **151** |
| `JFP_ALIAS` | **model**, no oracle | 0.94413 | **308** |
| `ALIAS_IN` | **ceiling** | 0.99243 | **472** |

From `c2rs gap` at `316e1c4` / `c303ad0`, over **871 graded** TUs of 878:
`A 28 · B 338 · C 169 · D 8 · E 2`, `B∧C 151`, `A∧B∧C 27`, FRONTIER 19,
frontier-if-A 141.

**Two different denominators (850, 871) and no published join.** §3 registers
what I expect the join to be, and it is scored first, because an intersection
across a mismatched join is worse than no number at all.

---

## 1. Hypothesis

**H0 (the null).** `{model-exact}` and `B∧C` are independent over the graded
TUs, so `|{exact} ∩ B∧C| = |exact| × 151/850 = |exact| × 0.17765`.

**H1 (what I actually expect).** They are **positively** correlated, so every
intersection lands **at or above** the H0 product. The mechanism I am claiming
in advance, so it can be wrong:

* **B** ("every emitted symbol binds", `emit-set-ceiling-today`) and
  model-exactness are two implementations of the *same underlying fact* — that
  the `.gl` stream accounts for every emitted name in this TU. They are not the
  same code path (B is `EmitBinding` in `crates/`; exactness is w-emitp's
  closure in Python) and they are not the same predicate, but they cannot be
  independent.
* **C** ("obj section set ⊆ the writer's 10 names") is a property of the obj's
  *containers*, not of its symbol set, so I expect C to be **near-independent**
  of exactness — with a mild positive tilt, because small/simple TUs satisfy
  both.

So the correlation should enter almost entirely through B, and `B∧C / C = 89.3 %`
means B is nearly free once C holds. That caps how much lift the correlation can
buy: the intersection cannot exceed `|C ∩ exact|`, and `C = 169 / 871 = 19.4 %`.

**H1 therefore predicts each intersection lands between the H0 product and
roughly 1.6× it.**

---

## 2. Registered points and intervals — the four intersections

Denominator note: each is capped at `|B∧C ∩ (the 850)|`, which §3 predicts is
**≤ 151** and is itself measured, not assumed.

| # | quantity | H0 product | **registered point** | interval |
|---|---|---:|---:|---|
| **R1** | `\|{JFP-exact} ∩ B∧C\|` — the incumbent model | 23.4 | **30** | [10, 70] |
| **R2** | `\|{ORACLE-exact} ∩ B∧C\|` — the **previous ceiling** | 26.8 | **34** | [12, 75] |
| **R3** | `\|{JFP_ALIAS-exact} ∩ B∧C\|` — **the implementable model** | 54.7 | **72** | [30, 145] |
| **R4** | `\|{ALIAS_IN-exact} ∩ B∧C\|` — the **new ceiling** | 83.8 | **106** | [55, 151] |

**R5 — the increment that is the lane's headline.** `R3 − R1` = what a
codegen-free implementation of the alias channel is worth in reach, over the
incumbent. Point **+42**, interval [+10, +110].

**R6 — the increment of the ceiling.** `R4 − R2`. Point **+72**,
interval [+25, +130].

### 2.1 The two structural numbers, also registered

Reach is not conversion. A TU converts only under `A∧B∧C∧(D∨E)`; substituting a
model for A gives `{exact} ∧ B ∧ C ∧ (D∨E)`.

| # | quantity | point | interval |
|---|---|---:|---|
| **R7** | `\|{JFP_ALIAS-exact} ∩ B∧C ∩ (D∨E) ∩ ¬match\|` — TUs a perfect implementation of this model would convert with **ZERO codegen** | **2** | [0, 8] |
| **R8** | `\|{JFP_ALIAS-exact} ∩ B∧C ∩ ¬(D∨E) ∩ ¬match\|` — the codegen frontier under this model | **68** | [25, 140] |

R7's point is 2 because `factor_projection_divergence` is currently exactly 2
TUs (`src/system/decomp_pch.cpp`, `src/system/math/vec.cpp`) — the TUs inside
`B∧C` that fail A and that the port already accepts. **If the model is exact on
both, R7 = 2 and the alias channel converts two TUs the moment it ships inside a
Phase-7 emit-set model.** If it is exact on neither, R7 = 0 and the channel
converts nothing at all without codegen. **This is the single most decidable
prediction on the page and I do not know which way it goes.**

---

## 3. The join — scored FIRST, before any intersection is quoted

`w-emitp` grades **850**; `gap` grades **871** of 878. The 850 come from
`work/w-db/cacheidx.tsv` (850 rows), the 871 are `878 − 7 capture-fail`.

**J1 — registered:** the 850 is a **strict subset** of the 871, i.e.
`|850 \ 871| = 0`. Point **0**, interval [0, 0] — *this one is registered with a
zero-width interval on purpose*: if it is nonzero the intersection is unsound and
this lane reports the discrepancy instead of the number.

**J2 — registered:** `|871 \ 850| = 21`, and those 21 are exactly
w-emitpred's **21-TU held-out quarantine**. Point **21**, interval [7, 28].
Arithmetic: `871 − 850 = 21`, and 21 is the quarantine's size, which would be a
coincidence worth one line if it is not the explanation.

**What would surprise me:** J2 coming back with a set that is *not* the
quarantine — e.g. TUs that failed the Python IL reader rather than TUs held out
on purpose. w-emitp says "**850 of 857**", which implies 7 cache entries it could
not use, and `857 − 850 = 7` is the same 7 as `capture-fail 7`, which would be a
different story entirely. **The two arithmetics are in tension and I have not
resolved it in advance.** J2 is registered at 21 anyway.

**J3 — the cap.** `|B∧C ∩ (the 850)|`. Point **147** (151 less ~4 of the 21),
interval [130, 151]. Every R above is quoted against **this** denominator, not
against 151, and both are printed.

---

## 4. What this lane will NOT do

1. **It will not claim TU match moves.** It ships no emit predicate. TU match is
   **8** at both ends and the rung says so in its first line.
2. **It will not scale, extrapolate, or interpolate any intersection.** If a
   number cannot be measured on the join, it is reported as not measured.
3. **It will not touch `crates/c2-il` (w-vocab), `crates/c2-core` (w-rdata),
   `scripts/`, `fixtures/`, `tools/`, `crates/c2-obj` or `crates/c2-reference`.**
4. **It will not re-price the frontier.** `w-conv`'s hand-count is UNVERIFIED on
   the two newest members and this lane does not fix that.
5. **Every number is conditional on `C = 169`.** Lane `w-rdata` is live on
   `.rdata$r`, the ladder's `+421` step. **If C moves, `B∧C` moves, and every R
   above must be RE-MEASURED, not rescaled.** The rung states this in its own
   §1, not in a footnote.

## 5. Decline clauses

* **D1** — if J1 is nonzero, **report the discrepancy and do not publish the
  intersections as reach**; publish them as "over the intersecting join" with
  both denominators, or not at all.
* **D2** — if the reproduction of w-emitp's six incumbents (KA-A) does not come
  back to the digit, **stop**: the exact-sets are not w-emitp's and no
  intersection taken against them means anything.
* **D3** — nothing under `crates/c2-core`, `crates/c2-il`, `scripts/`,
  `fixtures/` changes. Proven by an empty `git diff` pathspec at the end.
* **D4** — the port's emitted bytes must not move. Proven by an **identical**
  `scripts/gate.sh` per-lane table, not by a green status.
* **D5** — every table prints the **count and its denominator**. A bare
  percentage of an unstated population is what produced `107`.

## 6. The known-answer control

The exact-sets are regenerated here from w-emitp's `scan.py` and `score.py`
**byte-identical** (copied, not edited), against `work/w-emit/truth` and a
`dtruth` rebuilt by w-joint's `truth_data.py`. **All eleven variant rows of
w-emitp §2.2 must reproduce to the digit** — `|P|`, precision, recall, F1 and
per-TU exact. That is D2, and it is the only thing that makes the per-TU *sets*
(which w-emitp published only as counts) trustworthy.
