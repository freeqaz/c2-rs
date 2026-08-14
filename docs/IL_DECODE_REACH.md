# The statement-layer decode reach, and the two opcodes that were holding it

**WDR, 2026-07-31.** Census delta **0** — this is a measurement, not a rung, on
the model of `docs/rungs/2026-07-31-cflow-decode.md` and `docs/EH_RECORDS.md` §7.
It admits nothing, it lowers nothing, and every body it newly understands still
returns `NotImplemented`. It belongs in `docs/` proper for exactly that reason.

Corpus: the 878-TU `dc3-decomp` workload, 2,462,571 IL functions, at the
workload's own flags (`/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc`). Scans
`work/WDR/scan-base.jsonl` (baseline, same commit) and `work/WDR/scan-wdr.jsonl`.

> **Bodies decoded end to end: 2,129,811 (86.5 %) → 2,318,605 (94.2 %), +188,794.**
> **`eh-unknown`: 288,072 → 137,187, −150,885 (−52.4 %).**
> Census numerator **685,165, +0**; census/gate disagreement **0**; the
> `fn_blockers` histogram is **byte-identical, 720 keys before and after, every
> delta zero** — so there is nothing to rename and no comparison is invalidated.

---

## 1. What the two rows were

`docs/EH_RECORDS.md` §7.6 named them: *"Establishing `0x64` (145,237 bodies) and
`0x67` (45,631) — now the two largest rows on the control-flow axis — is what
would shrink [`eh-unknown`]."* Both were listed unidentified in
`docs/IL_CALL_GRAMMAR.md` §7, and `control_flow.rs` refused at them by policy
rather than by accident: a first cut of that file had given `67` a TYPE on the
strength of its neighbours' shape and failed at a non-tag byte in 29,687 bodies.

`eh-unknown` is not an independent population, and the identity is what made this
schedulable at all:

```
eh-unknown  =  (bodies that do not decode)  −  eh-partial
288,072     =   332,760                     −   44,688        (exactly)
```

A body is `eh-unknown` iff its walk stopped **before** meeting a `5C`/`5D`/`5E`.
So decode reach is the only lever on it.

## 2. `67 <varint vtable-byte-offset> <token>` — virtual dispatch

Reproduced from hand-written source before anything was sized
(`work/WDR/probe/p1.cpp`), at the workload's flags:

```cpp
struct V { virtual int VGet(); virtual int VSet(int); virtual ~V(); int x; };
int v_stmt(V* p) { return p->VGet(); }        // census key body-0x67
int v_rhs(V* p)  { int x; x = p->VGet(); return x; }   // expr-op-0x67
```

```
67 00 e4 09              dispatch, vtable byte offset 0, method token
b9 f6 09 86 43 81 20     load p
30 a6 43 85 20           -> the vtable pointer
30 86 43 99 20           -> the slot
9a 86 43 86 20           bind it
bd 86 41 74 00 80 06 10 00 00  4c
```

The first field is the **byte** offset, not the slot index: four virtuals in
declaration order emit `00`, `04`, `08`, `0C`.

### 2.1 The field is a signed varint, and only one probe can say so

Every witness this project had — the six in `IL_CALL_GRAMMAR.md` §7 and the four
above — is below `0x80`, where a plain byte and a signed varint are **the same
bytes**. `work/WDR/probe/p3.cpp` separates them with a class carrying forty
virtual functions:

| source | emitted |
|---|---|
| `int w31(Wide* p){ return p->v31(); }` | `67 7C 03 0A` — offset 124, short form |
| `int w32(Wide* p){ return p->v32(); }` | `67 80 80 00 00 00 04 0A` — offset **128, escaped** |
| `int w39(Wide* p){ return p->v39(); }` | `67 80 9C 00 00 00 0B 0A` — offset 156 |

One digit in the source, four bytes in the field. A plain-byte reading
desynchronizes on every class in this corpus with more than 32 virtuals, and the
workload prices that at **926 bodies** (§5).

## 3. `9A <TYPE>` — the vtable-slot bind, and why `67` alone is worth nothing

`9A` is the virtual sibling of `99`. Its width is **not** inferable from `99`'s
`<TYPE> <varint>`, and the corpus is emphatic about which it is: a trailing
varint swallows the `BD` that follows at every site, leaving the walk on a TYPE
tag, which is not an operand opcode. On the workload, `9A <TYPE>` decodes
**69,246** bodies that `9A <TYPE> <varint>` does not.

