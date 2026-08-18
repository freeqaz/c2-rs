# CALLEEGUARD — all four raise sites of the `callee-unresolved` family go RED, and the mechanism they were supposed to illustrate turns out to explain at most 5 of the census's 30

    Tag:       CALLEEGUARD
    Slug:      calleeguard
    Date:      2026-08-18
    Kind:      construct — builds one witness per RAISE SITE for the
               `callee-unresolved` census key family, the standing site count
               that keeps the family's site set from moving unwatched, and
               `C2RS_REQUIRE_TOOLCHAIN` (`w-mutcensus` F1)
    Outcome:   instrument
    Fixtures:  none — construct rung: guards for `callee-unresolved-framed-call`,
               `-call-sequence`, `-dtor-delegation` and `-tail-call`, all reached
               through the PUBLIC `IlBundle::census_functions()`
    Census:    +0 — required-zero byte delta; **zero bytes land in
               `crates/c2-il`**, every `c2-il` edit in this lane is a reverted
               mutant, and the 878-TU scan is identical on all 394 `gap-metric`
               keys at both ends
    Record:    this file; prereg
               `docs/rungs/_2026-08-18-w-calleeguard-prereg.md` (frozen at
               `9673841c`, committed BEFORE the first probe); additions and
               deviations `work/w-calleeguard/deviations.md`; raw logs, runner
               and the log-derived table generator under `work/w-calleeguard/`
               (tracked); board rows `#3244`–`#3248`, allocated by the
               coordinator

Provenance: `docs/rungs/2026-08-17-mutcensus.md` §4.3 — *"the `callee-unresolved`
family is the one to read first. All four of its routing arms are unguarded,
including `CS8`, the **default** arm … That key — `callee-unresolved-tail-call` —
is the one board **#3209** measured rising to **1,296** bodies … **The single
most populous refusal key on the 878-TU workload can be swapped for a sibling
and the entire suite stays green.**"* This is the lane that closes it.

---

## 0. The answer

**Four of four raise sites are guarded, and each was measured GREEN before and
RED after, in this worktree, against a live real-`c2` differential.**

```
                                            phase R (guards skipped)   phase G (guards in)
  R5/G5  census.rs:1308  "framed-call" =>          GREEN 1660/0   ──▶   RED 1662/3
  R6/G6  census.rs:1310  "call-sequence" =>        GREEN 1660/0   ──▶   RED 1662/3
  R7/G7  census.rs:1313  "empty-dtor" =>           GREEN 1660/0   ──▶   RED 1662/3
  R8/G8  census.rs:1315  _ =>   (the DEFAULT arm)  GREEN 1660/0   ──▶   RED 1661/4
```

**Five tests, 1,660 → 1,665, 43 → 45 targets, and not one byte of
`crates/c2-il`.** All four sites are in a crate this lane does not own
(`w-dataseam`'s seam this wave), so every guard reaches them through the
**public** `IlBundle::census_functions()` and asserts on the **key string**
`FnVerdict::key()` publishes — never on the constant, for `w-guards`' exact
reason: *a guard on the constant would pass a mutation that renamed the constant
and its uses while the published key moved.*

**But the more valuable half of this lane is what it found about the mechanism
it was sent to retire**, and it points the other way from the brief:

> **`w-mutcensus` F2's mechanism — *"a key with k raise sites contributes k − 1
> unguarded sites by construction"* — explains at most 5 of the 30 GREEN sites,
> does not apply to this family at all, is already contradicted inside the
> census's own data, and its leading worked example is dead code.**

§4 is that argument with its numbers. **33 registered colours, 30 hits, 3
misses** (§6).

---

## 1. Populations, each re-measured at this lane's own base

`w-vocabgap`'s house rule: no figure is inherited. Base is master **`44794fa4`**.

| tag | population | measured here |
|---|---|---|
| **P-T** | `cargo test --workspace --release --no-fail-fast` at the base | **1,660 passed / 0 failed / 43 targets** — run `N0R`, the clean tree with this lane's three arm witnesses skipped **by name**. Reproduces the briefed base figure exactly |
| **P-T′** | the same at the tip | **1,665 / 0 / 45** (run `N0T`) |
| **P-W** | 878-TU workload scan | `match` **26** · `mismatch` **0** · `codegen-gap` **0** · `vocab-gap` **844** · `capture-fail` **8** — **identical at both ends** |
| **P-K** | `gap-metric` keys on P-W, prefix-anchored `^ *gap-metric ` | **394**, and the identity diff over all 394 is **ZERO differing lines** at both ends |
| **P-F** | `gate.sh --jobs 4 --require-graded` | **PASS at both ends**, 18/18 lanes, **6,948** fixture-verdicts, sweep **19,556 / 19,460**, cross **90,424 / 90,812**, debug-lane 18/18 at **0 panic** |
| **P-S** | the four sites, **re-located at this base** | `crates/c2-il/src/func/census.rs` **:1308**, **:1310**, **:1313**, **:1315** — the census's `3835469c` figures were `:1265`, `:1267`, `:1270`, `:1272`, a uniform **+43** |

### 1.1 Board #3249 is visible in this lane's own reading, and is reported rather than adjusted

The coordinator's correction arrived mid-lane and it is scored here as it was
given. **The keys that must be identical are, and the `fnbyte-*` family moved
against the *briefed* figures at the same commit:**

| | briefed at `44794fa4` | this lane's **base** end (detached `44794fa4`) | this lane's **tip** |
|---|---:|---:|---:|
| `fnbyte-exact` | 35,897 | **35,899** | **35,899** |
| `fnbyte-refused-parse` | 113,449 | **113,447** | **113,447** |
| `fnbyte-differs` | — | 1,958 | 1,958 |
| sum of the first two | 149,346 | **149,346** | **149,346** |

**Base and tip agree digit for digit**, so nothing this lane landed moved
anything; what moved is the same commit read at two times, which is exactly
board **#3249** — the scan's inputs (`work/capture-cache`, resolved through
`main_repo_root()` and shared by every worktree; `work/dc3-workload`, untracked)
are not in the tree, so `fnbyte-*` is a reading of *(commit × cache state ×
untracked workload)*. **Nothing outside the `fnbyte-*` family moved**, which is
the part that would have been a finding about this lane. No number was adjusted
to make the identity claim come out clean.

