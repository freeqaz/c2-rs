# WB_TAILCLASS — FINDINGS and prereg grade, lane `w-tailread`

> **PROVENANCE — DISASSEMBLY-DERIVED.** See [`DISCLOSURE.md`](DISCLOSURE.md).

Prereg: [`WB_TAILCLASS_PREREG.md`](WB_TAILCLASS_PREREG.md). Spec page:
[`ref/P_OPATTR.md`](ref/P_OPATTR.md). Amendments landed beside R6's originals in
[`ref/P_EXPAND.md`](ref/P_EXPAND.md) §1.1, §1.2, §3, §5, §6.

**Image** `compilers/X360/16.00.11886.00/c2.dll`, sha256
`c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`.

---

## 1. What this lane admits, and what it refuses

**Admits.** `0x10c3afd8` read completely: **664 entries**, extent derived rather
than assumed, **byte-identical to the `0x10b1b260` mnemonic table's flags field
on 664 of 664**, its low three bits decoded as an **operand-shape class**
(1 move / 2 load / 3 store / 4 sign-extend / 0 other; 5–7 and bit `0x80`
unused), and **38 consumers** located image-wide. The dispatch tail read to its
five callees and shown to **emit nothing**. The **delete primitive**
`0x10bd5516` named for the first time in this record, with its call-count
asymmetry against the mint. The peephole's arm 6 read *and* obj-checked.
`0x10b1d180` **settled**.

**Refuses.**

* **The `mr r8,r8` idiom is not explained.** 3,792 instances, one register,
  branch-adjacent, and this lane declines to guess what they are. A plausible
  story exists and is deliberately absent from `P_OPATTR.md`.
* **`P_EXPAND.md` §3 is not re-scored.** One arm is named as wrong in sign; a
  corrected signed word table is a follow-up, not a deliverable here.
* **The 506 callback sites are counted, not classified.** One was spot-checked.
* **Every `[R]` in `P_OPATTR.md` stays `[R]`.** Exactly one claim is `[O]`, and
  it is the one that came back **negative**.

**The deliverable came back a different shape than the brief asked for, and
that is the finding.** The brief asked to *"convert '767 opcodes reach the tail'
into 'opcode X is / is not expanded'"*. **There was never a 767-element set to
convert** — 767 is `0x2ff`, the walk's entire domain, and it reports 1024 if you
set the bound to `0x400`. What replaced it is a statement about the tail, which
is stronger: **the tail's word delta is zero for every opcode that reaches it**,
because it attaches an operand rather than emitting one.

---

## 2. Prereg grade — 11 HIT / 2 MISS / 4 PARTIAL / 4 UNGRADED over 21

`[POST]` predictions are scored **UNGRADED — post-read** by the prereg's own
rule (§1) and are **not** counted as hits: I had already seen the answer during
the orientation pass that produced §0.1.

### 2.1 The table

| # | prediction | grade | outcome |
|---|---|---|---|
| **P1.1** `[POST]` | class has ≤ 8 values, ≤ 5 populated | **UNGRADED** | exactly 5 populated |
| **P1.2** `[POST]` | byte-identical to the mnemonic flags field | **UNGRADED** | 664 of 664 |
| **P1.3** | extent exactly `0x298`, not the `0x300` the tail implies | **HIT** | derived two independent ways |
| **P1.4** | some consumer indexes it out of extent; the tail is my candidate | **HIT** | the tail, no bound check |
| **P1.5** | that out-of-bounds read is benign | **PARTIAL** | benign **through `0x302`**, which is every opcode the switch discriminates — `{0,1,4}`, 0 class-2/3 hits. **But not benign in general**: from opcode **`0x33c`** the second table decodes to class 2 and the tail would treat it as a load. I published the unqualified "benign" on a range I had not bounded, then bounded it |
| **P1.6** | more than 5 but fewer than 40 consumers | **HIT** | **38** |
| **P1.7** | the class partitions by **operand shape**, not expansion behaviour | **HIT** | load / store / move / sign-extend |
| **P1.8** | bit `0x80` unused across the table | **HIT** | n = 0 |
| **P1.9** | a second byte table of the same extent follows, also unnamed | **HIT** | `0x10c3b270`, `0x298` entries, 2 consumers, in no document |

### 2.2 The tail

