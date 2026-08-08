# w-hatchroot — the hatch edited the wrong repository and said nothing, and the second frontier instrument was still run by hand

    Tag:       w-hatchroot
    Slug:      w-hatchroot
    Date:      2026-08-08
    Fixtures:  none — both items are verification apparatus; this rung admits no class
    Census:    711,486 / 2,463,443 unchanged (28.88 %), **+0**. TU match
               **11 → 11**, mismatch **0 → 0**, codegen-gap **0 → 0**,
               vocab-gap **860 → 860**, capture-fail **7 → 7**. All 187
               `gap-metric` lines byte-identical; `peerkeys.py` reports **0
               families vanished** and **0 moved**. FRONTIER **16 → 16**.
    Record:    this file; prereg `work/w-hatchroot/PREREG.md`, committed at
               `7254d1e8` **before a line of either item was written**.
    Lane:      w-hatchroot, worktree branch `wt-w-hatchroot` off master
               **`85e180d4`**.
    Ships:     board #1460 closed — `work/w-front3/hatch.py` resolves its target
               repository from the **invoking cwd** and refuses a cross-tree run,
               and `work/w-hatch/hatch_red.py` (which had the same defect and a
               worse blast radius) does the same; board #1406's **second half**
               closed — `work/w-ladders/ladder_red.py` is a `scripts/gate.sh` row
               with its own classifier, its own eleven words and 18 new selftest
               cases. New instruments `work/w-hatchroot/{row_red.sh,
               mutate_gate.sh}`. **No `crates/` change at all.** Board rows
               **#1493**–**#1499**.

---

## 1. The result table

| | base `85e180d4` | tip `wt-w-hatchroot` |
|---|---|---|
| TU match / mismatch / codegen-gap / vocab-gap / capture-fail | 11 / 0 / 0 / 860 / 7 | **11 / 0 / 0 / 860 / 7** |
| `gap-metric` lines differing | — | **0 of 187** |
| `peerkeys.py` families vanished / moved | — | **0 / 0** |
| FRONTIER | 16 | **16** |
| census | 711,486 / 2,463,443 (28.88 %) | **unchanged, +0** |
| `git grep -c '#[test]'` under `crates/` | 1,207 across 107 files | **1,207 across 107 files, +0** |
| `cargo test --workspace --release` | 1,202 passed / 0 failed / **36 targets** | **1,202 passed / 0 failed / 36 targets, +0** |
| `gate.sh --selftest` | 120 cases, floor 120 | **138 cases, floor 138** |
| `hatch_red.py` | 11 arms, 9 red, 2 green, 8 words | **14 arms, 11 red, 3 green, 10 words** |
| `ladder_red.py` | 5 arms, run by hand | **5 arms, a `gate.sh` row** |

**`git diff 85e180d4..HEAD -- crates/ Cargo.toml Cargo.lock fixtures/` is EMPTY**
— zero lines. The base and tip binaries are therefore not two builds but **one**:
`sha aef73ac63309`, the same sha the gate pinned. So **the two scan columns above
are the same measurement quoted twice**, and that is stated rather than presented
as two independent readings. What it does establish is that neither instrument
change perturbed the scan, which is the claim this lane needs and the only one it
is entitled to.

`diff work/w-hatchroot/metrics_{base,tip}.txt` is **two lines, and they are the
same line**:

```
1c1
< GAP REPORT (878 TUs in 2.6s)
---
> GAP REPORT (878 TUs in 2.7s)
```

A wall clock, in a header. Every count, every one of the **187 `gap-metric`
lines**, the FRONTIER membership and the census are byte-identical.

### 1.1 The gate

`scripts/gate.sh --require-graded`, in the foreground, own output file
(`work/w-hatchroot/gate_tip.txt`), concurrency defaults untouched
(`jobs=16`, `C2RS_JOBS=8`) — **`GATE: PASS`**:

| | |
|---|---|
| lanes | **18 in the registry — 18 PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT** |
| fixture verdicts | **5,202** across all lanes |
| generated sweep | **19,556 of 19,556 reached, 19,460 GRADED, 0 mismatch** (96 ungraded — the reference rejects the source) |
| mode cross | **90,812 of 90,812 selected, 90,424 GRADED, 0 mismatch** |
| **`hatch-red`** | **PASS 14/14 — 11 red, 3 green controls** |
| **`ladder-red`** | **PASS 5/5 — 3 red, 2 green controls** |
| **mismatch, anywhere** | **0** |

Both instrument rows verbatim off the table, which is the first gate on this
project to carry two of them:

```
hatch-red            PASS           14/14         11       n/a  arms (3 green controls)
ladder-red           PASS            5/5           3       n/a  arms (2 green controls)
```

`n/a` in the mismatch column and not `0`, on both: these rows grade instruments,
not objs, and a `0` there would read as a graded zero — the strongest claim on
that table and one neither row can make.

**No wall clock is quoted as a result.** Two other lanes were on the box; the
gate's own preflight names one of their run trees as `LIVE` in the log.

### 1.2 The master-gate diff — the 18 lanes are untouched by the new row

The test that matters for a lane that adds a gate row while two peers are running
is not "my gate passes". It is **"master's gate, run against my tree, prints the
same numbers"**. Master's `scripts/gate.sh` was checked out to
`scripts/.master-gate.sh` (in `scripts/`, so `dirname($0)/..` still resolves to
this worktree) and run against this tree with the same pinned binary:

```
$ git show 85e180d4:scripts/gate.sh > scripts/.master-gate.sh
$ scripts/.master-gate.sh --require-graded          # master's gate, THIS tree
$ diff <master verdict block> <tip verdict block>
23a24
> ladder-red           PASS            5/5           3       n/a  arms (2 green controls)
30c31
< logs:   /tmp/c2rs-gate-3634853/<lane>.log, …
---
> logs:   /tmp/c2rs-gate-2890289/<lane>.log, …
```

**One added line — the new row — and one changed line, which is the run
directory.** `/tmp/c2rs-gate-$$` differs between any two runs of the same gate,
so that hunk is a property of there being two runs and not of there being two
gates. `w-cache` reported exactly this shape and this matches it.

Both runs pinned the **same binary**, `sha aef73ac63309`, and every other line is
identical:

| | master's `gate.sh` | this lane's |
|---|---|---|
| headline | `GATE: PASS — 18/18 lanes ran …` | **identical** |
| lanes | `18 in the registry — 18 PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT` | **identical** |
| graded | `5202 fixture-verdicts across all lanes` | **identical** |
| sweep | `19556 of 19556 reached, 19460 GRADED, 0 mismatch` | **identical** |
| cross | `90424 of 90812 graded, 0 mismatch` | **identical** |
| the 18 per-lane rows | `289/289` each, match counts 146/138/136/142/18 | **identical, digit for digit** |
| `hatch-red` | `PASS 14/14 11 n/a arms (3 green controls)` | **identical** |
| `ladder-red` | *(absent — master has no such row)* | `PASS 5/5 3 n/a` |

**The `hatch-red` row reading `14/14` under MASTER's gate is prediction 2.5
landing.** Master's `hatch_red_run` reads the expected arm count from
`hatch_red.py --list`, so adding three arms moved that row's numbers in *both*
gates and the diff stayed at one line. Had the count been written into the gate,
this lane would have shown up as a two-line change in every peer's verdict block.

---

## 2. ITEM 1 — `hatch.py` edited the repository its own file lives in

### 2.1 What the defect was, and why it is the worst shape available

`ROOT = dirname³(os.path.abspath(__file__))` — the repository **this script's own
file** lives in. A worktree lane has no `work/w-front3/` of its own unless it
makes one, so the natural invocation from a worktree is

    python3 ../../../work/w-front3/hatch.py apply

and that applied **eight edits across five files in the MAIN repository** while
`git status --short` in the worktree read empty. It fired this session, in lane
`w-5c2`, which reverted immediately; the main tree was verified clean at
`851938df` and re-verified at the merge.

