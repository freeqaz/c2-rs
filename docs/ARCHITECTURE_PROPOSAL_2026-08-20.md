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

1. **Parse == accept: there is no lossless decode layer.**
   `IlBundle::functions()` (`crates/c2-il/src/func/bundle.rs:1961` — this read
   `:699` in the first draft, which is inside the `OPT_WORD_*` constant block;
   lane `ir0` measured `:1939`, and its own `ex_segments_*` seams then pushed
   the function down 22 lines to `:1961`, so **the correction was stale in the
   very tree that carried it** — a raw line number into a live file is a
   drift-prone anchor and `board_audit.sh` exempts these only inside frozen
   rung files) couples
   framing, name binding, semantic understanding and *admission* into one
   verdict. Consequences: (a) the whole recognized function is the only unit
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
3. **`IlFunction` is ~15 parallel `Option` shape fields**
   (`crates/c2-il/src/func/mod.rs:2980`) **and `select_function` re-derives
   the shape in a 38-return ordered match.** Named as debt in
   ARCHITECTURE_SEAMS §2.3b on 2026-07-30, scheduled as "step 5, with W8",
   still unbuilt. Every new class costs a field, an arm, and a name-paired
   file — the linear-coverage signature the review's H1 describes.
4. **The TU-level object plan has no home.** The emit-set closure (factor A —
   mechanism already read: the `0x20` bit at `sym+0x4c` closed under
   "referenced by an already-emitted function", `C2_MAP.md` §3E), weak
   externals (675 TUs), COMDAT synthesis (450 TUs), the section ladder (3
   names left) are scattered across `comdat.rs`, `coff/`, `elide.rs`,
   `splice.rs` and unbuilt phases. This matters because **the remaining
   distance is a conjunction**: 523 of 845 remaining TUs fail A, B and C
   *simultaneously* (`w-871`), a perfect reader converts 2 (#3191), a perfect
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
  refusal *mass*, yes — but a perfect reader converts **2 TUs** (#3191);
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

---

## 4. The stage oracle is the go/no-go, and it should come first

SHIPPING_ROADMAP M2, adopted here as **step 0** of the migration: instrument
the real c2 under wibo at stable subsystem boundaries (post-IL-load,
post-COLOR, post-schedule, pre-emit — the addresses are already in
`P_DAG.md`/`P_REGALLOC.md`/`C2_MAP.md`) and serialize canonical snapshots for
representative TUs of each family.

Why first: it changes the economics of every later step. Black-box recovery of
the allocator priced at 52,416 refuted rivals *for one tie-band*; with stage
snapshots, a ported COLOR is graded against COLOR's own output, and a
divergence localizes to a pass instead of costing a whole-object byte
archaeology session. It also converts the whitebox record from reference
material into the port's active specification. **Stop condition** (peer's,
kept): if stable stage observations cannot be obtained, the pipeline port
re-prices as black-box research before any restructuring is funded — and
steps 1–4 below are still individually worth doing, priced on their own.

The final judge is untouched. Snapshots are development instruments; nothing
is admitted on snapshot equality alone.

---

## 5. Migration path — the 26 stay green at every step

Every step is a construct rung or characterization lane (the units CLAUDE.md
already defines), landed under the full gate, with the identity criterion
ARCHITECTURE_SEAMS §0 used four times: census numerator, per-key histogram,
scan JSONL rows, disagreement counters, `match 26 / mismatch 0`, fnbyte
non-decreasing — byte-identical or explicitly accounted, per step.

| # | step | kind | graded by | what it unblocks |
|---|---|---|---|---|
| 0 | **Stage oracle** (§4) | characterization lanes | snapshot determinism + one end-to-end instrumented TU per family | everything in step 5; divergence localization forever after |
| 1 | **IR0 under the current reader**: build the lossless framer; ~~re-express `IlBundle::functions()` and the census splitters as *views* over it~~ | construct | identity protocol + new totality/opaque denominators printed both sides of the change (trap-0/#961 discipline) | **DONE IN PART — lane `ir0`, 2026-08-20, outcome `built`.** The framer, its totality controls and the opaque denominators shipped at a required-zero byte delta (14 `ir0-*` keys; `.ex` **99.61 %** framed on the workload, **31.35 %** on the fixtures). The K1 codec's round-trip — the step's stated spine — ran on dc3 for the first time: **870 of 870**, 0 broken (**#3332**). **The re-expression half was BUILT, MEASURED AND REVERTED**: all eleven production call sites switched, byte-identical by construction, at a cost of **+2.03 % port time per obj** (95 % CI [+1.72, +2.34]; **#3335**). ~~8–14 % of the port's geomean; the price is the `Vec<Record>` the gate view never reads~~ — **both figures WITHDRAWN by the lane's fix round.** The first reading was the measurement's own null (the same protocol reads −13.7 % on the *reference* side, where the effect is zero by construction), and the records-suppressed probe measures **slower**, not faster, so the cause is **unattributed**. **The scheduling answer is unchanged and did not depend on the wrong number: do it inside step 4**, where a record carries a binding, not here — 2 % of the port's throughput for a layer with no reader yet is still the wrong trade at step 1. And the "kills 97% no IR" claim was **false as written** (#3332's rung, DD2) |
| 2 | **The W8 sum type**: `IlFunction.body: BodyShape`; delete the parallel `Option`s and `select_function`'s re-derivation | construct (the long-deferred SEAMS step 5; needs its quiesce window) | identity protocol; census/gate disagreement stays 0 | new classes stop costing a field + an arm + a pair of files |
| 3 | **IR2 ObjPlan + the manifest instrument**: consolidate `plan/`; add the structural-manifest grade over all 870 TUs as a gap-metric family (`plan-exact`, per-component) | construct + instrument | manifest identity on the 26 matched TUs (required-exact); the other 844 become a *measured curve* | un-conjuncts A∧B∧C: emit-set closure, weak externals (675), COMDAT synthesis (450), the 3-name section ladder each become independently gradeable lanes with an honest denominator |
| 4 | **IR1 consolidation**: one binding, `bind.rs`'s pinned-apart seam closed on purpose (the numerator move recorded, not absorbed) | construct | binding counters + JSONL identity; the numerator move published as its own row | claim 2 of the ledger |
| 5 | **Port the middle**: COLOR, the DAG scheduler, the passes the pinned workload exercises, item F liveness — from the disassembly, per-stage against step 0's snapshots, admitted regime-by-regime behind claim 4 | characterization → construct pairs | per-stage snapshot equality on the graded corpus, then the byte judge; existing `alloc`/`order`/`schedule` rules as regression fences | the call-bearing 0.000; the loop classes; the long pole |
| 6 | **Demote shapes**: as the general path reproduces a class byte-exactly, its dispatch arm flips from producer to cross-check, then to `#[cfg(test)]` witness | construct, one class at a time | identity diff of per-class gate counts (#290) — a demotion that moves any count is a finding, not a cleanup | the catalogue stops being production dispatch (peer M5's rule, adopted) |

Ordering notes: steps 1–2 are independent of step 0 and can start immediately;
step 3's instrument half is independently valuable even if step 5 stalls;
step 5 is strictly behind step 0. Steps 1, 2 and the step-3 consolidation are
quiesce-window moves in the SEAMS sense (mechanical, conflict with everything
in flight, land in announced gaps).

**What survives:** the warranty stack whole; encode/frame/labels/block_ir/
cond/coff/elide/splice/comdat; c2-obj; c2-reference; the fixtures; the
whitebox record; the shape files (as witnesses). **What is re-expressed:**
bundle.rs → IR0/IR1 views; shape emitters → MIR lowerings; `select_function`
→ the claims ledger. **What is eventually deleted:** the parallel `Option`
fields; `try_parse_*` as admission gates; the 38-arm match as a production
accept path — each only after its replacement is identity-graded, never
before.

---

## 6. Honest cost, and what it buys

* Steps 0–4 are **weeks each**, not months — they are the same size class as
  moves this repo has already landed (the codegen split, block_ir, the lane
  registry), and each is independently valuable: step 1 gives the reader a
  denominator, step 3 gives the first honest continuous curve toward 870
  (structural manifests, byte-weighted), step 0 caps the price of every
  future divergence.
* Step 5 is the program: the peer's **12–24 engineer-months to native dc3
  parity, high uncertainty, re-estimate after the stage oracle and after the
  plan manifest** is the right order of magnitude and the right hedging. The
  calibration record (optimism ~5:1 on forward costs, CEILING §5) applies.
* What the re-architecture buys that the current shape route cannot:
  the current route prices at 3,400–10,400 lanes with a frontier of 2 and a
  conjunction that makes every subsystem's completion convert ~0. The staged
  design replaces "convert the next TU" with four monotone curves (decode
  totality, binding accountability, manifest equality, per-stage pass parity)
  each of which is gradeable today and none of which can silently regress
  under the existing gate.
* **If the stage oracle fails** (M2's stop condition), the correct fallback is
  not the status quo: steps 1–3 still stand on their own, and step 5 re-prices
  as black-box characterization at the measured rates — which is the point at
  which the vendor-backed service stops being optional product and becomes
  the only shippable coverage story.

## 7. The vendor-backed service (peer proposal §5) — orthogonal, with one shared component

The compatibility service is **product architecture, not compiler
architecture**: it changes nothing in §3, and under Option A its performance
case is weak by the repo's own Amdahl numbers (1.03× at 26/870). Two things
are worth taking from it now regardless:

1. **The resident-c2 plumbing is shared with the stage oracle.** M1's
   fork-server/wibo-hook engineering is most of M2's harness. Build it once,
   for the oracle; shipping it as a service afterwards is cheap if a consumer
   exists.
2. **The three definitions of 100% (§4 there) should be adopted as language**
   — vendor-backed coverage, native dc3 parity (870), declared-surface
   general parity — so no future claim of "done" is ambiguous.

Ship M1 as a product decision on its own merits; do not let it reorder the
native plan, and do not let the native plan block it.

---

## 8. Decisions requested

1. Adopt the staged-IR target (§3) and the claims-ledger refusal boundary
   (§3.3), judge unchanged.
2. Approve step 0 (stage oracle) as the next characterization campaign, with
   the peer's stop condition verbatim.
3. Schedule steps 1–2 (IR0 framer; W8 sum type) as the next quiesce-window
   construct rungs.
4. Approve step 3's manifest instrument as a standing gap-metric family — the
   progress curve Option A is otherwise missing.
5. Treat the vendor service as a separate product track (§7), sharing only
   the resident-c2 plumbing.
