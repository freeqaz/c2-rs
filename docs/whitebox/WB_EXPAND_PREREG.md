# WB_EXPAND — PREREG for read R6 (the final-expansion switches)

> **PROVENANCE — DISASSEMBLY-DERIVED.** See [`DISCLOSURE.md`](DISCLOSURE.md).
> Nothing here may enter `crates/` without a `DISCLOSURE.md` row naming the
> address it came from.

**Lane:** `w-read-r6` · **kind:** characterization lane
(`docs/rungs/README.md` § "Lane kinds" 3) · **Fixtures:** none ·
**Census:** +0 · **predicted reach:** 0, registered.
**Board rows:** **#3429**–**#3432** (reserved, `docs/BOARD.md` fifth-wave ledger).

**Subject.** Read R6 of the read plan
(`docs/whitebox/READ_PLAN_2026-08-21.md` §3, funded by the owner 2026-08-23 —
`docs/DECISIONS_2026-08-22.md` decision 7, board **#3423**): **the final-expansion
switches**. Named targets: `FUN_10c0d57e` (3,899 B), `FUN_10c182b4` (426 B), and
the `0x2f4` / `0x2f0` prologue arms reaching `FUN_10bff95c`
(`WB_REGALLOC_FINDINGS.md:282`).

**The deliverable, in the plan's words:** *the pseudo-op expansion table —
which opcodes expand to how many words.*

**Why it is funded, concretely:** `w-s1bc` is landing S1 in `crates/` right now,
and S1's bijection instrument is **known** to go red on framed functions because
the final expansion switch rewrites the prologue pseudo-op in situ into many
words — which is this lane's subject. `ROADMAP_SLICING_2026-08-21.md` §5's
AMENDED block records the demotion to a per-function ratio. This table is what
would let it be more than a ratio.

---

## §0 — Image, and the addresses VERIFIED before this file was written

**Image.** `compilers/X360/16.00.11886.00/c2.dll`, sha256
`c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`, size
1 347 072 — **verified by this lane by `sha256sum` against the pin at
`ref/README.md:21` / `C2_MAP_METHOD.md` §0 before any address below was
touched.** The Ghidra flat export at `~/ghidra-projects/export/c2/` is dated
2026-08-04; quoting its addresses is licensed by the image digest matching.

**The brief's address list, checked against the export's `functions.tsv`
BEFORE this prereg was frozen** — the read brief's own instruction, and the
standing finding that every coordinator-supplied address list in this wave has
needed correction (R4's "mint site" was the wrong function; R5's "189 arms" was
62):

| target | brief says | export says | verdict |
|---|---|---|---|
| `FUN_10c0d57e` | 3,899 B | `10c0d57e 3899 FUN_10c0d57e`, 3 callers, 41 callees | ✅ **exact** |
| `FUN_10c182b4` | 426 B, 18 arms | `10c182b4 426 FUN_10c182b4`, 1 caller, 15 callees | ✅ size exact; arm count not yet checked |
| `FUN_10bff95c` | (no size given) | `10bff95c 327 FUN_10bff95c`, **2 callers**, 11 callees | ✅ exists; 327 B |

**The two callers of `FUN_10bff95c` are 2, and the brief names 2 arms
(`0x2f4`, `0x2f0`).** That is a coherence check the brief did not make and it
passes — but it is consistent with *either* "both arms call it" *or* "one arm
plus one unrelated caller", and P3.1 below is registered on that split.

### §0.1 — ONE THING IN THE BRIEF IS ALREADY CONTRADICTED BY THIS REPO

Registered here, before reading, so it cannot be claimed as a discovery later.
The brief describes `FUN_10c182b4` as one of "the final-expansion switches"
(18 arms). **The repo's own later reading says it is not an expansion pass at
all**:

* `docs/whitebox/labels/W-SELECT-R2.tsv:21` (lane WB-D run 2) — *"the in-place
  expansion switch … jump table `0x10c18460` … only 18 arms. Rewrites pseudo-ops
  and narrowings in situ"*.
* `docs/whitebox/c2_functions.tsv:4499` (lane W-TABLES, **later**) —
  *"**The peephole pass, not an expansion pass**: walks the whole instruction
  list TWICE (`local_c = 2`), 18 arms, every arm takes `&next` so an arm may
  delete and back up. Called once, from `0x10b7dd2c`, gated on `DAT_10c2e2fc`.
  **CORRECTS `WB_SELECT_FINDINGS_R2.md` §4's 'in-place expansion switch'**"*.
* `WB_SELECT_RECONCILED.md:49,162` — *"R2's `FUN_10c182b4` is a machine-level
  peephole phase, not the selection switch"*; both functions real, different
  passes.

So the brief inherited the **superseded** of two repo claims. P4 below scores
which is right. This lane reads `FUN_10c182b4` anyway — a pass that runs at
final expansion time and can *delete* instructions is load-bearing for a
word-count table whether or not it expands — but it is read as a **count
perturbation**, not as an expansion switch.

### §0.2 — prior art this lane must not re-derive

Checked before writing this file. Already established elsewhere, inherited and
cited rather than re-claimed (and excluded from this lane's own numerators
unless independently re-derived):

* `FUN_10c0d57e` is *"a **binary decision tree**, not a jump table, and it spans
  both halves of one opcode space (≤ `0x298` machine, > `0x298` tuple/IR)"*,
  with arms named at `0x0b..0x0d` (addi/addic/addic.), `0x21` (bc), `0x2e`/`0x30`
  (cmpi/cmpli), `0x270` (li) and `0x2f0`/`0x2f4` —
  `WB_SELECT_FINDINGS.md:129,668`, `labels/W-SELECT.tsv:31`.
* `0x10c0dabc` inside `FUN_10c0d57e` calls the `rlandi` expander `FUN_10c0a2e2`
  — `WB_TABLES_FINDINGS.md:289`, `WB_SELECT_RECONCILED.md:705`.
* The `0x26e`/`0x26f` arm reaches `FUN_10c0a2e2` — `WB_SELECT_RECONCILED.md:175`.
* `FUN_10c16a46` (351 B) is *"the entry point of every arm of `0x10c182b4`"* —
  `c2_functions.tsv:4469`.
* Opcode-space anchors from R2: mnemonic table `0x10b1b260` (stride 12), machine
  table `0x10b202b0`, `LAST_MACHINE_OPCODE = 0x294` (`_last` is `0x295`) —
  `ref/P_ENCODE.md` §2.1, `scripts/dump_opcode_tables.py`.

---

## The grading rule

Registered **before** any byte of any target body was read. Tier **PREREG** by
`PREREG.md`'s ladder: committed to git as this branch's first commit, before the
answer existed anywhere in this lane. Every prediction is scored HIT / MISS /
UNGRADED in `WB_EXPAND_FINDINGS.md`; **misses are reported as misses and are not
smoothed**, and a prediction too vague to be falsifiable earns nothing and is
marked UNGRADED rather than counted.

**Numerators carry denominators.** The headline denominator is
`arms-read / <total non-default arms in FUN_10c0d57e>`, and **that total is to be
measured, not assumed** — R5's target was 62 arms where five documents said 189.
A partial numerator is the honest outcome.

---

## P1 — the shape and the population

| # | prediction | grade if |
|---|---|---|
| **P1.1** | `FUN_10c0d57e`'s top-level dispatch is a **binary decision tree on the opcode**, with **no** opcode-indexed jump table. (Re-derivation of `WB_SELECT_FINDINGS.md:668`'s PARTIAL, scored here as a reproduction check on that lane.) | HIT / MISS |
| **P1.2** | The number of distinct opcode values receiving a **non-default** arm is **≤ 40**, against an opcode space of ≥ `0x294`. i.e. expansion is **sparse**: the overwhelming majority of opcodes pass through untouched. | HIT at ≤ 40, MISS above, with the true count either way |
| **P1.3** | **THE HEADLINE.** The arms split cleanly by opcode band: **every arm that increases the instruction count (1→N, N ≥ 2) has opcode > `0x294`** (the pseudo-op band), and **every arm at opcode ≤ `0x294` is count-preserving (1→1) or count-reducing (1→0)**. | HIT if the partition is clean; **MISS on a single counterexample**, and the counterexample is the finding |
| **P1.4** | The most likely falsifier of P1.3 is named in advance: **`0x21` (`bc`)**, because a conditional branch whose target is out of `BD` range is classically expanded to `bc`+`b` (1→2) and `bc` is a machine opcode at `0x21 ≤ 0x294`. If P1.3 misses, this is where. | scored as "falsifier correctly anticipated" / "missed somewhere else" |

