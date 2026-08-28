# PREREG — lane `w-sched` (rung `docs/rungs/2026-08-28-w-sched-r7.md`)

> **FROZEN BEFORE MEASUREMENT.** Committed as the first commit on branch
> `wt-w-sched`. No tap has been run, no mutation graded, no disassembly of
> `FUN_10b327cd` read, at the moment this file is committed. What HAS been read
> and is therefore not predicted: `CLAUDE.md`,
> `docs/DECISIONS_2026-08-22.md` § decision 21, `READ_PLAN_2026-08-21.md` §3
> row R7 and its two amendment boxes, `ref/P_DAG.md` (whole),
> `WB_SCHEDCONF_FINDINGS.md` (whole, including §3.3's clause histogram and
> §4.1's reorder table), `WB_F0PRICE_FINDINGS.md` §6 row 4,
> `docs/rungs/README.md` § "Lane kinds", `docs/whitebox/scripts/grade_regions.py`
> (source, not output), and `c2host/stagetap.c` (source, not output).
>
> **The published numbers I already know, and which therefore cannot count as
> findings of this lane:** 1,461/1,461 region grades; the clause histogram
> `excl-0x17/0x30f 1121 · incl-cat-12 204 · incl-cat-1b 136`, four exits never
> fired; longest region 14; run-4 reorder 3/357.

**Lane kind** characterization. **Fixtures** none. **Census** +0.
**Predicted reach 0. Predicted `crates/` bytes 0. Predicted `DISCLOSURE.md`
rows 0.** **No `gate.sh` row is added** (`#3691`).

---

## 0. THE FIRST THING THIS LANE MUST SAY, AND IT IS REGISTERED BEFORE ANY WORK

**R7 has already run.** It was discharged on 2026-08-23 by lane `w-read-r7`
(`docs/rungs/2026-08-23-w-read-r7.md`, outcome `built`, board #3433–#3436,
prereg `WB_SCHEDCONF_PREREG.md` frozen at `af966da13`). Its brief was, word for
word, this lane's brief: *promote the scheduler model from `[R]` to `[O]` by
confronting it with the live tap.* Decision 21's `w-sched` row re-issues it.

So **P0.1 (confidence 0.97): this lane's honest first deliverable is a
staleness banner, not a fresh confrontation.** It is instance N of MEMORY's
*"check the board before dispatching"* and of `docs/rungs/README.md`'s own
*"CHECK THE BOARD, AND CHECK THE TREE, BEFORE PRICING A NEW INSTRUMENT"*.
**Refuted if** `docs/rungs/2026-08-23-w-read-r7.md` turns out not to exist, or
its `Outcome:` is not `built`, or its scope excludes the tap confrontation.

**What is NOT already done, and is this lane's actual work.** `w-read-r7`
answered the discriminability question **in aggregate** — *"a simulator that
returns its input scores 98.9 %"*, §4.2 — and gave four prose bullets in §7.
Nobody has asked it **clause by clause**: *for each clause of `P_DAG` §3/§5, is
there an observation the live tap can make that the clause's negation would not
also produce?* That is decision 21's `w-sched` row read literally, it is
executable, and it is what this lane registers below.

---

## 1. THE INSTRUMENT, AND THE CONTROL THAT MUST BE ABLE TO FAIL

**Instrument A — the mutation grid.** `grade_regions.py`'s `find_region` is
`FUN_10be5d4b` transcribed clause by clause. This lane parameterises it, runs
each single-clause mutant over ONE frozen `c2rs stage snap` stream, and reports
hits/misses per mutant. **A mutant that scores identically to the unmutated
rule is a clause the tap cannot confirm** — the negation produces no
observation the affirmation does not.

**Control C-A (pinned by NAME and by COUNT, and it can fail).** The unmutated
rule must score **exactly 1,461 GRADED / 1,461 HIT / 0 MISS** with clause
histogram `excl-0x17/0x30f 1121 · incl-cat-12 204 · incl-cat-1b 136` and 1,368
UNGRADED, reproducing `WB_SCHEDCONF_FINDINGS.md` §3.1/§3.3 exactly. **If it
does not, my snap stream is a different population from R7's and every mutant
colour taken on it is VOID** (`docs/rungs/README.md`'s rule: a colour taken in
an unvalidated environment is void, not provisional). Registered failure
action: keep the invalid log, re-derive the fixture set, re-run; if it still
will not reproduce, the mutation grid is reported **UNGRADED** and this lane
falls back to §3 and §4 alone.

**Control C-B (a mutant that MUST die).** At least one mutant must go red, or
the grid is decoration (`#3336`). Registered in advance as C-B: **M-CAP-2**
(cap lowered to 2). If M-CAP-2 does not go red the grid is not measuring
anything and is reported as such.

**Instrument B — the disassembly.** `FUN_10b327cd` (node create, 158 B) and
`FUN_10b328da` (DAG build) read from
`~/ghidra-projects/export/c2/objdump_intel.asm`, cross-checked against the
pinned image `compilers/X360/16.00.11886.00/c2.dll`
sha256 `c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`.

---

## 2. THE MUTATION GRID — predictions, one row per mutant

Each mutant changes exactly one clause of the transcribed rule. `GREEN` means
it scores the same as the unmutated rule (**clause unconfirmed by the tap**);
`RED` means it loses cells (**clause confirmed, and by how many cells**).

| id | mutation | prediction | conf | refuted if |
|---|---|---|---:|---|
| **M0** | none (control C-A) | 1461/1461 | 0.85 | any other number |
| **M-HEAD-DROP** | remove the `opcode == 0x30f` head special case | **RED, ≥ 1000 cells** | 0.85 | GREEN, or < 500 red |
| **M-HEAD-ANY** | head taken unconditionally (any opcode) | **RED, but small (< 100)** | 0.40 | GREEN ⇒ no walk begins on a stop tuple |
| **M-HEAD-OP** | head constant `0x30f` → `0x30e` | **RED, ≈ M-HEAD-DROP** | 0.85 | GREEN |
| **M-12-EXCL** | cat `0x12` inclusive → exclusive | **RED, ≈ 204** | 0.90 | GREEN |
| **M-1B-EXCL** | cat `0x1b` inclusive → exclusive | **RED, ≈ 136** | 0.90 | GREEN |
| **M-14-EXCL** | cat `0x14` inclusive → exclusive | **GREEN** | 0.95 | any red cell |
| **M-19-INCL** | cat `0x19` exclusive → inclusive | **GREEN** | 0.95 | any red cell |
| **M-14-DROP** | remove `0x14` from the stop set | **GREEN** | 0.90 | any red cell |
| **M-19-DROP** | remove `0x19` from the stop set | **GREEN** | 0.90 | any red cell |
| **M-12-DROP** | remove `0x12` from the stop set | **RED** | 0.90 | GREEN |
| **M-1B-DROP** | remove `0x1b` from the stop set | **RED** | 0.90 | GREEN |
| **M-17-ANY** | `0x17` stops regardless of opcode | **RED** | 0.50 | GREEN ⇒ cat `0x17` never appears mid-region with another opcode |
| **M-17-INCL** | `0x17`/`0x30f` exclusive → inclusive | **RED, ≈ 1121** | 0.90 | GREEN |
| **M-CAP-k** | cap `0x50` → k, swept over k = 80, 40, 20, 16, 14, 12, 10, 8, 4, **2** | **GREEN for all k ≥ 15, RED below** | 0.70 | a different threshold — the threshold itself is the deliverable |
| **M-CAP-UNSIGNED** | signed `>` → unsigned `>` | **GREEN, and PROVABLY so** | 0.98 | any red cell |

**P2.1 (conf 0.9).** At least **four** of the rule's clauses will be shown
GREEN — unconfirmable by this tap at any cell count — and they will be exactly
the four exits `WB_SCHEDCONF` §3.3 reports as never fired, plus the cap
constant.

**P2.2 (conf 0.8).** The tap does not confirm `CAP = 0x50`. It confirms a
**ray** `CAP ≥ k` for some `k ≤ 16`, and the read constant sits **≥ 5×** beyond
anything the corpus exercises. *This is the model-or-fit failure mode of
`#3388` in a second place: a constant whose control is structurally incapable
of exercising it.*

**P2.3 (conf 0.75).** M-CAP-UNSIGNED is GREEN for a reason **different in kind**
from the four never-fired exits: it is unfalsifiable *by construction* (the
counter is initialised to 0 and only incremented, so the two compares agree on
every reachable input), not *by corpus*. The findings must separate
`UNCONFIRMABLE-BY-CORPUS` from `UNCONFIRMABLE-BY-CONSTRUCTION`; conflating them
would report a tautology as a coverage gap.

---

## 3. THE CLAUSE DISCRIMINABILITY TABLE — the lane's main deliverable

Every clause of `P_DAG` §3 (priority, as amended by R7 to six terms) and §5
(latency, as amended to tag dispatch), plus §2's region and issue rows,
classified into exactly one of:

* **`[O]`-CONFIRMED** — the tap graded it and a mutation of it goes red.
* **UNCONFIRMABLE-BY-CORPUS** — the tap has a channel for it, but this corpus
  has no cell in which the clause and its negation differ.
* **UNCONFIRMABLE-BY-CONSTRUCTION** — the clause and its negation are
  output-identical on every reachable input.
* **NO CHANNEL** — the tap emits no observable that any negation of the clause
  could move. `[R]` forever *by this instrument*.

**P3.1 (conf 0.9).** The tap's per-tuple row is `{opcode, cat, flags, cc}` from
the **tuple** record (`stagetap.c` `tap_walk_tuples`), optionally plus a raw
tuple-byte window and an operand/symbol walk. **No DAG-node field is emitted by
any tap site.** Therefore **every clause of §3's priority function and every
clause of §5's latency mechanism falls in NO CHANNEL or
UNCONFIRMABLE-BY-CORPUS — not one is `[O]`-CONFIRMED**, and the only
`[O]`-CONFIRMED clauses in the whole model are region-partition clauses.
**Refuted if** any priority or latency clause can be shown to move a tap
observable on this corpus.

**P3.2 (conf 0.85).** The **only** thing the tap confirms about ORDER is the
single bit *"the final schedule is not the identity"*, witnessed by
**3 functions of 357**. Every finer clause — which of the six priority terms,
which weight, the `+0x3c`-vs-`+0x38` field, the 16-bit truncation of `+0x44`,
the per-cycle re-sort, the `≤ 2` issue cap, the slack term, the `+15`/`+40`
stall penalties, the iteration — is unconfirmed, and **most are pairwise
indistinguishable on 3 cells regardless of how the cells come out**.

**P3.3 (conf 0.6).** At least one clause `P_DAG` presents as a *fact about c2*
will turn out to be unfalsifiable in **both** senses at once — no channel AND
no population — and the page marks it neither.

**P3.4 (conf 0.55).** The count of NO-CHANNEL clauses will **exceed** the count
of `[O]`-CONFIRMED clauses by more than 3:1.

---

## 4. THE §8 FOLLOW-UP PRICES — checked, because F0 now leans on them

`WB_SCHEDCONF_FINDINGS.md` §8 prices two follow-ups, and `w-f0price` carried
the first into F0's price as an **UNPRICED** term (`#3716`, F0 sub-item 4).
This lane does **not** re-price F0 (decision 21's hard limit); it checks the
two prices §8 published, and reports the bearing only.

**P4.1 (conf 0.90).** §8.2 — *"Expose `node+0x26` and `node+0x38` in the tap
(≈0.5 day). **Three fields in `tap_walk_tuples`**"* — is **wrong about the
record**. `+0x26`/`+0x38` are **DAG node** fields; `tap_walk_tuples` walks
**tuple** records. They are different structures and the tap has no node
pointer. **Refuted if** a tuple field at `+0x26`/`+0x38` is the same storage.

**P4.2 (conf 0.85).** §8.2 is **also wrong about the time**. The region tap
fires at region-finder **ENTRY** (`0x10be643e`, `stagetap.c` `g_sites`), and
`schedule_run`'s body is `find_region → build_dag → prioritise → emit_cycles`,
so at the hook the DAG for **this** region does not yet exist. No edit to
`tap_walk_tuples` can read a field of a node that has not been created.

**P4.3 (conf 0.45) — the rescue, registered because it would make §8.2's price
nearly right for a reason §8.2 does not give.** The region tap fires once per
region, so at region *k*'s entry the DAG of region *k−1* is **complete**
(built, prioritised, scheduled). If a tuple→node back-pointer exists, fanout
is readable at the existing site with **no new hook** — off by one region.
**Registered both ways**: 0.45 that `FUN_10b327cd` writes a node pointer into
the tuple record; 0.55 that it does not.

**P4.4 (conf 0.7).** Whatever P4.3 comes out, the corrected price of §8.2 is
**strictly greater than 0.5 d**, and this lane will publish it as a *shape*
(what must be built), not as a wall-clock number —
`WHITEBOX_LEVERAGE` §3.1's rule, and the same refusal `#3716` made.

---

## 5. WHAT THIS LANE WILL NOT DO

* **No scheduler.** Decision 20 §2 and decision 21 §4 stand. No simulator is
  built, no order model is fitted, `crates/` is not touched.
* **No re-pricing of F0.** `w-f0price`'s number is quoted as theirs.
* **No new `gate.sh` row** (`#3691`).
* **No population built to reorder.** §8.1's ≈1 d item is out of scope; this
  lane's job is to say precisely what is missing without it.
* **No renaming** of `P_DAG`'s terms. R7 flagged *"has-symbol-dest"* as
  backwards and declined to rename; that decline stands.

## 6. DECLINE / FAILURE CRITERIA, registered in advance

1. If control **C-A** cannot be reproduced, the mutation grid is **UNGRADED**
   and said to be so; the lane does not quote a mutant colour taken on an
   unvalidated stream.
2. If control **C-B** does not go red, the grid is **decoration** and is
   reported as having measured nothing.
3. If the toolchain is absent the tap half is **SKIPPED**, said in that word,
   and §3's table is delivered as an argument from the tap's *source* with
   every row marked as unmeasured. The lane's outcome word would then be
   `declined`, not `built`.
4. If P0.1 is refuted — R7 did not in fact run — this prereg is void and the
   lane runs R7 as originally specified.
5. **Outcome word.** `built` only if the mutation grid grades with both
   controls live AND §3's table is delivered complete. Otherwise `declined`,
   or `FAILED` in that word if neither lands.
