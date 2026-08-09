
### 10.29 w-inlfence2 — the port REFUSES a body that emits a call c2 replaces with the callee's body: 1,004 wrong bodies removed for ZERO byte-exact functions lost, and the 444 are untouched for a reason nobody predicted (2026-08-09)

Lane `w-inlfence2`, branch `wt-w-inlfence2`, off master `0faa855a`
([`rungs/2026-08-09-w-inlfence2.md`](rungs/2026-08-09-w-inlfence2.md), board
**#2150**–**#2162**). Commissioned off §10.28/§10.28.1's finding — the port
emits the calls the IL contains where c2 has inlined both callees — with the
instruction to *make the port REFUSE what c2 inlines, rather than emit a call c2
does not*.

**What ships.** `c2_core::comdat::fenced_inlined_callee`, at the composition
seam so that `PortC2::build` and the FBM instrument both run it and neither runs
a copy:

> Refuse a composed body that emits a `REL24` against a name **this TU defines**,
> when the port **can lower** that callee and its lowered `/Gy` body is at most
> `splice::INLINE_UNBOUNDED_BYTES` (64) bytes.

**No new constant.** That bound is `w-splice`'s S7 —
[`INLINE_PREDICATE.md`](INLINE_PREDICATE.md) §2's `N_max` unbounded at
`index <= 64` in *both* linkage classes with every correction term subtractive.
`splice.rs` reads it as *"the port MAY expand this body"*; the fence reads the
identical claim as *"the port MUST NOT emit a call to it"*. Board **#2154**.

| 878-TU scan | base `0faa855a` | tip | Δ |
|---|--:|--:|--:|
| `fnbyte-exact` | 36,228 | **36,228** | **0** |
| `fnbyte-differs` | 2,555 | **1,880** | **−675** |
| `fnbyte-reloc-differs` | 861 | **532** | **−329** |
| `fnbyte-refused` | 130,116 | 131,120 | +1,004 |
| emitted census · per-function census | 39,644 · 712,238 | 39,644 · 712,238 | 0 · 0 |
| TU match (by NAME) · mismatch | 18 · 0 | **18, identical set** · **0** | 0 · 0 |

**The commission's coarse form was priced and DECLINED.** *"Refuse any body
whose callee this TU defines"* removes 2,530 differs and 858 reloc-differs — and
costs **1,074 byte-exact functions**, forty-three times decline clause D2's
stated size of 25. **1,055 of those 1,074 are byte-exact TAIL CALLS to a same-TU
callee**, which refutes the premise the coarse form rests on: c2 does not inline
every callee its TU defines. Board **#2151**.

**Why it keeps them is the size, and WB-INLINE reproduces from the other side.**
Every same-TU call site crossed against the callee's own COMDAT size in c2's
obj: below ~80 B the caller is **wrong 4,357** times and **right 10**; above it
**right 1,071** and **wrong 9**.
[`whitebox/WB_INLINE_FINDINGS.md`](whitebox/WB_INLINE_FINDINGS.md) F1/F9 were
measured on 320 compiled cells at swept flags; this is the same boundary read
out of the workload's own objs by the port's own failure pattern, on 60× the
population, with no fitting. Board **#2152**.

**And the input the port actually has is conservative in the safe direction.**
Not one byte-exact function in the workload has a local callee the port can
lower — all 1,081 are `port=none`, naming callees of 65–308 emitted bytes, which
is exactly the class c2 keeps the call to. So the shipped predicate fires on
1,004 functions that are **100 % wrong today** and on **zero** that are right.
Board **#2153**.

#### The 444 are untouched, and the reason is not the expected one

`work/w-inlfence2/reach.py` and `r2arm.py`, as set intersections per `(TU, sym)`
over three 878-TU scans (`05d743f7`, `0faa855a`, this tip):

```text
R2 = base \ pre  (w-fltret's increment)      : 444
REMOVED by the fence = base \ tip             : 675
of R2 (444), the fence removes                : 0    (0.0%)
of the BASE 2,111, the fence removes          : 675  (32.0%)
```

and at the fenced tip **444 of 444 are `localcallee`**. The fence *sees* every
one and declines to fire because `Timer::Split` and `Timer::Ms` are
`expr-op-0x27`: the IL parser refuses them, `TuContext::definition` returns
`None`, and there is **no size to test**.

> **The missing input is NOT definedness.** That is visible, cheap, and the
> fence uses it — `Bindings::names()` × `IlFunction::callees()`, the same cross
> `IlBundle::functions()` has done since long before this lane. **It is the
> callee's SIZE**, and the callee is exactly the thing the port cannot lower.

Board **#2155**. The residue — 1,855 differing and 529 reloc-differing
functions, all 444 among them — is priced at board **#2161** and needs the
callee's size **before codegen**, which is the quantity c2 itself uses
(`WORD [sym+0x50]`, WB_INLINE §2.1/§5). That is a fitted model; this lane ships
no fit.

#### Three findings that outlive the fence

1. **Board #139's rule does not reach this question** (**#2156**). *"Acceptance
   lives in the IL parser"* holds for every stage a parser clause can express.
   This one cannot be: whether the port still emits the call is decided **after**
   mechanism E (`elide`) and mechanism I (`splice`). A parser clause fires on
   both and un-ships them — the 1,074 is that price. The fence is the **fourth**
   post-lowering stage beside `gy-shape` and `data-ref`, both of which read 0 on
   this workload, which is why the rule's exceptions had never been tested.
2. **The accept side of the inline decision is safe to consult in exactly one
   place** (**#2160**). WB_INLINE §7 offers only decline rules because *"a
   mis-predicted accept is a wrong obj"* — a warning written for a lane that
   would **perform** the inline. When the prediction drives a **refusal**, a
   miss costs reach and cannot cost a byte. `noinline_boundary`'s `w04a` is that
   cost, compiled and pinned: **one function, and zero on the workload**.
3. **The 444 were never a live wrong-obj liability** (**#2159**), and this is
   the one inherited claim that did not survive re-derivation.
   `IlBundle::functions()` has refused any TU defining one of its own callees
   since long before `w-fltret`. `work/w-inlfence2/probe/M3.cpp` is the
   reduction: the census reads **`4/4 functions in class`** and the differential
   reads **`Port=NotImplemented`** for the same TU. The 444 were a **census and
   FBM** liability — a different repair, and a smaller one.

#### Two real-toolchain cells moved and both are recorded

* [`reloc_identity`](../crates/c2-harness/tests/reloc_identity.rs) `s12`:
  `RelocDiffers(Target)` → **`Refused`**. **The repair.** `s12` is the canonical
  reproducer, and 858 of the workload's 861 `fnbyte-reloc-differs` bodies
  relocate against a name their own TU defines.
* [`noinline_boundary`](../crates/c2-harness/tests/noinline_boundary.rs) `w04a`:
  `Exact` → **`Refused`**. **The cost**, and board **#1039**'s undecoded
  two-byte `.gl` field is why the port cannot see the attribute.

Both files now assert their finding **against c2's own relocation table**
instead of inferring it from the port agreeing — strictly stronger, because a
verdict of `Exact` never said *what* the two sides agreed on.

#### On the shipped class

This lane does **not** revert `w-fltret` (decline clause D7) and does not
recommend leaving #2089 as it stands. The 444 are not a peculiarity of the float
value tail: they are 13 % of a 3,416-function `localcallee` population that has
been on this board since the MVP, and the same defect is live in `int-tail-call`
(#2091). A revert would move `fnbyte-differs` and the emitted census by −444
each and leave 1,411 identical functions behind it. **The fence is the general
repair and the revert is not**; the decision on the class is the coordinator's.

[`rungs/2026-08-09-w-inlfence2.md`](rungs/2026-08-09-w-inlfence2.md).
