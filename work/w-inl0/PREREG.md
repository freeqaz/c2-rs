# w-inl0 — PREREGISTRATION

    Lane:     w-inl0
    Branch:   w-inl0, worktree off master `71e38a2`
    Target:   board #980 — the 370-body `tail | port-longer | sub+ins | opcode`
              cluster: port emits `li rN,0 ; b callee`, c2's whole body is one
              `blr` with ZERO relocations, 211 TUs.
    Blocker:  board #966/#971 — the callee's IL body is PARSE-REFUSED today,
              production `expr-intrinsic-memset` (369 of the 370; the family
              blocks 572 differs, of which 203 are mechanism I).

**Committed before any cell is compiled and before any IL is captured.** Nothing
below has been measured; every number is a prediction, and §6 says what result
makes me abandon the rung.

---

## 0. What is already known, and what is therefore NOT a finding of this lane

Read off the record, not measured here:

* `elide.rs` mechanism **E** fires when the callee's body **decodes empty**
  (`IlFunction::empty_body`), closed under itself by the `TuEmptyCallees` least
  fixpoint (`w-empty`, `w-fix`). A parse-refused body is not empty, so E
  correctly does not fire on the 370.
* `expr-intrinsic-memset` is intrinsic selector **173**, whose production is
  `33 86 41 74 80 ad 00 00 00` + `40 <TYPE>` + args + `4C`
  (`docs/IL_INTRINSIC_CALL.md` §1, `docs/IL_CAST_CONVERT.md` §1.3), and whose
  *ordinary* c2 expansion is `b <memset>` **with a REL24** — i.e. the intrinsic
  is emphatically **not** free in general.
* The canonical pair is `??$_Destroy_Range@PAH@stlpmtx_std@@YAXPAH0@Z` (caller,
  port `li r5,0 ; b`) → `??$__destroy_range@PAHH@stlpmtx_std@@YAXPAH00@Z`
  (callee, parse-refused), and **c2 emits `4e800020` for both**
  (`w-seq` §4.4, dumped from a compiled obj).
* STLport's source for the callee (`src/system/stlport/stl/_construct.h:172`)
  is `__destroy_range_aux(__first, __last, _Trivial_destructor())`, where for a
  trivially-destructible element type `_Trivial_destructor` is `__true_type` and
  the selected `__destroy_range_aux` overload has an **empty body**.

So the *outcome* (c2 emits nothing) is known. What is not known, and what this
lane must measure, is **why the IL says so** — and whether that can be read from
the IL without guessing.

---

## 1. THE GRAMMAR HYPOTHESIS — H1

> **H1.** The `expr-intrinsic-memset` refusal in `??$__destroy_range@PAHH@…` is a
> body consisting of **exactly one statement**: an intrinsic-173 call whose
> **destination operand is the address of a function-local automatic** declared
> in this function's `.sy`, whose **count operand is a small literal** (predicted
> `1` — `sizeof(__true_type)`, an empty tag struct), and whose destination local
> is **read nowhere else in the segment**. Everything else in the segment is the
> standard prologue/epilogue plumbing (`4C 4F 11 53` … `54 02` … `4F 12 47 54 01
> 54 00`).
>
> The construct it comes from is the value-initialization of the empty tag
> object `_Trivial_destructor()` that is passed by `const&` to an inline callee
> with an empty body: c1xx materializes the temporary (a memset over its 1 byte)
> and inlines the callee to nothing, leaving a store to a dead stack slot.

### 1.1 The predicted decode, field by field

`docs/IL_CAST_CONVERT.md` §1.3 records that `memcpy`/`memset`/`memcmp` carry
**leading literal alignment hints the source does not have** (`?t_memset` pushes
one; `Dir.cpp` fn931 pushes `04` where the operands are 4-byte aligned).
Predicted argument list, in stream order:

```
33 86 41 74 <align:lit>  55 86 41 74      the alignment hint          (predicted 01)
<addr-of local>          55 <ptr type>    the destination             (a .sy automatic)
33 86 41 74 <fill:lit>   55 86 41 74      the fill byte               (predicted 00)
33 86 41 74 <count:lit>  55 86 41 74      the count                   (predicted 01)
4C                                        apply
4B                                        discard the void result
```

**Predicted census/refusal offset:** the `33` of the selector literal, as
`intrinsic_call_census_reports_the_selector_not_the_opcode` already pins for the
other four ids.

