# `w-clausefix` — PREREG

Committed **before** `work/w-inlmetric/CLAUSES.tsv` is edited and before
`work/w-inlmetric/check_table.py` is changed. Lane `w-clausefix`, wave 18,
board `#3780`–`#3785`. Base: master `4b79bf46a`.

Charter: `docs/ADOPTION_BRIEF_2026-08-28.md` §L5, discharging `w-inlfit`'s
`#3721`.

---

## §1 — What this lane owns, and what it may not touch

**Owns** (this prereg is the instrument that authorises the edit `w-inlfit`'s
own prereg correctly forbade):

* `work/w-inlmetric/CLAUSES.tsv` — the **`addr` column only**, plus one
  additive column (§4).
* `work/w-inlmetric/check_table.py`.
* `work/w-inlfit/addr_align.py` — may be folded/moved; where it went is
  recorded in the rung.
* `work/w-clausefix/`, a new `crates/c2-harness/tests/` target, board rows
  `#3780`–`#3785`, `docs/rungs/2026-08-28-w-clausefix.md`.

**Must not touch:** `docs/whitebox/ref/P_INLINE.md` (`w-inlswitch`),
`crates/c2-core/src/codegen/splice.rs` and `crates/c2-core/src/surface.rs`
(`w-inlbudget`), `crates/c2-core/src/codegen/mop.rs` (`w-encarms`),
`docs/whitebox/ref/P_GLOBREGS.md` (`w-globobj`), `docs/STATUS.md`,
`docs/rungs/INDEX.md`.

> **Noted conflict, resolved in favour of the dispatch.**
> `docs/ADOPTION_BRIEF_2026-08-28.md` §4's seam table lists `crates/` under
> this lane's *must not touch*. The dispatch brief for this lane explicitly
> assigns "a new `crates/c2-harness/tests/` target if that is the honest
> wiring". The dispatch is the later and more specific instruction, and the
> collision risk is nil because a **new file** under `crates/c2-harness/tests/`
> touches no file any peer lane owns. No existing `crates/` file is edited.

---

## §2 — The invariant this lane may not move

`CLAUSES.tsv` today reads, and must still read at this lane's tip:

```
24 rows · absent 17 · fitted 2 · R-derived 2 · unexercisable 3
reachable denominator 21 (C21–C23 are unexercisable)
```

**No `state`, `witness`, `exercised`, `note`, `owner` or `clause` cell may
change.** This lane repairs *addresses* and the *checker*. If an address repair
were to force a state change, that is a headline and gets its own board row and
a stated reason — and it is registered here as an outcome I expect **not** to
occur, because an address is not evidence about whether a port counterpart
exists.

---

## §3 — The rows I expect to repair, and what I predict each meant

**These predictions are DERIVED, not blind, and this prereg says so.** They were
read out of the independent objdump listing
(`objdump -d -M intel`, PE32 as `pei-i386` at true VAs, 425,871 instruction
starts) during re-derivation, *before any file was edited*. What is genuinely
predictive here is §3.2 and §5 — the closed row set, the untouched columns, and
the falsifier — not the ten addresses themselves. Registering a derivation is
still worth doing: it fixes the repair rule and the row set before the table
moves, so a later reader can tell a repair from a fit.

### 3.1 The repair rule, fixed in advance

For each defective row, the repaired `addr` is **the instruction that performs
the operation the `clause` column names**, decoded in the objdump listing. When
the clause names a multi-instruction sequence, the address is the **first**
instruction of that sequence that mentions the constant or the global the
clause quotes. The `owner` column is *not* adjusted to fit — if a repaired
address leaves the function `owner` names, the row is **flagged, not moved**
(§5).

### 3.2 The closed row set

`#3721` names **eight** misaligned rows: C2, C3, C4, C14, C16, C17, C18, C19.
Re-derived on this tree, `addr_align.py` reproduces exactly those eight — the
claim holds.

**But the alignment check is not the defect.** Alignment is a *necessary*
condition on a citation, not a sufficient one, and this lane predicts the
content-defect set is **larger than eight**. Registered before the edit:

| row | current `addr` | current decode | predicted repair | decode at the repair |
|---|---|---|---|---|
| C2 | `10b626d8` | *mid-instruction* (+1) | **`10b62703`** | `mov ds:0x10c3f5cc,eax` |
| C3 | `10b626f4` | *mid-instruction* (+1) | **`10b62708`** | `add eax,eax` |
| C4 | `10b6276a` | *mid-instruction* (+6) | **`10b6276e`** | `call 0x10b61ee1` |
| **C10** | `10b609d3` | `call 0x10b5e64d` — **aligned, wrong** | **`10b60a28`** | `and eax,0x2000` |
| C14 | `10b609ae` | *mid-instruction* (+1) | **`10b60a1c`** | `cmp ecx,0x10` |
| **C15** | `10b609bd` | `cmp ecx,ebx` — **aligned, wrong** | **`10b60a2f`** | `cmp edx,0xff` |
| C16 | `10b609ee` | *mid-instruction* (+3) | **`10b60a63`** | `cmp DWORD PTR ds:0x10c3f5cc,0x88b8` |
| C17 | `10b60a04` | *mid-instruction* (+2) | **`10b60a73`** | `cmp DWORD PTR [ebp+0x10],eax` |
| C18 | `10b6249b` | *mid-instruction* (+1) | **`10b625b6`** | `cmp eax,0x28` |
| C19 | `10b624a2` | *mid-instruction* (+1) | **`10b625bb`** | `sub DWORD PTR [edi],eax` |

**Ten, not eight.** C10 was filed by `w-inlfit` and not pursued. **C15 was
filed by nobody** and is this lane's own finding: `0x10b609bd` decodes to
`cmp ecx,ebx` — the POGO-record-present test — where the row claims the
`maxlevel` comparison.

The other **fourteen** rows (C1, C5, C6, C7, C8, C9, C11, C12, C13, C20, C21,
C22, C23, C24) are predicted **correct and unchanged**: each is aligned *and*
decodes to something the clause names (four of them are function entries, which
is what their clause cites).

### 3.3 The two structural predictions

* **P-A.** C18 and C19 are **not** best explained by "landed in a duplicate of
  the wrong function". Both original addresses are exactly **`0x11b` below** a
  real instruction boundary inside the correct block
  (`0x10b6249b + 0x11b = 0x10b625b6`; `0x10b624a2 + 0x11b = 0x10b625bd`), which
  is a **uniform transcription shift**, a simpler and stronger explanation than
  a coincidence at two independent sites.
* **P-B.** The alleged duplicate is real as a *structure* and false as a
  *byte* claim: `0x10b62488`–`0x10b624be` and `0x10b5fb85`–`0x10b5fbbb` differ
  in register allocation (`ecx` vs `edi`), so they are not
  "instruction-for-instruction" identical; and `0x10b62488` is inside
  `FUN_10b6242a`, which is the function the table's `owner` column **already
  names** — not a "wrong function". A **third** copy of the same idiom sits at
  `0x10b62519`, also inside `FUN_10b6242a`.

---

## §4 — The checker changes, registered

1. **Fold `addr_align.py` into `check_table.py`** as check **4. ALIGN**.
   Reason, fixed here: `w-inlfit` kept it separate *because `check_table.py`
   was another lane's frozen instrument* — a governance reason, and this prereg
   dissolves it. `#3679`'s lesson (a `scripts/` entry no funnel invokes is not
   enforcement) says two programs is two chances to run only one; and ADDRESS
   and ALIGN are two halves of a single question — *is this the address of the
   thing the clause names*. `work/w-inlfit/addr_align.py` becomes a shim that
   delegates, so `w-inlfit`'s rung citation keeps resolving and the duplicate
   implementation goes away.

2. **Add check 5. DECODE and an additive `asm` column.** C10 and C15 are the
   class the ALIGN check *cannot* reach: aligned, in the right function, wrong
   instruction. `asm` holds the objdump mnemonic text at `addr`, and DECODE
   asserts it matches. This pins the address to a decode. **It does not verify
   the clause** — for the four function-entry rows the witness is `push ebp`,
   which is weak on purpose and is documented as such in the table header.

3. **SKIP stays loud.** The objdump listing is uncommitted and regenerable.
   Absent listing ⇒ ALIGN and DECODE print `SKIP` with the path and the row
   count, and the ADDRESS/WITNESS/ABSENCE checks still run and still grade all
   24 rows. `#3470`: a clean report over zero rows is not clean.

