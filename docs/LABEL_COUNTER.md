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

`scripts/gt_label_inline.py`, **159 probe families**, `/O1 /GS- /c` and
`/Ox /GS- /c`, every row's in-object control held. Each family carries a
predicted charge that the script checks and can print `REFUTES` against; the
handful recorded as `NOT MODELLED` are listed with their reasons in §6.11
and §6.12.

> **Rounds 13–17 (§6.8–§6.11) extended the law to the C++ shapes and, in doing
> so, retracted most of §6.7.** Of the ten shapes §6.7 recorded as beyond law
> L′, **eight are now modelled and exact**; the `switch` was never non-uniform
> in N; the constructor and destructor were never refutations at all, only law
> L′ asked about a tree two instances deeper than the prediction assumed. Two
> rows were left as **live refutations** the script re-ran rather than
> remembered.
>
> **Rounds 18–22 (§6.12) took the law down the depth ladder and rewrote two of
> those three new rules.** The `switch` survives depth 3 exactly. The
> scope-exit charge is **not an `E` unit** — arithmetically it cannot be one —
> and is `d + 1`. The addressability +1 is **not a per-callee charge**: four
> pre-registered rivals died, and what is left fires only when P's entire
> expansion is flat, which in a real TU is almost never. Both of §6.11's live
> refutations are consequences of the corrections rather than outstanding
> misses, and the wordings they killed are re-refuted from each run's own
> numbers by the script's `SUPERSEDED` dict.

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
at the call site, no callee function anywhere in the TU. On **72 of the 100**
families the control's charge is **0 at every N** — and on the ones where it is
not, it is P paying a §1.1 surcharge for code it now contains (a pooled
constant, a materialised relational, a loop), never anything to do with
inlining. Where the two objs' `.text` for P can be compared at all they are
**byte-for-byte identical** (`TEXT-IDENTICAL` in the run log). For `framed` at
N=6: two objs, the same 120 bytes of `.text` for P, strides **23 and 5**.

> The inline charge is bookkeeping the IL carries about the expansion. It is
> invisible in the code and it is worth 18 label slots.

That is the practically important half of this section. Any emitter that decides
label numbers by looking at the *instructions* it is about to emit will be wrong
by 3 per inlined site on a TU it otherwise gets exactly right.

> **Latent in this port, not live — checked rather than argued (2026-07-31).**
> `PortC2` decides label numbers exactly that way, so the hazard above is real
> for it. What stops it reaching an obj is the TU-level gate in
> `crates/c2-il/src/func/bundle.rs`: *"a callee that is also DEFINED here is out
> of class: c2 may inline it"* — a rule that was added for a different reason
> (c2 cloned a callee into its caller and the port emitted a `b` against an
> external that no longer existed) and turns out to cover this one exactly.
> Verified through `c2rs diff` on both spellings that reach the inliner —
> `static int helper(int a){...}` called twice, and an `inline` function called
> once — with a second, non-inlined function present so the counter has somewhere
> to go wrong. Both return `Port=NotImplemented`, reference replay byte-exact.
>
> So this section is a **licence check**, in the same sense as §2.1: nothing the
> port emits today is wrong, and the number is here so that the first rung to
> relax that gate — which is what admitting real workload TUs will require, since
> they are inlined into constantly — cannot do it on the assumption that an
> inlined site is free. It is not free. It costs 3, and 2*d+1 + d*E when nested.

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
| …and an explicit `else` on top of it | 1 each | `cf-else` 5 vs `cf-if` 4 (§6.9) |
| a `switch`, by statement group | groups + 2 | `sw-arms2`…`sw-arms6` 7…11 (§6.8) |
| a parameter the body **assigns to** | 1 each | `parammod` 4 |
| an argument at that site that is not already a plain lvalue | 1 each | `arg-expr` 4, `arg-call` 4 (`arg-plain`, `arg-const` 3) |
| an **unnamed temporary object** | 1 each, **and a flat +1 beside it** | `ctor-noloc` 10, `d3-ctor-noloc` 28 (§6.12) |
| the hidden **return slot** of a by-value struct return | 1, alongside the declared local | `struct-ret` 5, `d2-struct-ret` 13 (§6.12) |

Two rules that used to be listed in this table are **not `E` features** — see
§6.12, which removed them from it:

| charge | worth | measured by |
|---|---|---|
| owning a local with a **non-trivial destructor** | `d + 1` at the owner's own depth, once per function, **not per object** and **not** `2d` | `dtor-3obj` 38, `d2-dtor` 27, `d3-dtor` 40 (§6.9, §6.12) |
| the address of a **scalar automatic** handed to the callee | **+1**, once per depth-1 instance, and **only when P's whole expansion is flat** | `ptr-param` 4 vs `ptr-global` 3 (§6.10); `d2-ptr-p` 8, `ptr-sibling` 11 (§6.12) |

plus **+1 flat, at any depth**, once per multi-exit callee whose result has to
be materialised — i.e. unless it is `void`, or its result is assigned straight
to a variable at depth 1 — and the *same* flat +1 once per unnamed temporary
(§6.12); plus, for each **loop** in that callee's body, a term that is *not*
part of the `d * E` product (§6.6):

| loop form | cost at depth `d` | d=1 | d=2 | d=3 |
|---|---|---:|---:|---:|
| `for` | `3d + 2` | 5 | 8 | 11 |
| `while` | `d + 3` | 4 | 5 | 6 |
| `do`/`while` | `2d + 2` | 4 | 6 | 8 |

The loop row is **`/O1` only** — see §6.4. Everything above it holds at `/Ox`
unchanged.

And on top of that, P still pays its **own** §1.1 surcharges for the code it
ends up containing. `cf-tern`'s 7 is 5 of bookkeeping plus the 2 that a
materialised signed relational costs — and the hand control measures that 2
independently on the same row.

> **Read `book`, not `marginal`, when you want the cost of inlining.** The run
> now prints `bookkeeping = marginal(inlined) − marginal(hand)` on every family:
> the inline record with P's own surcharge differenced out. On 72 of the
> original 100 families the hand control is 0 and the two are the same number.
> On the rest they are not, and confusing them is what made §6.7 report the
> `switch` as "10 at N=1, 14 marginal, not even uniform in N" — see §6.8. The
> `LAW_BOOK` dict grades the C++ shapes against `book`; the older `LAW` dict
> grades everything fitted before round 13 against the marginal, which is what
> those entries have always meant.

### What costs nothing

