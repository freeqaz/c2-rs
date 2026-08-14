# w-read2 — board rows, DRAFTED and awaiting the rebase

    Range allocated by the coordinator: #3129-#3134 (six).
    Next free after minting:            #3135.  No lane in flight.
    Append at the BOTTOM of docs/BOARD.md (w-ir-g §8.5's ordering hazard,
    invisible to board_audit.sh).

**NOT YET MINTED.** Master is at `ac3cdd8c`; `w-item-d` (#3119-#3124) and
`w-fenceb` (#3125-#3128) are complete on their branches (`wt-w-item-d`
`660a832c`, `wt-w-fenceb` `b06ceb1b`) but **neither is merged**. The rebase, the
re-gate and the mint all wait on that, per the coordinator's own ordering.

---

## ⚠ ONE INSTRUCTION IN THE DISPATCH IS WRONG, AND IT IS THE SAME ERROR CLASS TWICE

> *"`w-fenceb` CONVERTED, so `match` is 26, not 25. … Your required-zeros must
> be re-stated as `match` 26 → 26 on the new base."*

**The 878-TU workload `match` does NOT move to 26.** Verified three ways from
`w-fenceb`'s own committed evidence, not inferred:

1. `work/w-fenceb/gap.txt` — its own scan output — reads **`gap-metric match 25`**.
2. Its rung doc §260: *"878-TU workload `match` | 25 → **25** — the fixture is
   not a workload TU | **25.** Every scan digit is master's"*, and §280:
   *"digit-for-digit master's, as registered"*.
3. Its PREREG line 157, written **before** its own measurement:
   > *"878-TU workload `match` **25 → 25**. `whash_loop_then_framed.cpp` is a
   > *fixture*, not a workload TU. **The brief's "26th TU match" is the
   > fixture-gate number**; the workload number does not move and this lane will
   > not claim it does."*

`w-fenceb`'s `Outcome: converted` is a **tracked-fixture** conversion
(`whash_loop_then_framed.cpp`, `vocab-gap → match` at `/O1` and at the workload
flags), and its `Census:` line says `+0 — no new function class is admitted;
what moved is a TU-level GATE`.

**So `w-fenceb` was handed the same conflation in ITS brief, caught it, refused
to claim it, and registered 25 → 25 — and the number has now been propagated one
lane further.** It is the same shape as the `0x64` 8,000 / `0x9A` 2,674
correction this dispatch opens by acknowledging: **a number quoted from one
population as if it were another.** Two populations are both called `match`
here — the 381-fixture gate and the 878-TU workload — exactly the trap
`docs/STATUS.md` exists to carry.

**Consequence for this lane:** my required-zero stays **`match` 25 → 25** on the
878-TU workload. Restating it as 26 → 26 would make the identity diff *fail* and
would publish a number no scan produces. It will be **re-measured** on the
rebased base rather than assumed either way — `w-item-d` is the one that could
move the graded tree and the test count, and neither of those is `match`.

---

## #3129 — the headline: the ranking correction, and its MECHANISM

| field | text |
|---|---|
| **item** | **EVERY SINK INSTRUMENT IN THIS TREE IS CONSULTED FROM INSIDE `parse_expr`, SO A KEY RAISED OUTSIDE IT IS INVARIANT UNDER EVERY CEILING THOSE SINKS CAN MEASURE — WHICH IS WHY THE STATEMENT LAYER IS THE ONLY ROW OF THE SIX WHOSE CEILING NUMBER IS ITS BASE NUMBER** |
| **worth** | **MEASURED, on this lane's own base scan, BEFORE the target was picked and before the first `crates/` byte changed.** Board #3107's rule applied to the other five residue items of `w-readphase` §4.2: **`0x9A` = 0** · **`0x00` = 0** · compound-assign **45**, not 5,269 (**117×**) · `0x64` **422**, not 8,000 (**19.0×**) · `op:BD` **31**, not the 40,530 reach · and `5D`/`5E` **0** (#3107). **The statement layer reads 7,911 against a published 7,903 — the only agreement.** The mechanism, measured not argued: the five statement-layer keys (`body-cflow-label` 2,832 · `body-0x9B` 2,213 · `return-scope-close-cflow-label` 1,814 · `body-0x67` 1,044 · `body-0x5D` 8) read **+0 from base to the full 49-token + `type`/`convert`/`intrinsic` ceiling, while the other 610 of 615 keys moved**; also `+0` under `C2RS_SINK_BRANCH` at all three levels |
| **defined** | 5 keys · 7,911 · +0 · 25.0 % of the 31,650 ceiling residue · 6 of 6 items re-derived |
| **notes** | `chain_sink` (#660), `branch_sink` (#440), `rel_sink_enabled` (#420) and `off_add_sink_enabled` (#143) are **all** reached from `parse_expr_classed`. #3107 found that a residue ranking is counterfactual; this names **why** — the instrument's own call site — and makes it a rule rather than an observation. `docs/IL_DECODE_REACH.md` §12 now names all five sinks in one place, which is half of what **#3098** asks for; **#3098 stays OPEN**, because the board-topic-discoverability half is untouched and a `docs/` section does not fix it |

## #3130 — the ceiling, corrected a third time, and additive by measurement

| field | text |
|---|---|
| **item** | **THE DECODE CEILING IS 93,990 OF 120,456 (78.0 %), AND THE CORRECTION IS AN EXACT ADDITION BECAUSE THE EXPRESSION AND STATEMENT LAYERS ARE ORTHOGONAL** |
| **worth** | **MEASURED.** `w-readphase` §4 **76,041 (63.1 %)** → `w-deaccept` §4.5 **88,806 (73.7 %)** → **93,990 (78.0 %)**. The `+5,184` is `stmt-chain-fntail` **3,684** + `rsc-chain-fntail` **1,500**. Proven additive by a **diff-of-diffs**: `base → stmt-sink` and `chain-ceiling → both-sinks` are **identical, key for key and count for count**, and `expr-chain-noform-0x4F` holds at 88,806 across the second — the two layers share **no function** |
| **defined** | 88,806 → 93,990 · +5,184 · 2 layers · 0 shared functions |
| **notes** | Third correction to this number in three lanes and the first that is additive rather than a full re-measurement. **The lane's own load-bearing prereg miss lives here**: `C-a` registered *"< 1,000 of the 7,911 reach the tail"* at p = 0.75 and the answer is **5,184 — 5.2× off**. Being wrong is what made #3131's ladder worth running; had `C-a` held, this lane would have reported *"the statement layer is small, move on"* |

## #3131 — the leave-one-out, and the instrument every published ladder here uses

| field | text |
|---|---|
| **item** | **THE STATEMENT LAYER'S GAIN IS A STEP FUNCTION AND A FIRST-BLOCKER RANKING STRUCTURALLY CANNOT FIND ITS STEP: 19 GREEDY RUNGS BUY REACH ZERO, WHILE THREE TOKENS NO MASS RANKING WILL EVER NAME ARE EACH WORTH THE WHOLE 5,184** |
| **worth** | **MEASURED, 76 scans.** The greedy ladder grants `29 9B 53 B9 26 BD 33 2C 55 4C 32 30 67 20 44 27 43 4B 4F` by head mass and the reach is **0 at every one of 19 rungs**, residue pinned at 7,911. 19 tokens + `54` = **0**; + `41` = **0**; + `3A` = **0**; + `54,41` = **0**; **+ `54,41,3A` = 860**; the three terminators alone = **0**. Leave-one-out over the 49: removing **`op:54`**, **`op:3A`** or **`op:29`** takes the reach **5,184 → 0** — each worth the whole thing — and **the marginals sum to 98,039 against a total of 5,184, 18.9×**. `op:67`, `op:0D`, `type`, `convert` and `intrinsic` are worth **0** at the margin |
| **defined** | 19 rungs · reach 0 · 3 tokens × full reach · 18.9× · `op:29` = 0 vs 5,184 |
| **notes** | **A terminator is never a first blocker**, so the tokens carrying the gain are invisible to the ranking by construction. **`op:29` reads 0 as a rung-1 grant and 5,184 as a leave-one-out margin — one token, two of this repo's standard instruments, an unbounded spread.** Every published per-TU and class-wide ladder in `docs/` is a greedy head-mass climb, so **this invalidates the instrument, not one result**. `work/w-read2/loo.py` is **30 lines** and **re-running the published ladders through it IS A LANE** — the most valuable follow-up on this page. Also confirms `IL_DECODE_REACH.md` §3's *"decoding `67` and nothing else moves the decode reach by ZERO"* from a different instrument on a different population, while `body-0x67` is 1,044 first-blocker functions |

## #3132 — the unit judgement

| field | text |
|---|---|
| **item** | **ARM R DECLINED AND PRICED: NO INCREMENTAL STATEMENT-LAYER WIDENING CAN MOVE `fnbyte-refused-parse`. THE STATEMENT LAYER IS A PHASE, NOT A RUNG** |
| **worth** | **MEASURED, both sides.** Cost of refusing: **0 emitted functions** — reach is **0 at 20 granted tokens** (#3131), and the 5,184 width-walk reach is an *upper bound* on an accept that would additionally need the whole class model. Cost of shipping: `IL_STMT_GRAMMAR.md` §14.2 **step 5's fail-closed boundary is unpaid** — a decoded label is not a lowered CFG — plus `w-readphase` §7's live pre-emption hazard, `parse_expr` being called **by** the shape recognizers |
| **defined** | 0 functions · 20 tokens · 1 unpaid boundary |
| **notes** | Registered as contingent at p = 0.35 (`PREREG` R1) and **declined on the measurement**, not on taste. `CLAUDE.md`'s *Units of work* section exists to make this sentence sayable: a phase is dispatched as a construct rung or a characterization lane and **never as a TU lane**, and forcing one to produced 150 rungs of predicted saturation. This is the first lane to reach that verdict from a *measured* step function rather than from the shape of the work |

## #3133 — where two of the "unreachable" targets actually are

| field | text |
|---|---|
| **item** | **`0x9A` AND `0x00` ARE NOT ABSENT FROM THE WORKLOAD — THEY ARE MASKED BY THE STATEMENT LAYER, WHICH IS THE VERY CLAUSE THEY WERE RANKED BEHIND** |
| **worth** | **MEASURED.** Both read **0** in `emit_blockers` (615 keys / 113,612) **and** `fn_blockers` (635 keys / 1,705,627) at base, over 878 TUs. Behind `C2RS_SINK_STMT`: **`stmt-chain-0x9A` = 1,598** and **`stmt-chain-0x00` = 121** — both still short of `w-readphase` §4.2's ceiling readings of 2,674 and 2,276 |
| **defined** | 0 → 1,598 · 0 → 121 · both still under their ceiling |
| **notes** | The dispatch named both on their **ceiling** sizes. This row says *where they are*, not only that they are not where they were ranked — **#3095's masking phenomenon at a second site and a second layer**. `stmt-chain-0x9A` is the largest single thing behind the statement layer and `IL_DECODE_REACH.md` §3 already has its width and meaning, so it is a **phase-ordering** fact rather than a rung. Separately: **`is_statement_layer` (`expr.rs`) was left untouched**, and this lane confirms `w-deaccept`'s found-and-not-taken #1 zero from a second angle — `body-0x5D` is 8 emitted functions whose rung-1 successor is `0x26` on all 8 |

## #3134 (OPEN) — the transient, filed as a candidate fourth gate-void mechanism

| field | text |
|---|---|
| **item** | **TWO LANES INDEPENDENTLY SAW A GRADING INSTRUMENT REPORT NOTHING-GRADED ON A FIRST RUN AND CLEAN ON A RE-RUN, THE SAME DAY, WITH CONCURRENT GATE RUNS SHARING `/tmp` AS THE ONE COMMON FACTOR — AND NEITHER IDENTIFIED A CAUSE** |
| **worth** | **OPEN — OBSERVED TWICE, NOT REPAIRED, NO CAUSE IDENTIFIED.** (1) `w-read2`: `scripts/debug_lane.sh` first run **failed 2 of 18** — `O2` and `O1-Oi-GR` at **`graded=0` `total=381` `match=0` `mismatch=0` `panics=0` `rc=0`** — with **three live peer gate runs** named in the gate's own `/tmp` preflight; second run **18/18 at 381/381**. (2) `w-fenceb`, same day, independently: **two `/O1` lanes read one low on the first sweep, clean on re-run**, no cause identified. Two lanes, **two different symptom surfaces**, one common factor |
| **defined** | 2 lanes · 2 surfaces · 0 causes · 1 common factor |
| **notes** | **The failure DIRECTION is the point: a lane that did not re-run would have recorded a green-looking sweep over lanes that graded NOTHING.** Board **#1406** says `graded=0` must never read as a pass; **#299** and **#1077** are the same family *inside* the gate. Candidate **fourth gate-void mechanism**, after **#3048** (byproduct in graded dirs), **#3075** (edit under a live gate) and **#3117** (two writers, one artifact). **No fix attempted, deliberately** — it is a `scripts/` seam and it needs its own lane with a **reproduction**, which neither observation has. Until then the standing advice is the cheap one: **re-run `debug_lane.sh` and compare, and never quote a first run whose graded count is 0** |

---

## The free-range note (superseded, not edited — house style since #3073)

> ### `#3129`–`#3134` are `w-read2`'s, MINTED. **THE NEXT FREE NUMBER IS `#3135`, AND NO LANE REMAINS IN FLIGHT.**
> `#3134` is in **Open** (the gate-void candidate); `#3129`–`#3133` are in Done.
> `#3098` also remains **OPEN** and is *not* closed by `#3129` — see that row's notes.
