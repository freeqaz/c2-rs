# Findings — lane `w-repro`: IL capture is reproducible; the CORPUS is not

Pre-registration: [`_2026-08-04-w-repro-prereg.md`](_2026-08-04-w-repro-prereg.md),
committed before any measurement.

**One line:** the front end is byte-deterministic — 0 variation in 72 repeated
captures of the TU that supposedly drifted, and 6 whole-871-TU censuses that
collapse to exactly **2 distinct sha256, one per corpus**, across a 32× spread in
concurrency — and the two disagreeing census files were taken **against two
different versions of the dc3 source tree**, 40+ commits apart. w-bss2's landed
**110/117** and **68/68 survive unchanged on every capture file taken at the same
path**. A second, previously unrecorded variable turned up on the way: the census
is **bound to the corpus's directory path**, and moving it silently deletes 20 %
of the graded population.

---

## 1. What was predicted, and what happened

| | registered | outcome |
|---|---|---|
| **C0** control | `one()` over a stored `.gl` reproduces its record exactly | **held** (below) |
| **P1** | `Anim.cpp` × 24 serial → **exactly 2** distinct `.gl` hashes | **REFUTED** — 1 hash |
| **P1′** rival | 1 hash serially; ≥ 2 under load | **partly** — 1 serially **and** 1 under 14-way load, so even P1′ overstates it |
| **P1″** rival | > 2 hashes, drift | refuted |
| **P2** | third whole census confines instability to ≤ 4 TUs | **held, vacuously** — run 3 is **byte-identical** to run 2 |
| **P2′** rival | > 20 TUs unstable | refuted at a fixed corpus; **68 TUs** differ across the corpus change |
| **P3** | `gid_new − gid_old` is a constant **+4** on 100 % of records | **REFUTED** — 91.56 % are **0**, 7.39 % are +4, the rest a tail |
| **P3′** rival | the shift is not constant | **held** |
| **P4** | `grade.py` gives 110/117 and 68/68 on every file | **held** on all three same-path files |
| **P4′** rival | they move | refuted at a fixed path; **the denominator moves at a different path** (§5) |

Score: **2 predictions right (C0, P4), 2 wrong (P1, P3), 1 right-but-empty (P2).**
Both wrong ones were wrong in the reassuring direction — the instrument is
*better* behaved than registered — and both rivals that won were registered.

The decline clause was **not** invoked: the mechanism was identified causally, so
nothing had to be declined.

---

## 2. The front end is deterministic. Measured, not assumed.

`src/system/rndobj/Anim.cpp`, the workload's own flags, `/Bd /d2nop`, cwd
`../dc3-decomp`:

| arm | N | distinct `.gl` sha256 | distinct `.ex` sha256 | distinct kept-record set |
|---|---:|---:|---:|---:|
| serial | 24 | **1** | **1** | **1** |
| 14-way concurrent | 48 | **1** | **1** | **1** |

`.gl` = `0181b16e84e3e3af…`, `ngl` = 65, 47 kept records, on all 72 runs.

And at whole-census scale, with the tree held still:

```
glcensus 09:20 (run 2)  sha256 4c6c704cd62a522ed44859cc4c8654411526c706fbb1f3046dad8c3092c492d1
glcensus 09:25 (run 3)  sha256 4c6c704cd62a522ed44859cc4c8654411526c706fbb1f3046dad8c3092c492d1
```

**Byte-identical, 871/871 TUs.** So `C0` holds and the parse side is not the
source of anything.

Three candidate mechanisms are dead on inspection or on measurement:

* **the capture cache is not on this path.** `work/w-bss2/cap.py` shells straight
  to `wibo cl.exe` with `TMP`/`TEMP` pointed at a fresh per-call
  `tempfile.mkdtemp`. It never reads `work/capture-cache` and never runs `c2rs`.
  (Stated in the prereg before measuring, so it is not a post-hoc exclusion.)
* **`sections.jsonl` did not change.** mtime 07:21, before both runs, clean
  against `HEAD`. The `wanted_names()` filter was the same on both sides.
* **concurrency is not it** — 48 concurrent captures agree with 24 serial ones,
  and the jobs sweep in §6 agrees at 1, 14 and 32.

---

## 3. The mechanism: the corpus moved under the instrument

`../dc3-decomp` is a **live decompilation repo that other agents are merging
into continuously.** Between the two censuses it took 40+ commits:

```
dd9a4bdc  08:50:45   <- last commit before run 1
…
979488c2  09:14:38   Merge re7-animtask: AnimTask::Poll 87.2% -> 99.9% and four live bugs
940d07dc  09:15:17   Merge reentry-7fn: RndRibbon::UpdateChase …   <- HEAD at runs 2 and 3
```

`git diff dd9a4bdc..HEAD` touches **8 `.cpp` and 3 `.h` under `src/`** — including
`src/system/rndobj/Anim.cpp` (+50/−38), whose `AnimTask::Poll` was rewritten. The
three symbols that vanished from the census line
(`$?msg@?CC@??Poll@AnimTask@@…`, `$?msg@?DO@…`, `$?$S6@?CC@…`) are
**function-local statics of `AnimTask::Poll`**, whose mangled names carry a
lexical-scope index. Rewriting the function renumbered the scopes.

### 3.1 The full differing set is accounted for, 68/68

Masking nothing, **68 of 871** TU records differ between run 1 and run 2 (the
parent's "one TU" was the count after masking `gid`; the other 67 differ in `gid`
only). Every one is explained:

| bucket | TUs |
|---|---:|
| include closure contains a changed header (`HamListRibbon.h`, `HamNavList.h`, `DoubleExponentialSmoother.h`), measured with real `/showIncludes` | **64** |
| the TU's own `.cpp` changed and it includes none of those headers (`Dir.cpp`, `Anim.cpp`, `Mat_NG.cpp`, `TexBlender.cpp`) | **4** |
| **unexplained** | **0** |

The include predicate is **containment, not prediction**, and the control says
so: of 40 randomly sampled *non*-differing TUs, **13 also include a changed
header**. Two changed `.cpp` (`HiResScreen.cpp`, `Ribbon.cpp`) likewise do *not*
appear in the differing set. A source change is necessary here, not sufficient —
which is the right shape, since a body edit only moves this census if it moves a
data-global record.

### 3.2 Causal proof, single TU

`git cat-file blob dd9a4bdc:src/system/rndobj/Anim.cpp` compiled **today**, with
today's `sections.jsonl` and today's parser:

```
nkeep probe=50   run1=50   run2=47
probe == run1 record (ngl + all 50 keep entries, i and gid included): True
probe == run2 record:                                                False
```

The pre-window source reproduces run 1's record **exactly**, indices and ids
included. Nothing about the harness had to change.

### 3.3 Causal proof, whole census

`git archive dd9a4bdc src` materialised to `work/w-repro/dc3-base` and censused.
Against run 1, with anonymous-namespace records excluded (see §5 — they cannot be
joined across paths):

```
non-anon name-set differs : 0 / 871
ngl differs               : 0 / 871
gid / size / align / linkage differs : 0 / 871
```

**Run 1's corpus content is exactly commit `dd9a4bdc`.** The census was never
irreproducible; it was reproducible against a corpus nobody wrote down.

---

## 4. Is `gid` safe to sort by?

`work/w-bss2/grade.py:96` sorts by `(gid, i)`; `work/w-bss2/r56.py:124` sorts by
`gid` alone.

**The "+4 uniform" reading is wrong.** Over the 13,506 records comparable between
run 1 and run 2:

| `gid_new − gid_old` | records | share |
|---|---:|---:|
| **0** | 12,366 | 91.56 % |
| **+4** | 998 | 7.39 % |
| +1 / +5 / −2 / +7 / −1 / +2 / +3 | 73 | 0.54 % |
| \|Δ\| > 75,000 (e.g. −75,942, −84,327, −107,695) | 5 | 0.04 % |

The +4 bloc is the header-driven TUs of §3.1, not a global base shift. **39 of
871 TUs sort into a different `(gid, i)` order** between the two files — but that
is the corpus moving, not the key misbehaving: at a fixed corpus and path the two
files are byte-identical, so the order is identical too.

**Answer: `gid` is a deterministic function of the `.gl` bytes, and sorting by
`(gid, i)` is reproducible.** `grade.py` and `r56.py` are not in question on
these grounds.

### 4.1 But `gid` is a weak key on its own merits

Measured **inside one file** (run 3), over 13,524 kept records in 713 TUs:

| property | value |
|---|---:|
| records sharing a `gid` with another record in the same TU | **6,226 (46.04 %)** |
| adjacent `.gl`-order pairs where `gid` *decreases* | **5,718 / 12,811 (44.63 %)** |
| `gid` > 65,535 | 2,078 (15.37 %) |
| decimal-digit histogram | 2:93 · **3:6,842** · 4:80 · **5:6,120** · 6:389 |

That histogram is **bimodal**, which a monotone per-TU record counter cannot be,
and `glparse.py`'s own docstring already warns that a record id that is a
multiple of 128 is indistinguishable from a separator. Nearly half of all
ordering decisions therefore rest on either a tie or a decrease. This does **not**
invalidate the landed numbers — the key is deterministic and `grade.py` breaks
ties with `i` — but "ascending id" is `grade.py`'s **winning `.data` walk model**
(52/68 = 76.5 %), so the model is fitted on a field whose framing is not fully
established. Board row proposed below.

### 4.2 One latent hazard, measured not to fire

`r56.py:124` reads `sorted(names, key=lambda x: r["glrec"][x]["gid"])` where
`names` is a **`set`**. Python's sort is stable, so a `gid` tie resolves in set
iteration order, which depends on `PYTHONHASHSEED` and therefore varies between
*processes*. Tested: **5 runs at `PYTHONHASHSEED` 0–4 produced byte-identical
output** (`md5 841f037bd0a57aff974f0f0751c99a96` ×5). Latent, not live — the
probe cells are small enough that no tie occurs. Worth fixing to `(gid, index)`
before anyone enlarges the grid.

---

## 5. The finding nobody was looking for: the census is bound to the corpus PATH

