# PREREG — w-fence163 (frozen BEFORE any probe, first commit of branch `wt-w-fence163`)

    Lane:      w-fence163
    Base:      master `3835469c` (verified ancestor of current master `7e541a54`,
               which is one docs commit ahead)
    Date:      2026-08-17
    Mission:   convert `w-section`'s declined L5 into a shippable change by
               measuring the ONE fence it needs — the input-side predicate that
               refuses `?ContentPath@XboxContentMgr@@UAAPBDH@Z` (port 3 words vs
               real c2's 14 under the L5 widening) while keeping the +163
               relocation-graded `fnbyte-exact`.

## 0. What is frozen here and what is not

Frozen: the predictions below, their probabilities, the mutant colours, the
ceilings (NO discount factor anywhere), and the decision rule. Not frozen:
capture-cell grids — none exist yet because the mechanism is unmeasured; any
grid built later is recorded as **measurement** in the rung doc with its content
hash at the commit that adds it, and scores nothing here (registering an
already-taken measurement as a prediction is scoring a coin after it landed —
`w-section` §2's rule).

Nothing in this lane touches `crates/c2-core/src/coff/` or
`crates/c2-core/src/codegen/` (peer `w-npos` / seam discipline). If the fence
turns out to need the `gl-stop-26-introduced` site (`c2-il/src/func/diag.rs:76`
constant, wherever it is raised), this lane STOPS on that part and records it.

## 1. Handed-down base figures, to be re-measured before use (P1)

From the dispatch, cited at `3835469c`: match 25 · mismatch 0 · codegen-gap 0 ·
vocab-gap 845 · capture-fail 8 · fnbyte-exact 35,734 · fnbyte-refused-parse
113,612 · anchored `gap-metric` keys 394 · `cargo test` 1,648 / 0 / 42.

From `docs/rungs/2026-08-16-section.md` (merged at this base), measured by
`w-section` at ITS base `202bfc3f`:

* L5 = L2 (call fences lifted) + `bind.rs:886` `resolve_data` admits `??_C@_0…`:
  `fnbyte-exact` +163 (all relocation-graded), `fnbyte-refused-parse` −164,
  `fnbyte-differs` +1 (named: `?ContentPath@XboxContentMgr@@UAAPBDH@Z`,
  `src/system/os/ContentMgr_Xbox.cpp`, port 3 words `lis·addi·b` vs c2 14,
  sub 2 / del 11, classes `mixed:reg+imm` + `opcode`), match 25 Δ0, mismatch 0.
* Without the call-fence lift the class is **163 functions over 67 TUs** (§5.1);
  L1 alone moves `data-sym-unresolved:eof` −163 → `data-sym-not-extern:eof` +163
  and NO graded column.

**This lane's planned ship is the TWO-SITE widening only** — `gl.rs:1085`
`NAME_SEPARATORS += 0x25` and `bind.rs:886` `resolve_data` admits `??_C@_0…` —
**without** the `calls.rs:431/437` call-fence lifts, because (a) the lifts
convert nothing (`w-section` §3.4: the 1,293 multi-sym bodies move to
`callee-unresolved-tail-call` and convert zero), and (b) lifting `syms > 1`
would turn `w-guards`' M1 arity-fence guard RED, which is a guard firing on a
real weakening, not on this lane's deliverable.

## 2. Hypotheses for the ContentPath over-acceptance (registered before reading
##    the source, the IL, or the obj)

The port's whole-segment parser accepted the body (acceptance requires the
cursor to reach `seg.len()` — census.rs's `:eof` comment), decoded it as a
`MultiArgTailCall` with one `??_C@_0` SymAddr argument, and emitted 3 words
where c2 emits 14 (1 equal + 2 sub + 11 del). Mechanism hypotheses:

| id | mechanism | P |
|---|---|---:|
| **H-A** | the literal is not (only) a direct call argument — the body's real computation (conditional select, index, format) collapses into productions the parser consumed as argument/return plumbing, i.e. a production is over-broad on this byte pattern | 0.35 |
| **H-B** | the call is not actually in tail position / its result is post-processed, and the parser's tail-call matcher over-consumed the difference | 0.25 |
| **H-C** | callee- or signature-side: c2 declines the tail-conversion for this callee (variadic / different arg homing), emitting a framed call, while the shape itself is as parsed | 0.20 |
| **H-D** | something else entirely (multi-block body, EH state, second statement) | 0.20 |

**A decidable input-side predicate exists** (some byte-level fact in the `.ex`
segment / `.gl` records separates ContentPath from the 163): **P = 0.75.**

## 3. Predictions, probability form, ceilings with NO discount

| id | prediction | P |
|---|---|---:|
| **P1** | the base 878-TU scan re-measured in THIS worktree at `3835469c` reproduces every §1 figure exactly (match 25, mismatch 0, codegen-gap 0, vocab-gap 845, capture-fail 8, fnbyte-exact 35,734, fnbyte-refused-parse 113,612, 394 keys) | 0.90 |
| **P2** | the two-site widening alone (NO call-fence lift) moves `fnbyte-exact` +163, `fnbyte-refused-parse` −164, `fnbyte-differs` +1, and the +1 is `?ContentPath@…` | 0.70 |
| **P3** | `match` 25 Δ0 and `mismatch` 0 and `codegen-gap` 0 at every rung of this lane, including the shipped tip | 0.92 |
| **P4** | a decidable pre-emission fence exists that refuses ContentPath AND holds back ≤ 13 of the 163 (i.e. shipped `fnbyte-exact` ≥ +150). Ceiling, no discount: **+163** | 0.60 |
| **P5** | the fence's cost side (currently-right bodies it refuses) is **0**: it is scoped to the newly-admitted `??_C@_0`-argument population, so nothing exact at base changes verdict. Checked per symbol, not by subtracting totals | 0.75 |
| **P6** | `w-guards`' cell B test (`the_data_symbol_linkage_gate_is_the_one_byte_that_moves_the_key`) stays **GREEN** under the prefix-gated admission — contra `2026-08-16-guards.md` §8.1's advance call of RED — because cell B's refused name is `?objA@@3HA`, which does not carry the `??_C@_0` prefix. Either way the recorded response is executed: a string-literal cell is ADDED beside cell B; no existing guard is weakened or deleted | 0.65 |
| **P7** | `cargo test --workspace --release` at tip: 0 failed, 42 targets, passed = 1,648 + (number of tests this lane adds) | 0.85 |
| **P8** | the 163 newly-exact functions span 67 TUs and convert 0 of them (all remain `vocab-gap`) | 0.80 |
| **P9** | identity control: reverting the `crates/` change at the end reproduces the base scan at 0 deltas over all anchored `gap-metric` keys | 0.95 |
| **P10** | the new admitted-class keys and the fence's refusal key are visible in the scan (the fence publishes a countable row — a standing `fence-blocks-*` count ≥ 1, naming ContentPath's population), per CFG_SHAPE §6.3 rule 2 | 0.80 |

Decision rule (frozen): ship the two-site widening **iff** (a) the fence
predicate is decidable from the IL container alone, (b) re-measured
`fnbyte-differs` Δ = 0 and `mismatch` = 0 and `match` = 25 at tip, and (c) the
two-sided price is favorable in the goal's units — the fence refuses nothing
that is exact at base (P5) and holds ≤ 13 of the +163 (P4). Any wrong emit
(`fnbyte-differs` +N, N ≥ 1, or `mismatch` ≥ 1) ⇒ revert and **Outcome:
declined** with the measurement as deliverable.

## 4. Mutant colours, registered BEFORE any mutant runs

Probe: `cargo test --workspace --release --no-fail-fast`, at the lane tip (with
the lane's new tests in the tree). Controls mutate the INPUT (the crates under
test), never the oracle (#3174). Each mutation applied to exactly one site,
site-count asserted and printed, built, tested, reverted.

| id | site | mutation | registered |
|---|---|---|---|
| **MF1** | the new fence predicate | invert it (admit what it refuses, refuse what it admits) | **RED** |
| **MF2** | the new fence predicate | delete it (admit the whole `??_C@_0` population unfenced) | **RED** |
| **MF3** | `bind.rs` admission prefix | widen `??_C@_0` → `??_C@` (admits wide/other literals) | **RED** (a new test pins narrow-only) |
| **MF4** | `bind.rs:886` | drop `extern_data.contains(&name)` (w-guards' M3, re-run with this lane's change in place) | **RED** (the existing guard must still fire) |
| **MF5** | `gl.rs:1085` | `NAME_SEPARATORS` loses `0x26` (i.e. `[0x00, 0x25]`) | **RED** |

## 5. Registered bias

The registered bias: having been dispatched to ship, I will want the fence to
look decidable and cheap; the two directions that flatters are (i) a
name-shaped or population-shaped "predicate" that is really a denylist of the
one known-wrong body (which is NOT decidable in §6.3's sense and must be
declined), and (ii) under-counting the fence's hold-back among the 163. Both
are scored above (P4, P5) so the flattering direction is falsifiable.
