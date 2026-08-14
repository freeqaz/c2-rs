# STRATEGY REVIEW — 2026-08-13

**What this is.** A commissioned review of the project's direction, process and
doctrine, written as a measurement: five named hypotheses were tested against
the tree with the intent to refute them, and this page records which survived,
which did not, and the evidence either way. It was produced by a coordinator
plus four parallel investigation lanes (rung corpus, doctrine, phases,
metrics/goal), each required to cite file:line for every load-bearing claim.
**Base tree: `08915add`** (the `wb-live` merge). No scan was re-run; every
number is quoted from the rung record and generated blocks at that tree, with
the source named. Where two sources disagree, both are quoted.

**The verdict in one paragraph.** The project is not stuck because of its
incentives, and it is not stuck at all in the sense the question assumed — TU
match went 6 → 25 in nine days at an *improving* marginal rate (17.2 → ~4
rungs/TU) and then hit, exactly, the ceiling its own arithmetic had published
in advance: `A∧B∧C = 27`, 25 taken, the last 2 priced at ~20 live refusals
each and declined (`CEILING.md` §2.5, `rungs/2026-08-13-w-keygen.md` §7). What
is true is narrower and more actionable: **the repository maintains exactly one
unit of work — a fixture-claim rung — and that unit structurally cannot build
the seven phases that stand between 27 and 871.** Five of the seven phases
having no building rung is a predicted consequence of the unit, not of effort
or incentives. The two exceptions that exist (board #290's zero-byte construct
rung; `wb-live`'s characterization lane) are exactly the shape the next work
needs, and neither is a first-class convention. The single highest-value move
is to make that shape legal and use it to build `CFG_SHAPE.md` §6.2's block IR.

---

## 1. Ground truth, reconciled

| quantity | value at `08915add` | source | notes |
|---|---|---|---|
| TU match / mismatch / codegen-gap | **25 / 0 / 0** | `rungs/2026-08-13-w-keygen.md` §0 (measured at both lane ends); STATUS generated block (2026-08-10) agrees | |
| vocab-gap / capture-fail | **845 / 8** | w-keygen, both ends | STATUS block (08-10) reads **846 / 7**; same total 853 — one TU moved buckets after collection. Quote from a scan. |
| landed rungs | **209** | `ls docs/rungs/2026-*.md` minus `_`-prefixed; matches `CEILING.md:405`'s definition | the commissioning brief said "280+"; 310 *files* exist, 101 are preregs/findings/index |
| board rows | **1,607 numbered rows; numbering reaches #3069** | `docs/BOARD.md` | "3,000+ rows" conflates the max minted number with the row count; numbers are never reused (BOARD conventions) |
| workspace tests | **1,527** | `rungs/2026-08-13-wb-live.md` gate table | |
| frontier (codegen breadth alone) | **2, both declined** | `gap-metric frontier`; `wordwrap.cpp` (`CEILING.md:1821`), `keygen_xbox.cpp` (w-keygen) | |
| factors | A 28 · B 338 · C 169 · D 24 · E 3; `A∧B∧C` = 27; `A∧B∧C∧(D∨E)` = **25 = match** | STATUS generated block :83-84 | |

Three internal contradictions found while reconciling, all repaired by this
review's companion edits (§7):

1. **`STATUS.md`'s "one-paragraph answer" read "TU match is 10/878"** — 15
   conversions behind the generated block on the same page, wrong since
   2026-08-08.
2. **`CEILING.md` §4 ("Cost per converted TU") was computed at match 11** and
   still headlined "~17 rungs per TU" and "24 rungs since the last conversion
   bought 0" — while §16/§17 of the same file record the 23→24 and 24→25
   conversions. The whole-record rate at 08-13 is **209 rungs / +19
   conversions ≈ 11 rungs/TU**, and the burst-window marginal rate was **~4**.
3. **`CEILING.md` §6.1's "Five of the seven have never had a rung"** is
   contradicted by `rungs/INDEX.md`: items 2 (inliner) and 4 (EH) have had
   five and two rungs respectively. The sentence is true only as "never had a
   rung that *built* the phase" — which is H1 (below), and the literal text
   undersold it.

Also noted: `CEILING.md` carries **two sections numbered "## 10"** (lines 723
and 737), so citations to "CEILING §10" are ambiguous — the widely-quoted
"only `fnbyte-exact` maps to the goal / census fail-open on 845 of 871" lives
in the *second* (the 2026-08-09 addendum).

---

## 2. The hypotheses

### H1 — "The unit of work is mis-sized." **SURVIVES, sharpened.**

The claim: lanes are TU-scoped, phases are not TU-shaped, so no lane can build
a phase and five phases having no rung is process, not coincidence.

What the evidence adds beyond the claim:

* **The unit is defined, in writing, as a fixture claim.**
  `docs/rungs/README.md:14-22`: the slug *is* the fixture prefix; the registry
  test rejects a rung naming no fixture. `CEILING.md:405`: a landed rung is
  "the only lane-shaped unit the repository maintains." A phase has no
  fixture.
* **Every phase-named lane collapsed into the unit.** `w-cfg` produced a spec
  and zero `crates/` change; `w-cfgimpl` shipped one shape (`cond_tail.rs`)
  whose own module doc records the moment the phase was scoped down to a shape
  and *why* ("building [a fixup list] now would be a mechanism with no fact
  behind it", `cond_tail.rs:40-50`); `w-blockir` — *named for the phase* —
  converted one TU. The inline lanes shipped only the decline-side fence; the
  EH lanes shipped readers and censuses. The serial block-IR restructure was
  explicitly **cancelled once on a counterfactual** (2026-07-31 wave).
* **The physical signature.** `crates/c2-core/src/codegen/` is ~40 modules
  named after individual dc3 functions (`xtea_encrypt_loop.rs`,
  `json_utf8_copy.rs`, …), dispatched by an ordered match. There is no
  `BasicBlock`, no `Terminator`, no CFG type anywhere in `crates/`.
  `CFG_SHAPE.md` §6.2 scores **2 of 7 built** (B: labels/fixups, C: two branch
  forms), one half-built (D: range check without the expansion), **A, E, F, G
  absent** — and the spec declared itself "due rather than deferred" on
  2026-08-04, ~150 landed rungs ago.
* **The exemptions exist and are ad-hoc.** Board **#290** (item B) is the one
  construct rung ever landed: 0 fixtures of its own claim, success criterion
  *zero changed bytes*, graded by a line-for-line identity diff of per-lane
  match counts. `wb-live` (2026-08-13) is the one characterization lane:
  `Fixtures: none`, predicted reach 0 at p=0.80, realized 0, and it discharged
  §6.2 item F's "depends on work nobody has done" clause. Both had to disguise
  or exempt themselves; neither is a convention a dispatcher can reach for.

**Refuted part of H1:** "no lane is ever authorized to start one" is too
strong — phase-named lanes were dispatched repeatedly. What is true is that
the unit they had to land in cannot carry a phase, so each either converted a
TU, priced a decline, or wrote a doc.

### H2 — "The shape-whitelist doctrine makes the goal unreachable." **HALF-SURVIVES.**

The operative doctrine (`IL_STMT_GRAMMAR.md:917-920`, `:968-973`;
`CFG_SHAPE.md:1128-1132`): decoding is not licence to emit; the emission gate
"stays a whitelist of *shapes* and never becomes 'the branches decoded'".

**The half that survives scrutiny — the safety property is real, and the
review's own premise was wrong.** The commissioning brief assumed a wrong emit
would be caught as `mismatch` by the next gap scan. The record refutes that
premise: board **#232** was a live wrong emit on master for **255 commits**
while every gate ran green, because no instrument could generate the shape
(`STATUS.md:184-191`); **#2533** was invisible to the entire 878-TU scan and
was caught only by a both-modes fixture neutrality scan (`BOARD.md`, #2533).
The judge is a sampler over a population the port's own refusals shape, not a
net. And the whitelist's stated rationale is a measurement, not taste: a
naively general `if` lowering is wrong on 6 of 7 measured leaf bodies
(`CFG_SHAPE.md:1102-1107`, the fold-band table).

**The half that does not survive — the whitelist bundles two rules, and only
one of them is the safety property.**

1. *"Emit only where every byte is determined by a rule the port can state and
   check"* — a genuine, keepable safety property, and item G is already
   written as a **predicate** ("band 3 or refuse"), not a list.
2. *"The accepted set is a hand-enumerated catalogue of named function
   shapes"* — an implementation choice that hardened into doctrine. Combined
   with a per-TU payoff metric it is a local-optimum trap the project has
   measured on itself: "seven one-function transcriptions bought +7
   `fnbyte-exact` and +7 TU conversions; one 444-wide admission bought +0 and
   +0" (`CEILING.md:786-788`). A general `lower_expr` is derivable at "~640
   lines and ~60 rules plus two cost models" (`ROADMAP.md` §10.27), and "the
   emitter is no longer the constraint; the reader is" (`ROADMAP.md:10825`) —
   yet each individual widening priced out against a per-TU conversion
   metric, so neither side's generality was ever built.

Nowhere in `docs/` had the doctrine itself been named as a candidate cause of
the rate — the arithmetic was admitted in full, the architecture never put on
trial. That was the gap in the record; this page closes it.

**What replaces the list, judge untouched** (the byte-exact compare against
real c2 stays the sole gate):

1. Restate the rule: the emission gate is a **decidable pre-emission
   predicate**; a general lowering behind a checked predicate satisfies it; an
   enumerated list of function names is one implementation, not the rule.
2. **Price every fence two-sided.** The project has done this twice and the
   answer flipped both times: #1042 (a refusal rule costing 1,065 byte-exact
   bodies to remove 531 wrong claims — forbidden) and NC-5/#2691 (a fence
   holding a 5-of-5 byte-exact TU out of `match`). Publish a standing
   `fence-blocks-exact` counter beside `mismatch`.
3. **Grade generality offline before promoting it.** FBM already grades
   per-function bytes against the reference obj without emitting an obj; a
   general lowering can be scored across 162k emitted functions behind an
   env-gated sink and promoted only on the evidence — a larger base than any
   whitelist entry ever had.
4. **Every generalization rung ships a generator for the shapes its predicate
   admits.** The detection asymmetry (16 of 56 shape markers with zero corpus
   cases, `try`/`throw` among them — board #283) is what makes wrong emits
   scary; adversarial corpus per widening is the honest fix, and cheaper than
   enumeration.

### H3 — "The incentive gradient rewards declines and audits over conversions." **REFUTED in its strong form.**

The counts (method: the machine-read `Census:` header field over all 209 rung
docs, then headline classification; 45 docs ambiguous and counted as such):

| outcome | count | share |
|---|---:|---:|
| converted (TU match moved) | 19 docs / 18 events | 9.1% |
| landed an emit, census moved, TU match +0 | 35 | 16.7% |
| priced decline | 61 | 29.2% |
| instrument correction | 38 | 18.2% |
| doc-only / spec | 11 | 5.3% |
| ambiguous compound headline | 45 | 21.5% |

Declines outnumber conversions 3.2:1; decline+audit outnumber them 5.2:1; 76%
of rungs declare `+0` census in their own header. The "ranking artifact"
series is real and larger than folklore: **11 refuted selectors, 12 refuted
placement rules, a 12-deep allocation-key graveyard** — three independent
ordinal series, not ten items.

**Why the strong hypothesis still fails:**

* **The coordinator's alleged thumb on the scale does not appear in the
  corpus.** "A priced decline is a successful rung" occurs **zero times** in
  `docs/`; "successful rung" zero; only 3 of 1,607 board rows carry "priced
  decline" as a verdict; one rung of 209 (`w-target`) uses the framing at all.
  The briefs themselves are not in the repo, so the framing's *presence in
  dispatch* is attested only by the coordinator's own account (see §6) — but
  lanes demonstrably do not cite it when declining.
* **The declines fed the conversions, traceably.** `w-pool` declined
  `Pool.cpp` at 5 mechanisms → `w-pool2` converted it. `w-xtea` declined at
  ≥27 → `w-xtea2` at 3 → `w-xtea3` converted. `w-main` declined at 13 →
  `w-main2` converted. The price-first rungs are why the conversion rungs were
  cheap, and the marginal rate *improved* threefold while the decline count
  grew.
* **The binding cause of the current zero-rate is arithmetic, not appetite.**
  `FRONTIER = 2`, both declined at ~20 refusals; `codegen-gap 0` over all 878
  — no TU waits behind a writer term. The conversion engine stopped because
  its reachable population is empty, at its best-ever rate.

**The residue that is true:** the corpus has an honorific vocabulary for
productive non-results ("the decline is the deliverable", the graveyard
ordinals) and **no vocabulary at all for a lane that simply failed** — a
declining lane is indistinguishable in the record from a converting one at the
level of artifacts produced. That is worth fixing with a one-word outcome
field, not with a doctrine change.

### H4 — "CLAUDE.md constraints are correctness doctrine over-applied to scope." **SPLITS 2/2.**

| constraint | classification | finding |
|---|---|---|
| "outside the ported class, honestly return `NotImplemented`" | **correctness — keep verbatim** | It is the only reason `mismatch 0` carries information. But the *sentence around it* is stale: CLAUDE.md still names "a single framed non-leaf call" in the byte-exact MVP class, and `framed-call` measures **0 byte-exact of 123** on the workload (`ROADMAP.md` §10.30; README already carries the correction). |
| "do not add neutrality / behavior-preserving classifiers as gates" | **correctness — keep, one clarification** | Bans a semantic classifier standing in for the byte judge. It does **not** ban neutrality *measurement* (comparing per-fixture verdicts against real c2 at both modes), which caught live wrong emit #2533. A parenthetical would prevent a new agent conflating the two. |
| "std only, zero external crates" | **scope — benign, one measured side effect** | The big declines (SQLite, registry dispatch) were argued on independent merits. The real cost: instruments get written into `work/`/`scripts/` and thereby escape `cargo test` and `gate.sh` (#1406: two graded instruments "invoked by a human who remembers to"). Add: the rule binds `crates/` and is never a reason to keep an evidence-producing instrument out of the gate. |
| "a wrong emit is strictly worse than a gap" | **correct scoring rule; over-applied as a one-sided decision rule** | As a metric rule it is structurally necessary (`PROGRESS_METRIC.md:30-32`, unit-tested). As a decision rule it became the standard way to decline — while the *refusal's* price is only sometimes counted. When it was counted, the answer flipped both times (#1042; NC-5). Add the symmetric half: any new fence must be priced two-sided, in the units the goal is written in, before it ships. |

This review **did not edit CLAUDE.md** — an agent-commissioned review cannot
self-authorize changes to core directives. The proposed wording is in §5 for
the user to adopt or reject.

### H5 — "The metrics we optimize are not the goal." **CONFIRMED — and the project confirmed it first, including for `fnbyte-exact`.**

Every continuous metric in the tree is labelled "a driver, not the target" in
STATUS's own table, and three have documented instances of moving while TU
match did not (census +444 → fnbyte +0; micro-F1 +7.4pp → 0 TUs, board #250;
byte-fraction keyed on a name channel that disagrees with the binding channel
on 74,955 rows).

The sharper finding: **`fnbyte-exact` is a driver too.** Detector T1
(ALL-EXACT-NO-MATCH) has fired four times — `EncryptXTEA.cpp` sat at
`fnbyte-exact 5 of 5` while grading `codegen-gap`; `w-blockir`'s TU was
byte-exact per body and mismatched as an obj (one symbol short). Only **21 of
865** TUs are 100% fnbyte-exact against 25 matching — the sets differ in both
directions. 1,516 of the credits are two STL templates counted per
instantiation. Derivation (not a published key): >99% of the 35,811 credits
sit in TUs that cannot match at today's factor A. **The metric that is
actually binding is `|D∨E|` — the port's codegen class, counted per TU**
(`CEILING.md` §2.5: perfect A, perfect B, C=871 together move match by +2).

`w-keygen` §7 does not decline to answer whether fnbyte is worth spending on —
it *poses* the question and routes it to `CEILING.md` §6.2, which routes it to
the user. **The question is currently owned by nobody.** This page's §4 is the
staff work for answering it.

### The goal itself — does 871/878 serve the verifier-throughput thesis? **Only in one of the two loops, and the repo already says so.**

* **The hybrid the brief asked about already exists, stricter than proposed.**
  `crates/c2-harness/src/prefilter.rs` is the shipped reject-only pre-filter:
  only `Reject` licenses skipping a real compile, `Match` licenses nothing,
  everything else fail-closed; `GAPS.md` §"only doctrinally-legal integration
  shape" adds 1-in-N real-compile audits. Sound by construction today.
* **Amdahl, with the repo's own numbers** (`docs/perf/perf_scale.csv`: port
  1.105 µs/obj, c2 4.23 ms/obj): hybrid speedup on the c2 stage is ≈1/(1−p).
  At match 25/878: **1.03×**. 2× needs 439 TUs; 10× needs 790; at 871/878:
  **~121×**. Essentially all value lives above p ≈ 0.9 — the goal is a step
  function near completeness, which is an argument both *for* 871 (nothing
  less pays) and *against* per-TU incrementalism as the route (intermediate
  match counts buy ~nothing).
* **The end-to-end cap is elsewhere.** `PRIOR_ART.md:107-137` (measured on six
  real dc3 TUs): an infinitely fast c2 speeds a source→obj compile by
  **1.1–1.6×**; `GAPS.md:1006-1008`: even 100%-coverage backend caps the
  consumer's funnel at **≲2.4×** without the front end. The thesis pays fully
  only in IL-space search, where c2 is 100% of the work — "a research bet that
  has not yet paid" (`PRIOR_ART.md`), and where the meaningful coverage unit
  is "the TU under work", not a workload fraction.
* **A disclosure defect, now repaired (§7):** README quoted 1200–5000× /
  922k/s / 15.3M/s without stating the measurement population — one trivial
  fixture (`cli/perf.rs:207`, `mvp_add3.cpp`) on which **94% of c2's cost is
  process spawn + PE load** (`rungs/2026-08-04-w-fork.md`). On real TUs, where
  c2 does ~150 ms of genuine work, the port's speedup is **unmeasured**
  because the port cannot compile one. `PRIOR_ART.md` recommended stating this
  where the ratio is quoted; it had not propagated.

**Recommendation (recommend, not assume — the goal is the user's):** keep
871/878 as the standing goal *only* paired with the phase plan in §4, because
the arithmetic shows no intermediate match count delivers throughput. If the
thesis (not the number) is the commitment, the highest-leverage alternative
the repo itself has priced is **Option C: a `c1xx`+`c2` fork-server**
(`w-fork` §6: pipeline fixed cost ~25 ms, 3.7× larger than what a c2-only
fork-server removes; `c1host` already drives `c1xx.dll`) — 100% coverage
today, helps the 853 TUs the port refuses, and `w-fork` names it "the lane
worth commissioning." Options in full: **A** hold 871 (pays ~121× at arrival,
~nothing before p≈0.9); **B** ship the prefilter as the deliverable (honest
reading: 1.03× today — it makes the step function explicit); **C** retarget
at the measured bottleneck (fork-server / front end); **D** redefine coverage
in the consumer's unit (needs one input from the consumer: which TUs, how
often — not expressible in-repo today).

---

## 3. Why the TU count is what it is — the one-screen answer to commission question 3

1. The process converts TUs at 4–17 rungs each **inside** a hard reachable set
   of `A∧B∧C = 27`. It has taken 25, and priced-and-declined the other 2 at
   ~20 refusals each. `codegen-gap 0` over all 878: nothing waits behind the
   emitter.
2. The other 846 TUs are all `vocab-gap`: **130,575 of the 139,792 emitted
   functions still needed are blocked at the IL reader** before any codegen
   question can be asked. Reaching them requires the seven phases of
   `CEILING.md` §6.1 — an IR, an allocator, EH, weak externals, COMDAT
   synthesis — none of which fits the fixture-claim rung, which is the only
   unit of work the repo maintains (H1).
3. So the low TU count is neither incentive failure (H3 refuted) nor doctrine
   failure alone (H2 half) — it is the predicted saturation of a correctly-run
   process whose unit of work cannot express the next tier. The volume of
   declines and instrument audits is mostly the sound of that unit being
   applied honestly to a population it cannot convert.

---

## 4. Ranked levers

Costs in the repo's own comparable units where they exist; calibration
(`CEILING.md` §5) applies to every forward number: optimism dominates ~5:1 and
every cost below is a **lower bound**.

| # | lever | what it is | cost signal | what it unlocks |
|---|---|---|---:|---|
| 1 | **Legalize the phase-sized unit** | A first-class **construct rung** kind (0 fixtures, 0 census, *required-zero* byte delta, graded by identity diff of per-lane gate counts — board #290's pattern) and a **characterization lane** kind (`wb-live`'s pattern: prereg, address-cited findings, obj confirmation). A registry/README change plus this precedent. | ~1 doc lane | every lever below stops having to disguise itself as a TU lane |
| 2 | **Build the block IR** (`CFG_SHAPE.md` §6.2 items A, D-expansion, E, G) | Construct rungs re-expressing already-byte-exact classes through the IR, `cond_tail` first (its module doc names the exact gap), then `float_walk_loop`, `ptr_walk_loop`. Zero conversions *by design*. | 2–4 construct rungs (item B cost 1) | items F and G have nowhere to live today; Phase 1 generality and Phase 7 are strictly downstream |
| 3 | **Read `dag.c`'s lowering order** (`0x10b3219f`) | One whitebox lane on the wb-live pattern. It is the *sole* remaining characterization blocker for item F — wb-live discharged the allocator: no interference graph, backward-fixpoint liveness ∩ availability, and #3057 lists four things that are NOT blocking so nobody re-prices them. | 1 characterization lane | item F ("~30 lines plus the textbook" once the IR exists) → register allocation across a back edge → the loop-bearing 80% of the frontier's reader-blocked functions |
| 4 | **Rewrite the emission gate from list to predicate** (H2's four-part replacement) | Doctrine edit + a standing `fence-blocks-exact` counter + offline FBM grading of general lowerings + a generator per widening. Judge untouched. | doctrine lane + 1 instrument rung | removes the local-optimum trap that priced every generalization out; makes lever 2's IR usable for emission rather than transcription |
| 5 | **Reader generality as a phase, graded offline** ⚠ **PRICED 2026-08-14 — see below** | The reader holds 93% of the remaining distance, but `w-readpx` measured that *no reader rung converts a TU* (7 transcriptions = +7; one 444-wide admission = +0). Reader work must be phase-shaped and graded by fnbyte/offline, never dispatched as TU lanes. | unpriced (the largest unknown) | the 846 |

> **⚠ 2026-08-14 — lever 5 is PRICED, and this row's "93 %" is two errors at
> once** (`rungs/2026-08-14-readphase.md`, board **#3092**–**#3098**).
>
> * **The ratio is 89.9 %, and "93 %" is AMBIGUOUS before it is stale.** Three
>   different quantities carry that name: **89.9 %** (share of
>   additionally-acceptable functions), **99.2 %** (share of the *refusal*
>   population) and **70.1 %** (share of the whole denominator). **A lane
>   pricing against the wrong one is off by 29 points.** §3 item 2's
>   `130,575 / 139,792` re-derives to **113,612 / 126,315**; the widening order
>   is **615 keys**, not 648.
> * **The head class's realized worth is NEGATIVE** (#3093). Lifting the
>   *entire* `.gl` walk — two rungs, four clauses — gives `match` **+0** and
>   `fnbyte-exact` **−65**. The 22-token decode-only widening costs **−7
>   `match` and −5,949 `fnbyte-exact`**, and one token (`op:41`) buys **zero**
>   decode distance while costing 2,694 functions and `mmio.cpp` — that token
>   is in every published per-TU ladder's `SEED`. **This row read alone
>   dispatches a lane that loses ground.**
> * **The grading unit this review called missing already existed** and needed
>   a side nobody had asked for: `fnbyte-refused-parse` (**113,612 of
>   162,049**) must fall with **three required-zeros** — `fnbyte-exact`
>   non-decreasing, `match` non-decreasing, `mismatch == 0`. One scan, no obj
>   emitted. First milestone **≤ 44,415**.
> * **The ladder is ≥ 3 clause rungs deep and rung 3 is a 615-key space**
>   (#3095); `decode_causes` under-reports it by up to **725×**, so *"a
>   first-blocker count is not a distance"* holds for the all-cause set too.
| 6 | **Phases 5 and 6 in parallel** (weak-external writer records; COMDAT synthesis) | Terminal, independent, unattempted; populations 675 and 450 TUs. Worth **0 today** and mandatory at 871 — schedule them behind levers 2–5, not before. | unpriced | the symbol-table half of 871 |
| 7 | **Put the goal question to the user** with §2's options A–D | `w-keygen` → `CEILING.md` §6.2 → "the user's call": the chain ends at a decision nobody has been asked to make. | one conversation | either recommits to 871 with the phase plan, or retargets at the fork-server where 100% coverage is available today |

**Explicitly de-ranked:** the inliner phase (CEILING §6.1 row 2) — amended
2026-08-13 to 0 conversions, mechanism smaller than believed (`SPLICE-0`, not
a register allocator); more frontier-TU lanes (the frontier is empty); more
blocked-mass rankings ("a mass ranking… has never once been a forecast of
conversions on this project", and four-of-four dispatched off one found the
ranking was the artifact).

---

## 5. Directive changes recommended (proposed, not applied)

For `CLAUDE.md` (repo) — **not edited by this review**; adoption is the
user's:

1. Correct the stale MVP sentence: the port is "byte-exact on the MVP function
   class *as fenced by the fixtures*; on the 878-TU workload the admitted
   classes are bimodal — ten one-function classes at 11/11 and five
   call-bearing classes at 0.000 over 1,106 bodies (ROADMAP §10.30)". Keep the
   `NotImplemented` rule verbatim.
2. Append to the neutrality-classifier ban: "(This bans a semantic classifier
   standing in for the byte judge; it does not ban neutrality *measurement* —
   comparing per-fixture verdicts against real c2 at both modes is required,
   and caught #2533.)"
3. Append to std-only: "The rule binds `crates/`. It is never a reason to move
   an instrument that grades the port out of the workspace: anything whose
   output is quoted as evidence must run under `cargo test` or
   `scripts/gate.sh` (#1406)."
4. New line near the correctness rule: "A wrong emit scores strictly below the
   refusal it replaced — a scoring rule, unchanged. It is not a licence to
   refuse without pricing: every new fence is priced two-sided (#1042,
   NC-5/#2691), in the units the goal is written in, before it ships."
5. New process line: "Two lane kinds exist besides the fixture-claim rung: the
   **construct rung** (required-zero byte delta, identity-diff graded — board
   #290) and the **characterization lane** (`wb-live`). Phase work is
   dispatched in those units, never as TU lanes."

For the standing dispatch brief (coordinator-owned): keep "a priced decline is
a good outcome" — the record shows declines fed conversions — but add the
missing symmetric element: a lane that neither converts nor declines nor
corrects an instrument reports **FAILED**, in those words, and the rung
header's outcome field says which of the four it was.

---

## 6. What this review did NOT establish

* **No scan was re-run.** All numbers are quoted from rung docs and generated
  blocks at `08915add`; the 845/8 vs 846/7 split is reconciled by trusting the
  newer measurement, not by re-measuring.
* **The dispatch briefs themselves were not read** — they live in session
  transcripts, not the repo. H3's refutation covers the corpus's *response* to
  the alleged framing (zero echoes, declines feeding conversions); it cannot
  exclude that the framing shaped which lanes were dispatched at all
  (selection, rather than grading, bias). The orchestration memory does
  contain "tell agents that declining… is a good outcome", so the mechanism
  existed; its net effect is what the corpus fails to show as harmful.
* **45 of 209 rung docs (21.5%) resisted classification** — compound headlines
  asserting several findings. The category counts have that much slack; the
  76%-declare-census-+0 figure does not (it is mechanical).
* **The cost of levers 2–5 is not established.** Item B cost one lane, but the
  calibration record (5:1 optimism, misses specifically on forward cost) says
  to read every §4 estimate as a lower bound. The reader phase (lever 5) is
  entirely unpriced.
* **That a general lowering behind a predicate keeps `mismatch` at 0 is a
  design argument, not a measurement.** #232 is the standing precedent that
  widenings mint wrong emits; the two-sided pricing and per-widening
  generators are mitigations, not proofs.
* **Which loop the consumer actually runs** (IL-space vs source-candidate) —
  Option D needs an input from decomp-synth that this repo does not hold, and
  `GAPS.md` is explicit the re-assessment is the consumer's to re-issue.
* **">99% of fnbyte credits sit in never-matching TUs" is a derivation** from
  published counts, not a published key; treat accordingly.

---

## 7. Edits made by this review (same branch, `wt-strategy-review`)

1. This file.
2. `docs/STATUS.md` — the "one-paragraph answer" prose corrected from the
   stale "TU match is 10/878" to the current reading, with the burst window
   named; the generated block untouched.
3. `docs/CEILING.md` — dated staleness banner on §4 (computed at match 11;
   current whole-record and marginal rates given); §6.1's "five of the seven
   have never had a rung" tightened to "never had a rung that built the
   phase"; a disambiguation note at the first of the two "## 10" headings.
4. `docs/CFG_SHAPE.md` — **no edit needed**: §6.2 item F's "depends on work
   nobody has done" clause was already amended by the `wb-live` merge itself
   (commit `ab4033b0`); an investigation-lane claim that it had not been was
   checked against HEAD and found stale.
5. `README.md` — population caveat beside the 1200–5000× claim,
   citing `PRIOR_ART.md`'s Amdahl section, as that file recommended.

Not edited, and why: `CLAUDE.md` (§5 — user's call), `docs/BOARD.md` and
`docs/rungs/INDEX.md` (contended seams), `docs/whitebox/` (lane-owned),
`scripts/gate.sh` and helpers (lane `w-gate3048` pending), anything in
`crates/` or `fixtures/` (this is a review, not a lane).
