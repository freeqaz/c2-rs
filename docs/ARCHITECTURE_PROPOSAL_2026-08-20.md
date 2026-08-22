# ARCHITECTURE PROPOSAL — the staged pipeline, and how to get there without losing the 26

    Status:    proposal for review. Analysis only — nothing in crates/ moved.
    Date:      2026-08-20
    Basis:     tree 977827d78 (clean), STATUS generated block of 2026-08-19
               (match 26 / mismatch 0 / vocab-gap 844), CEILING.md §6/§6.1,
               STRATEGY_REVIEW_2026-08-13.md (Option A decided §8.1),
               ARCHITECTURE_SEAMS.md, SHIPPING_ROADMAP_2026-08-19.md (peer,
               uncommitted), and the code cited inline.
    Judge:     unchanged and untouchable — real c2.dll under wibo, byte-exact
               obj compare, timestamp zeroed. Nothing here proposes a second
               judge; the stage oracle in §4 is a development instrument in
               exactly the sense SHIPPING_ROADMAP §2 states.
    Amended:   2026-08-21, lane `w-archamend`, against
               `docs/ARCH_REVIEW_2026-08-21.md` (seven review lenses, itself
               amended the same day), `docs/STEP5_PRICING_2026-08-21.md`, and
               the measurements of the very lanes §5 commissioned. **Every
               amendment is in place and visible**: refuted text stays under
               ~~strikethrough~~ with the row that killed it named beside it,
               per the step-1 row's own precedent (`765095b42`). Nothing below
               is silently rewritten. **The verdict, recorded and not softened:
               step 5 as written is NO-GO** — but read §4.1 for WHY, because
               the reason changed under measurement inside 24 hours. The
               review said COLOR could not be graded; lane `w-restim` measured
               that it can (#3351), and that **a PORT cannot be graded against
               it at any stage**, because the port→tuple projection is
               undefined rather than unequal (#3354). Per-stage observability
               buys **characterization, not a differential grade**. Two hard
               prerequisites still appear in no row of §5 and no line of §6,
               and they are now priced: **15–45 engineer-months, lower bound**
               (#3355).

---

## 0. Corrected measurements (the brief this responds to had four wrong)

Measured at this tree before anything below was written:

| claim | measured | note |
|---|---|---|
| "163k lines across 5 crates" | **178.1k** total; src: c2-il 78.2k, c2-core 48.7k, c2-harness **31.8k** (+13.6k `tests/`), c2-obj 2.4k, c2-reference 2.1k | the harness "45.3k" includes its integration tests; no other crate has a tests/ dir of size |
| "27 paired by identical filename" | 27 shared names, of which `mod.rs`, `testutil.rs` are structural and `calls.rs` pairs two *different* subsystems (shape machinery vs the frame spine) — **~24 real shape pairs** | the pairing claim survives; the count was padded |
| "the reader is big because shape-structured" | shapes/ is 33.9k of c2-il's 78.2k, and **44% of shapes/ and 46% of codegen/ are comment lines**; `mcall.rs` (5.4k) accepts nothing — it is the census instrument | the reader is big because it is *evidence-dense* and hosts instruments, not only because it is shape-structured |
| "frontier-codegen reads reader 22, wrong 0, refused 0" | confirmed — `denominator 23 · exact 1 · wrong 0 · refused 0 · reader 22` (CEILING §6.1 banner, 2026-08-19) | and `frontier = 2`, both TUs declined at ~20 refusals each |

---

## 1. Verdict on the current architecture

### 1.1 What is right and must be preserved

1. **The warranty stack.** `c2-reference` + the capture cache + `gate.sh`'s 18
   enumerated lanes + the generated sweep + the mode cross + the debug row +
   the gap-metric registry. Every retraction in the record came from **widening
   an instrument, never from a gate going red** (STATUS, stated four times);
   all five wrong-emit families were found by probe grids or instrument
   widenings. This is not overhead beside the product — it *is* the product's
   correctness argument, and it is what makes every migration step below
   verifiable in about a minute of wall clock.
2. **The construct-rung mechanism** — re-express an already-byte-exact class
   through new machinery at a **required-zero byte delta**, graded by identity
   diff of per-lane gate counts (#290's pattern). `block_ir`, `cond`, the
   `labels` map and the `w-layout` position-ownership change all landed this
   way. **This is the migration vehicle for everything in §5.** No rewrite is
   proposed anywhere in this document; every step is a re-expression graded by
   the identity protocol ARCHITECTURE_SEAMS §0 already executed four times.
3. **The measured-rule middle that already exists.** `frame`, `alloc`, `order`,
   `schedule`, `elide`, `splice`, `comdat`, `coff/` — ≈12k lines of genuine
   backend machinery, each rule established by pre-registered rival
   elimination (52,416 allocators refuted, 13,104 schedulers refuted, 2,470
   graded splice callers). These survive whole: first as production rules for
   their regimes, permanently as regression fences for the ported algorithms.
4. **The refusal-first rule and two-sided fence pricing** — kept verbatim.
5. **The whitebox record** — `ref/ADDR.tsv`, `FUNCS.tsv` (4,917/4,917
   functions), the six subsystem pages, the provenance legend, DISCLOSURE.md.
   §4 makes this the load-bearing input rather than a side archive.
6. **The ~24 shape files as witnesses.** Each is a fenced byte-exact specimen
   with graded negative cells on both sides of every clause. They are the
   cheapest regression corpus the general pipeline will ever get. They are
   *demoted from production dispatch* over time (§5 step 6) — never deleted.

### 1.2 What is wrong, in order of what it costs

1. **Parse == accept:** ~~there is no lossless decode layer.~~
   **RETRACTED 2026-08-21 — the headline was false when it was written**
   (arch review §6; **#3332**). A lossless decode layer has shipped since K1:
   `crates/c2-il/src/codec.rs`'s `IlModel::parse`/`encode` frames every byte of
   `.ex` and `.gl` into typed spans, coalesces unrecognized runs into
   `Span::Opaque`, and is **fail-closed** — it returns `CodecError::CannotRoundTrip`
   rather than a model it cannot re-serialize. `ir0` ran it on the workload for
   the first time and it is **870 of 870 TUs, 0 broken**. The misdiagnosis had a
   measured cost and it is not hypothetical: `ir0` built a *second, weaker*
   framer against this sentence and reverted it, where the stronger invariant
   already in the tree needed one `if`.
   **The complaint that survives, restated, and it is the one that costs:**
   `IlBundle::functions()` (`crates/c2-il/src/func/bundle.rs:1968` — this read
   `:699` in the first draft, which is inside the `OPT_WORD_*` constant block;
   lane `ir0` measured `:1939`, then `:1961`; `w8sum`'s sum type moved it again
   to `:1968`, so **the anchor has now been stale in three successive trees that
   carried it** — a raw line number into a live file is a drift-prone anchor and
   `board_audit.sh` exempts these only inside frozen rung files) couples
   framing, name binding, semantic understanding and *admission* into one
   all-or-nothing verdict. Decode being lossless somewhere else in the crate does
   not un-couple them: the four questions are still answered by one `Option`
   return, and nothing downstream can consume a body whose TU that return refused. Consequences: (a) the whole recognized function is the only unit
   that can exist downstream, so every new capability is a new whole-function
   grammar; (b) the admission verdict is **all-or-nothing per TU** — one
   unrecognized body refuses the whole translation unit, which is what
   `vocab-gap 844` counts, so no middle stage can be built against a *whole*
   TU's data. **`vocab-gap 844` is a TU-level admission count and NOT a claim
   that 97% of bodies fail to decode** — the per-function census decodes a
   `BodyShape` for a large minority of bodies workload-wide, and an earlier
   draft of this document misread the TU figure as a per-body one. Any
   per-body denominator quoted downstream must come from a census run, cited
   with its workload stamp, never from this key; (c) the
   refusal boundary is structurally pinned in the parser —
   `frontier-codegen-refused` is 0 *by construction* (#1475), and that is fixed
   by the claims ledger (§3.3), not by step 1. Cost evidence:
   113,612 of the 126,315 additionally-needed emitted functions are blocked at
   the reader (CEILING §6, #3092).
2. **The middle is point-rules, not a pipeline, and `passes/mod.rs` is a
   30-line placeholder.** Real c2 runs a 35-pass pipeline with COLOR at index
   14 and a cycle-driven dependence-DAG scheduler run four times per function
   at `/O1` (`P_DAG.md` §1, driver `0x10be6382`). The port's alloc/order/
   schedule rules are byte-true in narrow regimes (≤3 producers, one base
   symbol, …) and the exhaustive rival refutations are the **measured proof
   that black-box recovery of the scheduler/allocator does not converge**. The
   endgame algorithms must be ported from the disassembly — which is
   authorized, encouraged, and already mapped to subsystem pages.
3. **`IlFunction` is ~~~15~~ 34 parallel `Option` shape fields**
   (~~`crates/c2-il/src/func/mod.rs:2980`~~) **and `select_function` re-derives
   the shape in a 38-return ordered match.** Named as debt in
   ARCHITECTURE_SEAMS §2.3b on 2026-07-30, scheduled as "step 5, with W8",
   ~~still unbuilt~~.
   **AMENDED 2026-08-21 (arch review §6; `docs/rungs/2026-08-21-w8sum.md:1,24`).**
   The count was never "~15": it was **34**, as the lane that retired them
   measured and titled itself.
   **The count is cited to the rung and NOT to board #844, deliberately.**
   #844 is `w-alloc2`'s *"the store-run emitter is leaf-only by construction"*
   and contains **zero** occurrences of the word `Option`; it is about the run
   and the frame being two emitters that do not compose. The attribution chain
   that says otherwise runs `w8sum` rung `:23-24` → board **#3345**'s headline
   (*"BOARD #844's '34 mutually-exclusive `Option<Shape>` fields…'"*) → the
   arch review → this document's first amendment draft. **#844 is the right
   row for the half-emit hazard and the wrong row for the field count**; the
   two got welded together on the way here and are separated again now.
   **And the debt is now paid**: §5 step 2 landed as lane `w8sum`
   (merge `c7049bac1`, 2026-08-21), collapsing the 34 fields into one
   `pub body: BodyShape` of **35 variants** and turning `select_function`
   (`crates/c2-core/src/codegen/select.rs:275`) into a **total, exhaustive
   35-arm `match`**. The invariant "exactly one is `Some`" is the `enum` now,
   not a convention — and the half-emit hazard #844 *does* name (a store run
   without its `bl`, or a `bl` without its run) is unspellable by construction.
   *(Anchors deliberately not replaced with new line numbers: the whole point
   of this bullet's own history is that a raw line number into a live file
   drifts. Symbol names are stable; `:2980` is deleted rather than refreshed.)*
   ~~Every new class costs a field, an arm, and a name-paired
   file — the linear-coverage signature the review's H1 describes.~~ It now
   costs **a variant and an arm**, and the arm cannot be forgotten. The
   name-paired file is untouched by step 2 and is step 6's business.
4. **The TU-level object plan has no home.** The emit-set closure (factor A —
   mechanism already read: the `0x20` bit at `sym+0x4c` closed under
   "referenced by an already-emitted function", `C2_MAP.md` §3E), weak
   externals (~~675 TUs~~ **674 TUs / 4,009 records**), COMDAT synthesis
   (~~450 TUs~~ — **and the associative-COMDAT population is 846 TUs /
   101,120 sections, which is a different set**), the section ladder (3
   names left) are scattered across `comdat.rs`, `coff/`, `elide.rs`,
   `splice.rs` and unbuilt phases. **AMENDED 2026-08-21 (#3331, `w-objplan`
   §2.8):** the reference-side inventory re-derived both figures this bullet
   had only *carried* from the roadmap with no locator. This matters because **the remaining
   distance is a conjunction**: 523 of 845 remaining TUs fail A, B and C
   *simultaneously* (~~`w-871`~~ **`w-vocabgap` / #3189** — citation corrected
   2026-08-21; `w-871` crosses by *mechanism* and states no such figure, and
   #3189 is where `--factors-tsv` restricted to the 845 reads A false on 842,
   B on 546, C on 701, and 3-of-3 on **523**), a perfect reader converts 2
   (~~#3191~~ **#3190** — corrected the same day, arch review "what survives";
   #3191 is the per-TU blocking-set distribution, a different row), a perfect
   section emitter converts 0 (#3210), item F converts 0 (`w-itemf`).
   *(The `845` is #3189's population at its own stamp and is kept as measured;
   `vocab-gap` reads **844** at this tree. Both slips above were copied
   forward verbatim from `STRATEGY_REVIEW_2026-08-13.md:485-489` and are fixed
   there in the same commit series.)*

   > **AMENDED 2026-08-21 — A∧B∧C∧(D∨E) is measurably NOT NECESSARY, and there
   > is one named exception.** The factor TSV header
   > (`crates/c2-harness/src/gap/factors.rs:670`) asserts *"A byte-exact obj
   > requires A and B and C and (D or E)"*, and this bullet restates it as if
   > it held without exception. **`src/system/decomp_pch.cpp` is a `match`
   > with A FALSE** — letters `-BCDE`. It is derivable from two independent
   > places in the repo without re-running a scan: its reference obj is
   > **901 B with zero `.text`** and `emit-emitted` **0** (`CEILING.md` §2.5's
   > amendment table), so obj `.text` COMDATs = 0 against a non-empty `.ex`
   > and factor A — *"`.ex` segments == obj `.text` COMDATs"* — is false; and
   > it is one of the six matched TUs the stage oracle names as producing a
   > structurally empty snapshot (`rungs/2026-08-20-stageoracle.md` §3). Note
   > that D is **vacuously** true there, over zero emitted COMDATs.
   > **This does NOT weaken the conjunction argument**, which survived review
   > as the strongest part of this document: the conjunction is about where
   > the *refusal mass* sits and why single-stage completion converts ~0, and
   > every figure in it stands. What is corrected is the word *requires* — the
   > conjunction is the route for TUs with bodies, not a law over all TUs, and
   > the whole-TU recognizer path (E) can convert a TU that fails A. No
   single seam moves `match`; only the composed plan does — so the plan needs
   to be a *thing* that can be graded as a whole (§3, IR2).
5. **A slice of the instrument mass measures the shape architecture itself**
   (the `disp-*`/`prod-*` axes, the 37 mcall tags, parts of the census). Not
   waste — it answered questions nothing else could — but it is re-pointed at
   stage boundaries in the target design, not grown.

### 1.3 The commissioning hypotheses, scored against the code

* **H1 "a lookup table wearing a compiler's clothes" — HALF.** The catalogue
  is real (38 dispatch returns; ~24 name-paired files; several are explicit
  one-function transcriptions and say so). But the newer classes are
  rule-shaped, not row-shaped — `counted_accum_loop` is 7 opcodes × 2
  signednesses × every simm16 init, derived from the whitebox reading of c2's
  own three-pass loop lowering with a graded negative cell per clause. And
  the bimodal result is **not** explained by enumeration alone: the
  call-bearing classes sit at 0.000 because scheduler/allocator/inliner
  interaction on real bodies is unported — the missing middle, not the
  missing rows.
* **H2 "there is no real middle" — TRUE for the pipeline, with a correction.**
  `passes/` is empty, confirmed. But the middle's *foundations* exist
  (block_ir + cond + labels + frame + the measured rules, ≈12k lines) and were
  built by exactly the zero-delta mechanism the migration needs. The middle is
  not absent; it is embryonic and has no substrate to run on (§1.2 item 1).
* **H3 "effort aimed at the wrong stage; the reader is the constraint" —
  DIRECTION REFUTED by the repo's own measurements.** The reader holds the
  refusal *mass*, yes — but a perfect reader converts **2 TUs** (~~#3191~~
  **#3190**, corrected 2026-08-21);
  lifting the entire `.gl` walk measured `match +0 / fnbyte −65` (#3093); a
  real unpoisoned decode widening measured a required-zero delta (#3104).
  The binding constraint is the **conjunction** (§1.2 item 4). Pouring effort
  into the reader *alone* is the same mistake as pouring it into the emitter
  alone; the fix is a design in which each stage's progress is independently
  gradeable so the conjunction stops hiding everything (§3).
* **H4 "the harness is instrument mass a cleaner core would not need" —
  MOSTLY NO.** 31.8k src + 13.6k tests, and the record says the instruments,
  not the gates, found every defect and every false claim. What a cleaner
  core removes is not the harness but the *growth rate* of whole-object byte
  archaeology — which is precisely what the stage oracle (§4) buys.

---

## 2. The design constraint that everything below serves

Option A is decided: **reproduce c2 fully; the target is 870/878** (STRATEGY
REVIEW §8.1). The review's own arithmetic makes the shape of the solution
non-optional:

* ~~no intermediate match count pays (hybrid speedup 1.03× at 26/870; value
  arrives near p≈0.9)~~ **— REASONING RETIRED 2026-08-21, CONCLUSION KEPT.**
  That premise is a *throughput* argument, and throughput is no longer the goal
  (`GOAL_DECISION_2026-08-21.md`). The conclusion survives on the goal itself:
  goal (2) is **parity**, and parity is `match` → 870/878, so partial coverage
  does not pay in proportion either way — the architecture must still optimize
  for *eventual totality*, not for the next conversion;
* the rung-shaped route prices at 3,400–10,400 lanes — dead;
* function counts read ~9× optimistic as bytes — progress must be graded in
  bytes and in structural manifests, not in rows admitted.

So the design goal is: **a pipeline in which every stage has (a) a lossless
input it can be total over, (b) an invariant that is checkable against the
reference without the downstream stages existing, and (c) a refusal predicate
that names itself.** That is what dissolves the conjunction into independently
movable curves.

---

## 3. Target architecture

### 3.1 The IRs, named, with the invariant each one holds

| IR | name | what it is | invariant (checkable today, without downstream stages) |
|---|---|---|---|
| IR0 | **RecordStream** | lossless framing of all five bundle streams (`.ex`/`.gl`/`.sy`/`.in`/`.db`): typed records, each carrying its exact byte span; unknown record kinds are opaque-with-span, never dropped | **totality + byte identity**: every byte of every stream owned by exactly one record; re-serialization reproduces the input byte-for-byte. Refusal at this layer means *malformed input*, nothing else. Denominators (`unanchored`, `fail-closed`, `opaque`) print on every scan — trap-0's discipline made structural |
| IR1 | **SymGraph** | the symbol/type/binding model: `.gl` records ↔ `.ex` segments ↔ names ↔ aliases ↔ data initializers, one binding (closing the two-splitters seam ARCHITECTURE_SEAMS §0.1 pinned apart) | **accountability**: every symbol in the reference obj is claimed by exactly one IR1 node or counted in a published residue (factor B's ledger, `.gl` invariants block) |
| IR2 | **ObjPlan** | the TU-level plan, independent of body bytes: emit-set closure (the whitebox reachability rule), dependency order (#259's rule), section set + ladder, COMDATs + checksums + associativity, weak externals, relocation inventory, EH/`.pdata` records, label-counter charges | **structural manifest equality**: section order/attrs, symbol order/fields, reloc locations/types, COMDAT associations, EH inventory match the reference obj — gradeable on **all 870 TUs today** because none of it needs body bytes. This is the new continuous progress curve for A∧B∧C |
| IR3 | **Mir** | typed values + explicit CFG: today's `block_ir` (blocks, terminators, placement-order-is-explicit) + `cond` (item E) extended with values and cross-block liveness (item F) | **attributability**: every instruction in a finished body names the pass that placed it; block order is a stated choice; a `Bc` and its producer cannot disagree (already enforced by `BodyLayout::place`) |
| — | **Passes** | c2's own pipeline in observed order: the optimizer passes the pinned workload exercises, COLOR (index 14), the dependence-DAG scheduler — **ported from the disassembly**, graded per-stage against the stage oracle (§4), admitted regime-by-regime | per-pass: output equals the real pass's serialized state on the graded corpus; per-regime: the existing measured rules (`alloc`, `order`, `schedule`) become the fences and the regression tests |

`Lower`/`Emit` stay what they are: the selection tables
(`WB_SELECT_RECONCILED.md`), `encode`, `frame`, the COFF writer.

### 3.2 Crates and modules

Std-only, zero external crates, unchanged. No new workspace crate is strictly
required; the layout below keeps every seam a module boundary first and makes
crate splits available later if contention demands them (that is
ARCHITECTURE_SEAMS' own lesson: split for contention, not for aesthetics).

```
crates/c2-il/
  stream/        IR0: the lossless framer + re-serializer + totality controls
  sym/           IR1: bind.rs, glalias.rs, sy.rs, ininit.rs consolidated behind
                 one binding (the bindings-pinned-apart test retires here)
  sem/           the typed expression/statement decoder (grows out of
                 expr.rs/readers.rs; produces IR1-attached bodies with opaque
                 nodes where understanding stops)
  func/body/shapes/   UNCHANGED during migration; demoted per §5 step 6
  census / mcall      instruments, re-pointed at IR0/IR1 denominators

crates/c2-core/
  plan/          IR2: comdat.rs, elide.rs, splice.rs, coff/order.rs, the label
                 plan, + the unbuilt pieces (emit-set closure, weak externals,
                 COMDAT synthesis, RTTI/.rdata$r, .text$yd, .xdata$x)
  mir/           IR3: block_ir.rs, cond.rs, labels.rs (+ item F when built)
  passes/        finally non-empty: ported passes, COLOR, the DAG scheduler;
                 alloc.rs / order.rs / schedule.rs live here as the regime
                 rules until superseded, then as #[cfg(test)] fences
  lower/         select.rs, straightline.rs, leaf/, the shape lowerings
                 (until demoted)
  emit/          coff/ writer, encode.rs, frame.rs

crates/c2-obj, c2-reference    unchanged
crates/c2-harness              unchanged in role; gains the stage-oracle
                               capture/compare (§4) so it runs under gate.sh
                               (#1406: evidence-producing instruments live in
                               the workspace, never in scripts/ alone)
```

### 3.3 Where the refusal boundary lives, and how it stays precise

Today: one boundary, in the parser — a function the reader does not recognize
whole does not exist. Target: **a claims ledger per TU**, evaluated at emit
time, every claim decidable and printable:

1. **decode-total** — IR0 framed every byte (opaque count 0 *within the
   reachable emit set*; opaque elsewhere is fine and counted);
2. **bound** — IR1 accounts for every symbol the plan will emit;
3. **plan-complete** — IR2's manifest contains no record the writer cannot
   express (weak externals, section names, …);
4. **lowerable** — every body in the emit set is inside the shipped pass
   coverage: the regime predicates (`MAX_MODELLED_PRODUCERS`, the signedness
   fences, `LabelMap` invariant 4, …) stay exactly as sharp as they are today.

Emit iff all four hold; otherwise refuse, and **the failing claim is the
census key**. This is *more* precise than today's boundary, not less: today a
refusal names where a recognizer gave up; a claim names which stage owes work.
`NotImplemented` semantics, the wrong-emit-scores-worse rule, and fail-closed
degradation are unchanged. Promotion of any general lowering into claim 4
follows the H2 replacement protocol already in doctrine: two-sided fence
pricing, offline FBM grading over the 162k emitted functions before an obj is
ever emitted, and a generator for every shape the predicate admits (#283's
asymmetry is the standing reason).

> **AMENDED 2026-08-21 — the claims ledger is UNBUILT and no step in §5 builds
> it** (arch review §2). `sym/`, `sem/`, `mir/`, `lower/`, `emit/` do not
> exist; `passes/mod.rs` is a 30-line comment. Yet §5 step 5 admits ported work
> *"behind claim 4"*, which is a dependency on a structure nothing in the table
> constructs. Either a step builds the ledger or step 5 must be worded against
> **today's** refusal predicates (`BodyShape` admission plus the regime fences)
> — the row now says the latter, and the ledger's construction is named as
> owed. See §5 step 5's amendment.

---

## 4. The stage oracle is the go/no-go, and it should come first

SHIPPING_ROADMAP M2, adopted here as **step 0** of the migration: instrument
the real c2 under wibo at stable subsystem boundaries (post-IL-load,
post-COLOR, post-schedule, pre-emit — the addresses are already in
`P_DAG.md`/`P_REGALLOC.md`/`C2_MAP.md`) and serialize canonical snapshots for
representative TUs of each family.

Why first: it changes the economics of every later step. Black-box recovery of
the allocator priced at 52,416 refuted rivals *for one tie-band*; with stage
snapshots, ~~a ported COLOR is graded against COLOR's own output~~, and a
divergence localizes to a pass instead of costing a whole-object byte
archaeology session. It also converts the whitebox record from reference
material into the port's active specification. **Stop condition** (peer's,
kept): if stable stage observations cannot be obtained, the pipeline port
re-prices as black-box research before any restructuring is funded — and
steps 1–4 below are still individually worth doing, priced on their own.

The final judge is untouched. Snapshots are development instruments; nothing
is admitted on snapshot equality alone.

### 4.1 AMENDED 2026-08-21 — the premise fails in ONE direction, and it is not the direction the review first named

> **This subsection was rewritten within hours of first being written, and the
> first version is wrong.** Drafted from `docs/ARCH_REVIEW_2026-08-21.md`'s
> finding 1, it said *"COLOR's output and `sched0`'s output are NOT gradeable"*
> and gave `#3323`'s `IDENTICAL 7/7` as the evidence. **Lane `w-restim`
> (merge `b6fd2bf48`) overturned that by measurement**, and the review has
> itself been amended (its header block). The superseded version is in this
> branch's history at `00d46233a`; it is not reproduced here, because unlike
> the struck sentence above it was never anybody's plan of record. **The
> review was dispatched to be tested and one of its load-bearing findings did
> not survive contact with a measurement — that is the mechanism working.**

The struck sentence above was the load-bearing premise of §5 step 5. It fails,
but **in one direction only**, and the split is the whole re-pricing
(`docs/STEP5_PRICING_2026-08-21.md`; board **#3351**, **#3352**, **#3354**).

**c2's side: every stage is observable, COLOR included.** Measured over 384
fixtures / **2,946 function-pairs per bracket**, each bracket being the same
traversal read at two adjacent phases:

| stage | bracket | spine moves | operand-only | total | gradeable vs real c2 |
|---|---|---:|---:|---:|---|
| scheduler run 1 | `sched1`→`globregs` | 453 (15.4 %) | 35 (1.2 %) | **488 (16.6 %)** | YES |
| **globregs** | `globregs`→`sched2` | 2,766 (93.9 %) | 36 (1.2 %) | **2,802 (95.1 %)** | YES |
| scheduler run 2 | `sched2`→`color` | 171 (5.8 %) | 4 (0.1 %) | **175 (5.9 %)** | YES |
| **COLOR** | `color`→`sched3` | 130 (4.4 %) | **771 (26.2 %)** | **901 (30.6 %)** | **YES — and 85.6 % of its visible footprint is operand-only, invisible to every prior measurement including the review's** |
| run 3 **+ the lowering band** | `sched3`→`sched0` | 2,946 (100 %) | 0 | **2,946 (100 %)** | YES but **NOT SEPARABLE** — no tap site exists between them |
| **final schedule (run 4)** | `sched0`→`after0` | 118 (4.0 %) | 31 (1.1 %) | **149 (5.1 %)** | **YES — newly observable, site `0x10b7e701`** |

**What COLOR's write IS** (#3351): the *operand's symbol pointer* is re-pointed
from a **candidate** record to a **physical-register** record. Verbatim across
COLOR on `add3.cpp` fn1, `OP S 0 01 1004 **02 00000002 none**` →
`OP S 0 01 1004 **01 00000004**` — candidate id 2 with a null descriptor
becomes physical register `4` = `r3` under the `n = r+1` encoding at
`0x10b181c0`. It is not written into the tuple, and it is not written into the
candidate the operand already points at; **the allocator re-points the
operand**, which is why a walk that stopped at the tuple record saw nothing.
It also corrects a reading on the way: `P_REGALLOC.md` §4.1's `+0x1c` is the
candidate **id**, not a register.

**`#3323` is unmoved and reproduces exactly — it simply never generalised**
(#3356). Its fixture, `il_call_perm.cpp`, is one where COLOR is genuinely a
no-op on all seven functions, at every fidelity: spine, operands, symbols,
candidate ids, both register fields. That lane's own highest-confidence
prediction (0.85, that the operand-level pair would DIFFER there) was
**refuted as registered and hit on the population its prereg did not name** —
a population of one fixture, and the one fixture where the pass has nothing to
do. Across 384 fixtures the same comparison differs on 771 of 2,946.

**The port's side: NO, and not by a margin** (#3354, probe C). This is now
**the** binding constraint, and it is harder than the one it replaced. On
`w5_chain.cpp` — asserted byte-exact on **4 of 4** functions *first*, because
against a refused function the comparison is vacuous in the most ordinary way —
c2 carries **19–22 tuples and 29 regions where the port emits 4–5
instructions** (4.4–4.75× more tuples than emitted instructions). c2 expresses
a region boundary as a **tuple index at a pre-lowering phase**; the port's most
granular structure is `block_ir::BasicBlock`, whose `body()` is `&[u8]`. A
boundary given as a tuple index has **no image** in that coordinate system.
**The port→tuple projection is UNDEFINED, not merely unequal.**

> **So the corrected position, and it is the sentence step 5 must be planned
> against: per-stage observability buys CHARACTERIZATION, not a per-stage
> DIFFERENTIAL GRADE.** Reading what c2 does at each stage is now cheap and
> mechanised. Grading a *ported pass* against that reading still requires
> reproducing c2's region decomposition and tuple coordinate system, which is
> row `4b`'s work and does not exist today. Any §5 row that assumes a
> per-stage grade for a port is mispriced.

**The instrument defect this subsection first reported is FIXED** (#3349): the
raw-window verdict skipped length-mismatched functions and then tested only
`hot.is_empty()`, so it printed *"the allocator wrote nothing in this window"*
over **zero comparisons on 92 of 384 fixtures**. The vacuous and the
substantive case now have different sentences and the skipped count prints
either way. A second instrument correction landed beside it (#3350):
`stage-snap-tuples` is a **payload size, not a coverage** — 303,600 published
rows are 117,174 distinct tuple positions, an inflation of **2.59×**, because
the walk terminates on the end of the function's list rather than the end of
the region it was handed.

### 4.2 AMENDED 2026-08-21 — the stop condition is binary and reality is a PARTIAL FIRE

The stop condition above admits two states — *stable stage observations
obtained*, or *re-price as black-box research*. **Neither is what happened**,
and the third state survived even after `w-restim` moved which boundaries are
observable. The mechanism is green (determinism, canonicality, a null control,
and a required-zero neutrality grade against the judge); c2's side is
observable at every stage; **and the thing the observation was supposed to
grade cannot be put in the same coordinates as it.** The proposal has no
language for that, and supplying it is the single highest-value amendment the
review found (arch review §6).

The rule this section now carries, in its corrected form:

> **Partial fire is a first-class outcome of a characterization campaign, and
> it is priced per boundary — AND OBSERVABILITY IS NOT GRADEABILITY.** A
> mechanism that observes *n* of *m* boundaries funds characterization on the
> *n* and re-prices the *m − n*; it does not fund all *m*, and it does not stop
> the campaign. **Separately and additionally: an observation is only a GRADE
> if both sides can be expressed in its coordinates.** At tip, c2's side is
> observable at 6 of 6 brackets (one of which, run 3 + lowering, is not
> *separable*), and the port's side is expressible at **none** of them. A lane
> that reports "stage parity" for a port is reporting an instrument, not the
> pass — and the standing bound (`rungs/2026-08-20-stageoracle.md` §8: no
> `crates/` rule enters on snapshot equality) is what keeps that mistake away
> from the judge.

### 4.3 AMENDED 2026-08-21 — the correct positive case for step 0, which is stronger than the one above

The argument §4 made for step 0 was *"a ported COLOR is graded against COLOR's
own output"*. Half of that is now measured true and half measured false: COLOR
*is* readable against its own output; a *ported* COLOR still cannot be graded
against it. **The case that survives is better than either, and was available
before any of these lanes ran** (arch review, "what survives"):

**The snapshot captures the tuple order *entering* COLOR** — which is item
**F0**'s input half, and F0 is **8 of item F's 17 lanes**, the largest single
line in `CEILING.md` §6.1's decomposition (F1 2 · F2 1 · F4 2 · F5 2 · F6 1 ·
F7 1 = 9 for everything else combined). It is also **exactly the black-box
blind spot**, in CEILING's own words: downstream of allocation sit four
order-changing stages, so *"the order that decided the registers does not
appear in the obj"* — which is why `codegen::alloc`'s 52,416-configuration
residual and `codegen::schedule`'s 13,104-configuration residual are both fits
against bytes that no longer carry the input the rule was keyed on.

**And that order is now directly readable at six phases plus after run 4, with
the register assignment readable beside it.** F0's cost changes in *kind* —
from a black-box search over obj-visible consequences to a differential read
against a live trace — and `STEP5_PRICING` §3 re-prices it **8 → 4 lanes raw**,
taking item F from **17 → 13** (×5 calibrated: **65**). **It does not go to
zero, and two things are the reason it does not**: reading a trace is not
deriving a rule (`P_DAG.md` §3's priority formula and §5's machine model are
still `[R]`), and **probe C's residue is F0's residue** — a rule read off c2's
trace still has to be implemented in a port that has no tuple, no category and
no region, so the implementation lands in `4a`/`4b`'s coordinate system, not in
this one.

> **↳ 2026-08-22 — 13 raw / 65 calibrated is the BLACK-BOX price and may not be
> quoted as the cost of the facts** (`ROADMAP_SLICING_2026-08-21.md` §6 rule 6;
> `CEILING.md` §6.1 and `STEP5_PRICING_2026-08-21.md` §3 carry the matching
> annotations). Two of item F's rows have a priced read:
> **R7** (`[R]` → `[O]` on the scheduler, no new reading, 3–5 d) and **R4**
> (`FUN_10b55732`, item F1, 3–5 d) — `whitebox/READ_PLAN_2026-08-21.md` §3.
> **This paragraph is already reasoning the right way** — it is the one that
> reclassifies F0 from black-box search to differential read — so the pointer
> is a *cross-reference*, not a correction; both its reasons why the cost does
> not go to zero survive intact, and the second one (`4a`/`4b`'s coordinate
> system) is exactly why the reads do not re-price `4a`. Propagated by lane
> `w-readdocs`.

Step 0 bought the input half of the most expensive item on the ceiling page,
and `w-restim` bought the output half of it too. That is the claim to make for
this section. It is not the claim §4 made.

---

## 5. Migration path — the 26 stay green at every step

Every step is a construct rung or characterization lane (the units CLAUDE.md
already defines), landed under the full gate, with the identity criterion
ARCHITECTURE_SEAMS §0 used four times: census numerator, per-key histogram,
scan JSONL rows, disagreement counters, `match 26 / mismatch 0`, fnbyte
non-decreasing — byte-identical or explicitly accounted, per step.

> **AMENDED 2026-08-21 — three structural changes to this table** (arch review
> §3, §4, §6): (a) every row gains **one axis on which the step can FAIL even
> when its bytes are identical**, because #3336 measured that a required-zero
> *byte* delta is silent about a required-zero *cost* delta and this migration
> is six construct rungs deep; (b) an explicit **integration row `4a`** carries
> the two prerequisites step 5 needs and that appeared in no row and no budget;
> (c) **IR3 gets its own step, `4b`** — it was the only IR in §3.1 with no
> migration step at all. Steps are inserted as `4a`/`4b` rather than
> renumbered, so every existing citation of "step 5" and "step 6" elsewhere in
> the repo still resolves.

| # | step | kind | graded by | one axis on which it can FAIL (#3336) | what it unblocks |
|---|---|---|---|---|---|
| 0 | **Stage oracle** (§4) | characterization lanes | snapshot determinism + one end-to-end instrumented TU per family | **neutrality**: armed-vs-disarmed obj bytes. A tap that perturbs c2's own output makes every snapshot a measurement of the tap | **LANDED — lane `w-stageoracle`, merge `427c44255`, 2026-08-20, outcome `built`; verdict GO, with one boundary named.** The mechanism is green: G1 neutrality **required-zero** against the judge, G2/G2b determinism + canonicality, G3 the null control, G5 content cross-derived three ways. Board **#3322**–**#3326**. **Deviations from this row as written, all deliberate and all published:** (i) **residency was NOT built** and the rung says why — one process per compile is what makes snapshot determinism testable (§7.1 is reversed below); (ii) P11's 57 phase-beacon sites are **not armed**, and the record correction stands anyway — `0x10bec297` is the abort poll, not *"the timer"* `P_DAG.md` §2 calls it; (iii) **6 of the 26 matched TUs produce a structurally EMPTY snapshot** (`hits 0 · regions 0 · tuples 0`) because they emit no function bodies — reported, not averaged away; (iv) the "2 functions × 7 sites" arithmetic in the hand-off was wrong twice over and is corrected in the rung. **And the headline this row promised did not survive intact — see §4.1, and note the direction, because it is not the one the review first named.** `w-restim` (merge `b6fd2bf48`) then measured that **c2's side is observable at every bracket, COLOR included** (#3351: 771 of 2,946 pairs move at the operand's symbol pointer) and added an eighth site for run 4 (#3352). What *is* refuted is *"everything in step 5"*: a **port** cannot be graded against any of it, because the port→tuple projection is undefined (#3354 — 19–22 tuples and 29 regions against 4–5 emitted instructions on a 4/4 byte-exact fixture). So this row unblocked **characterization at every stage**, plus the tuple order *entering* COLOR — item F0, 8 of item F's 17 lanes and now re-priced to 4 (§4.3) — and it did **not** unblock a per-stage differential grade for a ported pass. That is row `4b`'s work |
| 1 | **IR0 under the current reader**: build the lossless framer; ~~re-express `IlBundle::functions()` and the census splitters as *views* over it~~ | construct | identity protocol + new totality/opaque denominators printed both sides of the change (trap-0/#961 discipline) | **throughput** — and this is the row where it FIRED: the switch was byte-identical by construction and cost port time, so byte identity could not have caught it (#3335, #3336) | **DONE IN PART — lane `ir0`, merge `c19d07357`, 2026-08-20, outcome `built`.** The framer, its totality controls and the opaque denominators shipped at a required-zero byte delta (14 `ir0-*` keys; `.ex` **99.61 %** framed on the workload, **31.35 %** on the fixtures). The K1 codec's round-trip — the step's stated spine — ran on dc3 for the first time: **870 of 870**, 0 broken (**#3332**). **The re-expression half was BUILT, MEASURED AND REVERTED**: all eleven production call sites switched, byte-identical by construction, at a cost of **+2.03 % port time per obj** (95 % CI [+1.72, +2.34]; **#3335**). ~~8–14 % of the port's geomean; the price is the `Vec<Record>` the gate view never reads~~ — **both figures WITHDRAWN by the lane's fix round.** The first reading was the measurement's own null (the same protocol reads −13.7 % on the *reference* side, where the effect is zero by construction), and the records-suppressed probe measures **slower**, not faster, so the cause is **unattributed**. **The scheduling answer is unchanged and did not depend on the wrong number: do it inside step 4**, where a record carries a binding, not here — 2 % of the port's throughput for a layer with no reader yet is still the wrong trade at step 1. And the "kills 97% no IR" claim was **false as written** (#3332's rung, DD2). **AMENDED 2026-08-21: THE DEFERRAL EXPIRED WITHOUT BEING HONOURED.** Step 4 has landed (merge `d7be7aadc`) and `git diff c7049bac1..d7be7aadc -- crates/` touches exactly `bind.rs`, `census.rs`, `diag.rs` — **no `stream/` file, no call site switched**. So *"do it inside step 4"* named a step that then did not do it, and the arch review's §2 finding applies at full force: the production switch survives in **no** commit (the "reverted" commit `09d42b15f` is 47 doc-comment lines), so redoing it starts from zero across eleven call sites. Whoever picks it up owns the re-derivation, not a revert |
| 2 | **The W8 sum type**: `IlFunction.body: BodyShape`; delete the parallel `Option`s and `select_function`'s re-derivation | construct (the long-deferred SEAMS step 5; needs its quiesce window) | identity protocol; census/gate disagreement stays 0 | **precedence**: the old `is_some` chain's *order* was load-bearing in exactly one place, so a variant promoted to a peer arm that should have stayed ordered moves no byte on the corpus that ran — it moves them on the one that did not | **LANDED — lane `w8sum`, merge `c7049bac1`, 2026-08-21, outcome `built`; board #3345; rung `docs/rungs/2026-08-21-w8sum.md`.** The **34** `Option<Shape>` fields (that count is the rung's, `:1,24` — **not** board #844's, see §1.2 item 3) collapse to one `pub body: BodyShape`, **35 variants**; `select_function` is a **total, exhaustive 35-arm `match`**; `cfg_class::lowering_of`, the mirror precedence helper, converted the same way. Required-zero held: 18-lane gate table **diff = 0 lines** over all 23 count-bearing rows, `mismatch 0`, `match 26 / vocab-gap 844` unmoved, partest **1,761 / 0 / 1** with **0** `SKIP: toolchain absent`, net **−438 lines / 35 files**. **Deviation worth carrying:** one precedence survives and is transcribed verbatim — `Plain`'s four-matcher leaf order (`indirect_load_text` → `addr_leaf_text` → `store_leaf_text` → `select_text`), because those four read the same `func.ops` stream; the order *over the 34 fields* was incidental (they were mutually exclusive at the parser) and is now a type. **The `is_some` half of this row is done; the "pair of files" half is not** — that is step 6 |
| 3 | **IR2 ObjPlan + the manifest instrument**: consolidate `plan/`; add the structural-manifest grade over all 870 TUs as a gap-metric family (`plan-exact`, per-component) | construct + instrument | ~~manifest identity on the 26 matched TUs (required-exact); the other 844 become a *measured curve*~~ — **VACUOUS as written** (#3330): with the port side computed as `observe(port_obj)`, `port_bytes == ref_bytes` on the 26, so `manifest(port) == manifest(ref)` for **any** pure `manifest`; and on the other 844 the port emits no obj, so the port side is **undefined**. Repaired by two independent producers plus a **named** control (`docs/plan/CONTROL_TUS.txt`) | **the control's own size**: a `plan-*` component whose extractor collapses to `Some(empty)` reads *more* exact, not less — a 20-TU mutation reads `exact` unchanged and control size **0**. `plan-control-obs-size`/`-obs-empty-tus`/`-substantive-tus` now publish it | **LANDED — lane `w-objplan`, merge `14d26f44e`, 2026-08-20, outcome `built`; board #3327–#3331.** ~~un-conjuncts A∧B∧C: emit-set closure, weak externals (675), COMDAT synthesis (450), the 3-name section ladder each become independently gradeable lanes with an honest denominator~~ — **RE-SCOPED 2026-08-21 against the lane's own §5 "found and NOT taken"**: (1) **the SECTION component was not built and the reason is not time** — its only walk-free substrate is the whole-TU recognizers on the *admission* side of the reader, i.e. the exact route `plan/`'s fence exists to keep out; shipping it would have printed `known ≈ 30 of 870` and re-published `vocab-gap 844` under new keys; (2) **weak externals were not made a component** because the port models none, so it would grade **`Unknown` on 870 of 870** and say nothing the `Unknown` histogram does not; (4) the emit-set closure lane is **re-priced**, and its first deliverable changed — `gl_function_attrs`' silent **skip path**, not the closure. **Both components that DID ship, ship `Unknown`** (#3329/#3330). **What this step actually delivered is ground truth, not curves**: 870/870 observed inventories (weak externals 674 TUs / 4,009 records; associative COMDATs 101,120 sections over 846 TUs; `SELECT_LARGEST` on 5,214 sections, which nothing in the repo had counted; `sel-unknown` **0**) and **four lanes now denominated** — emit-set closure, weak externals, COMDAT synthesis, the section ladder (853/854 distinct name/attr sequences, owing only a walk-free predictor). **And the predictor denominator ceiling is 854, not 870**: `gl_function_attrs` answers on 854 TUs, so 16 are outside any `.gl`-keyed curve before it starts. **THE CORRECT READING OF THIS CELL, replacing "un-conjuncts A∧B∧C": step 3 makes each conjunct INDEPENDENTLY MEASURABLE; conversion remains CONJUNCTIVE.** Measuring the conjuncts apart is a real gain and it is not the same thing as dissolving the conjunction — every TU still needs all of them, which is §1.2 item 4's whole point. **And the three denominators this cell quoted as lane sizes are stale, two of them badly:** `845` is `844` (`vocab-gap` at this tree); weak externals `675` is **674** (#3331); and **COMDAT synthesis `450` is the `emit-set-ceiling-wall` key, which has read 451, 450, 400 and 399 at four different workload stamps** — `docs/ROADMAP.md:8540` **451 of 871**, `docs/STATUS.md`'s hand-written §306 **451**, its *generated* block **400** (beside `470 repaired`, and `470 + 400 == 870`), and the conjunction lens's re-measurement at `d7be7aadc` **399** (`471 + 399 == 870`). **The spread is not error, it is the #3306/#3311 effect** — the key moves with the sibling `dc3-decomp` stamp — which is exactly why a figure like this must not be quoted as a lane size at all. **`CEILING.md` §2.3 also warns that the number's SIGN is routinely inverted**: the wall is *"the number of TUs a segment-driven model can never reach"*, so *"quoting 'the 450 wall' as a figure the project might attain inverts its sign"* — the reachable complement is `repaired`, not the wall. This cell had it the wrong way round |
| 4 | **IR1 consolidation**: one binding, `bind.rs`'s pinned-apart seam closed on purpose (the numerator move recorded, not absorbed) | construct | binding counters + JSONL identity; the numerator move published as its own row | **a name the compiler cannot grade**: the byte judge is silent on `mangled_name`, so a binding change is wrong-but-green by construction and only a gap metric can see it (`docs/GAPS.md` §8) | **LANDED — lane `ir1`, merge `d7be7aadc`, 2026-08-21, outcome `built`; board #3347–#3348.** `Bindings::positional` (a narrow `mangled_names` scan, `paired` on **count alone**, body offsets ignored, blind to `??`-names and to data-vs-function) is **deleted**; the census and the `diag.rs` locals probe now use `Bindings::census` — the gate's exact, `??`-aware, offset-checked 1:1 pairing, made **total** (empty names where it does not bind). Required-zero obj byte delta; the emit path was already on `per_record`/`selective` and is untouched. **The numerator move is published, not absorbed**: `fnbyte-name-disagree` **74,033 → 0**, and a full `diff` of the 484-line GAP-METRICS block base-vs-tip returns **exactly that one changed line**. Not vacuous: of the emitted rows, 44 now carry a per-record name and **all 44 agree**. **Deviation:** step 1's deferred re-expression (*"do it inside step 4, where a record carries a binding"*) was **not** done here — see the step-1 row |
| **4a** | **INTEGRATION — the two prerequisites step 5 has and no row budgets** (added 2026-08-21, arch review §3): (i) a **general op-level IL decode** — IR0 stops at a two-variant byte framing and `BodyShape` starts at 35 whole-function grammars that are *simultaneously the admission gate*, so the semantic middle a COLOR pass would consume does not exist; (ii) a **general lowering to `coff::Function`** — today a 35-arm per-shape dispatch, `crates/c2-core/src/comdat.rs` carrying **43** `Selected::` references | construct pairs, quiesce-window | required-zero byte delta on the shipped classes re-expressed through the general path (the #290 protocol), **plus** an explicit statement of which `coff::Function` field each new general lowering writes | **throughput, and stage-parity-with-no-consumer**: without this row a step-5 lane's only progress signal is snapshot parity — an instrument with **no emit-path consumer**, i.e. **#3336 at program scale**, and unlike #3336 there is no contrast case to catch it | **PRICED TWO-SIDED, AND IT IS THE CRITICAL PATH** (`STEP5_PRICING_2026-08-21.md` §2/§4; board **#3355**). *Cost of building it:* raw **3–9 engineer-months** (I1 1.5–4.5 + I2 1.5–4.5), and CEILING §5's ~5:1 is **applied, not cited** — **15–45 engineer-months as a LOWER BOUND**. *Cost of NOT building it:* step 5 cannot reach the byte judge at all, so every step-5 lane lands an **unconsumable instrument**; the project has a measured precedent for exactly that at one-lane scale (**#3336**, `ir0` — a required-zero byte delta that held *by construction* because the tree had no production caller), and at program scale that shape has **no contrast case to catch it**. **That is the more expensive side.** Probe C sharpens I1 specifically: c2's middle is a tuple list with per-tuple categories and two operand lists, the port has neither, and the gap is **4.4–4.75× more tuples than emitted instructions** on four byte-exact functions (#3354). **Cheap prophylactic, adopted now and independent of the row landing:** every step-5 lane names in its rung header which `coff::Function` field its pass would eventually write — a smoke alarm, not a substitute |
| **4b** | **IR3 gets its own step** (added 2026-08-21, arch review §4 — it was the only IR in §3.1 with no migration step): define IR3 **in c2's coordinates**, not in the port's. The proposal contains **zero** occurrences of "tuple" and **zero** of "region" — the words that name c2's actual IR (`WB_REGALLOC_FINDINGS.md`: a tuple *list*, not a tree) and the scheduler's unit of work. IR3 must **carry** tuple identity — **opcode number** (c2's own numbering: `li` = 624 vs `addi` = 11), **category**, **monotonic index** — and the **region partition** as first-class fields, following this repo's own `Terminator::Bc`-carries-`(BO,BI)` precedent | construct | **the port→snapshot projection as a graded artifact, with its own required-zero criterion** — this is the deliverable, not a side effect | **non-functionality of the map**: the port's encoded-byte blocks have already erased c2's opcode numbering, so port→tuple is not a function until IR3 carries the number; and **regions cut through shipped block boundaries** (a call *ends* a region in c2 and sits mid-block in the port), so a projection can be green per block and wrong per region | **MEASURED EVIDENCE FOR THIS ROW LANDED 2026-08-21 — probe C, board #3354.** This was the review's inference; it is now a number. On `w5_chain.cpp`, byte-exact on **4 of 4** functions (asserted *first*, so the comparison is not vacuous against a refusal), c2 carries **19–22 tuples and 29 regions** where the port emits **4–5 instructions**. c2 expresses a region boundary as a **tuple index at a pre-lowering phase**; the port's most granular structure is `block_ir::BasicBlock`, whose `body()` is `&[u8]`, so that boundary has **no image** in the port's coordinates — the projection is **undefined, not unequal**. **This row is therefore a hard prerequisite for any per-stage grade of a port, not a tidiness item.** Without it the grade is ill-defined by four independent routes (arch review §4 a–d), including that **COLOR's real state lives in a third representation** — 0x48-byte candidate records, whose `+0x1c` is the candidate **id** and not a register, the assignment being one hop further out (#3351) — that IR3 as §3.1 defines it has no concept for. ~~"cross-block liveness (item F)"~~ is the name §3.1 uses and **this repo has refuted that name**; item F's price is **17 lanes** (`CFG_SHAPE.md`, `CEILING.md` §6.1), which §3.1 never quotes, **re-priced to 13 raw / 65 calibrated** now that F0 is readable (`STEP5_PRICING` §3) |
| 5 | **Port the middle**: COLOR, the DAG scheduler, the passes the pinned workload exercises, item F liveness — from the disassembly, per-stage against step 0's snapshots, ~~admitted regime-by-regime behind claim 4~~ **admitted regime-by-regime behind today's refusal predicates** (`BodyShape` admission + the regime fences: `MAX_MODELLED_PRODUCERS`, the signedness fences, `LabelMap` invariant 4) — **reworded 2026-08-21 because §3.3's claims ledger is unbuilt and no step in this table builds it**; whoever wants the ledger owes it a step | characterization → construct pairs | ~~per-stage snapshot equality on the graded corpus~~, then the byte judge; existing `alloc`/`order`/`schedule` rules as regression fences. **AMENDED 2026-08-21: per-stage snapshot equality against a PORT is not available at ANY bracket** (§4.1, #3354). c2's side reads at all six; the port's side is expressible at none, so the grade is undefined rather than failing. What the snapshots grade is **c2 against itself** — determinism, canonicality, a rule read off a live trace — which is characterization | **grading a pass in a coordinate system the port does not have.** A "stage parity" number for a ported pass is an instrument reading, not a pass reading, and the standing bound (stageoracle §8) is what keeps it out of the judge | **NO-GO AS WRITTEN, 2026-08-21** (arch review verdict, and `w-restim` re-priced rather than rescued it). Gated on: (a) these amendments; (b) row **4a**, now priced at **15–45 engineer-months** as a *lower bound* (`STEP5_PRICING_2026-08-21.md` §2/§4 — I1 + I2, CEILING §5's ~5:1 applied per row); (c) row **4b**, without which no per-stage grade for a port can exist at all; and ~~(d) the **owner** re-owning the thesis-vs-870 goal question, unowned since `STRATEGY_REVIEW_2026-08-13.md:251` (*"The question is currently owned by nobody"*) and still true~~ **(d) — DISCHARGED 2026-08-21: the owner answered it (full reproduction, for understanding MSVC's internals and for parity — `GOAL_DECISION_2026-08-21.md`), which makes step 5's economics read off the Option A branch alone. (a), (b) and (c) still gate; the NO-GO stands on them.** **Depends on 4a AND 4b**, not on step 0. **The critical path is INTEGRATION, not any single pass** — registered as E2 at 0.70 and it holds; item F's 13 re-priced lanes (×5 = 65) are the *second* cost, and every characterization lane is cheap beside both. What it would unblock is unchanged and is still the prize: the call-bearing 0.000; the loop classes; the long pole |
| 6 | **Demote shapes**: as the general path reproduces a class byte-exactly, its dispatch arm flips from producer to cross-check, then to `#[cfg(test)]` witness | construct, one class at a time | identity diff of per-class gate counts (#290) — a demotion that moves any count is a finding, not a cleanup | **coverage weight**: a demoted class that the general path reproduces on the *fixtures* and not on the workload moves no gate count, because the gate corpus is the fixtures (#3333's shape: a fence whose triggering condition is rare in the corpus that runs it is not a fence) | the catalogue stops being production dispatch (peer M5's rule, adopted). **AMENDED 2026-08-21: depends on row `4a`, not on step 5.** A shape can only be demoted once a *general* lowering to `coff::Function` exists to demote it to; that is 4a's second half, and step 5 is neither necessary nor sufficient for it |

Ordering notes: steps 1–2 are independent of step 0 and can start immediately;
step 3's instrument half is independently valuable even if step 5 stalls;
~~step 5 is strictly behind step 0~~ **step 5 is behind step 0 AND behind row
`4a`, and 4a is the longer pole** (amended 2026-08-21). Steps 1, 2 and the
step-3 consolidation are
quiesce-window moves in the SEAMS sense (mechanical, conflict with everything
in flight, land in announced gaps). **Row `4b` (IR3) is independent of step 5
and should precede it**: without a projection defined in tuple/region
coordinates there is nothing for a per-stage grade to compare.

> **AMENDED 2026-08-21 — three of the four staged IRs have zero production
> callers** (arch review §2, by compiler-checked amputation and by an
> independent caller sweep, same answer). Deleting `c2-il/src/stream/` (IR0),
> `c2-core/src/plan/` (IR2) and `c2-core/src/passes/` from a scratch copy of
> master **builds clean and passes 1,230 unit tests**; a control amputation of
> `block_ir.rs` correctly fails with 14 errors. Only IR1 (`bind.rs`) is on the
> emit path, and step 4's own delta was instrument-scoped (census + diag),
> which its commit says. IR2 is not merely unwired but **fenced off** the emit
> path by its own BANNED-list test (`crates/c2-core/src/plan/mod.rs:483`), for
> a documented reason (#3237) — wiring it in means deleting an invariant.
> **None of this is concealed**: the source declares each layer's inertness,
> `passes/mod.rs` is a 30-line comment that says so, and `sym/`, `sem/`,
> `mir/`, `lower/`, `emit/` do not exist. This is honest scaffolding, not the
> wrong-but-green family — **the defect is in the plan's shape, not the work.**
> It is recorded here because a reader of §3.1's five-row table would otherwise
> conclude that four IRs exist and one of them (IR3) merely lacks a step.

**What survives:** the warranty stack whole; encode/frame/labels/block_ir/
cond/coff/elide/splice/comdat; c2-obj; c2-reference; the fixtures; the
whitebox record; the shape files (as witnesses). **What is re-expressed:**
bundle.rs → IR0/IR1 views; shape emitters → MIR lowerings; `select_function`
→ the claims ledger *(unbuilt, and no step in the table above builds it —
see §3.3's amendment)*. **What is eventually deleted:** ~~the parallel
`Option` fields~~ **(done — step 2, `c7049bac1`)**; `try_parse_*` as admission
gates; ~~the 38-arm match~~ **the 35-arm `match`** as a production
accept path — each only after its replacement is identity-graded, never
before.

---

## 6. Honest cost, and what it buys

> **AMENDED IN FULL, 2026-08-21** (arch review §5, §6). This section cited
> CEILING §5's calibration instead of **applying** it, priced its steps against
> a reference class of survivors, gave no step a criterion anyone could score,
> and asserted three properties of its four curves that are each false at
> master. Rewritten below; the original bullets are struck, not deleted.

* ~~Steps 0–4 are **weeks each**, not months — they are the same size class as
  moves this repo has already landed (the codegen split, block_ir, the lane
  registry)~~ — **the reference class is survivorship** and is withdrawn: "moves
  this repo has already landed" selects on landing, so it cannot price a move
  that might not. (Steps 0–4 did in fact land in about a week; that is an
  outcome, not the estimate that produced it, and it is one draw.) What replaces
  it is a **scorable done-criterion per step**, so "is this step finished" stops
  being a judgement call:

  | step | DONE when — one scorable criterion |
  |---|---|
  | 0 stage oracle | a named boundary set, each boundary either **observed** (a pre/post pair that DIFFERS over a *population*, not one fixture) or **published as unobserved**; neutrality required-zero against the judge. *Met — and note the population clause is not decoration: the same criterion read against one fixture put COLOR in the unobserved column for a day (#3323/#3356), and against 384 fixtures it reads 771 of 2,946 (#3351).* |
  | 1 IR0 | round-trip **870/870, 0 broken** on the workload, with the opaque denominator printed both sides. *Met (#3332). The re-expression half is NOT met and is not scheduled — see the step-1 row.* |
  | 2 W8 sum type | `select_function` is a **total, exhaustive `match`** with no `is_some` chain, at a gate-table diff of **0 lines**. *Met (#3345).* |
  | 3 IR2 | every plan component ships with a **named control** and either a published agreement rate or an explicit `Unknown`; ground truth graded on the full answerable denominator. *Met — and both components ship `Unknown` (#3329/#3330), which is the criterion working, not failing.* |
  | 4 IR1 | one binding; the numerator move **published as its own row**, not absorbed into a green scan. *Met (#3348: 74,033 → 0, one changed line in a 484-line block).* |
  | 4a integration | one already-byte-exact class reproduced end-to-end through the **general** decode and the **general** lowering, at a required-zero byte delta, with the `coff::Function` fields it writes named. **Not started.** |
  | 4b IR3 | a port→snapshot projection that is a **function** (opcode number carried, region partition carried) and agrees with the tap on one byte-exact fixture at a required-zero delta. **Not started.** |
  | 5 the middle | a ported pass whose output reaches **the byte judge** on a regime, with the pre-existing measured rule kept as a fence. **Not started, and NO-GO as written.** |
  | 6 demote | a class whose dispatch arm is `#[cfg(test)]` and whose gate counts are **identical** to the arm's producing state. **Not started; behind 4a.** |

* Step 5 is the program: the peer's **12–24 engineer-months to native dc3
  parity, high uncertainty, re-estimate after the stage oracle and after the
  plan manifest** is ~~the right order of magnitude and~~ the right hedging.
  ~~The calibration record (optimism ~5:1 on forward costs, CEILING §5)
  applies.~~ **APPLIED, not cited (amended 2026-08-21):** CEILING §5's
  instruction is *"read every forward cost figure on this page as a LOWER
  BOUND"* — the tally is ten-or-eleven optimistic misses against one-or-two
  pessimistic, and **the misses are specifically on forward cost**, not on
  measurement. So **12–24 engineer-months is a lower bound**, and calling it
  "the right order of magnitude" is exactly the move the calibration forbids:
  it converts a floor into a centre.

  **RE-PRICED PER STAGE, 2026-08-21 — `docs/STEP5_PRICING_2026-08-21.md`**, the
  re-estimation this very bullet asked for (*"re-estimate after the stage
  oracle and after the plan manifest"*), delivered by lane `w-restim` with
  CEILING §5's ~5:1 **applied per row** rather than cited. Its verdict on the
  12–24 figure is sharper than "too low": **the figure is not obviously wrong
  in magnitude, it is wrong in COMPOSITION.** It was written for the passes,
  and the passes are not what dominates.

  | row | raw | ×5 lower bound | on the critical path of a byte-judged output? |
  |---|---:|---:|---|
  | **I1** general op-level IL decode | 1.5–4.5 eng-mo | **7.5–22.5 eng-mo** | **YES** |
  | **I2** general lowering to `coff::Function` | 1.5–4.5 eng-mo | **7.5–22.5 eng-mo** | **YES** |
  | characterization, all stages | 5–6 lanes | **25–30 lanes** | no — it is the *input* to the construct rows |
  | item F construct, re-priced (F0 8→4) | 13 lanes | **65 lanes** | YES |
  | the lowering-band tap site | 1 lane | **5 lanes** | no |

  **The critical path is INTEGRATION, not any single pass** (registered E2 at
  0.70, holds), and **the 12–24 figure does not cover I1 and I2 at all** —
  those two rows alone are **15–45 engineer-months at the lower bound**. The
  pricing lane declines to convert lanes into engineer-months rather than
  invent a rate, so its E3 (*"the calibrated total exceeds 12–24 months"*,
  registered 0.55) is recorded as **not cleanly resolved**; that refusal is
  more useful than a fabricated conversion and is repeated here rather than
  smoothed over. The honest statement is: **≥ 12–24 engineer-months for the
  middle, plus ≥ 15–45 for the integration it presupposes, plus 65 calibrated
  lanes for item F — and the project's own record says every one of those is
  the low end of its distribution.**
* What the re-architecture buys that the current shape route cannot:
  the current route prices at 3,400–10,400 lanes with a frontier of 2 and a
  conjunction that makes every subsystem's completion convert ~0. The staged
  design replaces "convert the next TU" with four ~~monotone~~ curves (decode
  totality, binding accountability, manifest equality, per-stage pass parity)
  ~~each of which is gradeable today and none of which can silently regress
  under the existing gate~~.

  **ALL THREE PREDICATES ARE FALSE AT MASTER AND ARE DELETED, 2026-08-21**
  (arch review §5). Taken one at a time, because each fails by a different
  route:

  1. ~~"each of which is gradeable today"~~ — **two of the four are not.**
     *Per-stage pass parity* has no grade for COLOR or `sched0` (§4.1).
     *Manifest equality* ships **both** of its components as `Unknown`
     (#3329/#3330), by the lane's own registered rule, and the section
     component was not built at all because it has no walk-free substrate
     (`w-objplan` §5.1).
  2. ~~"none of which can silently regress under the existing gate"~~ —
     **the gate cannot see them.** `scripts/gate.sh` is 5,854 lines and
     contains **zero** occurrences of `gap-metric` or `plan-` (verified at
     this tip). A key the gate never reads cannot be protected by it, and
     `c2rs perf` is likewise **reported, never gated** (#3336).
  3. ~~"four monotone curves"~~ — **`gap-metric` keys move with the live
     corpus, not only with `crates/`.** #3306: **82 of 394 keys** moved
     between the two ends of one campaign on the same commit pair, same
     binary, same machine — the sibling `dc3-decomp` stamp had advanced.
     #3311 is the third instance, on a lane whose `git diff -- crates/` is
     **empty**, and it moved `fnbyte-exact` by 70 and the denominator by 334.
     `w-objplan` §5.5 measured **43 of 439** keys moving between two runs of
     one binary. A quantity that moves under someone else's merge is not
     monotone; it is not even a function of this repo.

  The replacement sentence, and it is the one to approve or reject: **four
  curves, of which ZERO currently carry a measurement** (#3327 — *"neither
  predictor ships as a curve, because the lane's own registered ship rule
  kills both"*). What the design *does* buy, stated without the three
  predicates: **four denominated lanes where there were none** (emit-set
  closure, weak externals 674 TUs, COMDAT synthesis 846 TUs, the section
  ladder 853/854), each with ground truth graded on the reference side —
  which is a real and new thing, and is not a curve.

  **So step 5 must NOT be funded on "un-conjuncts" or "four monotone
  curves".** Those are the two sentences a reader approves off, and they are
  the weakest lines in this document. The arguments that survive review and
  should carry the decision are **§2's eventual-totality argument** (no
  intermediate match count pays, so optimize for totality rather than for the
  next conversion) and **step 0's divergence-localization economics** (at the
  boundaries §4.1 says are observable, a divergence costs a pass instead of a
  whole-object byte archaeology session).

* ~~step 3 gives the first honest continuous curve toward 870 (structural
  manifests, byte-weighted)~~ — **FALSE AT MASTER, corrected 2026-08-21 to
  what step 3 delivered.** #3331 and #3327–#3330: step 3 delivered **ground
  truth on 870 of 870 TUs** and **four denominated lanes**, and **zero
  curves** — both predictors were withdrawn to `Unknown` by the lane's own
  prereg, and correctly so. The only curve component ever measured, emit-set
  closure from the `.gl` `0x20` seed, disagreed with real c2 on **821 of 854
  TUs (96.1 %)**. **That the design caught this early and cheaply is a genuine
  point in its favour** — the withdrawal cost one fix round and no wrong emit
  — but it is not a continuous curve, and the honest ceiling for any
  `.gl`-keyed predictor is **854**, not the promised 870.
* **If the stage oracle fails** (M2's stop condition), the correct fallback is
  not the status quo: steps 1–3 still stand on their own, and step 5 re-prices
  as black-box characterization at the measured rates — which is the point at
  which the vendor-backed service stops being optional product and becomes
  the only shippable coverage story. **AMENDED 2026-08-21: it did not fail and
  it did not succeed — it PARTIALLY FIRED, and the fire line is not between
  boundaries, it is between c2's side and the port's** (§4.2). Observation of
  c2 succeeded at every bracket, COLOR included, and is now **cheap and
  mechanised**; what did not arrive is a per-stage grade for a *port*, because
  the two have no common coordinate (#3354). So this bullet's binary resolves
  as: **characterization is funded at the newly-measured rate** — F0 re-prices
  8 → 4 lanes, item F 17 → 13 — **and the differential grade is not
  black-box-priced, it is BLOCKED until row `4b` exists.** Black-box is the
  fallback for a rule you cannot see; here the rule is visible and the
  *implementation target* is what is missing, which is a different and more
  tractable problem than the 52,416 refuted allocator rivals this project paid
  for once already.

## 7. The vendor-backed service (peer proposal §5) — orthogonal, with one shared component

The compatibility service is **product architecture, not compiler
architecture**: it changes nothing in §3, and under Option A its performance
case is weak by the repo's own Amdahl numbers (1.03× at 26/870). Two things
are worth taking from it now regardless:

1. ~~**The resident-c2 plumbing is shared with the stage oracle.** M1's
   fork-server/wibo-hook engineering is most of M2's harness. Build it once,
   for the oracle; shipping it as a service afterwards is cheap if a consumer
   exists.~~
   **REVERSED BY MEASUREMENT, 2026-08-21 (arch review §6; #3262; the
   stageoracle rung §6).** The dependency runs the *other* way, and the
   premise it rested on is refuted:
   * **The oracle does not need residency, and building it would have been
     actively harmful.** `w-stageoracle` declines residency **in writing**:
     *"one process per compile is precisely what makes snapshot determinism
     testable — no cross-compile state, no allocator reuse, no counter
     carry-over. Building residency 'for free' here would be building the
     thing most likely to break the load-bearing property."*
   * **Spawn was never the lever anyway.** #3262 counted it rather than
     inferring it: `c2rs` process startup is **under 1 ms** (`c2rs nosuchcmd`
     and `/bin/true` both time at 0 ms over 20 runs), and the expensive spawn
     is `cl.exe` under wibo, one in six. A fork server buys **under 2 %**.
   * **So the shared component is real but the direction is inverted:** what a
     future M1 reuses is already built — `tap_arm`'s slide derivation and
     fail-closed self-check, the multi-source `build_host_stub`,
     `replay_tapped`'s command construction, and the structural fact that c2
     runs inside a host process we own. **What M1 adds is a loop and a socket,
     and it should be graded AGAINST these snapshots as its regression
     fence.** Residency is fenced *by* the oracle; the oracle is not funded by
     residency.
2. **The three definitions of 100% (§4 there) should be adopted as language**
   — vendor-backed coverage, native dc3 parity (870), declared-surface
   general parity — so no future claim of "done" is ambiguous.

Ship M1 as a product decision on its own merits; do not let it reorder the
native plan, and do not let the native plan block it.

---

## 8. Decisions requested

0. **NEW, and it now outranks everything below it: approve rows `4a`
   (integration) and `4b` (IR3) as steps, or say in writing that step 5 is a
   characterization program with no path to the byte judge.** Those are the
   two honest options at tip. `4a` is the **critical path** and is priced at
   **15–45 engineer-months, lower bound** (`STEP5_PRICING_2026-08-21.md`
   §2/§4); `4b` is what makes a per-stage grade for a port *definable at all*
   (#3354). Neither existed as a row in this document before 2026-08-21.
1. Adopt the staged-IR target (§3) and the claims-ledger refusal boundary
   (§3.3), judge unchanged. **[amended: the ledger is UNBUILT and no step
   builds it — adopting it means funding a step for it]**
2. ~~Approve step 0 (stage oracle) as the next characterization campaign, with
   the peer's stop condition verbatim.~~ **[SPENT — landed `427c44255`,
   extended by `b6fd2bf48`; the stop condition needed the partial-fire third
   state, §4.2, and the fire line turned out to run between c2's side and the
   port's rather than between boundaries]**
3. ~~Schedule steps 1–2 (IR0 framer; W8 sum type) as the next quiesce-window
   construct rungs.~~ **[SPENT — `c19d07357`, `c7049bac1`; step 1's
   re-expression half is unscheduled and unowned]**
4. ~~Approve step 3's manifest instrument as a standing gap-metric family — the
   progress curve Option A is otherwise missing.~~ **[SPENT — `14d26f44e`;
   it is a standing family, and it produced ground truth and denominators
   rather than the curve this line asked for, §6]**
5. Treat the vendor service as a separate product track (§7), ~~sharing only
   the resident-c2 plumbing~~ **[amended: the sharing is the other way round —
   the oracle is built, residency is not, and residency should be graded
   against the oracle's snapshots, §7.1]**.

**AMENDED 2026-08-21 — the decision this document does not ask for, and it is
the one that gates everything below step 4** (arch review §7). The economics of
step 5 are **split between two goals**, and the answer flips with the goal:

* Against the **verifier-throughput thesis** (this repo's stated thesis, `README`
  and `CLAUDE.md`), step 5 is a **WRONG-TRADE** on measured evidence — the only
  real consumer is source-space and capped at **≈ 2.4×** even with an infinitely
  fast c2 (corroborated in this repo by `STRATEGY_REVIEW_2026-08-13.md:271`,
  *"≲2.4× without the front end"*); and on the review's own consumer-side
  measurements — **not re-verified in this repo, whose tree contains none of
  the consumer's job env** — its wall-clock bound has moved off compilation
  onto generation, the shipped `c2rs prefilter` seam has **never been enabled
  once** by the consumer, and work-weighted coverage is 46 of 162,147 emitted
  functions (**0.028 %**), making the published 1.03× hybrid payoff ~100×
  optimistic.
* Against **Option A** (reproduce c2 fully, 870/878 — the goal
  `STRATEGY_REVIEW_2026-08-13.md` §8.1 records as decided), step 5 is the right
  and **only** trade: the middle (D ∨ E) is needed by **843 of 845** refused TUs
  (arch review §7), its yield is multiplied 2 → 120 by step 3's factor A, and
  nothing substitutes.

`STRATEGY_REVIEW_2026-08-13.md:251` — *"The question is currently owned by
nobody."* ~~**Still true, eight days on. This is the owner's call, not an
engineering one**, and no amendment to this document can supply it.~~

> **✔ ANSWERED 2026-08-21 — and it was indeed the owner who supplied it, not
> an amendment.** *[`docs/GOAL_DECISION_2026-08-21.md`](GOAL_DECISION_2026-08-21.md);
> `CLAUDE.md` § "The goal". Annotated by lane `w-goaldocs`; the two bullets
> above are left exactly as written.*
>
> The goal is **perfect reproduction**, for two ends ~~ranked equally~~
> **RANKED — (1) primary, (2) a real end AND instrumental to (1); amended by
> the owner the same day, propagated here 2026-08-22 by lane `w-readdocs`
> (`GOAL_DECISION_2026-08-21.md` § "AMENDED")** — (1) a
> clear understanding of MSVC's internals in service of decomp, and (2) parity,
> a 100 % open-source implementation. **The verifier-throughput thesis is
> retired**, so the first bullet's branch is dead: its measurements stand, its
> *conclusion* does not apply, and step 5's economics are read **only** off the
> second bullet — where step 5 is the right and only trade.
>
> **This does not un-gate step 5.** The split above was clause **(d)** of row
> 5's NO-GO; (a) the amendments, (b) row **4a** and (c) row **4b** are
> untouched and still gate it. What the decision does change is **which
> question §8 decision 0 is answering**: not *"is the port worth doing"* —
> that is settled — but *"fund 4a now, or bank understanding first."* Parity
> (goal 2) is unreachable without 4a, because a ported pass with no route to
> `coff::Function` cannot move one obj byte. Goal (1) is served by
> characterization alone, and characterization is now a **first-class
> deliverable** rather than the consolation prize
> `STEP5_PRICING_2026-08-21.md`'s headline reads as.

> **✔ A THIRD OPTION, added 2026-08-21 by the coordinator after the research
> round.** *[`docs/ROADMAP_SLICING_2026-08-21.md`](ROADMAP_SLICING_2026-08-21.md);
> board **#3361**–**#3366**. The framing above — "fund 4a now, or bank
> understanding first" — is left exactly as written, because it was the right
> reading of what was known when it was written.*
>
> Four lenses were dispatched against the owner's question *"how can we chop
> the roadmap into deliverables in fewer than 45 months"*. **Slicing does not
> shorten it**: enumerated bottom-up with `CEILING` §5's ~5:1 applied per row
> it reads **31–59 engineer-months** against the review's top-down 15–45, and
> the direction is the finding — ten constructs carrying 97.3 % of the residue
> were absorbed into two rows, and super-additivity is an overhead *of*
> slicing, not a saving (#3365).
>
> All three candidate shortcuts are refuted, measured rather than argued:
> **no near-miss TU population exists** (the 90–100 % FBM band is empty, hard
> floor 0.200 — #3361); **the 818-TU first refusal converts exactly zero** at
> three lift depths with an identity control, and `body-out-of-class` — row
> 4a's own first half — is co-resident on 818 of 818 (#3362); and
> **composition is refuted four ways**, decisively because it is anti-safe
> under `PROGRESS_METRIC.md` (#3363).
>
> **So the choice is not binary.** Neither branch above needs to be taken
> blind, because there is an **8-week Phase 0** that decides them on evidence
> and banks a standing instrument either way — S0, the blind-reach measurement,
> and S1, a general `Plain`+`Tail` lowering as a required-zero re-expression on
> the live dispatcher (`ROADMAP_SLICING` §5).
>
> Phase 0 attacks the assumption **nothing in this tree has ever measured**:
> whether the port's byte-exactness is a **model or a fit**. `select_function`
> is never called for a parse-refused function, so its 91.2 % is the catalogue
> graded against its own admission gate. The concern is not hypothetical —
> `codegen::alloc`'s clauses are this repo's own *"fitted stand-in"* for c2's
> unread worklist order, **clause 2 refuted on 7 of 56 fresh-holdout cells**
> under a preregistered 52,416-configuration search. If the incumbent bytes are
> a fit, every "general" layer is a re-fit and **4a is not 15–45 months, it is
> unbounded**.
>
> Phase 0 can therefore move the figure **in either direction**, and one of its
> outcomes should **stop the program**: if S1 holds its required-zero delta but
> workload `fnbyte-exact 35,894` moves at all, the pricing basis is void. No
> smaller slice can produce that result, which is the argument for spending the
> 8 weeks before committing to either branch above.
>
> **What this does NOT do:** it does not un-gate step 5 either. Clauses (a),
> (b) and (c) of row 5's NO-GO stand exactly as the block above leaves them.
> Phase 0 is a way to price 4a, not a substitute for approving it.

> **✔ NEW INFORMATION FOR DECISION 0, added 2026-08-22 by lane `w-readdocs`.
> THE DECISION IS STILL THE OWNER'S AND IS STILL OPEN — this block states what
> changed and deliberately does not choose.** *`docs/whitebox/READ_PLAN_2026-08-21.md`;
> `docs/WHITEBOX_LEVERAGE_2026-08-21.md` §1/§3; board **#3367**–**#3368**. The
> three blocks above are left exactly as written.*
>
> Decision 0 asks the owner to approve rows **4a** and **4b**, or declare step 5
> characterization-only. **4a's 15–45 engineer-month price rests entirely on two
> rows — I1 and I2 — and as of 2026-08-21 each has a named, addressed, sized,
> mechanical READ:**
>
> * **I2** ← **R2**, the encoder: two tables (`0x10c3a578` base word,
>   `0x10c39b18` encode form) plus **79 distinct arms** off the jump table at
>   `0x10bfae2d` — 111 entries, 79 targets, **all inside `FUN_10bf9f15`'s
>   3,861 B**, coordinator-verified against the pinned image. **2–4 days.**
> * **I1** ← **R5**, `FUN_10bc2d7a` (5,080 B), the **189-arm** IL-record →
>   codegen-tuple dispatch, **zero arms read today**. **15–25 days**, and it is
>   also the shared input to all ten Phase-1 construct slices.
>
> The sum of all nine ranked reads is **≈6–10 engineer-weeks**.
>
> **Three ways this bears on the choice, and none of them settles it:**
>
> 1. **The third option's framing may be understated in one direction.** The
>    block above frames the choice as *"fund 4a now, or bank understanding
>    first,"* with an 8-week Phase 0 to decide on evidence. R1→R2→R3 is
>    **≈5–9 days** and is squarely "bank understanding" — but unlike
>    characterization generally, its output is **the spec 4a would be built
>    from**, so it is not obviously the *other* branch. Whether that makes it a
>    cheaper Phase 0, a prerequisite *to* Phase 0, or an orthogonal third thing
>    is a scheduling judgement this lane does not make.
> 2. **It does not lower 4a's price, and saying otherwise would be the units
>    error the read-plan itself warns about.** A read produces a spec; I1 and I2
>    are implementations in `crates/`. What the reads remove is the *discovery*
>    cost these estimates carried implicitly — and `CEILING` §5's ~5:1
>    calibration was fitted on lane-shaped construction work, so it must not be
>    applied to a read.
> 3. **It sharpens Phase 0's stop condition rather than replacing it.** Phase 0
>    exists to test whether the port's byte-exactness is *a model or a fit*.
>    `READ_PLAN` §2 is the first enumeration of every fitted constant in
>    `crates/` beside the read that would replace it — three preregistered
>    searches (52,416 / 13,104 / 1,048,576 configurations, each a negative
>    result), ten refuted allocation keys, and seven shipped constants with
>    unread provenance. **That is evidence about the same question, from the
>    other side**, and it arrived after the block above was written. Note
>    `READ_PLAN` §2's own scope bound (#3366): every production caller of
>    `codegen::{alloc, order, schedule}` is inside `leaf/store.rs`, so those
>    constants fence store runs only — they are the **template** Phase 1's
>    general layers would be written in, not a load-bearing surface today.
>
> **And the caveat that keeps this honest:** `[R]` means *"the instructions were
> read correctly,"* not *"this is what c2 does"* (`READ_PLAN` §5.3 — the `.bss`
> bump rule was read correctly out of a clean function and was wrong about c2).

> **✔✔ PARTIALLY DECIDED BY THE OWNER, 2026-08-22** (*"#1 funded for option
> 4"* — `docs/DECISIONS_2026-08-22.md` decision 1; board **#3371**). The
> reads-first path is **funded**: R1→R2→R3 dispatched as characterization
> lanes under prereg. **The branch choice this section poses — approve 4a/4b,
> characterization-only, or Phase 0 — remains OPEN and is deferred until
> R1–R3 report.** Nothing above is approved, re-priced or waived by the
> funding; block 3's three "bears on the choice" points are now the questions
> the reads' results will be read against.
> Every read ends in a confirmation probe, and the byte judge is untouched by
> all of it. **Clauses (a), (b) and (c) of row 5's NO-GO still stand.**
