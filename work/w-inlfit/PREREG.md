# PREREG — lane `w-inlfit`, the inliner's two `fitted` clauses

**Registered 2026-08-27, BEFORE any byte of `c2.dll` was disassembled by this
lane.** Written against the committed artifacts only: `work/w-inlmetric/CLAUSES.tsv`,
`work/w-inlmetric/check_table.py`, `docs/whitebox/ref/P_INLINE.md`,
`crates/c2-core/src/splice.rs`, `crates/c2-core/src/comdat.rs`. Base tree
`42f76b849`; the table was frozen on `f8693d6e4`.

Lane kind: **characterization** (`docs/rungs/README.md` § "Lane kinds", kind 3).
`Fixtures: none`. `Census: +0`. **Predicted reach: 0.**

---

## 1. The denominator, verified here rather than taken

Re-counted on this tree, two ways — `check_table.py` and a bare `awk` over
column 5 — and they agree:

```
rows: 24
  state    : absent 17 · fitted 2 · R-derived 2 · unexercisable 3
  exercised: yes 9 · not-separable 6 · no 6 · unexercisable 3
CONFORMANCE-CHECK: GREEN  (0 failure(s) over 24 rows)
```

**The reachable denominator is 21** — the three `unexercisable` rows (C21, C22,
C23) are reached by no compilation this project runs. This lane does not restate
24 as a reachable count and proposes **no new rows**, which is `#3505`'s class
and `P_INLINE.md` §6.5's own closing fence.

## 2. The two clauses, named

| id | clause | c2 addr | the port's counterpart | why `fitted` |
|---|---|---|---|---|
| **C8** | candidacy size test: `cmp WORD [sym+0x50], DAT_10c46318`; `jl` = candidate | `0x10b5fc8a` in `FUN_10b5fb5f` | `splice.rs:INLINE_UNBOUNDED_BYTES = 64`, and its two relatives `comdat.rs:INLINE_DECLINE_BYTES = 128` / `INLINE_DECLINE_LOOP_BYTES = 80` | all three are **lowered PPC byte counts fitted to an obj bracket**; c2's operand is a pre-codegen instruction count |
| **C20** | the expansion recurses back into the driver for the inlined body | `0x10b620fc` in `FUN_10b620fc` | `splice.rs:S6-chain`, the fixpoint walk in `splice_body_why` | fitted to `#1020` — `t11` plus **150 workload relocation witnesses**, a black-box measurement |

## 3. The predictions, and what refutes each

### C8 — PREDICTED: **the read does NOT determine it.**

The port's constant will still be a fit when this lane ends. Registered reasons,
in advance:

1. **The units differ and the converter is outside the band.** c2 tests
   `WORD [sym+0x50]`, which `P_INLINE.md` §2.1a locates as the `.gl` `SIZE`
   field and §2.1b measures as an *upper bound* reduced by whatever runs before
   the inliner. The port tests bytes of an already-lowered PPC body. Nothing in
   the inliner band converts one into the other.
2. **The one arithmetic bridge is already refuted.** §5 records that
   `16 << k` with the image's `k = 3` gives **128 instructions**, and that this
   "does not compose into the measured numbers".

**Registered numeric prediction:** the read's ceiling and the port's constant
will be apart by **more than 2×** under every unit assumption I can state in
advance — 128 instructions vs. 64 bytes = 16 PPC words is 8× at 1 word per
instruction, and the measured `/O1` obj bracket `(100,116]` is 25–29 words,
still 4.4–5×.

**What refutes me:** any of —

* a read that shows `DAT_10c46318` is *not* `16 << DAT_10c2ea98` on the workload
  path (e.g. a second writer, or an option handler that stores a different
  value at `/O1`), and the value it does hold composes into 64/80/128;
* a read that finds a **byte-unit** or post-lowering size test in the candidacy
  chain, so the port's unit is c2's unit after all;
