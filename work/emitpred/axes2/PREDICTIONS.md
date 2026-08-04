# axes2 — pre-compile predictions for A2 / A3 / A4 / A9

Agent `axes2`, lane `w-emitpred`, worktree `wt-w-emitpred`.
**Written before any axis cell was compiled.** The only compile performed before
this file was frozen was one throwaway smoke cell
(`work/emitpred/axes2/smoke/smoke.cpp`, not an axis cell) used solely to confirm
the wibo/cl invocation and the COMDAT reader.

Cells: `work/emitpred/axes2/cells/<axis>/<cell>/cell.cpp`
Flags at grade time: `/O1 /Oi /EHsc /GS- /c` (workload flags, no additions).
Ground truth at grade time: the obj's `.text` COMDAT leader symbol set
(`work/phase7plan-d/../probes/coffsyms.py`); `/FAsc` `.cod` PROC names are a
cross-check only.

## Source of the predictions

Every predicted set below is derived **only** from `PHASE7_PLAN.md` §2, quoted
here as the operative text (clause labels are mine, used throughout):

* **R1** roots: "every definition with external non-COMDAT linkage — plain
  extern, `extern "C"`, *any out-of-line definition* (member, static member,
  virtual), and anonymous-namespace functions not declared `static`"
* **R2** roots: "explicit instantiation definitions, including never-referenced
  members"
* **R3** roots: `__declspec(dllexport)` closure (unused by these cells)
* **R4** roots: "dynamic-initializer thunks (`??__E`)"
* **R5** roots: "kept data definitions — external-linkage data and non-const
  internal data"
* **P** propagation: "F is added if an already-kept definition ODR-uses it — a
  call anywhere in the pre-optimization body …, an address-take, or a data
  initializer. `sizeof` does not count. References from removed (never-kept)
  definitions never count."
* **VT** vtable rule: "a kept constructor of C keeps C's vtable, whose slots
  force **every** virtual of C plus the synthesized scalar-deleting destructor,
  called or not."

## Naming conventions used in the predictions

Verified from pre-existing lane-A probe artifacts (`work/probes/*.cod`), i.e.
established facts about this target's decoration, not about emission:

* free function: `?anchor@@YAHH@Z`
* non-virtual public member: `?a@H@@QAAHH@Z` (calling-convention letter `A` =
  `__cdecl`; PPC has no `__thiscall`)
* virtual public member: `?f@A@@UAAHH@Z`
* constructor / destructor: `??0A@@QAA@XZ` / `??1A@@UAA@XZ` (or `QAA` when
  non-virtual)
* scalar deleting destructor: `??_GA@@UAAPAXI@Z`
* unnamed namespace: `?cand@?A0x########@@YAHH@Z` (`########` = a per-TU hex
  hash; predictions write `?A0x#` and match any hash)
* class template specialization: `?$H@H` for `H<int>`; function template
  specialization `??$cand@H@@YAHH@Z`

Grading is on **set identity of the emitted function entities**. Where I am
unsure of a decoration detail (never of the entity), the entity description is
authoritative and the decoration is advisory; those cases are flagged inline.

`extern int sink(int);` and `extern void use(void*);` are declared-only in every
cell: they have no definition here, so they can never appear in the emitted set.
They exist to force ODR-uses that survive the `pre-optimization` reading.

---

# Axis A2 — templates (9 cells)

## a2_01_extern_template_member_called
**Structural point:** explicit instantiation *declaration* (`extern template`) —
a construct §2's root list does not mention at all — with one member ODR-used.
**§2 clauses:** R1 (`anchor`), P (`anchor` calls `H<int>::a`).
**Derivation:** `anchor` is external non-COMDAT ⇒ root. Its pre-optimization body
calls `H<int>::a` ⇒ kept. `H<int>::b` is never ODR-used and there is no explicit
instantiation *definition*, so R2 does not apply ⇒ dropped. `H` has no virtuals
⇒ VT idle. `H<int> h` uses a trivial implicit default constructor (no body).
**Predicted (2):**
1. `?anchor@@YAHH@Z`
2. `?a@?$H@H@@QAAHH@Z`

**STRETCH — §2 is silent on `extern template`.** The literal reading simply
ignores the declaration, so the prediction is identical to the no-`extern
template` case. If the toolchain instead suppresses `H<int>::a` (emitting only
`anchor`), §2 as stated is missing a root-suppressing clause. Recorded now so
the grade cannot be re-derived after the fact.

