# WB_MERGER4 — the fourth block merger is `0x10b3baa8` → `0x10b3a790`, and it is not a DAG client

> **PROVENANCE — DISASSEMBLY-DERIVED.** Every address below is an absolute VA in
> the exact image pinned in [`C2_MAP_METHOD.md`](C2_MAP_METHOD.md) §0 —
> `sha256 c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`,
> verified on `compilers/X360/16.00.11886.00/c2.dll` at the top of this lane and
> **re-verified unchanged at its tip**. Navigation only. **This lane adopts
> nothing into `crates/` and adds no `DISCLOSURE.md` row.**

PREREG: [`WB_MERGER4_PREREG.md`](WB_MERGER4_PREREG.md) (R1) committed at
`8667f7a1` **before the first grep of the export**;
[`WB_MERGER4_PREREG_R2.md`](WB_MERGER4_PREREG_R2.md) with the grid
(`grids/wb-merger4/merger4_grid.cpp`, sha256 `d61a3477db50ad29…`) at `7d10cdd9`
**before the first `cl.exe`**. Scored in §7.

**The commissioned question** — board **`#3103`**, `w-dagclients`' own stated
relocation: *with `0x10b3b167`, `0x10b3b41b` and `0x10b3b5fd` all ablated,
`dk_join3` still collapses three copies of a common tail store to two. Find the
fourth merger, or establish that the residual has another cause.*

---

## 1. THE HEADLINE

**`#3103` is CLOSED, and both halves of it resolve — one to an address, one to
a different transform entirely.**

1. **The fourth block merger is `FUN_10b3baa8` @ `0x10b3baa8`, whose worker is
   `FUN_10b3a790` @ `0x10b3a790`.** It is a **purely textual tail merger over
   a label's predecessor list** — it walks two predecessors backwards in
   lockstep through `tuple+0x10`, compares them with **`0x10b36f7e`**, the very
   equivalence K1/K2/K3 use, and commits with `0x10b36e93` / `0x10bd417d` /
   `0x10bd5952` / `0x10bd5648`. **It never calls `0x10b328da`** — there is no
   dependence DAG anywhere in it, which is exactly why `#3103` could say
   "a fourth block merger that is NOT a DAG-builder client" and be right.
2. **The ladder is exact on the commissioned cell.** `mg_arm3` (`dk_join3`
   restated) at `/O1`: **`A0` 1 copy → `A123` 2 copies → `A123B1` 3 copies**,
   and 3 is the source arm count. `#3103`'s residual 3 → 2 **is** `0x10b3baa8`,
   in full, and ablating it restores the source exactly.
3. **The merger set inside the driver is CLOSED.** `AFULL` — the whole of
   `0x10b3c2cc` patched to `return 0` — has **the same copy count as `A123B1`
   in every one of the 13 cells at every one of the 6 optimization levels**.
   With K1, K2, K3 and `0x10b3baa8` dead, killing the entire driver removes no
   further duplicate. There is no fifth merger under `0x10b3c6e5`.
4. **`#3103`'s other data point was not a merge at all.** `dk_loop_join`'s
   collapse survives `AFULL`, and the listing says why: the store is
   **hoisted out of the loop into the preheader** — loop-invariant code motion,
   not block merging (§4.4). `#3103` read a copy-count drop as merging; that
   half of the row is **corrected, not confirmed**.
5. **Three of `#3103`'s four named candidates are positively excluded**, and
   the fourth **cannot be ablated at all**: `0x10b3a253`, `0x10b38cd4` and
   `0x10b388eb` are each **entered** on the grid (`ud2` traps at `/O1` and
   `/O1 /Ot`) and each leaves the obj **byte-identical** at all levels, alone
   and on top of `A123B12`; `0x10b36805` patched to `ret` makes `cl` **fail to
   compile at every level**, because its caller has already performed the
   `0x10bd38b0` splice by the time it is called. **`#3103`'s candidate list was
   0 for 4.**

