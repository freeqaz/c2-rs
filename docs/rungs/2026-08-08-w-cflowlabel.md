# w-cflowlabel — the second-largest blocker was already priced, the price was never re-derived, and the instrument that produced it is a DIFFERENT PREDICATE from the port's class in both directions

    Tag:       w-cflowlabel
    Slug:      w-cflowlabel
    Date:      2026-08-08
    Fixtures:  none — this rung ships an instrument and declines a class
    Census:    711,486 / 2,463,443 unchanged (28.88 %), **+0** — this lane adds
               no arm to any acceptance path. TU match **11 → 11**, mismatch
               **0 → 0**.
    Record:    this file; prereg `work/w-cflowlabel/PREREG.md`, written before
               the first line under `crates/`.
    Lane:      w-cflowlabel, worktree branch `wt-w-cflowlabel` off master
               **`f49fe5e1`**.

---

## §0 The one-paragraph answer

`body-cflow-label` is **14,990 of 130,575 blocked emitted functions (11.5 %)**
and rank 2. It is **category (3) — real but far smaller than its size** — and
that was already known: lane `WCF` re-priced this exact row on 2026-07-31, from
48,102 bodies to **718 blocked on control flow alone** and from 14,947 emitted to
**10**. What this lane found is that **the re-price has never been re-derived,
the number it produced has not moved in eight days, and the predicate that
produces it is not the port's class** — it is a hand-written mirror that the port
has since outgrown by **72.9 %** *and* that admits **83,776** bodies the port
refuses. So the standing price of the block-IR restructure is not a bound in
either direction; it is a proxy whose error was assumed and is now measured.

**Nothing was widened and no TU converted, by design.** What shipped is the
control that would have caught this, five `gap-metric` keys and six portable
tests. **The rung DECLINES at four independent refusals**, registered in the
prereg before the numbers came in.

---

## §1 Results

| quantity | base `f49fe5e1` | tip | Δ |
|---|---:|---:|---|
| TU match / mismatch / codegen-gap / vocab-gap / capture-fail | 11 / 0 / 0 / 860 / 7 | **11 / 0 / 0 / 860 / 7** | **0** |
| `gap-metric` lines | 139 | 144 | **139 of 139 byte-identical, +5 NEW, 0 changed** |
| `fn_blockers` | 671 keys, 1,751,957 | 671 keys, 1,751,957 | **0 keys moved** |
| `emit_blockers` | 648 keys, 130,575 | 648 keys, 130,575 | **0 keys moved** |
| `peerkeys.py` | — | — | **0 families vanished** |
| `git grep -c` of the test attribute | 1,257 | **1,263** | **+6** — and it read 1,264 for one draft; see below |
| workspace tests | 1,159 pass / 0 fail / 36 targets | **1,165 pass / 0 fail / 36 targets** | +6, targets +0 |
| `factor-a…e`, `frontier`, `b-and-c`, FBM, PROGRESS MASS | — | — | **every digit unchanged** |

**Why the `1,264` is worth a paragraph.** `git grep -c` counts the attribute in
**every tracked file**, and an earlier draft of *this document* quoted it
literally — so the rung doc counted itself as a test and the delta read `+7`
against a real `+6`. Per file: `crates/c2-harness/src/gap/tests.rs` **62 → 68**,
and nothing else under `crates/` moved (`git diff f49fe5e1 -- crates/`: **6
added, 0 removed**). Recorded rather than quietly fixed, because a metric a
lane's own write-up can move is trap 5 at its smallest, and it took a per-file
diff to see — the subtraction `1,264 − 1,257 = 7` looked like an ordinary count
and was off by one lane-authored line.

The five new keys:

```text
gap-metric cflow-emitted-branchy                    38227
gap-metric cflow-emitted-modeled                        9
gap-metric cflow-residue-inclass-modeled           192495
gap-metric cflow-residue-inclass-offclass          518991
gap-metric cflow-residue-straight-modeled-blocked   83776
```

---

## §2 Which of the five categories, and the evidence

**(3), on top of a (1) one level up.** Both halves, cheapest first, as the brief
orders them.

### §2.1 Not (2) or (5) — settled by grep, before any measurement