## a2_02_extern_template_virtual_object
**Structural point:** `extern template` on a class template *with virtuals*,
with a constructor ODR-used — VT vs a construct §2 does not model.
**§2 clauses:** R1 (`anchor`), P (`anchor` constructs `V<int>` and destroys it),
VT (kept ctor ⇒ vtable ⇒ every virtual + synthesized `??_G`).
**Predicted (6):**
1. `?anchor@@YAHH@Z`
2. `??0?$V@H@@QAA@XZ` — implicit default ctor of `V<int>` (non-trivial: sets vptr)
3. `?f@?$V@H@@UAAHH@Z`
4. `?g@?$V@H@@UAAHH@Z`
5. `??1?$V@H@@UAA@XZ`
6. `??_G?$V@H@@UAAPAXI@Z` — synthesized scalar deleting destructor

**STRETCH:** as a2_01, `extern template` is unmodelled by §2. Also: VT says
the vtable is forced by the kept constructor, which presupposes the implicit
default constructor of `V<int>` is itself emitted; predicted as such.

## a2_03_explicit_fn_specialization_unref
**Structural point:** an explicit **specialization** (`template <> int
cand<int>(int)`), never referenced. §2's R2 covers explicit *instantiation
definitions*; a specialization is neither that nor an implicit instantiation.
**§2 clauses:** R1 ("*any out-of-line definition*"), R1 (`anchor`).
**Derivation:** the specialization is a namespace-scope, out-of-line, external,
non-`inline` definition. R1's "any out-of-line definition" clause makes it a
root, unreferenced or not.
**Predicted (2):**
1. `?anchor@@YAHH@Z`
2. `??$cand@H@@YAHH@Z`

**STRETCH — the sharpest statement-looseness cell in A2.** R1's parenthetical
enumerates "(member, static member, virtual)", i.e. class members; a rival
reading confines "any out-of-line definition" to those and treats an explicit
function-template specialization as COMDAT like an implicit instantiation,
predicting **1** (`anchor` only). I take the literal, wider reading because the
clause says "any". If truth = 1, §2's R1 is over-broad as written.

## a2_04_explicit_member_specialization_unref
**Structural point:** explicit specialization of *one member* of a class
template, never referenced, alongside a normally-instantiated sibling.
**§2 clauses:** R1 (`anchor`; the out-of-line member specialization), P.
**Predicted (3):**
1. `?anchor@@YAHH@Z`
2. `?a@?$H@H@@QAAHH@Z` — ODR-used by `anchor`
3. `?b@?$H@H@@QAAHH@Z` — root by R1 "any out-of-line definition (member…)"

**STRETCH:** same reading question as a2_03; here R1's parenthetical *does*
name "member", so the literal reading is stronger. Rival reading predicts 2.

## a2_05_class_tmpl_virtuals_object
**Structural point:** control for a2_02 — identical source minus the
`extern template` line. Isolates whether any a2_02 divergence is attributable to
`extern template` rather than to templates-with-virtuals generally.
**§2 clauses:** R1, P, VT.
**Predicted (6):** identical to a2_02's list (items 1–6).

## a2_06_template_over_template
**Structural point:** a template instantiated over another template
(`Out<int>` holding `In<int>`), with exactly one path through the nest ODR-used.
**§2 clauses:** R1 (`anchor`), P (two propagation steps through implicitly
instantiated members).
**Predicted (3):**
1. `?anchor@@YAHH@Z`
2. `?call@?$Out@H@@QAAHH@Z`
3. `?get@?$In@H@@QAAHH@Z`

`Out<int>::never` and `In<int>::unused` are never ODR-used from a kept
definition ⇒ dropped.

## a2_07_explicit_inst_outer_forces_inner
**Structural point:** an explicit instantiation definition of the OUTER template
only; its never-referenced member is the sole path to the inner template's
never-referenced member. Tests that R2 ("including never-referenced members")
feeds P transitively across a template boundary.
**§2 clauses:** R1 (`anchor`), R2 (`Out<int>` incl. never-referenced members),
P (from those members into `In<int>`).
**Predicted (5):**
1. `?anchor@@YAHH@Z`
2. `?call@?$Out@H@@QAAHH@Z`
3. `?never@?$Out@H@@QAAHH@Z`
4. `?get@?$In@H@@QAAHH@Z`
5. `?unused@?$In@H@@QAAHH@Z`

**STRETCH:** R2 makes `Out<int>`'s members roots; it says nothing about whether
the *inner* template's members reached from them are implicitly instantiated.
The literal reading routes them in through P, which §2 does license
unconditionally over kept definitions.

