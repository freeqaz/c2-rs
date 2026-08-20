# PREREG — lane `w-warranty`

    Lane:      w-warranty
    Branch:    wt-w-warranty
    Base:      c98803beb (master at lane start)
    Kind:      construct rung + instrument. Fixtures: none. Census: +0.
               REQUIRED-ZERO byte delta in emitted bytes.
    Frozen:    2026-08-20, as this branch's FIRST commit, before any edit
               and before any probe was run.
    Block:     #3337-#3341, allocated by the coordinator. Rows beyond that
               are drafted UNNUMBERED and say so.

---

## 0. The thesis

Three live instances of *absence read as success* sit inside the warranty
layer, named by `docs/REFACTOR_REVIEW_2026-08-20.md` §0.1/§0.2/§0.3 and by
boards **#3247** and **#3219**. This lane closes them, and closes them under
the rule that makes the closure worth anything:

> **Every guard added must be PROVEN to fire, by mutation, before it is
> claimed.** An assertion that cannot fail is this repo's signature failure;
> adding one while fixing three would be a net loss.

Nothing here converts a TU. `match` is predicted **26 → 26** and `mismatch`
**0 → 0** by construction: no file under `crates/{c2-il,c2-core,c2-obj,
c2-reference}` is touched by any planned edit. Predicted reach: **0**.

---

## 1. What I read out of the source BEFORE freezing this, and the correction
   it forces on the review

Read-only inspection of `crates/c2-harness/src/cli/reference.rs`,
`crates/c2-harness/src/cli/perf.rs`, `crates/c2-harness/src/lib.rs` and
`crates/c2-harness/tests/cli_flags.rs` at `c98803beb`. Three facts, each of
which changes what the review's proposed fix can honestly claim:

**F1. `accepted_group`'s only assertion is `assert_ne!(code, Some(2))`**
(`cli_flags.rs:1013`) — "the parser did not refuse this argv". Established by
the coordinator; re-read and confirmed here. The three heavy `#[test]`s
(`…_accepted_selftest`, `…_accepted_bench`, `…_accepted_perf`) each run one
whole-corpus invocation to completion and discard its verdict.

**F2 — THE CORRECTION. `c2rs perf` deliberately exits 0 on a port
`Mismatch`.** `cli/perf.rs:153-161`, verbatim: *"the reference is the sole
judge, so a port Match/Mismatch/NotImplemented is per-TU reporting, not a
harness failure. Only a capture/replay error or a broken P0.1 replay
(ref-replay-inexact) is a hard failure of the benchmark itself."* The
`Port=Mismatch → ExitCode::FAILURE` the review cites at `reference.rs:617-628`
is in **`cmd_diff`** — the per-fixture command — **not** in `cmd_bench`.

**F3. `bench` and `selftest` do not grade the port at all.** Both loop
`oracle_selftest` (`lib.rs:404`), which is *reference determinism + reference
capture stability* (`run_selftest`, `lib.rs:412-470`). Neither ever calls the
port. `cmd_bench`'s `fail == 0 && err == 0` (`reference.rs:663-668`) is a
statement about `c2.dll`'s own reproducibility, not about `port(IL)`.

**Therefore the review's §0.1 fix as written — `assert_eq!(code, Some(0))` on
all three — would NOT catch a corpus-wide wrong emit.** It would catch
reference non-determinism and capture instability (real warranty, worth
having) and it would read as if it caught wrong emits. That is the defect
family again, one level up: a guard whose name promises more than it can
fire on.

The corpus-wide port compare *is* computed by `perf` — `perf::bench_fixture`
sets `r.port` per fixture and the summary line prints
`summary: {matched} port Match, {mismatched} mismatch, {ni} not-implemented`
(`cli/perf.rs:143-145`). It is printed and then dropped on the floor. So the
honest guard reads **stdout**, not the exit code, and does not disturb
`perf`'s deliberate exit-code contract.

---

## 2. The three guards, each with what it catches and what it CANNOT

### G1 — `accepted_perf` asserts `0 mismatch` in the corpus summary
New, separately named assertion on the SAME execution as
`every_invocation_the_scripts_make_is_still_accepted_perf`. Parses the
`summary:` line of `c2rs perf`'s stdout and requires the mismatch count to be
`0`, and requires the summary line to be PRESENT.