* a read that locates the reduction of `[sym+0x50]` between the `.gl` decode at
  `0x10b9bf6c` and the test at `0x10b5fc8a`, closing §2.1b's gap.

If any fires, C8 becomes adoptable and this lane owes a `DISCLOSURE` row, a
`PROV[R]` marker and a **two-sided price** — because a changed inline predicate
is an emit change, not a comment.

### C20 — PREDICTED: **the read DETERMINES c2's side and does NOT reach the port's.**

Split deliberately, because the row's two halves can land differently:

* **(a) c2's side — PREDICTED CONFIRMED.** `FUN_10b620fc` contains a call edge
  reaching `FUN_10b61ee1` (the driver), so the expansion re-enters the whole
  decision for the inlined body. `P_INLINE.md` §1 already asserts this `[R]`
  with **1 cite**; this lane verifies it in the image rather than quoting it.
* **(b) the port's side — PREDICTED NOT REACHED.** `splice.rs`'s S6-chain is a
  walk to the chain's **end** that asks its size clause **once**, at the end
  ("one check covers every link"). c2's recursion re-enters the driver with
  `level + 1` and a **decremented budget**, so c2 re-decides at each level under
  C14 (depth), C17 (budget) and C8 (size) — three tests the port has no
  counterpart for (C14, C17 are `absent`; C8 is the other fitted row). I predict
  the read shows the two rules are **structurally different and agree on the
  workload only because the port's admitted chains are a subset where c2's
  re-decision always accepts**.

**What refutes me:** no call edge from `0x10b620fc` into `0x10b61ee1` in the
image (which would make §1's row wrong and is the more interesting outcome); or
the recursion carrying the *same* level and budget, which would make c2's rule a
plain walk-to-end and the port's counterpart genuinely derived from it.

### 3.1 The state changes this lane is willing to make, registered in advance

Fixing these now so no state is chosen after seeing the answer:

| outcome | C8 state after | C20 state after |
|---|---|---|
| predictions hold | **stays `fitted`** | **stays `fitted`** — (a) alone does not reach the port's counterpart, and ties break toward the weaker state (`w-inlmetric` PREREG §5) |
| C8 refuted | `R-derived`, with a `DISCLOSURE` row, a `PROV[R]` marker and a two-sided price in the same commit | — |
| C20 (b) refuted | — | `R-derived`, with an address-cited comment in `splice.rs` and a `DISCLOSURE` row (C13's pattern) |
| C20 (a) refuted | — | still `fitted`; `P_INLINE.md` §1's row is corrected beside, never rewritten |

**No row is added, removed or renumbered under any outcome.**

## 4. The traps this lane has been handed, and the control for each

| trap | control |
|---|---|
| an inline-predicate change is an **emit change** (`INLINE-P` frozen at 0.9678 @ n=8,936, a single threshold on this workload) | predicted outcome writes **zero `crates/` logic**; any adoption gets the full gate + a two-sided count before it ships |
| `#3641` — prose *about* mark letters moves `subsys.rs::count_marks` | baseline captured **before** any edit: `[inline] agreement: marks [O] 11 of 40 (27.5 %) — [R] 29 [I] 0`. Re-rendered after, and the delta reported in the rung with its cause |
| `#3505` — constructing a denominator | §1: 21, verified two ways, no new rows |
| `#3691` — a 22nd `gate.sh` row breaks `gate_identity_diff.sh` for every live lane | this lane adds **no** count-bearing gate row |
| `#3336` — an unwatched control is decoration | `check_table.py` is re-run **and watched failing** on a planted verdict on this tree before its green is quoted |

## 5. What would make this lane FAILED

Producing neither (a) a read-derived replacement for a fitted constant nor
(b) a precise, address-cited statement of what the read does not reach. A lane
that reports "still fitted" **with the reason located in the image** has produced
its deliverable; a lane that reports "still fitted" because it did not look has
not, and says `FAILED` in that word.
