# w-layout — PREREG, frozen before the first `crates/` change

    Lane:   w-layout
    Kind:   construct rung (docs/rungs/README.md § "Lane kinds"; precedent
            board #290 / #3072 / #3078 / #3114 / #3119)
    Base:   ba23e8c3  (master, 2026-08-15, "docs: regenerate STATUS after wave
            three")
    Builds: board #3124 — put the crate's hand-patched branch sites into
            `codegen::block_ir::BodyLayout`, so that **a site's position is a
            thing the layout owns** rather than a constant baked into each
            lowering. The PREREQUISITE for a relaxation pass, never the pass.

This file is frozen. It is not edited after the first `crates/` byte moves;
the rung doc (`2026-08-15-layout.md`) scores it.

---

## 0. The success criterion, stated before the claims

**Required-zero byte delta.** This lane converts **zero** TUs *by design*. A
conversion would mean behaviour moved, which is this kind's failure mode and not
its prize. Grading is a **line-for-line identity diff** of the 878-TU scan's
`gap-metric` keys, its 878 per-TU verdict lines, and every gate lane's
fixture-verdict counts, base against tip. **Any byte that moved = `FAILED`**,
whatever else the lane achieved, stated in that word, with the moved byte named
and not rationalized.

**Absence is not the check.** Every row below is graded **positively on
content**: the scan must print its `gap-metric` block and be digit-for-digit
equal; the gate must print `graded > 0` per lane and the per-lane counts must be
equal; the workspace run must print a pass count **and** a target count. A
missing number is a MISS, never a zero.

## 0.1 Reading the item's TITLE and its SENTENCE separately — priced up front

`w-ir-g` (#3114) and `w-item-d` (#3119) were both bitten by an item's prose,
in opposite directions, and the rule is now standing. This lane is working
directly on #3119's consequence, so #3124's own two halves are read apart
**before** anything is built:

* **#3124's title** — *"item A has ONE production client and the crate has
  twenty-three hand-patched branch sites"*. This is a **count**, it is checkable,
  and it is re-derived on this base rather than quoted: §1.4.
* **#3124's sentence** — *"migrate the 23 sites onto `BodyLayout`, then the pass
  has somewhere to stand"*. Registered here as **predicted only partly
  performable today**, and the reason is not effort. Two fences that are *not
  this lane's* stand between a subset of those sites and any layout:
  * **`LabelMap` invariant 4 refuses every BACKWARD reference** (#746; peer
    `w-slots` holds the fence; #3089 says it costs zero on both sides). A body
    with one back edge cannot go through `BodyLayout::finish` at all, because
    `finish` resolves *every* branch through the one map. That is 3 modules.
  * **`Terminator` has no CTR variant**, and `CFG_SHAPE.md` §6.3 declines
    CTR-loop discovery. A body whose back edge is `bdnz` has no terminator to be
    spelled with. That is 2 modules.
  * *Prediction **P16**: the migration is **partial by construction, not by
    budget**, and the residue is exactly the loop bodies. `#3124`'s "migrate the
    23 sites" is not performable in full until #746's fence moves.* (p 0.88)

## 0.2 The second fact the sites depend on, registered before it is discovered

The 23 sites are not the only thing those 13 lowerings compute out of a running
`t.len()`. Every one of them also **publishes** offsets — `bl_offsets` (a REL24
site, whose word encodes its own `.text` offset, §3.3/#191), `prolog_len`,
`FpConstRef::{hi_off, lo_off}`. Those are positions too, and a lowering cannot
hand its branch positions to a layout while keeping its relocation positions in
a running counter, because both come off the same `t`.

*Prediction **P17**: moving the branch sites **forces** the published offsets
into the layout as well, and one new fact — **where a placed block starts** —
is sufficient for all of them. No second mechanism, and in particular no
per-site "mark" type.* (p 0.75)

---

## 1. The registered numbers

**Every numerator carries its denominator** (#3125: three population
conflations in three waves).

### 1.1 The required-zero rows

| # | claim | value | p |
|---|---|---|---|
| P1 | 878-TU scan `match` | **25** of 878, delta **0** | 0.97 |
| P2 | 878-TU scan `mismatch` | **0** of 878, delta **0** | 0.97 |
| P3 | 878-TU scan `codegen-gap` | **0** of 878, delta **0** | 0.96 |
| P4 | 878-TU scan `vocab-gap` | **845** of 878, delta **0** | 0.96 |
| P5 | 878-TU scan `capture-fail` | **8** of 878, delta **0** | 0.90 |
| P6 | 878-TU scan `frontier` | **2** of 878, delta **0** | 0.95 |
| P7 | fixture **census**, every prefix | **+0** | 0.97 |
| P8 | 878-TU scan **`gap-metric` keys** | **372** of 372 identical, digit for digit; `fnbyte-exact` **35,734** at both ends | 0.94 |
| P9 | 878-TU scan **per-TU verdict lines** | **878** of 878 identical (name, class, reason), compared **sorted** | 0.95 |
| P10 | `gate.sh --jobs 4 --require-graded` `graded tree` | **identical at both ends of each run**; a move VOIDS that run and is diagnosed, not rationalized (#3075, #3117). Base predicted **`e89e9b9be058`, 730 files** | 0.93 |
| P11 | every gate lane's **fixture-verdict count** | **identical**, `diff` over the lane table empty. (The table's own line count is *not* registered — #3119's P11 registered `24` by copying a peer's doc and the base had 25. It is not a property of the port.) | 0.94 |
| P14 | fnbyte-exact delta | **0** of 35,734 | 0.96 |

### 1.2 The deliverable rows

| # | claim | value | p |
|---|---|---|---|
| **P18** | **`reach::direct` call sites moved into `BodyLayout`** | **14 of 23** source sites, in **7 of 13** modules. **Stretch 15/23 (8/13)** if the Class-C helper epilogue admits `Terminator::TailCall` (§2.3). The denominator **23 is a count of source call sites**, as #3124 and `reach.rs`'s header count them — it is **not** the number of emitted branch *words*, which is data-dependent (`guard_ret_chain` emits one per guard) and is not registered | 0.62 |
| **P19** | **sites that CANNOT move, and the fence each is behind** | **9 of 23** in **6 of 13** modules — **6** behind `LabelMap` invariant 4 (`ptr_walk_loop` 2, `ptr_walk_chain_loop` 2, `json_utf8_copy` 2), **2** behind the absent CTR terminator (`pool_ctor_chain` 1, `xtea_encrypt_loop` 1), **1** behind the Class-C helper epilogue (`xlrc_create_guard`, the stretch). 14 + 9 = 23, and the partition is stated so it can be checked | 0.75 |
| **P20** | **item A's production-client count** | **1 → 8** (`cond_tail` plus the 7 migrated), or 9 on the stretch. `CFG_SHAPE.md` §6.2 item A's ✔ block gets a dated amendment **in place** | 0.62 |
| **P21** | **new mechanism in `block_ir.rs`** | **exactly one new fact** — *where a placed block starts* — reachable from the finished body. No new per-site type, no second fixup list, no relaxation, no `Form` variant, no `Terminator` variant | 0.70 |

### 1.3 The instrument rows

| # | claim | value | p |
|---|---|---|---|
| P12 | `cargo test --workspace --release --no-fail-fast` | base **1,602 passed / 42 targets / 0 failed**; tip **1,602 + N**, `N` a **CEILING of +40 with NO discount factor**, **42** targets, **0** failed | 0.85 |
| P13 | `git grep -c '#\[test\]' -- 'crates/*'` | base **1,612**; the delta **agrees with the runner's**, and the *level* runs a **constant +10 ahead** (#3076, seventh reproduction) | 0.85 |
| P22 | mutants | **at least 5**, each watched **red** and reverted, and **at least one with a real-obj control** — a byte oracle built from a real `c2.dll` capture, not from the model. `w-ir-g` set the bar at six mutants with two independent real-obj controls; `w-item-d`'s `RC` reddened 34 | 0.90 |

**P12's unit is the change's unit, and the ceiling has no discount.** Five of
six times a discount was applied on this project it was the error. The change is
one new accessor on `BodyLayout`/`FinishedBody` plus seven emitter
re-expressions. The accessor carries its own tests (a positive statement of
where each block landed, the refusals, an unplaced-block case); each migrated
module keeps its existing byte oracles **unchanged** — those add zero — and may
gain one structural test naming its blocks. Ceiling **+40**.

---

## 2. What is being built, stated before it exists

### 2.1 The one new fact

`crates/c2-core/src/codegen/block_ir.rs`: **`FinishedBody` can be asked where a
placed block starts.** That is the whole mechanism. Every position a lowering
publishes today out of a running `t.len()` — a `bl`'s REL24 site, a float
constant's `REFHI`/`REFLO` pair, a prologue's length — becomes
`start_of(block) + <an offset inside that block's own run>`, where the second
term is a constant of a **block**, not of the whole body.

That is the property #3124 asks for and the reason it calls the migration a
prerequisite: **a fixed site has nowhere to grow**, but a block-relative site
grows with its block. A later relaxation pass that inserts a word inside
`finish` changes every `start_of` answer and the lowering needs no edit.

### 2.2 What each migrated lowering becomes

`declare` its blocks, `place` each with its straight-line run and its **one**
terminator, `finish`, then derive its published offsets from block starts. The
branch displacements are computed by `LabelMap` — which is where they were
always supposed to be (§6.2 item B, board #290) — and the lowering stops
spelling them.

### 2.3 The stretch, stated as a condition and not a hope

`xlrc_create_guard` ends in the Class-C helper epilogue, whose last word is an
**external** `b __restgprlr_N`. That is `Terminator::TailCall`'s shape — a
placeholder plus a reported site — but the word is encoded by
`frame::epilogue_gpr_helper` from an absolute `.text` offset. It moves **only
if** that composes without a new mechanism. If it needs one, it does not move,
and P18 lands at 14.

---

## 3. What is NOT built — declined here, before it is tempting

* **No relaxation pass, and no application of `reach::LongBranch` to anything.**
  #3124 is explicit that the reverse order builds an ungraded code path (w-frame
  row **F-c**), and `encode_b_intra`'s own header records that mistake being made
  and reverted once already. This lane builds the floor the pass would stand on
  and does not stand on it.
* **No loops.** The binding fence is `label_slots → None` (#746); peer `w-slots`
  is working it; `LabelMap` invariant 4 stays exactly as written and this lane
  adds no second, friendlier copy of it. `block_ir`'s
  `a_backward_branch_is_refused_by_the_label_maps_own_rule` must still be the
  test that fires, reading `labels.rs`'s own words.
* **`crates/c2-core/src/codegen/labels.rs` is NOT edited** — peer `w-slots` may
  need it. `AD-e` (#3123, `Form::encode` private, the dispatch written twice)
  stays open; closing it is that file's owner's one-line change.
* **`coff/` is not touched.** Single-occupancy.
* **No code motion, no cost model, no loop rotation, no CTR-loop discovery, no
  neutrality classifier, no instruction scheduling** (§6.3). **Item F is not
  re-priced** — its floor is #3111's and this lane does not go near it.
* **No shared predicate is narrowed, shadowed or redefined.** Before any type or
  method is added, the crate is asked whether it already has a reader of that
  fact. Three incidents here of lanes colliding through semantics with no textual
  conflict.

---

## 4. How it is graded

1. **BASE** captured at `.claude/worktrees/w-layout-base`, a checkout of
   `ba23e8c3` that **nothing edits** for the lane's whole life (#3075).
2. Build on `wt-w-layout`.
3. **TIP** captured the same way, and diffed **line for line**.
4. Any byte that moved → `FAILED`, named.
5. **The mutation battery.** A construct rung's zero is worthless if nothing
   could have made it non-zero. Each mutant is applied, watched red, reverted,
   and the tree re-verified green.
6. Portable unit tests, `git grep` and the runner reported at both ends.

Each gate runs as the **sole writer of its own output file, in the foreground of
one job** (#3117/#3128). No two gates run at once.
