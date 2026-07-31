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

`scripts/gt_label_inline.py`, **161 probe families**, `/O1 /GS- /c` and
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
| owning a local with a **non-trivial destructor** | `d + 1` at the owner's own depth, once per function, **not per object** and **not** `2d` | `dtor-3obj` 38, `d2-dtor` 27, `d3-dtor` 40 (§6.9, §6.12), `d4-dtor` 55 (§6.14) |
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

**The loop term and the scope-exit term ADD, and each keeps its own law**
(§6.14). They are the only two charges outside the `d·E` product and they were
fitted on disjoint bodies; measured together they are exact at depth 1
(`dtor-loop` 23/21), depth 2 (`d2-dtor-loop` 39/**37**, against two controls
carrying one term each) and depth 3 (`d3-dtor-loop` 57/**55**, a hold-out).
Nothing is shared, absorbed or capped between them.

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
current score, over all 161, is in §6.13.

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
artefact. The whole run **as this subsection was written** was
`controls failed: 0   families refuting LAW L': 2` in both modes, on 138
families; §6.12 added 23 more and drove the second number to 0 by explaining
those two rather than by adjusting them.

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
Twenty-three families later: the `switch` rule survives depth 3 unchanged, the
**scope-exit and addressability rules are both rewritten**, and the two rows
§6.11 refused to model turn out to be separable by depth and nothing else.
Nothing here was fitted away — every `PRED` below was committed to
`scripts/gt_label_inline.py` *before* the capture that graded it (the file's git
history is the record), and the corrections are derived from cells that were
held out from them. **Nine of the twenty-three registered predictions missed** —
`d2-dtor-only`, `d2-dtor-2obj`, `d3-dtor`, `d2-ptr-p`, `d3-ptr-auto`,
`ptr-use-d1`, `ptr-use-nest`, `ptr-sibling`, `d2-ctor-noloc` — and those are the
useful rows. Two of the three rules this ladder was built to test did not
survive it.

Every row in this section is `TEXT-IDENTICAL` to its hand control at every `N`
— except `ctor-base`, where the two objs' `.text` for P is the same **size**
(16 bytes per site, matching the control exactly) in a different instruction
order. That column is the only evidence about how deep the expansion tree
actually is (§6.4); where it collapses, the row is reported as a budget finding
and not as a measurement — see the table at the end.

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

### The base-class constructor is an ordinary instance one level down

§6.9's constructor tree was measured entirely on standalone `struct`s, and the
commonest constructor in a DC3 TU runs a **base-class** constructor first. That
is the shape where §6.9's own trap — a tree one instance deeper than the
prediction assumes — is easiest to fall into, so it was predicted before
capture:

| probe | law | rival | measured |
|---|---:|---:|---:|
| `ctor-base` — a derived ctor with a base ctor, `4 + [5] + [7]` | 16 | 11 if the base ctor folds in | **16** ✓ |

So a wrapper owning one derived object builds a **three-instance tree**:
the wrapper at depth 1, `DD2::DD2` at depth 2, `BB::BB` at depth 3 — and both
constructors appear as their own COMDATs in the `EMIT:` column, exactly as §6.5
says an inlined callee does. Same integer at `/Ox`.

This is the one row in the section whose `.text` is not byte-identical to its
hand control: the same 16 bytes per site in a different instruction order, so
the evidence that the inline happened is `dtext` matching the control at every
`N` rather than the hash. Worth naming rather than glossing, because `.text`
growth is the *only* column that carries depth evidence.

### The two terms outside the `d·E` product do add — and mind the units

Law L′ now has two charges that are *not* `E` features: §6.6's affine loop term
and §6.12's scope-exit `d + 1`. They were fitted on disjoint bodies and had
never appeared in the same instance, which is a large hole given that a DC3
constructor looping over its members is an ordinary sight.

`dtor-loop` — a callee that owns a destructible object **and** runs a `for`
loop — is the row, and it comes with a units trap worth recording:

| reading | decomposition | value |
|---|---|---:|
| marginal (what §6.6's `for` = `3d + 2` is stated in) | `[3 + 3 locals + 5 + S(1)=2] + 5 + 5` | **23** ✓ predicted |
| inline record (`bookkeeping`, what `LAW_BOOK` grades) | `[3 + 3 + 3d + 2] + 5 + 5` | **21** |
| the difference | P's own §1.1 surcharge for containing a loop at all | **+2/site**, measured by the hand control on this very row |

**The two terms simply add**, under either convention. Two things fall out for
free. The hand control reproduces §4's "a `for` loop written out costs P +2 at
`/O1`" **independently, on a body it was not fitted on**. And the inline record
of 21 is §6.11's split of the `for` term into `3d` plus P's own `+2` holding on
a new body — which §6.11 explicitly declined to call established, because
`lp-inf` splits 2/3 rather than 3/2. One more confirming body does not settle
that; it is **strengthened, still not general**.

The trap itself is the transferable part: **the `LAW` dict is stated in
marginals and `LAW_BOOK` in inline records, and the two only differ where the
hand control is non-zero.** This is the first family with both a loop and a
non-zero hand control, so it is the first place the choice of dict changes the
number — and filing a marginal prediction under `LAW_BOOK` reads as a
refutation when it is an arithmetic error. §6.2's warning to *"read `book`, not
`marginal`, when you want the cost of inlining"* has a converse, and this row is
it.

### The inliner's budget, per shape and per mode — checked, not inferred

§6.11 warns that the budget "has to be checked per shape, per mode, on every
row", and on these shapes it bites hard. `INLINE-DECLINED?` caught every case
from P's `.text` growth against the hand control:

| family | `/O1` | `/Ox` |
|---|---|---|
| `d3-switch` (5 groups, depth 3) | inlines to **N=2**, declines at N=3 | **declined from the first site** — 3/site, `dtext` 8 against the hand's 88, i.e. a `bl` |
| `d3-sw2` (2 groups, depth 3) | inlines to N≥3 | inlines to N≥3, **confirms 28** |
| `d2-sw-void`, `d2-sw-1exit` | inline to **N=2** | **declined from the first site** (0/site) |
| `dtor-loop` | N≥3 | **declined from the first site** — a small loop body, exactly §6.4's `/Ox` refusal |
| every ctor / dtor / pointer / temporary row above, and `ctor-base` | N≥3 | N≥3, **every charge the same integer** |

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

> **The riskiest thing still unmeasured on this axis** is no longer depth in
> the abstract — it is **depth combined with the non-`E` terms**. `dtor-loop`
> shows the loop term and the scope-exit term add *at depth 1*, and that is the
> only cell where they have ever met. Neither the loop term nor the scope-exit
> term has been measured against the other at depth 2, and the loop term has
> never been measured inside a C++ shape at any depth — no probe here puts a
> `for` in a constructor, which is what a DC3 container's constructor mostly
> is. The two terms have different depth laws (`3d + 2` against `d + 1`), so
> there is no reason beyond habit to expect them to keep adding when both are
> scaled; that is exactly the assumption §6.6 refuted for loops the first time
> it was made. **The probe to write next is `d2-dtor-loop`**, and its rivals
> separate by 3 at depth 2.
>
> Second on the list, and cheap: the whole ladder is `int` and small structs.
> A **virtual** call the front end devirtualises and inlines, and a **template**
> instantiation, are both ordinary in a real TU and neither has a single row.

> **Answered by §6.14 (round 27).** `d2-dtor-loop` was written and it **holds**:
> the two terms add at depth 2 and again at depth 3, each keeping its own depth
> law. `S(d) = d + 1` also survives at depth 4, which closes the first bullet
> above. What did *not* survive is the assumption that the DC3 shape could be
> measured at all — see §6.14.

## 6.13 Reproduction

```sh
export C2RS_WIBO=<the repo's resolved wibo>          # NOT ../wibo/build/wibo
scripts/gt_label_inline.py                            # /O1, all 161 families
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
# --- §6.14, round 27: the two non-E terms meet AT DEPTH ------------------
# --max 4, and NOT the default 6: at /O1 the inliner stops at N=5 on the
# joint rows, and a marginal read across that boundary measures the budget.
scripts/gt_label_inline.py --max 4 d2-loop-3loc d2-dtor-3loc d2-dtor-loop
scripts/gt_label_inline.py --max 4 d3-dtor-loop d4-dtor
scripts/gt_label_inline.py --max 4 ctor-loop d2-loop-asctor ctor-loop-leaf \
    d2-ctor-loop                                  # the DC3 shape + its controls
scripts/gt_label_inline.py --list
```

`--max 2` on the depth-2 **and depth-3** switch rows is not tuning: the front
end abandons inlining that shape at three sites, the run tags the row
`INLINE-DECLINED?` from P's own `.text` growth, and the charge stops growing.
Reading a marginal across that row measures the inliner, not the counter. At
`/Ox` three of those four rows are declined **from the first site** and are
unobservable rather than different — §6.12's budget table.

The last line of a run is
`controls failed: 0   families refuting LAW L': 0`, on 170 families in both
modes. The second number is the one to read: it is the law's own falsifier,
computed rather than remembered. It reached 0 by the two rows §6.11 left
printing being **explained** — see §6.12 — and the wordings they refuted are
kept in the script's `SUPERSEDED` dict, which re-derives each refutation from
the run's own measurement and prints it beside the verdict:

```
d2-dtor  -> ... law(book) 27 OK   [retired 'scope-exit as E += 2' said 28, measured 27]
```

## 6.14 Round 27 — the two non-`E` terms add at depth, and the shape that matters is the one the inliner refuses

§6.12 closed by naming this as the riskiest thing left: the loop term
(`for` = `3d + 2`, slope 3) and the scope-exit term (`S(d) = d + 1`, slope 1)
sit outside the `d·E` product, have **different depth laws**, and had met in
exactly one cell — `dtor-loop`, at depth 1, where they add. Nine families
later, **they add at depth 2 and at depth 3 as well**, and the row that was
supposed to carry the finding into a real TU turns out to be invisible to this
instrument for a reason worth more than the number would have been.

This is the first round in this lane in two days that **confirms rather than
refutes**, and saying so is the point: my pre-registered expectation
(`work/gt-relational/ESTIMATE-task2.txt`) was *"I expect 4 of the 7 to land…
my honest prior on 'they simply add' is about 55 %, NOT the ~85 % the shipped
law implies"*. **Every observable row landed — 7 of 7, on the integer, first
try.** Expecting a refutation because the last six probes produced one is the
same failure as expecting a confirmation because the last six did; the bias
had simply changed sign, and it was written down in advance, which is the only
reason it can be named now.

### The design: two controls carrying one contested term each

§6.12's own rule is that a correction must be derived from a **held-out** cell.
The same rule applied to a *test* means the joint cell must be predictable from
cells that are not it, so additivity is arithmetic rather than a fit:

| probe | what it carries | pred (marginal / book) | rivals | measured |
|---|---|---:|---|---:|
| `d2-loop-3loc` — the joint body **minus the object** | loop term only, depth 2 | 22 / **20** | 19 if the loop term does not scale in a 3-local body | **22 / 20** ✓ |
| `d2-dtor-3loc` — the joint body **minus the loop**, `E` held at 3 | scope-exit only, depth 2 | 31 / **31** | 32 if scope-exit were still `E += 2` | **31 / 31** ✓ |
| **`d2-dtor-loop`** — the joint cell | **both**, depth 2 | 39 / **37** | 36/34 three ways · 31/29 | **39 / 37** ✓ |

```
    22  (loop only)  +  31  (scope-exit only)  −  14  (shared base)  =  39
```

The joint measurement is **39**. The two terms simply add, each retaining its
own law: `L(2) = 8` and `S(2) = 3` inside one instance, with the `d·E` product
and the depth-3 ctor/dtor instances unchanged around them.

Three rivals shared the value 36 — scope-exit not paid beside a loop, the loop
pinned at its depth-1 value, and the two collapsing into one term at the loop's
law. None of them fired, so the controls were not needed to discriminate; they
are still what makes the 39 a *prediction* rather than a reading.

### It holds at depth 3, and `S(d)` holds at depth 4

| probe | pred (book) | rivals | measured |
|---|---:|---|---:|
| `d3-dtor-loop` — the joint cell at **depth 3** — hold-out | **55** | 57 under the retired `E += 2` | **55** ✓ |
| `d4-dtor` — the destructible object at **depth 4** — §6.12's first `NOT MODELLED` | **55** | 54 if `S` saturates at 4 · 58 under `E += 2` | **55** ✓ |

`d3-dtor-loop` = `3 + 5 + [7 + 3·3 + L(3)=11 + S(3)=4] + 9 + 9` = 57 marginal.
Both non-`E` terms scale independently, three levels down, in the same
instance. **The additivity is not a depth-2 coincidence.**

`d4-dtor` = `3 + 5 + 7 + [9 + 4·1 + S(4)=5] + 11 + 11` = 55, and the
saturating rival misses by 1. That separation is **thin, and was called thin in
the pre-registration rather than after the fact** — one row at a 1-wide
separation should not by itself move a rule. What makes it usable is that it is
the *fourth* point on a two-parameter affine form whose third point (`d3-dtor`)
was already an extrapolation that landed: `S(d) = d + 1` now has two successive
successful extrapolations, and the retired `E += 2` wording is re-refuted on
this row by 3.

### The DC3 shape: the front end refuses it, and law L′ predicts the refusal

§6.12 named "no probe puts a `for` inside a constructor, and a DC3 container's
constructor is mostly exactly that". Both probes that do come back
**`INLINE-DECLINED?`, in both modes**:

| probe | `/O1` | `/Ox` | charge |
|---|---|---|---:|
| `ctor-loop` — a `for` with a call, inside a ctor | declined **from the first site** | declined | **4**/site |
| `d2-ctor-loop` — the same, one level down | declined from N=2 | declined from N=2 | **10**/site |

A declined row is **unobservable, not refuted** — but these two are not silent.
Their charges decompose *exactly* as law L′ for the tree the front end actually
built, i.e. the tree with the constructor left as a `bl`:

```
    ctor-loop      4  =  3 + 1·1              the wrapper `lcl` alone, E = 1 (the object)
    d2-ctor-loop  10  =  3 + [5 + 2·1]        both wrappers, CL::CL left out
```

So law L′ is not being dodged here; it is predicting the *observed* expansion
tree to the integer while the *intended* one never got built. That is the
useful form of a decline, and it is only visible because `INLINE-DECLINED?`
identified which tree was measured.

**Two controls say exactly what the budget is refusing**, because "a decline"
is not a finding until you know which feature caused it:

| control | differs from `ctor-loop` by | `/O1` | measured |
|---|---|---|---:|
| `d2-loop-asctor` — the identical tree with a plain `static` function where the ctor was | ctor → plain function | **inlines to N=6** | 21 / **19** ✓ |
| `ctor-loop-leaf` — the same constructor, loop body makes **no call** | call in loop → arithmetic | **inlines to N=6** | 19 / **17** ✓ |

Neither the constructor alone nor the call-in-a-loop alone is refused. **It is
the conjunction.** And `ctor-loop-leaf` measures **17** — which is `ctor-loop`'s
own pre-registered prediction, on the identical tree minus a call that §6.6
already measured as free to the counter (`lp-for` = `lp-for-leaf` = 10). The
law's answer for the DC3 shape is therefore *corroborated on its nearest
observable neighbour* and **never measured on the shape itself**. Those are
different claims and the difference is the whole content of the next bullet
list.

### The budget, again, and again not monotone

| shape | `/O1` | `/Ox` |
|---|---|---|
| `d2-dtor-3loc` (3 locals + scope-exit, no loop) | inlines to **N=4**, declines at N=5 | inlines to **N=6** |
| `d2-dtor-loop`, `d3-dtor-loop` | inline to **N=4** | declined from **N=2** |
| `d2-loop-3loc` (3 locals + loop) | inlines to **N=5** | declined from **N=2** |
| `d2-loop-asctor` (2 locals + loop) | inlines to N=6 | **inlines to N=6** |
| `ctor-loop-leaf` (ctor + loop, no call) | inlines to N=6 | declined from N=2 |

Two things to read off it. `d2-dtor-3loc` is a **fourth** instance of the
non-monotonicity §6.12 recorded: `/O1` declines at N=5 what `/Ox` carries to
N=6, on a shape with no loop in it at all. And `d2-loop-3loc` versus
`d2-loop-asctor` at `/Ox` — declined from N=2 against inlined to N=6 — differ
by **one declared local**. "Loops are declined at `/Ox`" is false as a general
statement; the budget is counted per shape and one local is enough to cross it.

**These are `--max 4` results at `/O1`, and that is not tuning.** At the default
`--max 6` the joint rows read `0/site marginal, one-off +39 at N=1` — the
marginal is taken across the boundary where the inliner stops, so it measures
the budget rather than the counter, exactly as §6.13 records for the switch
rows. Inside the window every row is **exactly linear** (`d2-dtor-loop`
39 / 78 / 117 / 156 at N = 1…4), which is the check that the window is the right
one.

### One instrument fix, and it was under-reporting

`INLINE-DECLINED?` fires when P's `.text` growth is much smaller than the hand
control's, and the test was `dtext * 2 < hd`. `ctor-loop` at N=1 grows P by
**12** where the control grows it by **24** — a `bl` instead of a body, and
*exactly* on the boundary, so the strict `<` said nothing. Widened to `<=`.

The change was graded before it was kept, on all 170 families in both modes:
**not one verdict line moved**, and the flag count went 91 → 92 at `/O1`
(`ctor-loop` N=1) and 137 → 139 at `/Ox` (`noinline` N=3 and N=5 — the family
whose entire purpose is to be declined, and which was under-reported at two of
its six rows). Both new flags are true positives; there are no new false ones.
This matters beyond one row: trap #1 of this axis is that a wrong-depth
expansion tree is invisible in stride, residual and linearity, and `.text`
growth is the *only* column carrying that evidence. A detector that misses the
exactly-2× case is a detector that misses the cheapest possible decline — a
callee replaced by a single call.

### What round 27 leaves `NOT MODELLED`

* **The DC3 shape itself.** `ctor-loop` and `d2-ctor-loop` are unobservable in
  both modes. `ctor-loop-leaf` = 17 is the law's number on the identical tree
  minus a call known to be free, which is strong corroboration and **is not a
  measurement of the row**. Do not write 17 into a table as if it were.
* **`S(d)` above depth 4** — the bullet moved up one level rather than closing.
  Two successive extrapolations have landed, so the affine form is now the
  strongest-supported rule in §6.2; `d = 5` is still unmeasured.
* **the addressability "flat P" clause** — unchanged, still two rows. Since it
  can only ever *remove* a `+1`, a port should treat the `+1` as absent.
* **`lp-two`, the `while`/`do` ladders at `/Ox`, the depth-2/3 `switch` at
  `/Ox`** — unchanged, all budget rather than counter.
* **the non-`E` terms above depth 3, and `while`/`do` versions of the joint
  cell.** Only the `for` form has been carried to depth 3 beside a scope-exit.

> **The riskiest thing still unmeasured on this axis** is no longer whether the
> terms compose — they do, at three depths, on held-out cells. It is that
> **the shape the roadmap needs the law for is the shape the front end
> declines**. A DC3 container constructor loops over its members and calls
> something inside the loop, and that exact conjunction comes back
> `INLINE-DECLINED?` at `/O1` *and* `/Ox`, from the first site. The consequence
> is not that the law is untested there — it is that **an emitter which assumes
> the inline happened will build the wrong tree on the commonest shape in the
> corpus.** Law L′ predicts the declined tree exactly (`ctor-loop` 4,
> `d2-ctor-loop` 10), so the modelling job is real and tractable; what does not
> exist yet is any rule for *predicting the decline itself*, and the budget has
> now been shown non-monotone in the optimisation level four times and
> sensitive to a single declared local once. Until such a rule exists, a port
> that relaxes `bundle.rs`'s "a callee defined here may be inlined" gate has no
> way to know which tree it is counting for.
>
> Second, unchanged and still cheap: the whole ladder is `int` and small
> structs. A **virtual** call the front end devirtualises, and a **template**
> instantiation, are both ordinary in a real TU and neither has a single row.

> **Answered by §6.15 (round 28), with a second instrument and three
> refutations.** There *is* a rule at `/O1`: the decision is all-or-nothing per
> (caller, callee) pair and the number of sites taken is an exact function of
> the callee's own emitted `.text` size — including for the four families whose
> budget this section measured. `/Ox` is a different mechanism with no
> N-dependence at all. What did *not* survive is this section's own reading of
> two rows: `d2-loop-3loc` at `/Ox` is declined from **N=1**, at the **inner**
> level, and it differs from `d2-loop-asctor` by rather more than "one declared
> local". The `ctor-loop` refusal is **not** "the conjunction" — filling the
> 2×2 breaks it in both directions.

## 6.15 Round 28 — the DECLINE, on its own axis, with a second instrument

§6.14 closed by naming the gap: *"law L′ predicts the declined tree exactly …
what does not exist yet is any rule for predicting the decline itself."* This
round builds a predictor for `/O1`, characterizes `/Ox` separately as the
different mechanism it turns out to be, and refutes **four** things this
document had shipped — one of its own instruments (for the third time), and
three of §6.14's readings — plus **two** closed-form fits written and
pre-registered inside this round itself.

New instrument: `scripts/gt_inline_decline.py`. It does not use `.text`
growth. **An inlined call leaves no trace in P's relocation table; a declined
one leaves exactly one `bl` against the callee's symbol.** So the reloc
*count* is the number of sites that were declined — per-site resolution from
one capture — and the reloc *name* says **which instance of a multi-level tree
was refused**, which no inequality on byte counts can say. Both detectors are
printed side by side on every family row, so a disagreement is a printed row
rather than a memory.

### 6.15.0 The instrument lied a third time, and in a new place

**`/O1` implies `/Gy`. `/Ox` does not.** At `/O1` every function gets its own
`.text` COMDAT, so "P's section" and "P" are the same bytes. At `/Ox` this
compiler packs the whole TU into **one** `.text`, and `gt_label_inline.py` was
reading `len(section)` — P *plus both callees plus all three anchors*.

| | consequence |
|---|---|
| `dtext` at N=0→1, `/Ox` | inflated by the callee's **own first emission** — the largest single term in the sweep, and precisely the row where a decline has to be caught |
| `TEXT-IDENTICAL` at `/Ox` | a claim about the whole section, not about P; §6.12's *"every row is `TEXT-IDENTICAL` at every N"* is an **`/O1` statement** |
| `INLINE-DECLINED?` at `/Ox` N=1 | passes `noinline` — **the family whose entire purpose is to be declined** |

That last one is the same row §6.14 widened the test from `<` to `<=` for, and
the widening did not reach it: at `/Ox` `noinline` N=1 read `dtext` **56**
against a hand control of 16, and 48 of those 56 bytes are the callee
appearing in the section for the first time. Sliced to P's own bytes it reads
**8 against 16** and the flag fires. The fix is to take P's `[start, next
function)` range out of the section, and it is now in both scripts.

**Graded before it was kept**, per §6.14's own precedent, on all 170 families
in both modes:

| | `/O1` | `/Ox` |
|---|---|---|
| verdict lines moved | **0** | **0** |
| `controls failed` / `refuting LAW L'` | 0 / 0, unchanged | 0 / 0, unchanged |
| `INLINE-DECLINED?` flags | 92 → **92** (output byte-identical) | 139 → **160** |
| `TEXT-IDENTICAL` rows | 1042 → **1042** | 213 → **219** |

