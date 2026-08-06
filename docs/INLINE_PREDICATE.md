# INLINE_PREDICATE — when c2 does not emit the call the IL contains

> ## Status, 2026-08-08 — **MECHANISM I's BYTES ARE SHIPPED TOO. What is still NOT shipped from this page is the *decision rule* — §2's cost model — and it must not be.**
>
> Lane `w-splice` ([`rungs/2026-08-08-w-splice.md`](rungs/2026-08-08-w-splice.md))
> built §2's *bytes* into the port as `crates/c2-core/src/splice.rs`:
> **SPLICE-0-PORT**, the rule that a caller whose whole emitted body is one call
> to a same-TU callee the port lowers emits **the callee's body**. On the 878-TU
> workload `fnbyte-differs` went **3,195 → 2,472**: **723 emitted functions moved
> `differs → exact` and zero moved the other way**, per `(TU, emit_name)`. Board
> rows **#990**–**#995**, **#1006**–**#1009**.
>
> **Three things it did NOT take, and each is load-bearing:**
>
> 1. **It did not take §2's decision rule.** `INLINE-P` is 0.9716 on a 9,993-callee
>    hold-out with a 2.84 % residual §7 leaves NOT MODELLED, and 3 % of a guess is
>    a wrong emit. What the shipped clause consults is the **region where that rule
>    is categorical** — `s ≤ 64`, where `N_max` is UNBOUNDED in *both* linkage
>    classes and therefore independent of linkage, of `inline`, of `nparams`, of
>    the site count and of §5's unreadable `leaf` bit. That bound is a different
>    object from the rule, and it **never binds on today's port** (#1008).
> 2. **It did not take the whole of what SPLICE-0 predicts.** `w-seq` priced 1,967
>    and the shippable subset at 726; the shipped number is **723**, and the
>    missing 3 are named (`S6-chain-open`, #991) rather than widened away.
> 3. **It did not narrow `IlBundle::functions()`.** §6 item 4's standing hazard is
>    intact; `mismatch` is still 0 and none of the 723 has reached an obj.
>
> **And it found that mechanism I is a FIXPOINT, like E** (#1009): c2's body for a
> caller two links above a lowerable callee is the *end's* body, measured on a
> compiled cell and again on **150 workload relocation witnesses**. See §2's
> update block.
>
> ## Status, 2026-08-07 — **MECHANISM E IS SHIPPED. MECHANISM I's decision rule is NOT, and must not be taken from this page without a grid of its own.**
>
> Lane `w-empty` ([`rungs/2026-08-07-w-empty.md`](rungs/2026-08-07-w-empty.md))
> built §1 into the port as `crates/c2-core/src/elide.rs`. On the 878-TU
> workload `fnbyte-differs` went **4,711 → 3,338**: **1,373 emitted functions
> moved `differs → exact` and zero moved the other way**, and every body the
> elision produced is byte-identical to real c2's (`fnbyte-elided 1373 /
> fnbyte-elided-exact 1373`). `IlBundle::functions()` is untouched, so `mismatch`
> is still 0.
>
> **Three things on this page were corrected by shipping it, and each is marked
> in place below:**
>
> 1. **§1's *"whose source body is empty"* is REFUTED as written — E is a
>    FIXPOINT** (§1.2, board #920). **And the fixpoint is SHIPPED since
>    2026-08-07** on 94 graded call edges — `fnbyte-differs` **3,338 → 3,195**,
>    board **#946**, lane `w-fix`. §1.2 carries the stops.
> 2. **E is a property of the call SITE too**, and the port is safe today only
>    because the IL parser refuses the indirect productions (§1.3, board #921).
> 3. **The shipped predicate is a strict SUBSET of E**, because c2 applies E
>    after its own dead-code elimination and the IL parser refuses four of the
>    bodies that survive it (§1.4, board #922).
>
> Everything about **mechanism I** below is unchanged and still unshipped.

**Status of the rest: SPEC.** No acceptance boundary changed for this document.
`crates/` was untouched by the lane that wrote it (`w-inline`,
[`rungs/2026-08-07-w-inline.md`](rungs/2026-08-07-w-inline.md)).

`IlBundle::functions()` refuses any TU where a callee is also defined
(`crates/c2-il/src/func/bundle.rs:1337`), with the comment *"Refused wholesale
rather than by callee size, because what makes c2 inline … is uncharacterized."*
It is characterized. This document says what by, how well it is graded, and —
more useful — that **the biggest single family the refusal protects against is
not inlining at all**.

---

## 0. TWO mechanisms, not one, and only one of them is a cost model

| | mechanism | governed by `/Ob`? | is it a cost model? | share of w-fnbyte's 4,711 |
|---|---|---|---|---|
| **E** | **EMPTY-CALLEE ELISION** — a call whose callee's *source* body is empty is never emitted | **NO** | **no — syntactic** | family A, **1,886** (40 %) |
| **I** | **INLINE EXPANSION** — `INLINE-P` below | yes | yes | the rest of the call-bearing differs |

Keeping them apart is the whole content of this page. `w-fnbyte` §5.2 named
mechanism **I** and demonstrated it with a probe that is actually mechanism
**E**:

```cpp
void g() {}
void f() { g(); }      // c2 emits `?f` as a bare `blr`, 0 relocations
```

That probe's bytes are exactly as reported. Its *cause* is not: recompile it
with `/Ob0` appended to the workload's own flags — inline expansion off — and
`?f` is **still** a bare `blr` with no relocation. There is no expansion to
disable. The control that separates them is one line apart:

```cpp
int g(int a) { return a; }        // EMITS a bare `blr` — r3 already holds it
int f(int a) { return g(a) + 0; } // /O1: no call.  /Ob0: `bl ?g` SURVIVES
```

Two callees whose emitted `.text` is the identical single `blr` word, and
opposite verdicts under `/Ob0`. **The elision reads the source body, not the
emitted one.** (`work/w-inline/ctl_ob0.py`, probes p2/p6/p8; p4 is its positive
control and it passes — `/Ob0` does restore a genuine expansion.)

> **This is the correction that matters for the port.** `w-fnbyte` §8.1 declined
> family A on the grounds that *"the predicate ('c2 will inline this') is a
> **cost model**, not a syntactic fact"*. For family A it **is** a syntactic
> fact — "does the callee's body do anything" — and the callee's body is in the
> same IL bundle. Measured on 120 workload TUs against the standing instrument's
> own `fnbyte-differs-fn` witnesses: **532 of 532 family-A callers have no
> same-TU callee even at `/Ob0`.** Not one of them is an inline decision.

---

## 1. Mechanism E — the empty-callee elision

> **E.** c2 emits no call, no relocation and no external symbol for a call whose
> callee is defined in this TU and whose **source body is empty**.

Measured at `/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc`, with and without
`/Ob0`, on `work/w-inline/ctl_ob0.py`'s probes:

| probe | callee | callee emits | `/O1` call | `/Ob0` call |
|---|---|---|---|---|
| p2 | `void g() {}` | `blr` | — | **—** |
| p8 | `void g(int a) {}` | `blr` | — | **—** |
| p3 | `~B()` over a trivial `~A()` | `blr` | — | **—** |
| p5 | the same with data members | `blr` | — | **—** |
| **p6** | `int g(int a) { return a; }` | **`blr`** | — | **`bl ?g` ✔** |
| p4 | `int g(int a) { return a + 1; }` | 2 words | — | `bl ?g` ✔ |

p6 is the discriminating cell and p4 is the positive control. **`work/WEC`'s
`eh_unwind_callees` accounting is one special case of this rule** — the
`bundle.rs` comment already records *"the base destructor an empty constructor
names as its unwind action: on the cheap side there is no funclet, so c2 emits
no `bl`, no relocation and no symbol for it"*. That is E, restricted to the
unwind edge; E is the general statement, and it covers ordinary call edges too.

### 1.1 The boundary, walked — twelve more probes

`work/w-inline/eboundary.py`, run **after** the lane's rule was frozen and
GRID-2b graded, so nothing here feeds `INLINE-P`. Same question of every row:
*does the caller keep a REL24 at `/Ob0`?*

| probe | callee | `s` | `/O1` | `/Ob0` | |
|---|---|---:|:--:|:--:|---|
| `e0-empty` | `void g() {}` | 4 | no | **no** | **E** |
| `e1-unused-local` | `void g(int a){ int x; }` | 4 | no | **no** | **E** |
| `e2-dead-store` | `void g(int a){ int x = a; }` | 4 | no | **no** | **E** |
| `e5-static-empty` | `static void g() {}` | 4 | no | **no** | **E** |
| `e6-inline-empty` | `inline void g() {}` | 4 | no | **no** | **E** |
| `e7-member-empty` | an empty in-class member | 4 | no | **no** | **E** |
| `e8-virtual-empty` | an empty **virtual** member, called `s.S::g()` | 4 | no | **no** | **E** |
| `e9-empty-loop` | `void g(int a){ for(int i=0;i<a;++i){} }` | 4 | no | **no** | **E** |
| `e10-arg-effect` | `void g(int a){}` called as `g(sink++)` | 4 | no | **no** | **E** |
| **`e3-return-param`** | `int g(int a){ return a; }` | **4** | no | **yes** | **I** |
| **`e4-return-const`** | `int g(){ return 0; }` | 8 | no | **yes** | **I** |
| `e0-ctl-nonempty` | `int g(int a){ return a+1; }` | 8 | no | **yes** | **I** (control) |

Three things follow, and all three are measurements:

1. **E is keyed on the body being empty *after the front end's own dead-code
   elimination*** — `e1` and `e2` write a local and are still E, and `e9`'s
   whole loop is.
2. **E is independent of linkage, of `inline`, of member-ness and of
   `virtual`.** Five spellings, one verdict. That is the opposite of mechanism
   I, where linkage is the axis (§6.17.3) and `inline` is worth 8 bytes.
3. **A body that produces a *used return value* is not E**, even at `s = 4`.
   `e3` is the discriminator against `e0` at the same emitted size, and it is
   `p6` again with the caller varied.

**What E is still NOT bounded on.** Whether the argument's side effect in `e10`
survives (the call vanishes; `sink++` is not checked here), whether an empty
body with a *volatile* access counts, and what happens at a virtual call
through a *pointer*, where §6.18.4's site rule applies before E can be asked. **NOT MODELLED.**

> **All three are answered below by `w-empty`'s GRID-1/GRID-2** (§1.2–§1.5), on
> the same instrument: 40 cells, `sha256`-frozen and committed before the first
> `cl.exe`, each compiled at the workload's flags **and** with `/Ob0`.
> `f05_side_effect_arg`: the call vanishes and `sink++` **survives**, as four
> words. `c16_volatile_local`: a `volatile` local store is **not** E — it is
> mechanism I. `f10_virtual_ptr`: a `bcctrl`, ungradeable on this observable and
> excluded, exactly as `virt-ptr` is in §4.

## 1.2 §1 is REFUTED as written — E is a FIXPOINT (board #920)

```cpp
void h() {}
void g() { h(); }        // source body NOT empty
void f() { g(); }
```

c2 emits **both** `?f` and `?g` as a bare `blr`. The rule is therefore not *"the
callee's source body is empty"* but

> **the callee's body REDUCES TO NOTHING** — closed under E itself.

`work/w-empty/cells2/g07_empty_calls_empty.cpp`.

> **2026-08-07 — THE FIXPOINT IS SHIPPED, on 94 graded call edges rather than on
> that one cell.** Lane `w-fix` (board **#946**, [`rungs/2026-08-07-w-fix.md`](rungs/2026-08-07-w-fix.md))
> built GRID-3/3b/3c — **34 cells, 94 edges, 94 graded**, each compiled at the
> workload's flags and again at `/Ob0` and scored **per edge** — and
> `crates/c2-core/src/elide.rs`'s `TuEmptyCallees` is now the **least fixpoint**
> of `empty_body` under *"an elidable tail call to a name that reduces to
> nothing"*. `fnbyte-differs` **3,338 → 3,195**; the **143** functions this
> paragraph priced converted, **0** moved the other way, and all 143 are
> `??1?$_Rb_tree_base@…` (board **#952** — #925's caution, repeating with the
> next template up).
>
> **Where the chain STOPS is the part to carry off that grid**, because three of
> the four stops were not obvious from one cell:
>
> * an **all-empty** chain collapses at every link at depths **1, 2, 3, 4, 5, 6
>   and 8** — every caller one `4e800020` (#947, #955);
> * a link whose body calls an external stops it at each of depths 1, 2, 3, and
>   at `/Ob0` every caller at or above the break keeps its REL24;
> * **mechanism I mid-chain is a bare `blr` at every level** — `int
>   m(int a){return a;}` under two callers gives two relocation-free `blr`s at
>   `/O1` and two surviving REL24s at `/Ob0`, so a fixpoint fitted to the
>   *bytes* takes the whole chain and is wrong about all of it (#954). §2.2's
>   `c19` trap, one level up;
> * a mid-node that keeps **bytes** — `g(sink++)`, or a store to a global —
>   drops its own call and **does not** let its caller drop one. What propagates
>   is *emits nothing*, not *elided its call*;
> * a **`Seq`** mid-node whose calls all elide **is** `E` in c2, and the port
>   declines it anyway (#948);
> * a **cycle** is not `E` — and `void r(){r();}` emits a self-branch that takes
>   **no relocation at all**, so the relocation observable reads `E` on a body
>   that is plainly not nothing (#950). The least fixpoint never seeds a cycle,
>   so termination needed no special case.
>
> What is still **NOT MODELLED** here: c2's own dead-code elimination crossing
> the chain (#949 — `void g1(int a){ m(a); }` over `int m(int a){return a;}` is
> `E` at *both* edges and the port keeps both branches), and the `Seq` tier.

## 1.3 E is a property of the CALL SITE too, and that is the hazard (board #921)

```cpp
void g() {}
void f() { void (*p)() = g; p(); }   // c2 emits `b ?g` WITH a REL24
```

The callee is empty and **E does not fire**. c2 folds the pointer to a direct
branch and keeps the relocation. This is §2's *"an indirect site names no callee,
so there is nothing to decline"* seen from E's side, and it means a port
recognizer that reads only the callee is wrong at this site.

`crates/c2-core/src/elide.rs` **does not model the site.** It is safe only
because the IL parser refuses both indirect callers
(`expr-call-in-expr-data-addr-then-plain-call-whole` and `body-0x67`), and
`crates/c2-harness/tests/empty_elision.rs` pins those two refusals against the
real toolchain so that widening either production turns a test red in the same
commit. **Do not widen them without giving E a site condition first.**

## 1.4 The shipped predicate is a strict SUBSET of E (board #922)

c2 applies E *after its own dead-code elimination*; the port's rule asks whether
the IL body decodes as `empty-body`. Four GRID-1 cells separate them —
`void g(int a){ int x = a; }`, `if(a){}`, `{ return; }` and an empty `for` are
**E in c2** and **parse-refused** by `c2-il`. The port keeps its branch there,
which is the direction the correctness rule wants, and it costs **370** workload
functions (every `??$_Destroy_Range@…` whose callee is refused as
`expr-intrinsic-memset`).

> ### ✔ 2026-08-08 — **138 of that 370 are recovered, and the subset relation is now MEASURED at both ends.**
>
> Lane `w-inl0` read the refused callee. It is not an `if(a){}`-style body c2's
> DCE erased: it is a **call whose only other content is an empty tag temporary**
> (`memset(&tmp, 0, 1)` into a `9B` temp bind, passed by reference), and its own
> callee's body is literally `empty-body`. So E's chain was intact all along and
> the middle link was merely **unreadable**.
> `crates/c2-il/src/func/body/shapes/no_effect.rs` reads it **without accepting
> it**, and `c2_core::elide::Reduction::NoEffectCall` hands the fixpoint a
> **link** — never a seed, so a cycle is still never admitted and the round
> ceiling still cannot fire. `fnbyte-differs` **3,195 → 3,057**,
> `fnbyte-elided` **1,516 → 1,654**, 138 closed and **0** opened per symbol,
> `IlBundle::functions()` untouched.
>
> **The subset is still strict, and the residue is still E.** The other 232 are
> `_Destroy_Range` over **class** element types, which take STLport's
> `__false_type` **loop** overload; a compiled cell of that shape is one
> `4e800020` **at `/Ob0` as well**, so it is c2's dead-code elimination and not
> its inliner. §1.4's sentence therefore stands with a sharper population: what
> the port cannot establish is not *"c2 eliminated dead code"* in general but
> **one production at a time**, and the next one is
> `return-scope-close-cflow-label` at 228 bodies. Boards **#990**–**#995**;
> [`rungs/2026-08-08-w-inl0.md`](rungs/2026-08-08-w-inl0.md).

## 1.5 The caller's whole body collapses — the setup goes with the call

In **29 of the 30** cells graded E, the caller's entire `.text` COMDAT is one
`4e800020`, whatever its argument setup would have been: a four-word register
permutation, a two-word literal, an FP argument, three formals, a global's
address. **The thirtieth is an argument with a side effect** — `g(sink++)` drops
the call and keeps the increment, four words — so the rule is *"E discards the
call and a **pure** setup with it"*, not *"E makes the caller a `blr`"*. The port
refuses that caller outright, and every `Selected::Tail` setup is pure by
construction, so nothing shipped depends on it — but a widening that admits a
side-effecting setup has to re-grade this.

---

## 2. Mechanism I — `INLINE-P`, and it is prior art

> ### 2026-08-08 — **what mechanism I's BYTES are is now measured, on 3,195 workload functions, and it is not what this page implies.**
>
> Lane `w-seq` ([`rungs/2026-08-08-w-seq.md`](rungs/2026-08-08-w-seq.md)) took the
> residual `fnbyte-differs` **3,195** and resolved every differing body's callee
> set against the same TU's census rows. **2,801 (87.7 %) are mechanism I**, 369
> are E behind a parse refusal, and 25 are neither and are named (board **#966**).
>
> The page below answers *whether* c2 inlines. It says nothing about *what the
> caller's bytes then are*, and the two candidate answers separate cleanly:
>
> | hypothesis | graded | exact |
> |---|---:|---:|
> | **SPLICE-P** — the port's argument setup ++ c2's callee body | 2,470 | **578** — and **578 of 578** where the port emits no setup, **0 of 953** where it does (#967) |
> | **SPLICE-0** — c2's callee body **alone**, the setup discarded | 2,470 | **1,967** — `seq` **816/816**, `tail` 1,151/1,531, `framed` **0/123** (#968) |
> | **SPLICE-N** — two or more callees, concatenated | 548 | **0** |
>
> **When SPLICE-0 fails it fails in ONE FIELD** (#969): a source register rename
> `r3 → r4` (286, `?Release@Object@Hmx@@`), a destination rename `r3 → r11` (123,
> every `framed` differ), a displacement fold (~92). **No reorder appears in any
> of the 503 failures** — the schedule is not perturbed, the register allocation
> and the frame are.
>
> **Worth on today's port: 726**, the subset where SPLICE-0 is exact *and* the
> port already lowers the callee byte-exactly. The other 1,241 need a parser
> widening first, and #970 prices those productions. **Nothing shipped** — the
> spec is `w-seq` §6 and its first clause is that this is not a substitute for
> §3's decision rule, whose 2.84 % residual is still **NOT MODELLED** below.
>
> ### 2026-08-08 — **SHIPPED, at 723, and SPLICE-0 turns out to be a FIXPOINT.**
>
> Lane `w-splice` ([`rungs/2026-08-08-w-splice.md`](rungs/2026-08-08-w-splice.md))
> built the table above into `crates/c2-core/src/splice.rs` as a **predicate**,
> not as the list of 726: nine clauses, none of which reads the reference obj.
> **723 emitted functions converted, 0 regressed**, per `(TU, emit_name)`.
>
> **Three corrections this page needs, and the second is the one to carry off:**
>
> 1. **SPLICE-0 is exact on `seq` bodies WITH an argument setup, and the setup is
>    the port's frame bookkeeping** (#1007). All **816 of 816** single-call `seq`
>    differs carry a non-empty `setups[0]` whose IL-level argument mapping is the
>    identity — the `mr r31,r3` that saves `this` across the `bl`. c2's inlined
>    body has no frame at all, so the save is the port's lowering and not a
>    transform of the callee's arguments. A rule fitted to the emitted setup
>    converts **zero** of them; a rule that reads the IL converts **634**.
> 2. **c2 CLOSES THE CHAIN** (#1009). `int h(int a){return a+1;} int g(int a){return
>    h(a);} int f(int a){return g(a);}` — c2 emits **`?h`'s two words for all
>    three**. Measured again on the workload from the other side: a rule that took
>    one level relocated against the chain's *intermediate* in **150 of 945**
>    functions where c2 relocates against its *end*
>    (`??1length_error@stlpmtx_std@@` → `??1__Named_exception@…` against c2's
>    `??1exception@std@@`, 145 times). **Mechanism E is a fixpoint (#946) and so
>    is mechanism I.**
> 3. **A chain the port cannot FOLLOW is not a chain that ENDED** (#991). Where
>    the next link is parse-refused, or carries a setup, or has no census row at
>    all, c2 keeps going and the port cannot. Both cases refuse, which costs 9
>    functions and removed the last relocation disagreement.
>
> **Every function the rule moves had its relocation set verified against the
> reference obj, per symbol** — 723 of 723 agree, and `fnbyte-exact-relocated`
> (#882) reads 4,664 at both ends. That check is what caught corrections 2 and 3;
> FUNCTION BYTE MATCH scored all three broken versions `exact`.


Every constant below is transcribed from `docs/LABEL_COUNTER.md` §6.15–§6.20 and
carries its section number. **This document derives none of them.** It grades
them, on a corpus and at flags those rounds never used.

Inputs, per callee `G`:

| input | §6 source | obj-readable? | IL-readable? |
|---|---|---|---|
| `s` — `G`'s own emitted `.text` size at `/O1` | §6.5 | **yes** — c2 emits the callee whether or not it inlined it | only by lowering `G` |
| `linkage(G)` ∈ {STATIC, EXTERNAL} | §6.17.3 | yes — COFF storage class | yes — `.gl` |
| `inline(G)` | §6.17.5 | **yes — the COMDAT selection**: `SELECT_ANY` ⇔ `inline` (verified, §4) | yes |
| `nparams(G)`, `this` included | §6.17.6 | yes — mangled name | yes |
| `varargs(G)` | §6.18.5 | yes — mangled name | yes |
| **`leaf(G)`** — *the source has no call* | §6.18.6, §6.19.6 | **NO — see §5** | **yes** |

```
index(G) =  s                                        linkage == STATIC
            s - 4*(nparams - 1) - 8*[inline]         linkage == EXTERNAL
         -  48*[leaf]                                both classes     (§6.18.6)

N_max(G) =  0                                        if varargs(G)    (§6.18.5)
            EXTERNAL:  UNBOUNDED if index <= 64 else 0                (§6.17.4)
            STATIC:    i = index/4
                       0                             i >= 65
                       UNBOUNDED                     i <= 16
                       min(9, 1 + floor(19/(i-16)))  otherwise         (§6.18.9)

c2 inlines EVERY site of G iff n_sites(G) <= N_max(G); the decision is
ALL-OR-NOTHING per (caller, callee) pair (§6.15.1) and is a property of the
CALLEE alone (§6.15.3a, §6.19.5).
```

**Site-side exceptions**, which are properties of the call and not of the callee:

* an **indirect** site names no callee, so there is nothing to decline
  (§6.18.4). A virtual call through a pointer is one; the same member on an
  object whose dynamic type is known devirtualises and then obeys the rule.
* a **conditional** site moves the `1 → 0` ceiling from `(256, 260]` to
  `(160, 164]` and moves no other boundary (§6.19.9).
* **NEW, and the incumbent could not have seen it**: a site inside a `/EHsc`
  **unwind funclet** (`__unwind$N` / `__catch$N`) is never inlined. Every
  capture in §6.15–§6.20 is at `/O1 /GS- /c`, where no funclet exists. On
  `Utl.cpp` alone, **40 of 73** apparent falsifications of the rule are calls
  whose only caller is a funclet. This matters for the port only indirectly —
  the port emits no funclets — but it is why a naive obj-side grader reads the
  rule as far worse than it is.

---

## 3. How well it is graded, and against what

Graded by **real `c2` under wibo**, from **obj bytes** (#843), by §6.15's own
observable: *an inlined call leaves no trace in the caller's relocation table; a
declined one leaves exactly one REL24 against the callee's symbol.*

The site set is not modelled either. Each TU is compiled **twice** — once at the
workload's flags and once with `/Ob0` appended — and `n_sites(G)` is the REL24
count in the second. `work/w-inline/grade_pair.py`.

| corpus | n | accuracy | majority baseline | precision / recall on `INLINED-ALL` |
|---|---:|---:|---:|---|
| SAMPLE-A, 20 workload TUs (diagnostic) | 12,242 callees | **0.9760** | 0.7232 | 0.977 / 0.990 |
| **SAMPLE-B, 100 workload TUs — HOLD-OUT** | **9,993 callees** | **0.9716** | 0.6434 | 0.969 / 0.988 |
| **GRID-2b, hand probes, 11 families** | **5,040 cells** | **0.9980** | — | — |
| the 4,711's own non-family-A callers | 687 pairs / 681 callers | **1.0000** | — | — |

Both samples are `sha256`-frozen (`work/w-inline/sample_a.txt`,
`sample_b.txt`); GRID-2b is stamped `0c5f520c…` and was committed before one
cell was compiled.

**The step is where §6.17.4 + §6.17.5 say it is.** 11,866 `EXTERNAL` +
`SELECT_ANY` callees of SAMPLE-A (the *diagnostic* sample — SAMPLE-B's own
accuracy is the hold-out number and is in the table above), observed inline
rate by index:

```
   index  <=24   36   44   52   60   64 | 68   72   80   92  >=112
    rate  .997 .890 .911 .928 .872 .701 |.157 .000 .000 .036 .001
                                        ^ the 64/68 step
```

Fitted on `static int f(int)` ladders at `/O1 /GS- /c`; reproduced on real C++
templates, constructors, destructors and operators at `/GR /O1 /Oi /EHsc`, with
`this`, `sret`, references and up to five parameters.

---

## 4. What GRID-2b adds that §6.15–§6.20 does not have

5,376 cells, 12 families × k/p/q rungs × site counts, at the workload's flags.
Every family the incumbent has **no row for anywhere**:

| family | cells | result |
|---|---:|---|
| `tmpl` — a function template instantiation | 336 | 326/336; the 10 misses are all at index 60–64, the boundary cell |
| **`recurse`** — direct recursion, §6.19.10's *"the one call-graph shape that has never appeared on either side of the pair"* | 336 | **336/336** — a self-recursive callee is refused at every size, and `INLINE-P` calls it refused because its own `bl` makes it non-leaf |
| `site-eh` — a destructible object live across the site | 336 | 336/336, identical to the straight-line control |
| `two-level` — the callee itself calls a same-TU function | 336 | 336/336 |
| `member-inclass` vs `member-outclass` | 672 | 672/672 — the `inline` step differs by exactly 8 bytes, §6.17.5 |
| `static-plain`, N swept 1…10 | 1,680 | **1,680/1,680** — SCHEDULE D's whole graduated middle |
| `varargs` | 336 | 336/336, categorical at every size |
| **`virt-ptr`** | 336 | **UNGRADEABLE and excluded**: the site is a `bcctrl`, there is never a REL24, so this observable cannot distinguish "inlined" from "no direct call ever existed". §6.18.4 says the same thing from the other side. |

---

## 5. THE OPEN INPUT — `leaf` is not obj-readable, and that is why the port needs the IL

§6.19.6 already said it: the index is *"a post-allocation byte count, minus 48
for a predicate that is false in the emitted code and true only upstream of
it."* On the ladders that never bit, because no callee's own calls were
inlinable. On real C++ they are, and it bites hard:

| leaf bit derived from | SAMPLE-B accuracy |
|---|---:|
| the `/O1` obj (a callee whose calls were inlined away reads LEAF) | 0.9631 |
| **dropped entirely** | **0.9716** |
| the `/Ob0` obj (source-leaf) | 0.9688 |

and on GRID-2b, whose callees are genuinely leaf in source, the same three
readings give **0.9980 / 0.8401 / 0.9980**. So:

> **The 48-byte term is real** — GRID-2b's ladders step at `s = 112/116`, which
> is 64/68 + 48 exactly, on hand probes at the workload's own flags. **And no
> compilation supplies its input.** `/O1` over-reports leaf; `/Ob0`
> under-reports it, because c2 decides bottom-up and a callee's own inlinable
> calls are gone by the time it is priced. The truth is between the two and
> **NOT MODELLED**.

**This is the one place the port is better off than any obj-side instrument.**
The IL bundle contains the callee's *un-lowered* body, and it contains it before
any expansion. A recognizer can ask "does this body contain a call" of the IL
directly. Nothing in this lane could.

---

## 6. What a port recognizer would consume, and in what order

Nothing here is implemented. This is the shape, so the next lane does not have
to re-derive it.

1. **Mechanism E first, and alone if that is all that is taken.** ~~For a call
   edge `F → G` with `G` defined in this bundle, `G`'s IL body empty **after
   dead-code elimination**, and `G`'s return value **unused at the site**: emit
   no branch, no REL24 and no external symbol for `G` on `F`'s account.~~
   **DONE 2026-08-07** — `crates/c2-core/src/elide.rs`, and the shipped rule is
   narrower than this paragraph in one way and wider in another. Narrower: the
   port asks `IlFunction::empty_body`, not "empty after DCE", because it has no
   DCE (§1.4). Wider: it also needs a **call-site** condition it does not have
   (§1.3). Linkage, `inline`, member-ness and `virtual` do not enter it (§1.1) —
   that part held on 40 cells. It covered **1,373 of the 1,886** with the
   emitter untouched, and the residual is accounted in
   [`rungs/2026-08-07-w-empty.md`](rungs/2026-08-07-w-empty.md) §5.

   **The one thing this paragraph got most wrong is not in the rule.** It is
   name-keyed, and *which name* is the whole problem: `IlFunction::mangled_name`
   is paired positionally over `.ex` segments and disagrees with the per-record
   `FnCensus::emit_name` on **74,955** rows of this workload. Keyed on the first,
   the rule turned 14 byte-exact bodies wrong and converted nothing. Board #918;
   the scan prints `fnbyte-name-disagree` on every run now.
2. ~~**Mechanism I only with `s`.** `INLINE-P` is indexed on the callee's own
   *emitted* size, so a recognizer can only apply it to a `G` the port can
   already lower — otherwise it does not know `s`. That is a real ordering
   constraint and it is why I is not cheap: the port must lower `G` to decide
   whether to inline `G` into `F`.~~
   **DONE 2026-08-08** — `crates/c2-core/src/splice.rs`, and the ordering
   constraint is exactly as this paragraph states: S6 lowers `G` before S7 can
   ask its size. What the paragraph does not say, and what makes the rule cheap
   after all, is that **`s ≤ 64` makes the decision categorical in both linkage
   classes**, so the port never has to evaluate `index` or `N_max` at all — and
   on today's lowered class that bound never binds (#1008). The rule reached
   **723** functions with `IlBundle::functions()` untouched.
3. **`inline(G)` comes from the COMDAT selection** if read from an obj, and from
   the `.gl` record if read from IL. Verified obj-side on GRID-2b: every
   `inline`, in-class member and template instantiation is `SELECT_ANY`; every
   plain out-of-class definition is `SELECT_NODUPLICATES`; and the two classes'
   ladders differ by exactly the 8 bytes §6.17.5 measured.
4. **Do not narrow `functions()` on I.** The 2.4–2.8 % residual is a wrong-bytes
   emit if it lands on the accept side, and board #269/#844's standing hazard
   applies. E has no residual on the 532 callers measured and is the one that
   can be taken safely.

---

## 7. What this leaves `NOT MODELLED`

* ~~**E's exact source predicate.** Three probes, one boundary (§1).~~
  **CLOSED to the extent 134 graded cells close it** (§1.1–§1.5): the boundary is
  walked, the rule is shipped, and the three things it is still not was a list
  rather than a gap — the **fixpoint** (#920), the **call site** (#921), and
  **c2's own DCE** (#922). **The fixpoint is closed too** (#946): 94 further call
  edges, and the rule iterates. What remains genuinely unmodelled about E:
  **c2's own DCE crossing a chain** (#949), the **`Seq` tier** (#948), whether a
  `volatile` access in an otherwise empty body is E (`c16` says no, n = 1), and
  what happens at an indirect site once the parser can reach one.
* **`leaf`'s true input** (§5) — the largest single source of `INLINE-P`'s
  workload residual.
* **The 2.84 % SAMPLE-B residual itself**: 205 false inlines and 79 false
  declines, clustered within ±8 bytes of the step. On SAMPLE-A, where the
  population was examined, **238 of 294 misses are template instantiations** —
  and §6.15–§6.20 has no template row anywhere. **No term was fitted to them**,
  per `work/w-inline/PREREG.md` §5.
* **Everything §6.19.10 leaves open**, unchanged — the rule generating SCHEDULE
  D, what the 48 bytes *are*, the `/Ox` loop threshold, and why the two linkage
  classes use different size measures.
* **Whether E and I interact.** Every probe here has one mechanism or the other.
