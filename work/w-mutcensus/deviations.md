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
