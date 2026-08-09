# GRID-X — two source spellings × two store forms, and ONE set of bytes

Four cells, each compiled with the real `c2.dll` under wibo at the workload's
own flags and cwd, six formals, `char *` buffer. The axes are the **error-arm
spelling** and the **store form**.

| cell | error arms | store | verdict at master | `.text` |
|---|---|---|---|---|
| `x_deref_split` | inline (`*e() = K; v(); return R;` twice) | `*buffer = 0` | **match** | 38 words |
| `x_index_split` | inline | `buffer[0] = 0` | vocab-gap | **the same 38 words** |
| `x_deref_merged` | sunk (`ep = e(); ev = K;` + one merged block) | `*buffer = 0` | vocab-gap | **the same 38 words** |
| `x_index_merged` | sunk | `buffer[0] = 0` | vocab-gap | **the same 38 words** |

**All four `.text` sections are byte-identical** (md5 of the decoded word list,
`ed606ae4728f`, on all four). So both axes are **reader work with no emitter
change at all** — there is no new byte anywhere, and therefore no new way to
emit a wrong one.

## And yet the OBJS are not identical: the label counter moves

The `split` and `merged` spellings charge c2's compiler-label counter
**differently**, on those same bytes:

```text
  x_deref_split    $M2572 @0x10   $M2573   $T2574
  x_deref_merged   $M2574 @0x10   $M2575   $T2576
```

Measured properly — in-TU and seed-free, two probe TUs of **two** bodies each so
the seed cancels:

```text
  lead_split2.obj   $M2601 … $M2607   stride 6   ->  lead 1   (the CONTROL: reproduces the shipped charge)
  lead_sunk2.obj    $M2606 … $M2611   stride 5   ->  lead 0
```

**This refutes `w-osfinfo`'s lead rule from zero byte distance.** That rule —
*"the lead is the number of unconditional intra-section `b` words in the body"* —
already had one refutation from `w-xlr`. Here the two spellings emit the
**identical thirty-eight words, with exactly one `b` in each**, and charge **1
and 0**. The lead is not a function of the emitted bytes at all: nothing a
codegen lane can look at distinguishes these two bodies, and only the IL's block
structure does.

## STRUCTURAL BLIND SPOT

Six formals, three guards, `char *`, one arity, one guard order, `/O1`. The grid
crosses two axes and holds everything else; it cannot see a rule in which the
lead depends on arity, on the guard count, or on the store width.
