# guard2 — independent re-derivation of the §2 emit prediction for six axes2 cells

**Phase A. Written before reading any truth artifact.** Agent `guard2`, lane
`w-emitpred`, worktree `.claude/worktrees/w-emitpred`.

## 0. Inputs actually read, exhaustively

1. `CLAUDE.md` (worktree root) and `~/.claude/CLAUDE.md`.
2. `docs/PHASE7_PLAN.md` **§2 only** (lines 75–151; I read to the `## 3` header
   and stopped).
3. `docs/rungs/_2026-08-02-w-emitpred-prereg.md` (whole file).
4. The six `cell.cpp` sources, and **nothing else** in those directories. The
   directory listing used `find -type f`; it printed names only. Each of the six
   cell directories contains exactly one file, `cell.cpp` — there is **no
   `spec.json`** in any of them, so that hazard did not arise.

Nothing on the quarantine list has been read: no `observed.json`,
`il_names.json`, `RESULTS.md`, `PREDICTIONS.md`, nothing under `axes1/`,
nothing under `guard1/`, no `MAGNITUDE.md`, no `PHASE7_VALIDATION.md`, no obj /
`.cod` / `.pch` / `_CL_*`, no `git log` bodies (I ran no `git log` at all).
I compiled nothing.

## 1. Contamination disclosure — the brief leaks, and I am naming it

I was asked to say loudly if any sentence of my tasking entails facts about
truth. **Two do**, both repeated in the follow-up message:

> "Your six cells decide whether this lane's headline number is **5 or 7**"

> "whether **A3 and A4** break — that is the question your work decides"

These are joint leaks about truth-relative-to-axes2, in this form:

* "5 or 7" with six cells at stake ⇒ **exactly two of my six cells are cells
  where axes2's prediction already disagrees with truth** (only those can
  promote to a confirmed violation). By complement, **on four of my six cells
  axes2's prediction matched truth.**
* "whether A3 **and** A4 break" ⇒ those two candidate cells are distributed one
  in A3 and one in A4 (or at minimum, candidates exist in both axes) ⇒
  **`a2_04` is almost certainly not a candidate**, i.e. axes2's prediction for
  it matched truth.

This does not tell me *which* A3 or A4 cell, nor any symbol. But it is a real
bound on truth and it creates a specific bias: it invites me to reverse-engineer
axes2's prediction on four cells rather than derive my own. **I have not used
it.** Everything in §3 below was derived from the six sources and §2's text; I
noticed the inference only when re-reading the brief for exactly this defect,
after the derivation was fixed in my head. I record it so the lead can discount
my agreements accordingly — my *disagreements* are unaffected by it, since the
leak gives no reason to disagree anywhere.

I found nothing in the brief of the Selection-byte / section-size / symbol-count
class. That trap was avoided.

A third, much weaker item: §2 line 149 names "`??_9` adjustor thunks, multiple
inheritance" as out-of-grid unknowns. That is a caveat about coverage, not about
any cell's output, and I treat it as legal §2 text.

## 2. Rulings on §2's under-determined clauses, committed before any cell

These four rulings drive everything below. Each is a commitment, with the
alternative recorded but explicitly **not** my primary.

### R-a. The scalar-deleting-destructor rider is **conditional on the class
having a virtual destructor.**

§2: "a kept constructor of C keeps C's vtable, **whose slots** force every
virtual of C plus the synthesized scalar-deleting destructor, called or not."

The forcing agent named by the sentence is *the slots*. `??_G` is the entity
that occupies the destructor slot of a vtable; a class with no virtual
destructor has no such slot and no such function. "**the** synthesized
scalar-deleting destructor" is a definite description, and a definite
description presupposes existence — a predicate cannot predict emission of a
function that the language never causes to exist. So for a polymorphic class
with no virtual destructor, §2-as-written predicts **no `??_G`**.

*Alternative (rejected):* read the rider as an unconditional conjunct, in which
case `??_GA@@…`, `??_GB@@…`, `??_GD@@…` are predicted in `a3_01`, `a3_02`,
`a3_08`. I reject it because it makes §2 predict symbols that cannot exist,
which is not a falsifiable claim about c2 but a malformed one. Under that
reading all three A3 cells would trivially "violate" §2 — an artifact of the
reading, not of the compiler, and exactly the over-promotion this lane's guard
mechanism exists to stop.

