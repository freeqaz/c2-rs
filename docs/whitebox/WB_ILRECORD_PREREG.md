# WB_ILRECORD — PREREG for read R5 (the IL-record → codegen dispatch)

> **PROVENANCE — DISASSEMBLY-DERIVED.** See [`DISCLOSURE.md`](DISCLOSURE.md).
> Nothing here may enter `crates/` without a `DISCLOSURE.md` row naming the
> address it came from.

**Lane:** `w-read-r5` · **kind:** characterization lane
(`docs/rungs/README.md` § "Lane kinds" 3) · **Fixtures:** none ·
**Census:** +0 · **predicted reach:** 0, registered.

**Subject.** Read **R5** of the funded read-plan
(`docs/whitebox/READ_PLAN_2026-08-21.md` §3, funded by the owner 2026-08-23 —
`docs/DECISIONS_2026-08-22.md` decision 6, board **#3410**): the IL-record →
codegen dispatch `FUN_10bc2d7a` (5,080 B per `ref/FUNCS.tsv:2870`), its jump
table at `0x10bc4152` over opcodes `0x01..0xBD` (`labels/W-IL.tsv:36`), and
the arms behind it. Priced 15–25 days — **by far the largest read this project
has dispatched, and this lane will not finish it.** The registered deliverable
is therefore *the largest evidence-complete increment with an honestly named
boundary*, not 189 rows (§P5).

**Image.** `compilers/X360/16.00.11886.00/c2.dll`, sha256
`c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258` —
**verified by this lane before any address was read**, matching the pin at
`ref/README.md:21`. The Ghidra flat export at `~/ghidra-projects/export/c2/`
is dated 2026-08-04, nineteen days before this tip; per `READ_PLAN` §5.4 its
input digest must match the pinned image before its addresses are quoted, and
**this lane parses arm bytes out of the pinned image directly** rather than
trusting the export's function boundaries (§5.4's `0x10b7f022` trap: "Ghidra
found 4,916 functions" is a statement about Ghidra).

**Prior art, checked before this file was written** (`grep -rn` over `docs/`
`scripts/` `crates/`, plus a separate topic search of `docs/BOARD.md` rows, per
the standing rule that a topic grep cannot see board rows):

- **Not one arm of `FUN_10bc2d7a` has ever been read.** Three independent
  statements: `C2_MAP.md:1012` ("naming each opcode needs reading 189 arms"),
  `READ_PLAN:99` ("ZERO read"), `STEP5_PRICING_2026-08-21.md:139` ("zero arms
  read today"), `ref/P_ENCODE.md:594` ("unstarted"). `ref/P_ILRECORD.md` does
  not exist; four documents forward-reference it.
- **`ref/ADDR.tsv:755` records `0x10bc4152` as `data`, size 4, `unknown`** —
  the jump table has never been parsed. Parsing it corrects an inventory row.
- **`0x10bc4715` is the caller** (`labels/W-IL.tsv:37`): per-function, seeks
  `.sy@+0x58` then `.ex@+0x54`, then codegen. This is the only structural fact
  anyone holds about the seam's context.
- **Board #3359** (`w-ildecode`): *"there is no intermediate between the IL
  token stream and the machine tuple list — by the time any tap can see a
  tuple, selection has already run"*, concluding *"a general op-level IL decode
  is not 'read the records'; it is 'reproduce selection'."* **This lane's
  select-vs-decode finding either confirms or refutes that row**, and it is
  registered here as the row at stake.
- **Board #1591 / #1595** (`wb-reader`): the operand-class table `0x10b25e48`
  (192 entries, 29 classes) and the `0x27` special case, both obj-confirmed.
  **Inherited, not re-derived** — this lane *joins against* the class table and
  does not re-read it. Its arms-read numerator counts only `0x10bc2d7a` arms.
- The ten residue constructs and their masses:
  `ROADMAP_SLICING_2026-08-21.md:162-169` (mass table) and `:277-280`
  (`C1 off-add · C2 intrinsic · C3 bind · C4 load-type · C5 temp · C6 lit-type ·
  C7 compare · C8 bitwise · C9 materialize-64 · C10 virtual-slot`).
- Peer lane `w-read-r4` is reading `FUN_10b55732` (globregs mint/merge) in its
  own worktree. **This lane edits no file R4 creates** — not `ref/P_GLOBREGS.md`,
  not `ref/P_REGALLOC.md`. Any overlap is stated as open and reported.

---

## The grading rule

Registered **before** any byte of `FUN_10bc2d7a`, its jump table, or any arm
body was read. Tier **PREREG** by `PREREG.md`'s ladder: committed to git as the
first commit on `wt-w-read-r5`, before the answer existed anywhere in this
lane. The DAG ordering is the evidence — this commit provably precedes the
commit that teaches the tooling to parse the table at all.

Each prediction is scored HIT / MISS / UNGRADED in `WB_ILRECORD_FINDINGS.md`.
**Misses are reported as misses and are not smoothed**; a prediction vague
enough to be unfalsifiable earns nothing and is marked UNGRADED rather than
counted. Numerators are reported with denominators, and **the denominator is
named before the numerator is known**.

**Everything in the structural block below is arithmetic on committed TSV rows
and prose, not on the target's bytes.** `0xBD − 0x01 + 1 = 189`, which is why
P1.2 exists.

---

## P1 — the structural facts in the brief, re-measured (THE VERIFICATION)

The coordinator's dispatch states, and explicitly disclaims having verified:
entry `0x10bc2d7a`, 5,080 B, table `0x10bc4152`, **189 arms**. Every
coordinator-supplied address list this project has issued has needed
correction by the lane that used it. These are registered as predictions so
the correction is *scored*, not merely noted.

| # | prediction | grade if |
|---|---|---|
| **P1.1** | `0x10bc2d7a` is a real function entry in the pinned image and its extent is **5,080 B ± 0** (ends at `0x10bc4152`, i.e. **the jump table is the first thing after the body** — that is why the two numbers are adjacent). | HIT on exact size; MISS with the true extent |
| **P1.2** | **"189 arms" is an ENTRY count, not a distinct-target count.** The table at `0x10bc4152` holds **189** four-byte entries (opcodes `0x01..0xBD`, indexed `op−1`), and the number of **distinct** arm targets is **strictly fewer than 189** — registered bracket **[100, 170]**. This is R2's 111-entries/79-targets lesson applied in advance. | HIT if entries = 189 **and** distinct ∈ [100,170]; MISS with both true counts. A distinct count of exactly 189 is a MISS and a genuine surprise |
| **P1.3** | Every distinct target lies **inside** `FUN_10bc2d7a`'s own extent — the coherence check the survey ran on the encoder and did not run here. | HIT at 100 % inside; any outlier is a finding, not a rounding error |
| **P1.4** | The table is a **dense** `op−1` index with a range check of the shape `cmp op, 0xBD` / `ja default`, and there is a **default arm** for out-of-range opcodes. Registered against the alternative that opcodes `0x00` and `0xBE..0xBF` are handled by a separate earlier branch. | HIT / MISS on the observed guard |

## P2 — SELECT vs DECODE: the boundary that has never been located

**This is the lane's headline and the reason R5 is priced above every other
read.** `READ_PLAN` §4: *"Per arm: does it select or merely decode — that
boundary is the I1/I2 split the whole 15–45 eng-mo estimate rests on, and it
has never been located."*

**The classification rule is fixed HERE, before any arm is read, so it cannot
be tuned to whatever the arms turn out to look like:**

> An arm is **DECODE** iff its effect is a function of the record's own
> operand bytes alone: it may load from the record, shift, mask, store into
> the node/tuple, and call helpers that are themselves pure operand
> extractors — but it makes **no choice among two or more distinct machine
> outputs**, reads **no global**, and branches on **no type/mode field**.
>
> An arm is **SELECT** iff it chooses among ≥ 2 distinct outputs — a branch on
> a type class, a size, a signedness, a target/ABI global, an optimization
> flag, or a table lookup whose result is an opcode rather than a field value.
>
> An arm is **MIXED** if it decodes operands and then makes exactly one such
> choice, and **DEFER** if its entire body is a call to a subroutine that is
> not read at this lane's depth bound (§P6.4).

| # | prediction | grade if |
|---:|---|---|
| **P2.1** | **The majority of arms read are not pure DECODE.** Registered: **≤ 40 %** of arms in the read subset classify DECODE under the rule above. This is the direct opposite of R2's P3.1 (which predicted ≥ 60/79 encoder arms were *pure field composition*, and hit) — registered against it deliberately, because if this dispatch were as mechanical as the encoder, `C2_MAP.md:1012`'s "mechanical, recipe is exact" would be right. | HIT at ≤ 40 % DECODE; MISS above, with the true fraction |
| **P2.2** | **`C2_MAP.md:1012`'s difficulty claim — "mechanical, recipe is exact" — is WRONG**, and this lane grades it. It is the one prior assertion about this function that is falsifiable, it carries **no board number**, and nobody has ever tested it. Registered: the arms are *not* uniformly mechanical; at least one construct requires reading state established outside the arm. | HIT if ≥ 1 arm's behaviour cannot be stated without a global or an outside-the-arm data structure; MISS if every arm read is a self-contained recipe |
| **P2.3** | **Board #3359 is CONFIRMED, not refuted**: selection genuinely happens at or below this dispatch, so the I1/I2 boundary does **not** fall between `FUN_10bc2d7a` and the encoder. Registered falsifiably: if the arms turn out to build an opcode-neutral tuple and defer all machine-opcode choice to a later pass, #3359 is refuted and this prediction MISSES. | HIT / MISS, with the arms that decide it named |
| **P2.4** | **The boundary is not a clean cut at all** — i.e. the split is not "arms 1..k decode, arms k+1..189 select" but is **interleaved by construct**, so I1 cannot be scoped as a prefix of the opcode space. Registered because the 15–45 eng-mo estimate implicitly assumes a separable I1. | HIT if DECODE and SELECT arms interleave across the opcode range with no contiguous partition; MISS if a partition exists |

## P3 — globals, and what makes an arm context-dependent

`READ_PLAN` §4 asks each construct's section to state **whether the arm reads
any global** — "that is what makes an arm context-dependent."

| # | prediction | grade if |
|---|---|---|
| **P3.1** | **≥ 25 %** of the arms read reference at least one absolute `DAT_10c…`/`0x10c…` address (a global), making them context-dependent in the sense the read plan means. | HIT at ≥ 25 %, MISS below, true fraction reported either way |
| **P3.2** | The globals referenced are **dominated by a small set** — registered: **≤ 12 distinct globals** account for **≥ 80 %** of all global references across the arms read. If true, the context an I1 implementation must model is small and enumerable; if false, I1 carries an unbounded context. | HIT / MISS with the histogram |
| **P3.3** | At least one referenced global is an **optimization-mode or target/ABI flag** already named elsewhere in `docs/whitebox/` (e.g. `0x10c2e310` favor-speed, `DAT_10c2edc4`, `DAT_10c67fc0`, `DAT_10c6fd9c`, `DAT_10c2e978`), i.e. the dispatch shares context with the reader/encoder rather than owning a private one. | HIT / MISS, naming which |

## P4 — the ten residue constructs, and the `0x27` special case

`READ_PLAN` §4: *"Carry the `0x27` special case (`WB_READER_FINDINGS.md:228-234`)
— a spec that omits it is wrong on the largest single construct."* `0x27`
(off-add, C1) is **696,164 bodies, 33.3 % of the residue**
(`ROADMAP_SLICING:162`).

The inherited fact, obj-confirmed as board **#1595**, is in the *type* reader
`FUN_10b3d546` and **not** in this dispatch: `if (opcode == 0x27) node[+6] |=
0x4000` at `0x10b3d581`, and `if (opcode == 0x27) return` at `0x10b3d5b9`,
skipping the entire classification tail — for `0x27` and no other opcode.

| # | prediction | grade if |
|---|---|---|
| **P4.1** | **`0x27`'s arm in `FUN_10bc2d7a` tests the `0x4000` bit** that `0x10b3d581` set — i.e. the reader's special case and the dispatch's special case are **the same mechanism seen at two addresses**, and the flag is the channel between them. | HIT if the arm (or a callee within the depth bound) reads bit `0x4000` of the node's `+6` word; MISS if the two special cases are unrelated, which would itself be the finding |
| **P4.2** | Because `0x27` skips the classification tail, its node carries **no `size_index` at `+0x28`** and **no composed type at `+4`**. Registered consequence: `0x27`'s arm must therefore obtain size/type from somewhere else, or not need it. | HIT / MISS with where it gets it |
| **P4.3** | The ten constructs C1–C10 do **not** map one-arm-per-construct. Registered: the ten constructs touch **more than 10 and fewer than 60** distinct arms in aggregate. | HIT if the count lands in (10, 60); MISS with the true count |
| **P4.4** | **C1 (`0x27`) and C3 (`0x99`/`0x9A`/`0x9B`) do not share an arm**, and C3's three opcodes **do** share one (their operand classes differ — `0x99` is class `1C`, `0x9B` is class `12`, per `WB_READER_FINDINGS.md` §3.1 — so a shared arm would be a positive finding about the tuple, not about the grammar). | HIT / MISS on both halves separately |

## P5 — coverage: the subset, chosen HERE so it cannot be cherry-picked

**A coverage claim that does not name its population is the failure this repo
has hit a dozen times.** This lane will read **N of 189** arms with N < 189.
The selection rule is fixed now:

The read subset is the **union** of three strata, and the findings report
`arms-read / 189` **broken out by stratum**, never pooled into one percentage:

1. **S-A, construct-keyed** — every arm reached by an opcode named in the ten
   residue constructs (C1 `0x27`; C2 `0x40`; C3 `0x99`/`0x9A`/`0x9B`; C4/C5–C10
   as their opcodes are identified). This is the mass-weighted stratum and the
   one the read plan asks for by name.
2. **S-B, frequency-ranked** — the top opcodes by occurrence in the workload's
   captured IL, descending, until the budget is spent. Reported with the
   cumulative mass they cover.
3. **S-C, a uniform random control sample of the remainder**, seed **20260823**,
   drawn with a committed, re-runnable script. **This stratum exists solely so
   the select-vs-decode fractions in P2 have a denominator that was not chosen
   for being interesting.** P2.1's fraction is reported *separately* for S-C, and
   **S-C's fraction is the one that grades P2.1** if the two disagree.

| # | prediction | grade if |
|---|---|---|
| **P5.1** | S-C's DECODE fraction and (S-A ∪ S-B)'s DECODE fraction differ by **≤ 20 percentage points** — i.e. the interesting arms are not structurally unlike the rest. | HIT / MISS; a MISS is a finding that the mass-weighted strata are unrepresentative, which is worth more than a HIT |
| **P5.2** | The ten constructs' arms (S-A) are **individually larger** than the median arm — registered: S-A's median arm body exceeds the all-arms median body size. | HIT / MISS |

**Registered honestly in advance:** if the budget yields **N < 40**, the
outcome word is still `built` provided the boundary in P2 is located on S-C
with a stated confidence — but the findings must lead with the N and the
strata, and must state **what the unread remainder is expected to contain**.

## P6 — the confirmation probe (the `[R]` → `[O]` step)

`READ_PLAN` §5.3 and `ref/README.md:49`: `[R]` means *"the instructions were
read correctly"*, **not** *"this is what c2 does"* — the `.bss` bump rule was
read correctly out of a clean function and was wrong about c2. **The probe
must be capable of failing.**

**The hard problem, named before it is hit:** `READ_PLAN` §3 says *"the tap
cannot see this seam"*, and board #3359 says no tap can observe a tuple before
selection has run. So **there is no direct observation of this dispatch's
output.** Every probe below is therefore an *indirect* consequence test, and
the findings say so in those words rather than claiming `[O]` on the arms
themselves.

**Grids and corpora fail in opposite directions, so both run.**

### P6.1 — the corpus probe (tests the CLASSIFICATION, at scale)

The select-vs-decode classification has a corpus-scale consequence:

> If arm(X) is **DECODE**, the IL opcode X maps to a fixed machine output
> independent of context, so across many TUs the count of X in the IL and the
> count of its predicted machine opcode in the real c2 obj stand in a **fixed
> ratio**. If arm(X) is **SELECT**, they do not.

| # | prediction | grade if |
|---|---|---|
| **P6.1** | Over **≥ 200 TUs** of the workload, opcodes whose arms this lane classified DECODE show a per-TU IL:machine count ratio with coefficient of variation **< 0.05**, and opcodes classified SELECT show CoV **> 0.05**. Reported as a 2×2 confusion table against the static classification. | HIT if the two populations separate; MISS with the confusion table either way. **A MISS here falsifies the classification rule itself**, which is the point |

### P6.2 — the grid probe (tests SPECIFIC ARMS, one variable at a time)

A minimal fixture per targeted construct, varying exactly one thing (type
class, width, signedness, `/Od` vs `/O1`), compiled by **real c2 under wibo**,
with the emitted `.text` compared against the arm's predicted output.

**Named failure modes the grid is built against** — the discipline R2 used,
and R2 still could not see a fifth:

1. **A dead arm** — read correctly, never reached on real input. *Detected by*
   S-B/S-C occurrence counts of zero in the corpus; reported, not hidden.
2. **A SELECT misread as DECODE because nothing was varied.** *Detected by*
   requiring every S-A arm's grid cell to vary at least type class **and**
   width, and at least one cell to vary optimization mode.
3. **A DECODE misread as SELECT because a helper call looked like a choice.**
   *Detected by* the one-level callee read (§P6.4) — and where the callee is
   not read, the arm is classified **DEFER**, never guessed.
4. **Attribution error** — a machine opcode observed in the obj that a *later*
   pass (scheduler, final expansion `FUN_10c0d57e`, regalloc) introduced, not
   this dispatch. *Mitigated by* restricting grid claims to opcodes no read
   pass is documented to synthesize, and **reported as a standing bound** on
   every `[O]`-ish claim in the findings.
5. **Coincidental agreement on a small sample.** *Mitigated by* P6.1's ≥ 200-TU
   population — R2's control only became capable of failing at 500 objs.

| # | prediction | grade if |
|---|---|---|
| **P6.2** | On a grid meeting (1)–(5), **≥ 80 %** of targeted cells produce the machine opcode the arm predicts, with every residual named and attributed. | HIT at ≥ 80 %, MISS below |
| **P6.3** | The residuals concentrate in **SELECT** arms rather than **DECODE** arms — the classification's own prediction about where a static read goes wrong. | HIT / MISS |

### P6.4 — the depth bound, registered

Arms call helpers. This lane reads callees to **depth 1** (the arm's direct
callees) and no further. An arm whose behaviour is determined below depth 1 is
classified **DEFER** and **counted in the denominator as unresolved**, never
silently classified. The count of DEFER arms is a headline number in the
findings, because it is the honest measure of how much of this seam a 15–25
day price actually buys.

---

## What this lane's controls are STRUCTURALLY INCAPABLE of catching

Registered in advance, because R2's prereg named four failure modes and its
probe still could not see a fifth.

1. **No tap can observe this seam's output.** Every probe is a consequence
   test through the whole downstream pipeline. A wrong arm reading that
   happens to produce the same final `.text` — because a later pass normalizes
   it — is **invisible to every control in this file**. This is the fifth
   failure mode by construction, and unlike R2's it is known in advance and
   still not fixable within the lane.
2. **Right bits, wrong operand.** `P_ENCODE.md` §9.6's bound applies here
   verbatim: a rule that builds the right tuple from the wrong field of the
   record passes every count-based and opcode-based check in P6.
3. **Path-dependence.** An arm guarded by a condition the fixtures never vary
   is read as if unconditional. The grid varies type, width and mode; it does
   **not** vary target/ABI, PGO, LTCG, or `/GL`, and cannot speak to arms those
   flags gate.
4. **The DECODE/SELECT rule is a judgement on instruction text.** It is
   reproducible (the rule is fixed above, and every classification is
   published per-arm so a reader can re-grade it) but it is **not measured**.
   P6.1 is the only thing that tests it against reality, and it tests the
   *population*, not the individual arm.
5. **Static reachability ≠ semantic reachability.** "All 189 entries point
   inside the body" says nothing about which arms real IL reaches.
6. **The unread remainder.** Whatever N is, the 189 − N unread arms are
   characterized by *expectation*, not evidence. The findings state what they
   are expected to contain and label it as expectation.

---

## What this lane will NOT claim

- **No `crates/` change of any kind.** Docs-only, `Census: +0`, predicted
  reach 0. If the read implies a `crates/` change, it is **reported as a
  finding for a follow-up lane**, never edited under a docs-only fence.
- **No re-pricing of I1.** R2's discipline is the standard: it delivered a
  complete encoder spec and then listed six reasons it was not buildable and
  **explicitly declined to lower I2's estimate**. If arms turn out cheaper or
  dearer than the 15–45 eng-mo estimate assumed, that is reported as a
  *finding*; the re-pricing is not this lane's and is not the owner's decision
  yet. #1767's rule (a 3-cell measurement extrapolated to 111 arms is not an
  estimate) binds here at 189.
- **No approval of any row of 4a/4b** (board #3410's own warning). A read
  produces a spec, not an implementation.
- **No adoption without a `DISCLOSURE.md` row.** A characterization lane
  usually owes none, and this one expects to owe none.
- **No edit to any file `w-read-r4` is creating**, and no re-derivation of the
  operand-class table (#1591) or the `0x27` type-reader case (#1595) — both
  inherited with citation, both excluded from this lane's numerator.

## When this lane DECLINES

Registered so the decision is not made after seeing the answer:

- If the table at `0x10bc4152` does not parse as a jump table at all (P1.2
  fails structurally, not numerically), the lane reports **FAILED** in that
  word and publishes what the bytes actually are.
- If the arms prove to be uniformly thin trampolines into one shared
  subroutine — i.e. the real dispatch is somewhere else — the lane reports the
  redirection as its finding and **declines** to manufacture 189 rows about
  trampolines.
- If P6.1's corpus probe cannot be built because the IL side cannot be
  tokenized at the needed scale, that is reported as a missing control, the
  affected claims stay `[R]`, and **the outcome word is still graded against
  whether the boundary was located** — not against the probe's existence.

## Registered outcome shape

`built` **only if** (a) P1's structural facts are re-measured and any
correction to the brief is published, **and** (b) the select-vs-decode
boundary is located on a stated, named population with at least one control
that could have gone red. If the boundary is not located, or if the table does
not parse, the outcome word is **FAILED**, in those words — not a compound
headline.
