# w-mutcensus — deviations, corrections, and hazards, carried forward

Every item here is kept on the page because the campaign's colours are only as
good as the conditions they were read under.

## D1 — The runner ran in TWO DETACHED SIDECAR WORKTREES, not in the lane checkout

**Deviation from the first session's single-worktree plan, taken deliberately,
and it is the structural fix for the `w-bind16` hazard rather than a shortcut.**

The lane's branch checkout is
`.claude/worktrees/w-mutcensus` (branch `wt-w-mutcensus`). The 56 remaining
mutants ran in two worktrees detached at the lane tip `662e5c5d`:

| worktree | ids | count |
|---|---|---|
| `.claude/worktrees/w-mutcensus-b` | `CS2`…`CS12`, `G1`–`G3`, `BU1`–`BU3`, `D1`, `D2`, `B2`…`B10` | 28 |
| `.claude/worktrees/w-mutcensus-c` | `L4`…`L9`, `CA2`…`CA23` | 28 |

Three reasons, in order of weight:

1. **It closes the `w-bind16` stale-index hazard by construction.** `w-bind16`'s
   first mutant read a **false RED** because its own uncommitted doc had made
   `docs/rungs/INDEX.md` disagree with `scripts/gen_rung_index.sh`, and
   `crates/c2-harness/tests/rung_registry.rs` fails on exactly that. In this
   layout the mutant runs read `docs/` **at the frozen commit `662e5c5d`** in a
   worktree that nothing edits, while all doc authoring happens in the branch
   checkout. A doc edit *cannot* colour a mutant here, rather than merely being
   unlikely to.
2. **It makes committing safe while the campaign is live.** In a single
   worktree the tree carries an applied mutation for ~4 of every ~4.5 minutes,
   so any `git commit` risks committing the mutant. With the branch checkout
   never mutated, interim tables were committed during the run without a window
   in which a mutation could be staged.
3. Parallelism: two runners at ~4 min per mutant finished 56 sites in ~2 h
   instead of ~3.7 h serial. Separate worktrees mean separate `target/` dirs,
   so the two `cargo test --workspace --release` runs do not contend for the
   build lock.

**The cost, stated:** each sidecar needed its own cold release build, and each
therefore had to be *validated against the frozen baseline before its colours
counted*. Both were. `N0` (prereg §2.1's clean-tree control, registered GREEN
at 1,648 / 0) was run first in **each** sidecar:

* `w-mutcensus-b` `N0` = **1,648 passed / 0 failed / 42 targets** — HIT
* `w-mutcensus-c` `N0` = **1,648 passed / 0 failed / 42 targets** — HIT

The drivers refuse to run a single mutant unless their `N0` reads exactly
1,648/0/42 (`drive.sh` in each sidecar, `exit 2` otherwise), so a
mis-provisioned sidecar could not have produced a colour. The registered N0 is
therefore observed **twice**, on two independent cold builds, in addition to
the first session's measurement in the lane checkout.

No mutant artifact left `work/w-mutcensus/`; the sidecars' logs were copied into
the lane checkout's `work/w-mutcensus/results/` and nothing else was taken from
them. The sidecars are detached, so nothing they contain can reach the branch.

## D9 — The differential-duration probe misread a GRADED RED as INVALID, and the derived table is why all three rule bugs were recoverable

D6's fix reads the `census_gate` target's duration to prove the run graded. The
first spelling took a fixed `grep -A3` window after the marker test name. **When
that target's own test FAILS, cargo prints a `failures:` block before the
`test result:` line, so the line falls outside the window, the duration reads
`absent`, and the run is classified INVALID.**

The run that exposed it is the most informative in the campaign: `L4` came back
**1,646 / 2 / 42** with

* `leaf_store::tests::every_bind_gate_fires_on_a_named_input`, and
* `census_gate::the_census_and_the_port_agree_about_what_is_in_class`

— i.e. **the real-`c2` differential itself caught the mutation.** That is a
maximally-graded RED, and the rule meant to certify grading threw it away. Fixed
to take the first `test result:` line at or after the marker however far away it
is (an `awk` state machine, not a line window).

