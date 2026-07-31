# The compiler-label counter — measured seed-free

The `$M`/`$T` numbers c2 stamps into the symbol table come from one running
counter. Getting it wrong is six wrong bytes in an obj that still links, and it
has been wrong three times: the `/Gy` framed stride (5, actually 7 for
helper-using functions), the FP leaf stride ("touches FP ⇒ 2", right at exactly
one FP function), and — this document — the *unified* rule that replaced them.

Everything below is bytes out of an obj produced by the real toolchain
(`cl.exe` 16.00.11886.00 under wibo 1.0.1-23), captured and reduced by
`scripts/gt_label_stride.py`. Flags are labelled per table; `/O1` implies `/Gy`,
`/Ox` alone does not.

---

## 0. The instrument, and why it is different from §3.5's

`docs/OBJ_GY_SHAPES.md` §3.5 measures the counter against a **seed** read out of
the IL (`B = u32(.gl[7..11]) + 9`). That works, but it couples every reading to
a second quantity, and §3.4 already records the failure mode in this exact
place: *a stride and a seed that are both unknown can absorb each other's
error*. It also forces one TU per probe, and the seed is a function of the
source text, so no two probe TUs are directly comparable — the trap that made a
raw label seed unreadable when two probes' mangled names had different lengths.

`scripts/gt_label_stride.py` deletes the seed. Every probe is compiled as one TU

```c
    <declarations>
    int a0(int a){ return ga(a)+1; }      /* anchor  */
    <the probe function P>
    int a1(int a){ return ga(a)+2; }      /* anchor  */
    int a2(int a){ return ga(a)+3; }      /* anchor  */
```

and, writing `first(F)` for the lowest `$M`/`$T` in F's symbol group:

```
    base      = first(a2) - first(a1)                 measured IN THIS OBJ
    stride(P) = first(a1) - first(a0) - base          slots P consumes in total
    extra(P)  = first(P)  - first(a0) - base          slots taken BEFORE P's own $M
```

Three consequences worth stating, because each kills one way of being wrong:

* Every number is a difference **inside one object**, so the seed, the mangled
  name lengths and the `/Gy` per-function surcharge all cancel exactly.
* `base` is *measured*, not assumed, by the third anchor — so the same script
  runs packed (`base = 4`) and under `/Gy` (`base = 5`) with no mode constant
  anywhere, and a run in which the anchors disagreed with each other would be
  reported rather than averaged.
* The flags reach `cl.exe` through `subprocess` argv, never a shell word-split,
  so the "packed and `/Gy` were silently the same capture" trap cannot recur.

`extra == stride - base` on **every framed probe below without exception**, so
the shipped phrasing "the helper pair's two slots are allocated *before* its own
`$M` pair" is not special to the helper: **every surcharge a function pays is
allocated ahead of its own `$M` pair.** That is stronger and simpler than what
§3.5/§4.4 claim, and it is what an emitter needs.

---

## 1. The measured table

`/O1 /GS- /c` (i.e. `/Gy`), `base = 5`, every row's control held. `minted` is a
mechanical count of the symbol-table entries in P's group that c2 minted itself
(everything but the function symbol and the callees the IL named) — see §3.