---

## 2. Where the guards had to go, and what each cell is

`crates/c2-il` is peer `w-dataseam`'s seam. `CALLEE_UNRESOLVED_*`,
`shape_to_function` and the `match label` that routes them are all `pub(crate)`
and unreachable across the crate boundary. The route that exists is
`IlBundle::census_functions()`, driven by hand-built synthetic `.ex` + `.gl`
bundles — `w-guards`' instrument, extended with four new transcripts.

**Every raise site fires on one condition: the body parsed, and
`shape_to_function` returned `None` with no data-symbol `sym_fail` pending.** The
honest way to produce that is a callee token with no `.gl` name — which is what
the keys are *named after*. So each cell is a **pair on otherwise identical
bytes**: the same `.ex` transcript with a `.gl` that names the callee, and one
whose callee record differs in **exactly one byte**.

| cell | transcript (`c2-il`'s own captures, transcribed) | `.gl` names the callee | one byte later |
|---|---|---|---|
| **F** | `int f(int a){ return g(a)+1; }` | `framed-call` | **`callee-unresolved-framed-call:eof`** |
| **Q** | `void f(int a){ g1(a); g2(); }` | `call-sequence` | **`callee-unresolved-call-sequence:eof`** |
| **E** | `Der::~Der() {}` | `empty-dtor-delegation` | **`callee-unresolved-dtor-delegation:eof`** |
| **V** | `void f(){ g(); }` | `void-tail-call` | **`callee-unresolved-tail-call:eof`** |
| **M** | `w-guards`' `DYNINIT` + `gl_named(0x02)` | — (refuses past the gate) | **`callee-unresolved-tail-call:eof`** |

The four transcripts are **copies of `c2-il`'s own `#[cfg(test)]` captures**
(`func::test_fixtures::{MVP_FRAMED, SEQ_TWO_VOID, DTOR_DELEGATE, MVP_CALL}`),
transcribed for the reason `w-guards` transcribed `DYNINIT`: that module is
`#[cfg(test)]` inside a crate this lane does not own. They are copies of
captures, **not second sources of truth** — if `c2-il`'s readers change under
them these guards break, and that is the guard working.

### 2.1 Cell separation, stated as the minimal difference and ASSERTED

* **The two `.gl` of every cell differ in exactly ONE byte** — the callee
  token's high byte, `^= 0x05`. Same length, same record count, same name. The
  test computes the differing-byte count and asserts it is `1`; a pair that
  drifted apart fails rather than passing on a coincidence.
* **The positive control is the same bytes with the callee named**, and it must
  report the *in-class shape label*. Without it a refusal is consistent with the
  arm refusing its whole input, and the table would be a constant rather than a
  discrimination.
* **"Exactly one census row" is a NAMED failure**, through `key_of`'s existing
  panic path — never an `unwrap`. A bundle that stopped segmenting would make
  every assertion unreachable and the module would go green for the reason
  `docs/STATUS.md` trap 5 names.
* **The four family keys are asserted pairwise distinct as a COUNT** (4 of 4). A
  collapse would make every equality above satisfiable by one key.

### 2.2 The default arm needs TWO witnesses, and this is a site-level property no key-level assertion can express

`census.rs:1315` is `_ =>`. One cell reaching `callee-unresolved-tail-call` is
**equally consistent with the arm having been rewritten to match that cell's own
label**. Cells **V** and **M** carry *different* in-class labels —
`void-tail-call` and `multiarg-tail-call`, and the test proves they differ by
asking the census, not by asserting it in prose — and both reach the one key.
That is the statement that the arm is the fallthrough.

It is also the clean demonstration that **site-witnessing is strictly stronger
than key-witnessing**: no assertion about the *key* can distinguish "the default
arm" from "an arm keyed on `multiarg-tail-call`", because both produce the same
string on the same input.

### 2.3 The standing site count (A1)

A witness table covers the sites that existed when it was written, and **nothing
makes a new arm add a row** — `w-mutcensus` **F4**, which that lane could not
take. `crates/c2-harness/tests/callee_unresolved_sites.rs` is F4's standing
count scoped to one dispatch: it reads `c2-il`'s source, brace-matches the
`match label { … }` block (never a line number — the census's own line numbers
went stale on two peer merges inside one lane's wall clock), and asserts

* each of the four constants is raised **exactly once** — the condition that
  makes a per-key witness *equal* a per-site witness here, and the thing that
  fails the moment F2's mechanism starts to apply to this family;
* the family's site total is **4**;
* each of the dispatch's **7** arm patterns appears exactly once, and the arm
  count is 7 — because `callee-unresolved-tail-call` is *whatever no earlier arm
  claimed*, so adding or reordering any arm can move bodies out of #3209's 1,296
  without touching one line of the family;
* `body/mod.rs` declares each constant with its exact **key string**, which is
  the binding that makes the string-asserting witnesses and the
  constant-counting site test one guard rather than two that can drift apart.

---

## 3. Mutants — every colour registered in the prereg before any run, every colour DERIVED from the logs

`work/w-calleeguard/{run_mutant.sh,campaign.sh,rederive.py}`, logs under
`work/w-calleeguard/logs/` (all tracked). The runner **prints the site count and
aborts unless it is 1**, refuses to start on a dirty tree, records the
`census_gate` target's **duration** per run, and verifies the revert. The table
below is produced by `rederive.py` from those logs and is **never accumulated**
(`docs/rungs/README.md` probe rule 2).

| id | phase | colour | passed/failed | targets | `census_gate` | wall | failing tests |
|---|---|---|---:|---:|---:|---:|---|
| `N0` | baseline, guards in, pre-A1/A2 | **GREEN** | 1663/0 | 43 | 68.46 s | 216 s | — |
| `N0R` | baseline, guards skipped by name | **GREEN** | **1660/0** | 43 | 65.06 s | 195 s | — |
| `N0T` | baseline at the tip | **GREEN** | **1665/0** | 45 | 79.25 s | 222 s | — |
| `R5` | R | **GREEN** | 1660/0 | 43 | 92.81 s | 237 s | — |
| `R6` | R | **GREEN** | 1660/0 | 43 | 93.98 s | 255 s | — |
| `R7` | R | **GREEN** | 1660/0 | 43 | 89.80 s | 233 s | — |
| `R8` | R | **GREEN** | 1660/0 | 43 | 95.78 s | 240 s | — |
| `G5` | G | **RED** | 1662/3 | 45 | 94.13 s | 251 s | `…arms::every_raise_site…`, `…arms::each_…one_gl_byte…`, `the_callee_unresolved_family_still_has_exactly_four_raise_sites` |
| `G6` | G | **RED** | 1662/3 | 45 | 72.30 s | 209 s | the same three |
| `G7` | G | **RED** | 1662/3 | 45 | 68.92 s | 202 s | the same three |
| `G8` | G | **RED** | **1661/4** | 45 | 65.84 s | 196 s | the same three **plus `…arms::the_default_arm_is_the_catch_all_reached_by_more_than_one_label`** |
| `C1a` | control, before phase G | **RED** | 1663/2 | 45 | 78.79 s | 225 s | `the_call_argument_arity_fence_is_a_series_and_admits_exactly_one_symbol`, `the_two_symbol_thunk_exemption_turns_on_the_bare_body_marker_alone` |
| `C1b` | control, after phase G | **RED** | 1663/2 | 45 | 83.35 s | 218 s | **the identical pair, by name** |
| `N1` | input perturbation | **RED** | 1662/3 | 45 | 77.67 s | 220 s | the three arm witnesses |
| `D6a` | no toolchain, demand unset | **INVALID** | 1665/0 | 45 | **0.00 s** | **7 s** | — |
| `D6b` | no toolchain, demand set | **RED** | 1664/1 | 45 | 0.00 s | 6 s | `require_toolchain::a_run_that_claims_to_grade_must_have_a_toolchain_to_grade_with` |

**`G8` is the sharpest row.** It takes down four tests where `G5`–`G7` take
three, and the fourth is exactly the default-arm catch-all witness — because
swapping the *default* arm's key moves **both** default-arm cells while swapping
a named arm's key moves neither. The guard discriminates the arms, not just the
family.

### 3.1 Phase R is not ceremony: it is `w-fence163`'s rule applied

`w-fence163` found that **a guard that catches a mutation incidentally is not a
guard** — one of its corroborating tests stopped catching a mutant when a peer
merged an unrelated file, with the test byte-identical. Two peers (`w-npos`,
`w-fence163`) landed in `c2-il` between the census's `3835469c` and this lane's
`44794fa4`, and `census.rs` is one of the files they rewrote (+146 lines).

**All four sites read GREEN at `44794fa4`**, so `w-mutcensus`'s `CS5`–`CS8`
reproduce exactly at this base and **none of the four had become incidentally
guarded**. Phase R ran with the three arm witnesses **skipped by name**, so no
file was destroyed to measure it (`w-readphase`'s runner defect), and the skip's
cost is measured rather than assumed: `N0` 1,663 → `N0R` 1,660.

### 3.2 Probe soundness — and this lane both obeyed the rule and DEMONSTRATED the failure it prevents

`docs/rungs/README.md`'s probe rule 1 (boards #3219 / #3231) requires a control
pinned **by name**, re-run in every environment, plus executed-population checks
rather than exit codes.

* **`C1` reproduces `w-guards`' pair to the test name, before the first mutant of
  phase G and after the last** — both at 1,663/2. Both tests in that set are
  capture-driven, so a worktree whose captures were skipping could not produce
  it.
* **Every run in the campaign has a live differential**: minimum over the 14
  provisioned runs is **65.06 s**; none is anywhere near 0.00 s.
* **The worktree was provisioned by `scripts/setup_worktree.sh`**, whose own hard
  gate (`fixtures/cpp/w5_chain.cpp -> 4/4 functions in class`) passed at setup.

And then the failure mode itself was reproduced **at this base**, deliberately:

> `D6a` and `N0T` are the **same tree, the same command, the same target
> count**. `N0T`: **1,665 / 0 / 45**, differential **79.25 s**, wall **222 s**.
> `D6a` with `C2RS_COMPILERS` pointed at nothing: **1,665 / 0 / 45**,
> differential **0.00 s**, wall **7 s**. **The totals are byte-identical,
> including this lane's own five new tests.** That is why a probe defined by a
> count cannot see it, and why the prereg classified `D6a` **INVALID rather than
> GREEN** — its log is **kept**, as `D6a.INVALID.log`.

`D6b` is the same environment with `C2RS_REQUIRE_TOOLCHAIN=1` and it is **RED**,
failing exactly one test by name. **One environment variable turns the repo's
most-recorded defect from invisible into a failure.**

---

## 4. The mechanism — and why F2 is not it

This is the deliverable the brief cared about most, and the answer is not the one
the brief anticipated.

### 4.1 It does not apply to this family at all, and that was known before the first probe

`grep -rn 'CALLEE_UNRESOLVED_' crates/` at `44794fa4` returns, outside the four
`const` declarations and one `use` line, **exactly one raise site per key**. So
**k = 1 four times over**, and F2's *"a key with k raise sites contributes k − 1
unguarded sites by construction"* has nothing to contribute here. **These four
sites were not unguarded by construction. Nobody had written a witness.** That
was registered as a frame fact in the prereg §1.1, read out of the source before
any probe, precisely so it could not be discovered afterwards and told as a
result.

### 4.2 Sized over the whole crate: F2 can account for at most 5 of the 30 GREEN

Every `pub(crate) const … : &str` in `crates/c2-il/src/func/body/mod.rs` was
enumerated and its raise sites counted (excluding declarations, `use` lines, doc
comments and everything at or past each file's `#[cfg(test)]` boundary), then
cross-checked by grepping the raw **key strings** to confirm no site raises a key
by literal instead of by constant — the only literal hit is a doc comment at
`census.rs:651`.

23 constants, of which 5 are dispatch/production **axis tags** read by
`dispatch_site()`/`prod_site()` and never reaching `Block::at_end`. Over the **18
census fence keys**:

| k (raise sites) | count | which |
|---:|---:|---|
| **0** | 1 | `STORE_RUN_BIND_CALL_TAIL_RETIRED` — `w-mutcensus` F5's key with no fence |
| **1** | **14** | `OPT_MODE`, `CALLEE_UNRESOLVED_TAIL`, `CALLEE_DEFINED_IN_TU`, `STORE_RUN_CALL_NO_CARRIER`, `STATIC_SCAN_LOOP_OBJECT`, `STORE_RUN_BIND_NO_CARRIER`, `STORE_RUN_BIND_MIXED_KIND`, `STORE_RUN_BIND_ADDR_PRODUCER`, `STORE_RUN_BIND_SYMBOL_CROSSINGS`, `CALLEE_UNRESOLVED_DTOR`, `CALLEE_UNRESOLVED_FRAMED`, `CALLEE_UNRESOLVED_SEQ`, `DATA_SYM_UNRESOLVED`, `DATA_SYM_LINKAGE` |
| **2** | 2 | `STORE_RUN_BIND_MULTI_PRODUCER` (`leaf_store.rs:2391`, `:2400`) · `DATA_SYM_STRLIT_FENCED` (`census.rs:1259`, `:1511`) |
| **≥4** | 1 | `STORE_RUN_BIND_GROUP_SHAPE`, k = 4 (`leaf_store.rs:2254`, `:2257`, `:2285`, `:2456`) |

**15 of 18 fence keys have k ≤ 1**, so F2's mechanism is *structurally
inapplicable* to 83 % of them. Its entire predicted unguarded contribution is
`Σ(k − 1)` = **5 sites**. The census observed **30 GREEN of 63**. So the
mechanism can account for **at most 5 of 30 — 17 %** — and in fact for less, per
the next two subsections. **`callee-unresolved` is not an exception to F2; it is
the modal case.**

### 4.3 The census's own data already contradicts F2's `1/k` bound

F2 states *"the guarded fraction of a k-site key family is bounded above by
`1/k`"*. `STORE_RUN_BIND_MULTI_PRODUCER` is **k = 2 with BOTH sites RED** in
`w-mutcensus` §3's own table — rows `L6` (`leaf_store.rs:2390/:2391`,
`lits.len() > 1`) and `L7` (`:2399/:2400`, the pool bound at ≥ 9 formals), both
guarded by `every_bind_gate_fires_on_a_named_input`, whose case 3 and case 3b
(`leaf_store.rs:3166`, `:3174`) are **one witness per site**.

So the suite F2 characterises as *"per-KEY"* is in fact a **witness table that
already does site-level coverage where someone bothered**. The limit is not the
form of the suite. It is how many rows were written.

### 4.4 F2's leading worked example is DEAD CODE, and its GREEN is not evidence of an unguarded fence

`STORE_RUN_BIND_GROUP_SHAPE`'s fourth site — `leaf_store.rs:2456`, census row
`L9`, the one F2 leads with — **cannot be reached by any input at all.**
Verified in this lane by reading `bind_run_ops`
(`crates/c2-il/src/func/body/shapes/leaf_store.rs:2217`), whose two walks are
over the **same immutable `ops: &[IlOp]` parameter**, the second strictly after
the first, with no rebinding in between:

* the **first** walk (`:2252`–`:2288`) destructures `[b, v, IlOp::StoreInd{..},
  tail @ ..]` and returns `Err` unless `b` is `IlOp::Load`, advancing by three —
  so on success `ops.len() % 3 == 0` **and every 3k-th op is a `Load`**;
* the **second** walk (`:2453`–`:2461`) re-walks the same slice in threes and
  asks `if !matches!(b, IlOp::Load(_))`.

That condition can never be true. **No witness could ever kill the `L9` mutant**,
so `L9`'s GREEN says nothing about the witness suite — it is a dead backstop.
The file already carries a recorded instance of exactly this class **sixteen
lines above the dead site**: *"`codegen::leaf::store`'s `value_bound` refusal was
a backstop with no reachable input (`w-mrslot` §5.1 — board #1218…)"*.

**The consequence for the census is precise and it cuts the mechanism further:**
of `GROUP_SHAPE`'s four sites, one (`:2254`) is publicly reachable — a bind run
containing an **FP store** yields a 2-op group (`finish_fp_store_stmt`,
`leaf_store.rs:1042`) that mis-aligns the 3-op window — one (`:2456`) is dead,
and two (`:2257`, `:2285`) could not be constructed by reading, because
`parse_store_stmt` hard-codes `IlOp::Load(base_tok)` in slot 0 and
`admissible_operand` (`:645`) is *exactly* `:2285`'s accepting predicate. So the
k = 4 that F2's whole argument rests on is **1 confirmed-reachable, 2 unknown, 1
provably dead**.

> **This is a reading, not a measurement**, and it is labelled as one. The
> decisive experiment is §7 F1 and it is priced there. Note the asymmetry that
> makes it matter: **a dead site and an unguarded site are indistinguishable to a
> mutation census**, because neither can be killed. A census counts GREEN; it
> cannot tell "no test reaches this" from "nothing can".

### 4.5 So: can a guard bind ALL raise sites of one key? Yes — and it needs nothing inside `c2-il`

F2's proposed fix was *"each raise site carries a distinct `#[cfg(test)]`-visible
discriminant, or the witness table is required to cover each site."* **The first
half is unnecessary.** The correct construction is the second half alone:

> **A witness TABLE with one row per RAISE SITE, keyed on the input that reaches
> that site, asserting the published key string. It catches a swap at any site
> even when several sites share one key, and it is constructible from OUTSIDE the
> crate wherever the site is reachable at all — which it must be, or the site is
> dead.**

This lane lands that table (`ARMS`, four rows, each naming its `file:line` and
the arm text it pins) and demonstrates the site-level property a key-level
assertion cannot express (§2.2). What it does **not** do is demonstrate the form
on a family where two sites share one key — that was registered at P = 0.20 and
is a **miss** (§6). It is a miss whose consolation is larger than the
demonstration would have been: the population was *measured* instead, and of the
three k ≥ 2 keys, one is already fully site-guarded (§4.3), one is trivially
input-distinguishable from outside (`DATA_SYM_STRLIT_FENCED`'s two sites are
mutually exclusive arms of one `match` — one is the pre-parse `sym_fail` probe,
the other a `Some(f)` post-parse gate, and no input reaches both), and one is
mostly dead (§4.4).

**The residual hard case is not "one key, several sites". It is "a site no input
can reach", and the response to that is deletion, not a guard.**

---

## 5. The re-measured census — 30 GREEN → 26, with its scope stated

Only the affected rows were re-run, as instructed.

| | count | how |
|---|---:|---|
| GREEN at `3835469c` (`w-mutcensus` §0) | **30** | published |
| **measured RED here** | **4** | `CS5`–`CS8` = `R5`–`R8` GREEN → `G5`–`G8` RED, full-suite runs, logs tracked |
| **re-measured GREEN** | **26** | 30 − 4 |

**Three caveats, because the flat number is the least interesting part:**

1. **26 is 4 measured and 26 reasoned.** The other 26 GREEN rows were *read*, not
   re-run: none of this lane's five tests can reach them. The three arm witnesses
   drive `census_functions()` only (never `dyninit_tu`/`data_tu`, which is `D1`
   and `D2`); their unresolved cells return `None` from `shape_to_function` and
   never reach the `Some(f)` gates (`CS9`); their positive-control cells build
   cleanly, so `Some(f)` gate removals do not move them; and `A1` counts arm
   *patterns* and `=>` occurrences, which a key swap inside an arm body
   (`CS2`, `CS3`, `CS4`) does not change.
2. **15 of those 26 are UNVERIFIED at this tree, for a reason that is not this
   lane's** — `CS2`, `CS3`, `CS4`, `CS9`, `B2`–`B8`, `BU3`, `D1`, `D2`, `G2` all
   sit in files peers rewrote since `3835469c` (`census.rs` +146, `bind.rs` +205,
   `bundle.rs` +327, `gl.rs` +269). Their GREEN is a fact about `3835469c`. The
   remaining 11 (`CA2`, `CA6`, `CA8`, `CA9`, `CA10`, `CA13`, `CA16`, `CA18`,
   `L2`, `L3`, `L9`) are in `calls.rs` and `leaf_store.rs`, which **no peer
   touched**, so their site text is byte-identical.
3. **`L9` should arguably not be in the denominator at all** (§4.4). If a dead
   site is not a fence, the census's N is 62 and its X is 29 before this lane
   touched anything. That is a question about the frame, not a correction to it,
   and it is not this lane's to make.

---

## 6. Prereg scorecard — 33 registered colours, 30 hits, 3 misses

| id | prediction | P | outcome |
|---|---|---:|---|
| **P1** | all four guardable from `c2-harness`, **zero bytes** in `c2-il` | 0.75 | **HIT** |
| **P2** | cell F keys `callee-unresolved-framed-call` | 0.60 | **HIT** |
| **P3** | cell Q keys `callee-unresolved-call-sequence` | 0.55 | **HIT** |
| **P4** | cell E keys `callee-unresolved-dtor-delegation` | 0.50 | **HIT** |
| **P5** | cell T1 keys `callee-unresolved-tail-call` | 0.90 | **HIT** |
| **P6** | T2 reaches the same key under a **different** label | 0.60 | **HIT** — `void-tail-call` beside `multiarg-tail-call` |
| **P7** | the four family keys pairwise distinct | 0.90 | **HIT** — asserted as a count of 4 |
| **P8** | every cell yields exactly one census row | 0.70 | **HIT** |
| **P9** | ≥1 cell needs a transcript repair **before it produces a row at all** | 0.65 | **MISS** — all five produced a row on the first attempt. The four body-only transcripts needed the `4F 1F` header wrap to reach the intended *label*, but **without** it they still produced a row (keyed `formals-marker:mid`), so the registered condition did not occur |
| **P10** | ≥1 site proves **unguardable from the harness side** | 0.20 | **MISS, in the lane's favour** — all four were guardable. Recorded as a miss, not reframed |
| **P11** | re-measured census GREEN = **26** | 0.70 | **HIT** (§5, with its scope) |
| **P12** | an all-raise-sites guard is **expressible**, and the general form stated and priced | 0.80 | **HIT** (§4.5) |
| **P13** | that form **demonstrated** on a family where ≥2 sites share one key | 0.20 | **MISS** — not demonstrated. The population was measured instead (§4.2–§4.4) |
| **P14** | suite ends at 1,660 + k, 1 ≤ k ≤ 6 | 0.80 | **HIT** — k = **5** |
| **P15** | 878-TU scan, 0 deltas over 394 keys at both ends | 0.97 | **HIT** — 0 of 394, *including* `fnbyte-*`; the #3249 movement is against the **briefed** figure, not between this lane's ends (§1.1) |
| **P16** | `gate.sh --require-graded` PASS with the debug-lane row | 0.92 | **HIT** — both ends |
| **R5–R8** | all four **GREEN** at `44794fa4` | 0.85 ea | **4 HITS** |
| — | P(all four still GREEN, i.e. none incidentally guarded) | 0.70 | **HIT** |
| **G5–G8** | all four **RED** with the guards in | 0.90 ea | **4 HITS** |
| **C1a**, **C1b** | RED, failing set **exactly** the G1 pair by name | — | **2 HITS** — 1,663/2 both times, identical sets |
| **N1** | RED with every assertion byte-identical | 0.80 | **HIT** — 1,662/3 |
| **A1** clean / under G | GREEN / RED | 0.95 / 0.90 | **2 HITS** |
| **D6a** / **D6b** / **D6c** | INVALID-not-GREEN / RED by name / GREEN | 0.90 / 0.90 / 0.95 | **3 HITS** |

**The three misses are P9, P10 and P13, and none is reframed as a bonus.** P10
and P13 are both registered-low predictions that did not occur; P9 is a
registration that was simply wrong about the instrument.

---

## 7. Gate evidence

| check | result |
|---|---|
| `cargo test --workspace --release --no-fail-fast` | **1,665 passed / 0 failed / 45 targets** at the tip (`N0T`); **1,660 / 0 / 43** at the base (`N0R`) — above 1,660 as required |
| `scripts/gate.sh --jobs 4 --require-graded` | **PASS (HATCH-RED REFUSED) at BOTH ends.** 18/18 lanes PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT · **6,948** fixture-verdicts · sweep `checked=19556 mismatches=0 graded=19460` · cross `checked=90812 mismatches=0 graded=90424` · debug-lane 18/18, 6,948 graded, **0 panic** |
| **per-lane gate-count identity diff** | **0 of 28 rows differ**, and the **range LENGTH is asserted at both ends** (28 = 28) — a diff of two empty ranges also returns 0. This is the discriminating check for a lane that lands tests (board **#3215**) |
| graded tree | `5b550a38d90b` (**738** files) → `24aad38816d7` (**740** files). **+2 = exactly this lane's two new test files.** A test-landing lane cannot claim tree identity and does not claim it |
| release-binary sha256 across worktrees | **NOT compared** — board **#3224**: `CARGO_MANIFEST_DIR` is compiled in, so the comparison is void |
| 878-TU workload scan | `match` **26** · `mismatch` **0** · `codegen-gap` **0** · `vocab-gap` **844** · `capture-fail` **8** — **identical at both ends**, asserted by name |
| `gap-metric` identity | **394 keys, 0 differing lines**, base vs tip, compared line for line |
| `scripts/debug_lane.sh` | inside the gate's fourth row: **18 lanes, 0 failed, 0 panics, 0 mismatch** |
| `scripts/board_audit.sh` | all-zero |
| `crates/c2-harness/tests/rung_registry.rs` | **2/2** |
| `docs/rungs/INDEX.md` | regenerated by `scripts/gen_rung_index.sh`, never hand-edited |

---

## 8. Found and not taken, ranked

### F1 — `leaf_store.rs:2456` is dead code, and the decisive experiment is one suite run

§4.4 establishes it by reading the two walks of `bind_run_ops`. **The experiment
that would settle it costs one suite run plus one gate run (~15 min)**: replace
`return Err(STORE_RUN_BIND_GROUP_SHAPE)` at `:2456` with `panic!()` and run the
workspace suite, the 19,556-case generated sweep, the 90,812-cell mode cross and
the 878-TU scan. Nothing panicking upgrades "dead by reading" to "unreached by
every corpus this project owns" — which is still not a proof, and the reading is
the stronger of the two arguments.

**Not taken because it is a `c2-il` edit** and `c2-il` is `w-dataseam`'s seam
this wave. It is a five-minute rung for whoever owns that file next, and the
right resolution is **deletion with a comment**, not a guard: `leaf_store.rs`
already records one dead backstop sixteen lines above this one (`w-mrslot` §5.1,
board #1218), so this would be the file's second.

### F2 — a mutation census cannot distinguish a DEAD site from an UNGUARDED one, and nothing in `w-mutcensus`' frame notices

This is the generalization of F1 and it is worth more than the instance.
**Both a dead site and an unguarded site produce GREEN, by the same mechanism:
no test fails.** `w-mutcensus`' §2 enumeration is textual (`refuse(`,
`Block::at_end(`, constant raises), so a dead raise site enters `N` exactly like
a live one and inflates `X` exactly like an unguarded one. `L9` is one confirmed
instance out of 63; `w-mutcensus` §2.1's **1,227-site grammar class** was
dropped from that census for budget and is the population where this would
matter most.

The cheap discriminator is the one F1 names: a site whose mutation is GREEN
should be probed **once** with `panic!()` rather than a key swap. A panic that
never fires is a dead site; a panic that fires is a live unguarded one. **That is
a second colour per GREEN row, not a second campaign**, and it would partition
`X` into "unguarded" and "unreachable" — two very different backlogs, currently
summed.

### F3 — `DATA_SYM_STRLIT_FENCED`'s two raise sites entered the tree after the census froze and were never mutated

`census.rs:1259` and `:1511`, landed by `w-fence163` (`d28326b4`) *during*
`w-mutcensus`' campaign — its own §2.2 records the constant and says the frame
necessarily misses it. Both sites are **input-distinguishable from outside**
(§4.5), so two rows in this lane's table shape would close them, and the cells
are cheap: `w-fence163` already built the narrow/wide literal `.gl` records and
`w-guards`' `gl_strlit` helper is in the same module as this lane's `ARMS` table.
**Not taken for budget**, and it is the single cheapest follow-on here.

### F4 — the standing site count exists for ONE dispatch, and `w-mutcensus` F4 asked for it over all 63

`crates/c2-harness/tests/callee_unresolved_sites.rs` is F4's *"gate row that
compares that count against a checked-in expectation"*, scoped to the
`match label` block. The general version — run `work/w-mutcensus/enumerate.sh`
and compare its site count against a checked-in number — is still not landed, and
F4's two stated blockers (the byte-delta rule; `debug_lane.sh` being wired into
`gate.sh` by a live peer) **have both expired**: this lane lands test code by
design, and `w-gatewire`'s row is in the gate now. It is one test in
`crates/c2-harness/tests/`, in the same shape as the one landed here.

### F5 — the four transcripts are now a THIRD copy of captures that live in `c2-il`

`w-guards` §8 item 4 recorded this for the `.gl` record builders; this lane
copies four whole `.ex` segments as well. Two spellings of one format is how a
format drifts, and there are three now. The fix is a `pub` on `c2-il`'s
`test_fixtures` (or a small `pub` accessor behind a feature), which is a `c2-il`
edit and therefore not this lane's. **The mitigation that IS in place**: every
transcript's doc comment says it is a copy of a capture and that breaking it
means re-deriving from the capture, never deleting the test.

### F6 — `require_toolchain` is landed but nothing SETS the variable

`C2RS_REQUIRE_TOOLCHAIN` is inert by default, which is the correct contract — the
demand belongs to the caller. But **no caller sets it today**, so the instrument
is armed and unused. The obvious callers are `scripts/gate.sh` (which already has
`--require-graded` and the same argument in its header) and the merge funnel's
suite row, which is the row `w-mutcensus` F1 points at: *"quoted as evidence in
essentially every rung doc in `docs/rungs/`"*. Both are `scripts/` edits and a
convention change, which is a coordinator decision rather than a lane's.

### F7 — `w-mutcensus` §4.2's mechanism is published as the census's headline explanation and it is not

§4 is the correction, with numbers. The rung's §0 says *"it is the rule, and the
axis it runs along is whether a site raises a KEY or decides a GATE"*, and §4.2
supplies the mechanism. **The key/gate axis survives** — 75 % GREEN for swaps
against 44 % for removals and 18 % for widenings is a real and reproducible
ordering, and it is the census's best result. **The `k − 1` mechanism does not**:
it covers at most 5 of 30, is contradicted by `L6`/`L7` in the census's own
table, and its worked example is dead code. Said here rather than left implicit,
because F2 is currently the reason a future lane would reach for an in-crate
`#[cfg(test)]` discriminant, and it does not need one.