### 1.2 Falsifiers for H1 — any one of these kills it

1. The destination is a **formal**, a **global/data symbol**, or reached through
   a pointer parameter, rather than a local automatic.
2. The count is **not a literal**, or is larger than the local's own size.
3. The segment carries **any other statement** — a second call token, a store to
   a formal's target, a branch — beside the plumbing.
4. The local written by the memset is **read** later in the same segment.
5. The refusal is not at the selector, i.e. the production is reached by a route
   `intrinsic_selector` does not recognize.
6. The body is not one statement but a **loop** (the `__false_type` overload's
   shape) — which would mean the 369 are not the shape this lane thinks.

Any of 1–4 firing means "emits nothing" is **not** readable from this body and
the rung declines (§6).

## 1.3 H2 — the cell hypothesis

> **H2.** A minimal cell of the same *shape* — an empty tag struct
> value-initialized as a temporary and passed by `const&` to an empty inline
> function, instantiated so that the wrapper is emitted as its own COMDAT —
> reproduces (a) the same `expr-intrinsic-memset` refusal key at the same
> production, and (b) `4e800020` with **zero relocations** as c2's whole COMDAT
> for the wrapper.

Falsifier: the cell produces no selector-173 site, or c2 emits anything other
than a bare `blr` for the wrapper.

## 1.4 H3 — the general claim the rule would rest on

> **H3.** A body whose only content is an intrinsic memset into a **dead local
> automatic** is a body c2 emits **nothing** for, at the workload's flags and at
> `/Ob0`, and the fact is a property of the **decoded IL** and not of the
> relocation count.

Falsifier, and the one I most expect to lose: a cell in which the memset's
destination local **is** subsequently read shows c2 keeping bytes (good — that
is the guard earning its place), **or** a cell in which the destination local is
dead shows c2 keeping bytes anyway (fatal — the rule is unsound and the lane
declines).

**#950's hazard is inherited verbatim.** The relocation observable reads
"nothing happened" on a self-recursive body that is plainly not nothing. Every
verdict in this lane is printed as the caller's **whole `.text` words**, never
as a relocation count, and the rule is keyed on the **callee's decoded IL**.

---

## 2. THE SHIPPING PLAN, and the two doors it deliberately does not open

The rule extension is *"the callee's body inlines to nothing"*, keyed on the
callee's decoded IL, seeded into the **existing** `TuEmptyCallees` least
fixpoint so that the closure and the cycle refusal are inherited unchanged.

**Preferred door (`SEED-ONLY`).** The body stays **parse-refused for emission**;
a separate, narrow predicate over the same segment answers *"does c2 emit
anything for this body"*, and its `true` **seeds** the fixpoint. Consequences,
registered:

* `IlBundle::functions()` is **untouched** — #971 condition 4, #878's loaded gun.
* the port never emits a body for the callee, so the callee cannot become a new
  wrong emit; it stays in `fnbyte-refused`.
* the only bytes that move are the **callers'**, from `li rN,0 ; b callee` to
  one `blr`, which is E's shipped body.

**The other door (`ACCEPT`)** — making the body a real `BodyShape` the census
and the emitter both accept — is an **instrument widening**: `functions()`
widens by construction (it refuses a TU when any function refuses), which is
exactly what #971 condition 4 forbids in this commit. It is not taken. If
measurement forces it, the widening is declared as a widening, with counts, and
every new differ named.

**FORBIDDEN, stated so it cannot be reached by accident:** a name-based special
case (`_Destroy_Range` / `__destroy_range` by mangled pattern, or any STL
spelling). That is a neutrality classifier by another door. The rule must be a
property of decoded bytes only.

---

## 3. THE PREDICTED END STATE

Baseline (master `71e38a2`): FBM `exact 35,982 · differs 3,195 · partial 0 ·
refused 130,573 · unbound 9,225` of 178,975 · `elided 1,516 / elided-exact
1,516` · TU match 10 · mismatch 0 · vocab-gap 861 · tests 978/30 · gate 18/18.

