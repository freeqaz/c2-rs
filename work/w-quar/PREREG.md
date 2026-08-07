# w-quar — PRE-REGISTRATION: spending w-emitpred's one-shot Part-1 quarantine gate on `JFP_ALIAS`

    Lane:      w-quar, worktree `wt-w-quar`, branched at master `5bef565f`
    Written:   BEFORE any quarantined `out.obj` is opened, before any truth
               artifact for any of the 21 exists, and before the first `cl.exe`
               of this lane runs.  Committed as a git object; the commit that
               carries this file also carries the 21 predicted symbol SETS
               (`pred21.sets.txt`) and their digests (`pred21_sha.txt`).
    Board:     #1027–#1036
    Ships:     NOTHING under `crates/`.  This is a measurement lane.

**This gate is spent exactly once and it is being spent now.**  It was
registered as unspendable until board **#960** closed; #960 closed with lane
`w-inread` (merge `8d2956b`), and `w-emitp2` §5 registered the spending
condition verbatim as *"spend it on the model built after #960 lands, which will
have a genuinely new node universe"*.  The coordinator's decision to spend it
now, before Phase 7 integration rather than after, is executed here and not
re-litigated: the gate exists to decide whether integration is justified, and a
gate spent after the integration cannot decide that.

---

## 0. The population, and the proof it is untouched

The 21 TUs are `work/emitpred/magnitude/heldout.txt`, drawn by
`random.Random(161).sample(...)` in w-emitpred's own prereg (2026-08-02) and
never scored since.

    src/keygen_xbox.cpp
    src/lazer/game/PartyModeMgr.cpp
    src/lazer/meta_ham/CalibrationPanel.cpp
    src/lazer/meta_ham/Challenges.cpp
    src/lazer/meta_ham/DifficultyProvider.cpp
    src/lazer/meta_ham/HamStoreOffer.cpp
    src/system/os/Archive.cpp
    src/system/os/FileCache.cpp
    src/system/os/HolmesClient_NetSocket.cpp
    src/system/os/Joypad.cpp
    src/system/os/Keyboard_Xbox.cpp
    src/system/rndobj/TexProc.cpp
    src/system/synth/FxSendEQ.cpp
    src/system/synth_xbox/FxSendEQ.cpp
    src/system/synth_xbox/soundtouch/source/SoundTouch/FIRFilter.cpp
    src/system/ui/LocalePanel.cpp
    src/system/ui/PanelDir.cpp
    src/system/ui/UIListMesh.cpp
    src/system/ui/UIListWidget.cpp
    src/system/utl/Compress.cpp
    src/system/utl/Option.cpp

`work/w-quar/contam.py`, output in `contam.txt`, **re-derived rather than
cited** (commit `d210d664`):

| population | size | ∩ quarantine | |
|---|---:|---:|---|
| `work/w-db/cacheidx.tsv` — the 850-TU model corpus | 850 | **0** | must be 0 |
| `work/emitpred/magnitude/truthlist.txt` — every truth read | 857 | **0** | must be 0 |
| w-emitpred DEV (truth-open) | 8 | **0** | must be 0 |
| `magnitude/tus.txt` — the 878 workload | 878 | **21** | **CONTROL** — the test can fail |
| `work/dc3-workload/files.txt` — the 878 workload | 878 | **21** | **CONTROL** |
| `work/w-emit/truth/*.txt` — E truth on disk | 1 700 files | **0** | must be 0 |
| `work/emitpred-truth/*` | 1 700 files | **0** | must be 0 |
| every `dtruth/` any lane left on disk | — | **0** | must be 0 |

