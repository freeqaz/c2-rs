# PREREG — `w-gatehash` (wave 21, L3)

    Lane:   w-gatehash
    Kind:   instrument
    Base:   1d52f8902  (master, wave-21 dispatch brief)
    Board:  #3863–#3869 (reserved for this lane, and only these)
    Date:   2026-08-29
    Rows:   `gate_identity_diff.sh` WANT_ROWS=21 — must still be 21 at tip

Written and committed **before** `scripts/gate.sh` was edited.

---

## 1. The commission, and the first thing that contradicts it

`#3835` / `WAVE21_BRIEF` §2 L3 both say:

> `scripts/gate.sh` prints its graded-tree hash **twice in one run and nothing
> compares them**.
>
> — and *"The cheap fix is a comparison the gate already has the data for: it
> prints both hashes and could assert their equality in one line."*

**That premise is false, and it was false when `#3835` was filed.** Read at
base `1d52f8902`, `scripts/gate.sh:5836-5843`:

```sh
if [ "$GRADED_TREE_0" != "$GRADED_TREE_1" ]; then
    echo "  *** THE TREE MOVED UNDER THIS RUN — it began at ${GRADED_TREE_0%%:*}"
    ...
    gate_status=1
fi
```

The comparison exists, has existed since `c6d6602e` (lane `w-gatefix`, board
**#2943**, 2026-08-10), and it already sets the exit status to 1.

So the lane's job is **not** to add the missing one-line assertion. It is to
find out why a comparison that exists, fires, and exits 1 was read by a
competent lane as *absent* — and to fix **that**.

## 2. The diagnosis this lane registers, before testing it

The comparison runs **after** `decide`, and `decide` is what prints the
`GATE:` verdict line. So on a run whose tree moved, the transcript reads, in
order:

```
GATE: PASS — 18/18 lanes ran and every one of them graded a corpus, ...
  ...
graded tree: 24afdf51d441  (810 files: crates fixtures scripts)
  *** THE TREE MOVED UNDER THIS RUN — ...
```

`GATE: PASS` is printed, unqualified, over a run the gate has already decided
is evidence about neither tree. Three things in this repo's own standing
method then read past it:

* **`WAVE21_BRIEF` §5 and every dispatch brief before it**: *"Read the `GATE:`
  verdict LINE, never the exit code."* The one artifact lanes are instructed to
  read is the one artifact that does not move.
* **`scripts/gate_identity_diff.sh`**, the required-zero instrument every merge
  touching `crates/` runs, says in its own header that it *"does NOT read
  `GATE:` or any exit status"*. It compares the 21 count-bearing rows — which
  are printed **before** the second hash exists.
* **`#3845`**, last wave: a caveat printed one line *below* the number it
  governs was read past by the coordinator. This is the same shape with ~100
  lines of separation instead of one.

**Registered prediction (falsifiable):** a run whose tree moves mid-flight
prints an unqualified `GATE: PASS` headline at base. If instead the headline
already carries a tree-moved qualifier, this diagnosis is wrong and the lane
reports that and re-diagnoses.

## 3. What will change

**`scripts/gate.sh` only.** No new count-bearing row (`#3691`).

1. Compute the second identity (`GRADED_TREE_1` / `GRADED_IGNORED_1`)
   **before** `decide` rather than after it. Nothing between the last row and
   `decide` writes into `crates/ fixtures/ scripts/` — `decide` reads
   `results.tsv` out of the run dir under `/tmp` — so this does not narrow the
   window the identity covers.
2. Set a **global** `gate_tree_moved` (0/1) plus a one-line detail string.
   Globals are the file's existing idiom for exactly this: `decide` already
   reads `require_graded` and `allow_dirty` the same way, so no call site's
   signature moves and the ten `--selftest` call sites are untouched.
3. Inside `decide`, check it at the **documented seam** — the point the file
   itself marks as *"THE LAST POINT AT WHICH EVERY REMAINING OUTCOME EXITS
   0"*, immediately beside the `require_graded` block. Everything above it
   already returns 1; everything below it returns 0 in some form. One check
   there covers `PASS`, `PASS (SAMPLED)`, `PASS (LANES FILTERED)`, `SKIPPED`
   **and every zero-exit outcome a future lane adds** — which an enumeration of
   today's headlines would not.
4. The headline becomes `GATE: FAIL (TREE MOVED UNDER THIS RUN)` and `decide`
   returns 1. The epilogue block keeps printing both identities and keeps its
   own `gate_status=1`, redundantly and on purpose: if a future refactor drops
   the `decide` check, the run still goes red.

**Why FAIL and not REFUSED.** `REFUSED` exits 0 in this file and that is how
`hatch-red` stayed dead for 1,681 commits. A moved tree is not a condition the
gate declines to evaluate; it is a run whose verdict is already known to be
unattributable, and the caller must not be able to bank it.

**Why not first in `decide`.** A real `FAIL` — a mismatch above all — is
already red, already named, and CLAUDE.md ranks a mismatch above everything.
Pre-empting the mismatch alarm banner with a tooling headline would trade one
unreadable verdict for another. The moved tree is stated in the epilogue on
every path regardless.

## 4. Second file: a `cargo test` target

New file `crates/c2-harness/tests/gate_tree_identity.rs` (not `gate.sh` rows;
not `clause_table.rs`, which is `w-budget`'s this wave). Two tests:

* **`gate_selftest_passes`** — runs `sh scripts/gate.sh --selftest` and requires
  `--selftest: PASS` plus the case count. `gate.sh --selftest` is presently run
  by **nothing**: not by `cargo test --workspace`, not by `gate.sh` itself, not
  by any script in `scripts/`. It is 199 cases (~34 s) that only run when a
  human types them. That is the same defect class one level up.
* **`removing_the_tree_moved_check_reddens_the_selftest`** — copies `gate.sh` to
  a temp dir, deletes the tree-moved check, runs `--selftest` on the copy, and
  requires it to **FAIL**. A check nobody has watched produce a nonzero is not a
  check (`#1236`). The repo tree is never mutated.

New `--selftest` cases inside `gate.sh` (its case floor is raised in the same
commit, per the file's own rule):

* tree-moved turns a would-be `PASS` red, and `GATE: PASS` appears **nowhere**
  in that output;
* tree-moved turns a would-be `SKIPPED` red (toolchain absence over a moving
  tree is still unattributable);
* tree-moved turns a would-be `PASS (SAMPLED)` red;
* a **control**: an unmoved tree still says `PASS`, so the case above is not
  passing because the gate reddened everything.

## 5. What would falsify this lane

| # | claim | falsifier |
|---|---|---|
| F1 | the base headline is unqualified `PASS` when the tree moves | a base run with a moved tree whose `GATE:` line already names it |
| F2 | the fix reddens the headline | tip run with a moved tree still printing `GATE: PASS` anywhere |
| F3 | the fix is byte-neutral | any `crates/` byte changes; any mode-lane count moving |
| F4 | `gate_identity_diff.sh` still enumerates 21 rows | `--self-test` or a real base/tip diff reporting any other count |
| F5 | the new check can go red | deleting it leaves `--selftest` green |
| F6 | the check does not fire spuriously | a clean, unmoved tip run printing `TREE MOVED` |

## 6. Predicted reach and byte delta

**Predicted reach 0. Predicted byte delta 0.** Nothing under `crates/c2-core`
is touched; the only `crates/` file added is a test. Both will be shown to have
held by a base/tip identity diff over the 21 count-bearing rows, not asserted.

## 7. `#3834`, the conditional second item

E4's boundary screen finds boundaries by **name over `const` items** and cannot
see struct fields. `crates/c2-core/src/surface.rs` is `w-budget`'s this wave and
is **not touched by this lane under any circumstance**. This lane will diagnose
where the screen actually lives and take the fix only if it lands outside that
file; otherwise it reports the diagnosis and declines. A correct decline is a
complete result.

## 8. Method commitments

* Controls watched RED before any verdict from them is quoted (`#3336`).
* The end-to-end demonstration mutates the tree **mid-run** by creating one
  untracked, non-ignored file under `scripts/` — reproducing `w-globset`'s
  808→810 exactly — and removes it afterwards. No tracked file is left dirty.
* No `pkill`/`pgrep -f` on anything matching `gate.sh` or `cargo`: three peer
  lanes are running gates from their own worktrees on this box. Waits are on a
  PID with `kill -0`, and every wait is bounded.
* Transcripts committed under `work/w-gatehash/` are sanitised of absolute
  paths and verified with `scripts/tracked_artifact_audit.sh`'s own regexes.