| | measured |
|---|---|
| the callee's own class — leaf vs framed vs FP vs multi-call | `leaf`, `framed`, `fp`, `body-2call`, `body-3call` all **3** |
| the number of **parameters** | `param0`…`param3` all **3** |
| lexical blocks, nested or not | `blk-nomod` 3, `blk1` 4 (that +1 is the parameter assignment), `blk2` **4** — they do not stack |
| the number of **distinct** callees | `distinct` **3/site** at 1…5 distinct callees, one site each |
| whether the callee introduces a symbol new to P | `newsym` = `samesym` = **3** |
| `static` vs `inline` vs a C++ **member function** | `framed` = `extern-inline` = `method` = **3** |
| a **constructor** or **destructor**, as an instance | `ctor-direct`, `dtor-direct-only` **3** — ordinary instances, `E = 0` (§6.9) |
| a struct passed **by value**, or by `const&` | `struct-param`, `struct-ref` **3** |
| whether a `switch`'s case values are dense or sparse | `sw-dense` = `switch-body` **10** (§6.8) |
| whether P is framed or a **leaf** | `leafP` **3** |
| a call the front end did **not** inline | `noinline` **0/site** |
| `/Gy` vs packed | §6.4 |

## 6.3 What the law predicted before it was measured

Predictions were written down and *then* captured, per this lane's standing
rule. They are not recalled here: each hold-out family in
`scripts/gt_label_inline.py` carries its `PRED n` in the family note, committed
in the same change that added the probe and before the row that graded it.
Hold-outs — shapes the law was **not** fitted to:

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
| `lp-min` / `lp-min-outer` / `lp-inf` / `lp-nested` | 9 / 9 / 9 / 16 | **9 / 9 / 9 / 16** ✓ |
| `d3-lp-for` (after fitting `for` = 3d+2 on d=1,2) | 32 | **32** ✓ |
| `d3-lp-while` / `d3-lp-do` (after fitting on d=1,2) | 27 / 29 | **27 / 29** ✓ |
| `d2-lp-for` | 22 scaled **or** 17 flat | **20** ✗ (both wrong) |
| `blk2` (two nested blocks) | 5 | **4** ✗ |
| `d2-inner-if` | 10 | **11** ✗ |
| `d3-inner-if` | 20 or 21 or 18 | **19** ✗ (all three rivals wrong) |

**The four misses are why the law has the shape it does**, and they are the
useful part of this table. `blk2` killed "a block costs 1" and pointed at the
parameter assignment instead (`parammod` confirmed it, `blk-nomod` confirmed the
negative). `d2-inner-if` and `d3-inner-if` added the multi-exit result temp *and*
pinned it to **+1 flat rather than depth-scaled** — the depth-scaled readings
predict 20 and 21 at depth 3 and the measurement is 19. `d2-lp-for` is the one
that forced loops out of the `d * E` product entirely; see §6.6.

`scripts/gt_label_inline.py` carries the whole law as a `LAW` dict, prints it
beside each family's measured slope, and prints
`<== *** REFUTES LAW L' ***` on any disagreement. **Score at the time this
subsection was written: 90 families, 0 refutations at `/O1`, 0 at `/Ox`.** The
current score, over all 159, is in §6.13.

## 6.4 Packed is the same law — but the *inliner* is not the same inliner

Re-run at `/Ox /GS- /c` (`base = 4`, controls held on every row): **every charge
is the same integer**, 0 families refuting. `/Gy` moves the framed base and
nothing else, exactly as §1.2 says for the other surcharges. That includes the
whole `for` ladder (`lp-for` 10, `d2-lp-for` 20, `d3-lp-for` 32, identical to
`/O1`).

Three `/Ox` artefacts the instrument had to learn to separate, **all three of
which read as a broken law first**:

* **A partial inline refusal.** The first `/Ox` run reported *ten* refutations,
  every one of them a loop family. They are not counter differences: at `/Ox`
  the front end declines to inline a `while`/`do` loop body at an **inner**
  level, so `d2-lp-while`'s expansion tree is one instance deep, not two, and
  the charge is honestly 3 rather than 17. Nothing in the stride, the residual
  or the linearity shows this — the charge stays exactly linear at the wrong
  value. What shows it is P's `.text` growth: 8 bytes per site (a `bl` and a
  `mr`) against the hand control's 32. The script now computes exactly that
  comparison, tags the row `INLINE-DECLINED?`, and downgrades the verdict from
  `REFUTES` to *"a different expansion tree"* — which is a claim about the
  inliner, printed with the bytes that justify it, not an excuse.

  > This is the failure mode worth carrying away from the whole section. A law
  > about the expansion tree is only ever as good as your evidence that the tree
  > is the one you think it is, and the *only* column that carries that evidence
  > is code size.

* **A one-off at N=1.** At `/Ox`, P pays +1 once for containing a branch at all —
  the *hand* control shows the same +1 and then goes flat. The N=1 slope
  therefore over-reads by 1 on `cf-void-if`, `d2-inner-void-if` and `d3-mid-if`.
  The script now reports the **marginal** slope (the last two rows differenced)
  beside the N=1 one and checks the law against the marginal, printing
  `one-off +1 at N=1` when they differ.
* **Total inliner refusal.** At both `/O1` and `/Ox` the front end abandons
  inlining outright once the accumulated body is big enough (`body-3call` at
  N=5, `lp-two`, and at `/Ox` most of the loop families from the first site
  onwards). The charge then simply stops growing, which is indistinguishable
  from a non-linearity in the counter. `dtext` collapses to zero or negative on
  exactly that row, and the same `INLINE-DECLINED?` check catches it.

So §6.6's three affine loop terms are stated for **`/O1`**. What `/Ox` actually
says about them, stated precisely rather than waved at:

* the **`for` ladder is identical** at `/Ox` — 10 / 20 / 32 at depths 1 / 2 / 3,
  and `lp-inf` 9. That is a real second-mode confirmation.
* the **`while` and `do/while` ladders cannot be measured at `/Ox`**, because
  the front end declines the inner inline on exactly those bodies. Not "they
  differ" — they are unobservable there, and the rows that look like differences
  are measuring a shallower tree.
* the small `for` bodies (`lp-min`, `lp-min-outer`, `lp-for-leaf`, `lp-nested`)
  are refused outright at `/Ox` from the first site.

§4 already records that a loop *written out* costs `for` +8 / nested +10 at
`/Ox` against +2 / +4 at `/O1` — "the only place in this document where the `/O`
level moves the counter" — and the hand controls here reproduce that (3/site
against 2/site). None of that reaches the inline charge itself.

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

## 6.6 Loops (`/O1`) — a loop is not an `E` feature, and the three forms do not share a slope

This was written up as "measured, not modelled" and then measured properly,
because a loop in an inlined body is not an exotic shape — it is what a real
workload TU is mostly made of. Everything in this subsection is `/O1`; §6.4 says
why there is no `/Ox` counterpart.