**Why P1.3 is the prediction worth registering.** If it holds, the bijection
defect `w-s1bc` is fighting becomes **enumerable by opcode band** — an instrument
can assert 1:1 over the machine band and consult a table only for the pseudo
band. If it fails, S1 needs the full table before it can assert anything, and the
per-function ratio was the right demotion. Either answer is actionable for the
peer lane; that is why it is the headline.

## P2 — the arity classification

| # | prediction | grade if |
|---|---|---|
| **P2.1** | Every non-default arm classifies into exactly one of **{1→0 delete, 1→1 rewrite-in-situ, 1→k fixed fan-out, 1→n data-dependent fan-out}**. In particular **no arm is many→1** — fusing several instructions into one is the *peephole's* job (`FUN_10c182b4`), a different pass. | HIT if all arms classify and none is many→1; MISS names the exception |
| **P2.2** | The **data-dependent** class (1→n, n not statically known) has **≤ 4 members**, and the prologue is one of them. | HIT at ≤ 4 with the prologue present; MISS with the true count |
| **P2.3** | For every **fixed** fan-out arm, **k ≤ 4**. | HIT at max k ≤ 4, MISS with the true max |
| **P2.4** | At least one arm's fan-out is **conditional on an operand value** (e.g. an immediate that does not fit a 16-bit field expanding to a two-instruction materialisation). Registered because a table of pure constants would be a *simpler* answer than this lane expects. | HIT / MISS |