It matters here because of what it does to the ranking of `67`:

> **Decoding `67` and nothing else moves the decode reach by ZERO.** The scan
> reports 2,129,811 bodies decoded — the baseline value, to the function. The
> 45,631-body `cf-expr-0x67` row simply becomes a 45,631-body `cf-expr-0x9A` row
> two tokens later.

This is `ROADMAP.md` §6n's rule — *a large blocking row is one of five things and
a first-blocker histogram distinguishes none of them* — arriving as a **predicted**
null rather than a discovered one. The prediction is in `work/WDR/ESTIMATE.md`,
written before the workload was scanned, under the heading "the one thing I
expect to be wrong about". `cf-expr-0x9A`'s first-blocker row at the time was
**222**.

## 4. `64 <TYPE>` — the by-value return's materialize

`docs/IL_CALL_IN_EXPR.md` §14.6 recorded `0x64` as appearing at
`op-0x9B` sites and undecoded. Found by **elimination**, not by inspection:
`work/WDR/probe/p2.cpp` is a 27-function battery — `static_cast` up and down,
`dynamic_cast`, `const_cast`, `reinterpret_cast`, pointer-to-member data and
function, `new`, `new[]`, `delete`, `delete[]`, a call through a function
pointer, aggregate assignment, an aggregate copy return, a by-value argument, a
global address, an object-array subscript, a reference formal. **Exactly one
function in it emits `64`:**

```cpp
struct A { int x, y; };
struct B { A Val(); … };
void c_val(B* b) { b->Val(); }        // Val() returns a class BY VALUE
```

```
26 f0 09                       push Val
b9 37 0a 86 43 aa 20           load b
99 86 43 89 20 00              member bind
bd 86 43 8a 20 00 80 09 10 00 00   call, returning A*
9b 86 86 84 20 3a 0a           bind the temporary
2c 86 43 8a 20 00              its address
64 86 43 8a 20                 MATERIALIZE into it
4c
30 86 86 84 20  4b             read it back, end of statement
```

`64` sits in the slot a `BD` occupies and is closed by the same `4C`. It carries
a result TYPE and no trailing field, on the model of `40 <TYPE result>` (the
intrinsic call).

### 4.1 …and that last sentence is INDISTINGUISHABLE from one alternative

`4C` follows `64 <TYPE>` at every witnessed site — twenty of them across probes
and wild TUs — so a `64 <TYPE> <varint>` reading swallows the `4C` and reaches
the same function tail. Over the whole workload the two readings differ by
**two bodies in 2,462,571**.

That is the same standing `99`'s trailing `00` has (`IL_CALL_IN_EXPR.md` §3:
*"INDISTINGUISHABLE from a constant here"*), and it is recorded rather than
dressed up. The reason to prefer `64 <TYPE>` is structural, not statistical:
`4C` is an opcode of this grammar in its own right, with its own arm, and it
closes a call's operand region — so it is not `64`'s payload. Requiring it
literally was tried and rejected: it refuses 3 bodies per 838k at sites whose
preceding `30 <TYPE>` is itself unusual, i.e. it converts an ambiguity into a
row that measures something else.

## 5. The variant table — one thing changed at a time, on the whole workload

Every row is a full 878-TU scan. `mismatch 0` and census `685,165` in all of them
(the scanner cannot affect either, and the rows are the proof rather than the
claim).

| variant | bodies decoded end to end | vs baseline |
|---|---:|---:|
| baseline — none of the three | 2,129,811 (86.5 %) | — |
| **`67` alone** | **2,129,811** | **+0** |
| `67` + `9A` | 2,167,450 (88.0 %) | +37,639 |
| `64` alone | 2,249,358 (91.3 %) | +119,547 |
| **all three, as implemented** | **2,318,605 (94.2 %)** | **+188,794** |
| `67`'s slot read as a plain byte | 2,317,679 | −926 |
| `9A` read as `<TYPE> <varint>` | 2,249,359 | −69,246 |
| `64` read as `<TYPE> <varint>` | 2,318,603 | −2 |

Two things in that table are worth more than their row.

1. **The three are super-additive.** 37,639 + 119,547 = 157,186, against 188,794
   together: **31,608 bodies contain both constructs and neither opcode alone
   moves one of them.** A ranking that scheduled them as two independent rungs
   would have under-priced the pair by 17 %.