**At depth 1 a loop looks like an ordinary `E` feature.** Five probes, five
exact predictions from "3 + locals + L":

| probe | body | law | measured |
|---|---|---:|---:|
| `lp-min` | `for(int i=0;i<a;i++) gs(i);` — 1 local | 3+1+5 | **9** |
| `lp-min-outer` | same, `i` declared outside the `for` | 3+1+5 | **9** |
| `lp-inf` | `for(;;){ if (gs(a)) break; }` — 0 locals, 1 `if` | 3+0+1+5 | **9** |
| `lp-for` | `int t=0; for(int i…) t+=gs(i);` — 2 locals | 3+2+5 | **10** |
| `lp-nested` | two nested `for`s, 3 locals | 3+3+2·5 | **16** |

`lp-inf` is the one that matters: a `for` with **no** induction variable and no
accumulator still costs 5, so the 5 is the loop construct and not its locals.

**At depth 2 it stops behaving like one.** `d2-lp-for` was predicted at 22 (loop
scaled like every other feature) or 17 (loop flat like the result temp) and
measured **20**. Neither rival survives. Solving the two depths gives `3d + 2`,
and depth 3 was then held out: predicted **32**, measured **32**.

`while` and `do/while` were read, not predicted, at depth 2 — 17 and 18 — which
fixes them at `d + 3` and `2d + 2`. Both were then held out at depth 3:
predicted 27 and 29, measured **27 and 29**.

> **`while` and `do/while` both cost 4 at depth 1 and diverge at depth 2.** A
> capture set that only ever inlines one level deep merges them, gets every row
> right, and is wrong by 1 the first time a `while` appears two levels down.
> That is the same failure this document records for the FP leaf stride and for
> §4's own `do/while` +1 — the third instance of it, and it is why the loop term
> is written as three separate affine functions rather than one "loop" row.

Full ladder, all measured:

| body | depth 1 | depth 2 | depth 3 |
|---|---:|---:|---:|
| `for` accumulator body (`lp-for`) | 10 | 20 | 32 |
| `while` accumulator body | 9 | 17 | 27 |
| `do/while` accumulator body | 9 | 18 | 29 |

Only `lp-two` (two sequential `for`s) stays unmodelled, and not because of the
counter: it is 16 at N=1 and then the front end **refuses to keep inlining**, so
its marginal slope is 0 and there is nothing to check a law against. That row
measures the inliner's budget, not the label allocator.

## 6.7 Where law L′ was thought to stop (superseded by §6.8–§6.11)

Law L′ was fitted entirely on `int` scalars. A DC3-shaped TU passes structs by
value, returns them, takes references, and runs constructors, so those are all
hold-outs. Predictions were written before the capture and **four of eight
missed**:

| probe | predicted | measured | |
|---|---:|---:|---|
| `struct-param` — callee takes a 2-int struct **by value** | 4 | **3** | ✗ (the by-value copy is free) |
| `struct-ref` — callee takes `const S&` | 3 | **3** | ✓ |
| `struct-ret` — callee **returns** a 2-int struct by value | 4 | **5** | ✗ |
| `ref-param` — callee takes `int&`, writes through it | 3 | **4** | ✗ |
| `ptr-param` — callee takes `int*`, writes through it | 3 | **4** | ✗ |
| `switch-body` — 5-arm `switch` (§4: a `switch` written out costs **+0**) | 3 | **10 at N=1, 14 marginal** | ✗✗ |
| `ctor` — the callee constructs an object | 4 | **9** | ✗✗ |
| `dtor` — …and the class has a destructor | 4 | **16** | ✗✗ |

Two of the misses are small and have an obvious reconciliation. **The
reconciliation was tested and it is wrong.** `ref-param` and `ptr-param` are 4
rather than 3, and the tidy story is "binding a reference or taking an address
materialises the argument, so it is an argument that is not already a plain
lvalue and L′ already charges +1". Predicting *from* that story:

| probe | predicted by the story | measured |
|---|---:|---:|
| `ptr-already` — the argument is already a pointer **variable** | 3 | **4** |
| `ptr-global` — the argument is `&<a global>` | 4 | **3** |

Both inverted. Whatever the +1 is, it is not "the argument needed a temp"; the
only thing that separates the 4s from the 3s in these five rows is that the 4s
point **into a local of P** and the 3 points at a global.

> **That third story was then tested too, and it is also wrong.** §6.10 settles
> it: a `const int&` bound to a local **scalar** costs 4 and a `const SR&` bound
> to a local **struct** costs 3 — same storage class, same constness, same
> read-only use. It is not locality and it is not storage class. It is whether
> the pointee had to acquire an address.

### What §6.8–§6.11 retract from the table above

Everything in the table is a correctly measured number; what was wrong was the
disposition — three separate ways of asking law L′ the wrong question.

| row | §6.7's reading | what it actually is |
|---|---|---|
| `switch-body` | "10 at N=1, 14 marginal — not even uniform in N" | **uniform at 10.** `dhand` is +10/site flat to N=5; the 4 is what a *second written-out* switch costs P, on the same row, with nothing inlined (§6.8) |
| `ctor` = 9 | "2× the prediction" | **law L′ exactly**, on the depth-**2** tree the front end builds. A constructor is itself an inlined function; the prediction of 4 counted only the wrapper (§6.9) |
| `dtor` = 16 | "4× the prediction" | **law L′ plus one new rule** — the function that *owns* a destructible local pays E += 2, once (§6.9) |
| `ref-param` / `ptr-param` = 4 | "points into a local of P" | a scalar automatic that had to acquire an address (§6.10) |

And §4's "a `switch` written out costs **+0**" is itself incomplete: the *first*
written-out switch costs 0 and every one after it costs `groups − 1` (§6.8).

`struct-ret` and `ctor-noloc` are the two shapes still `NOT MODELLED`; see
§6.11, which also carries the two live refutations.

## 6.8 The `switch` is an ordinary E feature, and it was never non-uniform

§6.7 recorded `switch-body` at "10 at N=1, 14 marginal". Both numbers are right
and the reading was wrong. The same run's **hand control** — the identical
switch written out at the call site with no callee in the TU — charges 0 at
N=1 and then **4 per site**, and `dhand` (inlined minus hand, same N) is:

```
N        1     2     3     4     5
dhand  +10   +20   +30   +40   +50
```

Dead linear at **10 per site to N=5**, every row `TEXT-IDENTICAL`. The 4 is
what a *second* written-out `switch` costs P under §1.1 — P's own code, nothing
to do with inlining. The instrument now prints that subtraction as
`bookkeeping` on every family so the mistake cannot be repeated silently.