`body-cflow-label` is **not a codegen key**. It is emitted by `Block::feature()`
(`crates/c2-il/src/func/body/mod.rs:1491`) when a *straight-line* production
meets byte `0x29`, a label definition. It is the 1:1 rename of `body-0x29`. Four
shipped productions already eat `0x29` — `cond_tail`, `early_return`,
`guarded_seq`, `ptr_walk_loop` — so it is not a misfiled production, and the key
names exactly what the byte is. **The brief's framing — "the label machinery is
control flow's entry point, start from `codegen/labels.rs`" — points at the
wrong crate.** `labels.rs` is the *emitter's* label map and it is not what this
row is about; the row is an IL-decode refusal in `c2-il`.

### §2.2 It is (3), and the deflate is the largest on the record

| axis | the row | blocked on control flow ALONE | deflate |
|---|---:|---:|---:|
| bodies, 2026-07-31 (`WCF`) | 48,102 | 718 | 67× |
| bodies, this lane | 48,102 → n/a¹ | **718** | — |
| **emitted, 2026-07-31** (`WCF`) | 14,947 | **10** | 1,495× |
| **emitted, this lane** | **14,990** | **9** | **1,665×** |

¹ the bodies row was renamed and re-bucketed since; the counterfactual is the
comparable number and it is byte-identical.

**718 = 713 `cflow-if-1+expr-modeled` + 5 `cflow-switch+expr-modeled`, which is
`WCF`'s 713 and 5 to the function**, eight days and a per-function census of
491,013 → 711,486 (+45 %) later. On the column that ranks it went **10 → 9** —
*down*. The rung's realizable worth has not grown; by the standing instrument it
shrank by one function.

### §2.3 …and the instrument that says so is (1) — a private limit, at the
### counterfactual rather than at a recognizer

`CfResidue::Modeled`
(`crates/c2-il/src/func/body/shapes/control_flow.rs:131`) is what makes a
`+expr-modeled` row. Its vocabulary is **hand-written**: `is_int4_type` /
`is_ptr_to_4`, the `+ - *` chain, `26`, the call quadruple. Its own doc says it
mirrors the accepting parser deliberately, so that the counterfactual cannot
over-claim. **Nothing ever checked the mirror against the parser.**

The check is free and the answer is on the one population where it is knowable
without lowering anything — the bodies the port **accepts**:

```text
in-class bodies                                711,486   (== fn_in_class, exactly)
  …the residue calls `+expr-modeled`           192,495   (27.06 %)
  …the residue calls off-class                 518,991   (72.94 %)
```

**The port accepts 711,486 bodies and its own counterfactual predicate
recognises 192,495 of them.** `0x05`/`0x06` (`/`, `%`) call `off_class()` on the
same line as a comment describing `div_mod_leaf`, which has shipped since and is
graded 185 of 185.

**And the error is TWO-SIDED, which is the part that changes the wording rather
than the magnitude.** Restrict to `cflow-straight`, where "blocked on control
flow alone" is vacuous:

```text
cflow-straight+expr-modeled, and the port REFUSES it     83,776
```

So `Modeled` **neither contains nor is contained in** the port's class. It is
not conservative; it is a *different predicate*. **"718 is a lower bound" is
therefore the wrong sentence** — the true statement is that 718 is an
unvalidated proxy with a measured two-sided error, and no arithmetic recovers
the number it is a proxy for.

This is `w-bd`'s `chain_skip_form` finding in a second file: one enum arm
carrying two meanings that nothing distinguishes. Here `Expression` means both
*"this body needs expression work"* and *"this body uses an expression the
residue's frozen table does not list"*, and the second is 72.94 % of the port's
own class.

### §2.4 What the honest bracket is, and why it is not a price

The residue is the lower side. The upper side is the **necessary** condition: a
body blocked on control flow alone must have a control-flow key as its **first**
blocker, because the straight-line parser would have consumed everything before
the branch.

| population | lower (residue) | upper (first blocker is a control-flow key) | spread |
|---|---:|---:|---:|
| bodies, branching, blocked | **718** | **115,364** | **161×** |
| **emitted, blocked** | **9** | **22,342** | **2,482×** |
| emitted a block IR must SERVE | — | 38,227 | — |

**The instrument cannot price this rung to better than three orders of
magnitude.** That is the finding, and it is worth more than either endpoint.
The 22,342 upper is `body-cflow-label` 14,990 + `expr-brfalse` 3,105 +
`expr-brtrue` 1,939 + `return-scope-close-cflow-label` 1,814 + `expr-jump` 303 +
`call-ref-cflow-jump` 103 + `expr-label` 51 + `expr-switch-dispatch` 36 +
`call-postop-cflow-brfalse` 1 = **17.1 % of the 130,575**.

