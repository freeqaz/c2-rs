# WB_MERGER4 — PREREG R2 (frozen by CONTENT HASH before the first `cl.exe`)

> **PROVENANCE — DISASSEMBLY-DERIVED.** Addresses are absolute VAs in the image
> pinned in [`C2_MAP_METHOD.md`](C2_MAP_METHOD.md) §0,
> `sha256 c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`,
> verified at the top of this lane. Navigation only; nothing is adopted into
> `crates/` and no `DISCLOSURE.md` row is owed.

R1 is [`WB_MERGER4_PREREG.md`](WB_MERGER4_PREREG.md), committed at `8667f7a1`
**before the first grep of the export**. This file is written **after** the
read of §2 and **before** the first `cl.exe` of the lane.

## The grid, frozen by content

| file | sha256 |
|---|---|
| `grids/wb-merger4/merger4_grid.cpp` | `d61a3477db50ad2924438416922ed575868273e1abcb0963fdd03c5467f9f6de` |

**Frozen by hash, not by name** — `w-keygen`'s rule. Any edit to the grid
invalidates every prediction below and requires a new file with a new hash,
`w-dagclients`' grid-2 convention.

## What the read has already established (R1 §2's `N3`/`N4`/`N7` are now resolvable)

`0x10b36f7e`, the pairwise tuple-equivalence test, has **seven** callers, not
three: `0x10b39150`, `0x10b394f5`, `0x10b3a253`, `0x10b3a790`, and K1/K2/K3.
Rooted at `0x10b3c2cc` this gives **two mergers `#3103`'s candidate list does
not name**:

| id | root | inner | shape |
|---|---|---|---|
| **M4** | `0x10b3baa8` | `0x10b3a790` | tail merge over **all pairs in a label's predecessor list** (`label+0x28`), no DAG |
| **M5** | `0x10b3ab86` | `0x10b394f5` | tail merge over **pairs of predecessors that both end in a conditional branch**, no DAG, mints a new label |

Both are called from `0x10b3c2cc`'s **label** class (`tuple+8 == 0x1b`) at
`LAB_10b3c4a5` under `param_2 == 2`, i.e. from the same `0x10b7ded5`
invocation as K1/K2/K3, and both are gated `DAT_10c3de20 != 1` (not
`/LTCG:PGI`). Neither calls `0x10b328da`, so **both satisfy `#3103`'s
"not a DAG-builder client"** exactly.

## The ablation ladder (patched COPIES; the pinned image is never modified)

| image | patch | convention |
|---|---|---|
| `A0` | none — the control | |
| `A123` | K1 `0x10b3b167`, K2 `0x10b3b41b`, K3 `0x10b3b5fd` → `return 0` | reproduces `w-dagclients` |
| `B1` | `0x10b3baa8` → `xor eax,eax; ret` | M4 |
| `B2` | `0x10b3ab86` → `xor eax,eax; ret` | M5 |
| `A123B1` | `A123` + `B1` | |
| `A123B12` | `A123` + `B1` + `B2` | the fullest client ablation |
| `A123B12C` | + `0x10b3a253`, `0x10b36805`, `0x10b38cd4`, `0x10b388eb` | **`#3103`'s four named candidates**, added on top |
| `AFULL` | `0x10b3c2cc` → `xor eax,eax; ret 0xc` | **the whole driver subtree dead** — the C-C / C-D discriminator |
| `NULL` | `0x10c1ce93` → `ret` | **the measured null.** K4 never runs without `/QXSTALLS /FAsc` (`#3102`), so this image **must** be byte-identical to `A0` |

Reachability ladder (`ud2`), the `#3101` shape — entered / past-gates / fired:

| probe | VA | meaning |
|---|---|---|
| `XB1` | `0x10b3baa8` | M4 entered |
| `RB1` | `0x10b3bb3a` | M4 reached its `call 0x10b3a790` |
| `FB1` | `0x10b3bb5a` | M4's inner call **returned nonzero** — it fired |
| `X790` | `0x10b3a790` | the pairwise walk entered |
| `R790` | `0x10b3a925` | it reached its `call 0x10b36f7e` — past every prologue gate |
| `F790` | `0x10b3ab00` | it reached its commit `call 0x10b36e93` |
| `XB2` | `0x10b3ab86` | M5 entered |
| `RB2` | `0x10b3ac35` | M5 reached its `call 0x10b394f5` |
| `X4f5` | `0x10b394f5` | M5's walk entered |
| `R4f5` | `0x10b395fe` | it reached `call 0x10b36f7e` |
| `F4f5` | `0x10b39697` | it reached its commit `call 0x10b36e93` |

## Registered predictions (R2)

| id | prediction | p |
|---|---|---|
| **Q1** | **`B1` on top of `A123` restores the full source copy count (3) on `mg_arm3` at favor-size** — M4 is the fourth merger | **0.60** |
| **Q2** | `XB1` traps on at least one optimized cell — `0x10b3baa8` is entered | **0.90** |
| **Q3** | `F790` traps on at least one cell — `0x10b3a790` reaches its commit | **0.75** |
| **Q4** | `B2` alone on top of `A123` does **not** restore `mg_arm3`'s count — M5 is not the responsible one on this shape | **0.55** |
| **Q5** | **`AFULL` gives the full source copy count on `mg_arm3`** — C-C and C-D refuted, the collapse lives inside `0x10b3c2cc`'s subtree | **0.75** |
| **Q6** | `NULL` is byte-identical to `A0` at every optimization level — the instrument's own null | **0.95** |
| **Q7** | `mg_len1` and `mg_len2` disagree at favor-size on at least one image — `0x10b3a790`'s length-1 guard has a visible cell | **0.45** |
| **Q8** | `mg_none3` never merges in **any** image at **any** level | **0.95** |
| **Q9** | a cell exists where `A0` and `A123` **agree** and `A123B1` (or `A123B12`) **disagrees** — the fourth merger's own red cell | **0.70** |
| **Q10** | `mg_call3` still collapses under `A123` — the residual crosses a **call**, which `#3069` proves the scheduler cannot | **0.65** |
| **Q11** | under the fullest ablation the copy count equals the source arm count on **all** of `mg_arm2` / `mg_arm3` / `mg_arm4` | **0.55** |
| **Q12** | family L splits on favor-size vs favor-speed on at least one image — `0x10b3a790`'s `(-(DAT_10c2e310 != 0) & 0x12) + 2` budget is observable | **0.60** |
| **Q13** | `mg_cond2` is the cell that separates M5 from M4: it merges on `A123B1` and stops on `A123B12` | **0.40** |

## Loud failure

The result tables print the count of **discriminating** cells — cells where at
least two of the nine images disagree. **0 discriminating cells is `FAILED`**,
not a clean negative (R1 §5).

## Trap held from `#3100`

`cl` embeds the `/Fo` and `/Fa` **path strings** in the obj (file offset
`0x44a`). Every variant compiles to **one fixed path** and is copied afterwards;
`TimeDateStamp` (offset 4..8) is zeroed before any hash compare.
