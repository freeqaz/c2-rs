# WEC — the empty base-delegating constructor, and the label slot the cheap side of EH was already owed

    Tag:       WEC
    Slug:      ctor-base-delegation
    Date:      2026-07-31
    Fixtures:  wec_ctor_base.cpp wec_ctor_base_neg.cpp
    Census:    685,882 → 691,744 (27.85 % → 28.09 %), +5,862
    Record:    this file; the estimate in work/WEC/ESTIMATE.md (gitignored)

Two things land together, and the second is why the first could not land alone.

## 1. The alarm: `eh-bare` costs a label slot, and the port was not paying it

`docs/EH_RECORDS.md` §8.5d measured the surcharge on **one** row —
`void P(){ SE s; }`, a *framed* body — and recorded it as
*"`eh-bare` costs +1 label slot PER FUNCTION"*. The port's three shipped
`empty-dtor-*` shapes are `eh-bare` **leaves**, which that row cannot speak to,
and `IlFunction::label_slots` returned **1** for them.

Measured here, `scripts/gt_label_stride.py` (four new rows, seed-free and in-TU,
the anchor control holding on every one):

| probe | what P is | `/EHsc` extra | `/EHsc` stride | no-`/EH` stride |
|---|---|---:|---:|---:|
| `eh-bare-dtor` | `One::~One(){}`, one member — a bare branch | – | **2** | 1 |
| `eh-bare-dtor-led` | same, behind another `eh-bare` function | – | **2** | 1 |
| `eh-bare-dtor-adj` | member at offset 4 — `addi ; b` | – | **2** | 1 |
| `eh-bare-dtor-deleg` | `D1::~D1(){}`, the base form | – | **2** | 1 |
| `eh-bare-ctor` | this rung's constructor, framed | **1** | **6** | 5 |
| `eh-bare-ctor-led` | same, behind an `eh-bare` function | **1** | **6** | — |
| `eh-bare-ctor-ehled` | same, behind a full EH function | **1** | **6** | — |
| `eh-none-ctor-ctl` | **the control** — same body, base with no destructor | **0** | **5** | 5 |

Four facts, and only the first was already known:

* **It is per function, not per TU** — the `-led` rows charge it again, in both
  the leaf and the framed family.
* **It applies to a LEAF.** The three `empty-dtor-*` shapes are 35,964 already
  in-class functions on the workload, every one of them `eh-bare`.
* **It is keyed on `/EHsc`, and the IL says so.** `/EH…` clears bit `0x10` in
  both the `5C` statement trailer's flag and the `5D`/`5E` count trailer's, so
  `(0x11, 0x31)` is the no-EH profile and `(0x01, 0x21)` the workload's. The flag
  is read out of **that byte**, never out of the compile flags: the IL bundle
  does not record argv and `plan_labels` cannot see them.
* **`eh-none-ctor-ctl` separates it from the shape.** The identical constructor
  over a base with no destructor prints 5 at `/EHsc`. Without that cell the `+1`
  could just as well have been a property of the constructor.

**It was live, not latent.** `work/WEC/live/t1.cpp` — five lines, a generated
destructor ahead of an ordinary framed call, at the workload's own flags —
graded `mismatch`, first divergence at file offset **1039**, both objs 1,221 B:
the `$M`/`$T` names, six wrong bytes in an obj that still links, visible only to
a *following* function in the same TU. It is byte-exact now.

The repair is one number in one place. `IlFunction::label_lead` gains
`u32::from(eh_bare)`, `coff::plan_labels` already adds the lead before it looks
at the frame, and the TU-level gate in `bundle.rs` changes from `!= 1` to
`!= label_lead() + 1` — because the gate's job is "does this class's stride agree
with what `plan_labels` will advance", and written as `!= 1` it would have turned
the wrong-bytes emit into a wholesale refusal of every `/EHsc` TU containing a
generated destructor.

**Nothing here rests on `docs/EH_RECORDS.md` §6/§7's cheap/EH predicate**, which
was refuted mid-rung by a peer lane (the boundary is `maxState >= 1` over the
live-object sets at outbound control transfers, not a statement count). The
surcharge is set by the two *grammars* that produce it, from the trailer byte in
their own bodies, and both require the count trailer to be `01` — so no admitted
body can carry a second tracked object, and "per function or per state" cannot
arise inside the class. It is `NOT MEASURED` outside it.

