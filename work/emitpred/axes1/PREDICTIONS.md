# axes1 — PREDICTIONS (Phase 1, frozen before any axis cell is compiled)

Agent `axes1`, lane `w-emitpred`, worktree `wt-w-emitpred`. Axes **A1, A5, A6,
A7, A8** of the prereg's Part 2. 43 designed cells, 60 graded objs.

Every predicted set below is derived **only** from `PHASE7_PLAN.md` §2's text.
Nothing here was informed by compiling any axis cell — at the time of writing
the only compile performed by this agent is one throwaway smoke cell
(`smoke/smoke.cpp`, `extern int sink(int); int smokefn(int x){return sink(x)+1;}`)
run to verify the toolchain invocation, whose only use is that
`cl.exe` under wibo exits 0 and the reader finds `?smokefn@@YAHH@Z`.

## Clause labels used in the derivations

From §2 verbatim:

| tag | §2 clause |
|---|---|
| **R1** | root: *every definition with external non-COMDAT linkage* — plain extern, `extern "C"`, **any out-of-line definition** (member, static member, virtual), anon-ns functions not declared `static` |
| **R2** | root: explicit instantiation definitions, incl. never-referenced members |
| **R3** | root: `__declspec(dllexport)` closure incl. implicit special members |
| **R4** | root: dynamic-initializer thunks (`??__E`) |
| **R5** | root: kept data definitions — external-linkage data, and non-const internal data (internal *const* data is dropped when unreferenced and its references then do not count) |
| **P1** | propagation: F is added if an already-kept definition ODR-uses it — a call anywhere in the **pre-optimization** body (incl. statically dead branches and `catch` handlers), an address-take, or a data initializer |
| **P2** | `sizeof` does not count |
| **P3** | references from removed (never-kept) definitions never count; the fixpoint is over kept code only; cycles do not sustain themselves |
| **V** | vtable rule: a kept constructor of C keeps C's vtable, whose slots force **every** virtual of C plus the synthesized scalar-deleting destructor |
| **S** | scope: *least-fixpoint reachability from roots, computed over kept definitions only, at ODR-use granularity, pre-optimization* — and the predicate is stated **per translation unit** (§2's grading unit is one TU's obj; R3 of the route applies it per `.ex` segment of **one** TU's IL) |

### Two derivation conventions I am fixing now, in advance

1. **`inline` ⇒ COMDAT linkage.** R1's head says *external **non-COMDAT**
   linkage*; the parenthetical ("plain extern, `extern "C"`, any out-of-line
   definition") enumerates instances of that head, it does not override it. So
   an `inline` definition — however it is spelled, including `extern "C"
   inline` and `extern inline` — is **not** an R1 root. This is the reading I
   predict by. Cells **a5c1 / a5c4** are designed so that the alternative
   reading (parenthetical taken as an independent enumeration) gives a
   *different* set; if truth matches the alternative, that is reported as a
   defect in §2's *statement* (AMBIGUOUS class of the prereg's guard 1), not
   silently absorbed.
2. **`static` ⇒ internal linkage, therefore never an R1 root**, whatever the
   linkage specification. `extern "C" static` and `extern "C" static inline`
   are internal. (The 172 cells already contain `e02_externC_static_unref`,
   so this reading is not novel; the *inline* crossings and the
   *header-defined* crossings are what is new here.)

### Graded quantity

Ground truth per cell-obj = the obj's **`.text` COMDAT leader symbol set**
(standing rule). Because axis **A7** deliberately renames the code section with
`#pragma code_seg`, my reader (`leaders.py`) computes the leader set over
sections carrying `IMAGE_SCN_CNT_CODE` and **also** over `.text`-prefixed
sections, and records both. For all 42 non-`code_seg` cells the two are
identical by construction; for **a7c8** the code-characteristic reading is the
one I predict against, and any divergence is reported as an instrument note,
never as a violation.

`??_7C@@6B@` (the vtable) lives in `.rdata`, not a code section, so it never
appears in a predicted set below even where **V** fires; `.rdata`/`.data`
leaders are recorded separately as context.

### Decoration convention (instrument, not predicate)

Decorated names below follow the MSVC PPC convention already visible in this
project's fitted probe objs (`?f@@YAHH@Z` free function; `??0G@@QAA@XZ` ctor;
`??1G@@UAA@XZ` virtual dtor; `??_GG@@UAAPAXI@Z` scalar-deleting dtor;
`?v@G@@UAAHH@Z` virtual member; `??__Eg@@YAXXZ` dynamic-init thunk;
`extern "C"` ⇒ undecorated, no leading underscore). Reading the *spelling* of a
name off existing fitted cells is instrument calibration and is disclosed here;
it decides no predicate question. Where a decoration is genuinely uncertain
(only `extern "C" static`, cell a5c9) I give the unambiguous description plus
the count, and decoration-only variance does not change a verdict.

