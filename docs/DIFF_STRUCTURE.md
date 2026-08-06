# DIFF_STRUCTURE — what is *inside* a `fnbyte-differs` body

**The one-sentence answer.** The 3,195 differing function bodies are not
*nearly-right* code: **100 % of the port's bodies make a call or a tail branch
and 78.9 % of c2's counterparts make none at all**, so the population is one
mechanism — *c2 did not emit the call the port emitted* — and not a register
allocator, a scheduler, an immediate or a displacement. Eleven signature
clusters cover all 3,195; the top five cover **98.2 %**; **94.3 %** of bodies are
already wrong at **word 0**; and of the 5,189 substituted instruction words,
**5,173 (99.7 %) differ in their opcode**, 16 differ in a field, and **zero fail
to decode**.

Measured at tree `0c8a185` on the 878-TU dc3 workload, `/GR /O1 /Oi /EHsc`.
Lane `w-bytes`; boards **#976**–**#983** (the header read `#976`–`#985` and
`#984`/`#985` are `w-drop3`'s — corrected 2026-08-06);
[`rungs/2026-08-06-w-bytes.md`](rungs/2026-08-06-w-bytes.md).

> **⚠ §3.2 and one row of §4 are REFUTED, and §6 is why.** Lane `w-drop3` read
> the reference obj's **relocation targets** as well as its bytes: a `/Gy`
> branch word carries the same four bytes for every callee, so the equality this
> whole page's alignment is built on credits a relocated word it never checked.
> The 140-body cluster is **mechanism I misread as a deletion**, and **861
> functions this page's `exact` bucket credits call a different symbol than c2
> does**. Boards **#984**–**#989**;
> [`rungs/2026-08-06-w-drop3.md`](rungs/2026-08-06-w-drop3.md). **The rest of
> the page stands**: §2's byte counts, §2.1's field census and §3's transfer
> census are all measurements the relocation blindness does not touch.

> **This page is an instrument's output, not a licence.** Nothing here reaches a
> numerator, appears in an accept/refuse path, or grades the port. The judge is
> still real `c2` under wibo plus a byte-exact obj compare, and every body on
> this page has already been called **wrong** by it. The value of the page is
> that it says *how many different ways* wrong, and the answer is: about one.

---

## 1. Where the numbers come from

| | |
|---|---|
| the measurement | `crates/c2-harness/src/gap/fndiff.rs` — decoder, alignment, signature |
| where it runs | `crates/c2-harness/src/gap/fnbytes.rs`, on the `fnbyte-differs` path only |
| the counts | printed by **every** `c2rs gap` scan, under `DIFF STRUCTURE` |
| the per-symbol rows | `c2rs gap … --fnbyte-diff-jsonl PATH` (opt-in; one JSON object per differing function) |
| the rendering | `scripts/fndiff_report.py <jsonl> [--top N] [--cluster KEY]` |
| the relocation sites | `c2_obj::ObjImage::text_comdat_reloc_sites` (new) |

Reproduce:

```sh
cargo build --release -p c2-harness
./target/release/c2rs gap --list work/dc3-workload/files.txt \
    --flags-file work/dc3-workload/flags.txt --cwd ../dc3-decomp --jobs 12 \
    --fnbyte-diff-jsonl work/fndiff.jsonl
python3 scripts/fndiff_report.py work/fndiff.jsonl --top 11
```

### 1.1 The method, and what it took from objdiff

`../objdiff` aligns two symbols at **instruction** granularity, diffs
relocation-aware, and renders per decoded field. All three transfer here, and
the change of ISA is what makes them cheap: PPC is fixed-width, so an
"instruction" is a 4-byte big-endian word, the alignment is a plain LCS over
`u32`s, and the field decode is a bit-field partition rather than a length
decoder.

What is **not** taken is objdiff's *scoring*.
[`FUNCTION_BYTE_MATCH.md`](FUNCTION_BYTE_MATCH.md) and `fnbytes.rs`'s module
docs record why a fuzzy percentage inverts the correctness rule here — it pays
more for a wrong emit than for the honest refusal it replaced. There is no
partial credit on this page. A cluster's size is a count of bodies the judge
called wrong, and it goes up when the port emits more wrong bodies.

