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
| a body the front end **inlined into** | **+3 / +8 / +13** for 1 / 2 / 3 inlined call sites of the same framed callee — +5 per site after the first | measured, **not modelled**; see below |
| whether the `/Gy` upfront surcharge is per function or per `.text` COMDAT | — | **still open**: it is an absolute-seed question and cancels out of every difference here. A `static` function that is *not* emitted (fully inlined, no COMDAT) is the discriminator |

The `do/while` row is the small print worth reading twice: "a loop costs 2" is
already a rule fitted to two of three loop forms.

**Inlining is the riskiest row, and it nearly got fitted wrong.** With a static
callee `int lst(int a){ return gs(a)+a; }` (framed, stride 5 on its own) inlined
into P, P's *total* strides are 8, 13 and 20 for one, two and three call sites —
deltas 3, 5, 7, which reads as quadratic. It is not: at three sites P has
crossed into Class C and 2 of that 7 is the `__savegprlr_29` pair, which
`scripts/gt_label_stride.py`'s `minted` column shows independently (5, 5, 7).
Net of it the surcharge is **3, 8, 13** — linear at +5 per additional site,
which is one framed function's worth per inlined body. A marked `inline` free
function behaves identically to a `static` one (13 at two sites).

Three data points, one callee class, and the first site's 3 unexplained, so it
is recorded and not claimed. It matters more than anything else in this table
because **real workload TUs are inlined into constantly**, and a framed function
downstream of an inlined body gets the wrong `$M` the moment the class gate
admits one.

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
