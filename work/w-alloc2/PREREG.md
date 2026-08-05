# w-alloc2 — PREREG

Lane `w-alloc2`, worktree `agent-a7204fc9708972c2c` off master **`f128b21`**.
Board range **#836–#845**.

**Committed before any grid script in this lane directory exists.** The only
files beside this one at commit time are `scan.sh` (a locator wrapper around
`c2rs gap`, read-only w.r.t. `crates/`) and `baseline_scan.txt`, whose digits R1
scores. No probe of anything in §2 has been written, let alone run.

---

## 0. What I inherit, and the one code fact I read before predicting

w-next (`docs/rungs/2026-08-05-w-next.md` §5.1.2) measured a single priority key
over the mixed-kind store run:

> rank by **`uses + (1 if register-derived else 0)`, descending**
> — 24 cells, **0 misses**; clause 1 alone misses 4, clause 2 alone misses 4.

Its fitted domain is narrow and w-next says so: **exactly two producers** (one
`addi`-form register-derived, one `li` constant), use counts `1..4`, **leaf**
bodies, constants **first in source**, one consumption pattern.

### 0.1 The code fact, read before any prediction below

`crates/c2-core/src/codegen/leaf/store.rs:250-257` builds every `alloc::Producer`
with `kind: alloc::ProducerKind::Constant`, hard-coded, with the comment *"Every
producer that reaches here is a literal"*. `parse_simple_gpr_run` (same file, 94)
admits a store's value as either `IlOp::Lit` (a constant producer) or
`IlOp::Load` of a formal (**not a producer at all** — already live in a
register). There is no third arm.

**So no `RegisterDerived` producer can reach `alloc::allocate` from
`PortC2::build` today, and therefore `alloc`'s mixed-kind refusal is unreachable
from the emitter.** This is stated here, before the fact is used, because it
determines what "ship the key" can and cannot mean and it is the single most
misreadable thing this lane could land. R6 registers its verification.

---

## 1. The fresh holdout — designed here, not after the run

**The 24 cells w-next scored are NOT re-used to score anything.** They are
replayed only as an anchor control (F0). Every cell below varies an axis w-next
held fixed.

