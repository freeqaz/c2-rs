# W-DAGCLIENTS — the four DAG clients that bypass the region finder: two of them reorder tuples

    Tag:       W-DAGCLIENTS
    Slug:      dagclients
    Date:      2026-08-13
    Kind:      characterization
    Outcome:   built
    Fixtures:  none — characterization: the four DAG-builder clients that bypass the region finder (#3071)
    Census:    +0 — nothing admitted, no crates/ file touched
    Record:    docs/whitebox/WB_DAGCLIENTS_FINDINGS.md
    Board:     #3099–#3103 (#3099–#3102 Done, #3103 Open)

## What it admits, and what it refuses

**No `crates/` change, no `DISCLOSURE.md` row, no fixture prefix, no function
class.** The lane answers board **`#3071`** — `wb-dagorder`'s own stated blind
spot, filed in the header of its headline row so nobody would infer coverage
from its 15 green cells.

**The answer is YES, and it resolves against `wb-dagorder`'s scope.**
`0x10b3b167` (K1) and `0x10b3b41b` (K2) are a **dependence-DAG block merger** —
tail-merge / cross-jump and head-merge / hoist. Each builds the scheduler's own
DAG (`0x10b328da`) over **two blocks at once** with the region finder
(`0x10be5d4b`, which has exactly one caller) never involved, takes the
dependence-free frontier at the end (`0x10b3ada1`, fanout 0) or the start
(`0x10b3ad62`, pred count 0), matches tuples pairwise (`0x10b36f7e`), and
**unlinks and re-inserts each match** through `0x10bd38b0` / `0x10bd3892` — the
same `tuple+0` next / `tuple+0x10` prev links `WB_DAGORDER_FINDINGS.md` §2
gives the scheduler. They run at `0x10b7ded5`, **before** `0x10b7df57`'s final
schedule, so the scheduler's `node+0x44` original-index tie-break is assigned
from the *post-merge* order: their output is the scheduler's input, not a
cosmetic CFG pass it re-flattens.

Gates, complete: `mode == 2` (only `0x10b7ded5` passes it — three of
`0x10b3c6e5`'s four callers pass 0, and `mode == 1` is dead in this build),
`DAT_10c2e2fc != 0` (`/Og`), `DAT_10c3de20 != 1` (**not** a `/LTCG:PGI`
build — the POGO level, from `0x10b848dc`), plus for K3 `DAT_10c2e310 == 0` =
**favor-size**, the bit `wb-memcpy` already identified black-box (`#1611`).

It refuses three things by name:

* **K3 (`0x10b3b5fd`) is entered 4/4 and reaches its DAG build 0/4.** Filed
  **grey-zone**, no board row: "reached and bailed out on every cell tried",
  never "K3 does not reorder". Grid 2 was written to reach it and did not.
* **`dt_dep`, the registered dependence-gating discriminator, proves nothing** —
  its independent control `dt_sink` read the same two copies, so the cell has
  no discriminating power. Scored grey-zone, not a hit.
* **Why `dt_sfx` merges and `dt_sink` does not** — one arm's two independent
  stores in the other order — is the largest hole in the mechanism story and is
  stated as such. The port cannot predict *which* pairs merge from this write-up.

## Estimate vs outcome

Predicted reach **0** TU (characterization lane, `wb-live`'s pattern);
realized **0**. The prediction that matters is the registered one, and it went
the other way: **M9** — *"the detecting cell exists and stays green, i.e. none
of the four reorders"* — was registered at `p = 0.55` before the first grep of
the export, and **M1** (*"at least one re-threads the tuple list"*) at
`p = 0.20`. The cell exists and **went red**. Six of R1's ten claims resolved
against their registered direction.

The instrument that answered the question was **not the one preregistered**,
and this is the lane's method finding. R2's grid tried to detect motion by
*shape* (`dt_sink`: a common statement that is not a common suffix), which
would have been positive only if c2 merged that shape — it does not, so P3,
P6 and P11 all missed together on one wrong assumption. What worked was
**ablation of patched copies of the pinned image**: `A123` (K1+K2+K3 → `return
0`) turns `dt_sfx@/O1` from 13 instructions with one copy of the common store
**above the branch** into 16 instructions with two copies, one inside each arm.
Cross-checked by a `ud2` reachability ladder (entered / past-gates / fired)
that agrees with the obj deltas in **all 8 grid × favor cells**, and validated
by `A4` — a patched byte in a function that does not run changes nothing.

> **Before pricing this as codegen, run `CEILING.md` §11.4.** Not applicable —
> no conversion was attempted. What this lane changes for §6.1 item F is the
> *price*: `WB_DAGORDER_FINDINGS.md` §7's five-step recipe is incomplete, and
> the missing step (`WB_DAGCLIENTS_FINDINGS.md` §5, step 3b) is a block merger,
> not a scheduling refinement. On the dc3 workload's `/O1 /Oi` — favor-size —
> all three mergers' gates are open.

## Gate evidence

**The gate of record is the REBASED one.** This lane first gated at its
original base `d9dbefc2` (`graded tree 996f0bf2b4bc`, 725 files, `cargo test`
1,532 / 42) and then rebased onto `494993f1` — five peer lanes, **three of
which changed `crates/`** (`w-ir-cond` `codegen/block_ir.rs`, `w-ir-e`
`codegen/cond.rs`, `w-dbgassert` `coff/writer.rs` / `coff/ehscope.rs` /
`gap/fnbytes.rs` / the new `scripts/debug_lane.sh`). **The merged configuration
is one no earlier run covered**, so everything below was re-run on it; the
pre-rebase numbers are superseded, not deleted.

| lane | result (rebased tree, `494993f1` + this lane) |
|---|---|
| `cargo test --workspace --release --no-fail-fast` | **1,567 passed, 0 failed, across 42 targets** — pass count matches current master's, and the **target count is unchanged at 42**, so no earlier target failed and left a partial |
| `scripts/gate.sh --jobs 4 --require-graded` | **GATE: PASS (HATCH-RED REFUSED)** — 18 lanes in the registry, **18 PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT**, **6,858 fixture-verdicts**; `graded tree b865e54d6939 (728 files under crates fixtures scripts)` **identical at both ends**, 0 gitignored byproducts unhashed |
| `scripts/expr_sweep.sh` | gate row `expr-sweep PASS 19556/19556`, 19,460 graded, **0 mismatch** |
| `scripts/mode_cross.sh` | gate row `mode-cross PASS 90812/90812`, 90,424 graded, **0 mismatch** |
| `scripts/debug_lane.sh` (board #3087) | **`DEBUG-LANE-TOTAL lanes=18 ran=18 failed=0`** — all 18 lanes `graded=381 total=381`, **0 mismatch, 0 panics** in the debug profile |
| 878-TU workload scan | `match 25 · mismatch 0 · codegen-gap 0 · vocab-gap 845 · capture-fail 8` — **digit-identical**, re-measured on the rebased tree rather than argued from construction |
| `scripts/board_audit.sh` | all-zero (0 cited-but-absent, 0 unresolved anchors, 0 raw line anchors, 0 rows behind the prose, 0 duplicate numbers) |
| `rung_registry` | **2/2** |
| fixtures, `c2rs census` | untouched — `git diff 494993f1..HEAD -- crates fixtures scripts` is **EMPTY**; the nine changed files are all under `docs/` |

> `docs/rungs/INDEX.md` was regenerated with `scripts/gen_rung_index.sh` after
> the rebase, never resolved by hand.

## Found and not taken

Ranked by what the next lane gets per unit of work.

1. **The fourth merger.** With K1, K2 and K3 all ablated, `dk_join3` still
   collapses 3 copies of a common store to 2, and `dk_loop_join` still merges.
   **Something else merges blocks and it is not a DAG-builder client.** One
   `ud2` ladder over `0x10b3c2cc`'s other call sites (`0x10b36805`,
   `0x10b38cd4`, `0x10b388eb`, `0x10b3a253`) would name it in an afternoon.
   Largest remaining hole in the ordering model.
2. **Reach K3.** Its own body is K1's shape with a searched second block
   (`0x10b35f88`, else the label's predecessor list `label[10]`). Six shapes
   failed. Reading `0x10b35f88` first — 20 minutes — would say what to write
   instead of guessing again; grid 2 guessed and cost a full compile round.
3. **Which pairs merge.** `dt_sfx` merges, `dt_sink` does not, and they differ
   only in the order of two independent stores in one arm. Either `0x10b36f7e`
   is position-sensitive or the frontier walk order is. This is the one thing a
   port would need that this lane does not supply.
4. **`/QXSTALLS /FAsc` at `/Od` is a free DAG dump.** K4 runs with the
   optimizer **off**, and its reporter `0x10c1c3f7` prints per-block IPC and
   named dependency stalls. That is c2 narrating its own dependence graph on
   unoptimized code — an instrument for the latency matrix that costs one flag
   pair, and `wb-dagorder` §8 item 1's unresolved transposition is exactly the
   kind of question it could settle.
5. **Ablation as a standing instrument.** `work/w-dagclients/patch.py` +
   `trap.py` + `trap2.py` are ~60 lines and turn "does pass X matter here" into
   one compile. Every `#3067`-style absence in the whitebox record is now
   cheaply testable. **Not promoted to `scripts/`** — it patches a copy of a
   Microsoft binary, and that belongs in a lane's scratch, not in the graded
   tree.
6. **`mode == 1` is dead**, so `0x10b3c065` is unreachable. Established from
   four call sites and no indirect xref — which is an absence, hence grey-zone.
   Worth 10 minutes with a `ud2` at `0x10b3c065` to make it positive.