*Consequence, stated in advance:* under my primary, **no `??_G` in any of
`a3_01`, `a3_02`, `a3_08`**; and `a4_05`'s `V` does have a virtual destructor,
so the rider *would* fire there if its vtable were kept — but it is not kept
(see §3.5), so no `??_G` there either. **`??_G` appears in none of my six
predicted sets.**

### R-b. R1's anonymous-namespace clause reaches namespace-scope and
out-of-line definitions, **not in-class (implicitly inline) members.**

§2 R1: "every definition with **external non-COMDAT linkage** — plain extern,
`extern "C"`, *any out-of-line definition* (member, static member, virtual), and
anonymous-namespace functions not declared `static`".

Three textual reasons:

1. The head clause characterises roots as the *strong-linkage-like* class. An
   in-class member definition is the paradigm COMDAT/inline definition, which
   §2 everywhere else treats as needing a reference to survive.
2. The member-related item in the list is qualified: "*any **out-of-line**
   definition* (member, static member, virtual)". "Out-of-line" is doing work;
   its contrast class is precisely in-class definitions.
3. "anonymous-namespace **functions** not declared `static`" draws its contrast
   with `static` — i.e. it is about the anon-ns-vs-`static` axis at namespace
   scope, the thing §2's refutation list calls out ("anon-namespace-equals-
   static" refuted). It says nothing about inline-ness, so it cannot be read as
   overriding the in-line/out-of-line distinction that item 2 draws.

*Alternative (rejected):* "anonymous-namespace functions" is read to mean every
function whose enclosing scope chain passes through an unnamed namespace,
including in-class members. That would make `a4_05`'s `V::f`, `V::g`, `V::~V`
(and, per R-a, `??_GV@@…`) all roots with no object ever constructed, and would
make `a4_06`'s `S::inl` a root. I reject it as reading past both the head clause
and the "out-of-line" qualifier.

### R-c. R1's "(member, static member, virtual)" parenthetical **does** reach an
explicit specialization of a member of a class template.

`template <> int H<int>::b(int x) { … }` satisfies R1 twice over: it is (i)
literally an out-of-line member definition, matching "*any out-of-line
definition* (member…)" — and the parenthetical is illustrative, governed by
"any"; and (ii) a definition with external, **non-COMDAT** linkage, matching the
head clause, because an explicit specialization is not implicitly inline and
must be defined in exactly one TU. Nothing in §2 carves templates out of R1;
§2's template-specific root is R2, "explicit **instantiation** definitions",
which is a different construct and does not apply here.

**Commit: it is a root, and is emitted although never referenced.**

*Alternative (rejected):* treat anything template-related as COMDAT-linkage and
therefore reference-gated, which would drop `H<int>::b`. Rejected: it confuses
explicit specialization (one definition, strong) with implicit instantiation
(COMDAT), and §2's R1 head clause keys on linkage, which is unambiguous here.

### R-d. Propagation reaches **implicitly-defined, non-trivial special members**
— in particular, base-class default constructors called by a kept derived
constructor.

§2: emission is "at **ODR-use** granularity, **pre-optimization**"; propagation
adds F when a kept definition ODR-uses it via "a call anywhere in the
pre-optimization body".

ODR-use is a language-level notion, and it is unambiguous here: `D o;` odr-uses
`D::D()`; `D::D()`, in initialising its bases, odr-uses `A::A()` and `B::B()`.
For a polymorphic base these implicit constructors are **non-trivial** (they
must establish the base's own vfptr), so they are real definitions with bodies,
and "pre-optimization" is the clause that stops the subsequent inlining of those
bodies from removing them. §2 chose "pre-optimization" deliberately (its
refutation list rejects "post-optimization reference sets"), so the fact that
`/O1` collapses these calls to two stores is by construction irrelevant.

Symmetrically I **exclude trivial** implicit special members: `H<int> h;` in
`a2_04` odr-uses a trivial default constructor, which has no definition to emit,
so nothing is added. `D`'s implicit destructor in the A3 cells is trivial (not
virtual, no non-trivial subobject dtors) — nothing added.

*Consequence, and it is the largest single lever in this derivation:* a kept
`A::A()` is itself a kept constructor, so **R-d feeds the vtable rule**: it
keeps `??_7A@@6B@` and forces every virtual of `A`; likewise for `B`. In
`a3_01` this is decisive — `B::g` is overridden in `D`, so it appears in no slot
of either of `D`'s vtables, yet it is forced anyway through `B`'s own vtable via
`B::B()`.

