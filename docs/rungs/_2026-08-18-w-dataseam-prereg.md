# PREREG — w-dataseam (frozen BEFORE any probe, first commit of branch `w-dataseam`)

    Lane:      w-dataseam
    Kind:      construct
    Base:      master `44794fa4`
    Date:      2026-08-18
    Mission:   execute `docs/rungs/2026-08-17-fence163.md` §8.1's phase seed —
               LIFT the `data_syms` scoping off the EH-state inline fence so the
               predicate becomes a clause of the INLINE MODEL rather than of the
               string-literal admission; price it two-sided; pay it together
               with clause (c); measure realized reach rather than estimating it
               from a blocker row.

## 0. What is frozen here, and what is not

Frozen: everything below — the restated seed, the variant ladder, the
predictions with probabilities, the ceilings (**no discount factor anywhere**),
the decision rule, the mutant colours, and the registered bias. Not frozen: the
capture grids, which do not exist yet; any grid built later is recorded as
**measurement** in the rung doc with its content hash at the commit that adds
it, and scores nothing here (`w-section` §2's rule — registering an
already-taken measurement as a prediction is scoring a coin after it landed).

**Seams.** This lane owns `crates/c2-il` at the `data_syms` scoping /
`bind.rs` / `census.rs` region and whatever `c2-core` codegen the lift requires.
It does **NOT** touch `crates/c2-harness/src/gap/tests.rs` or
`crates/c2-harness/tests/` (peer `w-calleeguard`); if a guard is needed there it
is **described in the rung doc and routed**, not written. It does not touch
`docs/whitebox/` (peer `w-dagorder`). `crates/c2-core/src/coff/` is unoccupied
this wave; entering it would be announced loudly and is **not** expected.

