# W-DCLASS/C — the store-operand type vocabulary is three kinds, and all three are worth zero

    Tag:       W-DCLASS-C
    Slug:      w-dclass-c-storetype
    Date:      2026-08-05
    Fixtures:  none — this rung ships an INSTRUMENT, not an accepted class: it
               renames one census bucket into three, admits nothing, and DECLINES
               on a measured +0 the widening it was sent to price.
    Census:    706,555 → 706,555 / 2,463,393 (28.68 % → 28.68 %), **+0**
    Record:    this file
    Lane:      w-dclass, subagent C (`wt-w-dclass-c-storetype`, base master `9f9e6c0`)
    Verdict:   **0 TUs converted, 0 functions converted, and both target TUs
               declined with a price.** What lands is a census split that turns
               8,222 unnamed rows into two named constructs, plus the three
               measurements that close the seam.

---

## 1. Result, up front

| | master `9f9e6c0` | this branch | Δ |
|---|---|---|---|
| 878-TU scan: **match** | **8** | **8** | **0** |
| mismatch / codegen-gap / vocab-gap / capture-fail | 0 / 0 / 863 / 7 | 0 / 0 / 863 / 7 | 0 |
| per-function census | 706,555 / 2,463,393 | 706,555 / 2,463,393 | **+0** |
| census/gate disagreement | 0 | 0 | 0 |
| FRONTIER | 19 | 19 | 0 |
| distinct blocker keys | 674 | **675** | +1 — two rows became three |
| blocked function-sites | 1,756,838 | 1,756,838 | 0 |
| every key outside the store family | — | **identical, name and count** | checked, not assumed |
| `cargo test --workspace --release` | 806 / 0 / **27 targets** | **809 / 0 / 27 targets** | +3 tests |

Both baselines were re-measured on this branch's own inputs, not quoted.

## 2. The type-triple census — and it is a CLOSED vocabulary, not a sample

`assign-store-type-0x86` was 8,222 workload functions in one row that named none
of them. The refusal was raised through `blk`, which packs no `aux`, so
`Block::feature` fell through to its bare `<ctx>-0x<byte>` arm and printed the
**tag** — and the tag is the *slot's* width, not the type. `0x86` says "4 bytes
wide" and nothing more, so a data pointer, a code pointer, a `float` and a
`pack(4)` `long long` were all one bucket. The sibling `expr-load-type-*` keys
have rendered `<tag><kind>` since `GAPS.md` §6; this site never got the same
treatment.

Raising it through `blk_type` instead splits it, exhaustively:

```text
  assign-store-type-0x86   8,222  ->  8643   6,820   4-byte DATA pointer
                                      8644   1,402   4-byte CODE pointer
  assign-store-type-0x82   1,906  ->  8212   1,906   bool / unsigned char
  assign-store-op                              0     the `32` opcode matched at
                                                     all 10,128 sites
```

6,820 + 1,402 = 8,222 exactly, with no third kind and no remainder. The distinct
key count goes 674 → **675** (two rows became three), the 1,756,838-site total is
unchanged, and **every key outside the store family is identical in name and in
count** — checked programmatically over both scans rather than eyeballed. That is
the positive check with a printed count, not an absence.

`assign-store-op` is a **guard, not a bucket**, and its zero is the point: the
`32` opcode matched at every one of the 10,128 sites, so the old `||` never once
reported the missing-opcode case — which is why folding the two facts together
was harmless in fact and unsound in principle. `blk_type` would pack garbage if
the opcode had not matched, so the split is load-bearing even though this arm
never fires on this workload.

### 2.1 Each kind established by capture, never inferred from a neighbour

A probe of 24 one-store functions, compiled at the workload's own
`/O1 /Oi /EHsc /GR` profile, with `c2rs census` reading the triple back under its
`>` marker. **The source is §8 of this file rather than a path**: it lived in
`work/`, which is gitignored, and a rung whose evidence cannot be regenerated
from the committed record is a rung whose evidence does not exist.

