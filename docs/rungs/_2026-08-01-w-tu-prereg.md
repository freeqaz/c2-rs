# Pre-registration — lane `w-tu`, board #122 forensics + the one-away band (2026-08-01)

Written and committed **before** any per-TU byte-level measurement of this lane
and before any code was touched. Graded in `docs/rungs/_draft-roadmap-9.16.md`.

## What was already known at registration time, and is NOT a finding of this lane

Stating these up front so the score cannot borrow credit from them:

* **`docs/GAPS.md` §9 (W-ONEAWAY, 2026-08-01) already measured nine one-away
  TUs** and already refuted the premise: 7 of 9 are not single basic blocks,
  1 is `eh-state1`, 1 (`xboxheap.cpp`) is three independent refusals. That lane
  pre-registered "0 of 9 converted" and delivered 0. My prior is therefore
  *informed and pessimistic*, not brave, and every estimate below is declared
  with that bias.
* **Master's merge `6b07500` states "TU match 6 -> 6" in its subject line.**
  So the claim under investigation is already contradicted by the tree before I
  start; what is open is *which* of the brief's four explanations is right.
* **The base scan of this lane was run before this file was written** (it is the
  baseline, not a result). Anything already visible in that scan output is
  marked POST-HOC below and scores nothing.

## POST-HOC — read off the base scan, claims no predictive credit

* Base scan at `1f3e00e`: TU match **6**, mismatch 0, codegen-gap 0, vocab-gap
  865, capture-fail 7. Census 706,402/2,462,571 (28.69 %). Emitted census
  36,059/178,968 (20.15 %). Distance histogram **≤0: 1, ≤1: 10, ≤10: 25,
  ≤100: 32, ≤1000: 210**.
* The `≤1: 10` bucket is **cumulative** and its first member is
  `src/system/utl/Spew.cpp` at distance **0**, which already matches. So the
  bucket holds **nine** one-away TUs and one already-converted one. This was
  read directly off the printed table and is not a prediction.

---

## The estimates

Unit is stated per item and matches the unit of the thing being claimed.

| # | claim | point | interval |
|---|---|---:|---|
| **E1** | Board #122's "6 → up to 15" is **the projection branch** of the brief's four options — an item-ceiling restated as an outcome — and not "converted then regressed", "wrong measure", or "wrong distance metric" | projection | one of the four |
| **E2** | The string `15/878` (or any recorded TU match ≠ 6) appears in **zero** commits anywhere in this repository's history, all branches | **0** | [0, 2] |
| **E3** | Of the 9 TUs at distance exactly 1, the number that would convert if **only** their named first blocker were removed | **0** | [0, 1] |
| **E4** | Largest number of the 25 TUs at distance ≤10 that any **single** change converts — i.e. does a "one-away lever" exist at all | **1** (no lever) | [1, 3] |
| **E5** | Of the 7 TUs at distance 4–10 never crossed with the cflow/EH axes (`IPP_basicmath_xbox`, `EncryptXTEA`, `xboxmem`, `JsonMemory`, `Rand2`, `VorbisMem`, `MeterEffect`), the number whose **every** blocked body is `cflow-straight` **and** `maxState == 0` — i.e. reachable by widening alone | **2** | [0, 5] |
| **E6** | Number of the 9 one-away TUs whose §9.2 key name, taken to the byte, names a construct that is **not** in fact the thing that must fall for that TU to match (the §9.13/§9.14 "the key name lied" failure) | **2** | [0, 5] |
| **E7** | TUs converted to byte-exact by this lane | **0** | [0, 1] |
| **E8** | `xboxheap.cpp` — the only one of the nine that is neither control flow nor EH — remaining independent refusals after WLR took the first | **2** | [2, 3] |

**Bias, declared: pessimistic on E3/E4/E7, and it is borrowed pessimism.** I read
§9 before registering. The honest statement is that E3/E7 are near-certain and
score little; **E5, E6 and E8 are the ones that can go wrong**, because nobody
has crossed the 4–10 band with the two axes and nobody has checked the nine key
names against the bytes.

## What would make each of these go red

The discipline this week is that a negative claim needs a control that *could*
have failed:

* **E2** goes red if any `git log -S` / `-G` over all branches finds a recorded
  match count other than 6. The search is run over the whole DAG, not master,
  and over both the doc text and the commit messages — so a lane that moved the
  number on a branch and never merged would still show.
* **E3/E4** go red the moment a single measured change converts a TU, or the
  moment two TUs are found to share a blocker whose removal is one rung.
* **E5** goes red in the honest direction: if 4+ of the 7 are straight-line and
  state-0, the 4–10 band is a real widening target the board has never named,
  and this lane should build into it rather than report on it.
* **E6** goes red if all nine key names survive the bytes — which would be the
  first week in three that a key name did not mislead, and worth recording as
  such.
* **E8** goes red if `xboxheap` is now 1 refusal away (WLR plus something else
  having landed), which would make it a live conversion candidate and change
  what this lane does with the rest of its time.

## The control that has to exist before any "the TU cannot convert" claim

`c2-il::func::census`'s `every_in_class_row_is_a_single_basic_block` is the
invariant the control-flow refutation rests on. It is an assertion over the
workload, not an argument, and it is re-run in this lane's scan. **If it ever
stops holding, every "needs Phase 6" verdict in §9.4 and in this lane is void.**
That is the control, it is already wired, and it can go red.

The distance metric itself gets a control it has never had: **is `blocked == 0`
actually sufficient for a TU to match?** The metric is
`fn_total - fn_in_class <= k` (`gap.rs::near_match_tus`), which is a claim about
the *census*, not about the obj. Today exactly one non-empty TU sits at distance
0 and it matches, so the rule has been tested at n = 1. This lane states that
sample size out loud rather than treating the metric as validated.
