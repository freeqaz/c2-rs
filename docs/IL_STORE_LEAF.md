# The store leaf and the one-byte-unsigned value class — W25 + W26, landed

The three candidates this rung was commissioned against were
`expr-intrinsic-base-member-addr` (2117), `expr-load-type-8645` (float) and
`expr-load-type-8212` (`bool`/`unsigned char`), each picked because its
**`calls-0`** population looked leaf-shaped on the D6 frame axis and so takeable
without any frame work. All three were measured by counterfactual before
anything was implemented, and the ranking the row sizes suggested is not the one
the measurement produced:

| candidate | row | `calls-0` | released by the decode | **whole-body complete** |
|---|---:|---:|---:|---:|
| `expr-intrinsic-base-member-addr` | 118,331 | 27,139 | — | **740** |
| `expr-load-type-8645` (float) | 98,813 | 10,702 | 98,813 | **1,004** |
| `expr-load-type-8885` (double) | 82,810 | 2 | 82,810 | **0** |
| `expr-load-type-8212` (`bool`/`uchar`) | 29,245 | 7,203 | 52,650 † | **23,122** |
| the **store leaf** (this rung) | — | — | 23,646 | **22,821** |

† with its literal twin `expr-lit-type-8212` (23,405, of which 23,220 are
`calls-0`), which the same one-line type widening releases.

So: `8885` is refuted outright, `8645` is worth **1,004** (and that population is
the already-named FP `fmr` rung, `IL_CALL_IN_EXPR.md` §23.1, not a new one), and
the 2117 row — the largest of the three and the one the brief ranked first — is
worth **740 on its own**. What made it worth doing is what its `calls-0` bodies
*are*: a **store**, and the same store production reached through the *plain*
designator is **29× bigger** and lives in `expr-op-0x27`, the #1 row on the whole
board.

That is `IL_CALL_IN_EXPR.md` §19.3's finding a second time, at a bigger ratio:
**grep for every site that implements the rule you are changing.** D7 found the
address leaf's plain half was 5.0× its intrinsic half; here the store leaf's
plain half is 29×.

This document covers **both** rungs the measurement selected: the store leaf
(22,821 measured, **23,645 realized**, §1–§6) and the one-byte-unsigned value
class it ranked next (23,122 measured, **22,311 realized**, §9). Together
**+45,956**, census 418,628 → 464,584 (17.00 % → 18.87 %), mismatch 0 and
census/gate disagreement 0 at every step.

---

## 1. What the store leaf is — MEASURED by capture

`work/lf/probes/p1.cpp` and `p3.cpp` (gitignored scratch, regenerable from §8);
every word below read off the reference obj at the fixture profile
(`/Ox /GS- /c`) with `work/lf/tools/objdump.py`, not derived from an encoding
rule.

```text
  void s_a  (S* s, int v)       { s->a  = v; }   90830000  stw  r4,0(r3)   ; blr
  void s_b  (S* s, int v)       { s->b  = v; }   90830004  stw  r4,4(r3)   ; blr
  void s_p  (S* s, void* v)     { s->p  = v; }   90830008  stw  r4,8(r3)   ; blr
  void s_c  (S* s, char v)      { s->c  = v; }   9883000c  stb  r4,12(r3)  ; blr
  void s_sh (S* s, short v)     { s->s  = v; }   b083000e  sth  r4,14(r3)  ; blr
  void s_uc (S* s, unsigned char v)             98830010  stb  r4,16(r3)
  void s_q  (S* s, long long v) { s->q  = v; }   f8830020  std  r4,32(r3)  ; blr
  void s_e2 (S* s, int v)       { s->arr[2]=v; } 90830030  stw  r4,48(r3)  ; blr
  void s_bo (S* s, bool v)      { s->bo = v; }   98830038  stb  r4,56(r3)  ; blr
  void s_arg2(int x,S* s,int v) { s->b  = v; }   90a40004  stw  r5,4(r4)   ; blr
  void s_k  (S* s)              { s->a  = 7; }   39600007 91630000  li r11,7 ; stw r11,0(r3)
  void s_k0 (S* s)              { s->a  = 0; }   39600000 91630000  li r11,0 ; stw r11
  void s_lit(S* s)              { s->f  = true;} 39600001 99630000  li r11,1 ; stb r11
  void s_id0(S* s, int v)       { *(int*)s = v;} 90830000  stw  r4,0(r3)   ; blr
  void D::sb1(int v)            { b1 = v; }      90830004  stw  r4,4(r3)   ; blr   (2117)
  void t_sb1(Der* d, int v)     { d->b1 = v; }   90830004  the identical word
```

and **no `.pdata` entry**: the body is a leaf, exactly like the load and address
leaves it shares a designator with.

Four facts fall out, and each is a gate:

1. **The width picks the opcode and nothing else does.** `stb` / `sth` / `stw` /
   `std` come from the *stored* TYPE, which is the exact reverse of the address
   leaf, where the same field reaches no instruction at all
   (`IL_CALL_IN_EXPR.md` §19.2 fact 1). The three consumers of one designator
   therefore need three different rules from the same bytes: `is_ptr_to_4` +
   `SIZED_PTEE` for the load, `is_ptr_any` for the address, and the stored TYPE
   for the store.
