# `#3847` — the multiply-blocked audit of all 12 `absent` rows

> **Lane `w-budget`, wave 21 L1. Deliverable 2.** Board **#3852**.
> Under `work/w-budget/PREREG.md` §5. Table: `work/w-inlmetric/CLAUSES.tsv`.

## 0. The question, and the answer in four lines

`#3847`: **the `blocker` column holds one cell per row, and C4 provably needs
two.** Nobody had checked which *other* rows are multiply blocked, so any count
taken off that column understates the work by an unknown amount.

> **Measured: 10 of the 12 `absent` rows carry a second blocker.** The
> `no-instr-count 4 / no-instr-stream 2 / emit-change 5 / writer-unread 1`
> partition that `#3816` publishes is therefore not a partition of the work; it
> is a partition of the *first reason somebody wrote down*.
>
> The column gets a sibling: **`blocker2`**, audited row by row below and
> landed in the same commit as this file. `-` is a real value and two rows
> carry it.

> **§1 ALSO CLAIMED THE SECOND IS *BINDING* ON FIVE ROWS. THAT COUNT IS
> CORRECTED TO THREE IN §5 AND THE ORIGINAL IS LEFT STANDING IN §1 SO THE
> CORRECTION IS READABLE.** Peer lane `w-emitprice` priced the five
> `emit-change` clauses and two of my five orderings did not survive it: C7's
> ordering is withdrawn (its price is negative *because* the value is refuted —
> one finding, not two ordered ones) and C9's first cell **dissolved**
> (measured byte-neutral, so there is nothing for a second to be binding over).
> **`binding` is an ordering, and 2 of 5 of mine were artifacts of having one
> lane's evidence.** The count of multiply-blocked rows — 10 of 12 — does not
> move. **No cross-row ranking is offered**; see §5.5.

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

---

## 5. RECONCILIATION with `w-emitprice` — and it breaks two of §1's own calls

*Added after the coordinator relayed peer lane `w-emitprice`'s result. **Its
five verdicts are RELAYED, not re-derived here**: its evidence is not in this
tree and this lane did not re-run any of it. Where the two audits meet, the
agreement is independent; where they part, this section says which claim
loses.*

### 5.1 The score

| clause | this audit's `b2` | `w-emitprice` | verdict |
|---|---|---|---|
| **C7** | `value-refuted`, **binding** | genuinely `emit-change`, and **priced NEGATIVE**: adopting `DAT_10c46318 = 128` costs 3 wrong emits at `/O1` against a standing-refusal cost of **0** byte-exact functions | **AGREE on the outcome, and my "binding" ORDERING IS WITHDRAWN** — §5.2 |
| **C9** | `depends-on-C8`, **binding** | **`emit-change` is simply the wrong blocker**: `0x10b8238d` computes `DAT_10c2e310 = (option_word >> 23) & 1` and the port already holds that word (`OPT_WORD_O1 = 0x00200005`, bit 23 clear; `OPT_WORD_OX = 0x00a00005`, set), so the clause is adoptable and byte-neutral | **THEY ARE RIGHT AND MY `b1` DISSOLVED UNDER ME** — §5.3 |
| **C10** | `attr-hi-unread`, **binding** | one of three rows wanting the same absent thing: **the port has no `.gl` symbol-record decoder**, only a fixed-pattern walk reaching one byte | **INDEPENDENT AGREEMENT** — §5.4 |
| **C11** | `no-field-mapping` | same missing link; C11 needs a field read **before the walk's anchor** | **INDEPENDENT AGREEMENT**, and they unify what I split |
| **C12** | `attr-hi-unread`, **binding** | same missing link, bits 9 and 19; **C11 and C12 are ONE PREDICATE** — six interleaved tests, one sink — and C13 converted while C12 did not because of **field width**, not difficulty | **INDEPENDENT AGREEMENT**, plus a fact I did not have |

### 5.2 C7 — the price is negative *because* the value is refuted, so they are one finding and not two ordered ones

§1 called `value-refuted` **binding over** `emit-change`, on the argument that
*"a lane willing to pay the emit price still has nothing to write down."*
**`w-emitprice` wrote it down.** It adopted `128`, measured it, and got 3 wrong
emits at `/O1` — which is not "nothing to write down", it is *the wrong thing,
measured*.

