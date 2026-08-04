# w-afail — the factor-A failure census: **one mechanism, 843 of 843 TUs**

    Lane:      w-afail, 2026-08-04, worktree `wt-w-afail` off master `d72432c`
    Prereg:    rungs/_2026-08-04-w-afail-prereg.md, committed at `4b08932`
               BEFORE any measurement. Scored in §7.
    Ships:     three additive read-only keys in `crates/c2-harness/src/gap.rs`
               (`be18b78`). No fixture. No codegen. **This is a measurement.**
    Status:    FINDINGS. Not a completed rung — nothing was widened and TU
               match is 8 at both ends.

**One-line answer:** *Factor A fails on **843 of 871** graded TUs for **one**
mechanism — c2 emits a `.text` COMDAT for only **7.15 %** of the function bodies
the front end hands it (2,501,606 `.ex` segments → 178,975 COMDATs, **14.0×**) —
and **no single bucket of that mechanism, closed alone, converts one TU**;
closing all of it moves `A∧B∧C` from **25** to **107**, which is `|B∧C|`, so A's
entire remaining capacity is **82 TUs** and it is indivisible.*

---

## 0. Provenance — every number on this page

| | |
|---|---|
| c2-rs branch | `wt-w-afail`, based on master **`d72432c`** ("merge wt-w-cfgimpl") |
| c2-rs HEAD at scan time | **`4b08932`** + the uncommitted instrument (`c2rs_dirty = True`; landed as `be18b78`) |
| harness binary sha | `5b2a98380030e86d39f0e592e785576c` |
| **dc3-decomp HEAD BEFORE the run** | **`940d07dcb0960964ad61aa5f025658f993eb46b2`** |
| **dc3-decomp HEAD AFTER the run** | **`940d07dcb0960964ad61aa5f025658f993eb46b2`** — **it did not move**, and `workload_dirty = False` |
| toolchain | X360 `16.00.11886.00` `cl.exe`/`c1xx.dll`/`c2.dll` under `wibo 1.0.1-7-g3b0f71c-dirty` (`wibo_stale = True` against known-good `1.0.1-23`, unchanged from every other lane today) |
| flags | the unmodified workload line, `work/dc3-workload/flags.txt` |
| TU list | `work/dc3-workload/files.txt`, **878** entries; **871 graded**, 7 capture-fail |
| capture cache | root `work/capture-cache`, context `35cdbafd…` — **6 hits, 872 misses**: the run **re-captured the whole workload** (my `--cwd` was the absolute path, a different cache context from the incumbent lanes' `../dc3-decomp`). Every number here is from a fresh capture at the HEAD above, not from a cached one. |
| raw scan | `work/w-afail/scan-afail.jsonl` (879 lines: 1 provenance + 878 TU rows), log `scan-afail.log`, analyses `afail.py` / `afail2.py` (stdlib only) |

**The corpus did not move under this run.** That is the first thing checked and
the reason every incumbent below reproduced exactly.

---

## 1. What factor A is, restated as an integer

`A ⇔ n == c`, where per TU

| | |
|---|---|
| `n` | `.ex` function segments on the **gate** splitter `4F 1F` (`IlBundle::ex_segment_count`) — what `PortC2::build` consumes |
| `c` | `.text*` COMDAT leaders in the reference obj (`emit-emitted`) |
| `t` | `fn_total`, the census's `LO`-anchored (`4C 4F 11`) segment count |

So A's failure is a **signed integer per TU**, and the whole census is a
decomposition of that integer. The exact identity, with `join-residual` measured
at **0 on all 871 TUs**:

```
n − c  =  (n − t)               splitter
        + afail-row-not-emitted   IL body, named, no COMDAT  ] c2 was handed the
        + afail-row-unnamed       IL body, the .gl binding   ] body and did not
                                  had no name for it        ] emit it
        − emit-residue-generated  COMDAT no row claims, ??_G/??_E/??__E-shaped
        − emit-residue-unbound    COMDAT no row claims, anything else
```

The last three keys already existed. **The middle two are this lane's
instrument**: every pre-existing `emit-*` key walks `emitted` and asks which row
claims each COMDAT, so the *surplus* direction — an IL body with no COMDAT — was
invisible to all of them, because it is not a member of `emitted`.

---

## 2. The census

### 2.1 Direction: it is one-sided, absolutely

| | TUs | share of the 843 |
|---|---:|---:|
| **surplus** — `n > c`, the IL carries more bodies than c2 emits | **843** | **100.0 %** |
| deficit — `n < c`, c2 emits a COMDAT with no `.ex` behind it | **0** | 0.0 % |

`|n − c|`: min **2**, p25 **1,309**, median **1,998**, p75 **3,168**, p90
**7,233**, max **9,429**. **832 of 843** are off by more than **100**.

There is no "c2 synthesizes COMDATs the port cannot see" population at the TU
level. Synthesis exists (`emit-residue-generated`, 1,961 symbols) but it is
**0.08 %** of the surplus and never dominates a single TU.

### 2.2 Buckets, on the frozen key

`(dir, dominant)` where `dominant` is the largest `|contribution|`. **10 buckets
were possible; 3 are occupied.**

| TUs | share | bucket |
|---:|---:|---|
| **435** | 51.6 % | surplus / **rows-unnamed** |
| **407** | 48.3 % | surplus / **rows-not-emitted** |
| 1 | 0.1 % | surplus / splitter |

Two buckets cover **99.9 %**.

### 2.3 …and the two top buckets are **the same mechanism**, provably

`rows-unnamed` looks like an instrument limit — a census row the `.gl` binding
could not name. **The obj bounds how much of it can be one.** An unnamed row can
only be a body c2 *did* emit if some emitted COMDAT is unclaimed, and the number
of unclaimed COMDATs is measured directly:

| | over the 843 A-failing TUs |
|---|---:|
| unnamed census rows | **988,672** |
| unclaimed COMDATs (`residue-generated + residue-unbound`) — the cap | **9,221** |
| ⇒ unnamed rows that can possibly be emitted bodies | **≤ 0.93 %** |
| ⇒ unnamed rows that are provably **discarded bodies** | **≥ 99.07 %** |

The binding's coverage of the **emitted** side is **94.85 %** (169,672 of
178,893 COMDATs claimed by exactly one row). The 42 % of the surplus called
`rows-unnamed` is unnamed rows on the **non-emitted** side — which is exactly
where a `.gl` name is least likely to exist, and where its absence is *itself*
weak evidence of non-emission.