Per differing symbol the instrument records: common prefix and suffix in words,
the first divergence, an LCS alignment with adjacent insert/delete runs **paired
into substitutions**, a field classification per substituted pair, a
`same-multiset` bit (is this a pure reordering?), and whether the disagreement
sits under a relocation. Keyed on `FnCensus::emit_name` throughout — board
**#918**, `IlFunction::mangled_name` disagrees on 74,955 rows and is never used
here.

### 1.2 The decode discipline, and the control on it

[`CODEGEN_W6_COMPARE.md`](CODEGEN_W6_COMPARE.md) established the rule: a word is
decoded only when it **re-encodes from its fields bit-exactly**. There it was 29
words done by hand; here it is structural. A decoded word is a list of fields
with explicit bit ranges, and `Decoded::reencode` ORs them back together — a
form whose partition does not cover all 32 bits, or covers one twice, cannot
reproduce its word and is returned as **`undecoded`**. Primary opcode `04`
(VMX/VMX128) is refused outright rather than fitted to a generic form table.

Two controls, both printed on every scan:

* **`fndiff-accounting-broken` — 0.** Per row, `equal + sub + del == ref_words`
  **and** `equal + sub + ins == port_words`. A broken alignment would still
  produce a tidy-looking cluster table, so the identity is positive, per row and
  counted (STATUS trap 5: compare a count, never a status).
* **`fndiff-oe-set` — 0.** The one decode simplification (XO-form's `OE` bit
  folded into the extended-opcode field) is a measured non-issue rather than an
  assumed one.

> **⚠ The first run of this census reported `9.1 % undecoded`, and it was an
> instrument defect, not a fact about the bytes.** All **470** of those words
> decoded perfectly. `addi` (primary 14, fields `RT`/`RA`/`SI`) and `lwz`
> (primary 32, fields `RST`/`RA`/`D`) share the form name `D` and share **no**
> field names, and the classifier fell through to `undecoded` when it could not
> find a field on the other side. They are simply *different instructions*. The
> fix compares the primary opcode **before** any field, the regression is a unit
> test (`two_d_form_instructions_with_different_field_names_are_an_opcode_difference`),
> and `undecoded` now means one thing only: a form this file does not model.
> Board **#977**. The number that page-one depends on — *"do we understand the
> layout"* — was wrong by 470 words for exactly one scan, and it was wrong in
> the pessimistic direction.

---

## 2. The cluster table

**3,195 signatures · 11 clusters · 0 accounting breaks · 0 LCS-capped rows · 0
pure reorderings · 588 TUs.**

Cluster key = `shape | length relation | edit shape | field classes`.

| n | % | cluster | port makes a call | c2 makes one | what it is |
|---:|---:|---|---:|---:|---|
| **1349** | 42.2 % | `seq \| port-longer \| sub+ins \| opcode` | 1349/1349 | **1**/1349 | port frames and calls; **c2 inlined the callee** and needed no frame |
| **1157** | 36.2 % | `tail \| ref-longer \| sub+del \| opcode` | 1157/1157 | 491/1157 | port tail-calls; **c2 inlined the callee's body** in place |
| **370** | 11.6 % | `tail \| port-longer \| sub+ins \| opcode` | 370/370 | **0**/370 | port emits `li rN,0 ; b callee`; **c2's whole body is `blr`** |
| **140** | 4.4 % | `seq \| ref-longer \| del-only \| -` | 140/140 | 140/140 | **the reverse** — c2 emits a 7-word call the port omits |
| **123** | 3.8 % | `framed \| port-longer \| sub+ins \| opcode` | 123/123 | **0**/123 | port frames and calls; c2 inlined to 2 words + `blr` |
| 38 | 1.2 % | `seq \| same-len \| sub-only \| opcode` | 38/38 | 38/38 | one `bl` against one `lwz` — c2 inlined **one of two** calls |
| 12 | 0.4 % | `seq \| port-longer \| sub+ins \| mixed:reg+disp+opcode` | 12/12 | 0/12 | port frames a ctor call; c2 inlined it as five stores |
| 2 | 0.1 % | `tail \| same-len \| sub-only \| imm` | 2/2 | 2/2 | **the only pure-immediate cluster in the workload** |
| 2 | 0.1 % | `tail \| ref-longer \| sub+del \| opcode+reg` | 2/2 | 0/2 | |
| 1 | 0.0 % | `seq \| ref-longer \| sub+ins+del \| opcode` | 1/1 | 1/1 | |
| 1 | 0.0 % | `seq \| ref-longer \| sub+del \| opcode` | 1/1 | 1/1 | |

