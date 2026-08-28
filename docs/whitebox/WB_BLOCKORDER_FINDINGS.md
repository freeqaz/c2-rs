# WB_BLOCKORDER — FINDINGS and PREREG score, read R8

**Lane** `w-read-r8` · characterization lane · **Fixtures:** none ·
**Census:** +0 · **reach:** 0, as predicted · **`crates/` bytes changed: 0**.

**Prereg:** [`WB_BLOCKORDER_PREREG.md`](WB_BLOCKORDER_PREREG.md), committed
`cf2f0509b` as the **first commit on the branch, before a byte of any target
was read**. **Spec page:** [`ref/P_BLOCKORDER.md`](ref/P_BLOCKORDER.md).
**Board:** **#3437**–**#3441**.

**Outcome: `instrument`** — the rule was **found**, not declined and not
bounded. §4's decline criterion was not reached; §7 records what it would have
taken.

---

## 0. The headline, in one line each

1. **c2 has no block-ordering pass.** The emit walk `FUN_10b338f5` follows
   `tuple+0` and does nothing else — no sort, no comparator, no ordering key.
   Emission order is list order.
2. **`M1`/`M2` are not rival rules.** They are two traversals inside the
   `switch` lowering, whose address nobody in this repo had ever found.
3. **`M2` is descending case VALUE, not "reverse source order"** — and every
   published statement of it in this repo says source. Five documents are
   wrong on the same point and are corrected beside, not over.
4. **There are three switch lowerings, not two.** The CTR ladder at 8–9 dense
   cases has never been named here.
5. **Both thresholds are read constants**, `8` at `0x10bd1388` and `range > 9`
   at `0x10c1dc7b`, and the obj grid lands on both from the other side.
6. **The closure argument that worked for R3 fails here, and the failure is the
   finding**: splice direction is a runtime parameter passed by pointer at 707
   sites.
7. **c2 emits HYBRID switches** — one dense cluster as a jump table, the
   outliers as a decision tree, in one function. 1 cell in 240, absent from the
   record, and explained by an already-read constant rather than a fitted one.
8. **`FUN_10bd415e` places a label**, which is what `P_LABEL.md` §8 open #1
   asked for: a minted label symbol becomes a kind-`0x1b`/op-`0x308` tuple, and
   it lands exactly where that tuple was spliced.

---

## 1. PREREG score, verbatim

`H` hit · `M` miss · `U` unscoreable.

### 1.1 About the brief's addresses (prereg §3.1)

| # | prediction | p | verdict | note |
|---|---|---:|---|---|
| **P1.1** | `FUN_10b36133` contains no block-ordering logic; it is an anchor | 0.85 | **H on the graded claim, M on the parenthetical** | It is a 54-byte **opcode classifier** on `*(p+4)` against `0x30d`/`0x30f`/`0x313`/`0x317`. The prereg guessed "a label/symbol construction wrapper" — wrong; its one callee is the ICE `FUN_10b33526`. Scored the way `WB_REGALLOC_FINDINGS.md` §7.8 scored its `wbr_glob3` row |
| **P1.2** | `FUN_10b34a89` is on the block-order path, part of the **tail merger** | 0.55 | **M** | It is an **arithmetic identity/exactness predicate** over opcodes `0x2c7`, `0x2c9`, `0x2ca`, `0x2cb`, `0x2cf` — shifts, divides, remainders, returning 0/1. Not tail merging, not block order. §2 carries the correction this forces on `WB_REGALLOC_FINDINGS.md:33` |
| **P1.3** | `FUN_10b968b0` is not on the block-order path at all | 0.75 | **H** | A 507-byte decorated-name suffix builder over `%s$%s$%d%s` / `$%s$%d%s`, splitting at `@` |
| **P1.4** | At least one of the three seeds is a dead lead **and the lane says which** | 0.90 | **H** | **All three** are dead leads for block order |

### 1.2 About the mechanism (prereg §3.2)

| # | prediction | p | verdict | note |
|---|---|---:|---|---|
| **P2.1** | **H-LIST**: the emit walk performs no sort and no comparison on an ordering key; it follows a linked list. Hit requires naming the walk's address and its `next` offset | 0.55 | **H** | `FUN_10b338f5` @ `0x10b338f5`; advance `tuple = *(tuple+0)` at **`0x10b33a21`** (`8b 36`, two bytes) |
| **P2.2** | **The payload clause** — at least one list-construction site read to **direction**, with address and field offsets, or P2.1 is scored M regardless | 0.45 | **H, five times over** | `0x10bd3815` AFTER, `0x10bd3824` BEFORE, `0x10bd3835` CHAIN-AFTER, `0x10bd3852` UNLINK, `0x10bd38d0` MOVE-RANGE, all with bodies |
| **P2.3** | The `M2` reversal is produced at **construction**, not at emission | 0.45 | **H** | It is produced by the decision-tree traversal in the switch lowering; emission has no opinion |
| **P2.4** | Block order has **more than one author** | 0.70 | **H** | The scheduler (×4/function via `FUN_10be626c`), two `factor.c` routines, and 707 inserter-pointer sites. §3 |
| **P2.5** | No edge-weight or profile-driven layout heuristic at `/O1` | 0.75 | **H, and strengthened** | There is no layout pass at all to carry one |

