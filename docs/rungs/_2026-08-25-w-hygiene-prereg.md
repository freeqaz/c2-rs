# PREREG — `w-hygiene`, five named defects, each priced, each blocking something

    Tag:       w-hygiene
    Date:      2026-08-25
    Kind:      construct rung
    Base:      a8593651b  (master, clean)
    Board:     #3539–#3545 (reserved by the coordinator for this lane)
    Fixtures:  none — construct rung: no fixture is claimed and no census key moves
    Census:    +0

**Frozen before any measurement of this lane's own changes.** Predictions below
are never edited afterwards. Navigation pointers may be repaired by *amending
beside*, per `#3495`'s convention; a number or a probability may not.

---

## 0. What this lane is, and the axis on which it can fail

Five defects, each independently landable, taken in the order the brief gives
them. This is a **construct rung**: `Fixtures: none`, `Census: +0`,
**required-zero byte delta**, graded by a strict identity diff of the **21
count-bearing gate rows** against base `a8593651b`.

**THE COST CLAUSE (`#3336`, as amended 2026-08-21) — the axes this rung can
fail on even when every byte is identical.** Three, named before starting:

1. **The oracle's resolution denominator.** Item 2 changes how the harness finds
   `compilers/` and `wibo`. A fix that resolves a *different* toolchain, or
   resolves one where base resolved none (or vice versa), silently changes what
   the judge is — while every emitted byte could stay identical. Observed by:
   the gate's `graded/total` per lane (the 21-row diff already carries it) **and**
   by comparing the resolution banner `c2rs census` prints at base and at tip.
2. **A guard that cannot fire.** Item 5 ships a validator. A validator never seen
   red is decorative (`CLAUDE.md`). Observed by: planting one deliberate
   violation of **each** class and requiring a nonzero exit on each, with the
   planted file removed and the guard green again afterwards.
3. **Build reproducibility.** Item 2's stated purpose is to remove a
   path-length-dependent capture. Observed directly by rebuilding at two
   deliberately different path lengths and comparing `md5` and size — the same
   instrument `#3525` used.

`mismatch 0` is **not** `match unchanged` (`#3515`). The exit code is not the
grade; the 21-row diff is.

---

## 1. ITEM 1 — EXPERIMENT F, the build-to-build cost floor

**Registered elsewhere and NOT re-registered here.** Experiment F's design,
prediction and three-way conclusion rule live in
`docs/rungs/_2026-08-24-w-adjacency-prereg.md` **§8** (`#3523`, `#3525`), and
`docs/rungs/2026-08-24-w-adjacency.md` §7.3. **This lane runs it as registered
and does not touch those predictions.** Restated here for the reader only, as a
pointer, not as a new registration:

> **P5 (w-adjacency's numbering):** `F` = the largest `|mean|` over the pairwise
> readings, with its split. **Prediction: `F` lands in ±0.2 % to ±0.7 %.**
> `< ±0.2 %` → build layout does not explain the 1.8-point swing.
> `±0.2–0.7 %` → a real build floor exists, comparable in size to what the COST
> CLAUSE measures, and every published reading must be quoted beside it.
> `> ±0.7 %` → **every published cost reading in this project is inside the noise
> of its own build.** Scored on the mean **and** the split.

### 1.1 One thing about F HAS changed and is registered here, because it must be

`w-adjacency`'s three binaries **were reaped with its worktree** (`§7.6`'s
failure mode, arriving a second time). This lane therefore **rebuilds all three
arms** at `f6f56df78` in three fresh directories. Consequences, registered:

* The registered `cmp` precondition of prereg §8.2 is a **per-run** check and is
  re-taken here, not inherited. If this lane's three builds come back
  byte-identical, **the experiment degenerates into the existing null arm and is
  reported as that and stopped** — the exact branch §8.2 registers.
* The three directory names reproduce the registered lengths (`b1`, `b2xx`,
  `b3yyyyyy` — 2, 4, 8 characters).
* **P1a — the three builds differ, and size rises monotonically with directory
  name length. p = 0.90.** (`#3525` measured `+32 B` at `+2` chars and `+48 B`
  at `+6` on a different parent path; this lane's parent path differs, so the
  *absolute* sizes are expected to differ from `#3525`'s and the *deltas* are
  not.) The registered failure branch is §8.2's.
