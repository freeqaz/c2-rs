# w-coldcross — **the lock that made shared case directories look expensive protects a 0.65-second regeneration and is held for the whole run; the contention `w-gateperf` declined on is an artefact of the lock's SCOPE, not of sharing**

    Tag:       w-coldcross
    Slug:      coldcross
    Date:      2026-08-18
    Kind:      construct — makes `scripts/expr_sweep.sh` and `scripts/mode_cross.sh`
               grade a shared, content-addressed, immutable corpus without changing
               a single case, count or verdict, and adds a verification of the
               corpus that did not exist in either driver
    Outcome:   built
    Fixtures:  none — construct rung: `scripts/corpus_dir.sh` (new), the two
               generated gate rows' case-directory resolution, `gate.sh`'s
               `corpus:` accounting line, and 13 new `--selftest` arms. The port
               is untouched: `git diff abc64be3..HEAD -- crates/ fixtures/` is
               EMPTY
    Census:    +0 — `crates/` is byte-identical to base
    Record:    this file; PREREG `work/w-coldcross/PREREG.md` (frozen at
               `bdf23a2f`, this lane's first commit, before any contention
               measurement); board rows **`#3282`**–**`#3286`**

## 1. The question, and the answer in one line

`w-gateperf` §11.1 declined worktree-independent case directories and said, in
its own words, that the decline **rested on reasoning about the lock's fallback
and not on a measurement**. This lane's job was to measure it.

> **MEASURED: `mode_cross.sh` holds its case-directory lock for the ENTIRE RUN —
> 29 s warm, 347 s cold — to protect a destructive regeneration that takes
> 0.65 s.** The private-cold fallback `w-gateperf` priced does fire, and it fires
> because the lock's scope is ~500× larger than the thing it guards. Make the
> shared corpus **immutable** and there is nothing to guard: no lock is taken,
> the fallback cannot fire, and the shared directory is not merely faster than
> the per-worktree one, it is **safer**, because two lanes with different
> `scripts/sweep.d` can no longer address the same directory at all.

`w-gateperf`'s reasoning was correct **about a mutable directory**, which is what
it priced. It is the premise that was replaceable, not the inference.

## 2. FIRST, A NUMBER IN THE DISPATCH IS WRONG, AND EVERYTHING DOWNSTREAM MOVES

The dispatch, `w-gateperf` §2 and board **#3265** all quote the cold mode cross
as **1,261 s** — "the largest number in the document", "~19 minutes per new
lane". Re-measured on a fresh `scripts/setup_worktree.sh` worktree at this
lane's base:

| | `w-gateperf` §2 | **this lane** |
|---|---:|---:|
| `--jobs` | **4** | **16** |
| cold mode cross | 1,261 s | **347 s** |
| cold sweep fill | — (its 303 s is *uncached*) | **92 s** (`cache: hit=1 miss=19555`) |
| whole cold gate | 1,755 s | **510 s** |

