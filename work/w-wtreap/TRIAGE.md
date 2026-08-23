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
  capability: ~~*"+90 [TU reach] the moment `.rdata$r` lands."*~~ The keygen lane
  landed without it — either declined for a reason the rung will state, or
  parked. If `.rdata$r` placement is ever funded, this branch holds a started
  implementation (7 crates files + 4 fixtures).

  > **CORRECTED 2026-08-23 — the two sections are DIFFERENT and this line
  > conflated them, inflating the branch's price.** `_2026-08-05-w-reach.md:312`
  > prices **`.rdata$r`**, the *COMDAT RTTI* section (step one of `BOARD.md:567`'s
  > ladder `.rdata$r` 590 → `.text$yd` 804 → `.xdata$x` 871). `w-keygen`'s open
  > refusal 15 (`2026-08-13-w-keygen.md:123`) is **non-COMDAT `.rdata` with
  > content — 384 bytes**, a TU-level section, and the branch's own decode bound
  > explicitly *refuses* COMDAT `.rdata`. **Landing `w-keygen` does not buy +90
  > TU reach.** Price it on refusal 15 (20 → 19) plus the live wrong emit it
  > closes plus its 12 GRID R cells. The verdict is unchanged and the branch is
  > still the highest-value item in this set — only the number was wrong.
- **`w-reltgt`** — ~~"require relocation-target agreement before crediting
  fnbyte-exact." No rung, no board row, no `relocation-target agreement`
  match in `gap/factors.rs`. The idea may have landed under other names in
  the reloc campaign, or may be a genuinely open credit-tightening.~~

  > **CORRECTED 2026-08-23 — SUPERSEDED, and the "no trace" above was a FALSE
  > NEGATIVE of my own making.** The grep ran against the *branch's* key names
  > and against the wrong file. The idea landed as lane `w-relo`, merge
  > `2abca17df` (2026-08-07) — *"exact now means exact — FBM grades relocation
  > targets, and 861 bodies it credited branch to the wrong function"* — with a
  > richer reader (`RelocDiffers(RelocKind)` at `gap/fnbytes.rs:140`), a
  > known-answer control and an independent second-reader replication. It landed
  > **four days before this branch's own salvage commit**. The hedge in the
  > struck text ("may have landed under other names") was the correct reading;
  > the flat "no trace on master" was not. **Verdict: ABANDON-SUPERSEDED.**
- **`w-scfl`** — ~~loop-production prereg with discriminators and falsifiers;
  no scfl rung or test on master.~~ **CORRECTED 2026-08-23: SUPERSEDED.** Master's
  lane `w-seed` (merge `ed511348b`) measured the branch's own predictions 27 h
  after its prereg — 223 of 228 converted, 0 regressed — and *partly refuted the
  central one* (H1 predicts a top-tested loop; the measured grammar is
  bottom-tested). Tip commit self-labels its only code as scratch to be deleted
  before landing. **Verdict: ABANDON-SUPERSEDED.**

  > **Net on this section: of the three branches flagged "worth a look", exactly
  > ONE survives — `w-keygen`.** Two were refuted by a deeper read. The section's
  > method (grep master for the capability's identifiers) is what produced both
  > false negatives: a grep keyed on the *branch's* vocabulary cannot see the
  > same idea landed under a different lane's names. Establish supersession by
  > reading the campaign, not by grepping the branch's own words.