**The arm ladder, all four hold-outs exact.** Predicted before capture, from
"one E unit per case arm plus one for the construct plus the multi-exit temp",
against a slope-2 rival:

| probe | arms | predicted | measured |
|---|---:|---:|---:|
| `sw-arms2` | 2 | 7 (rival 4) | **7** ✓ |
| `sw-arms3` | 3 | 8 (rival 6) | **8** ✓ |
| `sw-arms4` | 4 | 9 (rival 8) | **9** ✓ |
| `switch-body` | 5 | — (fitted) | **10** |
| `sw-arms6` | 6 | 11 (rival 12) | **11** ✓ |

So at depth 1 the charge is `3 + E` with

```
    E(switch) = (number of statement groups) + 2
```

and the switch **scales with depth like every other E feature** — it is *not* an
affine term of its own the way a loop is (§6.6). Both depth-2 rows are exact:

| probe | law | measured |
|---|---:|---:|
| `d2-sw2` — 2 arms at depth 2, `3 + [5 + 2·4] + 1` | 17 | **17** ✓ |
| `d2-switch` — 5 arms at depth 2, `3 + [5 + 2·7] + 1` | 21 → **23** | **23** |

`d2-switch` is the one row whose *first* prediction missed (21 against 23) and
it missed for an instructive reason: the 21 was written assuming the +1 in
`E = groups + 2` was the multi-exit result temp. It is not — the temp is a
*separate* flat +1 that applies to a switch exactly as L′ already says it
applies to an `if`, and the hold-out that proves it is `sw-ctx-expr` (the same
five arms with the result used in an expression rather than assigned straight
to a variable): predicted **11**, measured **11**.

**What the +2 is, and what it is not.** Four probes, all predicted first:

| probe | question | predicted | measured |
|---|---|---:|---:|
| `sw-nodefault` | 3 arms, **no** `default` | 9 | **9** ✓ |
| `sw-withdefault` | the same body **plus** a written `default` | 10 | **10** ✓ |
| `sw-fall` | 5 case **labels** sharing 4 statement groups | 10 labels / 9 groups | **9** ✗ (labels) |
| `sw-dense` | 5 arms, **contiguous** case values | 10 | **10** ✓ |

So a `default` is counted only when it is **written** — there is no implicit
default arm in the charge — and what is counted is **statement groups, not case
labels**: `case 1: case 2: return x;` is one group and costs one. The +2 is
therefore not a hidden default; it is the construct itself, twice over, and it
survives with no default present. Dense and sparse case values cost the same,
so the jump table does not enter the counter.

`sw-void` (the same five arms, `void` callee) is **10**, equal to the
int-returning row, which is L′'s own multi-exit rule doing its job: a result
assigned straight to a variable at depth 1 is exempt, and so is `void`.
`sw-1exit` (five arms funnelled through one local and one `return`) is **11** =
10 + 1 for the local, confirming that ordinary E features stack on top of the
switch term normally.

> **The practical form.** A five-arm `switch` — a small one by real-workload
> standards — costs **10 label slots per inlined site at depth 1 and 23 at
> depth 2**, while the identical control flow written out costs 0 the first
> time. `PortC2` decides label numbers by looking at the instructions it is
> about to emit, so this is the single largest number in this document that
> such an emitter gets wrong, and switches are not rare in a DC3 TU.

## 6.9 Constructors and destructors: the tree was two instances deeper

§6.7 graded `ctor` at 9 against a prediction of 4 and called it a 2× miss. It is
not a miss. **A constructor is itself an inlined function**, so
`static int lct(int a){ CT c(a); return c.v; }` is a **depth-2** expansion, and
law L′ on the tree the front end actually builds reads

```
    lct     depth 1, E = 1 (the declared local `c`)   ->  3 + 1*1 = 4
    CT::CT  depth 2, E = 0                            ->  2*2 + 1 = 5
                                                          ---------
                                                                  9
```

which is what was measured. The prediction of 4 was law L′ asked about a
depth-1 tree. **This is the §6.4 failure mode with the sign reversed**: there
the front end declined an inline and the law was asked about a *shallower*
tree; here it silently took one more and the law was asked about a *deeper*
one. Neither is visible in the stride, the residual or the linearity. The
`EMIT:` column carries it — `??0CT@@QAA@H@Z` appears as its own COMDAT beside
the wrapper, exactly as §6.5 says an inlined callee does.

The probe that settles it structurally rather than arithmetically is
`ctor-direct`, where **P** constructs the object so the constructor sits at
depth 1 and the depth term is read directly: predicted **3**, measured **3** —
an ordinary inline instance with `E = 0`, indistinguishable from a free
function. Four more, all predicted first:

| probe | law | measured |
|---|---:|---:|
| `ctor-loc` — the ctor **body** declares a local, `4 + [5 + 2·1]` | 11 | **11** ✓ |
| `ctor-init` — a member-initializer list instead of an assignment | 9 | **9** ✓ |
| `ctor-2mem` — the ctor assigns **two** members | 9 | **9** ✓ |
| `ctor-if` — the ctor body has an `if`/`else` | 11 | **13** ✗ |

`ctor-if` overshot by exactly 2 = 2 × (depth 2), i.e. one extra E unit, and the
only difference from `cf-if` is an explicit `else`. Tested at depth 1, where
the scaling cannot hide it:

| probe | body | law | measured |
|---|---|---:|---:|
| `cf-if` | `if (a>0) return gs(a); return a+1;` | 4 | **4** |
| `cf-else` | the same code **with `else`** | 5 if `else` counts | **5** ✓ |
| `cf-else-assign` | `int r; if…else…; return r;` | 6 | **6** ✓ |

> **An explicit `else` is its own `E` unit.** This is a new row in §6.2's
> feature table and it is not a corner case — a capture set built only from
> `if`-plus-fallthrough merges it, gets every row right, and is wrong by `d`
> the first time an `if/else` appears. It is the fourth instance of that
> pattern this document records (the FP leaf stride, §4's `do/while`, §6.6's
> `while` vs `do/while`, and now this).

### The destructor, and the one genuinely new rule

`dtor` = 16 against L′'s 14 for the three-instance tree. The tempting fix is to
give the destructor `E = 1`. **`dtor-direct` refutes it**: P declaring the
object itself costs **6** = 3 (ctor at depth 1) + 3 (dtor at depth 1), so a
destructor is an ordinary instance with `E = 0`, exactly like a constructor.
`dtor-direct-only` (a dtor-only class, no user ctor) is **3**, confirming it.

