# PREREG — lane `w-guards`, 2026-08-16

    Lane:     w-guards
    Branch:   wt-w-guards   (namespace checked — see §0)
    Base:     master 202bfc3f
    Kind:     Construct rung — builds guards for three unguarded census surfaces
    Frozen:   before the first `crates/` change in this worktree

**This file is frozen. Nothing below is edited after the first `crates/` change.**
Corrections land in the rung doc as corrections, never here.

---

## 0. Namespace checks, done before the branch was cut

`w-bind16` §12 records a branch-name collision that **no audit in this repo can
see** — `gate.sh` hashes `crates fixtures scripts`, `board_audit.sh` audits the
board, `rung_registry` reads header blocks, and none of the three can see a lane
landing on top of a merged branch. So both namespaces were checked by hand
first, and the result is recorded here rather than assumed:

| namespace | check | result |
|---|---|---|
| branch | `git branch -a \| grep -i guard` at master `202bfc3f` | **empty** — `wt-w-guards` is free |
| rung slug | `ls docs/rungs/ \| grep -i guard` | **empty** — slug `guards` is free |

---

## 1. The question

Board **#3199** (`docs/rungs/2026-08-16-bind.md` §8) registered four mutants and
**three came back GREEN against a registered RED**:

| id | site | mutation | #3199 result |
|---|---|---|---|
| **M1** | `crates/c2-il/src/func/body/shapes/calls.rs:431` | `syms > 1` → `syms > 2` | **GREEN** |
| **M2** | `crates/c2-core/src/codegen/calls.rs:1815` | `count != 1` → `count > 2` | **RED** |
| **M3** | `crates/c2-il/src/func/bind.rs:886` | drop `resolve_data`'s `extern_data` linkage gate | **GREEN** |
| **M4** | `crates/c2-il/src/func/census.rs:1211` | swap `DATA_SYM_UNRESOLVED` ⇄ `DATA_SYM_LINKAGE` | **GREEN** |

