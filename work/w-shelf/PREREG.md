# PREREG — lane `w-shelf` (decision 18, board #3674–#3679)

**Written and committed BEFORE the first removal, the first scrub, and before
any of the three predicted quantities was measured on this tree.** Never
edited after; graded in the rung, and a miss is said in the word **MISS**.

- base: `41ca1ee9a` (master tip at dispatch), worktree `.claude/worktrees/w-shelf`, branch `wt-w-shelf`
- date: 2026-08-27
- charter: decision 18 § "The lane this decision dispatches" —
  enforce the two carve-outs on the `work/` evidence shelf (**no binaries or
  build artifacts**, **no absolute machine paths**), repair the broken NUL
  guard wherever it lives, widen `scripts/tracked_artifact_audit.sh` to cover
  `work/`.

## What I already know, and therefore am NOT predicting

These are read from the record, not from this tree, and they are the inputs
the predictions below are formed against — quoting them is not a measurement:

- The coordinator's audit at `0dcfca959` reported **one** tracked ELF
  (`work/w-biquad/c2rs.base`, 3,835,864 B) and **488** tracked files under
  `work/` carrying `/home/free` (decision 18 § 2 table).
- `work/w-biquad/REMOVED_ARTIFACTS.md` documents a 21-file removal on
  2026-08-25 and does **not** list `c2rs.base`.
- Board `#1236`/`#3544`/`#3513` record `grep -P '\x00'` and `grep -c $'\0'`
  as non-firing NUL detectors, known since 2026-08-08.

## Predictions

### P1 — surviving tracked binaries, re-derived by a NUL test that works

**Point prediction: 1.** Bias direction: **biased LOW — I expect the true
number to be ≥ my point estimate, not ≤ it.**

The reasoning for the bias, stated so the grading can check it rather than my
number: the population that produced "1" was produced by a scan whose NUL test
did not fire, and the one hit that *is* known was found by a different method
after the fact. A detection method known to be incomplete cannot bound its own
population from above. My sweep is broader (every tracked file in the repo,
not only `work/`, and a byte-count NUL test rather than a `grep` predicate),
so it can only find the same set or a superset.

**Interval: 1–6.** Above 6 I would treat the removal of 2026-08-25 as having
missed a whole class rather than a file, and say so.

### P2 — tracked files under `work/` carrying an absolute `/home/<user>` path

**Point prediction: 488.** Bias direction: **biased LOW.** Three peer lanes
(`w-mopfold`, `w-secported`, `w-provaudit`) are live and committing evidence
to their own worktrees, and my base is two commits past the tree the 488 was
read on. Lane evidence gains absolute paths by default (a transcript quoting
the directory it ran in), so drift is one-directional.

**Interval: 488–520**, and *of those, the count I may actually scrub is
smaller*: `work/w-mopfold/**`, `work/w-secported/**`, `work/w-provaudit/**`
and `work/coordinator/**` are outside my fence and stay untouched however
many of them offend. **I predict 0–40 files fall in that excluded set**, and
I will report the excluded count separately rather than folding it into a
"scrubbed" number.

### P3 — broken NUL tests still in the repo

**Point prediction: 3** committed sites spelling a NUL test in a form that
does not fire (`grep -P '\x00'`, `grep -c $'\0'`, `grep -q $'\0'`, or a
`grep` NUL predicate by any other spelling), across `scripts/`, `work/`, and
any other tracked helper. Bias direction: **biased LOW** — the form is a
reflex, it was reached for again on 2026-08-26 eighteen days after being
filed, and no gate has ever been able to see it.

**Interval: 1–12.** A prose *mention* of the broken form (this file, the
board rows, `tracked_artifact_audit.sh`'s comment block) is **not** a site
and is not counted or repaired; only a live test whose verdict something
depends on counts.

### P4 — the audit's scope change

**Prediction: widening class 2 to `work/` turns the audit RED at my base and
GREEN only after the scrub**, and the count it goes red by equals P2 minus
the peer-owned exclusions. If the widened audit is green at my base, the
scope change did not take and I say so rather than shipping it.

## Decline floor — what makes me stop and report instead of proceeding

Any one of these and I decline the affected deliverable in the rung, in the
word `declined`, rather than forcing it:

1. **A file in scrub scope is held open by a live process.** Board `#1135`:
   a scrub punched a 122-byte NUL hole into a *passing* gate's transcript
   because a backgrounded `gate.sh` still held its `>` fd. I check
   `/proc/*/fd` per file (`#1236`'s repair, a fact about writers, not about
   timing) and skip — never rewrite-and-hope. A skipped file is reported by
   name.
2. **A scrub is not byte-identical outside the prefix.** If the sha256 of the
   measurement content of a scrubbed file differs from its pre-scrub value by
   anything other than the substituted prefixes, the whole scrub is reverted,
   not patched.
3. **More than one absolute root, and they are not separable.** If the
   transcripts reference roots whose distinction a substitution would flatten
   and no longer-prefix-first ordering preserves it, I stop and report the
   roots rather than choose for the record.
4. **The work implies a `crates/` byte.** Hard fence: I write no `crates/`
   bytes at all. If enforcement requires one, I STOP and report.
5. **`scripts/gate_identity_diff.sh` base-to-tip is not 0 lines over 21 rows.**
   Nothing in this lane may change what the port emits; a nonzero identity
   diff means it did, and the lane is `FAILED`, not "explained".

## What this lane does NOT set out to do

- **No history rewrite.** Decision 12's precedent and the owner's standing
  choice: the blob stays reachable, nothing is destroyed. No `filter-branch`,
  no `filter-repo`, no force-push, no rebase.
- **No scrub of `e:/lazer_build_gmc1` or `dc3-decomp`.** 366 files name these
  and `CLAUDE.md` says they are INTENTIONAL. Only `/home/<user>`-style
  *machine* paths are in scope. I expect to touch **0** of the 366 for that
  reason (some may be scrubbed incidentally if they *also* carry a
  `/home/<user>` path — that is a different token in the same file, and the
  `e:\` string stays byte-identical).
- **No `.gitignore` `/work` removal.** The line governs the default for
  untracked scratch and stays. If I touch `.gitignore` at all I say exactly
  why.
- **No touching of `work/w-mopfold/**`, `work/w-secported/**`,
  `work/w-provaudit/**`, `work/coordinator/**`** — live peers and the
  coordinator's gate-base shelf.
