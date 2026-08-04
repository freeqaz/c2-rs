# Findings — lane `w-prov`: the corpus is now recorded, and the join is pinned

Pre-registration: [`_2026-08-04-w-prov-prereg.md`](_2026-08-04-w-prov-prereg.md),
committed at `3f979df` before any measurement this lane made.

**One line:** every census now carries a machine-readable stamp of the corpus it
was graded against and `grade.py` refuses to join two that disagree; the
committed `sections.jsonl` did carry an unrecorded corpus and **18 of 871 TU
records were stale**; and the path-bound join is **pinned, not repaired** —
because a path-free join was measured to work perfectly on the population and to
silently move the winning `.bss` walk score from **85/110 to 81/110**.

---

## 1. Scoreboard

| | registered | outcome |
|---|---|---|
| **KAC** | a repeated `gap` scan reproduces the merged tree exactly | **held** — every figure identical |
| **KAC-2** | `prov.py` accepts a matched pair and rejects six mismatch classes | **held**, 21 PASS / 0 FAIL, no toolchain |
| **P1** | frozen dc3 `dd9a4bdc` restores census `706402/2463318` and emitted `38457/178972` | **held on the census exactly**; emitted denominator exact, numerator `38459` (+2) |
| **P2** | frozen dc3 `940d07dc` at another path reproduces the live figures | **held on the census exactly**; emitted numerator `38457` vs live `38455` (+2) |
| — | (the registered confound) | **P2 measures the path term at exactly +2 on both corpora, which fully accounts for P1's residual** |
| **P3** | regenerating `sections.jsonl` changes it, 1–100 TUs differ | **held — 18 of 871** |
| **P4** | `grade.py` still gives `.bss` 110/117 and `.data` 68/68 | **half held** — `.bss` 110/117 unchanged, `.data` **68 → 69** |
| **P5** | a path-free join restores 93→117 and 53→68 | **held**, but only when applied at *capture*, and it moves the walk score |
| **P5′ / P5″** | partial recovery / over-joining | both **refuted** — full recovery, zero ambiguous matches |
| **P6** | `globals_in_order()` noise is 6.5 % ± 1.0 pp over 871 TUs | **REFUTED — 18.39 %**, 2.8× the 40-TU sample |
| **P7** | the `r56.py` fix leaves output byte-identical at seeds 0–4 | **held**, md5 `841f037b…` ×5, equal to the pre-fix incumbent |
| **decline** | `JobMgr.cpp` after three probes | **INVOKED** — declined, characterized, §6 |

Two registered predictions were refuted (P6 badly), one was half-refuted (P4),
and the decline clause was invoked once. All three are reported at full strength.

---

## 2. The corpus attribution, measured rather than argued

The merged tree measured **706552/2463393** and **38455/178975** against brief
incumbents of **706402/2463318** and **38457/178972**, with `crates/` untouched.
Three scans, one variable each:

