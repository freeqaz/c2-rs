# axes2 — RESULTS for A2 / A3 / A4 / A9

Agent `axes2`, lane `w-emitpred`, worktree `wt-w-emitpred`.

**Predictions frozen at `bac2372`** — that commit contains `PREDICTIONS.md`, all
35 `cell.cpp`, and `run_cells.py`, and was made **before any axis cell was
compiled**. Every grade below is against that pinned text. Nothing in
`PREDICTIONS.md` was edited after the first compile.

**These 35 cells are out-of-sample by construction.** While designing them I
read the *sources* of the 166 lane-A probe cells (to guarantee I was not
re-running structures the predicate was fitted on) but **never their compiled
truth**. In particular the wide-vs-narrow reading of §2's anonymous-namespace
root clause was left open in A4 rather than resolved from the in-sample cells
`ns02`/`ns05`, which would have leaked the fitters' intended reading. A reader
who does not know this would wrongly assume the grid is in-sample.

## Method

* Flags: `/O1 /Oi /EHsc /GS- /c`, X360 `16.00.11886.00` `cl.exe` under
  `wibo` — workload flags exactly, nothing added.
* **Ground truth = the obj's COMDAT leader symbol set over sections selected by
  `IMAGE_SCN_CNT_CODE` (0x20) in the section characteristics**, not by a
  `.text` name prefix. Both readings were computed and recorded separately.
* `/FAsc` `.cod` PROC names: cross-check only, never the judge.
* c1xx-side channel for the detectability work: the `/Bd /d2nop` capture that
  makes c2 abort with C1007 **before** deleting the temp `_CL_*` quintet, so the
  front end's `.gl`/`.ex` survive while the back end never decides anything.
  `.gl` names read with the separator-aware splitter (split on `0x00|0x26`),
  never raw `strings`.

Artifacts: `run_cells.py` → `observed.json`; `capture_il.py` → `il_names.json`;
`detectors.py` (detectability demonstration). Objs/listings/IL under `out/` and
`il/` — build products, not to be committed.

### Instrument controls, all clean

| control | result |
|---|---|
| cells compiled | **35 / 35** — no cell failed to build, including the hand-declared `type_info` cell (a9_08) |
| `CNT_CODE` vs `.text`-name section selection | **agree on 35/35** — no cell where the name prefix would have misled |
| non-COMDAT code sections (a place emissions could hide) | **0 cells** |
| obj code-leader set vs `.cod` PROC set (anon-ns hash normalized) | **agree on 35/35** |
| every emitted name present in the c1xx `.gl` table | **35/35, 0 exceptions** |

The last row independently reproduces the plan's lane-A/B corroboration
("the `.gl` name table is a necessary condition") on 35 structurally new cells.

## Verdict counts — five-way, reported separately

| axis | graded | MATCH | VIOLATION | AMBIGUOUS | INSTRUMENT-FAIL |
|---|---:|---:|---:|---:|---:|
| A2 templates | 9 | 7 | 1 | 1 | 0 |
| A3 virtual & multiple inheritance | 8 | 1 | 4 | 3 | 0 |
| A4 anonymous namespaces | 9 | 6 | 1 | 2 | 0 |
| A9 vtable with no kept constructor | 9 | 5 | 4 | 0 | 0 |
| **total** | **35** | **19** | **10** | **6** | **0** |

These are counts, not a pass rate. **All four axes carry ≥ 1 candidate
violation**, so on my axes alone V6 (registered point 1, interval [0, 3]) reads
**4**. Per the pre-registration's guard 1 these are *candidate* violations until
the independent re-derivation confirms them; AMBIGUOUS cells are already the
lesser defect class and are not counted as violations.

## Per-cell table

`pred` / `obs` are set cardinalities; the verdict is on set identity, not size.

### A2 — templates

