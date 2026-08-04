# guard1 — Phase A, independent re-derivation (FROZEN)

Agent `guard1`, lane `w-emitpred`. Written from **§2's text + cell sources only**.
Prereg guard 1 (`docs/rungs/_2026-08-02-w-emitpred-prereg.md`): a violation on a
designed cell scores only if a second agent, given source + §2 and nothing else,
reproduces the first agent's prediction; disagreement ⇒ `AMBIGUOUS`.

## 0. Inputs actually read, and one disclosed contamination

**Read (allowed):** `docs/PHASE7_PLAN.md` §2 (lines 75–151), the prereg, and the
cell sources listed below. `CLAUDE.md` (repo + global).

**Deliberately NOT read:** every artifact on the quarantine list, plus
`cells/*/spec.json` — the spec files were not on the list but could plausibly
carry a prediction or an expected set, so I left them closed. Only `main.cpp` /
`cell.cpp` / `*.h` were opened.

**CONTAMINATION, disclosed loudly and unprompted.** My own tasking message leaks
truth for `a5c4`. It states that §2's nominated observable "measures `a5c4`'s
`cand` and `cand2` as **Selection = 2 (ANY)**". A COMDAT Selection byte exists
only for a COMDAT that is *present in the obj*. Therefore I knew, **before**
writing a word of this file, that:

> truth(`a5c4`) ⊇ {`cand`, `cand2`} — both are emitted.

This is unavoidable — it is in the brief that constitutes my task — but it means
my Phase-A independence on `a5c4` is **partial, not clean**. Consequences I bind
myself to:

* `a5c4` is marked **CONTAMINATED-GUARD** in the Phase-B table. My agreement or
  disagreement with axes1 on `a5c4` is weaker evidence than on any other cell.
* The `a5c1` derivation is **uncontaminated** — nothing in the brief tells me
  whether `cand` is emitted there. `a5c1` is therefore the load-bearing cell for
  the R1 question, and I say so *before* seeing it.
* The Reading-A/Reading-B commitment in §1.4 below is argued **from §2's text
  alone**, and I have written the argument so it can be audited without
  reference to any cell outcome. If a reader thinks the argument only works
  because I knew a5c4's truth, the argument has failed and should be discarded.
