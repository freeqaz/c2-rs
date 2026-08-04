# w-joint — it is NOT a fixpoint. The defined-data set is CLOSED (50 897/50 897), so the model is a FILTER — and given that filter as an oracle the code half is essentially SOLVED: F1 0.97888, five false positives in 167 213.

    Lane:      w-joint, 2026-08-04, worktree `wt-w-joint` off master `78d29e6`
    Prereg:    work/w-joint/PREREG.md (= rungs/_2026-08-04-w-joint-prereg.md),
               committed at `61c9735` BEFORE any corpus-wide measurement.
               Scored in §7.  §9 of the prereg discloses the 5-TU pilot.
    Ships:     NOTHING under `crates/`.  No fixture, no codegen, no widening,
               no DISCLOSURE.md row.
    Status:    FINDINGS.  TU match is 8 at both ends.

**The correction goes first, because it refutes my own §1 and the brief that
scoped this lane.** ***The emit set is NOT a joint DATA+CODE fixpoint on this
channel.*** The dd-edge — a data symbol's initializer naming another data symbol
— **fires 50 897 times from a defined owner across 850 TUs, and in 50 897 of
those 50 897 the target is itself defined.** The defined-data set is *absorbing*
under the initializer relation. So iterating the data half adds **nothing**
(`|live| == |Rd|` on **850/850 TUs**, at three different `Rd` sizes), and the
model is extensionally a **one-shot FILTER over an independently-determined data
emit set** followed by w-refs' existing code closure. It is a fixpoint only in
the sense that the iteration converges in one step.

**And with that filter supplied as an ORACLE the code half is essentially
solved.** Over the same 850 TUs and 174 417 emitted names:

> ### **precision 0.99997 — FIVE false positives in 167 213 predictions — recall 0.95867, F1 0.97888, per-TU exact 151/850.**
> **+12.63 pp of F1 and +19 TUs of exact over w-refs' 0.85260 / 132, with ZERO
> TUs lost: the ORACLE-exact set is a strict superset of the incumbent's.**
> With `#152` excluded, **F1 0.99025 and recall 0.98072.**

**But no static rule predicts the filter, so decline clause 1 FIRED.** The best
of the twelve pre-registered `.gl`-only `Rd` variants is `TAG_01` at **0.80985**
— **4.3 pp BELOW the incumbent** — and the best *including* the degenerate
`Rd = {}` is the incumbent itself, unchanged. The model half of this page is
published as a **refuted hypothesis**. **The one-shot Part-1 gate is NOT spent**
(§9), for w-skip's reason: a held-out set cannot improve a refutation.

**The result is a RELOCATION.** Phase 7's emit-set problem is no longer "what
are the roots of the code closure". It is exactly one question:

> ### **which DATA symbols does c2 define?** Answer that and 0.97888 is available; the code half needs nothing further.

---

## 0. Provenance — every number on this page

| | |
|---|---|
| c2-rs branch | `wt-w-joint`, based on master **`78d29e6`** (the merge of `wt-w-skip`) |
| c2-rs HEAD at the prereg | **`61c9735`**, clean — **no `crates/` change exists in this lane** |
| **dc3-decomp HEAD BEFORE** | **`940d07dcb0960964ad61aa5f025658f993eb46b2`** |
| **dc3-decomp HEAD AFTER** | **`940d07dcb096…`** — **it did not move** (`work/w-joint/prov_{before,after}.txt`) |
| wibo | **`1.0.1-23-g4a9dd6f`**, checked at lane start, **not stale** (`work/w-joint/wibo.txt`) |
| c2.dll | `compilers/X360/16.00.11886.00/c2.dll`, image base `0x10b00000` |
| IL + obj | the harness's **capture cache**, filtered to the workload argv **and** to `tree 940d07dc…+clean` — §1a. **850 of 857**, the 7 misses w-emit's 7 by name |
| truth `E` | w-emit's `truth/`, unchanged, 174 417 names — reproduced independently here on **850/850** (KA-AGREE) |
| truth `D` | **NEW, this lane**: 685 848 defined symbols / 232 156 defined DATA symbols |
| flags | the unmodified workload line, `work/dc3-workload/flags.txt` |
| scratch | `work/w-joint/` (gitignored); scripts and text outputs force-added, no IL or obj committed |

**The 21-TU quarantine is intact and w-emitpred's one-shot Part-1 gate is
UNSPENT** — §9. Every mutation TU was checked against `heldout.txt` **by the
script, before writing anything** (`mutate_joint.check_quarantine`).

---

## 1. THE DELIVERABLE — the extended truth capture, and its own grade

w-skip §8 item 4 named this lane's first task: *the corpus-wide owner-emitted
filter needs a truth capture that records **defined data symbols**;
`work/w-emit/truth` holds **code COMDAT leaders only**.*

### 1a. Where it comes from — and what that costs

Four lanes re-ran `cl` for IL or truth. **The harness's capture cache was
already holding, per key, the whole `_CL_*` quintet — `gl ex in db sy` — AND
`out.obj`.** 850 of the 857 workload TUs are in it. This lane reads the cache
instead of running the toolchain, which is why the corpus scan takes **56
seconds** rather than an hour.

That shortcut is only sound if the entries are the right ones, and two filters
do that work (`work/w-joint/cacheindex.py`):

