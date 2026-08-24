# WB_RELATION — c2's relation algebra is **three 20-entry byte tables**, and every relational decision in the backend is a lookup in one of them

> **PROVENANCE — DISASSEMBLY-DERIVED.** Every address here is an absolute VA in
> `compilers/X360/16.00.11886.00/c2.dll`, sha256
> `c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`
> (**verified by this lane** against `C2_MAP_METHOD.md` §0 before the first
> grep). Nothing here has been copied into `crates/` — `w-c7` ships **zero
> `crates/` lines** — so no `DISCLOSURE.md` row is due. A lane that adopts any
> byte below owes one.

    Lane:      w-c7 (Phase 1 slice C7, compare)
    Date:      2026-08-24
    Prereg:    ../rungs/_2026-08-24-w-c7-prereg.md, clauses W1-W4
    Findings:  this file
    Method:    READ, per `CLAUDE.md` § read-before-probe. No probe grid was run
               to obtain any claim in §1-§3.

**This lane's whitebox prereg is clauses W1–W4 of the lane prereg**, not a
separate `WB_RELATION_PREREG.md`. `w-c7` is a Phase-1 *slice*, not a
characterization lane, and splitting its four location predictions out of the
one frozen file would have made them look like a second, later registration.
The pair convention (`README.md` § "The reading rule") is otherwise unchanged.

---

## 1. The headline

c2 carries **one 5-bit relation code** and **three contiguous 20-byte tables**
that transform it. The tables are adjacent in `.data`, stride `0x14`:

| VA | what it does | shape |
|---|---|---|
| **`0x10b189a4`** | **signedness remap** — signed → unsigned | idempotent; **EQ and NE are FIXED POINTS** |
| **`0x10b189b8`** | **strictness flip** — `<` ↔ `<=`, `>` ↔ `>=` | involution; EQ/NE fixed |
| ⚠ | **AMENDED — `0x10b189b8` is OPERAND EXCHANGE (reflection), not a strictness flip.** The addresses and the byte contents in this table are correct; this one row's *name* is not, and §2 uses that name as a premise. See §2's banner and `WB_RELREAD_FINDINGS.md` §3 · **#3518** | |
| **`0x10b189cc`** | **negation** — `!rel` | involution; EQ ↔ NE |

Read out of the image (20 entries each; index is `code & 0x1f`):

```text
  0x10b189a4  00 01 02 07 08 09 0a 07 08 09 0a 00 00 00 00 0f 10 11 12 00
  0x10b189b8  00 01 02 04 03 06 05 08 07 0a 09 00 00 00 00 0f 10 11 12 00
  0x10b189cc  00 02 01 06 05 04 03 0a 09 08 07 0c 0b 0e 0d 10 0f 12 11 00
```

## 2. The relation code, recovered — and it is **the IL opcode minus `0x1E`**