`/O1` is byte-for-byte unchanged, which is the control: `/O1` implies `/Gy`, so
there the slice is a no-op and any movement would have meant a broken fix. At
`/Ox` the flag count jumps by **21** — twenty-one rows where the front end had
declined an inline and this document's instrument said it had not — and the
before-run reproduces §6.14's recorded 139 exactly, so the two runs are
comparable. No verdict changed, because §6's law is graded on label strides
and the declines it now sees were already being handled correctly for other
reasons; what changed is that the rows now say so.

> **`.text` growth was called "the *only* column that carries depth evidence"
> in §6.4 and again in §6.12, and it is the column that was wrong.** The
> relocation table carries the same evidence exactly, by name, and cannot be
> fooled by a section boundary.

### 6.15.1 The decision is ALL-OR-NOTHING, per (caller, callee)

Fourteen ladders, both modes, N swept to 12 throughout and to **24** on the
ladder that pins the boundaries — **449 rungs, 2 904 objects, not one of them
mixed.** The front end never inlines the first K sites and declines the rest.
When it declines, `.text` for P collapses to a call sequence and *every* site
keeps its `bl` — `d2-loop-3loc` at `/O1` goes from 332 bytes of inlined P at
N=5 to 72 bytes and six `bl`s at N=6.

This **refutes the depleting-budget reading** that "inlines to N=4, declines at
N=5" invites, and which was pre-registered here at p≈0.65
(`work/gt-inline-decline/ESTIMATE-round28.txt`). There is no depletion. The
front end decides once, for the pair, knowing N.