2. **A floating-point value is a different instruction *and* a different
   register file.** `s->f = v` is `d0230014` (`stfs f1,20(r3)`) and `s->d = v` is
   `d8230018` (`stfd f1,24(r3)`). Both are 4 and 8 bytes wide, so a width-only
   rule would emit `stw r4` / `std r4` — wrong bytes inside an accepted class.
   Worse, the FP argument register is numbered over the FP parameters *alone*,
   which is the same off-by-one `float_leaf_text`'s header records as a live
   mis-emit. Refused, and sized: **0 functions** of the realized gain.
3. **A literal value goes through the SCRATCH register r11, never r3.** Read off
   the capture rather than assumed — a `void` function's r3 holds nothing the ABI
   cares about, so `li r3,7` would have been just as plausible. The wide form is
   the ordinary `lis`+`ori` pair through the same register.
4. **A conversion of the value is free only in the two 4-byte classes.**
   `void f(S* s, S* v){ s->p = v; }` converts `S*` to `void*` and emits the same
   bare `stw`; but `void M::setb(bool v){ m0 = v; }` — an `int` member, a `bool`
   parameter — carries the same-looking `2C 86 41 74 00` and emits
   `548b063e ; 91630000`, `clrlwi r11,r4,24 ; stw r11,0(r3)`. A real mask through
   the scratch register. This is the fourth time in this project a `2C` has been
   free on one axis and an instruction on another; the production admits it only
   where `eat_value_type` has been byte-graded since the getter rungs.

### 1.1 The production

```text
  <designator>                       B9 <tok> <PTR4>   |  the intrinsic-2117 form
  ( 33 <int-like> k 27 <PTR>         byte-offset adds, ANY number, summed
  | 33 <int-like> k 28 00 00 )*
  [ 2C <PTR> 00 ]                    a cv strip / array-to-pointer decay: the ADDRESS
  ( B9 <tok> <VT> | 33 <VT> <k> )    THE VALUE: a formal, or an integer literal
  [ 2C <VT'> 00 ]                    class-preserving, width 4 only
  32 <VT>                            the store; its TYPE restates the value's
  4B                                 statement end — and the body ends here
  <return plumbing, void, reaching the segment end>
```

Witness, `void s_b(S* s, int v){ s->b = v; }`, whole body from `LO`:

```text
4c 4f 11 53                       LO SS
b9 f9 09 86 43 81 20              LOAD s : S*
33 86 41 74 04  27 86 43 f4 08    + 4, address re-typed int*
b9 fa 09 86 41 74                 LOAD v : int
32 86 41 74                       STORE int
4b                                statement end
3a fc 09 54 02 29 fc 09           return plumbing (void)
4f 12 47 54 01 54 00              function tail
```

and the same store through the intrinsic-2117 designator
(`void D::sb1(int v){ b1 = v; }`), whose two literals are the member offset and
the base offset and whose sum is the displacement:

```text
33 86 41 74 80 45 08 00 00        selector 2117
40 86 43 f4 08                    intrinsic call -> int*
66 02 8f 20 91 20                 class-pair descriptor (LEB ids)
55 86 41 74
33 86 41 74 04  55 86 41 74       member offset 4
33 86 41 74 00  55 86 41 74       base offset 0
b9 61 0a 86 43 97 20  55 …  4c    the object pointer, applied
b9 62 0a 86 41 74  32 86 41 74 4b the value, stored
```

`parse_base_member_designator` and `eat_addr_offset_adds` are reused verbatim —
one fact, one locator. Nothing about either changed.

## 2. Where the population lives, and where it does NOT

The census key `expr-op-0x27` is **not** "a byte-offset add". It is the whole
grab-bag of bodies that reach `parse_expr` with a designator in hand — stores,
`float`/`double` getters, two-member binary ops, `p->a + p->b`. The store leaf
drains one production out of it and leaves the rest, which is why the realized
bucket drop (23,646) and this rung's gain (23,645) are within one function of
each other while the row itself only falls from 491,808 to 469,713.

| key | baseline | after | drop |
|---|---:|---:|---:|
| `expr-op-0x27` | 491,808 | 469,713 | **−22,095** |
| `expr-lit-type-8211` | 814 | 3 | **−811** |
| `expr-intrinsic-base-member-addr` | 118,331 | 117,591 | **−740** |
| `expr-lit-type-8422` | — | — | −1 |
| `opt-mode-00200001` | — | — | +2 |
| | | | **−23,645 net** |

`expr-lit-type-8211` falling 814 → **3** is the informative row: it is a *char
literal stored at offset 0* (`*p = 'x'`, `s->c = 7` on a first member), which
never reached the `27` at all because there is no offset add — the parse blocked
one token later, at the literal's own TYPE. A ranking that had looked only at
`expr-op-0x27` would have missed 811 functions sitting under a key whose name
says "literal type". (The 3 that remain are `calls-2plus`.)

The `+2 opt-mode-00200001` is not a loss: those two bodies now parse in class and
are refused on the *optimization word* instead, which is a post-parse gate. They
were previously blocked earlier in the same body.

## 3. The estimate, quoted before the outcome