| probe | triple at the `32` | key |
|---|---|---|
| `int *x; x = q;` | `86 43 f4 08` | `assign-store-type-8643` |
| `void *x;` · `S *x;` · `const int *x;` | `86 43 …` | same |
| `FnPtr x; x = q;` (`typedef int (*FnPtr)(int)`) | `86 44 88 20` | `assign-store-type-8644` |
| `bool x; x = b;` · `unsigned char x; x = c;` | `82 12 30` | `assign-store-type-8212` |

The two FRONTIER witnesses decode straight out of that table: `Sort.cpp`'s
`?HashString` blocks on `32 86 43 a0 08` (`unsigned char *u = (unsigned char *)str`)
and `negate_test.cpp`'s two functions on `32 86 43 82 20`
(`const CharGraphNode *n = 0`).

Re-reading the whole frontier through the split keys sharpens the brief this lane
was given. Exactly two FRONTIER TUs have `assign-store-type` as their **only**
blocker key — `Sort.cpp` (1 site) and `negate_test.cpp` (2) — and it appears
elsewhere only inside `keygen_xbox.cpp`, at 2 sites of 10 keys. **All five sites
are `8643`.** The code-pointer kind, 1,402 functions and 17 % of the old bucket,
touches no frontier TU at all; it could not have been seen before the split,
because the bucket it hid in did not distinguish it.

### 2.2 The zeros are DERIVED, and that is the seam's bound

In a real `T x; x = q;` the operand and the destination carry the same type and
the operand LOAD is parsed **first**. `eat_operand_type` admits exactly three
classes — 4-byte integer, 4-byte pointer, 1-byte unsigned — so every other type
refuses one token earlier and lands in `expr-load-type-*` instead. Captured, per
type:

```text
  float x;        x = f;   ->  expr-load-type-8645    (never reaches the store)
  double x;       x = d;   ->  expr-load-type-8885
  signed char x;  x = c;   ->  expr-load-type-8211
  short x;        x = s;   ->  expr-load-type-8421
  unsigned short / wchar_t ->  expr-load-type-8422
  long long x;    x = q;   ->  expr-load-type-8881
  volatile int x; x = v;   ->  expr-load-type-9641    (the volatile-spill gate)
```

So `assign-store-type` can only ever name a type that clears the operand gate and
fails `eat_int_like` — the pointer classes and the 1-byte unsigned one.
**The three keys above are the whole set.** There is no tail to discover here,
and that is itself the bound on what widening this seam could be worth.

**This was found by the tests refuting the first draft of themselves.** That
draft varied one type and wrote it into *both* slots, which cannot separate "the
store type is unmodeled" from "the operand type is". It asserted
`assign-store-type-8645` for a `float` and got `expr-load-type-8645`, and
`assign-dst-not-formal-0x26` for a `volatile int` and got `expr-load-type-9641`.
The corrected test — `everything_outside_the_operand_vocabulary_refuses_at_the_load_instead`
— is that refutation kept as a check rather than as a sentence.

## 3. The widening: DECLINED on a measured +0, twice

Two counterfactual builds, each a full 878-TU scan, each reverted after reading.

**Counterfactual 1 — admit a width-4 pointer store type** (`eat_int_like` →
`eat_int_like_or_ptr4` at the store: the one-line widening this lane was sent to
price):

```text
  census 706,555 -> 706,555  (+0)    TU match 8 -> 8    FRONTIER 19 -> 19
  -6,820  assign-store-type-8643  ->  +3,855  expr-op-0x60
  -1,402  assign-store-type-8644      +3,086  expr-jump
                                        +722  expr-op-0x10
                                        +451  expr-op-0x27
                                        + 51  rows in a tail of 1s and 2s
```

**Counterfactual 2 — lift the store-TYPE gate ENTIRELY** (any decodable type
consumed), which prices the whole 10,128-row family in one number:

```text
  census 706,555 -> 706,555  (+0)    TU match 8 -> 8    FRONTIER 19 -> 19
  +4,035 expr-jump · +3,855 expr-op-0x60 · +809 recv-load-then-call-other
  +722 expr-op-0x10 · +480 expr-op-0x27 · 74 further rows
```

**The realizable worth of the entire `assign-store-type` family is exactly zero
functions and zero TUs.** Not one workload function has a store type as its only
remaining blocker.

