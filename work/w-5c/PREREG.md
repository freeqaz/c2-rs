# w-5c — PREREG, written and committed BEFORE the first capture

Lane `w-5c`, worktree `wt-w-5c` off master **`119af05f`** (the `w-4c` merge).
Rung: **`expr-op-0x5C`**, the EH LIVE-STATE marker — the floor lane `w-4c`
preregistered by name and then measured at **0 → 36,477** with the `0x4C` sink
on (board **#1390**, `rungs/2026-08-08-w-4c.md` §5).

Nothing below was written after a capture. Score every row in the rung's §8.

---

## 0. What I read first, and what it already settles

`grep -ril 0x5C docs/ scripts/` + a topic pass over `BOARD.md`'s rows. Oldest
hit read last. The relevant prior art, in the order it matters:

* **`docs/EH_RECORDS.md` §7.1 (2026-07-31, lane WEH) already MEASURED the
  width.** `5C <TYPE> <varint>`, on `work/WEH/probe/{p1,p2}.cpp` at the
  workload's own flags, with the TYPE varying in length (`86 41 74` 3 B,
  `A6 43 8C 20` 4 B) and the second field a varint that really does escape
  (`5C 86 41 74 80 01 01 00 00`). *"Neither a fixed width nor a plain-byte read
  survives the corpus."*
* **`crates/c2-il/src/func/body/shapes/control_flow.rs`'s `operand()` CONSUMES
  that width today** — the `0x5C` arm reads `cf-eh-live-type` then
  `cf-eh-live-state` and bumps `eh.live_stmts`. So this is **`0xBD`'s
  diagnosis, second instance**: `chain_skip_form`'s `None` at `0x5C` is *"no row
  was written"*, not *"no evidence exists"*. The difference from `0xBD` is that
  `SkipForm::TypeVarint` **can already spell it** (`2C`, `33`, `99` use it), so
  the omission is a plain gap and not an expressiveness one.
* `ctor_dtor.rs` eats `5C <INT TYPE> <state>` in four shape recognizers.
* Board **#1354**/**#1357** (lane `w-one`): `src/Main.cpp`'s ladder is a
  **cycle** whose repeated key is
  `expr-call-in-expr-recv-object-then-op-0x5C`, and **granting `op:5C` left the
  blocker set IDENTICAL** — measured, and the reason the shipped `ladder.py`
  trials every tail-opcode candidate.
* Board **#1387**: `w-bd` widened `LEGAL_OPEN` once *for the `5C`/`5D`/`5E`
  family* and recorded doing so. My instrument must not repeat that quietly.

**Five board rows have been re-scheduled after already measuring zero**, so the
first question I owe is whether this one has been measured. It has not: no row
pins `chain_skip_form(0x5C)`, and #1390 files the 36,477 as a *floor*, not as a
decline.

---

## 1. P-WIDTH — the width, registered

**`chain_skip_form(0x5C) == Some(SkipForm::TypeVarint)`** — `5C <TYPE> <varint>`.

Registered rivals, all four to be scored on the workload:

| | reading |
|---|---|
| **TV** | `5C <TYPE> <varint>` — **the claim** |
| P | payload-free |
| T | `5C <TYPE>` |
| V | `5C <varint>` |
| TT | `5C <TYPE> <token>` |

---

## 2. P-SHAPE — which of `w-4c`'s two shapes this token is

`w-4c` missed by +109 % because `4C` turned out to be *a closing bracket at the
end of every call* — one floor under every call site at once — rather than a
token mid-body. The brief's first question is which shape `0x5C` is. **I
register: NEITHER of the two, and the third thing.**

**`5C` is a STATEMENT-TERMINAL TRAILER OVER A NARROW POPULATION.** It is
bracket-*like* in position — `control_flow.rs`'s own comment says *"the `5C` is
the last token of its statement (it stands immediately before the `4B`)"* — and
mid-body-*like* in reach, because it is emitted only for a statement in which an
object with a destructor became live, not for every statement and not for every
call.

Registered discriminators, to be measured on the workload before anything is
read off the result:

| | registered |
|---|---|
| **P-SHAPE-1** — `5C` sites per body, among bodies that carry one | **median 1**, mean **< 3** (`4C`'s is ~4 by `w-4c`'s own 3.54 M sites / 878 TUs) |
| **P-SHAPE-2** — fraction of `5C` sites whose payload is immediately followed by `4B` | **≥ 85 %** (statement-terminal) |
| **P-SHAPE-3** — total anchored `5C` sites on the workload | **150,000 – 600,000** (`4C` is 3.54 M; EH_RECORDS §7.3 puts 310,371 bodies on the EH axis at all) |