4. **Wire it to `cargo test`.** New target
   `crates/c2-harness/tests/clause_table.rs`. **No `gate.sh` row** — `#3691`,
   a 22nd count-bearing row makes `gate_identity_diff.sh` exit 2 for every
   other live lane in this wave.

   > **Blast radius, declared and not discovered (`#3684`).** The ABSENCE check
   > greps `crates/` for tokens that must be *absent* — e.g. `INLINE_BUDGET`
   > (C3), `budget_decline` (C17), `inline_charge` (C19). `w-inlbudget` is
   > adopting `P_INLINE` §6.6.2's budget model into `splice.rs` this same wave.
   > If it names a symbol with one of those spellings, this new target goes
   > **RED on that lane's tree**. That is a **true positive** — it means the
   > table's `absent` verdict has gone stale — but the peer cannot fix it,
   > because it must not touch `CLAUSES.tsv`. The failure message therefore
   > names that case explicitly and says the remedy is a clause-state edit by
   > this table's owner, not a change to the peer's code. This is registered
   > *before* it can happen rather than explained afterwards.

---

## §5 — What I do if a row cannot be located

**Say so and mark the row. Do not invent a plausible address.** A wrong repair
to an instrument is worse than a flagged one, because everything downstream of
the inliner's conformance story is graded on this table.

Concretely: the `addr` cell is left at its current (wrong) value, the row is
listed in the rung under a heading that says *not located*, the checker's
output names it, and a board row records it. **No row's address is changed to
one that does not decode to what the clause claims**, and no clause text is
softened to fit an address that was easier to find.

---

## §6 — Controls, watched RED before any verdict is quoted (`#3336`)

`addr_align.py` was watched red in `w-inlfit`'s tree. **That does not carry**:
it is re-watched here, in its folded form, and both transcripts are committed
under `work/w-clausefix/`.

* **CTL-1 (ALIGN red).** A one-byte shift planted into a repaired row's `addr`
  ⇒ `ALIGN: RED`.
* **CTL-2 (DECODE red).** A row's `addr` moved to a *different real instruction
  boundary* in the same function ⇒ `ALIGN: GREEN`, `DECODE: RED`. This is the
  C10/C15 class and it is the control that shows ALIGN alone is insufficient.
* **CTL-3 (SKIP loud).** `C2RS_OBJDUMP_ASM` pointed at a nonexistent path ⇒
  ALIGN and DECODE `SKIP`, exit 0, **and the row count still printed**, and
  ADDRESS/WITNESS/ABSENCE still graded 24 of 24.
* **CTL-4 (green).** The repaired table ⇒ all five checks GREEN over 24 rows.
* **CTL-5 (the cargo target reddens).** The new `cargo test` target fails when
  the table is defective — watched, not assumed.

A verdict quoted before its control has been seen red is decoration.

---

## §7 — Falsifiers

This prereg is wrong if any of these turns out true, and each gets reported as
a refutation of *this document* rather than quietly dropped:

* **F1.** Fewer than eight rows are misaligned on this tree ⇒ `#3721` is stale
  and the lane reports that instead of repairing.
* **F2.** Any of the ten predicted repair addresses does not decode to what
  §3.2's right-hand column says ⇒ the row is *not located* per §5.
* **F3.** A repaired address falls outside its `owner` function ⇒ flagged, not
  moved (§3.1).
* **F4.** C15 turns out to be correct as written — i.e. `cmp ecx,ebx` at
  `0x10b609bd` *is* the maxlevel test ⇒ the "ten, not eight" headline falls and
  `#3721`'s eight stands.
* **F5.** P-A's uniform `0x11b` shift does not hold for **both** C18 and C19 ⇒
  `w-inlfit`'s duplicate-function explanation is not displaced.
* **F6.** The conformance split moves ⇒ §2's invariant is broken and the lane
  is `FAILED` unless the movement is forced and argued.

---

## §8 — Predicted reach

**Zero.** This is a characterization/instrument lane. It writes no `crates/`
*source*, proposes no `DISCLOSURE` row, adopts no constant, adds no `gate.sh`
row, and changes no clause's substantive claim. The one `crates/` file is a new
**test**, which licenses nothing.
