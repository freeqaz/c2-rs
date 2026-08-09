# W-IFN — the compiler-label lead of the three `mmio.cpp` shapes, MEASURED

`docs/LABEL_COUNTER.md`'s published surcharges have now been measured wrong by
**four consecutive lanes** (`w-json` by two, `w-bdnz` by six, `w-main` by
twenty-five, `w-blockir` by nine), and `w-blockir` additionally found the charge
**sub-shape** dependent. Nothing below is quoted from that table. Every number
comes from `work/w-ifn/probe/lead.sh`, which compiles six one-cell TUs with real
`cl.exe` 16.00.11886.00 under wibo and reads the `$M`/`$T` symbols straight out
of the obj, at the workload's `/O1 /Oi /EHsc /GR` and again at `/Ox /GS-`.

## The form, and the correction this lane has to make to it

`w-json`'s counterfactual: each TU is `[<subject…>, framed]`, the subject varies
and the framed control does not, and the lead is read off the control's `$M`.

    lead_ctl    int leaf_none(int a){ return a+1; }           the control
    lead_fr     int subject(int a){ return gz(a)+3; }         an ORDINARY framed call
    lead_g      mmioGetInfo's shape (2 guards + memcpy)
    lead_s      mmioSetInfo's shape (2 guards + memcpy + a conditional member store)
    lead_c      mmioClose's shape (1 guard + 3 calls + 2 result tests)
    lead_cctl   lead_c's two same-TU callees WITHOUT the subject

> **⚠ THE SEED IS NOT CONSTANT ACROSS CELLS, AND SUBTRACTING TWO CONTROL `$M`s
> MEASURES THE SEED AS WELL AS THE CLASS.** `lead_ctl`'s control sits at
> `$M2555` and `lead_g`'s at `$M2568` — a difference of **+13** — but
> `lead_g`'s *subject* starts at `$M2563`, so nine of those thirteen are the
> **seed** moving (this TU declares `memcpy` and two `void*` formals; the seed
> is a function of the `.gl`, which `coff::plan_labels` takes as its `counter`
> argument and which is not a property of any body). The number that belongs to
> the class is the **in-TU stride** — the next framed function's base minus this
> one's — which is seed-free by construction. It is what `plan_labels` actually
> consumes, and it is what is tabulated below.

## The measurement

| cell | `/O1` subject base | `/O1` control base | **stride** | `/Ox` subject base | `/Ox` control base | **stride** |
|---|---:|---:|---:|---:|---:|---:|
| `lead_fr` — an ordinary framed call | 2554 | 2559 | **5** | 2548 | 2552 | **4** |
| `lead_g` — mmioGetInfo | 2563 | 2568 | **5** | 2557 | 2561 | **4** |
| `lead_s` — mmioSetInfo | 2576 | 2581 | **5** | 2570 | 2574 | **4** |
| `lead_c` — mmioClose | 2597 | 2602 | **5** | 2585 | 2589 | **4** |

Raw output, both modes, verbatim (`work/w-ifn/probe/lead.out`):

```
lead_ctl  o1: $M2556 $M2555 $T2557
lead_ctl  ox: $M2550 $M2549 $T2551
lead_fr   o1: $M2555 $M2554 $T2556  $M2560 $M2559 $T2561
lead_fr   ox: $M2549 $M2548 $T2550  $M2553 $M2552 $T2554
lead_g    o1: $M2564 $M2563 $T2565  $M2569 $M2568 $T2570
lead_g    ox: $M2558 $M2557 $T2559  $M2562 $M2561 $T2563
lead_s    o1: $M2577 $M2576 $T2578  $M2582 $M2581 $T2583
lead_s    ox: $M2571 $M2570 $T2572  $M2575 $M2574 $T2576
lead_c    o1: $M2598 $M2597 $T2599  $M2603 $M2602 $T2604
lead_c    ox: $M2586 $M2585 $T2587  $M2590 $M2589 $T2591
lead_cctl o1: $M2575 $M2574 $T2576
lead_cctl ox: $M2563 $M2562 $T2564
```

## The finding, and it is the OPPOSITE of the last four lanes'

**All three shapes charge exactly what an ordinary framed call charges: 5 under
`/Gy` and 4 packed, i.e. `label_lead() == 0`.** `coff::plan_labels`
(`crates/c2-core/src/coff/label.rs:85`) already advances `if comdat { 5 } else
{ 4 }` for any function with a `frame`, and `IlFunction::label_slots` already
returns `label_lead() + 5 / + 4` for `is_framed()`. **So this class needs no
`label_lead` arm and no `label_slots` arm at all.**

The `/O1`-vs-`/Ox` difference in the table is **not** a mode dependence of the
class: `/O1 /Oi /EHsc /GR` implies `/Gy` (each function gets its own `.text`
COMDAT) and the bare `/Ox /GS-` cell does not, so the 5 and the 4 are the two
arms of `plan_labels`' `comdat` flag on one rule. `lead_fr`, an ordinary framed
call the port has been emitting byte-exactly since the MVP, reads exactly the
same 5/4 — which is the anchor control that says so.

**Why this class escapes the trap the last four lanes fell into**, stated so it
is not read as luck: every one of those lanes was widening a **leaf** class —
a loop — where `plan_labels` mints *nothing* and the counter nonetheless
advances by a body-shape-dependent amount that has no representation in the obj.
This class is **framed**, so the triple is minted, the stride is observable in
the obj, and the only question is whether it is the framed constant. It is.

## The second, independent derivation — the target TU's own obj

`work/w-ifn/ref/mmio.obj`, the real workload obj, with eleven functions of which
three are framed:

```
  mmioGetInfo   $M3381 $M3382 $T3383
  mmioSetInfo   $M3386 $M3387 $T3388
  mmioClose     $M3396 $M3397 $T3398
```

`mmioGetInfo → mmioSetInfo` is **+5** with no function between them.
`mmioSetInfo → mmioClose` is **+10** with **five** leaves between them
(`mmioStringToFOURCCW`, `mmioFlush`, `mmioSeek`, `mmioSetBuffer`, `mmioOpenW`),
each charging 1 — so the subject charges **5** again. Two derivations, one from
one-cell counterfactuals and one from the target's own obj, agreeing.

## The must-fail mutation

Because `label_lead()` is 0 and `label_slots` needs no arm, the mutation that
has to fail is the *opposite* of the last four lanes': it is adding a charge
this class does not have. Recorded in `work/w-ifn/NEG_CLAUSES.md` §4 with the
tree it was run on.
