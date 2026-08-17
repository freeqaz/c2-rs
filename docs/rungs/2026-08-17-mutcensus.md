# MUTCENSUS — how many of `c2-il`'s refusal sites have no test that can fail on them

    Tag:       w-mutcensus
    Slug:      mutcensus
    Date:      2026-08-17
    Kind:      characterization — the question: which of `crates/c2-il`'s
               refusal/fence sites are unguarded, measured by one registered
               mutation per site against the full workspace suite
    Outcome:   instrument
    Fixtures:  none — characterization
    Census:    +0 — required-zero byte delta; every `crates/` edit in this lane
               is an applied-and-reverted mutant, and
               `git diff master..HEAD -- crates fixtures scripts` is EMPTY at
               the tip
    Record:    this file; prereg `_2026-08-17-w-mutcensus-prereg.md` (frozen at
               `58cb6803`, committed BEFORE the first mutant ran); deviations
               and corrections `work/w-mutcensus/deviations.md`; raw logs,
               runner and table generators under `work/w-mutcensus/` (tracked)

Provenance: board **#3217** — *"`#3199`'s list of unguarded surfaces was NOT
exhaustive … and NOTHING ANYWHERE ENUMERATES THE FENCES THAT HAVE NO TEST … a
mutation census over `crates/c2-il`'s refusal sites is a lane, and it is the only
thing that turns this from anecdote into a NUMBER."* This is that lane.

## 0. The answer

<!-- FILL:ANSWER -->

## 1. Populations, and where each figure was measured

<!-- FILL:POPULATIONS -->

## 2. What a fence site is here, and how the 63 were enumerated

The rule is `work/w-mutcensus/enumerate.sh`, run from the repo root at
`3835469c`, plus a **bounded, published** reading step. Both are quoted from the
frozen prereg §1 rather than restated, so the frame cannot drift after the fact:

* **E1** — every `refuse("<key>")` raise site: **23**, all in
  `func/body/shapes/calls.rs`.
* **E2** — every non-test, non-doc line that **raises** one of the **19
  fence-key constants** (the 24 `pub(crate) const … : &str` in `func/body/mod.rs`
  minus the 5 dispatch-state constants and the 2 grammar-context constants).
* **E3** — every `Block::at_end(` site.
* **Reading step (bounded):** within each function containing an E1–E3 site,
  every conditional that *decides whether the raise fires* is part of the site,
  plus the resolver/gate functions those sites call (`resolve_data`,
  `resolve_data_def`, `resolve_bss_def`, `is_varargs`, `gl_extern_data_names`,
  `NAME_SEPARATORS`, `opt_word_mode`) and `IlBundle`'s TU-level admission gates
  (`functions`, `dyninit_tu`, `data_tu`).

**N for the headline is the 63 mutated `c2-il` sites.** `C2` — the `c2-core`
backstop at `codegen/calls.rs:1815` — is run as a control but is **not** in N,
because it is not a `c2-il` fence.

### 2.1 What was enumerated and deliberately NOT mutated — with counts

No silent caps: every dropped class is published with its size and its reason
(prereg §1 E5).

| dropped class | count | why |
|---|---:|---|
| grammar fail-closed `blk(` sites | **1,227** raw grep lines | one suite run per site is ≈ 5 days of wall clock, and they are a different guard class — the key is generated *from the blocking byte*, so a key-swap mutation does not exist and a removal merely moves the parse to the next blocking byte. A **sampled** census over them is a future lane |
| `blk_type(` | **6** | same class |
| `Block::refuse(` | **106** | same class (counts overlap; they include helper definitions and test uses) |
| shape-file `OptWordMode` admission predicates | **18** non-test comparison sites | budget; second-tier published-key proximity |
| `IlBundle::dyninit_tu` `return None` clauses | **12**, of which **1** mutated (`D1`) | budget — **11 dropped** |
| `IlBundle::data_tu` `return None` clauses | **14**, of which **1** mutated (`D2`) | budget — **13 dropped** |
| `IlBundle::functions()` interior gates past the three mutated | enumerated by reading | budget |
| `STORE_RUN_BIND_CALL_TAIL_RETIRED` | **0 live raise sites** | a fence key with **no fence** — test-only since #1212's correction. No mutant is possible |

**So the headline is not "all fences in `c2-il`".** It is *all 63 sites the
frozen frame enumerated*, beside a published 1,227-site class the frame
deliberately does not reach.

### 2.2 The frame already has a hole, and a peer put it there during the campaign

`w-fence163` landed `d28326b4`
(*"admit narrow string literals behind an EH-state inline fence"*) while this
campaign was running. It adds a **20th** fence-key constant —
`DATA_SYM_STRLIT_FENCED = "data-sym-strlit-fenced"` in `func/body/mod.rs` — with
**5** lines mentioning it and new deciding gates in `bind.rs`,
`bundle.rs::functions()`, `census.rs` and `gl.rs` (+240 / −13 over five `c2-il`
files).

**This lane did not re-enumerate to absorb it, and must not:** the frame and all
64 registered colours were frozen at `3835469c` before the first mutant ran, and
widening the frame afterwards would unfreeze the prereg. So the site is recorded
as one the census **necessarily misses** — and the more useful thing it
establishes is the instrument's **shelf life**:

> **One peer lane landing one fence is enough to make X/N stale.** A mutation
> census over `c2-il`'s fences is a fact about a *commit*, not about the
> repository. Re-running `enumerate.sh` is a precondition of quoting X/N against
> any later head, and *nothing in the repo enforces that* — see §9.

## 3. The table — every one of the 63 sites, registered against observed

<!-- FILL:TABLE -->

