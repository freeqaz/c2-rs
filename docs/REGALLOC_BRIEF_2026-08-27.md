# REGALLOC BRIEF — 2026-08-27

**Status: a proposal, not a decision.** Written by the coordinator after the
owner named the register allocator and the inliner as the two most valuable
subsystems (decision 18, answer #2). Nothing here is funded until the owner
says so, and no board numbers are spent by this document beyond its own row.

Every number below is quoted from a run on tree `f8693d6e4`, not from a page.
Where a figure comes from a page it says which page and which line.

---

## 1. The finding that should decide the shape of the work

**The register allocator is the best-read subsystem in the project with the
worst-ported one, and those two facts have a single cause that is not
"nobody got to it".**

Read side, from `c2rs subsys` on this tree:

```
[regalloc] register allocator — P_REGALLOC.md
  1 read      : sites 70 ⊇ read 33 (18 code + 15 data) ⊇ ported RESIDUE
  2 agreement : marks [O] 7 of 49 (14.3 %) — [R] 41 [I] 1
  3 exercised : RESIDUE — nothing traces c2.dll's own addresses over the workload
  4 byte-owned: CITED #3534
```

47 % of the band is read, against a project median nearer 20 %, and the read
is not shallow. `P_REGALLOC.md` carries:

* **The algorithm identified** — priority colouring (Chow–Hennessy), *not*
  Chaitin: no interference graph, no simplify/select stack (`§ The
  identification`). The page notes this is *why* the repo's earlier black-box
  attempts kept reaching for a traversal rule and kept missing.
* **The selector's decision rule, read and obj-confirmed** — minimum cost among
  allowed registers, cost = interference and constraint penalties minus copy
  preferences, ties to the earliest register in the fixed order
  `r11, r10, …, r3, r31, r30, …, r14`. `[O]` on 6 frozen cells, with **three
  rival rules refuted by cell count** (`§3`).
* **The worklist comparator, read with its order `[O]` on 20 cells at two
  profiles** — primary key `cand+0x0c` DESC signed, tie `cand+0x44` DESC
  unsigned, and the tie tier compares `<=` so an exact tie in both keys puts
  the **newly inserted** candidate first (`§4`).
* **A spilled candidate re-enters by priority, not at the head** — so a port
  modelling the worklist as a stack or a queue is wrong in both directions
  (`§4`, consequence 2).

Port side, measured on this tree rather than asserted:

```
grep -rin "register allocat|regalloc|colou?ring|interference|live range"
     crates/c2-core/src crates/c2-il/src --include=*.rs
  -> 34 hits, ALL of them in comments; 0 in code
```

The scoreboard's residue text is exact: *"the port has no register allocator of
this shape at all — the byte-exact classes are one-function bodies whose
registers are assigned by `codegen::select_function`'s own rules… A site-level
numerator is not merely unmeasured, it is not yet defined."*

**So the gap is not comprehension. It is that a well-read algorithm has never
been written down as code.** That is a different kind of work from every lane
this project has run recently, and it should be priced as such.

---

## 2. The blocker, stated before the plan rather than discovered inside it

`P_REGALLOC.md` §7 names it in its own words, and it is load-bearing:

> a candidate is a **(symbol, live-range version)** pair, and the versions need
> the backward walk over the **lowered** tuple list.

and

> **F5 is not separable from F0.** F5's input is `cand+0x0c`, accumulated over
> the code **the scheduler produced**, and F0 — priced at 8 — is what produces
> it.

Cross-check against the `dag` row on the same scoreboard: *"the port schedules
nothing — emission order is tuple-list."*

**Therefore a complete allocator is gated behind the scheduler, and any brief
that proposes one without saying so is mispriced.** `READ_PLAN` R7 re-prices F0
from 8 to 4 raw, which changes the number and not the dependency.

This is the single most important sentence in this document: **do not fund a
register allocator.** Fund the parts of it that are provably downstream of
nothing.

---

## 3. What IS separable, and why each piece is

The allocator decomposes into a part that *computes* priorities (needs the
scheduler) and a part that *consumes* them (does not). The second part is where
the reading is strongest and where every obj-confirmed cell already lives.

### L1 — the selector, as a settable module (construct rung)

**Inputs:** a candidate set, each candidate's allowed-register set, and a cost
array. **All three are given to it**; it computes none of them. So it is a pure
function of arguments the scheduler does not supply.

**Deliverable:** `codegen::regalloc::select` — the minimum-cost walk with the
fixed order as a **named, settable parameter**, not a baked constant. That is
decision 15's explicit instruction for every general layer, and this is the
first subsystem where the parameter to expose is already read and obj-confirmed.

**Grading:** construct rung — `Fixtures: none`, `Census: +0`, **required-zero
byte delta**, identity-diff on the 21 gate rows. Re-express the already
byte-exact classes' register choices through the module and require the objs
not to move. Board `#290`'s pattern.

**The honest caveat, which the page supplies and this brief will not bury:**
`P_REGALLOC` §3's correction box records that on all 10 cells of `wb-live`'s
grid and all 15 of `wb-regalloc`'s, **every cost array is uniformly zero over
its allowed set** — the answer is decided entirely by list order. So L1's cost
arithmetic is `[R]` and will stay `[R]`; what L1 makes executable and testable
is the **order**, which is the part that is `[O]`.

### L2 — the worklist comparator (construct rung)

`0x10b2b82d`'s sorted insert, including the `<=` tie tier and the
re-entry-by-priority rule. Also a pure function, also scheduler-independent,
and it has **20 obj-confirmed order cells at two profiles** to test against —
the densest evidence any allocator claim in this repo has.

**Why it is worth a lane on its own:** the two consequences the page names are
both *falsifiable predictions about a port that does not exist yet*. Writing
the comparator and running the 20 cells through it either confirms them or
finds the read wrong, and either outcome is a goal-(1) deliverable.

### L3 — close the two named empty cells (characterization)

`P_REGALLOC` §7 lists exactly two things that are read and have **no obj cell
in existence**:

* the FPR order at `0x10c37f20` — *"read and never obj-checked; no cell in any
  grid uses floating point"*;
* F4's non-call physical def.

Both are buildable with fixtures this project already knows how to write. This
is the cheapest `[R]` → `[O]` conversion available in the subsystem and it moves
the agreement strength, which is currently 7 of 49 (14.3 %) — the second-lowest
of the ten.

### L4 — price F0 properly, by reading (characterization)

F0 is quoted at 8 in one place and 4 raw in another (`READ_PLAN` R7). Under
read-before-probe the correct next move is not to pick one but to **price the
read that settles it**. Until F0 has a real price, every statement about when a
full allocator becomes possible is a guess wearing a number.

---

## 4. The inliner, continued alongside

Machine-checked from `work/w-inlmetric/CLAUSES.tsv` on this tree — 24 clauses:

| state | n | meaning |
|---|---|---|
| `absent` | **17** | no counterpart in `crates/`, token verified absent |
| `unexercisable` | 3 | no compilation this project runs reaches the clause |
| `fitted` | 2 | a counterpart exists and is a black-box fit |
| `R-derived` | 2 | the port's counterpart comes from the same field c2 tests |

**The reachable denominator is 21, not 24**, and the two `fitted` clauses are
the highest-leverage cells on the table: replacing a fit with a read is exactly
what `WHITEBOX_LEVERAGE`'s doctrine is for, and the port currently carries a
**fitted inline predicate** (the scoreboard's own residue text). Two clauses is
a small, well-bounded lane with a table that already grades it.

---

## 5. What this brief deliberately does not propose

* **A full allocator.** §2.
* **A `ported` numerator for regalloc.** The scoreboard says a site-level
  numerator *is not yet defined* for this subsystem, and inventing a
  denominator to make a percentage move is `#3505`'s failure — four for four,
  every lane dispatched off a constructed ranking found the ranking was an
  artifact. L1/L2 move the **read** and **agreement** strengths and should be
  graded on those.
* **Re-taking `#3534`.** byte-owned stays cited, not re-measured.
* **A gate row.** Any new count-bearing row breaks the identity diff for every
  live lane holding a 21-row base (`w-wire`'s measurement, `#3691`).

---

## 6. What the coordinator needs from the owner

1. **Scale.** L1+L2 is a two-lane wave and the smallest thing that produces
   executable allocator code. L1–L4 is four and closes the subsystem's cheap
   evidence too. Adding the inliner's two fitted clauses makes it five.
2. **Whether to dispatch now**, given the answer to #4 in decision 18 was
   *"not yet — we are still mid review and planning"* about the **broader**
   goal. Per-subsystem work is the current phase and is not the broader goal,
   so this is a scale question rather than a permission one — but it is the
   owner's call and not the coordinator's.
