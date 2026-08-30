# `#3847` — the multiply-blocked audit of all 12 `absent` rows

> **Lane `w-budget`, wave 21 L1. Deliverable 2.** Board **#3852**.
> Under `work/w-budget/PREREG.md` §5. Table: `work/w-inlmetric/CLAUSES.tsv`.

## 0. The question, and the answer in four lines

`#3847`: **the `blocker` column holds one cell per row, and C4 provably needs
two.** Nobody had checked which *other* rows are multiply blocked, so any count
taken off that column understates the work by an unknown amount.

> **Measured: 10 of the 12 `absent` rows carry a second blocker, and on FIVE of
> them the second is the BINDING one** — the cell that is published names the
> *cheaper* of the two. The `no-instr-count 4 / no-instr-stream 2 /
> emit-change 5 / writer-unread 1` partition that `#3816` publishes is
> therefore not a partition of the work; it is a partition of the *first reason
> somebody wrote down*.
>
> The column gets a sibling: **`blocker2`**, audited row by row below and
> landed in the same commit as this file. `-` is a real value and two rows
> carry it.

**The sharpest single finding, because it inverts a published readiness claim:**
**C10 and C12 are marked `R1` = *"READ AND DERIVABLE — a port counterpart could
be written today"*, and they are not derivable today.** Both test bits of
`[sym+0x4c]` **above the low byte** (`0x2000` = bit 13, `0x80000` = bit 19,
`0x200` = bit 9), and the port's `.gl` `ATTR` reader takes **the low byte only**
and says so in its own words: *"A reader that ever needs `ATTR`'s value — not
its bit 6 — must decode the continuation; nothing does today"*
(`crates/c2-il/src/func/gl.rs`, § "`ATTR` is not a byte"). The field is a
two-or-four-byte little-endian value with bit 15 as a continuation flag;
`__declspec(noinline)` takes it from `0x1068` to `0x801028`. **The input to
these two clauses is unreadable by the port**, which is a different and prior
obstacle to `emit-change`'s *"derivable, but it costs an emit"*.

---

## 1. The audited table

`b1` is the cell as published. `b2` is what the second column carries. **`bind`
names which of the two is binding** — i.e. which one a lane would hit first.

| # | b1 (published) | **b2 (this audit)** | bind | evidence |
|---|---|---|---|---|
| **C1** | `writer-unread` | **`no-pass`** | **b2** | The row's own `note`: *"the port has no inline PASS at all; splice/elide are emit-time rules inside lowering."* `DAT_10c40ec4`'s writer being unread is the *smaller* obstacle — reading it names a switch with nothing to switch. `splice.rs` § "Where it may NOT fire" is the port's actual shape. |
| **C2** | `no-instr-count` | **`-`** | — | **CLOSED by this lane.** The count is threaded and the seed is on a production path. No second blocker was found and none is claimed. |
| **C4** | ~~`no-instr-count`~~ → **`no-driver`** | **`no-instr-stream`** | b1 | Three halves, two still absent. The entry **state** is present, live and address-cited (`Expansion::at_pass_entry()`, `splice.rs`, called in production). The **`budget = B` argument** is closed by this lane. What remains is the **driver** `FUN_10b61ee1` — a recursive walk with **fan-out**, cited nowhere under `crates/` — and the **site stream** it walks. `w-clausegen/RESULT.md` §3a and `WB_INSTRCOUNT_FINDINGS` §7 agree; `b1` is re-pointed here because leaving `no-instr-count` on it after this lane would publish something false. |
| **C5** | `no-instr-stream` | **`no-driver`** | b1 | A site collector's only consumer is C4's driver. Building the collector alone lands dead code, and a *second* answer to "where are this body's calls" beside `Selected`/`Terminator` is `docs/GAPS.md` §6's one-fact-two-implementations hazard in the emitter's own predicate. |
| **C6** | `no-instr-stream` | **`emit-change`** | b1 | **Different class from b1, and that is the point.** C6 flags a conditional/EH site into bit 1; the port **refuses** a conditional site (`S1`/`S3`) rather than flagging it. Adopting C6 means widening `S1`/`S3` to admit what they refuse — an emit change needing a two-sided price — *on top of* the stream. A lane costing C6 off `no-instr-stream` alone prices half of it. |
| **C7** | `emit-change` | **`value-refuted`** | **b2** | The ceiling **value** does not exist to be adopted. `WB_INSTRCOUNT_FINDINGS` §6, in the corrected unit: the verdict flips in `count ∈ [261,267]` (static) and `[93,99]` (external); **no `0x10 << k` lands in either window and no single value satisfies both.** `#3732` bans adopting 128 (8 counterexamples each way). So a lane willing to pay `emit-change`'s price still has nothing to write down. |
| **C9** | `emit-change` | **`depends-on-C8`** | **b2** | C9 says the favour-speed bit **skips the size test**. The port's size test is C8, which is `fitted` with blocker `unit-gap` — it tests lowered BYTES where c2 tests a pre-codegen COUNT, on three obj-fitted constants. Skipping a proxy is not modelling c2's skip. Compounded by `WB_INSTRCOUNT_FINDINGS` §2.5: the test C9 gates is **neither necessary nor sufficient**. A third fact, not a blocker but a grading obstacle: `exercised: no` — `/O1` pins the bit, so this corpus cannot grade an adoption either way. |
| **C10** | `emit-change` | **`attr-hi-unread`** | **b2** | The bypass is `test [sym+0x4c],0x2000` — **bit 13**. The port reads the low `ATTR` byte only (`gl.rs`, § "`ATTR` is not a byte"). `w-clausegen/RESULT.md` §3 established the other half: the port already carries a `forceinline` **parameter**, swept over both values in the registered decision surface and cited four times — *what is absent is the reader.* So `emit-change` is not the first obstacle and arguably not an obstacle at all: with the reader, the parameter is already there. |
| **C11** | `emit-change` | **`no-field-mapping`** | **b2** | The masks are on `[sym+0x20]`, a field of c2's in-memory symbol record with **no identified `.gl` origin anywhere in this repo**. The port's `.gl` function-record decode is name / TYPE / offset / SRCPOS / SIZE / ATTR; `+0x20` is none of them. Nothing can be written until the field is located on the container side. `exercised: no` — no workload witness isolates any of the four bits. |
| **C12** | `emit-change` | **`attr-hi-unread`** | **b2** | Same obstacle as C10, two bits over: `0x80000` is bit 19 and `0x200` is bit 9, both outside the low `ATTR` byte the port decodes. `exercised: no`. |
| **C16** | `no-instr-count` | **`-`** | — | **CLOSED by this lane.** Both terms are read — the seed (C2) and the `add` at `0x10b625c1`, which unlike the subtract one instruction above it is **not** gated by the 40 test. No second blocker was found and none is claimed. |
| **C17** | `no-instr-count` | **`no-driver`** | b2 → see §3 | The count is closed by this lane and the budget is threaded through the port's chain walk (`Expansion`), so C17's two operands are both derivable **on the set the port admits**. What is not derivable is C17 at c2's *fan-out*: `[ebp+0x10]` is drained by every site the driver expands, and the port expands one per link. §3 records what was measured and what was adopted. |

