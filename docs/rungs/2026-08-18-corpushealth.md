# CORPUSHEALTH — the refusal population is NOT an artifact of an unfinished decompilation: the ceiling is **2.95 %**, and the port refuses **55.9 %** of the bodies c2 emits from source that is already byte-matched to the shipped game

    Tag:       CORPUSHEALTH
    Slug:      corpushealth
    Date:      2026-08-18
    Kind:      characterization — reads `../dc3-decomp`'s own progress signal
               and real c2's own obj symbol tables, and asks whether the port's
               refusals concentrate in source the decompilation has not
               finished. Read-only on the peer repo; no `crates/` change.
    Outcome:   built
    Fixtures:  none — characterization: is `vocab-gap 844` /
               `fnbyte-refused-parse 113,447` inflated by an immature corpus?
    Census:    +0 — `crates/`, `fixtures/` and `scripts/` are **byte-identical
               at both ends** (`git diff 071d2d47 -- crates fixtures scripts` is
               empty). Docs and `work/` only.
    Record:    this file; prereg `work/w-corpushealth/PREREG.md`, frozen at
               `2ecf319b`, this branch's **first** commit, before any predicted
               quantity was measured.

Commits: `2ecf319b` prereg · `925623cc` the three instruments and their output ·
this file.

---

## 0. The question, and the three readings it has to be split into

The project owner:

> *"It's possible that c2-rs fails on some of our bytes because they are
> invalid. Not sure if that is an issue or not but maybe worth looking into."*

The workload is not Dance Central 3's source. It is `../dc3-decomp`, an
in-progress decompilation moving fast enough that board **#3238** caught it
shifting **61 commits / 56 source files / +324 −260** between a `STATUS.md`
reading and a lane's re-read of the same figures. So the corpus is being
reconstructed while we grade against it, and the hypothesis is worth a lane:
**some fraction of the port's refusals may be properties of an immature corpus
rather than gaps in the port.**

"Invalid" has three readings and they have different answers.

**Reading 1 — invalid to c2. Impossible by construction on the 870 graded TUs,
and this lane spends nothing more on it.** Real `c2.dll` compiled every one of
them; their IL is valid c2 input *by definition*. The lane confirmed the
stronger version anyway, because it was nearly free: `c2rs gap --replay-every 1`
re-ran **all 870 captured bundles through standalone real c2** and the scan
prints `replay soundness: 870 checked, 0 diverged`
(`work/w-corpushealth/replay.log`). The one place c2 itself refuses is the **8
`capture-fail` TUs**, and §5 shows what they actually are.

**Reading 2 — semantically wrong but well-formed.** Placeholder bodies, stubs,
scaffolding, wrong types: compiles fine, produces IL shapes unrepresentative of
the shipped game. This is the real question and §2–§4 are about it.

**Reading 3 — malformed or truncated containers.** Tested directly, three ways,
in §6. Clean negative.

---

## 1. The enumeration rule (published, so the measurement is reproducible)

`dc3-decomp` carries its own progress signal in two artifacts. Both are read
**read-only**; the lane wrote nothing to the peer repo.

* **`build/373307D9/report.json`** — objdiff-cli `4.2.3`
  (commit `88b425bc3bad`), relocation mode **`functionRelocDiffs=name_check`**,
  read out of the report's own `provenance` block and not assumed. 2,224 units;
  **980 carry `metadata.source_path`**, and that string space is *exactly*
  `work/dc3-workload/files.txt`'s — which is what makes the join **exact rather
  than heuristic**. Per-function `match_percent_normalized` is the decomp's own
  canonical ruler (`docs/PROGRESS_METRICS.md`: authorable normalized **91.21 %**,
  29,383 / 32,213).
* **`decomp.db`** — `functions(symbol, unit, verdict, excluded, is_stub, …)`.

**FINISHED ≡ `match_percent_normalized == 100`.** Registered before measuring,
with its rationale: a function whose *reconstructed* source reproduces the
target's bytes up to register permutation is IL the original build demonstrably
could have produced, so a refusal on it is a **real port gap**. A refusal on a
function the decomp has *not* matched is **eligible** to be a corpus artifact —
eligible, not proven. The lane therefore measures an **upper bound** on the
hypothesis throughout, and says so at every number.