Top five = **3,139 of 3,195 = 98.2 %**. By port shape: `seq` 1,541 · `tail`
1,531 · `framed` 123 — the same split `graded by shape × verdict` prints.

### 2.1 The three distributions that make the table one finding

| | |
|---|---:|
| bodies whose **first word** already differs | **3,013 / 3,195 = 94.3 %** |
| bodies with a non-empty common prefix | 182 |
| bodies with a non-empty common suffix (almost always the shared `blr`) | 1,665 |
| substituted words differing in **opcode** | **5,173 / 5,189 = 99.7 %** |
| … in a register field | 2 |
| … in an immediate | 2 |
| … in register + displacement together | 12 |
| … **undecoded** | **0** |
| bodies that are a pure instruction **reordering** | **0** |
| substitutions sitting under a relocation | 122 |
| deletions sitting under a relocation | 1,284 |
| relocation records not word-aligned (known answer 0) | 0 |

**Read the second block first.** If the port's defect on this population were a
register allocator, a scheduler or an addressing-mode choice, the substituted
words would differ in `reg`, in nothing (a permutation), or in `disp`. Sixteen
words out of 5,189 do. Everything else is a *different instruction at that
position*, 94 % of the time starting at the first instruction of the body.

---

## 3. The mechanism, priced

Across the whole population, ignoring clusters:

"Makes a call" here means **transfers control anywhere other than by its own
terminal `blr`** — primary 16 or 18, or primary 19 with XO 16/528, so an
indirect `bctrl` and a conditional tail are both counted. `scripts/fndiff_report.py`'s
`has_transfer` is the same predicate; every figure below is that one test.

| | |
|---|---:|
| port bodies containing a call or a branch | **3,195 / 3,195 = 100 %** |
| c2 bodies containing one | 674 / 3,195 = 21.1 % |
| **c2 bodies that are straight-line code ending in `blr`** | **2,521 / 3,195 = 78.9 %** |
| c2 bodies that are *exactly* `blr`, one word | **370** |
| port bodies with a linked call (`bl`/`bctrl`) | 1,664 |
| c2 bodies with one | 385 |
| port bodies with an unconditional tail branch | 1,531 |

**That is mechanism I** — [`INLINE_PREDICATE.md`](INLINE_PREDICATE.md)'s "c2 does
not emit a call the IL contains" — read off the bytes rather than off a
predicate's hold-out score. The lane did not set out to confirm it; the byte
structure produced it, from the opposite direction, with no inlining model in
the loop.

Three things this adds that the predicate's own page does not have:

1. **A size in bodies, from the judge's own denominator.** 2,521 of the 3,195
   differs are bodies where c2 emitted no call at all. Mechanism I's hold-out
   number (0.9716 on 100 TUs) is a classifier score; this is a count of
   functions whose bytes are wrong *today*, on the workload.
2. **A floor on what the rest could ever be.** Sixteen substituted words in the
   entire population differ in a register, an immediate or a displacement. Even
   a perfect inline predicate leaves the 140-body cluster (§3.2) and a
   two-figure remainder; there is **no** hidden register-allocation or
   scheduling debt inside this population waiting behind it.
3. **The `blr`-only sub-population is exactly 370 and exactly uniform.**
   `port = li rN,0 ; b callee` against `c2 = blr`. Every one of the 370 has that
   shape, in 211 TUs.

### 3.1 Worked example — cluster 3 (370 bodies), the sharpest target

`src/lazer/game/BustAMovePanel.cpp` ·
`??$_Destroy_Range@PAH@stlpmtx_std@@YAXPAH0@Z` · port 2 words, c2 1 word, c2
relocations **0**:

```
        port                              c2
opcode  38a00000  li r5,0            |    4e800020  blr
ins     4bfffffc  b -4               |
```