If P-SHAPE-1 or P-SHAPE-2 misses badly, the rung is a bracket after all and the
ladder estimate below is wrong in `w-4c`'s direction.

---

## 3. P-POP — the population, and what my grid must NOT exclude

`w-bd` declined `0x4C` because its 26,701 sites were **zero-argument calls
only**. Asked *before* grading, the classes my `5C` grid could wrongly exclude
are:

1. **The escaped state varint.** `5C 86 41 74 80 01 01 00 00` occurs (§7.1). A
   grid of only single-byte states would agree with the fixed-width rival T+1.
   **Registered: the grid reports the escaped-state count separately and it must
   be > 0.**
2. **The 4-byte TYPE.** `A6 43 8C 20` / `86 46 82 20` — a grid of only
   `86 41 74` would agree with a fixed 5-byte reading.
   **Registered: ≥ 2 distinct TYPE lengths, and the 4-byte class must be > 0.**
3. **`5C` in OPERAND position rather than statement position.** §7.2 records
   both spellings in one probe. **Registered: reported as its own row, never
   folded into the claim.**
4. **The generated-destructor sub-object `5C`** (`ctor_dtor.rs`'s four
   recognizers) versus the ordinary-function local (`int userfn(int a){ MemA s;
   g(a); return a+1; }`). §7.1's whole finding is that these are the same token;
   the grid must contain both.

---

## 4. P-ANCHOR — two anchors, one non-circular by construction

**Anchor A (non-circular).** Walk forward from a statement end (`4B`) with
`control_flow.rs`'s `operand()` widths — a **different table** from the one
under test — and **stop AT the first `5C`, never stepping over one**. The site's
position is therefore fixed by the *other* tokens' widths and finding it does
not presuppose its own answer. An opcode the stepper lacks **abandons** the site
rather than being guessed past.

**Anchor B (walk-free, different bias).** `55 <TYPE> 4C 5C` — an argument-closing
call-end located with no stepper at all (`w-4c`'s anchor B, now that `4C` is
pinned) with a `5C` immediately after it. Uses only the self-delimiting TYPE
reader.

**Registered: the two anchors disagree about a POSITION 0 times.** A B-site
strictly inside a region A walked past would be a real conflict and the decline
condition (§7).

**P-KAC — the known-answer control.** With the chain sink at `w-4c`'s own fixed
token set **plus `op:5C`**, the **base** 878-TU scan reads
**`expr-chain-noform-0x5C` = 36,477**, reproducing board #1390's published
`expr-op-0x5C` to the unit. A different number there indicts the instrument
before any new number is worth reading.

---

## 5. P-LADDER — how many ladders extend, and by how much

**Registered: 0 of the 17, +0 rungs.** Point estimate ZERO, and this is the
first row of this lane that a reader should check.

The reasoning, so it can be scored as reasoning: `w-4c`'s tip exits are
`READER-CLEAR` ×6, `noform-0x00` ×2, `noform-0x13`, `noform-0x1C`,
`noform-0x10`, `noform-0x11`, `assign-rhs-call-0x26`, and four truncated rows.
**Not one is a `5C`.** The one frontier TU known to reach the byte —
`src/Main.cpp` — reaches it as `expr-call-in-expr-recv-object-then-op-0x5C`, a
key raised by `mcall`'s **diagnostic classifier**, which does not consult
`chain_skip_form` at all; `w-one` measured that granting `op:5C` leaves that
blocker set identical, and a *width* cannot change a refusal that is not in
`parse_expr`.

**Registered direction of error: UPWARD.** Two named reasons, both of which
would make 0 too low:

* `work/w-front3/hatch.py` **applies on this master** (board #1405, lane
  `w-hatch`) where it refused on `w-4c`'s. Three rows `w-4c` had to truncate
  (`osfinfo`, `negate_test`, `Main.cpp`) and one more (`jsonwriter`) can climb
  further than that table shows, and a row that climbs further can meet a `5C`.
* Board **#770**'s streak. Ten consecutive optimistic misses, then `w-4c`'s
  pessimistic one. I am registering the **lowest possible** number, so my only
  available error is upward, and I am naming it rather than being surprised by
  it.