2. **Realization against the row.** `cf-expr-0x64`'s 145,237 realizes 119,547
   alone (82.3 %); `cf-expr-0x67`'s 45,631 realizes 0 alone and 37,639 with `9A`
   (82.5 %). Together the two rows sum to 190,868 and realize **188,794 = 98.9 %**
   — *the ceiling taken neat was right to within 1.1 %, and both of the discounts
   applied in the estimate were wrong.* That is the fourth time this board has
   recorded that shape.

**The two new productions' own residue is 183 bodies.** `cf-vbind-type-*` 180
(129 of them a `3A` where a TYPE was expected) and `cf-materialize-type-*` 3, out
of 190,868 bodies newly walked through — **0.096 %**.

> **REFUTED, 2026-07-31 (WVB, `docs/IL_TYPE_WIDE_TAG.md`).** This paragraph went
> on to say *"they are honest refusals, not desyncs"*. **All 183 were desyncs** —
> and not of `67`, `9A` or `64`: `read_type` was reading a five-byte type as
> three, so the walk resumed two bytes early on whatever stood there. The
> falsification test (land exactly on the seven-byte tail, every `54 <k>` depth
> agreeing) is sound and those bodies really did fail it; what it cannot do is say
> **where** a desync started, and this residue was read as though it could. All
> 183 now decode.

## 6. What the newly-classified bodies are

| EH class | before | after | delta |
|---|---:|---:|---:|
| `eh-none` | 1,864,128 | 2,009,514 | **+145,386** |
| `eh-unknown` | 288,072 | **137,187** | **−150,885** |
| `eh-plus-stmt` | 160,944 | 196,138 | +35,194 |
| `eh-bare` | 76,845 | 77,147 | +302 |
| `eh-partial` | 44,688 | 6,779 | −37,909 |
| `eh-multi` | 27,894 | 35,806 | +7,912 |

188,794 bodies changed class: 150,885 out of `eh-unknown` and 37,909 out of
`eh-partial`. An `eh-partial` body carries a marker by definition, so **none of
the +145,386 `eh-none` can have come from it** — every one came from
`eh-unknown`. Therefore:

> **Of the 150,885 bodies that left `eh-unknown`, at least 145,386 (96.4 %) carry
> no EH marker at all.** At most 5,499 of them are on the EH side or bare.

**This is a correction to how §6o's conclusion should be read, and it is a large
one.** The EH side (`plus-stmt` + `partial` + `multi`) grows from 233,526 to
238,723 — **+5,197, 2.2 %** — on a population of 188,794 newly legible bodies.
The 288,072 unmeasured bodies §6o flagged as *"larger than either side"* were
**not** hidden EH stock. Virtual dispatch and by-value returns are ordinary
expression work, and the axis now says so instead of declining to say.

What it does **not** change: EH is still 238,723 functions behind a model that
does not exist, and the cheap side is still 40,881. §6o's *phase* conclusion
stands; only its largest open risk is closed.

## 7. The control-flow shapes, re-measured

| shape | before | after | of which blocked on control flow ALONE |
|---|---:|---:|---:|
| `cflow-straight` | 1,517,863 | 1,650,903 | 276,271 → **276,271** |
| `cflow-if-1` | 229,995 | 234,254 | 713 → **713** |
| `cflow-if-2` | 25,883 | 28,903 | 0 → 0 |
| `cflow-if-n` | 9,255 | **43,335** | 0 → 0 |
| `cflow-loop` | 69,687 | 83,948 | 0 → 0 |
| `cflow-switch` | 139 | 273 | 5 → **5** |
| `cflow-multi-exit` | 0 | 0 | 0 |
| undecoded (`cf-*`) | 332,760 | **143,966** | — |

Two results.

1. **The block IR is still worth exactly 718 functions**, and the `+expr-modeled`
   column is unchanged *to the function* in every row. That is not a coincidence
   and it is not luck: `67`, `9A` and `64` each call `off_class()`, so no body
   they unblock can be "waiting on control flow alone". A reader hoping that 190k
   more decoded bodies would move the block-IR rung's price gets a measured **no**.
   `docs/rungs/2026-07-31-cflow-decode.md`'s item 4 is unchanged.
2. **The newly legible population is markedly branchier than the old one.**
   `cflow-if-n` **quadrupled** (9,255 → 43,335) and `cflow-switch` doubled
   (139 → 273), against `cflow-straight` growing 8.8 %. Bodies with virtual calls
   and by-value temporaries are bodies with several conditionals — which is a fact
   about what a block IR would have to serve that no previous scan could see,
   because these bodies stopped decoding before their second branch.

