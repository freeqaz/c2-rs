# Shipping roadmap — compatibility now, native parity deliberately

**Status:** proposal for review  
**Date:** 2026-08-19  
**Reviewed tree:** `fd11ad526729b201f7a8f49b71e405c67d8aecb2`  
**Decision requested:** approve the two-track product strategy and the native
phase gates in §7.

> ### ⚠ 2026-08-21 — **THE OWNER ANSWERED §1's OWN CONDITIONAL, AND THE ANSWER PUTS THIS PAGE'S TRACK 1 BELOW ITS TRACK 2.**
> *[`GOAL_DECISION_2026-08-21.md`](GOAL_DECISION_2026-08-21.md); `CLAUDE.md`
> § "The goal". Annotated in place by lane `w-goaldocs`; **no measurement,
> milestone, exit gate or estimate below is edited or withdrawn**, and this
> page's status stays *proposal for review*.*
>
> §1's last paragraph says: *"If the product requirement forbids using the
> vendor DLL, this is not a near-term shipping project. It is a
> compiler-backend reconstruction program and should be staffed, budgeted and
> reviewed as one."* **The owner's stated goal (2) is parity — a 100 %
> open-source implementation — so that conditional is now TRUE, and this page's
> own consequence follows: it is a reconstruction program.**
>
> * **Track 1 (the vendor-backed compatibility service, M1) is not the goal
>   and cannot become it.** It may still be worth building as a service; it
>   moves the parity scoreboard by zero and depends on the binary parity is
>   defined as replacing. **It may not be ranked ahead of native work on
>   throughput grounds** — the verifier-throughput thesis is retired, and §5.1's
>   2.76×/1.44×/1.01× and §5.2's `1/(1−p)` curve are kept as *measurements* that
>   neither justify nor forbid a lane.
> * **§6's operating-model items are UNAFFECTED and several get stronger.**
>   Items 3 (publish per-subsystem metrics), 4 (byte-weighted coverage), 5
>   (DEV/HELDOUT split) and 6 (ban path-keyed dispatch) are about honest
>   measurement of *reproduction* and stand on their own. Item 1's proposal to
>   retire the board is a live question this annotation does not settle.
> * **§4's "three meanings of 100 %" is the most durable thing here**, and goal
>   (2) is stated in its terms: `870/870` on the pinned dc3 workload is *native
>   dc3 parity*, not proof of all `c2.dll` behavior. That distinction survives
>   the goal decision intact.
> * **M2 (the intermediate-state reference oracle) is PROMOTED.** This page
>   calls it *"the go/no-go milestone for renewed native investment"*, i.e. an
>   instrument earning its keep by unblocking something else. Under goal (1) —
>   understanding MSVC's internals in service of decomp — the snapshots and the
>   whitebox record are **the deliverable**, not the gate in front of it. It
>   has since been built and graded (`docs/rungs/2026-08-20-stageoracle.md`,
>   board #3322) and re-priced (`docs/STEP5_PRICING_2026-08-21.md`).

## 1. Executive decision

This project can ship a useful, exact product soon, or it can continue toward a
vendor-free native backend. Those are different deliverables and should no
longer share one definition of done.

The recommended strategy is:

1. **Ship a vendor-backed compatibility service first.** Keep the real pinned
   `c2.dll` resident under wibo, expose a stable IL-bundle-to-obj interface, and
   use the real compiler for every input the native port cannot prove it handles.
2. **Keep `PortC2` as an optional native fast path.** It may accelerate admitted
   inputs, but must never be the coverage mechanism or silently widen its claim.
3. **Restart native work around subsystem reconstruction.** Build intermediate
   reference oracles, a lossless reader and symbol model, then a general compiler
   pipeline. Stop extending the production backend one named whole-function
   shape at a time.
4. **Define 100% as a versioned conformance surface.** `870/870` on the pinned
   dc3 workload is native dc3 parity. It is not, by itself, proof of all possible
   `c2.dll` behavior.

If the product requirement forbids using the vendor DLL, this is not a near-term
shipping project. It is a compiler-backend reconstruction program and should be
staffed, budgeted and reviewed as one.

## 2. What the project is implementing

The repository targets the MSVC Xbox 360 **back-end DLL**, `c2.dll` from XDK
`16.00.11886.00`. The operative interface is a five-file IL bundle plus compiler
arguments in, and a PPC COFF object plus diagnostics and exit status out.

It does not currently implement the complete `cl.exe` source-to-object pipeline.
That would also require a native replacement for `c1xx.dll`; `c1host` proves the
front-end replay seam but does not port the front end.

The final correctness rule remains unchanged:

```text
normalize_timestamp(PortC2(IL, argv)) == normalize_timestamp(c2.dll(IL, argv))
```

The real backend is the final judge. Intermediate oracles proposed below are
development instruments, not substitutes for the final object comparison.

## 3. Current position

### 3.1 What is solid

- Standalone replay of the real backend is proven and supplies a real
  differential oracle.
- The project has strong capture, replay, corpus, object inspection and
  fail-closed infrastructure.
- Every whole object currently emitted by the admitted TU gate matches; the
  current workload reports zero whole-object mismatches.
- White-box analysis is authorized and has already mapped the reader, object
  writer, symbol model, inliner, EH machinery, allocator and scheduler.
- The native implementation has useful reusable components: PPC encoders, COFF
  writing, framing helpers, label/fixup machinery, a small block IR and many
  precisely fenced regression cases.

### 3.2 Current measured coverage

A fresh scan at the reviewed tree produced:

| metric | current value |
|---|---:|
| workload TUs | 878 |
| graded by the reference compiler | 870 |
| reference capture failures | 8 |
| whole-TU byte matches | 26 |
| whole-TU mismatches | 0 |
| `vocab-gap` | 844 |
| `codegen-gap` | 0 |
| functions in the broad per-function class | about 29.3% |
| emitted functions in class | about 24.3% |
| function-byte match score | about 22.1% |
| emitted functions refused at reader/model boundary | 113,688 |
| emitted functions refused after reaching codegen | 946 |

The checked-in generated status block reports the same strategic position:
`match 26`, `vocab-gap 844`, and emitted coverage `24.27%`.

The most important interpretation is that **zero mismatch is coverage-bounded**.
Most of the workload is refused before the final emitter can be tested. It means
that nothing emitted by the whole-TU gate was wrong; it does not mean the
unreached backend behavior is correct.

### 3.3 The active wall

The current scan reports the first TU-level refusal as
`gl-stop-26-introduced` for roughly 818 of the 844 vocabulary-gap TUs. Across
the emitted-function view, almost all refusal mass is still assigned to the
reader/model side rather than the final emitter.

This label should not be mistaken for one missing byte-width rule. The raw `.ex`
width scanner can already walk much of this population. The coupled problems are:

- lossless parsing versus positive whole-function recognition;
- `.gl` record traversal and body/name binding;
- the emit-set closure and generated-symbol rules;
- typed expression and control-flow semantics;
- whether the resulting body can be lowered by a general backend.

Removing the first fence without implementing that chain merely changes the name
of the next refusal and risks admitting a wrong object.

### 3.4 Architecture assessment

The native path is still predominantly a catalog of recognized shapes. The
central selector has about 38 success returns and dispatches to named lowerers
such as `json_utf8_copy`, `xtea_encrypt_loop`, `pool_ctor_chain`,
`guard_ret_chain` and `ptr_walk_loop`.

These lowerers are valuable behavioral specimens and should remain as regression
tests. They are not a scalable architecture for arbitrary compiler input.

The new block IR is a correct shared foundation, but its own contract records the
missing general mechanisms: code motion, scheduling, condition/value modeling,
cross-block liveness and the machine cost model. The latest CFG census also
shows that promoting the existing partial CFG claims would convert zero frontier
TUs because those functions still fail before they reach that emitter machinery.

### 3.5 Process assessment

The repository contains 372 dated rung documents and about 20,000 lines across
`ROADMAP.md`, `BOARD.md`, `CEILING.md`, `STATUS.md` and the latest strategy
review. There have been 461 first-parent commits since 2026-08-01.

This record has found real defects and should be retained. It should no longer
serve as the day-to-day product backlog. The volume makes current commitments,
superseded measurements and actual completion gates too difficult to distinguish.

There is also one immediate release-hygiene defect. The substantive compiler and
oracle tests pass, but the full workspace command is red on
`rung_index_is_generated_and_current`: `scripts/gen_rung_index.sh` consumes a
shell glob without imposing a stable collation order. The committed index matches
`en_US.UTF-8`; a `C` or `C.UTF-8` environment generates a different ordering.

## 4. Three definitions of 100%

### 4.1 Compatibility-product completeness

Every request is handled by the real pinned `c2.dll`, either directly or as the
fallback behind the native fast path. The service reproduces output bytes,
diagnostics and failure behavior and is validated against ordinary
spawn-per-request execution.

This provides 100% **behavioral coverage through the vendor implementation**.
It is shippable, but it is not a vendor-free reimplementation.

### 4.2 Native dc3 parity

`PortC2` produces a normalized byte-exact object for all 870 dc3 TUs that the
reference compiler successfully compiles, using their pinned real flags.

The target is 870, not 878: the remaining eight fail in the reference pipeline,
so no object exists for a byte-exact port to reproduce.

This is a meaningful product milestone. Its claim is limited to one game, one
compiler build and the observed configuration.

### 4.3 General native backend parity

No `NotImplemented` remains inside a declared matrix containing at least:

- compiler DLL hash and host contract;
- supported IL bundle files and record versions;
- `/O1`, `/O2`, `/Ox`, `/Od` and function-level-linking behavior;
- EH, RTTI, generated symbols and static initialization;
- scalar PPC and VMX128 lowering;
- data, COMDAT, weak-external, relocation and section behavior;
- debug/listing output where in scope;
- failure results and diagnostics;
- PGO/LTCG explicitly included or explicitly excluded.

The declaration must be supported by pinned production corpora, independently
held-out projects and generated cross-mode tests. Differential testing cannot
prove behavior outside that surface.

## 5. Options and return on investment

| option | behavioral coverage | estimated cost | measured or expected return | verdict |
|---|---|---:|---|---|
| Existing capture cache | exact for cached inputs | complete | excellent for repeated captures | keep |
| Persistent/forked `c2.dll` service | vendor-exact | low | measured 2.76x on small bundles, 1.01x on large dc3 TUs | ship for IL-space work |
| Persistent `c1xx` + `c2` service | vendor-exact source-to-obj | low to medium | attacks about 25 ms of measured pipeline fixed cost | highest-priority experiment |
| Native fast path plus vendor fallback | exact overall | medium | about 1.03x on the current dc3 match rate; increases sharply only near complete native coverage | use as product architecture |
| Static recompilation of `c2.dll` | vendor-backed | high | no ISA-emulation cost exists to remove | reject |
| Fully native `PortC2` | eventually vendor-free | very high | large payoff only after roughly 90% coverage | long-term program |

### 5.1 Why the compatibility service is the cheapest shippable option

The fork-server prototype has already established:

- post-`LoadLibrary` fork state is safe for `c2.dll` on the tested population;
- 10,580 comparisons produced zero differing objects;
- small IL bundles improve by 2.76x;
- large dc3 TUs improve by only 1.01x because their time is real compiler work;
- the existing prototype and wibo hook remain available on their development
  branch.

The old decision to decline integration was reasonable for the mixed capture
population, where the weighted result was 1.44x. It should be revisited as a
product decision because IL-space search is dominated by the small-bundle
population where the measured result is 2.76x.

The larger possible win is front-end process reuse. The source pipeline pays
roughly 25 ms of fixed work, and `c1host` already invokes `c1xx.dll` standalone.
A state-safety probe against `c1xx.dll` is required before committing to this
route; the speedup has not yet been demonstrated and must not be promised before
that probe.

### 5.2 Why the current hybrid has little immediate payoff

For a native-hit fraction `p`, the idealized c2-stage speedup of a native-first,
vendor-fallback hybrid is approximately `1 / (1 - p)`. At 26 matches out of 870,
that is about 1.03x. A 2x result needs about 50% native coverage; a 10x result
needs about 90%.

The hybrid remains the right product architecture because it preserves exact
coverage while native support grows. It is not a compelling performance result
at current coverage.

### 5.3 Why static recompilation is not a shortcut

`c2.dll` is a 32-bit x86 PE and wibo is a loader, not an ISA emulator. The host
CPU already executes the compiler's instructions natively. Translating x86 PE to
x86 ELF does not remove an interpretation layer and introduces TLS, exception,
indirect-dispatch and ABI risks without addressing compiler-work cost.

## 6. Operating-model changes

Before beginning another native phase:

1. Replace the active board with 6–8 subsystem epics. Keep dated rungs as a
   research archive and provenance record.
2. Permit a new fixture/rung only when it is a minimized regression, answers a
   pre-registered behavior question, or advances a named phase exit gate.
3. Stop using whole-TU match as the only progress view. Publish separate metrics
   for lossless decode, binding, object planning, IR translation, codegen,
   relocation correctness and final byte match.
4. Use byte-weighted coverage as the main native-progress curve. Historical
   measurements show function counts can overstate byte coverage by roughly 9x.
5. Freeze a DEV corpus and preserve a genuinely unseen HELDOUT corpus. Do not
   repeatedly fit on the final acceptance population.
6. Ban production dispatch keyed to source paths, symbol identities or named
   application functions. Specialized fixtures may remain; specialized compiler
   paths do not count toward the general architecture.
7. Require a phase to end in one of `complete`, `failed` or `stopped by decision`.
   An instrument or document is an input to a phase, not completion of it.

## 7. Proposed roadmap

### M0 — Reproducible release baseline

**Indicative duration:** 1–2 weeks.

Deliverables:

- fix deterministic rung-index generation;
- pin compiler, wibo, workload revision, environment and exact flag matrix;
- regenerate `STATUS.md` from the pinned state;
- define the three completion claims in §4 in user-facing language;
- create one release command and one machine-readable result manifest;
- reduce the active backlog to the subsystem milestones below.

Exit gate:

- a clean clone produces the same generated files and metrics under supported
  locales;
- the complete release test command passes;
- the worktree and workload identities are recorded in the report.

### M1 — Vendor-backed compatibility release

**Indicative duration:** 2–6 weeks.

Deliverables:

- revive and rebase the existing c2 fork-server prototype;
- define a request containing the five IL files, exact argv, environment and
  output path;
- return object bytes, diagnostics, exit status and structured timing;
- add crash isolation, worker supervision, bounded concurrency and cleanup;
- audit a configurable sample against ordinary spawn-per-request execution;
- integrate the current native backend as an optional fail-closed fast path;
- benchmark the actual consumer workload, stratified by IL size.

Parallel experiment:

- run the DLL-initialization and per-request-state probes against `c1xx.dll`;
- if safe, prototype a combined source-to-object service;
- otherwise retain the c2-only service and document the failed state boundary.

Exit gate:

- zero differing outputs against the ordinary vendor path on the fixture,
  generated, dc3 and held-out service corpora;
- 100% fallback coverage for every request accepted by the pinned vendor DLL;
- a measured consumer-level speedup, reported separately for small and large
  inputs;
- installation and operation do not require the Rust native port to be complete.

### M2 — Intermediate-state reference oracle

**Indicative duration:** 4–8 weeks. This is the go/no-go milestone for renewed
native investment.

Instrument the real backend at stable subsystem boundaries and serialize
canonical snapshots of:

1. decoded `.ex` nodes, types and function boundaries;
2. `.gl` symbols, references and body bindings;
3. emit-set closure and function order;
4. tuple/CFG state before and after major optimizer passes;
5. scheduler DAG, chosen order and register assignments;
6. planned sections, symbols, relocations, COMDATs and EH records.

Exit gate:

- at least one representative simple, branching, loop, EH and generated-symbol
  TU can be compared at every relevant stage;
- snapshot schemas are deterministic and versioned;
- final object comparison remains mandatory;
- the team can localize a new divergence to one pipeline stage without relying
  on whole-object byte archaeology.

**Stop condition:** if stable stage observations cannot be obtained, keep the
compatibility product and re-estimate native parity as a black-box research
program before funding further expansion.

### M3 — Lossless IL and symbol frontend

Deliverables:

- decode every record in all five bundle streams without requiring a known
  whole-function shape;
- retain unknown semantics as typed or opaque nodes with their exact source
  bytes and boundaries;
- separate framing, binding, semantic understanding and lowering results;
- reproduce `.gl` traversal, body/name binding and generated-symbol handling;
- port the emit-set reachability closure from the real implementation rather
  than reviving the invalid fitted predicate.

Exit gate:

- every one of the 870 graded dc3 bundles is framed losslessly;
- zero functions are refused merely because a complete function/body boundary
  cannot be represented;
- decoded function, symbol and binding inventories match the stage oracle;
- malformed input behavior is covered by negative tests.

### M4 — Exact object planner

Deliverables:

- function and section emission order;
- packed and `/Gy` COMDAT layouts;
- weak externals and default-symbol recursion;
- section selection, associative relationships and checksums;
- relocation inventory and addends;
- `.data`, `.bss`, `.rdata`, debug and directive-section planning;
- EH region and record planning independent of final instruction bytes.

Exit gate:

- structural object manifests match the reference on all 870 graded TUs:
  section order and attributes, symbol order and fields, relocation locations
  and types, COMDAT associations and EH record inventory;
- remaining differences are attributable to body bytes or explicitly deferred
  content encoders.

M3 and M4 should be implemented together where symbol parsing and object planning
share facts. Their acceptance reports remain separate so a good writer cannot
hide a bad reader.

### M5 — General typed IR and scalar backend

Deliverables:

- typed values and operations rather than whole-function shape fields;
- explicit CFG, block order, terminators and condition producers;
- ABI-aware call, return, frame, stack and aggregate handling;
- general scalar instruction selection;
- dataflow, liveness and cross-block value tracking;
- the optimizer passes required by the pinned workload, in observed order;
- dependence-DAG scheduling and the Xenon machine model;
- register allocation across branches and backedges.

The existing named lowerers become regression fixtures and byte-exact witnesses.
They should progressively stop being production dispatch arms as their semantics
move into the general pipeline.

Exit gates are byte-weighted:

- 25%, 50%, 75%, 90%, 99%, then 100% function bytes exact;
- relocation-correct and relocation-unknown populations are reported separately;
- no wrong whole-object emit is allowed to replace a refusal;
- every threshold is validated on DEV and then checked once on HELDOUT.

### M6 — Hard features and native dc3 parity

Deliverables, as required by the pinned surface:

- exception handling and unwind state;
- RTTI and read-only-data COMDAT families;
- static initialization and compiler-generated functions;
- VMX128;
- all observed optimization modes and pragma mode changes;
- debug and listing data;
- exact diagnostics and failure behavior where claimed.

Exit gate:

- `PortC2` is normalized-byte-exact on 870/870 graded dc3 TUs;
- zero `NotImplemented` within the dc3 contract;
- zero mismatch on the held-out project corpus and generated cross-mode suite;
- real-TU native performance is reported—fixture microbenchmarks are not used as
  a substitute.

### M7 — General backend-parity release

Expand the declared surface one feature/mode family at a time. Each expansion
must include a corpus, held-out cases, reference snapshots and final object
comparison. PGO, LTCG, unusual debug/listing modes and malformed-input parity are
separate release criteria, not implied by dc3 parity.

Exit gate:

- the support matrix has no unknown or untested cell advertised as supported;
- the vendor fallback can be disabled for the declared surface without changing
  results;
- packaging, API stability and consumer integration are release-ready.

## 8. Cost and staffing reality

The appropriate order-of-magnitude budget is:

| deliverable | planning range |
|---|---:|
| compatibility service | weeks |
| stage oracle plus lossless reader/object planner | several engineer-months |
| native dc3 parity after the architecture reset | roughly 12–24 engineer-months, high uncertainty |
| broad general c2 parity | potentially multiple engineer-years |

These are program-sizing ranges, not delivery promises. The native estimate must
be revised after M2 and again after M4. The old shape/rung process implies
3,400–10,400 further lanes and is not an economically meaningful schedule.

If multiple engineers are available, the useful split is by subsystem, not by
fixture:

- reader, types and symbol binding;
- object planning, COMDAT, EH and debug records;
- typed IR, optimizer and lowering;
- reference instrumentation, corpora and release infrastructure.

## 9. Decisions requested

> **⚠ 2026-08-21 — items 1, 5 and 6 are affected by the owner's goal decision
> ([`GOAL_DECISION_2026-08-21.md`](GOAL_DECISION_2026-08-21.md)); see the
> banner at §1. In short: **1** may not be accepted *because* the vendor-backed
> service is faster to ship or faster to run — parity is goal (2) and a
> vendor-backed service moves it by zero, so this is an ordering question about
> two different products, not a ranking. **5** is weakened: M2 was framed as
> the go/no-go *gate* in front of native investment, and under goal (1) its
> snapshots are a deliverable in their own right — it has since been built and
> graded (`docs/rungs/2026-08-20-stageoracle.md`). **6** is a *throughput*
> priority question (which consumer's loop to optimize) and is no longer a
> decision this project needs in order to choose work; the IL-space regime was
> subsequently stood down in both repos (`ARCH_REVIEW_2026-08-21.md` §7).
> Items **2**, **3** and **4** are untouched, and item 3 is if anything
> strengthened. Annotated by lane `w-goaldocs`; nothing below is edited.**

Review should explicitly accept or reject each item:

1. **Product:** ship a vendor-backed compatibility service before native parity.
2. **Scope:** use the three distinct definitions of 100% in §4.
3. **Native strategy:** stop adding production whole-function special cases and
   adopt subsystem reconstruction plus stage oracles.
4. **Governance:** replace the active rung backlog with milestones M0–M7 while
   retaining rungs as evidence/history.
5. **Go/no-go:** require M2 to succeed before committing the larger native
   implementation budget.
6. **Primary target:** decide whether the near-term consumer is IL-space search
   or source-to-object compilation; this decides whether c2-only or combined
   c1xx+c2 process reuse receives priority.

## 10. Immediate next actions

If this proposal is approved, the next bounded sequence is:

1. Fix deterministic rung-index generation and obtain a completely green clean
   build.
2. Pin and publish the release/conformance manifest.
3. Rebase the existing fork-server branch and rerun its exactness suite.
4. Instrument actual consumer calls by bundle size to choose c2-only versus
   combined c1xx+c2 priority.
5. Run the c1xx initialization-state probe.
6. Draft the intermediate-snapshot schema and prove one end-to-end instrumented
   TU before any new native shape work is accepted.

That sequence produces a shippable result quickly and, at the same time, tests
whether the native program can move onto a path that plausibly converges.
