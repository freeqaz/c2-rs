# WB-J `wb-label` — PREREG round 3, frozen before the H- and S-cells are compiled

Lane `wb-label`, 2026-08-09. Third and last freeze. R1 was frozen before the
first export grep; R2 before the first `cl.exe` of any cell; **R3 is frozen after
X1–X6 and the primitives were measured and before `H1–H6` and `S1–S6` are
compiled.** The cell sources are already written (`work/wb-label/labgrid.py`,
`work/wb-label/seedgrid.py`) and are not edited after this commit.

> **Why a third freeze exists.** R2 §4 (P10) registered that the lane's
> *procedure* must be graded on cells it was not written from. X1–X6 refuted the
> construct-additive predictions this lane made from the binary alone (5 of 6
> missed, as P9.1 registered). The rule below is fitted to X1–X6 **and to the
> primitive grid measured after them**, so it needs a holdout, and this is it.

---

## 1. What has been measured (the fit set, not held out)

`scripts/gt_label_stride.py`'s construction with the subject **in the middle**,
`/O1 /GS- /c` and `/Ox /GS- /c`, `minted` read on every row. `lead = stride −
base` where `base` is measured in the same obj (5 `/Gy`, 4 packed).

| probe | `/O1` lead | `/Ox` lead | `/Ox` minted surcharge |
|---|---:|---:|---:|
| `p-none` (plain framed call) | 0 | 0 | 0 |
| `p-if` | 0 | 1 | 0 |
| `p-ifelse` | 0 | 0 | 0 |
| `p-for` | **2** | **10** | **+2** (the GPR helper pair) |
| `p-while` | **2** | **10** | **+2** |
| `p-dowhile` | 1 | 1 | 0 |
| `p-switch` (12 sparse arms + default) | 1 | 3 | 0 |
| `X1` = `p-switch` | 1 | 3 | 0 |
| `X2` (`for` with an `if` inside) | 2 | 3 | 0 |
| `X3` (`while` with an early `return`) | 3 | 3 | 0 |
| `X5` (`switch` inside a `for`) | **3** | **6** | 0 |
| `X6` (a 4-trip `for` c2 unrolls at `/Ox`) | 2 | 7 | 0 |

**The one regularity in the fit set:** `X5 = X1 + X2` exactly, at **both** modes
(3 = 1 + 2, and 6 = 3 + 3). That is the rule this round holds out.

## 2. The rule under test

> **R — CONSTRUCT-ADDITIVITY.** The lead of a body containing constructs
> `c₁ … c_n` is `Σ lead(cᵢ)`, with `lead` taken from the primitive table above,
> at the same mode.

## 3. H1–H6 — the held-out compositions

| cell | body | `Σ lead` `/O1` | `Σ lead` `/Ox` |
|---|---|---:|---:|
| **H1** | `if` inside a `while` | 2 + 0 = **2** | 10 + 1 = **11** |
| **H2** | `if/else` inside a `for` | 2 + 0 = **2** | 10 + 0 = **10** |
| **H3** | two sequential `if`s | 0 + 0 = **0** | 1 + 1 = **2** |
| **H4** | `switch` inside a `while` | 1 + 2 = **3** | 3 + 10 = **13** |
| **H5** | `for` inside a `for` | 2 + 2 = **4** | 10 + 10 = **20** |
| **H6** | `do/while` inside an `if` | 1 + 0 = **1** | 1 + 1 = **2** |

| # | prediction |
|---|---|
| **P11.1** | **`/O1`: ≥5 of the 6 HIT.** Registered as the optimistic half. |
| **P11.2** | **`/Ox`: ≤2 of the 6 HIT.** Registered as the pessimistic half, and registered *because* the fit set already shows `/Ox` splitting `X2` (3) from `p-for` (10) on bodies that differ only by an `if` inside the loop — a loop's `/Ox` lead is a property of what the unroller did, not of the keyword. |
| **P11.3** | If P11.1 and P11.2 both land, **the deliverable is a mode-split statement**: additive at `/O1`, not additive at `/Ox` — and even the `/O1` half is a *measured* regularity over seven primitives, **not a licence to put a number in `label_slots`**. |
| **P11.4** | `H5` (`for` in a `for`) is the single most likely `/O1` miss, because `LABEL_COUNTER.md` §4.2.1's `leaf-fornest` is `+4` over a leaf where two sequential loops (`leaf-for2`) are also `+4` — a nested pair and a sequential pair costing the same is the shape additivity cannot distinguish. |
| **P11.5** | `minted` is **0 surcharge** on all six at `/O1`, and **+2** (the GPR helper pair) on at least three of the six at `/Ox`. Scored separately: a lane that reports leads without the `minted` column double-charges, and §4's warning box says so. |

## 4. S1–S6 — the counterfactual form measured against the middle form

`work/wb-label/seedgrid.py`. Each cell is compiled twice: `w-json`'s
counterfactual `[subject, z]` (lead read off `z`'s `$M`) and the in-the-middle
`[a0, subject, a1, a2]` (stride read in-obj). Two cells carry **no control flow
at all**: `s_decl8` adds eight unused TU-level declarations, `s_loc8` eight
unused locals.

| # | prediction |
|---|---|
| **P12.1** | **`s_decl8`'s counterfactual lead is ≥ +4 and its in-TU stride is exactly 1** — identical to `s_ctl`'s. Eight declarations that emit not one instruction move the counterfactual reading. This is the whole claim, and one number kills it. |
| **P12.2** | **`s_loc8`'s counterfactual lead is ≥ +6 and its in-TU stride is exactly 1**, reproducing and explaining `w-bdnz`'s `lab_forever` row (*"two `int` locals cost +2 with no loop"*), which that lane read as a charge. |
| **P12.3** | **`s_loop`'s counterfactual lead reproduces `w-bdnz`'s `+7` at `/O1` to within ±2**, while its in-TU stride is **3** (leaf base 1 + the `for`'s 2, `LABEL_COUNTER.md` §4.2.1's `leaf-for` row). The gap between the two numbers is the finding. |
| **P12.4** | Registered as what would refute the reconciliation: if the counterfactual lead and the in-TU stride agree on **every** cell, then the four lanes' numbers are charges after all, this lane's §4 is wrong, and it says so. |
| **P12.5** | The mechanism claim behind P12.1: `c1xx` numbers **its own** symbols out of the same id space and hands c2 the next free value in `.gl`; c2's IL reader takes label ids **from the stream** (`FUN_10b9b5d2`-family, `+0x28 = FUN_10c1f91b()`, **no counter bump**) and only allocates fresh ids for labels it invents. So the counterfactual difference is `Δseed + Δcharge` and `Δseed` is a function of the source text. **Falsified if `s_decl8` moves nothing.** |
