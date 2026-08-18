# PREREG — `w-grammarscreen`

    Lane:   w-grammarscreen
    Kind:   characterization
    Base:   master `666fe6eb7`
    Frozen: this file is committed BEFORE the first probe is applied. Nothing
            below is edited afterwards; deviations go in
            `work/w-grammarscreen/deviations.md`.

The question: **`w-mutcensus` §2.1 dropped a 1,227-site grammar class with a
count. `w-deadsites` F1 said its screen makes that class affordable. Run it —
and take `w-deadsites`' THREE-bucket partition (unguarded / dead / unknown) on
it, not the two-bucket one `#3246` named.**

---

## 1. The enumeration rule — PARSED, not grepped (board #3288)

**#3288 is directly this lane's problem**: three enumerators in this repo are
wrong the same way, each under-counting silently and flatteringly, and the
`1,227` this lane inherits **is itself a raw grep result**
(`grep -rn -F 'blk(' crates/c2-il/src --include='*.rs' | wc -l`).

The rule is `work/w-grammarscreen/enumerate.py`, a hand-rolled Rust lexer. It
skips line comments, doc comments, **nested** block comments, string literals
(normal, escaped, byte, and raw with any hash count), char literals and
lifetimes, then finds the CALL TOKEN SEQUENCES

    `blk` `(`   |   `blk_type` `(`   |   IDENT `::` `refuse` `(`

in what is left, excluding the `fn` DEFINITION of each. It records, per site,
`file`, `line`, **`col`**, the `ctx` argument (literal or path), the syntactic
`form` the constructed `Block` is consumed in, and whether the site is inside a
`#[cfg(test)]` module.

**Second, differently-built count, per #3288's own transferable rule.** The
parsed site set is diffed line-for-line against the raw fixed-string grep line
set, in both directions, and every difference is named. That reconciliation is
part of the deliverable and its result is stated below rather than left to be
discovered.

### 1.1 What the enumeration ALREADY says — measured before this file was frozen

Stated here as a **measurement taken before freezing**, not as a prediction, so
it cannot be scored as a prereg hit:

| population | raw grep lines | **parsed call sites** | reconciliation |
|---|---:|---:|---|
| `blk(` | **1,227** | **1,225** | grep-only 2: `expr.rs:1357` (a **doc comment** quoting `blk(seg, p, "body")`) and `mod.rs:1856` (the `fn blk(` **definition**). parse-only **0**. **No line carries two sites** |
| `blk_type(` | **6** | **5** | grep-only 1: the `fn blk_type(` definition |
| `Block::refuse(` | **106** | **106** | identical line sets both ways; **1 of the 106 is inside `#[cfg(test)]`** |
| **total** | 1,339 | **1,336** (1,335 production + 1 test) | |

**So `1,227` moves to `1,225`, and it moves DOWN by exactly two nameable
lines.** The interesting half is what did *not* happen: the multi-site-per-line
failure mode — the one that makes a line grep an under-count — **is zero here**,
which is a fact this population happens to have and not a property of grep.

### 1.2 The frame — what is in, what is out, with counts (no silent caps)

**IN — the primary frame, covered in FULL. No sampling, no stride, no cap.**
All **1,336** sites. The screen is site-keyed and costs three one-line source
insertions, so covering the whole population costs the same as covering one
site; a sample would be a worse instrument at the same price.