| axis | cells | what it varies | key's prediction |
|---|---:|---|---|
| **F0** anchor | 2 | nothing — `anchor`, `g1x1` | `addi`→r11, `li`→r10 |
| **F1** use counts **beyond 4** | 12 | `(reg,const)` with a coordinate ≥ 5 | reg wins r11 iff `reg+1 ≥ const` |
| **F2** **source order swapped** | 8 | reg stores emitted FIRST in source, at and around the key tie | winner unchanged from the const-first cell |
| **F3** **two-word constant** (#644) | 6 | `li 7` → `lis`/`ori` value (65537, 123456) | **out of regime**, not a miss |
| **F4** other register-derived ops | 12 | `rlwinm` (`u<<3`), `add` (`u+v`), `addi` off a formal (`u+5`) — not `addi &q` | same threshold |
| **F5** **three producers, mixed** | 16 | 2 reg + 1 const and 1 reg + 2 const | a full r11/r10/r9 ranking |
| **F6** pool floor | 4 | extra int formals push `pool_floor` up | ranking unchanged |

Target ≥ 50 fresh cells. Reached and graded are **separate printed counters**;
an ungraded cell is never a pass (STATUS trap 5).

### 1.1 F2 is the discriminating axis and it is registered as such

In every one of w-next's 24 cells the **constant stores come first in source**.
The key's `≥` — the clause that makes a key *tie* (`reg+1 == const`) go to the
register-derived producer — is therefore confounded with *"the later producer in
source wins the tie"*, which is clause 4's sign. F2 emits the reg stores first at
`(1,2)`, `(2,3)`, `(3,4)`, `(4,5)`. **If the winner flips, the bonus is not a
kind bonus and the key is refuted on 4 fresh cells.**

---

## 2. Registered predictions

| # | claim | how it loses |
|---|---|---|
| **R1** | the baseline reproduces w-next's digits exactly — match **10**, mismatch 0, codegen-gap 0, vocab-gap **861**, capture-fail 7, FRONTIER **17**, `fnbyte-differs` **0**, A 28 (LO 27)/B 338/C 169/D 10/E 2, `B∧C` 151, `A∧B∧C` 27 | any digit differs |
| **R2** | **THE KEY DOES NOT SURVIVE.** It misses on at least one fresh cell, and I predict the miss lands in **F5 (three producers)** | F5 comes back 16/16 HIT |
| **R2a** | the reason: `alloc.rs` records a preregistered exhaustive search over **52,416 priority-function allocators** topping out at **179/236**, *"with its residual **exactly** the tie tier"*. The pure-run rule is **provably not a priority function**. A two-producer mixed cell grades only *"who gets r11"* and cannot expose a tie tier at all; **three** producers can. So a priority key fitted only on 2-producer cells is expected to break exactly where the tie tier appears | F5 misses for some other reason, or does not miss |
| **R3** | **F2 does NOT flip** — the bonus is a kind bonus, not source order | any F2 cell flips (which refutes the key outright, on 4 fresh cells) |
| **R4** | **F3 is OUT OF REGIME, not a miss** — a two-word constant beside another producer has its `lis`/`ori` **split** (#644), so the cell has no single "the constant's register" to grade | F3 grades cleanly as a normal cell (which would refute #644's reach into the mixed run) |
| **R5** | **`xboxheap.cpp` does NOT convert**, and re-priced with the facts the obj states as answers it costs **19** mechanisms — w-next's 14 plus **5**: (i) the store-run producer allocation, (ii) the callee-saved choice `r31` for the live-across-call `this`, (iii) the register-derived producer *kind* on the IL side (§0.1), (iv) the framed store run with a call, (v) the `mr 3,31` return-`this` epilogue. A true count ≥ 15 refutes nothing below 19 | the TU converts, or the count comes out < 15 |
| **R6** | **shipping the key changes ZERO emitted bytes.** Verified by a run, not by §0.1's argument: gate, sweep, mode cross and the 878-TU scan must be **byte-identical** at both ends — **and mutation M0, a deliberately absurd mixed key, must leave them byte-identical too** | any number moves (which would mean the mixed path IS live and §0.1 is wrong) |
| **R7** | the warranty does not move: `fnbyte-differs` **0** at both ends, TU match **10** at both ends, and no TU that matches today un-matches | either moves |

### 2.1 R2 is the row I want to lose

R2 predicts my own deliverable fails. If F5 comes back clean the key is stronger
than w-next left it and it ships wider. If R2 hits, **the miss is the
deliverable** and the key does not ship past the boundary the miss draws — the
brief's standing instruction, and P3's precedent (`floor((N−1)/2)` fit three
published cells exactly and died at N = 5).

### 2.2 M0 is a control against this lane's own gate

If the key ships and the gate is green, that is **not** evidence the key is
right — §0.1 says the gate cannot see it. M0 makes that concrete by shipping a
key that is *wrong on purpose* and showing the gate is **equally** green. A lane
that quoted "18/18 PASS" as support for the key would be making STATUS trap 4's
mistake (a control that only checks a total is not a control).

---

## 3. Mutations owed

The key is inert on the emit path (§0.1, R6), so a mutation of it **cannot**
produce wrong bytes through the differential. The mutations are therefore graded
against **real `c2.dll` bytes replayed as fixture data**: every fresh cell's
measured allocation is pinned into a unit test, so a mutated key turns those
red. Each must-fail mutation is **RUN**, and its failing assertion is quoted.

| M | mutation | must |
|---|---|---|
| M0 | mixed key inverted (`uses − bonus`) | unit tests RED, **gate/sweep/scan byte-identical** (the inertness control) |
| M1 | bonus 1 → 0 (clause 1 alone) | RED — w-next measured 4 misses |
| M2 | bonus 1 → 2 | RED on a cell at the threshold |
| M3 | `≥` → `>` at the key tie | RED on the F2/tie cells |
| M4 | mixed refusal restored | the new tests RED (they must exercise the new domain) |
| M5 | pool walk ascending for mixed runs | RED |
| M6 | drop the 3-producer boundary | RED if F5 misses; recorded as such if not |