* **`key.bin`'s `tree <rev>+clean` is the dc3 WORKLOAD rev**, and the cache
  holds entries at **at least 18 different ones** — `fbf097a5`, `9ad5c4c8`,
  `940d07dc`, `173eb73b+DIRTY`, … . `src/App.cpp` alone has **30** entries.
  An entry at another rev is a different corpus. `940d07dc…+clean` is required
  exactly; `+DIRTY` is never accepted. **35 883 entries are rejected for the
  tree and 8 710 for being dirty.**
* the argv signature must be the workload's (53 418 rejected), so a fixture or
  a `gate.sh` lane entry can never be picked up for a workload TU.

The surviving duplicates are **not** byte-identical — the obj embeds its own
`-Fo` path in `S_OBJNAME` — which is why KA-DUP compares the *classification*
and why the extractor works by **name** and never by file offset.

*Cache hygiene, since this repo has been OOM-killed twice by it: the indexer
does exactly one `os.scandir` at depth 1 and then opens only
`<entry>/meta.txt` by explicit path. No `*/`, no recursion, no `du`.*

### 1b. What it records

`work/w-joint/objsyms.py` classifies **every symbol-table entity** of the obj
into eleven buckets by section **characteristic** (never by a `.text` name
prefix — this project has twice been burned by name-as-proxy), and publishes:

| set | definition | corpus total |
|---|---|---:|
| `E` | COMDAT leaders of `IMAGE_SCN_CNT_CODE` sections | **174 417** |
| **`D_all`** | **every symbol with a real section number** — deliberately the **widest** reading, because it is the one `w-skip/mutate_owner.py` used for its 10/10 vs 0/10, and a narrower one here would let a decoder's blind spot look like a filter | **685 848** |
| `D_data` | those in a non-code section | **232 156** |
| `D_lead` | COMDAT leaders of non-code sections | 205 808 |
| `undef` | `secnum == 0, value == 0` | — |

The workload's **13 section names** fall out of the same read, reproducing
`STATUS.md`'s independently derived **count of 13** exactly: `.XBLD$W` 850,
`.debug$S` 850, `.drectve` 850, `.text` 842, `.pdata` 828, `.rdata` 826,
`.data` 736, `.bss` 675, `.rdata$r` 662, `.text$yd` 238, `.CRT$XCU` 124,
`.text$yc` 124, `.xdata$x` 66.  **The per-name obj counts are lower than
`c2rs gap`'s by a constant denominator, not by disagreement**: gap reads **871**
objs and this lane grades **850** (the 21-TU quarantine is excluded here and is
not excluded there).  Every name and every ordering agrees.

### 1c. THE GRADE — because the oracle cannot grade a correspondence

The compiler judges obj bytes; it cannot tell you whether census row *R* is
symbol *S*. So the instrument is graded on invariants of its own, and every
failure mode prints a **count and names**:

| invariant | registered | **measured** | |
|---|---|---|---|
| **AGREE** — my code-COMDAT leaders **==** w-emit's independently captured `truth/<slug>.txt` | 850/850, ≥ 845 | **850 / 850**, 0 disagree | **GREEN** |
| **TOT** — every entity in exactly one bucket, residue printed by name | 0, ≤ 500 | **0 entities over 850 TUs**; **0 unclaimed COMDAT sections** | **GREEN** |
| **AR-A1** — `sum(1 + naux) == NumberOfSymbols` | 0 TUs fail | **0**; corpus **1 534 428 records = 1 147 426 entities + 387 002 aux** | **GREEN** |
| **AR-A2** — `aux == nsym − entities` | 0 TUs fail | **0** | **GREEN** |
| **AR-A3** — every long name resolves in the string table | 0 TUs fail | **0**; **22 035 337 long-name bytes** counted | **GREEN** |
| **INJ** — a defined name defines one entity | 0 TUs, ≤ 40 | **116 TUs, 338 conflicting definitions** | **RED — §1d** |
| **KA-DUP** — two cache entries for one TU classify identically | ≥ 38/40 | **40 / 40** | **GREEN** |
| **KA-IL** — the cache's `gl` == w-emit's `gl`, byte for byte | 850/850, ≥ 845 | **849 / 850** | **AMBER — §1e** |
| **KA-POS** — the run GRADED something | > 0 | **850/850 TUs carry a defined data symbol** | **GREEN** |

**Could AGREE have gone red in the most likely failure mode? Yes.** The likely
failure is that the *capture* obj (`-il -typedil`, a different `-Fo`) is not the
obj a plain `cl` run makes. AGREE compares against 850 objs from a **separate
plain `cl` run in another lane at another wibo**, so a capture-path artifact
lands on it directly. It did not.

**And arity is not decoration.** TOT is satisfied exactly by moving an entity
between buckets (STATUS trap 4); a reader that mis-walks one aux record leaves
TOT silent at residue 0 and takes A1 red on the TU. A1 counts *records*, TOT
counts *entities*, and they are published as different numbers for that reason.

### 1d. INJ is RED, and the residue is characterised rather than counted

**116 TUs, 338 conflicting definitions — a MISS above the registered
interval [0, 40].** Characterised (`work/w-joint/dupcheck.py --inj`):

* **every conflicting name is a `$LN<n>` local label** (201 distinct names) —
  compiler-minted switch/jump-table labels that c2 defines once per function
  COMDAT, so `$LN12` really is defined several times in one obj;
