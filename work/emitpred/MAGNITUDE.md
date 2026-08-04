# MAGNITUDE — the virtual-slot false-positive class, measured over the dc3 workload

    Lane:      w-emitpred, agent `magnitude`, 2026-08-04
    Question:  over the real workload, how many TUs and how many function names
               does PHASE7_PLAN.md §2 (board item #161) predict Emit for, via
               the virtual-slot class axes1 refuted, where c2 emits nothing?
    Scripts:   work/emitpred/magnitude/*.py  (reproducible; see §7)
    Answer in one line: **289 TUs, 649 name-instances, 331 distinct names — a
               0.37 % share of the smallest possible predicted-Emit set, which
               caps V3 micro-precision at 0.9963. #161 needs a CLAUSE.**

---

## 0. Provenance — every number on this page

| | |
|---|---|
| dc3-decomp rev **measured at** | **`9ad5c4c8`** (repo HEAD at measurement time) |
| other revs in play (explicitly NOT measured at) | prereg froze `51fb5b73`; the plan session used `13b583df`; `glgraph.py`'s witness is `fbf097a5`; `scan-merged-20260731.jsonl`'s own provenance record says `605560e0`. **No number here is comparable to a cached figure from any of those without re-deriving it.** |
| flags | the unmodified workload line, `work/dc3-workload/flags.txt`: `/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc /I src/system/stlport /I src/xdk/LIBCMT /I src/system /I src/lazer /I src/system/oggvorbis /I src/system/synth/tomcrypt /I src/system/net/curl/include /I src` |
| compiler | X360 `16.00.11886.00` `cl.exe` / `c1xx.dll` / `c2.dll` under wibo (`/home/free/code/milohax/wibo/build/release/wibo`) |
| IL capture (detector input) | `cl /Bd /d2nop <flags>` — c2 aborts (`C1007 … in 'p2'`); **no c2 output is produced at all**, so this is quarantine-safe |
| truth scanner | `magnitude/truth_all.py` — COMDAT leaders of every section carrying **`IMAGE_SCN_CNT_CODE` (0x20)**, *never* a `.text` name prefix. The harness's name-prefix rule was computed in parallel on all 850 objs and **agreed 850/850, zero disagreements** (the trap was checked, and did not bite at this rev) |
| TU list | the 878 `src` values of `work/dc3-workload/scan-merged-20260731.jsonl` |

**Denominators, stated once and never elided:**

* **Detector (c1xx-side) coverage: 876 / 878 TUs.** Two TUs do not compile at
  `9ad5c4c8` at all — `src/system/synth_xbox/FxSendPitchShift360.cpp` and
  `…/FxSendSynapse360.cpp` (`error C2084: function … already has a body`) — so
  they have neither IL nor obj at this rev.
* **Truth read on 850 TUs.** 878 − 21 quarantined − 7 producing no obj = 850.
  **The quarantine is 21, not 20**: the held-out basename `FxSendEQ.cpp`
  resolves to *two* workload paths (`src/system/synth/FxSendEQ.cpp`,
  `src/system/synth_xbox/FxSendEQ.cpp`) and **both** were quarantined
  conservatively. The 7 no-obj TUs are the 2 above plus 5
  `src/system/synth_xbox/soundtouch/…` TUs.
* **No c2-output-derived artifact was read for any quarantined TU.** No obj, no
  `.cod`, no scan row, no `emit-*` key. `magnitude/truthlist.txt` (857 entries,
  the complement of `magnitude/heldout.txt`) is the only list `truth_all.py`
  was ever pointed at; the quarantined TUs appear in this report **only**
  through `.gl`/`.ex`, which are c1xx-side inputs and explicitly allowed.
* DEV TUs (Part, CharWeightSetter, Gen, CharClipDriver, FlowLabel, MoggClipMap,
  ShadowMap, HolmesUtl) are truth-open and are inside the 850.

---

## 1. Headline

**Over 850 TUs at dc3 `9ad5c4c8`, §2's Propagation clause over-predicts Emit
via the virtual-slot class on:**

| | count | denominator |
|---|---:|---|
| **TUs with ≥ 1 false positive of this class** | **289** | 850 TUs with truth — **34.0 %** |
| **false-positive name-instances (TU × name)** | **649** | — |
| **distinct decorated names involved** | **331** | — |
| total emitted code COMDATs over the same 850 TUs | 174 410 | — |
| total `.gl`-named bodies in the IL over the same 850 TUs | 1 508 554 | — |

Composition of the 649 instances: **481 ordinary virtual member functions, 166
`??_G` scalar-deleting destructors** (the `delete p`-through-a-base-pointer form
axes2 pinned on `a9_06`), **2 provable detector artifacts** (§3).

**Attribution sensitivity** — the one real uncertainty, measured rather than
assumed (§3b):

| owner-attribution mode | TUs | instances | distinct |
|---|---:|---:|---:|
| **strict** (edges only from `.gl`-named bodies) — sound lower bound | 280 | 638 | 331 |
| **strict + local-static owner recovery** — **headline** | **289** | **649** | **331** |
| *folded* (the recovered pipeline's rule) — **known-unreliable, upper variant** | 312 | 1 049 | 584 |

The folded row is reported only for sensitivity: `attrib.py` grades that rule
**1 / 14 842 correct** and it is not used for any conclusion.

**IL-side detector run over all 876 captured TUs (no truth):** the truth-free
*candidate* set — a name with a body in the TU whose only reference from any
attributed body is a vtable-slot edge — is **27 917 instances over 789 TUs**, of
which **668 instances over 19 TUs** fall in the 21 quarantined TUs. The
truth-validated class is 649 / 27 917 = **2.3 %** of candidates; applying that
rate to the quarantine predicts **≈ 15 instances in the held-out 21**. That is
an *extrapolation, explicitly labelled*, not a measurement — the quarantine
forbids measuring it and it stays unmeasured until the prereg's one-shot gate
is scored.

---

## 2. The V3-relevant fraction

V3 is a **micro-precision** gate (≥ 0.95): `precision = TP / |P|`, `P` = §2's
predicted-Emit set.

**§2's full predicted set is NOT computable with what this lane has, and I am
not going to invent it.** Part 1 (the fitted root model) was never finished —
the recovered `work/emitpred/pipeline/` + `scratch/` show its author still
sweeping root definitions (`roots=U−H`, `indeg0`, the union) with no gated
answer. A guessed root set would make the denominator fiction. So the number is
given as a **bound that needs no root model at all**, and it is tight in the
direction that decides:

* Every one of the 649 flagged names is (a) **predicted Emit by §2 with
  certainty** — the referring body is *itself emitted*, hence definitely a kept
  definition, and §2's Propagation clause says a call anywhere in its
  pre-optimization body adds the callee — and (b) **not emitted**. No root model
  is needed to establish either half.
* True positives are bounded by truth: `TP ≤ |E| = 174 410`.
* Therefore, **whatever §2's roots turn out to be:**

      micro-precision  =  TP / (TP + FP)  ≤  174 410 / (174 410 + 649)  =  **0.99629**

**As a share of the smallest possible predicted set: 649 / 175 059 = 0.371 %.**
Under the unreliable folded upper variant it is 1 049 / 175 459 = 0.598 %
(ceiling 0.99402).

**Sensitivity.** For this class *alone* to drag V3 micro-precision to 0.95 it
would have to number **9 179 instances** — **14.1×** the measured figure, and
**8.8×** even the known-inflated folded variant. The V3 conclusion is therefore
robust to every source of error identified in §3 by an order of magnitude.

**This class does not break V3. It is not the thing that will.** The tightest
honest statement about V3 is that it remains **unmeasured**, because `TP` and
the rest of `FP` both require the root model that does not exist yet; what is
now measured is that *this* class contributes at most 0.37 percentage points.

---

## 3. The detector, and its error rate

### 3a. The discriminator — a new, measured `.ex` fact

axes1 reported that `.gl` *names* cannot discriminate this class, and that the
detector that works is "syntactic and source-side". **The `.ex` operand stream
discriminates it directly**, which is cheaper and is what this measurement uses:

    direct call / reference     26 <token>
    VIRTUAL DISPATCH            67 <vtable-byte-offset> <token>

i.e. the byte two before an operand token is `0x67` exactly when the reference
goes through a vtable slot, and the byte immediately before the token is the
**byte offset of the slot**.

**Known-answer gate — `magnitude/gate.py`, 12 / 12 pass**, each graded against a
real obj compiled here:

| cell | construct | expected class | got |
|---|---|---|---|
| axes1 `mech/f1` | `pc->v(x)`, no ctor kept | `{?v@C}` | ✔ |
| axes1 `mech/f2` | `pc->nv(x)` non-virtual | `{}` | ✔ |
| axes1 `mech/f3` | `pc->C::v(x)` qualified | `{}` | ✔ |
| axes1 `mech/f4` | `&C::v` in a data init | `{}` (a `??_9` thunk, no vcall) | ✔ |
| axes1 `a6c5/tu2` | **the graded VIOLATION obj** | `{?v@C}` | ✔ |
| `p_w`, `p_u` | slots `0c`, `10` | the dispatched fn | ✔ |
| `p_ref` | receiver is a `C&` | `{?v@C}` | ✔ |
| `p_del` | `delete pc`, slot `00` | `{??_GC}` | ✔ |
| `p_base` | base-class virtual | `{?bv@B}` | ✔ |
| `p_mi` | multiple inheritance, two vftables | both | ✔ |
| `p_ctor` | *same* call **with** a kept ctor | `{}` — all virtuals emit | ✔ |

`p_ctor` is the control that matters: the only change from `f1` is a kept
constructor, and the whole class disappears — the vtable rule, not the call, is
what keeps a virtual.

### 3b. Class rule (no root model)

    F is a virtual-slot false positive in TU t  iff
      (1) F has a `.gl`-named body in t's IL
      (2) some EMITTED body A of t has a `67`-kind edge A -> F
      (3) no EMITTED body of t has a non-`67` edge to F
      (4) F is not emitted

### 3c. Error rate — four independent checks

1. **Provable artifacts inside the flagged set: 2 / 649 = 0.31 %.** A vtable
   slot can only hold a virtual function, a `??_G`/`??_E` deleting destructor or
   a `??_9` thunk, so any flagged name that demangles to something else is a
   token coincidence. Exactly two do: `?Size@DataArray@@QBAHXZ` and `fdimf`.
   Deducting them gives **647** genuine instances; the headline is left at 649
   so no number here is quietly improved.
2. **Slot-offset hygiene — zero violations, and this test is free.** A real
   vcall's slot byte must be a multiple of 4 on this ABI, and must be the *same*
   for a given function in every TU that dispatches to it. Over the 623 flagged
   cells that carry a slot byte: **0 with `off % 4 != 0`**, and **0 of 319
   distinct names disagree about their offset across TUs.** Under a coincidence
   model those two properties would hold with probability ≈ `4^-623`. Neither
   property is used by the detector.
3. **Raw-edge artifact rate, and why the class rule survives it.** Across all
   `67`-edges in the workload, 5 689 / 53 183 instances (10.7 %) point at a name
   that demangles to a *non-virtual* function — those are pure token
   coincidences. The class rule's conditions (1)/(3)/(4) filter them down to
   0.31 % of the flagged set. The raw rate is reported so the filter's strength
   is visible rather than assumed.
4. **Hand check, n = 20, stratified by a seeded uniform draw over the 649 cells
   (`magnitude/sample20.json`, seed 20260804): 20 / 20 confirmed, 0
   counterexamples.** Every sampled name is a `virtual` member declared with an
   inline body in a header the TU includes (verified in the dc3 sources:
   `MidiReceiver::OnAcceptMaps`, `UIListProvider::IsSnappableAtData` /
   `IsHidden`, `RndShaderMgr::GetPostProcMat` / `GetWork`, `FlowNode::IsRunning`
   / `ClassName`, `CharWeightable::SetWeight`, `Playlist::IsCustom`,
   `Profile::PreLoad`, `UIListSlotElement::Poll`, `File::WriteDone`,
   `NavListHeaderNode::GetToken`, `RndDrawable::GetDistanceToPlane`,
   `RndFontBase::AspectRatio`, `Synth::HasPendingVoices`, `Hmx::Object::ClassName`,
   two `??_G` deleting destructors). For 10 of them the dispatching call site in
   the flagged TU was located in the source by hand (`mRcvr.OnAcceptMaps(…)`,
   `Provider()->IsSnappableAtData(…)`, `TheShaderMgr.GetPostProcMat()`,
   `…->IsRunning()`, `mAnimBlender->SetWeight(…)`, `bs.Cached()`,
   `syncAnim->EndAnim()`, `it->RefOwner()`, `prov->SlotColorOverride(…)`,
   `RELEASE(mStreams[0])` on a `Stream*` with a virtual dtor).
   **0 / 20 gives a one-sided 95 % upper bound on the cell error rate of
   13.9 %** — that is what n = 20 buys and no more; checks 1–3 are what make the
   number tight, and the hand check is what makes them trustworthy.

### 3d. The attribution problem — found, measured, and routed around

`model.named_bodies` binds a name to only **66.3 %** of `.ex` body segments
(711 057 of 1 072 248). The recovered pipeline folds each unnamed segment onto
the nearest *preceding named* one. **That rule is wrong.**

`magnitude/attrib.py` grades it with an independent owner channel — a
function-local static's `.gl` name embeds its owner's decorated name
(`??_B?4??SetType@RndTex@@UAAXVSymbol@@@Z@54`) — and finds the folding rule
correct on **1 of 14 842** gradeable unnamed segments. Worked example: in
`Rnd.cpp` the `67 0c` dispatch to `?ClassName@RndTex@@UBA?AVSymbol@@XZ` lives in
an unnamed segment that folding credits to the *emitted*
`?StaticClassName@RndTex@@SA?AVSymbol@@XZ`, while the segment's own local
statics show it is `?SetType@RndTex@@UAAXVSymbol@@@Z`, which is **not** emitted.
That single misattribution accounts for most of the folded variant's 400-instance
excess.

Two facts make the strict rule sound rather than merely conservative:

* **`E ⊆ U` on 174 404 of 174 410 emitted names** (6 exceptions in 4 TUs, e.g.
  `?name@type_info@@QBAPBDXZ`). Every emitted function has at least one *named*
  body, so dropping unnamed segments never loses a whole caller — only *extra*
  segments of a caller that is already represented.
* Of the gradeable unnamed segments, only **4.1 %** have an owner that is
  emitted at all, and 27 % have an owner that already appears in `U`.

**Residual, stated as unmeasured:** 33.7 % of segments have no recoverable
owner, so `strict`/`strict+local` may still miss class members whose *only*
dispatching segment is unnamed. The local-static channel recovers a name for
only ~4 % of unnamed segments. This is the detector's dominant remaining
weakness and the reason the headline is presented as a lower bound.

### 3e. Limits, stated plainly

* The detector reads a **byte pattern in an undocumented IL**. It is gated on 12
  designed cells and 4 workload-scale consistency checks; it is not a decoder.
  If c1xx encodes a virtual call any other way in some construct not in the gate
  set (covariant-return thunks, virtual inheritance, `__declspec(novtable)`,
  virtual calls inside EH funclets), those cases are **missed, silently**. None
  of them is in the gate set; they are unmeasured.
* The token scan **over-approximates by construction** (inherited from
  `il.read_token_var` / `model.ref_graph`, kept deliberately, and not tightened
  without a known-answer gate). §3c bounds the resulting artifacts at 0.31 % of
  the flagged set.
* `?Size@DataArray@@QBAHXZ` and `fdimf` are known-wrong flags left in the count.
* The class rule requires the **referrer** to be emitted. A §2 false positive
  whose only referrer is itself a §2 false positive (a two-step over-prediction
  cascade) is **not counted**. That direction is unmeasured and can only make
  the true figure larger.

---

## 4. Distribution — spread wide, but thin

| per-TU class size | TUs |
|---|---:|
| 0 | 561 |
| 1 | 136 |
| 2–4 | 125 |
| 5–9 | 25 |
| 10+ | 3 |

* **34.0 % of TUs are touched** (289 / 850) — this is *not* a 3-TU curiosity.
* But it is **thin where it touches**: median 2 instances per affected TU, max
  17 (`src/system/movie/Movie.cpp`). The top 65 TUs carry half the instances;
  225 TUs carry 90 %.
* **Concentrated in names, not in TUs.** 331 distinct names produce 649
  instances, and the top 10 names alone produce 185 (28.5 %). The head is
  header-inline virtuals and container-node deleting destructors that everything
  includes:

  | instances (TUs) | name |
  |---:|---|
  | 67 | `??_GNode@?$ObjPtrVec@VRndTransformable@@VObjectDir@@@@UAAPAXI@Z` |
  | 18 | `?Cached@BinStream@@UBA_NXZ` |
  | 16 | `?DrawShowing@RndDrawable@@UAAXXZ` |
  | 15 | `??_GFile@@UAAPAXI@Z` |
  | 15 | `?ClassName@FlowNode@@UBA?AVSymbol@@XZ` |
  | 13 | `?ClassName@Object@Hmx@@UBA?AVSymbol@@XZ` |
  | 12 | `?WriteDone@File@@UAA_NAAH@Z` |
  | 12 | `??_GNode@?$ObjPtrList@VFlowNode@@VObjectDir@@@@UAAPAXI@Z` |

  The most-affected TUs are `Movie.cpp` (17), `ContentMgr.cpp` (12),
  `DirLoader.cpp` (10), `Shader.cpp` / `Text.cpp` / `UIListDir.cpp` (9).

The shape — broad in TUs, tiny in names per TU, dominated by a short head of
shared header virtuals — is the shape of a **systematic construct**, not of an
accident. It is also the shape that a single corrected clause fixes everywhere
at once.

---

## 5. Clause or replacement — one sentence

**A clause.** §2's Vtable rule is already right and its Propagation clause is
wrong only about one edge kind, so replacing "a call anywhere in the
pre-optimization body" with "a call anywhere in the pre-optimization body,
except that a *virtual dispatch* ODR-uses the vtable **slot** and not the
definition" repairs the whole class at 0.37 % of the predicted set — and the
`67`-vs-`26` byte in the `.ex` makes that clause **directly implementable**, so
R3 does not even need the fail-closed source-side guard axes1 proposed.

---

## 6. What this measurement did NOT measure — named, so absence never reads as success

1. **§2's actual predicted-Emit set, and therefore V3 itself.** No fitted root
   model exists; `TP` and the non-virtual-slot part of `FP` are both unmeasured.
   §2 gives a *ceiling* attributable to this class, nothing more. **A V3 number
   cannot be quoted from this file.**
2. **The 21 quarantined TUs.** Detector-side candidates only (668 instances over
   19 TUs); the ≈ 15 truth-validated instances is an extrapolation from the
   2.3 % candidate→class rate, not a measurement, and stays that way until the
   prereg's one-shot gate is scored.
3. **Cascade false positives** (§3e) — over-predictions whose only referrer is
   itself over-predicted. Unmeasured; can only increase the figure.
4. **Class members reachable only through an unnamed segment** (§3d) — 33.7 % of
   segments have no owner. Unmeasured; can only increase the figure.
5. **Virtual-dispatch encodings outside the 12-cell gate set** — covariant-return
   thunks, virtual inheritance, `novtable`, dispatch inside EH funclets.
   Unmeasured; can only increase the figure.
6. **axes1's second violation (the `/Yu` PCH root class)** — out of scope here;
   axes1 already showed `.gl` presence guards it at zero cost.
7. **Whether the 5 `soundtouch` + 2 `FxSend*360` TUs that do not build at
   `9ad5c4c8` would contribute.** Excluded from every denominator; unmeasured.
8. **Any comparison against the prereg's frozen rev `51fb5b73`.** Everything
   here is at `9ad5c4c8` and must be re-derived before being compared.

---

## 7. Reproducing

    cd work/emitpred/magnitude
    python3 capture_all.py  $ILROOT    tus.txt       24   # 876/878, ~18 s, c1xx only
    python3 truth_all.py    $TRUTHROOT truthlist.txt 24   # 850/857, ~21 s, RUNS c2
    python3 gate.py                                       # 12/12 known-answer gate
    python3 detect.py       $ILROOT $TRUTHROOT truthlist.txt class.jsonl 24
    python3 attrib.py       $ILROOT truthlist.txt 200     # folding-rule grade
    python3 coverage.py                                   # E ⊆ U
    python3 slotcheck.py    tus.txt truthlist.txt         # slot hygiene + candidates
    python3 validate.py     $ILROOT $TRUTHROOT truthlist.txt --sample 30

`$ILROOT`/`$TRUTHROOT` are gitignored scratch outside the tree (they hold IL and
objs, which are never committed). `truth_all.py` is the only script that runs
c2; it is only ever pointed at `truthlist.txt`, which excludes the quarantine.
`coff.py`, `gl.py`, `il.py`, `model.py` under `work/emitpred/pipeline/` are used
unmodified — `detect.py` inlines `read_token_var` for speed but is
semantics-identical, and `gate.py` re-checks that on 12 cells.
