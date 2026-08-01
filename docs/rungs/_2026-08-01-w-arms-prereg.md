# Pre-registration — lane `w-arms`, boards #142 / #143 (2026-08-01)

Written and committed **before** the first scan of this lane was run, off
`master` at `1f3e00e` (§9.14's landing commit). Everything below is graded in
`docs/rungs/_draft-roadmap-9.17.md`.

Two facts known at registration time, from the record and from reading the
source — **not** findings of this lane:

* §9.13's three receiver-designator sites are values of the **`prod` axis**
  (`tail-recv-not-a-plain-b9-load`, `chain-recv-…`, `cmp-second-recv-…`), set by
  `mcall_{tail,chain,cmp}` when `eat_receiver_this` returns `Err(Block)`. **That
  `Block` — which carries the refusal context and the byte — is discarded.** So
  the site is currently one undifferentiated bucket by construction, and no
  existing instrument can decompose it. The decomposition therefore needs a
  refinement of the `prod` tag, not a new scan of an existing axis.
* `Blocker::OffAdd` is `27 <TYPE>` / `28 00 00`, and `eat_int_operands`'s
  vocabulary (`B9` / `33` / `02|03|04`) does not contain it — so the #143 row's
  second construct is one token, not a family.

---

## Item A — #142, decompose the clean-not-whole receiver arms

A **measurement**, not a rung. §9.13 sized the three sites at 37,060 blocked
emitted / 9,111 clean / 1,399 clean∧complete, of which #128 shipped 1,380.

* **A1 — the denominator has aged and must be re-measured.** Blocked EMITTED at
  the three `prod` receiver sites at this HEAD: point **35,700**, interval
  **[32,000 , 38,500]**. (§9.13's 37,060 less #128's realized 1,385.) *Refuted
  by* landing outside the interval.
* **A2 — clean** (`cflow-straight*` ∧ `eh-none` ∧ ¬`calls-2plus`) among A1:
  point **7,730**, interval **[6,300 , 9,200]**.
* **A3 — clean ∧ complete** among A1: point **60**, interval **[0 , 900]**.
  #128 took 1,380 of the 1,399 and 19 remained at the tail arm; §9.14's walker
  repair moves completeness in both directions and may add some back.
* **A4 — the decomposition.** The single largest *receiver construct* among the
  clean stock is the **intrinsic `0x33` family** (the 2113 `this`-adjust and the
  2117 base-member designator). Share of clean: point **40 %**, interval
  **[20 % , 70 %]**. *Refuted by* another construct being larger, or by the
  share landing outside.
* **A5 — concentration.** The top three constructs cover **≥ 80 %** of the clean
  stock. *Refuted by* < 60 %.
* **A6 — how many arms are actually there.** Distinct named receiver constructs
  with ≥ 500 clean emitted functions: point **4**, interval **[2 , 8]**.
* **A7 — totality of the new axis, with a printed residue.** Every body that
  reaches a receiver-designator refusal gets a *named* construct; the honest hex
  bucket (`…-op-0xNN`) is printed per byte and never summarised as "other".
  Predicted residue in an unnamed bucket: **0**. *Refuted by* any body whose
  receiver construct cannot be printed.
* **A8 — the axis is read-only over the census.** The refined `prod` tag changes
  no count and no verdict: bodies, emitted, match/mismatch and census/gate
  disagreement all reproduce the base scan **to the unit**. *Refuted by* any
  delta. (This is §9.13's control, run rather than argued.)

## Item C — did §9.14's repair make the blocker NAMES trustworthy?

§9.14 repaired the completeness walker's *acceptance vocabulary*. The brief asks
whether that made the census keys at these sites trustworthy. The control has to
be able to come out **both** ways.

* **C1 — agreement.** Over the clean-not-complete stock, the fraction whose
  census key names a construct that IS at the receiver-designator position:
  point **55 %**, interval **[25 % , 85 %]**. ~0 % would mean the keys are still
  pure second-reader stops (§9.13's `expr-intrinsic-this-adjust` finding);
  ~100 % would mean the repair fixed the attribution too. Both are reachable.
* **C2 — the residual second-reader population.** The fraction whose key names a
  construct that is **not** at the receiver position at all: predicted **≥ 30 %**.
  *Refuted by* < 5 %.
* **C3 — the repair's scope, stated as a prediction.** §9.14 changed
  `mcall.rs`'s walker; the receiver-designator refusal is in `mcall_tail.rs` and
  is **not** on the walker's path. So predicted: the repair moved **0** of these
  keys' attribution at the receiver position. *Refuted by* finding a key at these
  sites whose name changed between §9.13's tip `be797bf` and this HEAD for a
  receiver-position reason.

## Item B — #143, `…recv-load-then-off-add-more`

* **B1 — ageing check.** The row at this HEAD: emitted **1,038**, clean **851**,
  distinct names **267**, ±15 %. This HEAD *is* §9.14's tip, so a larger drift
  would indict the measurement rather than the row. *Refuted by* > ±15 %.
* **B2 — the counterfactual.** Admit the byte-offset add (`27 <TYPE>` and
  `28 00 00`) into the member-call productions' call-argument operand
  vocabulary, everything else unchanged. Δ `emit-in-class` over the 878-TU
  workload: point **60**, interval **[0 , 400]**.
  Registered reasoning: the key carries **`-more`**, i.e. the greedy walker
  already says a further construct remains after granting *both* the receiver
  form and the off-add. The 1,008 / 1,038 `tail-argument-not-in-the-operand-vocabulary`
  is the production's **first** refusal, not its last, and §9.13's E1/E2 forbid
  transferring a rate from a body-column anchor.
* **B3 — independent refusals.** After granting the off-add, the refusals that
  remain are **≥ 3 independent** classes ("what varies between these refusals?"),
  not one quantity at different thresholds. *Refuted by* ≤ 1.
* **B4 — the decision rule, registered in advance.** BUILD iff
  Δ `emit-in-class` **≥ 250** *and* the residue reduces to ≤ 1 independent
  refusal class *and* no `crates/c2-core` codegen change is required (this lane
  may not touch it). Otherwise **DECLINE, with the numbers.**
  **Predicted outcome: DECLINE.**
* **B5 — control for B2.** The instrumented binary with the sink **disabled**
  reproduces the base scan on every published number. An instrument whose
  inertness is asserted rather than run is this project's twelfth-instance
  failure. *Refuted by* any difference.
* **B6 — the control that can see an over-claim.** `census/gate disagreement`
  **cannot** see a measure that over-claims (§9.13 E4). If B4 says BUILD, the
  control is a fixture in the widened shape put in front of the **differential**,
  in `fixtures/cpp/` so it runs in all 12 `gate.sh` lanes, together with a
  negative fixture; and `scripts/sweep.d/` is extended over **arity** (the number
  of call arguments and the off-add's slot), not only over values. If B4 says
  DECLINE, no fixture is added and no claim is made.

## Gate items

* **G1** — `scripts/gate.sh` PASS, 12/12 lanes, **0 mismatch**, verdict count
  ≥ 2,520 (§9.14's figure; more only if this lane adds fixtures).
* **G2** — `cargo test --workspace` 0 failed at base and at tip, both *measured*;
  `#[test]` grep over `crates/` quoted at both ends and reconciled against the
  runner total.
* **G3** — 878-TU workload scan: **6 match, 0 mismatch**, census/gate
  disagreement **0**.
* **G4** — `c2rs selftest` all PASS; `expr_sweep` and `cross_sweep` 0 mismatches.

**A mismatch anywhere outranks every item above and is reported immediately.**