## 2. Where it lives

The lead that found it: **`FUN_10b36f7e` @ `0x10b36f7e`, the pairwise
tuple-equivalence test, has SEVEN callers, not three.** `w-dagclients` read the
three that build a DAG. The other four are the merger family it did not see.

| what | address | notes |
|---|---|---|
| **M4 — the fourth merger, driver** | **`0x10b3baa8`** | called from `0x10b3c2cc`'s **label** class (`tuple+8 == 0x1b`) at `LAB_10b3c4a5`, gated **`param_2 == 2`** and **`DAT_10c3de20 != 1`**. Walks the label's **predecessor list** `label+0x28`, marks each predecessor whose terminator is a plain conditional branch (`tuple+0x34 == 0`, opcode ∉ {`0x2e4`,`0x21`,`0x22`}), and offers **every pair** to the worker |
| **M4 — the worker** | **`0x10b3a790`** | walks both predecessors backwards in lockstep through `tuple+0x10`, skipping labels (`0x1b`) and `0x317` pseudo-tuples, comparing each pair with **`0x10b36f7e`**; `local_10` counts the matched suffix. **No `0x10b328da` — no dependence DAG at all** |
| M4's commit | `0x10b3ab00` `call 0x10b36e93` | then `0x10bd417d` / `0x10bd5952` (retarget) / `0x10bd5648`. `0x10b36e93` and `0x10bd5648` are **the same commit pair K1 uses** |
| M4's re-thread helper | `0x10b3706f` | calls **both** splices `0x10bd38b0` and `0x10bd3892` |
| **M4's size budget** | inside `0x10b3a790` | `(-(DAT_10c2e310 != 0) & 0x12) + 2` — **2 at favor-SIZE, `0x14` = 20 at favor-SPEED**. `DAT_10c2e310` is `#1611`'s favor-speed bit |
| M4's length-1 guard | `0x10b3a988` `call 0x10bd4461` | at favor-size a **one-tuple** match is rejected when the tuple before the predecessor is a `0x12` branch that is **not** plain-conditional |
| **M5 — a fifth merger of the same family** | **`0x10b3ab86`** → **`0x10b394f5`** | the same lockstep walk, but over **pairs of predecessors that both end in a conditional branch**; mints a fresh label (`0x10b9a455` / `0x10bd415e` / `0x10bd3824`) and commits at `0x10b39697`. **Entered on every optimized cell here and never once reaches its `call 0x10b394f5`** (§4.5) |
| the compare/branch fuser, for contrast | `0x10b3a253` → `0x10b3a025` → `0x10b39150` | uses `0x10b36f7e` only to test **compare** tuples (opcodes `0x2d`/`0x2e`/`0x2f`/`0x30`/`0x11e`/`0x120`) for redundancy. **Not a code merger**, and measured byte-neutral (§4.6) |
| `0x10b35f88`, read first per `#3101` | `0x10b35f88` | **it is not a block search — it is a complementary-branch test.** Walks `tuple+0x10` back past labels; requires a `0x12` that is **not** plain-conditional, whose cc `(tuple+10) & 0x1f` equals **`DAT_10b189cc[(other+10) & 0x1f]`** — a **condition-inversion table** — whose branch targets resolve to the same object, and whose target's first non-label tuple is the caller's own tuple. So K3's "second block" is *the fallthrough predecessor reached by the inverted branch of the same test* |

**Gate summary for M4 and M5**, complete:

    M4 (0x10b3baa8)  DAT_10c3de20 != 1  and mode == 2  and reached via the LABEL class
    M5 (0x10b3ab86)  DAT_10c3de20 != 1  and mode == 2  and reached via the LABEL class

Both therefore run in **the same `0x10b7ded5` invocation as K1/K2/K3**, before
`0x10b7df57`'s final schedule — so, exactly as `#3099` says of K1/K2, **their
output is the scheduler's input.** `DAT_10c3de20 == 1` (`/LTCG:PGI`) disables
them, which is now true of **five** transforms, not three.

