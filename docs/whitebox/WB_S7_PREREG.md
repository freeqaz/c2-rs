# PREREG — lane `w-s7`: stage S7 `0x10b7e032`, read whole

**Frozen before the image was opened.** Characterization lane under decision 21
§2 (`docs/DECISIONS_2026-08-22.md`), board `#3737`–`#3742`.

Tree at freeze: `8213c7b77`. Pinned image
`compilers/X360/16.00.11886.00/c2.dll`, expected sha256
`c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`
(`f0_pipeline.py --verify` — **not yet run at freeze time**).

---

## 0. The question

`w-f0price` (`WB_F0PRICE_FINDINGS.md` §4.1, §6.2) found that **stage S7 —
`FUN_10b7e032` @ `0x10b7e032`, 225 B, 10 depth-1 passes, 2,489 B — has no F0
sub-item at all**, and that **2 of the 4 confirmed tuple-splicers sit in stages
F0 prices at 1 lane and 0 lanes**. Decision 21 funds the read.

Three deliverables, in the brief's own words:

1. **what S7 does** — an addressed account of the stage;
2. **what it can reorder**;
3. **whether the splicer `FUN_10b35c78` @ `0x10b35c78` is reachable in the
   configurations this project compiles** (`/O1 /EHsc /GR`, no POGO).

**This is not a re-price of F0.** `w-f0price`'s `≥ 10 raw sub-lanes + 2 UNPRICED
terms` stands; this lane reports *bearing* on it, not a replacement number.

## 0.1 What is ALREADY PUBLISHED and is therefore not this lane's finding

Registering this so a "hit" cannot be scored on something the tree already said.
At freeze this lane has read, from committed documents only:

* `WB_F0PRICE_FINDINGS.md` §4.1 — a **decompiled sketch of `FUN_10b7e032`'s
  body**, including the `(*(uint *)(*param_1 + 0x20) & 0x1000)` gate annotated
  *"the EH gate — /EHsc"*, the `DAT_10c3de20 == 0` mode-0 merger call, and the
  ten callee names. **The sketch is elided (`...` in three call rows) and its
  gate annotation is an interpretation, not a read of the bit's writers.**
* `ref/P_BLOCKORDER.md` §3 — the same ten callees, in call order, and
  `FUN_10b3421b` → `FUN_10b338f5` as the emit walk.
* `ref/P_BLOCKORDER.md` §6 open #2 and `WB_BLOCKORDER_FINDINGS.md` §6 — that
  `0x10b35c78` is **unread** and is the standing candidate for whether a
  decision tree's arm order is a block **move** or a leaf
  **materialization**.
* `WB_DAGCLIENTS_FINDINGS.md` §2 — `0x10b3c6e5`'s four callers and their modes.
* `ref/FUNCS.tsv` metadata rows for the ten (sizes, TU band, `cover`,
  `ncallers`/`ncallees`).
* `DISCLOSURE.md` `W-STAGETAP-3` — `0x10b7e701` is the call site of S7 in the
  orchestrator, already adopted as the `after0` tap.

**No body of any of the ten callees has been read.** Every prediction below is
about content this lane has not seen.

---

## 1. Registered predictions

Each carries a prior and an explicit refutation condition. **A prediction whose
refutation condition cannot fire is decoration (`#3336`) and is marked so.**