**1,261 s is a `--jobs 4` measurement.** `w-gateperf` itself moved the standing
gate flag to `--jobs 16` (board #3263) and its own tip table is at 16, but the
cold figure was never re-taken there, so a `--jobs 4` number and a `--jobs 16`
recommendation ended up in one sentence. The ratio 1,261 / 347 = **3.6×** is the
concurrency ratio, which is what it should be.

**The lane is still worth doing and the honest framing is smaller**: ~6 minutes
per new lane, not ~19. §9 does the arithmetic in the terms the dispatch asked
for, with the corrected numbers.

## 3. THE PROFILE — a fresh worktree's first gate, decomposed inside one run

Base `abc64be3`, `scripts/gate.sh --jobs 16 --require-graded`, first run in a
worktree created by `scripts/setup_worktree.sh`, load average 0.4 → 17,
**GATE: PASS, exit 0, 510 s**.

| leg | cold (this run) | warm (`w-gateperf` §13.1, jobs 16) | cold excess | shareable? |
|---|---:|---:|---:|---|
| 18 mode lanes | 19 s | 2 s | 17 s | **no** — `fixtures/cpp/` is the tree under test |
| generated sweep (19,556) | **92 s** | 29 s | **63 s** | **yes** |
| mode cross (90,812) | **347 s** | 29 s | **318 s** | **yes** |
| debug lane (18 × 386) | 44 s | 11 s | 33 s | **no** — same fixtures |
| overhead | ~8 s | ~7 s | ~1 s | — |
| **TOTAL** | **510 s** | **78 s** | **~430 s** | **381 s of 430 = 89 %** |

**The two generated rows are 89 % of the cold excess and they are the two whose
corpus is a pure function of `scripts/`.** The other 50 s belongs to the 386
hand-written fixtures, whose path is inside the worktree because *the fixtures
are the tree under test*; §10 prices sharing those and declines it.

## 4. WHY IT IS COLD, DEMONSTRATED RATHER THAN ASSERTED

The corpus a fresh worktree generates is **byte-identical** to the one already on
disk — `diff -rq` over 19,556 files against the main repo's live case directory
is empty, and two consecutive generations in one worktree are identical too. It
is cold anyway, because the capture cache's key contains the **source path**, and
must: `c2` bakes it verbatim into `.gl` and `.debug$S`.

**The base gate is its own proof.** Its sweep leg generated a corpus that is
byte-identical to a corpus this box has compiled thousands of times, at a new
path, and printed:

```
cache: hit=1 miss=19555 validated=0 cache-bad=0 (of 19556 cases)
```

**19,555 misses on 19,555 files whose bytes were already in the cache.** That
single line also settles one of the alternatives the dispatch asked about and
saves an experiment: **seeding a new worktree's case directory by copy or
reflink is worth exactly 0 s.** `/home` is btrfs and `cp --reflink=always` is
available, and it is the wrong tool — a reflink reproduces the *bytes*, and the
bytes were never the problem.

**And the converse, measured (M1).** From this fresh worktree, before any change,
`C2RS_CROSS_CASES=<main-repo>/work/mode-cross/cases scripts/mode_cross.sh` at
`--jobs 16`:

```
assigned 19556 cases over 18 lanes = 90812 cells
checked=90812 mismatches=0 graded=90424 ungraded=388 unknown=0
                                                          29 s
```

**29 s against 347 s, 12×, on a tree that had never compiled a case** — and the
counts are digit-identical to the cold run's, which is what makes it a statement
about speed and not about coverage. Nothing but the path string changed.

## 5. THE CONTENTION QUESTION — MEASURED

### 5.1 The lock's scope, which is the whole finding

| | measured |
|---|---:|
| `sweep_gen.py` writing all 19,556 cases | **0.65 s** (0.15 s user + 0.46 s sys) |
| …and verifying it against another generation (`diff -rq`, 19,556 files) | **1.2 s** total |
| `mode_cross.sh`'s hold on `work/mode-cross/.cross.lock` | **the entire run** — `mkdir` before generation, `rmdir` in an `EXIT` trap |
| that hold, warm | **29 s** |
| that hold, cold | **347 s** |

The lock exists for one reason, and the file says so: *"this driver deletes and
regenerates it, so two concurrent runs delete each other's cases mid-grade and
BOTH results are meaningless while looking perfectly ordinary."* That is a
correct reason and it applies to **0.65 s** of a run that holds the lock for up
to **347**. The exposure window is ~500× the hazard window.

### 5.2 The naive share, and the fixed share, driven at 2, 3 and 4 lanes

<!-- CONTENDTABLE -->

### 5.3 What the numbers say

<!-- CONTENDPROSE -->

## 6. WHAT SHIPPED — `scripts/corpus_dir.sh`

A shared generation is named `<main-repo>/work/corpus/gen-<digest>`, where the
digest is over `scripts/sweep_gen.py` and every file in `scripts/sweep.d/`,
hashed with **relative** names so two worktrees holding identical generators
produce an identical digest. Four properties, each of which replaces something
the old arrangement needed:

| property | replaces | asserted by |
|---|---|---|
| **content-addressed** — a different `sweep.d` is a different directory | the hazard `expr_sweep.sh`'s header names (#3249): two lanes overwriting each other's corpus | `corpus-digest-separates-corpora` |
| **published by `rename(2)`** into a sibling `.tmp-*` | a reader seeing a half-built corpus; two publishers clobbering | `corpus-concurrent-publish-converges` |
| **immutable after publication** | **the lock** — readers only read, so there is no window | `corpus-published-generation-immutable` |
| **verified by regeneration + full byte compare, every run** | nothing. There was no verification of the corpus at all | `corpus-{short,extra,tampered}-*-refused` |

**Every run still generates its own private corpus first**, exactly as before,
and adopts the shared one **only** if `diff -rq` finds all 19,556 files equal.
That costs 1.2 s and buys the sentence *"the corpus I am grading is byte-identical
to the one my own tree's generator produces"* as a measurement taken on every
run rather than a property inferred from a directory's name.

### 6.1 What is shared, and what is not — the question the dispatch put first

> *"A lane must never be able to pass on a peer's cached result for a tree it
> does not have."*

**Only the case SOURCES are shared, and they are a pure function of the reading
lane's OWN tree** (`sweep_gen.py` + `sweep.d`, both in the digest). Nothing
derived from any tree under test is shared, because:

* **the port is never cached** — `PortC2::compile_to` runs per case, per run,
  against a fresh `PortC2`, from the run's own `pin_harness` copy of the binary;
* **the obj compare runs per case, per run**;
* **the sweep's P0.1 replay runs per case, per run** — a cache HIT still spawns
  `c2host.exe c2.dll` (`w-gateperf` §3.4 item 2, visible in the wibo argv log);
* **what a warm cache serves is `c2`'s own obj and IL bundle**, keyed over source
  bytes, the source argument, flags, cwd, the `cl.exe`/`c1xx.dll`/`c2.dll`
  contents, the wibo version and the cache root — a set that does not and cannot
  contain the port. It is oracle **input**, not a verdict.

**And the cross-lane sharing this enables is not new.** `work/capture-cache` has
resolved through `provenance::main_repo_root()` — *"the same directory from every
linked worktree"* — since board **#181**, and the **878-TU workload scan has run
fully warm in a fresh worktree that whole time**, because its sources live in
`../dc3-decomp`, one path from every worktree. This lane gives the generated
corpus the property this repo's largest instrument already had. What changed is
the **hit rate**, not the trust boundary.

## 7. PROOF THE GATE STILL GOES RED — and still tells port-wrong from cache-wrong

<!-- INJECTION -->

### 7.1 The thirteen refusal arms

`gate.sh --selftest` is **183 cases, 0 failed** (floor raised 170 → 183 in the
same commit). Ten of the thirteen new arms construct a shared generation that is
**wrong** and require the resolver to refuse it:

| the shared generation is… | required | why the arm exists |
|---|---|---|
| **short by one case** | REFUSED, private, cold | the repo's defining defect family is an absence read as a success, and a corpus missing cases is exactly that shape |
| **long by one case** | REFUSED | a superset is a different defect from a subset, and a count-shaped check passes a swap of the two |
| **tampered by one byte**, same names, same count | REFUSED | the only tamper a name/count check cannot see, and the only one that would silently change what the gate grades |
| **unwritable / unreachable root** | private, **rc 0** | this helper may never be the reason a gate row dies |
| **ungenerable** (broken `sweep_gen.py`) | private, **rc 0** | same |
| repaired after a refusal | **adopted again** | a refusal that latches is one nobody can clear, and it would leave a permanently cold box that nothing explains |

The arms drive the **real** `resolve_corpus` against fabricated trees with
`C2RS_CORPUS_ROOT` pointed into the selftest's scratch — no toolchain, no
compiler, and no reimplementation, which would only prove the copy agrees with
itself.

## 8. Gate evidence

<!-- GATEEVIDENCE -->

## 9. THE PER-LANE ARITHMETIC

<!-- ARITHMETIC -->

## 10. Priced and NOT taken

<!-- NOTTAKEN -->

## 11. Estimate vs outcome — the PREREG scored

<!-- PREREGSCORE -->