* **P1b — this lane's absolute sizes will NOT equal `#3525`'s three
  (6,126,264 / 6,126,296 / 6,126,312). p = 0.85**, because the parent path
  differs in length. If they DO equal, that is a finding about what the capture
  is actually sensitive to, and is reported as one.

### 1.2 The box precondition, and it is a hard gate

F is a timing experiment with three peer lanes live. It runs only on a box
verified quiet by a **streak** predicate (`w-adjacency` §8.3 — an instantaneous
sample lands in the gap between one process exiting and the next starting):
`pgrep -xc cargo` and `pgrep -xc rustc` both 0 and 1-minute load below 4, held
across consecutive samples. `pgrep -x` matches the process NAME and cannot
self-match. **If the box does not go quiet, F is NOT RUN and is reported as
NOT RUN with the load it was held at** — a timing number taken on a loaded box
is void, not provisional.

### 1.3 What F must be run BEFORE

**F must be measured before item 2 lands anywhere it could matter.** F is a
build of the historical commit `f6f56df78`, so item 2's edit cannot reach it by
construction — but the ordering is registered anyway so that no reader has to
reconstruct it.

---

## 2. ITEM 2 — `repo_root()` at RUNTIME

Site: `crates/c2-reference/src/lib.rs:81` (**verified at base**, line number
correct):

```rust
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
```

Shape, from `w-adjacency` §7.7: `C2RS_REPO_ROOT` if set → else walk up from
`current_exe()` / `current_dir()` for a `Cargo.toml` + `crates/` marker → else
the `env!` value as last-resort fallback. **Degrade-cleanly is binding**:
toolchain absent still means `SKIP: toolchain absent`, never a panic.

### 2.1 A reading taken BEFORE this prereg was written, recorded as a reading

`#3525` closes with an explicit warning that the padding mitigation *"rests on
the **untested** assumption that `CARGO_MANIFEST_DIR` is the only
path-length-sensitive capture in the binary."* **That assumption has now been
tested by reading, and this is the read, taken 2026-08-25 at base
`a8593651b` before any prediction below was written.** A NUL-safe walk of every
`.rs` under `crates/` finds **30** `env!(` sites. Of those, **three** are in
non-test `src/` code and therefore live in the release binary:

| site | what it is | in the release binary |
|---|---|---|
| `crates/c2-reference/src/lib.rs:81` | `repo_root()` — resolves `compilers/`, `wibo` | **yes** |
| `crates/c2-harness/src/provenance.rs:44` | `repo_root()` — provenance, **deliberately** compile-time by its own nine-paragraph doc | **yes** |
| `crates/c2-harness/src/lib.rs:494` | `fixtures_dir()` — `../../fixtures/cpp` | **yes** |
| `crates/c2-core/src/plan/mod.rs:539` | inside `#[cfg(test)] mod tests` (opens at `:470`) | no |
| 26 further sites | all under `crates/*/tests/` | no |

So the assumption `#3525` flagged as untested is **false as read**, and the
sub-predictions below are the *measurement* that confirms or refutes the read.

### 2.2 Predictions

* **P2a — the fix lands and a binary relocated out of its build tree resolves
  the toolchain. p = 0.85.** Observed by: build, copy the binary to a directory
  that is not a repo, run the subcommand `#3470` caught, and require a graded
  result rather than `SKIP: toolchain absent`. **Positive control required**: the
  same relocation on a **base** binary must print `SKIP: toolchain absent` and
  exit 0. A test that has not been seen fail on the base binary does not count.
* **P2b — after fixing ONLY `c2-reference::repo_root()`, binary size STILL
  tracks build-directory path length. p = 0.85**, on the §2.1 read: two live
  captures remain. If P2b holds, **`#3470` is closed and `#3525` is only
  PARTIALLY closed**, and this lane says so in those words rather than claiming
  both. The alternative — p = 0.15 — is that the remaining two captures are
  optimised out or land in padding, and size stops moving.
* **P2c — `cargo test --workspace --release` stays at 54 targets with 0
  failures. p = 0.70.** The discount is because `provenance.rs`'s
  `every_worktree_resolves_to_one_main_repo_root` test and `cli_flags.rs:393`
  both assert on a root string, and a runtime resolver can change what they see.
  A red there is a *finding about the semantics*, not a build error, and is
  reported.