M4 is the worst-placed: **the two census keys #3177's ranking is built from can
be exchanged with nothing failing**, in a **machine-read** instrument. `w-loo`
separately measured that five of six published rankings carry no information
(ρ ≈ +0.047, **#3135**) — an unguarded key in a machine-read instrument is
exactly how that happens without anyone noticing.

**This lane builds the missing guards.** The standard is not "a test exists": it
is **mutate, watch it go RED, revert, report the mutation**.

---

## 2. Populations and denominators — one denominator beside every numerator

Every figure below was **re-measured in this worktree at base `202bfc3f`**, not
handed down. `w-vocabgap`'s house rule exists because **eleven** handed-down
figures were caught wrong in one wave; `w-bind16` §10.1 is the twelfth
(briefed 370, measured 394).

| tag | population | denominator | measured at `202bfc3f` |
|---|---|---|---|
| **P-W** | 878-TU **workload scan** | **878** TUs | `match` **25** · `mismatch` **0** · `codegen-gap` **0** · `vocab-gap` **845** · `capture-fail` **8** · **878** verdict lines |
| **P-E** | **emitted functions** in P-W | **162,049** | `fnbyte-exact` **35,734** · `fnbyte-refused-parse` **113,612** |
| **P-K** | **`gap-metric` keys** on the P-W scan | — | **394** distinct keys over **394** lines, **prefix-anchored** `^ +gap-metric ` |
| **P-T** | **portable test lane**, `cargo test --workspace --release --no-fail-fast` | **42** targets | **1,643** passed / **0** failed |
| **P-F** | **fixture gate** — `gate.sh`, 381 fixtures × 18 mode lanes | **381**/lane | graded tree `75864f22df31`, **731** files (master's registered value) |
| **P-M** | **modeled-reachable** subset of P-E (`emit-cflow-modeled-key\|*`) | **3,062** fns / **30** keys | head 801 / 529 / 495 / 464 (measured by `w-bind16` at `55933035`; **not re-derived here** and quoted as theirs) |

**On P-K, three traps, all of them #3181/#3199's and all of them respected here:**

1. `grep "gap-metric"` is a **substring match on a namespace over a log that
   documents its own keys** — it counts two prose lines. **Anchor to the
   prefix.**
2. **Two real keys carry STRING values** (`.rdata$r`,
   `src/system/rndobj/wordwrap.cpp`) and **two more carry FLOATS** (`0.21016`,
   `0.22053`), so a *"must end in a number"* filter drops four in two opposite
   directions.
3. **`emit-cflow-modeled-key|*` is NOT a `gap-metric` line at all.** It is a
   JSONL `emit`-map key with **no printer** (`report.rs:230`
   `cflow_emitted_modeled_keys`). **No guard in this lane may depend on that key
   being printed** — it would silently guard nothing.

---

## 3. Where the guards go, and the ownership fence

**This lane owns `crates/c2-harness/src/gap/` and nothing else.** Peer
`w-section` is single-occupancy in `crates/c2-core/src/coff/`; peer `w-three`
owns `crates/c2-il` and `crates/c2-core/src/codegen/`. **All three mutation
sites are in territory this lane does not own** — M1 and M3 and M4 are all in
`crates/c2-il`.

**That is the constraint the guards are designed around, and it is registered as
a constraint and not smoothed over:** the guards are written in
`crates/c2-harness/src/gap/tests.rs` (`#[cfg(test)]`, so the release binary
cannot move) and reach the three surfaces through the **public** entry point
`IlBundle::census_functions()`, driving hand-built synthetic `.ex` + `.gl`
bundles. `parse_segment_detail`, `DATA_SYM_UNRESOLVED`, `DATA_SYM_LINKAGE` and
`gl_extern_data_names` are all `pub(crate)` in `c2-il` and are **not** reachable;
the guard therefore asserts on the **observable key string** — `verdict.key()`,
which is verbatim what `scan.rs:654` concatenates into
`emit-cflow-modeled-key|{}`. Guarding the observable is the stronger form
anyway: it is the string the instrument publishes.

**A mutation is applied to `crates/c2-il` only as a reverted experiment inside
this lane's own private worktree, is never committed, and `git status` is
checked clean-to-base after every one.** No `crates/c2-il` byte is landed.

---

## 4. The cells, registered before they are built

Each guard needs a cell that **discriminates**. The registered construction is
the `??__E` dynamic-initializer transcript (`c2-il`'s own
`wr1_dyninit::TOMCRYPT_DYNINIT`, a real `/O1 /Oi /EHsc /GR` capture), whose body
decodes to `MultiArgTailCall { arg_sources: [SymAddr(0xF909), SymAddr(0xFC09),
Lit(0)] }`, re-hosted in a synthetic `IlBundle` with a **hand-built `.gl`**:

| cell | `.gl` for the first symbol token | registered key |
|---|---|---|
| **A** | **no record at all** — `resolve` returns `None` | `data-sym-unresolved` |
| **B** | a record, linkage byte **`01`** (defined here) — `resolve` returns `Some`, `resolve_data` returns `None` | `data-sym-not-extern` |
| **C** | a record, linkage byte **`02`** (undefined external) — both resolve | **neither** data-sym key |
| **D** | cell C's `.gl`, body edited so the LO is **composed** (`4C 4F 11`) not bare, so `body_start_is_bare` is false and `two_sym_thunk` cannot fire on `syms == 2` | `call-arg-multi-sym` |

**A and B differ in exactly one thing: whether `.gl` names the token.** That is
precisely the discrimination M4 destroys. **B and C differ in exactly one byte:
the linkage byte.** That is precisely the discrimination M3 destroys. **D is the
arity fence at rank 1 of #3177's reachable order, 1,296 functions.**

**Absence is not success** (~15 instances, twice inside the merge gate itself):
every guard **prints the count of discriminating cells it graded**, and asserts
that count is non-zero, so a guard whose bundle stopped producing a census row
fails loudly instead of passing vacuously.

**Each assertion carries a distinct failure message.** The lane-registry defect
— seven mutations all tripping one count, with the assertions behind it never
running — is the failure mode; a guard whose first assertion shadows the rest is
one guard, not four.

---

## 5. Mutants — every expected colour registered BEFORE any run

Probe: `cargo test --workspace --release --no-fail-fast`. Base **1,643 passed /
0 failed / 42 targets** (P-T, measured at `202bfc3f`). Each mutation is applied
to **exactly one site**, the site count is printed, and a patch that matches 0
or ≥ 2 sites **aborts** rather than running vacuously.

**The tree must be committed before any mutant runs.** `w-bind16` §8.1 had to
discard its first run: **M1 read RED off a stale `INDEX.md` fired by its own
uncommitted rung doc**, which — read at face value — would have confirmed a
registered RED that is really GREEN, *the flattering direction*. Every mutant
run in this lane is from a committed, `rung_registry`-2/2 tree.

### 5.1 Phase R — reproduce #3199's four on the UNGUARDED tree

| id | site + mutation | **registered** |
|---|---|---|
| **R1** | `c2-il .../shapes/calls.rs:431` `syms > 1` → `syms > 2` | **GREEN** |
| **R2** | `c2-core/src/codegen/calls.rs:1815` `count != 1` → `count > 2` | **RED** |
| **R3** | `c2-il/src/func/bind.rs:886` drop `extern_data.contains(&name)` | **GREEN** |
| **R4** | `c2-il/src/func/census.rs:1211` swap `DATA_SYM_UNRESOLVED` ⇄ `DATA_SYM_LINKAGE` | **GREEN** |

A phase-R result that disagrees with #3199 is reported as a **failure to
reproduce** and the guards are re-derived against what this tree actually does.

### 5.2 Phase G — the same four WITH the guards

| id | site + mutation | **registered** |
|---|---|---|
| **G1** | R1's | **RED** |
| **G2** | R2's | **RED** — must **stay** red; a guard that unpinned M2 is a regression |
| **G3** | R3's | **RED** |
| **G4** | R4's | **RED** |

### 5.3 Phase N — controls that must fire, so the guards are not decoration

| id | what is perturbed | **registered** |
|---|---|---|
| **N0** | nothing — clean tree, guards in | **GREEN** (identity control) |
| **N1** | **cell B's `.gl` linkage byte `01` → `02`** in the guard's own fixture, assertions untouched | **RED** — proves the assertion discriminates on *input content*, not on its own spelling |
| **N2** | **cell A's `.gl` record added back** in the guard's own fixture, assertions untouched | **RED** — same, for the unresolved/named axis |

**N1 and N2 mutate the INPUT, never the oracle.** #3174 (`w-json2`) is the
precedent: its first mutant spelling **rewrote the assertion it was testing** and
therefore read green. A control that edits the assertion proves nothing.

---

## 6. Predictions — probability form, denominator beside each

| id | prediction | denominator | P |
|---|---|---|---|
| **P1** | A hand-built synthetic `IlBundle` driven through the **public** `IlBundle::census_functions()` yields a census row at all (non-empty, one row) | 1 of 1 cell | **0.70** |
| **P2** | Cell **A** (token unnamed in `.gl`) keys `data-sym-unresolved` | 1 of 1 | **0.65** |
| **P3** | Cell **B** (named, linkage `01`) keys `data-sym-not-extern` | 1 of 1 | **0.60** |
| **P4** | Cell **C** (named, linkage `02`) keys **neither** data-sym key | 1 of 1 | **0.70** |
| **P5** | Cell **D** keys `call-arg-multi-sym` | 1 of 1 | **0.55** |
| **P6** | **G4 goes RED** — the key swap is caught | 1 of 1 mutant | **0.90** *conditional on P2 ∧ P3* |
| **P7** | **G3 goes RED** — the linkage gate is caught | 1 of 1 | **0.85** *conditional on P3 ∧ P4* |
| **P8** | **G1 goes RED** — the arity fence is caught | 1 of 1 | **0.80** *conditional on P5* |
| **P9** | **G2 stays RED** | 1 of 1 | **0.95** |
| **P10** | Phase R reproduces #3199 exactly: R1/R3/R4 GREEN, R2 RED | 4 of 4 | **0.85** |
| **P11** | **Required-zero byte delta**: all **394** `gap-metric` keys identical, **878** verdict lines, `match` **25**, `mismatch` **0**, `codegen-gap` **0**, `vocab-gap` **845**, `capture-fail` **8**, `fnbyte-exact` **35,734**, `fnbyte-refused-parse` **113,612** at both ends | 394 / 878 | **0.97** |
| **P12** | The **release binary is byte-identical** at both ends (the guards are `#[cfg(test)]`) — base sha256 `9989b36c…` | 1 of 1 | **0.90** |
| **P13** | **P-T target count stays 42**, passed rises to **1,643 + k** with **4 ≤ k ≤ 12** | 42 targets | **0.80** |
| **P14** | At least one of the four surfaces proves **unguardable from `gap/`** and is reported as unguarded rather than faked | 0 of 4 | **0.30** |

**A GREEN in phase G is not a pass.** It is reported in these words: *the guard
this lane wrote does not reach that clause, and the clause is still unguarded.*

---

## 7. Ceiling — NO discount factor

This lane is a **construct rung**: `Fixtures: none`, `Census: +0`,
**required-zero byte delta**. Its ceiling on every graded column is **stated as
zero and claimed as zero**:

| column | ceiling |
|---|---|
| `match` (P-W) | **0 of 878** |
| `fnbyte-exact` (P-E) | **0 of 162,049** |
| `fnbyte-refused-parse` (P-E) | **0 of 113,612** |
| `mismatch` / `codegen-gap` | **0**, and any movement is **FAILED** |

**The deliverable's own ceiling, with its denominator: 3 of the 4 registered
surfaces** (M2 is already pinned), i.e. **3 GREEN→RED transitions out of 4
mutants**. There is no discount factor because there is nothing to discount: a
guard either fires on its mutation or it does not, and both outcomes are
reported.

**What a ceiling of 3/4 is worth, priced honestly and two-sided:** it converts
**zero** TUs and buys **zero** bytes. What it buys is that #3177's ranking — the
ranking that dispatched **this entire wave** — can no longer have its two
constituent keys silently exchanged, and that the rank-1 fence over **1,296
functions** can no longer be silently weakened. The cost side: **+k tests** of
maintenance and **4 synthetic byte transcripts** that must be kept in step with
`c2-il`'s `.ex`/`.gl` readers. If a future reader change breaks these guards,
that is the guard **working**, and the correct response is to re-derive the
transcript, not to delete the test.

---

## 8. What this lane will NOT do

* **Not** land one byte of `crates/c2-il`, `crates/c2-core/src/codegen/` or
  `crates/c2-core/src/coff/`. Mutations there are reverted experiments in a
  private worktree and `git status` is checked clean-to-base after each.
* **Not** add, rename or remove a `gap-metric` key. **394 at both ends.** Adding
  a printer for `emit-cflow-modeled-key|*` would move P-K and is therefore
  **out of scope**, however tempting #3199 finding-4 makes it.
* **Not** narrow, shadow or redefine any shared predicate. Three semantic
  collisions here had no textual conflict, one of which moved a scan key
  **88,894 → 1,474,755** with no compile error, no test failure and no gate red.
* **Not** claim a guard fires without having watched it fire, reverted, and
  printed the mutation.
* **Not** mint a board number. Rows land **unnumbered**; the coordinator
  serializes (next free **#3200**, two peers in flight).