| # | prediction | p | refuted by |
|---:|---|---:|---|
| **P1** | The `0x1000` bit tested at `*(*param_1 + 0x20)` is **not** a mirror of the `/EHsc` command line. It is a **per-function or per-compiland record flag written by c2 itself**, so the gate can be false on a TU compiled *with* `/EHsc`. | 0.65 | finding a writer of that bit that is a direct copy of the `/EHsc` option word, with no per-function condition |
| **P2** | `FUN_10b35c78`'s direct splice call is an **unlink** (`0x10bd3852`), not an insert (`0x10bd38b0` / `0x10bd3892`) — i.e. it **removes** tuples and cannot author a new position. | 0.50 | its splice callee resolving to an inserter or to the bulk relink `0x10be626c` |
| **P3** | **`0x10b35c78` IS reachable on this project's configurations** — the gate above is satisfied for at least one function class the dc3 workload contains, so the splicer is live and not dead code. | 0.70 | the gate resolving to something no `/O1 /EHsc /GR` compiland sets (e.g. a `/CLR`, `/GH`, or POGO-instrument-only bit) |
| **P4** | **At least 3 of S7's 10 depth-1 passes are dead in this project's configurations** (POGO / instrument / mode gated), so "10 passes, 2,489 B" over-counts the live stage. One is free (`0x10b9d6be`, `DAT_10c3de20 == 2`), so this needs **2 more**. | 0.55 | fewer than 3 gated-dead |
| **P5** | S7 carries **≥ 2 distinct live authors of tuple order** at `/O1 /EHsc /GR`, counting the mode-0 `0x10b3c6e5` run and at least one more of `{0x10c21b03, 0x10b35c78, 0x10b36169}`. | 0.70 | exactly one, or zero |
| **P6** | **`w-f0price`'s A/B/C splice partition is wrong on at least one S7 row** — a pass it puts in group C (cannot reorder) either splices inline on `tuple+0`/`tuple+0x10`, or reaches a splice through an edge `calls.tsv` does not carry (indirect / tail call / jump table). | 0.40 | all ten S7 rows surviving the check |
| **P7** | **Order is frozen at `FUN_10b3421b`'s entry** — nothing S7 calls at or after `0x10b3421b` moves a tuple; the emit walk consumes the order and does not author it. | 0.75 | any splice reachable from `0x10b3421b` that runs before the walk emits |
| **P8** | **`cover=none` is again a statement about the index, not the image.** ≥ 3 of S7's 10 passes have a prior mention somewhere in this repo (`labels/*.tsv`, `rungs/`, `docs/`) despite their `FUNCS.tsv` `cover` column. | 0.60 | fewer than 3 |

### 1.1 The one prediction this lane most wants to be wrong

**P3.** A live splicer inside an unpriced stage makes F0's floor worse and makes
`P_BLOCKORDER` §6 open #1 answerable. A *dead* `0x10b35c78` would be the more
useful result for the project — it would close open #2 by elimination and shrink
the order-changing bracket from below. Registered so that the write-up cannot
present either outcome as the one it wanted.

### 1.2 Decoration check

**P5 is the weakest predicate here**: the mode-0 merger run is already published
as live, so P5 needs exactly one more author to fire, and `0x10c21b03` is
already in `w-f0price`'s group A. It is registered anyway because its *refutation*
is informative (if the merger's mode-0 arms turn out inert, "S7 reorders" is
false), but it is flagged as **low-information** in advance and will not be
counted as a headline result.

---

## 2. Method

**Read-before-probe** (`WHITEBOX_LEVERAGE_2026-08-21.md`). Order:

1. `python3 docs/whitebox/scripts/f0_pipeline.py --verify` — decline criterion 1.
2. `--stages` and `--splice`, unmodified, to reproduce `w-f0price`'s S7 rows.
   **The instrument is reused, not rebuilt** (decision 21 method rule). If a new
   view is needed it is added as a subcommand to `f0_pipeline.py`.
3. Read `FUN_10b7e032` whole from the flat export (`decomp_all.c` +
   `objdump_intel.asm`), then each of the ten callees, then `0x10b35c78`'s
   splice callee and the `0x1000` bit's writers via `xrefs.tsv`.
4. Corroborate the liveness claim against an obj-visible consequence **only if
   one exists**; if it does not, say so rather than manufacture one.

## 3. Decline criteria — registered in advance

The lane **declines and says so** if any of these holds:

1. **`--verify` reports MISMATCH.** No address is quotable; report and stop.
2. **The flat export is stale relative to the pinned image** — a callee address
   in `decomp_all.c` that `FUNCS.tsv` does not carry, or a size disagreement on
   `0x10b7e032` (expected 225 B).
3. **> 12 function bodies read without resolving the `0x1000` gate's writer.**
   In that case the reachability question (deliverable 3) is reported
   **UNRESOLVED with its cost**, and deliverables 1 and 2 still land.

## 4. Scope fences

* **No scheduler, merger, or allocator is built.** Decision 20 §2, decision 21 §4.
* **No count-bearing row is added to `scripts/gate.sh`** (`#3691`).
* **Nothing is adopted into `crates/`** unless a `DISCLOSURE.md` row plus a
  `PROV[…]` marker ship in the same commit. **Adopting nothing is the expected
  outcome** and is not a failure.
* **F0 is not re-priced.** Bearing only.
* Board rows stay inside `#3737`–`#3742`.

## 5. Predicted reach

**0.** Characterization lane; `git diff master..HEAD -- crates fixtures` is
expected empty at the tip.
