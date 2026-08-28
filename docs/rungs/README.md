# `docs/rungs/` — one file per rung

Rung write-ups land here, one file per rung, instead of growing a new
`§6`-letter section in `docs/ROADMAP.md`. The reason is in
`docs/ARCHITECTURE_SEAMS.md` §2.5/§3.4: on 2026-07-30 nine rungs landed in
parallel and *every* merge conflicted on `ROADMAP.md`, `GAPS.md` and
`expr_sweep.sh`, while the tag `W23` and the section letters §6e/§6f/§6g/§6i
were each claimed twice by concurrent agents. Section numbers and small integers
allocated concurrently collide silently; **filenames collide as add/add
conflicts git flags loudly**.

## The convention

* **Filename is the claim.** `YYYY-MM-DD-<slug>.md`. The slug is the rung's
  identity and matches its fixture prefix's meaning (`w25_store_leaf.cpp` →
  slug `store-leaf`). Two rungs claiming one slug is an add/add conflict.
* **`W`-numbers are assigned at merge**, by the one serial actor with the one
  sequence — never by a branch. A branch may leave `Tag:` as its slug until
  then. See §3.4.
* **Header block is machine-read.** `crates/c2-harness/tests/rung_registry.rs`
  (portable lane, no toolchain) parses the indented `Key: value` block and
  asserts: tags unique, declared slug equals the filename slug, every named
  fixture exists, every rung names at least one fixture, no fixture prefix
  claimed twice, and `INDEX.md` equal to what `scripts/gen_rung_index.sh`
  generates.
* **`INDEX.md` is generated**, never hand-edited: `scripts/gen_rung_index.sh`.
* Files whose name starts with `_`, plus `README.md` and `INDEX.md`, are not
  rung docs and are skipped by both the test and the index.

Start from `_TEMPLATE.md`.

## `W-<NAME>-<N>` is claimed by two registries — a bare one is a RUNG TAG

*Added 2026-08-27 by lane `w-wire` (owner decision 19,
[`../DECISIONS_2026-08-22.md`](../DECISIONS_2026-08-22.md) § Decision 19).
Board **#3681**. The mirror of this block lives in
[`../whitebox/DISCLOSURE.md`](../whitebox/DISCLOSURE.md) § "`W-<NAME>-<N>` is
claimed by two registries"; each end names the other's claim.*

The `W` sequence above is not only `W22`/`W25`/`W26`/`W30`. It has also issued
**hyphenated** tags, and `INDEX.md` line 16 carries one:

> `| 2026-07-30 | W-UNW-1 | [unwind-pdata](2026-07-30-unwind-pdata.md) | 5 |`

That spelling — `W-<NAME>-<N>` — is **also** the row-id grammar of the
whitebox provenance ledger, `docs/whitebox/DISCLOSURE.md`, which at
`bce2bfc68` holds 22 rows (`W-MOP-1`, `W-ALIAS-2`, `W-GLATTRS-1`, …). So a
reader who greps a bare `W-UNW-1` expecting to re-check a *provenance* claim
lands on fixtures and a codegen rung instead, and a reader who greps a bare
`W-MOP-1` cannot tell from the token alone which registry answers.

**Neither registry renames, and the fix is at the citation:**

- **A provenance citation must be qualified — `DISCLOSURE W-MOP-1`.** A bare
  `W-*-N` is a **rung tag** by default. `scripts/prose_audit.py`'s C1 check is
  built on exactly this reading: it fires only on a token *attributed* to the
  ledger and missing from it, so an unqualified rung tag is correctly silent.
- **Rung tags stay bare and stay as they are.** `W-UNW-1` has 37 citations in
  18 files at `bce2bfc68`, every one correct as written; renaming would touch
  all of them to fix a collision that only bites an unqualified grep. Decision
  19 rules that out in those words.
- **Tag uniqueness is enforced only *within* this registry.**
  `crates/c2-harness/tests/rung_registry.rs` asserts no two rung docs declare
  the same `Tag:`. It knows nothing about `DISCLOSURE.md`, and nothing
  cross-checks the two — which is why the rule above is a convention about
  *citations* and not an assertion about *names*.

