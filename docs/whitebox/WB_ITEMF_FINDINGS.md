# WB_ITEMF — item F, priced: 7 steps, a ceiling of 17 lanes, and a buy of ZERO on every population the goal is written in

> **PROVENANCE — DISASSEMBLY-DERIVED.** Every address below is an absolute VA in
> the exact image pinned in [`C2_MAP_METHOD.md`](C2_MAP_METHOD.md) §0 —
> `sha256 c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`,
> verified on `compilers/X360/16.00.11886.00/c2.dll` at the top of this lane.
> Navigation only. **This lane adopts nothing into `crates/` and adds no
> `DISCLOSURE.md` row.**

PREREG: [`WB_ITEMF_PREREG.md`](WB_ITEMF_PREREG.md), committed at **`5fe20768`**
**before the first grep of the export** and before the first measurement of
anything in this repository. Scored in §8.

**The commissioned question** (`CFG_SHAPE.md` §6.2, 6 of 7 built): **price item
F. Do not build it.**

---

## 1. THE HEADLINE

1. **Item F's TITLE and its ENFORCING LINE quantify over different sets, and
   neither contains the other.** The title says *"values live across block
   boundaries"*. The mechanism its own cells are explained by ranges over
   *"a candidate whose live range spans a physical def or a clobber-set
   operand"* — **which contains no block**. `WB_LIVE_FINDINGS.md` §6.2 already
   wrote the refutation of the title's framing and nobody read it as one:
   ***"Nothing is special about the entry block — the copy lands wherever the
   arrival is."*** There is **no rule anywhere in c2's allocator that reads
   "this value crosses a block boundary."** The block enters only through the
   liveness *transfer function* — how you **compute** the range — never through
   any **decision**. Four for four on the reading rule (#3114, #3119, #3151).

2. **Item F's entire evidence base is three cells. Two already ship byte-exact
   by transcription and the third is OUT OF THE ACCEPTED CLASS by the
   document's own fence.** So **item F buys ZERO new bytes on its own
   motivating cells** (§3).

3. **§6.2 item F and §6.3 bullet 1 are in CONTRADICTION, and nothing in the
   repository records it.** §6.3 says *"**No code motion.** §3.4.1's hoist and
   tail-merge are recorded as a limit on the accepted class … **not as a pass to
   build**"*. Item F's cell 2 (`?d_join`) **is** §3.4.1, and #3099 has since put
   addresses on both halves of it: `0x10b3b167` is the tail-merge/cross-jump and
   `0x10b3b41b` is the head-merge/hoist. **Item F cannot be built to its own
   cells without building the pass §6.3 declines.**

4. **NEW, and it is the pricing headline: FOUR order-changing stages sit between
   the register allocation and the emitted bytes.** Read out of `FUN_10b7e6af`
   @ **`0x10b7e6af`** and `FUN_10b7dc51` @ **`0x10b7dc51`** (§4). The allocator
   `0x10b31c9a` finishes *inside* `0x10b7dc51`; then a third mode-1 schedule,
   then the lowering band `0x10b7dd2c`/`0x10b7ddff`/`0x10b7de4a`, then the five
   block mergers at `0x10b7ded5`, then the final mode-0 schedule at
   `0x10b7df57`. **The order that decided the registers does not appear in the
   obj.** A port cannot fit item F from emitted bytes, and this is a candidate
   mechanism — **hypothesis, not measured** — for two of this project's largest
   preregistered negative results at once (§4.3).

5. **The price is 17 lanes, ceiling, no discount factor** (§6), and **the
   scheduler is NOT the expensive half**: F0 is 8 of 17 and the *allocator* half
   everyone has been quoting at *"~30 lines plus the textbook"* (#3057) is
   **9**. That is a registered **MISS** (P2.4) and it is the most useful number
   here after the zero.

6. **The buy is ZERO on every named population** — the 381×18 fixture gate, the
   `c2rs perf` `/Ox` gate, the 878-TU workload scan, and the frontier (§7). Not
   arguable: `codegen-gap` is **0 over all 878** and `frontier-codegen-refused`
   is **0 of 59**. Item F lifts refusals in the emitter; on the frontier the
   emitter refuses **nothing**, because the reader refuses first.

For calibration: `CEILING.md` §6.2 puts the measured cost of the last five
conversions at **~17 landed rungs each**. **Item F prices at one historical
conversion and converts nothing.**

---

## 2. The reading rule, applied first

### 2.1 The two sets, written out

**The title's set** — *values live across block boundaries*.

**The mechanism's set** — read off `WB_LIVE_FINDINGS.md` §2/§6.1 and the
addresses behind it: `FUN_10b54d32` @ `0x10b54d32` mints a candidate with
`allowed := DAT_10c3d024[class]`, and `FUN_10b2d630` @ `0x10b2d630` narrows it
in **one forward walk**, clearing every physical register defined, and every
register in a `kind 0x0b` clobber-set operand, while the candidate is on the
live list. The set the *decision* ranges over is therefore

> *a candidate whose live range spans a physical def or a clobber-set operand,
> in the instruction order the allocator is handed.*

**Neither set contains the other**, with a witness on each side:

| direction | witness | what it shows |
|---|---|---|
| **mechanism ∖ title** | `wbl_v3` (#3054) — `r11` holds **three different values inside one straight-line block with no call between them**, `+0x44`/`+0x68`/`+0x94` | The strongest refutation of the port's incumbent register model is **entirely intra-block**. It is not in the title's set at all. The whole 492-cell evidence base of `crates/c2-core/src/codegen/alloc.rs` is likewise **single-block store runs**. |
| **title ∖ mechanism** | `crates/c2-core/src/codegen/fp_store_diamond.rs`'s `FPR_A = 0` — *"read by the then-arm AND the join, which straddle the branch, so it may not be reused by the else-arm"* | A value live across a block boundary, held in a **volatile** FPR, with **no copy and no callee-saved register**, shipping **byte-exact today**. Crossing a block boundary with nothing clobbering costs the port exactly the intra-block *"don't reuse"* rule and nothing else. **The title's set is where the price is NOT.** |

### 2.2 The sentence the previous lane already wrote

`WB_LIVE_FINDINGS.md` §6.2, on item F's own flagship cell:

> *"…the copy exists because the value's **arrival** register and its **chosen**
> register differ. **Nothing is special about the entry block** — the copy lands
> wherever the arrival is."*

That is the title's refutation, published 2026-08-13, sitting inside the very
section written to *support* item F. This lane's contribution is to read it as
one. **P1.1, P1.2, P1.3 all HIT.**

### 2.3 "The real cost" is false as a claim about where the cost is

The title's second clause asserts item F is *"the real cost"* of the
restructure. §6 prices the dominant term at **F0 — the instruction order the
allocator is handed** — which is **not item F**, appears **nowhere** in §6.2's
seven items, and is **forbidden by §6.3**. The item names the cheap half. **P1.5
HIT.**

---

## 3. Item F's evidence base is three cells, and it buys zero new bytes on all three

| # | item F's cell | where it stands today | what item F would add |
|---|---|---|---|
| **1** | `?MemFree` — `v2` from r4 to **r11** in the entry block, both successors need it (§4.2 item 8) | **SHIPS BYTE-EXACT.** `crates/c2-core/src/codegen/cond_tail.rs`, `mr r11,r4` at offset `0x0000`, graded against §4.1's thirty-six published bytes | **Nothing emitted.** The `11` becomes *derived* instead of transcribed |
| **2** | `?d_join` — `b` held in **r31** across the call (§3.4.1) | **OUT OF THE ACCEPTED CLASS.** §3.4.1: *"block order is downstream of code motion, and code motion is a c2 pass this document does not and will not characterize"*; §8 and §6.3: *"a body whose arms end in the same call is outside anything specified here"* | **Nothing.** Emitting it requires the tail-merge (`0x10b3b167`) and the hoist (`0x10b3b41b`) that §6.3 bullet 1 refuses to build |
| **3** | `?b_if2`/`?b_ifn` — formals in **r31/r30** across calls, framed for that reason alone | **SHIPS BYTE-EXACT** in this shape. `crates/c2-core/src/codegen/if_call_join.rs` parks the scrutinee at `0x0c`; `guard_ret_chain`, `close_call_chain`, `guard_chain_shared_tail` each carry `PARK_REG = 31` | **Nothing emitted.** The `31` becomes derived |

**And cell 3 needs no block boundary.** `wb-live`'s own grid grades it on
`wbl_x1`/`wbl_x2`/`wbl_x5` — straight-line bodies with calls — against negative
control `wbl_x3`. `wbl_x4` is the *only* cell of the four that is multi-block,
and it is the one `wb-live` had to construct specially. **P1.2 HIT.**

> ### ⚠ 2026-08-15 — A PEER FINDING CORRECTED IN PLACE
>
> **`crates/c2-core/src/codegen/if_call_join.rs`'s rationale for its park
> register is REFUTED BY ITS OWN LISTING, and the bytes are right.** The comment
> at `0x0c` reads *"THE PARK: the scrutinee is read by three tests that straddle
> two `bl`s, so it cannot stay in a **volatile argument register**"* — and the
> word it emits is **`mr r10,r3`**, with
> `select.rs`: `ARG_REGS: [u8; 8] = [3, 4, 5, 6, 7, 8, 9, 10]`. **`r10` IS a
> volatile argument register.** Its own listing also shows both `cmpwi cr6,r10`
> at `0x18` and `0x24` sitting **above** both `bl`s at `0x2c`/`0x34`, so the
> scrutinee's range **never straddles a call**.
>
> L0 explains `r10` exactly and without the false premise: `r3` is clobbered by
> the very next word (`mr r3,r4`), so `r3` leaves the allowed set; `r11` is
> taken by the accumulator home at `0x14`; the selector walks `r11, r10, …` and
> returns **`r10`**. This is a **prose** defect, not a byte defect — the class
> is byte-exact and stays so. **Not repaired here: this lane is docs-only and
> `crates/` is out of bounds.** Board row **F-4** proposes it.

---

## 4. NEW — four order-changing stages sit between the allocation and the obj

### 4.1 The read

`FUN_10b7e6af` @ **`0x10b7e6af`**, whole, from the export:

```c
if ((param_1[0x25] & 0xc000000) == 0) {
  if (DAT_10c2e2fc != 0) {          // the /Og bit, set at 0x10b82429
    FUN_10b7dbf6(param_1);
    FUN_10b7dc51(param_1);          // <-- globregs, 3 schedules AND THE ALLOCATOR
  }
  FUN_10b7dd2c(param_1);            // ] the lowering band
  FUN_10b7ddff(param_1);            // ]
  FUN_10b7de4a(param_1);            // ]
  if (DAT_10c2e2fc != 0) {
    FUN_10b7ded5(param_1);          // <-- the FIVE block mergers
  }
  FUN_10b7df57(param_1);            // <-- the FINAL (mode 0) schedule
  FUN_10b7e032(param_1);
  ...
}
```

and `FUN_10b7dc51` @ **`0x10b7dc51`**, whose internal order is equally explicit:

```c
FUN_10b38099(param_1);                    // gated DAT_10c2e310 && DAT_10c3de20 != 1
FUN_10be6382(param_1, 1);                 // schedule, mode 1  (pass 1)
FUN_10b57633(param_1);                    // globregs
FUN_10be6382(param_1, 1);                 // schedule, mode 1  (pass 2)
FUN_10b31c9a(param_1);                    // ***** THE REGISTER ALLOCATOR *****
FUN_10be6382(param_1, 1);                 // schedule, mode 1  (pass 3)
```

`0x10b31c9a` has **exactly one caller** (`calls.tsv`: `10b7dc51`), and
`0x10b7ded5` reaches the merger driver as `FUN_10b3c6e5(param_1, **2**)` —
independently confirming `WB_MERGER4_FINDINGS.md`'s `mode == 2` gate from a
second angle.

### 4.2 What that composes to, which nobody has written down

> **The instruction order the allocator sees is the output of schedule pass 2.
> The instruction order in the obj is that, plus schedule pass 3, plus the
> three-pass lowering band, plus five block mergers, plus the final mode-0
> schedule. FOUR order-changing stages, and the order that decided the registers
> is not in the object file.**

Each half of this was published and neither was composed. `WB_DAGORDER_FINDINGS.md`
§1 places the three mode-1 passes relative to the allocator and its revision box
corrects mode-0's position; `WB_DAGCLIENTS_FINDINGS.md` places the mergers before
the final schedule. **Neither states that all of them are downstream of
`0x10b31c9a`,** and the consequence for item F has never been priced.

It also **sharpens** `WB_LIVE_FINDINGS.md` §9.2's ranked blocker. That section
names *"the lowering order (`dag.c`'s tree-to-tuple walk, `0x10b3219f`)"* as the
thing to read. The order item F actually needs is **earlier** than the lowering
band, not part of it: it is the post-globregs, post-pass-2 order. The lowering
band is not on item F's *input* path — it is on the path between item F's answer
and the bytes, which is a different and additional cost (§6, F0 sub-lane 7).

### 4.3 A hypothesis this lane does NOT claim to have measured

Two preregistered searches in this repository have the same shape of residual:

* `codegen::alloc` — **52,416 priority-function allocators** (4 scan directions ×
  3 assignment points × 2 pool walks × 2,184 lexicographic keys over 7 features)
  top out at **179 of 236** fit cells, residual **exactly the tie tier**;
* `codegen::schedule` — **13,104 list schedulers** (forward/backward × latency
  1..6 × a lexicographic priority over six DAG features) top out at **89 of 146**,
  residual **exactly the two-producer tier**.

Both were fitted against **emitted** bytes. §4.2 says the emitted bytes are four
stages downstream of the order that decided the registers. **That is a candidate
mechanism for both residuals at once, and it is a HYPOTHESIS, not a measurement
— this lane built no cell that could falsify it and it is labelled here rather
than banked.** Testing it is a lane (§9, row F-2), and it would be the cheapest
way to find out whether F5 is 30 lines or a characterization.

---

## 5. The steps, each with its fail-closed boundary

§14.2's style: what the step admits, what it refuses, and what makes the refusal
decidable **before** a byte is emitted.

### F0 — the order the allocator is handed, and the four stages after it
**NOT BUILDABLE. Needs something that does not exist.**
* **Admits:** nothing yet.
* **Refuses:** every body, because the port cannot state the order.
* **The boundary is not decidable.** The only pre-emission predicate that
  separates "my order equals c2's" from "it does not" is **simulating the
  scheduler**, which is the step itself. §6.3's amended rule 1 (a decidable
  pre-emission predicate) is satisfiable here **only** by building F0.
* **What does not exist:** a **tuple-level IR below item A**. Item A's
  `BasicBlock` carries its instruction run as **bytes** (`place` takes position
  + bytes + terminator). A dependence DAG needs kinds, operands, def/use chains
  and symbol destinations — **none of which survive encoding**. §6.2 called item
  A *"the minimum the new IR must carry"*; item F needs an IR **underneath** it.

### F1 — the candidate set (globregs, `0x10b55732`)
**NOT BUILDABLE.** `WB_LIVE_FINDINGS.md` §10: read only far enough to establish
that it mints candidates and inserts merge candidates at joins; *"its promotion
policy — **which** symbols become candidates — is not characterized, and a port
that gets that wrong has the wrong value set before any of §9.1 applies."*
* **Fail-closed boundary:** refuse any body containing a value that is neither a
  formal nor a single-block temp. **That is today's boundary, exactly.**

### F2 — the liveness solver
**BUILDABLE TODAY.** Per-block `use`/`def` by a backward walk; backward
round-robin fixpoint `live_in = use ∪ (live_out ∖ def)` in reverse layout order;
**and the forward availability fixpoint intersected in.** Item A gives blocks, a
stated placement order and `FinishedBody::start_of`; **nine** production clients.
* **Fail-closed boundary:** refuse a body with a **back edge** — and
  `LabelMap`'s invariant 4 (#746) already refuses exactly that, so the fence
  costs nothing new. **It also means F2's re-expression base excludes the five
  loop classes** (`ptr_walk_loop`, `ptr_walk_chain_loop`, `json_utf8_copy`,
  `xtea_encrypt_loop`, `pool_ctor_chain`), i.e. precisely the bodies whose
  liveness is interesting.

> **This step absorbed a second one, and that is the anti-inflation check
> firing.** The first pass of this decomposition had *availability* as its own
> step F3. **What varies between "no backward fixpoint" and "no forward
> availability intersection"?** On the evidence: **nothing**. `wb-live` calls the
> intersection *"the clause a port would omit"* and **no cell of the 25 this
> project has compiled for the question goes red without it**. Two refusals with
> no variable separating them are one refusal. **P2.3 HIT** — and it is the same
> trap as the five `"past the eight register slots"` sites in §6.2 below.

### F4 — allowed-set narrowing
**HALF BUILDABLE.** The **call** case is black-box re-derivable and cheap
(`WB_LIVE_FINDINGS.md` §11: `grids/wb-live/live_grid.cpp` exhibits it against
real `c2.dll` with no address). The **non-call physical def** is not:
`WB_LIVE_FINDINGS.md` §10 — *"The non-call physical-register def (§6.2's `r4`
case, P5.3) **has no cell**."*
* **This is item F's flagship cell's own mechanism, and it is
  disassembly-only, ungraded by any obj.** Labelled, not banked.
* **Fail-closed boundary:** admit only bodies whose sole clobber sources are
  call tuples; refuse on any bare physical def.

### F5 — the colouring, and the candidate ORDER
**NOT BUILDABLE, and this is the step whose price is wrong in the literature.**
The *register* order `[r11 … r3, r31 … r14]` is settled and cheap
(`W-REGALLOC-1` has a black-box re-derivation; this grid widened its witnessed
span to `r28`). **Which candidate is coloured first is not.**
* `WB_LIVE_FINDINGS.md` §10: *"The `wbl_x2` assignment order is unexplained: `a`
  took `r30` and `b` took `r31` … which candidate is coloured first is set by
  the driver's worklist order, **which this lane did not read**."*
* **The repo has attacked the same unknown from the black-box side and been
  refuted.** `codegen::alloc`'s clauses 1–4 **are** a fitted candidate order;
  **clause 2 is REFUTED** (board #836 / `w-alloc2`, 7 of 56 fresh-holdout
  cells); clauses 3 and 4 *"carry opposite signs inside one sort, which is why
  the rule is not a priority function"*; and the 52,416-configuration search is
  the receipt.
* **Nobody has connected these two.** `codegen::alloc`'s sort key and
  `0x10b31c9a`'s unread worklist order **are the same unknown from two sides.**
  This is the single largest correction this lane makes to item F's price: the
  step called *"~30 lines"* (#3057, `WB_LIVE_FINDINGS.md` §9.1) is a
  characterization lane plus a construct rung.
* **Fail-closed boundary:** admit only bodies with **one** candidate per class
  live at any point — where the order cannot matter. That is a real predicate
  and it admits almost nothing.

### F6 — the arrival copy and the save set
**BUILDABLE for the acyclic case.** A value whose arrival register differs from
its colour needs a copy at its arrival; the callee-saved colours **are** the
prologue's save set; **framing is a consequence of allocation, not a cause**
(#3052 — `wbl_v3`, a **leaf** with no call, framed by pressure alone). `frame::FrameLayout`
exists.
* **Fail-closed boundary, and the port already spells it:** the general arrival
  copy is a **parallel-copy / permutation sequencer**, and `crates/c2-core`'s own
  refusal names the blocker in its message text — *"a permuted or literal-carrying
  first call beside a callee-saved copy: **c2 breaks the cycle through the
  callee-saved register instead of r11, which is not characterized**"*. Refuse
  any arrival permutation with more than one non-trivial cycle.

### F7 — the fence that would ship F2/F4/F5/F6 without F0
**BUILDABLE, and its measured output admits nothing new.** §6.3's amended rule 2
demands every fence be priced two-sided. Three candidate predicates, all
falsified:

| candidate predicate | why it fails |
|---|---|
| *"no hoistable materialization"* | The `lis @ha` hoist is **emergent from list scheduling, not a hoisting pass, and it BREAKS AT n=3** (#3068). A port implementing *"hoist the `lis`"* reproduces the cell and is **wrong in general**. |
| *"the arms do not end in the same call"* (§8's existing limit) | Necessary, not sufficient. `0x10b3baa8`'s merger is **textual over every pair in a label's predecessor list**, and its equivalence is over **tuples, not statements** — `mg_none3`: `x = 9` and `x = 8` in two arms **do** share a common tail, the store tuple. **A source-level predicate is wrong in both directions.** |
| *"single block"* | Survives — and it is **today's boundary**, `codegen::alloc`'s domain. |

**The two-sided price does not flip here** (unlike #1042 and NC-5/#2691, where
it did, twice): the fence's cost is zero because the set it withdraws is empty.

---

## 6. The price

### 6.1 Per step — ceiling, NO discount factor

| step | buildable today? | lanes (ceiling) | what the ceiling counts |
|---|---|---:|---|
| **F0** the allocator's input order + the four stages after it | **no** | **8** | (1) a tuple-level IR below item A; (2) region finder `0x10be5d4b` + DAG builder `0x10b328da`; (3) the machine model — latencies `0x10c1c1d4`, priority `0x10be5df6`, per-unit issue `LAB_10c1bfe2` — **with DISCLOSURE rows**, since the grid shows the latencies' consequences and not their values; (4) the cycle loop, ready list and the `node+0x44` post-merge tie-break; (5) K1/K2 `0x10b3b167`/`0x10b3b41b`; (6) M4 `0x10b3baa8` → `0x10b3a790`, incl. tuple-not-statement equivalence; (7) **the lowering band `0x10b7dd2c`/`0x10b7ddff`/`0x10b7de4a` — three passes, unread by any lane, and §4.2 puts them on item F's path**; (8) the four-pass interleave with globregs `0x10b57633`, reproducing the **allocation-time** order specifically |
| **F1** the candidate set (`0x10b55732`) | **no** | **2** | 1 characterization of the promotion policy + 1 construct rung |
| **F2** liveness (backward fixpoint ∩ availability) | **yes** | **1** | textbook, over `BodyLayout`'s placement order; absorbs the availability step (§5) |
| **F4** allowed-set narrowing | **half** | **2** | 1 build for the call-clobber case + **1 grid lane to obtain the first obj cell for the non-call physical def**, which is item F's own flagship mechanism and today has none |
| **F5** the colouring and the candidate ORDER | **no** | **2** | 1 characterization of `0x10b31c9a`'s worklist + 1 construct rung re-expressing `codegen::alloc` **required-zero over its 492 exact cells** (236 fit + 250 holdout + 6 killer) |
| **F6** arrival copy + save set | **yes** (acyclic) | **1** | `frame::FrameLayout` exists; framing follows allocation |
| **F7** the fence | **yes** | **1** | and it admits nothing new |
| | | **17** | |

**Ceiling. No discount factor applied** — five of the six times one was applied
on this project it was the error, and this seam re-priced phase 7 upward three
times in three days.

**F0 is 8; the other six total 9.** **The scheduler is not the expensive half.**
The half quoted at *"~30 lines plus the textbook"* is the larger one.

### 6.2 The refusal count, measured — and the trap fires twice

`crates/c2-core/src/` carries **165 distinct** `out_of_class` refusal strings.
**30** are register- or liveness-grounded. Asking *"what varies between these
refusals?"* of all 30:

| variable read | sites | does item F lift it? |
|---|---:|---|
| **V-ARRIVAL** — *is this operand a live-in formal still sitting in its arrival register?* | **7** | **yes** |
| **V-CS** — *is this value live across a clobber, and therefore callee-saved?* | **6** | **yes** |
| **V-PLAN** — *"the register plan is measured at two"* (formal count vs 2) | **2** | **yes** |
| **V-POOL** — *"store run whose producer took no register (`codegen::alloc`)"* | **1** | **yes** |
| **V-FPPRESSURE** — *"no free FP scratch register (would spill f31/f30)"* | **1** | **yes** (the FP class) |
| **V-PERM** — the arrival permutation's cycle count | **3** | **no** — the message text names its own blocker: *"which is not characterized"* |
| **V-ARITY** — *argument index vs 8* | **5** | no — calling convention, not allocation |
| V-SHAPE / V-FPPOOL / V-FILE / V-LINK | 5 | no |

> **The trap the brief warns about is present and measurable: five distinct
> refusal messages — a call argument, a chain link's argument, a literal
> argument, a data symbol's address and a parked permutation, each *"past the
> eight register slots"* — are ONE variable read at five sites.** Counting them
> as five is exactly the error a previous ceiling made. This is the second
> instance in this lane, after F2/F3 (§5).

**Item F lifts 5 independent refusals over 17 sites.** **P3.2 HIT** on both
clauses (5 < 10; 5 < 17).

### 6.3 The re-expression base nobody has counted

**22 files** in `crates/c2-core/src/codegen/` hard-code at least one physical
register number (`const … : u8 = 11 | 30 | 31 | …`). A construct rung for item F
is graded by a **required-zero byte delta** (board #290), so **every one of the
22 must come back byte-identical with its constant replaced by a colouring
result** — plus `codegen::alloc`'s 492 exact cells. **P3.3 HIT** (22 ≥ 10), and
**P3.4 HIT**: F5's price is dominated by that base, not by the mechanism.

---

## 7. What item F buys, in each named population (#3125)

**Named every time, because these three move independently.**

| population | today | item F complete | why |
|---|---|---|---|
| **878-TU dc3 workload scan** | `match 25 · mismatch 0 · **codegen-gap 0** · vocab-gap 845 · capture-fail 8` | **0 conversions** | `codegen-gap` is **0 over all 878** — *no TU waits behind the emitter*. Every remaining row is `vocab-gap`. **A codegen item cannot move `match` when nothing reaches codegen.** |
| **381×18 fixture gate** | 150 port Match, 0 mismatch, 231 not-implemented of 381 | **0** | A construct rung for item F is **required-zero by the grading rule**, and F7's fence (§5) admits nothing new. Zero here is *by definition*, not by weakness. |
| **`c2rs perf`'s `/Ox` gate** | 451× geomean over matched fixtures | **0** | Perf times an obj already proven byte-exact. A register model changes no timing population. |
| **the frontier** | `FRONTIER` **2** reachable by codegen breadth alone; 120 if factor A were free | **0** | **`frontier-codegen-refused` is 0 of 59** — *"three of its four stages are zero by construction while acceptance lives in the IL parser"* (#1475), and 48 of 59 frontier functions die in the IL parser before any emitter question is asked. Item F lifts emitter refusals. **The frontier's emitter refuses nothing.** |

**P4.1, P4.2, P4.3, P4.5 all HIT.**

### 7.1 The one positive buy, and it has no unit

Item F would turn **22 files'** transcribed register constants into derived
ones, and would replace `codegen::alloc`'s fitted sort — **whose clause 2 is
refuted and whose clauses 3/4 carry opposite signs inside one sort** — with a
mechanism. That is a **soundness** buy, and **the project's scoring rule has no
unit for it**: `PROGRESS_METRIC.md` scores emitted bytes and refusals, and a
transcribed-but-correct constant scores identically to a derived one.

**So, plainly, as the brief asks: item F buys ZERO on every population the goal
is written in.** **P4.4 MISS** — the positive buy exists but is not measurable in
any named population, which is the same as zero for scheduling purposes.

---

## 8. Prereg, scored

`WB_ITEMF_PREREG.md` at `5fe20768`. **H = hit, M = miss, U = unresolved.**

| # | p | verdict | note |
|---|---:|---|---|
| P0.1 | 0.90 | **H** | sha256 matches; all 19 cited VAs resolve in `functions.tsv`; `0x10b36f7e` shows **7 callers**, independently confirming `WB_MERGER4`'s lead |
| P0.2 | — | **floor cleared** | 7 steps, all 7 with a stated fail-closed boundary; §6.2 and §6.3 are counts measured in this repo |
| P0.3 | 0.85 | **H** | four absence-grounded claims labelled: §4.3's hypothesis, `0x10b3b5fd`, M5, the availability clause |
| **P1.1** | 0.70 | **H** | §2.1, witness on each side |
| **P1.2** | 0.65 | **H** | `wbl_x1`/`x2`/`x5` are straight-line; `wbl_x4` is the only multi-block one |
| **P1.3** | 0.55 | **H** | `fp_store_diamond`'s `FPR_A = f0`, volatile, cross-block, byte-exact |
| **P1.4** | 0.60 | **H** | §6.2 item F requires the pass §6.3 bullet 1 forbids; `0x10b3b167`/`0x10b3b41b` are §3.4.1's two transforms |
| **P1.5** | 0.60 | **H** | §2.3 |
| P2.1 | 0.60 | **H** | **7**, as registered |
| P2.2 | 0.55 | **H** | buildable today: F2, F6, F7 (3). Need what does not exist: F0, F1, F5 (3), plus F4 half |
| **P2.3** | 0.50 | **H** | F3 collapsed into F2; and V-ARITY's 5 sites are one variable |
| **P2.4** | 0.65 | **M** | **F0 is 8 and the rest total 9.** The registered claim that F0 ≥ the sum of the others is **wrong**, and the miss is the finding |
| P2.5 | 0.40 | **M** | every step ordered on evidence; F0 ≺ F5 ≺ F6 is forced by `0x10b7e6af`'s call order |
| **P3.1** | 0.50 | **H** | 17 ≥ 12, F0 = 8 ≥ 8 |
| **P3.2** | 0.60 | **H** | 5 independent refusals over 17 sites |
| P3.3 | 0.55 | **H** | 22 files |
| P3.4 | 0.45 | **H** | F5 |
| **P4.1** | 0.85 | **H** | `codegen-gap 0` |
| P4.2 | 0.80 | **H** | required-zero by the grading rule |
| P4.3 | 0.80 | **H** | |
| **P4.4** | 0.40 | **M** | the buy exists and has **no unit** — reported as zero |
| P4.5 | 0.70 | **H** | `frontier-codegen-refused` **0 of 59** |
| P5.1 | 0.75 | **H** | §9 names **8** |
| P5.2 | 0.60 | **H** | §4.3 |
| P5.3 | 0.90 | **H** | the cost term is not re-priced in either direction; #3057's four are untouched |
| P6.1 | 0.90 | **H** | §10 |
| P6.2 | 0.85 | **H** | §10 |
| P6.3 | 0.95 | **H** | docs-only |

**24 H · 4 M** (P2.4, P2.5, P4.4, and P0.2 scored as a floor rather than a
prediction). **P2.4's miss is the most valuable line in the table**: the
registered belief that the scheduler dominates item F's price is wrong, and the
half everyone has been calling cheap is the larger one.

**The registered bias direction (UPWARD) is confirmed as real and was
partly corrected by the check that was registered against it.** P2.3 fired
twice — F2/F3, and V-ARITY's five sites — and both collapses came *out* of the
total. Without them the number would have been 18 and the refusal count 10.

---

## 9. What CANNOT be priced, named as such

1. **`0x10b3b5fd`** — a DAG-building merge client that is **reached but never
   fires** on either of `w-dagclients`' grids. **Absence-grounded**: its cost is
   *unknown*, not zero, and it is carried outside the 17.
2. **M5, `0x10b3ab86` → `0x10b394f5`** — entered on every optimized cell and
   **never once reaching its inner call**, 13 cells × 6 levels. #3111's closure
   says it cannot *grow* the merger set; it does not say what it costs.
3. **The globregs promotion policy at `0x10b55732`** — which symbols become
   candidates. F1's ceiling of 2 is a guess at the *shape* of the work, not a
   measurement of it.
4. **The availability intersection.** Mechanically trivial; **necessity
   unmeasured** — no cell of the 25 goes red without it. Priced at zero inside
   F2 and that zero is absence-grounded.
5. **The non-call physical-register def** — **item F's flagship cell's own
   mechanism**, disassembly-only, with **no obj cell in existence**.
6. **Cycle-breaking in the arrival permutation** — the port's own refusal string
   says c2's behaviour here *"is not characterized"*.
7. **The scheduler's cost term.** Measured **inert on all 25 cells**. It can be
   priced neither up nor down. **This lane does not re-price it**, and #3057's
   four not-blocking items — the interference graph (there is none), the cost
   function, the spiller, a callee-saved policy — **are not re-priced here.**
8. **The `/LTCG:PGI` axis.** The workload builds at `/O1 /EHsc /GR`, so it is
   off-path *for this workload*. That is a **coverage bound, not a zero**: a port
   reproducing instruction order under PGO reproduces a **different** order.

---

## 10. The gate

Docs-only. `git diff master..HEAD -- crates fixtures scripts` **empty**.

| check | result |
|---|---|
| `gate.sh --jobs 4 --require-graded`, both ends | `graded tree e6d4bfb38066` · **730 files** · identical at both ends |
| expression sweep | 19,556 checked · **0 mismatches** · 19,460 graded |
| 878-TU scan | `match 25 · mismatch 0 · codegen-gap 0 · vocab-gap 845 · capture-fail 8` · **370 keys** — digit-identical |
| `board_audit.sh` | all-zero |
| `rung_registry` | 2/2 |
| `cargo test --workspace --release --no-fail-fast` | **1,619 passed / 42 targets** |

## 11. Pre-drafted DISCLOSURE rows — NONE

Nothing here is adopted by `crates/`. A future lane that builds **F0** adopts the
priority weights, the latency matrix and the region rule and owes rows for
`0x10be5df6`, `0x10c3bf9c`, `0x10c1c1d4`, `0x10c3c1a8`, `0x10be5d4b`,
`0x10be5cea` and `0x10b328da` (`WB_DAGORDER_FINDINGS.md` §9); a lane that builds
**F5** from the whitebox side owes one for `0x10b31c9a`'s worklist order. The
black-box alternative is sufficient for the *register* order (`W-REGALLOC-1`) and
for *"a value live across a call loses every volatile"*
(`grids/wb-live/live_grid.cpp`), and **insufficient** for everything else here.

## 12. What this lane did NOT establish

* **It built no cell.** Every obj fact quoted is a predecessor's, re-read rather
  than re-measured. This lane's own new evidence is **two decompiled functions**
  (`0x10b7e6af`, `0x10b7dc51`), one call-graph query (`0x10b36f7e`'s 7 callers,
  `0x10b31c9a`'s 1 caller) and **four counts measured in this repository** (165
  / 30 / 17 sites / 22 files).
* **§4.3's hypothesis is untested.** It is the highest-value cheap experiment
  this lane found and it deliberately did not run one.
* **The 17 is a ceiling on the SHAPE of the work, not on its difficulty.** No
  lane-count on this project has ever been validated against an outcome; the one
  calibration that exists (`CEILING.md` §6.2, ~17 rungs per conversion) is
  explicitly a **lower** bound (§5's streak calibration).
* **It did not read `0x10b7dd2c` / `0x10b7ddff` / `0x10b7de4a`.** They are named
  as the lowering band and priced as one sub-lane on the strength of their
  position alone. If any of them is large, F0 is larger than 8.