## P3 — the prologue arms (the part `w-s1bc` needs)

| # | prediction | grade if |
|---|---|---|
| **P3.1** | `0x2f4` and `0x2f0` are **two different pseudo-ops sharing one driver**, most likely **prologue and epilogue** (alternative registered: framed vs frameless prologue). They are **not** two encodings of one op. | HIT with the pair identified; MISS if they are one op or an unrelated pair |
| **P3.2** | The prologue's expansion word count is **NOT a constant**. It is a function of at least {frame size, the set of saved callee-saved GPRs/FPRs, whether LR is saved}. | HIT if ≥ 2 inputs are shown to move the count; MISS if it is a constant |
| **P3.3** | **The shape prediction, registered against its alternative.** MSVC's X360 ABI saves callee registers through `bl __savegprlr_N` / `bl __restgprlr_N` glue rather than emitting one `stw` per register, so the common-case framed prologue is **≤ 8 words** and is **not linear in the number of saved registers** across its whole range; any linear-in-`nsaved` term appears only below a threshold. **The alternative, which would falsify this: one store per saved register, count linear in `nsaved` everywhere.** | HIT if the common case is ≤ 8 words and non-linear; MISS if linear everywhere, with the slope |
| **P3.4** | The epilogue expansion's word count is within **±2** of the prologue's for the same function. | HIT / MISS |
| **P3.5** | The count is computable from quantities available **before** the expansion runs (frame size and the saved set are decided by the allocator/frame layout upstream) — i.e. an instrument *could* predict it without simulating the expansion. | HIT if every input is named and upstream; MISS if any input is produced inside the expansion itself |

**P3.5 is the one that decides whether this read helps `w-s1bc` at all.** A rule
whose inputs only exist *inside* the pass cannot be lifted into an instrument
that runs beside it.

## P4 — `FUN_10c182b4`, scored against the brief (§0.1)