Two things about this are worth more than the decline itself:

* **It refuted my own model of the mechanism.** I predicted the rows would land
  on `assign-dst-not-formal`, because the destination is a pointer local and
  `.sy`'s local whitelist is plain-`int`-only. They do not: the statement loop
  runs on to the *next* statement and that statement's expression refuses first,
  because the destination gate is deliberately deferred to last (WAE,
  `docs/rungs/2026-07-31-assign-eof.md`). The prediction was wrong in its
  mechanism and right only in its number, and only the counterfactual could tell
  the two apart.
* **It reproduces that rung's counterfactual B to within one row.** It recorded
  `+4,034 expr-jump, +3,855 expr-op-0x60, +809 recv-load-then-call-other,
  +722 expr-op-0x10`; counterfactual 2 measures `+4,035 / +3,855 / +809 / +722`,
  five days and 27 merges later. A four-day-old negative result held across the
  whole intervening tree.

**No byte-emitting change is shipped, so no obj can be wrong in the direction the
correctness rule exists to forbid.** The counterexample grid that would have
gated such a change is §2.2, where it did its work as a refutation instead.

## 4. The two frontier TUs, priced from their real objs — both DECLINED

Board **#269**: a frontier TU at ≥4 independent unmodeled constructs is not a
target. Reference objs from `work/w-frame/refobj.sh` at the workload's own flags,
disassembled with `scripts/gt_dump.py`.

### 4.1 `src/system/math/Sort.cpp` — `?HashString@@YAHPBDH@Z`, ≥8

`int HashString(const char *str, int i)` — 20 instructions, `.text` only, no
relocations. Distinct constructs the port does not model:

1. **A loop**, and a rotated one: guard `bt 2,.+56` at the top, back-edge
   `bf 2,.-48` at the bottom. The port emits no loop of any kind.
2. **`lbzu 10,1(9)`** — a narrow indirect load in *update* form, fusing the `u++`
   into the load's addressing mode.
3. **A pointer induction variable carried across the back edge** (`mr 9,3` into
   the `lbzu` chain).
4. **`mr. 11,10`** — a record-form move setting cr0 as the loop condition.
   Record forms are emitted nowhere in the port.
5. **`divw` / `mullw` / `sub`** — the signed `%` expansion.
6. **`twi 6,4,0` and `twi 5,6,-1`** with their `rotlwi`/`addi`/`andc` predicate —
   the `/O1` divide-by-zero and `INT_MIN/-1` trap idiom, a family absent from the
   port and from `docs/`. Decoded from the raw words rather than read off a
   mnemonic, since the `TO` field is where the whole meaning lives:

   ```text
     0x38  0cc40000  op=3(twi) TO=00110 rA=r4 SIMM=0    trap if the divisor is 0
     0x40  0ca6ffff  op=3(twi) TO=00101 rA=r6 SIMM=-1   trap if the guard is -1
   ```
7. **`mulli 8,10,127`** in a loop-carried position.
8. **The schedule** — `mulli` hoisted above the `lbzu`, the divide interleaved
   with the trap computation. Byte-exactness needs c2's scheduler, which is
   qualitatively harder than any single item above.

`docs/rungs/2026-08-04-w-conv.md` §2 prices this TU at **7** by a different
partition. Independently re-derived here at **≥8**. Either is ≥4 twice over.

### 4.2 `src/system/negate_test.cpp` — both functions, ≥10

`FindNodeA` and `FindNodeB` emit **byte-identical** 80-byte bodies (the
`!(a != b)` / `a == b` fold), so this is one price paid twice. Distinct
constructs:

1. **A frame** — `mflr 12 · stw 12,-8(1) · stwu 1,-96(1)` and its epilogue, for
   two `bl`s rather than one.
2. **Two distinct callees on divergent arms, called with `bl` and joined** — not
   the two-tail-call `cond-tail-pair` shape W8 built.
3. **A value-carrying diamond join** — `mr 11,3` after both arms, then `mr 3,11`.
4. **`cmpwi 6,10,1` — a comparison into cr6**, not cr0.
5. **One comparison feeding two branches on different bits** — `bt 24,.+32`
   (cr6.lt) and `bt 26,.+28` (cr6.eq) share a single `cmpwi`. Nothing in the port
   shares a comparison across two source-level `if`s.
