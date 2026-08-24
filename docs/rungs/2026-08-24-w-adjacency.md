# The cost protocol's rotation now balances ADJACENCY — and the control that was meant to prove it came back clean, which is the larger finding

    Tag:       w-adjacency
    Slug:      w-adjacency
    Date:      2026-08-24
    Kind:      construct (instrument)
    Outcome:   built
    Fixtures:  none — instrument lane: `scripts/cost_arms.py` only; `crates/` untouched
    Census:    +0 — nothing in `crates/` moved; this lane's fence forbids it
    Record:    this file; prereg `docs/rungs/_2026-08-24-w-adjacency-prereg.md`
    Base:      67f276409   Board: #3521–#3525

> **What this lane can fail on, named before it started.** It is an instrument
> lane, so its failure axes are (i) shipping a design that does not actually
> balance, (ii) shipping a guard that does not refuse, and (iii) claiming a
> validation whose control did not fire. §2 answers (i) by re-deriving the
> counts from the flat sequence rather than from the generator, §3 answers (ii)
> by watching five refusals, and §4's registered P3a answers (iii) by requiring
> the *old* rotation to reproduce the artefact on the same binaries before any
> credit is taken for the new one.

---

## 0. HEADLINE

* **THE OLD ROTATION'S DEFECT HAS A CLOSED FORM.** For `arms[r%n:] + arms[:r%n]`
  over one `n`-round cycle: `a→a+1` occurs **`n−1`** times, `a→a+2` **once**, and
  **every other ordered pair zero** — while the position table is a perfect
  `1/1/…/1`, which is the only table anyone ever printed. **For `n ≥ 4` the
  reverse pair is exactly 0.** So `w-s1bc`'s four-arm run was the worst
  configuration in the record **by construction, not by luck**, and choosing an
  arm count under the old rotation was choosing how large an artefact to accept
  with no way to see it. §2.1, board **#3524**.
* **THE FIX IS BUILT**: a cycle of **`L = 2n`** rounds in which each arm holds
  each slot twice, each ordered pair `a→b` occurs exactly twice, and each `a→a`
  exactly twice. `L = 2n` and the 2-and-2 are **forced by arithmetic, not
  chosen**. The generator is deterministic and is **never trusted** — a separate
  verifier re-derives every count from the flat sequence and the run refuses if
  they disagree. `--rounds 9` over 3 arms is now **REFUSED**, and it is the count
  three of this protocol's four prior readings were taken at. §2, §3, board
  **#3521**.
* **THE ACCEPTANCE TEST IS UNPOWERED, and it says so because it was registered
  to.** The positive control — the **old, defective** rotation with the null in
  the position `w-permute` measured at +0.46 %/71 % — came back **−0.00 %
  [−0.11, +0.10], split 43 %** on a held, idle box. That is a **MISS on P3a**,
  and by the rule registered before it ran, **runs B–E were not taken and
  nothing here claims the fix works.** §4.1–4.3.
* **AND THE CONTROL FOUND SOMETHING BIGGER THAN WHAT IT CONTROLLED FOR.** On the
  **same two commits** `w-permute` measured at **−0.49 / −0.44 / −0.55 %**
  (splits 27–32 %), run A reads **+1.27 % [+1.01, +1.52], split 76 %** — a sign
  flip and a ~1.8-point swing on a quantity that is supposed to be a property of
  the code, **with the run's own null certificate reading clean**. A per-run null
  certificate is **necessary and evidently not sufficient**: a dirty null
  announces itself and this did not. The disagreement is established; its cause
  is **not**. §4.4, board **#3523**.
* **#3468's MEASUREMENT STANDS; ITS *SWEEP* DOES NOT** — found by reading, not
  measuring. Seven of the nine runs this protocol has ever produced had a
  **complete** cyclic rotation, with nulls from −1.08 % to +0.46 % and splits
  40–71 %. Two of the three predecessor runs #3468 swept into "an incomplete
  rotation" (`w-s1bc` 8-over-**4**, `w-s1c2` 6-over-3) were **not instances of
  it**. This **refutes and defers**: it shows completeness is not sufficient and
  establishes no cause. §5, §5.1, board **#3522**.
* **`#3525` — THREE BUILDS OF ONE COMMIT ARE THREE BINARIES**, and the size
  rises **monotonically with the build directory's name length** (+32 B at +2
  characters, +48 at +6). Site: `crates/c2-reference/src/lib.rs:81` bakes
  `env!("CARGO_MANIFEST_DIR")` — **the same line as #3470**, which produces an
  arm that grades nothing and exits 0. Two lanes, two unrelated-looking defects,
  one line. Every lane builds at a path of a different length **by
  construction**, so this has been varying across every cross-lane cost
  comparison ever made here. §4.6. **Measured in 42 seconds with no box.**
* **ALL THREE GATES GREEN AND THE IDENTITY DIFF IS 0 LINES OVER 21 ROWS.**
  `PARTEST_EXIT=0` (1,866 / 0 / 1 over 54 targets), `GATE_EXIT=0` (18/18 lanes,
  7,038 fixture-verdicts, sweep 19,460 of 19,556, cross 90,424 of 90,812, debug
  18/18 at 0 panics, `mismatch 0` everywhere), `AUDIT_EXIT=0` (2,119 rows, 0
  cited-but-not-on-the-board). **`crates/` is untouched, so the diff was run to
  prove an identity rather than to pass a check** — and it is a stronger
  statement than the exit code, because `mismatch 0` is not `match unchanged`.
  §8.
* **Outcome: `built`.** The design half — rotation, closed form, self-test with
  its negative case, five watched refusals — is pure arithmetic with known
  answers and stands on its own. The measurement half **declined to claim**, in
  the words it registered in advance.

---

## 1. THE BRIEF'S CITATIONS

All eleven verified; see the prereg §1. Two are worth lifting out.