* **zero of them is an `in` initializer OWNER.** The intersection is exactly 0.

So name→symbol is genuinely **not injective in this corpus**, the instrument is
right to say so, and the non-injectivity **cannot reach any number in this
lane**, because nothing here looks a `$LN` name up. Reported as a red invariant
with its reason, not repaired to make the table green.

### 1e. KA-IL — one TU, two bytes, and the sensitivity was measured

`src/system/hamobj/FreestyleMoveRecorder.cpp` differs from w-emit's
independently captured `gl` in **2 bytes of 397 295**, at offset `381488`:
`… 86 05 00 04 04 [40 47|24 43] c5 11 04 95 5f 00 "D3DCubeTexture_GetCubeMapSurface"`.
The two captures use different front-end argv (w-emit adds `/Bd /d2nop`), which
is the likely source.

**Measured rather than argued**: the TU was re-scored with w-emit's `gl`
substituted into the same bundle. `n_U`, `n_E`, `n_owner`, `n_PRGL`,
`n_E_in_PRGL`, `owner_in_D` and **every field of the `ORACLE`, `ALL` and `NONE`
variants** are identical. The difference is **inert for every number on this
page**, and it is recorded anyway.

---

## 2. THE MEASUREMENT — 850 TUs, 174 417 emitted names, one variable changed

`work/w-joint/scan.py` swaps only the root set and recomputes all three
incumbents in the same pass. Edges, `Seed`, the name binding, the truth reader
and the closure operator are w-roots'/w-refs'/w-skip's as landed.

### 2.1 KA-A — the incumbents reproduce to the digit

| | recorded | **this pass** |
|---|---|---|
| `\|U\|` / `\|E\|` / `\|E ∩ U\|` / `\|Seed\|` | 1 506 586 / 174 417 / 173 907 / 14 662 | **EXACT, all four** |
| `RGL` `\|P\|` / prec / rec / F1 / exact | 129 604 / 1.00000 / 0.74307 / 0.85260 / 132 | **EXACT, all five** |
| `INIT` | 613 532 / 0.27289 / 0.95991 / 0.42496 / 34 | **EXACT** |
| `SKIP` | 400 998 / 0.36420 / 0.83732 / 0.50761 / 34 | **EXACT** |

### 2.2 The table

| `Rd` | `\|Rd\|` | `\|P\|` | precision | recall | **F1** | **exact** |
|---|---:|---:|---:|---:|---:|---:|
| **`ORACLE`** *(a CEILING, not a model)* | 40 947 | 167 213 | **0.99997** | **0.95867** | **0.97888** | **151** |
| `ORACLE_LOOSE` | 41 797 | 167 436 | 0.99987 | 0.95985 | **0.97946** | **160** |
| `ORACLE_DATA` (`D_data` instead of `D_all`) | 40 947 | 167 213 | 0.99997 | 0.95867 | 0.97888 | 151 |
| `NONE` — the floor | 0 | 129 604 | 1.00000 | 0.74307 | 0.85260 | 132 |
| `TAG_01` | 587 199 | 147 151 | 0.88488 | 0.74655 | **0.80985** | 35 |
| `F20_2000` | 84 311 | 167 364 | 0.81639 | 0.78338 | 0.79955 | 124 |
| `F20_1000` | 553 300 | 447 773 | 0.32876 | 0.84400 | 0.47319 | 35 |
| `F20_4000` | 684 343 | 534 035 | 0.29597 | 0.90620 | 0.44620 | 34 |
| `TAG_02` | 115 063 | 595 762 | 0.27966 | 0.95525 | 0.43266 | 34 |
| `ALL` (= w-mark, as the degenerate case) | 702 262 | 613 309 | 0.27265 | 0.95873 | 0.42456 | 34 |
| `F20_60_20` | 702 262 | 613 309 | 0.27265 | 0.95873 | 0.42456 | 34 |
| `F20_400` / `F20_480` | 432 137 | 400 775 | 0.36388 | 0.83613 | 0.50709 | 34 |
| `F20_80` / `SC_STATIC` | 0 | 129 604 | 1.00000 | 0.74307 | 0.85260 | 132 |

> ### **The ORACLE makes FIVE false positives in 167 213 predictions over 850 TUs**, and its 151 exact TUs are a **strict superset** of the incumbent's 132 — **+19, with 0 lost**.

> ### **No static `Rd` reaches the incumbent.** Best non-degenerate: `TAG_01` **0.80985**, i.e. **−4.28 pp**. The registered wash bar was 0.87260. **Decline clause 1 FIRED.**

`F20_80` and `SC_STATIC` select **zero** owners on the whole corpus — printed
rather than dropped, because a rule that selects nothing must be visible as
such and not silently indistinguishable from the floor.

### 2.3 The owner accounting, including the circularity check