`?anchor@@YAHH@Z` (and `?anchor1/2/3@@YAHH@Z`, `?anchorg@@YAHH@Z`,
`?anchoru@@YAHH@Z`, `?anchoru2@@YAHH@Z`) is an **R1** root in every cell and is
therefore in every predicted set. `sink`/`seed` are declared-not-defined and
are never emitted anywhere.

---

# A1 — header inclusion depth (8 cells)

**Axis point:** §2 contains no term for *where* a definition was textually
reached from. If the same definition behaves differently at include depth 1
and 5, or through a diamond, or when the definition arrives textually after
the use, §2's fixpoint is under-specified.

### a1c1 `a1c1_depth1_inline_ref` — baseline, inline at depth 1, referenced
Clauses: R1 (anchor) → P1 (anchor calls `cand`) → `cand` kept.
**Predicted: `{?cand@@YAHH@Z, ?anchor@@YAHH@Z}` — 2.**

### a1c2 `a1c2_depth5_inline_ref` — the identical definition at depth 5
Clauses: R1, P1. §2 has no depth term ⇒ set must equal a1c1's.
**Predicted: `{?cand@@YAHH@Z, ?anchor@@YAHH@Z}` — 2.**

### a1c3 `a1c3_depth5_inline_unref` — depth 5, not referenced
Clauses: `inline` ⇒ COMDAT ⇒ not R1; no kept definition ODR-uses it (P3).
**Predicted: `{?anchor@@YAHH@Z}` — 1.**

### a1c4 `a1c4_depth3_static_one_ref` — two header statics, one referenced
Clauses: `static` ⇒ internal ⇒ not R1. `cand` kept by P1 from anchor; `dead`
has no kept referrer.
**Predicted: `{?cand@@YAHH@Z, ?anchor@@YAHH@Z}` — 2** (`?dead@@YAHH@Z` absent).

### a1c5 `a1c5_depth4_extern_def_unref` — external out-of-line definition living at depth 4, never referenced
Clauses: R1 ("any out-of-line definition", external, non-inline) ⇒ root
regardless of reference.
**Predicted: `{?cand@@YAHH@Z, ?anchor@@YAHH@Z}` — 2.**

### a1c6 `a1c6_diamond_two_depths` — the same guarded leaf reached at depth 3 and depth 2
Clauses: R1, P1. One definition, one inclusion (guard), no depth term in §2.
**Predicted: `{?cand@@YAHH@Z, ?anchor@@YAHH@Z}` — 2.**

### a1c7 `a1c7_use_before_def_depth5` — the ODR-use precedes the definition, which arrives at depth 5
Clauses: S ("reachability … over definitions", no textual-order term), R1, P1.
**Predicted: `{?cand@@YAHH@Z, ?anchor@@YAHH@Z}` — 2.**
*Breaking condition targeted:* a front end that decides emission at the point
of use would leave `cand` out.

### a1c8 `a1c8_chain_across_depths` — transitive inline chain spanning depths 5→3→1
Clauses: R1 (anchor) → P1 `topcand` (d5) → P1 `midcand` (d3) → P1 `leafcand` (d1).
**Predicted: `{?leafcand@@YAHH@Z, ?midcand@@YAHH@Z, ?topcand@@YAHH@Z, ?anchor@@YAHH@Z}` — 4.**

---

# A5 — `static` / `inline` / `extern "C"` / `static inline` crossings, incl. header-defined (9 cells)

