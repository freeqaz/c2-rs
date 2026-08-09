# W-BLOCKIR — the compiler-label lead of the float array-walk loop, MEASURED

`docs/LABEL_COUNTER.md`'s published surcharges have now been measured wrong by
**three separate lanes** and are **mode-dependent**. Nothing below is quoted from
that table. Every number here comes from `work/w-blockir/probe/lead.sh`, which
compiles four one-cell TUs with real `cl.exe` 16.00.11886.00 under wibo and reads
the `$M`/`$T` symbols straight out of the obj.

## The form

w-json's counterfactual: each TU is `[<subject>, framed]`, the subject varies and
the framed function does not. The lead is the difference between the framed
function's `$M` number in the cell and in the `leaf-none` control — a leaf
`int f(int a){ return a+1; }`, which `coff::plan_labels` charges **1**.

    lead_ctl   int leaf_none(int a) { return a + 1; }          the control
    lead_a     shape A   b[i] += a[i]
    lead_b     shape B   a[i] *= s
    lead_c     shape C   c[i] = a[i] * b[i]

## The measurement

| cell | `/O1` framed `$M` | lead | `/Ox` framed `$M` | lead |
|---|---:|---:|---:|---:|
| `lead_ctl` (`leaf-none`) | 2555 | — | 2549 | — |
| `lead_a` (shape A) | 2565 | **+10** | 2562 | **+13** |
| `lead_b` (shape B) | 2565 | **+10** | 2562 | **+13** |
| `lead_c` (shape C) | 2566 | **+11** | 2564 | **+15** |

Raw output, both modes, verbatim:

```
lead_ctl o1: $M2556 $M2555 $T2557
lead_ctl ox: $M2550 $M2549 $T2551
lead_a   o1: $M2566 $M2565 $T2567
lead_a   ox: $M2563 $M2562 $T2564
lead_b   o1: $M2566 $M2565 $T2567
lead_b   ox: $M2563 $M2562 $T2564
lead_c   o1: $M2567 $M2566 $T2568
lead_c   ox: $M2565 $M2564 $T2566  $M2569 $M2568 $T2570
```

## Three findings, and each is a reason `label_slots` returns `None`

1. **The charge is not +1.** `docs/LABEL_COUNTER.md` §4.2.1's `for` row read
   literally predicts **+1** over a leaf. It is **+10** at `/O1`. That is the
   fourth lane in a row to measure that table wrong, and this one is a factor of
   ten rather than a rounding.
2. **The charge is MODE-DEPENDENT** — +10 at `/O1` against +13 at `/Ox` for the
   same body. `IlFunction::label_slots` has **no mode parameter**, so `None` is
   the only value that can be right. That is `w-bdnz` board #1983's reason,
   re-derived here on this class rather than inherited from it.
3. **And it is SUB-SHAPE dependent as well**, which #1983 did not have to
   contend with: shape C charges one more than A and B at `/O1` and two more at
   `/Ox`. A correct `Some(k)` would need to be a function of *both* the shape and
   the mode, and `plan_labels` would additionally have to learn the same `k` —
   the second layer #1983 names. Three things a later rung owes, none of them
   this one's.

The `/Ox` row for shape C prints **two** label triples because at `/Ox` the
unrolled loop takes a frame of its own. That is a fact about `/Ox`, which this
class refuses outright, and it is recorded rather than folded into the lead.

## The must-fail mutation, and the cell that could not fail

`fixtures/cpp/wblockir_float_walk_then_framed_neg.cpp` is the cell. Replacing the
`None` with `Some(self.label_lead() + 1)`:

| tree | verdict under the mutation |
|---|---|
| the fixture as **first written** (framed first, loop second) | **`match`** — the cell could not fail |
| the fixture as **shipped** (loop first, framed second) | **`Port=Mismatch`, bytes diverge** |
| `probe/lab2.cpp` (loop, framed) | **mismatch** |
| `probe/lab.cpp` (framed, loop, loop, framed, loop, framed) | **mismatch** |
| `fixtures/cpp/wblockir_float_walk.cpp` (the separating control) | `match`, unmoved |

**The order is the whole cell**, and the first spelling of the fixture had it
backwards: a wrong charge on the *last* function in a TU moves nothing after it,
so a `_neg` cell that puts the subject last is a cell that cannot fail. It was
caught by running the mutation rather than by reasoning about it, which is the
only way it could have been caught.

With the `None` restored, all five trees read `vocab-gap` / `match` as they
should.