**The brief understated the cyclic defect.** It says one 3-arm cycle gives
`base→nulldup ×2`, `nulldup→tip ×2`, `tip→base ×2` and that `nulldup` "never
precedes" `base`. Re-derived mechanically by
`cost_arms.py --show-design 3 --rotation cyclic`:

```text
  adjacency counts over one cycle (circular; rows = predecessor)
        A   B   C            position counts (rows = arm, cols = slot)
    A   0   2   1              A   1   1   1
    B   1   0   2              B   1   1   1
    C   2   1   0              C   1   1   1
```

The reverse pairs are **1**, not 0 — the boundary between rounds supplies them —
so the imbalance is **2-vs-1 per cycle**, not 2-vs-0. The position table beside
it is the point: **perfectly balanced, and it is the only thing #3468's
criterion looked at.**

**The brief's "Williams design, 6 sequences" is the right number for the wrong
reason, and the difference matters at even `n`.** A Williams design balances
adjacency *within* a sequence. Here the round boundary is **not a pause** — the
last arm of round `r` and the first of round `r+1` are two back-to-back
`c2rs perf` invocations with nothing between them — so a Williams-only fix would
leave `1/n` of all adjacencies unbalanced (12 of 36 at 12 rounds over 3 arms):
the same defect at a third of the size. The design here balances the **whole
flat sequence**. For `n = 3` that is also 6, so the brief's number holds; the
general rule is `L = 2n` and it differs from Williams at even `n` (Williams
needs `n`).

---

## 2. THE DESIGN — `L = 2n`, and the counts are forced rather than chosen

The rotation is a cycle of `L = 2n` rounds, each round a permutation of the
arms, with three exact properties over one cycle:

1. each arm holds each slot of a round exactly **2×** — #3468's criterion,
   subsumed and not dropped;
2. each of the `n(n-1)` ordered cross pairs `a→b` occurs exactly **2×**;
3. each of the `n` self pairs `a→a` occurs exactly **2×**.

**Why `2n` and why 2-and-2.** A cycle of `L` rounds has `L·n` adjacencies read
circularly, of which `L(n-1)` lie inside rounds and are necessarily cross pairs.
Balance requires `n(n-1) | cross_total` and `n | self_total`. At `L = 2n` the
within-round pairs alone already supply `2n(n-1)` cross adjacencies out of
`2n²` total, so `cross_each = 2` and `self_each = 2` is the **only** solution —
which forces **every one of the `2n` round boundaries to be a self-repeat, two
per arm**. That is balance, not a defect: each arm gets exactly two warm
restarts per cycle, so the min-over-rounds estimator draws from the same mixture
for every arm. `L = n` admits no design at all for `n = 3` or `n = 4`
(exhaustive DFS, no solution, budget not reached), which is why the cycle is
`2n`.

**The generator is never trusted.** `carryover_cycle(n)` is a deterministic
depth-first search with no RNG — same design on every box, every run — and
`verify_design(cycle, n)` re-derives every count from the flat slot sequence
and is the only thing the runner believes. A disagreement between them stops
the run (`REFUSING: the balanced rotation … does NOT verify`). `--show-design N`
prints the design, both count tables, and the verdict.

| arms | cycle | `--show-design N` verdict |
|---|---|---|
| 2 | 4 rounds | `BALANCED` — cross 2, self 2, pos 2 |
| 3 | 6 rounds | `BALANCED` — cross 2, self 2, pos 2 |
| 4 | 8 rounds | `BALANCED` — cross 2, self 2, pos 2 |
| 5 | 10 rounds | `BALANCED` — cross 2, self 2, pos 2 |
| 6 | 12 rounds | `BALANCED` — cross 2, self 2, pos 2 |
| 7 | — | **`REFUSING: no balanced 14-round cycle found … within 2000000 steps / 40s`**, exit 1 |

Seven arms is a **refusal, not a degradation** — the script does not fall back
to a cyclic rotation, because a silent fallback is the exact defect #3495 filed.

### 2.1 THE OLD DEFECT GETS LINEARLY WORSE WITH MORE ARMS — meet this BEFORE choosing an arm count

Closed form for the cyclic rotation, `arms[r%n:] + arms[:r%n]`, over one
`n`-round cycle. Round `r` is `(r, r+1, …, r+n-1)`, so every within-round
adjacency has the form `a→a+1`; there are `n(n-1)` of them spread over the `n`
distinct such pairs. Each round ends on `r-1` and the next begins on `r+1`, so
every boundary has the form `a→a+2`, one per pair. Everything else is **zero**:

| pair | occurrences per cycle |
|---|---|
| `a → a+1` | **`n − 1`** |
| `a → a+2` | 1 |
| every other ordered pair | **0** |
| `a → a` | 0 |

| arms | counts printed by `--show-design N --rotation cyclic` |
|---|---|
| 3 | 2 / 1 / 0 — and here `a+2 = a-1`, so the reverse pair gets the 1 |
| 4 | **3 / 1 / 0** |
| 5 | **4 / 1 / 0** |
| 6 | **5 / 1 / 0** |

**For `n ≥ 4` the reverse pair is exactly 0.** The arm declared immediately
after another runs directly after it `n-1` times per cycle and directly before
it **never** — and the position table is a perfect `1/1/…/1` throughout, which
is the only thing #3468's criterion ever looked at.

So `w-s1bc`'s four-arm configuration was **the worst in the record by
construction**, not by bad luck: its `nulldup` sat in slot 2 and therefore
followed `base` on three of every four rounds and preceded it on none.

**This is actionable before a run, not after it.** A lane choosing an arm count
under the old rotation was choosing how large an artefact to accept, and had no
way to see it. Under the new rotation the count is 2-and-2 at every `n` the
script accepts, so the choice is free — and `--show-design N` prints it either
way, which is how this table was produced rather than argued.

**One adjacency in `rounds·n` is necessarily unbalanced, and it is the last
one.** The cycle balances as a *circle*; a run is a *line*, so the wrap from the
final arm back to the first never happens. `rounds·n − 1` adjacencies cannot
divide evenly among `n²` pair classes, so some class is short by one; this
design makes it exactly one, which is the floor and not a slack. At `--rounds 6`
over 3 arms that is 1 of 18. Registered in the prereg §2 before measuring, so it
is a stated property and not a discovery.