| # | prediction | grade | outcome |
|---|---|---|---|
| **P2.1** | mints zero words **transitively**, not just directly | **PARTIAL** | zero directly, and the five callees are read and named — but **the transitive form of the question turned out to be unanswerable**, see §3. The claim rests on the direct read |
| **P2.2** | so the honest answer is "not expanded", and the deliverable is about the tail, not a 767-row table | **HIT** | and the 767 dissolved for an independent reason |
| **P2.3** | the two live classes converge on one shared body | **HIT** | `0x10c0e398` |
| **P2.4** | that body attaches an operand rather than emitting one | **HIT** | operand node kind `0xb`, tag `0x2ac`, appended to `+0x2c` |
| **P2.5** | R6's counter is one-sided and ≥ 1 arm scored `0..0` in fact **removes** | **HIT** | `0x10c0e4a4`, and I named `0x10c0e4a4` in advance as the candidate |
| **P2.6** | ≥ 1 of the 10 fall-through bodies is a real arm the width rule wrongly excluded | **PARTIAL** | see §4 — the width rule's *verdict* was right on all ten, but for the wrong reason on one, and a **different** arm (`0x302`) was missed by a different mechanism |

### 2.3 The secondary items

| # | prediction | grade | outcome |
|---|---|---|---|
| **P3.1** | arm 6 is a copy-propagation / redundant-move arm, not an expansion | **HIT** | both, with the delete on the same-register path |
| **P3.2** | arm 6 mints nothing | **PARTIAL** | its thunk and handler mint nothing on the read path; the propagation fallback's four callees were not each chased |
| **P4.1** | the contradiction resolves by the **question being malformed** | **HIT** | there is no mapping, because nothing indexes by opcode |
| **P4.2** | name-keyed, not opcode-indexed; `+4` already carries the opcode | **HIT** | `0x10c0299c` reads `+4` explicitly, and the reference split by field settles it: the name search touches only the base and row 1, while the caller reads `+4`/`+8`/`+0xc` |
| **P4.3** | `+4` never `≥ 0x298`, so it can never name a pseudo-op | **HIT** | max `0x295` over 122 rows |
| **P4.4** | R6's `twlti` came from indexing the **first** table past its end | **MISS** | **wrong mechanism.** It came from R6's *stated* hypothesis `op = 0x298 + j` applied to the **extended** table — row 88 **is** `twlti`, exactly. Indexing the first table past its end gives `twige`, which is what misled me into predicting this |

### 2.4 Two more, registered in §0.1 before the read and worth scoring

| claim | grade | outcome |
|---|---|---|
| §0.1(a) the table is already recorded, contra three documents | **HIT** | #2040/#2044/#2106/#2206 |
| §0.1(b) "767" is the walk's domain, not a set | **HIT** | tracks OPMAX exactly at three settings |

### 2.5 Calibration — R6's note, checked against this lane