## 3. THE INSTRUMENT

`#3100`'s method, reused: ablation of patched **copies** of the pinned image,
plus a `ud2` reachability ladder. Nine ablation images and fifteen trap images;
`work/w-merger4/{patch.py,trap.py,mkimg.sh,run.sh,probe.sh,count.sh}`, ~150
lines. File offset = `VA − 0x10b00c00` (`.text` VMA `0x10b01000` at file
`0x400`); every patch site's original bytes are asserted against
`objdump_intel.asm` before writing, and **every replacement is a legal early
return for that function's own calling convention**, read off its `ret`
immediate (`0x10b3baa8` and `0x10b3ab86` end in a bare `ret`, so both take two
register arguments and are patched `33 c0 c3`).

| image | patch | sha256 of `c2.dll` |
|---|---|---|
| `A0` | none — the control | `c80981c015166eff…` (= the pinned image) |
| `A123` | K1 `0x10b3b167`, K2 `0x10b3b41b`, K3 `0x10b3b5fd` | `116c2d0f238d8359…` |
| `B1` | **M4 `0x10b3baa8`** | `d2468299930ea7de…` |
| `B2` | M5 `0x10b3ab86` | `0a2e722fba3ec584…` |
| `A123B1` | `A123` + `B1` | `f0e527ca2bc004e2…` |
| `A123B12` | `A123` + `B1` + `B2` | `c0b801ad68567b23…` |
| `A123B12C` | + all four of `#3103`'s named candidates | `a7f174e81c6b3b40…` |
| `AFULL` | **`0x10b3c2cc` → `xor eax,eax; ret 0xc`** — the whole driver | `fb5e32aa3d7a5eab…` |
| `NULL` | `0x10c1ce93` (K4) → `ret` — the **measured null** | `98a1c92841cf2dd3…` |
| `C1`…`C4`, `A123B12_C1`…`_C4` | the four candidates, singly and on top | see `work/w-merger4/mkimg.sh` |

Three validity checks, each of which could have failed:

* **`A123`'s image hash `116c2d0f238d8359…` and `NULL`'s `98a1c92841cf2dd3…`
  reproduce `w-dagclients`' `A123` and `A4` byte for byte** — the reconstruction
  of that lane's patch set from its published table is exact, independently of
  its scratch tree.
* **`NULL` is byte-identical to `A0` at all six optimization levels.** K4 never
  runs without `/QXSTALLS /FAsc` (`#3102`), so a patched byte in a function that
  does not execute must change nothing — **the instrument's own null is
  measured, not assumed**, per `#3100`.
* **The `ud2` ladder agrees with the obj deltas in every cell** (§4.5): M4
  reaches its commit exactly where `B1` differs from `A0`, and does not
  elsewhere. Two independent instruments, no disagreement.

`#3100`'s trap was held throughout: every variant compiles to **one fixed
`/Fo` and `/Fa` path** and is copied afterwards, and `TimeDateStamp` (file
offset 4..8) is zeroed before any hash compare.

**These images are ablation controls, never oracles.** Every behavioural claim
is read off the unpatched `A0` build.

## 4. THE ANSWER

### 4.1 The commissioned cell — `#3103`'s residual, fully attributed

`mg_arm3` is `dk_join3` restated: three predecessors of one join, each arm
ending in `dc_c = 9`. At `/O1` (favor-size), copies of the `dc_c` store:

| image | K1/K2/K3 | M4 `0x10b3baa8` | copies | instructions |
|---|---|---|---|---|
| `A0` | live | live | **1** | 19 |
| `A123` | **dead** | live | **2** | 24 |
| `B1` | live | **dead** | **1** | 19 (= `A0`) |
| **`A123B1`** | **dead** | **dead** | **3** | 25 |
| `A123B12` | + M5 dead | | **3** | 25 |
| **`AFULL`** | **the whole `0x10b3c2cc` dead** | | **3** | 25 |