> ## ⚠ AMENDED 2026-08-24 by `w-relread` — **THIS SECTION'S CODE→MNEMONIC MAP IS WRONG ON 8 OF ITS 10 CODES, AND SO IS THIS HEADING**
>
> **Amended beside, not edited.** Nothing below this banner has been altered;
> the dated record stands as written. See
> [`WB_RELREAD_FINDINGS.md`](WB_RELREAD_FINDINGS.md) §1–§3 and board
> **#3517**–**#3520**.
>
> c2 carries its own **19-entry name array** for this enum at **`0x10c38690`**
> (null-bounded, pointer array into a `.text` string pool). Read out of the
> pinned image:
>
> ```text
>  0 ILLEGAL   1 EQ    2 NE    3 LT    4 GT    5 LE    6 GE
>  7 ULT       8 UGT   9 ULE  10 UGE  11 SO   12 NSO  13 S    14 NS
> 15 VALL     16 NVALL 17 VNONE 18 NVNONE
> ```
>
> **§2's table reads `3 <=, 4 <, 5 >=, 6 >` and the image reads
> `3 LT, 4 GT, 5 LE, 6 GE`** — the two orderings within each pair are swapped,
> and the same swap propagates through 7–10 via `unsigned = signed + 4`.
>
> **The heading is wrong too: `code = IL opcode − 0x1E` is FALSE.** It holds for
> `EQ` and `NE` and fails on all four orderings. Against the port's own
> `Rel::from_opcode` (`crates/c2-il/src/func/mod.rs:1411`, byte-graded, `[O]`)
> the map is a **permutation**: `0x21 Le → 5`, `0x22 Lt → 3`, `0x23 Ge → 6`,
> `0x24 Gt → 4`. The site that performs it is **still unnamed** — this lane's
> prereg W2 missed it, and `w-relread` missed it again after eliminating the
> contiguous-byte-table form image-wide.
>
> ### What SURVIVES this amendment — most of the document does
>
> The enum's **labels** are wrong. The **mechanism** is right, and every one of
> these was re-verified by `w-relread` at the addresses §1–§3 give:
>
> * **the three tables at `0x10b189a4` / `0x10b189b8` / `0x10b189cc`** — their
>   addresses, their 20-byte contents, their stride and their adjacency: **all
>   correct**;
> * **`a4` is the signedness remap and `cc` is negation**: **correct**;
> * **EQ and NE are FIXED POINTS of `a4`** — §3.1, and it *is* `#1788`: **correct
>   and label-independent**, because codes 1 and 2 are `EQ`/`NE` under both the
>   wrong assignment and the right one;
> * **§3.2 — `#423`'s "three-way interaction" is two table lookups and a zero
>   test in `FUN_10c1a908`**, with the lookups at `0x10c1a947` (`a4`),
>   `0x10c1a96d` and `0x10c1a98f` (`cc`, once per constant-zero operand, the
>   second with an exchange): **all three addresses VERIFY exactly**;
> * **§3.3's two field-replace sites** `FUN_10bd50b7` / `FUN_10bd507f`:
>   **correct**, and `0x10bd507f`'s flip of the `+0xb` flag byte really is
>   *"invert this branch"*.
>
> **What is wrong is the mapping from code to mnemonic, and one table's NAME:**
>
> * **`0x10b189b8` is OPERAND EXCHANGE (reflection), not a "strictness flip".**
>   §1's table row and §2's constraint 4 both name it strictness. Its 2-cycles
>   are `(3 4)(5 6)(7 8)(9 10)` = `LT↔GT, LE↔GE, ULT↔UGT, ULE↔UGE`, and
>   `FUN_10c1a908`'s general block implements each pair as **one emitter called
>   with the operand slots exchanged** (`0x10c1ac34`).
> * **§2's claim that codes 11–18 are "left *fixed* by both `a4` and `b8`" is
>   wrong for 11–14**: `a4[11..14] = b8[11..14] = 00`, and `00` is `ILLEGAL`.
>   Only 15–18 are fixed points of both.
> * **§5's `FUN_10c198d2`/`FUN_10c19bc0` is not the "(default)" arm** — it is
>   code **1 `EQ`**, shared with code **9 `ULE`**. `FUN_10c1a908` has **no
>   default arm and no bound check**; Ghidra's rendering of it is unusable and
>   says so.
>
> ### HOW it went wrong — neither a transcription slip nor an off-by-one
>
> §2 states the assignment is *"over-determined, which is why it can be stated
> without a probe"*, on four constraints. **Three of the four are satisfied by
> both candidate assignments.** Only constraint 4 discriminates — and it
> discriminates by **assuming what `b8` is**: *"`b8` pairs `(3 4)` and `(5 6)` —
> a **strictness** flip within one direction — which fixes the assignment the
> rest of the way."* The name "strictness flip" is a *finding* in §1 and a
> *premise* in §2. Both candidate readings of `b8` are involutions fixing
> `{1,2}`, so the tables alone cannot separate them.
>
> **And the fourth "confirmation" was constructed, not observed.** Constraint 4
> ends *"and lands it exactly on the IL's own `0x1F`..`0x24` order"* — but the
> IL opcode is a **different namespace**, and an assignment chosen to make the
> map a subtraction will land on that order by construction. A coincidence with
> another namespace was counted as evidence about this one.
>
> The general lesson, which is the transferable part: *the algebra of a
> permutation table determines the labelling only up to the **automorphisms** of
> that algebra.* This lattice has an order-2 automorphism (exchange ↔ strictness
> on `{3,4,5,6}`) that no amount of table-reading breaks. **A consumer or a name
> is required** — `w-relread` needed both.

