# PREREG — `w-inlbudget`, wave 18 (decision 22, brief §L2)

    Lane:    w-inlbudget
    Kind:    construct rung (adoption)
    Branch:  wt-w-inlbudget, based at master 4b79bf46a
    Board:   #3762–#3767 (this lane's whole range; no row outside it)
    Written: 2026-08-28, and COMMITTED BEFORE THE IMAGE WAS OPENED.

**Nothing in this file was written after running `grab.py`, `peread.py`,
`objdump`, or reading `~/ghidra-projects/export/c2/`.** The only inputs so far
are this repo's own text: `CLAUDE.md`, `docs/ADOPTION_BRIEF_2026-08-28.md`,
`docs/DECISIONS_2026-08-22.md` § Decision 22, `docs/rungs/README.md`,
`crates/c2-core/src/surface.rs`, `crates/c2-core/src/splice.rs`,
`docs/rungs/2026-08-27-w-inlfit.md`, and `docs/whitebox/ref/P_INLINE.md` §6.6.2.

---

## 0. Two corrections to the dispatch brief, recorded before any work

1. **`splice.rs` is at `crates/c2-core/src/splice.rs`, not
   `crates/c2-core/src/codegen/splice.rs`.** The brief and
   `ADOPTION_BRIEF_2026-08-28.md` §4's seam table both name the `codegen/`
   path; `ls crates/c2-core/src/codegen/` has no such file. The seam assignment
   is unaffected — no peer lane owns either path — but the row is wrong as
   written and this lane owns the real file.
2. The brief says **13 uncovered** consts and board `#3746`'s own headline says
   **12 of 19**; `UNCOVERED` has **13 rows** and `UNCOVERED_RATCHET` is **13**
   on this tree, because `POOL_TOP` moved into the list. 13 is the live number
   and it is the one this lane is graded against.

## 1. The question

`P_INLINE.md` §6.6.2 (lane `w-inlfit`, board `#3719`/`#3720`) reads c2's
recursive inline expansion and publishes a budget model. The port has no
counterpart. **Adopt the model as an executable, parameterised decision surface
in the port, without moving a byte** — and, per `#3723`, prove the adoption is
graded by something other than the byte delta.

## 2. What I will do, registered as a plan and not as a result

### 2.1 Verify before adopting (`#3336`, and the brief's explicit instruction)

Re-derive each §6.6.2 claim from the image myself. Image
`compilers/X360/16.00.11886.00/c2.dll`, sha256 `c80981c0…a66258` (verified on
this tree before this file was written). Disassembly is the independent objdump
listing, regenerated and never committed.

| # | the claim as published | address cited |
|---|---|---|
| V1 | the recursion edge into the driver | `0x10b62402` |
| V2 | `FUN_10b61ee1` has exactly two callers | `0x10b6276e` + `0x10b62402` |
| V3 | `level` becomes `BYTE [site+0x18] + level` | `0x10b623f2`, `0x10b623f9` |
| V4 | the budget argument is `*budget / remaining_sites`, an `idiv` | `0x10b623ec` |
| V5 | the divisor is the site collector's out-parameter, decremented per site | `lea edx,[ebp-0xc]` `0x10b61f99`; `dec` `0x10b620c8` |
| V6 | `__forceinline` is charged nothing — the skip covers the local budget AND the global growth total | `0x10b6240f`, `0x10b62418`, `0x10b6241a` |
| V7 | stack 3/4 are one 64-bit quota that halves | `0x10b6204e` |

### 2.2 The adoption, in outline

A new region of `crates/c2-core/src/splice.rs` carrying:

* a `BudgetModel` value type whose fields are the model's decision points —
  the seed clamp (C3), whether the budget is divided among the remaining sites
  (C20), the depth cap (C14), the charge exemption (C18), whether
  `__forceinline` is charged (the §6.6.2 finding), the 64-bit quota and its
  shift;
* a `BUDGET_MODELS` slice, index 0 the c2-read default, every other entry an
  **instrument state that licenses no emit** — `regalloc::ORDERS`' shape, and
  the same pin: a test asserts the only production call site passes the
  default;
* the nested budget represented as **what the port can actually know**. `B`
  itself is unreadable by the port (it needs `WORD [fn+0x50]`, C2/C24, and
  §2.1b measured that field as an upper bound on the tested quantity and not
  the quantity). At `n = 1` the divisor is 1 and the nested budget is the
  parent's *whatever `B` is*; at `n ≥ 2` it is `B / k` and the port cannot
  evaluate it. So the type is `Parent` or `Divided { k }`, and `Divided`
  **refuses**;