6. **`.pdata`** — the section, the unwind word `40 00 14 03`, and an ADDR32 reloc.
7. **`$M` LABEL symbols** — `$M2581` at 0xc and `$M2582` at 0x50, in the symbol
   table, per function.
8. **The label counter's *gaps*** — fn A takes 2581/2582/2583 and fn B takes
   2587/2588/2589, so the counter advances by 6 while 3 symbols are emitted. The
   unemitted allocations have to be modeled to get the numbers right.
9. **`_fltused`** — a whole-TU external that the `float` parameter forces in.
10. **Argument pre-marshalling hoisted above the branch** — `mr 10,3 ; mr 3,4`
    parks the enum and makes `clip` call-ready before either arm is taken.

`w-conv` §2 and `w-cross` both price this TU at **9**, by two partitions that
differ from each other and from mine. Third independent derivation, **≥10**.

### 4.2a Re-derived against the frontier reprice, which says 4 — and the eight decisions it cannot see

A concurrent repricing lane puts `negate_test` at **4**, the cheapest genuinely
buildable TU on the frontier, by the partition
`price = |IL| + |HARD| + |SOFT|` over mnemonics and parse productions. My count
above is 10 and w-conv's is 9. Three counts that do not agree are worth more than
three that do, so here is mine restated in *the reprice's own taxonomy*, so the
disagreement is about the world and not about the vocabulary:

| | construct | why |
|---|---|---|
| **IL** | the `cflow-if-n` statement production with a **value-carrying** join | nothing parses a diamond whose arms both assign |
| **IL** | the **pointer-typed local** as an assignment destination | `.sy`'s `locals` admits plain `int` only (§6.2) |
| **IL** | a **call result stored into a local in a non-first statement** | `assign.rs` refuses `assign-rhs-call-0x26` once `env` is non-empty |
| **HARD** | a comparison into **cr6** | the port targets cr0 and nothing else |
| **HARD** | `.pdata` + the unwind word `40 00 14 03` + its ADDR32 reloc | no mechanism |
| **HARD** | `$M` **LABEL-class symbols** in the symbol table | `$M2581` @0xc, `$M2582` @0x50, per function |
| **HARD** | the label counter's **unemitted allocations** | see below |
| **HARD** | **`_fltused`** | a TU-level external the `float` parameter forces in |
| **SOFT** | the two-call frame | `FramedCall` exists; no production routes two calls through it |
| **SOFT** | argument marshalling across a branch | the mechanism exists for the straight-line case |

Ten, by a fourth cut — and note that §4.2's ten and this table's ten are **not
the same ten**. The shared comparison is a construct in §4.2 and a *decision*
below; the value-carrying join is one IL row here and two rows there. Two
partitions of the same TU landing on the same total with different membership is
the cross-check w-conv §2 says is the evidence worth having, and it is the same
relationship w-conv has with w-cross.

The reprice's 4 and my 10 are **not really in conflict**:
its own calibration says the metric *cannot see a selection, allocation or
scheduling decision*, and `?GetXAllocAttributes` is its worked example — four
independent facts, one mnemonic. `negate_test` is that case at scale.

**The eight decisions the mnemonic count is blind to, enumerated because that is
the metric's stated blind spot and this is the TU it matters for.** Every branch
word below was hand-decoded from the raw instruction rather than read off a
disassembler's mnemonic, because `llvm-mc` drops the CR field when it is cr0 and
a naive `ops[0]` reads `crf=3` from `cmplwi 3,0`. These are all `crfD=6`,
explicitly encoded:

```text
  0x18  2f0a0001  cmpi  crfD=6 rA=r10 SIMM=1
  0x1c  41980020  bc    BO=12(bt) BI=24 -> cr6.LT  disp=+32
  0x20  419a001c  bc    BO=12(bt) BI=26 -> cr6.EQ  disp=+28
```

1. **Which CR field.** cr6, not cr0 — a field-*allocation* decision, and no
   mnemonic distinguishes it.
