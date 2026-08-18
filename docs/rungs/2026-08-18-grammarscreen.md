# GRAMMARSCREEN — the 1,227-site grammar class `w-mutcensus` dropped is 1,225 sites, the corpus reaches 507 of 1,336, and the three-bucket partition DEGENERATES: the dead bucket is EMPTY and the only unreachability proof that scales is a compiler lint that is already green

    Tag:       GRAMMARSCREEN
    Slug:      grammarscreen
    Date:      2026-08-18
    Kind:      characterization — `w-deadsites` F1: *"the same screen makes the
               dropped 1,227-site grammar class affordable — ~20 runs vs 5
               days."* Run it, re-derive the 1,227 by PARSING, and take
               `w-deadsites`' THREE-bucket partition on the result
    Outcome:   instrument
    Fixtures:  none — characterization
    Census:    +0 — the 878-TU scan is identical on all **394** prefix-anchored
               `gap-metric` keys, base against every probe run. **Every**
               `crates/` edit in this lane is an applied-and-reverted probe;
               `git diff master..HEAD -- crates fixtures scripts` is **EMPTY**
               at the tip
    Record:    this file; prereg
               `docs/rungs/_2026-08-18-w-grammarscreen-prereg.md` (frozen at
               `f40c4b1f6`, committed BEFORE the first probe); deviations
               `work/w-grammarscreen/deviations.md`; the enumerator, the probe,
               the patcher, the re-derivation and every raw log under
               `work/w-grammarscreen/` (tracked); board rows **#3299**–**#3303**,
               allocated by the coordinator

Provenance: `docs/rungs/2026-08-18-deadsites.md` §10 **F1** — *"19 of 26 rows
are unreached. `w-mutcensus` §2.1 dropped a **1,227-site** grammar class for
budget, and `#3246` says the partition matters most there. This lane's screen
makes that affordable."* This is the lane that runs it.

---

## 0. The answer

**The population, re-derived by parsing Rust rather than grepping it:**

| | raw `grep -n \| wc -l` | **parsed call sites** |
|---|---:|---:|
| `blk(` | **1,227** | **1,225** |
| `blk_type(` | **6** | **5** |
| `Block::refuse(` | **106** | **106** |
| **total** | 1,339 | **1,336** (1,335 production + 1 in a test) |

**`1,227` moves, and it moves DOWN by exactly two nameable lines** — a **doc
comment** in `expr.rs:1357` that quotes `blk(seg, p, "body")`, and the `fn blk(`
**definition**. The other half of the reconciliation is the half worth
recording: **no line in this population carries two call sites**, so the
line-count/site-count gap that makes a line grep an *under*-count is zero here.
That is a property this population happens to have, not a property of grep.

**The screen, over all 1,336 sites at once:**

| bucket | n | of | share |
|---|---:|---:|---|
| **REACHED** — the corpus evaluates this site | **507** | 1,336 | **37.9 %** |
| ├─ *and is proven to REFUSE at it* (diverging forms) | **321** | 1,126 | 28.5 % |
| └─ *evaluated only, refusal NOT proven* (`.ok_or(blk(..))`) | **186** | 210 | — |
| **QUIET** | **829** | 1,336 | **62.1 %** |
| ├─ quiet in a function the corpus **demonstrably enters** | **737** | | a statement about the BRANCH |
| └─ quiet in a function **no site of which ever fired** | **92** | | a statement about DISPATCH |
| **DEAD** — quiet **and** a source-level proof | **0** | 1,336 | **0.0 %** |
| **UNKNOWN** — quiet, no proof | **829** | 1,336 | **62.1 %** |

**The table is not the finding. These four are.**

> **1. THE THREE-BUCKET PARTITION DEGENERATES ON THIS CLASS, AND IT DEGENERATES
> FOR A STRUCTURAL REASON RATHER THAN A BUDGETARY ONE.** `w-deadsites` split 26
> rows into 7 unguarded / 4 dead-with-a-proof / 15 unknown by **reading each
> quiet site**. That does not scale to 829, and the one unreachability proof
> that *does* scale over a thousand sites — **rustc's own `dead_code` lint** —
> is **already run on every build of this repo and is green**: `cargo build
> --release -p c2-il` emits **zero warnings**, and exactly **one** of the 191
> enclosing functions has no non-test caller (it *is* a `#[test]`). So the DEAD
> bucket is not small here; it is **empty**, and the partition on this class is
> **reached / unknown**, two buckets, with everything actionable in the first.

> **2. QUIET IS NOT "THE CORPUS NEVER ENTERS THIS PARSER" — IT IS A SPARSE
> TRAVERSAL OF EVERY PARSER.** **Zero** of the 30 files carry zero reached
> sites, and in 25 of the 30 the corpus reaches a site **within the last 15 %**
> of the file's gates while touching only 20–35 % of them. `float_walk_loop.rs`
> is 24 of 89 sites reached and the **deepest reached site is the 89th**. The
> intuitive story that would license deleting quiet sites — *"that shape never
> occurs, so its parser is dead"* — is **measured false, file by file.**