* The admissibility ruling on the Selection byte (lead's question 1) is
  **pre-committed in §3 below, before Phase B**, precisely because I already
  know which way that evidence cuts for a5c4. Deciding admissibility after
  seeing the rest of truth would be the goalpost-move I exist to prevent.

## 1. A1 — `a5c1` and `a5c4`, both readings of R1

R1, verbatim:

> **Roots:** (1) every definition with external non-COMDAT linkage — plain
> extern, `extern "C"`, *any out-of-line definition* (member, static member,
> virtual), and anonymous-namespace functions not declared `static`; …

### 1.1 Sources

`a5c1_externC_inline_unref/main.cpp`:

```cpp
extern "C" inline int cand(int x) { return x*3+1; }
extern int sink(int);
int anchor(int x) { return sink(x) + 3; }
```

`a5c4_extern_then_inline_unref/main.cpp`:

```cpp
extern int cand(int x);
inline int cand(int x) { return x*3+1; }
extern inline int cand2(int x) { return x*7+4; }
extern int sink(int);
int anchor(int x) { return sink(x) + 3; }
```

Facts common to both, derived first because they are not in dispute:

* `anchor` is a definition with external linkage, not `inline`, defined out of
  line at namespace scope ⇒ a root under **either** reading ⇒ emitted.
* `sink` is *declared*, never defined ⇒ there is no definition to keep; it is an
  undefined external, never a `.text` COMDAT leader. Not in any predicted set.
* `anchor`'s body calls only `sink`. Nothing in either TU ODR-uses `cand` or
  `cand2`. So propagation adds nothing; the entire question is rootness.
* C++ linkage facts used: an `inline` specifier on *any* declaration of a
  function makes the function inline for the whole TU (so `a5c4`'s `cand` is
  inline despite the earlier `extern` declaration); `extern inline` is inline
  with external linkage; `extern "C" inline` is inline with C language linkage
  and external linkage. MSVC gives every inline function definition it emits a
  COMDAT with Selection ANY. None of the three is `static`.

Name spelling (X360 `cl.exe` 16.00.11886.00, PPC — member/`__cdecl` convention
letter is `A`, not the x86 `E`). Where I am unsure of a decoration letter I give
the identity as well; a spelling difference is **not** a derivation
disagreement.

### 1.2 Reading A — the head governs, the dash-list is apposition

"external **non-COMDAT** linkage" is the test; the items after the em dash are
example *shapes* of it.

| cell | predicted `.text` COMDAT leader set under Reading A |
|---|---|
| `a5c1` | `?anchor@@YAHH@Z` — that is all |
| `a5c4` | `?anchor@@YAHH@Z` — that is all |

Rationale: `cand` (a5c1) is inline ⇒ COMDAT linkage ⇒ fails the head test ⇒ not
a root; unreferenced ⇒ never added by propagation ⇒ absent. `cand`/`cand2`
(a5c4) likewise.

### 1.3 Reading B — the dash-list is an independent enumeration of spellings

Anything spelled `extern`, `extern "C"`, or out-of-line is a root regardless of
COMDAT-ness.

| cell | predicted set under Reading B |
|---|---|
| `a5c1` | `?anchor@@YAHH@Z`, `cand` (C linkage, undecorated on PPC; `_cand` if the target decorates C names — same identity) |
| `a5c4` | `?anchor@@YAHH@Z`, `?cand@@YAHH@Z`, `?cand2@@YAHH@Z` |

Note under Reading B both `cand`s qualify twice over: by spelling (`extern "C"`
/ `extern`) and by being out-of-line namespace-scope definitions.

### 1.4 Which reading does §2 *as written* mean? — I commit to **Reading A**

Five arguments, all textual:

1. **Grammar.** The head is a complete quantified predicate: "every definition
   *with external non-COMDAT linkage*". An em dash after a complete quantified
   noun phrase, followed by a comma list ending in "and X", is standardly
   appositive — it enumerates instances of the stated property. Reading B
   requires the dash to mean "or, additionally", which the punctuation does not
   support.
2. **`non-COMDAT` must do work.** Under Reading B the qualifier is inert: every
   root is decided by the list, and the head becomes decorative. A reading that
   renders an author's explicit, unusual, load-bearing qualifier meaningless is
   the worse reading. `non-COMDAT` is not a throwaway word — it is the one term
   in R1 that is not standard C++ vocabulary, i.e. the one the author had to
   choose deliberately.
3. **§2's own ontology.** §2's third corroboration (lines 125–129) says the obj's
   COMDAT Selection byte "encodes the linkage split … 1 (NODUPLICATES) for
   strong-linkage and kept statics, 2 (ANY) for COMDAT-linkage … **the model's
   linkage axis is exactly what determines it**". §2 therefore *has* a category
   called COMDAT-linkage and asserts its own model's root axis coincides with
   it. Only Reading A uses that category. Reading B makes §2 contradict its own
   corroboration paragraph.
4. **Consistency with what §2 claims to have refuted.** §2 reports refuting
   "closure-over-all-bodies" on 13 dead-referrer cells and reports zero
   violations across 172 cells crossing "8+ linkage values". Reading B roots
   *every* unreferenced inline definition spelled `extern`/`extern "C"` or
   written out of line — an aggressively inflationary rule. A rule that
   inflationary sitting at zero violations over a linkage-crossing grid is far
   less plausible than Reading A, so Reading A is the better reconstruction of
   what was actually fitted.
5. **Reading B has no stopping rule.** "any out-of-line definition" with no
   COMDAT filter makes *every* namespace-scope function definition a root
   (all of them are out of line), which would make R1 say "everything defined is
   emitted" and collapse the whole least-fixpoint apparatus into nothing. A
   reading that trivialises the surrounding structure is not the intended one.

**COMMITTED: §2 as written means Reading A.** My prediction for `a5c1` is
`{anchor}`; for `a5c4` it is `{anchor}`.

### 1.5 R1 is nevertheless internally inconsistent, independent of any cell

Recorded here, in Phase A, before truth: even granting Reading A, R1 contains a
clause that contradicts its own head *on its face*.

The list item "*any out-of-line definition* (member, static member, virtual)" is
italicised on **any**, and the intersection

> an out-of-line member function definition that is marked `inline`

is simultaneously
(a) "any out-of-line definition" ⇒ root by the list, and
(b) COMDAT linkage ⇒ not a root by the head.

No reordering of the sentence resolves this: the head excludes it and the
emphasised list item includes it. So R1's inconsistency is a property of the
*text*, provable without compiling anything, and `a5c1`/`a5c4` are not the cause
of it — they are two more instances of the same intersection reached through the
non-member items (`extern "C"` ∧ inline; `extern` ∧ inline). **This is a finding
about §2 that stands whatever the cells say.**

### 1.6 Decision rule for Phase B, fixed now

Let `T1 = truth(a5c1) ∩ {cand}` and `T4 = truth(a5c4) ∩ {cand, cand2}`.

| observed | consequence for R1 |
|---|---|
| `T1 = ∅` and `T4 = ∅` | Reading A holds; Reading B refuted; my derivation MATCHes both cells |
| `T1 ≠ ∅` and `T4` full | Reading B holds; Reading A refuted; both cells VIOLATE my committed derivation |
| **split** (one emits, the other does not) | **neither reading explains both** ⇒ R1 admits no consistent reading ⇒ defect in §2 in its own right, reported regardless of the violation count |

I bind myself to this table now. I will not reclassify a split as "Reading B was
right all along on the cell where it wins".

## 2. A2 — re-derivation of axes2's ten candidate violations

Grading universe throughout: `.text` COMDAT leader symbols. Vtables (`??_7`),
vbtables (`??_8`), RTTI (`??_R*`) are `.rdata` data COMDATs and are **outside**
the graded set even when §2's reasoning routes through them.

Two defect classes are distinguished per the brief:

* **[REACH]** — §2's text produces a definite prediction for the symbol and (if
  truth differs) is *wrong*. A genuine violation of §2-as-stated.