**The exact surplus decomposition, summed:**

| term | count | share of `Σ(n−c)` = 2,322,631 |
|---|---:|---:|
| `rows-not-emitted` (named, provably discarded) | 1,304,970 | 56.2 % |
| `rows-unnamed` (≥ 99.07 % also discarded) | 988,672 | 42.6 % |
| `splitter` | 38,210 | 1.6 % |
| unclaimed COMDATs | −9,221 | −0.4 % |
| **one-mechanism lower bound** | | **≥ 98.35 %** |

**c2 provably discarded at least one IL body on 842 of the 843 A-failing TUs.**

> **The answer to "is it one mechanism, three, or three hundred": ONE.**
> ≥ 98.35 % of the surplus, and 842/843 TUs, are "the front end handed c2 a
> function body and c2 did not emit a COMDAT for it".

### 2.4 The discarded bodies, by mangling class

Reported because a residue named only by a number is a rumour — and because if
the surplus were concentrated in special-member classes the story would be
"synthesis", not "selection".

| class | discarded bodies | share | on TUs |
|---|---:|---:|---:|
| `ordinary` (`?…`) | 565,869 | 43.4 % | 833 |
| `dtor` (`??1…`) | 151,213 | 11.6 % | 829 |
| `template-operator` (`??$…`) | 143,483 | 11.0 % | 827 |
| `operator` (`??…`) | 141,313 | 10.8 % | 829 |
| `ctor` (`??0…`) | 123,605 | 9.5 % | 829 |
| `special-generated` (`??_…`) | 97,819 | 7.5 % | 830 |
| `undecorated` | 81,668 | 6.3 % | 742 |

**Spread across every class, on ~830 of 843 TUs each.** There is no
special-member sub-population to attack; this is ordinary user and header code
being dropped wholesale.

### 2.5 What A actually selects for: **size**