> **3. A THIRD OF THE REACHED BUCKET IS EVALUATED AND NOT REFUSED.** `.ok_or(X)`
> evaluates `X` on every pass, refusing or not — and `?` after it does not
> change that. **186 of the 507** reached sites are in eager position, so the
> probe proves control ran the expression and proves **nothing** about a witness
> existing. `w-deadsites` priced its UNGUARDED bucket as *"a witness each, and
> cheap: the probe proves an input exists"*; on this class that claim is sound
> for **321 sites, not 507**. A screen that does not separate the two overstates
> its own actionable half by **58 %**.

> **4. THE SCREEN IS CHEAPER THAN F1 COSTED IT BY AN ORDER OF MAGNITUDE, AND
> WHAT IT BUYS IS A PRICE.** F1 sized this at *"20 bitmask words, or ~20 runs"*
> against `w-mutcensus`' **five days serial**. It is **three source lines and
> four corpus runs** — `blk`, `blk_type` and `Block::refuse` marked
> `#[track_caller]`, because one `&'static Location` exists per call site and
> its ADDRESS is the site key. And the deliverable is not the count: a mutation
> campaign over this class now has a **necessary condition** to test against, so
> it is **507 runs, not 1,336** — the screen retires **62 %** of it before the
> first mutant.

**Method checks, all four green.** `P1` and `P2` — two full corpus passes —
agree **SET-for-SET on all 1,336 rows in every one of the six stages**, not
merely on counts (**#3286**: a count cannot see one byte different at the same
count). The instrumented runs reproduce the clean baseline **exactly**: suite
**1,671 / 0 / 46**, gate PASS 18/18 at every count, 878-TU scan **0 differing
lines over all 394 anchored keys**. The `panic!()` confirmation over **all 829
quiet sites at once** is §3.4. And the panic instrument carries a **positive
control**: deleting one reached address from its allowlist makes it fire and
name itself, so "no marker" is a measurement rather than an absence.

**Prereg: 14 registered colours, 11 hits, 3 misses** (§8). The headline
registration **REACHED = 534 (40 %), 80 % interval [400, 735]** against observed
**507 (37.9 %)** — inside, near the point estimate, and the calibration that
produced it was `w-deadsites` §8.1's explicit instruction to *register less
reach than intuition suggests*, applied before the number was written.

---

## 1. Populations, every one measured at this lane's own base

Base **`666fe6eb7`**, no rebase. `N0` is the clean tree in this worktree.