**This is the same class of bug as D3** (the INVALID predicate matching cargo's
own `error: test failed, to rerun …` line, which mislabelled `C1`'s genuine RED).
Twice now, a rule written to *exclude bad runs* excluded a good one, and in both
cases the direction was the same: **a guarded site read as unmeasurable**, which
deflates RED and would have been quietly reported as a smaller denominator rather
than a wrong colour.

**Why all three rule corrections (D3, D6, D9) were recoverable, stated as the
design decision it was:** `results/summary.tsv` is **derived** from the logs by
`rederive.sh`, never accumulated as ground truth. The logs are the source of
truth and the colour rule is a pure function over them, so every rule fix
reapplies to **every run already on disk** — including runs from a previous
session. Had the table been the primary record, D3's mislabel, D6's false GREENs
and D9's discarded RED would each have required re-running the mutant. **A
campaign that emits its own conclusions incrementally cannot correct its own
classifier; one that derives them can.**

A corollary worth stating because it was nearly got wrong: the *runner's* copy of
the colour rule is **advisory**. The published table comes from `rederive.sh` in
the lane checkout over the collected logs, so the eight in-flight runners did not
need patching mid-run — and overwriting a script that a running `sh` is still
reading is a real hazard that was taken for no benefit.

**IT DID BITE, and this paragraph is the correction.** The first version of this
entry said *"it happened not to bite (all eight runners verified alive and
advancing afterwards)"*. That check asked the wrong question: the runners **were**
alive — the damage was to the run each was in the middle of. **`B9` came back
`INVALID` at 7 of 42 targets**, with `cargo test` stopping straight after the 6th
target and **no error line of any kind**. That is the signature of a shell whose
script changed under it, not of a test failure and not of an OOM (no `dmesg` OOM
entry; 56 GB still available). `/bin/sh` reads a script incrementally and seeks
within it, and `cp` truncates and rewrites in place, so the running shell's file
offset lands in different text than it was parsing.

Damage audit, done properly this time — **every** log in **all eight** sidecars
checked for its target count: **`B9` is the only completed run affected.** Every
other short log had an active writer (6/42 or 14/42 and still growing). `B9` is
recorded `INVALID`, **never as a colour**, and was re-run from scratch.

The ledger on this hazard: **no benefit, one destroyed run, one contended re-run.**
**Never `cp` over a script a live shell is executing.** If a running runner must
change behaviour, write a new file and start a new runner.

## D8 — A KILLED RUNNER LEAVES ITS MUTANT APPLIED, and the next thing that ran measured it. Both guard layers caught it, on two worktrees, by two different mechanisms

**This is the `w-bind16` stale-state hazard reproduced live, from a cause nobody
had written down, and it is the campaign's best evidence that the layered guards
work.**

When D6's fault was found, the two runners were stopped with `pkill`. **`pkill`
kills the runner between its `apply` and its `revert`, so the mutation stays on
disk.** The reset then deleted the partial log (`rm results/*.log`), which removed
the only visible trace, and the relaunched drivers began their `N0` baselines in
worktrees that still carried a mutant:

| worktree | mutant left applied | site |
|---|---|---|
| `w-mutcensus-b` | **`CS4`** | `census.rs:1263`, `bind_key.unwrap_or(...)` dropped |
| `w-mutcensus-c` | **`L6`** | `leaf_store.rs:2390`, `lits.len() > 1` → `> 9` |

**Both were caught before a single colour was emitted, by two different layers:**

* **`w-mutcensus-b`** — the baseline read **`1,648 / 0 / 42`, differential
  267.23 s**, i.e. **a perfectly clean-looking baseline on a mutated tree**,
  because `CS4` happens to be unguarded. The driver's baseline gate passed it and
  the census pre-flight passed it. What caught it was `run_mutants.sh`'s
  **dirty-tree invariant**: `ABORT before CS2: tracked tree dirty`, naming
  ` M crates/c2-il/src/func/census.rs`.
* **`w-mutcensus-c`** — the baseline read **`1,647 / 1 / 42`**, because `L6` *is*
  guarded, and the driver's own gate stopped it:
  `WTC BASELINE MISMATCH — refusing to run mutants`.

**The lesson that generalizes, and it is D6's lesson pointing the other way.**
D6 showed the registered totals `1,648 / 0 / 42` cannot detect a *missing
toolchain*. `w-mutcensus-b` shows the same totals cannot detect an *applied
unguarded mutation* either — a tree carrying a live source mutation reproduced
the registered baseline **exactly**. So the prereg's probe definition is blind in
two independent directions at once, and for one reason: **it is a pair of totals,
and both faults preserve the totals.** The only things that caught either were
checks on something *other* than the counts — a per-run duration, and a
`git status`.