> **Estimate: +22,821 exactly, biased LOW.** The counterfactual
> (`work/dc3-workload/scan-cf-store.jsonl`: the shape is parsed and then
> converted to a refusal in `parse_segment_detail`, so nothing is claimed in
> class and the baseline numerator is unchanged at 418,628) measured this
> population and the shipped gate is the same parse. Biased low by one named
> and *sized* cause: the counterfactual build did not yet admit the
> class-preserving `2C` on the **value**, which `w25_store_leaf.cpp`'s `s_pv`
> needs. High only if a TU changed class or a body lost.

**Outcome: +23,645**, i.e. **+824 above the point estimate, in the direction
stated**, and the residual is exactly the named cause: `expr-op-0x27` fell 22,095
against the counterfactual's 21,269, a difference of 826, and the ptr→ptr value
conversion is the only rule that changed between the two builds. Third
consecutive rung whose estimate landed with its bias direction right, and the
first whose *residual* was attributable to a single named rule rather than to a
correction factor.

## 4. What shipped

* **`c2_il::IlOp::StoreInd { off, width }`** — the third and last op of an exact
  three-op stream. Deliberately not representable for a floating-point value
  (see §1 fact 2).
* **`BodyShape::StoreLeaf`**, its own census key `store-leaf`, kept apart from
  `indirect-load-leaf` and `addr-leaf` for the reason those two are kept apart
  from each other: the three share a designator and emit three different
  instructions, so admitting one as another is a wrong-bytes emit rather than a
  gap — and so the gain can be checked against the individual bucket drops.
* **`shapes::try_parse_store_leaf`**, tried last in the `0xB9 | 0x33` arm. It is
  the only one of the four leaf productions that ends on a `32 <TYPE> 4B`
  statement rather than on a `41` result, so nothing above it can reach it and it
  can reach nothing above it.
* **`shapes::store_value_width`** — one locator over the two predicates that
  already answer "how wide is this value, and is it a GPR value at all":
  `value_class` for the two 4-byte classes, `sized_ptee` for 1/2/8. It is a
  function and not a width lookup *because* of the FP types.
* **`codegen::encode_stw/stb/sth/std` + `store_leaf_text`**, pattern-matched
  ahead of the ordinary selector exactly as `indirect_load_text` and
  `addr_leaf_text` are. `select_text` grew a backstop refusal for `StoreInd`.
* **No new TU-level gate, no `.pdata`, no frame.** Every accepted body is
  `calls-0` by construction.

## 5. Gate evidence

Corpus `dc3-decomp` at **`05ca6d09`**; baseline scan taken in this worktree,
`work/dc3-workload/scan-base.jsonl`, 878 rows, `fn_total` 2,462,571, in class
418,628 (17.00 %), 569 keys, 6 `match` / 7 `capture-fail`, mismatch 0 —
reproducing master `b36a046` to the function.

| | baseline | W25 | delta |
|---|---:|---:|---:|
| rows / `fn_total` | 878 / 2,462,571 | 878 / 2,462,571 | 0 |
| in class | 418,628 (17.00 %) | **442,273 (17.96 %)** | **+23,645** |
| mismatch | 0 | **0** | 0 |
| `match` / `capture-fail` | 6 / 7 | 6 / 7 | 0 |
| distinct keys | 569 | 569 | **0 new** |
| census/gate disagreement | 0 | **0** | 0 |
| sum of every blocker key's delta | — | — | **−23,645** |

| lane | baseline | W25 |
|---|---|---|
| `cargo test --workspace --release` | 370 pass | **372 pass**, 0 fail |
| `c2rs bench` | 132 pass / 0 fail / 0 error | **132 pass** (2 new fixtures added to both) |
| `mode_lane.sh /Ox` | 56 match, 0 mismatch, disagreement 1 | **57 match, 0 mismatch, disagreement 1** |
| `/O1` · `/O2` · `/Ox /Gy` | 54 match, 0 mismatch, 2 codegen-gap, disagreement 9 | **55 match, 0 mismatch, 2 codegen-gap, disagreement 9** |
| `scripts/expr_sweep.sh` | checked 4,706 | checked **4,830**, mismatches **0** (see §9.5 for the final figure with W26) |

Byte-graded, and the grading is not only census movement:

* `fixtures/cpp/w25_store_leaf.cpp` — **41/41 in class, whole obj byte-exact**
  against real `c2`. Both designators crossed against: every stored width
  (1/2/4/8 and a pointer), zero and nonzero displacements, the subscript-add run
  at one and two adds, the bare deref and the address cast, five literal values
  spanning the `li` / `lis`+`ori` boundary and both signs, argument slots 0–3 for
  the base and the value independently, member functions where `this` is in r3,
  the 32,764 displacement edge, two inheritance steps (`66 03`), and the three
  accepted neighbours (`lwz`, `addi`, bare `blr`) that this production must not
  swallow. Three byte-identical bodies with different neighbours between them are
  the §17.3 locality tell — all three emit `90830004`.
* `fixtures/cpp/w25_store_leaf_neg.cpp` — **0/15 in class**, and the file must
  never mismatch. The float and double values, a bool→int and an int→char
  conversion of the value, a computed value, two stores in one body, the store's
  result returned, a 32,768 displacement, a global destination, a variable index,
  an aggregate assignment, a memory-to-memory copy, the base in the ninth
  parameter slot, and the wide-negative literal of §5.1.