## 2. What the rung admits, and what it refuses

`struct D : B { D(); };  D::D() {}` — the empty constructor that delegates to
**one** base sub-object. The mirror of the generated destructor's base form,
sharing its receiver recognizer (`this`-adjust intrinsic 2113, adjust 0) and none
of its lowering: an MSVC constructor hands `this` back in r3, `this` is live
across the base constructor's `bl`, so c2 frames the body.

```
  ??0Ka@@QAA@XZ:  48 B, F = 96, function symbol Value = 0
    mflr r12 ; stw r12,-8(r1) ; std r31,-16(r1) ; stwu r1,-96(r1)
    mr r31,r3 ; bl ??0B1@@QAA@XZ ; mr r3,r31
    addi r1,r1,96 ; lwz r12,-8(r1) ; mtlr r12 ; ld r31,-16(r1) ; blr
```

That is Class B with one saved formal and one new tail, `SeqTail::SavedFormal` —
`mr r3, rSaved`, in the position `CallValue`'s `addi` post-op occupies. It is
**not** `CallValue { add_k: 0 }`: that tail returns what the `bl` left in r3,
this one overwrites r3 with a value that predates the call. Folding them would
emit nothing at all here and return the base constructor's result, which happens
to be the same pointer on this ABI — so the obj would still link and still run,
and the four missing bytes would show up only as a length mismatch.

**Two forms, one emitter.** At `/EHsc` and over a base *with* a destructor the
statement carries a second half: a `26 <base dtor>` unwind action over the same
receiver, a `5C <OBJ> 01` and a `5D 01 21`. It emits **nothing** — no `bl`, no
relocation, **no symbol**, measured on `work/WEC/probe/p2.obj`. Over a base with
no destructor none of it is present. The two `.text` are byte-identical and
differ by one label slot.

That unwind destructor is a `.gl` name the obj legitimately does not carry, which
the TU-level unclaimed-symbol gate would otherwise refuse the whole TU for. It is
accounted by `IlFunction::eh_unwind_callees` — named as an exception rather than
by loosening a gate whose reading is right everywhere else.

### The refusals, each with its measured count

Instrumented build, every `return None` traced, over three real dc3 TUs
(`MoveDir.cpp`, `MetagameRank.cpp`, `BustAMovePanel.cpp`), 1,068 candidate
bodies that reached the recognizer:

| refusal | n | what it separates |
|---|---:|---|
| the `30 <OBJ>` type is not a class (kind ≠ `0x46`) | **505** | not this production at all |
| the `BD` result is not a 4-byte pointer | **237** | a `void` result is the *destructor* form |
| an explicit argument is not a bare `B9` load | **237** | a literal (`li r4,k`) or a computed value |
| the bind is not `99` | **23** | `9A` is virtual dispatch, through the vtable |
| a forwarded **floating-point** argument | **3** | see below — it was a live mismatch |
| a malformed `55` argument terminator | **3** | |

Six rules, six counts, and they separate. Further refusals with no witness in
that sample but a fixture each in `wec_ctor_base_neg.cpp`: a polymorphic derived
class (the base moves to offset 4 *and* the vfptr store is a second statement,
64 B not 48), two destructible bases (`eh-multi`, two `bl`s in reverse
declaration order), a permuted forwarding, an argument that is a formal past the
argument count, a widening conversion, and a member initializer beside the base.

### The FP argument was a live wrong-bytes emit, found by hand

`D::D(int a, float f, void* p) : B(a, f, p) {}` and the five-`double` form were
`Port=Mismatch @ offset 12` — the COFF header's `NumberOfSymbols`, one symbol
short — **in all five modes**. The obj carries `_fltused`; the body has no other
floating-point tell, and `IlFunction::touches_floating_point` enumerates *shapes*.

Two things generalize:

* **An unused FP formal costs nothing.** `int f(double d, int a){ return a+1; }`
  is byte-exact today. It is *passing* the value that mints `_fltused`, which is
  a fifth producer and is exactly W36's shape: FP-touching without being
  FP-*shaped*.
* **Nothing else found it.** The workload scan says the refusal costs **0**
  functions on 878 TUs, and the generated sweep had no such case until this rung
  added one. Two hand-written adversarial probes found it. This is the mirror of
  §6n's *"generated sweep axes found six live wrong-bytes emits; hand-written
  fixtures found none"* — the two corpora fail in opposite directions and both
  are load-bearing.

