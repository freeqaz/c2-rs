# w-label — pre-registration

Lane **w-label** (`wt-w-label`), 2026-08-04, branched at master `97343a7`.

**Committed before the first `crates/` edit.** Everything below was measured in
this worktree with the real toolchain (`cl.exe` 16.00.11886.00 / `c2.dll` under
wibo `1.0.1-23-g4a9dd6f`) or read off the port's own source. Where I reproduce
another lane's number I say how I re-derived it, and where I disagree with one I
say so rather than reconciling.

Provenance: dc3-decomp HEAD snapshot taken before the measurements (recorded in
`work/w-label/dc3_head_before.txt`); wibo `1.0.1-23-g4a9dd6f`.

---

## 0. What this lane was commissioned to do, and what it found first

The brief says: *"do not pick a TU, pick the construct"*, and names the
construct — **a real label→offset map**, which w-conv's construct ranking says
**14 of 19** FRONTIER TUs want. It also asks one question directly:

> the exit-value merge charges a **sixth label slot**, because c2 branches
> **backwards** into the first arm. w-conv declined it deliberately — *"the
> refusal is also the counter boundary."* That means the label counter and the
> codegen class fail at the same place, which is either a coincidence worth
> understanding or the shape of the real rule. **Find out which.**

§1 answers that question with 24 seed-free in-TU cells. §2 counts what the map
refuses. §3 registers the estimate. **§1 was measured before a line of `crates/`
was written and it is what the rest of this document is drawn from.**

### 0.1 Two things in the brief that are wrong, corrected up front

* **`work/w-conv/frontier_dis.txt` does not exist.** The brief names it as
  required reading. w-conv's rung §11 gives the reproduction (`refobj.sh` +
  `gt_dump.py` per TU) and its PREREG §1 carries the hand-count; the dump file
  itself was not committed. Nothing downstream depends on it — the count is in
  the prose — but a lane told to read it will not find it.
* **w-conv already built a label→offset map, and the brief reads as though none
  exists.** Board row **Z-c** (`docs/rungs/2026-08-04-w-conv.md`) is *"The port
  emits an intra-section `b` and resolves a real label→offset map — DONE"*. What
  is actually in `crates/c2-core/src/codegen/calls.rs:450–565` is **a fixup list
  with one implicit target**: `early_fixups: Vec<(usize, Option<usize>, u8, u8)>`,
  every entry resolved against `epi_start`. `calls.rs:519` says so in its own
  words — *"there is no fixup list and no label map"* — four lines above the
  block that resolves one. So the brief's premise ("build it") is right and
  Z-c's claim is stronger than the code. This lane builds the construct §6.2
  item B specifies and does not restate Z-c.

---

## 1. The question, answered: **the boundary is real and the coincidence is
partial**

### 1.1 The instrument

`work/w-label/cflabels.py` — `scripts/gt_label_stride.py`'s seed-free
construction (three in-TU anchors; every number a difference inside one obj, so
the `.gl` seed, the mangled-name lengths and the `/Gy` per-function surcharge all
cancel) with a probe list that varies **only the control-flow shape** and holds
the function class fixed at framed Class A with one call to `gp`. It **imports**
the shipped instrument rather than copying it, so the anchor control, the group
walker and the `minted` counter are the same code that produced
`LABEL_COUNTER.md` §1. `scripts/` is lane w-shapes' seam and was not edited.

`work/w-label/cftargets.py` reads the same objs' bytes back and counts
intra-section branch targets (discriminated by **the absence of a relocation**,
not by the opcode — `CFG_SHAPE.md` §3.3). `work/w-label/cfdis.py` prints one
probe's `.text` with every target's predecessor count.

**Controls: 0 failed on all 24 rows.** The anchor base came back 5 (`/Gy`) in
every obj, measured rather than assumed.

### 1.2 The table (`/O1 /GS- /c` — the workload's own mode)

`sur` is the stride less the base less `LABEL_COUNTER.md` §1.1's *minted*
surcharges (`minted - 5`), so a row that pays for a `__savegprlr_N` pair is not
credited with it. `int` = distinct interior branch targets; `join` = those of
them with ≥ 2 predecessors; `back` = those reached by a backward branch.

