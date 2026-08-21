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
               `docs/ARCH_REVIEW_2026-08-21.md` (seven review lenses) and the
               measurements of the very lanes §5 commissioned. **Every
               amendment is in place and visible**: refuted text stays under
               ~~strikethrough~~ with the row that killed it named beside it,
               per the step-1 row's own precedent (`765095b42`). Nothing below
               is silently rewritten. **The review's verdict is recorded here
               and not softened: step 5 as written is NO-GO** — its per-stage
               grading premise fails for both of its named targets, and two
               hard prerequisites appear in no row of §5 and no line of §6.

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
   **AMENDED 2026-08-21 (arch review §6; board #844; #3345).** The count was
   never "~15": board **#844** had carried *"34 mutually-exclusive
   `Option<Shape>` fields dispatched by an ordered `is_some` chain"* since
   2026-08-06, and this document undercounted its own citation by more than
   half. **And the debt is now paid**: §5 step 2 landed as lane `w8sum`
   (merge `c7049bac1`, 2026-08-21), collapsing the 34 fields into one
   `pub body: BodyShape` of **35 variants** and turning `select_function`
   (`crates/c2-core/src/codegen/select.rs:275`) into a **total, exhaustive
   35-arm `match`**. The invariant "exactly one is `Some`" is the `enum` now,
   not a convention — the #844 half-emit is unspellable by construction.
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
   *simultaneously* (`w-871`), a perfect reader converts 2 (~~#3191~~
   **#3190** — citation corrected 2026-08-21, arch review "what survives";
   #3191 is the per-TU blocking-set distribution, a different row), a perfect
   section emitter converts 0 (#3210), item F converts 0 (`w-itemf`). No
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

* no intermediate match count pays (hybrid speedup 1.03× at 26/870; value
  arrives near p≈0.9) — so the architecture must optimize for *eventual
  totality*, not for the next conversion;
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

### 4.1 AMENDED 2026-08-21 — what step 0 actually bought, boundary by boundary

The struck sentence above was the load-bearing premise of §5 step 5, and **the
lane this section commissioned refuted it in the two places it mattered**
(`w-stageoracle`, merge `427c44255`; board **#3323**; rung §3, §6.1). Recorded
here rather than in a footnote, because step 5 was priced on it.

| boundary | gradeable? | measured |
|---|---|---|
| `sched1` → `sched2` (a scheduler run + globregs) | **YES** | **DIFFERS 7 of 7** functions of `il_call_perm.cpp` (13,14,14,14,14,14,14 → 11,11,12,13,12,12,12 rows) |
| `sched2` → **COLOR** → `sched3` | **NO** | **IDENTICAL 7 of 7**, 83 tuples; raw window 128 B/tuple over 83 aligned pairs — *offsets COLOR writes: NONE* |
| `sched3` → `sched0` (the lowering band) | **YES** | **DIFFERS 7 of 7** |
| `sched0`'s **own output** (the run that fixes emitted instruction order) | **NO** | observed **nowhere**: the walks fire at region-finder entry (`c2host/stagetap.c:369`) and run 4 has no successor run to pair against |

**The two DIFFERS rows are the control that makes the COLOR null credible.**
The same five fields, read by the same walk on the same run, move at the
scheduler and at lowering; an instrument blind everywhere would be vacuous,
this one is blind in exactly one place, and
`the_tuple_walk_sees_the_scheduler_move_the_list` fences that — if it ever
fails, every COLOR conclusion here is void.

**So the position is: the scheduler and lowering boundaries ARE per-stage
gradeable today; COLOR's output and `sched0`'s output are NOT.** The assigned
register is not in the tuple record's first 128 bytes — it lives in the operand
records the tuple points to, or in the allocator's own candidate records
(the `0x48`-byte record, `docs/whitebox/ref/P_REGALLOC.md` §4.1 — the
stageoracle rung cites this as *"§5's `cand+0x28` / `+0x3c`"*, which is one
section off: `+0x3c` is in §4.1's table and `cand+0x28` is read by
`0x10b30517` in §2's function table). Both deciding probes are named,
cheap, and are `w-restim`'s (arch review "Consequences" 3): the operand /
candidate-record walk (stageoracle §6.1 q1) and a `sched0`-output probe
(§6.1 q2). **Until those land, a ported COLOR has no per-stage grade and step
5's allocator pricing is unchanged from black box** — stageoracle §6.1 q1, in
those words.

**Instrument caveat, and it is a live wrong-but-green:** the raw-window verdict
that produced *"offsets COLOR writes: NONE"* skips length-mismatched functions
(`if b2.len() != b3.len() { continue; }`) and then tests only `hot.is_empty()`
— `crates/c2-harness/src/cli/stage.rs:624,638`. With **zero** comparisons it
prints *"the allocator wrote nothing in this window"*. The #3323 instance had
83 real aligned pairs so the board row is **not** vacuous, but the instrument
can be, and guarding it on `compared > 0` is dispatched (arch review
"Consequences" 4).

### 4.2 AMENDED 2026-08-21 — the stop condition is binary and reality is a PARTIAL FIRE

The stop condition above admits two states — *stable stage observations
obtained*, or *re-price as black-box research*. **Neither is what happened.**
The mechanism is green (determinism, canonicality, a null control, and a
required-zero neutrality grade against the judge) **and one pass is
unobservable**. The proposal had no language for that state, and it is the
single highest-value amendment the review found (arch review §6).

The rule this section now carries:

> **Partial fire is a first-class outcome of a characterization campaign, and
> it is priced per boundary, never per campaign.** A mechanism that observes
> *n* of *m* boundaries funds the *n* and re-prices the *m − n* as black box —
> it does not fund all *m*, and it does not stop the campaign. Concretely at
> tip: the scheduler and lowering ports may be dispatched per-stage; the COLOR
> port and anything downstream of `sched0` may not, and any lane that claims
> otherwise is quoting a boundary this instrument never observed.

### 4.3 AMENDED 2026-08-21 — the correct positive case for step 0, which is stronger than the one above

The argument §4 made for step 0 was *"a ported COLOR is graded against COLOR's
own output"*, and that is exactly the claim that failed. **The case that
survives is better and was available before the lane ran** (arch review, "what
survives"):

**The snapshot captures the tuple order *entering* COLOR** — which is item
**F0**'s input half, and F0 is **8 of item F's 17 lanes**, the largest single
line in `CEILING.md` §6.1's decomposition (F1 2 · F2 1 · F4 2 · F5 2 · F6 1 ·
F7 1 = 9 for everything else combined). It is also **exactly the black-box
blind spot**, in CEILING's own words: downstream of allocation sit four
order-changing stages, so *"the order that decided the registers does not
appear in the obj"* — which is why `codegen::alloc`'s 52,416-configuration
residual and `codegen::schedule`'s 13,104-configuration residual are both fits
against bytes that no longer carry the input the rule was keyed on.

Step 0 bought the input half of the most expensive item on the ceiling page.
That is the claim to make for it. It is not the claim §4 made.

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
| 0 | **Stage oracle** (§4) | characterization lanes | snapshot determinism + one end-to-end instrumented TU per family | **neutrality**: armed-vs-disarmed obj bytes. A tap that perturbs c2's own output makes every snapshot a measurement of the tap | **LANDED — lane `w-stageoracle`, merge `427c44255`, 2026-08-20, outcome `built`; verdict GO, with one boundary named.** The mechanism is green: G1 neutrality **required-zero** against the judge, G2/G2b determinism + canonicality, G3 the null control, G5 content cross-derived three ways. Board **#3322**–**#3326**. **Deviations from this row as written, all deliberate and all published:** (i) **residency was NOT built** and the rung says why — one process per compile is what makes snapshot determinism testable (§7.1 is reversed below); (ii) P11's 57 phase-beacon sites are **not armed**, and the record correction stands anyway — `0x10bec297` is the abort poll, not *"the timer"* `P_DAG.md` §2 calls it; (iii) **6 of the 26 matched TUs produce a structurally EMPTY snapshot** (`hits 0 · regions 0 · tuples 0`) because they emit no function bodies — reported, not averaged away; (iv) the "2 functions × 7 sites" arithmetic in the hand-off was wrong twice over and is corrected in the rung. **And the headline this row promised did not survive: see §4.1** — COLOR's output and `sched0`'s output are observed nowhere, so *"everything in step 5"* is **not** what it unblocked. What it did unblock is the tuple order *entering* COLOR, i.e. item F0 — 8 of item F's 17 lanes (§4.3) |
| 1 | **IR0 under the current reader**: build the lossless framer; ~~re-express `IlBundle::functions()` and the census splitters as *views* over it~~ | construct | identity protocol + new totality/opaque denominators printed both sides of the change (trap-0/#961 discipline) | **throughput** — and this is the row where it FIRED: the switch was byte-identical by construction and cost port time, so byte identity could not have caught it (#3335, #3336) | **DONE IN PART — lane `ir0`, merge `c19d07357`, 2026-08-20, outcome `built`.** The framer, its totality controls and the opaque denominators shipped at a required-zero byte delta (14 `ir0-*` keys; `.ex` **99.61 %** framed on the workload, **31.35 %** on the fixtures). The K1 codec's round-trip — the step's stated spine — ran on dc3 for the first time: **870 of 870**, 0 broken (**#3332**). **The re-expression half was BUILT, MEASURED AND REVERTED**: all eleven production call sites switched, byte-identical by construction, at a cost of **+2.03 % port time per obj** (95 % CI [+1.72, +2.34]; **#3335**). ~~8–14 % of the port's geomean; the price is the `Vec<Record>` the gate view never reads~~ — **both figures WITHDRAWN by the lane's fix round.** The first reading was the measurement's own null (the same protocol reads −13.7 % on the *reference* side, where the effect is zero by construction), and the records-suppressed probe measures **slower**, not faster, so the cause is **unattributed**. **The scheduling answer is unchanged and did not depend on the wrong number: do it inside step 4**, where a record carries a binding, not here — 2 % of the port's throughput for a layer with no reader yet is still the wrong trade at step 1. And the "kills 97% no IR" claim was **false as written** (#3332's rung, DD2). **AMENDED 2026-08-21: THE DEFERRAL EXPIRED WITHOUT BEING HONOURED.** Step 4 has landed (merge `d7be7aadc`) and `git diff c7049bac1..d7be7aadc -- crates/` touches exactly `bind.rs`, `census.rs`, `diag.rs` — **no `stream/` file, no call site switched**. So *"do it inside step 4"* named a step that then did not do it, and the arch review's §2 finding applies at full force: the production switch survives in **no** commit (the "reverted" commit `09d42b15f` is 47 doc-comment lines), so redoing it starts from zero across eleven call sites. Whoever picks it up owns the re-derivation, not a revert |
| 2 | **The W8 sum type**: `IlFunction.body: BodyShape`; delete the parallel `Option`s and `select_function`'s re-derivation | construct (the long-deferred SEAMS step 5; needs its quiesce window) | identity protocol; census/gate disagreement stays 0 | **precedence**: the old `is_some` chain's *order* was load-bearing in exactly one place, so a variant promoted to a peer arm that should have stayed ordered moves no byte on the corpus that ran — it moves them on the one that did not | **LANDED — lane `w8sum`, merge `c7049bac1`, 2026-08-21, outcome `built`; board #3345.** The **34** `Option<Shape>` fields (board **#844**, not the "~15" §1.2 item 3 claimed) collapse to one `pub body: BodyShape`, **35 variants**; `select_function` is a **total, exhaustive 35-arm `match`**; `cfg_class::lowering_of`, the mirror precedence helper, converted the same way. Required-zero held: 18-lane gate table **diff = 0 lines** over all 23 count-bearing rows, `mismatch 0`, `match 26 / vocab-gap 844` unmoved, partest **1,761 / 0 / 1** with **0** `SKIP: toolchain absent`, net **−438 lines / 35 files**. **Deviation worth carrying:** one precedence survives and is transcribed verbatim — `Plain`'s four-matcher leaf order (`indirect_load_text` → `addr_leaf_text` → `store_leaf_text` → `select_text`), because those four read the same `func.ops` stream; the order *over the 34 fields* was incidental (they were mutually exclusive at the parser) and is now a type. **The `is_some` half of this row is done; the "pair of files" half is not** — that is step 6 |
| 3 | **IR2 ObjPlan + the manifest instrument**: consolidate `plan/`; add the structural-manifest grade over all 870 TUs as a gap-metric family (`plan-exact`, per-component) | construct + instrument | ~~manifest identity on the 26 matched TUs (required-exact); the other 844 become a *measured curve*~~ — **VACUOUS as written** (#3330): with the port side computed as `observe(port_obj)`, `port_bytes == ref_bytes` on the 26, so `manifest(port) == manifest(ref)` for **any** pure `manifest`; and on the other 844 the port emits no obj, so the port side is **undefined**. Repaired by two independent producers plus a **named** control (`docs/plan/CONTROL_TUS.txt`) | **the control's own size**: a `plan-*` component whose extractor collapses to `Some(empty)` reads *more* exact, not less — a 20-TU mutation reads `exact` unchanged and control size **0**. `plan-control-obs-size`/`-obs-empty-tus`/`-substantive-tus` now publish it | **LANDED — lane `w-objplan`, merge `14d26f44e`, 2026-08-20, outcome `built`; board #3327–#3331.** ~~un-conjuncts A∧B∧C: emit-set closure, weak externals (675), COMDAT synthesis (450), the 3-name section ladder each become independently gradeable lanes with an honest denominator~~ — **RE-SCOPED 2026-08-21 against the lane's own §5 "found and NOT taken"**: (1) **the SECTION component was not built and the reason is not time** — its only walk-free substrate is the whole-TU recognizers on the *admission* side of the reader, i.e. the exact route `plan/`'s fence exists to keep out; shipping it would have printed `known ≈ 30 of 870` and re-published `vocab-gap 844` under new keys; (2) **weak externals were not made a component** because the port models none, so it would grade **`Unknown` on 870 of 870** and say nothing the `Unknown` histogram does not; (4) the emit-set closure lane is **re-priced**, and its first deliverable changed — `gl_function_attrs`' silent **skip path**, not the closure. **Both components that DID ship, ship `Unknown`** (#3329/#3330). **What this step actually delivered is ground truth, not curves**: 870/870 observed inventories (weak externals 674 TUs / 4,009 records; associative COMDATs 101,120 sections over 846 TUs; `SELECT_LARGEST` on 5,214 sections, which nothing in the repo had counted; `sel-unknown` **0**) and **four lanes now denominated** — emit-set closure, weak externals, COMDAT synthesis, the section ladder (853/854 distinct name/attr sequences, owing only a walk-free predictor). **And the predictor denominator ceiling is 854, not 870**: `gl_function_attrs` answers on 854 TUs, so 16 are outside any `.gl`-keyed curve before it starts |
| 4 | **IR1 consolidation**: one binding, `bind.rs`'s pinned-apart seam closed on purpose (the numerator move recorded, not absorbed) | construct | binding counters + JSONL identity; the numerator move published as its own row | **a name the compiler cannot grade**: the byte judge is silent on `mangled_name`, so a binding change is wrong-but-green by construction and only a gap metric can see it (`docs/GAPS.md` §8) | **LANDED — lane `ir1`, merge `d7be7aadc`, 2026-08-21, outcome `built`; board #3347–#3348.** `Bindings::positional` (a narrow `mangled_names` scan, `paired` on **count alone**, body offsets ignored, blind to `??`-names and to data-vs-function) is **deleted**; the census and the `diag.rs` locals probe now use `Bindings::census` — the gate's exact, `??`-aware, offset-checked 1:1 pairing, made **total** (empty names where it does not bind). Required-zero obj byte delta; the emit path was already on `per_record`/`selective` and is untouched. **The numerator move is published, not absorbed**: `fnbyte-name-disagree` **74,033 → 0**, and a full `diff` of the 484-line GAP-METRICS block base-vs-tip returns **exactly that one changed line**. Not vacuous: of the emitted rows, 44 now carry a per-record name and **all 44 agree**. **Deviation:** step 1's deferred re-expression (*"do it inside step 4, where a record carries a binding"*) was **not** done here — see the step-1 row |
| **4a** | **INTEGRATION — the two prerequisites step 5 has and no row budgets** (added 2026-08-21, arch review §3): (i) a **general op-level IL decode** — IR0 stops at a two-variant byte framing and `BodyShape` starts at 35 whole-function grammars that are *simultaneously the admission gate*, so the semantic middle a COLOR pass would consume does not exist; (ii) a **general lowering to `coff::Function`** — today a 35-arm per-shape dispatch, `crates/c2-core/src/comdat.rs` carrying **43** `Selected::` references | construct pairs, quiesce-window | required-zero byte delta on the shipped classes re-expressed through the general path (the #290 protocol), **plus** an explicit statement of which `coff::Function` field each new general lowering writes | **throughput, and stage-parity-with-no-consumer**: without this row a step-5 lane's only progress signal is snapshot parity — an instrument with **no emit-path consumer**, i.e. **#3336 at program scale**, and unlike #3336 there is no contrast case to catch it | **PRICED TWO-SIDED, per the standing rule.** *Cost of building it:* reviewer's estimate **3–9 engineer-months** — and CEILING §5's calibration (optimism ~5:1 on forward cost) makes that a **lower bound**, not a range. *Cost of NOT building it:* step 5 cannot reach the byte judge at all, so every step-5 lane grades on snapshot parity only, and the judge — the project's sole correctness rule — never sees the ported pass. **That is the more expensive side**, which is why this row exists rather than a footnote. **Cheap prophylactic, adopted now and independent of the row landing:** every step-5 lane names in its rung header which `coff::Function` field its pass would eventually write |
| **4b** | **IR3 gets its own step** (added 2026-08-21, arch review §4 — it was the only IR in §3.1 with no migration step): define IR3 **in c2's coordinates**, not in the port's. The proposal contains **zero** occurrences of "tuple" and **zero** of "region" — the words that name c2's actual IR (`WB_REGALLOC_FINDINGS.md`: a tuple *list*, not a tree) and the scheduler's unit of work. IR3 must **carry** tuple identity — **opcode number** (c2's own numbering: `li` = 624 vs `addi` = 11), **category**, **monotonic index** — and the **region partition** as first-class fields, following this repo's own `Terminator::Bc`-carries-`(BO,BI)` precedent | construct | **the port→snapshot projection as a graded artifact, with its own required-zero criterion** — this is the deliverable, not a side effect | **non-functionality of the map**: the port's encoded-byte blocks have already erased c2's opcode numbering, so port→tuple is not a function until IR3 carries the number; and **regions cut through shipped block boundaries** (a call *ends* a region in c2 and sits mid-block in the port), so a projection can be green per block and wrong per region | Without this step the per-stage grade is ill-defined by four independent routes (arch review §4 a–d), including that **COLOR's real state lives in a third representation** — 0x48-byte candidate records with a phase-overloaded field — that IR3 as §3.1 defines it has no concept for. ~~"cross-block liveness (item F)"~~ is the name §3.1 uses and **this repo has refuted that name**; item F's actual price is **17 lanes** (`CFG_SHAPE.md`, `CEILING.md` §6.1), which §3.1 never quotes |
| 5 | **Port the middle**: COLOR, the DAG scheduler, the passes the pinned workload exercises, item F liveness — from the disassembly, per-stage against step 0's snapshots, ~~admitted regime-by-regime behind claim 4~~ **admitted regime-by-regime behind today's refusal predicates** (`BodyShape` admission + the regime fences: `MAX_MODELLED_PRODUCERS`, the signedness fences, `LabelMap` invariant 4) — **reworded 2026-08-21 because §3.3's claims ledger is unbuilt and no step in this table builds it**; whoever wants the ledger owes it a step | characterization → construct pairs | ~~per-stage snapshot equality on the graded corpus~~, then the byte judge; existing `alloc`/`order`/`schedule` rules as regression fences. **AMENDED: per-stage snapshot equality is available at the `sched1`→`sched2` and `sched3`→`sched0` boundaries and NOWHERE ELSE** (§4.1) | **grading a pass at a boundary the oracle never observed.** COLOR and `sched0` have no per-stage grade at tip; a lane that reports "stage parity" for either is reporting an instrument, not the pass | **NO-GO AS WRITTEN, 2026-08-21** (arch review, verdict). Gated on: (a) these amendments; (b) row **4a** priced by the re-estimation lane; (c) `w-restim`'s deciding probes for COLOR and `sched0`; and (d) the **owner** re-owning the thesis-vs-870 goal question, unowned since `STRATEGY_REVIEW_2026-08-13.md:251` (*"The question is currently owned by nobody"*) and still true. **Depends on 4a**, not on step 0 alone. What it would unblock is unchanged and is still the prize: the call-bearing 0.000; the loop classes; the long pole |
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
  | 0 stage oracle | a named boundary set, each boundary either **observed** (a pre/post pair that DIFFERS on a control fixture) or **published as unobserved**; neutrality required-zero against the judge. *Met, with COLOR and `sched0` in the unobserved column (§4.1).* |
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
  it converts a floor into a centre. **And 12–24 does not contain row `4a` at
  all** — the two prerequisites are an additional 3–9 engineer-months on the
  reviewer's estimate, itself a lower bound under the same rule. The honest
  statement is: **≥ 12–24 engineer-months for the middle, plus ≥ 3–9 for the
  integration it presupposes, and the project's own record says both numbers
  are the low end of their distributions.**
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

  What the design *does* buy, stated without the three predicates: **four
  denominated lanes where there were none** (emit-set closure, weak externals
  674 TUs, COMDAT synthesis 846 TUs, the section ladder 853/854), each with
  ground truth graded on the reference side — which is a real and new thing,
  and is not a curve.

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
  it did not succeed — it PARTIALLY FIRED** (§4.2), so this bullet's binary
  fires per boundary. The scheduler and lowering halves are funded at the
  measured rate; **the COLOR half is already in the black-box column** and its
  rate is the one this repo has measured twice — 52,416 refuted allocator
  rivals for one tie-band, 13,104 refuted schedulers.

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

1. Adopt the staged-IR target (§3) and the claims-ledger refusal boundary
   (§3.3), judge unchanged. **[amended: the ledger is UNBUILT and no step
   builds it — adopting it means funding a step for it]**
2. ~~Approve step 0 (stage oracle) as the next characterization campaign, with
   the peer's stop condition verbatim.~~ **[SPENT — landed `427c44255`; and
   the stop condition needed the partial-fire third state, §4.2]**
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
nobody."* **Still true, eight days on. This is the owner's call, not an
engineering one**, and no amendment to this document can supply it.
