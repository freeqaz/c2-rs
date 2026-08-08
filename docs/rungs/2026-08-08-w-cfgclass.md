# w-cfgclass — the frontier's first `cflow-if-n` conversion: `negate_test.cpp` is a match, and a transcription paid all ten of its hand-counted refusals at once

    Tag:       w-cfgclass
    Slug:      w-cfgclass
    Date:      2026-08-08
    Fixtures:  wcfg1_if_call_join.cpp wcfg1_if_call_join_neg.cpp
    Census:    711,486 / 2,463,443 → 711,488 / 2,463,443 (28.88 % → 28.88 %),
               **+2**. TU match **11 → 12**, mismatch **0 → 0**, codegen-gap
               **0 → 0**, vocab-gap **860 → 859**, capture-fail **7 → 7**,
               FRONTIER **16 → 15**.
    Record:    this file; prereg appended to `work/w-cfgclass/PREREG.md` at
               `1803bbab` and the fixture grid frozen at `23c9e760`, **before
               the first `cl.exe` on any grid cell**.
    Lane:      w-cfgclass, worktree branch `wt-w-cfgclass` off master
               **`b234d826`**.
    Ships:     `c2_il::func::body::shapes::if_call_join` (the recognizer),
               `c2_core::codegen::if_call_join` (the twenty words),
               `Selected::IfCallJoin`, two fixtures, one differential test.
               Board rows **#1630**–**#1638**; **#1639**–**#1659** left
               explicitly unminted.

---

## 1. The result

> ### `src/system/negate_test.cpp` — a FRONTIER TU — is a byte-exact `match`. **TU match 11 → 12, and the frontier is 15.** It is the first `cflow-if-n` body this port has ever emitted, and the first intra-section `b` it has emitted to a **join** block rather than to the epilogue.

| | base `b234d826` | tip |
|---|---|---|
| **TU match** | 11 | **12** |
| mismatch | 0 | **0** |
| codegen-gap | 0 | **0** |
| vocab-gap | 860 | **859** |
| port-error | 0 | **0** |
| capture-fail | 7 | **7** |
| **FRONTIER** | 16 | **15** |
| frontier-if-A | 138 | **137** |
| function census | 711,486 | **711,488** (+2) |
| emitted census | 39,185 | **39,187** (+2) |
| `fnbyte-exact` | 36,213 | **36,215** |
| `fnbyte-refused-parse` | 130,575 | **130,573** |
| frontier emitted functions | 59 | **57** |
| … of which reader-refused | 48 (81 %) | **46 (80 %)** |
| `fnbyte-tus-full` | 7 | **8** |
| workspace tests | 1,241 passed / 36 targets | **1,250 passed / 36 targets** |

Both of the TU's functions converted, and that is not two results: they are
**byte-identical to one another**, so the TU converts on one class or on none.
There was no partial-credit outcome available on this row and the prereg said so
(P5) before the build started.

---

## 2. Why this TU, out of the seven single-function frontier TUs

The brief's ladder says *start from the smallest measured distance, not the
biggest bucket*. The survey (§0 of the prereg, run before the predictions were
frozen) compiled the eight smallest frontier TUs at the workload's own flags and
disassembled every one. The ranking that decided it:

| TU | `.text` | fns | relocs | the marker that priced it out |
|---|---:|---:|---:|---|
| `Primes.cpp` | 64 | 1 | 6 | `w-loop` measured its first refusal in `crates/c2-il` and **three of five** structural refusals outside `codegen/`; ≥ 13 remain |
| **`negate_test.cpp`** | **160** | **2** | **2 + 2** | **nothing.** No `REFHI`/`REFLO`, no `__savegprlr_N`, no `.data`, no callee-saved GPR, no 32-bit constant, and the two bodies are **identical** |
| `undname.cpp` | 140 | 1 | 9 | r30/r31 saves, three `REFHI`/`REFLO` pairs, a store run |
| `osfinfo.cpp` | 152 | 1 | 10 | `srawi`/`mulli`/`lwzx` array indexing, a record-form `clrlwi.`, a `long long` return |
| `xlrcimpl.cpp` | 152 | 1 | 4 | `__savegprlr_26` — five callee-saved GPRs — and `lis`/`ori` constant pairs |
| `vswprnc.cpp` | 156 | 1 | 8 | a function *address* materialised into an argument register |
| `jsonwriter.cpp` | 304 | 1 | 2 | `cflow-loop`, `rlwimi`, `sthu` |
| `Main.cpp` | 124 | 1 | 6 | two `.pdata`, a 64-byte `.rdata`, `__CxxFrameHandler` and an EH funclet |