So the ordering was wrong and the two cells are one fact seen twice: the emit
price is negative **because** the value does not reproduce c2's boundary.
`b2 = value-refuted` stays, as the *mechanism* of `b1`'s negative price; the
claim that it is **binding over** `b1` is withdrawn.

Their mechanism also lands inside this lane's own seam and is worth carrying
forward: **the consequence of a size-rule error flips between the port's two
seams.** At `splice.rs`'s S7 (an **accept**) a false inline is a wrong emit; at
`comdat::fenced_inlined_callee` (a **refusal**) a false keep is. `INLINE_DECLINE_BYTES`
is *already* 128 and shows the same 3 error cells — harmless at the fence,
3 wrong emits at the accept seam. Same constant, same cells, opposite verdict.

### 5.3 C9 — my `b1` dissolved, so "binding second" has nothing to be binding over

§1 called `depends-on-C8` binding over `emit-change`. `w-emitprice` measured
that there **is no emit change**: the workload is pinned to `/O1`, bit 23 is
clear, c2's size test runs, and an adoption is byte-neutral by construction —
C15's own argument, one clause over.

`C9.blocker` is therefore re-pointed to **`none`** in `CLAUSES.tsv`, on
`w-emitprice`'s evidence and with the attribution in the row's `note`. **This
lane did not adopt C9 and was told not to**; the coordinator's constraint is
about port code, and leaving a blocker cell the peer lane measured false would
publish something known-wrong in the one file that is supposed to be the single
source (`#3814`).

`b2 = depends-on-C8` is kept and its meaning narrows: with `b1 = none` it is no
longer an obstacle to *making* the adoption but a **semantic caveat on what the
adoption would mean** — the port's size test is C8, which is `fitted` with
`unit-gap`, so "skip the size test" skips a byte-fitted proxy and not c2's
count test. `WB_INSTRCOUNT_FINDINGS` §2.5 sharpens it: the test C9 gates is
neither necessary nor sufficient.

### 5.4 C10 / C11 / C12 — two lanes, one link, reached from opposite sides

This is the reconciliation worth having, because neither lane could see the
other. I read the **container** side — `gl.rs`'s own docstring, *"a reader that
ever needs `ATTR`'s value … must decode the continuation; nothing does today"*
— and concluded the inputs are unreadable. `w-emitprice` read the **image**
side and concluded the same three rows want one absent thing: a `.gl`
symbol-record **decoder**, where the port has a fixed-pattern walk that reaches
one byte.

`attr-hi-unread` and `no-field-mapping` are the two faces of that one link, and
the legend in `CLAUSES.tsv` now says so. **`#3853` stands and widens to three
rows**: C10 and C12 are marked `R1` = *"read and derivable today"* and are not,
and C11 is in the same state for a neighbouring reason.

Two things from their read that this audit did not have and does not claim:
C11 and C12 are **one predicate** with six interleaved tests and one sink, and
C13 converted while C12 did not **because of field width**.

### 5.5 The count §1 published is corrected: 3, not 5

> §1 said *"on FIVE of them the second is the BINDING one."* **Two of those
> five do not survive contact with `w-emitprice`** — C7's ordering is withdrawn
> (§5.2) and C9's `b1` dissolved (§5.3). **The surviving count is THREE: C1,
> C10, C12.**

**And that is the artifact the coordinator told me to look for, firing on my own
instrument.** `#3505` is six for six on lanes that moved a number by
constructing one, and `MEMORY`'s standing note is that ranking instruments
measure themselves at five instances. `binding` is an **ordering**, and 2 of my
5 orderings were artifacts of having only one lane's evidence.

**So this audit publishes a PARTITION and asserts no ranking**, exactly as
`w-emitprice` did when four of its five rows priced identically at zero:

* **which rows carry a second blocker** — 10 of 12, and that number does not
  move under the reconciliation, because every one of the five rows above still
  carries two cells;
* **which second is binding** — claimed for **three** rows only, and each one
  is claimed because a *named, checkable* fact makes the first cell
  unreachable, not because it felt larger:
  * **C1** — reading `DAT_10c40ec4`'s writer names a switch for a pass that
    does not exist (`splice.rs` § "Where it may NOT fire");
  * **C10, C12** — the input is a `[sym+0x4c]` bit above the low byte and the
    reader takes the low byte only, by its own docstring, confirmed
    independently by `w-emitprice`.
* **no cross-row ranking of the ten is offered**, and none should be taken off
  this table. Naming the second obstacle is not costing it (§4).