| | `.ex` segments `n` |
|---|---|
| **A-true** (28 TUs) | min 0, p25 1, **median 1**, p75 3, **max 20** |
| **A-false** (843 TUs) | **min 3**, p25 1,419, **median 2,164**, p75 3,565, max 9,840 |

Every A-true TU has `n ≤ 20`. **25 of 28 have `n ≤ 5`.** Only **7** of the 843
A-failures have `n ≤ 20`, and they are named in §3.

The trivial predictor `n ≤ 20` has **recall 1.00** and **precision 0.80** for A
over the graded workload. **A is, empirically, a size filter.** The per-TU emit
rate `c/n` is min 0.0000, p10 0.0161, **median 0.0638**, p90 0.2023, max 1.0000;
aggregate **0.0715** — which independently reproduces the standing
"c2 emits ~7.2 % of IL bodies" figure from a different anchor.

That reframes what "every codegen lane works inside the 25 TUs where A holds"
means: those 25 are not a representative sample of small TUs, they are the TUs
that are **nearly empty**. Half of them define one function; six define none.

---

## 3. Is A's failure understood, or uncharacterized?

The brief demands these not share a bucket. They do not.

### Understood, modelled on paper, **not implemented** — ≥ 98.35 % of the surplus

The mechanism is exactly the subject of **`docs/PHASE7_PLAN.md` §2**, board
**#161**: *"emission is a least-fixpoint reachability from roots, computed over
kept definitions only, at ODR-use granularity, pre-optimization"*, with roots,
propagation and a vtable rule all stated, fitted black-box on **172 designed
cells with zero violations**, and with **eight rival hypotheses explicitly
refuted** by cells that would otherwise have gone red.

It is not implemented anywhere in `crates/`. `PortC2::build` emits one `.text`
COMDAT per `.ex` segment. **This lane's contribution is the magnitude of the job
that predicate has to do**: decide 2,501,606 segments down to 178,975 COMDATs,
i.e. reject **92.85 %** of what it is shown, on 871 real TUs — against a
predicate fitted on 172 synthetic cells whose own standing caveat is *"real
headers (STLport, templates over templates, `??_9` adjustor thunks, multiple
inheritance) are out of the grid."*

Its one measured out-of-sample defect is already on the board: **#161's
virtual-slot over-prediction**, 289 TUs / 649 name-instances / 331 distinct
names (`work/emitpred/MAGNITUDE.md`).

### Uncharacterized — ≤ 1.65 % of the surplus, and it is small

| | |
|---|---|
| `splitter` (`n − t`, gate sees more `4F 1F` segments than the census's `4C 4F 11`) | 38,210, **1.6 %**, nonzero on 633 TUs. Partly explained (§10.12's `??__E`/`??__F` bare-`4C`) and **dominant on exactly 1 TU**. |
| `residue-unbound` (COMDATs no row claims, not obviously generated) | 7,260, **0.3 %**, 86.3 % of it `ordinary`-mangled — i.e. mostly the **binding** losing a row, not c2 synthesizing. |
| `residue-generated` | 1,961, **0.08 %**. |
| the ≤ 0.93 % of `rows-unnamed` that could be emitted | ≤ 9,221 by the cap above. |

**No uncharacterized bucket dominates more than 1 of 843 TUs.**

---

## 4. The TU-weighted ranking — the part that matters

"If bucket X were closed" = **X's contribution to `n − c` removed, every other
term held**. A TU converts only if the remainder is exactly zero.

### 4.1 Single-bucket closure, on the frozen key

| bucket closed | A: 28 → | `A∧B∧C`: 25 → |
|---|---|---|
| `rows-unnamed` | **36** (+8) | 31 (+6) |
| `rows-not-emitted` | 28 (+0) | 25 (+0) |
| `splitter` | 28 (+0) | 25 (+0) |
| `comdats-generated` | 28 (+0) | 25 (+0) |
| `comdats-unbound` | 28 (+0) | 25 (+0) |

### 4.2 …and the one nonzero row is **spurious**. Corrected: every bucket is +0.

The 8 TUs that `rows-unnamed` closure converts are named, and **all eight have
`residue = 0`**:

| `n` | `c` | unnamed | B/C | src |
|---:|---:|---:|---|---|
| 3 | 1 | 2 | B– | `src/ChecksumData_xbox.cpp` |
| 13 | 2 | 11 | BC | `src/system/math/Rand2.cpp` |
| 11 | 3 | 8 | BC | `src/system/net/JsonMemory.cpp` |
| 13 | 7 | 6 | B– | `src/system/oggvorbis/VorbisMem.cpp` |
| 19 | 6 | 13 | BC | `src/system/os/CritSec.cpp` |
| 120 | 9 | 111 | BC | `src/system/synth_xbox/PitchCorrectedVoice.cpp` |
| 118 | 6 | 112 | BC | `…/soundtouch/source/SoundTouch/PeakFinder.cpp` |
| 3 | 1 | 2 | BC | `src/xdk/nuiapi/nuidetroit.cpp` |

`residue = 0` means **no COMDAT on those TUs is unclaimed**, so by §2.3's cap
**not one** of their unnamed rows can be an emitted body. Every one is a
discarded body. "Closing the instrument hole" on them is not a thing that can
happen; the +8 is an artifact of the closure model treating a measurement label
as a removable term.

> **Corrected P4: no single bucket, closed alone, converts a single TU to A.**
> Stated as bluntly as it deserves — **there is no lever inside A.**

### 4.3 Joint closure — and where it saturates

| closed | A: 28 → | `A∧B∧C`: 25 → |
|---|---:|---:|
| `rows-not-emitted` | 28 | 25 |
| + `rows-unnamed` | **242** | **97** |
| + `splitter` | **339** | **107** |
| + `comdats-unbound` | 390 | 107 |
| + `comdats-generated` (all five ⇒ A = 871) | **871** | **107** |

Two readings, both load-bearing:

1. **The two halves of the one mechanism only pay jointly.** Either alone: +0.
   Both: +214 on A. That is what "indivisible" means as a number.
2. **`A∧B∧C` saturates at 107 = `|B∧C|` after only three of the five buckets.**
   A perfect emit-set model — factor A true on all 871 TUs — takes `A∧B∧C` from
   **25 to 107**. The FRONTIER (`A∧B∧C` minus the 8 matches) goes **17 → 99**.

> **A's total remaining capacity is `107 − 25` = 82 TUs**, it requires the whole
> emit predicate, and no partial credit is available.

### 4.4 How many mechanisms are live at once

| simultaneously nonzero | TUs |
|---:|---:|
| 1 | 8 (the spurious set above) |
| 2 | 188 |
| 3 | 127 |
| 4 | 85 |
| **5** | **435** |

**Only 8 of 843 A-failing TUs have a single nonzero term** — and those 8 are the
artifact. On **435** every term is live at once.

---

## 5. C and B — one project or two?

### 5.1 A and C are **independent**, and C's work does not advance A

| set | size |
|---|---:|
| `A` | 28 |
| `C` | 114 |
| `A ∧ C` | **25** |
| `A \ C` | 3 |
| `C \ A` | **89** |

**25 of A's 28 are already inside C.** C's section-vocabulary ladder
(`.data` → `.rdata$r` → `.text$yd` → `.xdata$x`, `docs/OBJ_DATA_BSS_SHAPE.md`)
adds TUs to C, and **89 of C's 114 are A-failures already** — so C's remaining
work buys A nothing, and A's work buys C nothing. **Two projects.**

They do compose: `B∧C = 107` is the ceiling A saturates against in §4.3, so
extending C's vocabulary *raises the ceiling A is worth*. `.data` alone moves
C 114 → 169; `.rdata$r` (RTTI, not EH — §10.20) moves it to 590; the fourth step
reaches 871. That is the only coupling, and it is a ceiling coupling, not a
shared mechanism.

The **A-failure bucket ranking does differ** between the two populations by
label — C-true: `rows-unnamed` 67/89 (75 %); C-false: `rows-not-emitted` 385/754
(51 %) — but §2.3's cap says both labels are the same mechanism, so the
difference is one of `.gl` name coverage on small TUs, not of kind.

### 5.2 B

