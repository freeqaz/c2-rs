# Whitebox leverage — read before probe — 2026-08-21

Written on the owner's direction, the same day as the goal re-ranking
(`GOAL_DECISION_2026-08-21.md` § "AMENDED"). The strategic observation being
acted on, from the roadmap round:

> Most historical lane cost is **black-box inference** — probe grids and
> fitted-parameter searches recovering facts the binary states plainly.
> A single alignment nibble cost a lane; `dag.c`'s lowering order took two;
> register allocation is priced at **13 raw / 65 calibrated lanes** if
> recovered by probing. Meanwhile the encoder-table *read* took a lane two
> days and produced 32-of-32-bit reproduction on its population (#3358 —
> fenced at 3 functions / 9 words, all frameless leaf; the fence is the
> lane's own and quoting the ratio without it repeats #1459's error).

Whitebox reading was already authorized (`CLAUDE.md`, owner, 2026-08-17) and
already promoted to product (goal (1)). What was missing is the **doctrine
that makes it the default**, and a **grounded read-plan** so the next
characterization lanes attack the highest-leverage targets instead of the
nearest ones. Both are below.

---

## 1. The doctrine: read before probe

**Standing rule, effective 2026-08-21** (also entered as
`ROADMAP_SLICING_2026-08-21.md` §6 rule 6):

> Before any lane budgets a probe grid or a fitted-parameter search, it must
> price the binary read that would answer the same question — locate the
> function, read it, confirm with a *small* probe — and prefer the read
> unless the read is measurably more expensive.

What this changes concretely:

- **Item F may no longer be quoted at 13/65 lanes as if that were the cost of
  the facts.** That is the cost of *probing* the facts. The cost of *reading*
  the allocator and scheduler out of the image has never been priced, and §3
  prices it.
- **A fitted constant is a debt, not an answer.** `codegen::alloc`'s clauses
  are the repo's own *"fitted stand-in"* for c2's unread worklist order, with
  clause 2 already refuted on 7 of 56 fresh-holdout cells. Every fitted
  constant in `crates/` should carry a pointer to the read that would replace
  it (§3's table is the index).
- **Probes don't disappear — they change role.** The probe stops being the
  *discovery* instrument and becomes the *confirmation* instrument: read the
  mechanism, predict the observable, confirm with the smallest grid that
  could refute the reading. `w-ildecode`'s method (#3357–#3359: read a table,
  then grade the reading against live tap output, labelling every rule
  DERIVED or TRANSCRIBED) is the template.
- **The correctness rule is untouched.** Reading tells us what to build;
  real c2's output bytes remain the sole judge of what we built.

## 2. Why reading is structurally cheaper here