### 6.15.2 `/O1` — the axis is the callee's OWN emitted size, and it is NOT the charge axis

`Nfull`, the largest N at which every site is inlined, is a function of exactly
one variable: **`s`, the direct callee's own emitted `.text` size**. §6.5
guarantees `s` is in every obj for free, because c2 emits the callee's COMDAT
whether or not it inlined it.

Fourteen ladders move `s` by **five independent mechanisms** — rungs of one
instruction, one-statement integer arithmetic, calls, `if` statements, and
`double` arithmetic with an FPR frame and `_fltused` — at depth 1 and depth 2,
with and without a loop. **Zero disagreements at every shared value of `s`.**
Three of the five were pre-registered as hold-outs, `d1-dbl` at p≈0.45 and
`d1-if` at p≈0.5, and both landed.

Two negatives make the result mean something:

* **It is not the per-site expansion size.** `s=72, g=36` gives 6+ sites and
  `s=80, g=36` gives 5, where `g` is the bytes P actually gains per site. The
  front end prices the callee's own body, not what lands at the site.
* **It is not statements.** `d2-arith` and `d2-call` at k=2 have the same
  statement count and different `s`, and give 5 and 4.

And the finding that matters most for this document:

> **Dead locals move the decline by ZERO.** Twenty rungs across `d1-deadloc`
> and `d2-deadloc` add a declared local that generates no code; `s` does not
> move one byte and neither does `Nfull`. Law L′ charges each of those locals a
> **full `E` unit** (`loc1-dead` = 4 against `loc0` = 3). **The axis the
> counter charges on and the axis the inliner declines on are different
> axes**, and an emitter that assumes one predicts the other is wrong on the
> cheapest possible probe.

### 6.15.3 The schedule — measured exactly, and generated by no formula

Boundaries pinned to a single 4-byte step by `d1-fine`, whose rungs are one
instruction each, and reproduced by the other thirteen ladders:

| `s` (bytes) | instructions | sites inlined |
|---:|---:|---:|
| ≤ 64 | ≤ 16 | **unbounded** (≥ 24 measured) |
| 68 – 72 | 17 – 18 | 9 |
| 76 | 19 | 7 |
| 80 | 20 | 5 |
| 84 – 88 | 21 – 22 | 4 |
| 92 – 100 | 23 – 25 | 3 |
| 104 – 140 | 26 – 35 | 2 |
| 144 – 256 | 36 – 64 | 1 |
| ≥ 260 | ≥ 65 | **0 — never inlined, not even once** |