**Board rows `#3234`–`#3238` were allocated by the coordinator.** The next-free
pointer in `BOARD.md` is not consulted (`docs/rungs/README.md`, "Board numbers
are allocated by the coordinator").

## 1. The seed, restated from THIS lane's own reading of the tree at `44794fa4`

Read before any measurement, cited by file and line so a wrong restatement is
falsifiable:

| the thing §8.1 names | what it actually is at `44794fa4` | site |
|---|---|---|
| "the `data_syms` scoping" | the first conjunct of census clause **(c2)**: `f.data_syms.iter().any(|d| d.starts_with(STRLIT_NARROW_PREFIX))`, i.e. the whole EH-state fence is asked **only of bodies that materialize a `??_C@_0…` address** | `crates/c2-il/src/func/census.rs:1474-1477` |
| the predicate the scoping bounds | ∃ callee `c` : `c` is named by the **emit binding** ground map (`strlit_ground`) ∧ `c ∉ tu_modelled_callees` ∧ `cflow_key(segs[j]).1 != "eh-state1"` ⇒ **REFUSE** (`DATA_SYM_STRLIT_FENCED`) | `census.rs:1478-1511` |
| "clause (c)" | census clause **(c)** `CALLEE_DEFINED_IN_TU` — `callee_defined_here(&f, defined)` ∧ `callee_defined_here_unmodelled(&f, defined, empty_here)` ⇒ REFUSE. Its ground set is `bind::defined_name_set` = `gl_defined_names(gl).0`, the whole-TU **fail-open** walk that binds nothing on 845 of 871 workload TUs | `census.rs:1400-1421`; `bind.rs:1152-1210` |
| the narrow-prefix constant | `STRLIT_NARROW_PREFIX = "??_C@_0"` | `bind.rs:516` |
| the admission the scoping rides on | `resolve_data` admits a `??_C@_0…` name | `bind.rs:930` |
| the whole-TU refusal that keeps the yield byte-credit-only | `IlBundle::functions` refuses a TU whose admitted body references a `??_C@_0` sym | `crates/c2-il/src/func/bundle.rs` |

**Two facts about the ordering, which the seed does not state and which this
lane asserts as its own reading.** (i) Clause (c) is matched **before** clause
(c2), so on a walk-visible TU clause (c) answers first and (c2) is unreachable
(fence163 §6.1 measured this). (ii) The two clauses use **different ground
sets** — (c) uses the blind walk, (c2) uses the emit binding — so the "lift" is
not only the removal of a conjunct: it is also a **ground-set widening**, and
that is where the cost lives. If §8.1 is under-determined anywhere it is here,
and this lane records it as a finding rather than papering over it.

**Consequence, registered as this lane's own claim before measuring:** the naive
lift (delete the `data_syms` conjunct, change nothing else) is a **pure
widening of refusals** on 845 of 871 TUs and cannot admit a single new body,
because the relaxing half of the rule (`eh-state1` ⇒ admit) lives in a clause
that only ever fires *after* clause (c) has already refused. Paying clause (c) —
§8.1 item 4 — is therefore not optional garnish; **it is the only half of the
lift that can pay anything.**

## 2. The variant ladder to be measured (counterfactual builds, each scanned)

Each variant is a build of the 878-TU workload scan
(`c2rs gap --list work/dc3-workload/files.txt --flags-file …`), logs kept under
`work/w-dataseam/`, tables **derived from the logs and never accumulated**
(`docs/rungs/README.md` probe rule 2).

| id | change | what it tests |
|---|---|---|
| **B** | none (`44794fa4`) | P1, the base |
| **V1** | delete the `data_syms` conjunct from clause (c2). Nothing else. | §8.1 item 1 **to the letter** — the cost side alone |
| **V2** | V1 + the refusal additionally requires the callee's `.gl` `FN_FLAG_INLINABLE` bit (`gl::FN_FLAG_INLINABLE`, `0x40`) to be SET | whether the input-side inlinability bit the container already carries removes the over-refusal V1 buys |
| **V3** | V2 + clause **(c)** yields to the same two exemptions (callee `eh-state1`, or callee not inlinable) — i.e. **one** rule replaces (c) and (c2), keyed on the union ground set | §8.1 item 4 — the lift **paid together with clause (c)**; the only variant that can move anything in the paying direction |

If a variant's price is unfavourable under §3's decision rule it is reported as
a **measured decline**, with its numbers, not silently dropped.

## 3. Predictions — probability form, ceilings with NO discount factor

| id | prediction | P |
|---|---|---:|
| **P1** | the base scan re-measured in THIS worktree at `44794fa4` reproduces the dispatch figures exactly: `match` 26 · `mismatch` 0 · `codegen-gap` 0 · `vocab-gap` 844 · `capture-fail` 8 · `fnbyte-exact` 35,897 · `fnbyte-refused-parse` 113,449 · **394** anchored `gap-metric` keys | 0.90 |
| **P2** | **V1 is a net LOSS in the goal's units**: `fnbyte-exact` falls, i.e. Δ ≤ −1 against B | 0.80 |
| **P3** | V1's loss is **large, not marginal**: `fnbyte-exact` Δ ≤ −100 | 0.55 |
| **P4** | V1 admits **zero** new bodies — `fnbyte-refused-parse` moves by exactly `−(fnbyte-exact Δ + fnbyte-differs Δ)` with no term of the opposite sign, i.e. §1's "pure widening of refusals" claim holds | 0.75 |
| **P5** | the `FN_FLAG_INLINABLE` bit is **readable for the callee** at the census's clause site (an attrs map keyed by `EmitBinding::name` is already built there for `f.inlinable`) and V2 therefore builds without a new reader | 0.70 |
| **P6** | **V2 recovers most of V1's loss**: `fnbyte-exact`(V2) − `fnbyte-exact`(V1) ≥ 0.5 × (`fnbyte-exact`(B) − `fnbyte-exact`(V1)) | 0.50 |
| **P7** | **V3's paying half is non-empty**: relaxing clause (c) to yield to an `eh-state1` or non-inlinable callee newly admits ≥ 1 body (`CALLEE_DEFINED_IN_TU` count falls by ≥ 1). Ceiling, **no discount**: the full base count of `callee-defined-in-tu:eof` rows | 0.55 |
| **P8** | **the paying half pays in the GOAL's units too**: V3's newly-admitted bodies contribute `fnbyte-exact` ≥ +1 (not merely a census key movement). Ceiling, no discount: the same full base count | 0.35 |
| **P9** | **the whole lift is net-negative and the honest outcome is `declined`**: no variant achieves `fnbyte-exact` Δ ≥ 0 together with 0 new `fnbyte-differs` symbols | 0.55 |
| **P10** | `match` **26** and `mismatch` **0** and `codegen-gap` **0** in **every** variant measured, including any shipped tip | 0.92 |
| **P11** | whatever ships (possibly nothing) introduces **0 new `fnbyte-differs` symbols**, checked as a per-symbol SET compare over two `--fnbyte-diff-jsonl` dumps, never by subtracting totals | 0.85 |
| **P12** | identity control: reverting this lane's `crates/` change reproduces the base scan at Δ0 over all anchored `gap-metric` keys | 0.95 |
| **P13** | `scripts/gate.sh --jobs 4 --require-graded` PASS at both ends, and the **`debug-lane`** row (4th, debug-profile) reads `0 panic` at both ends and is asserted to be INSIDE the compared identity-diff range | 0.85 |
| **P14** | `cargo test --workspace --release --no-fail-fast` at tip: **0 failed**, targets ≥ 43, passed = 1,660 + (tests this lane adds) − (tests a shipped behaviour change legitimately retires, reported by name) | 0.80 |
| **P15** | **the reach is not estimable from the blocker row**: the realized count of bodies whose verdict changes under V1 differs from the base `callee-defined-in-tu:eof` blocker count by more than 2× in either direction | 0.60 |

### Decision rule (frozen)

Ship a variant **iff** all of:

1. it is a **decidable pre-emission predicate** in `CFG_SHAPE.md` §6.3 rule 1's
   sense — stated over container facts (emit binding, `cflow_key`, `.gl`
   attribute byte, the modelled set), with **no name list, no population list,
   and no `data_syms` / string-literal term**; and
2. `fnbyte-exact` Δ ≥ 0 against B; and
3. **0 new `fnbyte-differs` symbols**, per-symbol set compare; and
4. `match` = 26, `mismatch` = 0, `codegen-gap` = 0; and
5. `gate.sh --require-graded` PASS with a line-for-line per-lane gate-count
   identity diff (**not** a release-binary sha256 — board **#3224** voids that
   comparison across worktrees).

**Any `mismatch` ≥ 1 anywhere — gate, sweep, cross, debug-lane, or the 878-TU
scan — reverts the `crates/` change and the Outcome becomes `declined`, with
the measurement as the deliverable.**

**If what can be built is another named exemption** — i.e. the predicate needs a
`data_syms`-shaped, name-shaped or population-shaped term to stay net-neutral —
the lane **DECLINES and says so in those words**, per the dispatch. A construct
rung whose "shared machinery" is a second exemption has not built shared
machinery.

### The two-sided price, and how the REFUSAL's own cost is counted

Per CLAUDE.md #1042 / NC-5. The refusal here is *"decline to lift; leave clause
(c2) scoped to `??_C@_0`"*, and its cost is **not zero**:

* the shipped rule stays a **hypothesis tested on two STLport functions**
  (fence163 §7.2: 1,055 of 1,056 admitted bodies), so its `+163` is a property
  of the corpus and not of the port;
* clause (c2) remains reachable only where the walk is blind (845/871), so it is
  **retired silently** by any lane that repairs the walk (fence163 §6.1);
* the 7 unemitted fenced siblings (§7.4) stay latent rather than paid;
* Option A (full reproduction) needs the inline model as a *model*; a scoped
  exemption contributes nothing toward it.

Both sides are quantified in the rung's §-price table in the same units
(`fnbyte-exact`, `fnbyte-differs` symbols, census key counts) before any
recommendation is made. **A decline that does not price its own side is not a
priced refusal and would score as FAILED.**

## 4. Reach: measured, never estimated from a blocker row

§8.1 item 3 is binding and is restated as a rule of this lane: **no number for
"how many bodies the lifted rule touches" is derived from
`callee-defined-in-tu:eof`, from `data-sym-strlit-fenced:eof`, or from any other
blocked-key count.** The repo's measured ratio family (first-blocker counts run
~9× optimistic; `w-section`'s 1,457-bound realized 163;
`MEMORY.md` "Ranking instruments measure themselves") is the standing reason.

