# PREREG — lane `w-bind`, 2026-08-16

    Lane:     w-bind
    Branch:   wt-w-bind16   (NOT `wt-w-bind` — see §0)
    Base:     master 55933035
    Kind:     Characterization lane
    Frozen:   before the first `crates/` change in this worktree

**This file is frozen. Nothing below is edited after the first `crates/` change.**
Corrections land in the rung doc as corrections, never here.

---

## 0. A name collision, recorded before it can bite

`scripts/setup_worktree.sh w-bind` **fails**: branch `wt-w-bind` already exists,
created 2026-08-07 by the *previous* `w-bind` lane (`docs/rungs/2026-08-08-w-bind.md`,
tip `5250d81f`, long since an ancestor of master) and still checked out in a stale
worktree. This lane therefore runs on **`wt-w-bind16`**.

The rung-doc slug is unaffected and was checked separately: the existing file
declares slug **`w-bind`**, this lane claims slug **`bind`**. They are different
strings, so `rung_registry` sees no collision — but the *branch* namespace has no
such check, and the failure mode is a lane silently landing on top of a merged
branch. `w-json2` hit the doc-slug version of this last wave. Recorded because
neither `board_audit.sh`, `rung_registry` nor `gate.sh` can see it.

---

## 1. The question

Board **#3177** publishes the reachable widening order and names its head:

> the reachable head is call-argument / data-symbol **BINDING — 1,825 of 3,062,
> 59.6 % — not grammar at all.**

**1,825 = `call-arg-multi-sym:eof` 801 + `call-arg-multi-sym:mid` 495 +
`data-sym-unresolved:eof` 529.** #3177 names it as *one head*. This lane asks the
question the row does not: **is it one mechanism or two?**

Reproduced on this branch before freezing, from `work/w-bind/base.jsonl`
(the 878-TU workload scan at master `55933035`): **30 keys, 3,062 accounted**,
head `801 / 529 / 495 / 464`. #3177's table is exact.

---

## 2. Populations and denominators — registered, one per numerator