**Axis point:** the 172 fitted cells contain **no header at all** (verified:
`ls *.h` in `work/probes/` is empty) and no `inline`-crossed-with-`extern "C"`
or `static` spelling. R1's parenthetical names `extern "C"` explicitly; the
crossings test whether the *head* (external **non-COMDAT**) or the
*parenthetical* governs.

### a5c1 `a5c1_externC_inline_unref` — `extern "C" inline`, unreferenced
Clauses: R1 head — `inline` ⇒ COMDAT ⇒ **not** a root; no kept referrer (P3).
**Predicted: `{?anchor@@YAHH@Z}` — 1.**
*Registered alternative reading* (parenthetical as an independent enumeration):
`{cand, ?anchor@@YAHH@Z}` — 2. If truth is the alternative, verdict is
AMBIGUOUS-on-§2's-statement, and the fix is one word in §2 ("`extern "C"`
**non-inline**").

### a5c2 `a5c2_externC_inline_ref` — same, referenced
Clauses: R1 (anchor), P1.
**Predicted: `{cand, ?anchor@@YAHH@Z}` — 2.**

### a5c3 `a5c3_static_inline_ref_and_unref` — `static inline` ×2, one referenced
Clauses: internal ⇒ not R1; P1 keeps `candR`; `candU` has no kept referrer.
**Predicted: `{?candR@@YAHH@Z, ?anchor@@YAHH@Z}` — 2.**

### a5c4 `a5c4_extern_then_inline_unref` — `extern` declaration then `inline` definition; and `extern inline`
Clauses: both definitions are `inline` ⇒ COMDAT ⇒ not R1; neither referenced.
**Predicted: `{?anchor@@YAHH@Z}` — 1** (`?cand@@YAHH@Z`, `?cand2@@YAHH@Z` absent).
*Breaking condition targeted:* an `extern`-spelling-driven root test would emit
one or both.

### a5c5 `a5c5_header_static_one_ref` — two statics **defined in a header**, one referenced
Clauses: internal ⇒ not R1; P1 keeps `hcandR`.
**Predicted: `{?hcandR@@YAHH@Z, ?anchor@@YAHH@Z}` — 2.**

### a5c6 `a5c6_header_linkage_matrix` — six header-defined linkage classes, three referenced
`hiR`/`hiU` `inline`; `hsiR`/`hsiU` `static inline`; `hciR`/`hciU`
`extern "C" inline`. anchor calls `hiR`, `hsiR`, `hciR`.
Clauses: none of the six is an R1 root (all `inline` ⇒ COMDAT, and the two
`static inline` are additionally internal); the three referenced are kept by P1.
**Predicted: `{?hiR@@YAHH@Z, ?hsiR@@YAHH@Z, hciR, ?anchor@@YAHH@Z}` — 4**
(`?hiU@@YAHH@Z`, `?hsiU@@YAHH@Z`, `hciU` absent). Set comparison; order not graded.

### a5c7 `a5c7_header_externC_def_unref` — `extern "C"` **non-inline** definition in a header, unreferenced
Clauses: R1 head + parenthetical agree (external, non-COMDAT, out-of-line) ⇒ root.
**Predicted: `{hc, ?anchor@@YAHH@Z}` — 2.**

### a5c8 `a5c8_header_static_inline_addr_in_data` — header `static inline` reached only through a kept data initializer
`int (*g_p)(int) = &hsi;` at namespace scope.
Clauses: R5 (external-linkage data ⇒ kept) → P1 (address-take in a data
initializer) ⇒ `hsi` kept. `g_p` is data, not a code leader.
**Predicted code set: `{?hsi@@YAHH@Z, ?anchor@@YAHH@Z}` — 2.**
(Context, not graded: `g_p` expected as a `.data` symbol.)

### a5c9 `a5c9_externC_static_inline` — `extern "C" static inline` ×2, one referenced
Clauses: `static` ⇒ internal ⇒ not R1 (convention 2); P1 keeps `candR`.
**Predicted: 2 names — the anchor `?anchor@@YAHH@Z`, plus the referenced
function `candR` and *not* the unreferenced `candU`.** Decoration of `candR`
under `extern "C" static` is the one genuinely uncertain spelling in this file;
primary guess `candR`, acceptable variant `?candR@@YAHH@Z`. Decoration-only
variance does not change the verdict.

---