### 1.3 About the obj-side rule (prereg §3.3)

| # | prediction | p | verdict | note |
|---|---|---:|---|---|
| **P3.1** | #1906 replicates at scale: jump tables monotone in case index, decision trees not | 0.70 | **H** | Replicates on the grid **and on a 240-cell randomized corpus, 239/240**. #1906's *wording* is corrected all the same (§2 row 1) |
| **P3.2** | The tree/table threshold is a readable constant, and the lane names or bounds it | 0.40 | **H, and it is two constants** | `8` at `0x10bd1388` off the record at `0x10b2418c`; `range > 9` at `0x10c1dc7b` |
| **P3.3** | The corpus contains a switch shape neither #1906's cells nor this grid predicted | 0.65 | **H — and it is exactly one cell in 240** | A 240-cell randomized corpus was built and run after all. `sw_rc065` is a **HYBRID**: c2 partitioned 19 clustered values and gave the dense low cluster a **jump table** and the seven outliers a **decision tree**. No prior cell in the record is a hybrid. §5.1a |
| **P3.4** | `M2`'s `default`-first placement is a separate fact from the reversal | 0.50 | **H** | Tree → default first; ladder and table → default last. Independent of the arm rule |

### 1.4 The self-test that would have voided the lane (prereg §3.4)

Registered: the extractor must reproduce `WB_REGALLOC_FINDINGS.md` §7.6 —
seven leaves of a six-case switch as `default, 66, 55, 44, 33, 22, 11` — or
the lane grades nothing.

**GREEN.** `sw_dense06` emits `997, 281, 163, 307, 149, 211, 137`: default
first, then the six arms in reverse case order, three `cmplwi cr6` compares,
`li`/`blr` leaves, 92 bytes. Structurally identical to §7.6's cell.
`sw_dense03` likewise gives `997, 149, 211, 137`.

**Score: 14 H · 2 M · 1 partial · 0 U, over 16 registered rows.**

---

## 2. Corrections this lane forces on the record

Amended **beside**, never over (`ref/README.md` §2.1). Every one is a
disassembly- or obj-backed correction of a claim currently published.

| # | document | says | correction |
|---|---|---|---|
| 1 | board **#1906**, `WB_LOOP_FINDINGS.md:449`, `WB_REGALLOC_FINDINGS.md:541`, `ref/P_DAG.md:259`, `CFG_SHAPE.md:1757` | decision-tree switches emit arms in **reverse *source* order** | **Descending case VALUE.** Source order does not enter the tree path at all. Proven by the `scram` family, where the two disagree, and independently by `sw_spscram12` ≡ `sw_sparse12` |
| 2 | `WB_REGALLOC_FINDINGS.md:33` (TU table) | `factor.c` `10b34a89` = *"tail merging (a block-level reorder)"* | `FUN_10b34a89` is an **arithmetic identity/exactness predicate**. The TU attribution (`factor.c`, `in-anchor`) is right; the *description of the function* is wrong. Whether `factor.c` as a **file** does tail merging is untouched by this |
| 3 | the whole record | there are **two** switch lowerings | **Three.** The CTR ladder (`mtctr` + `bdz` chain, no index table) at 8–9 dense cases is unnamed anywhere |
| 4 | `CFG_SHAPE.md:888-891`, `:1757` | the table threshold is **unmeasured** | Measured **and read**: tree below 8, ladder at 8–9, table at ≥10 dense, from `0x10bd1388` and `0x10c1dc7b` |
| 5 | `WB_REGALLOC_FINDINGS.md:708` | the switch lowering is *"an unread algorithm"* | Read. Entry `0x10bd22a7`; decider `0x10bd1373`; recursive driver `0x10bd1f1a`; table builder `0x10bd1c85`; ladder `0x10c1de62`. `ref/P_BLOCKORDER.md` §4 |
| 6 | `WB_REGALLOC_FINDINGS.md` §7.6 | the switch value is compared unsigned *"even though the C type is `int`"*, unexplained | The type constant `0x2004` is **hard-coded** in `FUN_10c1de62`; `FUN_10bd1c85` forms `… & 0xfff \| 0x2000`. The source type is never consulted |
| 7 | the dispatch brief and `READ_PLAN` §3 row R8 | *"`0x10b968b0` (the label format strings)"* | It is a **function**, `FUN_10b968b0`, 507 B, that *uses* those strings. Recorded in the prereg §0.1 before reading |
| 8 | the dispatch brief | *"start at `fg.c` `0x10b36133`…"* reads as three candidate sites | All three are **TU anchors** from `c2map3`'s *"Found and not taken"*. Recorded in the prereg §0.1 before reading |