---

## 3. EVERY GUARD WATCHED REFUSING ON DELIBERATELY BROKEN INPUT

`CLAUDE.md`: *"Before relying on any `--check` flag, watch it fail on
deliberately broken input."* Four refusals, each **exit 1**:

| fed | result | exit |
|---|---|---|
| `--rounds 9`, 3 arms — **legal under #3468, illegal now** | *"not a positive multiple of 6 (the balanced cycle for 3 arms) … Use 12."* | **1** |
| `--rounds 8`, 3 arms — illegal under both | *"… Use 6."* | **1** |
| `--rounds 0` — degenerate | *"… Use 6."* | **1** |
| a null arm that is **not** byte-identical | *"arm nulldup is NOT byte-identical to base: it cannot be the null."* | **1** |
| `--show-design 7` — an arm count with no verified design | *"REFUSING: no balanced 14-round cycle found …"* | **1** |

The first row is the one that matters: **`--rounds 9` over 3 arms is now
refused, and it is the count three of this protocol's four prior readings were
taken at** (`w-s1c3` run 2, `w-permute` runs 1–3).

`--rotation cyclic` still accepts 9, deliberately — it exists to reproduce a
prior reading as a control and prints a warning saying its numbers are not
comparable to a balanced run's.

### 3.1 `--self-test`, and why HALF of it is a negative case

`cost_arms.py --self-test` checks the rotation for 2–6 arms and exits non-zero
on any failure. **It also checks that the CYCLIC rotation is correctly
REJECTED at every one of those arm counts, and that half is not decoration.**

> A self-test in which every case passes cannot distinguish a working verifier
> from one that returns `True`.

That is #1524's lesson and the `/FAsc` control's lesson in one sentence: an
acceptance-only suite is exactly as green when `verify_design` is right as when
it is broken open. The rejection cases are what give the acceptance cases
content — and they are also where §2.1's table comes from, since the rejection
report prints the offending counts.

**Anyone extending this to `n = 7` must add a rejection case, not only an
acceptance case.** Stated here because the acceptance case is the one that looks
like the test.

This is the #1406 tension (*"an instrument whose output is quoted as evidence
should run under `cargo test` or `scripts/gate.sh`"*) **partly** paid. The
timing half still cannot, for the reason the module doc already gives. The
design half is pure arithmetic with a known answer, and now it can — in one
command. It is **not** wired into `scripts/gate.sh`, because that file is
`w-joint`'s fence this wave; that is a fence, not a judgement, and it is the
obvious next step.

---

## 4. THE ACCEPTANCE TEST — **run A, the positive control, CAME BACK CLEAN, and by the rule registered before it ran the test is UNPOWERED**

### 4.1 Run A

`--rotation cyclic --rounds 6`, arm list `base, nulldup, tip` — the null in
position 2, the configuration `w-permute` measured at **+0.46 %, split 71 %** on
an idle box. Arms are `w-permute`'s own commits, rebuilt: base `f6f56df78`,
tip `0ff503eb0`, `nulldup` a `cp` of base with `cmp` exit 0 (the script verifies
it). n = **157**, the same population `w-permute` had.

| | mean | 95 % CI | median | aggregate | slower on |
|---|---|---|---|---|---|
| **`nulldup` — NULL, byte-identical** | **−0.00 %** | **[−0.11, +0.10]** | +0.00 % | +0.13 % | **67 of 157 (43 %)** |
| `tip` | **+1.27 %** | [+1.01, +1.52] | +1.29 % | +1.84 % | 120 of 157 (76 %) |

Box: **load 1.7 at start, 1.8 at end**, `cargo = 0`, `rustc = 0`, no `gate.sh`,
no other `c2rs` at either end — the coordinator held the box and all three peer
lanes confirmed drained. 13 m 19 s wall. Not contaminated, and not a
close call.

### 4.2 P3a is a MISS and the consequence was registered before the run

Registered: *"Run **A** must reproduce the artefact: null mean **> +0.15 %** with
a sign split **≥ 60 %**. If it does not, runs B and C prove nothing, and I will
report the acceptance test as **UNPOWERED** and say so in exactly those words."*

**−0.00 % and 43 %.** It is not merely below the dirty threshold — it satisfies,
to the digit, the **P3b band registered for a good run** (`|mean| ≤ 0.20 %`, CI
containing zero, split in [42 %, 58 %]). **The defective rotation produced a
textbook-clean null.**

**So: THE ACCEPTANCE TEST IS UNPOWERED.** A clean null under the new rotation
would have been worthless when the old rotation is also clean on the same
binaries, on the same box, in the same hour. Nothing in this lane's timing
claims the fix works, and §5's retrospective inherits the same verdict: it is a
**live anomaly with no established cause**, exactly as §5.1 said it would be if
this happened.

### 4.3 What run A DOES establish, stated at the size it is

**#3495's artefact did not reproduce under a deliberate attempt to reproduce
it.** That is a failure to replicate and it is reported as one — the same
service #3495 performed for #3468, arriving one level up again. Three things
differ between run A and `w-permute` run 2 and **none of them is controlled**,
so this identifies no cause:

* **6 rounds, not 9** — this lane's own amendment, made for wall clock;
* **rebuilt binaries** — same shas, same pinned toolchain, same box, same day,
  but built in a different directory, so the embedded `CARGO_MANIFEST_DIR`
  differs and the layout with it;
* **a different hour of a different box state.**

### 4.4 THE LARGER READING IN RUN A — the tip's sign flipped on the same two commits

This is not what the run was for and it is bigger than what it was for.

| | `w-permute` (3 runs) | this lane, run A |
|---|---|---|
| base / tip | `f6f56df78` / `0ff503eb0` | **the same two commits** |
| tip effect | **−0.49 %, −0.44 %, −0.55 %** | **+1.27 % [+1.01, +1.52]** |
| tip sign split | 32 %, 32 %, 27 % | **76 %** |