| | |
|---|---:|
| B-failing TUs | **533** |
| …with an emitted symbol having **no body record at all** (the `wall`) | **451 (84.6 %)** |
| …every emitted symbol at least **has** a body record (repairable in `bind.rs`) | 82 (15.4 %) |
| `A ∧ B` | 27 (`A \ B` = 1, `B \ A` = 311) |
| `B ∧ C` | 107 |

**B is mostly wall, not repair.** A perfect `bind.rs` moves B 338 → 420; the
remaining 451 need synthesis. And B is *not* nested with A either — one A-true
TU fails B (`src/system/synth_xbox/MeterEffect.cpp`, the only A-true TU with
`n ≠ t`, and the `26`-separator anomaly PHASE7_PLAN §2 already names).

---

## 6. The 28, by name — the population every codegen lane has been working in

`M` = graded match; the four letters are B, C, D, E.

```
    BC--  n=   1  src/Main.cpp                     BC--  n=   3  src/system/utl/Pool.cpp
    B---  n=  20  src/keygen_xbox.cpp            M BCD-  n=   2  src/system/utl/Spew.cpp
    B---  n=   1  src/system/math/Primes.cpp     M BC-E  n=   1  src/system/zlib/ZlibLicense.cpp
    BC--  n=   1  src/system/math/Sort.cpp         BC--  n=   1  src/xdk/LIBCMT/osfinfo.cpp
    BC--  n=   2  src/system/negate_test.cpp       BC--  n=   1  src/xdk/LIBCMT/undname.cpp
    BC--  n=   3  src/system/rndobj/wordwrap.cpp   BC--  n=   2  src/xdk/LIBCMT/vsnprnc.cpp
  M BC-E  n=   1  …/tomcrypt/TomCryptLicense.cpp   BC--  n=   1  src/xdk/LIBCMT/vswprnc.cpp
    BC--  n=   2  src/system/synth_xbox/Biquad.cpp BC--  n=  11  src/xdk/nuispeech/mmio.cpp
  M BCD-  n=   0  …/synth_xbox/GainEffect.cpp      BC--  n=   1  src/xdk/nuispeech/xboxheap.cpp
  M BCD-  n=   0  …/HeadsetPlaybackEffect.cpp      BC--  n=   4  src/xdk/nuispeech/xboxmem.cpp
    BC--  n=   4  …/IPP_basicmath_xbox.cpp         BC--  n=   1  src/xdk/xjson/jsonwriter.cpp
    ----  n=  13  …/synth_xbox/MeterEffect.cpp     BC--  n=   1  src/xdk/xlrc/xlrcimpl.cpp
  M BCD-  n=   0  …/synth_xbox/PeakDetector.cpp
  M BCD-  n=   0  …/soundtouch/…/mmx_optimized.cpp
  M BCD-  n=   0  …/soundtouch/…/sse_optimized.cpp
```

**Six of the 28 define zero functions.** Half define one. The largest defines 20.

---

## 7. Scoring the pre-registration

