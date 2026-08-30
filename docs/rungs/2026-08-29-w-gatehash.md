# w-gatehash — the comparison `#3835` asked for already existed; what it could not reach was the verdict line

    Tag:       w-gatehash
    Slug:      w-gatehash
    Date:      2026-08-29
    Kind:      instrument
    Outcome:   instrument
    Fixtures:  none — this rung ships an instrument, not an accepted class: a
               refusal inside `scripts/gate.sh`'s own verdict, plus the first
               thing that has ever run `gate.sh --selftest`. No fixture can
               reach it; it is graded by a mutation that must redden the suite.
    Census:    +0
    Fail axis: **the suite must go RED when the check is deleted.** Not "the
               check is present" and not "the gate exits 1" — the base gate
               already exited 1 for eighteen days while every transcript said
               `GATE: PASS`. The axis is a mutant `gate.sh` with the new block
               cut out of `decide` and nothing else touched, required to fail
               `--selftest` **with both controls still green**, so a check that
               merely reddened everything would not satisfy it.
    Record:    board #3863–#3869; prereg `work/w-gatehash/PREREG.md`;
               transcripts `work/w-gatehash/gate_{base,tip}_{clean,moved}.txt`;
               `crates/c2-harness/tests/gate_tree_identity.rs`

---

## 1. The commission was wrong, and finding out how was the work

`#3835` and `WAVE21_BRIEF` §2 L3 both say the gate *"prints its graded-tree
hash twice in one run and **nothing compares them**"*, and `#3835` prescribes
the fix: *"a comparison the gate already has the data for … it could assert
their equality in one line."*

**Read at base `1d52f8902`, `scripts/gate.sh:5836-5843`:**

```sh
if [ "$GRADED_TREE_0" != "$GRADED_TREE_1" ]; then
    echo "  *** THE TREE MOVED UNDER THIS RUN — it began at ${GRADED_TREE_0%%:*}"
    …
    gate_status=1
fi
```