The c2-rs side is one `c2rs gap --jsonl` run; the per-TU `emit` map already
carries `fnbyte-refused-parse` and its denominator, so **no `crates/` change was
needed or made**.

### 1.1 The instrument the join needed, and why a per-TU `min()` is not enough

Joining per TU gives Fréchet bounds and nothing tighter, because the scan
publishes refusal *counts*, not names. That is not merely loose — it is
**wrong in a known direction**: objdiff attributes each linked function to one
unit, so an unfinished symbol emitted into fifty objs is counted against one
of them. Every STLport template instantiation is of that kind.

So the lane built the missing side. `work/w-corpushealth/symcensus.sh` compiles
**all 878 TUs** with real `c2.dll` under wibo at the workload's own flags,
`objsyms.py` reads each obj's `.text` COMDAT symbol names out of its COFF symbol
table, and the obj is deleted. Result: **870 objs, 189,371 emitted body
instances, 90,897 distinct mangled names** — the name space the decomp's
per-symbol verdict can be intersected with. `namespace.py` does the
intersection; `correlate.py` does the rate split.

Four buckets over the 189,371 instances:

| bucket | meaning | instances | share |
|---|---|---:|---:|
| **UNFINISHED** | name is in an authorable unit at normalized < 100 | **3,349** | 1.77 % |
| **FINISHED** | name is in an authorable unit at normalized == 100 | 51,173 | 27.02 % |
| VENDOR | name is in a non-authorable unit (xdk / Bink) | 58 | 0.03 % |
| **ABSENT** | name is in **no** objdiff unit at all | **134,791** | **71.18 %** |

Only **UNFINISHED** is eligible to be a corpus artifact under the hypothesis.

---

## 2. The answer

**Base re-read at `071d2d47` in this worktree, on a lane-private binary
(`f4a1701317115d88`), immediately before every comparison** — #3249's standing
instruction, and this lane's subject matter rather than a nuisance:

| | |
|---|---|
| `match` / `mismatch` / `codegen-gap` / `vocab-gap` / `capture-fail` | **26 / 0 / 0 / 844 / 8** |
| `fnbyte-exact` / `fnbyte-refused-parse` / `fnbyte-denominator` | **35,899 / 113,447 / 162,046** |
| capture cache | **870 hit, 8 miss, 0 uncacheable, 0 POISONED**, 0 refused on provenance |
| `dc3-decomp` head | **`ccd4c80362f1d947d694fe953d5d77a62caabe56` (clean)** |
| objdiff report | `4.2.3` / `88b425bc3bad` / `functionRelocDiffs=name_check` |

The scan's provenance block records `c2rs_head 2ecf319b` — the **prereg
commit**, which is docs-and-`work/` only. Its graded tree is byte-identical to
the base: `git ls-tree <rev> crates fixtures scripts | sha256sum` reads
`a8adae3aca8adba2…` at **`071d2d47`, at `2ecf319b` and at this branch's tip**,
so the reading is a reading of `071d2d47`'s compiler.

`fnbyte-exact` reads **35,899**, not `STATUS.md`'s 35,897 — the same **+2 / −2,
sum conserved at 149,346** that #3249 measured on an unchanged master and #3238
hit independently. Attributed to #3249, **not adjusted**, and it is 60× below
this lane's smallest headline.

### 2.1 The ceiling

> **Of the 113,447 refusals, at most 3,349 — 2.95 % — can be attributed to
> source the decompilation has not matched.** Tight Fréchet upper bound
> `Σ min(R_i, unfin_i)` = **3,313 (2.92 %)**; lower bound = **37 (0.03 %)**.

That ceiling is *absolute* on the gradeable population: it assumes every single
emitted body sitting on unmatched source is also refused. It is not an
estimate — it is the size of the population the hypothesis could possibly be
about.

The control says the ruler is measuring something: the same bound taken against
**matched** source is **51,173 (45.11 %)** of the refusals, `Σ min(R_i, fin_i)`
= 50,964. The refusals are overwhelmingly on source the decomp has already
proved correct.

### 2.2 The sharper statement — refusal does not concentrate in immature source

`correlate.py`, over the 844 `vocab-gap` TUs, split by the decomp's own
"complete unit" definition (**every** function in the unit at normalized 100 —
the 416/967 headline, *not* `metadata.complete`; see §7):