The 2 is not in the destructor. It is charged to the function that **owns** the
destructible object:

```
    a function that owns any local with a non-trivial destructor
    pays  E += 2,  ONCE — not per object.
```

> **Superseded by §6.12 — the `ONCE, not per object` half is right and the
> `E += 2` half is not.** Every row in this subsection has its owner at depth 1,
> where an `E` unit is worth `2·1 = 2` and so is the true charge; the two only
> separate when the owner is deeper. Measured, the charge is `d + 1` at the
> owner's own depth, and it is a **separate term, not an `E` unit at all** —
> at depth 2 no integer `E` even reaches the measured value. Read §6.12 before
> using the decompositions below at any depth but 1.

P pays nothing because P is not an inline instance. That single rule fits every
measured row, and three of them were held out from it:

| probe | decomposition | law | measured |
|---|---|---:|---:|
| `dtor` | `[3+1+2] + 5 + 5` | 16 | **16** |
| `dtor-only` (no user ctor) | `[3+1+2] + 5` | 11 | **11** ✓ |
| `dtor-2obj` | `[3+2+2] + 2·5 + 2·5` | 27 | **27** ✓ |
| `dtor-3obj` — **hold-out** | `[3+3+2] + 3·5 + 3·5` | **38** (42 if per-object) | **38** ✓ |
| `dtor-body-loc` — **hold-out** | `16 + 2·1` | **18** | **18** ✓ |
| `dtor-empty` — **hold-out** | an empty `~DE(){}` is still a full instance | **16** | **16** ✓ |

`dtor-3obj` is the one that matters: per-object would give 42 and it is 38, so
the +2 is a **once-per-function scope-exit record**, not a per-object one.
`dtor-empty` is the §6.1 restatement — a destructor whose body is empty still
costs the full 5, because the charge is bookkeeping about the expansion and not
about the code.

## 6.10 The pointer/reference +1 is addressability, not storage and not locality

§6.7 offered three stories for why `ref-param` and `ptr-param` cost 4 where L′
says 3. The first two were turned into predictions and both inverted; the third
("it points into a local of P") was left as an observation. **It is wrong too.**
The row that kills it was already in §6.7's own table and went unread:
`struct-ref` binds a `const SR&` to a **local struct** of P and costs **3**,
while `ref-const-read` binds a `const int&` to a **local scalar** of P and costs
**4** — same storage class, same constness, same read-only use.

The rule that fits all twelve rows:

```
    +1, ONCE per callee, for a callee handed the address of a
    SCALAR AUTOMATIC variable — the one thing that has to leave a
    register in order to have an address at all.
```

| pointee | probe | law | measured |
|---|---|---:|---:|
| a scalar local of P | `ref-param`, `ptr-param`, `ref-const-read` | 4 | **4** |
| …reached through a pointer **variable** | `ptr-already` | 4 | **4** |
| …**two** such parameters | `ptr-2args` | 4 once / 5 per-arg | **4** ✓ |
| …one such plus one global | `ptr-mixed` | 4 | **4** ✓ |
| a global | `ptr-global`, `ref-global` | 3 | **3** ✓ |
| a **function-static** | `ptr-static-local` | 3 storage / 4 locality | **3** ✓ |
| two globals | `ptr-2global` | 3 | **3** ✓ |
| an element of a **local array** | `ptr-arrelem` | 3 addressability / 4 storage | **3** ✓ |
| a **member of a local struct** | `ref-member` | 3 / 4 | **3** ✓ |
| a whole local struct by `const&` | `struct-ref` | 3 | **3** |

Three things fall out that a "storage class" reading gets wrong. A
**function-static** is lexically inside P and costs 3, so it is not locality. A
**local array element** and a **local struct member** are automatic and cost 3,
so it is not storage class either. And `ref-const-read` costs 4 while never
writing through the reference, so it is not the write. What is left is the one
property the 4s share and the 3s do not: the pointee was a scalar living in a
register and now needs a stack slot.

The charge is **once per callee**, not per argument (`ptr-2args` = 4, and
`ptr-mixed` = 4 with only one of its two arguments qualifying).

> **Superseded by §6.12 on *when*, not on *which*.** Every row in the table
> above is a one-instance expansion, and on those the rule as stated is exact.
> It is wrong about everything else: the +1 does **not** fire at depth 2
> (`d2-ptr-auto`), does not fire on a depth-1 callee that is merely *handed*
> the address (`d2-ptr-p` = 8), does not fire on one that *uses* the pointee
> while something deeper also does (`ptr-use-d1` = 8), and does not survive a
> nested inline at an **unrelated call site** (`ptr-sibling` = 11). What
> actually gates it is that P's whole expansion be flat. The "one thing that
> has to leave a register in order to have an address at all" is still the best
> account of *which pointees* qualify — the table above stands — and is not an
> account of when the charge appears.

## 6.11 What is still `NOT MODELLED`, and the two refutations §6.12 answered

**Two rows were refutations of the law as extended above.** They were recorded
in `LAW_BOOK` as the *law's* prediction, not as the measurement, so every run
printed `*** REFUTES LAW L' ***` against them and a future fix had to face them
rather than inherit a fitted constant:

| probe | law said | measured | |
|---|---:|---:|---|
| `d2-dtor` — the destructible object one level deeper | 28 | **27** | off by 1 |
| `d2-ptr-auto` — the scalar-address +1 at depth 2 | 11 | **9** | the +1 does not fire at all |

Both are the same shape of failure, and it is the shape this lane keeps finding:
**a rule fitted where it was cheap to measure and then extended past its capture
set.** The scope-exit +2 is exact on eight rows whose owner is at depth 1 and
misses by one when the owner is at depth 2. The scalar-address +1 is exact on
ten rows at depth 1 and is simply absent at depth 2.

> **Both were answered later the same day by §6.12, and neither was fitted
> away.** The scope-exit charge is not an `E` unit at all — no integer `E`
> reaches `d2-dtor`'s measured value — and is `d + 1`, derived by subtracting a
> control that carries no contested term and then held out at depth 3. The
> scalar-address +1 turned out to be wrong in a second, larger way that
> `d2-ptr-auto` alone could not show: it does not fire when **P's expansion is
> not flat**, and four pre-registered rivals died establishing that. The retired
> wordings live in the script's `SUPERSEDED` dict and are **re-refuted from each
> run's own measurement** rather than remembered.

Left `NOT MODELLED` on purpose, because a number here is worse than a blank:

* ~~**`struct-ret`**~~ (a callee returning a 2-int struct by value) — 5 against
  L′'s 4. `E = 2` fits the single row if the hidden return slot counts alongside
  the declared local, but that is one row and one free parameter, so it is not
  written down as a rule. **RESOLVED in §6.12**: predicted at depth 2 from that
  one parameter and measured exactly, against a rival that missed.