**Same source, opposite sign, non-overlapping CIs, about 1.8 points apart.** The
one thing that differs is that these binaries were **rebuilt in a different
directory**.

`w-permute` anticipated the mechanism — *"it is at least as likely to be
whole-binary layout and inlining moving under `rustc`"* — but carried it as a
caveat on **attribution**. If this holds, it is more than that: it would mean
the COST CLAUSE's tip readings are **not reproducible across builds of the same
commits**, which puts #3495's −0.55 % and #3468's +0.99 % in the same category
as the null artefact — measuring the build, not the change.

**NOT CLAIMED.** This is one build-pair against one build-pair, `n = 1` versus
`n = 1`, and a third build in a third directory is the cheap test that would
settle it (§7). Recorded here at that size, because a number that contradicts a
published one is worth more in the record than out of it, and worth exactly what
its evidence supports.

### 4.5 A PER-RUN NULL CERTIFICATE IS NECESSARY AND EVIDENTLY NOT SUFFICIENT

This is the sharpest thing the lane measured, and it is a correction to the
doctrine the instrument has carried since #3468.

`cost_arms.py`'s own module doc says: *"What certifies a given run is the null
arm's own reading — CI containing zero and a split near 50 % — and that is a
check to perform per run, never an assumption."* #3495 sharpened it into *"a
null arm is a per-run CERTIFICATE, not a constant of the hardware"*, and every
lane since has read a certified null as licence to quote its tip.

**Run A's null is certified — −0.00 %, CI [−0.11, +0.10], split 43 % — and its
tip contradicts three prior runs of the same two commits in sign.** So the
certificate held and the number was still not comparable across sessions.

**That is strictly worse than a dirty null**, and the reason is structural: a
dirty null *announces itself*, and this did not. A lane that had run only run A
would have seen a beautiful certificate, quoted **+1.27 % [+1.01, +1.52]**, and
been as wrong as `w-permute` would have been quoting **−0.55 %** — and neither
lane could have detected it from inside its own run, because the only
within-run instrument either has is the null, and the null was clean in both.

**What is established is the DISAGREEMENT, not its cause** — the same
discipline §5.1 applies to itself. The candidates are named in §7 with the
experiment that separates them, and none of them is chosen here.

**What follows for the protocol, and it is a doctrine change rather than a
number:** the null certifies a run **against arm-order artefacts within that
run**. It does **not** certify the run's tip reading against anything that
varies *between* sessions and applies to all arms alike — build layout, kernel
or microcode state, an hour of the day. **Cross-session comparability was never
tested by anything this protocol prints, and this is the first run that looked.**

### 4.6 THREE BUILDS OF ONE COMMIT ARE NOT THE SAME BINARY, AND THE SIZE TRACKS THE BUILD DIRECTORY'S NAME LENGTH — same `env!` site as #3470

This cost **42 seconds and no box**, and it was run as a *precondition* for an
experiment that was then deferred. It is a finding on its own.

`f6f56df78` built three times, three directories, same pinned toolchain, same
box, same session:

| binary | md5 | size | dir name |
|---|---|---|---|
| `c2rs-b1` | `e63eb8bd50e4d97b5bb57c0c346edabd` | 6,126,264 B | `b1` (2 ch) |
| `c2rs-b2xx` | `b9a82a769ecd04df850895f16bd4fdcd` | 6,126,296 B | `b2xx` (4 ch) |
| `c2rs-b3yyyyyy` | `84268b6129ec827c59a275a14e104017` | 6,126,312 B | `b3yyyyyy` (8 ch) |

**All three differ, and the sizes rise monotonically with the directory-name
length** — +32 B at +2 characters, +48 B at +6. `rustc` nondeterminism would be
unordered and would correlate with nothing. **This has a mechanism and a site:**

> `crates/c2-reference/src/lib.rs:81`
> `Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")`

The repo root is captured **at compile time**. A longer build path is a longer
string literal, a larger `.rodata`, and a different address for everything
aligned after it.

**AND IT IS THE SAME LINE AS #3470.** One compile-time path capture produces two
failures that look unrelated:

| row | what the `env!` capture does |
|---|---|
| **#3470** (`w-s1c3`) | a binary built in a scratch tree resolves `compilers/` relative to *that* tree, prints `SKIP: toolchain absent`, and **exits 0** — an arm that graded nothing while every stamp read was correct |
| **#3525** (here) | the captured string sets `.rodata` size, so **binary layout is a function of build-directory path length** |

Two lanes, two unrelated-looking defects, one line of code.

**The consequence.** Every lane builds in its own worktree at its own path, and
**those paths differ in length by construction** — `c2-rs-wt-w-permute` and
`c2-rs-wt-w-adjacency` differ by three characters. So if build layout affects
timing, it has been varying across **every cross-lane cost comparison this
project has ever made**: systematically, invisibly, and correlated with nothing
anybody recorded.

**HELD, AND NOT CLAIMED.** *"The binaries differ"* is measured. *"The difference
explains the 1.8-point swing"* is **not** — that is exactly what experiment F
would measure, and F is registered and unrun (§7.3). Two separate claims; only
the first has evidence.

#### 4.6.1 What was BUILT in response, and an opinion on the proposed mitigation

**Built** (`scripts/cost_arms.py`, `arm_identity()`): the protocol now prints
**each arm's md5, size and build directory before anything is timed**, and flags
it when arms differ in size. Three lines of output. §7.6 wanted this as a
convention; a convention nothing enforces is the thing `CLAUDE.md` warns about,
so it is code instead.

**Opinion on padding every lane's build directory to a fixed width** — offered
because the coordinator asked, and it is a reasonable idea that I would rank
last of four:

1. **Build every arm of a comparison in ONE directory** (this lane did, for
   base/tip). It removes the confound *within* a run completely and costs
   nothing. It does **not** help across sessions, which is where the problem is.