* **[GAP]** — §2's text contains **no clause that can produce the symbol at
  all** (`??_D` vbase destructors, `??_E` vector-deleting destructors, `??_9`
  vcall thunks, `W`-form / `$4` vtordisp adjustor thunks). §2 names the *scalar*
  deleting destructor and nothing else synthesized except `??__E`. A GAP is a
  coverage defect, strictly weaker than a REACH violation, and must not be
  scored as "§2 predicted X and X was false".

### a2_08 — explicit instantiation of a class template with virtuals, no object

```cpp
template <class T> struct V { virtual T f(T x){…} virtual T g(T x){…} virtual ~V(){} };
template struct V<int>;
int anchor(int x) { return sink(x) + 3; }
```

R2 roots "explicit instantiation definitions, including never-referenced
members". An explicit instantiation *definition* instantiates every member with
a definition, **including the implicitly-declared special members** (default
ctor, copy ctor). §2's "including never-referenced members" is unrestricted and
gives no basis for excluding implicit ones. So `V<int>::V()` is a kept
definition, and the Vtable rule then fires on it: a kept constructor ⇒ vtable ⇒
every virtual + the scalar deleting destructor.

**Predicted:** `?anchor@@YAHH@Z`; `?f@?$V@H@@UAAHH@Z`; `?g@?$V@H@@UAAHH@Z`;
`??1?$V@H@@UAA@XZ` (`~V<int>`); `??0?$V@H@@QAA@XZ` (default ctor);
`??0?$V@H@@QAA@ABV0@@Z` (implicit copy ctor); `??_G?$V@H@@UAAPAXI@Z`.

**Sub-reading recorded (this is a soft spot in §2):** if "members" in R2 is read
as *user-written* members only, no ctor is kept, the Vtable rule never fires,
and `??0`/`??_G` drop out while `f`/`g`/`~V` survive as roots in their own
right. I commit to the *inclusive* reading (implicit members counted) because
R2's "including never-referenced members" is an expansion, not a restriction.
Flag: **if axes2 took the exclusive reading, this cell is AMBIGUOUS by rule.**

### a3_03 — MI, virtual dtors in both bases, override of the second base

```cpp
struct A { virtual int f(int); virtual ~A(){} };  // bodies in-class
struct B { virtual int g(int); virtual ~B(){} };
struct D : A, B { virtual int g(int){…} };
int anchor(int x) { D o; use(&o); return sink(x); }
```

`anchor` root. `D o;` ODR-uses `D::D()` (implicit) and, at scope exit, `D::~D()`
(implicit, virtual). `D::D()` ODR-uses `A::A()` and `B::B()`; `D::~D()` ODR-uses
`A::~A()` and `B::~B()`. `use(&o)` takes the address of a local — no symbol.