| key | predicted | band |
|---|---:|---|
| `fnbyte-differs` | **2,825** | 2,820–2,830 (369 ± the one external-callee row of #966 row 8) |
| `fnbyte-exact` | **36,352** | the same count, opposite sign — **exact must never shrink** |
| `fnbyte-elided` / `-elided-exact` | **1,885** / **1,885** | equal to each other, always |
| `fnbyte-refused` | **130,573** | **unchanged** under `SEED-ONLY` — the callee is still refused |
| `fnbyte-partial` · `-unbound` | 0 · 9,225 | unchanged |
| functions moved the **wrong** way | **0** | checked **per symbol**, never by subtracting totals |
| `mismatch` · TU match · vocab-gap | 0 · 10 · 861 | unchanged |
| controls `partition-broken` · `match-TU-differs` · `census-disagree` | 0 · 0 · 0 | unchanged |

**370 is a count of bodies, not of conversions.** `w-empty` closed 1,516 and TU
match went 10 → 10; trap 8 and `DIFF_STRUCTURE.md` §5 both say so. TU match is
predicted **unchanged at 10** and a movement there would be a surprise, not a
success criterion.

---

## 4. CONTROLS AND MUTATIONS — registered before the grid exists

1. **The grid is frozen before `cl.exe`.** Every cell's `sha256` is recorded in
   `work/w-inl0/CELLS.sha256` and committed before the first compile, `w-fix`
   discipline. A cell whose anchor (a call to a symbol the TU does not define)
   loses its REL24 is refused, not read.
2. **Every cell is compiled twice** — the workload's flags and again with
   `/Ob0` appended — so E cannot be confused with I (`w-fix` #954).
3. **Per-edge scoring**, never per cell.
4. **Mutation M1 (guard removal, must go RED and must actually mutate).** Remove
   the *"destination is a local automatic"* test from the new predicate and
   assert that a named cell — one whose memset writes through a **formal** —
   flips from refused to admitted. The mutation is verified to have changed the
   file it names (`git diff --stat` non-empty on that hunk) before the run is
   read, which is #951's precedent.
5. **Mutation M2.** Remove the *"the local is never read"* test and assert the
   corresponding cell flips. Same verification.
6. **Mutation M3 (the fixpoint's own guard).** The existing
   `the_round_ceiling_cannot_fire` and `a_cycle_is_not_elided_and_terminates`
   must still hold with the new seed in the bundle: a cycle beside a
   no-effect-seeded chain admits the chain and not the cycle.
7. **The known answer.** For every symbol the port newly elides, c2's own COMDAT
   is dumped in words. Predicted `4e800020`, one word, zero relocations, on
   **100 %** of them; any exception is named individually and, if it is not
   `blr`, the rule is wrong and is withdrawn.

---

## 5. PREDICTIONS I EXPECT TO LOSE

Registered so that a loss is a finding rather than an embarrassment:

* **L1.** That the count literal is `1`. The temporary is an empty struct;
  MSVC's `sizeof` for it is 1, but c1xx may materialize a padded or aligned slot,
  or hoist several temporaries into one memset.
* **L2.** That all 369 share **one** body shape. `#644` — a producer is not one
  contiguous field — and the 211-TU spread make several sub-shapes plausible.
  If the 369 split, the lane ships the sub-production its cells actually pin and
  counts the remainder, which is the `w-rtti` landing.
* **L3.** That `fnbyte-refused` is unchanged. If the seed predicate has to run
  through the ordinary parser to see the operands, refusals may reclassify.

---

## 6. THE DECLINE CLAUSE — what makes this lane land as a decline

Any of the following and the lane ships **the measured grammar + the reader
extension for the sub-production the cells actually pin, with the elide
extension DECLINED and the blocking remainder counted** (the `w-rtti`
precedent — a correct decline with the road named beats a guessed rule):

1. Falsifier 1–4 of §1.2 fires: "emits nothing" is not readable from the body.
2. H3's fatal branch: a dead-local memset cell shows c2 keeping bytes.
3. The 369 do not share a shape that one predicate covers (L2), and the covered
   sub-population is smaller than a quarter of them.
4. Any measured route by which the rule could fire on a body c2 emits bytes for,
   that the grid cannot close.
5. `SEED-ONLY` proves impossible without widening `functions()` in the same
   commit (#971 condition 4).

**Landing either way requires:** `cargo test --workspace --release` at 978/30 +
this lane's additions, `scripts/gate.sh --jobs 6` 18/18 PASS and 0 mismatch,
`scripts/status.sh --check`, `scripts/board_audit.sh`, rows **#990**+ in
`docs/BOARD.md` in numeric position, and `scripts/gen_rung_index.sh` for
`rungs/INDEX.md`.