2. **Print the identity — done above.** It does not remove the confound; it
   makes it *visible*, which is the read-before-probe move and is why it is
   ranked above removing it blindly. A future disagreement becomes answerable.
3. **Make `repo_root()` resolve at runtime** rather than baking
   `CARGO_MANIFEST_DIR`. This is the real fix, it would close **#3470** as well,
   and it is **outside this lane's fence** (`crates/`). Recommended to whoever
   holds that file.
4. **Padding paths to a fixed width.** It would work mechanically — equal-length
   strings mean an identical `.rodata` size and identical alignment downstream,
   and content differences alone move nothing. But it is **an unenforced
   convention protecting against an invisible failure**, which is the exact
   combination `CLAUDE.md` says produces documentation instead of enforcement.
   It also assumes `CARGO_MANIFEST_DIR` is the *only* path-length-sensitive
   capture, which is untested.

---

## 5. WHAT READING ALONE ALREADY SHOWS: #3468's *sweep* covered runs its
## explanation does not fit

This costs nothing and it was found before a single arm was timed, by reading
the three prior lanes' own protocol paragraphs for their **arm count and round
count** — which is `CLAUDE.md`'s read-before-probe doctrine applied to the
project's own record rather than to `c2.dll`.

#3468's finding is *"the ±1–1.7 % floor is not the box, it is an **incomplete
rotation**"*, and its evidence column sweeps in `w-s1bc` §4.3 and `w-s1c2`
§4.1/§4.3 — *"null arms −1.08 %, +0.47 %, +0.52 %; splits 40 %, 52 %"* — as
instances of the same defect. **Two of those three runs had a rotation that
completed.**

| lane | run | arms | rounds | cyclic rotation completes? | null | split |
|---|---|---|---|---|---|---|
| `w-s1bc` | first (loaded) | **4** | **8** | **YES** (8 % 4 = 0) | **+0.47 %** [+0.12, +0.83] | **57 %** |
| `w-s1bc` | final (quiet) | 4 | 8 | **YES** | −0.08 % [−0.34, +0.17] | 46 % |
| `w-s1c2` | 1 (loaded) | 3 | **6** | **YES** (6 % 3 = 0) | **−1.08 %** [−2.81, +0.65] | **40 %** |
| `w-s1c2` | 2 (quieter) | 3 | 8 | no | +0.52 % [−0.52, +1.57] | 52 % |
| `w-s1c3` | 1 | 3 | 8 | no | +0.57 % [+0.45, +0.68] | 76 % |
| `w-s1c3` | 2 | 3 | 9 | **YES** | +0.09 % [−0.07, +0.26] | 51 % |
| `w-permute` | 1 | 3 | 9 | **YES** | +0.29 % [+0.14, +0.44] | 62 % |
| `w-permute` | 2 | 3 | 9 | **YES** | +0.46 % [+0.32, +0.61] | 71 % |
| `w-permute` | 3 | 3 | 9, list reordered | **YES** | +0.06 % [−0.05, +0.17] | 54 % |

**Seven of the nine runs this protocol has ever produced had a complete cyclic
rotation, and their nulls range from −1.08 % to +0.46 % with splits from 40 % to
71 %.** Completeness predicts nothing. #3468's own *controlled comparison*
(8 vs 9 rounds, same box, same binaries, twenty minutes apart) stands untouched
— it is a real A/B and it found a real effect. What does not survive is the
**generalisation** to the two predecessor lanes: `w-s1bc`'s 8-over-4 and
`w-s1c2`'s 6-over-3 were already position-balanced, so whatever moved their
nulls was never the incompleteness.

And the 4-arm case is the worse one. `--show-design 4 --rotation cyclic` on
`w-s1bc`'s exact configuration:

```text
        A   B   C   D                    A   B   C   D
    A   0   3   1   0                A   1   1   1   1
    B   0   0   3   1                B   1   1   1   1
    C   1   0   0   3                C   1   1   1   1
    D   3   1   0   0                D   1   1   1   1
    adjacency per cycle              position per cycle
```

`A→B` happens **three** times per cycle and `B→A` **zero**. `w-s1bc`'s `nulldup`
sat in slot 2, immediately after `base`, on **three of every four rounds and
never before it** — a stronger version of exactly the relation `w-permute`
reversed. Its two runs are the pair with the largest disagreement in the table
(+0.47 % vs −0.08 %).

### 5.1 THIS SECTION REFUTES AND DEFERS — it establishes no cause, and must not be cited as if it did

**What §5 shows:** a complete cyclic rotation is **not sufficient** for a null
arm to read zero. That is a clean refutation and it rests on arithmetic anyone
can recheck (`8 % 4 == 0`, `6 % 3 == 0`).

**What §5 does NOT show, and is not evidence for:** that *adjacency* is what
explains the spread. Those nine runs differ in **many ways at once** — different
lanes, binaries, corpora, box loads, arm counts, dates — and **nothing in the
table varies adjacency alone**. The adjacency tables above explain how each
configuration *could* carry an order artefact; they do not measure that it did.

This is **#3483's sharpening in its exact shape**: a comparison can prove a
property is not sufficient without licensing any claim about what is. The
generalisation this lane is taking apart — #3468's sweep of two predecessor runs
into an explanation that did not fit them — was built by exactly this move, and
repeating it one level up is the failure mode, not the finding.

**The positive claim is carried by §4 and by §4 only**: runs A and B differ in
`--rotation` and in nothing else — same binaries, same box, same 157-fixture
population, same round count, minutes apart. Everything §5 offers is a reason to
have run that control, not a substitute for it.

**If run A comes back clean**, the registered UNPOWERED verdict covers this
section too: §5 would then be a **live anomaly with no established cause**, and
it is labelled that way rather than quietly retained as support.

**Filed beside #3468, never over it** (`w-r8idiom`'s convention, the same one
#3495 used). #3468 is merged and pushed; its measurement and its controlled
comparison stand untouched.

---

## 6. PREREG SCORECARD — every registered prediction, scored by the rule it was registered under

