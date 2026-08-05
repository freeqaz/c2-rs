# w-sched — PREREG. The store/producer schedule, attacked as a scheduler and not as a rule

    Tag:       w-sched-prereg
    Slug:      w-sched-prereg
    Date:      2026-08-05
    Fixtures:  none — this is a prereg. It admits no shape, moves no
               accept/refuse boundary and emits no obj byte.
    Census:    706557/2463393 (28.68%) at lane start — to be re-read at the end.
    Record:    this file. Findings in `docs/rungs/_2026-08-05-w-sched.md`.
    Lane:      w-sched, worktree `wt-w-sched` off master `33f169d`.

**Committed before the grid is generated and before any fit.** The control in §1
was run before this file was written and is labelled as such; it is a
known-answer reproduction of two *published* tables, not a fit.

---

## 0. The problem, and why this attack is different

The store/producer schedule has been refuted **twelve times by four lanes**:
`w-pair` §4 (six placement rules), `leaf_store.rs` (four allocation rules),
`w-dclass`/B **F4a** (fits 6/6, refuted by `o7` and `xboxheap`), and `w-conv`
("unpriceable"). Every one of those fitted **a rule to the cells in front of
it**.

Three things change here.

1. **The observable is the FULL EMITTED PERMUTATION, not a gap.** `w-pair`
   measured "slots between producer and first consumer". A gap is a *summary*;
   two different schedules can share a gap, and at least three of w-pair's six
   rules are stated in gap terms. Every cell below is scored on the exact
   instruction sequence.
2. **The hypothesis class is a LIST SCHEDULER, searched, not a rule, guessed.**
   The family is declared in §3 with its free parameters. A search over a
   declared family can *fail*, and its failure is a measured statement about the
   arity of the problem — which is the deliverable the brief asks for if no rule
   survives.
3. **The holdout is mechanical and enforced by the tooling.** `grid.py` writes
   `fit.tsv` and `holdout.tsv`. The fitting script reads only `fit.tsv`; the
   holdout is not opened until the model is committed. Partition rule in §4.

**No fitted schedule rule will be shipped into `crates/` unless it holds on the
held-out partition AND on `o7` AND on `xboxheap`.** `w-dclass` fitted F4a 6/6
and did not ship it; that was correct and it is the standard here.

---

## 1. Control (run before this prereg; a reproduction, not a fit)

`work/w-sched/control.py` compiles the union of `w-dclass`/B §3.4's ten cells
and `w-pair` §4's twenty cells in **one** TU at the workload's own flags
(`work/dc3-workload/flags.txt`, read from the file) with `/FAsc`, and parses the
listing.

**30 PROCs, and every cell whose emitted order either rung published reproduces
exactly** — o1..o8, w1, and w-pair's C0 control, C1, C2f, D1, D2, D3, D7, C7,
C8, D6, E5, C5, D5, E3, E1, E2, F1, F2, C3, C9. The gaps w-pair published (2, 3,
3, 3, 3, 7, 3, 3, `4,3`, 1, 1, 1, 1, 3, 1, 1, `4,4`, 5) all re-derive.

The instrument is therefore sound on 30 published cells before it is used on
anything new.

---

## 2. Registered predictions