| tag | population | measured here |
|---|---|---|
| **P-T** | `C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release --no-fail-fast` | **1,671 passed / 0 failed / 46 targets**, `census_gate` **66.51 s** |
| **P-G** | `scripts/gate.sh --jobs 16 --require-graded` | **PASS (HATCH-RED REFUSED)**, **124 s**, 18/18 lanes · **6,948** fixture-verdicts · sweep `checked=19556 graded=19460 mismatches=0` · cross `checked=90812 graded=90424 mismatches=0` · debug lane 18/18, **0 panic** |
| **P-W** | 878-TU workload scan | `match` **26** · `mismatch` **0** · `codegen-gap` **0** · `vocab-gap` **844** · `capture-fail` **8** |
| **P-K** | `gap-metric` keys, `^ *gap-metric \S+ \S+$` | **394** (#3269 — never the naive `grep -c`) |
| **P-F** | `fnbyte-*` | exact **35,899** · differs **1,958** · refused-parse **113,447** |
| **P-C** | the named control `C1` (`calls.rs:431`, `syms > 1` → `syms > 2`) | **RED 1,669 / 2**, failing exactly `the_call_argument_arity_fence_is_a_series_and_admits_exactly_one_symbol` and `the_two_symbol_thunk_exemption_turns_on_the_bare_body_marker_alone`, `census_gate` **83.71 s** |

**Every figure matches the dispatch brief digit for digit**, so the frame is not
stale and no invalidation rule fired.

### 1.1 The corpus this lane probed against, stated because it is the whole content of the QUIET claim

* `c2-rs` at **`666fe6eb7`**, worktree
  `.claude/worktrees/w-grammarscreen`, `compilers/` symlinked to the main repo
  (the fixture-census hard gate re-verified: `w5_chain.cpp -> 4/4 in class`).
* **`dc3-decomp` at `ccd4c8036`**, clean — the 878-TU workload.
* **`work/capture-cache` SHARED** with the main repo (resolved through the
  common git dir), and the generated corpus **shared and content-addressed**
  (`gen-169fae960ed84b63`, `w-coldcross`' `scripts/corpus_dir.sh`). The cross
  and sweep legs were therefore **warm**, and the `mode_cross` cold leg
  (**#3266**) was not paid.
* wibo `1.2.0-c2rs.1`; `cl.exe` / `c2.dll` / `c1xx.dll` from
  `compilers/X360/16.00.11886.00`.
* Stages covered: **the 1,671-test workspace suite · the 19,556-case generated
  sweep · the 90,812-cell mode cross · the 18-lane release fixture gate · the
  18-lane debug-profile lane · the 878-TU workload scan.** `hatch-red` is
  **REFUSED** here, on the byte-clean base as well as under
  `--allow-dirty-crates` (deviations **D6**), exactly as `w-deadsites` §1.1
  recorded for master.

**#3254 binds this section**: 71.2 % of the `fnbyte` denominator is bodies the
shipped image never contains, and `w-corpushealth` records the workload as one
head of a tree moving ~284 commits a fortnight. A QUIET row is a statement
about *that* corpus on *that* day and about nothing else.

---

## 2. The enumeration — parsed, and every difference from the grep named

### 2.1 The rule

`work/w-grammarscreen/enumerate.py` is a hand-rolled Rust lexer: line comments,
doc comments, **nested** block comments, string literals (normal, escaped,
byte, raw with any hash count), char literals and lifetimes are skipped, and the
call token sequences `blk` `(` / `blk_type` `(` / IDENT `::` `refuse` `(` are
found in what remains, excluding each `fn` definition. Per site it records
file, line, **column**, the `ctx` argument (literal or path), the syntactic
form, and whether the site is inside `#[cfg(test)]`.

### 2.2 The reconciliation — the check #3288 asks any denominator to carry

| population | grep-only | parse-only | lines carrying >1 site |
|---|---:|---:|---:|
| `blk(` | **2** — `expr.rs:1357` (doc comment) and `mod.rs:1856` (`fn blk(` definition) | **0** | **0** |
| `blk_type(` | **1** — the `fn blk_type(` definition | **0** | **0** |
| `Block::refuse(` | **0** | **0** | **0** |

### 2.3 The enumerator reproduced #3288's failure mode INSIDE the instrument built to answer it

Recorded in full as deviations **D1**. `enumerate.py`'s first version captured
each token's text **after** advancing the cursor, so every identifier came back
as the empty string, nothing matched, and it printed

    files scanned: 65
    sites parsed:  0

**with exit 0**. A silent, maximally flattering zero. It was caught by the
reconciliation against the raw grep — the check the prereg registered for
exactly this — and by nothing else. **The transferable half is that #3288's
mitigation works and is worth the ten lines**: an enumerator that carries a
second, differently-built count cannot publish a silent zero.

### 2.4 What a `ctx` is, and why the site key had to be a LOCATION

**237 of the 1,336 sites pass a `ctx` VARIABLE, not a literal** — `what` at 210
sites, `ctx` at 13, `key`, `site.badtoken`, `region(ix)`. A further **34 ctx
literals are shared by 2 or more sites**. So a probe keyed on the census key
could not have addressed this population at all: it would have merged 305 sites
into 34 rows and been blind to 237 more. `#[track_caller]` keys on the **call
site**, which is the unit the question is asked in.

---

## 3. The screen — three source lines for 1,336 sites

### 3.1 The instrument

`blk`, `blk_type` and `Block::refuse` are the **only** constructors of this
class. Marking those three `#[track_caller]` and asking
`std::panic::Location::caller()` who called them identifies the exact site,
file/line/**column**, with **no per-site edit**. One `&'static Location` exists
per call site, so its address is the site key and first-hit dedup is a
thread-local `HashSet<usize>`.

`w-deadsites` needed one bitmask entry and one `hit(ix, "ID")` call **per
site**, which is what made F1 cost 1,227 sites at ~20 runs. This is **4 files
touched, 7 inserted lines, and one build**.

Two hardening details that are not decoration:

* **`SEEN.try_with`, never `SEEN.with`.** A `thread_local!` access during
  thread destruction **panics**, and a probe that can panic can invalidate the
  run it is measuring. On failure the dedup is skipped and the hit is recorded
  **anyway** — a duplicate costs a `sort -u`, a dropped hit is an under-count,
  which is the flattering direction (#3288).
* **One `write_all` of one buffer, never `writeln!`** — inherited verbatim from
  `w-deadsites`' probe, whose first run produced an interleaved half-line.

### 3.2 Behaviour preservation is the instrument's own validity check

`hit` returns `()`, touches no program state and reads one environment
variable, so an instrumented corpus run must reproduce the clean baseline. It
does, at every count:

| | `N0` (clean) | `P1` (screen) | `P2` (screen) | `Q1` (`panic!()`) |
|---|---|---|---|---|
| suite | **1,671 / 0 / 46** | **1,671 / 0 / 46** | **1,671 / 0 / 46** | **1,671 / 0 / 46** |
| `census_gate` differential | 66.51 s | 114.55 s | 69.50 s | 73.49 s |
| gate | PASS 18/18, 6,948 verdicts | identical | identical | §3.4 |
| sweep / cross | 19,556 / 19,460 · 90,812 / 90,424, 0 mismatch | identical | identical | §3.4 |
| 878-TU scan, all 394 anchored keys | — | **0 differing lines** | **0 differing lines** | §3.4 |

**No differential anywhere near 0.00 s**, which is what an ungraded run reads
(#3231).

### 3.3 `P1` and `P2` agree SET-for-SET, not count-for-count

| stage | `P1` | `P2` | set-identical | `P1\P2` | `P2\P1` |
|---|---:|---:|---|---:|---:|
| suite | 280 | 280 | **yes** | 0 | 0 |
| sweep | 133 | 133 | **yes** | 0 | 0 |
| cross | 232 | 232 | **yes** | 0 | 0 |
| debug lane | 446 | 446 | **yes** | 0 | 0 |
| gate | 469 | 469 | **yes** | 0 | 0 |
| 878-TU scan | 339 | 339 | **yes** | 0 | 0 |
| **union** | **507** | **507** | **yes** | 0 | 0 |

Stated as sets deliberately. **#3286** — *"a count cannot see one byte
different at the same name and the same count"* — is the reason a replication
reported as `507 = 507` would not have been one.

### 3.4 The `panic!()` confirmation, batched over all 829 quiet sites

Second mode of the same module: given the file of reached addresses, any site
**not** in it `panic!()`s and names itself. A run that completes clean confirms
every quiet site simultaneously — `#3246`'s named probe, batched, and here over
**829** sites rather than 20.

**The instrument carries a POSITIVE control, and this is the part
`w-deadsites`' panic run did not have.** A `panic!()` that never fires is
indistinguishable from a `panic!()` that cannot fire — the absence-reads-as-
success shape this repo has now hit at least four times. So one **reached**
address (`expr.rs:1826:36`) was deleted from the allowlist and the same binary
re-run on one fixture:

```
thread 'main' panicked at crates/c2-il/src/grammarprobe.rs:92:13:
w-grammarscreen QUIET SITE REACHED crates/c2-il/src/func/body/expr.rs:1826:36
```

**`Q1`, with all 829 quiet sites armed:**

| | |
|---|---|
| suite | **1,671 / 0 / 46**, differential **73.49 s** |
| generated sweep | `checked=19556 graded=19460 mismatches=0` |
| mode cross | `checked=90812 graded=90424 mismatches=0` |
| debug-profile lane | 18/18, **0 panic** |
| gate | **PASS (HATCH-RED REFUSED)**, 18/18 lanes, **6,948** fixture-verdicts |
| 878-TU scan | `match` **26** · `mismatch` **0** · `port-error` **0** · `vocab-gap` **844**; **0 differing lines over all 394 anchored keys** against `N0` |
| **`w-grammarscreen QUIET SITE REACHED` markers, grepped from the raw text of every log AND of the gate's whole `/tmp/c2rs-gate-1799346` run tree** | **NONE** |
| any bare `panicked` in any log | **NONE** |

**So all 829 quiet sites are confirmed simultaneously**, in a run whose suite,
gate and scan figures are identical to the clean base — and `port-error 0` is
worth its own mention, because a panic the harness caught and counted would
have shown there.

Markers are grepped from the **raw text** of every log, never inferred from an
exit code: the gate carries a `panics=` column and a caught-and-counted panic
leaves no trace in a status (`w-deadsites` §3.2).

---

## 4. The partition, and why it has two buckets here and not three

### 4.1 REACHED — 507, of which only 321 prove a witness exists

| position | sites | reached | what a hit means |
|---|---:|---:|---|
| `return Err(blk(..))` and friends | 1,116 | **311** | **REFUSED** — the parse returned through this site |
| `Err(blk(..))?` | 10 | **10** | **REFUSED** |
| `.ok_or(blk(..))` / `.unwrap_or(..)` | 169 | **167** | **EVALUATED ONLY** — `ok_or` runs its argument on every pass |
| unclassified (tail-position `Err(..)`, closure bodies) | 41 | **19** | counted **conservatively as not refusal** |

**The eager/diverging test runs BEFORE the `?` test, and that ordering is
load-bearing.** `x.ok_or(blk(..))?` *looks* diverging and is not: `ok_or`
evaluates its argument whether or not the `?` returns. A classifier that tested
`?` first read 177 sites as diverging that are not.

So the honest sentence is: **the corpus is proven to refuse at 321 sites of
1,126 in refusing position (28.5 %), and to have merely run the expression at a
further 186.**

### 4.2 DEAD — 0, and the reason is structural

A DEAD row needs quiet **plus a source-level proof of unreachability**.
`w-deadsites` produced 4 such proofs by reading 26 sites. Reading 829 is not a
lane. What *is* affordable is the mechanical proof classes, and there are
exactly two, both run here:

1. **rustc's `dead_code` lint.** It proves "no caller exists", it runs on every
   build of this repo, and `cargo build --release -p c2-il` is **warning-clean**.
   An independent parse of the crate agrees: of the **191** functions enclosing
   a site, **one** has no non-test caller, and that one *is* a `#[test]`.
2. **"the enclosing function is never entered."** 50 of the 191 enclosing
   functions have zero reached sites, holding 92 quiet sites. **This is not a
   proof** — those are `eat_*` helpers whose callers simply refused earlier —
   and it is published as a sharpening of UNKNOWN, not as a promotion out of it.

**Declared reading, no silent cap.** One sub-population was read site by site:
the **11 quiet sites that share a `ctx` literal with a reached site** — the
exact shape `w-deadsites`' `L9`, `CA9` and `CA10` came from, and the population
where a proof was most likely. All 11 were read. **None admits a proof.** Two
examples, so the class is legible: `calls.rs:1326` is `k.checked_neg()`
returning `None`, reachable for exactly one literal value the corpus never
carries; `float_walk_loop.rs:353` is a `0x26` designator byte that was present
every time the parse reached it. Both are statements about the corpus.

**So: DEAD = 0, and it is 0 because the proof does not scale, not because the
sites are alive.** That is the honest form of the answer and it is the one
`#3246`'s "delete the code" instruction cannot be given here at all.

### 4.3 UNKNOWN — 829, split by whether control ever arrives

| | n | what is established |
|---|---:|---|
| quiet in a function the corpus **demonstrably enters** | **737** | control **reaches this function** and never takes this branch. A statement about the BRANCH — and the population where a witness is plausibly cheap, because the parse already gets there |
| quiet in a function **no site of which ever fired** | **92** | control may never arrive at all. A statement about DISPATCH |

**Neither is dead. Nothing in this lane is deleted, and nothing is proposed for
deletion.** The brief named this as the error this project keeps making and the
prereg registered it as **H11** at 0.97 for the same reason.

### 4.4 The `k − 1` mechanism, measured on this class — and it is 1.3 %

`w-mutcensus` **F2**: a per-key witness suite guards one raise site per key, so
a key with `k` raise sites contributes `k − 1` unguarded sites by construction.
`w-calleeguard` measured that it explains **at most 5 of 30** on the census
class. On the grammar class:

* **1,060** distinct `ctx` literals over 1,099 literal-ctx sites; **34** are
  shared by 2+ sites (73 sites).
* **11** keys have a reached site **and** a quiet sibling → **11 quiet sites**,
  **1.3 %** of the 829.
* **685** ctx literals are **wholly quiet** — census keys that nothing in the
  entire corpus has ever produced.

**So F2's mechanism, which explains 17 % of the census class, explains 1.3 %
here.** The grammar class's quiet population is not a witness-suite artifact at
all; it is corpus reach, and almost nothing else.

### 4.5 Does `w-deadsites`' 73 % hold?

**No — it is 62.1 %, and the comparison needs its denominators stated or it is
meaningless.** `w-deadsites` measured **19 of 26** *open GREEN rows* — sites a
mutation census had already found unguarded. This lane measures **829 of 1,336**
*sites*, mutated or not, because **no site in this class has ever been
mutated**.

The two are not the same ratio, and the honest reading is the stronger one:

> On the census class, 73 % of the **already-suspect** population turned out to
> be a fact about corpus reach. On the grammar class, **62 % of the ENTIRE
> population is a fact about corpus reach, established before a single mutation
> is run** — so for 829 sites a mutation census could not have measured test
> quality even in principle, and would have spent 829 of its 1,336 suite runs
> proving that.

**That is the operational result.** A mutation census over this class is now
**507 runs**, not 1,336 — and the 62 % it retires is retired by **four corpus
runs**.

---

## 5. Reach attributed to a corpus STAGE — `w-deadsites` F2, taken

F2 recorded that lane's probe as knowing *that* a site fired and not *where*,
so all seven of its UNGUARDED rows were priced identically. One
`C2RS_GRAMMARPROBE_LOG` per stage fixes it, and the answer is not what the
stage sizes suggest.

| stage | corpus | sites reached | **reached by NOTHING else** |
|---|---|---:|---:|
| workspace suite | 1,671 tests | **280** | **20** |
| generated sweep | 19,556 cases | **133** | **0** |
| mode cross | 90,812 cells | **232** | **1** |
| fixture lanes (debug profile) | 18 × 386 | **446** | **70** |
| 878-TU workload scan | 870 graded TUs | **339** | **17** |
| `gate.sh` (all rows) | — | **469** | — |
| **UNION** | | **507** | |

**Three things fall out of that table.**

1. **`gate.sh` \ (sweep ∪ cross ∪ debug-lane) = 0.** The 18 **release** fixture
   lanes reach not one site the debug lane, the sweep and the cross do not
   already reach. The release and debug halves of the fixture gate cover the
   identical site set — which is the positive form of `w-mutcensus` §0's
   "the two are orthogonal": orthogonal in *what they can catch*, identical in
   *what they touch*.
2. **The 19,556-case generated sweep adds ZERO exclusive reach.** Every site it
   reaches, something else reaches. It is the single largest generated corpus in
   the gate and, as a reach instrument over this class, it is redundant.
3. **The cheapest witness, per site:**

   | cheapest stage that reaches it | sites |
   |---|---:|
   | the workspace suite | **280** |
   | the generated sweep | **0** |
   | the mode cross | **46** |
   | the fixture lanes | **164** |
   | the 878-TU workload | **17** |

   **That is the price of a witness, per site, which is exactly what F2 said was
   missing.** 280 sites can be witnessed from a unit test; 17 need the whole
   878-TU workload and are the expensive ones.

### 5.1 Depth, and the finding that forbids the obvious wrong conclusion

If the quiet population were "shapes the corpus never sees", whole parsers would
be dark. **None is.** Per file, sites reached against the position of the
**deepest** site reached:

| file | sites | reached | deepest reached site | depth |
|---|---:|---:|---:|---:|
| `float_walk_loop.rs` | 89 | 24 | **89th** | **100 %** |
| `guard_chain_shared_tail.rs` | 91 | 24 | 86th | 95 % |
| `osf_handle_guard.rs` | 84 | 23 | 82nd | 98 % |
| `xtea_round_loop.rs` | 70 | 12 | 69th | 99 % |
| `ptr_walk_loop.rs` | 70 | 23 | 68th | 97 % |
| `calls.rs` | 64 | 49 | 64th | 100 % |
| … 24 more, all ≥ 83 % except one | | | | |
| **`xtea_encrypt_loop.rs`** | **57** | **5** | **14th** | **25 %** |

**Files with zero reached sites: 0 of 30.** The corpus enters every parser and
walks to within 15 % of the end of nearly all of them while touching a fifth to
a third of the gates on the way. `xtea_encrypt_loop.rs` is the single
exception — one parser the corpus enters and abandons at gate 14 of 57 — and it
is the one row on this table a follow-on lane should read first.

---

## 6. The independent cross-check — the census's own vocabulary, and the one contradiction it produced

`Block::feature()` renders a site's `ctx` into the census key the workload scan
and the fixture lanes already publish. That is **production code with no probe
in it**, so intersecting it with the reached set is a genuinely
differently-constructed count (**#3288**'s transferable rule).

| arm | features | resolved to a ctx with ≥1 site REACHED | contradictions | unresolved |
|---|---:|---:|---:|---:|
| the 878-TU scan's `blocking features` histogram (top 20 only) | 20 | **20** | **0** | 0 |
| the fixture lanes' per-TU `scan.jsonl` (`fn_blockers`, `emit_blockers`, `fn_gate_refusals`) | **231 distinct** | **194** | **1** | 36 |

**The one contradiction is the most useful row in this section, and it resolves
into a finding rather than a defect.** `mcall-chain-tail-load-class` blocks 28
fixture bodies, and the only site carrying that *literal* —
`mcall_chain.rs:500` — is **quiet**. Read: **`mcall_chain.rs:497` raises the same
key through a `ctx` VARIABLE**, and that site is **REACHED**.

> So the key is produced, by a **different raise site of the same key**, and the
> production census cannot tell the two apart while this lane's probe can. That
> is `w-mutcensus` **F2's own mechanism**, surfacing in the grammar class,
> **detected by a completely independent instrument**, and it is the concrete
> argument for site-level keying over key-level keying.

The 36 unresolved are the same population: features whose `ctx` arrives as a
variable, which a key-to-site map cannot address by construction.

---

## 7. The other classes `w-mutcensus` dropped, re-derived by parsing

The drop table is quoted as a denominator, so **#3288** says it owes a second
count. It gets one, and **three of its four figures move**:

| class | `w-mutcensus` §2.1 | **parsed, at `666fe6eb7`** | **parsed, at `3835469c` (the census's OWN base)** |
|---|---:|---:|---:|
| `IlBundle::dyninit_tu` `return None` | **12** | **11** | **11** |
| `IlBundle::data_tu` `return None` | **14** | **14** | **14** |
| `IlBundle::provide_data_tu` `return None` | *not listed* | **19** | **function does not exist** |
| shape-file `OptWordMode` gates | **18** | **19** | — |

**Three separate results, and they point in three different directions.**

1. **`dyninit_tu`'s 12 is wrong at the base it was measured on.** The count is
   **11** at `3835469c` *and* at `666fe6eb7`, read with `git show`, so head drift
   is excluded — #3269's rule (*a lane that finds an unexpected delta owes a
   measurement before it owes a cause*) is why that is a statement and not a
   guess. **And the error is an OVER-count**, which is the direction #3288 says
   is *not* the dangerous one — so this is a fourth member of that defect class
   pointing the other way, and the generalization needs widening: an
   enumerator's count is unverifiable, not merely under-stated.
2. **`provide_data_tu` is a whole function of the same class the drop table has
   never contained** — 19 more `return None` clauses, and it **did not exist**
   when the frame froze. That is `w-mutcensus` F7's staleness, third instance,
   this time **after** publication rather than during the campaign.
3. **The `return None` framing itself under-names the class.** `dyninit_tu`
   carries **10 `?` operators** and `provide_data_tu` **3** — each a fail-closed
   refusal the drop table does not count in any form. The published `12 + 14 =
   26` is, on the same reading applied consistently, at least **44 `return None`
   + 15 `?` = 59**.

**All four are OUT of this lane's probe frame** and stay out, for the reason
the prereg gave: `return None` and a comparison operator carry no callee to
hang `#[track_caller]` on, so they need a different instrument. The conditional
secondary frame of prereg §5 was **NOT RUN**, and is reported as not run with
its count — §10 **F3**.

---

## 8. Prereg scorecard — 14 registered colours, 11 hits, 3 misses

| id | registration | outcome |
|---|---|---|
| **H1** | REACHED **534 of 1,336 (40 %)**, 80 % interval **[400, 735]** | **HIT** — observed **507 (37.9 %)**, inside, just under the point estimate |
| **H2** | QUIET **802 (60 %)**, and the quiet fraction **higher** than `w-deadsites`' 73 % is low | **HIT on the count (829 vs 802), MISS on the framing** — 62.1 % is *lower* than 73 %, and §4.5 shows the two ratios have different denominators and should never have been registered as comparable |
| **H3** | the instrumented run reproduces the clean baseline exactly | **HIT** — suite, gate and all 394 scan keys identical across `N0` / `P1` / `P2` / `Q1` |
| **H4** | `P1` and `P2` agree on every one of the 1,336 rows | **HIT** — set-identical in all six stages, `P1\P2 = P2\P1 = 0` |
| **H5** | the `panic!()` run completes with zero markers | **HIT** — `Q1`, all **829** quiet sites armed at once, zero markers in any log or in the gate's run tree, and the instrument shown able to fire (§3.4) |
| **H6** | control `C1` **RED** with exactly the two named tests, base and tip | <!-- H6 --> |
| **H7** | stage attribution informative — ≥1 site reached only by the scan **and** ≥1 only by the suite | **HIT** — 17 scan-only, 20 suite-only |
| **H8** | DEAD ≤ 27 (2 %), for structural rather than budgetary reasons | **HIT, at the extreme** — DEAD = **0**, and §4.2 gives the structural reason: the only proof that scales is a lint that is already green |
| **H9** | UNKNOWN is the largest bucket, > 50 % | **HIT** — 829, 62.1 % |
| **H10** | the three-bucket partition **DEGENERATES** on this class | **HIT** — two buckets, DEAD empty |
| **H11** | the lane deletes nothing and publishes UNKNOWN untouched | **HIT** — 829 rows published, none touched, none proposed for deletion |
| **H12** | `git diff master..HEAD -- crates fixtures scripts` **EMPTY**; graded tree identical at both ends | **HIT** — §9 |
| **H13** | P(a contradiction between the probe and the census vocabulary) = **0.15** | **MISS** — one contradiction, and it **resolved into §6's finding** rather than into a probe defect. Scored a miss because the registration named the outcome and the outcome happened |
| **H14** | ≥1 shape file with **zero** sites reached, P = 0.35 | **MISS** — **0 of 30**. §5.1 is the measurement, and it is a more useful result than the hit would have been |

**Calibration: 11/14 = 79 % of registered colours correct**, and unlike
`w-deadsites`' campaign the errors are **not directional** — one framing error,
one contradiction that resolved, one structural expectation refuted.

**The one registration worth reading on its own is H1.** `w-deadsites` §8.1
closed with an instruction to the next lane: *"a future lane probing the
1,227-site grammar class should register LESS reach than intuition suggests, not
more."* That instruction was followed — intuition said ~55 %, the registration
said 40 %, the measurement said **37.9 %**. **A calibration note handed forward
between lanes was worth 15 percentage points**, and it is the first time in this
sequence one has been.

---

## 9. Gate evidence

| check | base `N0` (`666fe6eb7`) | **tip** |
|---|---|---|
| `C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release --no-fail-fast` | **1,671 / 0 / 46** | <!-- TIPSUITE --> |
| `census_gate` duration (the differential actually grading) | **66.51 s** | minimum over **every** run in this lane is **66.51 s** — none near the 0.00 s an ungraded run reads (#3231) |
| `scripts/gate.sh --jobs 16 --require-graded` | **PASS (HATCH-RED REFUSED)**, 124 s, 18/18 lanes, 0 FAIL / 0 SKIP / 0 NO-RESULT · **6,948** fixture-verdicts · sweep `19556 / 19460`, 0 mismatch · cross `90812 / 90424`, 0 mismatch · debug lane 18/18, **0 panic** | <!-- TIPGATE --> |
| 878-TU workload scan | `match` **26** · `mismatch` **0** · `codegen-gap` **0** · `vocab-gap` **844** · `capture-fail` **8** | <!-- TIPSCAN --> |
| `gap-metric` keys, `^ *gap-metric \S+ \S+$` | **394** | <!-- TIPKEYS --> |
| `fnbyte-exact` / `differs` / `refused-parse` | 35,899 / 1,958 / 113,447 | <!-- TIPFNB --> |
| named control `C1`, pinned **BY NAME** | `C1a` **RED 1,669 / 2**, `the_call_argument_arity_fence_is_a_series_and_admits_exactly_one_symbol` · `the_two_symbol_thunk_exemption_turns_on_the_bare_body_marker_alone`, differential 83.71 s | <!-- TIPC1 --> |
| `git diff master..HEAD -- crates fixtures scripts` | — | <!-- TIPDIFF --> |
| **graded tree identical at both ends** | — | <!-- TIPTREE --> |
| `scripts/debug_lane.sh` | 18 lanes, 0 failed, **0 panic** | <!-- TIPDBG --> |
| `scripts/board_audit.sh` | — | <!-- TIPBOARD --> |
| `crates/c2-harness/tests/rung_registry.rs` | 2/2 | <!-- TIPREG --> |
| release-binary sha256 across worktrees | **NOT compared** — **#3224**: `CARGO_MANIFEST_DIR` is compiled in, so the comparison is void by construction | |

---

## 10. Found and not taken, ranked

### F1 — The mutation census over this class is now PRICED, and it is 507 runs, not 1,336

This is the deliverable the screen exists to produce and **nobody has run it.**
`w-mutcensus` costed the whole class at *"one suite run per site ≈ 5 days"*.
Reach is a **necessary condition** for a mutation to be killable, so the 829
quiet sites are retired before the first mutant:

| campaign | runs | at this lane's measured 205–256 s per suite |
|---|---:|---|
| the whole class, as `w-mutcensus` costed it | 1,336 | ~5 days serial |
| **the reached sites** | **507** | **~34 h serial, ~2 h across 16 worktrees** |
| **the reached sites in REFUSING position** (§4.1) — the only ones where a key-swap is observable at all | **321** | **~22 h serial** |

The screen that produces this is **four corpus runs**. That ratio — four runs to
retire 62 % of a campaign — is the reusable result, not the 507.

### F2 — `xtea_encrypt_loop.rs` is the one parser the corpus enters and abandons at gate 14 of 57

Every other of the 30 files is walked to **≥ 83 %** of its gate depth (§5.1);
this one stops at **25 %**, with 5 of 57 sites reached. Two readings and they
need different work: either the shape's own fixtures do not exercise it past its
prologue — a hole in the fixture corpus for a shape this repo shipped — or an
early gate admits nothing the corpus carries. **One reading and one suite run
settles it**, and it is the single most anomalous row this lane produced.
NOT TAKEN.

### F3 — The conditional secondary frame was NOT RUN, and its count is published so nobody re-derives it

Prereg §5 registered the TU-admission gates as a conditional second frame. It
was not run. Published with counts (§7): **44 `return None` clauses** across
`dyninit_tu` (11), `data_tu` (14) and `provide_data_tu` (19), plus **15 `?`
operators** in the same three functions, plus **19** shape-file `OptWordMode`
gates. The `return None` half is a `#[track_caller] fn none<T>() -> Option<T>`
helper and 44 mechanical edits — **one build and one corpus pass**, the same
shape as this lane's screen. The `OptWordMode` and `?` halves need a per-site
boolean wrapper and are a different instrument.

### F4 — 685 of the 1,060 distinct census keys in this class have NEVER been produced by anything this repo runs

Wholly-quiet `ctx` literals: **685**. Those are named buckets in the census's
own vocabulary — the thing `docs/GAPS.md` and every widening-order table are
written in — that no input in the suite, the sweep, the cross, the fixture
lanes, the debug lane or the 878-TU workload has ever caused to be reported.
**Whether that is a corpus gap or a vocabulary that outran its evidence is a
real question and this lane does not answer it.** It is a different question
from *"is this site guarded"* and it may be the more useful one: a census key
with no witness anywhere is a bucket nobody can check the spelling of.

### F5 — The 19,556-case generated sweep contributes ZERO exclusive reach over this class

133 sites reached, **0** of them reached by nothing else (§5). Stated with its
limit, because **#3270**–**#3275** is exactly about over-reading a zero: the
sweep grades **bytes** and this lane measured **reach**, which is a different
axis, and a zero on one is not evidence on the other. What it does support is
narrow and useful: **a lane trying to buy grammar-site reach should not buy it
from the sweep.** The fixture lanes (70 exclusive) and the 878-TU workload (17)
are where the unique reach is.

### F6 — `#[track_caller]` is a general site-level probe for this repo and this is its first use

Any family that funnels through a small number of constructors can be screened
at **one line per constructor**, with no per-site edit and no bitmask: one
`&'static Location` exists per call site and its address is the site key.
`refuse("<key>")` (23 sites), `Block::at_end(`, and the `IlOp` producers are all
such families. `w-deadsites`' bitmask was the right instrument for 34
hand-picked sites and does not scale; this does, and the two agree on method.
**The pattern is the transferable half, not this lane's numbers.**

### F7 — The reach ratio ages, and a standing version needs a `crates/` change a zero-delta lane cannot land

Same shape as `w-mutcensus` F4 / **#3233**, third instance. The difference here
is that the screen is **3 lines and 4 runs**, so a standing version is
*imaginable* in a way a 56-run mutation campaign never was — a gate row that
re-runs the screen and fails when the reached count moves would turn "a fact
about a commit" into a maintained invariant. It still needs the probe under
`crates/`, and a required-zero-byte-delta lane cannot land one. **That is now
four waves running in which the instrument a lane proved necessary could not be
shipped by the lane that proved it.**

### F8 — At least one of `w-deadsites`' seven UNGUARDED rows is in EAGER position, so its "a witness is cheap" pricing may not hold — flagged, not corrected

`#3276` and `2026-08-18-deadsites.md` §4.1 price all seven UNGUARDED rows
identically: *"the probe has already done the expensive half of writing [a
witness]: it proves an input exists."* §4.1 of THIS rung shows that claim needs
the site to be in diverging position.

Checked against that lane's own row text, at this base:

| row | site | position | so a hit means |
|---|---|---|---|
| `CA6`, `CA8` | `calls.rs` `return Err(refuse("call-arg-nonformal"))` / `("call-arg-computed")` | **diverging** | refused — pricing holds |
| `B2`, `B7` | `bind.rs` `resolve_data_def` / `resolve_bss_def`, `if !o.comdat \|\| !o.initialized { return None }` | **diverging** | refused — pricing holds |
| `CS3` | `census.rs` `"static-scan-loop" => STATIC_SCAN_LOOP_OBJECT` | match arm, produces the verdict | refused — pricing holds |
| **`CS4`** | `census.rs` **`bind_key.unwrap_or(STORE_RUN_BIND_NO_CARRIER)`** | **EAGER** | **evaluated** — and that lane's own table says the site fires *"with `bind_key` **`Some`**"*, i.e. **with the default arm NOT taken** |

**So `CS4` looks like a row that records evaluation and not refusal**, and if so
its witness is not the cheap one the bucket is priced at. **NOT CORRECTED
HERE**, deliberately and twice over: a dated rung stays as written (**#3117**),
and this is a reading of that lane's text rather than a re-run of its probe. It
is offered as a question for the next reader of **#3276**, and settling it is
one probe placement.

### F9 — This lane measured REACH and not test quality, on purpose, and the distinction is the whole point

Stated so the rung cannot be over-read, and it is the same discipline
`docs/rungs/README.md` grew for `#3270`–`#3275`: **507 sites being reachable is
not 507 sites being guarded, and 829 being quiet is not 829 being unguarded.**
No mutation was run on any of the 1,336. What the screen establishes is the
**necessary condition** — a mutation at an unreached site cannot be killed by
any test — and therefore a *price*, not a verdict. The verdict is F1.