| # | prediction | outcome |
|---|---|---|
| **P1** | `--show-design N` verifies `BALANCED` with `cross = self = pos = 2` and `rounds = 2N`, for N = 2…6 | **HIT.** All five. Deterministic, no box involved, and claimed as nothing more than arithmetic. |
| **P2** | four guards watched refusing on deliberately broken input, each exit non-zero, message naming the legal count | **HIT, and one better — five.** `--rounds 9` (names 12), `--rounds 8` (names 6), `--rounds 0` (names 6), non-identical null, and `--show-design 7`. All exit 1. §3. |
| **P3a** | run A reproduces the artefact: null **> +0.15 %** with split **≥ 60 %** | **MISS.** −0.00 %, split 43 %. §4.2. |
| **P3b** | runs B and C both give `\|mean\| ≤ 0.20 %`, CI containing zero, split in [42 %, 58 %] | **NOT TAKEN.** The registered consequence of a P3a miss is that B and C prove nothing; they were not run. Recorded as not-taken, **not** as unscored. |
| **P3c** | `\|null(B) − null(C)\| ≤ 0.20` points | **NOT TAKEN**, same reason. |
| **P4** | *"I will re-take `w-s1c3`/#3468's reading only, and decline the other two with a price"*; tip stays positive in [+0.5 %, +1.5 %] | **NOT DONE.** Run E was not taken — the UNPOWERED verdict made a fourth reading of the same instrument worth less than the box time, and the coordinator stopped the sequence. **The binaries are built and pinned** (`e85253cda` / `4d04ee59e`), so the run is ~13 minutes whenever it is wanted. Scored as **not done**, because a prediction about what this lane would do is a prediction. |
| **P5** | run F's build-to-build floor lands in ±0.2–0.7 % | **REGISTERED, NOT RUN** (prereg §8). Its `cmp` precondition **was** checked and the experiment is live — §4.6. Whoever funds it inherits a prediction they cannot fit after the fact. |

**The one prediction this lane made about the world and could test came back
against it.** P3a is the MISS that governs everything downstream, and it was
registered with its consequence attached, so the consequence executed instead of
being argued about afterwards.

**A note on P4, because "not done" is the honest word and "declined" is not.**
The prereg said this lane *would* re-take #3468's reading. It did not. That is a
failure to deliver a registered intention, not a considered decline, and it is
recorded in those terms — the price is in §7.2 and the binaries are on disk.

---

## 7. What was NOT done — priced, not silently dropped

### 7.1 Runs B, C, D and E — NOT RUN, and that was the registered response

The acceptance test's positive control came back clean (§4.2), so by the rule
registered before it ran, B–E were not taken. **This is not an omission; it is
the branch the prereg registered**, and running them would have produced three
clean nulls that certified nothing, because the *defective* rotation produced a
clean null on the same binaries in the same hour.

**Cost to take them later: ~55 minutes of held box** (B, C, D at 13 min each,
E at 13 min plus 2 builds at 14 s). They are worth taking **only after** §7.3
resolves — a clean null under the new rotation means nothing until the
instrument is shown to agree with itself across sessions.

### 7.2 The three historical re-runs — DEFERRED WITH A MEASURED PRICE, and "cannot be re-derived" turned out to be FALSE

The prereg §5 priced `w-s1bc`'s re-run as needing *"a sha hunt"* because its rung
header names its base but not its `s1b` / `s1c` arm commits. **The hunt
succeeded** — `git log 4b19cda28..a9a48c163^2` names them directly:

| prior reading | arms | base | tip(s) | new legal `--rounds` | price |
|---|---|---|---|---|---|
| `w-s1c3` §4.3 / **#3468**, tip +0.99 % | 3 | `e85253cda` | `4d04ee59e` | 6 or 12 (**9 is now illegal**) | 2 builds + 1 run (~13 min) — **binaries already built and pinned by this lane** |
| `w-s1c2` §4.1/§4.3 | 3 | `f53877aa5` | `178423b56` | **6 — its run 1's original count** | 2 builds + 1 run (~13 min) |
| `w-s1bc` §4.3 | **4** | `4b19cda28` | `s1b` **`2aa49d76d`**, S1c **`43b0b7908`** + fixup **`c23ebc17d`** | **8 — its original count** | 3 builds + 1 run of 32 invocations (~27 min) |

**Two of the three are re-runnable at their ORIGINAL round count**, because the
new cycle lengths happen to divide them (`w-s1c2` run 1 was 6 over 3; `w-s1bc`
was 8 over 4, and the 4-arm cycle is 8). That would make **rotation the sole
variable** — the strongest controlled form available. Only `w-s1c3`'s 9-over-3
is now illegal.

**Total: ~53 minutes of held box and ~2 minutes of builds.** Not spent here, on
the coordinator's call, and for a reason that is about evidence rather than
budget: with the acceptance test unpowered, re-running three more configurations
of the same instrument produces more numbers with no established cause.

**And the price is not the binding constraint — see §7.3.1.** All three re-takes
would run against binaries **this lane rebuilt at this lane's path length**, so
each inherits §4.6's unbounded confound. **They are blocked on experiment F, not
on budget**, and taking them first would produce numbers in the same category as
the ones they are meant to audit.

**None of it "cannot be re-derived."** That claim was this lane's own prereg's,
and it is withdrawn.

### 7.3 THE TWO EXPERIMENTS THE TIP DISAGREEMENT NAMES — priced, registered, and NOT run

§4.4 and §4.5 leave a measured anomaly. Two experiments separate its candidates,
and **they are not interchangeable — the first is logically prior to the
second.**

**EXPERIMENT F — the build-to-build floor. ~15 min. FULLY REGISTERED, UNRUN.**
Three **independent builds of one commit** (`f6f56df78`) in three directories,
run as three arms. Every pairwise difference has a true value of **exactly
zero** by construction, so it is a known-answer test that **cannot come back
ambiguous** the way a `tip` comparison can. It yields the constant every cost
claim in this project has been quoted without: **the build-to-build floor.**
Registered in full in the prereg §8 — prediction **±0.2–0.7 %**, with the
three-way conclusion rule written out, including the branch in which
**#3468's +0.99 %, #3495's −0.55 % and this lane's +1.27 % are all inside the
noise of their own builds.**