**A calibration, labelled as one and not used as a result**: if the staleness
were uniform, the corrected bodies figure would be 718 / 0.2706 ≈ **2,653**
(3.70×). Uniformity is not measured and branching bodies are structurally
unlike straight ones — `WCF` measured that gap at 457× and it reads 81× today.
The number is written down so nobody has to re-derive it, **not** as an answer.

---

## §3 The rung DECLINES — four independent refusals, ceiling taken neat

Registered in `PREREG.md` §3 before any of §2's numbers existed.

| # | refusal | independent? |
|---|---|---|
| R1 | **No loop representation.** `Selected` has no variant with a back edge; no IL production accepts a general loop. `ptr_walk_loop` is a twenty-word transcription of one function class, `/O1` only, and `PORT_CFG_CLASSES` deliberately still omits `cflow-loop`. | yes |
| R2 | **The compiler-label counter.** c2 charges +1..+4 for a back edge over 17 seed-free cells; `coff::plan_labels` charges 0. `labels.rs` invariant 4 refuses every backward branch and `IlFunction::label_slots` returns `None` for the loop shape. **These three sites are ONE refusal, not three** — one variable at three thresholds, which is the prereg's own test. | yes, as one |
| R3 | **Register allocation across a back edge**, behind the frame/liveness spine. | yes |
| R4 | **Each frontier TU's own price** — `w-front2` and `w-heap` measured min **5**, second-cheapest **≥ 7**, and the standing ≥ 4 decline clause fires on every one. | yes |

**14,990 is taken neat.** No discount to a "realizable" figure is applied,
because the instrument that would compute one is §2.3.

### §3.1 …and `labels.rs` is not the place the relaxation belongs

`codegen/labels.rs` is **untouched**. Its own module header already states where
a loop rung's relaxation goes — `IlBundle::functions`' TU-level gate, which can
see whether a later function in the TU is framed, and which a per-body
`LabelMap` structurally cannot. This lane did not move that boundary and did not
weaken invariant 4.

**`crates/c2-core/src/codegen/coff.rs` was never opened.** Hard stop honoured;
no part of this rung required it.

---

## §4 Scoring the prereg

| # | prediction | outcome | verdict |
|---|---|---|---|
| P1 | in-class bodies the residue calls off-class: **55–70 %** | **72.94 %** | **MISS**, above the range |
| P2 | corrected counterfactual **2×–6×** of 718 | **UNSETTLEABLE** — the bracket is [718, 115,364], 161× wide | **the prediction was malformed**, and that is the finding |
| P3 | corrected counterfactual **< 2 %** of the 14,990 row | 9 = **0.06 %** at the lower end; 22,342 = **149 %** at the upper | **settleable only at one end** |
| P4 | **0 TUs converted at any width** | TU match **11 → 11** | **HIT** |
| P5 | ≤ 7 frontier TUs made CFG-**reachable** by a loop class, **0 conversions** | **7** of 16 frontier TUs are blocked on `cflow-loop`, 5 on it alone; conversions **0** | **HIT** |
| P6 | `mismatch` 0 at tip | **0** | **HIT** |

**P1 missed in exactly the direction §4 of the prereg named** — "I expect to
under-estimate the staleness" — which is the one honest thing to say for it: the
direction was registered, the magnitude was not.

**P2 is the useful failure.** It was written as a point range on the assumption
that a corrected counterfactual is a *number*. It is not; with a two-sided error
of 518,991 and 83,776 it is an interval, and the interval is 161× wide. A rung
that had "widened the residue and reported the new figure" would have published
a second single number with the same unexamined status as the first. **The
prereg's own framing was the thing that needed correcting, and no amount of
being right about P4 would have shown that.**

---

## §5 What shipped

Five `gap-metric` keys, one shared predicate, six portable tests, **zero**
changes to any acceptance path, any emitter, or any key an existing table names.