* `scripts/expr_sweep.sh` grew **124 cases** (4,706 → 4,830, mismatches 0) over
  exactly those axes: 11 stored types × (formal, literal); the base at argument
  slots 0–7 crossed with two widths; eleven literal values across the `li`
  boundary and the wide-negative bound crossed with two widths; the
  subscript/deref/cast forms over four pointee widths; the intrinsic designator at four widths × (one step, two steps,
  non-first register, member function); a store beside each of five accepted
  neighbours in both orders; and 14 refusing neighbours, each alone and beside an
  emitted leaf.

### 5.1 One census/gate hole, found by probing the production's own boundary

Not by a test — nothing tested it, which is the point. The parser admitted any
`Lit(k)` as a stored value, and `codegen::emit_load_imm` refuses a **wide
negative** constant (`lis`+`ori` covers non-negative only). So

```text
  void f(S* s){ s->a = -70000; }     census: 1/1 in class   Port=NotImplemented
```

— the census over-claiming, in the exact shape `IL_CALL_IN_EXPR.md` §24.7 records
and `crates/c2-harness/tests/census_gate.rs` exists to keep at zero. The
straight-line class already gated it **in the parser**
(`expr-out-of-class-wide-neg-lit`, `chain::straight_line_out_of_class_ctx`); the
new shape reached the same literal by a second route and did not. `GAPS.md` §6's
"one fact, two locators", fifth instance.

Fixed in the parser rather than in codegen, per §6c's invariant. It costs **0
functions on the workload** (the re-scan is identical to the function, 464,584,
disagreement 0), and `w25_store_leaf_neg.cpp`'s `n_negwide` plus 2 sweep cases
now pin it, with `s_kwide` (+70000) and `s_kneg` (−3) as the neighbours it must
not take with it.

The direction the brief asked to be watched — codegen refusing what the census
admits **and what the emitter would emit** — cannot happen for either of these
rungs, and the reason is structural rather than tested: `PortC2::build` (both
sectioning modes, `c2-core/src/lib.rs:274` and `:353`) and
`codegen::function_gate` both dispatch through the *same* `select_function`, and
`store_leaf_text` returns a plain `Selected::Plain` that both arms handle
identically. There is one dispatch, so there is nothing for the two to disagree
about.

## 6. What is NOT established, labelled

* **The counterfactual is a grammar measurement, not a differential.** No TU
  flipped under it, so no lane and no sweep graded that build; its `mismatch 0`
  is a statement about TU-level acceptance only. It was reverted and every number
  in §5 re-taken against the shipped tree.
* **`mismatch 0` on the workload is still a TU-level statement.** 865 of the 878
  TUs are `vocab-gap` and never reach the port. The byte grading is the two
  fixtures (55 functions), the 4,828-case sweep and the four lanes.
* **The `28` payload `00 00` remains undecoded.** Required literally, exactly as
  in `try_parse_indirect_load_leaf` and `try_parse_addr_leaf`.
* **The `2C` on the value is admitted only at width 4**, and the narrow case is
  not merely unimplemented — it is *measured* to cost an instruction
  (`clrlwi`). What that refusal costs is not separately sized; it is inside the
  1,652-function gap between the lax and strict `8212` counterfactuals in §7.
* **The FP store is measured and not implemented.** `stfs`/`stfd` are two
  encoders; what stops it is the FP argument-register numbering, which is the
  same `.sy` type-kind plumbing the FP `fmr` rung needs (`IL_CALL_IN_EXPR.md`
  §23.1). Doing them together is the obvious pairing.
* **`store_value_width` treats `bool` and `unsigned char` as one thing**, because
  their TYPE `<tag><kind>` is one thing (`82 12`) and only the id differs. Both
  are `stb`; no capture separates them here.
* **The "two stores in one body" neighbour is the dominant shape in the 2117
  row's `calls-0` population**, not the single store. Sampling 87 workload TUs,
  the recurring wild shape is a setter that writes a member *and then* sets a
  dirty flag with an `0x19` compound-assign op. That is why the intrinsic half of
  this rung is 740 and not 27,139, and it is the ranked next item in §7.

## 7. The order of work, re-ranked — with the frame axis applied

Measured on `work/dc3-workload/scan-w23.jsonl` (878 TUs, corpus `05ca6d09`),
over the 2,020,298 still-blocked functions.

1. **`expr-load-type-8212` + `expr-lit-type-8212` — 52,650 released, 23,122
   whole-body complete, 22,313 of them `calls-0`.** MEASURED by counterfactual
   (`C2RS_CF=boolnc`, §8), and it is the largest takeable item on the board. The
   family is "`bool` / `unsigned char` is a 4-byte-register value like an int in
   the LOAD / LIT / result positions", and **its lowering is no instruction at
   all**: `bool f(){ return false; }` is `li r3,0`, `bool f(bool b){ return b; }`
   is a bare `blr`, `bool f(int k, bool b){ return b; }` is the register move
   `mr r3,r4` the port already emits, and a `bool`-returning tail call is the
   same `b callee`. The one hazard is sized: a `2C` **out of** the class is a
   real `clrlwi` (`unsigned u(bool b){ return b; }` is `5463063e`), and refusing
   every such body costs 1,652 of the 24,774 the lax counterfactual admits — so
   the strict rung is 23,122 and the mask is a separate, smaller one. The work is
   a new `ValueClass` and the discipline of **not** widening `eat_int_like`,
   whose five call sites `ROADMAP.md` §6d already found the hard way.