Its `cmp` precondition was **checked and the experiment is LIVE**:

    c2rs-b1        e63eb8bd50e4d97b5bb57c0c346edabd   6,126,264 bytes
    c2rs-b2xx      b9a82a769ecd04df850895f16bd4fdcd   6,126,296 bytes
    c2rs-b3yyyyyy  84268b6129ec827c59a275a14e104017   6,126,312 bytes

All three differ, and **the sizes track the directory-name length** (+32 and
+16 bytes for +2 and +6 path characters). **So the mechanism is named, not
guessed: it is not `rustc` nondeterminism, it is the embedded
`CARGO_MANIFEST_DIR`** that `crates/c2-reference` takes with `env!`
(`lib.rs:81`, `Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")`). Longer
path → longer string → `.rodata` grows → everything aligned after it moves.

**And that generalises past this lane.** Every lane builds in its own worktree,
at its own path, and **those paths differ in length by construction**
(`c2-rs-wt-w-permute` and `c2-rs-wt-w-adjacency` differ by three characters). If
a build-layout floor exists, it has been varying across **every cross-lane cost
comparison this project has made**, systematically and invisibly, correlated
with nothing anyone recorded.

**EXPERIMENT G — the three-way discriminator. ~14 min. Priced, not registered.**
Rebuild `f6f56df78`/`0ff503eb0` into a **third** directory and re-run run A's
exact configuration. Tip near **+1.27 %** → the swing is session- or
round-count-scoped and the build is exonerated; tip near **−0.5 %** →
build-directory variation alone inverts a published reading; tip elsewhere →
the reading has no stable value at this resolution.

**F FIRST.** G mixes *build* and *change* in one comparison, so several stories
survive any result it returns. And if F already shows three builds of **one**
commit disagreeing by a point, **G has nothing left to discriminate.**

#### 7.3.1 AND P4 — THE #3468 RE-TAKE — IS BLOCKED ON F TOO. On F, not on budget.

§6 scores P4 **NOT DONE** and §7.2 prices it at ~13 minutes with the binaries
already on disk. **That price is real and it is not the reason it should stay
unrun**, and this subsection exists so the next reader does not pick it up as
cheap unfinished work.

**#3468's binaries are on disk because THIS LANE REBUILT THEM, in THIS
directory.** So a re-take would compare two arms whose layout is a function of a
path length that differs from the one #3468 measured under — **the identical
confound §4.6 measured and could not bound.** It would produce a number in the
same category as the number it is meant to check, and **a number that cannot be
compared to the reading it is auditing is not an audit.**

| if F reports | then P4 |
|---|---|
| floor ≈ **±0.1 %** | is meaningful and cheap — take it |
| floor ≈ **±1.5 %** | **should never be run at all**: #3468's reading and its re-take would both be inside the noise of their own builds, and **that** is the finding, not the re-take |

**This is the same shape as the padding objection in §4.6.1**: doing the
cheap-looking thing before the enabling measurement produces something that
*looks* like an answer. The ordering is **F → then P4 becomes interpretable**,
and it is a dependency rather than a queue position.

### 7.4 `--self-test` is not wired into `scripts/gate.sh`

A fence, not a judgement — `scripts/gate.sh` is `w-joint`'s this wave. §3.1.
One line, and it is the obvious next step for whoever holds that file.

### 7.5 Seven or more arms

`carryover_cycle` refuses above 6 arms after its search budget (§2). The design
exists mathematically; the deterministic DFS does not reach it in 40 s. A better
generator — or a table of designs verified by the existing verifier — is the
fix, and the refusal is correct behaviour until then.

### 7.6 `w-permute`'s original binaries cannot be re-run

They were reaped with its worktree. **This is a genuine
cannot-be-re-derived, and it is the only one in this lane** — which is why §4.4
is a disagreement between a rebuild and a published number rather than between
two binaries. Worth a convention: a lane publishing a cost reading should record
the **md5 and the build directory** of every arm, which costs one line and would
have made this an answerable question instead of an open one.

---

### 7.7 A NAMED, PRICED RECOMMENDATION TO WHOEVER HOLDS `crates/c2-reference`: resolve `repo_root()` at RUNTIME, and close TWO board rows from one site

**Not attempted — `crates/` is outside this lane's fence.** Handed over as a
two-defect fix rather than a tidy-up, because that is what it is.

```rust
// crates/c2-reference/src/lib.rs:81
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")   // captured at COMPILE time
}
```

| closes | what the compile-time capture does today |
|---|---|
| **#3470** | a binary built in a scratch tree resolves `compilers/` relative to *that* tree, prints `SKIP: toolchain absent`, and **exits 0** — an arm that graded nothing while every stamp read was correct |
| **#3525** | the captured string sets `.rodata` size, so **binary layout is a function of build-directory path length** (§4.6) |

**Shape of the fix**: resolve at runtime — from `C2RS_REPO_ROOT` if set, else by
walking up from `std::env::current_exe()` (or `current_dir()`) for a marker such
as `Cargo.toml` + `crates/` — keeping the `env!` value only as a last-resort
fallback. It must keep `CLAUDE.md`'s degrade-cleanly rule: absent toolchain
still means `SKIP: toolchain absent`, never a panic.

**Price**: one function, plus the tests that assert a relocated binary still
finds the toolchain — which is exactly the case #3470 discovered by accident and
nothing currently covers. **The hard part is not the code, it is deciding the
search rule**, and it should be decided by whoever owns that crate rather than
by a lane that cannot run its tests.