| | registered | measured | |
|---|---|---|---|
| **C0** | row partition == `fn_total`, 871/871, 0 breaks | **871 ok, 0 broken** (5 TUs write no key at all — those are the `fn_total = 0` TUs, so the identity holds as `0 == 0`) | **HIT** |
| **C1** | A recomputed in Python == the Rust's `emit-set-ceiling-gate` | **871 agree, 0 disagree**, 0 unknown-segment TUs | **HIT** |
| **C0′** (added) | `emit-emitted == bound + two-rows + residue×2` | 871 ok, 0 broken; `join-residual` **0 on every TU** | **HIT** |
| **C2** | A 28 / B 338 / C 114 / D 8 / E 2 / `A∧B∧C` 25 / graded 871 / match 8 / capture-fail 7 | **all nine exact**; also `A∧B∧C∧D` = 6, `A∧B∧C∧(D∨E)` = 8, `B∧C` = 107 | **HIT** — corpus did not move |
| **P1** | ≥ 95 % of A-failures are `n > c` | **100.0 %**, deficit **0** | **HIT** |
| **P2** | ≤ 4 buckets cover ≥ 90 %; **top bucket > 500 TUs** | 2 buckets cover **99.9 %** (3 occupied of 10); top bucket **435** | **SPLIT — first clause HIT, second REFUTED.** The concentration is *higher* than registered and the top bucket *smaller*, because the two top buckets are one mechanism split by an instrument label (§2.3). |
| **P3** | dominant bucket is `rows-not-emitted`, ≥ 400 of ~843 | `rows-not-emitted` covers **407 ≥ 400** but is **not** dominant: `rows-unnamed` **435**. Margin **28 TUs / 3.3 %**. | **MISS on the dominance clause; rival P3′ wins by label.** |
| **P4** | every bucket alone converts **< 60** TUs | max **+8**, and §4.2 shows that 8 is spurious ⇒ **+0** | **HIT, and stronger than registered** |
| **P4b** | ≤ 150 TUs have exactly one nonzero mechanism | **8** | **HIT** |
| **P4c** | top-3 closure moves `A∧B∧C` from 25 to **< 120** | **107** (= `|B∧C|`, the ceiling) | **HIT** |
| **P5** | `|A∧C| ∈ {25,26,27}`; same top bucket in C-true and C-false | `|A∧C|` = **25** HIT; top bucket **differs by label** (`rows-unnamed` 75 % vs `rows-not-emitted` 51 %) | **SPLIT.** §2.3's cap collapses the labels to one mechanism, so the *conclusion* (A and C are independent projects) survives on `|A∧C| = 25`, and the bucket clause is scored REFUTED. |
| **P6** | `|A∧B| ≥ 26`; ≥ 60 % of B-failures are `wall` | `|A∧B|` = **27**; wall **84.6 %** | **HIT** |

### 7.1 The decline clause fires, and here is what it cost

The prereg priced this exactly: *"If P3′ wins — `rows-unnamed` is the dominant
bucket — then after at most two further probes I decline to push the mechanism
census further and deliver a characterized boundary."*

**P3′ won on the registered reading.** The two probes were spent, and they did
not deliver "instrument-bounded":

1. **Cap the hole against the obj** (§2.3). ≤ 0.93 % of unnamed rows can be
   emitted bodies. The label is not a mechanism.
2. **Check the hole's other end** (§2.3). The binding names **94.85 %** of the
   emitted side. The unnamed rows are concentrated on the *non-emitted* side.

So the registered price — *"A's failure census is instrument-bounded; no
TU-weighted mechanism ranking from this lane is defensible; `bind.rs`'s name
coverage is the blocking prerequisite"* — **is not owed.** It was a real risk and
the two probes retired it with a bound, not with an argument.

**What is owed and is paid here:** the prereg's frozen bucket key put a
measurement label (`rows-unnamed`) and a compiler fact (`rows-not-emitted`) in
the same ranking, where the label out-ranked the fact by 28 TUs. Every §2.2
and §4.1 number is reported at the frozen key; §2.3, §4.2 and §4.3 are the
**corrected reading and are labelled post-hoc**. The frozen numbers are not
deleted, and the corrected ones do not silently replace them.

---

## 8. What these buckets do **not** predict

Registered in the prereg *before* the numbers existed, restated against them.
Board **#150**, fifth instance.

* **Bucket size does not predict work.** The largest bucket is 435 TUs and its
  closure, alone, converts **0**. The mechanism behind both top buckets is a
  fixpoint reachability analysis over pre-optimization IL — one project, and
  its size is not proportional to any count on this page.
* **Nothing here predicts TU yield.** A is necessary and **not sufficient**.
  Perfect A converts **0** TUs by itself; it moves `A∧B∧C` to 107 and the
  frontier to 99, and every one of those 99 still needs codegen breadth that
  w-pair and w-cfgimpl measured to be expensive (an instruction scheduler; an EH
  subsystem; five single-blocked-function TUs, **all framed**).
* **`n` does not rank TUs by difficulty.** §2.5 shows A is a size filter, which
  makes `n` a good *predictor of A* and a bad *ranking of work* — the
  `n = 3` near-miss (`nuidetroit.cpp`) is the TU w-pair measured as needing an
  instruction scheduler.
* **The mangling-class table (§2.4) is not a work queue.** Every class appears on
  ~830 of 843 TUs; no class is separable, and the emit predicate does not branch
  on mangling.