c2's entire function is `blr`. The port sets up an argument and tail-branches to
a callee that, once inlined, produces nothing. Compare `elide.rs`'s **mechanism
E**, which closed 1,516 bodies of exactly this *outcome* by a different *test* —
E fires when the callee's body is literally empty. Here the callee is not empty;
inlining it is what makes it nothing.

### 3.2 Worked example — cluster 4 (140 bodies), ~~the one pointing the other way~~

> ## ⚠ 2026-08-06 — **THIS SECTION IS REFUTED. The port does not omit a call.**
>
> Lane `w-drop3` read the IL and then real c2's own **relocation table**, and
> both halves of this section's reading are wrong. Boards **#985**, **#987**;
> [`rungs/2026-08-06-w-drop3.md`](rungs/2026-08-06-w-drop3.md).
>
> **The port emits the calls the source has, and c2 inlined one of them.**
> `??$Obj@V…@@DataArray@@` is `return Node(i).Obj<T>(this);` and the port parses
> it as a WCH/WCL **chain** — two calls, correctly:
>
> ```text
>  port  ?Node@DataArray@@…      →  ??$Obj@V…@@DataNode@@…   (link_args=[Formal(0)])
>  c2    ?Node@DataArray@@…      →  ?GetObj@DataNode@@…      →  __RTDynamicCast
> ```
>
> **c2's second `bl` relocates to `?GetObj@DataNode@@`, not to the callee the
> port names.** `DataNode::Obj<T>` is `return dynamic_cast<T*>(GetObj(source));`
> and c2 expanded it; its own COMDAT in the same obj is the "missing" seven words
> verbatim. This is **mechanism I** — `w-seq`'s family (a) — and not a new
> mechanism at all.
>
> **Why it looked like a deletion.** Under `/Gy` a call out of a COMDAT is
> emitted with the placeholder displacement `-(offset of the branch word)`
> **whatever the callee is**, so the port's `bl ??$Obj@…@DataNode@@` and c2's
> `bl ?GetObj@DataNode@@` are both `4bffffe5`. The alignment below is an LCS
> under **byte** equality, so it scored word 7 `=` and reported **7 deletions**
> where the truth is **1 substitution + 7 insertions**. Board **#989**: a
> cluster's *edit shape* is only as fine as the equality it is built on, and this
> table's `=` rows at words 5 and 7 are the demonstration.
>
> **And the `.rdata` string pair is RTTI.** `r5`/`r6` are
> `??_R0?AVObject@Hmx@@@8` and `??_R0?AV<T>@@@8`, the type descriptors of
> `__RTDynamicCast(pv, VfDelta, SrcType, TargetType, isReference)` — `r4` and
> `r7` are its `VfDelta` and `isReference`, both 0, ABI slot for slot. The 11
> relocations are **3 REL24 + 2 × (REFHI + PAIR) + 2 × (REFLO + PAIR)**.
>
> The section is kept **as written** rather than rewritten: it was quoted as a
> fix spec in §5 and in board #979, and the record of what a relocation-blind
> byte test said is worth more than a tidy page.


`src/lazer/game/Game.cpp` · `??$Obj@VHamUser@@@DataArray@@QBAPAVHamUser@@H@Z` ·
port 13 words, c2 20 words, c2 relocations **11**. The port's body is a strict
**subsequence** of c2's: eight identical words, then seven the port never
emitted, then a five-word shared epilogue.

```
=       7d8802a6  mflr r12           |    7d8802a6  mflr r12
=       9181fff8  stw r12,-8(r1)     |    9181fff8  stw r12,-8(r1)
=       fbe1fff0  std r31,-16(r1)    |    fbe1fff0  std r31,-16(r1)
=       9421ffa0  stwu r1,-96(r1)    |    9421ffa0  stwu r1,-96(r1)
=       7c7f1b78  mr r31,r3          |    7c7f1b78  mr r31,r3
=       4bffffed  bl -20             |    4bffffed  bl -20
=       7fe4fb78  mr r4,r31          |    7fe4fb78  mr r4,r31
=       4bffffe5  bl -28             |    4bffffe5  bl -28
del                                  |    3d600000  lis r11,0
del                                  |    3d400000  lis r10,0
del                                  |    38ab0000  addi r5,r11,0
del                                  |    38ca0000  addi r6,r10,0
del                                  |    38800000  li r4,0
del                                  |    38e00000  li r7,0
del                                  |    4bffffc9  bl -56
=       38210060  addi r1,r1,96      |    38210060  addi r1,r1,96
=       8181fff8  lwz r12,-8(r1)     |    8181fff8  lwz r12,-8(r1)
=       7d8803a6  mtlr r12           |    7d8803a6  mtlr r12
=       ebe1fff0  ld r31,-16(r1)     |    ebe1fff0  ld r31,-16(r1)
=       4e800020  blr                |    4e800020  blr
```

