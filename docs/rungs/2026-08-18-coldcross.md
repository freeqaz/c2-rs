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
               this lane's first commit, before any contention measurement);
               board rows **`#3282`**–**`#3286`**, plus a sixth drafted
               UNNUMBERED (the block was exhausted at five) — see BOARD.md.
               Rebased onto master `de269b644`; `w-sizebracket` merged mid-lane
               and moved no graded byte

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

Six runs of `scripts/mode_cross.sh` per arm, launched simultaneously, `--jobs 16`,
a 3,000-cell stride so the arms are affordable. **Every run in both arms printed
`checked=2939 mismatches=0 graded=2919 ungraded=20 unknown=0`** — identical
counts are what make the rest of this table a statement about speed.

**arm `naive`** — N concurrent runs against ONE **mutable** shared case
directory (`C2RS_CROSS_CASES=<main-repo>/work/mode-cross/cases`, already warm).
This is exactly the arrangement `w-gateperf` §11.1 priced:

| N | outcome | wall |
|---:|---|---|
| **2** | 1 lock-holder · **1 FALLBACK-COLD** | 5 s / 13 s |
| **3** | 1 lock-holder · **2 FALLBACK-COLD** | 6 s / 18 s / 18 s |
| **4** | 1 lock-holder · **3 FALLBACK-COLD** | 13 s / 25 s / 25 s / 25 s |

**`FALLBACKS: N-1 of N`, every time.** The fallback fires, it fires on the first
concurrent lane and every one after it, and `w-gateperf`'s decline was right
about the design it was declining.

**arm `shared`** — the content-addressed immutable corpus this lane ships. Run
inside **one** worktree, so all N *also* contend on the per-worktree case lock —
a strictly harder test than N separate lanes, which would not share that lock at
all:

| N | outcome | wall |
|---:|---|---|
| **2** | **2 SHARED-WARM**, 1 of them after losing the lock | 4 s / 4 s |
| **3** | **3 SHARED-WARM**, 2 of them after losing the lock | 5 s / 4 s / 4 s |
| **4** | **4 SHARED-WARM**, 3 of them after losing the lock | 4 s / 4 s / 4 s / 4 s |

**`FALLBACKS: 0 of N`, every time.**

### 5.3 What the numbers say

**The fallback does not fire, and the reason is not that the lock was removed.**
The per-worktree lock is untouched and it still fires — three of the four runs at
N=4 lost it and say so in their logs. What changed is that **losing it no longer
costs anything**: the loser generates its own private corpus (0.65 s), the
resolver verifies it against the shared generation (1.2 s), and it grades the
shared paths warm like everybody else. The lock went from *deciding whether a run
is cold* to *deciding which directory a run regenerates into and then discards*.

**So the trade `w-gateperf` described — "a cost paid once per lane for one paid on
contention" — is real for a mutable corpus and does not exist for an immutable
one.** Priced out with this lane's own numbers, at 4 concurrent lanes on their
first gate:

| | cost of the four cross legs |
|---|---:|
| per-worktree directories (today) | 4 × 347 s = **1,388 s** |
| naive shared **mutable** directory | 1 × 29 s + 3 × 347 s = **1,070 s** — better on run 1, and **worse from run 2 on**, where per-worktree costs 4 × 29 s and this costs the same 1,070 s again |
| **content-addressed immutable corpus** | 4 × ~29 s = **~116 s**, on run 1 *and* every run after |

The middle row is the one that matters: the naive share is not merely a smaller
win, it is **a permanent tax where the per-worktree arrangement pays once**. That
is the shape `w-gateperf` identified, and it is why the prerequisite it named —
"it needs the regeneration made non-destructive first" — was the right one.

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

A gate that got faster by grading a directory no lane owns is the single change
in `scripts/`'s history most able to make one lane's gate pass on another lane's
evidence. So nothing here is asserted.

A deliberate fault was injected in a scratch commit (`60e6fda0` —
`encode_addi` returning `si.wrapping_add(1)`, so the port emits a real, wrong
instruction word on every case that lowers an `addi`) and reverted (`1d3cab16`).
The release binary's sha256 returns to **`ee52fa6f78c4`**, byte-identical to
before the injection, at the same build path.