## 3. Estimate vs outcome

| | |
|---|---|
| predicted, `expr-intrinsic-this-adjust` ∧ `eh-bare` (§7.4) | **6,875**, range 2,000–6,875, bias HIGH |
| realized **from that cell** | **7** |
| realized **total** | **5,862** |

**982x over on the named row, and the rung is nonetheless 5,862.** The row is
exactly what §7.4 says it is; the *production* it names straddles the EH marker,
and **99.88 % of the production is on the no-marker side** —
`eh-none|empty-ctor-base` 5,855 against `eh-bare|empty-ctor-base` 7. A
constructor whose base has a destructor is a constructor whose derived class
almost always does something else as well, and that pushes it to `eh-plus-stmt`
or `eh-multi`. §6o's *"the same census key straddles the boundary"* is true in
this direction too, and much harder: here the key straddles it **99.9 % / 0.1 %**,
so sizing from the crossed cell was worse than sizing from the uncrossed row
would have been.

**What sized it correctly, in 1.3 seconds.** The estimate named six refusals and
called #1 — "the base constructor may take arguments" — the largest doubt. It
was the whole of it: **5 functions with the zero-argument gate, 5,862 without**,
a factor of **1,172**. The instrument was §6n's best one, a *counterfactual of
the production being widened*: lift the gate in the parser alone and re-scan.
With the capture cache warm the 878-TU scan is 1.3 s, so each gate costs one
edit and one scan, and the ranking is measured rather than argued.

Three further counterfactuals, all free:

* the **identity check** on the forwarded arguments (argument `j` must be formal
  `j+1`) costs **0** — every explicit argument in this production on the whole
  workload is already an identity load, so the positive gate is free;
* the **FP-argument refusal** costs **0**;
* the **`5D 01` count gate** costs 0 by construction (a second sub-object is a
  second `26` and fails earlier).

## 4. The instrument bug this rung had to fix first

`gt_label_stride.py` looked its probe up as a function named `P`. A destructor
cannot be called `P`, so the four leaf rows above were unreachable and would have
reported `missing group` rather than a stride. The probe tuple now carries an
optional mangled-name stem. This is the same shape as §8.0's `disasm()` fault and
§8.5d's funclet-grouping fault: **the instrument's own convenience assumption
excluded exactly the row that mattered.**

## 5. Gate evidence

| lane | result |
|---|---|
| `cargo test --workspace` | **509 / 0** (baseline 504; +5 new, 0 changed verdicts) |
| `c2rs bench` | **193 / 0 / 0** (baseline 191; +2 fixtures) |
| `scripts/mode_lane.sh` `/Ox` `/O1` `/O2` `/Ox /Gy` | **91 / 89 / 89 / 89**, mismatch **0** |
| `scripts/mode_lane.sh /O1 /EHsc` and `/Ox /EHsc` — **new lanes** | **89** and **91**, mismatch **0** |
| the same two lanes with the surcharge disabled | **1 mismatch each**; `/O1` without `/EH` unaffected |
| `scripts/expr_sweep.sh` | **13,707** cases (42 fragments, +106 from this rung), mismatch **0** |
| `scripts/cross_sweep.sh` | 28 families (this rung adds `empty-ctor-base`, 3 representatives from 3 fragments — no hole), **27,956 configurations x 4 lanes, 0 mismatches**; 388 of the 406 unordered family pairs covered. Its own re-grade of the 13,707-case pool: 9,577 match, **0 mismatched** |
| 878-TU workload scan | match 6, **mismatch 0**, census 691,744 / 2,462,571 = **28.09 %**, disagreement **0** |
| 878-TU workload scan, **debug build** | **0 panics**, mismatch **0**, census **691,744**, disagreement **0** — identical to release |
| fixtures, `c2rs census` | positive **8/8**, negative **0/8**, at both the fixture profile and the workload's |

**A standing `/EHsc` lane did not exist and this rung needed one.** Every mode
lane is `/Ox`, `/O1`, `/O2` or `/Ox /Gy`, none of them with `/EH`, and at those
flags every `eh-bare` row collapses onto its non-EH control — a *vacuous* run,
not a zero. The `+1` therefore had no standing regression gate whatsoever, which
is why the defect survived W14/W15 and every merge since. `mode_lane.sh` already
takes extra flags, so `scripts/mode_lane.sh /O1 /EHsc` is the lane, and
`fixtures/cpp/wec_ctor_base.cpp` is built to grade it: a generated destructor and
a framed call in one TU, so the destructor's second slot moves the framed
function's `$M`/`$T` and the byte compare sees it.

