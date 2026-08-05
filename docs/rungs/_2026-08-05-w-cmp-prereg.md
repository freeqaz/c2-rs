# w-cmp — PREREGISTRATION

    Tag:       w-cmp
    Slug:      w-cmp-prereg
    Date:      2026-08-05
    Fixtures:  none — this is a prereg, not a rung. It admits no shape, moves no
               accept/refuse boundary and emits no obj byte. The rung it
               pre-registers is `_2026-08-05-w-cmp.md`.
    Census:    706555/2463393 (28.68%) at registration — unchanged, +0.
    Record:    this file. Committed BEFORE any `crates/` edit on this branch.
    Lane:      w-cmp, worktree `wt-w-cmp` off master `b78249e`.
    Ships:     nothing under `crates/`.

---

## 0. The brief, and the premise I am registering doubt about

I am briefed to build **`expr-cmp-eq`**, then **`expr-cmp-ne`** — the honest head
of `work/w-dclass/rerank.py`'s greedy re-ranking of the FRONTIER. Re-run at the
lane's start against `work/w-dclass/scan-final.jsonl`, that instrument reproduces:

```
  expr-cmp-eq              converts 3   appears in 7
  assign-store-type-8643   converts 2   appears in 3
  expr-cmp-ne              converts 1   appears in 2  (2 once expr-cmp-eq closes)
```

with `expr-cmp-eq` alone converting `IPP_basicmath_xbox.cpp`, `vswprnc.cpp` and
`mmio.cpp`. That arithmetic is correct and I am not disputing it.

**What I am registering doubt about is the word "closing".** `rerank.py`
computes: *which FRONTIER TUs have every blocker key inside a hypothetically
closed set*. A **blocker key is a census label on a parse refusal**. "Closing"
it, in the sense the ladder needs, means the blocked functions become
**in class AND byte-exact under real c2** — not that the parser stops refusing
at that byte.

Those are different events, and the project has measured the gap between them
six times (board **#150**). The sixth is `w-dclass` §6.1, a two-scan
counterfactual one env var apart:

| | sink OFF | sink ON |
|---|---:|---:|
| `expr-op-0x27` emitted blocked | 23,090 (17.6 %) — **rank 1** | **0 — key absent** |
| emitted census | 38,458 | **38,464** |
| **TU match** | **8** | **8** |

Unblocking the **#1** blocker key end-to-end was worth **six functions and zero
TUs**: 23,084 of the 23,090 were **renamed, not converted**. `expr-op-0x27` is a
**fall-through key** — its mass is the mass of everything falling through to it.

My brief demands, correctly, that any rung sized off a blocker-key census owe an
argument for why it is not this case. **I do not have that argument yet, and
this prereg registers that I expect the measurement to go the other way.** The
argument is the lane's first deliverable and its cheapest one, and it is a
*measurement*, not a reading: admit `cmp-eq` in the expression parser with no
codegen behind it, rescan, and see whether the three TUs' blocked functions
become in-class or merely acquire a new key name.

### 0.1 Why `expr-cmp-eq` is not obviously immune

`expr-cmp-eq` is **not** sized off census mass — it is sized off the FRONTIER
*conjunction*, which is a stronger key than the one that mispriced `0x27`. That
is the case for it. Against it:

* The FRONTIER conjunction is computed over **census keys**, and a census key
  moving is exactly what a fall-through is. If `mmio`'s blocked function refuses
  at `cmp-eq` and, once `cmp-eq` parses, refuses at the *next* token, `mmio`
  acquires a new key and leaves the convertible set. The rerank cannot see this
  in advance — by construction, since the successor key does not exist yet.
* w-dclass §4 priced these three at **5 / 6 / 9** IL+HARD+SOFT facts, and §4.3
  labels that price a **lower bound low in a known direction** (it cannot see a
  selection, allocation or scheduling decision). Subagent C then found the
  reprice's cheapest genuinely-buildable TU at **4** costs **10** by an
  independent cut. **Board #269's clause (≥4 independent refusals ⇒ not a
  target) fires on all three of mine at the published price already.**
