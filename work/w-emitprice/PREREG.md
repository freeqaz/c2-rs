# PREREG — lane `w-emitprice`, wave 21 L2

    Lane:      w-emitprice
    Kind:      characterization
    Base:      1d52f8902  (master, wave-21 dispatch)
    Board:     #3856-#3862  (this lane writes NO row outside that block)
    Brief:     docs/WAVE21_BRIEF_2026-08-29.md §2 "L2"
    Committed: BEFORE the image is opened and before any measurement is run.

**Predicted reach 0. Predicted byte delta 0.** This lane writes no `crates/`
byte, proposes no `DISCLOSURE.md` row, adds no `gate.sh` row (`#3691`), and
touches neither `work/w-inlmetric/CLAUSES.tsv` nor
`docs/whitebox/ref/P_INLINE.md` — `w-budget` owns both this wave (`#3814`).

---

## §1 The question, stated so it can come out "no"

Five clauses of `P_INLINE.md` §6.1 — **C7, C9, C10, C11, C12** — carry
`blocker = emit-change` in `work/w-inlmetric/CLAUSES.tsv`. They are the largest
group in `absent` (5 of 12) and the only group nobody has priced. The
commission is to produce that price, **two-sided**, and two-sided means what
`#1042` and NC-5/`#2691` measured it to mean: **both of those flipped their
answer when the refusal's own cost was counted instead of only the change's.**

So for each row, three numbers and a verdict:

1. **BUY** — what does adopting it get, in the units the goal is written in?
2. **REFUSAL COST** — what does the standing refusal cost *today*: how many
   bodies/TUs does it refuse, and what would those be worth in the same units?
3. **Which is larger**, and what is the uncertainty on the comparison.

## §2 The units, fixed here and not chosen later

`docs/GOAL_DECISION_2026-08-21.md` (amended): goal (1) understanding of MSVC's
internals for decomp is **primary**; goal (2) parity — `match` → 870/878 — is a
real end and instrumental to (1). `docs/PROGRESS_METRIC.md` §5.2 governs the
scoring and is not relaxed: **a wrong emit scores strictly below the refusal it
replaced.**

The four admissible price units, in this order of authority:

| unit | what it is | where it is read |
|---|---|---|
| **U1 `Δmatch`** | whole objs byte-exact, of 878 | `c2rs gap`, `docs/STATUS.md` generated block |
| **U2 `Δfnbyte-exact`** | emitted functions byte-exact, of 162,205 | `docs/FUNCTION_BYTE_MATCH.md`; the only continuous number graded by the oracle's own bytes |
| **U3 warranty** | live wrong emits closed or opened — a sign, not a magnitude, and it **dominates** U1/U2 by §5.2 | `mismatch`, `scripts/gate.sh`, `scripts/expr_sweep.sh` |
| **U4 characterization** | for goal (1): does adopting the clause turn a baked constant or a blind spot into a **named, settable decision point**? | `crates/c2-core/src/surface/DOMAIN.txt`; `GOAL_DECISION` § AMENDED |

**Explicitly NOT admissible as a price unit, and each is banned for a measured
reason:**

* the **census** (per-function or emitted) — a census gain is not a goal gain,
  and the census is fail-open on most of the workload (`[[only-fnbyte-maps-to-the-goal]]`);
* the **`c2rs subsys` agreement number** — `#3845`: a literal `[R]`/`[O]`/`[I]`
  string count over a markdown page, writable by prose, and it moved a lane
  **down** last wave for doing good work;
* **clause-table row movements** (`absent` → `R-derived`) — that is this lane's
  own bookkeeping, and pricing a clause by how much it moves the instrument
  that counts clauses is the closed loop `#3505` is about;
* **throughput** — a property, not the goal (`GOAL_DECISION`).

## §3 The five predictions, registered before measurement

Each is falsifiable and each names what would falsify it. **A row whose honest
price is "unknown until a corpus exists that exercises it" is a real finding
and will be published in those words rather than as a manufactured number.**

| # | prediction | falsified by |
|---|---|---|
| **P1** | **C7's price is NEGATIVE** — adopting `DAT_10c46318`'s value (128) into the port's ceiling costs wrong emits in both directions and buys 0, because the port's constant is in a different unit. | Re-deriving `P_INLINE` §6.7.1's counterexample table off the committed cells and finding **0** counterexamples in either direction at `/O1`. |
| **P2** | **C9's price is UNKNOWN and the blocker is a READ, not a corpus** — the run-time value of `DAT_10c2e310` at `/O1` is not established anywhere in this repo, and the clause's own text ("non-zero ⇒ the size test is SKIPPED") cannot be reconciled with the measured finite `/O1` and `/O2` brackets. | Locating a committed read or measurement that settles `DAT_10c2e310`'s `/O1` run-time value, **or** showing the clause text and the brackets are consistent as written. |
| **P3** | **C10 is MISCLASSIFIED.** The port already carries `forceinline` as a swept parameter; wiring a reader to it is **byte-neutral by construction** at `INLINE_MAXLEVEL_UNBOUNDED`, exactly as C15's adoption was, so `emit-change` is the wrong blocker and the right one names the **missing ATTR-width reader**. | Finding any production path whose emitted bytes depend on the `forceinline` argument — i.e. a caller of `port_enter_site` / `declines_at_maxlevel` / `charge` that is reachable from `PortC2::build` and whose verdict moves when the argument flips. |
| **P4** | **C11 is NOT DERIVABLE and its `R1` cell overstates it.** `[sym+0x20]` is a back-end symbol-arena word written by c2's own passes, not a `.gl` field arriving verbatim the way `[sym+0x4c]` does (C24), so no `crates/` reader can compute it and `emit-change` is not the binding blocker. | Locating a read that gives `[sym+0x20]`'s four tested bits an IL-side provenance. |
| **P5** | **C12 shares C10's missing reader and is otherwise byte-neutral-or-better** — its two bits are REFUSE bits on the field the port already decodes, so adopting them can only narrow the port's accept set: the price is a **warranty** question (U3), not a conversion question (U1/U2). | Showing the port has an accept path whose currently-**matching** output would be withdrawn — which would make the price a real U1/U2 loss rather than a warranty gain. |