* **`residue = 0` is not evidence the binding is complete** — on those TUs it
  means every emitted COMDAT is claimed, which says nothing about the 99 % of
  rows that are not emitted.

---

## 9. Proposed board rows — **numbers NOT minted**

`BOARD.md` at `d72432c` reads *"Next free number: `#196`"* and its last row is
**195**. On 2026-08-04 **four** lanes proposed into that range concurrently
(w-pair 196–200, w-cfgimpl 196–200, w-repro from 201, this lane). I have
**minted nothing** and pinned no number in code — `gap.rs`'s new comment cites
this file, not a `#N`. Assign at merge.

| proposed | item | claim | where |
|---|---|---|---|
| **P-a** | **Factor A fails for ONE mechanism on 842 of 843 TUs** — c2 emits 7.15 % of the bodies it is handed (2,501,606 `.ex` → 178,975 COMDATs, 14.0×) | ≥ 98.35 % of the surplus is "body handed to c2, no COMDAT emitted"; ≤ 1.65 % is anything else; **deficit direction is empty (0 TUs)** | this file §2 |
| **P-b** | **There is no lever inside A** — every single bucket, closed alone, converts **0** TUs | the one apparent +8 is spurious: all 8 TUs have `residue = 0`, so none of their unnamed rows can be an emitted body | §4.1–§4.2 |
| **P-c** | **A's total remaining capacity is 82 TUs and is indivisible** — perfect emit-set modelling takes `A∧B∧C` 25 → **107 = `\|B∧C\|`**, FRONTIER 17 → **99** | joint closure saturates after three of five buckets | §4.3 |
| **P-d** | **A is empirically a size filter**: every A-true TU has `n ≤ 20`; `n ≤ 20` has recall 1.00 / precision 0.80 for A | A-true median `n` = **1**; A-false median **2,164**; 6 of the 28 define zero functions | §2.5, §6 |
| **P-e** | **A and C are independent projects** — `\|A∧C\| = 25` of A's 28, `\|C\A\| = 89` | C's section ladder raises the *ceiling* A saturates against (`B∧C`), and shares no mechanism with it | §5.1 |
| **P-f** | **`afail-row-*`: the row side of factor A is now a printed key** | the surplus direction was invisible to every pre-existing `emit-*` key; the three keys sum to `fn_total` on 871/871 | `gap.rs` 1e″ |
| **P-g** | **`rows-unnamed` must never be ranked as a mechanism** — it is a measurement label, and the obj caps it at ≤ 0.93 % | it out-ranked the real mechanism by 28 TUs on this lane's own frozen key, which is why the cap is published beside it | §2.3, §7.1 |

---

## 10. Reproducing every number here

```sh
# 1. the scan (879-line JSONL; ~872 fresh captures if your cache context differs)
c2rs gap --list work/dc3-workload/files.txt \
         --flags-file work/dc3-workload/flags.txt \
         --cwd ../dc3-decomp --jobs 16 --jsonl work/w-afail/scan-afail.jsonl

# 2. the registered census, controls and prediction scores
python3 work/w-afail/afail.py  work/w-afail/scan-afail.jsonl

# 3. the post-hoc bound, the size analysis and the near misses
python3 work/w-afail/afail2.py work/w-afail/scan-afail.jsonl
```

Both scripts are stdlib-only and read-only. `work/` is gitignored; the scripts
are preserved in worktree `wt-w-afail` for re-derivation, and every number above
is recomputable from the JSONL alone.

## 11. Gate

Incumbents held across the instrument (`be18b78`):

| | incumbent | this tree |
|---|---|---|
| `cargo test --workspace --release` | 687 passed, **0 failed**, 25 targets | **687 passed, 0 failed, 25 targets** |
| `cargo build --release` | 0 warnings | 0 warnings |
| `c2rs selftest` | 219 PASS | **219 PASS, 0 FAIL** |
| `scripts/gate.sh --jobs 6` | 12/12 PASS, 2,628 verdicts, 0 mismatch | **12/12 PASS, 2,628 verdicts, 0 mismatch** |
| TU match | 8 / 878 | **8 / 878** (§0 scan) |

*Compared on the FAILED count, never the passed count — a failing target aborts
the run and a smaller passed count then reads as a regression that is really a
truncation.*