**Why it is worth funding above the mitigations in §4.6.1**: it is the only
option that removes the *cause*. Everything else this lane did or proposed —
printing arm identity, building arms in one directory, padding paths — manages a
symptom, and the padding option additionally rests on an **untested uniqueness
assumption** (that `CARGO_MANIFEST_DIR` is the only path-length-sensitive
capture in the binary). **A mitigation resting on an untested uniqueness
assumption is worse than no mitigation, because it looks like the problem is
handled** — and that generalises well past this instance.

---

## 8. Gate evidence — and the identity diff is **0 lines over 21 rows**

`crates/` is untouched by this lane (`git diff --stat 67f276409..HEAD -- crates/`
is **empty**), so every count-bearing row is *expected* identical to base. It was
run to **prove** that rather than assume it: an expected-identical result that is
measured is evidence, one that is assumed is a gap.

| gate | result |
|---|---|
| `C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release --no-fail-fast` | **`PARTEST_EXIT=0`** — **1,866 passed / 0 failed / 1 ignored** over **54 targets**, and **0** `SKIP: toolchain absent` lines. Per #3341 the zero is recorded so the number exists and is **explicitly not the evidence**; the evidence is exit 0 under a flag that makes a toolchain-less run fail. |
| `sh scripts/gate.sh --jobs 4 --require-graded` | **`GATE: PASS (HATCH-RED REFUSED)`**, **`GATE_EXIT=0`**. 18 lanes in the registry — **18 PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT**. **7,038** fixture-verdicts, **391/391** on every fixture lane. Sweep **19,460** graded of **19,556** (96 ungraded: the reference rejects the source), **0 mismatch**, corpus 19,556. Cross **90,424** of **90,812** cells, **0 mismatch**. Debug **18/18** lanes through a DEBUG-profile `c2rs`, 7,038 more verdicts, **0 panics**. **`mismatch 0` everywhere.** Graded tree **`1b9559e2d8da`**, 778 files. |
| `sh scripts/board_audit.sh` | **`AUDIT_EXIT=0`** — **2,119** board rows, **259** distinct ROADMAP citations, **CITED BUT NOT ON THE BOARD: 0**, unresolved §-anchors **0**, raw line-number anchors **0**, rows-behind-the-prose **0**, duplicate row numbers **0**. Nine suppressed numbers, each with its printed reason. |

### 8.1 The strict identity diff — denominator printed at BOTH ends

Per `work/coordinator/gatebase/HOWTO_DIFF.md`: cut to `LANE VERDICT
graded/total match` before diffing, normalise `/tmp/c2rs-gate-<pid>` to
`RUNDIR`, drop the two `n/a`-mismatch rows (`hatch-red`, `ladder-red`).

    base count-bearing rows: 21
    tip  count-bearing rows: 21
    diff lines: 0

**0 lines over 21 rows.** The 21 are **enumerated, not asserted** — 18 mode
lanes (`O1`, `O1-EHsc`, `O1-Oi`, `O1-Oi-EHsc`, `Ox`, `Ox-EHsc`, `Ox-Gy`,
`Ox-Gy-EHsc`, `O2`, `O2-EHsc`, `Od`, `Od-EHsc`, `O1-Oi-GR`, `O1-Oi-EHsc-GR`,
`Ox-GR`, `Ox-EHsc-GR`, `Od-GR`, `Od-EHsc-GR`) plus `expr-sweep`, `mode-cross`
and `debug-lane`.

**This is a stronger statement than `GATE_EXIT=0`**, and after #3515 it is the
one that means something: `mismatch 0` is **not** `match unchanged`, and a change
can be `GATE: PASS`, exit 0, mismatch 0 and still cost matched TUs. The exit code
cannot see that class; only this diff can.

### 8.2 Two things in the gate output that are NOT this lane's, named so they are not inherited as ones

* **`hatch-red` REFUSED.** `hatch.py` patches only `crates/c2-il/*`, which this
  lane has not touched — `crates/` is untouched entirely. Board **#3219** /
  `w-joint3` measured this wave that it has been REFUSED for **22 consecutive
  runs since 2026-08-20**. The gate's own liveness counter printed *"1
  consecutive run (first seen 2026-08-24)"* here because `work/` does not follow
  `git worktree add`, so the history file is fresh in a worktree — **the counter
  under-reports from a worktree, and its own text already says an `N` is a floor
  on the true age, never the age.** Not this lane's, and not evidence of
  anything about this lane's tree.
* **The graded tree hash moved: `a33ca5dca59b` (base) → `1b9559e2d8da`.** It
  **must**, and it is not a contradiction of the identity diff. The gate
  content-hashes `crates fixtures scripts`, and this lane changed
  `scripts/cost_arms.py`. `crates/` and `fixtures/` are byte-identical to base.
  **A hash that did not move would have been the finding**, since it would mean
  the gate was not hashing the file this lane exists to change.

### 8.3 A method note this lane owes back to the wave: use a STREAK predicate, and know what it does not buy

Both false "box is drained" reads on 2026-08-24 — the coordinator's at load 31,
and this lane's own at 20:52 — were **instantaneous** samples landing in a gap
between one process exiting and the next starting. The fix is four lines:

```sh
q=0
for i in $(seq 1 N); do
  busy=$(( $(pgrep -xc cargo) + $(ps -eo cmd | grep -c '[g]ate\.sh --jobs') ))
  if [ "$busy" -eq 0 ]; then q=$((q+1)); else q=0; fi     # RESET on any hit
  [ "$q" -ge 12 ] && break                                 # 2 min sustained
  sleep 10
done
```

It distinguishes *"nothing is running right now"* from *"nothing has been
running for two minutes"*, and it would have caught both false reads.

**And the caveat is the more important half, so it travels with the shell:** a
streak predicate proves quiet **over its window, not going forward.** It makes
the *observation* sound; only the queue makes the *window* sound. This lane
measured that directly — a 2-minute sustained-quiet read at **22:11:42** that
`w-latent` invalidated by starting a gate at **22:15**. Had the lane started on
its own correct read it would have collided within four minutes. **Adopt both or
neither**; the predicate is not a substitute for asking the process that can see
the queue.

---