*Alternative (recorded, not primary):* propagation is read to range only over
functions that appear in the source, so implicit base constructors are invisible
to §2. Under it, drop `??0A@@…`/`??0B@@…` from all three A3 cells, and drop
`?g@B@@UAAHH@Z` from `a3_01` (it survives in `a3_02`, where it still occupies a
slot of `D`'s second vtable). I reject it because §2 says "ODR-use", which is a
term of art that plainly covers implicit special members, and because §2's
vtable rule already commits to reasoning about compiler-generated entities
(vtables, `??_G`) rather than only source-visible ones.

I flag R-d as the point where I am least confident of *agreement* with the
first derivation, and I say so before seeing it. Note that R-d does **not**
collapse the designed `a3_01`/`a3_02` contrast: those cells' own comments say
the contrast is the adjustor thunk ("adjustor thunk site" vs "no thunk needed"),
which is untouched.

### R-e (minor). Thunks and vtables.

* Adjustor thunks (`?g@D@@W3AA…`, and the `??_9` family) are **`[GAP]`**: §2
  contains no clause that could produce a thunk. Its vtable rule forces "every
  virtual of C", and the thunk is not a virtual of `D` — it is a synthesized
  entry-point *for* one. I list such symbols in my predicted set because a
  second-base override cannot be dispatched without one, but tagged `[GAP]` so
  that a miss here scores as "§2 has no clause", never as "§2 predicted and was
  wrong".
* Vtables themselves (`??_7…`) are **data** symbols and are outside the graded
  set (obj `.text` code-COMDAT leaders). They appear in my reasoning, never in
  my predicted sets.

## 3. Per-cell derivations

Mangling convention assumed: X360 `cl.exe` 16.00 — member functions are
`__cdecl`, so the calling-convention letter is `A`, giving `QAA…` (public
non-static) and `UAA…` (public virtual) where an x86 build would read `QAE…` /
`UAE…`. Free functions `?name@@YAHH@Z`. Unnamed namespace mangles as `?A@` in
this compiler generation (the `?A0x<hash>` form is later). **Name-spelling
mismatches of this kind are normalization errors, not violations**, and should
be reconciled before any cell is scored.

Common to all six: `sink` and `use` are declared `extern` and never defined in
the TU — they are references, not definitions, and are never in the emit set.

---

### 3.1 `a2_04` — explicit member specialization, unreferenced (A2)

```cpp
template <class T> struct H { T a(T x) { return x*3+1; } T b(T x) { return x+7; } };
template <> int H<int>::b(int x) { return x-5; }
extern int sink(int);
int anchor(int x) { H<int> h; return sink(h.a(x)); }
```

* **R1** → `anchor` (external, non-COMDAT).
* **R1 via R-c** → the explicit specialization `H<int>::b`.
* **Propagation** from `anchor` → `H<int>::a` (called; implicitly instantiated
  COMDAT). Kept even though `/O1` will inline it, because §2 is
  pre-optimization.
* **R2 does not apply** — there is no explicit instantiation definition, so the
  never-referenced generic `H<int>::b` template body is not instantiated (and
  in any case is displaced by the specialization).
* `H<int> h;` odr-uses a **trivial** default ctor → nothing added (R-d).
* No virtuals anywhere → no vtable rule, no `??_G`.

**Predicted set (3):**

| symbol | tag | clause |
|---|---|---|
| `?anchor@@YAHH@Z` | `[REACH]` | R1 |
| `?a@?$H@H@@QAAHH@Z` | `[REACH]` | propagation (call in `anchor`) |
| `?b@?$H@H@@QAAHH@Z` | `[REACH]` | R1 + R-c (out-of-line member def / external non-COMDAT) |

---

### 3.2 `a3_01` — MI, `D` overrides the second base's virtual (A3)

```cpp
struct A { virtual int f(int x) { return x*3+1; } };
struct B { virtual int g(int x) { return x+7; } };
struct D : A, B { virtual int g(int x) { return x-5; } };
int anchor(int x) { D o; use(&o); return sink(x); }
```

Layout: `A` at 0 (primary vfptr), `B` at 4. `D` has two vftables;
`??_7D@@6BA@@@` = [`A::f`], `??_7D@@6BB@@@` = [adjustor thunk for `D::g`].

* **R1** → `anchor`.
* **Propagation** → `??0D@@QAA@XZ` (`D o;`; non-trivial, sets vfptrs).
* **Propagation, R-d** → `??0A@@QAA@XZ`, `??0B@@QAA@XZ` (base subobject
  initialisation in `D`'s pre-optimization ctor body).
* **Vtable rule on `D`** (kept ctor) → every virtual of `D`: `A::f` (inherited,
  unoverridden, occupies slot 0 of the primary vftable) and `D::g`.
* **Vtable rule on `A`** (kept ctor, R-d) → `A::f` (already present).
* **Vtable rule on `B`** (kept ctor, R-d) → `B::g` — **even though `D`
  overrides it**, because `??_7B@@6B@` is kept by `B::B()` and its slot 0 holds
  `B::g`. This is the load-bearing consequence of R-d.
* No virtual destructor anywhere → **no `??_G`** (R-a).

**Predicted set (7 REACH + 1 GAP):**

| symbol | tag | clause |
|---|---|---|
| `?anchor@@YAHH@Z` | `[REACH]` | R1 |
| `??0D@@QAA@XZ` | `[REACH]` | propagation |
| `??0A@@QAA@XZ` | `[REACH]` | propagation (R-d) |
| `??0B@@QAA@XZ` | `[REACH]` | propagation (R-d) |
| `?f@A@@UAAHH@Z` | `[REACH]` | vtable rule (`D`'s slot; also `A`'s) |
| `?g@D@@UAAHH@Z` | `[REACH]` | vtable rule (virtual of `D`) |
| `?g@B@@UAAHH@Z` | `[REACH]` | vtable rule on `B` via R-d |
| `?g@D@@W3AAHH@Z` | `[GAP]` | adjustor thunk, `this-=4`; **no §2 clause produces it** |

---

### 3.3 `a3_02` — MI control, no override (A3)

```cpp
struct D : A, B { virtual int h(int x) { return x-5; } };   // new virtual, not an override
```

`??_7D@@6BA@@@` = [`A::f`, `D::h`], `??_7D@@6BB@@@` = [`B::g`]. No thunk.

* Same roots and propagation as 3.2, giving `anchor`, `??0D`, `??0A`, `??0B`.
* Vtable rule on `D` → `A::f`, `D::h`, and `B::g` (which here **does** occupy a
  slot of `D`'s second vftable, so it is forced by two independent paths — the
  `D` vtable and, via R-d, `B`'s own).
* No virtual destructor → no `??_G`.

**Predicted set (7 REACH, 0 GAP):**

| symbol | tag | clause |
|---|---|---|
| `?anchor@@YAHH@Z` | `[REACH]` | R1 |
| `??0D@@QAA@XZ` | `[REACH]` | propagation |
| `??0A@@QAA@XZ` | `[REACH]` | propagation (R-d) |
| `??0B@@QAA@XZ` | `[REACH]` | propagation (R-d) |
| `?f@A@@UAAHH@Z` | `[REACH]` | vtable rule |
| `?h@D@@UAAHH@Z` | `[REACH]` | vtable rule |
| `?g@B@@UAAHH@Z` | `[REACH]` | vtable rule (`D`'s second vftable; also `B`'s) |

Note the structural prediction that distinguishes this pair: **`a3_01` and
`a3_02` have the same REACH cardinality (7); the only difference is the
`[GAP]` thunk in `a3_01` and `D::g` vs `D::h`.** Under the R-d alternative both
sets lose `??0A`/`??0B`, and `a3_01` additionally loses `?g@B@@UAAHH@Z`
(4 REACH) while `a3_02` keeps it (5 REACH).

---

### 3.4 `a3_08` — SI, inherited virtual neither overridden nor called (A3)

```cpp
struct A { virtual int f(int x) { … } virtual int only_a(int x) { … } };
struct D : A { virtual int g(int x) { … } };
int anchor(int x) { D o; use(&o); return sink(x); }
```

`??_7D@@6B@` = [`A::f`, `A::only_a`, `D::g`]. `only_a` is the point of the cell:
§2's "every virtual of C … called or not" plainly covers an inherited,
never-called, never-overridden virtual, because it occupies a slot of `D`'s
vtable.

* R1 → `anchor`. Propagation → `??0D@@QAA@XZ`, and (R-d) `??0A@@QAA@XZ`.
* Vtable rule on `D` → `A::f`, `A::only_a`, `D::g`. Vtable rule on `A` adds
  nothing new.
* No virtual destructor → no `??_G`.

**Predicted set (6 REACH, 0 GAP):**

| symbol | tag | clause |
|---|---|---|
| `?anchor@@YAHH@Z` | `[REACH]` | R1 |
| `??0D@@QAA@XZ` | `[REACH]` | propagation |
| `??0A@@QAA@XZ` | `[REACH]` | propagation (R-d) |
| `?f@A@@UAAHH@Z` | `[REACH]` | vtable rule |
| `?only_a@A@@UAAHH@Z` | `[REACH]` | vtable rule ("every virtual of C, called or not") |
| `?g@D@@UAAHH@Z` | `[REACH]` | vtable rule |

Under the R-d alternative: drop `??0A@@QAA@XZ` (5 REACH); everything else is
unchanged, because `A`'s virtuals are already forced through `D`'s vtable.

---

### 3.5 `a4_05` — anon-ns class with virtuals, **no object constructed** (A4)

```cpp
namespace {
struct V { virtual int f(int x) { … } virtual int g(int x) { … } virtual ~V() {} };
}
extern int sink(int);
int anchor(int x) { return sink(x) + 3; }
```

* **R1** → `anchor` only. By **R-b**, `V`'s members are in-class definitions
  (implicitly inline) and are *not* reached by "anonymous-namespace functions
  not declared `static`"; nor are they out-of-line definitions.
* **No constructor of `V` is ever ODR-used** — no object of `V` exists anywhere
  in the TU. So the vtable rule **does not fire**: `??_7V@@6B@` is not kept, and
  neither `V::f`, `V::g`, `V::~V` nor `??_GV@@…` is forced.
* Nothing else references anything in `V`. Note `V` has a virtual destructor, so
  this is the one cell of my six where R-a's rider *would* have applied had a
  constructor been kept — it is not, so R-a is not load-bearing here.

**Predicted set (1 REACH, 0 GAP):**

| symbol | tag | clause |
|---|---|---|
| `?anchor@@YAHH@Z` | `[REACH]` | R1 |

**Explicitly predicted absent:** `?f@V@?A@@UAAHH@Z`, `?g@V@?A@@UAAHH@Z`,
`??1V@?A@@UAA@XZ`, `??_GV@?A@@UAAPAXI@Z`.

Under the R-b alternative all four of those become roots and the set is 5.

---

### 3.6 `a4_06` — anon-ns class, out-of-line member vs in-class member, neither
referenced (A4)

```cpp
namespace {
struct S { int m(int x); int inl(int x) { return x-7; } };
int S::m(int x) { return x*3+1; }
}
extern int sink(int);
int anchor(int x) { return sink(x) + 3; }
```

This cell is the direct test of R-b, and R1 hits `S::m` from two directions:
it is "*any out-of-line definition* (member…)", and it is an
anonymous-namespace function not declared `static`. `S::inl` is matched by
neither — it is an in-class, implicitly-inline definition — and nothing in the
TU references it.

* R1 → `anchor`; R1 → `S::m`.
* No propagation from `anchor` (it only calls the undefined `sink`), and
  nothing propagates *out of* `S::m` (its body calls nothing).
* No virtuals → no vtable rule, no `??_G`.

**Predicted set (2 REACH, 0 GAP):**

| symbol | tag | clause |
|---|---|---|
| `?anchor@@YAHH@Z` | `[REACH]` | R1 |
| `?m@S@?A@@QAAHH@Z` | `[REACH]` | R1 (out-of-line member def; anon-ns non-`static`) |

**Explicitly predicted absent:** `?inl@S@?A@@QAAHH@Z`.

---

## 4. Summary table

| cell | REACH | GAP | predicted symbols |
|---|---:|---:|---|
| `a2_04` | 3 | 0 | `?anchor@@YAHH@Z`, `?a@?$H@H@@QAAHH@Z`, `?b@?$H@H@@QAAHH@Z` |
| `a3_01` | 7 | 1 | `?anchor@@YAHH@Z`, `??0D@@QAA@XZ`, `??0A@@QAA@XZ`, `??0B@@QAA@XZ`, `?f@A@@UAAHH@Z`, `?g@D@@UAAHH@Z`, `?g@B@@UAAHH@Z`, + `?g@D@@W3AAHH@Z` `[GAP]` |
| `a3_02` | 7 | 0 | `?anchor@@YAHH@Z`, `??0D@@QAA@XZ`, `??0A@@QAA@XZ`, `??0B@@QAA@XZ`, `?f@A@@UAAHH@Z`, `?h@D@@UAAHH@Z`, `?g@B@@UAAHH@Z` |
| `a3_08` | 6 | 0 | `?anchor@@YAHH@Z`, `??0D@@QAA@XZ`, `??0A@@QAA@XZ`, `?f@A@@UAAHH@Z`, `?only_a@A@@UAAHH@Z`, `?g@D@@UAAHH@Z` |
| `a4_05` | 1 | 0 | `?anchor@@YAHH@Z` |
| `a4_06` | 2 | 0 | `?anchor@@YAHH@Z`, `?m@S@?A@@QAAHH@Z` |

## 5. Falsifiable advance calls

Written before any truth is read. Each states what would refute it.

1. **`a2_04`: `?b@?$H@H@@QAAHH@Z` is emitted** although never referenced.
   *Refuted if absent* — and that would be a **REACH failure** of R1/R-c, the
   sharpest single result available in my six.
2. **`a4_06`: `?m@…` present, `?inl@…` absent.** Refuted by either half.
   `?inl@…` present would refute R-b in the permissive direction; `?m@…` absent
   would refute R1's "any out-of-line definition" clause for internal linkage.
3. **`a4_05` contains exactly one code symbol.** Refuted if any member of `V`
   appears — which would be a REACH failure of the vtable rule's precondition
   ("a **kept constructor**"), i.e. something other than a constructor forcing a
   vtable, and would make `a4_05` a sibling of the A9/D6 `dynamic_cast` case.
4. **No `??_G` in `a3_01`, `a3_02`, `a3_08`** (none has a virtual destructor).
   If one appears, R-a is wrong; but note that in that event *no* reading of §2
   is safe, since the unconditional reading would also demand `??_G` for classes
   that have no destructor slot.
5. **`a3_01`: `?g@B@@UAAHH@Z` is present despite being overridden by `D`.**
   Refuted if absent — and its absence would come as a package with the absence
   of `??0A@@QAA@XZ`/`??0B@@QAA@XZ`, i.e. it would refute **R-d specifically**.
   Conversely, if `??0A`/`??0B` are present but `?g@B@@…` is absent, that is a
   genuine §2 REACH failure of the vtable rule (a kept ctor that does *not* keep
   its class's vtable).
6. **`a3_01` contains an adjustor thunk** (`?g@D@@W3AAHH@Z` under my spelling
   assumptions), and its presence is a **GAP**, never a §2 success — §2 has no
   clause that produces a thunk. `a3_02` contains none.
7. **Mangling:** the unnamed namespace spells as `?A@`, member functions as
   `…QAA…`/`…UAA…` (PPC `__cdecl`). If truth shows `?A0x…` or `…QAE…`, my
   *names* are wrong but my *sets* are not; such differences must be normalized
   before scoring and must not be counted as violations in either direction.

## 6. Where my derivation is most likely to disagree with the first one

Stated in advance so that a later agreement cannot be claimed as stronger than
it is, and a later disagreement cannot be explained away:

* **R-d (implicit base constructors)** — highest risk. It adds `??0A`/`??0B` to
  all three A3 cells and adds `?g@B@@UAAHH@Z` to `a3_01`. If the first
  derivation omitted them, all three A3 cells are `AMBIGUOUS` by the
  pre-registered rule, regardless of which of us truth favours.
* **R-a (`??_G` rider)** — if the first derivation took the rider literally, it
  predicted `??_G` for `a3_01`/`a3_02`/`a3_08` and we disagree there too.
* **R-c (explicit member specialization as a root)** — a clean binary; I expect
  agreement, but if not, `a2_04` is `AMBIGUOUS`.
* **R-b (anon-ns reach)** — a clean binary on both `a4_05` and `a4_06`; the two
  cells are designed so that R-b's two directions are separately visible.

**Frozen.** No edits to this file after the lead's commit.
