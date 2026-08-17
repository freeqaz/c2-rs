# MUTCENSUS — X of N fence sites in `c2-il` have no test that can fail on them

    Tag:       w-mutcensus
    Slug:      mutcensus
    Date:      2026-08-17
    Kind:      characterization — the question: which of `crates/c2-il`'s
               refusal/fence sites are unguarded, measured by one registered
               mutation per site against the full test suite
    Outcome:   instrument
    Fixtures:  none — characterization
    Census:    +0 — required-zero byte delta; every `crates/` edit is a
               reverted mutant, verified clean after each of the TODO runs and
               by `git diff master..HEAD -- crates fixtures scripts` empty at
               the tip
    Record:    this file; prereg docs/rungs/_2026-08-17-w-mutcensus-prereg.md
               (frozen at 58cb6803, committed BEFORE the first mutant ran);
               raw logs + runner in work/w-mutcensus/ (tracked)

---

## 0. The answer, in one paragraph

TODO after campaign: X of N, controls 5/5 RED, prereg hits/misses, headline.

## 1. Populations, measured here

| figure | value | measured |
|---|---|---|
| baseline suite | 1,648 / 0 / 42 in 3:43 | this worktree at `3835469c` |
| TODO scan at tip | | |

## 2. What a fence site is here, and the enumeration rule

(from prereg §1 — E1 23 `refuse("` sites; E2/E3 named-key + at_end raises;
bounded reading step for deciding predicates; E5 dropped classes with counts)

## 3. The table

TODO table.py output.

## 4. Prereg scorecard

TODO.

## 5. Gate evidence

TODO end-state block.

## 6. Found and not taken

TODO.

---

## STATUS AT INTERRUPTION (2026-08-17, operator wrap-up request)

The campaign was stopped by operator request after 10 of 64 planned runs
(8 complete, 2 aborted mid-suite and recorded as NOT RUN). **Registered
colours: 8 of 8 HIT so far.**

| id | site | registered | observed | failing tests |
|---|---|---|---|---|
| C1 | calls.rs:431 arity fence | RED .97 | **RED** 1,646/2 | the w-guards G1 pair (arity series + thunk exemption) |
| C2 | c2-core calls.rs:1815 backstop | RED .95 | **RED** 1,647/1 | the #3199-named `the_data_address_setup_refuses_…` |
| C3 | bind.rs:890 linkage gate | RED .97 | **RED** 1,645/3 | w-guards G3 triple (linkage-gate, keys-not-interchangeable, round-trip) — after one INVALID recipe iteration (E0277 precedence), recorded |
| C4 | census.rs:1216/1218 key swap | RED .97 | **RED** 1,646/2 | w-guards G4 pair |
| C5 | calls.rs:430 thunk exemption | RED .90 | **RED** 1,642/6 | thunk guard + both data-sym guards + round-trip + 2 wr1_dyninit decode pins |
| L1 | leaf_store.rs:2254 group-shape frame | RED .65 | **RED** 1,647/1 | `every_bind_gate_fires_on_a_named_input` |
| L2 | leaf_store.rs:2257 group-shape base | GREEN .55 | **GREEN** 1,648/0 | — UNGUARDED |
| L3 | leaf_store.rs:2285 group-shape value | GREEN .50 | **GREEN** 1,648/0 | — UNGUARDED |

CS2 and L4 were mid-suite when stopped (partial logs kept as `*.aborted.log`;
L4's partial log already showed 1 failure at 14 targets, consistent with its
registered RED, but a <42-target run is not a colour by prereg §4.5).

**Interim X/N: 2 GREEN of 3 non-control sites run; 55 of 63 sites NOT RUN.**
The controls reproduce w-guards' G1-G4 failure counts exactly, so the probe
and the guards are live; the two GREENs are the first two entries of the
unguarded-fence table this lane was commissioned to produce.

Everything needed to resume is committed on this branch: the frozen prereg
(58cb6803), mutants.py (64 line-pinned specs, all still matching), the runner
with its clean-tree invariants, rederive.sh/table.py, and the logs. Resuming
is `./work/w-mutcensus/run_mutants.sh CS2 … B10` + `L4 … CA23`.