* ~~**`ctor-noloc`**~~ (`return CN(a).v;`, an unnamed temporary) — 10, where both
  pre-registered readings (8 for "no local", 9 for "the temporary is a local")
  missed. Two different decompositions reach 10 and nothing distinguishes them.
  **RESOLVED in §6.12**: depth distinguishes them. One `E` unit plus one flat
  unit — and a third pre-registered reading missed on the way.
* **`lp-two`** — §6.6; the inliner's budget, not the counter.
* the `while` / `do-while` loop ladders at **`/Ox`** — §6.4; unobservable there,
  not different there.

### `/Ox` says the same thing, and the inliner's budget runs the other way

Re-run at `/Ox /GS- /c`: **every charge in §6.8–§6.10 is the same integer**, both
refutations included, so all of it is a two-mode result and not a `/O1`
artefact. The whole run is `controls failed: 0   families refuting LAW L': 2` in
both modes, on 138 families.

One asymmetry with §6.4 is worth carrying. §6.4 records `/Ox` as the mode that
*declines* inlines `/O1` accepts (the `while`/`do` loop bodies). On the C++
shapes it is the reverse: at `/O1` the front end abandons `sw-arms6`,
`dtor-2obj` and `dtor-3obj` at high site counts while `/Ox` inlines all six
sites and confirms 11 / 27 / 38. The `INLINE-DECLINED?` check caught all three
from P's `.text` growth and the verdict downgraded itself to *"a different
expansion tree"* rather than reporting three refutations — which is the check
earning its keep on shapes it was not written for. **The budget is not a
property of the optimisation level in one direction; it has to be checked per
shape, per mode, on every row.**

> **The riskiest thing still unmeasured on this axis** is no longer the C++
> shapes themselves — it is **depth**. Everything new in §6.8–§6.10 is exact at
> depth 1 and thin above it: the switch has two depth-2 rows and no depth-3 row
> at all, the scope-exit rule has one depth-2 row and it *misses*, and the
> scalar-address rule has one depth-2 row and it *misses*. Real workload TUs
> inline several levels deep — §6.3's `nest6` reaches six — and a ctor calling a
> ctor calling an accessor is an ordinary sight in DC3, not an exotic one. The
> inliner also refuses these shapes earlier than it refuses `int` scalars
> (`d2-switch` is declined outright at three sites), so the depth ladder has to
> be built with the `INLINE-DECLINED?` check live on every row or it will
> measure a shallower tree and stay perfectly linear while doing it.
>
> **That was written before §6.12 and it was right on every count**, including
> the last one: at `/Ox` the front end declines three of the four new switch
> shapes from the very first site, and only P's `.text` growth says so. Two of
> the three rules did not survive the ladder.

### A second reading of §6.6's loop terms, from the same run

The `bookkeeping` column also splits §6.6's three affine loop terms, at no extra
capture cost, into an inline part and a part that is P's own §1.1 surcharge for
containing a loop at all (which §4 measures independently at `for` +2 / `/O1`):

| form | §6.6's measured total | inline record | P's own |
|---|---|---|---|
| `for` | `3d + 2` | `3d` (3, 6, 9) | +2 |
| `while` | `d + 3` | `d + 1` (2, 3, 4) | +2 |
| `do`/`while` | `2d + 2` | `2d + 1` (3, 5, 7) | +1 |

Exact on all nine measured depth-1/2/3 rows. It is a tidier statement — the
`for` term loses its intercept entirely — but it is a **re-reading of existing
captures, not a new measurement**, and `lp-inf` (`for(;;)` with a `break`)
splits 2/3 rather than 3/2, so the decomposition is *not* established as
general across loop spellings. §6.6's table stays as the measured total.

## 6.12 The depth ladder — two of the three new rules did not survive it

§6.11 closed by naming **depth** as the riskiest thing unmeasured on this axis.
Twenty-one families later: the `switch` rule survives depth 3 unchanged, the
**scope-exit and addressability rules are both rewritten**, and the two rows
§6.11 refused to model turn out to be separable by depth and nothing else.
Nothing here was fitted away — every `PRED` below was committed to
`scripts/gt_label_inline.py` *before* the capture that graded it (the file's git
history is the record), and the corrections are derived from cells that were
held out from them. **Nine of the twenty-one registered predictions missed** —
`d2-dtor-only`, `d2-dtor-2obj`, `d3-dtor`, `d2-ptr-p`, `d3-ptr-auto`,
`ptr-use-d1`, `ptr-use-nest`, `ptr-sibling`, `d2-ctor-noloc` — and those are the
useful rows. Two of the three rules this ladder was built to test did not
survive it.

Every row in this section is `TEXT-IDENTICAL` to its hand control at every `N`,
which is the only column that carries evidence about how deep the expansion tree
actually is (§6.4). Where it is not, the row is reported as a budget finding and
not as a measurement — see the table at the end.

### The two uncontested controls, and why they come first

| probe | law | measured |
|---|---:|---:|
| `d2-ctor` — a constructed object at depth 2, `3 + [5+2·1] + 7` | 17 | **17** ✓ |
| `d3-ctor` — …at depth 3, `3 + 5 + [7+3·1] + 9` | 27 | **27** ✓ |

These carry **no contested term at all**: a constructed object with no
destructor is plain L′ arithmetic on a tree one and two instances deeper than
the wrapper. That makes `d2-ctor` the *instrument* for the scope-exit term
rather than another test of it, which is the whole point — §6.11's `d2-dtor`
miss could otherwise only be diagnosed from the row that was failing.

### Scope-exit is not an `E` unit. It is `d + 1`

> **`E += 2` was refuted, and it could not have been an `E` unit in the first
> place.** `d2-dtor` = 27 forces the depth-2 owner instance to cost 10, and
> `2d+1 + d·E` at `d = 2` is `5 + 2E` — **there is no integer `E` that reaches
> 10.** That is an arithmetic impossibility, not a poor fit. The scope-exit
> charge is a separate term, like a loop (§6.6), and never was an `E` feature.

Subtracting the control reads its value off the measurement:

```
    d2-dtor  27  =  3  +  [5 + 2·1 + S(2)]  +  7  +  7
    d2-ctor  17  =  3  +  [5 + 2·1       ]  +  7
    ------------------------------------------------------
    difference 10 = the depth-3 destructor instance (7)  +  S(2)   ->  S(2) = 3
```