* the model wired into `splice_body_why`'s chain walk as a **production
  caller**, so the identity diff is not a tautology over dead code
  (`rungs/README.md`'s construct-rung corollary).

### 2.3 The decision surface

`SURFACES` gains `splice.budget`, marked in `splice.rs`, with a domain over
`(model, n_sites, site_index, level, forceinline)` that **enumerates `n ≥ 2`**
— the region where the port's obligation is to refuse, and which no fixture
reaches because `S2` refuses a two-call body before the walk starts.

### 2.4 The `#3746` residue

Close as many of the 13 `UNCOVERED` rows as honestly possible by registering
new surfaces whose domains actually reach them, lower `UNCOVERED_RATCHET` to
match, and state what could not be closed and why. **Every claimed coverage is
tested by widening the const and requiring the domain to move** — `#3746`'s own
trap: 2 of 7 `guards` entries were false, and re-spelling `POOL_TOP` as a
literal `9` moved zero lines.

## 3. Predictions, registered now

| # | prediction | what it would take to falsify |
|---|---|---|
| **P1** | **All seven of V1–V7 confirm at their cited addresses.** | any one that does not decode as published |
| **P2** | The byte delta is **zero**: `scripts/gate_identity_diff.sh base tip` reads **0 lines over 21 rows**, and `gate.sh`'s verdict line is unchanged base→tip | a moved row |
| **P3** | `DOMAIN.txt` **grows by at least 200 lines** (1102 → ≥ 1302). A construct rung that adds a settable surface and does not move the domain is `#3723`'s own failure mode wearing this lane's name | a domain that does not move, or moves by < 200 |
| **P4** | **At least 4 of the 13** `UNCOVERED` rows close, and `UNCOVERED_RATCHET` falls to ≤ 9 | fewer than 4 |
| **P5** | `cargo test --workspace --release --no-fail-fast` has **no new failure** attributable to this lane | a new failure |

**P1 is the one I most expect to be wrong in part.** §6.6.2 was read by one lane
and the brief says so in those words. A refutation of any row of the V-table is
a better outcome for this lane than a clean adoption, and §5 fixes what each
refutation licenses **before** I look.

## 4. Controls, each to be watched RED before any verdict is quoted (`#3336`)

| # | control | must go |
|---|---|---|
| **C1** | plant a defect in the budget model (flip the default model's `divide` flag) and run `surface::tests::the_decision_surface_domain_matches_the_committed_baseline` | **RED**, then GREEN on restore |
| **C2** | for **each** const this lane newly claims as a `guards` entry: widen it by one step, re-render, require **≥ 1 moved domain line**; restore | **RED** each, then GREEN |
| **C3** | a two-call-site body reaches the budget model and the model refuses by name | a named refusal, asserted in a unit test |
| **C4** | `scripts/gate_identity_diff.sh --self-test` | PASS (its own 14-lines/7-rows signature) |
| **C5** | the `Fail axis:` enforcement in `crates/c2-harness/tests/rung_registry.rs` — delete the field from this lane's rung and require the test to fail; restore | **RED**, then GREEN. This lane is likely the first record the check grades and a check nobody has seen fail is decoration |

## 5. The state change each outcome licenses — fixed in advance

* **P1 holds in full** → adopt the model as read, with a `DISCLOSURE.md` row per
  adopted address. `C20` is **not** promoted out of `fitted`: this lane does not
  own `P_INLINE.md` and cannot edit `CLAUSES.tsv`; the promotion is offered as a
  quotable patch block in the rung and taken by whoever owns those files.
* **P1 fails on a row** → that row is **not adopted**. The model carries the
  parameter with the value marked unread, the rung publishes the refutation as
  its headline, and `#3719`/`#3720` get a correction row in this lane's board
  block.
* **P2 fails** → the lane is **FAILED** as a construct rung, in that word. A
  construct rung that moved a byte failed whatever else it did.
* **P3 fails** → the adoption is dead code and the lane says so; the outcome
  word is `FAILED`, not a compound headline.
* **P4 fails** → report the honest count of closed rows and the reason for each
  that stayed open. Not a lane failure on its own — it is the secondary.
* **No emit widening is licensed by anything here.** The budget model's default
  is the port's current behaviour restated; every non-default `BUDGET_MODELS`
  entry is an instrument state. The sole judge stays real `c2.dll` under wibo
  plus a byte-exact obj compare.

## 6. Fences this lane will not cross

* **No 128.** `#3732` closed it: §2.1b's one-sided rule holds at `T = 98` and
  the image's 128 has 8 counterexamples in each direction over
  `w-sizebracket`'s committed 168 cells. `INLINE_UNBOUNDED_BYTES` is not
  touched.
* **No new count-bearing `gate.sh` row** (`#3691`).
* **No edit to `docs/whitebox/ref/P_INLINE.md`** (`w-inlswitch` owns it), to
  `crates/c2-core/src/codegen/mop.rs` (`w-encarms`), to
  `work/w-inlmetric/CLAUSES.tsv` (`w-clausefix`), to `docs/STATUS.md` or
  `docs/rungs/INDEX.md` (generated).
* **No board row outside `#3762`–`#3767`.**
* **No push.**

## 7. Fail axis (the construct-rung requirement, named here first)

The byte delta cannot fail on this lane by construction — the divisor is 1 on
the admitted set — so it is the floor and not the grade. The axes this rung
**can** fail on:

1. **The refusal domain.** `splice.budget`'s domain enumerates `n ≥ 2`; if the
   model admits where it must refuse, the domain moves and the baseline test is
   red. This is the axis `#3723` says a byte delta cannot see.
2. **Precedence.** The model is called from inside the chain walk. If it is
   asked in the wrong order relative to `S2`/`S6-chain-open`, a body that used
   to refuse for a *named* reason refuses for a different one — visible in the
   refusal-reason census, not in any byte.
3. **Registry completeness.** A `guards` entry this lane adds that does not move
   a domain line under C2 is a **false coverage claim** and is reported as one.