| # | prediction | grade if |
|---|---|---|
| **P4.1** | `c2_functions.tsv:4499` (W-TABLES) is right and the brief (inheriting `WB_SELECT_FINDINGS_R2.md` §4) is wrong: `FUN_10c182b4` is a **peephole** pass — one caller `0x10b7dd2c`, gated on `DAT_10c2e2fc`, two walks of the list. | HIT if all three details reproduce; MISS otherwise |
| **P4.2** | **None** of its 18 arms increases the instruction count. Every arm is count-neutral (1→1) or count-reducing (2→1, 1→0). | HIT at 0 expanding arms; **any expanding arm is a finding for S1**, because it means the word count still moves after expansion |
| **P4.3** | Its 18 arms are dispatched through an **opcode-indexed jump table at `0x10c18460`** with a byte index at `0x10c184a8` — i.e. the *opposite* dispatch shape from `FUN_10c0d57e`'s decision tree. | HIT / MISS |

## P5 — the corpus control (scale; fails in the "rule is too narrow" direction)

Over the workload's **real-c2 reference objs**, decode `.text` and parse each
function's leading words against the prologue grammar the read produces.

| # | prediction | grade if |
|---|---|---|
| **P5.1** | **≥ 95 %** of functions in the corpus have a prologue that parses under the read's grammar, with the word count the rule predicts. | HIT at ≥ 95 %, MISS below, denominator reported |
| **P5.2** | The non-parsing residual is **concentrated in a named structural class** (EH funclets, varargs, huge frames, naked/`__declspec` cases) rather than spread uniformly. | HIT if ≥ 60 % of the residual falls in one named class; MISS if diffuse |
| **P5.3** | The corpus is **≥ 200 functions** drawn from **≥ 20 distinct TUs**. Registered as a floor because R2's probe *"was specified against four named failure modes and still could not see a fifth; only a 500-obj population made its control capable of failing"*. | HIT / MISS on the realised size |

## P6 — the grid control (minimal pairs; fails in the "arity is wrong" direction)

Compile minimal-pair fixtures under real c2 where the pair differs by exactly
one construct, and compare **Δ words** against the table's predicted expansion.