| cell | pred | obs | verdict | note |
|---|---:|---:|---|---|
| a2_01 extern_template_member_called | 2 | 2 | MATCH | `extern template` did **not** suppress; §2's silence is harmless |
| a2_02 extern_template_virtual_object | 6 | 6 | MATCH | full vtable closure survives `extern template` |
| a2_03 explicit_fn_specialization_unref | 2 | 2 | MATCH | unreferenced explicit specialization **is** emitted — R1's wide "any out-of-line definition" reading confirmed |
| a2_04 explicit_member_specialization_unref | 3 | 2 | AMBIGUOUS | `?b@?$H@H@@QAAHH@Z` absent; the rival reading I registered predicts exactly 2 |
| a2_05 class_tmpl_virtuals_object | 6 | 6 | MATCH | control for a2_02 |
| a2_06 template_over_template | 3 | 3 | MATCH | two propagation steps through the nest |
| a2_07 explicit_inst_outer_forces_inner | 5 | 5 | MATCH | R2's "including never-referenced members" propagates transitively into the inner template |
| a2_08 explicit_inst_virtuals_no_object | 8 | 5 | **VIOLATION** | see below |
| a2_09 extern_template_plus_member_inst | 3 | 3 | MATCH | member-level explicit instantiation overrides the class-level `extern template` |

**a2_04 is AMBIGUOUS but the asymmetry is the finding.** a2_03 (a *namespace-scope*
explicit specialization) is a root; a2_04 (an explicit specialization of a
*member*) is not. R1's parenthetical explicitly enumerates "(member, static
member, virtual)", so under the clause's own wording a2_04 was the **stronger**
case of the two — and it is the one that failed. §2's R1 is not merely loose
here, it points the wrong way.

### A3 — virtual & multiple inheritance

| cell | pred | obs | verdict | note |
|---|---:|---:|---|---|
| a3_01 mi_override_second_base | 10 | 7 | AMBIGUOUS | the three `??_G` absent; matches the registered VT-dtor rival reading exactly. **No adjustor thunk** — that break candidate did not fire |
| a3_02 mi_no_override_second_base | 10 | 7 | AMBIGUOUS | same three `??_G`; identical cardinality to a3_01, so the override alone produces no extra symbol |
| a3_03 mi_virtual_dtors | 13 | 14 | **VIOLATION** | all 13 predicted present **plus** `??_ED@@W3AAPAXI@Z` |
| a3_04 virtual_base_simple | 9 | 10 | **VIOLATION** | all 9 present **plus** `??_DD@@QAAXXZ` |
| a3_05 vbase_virtual_overridden | 9 | 10 | **VIOLATION** | all 9 present **plus** `??_DD@@QAAXXZ`. **No vtordisp thunk** — that break candidate did not fire |
| a3_06 diamond | 16 | 19 | **VIOLATION** | all 16 present **plus** `??_DB`, `??_DC`, `??_DD` |
| a3_07 mi_outofline_virtuals_no_object | 3 | 3 | MATCH | R1 isolated from VT under MI; no thunk even with an override |
| a3_08 si_inherited_inline_virtual | 8 | 6 | AMBIGUOUS | the two `??_G` absent; VT-dtor rival reading |

**The VT-scope stretch resolved in favour of the wide reading.** `?f@A@@UAAHH@Z`
and `?only_a@A@@UAAHH@Z` — virtuals D inherits and never overrides or calls —
are emitted in every cell where D's constructor is kept. "Every virtual of C"
does mean *including inherited*.

**The VT-dtor stretch resolved against §2's literal wording.** `??_G` appears
only where the destructor is virtual (a3_03–a3_06 yes; a3_01/a3_02/a3_08 no).
§2 states it unconditionally. Graded AMBIGUOUS on three cells because I
registered the rival reading in advance, but the correction is unambiguous:
the clause needs "…plus the synthesized scalar-deleting destructor **when the
destructor is virtual**".

### A4 — anonymous namespaces

| cell | pred | obs | verdict | note |
|---|---:|---:|---|---|
| a4_01 nested_anon_ns | 2 | 2 | MATCH | `?cand@?A0x5873b9ed@1@YAHH@Z` — nested unnamed ns uses a back-reference |
| a4_02 named_ns_inside_anon_ns | 2 | 2 | MATCH | a **named** ns inside an unnamed one still yields a root |
| a4_03 anon_ns_inside_named_ns | 2 | 2 | MATCH | |
| a4_04 anon_ns_class_virtuals_object | 6 | 6 | MATCH | |
| a4_05 anon_ns_class_virtuals_no_object | 4 | 1 | AMBIGUOUS | observed = the registered A4-narrow reading exactly |
| a4_06 anon_ns_outofline_member_unref | 3 | 2 | AMBIGUOUS | observed = the registered A4-narrow reading exactly |
| a4_07 anon_ns_extern_c | 2 | 2 | MATCH | `extern "C"` in an unnamed ns, unreferenced, emitted undecorated as `cand` |
| a4_08 anon_ns_static_decl_nonstatic_def | 1 | 1 | MATCH | the **entity-level** reading of "not declared `static`" is the right one |
| a4_09 anon_ns_dyninit_calls_anon_ns_static | 4 | 3 | **VIOLATION** | see below |