R6 recorded that **every one of its misses predicted the mechanism tidier than
it is**, and the prereg says I deliberately biased toward messier. That worked:
**P1.4, P1.6, P1.9, P2.5 and P2.6 were all "it will be messier than stated" and
four of the five hit.** The one MISS, **P4.4**, is the exception that proves the
rule — and it failed in the *opposite* direction from R6's: I predicted a
**messier** origin for `twlti` (a second-order trap) when the truth was the
**tidy** one (R6's stated hypothesis, applied exactly as stated). Biasing toward
mess is a correction, not a law, and it cost one prediction here.

---

## 3. The instrument defect this lane found in its own work

Registered because it is the most transferable thing here.

`dump_expansion.py`'s **"767 opcodes reach the tail"** is its walk's whole
domain. My **first** repair for the minting question had the identical defect: a
transitive *"can an instruction constructor be reached from the tail"* query
returns **True** — and returns True for every control too, because c2's call
graph is strongly connected through its arena and diagnostic machinery. A 22-hop
witness path through the error machinery is not evidence about codegen.

**Two saturated predicates, same shape, one lane apart.** The replacement is a
**minimum hop count** (BFS): **8** for all three tail bodies, **1** for three
control arms that demonstrably mint. That is evidence and not proof, 8 is not a
large number, and `dump_tailclass.py` prints the caveat in its own output so the
number cannot be quoted without it. **The argument `P_OPATTR.md` actually rests
on is the direct reading of the five callees**, not the hop count.

The general rule, which the repo already half-knows: **before quoting a count,
change a parameter the count should not depend on and re-run.** 767 → 1024 →
1536 takes ten seconds and would have caught it in R6.

---

## 4. R6's registered gap — the 10 shared fall-through bodies, closed

R6 excluded these *"by a width rule, not by reading them"* and said the arm map
is therefore a subset of the truth. Read, all ten, with predecessor counts:

| body | preds | what it is | was excluding it right? |
|---|---:|---|---|
| `0x10c0da99` | 1 | **not a body — dispatch.** `cmp eax,0x17e / je / cmp eax,0x26c / je / lea ecx,[eax-0x26e] / cmp ecx,1 / ja 0x10c0e30b` | **yes**, and for the wrong reason: it is tree, not an arm |
| `0x10c0e4ab` | 54 | the function **epilogue** | yes |
| `0x10c0e4a4` | 16 | the **delete** join (§ `P_OPATTR` 5.1) | yes as an arm — **but it is not a no-op** |
| `0x10c0dac6` | 2 | real body; reads the type word `[esi+0xa] & 0xfff` | already credited via its narrow path (`xori`) |
| `0x10c0db6b` | 1 | real body; passes `0x10bd3824` as a **callback** to `0x10bd575d` | already credited (`xori`) |
| `0x10c0dc0d` | 1 | real body; operand-kind bitmask test | not in the narrow map |
| `0x10c0dee6` | 12 | real body; type-word compare against `8` | not in the narrow map |
| `0x10c0dfdc` | 2 | real body; `call 0x10bd2d83 / jmp 0x10c0e4a4` → **deletes** | credited (`fmr, mr`) but **mis-scored `0..0`** |
| `0x10c0e103` | 1 | real body; `call 0x10bd2d83`, then operand work | credited (`fmr, mr`) |
| `0x10c0e20e` | 1 | real body; also passes `0x10bd3824` as a callback | not in the narrow map |

**Verdict: the width rule's exclusions were correct on all ten** — none is a
per-opcode arm the tree discriminates that the map lacks. R6's stated worry
("the true arm map is a superset") does **not** materialise here.

**But the map was short anyway, by a mechanism R6 did not consider**: opcode
`0x302` at `0x10c0e479`, missed because the *bound* was computed from literals
(`P_OPATTR.md` §3.1). So P2.6 is PARTIAL: I predicted the right conclusion
("the map is short") from the wrong cause.

---

## 5. The `[O]` cell, and why a negative is the valuable outcome

One claim in this lane is `[O]`, and it **failed**.

`P_OPATTR.md` §6.1 reads arm 6 correctly: same register ⇒ unlink. Allowing
`[R]` to mean "this is what c2 does" licenses *"c2 emits no self-move"*.
`probe_selfmove.py` over **120,000 objs** — 176,969 `.text` sections, 1,726,709
words, **135,218 move-form instructions decoded** as the liveness half —
returns **3,792 self-moves in 1,206 objs**, all `mr`, all `r8`, at `/Ox`, with
no relocation covering them.

**It survived 6,000 objs at zero.** A lane that had sized its corpus by
convenience would have shipped a confirmation of a false statement, with a real
denominator attached. The project's own correctness rule says a green run is
sound only on the IL it was tested against; this is that rule biting inside a
characterization lane rather than in the harness.

**The refutation is narrower than the claim**, and the split is the useful part:
`fmr` is **0 of 32,569**, so arm 6 itself is consistent with the corpus. The
violator is `mr` — arm 14, `0x10c16d83`, a handler this lane did not read.

---

## 5.1 A second error this lane caught in its own published claim

`P_OPATTR.md` §7.1 said *"one referencing function, not two"*, on a
base-literal grep that found 3. Widening the scan to the table's whole address
range found 20 — **seven of them phantoms**, because **both the mnemonic table
and the extended table live inside `.text`**, so `objdump -d` disassembles their
bytes as code and invents branch instructions whose operands land in the table's
own range. Filtering by `FUNCS.tsv` function membership gives the true answer:
**13 references in 3 functions.**

The correction **strengthens** §7 rather than weakening it — the split by
*field* is a cleaner form of the same argument — but the claim as published was
wrong, and it was wrong in a direction (*undercounting*) that a base-literal
grep will always produce for a table whose walker starts at row 1.

**Three of this lane's own defects were found by running things, not reading
them**: this, the `--minters` crash after a rename, and the probe's
vacuity-before-refutation ordering. A fourth (the consumer regex admitting
addresses below the table) was found by checking what actually matched.

## 6. For a follow-up

1. **What is `mr r8,r8`?** Obj-visible, 3,792 instances, needs no disassembly.
2. **Re-score `P_EXPAND.md` §3 with a signed delta.** Both primitives are now
   named; the instrument needs a delete oracle beside its mint oracle.
3. **The second byte table `0x10c3b270`** — `0x298` entries, two consumers
   bound-checked at `0x295`, out-of-range default `0x64`. Unread.
4. **Opcode `0x302`** — arm recovered, meaning not chased, absent from
   `P_ILRECORD.md`'s minting arms (as is `0x2f5`, R6's item 5).
5. **`WB_EXPAND_FINDINGS.md:79` and board #3432 carry the same false "unrecorded"
   sentence** as `P_EXPAND.md` §1.2. The page is amended; those two are not this
   lane's to edit beyond its four reserved rows.