**The one false alarm is written down rather than filtered away.**  The
harness's own `c2rs gap` workload scans run over all 878 TUs, so the 21 appear in
**57** scan jsonls on disk.  Those rows carry no c2 emitted-symbol quantity at
all — the keys are c1xx-side (`fn_names`, `fn_total`, `ex_len`, the blocker
histograms), port-side (`emit-*` is what **`PortC2`** emitted, never what c2
emitted), plus `class` and `replay_ok`, and w-emitpred's prereg drew its own
population from exactly those class labels.  `contam.py` classifies every scan
by whether it carries an emit-set grader's keys and prints the verdict per file.

**And `0 EMIT-MODEL scans on disk` is recorded as an ABSENCE, not as evidence.**
Those jsonls are regenerated and never committed.  The disjointness rests on the
input list: every emit-model scan reads `cacheidx.tsv` as its **only** TU list,
and a scan cannot reach a TU its input does not name.

---

## 1. The model under test, frozen

**`JFP_ALIAS`** — w-db's joint fixpoint `JFP` with `.in`-stream `02`-node targets
resolved through w-emitp's tag-0x10 alias table.  It conditions on **no truth**.

Frozen at commit `e75f46ac`, `work/w-quar/predict.py`, whose whole import closure
is digested by `freeze.py` at each module's **resolved** `__file__`:

    MODEL-CLOSURE-SHA256  15b9a571f15c5fa88172d367fd64ae932ed5e54f5e8650def91b0049ee01c44f
                          over 16 files (work/w-quar/freeze.txt)

`predict.py` reformulates nothing: `fixpoint` and the alias resolution `_resmap`
are w-emitp's own functions, loaded **by path** because `scan.py`, `glowner.py`
and `marks.py` each exist in more than one lane and a bare import silently picks
another lane's copy.

**In-sample known-answer control, re-run at this master (`ka850.txt`):**

| model | precision | recall | F1 | `\|P\|` | **EXACT / 850** |
|---|---:|---:|---:|---:|---:|
| `NEVER` (`P = ∅`) | — | 0.00000 | — | 0 | **6** (0.00706) |
| `ALL` (`P = U`) | 0.11543 | 0.99708 | 0.20691 | 1 506 586 | **32** (0.03765) |
| `RGL` | 1.00000 | 0.74307 | 0.85260 | 129 604 | **132** (0.15529) |
| `INIT` | 0.27289 | 0.95991 | 0.42496 | 613 532 | 34 |
| `SKIP` | 0.36420 | 0.83732 | 0.50761 | 400 998 | 34 |
| **`JFP`** — the INCUMBENT | 0.99899 | 0.86391 | 0.92655 | 150 833 | **132** (0.15529) |
| **`JFP_ALIAS`** — under test | 0.99825 | 0.89558 | **0.94413** | 156 479 | **308** (0.36235) |

Every figure is identical to w-emitp §2.2, w-emitp2 §1.2 and w-inread §1.2.
`NEVER` and `ALL` are measured here for the first time on this population; they
are the two trivial incumbents the protocol requires and they are **not**
assumed.

**`ALIAS_IN` is registered as an ORACLE-CONDITIONED CEILING and nothing else.**
It conditions on `D`, the obj's defined-data symbol table, so it cannot be
computed before truth is read and it is not a model.  In sample it is
0.99997 / 0.98500 / F1 0.99243 / **472 of 850 (0.55529)**.  **The verdict of this
gate rests on `JFP_ALIAS` alone.**  `ALIAS_IN` is scored on the 21 afterwards
purely as an upper reference, and no ship decision may be taken from it.

---

## 2. The predictions — committed before the first compile

`work/w-quar/pred21.sets.txt` carries the **full sorted predicted name set** for
`JFP_ALIAS` and for `JFP`, per TU, each block self-digesting.
`work/w-quar/pred21_sha.txt` is the index:

| src | `\|U\|` | `\|JFP\|` | `\|JFP_ALIAS\|` |
|---|---:|---:|---:|
| `src/keygen_xbox.cpp` | 20 | 20 | 20 |
| `src/lazer/game/PartyModeMgr.cpp` | 4 845 | 648 | 654 |
| `src/lazer/meta_ham/CalibrationPanel.cpp` | 5 277 | 158 | 164 |
| `src/lazer/meta_ham/Challenges.cpp` | 5 705 | 615 | 619 |
| `src/lazer/meta_ham/DifficultyProvider.cpp` | 5 106 | 100 | 103 |
| `src/lazer/meta_ham/HamStoreOffer.cpp` | 1 014 | 43 | 44 |
| `src/system/os/Archive.cpp` | 1 127 | 325 | 325 |
| `src/system/os/FileCache.cpp` | 1 376 | 252 | 260 |
| `src/system/os/HolmesClient_NetSocket.cpp` | 697 | 58 | 58 |
| `src/system/os/Joypad.cpp` | 805 | 132 | 135 |
| `src/system/os/Keyboard_Xbox.cpp` | 658 | 4 | 4 |
| `src/system/rndobj/TexProc.cpp` | 1 271 | 236 | 246 |
| `src/system/synth/FxSendEQ.cpp` | 664 | 82 | 90 |
| `src/system/synth_xbox/FxSendEQ.cpp` | 1 195 | 34 | 41 |
| `.../SoundTouch/FIRFilter.cpp` | 215 | 19 | 20 |
| `src/system/ui/LocalePanel.cpp` | 2 291 | 243 | 243 |
| `src/system/ui/PanelDir.cpp` | 2 227 | 610 | 629 |
| `src/system/ui/UIListMesh.cpp` | 2 079 | 198 | 214 |
| `src/system/ui/UIListWidget.cpp` | 1 628 | 250 | 257 |
| `src/system/utl/Compress.cpp` | 206 | 11 | 11 |
| `src/system/utl/Option.cpp` | 644 | 29 | 29 |
| **total** | **39 050** | **4 067** | **4 166** |

Corpus digests over the per-TU digests (`pred21_sha.txt`):

    JFP_ALIAS  d58a733b070375e84b11690f82dd6a3500deb185207cc3908af35119e2a27387
    JFP        79cadde0ac071d258e670a6418e19f2e6ee0dbe58cd505d95da8fc0b50763dc2
    RGL        cdf01fe2a85fa50b2a53b15d8a32a1c5888e1e5cdd10b47d65ebfda1642813da
    INIT       0966f02d38a2bff290a4ba19d0e96266444e89c03629b6204095dad2edf3377d
    SKIP       f0d7b1f9a2a14f6fcb9e8a52e1988e09cf882095021b1a2fcb6cbc49a7d40327
    ALL        c2f8725f557e31e906f9063ae7336a379c654c3405e9ec5f6ddfe9eda1e7d901
    NEVER      bc1ae2dc10b9695f5c149ce76209e84e2aafc99db58e8e72658d5c2dca72321d

`NEVER`/`ALL`/`RGL`/`INIT`/`SKIP` sets are pinned by digest rather than written
out in full (`ALL` alone is 39 050 names); they are reproducible from the frozen
model and `pred21.jsonl`.  **`JFP_ALIAS` and `JFP` — the model and its control —
are committed in full.**

---

## 3. THE REGISTERED GATES — points, intervals, and the decline floor

The unit is **per-TU exact by name** (`P(t) == E(t)` as sets), because that is
the metric factor **A** is a conjunction over and the one `STATUS.md` trap 8 says
must not be replaced by micro-F1.

### 3.1 The controls, named — never a bare threshold

The incumbents this model must beat, each with its own in-sample rate and the
expectation that rate implies on `n = 21`:

| control | in-sample | expected on 21 |
|---|---:|---:|
| `NEVER` — the no-model baseline (`P = ∅`) | 6/850 = 0.00706 | **0.15** |
| `ALL` — the port's current behaviour (`P = U`) | 32/850 = 0.03765 | **0.79** |
| **`JFP` — the PRIOR MODEL, the incumbent that matters** | 132/850 = 0.15529 | **3.26** |
| **`JFP_ALIAS` — under test** | 308/850 = 0.36235 | **7.61** |