**Fixes, and what was discarded.** Both leftover mutants were reverted with
`git checkout -- crates fixtures scripts` and both worktrees verified clean. Both
baselines are discarded as `N0wt{B,C}.dirtytree.DISCARDED.log`, with the runner
transcripts kept as `drive.dirtytree-incident.out`; **neither is a colour and
neither is used as one.** `CS4` reading `1,648 / 0` on that accidental tree is
*not* recorded as `CS4`'s colour — it was not a registered mutant run, and D4's
rule (a colour comes only from a complete, intended run) governs. `CS4` was
re-run properly.

The driver now **asserts a clean graded tree before it does anything at all**
(`exit 4` otherwise). `run_mutants.sh` already guarded every mutant; the gap was
that nothing guarded the *driver's own baseline*, which ran first.

**And the trailing-baseline change, stated as the budget decision it is.** The
re-launched `b`/`c` drivers run their **mutant list first and the `N0` baseline
last**, recovering the ~28 minutes of contended baseline the incident cost.
Justified because the per-run `census_gate` duration check (D6, layer 3)
validates that **every individual mutant run** graded against real `c2` — which
is the job `N0` was doing as a proxy — and because `d`/`e` produce clean `N0`
baselines from untouched worktrees regardless. What the trailing `N0` still adds
is the assurance that no worktree had a spurious standing failure inflating
`failed` on every mutant, so it is still run, just last; if a trailing `N0` is
not `1,648 / 0 / 42`, every colour from that worktree is suspect and this page
says so.

## D7 — FOUR runners, not two, and the two extra work the same lists BACKWARDS

The prereg budgeted **~6 min per mutant serial** (§3), which was measured on an
otherwise idle box. During this session **two peer sessions were running
`scripts/gate.sh --jobs 4 --require-graded` concurrently** (`/tmp/c2rs-gate-*`
work dirs, `w-npos` among them), and load average sat at **69–75 on 32 cores**.
The effect on this campaign's probe, measured: the `census_gate` target took
**267.23s** where the uncontended baseline took **84.17s** — 3.2×. At that rate
28 mutants per runner is ~5 h.

Adding runners was the right response *for this workload specifically*, and the
measurement is why rather than a guess. Per-target durations sum to **200.11s**
against a **223s** wall baseline, so `cargo test` runs the 42 targets
essentially **serially**, and the individual `wibo` processes each held
**6–9 % of one core**. A suite run here is **latency-bound on serial `wibo`
invocations, not CPU-bound** — so a second, third and fourth concurrent suite
buys nearly-linear throughput without adding much pressure to the peers'
CPU-bound gate runs.

So two more sidecars (`w-mutcensus-d`, `w-mutcensus-e`) were provisioned the
same way, each verified `4/4 functions in class`, and launched on the **same two
id lists in REVERSE order**:

| runner | list | order |
|---|---|---|
| `w-mutcensus-b` | `CS2`…`B10` | forward |
| `w-mutcensus-d` | `CS2`…`B10` | **reverse** (`B10` first) |
| `w-mutcensus-c` | `L4`…`CA23` | forward |
| `w-mutcensus-e` | `L4`…`CA23` | **reverse** (`CA23` first) |

**Reverse order rather than a fresh split, for two reasons.** It needed no
restart — `b` and `c` were already 25 minutes into their graded baselines, and
their lists are fixed in a running process's argv. And where the two ends of a
list meet, the same mutant gets measured **twice, in two independently
provisioned worktrees** — a free reproducibility check on the colour itself, at
the cost of only the duplicated ids at the meeting point.
`work/w-mutcensus/collect.sh` keeps both logs, reports **AGREE / DISAGREE** per
duplicated pair, and prints a disagreement loudly rather than letting one run
overwrite the other. A disagreement would mean a colour is not a property of the
site, which outranks the census.

**An unplanned benefit of the reverse order, worth stating because it changes
what a partial campaign is worth.** Under a budget that does not reach all 56,
forward-only runners would have finished the *front* of both lists and left whole
families untouched. With each list worked from both ends, a partial campaign
covers the **front and back of both** — so the `CS` block and the `B`/`D`/`BU`
block both get rows, and the `L` block and the `CA` block both get rows, instead
of one list being fully measured and the other not at all. Per-family shape (§4
of the rung) is what this census was commissioned for, and family coverage is
worth more to it than depth in any one family.

