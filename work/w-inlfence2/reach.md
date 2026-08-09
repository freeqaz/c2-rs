# w-inlfence2 §2 — what the fence reaches, and the 444 it does NOT

All figures script-counted. Three 878-TU scans, each with
`--fnbyte-diff-jsonl` (one row per `fnbyte-differs` FUNCTION, keyed
`(tu, sym)`), plus a per-function witness on the fenced tip:

| stem | tree | `fnbyte-exact` | `fnbyte-differs` | `fnbyte-reloc-differs` | emitted census |
|---|---|--:|--:|--:|--:|
| `pre` | `05d743f7` — w-fltret's parent | **36,228** | **2,111** | 861 | **39,200** |
| `base` | `0faa855a` — master, w-fltret in | **36,228** | **2,555** | 861 | **39,644** |
| `tip2` | this lane | **36,228** | **1,880** | **532** | 39,644 |

`work/w-inlfence2/reach.py`, on those three:

```
differs  pre(05d743f7) 2111   base(0faa855a) 2555   tip 1880

R2 = base \ pre  (w-fltret's increment)      : 444
     pre \ base (differing BEFORE, not after) : 0
REMOVED by the fence = base \ tip             : 675
ADDED by the fence   = tip \ base (must be 0) : 0

of R2 (444), the fence removes                 : 0  (0.0%)
of the BASE 2,111 (2111), the fence removes    : 675  (32.0%)
    check: 0 + 675 = 675 == 675  True
```

> ## **THE FENCE REACHES 32 % OF THE BASE 2,111 AND **ZERO** OF w-fltret's 444 — WHICH IS THE EXACT INVERSE OF WHAT THE COMMISSION AND THIS LANE'S PREREG BOTH EXPECTED.**

## 1. Why — and it is a SET, not an argument from two totals

`work/w-inlfence2/r2arm.py` crosses the 444 with the fence arm measured at the
**fenced** tip (`work/w-inlfence2/witness.fnd.err`, one line per emitted
function):

```
-- R2: (bucket at the FENCED tip, fence arm) --
     444  fnbyte-differs         localcallee

-- BASE-2111: (bucket at the FENCED tip, fence arm) --
    1411  fnbyte-differs         localcallee
     675  <no-witness>              (refused by the fence)
      25  fnbyte-differs         nolocal
```

**All 444 of them DO emit a call to a symbol their own TU defines.** The fence
sees every one. It declines to fire because it cannot **prove** the callee is
one c2 expands: `Timer::Split` and `Timer::Ms` are `expr-op-0x27`, the IL parser
refuses them, so `TuContext::definition` hands back `None` and the port has **no
size to test**. 434 of the 444 are the single symbol `?SplitMs@Timer@@QAAMXZ`.

So the missing input is **not definedness** — that is visible, cheaply, and the
fence uses it. It is the callee's **SIZE**, which the port can only obtain by
lowering the callee, and the callee is exactly the thing it cannot lower.

## 2. The coarse alternative, priced at the FUNCTION level

*"Refuse whenever the composed body relocates against a name this TU defines"*,
measured at the fenced tip (`xf-*` keys, `work/w-inlfence2/witness.fnd.out`):

| | it would additionally REMOVE | it would COST |
|---|--:|--:|
| `fnbyte-differs` | **1,855** (including **all 444**) | — |
| `fnbyte-reloc-differs` | **529** | — |
| **`fnbyte-exact`** | — | **1,074** |

**Decline clause D2 forbids it at a stated size of 25, and it exceeds that by
43×.** 1,055 of the 1,074 are byte-exact TAIL CALLS to a same-TU callee, and §2
of `crossing.md` says why they are right: the callee is 81–308 emitted bytes,
which is the class `WB_INLINE_FINDINGS` F1 measures c2 **keeping the call** to.

## 3. What would close the residue, named and not attempted

The port needs the callee's size **without lowering it**. That is what c2 itself
uses — `WORD [sym+0x50]`, a pre-codegen instruction COUNT, `WB_INLINE_FINDINGS`
§2.1, with the diagnostic string `"INL:\tInlining %s (%d instrs) into "` naming
the unit, and §5's finding that *"the index is a COUNT, and emitted bytes
over-credit a loop"*. The IL is a pre-codegen representation and the callee's
own `.ex` segment length is readable for **every** callee, including the ones
the parser refuses.

**That is a fitted model and this lane ships no fit** (PREREG §4). It needs its
own frozen grid, and its rungs would be graded exactly the way GRID-I was.

**But it is SAFE to fit, and that is the part worth carrying forward.**
`WB_INLINE_FINDINGS` §7 offers only *decline* rules because *"a mis-predicted
accept is a wrong obj"* — a warning written for a lane that would **perform**
the inline. On this fence the accept prediction drives a **refusal**: a
mis-predicted *"c2 inlines this"* makes the port decline a function it would
have got right. It costs reach and it cannot cost a byte. **The accept side of
the inline decision is safe to consult in exactly one place, and this is it.**