2. **The two-statement setter — the 2117 row's real `calls-0` content.** The
   shape is `p->Base::m = v; p->flags |= k;` and it needs the `0x19`
   compound-assign op plus a two-statement body. It is what is left of the 27,139
   after this rung took 740, and it is the shape `IL_CALL_IN_EXPR.md` §19.1
   measured at 13.4 % of the block *before* D7 and this rung drained the two
   whole-body forms around it. Decompose the operator before ranking: the census
   does not carry an operator histogram for this key.
3. **The FP store + the FP `fmr`, together.** 1,005 (`fmr`, §23.1) plus whatever
   the FP store is worth, which is unmeasured because the parser refuses it at
   `store_value_width` and the counterfactual was not run for it. Both need the
   same thing — the FP argument-register number, which counts FP parameters
   alone — and both are one encoder past that.
4. **`expr-load-type-8645` is worth 1,004 and `expr-load-type-8885` is worth 0.**
   Both are MEASURED here by counterfactual, and both should be struck off any
   ranking taken from row size: 98,813 and 82,810 blocked functions between them,
   and the double row completes **zero** bodies under a full type widening
   because its population is `2C`-converted call arguments
   (`+44,050 call-end-0x88`, `+38,756 expr-convert-target-8885` when the type is
   admitted). This is the fifth top-of-the-board row this project has measured
   into single digits of percent.
5. **The general frame**, unchanged and still first by size: 802,655
   `calls-2plus` functions, none of which any leaf rung can reach.

### 7.1 What this rung leaves ready for the frame rung

The brief asked this to be sized. The store leaf itself leaves **nothing** behind
the frame axis: all 23,645 admitted bodies are `calls-0`, and the production
cannot describe a body containing a call at all. What it does leave is the
*designator*: `parse_base_member_designator` + `eat_addr_offset_adds` now have
three graded consumers instead of two, so a framed body that stores into a
sub-object needs no new address decode — only the frame.

Sized on the two rows this rung touched, at the scan taken after it:

| key | remaining | `calls-0` | `calls-1` | `calls-2plus` |
|---|---:|---:|---:|---:|
| `expr-op-0x27` | 469,713 | 278,729 | 74,866 | 116,118 |
| `expr-intrinsic-base-member-addr` | 117,591 | 26,399 | 56,186 | 35,006 |
| `expr-load-type-8645` | 98,813 | 10,702 | 84,215 | 3,896 |
| `expr-load-type-8885` | 82,810 | 2 | 82,806 | 2 |

The **`calls-1` mass behind `8645` and `8885` is 167,021 functions, 99.9 % of
those two rows' non-`calls-0` content**, and the §7 (4) counterfactual says what
it is: a `2C`-converted FP value in a *call-argument* region
(`call-end-0x88` + `expr-convert-target-8885` alone account for 82,806 of the
double row). That is not a frame problem — a single-call body with an FP argument
is a tail call — so the convergence to size is with the **FP argument-register**
item (§7 (3)), not with the frame rung: decoding the FP value class is what makes
that 167,021 reachable, and the frame is what makes the 119,014 `calls-2plus`
functions in the two `27`-family rows reachable.

## 8. Reproduction

```sh
cargo build --release
cargo test --workspace --release                                # 373 pass
C2RS_JOBS=12 ./target/release/c2rs bench                        # 134 pass 0 fail 0 error
./target/release/c2rs census fixtures/cpp/w25_store_leaf.cpp      # 41/41 in class
./target/release/c2rs diff   fixtures/cpp/w25_store_leaf.cpp      # Port=Match
./target/release/c2rs census fixtures/cpp/w25_store_leaf_neg.cpp  # 0/15 in class
./target/release/c2rs census fixtures/cpp/w26_bool_value.cpp      # 15/15 in class
./target/release/c2rs diff   fixtures/cpp/w26_bool_value.cpp      # Port=Match
./target/release/c2rs census fixtures/cpp/w26_bool_value_neg.cpp  # 0/10 in class
./target/release/c2rs diff   fixtures/cpp/w25_store_leaf_neg.cpp  # Port=NotImplemented
C2RS_JOBS=12 scripts/mode_lane.sh /Ox                           # 58 match, 0 mismatch
C2RS_JOBS=12 scripts/mode_lane.sh /O1                           # 56 match  (also /O2, "/Ox /Gy")
C2RS_JOBS=12 scripts/expr_sweep.sh                              # checked=4900 mismatches=0
./target/release/c2rs gap --list work/dc3-workload/files.txt \
  --flags-file work/dc3-workload/flags.txt --cwd <dc3-decomp> --jobs 12 \
  --jsonl work/dc3-workload/scan-w24b.jsonl       # 464584/2462571, 570 keys, disagreement 0
# the lowering, read off the reference obj rather than inferred:
./target/release/c2rs compile work/lf/probes/p1.cpp --keep-obj work/lf/p1.obj
python3 work/lf/tools/objdump.py work/lf/p1.obj
# the counterfactuals (scratch, reverted; nothing is claimed in class by any of
# them — `parse_segment_detail` converts the successful parse into a refusal
# under its own census key, and the numerator stays at 418,628 in every build):
#   store  : return Err(ctx="cf-store") for Ok(BodyShape::StoreLeaf) -> 22,821
#   type   : widen `eat_int_like_or_ptr4` in `parse_expr`'s B9/33 arms and in
#            `eat_return_head`'s `41` gate to one extra <tag><kind> pair chosen by
#            an env var, set a thread-local when it fires, and sink any body that
#            then parses to the end:
#              C2RS_CF=bool    -> 24,774     C2RS_CF=float  -> 1,004
#              C2RS_CF=boolnc  -> 23,122     C2RS_CF=double -> 0
#            (`boolnc` additionally refuses any `2C` applied to the widened value,
#             which is the difference between the lax and the shippable rung)
# the sub-shape survey the characterization started from (a /tmp crate with a
# path dependency on c2-il, no repo edits): capture 87 stride-sampled workload
# TUs with `c2rs census <tu> --keep-il work/lf/il`, re-split `.ex` on the `LO`
# anchor, and dump whole segments for a chosen census key and frame class.
```