| code | relation | IL opcode | `c2_il::Rel` |
|---:|---|---|---|
| 1 | `==` | `0x1F` | `Rel::Eq` |
| 2 | `!=` | `0x20` | `Rel::Ne` |
| 3 | `<=` signed | `0x21` | `Rel::Le` |
| 4 | `<` signed | `0x22` | `Rel::Lt` |
| 5 | `>=` signed | `0x23` | `Rel::Ge` |
| 6 | `>` signed | `0x24` | `Rel::Gt` |
| 7–10 | the same four, **unsigned** | — | (carried by `CompareLeaf::signed`) |
| 11–18 | eight further relations, negation-paired `(11 12)(13 14)(15 16)(17 18)`, left **fixed** by both `a4` and `b8` — the FP ordered/unordered set | — | not modelled by the port |

**The derivation is over-determined, which is why it can be stated without a
probe.** Four independent constraints all agree:

1. `a4` fixes exactly codes 1 and 2 and maps 3,4,5,6 → 7,8,9,10. The only two
   sign-agnostic relations are `==` and `!=`, so `{1,2} = {EQ, NE}` and
   `unsigned = signed + 4`.
2. `cc` pairs `(1 2)`. Negation maps EQ↔NE. Reflection (operand exchange) does
   **not** — it fixes EQ. So `cc` is negation, not reflection.
3. `cc` pairs `(3 6)` and `(4 5)`. Under negation `¬(a ≤ b) = a > b` and
   `¬(a < b) = a ≥ b`, which forces `{3,6} = {LE, GT}` and `{4,5} = {LT, GE}`.
4. `b8` pairs `(3 4)` and `(5 6)` — a *strictness* flip within one direction —
   which fixes the assignment the rest of the way and lands it exactly on the
   IL's own `0x1F`..`0x24` order (`crates/c2-il/src/func/mod.rs`
   `Rel::from_opcode`).

**`0x10b189cc` is used 31 times from 26 functions** across the whole backend;
`0x10b189a4` only 6 times from 5. Negation is a general service; the signedness
remap is a lowering-time decision taken in a handful of places.

## 3. The three readings

### 3.1 `#1788` is a **fixed point**, not a coincidence — `0x10b189a4`

Board `#1788` records a live wrong emit caught by writing a `_neg` fixture:
*"`int size` and `unsigned size` emit the identical `22` relational byte — the
relational opcodes are sign-agnostic — and differ only in the operand TYPE."*

That is `a4[1] = 1` and `a4[2] = 2`, and it is the *same fact* for all six
relations at once: the IL byte never carries signedness because **c2 applies
the signedness at lowering time, by table**, gated on the operand's type-class
nibble. In `FUN_10c1a908` @ **`0x10c1a908`**, the gate is one test:

```c
    local_5 = *(byte *)(param_1 + 0xd);                 /* the relation code   */
    if ((*(ushort *)((int)piVar1 + 10) & 0xf000) == 0x2000) {
        local_5 = (&DAT_10b189a4)[local_5];             /* @ 0x10c1a947        */
    }
```

`0x2000` is the type-class nibble for the unsigned integer class — the same
nibble `#2041`/`#2109` found reaching the *selector* to pick `cmpi` vs `cmpli`.
This is that nibble reaching the **relation** instead of the opcode, one step
earlier.

**`#2109` closed the black-box derivation and said the code lane needs no
address. That is still true. What the address buys is the FIXED-POINT
STRUCTURE** — the reason a future widening cannot get signedness wrong by
forgetting one relation, because the table has no per-relation exceptions.

### 3.2 `#423`'s "three-way interaction" is **two table lookups and a zero test** — `0x10c1a908`

Board `#423` is the strongest live hazard on this family:

> The six relations are **NOT one family in the guard position** — four of six
> rewrite at exactly `k = 0`, unsigned. […] a **three-way interaction of
> (relation, signedness, literal)** firing at **exactly** one literal with the
> neighbours normal — the 63-burner / 32768 shape, third recorded instance.

It was established by a **36-cell probe grid** (`work/w-cmp/gt_guard_rel.py`,
6 relations × {signed, unsigned} × `k ∈ {0,1,2}`). The read replaces the grid
with a mechanism. Immediately after the signedness remap, the same function
tests each operand for the **constant zero** and, for each one it finds,
applies the **negation** table:

```c
    local_10 = *(int **)param_1[10];                            /* operand A   */
    if (((char)local_10[2] == '\a') &&                          /* kind 7 = const */
        (local_10[6] == 0) && (local_10[7] == 0)) {             /* value 0 (64-bit) */
        local_5 = (&DAT_10b189cc)[local_5];                     /* @ 0x10c1a96d */
        local_10 = (int *)*local_10;
    }
    if ((*(char *)(puVar8 + 2) == '\a') &&                      /* operand B   */
        (puVar8[6] == 0) && (puVar8[7] == 0)) {
        local_5 = (&DAT_10b189cc)[local_5];                     /* @ 0x10c1a98f */
        puVar7 = local_c; local_c = puVar8;                     /* and exchange */
    }
```

So the "three-way interaction" is **not** a special case per relation. It is:

* the operand type-class nibble selecting `a4` (signedness), **then**
* a structural test for the literal 0 selecting `cc` (negation), **once per
  zero operand**, with an operand exchange on the second,

and the four relations that "rewrite at exactly `k = 0`" are exactly the four
that are *not* fixed points of `cc` restricted to the unsigned block —
`(7 10)` and `(8 9)`, i.e. `<=u`/`>u` and `<u`/`>=u`. `==`/`!=` are the pair
`(1 2)`, which `cc` maps onto each other rather than onto a *degenerate*
relation, which is why `#423` found `==` in the safe cell and why the eight
frontier sites it examined were safe *"only because their relation is `==`"`.

**`#423`'s warning stands in full and this read strengthens it**: the rewrite
is table-driven and unconditional, so a table-lookup lowering that skips it is
wrong for a whole *block* of the relation code space, not for four hand-listed
cells.

**What is READ and what is NOT.** The two lookups, their guards, their
addresses and the tables' contents are read. The *consumer* semantics — how
`FUN_10c1a908`'s ten-arm switch turns the normalized code into branch bytes —
is **not** read here, and the one observation that bears on it is recorded as
an observation: **switch cases 2 and 8 share their emitter**
(`FUN_10c19936` / `FUN_10c19c87`), which is the identity `x != 0` ≡ `0 <u x`
and is the fourth independent confirmation of the §2 code assignment. Reading
the ten arms is priced at ~half a day and is the natural follow-on.

### 3.3 The strictness table `0x10b189b8` is the `k ± 1` normalization the port already pays for, unnamed

`fixtures/cpp/w6_rel_k.cpp`'s header records, from live captures:

> unsigned `<` needs `k` in a register (the borrow it wants is the one out of
> `a - k`, and `subfic` only computes `SIMM - rA`), so it is four instructions
> where unsigned `>` is three […] unsigned `<=` is the only shape whose literal
> rides in a `subfic` immediate

`b8` is why there is a choice to make: `a <u k` ≡ `a <=u k-1` is a **table
lookup plus a constant adjustment**, and `FUN_10bd50b7` @ **`0x10bd50b7`**
applies it as a one-line rewrite in place:

```c
    bVar1 = *(byte *)(param_1 + 10);
    *(byte *)(param_1 + 10) = ((&DAT_10b189b8)[bVar1 & 0x1f] ^ bVar1) & 0x1f ^ bVar1;
```

— the `& 0x1f` / `^` dance is a 5-bit field replace, which is how we know the
relation code is **the low 5 bits** of a byte that carries other flags above it.

Its sibling `FUN_10bd507f` @ **`0x10bd507f`** is the same field replace through
`cc`, **plus a flip of a separate flag byte at `+0xb`**:

```c
    *(byte *)(param_1 + 10)  = ((&DAT_10b189cc)[bVar1 & 0x1f] ^ bVar1) & 0x1f ^ bVar1;
    *(byte *)(param_1 + 0xb) = *(byte *)(param_1 + 0xb) ^ 1;
```

That is *"invert this branch"* — negate the relation and swap the taken bit —
and it is the cleanest single confirmation that `cc` is negation rather than
reflection.

## 4. What this is worth, and what it is not

**Worth.** Under goal (1) this is product: three tables, four addresses, and a
relation-code assignment that is over-determined by four independent
constraints. It converts `#423` from a fitted 36-cell grid into a mechanism, and
`#1788` from a measured coincidence into a fixed point. Both were previously
recoverable only by probing.

