# WB_DAGCLIENTS — tuple order has a SECOND AUTHOR: a dependence-DAG **block merger**

> **PROVENANCE — DISASSEMBLY-DERIVED.** Every address below is an absolute VA in
> the exact image pinned in [`C2_MAP_METHOD.md`](C2_MAP_METHOD.md) §0 —
> `sha256 c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`,
> verified on `compilers/X360/16.00.11886.00/c2.dll` at the top of this lane.
> Navigation only. **This lane adopts nothing into `crates/` and adds no
> `DISCLOSURE.md` row.**

PREREG: [`WB_DAGCLIENTS_PREREG.md`](WB_DAGCLIENTS_PREREG.md) committed at
`cff4a8db` **before the first grep of the export**;
[`WB_DAGCLIENTS_PREREG_R2.md`](WB_DAGCLIENTS_PREREG_R2.md) with grid 1
(`grids/wb-dagclients/dagclients_grid.cpp`, sha256 `4847a3a6…`) at `bdc5686d`
**before the first `cl.exe`**; grid 2 (`dagclients_grid2.cpp`, sha256
`70dd62fb…`, predictions in its own header) at `fbd7ea8e` **before its first
`cl.exe`**. Scored in §7.

**The commissioned question** — board **`#3071`**, `wb-dagorder`'s own stated
blind spot: *do any of the four DAG-builder clients that bypass the region
finder reorder tuples, and under what conditions?*

Landed as board **`#3099`–`#3103`**: `#3099` the answer, `#3100` the method
(the preregistered discriminator missed and ablation answered), `#3101` the
grey-zone K3 result stated positively, `#3102` `/QXSTALLS`, and `#3103` — in
**Open** — the fourth, unattributed merger.

---

## 1. THE HEADLINE

**YES. Two of the four do, and `wb-dagorder`'s ordering model is incomplete
without them.**

1. **`0x10b3b167` (K1) and `0x10b3b41b` (K2) are a `dag.c` block merger** —
   tail-merge / cross-jump and head-merge / hoist. Each builds the *same*
   dependence DAG the scheduler uses, over **two blocks at once**, harvests the
   nodes that are dependence-free at the end (`0x10b3ada1`, fanout 0) or at the
   start (`0x10b3ad62`, pred count 0), matches them pairwise across the two
   blocks (`0x10b36f7e`), **physically unlinks and re-inserts each match**
   (`0x10bd38b0` insert-before / `0x10bd3892` insert-after — both rewrite
   `tuple+0` next and `tuple+0x10` prev, the same two links
   `WB_DAGORDER_FINDINGS.md` §2 gives the scheduler), and then merges the two
   blocks. **They therefore re-thread the tuple list, and they do it across
   branch and call boundaries the scheduler provably cannot cross.**
2. **They run BEFORE the last scheduler pass**, so the scheduler's
   original-tuple-index tie-break (`node+0x44`, `0x10b327cd`) is assigned from
   the **post-merge** order. Their output is the scheduler's input. This is not
   a cosmetic CFG transform that the scheduler then re-flattens.
3. **`0x10b3b5fd` (K3) is entered on every optimized cell of both grids and
   never once reaches its DAG build** — verified positively, not by absence
   (§4.3). Its extra gate `DAT_10c2e310 == 0` is the **favor-speed** bit
   already identified black-box by `wb-memcpy` (board `#1611`).
4. **`0x10c1ce93` (K4) is read-only and its reachability is exactly
   `/QXSTALLS` AND `/FAsc`** — at *any* optimization level **including `/Od`**,
   which is the one place a DAG gets built with the optimizer off. With the
   flag pair the obj is byte-identical to the same build without `/QXSTALLS`
   at all five optimization levels.

The lane's registered deliverable was M9: *"the detecting cell exists and stays
green — none of the four reorders"*, `p = 0.55`. **The cell exists and it went
RED.** `dt_sfx` at `/O1` emits **13** instructions with one copy of the common
store hoisted above the branch; with K1+K2+K3 ablated it emits **16** with two
copies, one inside each arm (§4.1). The answer is the opposite of the one this
lane bet on, and it is the answer `#3071` said no grid could reach.

