# PREREG — `w-witness7`, frozen before the first probe

    Lane:      w-witness7
    Base:      master `666fe6eb7`
    Date:      2026-08-18
    Kind:      construct — one behavioural witness per UNGUARDED refusal site
               `w-deadsites` proved reachable, plus the measurement that settles
               board **#3281**
    Frozen:    this file is the FIRST commit on `wt-w-witness7`, before any
               probe, any mutant and any suite run in this worktree

Every colour below is registered here **before** it is observed. The results
table in the rung is **derived from the run logs** under `work/w-witness7/logs/`
by `work/w-witness7/rederive.py`, never accumulated by hand
(`docs/rungs/README.md` probe rule 2).

---

## 0. The item

`w-deadsites` (board **#3276**) partitioned `w-mutcensus`' 26 open GREEN census
rows into **7 UNGUARDED / 4 DEAD-with-a-proof / 15 UNKNOWN**, and named the 7 as
the only population where writing a guard is both possible and cheap — the probe
already proved an input reaches each of them.

    CS3  census.rs:1288   "static-scan-loop" => STATIC_SCAN_LOOP_OBJECT
    CS4  census.rs:1306   bind_key.unwrap_or(…) with bind_key SOME
    CS9  census.rs:1323   the opt-mode gate
    CA6  calls.rs:693     call-arg-nonformal, the SLOT path
    CA8  calls.rs:710     call-arg-computed
    B2   bind.rs:974      resolve_data_def's comdat/initialized gate
    B7   bind.rs:1030     resolve_bss_def's comdat/initialized gate

It also published a contradiction it declined to resolve (**#3281**): `B9` reads
**RED** in `w-mutcensus` while `w-deadsites` measured its site
(`bind.rs:1036`, `if o.size == 0` in `resolve_bss_def`) **unreached** under two
screens and a `panic!()`. `B9`'s mutation is `false &&` on that condition, which
is a semantic no-op on a branch never taken — so it cannot be both RED and
unreached. Priced at one suite run.

## 1. The published key each site produces — registered from SOURCE, before any capture

Read off `crates/c2-il` at `666fe6eb7`, no capture run. A wrong entry here is a
**MISS**, scored as such.

| site | the published key when the site fires | how the key is observed |
|---|---|---|
| `CS3` | `static-scan-loop-object-out-of-class:eof` | `FnVerdict::key()` on a `static-scan-loop`-labelled body whose object is out of class |
| `B2`  | **the same key** — `resolve_data_def`'s `None` is *why* `shape_to_function` returns `None` for a `StaticScanLoop` | same cell |
| `CS4` | one of `store-run-bind-{mixed-kind-alloc,address-producer,multi-producer,symbol-crossings}:eof` — **never** `store-run-bind-no-emitter-carrier` | four functions of `fixtures/cpp/w1199_bind_run_neg.cpp` |
| `CS9` | `opt-mode-<8 hex digits>` (`Block::feature` renders `OPT_MODE` from `aux`) | a body in class captured at a mode word the port does not emit under |
| `CA6` | `call-arg-nonformal:mid` | a 2-argument tail call one of whose arguments is a non-formal load |
| `CA8` | `call-arg-computed:mid` | a 2-argument tail call one of whose arguments is a computed operand stream |
| `B7`  | `callee-unresolved-tail-call:eof` — `GlobalStoreLeaf`'s label is not one the census `match` names, so it takes the `_` arm | a global-store leaf whose destination object is COMDAT or initialized |

**P(all seven key strings registered above are the ones observed) = 0.45.** The
two most likely to be wrong are the `:mid`/`:eof` suffixes on `CA6`/`CA8` (P =
0.65 each that `:mid` is right) and `B7`'s `_`-arm reading (P = 0.70).

## 2. `call-arg-nonformal` has **FIVE** raise sites, and the witness is a TABLE

Counted by parsing `crates/c2-il/src` for `refuse("call-arg-nonformal")` and
`Block::refuse(…, "call-arg-nonformal")` — **5**, the largest `k` of any
refuse-literal key in the crate:

| row | site | the production it is inside |
|---|---|---|
| 1 | `calls.rs:693` | `tail_call_shape`, the **slot** path (≥ 2 args) — this is `CA6` |
| 2 | `calls.rs:807` | `tail_call_shape`, the **single-argument operand** path |
| 3 | `calls.rs:1749` | the **framed** post-op path (`g(x) + k`) |
| 4 | `mcall_cmp.rs:246` | the member-call **comparison** production's receivers |
| 5 | `mcall_tail.rs:673` | the member-call **framed** production's receiver |

`w-deadsites` closed `w-calleeguard`'s **P13** on a `k = 2` family. This is the
same form at `k = 5`, and each row's control is an **in-class sibling whose
census label names the production** — which is what pins the row to its site
rather than to the key.

**Registered: `H4` — rows closed of the five.** Point estimate **3**, interval
**[2, 5]**. Rows 4 and 5 need a member-call shape and are the two I expect to
miss.

## 3. Registered colours

### 3.1 The named control, `C1` — pinned BY NAME, run in every environment

`crates/c2-il/src/func/body/shapes/calls.rs:431`, `if syms > 1 && !two_sym_thunk`
→ `if syms > 2 && !two_sym_thunk`.

Registered **RED**, failing **exactly**:

* `the_call_argument_arity_fence_is_a_series_and_admits_exactly_one_symbol`
* `the_two_symbol_thunk_exemption_turns_on_the_bare_body_marker_alone`

Both are capture-driven, so a worktree whose captures were skipping cannot
produce this pair. Run **before the first guard lands** (`C1a`) and **after the
last** (`C1b`). Per-run log files are keyed on the **run id**, not on the mutant
— `w-deadsites` D6 overwrote two of its three `C1` logs by keying on the
mutation.

**Stop rule.** If `C1` is not RED with that exact pair, every colour taken in
that environment is **void, not provisional**: discarded, re-run, invalid log
kept.

**Environment assertions on every suite run**, per `docs/rungs/README.md` probe
rule 1: `C2RS_REQUIRE_TOOLCHAIN=1`, the executed-test count recorded, and the
`census_gate` target's **duration** recorded and required to be **> 10 s** (an
ungraded run reads 0.00 s).

### 3.2 The seven guards, each registered GREEN → RED

For each site: `Mn` is `w-mutcensus`' own registered mutation, replayed at this
base. `BASE` is its colour on the tree as it stands at `666fe6eb7`; `TIP` is its
colour with this lane's guards in the tree.

| id | site | mutation | BASE registered | TIP registered | P(TIP RED) |
|---|---|---|---|---|---|
| `M-CS3` | `census.rs:1288` | `"static-scan-loop"` arm → `STORE_RUN_CALL_NO_CARRIER` | **RED** — `w-deadsites`' `MC3`, caught by `tests/fence_site_census.rs` alone | RED | 0.95 |
| `M-CS3B` | `census.rs:1288` | the match **label** `"static-scan-loop"` → `"static-scan-loop-x"`, so the arm is never selected and the body falls to `_ => CALLEE_UNRESOLVED_TAIL`. **The constant and its one raise site do not move**, so the source-text census cannot see it | **GREEN** | RED | 0.80 |
| `M-CS4` | `census.rs:1306` | drop `bind_key.unwrap_or` → always `STORE_RUN_BIND_NO_CARRIER` | GREEN | RED | 0.80 |
| `M-CS9` | `census.rs:1323` | `false &&` on the opt-mode gate | GREEN | RED | 0.65 |
| `M-CA6` | `calls.rs:693` | key `call-arg-nonformal` → `call-arg-computed` (slot arm) | GREEN | RED | 0.80 |
| `M-CA8` | `calls.rs:710` | key `call-arg-computed` → `call-arg-nonformal` | GREEN | RED | 0.85 |
| `M-B2` | `bind.rs:974` | `false &&` on the data-def comdat/initialized gate | GREEN | RED | 0.65 |
| `M-B7` | `bind.rs:1030` | `false &&` on the bss-def comdat/initialized gate | GREEN | RED | 0.70 |

`M-CS3B` is the row that decides **deliverable 2**. `CS3` is the one of the
seven that already reads RED at this base — and its **sole** catcher is
`tests/fence_site_census.rs`, which reads source text and never runs a compiler.
A count-shaped source census is blind to a change that leaves every count where
it was and moves what a body reports; `M-CS3B` is exactly that change.
**Registered: `H5` — `M-CS3B` is GREEN at base, P = 0.80.** If it is RED at
base, `CS3` is genuinely guarded and deliverable 2's answer is "none of the
seven is guarded only incidentally".

### 3.3 `H1` — how many of the seven close

**Point estimate 6 of 7**, interval **[4, 7]**. The one I expect to miss is
`CS9`: the opt-mode gate needs a body that parses **in class** at a mode word
the port refuses, and a `/Od` capture may not parse as any modelled shape at
all, in which case the gate is never reached and there is no cell.

Secondary registration, `H1b`: **at least one** of the seven turns out to need
a `crates/c2-il` edit to guard from the harness side. **P = 0.20** — the brief
records this outcome registered at 0.30 before and found false every time, and
`w-guards` and `w-calleeguard` both landed zero bytes in `c2-il`.

### 3.4 `H2` — board **#3281**, which instrument is wrong

**Registered: `w-mutcensus`' `B9` RED is the defect; `w-deadsites`' UNREACHED
screen is right. P = 0.70.**

The mechanism I register in advance: `B9`'s sole guard,
`reloc_identity::the_cells_population_is_three_functions_one_of_which_disagrees`,
fails for **capture/load** reasons rather than for the mutation, so its RED is
not attributable to `bind.rs:1036` at all.

Registered rivals, so the finding cannot be a story fitted afterwards:

* **P = 0.20** — the site IS reachable and `w-deadsites`' screen missed it,
  because the same test's capture yielded nothing in the *probe* run too. The
  two readings would then be consistent and both under-informative. **This is
  distinguishable and the distinguishing measurement is registered**: run
  `reloc_identity` alone at this base, in a provisioned worktree, and record
  whether it grades a non-empty population.
* **P = 0.10** — something else: the mutation is not the no-op it reads as, or
  `w-mutcensus`' recorded line is not the site `w-deadsites` re-located.

**The deliverable is naming which instrument is wrong, not reconciling the two
into a story.** If the measurement supports neither reading cleanly, this lane
says so.

Registered observable: `M-B9` (`false &&` on `bind.rs:1036`'s `if o.size == 0`)
run at this base, **twice**, and `reloc_identity` run alone on the **clean**
tree. Registered colours: `M-B9` **GREEN** (P = 0.70); `reloc_identity` on the
clean tree **PASSES with a non-empty graded population** (P = 0.75).

### 3.5 `H3` — the census re-measured after the guards

Of the 26 open GREEN rows, how many are still GREEN at this lane's tip.
Registered: **20**, interval **[19, 22]**, partitioned **UNGUARDED 1 ·
DEAD-with-a-proof 4 · UNKNOWN 15**. The base is 25 GREEN, not 26 — `CS3` is
already RED on master through `fence_site_census.rs`. Only the seven UNGUARDED
rows are re-run; the other 19 are sites this lane does not touch and re-running
them would be re-measuring `w-deadsites`.

### 3.6 `H6` — the suite

Ends at **1,671 + k**, `4 ≤ k ≤ 14`. A suite at exactly 1,671 means nothing can
fail on the new guards and the lane FAILED whatever else it produced.

### 3.7 `H7` — the graded figures do not move

`scripts/gate.sh --jobs 16 --require-graded` **PASS** at both ends, per-lane
count identity diff **0 rows differing with the range LENGTH asserted**; the
878-TU scan identical over all **394** prefix-anchored `gap-metric` keys
(`grep -cE '^ *gap-metric \S+ \S+$'`, never the naive `grep -c` → 396);
`fnbyte-exact` read back-to-back at base and tip per **#3249**.

**And that zero is registered here as evidence about REACH and nothing else**
(`docs/rungs/README.md`, `w-sizebracket` **#3270**–**#3275**). The correctness
claim of this lane is the **GREEN → RED demonstration**, not the delta.

## 4. Stop rules

1. **A quiet `C1` voids the environment.** Colours taken there are discarded and
   the invalid log is kept.
2. **A `census_gate` duration under 10 s voids the run.**
3. **No `crates/c2-il` byte lands.** If a site cannot be guarded from the
   harness side against the public API, it is recorded **unguardable-from-here**
   with the evidence, and the lane stops rather than crossing the seam.
4. **Every probe patch is verified reverted** (`git status --porcelain`
   over `crates/` empty) before the next run.
5. **The results table is re-derived from the logs**, and a log that was
   overwritten is a defect this lane names rather than a row it fills from
   memory.

## 5. What this lane will NOT do

* It will not delete or "clean up" any of the 15 UNKNOWN rows. Deleting a site
  because a probe did not reach it is the error `w-deadsites` H11 was registered
  against.
* It will not quote a `fnbyte-*` delta as evidence a guard is correct.
* It will not summarize the guards by a count without saying what the count
  cannot see.