**Diagnosis that was ruled out, so nobody retries it.** The obvious suspect for
the slowdown was the **shared capture cache**: `main_repo_root()` resolves to the
*true* main repo from inside a worktree, so all four sidecars and both peer gates
share `work/capture-cache`, which had **165–172 live lockfiles** during the run.
Checked and rejected: the two bottleneck tests spawn `c2rs perf` and
`c2rs selftest`, and **neither touches `CaptureCache`** (no reference in
`cli/perf.rs`, and selftest has no cache path) — the lockfiles are the peers' gap
scans. Giving each sidecar its own `C2RS_GAP_CACHE` root would have bought
nothing. The contention is raw CPU and `wibo` process throughput.

**Checked before trusting the contended runs:** no integration test in the
workspace asserts on wall-clock time (`grep` for `elapsed()` in `crates/*/tests`
returns nothing), so contention cannot manufacture a false RED via a timeout.
Every RED's failing-test names are reviewed for plausibility against the mutated
site regardless.

## D6 — CAUGHT MID-CAMPAIGN: the registered baseline 1,648 / 0 / 42 is IDENTICAL with and without a toolchain, so it cannot detect a differential that grades NOTHING

**This is the campaign's own instrument failure, found on the third mutant row,
before any affected colour was published. It is the most useful thing this lane
measured about its own probe, and it would have inflated the headline X.**