The wrapper, the declared local, the constructor and the whole depth arithmetic
cancel. With `S(1) = 2` from eight depth-1-owner rows and `S(2) = 3` from that
subtraction, the affine form is `S(d) = d + 1`, and depth 3 was then held out:

| probe | old law (`E += 2`) | new law (`S(d)=d+1`) | other rivals | measured |
|---|---:|---:|---|---:|
| `d2-dtor` | 28 | 27 | — | **27** |
| `d2-dtor-only` — a dtor-only object at depth 2 | 21 | 20 | 19 flat | **20** ✓ |
| `d2-dtor-2obj` — **two** objects at depth 2 | 44 | 43 | 48 per-object, 42 flat | **43** ✓ |
| `d3-dtor` — the object at depth 3 — **hold-out** | 42 | **40** | 38 flat | **40** ✓ |
| `d2-dtor-if` — …**plus** an if/else at depth 2 | 34 | 33 | — | **33** ✓ |

`d3-dtor` is the one that counts: the affine form is pinned by `(1,2)` and
`(2,3)`, so `S(3) = 4` is an extrapolation, and it lands. `d2-dtor-2obj` says
the **once-per-function, not per-object** finding of §6.9 survives at depth 2
(per-object would be 48). `d2-dtor-if` says `d+1` is not an artefact of a callee
body that had nothing else in it — it stacks on three ordinary `E` features and
the total is still exact.

The depth-1 table of §6.9 is unaffected, because `S(1) = 2` either way. **That
is exactly the merge this document keeps recording** — the FP leaf stride, §4's
`do/while`, §6.6's `while` vs `do/while`, §6.9's `else` — and it is now the
fifth instance. A capture set that never inlines a destructible object two
levels deep gets every row right and is wrong by `d − 1` the first time one
appears.

### Addressability: four pre-registered rivals, four refutations

`d2-ptr-p` was written to separate two readings of §6.11's miss, and it
**killed both**. It measured **8** where the rivals said 9 and 11, and 8
decomposes as `3 + 5` with `E = 0` on *both* instances — so `pb2`, which sits at
depth 1 and is handed `&t` where `t` is P's own scalar automatic, pays nothing.
The shipped wording ("+1 once per callee **handed** the address of a scalar
automatic") is therefore wrong in two independent ways at once.

Each subsequent probe was predicted from the reading that survived the previous
one, and each killed it:

| probe | the reading it was predicted from | pred | measured | what died |
|---|---|---:|---:|---|
| `d2-ptr-p` | depth-scaled `E` unit / fires at depth 1 | 11 / 9 | **8** | both |
| `ptr-use-d1` | fires on a load/store through the address **at depth 1** | 9 | **8** | "use at depth 1" |
| `ptr-use-nest` | fires when the **deepest use** is at depth 1 | 9 | **8** | "deepest use" |
| `ptr-sibling` | scoped to the **call site's own tree** | 12 | **11** | "per tree" |
| `ptr-sibling-rev` | the kill is order-independent | 11 | **11** ✓ | — |
| `d3-ptr-auto` | never fires below depth 1 | 17 | **17** ✓ | — |
| `d2-ptr-glob` | control: the pointee is a global | 8 | **8** ✓ | — |

The arithmetic that all seven rows agree on is blunt: **a two-instance tree with
a pointer costs exactly what a two-instance tree without one costs.**
`d2-ptr-p` 8, `ptr-use-d1` 8, `ptr-use-nest` 8, `d2-ptr-glob` 8 — and `nest2`,
which has no pointer anywhere, is **8**. One instance deep, the pointer costs
one more than the same tree without it: `ptr-param` 4 against `nest1` 3.

```
    +1, once per depth-1 instance handed the address of a SCALAR AUTOMATIC —
    but ONLY when P's ENTIRE expansion is flat, i.e. P contains no inline
    instance below depth 1 at all.
```

`ptr-sibling` is what forces the last clause, and it is the row worth reading
twice: a two-deep tree at one call site and a one-deep pointer tree at another,
and the two-deep tree — which never touches the address, never sees the
pointer, and shares nothing with it but the enclosing function — **removes the
+1 from the other site**. `ptr-sibling-rev` puts the pointer site first and gets
the same 11, so it is a property of P and not of what the front end has already
expanded.

> **The practical consequence is that this rule is nearly dead in real code.** A
> DC3 TU's function inlines something that inlines something else, so P is
> essentially never flat, and the +1 essentially never fires. Where it *does*
> fire is the small flat function an emitter is most likely to believe it
> already has right. §6.10's "the one thing that has to leave a register in
> order to have an address at all" is still the best physical story for *which*
> pointees qualify, and it is now known not to be the story for *when*.

### The `switch` needed no correction

Predicted before capture, measured after, all four exact:

| probe | law | measured |
|---|---:|---:|
| `d3-switch` — 5 statement groups at depth 3, `3 + 5 + [7+3·7] + 1` | 37 | **37** ✓ |
| `d3-sw2` — 2 groups at depth 3, `3 + 5 + [7+3·4] + 1` | 28 | **28** ✓ |
| `d2-sw-void` — the same 5 arms, `void`, at depth 2 | 22 | **22** ✓ |
| `d2-sw-1exit` — 5 arms funnelled through one local, at depth 2 | 24 | **24** ✓ |

Two group counts at depth 3 give a group slope of `(37−28)/(5−2) = 3 = d`, so
`E(switch) = groups + 2` scales with depth at rate 1 three levels down, exactly
as an ordinary `E` feature. `d2-sw-void` is `d2-switch`'s 23 less 1: the temp
is **still flat two levels down**, which is §6.3's `d3-inner-if` finding holding
on a completely different construct. `d2-sw-1exit` is `d2-sw-void` plus `2·1`
for the local and no temp — ordinary `E` features stack on the switch term at
depth 2 as they do at depth 1.

`d2-mix` (three locals + an `if` + an `else` at depth 2, one exit) is **18**,
predicted 18: the `E` features simply add inside the `d·E` product two levels
down. Every other depth-2/3 row in this section varies exactly one feature, so
this is the row that says the additivity is not an accident of that design.

### The inliner's budget, per shape and per mode — checked, not inferred

§6.11 warns that the budget "has to be checked per shape, per mode, on every
row", and on these shapes it bites hard. `INLINE-DECLINED?` caught every case
from P's `.text` growth against the hand control:

| family | `/O1` | `/Ox` |
|---|---|---|
| `d3-switch` (5 groups, depth 3) | inlines to **N=2**, declines at N=3 | **declined from the first site** — 3/site, `dtext` 8 against the hand's 88, i.e. a `bl` |
| `d3-sw2` (2 groups, depth 3) | inlines to N≥3 | inlines to N≥3, **confirms 28** |
| `d2-sw-void`, `d2-sw-1exit` | inline to **N=2** | **declined from the first site** (0/site) |
| every ctor / dtor / pointer row above | N≥3 | N≥3, **every charge the same integer** |

So the whole ctor/dtor/addressability ladder is a **two-mode** result, and
three of the four switch rows are `/O1`-only — *unobservable* at `/Ox`, not
different there, in the same sense as §6.4's `while`/`do` ladders. Note the
direction: §6.4 has `/Ox` declining what `/O1` accepts, §6.11 has `/O1`
declining what `/Ox` accepts, and here `/Ox` declines a shape `/O1` takes to two
sites. **There is no monotone reading of the budget in the optimisation level,
and there never has been.**

### Depth also settles the two rows §6.11 refused to put a number on

`struct-ret` and `ctor-noloc` were left blank because several decompositions
reached the one measured value and nothing at depth 1 could separate them. That
is exactly what depth separates: **an `E` unit is worth `d` and a flat unit is
worth 1 however deep it sits** — the same lever §6.3 used to pin the multi-exit
result temp as flat rather than scaled.

| probe | readings it separates | pred | measured |
|---|---|---:|---:|
| `d2-struct-ret` | `E = 2` (the hidden return slot counts alongside the declared local) **vs** `E = 1` + a flat +1 | 13 / 12 | **13** ✓ |
| `d2-ctor-noloc` | the temporary is 2 `E` units / 1 `E` + 1 flat / 2 flat | 19 / 18 / 17 | **18** ✗ |
| `d3-ctor-noloc` | …1 `E` + 1 **flat** vs the second unit scaling after all | 28 / 30 | **28** ✓ |

**`struct-ret` is resolved and tested**: one free parameter, fitted at depth 1
and then *predicted* at depth 2 against a rival that missed. `E = 2` — the
hidden return slot counts as a second declared local.

**`ctor-noloc` is resolved, and the registered prediction was wrong.** I
recorded reading A (two `E` units, 19) in `LAW_BOOK` before the capture, with
the bias written down beside it — *"'surely it just adds' has been wrong twice
in this file already"* — and it was wrong a third time. The measurement is 18:
an unnamed temporary is **one `E` unit and one flat unit**, i.e. it counts as a
declared local *and* its materialisation costs the same flat +1 that a
multi-exit result costs. That is two parameters solved from two cells, which is
exactly determined and therefore tests nothing, so `d3-ctor-noloc` was added as
the first cell that could disagree: **28**, predicted 28 against 30 for a
second unit that scaled. Both readings hold at `/Ox` unchanged.

`ctor-noloc` = `3 + 1·1 + 1 + 5` = 10 and `d3-ctor-noloc` = `3 + 5 + [7+3·1] +
1 + 9` = 28 — the same two constants, three depths apart.

### What this section leaves `NOT MODELLED`

* **`S(d)` above depth 3.** Three points fix an affine form and the fourth was
  the hold-out; `d = 4` is unmeasured, and the ctor/dtor tree reaches depth 4
  in `d3-dtor` only as an ordinary instance, never as an owner.
* **the addressability rule's "flat P" clause rests on two rows** (`ptr-sibling`
  and its reverse). They are decisive against the three rivals that preceded
  them, but the clause is a strange shape for a rule and deserves more cells
  before an emitter leans on it. Since it can only ever *remove* a +1, the safe
  reading for a port is to treat the +1 as absent.