# A6 — multiple TUs sharing one header, references differing per TU (8 cells, 18 objs)

**Axis point:** §2's fixpoint is stated per TU (**S**). Nothing in §2 says what
happens when one `cl.exe` process compiles several TUs that share a header —
i.e. when the front end has already parsed and kept a definition for TU1 and
then starts TU2. Cells a6c2/a6c3/a6c4/a6c5/a6c6/a6c7/a6c8 all put both/all TUs
in **one invocation** precisely so that cross-TU leakage, if it exists, has an
opportunity to appear; a6c1 is the separate-invocation control on identical
sources.

**Any name emitted into a TU's obj that this TU's own fixpoint does not reach
is a VIOLATION of S.**

### a6c1 `a6c1_shared_inline_separate_invocations` — control, two invocations
`shared.h`: `inline ca, cb, cc`. tu1 calls `ca`; tu2 calls `cb`.
**Predicted `tu1.obj`: `{?ca@@YAHH@Z, ?anchor1@@YAHH@Z}` — 2.**
**Predicted `tu2.obj`: `{?cb@@YAHH@Z, ?anchor2@@YAHH@Z}` — 2.**
`?cc@@YAHH@Z` in neither.

### a6c2 `a6c2_shared_inline_one_invocation` — byte-identical sources, ONE invocation
**Predicted: identical to a6c1, obj for obj.** `tu1.obj` 2, `tu2.obj` 2, no `cc`.

### a6c3 `a6c3_shared_inline_one_invocation_reversed` — one invocation, command-line order `tu2.cpp tu1.cpp`
**Predicted: identical to a6c1/a6c2, obj for obj** (§2 has no order term).

### a6c4 `a6c4_shared_static_one_tu_refs` — shared header statics; only tu1 references
`shared.h`: `static sa, sb`. tu1 calls `sa`; tu2 calls neither. One invocation.
Clauses: internal ⇒ not R1; per-TU P1.
**Predicted `tu1.obj`: `{?sa@@YAHH@Z, ?anchor1@@YAHH@Z}` — 2.**
**Predicted `tu2.obj`: `{?anchor2@@YAHH@Z}` — 1.**
*Breaking condition targeted:* `sa` appearing in `tu2.obj`.

### a6c5 `a6c5_shared_vtable_one_tu_constructs` — the vtable rule across two TUs
`shared.h`: `struct C { int f; C():f(1){} virtual ~C(){} virtual int v(int); virtual int w(int); };`
(all in-class). tu1: `C c; return c.v(x)+sink(x);`. tu2: `extern C* pc;
return pc->v(x)+sink(x);` — a virtual call with **no** construction.
One invocation.