`scripts/gate.sh --jobs 16 --require-graded`, **exit 1 in 94 s**:

```
expr-sweep   FAIL  19556/19556  19460   4891  generated cases   <- MISMATCH — the port emitted wrong bytes on 4891 case(s)
mode-cross   FAIL  90812/90812  90424  16214  case-lane cells   <- MISMATCH — the port emitted wrong bytes on 16214 case(s)
debug-lane   FAIL     18/18      1054   1369  DEBUG-profile lanes
Od           PASS    386/386       21      0  /Od
GATE: FAIL — expr-sweep failed: MISMATCH — the port emitted wrong bytes on 4891 case(s)
  *** A MISMATCH IS AN ALARM AND OUTRANKS EVERY OTHER PIECE OF WORK. ***
```

**Read each row's three lines together, because that triple is the whole
argument:**

```
expr-sweep   checked=19556 mismatches=4891  graded=19460 ungraded=96
             cache: hit=19460 miss=96 validated=190 cache-bad=0 (of 19556 cases)
             corpus: SHARED /…/work/corpus/gen-169fae960ed84b63

mode-cross   checked=90812 mismatches=16214 graded=90424 ungraded=388
             cache: hit=90424 miss=388 validated=894 cache-bad=0 (of 90812 cells)
             corpus: SHARED /…/work/corpus/gen-169fae960ed84b63
```

1. **Every one of the 4,891 and the 16,214 was found on cases served out of a
   directory shared with every other lane on this box**, against oracle bytes
   read off a disk. Sharing the corpus did not blunt either row: the counts are
   `w-gateperf`'s own injection figures to the digit (4,891 / 16,214), taken
   before any of this existed.
2. **`cache-bad=0` sits beside 4,891 and beside 16,214.** The run distinguishes
   *"the port is wrong"* from *"the cache is wrong"* — and **the cross could not
   make that distinction before this lane**, because it printed no cache line at
   all (§10.1).
3. **894 + 190 = 1,084 entries were re-captured through the real toolchain
   during this very run and agreed**, while the port was emitting wrong bytes on
   16,214 cells. The two signals are independent, and that is shown rather than
   argued.
4. **The four `/Od` lanes stayed PASS.** At `/Od` the port refuses more, so fewer
   `addi` lowerings reach an obj. A fault injection that reddens everything
   proves less than one that reddens the right things.

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

### 7.2 THE PATTERN BEHIND IT — a corpus-sized quantity trusted on a count

This is the **second** instance in two days, in the same two files, and it is
worth stating as a pattern rather than as two fixes:

| | the quantity | what stood in for checking it | how it failed |
|---|---|---|---|
| **#3264** (`w-gateperf`, 2026-08-18) | how many cases the cross is about to grade | `ls "$cases"/*.cpp \| wc -l` | `ls` blew `ARG_MAX`, `wc -l` read the empty output as **0**, and `cross of 0 cases` sat in a gate row's own headline for months |
| **#3286** (this lane) | **what those cases are** | `[ "$total_cases" -eq 0 ]` — the *positive* check `w-gateperf` added in response to the first | a count cannot see a corpus that is short by one case, long by one, or one byte different at the same name — and the third is the one that would silently change what the gate grades |

**The first fix is what made the second one visible.** `w-gateperf` replaced a
number-that-could-be-a-failure with a number that is positively checked, which is
strictly better and is still a *count* — and a count is the wrong shape of
evidence for a 19,556-file object. Both drivers have regenerated their corpus on
every run since they were written and neither has ever compared it to anything.

The rule this generalises to: **when the thing being checked is a corpus, the
check has to be corpus-shaped.** It cost 1.2 s here, which is the whole reason
the weaker check survived — nobody had priced the stronger one.

## 8. Gate evidence

**Load on this box ran 0.4 → 86 across the session** — three to four peer lanes
were gating concurrently for most of it — so **wall clocks below are labelled
with the load they were taken at and no two runs at different loads are
subtracted from each other.** The load-independent evidence is the counts, and
they are identical everywhere.