`negate_test.cpp` is not the smallest by `.text` and it is the smallest by
**everything a lowering has to build**. Board **#410** had already recorded that
all six of `w-cfgimpl`'s markers are absent from it; this lane re-confirmed that
on the current tree before choosing.

---

## 3. The twenty words, and the ten things about them

Ground truth: the real obj at the workload's flags, plus c2's **own `/FAsc`
listing** (`work/w-cfgclass/p/probe1.cod`), which names every block — so the
block *plan* below is read off c2's narration and not inferred from
displacements. The listing seam is black-box and free, and it is what made the
join block's identity (`$LN8`, distinct from the exit `$LN1`) a fact rather than
a reading.

```text
  0000  7d8802a6  mflr  r12                the shipped Class A 96-byte frame
  0004  9181fff8  stw   r12,-8(r1)
  0008  9421ffa0  stwu  r1,-96(r1)
$M2581:
  000c  7c6a1b78  mr    r10,r3        (1)  THE PARK
  0010  7c832378  mr    r3,r4         (2)  THE HOIST
  0014  39600000  li    r11,0         (3)  the result HOME
  0018  2f0a0001  cmpwi cr6,r10,1     (4)  ONE compare ...
  001c  41980020  blt   cr6,$LN1      (5)  ... read at LT ...
  0020  419a001c  beq   cr6,$LN1      (6)  ... and at EQ
  0024  2f0a0002  cmpwi cr6,r10,2
  0028  4198000c  blt   cr6,$LN2
  002c  4bffffd5  bl    ?FindLast     (7)  a call in a guarded arm
  0030  48000008  b     $LN8          (8)  the intra-section `b` to a JOIN
  0034  4bffffcd  bl    ?FindFirst
$LN8:
  0038  7c6b1b78  mr    r11,r3        (9)  the join ...
$LN1:
  003c  7d635b78  mr    r3,r11       (10)  ... undone in the next word
  0040  38210060  addi  r1,r1,96
  0044  8181fff8  lwz   r12,-8(r1)
  0048  7d8803a6  mtlr  r12
  004c  4e800020  blr
$M2582:
```

**(2) contradicts a rule the port already ships.** `w10_guarded_seq.cpp` pins,
in its own header, *"the guarded call's setup stays INSIDE the guarded block"* —
`g2` emits `cmpwi ; bt ; mr r3,r4 ; bl ?a1`, the `mr` **after** the branch. Here
the same `mr r3,r4` is **before every branch in the body**. Both are right: W10's
cell has one guarded call and this body has two calls that share one argument, so
the setup dominates both arms and c2 hoists it. Neither rule is more general than
the other and the port now holds both, in two productions that cannot reach each
other's bodies.

**(4)–(6) is the item with no representation in `Selected` at all.** The shipped
guard emitter (`seq_guard_emit`) emits one `cmpwi` per guard. A body routed
through it emits **twenty-one** words: the right program, four bytes long, and
every displacement after word 6 wrong. That is why this class does not route
through it, and `one_compare_serves_both_entry_guards` asserts it.

**(9)+(10) is `mr r3,r3`, and removing it is the defect.** The two words are two
*blocks*: `0x38` is the join both arms reach, `0x3c` is the exit the two entry
guards reach **without passing through it**. Every peephole a codegen lane would
reach for by reflex collapses them. Board **#1400** found exactly this on
`Primes.cpp` and wrote it as a warning; this is the second TU, and here it is a
`#[test]`.

---

## 4. Why a transcription paid ten refusals in one lane