| probe | stride | minted | **sur** | int | join | back |
|---|---:|---:|---:|---:|---:|---:|
| `cf-none` framed, no control flow | 5 | 5 | **+0** | 0 | 0 | 0 |
| `cf-if2` two guards, distinct literals | 5 | 5 | **+0** | 2 | 0 | 0 |
| `cf-if3` three guards, distinct literals | 5 | 5 | **+0** | 3 | 0 | 0 |
| `cf-ifelse-val` if/else, both arms return a literal | 5 | 5 | **+0** | 0 | 0 | 0 |
| `cf-goto-fwd` explicit forward `goto` | 5 | 5 | **+0** | 1 | **1** | 0 |
| `cf-ifelse` if/else with a join block | 6 | 5 | **+1** | 1 | 1 | 0 |
| `cf-merge-tail` guard returning the sequence's own literal | 6 | 5 | **+1** | 2 | 1 | 0 |
| `cf-merge2` two guards, **same** literal | 6 | 5 | **+1** | 2 | 1 | **1** |
| `cf-merge3` three guards, one literal | 6 | 5 | **+1** | 2 | 1 | **1** |
| `cf-merge-mixed` literals 5/11/5 | 6 | 5 | **+1** | 3 | 1 | **1** |
| `cf-goto-back` explicit backward `goto` | 6 | 5 | **+1** | 1 | 1 | **1** |
| `cf-dowhile` | 6 | 5 | **+1** | 1 | 1 | **1** |
| `cf-while` | 7 | 5 | **+2** | 2 | 2 | **1** |
| `cf-for` | 9 | 7 | **+2** | 2 | 2 | **1** |
| `cf-for-continue` | 9 | 7 | **+2** | 3 | 3 | **1** |
| `cf-forever` `for(;;)` + `break` | 8 | 5 | **+3** | 1 | 1 | **1** |
| `cf-for-break` | 10 | 7 | **+3** | 2 | 2 | **1** |
| `cf-dowhile2` two sequential do/whiles | 8 | 5 | **+3** | 2 | 2 | **2** |
| `cf-for2` two sequential `for`s | 11 | 7 | **+4** | 4 | 4 | **2** |
| `cf-fornest` nested `for` | 11 | 7 | **+4** | 3 | 3 | **2** |

Four further rows (`cf-if`, `cf-void-guard`, `cf-switch-dense`,
`cf-switch-sparse`) came back **stride 1**: c2 made those bodies *leaves*, so
they are a different class and are excluded from the surcharge column rather than
averaged in. That they are leaves is itself worth recording — an 8-arm dense
`switch` with a call in every arm is a leaf at `/O1` and emits **no jump table**,
which is the same reading `LABEL_COUNTER.md` §4 has and the same caveat.

### 1.3 Two candidate rules, both tested against the whole table, **both refuted**

Stated and killed here rather than shipped, because a table fitted by all its
cells and tested by none is exactly `CFG_SHAPE.md` §3.5's declined fold model.

* **"+1 per distinct interior branch target"** — **15 of 24 rows miss.**
  `cf-if3` has three interior targets and costs 0.
* **"+1 per distinct interior JOIN (≥ 2 predecessors)"** — **6 of 24 miss**, and
  one of the six (`cf-goto-fwd`: one interior join, cost **0**) misses **across
  the zero boundary**, which is the only direction that matters for a port that
  must decide accept-or-refuse.

### 1.4 What survives every cell

> **Every body containing a BACKWARD intra-section branch charges ≥ +1 — 11 of
> 11. No body without one charges more than +1 — 13 of 13.**

