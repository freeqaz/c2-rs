# w-coldcross — PREREG

Frozen as this lane's FIRST commit, before any contention measurement, before
any gate run, and before any change to `scripts/`. Base: master `abc64be3`
(the dispatch named `1744ced1`; `abc64be3` is two docs-only commits later and
`git diff 1744ced1 abc64be3 -- crates/ scripts/ fixtures/` is empty — checked,
§0 K7).

**No discount factor is applied to any row.** Every probability below is the
number I actually believe. A row I get wrong is scored as wrong, and the rows
most likely to be wrong are the ones about *contention*, which is precisely
what this lane exists to measure rather than reason about.

## 0. What is KNOWN at freeze, and is therefore NOT predicted

Established by reading code, plus two cheap measurements taken before this file
was written. They are recorded here so the prereg cannot later take credit for
"discovering" them. **K8 and K9 are measurements, and they are declared, not
predicted.**

* **K1.** The capture cache root is **already shared by every worktree**:
  `crates/c2-harness/src/cli/gap.rs:116` and `cli/reference.rs:527` both default
  to `c2_harness::provenance::main_repo_root().join("work/capture-cache")`, and
  `provenance::main_repo_root`'s doc comment says so in those words ("the same
  directory from every linked worktree"), with board #181 behind it. **Nothing
  in this lane introduces cross-lane sharing; the sharing exists.**
* **K2.** The cache key (`CaptureCache::key_material` + `new`) is a hash over
  *inputs only*: source bytes, the source **argument string**, the compile
  `cwd` (canonicalized), the flag list, `cl.exe`/`c1xx.dll`/`c2.dll` contents,
  the wibo version, the workload tree's git identity, and the cache root. **The
  c2-rs tree under test is NOT in the key and cannot be** — the cached artefact
  is c2's own obj + IL bundle, which is a function of the toolchain and the
  source, not of the port.
* **K3.** Concurrent same-key captures across processes are already guarded, by
  `KeyLock::acquire` (an `O_EXCL` lockfile), added for exactly the shared-root
  case. It is fail-open (`None` = proceed unguarded).
* **K4.** The 878-TU workload scan **already runs fully warm in a fresh
  worktree**, because its sources live in `../dc3-decomp` — a path that is
  identical from every worktree. So "a lane consuming a peer's cache entries"
  is not a new arrangement being proposed here; it is the standing arrangement
  of this repo's largest instrument.
* **K5.** `scripts/mode_cross.sh` takes its case-directory lock
  (`work/mode-cross/.cross.lock`, `mkdir`) **before** regeneration and releases
  it in an `EXIT` trap, i.e. **it holds the lock for the entire run** — the
  regeneration, the assignment, and all 90,812 gradings.
* **K6.** `scripts/expr_sweep.sh` has the same structure with two locks
  (`$out/.sweep.lock`, which REFUSES, and `$cases/../.cases.lock`, which falls
  back to a private cold set), also held for the whole run. Its header states
  the per-worktree choice was deliberate and names the hazard: *"two lanes with
  different `scripts/sweep.d` would overwrite each other's corpus between
  runs — board #3249's hazard"*.
* **K7.** `git diff 1744ced1 abc64be3 -- crates/ scripts/ fixtures/` is empty.
* **K8 — MEASURED before freeze.** `python3 scripts/sweep_gen.py <dir>
  scripts/sweep.d` writes all 19,556 cases in **0.65 s** wall
  (0.15 s user + 0.46 s sys), in this worktree, load ~2.
* **K9 — MEASURED before freeze.** The generated corpus is **deterministic and
  already identical across worktrees**: two fresh generations in this worktree
  `diff -rq` clean against each other **and** against the main repo's live
  `work/mode-cross/cases` (19,556 files, 0 differences).

**K5 + K8 together are the whole shape of this lane and are stated here so the
prereg is scored on what it did with them, not on having noticed them:** the
lock protects a **0.65 s** destructive regeneration and is held for **30–1,261
seconds**. Everything below is downstream of that ratio.

## 1. Predictions

Probability form. Interval rows give a point estimate and an interval. Scored in
the rung's final section.

### The contention question (the deliverable)

| # | prediction | p |
|---|---|---|
| **P1** | With the **current** whole-run lock discipline and a shared case directory, launching 4 gate-shaped `mode_cross.sh` runs simultaneously makes **3 of the 4 fall back to a private COLD case set** — i.e. the fallback fires, exactly as `w-gateperf` §11.1 reasoned. | 0.88 |
| **P2** | …and it therefore costs **more** than it saves under the current discipline: 4 simultaneous lanes pay ~3 × 1,261 s where per-worktree directories pay 4 × 1,261 s **once each**, so the naive share is a **wash or a loss** on any lane that gates more than once. | 0.70 |
| **P3** | **The fallback is an artefact of the lock's SCOPE, not of sharing.** With regeneration made non-destructive and the lock reduced to cover only regeneration, the measured fallback count at 4 concurrent lanes is **0**. | 0.80 |
| **P4** | The measured lock-hold window under the fix is **< 5 s** (point 1.5 s, interval 0.3–15 s), vs a measured whole-run hold of 30–1,300 s — a reduction of **≥ 20×** (point ~500×). | 0.85 |
| **P5** | **The overall answer is that sharing WINS** and I ship it. | 0.70 |