Vtable rule applied to **every** kept constructor, as written ("a kept
constructor of C keeps C's vtable") — that includes base-subobject ctors:
`A::A()` ⇒ A's vtable ⇒ `A::f`, `A::~A`, `??_GA`; `B::B()` ⇒ `B::g`, `B::~B`,
`??_GB`; `D::D()` ⇒ D's virtuals (`D::g`, `D::~D`; `f`'s final override is
`A::f`, already present) + `??_GD`.

**Predicted:** `?anchor@@YAHH@Z`; `??0D@@QAA@XZ`; `??1D@@UAA@XZ`;
`?g@D@@UAAHH@Z`; `??_GD@@UAAPAXI@Z`; `??0A@@QAA@XZ`; `??1A@@UAA@XZ`;
`?f@A@@UAAHH@Z`; `??_GA@@UAAPAXI@Z`; `??0B@@QAA@XZ`; `??1B@@UAA@XZ`;
`?g@B@@UAAHH@Z`; `??_GB@@UAAPAXI@Z`.

**[GAP] declared in advance:** D has two vptrs. The B-subobject vtable needs
`this`-adjusting thunks for `D::g` and for the deleting destructor (`W`-form,
e.g. `?g@D@@W3AAHH@Z`, `??_GD@@W3AAPAXI@Z`). §2 has **no clause** that emits an
adjustor thunk. If truth carries them, that is a GAP, not a wrong prediction.

### a3_04 — virtual (shared) base, base's virtual not overridden

```cpp
struct A { virtual int f(int){…} virtual ~A(){} };
struct D : virtual A { virtual int g(int){…} };
int anchor(int x) { D o; use(&o); return sink(x); }
```

Same shape as a3_03 with one base, held virtually.

**Predicted:** `?anchor@@YAHH@Z`; `??0D@@QAA@XZ`; `??1D@@UAA@XZ`;
`?g@D@@UAAHH@Z`; `??_GD@@UAAPAXI@Z`; `??0A@@QAA@XZ`; `??1A@@UAA@XZ`;
`?f@A@@UAAHH@Z`; `??_GA@@UAAPAXI@Z`.

**[GAP]:** a virtual base makes MSVC synthesize the vbase destructor `??_DD@@…`
and the ctor closure/vbase-flag machinery. §2 cannot name `??_D`.

### a3_05 — virtual base whose virtual **is** overridden (vtordisp site)

```cpp
struct A { virtual int f(int){…} virtual ~A(){} };
struct D : virtual A { virtual int f(int){…} };
```

`D`'s virtual set is `{D::f, D::~D}`. `A::f` is still forced — not through D's
vtable, but because `A::A()` is a kept constructor and the Vtable rule applies
to A independently.

**Predicted:** `?anchor@@YAHH@Z`; `??0D@@QAA@XZ`; `??1D@@UAA@XZ`;
`?f@D@@UAAHH@Z`; `??_GD@@UAAPAXI@Z`; `??0A@@QAA@XZ`; `??1A@@UAA@XZ`;
`?f@A@@UAAHH@Z`; `??_GA@@UAAPAXI@Z`.

**[GAP]:** the vtordisp adjustor thunk for `D::f` (`$4…` mangling) and `??_D`.
Neither is reachable from §2's text.

### a3_06 — virtual-base diamond, final override from one middle class

```cpp
struct A { virtual int f(int){…} virtual ~A(){} };
struct B : virtual A { virtual int g(int){…} };
struct C : virtual A { virtual int f(int){…} };
struct D : B, C { };
```

`anchor` ⇒ `D::D()` (constructs the virtual base A itself, plus B and C) and
`D::~D()`. Four kept constructors ⇒ four vtables ⇒ union of all virtuals + four
scalar deleting destructors.

**Predicted:** `?anchor@@YAHH@Z`; `??0D@@QAA@XZ`, `??1D@@UAA@XZ`,
`??_GD@@UAAPAXI@Z`; `??0B@@QAA@XZ`, `??1B@@UAA@XZ`, `?g@B@@UAAHH@Z`,
`??_GB@@UAAPAXI@Z`; `??0C@@QAA@XZ`, `??1C@@UAA@XZ`, `?f@C@@UAAHH@Z`,
`??_GC@@UAAPAXI@Z`; `??0A@@QAA@XZ`, `??1A@@UAA@XZ`, `?f@A@@UAAHH@Z`,
`??_GA@@UAAPAXI@Z`.