* **P2d — the 21-row identity diff is 0 lines. p = 0.90.** `c2-reference` is the
  oracle seam, not the emit path; but a resolution change that altered *which*
  toolchain is found would move `graded` counts, which is failure axis 1.

### 2.3 Decline floor for item 2 — registered

If keeping `main_repo_root()`'s worktree-collapse invariant green requires
changing `provenance.rs`'s deliberate compile-time semantics, **item 2 stops at
`c2-reference` and the `provenance.rs` half is reported as declined with its
price**, not silently extended. `provenance.rs`'s doc argues *for* compile-time
capture on the merits; overriding a documented decision is not this lane's call.
**Padding build paths to a fixed width is ranked LAST and is not done here**
(`#3525`), and §2.1 is now the reason it must not be: its uniqueness assumption
is read-false.

---

## 3. ITEM 3 — `scripts/configure_existing_worktree.sh`, two defects

Both **verified at base**, line numbers correct:

* **`#3500`** — line 28, `MAIN_REPO="$(cd "$(dirname "$0")/.." && pwd)"`; line 46
  compares it against `WORKTREE_PATH` and lines 47–48 print *"is the main repo;
  nothing to configure"* and `exit 0`. For a **sibling-directory** worktree
  invoked through its own copy of the script, those are the same path.
* **`#3516`** — line 202, `./target/release/c2rs bench   # every fixture, the
  correctness gate`. `cmd_bench` → `oracle_selftest` never calls
  `PortC2::build`; `CLAUDE.md` § Layout is the correct statement.

### 3.1 Predictions

* **P3a — the `#3500` defect reproduces on a deliberately planted sibling
  worktree BEFORE the fix. p = 0.90.** Required as the positive control: the
  script must be **seen** printing *"nothing to configure"* and exiting 0 from a
  sibling path. If it does not reproduce, `#3500` is re-read rather than
  patched, and this lane reports that instead.
* **P3b — deriving the main repo from `git rev-parse --git-common-dir` fixes it
  for the sibling case AND leaves the `.claude/worktrees/` case unchanged.
  p = 0.85.** Both cases are exercised; "unchanged" is checked, not assumed.
* **P3c — the `bench` line is replaced by the gate the comment claimed to name.
  p = 0.98.** The replacement must not create a second wrong claim: `bench` is
  kept and re-described as what it is (the oracle self-test), and the
  correctness gate line points at `scripts/gate.sh`.
* **P3d — no `crates/` byte moves for item 3. p = 0.99** (it is one shell file).

---

## 4. ITEM 4 — three NUL bytes

**Verified at base by a NUL-safe read**, and the brief's three line numbers are
correct:

    crates/c2-il/src/func/bundle.rs:3440   b"junkjunk\x00".to_vec()
    crates/c2-il/src/func/bundle.rs:3449   b"junkjunk\x00".to_vec()
    crates/c2-il/src/func/bundle.rs:3458   b"junkjunk\x00".to_vec()

Total NUL bytes in the file: **3**. Fix: `\0` → `\\0`, three sites.

* **P4a — after the fix the file contains 0 NUL bytes and `grep -rn` prints
  matching lines with content rather than `binary file matches`. p = 0.97.**
* **P4b — the three tests still pass and assert the same thing.** `b"junk...\0"`
  with a literal NUL and `b"junk...\0"` with the escape are the **same bytes**,
  so this is a source-encoding change with no semantic content. **p = 0.97.**
* **P4c — `bundle.rs` is the only `.rs` under `crates/` holding a NUL, and after
  the fix the count of such files is 0. p = 0.95** (`#3513` established the
  population; this lane re-derives it rather than inheriting it).
* **The detector must be proved to fire first.** `grep` cannot test for NUL:
  `grep -c $'\0'` counts lines and `LC_ALL=C grep -qP '\x00'` does not fire. The
  check used is a byte-count comparison (`tr -d '\000'`) or Python, and it is
  **run against `printf 'a\0b\n'` and seen to report a NUL** before any of P4a–c
  is believed.

---

## 5. ITEM 5 — the `#3156` funnel check