Both ends are round numbers in instructions: a callee of **16 instructions or
fewer is inlined at any number of sites**, and one of **65 or more is never
inlined at all**.

> **What this buys, concretely.** §6.1 records the inline charge as *latent*
> in this port: `crates/c2-il/src/func/bundle.rs` refuses any TU where a callee
> is also defined, and the first rung to relax that gate has to know which tree
> it is counting for. The schedule turns that from a guess into a construction.
> A fixture whose callee is **≤ 64 bytes and loop-free** is inlined at every
> site in **both** modes and exercises law L′ at a known depth; one at **≥ 260
> bytes** is never inlined at `/O1` and exercises the un-inlined path with the
> callee still emitted (§6.5). Both ends are stable enough to build a corpus
> on, and the middle of the table is where a fixture will silently measure a
> different tree than its author intended — which is exactly what happened to
> §6.14's `d2-loop-3loc`.

**There is no closed form, and this is the second two-parameter fit this lane
has watched die.** `(N−1)·(s−64) < 80` — "the first copy is free; spend at most
20 instructions of net duplication per pair" — is **exact on every cell of the
entire fourteen-ladder dataset at N ≤ 6**, which is where the sweep's own cap
had always stopped. It was written down as LAW D with its falsifier column
(`work/gt-inline-decline/ESTIMATE-round28d.txt`, p≈0.55) and **killed by the
first capture above the cap**: it predicts ≥12 sites at `s=68` where 9 are
taken, and 10 at `s=72` where 9 are taken. The refutation does not rest on one
row — `s=68` is `d1-fine` alone (only 4-byte rungs reach it), but **`s=72` → 9
is reproduced by five ladders**, `d1-fine`, `d1-if`, `d1-noloop-arith`,
`d1-noloop-call` and `d2-noloop-arith`. The retired wording is in the
script's `SUPERSEDED_D` and is re-derived from each run's own numbers.

What the table refutes, computed rather than asserted:

| rival | killed by |
|---|---|
| `N·(s−h) ≤ B`, any affine cost | the `s=68`/`s=76` pair forces `h < 49.3`; the `s=72`/`s=80` pair forces `h > 56` |
| any single-tier cost model | `s ≤ 64` is **unbounded**, so the cost there would have to be ≤ 0 |
| a threshold on P's final size | 372 bytes accepted (`s=72`, N=9) against 252 refused (`s=104`, N=3) |
| a threshold on P's growth | 288 accepted against 136 refused |
| a threshold on the growth *ratio* | 4.43× accepted against 2.17× refused |
| `(N−1)·(s−64) < 80` (LAW D) | `s=68` → 9, not ≥12; `s=72` → 9, not 10 |

Note the shape of the first row's kill: **`N=10` is refused at `s=68` while
`N=9` is accepted at `s=72`** — a strictly *smaller* total accepted nowhere and
a larger one accepted. That is an arithmetic impossibility for any product
model, in the same sense as §6.12's "there is no integer `E` that reaches 10",
not a poor fit.

So the honest statement is: **the axis is found and the schedule is exact; the
rule that generates the schedule is `NOT MODELLED`.** The table is what a
fixture author or a corpus builder needs, and it is falsifiable — the script
prints `sched D n vs m <== *** REFUTES SCHEDULE D ***` on any row that
disagrees.

**Held out, and it lands.** The schedule was fitted entirely on `int` ladders.
Graded against twelve of this document's own families — destructors,
constructors, a depth-3 tree, a `switch` — it is exact on all twelve, and the
four §6.14 recorded a `/O1` budget for come out on the integer:

| family | direct callee `s` | schedule | §6.14 measured |
|---|---:|---:|---:|
| `d2-loop-asctor` | 68 | 9 | ≥6 ✓ |
| `d2-loop-3loc` | 80 | 5 | 5 ✓ |
| `d2-dtor-3loc` | 84 | 4 | 4 ✓ |
| `d2-dtor-loop`, `d3-dtor-loop` | 88 | 4 | 4 ✓ |

That also settles §6.14's *"`d2-loop-3loc` and `d2-loop-asctor` differ by one
declared local"*, which was **wrong**: the two inner bodies differ by a local
**and a statement and a call**, and the two direct callees differ by 12 bytes —
three instructions, two bands of the schedule apart.

### 6.15.3a The limit is per PAIR — P's existing expansion does not move it

Every ladder above puts exactly one callee in P, so none of them can say
whether the limit for one pair moves once the caller has already absorbed an
unrelated expansion. §6.12's `ptr-sibling` is the standing warning that it
might: there, a two-deep tree at one call site **removed a `+1` from an
unrelated site**, and that rule is a property of P's *whole* expansion.

Two callees in one P — `sba` sized to sit exactly at its limit (`s=80`, five
sites), `sbb` unrelated:

| `nA` | `nB` | `s(sbb)` | P `.text` | declined |
|---:|---:|---:|---:|---|
| 5 | 0 | — | 268 | — |
| 5 | 5 | 80 | **488** | — |
| 6 | 1 | 80 | 116 | `sba`×6 |
| 5 | 1 | 208 | 440 | — |
| **5** | **2** | **208** | 276 | **`sbb`×2 only** |

**The two pairs are decided independently.** `sba` takes its five sites with P
grown to 488 bytes beside it and loses all six at `nA=6` with or without a
sibling; `sbb` at `s=208` takes exactly the one site its own row of the
schedule allows and is refused at two — **in the same object where `sba` is
fully inlined**. There is no shared pot.

And it is not P's *own* size either. Padding P with up to **40 statements of
its own** — P's `.text` from 268 to 428 bytes at the same five sites — leaves
the limit at exactly 5, declining at 6 in every one of the four rungs
(`--padp`). So the number of sites the front end will take is a function of
**the callee alone**: not of what P already contains, not of what P has
already inlined, not of how big P is.

That **refutes this round's own pre-registered PRED M3** ("the threshold
depends on what has already been inlined into P", p≈0.7, from `ptr-sibling`).
The prior had already been revised down to p≈0.3 *before* the capture
(`ESTIMATE-round28d.txt`), on the grounds that the schedule turned out to
depend on one callee's size and nothing else — so the swing is on the record
rather than reconstructed after it. `/Ox` says the same: independent verdicts,
`sba` inlined at every `nA`, `sbb` refused at every `nB`.

### 6.15.4 `/Ox` — a different mechanism, not a different constant

Characterized separately throughout, per this lane's rule, and it is not a
rescaling of `/O1`:

* **No N-dependence whatsoever.** Fourteen ladders, N swept to 12: every row
  is *all* or *none*. There is no `/Ox` counterpart to the schedule above.
* **Loop-free callees: one sharp threshold**, on the callee's size *as emitted
  at `/O1`* — **≤ 108 bytes (27 instructions) inlined, ≥ 112 (28) declined** —
  confirmed on five ladders (`d1-fine`, `d1-noloop-arith`, `d1-noloop-call`,
  `d1-if`, `d1-dbl`), three of them held out. The **nine loop-free cases** of
  §6.15.5 — constructors, member functions and pointer stores, all between 24
  and 108 bytes — are inlined at `/Ox` without exception, which is a tenth
  agreement on a shape family the threshold was not fitted to.
* **The `/Ox`-emitted size does not decide it**, and the refutation is as fine
  as it can be: `d1-fine` k=15 and k=16 both emit a **112-byte** callee at
  `/Ox` and get opposite verdicts. `/Ox` is `/Ot`; the standalone callee is
  unrolled, and the front end's estimate is not recoverable from those bytes.
* **Callees containing a loop are declined far earlier.** `d1-arith`,
  `d2-arith` and `d2-liveloc` accept at 72 bytes of `/O1` size and refuse at
  80; `d1-call` and `d2-call` accept at 68 and refuse at 76 — against 108–112
  for loop-free, and **no size measured in either mode unifies the two
  groups.** `NOT MODELLED`.

### 6.15.5 The categorical refusals — §6.14's "it is the conjunction" is refuted

Some callees are refused **at N=1**, at sizes the schedule says should take
several sites. Those are not the budget. Twenty-two one-off cases, run in both
modes (`--cases`), with the refused callee's own `s` printed so a categorical
refusal cannot be confused with a schedule row.

§6.14 concluded from two cells that the `ctor-loop` refusal is *"the
conjunction"* of a constructor and a call inside a loop. Filling the 2×2
**refutes that in both directions at `/O1`**:

(`s` is the **innermost** callee's own size — the one the front end refuses —
so the rows are comparable; in every case the outer wrapper was inlined.)

| body | `/O1` | `s` |
|---|---|---:|
| ctor, loop, call, store to `this->v` in the loop (`ctor-loop`) | **declined** | 80 |
| …the identical body accumulating into a **local** (`ctor-loop-local`) | **inlined** | 76 |
| …the identical body as a **member function** (`method-loop-call`) | **inlined** | 76 |
| member fn, **no loop**, two stores to `this->v` with a call between | **declined** | 84 |
| …the same through an `int*` instead of `this` (`ptr-2store-call`) | **inlined** | 84 |
| ctor, one store, two calls (`ctor-1store-2call`) | **inlined** | 60 |