* The compare *leaf* that exists (`shapes/leaf_compare.rs`) is `return <the
  function's single formal> <rel> <literal k>;` and nothing else — one formal,
  int/unsigned only, literal RHS, must reach segment end. Every FRONTIER
  `cmp-eq` site is by construction outside it, or it would not be blocked.

## 1. Predictions — registered before any measurement of my own

Intervals inclusive. A prediction with no interval scores hit/miss on the
proposition. **The wrong ones stay on the page.**

| # | prediction | interval |
|---|---|---|
| **R1** | Baseline reproduces master's block exactly: match 8, mismatch 0, vocab-gap 863, capture-fail 7, A/B/C/D/E = 28/338/169/8/2, `A∧B∧C` 27, FRONTIER 19; `cargo test --workspace --release` **809 passed / 0 failed / 27 targets** | exact |
| **R2** | **`expr-cmp-eq` is a FALL-THROUGH KEY.** Admitting the `==` operand production in `parse_expr` with no codegen behind it moves each of the three TUs' blocked functions to a **new** blocker key rather than into class. Number of the three TUs that become convertible under `rerank.py` after the widening | [0, 1] |
| **R3** | Independent per-function fact-count for the three TUs' blocked functions exceeds w-dclass's published 5 / 6 / 9 on **at least one** of the three | — |
| **R4** | **TU match at the end of this lane** | [8, 9] |
| **R5** | **TUs converted by this lane** | [0, 1] |
| **R6** | Board **#269**'s clause (≥4 independent refusals ⇒ not a target) **fires on all three** of `mmio`, `vswprnc`, `IPP_basicmath_xbox` | — |
| **R7** | At least one widening I draft is **refuted by my own constructed counterexample** before it ships — the `!=`→`>` trap shape, on the operator seam I have been warned it is waiting on | — |
| **R8** | `expr-cmp-eq`'s **census mass** (all-blocked functions carrying the key) is **large** — ≥10,000 — while its **conversion** worth is 3 TUs. The gap between the two is the same gap `0x27` had, and I register it as *predicted before measured* | ≥10,000 |
| **R9** | The three TUs' `cmp-eq` sites are **not one shape family**: the three blocked functions differ in at least two of {operand kind, consumer of the bool, containing production} | — |
| **R10** | Any census gain this lane produces is a **driver, not the result**. The result is TU match | — |

## 2. Declared bias

**I am the lane assigned to build `expr-cmp-eq`.** My incentive is to find it
buildable, and every reading of "this is nearly in class" I produce should be
discounted accordingly. Two specific guards:

* **Presence read as coverage** (w-dclass §3.2). An `encode_*` existing is not a
  capability; I will name the call site a mechanism is reachable *from*, or not
  claim it.
* **Absence read as success** (16 recorded instances). Every check I run prints
  a **count**, never a status, and I do not grade a run I did not see finish.

## 3. Decline clauses, registered in advance

* **D1 — fall-through.** If admitting `cmp-eq` in the parser moves the three
  TUs' blockers to new keys and converts zero functions, the lane's result is
  that measurement and the build is declined. The measurement is the deliverable.
* **D2 — #269.** If a target TU's blocked functions carry ≥4 independent
  unmodeled facts on an independent count, decline and report the count.
* **D3 — my own counterexample.** If a widening I draft is refuted by a
  constructed counterexample against real c2, it does not ship, and the
  counterexample is recorded whether or not the widening survives.
* **D4 — no fitted rules.** A rule fitted to a partial witness (w-dclass's F4a)
  does not ship even at 6/6.

## 4. The one-shot gate

**Declined**, for w-dclass's reason, adopted verbatim: codegen output is graded
byte-exact by real c2 through the ordinary differential, and a held-out
quarantine guards against overfitting a *fitted model*, which is factor A's
problem. I am codegen.