| group | TUs | emitted bodies | refused | **rate** |
|---|---:|---:|---:|---:|
| unit **FINISHED** (all fns norm == 100) | 310 | 33,006 | 18,448 | **55.89 %** |
| unit **not** finished | 529 | 156,092 | 94,831 | **60.75 %** |
| no objdiff unit at all | 5 | 226 | 165 | 73.01 % |

**Ratio not-finished / finished = 1.087.** Banded finer, there is a real but
tiny monotone gradient — 55.90 % → 58.68 % → 60.71 % → 63.73 % → 64.97 % as the
unit's unfinished fraction climbs through `0`, `<5 %`, `<15 %`, `<40 %`, `≥40 %`
— and the gradient is confounded with unit size, which the lane did not
disentangle because it does not need to. What matters is the **floor**:

> **In 310 TUs whose source is 100 % byte-matched to the shipped game, the port
> still refuses 18,448 of the 33,006 bodies c2 emits — 55.9 %.**
>
> Narrow it to the **200** of those TUs that emit *zero* unmatched-source bodies,
> where nothing whatever can be a corpus artifact under the ruler: **7,964
> refusals over 14,767 bodies, 53.9 %.**

**The rate is quoted on the wrong denominator on purpose, and the cross-check
says it does not matter.** `|S_i|` is every `.text` function symbol in the obj
(189,371 in total); `fnbyte-denominator` is the 162,046 the FBM instrument pairs
to an IL body, and it is the *smaller* one — so quoting `R_i / |S_i|` is the
**conservative** choice at every row. On the port's own denominator the same
three figures read **65.34 %** (finished units), **71.00 %** (not finished) and
**63.66 %** (the 200-TU clean set), with the whole workload at **70.01 %**. The
ratio is **1.087 on both denominators, to three places** — the group comparison
is invariant to the choice, and only the absolute level moves.

### 2.3 In the goal's units

If every unmatched-source body were free tomorrow:

* `fnbyte-refused-parse` **113,447 → 110,131** (−2.92 %).
* `vocab-gap` **844 → 844**. Exactly **four** of the 844 TUs have
  `R_i ≤ unfin_i` at all, and three of those four already have `R_i ≤ 2`:
  `src/link_glue.cpp` (11), `src/system/synth_xbox/FilterCoeffs.cpp` (2),
  `src/system/math/vec.cpp` (0), `src/system/os/NetworkSocket.cpp` (0).
* `match` **26 → 26**.

**A perfect decompilation converts zero TUs.**

---

## 3. The 71 % nobody can grade — published as a denominator, not folded in

**134,791 of the 189,371 emitted body instances (71.18 %) carry a name that is
in no objdiff unit at all.** These are `/Gy` COMDATs the linker never selected
anywhere: inline and template bodies that were inlined away or dead-stripped
before the shipped image existed. `??6DebugFailer@@QAAXPBD@Z` is emitted into
**732** of the 870 objs and appears in the final binary **zero** times;
`??$MakeString@…` into 528; `?Null@Symbol@@QBA_NXZ` into 526. 63.2 % of the
ABSENT instances come from names emitted into ≥ 2 TUs (shared header/template
code); 36.8 % from names emitted into exactly one.

**objdiff can never have a verdict on them** — it grades what is in the target
binary. `decomp.db` extends the coverage by only **1,217** names. So:

* the decomp's ruler grades **54,580 instances (28.8 %)**, and on that
  population **6.14 %** sit on unmatched source;
* extrapolating that rate to the whole emitted population gives
  **~11,620 / 113,447 = 10.24 %** as a *modelled* ceiling, next to the
  **2.95 %** that is actually measured.