| probe | what P is | extra | stride | minted |
|---|---|---:|---:|---:|
| `plain` | Class A framed, one new callee external | 0 | **5** | 5 |
| `plain-3callees` | Class A framed, **three** new callee externals | 0 | **5** | 5 |
| `gpr1` | Class B, 1 saved GPR inline | 0 | **5** | 5 |
| `gpr2` | Class B, 2 saved GPRs inline | 0 | **5** | 5 |
| `gpr3` | Class C, `__savegprlr_29` | 2 | **7** | 7 |
| `gpr7` | Class C, `__savegprlr_25` | 2 | **7** | 7 |
| `gpr3-dup` | Class C reusing the **same** `_29` a previous function introduced | 0 | **5** | 5 |
| `gpr3-dup-wide` | Class C needing a **different** width (`_28`) from the lead's `_29` | 2 | **7** | 7 |
| `fp0` | framed, FP-touching, 1 saved FPR inline, first FP function | 1 | **6** | 6 |
| `fp0-led` | same, `_fltused` already charged to a lead | 0 | **5** | 5 |
| `fpr3` | Class D, 3 saved FPRs inline, first FP function | 1 | **6** | 6 |
| `fpr3-led` | Class D, 3 saved FPRs inline, `_fltused` led | 0 | **5** | 5 |
| `fpr4` | **Class E, `__savefpr_28`**, first FP function | 3 | **8** | 8 |
| `fpr4-led` | **Class E, `__savefpr_28`**, `_fltused` led | **2** | **7** | 7 |
| `fpr5-led` | Class E, `__savefpr_27` | **2** | **7** | 7 |
| `both` | **Class F, both helper pairs**, first FP function | 5 | **10** | 10 |
| `both-led` | **Class F, both helper pairs**, `_fltused` led | **4** | **9** | 9 |
| `const1-led` | framed, ONE newly pooled FP constant | 2 | **7** | 7 |
| `const2-led` | framed, TWO newly pooled FP constants | 4 | **9** | 9 |
| `const1-dup-led` | framed, reuses a constant an earlier function pooled | 0 | **5** | 5 |
| `gpr3-const-led` | Class C helper **and** one new pooled constant | 4 | **9** | 9 |
| `leaf-int` | int leaf | — | **1** | 1 |
| `leaf-tail` | tail-call leaf, one new callee external | — | **1** | 1 |
| `leaf-float` | float leaf, first FP function | — | **2** | 2 |
| `leaf-float-led` | float leaf, `_fltused` led | — | **1** | 1 |
| `leaf-double-led` | double leaf, `_fltused` led | — | **1** | 1 |
| `leaf-float-c1-led` | float leaf, ONE pooled constant | — | **3** | 3 |
| `leaf-float-c2-led` | float leaf, TWO pooled constants | — | **5** | 5 |

Reproduce: `scripts/gt_label_stride.py` (whole table),
`scripts/gt_label_stride.py --mode '/Ox /GS- /c'` (packed).

## 1.1 The rule that fits it

> **stride = base + Σ surcharges**, where `base` is **1** for a leaf and **5**
> (`/Gy`) / **4** (packed) for a framed function, and every surcharge is
> allocated **before** the function's own `$M` pair.
>
> | surcharge | per | measured |
> |---|---|---:|
> | `_fltused` | the **first** FP-touching function in the TU | **+1** |
> | `__savegprlr_N` / `__restgprlr_N` | each **distinct N** first introduced | **+2** |
> | `__savefpr_M` / `__restfpr_M` | each **distinct M** first introduced | **+2** |
> | a newly pooled FP constant | each distinct `(bits,width)` first introduced | **+2** |
> | a callee external the IL names | — | **0**, at any count |
> | a helper width / FP constant an earlier function already introduced | — | **0** |

The three predictions `docs/CODEGEN_FRAMED_CALLS.md` §6 declined to claim are
now **measured and all three hold**:

* **the FPR helper pair costs +2** (`fpr4-led`, `fpr5-led`);
* **both pairs together cost +4** (`both-led`: stride 9 = 5 + 2 + 2);
* and the FP function that uses them pays `_fltused` on top when it is the first
  one (`both`: stride 10 = 5 + 1 + 2 + 2).

## 1.2 Packed is the same rule with a different base

Re-run at `/Ox /GS- /c` (`base = 4`, controls held on every row): **every
surcharge is byte-for-byte the same integer.** `gpr3` 6, `fpr4-led` 6,
`both-led` 8, `const2-led` 8, `fp0` 5, leaves 1/2/3/5 exactly as above.

> `/Gy` changes the framed **base** (4 → 5) and nothing else in this model. The
> per-function surcharges do not key on `/Gy` at all.

---

## 2. What this REFUTES

### 2.1 "one slot per TU-level external" — refuted (latent, not live)

`docs/CODEGEN_FRAMED_CALLS.md` §6 and `docs/ROADMAP.md` §6m unified the strides
as *"every function consumes 1 slot (framed: 4 packed / 5 `/Gy`), plus one extra
per TU-level external"*, reasoning from two data points: the GPR helper pair is
two externals and costs +2, and `_fltused` is one external and costs +1. That
rule is **wrong in both directions**, and the counterexamples are not exotic:

* **A newly pooled FP constant costs +2 and introduces no TU-level external at
  all.** `const1-led` is stride 7 against the rule's 5; `const2-led` is 9
  against 5. The rule mispredicts by 2 per constant. (The `.rdata` COMDAT it
  creates does add two *symbols* — a section symbol and the `__real@` — which is
  where §3 goes.)
* **A string literal costs 0 while creating exactly the same shape of `.rdata`
  COMDAT.** `void P(const char** p){ p[0]="hello"; }` is stride **1**; two
  strings **1**; three strings **1**. Three `.rdata` COMDATs, three `??_C@…`
  symbols, zero slots.
* **A materialised signed relational costs +2 and mints nothing.**
  `int P(int a){ return a<5; }` is stride **3**, two of them **5**, three **7** —
  linear at +2 each, and `docs/OBJ_GY_SHAPES.md` §3.6a already measured the
  single case as 3 without connecting it to this model. Nothing enters the
  symbol table.
* **A loop costs slots and mints nothing.** One loop 3, nested loops 5, a
  `while` 3, a `do/while` **2** — see §4, this one is not even uniform.

The rule is *not* merely incomplete: on any TU where a framed function follows a
float function that pools a constant, it under-counts by 2 per constant, which
is a wrong `$M` and a wrong `$T` in the obj.

> **It is latent, not live — checked rather than argued.** The rule is stated in
> `IlFunction::label_slots` and `coff::plan_labels` as the *reason* for the `+1`,
> but the shipped numbers (`+1` for `_fltused`, `4`/`5` framed, `1` leaf, 1-or-3
> comparison) are each measured independently and all still hold above. Every
> counterexample shape is refused by the TU-level gate in `func/bundle.rs`, which
> demands `label_slots(..) == 1` from every non-framed function in a TU that
> contains a framed one and is three-valued so an unmeasured class refuses. Three
> probes through `c2rs diff`, each a counterexample beside a framed function —
> a float leaf pooling `2.5f`, an FP store of a pooled constant in the framed
> body, and a materialised `a < 5` — return `Port=NotImplemented`, with the
> reference replay byte-exact on all three. What is live is the **licence**: the
> rule is the stated justification for widening the counter, and it would have
> admitted a pooled constant at +0.

### 2.2 "the helper pair's +2 is allocated before its own `$M` pair" — true, and true of everything else too

§4.4 states the pre-allocation as a property of the helper pair. Measured here:
`extra == stride - base` on **all 21 framed rows**, including `_fltused` (`fp0`:
extra 1), pooled constants (`const2-led`: extra 4), the comparison surcharge
(`framed-cmp`: extra 2) and every combination (`both`: extra 5). It is a
property of the counter, not of helpers.

### 2.3 "rest before save even though save is referenced first" — it is just the LIFO

§4.3 records the helper pair landing after `$T` with `__rest…` before
`__save…` as if the order were peculiar. The Class F capture makes it ordinary.
`work/gt/cf2.cpp`, `/O1`, a function using both pairs — code reference order is
`__savegprlr_28` (`.text+0x4`), `__savefpr_28` (`+0xc`), `__restfpr_28`
(`+0xa4`), `__restgprlr_28` (`+0xa8`), and the symbol table is

```
  [31] .pdata+aux (Number=8, Sel=5)   [33] $T2591
  [34] __restgprlr_28
  [35] __restfpr_28
  [36] __savefpr_28
  [37] __savegprlr_28
  [38] .text  (the NEXT function)
```

— the exact reverse of first-reference order, which is §4.1's rule for callee
externals and §2.3's for the `.rdata` pool. **One LIFO, three consumers.** The
callees in the same group obey it too (`?gp` referenced first, `?gd` emitted
first: `[28] ?gd  [29] ?gp`).

The same capture confirms the Class F code shape end to end: prologue
`mflr r12 ; bl __savegprlr_28 ; addi r12,r1,-40 ; bl __savefpr_28 ;
stwu r1,-160(r1)` (5 words, `-40 = -(8 + 8·4 saved GPRs)`), epilogue
`addi r1,r1,160 ; addi r12,r1,-40 ; bl __restfpr_28 ; b __restgprlr_28` —
**no `blr`**, the GPR restore is the tail branch and the FPR restore is a call.