* **Catches:** any port `Mismatch` on any of the 386 fixtures — a corpus-wide
  wrong emit, inside `cargo test`, for the first time.
* **CANNOT catch:** a wrong emit on IL outside the fixture corpus (the whole
  878-TU workload); a class the port refuses (`NotImplemented` is not a
  mismatch and must not be); anything `perf` skips.
* **Unprovisioned worktree:** GREEN. `perf` prints `SKIP: strace /
  i686-w64-mingw32-gcc absent` or exits early via `args.toolchain()`, and the
  guard's absence branch is reached — which is exactly why G2 exists and why
  G1 records the absence branch it took in its own output.

### G2 — `accepted_selftest` / `accepted_bench` assert `exit == 0`
Separately named from the roster assertion, on the same executions.

* **Catches:** reference non-determinism, capture instability, and per-fixture
  reference ERRORs, corpus-wide, in `cargo test`. Today these exit 1 and the
  suite is green.
* **CANNOT catch:** a wrong emit (F3 — the port is never invoked), and cannot
  catch anything on the eight `Rest` invocations, which are NOT covered:
  `c2rs diff` legitimately exits 1 on `Port=Mismatch` and `census`/`gap` have
  their own contracts. **The assertion is scoped to the two commands whose
  exit-0 contract I verified in source, not blanket-applied to the roster.**
* **Unprovisioned worktree:** GREEN (both exit 0 with `SKIP`).

### G3 — `C2RS_REQUIRE_TOOLCHAIN=1` by default in `scripts/partest.sh`
(+ `gate.sh` if inspection shows it does not already demand it), with an
opt-out. Board **#3247**'s closure.

* **Catches:** an entire suite run that graded nothing — the fresh-worktree
  case where 132 of ~179 integration tests early-return and the totals are
  byte-identical.
* **CANNOT catch:** a *partially* provisioned run (compilers present, strace
  absent), which still skips a subset silently. That residue is stated, not
  papered over, and is a candidate row.
* **Unprovisioned worktree:** RED — by design. That is the whole point.

### G4 — per-row consecutive-non-executing-verdict counter in `gate.sh`
Board **#3219**'s named missing instrument. Persists each row's verdict word
per run and prints `N consecutive` for a row that has not executed.

* **Catches:** a standing gate row that stopped executing and stayed that way.
* **CANNOT catch:** a row that executes and is wrong; nor can it retroactively
  say how long `hatch-red` has been stale — the counter starts at this commit.
* **Unprovisioned worktree:** unaffected (toolchain-free rows).

---

## 3. Mutation protocol — the colours registered IN ADVANCE, by NAME

No guard is claimed without its RED. Each mutation is applied, the named test
run, the colour recorded from the log, then **reverted** and the GREEN
re-taken. Logs kept under `work/w-warranty/logs/`.