The largest remaining undecoded rows, and what establishing each would buy —
**three of the six rows below do not exist.** `82`, `80` and most of the long tail
were the second byte of a wide type's id, and go to **0** under
`docs/IL_TYPE_WIDE_TAG.md`'s width; the residue is 384 keys → 8 and 143,966
bodies → 68,233. `05`, `59`, `60` and `08` are real and survive to within 0.4 %.

The largest remaining undecoded rows, and what establishing each would buy:

| opcode | bodies | note |
|---|---:|---|
| `05` | 32,755 | presumably `/`; `IL_STMT_GRAMMAR.md` §5's table stops at `%` = `06` and does not say |
| `82` | 23,254 | in §13's residue list |
| `59` | 16,016 | appears between two FP arithmetic ops |
| `80` | 14,185 | |
| `60` | 9,665 | `try`/`catch` — `body-0x60` in the census, EH by construction |
| `08` | 8,242 | an unwitnessed operator-table slot |

The whole remaining undecoded residue is 143,966 = **5.8 %** of the workload, and
the head row is now under a fortieth of what `64` was.

## 8. Fixtures, sweep, and the separation counts

`fixtures/cpp/wdr_virtual_byval.cpp` — 15 functions, **0/15 in class, 15/15
decoded end to end**. `fixtures/cpp/wdr_neighbours.cpp` — 11 functions, **5 in
class (the control group), 11/11 decoded**. The two are graded on the pair of
columns: a case that decodes but does not refuse would mean the scanner had been
wired into acceptance.

`scripts/sweep.d/93-virtual-byval.py`, **116 cases**, all of which must come back
SKIP; a MATCH is the alarm. Separation measured per rule on
`work/WDR/probe/sep.cpp` (one function per variant, one capture):

| rule varied | cases | distinct outcomes | note |
|---|---:|---:|---|
| `virtual` vs not, same class, same call site | 2 | **2** | and one side is **byte-exact today** (`int-tail-call`, in class) while the other refuses at `body-0x67`. The strongest kind of separation this repo has: the axis straddles the emitter. |
| vtable slot below / at / above byte offset `0x80` | 7 | **2 encodings** | worth **926 bodies** on the workload (§5); on source, one digit changes four bytes |
| returned type category × `virtual` (int / by value / by value **with a destructor** / pointer / reference) | 10 | **7** distinct (census key, EH class) | the destructor alone flips `eh-none` → `eh-bare`, **2/2** — this is the one axis that crosses `docs/EH_RECORDS.md`'s boundary |
| what the by-value result is used for | 6 | **3** | discard/member/two-calls share a key; `use(...)` and `sink(...+1)` share another; a named local is a third |
| what the virtual result is used for | 5 | **2** | statement/return/operand are `body-0x67`; argument and assignment are `expr-op-0x67` |
| a virtual call or a materialized temporary **beside** an emitted shape | 16 | **0** | measured, not asserted: across all four neighbours in both orders, `f`'s key is `body-0x67` or `…-then-temp-bind` according only to which construct **`f`** has, and the neighbour moves it zero times. Stated plainly, this rule separates nothing and is not a ranking axis. It is the mis-emit guard — the whole-body parser is supposed to make the neighbourhood irrelevant, and **0 is the assertion that it does**. |

## 9. Estimate vs outcome

`work/WDR/ESTIMATE.md`, written after the three probes and **before** any
workload scan, naming what the bucket was already filtered by.

