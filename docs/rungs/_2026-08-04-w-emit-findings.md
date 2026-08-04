# w-emit — §2's Propagation clause SURVIVES the workload. The unbuilt ROOT clause owes 1 in 5.

    Lane:      w-emit, 2026-08-04, worktree `wt-w-emit` off master `c7f7529`
    Prereg:    rungs/_2026-08-04-w-emit-prereg.md, committed at `25283d1`
               BEFORE any measurement of the headline quantity. Scored in §6.
    Ships:     NOTHING under `crates/`. No fixture, no codegen, no widening.
               **This is a measurement**, and declining to implement was
               PRE-REGISTERED (prereg §5), not a shortfall.
    Status:    FINDINGS. TU match is 8 at both ends.

**One-line answer:** *The measurement designed to kill `PHASE7_PLAN.md` §2's
Propagation clause on real TUs **failed to kill it** — the contradiction set is
**470 instances** before an artifact filter and **2 instances on 1 pair** after
one, against a pre-registered point of **250 000** — and the one survivor is a
`$4` adjustor thunk, i.e. **#152's synthesis gap, not #161's propagation**. The
predicate's risk is now entirely concentrated in the half nobody has built:
**the ROOT clause must supply 20.4 % of every emitted name (35 608 of 174 417),
and it is the exact clause `PHASE7_VALIDATION.md` §6b proved internally
inconsistent on its face.***

---

## 0. Provenance — every number on this page

| | |
|---|---|
| c2-rs branch | `wt-w-emit`, based on master **`c7f7529`** |
| c2-rs HEAD at measurement | **`25283d1`** (the prereg), `clean` — **no `crates/` change exists in this lane** |
| harness binary | `7eec3ce4cda03a3f` |
| **dc3-decomp HEAD BEFORE the run** | **`940d07dcb0960964ad61aa5f025658f993eb46b2`**, `workload_dirty = False` |
| **dc3-decomp HEAD AFTER the run** | **`940d07dcb096…`** — **it did not move** |
| flags | the unmodified workload line, `work/dc3-workload/flags.txt` |
| TU list | `work/dc3-workload/files.txt`, 878 entries |
| **wibo** | **`1.0.1-23-g4a9dd6f`** — see the warning below |
| capture cache | shared with the main repo; the scan reports **871 hit, 7 miss, 0 POISONED** |
| scratch | `work/w-emit/` (gitignored); text records force-added under the same path |

### 0a. The wibo under this lane is NOT the wibo under w-afail — and it did not move the numbers

Every other lane today ran under `wibo 1.0.1-7-g3b0f71c-dirty` and recorded
`wibo_stale = True` against the known-good `1.0.1-23`. **This lane ran under
`1.0.1-23-g4a9dd6f`.** That is a change of the *host* that executes `c1xx.dll`
and `c2.dll`, so it is a provenance fact, not a footnote, and a naive
cross-lane byte comparison is not automatically valid.

**Measured rather than assumed:** KA2 (§5) re-ran w-emitpred's unmodified
`detect.py` and reproduced `MAGNITUDE.md`'s workload class to **within 0.4 %**
across *both* a wibo change and a dc3 rev change (`9ad5c4c8 → 940d07dc`), and
KA5 reproduced every incumbent gate number exactly. **The host change is
recorded and is not detectable in these numbers.**

### 0b. Denominators, stated once

* **876 / 878** TUs yield front-end IL (`cl /Bd /d2nop`; c2 aborts `C1007 … in
  'p2'`, so no c2 output exists — quarantine-safe). The 2 misses are
  `FxSendPitchShift360.cpp` and `FxSendSynapse360.cpp` (`C2084`).
* **850** TUs have truth. 878 − **21 quarantined** − 7 producing no obj.
* **The 21-TU quarantine of `_2026-08-02-w-emitpred-prereg.md` is still in
  force and was honoured.** `truth_all.py` was pointed only at
  `truthlist.txt`. **This lane makes no prediction on the held-out
  population, so it does not spend it** — w-emitpred's one-shot Part-1 gate
  is still owed and still runnable.
* Population reproduces `MAGNITUDE.md`'s exactly: same 876 / 850, same failing
  TUs by name.

---

## 1. What was measured, and why it needs no root model

