# PREREG — lane `w-gate3048`

**Frozen 2026-08-13, BEFORE any edit to `scripts/gate.sh`.**
Base: `31a83377` (master tip at dispatch, `merge w-keygen`).
Worktree: `.claude/worktrees/w-gate3048`, branch `wt-w-gate3048`.

Mission: close board **#3048** — `scripts/gate.sh` writes a file into its own
`GRADED_DIRS` while it runs, which moves the graded-tree identity between the
run's two ends, and the tree interlock (#2943) then correctly declares the run
evidence about neither tree. Self-triggering exactly once per **fresh**
worktree, which is what every lane starts from.

---

## §0. The state I froze against (measured, not recalled)

Measured in the cold worktree before any edit:

| fact | value |
|---|---|
| files under `crates fixtures scripts` (filesystem walk) | **723** |
| graded-tree identity, cold | **`16da2ec97bd0`** |
| files git does not ignore under those dirs | **723** (identical — a cold tree has no byproducts) |
| `scripts/__pycache__` present | **no** |

Measured in the **main repo** (warm, many gate runs old), same instant:

| fact | value |
|---|---|
| files under `crates fixtures scripts` | 726 |
| git-visible (tracked + untracked-not-ignored) | 723 |
| the difference — every file, by name | `scripts/__pycache__/gt_dump.cpython-314.pyc`, `scripts/__pycache__/sweep_gen.cpython-313.pyc`, `scripts/__pycache__/sweep_gen.cpython-314.pyc` |

Two things that measurement already says and #3048 does not:

* the writer is **not one module**. `gt_dump.py` has a cache too, so at least
  **two** of `scripts/`' python modules are imported during a gate.
* the `313`/`314` pair says the trigger is **not only** "fresh worktree". An
  **interpreter upgrade** mints a new `.pyc` name in a tree that already has
  one, so a *warm* tree re-triggers the void once per python minor version.
  #3048's "exactly once per worktree" is a lower bound.

---

## §1. Predictions (probability form)

### 1.1 The reproduction

| # | claim | P |
|---|---|---|
| R1 | Run 1 of the **unfixed** gate in this cold worktree prints `*** THE TREE MOVED UNDER THIS RUN` and exits non-zero | **0.92** |
| R2 | The identity moves **723 → 724 or more**, never downward | 0.95 |
| R3 | **Every** file that joined the set is under `scripts/__pycache__/` | 0.80 |
| R4 | At least one file that joined is `sweep_gen.cpython-3*.pyc` | 0.90 |
| R5 | The run's own verdict *before* the interlock is `GATE: PASS` — i.e. the void is not a lane failure in disguise | 0.85 |

R3 is the one I most expect to be wrong, and it is the reason step 2 of the
method is a **before/after file-list diff of the whole graded set** rather than
a look at `__pycache__`. If R3 is false the extra names are the answer to
requirement 4 and I report them instead of guessing at them.

### 1.2 The fix

Intended fix (frozen here so it cannot be back-fitted): **the graded set becomes
"every file on disk under `GRADED_DIRS` that git is not explicitly ignoring"** —
`find` as today, minus `git ls-files -o -i --exclude-standard`. Fail-open: a
byproduct nobody gitignored is still graded, and a tracked file is graded no
matter what pattern it matches.

| # | claim | P |
|---|---|---|
| F1 | With the fix, run 1 in a **fresh** worktree reports an **identical** identity at both ends | **0.90** |
| F2 | The identity's **file count** in a cold fixed worktree is unchanged at 723 (the fix subtracts nothing that exists cold) | 0.95 |
| F3 | An untracked, non-ignored file created under `crates/` **still** moves the identity — the interlock is not weakened | **0.97** |
| F4 | The pre-fix hash function, read out of git and run on the same synthetic tree, **does** move — the new arm is red-capable and has been watched red | **0.90** |
| F5 | `--jobs`-level runtime of the gate is unchanged within noise (the fix adds one `git ls-files`, ~10 ms) | 0.95 |

### 1.3 Test-count DELTA — the number this lane is scored on