**The tell was `sha256sum`.** The "hatched" binary came out byte-identical to the
unhatched one, because the edits had gone somewhere else. Without that check a
climb would have published **the unhatched ladder table as "the hatch moved
nothing"** — a false zero, in the instrument that prices the entire frontier, on
the one axis (`+29` rungs, #1431) where the hatch is the whole difference.

That is the fourth distinct defect in this file in three days, and **every one of
them produced a plausible-looking wrong answer rather than an error**:

| board | defect | what it produced |
|---|---|---|
| #1322 | `apply` wrote as it walked and raised on the first bad needle | a tree hatched 7 of 8, silently |
| #1380 | `revert` was a bare `git checkout --` over six `crates/` files | a peer lane's unstaged fix, eaten |
| #1405 | the `assign-store-type` needle had drifted | `apply` refusing on master for a day, with nothing saying so |
| **#1460** | `ROOT` from `__file__` | **eight edits applied to the wrong repository** |

### 2.2 The fix, and the test I started with that does not work

`ROOT` now comes from the **invoking cwd**. The script's own location is used for
exactly one thing: deciding whether this script is *allowed* to edit that tree.
Two `git rev-parse --show-toplevel` calls — one from the cwd, one from the
script's directory — must return the same path.

**A lexical containment test does not work here, and this repository's own layout
is why.** My first instinct was `os.path.commonpath([root, script])`, and the
PREREG registered that as the direction I expected to be wrong in.
`scripts/setup_worktree.sh` puts worktrees at `<main>/.claude/worktrees/<name>`
— **inside the main repository's path** — so:

* worktree cwd, main's script: `commonpath` is `<main>` ≠ the worktree root, so
  a lexical test refuses. Correct, by luck.
* **main cwd, a worktree's script**: `commonpath` is `<main>` **==** the root, so
  a lexical test *accepts* — and the main tree gets edited with the worktree's
  `EDITS`. Wrong code, right-looking tree.

Two `git rev-parse`s have no such hole, because `rev-parse` from inside a linked
worktree returns the worktree. **Verified in both directions**, and the second
one is quoted verbatim in §2.3.

Two new refusal words, each leading its own line: `HATCH-FOREIGN-ROOT` (exit 6)
and `HATCH-NOREPO` (exit 7). **`NOREPO` is checked first**, which is trap A — so
`hatch_red.py`'s `A3` arm builds a **real second git repository** rather than a
bare temp directory, or its assertion would pass on the other guard's refusal.

And it now **says which repository it is about to touch, positively, before
touching it**:

    hatch: TARGET REPOSITORY  <abs>/wt-w-hatchroot   (resolved from the invoking cwd, NOT from this script's path — board #1460)
    hatch: THIS SCRIPT        <abs>/wt-w-hatchroot/work/w-front3/hatch.py
    hatch: COMMAND            check

#1460 was silent for the same reason #1322 was: **nothing ever stated the thing
that turned out to be wrong**, so there was nothing to disagree with.

### 2.3 The verbatim red, in the direction a lexical test would have missed

Main-repository cwd, the *worktree's* fixed `hatch.py`, read-only `check`:

```
HATCH-FOREIGN-ROOT — this script belongs to a DIFFERENT checkout from the one
you are standing in. NOTHING WAS WRITTEN.
  you are standing in   : <abs>/c2-rs
  this script belongs to: <abs>/c2-rs/.claude/worktrees/w-hatchroot
  this script           : <abs>/c2-rs/.claude/worktrees/w-hatchroot/work/w-front3/hatch.py

  Board #1460: `ROOT` used to come from this script's own path, so invoking the
  main repository's copy from a worktree (`python3 ../../../work/w-front3/hatch.py
  apply`) hatched the MAIN tree while `git status` in the worktree read empty. The
  binary you then built was byte-identical to the unhatched one and the ladder
  climbed on it would have been published as "the hatch moved nothing".

  Copy `work/w-front3/hatch.py` into the tree you mean to hatch and run that copy.
```

`rc 6`. **The unfixed file, invoked the same way, reports on the wrong tree and
exits 0** — that is `#1460` reproduced at base, and it is the reason this is a
fix and not a tidy-up:

```
$ cd .claude/worktrees/w-hatchroot && python3 ../../../work/w-front3/hatch.py check
hatch: 0 of 8 edit(s) present, 8 pending, 0 undecidable
crates/ diff: EMPTY
hatch: CLEAN
rc=0
```

Every line of that is about the MAIN repository. Nothing in it says so.

And `HATCH-NOREPO`, verbatim, from a cwd in no checkout at all:

```
HATCH-NOREPO — the invoking directory is not inside a git checkout.
NOTHING WAS WRITTEN.
  invoking cwd : <abs>/w-hatchroot-b55yye_8
  this script  : <abs>/wt-w-hatchroot/work/w-front3/hatch.py

  `hatch.py` edits crates/ in place and reverts through `git checkout --`,
  so a tree git does not know about is a tree it cannot put back.
```

`rc 7`.

### 2.4 `hatch_red.py` had the same defect, and a worse blast radius

`ROOT` in the red harness was `__file__`-derived too — **in the file written to
test `hatch.py`**. It is the more dangerous of the two, and the reason is
asymmetric: `hatch.py revert` **refuses** on a tree carrying anything that is not
the hatch (#1380's repair), and `hatch_red.py`'s `restore()` is an
**unconditional `git checkout -- crates/`** with no such guard. Run from a
worktree by relative path it would have discarded the **main** repository's
unstaged `crates/` work — #1380's incident, aimed by #1460's mechanism, from the
instrument built to prevent #1380.

Same resolution, with its **own** words (`HATCHRED-FOREIGN-ROOT`,
`HATCHRED-NOREPO`) deliberately kept **out of `ALL_WORDS`**: that refusal aborts
the whole run before any arm exists, and a word an arm could match is a word that
could satisfy an arm's expectation (trap B).

### 2.5 The arms — 11 → 14, 8 words → 10 of 10

`work/w-hatch/hatch_red.py`, standalone, verbatim in
`work/w-hatchroot/hatch_red_tip.txt`:

    ALL 14 ARMS PASS — 11 red, 3 green
    distinct leading words exercised: 10 of 10
    final crates/ diff: EMPTY

Three arms added, and **neither red one is INJECTED** — both hand
`resolve_root` the real argument pair the incident was produced by, which is why
`resolve_root` takes the cwd and the script path as *parameters* rather than
reading globals:

| arm | word | how it is built |
|---|---|---|
| `A3 FOREIGN-ROOT` | `HATCH-FOREIGN-ROOT` | cwd in a **real** second `git init` checkout, script in this one — #1460 exactly |
| `A4 NOREPO` | `HATCH-NOREPO` | cwd in no checkout at all, and it refuses to run if this box's `$TMPDIR` turns out to be inside one |
| `C3 ROOT-FROM-SUBDIR` | *(green)* | `cd crates && python3 ../work/w-front3/hatch.py` must still resolve to **this** checkout's root |

**`C3` is the arm I registered as most likely to fail, and it is the one that
matters.** A guard that refuses a cross-tree invocation *and* refuses the
ordinary subdirectory one is a guard the next lane deletes. Its postcondition is
positive and on the returned value, not on the absence of a refusal:

    [postcondition] resolve_root(cwd=<abs>/wt-w-hatchroot/crates)
    [postcondition]   -> <abs>/wt-w-hatchroot
    [postcondition] equals this checkout's root: YES

### 2.6 The counterfactual is sharper than the one it copies

`hatch_red.py --master 85e180d4` runs the identical 14 arms against the
`hatch.py` this lane started from (`work/w-hatchroot/hatch_red_counterfactual.txt`):

    FAILED: A3 FOREIGN-ROOT, A4 NOREPO, C3 ROOT-FROM-SUBDIR

**Exactly the three new arms fail and the eleven old ones pass.** That is a
stronger shape than `w-hatch`'s own counterfactual (`--master 2b1c89da`, 10 of 11
failing): it says the new arms test the new guard *specifically*, rather than
testing "this file is old". The eleven passing are `w-one`'s and `w-hatch`'s
repairs, still in master, still fired.

---

## 3. ITEM 2 — `ladder_red.py` is a gate row now

### 3.1 What was open

`work/w-ladders/ladder_red.py` — 5 arms, 3 red, 2 green, and **5 of 5 fail
against the pre-`w-ladders` `ladder.py`** (`--master 851938df`), which is what
makes it non-vacuous. It fires `LADDER-NOWIDTHTABLE` on a missing width-table
file, an empty table and a table truncated to two arms; the green controls check
the real table parses to 46 with `0xBD`/`0x4C`/`0x41` **in** and `0x00`/`0x1C`
**out**, and classify seven grants 3 RENAME / 4 rung.

It was run by hand and nothing in CI touched it. `w-ladders` declined to wire it
in mid-wave and said so; that was right, and this is the row.

### 3.2 The one shape change from the row it copies, and it is the point

`hatch_red.py`'s arms **write into `crates/`** and restore it, so the whole
directory is at risk from them and `hatch_red_run`'s interlock is
`git diff HEAD -- crates/`.

`ladder_red.py`'s arms **write nothing there** — `EXPR_RS` is repointed at a temp
file under `work/` — and read exactly **one** tree file. So `ladder_red_run`'s
interlock is that one file, `crates/c2-il/src/func/body/expr.rs`. Copying the
wide interlock would have produced a row that `REFUSED`s for reasons having
nothing to do with it, and a row that refuses for unrelated reasons is a row
nobody reads.

It is still `REFUSED` and not `FAIL`, for the row above's reason: a dirty
`expr.rs` is a property of the **tree** — a peer editing the width table would
move `G1 REAL-TABLE`'s counts — and reddening a peer's gate for their own work in
progress is not something this row may do. `REFUSED` exits 0 and forfeits the
unqualified headline.

**Both rows can refuse at once**, because a dirty `expr.rs` trips the wide
interlock and the narrow one. The suffixes are therefore **appended, not
merged** — `GATE: PASS (HATCH-RED REFUSED) (LADDER-RED REFUSED)` — because a
merged suffix could not say which row refused. Pinned by a selftest case
(`bothrows-refused-print-both-suffixes`).

### 3.3 Eleven words, none shared with the row above, and the non-sharing is asserted

| word | verdict | meaning |
|---|---|---|
| `LADDER-NO-LOG` | NO-RESULT | no output at all |
| `LADDER-TRUNCATED` | FAIL | fewer arms ran than the file declares |
| `LADDER-VACUOUS` | FAIL | red or green is zero, or they do not account for the total |
| `LADDER-EXIT` | FAIL | reported every arm passing and then exited non-zero |
| `LADDER-ARMS-FAILED` | FAIL | a guard did not fire |
| `LADDER-UNRECOGNIZED` | NO-RESULT | an unenumerated outcome |
| `LADDER-MISSING` | NO-RESULT | `ladder_red.py`, or `python3`, is absent |
| `LADDER-NOSUBJECT` | NO-RESULT | `ladder.py` is absent |
| `LADDER-NOGIT` | REFUSED | not a checkout, so the interlock is unreadable |
| `LADDER-DIRTY` | REFUSED | the width table differs from `HEAD` |
| `LADDER-RESIDUE` | FAIL | the arms modified `crates/` |

**`LADDER-NOSUBJECT` exists because of trap B, not for tidiness.** Without it, an
absent `ladder.py` makes every arm fail on an import error and the row reads
`LADDER-ARMS-FAILED` — *"the guards stopped working"* when the truth is *"the
instrument is gone"*. Two different facts get two different words.

The cross-row distinctness is **asserted**, not hoped for:
`ladderred-shares-no-word-with-hatchred` runs both classifiers over all twelve
refusal shapes and requires **12 distinct leading words across 12 shapes**.

### 3.4 Every one of the eleven fired, and the five that `--selftest` cannot reach

`--selftest` drives the classifier and the ruling with fabricated tuples — that
covers six of the words. The **runner**'s five touch `git` and a real tree, so
`work/w-hatchroot/row_red.sh` fires them against **real scratch checkouts** built
in `$TMPDIR`, never against this one. It **extracts `ladder_red_verdict` and
`ladder_red_run` from `scripts/gate.sh` and sources them** — a red test against a
reimplementation is a red test against a reimplementation — and refuses to run if
the extraction came back under 40 lines or without `ladder_red_run`.

Verbatim, from `work/w-hatchroot/row_red_report.txt`:

```
extracted 96 lines of ladder_red_verdict + ladder_red_run from scripts/gate.sh

CASE missing — expect NO-RESULT / leading word LADDER-MISSING
  VERBATIM: LADDER-MISSING work/w-ladders/ladder_red.py is not in this tree
CASE nosubject — expect NO-RESULT / leading word LADDER-NOSUBJECT
  VERBATIM: LADDER-NOSUBJECT work/w-front3/ladder.py is not in this tree, so there is nothing for the arms to fire
CASE nogit — expect REFUSED / leading word LADDER-NOGIT
  VERBATIM: LADDER-NOGIT this tree is not a checkout, so the interlock cannot be read
CASE dirty — expect REFUSED / leading word LADDER-DIRTY
  VERBATIM: LADDER-DIRTY the width table the arms read differs from HEAD: crates/c2-il/src/func/body/expr.rs
CASE residue — expect FAIL / leading word LADDER-RESIDUE
  VERBATIM: LADDER-RESIDUE the arms modified crates/, which they have no path to do: crates/c2-il/src/func/body/expr.rs
CASE green — expect PASS and an EMPTY detail
  tuple : PASS|5|5|3|2|
  => GREEN as required (5 of 5 declared arms, no detail)

distinct leading words fired: 5 of 5 refusals
ALL 6 CASES PASS — 5 runner refusals fired, 1 green control
```

**`LADDER-RESIDUE` is the one worth arguing about, and the file says so rather
than counting it.** `ladder_red.py` has **no code path** that writes into
`crates/`, so the postcondition cannot be fired by running the real arms at all.
It is fired here by a scratch checkout whose `ladder_red.py` *does* write there —
which is exactly what the postcondition defends against: a future edit, or a
SIGKILL between a write and a restore. That is weaker evidence than a defect
reproduction, and this is the same distinction `hatch_red.py` draws with its
`[INJECTED]` tag.

### 3.5 The mutation table — eight ways to break the row, eight reds

A `--selftest` that passes proves nothing unless breaking the thing it checks
makes it fail, **and which case it makes fail**.
`work/w-hatchroot/mutate_gate.sh` applies eight one-check mutations to a copy of
`gate.sh` and re-runs `--selftest` on each
(`work/w-hatchroot/mutate_gate_report.txt`):

| mutation | case(s) it reddened |
|---|---|
| M1 drop the `LADDER-TRUNCATED` check | `ladderred-short-run-is-not-a-pass` |
| M2 drop the `LADDER-VACUOUS` check | `ladderred-no-red-arms-is-vacuous` |
| M3 drop the `LADDER-EXIT` check | `ladderred-green-then-nonzero-exit` |
| M4 an unrecognized log falls through to `PASS` | `ladderred-junk-log-is-no-result` |
| M5 an absent ladder tuple is allowed | `ladderred-absent-tuple-fails-the-gate` |
| M6 `REFUSED` reddens the gate | `ladderred-refused-exits-zero`, `bothrows-refused-print-both-suffixes` |
| M7 the headline suffix goes away | `ladderred-refused-exits-zero`, `bothrows-refused-print-both-suffixes` |
| **M8 the word collapses onto `hatch-red`'s `ARMS-FAILED`** | `ladderred-failed-arm-is-a-fail`, **`ladderred-shares-no-word-with-hatchred`** |

**8 of 8 red, 0 survivors, each on the case its guard belongs to.** Each mutation
removes exactly ONE check, so the case that reddens is the case whose guard was
removed — trap A held fixed by construction. **M8 is trap B in person**, and it
is the one that fires two cases: the assertion on the word, *and* the cross-row
distinctness assertion. That second one is what a lane with two of six silent
mutation passes did not have.

### 3.6 And the mutation harness found a defect in itself first

The mutant was first written to `scripts/.gate-mutant.sh`. Four **reaper** cases
then failed on every mutation — **including an unmutated copy**:

```
unmutated, as a copy at .gate-mutant.sh: exit 1, reddened:
  pid-live-gate-is-kept reaper-keep-0-still-spares-live reaper-keeps-recent reaper-reports-a-count
```

`gate_pid_live` decides a pid is a live gate by grepping `/proc/<pid>/cmdline`
for `gate\.sh`, and those four cases drive it against **this very shell** — whose
`$0` was now `.gate-mutant.sh`, which does not contain that substring. A **noise
floor of 4**, which would have made a *surviving* mutation look red.

Renamed to `.mut-gate.sh` (which does contain `gate.sh`), and the harness now
carries an **M0 control run** that MEASURES the floor instead of assuming it:

```
unmutated, in place: 138 cases, PASS
unmutated, as a copy at .mut-gate.sh: exit 0, reddened: NONE
```

Every mutation's reddened set is read against that control, so a mutation that
reddens nothing beyond it is reported as **SURVIVED**. This is the third time
this week an instrument on this project has been wrong in a way that read as a
result; it is recorded here rather than quietly fixed.

### 3.7 One `gate.sh` line changed that is not the new row

`hatch_red_run`'s `--list` invocation now `cd`s to `repo_root`. It did not
before. `hatch_red.py` resolves its tree from the invoking cwd as of §2.4, so a
gate launched from outside the repository would have read an arm count of **0** —
and `0` is `TRUNCATED`'s own trigger, so the row would have gone `FAIL` for a
reason with nothing to do with the arms. The real run below it already `cd`ed;
that line did not. **This is a defect the fix in ITEM 1 would have introduced,
found by reading the caller rather than by running it.**

---

## 4. What the PREREG got right and wrong

`work/w-hatchroot/PREREG.md`, committed at `7254d1e8` before either item was
written. Every row registered a prediction **and the direction I expected to be
wrong in**; the second half is the falsifiable one.

| # | prediction | outcome |
|---|---|---|
| 1.1 | cwd-derived root + a containment check; the **lexical** version is wrong because worktrees live inside the main repo's path | **HIT**, and the registered direction is the one that held — the main-cwd/worktree-script direction is the hole, demonstrated in §2.3 |
| 1.2 | two words, `NOREPO` checked first, so the `FOREIGN-ROOT` arm needs a **real** second repository | **HIT** — `A3` builds one with `git init` |
| 1.3 | the announcement collides with nothing the gate classifier greps | **HIT**, checked against the real log rather than assumed |
| 1.4 | 11 → 14 arms, 9 → 11 red, 2 → 3 green, 8 → 10 words; **the green control is what will fail first** | **HIT on the counts.** The green control did not fail — but it *is* the arm the counterfactual shows failing at base, which is the same fact from the other side |
| 1.5 | `hatch_red.py` has the same defect and is strictly more destructive | **HIT** |
| 1.6 | `ladder.py` / `ladder_red.py` carry it too; **I expect to decline these** and be wrong to | **DECLINED, and named as residue** — board **#1499**. See §5 |
| 2.1 | the row copies hatch-red's shape but the interlock must be **narrow** | **HIT**, §3.2 |
| 2.2 | `decide()` grows an 8th argument; **eleven** selftest call sites must gain it | **MISS on the count — there are ten**, and the tenth is inside `hr_decide`, which is why it needed a defaulted parameter rather than a literal |
| 2.3 | ten distinct words; **two will turn out unreachable rather than merely unfired** | **MISS, and in the registered direction.** Eleven words, not ten (`LADDER-NOSUBJECT` was not foreseen), and **one** is a postcondition rather than a path — `LADDER-RESIDUE`. `LADDER-NOGIT` turned out perfectly reachable |
| 2.4 | both suffixes printed, ugly rather than wrong | **HIT**, pinned by a case |
| 2.5 | the master-gate diff is **exactly one line** | **HIT on the substance, and the registered failure mode did not occur.** One ADDED line, the new row; the second hunk is the `/tmp/c2rs-gate-$$` run-directory path, which differs between any two runs of the same gate. The `hatch-red` row read `14/14` under MASTER's gate too, which is what kept it to one line |
| 2.6 | the 18 lanes' counts unchanged, digit for digit | **HIT** — 18 rows at `289/289`, match 146/138/136/142/18, 5,202 fixture-verdicts, sweep 19,460/19,556, cross 90,424/90,812, all identical under both gates |
| S.1 | `crates/` empty; the two scan columns are one measurement | **HIT**, and labelled as one measurement |
| S.2 | `#[test]` +0, targets +0, selftest count up | **HIT** — 1,207 / 36 / 120 → 138 |
| S.3 | `peerkeys.py` 0 vanished, 0 moved | **HIT** |
| "the thing I most expect to get wrong" — the **interaction**: an arm added and not declared in `ARMS`, or vice versa | **Neither happened**, and the reason is that the count is read from `--list`: `_hr_exp` and the run's own total are two readings of the same table, so a disagreement is `TRUNCATED` from whichever side it came. What the PREREG said nothing could catch — a typo'd `want` word — did not occur, and **every word in this document is copy-pasted from a terminal rather than typed** |

Two misses, both published, both in the registered direction.

---

## 5. What this lane deliberately did NOT do

1. **`work/w-front3/ladder.py` and `work/w-ladders/ladder_red.py` still derive
   `ROOT` from `__file__`.** Board **#1499**. Both only *read* — `EXPR_RS` — so
   the failure mode is a wrong width table and therefore a wrong RENAME
   classification, not a destroyed tree. It is one screen of work and the PREREG
   registered that declining it might be the wrong call. **The reason it is safe
   to defer**: `gate.sh` invokes both from `repo_root` with an explicit `cd`, so
   the gate row cannot hit it; only a human typing a relative path can.
2. **`work/w-cmp/substitute.py` is still uncovered** — board #1406's row already
   says so and this lane does not change it.
3. **`gate.sh`'s concurrency defaults were not touched** (`jobs=16`,
   `C2RS_JOBS=8`), per the brief and per `w-cache`'s measurement.
4. **Nothing under `crates/` was opened.** `crates/c2-harness/src/gap/`,
   `TuResult`'s verdict types and `crates/c2-il/src/func/body/shapes/calls.rs`
   belong to peer lanes this wave, and `crates/c2-core/src/codegen/coff.rs` was
   not opened at all.

---

## 6. What the next lane inherits

1. **`hatch.py` now prints its target repository on every command.** If you are
   climbing a ladder, read that line before you read the rungs — and keep
   `sha256sum`ing the hatched binary anyway (#1460's own tell), because the guard
   covers the cross-tree case and not, say, a build that silently no-op'd.
2. **`ladder.py` still resolves `ROOT` from `__file__`** (#1499). One screen.
3. **A new `gate.sh` instrument row costs ~20 lines of classifier, ~35 of runner
   and 18 selftest cases**, and the `--selftest` floor must move with it. The
   mutation harness in `work/w-hatchroot/mutate_gate.sh` generalises: point its
   `run_mutation` fragments at the new checks.
4. **If you copy `gate.sh` to mutate it, put the copy at a path containing
   `gate.sh`** (§3.6), and run an unmutated control first. There is no way to
   deduce that noise floor; it has to be measured.
5. **The arm count is read from `--list`, and that is what made the master-gate
   diff readable.** A row that hard-codes its expected count makes every future
   arm addition a two-line diff in a peer's verdict block instead of zero.