Black-box recovery of a compiler-internal choice is exponential-ish in the
choice's interaction depth: each internal decision is observable only through
downstream byte consequences, so a probe grid must vary inputs until the
decision's signature separates from every other decision's — and fitted
searches (52,416 configurations for `alloc`, 13,104 residual for `schedule`)
are the price of that separation *on one population*, with no warranty on the
next (clause 2's holdout refutation is that warranty failing).

Reading is linear in the code that implements the choice: the worklist order
is a loop somewhere, and the loop says what it does. The two-days-vs-two-lanes
asymmetry measured on interface 2 (#3358: "this arrow is a 30-line function;
the other one is the whole compiler") is the honest bound in both directions —
reads are cheap where the mechanism is small, and reading does not make the
*whole compiler* small. The read-plan therefore ranks targets by
(black-box cost replaced) / (read price), not by size.

## 3. The read-plan

*Grounded by the 2026-08-21 survey (coordinator-verified citations); prices
assume the existing Ghidra project and the pinned `c2.dll` 16.00.11886.00.*

**Landed: `docs/whitebox/READ_PLAN_2026-08-21.md`** — the full survey:
inventory with denominators (10.8% of the image's 4,917 functions covered
at all; `CEILING` phase 1 the one UNSERVED phase of seven), the fitted-
constant index (§2 there), nine ranked reads (§3), spec-shapes for the top
three (§4), and five live caveats (§5). The short version:

| | read | days | replaces |
|---|---|---:|---|
| R1 | `DAT_10c400d4`'s scope (2 addresses) | 0.5 | a 1–2 wk black-box settle **and** the standing explanation for ten refuted allocation keys |
| R2 | the encoder: 2 tables + **79 distinct arms** ✅ (not 111 — coordinator re-measured from the pinned image) | 2–4 | **I2**, priced 1.5–4.5 eng-mo raw black-box; ~~also unlocks mismatch anatomy (§5 below)~~ **(that clause struck 2026-08-22 — mismatch anatomy is shipped and does not depend on these tables; R2 stands on I2 alone. §5 banner)** |
| R3 | the label charge: 31+132 enumerable call sites, closed by construction | 2–4 | the fitted `LABEL_SEED_GAP = 9` / `/Gy +3`, and §2.3(b)'s "not derivable" premise |
| R4 | `FUN_10b55732` — globregs promotion (item F1) | 3–5 | F1's 2-raw/10-calibrated lanes; may explain the 52,416-config null |
| R5 | `FUN_10bc2d7a` — ~~the 189-arm IL→tuple dispatch~~ **61 REAL ARMS over 95 opcodes, plus one refusal over 94 — "189" is the OPCODE count; the 15–25 d in the next cell is priced against the struck number. [`WB_ILARMS_MAP.md`](whitebox/WB_ILARMS_MAP.md) §1, `DECISIONS_2026-08-22.md` decision 13** | 15–25 | **I1** (1.5–4.5 eng-mo raw) + the shared input to all ten Phase-1 slices |
| R6–R9 | expansion switches · scheduler confirm · block order · `0x4F` | 13–23 | S1's instrument, item F0, `CEILING` phase 1, the last transcribed width |

**Dispatch order R1 → R2 → R3** (≈5–9 days, three `DISCLOSURE.md` rows,
each refuting or confirming a roadmap premise); R2 proves the arm-reading
method on 79 bounded bodies before R5 spends 15–25 days on 189. The
headline: **the two interfaces the 15–45 eng-mo estimate rests on (I1, I2)
both have a named, sized, mechanical read** — R5 and R2 — and the sum of
all nine reads is ≈6–10 engineer-weeks, against black-box prices for the
same facts that run to lane-years.

## 4. The port as instrument — the decision surface

The goal amendment names two consumers that change *how* general layers
should be built, not just whether:

1. **Training AI models to reverse the compiler** (matching-pretext
   generation). The port can emit what the binary cannot be made to emit:
   aligned `(IL, per-stage internal state, output bytes)` triples in
   unlimited volume, with every stage inspectable. The stage tap
   (`c2host/stagetap.c`, 8 sites including `after0`) already does this for
   real c2 on a per-capture basis; the port does it at ~10⁶ obj/s.
2. **A better permuter.** When candidate code is close but wrong because of
   opaque internal state, the fix is a search over the decisions that state
   controls. The repo has already run ad-hoc permuters — the 52,416- and
   13,104-configuration searches *are* permuter runs whose search space had
   to be reverse-engineered first. A port whose decision points are **named,
   enumerable parameters** (allocation order, scheduling tie-breaks, label
   counters) is a permuter whose search space is free.

**Design rule for S1 and every general layer after it** (also
`ROADMAP_SLICING` §6 rule 7): arbitrary choices ship as an explicit decision
surface, not baked constants. The default configuration must reproduce c2
byte-exactly (that is the judge); every non-default configuration is a
legal instrument state. A baked constant serves parity only; a named decision
point serves parity, the permuter, and the training pipeline at the same
correctness cost.

## 5. The judge stays binary; the scoreboard grows gradients

> ### ⚠ CORRECTED 2026-08-22 — **PART (c) OF THIS SECTION IS WRONG. THE COORDINATOR WROTE IT, AND IT PROPOSED BUILDING AN INSTRUMENT THAT IS ALREADY SHIPPED AND WHOSE ANSWER REFUTES THE TABLE IT PROPOSED.**
> *Found by the coordinator the following day and verified independently against
> the tree by lane `w-readdocs`, which was mid-sweep propagating this document
> when the correction arrived. Board **#3369**. Parts (a) and (b) below are
> unaffected and stand exactly as written.*
>
> Three errors, each checked against the tree at this lane's base (`a0d3bb58b`):
>
> 1. **"The missing instrument is mismatch anatomy" is FALSE — it shipped on
>    2026-08-06.** `crates/c2-harness/src/gap/fndiff.rs` (1,369 lines) is the
>    decoder, the LCS word alignment, the insert/delete pairing into
>    substitutions, the per-substitution decoded-field classification, the
>    `same_multiset` (pure-reordering) bit and the relocation-site awareness.
>    It is **wired and unconditional**: `gap/fnbytes.rs:2569` calls
>    `fndiff::signature` on every `fnbyte-differs` body, `gap/render.rs:1295+`
>    prints the `DIFF STRUCTURE` block — cluster table, substituted-words-by-
>    field-class, first-divergence histogram, relocation-aware counts — on
>    **every** `c2rs gap` scan. `--fnbyte-diff-jsonl` and
>    `scripts/fndiff_report.py` are the per-symbol opt-in on top. It has its own
>    page, [`DIFF_STRUCTURE.md`](DIFF_STRUCTURE.md), and it already obeys the
>    separation rule (c) proposes for it, in that page's own words: *"Nothing
>    here reaches a numerator, appears in an accept/refuse path, or grades the
>    port."* **The 1–2 wk price is withdrawn**; refreshing the numbers is a scan.
>
> 2. **The diff-class table below is REFUTED by that instrument's own output,
>    and its refutation is the more useful fact.** Measured at tree `0c8a185`
>    over the then-3,195 differing bodies (`DIFF_STRUCTURE.md` §1–§2): **0
>    bodies are a pure instruction reordering**, so the *permutation →
>    scheduling* row is an **empty class**; of 5,189 substituted words **5,173
>    (99.7 %) differ in their OPCODE**, 2 in a register field, 2 in an
>    immediate, 12 in reg+disp and **0 fail to decode** — so the *field-only →
>    allocation* and *immediate-only → layout* rows are 2 words and 2 words.
>    **94.3 % of bodies are already wrong at word 0.** The population is **one
>    mechanism** — c2 inlined a callee where the port emitted a call — not an
>    allocator, a scheduler, an immediate or a displacement. A table that
>    apportions wrong bytes across four pipeline stages describes a population
>    this workload does not contain.
>
> 3. **"The tables are already read and dumped" is FALSE.**
>    `docs/whitebox/scripts/dump_opcode_tables.py` reads `0x10B1B260`
>    (mnemonic) and `0x10B202B0` (machine) and **nothing else** — verified, the
>    file contains no other VA. The base-word `0x10c3a578` and encode-form
>    `0x10c39b18` tables are **not** dumped; dumping them is read-plan **R2**'s
>    job (a), which is exactly what `READ_PLAN_2026-08-21.md` §3 R2 says.
>    Independently: **mismatch anatomy never needed them.** `fndiff.rs` decodes
>    PPC directly under `CODEGEN_W6_COMPARE.md`'s re-encode-or-refuse rule
>    (`Decoded::reencode`, `undecoded` when a form's field partition does not
>    reproduce its word), which is why its undecoded count is 0 without any
>    table read out of c2. So R2's "also unlocks mismatch anatomy" is a
>    **spurious second justification**; R2 stands on I2 alone, which is the row
>    that was priced.
>
> **What is genuinely open, and it is not what (c) said.** `DIFF_STRUCTURE.md`
> carries its own `⚠` banner: §3.2 and one row of §4 are **REFUTED** by
> `w-drop3`'s relocation reading (boards #984–#989), and its 3,195 population
> is at tree `0c8a185` while the tree now reads `fnbyte-differs` **1,960** plus
> `fnbyte-reloc-differs` **530** — the caveat the page's `exact` bucket carried
> having since been closed by construction at `w-relo`/#884. So the *numbers*
> want a re-take (one scan) and the *page* wants its refuted sections marked
> where a reader meets them. **Neither is a new instrument, and neither is
> 1–2 wk.** This is instance N of *"check the board before dispatching"*.

The owner asked whether the judge can carry a sliding score, and whether
mismatches can be *modelled* rather than treated as opaque. Answer in three
parts, because the three layers have different rules:

**(a) The gate stays binary, and that is load-bearing.** A 90%-matching obj
*shipped* is a wrong emit, and a wrong emit scores strictly below the refusal
it replaced (`PROGRESS_METRIC.md`) — the 2,490-wrong-function measurement
(#3363) is what that rule is protecting against. Nothing here relaxes it.

**(b) The sliding score already exists as an instrument — one layer of it.**
`FUNCTION_BYTE_MATCH.md` grades every function the port can lower against
real c2's bytes inside refused TUs: `fnbyte-exact 35,894 / differs 1,960 /
reloc-differs 530`, with per-TU fractions and distributions (#3361). Its
separation rule — never in `gate.sh`, licenses no emit — is the template for
every gradient added after it. Two extensions are funded or planned:

- **S0 (blind reach)** extends the gradient to the 113,565 parse-refused
  functions the current instrument cannot even attempt
  (`ROADMAP_SLICING` §5).
- ~~**Mismatch anatomy** (below) extends it *inside* each differing function.~~
  **Mismatch anatomy ALREADY extends it inside each differing function** —
  [`DIFF_STRUCTURE.md`](DIFF_STRUCTURE.md) / `gap/fndiff.rs`, shipped
  2026-08-06 and printed on every scan. Corrected 2026-08-22; see the banner.

**(c)** ~~**The missing instrument is mismatch anatomy — and the whitebox reads
just made it cheap.**~~ **WRONG — IT IS NOT MISSING, AND THE READS ARE NOT WHAT
MAKES IT CHEAP. The paragraph is kept verbatim as the record of the error; see
the ⚠ banner at the head of this section for what is true instead.** Today
`fnbyte-differs` is a count; the diff itself is
opaque. But #3358's read of interface 2 gives us c2's own encoding: the
base-word table (`0x10c3a578`) and the encode-form table (`0x10c39b18`)
decode any `.text` word into `(opcode, fields)`. A differ that decodes both
sides through those tables can classify every wrong function into the
category that names *which pipeline stage diverged*:

**The table below is REFUTED — it is kept because the measured column is the
finding.** The right-hand column was added 2026-08-22 from `DIFF_STRUCTURE.md`
§1–§2 (tree `0c8a185`, 3,195 bodies, 5,189 substituted words):

| diff class | signature | implicated stage | permuter axis | **MEASURED** |
|---|---|---|---|---|
| field-only | same opcodes, same order, a register field differs | allocation | alloc order / tie tier | **2 words of 5,189** |
| permutation | same instruction multiset, different order | scheduling | schedule tie-breaks | **0 bodies — the class is EMPTY** |
| immediate-only | same opcodes/registers, displacement or target differs | layout / label plan | label counters, section offsets | **2 words (12 more reg+disp)** |
| reloc-only | words identical, relocation records differ | symbol/reloc planning | (already broken out: 530) | the one row that survives |
| length-changing | insertion/deletion of instructions | selection / expansion | construct lowering itself | **the population: 99.7 % of substituted words differ in OPCODE, 94.3 % of bodies wrong at word 0, and the mechanism is inlining, not selection breadth** |

This is **not** a semantic-closeness classifier standing in for the judge
(banned) — it is a *measurement against real bytes*, decoded through tables
read out of the real binary, published beside FBM under FBM's separation
rule. ~~Its value is threefold: it localizes 4a's risk per stage instead of
per function; it is the permuter's fitness gradient (a field-only diff says
*search allocation*, not *search everything*); and it is a training label
for the reversing models.~~ ~~Priced at **1–2 wk raw** — the tables are already
read and dumped (`docs/whitebox/scripts/dump_opcode_tables.py`), so this is
a decoder loop plus a classifier over 2,490 known-wrong functions.~~

> **Struck 2026-08-22.** The separation-rule sentence above is right and is
> what the shipped instrument already does. Everything after it is wrong:
> the price is not 1–2 wk because the instrument exists; the tables are **not**
> dumped by that script (it reads `0x10B1B260` and `0x10B202B0` only) and
> `fndiff.rs` never needed them, decoding PPC directly under a
> re-encode-or-refuse rule; and the *fitness gradient* claim is the one the
> measurement kills — a gradient with 2 field-only words and 0 reorderings in
> 5,189 tells a permuter to search nothing. **What the measured population
> points at instead is the INLINE decision**, and this tree already carries a
> cost model for it — `crates/c2-core/src/splice.rs:57-60`, graded 0.9716 with
> a 2.84 % NOT-MODELLED residual — which **no emitter consults**. That is a
> candidate lever; it is named here and deliberately **not** priced, because
> the owner's permuter use case is *matching pretext for hand-written decomp
> source*, a **different population** from the port's own refused bodies, and
> nothing in this repo has measured the two against each other. Do not
> conflate them. See §4's consumer framing, which states the consumer and does
> not claim the populations coincide.

**One caveat carried forward from probe C** (`ARCH_REVIEW_2026-08-21.md`):
the *current* port's internals have no defined projection onto c2's tuple
space, so stage-aligned internal comparison is not defined for the incumbent
shape emitters. S1's design — a general lowering driven from per-op values
carrying **c2's own opcode numbers** — is what makes the projection defined
from S1 onward. That is an additional, previously unstated reason for S1's
design choice.
