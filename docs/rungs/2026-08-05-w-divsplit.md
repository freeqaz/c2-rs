# w-divsplit — the 4,670 are all integer, the witness that would have found it was blind, and the population is a loop population

    Tag:       w-divsplit
    Slug:      w-divsplit
    Date:      2026-08-05
    Fixtures:  none — a sizing lane ships no fixture.
    Census:    711,427 → **711,427** (28.88%, **+0**). Emitted 39,177 →
               **39,177** (21.89%, **+0**). **TU match 10 → 10.**
               `fnbyte-differs` **0 → 0**.
    Record:    this file; prereg `work/w-divsplit/PREREG.md`, committed at
               `125a40d` **before the first row was dumped and before either
               probe script existed**.
    Lane:      w-divsplit, worktree branch off master **`8fd79b6`**.
    Ships:     a census-key **refinement** (`expr::EXPR_TYPED_OP`,
               `note_operand_type`), two decode-only table entries
               (`chain_skip_form`, `cf-expr`), five tests, two probes under
               `work/w-divsplit/`, board rows **#816**–**#824**, and the
               resolution of **#783**.
               **No emit. No new accepted shape. Nothing in `c2-core`.**

---

## 1. The result, in the order it should be read

> ### Board **#783** asked whether `expr-op-0x05`'s 4,670 is integer division or floating-point division. It is **integer, 4,670 of 4,670, and the float share is 0** — measured twice by independent instruments that agree row for row.

> ### The witness #783 itself specified would have measured **nothing**. It reads a 3-byte TYPE at `mark - 3`; the operand that ends at the opcode is a **literal** at 4,674 of 4,674 sites, whose type ends two to six bytes earlier. Agreement with the honest decode: **0 of 4,674**.

> ### And the number that matters is not 4,670. It is **506 emitted instances of 339 distinct functions** — 13.8× replication — of which **4,649 (99.5 %) sit inside a LOOP**, with a **compile-time-constant** divisor that is **not a power of two** at 84.5 %. The embedded-division rung is a loop rung with a magic-multiply inside it.

The first line is the assignment. The third is the one a rung-sizer should read.

| metric | baseline (`8fd79b6`, measured) | after |
|---|---:|---:|
| **TU match** | 10 | **10** |
| mismatch · codegen-gap · port-error | 0 · 0 · 0 | **0 · 0 · 0** |
| vocab-gap · capture-fail | 861 · 7 | **861** · 7 |
| per-function census | 711,427 / 2,463,393 | **711,427** (+0) |
| emitted census | 39,177 / 178,975 | **39,177** (+0) |
| census/gate disagreement | 0 | **0** |
| `fnbyte-differs` | 0 | **0** |
| blocker keys other than the two split | — | **0 moved** |
| total blocked bodies | 1,751,966 | **1,751,966** |

`fnbyte-differs` is 0 and no emit path was touched, which is the bar a sizing
lane is held to.

### The prereg, graded

`work/w-divsplit/PREREG.md`, committed at `125a40d`. Eight registered claims,
**seven hit and one missed**.

| # | registered | outcome |
|---|---|---|
| **R1** | FP share of the 4,670 is **0**; integer share ≥ 4,600 | **HIT** — 4,674 of 4,674 integer, 0 float |
| **R2** | a fixed-offset reader misclassifies **≥ 100** sites | **HIT, by 46×** — wrong or blind on **4,674 of 4,674** |
| **R3** | at least one **pointer**-class operand | **MISS** — 0 of 4,674 (§11) |
| **R4** | ≤ 6 distinct `(tag,kind)` pairs | **HIT** — **1** |
| **R5** | `expr-op-0x06`'s 4 are also integer | **HIT** — 4 of 4 |
| **R6** | conservation: sum 4,674, census/emitted/`fnbyte-differs` unmoved | **HIT** — every number identical, 0 other keys moved |
| **R7** | EMITTED share in **[150, 800]** | **HIT** — **506** |
| **R8** | > 50 % have a **literal** divisor | **HIT** — **100.0 %** |