| | |
|---|---:|
| distinct `in` owners | 702 262 |
| **owners ∩ `E`** — the circularity check | **0** |
| **owners ∩ `U`** | **0** |
| owners ∩ `D` (the ORACLE's roots) | **40 947 = 0.05831** |
| `in` records whose owner token this decoder cannot name | 177 115 = **0.20141** |
| `02` nodes whose target token it cannot name | 617 = **0.00033** |

**No owner is an emitted function and no owner has a function record**, so
conditioning on `D` is not smuggling `E` back in through the owner set. That
was registered as M6 at a point of 0 and it is exactly 0.

`ORACLE_LOOSE` — every unnameable owner contributed unfiltered — moves F1 by
**+0.00058**. The 20 % blind spot is not doing the work.

---

## 3. THE CORRECTION — it is not a fixpoint, and the check that says so is POSITIVE

The brief scoped this lane as a *joint DATA+CODE fixpoint*, and my own prereg
§1 argued the same from the disassembly. **The measurement refutes it.**

    |live(Rd)| == |Rd|   on 850/850 TUs, at Rd = 40 947, 41 797 and 702 262

Iterating the data half adds **nothing**. Read alone that is exactly what a
**broken** dd-edge would print — STATUS trap 5, absence reading as success — so
it gets a positive check (`work/w-joint/ddcheck.py`):

| | |
|---|---:|
| owner → target pairs | 1 835 682 |
| dd pairs (the target is itself an owner) | 965 118 |
| **dd pairs FROM a DEFINED owner** — if this were 0 the claim would be vacuous | **50 897** |
| …of which **the target is also DEFINED** | **50 897 = 1.00000** |

> ### **The dd-edge fires 50 897 times and lands inside `D` all 50 897 times. `D` is ABSORBING under the initializer relation.**

Two consequences, and both are corrections to landed text:

1. **The emit model on this channel is a FILTER, not a fixpoint.** `Rd = D ∩ owners`, one pass of `d → f` marks, then w-refs' existing code closure. w-skip's *"the model has to be a joint DATA+CODE fixpoint"* — and the brief that inherited it — is **too strong**: the two sorts are real and the dd-edge is real, but the data sort needs no iteration because it is already closed. w-skip's own §5 sentence *"a model must carry data symbols as first-class members of the fixpoint"* survives; *"fixpoint"* does not.
2. **The registered discriminating arm M13 cannot be run, and the reason is the finding.** M13 needed an owner that is undefined **and** dd-reachable from a defined one. That population is **empty on all 850 TUs** — 0 candidates on each of the three mutation TUs — *because* of the closure above. Reported as **UNDECIDABLE with a measured cause**, never as a pass.

---

## 4. KA-MUT — the SOLE JUDGE, on three TUs w-skip did not use

`work/w-joint/mutate_joint.py` reproduces w-skip's retarget (point one `02`
node's `varU` at a function c2 was not going to emit; byte-length preserving by
construction) and splits by whether the owner is a defined symbol in the
**baseline replayed obj**. The baseline reproduces the pipeline obj's leader set
on all three.

| TU | baseline leaders | defined syms | owners | **H+ owner IS defined** | **H− owner NOT defined, NOT dd** | **DD** |
|---|---:|---:|---:|---:|---:|---:|
| `src/system/rnddx9/Movie.cpp` | 155 | 594 | 561 | **5/5 APPEARS** | **0/5** | 0 candidates |
| `src/system/gesture/NavigationSkeletonDir.cpp` | 132 | — | 883 | **5/5 APPEARS** | **0/5** | 0 candidates |
| `src/system/synth/StreamNull.cpp` | 155 | 392 | 302 | **5/5 APPEARS** | **0/5** | 0 candidates |
| | | | | **15/15** | **0/15** | — |

> ### **w-skip's 10/10 vs 0/10 REPLICATES on three new TUs: 15/15 against 0/15. Pooled across both lanes, 25/25 against 0/25.**

**And the flag word occurs in both arms again**, which is what makes it a
refutation of a flag-based reading rather than a correlation:
`??_R0?AVRndAnimatable@@@8` (`+0x20 = 0x1c01`, **defined**, pulls its target in)
against `??_R0?AVexception@std@@@8` (`+0x20 = 0x1c01`, **not defined**, does
not) — on the same TU, same mutation shape, opposite outcome. `??_R0?AVStream@@@8`
carries `0x1501` and is H+; `??_R0?AVrange_error@stlpmtx_std@@@8` carries
`0x2601` and is H−.

**Could H− have gone red in the most likely failure mode?** Yes — under
w-mark's unfiltered reading H− must come back green-as-APPEARS and match H+.
It did not, 0/15. **Could H+ have gone red?** Yes — if the retarget were
carried by something other than the owner's own definition, H+ would be as inert
as H−. Some retargets pull whole subtrees (`??_7StreamNull@@6B@` gains **31**
COMDATs), which is correct closure behaviour.

**H− is TIGHTER than w-skip's.** An owner is H− here only if it is neither
defined **nor** dd-reachable from a defined owner. w-skip's H− did not make that
split; §3 shows the distinction is empty on this corpus, so the two arms coincide
— but the arm was constructed to be able to differ.

---

## 5. The coincidence calibration decline clause 2 demands

Uniform expectation over the part of `U` the incumbent does not predict:
`(174 417 − 129 604) / (1 506 586 − 129 604)` = **0.03254**. Base rate
`|E|/|U|` = **0.11577**.