When allocating a new tag, prefer the plain `W<N>` form. A hyphenated one is
legal and `W-UNW-1` keeps its own, but every new hyphenated tag is one more
token a provenance grep cannot disambiguate.

## Lane kinds

Adopted 2026-08-13 from `docs/STRATEGY_REVIEW_2026-08-13.md` §4 lever 1: the
fixture-claim rung was the only unit of work the repo maintained, and that unit
cannot carry any of `CEILING.md` §6.1's seven phases — which is why five of
them never had a building rung. Three kinds are now first-class. The registry
test already admits all three: kinds 2 and 3 use the `Fixtures: none — <reason>`
+ `Census: … +0` path (`rung_registry.rs`, the instrument-rung exception).

1. **Fixture-claim rung** (the default, everything above): names fixtures,
   claims a prefix, moves the census. The unit for TU-shaped work.
2. **Construct rung** (precedent: board **#290**, item B of `CFG_SHAPE.md`
   §6.2): builds shared machinery — an IR type, a pass, a gate predicate — by
   re-expressing **already-byte-exact** classes through it. `Fixtures: none —
   construct rung: <what it builds>`; `Census: +0`; success criterion is a
   **required-zero byte delta**, graded by a line-for-line identity diff of
   per-lane gate counts before/after. A construct rung that changed any byte
   FAILED, whatever else it did.

   > **THE DECISION-SURFACE CLAUSE (added 2026-08-22 from the owner's goal
   > re-ranking — `GOAL_DECISION_2026-08-21.md` § "AMENDED",
   > `ROADMAP_SLICING_2026-08-21.md` §6 rule 7).** A construct rung is the
   > lane kind that builds **general layers**, so this rule lands here: from S1
   > on, a general layer ships its arbitrary choices — allocation order,
   > scheduling tie-breaks, label counters — as **named, enumerable parameters
   > whose DEFAULT reproduces c2 byte-exactly**, not as baked constants.
   > **This does not relax the required-zero criterion; it is graded at the
   > default and nowhere else** — every non-default configuration is a legal
   > *instrument* state and licenses no emit. The reason is that a baked
   > constant serves goal (2) only, while a named decision point serves goal
   > (2), the permuter and the training pipeline at the same correctness cost,
   > and it is what turns a close-but-wrong mismatch from *opaque* into
   > *searchable*. **A rung that bakes a fitted constant owes a pointer to the
   > read that would replace it** (`whitebox/READ_PLAN_2026-08-21.md` §2 is the
   > index).

   **THE COST CLAUSE (added 2026-08-21, board #3336).** A required-zero
   **byte** delta is silent about a required-zero **cost** delta, and the
   criterion above cannot express throughput at all: nothing in
   `scripts/gate.sh`, `scripts/lanes.txt`, the sweep, the cross or the debug
   lane reads a timing, and `c2rs perf` is **reported, never gated**. Measured
   on `ir0`: with all eleven production call sites switched to the new framing
   the gate table was **identical line for line** — 18/18 PASS, 6,948 graded,
   `mismatch 0`, all 395 pre-existing `gap-metric` keys unchanged — while the
   change cost port throughput. A criterion that could not have failed did not
   pass; it abstained. **So a construct rung MUST name at least one axis on
   which it can fail even when every byte is identical**, in its rung header,
   before it starts. Throughput is the obvious one for anything on
   `PortC2::build`'s path — the port's whole thesis is verifier throughput, so
   cost is not a secondary axis for this project — but it is not the only one:
   a re-expression can also move a *denominator*, a *binding*, a *precedence*,
   or the *coverage weight* of the corpus that runs the fence (#3333). Name
   the axis, say how it would be observed, and say what you measured. "The
   bytes were identical" is the floor, not the grade.

   > **AMENDED 2026-08-21 — the cost clause SURVIVES; its *reason* does not.**
   > *"The port's whole thesis is verifier throughput"* was true when #3336 was
   > written and is **retired** (`docs/GOAL_DECISION_2026-08-21.md`). Throughput
   > is a property, not the goal, so it is no longer *automatically* the axis a
   > construct rung on `PortC2::build`'s path must name. **What survives
   > untouched is the rule itself**: a required-zero byte delta is silent about
   > everything that is not a byte, and a criterion that cannot fail abstains
   > rather than passes. Name an axis on which the rung **can** fail — a
   > denominator, a binding, a precedence, a coverage weight, a cost — and say
   > what you measured on it. #3336's measurement stands; only its
   > *"cost is not a secondary axis for this project"* clause loses its warrant.

   **Corollary, same row:** if the re-expression half of a construct rung is
   reverted, the rung's own grading instrument is disarmed — the identity diff
   becomes a tautology over a purely additive tree with no production caller.
   A rung in that state says so and names the criterion that CAN fail (`ir0`
   used four executed mutations of the framer with distinct signatures).

   > **THE REFUSAL-DOMAIN CLAUSE (added 2026-08-28, board #3723/#3743, lane
   > `w-doctrine`) — AND IT IS ENFORCED, NOT ADVISED.** The cost clause above
   > says a byte delta is silent about cost. `#3723` measured something worse:
   > **the byte delta is silent about a real widening of the EMIT itself.**
   > `w-regsel`'s control C6 opened the caller's allowed register set from the
   > volatiles to `r0..r31`, so c2's callee-saved tail became reachable from a
   > production path — and **471 of 475 crate tests still passed, no encoder
   > row moved, `GATE: PASS` at both ends, and the identity diff read 0 lines
   > over 21 rows.** The widening would have shipped.
   >
   > The reason is not fixable inside the criterion. **The gate can only see
   > emissions the corpus EXERCISES**, and a widening whose new emissions are
   > unexercised is invisible to it — `#1236`'s shape, a guard green precisely
   > because the offender is out of scope. **The required-zero byte delta stays
   > necessary and is now known not to be sufficient**, and `#290`'s pattern is
   > not wrong so much as under-specified for this class.
   >
   > So, for a construct rung over an **allowed set, a candidate set, or any
   > refusal boundary**:
   >
   > * **The surface goes in `c2_core::surface::SURFACES`**, with a domain that
   >   runs **past what any fixture reaches** — that is the whole mechanism, and
   >   a domain that stops where the corpus stops reproduces the defect. The
   >   rendered whole is committed as `crates/c2-core/src/surface/DOMAIN.txt`
   >   and four `cargo test` assertions grade it: the live domain equals the
   >   baseline; the source markers and the registry are a bijection; every
   >   surface meets a cell and a refusal floor; and every boundary-named
   >   `const` in the crate is either covered by a surface or listed in
   >   `UNCOVERED` under a ratchet. **A widening then has exactly one way
   >   forward — re-bless the baseline — which puts it in the diff as text
   >   somebody can read.** It is not stopped, it is made impossible to make by
   >   accident.
   > * **The rung header carries `Fail axis:`**, non-empty, and
   >   `rung_registry.rs` asserts it for every construct rung dated 2026-08-28
   >   or later. Earlier records stay exactly as written.
   >
   > **What this does NOT do, said here so it is not discovered later.** The
   > registry covers what somebody registered; `UNCOVERED` is a hole with a
   > ratchet on it, not a closed hole, and its own list carries two false
   > positives of the name screen that finds it. The header field checks
   > **presence, not measurement** — it cannot tell a named axis from a
   > measured one. And none of it is in `gate.sh`'s verdict or licenses any
   > emit (`docs/FUNCTION_BYTE_MATCH.md` §0): the sole judge stays real
   > `c2.dll` under wibo plus a byte-exact obj compare.
3. **Characterization lane** (precedent: `2026-08-13-wb-live.md`): reads real
   c2's behavior — whitebox addresses plus obj-grid confirmation — and lands
   findings, not code. `Fixtures: none — characterization: <the question>`;
   `Census: +0`; prereg frozen before the first probe; every load-bearing
   claim cites an address or a grid cell; disassembly-derived adoption rules
   (`docs/whitebox/DISCLOSURE.md`) apply unchanged.

   > **PROMOTED 2026-08-21 — a characterization lane's output IS a
   > deliverable, and predicted reach 0 is not a mark against it.**
   > *`docs/GOAL_DECISION_2026-08-21.md`; owner.* Goal (1) is *"perfect
   > reproduction that gives us a clear understanding of the MSVC internals, to
   > help us with decomp"* — so **the understanding is the product**, and
   > `docs/whitebox/` is product rather than provenance overhead (which
   > `CLAUDE.md` § "Whitebox analysis is AUTHORIZED" had already decided on
   > 2026-08-17 for independent reasons; the two agree). Concretely:
   >
   > * **A characterization lane owes no conversion story.** It does not have
   >   to argue that some later rung will convert a TU. `w-dagorder` was
   >   dispatched with *"predicted reach 0, registered as such"* and had to
   >   justify itself as *"phase machinery Option A requires"*
   >   (`STRATEGY_REVIEW_2026-08-13.md` §8.2); under goal (1) that second
   >   sentence is no longer owed.
   > * **`Census: +0` and reach 0 stay REGISTERED, and that does not change.**
   >   Preregistering a zero is what keeps the lane honest and is the opposite
   >   of an excuse — the promotion is about what the zero *costs the lane in
   >   the ranking*, never about relaxing prereg.
   > * **`built` still has to be earned.** The outcome word is for a lane that
   >   landed *what it preregistered*. A characterization lane whose findings
   >   did not confirm says `FAILED` in that word, exactly as before.
   > * **The judge is untouched.** Understanding c2's internals is **not** a
   >   licence to grade the port against c2's internal state; the byte compare
   >   against real c2 remains the sole judge, and the stage oracle's standing
   >   bound (`docs/STEP5_PRICING_2026-08-21.md`, stageoracle §8) is what keeps
   >   snapshot equality out of it.

   > **HOW THESE LANES ARE CHOSEN CHANGED 2026-08-22 — THE PROMOTION ABOVE SAYS
   > A CHARACTERIZATION LANE IS WORTH RUNNING; THIS SAYS WHICH ONE TO RUN AND
   > IN WHICH ORDER.** *`docs/WHITEBOX_LEVERAGE_2026-08-21.md` §1 and
   > `docs/ROADMAP_SLICING_2026-08-21.md` §6 rule 6, with the owner's goal
   > re-ranking; enumerated targets in
   > `docs/whitebox/READ_PLAN_2026-08-21.md` §3. Propagated by lane
   > `w-readdocs`.*
   >
   > * **READ BEFORE PROBE, and it is a dispatch precondition, not advice.**
   >   Before a lane budgets a probe grid or a fitted-parameter search it must
   >   price the binary read that would answer the same question — locate the
   >   function, read it, confirm with a *small* probe — and prefer the read
   >   unless the read is measurably more expensive. **A probe-grid lane on any
   >   of R1–R9's nine subjects must say in its prereg why it is not the read.**
   > * **The read-plan is the target list.** `READ_PLAN` §3 ranks nine reads by
   >   *(priced black-box cost replaced) / (measured read price)* — explicitly
   >   **not** by size or proximity, which is the family MEMORY records as
   >   *"ranking instruments measure themselves"* at five instances. Dispatch
   >   order is **R1 → R2 → R3**, with R5 gated on R2 proving the arm-reading
   >   method on 79 bounded bodies before it bets on 189.
   > * **The probe does not disappear; it changes ROLE.** It stops being the
   >   *discovery* instrument and becomes the *confirmation* instrument: read
   >   the mechanism, predict the observable, confirm with the smallest grid
   >   that could refute the reading. This is why `[R]` is not a finding —
   >   by `docs/whitebox/ref/README.md:49`'s own legend it says *"the
   >   instructions were read correctly"*, not *"this is what c2 does"*, and
   >   the `.bss` bump rule was read correctly and was wrong about c2. **Every
   >   read lane still ends in a confirmation probe**, and `DISCLOSURE.md`'s
   >   adoption rule is unchanged.
   > * **A fitted constant is a debt with a named creditor.** `READ_PLAN` §2
   >   indexes every fitted constant in `crates/` against the read that would
   >   replace it. A lane that ships a new one owes that pointer.
   > * **CHECK THE BOARD, AND CHECK THE TREE, BEFORE PRICING A NEW
   >   INSTRUMENT.** Instance N of the standing pattern, 2026-08-22: a
   >   2026-08-21 planning doc named "mismatch anatomy" as a missing instrument
   >   and priced it at 1–2 wk; it had shipped on 2026-08-06 as
   >   `crates/c2-harness/src/gap/fndiff.rs` / `docs/DIFF_STRUCTURE.md`, runs
   >   unconditionally on every scan, and **its own output refutes the table the
   >   doc proposed** (0 pure reorderings, 2 field-only words in 5,189). Board
   >   **#3369**. The cost of the check is one grep.

## Outcome, one word

Every rung doc's header carries an `Outcome:` line (new docs; the 209 landed
before 2026-08-13 are not backfilled). Exactly one of:

- `converted` — TU `match` moved.
- `declined` — a priced refusal; the decline and its price are the deliverable.
- `instrument` — built or corrected a measuring instrument.
- `built` — a construct rung or characterization lane that landed what it
  preregistered (zero-delta held / findings confirmed).
- `FAILED` — none of the above. Stated in that word. A lane that neither
  converts, declines, prices, nor builds is not "a compound finding" — before
  this field existed, a failed lane was indistinguishable in the record from a
  successful one at the level of artifacts produced (STRATEGY_REVIEW §2 H3).

The merge funnel checks the field is present and matches the headline before
authoring `work/merge-<lane>.txt`.

## Two rules a probe must satisfy — added 2026-08-17

Both were derived independently by two lanes in one wave (`w-fence163` board
**#3219**, `w-mutcensus` **#3231**), each catching the defect **in the
flattering direction** in its own instrument, before publishing an affected
result. They bind any lane that runs a campaign of measurements.

**1. Carry a control whose failing set is pinned by NAME, and re-run it in
every environment the campaign uses.** A control pinned by *count* passes in an
unprovisioned worktree the moment the count matches. The concrete failure: a
fresh `git worktree add` has no `compilers/` (gitignored by design, and it does
not follow a new worktree), so every capture-based test **skips**. The
red-maker then reports *"3 passed"* in 0.00 s, and **cargo swallows the SKIP
line for a passing test** — so a mutant that should be RED reads GREEN with a
clean suite, the right target count, and the right exit code. `w-mutcensus`'s
variant is worse still: its registered baseline was **byte-identical with and
without a toolchain** (1,648/0/42 either way, differential 84.17 s vs 0.00 s),
so the prereg's own probe definition *and* its `targets != 42` invalidation
rule were **both blind**. It surfaced as a **contradiction between two runs of
one mutant** — never by inspection. Assert the *executed-test count* and the
differential's *duration*, not the exit code. A colour taken in an unvalidated
environment is **void, not provisional**: discard it, re-run it, and keep the
invalid log.

**2. Derive the results table from the logs; never accumulate it.** This is
what let `w-mutcensus` reapply three classifier corrections retroactively
across a 159-run campaign. An accumulated table cannot be re-derived when the
classifier turns out to be wrong, and the classifier **was** wrong three times.

## Board numbers are allocated by the coordinator, not read from the pointer

Added 2026-08-17, on evidence from the same wave: **row-by-row verification is
the strongest check a lane can run and is still structurally insufficient.**
`w-fence163` and `w-gatewire` both minted `#3218`–`#3221`, each having verified
row-by-row against a master where neither had landed. `w-mutcensus` verified
`#3218`–`#3230` against `BOARD.md`, both peer branches, **and** every `#32NN`
citation in `ROADMAP.md`, concluded `#3222`, and was still wrong — the
verification was not at fault. Two lanes on separate branches are blind to each
other's mints by construction, and `board_audit.sh` counts duplicates only
*after* both merge.

So: **a lane with peers in flight leaves its rows UNNUMBERED and says so, or
takes a block the coordinator allocated explicitly.** The next-free pointer in
`BOARD.md` is a hint, not an authority — it is routinely already wrong at the
tip that prints it, because unmerged branches hold blocks it cannot see.

## Standing facts every lane brief must carry — updated 2026-08-18

These are the things lanes have most often been briefed *wrongly* about. A
coordinator dispatching a lane copies this block; a lane that finds one of
them stale reports it as a **dispatch defect**, not a preamble (`w-dagorder`'s
pattern — it was sent at a lever discharged the same day the review naming it
was written, and **detection was incidental**, so the defect's true rate is
unmeasured).

* **THE GOAL IS PERFECT REPRODUCTION, DECIDED BY THE OWNER 2026-08-21**
  (`docs/GOAL_DECISION_2026-08-21.md`; `CLAUDE.md` § "The goal"), for two ends
  ~~ranked equally~~ **RANKED, in that doc's § "AMENDED" the same day** —
  **(1) a clear understanding of MSVC's internals in service of decomp**,
  which is **PRIMARY**, and **(2) parity, a 100 % open-source implementation**,
  which stays a real end and is **additionally instrumental to (1)**: the port
  is an executable, tweakable model of c2 that emits signals about compiler
  state the opaque binary cannot be made to emit. The
  **verifier-throughput thesis is RETIRED**; throughput is a property that may
  neither justify a lane nor forbid one, and the 2026-08-13 NO-GO's economics
  are **superseded, not satisfied**. Two things a brief must get right as a
  result: **characterization output is a deliverable** (predicted reach 0 is
  not a mark against such a lane), and **`match` → 870/878 is the scoreboard**
  for goal (2), so partial coverage does not pay in proportion. A brief that
  ranks or declines a lane on throughput grounds is a **dispatch defect**.
  **Copying the words "ranked equally" into a brief is now also one** — the
  clause was live for a matter of hours and this bullet carried it for a day
  (found by lane `w-readdocs`, 2026-08-22).
* **TWO STANDING RULES FOLLOW FROM THE RANKING, AND A BRIEF THAT OMITS THEM IS
  A DISPATCH DEFECT** (`docs/WHITEBOX_LEVERAGE_2026-08-21.md`;
  `docs/ROADMAP_SLICING_2026-08-21.md` §6 rules 6 and 7; `CLAUDE.md`).
  **(1) READ BEFORE PROBE** — before any lane budgets a probe grid or a
  fitted-parameter search, price the whitebox read that answers the same
  question and prefer it. The enumerated targets are
  `docs/whitebox/READ_PLAN_2026-08-21.md` §3 (R1–R9); a probe-grid lane on any
  of those nine subjects must say why it is not the read. **Item F's
  13-raw/65-calibrated and STEP5's I1/I2 eng-month prices are BLACK-BOX
  numbers** and may not be quoted as the cost of the facts.
  **(2) EXPOSE THE DECISION SURFACE** — every general layer from S1 on ships
  its arbitrary choices (allocation order, scheduling tie-breaks, label
  counters) as **named, enumerable parameters whose DEFAULT reproduces c2
  byte-exactly**, not as baked constants. A baked constant serves parity only;
  a named decision point serves parity, the permuter and the training pipeline
  at the same correctness cost. **The judge is untouched by both rules**: the
  default configuration is what the byte compare grades, and reading c2's
  internals is not a licence to grade the port against c2's internal state.
* **Run the gate as `scripts/gate.sh --jobs 16 --require-graded` — ~80 s.**
  `--jobs 4` is ~153 s and is not the safer choice; the box was never
  CPU-starved. A lane's **first** gate in a fresh worktree still pays a
  ~1,261 s cold `mode_cross` leg (`w-gateperf` §11.1, board **#3266**) — that
  is a property of the worktree, not of the change.
* **Run suites as `C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release
  --no-fail-fast`.** Armed 2026-08-18. Without it, a fresh worktree with no
  `compilers/` skips every capture-based test **silently** — cargo swallows
  the `SKIP` line for a passing test — so a registered RED reads GREEN with a
  clean suite, the right target count and the right exit code (#3219, #3231).
  The two runs are otherwise **byte-identical**; only the durations differ.
* **DO NOT "assert 0 occurrences of `SKIP: toolchain absent`" — THAT CHECK IS
  VACUOUS AND IT IS SATISFIED BY THE FAILURE IT CLAIMS TO DETECT** (#3341,
  measured 2026-08-20). libtest swallows stdout for a **passing** test, and a
  skipping test *passes*; **136 test sites can emit that line and a passing
  suite log contains 0 of them either way.** Measured: 2 tests skipped, **0**
  SKIP lines captured, **2** under `--nocapture`. The bullet above already
  states this mechanism, and a coordinator who had just read it still wrote the
  vacuous form into four lane briefs and quoted its 0 as evidence on three
  merges — so the wrong check is named here explicitly rather than left as an
  inference. **Assert a DURATION instead** — `census_gate` reads **0.00 s**
  without a toolchain and **tens of seconds** with one — or run with
  `--nocapture` and count, which is the form that actually distinguishes the
  two (a provisioned run emits **0** SKIP lines because nothing skips; an
  unprovisioned one emits one per skipping test). **Do NOT hardcode a duration
  BAND**: the "~90-125 s" first written here was measured on a box under
  external load of 88-153, and the same target read **74.62 s** at load 15 two
  days later. The signal is `0.00` vs `not 0.00`, and a band tight enough to
  look rigorous will red a quiet box. The general rule the brief got right:
  **assert the environment, never the exit code.**
* **Count `gap-metric` keys with `grep -cE '^ *gap-metric \S+ \S+$'`.** Never
  `grep -c 'gap-metric'`, which over-counts: prose lines merely *mention* keys.
  **Three consecutive lanes hit this**, and the third *explained* the +2 by
  attributing it to a peer's merge (#3269). Hence the general rule: **a lane
  that finds an unexpected delta owes a measurement before it owes a cause** —
  a wrong count invites re-counting, a wrong cause removes the reason to check.
  **Measure the count yourself at your own base; do NOT carry it from a
  brief.** It read **394** on 2026-08-18 and **395** on 2026-08-19 at master
  `1165839fe`, with no commit in between able to add a key — a *measurement*,
  offered with **no cause**, and the workload moved in the same interval.
* **`fnbyte-*` is not a pure function of the commit** (#3249). It reads
  (commit × capture-cache state × untracked workload). Re-read base and tip
  **back to back**; state the cache state and `dc3-decomp` head; treat any
  effect under ~10 bodies as **unattributable rather than reporting it**.
* **Assert the two ends read the SAME workload stamp** (#3306) — `c2rs gap`
  prints `workload <sha> (clean) <path>` on every run, so the check is a
  `diff` of two strings the scan already emits. **RE-READ THE STAMP AT LANE
  START, assert EQUALITY between your own two ends, and never carry a stamp
  VALUE from a brief** — a pinned stamp pins a fact with a half-life of hours
  (#3311, the third instance). On 2026-08-19 alone
  the sibling `dc3-decomp` tree passed through **three** stamps —
  `897d0220fd1d` → `49ad7cfd5d26` → `eda64e956c87` — and a coordinator handed
  the middle one to a lane that measured the third. Anything derived from it
  moves with it: `fnbyte-exact` read 35,886 and 35,956 across that last step.
  Measured cost of skipping it:
  **82 of 394 keys** moved between one lane's two ends, 45 minutes apart, on
  the same commit pair, binary and machine, because a merge landed in the
  sibling `dc3-decomp` tree mid-campaign. #3249's "under ~10 bodies is
  unattributable" is calibrated for **noise** and does not cover this — 21 %
  of the key surface, from a repository the lane never touched. Re-reading
  the base at the *current* stamp gave 0 of 394 differing. The failure is
  silent, arrives with a plausible story attached, and gets **worse the
  longer the lane runs**.
* **`fnbyte-*` denominators are 71.2 % bodies the shipped image never
  contains** (#3254) — `/Gy` COMDATs the linker discards. A `fnbyte` ratio is
  progress over *what c2 emits*, which is the port's job; it is never progress
  over *the game*.
* **The per-symbol `fnbyte-differs` set compare is void** for any lane that
  changes the admitted population (#3237). Use a name-stable per-TU shape
  multiset.
* **"Graded tree identical at both ends" applies only to revert-everything
  lanes** (#3215) — any lane landing a test moves it by construction. **Do not
  compare release-binary sha256 across worktrees** (#3224): `CARGO_MANIFEST_DIR`
  is compiled in, so that comparison is void by construction.
* **A phase's size comes from a SCAN, never from `CEILING.md` §6.1's prose.**
  Added 2026-08-19 after a coordinator dispatched phase 1 on that table's
  *"`cfg-reach-shipped` 2 of `cfg-reach-top` 16 — 14 of 16 frontier TUs held
  by CFG class alone"*. A scan reads **0 of 2**, `frontier` **2**, and
  `frontier-codegen-reader` **22** with `-refused` **0** — so the phase
  converts **zero** and the distance is the *reader's*, not the emitter's.
  The row was not wrong when written; **the frontier converted underneath it**
  (`match` 11 → 26). `CLAUDE.md` already says to read `STATUS.md` before
  quoting `ROADMAP.md`; the same applies to **every** long-form doc, `CEILING.md`
  included — `STATUS.md`'s *generated* block already read `FRONTIER 2`.
  Before dispatching phase work: run the scan, quote its keys, and give the
  lane the key NAMES so it can re-derive them.
* **"Unserved in `docs/`" is not "unserved in the repo"** (#3314). A capability
  a lane was dispatched to characterize as missing was already shipping in
  `crates/` as KNOWN-ANSWER-0 gap keys on every gate invocation, invisible to
  any `docs/` grep. A coverage check reads `crates/` too.
* **Check the branch name is free** before briefing one: `git branch --list`.
  A 2026-08-08 lane still held `wt-w-cfgclass`, so the 2026-08-19 lane of that
  name could not create its worktree and landed on `wt-w-cfgclass2`.
* **Board blocks are allocated by the coordinator** — see the section above.

## A metric delta of zero is not evidence of correctness — added 2026-08-18

`w-sizebracket` (board **#3270**–**#3275**), and it is the sharpest instrument
finding this project has:

> **A predicate can be 39.6 % wrong about c2 and *free* in the metric used to
> choose it.** `fnbyte-exact Δ = 0` is evidence about **reach** — how much the
> predicate touches — and **never about correctness**.

`w-dataseam` found a size constant by sweeping and validated it out-of-sample
on a deterministic odd/even split: **both halves at zero cost, effect sizes
agreeing to one body.** That reads as strong replication and **it is blind by
construction — both halves share the blind spot**, because the split
resamples the *population* while the error lives in the *predicate*. Scoring
the same constant against real `c2` on **7,667 workload call edges** found it
wrong on **3,037**, of which **3,036 were in the unsound direction**, at a
metric delta of exactly **0**.

**So: a predicate is priced against the oracle, on the population it will
apply to, or it is not priced.** A zero-cost sweep, a split-half agreement,
and a required-zero identity diff are all compatible with a rule that is
wrong about c2 four times in ten. This is the same family as the wrong emit
that survived 255 commits of green gates — the instrument could not generate
the shape that would expose it — and it is why `mismatch 0` is never
evidence of correctness (`STATUS.md`'s standing trap).

Corollary, from the same lane: **a null result inside a dispatched window is
not a refutation of the window.** It found `[176, 232]` empty only because its
probe families ladder from `n = 0`; a lane probing only the window it was
handed would have reported *"no flip in range"* — a null that reads as a
measurement problem rather than as the window being in the wrong unit
entirely.

## What is here, and what is not

The historical rungs live in `docs/ROADMAP.md` §6a–§6m and in the per-subject
`docs/*.md` write-ups, and they **stay there** — history does not conflict, only
growth does (§9.6: "freeze and fork, don't migrate"). The files here are the
rungs that already had a `§6`-letter section, restated as a claim of their tag
and fixtures with a pointer to the authoritative record; the remaining
prefixes in `fixtures/cpp` (`w5`, `w6`, `w10`, `w12`, `w13`, `w13b`, `w14`,
`w15`, `w16`, `w17`, `w18`, `w19`, `w20`, `w21`, `w23`, `wfr`) are not yet
claimed here. Backfilling them is `ARCHITECTURE_SEAMS.md` §6 step 4; until
then the registry test asserts uniqueness over what *is* claimed and prints
the unclaimed prefixes rather than pretending to full coverage.
