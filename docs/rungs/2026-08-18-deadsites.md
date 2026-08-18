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

**Prereg: 43 registered colours, 33 hits, 10 misses** (§8).

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

## 6. The two guards, and each one shown GREEN → RED

*(filled in §6.1–§6.3 from `work/w-deadsites/logs/campaign2.log` and the phase-R
runs.)*

---

## 7. `DATA_SYM_STRLIT_FENCED` — `w-calleeguard` F3's "cheapest follow-on" was already closed, and this is the measurement

*(§7 below.)*

---

## 8. Prereg scorecard

*(§8 below.)*

---

## 9. Gate evidence

*(§9 below.)*

---

## 10. Found and not taken

*(§10 below.)*