---

## 3. The model that fits 28 rows and is still wrong

Worth recording because it is exactly the shape this lane exists to catch. Every
row in §1's table satisfies

> stride == the number of symbol-table entries c2 **mints** for the function
> (its section symbols, its `$M`/`$T` labels, and the synthesized externals
> `_fltused` / `__save*` / `__rest*` / `__real@`), excluding the function symbol
> itself and the callee externals the IL named.

Check it: a `/Gy` leaf mints one `.text` section symbol → 1. A `/Gy` framed
function mints `.text`, `$M`, `$M`, `.pdata`, `$T` → 5. A helper pair → +2. A
pooled constant mints `.rdata` + `__real@` → +2. Twenty-eight rows, twenty-eight
hits, and it *derives* the framed base rather than assuming it.

It is refuted by six rows: the three comparison leaves (stride 3, mints 1), the
loops (3 and 5, mints 1), the string literals (stride 1, mints 3 / 5 / 7 — the
sign is the other way), and packed mode, where a leaf mints nothing at all and
still costs 1. `scripts/gt_label_stride.py` prints `minted` beside `stride` on
every row and tags the disagreements, so the refutation is re-run automatically
rather than remembered.

The moral is the one `docs/GAPS.md` §6 keeps restating: a model fitted to the
classes that happen to be in the capture set reproduces them perfectly and says
nothing about the next class. **The counter is not a symbol count. It is c2's
internal label allocator, and symbols are one of several things that draw on
it.**

---

## 4. Uncharacterized, and therefore refuse rather than guess

| construct | measured | status |
|---|---|---|
| materialised signed relational (`a<5`, `a>=5`, `a>0`) | +2 each, linear to 3 of them | **established**, matches `OBJ_GY_SHAPES.md` §3.6a's "3" |
| `==` / `!=` / unsigned relational / `a<0` | +0 | established |
| an `if`/ternary that **branches** on a relational | +0 (`if-cmp` 1, `tern` 1, `tern-cmp` 3) | established — the surcharge is the **materialised value**, not the branch |
| `for` loop | +2, nested +4 at `/O1` | measured, **not modelled** |
| `while` | +2 | measured |
| `do/while` | **+1** | measured — *not* the same as `while`, so "per loop" is already wrong |
| a loop at `/Ox` | `for` +8, nested +10 | measured, and wildly different from `/O1` — the only place in this document where the `/O` level moves the counter, and it is the unroller changing the body, not `/Gy` |
| `switch` (8 dense arms) | +0 | measured; no jump table was emitted, so a jump-tabled switch is still unknown |
| integer `/`, `%`, variable shift | +0 | measured |
| a body the front end **inlined into** | ~~+3 / +8 / +13 for 1 / 2 / 3 sites, +5 per site after the first~~ **RETRACTED 2026-07-31 — see §6.** Measured against a family baseline it is **+3 per site, flat, from the first**, and the callee class does not enter it | **modelled**, §6 |
| whether the `/Gy` upfront surcharge is per function or per `.text` COMDAT | — | **ANSWERED 2026-07-31: per emitted `.text` COMDAT.** An unreferenced `static` (or `inline`) function is not emitted and costs **0** slots — `a0 → a1` is 5 with one sitting between them, exactly as with none. See §6.5 |

The `do/while` row is the small print worth reading twice: "a loop costs 2" is
already a rule fitted to two of three loop forms.

> **RETRACTION, 2026-07-31 — the inlining row above was wrong, and wrong in the
> way this document keeps warning about.** The numbers 3 / 8 / 13 were real
> readings, but they were differences against a *generic* framed function
> (stride 5), not against the same probe with the inlining removed. The probe
> `int P(int a){ return lst(a)+1; }` and its 2- and 3-site successors also
> changed their own register pressure and argument shapes as N grew, so three
> different effects were being added up and attributed to "per inlined site".
> With a family baseline (`stride(N) - stride(0)` of the *same* body) and
> chained sites that hold the class fixed, the same callee gives **3, 6, 9, 12,
> 15, 18** — flat at +3 per site including the first, linear to at least N=6.
>
> The unexplained constant was the whole of the error: there is no "first site
> is special" and no "+5". §6 replaces this row with a law tested on 74 probe
> families, and `scripts/gt_label_inline.py` re-runs the refutation.