Base at `31a83377`: **1,527 passed / 41 targets** (`cargo test --workspace
--release --no-fail-fast`).

| # | claim | P |
|---|---|---|
| T1 | `cargo test` delta is **exactly 0 / 0** — 1,527 passed, 41 targets. This lane touches no `crates/` file, by seam | **0.93** |
| T2 | A shrunken **target** count (< 41) would mean an earlier target failed, not a partial run, and I report it as a failure rather than as my delta | 1.00 (a rule, not a prediction) |
| T3 | The gate's **tree-integrity arm count** grows from its current value by **+3** (an ignored-byproduct arm, a not-weakened arm, one counterfactual) | 0.70; +2 or +4 are the live alternatives |
| T4 | Every other gate row's counts (18 lanes, sweep `reached`/`graded`, cross `selected`/`graded`, fixture verdicts) are **unchanged** run-to-run, and `mismatch` is **0** everywhere | 0.90 |

### 1.4 Landed claims

| # | claim | P |
|---|---|---|
| L1 | **No landed claim about the PORT is invalidated.** A void run is *ungraded*, not *wrong*; it withdraws evidence, it does not produce a counter-example | **0.97** |
| L2 | The number of landed lane gates whose *reported* run was the void first run is **> 0** | 0.85 |
| L3 | …and it is **not determinable from the repo for most lanes**, because a rung records the gate's counts and verdict but not always both tree hashes | **0.75** |
| L4 | For at least one lane I can decide it positively, because its rung quotes both hashes | 0.70 |

If L3 holds, the honest report is a **partition** — decidable-sound,
decidable-void, undecidable — with the undecidable bucket named and counted,
**not** an assumption in either direction.

---

## §2. What I am NOT claiming, in advance

* This does not make a green gate mean more than it meant. Trap 1 stands: a
  sound run is still coverage-bounded differential testing.
* It does not close the *class* "a gate row writes into the tree it grades". It
  closes the instances the repo has declared byproducts by gitignoring them.
* It says nothing about a writer that modifies a **tracked** file and restores
  it before the second hash. Neither scheme can see that, and I will say so.

---

## §3. Decline clause

I decline the `git ls-files` rule and fall back to
`PYTHONPYCACHEPREFIX=<outside GRADED_DIRS>` plus publishing the excluded list,
**if any of these is true at measurement time**:

* **D1** — the set of ignored files under `GRADED_DIRS` on a warm tree contains
  anything that is a **grading input** rather than a byproduct (a captured
  `.il`, a fixture obj, a generated corpus the gate reads). Excluding an input
  from the identity is a real weakening and I will not take it silently.
* **D2** — the fix makes any existing tree-integrity arm go red, or the new
  counterfactual arm (F4) cannot be made to go red, i.e. I cannot demonstrate
  the control is capable of failing.
* **D3** — `git ls-files` costs more than 1 s on the real tree, which would put
  a git invocation in a hot path of a script that hashes on every call.

I decline the whole lane, ship the reproduction and the enumeration as a
measurement-only rung, and file the fix as a board row, if:

* **D4** — the reproduction does not reproduce (R1 false) in a genuinely cold
  worktree. A fix for a bug I cannot make happen is a fix I cannot verify, and
  requirement 3 of the brief says so directly.
* **D5** — closing this requires touching `crates/`, `docs/whitebox/`,
  `coff.rs` or `codegen/labels.rs`. Those are other lanes' seams.

---

## §4. Method, frozen

1. Cold worktree, unfixed gate: snapshot the full graded file list, run
   `scripts/gate.sh --require-graded`, snapshot again. The **diff of the two
   lists** is the answer to "what else writes into `GRADED_DIRS`" — measured,
   not enumerated from reading.
2. Apply the fix + the arms. Commit.
3. **A second, genuinely fresh worktree** off the fixed branch. Run
   `scripts/gate.sh --require-graded` **twice**, disclose both, with the
   identity at both ends of each. A cold path is the only place this bug
   exists; a warm-tree demonstration demonstrates nothing.
4. `cargo test --workspace --release --no-fail-fast` (pass **and** target
   count), `scripts/board_audit.sh`, `rung_registry`.