Reach is reported as the **counterfactual build's realized delta** — the
per-symbol difference between two `--fnbyte-diff-jsonl` dumps plus the
`fnbyte-exact` / `fnbyte-refused-parse` movement — or the word **UNMEASURED**.
P15 scores the blocker row against the realized number explicitly, so the
estimate-from-blocker error is falsifiable rather than merely forbidden.

## 5. Probe soundness — the environment control, pinned by NAME

`docs/rungs/README.md` probe rule 1 (boards **#3219** / **#3231**) is binding.
This worktree was created by `scripts/setup_worktree.sh`, which symlinks
`compilers/`, so it *should* be provisioned — **that is a claim to be verified,
not assumed**, because the failure mode is a capture-based test SKIPPING and
reading GREEN.

The control, frozen here:

* **Control name (pinned by NAME, not by count):** the toolchain-gated
  integration test `crates/c2-harness/tests/strlit_fence.rs` — its cells are
  capture-based and cannot pass without `compilers/`. Environment validity is
  asserted by requiring the workspace test run's **executed-test count ≥ 1,660**
  AND the run's **wall duration ≥ 30 s** (an unprovisioned tree completes the
  capture targets in ~0 s). A run failing either assertion is **VOID, not
  provisional**: discarded, re-run, and the invalid log kept.
