# w-bdnz — the LABEL LEAD, measured against the obj

> ### ⚠ REVISED 2026-08-15, lane `w-labeltable` — **THE `$M` COLUMN REPRODUCES; THE `lead` COLUMN IS A COUNTER-GAP ARTIFACT**
>
> **Nothing below is rewritten.** The objs are right and the arithmetic over them
> is not, which is `#3148` in a second file: this instrument differences the
> framed `$M` **across two TUs**, and a TU's `.gl` label counter depends on its
> **own source text**. Re-differenced with the seed-cancelling form
> (`work/w-labeltable/table.py --bdnz`, output `work/w-labeltable/bdnz_o1work.txt`),
> at this file's own `/O1 /Oi /EHsc /GR`:
>
> ```text
>   cell           counter    real $M   published   SEED-FREE   gap vs lab_ctl
>   lab_ctl           2540       2556          —          0      +0
>   lab_forever       2542       2558        +2          0      +2
>   lab_loop          2545       2563        +7          2      +5
>   lab_while         2545       2563        +7          2      +5
>   lab_dowhile       2545       2562        +6          1      +5
>   lab_goto          2546       2564        +8          2      +6
>   lab_op            2545       2563        +7          2      +5
>   lab_uns           2545       2563        +7          2      +5
> ```
>
> **All eight `$M` values reproduce exactly, and every published lead is the
> seed-free lead plus that cell's own counter gap.**
>
> The four readings, scored:
>
> 1. ***"§4.2.1's `for` row is not the number for this class"*** — **REFUTED, and
>    in both terms.** A §4.2.1 surcharge **is** a lead (`plan_labels` charges a
>    leaf `label_lead + 1`, so `stride 3` is `lead 2`), not `lead 1`; and the obj
>    says **2**, not 7. **§4.2.1's `for` row and this class's obj AGREE at 2.**
>    §4.2.1 was independently re-audited row by row on the same day and holds
>    17 of 17.
> 2. ***"THE CHARGE IS MODE-DEPENDENT"*** — **SURVIVES**, and it is the reason
>    this arm's `None` still stands. The same body reads **2** at `/O1` and **3**
>    at `/Ox` (`bdnz_oxwork.txt`). The **step** this lane measured, `+1`, is
>    right; only the base was inflated.
> 3. ***"the `for`/`while` confound does not arise"*** — **SURVIVES.**
>    `lab_loop` and `lab_while` both read **2**, `lab_dowhile` reads **1**.
> 4. ***"the charge is constant on both free axes"*** — **SURVIVES.** `lab_op`
>    and `lab_uns` both read **2**.
>
> **And `lab_forever` — the separating control this file nets its locals out
> with — reads 0, not +2.** Two `int` locals cost the counter nothing, so the
> *"net of locals it is +5 against +1"* sentence is wrong in both terms.
>
> **This table is quoted verbatim into `IlFunction::label_slots`' shipped doc
> comment**, which is the mechanism `#3148` names: a number published in a
> `work/` table, quoted into code, never graded by anything, pointing in the
> direction that keeps a refusal in place. `w-labeltable` is a docs lane and did
> not edit `crates/`; the correction is filed for the coordinator.

**The commission's instruction was to measure it and not to trust
`docs/LABEL_COUNTER.md`'s table** — w-json measured its §1.1 surcharge two low
for a back-edge class. For this class the table read literally is **six low at
`/O1` and seven low at `/Ox`**, and the measurement produced a reason to return
`None` that is this lane's own rather than inherited.

## The instrument

w-json's counterfactual form. Two TUs differ in **exactly one function body**:
the control puts a `leaf-none` in the first slot, the test puts the cell there.
The **second** function is the same framed `int z9(int a){return gz(a)+7;}` in
every TU, and its `$M`/`$M`/`$T` triple — the only channel a label charge has to
the obj — is the readout. The difference between two runs is the first
function's charge. `work/w-bdnz/label.sh`, real `c2.dll` under wibo, at the
workload's own `/O1 /Oi /EHsc /GR` and again at `/Ox`.

## The table

`$M` is the first of the framed function's pair; lead is against `lab_ctl`.

| cell | body of the first function | `/O1` `$M` | lead | `/Ox` `$M` | lead |
|---|---|---:|---:|---:|---:|
| `lab_ctl` | `return n + k;` — `leaf-none`, 0 locals | 2556 | — | 2550 | — |
| `lab_forever` | `int s=0; int i=k; s-=i; return s;` — straight line, **2 locals** | 2558 | **+2** | 2552 | **+2** |
| **`lab_loop`** | **this lane's class** (`for`, `s -= k`) | 2563 | **+7** | 2558 | **+8** |
| `lab_while` | the `while` spelling — **byte-identical text** | 2563 | +7 | 2558 | +8 |
| `lab_dowhile` | the `do/while` spelling — **different text, no `bdnz` at all** | 2562 | +6 | 2556 | +6 |
| `lab_goto` | `?HashString`'s pointer-walk shape (`w-loop`'s "+3" row) | 2564 | +8 | 2559 | +9 |
| `lab_op` | the class with `*=` instead of `-=` | 2563 | +7 | 2558 | +8 |
| `lab_uns` | the class with an **unsigned** counter and bound | 2563 | +7 | 2558 | +8 |

`lab_forever` is the separating control: two `int` locals cost **+2** with no
loop, so this class's own charge net of its locals is **+5 at `/O1`** and
**+6 at `/Ox`**.

## Four readings

1. **`LABEL_COUNTER.md` §4.2.1's `for` row is not the number for this class.**
   That row records a leaf `for` loop at `+2` against `leaf-none = 1` — a lead of
   `+1`. The obj says **+7**. Read net of locals it is `+5` against `+1`. The
   commission's warning holds and by a wider margin than w-json's two.

2. **THE CHARGE IS MODE-DEPENDENT, and that is decisive.** `+7` at `/O1` and
   `+8` at `/Ox`, on the *same source* — and `IlFunction::label_slots(&self,
   fn_level_linking: bool)` has **no mode parameter**. Any `Some(k)` would be
   right at one mode and one wrong `$M` triple at the other: six wrong bytes in
   an obj that still links, board #263's shape. This lane's class accepts both
   modes (§1.3 of the PREREG), so it would meet the wrong one immediately.
   **`None` is not conservatism here; it is the only value that can be right.**

3. **The `for`/`while` confound `w-loop` cites does NOT arise for this class,
   and the `do/while` one does not either.** `lab_loop` and `lab_while` emit
   byte-identical text *and* charge identically at both modes — so the two
   spellings that a port cannot tell apart are not the two that charge
   differently. `lab_dowhile` charges differently and is **not in the class at
   all**: c2 does not convert it (no `mtctr`, no `bdnz`), which is `wb-loop`'s
   P3.4 miss (`cal_dowhile`) reproduced here on its own cell. So the inherited
   argument for `None` is *absent* for this class and reading 2 above is what
   replaces it.

4. **Within the class the charge is constant on both free axes.** `lab_op` (a
   different accumulate opcode) and `lab_uns` (a different guard compare and
   branch bit) read the same `2563`/`2558` as `lab_loop`. So a `Some(k)` is a
   4-cell fit — #1767's two-point bar is met on the *class* axes and fails
   outright on the *mode* axis, which is reading 2.

## What a later rung would need to take `Some(k)`

A `label_slots` that takes the optimization word, a `plan_labels` that can be
asked at two modes, and a mixed-frame-class fixture per mode
(`whash_loop_then_framed.cpp`'s shape, twice). None of that is this lane's, and
this lane's `None` is what makes the TU refusal fail closed until it exists.
