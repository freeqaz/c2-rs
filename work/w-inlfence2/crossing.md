# w-inlfence §1 — the crossing, script-counted at base `0faa855a`

Instrument: `work/w-inlfence/scratch_cross.patch` (151 lines over
`gap/fnbytes.rs` and `gap/factors.rs`), applied to the base tree and reverted
after. **In no commit as a `crates/` change.** Scans:
`work/w-inlfence/cross2.fnd.out`, `cross3.fnd.out`, `cross4.fnd.out`.

## 0. The base, re-derived (not inherited)

`work/w-inlfence/base.fnd.out`, one un-instrumented 878-TU scan at `0faa855a`:

```
    gap-metric fnbyte-exact             36228
    gap-metric fnbyte-differs            2555
    gap-metric fnbyte-reloc-differs       861
    gap-metric fnbyte-refused          130116
    gap-metric fnbyte-denominator      178977
    gap-metric fnbyte-partition-broken      0
    gap-metric fnbyte-census-disagree       0
  match            18    2.1%
  mismatch          0    0.0%
  EMITTED CENSUS (§8): 39644/178977 emitted functions in class (22.15%)
  FUNCTION CENSUS (P2b): 712238/2463443 functions in class (28.91%)
```

## 1. FBM bucket × does the port's OWN composed body relocate against a name THIS TU DEFINES

Per function. `localcallee` is the fence's coarse form — asked **after** elide
(E) and splice (I), so a spliced body is scored on the target it actually
relocates against, not on the call the IL spelled.

| FBM bucket | `localcallee` | `nolocal` | total |
|---|--:|--:|--:|
| `fnbyte-exact` | **1,074** | 35,154 | 36,228 |
| `fnbyte-differs` | **2,530** | 25 | 2,555 |
| `fnbyte-reloc-differs` | **858** | 3 | 861 |
| `fnbyte-partial` | 0 | 0 | 0 |

> **99.0 % of every differing function emits a call to a symbol its own TU
> defines** — and so do **3.0 %** of the byte-exact ones. The coarse fence is
> therefore **not free**: it would cost **1,074** `fnbyte-exact`. **Decline
> clause D2 fires at its stated size of 25.**

Shapes of the `localcallee` arm, by verdict and REL24 count:

| verdict | shape\|calls | n |
|---|---|--:|
| differs | `seq\|2` | 1,143 |
| differs | `tail\|1` | 1,080 |
| differs | `seq\|1` | 182 |
| differs | `framed\|1` | 123 |
| **exact** | `tail\|1` | **1,055** |
| **exact** | `seq\|2` | **15** |
| **exact** | `seq\|1` | **3** |
| **exact** | `cond-pair\|2` | **1** |
| reloc-differs | `tail\|1` | 842 |
| reloc-differs | `seq\|2` | 16 |

**1,055 byte-exact tail calls to a callee this TU defines is the refutation of
"c2 inlines every same-TU callee."** c2 keeps those calls, and the port is
right to emit them.

## 2. WHY it keeps them — the size discriminator, per CALL SITE

`ref` is the callee's own COMDAT size in c2's obj (**ground truth, and NOT an
input a shipped emitter has**). `port` is the port's own lowered `/Gy` body for
the callee (**the shippable input**; `none` = the port cannot lower it).

| caller's verdict | `ref` size of the local callee | sites |
|---|---|--:|
| differs | ≤ 64 B | 2,994 |
| differs | 65–80 B | 505 |
| differs | 81–308 B | 7 |
| differs | > 308 B | 0 |
| **exact** | ≤ 64 B | **0** |
| **exact** | 65–80 B | **10** |
| **exact** | 81–308 B | **1,050** |
| **exact** | > 308 B | **21** |
| reloc-differs | ≤ 64 B | 858 |
| reloc-differs | > 308 B | 2 |

> **The separation is at ~80 bytes and it is near-total**: below it the caller
> is wrong 4,357 times and right 10; above it the caller is right 1,071 times
> and wrong 9.
>
> **Those four totals are computed by `work/w-inlfence/sizetab.py`, not typed.**
> The first version of this paragraph published *"wrong 3,852"* — it summed the
> `<= 64 B` differs bucket and lost the 505 sites in `65–80 B` — and the slip
> reached the rung, the board row and the ROADMAP before the script existed.
> Corrected in place, and the script is the reason it stays corrected. That is `WB_INLINE_FINDINGS` F1/F9 reproduced from the other
> side — c2 inlines the small callee and keeps the call to the large one — on a
> population 60× the 320 cells that lane compiled, and with no flag axis.

## 3. …and the shippable input is CONSERVATIVE in exactly the safe direction

Crossed with what the port can actually see:

| | `port` ≤ 64 B | `port = none` |
|---|--:|--:|
| differs | **1,177** | 2,329 |
| reloc-differs | **329** | 531 |
| **exact** | **0** | **1,081** |

> **Not one byte-exact function in the whole workload has a local callee the
> port can lower.** Every one of the 1,081 is `port=none` — the port cannot
> lower an 81–308 B callee, which is precisely the class c2 does not inline.
> The port's own inability to lower the callee is, on this workload, an almost
> perfect proxy for "c2 kept the call".

## 4. THE SHIPPED PREDICATE, priced per FUNCTION

> **Refuse a composed body that emits a REL24 against a name this TU defines,
> when the port can lower that callee and its lowered `/Gy` body is at most
> [`INLINE_UNBOUNDED_BYTES`] = 64 bytes.**

No new constant. `c2_core::splice::INLINE_UNBOUNDED_BYTES` is already shipped
and already graded — `w-splice`'s S7, `INLINE_PREDICATE.md` §2's `N_max`
unbounded at `index <= 64` in **both** linkage classes with every correction
term subtractive. `splice.rs` uses it to decide *the port may expand this*;
this fence uses the identical claim to decide *the port must not emit a call to
this*. One constant, two consequences, and the second one cannot be wrong in a
direction the first one is right in.

| what the fence moves | functions |
|---|--:|
| `fnbyte-differs` refused | **675** |
| `fnbyte-reloc-differs` refused | **329** |
| `fnbyte-exact` refused | **0** |
| `fnbyte-partial` refused | **0** |
| `fnbyte-reloc-unknown` refused | **0** |

## 5. The residue, named rather than left silent

**1,855** of the 2,555 differing functions have a local callee the port
**cannot lower** (`port=none`), and 2,322 of those call sites have a reference
size ≤ 80 B — i.e. c2 inlined them and the port is wrong, and the port has no
way to know. Closing that needs the callee's size **before codegen**, which is
what c2 itself uses (`WORD [sym+0x50]`, `WB_INLINE_FINDINGS` §2.1 and §5: *"the
index is a COUNT, and emitted bytes over-credit a loop"*). An IL-tuple-count
estimator is a genuinely available lane and it is **not** this one's: it is a
fitted model, it needs its own frozen grid, and this lane ships no fit.

**It would be SAFE to fit**, and that is worth saying once: on this fence a
mis-predicted *"c2 inlines"* makes the port **refuse** a function it would have
got right. It costs reach and it cannot cost a byte. That is the inverse of the
hazard `WB_INLINE_FINDINGS` §7 warns about (*"a mis-predicted accept is a wrong
obj"*), which was written for a lane that would **perform** the inline.