* Every mutant is run in the **same** validated environment and the validation
  is re-asserted per run, not once for the campaign.
* The results table in the rung doc is **derived from the logs**, never
  accumulated.

## 6. Mutant colours, registered BEFORE any mutant runs

Probe: `cargo test --workspace --release --no-fail-fast` at the lane tip, in the
validated environment of §5. Mutations are applied to the **input** (the crates
under test), never to the oracle (#3174). Each mutation: one site, site-count
asserted and printed, built, tested, reverted.

**Registered conditionally, and the condition is registered too:** MD1–MD5
apply to a **shipped** lifted predicate. If the Outcome is `declined`, the
mutant campaign is reported as **NOT RUN, by the pre-registered condition** —
not quietly omitted, and not replaced with mutants of the unchanged tree.

| id | site | mutation | registered colour |
|---|---|---|---|
| **MD1** | the lifted clause | delete it entirely | **RED** |
| **MD2** | the lifted clause | invert the `eh-state1` test (`!=` → `==`) | **RED** |
| **MD3** | the lifted clause | drop the `tu_modelled_callees` exemption | **RED** |
| **MD4** | the lifted clause | swap the emit-binding ground map for `defined_name_set` (the blind walk) — i.e. silently restore clause (c)'s reach | **RED** |
| **MD5** | the lifted clause | re-add a `data_syms` / `STRLIT_NARROW_PREFIX` conjunct — i.e. silently re-scope the rule back to a named exemption | **RED** |

A mutant reading GREEN is a **hole in the cells**, reported as one, and the cell
that closes it is written (outside the peer-owned test seams; if it can only
live in `crates/c2-harness/tests/`, it is **described and routed**, per §0).

## 7. Registered bias

Dispatched to build a phase, I will want the lift to look like a rule and to
look cheap. The flattering directions, named so they are falsifiable:

1. **Reporting the census key movement as the result** when the goal's unit is
   `fnbyte-exact` (`MEMORY.md`: "Only fnbyte maps to the goal" — a census gain
   is not a goal gain). Scored by P8, which demands the paying half pay in
   `fnbyte-exact` and not in a key count.
2. **Keeping a small residual scoping term** — `data_syms`, a prefix, a name
   list, a "only where X" — to make the price come out flat, and calling the
   result a rule. Scored by decision-rule clause 1 and by mutant **MD5**.
3. **Estimating the reach from the blocker row** because the counterfactual
   build is expensive. Scored by P15 and forbidden by §4.
4. **Declining without pricing the decline**, which is the failure NC-5/#2691
   names. Scored by §3's two-sided price table being a required deliverable.

P9 is registered at 0.55 — i.e. this lane's own most likely outcome is
**`declined`** — precisely so that a decline cannot be presented later as a
disappointment that needed softening, nor a ship as a foregone conclusion.
