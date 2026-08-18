# DEADSITES — `X` was the sum of THREE backlogs, not two, and the largest of them is neither: 19 of the 26 GREEN sites are sites no input in this project's entire corpus reaches

    Tag:       DEADSITES
    Slug:      deadsites
    Date:      2026-08-18
    Kind:      characterization — the question board **#3246** left open: for
               each of `w-mutcensus`' GREEN refusal sites, is it UNGUARDED
               (reachable, nobody wrote a witness) or DEAD (no input can reach
               it)? — plus the two construct pieces that fell out of it, the
               standing fence-site census (`w-mutcensus` F4) and the strlit
               key's per-SITE witness table
    Outcome:   instrument
    Fixtures:  none — characterization + construct: `tests/fence_site_census.rs`
               and one row per raise site of `data-sym-strlit-fenced` in
               `tests/strlit_fence.rs`
    Census:    +0 — the 878-TU scan is identical on all 394 prefix-anchored
               `gap-metric` keys at both ends. The only `crates/c2-il` byte this
               lane lands is the **deletion** of one provably dead site
               (`leaf_store.rs:2456`); every other `c2-il` edit is an applied and
               reverted probe
    Record:    this file; prereg `docs/rungs/_2026-08-18-w-deadsites-prereg.md`
               (frozen at `14eeb3ec`, committed BEFORE the first probe);
               deviations `work/w-deadsites/deviations.md`; the site table,
               patcher, runners and every raw log under `work/w-deadsites/`
               (tracked); board rows **#3276**–**#3281**, allocated by the
               coordinator

Provenance: board **#3246** (`docs/rungs/2026-08-18-calleeguard.md` §8 F2) —
*"A MUTATION CENSUS CANNOT DISTINGUISH A DEAD SITE FROM AN UNGUARDED ONE … `X`
is the SUM OF TWO BACKLOGS THAT NEED DIFFERENT WORK — sites where somebody must
write a witness, and sites where somebody must delete the code — and the two are
indistinguishable in it. `L9` is one confirmed instance inside the 63; **the
partition has never been taken anywhere.**"* This is the lane that takes it.

---

## 0. The answer

**The partition, over the 26 open GREEN rows** (`w-mutcensus`' 30, minus the four
`w-calleeguard` turned RED, which stay in as controls):

| verdict | n | what it costs | rows |
|---|---:|---|---|
| **UNGUARDED** — the site FIRES in this corpus | **7** | **a witness each**, and cheap: the probe proves an input exists | `CS3` `CS4` `CS9` `CA6` `CA8` `B2` `B7` |
| **DEAD** — quiet, *and* a source-level proof of unreachability | **3** | **one deletion and two non-deletions** — see below | `L9` `CA9` `CA10` |
| **DEAD in the shipped configuration, LIVE under an instrument's hatch** | **1** | **neither**; deleting it breaks `hatch.py`'s ladder | `CA13` |
| **UNKNOWN** — quiet, no proof | **15** | **neither.** Not dead. A statement about this corpus | `CS2` `CA2` `CA16` `CA18` `B3` `B4` `B5` `B6` `B8` `BU3` `D1` `D2` `G2` `L2` `L3` |

**Three things in that table are worth more than the table.**

1. **`X` was not two backlogs. It was three, and the biggest share is neither of
   `#3246`'s two.** **19 of the 26** rows are sites that this project's *entire*
   corpus — the 1,666-test workspace suite, the 19,556-case generated sweep, the
   90,812-cell mode cross, the 18-lane fixture gate, the debug-profile lane and
   the 878-TU workload scan — **never reaches**. A mutation at a site no input
   reaches cannot be killed by any test, so those rows **had** to read GREEN.
   For **73 %** of its open population the census was measuring **corpus reach**,
   not test quality.
2. **The "delete the code" backlog is ONE LINE.** Of the four sites with a
   proof, `CA9` and `CA10` are arms of a `match` over `SlotArg` and removing
   them makes the match non-exhaustive; `CA13` is the backstop
   `work/w-front3/hatch.py`'s `call-arg-outer-formal` hatch needs in order to
   lift the gate above it without panicking. **Exactly one — `leaf_store.rs:2456`
   — is a deletion, and this lane lands it.** `#3246` priced the dead half as
   *"a deletion"*; that price is right for 1 of 4.
3. **`leaf_store.rs:2456` is confirmed.** `w-calleeguard` labelled it *a reading,
   not a measurement* and named the experiment. The experiment was run: the
   `panic!()` probe fires **nowhere** in the corpus, in a run whose suite,
   gate and scan figures are identical to the clean base.

**Method, and it is cheaper than `#3246` costed it.** That board row priced the
partition as *"a second colour per GREEN row, not a second campaign"* — 26 runs.
It is **two** runs: one behaviour-preserving first-hit screen over all 34 sites
at once, then `panic!()` at **every quiet site simultaneously**, because a run
that completes clean confirms all of them at the same time. Reproduced: **P1 and
P2 agree on all 34 rows.**