## §4 Method — read before probe, and what is deliberately NOT recompiled

`docs/WHITEBOX_LEVERAGE_2026-08-21.md` is standing doctrine. Before any grid is
budgeted the read that would answer the same question is priced and preferred.

**M1 — the port's ATTR reader width `[R]` source.** Read
`crates/c2-il/src/func/gl.rs` and the standing `plan-glattr` bit histogram in
`docs/STATUS.md` to fix exactly which bits of `[sym+0x4c]` any instrument in
this repo can see. No recompile.

**M2 — the port's accept path `[R]` source.** Enumerate every refusal the port
raises in the inline region and map each to the clause it stands in for. This is
what makes "the refusal's own cost" a *measurable* quantity rather than a
rhetorical one: a refusal with no distinct population has cost 0 by
construction, and saying so is the answer, not an evasion.

**M3 — the exercising population, per clause.** From `P_INLINE` §6.1's
`exercised` column, §6.4's 8,936-callee hold-out measurement, and the standing
`docs/STATUS.md` block. Where the population is **0 or unmeasured**, that is
published as the price. No new grid is compiled to manufacture one.

**M4 — C7's counterexamples, RE-READ.** `work/w-lowerband/ceiling_check.out`
and `work/w-sizebracket/series.jsonl` are already on disk. Re-read, never
recompiled — the same read-before-probe move `w-lowerband` made.

**M5 — the image, opened only for what M1–M4 cannot answer**, against the
pinned `c2.dll` sha256 `c80981c0…a66258`, with every address carrying its
evidence tier `[R]`/`[O]`/`[I]`.

**Controls watched RED before any verdict from them is quoted (`#3336`).** Any
script this lane writes is shown failing on a planted input before its green is
published.

## §5 THE ARTIFACT CHECK — `#3505` is six for six and this lane produces a ranking of five

`#3505`: **every lane dispatched off a constructed ranking or denominator found
the ranking was an artifact.** This lane ranks five rows. Registered here,
before the numbers exist, is what would make the ranking one — and each is
checked and reported in the findings, including when it fires.

* **A1 — the tie artifact.** If four or five rows price at the same value
  (e.g. all at "buy 0"), there is **no ranking**, only a partition, and
  publishing an order over a tie is manufacturing signal. *Check: report the
  distinct price values and their multiplicity; if the modal class holds ≥ 4 of
  5 rows, the deliverable is declared a PARTITION and not a ranking.*
* **A2 — the instrument-shaped denominator.** If the ranking is driven by a
  count only this lane's own new instrument produces, it is a statement about
  the instrument. *Check: every count carries a recipe and at least one count
  per row must come from a standing instrument (`c2rs gap` / `STATUS.md` /
  another lane's committed output), not from a script written here.*
* **A3 — the unexercised-population artifact.** Three of the five rows
  (`C9`, `C11`, `C12`) carry `exercised = no` in `CLAUSES.tsv`. A ranking whose
  order is really "which rows the workload happens to exercise" is a ranking of
  the workload, not of the clauses. *Check: report the ranking with and without
  the `exercised` term; if the two orders are identical, `exercised` is not
  carrying the ranking; if they differ, say so and name which rows moved.*
* **A4 — the prose-writable metric.** No row is ranked by any quantity a lane
  can move by writing a sentence (`#3845`).
* **A5 — the self-confirming price.** If every prediction in §3 comes out
  confirmed, that is evidence the predictions were written after the answer was
  known. *Check: this lane reports the score of §3 against the outcome
  explicitly, misses included, and treats a 5-of-5 as a flag to re-examine
  rather than as a result.*

## §6 What this lane may NOT do

* No `crates/` byte. No `DISCLOSURE.md` row. No `gate.sh` row.
* No edit to `work/w-inlmetric/CLAUSES.tsv` or `docs/whitebox/ref/P_INLINE.md`
  — `w-budget`'s this wave. Corrections to either are **recorded on this lane's
  own findings page with their evidence**, exactly as `w-instrcount` did, and
  land in a later wave.
* No adoption of 128 as the inline ceiling (`#3732`, brief §3).
* No board row outside `#3856`-`#3862`.
* No recommendation to *take* any of the five. This lane produces a **price**;
  the decision to spend it is the coordinator's.

## §7 Outcome

`built` if the five prices land with their derivations printed and the artifact
check is run and reported. **`FAILED`, in those words, if fewer than five rows
get a price** — including the case where the honest price is "unknown", which
counts as a price only when it names *what* would settle it and *what that
read costs*.