Two of the 22 cases show the two mechanisms coexisting cleanly, which is the
check that the separation is real rather than a reading: `glob-loop-call` and
`ptr-store-noloop` both have an 80-byte direct callee and both take **exactly
5 sites** — the schedule's row for `s=80`, not a categorical refusal, on
shapes (a static global written in a loop, a pointer written between calls)
that look like the refused ones.

`method-2store-call` is refused with **no constructor and no loop**, and
`ctor-loop-local` is accepted **with both**. Three pairs differing in exactly
one source feature give opposite verdicts, and they do not agree on which
feature matters. **No single-feature rule survives; the `/O1` categorical
refusal is `NOT MODELLED`**, and the 22 cells are in the script so the next
attempt starts from measurements rather than from a story.

**`/Ox` is cleaner, and has two separable triggers** — which is exactly why the
2×2 was worth filling:

* **(A) a store to memory inside a loop** — refused through `this`, through a
  pointer parameter, or to a static global, and refused even when the loop
  makes **no call**;
* **(B) a constructor whose body contains a loop at all** — `ctor-loop-local`
  and `ctor-loop-nostore` are refused while the byte-identical **member
  functions** `method-loop-local` and `method-loop-nostore` are **inlined**.

Controls that inline at `/Ox`: a ctor with a call and no loop; stores through a
pointer with calls between and no loop; every straight-line two-store shape.

### 6.15.6 The non-monotonicity, now twelve more instances

§6.14 recorded four cases where the budget runs the *other* way with the
optimisation level and concluded "there is no monotone reading". One 22-case
table adds **twelve more**, in both directions:

* `/O1` refuses what `/Ox` takes — `member-noloop-store`, `ctor-2store-call`,
  `ctor-2mem-call`, `method-2store-call` (4);
* `/Ox` refuses what `/O1` takes — `ctor-loop-local`, `ctor-loop-leaf`,
  `ctor-loop-nostore`, `method-loop-nostore`, `method-loop-call`,
  `ptr-loop-call`, `ptr-loop-store-nocall`, `glob-loop-call` (8).

And §6.14's own budget table needs one correction the old detector could not
have made: **`d2-loop-3loc` at `/Ox` is declined from N=1, not "from N=2", and
the instance refused is the INNER callee `lsa`** — the outer `lsb` is inlined
at every site. At `/O1` the same family refuses the **outer** one. Different
level, different mode, same source; only a symbol name can say so.

### 6.15.7 What round 28 leaves `NOT MODELLED`

* **The rule generating the `/O1` schedule.** The table is exact to a 4-byte
  step and reproduced by five mechanisms; six candidate closed forms are
  refuted above. A seventh should be checked against the table before it is
  believed, not against a ladder.
* **The `/Ox` threshold for callees containing a loop.** Loop-free is
  modelled; loops are declined far earlier and no measured size unifies them.
* **Every categorical refusal at `/O1`.** Twenty-two cells, three
  single-feature pairs disagreeing, no rule.

> **The riskiest thing still unmeasured** is that **`s` is a c2-side number
> standing in for a c1xx-side decision.** The front end chooses before register
> allocation; that the *emitted* size predicts its choice to a 4-byte step
> across fourteen ladders, five mechanisms and twelve of this document's own
> families is a strong empirical fact and **not a mechanism**. The place it has
> to break is a body where allocation moves the size a long way without moving
> the IL much — heavy spilling, many simultaneously live values, a large
> `__savegprlr_` pair — and **no such shape was probed**. That is also the one
> failure mode that would be invisible: such a callee would sit in the wrong
> row of the table and the table would look fine everywhere else. A ladder that
> grows the *live-value count* rather than the statement count, at roughly
> fixed statement count, is the probe, and it is cheap.
>
> Second: the schedule's own shape is unexplained. `≤ 16 instructions →
> unbounded` and `≥ 65 → never` are suspiciously round, and between them sits a
> sequence — 9, 7, 5, 4, 3, 2, 1 — that skips 8 and 6. A mechanism that
> produces exactly that is worth looking for, and finding it is what would turn
> §6.15.3 from a table into a law.
>
> Third, unchanged from §6.14: the whole of §6.15 is `int`, `double` and small
> structs, and every ladder callee has the same `int f(int)` signature. A
> **virtual** call the front end devirtualises and a **template** instantiation
> still have no row anywhere in this document.

### 6.15.7a The pre-registration, scored

Written before each capture and reproduced here because `work/` is gitignored
and a prediction nobody can check later is not a prediction. Twenty-five
registered, **sixteen landed and nine missed**, which is the useful ratio.

| prediction | p | outcome |
|---|---:|---|
| relocations catch a decline `dtext` misses | 0.5 | ✓ three `/Ox` N=1 rows, `noinline` among them |
| the budget depletes: sites 1..K inlined, K+1..N not | 0.65 | ✗ **no mixed row in 2 904 objects** |
| `/Ox`'s "declined from N=2" rows are a K=1 depletion | 0.6 | ✗ declined from N=1; `/Ox` has no N-dependence |
| the limit depends on what P has already inlined | 0.7 | ✗ and P's own size does not move it either |
| §6.14's "one declared local" wording is wrong | 0.75 | ✓ |
| **the ladders will NOT all flip at the same callee size** | **0.7** | ✗ **at `/O1` all fourteen do** |
| the decline axis is not the charge axis | 0.6 | ✓ dead locals move it by zero |
| `N_max × per-site growth` ≈ constant | 0.45 | ✗ |
| MODEL B: `N·(c₀+wk) ≤ 20`, fitted on `d2-arith` | 0.55 | ✗ dead on the first held-out ladder |
| `ctor-arith` refused at every rung | 0.85 | ✓ |
| `ctor-leaf-arith` behaves like an ordinary ladder | 0.8 | ✗ it is categorical too, from k=3 |
| `d1-fine` reproduces the curve at 4-byte resolution | 0.7 | ✓ |
| `d1-dbl` (FP opcodes, FPR frame, `_fltused`) lands on it | 0.45 | ✓ |
| `d1-if` (branches, basic blocks) lands on it | 0.5 | ✓ |
| `/Ox` is 6-or-0 everywhere | 0.9 | ✓ and still 12-or-0 at N=12 |
| `ctor-loop-local` inlines — "it is aliasing, not C++" | 0.5 | ✓ at `/O1`, ✗ at `/Ox` |
| `method-loop-call` is declined | 0.5 | ✗ at `/O1`, ✓ at `/Ox` |
| `ptr-loop-call` is declined | 0.4 | ✗ at `/O1`, ✓ at `/Ox` |
| `member-noloop-store` inlines | 0.85 | ✗ at `/O1`, ✓ at `/Ox` |
| LAW D: `(N−1)(s−64) < 80`, read 10 / 7 / 5 out of sample | 0.55 | ✗ **9** / 7 / 5 |
| no mixed row up to N=12 | 0.8 | ✓ |
| the doc's own ctor/dtor/depth-3 families reproduce | 0.5 | ✓ all twelve |
| the `/Ox` loop split survives N=12 | 0.85 | ✓ |
| a sibling callee does not move the limit | 0.7 | ✓ |
| padding P's own body does not move it | 0.85 | ✓ |

**The named bias fired, and in the direction I flagged.** Round 28's estimate
opened by naming *"the budget is chaotic, NOT MODELLED"* as the cheap answer
that lets me stop early, and pre-committed against it. The single biggest miss
in the table is exactly that prime: `/O1`'s axis is **cleaner** than I gave it
a 30 % chance of being. §6.14's opposite lesson — that expecting a refutation
because the last six probes produced one is the same bias with the sign
flipped — has a converse too, and this is it: after reading a section that
says *"non-monotone four times over"*, I priced order too low.

I was also **right for the wrong reason at `/Ox`**. "The ladders will not all
flip at the same size" is true there, and my stated reason — that emitted size
is a c2-side proxy for a c1xx-side decision — is not why. The reason is that
`/Ox` is `/Ot` and unrolls the standalone callee, which is a fact about the
back end, not about the proxy. A correct prediction from a wrong model is
worth less than a miss from a stated one, and it is recorded that way.

### 6.15.8 Reproduction

```sh
export C2RS_WIBO=<the repo's resolved wibo>
scripts/gt_inline_decline.py --list
# the axis, and the five mechanisms that agree on it
scripts/gt_inline_decline.py --max 6                       # all 14 ladders, /O1
scripts/gt_inline_decline.py --max 6 --mode '/Ox /GS- /c'  # …and packed
# the boundaries, at 4-byte resolution — this is the row that pins the table
scripts/gt_inline_decline.py --max 12 d1-fine
scripts/gt_inline_decline.py --max 2  d1-fine              # the >=260 ceiling
# the charge axis is not the decline axis: 20 rungs, zero movement
scripts/gt_inline_decline.py --max 6 d1-deadloc d2-deadloc
# the categorical refusals, both modes
scripts/gt_inline_decline.py --cases
scripts/gt_inline_decline.py --cases --mode '/Ox /GS- /c'
# HELD OUT: this document's own ctor/dtor/depth-3 families, graded
scripts/gt_inline_decline.py --max 8 --family d2-loop-3loc \
    --family d2-dtor-3loc --family d2-dtor-loop --family d3-dtor-loop