`match` has three meanings (**#3125**). Every number in this lane's rung doc names
its population. The four this lane uses:

| # | population | denominator | base value at `55933035` |
|---|---|---|---|
| **P-W** | 878-TU **workload scan** (`c2rs gap --list work/dc3-workload/files.txt`) | **878** TUs | `match` **25** · `mismatch` **0** |
| **P-E** | **emitted functions** in P-W | **162,049** | `fnbyte-refused-parse` **113,612** · `fnbyte-exact` **35,734** |
| **P-M** | the **modeled-reachable** sub-population of P-E — `emit-cflow-modeled-key\|*` | **3,062** functions, 30 keys | head 801/529/495/464 |
| **P-F** | the **fixture gate** — `gate.sh`, 381 fixtures × 18 mode lanes | **381**/lane | graded tree `75864f22df31`, 731 files |

Plus the portable lane: `cargo test --workspace --release --no-fail-fast` =
**1,643 passed / 42 targets** at base.

**P-M is the grading population for the characterization.** P-E's
`fnbyte-refused-parse` is the graded column for any *build*.

**A first-blocker count is not a distance** (#3062, #3095). #3095 measured
`decode_causes` under-reporting a ladder by up to **725×**, and its lesson is that
this is true of the **all-cause set too**. So every number below is produced by
**lifting the clause and re-scanning**, never by re-asking a diagnostic.

---

## 3. The two candidate mechanisms, located at their enforcing lines

Read the item's title and its enforcing line **separately** (four for four:
#3114, #3119, #3151, #3165). #3177's head is one phrase; it resolves to **two
different predicates in two different modules**:

| | key | enforcing line | predicate | what it asks |
|---|---|---|---|---|
| **M-COUNT** | `call-arg-multi-sym` (**1,296** = 801+495) | `crates/c2-il/src/func/body/shapes/calls.rs:431` | `syms > 1 && !two_sym_thunk` | **how many** symbol arguments |
| **M-NAME** | `data-sym-unresolved` (**529**) | `crates/c2-il/src/func/bind.rs:886` `Bindings::resolve_data` → `func/gl.rs` `NAME_SEPARATORS` excludes `25` | the token has **no `.gl` name at all** | **whether the symbol has a name** |

They are **in series, not in parallel**, and that is the whole reason the
question is open: the census's `sym_fail` probe (`crates/c2-il/src/func/census.rs:1211`)
runs **only on a body that already built a `MultiArgTailCall`** — i.e. one that
already passed M-COUNT. **M-COUNT fires before resolution is ever asked.** So
`data-sym-unresolved` = 529 is measured **only over bodies with exactly one
symbol argument**, and nothing in the corpus has ever asked what the 1,296
multi-symbol bodies would resolve to.

And the canonical multi-symbol body in `calls.rs`' own measured capture table
(§17.3, six captures) is **`void f(){ g("aa","bb"); }` — two string literals** —
which is precisely the population M-NAME refuses.

---

## 3.5 A contradiction found in the prior art, registered BEFORE the freeze

Found while reading, verified in both sources directly, and registered here
because it reframes the whole question. **Two measured records of the same
lowering disagree, and they disagree because they were captured at different
optimization profiles.**

| source | profile | `f(){ g("aa","bb"); }` emits |
|---|---|---|
| `docs/IL_CALL_IN_EXPR.md` §17.3 (a), lines 1769–1784 | **`/Ox /GS- /c`** (the fixture default) | `lis r11,0` · `addi r3,r11,0` · **`addi r4,r3,-4`** · `b` — **one pair per FUNCTION**, second symbol by **`.rdata` pool-offset difference** |
| `crates/c2-il/src/func/body/shapes/calls.rs:396–414`, six captures | **`/nologo /c /GR /O1 /Oi /EHsc`** — **the workload's own flags** | `lis r11,bb` · `lis r10,aa` · `addi r4,r11,bb` · `addi r3,r10,aa` · `b` — **one independent pair per SYMBOL, no pool difference anywhere** |

Both are real `c2.dll` captures. §17.2 **item 4** already contains the mechanism
that explains the split, one subsection above the place it is not applied:

> `/O1` implies `/GF` and `/Gy`, so string literals become `??_C@_…` **COMDAT**
> `.rdata` sections … **The two profiles need two different emitters**, and only
> the `/Ox` one is close to what the port has.

At `/Ox` every literal lives in **one ordinary `.rdata` section**, so pool-offset
differences exist and c2 exploits them. At `/O1` each literal is **its own COMDAT
section**, so **there is no pool to take a difference in** — hence the independent
pairs.

**The consequence, and it is why this is registered rather than left as a note:**
§17.3 is titled *"What is NOT established, and it is what stopped the rung"*, and
what stopped it is (b) — *"which symbol is the anchor is offset-dependent … a
HYPOTHESIS with no mechanism behind it"*, a **14-witness fit**. If the split above
is real, **(b) is a `/Ox`-only artifact**: at `/O1` there is no anchor to choose
because there is no pool to anchor in. A rung was declined on a property of a
profile **the graded workload is not compiled at**.

This is `#3160`'s mode-dependence and `#3165`'s title-versus-enforcing-line in one
object, and it is the **fifth** member of the reading-rule family (#3114, #3119,
#3151, #3165, and #3171/#3177 this wave).

| id | prediction | denominator | P |
|---|---|---|---|
| **P8** | The split is real and reproduces: at `/O1 /Oi /EHsc` a two-string-literal call emits **two independent `lis`/`addi` pairs and zero pool-difference `addi`**, and at `/Ox /GS- /c` the **same source** emits one pair plus a difference `addi` | 1 source, 2 profiles | **0.80** |
| **P9** | At `/O1` the emitted form is **compositional in n** — n symbol arguments give **n** independent `lis`/`addi` pairs, no anchor, for **n = 1, 2, 3** (the ≥ 3-cell SERIES §5 demands) | 3 cells minimum | **0.65** |
| **P10** | At `/O1` the `??_C@_…` COMDAT name for every literal is **already present in `.gl`**, so M-NAME is *not* "no name exists" but "a name deliberately not indexed" (§17.2 item 4's 237-against-237 count, re-derived on a workload TU) | 1 TU | **0.75** |

**P9 is the build gate.** If it holds at three distinct n it is a series, not a
cell, and §5's second condition is met. If it fails at any n, **the lane
declines** and says so in that word.

---

## 4. Hypothesis and predictions — probability form, denominator beside each

**H1 (the one this lane exists to settle).** M-COUNT and M-NAME are **one
mechanism seen twice in series**: M-COUNT masks M-NAME, and #3177's 1,825 is not
1,825 distinct problems but substantially one — `.rdata` string-literal binding —
with an arity fence standing in front of it.

**H0.** They are two independent mechanisms; the multi-symbol bodies resolve
fine and are blocked on something unrelated.

| id | prediction | denominator | P |
|---|---|---|---|
| **P1** | Lifting M-COUNT alone sends **> 50 %** of the 1,296 `call-arg-multi-sym` functions onto a **`data-sym-*`** key | 1,296 of P-M | **0.60** |
| **P1a** | The `:eof` half (801) goes to `data-sym-*` at a **higher rate** than the `:mid` half (495) — `:mid` means grammar remains, so it has somewhere else to go | 801 vs 495 | **0.75** |
| **P2** | Lifting M-COUNT alone moves `fnbyte-exact` (P-E) by **0** | 35,734 | **0.85** |
| **P3** | Lifting M-COUNT alone moves `match` (P-W) by **0** | 25 of 878 | **0.90** |
| **P4** | Lifting M-COUNT alone keeps `mismatch` (P-W) at **0**, because `c2-core`'s `sym_slots_text` carries an *independent* `count != 1` backstop (`crates/c2-core/src/codegen/calls.rs:1815`) — so an admitted body reaches the port and returns `NotImplemented` | 0 of 878 | **0.90** |
| **P5** | Lifting M-COUNT alone makes `fnbyte-refused-parse` **fall** (the graded column) | from 113,612 | **0.70** |
| **P6** | …and that fall is matched **one-for-one by a rise in `codegen-gap`**, i.e. it buys reclassification and not bytes — the `w-readphase` §6 shape | — | **0.65** |
| **P7** | Ladder depth from M-COUNT is **≥ 2** rungs before anything grammar-shaped appears | — | **0.70** |

**Ceiling, NO discount factor.** If every one of the 1,825 were converted to an
emitted byte-exact function, the ceiling on P-E is **1,825 / 162,049 = 1.13 %**
of emitted functions, and **1,825 / 113,612 = 1.61 %** of `fnbyte-refused-parse`.
On P-W (TU `match`) the ceiling is **stated as unknown and NOT claimed**: no
reader rung in this repo has ever converted a TU, and `w-readphase` measured
lifting the whole `.gl` walk at `match` **+0** / `fnbyte-exact` **−65**. **This
lane does not grade itself by TU conversion.**

---

## 5. Build gate — what would license shipping anything

Per the standing correction `w-slots` paid for (**#3147**): *reading one cell
gives a number right for that cell and wrong as a rule* — its fixture's obj read
**3**, the series was **2n+1**, and shipping the 3 would have been a wrong obj.

**This lane ships code only if BOTH hold:**

1. a **closed recognizer** — a decidable pre-emission predicate, no cost model,
   no heuristic; **and**
2. a **SERIES**, not a cell: the emitted form as a function of n (number of
   symbol arguments), read off **≥ 3 distinct n** from real `c2.dll` output.

**If either is missing the lane DECLINES and prices the decline.** A decline is
graded by a probe whose **positive control is demonstrated to FIRE** before the
null is read — three lanes did this last wave and it is the standard now (#3149,
#3160). A decline whose probe is not demonstrated capable of detecting a change
is absence read as success.

**Fail-closed boundary, stated up front:** decoding is not licence to emit. Every
ladder rung below is a **reader** lift; the port's `sym_slots_text` backstop stays
untouched, so a widened reader produces `codegen-gap`, never a wrong relocation.
If any rung takes `mismatch` off 0, **that rung is reverted immediately and the
non-zero is reported as the fence's measured price** — it is not landed under any
circumstances.

---

## 6. The ladder — three compiles, registered before the first one

Each rung = edit `crates/c2-il` only, `cargo build --release -p c2-harness`,
re-run the **full 878-TU scan**, diff `emit-cflow-modeled-key|*` against base.
**Single writer, foreground, no `crates/` patch while a scan is in flight**
(#3075, #3117, #3128 — `w-stmt5` violated this against its own prereg last wave
and had to discard the run).

* **L0** — base. Done, committed: `work/w-bind/base.jsonl`, `base-gap.log`.
* **L1** — lift **M-COUNT**: `calls.rs:431`, `syms > 1 && !two_sym_thunk` → admit.
  Answers P1/P1a and therefore **H1 vs H0**.
* **L2** — L1 + lift **M-NAME** far enough to name a `25`-separated token.
  `gl.rs` states the global risk explicitly (*"admitting a fourth separator there
  would re-bind tokens globally"*), so L2 is **scratch-only by construction** and
  is watched for `mismatch` at the top of the run.
* **L3** — whatever L2 reports next, if L2 is safe to run.

Every rung is reverted. `git status` clean-to-base is checked at the end and the
revert is a commit, not a discard.

---

## 7. Mutants — expected colour registered BEFORE any run

`w-stmt5` registered six and **two came back green and were RIGHT**: two clauses
had shipped with no test that could fail. That is the point of registering.

| id | mutation | expected |
|---|---|---|
| **M1** | `calls.rs:431` `syms > 1` → `syms > 2` (weaken M-COUNT) | **RED** |
| **M2** | `c2-core/src/codegen/calls.rs:1815` `count != 1` → `count > 2` (weaken the fail-closed backstop P4 leans on) | **RED** |
| **M3** | `bind.rs:886` `resolve_data` — drop the `extern_data.contains(&name)` linkage gate | **RED** |
| **M4** | `census.rs:1211` `sym_fail` — swap `DATA_SYM_UNRESOLVED` and `DATA_SYM_LINKAGE` | **RED** |

A **GREEN** is not a pass. It is a finding: the clause shipped with nothing that
could fail on it, and it is reported in those words.

**Discriminating-cell counts are printed for every check.** A check that ran on
zero cells fails loudly rather than reading as a pass.

---

## 8. What this lane will NOT do

* **Not** re-open codegen. `codegen-gap` is **0** over all 878, the frontier is
  **2** and both declined, and item F buys **zero** in all four populations
  (#3170). §6.2 is not a route to the goal.
* **Not** dispatch off a ranking. `w-loo` measured five of six published rankings
  at **ρ ≈ +0.047 — noise** (#3135). #3177's reachable order is used here as the
  *location of a question*, never as a work queue.
* **Not** touch `crates/c2-harness/src/gap/` (peer `w-vocabgap`), `coff/`
  (off-limits), or docs owned by `w-871`.
* **Not** grade itself by TU conversion (§4).