Three independent hand-counts priced this TU: **w-cross 9**, **w-conv 9**, and
board **#411**'s **10**, against a construct-reprice's 4. The prereg registered
that they would reproduce within ±2 and would *not* collapse to 4 (P1). **They
reproduce.** Every item on #411's list is in §3 and is real.

**And the price was paid by one production, because the ten are not ten
mechanisms — they are ten descriptions of ONE block plan.** #411's list reads
*cr6 field choice · the compare CSE'd across two `if`s · the `mr r3,r4` hoisted
above the branch · `n` home-allocated to r11 with a redundant round-trip ·
`blendMode` evicting r3→r10 · block layout · the join block's start · the 96-byte
frame size*, plus the reader and the label stride. A **general** `if-n` lowering
would owe each of them separately, as a rule with its own grid. A
**transcription** — one named function class, `/O1` only, `NotImplemented`
outside — owes them **once**, as twenty words with two immediate fields.

That is `codegen::ptr_walk_loop`'s precedent applied one CFG axis over, and this
rung is the second outcome on board **#770**'s standing question. The first
(`Sort.cpp`, `w-hash`) overturned a decline priced at ≥ 8. This one overturns
three declines priced at 9, 9 and 10.

**The honest form of the claim, stated so it cannot inflate.** A refusal count is
a description of the work and never a prediction that the work will not be done —
but neither is this lane evidence that the counts were wrong. They were **right**,
and they were counts of a *lowering* the project did not build. What the two
outcomes now say together is narrower and more useful:

> On a frontier TU whose blocked functions are ONE block plan, the hand-counted
> refusal total prices a **general** lowering and **over-prices a
> transcription** — because a transcription's cost is the number of *plans*, not
> the number of *facts about a plan*. On a TU whose blocked functions are several
> plans (`keygen_xbox.cpp`: 18 functions over 10 keys), it does not, and nothing
> here licenses that.

---

## 5. The two things that are not in any published price, and were found by building

### 5.1 The label stride of this class is **6**, not 5 (#1632)

Every framed class this port ships strides **5** under `/Gy`. This one strides
**6**. Measured **seed-free and in-TU** — the two functions of `negate_test.cpp`
are both this class and nothing else, and their triples are
`$M2581/$M2582/$T2583` then `$M2587/$M2588/$T2589`, a difference of **6** with
the TU's single `_fltused` slot cancelling out of the difference entirely.