# the two detectors side by side; `noinline` at /Ox is the one that mattered
scripts/gt_inline_decline.py --mode '/Ox /GS- /c' --family noinline
# per-PAIR, not per-caller: two callees in one P, decided independently
scripts/gt_inline_decline.py --sibling
scripts/gt_inline_decline.py --sibling --mode '/Ox /GS- /c'
# …nor does the CALLER's own size: P padded with 40 of its own statements
scripts/gt_inline_decline.py --padp
```

Reading the columns: `Nfull` is the largest N at which the **whole** planted
tree was inlined and `Ndir` the largest at which the **direct** callee was.
Schedule D is a claim about one (caller, callee) pair, so it is graded on
`Ndir`; a row where the two differ prints `INNER-DECLINED (a different pair)`
and is a categorical refusal of something deeper — the `ctor-*` ladders are
entirely of that kind, wrapper inlined at all twelve sites, constructor
refused at all twelve. Grading those against the schedule produced nine false
`REFUTES` lines before the split was added, and a falsifier that cries wolf is
worse than none.

**Current score of the shipped falsifier** *(as of round 28; superseded by
§6.16.13, which re-gates the same falsifier over 26 ladders after round 29
added nine)*, all 17 ladders at `--max 12` in both modes: `captures failed: 0`, **0 rows refuting SCHEDULE D**, with 9
`INNER-DECLINED` rows at `/O1` and 49 at `/Ox`. That second number is the one
to read alongside the first: it is the categorical refusals being attributed
to the pair that was actually refused instead of to the pair the schedule is
about, and `/Ox` having five times as many is §6.15.4 and §6.15.5 showing up
in the same column.

A row that genuinely disagrees with the table prints
`<== *** REFUTES SCHEDULE D ***`, and the readings this round retired are
re-derived from the run's own numbers beside the verdict:

```
d2-loop-3loc -> direct callee lsb  s=80  Ndirect=5  sched D 5 OK
               [retired 'N*(s-64) < 80' said 4]