A root model does not exist. This lane did not invent one — inventing it is
fitting, which `MAGNITUDE.md` §2 already declined for the same reason.

    E(t)   truth      COMDAT leaders of every IMAGE_SCN_CNT_CODE section
                      (never a `.text` name prefix)
    U(t)   universe   names with a `.gl`-named `.ex` body

    26-edge   exb[p-1] == 0x26   direct call / reference   (TIGHT)
    67-edge   exb[p-2] == 0x67   virtual dispatch          (EXCLUDED)

    X = { (t,F) : F in U(t), F not in E(t),
                  some A in E(t) has a 26-edge A -> F }

**The dilemma.** For any `(t,F) ∈ X`: either §2 predicts its own referrer `A`,
in which case its Propagation clause adds `F` and **`F` is a false positive**;
or `A ∈ E ∖ P` and **`A` is a false negative**. **Every element of `X` is an
error of §2, whatever the roots are.** `67`-edges are excluded, so `X` bounds
the predicate **after** `PHASE7_VALIDATION.md` §8a's repair #1 — the repaired
predicate, not the refuted one.

| | |
|---|---:|
| `\|E\|` | **174 417** |
| `\|U\|` | **1 508 530** |
| emit-everything precision (`\|E\|/\|U\|`, the port's behaviour today) | **0.11562** |
| `.ex` segments / `.gl`-named | 2 438 781 / 1 516 228 (62.2 %) |

---

## 2. The result: `X` is empty once its artifacts are removed

### 2.1 Raw

| variant | `\|X\|` instances | distinct names | TUs touched |
|---|---:|---:|---:|
| **strict** (`.gl`-named owners only) | **470** | 245 | 258 of 850 |
| strict + local-static owner recovery | 1 215 | 491 | 320 of 850 |
| loose (any non-`67` resolvable token) | 24 728 | — | — |

Registered point: **250 000**, interval **[50 000, 800 000]**. **Missed low by
roughly three orders of magnitude.**

### 2.2 The artifact filter, and it is not an opinion

`.gl` operand tokens are **per-TU values**, so a token collision is a per-TU
accident while a real call is a property of the source. For a pair `(A,F)`:

    opportunity = # TUs where A is emitted and F is in U
    recurrence  = # of those where the 26-edge A -> F is observed / opportunity

Measured over the 850 TUs, with the **agreeing** pairs (`F` emitted) as the
positive control:

| | pairs (opportunity ≥ 3) | median recurrence | ≥ 0.9 | ≤ 0.1 | 2-byte-token share |
|---|---:|---:|---:|---:|---:|
| **AGREE (control)** | 8 445 | **1.000** | **99.3 %** | 0.2 % | 33.8 % |
| **X (contradictions)** | 182 | **0.031** | **0.5 %** | **69.8 %** | **95.6 %** |

**A 200× separation.** The agreeing edges recur in essentially every TU where
the opportunity exists — they are source-determined, i.e. **real calls, and the
extractor works**. The contradiction edges do not recur at all, and **95.6 % of
them resolve through a 2-byte token** (only ~2¹⁵ values, the collision-prone
encoding) against 33.8 % of the controls.

**Of the 436 distinct contradiction pairs, exactly ONE survives
`recurrence ≥ 0.9`:**

    ?Handle@RndAnimatable@@$4PPPPPPPM@A@AA?AVDataNode@@…   ->   ?Handle@RndAnimatable@@UAA?AVDataNode@@…

That is a **`$4` virtual-adjustor thunk** referencing the virtual it forwards
to — **`#152`'s synthesis class, which `PHASE7_VALIDATION.md` §8a already files
as "§2 has no clause capable of producing this symbol"**. It is a GAP, not a
Propagation error.

| | |
|---|---:|
| X instances (strict, `(TU,A,F)`) | 500 |
| …on pairs surviving the recurrence filter | **2 (0.40 %)** |
| ⇒ surviving contradiction pairs | **1**, and it is a thunk |

> **§2's Propagation clause has no measurable false positives on the
> direct-call edge kind at workload scale.** The clause fitted on 172 synthetic
> cells **transferred**.

**Stated as the bound it is, not more.** 254 of the 436 pairs have opportunity
< 2 and are **untestable** by this filter; that 181 of 182 testable ones are
artifacts makes the same likely of them, and **that is an inference, labelled,
not a measurement.**

### 2.3 The loose extractor is indistinguishable from noise — measured, not asserted

| | targets reached in `U` | of which emitted |
|---|---:|---:|
| tight (`26`) | 139 277 | 138 807 (**99.663 %**) |
| loose (any non-`67`) | 164 226 | 139 498 (84.94 %) |
| **the increment loose adds** | **24 949** | **691 (2.77 %)** |
| *expected if the increment were uniform token coincidence over the unhit part of `U`* | — | **2.600 %** |

**Observed 2.77 % against an expected 2.60 %.** The loose extractor's entire
increment is statistically indistinguishable from uniform random token
collisions, so **`X_any` = 24 728 is not a bound on anything** and the ratio
`W6` is a fact about the loose extractor, not about §2. For contrast, chance
agreement over `U` is **11.56 %** and the tight extractor scores **99.663 %** —
the tight edges are real by a margin no coincidence model reaches.

---

## 3. Where the predicate's risk actually is: the ROOTS

`X` measures **over**-prediction from emitted referrers. It says nothing about
whether §2's *roots* are right. Two post-hoc numbers (labelled, not in the
frozen set) locate the remaining risk exactly:

| | | share of `\|E\|` |
|---|---:|---:|
| emitted names with **no** emitted `26`-referrer — **the root-set floor** | **35 608** | **20.4 %** |
| …with no emitted referrer of **any** edge kind (a looser, noise-deflated floor) | 33 246 | 19.1 % |
| transitive closure of `E` over `26`-edges, extra over `E` | 2 894 | 1.7 % |

**The emit set is already almost closed under direct reference** (closure adds
1.7 %, and that 1.7 % inherits §2.2's artifact rate). So the fixpoint is nearly
a no-op *given* the right roots, and:

> **Roots must supply 1 emitted name in 5 — about 42 per TU, 35 608 over the
> workload — and they are the one clause `PHASE7_VALIDATION.md` §6b proved
> internally inconsistent ON ITS FACE** (an out-of-line member marked `inline`
> is simultaneously "any out-of-line definition" → a root, and COMDAT → not a
> root), needing **rewriting, not patching** (§8a item 4).

**This relocates the plan's risk and it is this lane's most useful output.**
`PHASE7_PLAN.md` §2 devotes most of its text to Propagation and the Vtable
rule; those are the parts that survived contact. The one-paragraph root list is
the part that decides 20.4 % of the answer, has no consistent reading, and has
never been run against a real TU — because **w-emitpred's Part 1 died to an OOM
before freezing a prediction**, and Part 1 is exactly the root model's gate.

**What this does NOT say:** it does not say the roots are 20.4 % *hard*, or
that the remaining 79.6 % is free. It says the root clause is *necessary* for
20.4 % of `E`, and that no amount of Propagation work substitutes for it.

---

## 4. Why nothing was implemented, and why that was decided in advance

Pre-registered (prereg §5) before any number existed:

* **π ≥ 0.95 ⇒ no implementation on the strength of one lane's measurement.**
  Honoured. π (§6, W3) is **0.99731**, and after the artifact filter the bound
  is tighter still.
* **π is a CEILING, not a validation.** `X` is a *lower* bound on §2's errors,
  so `π = |E|/(|E|+|X|)` is an *upper* bound on precision. **π ≥ 0.95 means
  this measurement failed to refute §2, not that §2 is validated.** The
  project's standing rule that absence must not read as success applies to my
  own headline: a green result here is bounded by what the instrument could
  see, and it could not see the roots at all.
* Building R3 now would build the half that already works and leave the half
  that owes 20.4 % of every emitted name unmodelled and internally
  inconsistent. **`PortC2` still honestly returns `NotImplemented`; the gate
  was not widened.**

---

## 5. Known-answer controls

| # | control | registered pass | measured | |
|---|---|---|---|---|
| **KA1** | the `67` virtual-slot discriminator on 12 designed cells | 12/12 | **12/12** | **PASS**, with a caveat below |
| **KA2** | `MAGNITUDE.md`'s virtual-slot class, unmodified `detect.py`, at my rev | 649 inst / 289 TUs ± 10 % | **647 / 287** (strict 637/280 vs 638/280; folded 1049/312 vs 1049/312) | **PASS — within 0.4 %** |
| **KA3** | `E ⊆ U` coverage | ≥ 99.9 % | **99.9966 %** (6 of 174 417 outside) | **PASS** (w-emitpred: 6 of 174 410) |
| **KA4** | **the `26`-side extractor** — the control w-emitpred never ran | 3/3 | **5/5** | **PASS** |
| **KA5** | incumbent gate on the unmodified tree | see §7 | all reproduced exactly | **PASS** |
| **KA6** | hand check of `X`, n = 20, seeded | ≥ 15/20 confirmed | **fails on both variants** | **FAIL — decline clause fired, §6.1** |

**KA1's caveat, stated so it is not read as a byte-identical re-run.**
`magnitude/gate.py`'s original cell tree (`axes1/detect/`) holds IL and objs and
was therefore correctly never committed. The seven `p_*` cells are the original
committed sources, unmodified. The four `mech` cells are **reconstructed** from
the verbatim description in `PHASE7_VALIDATION.md` §3b; the twelfth,
`a6c5 tu2` — axes1's *graded violation obj* — was rebuilt from its committed
sources and **reproduces exactly**: `tu2` emits 1 of 5 with class `{?v@C}` and
no `26`-edge, while `tu1`, which constructs `C`, emits 6 of 6 with a `26`-edge.

**KA4, in full, because the headline rides on it:**

| cell | construct | `26`-edge from an emitted body | target emitted |
|---|---|---|---|
| `mf1` | `pc->v(x)` virtual dispatch | **no** (a `67`-edge instead) ✔ | no ✔ |
| `mf2` | `pc->nv(x)` non-virtual member | **yes** ✔ | yes ✔ |
| `mf3` | `pc->C::v(x)` qualified | **yes** ✔ | yes ✔ |
| `mf5` | direct call to a kept `static` | **yes** ✔ | yes ✔ |
| `mf6` | unreferenced `static` | **no** ✔ | no ✔ (and not in `U`) |

---

## 6. Scoring the pre-registration — 1 hit, 6 misses, and the misses are the result

| # | registered | measured | |
|---|---|---|---|
| **W1** | `\|X\|` = 250 000, [50 000, 800 000] | **470** (strict) / 1 215 (local) / **2 after the artifact filter** | **MISS, far below** |
| **W2** | 820 TUs of ~850, [600, 850] | **258** (strict) / 320 (local) | **MISS below** |
| **W3** | π = 0.41, [0.18, 0.78] | **0.99731** (strict) / 0.99308 (local) | **MISS above.** π ≥ w-emitpred's V3 ship floor of 0.95 |
| **W4** | π beats emit-everything by ≥ 20 pp | **+88.2 pp** (0.99731 vs 0.11562) | **HIT** — the one prediction registered in §2's favour |
| **W5** | `\|B\|/\|E\|` = 0.60, [0.30, 0.90] | **0.0023** (strict) | **MISS, far below** |
| **W6** | `X_any/X_26` = 2.0, [1.0, 5.0] | **52.6** (strict) | **MISS above → decline clause fired (§6.1)** |
| **W7** | `??_`-share of `X` = 0.15, [0.02, 0.45] | **0.0234** (strict) — inside; 0.0091 (local) — below | **HIT (strict) / MISS (local)** |

**The declared bias was deflationary and every deflationary prediction lost.**
I was briefed that the emit predicate is the critical path and that refuting a
landed plan beats a partial implementation; I registered `X` at 250 000 and
measured 470. **The one prediction I made in §2's favour is the only one that
hit.** That is the protocol working in the direction that costs me, and it is
the reason this report says §2's Propagation clause survives rather than
hedging.

### 6.1 The decline clauses, both fired, both honoured

* **`W6 > 5` ⇒ "decline to quote π as a point; publish the strict number with
  the artifact rate beside it."** Honoured: π is published as a **ceiling** with
  §2.2's recurrence table and §2.3's coincidence arithmetic beside it, and the
  loose variant is explicitly disqualified as a bound rather than averaged in.
* **`KA6 < 15/20` ⇒ "decline to quote π at all; publish only the order of
  magnitude with the measured artifact rate."** Honoured, and it is the reason
  §2.2 exists: the hand check is what forced the artifact question from an
  eyeball into the recurrence measurement. **The order of magnitude is
  10²–10³ raw and 10⁰ filtered, against a registered 10⁵.**

  KA6's draw is worth recording because it *found the instrument defect*. On
  the **strict+local** variant, 10 of 20 sampled contradictions blamed the same
  tiny header inline (`?StaticClassName@Object@Hmx@@SA?AVSymbol@@XZ`) and 5 more
  blamed `?SetType@Object@Hmx@@UAAXVSymbol@@@Z`; those two account for **60 %**
  of all blame edges. Traced to bytes: that function's local-static guard
  `??_B?1??StaticClassName…@51` carries token **`0x2667`**, a low-valued 2-byte
  token, and `detect.local_owners` attributes any unnamed segment containing
  that token to it. **The local-static owner channel is unreliable for
  low-valued tokens**, which is why this lane's headline is the **strict**
  variant — as the prereg registered it, before this was known.

### 6.2 Registered before the numbers existed, restated against them

* **`\|X\|` does not predict TU yield.** Confirmed vacuously and stated anyway:
  A is necessary and not sufficient; perfect A converts **0** TUs alone and
  moves `A∧B∧C` 25 → 107 (w-afail §4.3).
* **`\|X\|` is not a work queue**, and a small `X` is not a schedule. §3 is a
  *relocation* of risk, not a costing of it.
* **A surviving §2 does not name the correct predicate.** Failing to refute the
  Propagation clause says nothing about the root clause, which is unmeasured
  here and inconsistent as written.

---

## 7. Gate — every incumbent reproduced, on a tree with no `crates/` change

| | incumbent | this tree |
|---|---|---|
| `cargo test --workspace --release` | 687 passed, **0 failed**, 25 targets | **687 passed, 0 failed, 25 targets** |
| `cargo build --release` | 0 warnings | **0 warnings** |
| `c2rs selftest` | 219 PASS, 0 FAIL | **219 PASS, 0 FAIL** |
| `scripts/gate.sh --jobs 6` | 12/12 PASS, 2 628 verdicts, 0 mismatch | **12/12 PASS, 2 628 verdicts, 0 mismatch** |
| TU match / mismatch / vocab-gap / capture-fail | 8 / 0 / 863 / 7 | **8 / 0 / 863 / 7** |
| A / B / C / D / E | 28 / 338 / 114 / 8 / 2 | **28 / 338 / 114 / 8 / 2** |
| `A∧B∧C` / `A∧B∧C∧D` / `A∧B∧C∧(D∨E)` / `B∧C` | 25 / 6 / 8 / 107 | **25 / 6 / 8 / 107** |
| FRONTIER | 17 | **17** |
| **`census/gate disagreement`** | **0** | **0** |

*Compared on the FAILED count, never the passed count.* `cargo test` emits 3
`c2-il` **lib-test** warnings (`unused imports: ChainForm, chain_form`;
`unused variable: link`; `constant WILD_CHAIN_AS_RECV_LOAD is never used`);
they are pre-existing on master — this lane changes no Rust — and the
incumbent's "0 warnings" is `cargo build --release`, which is 0 here.

---

## 8. What this lane did NOT measure — named, so absence never reads as success

1. **§2's roots, and therefore recall, F1 and V3 itself.** No root model exists;
   `TP`, `FN` and the root half of `FP` are all unmeasured. **No V1/V2/V3 number
   may be quoted from this file.** §3's 20.4 % is the *size of the hole*, not a
   measurement of what fills it.
2. **The 21 quarantined TUs.** Untouched; **the held-out population is unspent**
   and w-emitpred's one-shot Part-1 gate is still runnable exactly once.
3. **The 254 untestable contradiction pairs** (opportunity < 2). §2.2's
   inference that they are artifacts too is labelled and unmeasured.
4. **Contradictions whose referrer is itself unemitted** (cascades), and
   **class members reachable only through an unnamed segment** — 37.8 % of
   `.ex` segments have no `.gl` name, and the local-static channel recovers
   only 2.7 % of those, and is unreliable (§6.1). Both can only make `X` larger.
5. **Virtual-dispatch and direct-call encodings outside the 17 gated cells** —
   covariant-return thunks, virtual inheritance, `novtable`, dispatch inside EH
   funclets. Unmeasured, and the one surviving contradiction is a thunk.
6. **Whether the wibo change (§0a) is inert in general.** It is inert on KA2 and
   KA5; that is two populations, not a proof.

---

## 9. Proposed board rows — **numbers NOT minted**

Four lanes proposed concurrently into `#196`–`#205` today and w-afail
deliberately minted nothing. Same here: **no number minted, no `#N` pinned in
code, `BOARD.md` / `ROADMAP.md` / `rungs/INDEX.md` untouched.** Assign at merge.

| proposed | item | claim | where |
|---|---|---|---|
| **P-a** | **§2's Propagation clause TRANSFERS to real TUs** — the root-model-free contradiction set is 470 raw and **2 instances on 1 pair** after a recurrence filter, over 850 TUs and 174 417 emitted COMDATs | every element of `X` is an error of §2 whatever its roots are; `67`-edges excluded, so this bounds the predicate **after** `PHASE7_VALIDATION` §8a repair #1 | this file §1–§2 |
| **P-b** | **The one surviving contradiction is a `$4` adjustor thunk** — `#152`'s synthesis gap, not `#161`'s propagation | 1 of 436 distinct pairs survives `recurrence ≥ 0.9`; §2 has no clause capable of producing it | §2.2 |
| **P-c** | **The predicate's whole remaining risk is the ROOT clause: it must supply 20.4 % of every emitted name** (35 608 of 174 417, ~42/TU) — and it is the clause proved internally inconsistent on its face | the transitive closure of `E` over direct edges adds only 1.7 %, so the fixpoint is nearly a no-op given the right roots | §3 |
| **P-d** | **`.gl` operand tokens are per-TU, so cross-TU RECURRENCE separates real edges from token artifacts** — median 1.000 on 8 445 controls vs 0.031 on the contradictions, a 200× separation | 95.6 % of contradiction instances resolve through a 2-byte token vs 33.8 % of controls | §2.2 |
| **P-e** | **The loose (non-`26`) token scan is indistinguishable from noise and must never be quoted as a bound** — its increment is 2.77 % emitted against 2.60 % expected under uniform coincidence | `model.ref_graph`'s deliberate over-approximation is sound for a fixpoint and **unsound for a count** | §2.3 |
| **P-f** | **`detect.local_owners` is unreliable for low-valued tokens** — two header inlines absorb 60 % of all blame because a local-static guard carries token `0x2667` | this is why `MAGNITUDE.md`'s strict/local spread exists; strict is the defensible variant for any *count* | §6.1 |
| **P-g** | **π is a CEILING on precision, not a validation** — `π ≥ 0.95` means the measurement failed to refute §2, and the instrument could not see the roots at all | registered as a ship gate, reported as a bound; no implementation followed, by pre-registration | §4 |

---

## 10. Reproducing every number here

```sh
# 1. front-end IL for 876 TUs (c2 never runs; quarantine-safe)
python3 work/emitpred/magnitude/capture_all.py $PWD/work/w-emit/il \
        work/emitpred/magnitude/tus.txt 20
# 2. truth for 850 TUs (RUNS c2; only ever pointed at truthlist.txt)
python3 work/emitpred/magnitude/truth_all.py  $PWD/work/w-emit/truth \
        work/emitpred/magnitude/truthlist.txt 12
# 3. KA1 + KA4 known-answer cells
python3 work/w-emit/ka/build.py  work/w-emit/ka/cells work/w-emit/ka/out
python3 work/w-emit/ka/kagate.py work/w-emit/ka/out
work/w-emit/ka/a6c5/run.sh                     # the 12th cell, two TUs one invocation
# 4. KA2 — w-emitpred's detector, unmodified
python3 work/emitpred/magnitude/detect.py work/w-emit/il work/w-emit/truth \
        work/emitpred/magnitude/truthlist.txt work/w-emit/class.jsonl 14
# 5. the headline scan, and the frozen scores
python3 work/w-emit/xscan.py    work/w-emit/il work/w-emit/truth \
        work/emitpred/magnitude/truthlist.txt work/w-emit/x2.jsonl 14
python3 work/w-emit/xanalyse.py work/w-emit/x2.jsonl
python3 work/w-emit/ka6.py      work/w-emit/x2.jsonl 20260804
# 6. the artifact filter
python3 work/w-emit/recur.py dump  work/w-emit/il work/w-emit/truth \
        work/emitpred/magnitude/truthlist.txt work/w-emit/edges.tsv 14
python3 work/w-emit/recur.py score work/w-emit/edges.tsv
```

All scripts are **stdlib-only** and read-only against the corpus. `work/` is
gitignored; the scripts and cell sources are force-added as text records, and
no IL, obj, `.cod` or `_CL_*` artifact is committed.