**Not worth.** It does **not** move `#420`. The port's ceiling on this family
was measured at **6 emitted functions and 0 TUs** by this same lane
(`../rungs/2026-08-24-w-c7.md` §3) with the project's own instrument, and
knowing how c2 computes a relation does not create a body to emit it in. A
whitebox read is a hypothesis about mechanism; it is never a coverage argument.

**Also not.** Nothing here is adopted. `crates/` is byte-identical to base at
this lane's tip, so `DISCLOSURE.md` is unchanged **on purpose** — and the first
lane to bake `0x10b189a4`/`0x10b189b8`/`0x10b189cc` into a port table owes three
rows, not one.

## 5. Ranked follow-ons

> ## ⚠ AMENDED 2026-08-24 — **follow-ons 1 and 2 were TAKEN by `w-relread`; follow-on 3 was named but its meanings refused**
>
> Amended beside, not edited. See [`WB_RELREAD_FINDINGS.md`](WB_RELREAD_FINDINGS.md)
> and board **#3517**–**#3520**.
>
> 1. **`FUN_10c1a908`'s arms: READ.** Two jump tables, `T1` @ **`0x10c1ac0c`**
>    (against zero) and `T2` @ **`0x10c1ac34`** (general), each **exactly 10
>    entries indexed `code − 1`**, dispatched **with no bound check**.
>    `FUN_10c198d2`/`FUN_10c19bc0` is **code 1 `EQ`, not the default** — there is
>    no default arm. The unidentified within-pair flag is **whether the value to
>    produce when the relation is TRUE is the constant `+1`** (`−1` and every
>    non-constant share the other arm), so `FUN_10c1a908` is the lowering of
>    `rel ? A : 0`. **`#423`'s grid is retired as a DISPATCH question and NOT as
>    a byte prediction** — the emitters' bodies are unread, so a grid measuring
>    emitted size or relocations per cell still has work to do. *"Would retire
>    `#423`'s grid entirely"* was too strong.
> 2. **`#2102` vs §2: SETTLED, and against a pair that did not contain the
>    answer.** `FUN_10c1ac5c`'s terminal code is **8** (`0x10c1ac c1 / je
>    0x10c1ad10`) and **8 is `UGT`**. `#2102`'s *"ULE"* is wrong; §2's
>    *"unsigned LT"* is wrong (unsigned LT is code **7**, which `0x10c1acde`
>    converts away by exchanging the operands); **board `#2207` was right, and it
>    was already on the board when this document was written.**
> 3. **Codes 11–18: NAMED, meanings REFUSED.** `11 SO, 12 NSO, 13 S, 14 NS,
>    15 VALL, 16 NVALL, 17 VNONE, 18 NVNONE`. They are **not** "the FP
>    ordered/unordered set" as §2 calls them, and `w-relread` declines to say
>    what they *mean*: every consumer it read dispatches codes 1–10 only.

1. **Read `FUN_10c1a908`'s ten switch arms** (~½ day) — turns §3.2's "READ but
   not interpreted" into the full guard-position lowering rule, and would
   retire `#423`'s grid entirely. The arms are
   `FUN_10c198d2`/`FUN_10c19bc0` (default), `FUN_10c19936`/`FUN_10c19c87`
   (cases 2 **and** 8), `FUN_10c199bc`/`FUN_10c19d50` (3),
   `FUN_10c19a07`/`FUN_10c19da9` (4), `FUN_10c19a7f`/`FUN_10c19e9a` (5),
   `FUN_10c19af9`/`FUN_10c19f69` (6) — the pair being selected by a flag this
   lane did not identify.
2. **Reconcile `#2102` against §2.** `#2102` reads `FUN_10c1ac5c` @
   `0x10c1ac5c` as *"normalises every unsigned relation to ULE"*; under §2's
   assignment its terminal code 8 is **LT**-unsigned, not LE. One of the two is
   mis-stated and this lane did not settle which — `#2102`'s claim is
   obj-confirmed and §2's assignment is over-determined, so the likely answer is
   that `#2102`'s prose names the wrong member of the pair. **Do not quote
   either as settled until someone reads the terminal arm.**
3. **The eight FP relations (codes 11–18)** are untouched by the port and
   unnamed anywhere in this tree.