### 3.2 The gates

| # | quantity | **point** | interval | gate |
|---|---|---:|---|---|
| **Q1** | `JFP_ALIAS` per-TU exact of 21 | **8** | **[3, 12]** — the 97.9 %-coverage acceptance region of Binomial(21, 0.36235); `P(X ≤ 2) = 0.0064`, `P(X ≥ 13) = 0.0148`. Mean 7.61, sd 2.20, mode 7 | **≤ 2 ⇒ DECLINE.** The model does not generalize and Phase 7 must be re-scoped |
| **Q1′** | the same, stratified on `B∧C` (board #348 names the 6 quarantined `B∧C` TUs; #302 measures `JFP_ALIAS` exact on 122 of 145 `B∧C` TUs and 186 of 705 outside it) | **9** | **[5, 13]** (Poisson-binomial, 6 × 0.84138 + 15 × 0.26383; mean 9.01) | reported, not gated. Registered because w-reach's lesson is that a levels-only prereg cannot see an increment |
| **Q2** | **`JFP_ALIAS` − `JFP`, paired, on the same 21** | **+4** | [0, +9] | **`JFP_ALIAS` ≤ `JFP` ⇒ DECLINE the alias channel** even if Q1 passes: the whole claimed contribution failed to transfer |
| **Q2′** | TUs `JFP` gets exact that `JFP_ALIAS` LOSES, **by name** | **0** | [0, 1] | in sample the alias channel lost **0 of 132**. ≥ 2 lost ⇒ the channel is not monotone out of sample and must be said so |
| **Q3** | `JFP` per-TU exact of 21 | **3** | [0, 7] | the control's own sanity check; `JFP` = 0 with `JFP_ALIAS` ≥ 6 would mean the two populations are not comparable and the gate is uninformative |
| **Q4** | `NEVER` per-TU exact of 21 | **0** | [0, 2] | reported |
| **Q5** | `ALL` per-TU exact of 21 | **1** | [0, 4] | reported |
| **Q6** | `JFP_ALIAS` micro-precision over the 21 | **0.998** | [0.97, 1.000] | **SECONDARY, NOT THE VERDICT (trap 8).** < 0.95 ⇒ not shippable into a fail-closed emitter (w-emitpred V3, restated) |
| **Q7** | `JFP_ALIAS` micro-recall over the 21 | **0.896** | [0.80, 0.96] | secondary |
| **Q8** | `JFP_ALIAS` micro-F1 over the 21 | **0.944** | [0.88, 0.98] | **secondary and explicitly not the verdict** |
| **Q9** | alias decode on the 21: bound / tag-0x10 | **0.996** | [0.95, 1.00] | a decode failure here would mean the channel does not even parse out of sample |
| **Q10** | `dom(alias) ∩ U` over the 21 | **0** | [0, 20] | w-emitp measured 0 over 95 820 records; a non-zero here refutes an invariant the §6 spec asks the reader to *assert* |
| **Q11** | `dom(alias) ∩ E` over the 21 | **0** | [0, 5] | an alias must never be emitted. > 5 ⇒ spec item 4 is wrong out of sample |
| **Q12** | `??_E<X>` → `??_G<X>` share of bound aliases | **0.99998** | [0.95, 1.00] | |
| **Q13** | `ALIAS_IN` (the CEILING) per-TU exact of 21 | **12** | [6, 17] | reported as an upper reference **only**; no ship decision may cite it |
| **Q14** | TUs where the model is exact **and** in `B∧C` — i.e. TU *reach*, board #302's quantity | **5** | [2, 6] | reported. Reach is what Phase 7 is priced in, and #302's headline is that the alias channel's reach increment was **+0** |
| **Q15** | the fresh replay through real `c2.dll` under wibo byte-matches the cached `out.obj` (TimeDateStamp zeroed) on all 21 | **21/21** | [19, 21] | **the toolchain control.** < 19 ⇒ INSTRUMENT-FAIL, the gate is unspendable today and the verdict is withheld rather than reported |

### 3.3 The decline rule, fixed now

* **GENERALIZES** — Q1 ≥ 5 **and** Q2 > 0 **and** Q15 = 21/21.  The model's
  out-of-sample rate is consistent with 0.36235 and it strictly beats every
  registered control.
* **GENERALIZES WEAKLY** — Q1 ∈ [3, 4] and Q2 > 0.  Inside the acceptance
  region but in its lower tail; the point estimate is reported with its interval
  and the verdict says the sample cannot separate 0.36 from 0.20.
* **DECLINE — DOES NOT GENERALIZE** — Q1 ≤ 2, **or** Q2 ≤ 0.  Reported as a
  refutation out of sample.  **The misses are analysed only to name axes.  No
  patching, no re-scoring, no second model.**
* **INSTRUMENT-FAIL** — Q15 < 19.  Not a pass and not a refutation.
* Whatever the outcome, **this population is spent** and may never be reused as
  a held-out set.

### 3.4 What is registered as NOT following from a pass

* **A pass does not convert a single TU.**  `PortC2` has no emit-set model at
  all; #302 measured the alias channel's TU-reach increment at **+0** without a
  `.rdata$r` writer, and nothing in this lane changes that.
* **A pass is not a proof.**  21 TUs is one coverage-bounded sample at one set
  of flags (`/O1 /Oi /EHsc /GR`, the workload's own line); the verdict language
  will say "survived 21 held-out TUs", never "correct".
* **Order is untouched.**  A right set in the wrong order is still a mismatch.
* **`ALIAS_IN` is a ceiling.**  No ship decision follows from Q13.

---

## 4. Declared bias, and the guards against it

**Inflationary.**  I am executing a decision to spend the gate, and a lane that
spends a one-shot resource has an incentive to come back with a pass.  Guards:

1. **Every threshold above is a number, registered before the first compile,
   with the direction that would refute it named.**  Q2 can fire even if Q1
   passes.
2. **The controls are models, not thresholds** — `JFP`, `NEVER` and `ALL` are
   run on the *same* 21 through the *same* frozen code, so "improvement" is a
   paired comparison and not a comparison to a remembered number.
3. **Per-TU exact is reported BY NAME on both sides** (board #250: a count is
   not a set), and gained/lost against `JFP` as a set difference.
4. **The secondary metrics are labelled secondary in the table itself**, because
   trap 8's failure mode is a lane that reports micro-F1 when per-TU exact did
   not move.
5. **No re-fit.**  If the model lands below the floor it is declined as it
   stands.  A patched model has spent the gate and learned nothing.

## 5. Method constraints, fixed now

* The judge is **real `c2.dll` under wibo at the workload's own `flags.txt`**,
  unmodified.  Truth `E(t)` is the obj's **code COMDAT leader set**
  (`objsyms.sets()[["E"]]`, the `IMAGE_SCN_CNT_CODE` characteristic rule, not a
  `.text` name prefix).  A `.cod` listing is never used as truth.
* The 21 sources are byte-identical at dc3 `940d07dc` (the rev the 850 are
  indexed at) and at today's HEAD `a44b1cf9` — `revcmp.py`, 21 identical / 0
  differ — so the fresh compile and the cached entry are the same compile, and
  Q15 checks that claim instead of assuming it.
* **Nothing under `crates/`.**  `git diff 5bef565f -- crates/ scripts/
  Cargo.toml Cargo.lock fixtures/` must be 0 bytes at the end, and the lane
  re-gates anyway.
* No IL, obj or `_CL_*` artifact is committed; no absolute machine path is
  committed; `work/` files are force-added.