## 2. Where it lives

| what | address | notes |
|---|---|---|
| **the pass driver** | **`0x10b3c6e5`** | `FUN_10b3c6e5(func, mode)`; loops `0x10b3c2cc` up to **`0x11` = 17** times until it reports no change |
| the per-tuple walker | **`0x10b3c2cc`** | walks the whole function's tuple list; dispatches on the tuple category byte (`tuple+8`) |
| **the only live route to K1–K3** | **`0x10b7ded5`** | the **`mode = 2`** call. `0x10b3c6e5` has exactly four callers — `0x10b7ddff`, `0x10b7ded5`, `0x10b7e032`, `0x10be0af1` — and **three of them pass `mode = 0`**. `mode = 1`, and with it the `0x10b3c065` route to K1, is **dead in this build** |
| phase order | `0x10b7e6af` | `0x10b7dbf6` · `0x10b7dc51` (3 mode-1 schedules) · `0x10b7dd2c` · `0x10b7ddff` · `0x10b7de4a` · **`0x10b7ded5` (this merger)** · `0x10b7df57` (the **last** schedule, `0x10be6382` mode 0) · `0x10b7e032` |
| **K1 — tail merge / cross-jump** | **`0x10b3b167`** | two blocks; DAG over each block's **tail**; sinks matches to just before the branch; commits with `0x10b36e93` / `0x10bd5952` / `0x10bd5648` |
| **K2 — head merge / hoist** | **`0x10b3b41b`** | two blocks; DAG over each block's **head**; hoists matches to just after the branch; commits with **`0x10b39075`** |
| **K3 — tail merge to a SEARCHED second block** | **`0x10b3b5fd`** | second block found via `0x10b35f88`, else by walking the label's predecessor list `label[10]` for another `0x12` targeting it. Gated `DAT_10c3de20 != 1` **and `DAT_10c2e310 == 0`**. **⚠ 2026-08-14 (`w-merger4`): "searched" overstates it — `0x10b35f88` is a COMPLEMENTARY-BRANCH TEST.** It walks `tuple+0x10` back past labels and requires a `0x12` that is *not* plain-conditional, whose cc `(tuple+10) & 0x1f` equals **`DAT_10b189cc[(other+10) & 0x1f]`** (a condition-**inversion** table), whose targets resolve to the same object, and whose target's first non-label tuple is the caller's own. So K3's second block is *the fallthrough predecessor reached by the inverted branch of the same test* — see [`WB_MERGER4_FINDINGS.md`](WB_MERGER4_FINDINGS.md) §2 |
| **K4 — `/QXSTALLS` stall report** | **`0x10c1ce93`** | allocates its own graph (`0x10b31fd4`), resets it (`0x10b32536`), builds one DAG over the **whole function**, tail-calls the reporter `0x10c1c3f7` ("`PX Dispatch Groups`", `0x10b24160`) |
| K1's backward scan limit | **`0x10b397ba`** | walks `tuple+0x10` (prev) at most **`0x1d` = 29** tuples, stopping at category `0x17`(non-`0x317`) / `0x18` / `0x1a` / `0x10` / a branch. **Consults `DAT_10c2e310`** — the window differs between favor-size and favor-speed |
| K2's forward scan limit | `0x10b39837` | the mirror image |
| sinkable set (fanout 0) | **`0x10b3ada1`** | walks `graph+8`, harvests nodes with `node+0x26 == 0` |
| hoistable set (pred count 0) | **`0x10b3ad62`** | walks `graph+0xc`, harvests nodes with `node+0x24 == 0` |
| tuple equivalence | **`0x10b36f7e`** | same `tuple+4` opcode, same operand lists (`0x10bd4e7e`), same `tuple+0xa & 0x1f` cc for category `0x12`; category `0x1b` (label) never matches |
| **the splices** | **`0x10bd38b0`** / **`0x10bd3892`** | unlink (`0x10bd3852`) then insert **before** / **after** an anchor. `tuple+0` = next, `tuple+0x10` = prev |
| graph alloc / reset | `0x10b39794` → `0x10b57850(graph, 7, 0x50, 0x1c)` | a **`0x50`**-node pool of `0x1c`-byte nodes; then `0x10b32536` |
| the region finder, for contrast | `0x10be5d4b` | **exactly one caller, `0x10be6382`** — so `#3071`'s "they bypass the region finder" is confirmed by construction, not inherited (M6) |

