# W32 — the `volatile` formal, the thirteenth live wrong-bytes emit

    Tag:       W32
    Slug:      volatile-formal
    Date:      2026-07-31
    Fixtures:  w32_volatile_formal_neg.cpp w32_volatile_free.cpp
    Census:    PENDING (a refusal rung — the delta is negative by construction)
    Record:    c2_il::func::readers::is_volatile_tag

A `volatile`-qualified parameter that the body **reads** was in class and emitted
the register move. It is a memory object: c2 homes the incoming argument register
in the frame and reads it back at every use. This was live on mainline across
**seven** shapes at once and is fixed by one predicate at one position.

## What it was

```text
  int   v3(int x, volatile int y)      { return y; }
      stw r4,124(r1) ; lwz r3,124(r1) ; blr        port: mr r3,r4 ; blr
  float v11(float x, volatile float y) { return gf(y); }
      mflr r12 · stw r12,-8(r1) · stwu r1,-96(r1)
      d041007c  stfs f2,124(r1)      <- homed
      c021007c  lfs  f1,124(r1)      <- read back
      4bffffed  bl ?gf               <- and therefore NOT a tail call
      addi r1,r1,96 · lwz r12,-8(r1) · mtlr r12 · blr
                                        port: fmr f1,f2 ; b ?gf
```

`Port=Mismatch @ offset 2` on the FP case — the section count, because the
reference obj has a `.pdata` the port never emitted. The affected shapes, each
witnessed:

| shape | witness |
|---|---|
| straight-line leaf | `int f(int x, volatile int y){ return y; }` |
| straight-line arithmetic | `int f(volatile int y){ return y + 1; }`, `{ return x + y; }` |
| integer tail call | `int f(int x, volatile int y){ return gi(y); }` |
| framed call | `int f(int x, volatile int y){ return gi(y) + 1; }` |
| discarded statement call | `void f(int x, volatile int y){ gv(y); }` |
| multi-argument permutation | `int f(volatile int x, int y){ return gi2(y, x); }` |
| pointer getter / identity | `int f(int x, int* volatile p){ return *p; }` / `{ return p; }` |
| **FP tail call (W31)** | `float f(float x, volatile float y){ return gf(y); }` |

Only the last is this session's; the other seven predate it. Found by W31's
generated neighbour grid — the cv-qualification axis, which is there because
`is_fp_type` reads nibbles rather than whitelisting triples and the grid asked
what the other tag bits do.

## What it admits, and what it refuses — the position is the whole content

The `volatile` bit is `0x10` on the TYPE tag (`86` plain, `A6` const, `96`
volatile, `B6` both). It appears at four positions and costs something at
exactly one:

```text
  int f(volatile int y)         { return y; }    b9 <y> 96 41 …   REFUSE  (spills)
  int f(int* volatile p)        { return *p; }   b9 <p> 96 43 …   REFUSE  (spills)
  int f(volatile S* p)          { return p->i; } b9 <p> 86 43 …   free — the POINTER
                                                 27     86 43 …   is not volatile
                                                 30     96 41 …
  struct S { volatile int i; };
  int f(S* p)                   { return p->i; } 30     96 41 …   free — one `lwz`
                                                                   either way
  int f(int x, volatile int y)  { return x; }    (no LOAD)        free — a bare `blr`
```

So the gate is on the **`B9` operand LOAD** and nowhere else: `eat_operand_type`
(which covers `parse_expr`, hence the straight-line leaf, every call argument,
the framed body, the sequence and the permutation), the three explicit `B9`
pointer-value reads in `leaf_addr` / `leaf_store` / `leaf_load`, the getter's
base pointer LOAD, and `is_fp_type`. It is deliberately **not** on
`eat_int_like`, `eat_value_type`, or the `27`/`30` designator readers, where the
same two bytes are free — a blanket gate on the tag would have cost every body in
`w32_volatile_free.cpp` for nothing.

`const` is free everywhere and is untouched. That pair is what makes this a
measurement rather than a guess: `const float y` and `volatile float y` differ in
one bit of one byte and in a whole stack frame.

**It could not be a `.sy`-side gate.** `.sy` does not carry the qualifier at all
in the fields this port reads — `const` and `volatile` leave the type tag and
kind alone and move the *tid* into the TU's constructed-type range, so telling
them apart there needs a type-table walk. Measured on a seven-formal probe: a
plain `int` formal, a `volatile int` one and a `const int` one are byte-identical
through `86 01 00 03 04 04` and differ only in the trailing id (`80 01 10 00 00`
vs `80 04 10 00 00`).

**"Read", not "declared".** `int f(int x, volatile int y){ return x; }` is a bare
`blr` and stays in class: the parameter is a volatile object, but with no access
there is no access to emit. That is the second reason the gate is on the LOAD.

## Estimate vs outcome

**Estimate, recorded before the scan: −150 functions**, biased HIGH in magnitude
(i.e. I expect it to cost fewer than 150). `volatile` on a *by-value scalar
parameter* is rare in application C++ — the idiom is a `volatile` member or a
`volatile*`, both of which stay in class. Outcome: PENDING.

## Gate evidence

PENDING

## Found and not taken

| item | size | what stops it |
|---|---:|---|
| the `volatile` formal itself, emitted rather than refused | the refusal's own cost | the home slot's displacement (124(r1) in the witness) is the frame layout's, and admitting it means a framed body for what looks like a leaf — the frame spine's, not a leaf rung |
| `volatile` at the `2C` conversion target | 0 observed | the LOAD refuses first; the strip (`96 41 … 2c 86 41 74 00`) is how the read is spelled, so gating the target as well would be a second lock on one fact |
| the `0x40` tag bit | unmeasured, refused everywhere already | `readers.rs` records that it occurs and no probe has produced one; the same "a field that never varied is indistinguishable from a constant" rule that produced this finding |