**Two guards landed, five new tests, and both shown GREEN → RED by construction**
(§6). **`w-calleeguard` F3's "single cheapest follow-on" turns out to be already
closed** and this lane measured it rather than assuming it: both raise sites of
`DATA_SYM_STRLIT_FENCED` are RED, **with disjoint failing sets** (§7).

**Prereg: 46 registered colours, 34 hits, 12 misses** (§8) — and the misses
are not scattered: **8 of the 10 probe misses are one direction**, the
registration over-estimating how much of `c2-il`'s refusal surface this
project's corpus touches.

---

## 1. Populations, every one re-measured at this lane's own base

No figure is inherited. Base is master **`1744ced1`**; run `N0`, a clean tree.

| tag | population | measured here |
|---|---|---|
| **P-T** | `cargo test --workspace --release --no-fail-fast`, `C2RS_REQUIRE_TOOLCHAIN=1` | **1,666 passed / 0 failed / 45 targets**, differential live |
| **P-G** | `scripts/gate.sh --jobs 16 --require-graded` | **PASS**, 18/18 lanes, **81 s** · sweep `checked=19556 graded=19460 mismatches=0` · cross `checked=90812 graded=90424 mismatches=0` · **6,948** fixture-verdicts · debug lane 18/18, **0 panic** |
| **P-W** | 878-TU workload scan | `match` **26** · `mismatch` **0** · `codegen-gap` **0** · `vocab-gap` **844** · `capture-fail` **8** |
| **P-K** | `gap-metric` keys, prefix-anchored `^ *gap-metric \S+ \S+$` | **394** (#3269 — never the naive `grep -c`) |
| **P-F** | `fnbyte-*` | exact **35,899** · differs **1,958** · refused-parse **113,447** |
| **P-S** | the 30 GREEN sites, **re-located by TEXT** | §2 |

### 1.1 `hatch-red` is REFUSED on master at this base, and that is why one deviation cost nothing

`scripts/gate.sh` refuses to run at all on a dirty `crates/`, so every probe run
had to pass `--allow-dirty-crates`, which **refuses the `hatch-red` row** instead
(deviations D1). That looked like a hole in the probed corpus until the base run
was read: **`N0`, on a byte-clean master tree, refuses the same row** —
`REFUSED HATCH-STALE`, board #1389, `hatch.py apply` cannot hatch this tree. So
the probe runs lost nothing master's own gate has. The consequence for `CA13` is
real anyway and is stated in §4.3: **no configuration in which `CA13` is live is
exercised by anything this repo runs.**

---

## 2. The frame — 30 rows, re-located by TEXT, and 15 of them were stale

`w-calleeguard` §5 flagged that **15 of the 26** it re-measured were UNVERIFIED
at its tree because peers had rewritten their files since `3835469c`. That
carried forward to this base: `git diff --stat 3835469c..1744ced1 -- crates/c2-il`
touches `bind.rs` (+205), `body/mod.rs` (+17), `bundle.rs` (+327),
`census.rs` (+146), `diag.rs`, `gl.rs` (+269), `func/mod.rs`, `lib.rs` — and
leaves **`calls.rs` and `leaf_store.rs` untouched**.

**All 30 were re-found by their site text**, and the patcher asserts a
**unique** match before it will apply anything, so a stale locator is a refusal
rather than a wrong edit. The full table is the prereg's §1.1; the shape of the
answer is:

| file | rows | shift `3835469c` → `1744ced1` |
|---|---|---|
| `census.rs` | `CS2`–`CS9` (8) | uniform **+43** |
| `bind.rs` | `B2`–`B8` (7) | uniform **+45** |
| `bundle.rs` | `BU3`, `D1`, `D2` | **+30**, **+49**, **+49** |
| `gl.rs` | `G2` | **+24** |
| `calls.rs` | `CA2`–`CA18` (8) | **0** — file untouched |
| `leaf_store.rs` | `L2`, `L3`, `L9` | **0** — file untouched |

**15 stale rows re-located and re-measured; 11 confirmed byte-identical.** Every
one of the 30 then carried a probe, so no row in this lane's table inherits a
line number or a colour.

**One row moved for a reason worth naming.** `D2` (`bundle.rs`'s `data_tu` `.in`
totality clause) is textually identical to a **second** occurrence at
`bundle.rs:3244` that a peer added after the census froze. The locator uses the
preceding comment line to disambiguate, and the second site is **outside this
lane's frame and unscored** — which is the standing census's whole point (§6).

---

## 3. The method, and the one place it departs from `#3246`

### 3.1 The screen: one behaviour-preserving corpus run for all 34 sites

`#3246` names the `panic!()` probe. A panic **aborts**, so the first site to fire
hides every later one — 30 sites would be 30 corpus runs. So the screen is a
first-hit marker instead: `crate::deadprobe::hit(ix, "ID")` on the branch the
census mutated, one `AtomicU64` bitmask, the first hit per index per process
appending the id to `C2RS_DEADPROBE_LOG`.

It **returns `()` and touches no program state**, so an instrumented corpus run
must reproduce the baseline exactly — and that identity is the instrument's own
self-check, not a hope: `P1` and `P2` both read **1,666 / 0 / 45** with live
differentials of **84.89 s** and **151.00 s**, and `P2`'s gate reproduced every
count of the clean base to the digit.

Eight **controls** ride in the same patch — the four sites `w-calleeguard`
proved reachable (`CS5`–`CS8`) plus one census-RED site per file the probe
touches (`X1` `leaf_store.rs:2254`, `X2` `calls.rs:747`, `X3` `bind.rs:1036`,
`X4` `census.rs:1421`).

### 3.2 The confirmation: `panic!()`, batched

Every site the screen reported quiet had its refusal replaced by
`panic!("w-deadsites <ID>")` — **all 20 at once** — and the whole corpus was
re-run. A run that completes clean confirms all of them simultaneously; a panic
names its own id in its message and would have split out. `Q1`:

| | |
|---|---|
| suite | **1,666 / 0 / 45**, differential **66.12 s** |
| gate | **PASS**, 18/18, sweep 19,556/19,460, cross 90,812/90,424, 6,948 verdicts, debug lane **0 panic** |
| 878-TU scan | clean, exit 0 |
| **`w-deadsites <ID>` markers, grepped from the raw text of every log** | **NONE** |

The markers are grepped out of the log **text**, never inferred from an exit
code: the gate carries a `panics=` column, and a panic that is caught and merely
*counted* leaves no trace in a status. This lane's whole claim is about branches
never taken, so it reads the branch's own word or nothing.

### 3.3 Probe soundness

* **The named control, `C1`, at both ends.** `calls.rs:431` `syms > 1` →
  `syms > 2`, RED **1,664 / 2**, failing exactly
  `the_call_argument_arity_fence_is_a_series_and_admits_exactly_one_symbol` and
  `the_two_symbol_thunk_exemption_turns_on_the_bare_body_marker_alone` — the
  pair `w-guards` pins, by name, reproduced before the first mutant and after
  the last (§6.3).
* **Every suite run carries `C2RS_REQUIRE_TOOLCHAIN=1`** and records the
  `census_gate` target's duration. Minimum over every run in this lane is
  **63.42 s**; none is anywhere near 0.00 s, which is what `w-calleeguard`
  measured an ungraded run at.
* **The table is DERIVED from the logs** by `work/w-deadsites/rederive.py`, never
  accumulated.
* **Every probe patch is verified reverted**, and the patcher refuses to start on
  a dirty tree.

### 3.4 The one control that did not fire, and why the run is not void

**`X3` is quiet in both runs, and it was registered `FIRES` at 0.90.** The
prereg's own stop rule (H3) says a quiet control voids the run. It is scored a
**MISS** and the rule is answered rather than waived:

* **The probe's plumbing in `bind.rs` is proven live by the same patch hunk.**
  `B2` and `B7` fire, and `B3`/`B8`/`X3` are the *adjacent clauses of the same
  two functions*, inserted by the same replacement. A per-file instrument failure
  cannot produce that pattern.
* **Seven of eight controls fire, across all four files.**
* **`X3`'s premise was wrong, and that is a finding rather than a defect.**
  `X3` is `w-mutcensus`' row `B9`, whose mutation is `false &&` on
  `if o.size == 0`. If that branch is never taken, the mutation is semantically a
  **no-op** and no test can fail on it — so `B9` cannot be both RED and
  unreached. `w-mutcensus` §4.4 already records that `B9` is guarded by
  **exactly one** test,
  `reloc_identity::the_cells_population_is_three_functions_one_of_which_disagrees`,
  which *"silently PASSES when its capture yields nothing"* and was the source of
  that campaign's **one** duplicate disagreement. The contradiction is reported,
  not resolved: it is board **#3281**.

---

## 4. The partition, row by row

### 4.1 UNGUARDED — 7 rows, and a witness for each is cheap

| id | site at `1744ced1` | what fires |
|---|---|---|
| `CS3` | `census.rs:1288` | `"static-scan-loop" => STATIC_SCAN_LOOP_OBJECT` |
| `CS4` | `census.rs:1306` | `bind_key.unwrap_or(…)` with `bind_key` **`Some`** |
| `CS9` | `census.rs:1323` | the opt-mode gate |
| `CA6` | `calls.rs:693` | `call-arg-nonformal`, slot path |
| `CA8` | `calls.rs:710` | `call-arg-computed` |
| `B2` | `bind.rs:974` | `resolve_data_def`'s comdat/initialized gate |
| `B7` | `bind.rs:1030` | `resolve_bss_def`'s comdat/initialized gate |

**The price is a witness, and the probe has already done the expensive half of
writing one**: it proves an input exists inside a corpus this repo runs on every
gate. That is the difference between this bucket and the next two, and it is why
the partition is worth more than the count it came from.

### 4.2 DEAD — 3 rows with a proof, of which ONE is a deletion

**`L9` — `leaf_store.rs:2456`.** `bind_run_ops`' first walk (`:2252`–`:2288`)
consumes `ops` in threes and returns `Err` unless slot 0 of each group is
`IlOp::Load`, so on success `ops.len() % 3 == 0` **and every 3k-th op is a
`Load`**. The second walk (`:2453`–`:2462`) re-walks the **same immutable slice**
in threes and asks `if !matches!(b, IlOp::Load(_))`. The condition cannot be
true. Quiet in both screens, `panic!()` fires nowhere. **Deleted — §5.**

**`CA9` / `CA10` — `calls.rs:732` and `:736`, and they are NOT deletions.**
`calls.rs:724` is reached only past `if syms > 0 { return … }` and
`if lits > 0 { return … }`, so `syms == 0 && lits == 0`; `SlotArg::Lit` is pushed
only where `lits += 1` and `SlotArg::SymAddr` only where `syms += 1`. `slots`
therefore holds `Formal` only and neither arm can be selected. **The source says
so itself** — *"Unreachable: `lits == 0` is exactly 'no `SlotArg::Lit` was
pushed', stated positively rather than as an `unreachable!`, because a panic in
the CLI is the failure mode this file's header records."*

**They cannot be deleted.** They are arms of a `match` over `SlotArg`; removing
them makes the match non-exhaustive, and the alternatives are an `unreachable!()`
— the panic that comment exists to avoid — or a merged catch-all that loses the
two distinct keys. They are **dead code that is load-bearing for totality**, and
`#3246`'s "delete the code" is the wrong instruction for them. Registered in the
prereg §3.3 at P = 0.90 before measuring, and **HIT**.

### 4.3 `CA13` — dead in the shipped configuration, live under an instrument's hatch

`calls.rs:772`'s `let Some((cycles, longest)) = permutation_cycles(…) else`
arm cannot be taken: `:747` already refused every `ix >= len` and `:758`–`:762`
every repeat, so at `:771` the sources are `len` distinct values in `0..len` — a
permutation by construction.

**But its own comment names the configuration that makes it live**, and the
configuration is one this repo maintains: *"This arm is what makes that gate
LIFTABLE: with `work/w-front3/hatch.py`'s `call-arg-outer-formal` hatch open,
the walk used to panic (`index out of bounds: the len is 2 but the index is 2`)
and the ladder instrument could not read a single rung below it."*
`work/w-front3/hatch.py:334` is that hatch, verbatim.

So `CA13` is a **fourth** category the census cannot express: a backstop that is
dead in everything that ships and load-bearing for an instrument that does not.
Deleting it would not change one emitted byte and would break the ladder. And
**nothing in the gate exercises it**: `hatch-red` is REFUSED on master (§1.1),
and `hatch_red.py` in any case tests `hatch.py`'s own refusal machinery rather
than compiling a lifted tree.

### 4.4 UNKNOWN — 15 rows, and calling them dead is the error this project keeps making

`CS2` `CA2` `CA16` `CA18` `B3` `B4` `B5` `B6` `B8` `BU3` `D1` `D2` `G2` `L2`
`L3`.

Every one is quiet under two independent screens and a `panic!()`. **None is
dead.** What is established is exactly: *no input in the workspace suite, the
generated sweep, the mode cross, the fixture gate, the debug lane or the 878-TU
workload reaches this branch.* The corpus is named because it is the whole
content of the claim — `#3254` records that **71.2 %** of the emitted denominator
never ships, and `w-corpushealth` records the workload as one head of a tree
moving 284 commits in 14 days.

Two of them have a reading that points at dead and is **deliberately not
promoted**: `L2` (`leaf_store.rs:2257`) and `L3` (`:2285`), which
`w-calleeguard` §4.4 already called *"could not be constructed by reading"*
because `parse_store_stmt` hard-codes `IlOp::Load(base_tok)` in slot 0 and
`admissible_operand` (`:645`) is exactly `:2285`'s accepting predicate. That is
an argument about the **production** path; `bind_run_ops` is `pub(crate)` and
its unit tests hand it arbitrary op vectors, so a proof would have to quantify
over callers rather than over one caller. It is not offered, and they stay
UNKNOWN.

### 4.5 What this does to `w-mutcensus`' headline

`X = 30 of 63` is on master and reads *"how many of `c2-il`'s refusal sites have
no test that can fail on them"*. Over its open population:

```
  26 open GREEN rows
   ├── 7  reachable, unasserted        ← the only rows the headline describes
   ├── 4  unreachable with a proof     ← 1 deletion, 2 totality arms, 1 hatch backstop
   └── 15 unreached, unproven          ← a fact about the corpus, not about the tests
```

**7 of 26.** The sentence *"no test can fail on them"* is true of all 26 and
**informative** about 7. For the other 19 it is a tautology: no test can fail on
a branch no input reaches, so the census could not have measured anything else
there.

---

## 5. `leaf_store.rs:2456` — settled, then deleted

The evidence, in the order it was taken:

1. **A type-level proof from the source** (§4.2), re-derived at this base rather
   than inherited.
2. **Two behaviour-preserving screens** over the whole corpus: quiet in both.
3. **`#3246`'s named `panic!()` probe**, in a run whose suite, gate and scan
   are identical to the clean base: **no marker**.

The file already carries one recorded instance of exactly this class **sixteen
lines above** — *"`codegen::leaf::store`'s `value_bound` refusal was a backstop
with no reachable input (`w-mrslot` §5.1 — board #1218)"* — and this is its
second. The deletion carries a comment naming the proof, the probe and this rung,
so the next reader does not re-derive it.

**And the deletion moved `store-run-bind-group-shape` from 4 raise sites to 3,
which is a row in `tests/fence_site_census.rs`.** Updating that row in the same
commit is the exact workflow that test prescribes when it fails — this lane's
own change is the first thing it caught.

---

## 6. The two guards, and each one shown GREEN → RED **by construction**

**Five new tests, 1,666 → 1,671, 45 → 46 targets.** Every assertion is on the
**published key string** `FnVerdict::key()` emits, never on the constant —
`w-guards`' rule, and `MC2` below is the measurement that the rule is actually
being followed rather than claimed.

### 6.1 `tests/fence_site_census.rs` — `w-mutcensus` F4's standing count, over all 20 keys

F4 asked for *"a gate row that compares that count against a checked-in
expectation and fails when a fence lands without the census being re-scored"*,
and could not land it: that lane's success criterion was a required-zero byte
delta. `w-calleeguard` landed the shape for **one dispatch** and recorded both
blockers expired. This is the general version — **one row per census fence key,
carrying `(raise sites, comparison reads)`**, plus the two textual populations
`w-mutcensus` §2 enumerated separately (`refuse("…")` literal-key sites and
`Block::at_end(` sites).

**It parses `func/body/mod.rs` rather than grepping it, and that alone recovered
two keys every prior enumeration dropped.** Both `w-mutcensus` and
`w-calleeguard` used `pub(crate) const [A-Z_]*: &str`; **that character class
excludes a digit**, so both silently missed `PTR_WALK_LOOP_NOT_O1` and
`PTR_WALK_CHAIN_LOOP_NOT_O1` — the `_O1` suffix is the whole of it. This lane
measures **20 keys over 24 raise sites** at the base where
`2026-08-18-calleeguard.md` §4.2 reports **18 over 22**, and the two reconcile
**exactly**: two keys at one raise site each. Board **#3269**'s rule — *a lane
that finds an unexpected delta owes a measurement before it owes a cause* — is
why that is a reconciliation and not an accusation.

**Four mutants, each a full workspace suite, each derived from its log:**

| id | mutation | expected | **observed** | failing tests |
|---|---|---|---|---|
| `MC3` | `census.rs:1288` `"static-scan-loop"` arm → `STORE_RUN_CALL_NO_CARRIER` — **`w-mutcensus`' own `CS3`, a site that lane measured GREEN.** Moves two rows and leaves the TOTAL at 24 | RED | **RED 1,669 / 2** | **`every_census_fence_key_has_the_sites_this_repo_last_scored`** (+ `rung_index_is_generated_and_current`, this lane's own un-regenerated index — see §6.4) |
| `MC2` | rename the **constant** `STORE_RUN_BIND_GROUP_SHAPE` and all 9 of its uses; the published key string does not move | **GREEN** | **GREEN 1,670 / 1** | — (only the index row) |
| `MC4` | move the **key string** to `"store-run-bind-group-shape-v2"`; the constant does not move | RED | **RED 1,669 / 2** | **`every_census_fence_key_has_the_sites_this_repo_last_scored`** |
| `MC5` | add one `refuse("call-arg-empty-probe")` site — E1's population, invisible to the per-key table by construction | RED | **RED 1,669 / 2** | **`the_two_textual_fence_populations_are_the_size_this_rule_measured`** |

**`MC3` is the sharpest row and it is the argument for this test's existence.**
`CS3` is a site `w-mutcensus` measured **GREEN** — no test among 1,666 could fail
on it. With this file in the tree it is **RED, and this file is the only thing in
the suite that catches it.** The standing census does not merely watch for new
fences; it converts a whole class of key-routing mutation from invisible to
named.

**`MC3` is also the argument for the table over the integer.** It moves
`static-scan-loop-object-out-of-class` 1 → 0 and
`store-run-call-no-emitter-carrier` 1 → 2. **The total is unchanged at 24** — so
a census kept as one number, which is the shape F4 literally asked for, is blind
to it.

**`MC2` is the guard on the guard.** A counting test keyed on the constant's
*name* would go red here for nothing: nothing observable moved. It stays GREEN,
and `MC4` — the same tree with the string moved instead — is RED. The pair is
`w-guards`' rule measured in both directions rather than asserted in a comment.

### 6.2 `tests/strlit_fence.rs` — one row per RAISE SITE, and phase R proves each row catches ALONE

`w-fence163` left `DATA_SYM_STRLIT_FENCED` with **two** raise sites and four
per-cell tests, none of which says *which* site produced the key. The new test
is a two-row table; each row varies **one** fact against a control that rules the
other site out:

| # | site | witness | control that excludes the other site |
|---|---|---|---|
| 1 | `census.rs:1259`, the **pre-parse** `sym_fail` probe | `WS` — **wide** literal, callee defined here, `eh-state1` | `S`, the identical TU with a **narrow** literal, is **IN CLASS** — so the post-parse gate is off for this callee |
| 2 | `census.rs:1511`, the **post-parse** `Some(f)` gate | `N` — **narrow** literal, callee defined here, `eh-none` | `X`, the identical literal with an **external** callee, is **IN CLASS** — so the pre-parse probe admits this literal |

**Phase R — `w-fence163`'s rule, that a guard which catches a mutation
incidentally is not a guard.** The four incumbent guards of this key were
skipped **by name** (never deleted — `w-readphase`'s runner defect), and the two
site mutations re-run against the new table alone:

| run | incumbents | colour | passed/failed | which assertion fired |
|---|---|---|---:|---|
| `N0R` | skipped | GREEN | **1,667 / 0** | — (the skip costs exactly the 4 tests) |
| `MS1R` | skipped | **RED** | **1,665 / 2** | **`strlit_fence.rs:457` — ROW 1**, "the WIDE literal must take … through the PRE-PARSE `sym_fail` probe (`census.rs:1259`)" |
| `MS2R` | skipped | **RED** | **1,665 / 2** | **`strlit_fence.rs:480` — ROW 2**, "the NARROW literal whose callee is DEFINED HERE … must take … through the POST-PARSE gate (`census.rs:1511`)" |

**Two mutations, two different rows of one table, each naming its own site.**
That is site-level discrimination *inside a single test*, and it is
`w-calleeguard`'s **P13** — *"the form demonstrated on a family where ≥2 sites
share one key"* — which that lane registered at 0.20 and recorded as a MISS.
It is closed here.

(The second failing test in both `MS*R` rows is
`every_census_fence_key_has_the_sites_this_repo_last_scored`: `MS1`/`MS2` each
retarget a raise site from `data-sym-strlit-fenced` to `data-sym-not-extern`, so
§6.1's table moves too. Two independent guards catching one mutation is the
opposite of the incidental-catch problem, and both are by construction.)

### 6.3 The named control, at both ends

`docs/rungs/README.md` probe rule 1. `C1` — `calls.rs:431`, `syms > 1` →
`syms > 2`:

| run | when | colour | passed/failed | failing tests |
|---|---|---|---:|---|
| `C1a` | before the first guard landed | **RED** | **1,664 / 2** | `the_call_argument_arity_fence_is_a_series_and_admits_exactly_one_symbol` · `the_two_symbol_thunk_exemption_turns_on_the_bare_body_marker_alone` |
| `C1b` | after the last | **RED** | **1,669 / 2** | **the identical pair, by name** |

Both tests in that set are capture-driven, so a worktree whose captures were
skipping could not produce it.

### 6.4 One honest note about the failing sets above

`rung_index_is_generated_and_current` appears in every `campaign2` row because
this lane added its rung doc before regenerating `docs/rungs/INDEX.md`. It is
this lane's own bookkeeping, not a property of any mutant — the index was
regenerated by `scripts/gen_rung_index.sh` before the tip runs — and it is named
here rather than filtered out of the table, because a results table that
silently drops a row it finds inconvenient is not derived from its logs.

---

## 7. `DATA_SYM_STRLIT_FENCED` — F3's "single cheapest follow-on" was already closed, and this is the measurement rather than the assumption

`w-calleeguard` §8 **F3** named these two sites *"the single cheapest follow-on
here"*, on the reasoning that `w-mutcensus`' frame froze before `w-fence163`
landed them so **neither has ever been mutated**. Both were mutated here, on the
base tree, before this lane landed anything:

| id | site | colour | passed/failed | failing tests |
|---|---|---|---:|---|
| `MS1` | `census.rs:1259` (pre-parse probe) | **RED** | 1,664 / 2 | `gap::tests::…::the_string_literal_admission_is_narrow_only_and_leaves_cell_b_alone` · `only_the_narrow_string_literal_is_admitted_and_the_wide_twin_still_refuses` |
| `MS2` | `census.rs:1511` (post-parse gate) | **RED** | 1,664 / 2 | `the_older_inline_fence_shadows_this_one_on_a_walkable_tu` · `the_strlit_fence_turns_on_the_local_callees_eh_state_and_nothing_else` |

**Both RED, and the failing sets are DISJOINT.** So the key was already guarded
**at site level**, not merely at key level, by `w-fence163`'s own cells — and
F3's premise, that being unmutated made them likely unguarded, was wrong. It is
scored as a HIT for this lane's prereg (H5, registered at 0.70 each before the
runs) and as a **withdrawal of F3's pricing**, not of F3's observation: the sites
genuinely had never been mutated, and mutating them is how anybody knows.

**This is the second counterexample to `w-mutcensus` F2's `1/k` bound**, after
`L6`/`L7`. Of the three `k ≥ 2` keys in the crate, **two are now measured with
every site guarded** and the third (`store-run-bind-group-shape`) is 1 reachable,
2 unknown and — as of this lane — one fewer site than it had.

**What was actually missing, and what this lane landed instead**, is in §6.2:
nothing said which site produced the key, and nothing stopped a **third** site
landing unwatched. The witness table answers the first; §6.1's row
`("data-sym-strlit-fenced", 2, 0)` answers the second.

**One residual, stated because it is real and not fixed here.** `MS1` is caught
by one **synthetic** test and one capture test; `MS2` by **two capture tests
only**, and now by the new table, which is also capture-driven. So site 2 has
**no toolchain-free guard at all**, and in an unprovisioned worktree its mutation
reads GREEN — #3219's family exactly. `C2RS_REQUIRE_TOOLCHAIN` makes that loud
when a caller demands grading, and this lane ran every suite with it armed; a
synthetic cell for site 2 is §10 F3.

---

## 8. Prereg scorecard — 46 registered colours, 34 hits, 12 misses

34 probe colours (24 / 10) + 11 headline registrations (9 / 2) + the one
structural prediction of §3.3 (HIT).

### 8.1 The 34 probe colours: 24 hits, 10 misses, and 8 of the 10 in ONE direction

| direction | count | rows |
|---|---:|---|
| registered **FIRES**, observed **quiet** | **8** | `CA16` `CA18` `B4` `B5` `BU3` `D1` `G2` `X3` |
| registered **quiet**, observed **FIRES** | **2** | `CS3` `CS4` |

**The registration was systematically optimistic about how much of `c2-il`'s
refusal surface this project's corpus touches**, and that single bias is the
whole of the gap between the registered split (UNGUARDED 12) and the observed one
(UNGUARDED 7). Stated as a number rather than a vibe: **24/34 = 71 %** of probe
colours correct, error directional rather than noisy. A future lane probing the
1,227-site grammar class `w-mutcensus` §2.1 dropped should register **less**
reach than intuition suggests, not more.

### 8.2 The 11 headline registrations

| id | registration | outcome |
|---|---|---|
| **H1** | DEAD **4** [3, 7] · UNGUARDED **12** [9, 16] · UNKNOWN **10** | **MISS**, scored on its weakest half: **UNGUARDED 7**, below the registered interval. **DEAD 4 is exact**, on the point estimate and inside [3, 7]. UNKNOWN **15** against 10 |
| **H2** | `leaf_store.rs:2456` confirmed dead; the `panic!()` does not fire | **HIT** |
| **H3** | all 8 controls fire; a quiet control **voids the run** | **MISS** — `X3` is quiet. The stop rule is answered in §3.4, not waived, and the answer is board **#3281** |
| **H4** | the instrumented run reproduces the baseline counts exactly | **HIT** — 1,666 / 0 / 45 instrumented and clean |
| **H5** | both `DATA_SYM_STRLIT_FENCED` sites already guarded; P(both RED) = 0.55 | **HIT** — both RED, disjoint sets (§7) |
| **H6** | the standing census lands as one test file, keyed on key strings and per-key counts, shown GREEN → RED | **HIT** (§6.1) |
| **H7** | suite ends at 1,666 + k, 2 ≤ k ≤ 8 | **HIT** — k = **5** |
| **H8** | 878-TU scan: 0 differing lines over all 394 keys, base vs tip | **HIT** — 0 of 394, *including* the `fnbyte-*` family |
| **H9** | gate PASS at both ends; per-lane count identity diff 0 rows, range length asserted | **HIT** — PASS both ends, **0 of 23** differing, 23 = 23 |
| **H10** | `git diff master..HEAD -- crates/c2-il` is nothing but proven-dead deletions | **HIT** — one deletion, `leaf_store.rs:2456`, justified by §5 |
| **H11** | the lane publishes a **non-empty UNKNOWN bucket** rather than resolving the population into two halves | **HIT** — 15 of 26 |

**H11 was registered because it is the failure this lane was most likely to
commit**, and it is the one the brief named: *deleting a site because your probe
did not reach it is exactly the error this project keeps making.* 15 rows are
published as UNKNOWN and none of them is touched.

### 8.3 The prereg's registered structural prediction, and it is a HIT

Prereg §3.3, frozen before any measurement: *"at least one of `CA9`/`CA10` will
turn out **not deletable** … P = 0.90 that the 'delete the code' price is wrong
for at least one row of the dead half."* Observed: **three of the four** sites
with a proof are not deletions (§4.2, §4.3). `#3246`'s pricing of the dead half
is wrong for 3 of 4.

---

## 9. Gate evidence

| check | base (`N0`, master `1744ced1`) | **tip** |
|---|---|---|
| `cargo test --workspace --release --no-fail-fast`, `C2RS_REQUIRE_TOOLCHAIN=1` | **1,666 / 0 / 45** | **1,671 / 0 / 46** — above 1,666 as required, k = 5 |
| `census_gate` duration (the differential actually grading) | 63.64 s | 63.55 s — minimum over **every** run in this lane is **62.90 s**, none near 0.00 s |
| `scripts/gate.sh --jobs 16 --require-graded` | **PASS**, 81 s | **PASS (HATCH-RED REFUSED)**, 77 s · 18/18 lanes, 0 FAIL, 0 SKIP, 0 NO-RESULT · **6,948** fixture-verdicts · sweep `checked=19556 graded=19460 mismatches=0` · cross `checked=90812 graded=90424 mismatches=0` · debug lane 18/18, 2,423 match, **0 panic** |
| per-lane gate-count identity diff | — | **0 of 23 rows differ**, and the **range LENGTH is asserted at both ends** (23 = 23) — a diff of two empty ranges also returns 0. This is the discriminating check for a lane that lands tests (board **#3215**) |
| 878-TU workload scan | `match` **26** · `mismatch` **0** · `codegen-gap` **0** · `vocab-gap` **844** · `capture-fail` **8** | **identical**, asserted line for line |
| `gap-metric` keys, `^ *gap-metric \S+ \S+$` | **394** | **394**, and `diff` over the sorted key lines is **EMPTY — 0 differing lines over all 394** |
| `fnbyte-exact` / `differs` / `refused-parse` | 35,899 / 1,958 / 113,447 | **35,899 / 1,958 / 113,447** — unmoved, per **#3249** read back-to-back with the base rather than against a briefed figure |
| `git diff master..HEAD -- crates/c2-il` | — | **one file, one hunk** — the `leaf_store.rs:2456` deletion of §5 and its comment. `git diff 1744ced1..HEAD -- crates fixtures scripts` is exactly three files: the two new test files and that deletion |
| `scripts/board_audit.sh` | — | **all-zero** — 0 cited-but-not-on-board · 0 unresolved anchors · 0 raw line anchors · 0 rows-behind-prose · 0 duplicate row numbers |
| `crates/c2-harness/tests/rung_registry.rs` | 2/2 | **2 / 2** |
| release-binary sha256 across worktrees | **NOT compared** — board **#3224**: `CARGO_MANIFEST_DIR` is compiled in, so the comparison is void | |
| graded-tree identity at both ends | **DOES NOT APPLY** — board **#3215**: this lane lands test code *and* deletes a `c2-il` line, so the content hash moves **by construction** and claiming it did not would be false | |

---

## 10. Found and not taken, ranked

### F1 — The census's GREEN population is 73 % a fact about the CORPUS, and the same question is open on the 1,227-site grammar class

19 of 26 rows are unreached. `w-mutcensus` §2.1 dropped a **1,227-site** grammar
class for budget, and `#3246` says the partition *"matters most"* there. This
lane's screen makes that affordable: it is **one** instrumented corpus run for
however many sites fit in the bitmask, not one run per site, and the
behaviour-preserving marker means the run's own totals are the validity check.
Sizing: 1,227 sites is 20 bitmask words, or ~20 runs of the shape run here —
against `w-mutcensus`' estimate of **five days serial** for the mutation version.

### F2 — A site's reach should be attributed to a corpus STAGE, and this lane did not do it

The probe records *that* a site fired, not *where*. That distinction is the price
of a witness: a site reached by the unit suite is a cheap witness, one reached
only by the 878-TU workload is an expensive one, and this lane reports all seven
UNGUARDED rows at one price. The fix is one field — the marker already carries an
id, and the stage is `$C2RS_DEADPROBE_LOG` per stage rather than per run. **NOT
TAKEN for budget**; it is a ten-minute change to `corpus.sh`.

### F3 — Site 2 of the strlit fence has no toolchain-free guard, and the file explains why building one is hard

§7's residual. `tests/strlit_fence.rs`' header argues the cells must be captures
because census clause (c) shadows clause (c2) on any TU whose `.gl` defined-name
walk succeeds. A **synthetic** `.ex`/`.gl` bundle carrying a defined-here,
non-`eh-state1`, unmodelled callee plus a narrow strlit data symbol would settle
it from `gap/tests.rs` with no toolchain at all, and would be the first cell in
that module to exercise the post-parse gate. It is a real lane, not a footnote.

### F4 — `w-mutcensus`' `X = 30 of 63` is on master and its headline sentence is now known to be informative about 7 of 26

Not corrected here, deliberately: a dated rung stays as written
(`#3117`). But `docs/BOARD.md` **#3276** carries the partition, and any future
quotation of *"X of `c2-il`'s refusal sites have no test that can fail on them"*
should carry the clause *"…of which the majority are sites no input reaches"*.
The same applies to `w-calleeguard` §5's re-measured **26**, which this lane
re-derived from scratch rather than inheriting.

### F5 — `B9`'s RED and `X3`'s silence cannot both be right, and this lane did not settle it

§3.4, board **#3281**. `w-mutcensus`' `B9` mutation is `false &&` on a branch
this lane's probe and `panic!()` both say is never taken; that mutation would
then be a semantic no-op and unkillable. The candidate explanation is already in
`w-mutcensus` §4.4 — `B9`'s **sole** guard is
`reloc_identity::the_cells_population_is_three_functions_one_of_which_disagrees`,
which *"silently PASSES when its capture yields nothing"* and produced that
campaign's only duplicate disagreement — but a candidate explanation is not a
measurement, and this lane owes it none. Re-running `B9` at this base is one
suite run.

### F6 — Two enumerations of one file both dropped the same two keys, and nothing but a parser would have caught it

§6.1. `[A-Z_]` excludes a digit; `PTR_WALK_LOOP_NOT_O1` has one. Two independent
lanes wrote the same grep and got the same wrong 18. The general item is that
**every enumeration in this repo that greps Rust identifiers with an explicit
character class is a candidate for the same defect**, and a five-minute audit
would find them. NOT TAKEN — it is a `scripts/` and `work/` sweep, and
`scripts/` is peer `w-coldcross`'s seam this wave.
