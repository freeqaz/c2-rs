# `w-calleeguard` — deviations from the frozen prereg, and additions registered mid-lane

The prereg (`docs/rungs/_2026-08-18-w-calleeguard-prereg.md`, frozen at
`9673841c` before any probe) is **not edited**. Everything that departs from it,
or is added to it, is recorded here with the time it was decided relative to the
run it governs. Every colour below was written down **before** the run that
produced it.

---

## D1 — `N0R` is run although the prereg does not list it

**Decided before the run.** The prereg's §4 lists `N0`, `R5`–`R8`, `G5`–`G8`,
`C1` and `N1`. `campaign.sh` also runs `N0R` — the clean tree with the three new
guards skipped by name — because phase R's denominator must be a *measured*
count and not `N0 − 3` arithmetic. Registered before the run: **GREEN at exactly
1,660 / 0 / 43**, which is also the briefed base figure at `44794fa4`.

*Outcome: GREEN, 1,660 / 0 / 43, census_gate 65.06 s.* The briefed base
reproduces exactly, and `--skip callee_unresolved_arms` is confirmed to remove
exactly the three guards this lane added (`N0` 1,663 → `N0R` 1,660).

---

## A1 — an ADDITIONAL guard: the standing SITE COUNT

**Registered 2026-08-18, after phase R had run and before it was landed or
phase G started.** File: `crates/c2-harness/tests/callee_unresolved_sites.rs`
(this lane's own seam; **zero bytes in `crates/c2-il`**).

The prereg promised one witness per raise site. A witness table covers the sites
that existed when it was written, and **nothing makes a new arm add a row** —
which is `w-mutcensus` **F4** (*"nothing re-runs this census, so X/N goes stale
on the next landed fence, and one already landed during the campaign"*). A1 is
F4's standing count, scoped to this one dispatch: it reads `c2-il`'s source and
asserts the arm set, the family's site count, and that each of the four
constants is raised **exactly once** — the condition that makes a per-key
witness equal a per-site witness here.

Registered colours, before the first run with it in:

| | registered |
|---|---|
| clean tree | **GREEN** 0.95 |
| under `G5`/`G6`/`G7`/`G8` | **RED** 0.90 — a **second, independent** mechanism catching the same swap. Named as such, not counted as a separate site closed |

**This makes `G5`–`G8`'s failing sets larger than the prereg implied**, and that
is recorded here rather than discovered in the table.

---

## A2 — an ADDITIONAL instrument: `C2RS_REQUIRE_TOOLCHAIN`

**Registered 2026-08-18, before the run that grades it.** File:
`crates/c2-harness/tests/require_toolchain.rs`.

This is `w-mutcensus` **F1**, which that lane published as **NOT TAKEN for a
structural reason**: it lands a test under `crates/`, and a characterization
lane's success criterion is a required-zero byte delta — *"the same commit's two
halves"*, and *"twice in two waves that the instrument a lane discovered it
needed could not be landed by the lane that discovered it"*. **This lane's
deliverable is test code in `crates/c2-harness/`, so that conflict does not
exist here.**

It also bears directly on this lane's own obligations: `docs/rungs/README.md`'s
probe rule 1 requires a control re-run in every environment, and F1 is the
mechanised version of the same demand.

Three demonstration runs, colours registered before any of them ran. These are
**demonstrations, not part of the registered mutant scoring** — no `crates/`
source is mutated in any of them; only the environment moves.

| id | environment | registered |
|---|---|---|
| **D6a** | `C2RS_COMPILERS=/nonexistent`, `C2RS_REQUIRE_TOOLCHAIN` unset, **before** A2 lands | the suite reads **fully GREEN with the right target count**, `census_gate` at **0.00 s** — i.e. `w-mutcensus` D6 reproduced at this base — and `rederive.py` classifies it **INVALID**, not GREEN. 0.90 |
| **D6b** | the same, `C2RS_REQUIRE_TOOLCHAIN=1`, **after** A2 lands | **RED**, failing exactly `require_toolchain::a_run_that_claims_to_grade_must_have_a_toolchain_to_grade_with`. 0.90 |
| **D6c** | provisioned worktree, `C2RS_REQUIRE_TOOLCHAIN=1` | **GREEN** — default behaviour does not move and the demand is satisfiable. 0.95 |

D6a's log is **kept**, named `*.INVALID.log`, per the prereg §4.4's *"void, not
provisional"* rule and `w-mutcensus` §7.1(4).

---

## A3 — the suite count moves past what P14 anticipated in composition, not in size

P14 registered `1,660 + k`, `1 ≤ k ≤ 6`. With A1 and A2 landed the tip is
`1,660 + 3 (arm witnesses) + 1 (site count) + 1 (require-toolchain)` = **1,665**,
so `k = 5` and P14 still holds — but it holds over five tests where the prereg
was describing three. Recorded so the arithmetic is not read as a coincidence.

---

## A4 — phase R measured the tree BEFORE A1 and A2 existed

Phase R (`R5`–`R8`) ran at commit `d7eb8ab2`, with the three arm witnesses
present but **skipped by name** and with A1/A2 not yet written. That is the
correct pre-guard state for the four sites: A1 and A2 are both *guards*, so a
phase-R run with them in would have measured the post-guard tree. Phase G runs
at the tip, with everything in. The two phases therefore differ by five tests
and not by three, which is why `N0R` (1,660) rather than `N0` is phase R's
denominator.
