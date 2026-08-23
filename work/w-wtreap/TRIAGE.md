# Unlanded-branch triage — 2026-08-23, lane w-wtreap

The worktree estate held 390 trees; 363 were mechanically reapable (branch or
detached HEAD an ancestor of master, tree clean — see `scripts/wt_reap.py` and
the board row). This file is the other half: the 17 branches whose commits are
NOT on master, adjudicated by evidence rather than removed blind.

**Method.** Three mechanical probes per branch, no content judgment required
for most verdicts:

1. *Supersession score*: for every file the branch changed, is the branch's
   blob byte-identical to master's? (identical = that change landed by
   another path).
2. *Rung overlap*: does master already carry the rung file the branch
   adds/edits, and is master's version the longer/later one?
3. *Semantic probe*: does master contain the capability's identifiers
   (`rdata_shell`, `memchr`, separator walk, reach sets, ptr-leaf fixtures)?

**Standing outcome: every clean worktree is removed; NO branch is deleted.**
An unmerged branch costs nothing as a ref and `branch -D` is the only way to
lose one — the verdicts below say which branches deserve a landing effort,
not which to delete.

## Superseded — the lane continued elsewhere and landed (7)

| branch | evidence |
|---|---|
| `worktree-agent-a90821e906953b0fd` (w-dclass/B) | **58/58 files byte-identical on master** — pure supersession |
| `wt-w-dclass-c-storetype` | rung byte-identical on master; branch `assign.rs` is 200 lines *behind* master's |
| `wt-w-biquad` | branch rung says DECLINED at 12; master's same-path rung says the TU **CONVERTS** — master is strictly later knowledge |
| `wt-w-nc` | master rung is the 555-line final; branch holds the 107-line early draft |
| `wt-w-gate3048-final` | the `gate.sh` fix is on master (byproduct-grading markers present); residual diff is drafts + a repro log |
| `w-keygen` | master `2026-08-13-w-keygen.md` is the final, three days after the branch's last commit — **but see "worth a look" below** |
| `worktree-agent-afeb2726cbf32392d` (w-reach) | `_2026-08-05-w-reach.md` on master with measured verdicts (+0 today / +90 on `.rdata$r`) |

## Likely superseded — capability present on master by another path (3)

| branch | evidence |
|---|---|
| `worktree-agent-a9b46436bd732cef7` | memchr-driven scans present in master `gl.rs`, `readers.rs`, `ex.rs` |
| `worktree-agent-ac6428ec202a75930` (W-ADOPT) | `2026-08-02-w-adopt.md` on master; `gl.rs` has 45 separator mentions |
| `worktree-wf_67a45230-6e9-2` | `w12_ptr_leaf*.cpp` fixtures on master; `c2-il` shapes long evolved past the branch (2026-07-30, oldest branch) |

## Scratch / evidence-only, by their own subjects (5)

| branch | what it holds |
|---|---|
| `wt-w-cflow` | "scratch census cross-tabs" |
| `wt-w-rank` | "scratch attribution instrumentation" |
| `worktree-agent-ad62a3e829ebdd26c` | 27 `wtrace_scratch/` probe files |
| `w-biquad` (agent tree) | base/tip ladder scans banked "behind the DECLINE" — the decline was later overturned (master: CONVERTS), so this is historical evidence only |
| `worktree-agent-a6419a796edcaffd3` | two wordwrap2 ground-truth `.obj`s from GRID B; the wordwrap2 rung landed 2026-08-10 |

## Worth a look before writing off (3)

- **`w-keygen`'s `rdata_shell` code is NOT on master** (`rdata_shell` /
  `rdata_two` grep empty) while master's W-REACH rung prices exactly that
  capability: *"+90 [TU reach] the moment `.rdata$r` lands."* The keygen lane
  landed without it — either declined for a reason the rung will state, or
  parked. If `.rdata$r` placement is ever funded, this branch holds a started
  implementation (7 crates files + 4 fixtures).
- **`w-reltgt`** — "require relocation-target agreement before crediting
  fnbyte-exact." No rung, no board row, no `relocation-target agreement`
  match in `gap/factors.rs`. The idea may have landed under other names in
  the reloc campaign, or may be a genuinely open credit-tightening. 2 commits,
  prereg included.
- **`w-scfl`** — loop-production prereg with discriminators and falsifiers;
  no scfl rung or test on master. Tip commit self-labels its test scratch
  ("deleted before the lane lands"), so the branch was mid-flight when
  abandoned.