**Registered ceiling: ≤ 4 of 17.** If more than four extend, `5C` was a bracket
and §2 is refuted.

---

## 6. P-WORKLOAD — the whole-workload sink scan (the instrument #1384 prefers)

Token set = `work/w-4c/sinkset.txt` **+ `op:5C`**, identical at both ends.

| | registered |
|---|---:|
| **P-W1** `expr-chain-noform-0x5C`, base | **36,477** (= P-KAC) |
| **P-W2** `expr-chain-noform-0x5C`, tip | **0** |
| **P-W3** the relabelling is an **IDENTITY** | negatives = positives, **net 0**, every moved key named |
| **P-W4** `fn_blockers` **sum** identical to the unit at both ends | yes |
| **P-W5** fraction of the 36,477 that reach the function TAIL (`noform-0x4F`) | **< 50 %** — `4C` sent 82.8 %, and I expect `5C` to be shallower because the EH bodies carry more after it |
| **P-W6** the single largest successor | **an EH trailer — `noform-0x5D` or `noform-0x5E`** (§6 of EH_RECORDS: 33 of 33 census representatives carry both a `5C` and a `5E`) |

## 6.1 P-QUIET — the sink is OFF everywhere that is published

| | registered |
|---|---|
| TU match | **11 → 11** |
| mismatch · codegen-gap · port-error | **0 · 0 · 0** at both ends |
| vocab-gap · capture-fail | **860 · 7** at both ends |
| `gap-metric` lines that move | **0 of 187** |
| `fn_blockers` / `emit_blockers` keys that move (sink OFF) | **0 / 0**, sums identical to the unit |
| `work/w-splice/peerkeys.py` families vanished | **0** |
| `#[test]` bodies under `crates/` | **1,190 → 1,192 … 1,194** (+2 to +4) |
| `scripts/gate.sh --require-graded` | **18/18 PASS, 0 mismatch anywhere** |

**Board #139's rule — a measure's acceptance vocabulary must match its
emitter's.** Registered as **not binding on the shipped diff** (it adds one
*width* to a width-only table inside a poisoned sink that cannot reach
`select_function`, and no acceptance vocabulary at all), and **binding on the
instrument**, where anchor B admits a `55 <TYPE>` at exactly the gate
`shapes::calls::eat_call_args` applies.

---

## 7. The DECLINE conditions, registered in advance

Declining with two confirmations that do not settle it is a full result. I
decline — ship nothing to `chain_skip_form` — if any of:

* **D1** the two anchors disagree about a position **even once**;
* **D2** TV's desync rate on the anchored population exceeds **2 %** and the
  residue does not resolve to unpinned opcodes by a control that never mentions
  `5C`;
* **D3** the escaped-state class or the 4-byte-TYPE class is **empty** in the
  grid — that is `w-bd`'s excluded-population failure and I would be pinning a
  width on the uninteresting half;
* **D4** the capture does not grade `ReferenceReplay=ByteExact`;
* **D5** P-KAC misses 36,477.

---

## 8. P-NEXT — the floor after this one, named before it is measured

**`0x5E`, the destructor-side EH count trailer**, with `0x5D` behind it.
`EH_RECORDS.md` §7.1 gives both as `<varint n> <varint state>` from the same
probes, and **`SkipForm` has no variant that can spell `<varint> <varint>`** —
which is `0xBD`'s expressiveness problem exactly, one family along. A lane
taking it inherits an enum change, not just a table row.

Second candidate: `expr-class-descriptor` (`0x66`), which `w-4c` measured at
+28,661 and which `chain_skip_form` lists as **deliberately absent**.

---

## 9. Files this lane may touch

`crates/c2-il/src/func/body/expr.rs`, `docs/IL_CALL_GRAMMAR.md`,
`docs/rungs/2026-08-08-w-5c.md`, `docs/rungs/INDEX.md`, `docs/BOARD.md`
(rows **#1423**–**#1432**), `work/w-5c/**`.

**NOT** `crates/c2-core/src/codegen/**` (lane `w-clear`),
`crates/c2-harness/src/**` or `scripts/gate.sh` (lane `w-cache`), and never
`crates/c2-core/src/codegen/coff.rs`.

Board **#1388** (`c2rs gap --cache <RELATIVE dir>` grades a byte-exact TU
`mismatch` on a cache HIT) is a landmine, not this lane's work: **every cache
path here is absolute**.