`A123` reproduces `#3103`'s observation exactly — 3 copies collapse to 2 with
the three DAG clients dead. **Adding `0x10b3baa8` to the ablation takes it to
3, the source count**, and adding the entire rest of the driver on top takes it
no further. `mg_arm4` does the same with four arms: `A0` 1, `A123` 2,
`A123B1` **4** = the source count.

### 4.2 The cell that could have gone red, and did — `mg_cond2`

`mg_cond2` is two **separate** `if` nests whose inner then-arms both end in
`dc_c = 9`. Its merged blocks are not the two arms of one branch, so neither
K1's nor K2's shape applies. At `/O1`:

```
A0 (pinned)                    A123 (K1+K2+K3 dead)          B1 (only M4 dead)
  ... arm dc_a ...               ... arm dc_a ...              lis  r9,dc_a
  b     $LN10          <--       b     $LN9         <--        lis  r8,dc_c
$LN5: ... dc_x2 ...  blr       $LN5: ... dc_x2 ... blr         li   r11,1
$LN6: ... arm dc_b ...         $LN6: ... arm dc_b ...          li   r10,9
$LN10:                         $LN9:                           stw  r11,dc_a(r9)
  lis   r8,dc_c                  stw   r10,dc_c(r8)  <-- ONE   stw  r10,dc_c(r8)  <-- copy 1
  li    r10,9                                                  blr
  stw   r10,dc_c(r8)   <-- ONE  25 -> 27 instructions         $LN6: ... arm dc_b ...
  blr                                                          stw  r10,dc_c(r8)  <-- copy 2
  1 copy, 25 instructions        1 copy                        2 copies, 28 instructions
```

**`A123` still merges it; `B1` does not.** On this cell `0x10b3baa8` is
**necessary and sufficient** and K1/K2/K3 are **neither** — the exact opposite
polarity from every cell `w-dagclients` measured. `A0` and `A123` agree on the
registered metric (1 copy) and `B1` disagrees, which is R2's `Q9` verbatim.
The motion crosses a `blr`-terminated block and two separate `if` nests;
`#3069` (15/15) establishes the scheduler cannot cross a branch at all.

### 4.3 What merges, and when — the fire table

Copies of the common store, `A0` (the pinned image), source arm count in
brackets. `/O1 ≡ /O2 /Os` and `/O1 /Ot ≡ /O2` **in all 13 cells × 8 images**,
0 exceptions — `#1611`'s deciding quad reproduced on a third unrelated
construct.

| cell | shape | `/Od` | `/O1` favor-size | `/O1 /Ot` favor-speed |
|---|---|---|---|---|
| `mg_arm2` [2] | 2 arms | 2 | **1** | **1** |
| `mg_arm3` [3] | 3 arms | 3 | **1** | 3 |
| `mg_arm4` [4] | 4 arms | 4 | **1** | 4 |
| `mg_len1` [3] | common tail 1 stmt | 3 | **1** | 3 |
| `mg_len2` [3] | common tail 2 stmts | 3 | **1** | **1** |
| `mg_len4` [3] | common tail 4 stmts | 3 | **1** | **1** |
| `mg_call3` [3] | a **call** before the tail | 3 | **1** | **1** |
| `mg_callin3` [3] | the call **is** the tail | 3 | **1** | **1** |
| `mg_head3` [3] | common code at the **head** | 3 | **1** | 3 |
| `mg_mid3` [3] | middle arm's tail differs | 3 | **2** | 3 |
| `mg_cond2` [2] | two separate `if` nests | 2 | **1** | 2 |
| `mg_none3` [3] | same store, **different values** | 3 | **1** | 3 |
| `mg_loop3` [2] | 2-arm `if` in a loop | 2 | **1** | **1** |

Three structural facts fall straight out:

* **At `/Od` nothing merges at all** — every cell equals its source arm count,
  in every image except `AFULL` (§4.7). The mergers are `/Og`-gated.
