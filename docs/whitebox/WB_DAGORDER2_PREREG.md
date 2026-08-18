# WB-DAGORDER2 — PREREG: the register allocator's candidate worklist order at `0x10b31c9a`

    Lane:      w-dagorder (worktree suffix `w-dagorder2`; `wt-w-dagorder` was TAKEN)
    Kind:      characterization
    Base:      master 44794fa4
    Frozen:    BEFORE the first grep of ~/ghidra-projects/export/c2/ and
               BEFORE the first cl.exe of this lane.

---

## 0. THE LANE WAS RETARGETED BEFORE IT PROBED, AND THIS SECTION IS WHY

The standing brief named **`STRATEGY_REVIEW` §4 lever 3 — read `dag.c`'s
lowering order at `0x10b3219f`** — as *"the sole remaining characterization
blocker for `CFG_SHAPE.md` §6.2 item F"*.

**That target was already discharged, five days before this lane was
dispatched.** Lane `wb-dagorder` landed it on 2026-08-13
(`docs/rungs/2026-08-13-dagorder.md`, `docs/whitebox/WB_DAGORDER_FINDINGS.md`,
board **#3067**–**#3071**), and it is in `master`: the branch `wt-w-dagorder`
exists, `git rev-list --count master..wt-w-dagorder` is **0**, and the merge
base is the branch tip. Its answer — `0x10b3219f` is one helper of a
dependence-DAG **list scheduler**, not a tree-to-tuple walk — was taken at the
workload's own `/nologo /c /GR /O1 /Oi /EHsc`, over a 15-cell grid frozen by
content hash, with a committed reference simulator reproducing 7 of 8 call-free
cells. Two further lanes have landed on top of it: `w-dagclients` (#3099–#3103,
the four DAG clients that bypass the region finder) and `w-itemf-price`
(#3166–#3170, item F decomposed and priced).

**Re-deriving it would be this repo's documented failure mode** (memory:
*"Check the board before dispatching — five rows re-entered the ranking after
already measuring zero"*). So the lane takes the **named remaining** blocker
instead, and the record names exactly one:

> **#3169** — *"`codegen::alloc`'s FITTED SORT AND `0x10b31c9a`'s UNREAD
> WORKLIST ORDER ARE THE SAME UNKNOWN FROM TWO SIDES, WHICH NOBODY HAD
> CONNECTED."* Item F step **F5**, `WB_ITEMF_FINDINGS.md` §5: *"The register
> order `[r11 … r3, r31 … r14]` is settled and cheap. **Which candidate is
> coloured first is not.**"*

and **#3166** closes with the dispatch instruction this lane is executing:

> *"A HYPOTHESIS RIDES ON THIS AND IS LABELLED, NOT BANKED … **This lane built
> no cell that could falsify it and does not claim it. Testing it is the
> cheapest experiment the lane found.**"*

**The retarget is inside the lane's authority and not a scope change**: same
kind (characterization), same seam (`docs/` + `docs/whitebox/` only), same
predicted reach (**0**), same phase (item F), and the same phase driver — the
brief's `0x10b3219f` and this lane's `0x10b31c9a` are two stages of
`FUN_10b7dc51` (`#3166`: `0x10b38099` · sched · globregs `0x10b57633` · sched ·
**`0x10b31c9a`** · sched).

---

## 1. The question, in one sentence

**In what order does `0x10b31c9a` colour the candidates of a register class,
and is that order a consequence of the scheduler `wb-dagorder` found rather
than a property of the source?**

The one published datum is `WB_LIVE_FINDINGS.md` §10, recorded as an open fact:

> *"The `wbl_x2` assignment order is unexplained: `a` took `r30` and `b` took
> `r31`, i.e. the **second** formal got the head of the callee-saved run.
> #1821's tie-break predicts the first candidate coloured takes `r31`; which
> candidate is coloured first is set by the driver's worklist order, which this
> lane did not read."*

`wbl_x2` is `extern "C" int wbl_x2(int a, int b){ wbl_void(0); return a + b; }`.

## 2. The rival readings, named before any cell compiles

| # | reading | what it predicts for `wbl_x2` | what it predicts for `return b + a` |
|---|---|---|---|
| **H-SRC** | worklist is source/formal order | `a`→`r31`, `b`→`r30` | unchanged |
| **H-REV** | worklist is reverse source order (a LIFO push list) | `a`→`r30`, `b`→`r31` ✔ | **unchanged** |
| **H-SCHED** | worklist order follows the **scheduled tuple order**, and `wb-dagorder`'s measured **right-first operand lowering** (`dg_sub`/`dg_sub2`) touches `b` before `a` | `a`→`r30`, `b`→`r31` ✔ | **FLIPS** — `a`→`r31`, `b`→`r30` |
| **H-USE** | worklist is `ORDER.md`'s rank *(use count desc, first-use asc)* — `codegen::alloc` clause 1 | tie at 1 use each; falls through to a tie-break | unchanged |
| **H-ARR** | worklist is arrival-register order descending (`r4` before `r3`) | `a`→`r30`, `b`→`r31` ✔ | unchanged |

**H-REV, H-SCHED and H-ARR all fit the single published cell.** That is the
whole reason this must be published as a **series over cells** and not as a
cell: `wbl_x2` alone cannot separate three readings, and #3147's correction
(`w-slots`, reinforced by `w-bind16`'s `2n+1` and `w-section`'s R-SEC at
n=1..4) says a single cell is not a finding.

## 3. Registered predictions — probability form

Scored verbatim in the findings doc. Misses stay on the page as misses.

| id | prediction | p |
|---|---|---|
| **P1** | The `wbl_x2` result reproduces on this box at the workload profile (`a`→`r30`, `b`→`r31`) | 0.90 |
| **P2** | **H-SRC is refuted** at n=2 | 0.90 |
| **P3** | **H-SCHED is confirmed**: reversing the operand order in the source (`b + a`) **flips** the register assignment, with the formal list unchanged | 0.55 |
| **P4** | The assignment is a **monotone series in n**: for n formals live across a call, the callee-saved run `r31, r30, …` is handed out in one consistent order over n = 1..8, with no n at which the rule changes kind | 0.60 |
| **P5** | The series **breaks at some n ≤ 8** — i.e. P4 is false and there is a small-n artifact, exactly as `wb-dagorder`'s `lis` hoist broke at n=3 (#3068) | 0.40 |
| **P6** | `/Ox` and `/O1` **agree** on the candidate order for every cell of the grid | 0.55 |
| **P7** | The candidate list `DAT_10c400d8[class]` is built by an **insertion at head** (making H-REV a mechanism, not a coincidence) | 0.45 |
| **P8** | At the end of the lane, **item F is still NOT unblocked**, and F5 is not the only thing left | 0.75 |
| **P9** | Some cell of the grid is **not explained by any of the five readings** | 0.50 |
| **P10** | This lane adopts **nothing** into `crates/` and files **no** `DISCLOSURE.md` row | 0.85 |

## 4. Ceilings — NO discount factor

* **Predicted reach: 0.** Census **+0**, `crates/` byte-identical at both ends,
  0 TUs converted, 0 fixtures claimed. `w-itemf-price` **#3170** measured that
  completing item F entire buys **0** in all four named populations (878-TU
  scan, the 381×18 fixture gate, `c2rs perf`, and the frontier), so this lane
  **must not** be justified by conversions and does not claim any.
* **Ceiling on what a positive result buys:** at most it converts F5 from
  *"not buildable"* to *"buildable"*, which by `WB_ITEMF_FINDINGS.md` §6.1 is
  **2 of item F's 17 lanes**. It does not move F0 (8 lanes), F1 (2) or the rest.
* **Ceiling on the black-box half:** a grid can show *that* the order follows
  the schedule; it cannot show the list's data structure. If H-SCHED confirms,
  the mechanism claim still rests on the export.
* **No `crates/` change is authorized by any outcome of this lane**, including
  a confirmed H-SCHED. `codegen::alloc`'s clause 2 stays refuted-and-shipped;
  repairing it is a construct rung, not this lane.

## 5. Probe soundness — the control, pinned by NAME

`docs/rungs/README.md`'s rule added 2026-08-17 (#3219/#3231): a fresh worktree
has no `compilers/` and capture-based work **silently skips**.

* **Control, pinned by name:** `fixtures/cpp/w5_chain.cpp` must report
  **`4/4 functions in class`** from `c2rs census` in every environment this
  lane compiles in. `scripts/setup_worktree.sh` already ran it once at
  provisioning and it returned `OK: fixtures/cpp/w5_chain.cpp -> 4/4 functions
  in class`; it is re-run and logged at the head of **every** probe batch.
* **Assert executed counts and durations, not exit codes.** A probe batch that
  compiles k cells must report k non-empty `.obj` **and** a nonzero wall time;
  a batch reporting a 0.00 s differential is **void**, not provisional.
* **A colour taken in an unvalidated environment is void, not provisional** —
  discarded and re-run, with the invalid log kept.

## 6. Grid freeze

The grid `docs/whitebox/grids/wb-dagorder2/candorder_grid.cpp` is frozen by
**content hash** in `WB_DAGORDER2_PREREG_R2.md`, committed before the first
`cl.exe` of this lane. A hold-out frozen by name is not frozen.

## 7. Seams

`docs/` and `docs/whitebox/` only. `crates/` **byte-identical at both ends** —
this is a revert-everything lane, so `graded tree` identical at both ends
**applies** and is recorded. Peer `w-dataseam` owns `c2-il`'s
`data_syms`/`bind.rs`; peer `w-calleeguard` owns `c2-harness/src/gap/tests.rs`
and `crates/c2-harness/tests/`. Neither is touched.

Board rows: **#3239**–**#3243**, allocated by the coordinator. The next-free
pointer in `BOARD.md` is not consulted.