```

## 6.16 Round 29 — the register-pressure probe: `s` is the axis, not a proxy

§6.15.7 closed by naming the riskiest thing left unmeasured, and named the
probe for it:

> **`s` is a c2-side number standing in for a c1xx-side decision.** … The
> place it has to break is a body where allocation moves the size a long way
> without moving the IL much … and **no such shape was probed**. That is also
> the one failure mode that would be invisible: such a callee would sit in the
> wrong row of the table and the table would look fine everywhere else.

This round builds it — **231 rungs, 2 837 objects**, in both modes. The
outcome is a **confirmation**, which this lane normally rates below a
refutation, but this one is worth more than the 449 rungs it checks, for a
reason worth stating before the numbers.

### 6.16.0 Why the existing 449 rungs could not have answered this

All fourteen ladders of §6.15 move `s` by **appending statements**. So in
every one of their cells `s` and any source-side count the front end could
hold — statements, expressions, IL nodes — move **together**. The design is
perfectly confounded, and 449 agreeing rungs of it are *silent* on which of
the two the front end reads. Breadth does not fix a confound; only a probe
that moves one and holds the other does.

That is why the schedule was shipped in §6.15.3 as an exact table with an
explicitly `NOT MODELLED` mechanism, and why this probe was named rather than
assumed away.

### 6.16.1 The design: a matched permutation pair

Two bodies that are **permutations of one multiset of statements**:

```c
LOW   int t0=gs(a+1); v=gs(v^t0);  int t1=gs(a+2); v=gs(v^t1);  …
HIGH  int t0=gs(a+1); int t1=gs(a+2); … ;  v=gs(v^t_{k-1}); … ; v=gs(v^t0);
```

Same `2k` statements, same `k` declarations, same `2k` calls, same operators,
at every `k`. **Every source-side count is equal by construction.** What
differs is that `LOW` kills each temp at the very next statement while `HIGH`
holds all `k` live across `k` calls — measured `nsave` of **1** against
**k+1** at every `k ≥ 2` (at `k = 1` the two source texts are identical, and
the row prints `INERT`).

The use is an **opaque extern call**, deliberately. Nothing algebraic connects
a use to its def, so no re-association can move it, and calls to an extern
cannot be reordered among themselves, so the def sequence is pinned too.

### 6.16.2 The first spelling was INERT, and an inert probe looks like a passing one

The pre-registration (reproduced in §6.16.12) specified the use as
`v=(v<<1)^t`, called it *"non-associative"*, and was **wrong**: that chain is
**linear over xor** — `((v<<1)^t0)<<1)^t1` is exactly `v<<2 ^ t0<<1 ^ t1` — so
the compiler re-associates it freely. The two orders compile **byte-identically
at every k**, and to the *high*-pressure schedule (`nsave = k+1`) in both: the
compiler chooses to keep `k` values live even where the source did not ask it
to. (Which component does it is not observable here; see §6.16.5a.)

That null is kept in the instrument as `d1-perm-lo` / `d1-perm-hi`, because of
what it would have looked like if it had not been checked:

> **An inert probe and a passing probe print the same thing.** Had the run
> reported only "no row refutes SCHEDULE D", that reads as a confirmation of
> the table, while in fact it is a report that the experiment never happened.
> `--pressure` therefore counts and prints **discriminating cells** and
> **inert rows** separately, and says in plain words `NO DISCRIMINATING CELL —
> this run says NOTHING about which axis is real` when there are none. This is
> the same class of error as §6.15.0's polluted `dtext` and §6.15.8's
> crying-wolf falsifier: the instrument agreeing with you is not evidence
> until you know it *could* have disagreed.

### 6.16.3 The divergence is 100% frame idiom — every byte of it

With the opaque use, the pair separates by exactly **−24 bytes at every k ≥ 2**
(`HIGH` is the *smaller* one), and the `body` column — bytes strictly between
the frame push and the frame pop — is **identical at every single k**:

```
k   s_lo   s_hi   ds    body lo/hi  Nlo  Nhi  press_lo    press_hi
1   72     72     0     28/28       9    9    1/112/0+1   1/112/0+1   INERT (same text)
2   92     68     -24   48/48       3    9    1/112/0+1   3/112/0+0
3   112    88     -24   68/68       2    4    1/112/0+1   4/128/0+0
4   132    108    -24   88/88       2    2    1/112/0+1   5/128/0+0
5   152    128    -24   108/108     1    2    1/112/0+1   6/144/0+0
…
11  272    248    -24   228/228     0    1    1/112/0+1   12/192/0+0
12  292    268    -24   248/248     0    0    1/112/0+1   13/192/0+0
```

**rows where the body sizes differ: 0.** The whole 24 bytes is prologue and
epilogue, and the mechanism is visible in the disassembly of the `k=2` pair —
both bodies are 48 bytes, and the frame code is not:

| | prologue | epilogue | frame code total |
|---|---:|---:|---:|
| LOW, 2 saved nonvolatiles, **inline** (`mflr`, `stw r12,-8(r1)`, `std 30`, `std 31`, `stwu` … `addi r1`, `lwz r12`, `mtlr`, `ld 30`, `ld 31`, `blr`) | 20 B | 24 B | **44 B** |
| HIGH, 3 saved nonvolatiles, **out-of-line helper pair** (`mflr`, `bl __savegprlr_29`, `stwu` … `addi r1`, `b __restgprlr_29`) | 12 B | 8 B | **20 B** |

Crossing from two saved nonvolatiles to three *removes* six instructions.

That is as clean as this question gets: a **register-allocator idiom
threshold**, six instructions wide, in code that does not exist until after
allocation, with every source-side count held fixed. What that implies about
*which component* is deciding is §6.16.5a.

### 6.16.4 The result: seven discriminating cells, zero refutations

A cell is **discriminating** when the pair straddles a schedule band. The
falsifier is one row where it straddles and `Nfull` comes out the same anyway.

| pair | discriminating cells | refuting rows |
|---|---:|---:|
| depth 1, opaque use | 4 | **0** |
| depth 2, opaque use | 3 | **0** |
| depth 1, re-associable use | 0 (all inert) | 0 |

The headline cell is `k=2`: **four statements — two declarations and two
assignments — and four calls, the identical multiset in both spellings, and
the front end takes 3 sites of the low-pressure one and 9 of the
high-pressure one.** `s` moved from 92 to 68 and
`Nfull` moved from 3 to 9, exactly as §6.15.3 tabulates both.

And `k=11` moves the **ceiling** itself: `s_lo=272` is never inlined, `s_hi=248`
is inlined once. The `≥260 → 0` boundary is crossed by nothing but a choice of
save/restore idiom.

> **`s` is not a proxy that happens to correlate.** It moves with every
> source-side count held constant, and the decision moves with it, to the
> tabulated row, at depth 1 and depth 2. This is the statement §6.15.3 could
> not make and the reason the table can now be used to construct fixtures
> rather than merely to describe measurements.

### 6.16.5 The direction is the opposite of the intuition that motivated the probe

§6.15.7 expected pressure to make a callee **bigger** ("heavy spilling … a
large `__savegprlr_` pair"). It makes it **smaller**: more simultaneously live
values crosses the helper-pair threshold and *saves* six instructions, so a
higher-pressure callee is inlined at **more** sites, not fewer. The prediction
that a `__savegprlr_` pair inflates the callee is **refuted** — the helper is
one instruction at every width. Across `d1-live-hi`, `nsave` runs 3 → 13 while
`s` moves in a straight line of +20 bytes per rung: **the save set's width
costs `s` exactly zero.**

### 6.16.5a This strains the premise §6.15.7 was written on

§6.15.7 states the risk as *"the front end chooses **before register
allocation**; that the emitted size predicts its choice … is a strong
empirical fact and **not a mechanism**."* The measurement above makes that
premise hard to hold:

> **The decision tracks a quantity that does not exist until after register
> allocation.** The three-nonvolatile save/restore idiom threshold is a
> register-allocator output. A chooser running before allocation cannot read
> it. Yet the decision moves with it, in both directions, at both depths, and
> at `/Ox` as well.

Two readings survive:

* **(A)** the decision is not the *front* end's at all. In the MSVC split
  `c1xx` parses and emits IL and **`c2` optimises and generates code**, so
  inlining is `c2`'s job — and `c2` can compile the callee first (§6.5: its
  COMDAT is emitted whether or not it was inlined), **measure** it, and then
  decide on the caller. On this reading `s` is not a proxy for the deciding
  quantity, it **is** the deciding quantity, which is what a schedule exact to
  a 4-byte step looks like.
* **(B)** the decision is `c1xx`'s and its own estimate happens to track the
  helper-pair threshold. That requires a coincidence at 4-byte resolution
  between two mechanisms with no shared input.

**(A) is far more economical, and this document should stop asserting (B).**
But (A) is a **hypothesis, not a measurement**, and is written here as one —
nothing in this lane has looked inside either binary, and "which component
decides" is not a question a differential capture can answer on its own.

One cheap consequence *was* tested. If the decider measures a compiled callee,
it needs the callee compiled first — so move the **definition** after the
caller, behind a forward declaration:

| `k` | `s` defined before | `s` defined after | sites before | sites after |
|---:|---:|---:|---:|---:|
| 1 | 72 | 72 | 9 | 9 |
| 2 | 92 | 92 | 3 | 3 |
| 3 | 112 | 112 | 2 | 2 |
| 5 | 152 | 152 | 1 | 1 |
| 8 | 212 | 212 | 1 | 1 |

**Definition order moves nothing** — not the schedule and not `s` itself, so
the comparison is not confounded. That is what a two-pass back end looks like,
and it rules out the naive single-forward-pass version of (A) without
distinguishing (A) from (B). `scripts/gt_inline_decline.py --order`.

### 6.16.6 The two round ends, under pressure

These are the ends a fixture author *constructs* with, so they were re-checked
with liveness rather than statements carrying the size:

| body | `s` | allocator state | sites | schedule |
|---|---:|---|---:|---|
| 3 call-defined values, all live at once | 60 | `nsave=3`, takes the helper | **24 / 24** | unbounded ✓ |
| 4 call-defined values, all live at once | 76 | `nsave=4` | **7** | 7 ✓ |
| 20 call-defined values, all live at once | 344 | `nsave=18`, **2 spill stores + 2 reloads** | **0** | 0 ✓ |
| 24 call-defined values, all live at once | 424 | `nsave=18`, **6 spill stores + 6 reloads** | **0** | 0 ✓ |

Both ends hold with real spill code in the callee. The 4-live row is a
**pre-registration miss** worth keeping: it was written down as "still ≤64 B,
the control that says the floor is a size and not 'small bodies always
inline'", and it is **76 bytes** — liveness alone carries a body over the floor
— so it became a held-out cell for the `76 → 7` row instead, which it hit
exactly.

### 6.16.7 The spill floor: a spilling callee cannot sit in the wrong row

The named risk was that spilling puts a callee in the **wrong** row. Scanning
the live-value count directly:

| values live | `s` | `nsave` | spill stores | schedule row |
|---:|---:|---:|---:|---:|
| 15 | 252 | 15 | 0 | 1 |
| 18 | 300 | **18** | 0 | 0 |
| **19** | **324** | 18 | **1** | 0 |
| 24 | 424 | 18 | 6 | 0 |

`nsave` saturates at **18** — every nonvolatile GPR, r14..r31 — and the first
byte of actual spill code appears at **19 live values and 324 bytes**.

> **At `/O1` a callee that genuinely spills is arithmetically confined to the
> never-inlined row.** Nineteen simultaneously live values cost at least one
> def and one use instruction each before any spill code exists, which lands
> the body **64 bytes past the 260-byte ceiling** — measured, not argued. There
> is no `s` at which a spilling callee could be in the *wrong* row, because
> there is only one row it can occupy.
>
> The risk was real but it was aimed one mechanism too far. The allocator
> artifact that *does* land inside the schedule's live range is far cheaper
> than spilling — the **three-nonvolatile save/restore idiom threshold** of
> §6.16.3 — and the schedule prices it correctly.

### 6.16.8 `/Ox`: §6.15.4's threshold is held out on this pair too, and it lands

§6.15.4's loop-free rule — **≤108 bytes of `/O1`-emitted size inlined, ≥112
declined** — was fitted entirely on statement-ladders. Graded on the pressure
pair (the runner captures the `/O1` reference size for the same source, since
the rule is stated on it):

**46 graded cells, 0 refutations, 2 discriminating.** The discriminating cells
are the sharp ones:

| `k` | spelling | `s` at `/O1` | `/Ox` predicted | `/Ox` measured |
|---:|---|---:|---|---|
| 3 | LOW | 112 | declined | **declined** |
| 3 | HIGH | 88 | inlined | **inlined** |
| 4 | LOW | 132 | declined | **declined** |
| 4 | HIGH | **108** | inlined | **inlined** |

Same source counts, opposite `/Ox` verdicts, and the `k=4` HIGH cell sits
**exactly on the 108-byte boundary** and inlines. So the allocator idiom moves
the `/Ox` decision as well, and §6.15.4's choice to state its threshold on the
`/O1`-emitted size rather than the `/Ox` one survives a mechanism it was not
fitted to.

### 6.16.9 The new grader cried wolf, in exactly the place §6.15.8 warned about

The `/Ox` grader above printed **six `*** REFUTES ***` lines** the first time
it ran, all on the depth-2 pair — and all false, for two compounding reasons:

* it charged an **inner** decline to the direct pair, which is precisely the
  failure §6.15.8 fixed for SCHEDULE D and re-introduced here from scratch;
* at `/Ox` the depth-2 wrapper collapses to an **8-byte tail-call thunk** and
  is inlined everywhere while `in2` — a different pair — is the one refused, so
  the `/O1` reference size is not a measurement of the function being graded
  at all.

Fixed, not fudged: the grade is now taken **per spelling** and **skipped** with
`not graded: INNER-DECLINED (a different pair)` wherever `Nfull != Ndirect`.
Six rows moved from a false alarm to an honest abstention; the twenty-four
depth-1 cells were unaffected and are what the claim rests on.

> This is the **fourth** time this document's instrument has misled, and the
> second time on this exact fault line. The lesson is not "be careful" — it is
> that the inner/outer distinction is not a detail of one grader but a property
> of every claim about a *pair*, and any new grader inherits it.

### 6.16.10 The cap the `unbounded` row still rested on — lifted

`≤64 B → unbounded` was never a measurement of *unbounded*; it was a
measurement of *at least 24*, and 24 is where the sweep stopped. **LAW D died
precisely because a cap was believed** (exact on every cell below the sweep's
own `N ≤ 6`, dead on the first capture above it), so the row was re-run to
**N = 64** — including on `s = 64` exactly, the band's own top boundary:

| `s` | sites inlined |
|---:|---:|
| 48 | 64 / 64 |
| 56 | 64 / 64 |
| **64** | **64 / 64** |

Not one `bl` survives at any of 64 sites. The floor is unbounded as far as
this instrument can reach, measured at the boundary rung rather than inside
the band.

### 6.16.10a The FPR frame class: the pair separates, and it still says nothing

The draft of §6.16.11 was about to ship *"every callee in this section is
`int f(int)`"* as the top remaining risk. Naming a gap is worth less than
closing it when closing it costs one generator, so the same permutation pair
was re-run with **`double` temps defined by `gd` calls** — an FPR frame,
`_fltused`, a `__savefpr_` set alongside the GPR one, and an entirely
different opcode mix, with the statement, declaration, call and operator
counts still equal between the spellings at every `k`.

It half worked, and the half that did not is the interesting half.

**The pair does separate, and in the opposite direction.** FPR saves are
emitted **inline, one `stfd` per register**, with no cheap out-of-line idiom to
cross, so more live `double`s makes the callee **bigger**, not smaller:

```
k   s_lo   s_hi   ds    body lo/hi  Nlo  Nhi  press_lo        press_hi
2   176    184    +8    132/132     1    1    1+1f/112/0+2    1+2f/128/0+2
3   228    244    +16   184/184     1    1    1+1f/112/0+2    1+3f/128/0+2
8   488    496    +8    444/444     0    0    1+1f/112/0+2    1+8f/176/0+2
```

`LOW` holds one live FPR at every `k`; `HIGH` holds `k`. The `body` column is
**identical at every k** again, so this separation is also 100% frame idiom.
So the *mechanism* generalises: a frame class the schedule was not fitted to
still moves `s` by pure allocation.

**But not one cell discriminates, and the instrument says so rather than
banking the agreement.** The cheapest FP callee this shape can build already
emits **116 bytes**, past the narrow region (68–100 B) where an 8–16 byte
delta could straddle a boundary; from 104 B up the bands are 36 and 112 bytes
wide and nothing this small crosses them. The run therefore prints:

```
discriminating cells: 0   refuting rows: 0   inert rows: 1
NO DISCRIMINATING CELL — the probe did not separate
axes and this run says NOTHING about which is real.
```

which is exactly the report §6.16.2 built that counter for. Sixteen FP rungs
of SCHEDULE D agreement is a **schedule** confirmation in a new frame class
and **zero** axis evidence, and those are different currencies.

> **The gap is narrowed, not closed.** SCHEDULE D now holds on a callee with
> an FPR frame and `_fltused`; the pure-allocation delta exists there too. What
> is still unmeasured is whether **`s` is the axis** in that frame class, and
> with this mechanism it is **not separable** — the FP idiom delta is too small
> and the FP callee too large for the two to meet at a band boundary.

### 6.16.11 What round 29 leaves `NOT MODELLED`

Unchanged from §6.15.7, and deliberately **not** narrowed:

* **The rule generating the `/O1` schedule.** Six closed forms refuted; no
  seventh is proposed here. Every cell this round produced *reproduces*
  §6.15.3's existing table, so a form fitted now would have **no hold-out at
  all** — the exact condition that killed LAW D. Lifting a cap (§6.16.10) was
  the available honest move and it was taken instead.
* **The `/Ox` threshold for callees containing a loop.** Untouched here; every
  pressure-pair callee is loop-free.
* **Every categorical refusal at `/O1`.** Untouched: 22 cells, no rule.

What round 29 **removes** from the risk list is the top item: `s` surviving the
allocator is now measured rather than hoped for, and the spill floor bounds the
one mechanism that could have hidden a wrong row.

> **The riskiest thing still unmeasured** is no longer the proxy — it is that
> the axis result rests on **one frame class**. Every discriminating cell in
> this round is GPR pressure in an `int f(int)` body, and §6.16.10a shows why
> that is not an oversight that one more ladder fixes: the FP pair **separates
> `s` and still cannot discriminate**, because the cheapest FP callee is
> already past the narrow bands. So the honest statement is not "the `double`
> case is untested" — it is tested and it is **uninformative on the axis**, and
> the same arithmetic will defeat any frame class whose minimum callee is
> large relative to its idiom delta.
>
> What would actually settle it is a shape with a **large** allocator delta at
> a **small** callee size. The GPR helper-pair threshold is the only one found
> so far that qualifies (24 bytes at 68–92 B). A **member function** — a `this`
> pointer live from entry, so one more live value before the first statement —
> is the cheapest untried candidate, and a **struct return** (hidden pointer
> parameter, sret) the next. Neither has been run.
>
> Unchanged and still third: a **virtual** call the front end devirtualises and
> a **template** instantiation have no row anywhere in this document.

### 6.16.12 The pre-registration, scored

Written before the captures, in `work/gt-inline-decline/ESTIMATE-round29.txt`
and reproduced here because `work/` is gitignored. Seventeen registered,
**fifteen landed, one missed, one vacuous** — in four tranches, each written
before its own capture, with each addendum naming what the previous result had
just made questionable. The vacuous one is kept in the table rather than
dropped: a prediction whose antecedent never occurred is not a hit.

| prediction | p | outcome |
|---|---:|---|
| S1 the permutation moves `s` at all | 0.85 | **✓ only after a redesign** — see below |
| S2 where the pair straddles a band, `Nfull` follows `s` | 0.60 | ✓ 7 / 7 cells |
| S3 at least one discriminating cell exists | 0.75 | ✓ seven |
| S4 no true spilling inside `68 ≤ s ≤ 256` | 0.80 | ✓ floor measured at 324 |
| S5 `≥65 instr → never` holds on spill bytes | 0.90 | ✓ at 344 and 424 |
| S6 `≤16 instr → unbounded` holds under max liveness | 0.90 | ✓ 24/24 at `s=60` |
| S7 the LOW ladder alone reproduces the schedule | 0.85 | ✓ 11 rungs |
| S8 `__savegprlr_` **width** moves the decline by zero | 0.85 | ✓ `nsave` 3→13, `s` linear |
| S9 depth 2 says the same as depth 1 | 0.85 | ✓ identical numbers |
| (unnumbered) the 4-live body is still ≤64 B | — | ✗ **76 B** |
| S10 `s = 64` still fully inlined at `N = 64` | 0.70 | ✓ |
| S11 `s = 48` likewise | 0.80 | ✓ |
| O1 definition order does not move the schedule | 0.70 | ✓ |
| O3 …nor `s` itself (the confound control) | 0.90 | ✓ |
| F1 the FP pair is not inert — `s` separates for FP too | 0.75 | ✓ +8 to +16 B |
| F2 where the FP pair straddles a band, SCHEDULE D holds | 0.65 | **vacuous — no FP cell straddles one** |
| F3 the FP separation is not 24 B, being a different idiom | 0.60 | ✓ and the opposite sign |

**The named bias, and what it actually did.** The estimate opened by naming
anchoring in *both* directions — 449 confirming rungs pulling one way, "the
brief hands me this as the invisible failure mode" pulling the other — and set
S2 at 0.60 while stating that un-anchored it would be 0.50. The confirmation
landed, so the anchor was not punished. That is not the same as the anchor
being harmless, and the honest reading is the one the estimate itself
pre-committed to: **the 449 rungs contributed nothing to this result and the
seven new cells carry all of it.**

**S1 is the entry that matters.** Its *outcome* landed and its *stated reason*
was wrong — the operator called "non-associative" in the estimate is linear
over xor, and the first probe measured nothing. §6.15.7a recorded the converse
case ("right for the wrong reason at `/Ox`") and rated it below a stated miss;
this is the same coin. What saved it was not insight but the estimate having
written down the inert failure mode as *"a real outcome and the first thing
the run must check"* — a pre-registered null check, doing the job a
pre-registered null check exists to do.

### 6.16.13 Reproduction

```sh
export C2RS_WIBO=<the repo's resolved wibo>
# THE PROBE: same statements, different allocation. Read the two counters at
# the bottom of each block — discriminating cells and inert rows.
scripts/gt_inline_decline.py --pressure --max 12
scripts/gt_inline_decline.py --pressure --max 12 --mode '/Ox /GS- /c'
# the two round ends under pressure, and the SPILL FLOOR scan
scripts/gt_inline_decline.py --ends
# the new ladders through the shipped falsifier
scripts/gt_inline_decline.py --max 12 d1-live-lo d1-live-hi d2-live-lo \
    d2-live-hi d1-perm-lo d1-perm-hi d1-cheap-hi