**Counts.** `-` on 2 rows (C2, C16 — both closed by this lane). A named second
blocker on **10 of 12**. The second is **binding** on **5** (C1, C7, C9, C10,
C12) — for those five, the published cell names the cheaper obstacle.

## 2. What the column should hold — the proposal, landed

`blocker2`, same vocabulary as `blocker` plus three values this audit needed and
the table did not have:

| value | meaning | rows |
|---|---|---|
| `no-pass` | the port has no inline pass; the clause gates something that does not exist | C1 |
| `no-driver` | no recursive expansion driver with **fan-out**; the port's walk is a chain pinned to one site per link | C4 (b1), C5, C17 |
| `value-refuted` | the clause is read, and the value it would contribute is measured **not** to reproduce c2's boundary | C7 |
| `depends-on-C8` | blocked behind another row of this same table | C9 |
| `attr-hi-unread` | the input is a `[sym+0x4c]` bit above the low byte, and the port decodes the low byte only | C10, C12 |
| `no-field-mapping` | the tested field has no located counterpart in the IL container | C11 |
| `-` | audited, and none found | C2, C16 |

**`no-driver` is deliberately NOT `no-instr-stream`.** They are two absences and
collapsing them is what made `#3816`'s partition read as *"one missing link,
four rows"*: C5 and C6 want the **stream**, C4 and C17 want the **driver** that
walks it. A lane can build either without the other and neither alone converts a
row.

## 3. C17, decided by measurement rather than by argument

C17 is the one row where this lane's own work could have moved the verdict
either way, so it is registered separately and the measurement is in the rung.
Adopting C17 puts a **new refusal on a production path**, and a refusal that
fires changes an emit — so a construct rung may adopt it only if it is measured
not to fire.

`WB_INSTRCOUNT_FINDINGS` §5.2's **first-site theorem** is why this is not a coin
flip: `B ≥ 1000` for every caller, so at an undrained budget C17 cannot decline
any site whose callee counts below 1000, and the caller's own size only scales
`B` **upward**. The port's walk drains the budget one link at a time down a
chain, so C17 can only bind on a chain deep enough (or a link fat enough) to
spend 1,000 count units.

The outcome is recorded in `docs/rungs/2026-08-29-w-budget.md` §5 with the
measured numbers, and this file is not the place it is decided.

## 4. What this audit did NOT do

* **It changed one `blocker` cell**, C4's, and only after reading the port, both
  wave-20 reads and `w-clausegen`'s reconciliation. Every other `blocker` cell
  is untouched. A blocker cell is a verdict.
* It did not re-classify any row's `state` on the strength of a *second*
  blocker. A row with two blockers is exactly as `absent` as a row with one;
  what changes is the price, not the verdict.
* It did not price any of the ten. Naming the second obstacle is not costing
  it, and a count taken off `blocker2` is as unpriced as one taken off
  `blocker`.