R1 and R7 were named in the prereg as the two that decide the headline, and they
point in opposite directions: R1 says the 4,670 is not diluted by floats, R7 says
it is diluted by emission and §5 says it is diluted again by replication.

---

## 2. Why the ambiguity is real about the byte and unreachable as a key

#783's mechanism is correct and worth restating, because it is *not* what the
measurement refutes. `parse_expr_classed` has no `0x05` arm; `IlOp::Div` is
produced only inside `leaf_float.rs`; so nothing in the *renderer* separates an
integer division from a floating-point one. All true.

What it missed is that **a census key is a body's FIRST blocker**. `expr-op-0x05`
is produced at exactly one site — the fall-through of `parse_expr_classed`
(`expr.rs`, `_ => return Err(blk(seg, *p, "expr"))`; grepped, `"expr"` as a ctx
with a blocking byte occurs nowhere else in `c2-il`). To *reach* that byte the
walk must already have consumed both operands, and every operand-producing arm
admits a type only through `eat_operand_type` (`Int4` / `Ptr4` / `Int1u`) or an
admitted `2C`. A `float` or `double` operand refuses one token earlier, under
`expr-lit-type-8645` / `expr-load-type-8645` — **a different bucket**.

That is an argument, so it is also a test:
`a_float_operand_blocks_at_the_load_not_at_the_division` builds both forms and
asserts the key, positively, rather than asserting that nothing float-shaped
appears in a histogram. *Absence read as success* is `STATUS.md` trap 5 and this
lane's whole subject is a bucket that was never looked into.

---

## 3. Two instruments, one partition

**Instrument 1 — byte-scan** (`work/w-divsplit/split.py`, over the
`C2RS_ROW_DUMP` TSV). It does **not** read a byte at a fixed offset. It decodes
the operand **token that ends exactly at the mark**, trying every form the
operand grammar has (`B9 <tok> <TYPE>`, `33 <TYPE> <payload>`, `2C <TYPE> 00`,
`27 <TYPE>`, a one-byte operator) and requiring the decode to land on the mark.
Sites where none lands, or where two land and disagree on the type, are counted
and printed.

**Instrument 2 — parse-derived.** `note_operand_type` records the `<tag><kind>`
at the cursor the parser has already proved a TYPE starts at, and a
divide/modulo refusal carries it in `Block::aux` under its own ctx
(`EXPR_TYPED_OP`). The key renders `expr-op-0x05-8641`.

They agree exactly:

```
byte-scan    4,674 of 4,674 int-signed · 0 undecodable · 0 conflicts · 1 distinct (tag,kind)
parse-derived  expr-op-0x05-8641 4,670   expr-op-0x06-8641 4   (878-TU scan)
```

The refinement is asserted to be one: `the_div_mod_key_is_an_exact_refinement_of_the_published_one`
runs both opcodes × every tag × every kind and requires each key to start with
the published string **and** to be injective on the recorded type, so rows can
only split and two types may never share a bucket. The completeness axis is
asserted unchanged in the same loop.

---

## 4. #783's own witness would have found nothing — #644, on a live population

The proposed reader takes the 3-byte triple at `mark - 3`. Measured against the
honest decode over all 4,674 sites:

```
fixed-offset(mark-3) agrees with the decoded token :     0
fixed-offset(mark-3) finds no 3-byte TYPE          : 4,673
fixed-offset(mark-3) reads a DIFFERENT type        :     1
=> wrong or blind on 4,674 of 4,674 (100.0%)
```