**[GAP]:** `??_D` for B, C, D; vtordisp/adjustor thunks for `C::f` in D's
layout; secondary-vtable `W`-form thunks. All unreachable from §2.

**Standing note on A3.** All four A3 cells are structurally the same
prediction and their most likely failure mode is *extra* synthesized symbols in
truth. Under §2's own vocabulary those are GAPs. **An axis broken only by GAPs
should not be scored the same as an axis broken by a REACH violation**, and I
will say so in the ruling.

### a4_09 — anon-ns dynamic initializer reaching an anon-ns `static` and an anon-ns non-static

```cpp
namespace {
static int helper(int x) { return x*3+1; }
int seed(int x) { return x+7; }
int g_v = helper(2) + seed(3);
}
int anchor(int x) { return sink(x) + 3; }
```

* `anchor` — R1 root.
* `seed` — R1's final item, "anonymous-namespace functions **not declared
  `static`**", verbatim. Root. Emitted **whether or not it is called.**
* `g_v` — R5 "kept data definitions — … non-const internal data". Kept.
* Its initializer is a call expression, not a constant expression ⇒ a dynamic
  initializer ⇒ R4 roots the `??__E` thunk.
* Propagation from the kept thunk: it calls `helper` and `seed` ⇒ both added.
  `helper` is `static` so it is not a root, but propagation is enough.
* **Pre-optimization is decisive here and §2 is explicit about it** — "a call
  anywhere in the *pre-optimization* body", and §2 lists "post-optimization
  reference sets" among the things it refuted. At `/O1 /Oi` both helpers are
  trivially inlinable into the thunk; §2 nonetheless commits to keeping them.

**Predicted:** `?anchor@@YAHH@Z`; `?seed@?A0x<hash>@@YAHH@Z`;
`?helper@?A0x<hash>@@YAHH@Z`; `??__E?g_v@?A0x<hash>@@YAXXZ`
(anon-namespace unique tag spelled `?A0x<hash>` — identity, not spelling).

**Class if truth differs:** **[REACH]** either way. If `helper` is absent, §2's
pre-optimization clause is violated. If `seed` is absent, R1's anon-namespace
item is violated. Both are clauses §2 states outright.

### a9_04 — reference form of `dynamic_cast`

```cpp
struct B { virtual int f(int); virtual ~B(); };      // declared, NOT defined here
struct D : B { virtual int f(int){…} virtual int g(int){…} };
int anchor(B& b, int x) { D& d = dynamic_cast<D&>(b); return sink(d.f(x)); }
```

* `anchor` — root: `?anchor@@YAHAAUB@@H@Z`.
* `B::f`, `B::~B` — declared only; **no definition in this TU** ⇒ nothing to
  emit.
* **No constructor of D is kept** — no D is ever constructed. So the Vtable rule
  does **not** fire. `D::g` is never mentioned ⇒ absent. `??_GD` is not forced.
* `d.f(x)` — §2's propagation clause: "a call anywhere in the pre-optimization
  body". Read literally, this is a call whose named callee is `D::f`, whose
  definition is in this TU and is kept-able. **§2-as-stated adds `D::f`.**

**Predicted:** `?anchor@@YAHAAUB@@H@Z`, `?f@D@@UAAHH@Z`.

**Class:** **[REACH]**. If truth omits `?f@D@@UAAHH@Z`, the propagation clause
is refuted for virtual calls: the call ODR-uses the *slot*, and with no vtable
kept there is nothing to force the definition. §2's text draws no distinction
between virtual and non-virtual calls, so it cannot be defended as "outside
scope".

### a9_05 — out-of-line virtual destructor is the only root touching the class

```cpp
struct D { virtual int f(int){…} virtual int g(int){…} virtual ~D(); };
D::~D() { }
int anchor(int x) { return sink(x) + 3; }
```

* `anchor` — root.
* `D::~D()` — an **out-of-line definition** (member, virtual), not `inline`,
  external linkage ⇒ root under Reading A *and* Reading B. `??1D@@UAA@XZ`
  emitted. (This is the one cell where the R1 dispute does not matter.)
* Vtable rule requires a **kept constructor**. There is none — no D is
  constructed anywhere. §2 therefore does **not** keep D's vtable, and `D::f`,
  `D::g`, `??_GD` are **not** predicted.
