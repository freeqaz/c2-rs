# The EH-`main` label triple — MEASURED, six cells, four seeds

Lane `w-main2`. Reproduce with `sh work/w-main2/cells.sh` (compiles every
`probe/*.cpp` at the workload's own `flags.txt` with real `c2.dll` under wibo,
captures each cell's IL, reads the `.gl` seed and prints the label table).

`docs/LABEL_COUNTER.md` has **no EH row**. This is the measurement for one, and
it is taken in **`LABEL_COUNTER.md` §7.6's in-the-middle form** — the seed is
read out of each cell's own `.gl` (`gl[7..11]` LE32) and every number below is
an *offset from that seed*, never a counterfactual difference between two
compilations. `wb-label` #2430–#2440: c1xx and c2 share one symbol-id space, so
a counterfactual measures Δseed + Δcharge.

## 1. The cells

| cell | source | funcs | EH fn position |
|---|---|---|---|
| `m0` | `App app(argc,argv); app.Run();` in `main` | 1 | first |
| `m1` | `m0` + three `extern int` declarations (moves the seed only) | 1 | first |
| `m2` | `m0` + a leaf `lf` **before** `main` | 2 | second |
| `m3` | `m0` + two leaves **before** `main` | 3 | third |
| `m4` | `m0` + a leaf `lf` **after** `main` | 2 | first |
| `m5` | two EH functions, `f2` then `main` | 2 | both |
| — | `src/Main.cpp` itself | 1 | first |

## 2. The one-function class — EXACT, over three seeds

`m0` (seed 2551), `m1` (2554) and `src/Main.cpp` (2575) give **identical
offsets**. With `S` = the `.gl` counter:

| symbol | number | section / value |
|---|---|---|
| `__unwind$` | **S+10** | `.text` `0x54` — the funclet's entry, STATIC, type `0x0020` |
| `$M` | **S+15** | `.text` `0x30` — ip-to-state entry 0 (state 0) |
| `$M` | **S+16** | `.text` `0x38` — ip-to-state entry 1 (state −1) |
| `$T` | **S+17** | `.rdata` `0x30` — the ip-to-state array itself |
| `$M` | **S+19** | `.text` `0x1c` — `main`'s prologue end |
| `$M` | **S+20** | `.text` `0x54` — `main`'s end |
| `$T` | **S+21** | `.pdata` (`main`'s record) `+0` |
| `$M` | **S+22** | `.text` `0x64` — the funclet's prologue end |
| `$M` | **S+23** | `.text` `0x7c` — the funclet's end |
| `$T` | **S+24** | `.pdata` (the funclet's record) `+0` |

`S+11..S+14` and `S+18` are **consumed and never emitted**, exactly as
`plan_labels` already models slack for a framed function.

## 3. It reconciles to `coff::plan_labels` with ZERO residual — at `label_lead = 7`

`plan_labels` computes `cur = S + LABEL_SEED_GAP(9) + 3·nfuncs`, then per
function `cur += label_lead` and, for a framed one, mints `[cur, cur+1, cur+2]`
and advances 5. Write `B = S + 9 + 3·nfuncs + Σ(preceding consumption)` — the
value of `cur` when this function's turn begins. Then over **all six cells**:

```
    ip-to-state $M pair   B+3, B+4
    ip-to-state $T        B+5
    the function triple   B+7, B+8, B+9      <=>  label_lead = 7
    the funclet triple    B+10, B+11, B+12
```

and this **re-derives `B` on every cell independently**: a leaf consumes 1, an
EH function consumes 17, the `/Gy` pre-pass is 3 per function — the incumbent
rules, unmodified. `m4` and `m5`'s first function pin `nfuncs` against position:
both have `nfuncs = 2` with the EH function first and both read `B = S+15`,
where `m0`'s one-function TU reads `B = S+12`.

**So the EH row of `LABEL_COUNTER.md` is one number: `label_lead = 7`, plus a
funclet triple that consumes a second framed function's worth of slots.** The
lead `w-main` §5 published — `+31` at `/O1` against a `leaf-none` control — is
the *counter advance* of the whole shape and is not this number; both are true
and they are different quantities, which is #2265's own point one level down.

## 4. `__unwind$` IS NOT AT A FIXED OFFSET FROM `B`, AND THAT IS WHY THE CLASS IS GATED AT ONE FUNCTION

The nine `$M`/`$T` labels sit at fixed offsets from `B` on every cell. The
funclet symbol does not:

| cell | funcs | preceding | `B` | `__unwind$ − B` |
|---|---:|---|---|---:|
| `m0`, `m1`, `Main.cpp` | 1 | — | S+12 | **−2** |
| `m4` | 2 | none (leaf follows) | S+15 | **−2** |
| `m5` `f2` | 2 | none (EH follows) | S+15 | **−2** |
| `m2` | 2 | one leaf | S+16 | **0** |
| `m3` | 3 | two leaves | S+20 | **0** |
| `m5` `main` | 2 | one EH function | S+32 | **0** |

`−2` whenever the EH function is the TU's **first**, `0` whenever anything
precedes it. Two readings fit the six cells and **nothing here separates them**
— the funclet label may be minted before the first function's own block, or the
`/Gy` pre-pass may not be a single lump — so this lane does **not** model it.

**The recognizer is gated at exactly one `.ex` segment**, which is the same gate
`IlBundle::dyninit_tu` takes and which makes the `−2` the only case that can
ever fire. A TU with two functions refuses. This is a decline recorded as a
decline, not a rule fitted to whichever branch the target happened to be on.