---

## 5. Reproduction

```sh
export C2RS_WIBO=<the repo's resolved wibo>        # NOT ../wibo/build/wibo
scripts/gt_label_stride.py                          # /O1 (i.e. /Gy), base 5
scripts/gt_label_stride.py --mode '/Ox /GS- /c'     # packed, base 4
scripts/gt_label_stride.py --list                   # the probe catalogue
scripts/gt_label_stride.py fpr4-led both-led        # just the two that were predictions
```

Exit status is non-zero only if a *control* failed (an anchor pair disagreeing
with `base`), never because a prediction did — the table is the result, and the
`<== REFUTES stride==minted` tags are the refutation, printed rather than
recalled.

---

# 6. The label cost of inlining (2026-07-31)

`scripts/gt_label_inline.py`, 74 probe families, `/O1 /GS- /c` and
`/Ox /GS- /c`, every row's in-object control held.

**This section retracts §4's inlining row.** The retracted claim was "+3 / +8 /
+13 for 1 / 2 / 3 sites, +5 per site after the first". The truth is **+3 per
site, flat, from the first site**, and the +5 was an artefact of three separate
things being added together and called one.

## 6.0 Why the old number was wrong

Three defects, in descending order of how much they cost:

1. **No family baseline.** The charge was `stride(P) - 5`, i.e. a difference
   against a *generic* framed function. But the probe was
   `int P(int a){ return lst(a)+1; }`, which is not the same function as a plain
   framed one even with the inlining removed. The correct control is
   `stride(N) - stride(0)` **of the same body**, which is what the new script
   sweeps. That alone turns 3 / 8 / 13 into 3 / 6 / 9.
2. **The sites were not independent of the class.** Successive sites
   (`lst(a)`, `lst(a+1)`, `lst(a+2)`) added live values, so P's prologue class
   drifted with N and the `__savegprlr_29` pair's +2 landed inside the delta.
   The new probes *chain* (`s = f(s);`), so exactly one value is live across
   every site and the class cannot drift; when it does anyway the row is tagged
   `CLASS+` and the pair's cost is printed as `hcost` and subtracted in the
   open.
3. **The argument shape varied with N.** `lst(a)` takes a plain lvalue and
   `lst(a+1)` takes an expression, and §6.2 shows those differ by a slot. Three
   sites therefore mixed a 3 and two 4s.

Net of all three, the *same callee* the old row used gives **3, 6, 9, 12, 15,
18** for N = 1…6 — flat, linear, first site not special.

## 6.1 The charge buys no code at all

Every family carries a **hand-inlined control**: the callee's body written out
at the call site, no callee function anywhere in the TU. On 68 of 74 families
the control's charge is **0 at every N** *and* P's `.text` bytes are byte-for-byte
identical to the inlined variant's (`TEXT-IDENTICAL` in the run log). For
`framed` at N=6: two objs, same 120 bytes of `.text` for P, strides **23 and 5**.

> The inline charge is bookkeeping the IL carries about the expansion. It is
> invisible in the code and it is worth 18 label slots.

That is the practically important half of this section. Any emitter that decides
label numbers by looking at the *instructions* it is about to emit will be wrong
by 3 per inlined site on a TU it otherwise gets exactly right.

## 6.2 Law L′

For each inline instance `I` in the expansion tree of one call site (P's own
call sites are **depth 1**; a site inside a depth-`d` body is depth `d+1`):

```
    cost(I) = (2 * depth(I) + 1)  +  depth(I) * E(I)
```

where `E(I)` counts, **in that callee's body**:

| feature | counts | measured by |
|---|---|---|
| a declared local variable | 1 each | `loc0`…`loc5` → 3,4,5,6,7,8 |
| …of any type, including `double` | 1 | `hold-dbl-loc` 4 |
| …even if it generates no code | 1 | `loc1-dead` 4 |
| …two names in one declaration | 2 | `hold-2in1decl` 5 |
| an `if` | 1 each | `cf-if` 4, `cf-if2` 5 |
| a parameter the body **assigns to** | 1 each | `parammod` 4 |
| an argument at that site that is not already a plain lvalue | 1 each | `arg-expr` 4, `arg-call` 4 (`arg-plain`, `arg-const` 3) |

plus **+1 flat, at any depth**, once per multi-exit callee whose result has to
be materialised — i.e. unless it is `void`, or its result is assigned straight
to a variable at depth 1.

And on top of that, P still pays its **own** §1.1 surcharges for the code it
ends up containing. `cf-tern`'s 7 is 5 of bookkeeping plus the 2 that a
materialised signed relational costs — and the hand control measures that 2
independently on the same row.

### What costs nothing

| | measured |
|---|---|
| the callee's own class — leaf vs framed vs FP vs multi-call | `leaf`, `framed`, `fp`, `body-2call`, `body-3call` all **3** |
| the number of **parameters** | `param0`…`param3` all **3** |
| lexical blocks, nested or not | `blk-nomod` 3, `blk1` 4 (that +1 is the parameter assignment), `blk2` **4** — they do not stack |
| the number of **distinct** callees | `distinct` **3/site** at 1…5 distinct callees, one site each |
| whether the callee introduces a symbol new to P | `newsym` = `samesym` = **3** |
| `static` vs `inline` vs a C++ **member function** | `framed` = `extern-inline` = `method` = **3** |
| whether P is framed or a **leaf** | `leafP` **3** |
| a call the front end did **not** inline | `noinline` **0/site** |
| `/Gy` vs packed | §6.4 |

## 6.3 What the law predicted before it was measured

Predictions were written down (`work/gt-inline/PREDICTIONS.md`, per this lane's
standing rule) and then captured. Hold-outs — shapes the law was **not** fitted
to:

| probe | predicted | measured |
|---|---:|---:|
| `loc4` / `loc5` | 7 / 8 | **7 / 8** ✓ |
| `hold-2loc-if` (2 locals + 1 `if`) | 6 | **6** ✓ |
| `hold-3loc-2if` (3 locals + 2 `if`s) | 8 | **8** ✓ |
| `hold-loc-argexpr` (1 local + expression arg) | 5 | **5** ✓ |
| `hold-dbl-loc` / `hold-2in1decl` | 4 / 5 | **4 / 5** ✓ |
| `nest4` / `nest5` / `nest6` | 24 / 35 / 48 **(additive rival: 13 / 18 / 23)** | **24 / 35 / 48** ✓ |
| `fan3` (depth 2, fanout 3) | 22 | **22** ✓ |
| `parammod` / `blk-nomod` | 4 / 3 | **4 / 3** ✓ |
| `d2-outer-loc` / `d2-inner-loc` / `d3-inner-loc` | 9 / 10 / 18 (flat-feature rival: 9 / 9 / 16) | **9 / 10 / 18** ✓ |
| `ctx-expr-2ret` / `ctx-stmt-2ret` | 5 / 4 | **5 / 4** ✓ |
| `d2-inner-void-if` | 10 | **10** ✓ |
| `d3-mid-if` / `d3-two-if` | 18 / 22 | **18 / 22** ✓ |
| `method` / `method-loc` (C++ member fn) | 3 / 4 | **3 / 4** ✓ |
| `blk2` (two nested blocks) | 5 | **4** ✗ |
| `d2-inner-if` | 10 | **11** ✗ |
| `d3-inner-if` | 20 or 21 or 18 | **19** ✗ (all three rivals wrong) |

The three misses are why the law has the shape it does. `blk2` killed "a block
costs 1" and pointed at the parameter assignment instead (`parammod` confirmed
it, `blk-nomod` confirmed the negative). `d2-inner-if` and `d3-inner-if` are
what added the multi-exit result temp *and* pinned it to **+1 flat rather than
depth-scaled** — the depth-scaled readings predict 20 and 21 at depth 3 and the
measurement is 19.

`scripts/gt_label_inline.py` carries the whole law as a `LAW` dict, prints it
beside each family's measured slope, and prints
`<== *** REFUTES LAW L' ***` on any disagreement. **Current score: 74 families,
0 refutations at `/O1`, 0 at `/Ox`.**