Always difference the scans through **absolute** paths and print each one's row
count and `fn_total` first: `work/dc3-workload/scan-*.jsonl` exists in several
reflinked worktrees with different contents, and reading one through a relative
path has already produced a published wrong number in this project
(`IL_CALL_IN_EXPR.md` §18.8).

---

## 9. W26 — the one-byte-unsigned value class, landed

§7 (1) ranked this first and sized it at **23,122 whole-body complete, 22,313 of
them `calls-0`**. It shipped as the `calls-0` half: **+22,311**, census
442,273 → 464,584 (17.96 % → **18.87 %**).

### 9.1 What it is — MEASURED by capture

`bool` and `unsigned char` share the operand TYPE `82 12 <id>` — one class,
because only the per-TU id differs and no capture separates them in any position
this parser reaches. **Inside** the class a value costs no instruction at all
(`work/lf/probes/p2.cpp`, `p3.cpp`, `p4.cpp`):

```text
  bool k_false()                    38600000  li r3,0    ; blr
  bool k_true()                     38600001  li r3,1    ; blr
  unsigned char k_uc()              386000c8  li r3,200  ; blr
  bool b_id(bool b)                           blr                  (already r3)
  bool b_r4(int k, bool b)          7c832378  mr r3,r4   ; blr
  bool b_r10(…7 ints…, bool h)      7d435378  mr r3,r10  ; blr
```

Every one of those is a word the ordinary integer selector has emitted since the
MVP (`li` for a literal, nothing for the r3 identity, the W18 register move
otherwise), so **the rung is a decode widening with no emitter change at all** —
the second such in this project after D12's conversion.

**Out of** the class it is not free, and that is the whole gate:

```text
  unsigned u_from_b(bool b) { return b; }   5463063e  rlwinm r3,r3,0,24,31
  int      i_from_b(bool b) { return b; }   5463063e  the same mask
  unsigned char uc_add(unsigned char a, unsigned char b)
                                            546a063e 548b063e 7d6a5a14 5563063e
  bool     b_not(bool b)    { return !b; }  546b063e 7d6a0034 5543dffe
```

The mask arrives on the **same `2C … 00` token** that is free between the two
width-4 classes (`w20_convert.cpp`). That is why `ValueClass::Int1u` is its own
class rather than a spelling of `Int4`, and why the `41` result annotation is
required to *restate* it: a `bool` value annotated `int` is the mask, and
admitting it as a register move would be wrong bytes inside an accepted class.

### 9.2 What shipped

* **`ValueClass::Int1u` + `is_int1u_type`** (`82 12` literally — the cv spellings
  `A2 12` / `92 12` are not admitted, because neither occurs as an operand in any
  capture taken here and a tag that never varied is indistinguishable from a
  constant). `82 11` — `char` / `signed char` — is deliberately a *different*
  class: same width, different signedness, and one predicate per fact.
* **`eat_operand_type`**, a new entry point rather than a widening of
  `eat_int_like_or_ptr4`. That locator has five call sites and gates three
  byte-graded shapes; `ROADMAP.md` §6d is the record of what changing a shared
  locator costs, and only the two `parse_expr` operand positions were graded for
  this class.
* **`parse_expr_classed`** returns the sub-expression's class, and two new guards
  refuse a `bool` that is *computed on* (`expr-int1u-arith`) or *mixed* with a
  width-4 operand (`expr-int1u-mixed`). Both refuse under their own census keys
  so the guards' cost is a number — and **the number is 0 on the whole workload**,
  which is the C++ rule showing through: every `bool` arithmetic converts first,
  so a raw chain over the class has no witness anywhere in 2.46 M functions.
* **The `41` result annotation** is consumed by the straight-line arm itself when
  the class is `Int1u`, so `eat_return_head`'s shared gate is untouched.
* **No codegen, no new `BodyShape`, no new census key for the accepted class** —
  these bodies are `straight-line`.

### 9.3 The estimate, quoted before the outcome

> **Estimate: +23,122, biased HIGH.** The counterfactual widened *three* sites —
> both `parse_expr` operand positions and `eat_return_head`'s `41` gate globally
> — while the shipped rung widens only the straight-line arm, so the 809
> `calls-1` bodies in the counterfactual (a `bool`-returning tail call) are
> expected to be lost. Predicted realization ≈ 22,313, the `calls-0` half.