| file | what |
|---|---|
| `crates/c2-harness/src/gap/classify.rs` | `cflow_needs_block_ir` — the branchy predicate, **named** so the scan and its test cannot hold two opinions of it. Three-valued in effect: `cf-…` is *not known*, `cflow-straight*` is the control, everything else is true. |
| `crates/c2-harness/src/gap/scan.rs` | the `\|IN-CLASS` / `\|BLOCKED` population cross on the control-flow axis (the EH axis's existing spelling); the emitted counterfactual counters. |
| `crates/c2-harness/src/gap/report.rs` | `cflow_residue_control`, `cflow_residue_overclaim`, `cflow_emitted_counterfactual`. |
| `crates/c2-harness/src/gap/factors.rs` | the five keys, emitted together. |
| `crates/c2-harness/src/cli/gap.rs` | the COUNTERFACTUAL / RESIDUE CONTROL block, and the exclusion that keeps the existing cross list byte-identical. |

### §5.1 Widen, never narrow — asserted, not asserted-to

* `fn_cflow`'s **class** histogram is built by `!k.contains('|')`, so the new
  keys cannot enter it. `population_cross_does_not_enter_the_class_histogram`
  pins that, and pins the accounting identity `IN-CLASS + BLOCKED == class`.
* The existing `<class>|<key>` cross **render** excludes the two new suffixes
  explicitly. Without that, `cflow-loop|BLOCKED` (98,386) would have entered the
  top-12 list and displaced a real row.
* `emit_blockers`, `fn_blockers`: **0 of 648 and 0 of 671 keys moved**, sums
  identical to the unit. `peerkeys.py`: **0 families vanished**.

### §5.2 The control that stays green under the mutation used as evidence

`cflow-straight+expr-modeled` is **276,271** bodies — 385× the whole
counterfactual — and it is **not** part of it: a straight-line body has no
control flow to lower. An implementation testing `ends_with("+expr-modeled")`
without excluding `cflow-straight` would size this rung at 385× its worth.

**Mutation-tested, not argued.** Rewriting `cflow_needs_block_ir`'s body to
`class.ends_with("+expr-modeled")` fails
`emitted_counterfactual_excludes_straight_line_bodies` and **160 of 161 other
tests stay green** — the control is the 160.

---

## §6 Found and not taken

1. **Widen `CfResidue`'s vocabulary to today's class.** The correct repair, and
   `crates/c2-il/src/func/body/shapes/` is lane **w-op27**'s file — not mine to
   edit. Filed as board **#1345** with the 518,991 / 83,776 attached. **It should
   not be shipped as a bare widening**: §4's P2 says the output would be another
   single number of unexamined status. What it owes is the pair.
2. **`cflow-if-n` is the bigger frontier lever than `cflow-loop`, and it has no
   back edge.** Of the 16 frontier TUs, **9** need `cflow-if-n` and **6 need it
   and nothing else** (`negate_test`, `osfinfo`, `undname`, `vsnprnc`,
   `vswprnc`, `xlrcimpl`); 7 need `cflow-loop`, 5 of them alone. `if-n` is
   forward-only, so `labels.rs`'s invariant 4 and R2 above **do not fire on it
   at all**. That is a strictly cheaper structure than the row this lane was
   sent to open — **and it still converts nothing**, because R4 prices every one
   of those six at ≥ 7 independent refusals. Board **#1346**.
3. **`WCF`'s 457× "branchy expressions are harder" reads 81× today**
   (13.80 % of straight bodies `Modeled` against 0.17 % of branching ones).
   Both figures come from the predicate §2.3 indicts, on both sides, so the
   *ratio* is better founded than either rate — but it is 5.6× smaller than the
   published one and the published one is quoted in ROADMAP §8.6. Board
   **#1347**.
4. **The 22,342 upper bracket is not screened for resync.** `expr-label` (51)
   and `call-postop-cflow-brfalse` (1) are first blockers naming a control-flow
   byte inside a production that is nowhere near one, which is the shape of the
   `:eof` mis-rendering. The bracket is stated with them in because removing
   them without a screen would be selection on the outcome.

---

## §7 Gate evidence

Both ends measured in-tree with `--cwd` pointed at the dc3 tree by absolute
path: `../dc3-decomp` **does not resolve from a worktree** and the scan reports
`capture-fail 878` rather than failing, which is `status.sh`'s documented
`C2RS_DC3` trap wearing a different name on `c2rs gap`. The first base scan of
this lane hit it.

| lane | result |
|---|---|
| `cargo test --workspace --release` | **1,165 passed, 0 failed, 36 targets** (1,159 / 36 at base) |
| `scripts/gate.sh --jobs 16 --require-graded` | see §7.1 |
| 878-TU scan, both ends | `match 11 · mismatch 0 · codegen-gap 0 · vocab-gap 860 · capture-fail 7`, identical |
| `gap-metric` | 139 of 139 byte-identical, +5 new |

**Both timings were taken on a CONTENDED box.** The gate's own preflight named a
concurrent lane's live gate (`/tmp/c2rs-gate-2772361`), and a third lane was
gating in the same window. No timing in this document is a performance claim.