---

## 3. The closure question, answered in the negative and on purpose

R3's closure argument — the allocator's VA occurs zero times as data, therefore
its direct calls are all of them — is the strongest move in this directory. It
was applied here and it **fails**:

| primitive | direct calls | address-takes |
|---|---:|---:|
| `0x10bd3815` INSERT AFTER | 131 | 201 |
| `0x10bd3824` INSERT BEFORE | 207 | 506 |
| `0x10bd3835` SPLICE CHAIN | 90 | 77 |

`docs/whitebox/scripts/dump_tuple_splice.py`, measured from the image and
cross-checked against the objdump — both agree to the digit. The takes are
`push imm32` / `mov r32, imm32` feeding builders that accept an **inserter
argument**; 75 functions load both.

**So this page states no closure and claims no sole author.** The prereg (§5.6)
registered that the most likely way this lane would be wrong is *"finding one
list-writer and reporting it as the author"*, and the instrument was built to
make that impossible to do accidentally.

---

## 4. What the controls could NOT catch — re-asked after the fact

The prereg listed six. Scoring them honestly:

1. **"A read of the emitter cannot see an earlier pass that permuted the
   list."** **Fired exactly as written, and it is why §6 open #1 is open.** The
   emit walk was read completely and correctly and says almost nothing about
   order.
2. **"A corpus scan cannot see block identity."** **Fired, in the grid.** Every
   arm is a `return CONST` leaf, so this lane cannot tell a *moved block* from a
   *materialized leaf* — `P_BLOCKORDER.md` §6 open #1.
3. **"Neither grid nor corpus separates list order from a pass reproducing it."**
   Held; the read is what separated them.
4. **"Fixed flags."** Two modes compiled, byte-identical; `/O2` and POGO
   untested, and the size-mode threshold record at `0x10b2417c` is read but
   never exercised.
5. **`[R]` is a hypothesis.** §1 and §2 of the page are `[R]`; §4.1's thresholds
   and §5's rule are `[O]`.
6. **"Reporting one writer as the author."** Prevented — §3.

**And one the prereg did not anticipate:** *the shape classifier itself was
wrong twice.* Keying on `mtctr` called a CTR ladder a jump table; the repaired
test matched `bdzf` but not plain `bdz`, mis-filing `sw_scram08`. **Neither
showed up as a failing run** — both produced clean, plausible tables. They were
caught only by reading the emitted words. Sixth entry in this repo's
*"ranking instruments measure themselves"* pattern, and the first where the
instrument was a *classifier* rather than a ranking.

---

## 5. The confirmation probe, and the fact that it went red first

The prereg required a probe *capable of failing*. It failed, and the failure is
the most load-bearing evidence on this page.

A **one-level** pivot model was fitted to three main-grid cells and matched them
exactly. It was frozen as a predictor over six fresh holdout cells
(n = 9, 10, 11, 14, 16, 20; value sets disjoint in shape from the fitting set;
every cell written in scrambled source order), committed, and only then
compiled.

**Result: 4 HIT / 2 MISS** — missing at n = 16 and n = 20, i.e. exactly where a
group large enough to split *again* first appears. A model that had only ever
met n ≤ 14 would have shipped looking correct.

The corrected rule is **recursive**, and its bottom-out constant is the same
`8` read at `0x10bd1388`:

```
emit(V):                          # V = case values, ascending
    n = |V|
    if n < 8:  return reverse(V)
    p = n // 2
    return emit(V[:p]) ++ [V[p]] ++ emit(V[p+1:])
```

**22 HIT / 0 MISS** over every decision-tree cell in both grids.

### 5.1a The randomized corpus, and the one cell that beat the rule

Built after the grid, on the prereg's standing ground that *grids and corpora
fail in opposite directions*: **240 cells**, seed-deterministic, with case count
(2–24), value set, density (dense / sparse / clustered / wide) and **source
order** randomized independently. Ground truth comes free from the generator, so
no source parsing is needed — which is what makes this a corpus rather than a
bigger grid.

**239 HIT / 1 MISS.** Breakdown: decision-tree 196/196, jump-table 38/39,
CTR-ladder 5/5.

An extractor honesty note that is not a detail: a marker constant too wide for a
`li r3,imm` field becomes `lis`/`ori` and the arm order becomes unreadable. The
**first** run of this corpus had markers up to 60,000 and reported **230 of 240
cells UNREADABLE** rather than scoring them — which is the behaviour that makes
the remaining 10 worth anything. The marker range was narrowed and the corpus
regenerated; the rule under test was already committed and published, so this is
an out-of-sample test of a fixed rule, not a search over corpora.