## 6. Found and not taken

Ranked, with the axis applied, from the same scan.

1. **`body-0x9B` ∧ `eh-bare` — 16,747.** The largest single cheap-side row and
   untouched by this rung. `expr-call-in-expr-op-0x9B` is another 39,366 and
   `body-0x9B` 27,073 in total, so the opcode is worth ~66k across both sides.
   `wt-vbind-9a` owns the file it lives in.
2. **`expr-intrinsic-base-upcast` ∧ `eh-bare` — 8,277.** Intrinsic 2114, the
   null-guarded upcast: five instructions with a control-flow split, so it is a
   codegen rung and not a grammar one.
3. **`expr-intrinsic-this-adjust` ∧ `eh-bare` — 6,871, essentially untouched.**
   This rung took 7 of it. What is left is *not* the empty constructor; the
   6,871 are bodies whose 2113 intrinsic is reached some other way, and the
   refusal tally above says the largest single reason a candidate declines is
   that the `30`'s type is not a class at all (505 of 1,068). **A first-blocker
   histogram cannot rank this row further — it needs the production instrumented,
   which now costs one build and one 1.3 s scan.**
4. **The two `plumbing-0x3A` rows — 2,058 + 1,655 = 3,713**, both `eh-bare`,
   both `expr-call-in-expr-recv-{intrinsic-this-adjust,field-off0}-…`. They reach
   the return plumbing and stop there, which is the signature of a *private limit
   inside an existing recognizer* — §6n category (1), the cheapest kind.
5. **The forwarding form's neighbours**, sized by their own refusal counts above:
   the literal/computed base-constructor argument (237 in three TUs) is the
   largest, and it needs only `li r4,k` before the `bl` — the same operand-stream
   setup `int_tail_call_text` has lowered since the MVP.

**And the correction the ranking needs.** §7.3's cheap-side widening order lists
`expr-intrinsic-this-adjust` at 6,875 as its third row. That row's *production*
was worth 5,862 — but **5,855 of those were never on the cheap-side list at all**,
because they carry no EH marker. Any future rung sized off §7.3's table should
size the production, not the cell: the cell and the production differ by three
orders of magnitude here, in the direction the table cannot show.

## 7. The riskiest thing left unmeasured

**Whether `_fltused` has more producers of this kind.** The rule that fits is
"the body *passes* a floating-point value", and this rung refuses rather than
models it — so the boundary is drawn at a place that is measured (an FP argument
type in this one production) and not at the place the fact actually lives
(`touches_floating_point`, which enumerates shapes and now has five known
producers and one refused sixth). Every future rung that admits a call with
arguments inherits this, and the failure mode is silent on the workload scan: the
refusal costs 0 functions there, so no census number and no ranking would ever
have surfaced it. Only an adversarial probe did.

**And the cross-product lane's own frontier report says the same thing from the
other side.** Of the 406 unordered family pairs, **18 never emitted a TU in any
configuration, at any arity or mode** — and every one of them is an FP *leaf*
beside a *framed* family:

```
    call-sequence{,-cmp-eq,-cmp-order,-lit,-load,-load-fp,-value} + {float,double}-leaf
    framed-call + {float,double}-leaf
    empty-ctor-base + {float,double}-leaf          <- this rung's, and the shape is pre-existing
```

That is the label counter refusing, not the emitter mis-emitting: a float leaf's
stride is 2 (or 4, or 6 with pooled constants), `IlFunction::label_slots` returns
`None` for it, and the TU-level gate then refuses any TU that also contains a
framed function. `empty-ctor-base` joined a frontier that already had 16 members
rather than creating a new one, and it joined it in the **safe** direction.

But *refused* and *graded* are different things, and this is the second measured
fact this rung has about the same seam: an FP value beside this production is
untested at the TU level and was a live mismatch inside a single function. The
two are the same seam — `_fltused` and the first-FP-function `+1` label slot are
**one** fact by construction (`Function::is_float` decides both), so whoever
prices the float leaf's stride and clears that frontier is also the one who has
to decide whether "passes an FP value" is a producer of it. `docs/GAPS.md` §6 #13
already names the frontier a debt; this rung adds two rows to it and the reason.