Both numbers are printed. Neither is folded into the other, and the ungraded
71.2 % is stated as its own line — this repo has paid twice for the opposite
habit (`STATUS.md` trap 0; boards **#961**, **#1002**).

**The argument that the ungraded population is healthier than average, offered
as an argument and not as a measurement:** these bodies are header and template
code compiled by every TU that includes it, and its correctness is pinned by the
**29,383 functions that ARE byte-matched** and that inline it. If
`?Str@Symbol@@QBAPBDXZ` were wrong, its matched callers would not match. That
is a strong prior and it is not a number; §8 prices what turning it into one
would cost.

---

## 4. Two corpus-artifact populations that are real, named, and small

Both exist. Neither is material.

**`src/link_glue.cpp` — 800 lines of decomp scaffolding, in the workload.** Its
own header says it: *"provides ICF-merged function definitions that are missing
from split objects … also provides stub definitions for unresolved link symbols
from third-party libraries (libjpeg, zlib, vorbis, curl, etc.) and Xbox SDK
functions that are not part of the decomp scope."* The original build never had
this file. **44 of its 55 emitted bodies sit on unmatched source — the highest
ratio in the workload — and it costs 11 refusals of 113,447.** It is also the
biggest unit in `decomp.db` (1,275 rows), which is where the unattributable
symbols were parked.

**`decomp.db is_stub = 1`: 686 rows over 158 units**, of which **498 rows over
138 units** map into the 878-TU workload (`src/system/os/PlatformMgr_Xbox.cpp`
69, `src/system/synth_xbox/ExternalMic.cpp` 43, `Synth.cpp` 41, …). But only
**115 of the 686 distinct stub symbols are emitted anywhere in the workload at
all**, for **147 body instances of 189,371 — 0.078 %.** The decomp's stubs are
mostly functions c2 never emits into the objs we grade.

---

## 5. The 18 TUs that are not in the game, and the 8 the compiler refuses

**18 of the 878 workload TUs resolve to no objdiff unit** (H1: the join is
**860/878 = 97.95 %**, carrying **99.85 %** of the refusal mass). They are not a
join defect — **16 of the 18 are not referenced by `dc3-decomp`'s own
`build.ninja` at all.** The cluster is unmistakable:

`soundtouch/…/3dnow_win.cpp`, `cpu_detect_x86_win.cpp`, `cpu_detect_x86_gcc.cpp`,
`mmx_optimized.cpp`, `sse_optimized.cpp`, `SoundStretch/main.cpp`,
`RunParameters.cpp`, `SoundTouchDLL.cpp`, `WavFile.cpp`, `BPMDetect.cpp`,
`PeakFinder.cpp` — **x86/MMX/3DNow/SSE sources and a command-line tool**, from a
vendored library, in a PowerPC console build. Plus `decomp_pch.cpp` (a PCH),
`negate_test.cpp` (a scratch file), `Spew.cpp`, `ZlibLicense.cpp`,
`StreamReceiver360.cpp`, `FxSendPitchShift360.cpp`, `FxSendSynapse360.cpp`.

**The 8 `capture-fail` TUs are the same story, and they are the one place
reading (1) is live.** By name and by c2's own error code:

| TU | c2 error |
|---|---|
| `soundtouch/.../3dnow_win.cpp` | **C1189** (`#error`) |
| `soundtouch/.../cpu_detect_x86_win.cpp` | **C1189** |
| `soundtouch/.../SoundStretch/main.cpp` | **C1083** (cannot open include) |
| `soundtouch/.../SoundStretch/RunParameters.cpp` | **C1083** |
| `soundtouch/.../SoundTouchDLL/SoundTouchDLL.cpp` | **C1083** |
| `src/system/synth_xbox/FxSendPitchShift360.cpp` | **C2084** (function already has a body) |
| `src/system/synth_xbox/FxSendSynapse360.cpp` | **C2084** |
| `src/system/utl/BinkIntegration.cpp` | **C2065** (undeclared identifier) |

Seven of the eight are files the decomp never builds. `BinkIntegration.cpp` is
the only one referenced by `build.ninja`, and it still produces no objdiff unit.
**This is the hypothesis confirmed in its purest form and it is worth 8 TUs of
878 — 0.91 % — and 0 refusals**, because a TU that never captured contributes no
bodies to the denominator.

### 5.1 A finding about `match 26` that this join makes visible, and that is NOT about validity

Of the 26 matching TUs, only **11** are authorable game code
(`Main.cpp`, `Primes.cpp`, `Sort.cpp`, `TomCryptLicense.cpp`, `Biquad.cpp`,
`GainEffect.cpp`, `HeadsetPlaybackEffect.cpp`, `IPP_basicmath_xbox.cpp`,
`PeakDetector.cpp`, `EncryptXTEA.cpp`, `Pool.cpp`). **Nine** are `src/xdk/*` —
units with source in the repo that the decomp deliberately does **not** build
from source (`undname.cpp`, `osfinfo.cpp`, `vsnprnc.cpp`, `vswprnc.cpp`,
`mmio.cpp`, `xboxheap.cpp`, `xboxmem.cpp`, `jsonwriter.cpp`, `xlrcimpl.cpp`).
**Six** are in the unjoined set above (`decomp_pch.cpp`, `negate_test.cpp`,
`mmx_optimized.cpp`, `sse_optimized.cpp`, `Spew.cpp`, `ZlibLicense.cpp`).

**Every one of those 26 is a genuine byte-exact obj against real c2 and none of
this retracts a single match.** It is a statement about *representativeness*, not
validity: the payoff metric's numerator is 11/26 game code, and the frontier
lanes already know this from the other direction (`xboxheap.cpp` is an xdk TU).

---

## 6. Reading (3): are any captured containers actually malformed? **No** — three independent ways

1. **Replay, over the whole population.** `c2rs gap --replay-every 1` fed all
   **870** captured bundles back through standalone real `c2.dll` under wibo:
   `replay soundness: 870 checked, **0 diverged**`. No truncated `.ex` or `.gl`
   reproduces a byte-identical obj.
2. **Independent COFF structural read, over the whole population.**
   `objsyms.py` is a from-scratch reader that checks `Machine == 0x01F2`,
   `SizeOfOptionalHeader == 0`, every section's raw-data and relocation pointer
   in range, the symbol table in range, the string-table length in range, and
   every symbol name resolvable. Over **870 objs: 0 BAD**, 8 COMPILE-FAIL (the
   `capture-fail` set, expected). This is a *different program* from the port's
   reader, so it is not the port certifying itself.
3. **Fresh re-capture against the cached record, on a stratified sample.** 50
   TUs (10 largest `.ex` + 40 uniform, **seed 20260818**, list committed at
   `work/w-corpushealth/ilcheck.list`) re-captured from source and compared to
   the cached scan's own `ex_len`: **50/50 clean** — all five bundle files
   present, none empty, `.ex` byte count identical across **51,985,843 bytes**.

The scan's own structural read agrees: `gate-side segment count KNOWN for 870 of
870 captured TUs; UNKNOWN for 0`, and `section headers: 870 objs read, 0 did not
decode`.

**Clean negative, published as one.**

---

## 7. Two prereg corrections, and the green zero one of them produced

Both are recorded in the scripts, not applied silently.

* **`metadata.complete` is not a match measurement.** H2 was registered against
  it. It is objdiff's *"this unit is built from source in the final link"* flag:
  true for exactly the **968** units that carry a `source_path`, and
  `default/keygen_xbox` carries `complete: true` at `matched_functions 16/20`.
  The decomp's own headline *"Complete units (all fns norm == 100) 416/967"*
  uses the other definition. H2 is scored on `U_i == 0` and the
  `metadata.complete` split is printed beside it as the authorable/vendor axis
  it actually is.
* **`decomp.db`'s `functions.unit` schema comment is wrong.** It says
  `"src/system/char/Char.cpp"`; the column holds the objdiff **unit name**
  (`default/system/char/Char`). Joining it against source paths returned
  **`0 rows over 0 units`** — a green-looking zero from a join that could never
  have matched, which is `STATUS.md` trap 5 in one line, caught here only
  because the number was implausible. Re-joined through `report.json`'s
  unit-name → source_path map: **498 rows over 138 units**.

### 7.1 Prereg scorecard

| # | prediction | p | outcome |
|---|---|---:|---|
| H1 | ≥ 90 % of 878 TUs join | 0.75 | **HIT** — 860/878 = 97.95 %, 99.85 % of refusal mass |
| H2 | 45–70 % of refusal mass in non-finished units, point 57 % | 0.55 | **MISS** — **83.59 %**, far outside. The coarse unit-level bound is dominated by unit size and is worthless; this is exactly why H3 was registered as primary |
| **H3** | **tight upper bound < 5 %** | 0.80 | **HIT** — 2.39 % per-TU, **2.92 %** name-space |
| H3b | …point value < 3 % | 0.6 | **HIT** — 2.92 % / 2.95 % |
| H4 | ≥ 300 `vocab-gap` TUs in finished units, point 370, interval 300–450 | 0.55 | **HIT** — **310**, in interval, below point |
| H5 | zero malformed containers in a ≥ 40-TU sample | 0.85 | **HIT**, and over the whole 870, not a sample |
| H6 | `is_stub` < 500 rows over < 100 units | 0.5 | **MISS on both** — 686 rows / 158 units. Immaterial anyway: 147 emitted instances of 189,371 |
| H7 | the answer is a clean negative | 0.70 | **HIT** |
| H8 | ≥ 1 nameable corpus-artifact population found anyway | 0.8 | **HIT** — `link_glue.cpp`, the soundtouch x86 cluster, the 8 `capture-fail` |
| H9 | headline metrics move by 0, `fnbyte-*` by ≤ ±2 | 0.9 | **HIT** — see §9 |

**Eight hits, two misses, two definitions corrected** (§7 — neither was voided
under the probe-soundness rule; the control executed on every reading). **Both
misses are on the COARSE instruments**, and both would have been reported as
the lane's answer by a session that stopped at the per-TU join: H2 would have
published *"83.6 % of the refusal mass sits in unfinished units"*, which is
true, size-driven, and off by a factor of 28 from the number that answers the
question.

---

## 8. Found and not taken

Ranked. None taken; this is a characterization lane.

1. **The FBM denominator counts 134,791 bodies that are not in the shipped
   game — 71.2 % of it.** `fnbyte-denominator 162,046` and
   `fnbyte-refused-parse 113,447` are dominated by `/Gy` COMDATs the linker
   discards everywhere. This is **not** corpus immaturity — the original build
   had the identical property, and c2 emits them because `/Gy` says to. But it
   means FBM's ratio is taken over a population three and a half times larger
   than the code that ships, and a "shipped-image FBM" is a different number
   nobody has computed. The lane did **not** compute it, because
   `docs/FUNCTION_BYTE_MATCH.md` would have to decide whether it wants it, and
   because the port is judged on whole objs, where those bodies are real bytes
   c2 wrote and the port must reproduce. **Sized, not priced.**
2. **The 28.8 % / 71.2 % gradeable split is the honest bound on this lane's own
   answer.** 2.95 % is measured on the part the decomp's ruler can see; 10.24 %
   is the same rate extrapolated. Closing the gap needs a per-body source
   verdict for header and template code, which objdiff structurally cannot give
   (the bodies are not in the binary). The one route that exists is a
   *source-level* one — attribute each emitted COMDAT to its defining
   header and ask whether that header is matched anywhere — and it needs a
   symbol → source-location map the repo does not have. **Unpriced.**
3. **`match 26` is 11 authorable game TUs, 9 non-built `src/xdk` TUs and 6 TUs
   in no unit at all** (§5.1). If the payoff metric is meant to track *the
   game*, its numerator is 11 and not 26. This is a **definition** question for
   `docs/STATUS.md`, not a defect, and the lane deliberately did not answer it:
   every one of the 26 is a real byte-exact obj against real c2, and retiring
   15 of them would be a scoring change dressed as a finding.
4. **`src/link_glue.cpp` and the 16 never-built TUs could be dropped from the
   workload, and should not be without a two-sided price.** They cost 11 + ~165
   refusals and 0 conversions. Dropping them shrinks a denominator without
   moving the goal, which is the exact shape `CLAUDE.md` requires be priced
   both ways before it ships. **Named, not proposed.**
5. **`docs/STATE_OF_THE_DECOMP.md` is stale against its own database** —
   it publishes `COMPLETE 29,655 + AT_LIMIT 3,628` over `33,560` non-excluded
   rows; `decomp.db` at `ccd4c8036` reads **28,680 / 1,650 over 31,425**. That
   is a peer project's business, not this one's, and this lane wrote nothing to
   it. Recorded because a future lane joining against that doc rather than the
   DB will get a different answer.
6. **The `is_stub` classifier disagrees with the emitted set by 571 of 686.**
   Only 115 of the decomp's stub symbols are ever emitted into the objs we
   grade. Whether that is stub-detection over-reach or simply functions c2
   inlines is unmeasured.

---

## 9. Gate evidence

Recorded per `docs/rungs/README.md` § "Two rules a probe must satisfy". **The
control is pinned by NAME, not by count**: a fresh worktree has no `compilers/`
and every capture-based measurement then silently skips
(#3219/#3231). `scripts/setup_worktree.sh` provisioned this worktree and its
own toolchain check reported `OK: fixtures/cpp/w5_chain.cpp -> 4/4 functions in
class`; every scan below reports **`870 hit, 8 miss`** on the cache and the
**same 8 `capture-fail` TUs by name** (§5), and both scans took nonzero wall
time (**9.5 s** and **23.7 s**) rather than the 0.00 s an unprovisioned
environment produces. Any reading taken without that control would have been
**void, not provisional**.

| lane | result |
|---|---|
| `cargo test --workspace --release` | **1,660 passed · 0 failed · 1 ignored · 43 targets** — the registered baseline exactly. **`SKIP: toolchain absent` appears 0 times** and the run took **3 m 52 s**, which is the executed-count-and-duration assertion #3219 requires in place of an exit code |
| `scripts/gate.sh --jobs 4 --require-graded` | **GATE: PASS** — 18/18 lanes PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT, **6,948 fixture-verdicts**; sweep **19,460 of 19,556 graded, 0 mismatch**; cross **90,424 of 90,812 cells graded, 0 mismatch**; `debug-lane` **18/18, 6,948 verdicts, 2,423 match, 0 mismatch, 0 PANIC**. Run **twice** — `gate.log` (tree mid-edit) and `gate2.log` (clean, committed) — with **byte-identical counts and the same `graded tree 5b550a38d90b`**. Both carry `HATCH-RED REFUSED`, which is board **#2511**'s standing master outage and not this branch — §9.1 |
| 878-TU workload scan, **base** | `match 26 / mismatch 0 / codegen-gap 0 / vocab-gap 844 / capture-fail 8`; `fnbyte-exact 35,899`, `fnbyte-refused-parse 113,447`; cache `870 hit, 8 miss, 0 uncacheable, 0 POISONED`, 0 refused on provenance; 9.5 s |
| 878-TU workload scan, **`--replay-every 1`** | identical classes; **`replay soundness: 870 checked, 0 diverged`**; 23.7 s |
| 878-TU workload scan, **end** | identical classes and cache line; **scan identity `396 / 396` `gap-metric` keys byte-identical to the base, values included — 0 deltas, INCLUDING the whole `fnbyte-*` family.** #3249's ±2 did **not** fire on this bracket, so nothing had to be attributed |
| `crates` / `fixtures` / `scripts` diff vs `071d2d47` | **empty**. `git ls-tree <rev> crates fixtures scripts \| sha256sum` = `a8adae3aca8adba2…` at `071d2d47`, at the prereg commit and at the tip; the gate's own content hash reads **`5b550a38d90b` over 738 files** at both runs |
| `scripts/board_audit.sh` | **all five checks 0** — cited-but-not-on-the-board 0, unresolved section anchors 0, raw line anchors 0, rows-behind-the-prose 0, duplicate row numbers 0 |
| `rung_registry`, `scripts/gen_rung_index.sh` | inside the 1,660 — the new rung's header parses, its slug matches its filename, and `INDEX.md` equals what the generator produces (regenerated in the same commit) |

### 9.1 Two things about the gate that are worth stating rather than eliding

* **The key count is 396, not the 394 the dispatch registered.** The lane did
  not chase the difference: the identity that matters is **base vs end at the
  same tip**, and that is `396/396` with **0** deltas. A brief's handed-down
  count is exactly the kind of figure #3249 says to re-read rather than trust.
* **`GATE: PASS (HATCH-RED REFUSED)`, and the first explanation for it was
  WRONG.** Run 1 happened while this file was mid-edit, so the obvious reading
  was the gate's own advice — *"commit or stash `crates/` and re-run"*. The
  lane re-ran it on a **committed, clean tree** (`gate2.log`) and
  **`hatch-red` REFUSED again, identically: `HATCH-STALE`, 0 of 14 arms, 0
  green controls.** The tree was never the cause. This is board **#2511**,
  already on the board: `hatch-red` refuses at master with `HATCH-STALE`, and
  that lane reproduced it by checking out an older `crates fixtures` — *"the
  whole instrument declining to hatch the tree at all"*, a standing outage, not
  a property of any branch. This lane's `crates/` diff against `071d2d47` is
  empty, so the row would refuse identically at the base.

  Recorded rather than elided, with board **#1406**'s caveat attached: a run
  with `hatch-red` refused **does not establish what a full run establishes**.
  Both logs are kept — the same rule `w-fence163` applied to its invalid
  mutation run (#3226): **keep the unqualified log too.** The 17 rows that did
  run are unaffected and are the evidence above.
