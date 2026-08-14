# w-read2 — PREREGISTRATION

    Lane:   w-read2
    Base:   master ac3cdd8c ("docs: regenerate STATUS after wave two — every
            scan metric held, only counts moved")
    Branch: wt-w-read2, worktree .claude/worktrees/w-read2
    Written BEFORE: the first line changed under `crates/`. `git status
            --porcelain crates/` is **0 lines** at the time of writing and
            `git diff ac3cdd8c -- crates/` is EMPTY.
    Written AFTER: the base 878-TU scan and the base `cargo test`. Both are
            *selection* measurements on an **unmodified binary** and neither
            touches `crates/`; §1 is the whole reason this lane's target is not
            the one the brief ranked first. Every number in §1 is reproducible
            from `work/w-read2/base.jsonl` with `work/w-read2/keys.py`.

Frozen once the first `crates/` byte changes. Corrections go in the rung doc as
scored misses, never by editing a line above the score table.

---

## 0. Base, measured in this worktree at `ac3cdd8c`

| | value |
|---|---:|
| `match` · `mismatch` · `codegen-gap` · `port-error` | **25 · 0 · 0 · 0** |
| `vocab-gap` · `capture-fail` · `frontier` | **845 · 8 · 2** |
| `fnbyte-exact` / `fnbyte-denominator` | **35,734 / 162,049** |
| **`fnbyte-refused-parse`** | **113,612** |
| `gap-metric` keys · jsonl rows | **372 · 878** |
| `emit_blockers` | **615 keys summing 113,612** |
| `fn_blockers` | **635 keys summing 1,705,627** |
| `cargo test --workspace --release --no-fail-fast` | **1,581 passed · 0 failed · 42 targets** |

---

## 1. WHICH REFUSAL I TAKE, AND WHY — the selector is #3107's own rule, applied to all six

Board **#3107** (`w-deaccept`) established that `w-readphase` §4.2's residue
ranking is **a ranking of a counterfactual population**: `expr-op-0x5D` and
`expr-op-0x5E`, published as *"the single largest residue item, 13,158"*, do not
exist as keys at base in either histogram, and the real widening built on that
ranking moved 0 of 372 keys.

**Nobody has applied that rule to the other five.** I did, before picking, on
this lane's own base scan. The dispatch brief's six targets, at base:

| refusal | w-readphase §4.2 ceiling | **base `emit_blockers`** | ratio |
|---|---:|---:|---:|
| `5D`/`5E` — shipped by `w-deaccept` | 13,158 | **0** | — (#3107) |
| `0x64` — by-value return materialize | 8,000 | **422** | **19.0× smaller** |
| **the statement layer** | 7,903 | **7,911** | **1.00× — IDENTICAL** |
| the compound-assign family | 5,269 | **45** | **117× smaller** |
| `0x9A` — vtable-slot bind | 2,674 | **0** | **key does not exist** |
| `0x00` — the 64-bit-literal desync | 2,276 | **0** | **key does not exist** |

And the brief's seventh item, `op:BD` (*"moves class-wide reach 14,479 →
40,530, the largest single rung in §3 by 3.5×"*): `expr-op-0xBD` at base is
**31**.

### 1.1 THE STATEMENT LAYER IS THE ONLY ONE WHOSE CEILING NUMBER IS ITS BASE NUMBER, AND THE REASON IS STRUCTURAL

`body-cflow-label` **2,832** · `body-0x9B` **2,213** ·
`return-scope-close-cflow-label` **1,814** · `body-0x67` **1,044** — and
`body-0x5D` **8**, which §4.2 did not count. Total **7,911**.

That agreement is not luck and it is the finding this pick is made on:

> **Every sink instrument in this tree lives inside `parse_expr`.**
> `chain_sink` (#660), `branch_sink` (#440), `rel_sink_enabled` (#420) and
> `off_add_sink_enabled` (#143) are all consulted from `parse_expr_classed`.
> A key raised **outside** `parse_expr` is therefore **invariant** under the
> entire ceiling measurement — so its ceiling reading *is* its base reading,
> and `w-readphase` §4's ceiling could never have moved it.

The four statement-layer keys are raised at `body/mod.rs`'s dispatcher `_` arm
(`blk(seg, p, "body")`, `disp-body-byte` = **91,802** bodies) and at
`expr.rs:241`'s `return-scope-close` in `eat_return_head` — both **before or
outside** any call into `parse_expr`. A body refused there never enters the
expression walk at all, so **it cannot reach the function tail no matter which
opcodes are sunk**: 7,911 of `w-readphase` §4.2's 44,409-function residue
(**17.8 %**) is structurally unreachable by the whole instrument family, and it
is the largest such block.

**Registered as a prediction, not asserted:** see B1/B2 below.

### 1.2 So what I take

**The statement layer**, and the deliverable is the instrument that can measure
it — `C2RS_SINK_STMT`, the first decode sink in this tree **outside**
`parse_expr` — plus any real, unpoisoned widening that measurement licenses.

I do **not** take `0x9A`, `0x00` or `5D`/`5E` (base population 0 — a widening
with no reach is #3107's own mistake repeated), nor `0x64` (422) or
compound-assign (45) on size, both of which are ranked by a counterfactual.

---

## 2. WHAT I WILL BUILD

**Arm I — the instrument (committed).** `C2RS_SINK_STMT`, env-gated, OFF by
default, in the body dispatcher. It consumes statement-position tokens by the
width `chain_skip_form` already pins, pushes **no `IlOp`**, and **poisons**: a
body that reaches the end having used one refuses anyway, under
`stmt-sink-poison`. Same four properties `chain_sink`'s doc states, and the
same reason. It is documented in the rung doc **and** in `docs/` — board
**#3098** is open precisely because the other four are not, and a fifth
undocumented instrument is a regression.

**Arm R — a real widening (CONTINGENT on Arm I's measurement, may not ship).**
If, and only if, the ladder shows a statement-layer clause whose successors are
already accepted, ship it unpoisoned with a stated fail-closed emission
boundary per `IL_STMT_GRAMMAR.md` §14.2. **Decoding is not licence to emit**:
`§14.2` step 5's boundary binds — a decoded label is not a lowered CFG, and any
body carrying one must still return `NotImplemented`.

---

## 3. THE REGISTERED PREDICTIONS

### 3.1 The three required-zeros — Arm I, sink OFF (the grade)

| id | registered | p |
|---|---|---:|
| **Z-a** | Δ`fnbyte-exact` = **0** (35,734) | 0.95 |
| **Z-b** | Δ`match` = **0** (25) and `mismatch` = **0** | 0.97 |
| **Z-c** | Δ`fnbyte-refused-parse` = **0** (113,612) | 0.95 |
| **Z-d** | **0 of 372** `gap-metric` keys and **0 of 878** verdict lines differ | 0.90 |

### 3.2 The phase column — stated with NO discount factor

**Arm I ships poisoned, so its registered Δ`fnbyte-refused-parse` is `0` by
construction.** Registering anything else would be dishonest. What the lane
owes instead is the **ceiling** — the counterfactual of the production being
widened, which per ROADMAP's own rule *is* the estimate with no discount:

| id | registered | p |
|---|---|---:|
| **C-a** | statement sink alone (no chain sink): **< 1,000** of the 7,911 reach the function tail | 0.75 |
| **C-b** | statement sink **+ the 49-token chain ceiling**: the whole-body ceiling rises **above** `w-deaccept`'s 88,806 of 120,456 | 0.80 |
| **C-c** | …and the rise is **> 7,911** — i.e. the statement layer also unblocks bodies whose *expression* successors were already granted | 0.45 |
| **C-d** | the corrected ceiling is still **< 120,456** — the statement layer is not the last thing | 0.93 |

### 3.3 The ladder — the thing the brief says I owe for any number I size

| id | registered | p |
|---|---|---:|
| **B1** | the four statement-layer keys are **invariant** under `C2RS_SINK_BRANCH` at all three levels (already measured true — recorded as a *stated* control, not scored) | — |
| **B2** | they are **invariant** under the full 49-token `C2RS_SINK_CHAIN` ceiling too | 0.90 |
| **D1** | the successor set of `body-cflow-label` under the statement sink has arity **≥ 2** — the ladder is not one rung | 0.85 |
| **D2** | the modal successor of `body-cflow-label` is an **`expr-*`** key, not another `body-*` key | 0.70 |
| **D3** | `return-scope-close-cflow-label` (1,814) and `body-cflow-label` (2,832) have **different** successors — they are two refusals, not one counted twice | 0.60 |
| **D4** | at least one of the four keys has a successor that is **already in the accepted vocabulary**, i.e. a real widening is licensed for it | 0.40 |

### 3.4 Arm R

| id | registered | p |
|---|---|---:|
| **R1** | Arm R ships at all | 0.35 |
| **R2** | if it ships, Δ`mismatch` = 0 | 0.98 |
| **R3** | if it ships, Δ`fnbyte-refused-parse` < 0 | 0.30 |
| **R4** | if it ships, Δ`fnbyte-exact` ≥ 0 and Δ`match` ≥ 0 | 0.85 |

### 3.5 Tests — **with the target count**

| id | registered | p |
|---|---|---:|
| **T-a** | base is **1,581 / 0 / 42** (measured above) | — |
| **T-b** | test-count delta **+3**, **target 1,584 passed / 0 failed / 42 targets** | 0.50 |
| **T-c** | target count unchanged at **42** | 0.95 |

### 3.6 The mutation control (`w-ir-e`/`w-ir-g`'s standard)

| id | registered | p |
|---|---|---:|
| **M-a** | a mutant that makes the statement sink consume a **wrong width** goes RED on a test whose oracle is the successor byte a real capture shows | 0.85 |
| **M-b** | a mutant that **removes the poison** goes RED — i.e. the poison is load-bearing and not decoration | 0.70 |

---

## 4. WHAT WOULD MAKE THIS LANE `FAILED`

Not "the numbers came out zero" — a measured zero on a registered ceiling is a
result. `FAILED` is: no instrument lands, or one lands that cannot be shown to
measure anything positive, or a required-zero breaks and ships anyway, or
`mismatch` is non-zero at any point.

## 5. WHAT I WILL NOT TOUCH

`crates/c2-il`'s `label_slots` / charge surface and `crates/c2-core/src/codegen/`
(peers `w-fenceb` and `w-item-d`). `is_statement_layer` (`expr.rs:1352`) is a
**shared predicate with a currently-zero population** flagged by `w-deaccept`'s
found-and-not-taken #1; I will **not** add `5D`/`5E` to it, and I will not
narrow, shadow or redefine it. If the work needs a change outside my files I
stop and report.
