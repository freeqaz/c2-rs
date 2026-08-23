# WB_TAILCLASS — PREREG for the read of `0x10c3afd8`, the per-opcode attribute table

> **PROVENANCE — DISASSEMBLY-DERIVED.** See [`DISCLOSURE.md`](DISCLOSURE.md).
> Nothing here may enter `crates/` without a `DISCLOSURE.md` row naming the
> address it came from.

**Lane:** `w-tailread` · **kind:** characterization lane
(`docs/rungs/README.md` § "Lane kinds" 3) · **Fixtures:** none ·
**Census:** +0 · **predicted reach:** 0, registered.
**Board rows:** **#3460**–**#3463** (reserved, `docs/BOARD.md` sixth-wave ledger).

**Subject.** R6's top-ranked follow-up
(`docs/rungs/2026-08-23-w-read-r6.md` § "Found and not taken" item 1;
`ref/P_EXPAND.md` §1.2, §8): **read the per-opcode byte table at
`0x10c3afd8`** that the final-expansion dispatch tail `0x10c0e30b` consults,
*"reached by 767 opcodes"*. Secondary: the peephole's arm 6
(`fmr` @ `0x10c1838b`), R6's registered gap of the 10 shared fall-through
bodies, and — **if and only if cheap** — the `0x10b1d180` index contradiction
R6 deliberately refused to publish.

---

## §0 — Image and addresses, VERIFIED before this file was written

**Image.** `compilers/X360/16.00.11886.00/c2.dll`, sha256
`c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`,
size 1 347 072 — verified by `sha256sum` against the pin at `ref/README.md:21`
before any address below was touched. The worktree has no `compilers/` of its
own; it is a symlink to the main repo's, which `.gitignore` line 46 already
anticipates. **Nothing under `compilers/` is committed.**

**The brief's four addresses, checked against the image itself.** The brief
says in terms that the coordinator has *not* verified them and that briefs have
been wrong eight lanes running.

| target | brief says | image says | verdict |
|---|---|---|---|
| `0x10c0e30b` | the dispatch tail | `8a 88 d8 af c3 10` = `mov cl,BYTE PTR [eax+0x10c3afd8]` | ✅ **exact, byte-for-byte** |
| `0x10c3afd8` | per-opcode byte table, low 3 bits a class | operand of the above; `80 e1 07` = `and cl,0x7` follows | ✅ **exact** |
| `0x10c1838b` | peephole arm 6 = `fmr` | *not yet read* — registered below as P3 | pending |
| `0x10b1d180` | stride-16 `{name, op, BO, BI}` | *not yet read* — registered below as P4 | pending |

### §0.1 — TWO THINGS IN THE BRIEF ARE ALREADY WRONG, AND I FOUND THEM BEFORE FREEZING THIS FILE

Registered here, before the rest of the read, so neither can be claimed as a
discovery later and neither can be quietly dropped.

**(a) The table is NOT unrecorded.** `ref/P_EXPAND.md` §1.2 says *"No document
in this repo records this table"*, `WB_EXPAND_FINDINGS.md:79` calls it *"an
unrecorded table"*, and board **#3432** repeats it as *"recorded in no document
here"*. **All three are false.** `0x10c3afd8` is board **#2040**, **#2044**,
**#2106** and **#2206** (lane `wb-select`, 2026-08-09) and
`docs/rungs/2026-08-09-wb-select2.md:67` states in one line: *"The same byte is
exposed as an array at `0x10c3afd8`, indexed by machine opcode."* #2044 decodes
four of its bits — `0x08` = `Rc=1`, `0x10` = has an `Rc` sibling, `0x20` =
writes `XER[CA]`, `0x40` = reads `XER[CA]`. R6 read a *known* table and believed
it new. **What is genuinely unread is the low 3 bits**, which no row decodes.

**(b) "767 opcodes reach the tail" is not a measurement of a set.** `0x2ff` =
767, and `dump_expansion.py`'s walk domain is exactly opcodes `1..OPMAX` with
`OPMAX = 0x2ff`. Re-run verbatim on the pinned image, the tool also reports
**six** of the ten shared fall-through bodies as reached by `767` — the same
number. 767 is the **whole domain**: it says the tail is reachable carrying an
un-narrowed opcode interval, which is a fact about the abstract interpretation,
not about c2. **The brief's framing — "converts 767 reach the tail into opcode
X is / is not expanded" — is therefore not a task that can be completed as
worded**, because there was never a 767-element set to convert. The deliverable
below is restated accordingly and the restatement is registered *in advance*.

---

## §1 — Grading rule

Each prediction is **HIT**, **MISS**, **PARTIAL**, or **UNGRADED**. A prediction
is scored only if this lane produced the evidence that decides it.

**Predictions marked `[POST]` were written AFTER I had already read the thing
they predict, during the orientation pass that produced §0.1. They are recorded
for completeness and are scored `UNGRADED — post-read`. They may not be counted
as hits.** Naming them is the point: a prereg whose author has already seen the
answer is not a prereg, and pretending otherwise is the failure the convention
exists to prevent.

**R6's calibration note binds this lane** (`2026-08-23-w-read-r6.md`): every one
of R6's four misses predicted the mechanism **tidier than it is** — a smaller
arm set, a cleaner partition, a flatter prologue. I have deliberately biased
every prediction below toward *messier*: more classes than needed, more
consumers than one, a table that does not tile its index space, and at least one
consumer that reads it out of bounds.

---

## §2 — Predictions on the table itself