* **At favor-SPEED a 3-or-more-arm join merges only when the common tail is
  ≥ 2 statements** (`mg_len1` 3 copies vs `mg_len2` 1). At favor-SIZE all of
  them merge. That is `0x10b3a790`'s `2` vs `20` budget and K1/K2's own window
  (`0x10b397ba`, which `#3099` already showed reads `DAT_10c2e310`) acting
  together — and it is the axis this grid was built to cross.
* **`mg_none3` is not the control it was written to be, and the reason is a
  mechanism finding.** Its three arms store *different constants* to the same
  variable, so as C statements they share no common tail — but c2 merges them
  anyway, to **one** `stw`, because at the tuple level the common suffix is the
  **store tuple** and the differing `li r10,N` value materialization is a
  *separate, earlier* tuple that stays in the arms. **The merger matches
  tuples, not statements.** This is a direct partial answer to
  `w-dagclients` §6 item 2, its self-declared largest hole.

### 4.4 `dk_loop_join` is loop-invariant code motion, not a merge — `#3103` corrected

`mg_loop3` (`dk_loop_join` restated) collapses 2 → 1 on **every** image
including `AFULL`, i.e. with the entire merge driver dead. The `/FAsc` listing
of `AFULL` at `/O1` says why, and it is not a merge:

```
  ...
  lis   r7,dc_c
  li    r10,9
  stw   r10,dc_c(r7)        <-- in the PREHEADER, above $LL5
$LL5@mg_loop3:              ; Start of loop
  cmpwi cr6,r4,0
  ...
```

Both arms store the same loop-invariant value to the same loop-invariant
address on every iteration, so the store is **hoisted out of the loop
entirely**. `AFULL`'s listing here is byte-identical to `A0`'s. **`#3103`'s
second data point — "`dk_loop_join` merges under the same ablation" — is
wrong: nothing merged it.** This is the "the residual has another cause" branch
of the commission, and it fires on one of the row's two functions.

### 4.5 The `ud2` ladder — positive, and it agrees with the obj

`ud2` (`0f 0b`) at each site; **TRAPPED** = reached.

| probe | site | `/O1` | `/O1 /Ot` | `/Od` |
|---|---|---|---|---|
| `XB1` M4 entered | `0x10b3baa8` | **TRAP** | **TRAP** | · |
| `RB1` reached `call 0x10b3a790` | `0x10b3bb3a` | **TRAP** | **TRAP** | · |
| **`FB1` the worker returned nonzero — it FIRED** | `0x10b3bb5a` | **TRAP** | · | · |
| `X790` worker entered | `0x10b3a790` | **TRAP** | **TRAP** | · |
| `R790` reached `call 0x10b36f7e` — past every gate | `0x10b3a925` | **TRAP** | **TRAP** | · |
| **`F790` reached its COMMIT `call 0x10b36e93`** | `0x10b3ab00` | **TRAP** | · | · |
| `XB2` M5 entered | `0x10b3ab86` | **TRAP** | **TRAP** | · |
| **`RB2` reached `call 0x10b394f5`** | `0x10b3ac35` | · | · | · |
| `X4f5` / `R4f5` / `F4f5` | `0x10b394f5` / `…5fe` / `…697` | · | · | · |
| `XC1` `0x10b3a253` | | **TRAP** | **TRAP** | · |
| `XC2` `0x10b36805` | | **TRAP** | **TRAP** | **TRAP** |
| `XC3` `0x10b38cd4` | | **TRAP** | **TRAP** | · |
| `XC4` `0x10b388eb` | | **TRAP** | **TRAP** | · |

Two things this buys that an obj delta alone cannot:

* **M4's favor-speed inactivity is measured positively.** At `/O1 /Ot` it is
  entered, reaches its worker, and the worker walks and compares tuples
  (`R790` traps) — and **never reaches its commit**. "M4 is favor-size-effective
  on this grid" is therefore a statement about where it *stops*, not an
  absence.