2. **The comparison is CSE'd across two source-level `if`s.** The source tests
   `blendMode >= kPlayNoBlend` and then `blendMode == kPlayNoBlend`; c2 emits
   **one** `cmpwi cr6,r10,1` and reads two different bits of it (LT at 0x1c, EQ
   at 0x20). Nothing in the port shares a comparison.
3. **`mr 3,4` is hoisted above the branch** — `clip` is marshalled into the
   argument register on the entry path, before it is known whether *either* call
   is reached. Code motion.
4. **`n` is home-allocated to r11 and round-tripped.** `li 11,0` … `mr 11,3`
   (0x38) … `mr 3,11` (0x3c). On the call path that pair is a no-op — the value
   is already in r3 — and c2 emits it because the join reads the *home*. Two
   instructions produced by one allocation decision; w-conv folds exactly this
   pair into its register-home row.
5. **`blendMode` is evicted from r3 into r10** (`mr 10,3`) to free the argument
   register, though it is read only twice.
6. **Block layout: which arm becomes the fall-through.** `FindLast` falls through
   at 0x2c and `FindFirst` is the branch target at 0x34 — the *inverse* of source
   order. This choice sets the polarity of `bt 24,.+12` and every displacement in
   the function.
7. **Where the join block begins** — 0x38 vs 0x3c — which is what makes the
   `b .+8` at 0x30 an 8 and not a 4.
8. **The 96-byte frame size.** No local is stored in it and no GPR is saved; 96
   is not derivable from anything visible in the body.

**And one new fact, which is decision 7's fingerprint.** The `$M`/`$T` counter
runs 2581/2582/2583 for `FindNodeA` and 2587/2588/2589 for `FindNodeB` — it
advances by **6** while **3** symbols are emitted per function. The three
unemitted numbers are exactly the three internal branch targets of the first
function (0x34, 0x38, 0x3c), none of which gets a symbol. So the counter appears
to be allocated per **basic-block boundary** and emitted only at the body start
and the body end, which means **the `$M` numbers of the second function are a
function of the first function's block layout**. Consistent with this witness;
n = 1, so it is a hypothesis with a mechanism, not a law.

**Conclusion on the lead's question: my independent count is 10, it agrees with
w-conv's 9 to within one and by a different cut, and it does not reproduce 4.**
Even taking the reprice's 4 at face value, board **#269** fires *at* 4 and the
decline stands; and the reprice's own disqualifying condition — "if
`negate_test`'s 4 hides even one selection, allocation or scheduling decision it
is a decline again" — is met **eight** times over. `negate_test.cpp` is not a
target.

### 4.3 A prior-art claim, verified rather than inherited — and partly WRONG

`docs/rungs/2026-08-04-w-cfgimpl.md` §6 item 2 states that all five
single-blocked-function frontier TUs, `negate_test` among them, are *"framed,
with data-symbol `REFHI`/`REFLO` pairs, cr0 record-form branches, stack locals,
`srawi`/`mulli`/`lwzx`, or `__savegprlr_26`."* **Its dump was never committed.**
Reproduced here at the workload's flags:

* **`framed` — TRUE.** `mflr` / `stw -8(1)` / `stwu -96(1)`, confirmed.
* **Every one of the six named markers — ABSENT from `negate_test`.** No `REFHI`
  or `REFLO` reloc (the obj has 6 relocations: 4 REL24 to the two callees and 2
  ADDR32 in `.pdata`); no cr0 branch (both compares go to **cr6**); no stack
  local (nothing is stored in the 96-byte frame — the saved LR at `-8(1)` is
  outside it); no `srawi`, `mulli` or `lwzx`; no `__savegprlr_26`.

The conclusion — that the `cflow-*` class was never these TUs' real distance —
**survives**, and is if anything strengthened: `negate_test`'s real distance is
the cr6 CSE'd compare, the value-joining diamond, `_fltused` and the `$M`/`$T`
label numbering, none of which is on that list. Recorded so the next lane reads
that list as *illustrative of five TUs collectively* rather than as a per-TU
claim about any one of them.

## 5. Gate evidence