That is the whole of what the 24 cells support, and it is enough to decide the
rung. Inside the forward-only class the charge is 0 on four cells and +1 on two,
and **both of the two are `CFG_SHAPE.md` §3.4.1's code-motion shapes** — a block
c2 created by tail-merging two paths (`cf-ifelse`'s shared `bl gp`,
`cf-merge-tail`'s shared `li r3,0`), which the port already refuses for an
unrelated reason (§3.4.1: *"a body whose arms end in the same call is out of
class"*).

### 1.5 The answer

**Not a coincidence, and not the rule either: the two boundaries coincide by
INCLUSION, not by identity, and the inclusion runs in the port's favour.**

* The port's accepted class sits **strictly inside** the counter's zero-cost set.
  So W11's refusal of the exit-value merge and the counter's +1 are the same
  fact seen twice, and w-conv was right to read one off the other.
* But the counter's zero set is **larger** than the codegen class —
  `cf-goto-fwd` costs the counter nothing and the port refuses it — so *"the
  counter charges"* cannot be used as a proxy for *"the port must refuse"*.
* And the zero set is **not closed under the codegen class's own widening
  direction**: `cf-merge-tail` is one literal away from an in-class body
  (`if(a) return 0; … return 0;` against `if(a) return 5; … return 0;`) and
  costs +1. A lane widening the early-return class by relaxing which literals an
  arm may carry walks out of the zero set without changing anything it would
  think of as control flow.

**The operational consequence, and it is this lane's licence:** a rung may widen
the codegen class **inside forward-only control flow** without touching `coff/`
at all — 13 cells license that — and may **not** assume the converse. The moment
a lowering emits a backward intra-section branch it owes `plan_labels` a
surcharge that is *measured but not modelled* (four distinct magnitudes over 11
cells: 1, 2, 3, 4), and getting it wrong is six wrong bytes in the symbol table.

**So the assertion this rung ships is: the label→offset map REFUSES a backward
reference, by name, and says why.** That is the "portable assertion per ordering
rule" the brief asks for, and it is the one that ties `codegen/` to `coff/`.

### 1.6 Two disagreements with the record, stated not reconciled

* **w-conv §4 reads "label lead 2" for the in-class cells and "lead 3" for the
  merged ones. I measure `extra` 0 / `stride` 5 and `extra` 1 / `stride` 6.**
  The *delta* — **+1 for the merge** — reproduces exactly and is the load-bearing
  half; the absolute does not. My anchors are three plain Class-A framed
  functions in the same TU with the base measured per obj, so `extra 0` is
  `first(P) - first(a0) - base` with `base` read from `first(a2) - first(a1)`. I
  do not know which quantity w-conv's "lead" names and have not tried to guess;
  both documents' deltas agree and that is what the port consumes.
* **`LABEL_COUNTER.md` §4 records `for` at +2 and nested `for` at +4.** Raw, this
  worktree reads **+4** and **+6**. The difference is entirely the
  `__savegprlr_29`/`__restgprlr_29` pair those loop bodies oblige, which §1.1
  charges +2 and which is visible as `minted 7` against `minted 5`. With it
  removed the numbers are §4's exactly. **§4's rows are right and are easy to
  re-derive wrong**; the `minted` column is what keeps them honest and this lane
  found that out by getting +4 first.

---

## 2. What the map refuses, counted as INDEPENDENT refusals

The unit of the question is: *what stands between "the label→offset map exists"
and "a FRONTIER TU converts"?* Per the project rule — **if one quantity governs
several boundaries, that is one refusal** — each row answers *"what varies
between these refusals?"*, and where the answer is "nothing", the rows are
collapsed and the collapse is stated.

| # | refusal | what varies between this and its neighbours |
|---:|---|---|
| 1 | **a backward reference** | the **sign of the displacement**. It is one variable and it is also a `coff/` quantity (§1.4: ≥ +1 on the counter, 11 of 11), so closing it is two files, not one |
| 2 | **an interior block with ≥ 2 predecessors that c2 created by code motion** | the **predecessor count** of a block, not the sign of anything reaching it. `cf-merge-tail` is forward-only and still charges, so #1 does not imply it and it does not imply #1 |
| 3 | **the long-branch expansion** (`CFG_SHAPE.md` §3.3.1 / §6.2 item D) | the **body's length**, not its shape. Independent of every other row: a 33 KB straight-line body needs it and no frontier TU is one |
| 4 | **a value with a register home live across a block boundary** (§6.2 item F) | whether a **non-formal value survives a transfer**. The allocator is *"demonstrably richer than a descending counter and not characterized"* (`CODEGEN_W6_COMPARE.md` §6) |
| 5 | **a branch on `cr0`** (§6.2 item E; board **X-e**) | **which unit set the condition** — a record-form instruction or a compare of a call result — not where the target is. 10 of the 19 FRONTIER TUs |
| 6 | **the entry-block park** (w-conv row Z-g) | **which register a formal is displaced into**, a question the map cannot ask |

**Six, and they do not collapse.** Rows 1 and 2 are the pair a partition could
argue about — both surface as *"+1 on the label counter"* — but the **quantity**
differs (displacement sign vs predecessor count) and §1.2 has a cell that
separates them in each direction, so they are two. This is precisely w-conv's
warning that *"the partition is not unique, which matters if anyone tries to
close them one at a time"*: **a lane closing #1 alone still emits a wrong `$M` on
#2's shape**, and both must be closed before `plan_labels` is right about either.
Recorded here so that is not rediscovered.

**The standing decline clause — *a target at ≥ 4 independent refusals is not a
target* — fires.** The label→offset map is **one** of six, and w-conv already
measured that the cheapest FRONTIER TU is 6 independent refusals deep with the
cheapest framed-and-branching one at 9. **This rung converts no TU and is
registered as converting none.**

---

## 3. The estimate — registered before implementation

### 3.1 In the unit of TU match (the payoff metric)

| | |
|---|---|
| **unit** | **TU match, of 878** — whole objs byte-exact at the workload's real flags |
| **point estimate** | **8** |
| **interval** | **[8, 8]** |
| **bias direction** | **high** — if this is wrong it is because I over-estimated. Six lanes have attacked the frontier and converted zero, and every discount applied to a refusal count on this project has been wrong (five confirmations) |
| **the decline clause keys on** | **the interval.** A construct that closes 1 of 6 independent refusals cannot move a per-TU gate, and the interval is degenerate on purpose rather than as a hedge |

The interval is degenerate because the argument for it is §2's count, not
caution. If it were wrong it would be wrong by the count being wrong, and the
count is published so it can be checked.

### 3.2 Separately, at construct level — **not** in the unit of TU match

Registered apart because *a per-function change cannot move a per-TU gate*, and a
lane once scored its only miss by conflating the two.

| quantity | registered |
|---|---|
| new **emitted** shapes | **0**. The map is a refactor of an existing byte-exact lowering plus refusals; every byte the port emits after it must be a byte it emitted before |
| per-function census | **+0** |
| emitted-function census | **+0** |
| `codegen-gap` | **0** (census/gate agreement holds across the change) |
| `vocab-gap` | **863**, unmoved — this rung touches no decode |
| gate fixture-verdicts | **unchanged** — this lane adds no fixture (`fixtures/cpp/` is lane w-shapes' seam and was not edited) |
| `#[test]` count | **rises** — the map's invariants are portable unit tests |

### 3.3 Predictions, to be scored in the rung

| # | prediction |
|---|---|
| **L1** | TU match **8**, mismatch **0**, `codegen-gap` **0**, `vocab-gap` **863**, `capture-fail` **7**, FRONTIER **19** |
| **L2** | **Every byte is unchanged.** `gate.sh` comes back with the same fixture-verdict count and the same per-lane match counts as the baseline, because the map replaces a lowering that is already byte-exact. A single changed byte is a defect, not a widening |
| **L3** | The **backward-reference refusal never fires on any fixture**, at any of the 12 lanes — the port emits no backward branch today and the assertion is a guard on the future, not a live path. It is therefore a **first witness with no GRADED coverage** and this rung says so under **F-c** rather than letting a unit test read as oracle coverage |
| **L4** | The `#[test]` count rises and reconciles against the runner at both ends |
| **L5** | `cargo build --release` stays at **0 warnings** |
| **L6** | **Refuted-rival:** the "interior join" rule of §1.3 will **not** be rescued by any held-out forward-only cell — i.e. no new forward-only probe will be found that has an interior join and costs 0 *other than* `cf-goto-fwd`'s shape. Registered because a single counterexample is a weak base for §1.4's boundary and I would rather be scored on it |

### 3.4 Decline clauses

1. **A target at ≥ 4 independent refusals is not a target.** Fired in §2 on the
   whole FRONTIER, before any implementation, for the second time in two lanes.
2. **A layout or allocation decision with fewer than three witnesses is refused,
   never fitted.** §1.3 kills two candidate rules on this clause.
3. **A disagreement ships the refusal, not a third layout.** If the map cannot
   reproduce `calls.rs`'s present bytes exactly, the map is wrong and is
   reverted — the incumbent is oracle-graded at 12 lanes and the map is not.
4. **A quantity that is measured but not modelled is refused with the
   measurement attached**, never interpolated. §1.4's four magnitudes are
   published and not fitted.

---

## 4. What this lane will not do

* **No `coff/` edit.** §1.5's licence is explicitly *"inside forward-only control
  flow, without touching `coff/`"*, and the counter is where both six-wrong-bytes
  defects came from. The label counter is read, measured and asserted against —
  not changed.
* **No `fixtures/cpp/` or `scripts/` edit** — w-shapes' and w-modes' seams.
* **No `BOARD.md` / `STATUS.md` / `ROADMAP.md` edit.** Rows proposed, lettered.
* **No new emitted shape.** The exit-value merge stays refused, now with the
  counter measurement behind the refusal instead of beside it.
* **No long-branch expansion.** §2 row 3; no fixture body is 32 KB, so it has no
  bytes for the oracle to compare and building it would be ungraded by
  construction.
