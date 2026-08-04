# MAGNITUDE — the virtual-slot false-positive class, measured over the dc3 workload

    Lane:      w-emitpred, agent `magnitude`, 2026-08-04
    Question:  over the real workload, how many TUs and how many function names
               does PHASE7_PLAN.md §2 (board item #161) predict Emit for, via
               the virtual-slot class axes1 refuted, where c2 emits nothing?
    Scripts:   work/emitpred/magnitude/{capture_all,truth_all,detect,gate}.py
    Status:    IN PROGRESS — numbers below are final unless marked PROVISIONAL.

## 0. Provenance — every number on this page

| | |
|---|---|
| dc3-decomp rev **measured at** | **`9ad5c4c8`** (repo HEAD at measurement time) |
| other revs in play (NOT measured at) | prereg froze `51fb5b73`; plan session `13b583df`; `glgraph.py` witness `fbf097a5`; `scan-merged-20260731.jsonl` provenance says `605560e0` |
| flags | the unmodified workload line, `work/dc3-workload/flags.txt`: `/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc /I src/system/stlport /I src/xdk/LIBCMT /I src/system /I src/lazer /I src/system/oggvorbis /I src/system/synth/tomcrypt /I src/system/net/curl/include /I src` |
| compiler | X360 `16.00.11886.00` `cl.exe` / `c1xx.dll` / `c2.dll` under wibo `/home/free/code/milohax/wibo/build/release/wibo` |
| IL capture | `cl /Bd /d2nop <flags>` — c2 aborts (`C1007 … in 'p2'`), **no c2 output produced**, so quarantine-safe |
| truth scanner | `magnitude/truth_all.py` — COMDAT leaders of every section with **`IMAGE_SCN_CNT_CODE` (0x20)** set, *never* a `.text` name prefix. The harness's name-prefix rule was computed alongside on all 850 objs and **agreed 850/850** (0 disagreements) |
| TU list | the 878 `src` values of `work/dc3-workload/scan-merged-20260731.jsonl` |

**Denominators, stated once and never elided:**

* **Detector (c1xx-side) coverage: 876 / 878.** 2 TUs do not compile at
  `9ad5c4c8` at all (`src/system/synth_xbox/FxSendPitchShift360.cpp`,
  `…/FxSendSynapse360.cpp` — `error C2084: function … already has a body`), so
  they have neither IL nor obj at this rev.
* **Truth read on 850 TUs.** 878 − 21 quarantined − 7 that produce no obj at
  `9ad5c4c8` = 850. The quarantine is **21, not 20**: the held-out basename
  `FxSendEQ.cpp` resolves to *two* workload paths
  (`src/system/synth/FxSendEQ.cpp`, `src/system/synth_xbox/FxSendEQ.cpp`) and
  both were quarantined conservatively. The 7 no-obj TUs are the 2 above plus 5
  `src/system/synth_xbox/soundtouch/…` TUs.
* **No c2-output-derived artifact was read for any quarantined TU.** No obj, no
  `.cod`, no scan row. `magnitude/truthlist.txt` (857 entries) is the only list
  `truth_all.py` was ever pointed at, and `magnitude/heldout.txt` is its
  complement.

## 1. Headline

**Over 850 TUs at dc3 `9ad5c4c8`, PHASE7_PLAN §2's Propagation clause
over-predicts Emit via the virtual-slot class on:**

| | count | denominator |
|---|---:|---|
| **TUs with ≥ 1 false positive of this class** | **312** | 850 TUs with truth (36.7 %) |
| **false-positive name-instances (TU × name)** | **1 049** | — |
| **distinct decorated names involved** | **584** | — |
| total emitted code COMDATs over the same 850 TUs | 174 410 | — |
| total named bodies in the IL over the same 850 TUs | 1 508 554 | — |

## 2. The V3-relevant fraction

V3 is a **micro-precision** gate (≥ 0.95). Precision is `TP / |P|` where `P` is
§2's predicted-Emit set. §2's *full* predicted set is **not** computable with
what this lane has — Part 1 (the fitted root model) was not finished, and a
guessed root set would make the denominator fiction. So the number is given as
a **bound that needs no root model at all**, and it is tight in the direction
that matters:

* Every name this measurement flags is (a) predicted Emit by §2 — the referrer
  is *itself emitted*, so it is definitely a kept definition, and §2's
  Propagation clause says a call in its pre-optimization body adds the callee —
  and (b) not emitted. So all 1 049 are **false positives of §2, with certainty
  about §2's prediction**, no root model required.
* True positives are bounded by truth: `TP ≤ |E| = 174 410`.
* Therefore, **whatever §2's roots turn out to be:**

      micro-precision  =  TP / (TP + FP)  ≤  174 410 / (174 410 + 1 049)  =  0.99402

**This class alone costs at most 0.598 percentage points of V3 micro-precision,
and cannot on its own take V3 below 0.95.** As a share of the smallest possible
predicted set, the class is **1 049 / 175 459 = 0.598 %**.

The complementary reading — *the class is not small in TU terms*: it touches
**36.7 % of TUs**. A fail-closed R3 that refuses a TU on encountering this
construct would refuse better than one TU in three.

*(Sections 3–6 — detector, error rate, distribution, verdict — below.)*