**A4's reading question is now settled, and it settles cleanly.** a4_05 and
a4_06 together pin the anonymous-namespace root clause to **namespace-scope free
functions only**: in a4_06 the out-of-line member `?m@S@?A0x…@@QAAHH@Z` *is*
emitted (it is over-determined — a root by R1's "any out-of-line definition"
clause independently), while the in-class `inl` is not. So §2's clause should
read "anonymous-namespace **namespace-scope** functions not declared `static`".
Both cells are AMBIGUOUS rather than VIOLATION because I registered the narrow
reading in advance, but the repair is not in doubt.

### A9 — vtable forced with no kept constructor (plan D6)

| cell | pred | obs | verdict | note |
|---|---:|---:|---|---|
| a9_01 dynamic_cast_to_derived_no_ctor | 1 | 1 | MATCH | |
| a9_02 dynamic_cast_virtual_base_no_ctor | 1 | 1 | MATCH | |
| a9_03 dynamic_cast_to_void | 1 | 1 | MATCH | |
| a9_04 dynamic_cast_reference_form | 2 | 1 | **VIOLATION** | `?f@D@@UAAHH@Z` predicted, absent |
| a9_05 outofline_virtual_dtor_no_ctor | 2 | 5 | **VIOLATION** | `?f@D@@`, `?g@D@@`, `??_GD@@` all unpredicted-present |
| a9_06 delete_through_pointer_no_ctor | 2 | 1 | **VIOLATION** | `??1D@@UAA@XZ` predicted, absent |
| a9_07 address_of_virtual_no_ctor | 2 | 2 | **VIOLATION** | two independent directions, below |
| a9_08 typeid_no_ctor | 1 | 1 | MATCH | hand-declared `type_info` compiled cleanly — not an instrument failure |
| a9_09 ctor_control | 6 | 6 | MATCH | **the positive control holds**, so the negatives above are interpretable |

**D6 is resolved, and it splits.** The `dynamic_cast`/`typeid` half that the plan
worried about is **clean**: a9_01/02/03/08 emit exactly `anchor` and nothing
else — RTTI descriptors land in data COMDATs (`??_R0?AUB@@@8` etc.), never in
code, so they cannot perturb the emitted function set. The *destructor* half is
where the rule breaks.

---

# Missing categories — the category-shaped holes in §2

Four families. Each is stated in one sentence, followed by the cells that
establish it and the clause it contradicts.

## Family 1 — §2 has no clause for compiler-synthesized thunk and closure symbols

**One sentence:** c2 emits code COMDATs for symbol classes that exist nowhere in
the source and that §2's root list and propagation rule never mention —
virtual-base destructors (`??_D`), adjustor vector-deleting destructors under
multiple inheritance (`??_E…W3…`), and vcall thunks (`??_9`) — so §2 applied
literally under-predicts every TU that has a virtual base, MI with a virtual
destructor, or a pointer-to-virtual-member.

**Clause contradicted:** the root list and the propagation rule jointly — every
one of these symbols is emitted while being reachable from no clause. §2
accounts for exactly one synthesized name (`??_G`) and only as a rider on the
vtable rule.