| quantity | estimate | outcome | bias |
|---|---|---|---|
| census numerator | 685,165, **+0** | 685,165, **+0** | none — not an estimate; no acceptance path is touched |
| census/gate disagreement | 0 | 0 | none |
| `fn_blockers` key set | unchanged, 0 renames | **720 → 720, every delta 0** | none |
| bodies decoded end to end | 2,290,000 (93.0 %), range 2,240,000–2,320,679, **ceiling 2,320,679** | **2,318,605 (94.2 %)** | estimate **LOW by 28,605**; the ceiling taken neat was low by **2,074 = 0.09 %**. Cause named in the estimate and applied anyway: I discounted for bodies re-stopping at a later undecoded opcode. They barely exist. |
| **`eh-unknown` shrink** | **−150,000**, range −100,000…−190,868 | **−150,885** | **0.6 % low.** Two discounts were applied and they cancelled: the borrowed `eh-partial` rate was wrong in one direction and the re-stop rate in the other. A cancelling pair is luck, not method, and the decode-reach row above is the honest reading of the same estimate. |
| split of the newly classified | `eh-none` ≥ 55 %, EH side ≤ 45 %; bias "`eh-none` called high" | **`eh-none` ≥ 96.4 %** | **badly LOW, and the stated bias was backwards.** I reasoned that dc3's by-value returns are dominated by classes with destructors. They are not — the overwhelming majority of the newly legible bodies carry no marker at all. This is the reasoning error worth keeping: I extrapolated a *type* property of one library idiom to a population defined by an *opcode*. |
| `67` alone is worth ~0 | stated as "the one thing I expect to be wrong about" | **+0, exactly** | the estimate's most valuable line, and the only one that needed no number |
| new `cf-*` keys | +2 to +4 | +22 rows, **+2 productions** (`cf-vbind-type-*`, `cf-materialize-type-*`) totalling 183 bodies | estimate **low on rows, right on productions** — a per-byte key expands to one row per witnessed byte, which is the design and which I forgot to count |

## 10. Gate evidence

| lane | result |
|---|---|
| `cargo test --workspace --release` | **498 pass, 0 fail** (494 → 498; the four new decode tests) |
| `c2rs bench` | **189 pass, 0 fail, 0 error** (187 → 189; the two new fixtures) |
| `scripts/mode_lane.sh` `/Ox` · `/O1` · `/O2` · `/Ox /Gy` | **89 / 87 / 87 / 87** match, **mismatch 0** in all four |
| `scripts/expr_sweep.sh` | **13,336 checked, 0 mismatches** (13,220 → 13,336) |
| `scripts/cross_sweep.sh` | **20,194 × 4, 0 mismatches** |
| 878-TU workload scan | match 6 · **mismatch 0** · census **685,165 / 2,462,571 (27.82 %)** · **disagreement 0** · binding violations 0 |
| **debug-build** 878-TU scan | **0 panics**, and 0 / 685,165 / 0 / 2,318,605 identical to release |
| census key drift | `fn_blockers` **720 keys → 720, every delta 0**; `fn_frames` likewise. **No rename.** |

## 11. Found and not taken

1. **`cf-expr-0x05` — 32,755 bodies, the new head row.** A quarter the size of the
   one just retired. `IL_STMT_GRAMMAR.md` §5's operator table has `%` at `06` and
   is silent about `05`; one probe per arithmetic operator settles it or refutes
   it. Decode-only again.
2. **`mcall`'s `op-0x64` / `op-0x67` keys still spell hex, and they should.** The
   statement-layer scanner decodes these opcodes; `mcall`'s own walk does not, and
   its keys name **its** vocabulary. Renaming them from here would be a census key
   change with no production behind it, which is the failure mode the naming rule
   exists to prevent. The rung that widens `mcall` renames them, with the 1:1
   proof.
3. **Virtual dispatch is now the largest fully-decoded thing with no lowering.**
   45,631 bodies whose entire production is understood byte for byte — receiver,
   two indirect loads, bind, `bctrl` — and which are `off_class` only because
   nothing emits them. Unlike `05`, this one is a *codegen* rung with the decode
   already done, and its shape is a single basic block. It is behind the frame
   spine (a virtual call is never a tail call from a leaf), so it is not next, but
   it is the first item on this list that would move the census.
4. **The `9A` residue's 129 `cf-vbind-type-cflow-jump`** — a `3A` standing where
   `9A`'s TYPE should be. Small, but it is the only row in this work that says a
   width might be incomplete rather than merely unimplemented, and it is the
   cheapest thing here to look at next. **TAKEN, and it was right for the wrong
   reason** (`docs/IL_TYPE_WIDE_TAG.md`): a width *was* incomplete, but not `9A`'s
   — `9A` has no second form and there is no virtual dispatch in those 129 bodies
   at all. The `9A` byte is the fourth byte of the preceding type.
5. **The by-value return's `9B`/`44`/`64` trio is decoded at the statement layer
   and still unnamed at the expression layer.** `IL_CALL_IN_EXPR.md` §14.6's
   "neither `9B`'s role nor `44`/`64` is decoded" is now two-thirds false; `44`'s
   meaning remains UNKNOWN (its width was already established) and nothing here
   tested it.

---

## 12. THE SINK INSTRUMENT FAMILY — the five, named in one place at last