| id | mutation | must go RED | must stay GREEN |
|---|---|---|---|
| **M1** | plant a wrong emit in the port: corrupt one byte of `PortC2`'s emitted obj | `cli_flags::every_invocation_the_scripts_make_is_still_accepted_perf`, failing on **G1's** message | `…_accepted_bench`, `…_accepted_selftest` (F3 — proves G2 does not secretly claim G1's catch) |
| **M2** | plant a reference determinism failure in `run_selftest` | `…_accepted_bench` AND `…_accepted_selftest`, failing on **G2's** message | `the_split_is_a_partition_of_the_roster` |
| **M3** | run the suite with the toolchain made unresolvable, `C2RS_REQUIRE_TOOLCHAIN=1` | exactly `require_toolchain::a_run_that_claims_to_grade_must_have_a_toolchain_to_grade_with` | the same run with the variable unset — GREEN, proving the portable lane's entitlement survives |
| **M4** | feed `gate.sh`'s counter a fabricated run history in which a row is REFUSED n times, then executes | the counter prints `n consecutive` and then resets to 0 | `gate.sh --selftest` overall |

**M1 is the load-bearing one.** If M1 does not redden G1, G1 is decoration and
this lane says so in those words.

**Anti-theft.** M1's RED must name G1's assertion message and NOT the
`assert_ne!(code, Some(2))` roster message; M2's RED must name G2's and not
G1's. A mutation that reddens the *wrong* assertion has not proven the guard.

---

## 4. Invalidation rules — stated before the first measurement

1. **A colour taken in an environment I did not validate is VOID, not
   provisional.** Validation = the `compilers/` symlink resolves AND the run
   reports **0** `SKIP: toolchain absent` AND `census_gate` has a non-zero
   duration. Void logs are kept and labelled void.
2. **Control pinned BY NAME**, not by count:
   `require_toolchain::a_run_that_claims_to_grade_must_have_a_toolchain_to_grade_with`
   must PASS in every provisioned run of this campaign, and
   `cli_flags::the_split_is_a_partition_of_the_roster` must PASS in all four
   mutation runs (it is toolchain-free, so it is the "did the binary build"
   control).
3. **Required-zero byte delta.** Any change to a file under
   `crates/c2-il`, `crates/c2-core`, `crates/c2-obj`, `crates/c2-reference`
   other than a mutation that is reverted in the same session **FAILS this
   lane**, whatever else it produced. Verified by `git diff --stat base..tip`
   over those four paths reading empty.
4. **`match` must read 26 and `mismatch` 0 at the tip.** Any other value
   invalidates the lane's zero-delta claim and is reported as the headline.
5. **Workload stamp equality.** `c2rs gap` prints `workload <sha> …`; base and
   tip reads must print the SAME stamp or every stamp-derived number in the
   rung is withdrawn (#3306/#3311). I read the stamp at lane start and at lane
   end and diff them; I carry no stamp VALUE from the brief.
6. **`gap-metric` key count is measured at MY base** with
   `grep -cE '^ *gap-metric \S+ \S+$'`, never carried from the brief. An
   unexpected delta owes a re-measurement before it owes a cause (#3269).
7. **#3288 — every published count is derived a second, differently-built way
   and the two are diffed.** Suite totals: cargo's own summary line vs a
   count of `^test .* \.\.\. ok$` lines. Guard counts: `grep` of the source vs
   the test's own runtime `ran` counter.
8. **Tables are derived from the logs at write time, never accumulated.**
9. **No absolute timings published.** The box is under heavy external load and
   a 9-agent workflow is live. PASS/FAIL and counts only. `census_gate`'s
   duration is quoted only as the non-zero liveness assertion rule 1 requires.

---

## 5. Predictions, recorded so they can be wrong

* **P1** G1 catches M1. *(If false, the lane's main deliverable is dead and
  says so.)*
* **P2** G2 does **not** catch M1 — because `bench`/`selftest` never call the
  port (F3). A GREEN `bench` under a planted wrong emit is the *evidence for*
  the correction in §1, not a failure.
* **P3** `perf` exits **0** under M1, confirming F2 against the running binary
  and confirming that the review's exit-code fix would have been decoration
  for the wrong-emit case.
* **P4** The suite total moves by exactly the number of `#[test]` functions
  this lane adds — no existing test's name changes.
* **P5** `match 26 / mismatch 0` unchanged; gate PASS; byte delta zero over
  the four judge crates.
* **P6** `hatch-red` reads `REFUSED HATCH-STALE` at my base too — master's
  pre-existing state, not any lane's — and G4's counter will print `1
  consecutive` on its first run because the history starts empty.

---

## 6. Board rows planned (block #3337-#3341)

* **#3337** — the corpus-wide differential's verdict is discarded, AND
  `perf` exits 0 on `Port=Mismatch` by design, so the review's exit-code fix
  is insufficient for the wrong-emit case; the guard must read stdout.
* **#3338** — `C2RS_REQUIRE_TOOLCHAIN` demanded by default by the canonical
  runner; #3247's closure.
* **#3339** — per-row consecutive-non-executing-verdict counter; #3219's
  named missing instrument.
* **#3340**, **#3341** — held for findings this lane has not made yet.
  Anything beyond the block is drafted UNNUMBERED and labelled so.

---

## 7. Outcome rule

If G1 lands with a proven RED, the outcome word is `instrument`. If the
guards land but no mutation reddens them, the outcome word is **`FAILED`**,
in that word, regardless of how many lines were written.
