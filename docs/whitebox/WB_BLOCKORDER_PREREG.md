# WB_BLOCKORDER — PREREG for read R8 (block emission order)

> **PROVENANCE — DISASSEMBLY-DERIVED.** Every address below is an absolute VA
> in `compilers/X360/16.00.11886.00/c2.dll`. See
> [`DISCLOSURE.md`](DISCLOSURE.md); nothing here may enter `crates/` without a
> row there naming the address it came from. Whitebox analysis is authorized
> and encouraged (`CLAUDE.md`, project owner, 2026-08-17).

**Lane:** `w-read-r8` · **kind:** characterization lane
(`docs/rungs/README.md` § "Lane kinds" 3) · **Fixtures:** none ·
**Census:** +0 · **predicted reach:** 0, registered · **`crates/` bytes: 0**,
registered as a fence, not an expectation.

**Subject.** Read **R8** of the read plan
(`docs/whitebox/READ_PLAN_2026-08-21.md` §3 row R8; funded by the owner
2026-08-23 — `docs/DECISIONS_2026-08-22.md` decision 7; board **#3423**;
rows **#3437**–**#3441**). The deliverable in the plan's own words is
*"the rule reconciling `M1` (source order) with `M2` (reverse case order) —
the other half of R3"*.

**Price and risk, as dispatched.** **5–10 days, explicitly uncertain**, and
R8 is *"the only row with no known address for the rule it seeks"*. A priced
decline is named as an acceptable outcome by decision 7 and by the dispatch
brief. §4 below is the decline criterion, written before any instruction was
read, so that declining cannot be a retreat decided after the fact.

**Image.** sha256
`c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`,
**verified by this lane against the repo copy before this file was written**
(`C2_MAP_METHOD.md` §0):

```
$ sha256sum compilers/X360/16.00.11886.00/c2.dll
c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258
```

The flat export at `~/ghidra-projects/export/c2/` is dated 2026-08-04; its
input digest matching the pinned image is what licenses quoting its addresses
(`READ_PLAN` §5.4).

---

## 0. WHAT WAS LOOKED AT BEFORE THIS FILE WAS WRITTEN

Stated exactly, because the prereg tier is worth nothing if the boundary is
vague. Before writing this file the lane read:

* the **metadata rows only** for the brief's three addresses —
  `functions.tsv` (entry, size, nparams, ncallers, ncallees, framesize),
  `ref/FUNCS.tsv` (TU + attribution confidence, strings, imports) and
  `ref/ADDR.tsv` (citation list). This is R4's precedent
  (`WB_GLOBREGS_PREREG.md` §0), adopted deliberately: the brief demands the
  addresses be verified up front, and a `functions.tsv` row is metadata, not
  instructions;
* the prose record: `CLAUDE.md`, `docs/rungs/README.md` § "Lane kinds",
  `READ_PLAN_2026-08-21.md` §3/§4/§5, `DECISIONS_2026-08-22.md` decision 7,
  `ref/README.md`, `ref/P_LABEL.md`, `WB_LABELCHARGE_FINDINGS.md`,
  `CFG_SHAPE.md` §3.4/§3.4.1, `WB_REGALLOC_FINDINGS.md` §1 and §7.6,
  `rungs/2026-08-19-c2map3.md` § "Found and not taken", and the `BOARD.md`
  rows surfaced by topic grep (**#1906**, **#3240**, **#3242**).

**No instruction of `FUN_10b36133`, `FUN_10b34a89` or `FUN_10b968b0` has been
read, no body has been pulled from `decomp_all.c`, and `calls.tsv` has not
been opened for any of them.** Every prediction in §3 is made from the
metadata rows and the prose record only.

### 0.1 Dispatch defect check — the brief's three addresses, scored up front

The brief says every coordinator-supplied address list in this wave has needed
correction by the lane that used it, that R5's target turned out to be a third
its claimed size, and that **these three were NOT verified by the
coordinator**. Scored before predicting anything:

| brief says | `functions.tsv` | `ref/FUNCS.tsv` | verdict |
|---|---|---|---|
| `fg.c` `0x10b36133` | entry, **54 B**, 1 param, **1 caller, 1 callee**, frame 4 | `fg.c`, **`in-anchor`** (a fact), 1 string hook: the `…\be\p2\fg…` build path | **entry and TU verify** |
| `factor.c` `0x10b34a89` | entry, **597 B**, 4 params, 1 caller, **9 callees**, frame 32 | `factor.c`, **`in-anchor`**, 1 string hook: the `…\be\p2\fa…` build path | **entry and TU verify** |
| `0x10b968b0` *"the label format strings"* | entry, **507 B**, 4 params, 1 caller, **13 callees**, frame 44 | `optimize.c`, **`gap`** (a hypothesis, not a fact), strings **`%s$%s$%d%s`** and **`$%s$%d%s`**, imports `sprintf_s`, `strchr`, `strncpy_s` | **MISTYPED — it is a function, not a string table** |

**Two corrections to the brief, both recorded here rather than in the findings
so they cannot be confused with a result the lane produced after looking:**

1. **`0x10b968b0` is not "the label format strings". It is the 507-byte
   function that *uses* them** — `FUN_10b968b0`, a `sprintf_s`/`strchr`/
   `strncpy_s` name-builder over `%s$%s$%d%s` and `$%s$%d%s`. `ref/P_LABEL.md`
   already carries two of its interior addresses (`0x10b96978`, `0x10b96a69`)
   as label-allocator call sites. Its `optimize.c` attribution is **`gap`**,
   i.e. a hypothesis from address banding, not the `in-anchor` fact the other
   two carry. Quoting "optimize.c" about it without that qualifier repeats the
   error `READ_PLAN` §5.4 names.

2. **All three addresses are TRANSLATION-UNIT ANCHORS, not candidate rule
   sites.** This is the material correction. Their sole common ancestor is
   `rungs/2026-08-19-c2map3.md:243-249`, under the heading **"Found and not
   taken"**, whose own sentence is *"Phase 1 (emitter CFG classes) — **the next
   read is named, not taken**"*. Two of the three are rows of
   `WB_REGALLOC_FINDINGS.md` §1's TU table, which maps *a translation unit* to
   *an address in its band*. So the brief's *"start at `fg.c` `0x10b36133`"*
   means **"start in `fg.c`'s band"**, not *"the rule is in this function"* —
   and `0x10b36133` at **54 bytes with one callee** cannot contain a block
   ordering rule of any kind. c2map3 says so in the same breath: for `fg.c`
   *"1 string hook in the whole TU, so the strings route is dead and **the call
   graph is the handle**."*

   This is consistent with the read plan's own framing — R8 is *"the only row
   with no known address for the rule it seeks"* — but it changes the shape of
   the work from *read three functions* to **enumerate a band**, and §4's
   decline criterion is written against the second, not the first.

**One thing the brief got right and it is the strongest lead on the page:**
`WB_REGALLOC_FINDINGS.md:33` labels `factor.c` **"tail merging (a *block*-level
reorder)"**, and `CFG_SHAPE.md` §3.4.1's one refuting cell out of eleven —
`?d_join` — is **a tail merge followed by a hoist**, which is precisely where
that document's block-order rule broke. The brief's second address and the
record's one known counterexample name the same pass. §3 P4 registers that.

---

## 1. The question, stated precisely

Given a function's IL, **in what order do its basic blocks land in `.text`?**

The record holds two answers taken from the same compiler at the same flags:

* **`M1` — source order.** `CFG_SHAPE.md` §3.4: *"Blocks land in `.text` in the
  order their statements appear in the `.ex` stream."* Ten cells, every one
  consistent; `?d_cold` holds it against a six-call cold arm with no
  out-of-lining.
* **`M2` — reverse case order.** `WB_REGALLOC_FINDINGS.md` §7.6: seven leaf
  blocks of a six-case switch come out `default, 66, 55, 44, 33, 22, 11`.
  *"**Block order is not source order.** … as a rule a port could use, it is
  **not established**, and this lane does not claim it."*

**Prior art that already narrows this, and which this lane must not
re-derive.** Board **#1906** (`wb-loop`) measured 5 of 7 frozen block-order
cells and reports the contradiction is not one: **decision-tree switches
reverse, jump-table switches do not** — cell `d5` (4 arms → decision tree)
reverses, cell `a7` (12 dense cases → `lbzx` index table + `bctr`) does not,
everything else is source order. It also found one exception: a block reachable
only as an exit from **inside** a loop body is **sunk past the function's normal
return**.

So the *obj-side* discrimination exists. **What does not exist anywhere in this
repo is a mechanism** — no address, no pass, no list. That is what R8 is for,
and it is the difference between a rule a port can implement and a table of
cells a port can only match. §3's predictions are about the mechanism; a lane
that merely re-measures #1906 on more cells has produced nothing R8 was funded
for, and §4 treats that outcome as a decline, not a hit.

**Why it matters.** `CEILING` phase 1 is the one UNSERVED phase, on the premise
*"a port cannot place labels."* R3 supplied the **charge** and explicitly
disclaims the order (`ref/P_LABEL.md` §8 open #1: *"The ORDER. Which block a
`$M` lands on is R8's. A charge rule alone cannot place a label."*). This is
the other input.

---

## 2. The four rival mechanisms, named before looking

Registered as mutually exclusive readings of *where* order is decided. Each
carries the observable that would separate it.

| id | mechanism | separating observable |
|---|---|---|
| **H-LIST** | **There is no block-ordering decision at emit time.** c2's IR is a *tuple list* (`c2map3` §1); blocks are spans of it; the emitter walks it linearly. Order is whatever the list's *builders* left, and `M1`/`M2` are two facts about **construction**, not two rules about emission | the emit walk contains **no comparison and no sort** on any block field — it follows one `next` pointer |
| **H-LAYOUT** | A real **layout/placement pass** near emit computes an order from edge weights, arm sizes, or a traversal, and rewrites the list | a function exists whose body reorders block records and whose only caller is on the emit path |
| **H-DFS** | Order is a **traversal** of the CFG from entry (DFS or RPO), and both `M1` and `M2` fall out of the successor-list order at each node — reverse case order being reverse successor order at a switch node | the traversal is a visible loop with a stack/worklist; `M2` reduces to *"the switch node's successor list is built by prepend"* |
| **H-SORT** | Blocks carry a sequence number and are **sorted** before emission | a comparator, in the shape `P_REGALLOC.md` §4's already-read one |

**H-LIST is the lane's prior at 0.55**, H-DFS 0.25, H-LAYOUT 0.15, H-SORT 0.05.
The reason H-LIST leads is that under it `M1` and `M2` stop being rivals
without any new pass being posited, and it is the only reading that predicts
`CFG_SHAPE` §3.4.1 in advance: if order is list order, then **any pass that
splices the list moves block order**, which is exactly what a tail merge does
and exactly what that section observed.

**H-LIST is also the reading most likely to be true and useless.** "Order is
list order" is unfalsifiable as stated and licenses nothing. §3 therefore
registers it with a **payload requirement**: H-LIST scores a hit only if the
lane also names **at least one list-constructing site whose direction (append
vs prepend) is read from the instructions**. Without that it is scored a miss,
and §4 makes it a decline.

---

## 3. Predictions, frozen

`H` hit · `M` miss · `U` unscoreable (premise did not occur). Probabilities are
the lane's honest priors, not decoration; a prereg whose predictions are all
0.9 is a description.

### 3.1 About the brief's addresses

| # | prediction | p |
|---|---|---:|
| **P1.1** | `FUN_10b36133` (54 B) contains **no block-ordering logic** and is not on the path to any. It is an anchor. **Hit** = the read finds it to be a small helper (predicted: a label/symbol construction wrapper, consistent with R3 attributing an `fg.c` constructor site to it); **miss** = it turns out to matter | 0.85 |
| **P1.2** | `FUN_10b34a89` (597 B, `factor.c`) **is** on the block-order path — specifically it is part of the **tail merger**, and tail merging **mutates block order** as a side effect | 0.55 |
| **P1.3** | `FUN_10b968b0` is **not** on the block-order path at all; it is a name-builder and reaches the label allocator, which is R3's subject, not R8's. Registered as a prediction that one of the three seeds is a **dead lead** | 0.75 |
| **P1.4** | At least one of the three seeds is a dead lead **and the lane says which** — the meta-prediction the brief's own warning implies | 0.90 |

### 3.2 About the mechanism

| # | prediction | p |
|---|---|---:|
| **P2.1** | **H-LIST**: the final emit walk over a function's blocks performs **no sort and no comparison on a block-order key**; it follows a linked list. Hit requires naming the walk's address and its `next` field offset | 0.55 |
| **P2.2** | **The payload clause.** At least one **list-construction site** is read to direction — i.e. the lane states, from the instructions, whether that site **appends** or **prepends**, with the address and the field offsets. Without this P2.1 is scored **M** regardless | 0.45 |
| **P2.3** | The reversal in `M2` is produced **at construction of the switch's arm/case list, not at emission** — i.e. a prepend, or a descending walk of a case table. Hit = the site is named; miss = the reversal is found at emission | 0.45 |
| **P2.4** | **Block order has more than one author.** At least two distinct passes are shown to write the block/tuple `next` linkage after the flow graph is first built. (`WB_DAGCLIENTS_FINDINGS.md` already proves two `factor.c` functions reorder tuples, per c2map3; this predicts the same is true at *block* granularity) | 0.70 |
| **P2.5** | There is **no edge-weight or profile-driven layout heuristic** on this path at `/O1` — no "hot arm falls through" cost model. Rival: one exists and `CFG_SHAPE` §3.4's `?d_cold` simply did not trip it | 0.75 |

### 3.3 About the obj-side rule (the corpus/grid half)

| # | prediction | p |
|---|---|---:|
| **P3.1** | **#1906 replicates at corpus scale**: over the 878-TU workload's emitted objs, jump-table switches have targets **monotone ascending** in case index, and decision-tree switches do not | 0.70 |
| **P3.2** | The **jump-table/decision-tree threshold** is a readable constant (a case count and/or a density ratio), and the lane names it or bounds it | 0.40 |
| **P3.3** | The corpus contains **at least one switch shape that neither #1906's 7 cells nor this lane's grid predicted** — the standing outcome whenever a hand grid meets a corpus | 0.65 |
| **P3.4** | `M2`'s `default`-first placement is **not** part of the reversal rule but a separate fact about where the default arm goes | 0.50 |

### 3.4 The self-test that would void the lane

Registered in R3's pattern (`WB_LABELCHARGE_FINDINGS.md` §6): an instrument
that cannot reproduce an already-published cell is not measuring what it
claims. **Before any new cell is graded, this lane's obj-side extractor must
reproduce `WB_REGALLOC_FINDINGS.md` §7.6's `M2` exactly** — seven leaf blocks
in the order `default, 66, 55, 44, 33, 22, 11`, from the same source shape at
the same flags. **If it does not, the lane reports the extractor broken and
grades nothing.** That is a stop condition, not a caveat.

---

## 4. The decline criterion — explicit, quantified, frozen

The brief authorizes a priced decline and warns against manufacturing a rule to
avoid one. Both directions are bounded here so neither is a judgement call made
after seeing the answer.

**The lane DECLINES if, at the budget below, it cannot do BOTH of:**

* **(a)** name **one address** that is either (i) the emit-time walk over
  blocks, read to the point of saying whether it sorts or follows a link, or
  (ii) a site that **writes** the block/tuple linkage in a way that changes
  order, read to **direction**; **and**
* **(b)** state the `M1`/`M2` reconciliation as **one sentence containing a
  mechanism**, not as a table of cells.

**Budget, frozen:** **30 function bodies** pulled from `decomp_all.c`, and
**3 working days** of reading. Whichever binds first ends the read phase. A
count is used as well as a clock because the clock is the thing a lane can
rationalize.

**A decline is not a null result.** A declining report must still deliver, and
these are the deliverables either way:

1. the **elimination list** — every function opened, with one line on why it is
   not the rule;
2. **what it would cost to continue**, in the same units (bodies, days);
3. **the next place to look**, named as an address or a band, not as a topic;
4. the **corpus measurement** of §3.3, which does not depend on the read
   succeeding and is worth landing on its own.

**And the anti-fitting clause.** If the lane arrives at a rule that reproduces
every cell it has looked at but rests on **no read instruction** — a curve
through the cases — it must be reported as a **fitted description with its
fitting set named**, not as the rule. This repo scores a wrong emit strictly
below the refusal it replaces (`PROGRESS_METRIC.md`) and #3242's own lane
declined to recommend dispatch for exactly this reason: *"the rule is being
read off an ABSENCE of separation."*

**FAILED** — as distinct from `declined` — is reserved for producing none of
1–4.

---

## 5. What the controls are structurally incapable of catching

The brief's rule: *a control run where the discrepancy cannot appear is not a
control*. Asked positively for each instrument this lane will use.

1. **A read of the emitter cannot see a pass that ran earlier and permuted the
   list.** This is not hypothetical — it is `CFG_SHAPE.md` §3.4.1 already
   measured: c2 tail-merged two `bl` sites and hoisted an `li` above the
   compare, and *"block order is downstream of code motion."* **If H-LIST is
   right, then reading the emit walk correctly tells you almost nothing about
   order**, and a lane that reads it and stops has read the wrong thing
   correctly. P2.4 exists to force the question.

2. **A corpus scan of emitted objs cannot see block *identity*.** It sees
   instruction runs. Blocks that were merged, emptied, folded to branchless
   arithmetic, or converted to `bclr` **leave no trace to order**. `CFG_SHAPE`
   §3.5's fold table measures this directly — six of seven `cflow-if-1` leaf
   probes emit no forward branch at all. So the corpus population is **biased
   toward blocks that survived**, and any order rule confirmed on it is
   confirmed on survivors only.

3. **Neither a grid nor a corpus can distinguish "list order" from "an
   ordering pass that reproduces list order on everything compiled".** Only the
   read separates those, and only if it enumerates the writers rather than
   finding one.

4. **The corpus is dc3 at fixed flags.** A mode-dependent layout pass — `/O2`,
   POGO, `/Og` — is invisible to it. `WB_REGALLOC_FINDINGS.md` §7.7's
   byte-identical two-mode result bounds `/Oi`, `/EHsc`, `/GR`, `/GS-` **on
   those shapes** and explicitly does not extend to `/O2` or POGO. This lane
   inherits that boundary and does not widen it.

5. **`[R]` is a hypothesis.** Anything this lane reads and does not confirm
   against an obj is marked `[R]` and says *"the instructions were read
   correctly"*, never *"this is what c2 does"*. The `.bss` bump rule was read
   correctly out of a clean function and was wrong about c2
   (`ref/README.md`:54-60).

6. **The specific way this lane is most likely to be wrong**, named in advance:
   **finding one list-writer and reporting it as the author.** The record
   already says there are at least two (`WB_DAGCLIENTS_FINDINGS.md`'s two
   tuple-reordering `factor.c` functions; `WB_DAGORDER_FINDINGS.md`'s
   scheduler, which runs **four times per function**). A single named site
   presented as *the* mechanism would be this lane's version of
   `P_REGALLOC.md` §4 stopping one tier too early — the exact error R4 found
   and corrected. **The findings document must state the enumeration's
   closure argument or admit it has none**, in R3's pattern (R3's closure
   argument was that the allocator's VA never occurs as data).

---

## 6. The confirmation probe — grids and corpora fail in opposite directions

Both are run. Named against the failure modes, then asked the positive
question.

### 6.1 The corpus check (broad, unbiased in shape, blind to identity)

**Instrument:** a committed, re-runnable script under
`docs/whitebox/scripts/` that verifies the image sha256 first, then extracts
from emitted objs, per switch-bearing function:

* **jump-table switches** — the table's entries, and whether target offset is
  **monotone ascending in case index**. This needs **no source-order ground
  truth**: the table index *is* the case ordinal. That is what makes it a
  corpus-scale test rather than a grid;
* **decision-tree switches** — the `cmpwi`/`cmplwi` immediates in the
  comparison chain and the offsets of the blocks they guard, giving arm order
  **as a function of case constant**, again without source.

**Failure modes it is built against:** (i) a rule fitted to seven hand-written
cells; (ii) a threshold guessed from one grid; (iii) switch shapes nobody
authored.

**Would it go red if the claim were false in the most likely way?** The most
likely falsehood is *"decision-tree arms reverse"* holding only for small
dense switches. A corpus of real switches spans case counts, densities, sparse
value sets and nested switches, so **yes** — a non-reversing decision tree at
any size prints as a counterexample row. It cannot go red on blocks that were
folded away (§5.2), and the report will say what fraction it could see.

### 6.2 The grid check (narrow, controlled, sees what the corpus cannot)

A frozen source grid, content-hashed before the first `cl.exe`, in
`docs/whitebox/grids/wb-blockorder/`, varying **only** case count across the
suspected table/tree threshold, case density, case-label source order (dense
ascending vs deliberately scrambled), and default position.

**The control that must be a real control.** The scrambled-source-order cell
is the one that matters: a grid whose cases are always written `0,1,2,…` is
**structurally incapable** of separating *source order* from *ascending case
value*, and every cell in `WB_REGALLOC_FINDINGS.md` §7.6 and — as far as the
record shows — in #1906 is written that way. **A cell whose source order and
value order disagree is this lane's positive control**, and it is registered
here because it is the discriminator the prior cells could not carry.

**Would it go red if the claim were false in the most likely way?** The most
likely falsehood is that "reverse case order" is really "reverse *source*
order" or "descending *value* order" and nobody has separated them. The
scrambled cell separates them by construction. **If the two orders disagree and
the emitted order matches neither**, the lane learns that too.

### 6.3 The self-test gate

§3.4's reproduction of §7.6's seven-leaf order runs **first**. Nothing else is
graded until it is green.

---

## 7. Invalidation rules

1. **If the extractor fails §3.4's self-test**, the lane grades nothing and
   reports the instrument broken.
2. **If the read budget (§4) is exhausted without (a) and (b)**, the outcome is
   `declined` and §4's four deliverables are owed. No rule is stated.
3. **If a rule is found that rests on no read instruction**, it is reported as
   fitted, with its fitting set named (§4's anti-fitting clause).
4. **If the read runs into the final-expansion switch** (`FUN_10c0d57e` /
   `FUN_10c182b4`), it is cited as an **open cross-reference to `w-read-r6`**
   and not read here. Same for the scheduler (`w-read-r7`) and `0x4F`
   (`w-read-r9`). Peer lanes' pages are not edited.
5. **Zero `crates/` bytes.** Any `crates/` implication is written as a finding
   for a follow-up lane. `w-s1bc` is the only lane in `crates/`.
6. **`DISCLOSURE.md` gets a row only if** a disassembly-derived constant is
   adopted into `crates/`. This lane predicts **no row is owed**, for R1's
   reason: a read produces a spec, not an adoption.

---

## 8. Deliverables and bookkeeping

| | |
|---|---|
| page | `docs/whitebox/ref/P_BLOCKORDER.md` (new — checked: does not exist) |
| grade | `docs/whitebox/WB_BLOCKORDER_FINDINGS.md`, scoring every row of §3 verbatim, wrong ones kept |
| instrument | `docs/whitebox/scripts/` — committed, re-runnable, image sha256 verified first |
| grid | `docs/whitebox/grids/wb-blockorder/`, content-hashed before first compile |
| rung | `docs/rungs/2026-08-23-w-read-r8.md`, `Outcome:` one of `converted \| declined \| instrument \| built \| FAILED` |
| board | rows **#3437**–**#3441** only |
| amendments | **beside, never over** — `ref/README.md` §2.1. `ref/P_LABEL.md` §8 open #1 is answered or explicitly left open by name |

**Gates:** `C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release
--no-fail-fast` (exit 0 under the flag is itself the liveness assertion —
`require_toolchain.rs` makes a toolchain-less run FAIL) and `scripts/gate.sh
--jobs 4` with counts quoted. `scripts/board_audit.sh` before/after row-count
diff.

**Frozen at commit time. Nothing below this line is edited after the first
instruction is read.**