*tu1 derivation:* R1 anchor1 → P1 ODR-uses `C::C()` ⇒ ctor kept → **V**: a kept
ctor keeps C's vtable, whose slots force every virtual of C (`v`, `w`) plus the
synthesized scalar-deleting destructor (`??_G`); `??_G` kept ⇒ P1 ⇒ `??1C`
kept (which the local object's destruction ODR-uses directly as well).
**Predicted `tu1.obj`: `{??0C@@QAA@XZ, ??1C@@UAA@XZ, ??_GC@@UAAPAXI@Z,
?v@C@@UAAHH@Z, ?w@C@@UAAHH@Z, ?anchor1@@YAHH@Z}` — 6.**
(`??_7C@@6B@` expected in `.rdata`, not graded.)

*tu2 derivation:* **no** constructor of C is kept in tu2 ⇒ **V does not fire**
⇒ no vtable ⇒ `w` and `??_G` are not forced. anchor2's pre-optimization body
contains a call to `C::v` ⇒ P1 ⇒ `v` kept.
**Predicted `tu2.obj`: `{?v@C@@UAAHH@Z, ?anchor2@@YAHH@Z}` — 2.**
*Breaking conditions targeted, two:* (i) `w`/`??_G`/`??1C`/`??0C` appearing in
`tu2.obj` ⇒ either V leaked across TUs or a virtual **call** alone forces the
vtable — either way §2-as-stated is wrong; (ii) `v` missing from `tu2.obj` ⇒
P1's "a call anywhere in the body" does not cover virtual dispatch.

### a6c6 `a6c6_shared_extern_def_neither_refs` — header carries an external non-COMDAT definition; neither TU references it
One invocation. Clauses: R1 fires **independently in each TU** (S).
**Predicted `tu1.obj`: `{?hc@@YAHH@Z, ?anchor1@@YAHH@Z}` — 2.**
**Predicted `tu2.obj`: `{?hc@@YAHH@Z, ?anchor2@@YAHH@Z}` — 2.**
*Breaking condition targeted:* the root firing in only the first TU compiled
(a compiler-process-level rather than TU-level root set).

### a6c7 `a6c7_three_tus_middle_refs` — three TUs in one invocation, only the middle references
**Predicted `tu1.obj`: `{?anchor1@@YAHH@Z}` — 1.**
**Predicted `tu2.obj`: `{?cb@@YAHH@Z, ?anchor2@@YAHH@Z}` — 2.**
**Predicted `tu3.obj`: `{?anchor3@@YAHH@Z}` — 1.**
*Breaking condition targeted:* leakage forwards (into tu3) and/or backwards
(into tu1) from tu2's keep.

### a6c8 `a6c8_shared_dyninit_per_tu` — a header-defined internal dynamic-init datum, independently rooted in each TU
`shared.h`: `extern int seed(); inline int mk(int x){return x*3+1;}
static int g_v = mk(seed());`. Both TUs include it and read `g_v`.
Clauses: R5 (`g_v` internal, **non-const** ⇒ kept) ⇒ a dynamic initializer
exists ⇒ R4 root `??__Eg_v@@YAXXZ` ⇒ P1 (the thunk's body calls `mk`) ⇒ `mk`
kept. `seed` is declared-not-defined. Independently in each TU (S).
**Predicted `tu1.obj`: `{?mk@@YAHH@Z, ??__Eg_v@@YAXXZ, ?anchor1@@YAHH@Z}` — 3.**
**Predicted `tu2.obj`: `{?mk@@YAHH@Z, ??__Eg_v@@YAXXZ, ?anchor2@@YAHH@Z}` — 3.**
The initializer is deliberately non-foldable (`mk(seed())`, `seed` external) so
that the existence of the dynamic initializer is not itself in question.

---

# A7 — pragma-created roots (10 cells)

**Axis point:** §2's root list (R1–R5) contains **no pragma clause**. If any
pragma manufactures a root, §2 is incomplete as stated. The 172 fitted cells
contain no `#pragma` of any kind (verified by inspection of `work/probes/*.cpp`).

### a7c1 `a7c1_linker_include_static` — `#pragma comment(linker, "/include:?cand@@YAHH@Z")` naming an unreferenced `static`
Clauses: R1–R5 list no pragma root; a `#pragma comment` emits a `.drectve`
string, which is neither a definition, nor data with an initializer, nor an
ODR-use inside any kept body (P1). `cand` is internal and unreferenced.
**Predicted: `{?anchor@@YAHH@Z}` — 1.**
*If `?cand@@YAHH@Z` is emitted ⇒ **VIOLATION** of §2's root enumeration.*

### a7c2 `a7c2_linker_include_inline` — same, naming an unreferenced `inline`
**Predicted: `{?anchor@@YAHH@Z}` — 1.** Same violation condition.

### a7c3 `a7c3_comment_lib_inert` — `#pragma comment(lib,…)` + `#pragma comment(exestr,…)` beside an unreferenced `static`
Control: pragmas that could not plausibly create a root.
**Predicted: `{?anchor@@YAHH@Z}` — 1** (only `.drectve` content differs).

### a7c4 `a7c4_initseg_compiler` — `#pragma init_seg(compiler)` over an external dynamic-init datum
Body (shared by a7c4–a7c7): `extern int seed(); static int mk(int x){…}
int g_v = mk(seed()); … int anchor(int x){return sink(x)+3;}`.
Clauses: R5 (`g_v` external-linkage data ⇒ kept) ⇒ R4 root `??__Eg_v@@YAXXZ`
⇒ P1 ⇒ `mk` kept; R1 anchor. §2 has **no section term**, so `init_seg` — which
only chooses which `.CRT$XC?` section holds the initializer pointer — must
leave the name set unchanged.
**Predicted: `{?mk@@YAHH@Z, ??__Eg_v@@YAXXZ, ?anchor@@YAHH@Z}` — 3.**

### a7c5 `a7c5_initseg_baseline_nopragma` — the paired control, no pragma
**Predicted: `{?mk@@YAHH@Z, ??__Eg_v@@YAXXZ, ?anchor@@YAHH@Z}` — 3.**
a7c4, a7c6, a7c7 must each equal this set exactly.

### a7c6 `a7c6_initseg_lib` — `#pragma init_seg(lib)`
**Predicted: `{?mk@@YAHH@Z, ??__Eg_v@@YAXXZ, ?anchor@@YAHH@Z}` — 3.**

### a7c7 `a7c7_initseg_named_section` — `#pragma init_seg(".mycrt$a")`
**Predicted: `{?mk@@YAHH@Z, ??__Eg_v@@YAXXZ, ?anchor@@YAHH@Z}` — 3.**
*Breaking condition targeted:* a user-named init section forcing an extra
thunk/root, or renaming the thunk, that §2 does not predict.

### a7c8 `a7c8_codeseg_static_unref` — `#pragma code_seg(".mytext")` over an unreferenced `static`
Clauses: naming the code section is not a root; `cand` internal + unreferenced.
**Predicted (code-characteristic reading): `{?anchor@@YAHH@Z}` — 1**, with
`anchor` expected to sit in a `.mytext`-named COMDAT rather than `.text`.
*Instrument note registered in advance:* under a strict `.text`-prefix reading
the same obj yields `{}`. That is an artifact of the name-prefix convention,
**not** a §2 violation, and will be reported as such.
*Violation condition:* `?cand@@YAHH@Z` present.

### a7c9 `a7c9_section_allocate_addrtake` — `#pragma section` + `__declspec(allocate(".mysec")) int (*g_p)(int) = &cand;`
Clauses: R5 (`g_p` external-linkage data ⇒ kept) → P1 (address-take in a data
initializer) ⇒ the `static cand` is kept. The pragma/`__declspec` only choose
the section.
**Predicted: `{?cand@@YAHH@Z, ?anchor@@YAHH@Z}` — 2.**

### a7c10 `a7c10_initseg_internal_datum` — `init_seg(compiler)` over an **internal** dynamic-init datum
`static int g_v = mk(seed());`, anchor reads `g_v`.
Clauses: R5 (non-const internal data ⇒ kept) ⇒ R4 ⇒ P1.
**Predicted: `{?mk@@YAHH@Z, ??__Eg_v@@YAXXZ, ?anchor@@YAHH@Z}` — 3.**

---

# A8 — PCH `/Yc` / `/Yu` (8 cells, 14 objs)

**Axis point:** with `/Yu` the front end does **not** re-parse the header text;
it restores parser state from the `.pch` image produced by the `/Yc` TU. §2's
fixpoint is per-TU (**S**) and says nothing about precompilation. Two things
could break: (i) roots that R1 establishes from header text might not be
re-established in a `/Yu` TU; (ii) keeps made by the `/Yc` TU might ride along
in the pch and contaminate every `/Yu` TU.

Shared `pcha.h` (a8c1–a8c4, a8c6): `inline ia, ib; static sa, sb`.

Every A8 cell is graded per obj. `pchgen.obj` is the `/Yc` TU's obj (it is a
real obj and is graded); `user.obj` is the `/Yu` TU's.

*Registered in advance:* if `/Yc`/`/Yu` cannot be made to work at all under
wibo (pch images are memory-mapped), that is reported as **INSTRUMENT-FAIL for
axis A8 only**, with the compiler's own diagnostics quoted — not as a
violation, not as a pass. Any `/Yc`↔`/Yu` flag interaction observed (e.g. a
required `/Fp`, a forced `/Fo`, warnings about mismatched switches) is reported
verbatim.

### a8c1 `a8c1_yc_no_refs` — the `/Yc` TU references nothing from the pch
Clauses: R1 anchor only; `ia/ib` COMDAT-unreferenced, `sa/sb` internal-unreferenced.
**Predicted `pchgen.obj`: `{?anchorg@@YAHH@Z}` — 1.**

### a8c2 `a8c2_yu_refs_ia_sa` — the `/Yu` TU references one inline and one static from the pch
**Predicted `pchgen.obj`: `{?anchorg@@YAHH@Z}` — 1.**
**Predicted `user.obj`: `{?ia@@YAHH@Z, ?sa@@YAHH@Z, ?anchoru@@YAHH@Z}` — 3**
(R1 anchoru → P1 ×2; `ib`, `sb` absent).
*Breaking condition targeted:* a pch-restored `static` failing to be emitted
even though a kept definition ODR-uses it.

### a8c3 `a8c3_yu_no_refs` — the `/Yu` TU references nothing
**Predicted `pchgen.obj`: `{?anchorg@@YAHH@Z}` — 1.**
**Predicted `user.obj`: `{?anchoru@@YAHH@Z}` — 1.**

### a8c4 `a8c4_nopch_control` — the same two user sources compiled with **no** pch
`user.cpp` here is byte-identical to a8c2's `user.cpp`.
**Predicted `user.obj`: `{?ia@@YAHH@Z, ?sa@@YAHH@Z, ?anchoru@@YAHH@Z}` — 3.**
**Predicted `user2.obj`: `{?anchoru2@@YAHH@Z}` — 1.**
This is the paired control that makes any A8 result crisp: a8c2's `user.obj`
must equal a8c4's `user.obj` **exactly**, and a8c3's `user.obj` must equal
a8c4's `user2.obj` up to the anchor's own name.

### a8c5 `a8c5_extern_def_in_pch` — an external non-COMDAT definition **inside** the pch header
`pchb.h`: `int ea(int x){…}` + `inline int ib(int x){…}`. Neither TU references
either.
Clauses: R1 fires per-TU (S) on `ea` in **both** TUs; `ib` is COMDAT and
unreferenced in both.
**Predicted `pchgen.obj`: `{?ea@@YAHH@Z, ?anchorg@@YAHH@Z}` — 2.**
**Predicted `user.obj`: `{?ea@@YAHH@Z, ?anchoru@@YAHH@Z}` — 2.**
*Breaking condition targeted:* `ea` missing from `user.obj` — i.e. R1's root
set is established at pch-creation time and not re-established for `/Yu` TUs.

### a8c6 `a8c6_yc_refs_yu_does_not` — the `/Yc` TU keeps `ia`+`sa`; the `/Yu` TU keeps neither
**Predicted `pchgen.obj`: `{?ia@@YAHH@Z, ?sa@@YAHH@Z, ?anchorg@@YAHH@Z}` — 3.**
**Predicted `user.obj`: `{?anchoru@@YAHH@Z}` — 1.**
*Breaking condition targeted — the sharpest A8 cell:* `ia` or `sa` appearing in
`user.obj` would show the pch carrying the `/Yc` TU's fixpoint state into every
`/Yu` TU, contradicting S.

### a8c7 `a8c7_pch_vtable` — polymorphic class defined in the pch; the `/Yu` TU constructs it
`pchc.h`: `struct C { int f; C():f(1){} virtual ~C(){} virtual int v(int);
virtual int w(int); };`
**Predicted `pchgen.obj`: `{?anchorg@@YAHH@Z}` — 1** (no ctor kept ⇒ V does not fire).
**Predicted `user.obj`: `{??0C@@QAA@XZ, ??1C@@UAA@XZ, ??_GC@@UAAPAXI@Z,
?v@C@@UAAHH@Z, ?w@C@@UAAHH@Z, ?anchoru@@YAHH@Z}` — 6** (R1 → P1 ctor → V).
*Breaking condition targeted:* V failing to fire, or firing in the `/Yc` obj.

### a8c8 `a8c8_pch_dyninit` — dynamic-init root in the `/Yu` TU over a pch-defined `static`
`pchd.h`: `extern int seed(); static int mk(int x){…}; static int nomk(int x){…}`.
`user.cpp`: `int g_v = mk(seed());` + anchoru reads `g_v`.
Clauses: R5 (`g_v` external data) ⇒ R4 `??__Eg_v@@YAXXZ` ⇒ P1 ⇒ `mk`.
**Predicted `pchgen.obj`: `{?anchorg@@YAHH@Z}` — 1.**
**Predicted `user.obj`: `{?mk@@YAHH@Z, ??__Eg_v@@YAXXZ, ?anchoru@@YAHH@Z}` — 3**
(`?nomk@@YAHH@Z` absent from both).

---

## Summary of the predicted sets

| axis | cells | graded objs | cells whose prediction is "a §2 root list that lists no such clause ⇒ absent" (violation-hunting) |
|---|---:|---:|---|
| A1 | 8 | 8 | a1c3, a1c7 |
| A5 | 9 | 9 | a5c1, a5c4, a5c6, a5c9 |
| A6 | 8 | 18 | a6c2, a6c3, a6c4, a6c5(tu2), a6c6, a6c7, a6c8 |
| A7 | 10 | 10 | a7c1, a7c2, a7c3, a7c7, a7c8 |
| A8 | 8 | 14 | a8c5, a8c6, a8c7 |
| **total** | **43** | **59** | |

Predicted-set cardinalities, in one table for scoring:

| obj | predicted count |
|---|---:|
| a1c1 main / a1c2 main / a1c4 main / a1c5 main / a1c6 main / a1c7 main | 2 each |
| a1c3 main | 1 |
| a1c8 main | 4 |
| a5c1 main / a5c4 main | 1 each |
| a5c2 / a5c3 / a5c5 / a5c7 / a5c8 / a5c9 main | 2 each |
| a5c6 main | 4 |
| a6c1 tu1,tu2 / a6c2 tu1,tu2 / a6c3 tu1,tu2 | 2 each |
| a6c4 tu1 | 2 · a6c4 tu2 | 1 |
| a6c5 tu1 | 6 · a6c5 tu2 | 2 |
| a6c6 tu1,tu2 | 2 each |
| a6c7 tu1 | 1 · tu2 | 2 · tu3 | 1 |
| a6c8 tu1,tu2 | 3 each |
| a7c1 / a7c2 / a7c3 / a7c8 main | 1 each |
| a7c4 / a7c5 / a7c6 / a7c7 / a7c10 main | 3 each |
| a7c9 main | 2 |
| a8c1 pchgen | 1 |
| a8c2 pchgen 1, user 3 |  |
| a8c3 pchgen 1, user 1 |  |
| a8c4 user 3, user2 1 |  |
| a8c5 pchgen 2, user 2 |  |
| a8c6 pchgen 3, user 1 |  |
| a8c7 pchgen 1, user 6 |  |
| a8c8 pchgen 1, user 3 |  |

## Grading rules, fixed now

* **MATCH** — observed code-COMDAT-leader set equals the predicted set exactly.
* **VIOLATION** — observed set differs, and the difference contradicts a named
  §2 clause under the derivation recorded above. Each VIOLATION is stated as
  *(clause, cell, the exact name added or missing)* and, per the prereg's guard
  1, is scored only after an independent re-derivation by a second agent from
  the cell source + §2's text alone; a disagreement downgrades it to AMBIGUOUS.
* **AMBIGUOUS** — observed set differs, but my prediction turns out not to be
  derivable from §2's text without an extra assumption (the registered
  alternative readings above are the pre-declared instances: a5c1, a5c4, a7c8's
  reader convention).
* Order of names within an obj is **not** graded (set comparison); `.rdata`/
  `.data` leaders are recorded as context only.
* For every VIOLATION I additionally report whether the breaking condition is
  detectable from **c1xx-side observables** (source text, IL/`.ex`/`.gl`,
  `/W4`+`/Wall` C4505/C4514), and demonstrate the detector on the violating
  cell where one exists.

## Artifacts

* Cell sources + per-cell compile plans: `work/emitpred/axes1/cells/<axis>/<cell>/`
  (`spec.json` carries the invocations).
* Generator (single auditable source of every cell): `work/emitpred/axes1/gencells.py`.
* COFF reader: `work/emitpred/axes1/leaders.py`.
* Phase-2 runner: `work/emitpred/axes1/runcells.py` (base flags `/O1 /Oi /EHsc
  /GS- /c`, 120 s per-invocation timeout, builds under `work/emitpred/axes1/build/`).
* Results will be written to `work/emitpred/axes1/results.json` and graded in
  `work/emitpred/axes1/RESULTS.md`.
