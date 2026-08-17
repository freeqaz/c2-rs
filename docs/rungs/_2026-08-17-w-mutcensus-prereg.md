# PREREG — w-mutcensus: a mutation census over the refusal/fence sites of `crates/c2-il`

Frozen BEFORE any mutant runs. Base: master `3835469c`. Lane branch
`wt-w-mutcensus`. Characterization lane, docs-only, Outcome target:
`instrument`. Required-zero byte delta on `crates/ fixtures/ scripts/`.

Provenance: #3199 (3 of 4 mutants GREEN-against-RED), #3212 (w-section's own
headline unguarded), w-guards' found-and-not-taken #2: "#3199's list was not
exhaustive and nothing enumerates fences with no test." This lane turns that
into a number: **X of N fence sites have no test that can fail on them.**

## 0. Populations at base, and where each figure was measured

| figure | value | measured |
|---|---|---|
| `cargo test --workspace --release --no-fail-fast` | **1,648 passed / 0 failed / 42 targets**, 3:43 wall | re-measured in this worktree at `3835469c` (work/w-mutcensus/baseline_test.log) |
| 878-TU scan | match 25 · mismatch 0 · vocab-gap 845 · fnbyte-exact 35,734 | briefed at `3835469c` (#3218 pointer read); will be re-measured for the end-state identity check |
| anchored `gap-metric` keys | 394 | briefed at `3835469c`; re-measured at end-state |

## 1. The enumeration rule (deliverable 1) — reproducible

`work/w-mutcensus/enumerate.sh`, run from the repo root at `3835469c`.
Candidate classes, over `crates/c2-il/src/**/*.rs`:

* **E1** — call-argument fences: every `refuse("<key>")` raise site.
  Measured: **23**, all in `func/body/shapes/calls.rs`.
* **E2** — named-key fence raises: every non-test, non-doc-comment line that
  raises one of the **19 fence-key constants** (`pub(crate) const *: &str` in
  `func/body/mod.rs` whose values are census refusal keys — the 24 string
  constants minus the 5 dispatch-state constants `DISP_NOT_RUN`/`PROD_*` and
  the 2 grammar-context constants `EXPR_TYPED_OP`/`CALL_IN_EXPR`).
* **E3** — post-parse gate raises: every `Block::at_end(` site.
* **Reading step (bounded, published):** within each function containing an
  E1–E3 site, every conditional that decides whether the raise fires is part
  of the site; plus the resolver/gate functions those sites call
  (`resolve_data`, `resolve_data_def`, `resolve_bss_def`, `is_varargs`,
  `gl_extern_data_names`, `NAME_SEPARATORS`, `opt_word_mode`) and the
  TU-level admission gates of `IlBundle` (`functions`, `dyninit_tu`,
  `data_tu`). This step is what turned the grep candidates into the site
  table in §2; every site carries file:line at `3835469c`.
* **E5 — enumerated and EXCLUDED from mutation** (deliverable 4; no silent
  caps — the drop is here, with counts and reasons):
  * grammar fail-closed sites: `blk(` **1,227** raw grep lines, `blk_type(`
    **6**, `Block::refuse(` **106** (counts overlap; they include the helper
    definitions and test uses). These are the byte-level parser refusals that
    key `expr-op-0xNN`-family census buckets. Dropped because (a) one full
    suite run per site ≈ 1,227 × ~6 min ≈ 5 days of wall-clock, and (b) they
    are a different guard class — the key is generated from the blocking
    byte, so a "key swap" mutation does not exist and a removal mutation
    merely moves the parse to the next blocking byte. A sampled census over
    them is a future lane.
  * shape-file `OptWordMode` admission predicates: **18** non-test
    comparison sites across `func/body/shapes/*.rs` (mode-specific shape
    admissions). Dropped for budget; second-tier published-key proximity
    (their keys are in-class labels).
  * `IlBundle::dyninit_tu`: **12** `return None` clauses — **one** mutated
    (D1), 11 dropped for budget.
  * `IlBundle::data_tu`: **14** `return None` clauses — **one** mutated
    (D2), 13 dropped for budget.
  * `IlBundle::functions()` interior gates past the three mutated here
    (label-counter gate, unclaimed-`.gl` accounting, `Bindings::selective`'s
    fail-closed binding): enumerated by reading, dropped for budget.
  * `STORE_RUN_BIND_CALL_TAIL_RETIRED`: **zero live raise sites** (test-only
    since #1212's correction) — a fence key with no fence; no mutant possible.

**N for the headline = the 63 mutated c2-il sites in §2.** The dropped
classes above are published beside it, each with its count, so the headline
cannot silently stand in for "all fences".

## 2. The mutants — every registered colour, frozen before any run

Probe: `cargo test --workspace --release --no-fail-fast`, 42 targets,
baseline **1,648 / 0** re-measured at `3835469c`. GREEN = no test fails
(unguarded). RED = ≥1 test fails (guarded; failing tests recorded). INVALID =
build error or missing target — not a colour, the recipe gets fixed and rerun.
Exact patches: `work/w-mutcensus/mutants.py` (line-pinned, aborts unless the
site text matches exactly; all 64 specs verified pristine-matching before this
freeze). Runner: `work/w-mutcensus/run_mutants.sh` — refuses a dirty tracked
tree before each mutant, reverts and re-verifies `crates/` clean after each
(the w-bind16 stale-state hazard, closed structurally).

Controls mutate the **port or the input, never the oracle** (#3174): every
spec touches `crates/c2-il` or `crates/c2-core`; none touches
`c2-reference`, wibo, or a capture.

### 2.1 Controls (w-guards' four surfaces + the M2 backstop) — all registered RED

| id | site (at `3835469c`) | mutation | reg. colour | P |
|---|---|---|---|---|
| N0 | none — clean tree | none | GREEN 1,648/0 | 0.97 |
| C1 =M1 | `calls.rs:431` arity fence | `syms > 1` → `syms > 2` | **RED** | 0.97 |
| C2 =M2 | `c2-core/codegen/calls.rs:1815` backstop | `count() != 1` → `> 2` | **RED** | 0.95 |
| C3 =M3 | `bind.rs:890` `.gl` linkage gate | `.contains(&name) \| true` | **RED** | 0.97 |
| C4 =M4 | `census.rs:1216/1218` data-sym keys | swap the two constants | **RED** | 0.97 |
| C5 | `calls.rs:430` thunk exemption | `false &&` | **RED** | 0.90 |

Any control reading GREEN is a finding that outranks the census (it means a
w-guards guard has stopped firing) and stops the campaign for triage.

### 2.2 The census sites (59), registered colour + probability each

| id | site | mutation | reg. | P |
|---|---|---|---|---|
| CS2 | `census.rs:1242` store-run-call routing | key → STATIC_SCAN_LOOP_OBJECT | GREEN | 0.75 |
| CS3 | `census.rs:1245` static-scan-loop routing | key → STORE_RUN_CALL_NO_CARRIER | GREEN | 0.75 |
| CS4 | `census.rs:1263` bind_key routing | drop `bind_key.unwrap_or` | GREEN | 0.65 |
| CS5 | `census.rs:1265` framed-call routing | key → CALLEE_UNRESOLVED_TAIL | GREEN | 0.70 |
| CS6 | `census.rs:1267` call-sequence routing | key → CALLEE_UNRESOLVED_TAIL | GREEN | 0.70 |
| CS7 | `census.rs:1270` empty-dtor routing | key → CALLEE_UNRESOLVED_TAIL | GREEN | 0.70 |
| CS8 | `census.rs:1272` default (tail) routing | key → CALLEE_UNRESOLVED_FRAMED | **RED** | 0.80 |
| CS9 | `census.rs:1280` opt-mode gate | `false &&` | **RED** | 0.60 |
| CS10 | `census.rs:1294` ptr-walk-not-O1 gate | `false &&` | **RED** | 0.60 |
| CS11 | `census.rs:1306` chain-not-O1 gate | `false &&` | **RED** | 0.60 |
| CS12 | `census.rs:1358` inline fence (callee-defined-in-tu) | `false &&` | **RED** | 0.90 |
| CA2 | `calls.rs:434` sym-overflow | threshold +9 | GREEN | 0.80 |
| CA3 | `calls.rs:442` sym-permuted | `false &&` | GREEN | 0.75 |
| CA4 | `calls.rs:529` lit-over-eight-slots | threshold +9 | GREEN | 0.80 |
| CA5 | `calls.rs:593` lit-permuted | `false &&` | GREEN | 0.70 |
| CA6 | `calls.rs:693` nonformal (slot arm) | key → call-arg-computed | GREEN | 0.50 |
| CA7 | `calls.rs:699` lit-wide | `false &&` | GREEN | 0.75 |
| CA8 | `calls.rs:710` computed | key → call-arg-nonformal | GREEN | 0.70 |
| CA9 | `calls.rs:732` lit-classified-twice | key swap | GREEN | 0.90 |
| CA10 | `calls.rs:736` sym-classified-twice | key swap | GREEN | 0.90 |
| CA11 | `calls.rs:747` outer-formal (panic guard) | `false &&` | **RED** | 0.70 |
| CA12 | `calls.rs:759` duplicated | `false &&` | GREEN | 0.70 |
| CA13 | `calls.rs:772` source-out-of-slots | key → call-arg-outer-formal | GREEN | 0.80 |
| CA14 | `calls.rs:774` multicycle | `> 1` → `> 9` | GREEN | 0.70 |
| CA15 | `calls.rs:780` long-cycle | threshold +9 | GREEN | 0.75 |
| CA16 | `calls.rs:792` repeated-leaf | `false &&` | GREEN | 0.70 |
| CA17 | `calls.rs:800` noncanonical-order (loads) | `false &&` | **RED** | 0.55 |
| CA18 | `calls.rs:803` noncanonical-order (chain) | `false &&` | GREEN | 0.55 |
| CA19 | `calls.rs:806` nonformal (post) | `false &&` | **RED** | 0.55 |
| CA20 | `calls.rs:868` mcall-chain overflow | threshold +9 | GREEN | 0.80 |
| CA21 | `calls.rs:878` mcall-chain nonformal | key swap | **RED** | 0.85 |
| CA22 | `calls.rs:883` mcall-chain lit-wide | `false &&` | GREEN | 0.80 |
| CA23 | `calls.rs:893` mcall-chain computed | key swap | **RED** | 0.85 |
| B2 | `bind.rs:929` data-def comdat/init | `false &&` | GREEN | 0.60 |
| B3 | `bind.rs:932` data-def thread-local | `false &&` | GREEN | 0.70 |
| B4 | `bind.rs:939` `.in` totality gate | `false &&` | GREEN | 0.65 |
| B5 | `bind.rs:942` `.in` refs gate | `false &&` | GREEN | 0.65 |
| B6 | `bind.rs:946` size-exact gate | `false &&` | GREEN | 0.60 |
| B7 | `bind.rs:985` bss-def comdat/init | `false &&` | GREEN | 0.60 |
| B8 | `bind.rs:988` bss-def thread-local | `false &&` | GREEN | 0.70 |
| B9 | `bind.rs:991` bss-def size==0 | `false &&` | GREEN | 0.70 |
| B10 | `bind.rs:862` varargs name gate | `false &&` | **RED** | 0.65 |
| G1 | `gl.rs:2188` extern-data linkage byte | `\|\| true` | **RED** | 0.90 |
| G2 | `gl.rs:2198` ambiguous-name refusal | retain → keep all | GREEN | 0.55 |
| G3 | `gl.rs:1085` NAME_SEPARATORS | drop `0x26` | **RED** | 0.85 |
| BU1 | `bundle.rs:1694` opt_word_mode table | unknown → `Some(Ox)` | **RED** | 0.70 |
| BU2 | `bundle.rs:1919` drectve gate (functions) | `false &&` | GREEN | 0.60 |
| BU3 | `bundle.rs:1940` empty-module LO probe | `\|\| true` | GREEN | 0.55 |
| D1 | `bundle.rs:2423` dyninit name clause | `false &&` | **RED** | 0.60 |
| D2 | `bundle.rs:2887` data_tu `.in` totality | `false &&` | GREEN | 0.55 |
| L1 | `leaf_store.rs:2254` group-shape (frame) | key → MULTI_PRODUCER | **RED** | 0.65 |
| L2 | `leaf_store.rs:2257` group-shape (base) | key → MULTI_PRODUCER | GREEN | 0.55 |
| L3 | `leaf_store.rs:2285` group-shape (value) | key → MULTI_PRODUCER | GREEN | 0.50 |
| L4 | `leaf_store.rs:2370` mixed-kind | `false &&` | **RED** | 0.80 |
| L5 | `leaf_store.rs:2374` addr-producer (disp==0) | `== 0` → `== i32::MIN` | **RED** | 0.60 |
| L6 | `leaf_store.rs:2390` multi-producer (lits) | `> 1` → `> 9` | **RED** | 0.75 |
| L7 | `leaf_store.rs:2399` pool bound | `false &&` | **RED** | 0.55 |
| L8 | `leaf_store.rs:2402` symbol-crossings | threshold +9 | **RED** | 0.75 |
| L9 | `leaf_store.rs:2455` group-shape (2nd walk) | key → MULTI_PRODUCER | GREEN | 0.60 |

### 2.3 The registered headline prediction

Over the **63** mutated c2-il sites (C1, C3, C4, C5 + the 59 above; C2 is the
out-of-crate backstop control and not in N):

* **X (GREEN, unguarded) = 38** point prediction; 80% interval **[30, 46]**.
* P(X ≥ 20) = 0.92 — i.e. this lane registers, before running, that the
  unguarded population is large; #3199's 3-of-4 was not an anomaly of four
  hand-picked sites.
* P(any of C1/C2/C3/C4/C5 reads GREEN) = 0.05 (if observed: finding outranks
  the census, campaign stops for triage).
* P(≥1 INVALID needing a recipe fix on first application) = 0.5 (recorded,
  fixed, rerun; not a colour).

No discount factor.

## 3. Budget, order, and what happens if the budget forces triage

Serial runs, ~6 min each (3:43 suite + rebuild) ≈ 6.5 h for 64 runs. Budget:
**9 h of mutant wall-clock**. Order, by published-key proximity (deliverable
5): controls → CS block (the keys the reachable ranking and the census tables
are built from) → G/BU/B/D blocks (the resolution gates those keys sit on) →
L block (#1199's four separately-sized residue keys) → CA block (the blocker
histogram's call-arg family). If the budget exhausts, every unrun row is
published as **NOT RUN** with its registered colour still in the table — a
dropped row is a logged row, never a silent cap.

## 4. Hygiene invariants (each checked by the runner, per mutant)

1. Tracked tree (`crates fixtures scripts docs`) clean before apply.
2. Patch applies with exactly-one-occurrence line match, or aborts.
3. After revert: `git status --porcelain -- crates fixtures scripts` empty.
4. Mutant logs live only under `work/w-mutcensus/results/`; no gate.sh, no
   scan, no instrument run happens while a mutant is applied.
5. A run with < 42 `test result:` lines is INVALID (absence is not success —
   STATUS trap 5), never GREEN.