The comparison has been there since `c6d6602e` (lane `w-gatefix`, board
**#2943**, 2026-08-10) and it has set the exit status to 1 the whole time.
Writing the one-line assertion would have been a no-op on a live defect.

So the lane's question became: **why did a competent lane, looking at its own
evidence, read a check that fires and exits nonzero as absent?**

## 2. Reproduced before anything was changed

`work/w-gatehash/run_moved_gate.sh` launches a full
`--jobs 16 --require-graded` gate and, 45 s in, creates **one** untracked,
non-gitignored file under `crates/` — `w-globset`'s accident, on purpose.

At base (`work/w-gatehash/gate_base_moved.txt`):

| line | text |
|---|---|
| 16 | `graded tree: c1eb31f530bd  (810 files under crates fixtures scripts…)` |
| **104** | **`GATE: PASS — 18/18 lanes ran and every one of them graded a corpus,`** |
| 112 | `graded tree: 7ca8045cb592  (811 files: …)` |
| 118 | `  *** THE TREE MOVED UNDER THIS RUN — it began at c1eb31f530bd` |
| 126 | `GATE_EXIT=1` |

**The check fired, the run exited 1, and the headline said `PASS` fourteen
lines earlier.** That is the whole of `#3835`, and it is not a missing
comparison. Every reading convention this project owns points at line 104:

* **the briefs**: *"Read the `GATE:` verdict LINE, never the exit code"* —
  standing method in `WAVE21_BRIEF` §5 and in every brief before it, and it is
  correct advice for its own reason (`REFUSED` exits 0, so a status is not
  evidence);
* **`scripts/gate_identity_diff.sh`**, which every merge touching `crates/` is
  bound by, states in its own header that it reads *"NOT … `GATE:` or any exit
  status"*. Its 21 count-bearing rows are cut from output printed **before**
  the epilogue's second hash exists.

### 2.1 And the merge instrument is blind to it — measured, not argued

Run the required-zero diff between this lane's clean tip table and its **void**
one, the run the gate itself declared evidence about neither tree:

```
$ bash scripts/gate_identity_diff.sh work/w-gatehash/gate_tip_clean.txt \
                                     work/w-gatehash/gate_tip_moved.txt
count-bearing rows: 21 base, 21 tip (enumerated, not asserted)
IDENTITY DIFF: 0 lines over 21 rows — required-zero byte delta HOLDS
exit 0
```

**A void run passes the merge gate's identity diff, silently, at exit 0.** The
diff is not wrong to do this — it is documented not to read verdicts — but it
means the headline was the *only* place a moved tree could be stopped, and it
was the one place that did not say so.

## 3. What now goes red

`scripts/gate.sh`, and nothing else in `crates/` outside a new test file.

1. **The second identity is taken BEFORE `decide`**, not after. The window it
   covers does not narrow by one writer: between the last row and `decide`
   there is a single `printf` of the disk low-water mark, and `decide` reads
   `results.tsv` and the per-row logs out of `$work` (`/tmp/c2rs-gate-$$`),
   which is not under `GRADED_DIRS`. Every row that *can* write into
   `crates/ fixtures/ scripts/` — the lane leg, the sweep, the cross, the debug
   row, `hatch-red`'s splice — has already finished.
2. **A global `gate_tree_moved`**, which is the idiom `decide` already uses for
   `require_graded` and `allow_dirty`. No call site's signature moves and the
   ten `--selftest` `decide` call sites are untouched.
3. **The check sits at the seam `--require-graded` documents** as *"THE LAST
   POINT AT WHICH EVERY REMAINING OUTCOME EXITS 0"*. Everything above returns
   1; everything below returns 0 in some form. One check there covers `PASS`,
   `PASS (SAMPLED)`, `PASS (LANES FILTERED)`, `SKIPPED` **and every zero-exit
   outcome a future lane adds** — which an enumeration of today's headlines
   would not.
4. The headline is **`GATE: FAIL (TREE MOVED UNDER THIS RUN) — NOTHING ABOVE IS
   EVIDENCE ABOUT ANY TREE.`**, followed by both identities, both file counts,
   and the two named causes (`#3835`'s authoring-in-the-worktree, `#3048`'s
   first `__pycache__`).

**FAIL and not REFUSED, said out loud.** `REFUSED` exits 0 in this file, and
exiting 0 is how `hatch-red` stayed dead for 1,681 commits. A moved tree is not
a condition the gate declines to evaluate; it is a verdict already known to be
unattributable, and the caller must not be able to bank it.

**Not placed first in `decide`, also deliberately.** A real `FAIL` — a mismatch
above all — is already red and already named, and CLAUDE.md ranks a mismatch
above every other piece of work. A tooling headline displacing the alarm banner
would trade one misread verdict for another. This is asserted as a case
(`tree-moved-does-not-mask-a-mismatch`), not left to the comment.

**The epilogue block stays**, with its own redundant `gate_status=1`: if a
future edit moves, renames or drops the check inside `decide`, the run still
exits nonzero rather than returning to eighteen days of green transcripts.

## 4. The same experiment, at tip

Identical script, identical flags, identical mutation:

```
line  20  graded tree: 1f3b2ab67571  (811 files under crates fixtures scripts…)
line 108  GATE: FAIL (TREE MOVED UNDER THIS RUN) — NOTHING ABOVE IS EVIDENCE ABOUT ANY TREE.
line 109    It began at 1f3b2ab67571 (811 files) and ended at 686b3cdc0916 (812 files).
line 137  GATE_EXIT=1
```

(That run was taken at `f0440e5a8`; the later flake fix in §9.1 moved the clean
tip identity to `9bf0f95134d3`, which is the figure §8 and §9 quote. The
moved-run transcript is kept at the tree it was taken on rather than re-taken,
and its identity is printed in it — which is the whole point of the
instrument.)

`work/w-gatehash/gate_tip_moved.txt`. Base said `PASS`; tip says `FAIL` in the
line that gets read.

## 5. Watched red before it was trusted (`#1236`, `#3787`, `#3336`)

`#3787` is the case where a checker printed the defect, printed `CLEAN`, and
exited 0 — one line apart. So the new cases were run against a **mutant**
`gate.sh` built by `work/w-gatehash/mutate.sh`: the tree-moved block is cut out
of `decide` and **nothing else is touched**, so the mutant is byte-for-byte the
pre-fix behaviour — the epilogue comparison is still there, it still exits 1,
and the headline still says `PASS`.

```
$ sh target/wg_mutant_gate.sh --selftest
  FAIL  tree-moved-turns-pass-red      wanted FAIL, got PASS — GATE: PASS — 2/2 lanes ran …
  ok    tree-still-keeps-pass          GATE: PASS — 2/2 lanes ran …
  FAIL  tree-moved-turns-skipped-red   wanted FAIL, got PASS — GATE: SKIPPED — all 2 lanes …
  ok    tree-still-keeps-skipped       GATE: SKIPPED — all 2 lanes …
  FAIL  tree-moved-turns-sampled-red   wanted FAIL, got PASS — GATE: PASS (SAMPLED) — 2/2 …
gate.sh --selftest: FAIL — 11 of 205 checks did not behave as required.  (exit 1)
```

`work/w-gatehash/selftest_mutant.txt`. **The three headline cases go red and
both controls stay green** — which is the assertion that separates a working
check from one that simply fails everything, and it is asserted mechanically in
the test below, not read off this transcript.

The mutant reproduces the historic defect exactly: `GATE: PASS`,
`GATE: SKIPPED` and `GATE: PASS (SAMPLED)` over a tree that moved.

## 6. Six `--selftest` cases, floor 199 → 205, every headline paired with a control

| case | want | asserts |
|---|---|---|
| `tree-moved-turns-pass-red` | FAIL | the headline names it; `GATE: PASS` appears **nowhere**; both counts are carried |
| `tree-still-keeps-pass` | PASS | **control** — same tuples, unmoved tree, still `GATE: PASS` |
| `tree-moved-turns-skipped-red` | FAIL | all-skip over a moved tree is red, and `GATE: SKIPPED` is not also printed |
| `tree-still-keeps-skipped` | PASS | **control, the load-bearing one** — CLAUDE.md's degrade-cleanly path is untouched on a box with no toolchain |
| `tree-moved-turns-sampled-red` | FAIL | the third zero-exit outcome, which a fix aimed at the word `PASS` would miss |
| `tree-moved-does-not-mask-a-mismatch` | FAIL | a mismatch under a moved tree still reaches the `ALARM` banner and still names the lane |

`run_case` clears the flag after every case and `gate_tree_moved` is reset to 0,
so the `decide` call sites that do not go through `run_case` cannot inherit it —
a per-case flag that leaks is a selftest whose later cases assert the wrong
thing.

## 7. `gate.sh --selftest` was run by nothing, and now it is

Measured at base: `grep -rn selftest crates/ scripts/` finds every other
instrument's self-test wired into something — `gt_dump.py --selftest` is a gate
row, `tracked_artifact_audit.sh` and `subsys_metrics.sh` have their own — and
**`gate.sh --selftest` wired into nothing.** Not `cargo test --workspace`, not
`gate.sh` itself, not `scripts/gate_identity_diff.sh`, not any script under
`scripts/`. 205 cases, no toolchain, no compiler, ~35 s, and they ran when a
human typed them, which on this project means they ran when somebody was
already suspicious. That is the same defect one level up: a check nobody runs
and a check nobody reads are the same check.

`crates/c2-harness/tests/gate_tree_identity.rs`:

* **`gate_selftest_is_green_and_has_not_shrunk`** — runs it, requires the `PASS`
  summary *and* a case-count floor of 205. The floor is asserted here as well as
  inside the script, because a floor that only lives inside the thing it
  measures cannot tell anyone the suite shrank.
* **`deleting_the_tree_moved_check_reddens_the_selftest`** — performs §5's
  mutation hermetically and requires the suite to fail, the three named cases to
  be among the failures, and **both controls to remain green**. The mutant is
  written to `target/`, which is gitignored *and* outside `GRADED_DIRS`, so the
  test cannot move the graded tree of a gate running in the same worktree.
  `gate.sh` derives `repo_root` as `dirname $0/..`, so a copy one level below
  the root still resolves the real repository.

**Not a `gate.sh` row (`#3691`).** A 22nd count-bearing row makes
`gate_identity_diff.sh` exit 2 and refuse to diff for every lane on a 21-row
base. `#1406` names `cargo test` as the other admissible home, and nothing in
this file prints a `<name> PASS <n> <n>` line.

## 8. The 21-row verification, which is the one way this lane could have broken every future lane

```
$ bash scripts/gate_identity_diff.sh --self-test
  enumeration: 21 count-bearing rows (hatch-red/ladder-red dropped)
  control: a table against itself                      -> 0 lines, exit 0
  #3515's one-TU-refused signature                    -> 14 lines, 7 rows
  the signature case exits NONZERO
  a TRUNCATED table -> exit 2 (a short extraction is not 'no differences')
SELF-TEST PASS: enumeration 21, control silent, #3515's signature found
  exactly (14 lines / 7 rows) and nonzero, truncation refused.   (exit 0)

$ bash scripts/gate_identity_diff.sh work/w-gatehash/gate_base_clean.txt \
                                     work/w-gatehash/gate_tip_clean.txt
count-bearing rows: 21 base, 21 tip (enumerated, not asserted)
IDENTITY DIFF: 0 lines over 21 rows — required-zero byte delta HOLDS   (exit 0)
```

A real base/tip diff, not only the self-test. **Predicted reach 0, realised 0.
Predicted byte delta 0, realised 0** — over 21 enumerated rows on two full
`--jobs 16 --require-graded` runs, base `1d52f8902` and tip.

Base identity `c1eb31f530bd` (810 files) at both ends; tip identity
`9bf0f95134d3` (811 files) at both ends. The one extra file is
`crates/c2-harness/tests/gate_tree_identity.rs`. **Neither run's tree moved** —
which is now a claim the headline would have contradicted rather than a claim
the reader has to assemble from two lines a hundred apart.

`sh scripts/gate.sh --check` also still passes its 16 tree-integrity arms and
its four counterfactuals, unchanged.

## 9. Gate evidence

| lane | result |
|---|---|
| `C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release --no-fail-fast` | **63 targets, 2,016 passed, 1 failed, 2 ignored** |
| — the one failure | `rung_index_is_generated_and_current`, because `docs/rungs/INDEX.md` is regenerated at merge and this rung is new. `WAVE21_BRIEF` §4: *"WILL be red at every lane tip and that is expected, not yours to fix."* Nothing else in 63 targets is red |
| `scripts/gate.sh --jobs 16 --require-graded` (base, clean) | `GATE: PASS`, 18/18, identity `c1eb31f530bd` (810 files) at **both** ends |
| `scripts/gate.sh --jobs 16 --require-graded` (tip, clean) | `GATE: PASS`, 18/18, identity `9bf0f95134d3` (811 files) at **both** ends |
| the same gate, deliberately broken (**tip**, tree moved mid-run) | **`GATE: FAIL (TREE MOVED UNDER THIS RUN)`** on line 108, `GATE_EXIT=1`, 811 → 812 |
| the same gate, deliberately broken (**base**, tree moved mid-run) | `GATE: PASS` on line 104, `GATE_EXIT=1`, 810 → 811 — **the defect, reproduced** |
| `bash scripts/gate_identity_diff.sh --self-test` | `SELF-TEST PASS: enumeration 21, control silent, #3515's signature found exactly (14 lines / 7 rows) and nonzero, truncation refused` |
| base/tip identity diff | `21 base, 21 tip (enumerated, not asserted)` · **`0 lines over 21 rows`** |
| `sh scripts/gate.sh --selftest` | `PASS — 205 cases` (was 199) |
| the same, on the mutant | `FAIL — 11 of 205` standalone / **15 of 205** under `cargo test`; controls green in both |
| `sh scripts/gate.sh --check` | 16 tree-integrity arms, 0 failed, 4 counterfactuals resolved |

Both gate rows were re-taken after the last commit, so the tip figures are the
tip's and not an earlier tree's. The clean tip run discharges falsifier **F6**:
the new check does not fire on a tree that did not move.

### 9.1 The suite found a second defect in the first thing it ever ran

`gate.sh --selftest`'s concurrent-publisher case forks two subshells that each
run `resolve_corpus` and then write a result file. `set -e` is on, so a subshell
whose `resolve_corpus` returned nonzero **died before writing**, and the `cut`
after `wait` then aborted the **entire 205-case suite** with

```
gate.sh: line 4292: …/selftest/corpus/r2: No such file or directory
```

and **no case verdict at all** — not a `FAIL`, not a count, the run simply
stopped partway. Run by hand on an idle box it never fired; under
`cargo test --workspace` with three peer lanes' gates on the same machine it
fired on the first attempt.

A losing arm must make that case **report** — the loser discarding is the very
thing being asserted — never kill the suite. Each arm now ends `|| true` and
writes `${C2RS_CORPUS_KIND:-NONE}`, with a `MISSING` floor after `wait`; the
invariant is unchanged and an arm coming back `private`, `NONE` or on a
different directory still fails the case, and now says which.

The other half is in the test: `cargo` runs a target's tests on N threads, so
the green run and the mutant run started together — two 35-second shell suites
racing publishers, for no benefit, since the mutant run is a *controlled
comparison against* the green one. A `Mutex` serialises them.

**This is `#3866`'s own argument, discharged within minutes of making it.** A
suite that only ran on suspicion had a suite-killing flake in it, and the flake
was invisible precisely because nobody ran it under load.

## 10. `#3834` — DIAGNOSED AND DECLINED, with the reason

E4's boundary screen was to be taken *"only if the first lands cheaply"* and
*"if the fix lands elsewhere"* than `crates/c2-core/src/surface.rs`, which is
`w-budget`'s this wave. **It does not land elsewhere, and the diagnosis is
larger than the brief states.**

`boundary_named_consts()` (`crates/c2-core/src/surface.rs:644`) is the screen;
`UNCOVERED`, `UNCOVERED_RATCHET`, `SURFACES` and the test that joins them are
all in the same file, within 90 lines of it. There is no second site.

And the brief's diagnosis — *"finds boundaries by NAME over `const` items, so it
could not see `globset.rs`'s five real boundaries, which are struct fields"* —
is **necessary but not sufficient**. The screen would still miss them after
being taught about fields, for two further reasons:

```
$ grep -n 'pub .*_at_or_below\|pub sentinel_kind\|pub temp_kind' \
      crates/c2-core/src/codegen/globset.rs
533:    pub sentinel_kind: u8,
535:    pub temp_kind: u8,
537:    pub reject_at_or_below: u8,
541:    pub auto_at_or_below: u8,
545:    pub coff_at_or_below: u8,
```

* the screen's `WORDS` list is `MAX MIN TOP LIMIT CEILING FLOOR THRESHOLD BOUND
  CAP`, and **not one of the five field names contains any of them** —
  `at_or_below` is the spelling `globset` actually uses for a bound;
* the screen takes name characters with
  `c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_'`, so a **lowercase**
  field name is truncated to the empty string before the word test is reached.

So `#3834` is three changes to one function — item kind, case, and vocabulary —
each of which widens what E4 demands be covered, and every newly-found boundary
must then be either registered to a surface or added to `UNCOVERED` **with the
ratchet raised**, in `surface.rs`. That is `w-budget`'s file and its ratchet
this wave. Taking it here would have collided on both.

**Reported, not worked around.** `globset.rs:568` already says the same thing in
its own words, and is worth quoting because it is the design consequence rather
than the defect: the module deliberately shipped `GateA::with_auto_bound` as a
**function** rather than a `const A6_BOUND_7`, *because* a const with that name
trips a screen that cannot see the boundary the module actually has. **The
screen is currently shaping the code to avoid it.** That is the reason to fix
it, and it is a stronger reason than the missed coverage.

## 11. What contradicts the brief

1. **`#3835`'s premise.** *"Nothing compares them"* is false and was false when
   filed; the comparison is `w-gatefix`'s and is eighteen days old. `#3835`'s
   prescribed *"cheap fix … one line"* would have been a no-op on a live defect.
   The board row is amended in place (`#3863`) rather than deleted.
2. **The brief's framing of `#3834`** — *"finds boundaries by NAME over `const`
   items"* — is one of **three** reasons the screen misses `globset`'s fields,
   and fixing only the named one would leave the row still open while looking
   closed. §10.
3. **`gate.sh --selftest` was run by nothing**, which no board row records and
   which the brief does not mention. It is arguably a larger hole than `#3835`:
   205 cases, all of the gate's own fail-when-it-should proofs, executed only on
   suspicion. `#3866`.
4. **`gate_identity_diff.sh` passes a void run at exit 0** (§2.1). Not a defect
   in that script — it is documented not to read verdicts — but it removes the
   last fallback and nobody had measured it. `#3865`.