* **the depth-2/3 `switch` at `/Ox`** — unobservable, per the budget table.
* `lp-two` and the `while`/`do-while` ladders at `/Ox` — both pre-existing, and
  both the inliner's budget rather than the counter. **`struct-ret` and
  `ctor-noloc` have left this list**, see above.

## 6.13 Reproduction

```sh
export C2RS_WIBO=<the repo's resolved wibo>          # NOT ../wibo/build/wibo
scripts/gt_label_inline.py                            # /O1, all 159 families
scripts/gt_label_inline.py --mode '/Ox /GS- /c'       # packed
scripts/gt_label_inline.py --max 6 framed leaf        # the retraction, in 6 s
scripts/gt_label_inline.py nest1 nest2 nest3 nest4 nest5 nest6   # the depth law
scripts/gt_label_inline.py lp-for d2-lp-for d3-lp-for            # the loop ladder
scripts/gt_label_inline.py --max 3 sw-arms2 sw-arms3 sw-arms4 \
    switch-body sw-arms6                              # §6.8, the arm ladder
scripts/gt_label_inline.py --max 2 d2-sw2 d2-switch   # …at depth 2
scripts/gt_label_inline.py --max 3 ctor ctor-direct dtor dtor-direct \
    dtor-3obj                                         # §6.9, the ctor/dtor tree
scripts/gt_label_inline.py --max 3 ref-const-read struct-ref ptr-arrelem \
    ptr-static-local                                  # §6.10, addressability
scripts/gt_label_inline.py --max 3 d2-dtor d2-ptr-auto  # §6.11's two, now answered
# --- §6.12, the depth ladder ---------------------------------------------
scripts/gt_label_inline.py --max 3 d2-ctor d3-ctor    # the uncontested controls
scripts/gt_label_inline.py --max 3 d2-dtor d2-ctor d2-dtor-only \
    d2-dtor-2obj d3-dtor d2-dtor-if                   # scope-exit = d+1
scripts/gt_label_inline.py --max 3 d2-ptr-p ptr-use-d1 ptr-use-nest \
    ptr-sibling ptr-sibling-rev d3-ptr-auto d2-ptr-glob   # the four rivals
scripts/gt_label_inline.py --max 2 d3-switch d3-sw2 d2-sw-void d2-sw-1exit
scripts/gt_label_inline.py --list
```

`--max 2` on the depth-2 **and depth-3** switch rows is not tuning: the front
end abandons inlining that shape at three sites, the run tags the row
`INLINE-DECLINED?` from P's own `.text` growth, and the charge stops growing.
Reading a marginal across that row measures the inliner, not the counter. At
`/Ox` three of those four rows are declined **from the first site** and are
unobservable rather than different — §6.12's budget table.

The last line of a run is
`controls failed: 0   families refuting LAW L': 0`, on 159 families in both
modes. The second number is the one to read: it is the law's own falsifier,
computed rather than remembered. It reached 0 by the two rows §6.11 left
printing being **explained** — see §6.12 — and the wordings they refuted are
kept in the script's `SUPERSEDED` dict, which re-derives each refutation from
the run's own measurement and prints it beside the verdict:

```
d2-dtor  -> ... law(book) 27 OK   [retired 'scope-exit as E += 2' said 28, measured 27]
```