**Outcome: +22,311**, i.e. **2 functions off the `calls-0` prediction and 811
below the counterfactual**, in the direction stated and for the stated reason.
The 809 tail calls stayed blocked (`call-args-none:eof` and `call-end-0x82`,
which are the *call* path's own gates and were not part of this rung).

### 9.4 What the refusals cost, sized rather than waved at

Measured on `work/dc3-workload/scan-w24.jsonl`:

| key | functions | `calls-0` | what it is |
|---|---:|---:|---|
| `expr-convert-target-8642` | 4,906 | 1,652 | `bool` → `unsigned`: the `rlwinm` mask |
| `expr-convert-target-8641` | 41 | 24 | `bool` → `int`: the same mask |
| `expr-load-type-8211` | 1,646 | 1,645 | `char` / `signed char`, the *other* one-byte class |
| `expr-int1u-arith` · `expr-int1u-mixed` | **0** · **0** | — | the two guards, and they cost nothing |

The mask is one encoder (`encode_rlwinm` already exists) over a
(source × target) matrix that has not been graded; `82 11` is a bare `blr` today
and would be free to admit **now**, which is exactly why it refuses — the two
one-byte classes part company one token later, where the unsigned widening is a
`rlwinm` and the signed one an `extsb`.

### 9.5 Gate evidence

| | after W25 | W26 | delta |
|---|---:|---:|---:|
| rows / `fn_total` | 878 / 2,462,571 | 878 / 2,462,571 | 0 |
| in class | 442,273 (17.96 %) | **464,584 (18.87 %)** | **+22,311** |
| mismatch | 0 | **0** | 0 |
| `match` / `capture-fail` | 6 / 7 | 6 / 7 | 0 |
| distinct keys | 569 | 570 | +1 |
| census/gate disagreement | 0 | **0** | 0 |

| lane | W25 | W26 |
|---|---|---|
| `cargo test --workspace --release` | 372 pass | **373 pass**, 0 fail |
| `c2rs bench` | 132 pass | **134 pass**, 0 fail, 0 error |
| `mode_lane.sh /Ox` | 57 match, 0 mismatch, disagreement 1 | **58 match, 0 mismatch**, disagreement 1 |
| `/O1` · `/O2` · `/Ox /Gy` | 55 match, 0 mismatch, 2 codegen-gap, disagreement 9 | **56 match, 0 mismatch**, 2 codegen-gap, disagreement 9 |
| `scripts/expr_sweep.sh` | checked 4,830 | checked **4,900**, mismatches **0** |
| `census fixtures/cpp/w26_bool_value.cpp` | — | **15/15 in class**, `Port=Match` |
| `census fixtures/cpp/w26_bool_value_neg.cpp` | — | **0/10 in class**, `Port=NotImplemented` |

`w26_bool_value.cpp` crosses both spellings against five literal values, the
identity from r3 and from argument slots 1, 2, 3 and 7, and the three accepted
neighbours that share the class's bytes (the T3 `lbz` getter, the W25 `stb`
store, and an ordinary int literal). `w26_bool_value_neg.cpp` is the more
load-bearing file — the positives cost no instruction, so everything that can go
wrong is a refusal: both conversions out of the class, arithmetic, `!`, `&&`,
`char` and `signed char`, the local-variable path, the tail call, and a `bool*`
pointee feeding arithmetic.

### 9.6 The board after both rungs

| # | functions | `calls-0` | `calls-1` | `calls-2plus` | key |
|---:|---:|---:|---:|---:|---|
| 1 | 469,713 | 278,729 | 74,866 | 116,118 | `expr-op-0x27` |
| 2 | 280,283 | 0 | 114,059 | 166,224 | `expr-op-0x99` |
| 3 | 141,800 | 15 | 57,894 | 83,891 | `expr-intrinsic-this-adjust` |
| 4 | 117,591 | 26,399 | 56,186 | 35,006 | `expr-intrinsic-base-member-addr` |
| 5 | 98,813 | 10,702 | 84,215 | 3,896 | `expr-load-type-8645` |
| 6 | 82,810 | 2 | 82,806 | 2 | `expr-load-type-8885` |
| 7 | 48,102 | 1,622 | 4,674 | 41,806 | `body-0x29` |
| 8 | 39,366 | 0 | 2 | 39,364 | `expr-call-in-expr-op-0x9B` |
| 9 | 34,795 | 7,318 | 12,682 | 14,795 | `expr-intrinsic-memset` |
| 10 | 32,381 | 4 | 31,485 | 892 | `expr-bit-and` |

Rows 5 and 6 are **measured at 1,004 and 0** whole-body complete (§7 (4)) and
should be struck from any ranking taken from size. Row 1 is a grab-bag whose
next production is the two-member binary op (§7 (2)); rows 2, 3, 8 and 10 are
overwhelmingly framed. The ranked work is §7 (2)–(5), plus three items this rung
sized on its way past:

* the `bool` **tail call** — 809 functions, blocked in the *call* path
  (`call-args-none:eof`, `call-end-0x82`), which this rung did not widen;
* the `bool` **local** — the assignment-body parser calls `parse_expr` (which
  discards the class) and then the shared `41` gate, so it refuses one token
  later; part of `assign-dst-not-formal:eof`'s 13,887;
