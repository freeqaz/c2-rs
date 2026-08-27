# W-REGCELLS — the FPR order is `[O]`; F4's "no obj cell exists" is refuted, and 213 of them were already on disk

    Tag:       W-REGCELLS
    Slug:      w-regcells
    Date:      2026-08-27
    Outcome:   instrument
    Fixtures:  none — characterization lane: obj cells for the two things
               docs/whitebox/ref/P_REGALLOC.md §7 names as read with NO obj
               cell in existence. The grids live in
               docs/whitebox/grids/w-regcells/, deliberately NOT in fixtures/cpp/
    Census:    +0 — nothing is admitted, no crates/ file is touched
    Reach:     0, as predicted
    Record:    docs/whitebox/WB_REGCELLS_FINDINGS.md
    Board:     #3706–#3711
    Lane:      L3 of docs/REGALLOC_BRIEF_2026-08-27.md, funded by decision 20

**Outcome word: `instrument`.** The lane produced the cells it was
commissioned for plus a grader that re-derives its own answer key from the
pinned image, and it converted marks on `P_REGALLOC`. It converted no TU and
adopted nothing into `crates/`, which is the shape decision 20 priced it at.

---

## 1. Did the read survive contact with an obj?

**Q1 — the FPR order at `0x10c37f20`: YES, and unambiguously.**
`fp0, fp13…fp1, fp31…fp14`, **20 of 20 graded cells, 0 unscoreable**, at both
`/O1` (the workload profile) and `/Ox` (the fixture profile). Four rivals
refuted, three of them by ≥18 cells. **29 of the list's 32 entries are
witnessed in position**; `fp1`, `fp15`, `fp14` are not, and the finding says so
rather than letting the `[O]` read as the whole table.

**Q2 — F4's non-call physical def: the READ survives; the sentence around it
does not.** `P_REGALLOC` §7's *"still no obj cell in existence"* is **wrong,
and was wrong when it was written.** Conditions (a) *a bare physical def of an
allocatable GPR* and (b) *a candidate live across it* are `[O]` on **216
cells** — 213 of which have been sitting in `scripts/gt_argperm.py --pure`'s
grids since 2026-07. Condition (c), the narrowing separated from ordinary
pressure, is **`[R]` and unreachable by construction**, and that ceiling was
registered before the deciding cell was compiled.

**A refuted claim is the better result and this lane got two of them** — one
against the reference page, one against its own prereg (§3).

---

## 2. What it admits, and what it refuses

**Ships no `crates/` change, adds no `DISCLOSURE.md` row** (nothing was
adopted, so none is owed), **adds no gate row** (`#3691`), admits no function
class, and commits **no obj** — only the two `.cpp` grids, the grader, the
findings, the prereg and the grade transcript.

**Refuses to claim**, in the findings' own words:

* the selector's **cost arithmetic** — every cost array in this grid is
  uniformly zero over its allowed set, so only the **order** is `[O]`, exactly
  as decision 20 §3 said in advance;
* the positions of **`fp1`, `fp15`, `fp14`**, which no cell reached;
* condition **(c)** of F4, with the structural reason stated rather than the
  gap left implicit;
* any statement about **class 5 (VMX)**, whose list is filled at run time and
  was not read.

---

## 3. The axis on which this lane could fail, and it did — twice

`#3336`: a control never watched fail is decoration. Three here, and two of
them went red before they went green.

| control | first arming | after |
|---|---|---|
| **`/Gy` moves no code** (mode X needed COMDATs) | **FAILED, 2 of 10 cells** — the two containing a `bl` | `REL24`-carrying words masked (displacements are section-layout dependent by construction): **10/10 verbatim**, and the negative arm — a deliberately corrupted cell — is still absent |
| **the grader itself** (`grade_fpr_cells.py --selftest`) | **caught a real bug in itself**: the first run classified `bl __savefpr_16` as a clobbering call and silently emptied `fpc_p2`'s live-across-call set | 6 assertions, **3 of them the grader having to REJECT** — an ascending FREE set, `f14` before `f31`, and a body with no FP instruction scored `U` rather than passed |
| **`ctr` and `r12` displace nothing** (`pd_ctr_p`, `pd_lr`) | held | could have gone red — a class-2/3/4 list would have shown here. **A physical def only narrows if the register is in the class's list** |

And the prereg's own premise test: a cell with no FP instruction, or one
holding fewer distinct FPRs than its prediction names, scores `U` and enters no
numerator and **no denominator**. **0 cells scored `U`** — so no count in this
lane rests on an absence, which is the repo's most repeated defect.

---

## 4. The prereg score, by tier, not pooled

| tier | commit | hits | misses |
|---|---|---:|---:|
| **PREREG** (before any `cl.exe` run by this lane) | `2c89de6a4` | **7** | **2** |
| **IN-FLIGHT** (stated before the cell existed, not committed first) | `9d0e23b2d` | **5** | 0 |

**Miss 1** — §1.2 predicted the FPR register **set** would be identical at both
profiles. It differs on 4 of 10 cells. The **rule** is invariant; the set is
not, and the prediction as written names the set.

**Miss 2, and it is the useful one** — §2.2's negative prediction for F4 fell
on the first compile. The reasoning error, named because it is repeatable: it
treated the call **tuple** and the `bl` **instruction** as the same object.
They are not — a tail call is a tuple with no `bl`, and the prologue's
`bl __savegprlr_29` is an instruction with no tuple.

---

## 5. What moved

`c2rs subsys`, `[regalloc]` row, measured on this tree:

```
before   2 agreement : marks [O]  7 of 49 (14.3 %) — [R] 41 [I] 1
after    2 agreement : marks [O] 12 of 56 (21.4 %) — [R] 43 [I] 1
```

**+5 `[O]`; the denominator grew by 7 because the lane also filed new `[R]`
residue** (`fp1`/`fp15`/`fp14`, and F4's condition (c)) rather than closing the
questions silently. `agreement` was the second-lowest of the ten subsystems and
is the strength decision 20 graded this lane on.

**Nothing else moved and nothing else should have**: `read` is unchanged (no
new site was opened), `exercised` is unchanged (still RESIDUE — nothing traces
c2's own addresses over the workload), `byte-owned` stays **cited at `#3534`,
never re-taken**, per decision 20 §2.

---

## 6. Handoffs

* **`w-regsel`** (L1): the register order it must expose as a named settable
  parameter is now `[O]` on **both** classes, and §1.1's homology says it is
  **one generator, not two tables**. `#3709`.
* **`w-regprio`** (L2): unaffected — this lane touched no comparator key.
* **Item F / a future F4 lane**: F4's remaining price is **1, not 2**, and its
  proposed fail-closed boundary *"refuse on any bare physical def"* **must not
  ship** — priced two-sided it withdraws every permuted call, a class the port
  emits today. `#3710`.
* **Whoever reads `FUN_10bfb00d`'s class-5 fill**: check it against §1.1's
  rule before transcribing a third table.
* **A lane wanting the last 3 entries**: build a body with 14 simultaneously
  live no-preference FP values and **no call**. `#3707`.

---

## 7. Gate

`scripts/gate.sh` verdict line and `cargo test --workspace --release` at the
lane tip: see §8 of `docs/whitebox/WB_REGCELLS_FINDINGS.md`'s artefact table and
the transcript committed at `work/w-regcells/run/`.