* §2 offers no other route: the destructor body's vptr store is not a "call, an
  address-take, or a data initializer" of `f`/`g`, and §2 has no
  "vtable-reference keeps the vtable" clause.

**Predicted:** `?anchor@@YAHH@Z`, `??1D@@UAA@XZ`.

**Class:** **[REACH]**, and it is the sharpest cell in the set. The plan itself
(prereg, axis A9) says D6 "could break the ctor⇒vtable formulation". If truth
emits `f`, `g`, `??_G`, then §2's Vtable rule is **under-inclusive**: the
vtable-forcing event is "a kept definition that installs the vptr", of which the
constructor is only one instance — the destructor is another.

### a9_06 — `delete` through a pointer, no constructor kept

```cpp
struct D { virtual int f(int){…} virtual int g(int){…} virtual ~D() {} };
int anchor(D* p, int x) { delete p; return sink(x) + 3; }
```

* `anchor` — root: `?anchor@@YAHPAUD@@H@Z`.
* No constructor kept ⇒ Vtable rule silent ⇒ `f`, `g` not predicted.
* `delete p` — §2's propagation says "a call anywhere in the pre-optimization
  body". The pre-optimization body of `anchor` contains a call to the
  destructor of `D`. §2's only named destructor vocabulary is `D::~D` and "the
  synthesized scalar-deleting destructor". Read literally, `delete p` is a call
  to `D::~D`, whose definition (in-class, hence inline/COMDAT) exists here ⇒
  **§2-as-stated adds `??1D@@UAA@XZ`**.
* `??_GD` — §2 produces the scalar deleting destructor **only** as a vtable slot
  under a kept constructor. With no kept constructor, §2 has **no clause** that
  emits `??_G` here, even though `delete p` through a virtual destructor is
  exactly what calls it.

**Predicted:** `?anchor@@YAHPAUD@@H@Z`, `??1D@@UAA@XZ`.

**Class:** mixed, and the two halves must be reported separately.
– If truth lacks `??1D` ⇒ **[REACH]**, same virtual-call-uses-the-slot mechanism
as a9_04 (the delete dispatches through the vtable slot to `??_G`; nothing forces
the definition).
– If truth carries `??_GD` ⇒ **[GAP]** on `??_G`-without-a-ctor.
– If truth carries `f`/`g` ⇒ **[REACH]**, same under-inclusive Vtable rule as
a9_05.

### a9_07 — address-take of a virtual member function, no constructor kept

```cpp
struct D { virtual int f(int){…} virtual int g(int){…} };
typedef int (D::*PMF)(int);
PMF anchor() { return &D::f; }
```

* `anchor` — root: `?anchor@@YAP8D@@AAHH@ZXZ`.
* `&D::f` — §2's propagation names **"an address-take"** explicitly. `D::f` has
  a definition in this TU. **§2-as-stated adds `?f@D@@UAAHH@Z`.**
* `D::g` — untouched, no ctor kept, no vtable ⇒ absent.
* D has no user destructor and none is used ⇒ no `??_G` from §2.

**Predicted:** `?anchor@@YAP8D@@AAHH@ZXZ`, `?f@D@@UAAHH@Z`.

**Class:** **[REACH]** on the `D::f` half — the address of a *virtual* member
function is not the address of its definition; MSVC materialises a vcall thunk
`??_9D@@$BA@AA` and the pointer-to-member value denotes the **slot**. If truth
shows the thunk in place of `D::f`, §2's address-take clause is refuted for
virtuals. Additionally **[GAP]** on `??_9`, which §2's text cannot name at all —
`??_9` appears in §2 only in the standing-caveat sentence about real headers
being "out of the grid", i.e. §2 already concedes it does not model them.

### 2.1 Mechanism grouping recorded before Phase B