* The ladder and the obj agree in all 6 grid × level cells tried: `FB1`/`F790`
  trap exactly where `B1 ≠ A0`.

### 4.6 `#3103`'s four named candidates: 0 for 4, and one of them cannot be tested

| candidate | entered? | obj vs `A0`, all 6 levels | on top of `A123B12` |
|---|---|---|---|
| `0x10b3a253` | **yes**, `/O1` + `/O1 /Ot` | **byte-identical** | identical to `A123B12` |
| `0x10b36805` | **yes**, incl. `/Od` | **`cl` FAILS to compile**, all 6 levels | fails |
| `0x10b38cd4` | **yes**, `/O1` + `/O1 /Ot` | **byte-identical** | identical to `A123B12` |
| `0x10b388eb` | **yes**, `/O1` + `/O1 /Ot` | **byte-identical** | identical to `A123B12` |

Three are excluded **positively** — entered, and changed nothing — rather than
by absence. `0x10b36805` is a different kind of answer: `0x10b3c2cc` performs
the `0x10bd38b0` splice **itself**, immediately before calling it, and
`0x10b36805` is what completes the block concatenation; short-circuiting it
leaves the tuple list inconsistent and `cl` dies. **It is not ablatable in
isolation, so no ablation lane can ever score it** — which is worth recording,
because `#3103` proposed exactly that experiment.

### 4.7 What `AFULL` still does — and why it is not a sixth merger

`AFULL` differs from `A123B12` in three places, and **none of them changes a
copy count**:

* at `/Od`, **+1 instruction in every one of the 13 functions** (e.g.
  `mg_arm3` 29 → 30), obj 7333 → 7393;
* at favor-speed, **+1 instruction in `mg_loop3` only** (22 → 23), obj
  8049 → 8053;
* at favor-size, **nothing at all** — `AFULL` is byte-identical to `A123B1`.