*(Added by lane `w-read2`. Board **#3098** is open because the first four are
documented **only** in `crates/c2-il/src/func/body/expr.rs` doc comments,
reachable by no `docs/` grep and no board-topic search: `w-readphase` registered
a prereg against their existence after two hours of orientation and then found
them by reading the source. This section is the table that row asks for.)*

Every one is **env-gated, OFF by default, off on every gate lane and every
default scan**, consumes its token so the census reports the **successor**, and
pushes no `IlOp`.

| variable | board | what it sinks | where it is consulted |
|---|---|---|---|
| `C2RS_SINK_OFF_ADD_ARG=expr` | **#143** | `0x27`, the offset-add | `parse_expr_classed` |
| `C2RS_SINK_REL=expr` | **#420** | the relational family `1F`..`24` | `parse_expr_classed` |
| `C2RS_SINK_BRANCH=expr\|cflow\|stmt` | **#440** | `38`/`39`; +`29`/`3A`/`4B`; +`53`/`54` | `parse_expr_classed` |
| `C2RS_SINK_CHAIN=<spec>` | **#660** | any pinned opcode, plus `type`/`convert`/`intrinsic` | `parse_expr_classed` |
| **`C2RS_SINK_STMT=<spec>`** | **`w-read2`** | the same spec, in the **statement** layer | `parse_body`'s dispatcher `_` arm; `eat_return_head`'s scope-close run |

### 12.1 Three warnings, each of which has already cost a lane

1. **`C2RS_SINK_OFF_ADD_ARG` is not in the family.** Its `0x27` arm pushes
   `IlOp::Add` and has **no poison** — it is a real widening behind an
   environment variable, and board **#403** records `cargo test` going to *16
   targets / 754 passed / 2 failed* under it. It must never be used as a chain
   step.

2. **A poisoned sink is fail-closed but NOT emission-neutral** (board **#3094**,
   corrected by **#3104**/**#3105**). `C2RS_SINK_CHAIN` **de-accepts** on 5 of
   its 49 pinned tokens — `33` −5 `match`, `B9` −4, `55` −6, `41` −1, `2C` −1 —
   and is bit-for-bit neutral on the other 44. The five are exactly the bytes
   `parse_expr` **already handles**, by two mechanisms: `chain_sink()` is
   consulted **before** the `b == stop` check (deliberately, board **#663**), and
   sinking an accepted production replaces it with a width-skip. **Before quoting
   any sink run's `match`/`fnbyte-exact`, check whether the sunk token is one
   `parse_expr` accepts or stops on. If it is, the number is about the
   instrument.**

3. **`C2RS_SINK_STMT` cannot de-accept, and the reason is structural rather than
   a flag.** `stmt_sink_walk` returns `Option<Block>` — it has no representation
   for accepting — and it is only ever consulted on a path that has **already
   decided to return `Err`**. It replaces the key on a refusal, never the
   verdict. Measured over 878 TUs at the full 49-token spec: `match` **25**,
   `fnbyte-exact` **35,734**, both unmoved from base.

### 12.2 What the fifth one was for, and the number it corrects

The first four are **all** consulted from `parse_expr_classed`, so a census key
raised **outside** `parse_expr` is invariant under every ceiling any of them can
measure. Lane `w-read2` measured that invariance rather than assuming it: the
five statement-layer keys — `body-cflow-label` **2,832** · `body-0x9B` **2,213**
· `return-scope-close-cflow-label` **1,814** · `body-0x67` **1,044** ·
`body-0x5D` **8** — read **7,911 at base and 7,911 at the full 49-token +
`type`/`convert`/`intrinsic` ceiling**, `+0` on every one, while every other key
of the 615 moved.

So the published decode ceiling was measured with **25.0 % of its own residue
held fixed**, and it composes exactly, because the two layers share no function:

| | reached the function tail | of 120,456 |
|---|---:|---:|
| `w-readphase` §4 | 76,041 | 63.1 % |
| `w-deaccept` §4.5 (`5D`/`5E` pinned) | 88,806 | 73.7 % |
| **+ `C2RS_SINK_STMT` (`w-read2`)** | **93,990** | **78.0 %** |

The `+5,184` is `stmt-chain-fntail` **3,684** + `rsc-chain-fntail` **1,500**, and
the addition is exact rather than estimated: the `base → stmt-sink` key diff and
the `chain-ceiling → both-sinks` key diff are **identical, key for key and count
for count**, so the expression layer and the statement layer are **orthogonal**
on this workload.