| | new marks over `P_RGL` | of which emitted | measured | **ratio** | root soundness |
|---|---:|---:|---:|---:|---:|
| **`ORACLE`** | 31 457 | **31 456** | **0.99997** | **30.73×** | 0.99997 = **8.64× base** |
| `ORACLE_LOOSE` | 31 519 | 31 502 | 0.99946 | 30.71× | 8.63× base |
| `ALL` (w-mark) | 242 056 | 31 456 | 0.12995 | 3.99× | 1.22× base |
| `F20_2000` (best static by precision) | 28 507 | 5 728 | 0.20093 | 6.17× | 1.99× base |
| w-skip's filtered roots | — | — | 0.08739 | 2.69× | **0.82× base — below chance** |
| w-emit's disqualified loose scan | — | — | 0.0277 | 1.07× | — |

> ### **31 456 of the 31 457 names the owner filter adds are emitted.** 30.73× the uniform expectation, against w-mark's 4.00× and w-skip's 2.69×. This is not distinguishable from chance in the direction that matters — it is 7.7× further from chance than the best previous channel.

`ALL` and `ORACLE` add **the same 31 456 emitted names**. The filter removes
**210 600 false marks and loses one true one.**

---

## 6. Stratified, so `#152` cannot dominate either direction

`??_G`/`??_E` deleting destructors are **5 344 = 3.064 %** of `E`. Removed from
both `E` and `P`:

| model | `\|P\|` | precision | recall | **F1** |
|---|---:|---:|---:|---:|
| `RGL` | 128 781 | 1.00000 | 0.76169 | 0.86473 |
| **`ORACLE`** | 165 818 | **0.99997** | **0.98072** | **0.99025** |
| `ORACLE_LOOSE` | 166 040 | 0.99987 | 0.98194 | 0.99082 |
| `ALL` | 611 914 | 0.27099 | 0.98078 | 0.42465 |

**Every conclusion is unchanged with `#152` excluded** — the ordering, the sign
and the magnitude of every gap survive. And the ceiling's residual is now
**dominated** by that class:

| `E ∩ U` the ORACLE still misses (6 699 names) | n | % |
|---|---:|---:|
| **`??_G`/`??_E` deleting dtor (`#152`)** | **3 949** | **58.95 %** |
| `$` in the qualified name | 971 | 14.49 % |
| non-virtual member | 825 | 12.32 % |
| VIRTUAL member | 452 | 6.75 % |
| static member | 368 | 5.49 % |
| free / file-scope function | 109 | 1.63 % |
| everything else | 25 | 0.37 % |

**This sharpens w-mark's R-f.** Under a *perfect* data oracle, `#152` is not
merely the largest remaining class — it is **the majority of what is left**, and
it is unreachable by construction: those symbols are synthesized by c2 and named
by no `02` node. **A perfect answer to "which data symbols are defined" leaves
`#152` standing and stops at recall 0.95867.**

Root floor (comparability only — clause 8 forbids it as a key): `|Rfloor|`
**36 141**, w-refs' figure to the digit; `Seed` coverage **0.18796**, w-refs'
figure to the digit; `Seed ∪ ORACLE` marks **0.86799**.

---

## 7. Scoring the pre-registration — 15 hits, 3 misses, 1 pass, 1 undecidable

| # | registered **point** | interval | **measured** | |
|---|---|---|---|---|
| **T1** | AGREE **850/850** | [845, 850] | **850 / 850** | **HIT**, at the point |
| **T2** | TOT residue **0** | [0, 500] | **0** | **HIT**, at the point |
| **T3** | AR A1/A2/A3 **0/0/0** | [0,5] each | **0 / 0 / 0** | **HIT** |
| **T4** | INJ **0** TUs | [0, 40] | **116 TUs, 338 names** | **MISS above the interval** — §1d |
| **T5** | KA-DUP **40/40** | ≥ 38/40 | **40 / 40** | **HIT** |
| **T6** | KA-IL **850/850** | [845, 850] | **849 / 850** | **HIT** inside, point missed by 1 — §1e |
| **T7** | `\|D_data\|` **260 000** | [80 000, 600 000] | **232 156** | **HIT**, below |
| **T8** | `\|D_all\|` **620 000** | [250 000, 1 200 000] | **685 848** | **HIT**, above |
| **M1** | ORACLE precision **0.999** | [0.950, 1.000] | **0.99997** | **HIT**, above |
| **M2** | ORACLE recall **0.930** | [0.800, 0.980] | **0.95867** | **HIT**, above |
| **M3** | **ORACLE F1 0.955** | [0.870, 0.990] | **0.97888** | **HIT**, above the point |
| **M4** | ORACLE per-TU exact **0.30** | [0.12, 0.70] | **0.17765** (151/850) | **HIT**, below |
| **M5** | owner-emitted fraction **0.020** | [0.005, 0.200] | **0.05831** | **HIT** |
| **M6** | `\|owners ∩ E\|` **0** | [0, 500] | **0** | **HIT**, at the point |
| **M7** | **best static `Rd` F1 0.66** | [0.35, 0.92] | **0.80985** (`TAG_01`) | **HIT**, above — and **below the 0.87260 wash bar, so clause 1 fires** |
| **M8** | LOOSE − STRICT **+0.002** | [−0.010, +0.050] | **+0.00058** | **HIT** |
| **M9** | coincidence **6.0×** | [1.5×, 12×] | **30.73×** | **MISS above the interval** |
| **M10** | stratified F1 **0.965** | [0.880, 0.995] | **0.99025** | **HIT**, above |
| **M11** | owner-unbound **0.28** | [0.05, 0.50] | **0.20141** | **HIT** |
| **M12** | H+ ≥ 4/5, H− ≤ 1/5 per TU | — | **15/15 and 0/15** | **PASS**, at the ceiling |
| **M13** | dd-edge arm ≥ 3/5 | — | **0 candidates on 850 TUs** | **UNDECIDABLE — §3** |
| **M14** | `NONE` == `P_RGL` **850/850** | [850, 850] | **850 / 850** | **HIT**, at the point |