| # | prediction | rationale |
|---|---|---|
| **P1.1** `[POST]` | The low 3 bits are a class with **≤ 8** values, of which **at most 5** are populated | orientation |
| **P1.2** `[POST]` | The table is byte-identical to the `0x10b1b260` mnemonic table's `+8` flags field | #2044 says "the same byte" |
| **P1.3** | The table's extent is **exactly `0x298` = 664 entries** (opcodes `0x000..0x297`) and **not** the `0x300` the dispatch tail's index range implies | the mnemonic table ends exactly at `0x10b1d180` |
| **P1.4** | **At least one consumer indexes it with an opcode > `0x297`, i.e. out of its extent.** The dispatch tail is my candidate: it applies no bound check | biased messy, per R6's calibration |
| **P1.5** | If P1.4 holds, the out-of-bounds read is **benign** — the bytes it lands in do not decode to the classes the tail acts on | a live OOB in a shipped compiler would have been noticed |
| **P1.6** | The table has **more than 5 but fewer than 40 consumers** image-wide | R6 implied one; one is never the answer |
| **P1.7** | The class field partitions along **operand shape** (load / store / move), not along expansion behaviour | the tail's two live classes both re-walk an operand list |
| **P1.8** | **Bit 7 (`0x80`) is unused** across the whole table | — |
| **P1.9** | A **second** byte table of the same extent sits immediately after this one and no document names it either | `0x10c3b270` = base + `0x298` is referenced twice |

## §3 — Predictions on what the tail DOES

| # | prediction | rationale |
|---|---|---|
| **P2.1** | The dispatch tail mints **zero instruction words on every path**, transitively — not merely zero direct constructor calls | it re-walks operands, it does not build |
| **P2.2** | Therefore the honest answer for every opcode reaching the tail is **"not expanded"**, and the deliverable is a statement about the tail, not a 767-row table | follows from P2.1 |
| **P2.3** | The tail's two live classes converge on **one** shared body | `0x10c0e398` appears as a `je` target from both |
| **P2.4** | That shared body **attaches an operand** to the instruction rather than emitting one | — |
| **P2.5** | **R6's word-count instrument is one-sided: it counts additions and cannot see deletions**, and at least one arm R6 scored `0..0` in fact **removes** the instruction | `0x10c0e4a4`, R6's "no-op join", tail-calls something before returning |
| **P2.6** | At least one of the 10 shared fall-through bodies, once read, turns out to be a **real** arm that R6's width rule wrongly excluded | biased messy |

## §4 — Predictions on the secondary items

| # | prediction | rationale |
|---|---|---|
| **P3.1** | Peephole arm 6 at `0x10c1838b` handles `fmr` and is a **copy-propagation / redundant-move** arm, not an expansion | its class-1 siblings are `mr`, `mr.`, `vmr` |
| **P3.2** | Arm 6 mints **no** instruction | R6's `no-mint` column, all 18 |
| **P4.1** | The `0x10b1d180` contradiction **resolves**, and it resolves by the question being **malformed** rather than by a mapping being found | R6 looked for an index that nothing computes |
| **P4.2** | The table is **name-keyed, not opcode-indexed** — its `+4` field already carries the real opcode, so no row-index → opcode mapping exists to publish | its only consumers are string-compare loops |
| **P4.3** | The `+4` field **never** holds a value `≥ 0x298`, i.e. this table can never name a pseudo-op — which is what makes `"0x2f0 decodes to a trap mnemonic"` a non-statement | if it could, the assembler could write pseudo-ops |
| **P4.4** | R6's `twlti` reading came from indexing the **first** table past its end (the #3357 trap), not from this table's own index | I get a *different* garbage mnemonic at `0x2f0` |

## §5 — Decline criteria, registered in advance

This lane **declines and publishes nothing** on an item when:

1. **`0x10b1d180`** — if the resolution is not decisive, I follow R6's precedent
   verbatim: refuse to publish a mapping and say why. **A declined item here is
   the correct outcome, not a failure** (brief's own words). Specifically: I
   publish only if I can state what *does* index the table and show the
   contradiction dissolving, not merely offer a better-fitting index.
2. **Any `[O]` upgrade** — I mark a claim `[O]` only if a real obj or a
   `/FAsc` listing decided it. Reading the table correctly is `[R]` forever;
   that is the `.bss`-bump rule (`C2_MAP_METHOD.md` §7) and R9's #3444 is the
   most recent lane to have to be reminded of it.
3. **The 10 fall-through bodies** — if reading them cannot separate "real arm"
   from "shared exit", I report the ambiguity rather than a number.

## §6 — Fences on the instrument

`dump_tailclass.py` must, before it is quoted:

* verify the pinned sha256 and **refuse** otherwise, and
* be **watched failing on deliberately broken input** — a fence that has never
  been seen to refuse is not evidence that anything was fenced
  (`CLAUDE.md`; R6's own rule).

Both are recorded in the rung with the observed output, not asserted.

## §7 — What this lane may touch

`docs/whitebox/**`, `docs/rungs/`, and **only** rows #3460–#3463 of
`docs/BOARD.md`. **Zero `crates/` bytes, zero `fixtures/` bytes.** Peer lanes
`w-s1c2`, `w-4f01` and `w-pwords` hold those trees this wave.

Corrections to peer documents are **amended beside, never rewritten in place**
(`ref/README.md` §2.1; R9's `P_LABEL.md` precedent). §0.1(a) and §0.1(b) both
correct `ref/P_EXPAND.md`, and both will land as amendment boxes with R6's
original claim intact next to them.