| lane | result |
|---|---|
| `scripts/gate.sh --jobs 16 --require-graded`, **base `abc64be3`, first run in a fresh worktree** | **PASS, exit 0, 510 s**, load 0.4→17. lanes 19 · sweep **92** (`hit=1 miss=19555`) · cross **347** · debug 44 |
| `scripts/gate.sh --jobs 16 --require-graded`, **tip, first run** (publishes the shared generation at a new path, so still cold) | **PASS, exit 0, 756 s**, load 40→52. Both rows `corpus: SHARED` |
| `scripts/gate.sh --jobs 16 --require-graded`, **tip, warm** | **PASS, exit 0, 133 s**, load 30→38. sweep 50 (`hit=19460 miss=96 validated=190 cache-bad=0`) · cross 53 (`hit=90424 miss=388 validated=894 cache-bad=0`) · both `corpus: SHARED` |
| **`scripts/gate.sh --jobs 16 --require-graded` in a BRAND-NEW WORKTREE, its FIRST run** | **PASS, exit 0, 157 s**, load 26→86. sweep **40 s, `hit=19460 miss=96`** · cross **54 s, `hit=90424 miss=388`** — **fully warm on a tree that had never compiled a case** |
| **verdict block identity** | the **25-row** block is **byte-identical** across all three: base (cold, pre-change) `diff` tip (warm) `diff` brand-new worktree. `18/18 PASS`, 6,948 fixture-verdicts, sweep `19556/19556 · 19460 graded · 0 mismatch`, cross `90812/90812 · 90424 graded · 0 mismatch` |
| `scripts/gate.sh --jobs 16 --require-graded`, **at the final tip `d6ea788b`** | **PASS, exit 0, 96 s**, load 7→22 — same 25-row block, `cache-bad=0` on both rows, both `corpus: SHARED` |
| `scripts/gate.sh --jobs 16 --require-graded`, **REBASED onto master `de269b644`** | **PASS, exit 0, 86 s**, load 11→22 — the 25-row block **byte-identical to the pre-change base** for the fourth time, `cache: hit=19460 miss=96 validated=190 cache-bad=0` and `hit=90424 miss=388 validated=894 cache-bad=0`, both rows `corpus: SHARED`. `git diff abc64be3 de269b644 -- crates/ fixtures/ scripts/` is **empty**, so the rebase moved no graded byte and the figures above carry |
| `scripts/gate.sh --selftest` | **PASS — 183 cases, 0 failed** (floor raised 170 → 183) |
| the injected wrong emit | **exit 1 in 94 s**, §7 |
| `C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release --no-fail-fast` | **1,666 passed / 0 failed / 45 targets**, exit 0 — master `abc64be3`'s own reading, unmoved |
| `scripts/debug_lane.sh` | `DEBUG-LANE-TOTAL lanes=18 ran=18 failed=0` |
| `scripts/board_audit.sh` | **all-zero**, exit 0, with the five new rows — 0 unresolved anchors, 0 raw line anchors, 0 rows behind the prose, 0 duplicate row numbers, 0 cited-but-not-on-the-board |
| `rung_registry` | **2 passed / 0 failed**; `INDEX.md` regenerated by `scripts/gen_rung_index.sh` |
| 878-TU workload scan | **394 anchored keys** (`grep -cE '^ *gap-metric \S+ \S+$'`) · `match 26 · mismatch 0 · codegen-gap 0 · vocab-gap 844 · capture-fail 8` · `fnbyte-exact 35899 · fnbyte-differs 1958 · fnbyte-denominator 162046` |