The two sidecars of D1 were created by a plain `git worktree add`, so neither had
`compilers/` — that directory is gitignored (MS binaries are never committed) and
is provided per-worktree by `scripts/configure_existing_worktree.sh`. By design
(CLAUDE.md: *"Integration tests + the `c2rs` CLI must degrade cleanly when the
toolchain is absent"*), every toolchain-driven test then prints
`SKIP: toolchain absent` **and passes**.

**A skipped test is a passed test, so the suite totals are unchanged.** Measured,
not reasoned:

| run | worktree | toolchain | passed / failed / targets | `census_gate` target |
|---|---|---|---|---|
| baseline, session 1 | `w-mutcensus` | present | **1,648 / 0 / 42** | **84.17s** |
| `N0wtB` | `w-mutcensus-b` | **absent** | **1,648 / 0 / 42** | **0.00s** |
| `N0wtC` | `w-mutcensus-c` | **absent** | **1,648 / 0 / 42** | **0.00s** |

The prereg's §2 probe definition — *"42 targets, baseline 1,648 / 0"* — is
therefore **blind to this fault**, and so is prereg §4.5's `targets -ne 42` rule:
all 42 targets ran, and 42 of 42 reported `ok`. A worktree that grades nothing
passes every hygiene invariant the prereg wrote down.

**Why that is a false-GREEN generator, in one line:** GREEN means *no test in the
suite can fail on this site*. In an unprovisioned worktree the tests that drive
real `c2.dll` cannot fail on **anything**, so every site guarded *only* by the
differential reads GREEN — and X, the published headline, is a count of GREENs.
The error is one-directional: it can only inflate X.

**How it surfaced** — not by inspection, by a contradiction between two runs of
the same mutant. `L4`'s interrupted session-1 log (worktree with toolchain)
showed the failing test
`census_gate::the_census_and_the_port_agree_about_what_is_in_class` after 171.58s;
`L4`'s sidecar run showed that same test **`ok` in 0.00s** and a different single
failure. Same mutation, same commit, two different failing sets — which is only
possible if the two runs were not running the same suite.

**The fix, structural, three layers:**

1. Both sidecars were provisioned with `scripts/configure_existing_worktree.sh`,
   whose own hard gate is the fixture census verdict; both now report
   `fixtures/cpp/w5_chain.cpp -> 4/4 functions in class`.
2. `run_mutants.sh` gained a **pre-flight** on the clean tree — the same census
   probe — and aborts the whole list rather than emitting a colour if the
   differential does not grade.
3. `run_mutants.sh` and `rederive.sh` now record the **`census_gate` target
   duration per run** as a TSV column and make any run under 1s **INVALID**. This
   is a per-run check, so it also catches a toolchain that disappears mid-list,
   and because `rederive.sh` derives the table from the logs it applies
   **retroactively to every log on disk**.

**What was discarded, and what survived.** Two colours were read in the
unprovisioned sidecars and are **discarded**, not used: `CS2` (read GREEN) and
`L4` (read RED). Both were re-run from scratch after provisioning. The faulted
logs are kept as evidence under names that cannot be mistaken for colours —
`CS2.notoolchain.DISCARDED.log`, `L4.notoolchain.DISCARDED.log`,
`N0wtB.notoolchain.log`, `N0wtC.notoolchain.log` — and the new rule classifies
all four **INVALID** at 42/42 targets, which is the check working.

**All 8 session-1 colours SURVIVE the new rule**, re-derived from their logs:
`C1` 94.81s · `C2` 70.15s · `C3` 80.09s · `C4` 74.79s · `C5` 91.17s · `L1`
76.21s · `L2` 76.63s · `L3` 87.53s. Every one graded 70–95s against real `c2`,
so both published GREENs (`L2`, `L3`) were measured with a live differential and
the controls' reproduction of `w-guards`' counts stands.

**Generalization worth more than this lane's X:** the repo already knew this
trap — `scripts/configure_existing_worktree.sh`'s own header says *"`cargo test`
is green, `c2rs diff` says SKIP, and a change that mis-emits looks exactly like a
change that is byte-exact"* — and this lane walked into it anyway, because the
prereg specified its probe as a **pair of totals** and totals are exactly the
thing the fault preserves. **A probe defined by a count cannot detect a
population that silently left the count.** That is STATUS trap 5
(absence-read-as-success) one level up: not a missing target, a present target
that measured nothing.

## D2 — C3's recipe was refitted once (E0277), carried from the first session

`C3`'s first spelling was `.contains(&name) | true`, which does not compile:
`|` binds looser than the method call, so `true.then_some(name)` parsed first
(E0277). That run was recorded **INVALID**, not a colour, and the recipe was
refitted to `w-guards`' M3 form in effect — the gate's answer becomes an
unconditional `Some(name)`. Prereg §2 registered P(≥1 INVALID needing a recipe
fix) = 0.5; this is that event, and it is a HIT of that registration.

## D3 — The runner's INVALID rule had matched cargo's own `error: test failed`

Carried from the first session. Cargo prints `error: test failed, to rerun ...`
for **every** target with a failing test. The runner's first INVALID predicate
matched bare `error:` and therefore mislabelled `C1` — a genuine RED — as
INVALID. The predicate is now `^error\[E[0-9]+\]|could not compile`, i.e. build
failures only, and `results/summary.tsv` is **derived** from the logs by
`rederive.sh` so the fix is retroactive over every log rather than applying only
to later runs. Both the mislabel and the fix are on the page because a colour
rule that silently reclassifies REDs as INVALID would have deflated X by
counting guarded sites as unrunnable.

## D4 — Two runs from the first session were aborted mid-suite and are NOT colours

`CS2` and `L4` were mid-suite when the first session was stopped. Their partial
logs are kept as `results/CS2.aborted.log` and `results/L4.aborted.log` and are
**never** read as colours: prereg §4.5 makes a run with fewer than 42
`test result:` lines INVALID, because an absent target is an absence and not a
pass (STATUS trap 5). `L4`'s partial log showed 1 failure at 14 targets, which
*looked* consistent with its registered RED — and that is exactly the inference
this rule forbids, so `L4` was re-run from scratch in this session and its
colour comes only from the complete run. Both ids were re-run whole; the
`.aborted.log` files are retained as evidence of the interruption, and
`publish.py` excludes any id ending in `.aborted`.

## D5 — The enumeration is frozen at `3835469c` and a live peer already invalidates it

`w-fence163` (peer, live during this campaign) landed
`c2-il: admit narrow string literals behind an EH-state inline fence` at
`d28326b4`, which adds a **20th fence-key constant**
`DATA_SYM_STRLIT_FENCED = "data-sym-strlit-fenced"` (`body/mod.rs`), **5** lines
that mention it, and new deciding gates in `bind.rs`, `bundle.rs::functions()`,
`census.rs` and `gl.rs` (+240 / −13 across five `c2-il` files).

**This lane did NOT re-enumerate to absorb it, and must not:** the enumeration
rule and all 64 registered colours were frozen at `3835469c` before the first
mutant ran, and widening the frame after the fact would unfreeze the prereg.
The site is therefore recorded as one this census **necessarily misses**, and
the more useful thing it establishes is the **instrument's shelf life**: one
peer lane, landing one fence, adds at least one raise family the census has no
row for. A mutation census over `c2-il`'s fences is not a fact about the
repository; it is a fact about a commit. Re-running `enumerate.sh` is a
precondition of quoting X/N against any later head.