| lane | result |
|---|---|
| `cargo test --workspace --release` | **809 passed / 0 failed / 27 targets** (baseline 806 / 0 / 27). The target count is quoted because `cargo test` stops at the first failing target, so a truncated run reports fewer passes *and* fewer targets and reads as a smaller passing run. |
| `scripts/gate.sh --jobs 6` | **GATE: PASS** — 18 in the registry, **18 PASS / 0 FAIL / 0 SKIP / 0 NO-RESULT**; **4,410 fixture-verdicts**; sweep 16,394 of 16,394 reached, **16,298 graded**, 96 ungraded, **0 mismatch**; cross 76,217 of 76,217 cells, **75,829 graded**, 388 ungraded, **0 mismatch**. Quoted from the gate's own summary lines, not transcribed from a lane list. |
| 878-TU workload scan | match **8** / mismatch **0** / codegen-gap **0** / vocab-gap **863** / capture-fail **7** / census **706,555 / 2,463,393** / disagreement **0** |
| fixtures, `c2rs census` | n/a — this rung ships no fixture and admits no class |

**Both must-hold ungraded figures held: the sweep's 96 and the cross's 388.** The
388 is board **#294**'s cold-cache discriminator; if it moves, suspect the capture
cache before the code. Per-lane fixture match is unchanged from the incumbent at
every lane — 118 at the eight `/O1`/`/Ox`/`/O2` lanes and their `/GR` variants,
116 at the two `/Gy` lanes, 10 at the four `/Od` lanes.

> `scripts/expr_sweep.sh` reads `c2-core` only and this rung's change is entirely
> in `c2-il`, so the sweep's 16,298 graded cases say **nothing** about it. The
> checks that do cover it are the three unit tests in §2 and the two full-workload
> counterfactual scans in §3 — a green sweep here is a control, not evidence.

## 6. Found and not taken

1. **`call-bound-store-0x86` (30 functions) has the identical defect.**
   `crates/c2-il/src/func/body/shapes/calls.rs:1157` raises the same `blk` on the
   same `eat_byte(0x32) || eat_int_like` pair, so it too prints the tag and not
   the kind. The whole `call-bound-store-*` family is 42 functions, 30 of them in
   the `0x86` row. Left alone deliberately: `calls.rs` is not this subagent's seam
   and 42 rows do not justify a cross-seam edit while a sibling lane is live. One
   line, the same shape as §2.
2. **`.sy`'s local whitelist is plain-`int`-only**, and that — not the store type
   — is the gate a pointer store would ultimately have to clear. Captured here as
   a side effect: `unsigned`, `long`, `unsigned long`, `enum` and `const int`
   locals all pass the store type and refuse at `assign-dst-not-formal-0x26`,
   while a plain `int` and a `typedef int` are accepted. Widening `.sy` to
   pointer locals is a real seam; §3 measures it at +0 *on its own*, and it would
   have to be paid together with whatever the next statement blocks on.
3. **The frontier ranking is by blocked-function count and that is still the
   wrong key** — fourth lane in a row to find this. `Sort.cpp` and
   `negate_test.cpp` sit at 1 and 2 blocked functions, the cheap end of the
   FRONTIER table, and price at ≥8 and ≥10.

## 7. Board rows PROPOSED, not minted

Numbers `#406`–`#411` are the range this subagent was allotted. **`docs/BOARD.md`
is not edited here** — the lead mints these, or renumbers them, in one place.