## a2_08_explicit_inst_virtuals_no_object
**Structural point:** the A2×A9 crossing — an explicit instantiation definition
of a class template with virtuals, where **no object is ever constructed**.
Tests whether R2's "including never-referenced members" reaches the implicitly
declared special members, and therefore whether VT fires with no user-written
constructor use.
**§2 clauses:** R1 (`anchor`), R2, VT.
**Derivation (literal):** R2 makes every member of `V<int>` a root, including
the implicitly declared special members (default ctor, copy ctor, copy
assignment) — "including never-referenced members" has no carve-out. The kept
default constructor then triggers VT.
**Predicted (8):**
1. `?anchor@@YAHH@Z`
2. `??0?$V@H@@QAA@XZ` — implicit default ctor
3. `??0?$V@H@@QAA@ABU?$V@H@@@Z` — implicit copy ctor (decoration advisory)
4. `??4?$V@H@@QAAAAU?$V@H@@ABU0@@Z` — implicit copy assignment (decoration
   advisory; the pre-existing probe corpus shows the shape `??4A@@QAAAAU0@ABU0@@Z`)
5. `?f@?$V@H@@UAAHH@Z`
6. `?g@?$V@H@@UAAHH@Z`
7. `??1?$V@H@@UAA@XZ`
8. `??_G?$V@H@@UAAPAXI@Z`

**STRETCH (two, both recorded):** (i) whether R2's "members" includes
*implicitly declared* special members. If it does not, the literal core is
**5** = {`anchor`, `f`, `g`, `~V`, and no `??_G`} because VT would never fire.
(ii) whether VT's "kept constructor" is satisfied by a constructor kept as a
root rather than by an ODR-use. I predict the 8-name set as §2's most literal
reading and register the 5-name and 4-name alternatives explicitly.

## a2_09_extern_template_plus_member_inst
**Structural point:** `extern template` on the class *plus* a member-level
explicit instantiation definition of the never-referenced member — R2 applied
inside a construct §2 does not model.
**§2 clauses:** R1 (`anchor`), P (`anchor` calls `a`), R2 (member `b`).
**Predicted (3):**
1. `?anchor@@YAHH@Z`
2. `?a@?$H@H@@QAAHH@Z`
3. `?b@?$H@H@@QAAHH@Z`

**STRETCH:** §2 silent on `extern template`; also silent on whether a
member-level explicit instantiation definition interacts with a class-level
declaration. Literal reading: R2 applies to the member, `extern template` is a
no-op. If the cell fails to compile under this toolchain that is reported as a
cell-level INSTRUMENT note, not as a violation.

---

# Axis A3 — virtual & multiple inheritance (8 cells)

Two readings of VT recur across this axis and are declared once here, applied
uniformly below:

* **VT-scope.** "every virtual of C" is read **literally and widely**: every
  virtual member function of C, *including* those inherited from a base and not
  overridden (they are virtual members of C, and they occupy slots in C's
  vtable — VT's own justification is "whose slots force"). Rival narrow reading:
  only virtuals *declared* in C.
* **VT-dtor.** "plus the synthesized scalar-deleting destructor" is stated
  **unconditionally**, with no requirement that any destructor be virtual. I
  predict `??_G` for every class whose constructor is kept, including classes
  with non-virtual destructors (a3_01, a3_02). Rival reading: `??_G` exists only
  where the destructor is virtual.
* **VT-recursion.** A kept constructor of C ODR-uses the constructors of its
  bases (P: "a call anywhere in the pre-optimization body"), so VT applies again
  to each base. Predictions include base vtable closures.
* **Thunks.** §2 contains **no clause about adjustor thunks, vcall thunks,
  vtordisp thunks, virtual-base destructors (`??_D`), or vbtables.** The literal
  reading therefore predicts **none of them**. Any such symbol observed in the
  `.text` COMDAT leader set is an unpredicted emission and, under this
  pre-registration, a violation of §2-as-stated (its detectability is then the
  question that decides whether the axis is guardable).

## a3_01_mi_override_second_base
**Structural point:** MI where D overrides the **second** base's virtual — the
canonical MSVC adjustor-thunk site. Destructors are non-virtual throughout, so
this is also the sharpest VT-dtor test.
**§2 clauses:** R1 (`anchor`), P (construct `D`; D's ctor calls A's and B's),
VT ×3 (D, A, B).
**Predicted (10):**
1. `?anchor@@YAHH@Z`
2. `??0D@@QAA@XZ`
3. `??0A@@QAA@XZ`
4. `??0B@@QAA@XZ`
5. `?f@A@@UAAHH@Z` — a virtual of D (inherited, not overridden) and of A
6. `?g@D@@UAAHH@Z` — D's override
7. `?g@B@@UAAHH@Z` — forced by A/B closure via B's own vtable
8. `??_GD@@UAAPAXI@Z`
9. `??_GA@@UAAPAXI@Z`
10. `??_GB@@UAAPAXI@Z`

**STRETCH:** VT-scope (item 5), VT-dtor (items 8–10 with non-virtual dtors),
and §2's silence on the second vtable and on the adjustor thunk for `D::g` in
B's subobject vtable (predicted absent).

## a3_02_mi_no_override_second_base
**Structural point:** control for a3_01 — MI with no override anywhere, so no
adjustor thunk is structurally required. Any thunk difference between a3_01 and
a3_02 isolates the override as the cause.
**§2 clauses:** as a3_01.
**Predicted (10):**
1. `?anchor@@YAHH@Z`
2. `??0D@@QAA@XZ`
3. `??0A@@QAA@XZ`
4. `??0B@@QAA@XZ`
5. `?f@A@@UAAHH@Z`
6. `?g@B@@UAAHH@Z`
7. `?h@D@@UAAHH@Z`
8. `??_GD@@UAAPAXI@Z`
9. `??_GA@@UAAPAXI@Z`
10. `??_GB@@UAAPAXI@Z`

**STRETCH:** VT-scope (5, 6), VT-dtor (8–10).

## a3_03_mi_virtual_dtors
**Structural point:** MI with virtual destructors in both bases plus an override
of the second base's virtual — VT-dtor is no longer a stretch, so a divergence
here is attributable to MI structure rather than to the `??_G` clause.
**§2 clauses:** R1, P, VT ×3.
**Predicted (13):**
1. `?anchor@@YAHH@Z`
2. `??0D@@QAA@XZ`
3. `??0A@@QAA@XZ`
4. `??0B@@QAA@XZ`
5. `?f@A@@UAAHH@Z`
6. `?g@D@@UAAHH@Z`
7. `?g@B@@UAAHH@Z`
8. `??1D@@UAA@XZ`
9. `??1A@@UAA@XZ`
10. `??1B@@UAA@XZ`
11. `??_GD@@UAAPAXI@Z`
12. `??_GA@@UAAPAXI@Z`
13. `??_GB@@UAAPAXI@Z`

**STRETCH:** VT-scope (5, 7); thunks (D::g and D's destructor in B's subobject
vtable) predicted absent.

## a3_04_virtual_base_simple
**Structural point:** virtual (shared) base with no override — vbase
construction machinery, which §2 does not model.
**§2 clauses:** R1, P, VT ×2 (D, A).
**Predicted (9):**
1. `?anchor@@YAHH@Z`
2. `??0D@@QAA@XZ`
3. `??0A@@QAA@XZ`
4. `?f@A@@UAAHH@Z`
5. `?g@D@@UAAHH@Z`
6. `??1D@@UAA@XZ`
7. `??1A@@UAA@XZ`
8. `??_GD@@UAAPAXI@Z`
9. `??_GA@@UAAPAXI@Z`

**STRETCH:** §2 says nothing about the virtual-base destructor (`??_DD@@…`),
the vbtable, the vbase-ctor's hidden most-derived flag, or vector iterators;
all predicted absent from `.text`.

## a3_05_vbase_virtual_overridden
**Structural point:** a virtual base whose virtual **is** overridden in the
derived class — the vtordisp adjustor site.
**§2 clauses:** R1, P, VT ×2. Note `A::f` is not a virtual *of D* (D overrides
it), but A's own kept constructor forces A's vtable, which forces `A::f`.
**Predicted (9):**
1. `?anchor@@YAHH@Z`
2. `??0D@@QAA@XZ`
3. `??0A@@QAA@XZ`
4. `?f@D@@UAAHH@Z`
5. `?f@A@@UAAHH@Z`
6. `??1D@@UAA@XZ`
7. `??1A@@UAA@XZ`
8. `??_GD@@UAAPAXI@Z`
9. `??_GA@@UAAPAXI@Z`

**STRETCH:** the vtordisp thunk for `D::f` (MSVC's `$4…`/`$R…` adjustor form)
is predicted absent — §2 has no clause for it.

## a3_06_diamond
**Structural point:** the full virtual-base diamond; the final override of `A::f`
comes from one middle class, so D's vtable, B's, C's and A's all differ.
**§2 clauses:** R1, P, VT ×4 (D, B, C, A).
**Derivation:** virtuals of D = `f` (final override `C::f`), `g` (`B::g`), `~D`.
Virtuals of B = `f` (`A::f`), `g`, `~B`. Of C = `f` (`C::f`), `~C`. Of A = `f`,
`~A`. Union of function *definitions* forced: `A::f`, `C::f`, `B::g`, the four
destructors, the four `??_G`.
**Predicted (16):**
1. `?anchor@@YAHH@Z`
2. `??0D@@QAA@XZ`
3. `??0B@@QAA@XZ`
4. `??0C@@QAA@XZ`
5. `??0A@@QAA@XZ`
6. `?f@A@@UAAHH@Z`
7. `?f@C@@UAAHH@Z`
8. `?g@B@@UAAHH@Z`
9. `??1D@@UAA@XZ`
10. `??1B@@UAA@XZ`
11. `??1C@@UAA@XZ`
12. `??1A@@UAA@XZ`
13. `??_GD@@UAAPAXI@Z`
14. `??_GB@@UAAPAXI@Z`
15. `??_GC@@UAAPAXI@Z`
16. `??_GA@@UAAPAXI@Z`

**STRETCH:** VT-scope; thunks/`??_D`/vbtable predicted absent; also whether the
implicit destructors of B, C, D exist as separate emitted bodies at all.

## a3_07_mi_outofline_virtuals_no_object
**Structural point:** MI with out-of-line virtual definitions and **no object
constructed anywhere** — isolates R1 from VT under MI. The in-sample cell
`v12_outofline_one_inline_one` is the single-inheritance precedent; this asks
whether MI adds anything (a thunk for `D::g` in B's subobject) even with no
vtable forced.
**§2 clauses:** R1 only ("any out-of-line definition (… virtual)").
**Predicted (3):**
1. `?anchor@@YAHH@Z`
2. `?f@A@@UAAHH@Z`
3. `?g@D@@UAAHH@Z`

`A::inl` is in-class (COMDAT), never ODR-used, no constructor kept ⇒ VT idle ⇒
dropped. No `??_G` for any class. No thunk predicted.

## a3_08_si_inherited_inline_virtual
**Structural point:** the cleanest VT-scope test — single inheritance, D's
constructor kept, and a base virtual (`A::only_a`) that D neither overrides nor
calls and that nothing else references. Under the wide reading it is emitted;
under the narrow reading ("virtuals declared in C") it is emitted only via A's
own vtable, which is also forced here — so this cell additionally checks whether
the *base's* vtable closure fires at all.
**§2 clauses:** R1, P, VT ×2 (D, A).
**Predicted (8):**
1. `?anchor@@YAHH@Z`
2. `??0D@@QAA@XZ`
3. `??0A@@QAA@XZ`
4. `?f@A@@UAAHH@Z`
5. `?only_a@A@@UAAHH@Z`
6. `?g@D@@UAAHH@Z`
7. `??_GD@@UAAPAXI@Z`
8. `??_GA@@UAAPAXI@Z`

**STRETCH:** VT-dtor (7, 8 — destructors here are non-virtual).

---

# Axis A4 — anonymous namespaces (9 cells)

The operative R1 sub-clause is **"anonymous-namespace functions not declared
`static`"**. Two readings recur:

* **A4-wide** (taken as literal throughout): *any* function whose enclosing
  scope chain includes an unnamed namespace, and which is not declared `static`,
  is a root — including class members and `inline` functions, since the clause
  states no exception.
* **A4-narrow** (the rival): only namespace-scope free functions written
  directly in an unnamed namespace.

`ns05_anon_ns_inline_unref` and `ns02_anon_ns_class_member_unref` already exist
in the in-sample 172-cell grid; **their compiled truth was deliberately not
read** while deriving these predictions, precisely so the reading question stays
open here. That is recorded as a protocol fact, not as an excuse.

## a4_01_nested_anon_ns
**Structural point:** an unnamed namespace nested *directly* inside an unnamed
namespace.
**§2 clauses:** R1 (`anchor`), R1 anon-ns sub-clause (`cand`).
**Predicted (2):**
1. `?anchor@@YAHH@Z`
2. the unreferenced `cand(int)`, qualified by **two** nested unnamed-namespace
   scopes — decoration `?cand@?A0x#@?A0x#@@YAHH@Z` (two `?A0x` hashes;
   decoration advisory, entity authoritative)

**STRETCH:** §2 says "anonymous-namespace functions" without addressing nesting.
Literal reading: still an anonymous-namespace function ⇒ root.

## a4_02_named_ns_inside_anon_ns
**Structural point:** a **named** namespace nested inside an unnamed one — the
function's own immediately-enclosing namespace is named.
**§2 clauses:** R1 (`anchor`), R1 anon-ns sub-clause (`N::cand`) under A4-wide.
**Predicted (2):**
1. `?anchor@@YAHH@Z`
2. `?cand@N@?A0x#@@YAHH@Z` — the unreferenced `N::cand(int)`

**STRETCH — significant.** `N::cand` is not written *in* an unnamed namespace,
only *inside* one. A4-wide makes it a root (predicted); A4-narrow drops it,
predicting **1**. Its linkage is internal either way, so under A4-narrow it
would behave like an unreferenced `static`.

## a4_03_anon_ns_inside_named_ns
**Structural point:** the mirror of a4_02 — an unnamed namespace nested inside a
named one. The function *is* directly in an unnamed namespace, so both readings
agree; this cell isolates "does outer named nesting change anything".
**§2 clauses:** R1 ×2.
**Predicted (2):**
1. `?anchor@@YAHH@Z`
2. `?cand@?A0x#@N@@YAHH@Z` — the unreferenced `N::{anon}::cand(int)`

## a4_04_anon_ns_class_virtuals_object
**Structural point:** A4×A3 — an unnamed-namespace class with virtuals, with an
object constructed. Both R1-anon-ns (under A4-wide) and VT force the same set,
so a divergence here is not attributable to the reading question.
**§2 clauses:** R1 (`anchor`), P, VT, and (redundantly) R1 anon-ns.
**Predicted (6):**
1. `?anchor@@YAHH@Z`
2. `??0V@?A0x#@@QAA@XZ`
3. `?f@V@?A0x#@@UAAHH@Z`
4. `?g@V@?A0x#@@UAAHH@Z`
5. `??1V@?A0x#@@UAA@XZ`
6. `??_GV@?A0x#@@UAAPAXI@Z`

## a4_05_anon_ns_class_virtuals_no_object
**Structural point:** the discriminator for A4-wide vs A4-narrow — the same
unnamed-namespace polymorphic class with **no object constructed**, so VT is
idle and only the anon-ns root clause can emit anything.
**§2 clauses:** R1 (`anchor`), R1 anon-ns sub-clause under A4-wide.
**Predicted (4) — A4-wide, the literal reading:**
1. `?anchor@@YAHH@Z`
2. `?f@V@?A0x#@@UAAHH@Z`
3. `?g@V@?A0x#@@UAAHH@Z`
4. `??1V@?A0x#@@UAA@XZ`

No `??_G` (no constructor kept ⇒ VT idle) and no constructor.

**STRETCH — the designed one.** A4-narrow predicts **1** (`anchor` only). If
truth = 1, §2's anon-ns root clause is over-broad as written and the defect is
one of *statement*, which under guard 1 of the pre-registration is graded
AMBIGUOUS rather than VIOLATION unless the independent re-derivation agrees
with A4-wide. If truth is neither 1 nor 4 (e.g. 2), that is a violation of both
readings.

## a4_06_anon_ns_outofline_member_unref
**Structural point:** an unnamed-namespace class with one **out-of-line** member
and one in-class member, neither referenced. The out-of-line member is a root
under R1's "any out-of-line definition (member…)" clause *independently* of the
anon-ns clause; the in-class member is a root only under A4-wide. The cell
therefore separates the two R1 sub-clauses.
**§2 clauses:** R1 out-of-line-definition sub-clause; R1 anon-ns sub-clause.
**Predicted (3) — literal:**
1. `?anchor@@YAHH@Z`
2. `?m@S@?A0x#@@QAAHH@Z` — out-of-line, root by two independent sub-clauses
3. `?inl@S@?A0x#@@QAAHH@Z` — root only under A4-wide

**STRETCH:** A4-narrow predicts **2** (items 1–2). A result of 2 pins the defect
to the anon-ns clause specifically, since `m` is over-determined.

## a4_07_anon_ns_extern_c
**Structural point:** `extern "C"` *inside* an unnamed namespace — R1 names
`extern "C"` as strong linkage and separately names anon-ns-not-static, and the
two sub-clauses collide on one entity whose real C++ linkage is internal.
**§2 clauses:** R1 (`extern "C"` sub-clause), R1 (anon-ns sub-clause).
**Predicted (2):**
1. `?anchor@@YAHH@Z`
2. `cand` — undecorated (C language linkage), unreferenced but a root twice over

**STRETCH:** §2's `extern "C"` sub-clause sits under the heading "external
non-COMDAT linkage", which this entity arguably lacks. Both applicable
sub-clauses nevertheless make it a root, so the literal prediction is robust;
a truth of 1 would contradict both.

## a4_08_anon_ns_static_decl_nonstatic_def
**Structural point:** the crossing the axis is named for — the entity is
*declared* `static` at one declaration and defined without `static` at another.
§2's clause says "not declared `static`", which is a predicate on the entity,
not on a particular declaration.
**§2 clauses:** R1 (`anchor`); R1 anon-ns sub-clause **excluded** — a
declaration of this entity does say `static`.
**Predicted (1):**
1. `?anchor@@YAHH@Z`

**STRETCH — the designed one.** The definition-site reading ("the definition is
not declared `static`") makes `cand` a root and predicts **2**. §2's wording
does not say which declaration the test applies to; I take the entity-level
reading as literal because "declared static" is naturally a property of the
entity. Registered before compiling, so the grade cannot follow the truth.

## a4_09_anon_ns_dyninit_calls_anon_ns_static
**Structural point:** roots R4 and R5 operating *inside* an unnamed namespace,
with the dynamic initializer as the only path to a `static` helper that would
otherwise be dropped.
**§2 clauses:** R5 (`g_v` is non-const internal data ⇒ kept), R4 (its
dynamic-initializer thunk `??__E` is a root), P (the thunk's body calls
`helper` and `seed`), R1 anon-ns sub-clause (`seed`, independently).
**Predicted (4):**
1. `?anchor@@YAHH@Z`
2. `??__Eg_v@?A0x#@@YAXXZ` — the dynamic-initializer thunk (decoration advisory)
3. `?helper@?A0x#@@YAHH@Z` — reached only through the thunk
4. `?seed@?A0x#@@YAHH@Z` — reached through the thunk and a root anyway

**STRETCH:** §2 names `??__E` as a root but says nothing about the matching
atexit/teardown thunk `??__F`; predicted absent. If `??__Fg_v@…` appears, §2's
root list is incomplete rather than wrong.

---

# Axis A9 — vtable forced with NO kept constructor (9 cells)

This axis is the plan's own decision point **D6**. §2's VT clause is stated
*only* for constructors: "a kept **constructor** of C keeps C's vtable". Every
cell below arranges for a class's vtable to be needed (or plausibly needed)
while no constructor of that class is ODR-used or kept anywhere in the TU.
Under §2 as stated, VT never fires in cells a9_01–a9_08 and the virtuals must
not be emitted unless some other clause reaches them.

§2 also has **no clause for `dynamic_cast`, `typeid`, `delete`, vcall thunks,
or RTTI descriptors**. Under P, a `dynamic_cast` or `typeid` is none of {a call
to a definition in this TU, an address-take of one, a data initializer}, so it
contributes nothing. That is the literal reading applied below.

## a9_01_dynamic_cast_to_derived_no_ctor
**Structural point:** `dynamic_cast<D*>` to a polymorphic derived class with no
constructor of `B` or `D` kept. `B::f`/`B::~B` are declared-only, so the only
candidate bodies in the TU are `D::f` and `D::g`.
**§2 clauses:** R1 (`anchor`) only.
**Predicted (1):**
1. `?anchor@@YAHPAUB@@H@Z`

**STRETCH:** MSVC lowers `dynamic_cast` to a call to `__RTDynamicCast`, which
has no definition here; the RTTI descriptors live in data sections and so
cannot appear in the `.text` COMDAT leader set regardless.

## a9_02_dynamic_cast_virtual_base_no_ctor
**Structural point:** the same across a **virtual** base, which needs strictly
more RTTI machinery.
**§2 clauses:** R1 (`anchor`).
**Predicted (1):**
1. `?anchor@@YAHPAUA@@H@Z`

## a9_03_dynamic_cast_to_void
**Structural point:** `dynamic_cast<void*>`, which needs the runtime vtable of
the *static* type only and no target-type descriptor.
**§2 clauses:** R1 (`anchor`).
**Predicted (1):**
1. `?anchor@@YAHPAUB@@H@Z`

## a9_04_dynamic_cast_reference_form
**Structural point:** the reference form, which throws `std::bad_cast` on
failure and therefore crosses `/EHsc`; it also *calls* `D::f`, so one virtual is
reached by P without any constructor.
**§2 clauses:** R1 (`anchor`), P (`anchor` calls `d.f(x)`).
**Predicted (2):**
1. `?anchor@@YAHAAUB@@H@Z`
2. `?f@D@@UAAHH@Z`

`D::g` is never ODR-used and no constructor is kept ⇒ VT idle ⇒ dropped. No
`??_G`.

**STRETCH:** the call `d.f(x)` is a *virtual* call through a `D&`, which MSVC
may devirtualize or may route through the vtable; §2's P counts it as an
ODR-use of `D::f` either way. If the vtable is emitted as a consequence, `D::g`
and `??_GD` would appear — unpredicted.

## a9_05_outofline_virtual_dtor_no_ctor
**Structural point:** **the sharp D6 cell.** An out-of-line *virtual
destructor* definition is the only root touching `D`; no constructor is kept.
§2's VT clause names constructors, and a destructor is not a constructor.
**§2 clauses:** R1 ("any out-of-line definition (… virtual)") for `D::~D`;
R1 for `anchor`. **VT does not fire.**
**Predicted (2):**
1. `?anchor@@YAHH@Z`
2. `??1D@@UAA@XZ`

**STRETCH — this is the registered break candidate.** A destructor of a
polymorphic class also establishes the vptr and so references the vtable; if
truth includes `?f@D@@UAAHH@Z`, `?g@D@@UAAHH@Z` and/or `??_GD@@UAAPAXI@Z`, then
VT is **refuted as stated** — "constructor" is too narrow and the correct
formulation is "a kept constructor *or destructor*". Registered before
compiling.

## a9_06_delete_through_pointer_no_ctor
**Structural point:** the second sharp D6 cell — `delete p` on a `D*` whose
class has a virtual (in-class) destructor, with no constructor kept. Under P,
`delete p` is a call to `D::~D`, and nothing else.
**§2 clauses:** R1 (`anchor`), P (`delete p` ODR-uses `D::~D`).
**Predicted (2):**
1. `?anchor@@YAHPAUD@@H@Z`
2. `??1D@@UAA@XZ`

**STRETCH — registered break candidate.** MSVC implements `delete p` on a
virtual destructor as a *virtual call to the scalar deleting destructor*
`??_GD@@UAAPAXI@Z`, which is a vtable slot, which requires the vtable. If truth
includes `??_GD`, `?f@D@@` or `?g@D@@`, §2 is refuted twice over: VT fires with
no constructor, and the synthesized `??_G` is forced by something §2's
propagation rule does not name.

## a9_07_address_of_virtual_no_ctor
**Structural point:** an address-take of a **virtual** member function — §2's P
explicitly lists "an address-take" as an ODR-use, with no virtual/non-virtual
distinction.
**§2 clauses:** R1 (`anchor`), P (address-take of `D::f`).
**Predicted (2):**
1. `?anchor@@YAP8D@@AAHH@ZXZ` — the free function `anchor()` returning a
   pointer-to-member-function (decoration advisory, entity authoritative)
2. `?f@D@@UAAHH@Z`

**STRETCH — registered break candidate, in both directions.** MSVC materialises
`&D::f` for a *virtual* member as a **vcall thunk** (`??_9D@@$B…`) rather than
the function's address, so truth may (a) omit `?f@D@@UAAHH@Z` — falsifying P's
"an address-take" for virtuals — and/or (b) contain an unpredicted `??_9…`
thunk symbol, which §2 has no clause for.

## a9_08_typeid_no_ctor
**Structural point:** `typeid` on a polymorphic type with no constructor kept.
`type_info` is declared by hand because this toolchain is invoked with no
INCLUDE path; if that declaration is rejected the cell is reported as a
cell-level INSTRUMENT note, not a violation.
**§2 clauses:** R1 (`anchor`).
**Predicted (1):**
1. `?anchor@@YAHH@Z`

`type_info::name()` is declared-only, so it can never be emitted.

## a9_09_ctor_control
**Structural point:** the positive control for the whole axis — identical
polymorphic class, one constructed object, so VT *must* fire. If this cell does
not produce the full vtable closure, no negative result elsewhere on A9 is
interpretable.
**§2 clauses:** R1, P, VT.
**Predicted (6):**
1. `?anchor@@YAHH@Z`
2. `??0D@@QAA@XZ`
3. `?f@D@@UAAHH@Z`
4. `?g@D@@UAAHH@Z`
5. `??1D@@UAA@XZ`
6. `??_GD@@UAAPAXI@Z`

---

# Summary of predicted cardinalities

| axis | cells | predicted |
|---|---:|---|
| A2 | 9 | 2, 6, 2, 3, 6, 3, 5, 8, 3 |
| A3 | 8 | 10, 10, 13, 9, 9, 16, 3, 8 |
| A4 | 9 | 2, 2, 2, 6, 4, 3, 2, 1, 4 |
| A9 | 9 | 1, 1, 1, 2, 2, 2, 2, 1, 6 |

**Registered break candidates, named before compiling:** a9_05, a9_06, a9_07
(VT stated for constructors only; vcall thunks), a3_01/a3_03/a3_05 (adjustor
and vtordisp thunks unmodelled), a2_01/a2_02/a2_09 (`extern template`
unmodelled), a2_03 (explicit specialization vs R1's "any out-of-line
definition"), a4_05/a4_08 (statement-looseness in the anon-ns root clause).