**OUT, enumerated and published with counts** (`w-mutcensus` §2.1's discipline):

| dropped class | count | why out of frame |
|---|---:|---|
| shape-file `OptWordMode` admission predicates | `w-mutcensus` says **18** — re-derived by parsing in §6 | not a constructor call; screening a comparison operator needs a per-site boolean wrapper, a different instrument |
| `IlBundle::dyninit_tu` `return None` clauses | `w-mutcensus` says **12** | same — `return None` carries no callee to hang `#[track_caller]` on. **Conditional secondary frame**, see §5 |
| `IlBundle::data_tu` `return None` clauses | `w-mutcensus` says **14** | same |
| `Block::at_end(` sites | E3 of `w-mutcensus`' 63 | already mutated there; not this class. **Note:** `Block::at_end` is implemented as `Block::refuse(seg, seg.len(), ctx)`, so `mod.rs:1597` **is** one of this lane's 106 sites and fires whenever any `at_end` site does. Recorded, not hidden |

---

## 2. The screen — how 1,336 sites are covered by THREE source lines

`w-deadsites` screened 34 sites with a 34-entry bitmask and one `hit(ix, "ID")`
call **per site**; F1 sized 1,227 sites at "20 bitmask words, ~20 runs".

That is not necessary. `blk`, `blk_type` and `Block::refuse` are the **only**
constructors of this class, so marking those three `#[track_caller]` and asking
`std::panic::Location::caller()` who called them identifies the **exact call
site**, file/line/**column**, with no per-site edit at all. One
`&'static Location` exists per call site, so its address is the site key and the
first-hit dedup is a thread-local `HashSet<usize>`.

**Cost: one instrumented build, and the whole 1,336-site population in one
corpus pass.** Not 20 runs, and not 5 days.

**Behaviour-preserving by construction.** `hit` returns `()`, touches no program
state, and reads only an environment variable. The instrumented run must
reproduce the clean baseline's suite / gate / scan counts **exactly**; that
identity is the instrument's own validity check (`w-deadsites` §3.1), and it is
registered as **H3** below.

### 2.1 A hit means EVALUATED, and only sometimes REFUSED

`.ok_or(blk(..))` evaluates its argument **eagerly**, so at those sites a hit
says "control reached this expression", not "this site refused". The
enumeration therefore records `form` per site and the reach table is split by
it. The asymmetry runs in the safe direction and is stated once:

> **evaluated ⊇ refused, so QUIET IS SOUND FOR EVERY FORM** — a site the probe
> never records cannot have refused either. Only the *reached* bucket carries
> the caveat.

### 2.2 Reach is attributed to a corpus STAGE — `w-deadsites` F2, taken

F2 records that lane recording *that* a site fired and not *where*, so all seven
of its UNGUARDED rows were priced the same. Here `C2RS_GRAMMARPROBE_LOG` is a
**fresh file per stage**, and the stages are run separately:

    suite   cargo test --workspace --release --no-fail-fast (C2RS_REQUIRE_TOOLCHAIN=1)
    bench   ./target/release/c2rs bench            (the fixture gate)
    sweep   scripts/expr_sweep.sh                  (the generated corpus)
    cross   scripts/mode_cross.sh                  (the mode cross)
    debug   scripts/debug_lane.sh                  (the debug-profile lane)
    gate    scripts/gate.sh --jobs 16 --require-graded --allow-dirty-crates
    scan    the 878-TU workload scan

The union is REACHED; the per-stage sets are the price of a witness.

### 2.3 The confirmation — `#3246`'s `panic!()`, batched over ~all quiet sites at once

Second mode of the same module: given the file of REACHED sites, any site NOT
in it `panic!()`s and names itself. A run that completes clean confirms **every
quiet site simultaneously**. Markers are grepped from the **raw text** of every
log, never inferred from an exit code (`w-deadsites` §3.2 — the gate has a
`panics=` column and a caught-and-counted panic leaves no trace in a status).

### 2.4 Probe soundness — the two standing rules

1. **A control pinned by NAME, re-run in every environment.** `C1` =
   `crates/c2-il/src/func/body/shapes/calls.rs:431`, `syms > 1` → `syms > 2`.
   Registered failing pair, by name:
   `the_call_argument_arity_fence_is_a_series_and_admits_exactly_one_symbol`
   and `the_two_symbol_thunk_exemption_turns_on_the_bare_body_marker_alone`.
   Run at the clean base and again at the tip.
2. **Every suite run carries `C2RS_REQUIRE_TOOLCHAIN=1`** and records the
   `census_gate` target's **duration**. Any run whose differential is under
   1 s is **INVALID, not provisional** — discarded, re-run, log kept.
3. **The results table is DERIVED from the logs** by
   `work/w-grammarscreen/rederive.py`, never accumulated.
4. **Every probe is applied and verified reverted**; the patcher refuses to
   start on a dirty tree and asserts a **unique** textual anchor per edit.

---

## 3. Registered predictions

Calibration note taken from `w-deadsites` §8.1 and applied **before** these
numbers were written: that lane's probe misses were **8 of 10 in one
direction**, the registration over-estimating how much of `c2-il`'s refusal
surface this corpus touches, and its explicit advice was *"a future lane probing
the 1,227-site grammar class should register LESS reach than intuition
suggests, not more."* Intuition here said ~55 % reached; the registered figure
is **40 %**.

| id | registration | P |
|---|---|---|
| **H1** | **REACHED = 534 of 1,336 (40 %)**, 80 % interval **[400, 735]** = [30 %, 55 %] | — |
| **H2** | **QUIET = 802 (60 %)**, and the quiet fraction here is **HIGHER than `w-deadsites`' 73 %-of-open-GREEN is low** — i.e. the "measured corpus reach, not test quality" reading **generalizes to this class and gets stronger**, because the grammar sites are deeper in the parsers than the census fences are | 0.75 |
| **H3** | the instrumented run reproduces the clean baseline **exactly**: suite `1,671 / 0 / 46`, gate PASS 18/18 with identical sweep / cross / verdict counts, 878-TU scan identical on all 394 anchored keys | 0.90 |
| **H4** | **P1 and P2 — two independent screen runs — agree on every one of the 1,336 rows** | 0.80 |
| **H5** | the `panic!()` confirmation run completes with **zero** `w-grammarscreen QUIET SITE REACHED` markers in the raw text of every log | 0.75 |
| **H6** | control `C1` is **RED** with exactly the two named tests, at the base and at the tip | 0.95 |
| **H7** | stage attribution is **informative**: at least one site is reached by the 878-TU scan and by **no other stage**, and at least one is reached by the suite and by no other stage | 0.90 |
| **H8** | **DEAD (quiet AND a source-level proof) ≤ 27 sites (2 % of the population)**, and the reason is structural, not budgetary: a per-site unreachability proof does not scale to ~800 quiet sites, so only class-level mechanical arguments are available | 0.80 |
| **H9** | **UNKNOWN dominates**: it is the largest of the three buckets, > 50 % of the population | 0.75 |
| **H10** | **the three-bucket partition DEGENERATES on this class** — with DEAD tiny and UNKNOWN dominant, the actionable content of the screen is the *reach* attribution (§2.2), not the partition. Registered as the lane's expected structural finding | 0.80 |
| **H11** | **the lane deletes NOTHING and publishes the UNKNOWN bucket untouched.** This is the failure this lane is most likely to commit and the brief names it: *deleting a site because a probe did not reach it is the error this project keeps making* | 0.97 |
| **H12** | `git diff master..HEAD -- crates fixtures scripts` is **EMPTY** at the tip; graded tree identical at both ends (this is a revert-everything lane, so #3215's exclusion does **not** apply and the row must be recorded) | 0.95 |
| **H13** | a **contradiction** between the probe and the independently-constructed `ctx` cross-check of §4 — a `ctx` the production census reports as a blocking feature on the workload whose only site the probe recorded quiet | 0.15 |
| **H14** | at least one shape file has **zero** sites reached by any stage — a whole parser the corpus never enters past its first gate | 0.35 |

### 3.1 Invalidation / stop rules

* A suite run whose `census_gate` differential is **< 1 s** voids that run.
* Baseline suite **≠ 1,671 / 0 / 46** at the clean base voids the frame; the
  lane re-measures and reports before continuing.
* A **quiet control** (§2.4's `C1` reading GREEN) voids the campaign and
  outranks every other finding.
* If the `panic!()` run fires at a site the screen called quiet, the screen is
  wrong and **the whole reach table is re-derived**, not patched.
* If a peer lands a `crates/c2-il` change that moves a site mid-lane, the frame
  is **NOT re-enumerated** (that would unfreeze this prereg). It is recorded as
  a site the frame necessarily misses and as evidence of the enumeration's
  shelf life — `w-mutcensus`' went stale **twice inside one lane's wall-clock**.

---

## 4. The independent cross-check — the production census's own vocabulary

`Block::feature()` renders `ctx` into the census key the 878-TU scan already
publishes. So *"which refusal actually blocked a body"* is measured on master,
by production code, with no probe at all. Intersecting that vocabulary with the
probe's reached set is a **second, differently-constructed count** of reach over
the same population — exactly what #3288 asks any published denominator to
carry. It is a partial check by construction (a ctx is not a site: **38 ctx
strings are shared by 2+ sites, 305 sites in all, and 237 sites pass a ctx
*variable* rather than a literal**), and it is reported as a check, never
merged into the reach number.

---

## 5. Conditional secondary frame

If the primary frame lands with budget left: the **12 `dyninit_tu` + 14
`data_tu` `return None` clauses** get the same treatment via a
`#[track_caller] fn none<T>() -> Option<T>` helper — 26 mechanical edits, one
extra build, and their own registered reach. **If it is not run, it is reported
as not run, with its count.** The `OptWordMode` predicates stay out either way.

---

## 6. Deliverables

1. Re-derived site count and whether **1,227** moved — §1.1 above, restated in
   the rung with the reconciliation.
2. REACHED / QUIET with denominators, per stage.
3. The three-bucket partition, and whether **73 %** holds, is exceeded, or
   collapses on this class.
4. Re-derived counts for the other dropped classes (18 / 12 / 14), parsed.
5. `crates/ fixtures/ scripts/` **byte-identical at both ends**, stated in those
   words, plus the graded-tree identity row.