**The miss, `sw_rc065`** (19 values, clustered):

```
emitted, as values:  36 37 52 27 26 53 32 38 28 29 33 31 | 119 | 175 174 173 122 121 120
                     `----------- SOURCE order -----------'   `---- the tree rule ----'
                        12 low values, as a JUMP TABLE          pivot 119, then reverse
```

c2 **partitioned the case set and lowered the parts differently** — a hybrid,
and the record has never described one. §5's rule holds on each part; what it
omits is the partition, and the partition constant is **already read**:
`FUN_10bd1801` @ `0x10bd1801` uses the **max-gap `3`** at `[0x10b24184]`, loaded
at `0x10bd1844`, and the cluster boundaries are exactly this cell's gaps > 3
(38→52, 53→119, 122→173).

**So the rule's honest scope is per contiguous cluster**, and `ref/P_BLOCKORDER.md`
§5 now says so. The miss cost nothing to explain because the read had already
supplied the constant — which is the second time on this lane that read-first
paid at the moment the objs disagreed.

> **Would the probe have gone red if the claim were false in the most likely
> way?** The most likely falsehood was *"reverse case order" is really reverse
> **source** order* — nobody had separated them. The `scram` family separates
> them by construction and the objs chose value. The second most likely was a
> rule fitted to small n; the holdout caught that one for real.

---

## 6. What was NOT done, stated so absence does not read as coverage

* **The corpus is randomized, not the dc3 workload.** The prereg registered a
  workload-scale extractor; what was built instead is a 240-cell randomized
  corpus (§5.1a). The trade is deliberate — the workload has real switches but
  no ground truth about their case **values** without parsing dc3 sources, and
  the rule is stated over values. **A workload scan would still add something
  this cannot: the real distribution of switch shapes.** Unclaimed follow-up.
* **The cluster partition is named, not modelled.** §5.1a. One cell in 240.
* ~~**`FUN_10b35c78`** is unread; it is the standing candidate for
  `P_BLOCKORDER.md` §6 open #1.~~ — **READ and ELIMINATED 2026-08-28 (`w-s7`,
  board #3737).** It is a genuine move (unlink + insert-after at every kind-`0x1b`
  label, draining `tuple+0x2c`), **and it did not run on this lane's grid**:
  both its callers are inside `0x10b7e032`'s `sym+0x20 & 0x1000` gate, clear on
  2,946 of 2,946 functions at `/O1 /EHsc`. Open #1 keeps its question and loses
  its candidate. [`WB_S7_FINDINGS.md`](WB_S7_FINDINGS.md) §2, §4.
* **No call-bearing-arm grid**, which is what would separate a block *move*
  from a leaf *materialization*.
* **No `/Os` cell**, so the size-mode threshold record is read but unexercised.
* **No new `if`/loop cells.** `CFG_SHAPE.md` §3.4, #2352 and #1906 are cited,
  not re-measured.

## 7. What a decline would have cost, recorded although it was not taken

The prereg's decline criterion (§4) was 30 bodies / 3 days, and the deliverable
either way was an elimination list, a continuation price, and a next place to
look. **The read cleared its own criterion inside ~15 function bodies**, because
the seeds were eliminated in three reads and the emit walk was four call-graph
hops from the encoder that R2 had already addressed. The single highest-leverage
move was not any address in the brief — it was **`P_DAG.md:113`'s one-line note
that `0x10be626c` re-links the tuple list `(tuple+0 next, +0x10 prev)`**, which
named the data structure and turned the question from "find the ordering pass"
into "find the list". Read-before-probe worked here in the strong sense: the
grid was designed *after* the read, and it was the read that told the grid which
control to carry.

---

## 8. Cross-references to live peer lanes — cited, not read

* **`w-read-r6`** (final-expansion switches). `FUN_10c0d57e` is one of the 75
  functions that load **both** inserters (3 AFTER, 12 BEFORE) and is a
  label-constructor caller in R3's census. It is R6's target and **was not read
  here**. If R6 finds the final expansion re-splices arm tuples, that bears
  directly on `P_BLOCKORDER.md` §6 open #1.
* **`w-read-r7`** (scheduler). The scheduler re-links the list four times per
  function through `FUN_10be626c` and is therefore a first-class author of
  block order. Not read here.
* **`w-read-r9`** (`0x4F` sub-record). No overlap found.
* **`w-s1bc`** is the only lane in `crates/`. **This lane changed zero
  `crates/` bytes**, and its one `crates/` implication — that a port's block
  emitter needs a splice-ordered list and an inserter parameter, not a sort — is
  written here as a finding for a follow-up lane, not applied.