Gates, complete:

    K1  DAT_10c3de20 != 1                       and mode == 2 and DAT_10c2e2fc != 0
    K2  DAT_10c3de20 != 1                       and mode == 2 and DAT_10c2e2fc != 0
    K3  DAT_10c3de20 != 1  and DAT_10c2e310 == 0  and mode == 2 and DAT_10c2e2fc != 0
    K4  /QXSTALLS and /FAsc  (no optimizer gate at all)

`DAT_10c2e2fc` is the optimizer bit (`#3067`). `DAT_10c3de20` is the **POGO
level**, written from `DAT_10c6f1c8` (`0x10b848dc`: `1` = `/LTCG:PGI`
instrument, `2` = `/LTCG:PGO` optimize, `0` = an ordinary build): **`== 1`
disables all three mergers**, which is what keeps a PGI build's arc counters
intact. `DAT_10c2e310` is bit 23 of the option word = **favor-speed**
(`#1611` / `DISCLOSURE.md` W-MEMCPY-2): `1` at `/O2`, `/Ox`, `/O1 /Ot`; `0` at
`/O1`, `/O2 /Os`. **The dc3 workload's `/O1 /Oi` is favor-size, so K3's gate is
open on the workload** — it simply never fired on either grid here.

Note the split inside `0x10b3c2cc`: **K1 is called on the branch class where
`tuple+0x34 == 0` and the opcode is not `0x2e4` / `0x21` / `0x22`; K2 and K3
are called on the complement**, and K3 only after K2 has returned 0.

## 3. THE INSTRUMENT — ablation, and why this lane could answer what `#3071` could not

`wb-dagorder` could not detect a second author because every cell of its grid
observed only the **final** tuple order, and a second author that agrees with
the scheduler is invisible in that view. This lane used a different instrument:
**ablation of the pinned image**, plus a `ud2` reachability ladder.

Six ablation images and six trap images were built by patching **copies** of
`c2.dll` (the pinned original is untouched and was re-verified at the end):

| image | patch | sha256 of the derived image |
|---|---|---|
| `A0` | none — the control | `c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258` (= the pinned image) |
| `A1` | `0x10b3b167` ← `xor eax,eax; ret 8` | `d7cfb8e331bf98bd…` |
| `A2` | `0x10b3b41b` ← `xor eax,eax; ret 8` | `97afc1f0ce7a0411…` |
| `A3` | `0x10b3b5fd` ← `xor eax,eax; ret 0xc` | `70426e4cff4b6a2d…` |
| `A4` | `0x10c1ce93` ← `ret` | `98a1c92841cf2dd3…` |
| `A123` | all three of K1/K2/K3 | `116c2d0f238d8359…` |
| `XK1/2/3/4` | `ud2` at each client's **entry** | *(reachability)* |
| `RK1/2/3` | `ud2` at each client's **first `call 0x10b328da`** | *(past every gate)* |
| `FK1/2/3` | `ud2` at each client's **commit / return-1** | *(it actually fired)* |

Every patch site's original bytes were asserted against `objdump_intel.asm`
before writing (`work/w-dagclients/patch.py`, `trap.py`, `trap2.py`), and the
file offsets are `VA − 0x10b00c00` (`.text` VMA `0x10b01000` at file `0x400`).

**These images are ablation controls, never oracles.** Every behavioural claim
below is read off the **unpatched** `A0` build; a patched build only ever
answers "was this function responsible".

Three validity checks the instrument passed, each of which could have failed:

* **A0 is byte-identical to the pinned toolchain's output and deterministic
  across repeats** — and the first attempt at this table was wrong for a reason
  worth recording: `cl` embeds the `/Fo` and `/Fa` **path strings** in the obj,
  so per-variant output filenames produced a one-byte diff at file offset
  `0x44a` that read as "every ablation changes the obj". Fixed by compiling
  every variant to one fixed path and copying afterwards.
* **`A4` (K4 ablated) is byte-identical to `A0` at all five optimization
  levels** — a patched byte in a function that does not run changes nothing, so
  a positive `A1`/`A2` delta is not an artifact of patching.
* **The `ud2` ladder agrees with the obj deltas in all 8 grid × favor cells**:
  wherever `FK<n>` trapped, `A<n>` differed from `A0`; wherever it did not,
  `A<n>` was identical. Two independent instruments, no disagreement.

## 4. THE ANSWER

### 4.1 The cell that went red

`dt_sfx` — an `if`/`else` whose arms are `{dc_a=1; dc_c=9;}` and
`{dc_b=2; dc_c=9;}` — at `/O1`, real image vs `A123`:

```
A0 (pinned image)                        A123 (K1+K2+K3 ablated)
  lis   r8,dc_c                            cmpwi cr6,r3,0
  li    r10,9                              beq   cr6,$LN2
  cmpwi cr6,r3,0                           lis   r9,dc_a
  stw   r10,dc_c(r8)   <-- ABOVE the       lis   r8,dc_c
  beq   cr6,$LN2           branch          li    r11,1
  lis   r9,dc_a                            li    r10,9
  li    r11,1                              stw   r11,dc_a(r9)
  stw   r11,dc_a(r9)                       stw   r10,dc_c(r8)   <-- copy 1
  blr                                      blr
$LN2:                                    $LN2:
  lis   r9,dc_b                            lis   r9,dc_b
  li    r11,2                              lis   r8,dc_c
  stw   r11,dc_b(r9)                       li    r11,2
$LN1:                                      li    r10,9
  blr                                      stw   r11,dc_b(r9)
                                           stw   r10,dc_c(r8)   <-- copy 2
  13 instructions, ONE copy              $LN1:
                                           blr
                                           16 instructions, TWO copies
```

`dc_c = 9` is written **last** in both arms of the source. On the real image
its three tuples are emitted **before the `beq`** — they left both arms, passed
`dc_a = 1` in one of them, and crossed the branch. `WB_DAGORDER_FINDINGS.md`
§3/§4 and board `#3069` establish that **no tuple crosses a branch or a call**
under the scheduler (15/15 cells): the region ends at the branch. So this
motion is not the scheduler's, and the ablation names who: with K1, K2 and K3
returning 0 it does not happen.

Note also the `cmpwi` sitting **between** the moved `li` and the moved `stw` in
the `A0` column. The merged tuples became part of the pre-branch region and
were then re-scheduled with it — the merger's output feeding the scheduler's
input, visible in one cell.

`dk_call_join` (grid 2) is the same result across a **call**: one copy of the
common store on the real image at both favor levels, two (`/O1`) and three
(`/O1 /Ot`) with `A123`.

### 4.2 Which client, when — the fire table

`X` = entered (`ud2` at entry) · `R` = reached its `0x10b328da` call ·
`F` = fired (commit / return-1) · `Δ` = its `A<n>` obj differs from `A0`.

| | grid 1 `/O1` | grid 1 `/O1 /Ot` | grid 2 `/O1` | grid 2 `/O1 /Ot` | `/Od` (both grids) |
|---|---|---|---|---|---|
| **K1** `0x10b3b167` | X R **·** Δ— | X R **F** Δ✓ | X R **F** Δ✓ | X R **F** Δ✓ | **not entered** |
| **K2** `0x10b3b41b` | X R **F** Δ✓ | X R **F** Δ✓ | X R **F** Δ✓ | X R **·** Δ— | **not entered** |
| **K3** `0x10b3b5fd` | X **·** · Δ— | X **·** · Δ— | X **·** · Δ— | X **·** · Δ— | **not entered** |
| **K4** `0x10c1ce93` | not entered (see §4.4) | | | | not entered |