`XC2` traps at `/Od` while `XB1`, `XB2`, `XC1`, `XC3` and `XC4` do not, so the
`/Od` delta is the **block-concatenation / label path** (`0x10b36805`,
`0x10b37285`, `0x10b3c2cc`'s own two `0x10bd38b0` calls) removing one redundant
branch per function. That is CFG cleanup, not duplicate elimination.

**Therefore: on this grid, with `0x10b3c2cc` entirely dead, `c2` removes no
duplicated code except loop-invariant code motion (`mg_loop3`, §4.4) — 12 of
13 cells match their `/Od` copy counts exactly.** That is the closure argument
for the merger set, and it is a positive one: not "we found no fifth merger"
but "killing everything that could have been one changes no copy count".

### 4.8 Discriminating cells

Printed by `count.sh` so a vacuous run is loud (R2 § "Loud failure"):

| level | discriminating cells |
|---|---|
| `/O1` | **12** of 13 |
| `/O2 /Os` | **12** of 13 |
| `/O1 /Ot` | **6** of 13 |
| `/O2` | **6** of 13 |
| `/Ox` | **6** of 13 |
| `/Od` | **13** of 13 |

## 5. What this costs the port

`WB_DAGCLIENTS_FINDINGS.md` §5's step `3b` is **incomplete in the same way
`WB_DAGORDER_FINDINGS.md` §7 was**. The corrected shape:

    3b. BLOCK MERGE (0x10b7ded5, up to 17 rounds of 0x10b3c2cc), FIVE clients,
        all gated mode == 2 and DAT_10c3de20 != 1 (not /LTCG:PGI):
      i.   0x10b3b167  DAG tail merge / cross jump      (branch class)
      ii.  0x10b3b41b  DAG head merge / hoist           (complement class)
      iii. 0x10b3b5fd  DAG tail merge, second block via 0x10b35f88's
                       COMPLEMENTARY-BRANCH test (favor-size only)
      iv.  0x10b3baa8  TEXTUAL tail merge over ALL PAIRS in a label's
                       predecessor list — no DAG. Budget 2 (favor-size) /
                       20 (favor-speed). THIS is what merges 3-and-4-arm joins.
      v.   0x10b3ab86  TEXTUAL tail merge over pairs of conditional-branch
                       predecessors — no DAG. Never fired here.
        Matching is TUPLE-wise (0x10b36f7e), so `x=9` and `x=8` in two arms
        DO share a common tail: the store tuple.
    …then the LAST schedule runs over the re-threaded list.

The practical consequence sharpens `#3099`'s: a port that implements the
scheduler **and** the DAG mergers will still disagree with c2 on every
**three-or-more-predecessor join** at favor-size, which is the dc3 workload's
`/O1 /Oi` setting, because that shape is `0x10b3baa8`'s and no DAG client
covers it.

## 6. Grey-zone / not established

Filed, not banked.

1. **Whether M5 `0x10b3ab86` ever fires on any input.** Entered 2/2 at `/O1`
   and `/O1 /Ot`; **never reaches its `call 0x10b394f5`** on any of 13 cells.
   Its worker `0x10b394f5` is a full merger by construction (it mints a label
   and commits at `0x10b36e93`), so this is a statement about the **grid**, not
   about M5 — the same shape as `#3101`'s K3, and filed the same way. `mg_cond2`
   was written specifically to reach it and did not. **No claim either way.**
2. **Whether K3 `0x10b3b5fd` ever fires.** Still open — this lane did not
   re-probe it, but §2 now says what its second block *is* (`0x10b35f88`: the
   **inverted-condition** branch to the same target, via the inversion table
   `DAT_10b189cc`), which is the shape a future grid should write.
3. **The closure claim is grid-bounded.** "No sixth merger" rests on `AFULL`
   changing no copy count **on these 13 cells at these 6 levels**. A shape none
   of them carries could still be merged elsewhere. This is stated as the
   coverage-bounded claim it is, not as a proof — `#3071` → `#3103` → this lane
   is three relocations of exactly that caveat.
4. **The exact meaning of `0x10b3a790`'s budget arithmetic** — `iVar12` counts
   something over `0x10bf96c6` and `uVar9` is the max of the two blocks'
   `tuple+0xe` shorts. The *direction* (favor-speed needs a longer common tail)
   is measured on `mg_len1`/`mg_len2`; the *units* are not decoded.
5. **`DAT_10b189cc` is read as a condition-inversion table** from its use in
   `0x10b35f88` and the sibling `DAT_10b189e0` / `DAT_10b18a30` bitmask tables
   in `0x10b388eb`. The tables themselves were **not dumped**.
6. **`0x10b36805` is untestable by ablation** (§4.6). Its role is read from the
   call site only.

## 7. Prereg scored

### R1 (frozen before the first grep)

| id | claim | p | outcome |
|---|---|---|---|
| N1 | **C-D**: c1xx already emitted < 3 copies | 0.35 | **FALSE.** `AFULL` and `A123B1` both give 3 — c2 receives three and merges them |
| N2 | **C-A**: one of the four named candidates | 0.30 | **FALSE.** 0 for 4 (§4.6) |
| N3 | ≥1 named candidate is entered | 0.85 | **TRUE** — all four are |
| N4 | ≥1 named candidate rewrites tuple links | 0.50 | **TRUE but degenerate** — `0x10b36805`'s caller splices *for* it, which is why it cannot be ablated |
| N5 | the fourth cause is favor-size-**independent** | 0.60 | **FALSE.** M4 commits only at favor-size (§4.5) |
| N6 | a cell where `A0` ≡ `A123` and the fuller ablation differs | 0.55 | **TRUE — `mg_cond2`** (§4.2) |
| N7 | reading `0x10b35f88` yields a nameable structural condition | 0.70 | **TRUE** — the complementary-branch test (§2) |
| N8 | K3 made to fire | 0.30 | **FALSE** — not attempted; see §6.2 |
| N9 | the surviving copy count is **not** constant across 2/3/4 arms | 0.65 | **TRUE** — 2 / 3 / 4, it equals the source count |
| N10 | `/Od` gives the full source count | 0.80 | **TRUE**, 13/13 |
| N11 | **`#3103` CLOSED** | 0.55 | **TRUE** — named to `0x10b3baa8`, and its second function corrected to LICM |
| N12 | the merger set is **closed** — no fifth relocation | 0.35 | **TRUE on this grid**, by `AFULL` (§4.7); grid-bounded per §6.3 |

**The lane's own framing was the thing it got wrong.** R1 spread `p` across
four causes and put **0.65** on C-A ∪ C-D; the answer was **C-B** — the cause
R1 wrote down but assigned no probability, "something else inside the driver
that the four-candidate list does not name". `#3103`'s candidate list was the
anchor, and hedging around it was not the same as ignoring it.

### R2 (frozen by content hash before the first `cl.exe`)

| id | prediction | p | outcome |
|---|---|---|---|
| **Q1** | `B1` on top of `A123` restores 3 on `mg_arm3` at favor-size | **0.60** | **HIT — the headline** |
| Q2 | `XB1` traps | 0.90 | **HIT** |
| Q3 | `F790` traps | 0.75 | **HIT** at favor-size |
| Q4 | `B2` alone does not restore `mg_arm3` | 0.55 | **HIT** |
| Q5 | `AFULL` gives the source count on `mg_arm3` | 0.75 | **HIT** — and equals `A123B1` exactly |
| Q6 | `NULL` byte-identical to `A0` at every level | 0.95 | **HIT**, 6/6 |
| Q7 | `mg_len1` / `mg_len2` disagree **at favor-size** | 0.45 | **MISS as written** — they disagree at favor-**speed** (3 vs 1 copies); at favor-size both merge to 1 |
| Q8 | `mg_none3` never merges in any image | 0.95 | **MISS, and instructive** — it merges everywhere, because matching is tuple-wise (§4.3) |
| Q9 | a cell where `A0` ≡ `A123` and `A123B1` differs | 0.70 | **HIT — `mg_cond2`**, on the registered copy-count metric |
| Q10 | `mg_call3` still collapses under `A123` | 0.65 | **HIT** — 3 → 2 across a call |
| Q11 | full ablation = source count on arm2/3/4 | 0.55 | **HIT**, 2 / 3 / 4 |
| Q12 | family L splits on favor-size vs favor-speed | 0.60 | **HIT** |
| Q13 | `mg_cond2` separates M5 from M4 | 0.40 | **MISS** — `mg_cond2` is M4's cell, not M5's; M5 never fired |

10 of 13 resolved as registered. The three misses are `Q7`, `Q8` and `Q13`, and
**`Q8` — the control that was supposed to be unmergeable — is the most useful
thing in the table**: it failed because the grid was written in C statements
and the merger works in tuples.

## 8. Corrections filed in place

* [`WB_DAGCLIENTS_FINDINGS.md`](WB_DAGCLIENTS_FINDINGS.md) §6 item 4 — dated
  revision box: the fourth merger is named, and its `dk_loop_join` half is
  corrected to LICM.
* [`WB_DAGCLIENTS_FINDINGS.md`](WB_DAGCLIENTS_FINDINGS.md) §2 — the K3 row's
  "second block found via `0x10b35f88`" is *searched* only in the loosest
  sense; §2 above gives what it actually tests.

Nothing in `WB_DAGORDER_FINDINGS.md` is touched by this lane.

## 9. Pre-drafted DISCLOSURE rows — NONE

Nothing here is adopted by `crates/`. A future lane implementing §5's step
`3b` iv/v owes `DISCLOSURE.md` rows for `0x10b3baa8`, `0x10b3a790`,
`0x10b3ab86`, `0x10b394f5`, `0x10b3706f`, `0x10b36e93`, `0x10b35f88` and
`DAT_10b189cc` in the same commit, in addition to `#3099`'s list.