| scan | corpus | path | per-function census | emitted census | `.gl` binding records |
|---|---|---|---|---:|---:|
| KAC | `940d07dc` | live | 706552/2463393 | 38455/178975 | 1515167 · 39296 · 731 |
| **base** | **`dd9a4bdc`** | frozen | **706402/2463318** | 38459/**178972** | **1515163 · 39294 · 732** |
| **head** | `940d07dc` | frozen | **706552/2463393** | 38457/**178975** | 1515167 · 39296 · 731 |

* **The per-function census is 100 % corpus and 0 % path.** `base` reproduces the
  incumbent to the digit; `head` reproduces the live figure to the digit.
* **So is `docs/STATUS.md`'s `.gl` binding invariants line** — `1515163 records,
  39294 row-conflicts, 732 name-conflicts` is exactly what `base` prints. That
  line is a workload measurement and nothing on the page said so.
* **The emitted census carries a +2 PATH term**, reproducible on both corpora
  (`head` 38457 vs live 38455; `base` 38459 vs incumbent 38457). `bound` moves
  −3 and the residue +3 with it. So the registered confound was real, was
  measured, and exactly accounts for P1's residual: `38459 − 2 = 38457`.

**That is a new finding and it is in `crates/`, not in the Python.** w-repro's
path effect was thought to live in the `.gl` name join; it also reaches the
front-page **emitted census**, by 2 of 38,457. Small, but it is not zero and
nothing announces it. Proposed as a board row; **not fixed here** — this lane
does not touch `crates/`.

---

## 3. `sections.jsonl` was stale, and the determinism control comes first

`work/w-bss/census/sections.jsonl` is committed and was last written at **04:52**,
i.e. at dc3 **`86357b58`** — the same commit w-cfgimpl independently named as its
own stale baseline. Regenerated at `940d07dc`:

```
sha256  e7a328b9fabb2fa08fab8058... ->  fb2f5865df1d35fc28acf199...
871 records both sides, src set identical, 18 TU records differ
```

**The control ran first, because "18 records changed" means nothing without it.**
Two independent full regenerations — 871 compiles and ~102 MB of intermediate
objs each — are **byte-identical** (`fb2f5865…` twice). The obj census is
deterministic; the 18 are corpus.

| bucket | TUs |
|---|---:|
| include closure contains `HamListRibbon.h` / `HamNavList.h` / `DoubleExponentialSmoother.h` | 7 |
| include closure contains `Part.h` / `Str.h` | 10 |
| the TU's own `.cpp` changed (`Anim.cpp`) | 1 |
| **unexplained** | **0** |

**This is an accounting, not a prediction, and the control is brutal about it:
39 of 40 randomly sampled NON-differing TUs also include one of those headers.**
`Str.h` is in nearly every include closure in the workload. A changed header is
necessary here and nowhere near sufficient — only a change that moves a
`.data`/`.bss` record moves this census. (w-repro's version of this control was
13/40; at this header set it is 39/40, so the containment predicate is close to
vacuous and is reported as such.)

### 3.1 What moved in a landed document

`docs/OBJ_DATA_BSS_SHAPE.md` publishes `.bss` **110/117** and `.data` **68/68**.
On a jointly regenerated pair at `940d07dc`:

| | published | regenerated |
|---|---|---|
| `.bss` non-COMDAT ≥2 syms | 117 | **117** |
| `.bss` pure bump | 110/117 | **110/117** |
| `.bss` winning walk (`.gl` order) | 85/110 = 77.3 % | **85/110 = 77.3 %** |
| `.data` non-COMDAT ≥2 syms | 68 | **69** |
| `.data` pure bump | 68/68 | **69/69** |
| `.data` winning walk (id order) | 45/68 = 66.2 % | **46/69 = 66.7 %** |
| R0 | 12207/12207 = 100 % | 12221/12221 = 100 % |

**`.bss` did not move at all. `.data`'s population grew by one and its rate
stayed at 100 %.** The published claims survive; the population one of them was
measured over did not. This is exactly the case `grade.py`'s new population check
exists to make loud, and **it fired on its first real run**.

---

## 4. The provenance stamp

`work/w-bss2/prov.py`. Every census writes `<file>.prov`:

```
corpus:  head, head_after, moved_during_run, dirty, path_rel, path_sha256
data:    data_file, data_sha256, data_records
inputs:  flags_sha256, files_sha256, sections_sha256 (glcensus only)
meta:    schema, tool, generated_utc, begin_scope, committed_safe
```

**HEAD is snapshotted before the run and re-checked after**, and
`moved_during_run` is a hard error at write time *and* at read time. A `sections`
census takes minutes; being straddled by a merge is the normal case.
`begin_scope` is an honesty field: `"run"` means the snapshot covers the
compiles, `"aggregate"` means it does not and drift during compilation was
invisible to that stamp.

**How a consumer fails.** `grade.py` calls `prov.read` on both files and
`prov.require_join`; any of these exits **2** with a banner:

* no sidecar; wrong schema
* the sidecar does not hash to its data file (stale or regenerated without it)
* `moved_during_run`
* `head` differs between the two censuses
* **`path_sha256` differs** — and the error prints the measured cost of the case
  it is blocking (`.bss` 117→93, `.data` 68→53, rates intact)
* `flags_sha256` differs
* `glcensus`'s recorded `sections_sha256` ≠ the `sections.jsonl` on disk

and separately exits **3** when the graded **population** differs from its
incumbent. `--no-prov-check` and `--allow-population-change` exist and print an
`UNVERIFIED` banner; the incumbent is named with the corpus it was taken at.

**The absolute path is recorded but never committed.** `sections.jsonl` is
force-added, `glcensus.jsonl` is not, so `prov.write(committed=True)` strips
every absolute path and **raises if one survives** — a future field that would
leak `/home/<user>/…` fails there rather than in the history. What the committed
sidecar keeps is `path_rel` (`../dc3-decomp`; `null` when it would need two
levels up, which is exactly the case that encodes a machine layout) and
`path_sha256`. **The pin the join compares is `path_sha256`**, which is present
on both sides of the tracked/untracked boundary and is opaque.

`prov.py selfcheck` exercises **every** rejection path — 21 PASS, 0 FAIL, no
toolchain and no corpus. A stamp checker that cannot be shown to fail is trap 5
wearing a lab coat.

`scripts/status.sh` now stamps the **workload commit** beside the tree and binary
in the generated block. Half the numbers on that page are workload measurements
and none of them said so.

### 4.1 Two defects the stamp found on its own first run

* **`paths.SECTIONS` resolved to the MAIN repo unconditionally.** A worktree that
  regenerated `sections.jsonl` then built its `.gl` census against the main
  repo's *older* copy — a silent cross-corpus join, the exact defect this lane
  exists to close. Found because `glcensus` printed `NO PROVENANCE` for a file
  the worktree had just stamped. Now lane-preferring.
* **`regen_census.sh`'s sibling default was `$ROOT/..`**, which in a worktree is
  `.claude/worktrees/dc3-decomp`. The script printed `SKIP: toolchain absent` and
  **exited 0** — a caller saw success and a census that never ran. Trap 5, inside
  the tool built to close trap 5.

---

## 5. Pin, not path-free — and the measurement that decided it

**The path binding does not live in the join.** The first attempt normalised
`?A0x[0-9a-f]{8}` at join time and recovered **exactly nothing** (93 stayed 93).
That was an invocation being checked rather than a result: `glcensus.py`'s
`wanted_names()` filters each TU's `.gl` against `sections.jsonl`'s symbol names
**at capture time**, so at a moved path the anonymous-namespace records are
discarded before any join exists. `glcensus.head-copy.jsonl` contains **zero**
`?A0x` records. A path-free *join* is architecturally incapable of repairing
this.

Re-captured with the filter normalised (51 TUs — the ones carrying an
anonymous-namespace symbol; the rest cannot move):

| | same path | moved path, literal | moved path, **path-free capture** |
|---|---:|---:|---:|
| `.bss` cells | 117 | 93 | **117** |
| `.bss` skipped | 4 | 28 | **4** |
| `.bss` pure bump | 110/117 | 87/93 | **110/117** |
| **`.bss` winning walk** | **85/110** | 68/87 | **81/110** |
| `.data` cells | 68 | 53 | **68** |
| `.data` winning walk | 45/68 | 33/53 | **45/68** |
| ambiguous name match | 0 | 0 | **0** |

**P5 is green and P5′/P5″ are refuted: normalisation recovers the population
perfectly and over-joins nothing. And it moves the winning `.bss` walk score from
85/110 to 81/110 with nothing announcing it.**

All four flipped cells flip because the `.gl` record **order** changed, not
because a name failed to join. In `App.cpp` the two `?A0x` records sit at `.gl`
indices 273–274 *after* `?gRealCallback` at 192, and at the moved path at 80–81
*before* it at 194 — same obj, same addresses, different predicted order. Same
shape in `ContextChecker.cpp`, `MetagameRank.cpp` and `LiveCameraInput.cpp`.

**Decision: PIN.** The rule was fixed in the pre-registration before any of this
was measured — *pin unless P5 is green **and** the `.gl`-order perturbation is
explained* — and the second condition failed (§6). Three further reasons:

1. **Pin is general; normalisation is not.** The pin catches every path-derived
   effect, including the order one and the +2 in the emitted census.
   Normalisation addresses only the name axis.
2. **Path-free would mean changing what the census *contains***, not how it is
   read — the filter is upstream — so every landed number's basis would move.
3. Adopting it would have published 81/110 = 73.6 % where the landed figure is
   85/110 = 77.3 %, with the denominator "restored" and nothing to say the
   numerator had been corrupted. That is `STATUS.md` trap 5 with the mask
   reversed, and it is the failure this lane was created to stop.

---

## 6. `JobMgr.cpp` — DECLINED after the three registered probes

`src/system/utl/JobMgr.cpp` has no anonymous namespace, and `$gJobIDCounter` has
identical `gid` (18425), size, alignment and linkage everywhere. Its `.gl` index:

| tree | corpus | path len | index |
|---|---|---:|---:|
| live `../dc3-decomp` | `940d07dc` | 34 | **1** |
| `work/w-repro/dc3-head` | `940d07dc` | 51 | **7** |
| `work/w-repro/dc3-base` | `dd9a4bdc` | 51 | **1** |

* **Probe 1** — the record *set* is identical (19 records, `ngl` 19 both sides).
  It is a **reorder, not an insertion**.
* **Probe 2** — **not path length**: `dc3-base` and `dc3-head` have the same
  path, same length, same parent, and differ in index.
* **Probe 3** — **no `?A0x` name appears anywhere** in either record list, so the
  anonymous-namespace mechanism is not involved.

**Newly established, and it narrows w-repro's account:** the index moves under a
path change at fixed content *and* under a content change at fixed path. It is
not a path-only effect, and it is not the `?A0x` effect. No single measured
variable accounts for it, so the decline clause fires as priced.

**The price, as registered:** `grade.py`'s winning `.bss` model (`.gl` order,
85/110 = 77.3 %) stays fitted on a key with one known, unexplained perturbation;
board **#203** stays open; any future lane that moves the corpus must re-derive
this. It is left **characterized, not fitted**.

---

## 7. `globals_in_order()` is 18.4 % noise, not 6.5 %

Full 871-TU measurement, `(&'"{}#!` in the first four characters:

| | value |
|---|---:|
| records (`ngl`, total) | 84,898 |
| non-symbol records | **15,613 = 18.39 %** |
| TUs carrying at least one | 826 = **94.8 %** |
| real records whose `i` is shifted by preceding noise | 62,420 = **90.09 %** |

**P6 is refuted at 2.8× the registered band.** w-repro's 40-TU sample badly
understated it. Top prefixes are `D&??`, `c&??`, `?&??`, `F&??`, `z(&_` — RTTI
and EH descriptor fragments, not symbols.

**What it does and does not cost.** `grade.py` only ever *sorts* by `i`, and
inserting noise between two real records preserves their relative order, so **no
cell's score is affected and every landed number stands**. What is wrong is
`ngl`, which is published per TU in `glcensus.jsonl` and is inflated by 18.4 %,
and any future model that reads `i` as a *position* rather than an *order*.

---

## 8. Proposed board rows (this lane mints only w-repro's)

`BOARD.md` reads next free **#196**. Lane `w-cfgimpl`'s landed rung names
**196–200** in its own frozen text and `w-repro`'s names **201–205**; both are
landed, so neither is renumbered and w-pair's unnumbered proposals stay
unnumbered. **This lane lands w-repro's #201–#205 only** and records that
196–200 are claimed, so nobody re-mints them. Its own findings are proposed from
**#206**:

| # | status | row |
|---|---|---|
| **#206** | **OPEN** | **The emitted census carries a path term.** Identical corpus at a different directory moves the emitted-census numerator by **+2** (38455 → 38457), `bound` by −3 and the residue by +3, reproducibly on two different corpora. The per-function census and the `.gl` binding-invariants line are path-neutral to the digit. This one is in `crates/c2-harness`'s binding, not in the Python join, and is the first evidence that w-repro's path effect reaches the front page. |
| **#207** | **CLOSED by w-prov** | **Census provenance exists and is enforced.** Every census writes a `.prov` sidecar; `grade.py` exits 2 on any of seven mismatch classes and 3 on a population change; `prov.py selfcheck` proves all of them fail (21 PASS). Closes the fix half of **#202**. |
| **#208** | **OPEN** | **The `.gl` record ORDER is not invariant under either corpus path or corpus content.** Four `.bss` cells flip verdict on a path change alone (85/110 → 81/110), and `JobMgr.cpp` moves 1 → 7 under path *or* content with no `?A0x` involved. Declined after three probes; this is why the join is pinned rather than made path-free. |
| **#209** | **REVISED, was #205** | **`globals_in_order()` is 18.39 % non-symbols over 871 TUs**, not the 6.5 % of a 40-TU sample; 94.8 % of TUs carry some and 90.09 % of real records sit after one. Scores unaffected (`i` is only ever a sort key); `ngl` is inflated by 18.4 %. |
| **#210** | **CLOSED by w-prov** | **`sections.jsonl` was stale by 18 of 871 TU records** (dc3 `86357b58` → `940d07dc`), regenerated and stamped, with a byte-identical two-run determinism control. `.data`'s graded population moved 68 → 69; `.bss` did not move. |

---

## 9. Reproducing this

| what | command |
|---|---|
| the stamp's own tests | `python3 work/w-bss2/prov.py selfcheck` |
| read a stamp | `python3 work/w-bss2/prov.py work/w-bss/census/sections.jsonl` |
| regenerate both censuses, stamped | `scripts/regen_census.sh --jobs 14` |
| grade, with the pin enforced | `cd work/w-bss2 && python3 grade.py` |
| the corpus attribution | `c2rs gap … --cwd work/w-repro/dc3-base` vs `--cwd ../dc3-decomp` |
| P5, path-free capture | `work/w-prov/p5_recapture.py <tree> <out>` then `p5_pathfree.py … --norm` |
| P6 | `python3 work/w-prov/p6_glnoise.py 14` |

`work/w-prov/` is gitignored; the census sidecars are not.
