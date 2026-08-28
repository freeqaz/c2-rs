# The shared floor — both named subsystems bottomed out on the same thing, and it is not a subsystem

**Lane `w-floor`, 2026-08-28.** Consolidation lane funded by
[`DECISIONS_2026-08-22.md`](DECISIONS_2026-08-22.md) **decision 21** §2. Board
**#3749**–**#3754**. This document is half 1; the re-price is
[`REPRICE_2026-08-28.md`](REPRICE_2026-08-28.md).

**Nothing here is a new measurement.** Every figure is re-derived on this tree
(`8213c7b77`) from the lane records cited beside it, and §5 says how to re-run
each one. This lane opened no image, wrote no `crates/` byte, and moved no
census.

---

## 0. The finding, in one paragraph

Decision 15 restructured dispatch onto **per-subsystem scoreboards**, on the
premise that subsystems are independently advanceable. Wave 16 funded five
lanes across the two subsystems the owner named in decision 18 — the register
allocator and the inliner — and **both reached a boundary past which that
premise is false**, from two independent directions, on the same tree, in the
same wave. The boundary is not a third subsystem. It is **per-compilation
accumulated state**: a quantity that no read confined to the subsystem's own
band can produce, because it is the running result of passes that belong to
other subsystems.

> **The premise is amended by measurement, not by argument.** Per-subsystem
> dispatch worked — it is what got both subsystems *to* their boundary in one
> wave. What it cannot do is carry either one past it.

---

## 1. The register allocator — the consuming half is built, and there is no third piece

`docs/REGALLOC_BRIEF_2026-08-27.md` §3 split the subsystem into a part that
**computes** priorities (needs the scheduler) and a part that **consumes** them
(does not), and said in §2 in its own words: *"do not fund a register
allocator. Fund the parts of it that are provably downstream of nothing."*

Wave 16 built the consuming part, whole:

| lane | what it made executable | size on this tree |
|---|---|---:|
| `w-regsel` | `codegen::regalloc::select` — c2's selector `0x10b2e7f8`, with the allocation order as a named, settable parameter (`ORDERS`, four entries) | `crates/c2-core/src/codegen/regalloc.rs` **793 lines** |
| `w-regprio` | the priority worklist comparator `0x10b2b82d`, seven decision points exposed as named parameters | `crates/c2-core/src/codegen/regalloc_worklist.rs` **402 lines** |
| | | **1,195 lines** |