The cause is that the operand ending at the opcode is a **literal** at every
site, `33 <TYPE> <payload>`, and the payload is **one byte below `0x80` and five
bytes above it** (`readers.rs:423` — a signed byte, or `0x80` + a 4-byte LE
`i32`; *not* LEB128, which is what the TYPE's own id field uses). Two
variable-length encodings in one container, and the naive stride assumes
neither.

This cost the lane a wrong answer before it cost anyone else one: a first draft
of `split.py` used LEB128 for the literal payload and reported **41 sites as
UNDECODABLE**. Those 41 are ordinary divisions by a struct size ≥ 128 — exactly
the values the escape form encodes. They were found because the probe *prints
its undecodables* instead of bucketing them.

---

## 5. What the population actually is, by count

| axis | measurement | denominator |
|---|---:|---:|
| EMITTED | **506 (10.8 %)** | 4,674 |
| distinct mangled names, all rows | **339** | 4,674 |
| distinct mangled names, emitted rows | **330** | 506 |
| TUs carrying ≥ 1 row | **695** | 878 |
| TUs carrying ≥ 1 emitted row | **182** | 878 |

The workload-wide emitted rate is 178,975/2,463,393 = **7.3 %**, so at 10.8 %
this population is barely enriched for emission. The replication factor is
**13.8×**: the top eight names are all `stlpmtx_std` template instantiations,
led by `??$__push_heap@…` at 684 rows and six `??$__copy@…` at 431, 427, 202,
200 and 197.

`witness.rs`'s own doc raises exactly this question — *"is a row N distinct
source functions or one replicated across TUs"* — and this is it asked of the
largest blocked population anyone had identified. **A sizing that reads 4,670 as
4,670 units of work is off by 13.8×.** The reading in the other direction is the
useful one: 339 functions is small enough to enumerate.

---

## 6. What is behind the row — the successor question, #622

#622 measured that closing `xboxheap`'s `expr-op-0x27` moved the label to
`expr-op-0x32` and converted nothing. The same question, asked here through the
existing chain sink (`C2RS_SINK_CHAIN=op:05,op:06`), comes back the other way:

```
-4,670  expr-op-0x05-8641        +4,653  expr-chain-sink-poison:mid
    -4  expr-op-0x06-8641           +13  expr-cmp-gt
                                      +8  expr-cmp-lt
```

**4,653 of 4,674 (99.6 %)** walk to the end of the expression with nothing else
unmodeled in it. Only 21 meet a further blocker.

**Read the scope exactly.** The poison fires at the end of the *expression* the
division sits in, and its `:mid` half says there was segment left to parse — so
this says nothing about the rest of the body. §8 is what the rest of the body
costs, and it is not cheap.

---

## 7. The stale comment that was costing 32,871 bodies their CFG

The sink needed a width for `05`, and `chain_skip_form` had none — the first
sink run returned `expr-chain-noform-0x05` 4,670 times, which is the instrument
refusing to guess and working correctly.

Establishing it turned up the real finding. **Two** decode-only tables listed
`05` as unwitnessed, and one of them was `control_flow.rs`'s payload-free
operator arm, whose own comment says the size of a `cf-expr-0xNN` row *"is what
tells the next rung whether establishing them is worth a probe"*.

* The row was **32,871 bodies** — the largest thing that table refused.
* `06` was sitting in the **witnessed** list beside it, from the same
  `IL_STMT_GRAMMAR.md` §5 probe. One production, half of it carried across.
* It had been witnessed since `lane w-divmod`: four captured leaf bodies
  `B9 <tok> <T> B9 <tok> <T> >05< 41 <T> 3A …`, graded **185 of 185** against
  real `c2.dll`.

Re-confirmed on the workload before moving it, rather than on the strength of
the older capture alone: at all **4,674** sites the byte after the opcode opens
a new token — `32 <TYPE>` (a store) at 4,646 and `33 <TYPE> <payload>` at 26 —
so there is nowhere for a payload to be.

What it decodes:

```
cf-expr-0x05  -32,871      cflow-straight  +10,777
                           cflow-loop       +7,042
                           cflow-if-1       +6,470
eh-unknown    -23,988      eh-none         +23,223
```

**The generalizing defect is a witness landing in one file and never being
propagated to the tables that gate on it.** Board **#820**.

---

## 8. The price: this is a LOOP population

Only measurable because of §7 — before it, 4,671 of the 4,674 read
`cf-expr-0x05` and their control flow was *unknown*, which is the honest reason
nobody had priced this rung.

| axis | value | share |
|---|---:|---:|
| `cflow-loop` | **4,649** | **99.5 %** |
| `cflow-if-1` | 15 | 0.3 % |
| `cflow-switch` | 5 | 0.1 % |
| `cflow-straight` | **4** | 0.1 % |
| `cflow-if-2` | 1 | 0.0 % |
| `eh-none` | **4,674** | **100 %** |
| `calls-1` | 3,475 | 74.3 % |
| `calls-0` | 1,176 | 25.2 % |
| `calls-2plus` | 23 | 0.5 % |

**Not one row carries the `+expr-modeled` suffix**, so every one of them needs
expression work beside the control flow rather than being blocked on control
flow alone.

EH is the one cheap axis and it is *completely* cheap: 4,674 of 4,674 `eh-none`,
so `/EHsc` costs this population nothing.

---

## 9. What an embedded-division emit would need, enumerated from the bytes

The divisor is a **compile-time constant at 4,674 of 4,674 sites** — the token
ending at the opcode is a literal, always. 40 distinct values:

```
/20   3,319  71.0%      /12  29    /40  19    /28  9    /48  6    /36  5
/2      724  15.5%      /6   21    /60  16    /84  8    /96  6    /732 5
/24     409   8.8%                 /56  13    /44  7    /72  6    /3   5
                                             /100  7    /88  6    /76  5
power-of-two divisors: 724 — and every one of them is the /2 family
```

So the enumeration, each item with the count that forces it:

1. **A loop lowering.** 4,649 of 4,674. `PORT_CFG_CLASSES` still does not list
   `cflow-loop` (#761); what exists is a twenty-word transcription of one
   function (`ptr_walk_loop`) and one body-parameterized chain loop.
2. **Division by a non-power-of-two constant → a magic-number multiply.**
   3,950 of 4,674 (84.5 %). Not `divw`, and **not** the shipped `div_mod_leaf`
   spine: that emits a register division and #781 records that it refuses every
   constant divisor. **The overlap between the shipped leaf and this population
   is 0 sites.**
3. **Division by 2 → the `srawi` + `addze` idiom.** 724 of 4,674 (15.5 %), a
   different instruction sequence from item 2, so it is a second cell and not a
   special case of one.
4. **A pointer-difference dividend.** A `03` SUB sits immediately before the
   divisor at 4,665 of 4,674 (99.8 %) and a pointer-class TYPE is in the window
   at 4,027 (86.2 %). The shape is `(p - q) / sizeof(T)`; the `/2` family is the
   other one, `(i - 1) / 2`, the binary-heap parent.
5. **A call inside the loop body.** 74.3 % are `calls-1`.
6. **The quotient's consumer.** `32 <TYPE>` — a store — follows the opcode at
   4,646 of 4,674 (99.4 %).
7. **`twi` placement made load-bearing.** #780's rule is measured over 161 cells
   and deliberately **not** shipped, because `div_mod_leaf` accepts only bodies
   where the clause is constant. Item 1 puts the division inside a loop body,
   which is one of #780's own named conditions (*"that block is not a loop
   body"*), so this population is precisely where the rule stops being free.

**Six consecutive frontier enumerations have come back dearer on contact and
this is the seventh.** The honest one-line statement of the rung is *"a loop
lowering with a constant-divisor magic multiply inside it, over bodies three
quarters of which make a call"* — not *"teach the parser `05`"*.

---

## 10. The must-fail mutation, and the control that it PASSED

`note_operand_type` was mutated to a **fixed stride** (`*p + 3` in the LITERAL
arm — exactly #644's mistake, and exactly the reader #783 proposed), the harness
rebuilt, and the full 878-TU scan re-run.

| control | result under the mutation |
|---|---|
| unit test `the_div_mod_key_names_the_operand_type` | **FAILED** — caught |
| cross-instrument agreement with `split.py` | **DISAGREED** — caught |
| bucket count (2 keys → **20**) | `…-8643` 3,905, `…-8641` 724, **17 junk buckets** over 41 rows — caught |
| **conservation: the buckets still sum to 4,674** | **PASSED** — *not* caught |

The fourth row is the point. `STATUS.md` trap 4 says a totality residue *"is
satisfied exactly by moving a record from one bucket to another"*; here is that
sentence with a mutation behind it, on a classifier that put every single row in
the wrong bucket. **A lane whose only control is that its split sums to the old
total has no control.** This is why two readers were built for one number.

The mutant's answer is legible in hindsight — the stride missed the literal's
type and the recorded value fell through to the *dividend's*, which is why 3,905
read as a data pointer. Nothing in the mutant run said so.

The Python instrument was calibrated the same way, by shifting its decode target
off the mark: `−1` and `+2` give **100 % UNDECODABLE**; `+1` degrades to a
100 % untyped `op-result` form. **No shift produces a different *type*** — it
produces nothing, or an untyped reading.

---

## 11. What missed, and what is left open

**R3 MISSED.** The prereg registered at least one pointer-class *operand*,
reasoning that `p - q` scaled by the element size is a division over pointers.
The reasoning is right — §9 item 4 — and the predicate was aimed one token too
far right: the operand that ends at the opcode is always the int literal
*divisor*.

Trying to decode the **dividend** instead runs into a real limit and it is filed
rather than papered over: `CENSUS_HEX_BACK` is 16 bytes, and the dividend does
not fit. Over the 4,674 sites, decoding the operand ending at the `03` gives
**1,429 UNDECODABLE (30.6 %)**, **2,509 untyped `op-result` (53.7 %)**, 722
`int-signed`, 5 `data-pointer` — resolved on **727 of 4,674 (15.6 %)**. Widening
the constant is a change of meaning to a published field every row-dump consumer
reads, and this lane did not need it. Board **#824**.

Also left open, deliberately:

* **The key refinement covers `05`/`06` only.** The same mechanism would give
  `expr-op-0x30`, `expr-op-0x41` and a dozen others their operand type, and each
  is a published key whose meaning change has to be paid for in docs. One ctx,
  one rekey, one board row.
* **Nothing is proposed for `PORT_CFG_CLASSES`.** §8 says the population is
  `cflow-loop`; whether an entry is warranted is a loop lane's finding and #810
  is the mechanism for pricing one.
* **No `twi` rule is fitted.** #780 refuted three readings and `w-pair` §4 plus
  `leaf_store.rs` account for ten more; this lane adds none.

---

## 12. Reproducing it

```sh
# the row dump (read-only over the census, asserted so in witness.rs)
C2RS_ROW_DUMP=expr-op-0x05-8641,expr-op-0x06-8641 \
C2RS_ROW_DUMP_OUT=work/w-divsplit/rows2.tsv \
  ./target/release/c2rs gap --list work/dc3-workload/files.txt \
    --flags-file work/dc3-workload/flags.txt --cwd ../dc3-decomp --jobs 8 \
    --jsonl work/w-divsplit/scan-cfg.jsonl

python3 work/w-divsplit/split.py work/w-divsplit/rows2.tsv          # the split
python3 work/w-divsplit/split.py work/w-divsplit/rows2.tsv --mutate 1  # calibration
python3 work/w-divsplit/shape.py work/w-divsplit/rows2.tsv          # §5, §8, §9

# the successor question
C2RS_SINK_CHAIN=op:05,op:06 ./target/release/c2rs gap … --jsonl …
```

From a worktree, set `C2RS_COMPILERS`, `C2RS_WIBO`, `C2RS_C2HOST`, `C2RS_C1HOST`
and give `--list` / `--flags-file` absolute paths into the main repo; the capture
cache resolves to the main repo on its own (`main_repo_root`).