**The declared bias was that M13 was the one I most expected to be wrong about,
and I was wrong about it in a way I did not anticipate: not refuted, but
VACUOUS — and the vacuity is the lane's most important finding.** I registered
it at ≥ 3/5 because my §1 said a data symbol reached from an emitted data symbol
is itself live. That is *true* (50 897/50 897) and it is exactly why the arm has
no population: nothing is ever *reached-but-not-already-defined*. I was right
about the edge and wrong about what it implies for the model's shape.

Second declared: **M7**, that every static `Rd` would fail. It did — best
0.80985 against a 0.87260 bar — and I registered it *low* at 0.66, so the miss
is in the direction that costs me nothing to admit and the conclusion stands
either way.

**Three misses, and none is in the direction of the model working better than I
said.** T4 is an instrument invariant going red and being reported red. M9 is
the effect being **five times stronger** than I registered. T6 is off by one TU
and measured inert.

### 7.1 The decline clauses — one fired, one triggered as a correction, all honoured

* **Clause 1 (best MODEL F1 < 0.87260) FIRED.** Honoured: the model half is
  published as a refuted hypothesis, the headline says so, and **I did not go
  looking for a further channel after the number arrived.** `db`, `sy`,
  `0x10b3389b`, `0x10b9aa26` and node kind `0x14` are **named in §8 and left
  undecoded.** Not one was pursued.
* **Clause 4 (M13 refutes the dd-edge ⇒ correct §1 above the headline)
  TRIGGERED in substance and honoured literally.** The arm was not refuted, it
  was empty; the correction it exists to force is nevertheless owed, and it is
  the **first paragraph of this page**.
* **Clause 2 (M1 < 0.95) NOT triggered** — 0.99997. The calibration is
  published anyway, in w-mark's exact shape (§5).
* **Clause 3 (M12 fails) NOT triggered** — 15/15 and 0/15.
* **Clause 5 (T1 < 845) NOT triggered** — 850/850.
* **Clause 6 (no instrument tuning after truth) HONOURED, with two
  disclosures.** (i) `ORACLE_LOOSE` was added **before** the prereg commit and
  is registered as M8, disclosed in prereg §9.3. (ii) After the first corpus
  scan and **before any of its numbers were read**, four counters were added to
  the scan (`n_live`, `n_mark_in_E`, `n_mark_new`, `n_mark_new_in_E`) and the
  `Rfloor` block, because clause-2's calibration needs them; the scan was re-run
  from scratch and no scored definition changed. `joint.py`, `objsyms.py`, the
  `Rd` enumeration and the truth reader are byte-identical to the prereg commit.
* **Clause 7 (nothing ships) HONOURED.** No `crates/` change; `PortC2` still
  returns `NotImplemented` outside its class.
* **Clause 8 (`Rfloor` is not a key) HONOURED** — §6, reported for
  comparability.
* **Clause 9 (`ORACLE` is never quoted as a model) HONOURED** — it is labelled
  a ceiling in the title, the headline, §2.2, §7 and §9.

### 7.2 Registered before the numbers existed, restated against them

* **TU match stays 8.** It did — 8 at both ends (§10).
* **`census/gate disagreement` stays 0.** It did.
* **A high ceiling is not a shippable predicate**, and 0.97888 costs an
  instrument nobody has built.
* **Order is untouched.** A right set in the wrong order is still a mismatch.

---

## 8. What this lane did NOT measure — named, so absence never reads as success

1. **Where a data symbol's definition comes from.** `ORACLE` conditions on it
   and explains **nothing**. **This is now the whole problem** and it is not
   touched here.
2. **`db`.** The capture cache holds it, per TU, for all 850 — and **no lane has
   ever read it**. It is the obvious instrument for item 1 and it is
   **deliberately not decoded**, under clause 1.
3. **`sy`.** Same, still unread.
4. **`0x10b3389b`** (`dag.c`, edges added during codegen) and **`0x10b9aa26`**
   (the by-name intern, roots added during codegen). w-skip named both as the
   source of the ordering requirement; neither is modelled. **§3's "no iteration
   needed" is a claim about the INITIALIZER channel only** — it says nothing
   about those two.
5. **`#152`.** 58.95 % of the ceiling's residual and unreachable by any
   initializer model.
6. **Node kind `0x14`.** Only the stream's `0x02` byte kind is decoded.
7. **The 617 unnameable `02` target tokens** and the **177 115 unnameable owner
   tokens.** Both are resolved in **both** directions (STRICT/LOOSE) and differ
   by 0.00058 of F1; they are not characterised further.
8. **Order.** A right set in the wrong order is still a mismatch.
9. **The 21 quarantined TUs.** Untouched (§9).
10. **Whether `D` is predictable at all.** The lane hands this on; it does not
    answer it, and nothing here bounds how hard it is.

---

## 9. The one-shot Part-1 gate — NOT spent, as pre-registered