### The mechanism

| # | prediction | p |
|---|---|---|
| **P6** | A fresh worktree's `mode_cross.sh` run pointed at an already-warm shared case directory reads **warm**: interval 15–150 s, point 35 s, against the 1,261 s `w-gateperf` measured cold. | 0.85 |
| **P7** | The same holds for `expr_sweep.sh`: a fresh worktree's sweep against a shared case dir reads its warm figure (point 30 s at 16 jobs, interval 15–130 s) rather than the 117 s cold fill. | 0.80 |
| **P8** | The saving is **entirely** the capture cache's source-path key component and **nothing else** — i.e. seeding a worktree's own case directory by `cp --reflink` from another worktree's saves **0 s**, because the path differs and the path is in the key. I predict I can demonstrate this rather than assert it. | 0.90 |
| **P9** | `/home` is btrfs, so `cp --reflink=always` is available — and it is nonetheless **the wrong tool here** (P8). | 0.92 |

### The design, and the coverage constraint

| # | prediction | p |
|---|---|---|
| **P10** | I adopt a **content-addressed, immutable** shared case directory (the dir name carries a digest of `scripts/sweep.d` + `sweep_gen.py`), rather than a mutable shared dir with write-if-differs. | 0.60 |
| **P11** | Content addressing **closes board #3249's hazard that `expr_sweep.sh`'s header names** — two lanes with different `sweep.d` get different directories by construction and cannot overwrite each other — so the shared design is *safer* than the per-worktree one on that axis, not merely faster. | 0.85 |
| **P12** | An injected wrong emit (an `encode_*` off-by-one, `w-gateperf` §10.1's method) still reddens **both** the sweep and the cross **through the shared case directory**, with `cache-bad=0` alongside a non-zero mismatch count — the port-wrong / cache-wrong distinction survives sharing. | 0.95 |
| **P13** | I introduce **no new unbounded cache**. The shared case directory is bounded at **one directory per distinct corpus content** (19,556 files, ~2 MB of source), and I will state that bound. The capture cache's own unboundedness (#3265) is **not** made worse in entry count: the shared paths *reduce* the number of distinct keys minted per lane from 19,556 + 90,812-worth to ~0. | 0.85 |
| **P14** | Everything I ship is labelled **coverage-preserving**; I ship nothing coverage-reducing. | 0.90 |

### The arithmetic, and the outcome

| # | prediction | p |
|---|---|---|
| **P15** | The cold excess is **88–93 %** (point 90 %) of a twice-gating lane's whole-lifetime gate cost. (The dispatch says ~94 %; I think it is slightly lower and I am registering my own number, not theirs.) | 0.60 |
| **P16** | Per-lane saving, if shipped: **≥ 1,100 s** on a lane's first gate (point 1,250 s). | 0.75 |
| **P17** | `Outcome: built`. | 0.65 |
| **P18** | End-state gate `--jobs 16 --require-graded` PASS with sweep **19,556 / 19,460 / 0 mismatch** and cross **90,812 / 90,424 / 0 mismatch**, digit-identical to base. | 0.90 |
| **P19** | Suite **1,666 / 0 / 45**. | 0.80 |
| **P20** | 878-TU scan identity over **394** anchored keys (`grep -cE '^ *gap-metric \S+ \S+$'`), 0 changed, with `fnbyte-*` permitted ±2 per #3249. | 0.85 |
| **P21** | I take the per-process cache context item (`w-gateperf` §11.2 item 2, ~15 % of the warm sweep leg) — **NO**. I predict I leave it, because it needs a stated invalidation rule and this lane's seam is `scripts/`. | 0.70 |
| **P22** | I find **≥ 1** further defect or absence-read-as-success on the way that nobody filed. | 0.55 |
| **P23** | The `hatch-red` row still reads `REFUSED HATCH-STALE` on every run of this lane (#1389/#3219), unfixed here. | 0.90 |

## 2. What would make this lane a FAILURE

Stated in advance so it cannot be renegotiated afterwards:

* shipping a shared case directory whose gate run can pass on a **peer's** port
  verdict rather than its own (the cached artefact must remain oracle-side only);
* shipping any sharing that cannot be shown to still go RED on a real injected
  wrong emit;
* a faster gate whose per-lane counts are not digit-identical to base;
* reporting a saving measured on a run that did nothing (`C2RS_REQUIRE_TOOLCHAIN=1`
  is armed on every suite row here, and the executed counts are asserted, not the
  exit codes — #3219/#3231, and this lane optimises wall clock, which makes it
  the lane most exposed to that hazard).
