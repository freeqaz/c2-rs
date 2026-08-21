# Whitebox leverage — read before probe — 2026-08-21

Written on the owner's direction, the same day as the goal re-ranking
(`GOAL_DECISION_2026-08-21.md` § "AMENDED"). The strategic observation being
acted on, from the roadmap round:

> Most historical lane cost is **black-box inference** — probe grids and
> fitted-parameter searches recovering facts the binary states plainly.
> A single alignment nibble cost a lane; `dag.c`'s lowering order took two;
> register allocation is priced at **13 raw / 65 calibrated lanes** if
> recovered by probing. Meanwhile the encoder-table *read* took a lane two
> days and produced 32-of-32-bit reproduction on its population (#3358).

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

**PENDING — this section is populated from the survey lane in the same
commit or the immediately following one. If you are reading this sentence in
a commit older than HEAD, check `git log --follow` for the fill; if it is
still here at HEAD, the survey has not landed and the doctrine above binds
without an enumerated target list.**

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
- **Mismatch anatomy** (below) extends it *inside* each differing function.

**(c) The missing instrument is mismatch anatomy — and the whitebox reads
just made it cheap.** Today `fnbyte-differs` is a count; the diff itself is
opaque. But #3358's read of interface 2 gives us c2's own encoding: the
base-word table (`0x10c3a578`) and the encode-form table (`0x10c39b18`)
decode any `.text` word into `(opcode, fields)`. A differ that decodes both
sides through those tables can classify every wrong function into the
category that names *which pipeline stage diverged*:

| diff class | signature | implicated stage | permuter axis |
|---|---|---|---|
| field-only | same opcodes, same order, a register field differs | allocation | alloc order / tie tier |
| permutation | same instruction multiset, different order | scheduling | schedule tie-breaks |
| immediate-only | same opcodes/registers, displacement or target differs | layout / label plan | label counters, section offsets |
| reloc-only | words identical, relocation records differ | symbol/reloc planning | (already broken out: 530) |
| length-changing | insertion/deletion of instructions | selection / expansion | construct lowering itself |

This is **not** a semantic-closeness classifier standing in for the judge
(banned) — it is a *measurement against real bytes*, decoded through tables
read out of the real binary, published beside FBM under FBM's separation
rule. Its value is threefold: it localizes 4a's risk per stage instead of
per function; it is the permuter's fitness gradient (a field-only diff says
*search allocation*, not *search everything*); and it is a training label
for the reversing models. Priced at **1–2 wk raw** — the tables are already
read and dumped (`docs/whitebox/scripts/dump_opcode_tables.py`), so this is
a decoder loop plus a classifier over 2,490 known-wrong functions.

**One caveat carried forward from probe C** (`ARCH_REVIEW_2026-08-21.md`):
the *current* port's internals have no defined projection onto c2's tuple
space, so stage-aligned internal comparison is not defined for the incumbent
shape emitters. S1's design — a general lowering driven from per-op values
carrying **c2's own opcode numbers** — is what makes the projection defined
from S1 onward. That is an additional, previously unstated reason for S1's
design choice.