Read it as: K1 and K2 are **redundant covers of one another** on grid 1 at
`/O1` — ablate K2 and K1 performs the merge instead (with slightly different
code), ablate K1 and K2 does it identically, ablate both and the merge is gone.
That redundancy is exactly why a single-client ablation is not enough and
`A123` is the load-bearing cell.

Whole-obj sizes and hashes, `TimeDateStamp` zeroed, grid 1:

| image | `/Od` | `/O1` | `/O2 /Os` | `/O1 /Ot` | `/O2` |
|---|---|---|---|---|---|
| `A0` | 8836 `3c9961cb` | 9984 `521292c3` | 9984 `521292c3` | 10230 `8b83a368` | 10230 `8b83a368` |
| `A1` | = | = | = | 10230 `4f40c030` | 10230 `4f40c030` |
| `A2` | = | 9988 `a2fb75f0` | 9988 `a2fb75f0` | 10338 `e3ba1685` | 10338 `e3ba1685` |
| `A3` | = | = | = | = | = |
| `A4` | = | = | = | = | = |
| `A123` | = | 10092 `b8e10de4` | 10092 `b8e10de4` | 10338 `6a5bb5fd` | 10338 `6a5bb5fd` |

**The obj is a function of favor-speed, not of `/O<n>`**, on both grids and in
every one of the six images: `/O1 ≡ /O2 /Os` and `/O1 /Ot ≡ /O2`, 10 variant ×
grid pairs, 0 exceptions. That is `#1611`'s deciding quad reproduced on an
unrelated construct, and it is why K1's contribution shows up only at
favor-speed on grid 1: **`FUN_10b397ba` reads `DAT_10c2e310`** when deciding
whether to stop its backward scan at an unconditional branch, so K1's window —
not K1's gate — is what moves.

### 4.3 K3 is entered and never fires — stated positively

`A3` is byte-identical to `A0` in every cell of both grids. **That is an
absence and this lane does not bank absences**, so it was replaced with a
positive measurement:

* `XK3` (`ud2` at `0x10b3b5fd`) **traps** on grid 1 and grid 2, at `/O1` and at
  `/O1 /Ot` — K3 **is** called, 4/4.