**Cells (5):**
* `a3_03` — `??_ED@@W3AAPAXI@Z` (adjustor form of the vector deleting destructor
  for D in B's subobject vtable), on top of all 13 predicted names.
* `a3_04`, `a3_05` — `??_DD@@QAAXXZ`, the virtual-base destructor.
* `a3_06` — `??_DB@@QAAXXZ`, `??_DC@@QAAXXZ`, `??_DD@@QAAXXZ`.
* `a9_07` — `??_9D@@$BA@AA`, the vcall thunk.

**Not fired:** the plain MI adjustor thunk for an overridden *non-destructor*
virtual (a3_01, a3_02, a3_07) and the vtordisp thunk (a3_05) produced **no**
extra code COMDAT. The category is narrower than the prereg's shorthand
("`??_9` adjustor thunks") assumed.

## Family 2 — the vtable trigger is not "a kept constructor"

**One sentence:** §2 says "a kept **constructor** of C keeps C's vtable", but the
full vtable closure (every virtual plus `??_G`) fires with **no constructor
emitted at all** — an out-of-line virtual *destructor* definition does it, and
so does an explicit instantiation definition.

**Clause contradicted:** the vtable rule, verbatim.

**Cells (2):**
* `a9_05` — predicted `{?anchor@@YAHH@Z, ??1D@@UAA@XZ}`; observed
  `{??1D@@UAA@XZ, ??_GD@@UAAPAXI@Z, ?anchor@@YAHH@Z, ?f@D@@UAAHH@Z,
  ?g@D@@UAAHH@Z}`. No constructor of D exists anywhere in the TU.
* `a2_08` — predicted 8 names including three implicit special members; observed
  `{??1?$V@H@@UAA@XZ, ??_G?$V@H@@UAAPAXI@Z, ?anchor@@YAHH@Z, ?f@?$V@H@@UAAHH@Z,
  ?g@?$V@H@@UAAHH@Z}` — **no constructor, no copy constructor, no copy
  assignment**, yet `??_G` is present. Neither of the two alternative readings I
  registered predicts this set, which is why it grades VIOLATION rather than
  AMBIGUOUS: the registered 5-name alternative was
  `{anchor, f, g, ~V, and no ??_G}`, and the observed 5 is a *different* 5.

**Minimal repair:** "a kept constructor **or destructor** of C keeps C's vtable",
plus a rule that an explicit instantiation definition of a polymorphic class
template keeps the vtable without keeping the special members.

## Family 3 — a virtual call is not an ODR-use of the callee

> **[Lead note, added at recovery — evidential weight of this family.]** axes1
> discovered this mechanism independently and first, on `a6c5 tu2`, and pinned
> it with four separating probes (virtual dispatch vs non-virtual member call
> vs qualified call vs member-pointer take). axes2 ran concurrently and without
> sight of axes1's results, so `a9_04`/`a9_06` are a genuine **independent
> replication** on different structures — but they are **confirmatory, not
> discovery**, and must be weighted that way. A cell that re-finds a known
> mechanism is weaker evidence than the cell that found it. Concretely: the
> whole of axis **A9** is now the confirmatory follow-up to axes1's A6
> violation, not an independent probe of §2, and V6 must not double-count
> A6 and A9 as two independent refutations of the same clause — they are one
> mechanism found twice.

**One sentence:** §2's propagation counts "a call anywhere in the
pre-optimization body" with no virtual/non-virtual distinction, but a call that
dispatches through a vtable does **not** keep the callee's definition — the
vtable slot is the reference, and if the vtable is not itself kept, nothing is.

**Clause contradicted:** the propagation rule's "a call anywhere in the
pre-optimization body (including statically dead branches and `catch`
handlers)".

**Cells (2):**
* `a9_04` — `anchor` calls `d.f(x)` on a `D&`; `?f@D@@UAAHH@Z` predicted, and it
  is **absent**. Observed `{?anchor@@YAHAAUB@@H@Z}` only.
* `a9_06` — `anchor` does `delete p` on a `D*` whose destructor is virtual;
  `??1D@@UAA@XZ` predicted, and it is **absent**. Observed
  `{?anchor@@YAHPAUD@@H@Z}` only.

This is the mirror image of family 2 and the two must be repaired together: in
a9_05 the vtable is kept and therefore *everything* comes along; in a9_06 the
vtable is not kept and therefore *nothing* does — including the destructor that
was syntactically called.

## Family 4 — root 4 (`??__E`) fires without emission and defines no trigger

**One sentence:** §2 lists dynamic-initializer thunks as roots but gives no rule
for which initializers produce one, and on `a4_09` the front end *does* name a
`??__E` thunk that c2 then does **not** emit.

**Clause contradicted:** root 4.

**Cell (1):** `a4_09` — predicted `{anchor, ??__Eg_v@…, helper, seed}`; observed
`{?anchor@@YAHH@Z, ?helper@?A0x…@@YAHH@Z, ?seed@?A0x…@@YAHH@Z}`. The thunk name
`??__Eg_v@?A0xd1100455@@YAXXZ` **is present in the `.gl` table**, so the
front-end entity exists; c2 folded the initializer and dropped the thunk while
the datum survives in `.data`.

**A corroboration worth recording from the same cell:** `?helper@…` is a
`static` reachable *only* through that thunk, and it is emitted anyway. So the
reference from a definition that was ultimately **not emitted** still counted.
That is §2's "pre-optimization" clause working exactly as written, and it also
sharpens the "references from removed definitions never count" clause: "removed"
must mean *never kept by the fixpoint*, not *absent from the output*.

---

# Detectability — can a fail-closed R3 refuse instead of emitting a wrong obj?

Demonstrated, not asserted: `detectors.py`, run on all 35 cells.

| detector | channel | family | target | flagged | false neg | over-flags |
|---|---|---|---:|---:|---:|---:|
| D1a `.gl` names a `??_D` / `??_9` / adjustor-`W` symbol | c1xx `.gl` | 1 | 5 | 5 | **0** | **0** |
| D1b `.gl` names a `??__E` thunk | c1xx `.gl` | 4 | 1 | 1 | **0** | **0** |
| D2a out-of-line virtual dtor, or explicit inst. of a polymorphic class template | source | 2 | 2 | 2 | **0** | **0** |
| D2b `virtual` present ∧ (`delete` ∨ a member call) | source | 3 | 2 | 2 | **0** | **0** |
| **union refusal rule** | — | all | **10** | **10** | **0** | **0** |

The union rule refuses **10 of 35** cells and every one of them is a violating
cell: zero non-violating cells are refused on this population. For a fail-closed
consumer that is the good direction on both counts.

**The `.gl` channel is sufficient for families 1 and 4 and provably insufficient
for families 2 and 3.** The proof is a minimal pair inside my own grid:

```
a9_05 .gl (minus anchor) = a9_06 .gl (minus anchor)
  = {??1D@@UAA@XZ, ??3@YAXPAX@Z, ??_7D@@6B@,
     ??_ED@@UAAPAXI@Z, ??_GD@@UAAPAXI@Z, ?f@D@@UAAHH@Z, ?g@D@@UAAHH@Z, ...}
a9_05 emits 5 functions.   a9_06 emits 1.
```

Identical front-end name tables, emitted sets differing by four functions. So
for the destructor/virtual-call families the discriminator is **not** in the
`.gl` names and R3 must get it from the IL body structure (a vtable-dispatched
call is a different construct from a direct call) or from source. My D2a/D2b
demonstrations are **source-side** greps: they are an existence proof that the
breaking condition is visible at all, **not** a production detector, and I did
not build the IL-side version. That gap should be stated plainly wherever this
feeds R3.

This also re-confirms, on new ground, that `.gl` presence is a *necessary*
condition and never a predicate: `?f@D@@UAAHH@Z` sits in a9_06's `.gl` and is
not emitted.

---

# What did NOT break, and it matters

Registered break candidates that **failed to fire** — these are the cells where
§2 survived an attack designed to kill it:

* **`extern template` is a no-op for emission** (a2_01, a2_02, a2_09, all MATCH).
  §2's total silence about explicit instantiation *declarations* turns out to be
  harmless: the member is still emitted, the vtable closure still fires, and a
  member-level explicit instantiation definition still works alongside it.
* **Plain MI adjustor thunks and vtordisp thunks produce no code COMDAT**
  (a3_01, a3_02, a3_05, a3_07). Only the *destructor* adjustor does (a3_03).
* **`dynamic_cast` and `typeid` force nothing** (a9_01–a9_03, a9_08). The half of
  D6 the plan named explicitly is clean; the half it did not name is what broke.
* **R2 propagates transitively across template boundaries** (a2_07, MATCH, 5/5).
* **VT-scope is the wide reading** — inherited, never-overridden, never-called
  virtuals are emitted (a3_08, a3_01).
* **The positive control holds** (a9_09, 6/6), so A9's four negatives are not an
  artifact of a dead cell design.

# Bearing on the lane's verdict

Part 2 scoring says any confirmed violation makes an axis **BROKEN**, and a
broken axis blocks SHIP-CANDIDATE unless the breaking condition is demonstrated
detectable. On my four axes:

* All four axes carry candidate violations ⇒ **all four BROKEN**, pending guard
  1's independent re-derivation.
* All 10 violations are covered by a demonstrated detector with 0 false
  negatives and 0 over-flags ⇒ all four axes are **guardable** in the
  pre-registration's sense, with the honest caveat that families 2 and 3 are
  guarded from *source*, not from the IL that R3 will actually consume, and the
  `.gl` channel is proven unable to do it.

The six AMBIGUOUS cells are the lesser defect class and they cluster: three are
the same VT-dtor over-statement, two are the same anonymous-namespace
over-statement, one is the member-specialization asymmetry. All three are
one-line repairs to §2's *wording* whose direction is unambiguous from the data,
which is a much better position than a genuine modelling gap.