MSVC mangles an anonymous namespace as `?A0x<8 hex>`, and **that hash is derived
from the translation unit's path**. Single-variable experiment — `git archive
HEAD src` into `work/w-repro/dc3-head`, byte-identical content, different
directory, everything else held:

| | same path (run 3) | different path (head-copy) |
|---|---:|---:|
| TUs whose record differs | — | **49** |
| — of those, carrying an anonymous-namespace record | — | **48** |
| — of those, with no anonymous namespace at all | — | **1** (`src/system/utl/JobMgr.cpp`) |
| `gid` / size / align / linkage differences | — | **0 TUs** |
| `ngl` differences | — | **0 TUs** |
| relative `.gl` order of non-anon records changed | — | **1 TU** (`MoveDir.cpp`) |

So the compiler output is stable; what breaks is the **join**. 48 TUs carry
anonymous-namespace symbols (226 records in the 0850 file), their names no longer
match `sections.jsonl`, and they silently drop out. Re-grading:

| | run 1 / 2 / 3 (same path) | base / head-copy (other path) |
|---|---|---|
| `.bss` non-COMDAT ≥ 2 syms | **117**, skipped 4 | **93**, skipped **28** |
| `.bss` pure bump | **110/117 (94.0 %)** | 87/93 (93.5 %) |
| `.data` non-COMDAT ≥ 2 syms | **68**, skipped 2 | **53**, skipped **17** |
| `.data` pure bump | **68/68** | 53/53 |
| A1 `.gl` walk, `.bss` | 85/110 = 77.3 % | 68/87 = 78.2 % |
| A1 id walk, `.data` | 45/68 = 66.2 % | — |

### 5.1 The path also moves `.gl` file order, with no anonymous namespace in sight

`src/system/utl/JobMgr.cpp` has one anonymous namespace: none. Its one kept
record is `$gJobIDCounter`, identical in `gid` (18425), size, alignment and
linkage in both trees — and it sits at **`i = 1` in `../dc3-decomp` and `i = 7`
in `work/w-repro/dc3-head`**, jumping past six other records whose own relative
order is unchanged. Recaptured live from both trees in one process, so the only
difference is `cwd`.

`i` — the `.gl` file order — is `grade.py`'s **winning `.bss` walk model**
(85/110). It is not invariant under a corpus path change. The effect is rare
(1 TU outside the anonymous-namespace population) and this lane has **no
explanation for it**.

Related, and visible in the same dump: `glparse.globals_in_order()` admits
obvious non-symbols. `JobMgr.cpp`'s 19 records include `r(&_TI4?AV…`,
`c&??_B?7??…`, `u(&??_R0…`, `t(&_CTA4…` — junk-prefixed, all `gid = 1`. Over a
40-TU sample, **569 of 8,789 records (6.5 %)** have one of `(&'"{}#!` in their
first four characters, i.e. cannot be a mangled symbol. Those records are
harmless to `grade.py`'s *scores* (its cells are filtered against real obj
symbols) but they inflate `ngl` and they shift every `i` after them, so the
`.gl`-order key is an index into a list that is 6.5 % noise.

**The rates barely move; the denominator loses 20 %.** A percentage that stays
healthy while its population quietly shrinks is `STATUS.md` trap 5 — *absence
reads as success unless something forbids it* — with a new instance. Nothing in
the pipeline prints the denominator's provenance, so a census taken from a
different checkout would have looked fine.

---

## 6. Concurrency control

Registered arm: is the harness racing? Same frozen tree
(`work/w-repro/dc3-head`, a `git archive` of `940d07dc` that cannot move under
the run), same path, four censuses:

| run | jobs | wall | sha256 |
|---|---:|---|---|
| head-copy | 14 | 24 s | `1bd3a467ca7e9d2924128cf5b7c0f487179c3978ff4dbec77acde972b64355fd` |
| j1 | **1** | 3 m 38 s | `1bd3a467…` |
| j14b | 14 | 24 s | `1bd3a467…` |
| j32 | **32** | 22 s | `1bd3a467…` |

**All four byte-identical, 871/871 TUs, across a 32× spread in concurrency.**
`glcensus.py`'s `ThreadPoolExecutor`, `cap.py`'s per-call `mkdtemp`, and wibo's
`_CL_*` bundle naming are not racing. Together with the 72 repeated single-TU
captures in §2 this closes the harness-race hypothesis: **the only measured
variables are the corpus's content and its path.**

---

## 7. Do w-bss2's landed numbers survive? Yes.

`grade.py` re-run against every same-path capture file, via a shim that rebinds
`grade.GLC` and changes nothing else:

```
run 1 (0850):  .bss 110/117    .data 68/68    R0 12207/12207 = 100.00%
run 2 (0920):  .bss 110/117    .data 68/68    R0 12204/12204 = 100.00%
run 3 (0925):  .bss 110/117    .data 68/68    R0 12204/12204 = 100.00%
```

The three outputs are **identical line for line** except R0's denominator, which
moves by exactly the 3 `AnimTask::Poll` locals. **`docs/OBJ_DATA_BSS_SHAPE.md`'s
110/117 and 68/68 stand as published**, and they stand *across* a corpus change,
which is a stronger result than the lane set out to get: the walk-order and
allocator conclusions did not depend on the version of the game source they were
measured against.

They do **not** stand across a *path* change, and that is §5.

---

## 8. What is still not known

Named so it is not mistaken for settled:

* **Why `gid`'s digit histogram is bimodal.** §4.1 measures it; nothing here
  explains it. `_gid_before`'s framing may be reading two different fields.
* **Why a corpus-path change perturbs `.gl` record *positions*** — in 33 TUs via
  the anonymous-namespace hash (uniformly, order-preserving in 32 of them,
  genuinely reordered in `MoveDir.cpp`), and in `JobMgr.cpp` with no anonymous
  namespace involved at all (§5.1). Measured, unexplained. This is the loose end
  most worth pulling, because `i` is the winning `.bss` walk model.
* **Whether the 6.5 % of `globals_in_order()` records that are not symbols
  (§5.1) shift any *cell's* walk order.** They cannot change a cell's *score*
  directly, but they sit between real records and every `i` past them moves.
* **Whether `sections.jsonl` itself is corpus-pinned.** It was built at 07:21
  from a tree that had already moved past whatever the earlier lanes used. This
  lane did not regenerate it (it costs the real toolchain and ~102 MB of objs),
  so every number graded against it inherits an unrecorded corpus version. That
  is the same defect as the one this lane characterized, one level up, and it is
  the more expensive one.

---

## 9. Proposed fixes — NOT made by this lane

Lane `w-cfgimpl` owns `crates/c2-il/` and `crates/c2-core/`; `scripts/` and
`work/w-bss2/` belong to other lanes. Proposals only.

1. **Stamp provenance into every census.** `scripts/regen_census.sh` should write
   a first line (or a sidecar `.prov`) carrying `git -C $C2RS_DC3_SRC rev-parse
   HEAD`, the dirty-tree flag, the **absolute** `C2RS_DC3_SRC`, the flags-file
   hash, and the `sections.jsonl` hash. Two censuses that disagree would then say
   so in one `diff` of the header instead of costing a lane.
2. **Refuse to regenerate against a dirty or moving tree** unless
   `--allow-dirty` is passed, and re-check `HEAD` *after* the run — a 23 s census
   is easily straddled by a merge, and a 48 min one certainly is.
3. **Print the denominator's provenance in `grade.py`**, and make
   `skipped {'symbol absent from .gl': N}` a hard warning above a registered
   threshold. At 28/121 it was already 23 % and printed as an aside.
4. **`r56.py:124`**: sort by `(gid, index)`, not `gid` alone (§4.2).
5. **Pin the corpus path**, or key the join on something path-free. Anything that
   joins on mangled names cannot be moved between directories, and nothing
   currently says so.

## 10. Proposed board rows

`BOARD.md` says next free is **#196**, and lane `w-pair` has also proposed
196–200 in an unlanded report, so these are numbered from **#201** to avoid a
collision. Renumber down if w-pair's rows never land.

| # | status | row |
|---|---|---|
| **#201** | **REFUTED** | *"The IL capture is nondeterministic."* Front end is byte-exact over 72 repeated captures and two whole-871-TU censuses; the 0850/0920 disagreement is 68 TUs of **moved corpus** (dc3 `dd9a4bdc` → `940d07dc`), causally proven by recompiling the pre-window blob. |
| **#202** | **OPEN** | **Census provenance.** No artefact records which dc3 commit, which path or which `sections.jsonl` it was built from. Cost of not having it: this lane. Fix in §9.1–9.2. |
| **#203** | **OPEN** | **The census join is path-bound.** `?A0x<hash>` anonymous-namespace mangling is path-derived; moving the corpus drops 48 TUs' anon symbols and **20 % of the graded `.bss`/`.data` population** while the printed rates hold (110/117 → 87/93, 68/68 → 53/53). New instance of trap 5. |
| **#204** | **OPEN** | **`gid` framing is not established.** 46.04 % of kept records share a `gid` in-TU, 44.63 % of adjacent `.gl` pairs decrease, digit histogram bimodal at 3 and 5. Deterministic, so the landed numbers stand — but "ascending id" is the winning `.data` walk model (52/68) and is fitted on this field. |
| **#205** | **OPEN** | **`globals_in_order()` admits non-symbols.** 569/8,789 = **6.5 %** of records over a 40-TU sample have `(&'"{}#!` in their first four characters. They inflate `ngl` and shift every `i` past them; `.gl` file order is the winning `.bss` walk model (85/110). Scores are unaffected — cells are filtered against real obj symbols — but the *key* is 6.5 % noise. |

---

## 11. Reproducing this

Everything is in `work/w-repro/` (gitignored — derived data and captured IL are
never committed). All of it is stdlib-only Python and touches no `crates/`.

| what | command, from `work/w-repro/` |
|---|---|
| compare two census files, gid masked | `python3 cmp.py A.jsonl B.jsonl` |
| the same, ignoring anonymous-namespace records | `python3 cmp2.py A.jsonl B.jsonl` |
| which TUs differ, and gid's intrinsic properties | `python3 whichtus.py A.jsonl B.jsonl` |
| gid collision / monotonicity / digit histogram | `python3 gidsanity.py A.jsonl` |
| N repeated captures of one TU | `python3 probe.py src/system/rndobj/Anim.cpp 24 serial` |
| the causal test | `python3 causal.py <old-blob.cpp> <census-src> run1.jsonl run2.jsonl` |
| re-grade an arbitrary census with w-bss2's unmodified `grade.py` | `python3 regrade.py <census.jsonl>` |
| include closure, real `/showIncludes` | `python3 includes.py <tu-list> <header…>` |
| a census against a frozen corpus | `C2RS_DC3_SRC=<tree> python3 ../w-bss2/glcensus.py out.jsonl <jobs>` |

Census files kept: `glcensus.20260804-0850.jsonl` (dc3 `dd9a4bdc`),
`…-0920.jsonl` and `run3.jsonl` (dc3 `940d07dc`), `base.jsonl` (`dd9a4bdc` at
another path), `head-copy/j1/j14b/j32.jsonl` (`940d07dc` at another path).
Frozen source trees: `dc3-base/`, `dc3-head/` (28 MB each, `git archive`).
