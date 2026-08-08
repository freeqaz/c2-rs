# w-hatch — PREREGISTRATION

Frozen at master `2b1c89da`, branch `wt-w-hatch`, **before the first
counterfactual scan of item 2 and before the first `crates/` byte of any kind**.
Item 1's repair is confined to `work/w-front3/hatch.py` and touches no `crates/`
file, so it does not need a prereg; it needs a red test, and that is `§1.3`.

---

## §1 Item 1 — board #1380, `hatch.py revert`

### §1.1 What I expect to find

| # | expectation | direction I expect to be wrong in |
|---|---|---|
| P1 | `revert` on master is a bare `git checkout -- <FILES>` with no guard | — (read, not predicted) |
| P2 | the implementation is recoverable from dropped commit `09622f6f` as the brief and board **#1380** both state | **that it is NOT there.** `09622f6f`'s stat lists `hatch.py`, but a stat is not a hunk. If the commit's `revert()` is byte-identical to master's, the board row's provenance cell is wrong and that is a finding, not an obstacle |
| P3 | `hatch.py apply` succeeds on this master | **that it REFUSES.** #1355's own fourth control cell records `assign-store-type` drifting when `dc844f64` landed (`eat_int_like` → `store_type_gate`). If it still refuses, the hatch instrument is DOWN on master and item 1 cannot demonstrate its own control without re-deriving that needle |
| P4 | the hatch-only control does not fire | — this is the control and it must stay green |

### §1.2 The stopping rule for item 1

Ship the guard **only if** all of these hold, and decline otherwise:

1. every refusal fires on purpose, in a named arm, with its verbatim text
   captured to a file;
2. the **hatch-only** tree and the **clean** tree are both GREEN — no refusal
   word appears at all;
3. after every red arm, the tree is **byte-unchanged** — checked as a `git diff`
   count, not as an exit status;
4. `--force` reverts, prints what it destroys, and leaves `crates/` clean.

### §1.3 The two mutation traps, and how each arm forecloses it

**Trap A — an early guard makes a later assertion unreachable.** Mutating in a
way that also moves an earlier quantity means the later gate never runs and its
arm passes on the earlier gate's refusal.

> Foreclosed by: the `DIRTY` arms are produced by *adding a genuine foreign
> edit*, never by breaking the un-apply machinery; and the two arms whose guard
> is structurally unreachable on a well-formed tree (`HATCH-RESIDUE`,
> `HATCH-CHECKOUT-FAILED`) are fired by **declared injection through one seam**
> (`_checkout`) and are labelled INJECTED in the output rather than passed off
> as natural.

**Trap B — a shared message prefix lets a later gate's refusal satisfy an
earlier case's expectation.** `w-throughput` had **two of six mutations pass
silently** this way.

> Foreclosed by: every refusal leads with **its own word**; the harness asserts
> on the **leading whitespace-token of a line**, never on a substring anywhere in
> the output; and each red arm additionally asserts that **every other arm's
> expected word is ABSENT** unless the co-occurrence is declared in the arm
> table. That last check is the one that would have caught `w-throughput`.
>
> `apply`'s two failure kinds shared `HATCH FAILED:` on master. They are split
> here (`HATCH-DRIFT` / `HATCH-PAID-MISSING`) for exactly this reason.

---

## §2 Item 2 — board #1369, `w-cmp`'s `expr-brfalse` credit of 5 TUs

### §2.0 THE ROW HAS ALREADY MEASURED ZERO — recorded BEFORE the counterfactual

The brief instructs: *"Check whether the row has already measured zero."* It has.

* **Board #440**, lane `w-brfalse`, **2026-08-05**, at base `cf86b09`:
  `C2RS_SINK_BRANCH` — a purpose-built, poisoned, OFF-by-default sink at four
  nested levels — run as **five 878-TU scans of one binary, environment
  variables apart**. Closing the whole intra-body control-flow skeleton
  (`brfalse` 5,484 · `brtrue` 659 · `jump` 303 · `label` 48 = **6,494** blocked
  emitted functions) converted **0 functions and 0 TUs at every level**, against
  a ladder credit of 5 (OFF→REL) / 6 (B1) / 7 (B2, B4).
* **Board #422** — the very w-cmp row carrying the 5 — is **already marked
  REFUTED on the board**, *by #440*, three days before #1369 was filed.
* **Board #441** generalises it: *the ladder's head is a FIXED POINT of the sink
  operation, not a name for work.*
* **The sink is still live**: `crates/c2-il/src/func/body/expr.rs`, `BranchSink`.
  It was **not** reverted, which is the difference between this row and #407.
* **The reproduction script is committed**: `work/w-brfalse/arms.sh`, five arms,
  one binary.

So **#1369's premise — *"has never been measured the way those 2 just were"* —
is FALSE as written**, and this is the fifth time on this project a row has
re-entered a ranking on size alone after already being measured. `w-667`'s own
§5.1 cites #440 by number in the same paragraph in which it says the number does
not exist; what it meant, and what is true, is that *it* did not run the arm.