**`scripts/` is in `GRADED_DIRS`, so the graded-tree hash moves by construction**
(board #3215) and is not quoted as evidence in either direction. **No
cross-worktree binary sha comparison is made** (#3224); the two sha comparisons
here are of binaries built at the *same* path, which is that board's own stated
precondition.

### 8.0 WHAT THE IDENTITY DIFF ESTABLISHES, AND WHAT IT DOES NOT

`docs/rungs/README.md` gained a section on 2026-08-18, after this lane's base and
while it was running, that names the exact evidence this rung leans on:

> *"A zero-cost sweep, a split-half agreement, and **a required-zero identity
> diff** are all compatible with a rule that is wrong about c2 four times in
> ten."*

It applies here and the honest reading is worth writing down, because a
25-row byte-identical verdict block is a persuasive-looking artefact.

**What the identity diff establishes:** the shared corpus is the *same corpus* —
the same 19,556 cases reached the same 18 lanes and produced the same 90,424
graded cells and the same 6,948 fixture-verdicts. That is a statement about
**reach**, and reach is precisely what this lane had to hold constant, because a
faster gate that grades *less* is the failure the dispatch names first.

**What it does not establish:** that the gate can still *see* a wrong emit
through a shared, cached oracle. **Nothing in the identity diff could tell those
apart** — a run that had quietly stopped grading would produce an identical block
if it also stopped counting. That question is answered by §7 and only by §7:
a real fault, injected, reddening both rows through the shared corpus at
`cache-bad=0`, with 1,084 entries re-captured through the real toolchain agreeing
in the same run. **The identity diff is the coverage claim; the injection is the
correctness claim; neither substitutes for the other**, and this lane would have
been unsound with either one alone.

**And the residue, stated rather than left implicit:** the shared generation is
verified byte-for-byte against a generation from the reading lane's own tree, so
the only way both can be wrong together is if `sweep_gen.py` itself is wrong.
That is out of this lane's scope and unchanged by it — `gate.sh --selftest`'s
four pre-existing `corpus-*` shape arms are what grade the generator, and they
are untouched here.

### 8.1 The scan identity is stronger than a diff, and it is stated as what it is

`w-gateperf`'s pattern is to build a base binary and diff two scans. **That
comparison is vacuous here and saying so is more honest than performing it**:
`git diff abc64be3..HEAD -- crates/ fixtures/` is **empty**, so the base and tip
binaries are the same program, and the tip binary in this worktree hashes to
`ee52fa6f78c4` — the value it had when it was built at `bdf23a2f`, this lane's
first commit, which is base plus a PREREG file. **The binary never moved**, so a
two-ended scan would have diffed a file against itself. The scan above is run
once, at the tip, and its 394 keys are compared against the dispatch's
registered 394. `fnbyte-exact 35899` is `w-dataseam`'s reading rather than
`STATUS.md`'s 35,897 and is **#3249/#3238's ±2, attributed and not adjusted**.

**The key count is `394` under the anchored pattern.** `grep -c 'gap-metric'`
reads 396 and two of those lines are prose that *points at* keys rather than
being keys — `w-fence163` §3.1 settled this, `w-corpushealth` caught itself
reaching for it, `w-gateperf` reached for it and manufactured a cause (#3269).
This lane used the anchored pattern from the start, which is the only thing that
row asks of anybody.

## 9. THE PER-LANE ARITHMETIC

The dispatch asked for this in one specific form — *"a 1,261 s cold leg is ~94 %
of a lane's entire gate cost across its lifetime if the lane gates twice"* — and
the honest answer moves, because §2's number moves.

**With `w-gateperf`'s warm gate (78 s, `--jobs 16`, low load) as the reference
and this lane's measured cold gate (510 s):**

| | before this lane | after |
|---|---:|---:|
| a lane's **first** gate | **510 s** | **157 s** (measured in a brand-new worktree, under 3–5× the load) |
| every gate after | ~78–133 s | ~78–133 s |
| **lifetime, a lane that gates twice** | **588 s** | **~235 s** |
| the cold excess, as a share of that lifetime | **432 / 588 = 73 %** | — |
| of which is **shareable** (the two generated rows) | **381 s = 88 % of the excess** | removed |
| **what the lane removes, as a share of a twice-gating lane's whole gate cost** | — | **381 / 588 = 65 %** |

**So: 73 %, not 94 %** — and the difference is entirely §2's `--jobs 4` / `--jobs
16` correction, not a disagreement about method. Restated in the dispatch's own
words with the corrected input: *a fresh worktree's cold legs are ~73 % of a
twice-gating lane's entire gate cost, and this lane removes 88 % of them.*

**The single fairest before/after in this document is the pair of first-gate
runs**, because both are "the first gate in a worktree created by
`scripts/setup_worktree.sh`", nothing else about them differs, and the *after*
was taken at load 26→86 against the *before*'s 0.4→17:

> **510 s → 157 s, 3.2×**, with a 25-row verdict block that is byte-identical
> between them, and with the two shareable legs going **439 s → 94 s, 4.7×**.

**And the cost side, stated plainly.** The saving is not free the first time a
*corpus* is seen: publishing a new generation puts the cases at a new path, so
that run is cold (measured: 756 s, §8). What the lane changes is the
**denominator of that cost** — it moves from **once per worktree** to **once per
distinct corpus content, per box, ever**. On a box that has run tens of
worktrees against one `sweep.d`, that is the entire difference.

## 10. Priced and NOT taken

**1. Sharing `fixtures/cpp/` the same way — 50 s, NOT taken, and the reason is
not the arithmetic.** §3 leaves 50 s of cold excess on the two fixture-driven
rows (the 18 mode lanes, 17 s; the debug row, 33 s), cold for exactly the same
reason: 386 hand-written fixtures at a path with the worktree in it. The
mechanism would transfer unchanged — a digest over `fixtures/cpp/` would give a
lane that edits a fixture its own generation, correctly. **It is declined on
tree integrity, not on safety.** `fixtures` is in `GRADED_DIRS`; the gate's
identity is a content hash over `crates fixtures scripts`, and pointing a gate
row at a *copy* of the fixtures outside the tree decouples "what was graded"
from "the hash that says what was graded". The generated corpus has no such
problem because it is not in the tree at all — it is a function of `scripts/`,
which **is** hashed. 50 s does not buy that.

**2. The per-process cache context (`w-gateperf` §11.2 item 2) — PRICED WITH THE
CORRECTED PROFILE, NOT taken.** `CaptureCache::new` runs `wibo --version` and
FNV-hashes 3.16 MB of `cl.exe` + `c1xx.dll` + `c2.dll` **once per case** for
`c2rs diff`, i.e. 19,556 times. `w-gateperf` sized it at ~4 ms of a 17 ms warm
case — *"~15 % of the warm sweep leg"*. Against **this** lane's numbers that is
~4.4 s of a 29 s sweep leg and **5.6 % of a 78 s warm gate**, and the change is a
memoised toolchain digest inside the one file whose module docs are most careful
about exactly this (*"the key is a hash over **inputs**, never over mtimes"*).
**Five seconds is not the price of touching the cache key's soundness**, and it
is outside this lane's `scripts/` seam besides. It wants its own lane with a
stated invalidation rule, which is what `w-gateperf` said and this lane agrees
with after re-pricing rather than by deferring to it.

**3. A garbage collector for `work/corpus/` — NOT taken, and the bound is stated
instead.** One directory per distinct corpus content, 19,556 files, 12 MB
apparent / 83 MB allocated. A generation appears only when `sweep_gen.py` or
`sweep.d` changes. **It is safe to `rm -rf` any generation at any time**: the
resolver re-verifies the whole corpus by byte compare on every run, so the worst
case is one loud, cold, correct run that regenerates it in 0.65 s — which is a
property `work/capture-cache` (#3265) does not have and is why that one needs a
policy and this one does not. `corpus: SHARED` prints how many generations exist,
so the count is visible rather than discovered.

**4. `hatch-red` still reads `REFUSED HATCH-STALE` on every run of this lane**,
on both trees, exactly as `w-gatewire` §10 and `w-gateperf` §11.2 item 4 report
(boards **#1389**/**#3219**). Not caused here, not fixed here, duration still
unmeasured.

### 10.1 Found on the way, and TAKEN — the cross row had no cache validation

`w-gateperf` gave the sweep a standing bypass-and-compare validator (~190
re-captures per run) and made `poisoned`/`foreign` a hard red, precisely because
that row had just acquired a dependency on cache integrity. **The mode cross has
had that dependency since 2026-08-04 and never acquired the validator**, and it
prints no `cache:` line at all — which is why `gate.sh`'s selftest carries an
explicit *"an ABSENT cache line is not a zero and not a failure"* case for it.

That asymmetry was tolerable while a fresh worktree's cross was ~100 % misses. It
is not tolerable now that this lane takes it to ~100 % hits, which is the
honest cost of what shipped here. So the cross row now passes
`--validate-cache` to each of its 18 `c2rs gap` batches and prints a `cache:`
line **in `expr_sweep.sh`'s exact spelling** — so `gate.sh`'s existing
`sweep_verdict`, which already rules both rows and already has four selftested
cases for the clean / poisoned / cold / absent states, reddens the cross on
`cache-bad > 0` with **no change to `gate.sh`'s decision logic at all**.

Measured at the tip, warm: `cache: hit=90424 miss=388 validated=894
cache-bad=0 (of 90812 cells)`. **`hit + miss` = `graded + ungraded` exactly**,
the same identity the sweep's line has.

**And it is nearly free, A/B'd back to back in one session at one load rather
than across runs**, `--jobs 16`, four runs alternating, every one printing
`hit=90424 miss=388 cache-bad=0`:

| `C2RS_CROSS_VALIDATE` | wall | validated |
|---:|---:|---:|
| 0 | **29 s** | 0 |
| 100 | **32 s** | **894** |
| 0 | **29 s** | 0 |
| 100 | **31 s** | **894** |

**894 real `cl.exe`-under-wibo re-captures for 2–3 s**, ~9 % of the leg, because
they ride on `gap`'s own `--jobs`. (The 29 s here also reproduces
`w-gateperf` §13.1's warm cross to the second, which is the check that this
lane's own warm figures are comparable to that lane's at all.) The
`--validate-cache` trap `w-gateperf` documented does **not** apply here and the
difference is worth stating: `c2rs diff` performs exactly one capture per
process, so an in-process `--validate-cache N` tests `1 % N` and validates
nothing for any `N > 1`; each `c2rs gap --list` batch here carries thousands of
cases in one process, so its counter reaches `N`. **The count is printed for
that reason** — a validator whose count is not published is one nobody can tell
apart from a disabled one, and the driver says so in words when the hits exceed
the stride and the validator re-captured zero.

## 11. Estimate vs outcome — the PREREG scored

Frozen at `bdf23a2f`, this lane's first commit, before any contention
measurement. 23 rows; nine facts were declared KNOWN-at-freeze (K1–K9) so the
prereg cannot claim to have discovered them — **K8 and K9 are measurements** (the
0.65 s regeneration and its determinism) and are declared rather than predicted,
because they were taken before the file was written.

| # | registered | outcome | |
|---|---|---|---|
| **P1** | the naive share's fallback fires — **3 of 4** at 4 concurrent lanes, p = 0.88 | **3 of 4**, and **1 of 2** and **2 of 3** besides | **HIT**, exact on the point |
| **P2** | …and it is a wash or a loss on any lane that gates more than once, p = 0.70 | worse: the naive share is a **permanent** 3 × 347 s tax where per-worktree pays 4 × 347 s **once** | **HIT** |
| **P3** | the fallback is an artefact of the lock's SCOPE; with the corpus immutable the count at 4 lanes is **0**, p = 0.80 | **0 of 2, 0 of 3, 0 of 4** — and 3 of the 4 lost the lock anyway and were warm | **HIT**, and the "lost-lock=yes, warm" column is the cleanest statement of why |
| **P4** | the lock-hold window under the fix is < 5 s (point 1.5 s), a ≥ 20× reduction | **no lock is taken at all** on the shared corpus; the per-worktree lock is unchanged and its loss is now free | **HIT on the consequence, WRONG on the mechanism I predicted.** I registered "shrink the window" and shipped "there is no window", which is a better answer than the one I priced |
| **P5** | the overall answer is that sharing WINS and I ship it, p = 0.70 | shipped | **HIT** |
| **P6** | a fresh worktree against a warm shared case dir reads 15–150 s, point 35 s | **29 s** | **HIT**, 17 % off the point |
| **P7** | the same holds for the sweep, point 30 s, interval 15–130 s | **40 s** in the brand-new worktree, `hit=19460 miss=96`, at load 26→86 | **HIT** |
| **P8** | seeding by copy/reflink saves **0 s**, and I can demonstrate rather than assert it, p = 0.90 | demonstrated, and **by a line the base gate already printed** (`hit=1 miss=19555` on a byte-identical corpus) — no experiment was needed | **HIT**, and cheaper than registered |
| **P9** | `/home` is btrfs so reflinks exist, and they are nonetheless the wrong tool | btrfs confirmed; wrong tool confirmed | **HIT** |
| **P10** | I adopt content-addressing rather than a mutable dir with write-if-differs, p = 0.60 | content-addressed | **HIT** |
| **P11** | content addressing closes #3249's hazard **by construction**, so the shared design is *safer* on that axis, p = 0.85 | two `sweep.d`s cannot address one directory; arm `corpus-digest-separates-corpora` | **HIT** |
| **P12** | an injected wrong emit still reddens both rows through the shared corpus, with `cache-bad=0` beside a non-zero mismatch, p = 0.95 | 4,891 and 16,214, `cache-bad=0` on both, `corpus: SHARED` on both | **HIT** |
| **P13** | I introduce no new unbounded cache and I state the bound; the shared paths *reduce* keys minted per lane, p = 0.85 | one directory per corpus content; ~110,000 fewer capture-cache keys and 166 MB fewer corpus copies per worktree | **HIT** |
| **P14** | everything I ship is coverage-preserving; nothing coverage-reducing, p = 0.90 | and two things are coverage-**increasing**: the corpus byte compare (§6) and the cross's cache validator (§10.1) | **HIT** |
| **P15** | the cold excess is **88–93 %** (point 90 %) of a twice-gating lane's lifetime gate cost, p = 0.60 | **73 %** | **MISS**, and it is the useful one — I registered the dispatch's framing with my own slightly lower number, and the real cause was that the 1,261 s input is a `--jobs 4` figure (§2). **I predicted the wrong direction of my own doubt** |
| **P16** | per-lane saving ≥ 1,100 s on a first gate, point 1,250 s, p = 0.75 | **353 s** (510 → 157), or **345 s** on the two shareable legs | **MISS, badly**, and by the same single cause as P15: I priced the saving off the 1,261 s number instead of measuring the cold gate first. The *ratio* I would have predicted (3.2× on a first gate) is intact; the absolute seconds were never mine to predict |
| **P17** | `Outcome: built`, p = 0.65 | `built` | **HIT** |
| **P18** | end-state gate PASS with sweep 19,556 / 19,460 / 0 and cross 90,812 / 90,424 / 0, digit-identical to base, p = 0.90 | the whole **25-row** block is byte-identical at three points | **HIT** |
| **P19** | suite 1,666 / 0 / 45, p = 0.80 | **1,666 / 0 / 45** | **HIT** |
| **P20** | scan identity over **394** anchored keys, 0 changed, `fnbyte-*` ±2 per #3249, p = 0.85 | **394**; and the identity diff is **vacuous by construction** — `crates/` is byte-identical to base and the binary sha never moved, so §8.1 states that instead of performing it | **HIT on the number, and the method was replaced by a stronger one** |
| **P21** | I do NOT take the per-process cache context item, p = 0.70 | not taken, and **re-priced** against this lane's profile: 5.6 % of a warm gate, not "~15 % of the sweep leg" of a gate that no longer looks like that | **HIT** |
| **P22** | I find ≥ 1 further defect or absence-read-as-success nobody filed, p = 0.55 | **three**: the cross row has consulted the cache unchecked since 2026-08-04 (§10.1); neither generated row ever verified its corpus beyond `count > 0` (#3286); and the `1,261 s` figure is a `--jobs 4` number quoted against a `--jobs 16` recommendation (§2) | **HIT**, three times |
| **P23** | `hatch-red` still reads `REFUSED HATCH-STALE` on every run, p = 0.90 | it did, on every run of this lane | **HIT** |

**Twenty-one hits, two misses, and both misses are the same mistake:
P15 and P16 are the two rows that took a number out of the dispatch instead of
measuring it first.** Every row that describes something I went on to *measure*
(P1–P3, P6–P9, P12, P18, P20) hit, including three interval rows inside their
points. The one row worth reading twice is **P4**, which is scored a hit on its
consequence and a miss on its mechanism: I registered "shrink the lock's window"
and the answer turned out to be "delete the window", which is the same finding
one step further along and is not something the prereg gets credit for.

**And the shape of the P15/P16 miss is exactly `w-gateperf`'s own P1/P23.** That
lane recorded *"I predicted a WARM box and measured a COLD worktree"*; this one
predicted against **that lane's cold number at a flag nobody runs any more**. In
both cases the prereg was written against a figure inherited from a document
rather than one taken from the tree, and in both cases the first measurement of
the lane moved it by 3–4×. **The rule that would have caught both is the same:
take the base measurement before freezing the prereg, not after.**