The 21-TU quarantine is intact and w-emitpred's Part-1 gate is **still runnable
exactly once**, six lanes running.

The brief said the gate belongs to this lane because the joint model is the
first with **fitted parameters**. It does have them — the twelve `Rd` variants
of prereg §1a — and **they are refuted in sample, on 850 TUs**: the best is
0.80985 against an incumbent 0.85260. **A held-out set cannot improve a
refutation**, which is w-skip's reason and it is the right one here too.

**The high number on this page is an ORACLE, which is the other reason not to
spend it.** `ORACLE` has no parameters to overfit — it reads `D(t)` out of the
obj — so a held-out population cannot tell you anything about it that the 850
in-sample TUs do not. Held-out validation earns its keep against *fitting*, and
there is nothing here that was fitted and survived.

**The registered reversal condition did not trigger, and I checked it honestly.**
No definition in `joint.py`, `objsyms.py` or the `Rd` enumeration was chosen by
looking at `E` or `D`: the masks are transcribed from named instructions in
w-skip §1a/§1b, the section selection is by COFF characteristic, `D_all` is the
widest available reading, and after M3 came in at 0.979 I changed nothing.
§7.1 clause 6 discloses the two changes that were made and when.

> **The gate is still owed by whoever first ships a model that PREDICTS `D`.**
> That model will have fitted parameters, it will be validated against 850 TUs
> that this lane has now made gradeable, and the 21 held-out TUs are exactly
> what will catch it fitting. **Do not spend the gate before that model exists.**

---

## 10. Gate — every incumbent reproduced, on a tree with no `crates/` change

| | incumbent — master `78d29e6`, **re-measured**, not transcribed | **this tree** |
|---|---|---|
| `cargo test --workspace --release` | **698 passed, 0 FAILED, 1 ignored, 25 targets** | **698 passed, 0 FAILED, 1 ignored, 25 targets** |
| `cargo build --release` | 0 warnings | **0 warnings** |
| `c2rs selftest` | 222 PASS, 0 FAIL | **222 PASS, 0 FAIL** |
| `scripts/gate.sh --jobs 6` | 12/12 PASS, 2 664 verdicts | **12/12 PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT, 2 664 fixture-verdicts** |
| TU match / mismatch / codegen-gap / vocab-gap / capture-fail | 8 / 0 / 0 / 863 / 7 | **8 / 0 / 0 / 863 / 7** |
| A / B / C / D / E | 28 / 338 / 114 / 8 / 2 | **28 / 338 / 114 / 8 / 2** |
| `A∧B∧C` / `A∧B∧C∧(D∨E)` / `B∧C` | 25 / 8 / 107 | **25 / 8 / 107** |
| FRONTIER | 17 | **17** |
| **`census/gate disagreement`** | **0** | **0** |
| capture cache | 871 hit, 7 miss, 0 POISONED | **871 hit, 7 miss, 0 POISONED** |

**`git diff 78d29e6 -- crates/ scripts/` is EMPTY**, so the incumbent column and
this column are the same tree measured once; both were run here rather than
copied from a rung doc. Several rung docs still quote **687 / 219 / 2 628** and
w-skip quotes **690 / 222 / 2 664** — master has grown fixtures under all of
them, and the numbers above are this session's.

*Compared on the **FAILED** count and the **target** count, never the passed
count — a failing target aborts the run, so a lower passed count reads as green.*

---

## 11. Proposed board rows — **numbers NOT minted**

Same discipline as w-roots, w-emit, w-refs, w-mark and w-skip: **no number
minted, no `#N` pinned in code, `BOARD.md` / `ROADMAP.md` / `rungs/INDEX.md`
untouched by hand** (w-book2 owns the board). w-skip left `T-a…T-h` unminted;
this lane uses **`U-`**.