| # | prediction | grade if |
|---|---|---|
| **P6.1** | On a grid of minimal pairs, **Δwords equals the table's prediction in ≥ 80 %** of cells, every residual named. | HIT at ≥ 80 %, MISS below |
| **P6.2** | A pair that adds one callee-saved register to the live set moves the prologue+epilogue word count by an amount the rule predicts **exactly** (this is the cell that tests P3.3's non-linearity, and it is designed so that the "one `stw` per register" alternative gives a *different* number). | HIT / MISS — and this cell is the one that can embarrass P3.3 |

**Why both.** `READ_PLAN`/the brief: *grids and corpora fail in opposite
directions.* A grid can hit 100 % on four hand-built shapes and be blind to the
fifth; a corpus can be 99 % green because 99 % of functions are the easy shape.
Neither alone is a control here.

### The probe must be capable of failing — the failure modes it is built against

Named in advance, per the standing requirement:

1. **The count is right and the rule is wrong** — a prologue grammar loose
   enough to parse anything. Mitigation: the grammar must predict the *exact*
   word count and the *exact* opcodes, and P5.1 is scored on both, not on count
   alone.
2. **The corpus is all one shape** — P5.3's ≥ 20-TU / ≥ 200-function floor.
3. **The grid tests only what the read already said** — P6.2 is constructed so
   the two competing hypotheses (glue call vs per-register store) give
   *numerically different* answers, so the cell can come back on the wrong side.
4. **A later pass moves the count** — P4.2 exists precisely to bound this, and
   §7 below says why it cannot be fully bounded from objs.
5. **The `[R]` trap.** `[R]` means *"the instructions were read correctly"*, not
   *"this is what c2 does"* — the `.bss` bump rule was read correctly out of a
   clean function and was wrong about c2 (`ref/README.md:49`,
   `C2_MAP_METHOD.md` §7). Every claim that ends the lane un-probed says `[R]`
   in the findings.

**"Would this go red if the claim were false in the most likely way?"** The most
likely way P3 is false is that the prologue word count depends on something I did
not vary (a compile mode, an EH flag) and I report a constant that is not one.
P5.2's structural-class requirement and a mode axis in the grid are what would
show it — and §7 item 6 records that if the dependence is on a global set by an
earlier pass, **neither control can see it**.

---

## §7 — What these controls are STRUCTURALLY INCAPABLE of catching

Registered before the fact, in the house convention. These are not caveats added
to soften a bad result; each names a class of wrong answer that would come back
**green**.

1. **An obj is post-everything.** Every word I count has been through selection,
   expansion, the peephole, scheduling and the encoder. If the expansion emits a
   word the peephole then deletes, my corpus count is *right about the obj* and
   *wrong about the expansion*. **No obj-derived measurement can separate the two
   passes**, and this lane is not building a live tap.
2. **Word count is a scalar projection.** Two expansion rules with the same arity
   are indistinguishable by count. So P2/P3's **arities** can go green while the
   **content** of each expansion stays `[R]` — a table that says "1→4" and is
   wrong about *which* four words scores a full HIT here.
3. **A grid only reaches arms that source I can write triggers.** An arm for an
   opcode c2 only produces under a mode I do not compile, from an intrinsic, or
   from a front-end construct I did not think of, is invisible — and **I cannot
   distinguish "unreachable" from "not reached by my grid"**.
4. **Dead arms are undetectable.** An arm that is present, correct-looking, and
   never on any path reads exactly like a live one. This is the `.bss` failure
   mode and it is the reason `[R]` exists.
5. **The corpus cannot see refusals.** Functions c2 declined to compile, or that
   the workload does not contain, contribute nothing and are silently absent from
   the denominator.
6. **A hidden upstream input reads as a constant.** If an arm's fan-out depends on
   a global written by an earlier pass, and both controls hold that global fixed,
   I will report a constant and both controls will agree. Only varying the
   upstream pass could show it, and identifying *which* global would itself
   require the read I am doing.
7. **I never observe the pre-expansion instruction list.** Every "1→k" claim is
   read-derived and confirmed only through its *effect* on a final count — the
   same limit R3 hit and reported (*"per-unit attribution needs a live tap on
   `0x10b97de5`, which this lane did not build"*).

---

## What this lane will NOT claim

* **No `crates/` change. Zero bytes.** `w-s1bc` is the only lane in `crates/`
  this wave. If the read implies a `crates/` change it is **reported as a finding
  for a follow-up lane** — R3's precedent, which reported a shipped defect rather
  than fixing it under a docs-only fence.
* **No re-pricing of S1 from this read alone.** The read produces a spec; what
  building an instrument against it costs is a separate number, and #1767's rule
  (a small measurement extrapolated to a full arm count is not an estimate)
  binds here too.
* **No `DISCLOSURE.md` row is owed** unless a constant from this read is adopted
  into `crates/`, which this lane will not do. R1's precedent: that file is the
  ledger of findings *adopted*, and a characterization lane usually owes none.
* **Not a complete expansion semantics.** The deliverable is *how many words*,
  plus whatever content comes free. §7 item 2 is the boundary and it is stated in
  the spec's own voice, not only here.

## Registered decline conditions

Declared now so a bad outcome cannot be re-narrated as a good one:

* **If `FUN_10c0d57e` is not the pass the plan thinks it is** — as happened to
  R4 (the named "mint site" was the wrong function) and R5 (62 arms, not 189) —
  the lane **reports the re-target and does not force a table onto the wrong
  function**.
* **If the arity is not statically determinable for the majority of arms**, the
  table lands **partial, with its denominator**, and the outcome word reflects
  that.

## Registered outcome shape

**`built`** only if all three hold:

1. the non-default arm population of `FUN_10c0d57e` is **enumerated against a
   measured denominator**;
2. the prologue expansion rule is stated as a **formula with every input named**,
   and P3.5 is answered either way;
3. **both** a grid and a corpus control ran, and at least one was demonstrably
   **capable of failing**.

If the arm population cannot be enumerated, or if no control capable of failing
could be built, the outcome word is **FAILED**, in that word — not a compound
headline (`docs/rungs/README.md` § "Lane kinds").