`w-regcells` then closed the subsystem's two named empty cells — the FPR order
at `0x10c37f20` is `[O]` on 20 of 20 graded cells at both `/O1` and `/Ox`, and
F4's non-call physical def was **refuted as an empty cell**: 213 obj cells had
existed for a month, eleven directories away (`P_REGALLOC.md` §7, boards
**#3706**–**#3710**).

**What remains has one shape.** `P_REGALLOC.md` §7, unamended and explicitly
untouched by `w-f0price`'s amendment box:

> a candidate is a **(symbol, live-range version)** pair, and the versions need
> the backward walk over the **lowered** tuple list

> **F5 is not separable from F0.** F5's input is `cand+0x0c`, accumulated over
> the code **the scheduler produced**

So both of the allocator's remaining inputs — the candidate *set* (F1) and the
candidate *order* (F5) — are functions of a tuple list that other passes have
already rewritten. `w-f0price` then made that blocker both larger and better
specified: **F0 is ≥ 9 raw sub-lanes plus two UNPRICED terms** (its published
`≥ 10` is one lane above its own addends —
[`REPRICE_2026-08-28.md`](REPRICE_2026-08-28.md) §2.1), it names **1 of 34**
post-allocator depth-1 passes, **27 are `cover=none`**, and stage **S7** has no
sub-item at all.

**There is no third separable piece**, and that is a statement about the
subsystem's structure rather than about anyone's appetite: everything left runs
through `cand+0x0c`.

## 2. The inliner — the two missing links are named, and neither is in the band

`w-inlfit` read C8 end to end. `P_INLINE.md` §6.6.1 establishes that
`DAT_10c46318` has **exactly one reader in the entire image** (C8's own `cmp` at
`0x10b5fc8a`) and exactly two writers, both inside `FUN_10b5e4cc`; that
`k = DAT_10c2ea98 = 3`, so the ceiling is `0x10 << 3 = 128` instructions; and
that `k`'s address is planted in an option descriptor whose name field resolves
to the undocumented switch **`-vol#`**. That is as complete as a read of one
constant gets.

**And the clause still cannot be replaced.** §6.6.1's own words:

> **So C8 stays `fitted`, and the fit is not replaceable by any read confined to
> `0x10b5b86d`–`0x10b62b00`.**

The two missing links, both named on the page and **neither inside the inliner's
band**:

1. **`[sym+0x50]` is initialised from the `.gl` `SIZE` field at `0x10b9bf6c` and
   is reduced by every pass that runs between there and `0x10b5fc8a`** —
   *"nothing yet located reads that reduction."* §2.1b measures its consequence:
   `arith_012` and `mix_008` carry an identical `SIZE` of 115 and get opposite
   verdicts.
2. **Turning a count into emitted PPC bytes is the whole of lowering** — which
   is the unit the port's own constant is denominated in.

C20 stays `fitted` for a second, structurally identical reason: c2 divides the
growth budget among the remaining call sites (`idiv` at `0x10b623ec`), and the
divisor is the site collector's out-parameter — a per-compilation running count
the port's `splice.rs` has no concept of. §6.6.2's own summary: *"These are
different rules."*

## 3. So what IS the floor — stated as a predicate, not as a mood

Both failures have the same form, and the form is checkable:

> **A subsystem hits the shared floor exactly when its decision function reads
> state that other passes accumulate — and it does not when its decision
> function is a pure function of the single record it is handed.**

Two accumulators are now named on this tree, and they are different objects:

| accumulator | who mutates it | who is blocked on it | evidence |
|---|---|---|---|
| **the tuple list** — its order, and every count derived from it | the 7 stage drivers / 34 depth-1 passes downstream of `0x10b31c9a`; 4 confirmed direct splice callers | regalloc (`cand+0x0c`), inliner (`[sym+0x50]`) | `WB_F0PRICE_FINDINGS.md` §4, §4.2; `P_REGALLOC.md` §7; `P_INLINE.md` §6.6.1 |
| **the label counter `DAT_10c2edd0`** | 163 charging sites across 21+ translation units, **42 of them on loop back edges** | label numbering | `P_LABEL.md` §0, §2(c) |

The second row is the reason this is a *class* and not a coincidence about
tuples. `P_LABEL.md` §2 is the cleanest statement of the floor anywhere in this
repo, and it was written before the floor had a name:

> The site table is closed and finite. The charge is a **sum over c2's own
> object population**, and that population is what a port would have to
> reproduce.

**A closed site population is separability; a closed charge is not, and the two
are not the same property.** That distinction is exactly what `w-regsel` hit
(the selector's *sites* are read and executable; the *candidates* it selects
among are not), what `w-inlfit` hit (C8's operand has three references in 22 MB;
its *value at the moment of the test* has an unread reduction chain), and what
`P_LABEL` measured a week earlier.

### 3.1 The corollary that decides how to dispatch

**A subsystem at the floor is not stalled and is not finished.** Both of ours
are at a genuine, well-marked boundary with real deliverables behind them. What
changes is *which lane kind moves them*: the next unit of progress on regalloc
or the inliner is a lane against the **accumulator**, and it will be dispatched
under a name that is not either subsystem's. Decision 21's `w-sched`,
`w-lowerband` and `w-s7` are exactly that, and none of them is a regalloc lane
or an inliner lane.

---

## 4. The other eight — registered predictions, with falsifiers

`SUBSYS.md` §1 lists ten subsystems. Two are §1–§2 above. **These are the other
eight**, each classified by §3's predicate.

> ### The `#3505` check this lane owes, taken before the table rather than after
>
> **This is a partition by a read predicate, not a ranking**, and no conclusion
> below depends on any ordering. The predicate ("does the decision function read
> cross-pass accumulated state") is evaluated per subsystem from that
> subsystem's own page, and the rows are printed in `SUBSYS.md` §1's order for
> diffability. **The available ranking was deliberately not used**:
> `SUBSYS_METRICS.md` §3 publishes an `[O]`-mark ratio per subsystem (4.2 % to
> 34.1 %) and it is tempting as a "how close is this one" score — but that file's
> own §2 says the mark census *"counts a page's claims about its own evidence
> tier, not sites and not agreements"*, and a page may cover twenty addresses
> with one mark. **Ranking eight subsystems by it would be `#3505`'s sixth
> instance, and it would be this lane's own prereg supplying the bad
> instrument** — the exact failure `w-f0price` scored on itself at §7 P3/P4.
> Nothing below reads that column.

| # | subsystem | shares the floor? | p | the accumulator, or the reason there is none | **falsifier** |
|---|---|---|---:|---|---|
| 1 | **DAG build + scheduler** (`dag`) | **YES — already established, not predicted** | 1.00 | `node+0x44`, the tie-break, is the original index **assigned from the POST-MERGE order** — the mergers run at `0x10b7ded5` before the final schedule, so *"their output is the scheduler's input, and the tie-break `w-dagorder` fitted is fed by a pass it never saw"* (`CEILING.md` §6.1, `w-dagclients`) | none needed; cited as established |
| 2 | **globregs** (candidate SET / ORDER / tie key) | **YES — already established, not predicted** | 1.00 | `P_REGALLOC.md` §7 in its own words: the versions *"need the backward walk over the **lowered** tuple list"* | none needed; cited as established |
| 3 | **compiler-label numbering** (`label`) | **YES — already established, on a DIFFERENT accumulator** | 1.00 | the counter, not the tuple list. 163 sites closed; **42 loop-resident**, so the charge is a sum over a data-dependent population (`P_LABEL.md` §2(c)) | none needed; cited as established |
| 4 | **EH state synthesis** (`eh`) | **YES — predicted** | 0.80 | EH's remaining work is region **cutting** over the node list, and it runs inside the stage nothing prices. `0x10c21fd2` *"walks the node list cutting `.text` into regions"*; `0x10c21b03` is one of `WB_F0PRICE` §4.2's **four confirmed direct splice callers**, in **stage S7**; `0x10c219c4`'s propagation is read and *"its fixpoint is NOT established"* (`P_EH.md` §5). Note the shape: **`0x10c21b03`, `0x10c217fd`, `0x10c220c9`, `0x10bff811` and `0x10b3421b` are all on `P_EH.md`'s own entries table and all outside `SUBSYS.md` §1's stated EH band `0x10be04e7`–`0x10be3800`** — the page already left its band, exactly as `P_INLINE` §6.6 had to | **Read `0x10c219c4` and `0x10c21fd2`'s region cut to a rule, and reproduce the `.pdata` region boundaries on a workload TU with a `try`, using only state available at `ehexcept.c`'s entry.** If that succeeds, the prediction is wrong |
| 5 | **instruction encoder** (`encode`) | **NO — predicted** | 0.90 | Its decision function is `encode(tuple) → u32`, a **total function** of one tuple, recovered by R2 from base-word table `0x10c3a578`, encode-form table `0x10c39b18` and 79 arms. It reads no running state; the read executed in **1 h 36 m** against a 2–4 day estimate. It is also the only subsystem with a real differential (99.38 % of 630,548 `.text` words under the strict mask) and one of only two with a measured `ported` (27 of 79 arms) | **Find one encode arm whose output word depends on any quantity outside the tuple it is handed** (a running count, a prior tuple, a phase beacon). One such arm falsifies this row |
| 6 | **obj writer** (`coff`) | **NO — predicted** | 0.75 | Its input is the finished section/symbol model; its decisions are file-format-local, and its exercised proxy is 100 % by construction (every obj in the workload went through it). No page cites a running quantity | **Find a field the writer emits whose value depends on the order or count of earlier writes** rather than on the model handed to it. `TimeDateStamp` does not count (it is zeroed by the judge) |
| 7 | **symbol records** (storage class, section number, weak externals) | **NO — predicted** | 0.70 | Per-record decisions over one symbol; parts are already `[O]` against the port's own `ObjImage::weak_externals` with known-answer alarms (`SUBSYS_METRICS.md` §4) | **A storage-class or section-number decision that reads the symbol table's prior contents** falsifies it. The weak-external *pairing* is the row's own risk and is where I would look first |
| 8 | **section & symbol model** (`section`) | **NO, with a named risk — predicted** | 0.60 | The `.gl` record dispatcher is a 15-live-arm switch over records, which is record-local. **The risk is COMDAT selection and section base resolution**, where `P_SECTION.md` §7 says c2 has *"a kind switch, a remapper, a base resolver and an alignment chooser"* where the port carries 17 fully-resolved constants — a remapper is where a running state would hide | **Show the base resolver reads a per-compilation running value** (a section index high-water mark, a prior COMDAT's selection). This is the least confident row on the table and it is stated as such |

**Score: three established, five genuinely predicted, and the split is 1 YES /
4 NO among the predictions.** That asymmetry is itself the claim worth checking:
if the floor were a mood rather than a predicate, it would predict every
remaining subsystem shares it. It does not.

### 4.1 What the prediction implies for dispatch, said plainly

* **Four subsystems (`encode`, `coff`, `symbol`, `section`) are predicted
  independently advanceable to completion**, and decision 15's frame applies to
  them unamended. They are also, not coincidentally, the two subsystems that
  already have a measured `ported` numerator plus the two nearest to having one.
* **Six (`regalloc`, `inline`, `dag`, `globregs`, `label`, `eh`) are at or
  predicted to reach the floor.** Five of the six sit on the tuple list; `label`
  sits on the counter.
* **The floor is therefore a majority of the named subsystems, and it is one
  or two objects.** That is the argument for funding the accumulator directly,
  and it is decision 21 §2's argument arrived at from the subsystem side rather
  than from the pipeline side.

---

## 5. Reproducing every claim on this page

| claim | command |
|---|---|
| the 1,195 lines | `wc -l crates/c2-core/src/codegen/regalloc.rs crates/c2-core/src/codegen/regalloc_worklist.rs` |
| regalloc's remaining shape | `awk '/^## 7\./,/^## 8\./' docs/whitebox/ref/P_REGALLOC.md` |
| the inliner's two missing links | `sed -n '698,790p' docs/whitebox/ref/P_INLINE.md` |
| the label accumulator | `sed -n '114,142p' docs/whitebox/ref/P_LABEL.md` |
| EH's out-of-band entries | `sed -n '25,55p' docs/whitebox/ref/P_EH.md` and compare against `SUBSYS.md` §1's EH band |
| the four splice callers and stage S7 | `sed -n '260,346p' docs/whitebox/WB_F0PRICE_FINDINGS.md` |
| the mark-ratio column this page refuses to rank on | `docs/SUBSYS_METRICS.md` §2, §3 |

## 6. What this page does NOT claim

* **It does not say per-subsystem dispatch failed.** It succeeded to a boundary
  on both subsystems in one wave, which is faster than any TU-shaped unit has
  ever moved either of them.
* **It does not re-derive F0.** `w-f0price` did that on 2026-08-27; this page
  cites it and [`REPRICE_2026-08-28.md`](REPRICE_2026-08-28.md) §2.1 corrects
  one addition inside it.
* **It does not construct a `ported` numerator for regalloc.** Decision 20 §2
  and decision 21 §4 both forbid it and it is still not *defined*. Note also
  that `[regalloc] ported = RESIDUE` is **unconditional** on this instrument —
  `SUBSYS_METRICS.md` §4 measures `ported` on two rows only — so the RESIDUE
  wave 16 produced is the instrument declining to answer, not a measurement
  that 1,195 lines bought nothing.
* **It does not re-take `#3534`.** `byte-owned` stays cited.
* **It adds no gate row** (`#3691`) and writes no `crates/` byte.