| proposed | item | claim | where |
|---|---|---|---|
| **U-a** | **The emit set is NOT a joint fixpoint on the initializer channel — the DEFINED-DATA set is ABSORBING.** The dd-edge fires **50 897** times from a defined owner over 850 TUs and the target is itself defined **50 897/50 897 = 1.00000**, so `\|live\| == \|Rd\|` on **850/850** at three different `Rd` sizes. The model is a **one-shot FILTER** over an independently determined data emit set, followed by w-refs' code closure | **CORRECTS w-skip T-d and the lane brief that inherited it.** The two sorts and the dd-edge are both real; the *iteration* is not needed. Graded by a positive check, because `residue 0` alone is what a broken edge prints | this file §3 |
| **U-b** | **Given the owner filter as an ORACLE the code half is essentially SOLVED: precision 0.99997 — FIVE false positives in 167 213 — recall 0.95867, F1 0.97888, per-TU exact 151/850**, against w-refs' 1.00000 / 0.74307 / 0.85260 / 132, **with zero TUs lost**. With `#152` excluded, **0.99025** | 850 TUs, 174 417 emitted names, one variable changed, all three incumbents reproduced to the digit in the same pass | §2 |
| **U-c** | **NO static `.gl` rule predicts the filter.** Twelve pre-registered `Rd` variants; the best non-degenerate is `TAG_01` at **0.80985**, **4.28 pp BELOW the incumbent**, and two of the twelve select zero owners corpus-wide | registered decline clause 1 fired; the model half is published as a refuted hypothesis and no further channel was pursued | §2.2 |
| **U-d** | **Phase 7's emit-set problem RELOCATES to one question: which DATA symbols does c2 define?** Answer it and 0.97888 is available; the code half needs nothing further. The `db` sub-stream — held by the capture cache for all 850 TUs and **read by no lane** — is the named next instrument | the quantitative form of U-b and U-c together | §2, §8 |
| **U-e** | **THE EXTENDED TRUTH CAPTURE — defined DATA symbols for 850 TUs, graded on its own invariants**: AGREE **850/850** against w-emit's independently captured truth, TOT residue **0**, arity **1 534 428 records = 1 147 426 entities + 387 002 aux** with A1/A2/A3 all 0, KA-DUP **40/40**. `\|D_all\|` **685 848**, `\|D_data\|` **232 156** | w-skip §8 item 4, delivered. The oracle cannot grade a correspondence, so the instrument is graded on injectivity, totality-with-a-named-residue, **arity** and agreement where the oracle already ruled | §1 |
| **U-f** | **INJECTIVITY IS RED AND THE REASON IS BENIGN: 116 TUs define a name twice, 338 definitions, ALL of them `$LN<n>` local labels, and ZERO of them is an `in` OWNER** | registered at a point of 0 with interval [0,40] and missed above it; characterised rather than repaired, because a residue is not the thing it is a proxy for | §1d |
| **U-g** | **w-skip's owner split REPLICATES: 15/15 against 0/15 on three TUs it did not use — 25/25 against 0/25 pooled** — with `+0x20` occurring in both arms again (`0x1c01` defined *and* undefined on one TU) | real `c2.dll`, byte-length-preserving retarget, H− tightened to exclude dd-reachable owners; both arms could have gone red | §4 |
| **U-h** | **The owner filter's roots are 30.73× the uniform expectation and 8.64× the base rate — 31 456 of 31 457 added names are emitted** — against w-mark's 4.00×/1.22× and w-skip's 2.69×/**0.82× (below chance)**. The filter removes **210 600** false marks and loses **one** true one | published in w-mark's exact calibration shape so the three lanes are comparable | §5 |
| **U-i** | **Under a perfect data oracle `#152` becomes the MAJORITY of the residual — 3 949 of 6 699 = 58.95 %** — and caps recall at 0.95867 | **sharpens w-mark R-f**: those symbols are synthesized by c2 and named by no `02` node, so no initializer model of any shape reaches them | §6 |
| **U-j** | **The harness's capture cache already holds the whole `_CL_*` quintet AND `out.obj` for 850 workload TUs** — but entries are keyed on the *worktree's* identity, so one source has up to **30** of them spanning **≥ 18 dc3 revs**; a `tree <rev>+clean` filter plus an argv-signature filter is required, and duplicates are never byte-identical because the obj embeds its own `-Fo` path in `S_OBJNAME` | four lanes re-ran `cl` for data the cache was holding; the corpus scan is 56 s with it. KA-DUP **40/40** and KA-IL **849/850** (the one, two bytes, measured inert) are the price of the shortcut | §1a, §1e |

---

## 12. Reproducing every number here

```sh
# 0. index the capture cache -> 850 TUs at the pinned dc3 rev  (no toolchain)
python3 work/w-joint/cacheindex.py <main-repo>/work/capture-cache \
        work/emitpred/magnitude/truthlist.txt work/w-joint/cacheidx.tsv

# 1. THE EXTENDED TRUTH + its four invariants                  (no toolchain)
python3 work/w-joint/truth_data.py work/w-joint/cacheidx.tsv work/w-joint/dtruth \
        <main-repo>/work/w-emit/truth 12
python3 work/w-joint/dupcheck.py --inj work/w-joint/dtruth work/w-joint/cacheidx.tsv
python3 work/w-joint/dupcheck.py <main-repo>/work/capture-cache \
        940d07dcb0960964ad61aa5f025658f993eb46b2 \
        work/emitpred/magnitude/truthlist.txt 40      # KA-DUP

# 2. the headline scan and the scores                          (no toolchain)
python3 work/w-joint/scan.py work/w-joint/cacheidx.tsv work/w-joint/dtruth \
        <main-repo>/work/w-emit/truth <main-repo>/work/w-emit/il \
        work/w-joint/scan.jsonl 16
python3 work/w-joint/score.py work/w-joint/scan.jsonl     # -> score.txt

# 3. the dd-edge POSITIVE check — §3                           (no toolchain)
python3 work/w-joint/ddcheck.py work/w-joint/cacheidx.tsv work/w-joint/dtruth 12

# 4. KA-MUT — RUNS real c2.dll under wibo, on non-quarantined TUs
export C2RS_DC3=<dc3-tree> C2RS_WIBO=<wibo>
python3 work/w-joint/mutate_joint.py src/system/rnddx9/Movie.cpp 5
python3 work/w-joint/mutate_joint.py src/system/gesture/NavigationSkeletonDir.cpp 5
python3 work/w-joint/mutate_joint.py src/system/synth/StreamNull.cpp 5
```

All scripts are **stdlib-only** and read-only against the corpus; the mutation
script writes only inside `work/w-joint/mut/` and restores the `in` between
runs. `work/` is gitignored; the scripts and the text outputs are force-added as
records, and no IL, obj or `_CL_*` artifact is committed.