## 4. The per-family pattern — the shape #3217 asked to be counted

<!-- FILL:FAMILY -->

## 5. Peer verification: the four guards `w-guards` landed last wave DO hold

**Stated plainly, as an independent finding of this lane rather than a
restatement of `w-guards`'.** The five controls were run first, from a different
session, at a different commit, in a different worktree, with recipes written
from the site text rather than from `w-guards`' patches. Four of them reproduce
`w-guards`' G1–G4 failing-test **sets and counts exactly**:

| control | site | observed | failing tests |
|---|---|---|---|
| `C1` = M1 | `calls.rs:431` arity fence | **RED 1,646 / 2** | `the_call_argument_arity_fence_is_a_series_and_admits_exactly_one_symbol` · `the_two_symbol_thunk_exemption_turns_on_the_bare_body_marker_alone` |
| `C2` = M2 | `c2-core/codegen/calls.rs:1815` backstop | **RED 1,647 / 1** | `the_data_address_setup_refuses_the_shapes_it_has_no_capture_for` (the #3199-named test) |
| `C3` = M3 | `bind.rs:891` `.gl` linkage gate | **RED 1,645 / 3** | `the_data_symbol_linkage_gate_is_the_one_byte_that_moves_the_key` · `the_two_data_symbol_census_keys_are_not_interchangeable` · `the_census_key_survives_the_round_trip_into_the_reachable_ranking` |
| `C4` = M4 | `census.rs:1216/1218` data-sym key swap | **RED 1,646 / 2** | `the_data_symbol_linkage_gate_…` · `the_two_data_symbol_census_keys_are_not_interchangeable` |
| `C5` | `calls.rs:430` thunk exemption | **RED 1,642 / 6** | the thunk guard, both data-sym guards, the round-trip, and two `wr1_dyninit` decode pins |

**Controls: 5 of 5 RED. Zero control anomalies.** Prereg §2.3 registered
P(any control reads GREEN) = 0.05 and made a GREEN control a
campaign-stopping finding that outranks the census; that branch was not taken.

Two things this establishes that `w-guards`' own rung could not:

1. **The guards fire for the reason claimed, not incidentally.** `C3`'s and
   `C4`'s failing sets differ by exactly the round-trip test, and `C5` — a
   surface `w-guards` found *while building* the third guard — takes down six
   tests including both data-symbol guards, which is the interaction #3216
   predicted in advance.
2. **The probe is live.** A control that reproduces a known failing set to the
   test name is the positive check that the mutation harness can see a guarded
   site at all. Without it a table of GREENs is indistinguishable from a broken
   runner — and this campaign found exactly that failure mode in its own
   instrument (§7), so the check is not ceremonial.

Each control's colour was re-derived under the corrected rules of §7 and every
one graded **70–95 s** against real `c2` in the `census_gate` target.

## 6. Prereg scorecard

<!-- FILL:SCORECARD -->

## 7. The campaign's own instrument failure, found mid-run

**The registered baseline `1,648 / 0 / 42` is byte-identical with and without a
toolchain, so it cannot distinguish a run that graded against real `c2` from one
that graded nothing.** Full account and the three-layer fix:
`work/w-mutcensus/deviations.md` D6. In brief:

| run | toolchain | passed / failed / targets | `census_gate` target |
|---|---|---|---|
| session-1 baseline | present | **1,648 / 0 / 42** | **84.17 s** |
| `N0wtB` | **absent** | **1,648 / 0 / 42** | **0.00 s** |
| `N0wtC` | **absent** | **1,648 / 0 / 42** | **0.00 s** |

By design (CLAUDE.md) every toolchain-driven test prints `SKIP: toolchain absent`
**and passes**, so the totals are preserved and prereg §4.5's `targets != 42`
rule sees 42 of 42 reporting `ok`. **GREEN means "no test can fail on this
site", so in an unprovisioned worktree every site guarded only by the real-`c2`
differential reads GREEN — the error is one-directional and it inflates X, the
headline.**

It surfaced as a *contradiction between two runs of one mutant*, not by
inspection: `L4` failed
`census_gate::the_census_and_the_port_agree_about_what_is_in_class` after
171.58 s in the provisioned worktree, and passed that same test in **0.00 s** in
the sidecar. Same mutation, same commit, two different failing sets.

Fixed in three layers — worktrees provisioned via
`scripts/configure_existing_worktree.sh` (whose own hard gate is the fixture
census verdict); a **pre-flight** census probe in `run_mutants.sh` that aborts
the list rather than emitting a colour; and the **`census_gate` duration recorded
per run**, with anything under 1 s classified `INVALID`. Because the table is
*derived* from the logs by `rederive.sh`, the rule applies retroactively to every
log on disk.

**Two colours were discarded** (`CS2` read GREEN, `L4` read RED, both in
unprovisioned sidecars) and re-run from scratch. **All 8 session-1 colours
survive** re-derivation. The faulted logs are kept as `*.notoolchain*.log` and
the new rule classifies all four `INVALID` at 42 of 42 targets, which is the
check working.

> **The generalization is worth more than this lane's X.** The repo already knew
> this trap — `configure_existing_worktree.sh`'s own header says *"`cargo test`
> is green, `c2rs diff` says SKIP, and a change that mis-emits looks exactly like
> a change that is byte-exact"* — and this lane walked into it anyway, **because
> the prereg specified its probe as a pair of totals, and totals are exactly what
> the fault preserves.** A probe defined by a count cannot detect a population
> that silently left the count. That is STATUS trap 5 one level up: not a missing
> target, but a present target that measured nothing.

## 8. Gate evidence

<!-- FILL:GATE -->

## 9. Found and not taken

<!-- FILL:FOUND -->
