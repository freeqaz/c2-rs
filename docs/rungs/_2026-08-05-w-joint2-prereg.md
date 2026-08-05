# w-joint2 — PRE-REGISTRATION. Written before the first number.

    Tag:       w-joint2-prereg
    Slug:      w-joint2-prereg
    Date:      2026-08-05
    Fixtures:  none — a pre-registration admits no class, moves no accept/refuse
               boundary and emits no byte. It is a record of predictions.
    Census:    not measured yet — this file precedes every measurement.
    Record:    this file. Findings in `_2026-08-05-w-joint2.md`.
    Lane:      w-joint2, worktree `wt-w-joint2` off master `bed9894`.

---

## 0. Why this file exists and what it is committed before

The lane brief instructs me to build **two rungs jointly** — the `.rdata$r`
writer (board **#300**, declined by `w-rdata` at seven facts) and the tag-0x10
ALIAS channel (board **#302**, priced at +0 by `w-reach`) — on the stated
ground that:

> `.rdata$r` alone = +1 TU. The alias channel alone = +0 TU. TOGETHER = +91
> (model) / +158 (ceiling).

and that this lane "is the one that can actually move TU match, which has been
stuck at 8 of 878."

**This file is committed before I run `c2rs gap` even once.** Everything below
is written from reading `docs/OBJ_RDATA_R_SHAPE.md`,
`docs/rungs/_2026-08-05-w-reach.md`, `docs/rungs/_2026-08-05-w-alias-findings.md`
and `crates/c2-harness/src/gap/factors.rs` — no measurement of my own.

## 1. DECLARED BIAS, first, because it points the other way from my predictions

I have been handed a brief that asserts a **+91** headline and told I am the
lane that can move the payoff metric. The incentive is to find the brief right.
Two specific biases I am registering so they cannot be claimed as insight later:

* **Confirmation.** If the joint number comes back large I will be tempted to
  report it as vindication of the brief. It would not be — see §2, where I
  predict the brief is quoting the wrong quantity, and if I am *wrong* about
  that I must say the prediction failed rather than that the lane succeeded.
* **Sunk build.** The seven facts are a lot of work. Once any of them is written
  I will be biased toward shipping the rest. §4's decline clauses are written
  now, before a line of code, precisely so that a decline stays cheap.

I also register the reverse risk: I am about to predict that a brief handed to
me by the coordinator contains an error. That is a comfortable, contrarian place
to be, and it is wrong as often as it is right. The predictions below are
therefore **point values with intervals**, not a posture.

## 2. THE CENTRAL PREDICTION — the brief's `+91` is REACH, not TU match

`crates/c2-harness/src/gap/factors.rs` states the model:

> A byte-exact obj requires **A ∧ B ∧ C ∧ (D ∨ E)**.

`w-reach`'s `+91` is `|{JFP_ALIAS exact} ∩ B∧C|` moving 122 → 213 at `C = 590`.
That quantity is a **`B∧C` intersection**. It carries **no A term and no `D∨E`
term**. `w-reach` §6.1 says so in its own words — *"TU match does not move and
this rung does not claim it will"* — and §4.1 measures the zero-codegen
conversion at **1 TU**, the same one for all four models.

So I predict the joint that actually bounds TU match behaves as follows.

| # | registered quantity | point | interval |
|---:|---|---:|---|
| **R1** | baseline `factor-c` / `b-and-c` / `A∧B∧C` / `A∧B∧C∧(D∨E)` / `match` | 169 / 151 / 27 / 8 / 8 | exact — a known-answer control on master's published block |
| **R2** | baseline `D` / `E` / **`\|D∨E\|` over the 871 graded** | 8 / 2 / **10** | `\|D∨E\|` in [10, 10]; D and E are measured disjoint (`w-reach`: D fails on the two `??__E` TUs, E fails on all six per-function matches) |
| **R3** | **`A∧B∧C∧(D∨E)` with `.rdata$r` added to `PORT_WRITER_SECTIONS` and nothing else** | **8** | **[8, 10]** |
| **R4** | `factor-c` / `b-and-c` at that same counterfactual | 590 / 315 | exact — `w-rdata`'s writer-edit figures and `w-reach`'s independent key reconstruction agree on both, so this is a KA control on my instrument, not a result |
| **R5** | **`A∧B∧(D∨E)`** — the joint with **C dropped entirely**, i.e. the exact TU-match ceiling of *all* remaining section-vocabulary work, `.rdata$r` + `.text$yd` + `.xdata$x` and every name after them | **9** | **[8, 11]** |
| **R6** | `A∧B∧C∧(D∨E)` at `C = 871` (all three ladder names added) | 8 | [8, 11] — must equal R5 if the ladder closes C |
| **R7** | TUs in `D∨E` that are **not** `match`, by name | 2 | [1, 4]; I expect them to be `src/system/decomp_pch.cpp` and `src/system/math/vec.cpp` — board #213's divergence pair, and `w-reach` §4.1's whole zero-codegen population |

**R5 is the number this lane exists to produce and nobody has computed it.**
`w-reach` measured `B∧C` at four values of C and published the reach ladder. It
did not intersect with `D∨E`, and `A` never enters any figure on that page.
`A∧B∧(D∨E)` is monotone-dominating every `A∧B∧C∧(D∨E)` at every possible C,
because A, B, D and E are **independent of the writer's section vocabulary** —
A is `.ex`-vs-`.text`-COMDAT agreement, B is symbol binding, D is the
per-function census verdict, E is a whole-TU recognizer. None of the four reads
`PORT_WRITER_SECTIONS`.

> ### If R5 comes back at 8 or 9, then **the entire section ladder — all three names, `.rdata$r` included, and the alias channel on top of it — is worth at most +1 TU match**, and the brief's `+91` is a reach figure that cannot convert without codegen the project does not have. That is a refutation of the lane's premise, and per `CLAUDE.md` it is a result rather than a problem.

### 2.1 The arithmetic that forces R3, stated so it can be checked rather than believed

`A∧B∧C∧(D∨E) ≤ |D∨E|`. If R2 holds at 10, then **no** amount of section work,
emit-set modelling or binding repair can take TU match above **10** on this
workload. Match is 8. The headroom is **2 TUs**, at every C, forever, until the
port's *codegen class* widens.

I could be wrong in exactly one way and I name it now: **fact 3 of the seven**
(codegen for `lis/addi/stw rD,0(r3)/blr`, the vfptr-store leaf) *is* a codegen
widening, so it would move **D**. That is the only route by which this lane's
own work could raise the bound. **R8: I predict the vfptr-store leaf alone moves
`D` by 0**, interval [0, 3] — because a `.rdata$r` TU needs *every* emitted
COMDAT in class, and the 676 TUs carrying `.rdata$r` are real workload TUs with
hundreds of functions each, not minimal grid objs.

## 3. Predictions about the seven facts

| # | registered | point |
|---:|---|---|
| **F1** | the decomposition in `OBJ_RDATA_R_SHAPE.md` §8 is **7 facts, 5 in `c2-core` + 2 in `c2-il`**, as the brief states | I expect the brief to be **right** on the count and the split |
| **F2** | fact 1's claim that `expr-op-0x27` is **refused at parse** | I expect this to be **partly refuted**: `grep` already shows `0x27` accepted in `codec.rs`, `designator.rs`, `control_flow.rs`, `expr.rs` and `ctor_dtor.rs`. The refusal is a *census row label*, not a blanket parse refusal, and the distinction matters because it changes fact 1's price |
| **F3** | whether the **emit-set model** in `PortC2` is needed | **YES, and it is its own lane.** `w-alias` §1 registered that w-emitp §6.3's application site does not exist. Nothing in this lane's seam creates it cheaply |

## 4. DECLINE CLAUSES — registered now, binding on me later

* **D1 — the bounding clause.** If **R5 ≤ 10** (i.e. the TU-match ceiling of all
  section work is at most +2 over today's 8), I **decline to build the seven
  facts as a TU-match rung** and ship the measurement instead. Seven independent
  facts for ≤ 2 TUs fires the standing decline clause (board #269, ≥ 4 refusals)
  by a factor of three. I will say so plainly and will **not** soften it into
  "worth doing for reach".
* **D2 — the no-caller clause.** I will **not** add `".rdata$r"` to
  `PORT_WRITER_SECTIONS` except transiently inside a counterfactual that reverts
  and asserts `git status --porcelain crates/` is clean. `w-rdata` §9 and board
  #278 are explicit that a vocabulary entry without a caller is `+421` of
  reachability the port does not have, and `gap` cannot catch it.
* **D3 — rule 4 stays a count.** I will not convert `w-emitp` §6.4 ("never emit
  a name in `dom(alias)`") into a hard-coded rule. If my work would make it safe
  I must prove it against `an_alias_that_also_has_a_body_is_counted` explicitly.
* **D4 — the one-shot gate.** I will **not** spend `w-emitpred`'s Part-1 gate.
  If I come to want it I will ask in the report and stop.
* **D5 — absence is not success.** Every claim in the findings gets a **positive
  check with a printed count**. If I find myself writing "nothing broke", that is
  instance 17 and I rewrite the sentence.
* **D6 — half-built is worse than declined.** If the emit-set model turns out to
  be load-bearing design (F3), I stop and say it is its own lane rather than
  shipping a partial one.

## 5. What would surprise me

* **R5 coming back above 20.** That would mean `D∨E` is much larger than D + E
  as published, or that A and B are far less binding on the `D∨E` set than on
  the corpus — either would be a genuine finding and would flip D1.
* **R4 missing.** `w-rdata` (writer edit) and `w-reach` (key reconstruction)
  agree on 590/315 by two methods. If my run disagrees, my instrument is wrong
  and nothing else on the page may be quoted. This is the control that licenses
  R3 and R5.
* **R7 coming back as names other than the divergence pair.** `w-reach` §4.1
  names `decomp_pch.cpp` and `vec.cpp` and calls them the whole zero-codegen
  population. If `D∨E \ match` holds something else, that page is incomplete.

## 6. The one thing I am NOT registering, deliberately

I am not registering a prediction about **whether the pair should eventually be
built**. That is a schedule decision for the coordinator and it depends on
whether the project values reach. My predictions are about *what the numbers
are*, and I will hand the schedule question back with the numbers attached
rather than answering it inside a measurement.