* the **mask** and the **`char` class** — 4,947 and 1,646 (§9.4).

---

## 10. The merged tree — W25 + W26 against master `7011b49`

Both rungs were developed against master `b36a046` (census 418,628). Master
advanced four times while they were in flight: D14 (`.gl` record separator,
+9,027), a ground-truth docs drop, instrument hardening (the capture cache,
provenance record 0, per-lane pinned disagreements), and the frame model
(#35 step 1, `FrameLayout` + argument setup + the comparison label-stride table).
The merged configuration is one no prior run covered, so the whole gate was
re-run on it.

**Corpus `dc3-decomp` at `05ca6d09`** — unchanged, and now carried in the scan's
own provenance record 0 (`workload_head 05ca6d09…`, `wibo_stale false`,
`wibo_known_good 1.0.1-23`).

### 10.1 The census, and additivity MEASURED rather than assumed

| tree | in class | % |
|---|---:|---:|
| this rung's base, master `b36a046` | 418,628 | 17.00 |
| + W25 + W26 (this document) | 464,584 | 18.87 |
| master `7011b49` (with D14) | 427,655 | 17.37 |
| **merged** | **473,611** | **19.23** |

473,611 = 427,655 + 45,956 = 464,584 + 9,027, and the second identity is the one
that was *measured* key by key rather than inferred. Differencing this document's
own tree against the merged scan moves **exactly two keys**:

| key | delta |
|---|---:|
| `callee-unresolved-dtor-delegation:eof` | **−9,028** |
| `callee-unresolved-tail-call:eof` | +1 |
| | **−9,027 net** |

That is D14's population and nothing else: no key this rung touched moved, and no
key moved that neither rung named. The two are independent, and the interaction
term is **0**.

### 10.2 Gate evidence on the merged tree

| lane | master `7011b49` | merged | delta |
|---|---|---|---:|
| `cargo test --workspace --release` | — | **398 pass, 0 fail** | — |
| `c2rs bench` | 138 pass | **142 pass / 0 fail / 0 error** | +4 fixtures |
| `mode_lane.sh /Ox` | 61 match, 0 mismatch, disagreement 1 | **63 match, 0 mismatch**, disagreement **1** | +2 |
| `/O1` · `/O2` · `/Ox /Gy` | 59 match, 0 mismatch, 2 codegen-gap, disagreement 9 | **61 match, 0 mismatch**, 2 codegen-gap, disagreement **9** | +2 |
| `scripts/expr_sweep.sh` | 4,829 cases | **5,023 cases, 0 mismatches** | +194 |
| 878-TU scan | 427,655, disagreement 0 | **473,611 / 2,462,571 (19.23 %)**, match 6, **mismatch 0**, capture-fail 7, disagreement **0**, 570 keys | +45,956 |

The master lane figures were taken in this worktree with this rung's four
fixtures moved aside, so the `+2` is attributable rather than assumed: it is
`w25_store_leaf.cpp` and `w26_bool_value.cpp`, one match each, in every lane.

**`census_gate.rs` passes at its recorded per-lane values (1 packed / 9 `/Gy`)
with its named causes unchanged.** That is the honest outcome rather than an
edited one: this rung's gates all live in the parser, so they add nothing to
either lane's residual, and the test's assertions did not need to move.

Both this rung's fixtures still grade N/N — `w25_store_leaf.cpp` **41/41
`Port=Match`**, `w26_bool_value.cpp` **15/15 `Port=Match`**, the two negatives
**0/15** and **0/10** `Port=NotImplemented` — and the frame rung's do too:
`wfr_argreg` **4/4**, `wfr_argreg_member` **2/2**, `wfr_argreg_types` **7/7**,
`wfr_cmp_stride` **13/13**, all `Port=Match`.

### 10.3 The two resolutions worth recording

* **`encode_std` was defined twice** after the merge — once here for the `long
  long` store leaf (captured as `f8830020`, a member at offset 32) and once by
  the frame model for the callee-saved GPR prologue (captured as `fbe1fff0` =
  `std r31,-16(r1)`). Byte-identical encoders, two independent captures. The
  frame side's definition is kept **untouched** (a concurrent agent owns that
  region) and this rung's duplicate is removed, with its witness recorded in
  `store_leaf_text`'s own table and a pointer left where the duplicate was. Git
  did not flag this — it is a semantic conflict inside a cleanly auto-merged
  file, and only the build caught it.
* **The rung tag `W23` was taken twice.** D14's fixtures are
  `w23_gl_callee_bind*.cpp` and this rung's were `w23_store_leaf*.cpp` /
  `w24_bool_value*.cpp`, developed in parallel from the same base. There is no
  filename collision, but the *label* is ambiguous in a ledger that indexes rungs
  by tag, so this rung renumbered to **W25 + W26** — the same reason its ROADMAP
  section renumbered from §6f to §6i. D14's references were left alone.

### 10.4 What was NOT decided here

The frame model's `select_function` framed path and `framed_call_text` merged
cleanly and are untouched by this rung beyond the `encode_std` de-duplication;
`framed-arg-over-eight-formals` and the other new parser-side gates live in
`parse_call_shape`, which the store leaf's dispatch arm never reaches. If a
resolution in that region turns out to be needed it belongs to the framed side,
not here.