| # | prediction | how it is scored |
|---|---|---|
| **R1** | The control reproduces both published tables exactly | already **HIT** (§1), recorded as a control |
| **R2** | **`xboxheap` is not a third regime.** Its two *value* producers sit at slots **0 and 2**, exactly like `c3`/`o5`/`o7`/`e5`; the published "0, 2, 5" counts `mr r31,r3` — a live-range save across the call, not a store-value producer — as a third producer. Concretely: **`c3`'s emitted sequence equals `xboxheap`'s after deleting the 4 prologue, the `mr r31,r3`, the `bl`, the `mr r3,r31` and the epilogue** | instruction-for-instruction compare against the reference obj for `src/xdk/nuispeech/xboxheap.cpp` at the workload's flags |
| **R3** | **Single-`li`-producer law.** For `n` word stores through one base pointer with exactly one literal-valued store at source index `i`: the producer issues at slot 0, and the stores issue in source order except that the consumer is displaced to the earliest slot `≥ 3`. Holds on **all** tier-1 cells | exact permutation, tier 1, `n ∈ 2..9`, `i ∈ 0..n-1` |
| **R4** | **Arity, upper half.** The permutation is INDEPENDENT of store width, of which formal supplies a non-produced value, of the literal's value, and of the member offsets used. Tier-7 control rows are permutation-identical to their tier-1 counterparts | exact permutation |
| **R5** | **Arity, lower half — a REFUTATION I expect.** The permutation is **NOT** independent of producer *kind*. `d5` (producer `addi r11,r3,8`, consumer at source index 0, gap **1**) and `e2` (producer `addi r11,r4,8`, same consumer index, gap **3**) differ only in the producer's base register, so at least one axis beyond (n, consumer indices) is live. I predict the discriminating axis is **producer base == store base**, and I predict that the clean controlled cell `g1` (producer `addi r11,r3,8`, *all* stores through r3, consumers at 0 and 3 — the swap `e1`/`e2` failed to isolate because it moved both) comes out at **gap 1** | exact permutation, tier 6 |
| **R6** | **Multi-producer slotting.** With `k ≥ 2` producers the producers occupy slots **0, 2, 4, …** in at least **90 %** of tier-4/5 cells | producer slot vector |
| **R7** | A member of the declared list-scheduler family (§3) reaches **≥ 95 %** exact-permutation accuracy on the FIT partition | exact permutation, fit partition |
| **R8** | Held-out accuracy is within **5 points** of fit accuracy | exact permutation, holdout partition |
| **R9** | **TU match at lane end ∈ [9, 10].** Point estimate **9** | `c2rs gap` |
| **R10** | **`xboxheap` does NOT convert in this lane.** It needs F1/F2/F3 (three parse-layer facts) as well as the schedule, and this lane is scoped to the schedule | `c2rs gap` |
| **R11** | At least one premise carried in my brief is refuted by measurement | named in the findings |

**R2 is registered as a prediction even though §1's control makes it very
likely**, because the confirming compare (against the real `xboxheap` obj) has
not been run at the time of writing. Recording it any other way would be
back-dating.

---

## 3. The declared hypothesis family

A **list scheduler** over the block's dependence DAG, with these free
parameters and no others:

| parameter | values searched |
|---|---|
| direction | forward (top-down), backward (bottom-up, result reversed) |
| producer→consumer latency `L` | 1..6 |
| store→store latency | 1 (fixed) |
| priority | source position · critical-path height · height then source · source then height · producer-first · store-first |
| tie-break | source order · reverse source order |
| memory model | stores through the same base at distinct constant offsets are independent; stores through *different* base registers are ordered (may-alias) |

The dependence DAG is built from the source statements. A producer with two
consumers is one node with two out-edges. **Nothing outside this table is a free
parameter**; if the search needs one it is reported as an *added axis* and
counted, because the arity is the deliverable.

---

## 4. The holdout partition — declared now, enforced by the tooling

`work/w-sched/grid.py` writes every cell to `fit.tsv` or `holdout.tsv`. A cell
is **HELD OUT** iff any of:

1. `sha1(cell_id).hexdigest()` interpreted as an int is `≡ 0 (mod 4)`; **or**
2. it is a tier-1 cell with `n == 7`; **or**
3. it is a tier-5 cell (three producers — `o7`'s regime); **or**
4. it is a tier-3 cell (one producer, two consumers) with consumer indices
   `j - i >= 3` — `xboxheap`'s regime.

`fit.py` refuses to open `holdout.tsv`; `score.py` opens it and is run **after
the model is committed**. The commit hash of the frozen model is printed by
`score.py` and quoted in the findings.

**External check cells, never fitted:** `o7`, `xboxheap` (through its real obj),
`c3`, `c9`, `d5`, `e2`, and w-pair's `F1`/`F2`.

---

## 5. Bars

* `cargo test --workspace --release` — baseline **818 passed / 0 FAILED / 27
  targets**; the **target count** is the number checked.
* `scripts/gate.sh --jobs 6` — **PASS, 18/18, 4,500 verdicts**; sweep 16,394 /
  16,298 graded / **96 ungraded**; cross 75,829 of 76,217 / **388 ungraded**;
  0 mismatch on both. Registered *before* the run.
* `c2rs gap` — **match 9**, mismatch 0, vocab-gap 862, capture-fail 7;
  A/B/C/D/E = 28 (LO 27)/338/169/9/2; `A∧B∧C` 27; FRONTIER 18.
* `scripts/status.sh --check` PASS (23 metrics); `board_audit.sh` 0/0/0.

## 6. Board rows taken

**#520–#525**, per the brief's instruction to start at #520.