**All 140 are byte-identical in the missing sequence** — the same seven words
every time, at the same word index 8, with the same 11 relocations — and all 140
are the same template, `??$Obj@V…@@DataArray@@QBAPAV…@@H@Z`, across 101 TUs.
The two `lis`/`addi` pairs are relocated address halves (a `.rdata` string pair,
by shape), so the missing call is a **third** call in a three-call sequence whose
first two the port emits correctly. This is the port's `seq` recognizer
under-counting a sequence, and it is the only cluster in the census where the
port emits *less* than c2.

### 3.3 Worked example — cluster 1 (1,349 bodies), the largest

`src/lazer/game/BustAMovePanel.cpp` ·
`??0?$_List_iterator@VSymbol@@…@stlpmtx_std@@QAA@PAU_List_node_base@1@@Z` ·
port 12 words, c2 **2**:

```
opcode  7d8802a6  mflr r12           |    90830000  stw r4,0(r3)
ins     9181fff8  stw r12,-8(r1)     |
ins     fbe1fff0  std r31,-16(r1)    |
ins     9421ffa0  stwu r1,-96(r1)    |
ins     7c7f1b78  mr r31,r3          |
ins     4bffffed  bl -20             |
ins     7fe3fb78  mr r3,r31          |
ins     38210060  addi r1,r1,96      |
ins     8181fff8  lwz r12,-8(r1)     |
ins     7d8803a6  mtlr r12           |
ins     ebe1fff0  ld r31,-16(r1)     |
=       4e800020  blr                |    4e800020  blr
```

c2 inlined the base constructor into one store and emitted **no frame at all**;
the port emits the full save/restore because it emits the call. The second modal
length in this cluster is `19w → 6w`, the same shape one level of inlining
deeper. **One of the 1,349 c2 bodies contains a branch**; 1,348 do not.

---

## 4. Do we understand the layout well enough?

**At the instruction-encoding level: completely, and it is measured rather than
asserted.** Every one of the 5,189 substituted words decodes into a field
partition that re-encodes it bit-exactly; `undecoded` is **0**; the alignment's
accounting identity holds on all 3,195 rows; the relocation sites are all
word-aligned. There is no residue in the byte reading.

**At the "why are these bytes different" level: 95.5 % is one already-named
mechanism, 4.4 % is one new and perfectly uniform cluster, and the residue is
five bodies.** The partition below is by the **byte test**, not by narrative —
which side transfers control (§3), and in which direction the length moved. It
sums to 3,195 exactly.