`a9_04`, `a9_06`, `a9_07` all instantiate **one** mechanism: *a virtual entity
reached through the dispatch machinery (virtual call, virtual delete,
pointer-to-member-of-virtual) references the vtable slot, not the definition,
so §2's propagation clause over-predicts.* `a9_05` is a **different** mechanism:
*the vtable is forced by vptr-installing definitions generally, not only by
constructors, so §2's Vtable rule under-predicts.* I record this grouping now
so that my answer to the V6-weighting question (lead's question 3) is not
retrofitted to the count it produces.

## 3. Pre-committed rulings on the lead's questions (written before Phase B)

Written now, before any truth artifact is read, because two of them are
questions where knowing the answer would let me choose the rule that produces
the answer I like.

### 3.1 Is the COMDAT Selection byte admissible? — **admissible as corroboration, NOT as an interpretation of §2**

The distinction the lead asks for is real and it decides the question.

The Selection byte is an **observation of what c2 did**. It is produced by the
same process that produces the emit set; it is not an independent oracle for
what §2's *words* mean. §2's authority over its own text is not delegated to
c2's output — if it were, §2 could never be violated at all, since any
disagreement would be re-read as "§2 must have meant whatever c2 did". That is
exactly the goalpost-move guard 1 exists to block, and it would block the lane's
own refutations too.

So: **the Selection byte cannot make Reading A true.** What it *can* do:

1. It shows §2's stated corroboration is **self-consistent under Reading A and
   self-contradictory under Reading B.** §2 asserts (lines 125–129) that
   Selection tracks "the model's linkage axis". If `cand`/`cand2` measure
   Selection 2 (ANY) — i.e. c2 classifies them as COMDAT-linkage — then under
   Reading B §2 roots symbols that its own nominated observable places in the
   non-root class. That is a **textual** finding about §2's internal coherence,
   which is admissible, because it is an argument about §2's words using §2's
   own claimed correspondence.
2. It is therefore **evidence about what §2 means only in the weak, structural
   sense** that Reading B makes §2 inconsistent with itself. That is the fifth
   argument in §1.4 restated in stronger form, and it points the same way.

**Ruling, pre-committed:** the Selection byte is admitted, weighted as
corroboration of the reading commitment already made in §1.4 on text-only
grounds, and it is **not** by itself sufficient to convert an AMBIGUOUS cell to
a VIOLATION. An AMBIGUOUS grade is a statement about a *disagreement between two
derivations*; a byte in the obj cannot repair that disagreement, because the
disagreement is about English, not about the object file. If axes1 and I both
land on Reading A, the cell is a VIOLATION on our agreement — and the Selection
byte is then a nice-to-have. If axes1 landed on Reading B, the cell stays
AMBIGUOUS and the Selection byte is reported as supporting evidence, not as the
grade. **The prereg says AMBIGUOUS is automatic on disagreement, "regardless of
which of you truth favours"; a c2-derived byte is truth, and does not get a
carve-out.**

Note recorded on axes1's conduct: axes1 graded its own cell AMBIGUOUS as
pre-registered and reported the stronger evidence *separately* rather than
re-grading upward. That is the guard working as designed, from the biased
agent's own side, and it should be credited as such.

### 3.2 V6 weighting — **A6 and A9 count as one mechanism found twice, but as two axes for V6**

Pre-committed reasoning, before I know the counts:

V6 is registered as "**axes** (of 9) with ≥ 1 confirmed violation of
§2-as-stated", point 1, interval [0, 3]. Its unit is *axes*, and its purpose in
Part 2 is stated in the scoring paragraph: "any confirmed violation ⇒ the axis
is **BROKEN** … A broken axis blocks SHIP-CANDIDATE unless the breaking
condition is demonstrated detectable from c1xx-side observables." V6 is a
**coverage-of-the-structure-space** metric feeding a **ship gate**, not a count
of distinct causes. An axis with a confirmed violation is unsafe to ship
regardless of whether its cause was already seen elsewhere, and the registered
definition is mechanical. Re-defining V6's unit from "axes" to "mechanisms"
after seeing that two axes share a cause is a post-hoc goalpost-move that
happens to lower the score — the deflation-of-the-deflationary-lane error is
still an error.

**So: V6 counts axes, and A6 and A9 each count if each has ≥ 1 confirmed
violation, even if the mechanism is identical.**

But the *interpretation* must say plainly what the axis count does not: that
A6's `a6c5` and A9's `a9_04`/`a9_06`/`a9_07` are **one defect in §2 with one
repair**. This is the more important fact for R3, and burying it inside a "2 of
9 axes broken" headline would misinform. The independent replication across two
agents who could not see each other's work is *strong evidence the mechanism is
real* — which raises confidence in the finding while lowering the number of
distinct repairs §2 needs. Both statements go in the ruling.

## 4. Freeze

This file is frozen at this point. Nothing below the line is written until the
lead confirms it is committed. Contamination on `a5c4` is disclosed in §0 and
carried forward into the ruling.