## 6.4 Packed is the same law

Re-run at `/Ox /GS- /c` (`base = 4`, controls held on every row): **every charge
is the same integer**, 0 families refuting. `/Gy` moves the framed base and
nothing else, exactly as §1.2 says for the other surcharges.

Two `/Ox`-only artefacts the instrument had to learn to separate, both of which
would have read as a broken law:

* **A one-off at N=1.** At `/Ox`, P pays +1 once for containing a branch at all —
  the *hand* control shows the same +1 and then goes flat. The N=1 slope
  therefore over-reads by 1 on `cf-void-if`, `d2-inner-void-if` and `d3-mid-if`.
  The script now reports the **marginal** slope (the last two rows differenced)
  beside the N=1 one and checks the law against the marginal, printing
  `one-off +1 at N=1` when they differ.
* **Inliner refusal.** At both `/O1` and `/Ox` the front end abandons inlining
  once the accumulated body is big enough (`body-3call` at N=5, `lp-two`). The
  charge then simply stops growing, which is indistinguishable from a
  non-linearity in the counter. `dtext` (P's `.text` growth since N−1) collapses
  on exactly that row and it is tagged `INLINE-REFUSED?`.

## 6.5 Two structural findings that matter more than the arithmetic

**The callee is emitted anyway.** `cl 16.00.11886.00` emits a fully-inlined
`static` (and `inline`, and member) function as its own `.text`/`.pdata` COMDAT
regardless, *ahead of* the caller in the symbol table. So every symbol the
callee introduces — `__savegprlr_N`, `__savefpr_M`, `_fltused`, `__real@…` — is
charged to **its** group, not to the inlining caller's. The `const` family is
the demonstration: a callee that pools `2.5f` costs the caller **3**, the plain
inline charge, while writing the same expression out by hand costs the caller
**2** for the pooled constant. This is the opposite sign from what §1.1's
"per distinct symbol first introduced" rule predicts if you assume the inlined
body's symbols land on the caller. They do not.

**An un-emitted function costs 0 — which answers §4's other open row.** An
unreferenced `static` (or `inline`) function is dropped, and `first(a1) −
first(a0)` is **5** with one sitting between the anchors, identical to a TU with
none (measured: control 2560→2565→2570, with-dead-static 2565→2570→2575). So the
`/Gy` upfront surcharge is **per emitted `.text` COMDAT, not per source
function**.

## 6.6 Still not modelled — loops, deliberately

| callee body | charge/site | hand control |
|---|---:|---:|
| `for` with a call | 10 | 2 |
| `for` with no call | 10 | 2 |
| `while` | 9 | 2 |
| `do/while` | 9 | 1 |
| two sequential `for`s | 16 at N=1, then the inliner refuses | 4 |

Law L′ predicts 5 for the `for` bodies (3 + two locals `t` and `i`) and 5 for
the `while`/`do` bodies (3 + one local + one assigned parameter). Net of the
hand control the residual is **3, 3, 2, 3** — small, consistent, and completely
unexplained. §4 already refuses to model a loop's own +2/+2/+1, so a loop inside
an inlined body inherits that plus a residual of its own; `LAW` records these as
`None` and the script prints `law: NOT MODELLED` rather than a number.

**This is the riskiest thing still unmeasured on this axis**, because a loop in
an inlined body is not an exotic shape — it is what a real workload TU is mostly
made of.

## 6.7 Reproduction

```sh
export C2RS_WIBO=<the repo's resolved wibo>          # NOT ../wibo/build/wibo
scripts/gt_label_inline.py                            # /O1, all 74 families
scripts/gt_label_inline.py --mode '/Ox /GS- /c'       # packed
scripts/gt_label_inline.py --max 6 framed leaf        # the retraction, in 6 s
scripts/gt_label_inline.py nest1 nest2 nest3 nest4 nest5 nest6   # the depth law
scripts/gt_label_inline.py --list
```

The last line of a run is
`controls failed: 0   families refuting LAW L': 0`. The second number is the one
to read: it is the law's own falsifier, computed rather than remembered.