### 5.1 A reading taken BEFORE this prereg, recorded as a reading, not a prediction

`#3156`'s own prescription is *"a one-line check — `git ls-files` against
`.gitignore`'s own patterns — would have caught all nineteen."* **Run at base
`a8593651b` on 2026-08-25, before this prereg was written:**

    git ls-files -c --ignored --exclude-standard | wc -l   ->  8041

**8,041 tracked files match `.gitignore`.** Essentially all are under `/work`
(`work/w-mmio` 918, `work/emitpred` 277, `work/w-prod` 209, …), which
`.gitignore:24` ignores as `/work` and which ~200 lanes have force-added
evidence into for months. **So the prescription as literally written cannot be a
gate**: it is red at HEAD by 8,041 and would have to be adopted with an 8,041-entry
allowlist, which is not a one-line check.

The **forbidden classes** `CLAUDE.md` actually names — `*.obj`, `*.o`, `*.il`,
`_CL_*`, `*.profraw`, `*.profdata`, `*.pyc` — read **0** tracked files at base,
which is `2c0de2ad4` having worked.

One method note recorded with it, because it is a trap: **`git check-ignore`
does not report tracked files unless given `--no-index`**, so a guard built on
bare `git check-ignore` reports clean over exactly the population it exists to
police.

### 5.2 Predictions

* **P5a — the guard, scoped to the forbidden classes rather than to all of
  `.gitignore`, is green at HEAD. p = 0.95** (0 is already read; the discount is
  for the guard's own pattern set being wider than the read one).
* **P5b — the guard goes RED on a deliberately planted violation of EVERY class
  it claims to cover, one plant at a time, and returns to green when the plant
  is removed. p = 0.90.** Classes to plant, enumerated now so the count cannot be
  chosen after the fact: `.obj`, `.o`, `_CL_*`, `.il`, `.profraw`, an absolute
  `/home/<user>/` path inside a tracked file, and a `compilers/` entry —
  **7 plants**. A class that cannot be planted is reported as uncovered rather
  than quietly dropped.
* **P5c — the guard runs with NO toolchain and needs nothing but `git`.
  p = 0.95.**
* **P5d — the guard's exit status gates what follows it.** It is wired so that a
  red guard fails something, rather than printing into a log nobody reads. Where
  it is wired is decided during the item; **that it is wired somewhere with a
  nonzero exit is registered now. p = 0.85.**

### 5.3 Decline floor for item 5

If wiring the guard into `scripts/gate.sh` turns out to conflict with a peer
lane holding that file, the guard **still ships as a script with a proved-red
self-test**, and the wiring is reported as the one part not done, with its
price. A guard that exists and is tested beats a guard blocked on a merge.

---

## 6. The stretch — `#3510` — and its floor

`emit_set_violations()` reads 1 (`src/system/decomp_pch.cpp`, `fn_total 1260`
vs `emit-emitted 0`). **Attempted only if items 1–5 land cleanly and time
remains, and DIAGNOSIS ONLY** — repairing the ceiling's predicate is explicitly
not in scope. **P6 — not reached. p = 0.60.** Registering the likely
non-attempt is the honest form; a stretch that displaces a funded item is a
failure of this lane, not a bonus.

---

## 7. The grade

| gate | requirement |
|---|---|
| `sh scripts/gate.sh --jobs 4 --require-graded` | `GATE: PASS`, `GATE_EXIT=0`, lane count / verdict count / mismatch count all quoted |
| identity diff | **0 lines over 21 count-bearing rows** vs base `a8593651b`, denominator printed at BOTH ends, `LANE VERDICT graded/total match` cut, `/tmp/c2rs-gate-<pid>` normalised, `hatch-red`/`ladder-red` dropped |
| `cargo test --workspace --release` | **target count recorded** — a dropped target count means an earlier target failed |

**Outcome word**: exactly one of `converted | declined | instrument | built |
FAILED`. A lane that produced none of its deliverable says **FAILED** in those
words. **Registered expectation: `built`, p = 0.70** — the alternative that
worries this lane is item 2 exceeding its price and taking the wave with it,
which the brief pre-empts by making each item independently landable, and which
§2.3's decline floor makes an outcome rather than a stall.