| # | row | state |
|---|---|---|
| 406 | `assign-store-type`'s census key printed the type's **tag** — the slot's width — and not its kind, so 8,222 rows named none of their contents. Split at `blk_type` into `8643` / `8644` / `8212`. | **CLOSED** by this rung |
| 407 | The **whole** `assign-store-type` family (10,128 functions) is worth **exactly 0 functions and 0 TUs**, by two full-workload counterfactuals. Not one function has a store type as its only blocker. **Do not rank it.** Reproduces `2026-07-31-assign-eof.md` counterfactual B to within one row, 5 days and 27 merges later. | **CLOSED — a decline with a number** |
| 408 | That key's vocabulary is **closed at three kinds** by the upstream operand gate: `eat_operand_type` admits int4 / ptr4 / int1u, so anything else refuses one token earlier as `expr-load-type-*`. The census is the whole set, not a sample — a future lane needs no re-measurement. | **CLOSED** |
| 409 | `call-bound-store-0x86` (30 functions) has the **identical** tag-not-kind defect at `crates/c2-il/src/func/body/shapes/calls.rs:1157`. One line, same shape as #406. Not taken: wrong seam, live sibling lane. | **OPEN** |
| 410 | `2026-08-04-w-cfgimpl.md` §6 item 2's marker list (`REFHI`/`REFLO`, cr0 record-form, stack locals, `srawi`/`mulli`/`lwzx`, `__savegprlr_26`) **does not describe `negate_test.cpp`** — verified against the reference obj, which has none of the six. Its `framed` claim and its conclusion both survive; read the list as illustrative of five TUs collectively. | **CLOSED — a correction** |
| 411 | `Sort.cpp` ≥ **8** and `negate_test.cpp` ≥ **10** independent unmodeled constructs, from the real objs at the workload's flags. Both **DECLINED** under #269. Third independent derivation of `negate_test`; `w-conv` and `w-cross` say 9 by two other partitions. | **CLOSED — a decline with a price** |

## 8. The capture probe, in full

Reproduce §2.1 and §2.2 with:

```sh
cp <this block> $C2RS_DC3/probe_storetype.cpp
c2rs census probe_storetype.cpp --flags-file work/dc3-workload/flags.txt \
    --cwd $C2RS_DC3
```

```cpp
struct S { int a; };
enum E { E0 = 0, E1 = 1 };
typedef int TdInt;
typedef unsigned char Byte;
typedef int (*FnPtr)(int);

int g_target(int);

// --- the two kinds the workload census actually reports at tag 86 -------------
int st_dptr(int *q, int k) { int *x; x = q; return k; }
int st_cvptr(const int *q, int k) { const int *x; x = q; return k; }
int st_vptr(void *q, int k) { void *x; x = q; return k; }
int st_sptr(S *q, int k) { S *x; x = q; return k; }
int st_fnptr(int k) { FnPtr x; x = g_target; return k; }
int st_fnptr_p(FnPtr q, int k) { FnPtr x; x = q; return k; }

// --- tag 82, the one the census reports there --------------------------------
int st_bool(bool b, int k) { bool x; x = b; return k; }
int st_uchar(unsigned char c, int k) { unsigned char x; x = c; return k; }
int st_byte_td(Byte c, int k) { Byte x; x = c; return k; }

// --- the ones the census reports as ZERO. Probed so the zero is a measurement,
//     not an absence: each of these must produce a DISTINCT key here.
int st_schar(signed char c, int k) { signed char x; x = c; return k; }
int st_char(char c, int k) { char x; x = c; return k; }
int st_short(short s, int k) { short x; x = s; return k; }
int st_ushort(unsigned short s, int k) { unsigned short x; x = s; return k; }
int st_wchar(wchar_t w, int k) { wchar_t x; x = w; return k; }
int st_float(float f, int k) { float x; x = f; return k; }
int st_double(double d, int k) { double x; x = d; return k; }
int st_longlong(long long q, int k) { long long x; x = q; return k; }

// --- the ones that must be ACCEPTED (int-like), so the accept side is exercised
int st_int(int a) { int x; x = a; return x; }
int st_uint(unsigned a) { unsigned x; x = a; return (int)x; }
int st_long(long a) { long x; x = a; return (int)x; }
int st_ulong(unsigned long a) { unsigned long x; x = a; return (int)x; }
int st_enum(E e, int k) { E x; x = e; return k; }
int st_tdint(TdInt a) { TdInt x; x = a; return x; }
int st_constint(int a) { const int x = a; return x; }
```

`st_fnptr` (the `x = g_target` form) blocks *earlier*, at
`expr-call-in-expr-data-addr-whole`, because taking a function's address is its
own construct — its store triple `86 44 88 20` is still visible in the census's
byte window. `st_fnptr_p`, which takes the function pointer as a *parameter*,
reaches the store and is the direct `assign-store-type-8644` witness. Both are
kept: the pair is what shows the key is about the destination's type and not
about how the value was obtained.
