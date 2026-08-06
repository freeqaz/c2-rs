# INLINE_PREDICATE — when c2 does not emit the call the IL contains

> ## Status, 2026-08-07 — **MECHANISM E IS SHIPPED. MECHANISM I IS NOT, and must not be taken from this page without a grid of its own.**
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
>    FIXPOINT** (§1.2, board #920).
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

`work/w-empty/cells2/g07_empty_calls_empty.cpp`. **The port ships the one-step
version**: after `w-empty`, `?g` converts and `?f` does not, and the residual it
leaves is **143** workload functions (`??1?$_Rb_tree_base@…`, whose callee is a
`_STLP_alloc_proxy` destructor that is itself a tail call that elides). The
fixpoint is board **#924** and has one cell behind it, which is why it was not
taken.

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

## 1.5 The caller's whole body collapses — the setup goes with the call

In **all thirty** cells graded E, the caller's entire `.text` COMDAT is one
`4e800020`, whatever its argument setup would have been: a four-word register
permutation, a two-word literal, an FP argument, three formals, a global's
address. The one exception is an argument with a **side effect** — `g(sink++)`
keeps the increment and drops the call — and the port refuses that caller
outright, so no shipped rule depends on it.

---

## 2. Mechanism I — `INLINE-P`, and it is prior art

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
2. **Mechanism I only with `s`.** `INLINE-P` is indexed on the callee's own
   *emitted* size, so a recognizer can only apply it to a `G` the port can
   already lower — otherwise it does not know `s`. That is a real ordering
   constraint and it is why I is not cheap: the port must lower `G` to decide
   whether to inline `G` into `F`.
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
  **CLOSED to the extent 40 graded cells close it** (§1.1–§1.5): the boundary is
  walked, the rule is shipped, and the three things it is still not is now a
  list rather than a gap — the **fixpoint** (#920), the **call site** (#921), and
  **c2's own DCE** (#922). What remains genuinely unmodelled about E: whether a
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