# the cap lift: `unbounded` measured at N=64, at s=64 exactly
scripts/gt_inline_decline.py --max 64 --kmax 2 d1-noloop-arith
# does the callee's DEFINITION ORDER move anything? (it does not)
scripts/gt_inline_decline.py --order --max 12
```

The fourth `--pressure` block is the FPR frame class of §6.16.10a; read its
`discriminating cells: 0` line before reading its `refuting rows: 0` one.

**Re-gate of the shipped falsifier, superseding §6.15.8's 17-ladder score.**
Round 29 adds nine ladders (`d1-live-lo/hi`, `d2-live-lo/hi`, `d1-perm-lo/hi`,
`d1-cheap-hi`, `d1-fp-lo/hi`), so the whole set was re-run end to end in both
modes:

| | `/O1` | `/Ox` |
|---|---:|---:|
| ladders | 26 | 26 |
| rungs | **344** | **344** |
| objects | 4 128 | 4 128 |
| `captures failed` | **0** | **0** |
| rows refuting SCHEDULE D | **0** | **0** |
| `INNER-DECLINED` rows | 9 (unchanged) | 59 (was 49) |

The `/O1` `INNER-DECLINED` count is **unchanged at 9**, which is the control: the
nine new ladders are depth-1 and depth-2 shapes whose direct callee is never
the refused one, so any movement there would have meant the split had broken.
The `/Ox` count rises by ten, all from the new depth-2 rows where the wrapper
collapses to a thunk (§6.16.9).

Every existing function in the script is **untouched** — the round-29 diff
removes exactly one line (`mode, nmax = …` gains a third name) and is otherwise
pure addition — so the pre-existing rows could not have moved, and the re-gate
confirms they did not.

> **The two background re-gates wrote to distinct paths and each capture
> process makes its own `mkdtemp`, so the shared-cache hazard does not apply
> here; the `mode:` header line was checked in both files before the numbers
> were read.** Worth recording separately: the *readiness check* lied. `pgrep -f
> gt_inline_decline.py` matched the waiting shell's own command line, so both
> runs read as still-running long after they had finished. A watcher that can
> match itself is a watcher that never fires.

`--pressure` prints `<== s TRACKS` on a discriminating cell that agrees,
`<== *** s IS NOT THE AXIS: same IL, ±n bytes, same Nfull ***` on one that
does not, and `INERT` where c2 collapsed the permutation. The `body lo/hi`
column is the load-bearing one: while it reads `x/x`, the whole of `ds` is
prologue and epilogue and the front end cannot have seen any of it.