It ships as `IlFunction::label_lead() + 1`, i.e. a slot taken **before** the
function's own triple, which is the placement `docs/LABEL_COUNTER.md` §1.1
records for every other surcharge in the table. A wrong `$M` is six wrong bytes
in an obj that still links (board **#263**), so this is the item most likely to
have shipped silently wrong — and it did not, because the differential compares
the symbol table.

### 5.2 `touches_floating_point` is **structural** here, and a body scan says no (#1633)

The class's third formal is a `float` that travels in fr1 and **emits no
instruction at all**. So a predicate written as *"does the body have FP ops"* —
which is what `touches_floating_point` mostly is — answers **no**, and the obj is
then one symbol short of `_fltused`: `Port=Mismatch @ offset 12`, the COFF
header's `NumberOfSymbols`, on every positive case at once.

That doc-comment already **warns** about this exact miss (*"the shape that gets
missed is the one that is FP-touching without being FP-shaped"*, W36's lesson),
and this is its fourth instance. The predicate is answered from the class rather
than from a body scan, because the recognizer requires the third formal to be a
4-byte real — so it is a structural fact and cannot drift from the bytes.

### 5.3 The mode fence belonged in the PARSER, and the census was over-claiming until it moved (#1638)

The `/O1`-only clause started in the emitter, beside `ptr_walk_loop`'s. That is
one locator for a fact the census also needs, and
`crates/c2-harness/tests/census_gate.rs` failed on it in the exact words it was
written to fail in — *"a new gate landing in codegen instead of the parser fails
this test"*:

```text
  census/gate [fixtures, fn_level_linking=false]: 6 disagreements
       5  not implemented: if/else-with-a-join at /Ox or /O2 …
       1  not implemented: no free FP scratch register   <- the standing one
```

Five fixture functions the census counted **in class** and `PortC2` refused —
five entries on the error term of the published coverage numerator, which is
`docs/GAPS.md` §6's one-fact-two-locators defect in its purest form. The
recognizer now asks the opt word **first**, before any body byte is read, so the
refusal cannot depend on how far the walk got; the emitter keeps its own clause,
because `select_function` is what `function_gate` runs and a shape arriving there
under the wrong mode must refuse rather than emit.

**And `codegen::ptr_walk_loop` carries the identical `/O1`-only clause with no
parser half.** It does not trip the cross-check today only because no fixture
puts its shape in front of the packed lane's profile — an absence of evidence,
not a clean bill. The comment is in `if_call_join.rs` so the next lane to touch
that class knows the fix is one call. **This lane did not make it**, because
changing `ptr_walk_loop`'s accept boundary is a change to a class it did not
grade.

---

## 6. The fence, and where it is narrower than c2's class

### 6.1 The grid, frozen at `23c9e760` before the first `cl.exe` on it

Five positive cells, six negative. **Every registered prediction hit.**

| cell | prediction | outcome |
|---|---|---|
| p0 | Match | **Match** |
| **p1** (`!(b != K)` for `b == K`) | Match **and byte-identical to p0** | **Match, byte-identical** |
| p2 (both literals moved) | Match; exactly two words differ | **Match; words 6 and 9, exactly** |
| p3 (negative literal) | Match | **Match** |
| p4 (other callees, other pointee) | Match | **Match** |
| n0…n4 | `NotImplemented`, never `Mismatch` | **0 of 6 in class** |

**p1 is the separating cell** and nothing else in the corpus does its job: two
source spellings that must emit **one word**. c2 deletes the empty middle arm and
inverts the sense once, so `1F`+`38` (`==`, branch-if-false) and `20`+`39`
(`!=`, branch-if-true) name the same successor. The reader admits both; if they
had diverged, the alternation was wrong and the spelling the workload does not
contain had to be refused. They did not diverge.

**p2/p3 vs p0 differ in words 6 and 9 and nowhere else**, which is how "the two
literals are the only two immediate fields" became a measurement instead of a
claim.

### 6.2 The negative cells are braced like the positives, and that is the confound

`w-clear` was bitten twice by a grid whose cells failed for a reason other than
the one under test. **This grid was bitten too, and by its own instrument, on the
first draft** — see #1636.

`c2rs census` reports only the **fall-through** blocker, so all six cells read
`assign-store-type-8643` whatever they actually tripped, and the file looked
exactly as complete before the fix as after. Printing the recognizer's *own*
decline context per cell (`work/w-cfgclass/decline_probe.patch`, a scratch diff,
applied and reverted) found two cells failing on a clause another cell already
covered:

| cell | first draft | after |
|---|---|---|
| **n0** | four formals → `ifjoin-formals-not-3`, the **arity** clause `n2` already holds | three formals, one arm's argument changed → **`ifjoin-arm-arg1`** |
| n1 | `ifjoin-dead-store` | unchanged |
| n2 | `ifjoin-formals-not-3` | unchanged |
| n3 | `ifjoin-acc-not-a-ptr-local` | unchanged |
| **n4** | `b < 5` against an outer `b >= 1` → `ifjoin-mid-lit-differs`, the **shared-literal** clause | `b < 1` → **`ifjoin-mid-rel`** |
| n5 | `ifjoin-inner-body-scopes` | unchanged |

**Six cells, six distinct clauses, none of them the fall-through.** Every cell is
braced exactly like the positive file — the recognizer pins every `54 <k>` scope
depth, so an unbraced arm refuses on the *bracing* — and **`n5` isolates the
bracing itself** so the other five cannot be confounded by it.

### 6.3 The fence is NARROWER than the class c2 has, and `n5` says so (#1635)

`n5` is `p0`'s program with the two innermost arms unbraced. It **refuses**, and
c2 almost certainly emits it identically — bracing is visible to this reader and
invisible to `.text`. That is recorded as a cell rather than left to be
discovered, because a fence narrower than the class is the safe direction and a
fence *wider* than the graded cells is board #232's 241 commits.

Widening it needs its own graded cells. This lane did not run them.

### 6.4 What WAS widened, on evidence

The `2C` enum→int widening started **required** and refused every non-enum
spelling of the same body (p2/p3/p4 came back `GAP`). Requiring its *absence*
would have refused the workload's own. It is now optional and **both forms are
graded** — p0/p1 enum, p2/p3/p4 `int` — which is the only reason the widening is
in and not a guess.

### 6.5 The mode fence

`/Ox` and `/O2` refuse in codegen with the measurement in the message: the join
block **tail-duplicates** there, on a threshold W10 bracketed with one cell either
side and did not fit (board row **X-b**). `/Od` refuses upstream on the
optimization word. `codegen::ptr_walk_loop` carries the same clause for the same
reason. The differential test grades exactly that arm, because `differential()`
drives the default `/Ox` profile and has no `--flags-file`; the `/O1` arm is
graded by `scripts/mode_lane.sh /O1` inside the gate.

---

## 7. The defect that cost the most time, and it is a copying defect (#1636)

The recognizer's accumulator clause was copied from `ptr_walk_loop`'s:

```rust
if !locals.contains(&acc) || !ptr_locals.contains(&acc) {   // WRONG
```

`.sy` classifies an automatic width-4 **data pointer** whose address is never
taken into `ptr_locals`, and an **integer** local into `locals`. They are
disjoint. `ptr_walk_loop`'s accumulator is an `int` and its pointer is a pointer,
so it asks each list once; this class's accumulator is a pointer, and the
conjunction refuses **every** instance of the class by construction.

It is in this rung because the shape of it recurs: a clause copied verbatim from
the nearest sibling production, correct there, vacuously false here, and failing
**closed** — so nothing in the census, the gate or the differential would ever
have reported it as anything but "the class matches nothing". The only instrument
that found it was a one-line print.

---

## 8. `assign-store-type-8643` recovered two TUs' worth here, and #407/#1363 measured **zero** — both are right (#1637)

Board **#407** and **#1363** measured this key's *sink* at **0 recovered of
1,133**, over five 878-TU scans, with the emitted census `+0`. `negate_test.cpp`
is the one frontier TU whose entire emitted blocker set is this key, and **#1365**
measured its ladder credit at **0** across four arms.

This rung converts it. The two results are compatible and the distinction is the
point:

* a **sink** consumes a token and pushes no `IlOp`, so it can only ever *rename*
  the blocker — #1465's rename trap, confirmed nine times;
* a **class** consumes the whole body and emits bytes.

`assign-store-type-8643` was never a lever and is not one now: it is the first
gate in front of exactly one named function class, and the other 1,131 rows
behind it are untouched. **Nothing here reopens the key as a family**, and the
prereg registered that (§5) before the build.

---

## 9. What this does NOT claim

**It is not a `cflow-if-n` lowering.** Of the frontier's blocked functions,
`cflow-loop` holds 21, `cflow-if-n` 11 and `cflow-if-2` 1. This recognizer takes
**two** of the eleven. The remaining nine `cflow-if-n` functions sit in
`osfinfo`, `undname`, `vsnprnc`, `vswprnc`, `xlrcimpl`, `mmio`, `wordwrap` and
`keygen_xbox`, and §2's table is the reason none of them is next for free.

**The CFG-reachability instrument still reads 2 of 15**, not 3. That block counts
a frontier TU as reachable when its blocked functions' CFG classes are port
classes; `negate_test.cpp` left the frontier by *matching*, so it is out of the
denominator rather than in the numerator. The instrument is unchanged and its
text still describes three shapes — a lane widening it should read §9's first
paragraph first, because "the port has a `cflow-if-n` class" and "the port can
express `cflow-if-n`" are not the same sentence and this rung only establishes
the first.

**A green gate is sound only on the IL it was tested against.** This class was
graded on 5 fixture cells and 2 workload functions.

---

## 10. Pre-registration, scored

Nine predictions plus a registered bias, committed at `1803bbab` before the first
line under `crates/`. **7 right · 1 wrong · 1 half · the registered bias fired in
the direction registered.**

| # | prediction | verdict |
|---|---|---|
| **P1** | ≥ 8 independent refusals; #411's 10 reproduces within ±2 and does **not** collapse to 4 | **RIGHT.** All ten reproduce, plus two the published counts do not contain (§5) |
| **P2** | the first refusal is in `crates/c2-il`, not `crates/c2-core` | **RIGHT** — `assign-store-type-8643` at the `n = 0` store, and no IL body reached `select_function` |
| **P3** | the intra-section `b` to a join is the one item never emitted in any graded cell, and the class must refuse `/Ox` | **RIGHT** on both halves; the refusal is §6.5 |
| **P4** | the compare CSE has no representation in `Selected` and none in the guard emitter | **RIGHT**, and it is the reason the class does not route through `seq_guard_emit` |
| **P5** | the two functions emit byte-identical `.text`; no partial-credit outcome exists on this TU | **RIGHT**, and `p1` reproduces it in a fixture |
| **P6** | **THE CONVERSION CALL: match 11 → 11. This lane does not convert `negate_test.cpp`** | **WRONG — match 11 → 12.** The registered reason was P1+P2 together: *"the reader half is a new statement-layer production, which is the block IR §7 says has never been sized"*. The block IR **was not built**; the reader half turned out to be a linear token pattern of ~470 lines, which is exactly what decline clause **D2** was written to detect and D2 correctly did **not** fire |
| **P7** | the deliverable is a fenced **transcription** in `ptr_walk_loop`'s tradition, not a general `if-n` lowering | **RIGHT**, and §4 is why it converted anyway |
| **P8** | mismatch 0, codegen-gap 0, gate 18/18 with 0 mismatch | **RIGHT** — §11 |
| **P9** | REGISTERED BIAS: if P6 is wrong it will be wrong **optimistically on the reader**, not the emitter — the emitter half is smaller than it looks and the reader half larger | **HALF, and the half it got wrong is the load-bearing one.** The emitter half *was* smaller than it looks (the frame, the label lead and the branch encoders are all shipped). The reader half was **not** larger — it was one pattern, and my P6 rested on assuming it needed a block IR. So P9 correctly named which half I would misprice and got the **sign** backwards |

### What the scoring bought

**P6 is the useful loss, and it is board #770's twelfth entry on the pessimistic
side of a streak that is ten-to-one optimistic.** The brief said *"most declines
were REAL, most optimistic estimates were wrong; do the work before declining"*.
This lane registered a decline, did the work anyway, and the decline was wrong.

The mechanism is worth more than the outcome: **I priced a transcription with a
lowering's cost model.** Every number I trusted (9, 9, 10) is a count of what a
*general* `if-n` lowering owes, and every one of them is correct as such. None of
them is the cost of matching one token sequence and emitting twenty words. Board
**#666** already names this failure from the other side — *"a correct construct
inventory and a wrong cost model"* — and this is the same error with the sign
flipped: a correct inventory, a cost model that was right for a thing I was not
building.

---

## 11. Gate

### 11.1 `scripts/gate.sh --require-graded`

Run at tree `9ccf853b` (the last code commit; the later commits are docs only).

```text
  18 lanes in the registry — 18 PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT
  graded    5,238 fixture-verdicts across all lanes
  sweep     PASS — 19,556 of 19,556 selected cases reached,
                   19,460 GRADED by the oracle, 0 mismatch
                   (96 ungraded: the reference rejects the source)
  cross     PASS — 90,812 of 90,812 case-lane cells selected,
                   90,424 GRADED, 0 mismatch
  hatch-red PASS 14/14 · ladder-red PASS 5/5
  GATE: PASS — 0 mismatches anywhere
```

`5,238` is the brief's baseline `5,202` **plus 36** — two new fixtures across
eighteen lanes, which is the arithmetic a lane adding fixtures owes.

**Per-lane `match`, and the one number that moved.** `/O1` and every `/O1`
variant read **147**; `/Ox` 138, `/Ox /Gy` 136, `/O2` 142, `/Od` 18. Measured
directly on the two new fixtures at each lane's own flags:

```text
  /O1 /GS- /c   wcfg1_if_call_join.cpp      match        <- the +1
  /Ox /GS- /c   wcfg1_if_call_join.cpp      vocab-gap
  /O2 /GS- /c   wcfg1_if_call_join.cpp      vocab-gap
  /Od /GS- /c   wcfg1_if_call_join.cpp      vocab-gap
  every lane    wcfg1_if_call_join_neg.cpp  vocab-gap
```

So the class contributes **+1 to the four `/O1` lanes and +0 to every other**,
which is what an `/O1`-only class must look like from outside. The `/Ox` reading
is `vocab-gap` and not `codegen-gap` **because §5.3 moved the gate into the
parser** — before that move it read `codegen-gap`, and the two spellings are the
census over-claiming and not.

`cargo test --workspace --release`: **1,250 passed, 0 failed, 36 targets**
(base 1,241 / 36).

### 11.2 The workload scan, both ends

Two full 878-TU scans, one at `b234d826` and one at the tip, compared line by
line (`work/w-cfgclass/scan_base.out`, `scan_tip.out`). **The only differences
outside the provenance header and the parallel-completion ordering are the ones
this rung is about**:

```text
  match                 11 → 12          mismatch 0 → 0     codegen-gap 0 → 0
  vocab-gap            860 → 859         capture-fail 7 → 7
  FRONTIER              16 → 15          frontier-if-A 138 → 137
  function census  711,486 → 711,488     emitted 39,185 → 39,187
  disp-assign      729,173 → 729,171     disp-if-call-join   (new)  2, blocked 0
  fnbyte-refused-parse 130,575 → 130,573
  fnbyte-shape-if-call-join-exact  (new key)  2
  the joint IS the match set, 12 TUs by name, `negate_test.cpp` among them
```

**No key vanished and exactly two appeared** (`disp-if-call-join`,
`fnbyte-shape-if-call-join-exact`), which is what a class that accepts exactly
two functions must look like. `progress-mismatch-zeroed` is 0 at both ends.

### 11.3 The known-answer controls, at the tip

* `the joint is EXACTLY the match set` — 12 TUs, by name, including
  `src/system/negate_test.cpp`.
* `known-answer control — matching TUs failing each NECESSARY term`: A 0, B 0,
  C 0, D-or-E 0 over **12** matching TUs.
* `BYTE-FRACTION CONTROL` (#501): 5 of 12 matched TUs at 100 %, up from 4 of 11 —
  the new TU is one of the 100 % rows, which is the control that says the port
  produced a body for every `.text` byte in it.
* `gate-anchored control on matching TUs`: 0 violations over 12.
* `partition-broken` 0, `frontier-codegen-measured` unchanged.

---

## 12. Reproduction

```sh
export C2RS_DC3=<the dc3-decomp tree>          # resolves from a worktree
export C2RS_WIBO=<the wibo build>

echo src/system/negate_test.cpp > one.txt
c2rs gap --list one.txt --flags-file work/dc3-workload/flags.txt --cwd "$C2RS_DC3"
#   [1/1] match        src/system/negate_test.cpp

c2rs census fixtures/cpp/wcfg1_if_call_join.cpp     --flags-file <(echo '/nologo /c /O1 /Oi /EHsc /GR')
#   5/5 functions in class
c2rs census fixtures/cpp/wcfg1_if_call_join_neg.cpp --flags-file <(echo '/nologo /c /O1 /Oi /EHsc /GR')
#   0/6 functions in class

cargo test -p c2-core --release codegen::if_call_join     # 8 tests
scripts/gate.sh --require-graded
```

The listing seam that produced §3's block names:

```sh
c2rs listing work/w-cfgclass/p/probe1.cpp --flag /O1 --flag /Oi --flag /EHsc \
    --flag /GR --flag /c --out work/w-cfgclass/p/probe1.cod
```