| population | n | % | maps onto | status |
|---|---:|---:|---|---|
| **c2's body makes no call or branch at all**; the port's does | **2,521** | 78.9 % | [`INLINE_PREDICATE.md`](INLINE_PREDICATE.md) mechanism **I** — and **E**'s neighbourhood for the 370 whose c2 body is `blr` alone | **named, unmodelled.** E is shipped (`elide.rs`, 1,516 bodies); I is not, and holds at 0.9716 on a 100-TU hold-out |
| c2 still calls, but a **different, longer** body than the port's tail branch | 491 | 15.4 % | the same mechanism partially applied — c2 inlined some of the chain | same |
| ~~**the port omits a call c2 makes**~~ — **REFUTED (§3.2, #985): the port omits nothing; c2 inlined the port's SECOND callee, and the byte test could not see the relocation target** | **140** | 4.4 % | `INLINE_PREDICATE.md` mechanism **I**, like row 1 | **row 1's population is 2,521 + 140** |
| c2 replaced **one of two** calls with a load, same body length | 38 | 1.2 % | mechanism I, one call deep | same as row 1 |
| an **immediate** is wrong (`addi r3,r3,4` vs `…,8`) | 2 | 0.1 % | ordinary codegen | the only wrong *number* in 3,195 bodies |
| three long singletons | 3 | 0.1 % | unclassified individually | named in the JSONL by symbol |
| **pure scheduling permutations** | **0** | 0 % | — | **the schedule is not a defect on this population** |

**What is genuinely unexplained is small and is named.** ~~The 140-body cluster is
new (§3.2) and reproducible to the word.~~ **(2026-08-06: it is reproducible to
the word and it is not new — §3.2's banner. It is mechanism I, so this table's
"one new and perfectly uniform cluster" reads 0, and the 95.5 % that is "one
already-named mechanism" reads 99.9 %.)** The `same-len | sub-only | imm` pair (2
bodies, `addi r3,r3,4` against `addi r3,r3,8` — a structure offset) is the only
place in 3,195 bodies where the port picks a wrong *number*. Three long
singletons are not individually classified and are named here so nobody has to
re-derive them: `?supershuffle@@YAXPAD@Z` (`src/keygen_xbox.cpp`, 21w→26w),
`?Init@Sequence@@SAXXZ` (`src/system/synth/Sequence.cpp`, 13w→43w) and
`??APaddedJointPos@@QAAAAMH@Z` (`src/system/hamobj/DetectFrame.cpp`, 13w→1w,
where c2's whole body is a bare `b` — a tail call the port framed).

**So the residue that is neither the inline mechanism nor the one new cluster is
five bodies of 3,195 — 0.16 %.**

**One caution about the 491-row row.** "c2 still calls, but a different body" is
a *byte* statement, and the narrative attached to it — partial inlining — is an
inference this lane did not test. What is measured is that the port emitted a
1–3 word tail branch and c2 emitted a longer body that itself branches. A lane
that wants to act on those 491 owes a probe, not this row.

**The caveat that does not go away.** This is 3,195 bodies out of 178,975
emitted functions; 130,573 are refused before any of this applies, and the
port's defect surface *there* is unmeasured by construction — a refused body has
no bytes to diff. Trap 0 in [`STATUS.md`](STATUS.md) applies to this page in its
most literal form: **the shape of the differs is a statement about the
population the port already produces bodies for.** Widening
`IlBundle::functions()` will widen this census too, and there is no reason from
this page to expect the new population to look like this one.

---

## 5. What a fix lane would take, and what this lane deliberately did not ship

Nothing here shipped an emitter change; this lane is measurement and tooling.
Three specs fall out, ranked by bodies per unit of new mechanism:

1. **The `blr`-only cluster — 370 bodies, one shape.** `port = li rN,0 ; b
   callee` where c2's whole body is `blr` and carries **no relocation**. The
   relocation-count-zero test is an oracle-side fact that separates it cleanly.
   This is E's outcome reached by I's test, so the spec is: extend `elide.rs`'s
   fixpoint from *callee body is empty* to *callee body inlines to nothing*.
   Board **#980**. **The hazard is the direction #950 already recorded** — the
   relocation observable reads "nothing happened" on a self-recursive body that
   is plainly not nothing, so a rule keyed on relocations alone is not sound and
   must be keyed on the callee.
2. ~~**The `DataArray::Obj<T>` cluster — 140 bodies, one template, one missing
   7-word call.** Not an inlining question at all: the port emits two of three
   calls in a `seq` and drops the third. Board **#979**. Smallest and most
   mechanical of the three; the missing sequence is byte-identical across all
   140, so a fix has an exact known answer.~~

   > **⚠ WITHDRAWN 2026-08-06 — every clause of that sentence is false.** It
   > **is** an inlining question, the port drops nothing, and there are two
   > calls in the source rather than three. §3.2's banner and board **#985**
   > have the compiler's own relocation table. The 140 are **mechanism I**, so
   > they belong to spec 3 below and the population there is **3,190**, not
   > 3,050.
   >
   > **What a fix lane must not take from this row**: that the cluster is
   > *cheap*. Lane `w-drop3` was briefed on this spec, took it, and found no
   > sound emitter change at this rung — board **#988** kills all three
   > candidates by name, including the honest-refusal escape hatch, which
   > cannot be grounded on any rule that does not also shrink `fnbyte-exact`.
3. **The main inline population — 3,050 bodies** (2,521 + 491 + 38). This needs the predicate, and
   `INLINE_PREDICATE.md` §1.2's six places the chain stops all apply. Not a
   small target; listed here so its size is on the record next to the two that
   are.

**And one thing a fix lane must NOT read off this page.** Cluster size is not
conversion. Closing a cluster moves `fnbyte-differs` and moves TU match by zero
unless the whole TU's every other defect also closes — trap 8's shape, and the
`w-empty`/`w-fix` precedent: 1,516 bodies closed, TU match `10 → 10`.

---

## 6. What the byte test could not see — the CALL TARGET

**Everything above §5 compares `.text` bytes, and a `/Gy` branch word cannot
carry its callee.** c2 writes a call out of a COMDAT with the placeholder
displacement `-(offset of the branch word)` — the same four bytes for every
target alike — so two bodies calling two entirely different functions are
byte-identical and every alignment, every cluster key and every `=` row on this
page scores that word equal.

That is board **#882** (`fnbyte-exact-relocated`: 4,664 credited functions carry
a relocation FBM does not check) stated as a caveat. Lane `w-drop3` made it a
count. Boards **#984**–**#989**;
[`rungs/2026-08-06-w-drop3.md`](rungs/2026-08-06-w-drop3.md).

| | |
|---|---|
| the reference side | `c2_obj::ObjImage::text_comdat_call_targets` — `REL24` targets by symbol name, per COMDAT, in offset order |
| the port side | `c2_core::comdat::comdat_function_body`'s own `calls` list — **the emitter's**, never a second walk over `IlFunction` |
| the comparison | `(offset, name)` pairs, in emitted order |
| the counts | `gap-metric fnbyte-calltarget-*` on **every** scan |

### 6.1 The result, on the 878-TU workload

| key | value |
|---|---:|
| `fnbyte-calltarget-graded` | **39,177** — exactly `exact 35,982 + differs 3,195` |
| `-ungraded` · `fnbyte-call-targets-unreadable` | **0** · **0** |
| `-agree` | 35,121 |
| **`-disagree`** | **4,056** |
| — of which the byte test calls **`differs`** | **3,195** — *all of them* |
| — of which the byte test calls **`exact`** | **861** |
| by call **count** · by **name** at the same count | 2,867 · 1,189 |

**Two readings, and the second is the one that costs something.**

1. **All 3,195 differing bodies call the wrong things.** §3 derived "one
   mechanism" from opcodes; this derives it from the symbol table, with no
   alignment in the loop. The two agree.
2. **861 bodies the judge's byte test CREDITS relocate against a different
   symbol than c2 does.** Hand-verified from an obj this lane compiled
   (`src/lazer/game/BustAMovePanel.cpp`):

   ```text
   ??1?$list@H…@QAA@XZ        48000000   b  →  ?clear@?$_List_base@H…@QAAXXZ   ← c2
   ??1?$list@H…@QAA@XZ        48000000   b  →  ??1?$_List_base@H…@QAA@XZ       ← the port
   ```

   Same four bytes. c2 inlined `~_List_base()`, which is itself a one-word tail
   call to `clear`. `fnbyte-exact` counts this.

**`mismatch` is still 0 and `IlBundle::functions()` is untouched** — every one of
the 861 sits in a TU the parser refuses, so none has reached an obj. What is
wrong is the *credit*, exactly as board #878 says of the 3,195, and the hazard is
the next `functions()` widening.

### 6.2 What this does to the rest of the page

**A cluster's byte counts are a measurement; its EDIT SHAPE is a hypothesis.**
The alignment is an LCS over 4-byte words under byte equality, which is strictly
coarser than instruction equality wherever a relocation sits — so a substitution
under a relocation is invisible and the edit shape reported around it can be
wrong. §3.2 is the worked instance: `del-only` on bodies whose true edit is one
substitution and seven insertions.

**Not repaired here, on purpose.** Making the alignment relocation-aware would
move every count in §2's published table and would move `fnbyte-exact` itself;
that is a lane with its own before/after, not a side effect of the lane that
found it. Board **#989**. What ships instead is the number, printed beside the
cluster table on every scan, so the coarseness is visible rather than assumed
away.