* `RK3` (`ud2` at K3's first `call 0x10b328da`, `0x10b3b6f8`) **compiles
  cleanly** in all 4 — K3 **never reaches its DAG build**.
* `FK3` (`ud2` at K3's `return 1`, `0x10b3b940`) compiles cleanly in all 4.

So the finding is *"K3 is reached and bails out before building a DAG on every
cell tried"*, not *"K3 does not reorder"*. Its own code is the same
sink-and-merge shape as K1's, so **the null result here is about the grid, not
about K3**, and the honest statement of what remains open is in §6. Grid 2 was
written specifically to reach it — three- and four-predecessor joins, a
`switch`, a call-terminated join, a loop latch — and none did.

### 4.4 K4 is read-only, and reachable only under `/QXSTALLS` **and** `/FAsc`

| `/QXSTALLS` | `/FAsc` | `/Od` | `/O1` | `/O1 /Ot` |
|---|---|---|---|---|
| off | off | never entered | never entered | never entered |
| off | on | never entered | never entered | never entered |
| **on** | off | never entered | never entered | never entered |
| **on** | **on** | **ENTERED** | **ENTERED** | **ENTERED** |

The `on`/`off` cell is the one that catches the trap: `/QXSTALLS` alone does
**nothing** — the client hangs off the listing writer `0x10b71d8f`, so without
`/FAsc` there is no listing and no call. A lane that had probed only
`/QXSTALLS` on/off without `/FAsc` would have concluded "K4 never runs" from a
misconfigured probe.

With the flag pair, on the real image, the obj is **byte-identical** to the
same build without `/QXSTALLS`, `TimeDateStamp` zeroed, at **all five**
optimization levels, over a 14-function body containing the four region-ender
shapes (no interior barrier, interior branch, interior call, interior label)
and a `> 0x50`-tuple straight-line body. Two independent run-proofs, both
positive on content:

1. every ON listing carries **14** `Stall summary for function` blocks (one per
   `PROC`), 29–36 `Estimated block IPC` lines and 19–28 dependency-stall notes;
   every OFF listing carries **zero** of each;
2. `A4` — K4 patched to `ret` — compiles **byte-identically to `A0`** without
   the flags and **SIGSEGVs** with `/QXSTALLS /FAsc`, because `0x10b71d8f`
   immediately dereferences the graph `DAT_10c3d184` that K4 was supposed to
   build.

**K4 also runs at `/Od`.** That is the one configuration in which this `c2`
builds a dependence DAG with the optimizer off, and it refutes this lane's own
M4 (*"at `/Od` none of K1–K4 runs"*, `p = 0.65`) — which was inherited from
`#3067`'s true statement about the **scheduler** and wrongly generalized to the
clients.

## 5. What this costs the port

`WB_DAGORDER_FINDINGS.md` §7's five-step recipe is **not sufficient** to
reproduce c2's tuple order on any function with a two-way branch whose arms
share work. The missing step sits *between* its steps 4 and 5, and it is not a
scheduling step at all:

    …
    3b. BLOCK MERGE (0x10b7ded5, up to 17 rounds of 0x10b3c2cc):
        for each branch, build the same DAG over BOTH successor/predecessor
        blocks (0x10b328da, region finder NOT used); take the sinkable
        (fanout-0) or hoistable (pred-0) frontier; match tuples pairwise
        (0x10b36f7e); UNLINK AND RE-INSERT each match at the merge point;
        merge the blocks.  Only in favor-SIZE-independent K1/K2; K3 adds a
        favor-size gate.  Disabled entirely under /LTCG:PGI.
    …then the LAST schedule runs over the re-threaded list.

The practical consequence for `CEILING.md` §6.1 item F: a port that implements
the scheduler alone will disagree with c2 on **every** function where c2 merged
two blocks, and the disagreement is not local — the merge changes the tuple
indices the scheduler tie-breaks on, so the whole surrounding region moves. On
the dc3 workload's `/O1 /Oi` (favor-size) all of K1, K2 and K3's gates are
open.

## 6. Grey-zone / not established

Filed here rather than banked, per `wb-live`'s cost-array precedent:

1. **Whether K3 ever fires on any input.** Reached 4/4, past its gates 0/4.
   Both grids failed to produce the shape `0x10b35f88` looks for. **Grey-zone
   — no board `DISCLOSURE` row, no claim either way.**
2. **Why `dt_sink` does not merge but `dt_sfx` does.** The two differ only in
   the order of two independent stores inside one arm, and both stores are
   dependence-free in their block's DAG, so the sink/hoist frontier should
   contain both. The matcher `0x10b36f7e` or the frontier walk order is
   evidently position-sensitive in a way this lane did not isolate. **This is
   the single largest hole in the mechanism story and it is stated, not
   smoothed** — the port cannot predict *which* pairs merge from what is
   written here.
3. **`dt_dep` proves nothing.** It was registered (P4) as the
   dependence-gating discriminator, and it did come out at two copies — but so
   did its *independent* sibling `dt_sink`, so the cell has **no discriminating
   power** and is scored grey-zone, not a hit. A cell whose control and
   treatment agree is not evidence.
4. **`dk_join3` / `dk_loop_join` merge partially even under `A123`** (3 → 2
   copies with all three mergers ablated), so **a fourth merger exists** that
   is not a DAG-builder client. Not chased.

   > ### ⚠ 2026-08-14 — CHASED, and it splits in two (`w-merger4`)
   >
   > **`dk_join3`: the fourth merger is `FUN_10b3baa8` @ `0x10b3baa8`**, whose
   > worker is `FUN_10b3a790` @ `0x10b3a790` — a **textual** tail merger over
   > **every pair in a label's predecessor list**, using the same `0x10b36f7e`
   > equivalence and the same `0x10b36e93` / `0x10bd5648` commit as K1, and
   > **calling `0x10b328da` nowhere at all**. `mg_arm3` (this cell restated) at
   > `/O1`: `A0` **1** copy → `A123` **2** → `A123`+`0x10b3baa8` **3**, the
   > source count. The lead was that **`0x10b36f7e` has SEVEN callers**, not
   > three; §2 above reads only the three that build a DAG.
   >
   > **`dk_loop_join`: NOT a merge.** It collapses 2 → 1 even with the entire
   > driver `0x10b3c2cc` patched to `return 0`, and the `/FAsc` listing shows
   > the store **hoisted into the loop preheader** — loop-invariant code
   > motion. This half of the item is **corrected, not confirmed**.
   >
   > The merger set under `0x10b3c6e5` is now **closed on that lane's grid and
   > this one's**: with K1, K2, K3 and `0x10b3baa8` ablated, killing the whole
   > driver changes **no copy count** in 13 cells × 6 optimization levels.
   > `#3103`'s four named candidates went **0 for 4** — `0x10b3a253`,
   > `0x10b38cd4` and `0x10b388eb` are each entered and byte-neutral, and
   > `0x10b36805` **cannot be ablated at all** (its caller performs the
   > `0x10bd38b0` splice before calling it, so short-circuiting it makes `cl`
   > fail).
   >
   > See [`WB_MERGER4_FINDINGS.md`](WB_MERGER4_FINDINGS.md) §1, §4.1, §4.4,
   > §4.6, §4.7 and board `#3109`–`#3113`.
5. **The K1/K2 redundancy** — which one wins when both are live is decided by
   `0x10b3c2cc`'s branch classification (`tuple+0x34`, opcodes `0x2e4`/`0x21`/
   `0x22`), and those opcode numbers were **not decoded**.

   > **⛔ HALF-CLOSED — lane `w-2e4`, 2026-08-24, board #3502.** `[R]`. The
   > opcode numbers are decoded: **`0x21` is `bc`, `0x22` is `bca`** (c2's own
   > mnemonic table), and **`0x2e4` is an unnamed one-operand branch pseudo-op
   > whose operand is a label** ([`WB_2E4_FINDINGS.md`](WB_2E4_FINDINGS.md)).
   > The classification `0x10b3c2cc` performs is written out verbatim in
   > `inline.c` `0x10b6e99b` and `p2symtab.c` `0x10b9f04e`:
   > `PLAIN_CONDITIONAL(t) := kind == 0x12 && t[+0x34] == 0 && opcode ∉ {0x2e4, 0x21, 0x22}`,
   > and the two halves are **not** redundant — a fresh `0x2e4` has
   > `t[+0x34] == 0` like a plain branch, because the constructor never writes
   > that field. **Which of K1/K2 wins is still not decided here**; only the
   > predicate that splits them is.
6. **`0x10b39794`'s `0x50`-node pool** is the same constant as the region cap.
   Whether that is one constant or two is not established.
7. **`mode = 1` is dead in this build**, so the `0x10b3c065` → K1 route is
   unreachable. Established from the four call sites of `0x10b3c6e5`; an
   indirect call through a table would defeat that, and `xrefs.tsv` shows none,
   which is an absence — hence grey-zone rather than a claim.

## 7. Prereg scored

### R1 (frozen before the first grep)

| id | claim | p | outcome |
|---|---|---|---|
| M1 | ≥1 of K1–K4 re-threads the tuple list | **0.20** | **TRUE — and badly under-predicted.** K1 and K2 both do |
| M2 | K4 is read-only w.r.t. tuple order | 0.85 | **TRUE** |
| M3 | K1–K3 share **one** gating condition, not bit 21 alone | 0.55 | **FALSE as written.** They share `DAT_10c3de20 != 1` + mode 2 + the optimizer bit; K3 adds `DAT_10c2e310 == 0` |
| M4 | at `/Od` none of K1–K4 runs | 0.65 | **FALSE.** K1–K3 confirmed not entered; **K4 IS entered at `/Od`** under `/QXSTALLS /FAsc` |
| M5 | K1–K3's gate is reachable from a `cl` command line | 0.50 | **TRUE** (any `/Og`-class line) |
| M6 | K1–K3 do not call `0x10be5d4b` | 0.80 | **TRUE** — it has exactly one caller |
| M7 | ≥1 of K1–K3 is a loop / cross-**block** analysis | 0.40 | **TRUE** in the cross-block reading; none is a loop analysis |
| M8 | `/QXSTALLS` changes the obj on ≥1 cell | 0.30 | **FALSE** (5 levels × 14 functions) — correctly predicted |
| M9 | the detecting cell exists **and stays green** | 0.55 | **FALSE — the cell exists and went RED.** The lane's headline |
| M10 | ≥1 client shares an already-built DAG | 0.25 | **FALSE** — each allocates and resets its own graph |

6 of 10 resolved against the registered direction. The two that matter, M1 and
M9, were both bet the wrong way at `p = 0.20` / `0.55`, which is the honest
record of a lane that expected to confirm a negative and did not.

### R2 (frozen by content hash before the first `cl.exe`)

| id | prediction | p | outcome |
|---|---|---|---|
| P1 | `/Od`: two copies everywhere | 0.90 | **HIT** |
| P2 | `dt_sfx@/O1` one copy | 0.80 | **HIT** |
| **P3** | **`dt_sink@/O1` one copy** | **0.70** | **MISS — two copies at every optimized level** |
| P4 | `dt_dep@/O1` two copies (dependence-gated) | 0.60 | **GREY-ZONE** — true, but its control also read two, so it discriminates nothing (§6.3) |
| P5 | `dt_none` never merges | 0.95 | **HIT** |
| P6 | `dh_hoist@/O1` one copy | 0.50 | **MISS** |
| P7 | a cell splits on favor-speed, identically at both `/O` levels | 0.45 | **HIT, larger than predicted** — the *whole obj* splits, 10/10 pairs. Its stated cause (K3) is **wrong**: K1 is the favor-speed-sensitive client, via `0x10b397ba` |
| P8 | `/QXSTALLS` obj-identical in all region shapes, run-proof present | 0.85 | **HIT** |
| P9 | the `> 0x50`-tuple body also identical | 0.80 | **HIT** |
| P10 | probe validation passes (PROC count, a real branch) | 0.90 | **HIT** — 14/14 `PROC`, exactly one conditional branch per family-T/H cell |
| P11 | `dt_mid` merges, `dt_far` does not (the `0x1d` window) | 0.35 | **MISS** — premise P3 failed, so the window was never exercised |

### Grid 2 (frozen in its own header before its first `cl.exe`)

| id | prediction | p | outcome |
|---|---|---|---|
| G2-1 | `A3` changes something at favor-size | 0.45 | **MISS** |
| G2-2 | `A3` changes nothing at favor-speed | 0.85 | **HIT** (vacuously — it changes nothing anywhere) |
| G2-3 | `A123` differs from `A0` somewhere here | 0.90 | **HIT** — 4 of 6 functions |
| G2-4 | `dk_join3` one copy at favor-size | 0.60 | **HIT** (1 copy; 3 at favor-speed) |

**The registered discriminator failed and the lane still landed**, because the
instrument that answered the question — ablation — was not the one registered.
That is worth saying plainly: R2's grid was designed to detect motion by
*shape*, and the shape it chose (`dt_sink`) turned out not to be one c2 merges.
The ablation ladder was added after grid 1's first read, is not preregistered,
and its results are therefore **exploratory rather than confirmatory** —
except for `A123` on `dt_sfx`, which tests P2's already-registered cell.

## 8. Correction to `WB_DAGORDER_FINDINGS.md`

Filed in place, in that document's own revision-box convention. Nothing in
`wb-dagorder`'s scheduler reading is contradicted; its **scope** is.

## 9. Pre-drafted DISCLOSURE rows — NONE

Nothing here is adopted by `crates/`. A future lane that implements §5's step
3b owes `DISCLOSURE.md` rows for `0x10b3c6e5`, `0x10b3c2cc`, `0x10b3b167`,
`0x10b3b41b`, `0x10b3b5fd`, `0x10b3ada1`, `0x10b3ad62`, `0x10b36f7e`,
`0x10bd38b0`, `0x10bd3892` and `0x10b397ba` in the same commit.