### §2.1 What I am running anyway, and why that is not redundant

I am **not** funding the rung. I am re-taking the number, because board **#1337**
is explicit that on a fall-through row *"re-measuring is cheap and transcribing
is wrong — #400's 6 was three days old and 25 % low"*, and #440 is **three days
old at a base where the population has moved**: TU match 8 → **11**, emitted
census 38,458 → **39,185**, FRONTIER 19 → **16**, and `w-lineage`/`w-mrslot`/
`w-667` have all widened readers underneath it. A fall-through row's worth is a
function of how wide the readers were **on the day it was measured**.

Cost: `work/w-brfalse/arms.sh` unmodified, from this worktree, with `C2RS_DC3`
set. No `crates/` edit. Five arms.

### §2.2 THE STOPPING RULE, frozen before the first arm

I will run **exactly the five arms of `arms.sh`** — `off`, `rel`, `b1`, `b2`,
`b4` — and then **STOP**. Specifically:

* **I will not add a sixth arm**, however the five read. A sink that was built,
  levelled and merged is the instrument; adding a level to it because the answer
  displeases is the shape of fitting.
* **I will not build a new sink.** The brief's *"if a counterfactual sink for
  this row does not exist, ship one"* is **discharged by finding that it does**
  — `C2RS_SINK_BRANCH`, committed and live. Shipping a second one would be #407's
  build-and-revert loop in reverse.
* **I will not edit `crates/` for item 2 at all.** If the arms disagree with
  #440 I report the disagreement and file it; I do not chase it in this lane.
* **The verdict is written from the `off` → `b2` pair.** `b1` and `b4` are the
  nesting controls.

### §2.3 What I expect, and the direction I expect to be wrong in

Board **#770** records **ten consecutive misses in the OPTIMISTIC direction**, so
this is registered pessimistically on purpose and the miss direction is named.

| # | prediction | confidence |
|---|---|---|
| Q1 | **TU match is 11 in every one of the five arms** — recovered **0** | high |
| Q2 | blocked-emitted TOTAL is **identical in all five arms** — every closed key is *renamed*, not recovered. This is w-667's *"a harder zero than a small positive"* | high |
| Q3 | the `gap-metric` factorization block (`factor-c`, `b-and-c`, `a-and-b-and-c`, `frontier`, …) is **byte-identical** between `off` and every sunk arm | high |
| Q4 | `expr-brfalse` on the emitted axis reads **≈ 3,105** at `off` (`work/gap-merged-lineage.txt` has 3,105 at the merge) and **0** at `b1` | medium — the count, not the direction |
| Q5 | the successor keys the closed mass lands in number **between 20 and 60** | low — this is the number I have least basis for |
| Q6 | the five TUs w-cmp credited (`IPP_basicmath_xbox`, `osfinfo`, `undname`, `mmio`, `jsonwriter`) still resolve to **call-in-expression**, not control flow, as #441 found | medium |

**The direction I most expect to be wrong in:** that the *closed mass* has grown
since #440 (5,484 → more) because three lanes widened readers underneath it, and
that I will therefore be tempted to read a bigger absorbed number as a bigger
result. **It is not one.** The only quantity that scores this row is
`recovered`, and a larger renamed population with the same zero recovery is the
*same* result at higher confidence, not a better one.

**What would make me report a POSITIVE:** any arm in which TU match exceeds the
`off` arm's, or in which the blocked-emitted total falls. Either would be a
disagreement with #440 at a newer base and would be reported as such, loudly,
whatever it did to this lane's tidiness.

### §2.4 The provenance discipline

* **One binary, environment variables apart.** `md5sum target/release/c2rs`
  taken **before the first arm and after the last**, both printed.
* **`binary_sha` read out of every arm's own JSONL provenance record** and
  asserted equal across the five — the checksum proves the file did not change,
  the provenance record proves *that file* is what ran.
* **Board #1367's hazard is live for this exact script**: a peer lane's probe run
  from a worktree resolved `ROOT` three directories up and graded the **main
  repo's** binary, with the change under test silently absent. `arms.sh` resolves
  `ROOT` from its own path, so from this worktree it takes the worktree's binary
  — **checked by comparing the `md5sum` it prints against this worktree's**, not
  assumed.
* `capture-fail 7` is the positive discriminator that the run reached the real
  workload: a bad `--cwd` gives `capture-fail 878 / match 0` and otherwise looks
  entirely ordinary.

---

## §3 Peer-collision check

Two other lanes are running. `work/w-splice/peerkeys.py` is run at **both ends**
and any key family that moved is reported, because file ownership is necessary
and not sufficient — lanes here have collided through shared semantics with no
git conflict three times in one wave.

**This lane's `crates/` diff is expected to be EMPTY at both ends.** Item 1 is
`work/` only; item 2 is measurement only. If `git diff master -- crates/` is
non-empty at the tip, this lane did something it did not preregister.
