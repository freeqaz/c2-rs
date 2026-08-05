# The CFG step — byte-level characterization

Lane **w-cfg**. This is the specification the control-flow step is meant to be
built from: how the `.ex` encodes control flow, **what `c2` emits for each
shape**, the minimal instance to build first, and what a block/instruction IR
must carry to serve it. Every byte below is transcribed from an obj produced by
the real `cl.exe` 16.00.11886.00 / `c2.dll` under wibo, or from a `.ex` captured
at the same flags. Read-only lane: **no file under `crates/` was touched.**

The IL half is largely a **confirmation** of `docs/IL_STMT_GRAMMAR.md` §7–§9 and
of the decode-only scanner in
`crates/c2-il/src/func/body/shapes/control_flow.rs`; §2 marks every claim as
confirmed or new. The **emission half (§3–§4) is new** — nothing in `docs/`
states it today.

Control for this lane is
[`rungs/_2026-08-04-w-cfg-prereg.md`](rungs/_2026-08-04-w-cfg-prereg.md),
committed at `eefc229` **before the first capture**, with 21 predictions each
carrying a named rival reading. It is scored verbatim in §1 and the wrong ones
stay on the page.

Companion docs: `docs/IL_STMT_GRAMMAR.md` (the statement layer),
`docs/CODEGEN_W6_COMPARE.md` (the comparison *value* spines, which this document
shows are a **different** family from a comparison feeding a branch),
`docs/OBJ_DYNINIT_SHAPE.md` (the obj shell and the external-branch encoding this
document contrasts intra-function branches against), `docs/LABEL_COUNTER.md`
(the `$M`/`$T` counter, which §3.6 shows does **not** move with block count).

---

## 0. The headline, before the tables

1. **An `if` in the IL does not reliably become a branch in the obj.** c2 folds a
   large fraction of `cflow-if-1` bodies into *branchless arithmetic* or into a
   **conditional return** (`bclr`), emitting no label and no displacement at all.
   Of the seven `cflow-if-1` leaf probes in `pa.cpp`, **six** emit no forward
   branch; of the two real `cflow-if-1` functions in the frontier TU
   `src/system/utl/Pool.cpp`, **both** fold to `beqlr cr6`. An implementer who
   builds a branch lowering and grades it on `Pool.cpp` will grade nothing.
   §3.5 gives the measured fold table and says plainly that the *decision* is a
   c2 cost model this lane did not crack.

2. **The cell that does branch, and is the one to build first, is real and
   small.** `?MemFree@NUISPEECH@@YAXPAX0K@Z` in
   `src/xdk/nuispeech/xboxmem.cpp` — a frontier TU — is 0x24 bytes, nine
   instructions, one `bc`, two `REL24`, no frame, no `.pdata`. §4 specifies it
   byte for byte.

3. **The branch target is a plain self-relative displacement and carries no
   relocation.** `bc` stores `(target − addr)` in bits 16..31; an
   **intra-section** `b` stores the true relative displacement
   (`d_cold`: `48000008` at 0x38 → 0x40). This is the exact opposite of the
   **external** `b`, which stores a section-start-relative word and takes a
   `REL24` (`docs/OBJ_DYNINIT_SHAPE.md` §3.3). Same opcode, two encodings,
   discriminated by whether the target is inside this section. §3.3.

4. **Condition registers are two-valued, not one.** An explicit `cmpw`/`cmpwi`/
   `cmplwi` feeding a branch always writes **cr6** — reused, never allocated,
   confirmed across three sequential compares in one body. But a **record-form**
   instruction (`addic.`) writes **cr0**, and c2 branches on cr0 there. A
   lowering that hard-codes cr6 emits wrong bytes for every decrement-and-test
   loop. §3.2.

5. **Block order is the IL's statement order — in 10 of 11 measured cells, and
   refuted in the eleventh.** `d_join` (`if(a) r=gi(1); else r=gi(2);`) is
   tail-merged into a single `bl` with the argument selected by the branch, and
   the layout inverts. §3.4 states the rule and the refutation together.

6. **Loops are rotated, and leaf counted loops become CTR loops.** Every back
   edge measured is a **conditional** branch — never the IL's unconditional
   `3A TOP` — and a leaf loop with a compile-time trip count is lowered to
   `mtctr` + `bdnz` (`BO=16, BI=0`), which is a different instruction family
   from anything in the port today. §3.7.

7. **The long-branch expansion is measured, not assumed.** At a displacement of
   +32628 bytes c2 emits a direct `bne`; at +34148 it emits `beq cr6,+8` over an
   unconditional `b`. The switch is at the architectural 14-bit `BD` limit
   (±32764), with no slack. §3.3.1.

8. **Flag provenance, measured as a control.** `c2rs capture` hardcodes
   `/Ox /GS- /c` and silently ignores flags
   (`crates/c2-reference/src/lib.rs:465`). Every `.ex` this document quotes was
   re-captured through `c2rs census --flags-file`, and the control in §10.1
   shows the on-disk bundle reproduces byte-for-byte from that path while the
   `/Ox` capture's `.ex` **differs** — in exactly 7 bytes, one per function, the
   per-function optimization word (`0x00a00005` → `0x00200005`). **This bounds
   the exposure for this lane's measurements only. It clears no other lane's
   captures.**

9. **What this document does not contain: an optimizer.** The port is
   I/O-behavioral. §3.5's fold table and §3.4's tail-merge refutation are
   recorded as *required emission rules for the accepted class* and as *reasons
   to keep the class narrow* — never as passes to reproduce in general. §8 is
   the list of what is still unbuildable, and it is long on purpose.

## 1. Pre-registration, scored

Verbatim from `rungs/_2026-08-04-w-cfg-prereg.md`. Bias registered in advance:
*"I want the answer to be — c2 emits basic blocks in the IL's own order, with the
branch sense inverted at `38`, and the target as a plain self-relative
displacement with no relocation."* Three of those four came out right, which is
why the two wrong predictions below are the load-bearing part of this section.

**14 clean right · 3 half · 2 wrong · 1 right-with-a-refuting-cell · 1 right
only because a held-out probe answered what the grid could not.**

### A — the branch instruction and the condition register

| # | prediction | verdict |
|---|---|---|
| A1 | compares feeding a branch always use **cr6**, even multi-compare | **HALF.** cr6 ✓ and *reused*, never allocated — `?b_ifn` writes cr6 three times in one body, so the registered rival R-A1 (CR allocation like GPR temps) is refuted. But "always" is **false**: a record-form `addic.` writes **cr0** and c2 branches on cr0 (`?c_do`, `?d_break`, `?c_callloop`). The rule is two-valued (§3.2) |
| A2 | `bc` op16, `AA=0 LK=0`, `BO ∈ {4,12}`, `BI ∈ {24..27}`; **no** CTR-decrement `BO` | **HALF.** Exactly right for every branch on a compare. Refuted in scope by the thing I said would not appear: leaf counted loops emit **`bdnz`, `BO=16, BI=0`** (§3.7) |
| A3 | sense = **negation** of the IL relation at `38`, the relation itself at `39` | **RIGHT.** `MemFree` (`1f` EQ + `38`) → `bne cr6`; `d_cold` (EQ + `38`) → `bne cr6`; `b_or` (`39` on a bare value) → `bne cr6`. R-A3 (one sense, swapped successors) refuted |
| A4 | `cmpwi`/`cmplwi` for an i16 literal, `cmpw`/`cmplw` for a register; signedness from the operand type triple | **RIGHT.** `cmplwi cr6,r3,0` for a pointer operand, `cmpw cr6,r3,r4` for two int registers, `cmpwi cr6,r3,7` for a literal |
| A5 | the compare is **fused into the branch**; the W6 branchless spines never appear when a comparison feeds `38`/`39` | **WRONG — and the registered rival was wrong too.** `a_eq`, `a_eqk`, `a_else`, `d_early`'s tail and `f_eqzk` all emit a branchless *arithmetic select* for the whole `if` statement. Neither "fused branch" nor R-A5's "materialize then `cmpwi 0`" is what happens; the third reading is §3.5, and finding it is what this cell bought |

### B — targets, fixup, relocations

| # | prediction | verdict |
|---|---|---|
| B1 | intra-function target is a plain self-relative displacement in the word; **no relocation** | **RIGHT**, and refined: an intra-section **`b`** also stores the true displacement (`48000008`), where an *external* `b` stores the section-start-relative word plus `REL24`. R-B1 refuted |
| B2 | `pa.cpp` `.text` relocations = **0**; `pb.cpp` = one `REL24` per emitted call site | **RIGHT.** All seven `pa.cpp` code sections report `nrel=0` despite carrying branches; `pb.cpp`'s counts match its call sites exactly |
| B3 | every branch in the grid fits `BD`; no long-branch expansion needed | **RIGHT as stated** — and worth nothing on its own. The held-out `pe.cpp` probe forced the expansion and **answered** it (§3.3.1); without it the document would have shipped silent on the case |
| B4 | labels mint **no** symbol records; `$M`/`$T` count does not track block count | **RIGHT, strongly.** Leaf bodies with three branch targets (`d_early`, `d_switch`) carry **zero** `LABEL` symbols; framed bodies carry exactly **two** `$M` whether they have two blocks (`d_goto`) or four (`d_cold`). R-B4 refuted |

### C — block order

| # | prediction | verdict |
|---|---|---|
| C1 | blocks land in `.text` in **IL statement order**; `b_ifelse` emits `g` before `h` | **RIGHT IN 10 OF 11 CELLS, REFUTED IN THE ELEVENTH.** Holds for `b_ifelse`, `b_ifval`, `b_if2`, `b_ifn`, `d_early`, `d_goto`, `d_cold`, `f_eqcall`, `MemAlloc`, `MemFree`, `MemSize`. Refuted by **`d_join`**: the two arms' identical `bl gi` are **tail-merged**, the then-block empties, and the layout inverts so the *else* block is the fall-through. R-C1 partially vindicated (§3.4.1) |
| C2 | the join `b` is elided on fall-through; a final `return`'s `3A` emits nothing | **RIGHT** |
| C3 | `while` rotates: top guard, body, **conditional** backward branch; no unconditional `b` back-jump | **HALF.** The back edge is conditional in **every** loop measured — never an unconditional `b` — so the core claim holds and R-C3 (literal IL emission) is refuted. But the *top* is not always a guard: `d_cont` emits an unconditional **forward** `b` into the test. And the unpredicted finding: leaf counted loops become **CTR loops** (§3.7) |
| C4 | `for` is straightened; the IL's `b INCR` disappears and the increment moves below the body | **RIGHT.** `c_for` and `c_forcall` each emit a single back edge with the increment below the body. A port emitting the IL's block order literally here would emit wrong bytes, exactly as registered |
| C5 | `&&`/`||` emit one `bc` per operand, both to the same label; no `crand`/`cror` | **RIGHT.** `b_and` and `b_or` emit two `cmpwi`+branch pairs and no CR-logic instruction |
| C6 | the epilogue is emitted once, at the end; non-final returns become a forward `b` to it unless folded to `bclr` | **RIGHT, and refined the expensive way:** the epilogue block is emitted **even when it is unreachable**. `b_if`, `b_and` and `b_or` each end in a dead `4e800020` that no path can reach, because every path to it folded into a `bclr`. An emitter that drops it is short four bytes |

### D — the folding hazard the bias pointed at

| # | prediction | verdict |
|---|---|---|
| D1 | **at least three** of the seven `pa.cpp` leaves emit no label and no displacement | **RIGHT, and understated: six of seven.** Only `a_var` emits a `bc` |
| D2 | `a_eq` emits exactly 20 B: `7f 03 20 00 · 38 60 00 01 · 4d 9a 00 20 · 38 60 00 02 · 4e 80 00 20` | **WRONG — and R-D2 (the label form) also WRONG.** Actual: 24 B of *branchless arithmetic*, `7d 63 20 50 · 7d 6b 00 34 · 55 6b df fe · 69 6b 00 01 · 38 6b 00 01 · 4e 80 00 20`. **This is the cell that found the fold**, and it found it precisely because both registered readings were branch-shaped. A prediction with no rival here would have shipped a branch spec for a body that does not branch |
| D3 | `b_if` is a **leaf**: `cmpwi cr6,r3,0 ; beqlr cr6 ; b g`, one `REL24`, no frame | **RIGHT** — and one thing unpredicted: a dead `blr` follows (C6) |

### E — the listing seam, and instrument controls

| # | prediction | verdict |
|---|---|---|
| E1 | `/FAsc` prints `$LN<k>` labels, and the listing's block order equals the obj's byte order | **RIGHT** — unlike *section* order, which `docs/OBJ_DYNINIT_SHAPE.md` §6 measured as disagreeing |
| E2 | the listing prints the **canonical** word for an external `b`/`bl` but the **real** word for an intra-function `bc` | **RIGHT, exactly.** `MemFree`'s listing shows `409a0010` (the obj word) for the `bc` and `48000000` for the tail `b`, where the obj carries `4bffffec` |
| E3 | `c2rs census` reports the shape each probe was written to have | **RIGHT.** `pa` 7×`cflow-if-1`; `pb` 3/3/1 `if-1`/`if-2`/`if-n`; `pc` 5×`cflow-loop`. The probes are the shapes claimed, checked rather than presumed |

### 1.1 What the scoring bought

The two wrong predictions (A5, D2) are the same finding twice, and it is the
finding that changes the implementation order: **the shape named `cflow-if-1` in
the census is not the shape "a conditional branch" in the obj.** Both were
registered with rivals; both rivals were also branch-shaped; and it took the
*byte* answer to produce the third reading. The sibling lane's observation
holds here — the predictions that would have silently misled a writer are
exactly the ones whose registered alternative was still inside the wrong family.

## 2. How the IL encodes control flow

Almost all of this is **confirmation**, not discovery: `docs/IL_STMT_GRAMMAR.md`
§7–§9 characterized it and
`crates/c2-il/src/func/body/shapes/control_flow.rs` implements it as a
decode-only scanner. It is restated here because the emission half in §3 is
unreadable without it, and because a spec that sends an implementer to two
documents for one production is a spec that will be half-read. Each claim is
tagged **[C]** confirmed-this-lane or **[N]** new-this-lane.

### 2.1 The four opcodes, and that is the whole vocabulary

```text
29 <tok>     define label <tok>                                       [C]
38 <tok>     pop a value; branch to <tok> if it is FALSE              [C]
39 <tok>     pop a value; branch to <tok> if it is TRUE               [C]
3A <tok>     unconditional branch to <tok>                            [C]
```

`<tok>` is a `read_token_var` LEB-ish token, 2 bytes normally and 4 when bit 7 of
the second byte is set. **All of `if`, `if/else`, `&&`, `||`, `!`, `while`,
`for`, `do`/`while`, `break`, `continue`, `goto` and early `return` are built
from these four and nothing else** — `IL_STMT_GRAMMAR.md` §8.4's result, and it
holds on every probe and every frontier function this lane read. `switch` adds
`3B`/`3C`/`3D` and is a different problem (§5.4).

Three properties that decide the shape of a decoder, all **[C]**:

* **`3A` carries no direction.** Forward and backward are decided by where
  `29 <tok>` happens to sit, so a back edge is only visible after a position
  scan. This is why `control_flow.rs`'s `Site` records an offset and not a count.
* **A label may be defined at a shallower scope depth than the branches that
  target it** (`do`/`while`, `IL_STMT_GRAMMAR.md` §8.3), so the label table must
  not be scoped.
* **The branch condition is not a separate materialization step.** The
  comparison feeds the branch directly, with no `2C` convert — contrast the W6
  comparison *leaf*, which converts bool→int because it returns the value.

### 2.2 The blocker vocabulary, decoded

The census keys the brief names, resolved against the bytes that produce them.
`cflow-*` come from `CfShape::name`; `expr-*` and `cf-expr-*` are the *first
refused byte* in the operand walk.

| key | what it is in the byte stream |
|---|---|
| `cflow-if-1` | body decoded end to end; **one** `38`/`39` site; no back edge; no `3B`/`3C`/`3D` |
| `cflow-if-2` | as above with **two** `38`/`39` sites |
| `cflow-if-n` | as above with **three or more** |
| `cflow-loop` | some `38`/`39`/`3A` names a label whose `29` **already went past** — a back edge. Ranked above `if-n` however few conditionals it has, because the back edge is the expensive fact |
| `cflow-straight` | ≤1 `3A` and ≤1 `29`, i.e. the epilogue's own pair, and no conditional — a single basic block |
| `cflow-multi-exit` | no conditional but >1 jump or label — several `return`s converging on the epilogue |
| `expr-brfalse` / `expr-brtrue` | the **accepting** parser (not the scanner) stopped at a `38` / `39` it cannot lower |
| `expr-jump` / `expr-label` / `body-cflow-label` | likewise at `3A` / `29`, the last at a `29` in body-leading position |
| `cf-expr-0x05` | **not control flow at all.** The decode-only *scanner* stopped at operand byte `0x05`, which `docs/CODEGEN_W6_COMPARE.md` §1.2 pins as **DIV**. A body filed here has an unknown-width token before its control flow was ever read, so its `cflow-*` class is unmeasured |

**`cf-expr-0x05` is the trap in this table.** Three of the seventeen frontier
TUs (`Biquad.cpp`, `wordwrap.cpp`, `Pool.cpp`) contain a function filed there,
and it is *not* a control-flow blocker — it is an integer divide whose operand
width the scanner declines to guess. Those three TUs are unreachable by the CFG
step no matter how complete it is (§5.5).

### 2.3 The shapes, from the probes' own bytes

Captured at the workload flags (§10.1), body shown from the `4C 4F 11` marker.

**`cflow-if-1`, the whole of it** — `?a_lt@@YAHHH@Z`,
`int a_lt(int a,int b){ if(a<b) return 1; return 2; }`:

```text
4c 4f 11              body marker
53                    open body scope            (depth 3)
53                    open the if-statement's own scope   (depth 4)   [see below]
b9 ed 09 86 41 74     LOAD  a   int
b9 ee 09 86 41 74     LOAD  b   int
22                    CMP LT                     -> bool
38 f1 09              brFALSE -> L_09F1   (the else entry)
53                      open the then-clause scope (depth 5)
33 86 41 74 01          LIT int 1
41 86 41 74             RESULT int
3a f0 09                JUMP -> L_09F0  (the epilogue label) = `return 1;`
54 04                   close the then-clause      (4 remaining)
29 f1 09              L_09F1:
54 03                 close the if scope           (3 remaining)
33 86 41 74 02        LIT int 2
41 86 41 74           RESULT int
3a f0 09              JUMP epilogue   = `return 2;`
54 02                 close the body               (2 remaining)
29 f0 09              L_09F0:   the epilogue label
4f 12 47 54 01 54 00  function tail
```

Everything an `if` needs is in those twelve lines: **one conditional branch, one
label definition, and the epilogue label that every `return` jumps to.** **[C]**

**`if/else` adds one jump and one label** — `?a_else@@YAHHH@Z`, the delta against
the above being `54 04 · 3a fb 09 · 29 fa 09` where `a_lt` has `54 04 · 29 f1
09`: the then-clause's close is followed by a `3A` **over** the else arm, and the
join label `29 fb 09` is defined after the else arm closes. Note the jump is
emitted *after* the then-clause's `54`, in the `if` statement's own scope. **[C]**

**`cflow-if-2` is two independent `38` sites and nothing else** —
`?b_if2@@YAXHH@Z`, `void b_if2(int a,int b){ if(a) g(); if(b) h(); }`:

```text
53 53  b9 ff 09 86 41 74  38 03 0a   53 26 e3 09 bd … 4c 4b  54 04  29 03 0a  54 03
       53  b9 00 0a 86 41 74  38 04 0a   53 26 e4 09 bd … 4c 4b  54 04  29 04 0a  54 03
       3a 02 0a  54 02  29 02 0a  4f 12 47 54 01 54 00
```

The second `if` is a byte-for-byte repetition of the first with different tokens.
`cflow-if-n` (`?b_ifn@@YAXHHH@Z`) is the same production a third time.
**Structurally, `if-2` and `if-n` add nothing over `if-1`** — no new opcode, no
new nesting rule, no new join discipline. **[N]** — the *implication* is new even
though the bytes confirm §7; see §5.2, where it is what makes the widening order
`if-1 → if-n` rather than `if-1 → if-2 → if-n`.

**A short-circuit `&&` is already two branches in the IL** —
`?b_and@@YAXHH@Z`, `if(a && b) g();`:

```text
b9 f4 09 86 41 74  38 f8 09        brFALSE(a) -> SKIP
b9 f5 09 86 41 74  38 f8 09        brFALSE(b) -> the SAME label
53 26 e3 09 bd … 4c 4b  54 04
29 f8 09                           SKIP:
```

and `||` (`?b_or@@YAXHH@Z`) is `39` to the *entry* then `38` to the skip:

```text
b9 f9 09 86 41 74  39 fe 09        brTRUE(a)  -> L_09FE (the call)
b9 fa 09 86 41 74  38 fd 09        brFALSE(b) -> L_09FD (skip)
29 fe 09                           L_09FE:
53 26 e3 09 bd … 4c 4b  54 04
29 fd 09                           L_09FD:
```

The `0x1A`/`0x1B`/`0x1C` operator bytes for `!`/`||`/`&&` **do not appear in a
condition** — c1xx lowered them to branches already. **[C]** Note the census
files these as `cflow-if-2`, not as a distinct shape, which is correct: they are
two branches. **[C]**

**A loop is one `29` whose position precedes a branch that names it** —
`?c_callloop@@YAXH@Z`, `void c_callloop(int n){ while(n){ g(); n=n-1; } }`:

```text
53 53
29 00 0a                        TOP:
b9 fc 09 86 41 74  38 01 0a     brFALSE -> EXIT
53  26 e3 09 bd … 4c 4b         g();
    26 fc 09 b9 fc 09 86 41 74 33 86 41 74 01 03 32 86 41 74 4b    n = n - 1;
54 04
3a 00 0a                        JUMP TOP   <- the back edge, an UNCONDITIONAL 3A
29 01 0a                        EXIT:
54 03  3a fe 09  54 02  29 fe 09  4f 12 47 54 01 54 00
```

The back edge in the **IL** is an unconditional `3A`. §3.7 shows it is *never* an
unconditional branch in the obj — which is the single largest gap between the IL's
block structure and the emitted one. **[N]**

**`for` arrives pre-rotated, and not in source order** — `?c_for@@YAHH@Z`
reproduces `IL_STMT_GRAMMAR.md` §8.2 exactly: `init · 3a COND · 29 INCR · incr ·
29 COND · cond · 38 EXIT · body · 3a INCR · 29 EXIT`. The increment sits
**before** the condition in the byte stream and runs after it. **[C]** §3.7 shows
c2 straightens this away entirely, so a lowering that preserves IL block order
emits wrong bytes here specifically.

### 2.4 The IL is flag-invariant for this class — measured

The `.ex` for `pa.cpp` at `/Ox /GS- /c` and at the workload's
`/O1 /Oi /EHsc /GR …` are **the same length and differ in exactly 7 bytes**, one
per function, at the per-function optimization word `4F 1F 80 05 00 <b> 00`:
`b = a0` at `/Ox` (`OPT_WORD_OX = 0x00a00005`), `b = 20` at `/O1`
(`OPT_WORD_O1 = 0x00200005`). **Every statement and expression byte quoted in
this section is identical under both.**

That is a useful and a dangerous fact at once. Useful: the control-flow *decode*
is flag-independent for this class, so a decoder graded at one setting is graded
at both. Dangerous: it means a cross-flag capture **looks fine** on the IL side
and is wrong only on the obj side — which is exactly the exposure §10.1 exists to
close, and precisely why the control was run rather than assumed.

## 3. What c2 emits

All bytes below are read off objs built with
`c2rs compile --flags-file` at the flags in §10.1. Words are big-endian, offsets
are relative to the start of the function's own `.text` COMDAT section — which,
under `/O1` (⇒ `/Gy`), is **one section per function**, so "section offset" and
"function offset" are the same number. That matters for §3.3: an intra-function
branch is also an intra-section branch, always.

### 3.1 The branch instruction forms, encoded

Four forms cover every branch this lane measured.

| form | word | fields |
|---|---|---|
| conditional, to a label | `0x40000000 \| (BO<<21) \| (BI<<16) \| (BD & 0xFFFC)` | op 16, `AA=0`, `LK=0`; `BD` a **signed 14-bit** displacement in bits 16..29 |
| conditional return | `0x4C000000 \| (BO<<21) \| (BI<<16) \| 0x20` | op 19, `XO=16` (`bclr`), `LK=0` |
| unconditional, intra-section | `0x48000000 \| (LI & 0x03FFFFFC)` | op 18, `AA=0`, `LK=0` |
| unconditional / linking, **external** | `0x48000000 \| ((-k) & 0x03FFFFFC) \| LK` | `k` = the branch's own section offset; the target comes from a `REL24` |

`BO` and `BI` for a branch on a compare:

```text
BO = 12   branch if the CR bit is SET     ("branch true")
BO =  4   branch if the CR bit is CLEAR   ("branch false")
BI = 4*crf + { 0 = LT, 1 = GT, 2 = EQ, 3 = SO }
```

so with `crf = 6`: `BI = 24` LT, `25` GT, `26` EQ, `27` SO. Every conditional
branch measured in this lane uses one of exactly these eight `(BO,BI)` pairs —
**except** the CTR loops of §3.7, which use `BO = 16, BI = 0`.

Worked, from `?MemFree@NUISPEECH@@YAXPAX0K@Z` at offset 0x08, target 0x18:

```text
BO = 4, BI = 26, BD = 0x18 - 0x08 = +16
0x40000000 | (4<<21) | (26<<16) | 0x0010
  = 0x40000000 | 0x00800000 | 0x001A0000 | 0x0010
  = 0x409A0010                                  <- the obj carries 409a0010  ✓
```

The observed conditional-return words, for reference — these are *constants* once
the relation is known and are worth table-driving rather than encoding:

| mnemonic | word | `(BO,BI)` | seen in |
|---|---|---|---|
| `beqlr cr6` | `4d 9a 00 20` | (12,26) | `b_if`, `b_and`, `Pool::Alloc`, `Pool::Free`, `f_eq59` |
| `bnelr cr6` | `4c 9a 00 20` | (4,26) | `a_store`, `f_ne59`, `f_eqvoid` |
| `bltlr cr6` | `4d 98 00 20` | (12,24) | `a_lt` |
| `bgtlr cr6` | `4d 99 00 20` | (12,25) | `f_gt59` |
| `blelr cr6` | `4c 99 00 20` | (4,25) | `c_for` |
| `blr` | `4e 80 00 20` | (20,0) | everywhere |

Note `blr` is the same instruction with `BO = 20` — "branch always" — so an
emitter that builds `bclr` from `(BO,BI)` gets the plain return for free, and one
that special-cases `4e800020` has two code paths for one instruction.

### 3.2 The condition register is two-valued

**Rule, measured.** An explicit compare feeding a branch writes **cr6**. A
**record-form** arithmetic instruction writes **cr0**, and c2 branches on cr0
there without an intervening compare.

| body | compare | branch | CR |
|---|---|---|---|
| `?MemFree` | `2b 03 00 00` `cmplwi cr6,r3,0` | `409a0010` `bne cr6` | **cr6** |
| `?b_ifn` (three sequential ifs) | `2f 03 00 00`, `2f 1f 00 00`, `2f 1e 00 00` — all `cr6` | three `419a0008` | **cr6, reused** |
| `?c_do` | `35 6b ff ff` `addic. r11,r11,-1` | `4082fff8` `bne cr0` | **cr0** |
| `?c_callloop` | `37 ff ff ff` `addic. r31,r31,-1` | `4082fff0` `bne cr0` | **cr0** |
| `?d_break` | `37 ff ff ff` `addic. r31,r31,-1` | `4082fff0` `bne cr0` | **cr0** |
| `?d_cont` | `2f 1f 00 00` `cmpwi cr6,r31,0` | `409affec` `bne cr6` | **cr6** |
| `?c_forcall` | `7f 1f e8 00` `cmpw cr6,r31,r29` | `4198ffec` `blt cr6` | **cr6** |

`?b_ifn` is the cell that refutes CR **allocation**: three compares live in one
body, all three write cr6, each branch consuming its own before the next is
issued. c2 does not treat CR fields the way it treats GPR temps (which descend
from r11 — `docs/CODEGEN_W6_COMPARE.md` §6).

The cr0 rows are all the same construct: a loop counter decremented with
`addic.` where the decrement's own condition code is the loop test. That is a
**fused compare**, and it is why `BI = 2` (cr0's EQ bit) appears in the branch
word `4082fff8` rather than `BI = 26`.

> **Hazard.** A lowering that hard-codes `BI = 4*6 + bit` emits `409a…` where the
> obj has `4082…` for every decrement-and-test loop — a two-byte difference in a
> word that still disassembles to a plausible branch. This is exactly the
> fuzzy-invisible class `docs/CODEGEN_PPC_MVP.md` warns about.

Compare-instruction selection **[C]**, unchanged from
`docs/CODEGEN_W6_COMPARE.md` §1.1's signedness rule:

| operand form | instruction | witness |
|---|---|---|
| register vs register, signed | `cmpw cr6,rA,rB` = `7f 03 20 00` (r3,r4) | `a_lt`, `f_eq59` |
| register vs i16 literal, signed | `cmpwi cr6,rA,k` = `2f 03 00 07` (r3,7) | `d_switch` |
| register vs literal, **unsigned/pointer** | `cmplwi cr6,rA,k` = `2b 03 00 00` (r3,0) | `MemFree`, `Pool::Free` |
| register vs register, unsigned | `cmplw cr6,rA,rB` | — *unvaried in this grid* |

The signedness comes from the shared operand type triple at the comparison, not
from the comparison opcode — `86 41 74` (int) → `cmpw`/`cmpwi`, `86 43 83 08`
(pointer) → `cmplwi`. A pointer null-check is therefore an **unsigned** compare,
which is what `MemFree` and both `Pool.cpp` functions emit.

### 3.3 Targets and fixup

**`bc`.** `BD = target_offset − branch_offset`, stored in bits 16..31 of the
word with the low two bits zero. It is **not** relative to the section start, it
is relative to the branch instruction itself. **No relocation record is emitted.**

**Intra-section `b`.** The same rule with the wider `LI` field:
`?d_cold` at 0x38 branches to the epilogue at 0x40 and stores `48000008` —
`LI = +8`, the true displacement. **No relocation.**

**External `b` / `bl`.** The word stores `(−k) & 0x03FFFFFC` where `k` is the
branch's own section offset, i.e. the encoded target is the section start, and a
`REL24` supplies the real one. `?MemFree`'s tail call at 0x14 stores `4bffffec`
(`LI = −20 = −0x14`) and at 0x20 stores `4bffffe0` (`LI = −32 = −0x20`). This
reproduces `docs/OBJ_DYNINIT_SHAPE.md` §3.3 exactly, on ordinary functions.

> **The discriminator is the target, not the opcode.** `48000008` and `4bffffec`
> are the same instruction. The first is an intra-section jump carrying its own
> displacement; the second is an external call carrying a placeholder plus a
> relocation. An emitter must decide which it is from *where the target lives*,
> and a fixup pass that treats every `b` alike will corrupt one of the two.

**Relocation counts, measured.** `pa.cpp`: seven code sections, every one
`nrel = 0`, despite six of the seven containing a branch and one containing a
`bc` with a real displacement. `pb.cpp`: `nrel` equals the number of emitted
call sites in every section (1,2,2,1,1,2,3). **Branches never contribute a
relocation; only calls to symbols do.**

#### 3.3.1 The long-branch expansion — measured, with the threshold bracketed

`BD` is a signed 14-bit field scaled by 4, so a `bc` reaches ±32764 bytes.
Held-out probe `pe.cpp` is `if(a==0){ <N volatile stores> return b; } return b+1;`
with `N` swept:

| N | displacement needed | what c2 emits at the branch |
|---:|---:|---|
| 4000 | +31176 | `409a79c8` — direct `bne cr6, +31176` |
| 4200 | **+32628** | `409a7f74` — direct `bne cr6, +32628` |
| 4400 | **+34148** | `419a0008` `beq cr6,+8` **then** `48008564` `b +34148` |
| 6000 | +46708 | `419a0008` then `4800b674` |

So the expansion is: **invert the condition, branch over an unconditional `b`,
and put the far target on the `b`.** The transition sits between +32628 and
+34148, i.e. at the architectural limit of ±32764 with **no slack** — c2 uses the
full field before expanding. Two instructions, never a register-indirect
`bcctr` form.

This is not a case any current probe or frontier function needs, and it is
recorded for exactly that reason: an emitter that never checks the range
produces a truncated `BD` on the first function that exceeds it, and a truncated
`BD` is a legal-looking branch to the wrong place.

### 3.4 Block order

**Rule.** Blocks land in `.text` in the order their statements appear in the
`.ex` stream. For an `if`/`else` that is: the condition, the `bc` to the *else
entry*, the then-block, a `b` to the join, the else-block, the join. The branch
sense is the negation of the IL relation (because `38` is brFALSE), so the `bc`
is the edge to the **else**, and the **then** block is the fall-through.

Ten cells, every one consistent:

| body | layout, by ascending offset | branch |
|---|---|---|
| `?b_ifelse` | `cmpwi` · `bc`→else · `b g` · `b h` | `419a0008` beq→0x0c |
| `?b_ifval` | `cmpwi` · `bc`→else · `li r3,1 ; b gi` · `li r3,2 ; b gi` | `419a000c` beq→0x10 |
| `?b_if2` | prologue · if₁ · `bl g` · if₂ · `bl h` · epilogue | two `419a0008` |
| `?b_ifn` | prologue · if₁ · `bl g` · if₂ · `bl h` · if₃ · `bl g` · epilogue | three `419a0008` |
| `?d_early` | if₁ · `li 1 ; blr` · if₂ · `li 2 ; blr` · tail | two `419a000c` |
| `?d_goto` | `cmpwi` · `bc`→over · `bl g` · `bl h` · epilogue | `409a0008` bne→0x18 |
| `?d_cold` | `cmpwi` · `bc`→else · **6×`bl h`** · `addi r3,r31,1` · `b` join · `addi r3,r31,2` · epilogue | `409a0024` bne→0x3c |
| `?f_eqcall` | `cmpw` · `bc`→else · `li 5 ; b gi` · `li 9 ; blr` | `409a000c` bne→0x10 |
| `?MemAlloc` | setup · `cmplwi` · `bc`→else · then(2) · `b XMemAlloc` · else(2) · `b RtlAllocateHeap` | `409a000c` |
| `?MemFree` | setup · `cmplwi` · `bc`→else · then(3) · `b XMemFree` · else(2) · `b RtlFreeHeap` | `409a0010` |

`?d_cold` is the cell that matters most for the rule's strength: its then-block
is six calls long and c2 still leaves it **in line**, as the fall-through, and
branches *over* it to the else. There is no out-of-lining of a cold arm and no
"shorter arm falls through" heuristic at `/O1` — registered rival **R-C1**
predicted one and it does not happen.

The join `b` is present only when it is needed. `?d_cold` emits `48000008` to
skip the else block; `?b_ifelse`, `?b_ifval`, `?MemFree` emit nothing at the end
of their then-block because it ends in a tail call, and `?d_early` emits nothing
because its then-block ends in `blr`.

#### 3.4.1 The refutation — `?d_join`, and what it costs

`int d_join(int a,int b){ int r; if(a) r=gi(1); else r=gi(2); return r+b; }`

```text
0010  2f030000  cmpwi cr6,r3,0
0014  7c9f2378  mr    r31,r4          ; b, live across the call
0018  38600001  li    r3,1            ; the THEN block's argument, HOISTED above the branch
001c  409a0008  bne   cr6,0x24        ; a != 0 -> take the THEN edge
0020  38600002  li    r3,2            ; the ELSE block, at the fall-through
0024  4bffffdd  bl    ?gi@@YAHH@Z     ; ONE call, shared by both arms
0028  7c63fa14  add   r3,r3,r31
```

Two things happened that §3.4's rule does not describe. c2 **tail-merged** the
two identical `bl gi` sites into one, which emptied the then-block down to a
single `li r3,1`; and it then **hoisted that `li` above the compare**, leaving
the then-block genuinely empty. With an empty then-block the natural layout
inverts: the fall-through becomes the *else*, and the `bc` carries the *then*
edge — note the sense is `bne` (branch-if-`a`-true) where every other cell in
§3.4 emits the negation.

**This is one cell out of eleven and it is stated rather than rounded away.**
What it establishes is not a competing layout rule; it is that **block order is
downstream of code motion**, and code motion is a c2 pass this document does not
and will not characterize. §8 carries it as an explicit limit on the accepted
class: a body whose arms end in the *same* call is outside anything specified
here.

### 3.5 The fold table — when a `cflow-if-1` emits no branch at all

This is the section that changes the implementation order, and it is the one
whose *rule* this lane did not crack. What follows is the measured table, then
an honest statement of what decides it.

| body | condition | arms | emitted | branch? |
|---|---|---|---|---|
| `?a_eq` | `a==b` | `return 1 : 2` | `subf r11,r3,r4 ; cntlzw r11,r11 ; rlwinm r11,r11,27,31,31 ; xori r11,r11,1 ; addi r3,r11,1 ; blr` | **none** |
| `?a_ne` | `a!=b` | `1 : 2` | as above minus the `xori` | **none** |
| `?a_eqk` | `a==7` | `1 : 2` | `addi r11,r3,-7` then the same tail | **none** |
| `?a_else` | `a==b`, explicit `else` | `1 : 2` | **byte-identical to `?a_eq`** | **none** |
| `?d_early` (3rd if) | `c!=0` | `3 : 4` | `cntlzw ; rlwinm ; addi r3,r11,3` | **none** |
| `?f_eqzk` | `a==0` | `5 : 9` | `subfic r11,r3,0 ; subfe r11,r11,r11 ; rlwinm r11,r11,0,29,29 ; addi r3,r11,5` | **none** |
| `?d_switch` (last case) | `a==7` | `70 : 0` | `addi ; li ; addic ; subfe ; and r3,r11,r10` | **none** |
| `?a_lt` | `a<b` | `1 : 2` | `cmpw cr6,r3,r4 ; li r3,1 ; bltlr cr6 ; li r3,2 ; blr` | **`bclr`** |
| `?f_eq59` | `a==b` | `5 : 9` | `cmpw ; li r3,5 ; beqlr cr6 ; li r3,9 ; blr` | **`bclr`** |
| `?f_gt59` | `a>b` | `5 : 9` | `cmpw ; li r3,5 ; bgtlr cr6 ; li r3,9 ; blr` | **`bclr`** |
| `?f_eq3` | `a==b` | `return c : 9` | `cmpw ; mr r3,r5 ; beqlr cr6 ; li r3,9 ; blr` | **`bclr`** |
| `?a_store` | `a==0` | `*p=1` / nothing | `cmpwi ; bnelr cr6 ; li r11,1 ; stw r11,0(r4) ; blr` | **`bclr`** |
| `?f_eqvoid` | `a==b` | two stores / nothing | `cmpw ; bnelr cr6 ; li ; li ; stw ; stw ; blr` | **`bclr`** |
| `?Pool::Alloc` | `p==0` | early return | `cmplwi ; beqlr cr6 ; lwz ; stw ; blr` | **`bclr`** |
| `?Pool::Free` | `p==0` | early return | `cmplwi ; beqlr cr6 ; lwz ; stw ; stw ; blr` | **`bclr`** |
| `?a_var` | `a==b` | `r=1` / `r=2`, joined | `li r11,2 ; cmpw ; bne cr6,+8 ; li r11,1 ; mr r3,r11 ; blr` | **`bc`** |
| `?f_eqcall` | `a==b` | `return gi(5) : 9` | `cmpw ; bne cr6,+12 ; li r3,5 ; b gi ; li r3,9 ; blr` | **`bc`** |
| `?MemFree` | `v1==0` | two different calls | §4 | **`bc`** |

Three bands, and the boundaries between them are what an implementer needs:

1. **No branch — a branchless arithmetic select.** Reached only when the
   relation is `==`/`!=` (or reducible to it: `a==b` → `a−b==0`, `a==7` →
   `a−7==0`) **and** both arms are constants **and** the constant pair is cheap
   to build from a 0/1 or 0/−1 mask. `{1,2}` is one `addi`; `{5,9}` from a mask
   is one `rlwinm`+`addi`; `{70,0}` is one `and`. Ordered relations (`<`, `>`)
   never fold — their branchless bool spine is 4–6 instructions before the select
   (`docs/CODEGEN_W6_COMPARE.md` §4.4/§4.5), so the branch is cheaper.
2. **A conditional return (`bclr`), no label, no displacement.** Reached when one
   successor **is** the function's epilogue and the other is short enough to fall
   through. This is the majority band and it covers **both** real `cflow-if-1`
   functions in `Pool.cpp`.
3. **A real forward `bc` with a displacement.** Reached when neither arm can be
   the fall-through-plus-conditional-return: because an arm ends in a transfer
   that is not the epilogue (`MemFree`'s two tail calls, `f_eqcall`'s tail call),
   or because both arms have content that joins (`a_var`).

> **The decision between bands 1 and 2 is a c2 cost model and this lane declines
> it.** `?a_eq` (`a==b → 1:2`) folds; `?f_eq59` (`a==b → 5:9`) does not; `?f_eqzk`
> (`a==0 → 5:9`) does. Every fitted rule I could state is consistent with the
> eighteen rows above and none of them is *tested* by them, which per this
> project's own standard makes it a hypothesis, not a measurement. What the port
> needs is not the rule but the **boundary of an accepted class**, and §5.1
> chooses that boundary to sit entirely inside band 3.

**The consequence for the frontier, stated plainly.** Band 2 is why
`?Pool::Alloc` and `?Pool::Free` — two of the five `cflow-if-1` functions on the
entire frontier — need **no branch lowering at all**. An implementer who builds
band 3 and grades on `Pool.cpp` will see no movement and will not know whether
the branch code is right or the fold code is. Grade on `xboxmem.cpp` (§4).

### 3.6 What control flow does *not* change

Measured, and worth stating because each one is a place work could be wasted:

* **Labels mint no symbols.** `?d_early` and `?d_switch` are leaves with three
  branch targets each and carry **zero** storage-class-6 records. The only
  `LABEL` symbols in any obj here are the `$M<n>` pair a **framed** function
  gets, one at the body offset and one at the section end — exactly
  `docs/LABEL_COUNTER.md`'s model, with **no** dependence on block count:
  `?d_goto` (2 blocks) gets `$M2651`/`$M2652`, `?d_cold` (4 blocks) gets
  `$M2656`/`$M2657`. Registered rival **R-B4** (a `$L` family keyed on blocks) is
  refuted.
* **Branches do not force a frame.** `?d_nest`, `?d_early`, `?d_switch`,
  `?a_var`, `?MemFree`, `?Pool::Free` all branch and are all leaves with **no
  `.pdata`**. Frame class is decided by calls and saved registers, not by control
  flow.
* **No alignment padding is inserted at a branch target.** No loop head in any
  probe is aligned and no `nop` appears inside a function. Function *starts* are
  padded to 8 bytes as `docs/CODEGEN_W6_COMPARE.md` §3 records; under `/O1`'s
  per-function COMDATs the question is mostly moot.
* **The epilogue block is emitted even when unreachable** (§1 C6). `?b_if`,
  `?b_and` and `?b_or` each end with a `4e800020` that no path reaches, because
  every edge into it became a `bclr`. It is a real four bytes of `.text` and it
  is in the section size.

### 3.7 Loops

> ### 2026-08-05, lane `w-rotate` — **80 cells, five grids, and three of this section's statements move.** (`rungs/2026-08-05-w-rotate.md`, `work/w-rotate/rotgrid.py`, boards **#771**–**#779**.)
>
> * **(d) THE GUARD'S FORM IS A RULE, and it is decided by the block the loop falls out to** — **46 of 46** rotated cells, both poles present (14 `bc`, 32 `bclr`). The guard branches to the loop's fall-out block and folds to a **`bclr`** form **exactly** when that block is a bare `blr`. Hold the loop fixed and move only the exit: `while(*s){r=r+*s;s++;} return r` takes `bclr`, `... return r+1` takes `bc`. Board **#771**. This is the rule `codegen::ptr_walk_loop`'s hard-coded `bc` would need in order to widen.
> * **(e) THE ENTRY FORM IS NOT DECIDED BY `continue`, AND §3.7a's `?d_cont` READING DOES NOT REPRODUCE.** `for-call-cont` — a `continue` **and** a call, §3.7a's own combination — is a **guard**, and so is `while-call-cont`. **Not one JUMPIN cell in 80 has a `continue`.** What decides it is whether the entry's test block and the back edge's test block share a suffix: same register for the peel and the induction load, or an explicit compare feeding the back edge. **8 of 8 held out**, scoped to the sentinel walk. Boards **#773**, **#776**. **This dissolves §8.2's L4 rather than answering it** — see the banner there.
> * **(f) The two test sites are NOT copies.** **28 of 35** comparable cells share `BI` with the sense inverted; the 7 that do not are the counted family, where the guard reads **cr6** from an explicit compare and the back edge **cr0** from a record form — or where the guard is a *constant-folded specialization* of the loop test (`for-break` guards `cmpwi cr6,r3,0` against a back edge of `cmpw cr6,r11,r3`). Exact on the sentinel family, 23 of 23. Board **#779**.
> * **(a) gains one measured exception.** `for(;;){}` emits **one word**, `48000000` — `b .+0`, an **unconditional back edge to itself**. 78 of 79 loops in the grid are conditional; that one is not. Board **#777**.

Three findings, in decreasing order of how much they change a lowering.

**(a) The back edge is never an unconditional branch.** The IL's back edge is an
unconditional `3A TOP` (§2.3). In the obj it is always a **conditional** branch —
`bc` or `bdnz` — and the loop is bottom-tested:

| body | guard | back edge |
|---|---|---|
| `?c_callloop` | `cmpwi cr6,r3,0 ; beq cr6,+16` | `addic. r31,r31,-1 ; 4082fff8 bne cr0,-8` |
| `?d_break` | `cmpwi cr6,r3,0 ; beq cr6,+24` | `addic. r31,r31,-1 ; 4082fff0 bne cr0,-16` |
| `?c_forcall` | `cmpwi cr6,r3,0 ; ble cr6,+28` | `cmpw cr6,r31,r29 ; 4198ffec blt cr6,-20` |
| `?d_cont` | `48000014 b +20` — **an unconditional jump *into* the test** | `cmpwi cr6,r31,0 ; 409affec bne cr6,-20` |

`?d_cont` is why prediction C3 scores half: the loop *entry* is not always a
guard compare. Its source has a `continue`, which makes the test a real join
target, and c2 enters the loop by jumping to it.

**(b) `for` is straightened; the IL's rotation does not survive.** `?c_forcall`'s
IL is the §8.2 form — `init · 3a COND · 29 INCR · incr · 29 COND · cond ·
38 EXIT · body · 3a INCR · 29 EXIT`, i.e. **two** labels and **two** jumps with
the increment textually above the condition. The obj is a single back edge with
the increment below the body:

```text
0018  2f030000  cmpwi cr6,r3,0
001c  4099001c  ble   cr6,0x38        ; guard
0020  7fe3fb78  mr    r3,r31          ; <- loop top
0024  4bffffdd  bl    ?gi@@YAHH@Z
0028  3bff0001  addi  r31,r31,1       ; the INCREMENT, now below the body
002c  7fc3f214  add   r30,r3,r30
0030  7f1fe800  cmpw  cr6,r31,r29     ; the CONDITION, now at the bottom
0034  4198ffec  blt   cr6,0x20        ; back edge
0038  7fc3f378  mr    r3,r30
```

**A lowering that emits the IL's block order literally emits wrong bytes here.**
Prediction C4, registered with the rival that the rotation survives verbatim;
the rival is refuted.

**(c) Leaf counted loops become CTR loops — entirely unpredicted.** When the loop
body contains **no call**, c2 computes the trip count, loads it into the count
register, and uses `bdnz`:

```text
?c_while   (int n){ int s=0; while(n){ s=s+n; n=n-1; } return s; }
    0008  2f0b0000  cmpwi cr6,r11,0
    000c  4d9a0020  beqlr cr6
    0010  7d6903a6  mtctr r11
    0014  7c635a14  add   r3,r3,r11        <- loop top
    0018  396bffff  addi  r11,r11,-1
    001c  4200fff8  bdnz  -8               <- BO=16, BI=0
    0020  4e800020  blr
```

`?c_for` is the same shape, and `??0Pool@@QAA@HPAXH@Z` — a **real** frontier
function — is too (`mtctr r10 ; … ; 4200fff4 bdnz -12`). `?c_callloop` and
`?c_forcall`, whose bodies call, get the compare form instead: CTR is not usable
across a call.

`bdnz` is `0x42000000 | (BD & 0xFFFC)` — op 16 with `BO = 16` (decrement CTR,
branch if the result is non-zero) and `BI = 0`. `mtctr rS` is
`0x7C0903A6 | (rS<<21)`. **Neither instruction exists in the port today**, and
neither appears anywhere in `docs/`. Any loop rung must decide up front whether
its accepted class includes leaf counted loops — and if it does, it inherits
trip-count computation, which is not a CFG problem at all.

## 4. The minimal instance — `cflow-if-1`, in full

**Build this one first.** `?MemFree@NUISPEECH@@YAXPAX0K@Z`, from the frontier TU
`src/xdk/nuispeech/xboxmem.cpp`. It is chosen over every probe in this lane for
three reasons: it is a **real** workload function, it sits in fold band 3 so it
provably emits a branch (§3.5), and its TU is the **only** frontier TU that
`cflow-if-1` alone unblocks (§5.1).

### 4.1 Source, IL, obj — the whole function

```cpp
void NUISPEECH::MemFree(void *v1, void *v2, unsigned long ul) {
    if (v1 == nullptr) {
        XMemFree(v2, ul);
        return;
    }
    RtlFreeHeap(v1, 0, v2);
}
```

The `.ex` body, from the `4C 4F 11` marker, every byte accounted for:

```text
4c 4f 11                          body marker
53                                open body scope                     depth 3
4f 01 10                          line 16
53                                open the if scope                   depth 4
b9 6c 0f 86 43 83 08              LOAD  tok 0x0F6C (v1)  TYPE ptr
33 86 43 83 08 00                 LIT   TYPE ptr, 0
1f                                CMP EQ                              -> bool
38 71 0f                          brFALSE -> L_0F71    (the else entry)
53 53                             then-clause scopes                  depth 5,6
4f 01 11                          line 17
26 da 0e                          push callee XMemFree
bd 82 07 03 00 80 05 10 00 00     CALL header, void result
b9 6e 0f 86 42 22 55 86 42 22     arg: LOAD ul,  push
b9 6d 0f 86 43 83 08 55 86 43 83 08   arg: LOAD v2, push
4c 4b                             end args, end statement
4f 01 12
3a 70 0f                          JUMP -> L_0F70  (epilogue)  = `return;`
4f 01 13  54 05                   close                               depth 5
4f 01 14  54 04                   close                               depth 4
29 71 0f                          L_0F71:      the else entry
54 03                             close the if scope                  depth 3
26 ec 09 bd 86 42 75 00 80 07 10 00 00    CALL RtlFreeHeap
b9 6d 0f 86 43 83 08 55 86 43 83 08       arg: v2
33 86 42 22 00 55 86 42 22                arg: literal 0
b9 6c 0f 86 43 83 08 55 86 43 83 08       arg: v1
4c 4b
4f 01 15
3a 70 0f                          JUMP epilogue
54 02                             close the body                      depth 2
29 70 0f                          L_0F70:      the epilogue label
4f 12 47 54 01 54 00              function tail
```

The obj — one `.text` COMDAT, `SizeOfRawData = 0x24`, `nrel = 2`,
`Characteristics = 0x60401020`, **no `.pdata`**, symbol
`?MemFree@NUISPEECH@@YAXPAX0K@Z` EXTERNAL at `Value = 0`:

```text
        7c 8b 23 78   mr      r11,r4        ; v2 -> r11, live across BOTH arms
        2b 03 00 00   cmplwi  cr6,r3,0      ; v1 == 0, UNSIGNED (pointer operand)
        40 9a 00 10   bne     cr6,+16       ; -> 0x18, the else entry
        7c a4 2b 78   mr      r4,r5         ; then: arg2 = ul
        7d 63 5b 78   mr      r3,r11        ; then: arg1 = v2
        4b ff ff ec   b       XMemFree      ; REL24 @0x14   (`return;` folded to a tail call)
        7d 65 5b 78   mr      r5,r11        ; else: arg3 = v2
        38 80 00 00   li      r4,0          ; else: arg2 = 0
        4b ff ff e0   b       RtlFreeHeap   ; REL24 @0x20
```

as a byte string:

```text
7c 8b 23 78 2b 03 00 00 40 9a 00 10 7c a4 2b 78 7d 63 5b 78 4b ff ff ec
7d 65 5b 78 38 80 00 00 4b ff ff e0
```

Relocations, both `IMAGE_REL_PPC_REL24` (`0x0006`), no `PAIR`:

```text
14 00 00 00  <sym XMemFree>     06 00
20 00 00 00  <sym RtlFreeHeap>  06 00
```

### 4.2 Every decision the emitter makes here, enumerated

An implementer can check their lowering against this list item by item.

1. **The compare instruction and its width/signedness.** Operand TYPE is
   `86 43 83 08` (pointer) → **unsigned** → `cmplwi`, not `cmpwi`. Literal 0 fits
   the immediate field. Destination field **cr6**. → `2b 03 00 00`.
2. **The branch sense.** IL is `1f` (EQ) consumed by `38` (brFALSE), so the
   emitted condition is the **negation** of EQ → `bne`, `BO = 4`, `BI = 26`.
3. **The branch target.** The IL names label `0x0F71`; its `29` sits after the
   then-clause. The emitter must resolve token → offset, which it cannot do on
   first pass because `3A`/`38` carry no direction (§2.1) — so a **fixup list**
   is required even for this one branch.
4. **The displacement.** `BD = 0x18 − 0x08 = +16`, self-relative, low two bits
   zero, **no relocation**. → `40 9a 00 10`.
5. **Block order.** Then-block first (§3.4), else-block second, no join.
6. **No join branch.** Both arms end in a tail call, so nothing is emitted at the
   end of the then-block (§3.4, C2).
7. **The epilogue is never materialized.** Both `3A → L_0F70` become tail calls;
   the epilogue label has no block. Contrast §3.6, where an unreachable epilogue
   *is* emitted — the difference is that here no edge reaches it at all.
8. **A value is live across the branch.** `v2` arrives in r4, which both arms
   need to clobber, so it is copied to **r11** in the entry block and read in
   both successors. This is the item a shape-matcher cannot express (§6).
9. **The argument shuffles are *not* uniformly hoisted.** Only the shuffle both
   arms need (`mr r11,r4`) moves to the entry block; `mr r4,r5` stays in the
   then-block. Contrast the sibling `?MemAlloc` in the same TU, where `mr r4,r5`
   **is** hoisted, because there both arms consume r5's value. The discriminator
   is whether the value is needed on both paths — a liveness fact, not a syntax
   one.
10. **The tail calls encode section-start-relative and take a `REL24`** (§3.3),
    where the `bc` at step 4 encodes its true displacement and takes none. Two
    encodings, one pass.

### 4.3 The sibling cells in the same TU, for a 4/4 grade

`xboxmem.cpp` has four functions and converts only when **all four** do.

| fn | shape | obj |
|---|---|---|
| `?GetXAllocAttributes` | `cflow-straight`, blocked on `expr-cmp-ne` | 0x18 B, 0 reloc, no branch: `addic ; lis ; subfe ; rlwimi ; mr ; blr` |
| `?MemAlloc` | `cflow-if-1`, blocked on `expr-cmp-eq` | 0x24 B, 2 reloc, one `409a000c` |
| `?MemFree` | `cflow-if-1`, blocked on `expr-cmp-eq` | 0x24 B, 2 reloc, one `409a0010` |
| `?MemSize` | `cflow-if-1`, blocked on `expr-cmp-eq` | 0x24 B, 2 reloc, one `409a0010` |

`?MemSize` is byte-identical to `?MemFree` except for its two relocation targets
(`XMemSize`/`RtlSizeHeap`) — the two functions' `.text` payloads are the same 36
bytes. `?MemAlloc` differs in the entry block (two hoisted shuffles, §4.2 item 9)
and in the else arm (`rlwinm r4,r4,5,28,28` for `(attrs>>0x1b)&8`).

**`?GetXAllocAttributes` is not a CFG problem and must not be counted as one.**
It is `cflow-straight` and its blocker is the comparison *value* family of
`docs/CODEGEN_W6_COMPARE.md` — a `!=0` spine feeding a shift and an `or` with a
constant. It needs `expr-cmp-ne` plus `<<`, `|` and a `lis`-materialized
constant, none of which this document specifies. **The CFG step alone does not
convert `xboxmem.cpp`.** See §5.

### 4.4 What the *second* case adds

Stated as a decision this document makes, so the widening order is not
improvised:

* **`if-2` adds nothing.** Two independent `38` sites, each a repetition of the
  first (§2.3), two `bc`, two fixups, cr6 reused. No new production, no new
  rule. It should be admitted *in the same rung* as `if-1`, not after it.
* **`if-n` adds nothing either** — `?b_ifn` is `?b_if2` a third time. The census
  distinction between `if-1`, `if-2` and `if-n` is a **count**, and it does not
  correspond to a cost step in the emitter. This is the single most useful
  scheduling fact in the document (§5.2).
* **Nesting adds nothing.** `?d_nest` (`if(a){ if(b) return 1; return 2; } return
  3;`) emits one `bc` and one fold, at 0x20 B. Nested `if` is `if-n` with
  different label positions.
* **`&&`/`||` add nothing** — already two branches in the IL, no `crand`/`cror`
  in the obj (§3.4, C5).
* **A loop adds three things at once**, and this is where the cost step really
  is: a **back edge** (so values must be allocated across it), a **bottom-tested
  rotation** the IL does not have (§3.7b), and — for leaf counted loops — an
  entirely new instruction family (§3.7c). A loop rung is not "`if-n` plus a
  negative displacement".
* **`switch` adds a fourth thing and should be last.** `?d_switch` (cases 1, 2, 7
  + default) emits **no jump table at all** — a compare chain plus a fold — and
  its blocks land in **reverse** source order (default's tail at 0x10, case 2 at
  0x28, case 1 at 0x30). A wider switch presumably does emit a table
  (`IL_STMT_GRAMMAR.md` §11 has the IL for one), and this lane measured only the
  narrow case. Both the table threshold and the block-order rule are **unmeasured**.

## 5. The widening order, ranked by TUs

### 5.0 Two traps, before any number

**Trap 1 — `complete-none` means the parse stopped, not that one construct is
missing.** **32 of the 35** blocked frontier functions read `complete-none`
(`docs/rungs/2026-08-04-w-front.md` §3.1). A blocker name is the **first** thing
that stopped the decode and is never a promise that granting it converts the
function. Only three rows carry any completeness signal at all. **Every count in
this section is therefore an upper bound on TUs the CFG step *unblocks*, not a
count of TUs it *converts*.** Anyone converting a row should expect a second
blocker and budget for it.

**Trap 2 — rank by TUs converted, never by function count.** `expr-cmp-eq` is
the largest blocker bucket on the frontier — **14 functions across 7 TUs** — and
granting it alone converts **zero** TUs: 13 of the 14 sit behind
`cflow-if-*`/`cflow-loop`/`cf-expr-0x05`, and the fourteenth shares its TU with a
function blocked on something else. `docs/BOARD.md` #150 already recorded the
same trap for `expr-op-0x27` (22,759 emitted bodies, **6** converted functions).
The tables below are keyed on **TUs**, and the function counts are shown only so
the divergence stays visible.

### 5.1 The payoff case

w-front measured the 17 frontier TUs — the TUs whose *only* remaining blocker is
codegen breadth. They carry **35 blocked functions**: 8 straight-line, 5
`cflow-if-1`, 1 `cflow-if-2`, 10 `cflow-if-n`, 8 `cflow-loop`, and 3 that did not
decode end to end. A TU converts only when **every** blocked function in it
converts, so:

> **Exactly 2 of the 17 are straight-line-only. The other 15 sit behind the step
> this document specifies.**

That is the payoff case, and it is why `docs/ROADMAP.md` §G4's "restructure into
a real IL→lower→emit pipeline when the CFG step forces a block/instruction IR"
is now due rather than deferred.

### 5.2 The ladder

Each step's yield is the number of frontier TUs whose blocked functions all fall
inside the shapes admitted **so far** (plus `cflow-straight`, which other lanes
own).

| step | admits | frontier TUs newly unblocked | running total |
|---|---|---:|---:|
| **1** | `cflow-if-1` **and `if-2` and `if-n` together** (§4.4: they are one rung) | **8** | 8 |
| **2** | `cflow-loop` | **4** | 12 |
| **3** | `switch` | **0** | 12 |

Split finer, to show why step 1 is one rung and not three:

| sub-step | TUs unblocked |
|---|---:|
| `if-1` alone | **1** — `src/xdk/nuispeech/xboxmem.cpp` only |
| `+ if-2` | **+0** — the only `if-2` function is in `mmio.cpp`, which also has two `if-n` functions |
| `+ if-n` | **+7** — `osfinfo.cpp`, `undname.cpp`, `vswprnc.cpp`, `xlrcimpl.cpp`, `negate_test.cpp`, `vsnprnc.cpp`, `mmio.cpp` |

**Shipping `if-1` and stopping is worth 1 TU. Shipping `if-1`+`if-2`+`if-n` is
worth 8.** Since §4.4 establishes that `if-2` and `if-n` require *no production
`if-1` does not already need*, splitting them across rungs buys nothing and costs
seven TUs of visible progress.

Step 2's four TUs: `Sort.cpp`, `jsonwriter.cpp`, `IPP_basicmath_xbox.cpp` (4
functions at once, all `cflow-loop`, no EH, no data sections — the purest loop
specimen on the frontier), `EncryptXTEA.cpp`.

Step 3 is zero because no frontier TU contains a `switch`-classed function at
all. `switch` should be *recognized and refused precisely* (its IL is in
`IL_STMT_GRAMMAR.md` §11), not built.

### 5.3 The five TUs the CFG step cannot reach

Of the 17: 12 are in the table above, and the remaining 5 are out of scope for
this document.

| TU | why |
|---|---|
| `src/xdk/nuispeech/xboxheap.cpp` | `cflow-straight`, blocked on `expr-op-0x27`. Not this step — and `docs/BOARD.md` #150 puts its whole-corpus yield at 6 emitted functions, so it is a one-TU win and should be scoped as one |
| `src/Main.cpp` | `cflow-straight`, but `eh-state1` — the whole EH record set must be modeled too. Not one widening away |
| `src/system/synth_xbox/Biquad.cpp` | one function is `cf-expr-0x05` (a DIV width refusal, §2.2) — its control flow was never read |
| `src/system/rndobj/wordwrap.cpp` | same, plus an `if-n` and a straight-line function |
| `src/system/utl/Pool.cpp` | same. Its two `cflow-if-1` functions are already fully emitted by c2 as `bclr` folds (§3.5 band 2) and need no branch, but its constructor is `cf-expr-0x05` |

`Pool.cpp` deserves the emphasis: it is named in the brief as a `cflow-if-1` site
and it is one, but **it is not a branch site and it is not convertible by this
step**. It is the wrong TU to grade on, twice over.

### 5.4 What a step-1 rung must also carry that is not CFG

Stated because §5.2's "8 TUs" will otherwise read as a work estimate, which
Trap 1 forbids. Every one of the 8 also carries at least one **expression**
blocker, and the CFG step does not touch any of them:

| TU | expression blocker(s) named as first-refused |
|---|---|
| `xboxmem.cpp` | `expr-cmp-eq` ×3, `expr-cmp-ne` ×1 |
| `osfinfo.cpp` | `expr-cmp-ge` |
| `undname.cpp` | `expr-cmp-ne` |
| `vswprnc.cpp` | `expr-cmp-eq` |
| `xlrcimpl.cpp` | `assign-rhs-call-0x26` |
| `negate_test.cpp` | `assign-store-type-0x86` ×2 |
| `vsnprnc.cpp` | `expr-cmp-eq`, `call-arg-lit-permuted:mid` |
| `mmio.cpp` | `expr-cmp-eq` ×3 |

And per Trap 1, 32 of these rows are `complete-none`, so granting **both** the
CFG shape and the named expression blocker still does not promise a conversion.
The honest framing for a rung brief is: *the CFG step removes the control-flow
blocker from 12 frontier TUs; how many of those then convert is a second
question that only the byte compare answers.*

## 6. What the block/instruction IR must carry

`docs/ROADMAP.md` §G4 defers the restructure "until the CFG step (W8) and frames
(W10) force a block/instruction IR". This section says what that IR must support,
derived from §3–§4 and from what the current representation demonstrably cannot
express.

### 6.1 What the current representation is, and why it cannot be extended

`c2_il::IlFunction` is a struct of roughly a dozen **mutually-exclusive
`Option` fields**, one per whole-body shape: `ops` (a flat `Vec<IlOp>`),
`tail_call`, `framed_call`, `call_seq`, `compare`, `float_leaf`, `fp_tail`,
`fp_arg_sources`, `arg_sources`, … . `c2_core::codegen::select_function` is an
ordered `match` over them whose "dispatch order is load-bearing and documented
per adjacency".

That design is exactly right for what it does and it has three properties that
make it unable to carry a CFG:

1. **The unit of recognition is the whole function.** `BodyShape::StraightLine`,
   `IntTailCall`, `FramedCall`, `CallSeq` each describe an entire body. There is
   no sub-function unit at all, so there is nothing for a second basic block to
   *be*.
2. **`ops: Vec<IlOp>` is a single flat postfix stream with no terminator
   concept.** `IlOp` has `Load`, `LoadInd`, `AddrOf`, arithmetic — and no
   variant that transfers control. A branch cannot be added to it without the
   stream ceasing to mean "evaluate these in order", which is the one assumption
   every existing lowering rests on.
3. **Emission is position-free.** `select_text` returns a `Vec<u8>` and the
   callers that need an offset (`Selected::Tail`, `Framed`, `Seq`) get it by the
   caller knowing where the function lands. Nothing in the pipeline can express
   "this word's value depends on the address of a thing not yet emitted", which
   is precisely what a branch displacement is.

Adding an `IlOp::Branch` would break (2) and would still leave (1) and (3). The
restructure is not avoidable by widening the enum.

### 6.2 The minimum the new IR must carry

Derived item by item from the measurements, with the section that forces each.

**A. Basic blocks with an explicit emission order, and terminators.**
A block is a straight-line instruction run ending in exactly one terminator:
fall-through, `bc(cond, target)`, `b(target)`, `bclr(cond)`, `blr`, or a tail
`b(symbol)`. The *order of blocks in the output* must be an explicit property,
not implied by traversal, because §3.4 shows it is the IL's statement order and
§3.4.1 shows one measured case where it is not — a lowering must be able to state
the order it chose, so the two can be told apart.

**B. Labels as first-class, resolved by a fixup pass.**
`3A`/`38`/`39` carry no direction (§2.1), so the target's offset is unknown when
the branch is emitted. The IR needs a label identity (the IL token will do), a
map from label to block, and a **fixup list** of (word offset, label, form).
Even §4's single-branch minimal instance needs this — it is not an optimization
for the many-block case.

**C. Two encodings for a branch, chosen by target kind.**
`bc`/intra-section `b` patch a true self-relative displacement and emit **no**
relocation; external `b`/`bl` patch a section-start-relative placeholder and emit
a `REL24` (§3.3). The fixup entry must record which, and a single "patch the
branch" path that assumes one of them corrupts the other.

**D. A displacement range check with a defined expansion.**
`bc` reaches ±32764. Past that, invert the condition and branch over an
unconditional `b` (§3.3.1). This must be in the fixup pass, because the overflow
is only visible after layout.

**E. A condition-code model with two producers.**
The IR must distinguish *"compare X against Y into cr6"* from *"this arithmetic
instruction's record form sets cr0"*, because §3.2 shows c2 branches on both and
the `BI` field differs (26 vs 2). A model with a single implicit condition
register emits wrong bytes for every decrement-and-test loop.

**F. Values live across block boundaries — the real cost.**
§4.2 item 8: `MemFree` copies `v2` from r4 to **r11** in the entry block because
both successors need it after clobbering r4. §3.4.1's `d_join` holds `b` in
**r31** across a call. `?b_if2` and `?b_ifn` hold their formals in r31/r30 across
calls and are **framed** for that reason alone.

This is the item that makes the restructure a restructure. Today's register
model is positional and local: formals occupy `ARG_REGS` by declaration order,
the result is r3, temps descend from r11 in emission order
(`docs/CODEGEN_W6_COMPARE.md` §6). That model has no notion of a value being
live at a program point, and every one of the cells above needs one. Note also
that `docs/CODEGEN_W6_COMPARE.md` §6 already records the allocator as
"demonstrably richer than a descending counter and **not** characterized" — so
this item depends on work nobody has done, not merely on work nobody has
scheduled.

**G. A place to record the folds, per accepted shape.**
§3.5's three bands are not passes to reproduce; they are the reason the accepted
class must be *stated as a shape*, and the shape must be checkable before
emission. Concretely: the port must be able to say "this `cflow-if-1` is band 3"
and refuse otherwise, rather than emitting a branch and being wrong on 6 of 7
leaf bodies.

### 6.3 What it must NOT carry

The port is **I/O-behavioral** and does not reimplement c2's 35 passes. This
document specifies no optimizer and the IR should not host one:

* **No code motion.** §3.4.1's hoist and tail-merge are recorded as a *limit on
  the accepted class* (a body whose arms end in the same call is out of class),
  not as a pass to build.
* **No cost model.** §3.5's band 1/band 2 boundary is declined. The class is
  drawn to sit inside band 3.
* **No loop rotation as a general transform.** §3.7's rotation is an *emission
  shape* for the accepted loop class, established per shape from bytes — not a
  transform applied to arbitrary IL.
* **No CTR-loop discovery.** §3.7c's trip-count computation is a separate
  problem; a loop rung should either admit the shape from measurement or refuse
  bodies that would need it.
* **No neutrality or behavior-preservation classifier.** Per `CLAUDE.md`: the
  compiler plus the byte compare is the sole judge.

The discipline that keeps this honest is the existing one: **decoding a
production is not licence to emit it** (`IL_STMT_GRAMMAR.md` §14.2). The
decode-only scanner in `control_flow.rs` already reads every shape in this
document; the emission gate stays a whitelist of *shapes* and never becomes "the
branches decoded".

## 7. The `/FAsc` listing as a decode aid

Recorded because `docs/OBJ_DYNINIT_SHAPE.md` §6 measured the listing disagreeing
with the obj, and because for control flow the answer comes out the **other
way** — which is worth knowing precisely, not vaguely.

**The listing is byte-faithful for intra-function branches and not for external
ones.** `?MemFree`'s listing beside its obj:

| site | listing prints | obj carries | agree? |
|---|---|---|---|
| `bc` @0x08 | `409a0010  bne cr6,$LN1@MemFree` | `40 9a 00 10` | **yes, exactly** |
| `b` @0x14 | `48000000  b XMemFree` | `4b ff ff ec` | **no** — canonical vs section-relative |
| `b` @0x20 | `48000000  b RtlFreeHeap` | `4b ff ff e0` | **no** |

The reason is §3.3: the intra-function branch has no relocation, so the listing's
word *is* the final word; the external branch's word is a placeholder the
relocation completes, and the listing prints the canonical form instead.

**Block order in the listing equals block order in the obj.** The listing places
`$LN1@MemFree` between the two tail calls exactly where offset 0x18 falls. This
holds on every probe: `$LN1@b_if` at 0x0c, `$LN1@b_and` at 0x14, `$LN1@b_or` at
0x10 and `$LN2@b_or` at 0x14, `$LN2@b_if2` at 0x20 then `$LN1@b_if2` at 0x2c.
Contrast *section* order, which `OBJ_DYNINIT_SHAPE.md` §6 measured as disagreeing
— **the listing is reliable for intra-function layout and unreliable for
inter-section layout**, and those are different questions.

Two further uses, and one non-use:

* The listing **names the labels**, which makes an IL-token → block mapping
  readable at a glance while developing a fixup pass.
* It **shows the dead epilogue block** explicitly (`$LN1@b_if:` immediately
  before the unreachable `blr`), which is how §3.6's "emitted even when
  unreachable" was noticed rather than mistaken for trailing padding.
* `$LN<k>` numbering is **not** emission order (`$LN2@b_if2` precedes
  `$LN1@b_if2` in the bytes) and mints **no symbol records** (§3.6). It is
  listing-local naming and nothing should be derived from the numbers.

**It is a decode aid and never a gate.** The obj byte-compare is the sole judge
(`CLAUDE.md`). Every claim in §3 and §4 is transcribed from the obj; the listing
is quoted here only where the two were compared on purpose.

## 8. What an implementer still cannot build from this document

The most important section here. Understating it costs more than any missing
spec, so it is deliberately long and each item says **what is missing**, **what
would close it**, and **whether it blocks the step-1 rung** of §5.2.

### 8.1 Blocking for step 1 (the `if-1`/`if-2`/`if-n` rung)

| # | what is missing | what would close it |
|---|---|---|
| **B1** | **The band-1 ↔ band-2 fold decision** (§3.5). I can tell you which of eighteen measured bodies folds; I cannot tell you which of an unmeasured one will. Every rule consistent with the rows is untested by them. | A probe family that varies **only** the constant pair over a fixed relation (`{0,1} {1,2} {2,4} {5,9} {70,0} {1,100} …`) and only the relation over a fixed pair, then reads the fold/no-fold boundary directly. ~30 cells, one TU, cheap. **This is the single highest-value follow-up in the document.** |
| **B2** | **The expression layer for every function in the 8 TUs** (§5.4). Nothing here specifies `expr-cmp-eq`, `expr-cmp-ge`, `assign-store-type-0x86`, `assign-rhs-call-0x26` or `call-arg-lit-permuted`. The CFG step is necessary and nowhere near sufficient. | Not this seam. `docs/CODEGEN_W6_COMPARE.md` §7 specifies the comparison *value* family; the comparison-feeding-a-branch family is §3.2 here; the rest are open. |
| **B3** | **Where the entry block's shuffles go.** §4.2 item 9: `MemFree` hoists one `mr` and not the other; `MemAlloc` hoists both. I state the discriminator as "needed on both paths", which **fits** both cells and is **tested by** neither. | Probes with 2–4 arguments where the set of values needed on both paths is varied independently of arity. |
| **B4** | **`cmplw` (register-vs-register unsigned)** is **unvaried** — no cell in this grid produced one. Marked unvaried, not constant. | One probe: `if(pa < pb)` on two pointers. |
| **B5** | **Whether the `bc` is ever emitted with a hint bit set.** Every `BO` observed is 4, 12, 16 or 20 — no `BO` with the `y` prediction bit. That is an observation over ~30 branches, not a rule, and the frontier is not fully swept. | A sweep of `BO` values over the whole 878-TU workload's objs. |

### 8.2 Blocking for step 2 (the loop rung)

> **PARTIAL CLOSURE 2026-08-05, lane `w-loop`** (`docs/rungs/2026-08-05-w-loop.md`,
> `work/w-loop/loopcost.py`, boards #741/#744/#745). The five rows below were
> written from a grid whose loop cells are framed or call-bearing on 4 of 7 rows,
> and **every loop function on the codegen frontier is a leaf**, so a 17-cell
> leaf grid was run. What it moved:
>
> * **L2 — answered in scope.** CTR (`mtctr`/`bdnz`) when the trip count is
>   computable at entry and the body has no call: `while`(decrement), counted
>   `for`, **stride 3**, counting down, `+continue`, indexed load — **7 cells**.
>   Compare-form otherwise: `do/while`, `for(;;)`+`break`, backward `goto`,
>   `+break`, pointer-walk-to-sentinel — **5 cells**. **A `break` does kill CTR**
>   (`?d_break` generalizes). The boundary is *"is there an entry test whose trip
>   count c2 can compute"*, not *"is it a `for`"*.
> * **L3 — partly.** Stride 3 gets CTR, so c2 **does** compute
>   `(hi-lo+k-1)/k`. The computation itself is still unmodelled and a non-zero
>   start is still unvaried.
> * **L4 — one cell moved.** `+continue` keeps CTR here (§3.7a's `?d_cont` is
>   call-bearing and got the compare form), so the entry-form rule is **not**
>   "a `continue` makes the test a join target". Still unstated.
> * **L5 — the rotation held on 28 more back edges.** §3.7a extended: **0 of 28**
>   backward intra-section branches across the leaf grid is unconditional. The
>   registered risk cell was `for(;;)` with its only exit inside the body, and it
>   emits `bne cr0` on an `addic.`.
> * **L1 is untouched and is still the largest unknown.**
>
> **And a sixth item L1–L5 do not name, which is the one that blocks first:**
> a leaf loop charges the **compiler-label counter** +1..+4 while
> `coff::plan_labels` charges 0 (`LABEL_COUNTER.md` §4.2.1, 17 cells). That
> charge is *unobservable* in a TU with no framed function (§4.2.3, 34 of 34,
> control 17 of 17) — which is why 3 of the 6 `cflow-loop` frontier TUs read
> `label-free` in the scan — but it is a hard refusal everywhere else, and it is
> **not derivable from the emitted bytes**: `do/while`, `for(;;)`+`break` and a
> backward `goto` emit the *same 24 bytes* and charge +1, +3, +1.


| # | what is missing | what would close it |
|---|---|---|
| **L1** | **Register allocation across a back edge.** §6.2 item F. `docs/CODEGEN_W6_COMPARE.md` §6 already records the allocator as richer than a descending counter and **uncharacterized**; a loop needs it *and* needs it to agree across the edge. This is the largest single unknown in the document. | A characterization lane on the allocator itself. It is a prerequisite, not a sub-task of the loop rung. |
| **L2** | **When c2 chooses a CTR loop.** §3.7c measures *that* leaf counted loops become `mtctr`/`bdnz` and that loops containing calls do not. The boundary — variable trip counts, multiple exits, `break`, nested loops, loops whose counter is used after the loop — is **unmeasured**. `?d_break` has a `break` and got the compare form; that is one cell. | A loop probe grid crossing (trip count known/unknown) × (body has call / not) × (has break / not) × (counter live after / not). |
| **L3** | **Trip-count computation.** `?c_for` emits `mtctr r11` where r11 is the loop bound; a general `for(i=lo;i<hi;i+=k)` needs `(hi-lo+k-1)/k`, and no cell here has a stride other than 1 or a non-zero start. | Same grid as L2 with strides and starts varied. |
| **L4** ⚠ **DISSOLVED, not answered — see the banner below the table** | **Loop entry form.** §3.7a: `?c_callloop` and `?d_break` guard with a compare; `?d_cont` jumps into the test. I can say *that* both occur and that `?d_cont` differs by having a `continue`; I cannot state the rule. | ~~Probes varying whether the test is a join target, independently of `continue`.~~ **That grid was run — 80 cells — and it cannot close this row, because the discriminator is not in the loop's shape.** |
| **L5** | **Whether the rotation is always safe to assume.** Every loop measured is bottom-tested in the obj. Zero-trip loops are handled by the guard, but a `do`/`while` (`?c_do`) has no guard at all and I did not test a `while` whose condition has side effects. | Probes with a side-effecting condition. |

> ### 2026-08-05, lane `w-rotate` — **L4 IS NOT AN INDEPENDENT ROW AND NO GRID OVER LOOP SHAPES CAN CLOSE IT.** It was filed as a peer of L1 and it is a **consequence** of L1.
>
> 80 cells across five grids establish the chain:
>
> ```
>   SCHEDULE       where `lbzu` lands relative to the body's last use of the
>    (L1', new)    loop-carried value
>       |
>       v
>   REGISTER NEED  one register if that value is dead before the induction load,
>    (L1)          two if it is live across it
>       |
>       v
>   ENTRY FORM     identical test blocks -> emit once and jump in;
>    (L4)          different -> duplicate, i.e. ROTATE
> ```
>
> The rule is stated and graded — **H-SUF, 8 of 8 held out**: c2 shares the
> maximal common **suffix** of the two test blocks and duplicates the differing
> prefix. **It is still not evaluable by the port**, because both its inputs (which
> register the induction load writes, which producer feeds the CR bit) exist only
> *after* scheduling and allocation. Board **#773**.
>
> **And L1 itself is narrower than this document says.** *"The largest single
> unknown"* is a **constant** when only the loop **body** moves: over ten
> accumulate bodies with the signature held fixed, there is **1** distinct
> `(entry, tail)` plan and the body length moves 10 → 12 words. What moves
> instead is a **fourth mechanism this table does not have a row for** — the
> **interleave**, the placement of the loop-carried `lbzu` and the record-form
> test *into* the accumulate chain, at positions that change with the chain's
> length. Boards **#774**, **#775**.
>
> | | |
> |---|---|
> | **L1'** | **THE INTERLEAVE.** `lbzu` at slot 0 for a 1- and 2-op body, slot **1** for a 3-op body; the record form always exactly **two slots after it**. Measured on three lengths and **deliberately not fitted to three cells**. |
> | *what would close it* | A schedule rung: at least **6 chain lengths × 3 operator families**, position predicted per cell and graded `n of m` against real `c2`. **Everything else the sentinel-walk lowering needs is now measured** — the guard form (#771), the entry form given the allocation (#773), and the allocation itself (#775). |

### 8.3 Blocking for anything beyond

| # | what is missing |
|---|---|
| **S1** | **`switch` almost entirely.** §4.4 measured one narrow case (3 cases + default → compare chain, **no** jump table, blocks in **reverse** source order). The table threshold, the table's section (`.rdata` or `.text`), its relocation shape, the block-order rule for a wide switch, and dense-vs-sparse handling are all unmeasured. Zero frontier TUs need it. |
| **S2** | **Control flow interacting with EH.** Every cell in this lane is `eh-none`. `Main.cpp` is `eh-state1` and one frontier TU. How a branch interacts with EH state transitions is untouched here; `docs/EH_RECORDS.md` is the seam. |
| **S3** | **Control flow interacting with frames.** §3.6 establishes branches do not *force* a frame, but `?b_if2`/`?b_ifn`/`?d_join` are framed **because** values live across calls in different blocks — so the frame class becomes a function of the CFG. `docs/CODEGEN_FRAMED_CALLS.md` and W10 own this and it is not specified here. |
| **S4** | **Floating-point conditions.** No cell has one. `docs/CODEGEN_W6_COMPARE.md` §7 records that an FP compare is "a different family"; whether it uses `fcmpu` into cr6 and the same `BO`/`BI` discipline is **unmeasured**. |
| **S5** | **64-bit conditions.** `cmpdi`/`cmpd` appear in the disassembler's table and in **no** measured cell. `long long` comparisons are noted in `CODEGEN_W6_COMPARE.md` §7 as a different family. |
| **S6** | **The three `cf-expr-0x05` functions.** Not control flow (§2.2) and not addressable by this step at all. |

### 8.4 Stated limitations of the instrument

* **Flag exposure, bounded by measurement rather than by the fix.** At the time
  this lane's captures were taken, `c2rs capture` hardcoded `/Ox /GS- /c` and
  scanned its arguments with a `position(|a| a == "--keep-il")` that ignored
  every other one by construction — so `--flags-file` was accepted and silently
  dropped. **This lane ran against that binary.** Every `.ex` quoted here was
  therefore captured through `c2rs census --flags-file`, which did honour flags,
  and §10.1's control confirms it: the on-disk bundle reproduces byte-for-byte
  from the `census` path while the `capture` bundle differs.

  The instrument has since been repaired on `master` (`6a33b4d`, *"c2rs capture:
  honour `--flags-file`, and pin the dropped-flag failure mode"* — `capture` now
  takes `--flags-file`/`--cwd`, refuses unknown options instead of scanning past
  them, prints the profile it used, and the hardcoded list is the published
  `CAPTURE_IL_DEFAULT_FLAGS`). That commit landed **after** these measurements,
  so it neither validates nor invalidates them; the control does, and the control
  is why this document did not need to wait for the fix.

  **The control bounds this lane's measurements only. It clears no other lane's
  captures.** Any earlier document that captured IL via `c2rs capture` and read
  it against `/O1` objs is cross-flag, and — per that commit's own finding, that
  `/Ox` does not imply `/GF` — should be re-checked by whoever owns it.
* **All objs were built at the probe flags, not the workload flags, for the
  probes.** The probe flags (§10.1) are the workload's minus its `/I` include
  paths, which no probe needs. The two frontier TUs *were* built at the full
  workload flags with `--cwd`. Mixing the two in one table is avoided
  throughout; where a claim rests on probes only, it says so.
* **~30 branches, 6 probe TUs, 2 frontier TUs, 1 held-out long-branch sweep.**
  That is a small corpus. `docs/GAPS.md` §6's unstable-attribution rule applies:
  a shape seen in one cell is *unvaried*, not constant, and this document uses
  that word where it means it (§8.1 B4, §3.2's `cmplw` row).
* **No code was run against the port.** This lane wrote nothing under
  `crates/`, so nothing here is checked by a byte compare of *port* output —
  only by reading c2's. The first rung that implements §4 is what tests this
  document, and it should expect to find at least one thing wrong.

## 9. Proposed board rows

Not written to `docs/BOARD.md` — this lane does not own it. w-front proposed
183–185, so the next free number is **186**.

| # | title | state | note |
|---|---|---|---|
| 186 | A `cflow-if-1` body often emits **no branch at all** | **OPEN** | Three fold bands (§3.5). 6 of 7 `pa.cpp` leaves and **both** real `if-1` functions in `Pool.cpp` fold to a branchless select or a `bclr` conditional return. Grade a branch lowering on `xboxmem.cpp`, never on `Pool.cpp`. |
| 187 | The band-1 ↔ band-2 fold decision is a c2 cost model | **OPEN, DECLINED here** | §3.5. Eighteen rows fit; none tests. A ~30-cell probe varying only the constant pair over a fixed relation would settle it — §8.1 B1, the highest-value follow-up in `docs/CFG_SHAPE.md`. |
| 188 | Condition registers are two-valued: cr6 for an explicit compare, **cr0** for a record-form | **OPEN** | §3.2. Hard-coding `BI = 4*6+bit` emits `409a…` where the obj has `4082…` on every decrement-and-test loop — a plausible-looking wrong branch. CR fields are **reused**, never allocated (three live compares in one body all write cr6). |
| 189 | `if-2` and `if-n` add **no production** over `if-1` | **OPEN** | §4.4. `if-1` alone unblocks **1** frontier TU; `if-1`+`if-2`+`if-n` unblocks **8**. Splitting them across rungs costs seven TUs and buys nothing. Admit all three in one rung. |
| 190 | Leaf counted loops become **CTR loops** (`mtctr`/`bdnz`) | **OPEN** | §3.7c. An instruction family absent from the port and from `docs/` entirely, and it drags in trip-count computation, which is not a CFG problem. A loop rung must decide up front whether its class includes them. |
| 191 | Intra-section and external branches use **the same opcode with different encodings** | **OPEN** | §3.3. `48000008` (true displacement, no relocation) vs `4bffffec` (section-start placeholder + `REL24`). A fixup pass that treats every `b` alike corrupts one of the two. |
| 192 | The epilogue block is emitted **even when unreachable** | **OPEN** | §3.6. `b_if`, `b_and`, `b_or` each end in a dead `4e800020` because every edge into it folded to a `bclr`. It is four real bytes and it is in the section size. |
| 193 | Block order is IL statement order — with one measured refutation | **OPEN** | §3.4 / §3.4.1. Holds in 10 of 11 cells; `d_join` tail-merges two identical `bl` sites, empties the then-block and inverts the layout. Block order is downstream of code motion, so **a body whose arms end in the same call is out of any class specified here**. |
| 194 | `c2rs capture` silently dropped `--flags-file` and captured at `/Ox` | **FIXED** on `master` at `6a33b4d`; the **audit is OPEN** | The instrument is repaired (`capture` takes `--flags-file`/`--cwd` and refuses unknown options). What is *not* done is the back-audit: the `.ex` differs between `/Ox` and `/O1` in exactly the per-function opt words (§2.4) and `.gl`/`.sy` not at all, so a cross-flag capture **looks correct on the IL side and is wrong only on the obj side**. Every document that captured IL via `c2rs capture` before `6a33b4d` and read it against `/O1` objs needs re-checking by its owner. This lane is clear by the §10.1 control, which was run against the *pre-fix* binary. |

## 10. Reproducing this

### 10.1 Flags, and the provenance control

Two flag sets, used deliberately and never mixed in one table.

```sh
# probes — the workload's flags minus its /I paths, which no probe needs
printf '/nologo /wd4355 /wd4164 /c /GR /O1 /Oi /EHsc\n' > work/w-cfg/flags.txt

# frontier TUs — the workload's own flags, with the workload as cwd
#   work/dc3-workload/flags.txt   --cwd <dc3-decomp>
```

`/O1` implies `/Gy`, hence one `.text` COMDAT per function; `/GR` is in the
workload set and **not** in `c2rs`'s CLI defaults.

**The control that closes the `c2rs capture` exposure.** `capture` hardcodes
`/Ox /GS- /c`; `census --flags-file` does not. Run both against the same source
and compare:

```sh
c2rs census work/w-cfg/p/pa.cpp --flags-file work/w-cfg/flags.txt --keep-il work/w-cfg/ctl-o1
c2rs capture work/w-cfg/p/pa.cpp                                  --keep-il work/w-cfg/ctl-ox
cmp <ctl-o1>/*.ex <il-pa>/*.ex     # IDENTICAL — the on-disk bundle is the /O1 one
cmp <ctl-ox>/*.ex <il-pa>/*.ex     # DIFFERS
```

Measured result: `ctl-o1` reproduces the on-disk bundle **byte-for-byte in all
five files**; `ctl-ox`'s `.ex` differs in **exactly 7 bytes**, one per function,
at the per-function optimization word `4F 1F 80 05 00 <b> 00` —
`b = a0` (`OPT_WORD_OX = 0x00a00005`) against `b = 20`
(`OPT_WORD_O1 = 0x00200005`). `.gl` and `.sy` are identical either way, which is
why the failure is invisible without this comparison.

**Scope: this bounds this lane's measurements only.** It does not clear any other
lane's captures.

### 10.2 Commands

```sh
cargo build --release -p c2-harness

# probe TUs: obj at the probe flags, IL at the SAME flags (census, not capture)
for f in pa pb pc pd pe pf; do
  c2rs compile work/w-cfg/p/$f.cpp --flags-file work/w-cfg/flags.txt \
       --keep-obj work/w-cfg/$f.obj
  c2rs census  work/w-cfg/p/$f.cpp --flags-file work/w-cfg/flags.txt \
       --keep-il  work/w-cfg/il-$f
done

# the two frontier TUs, at the workload's own flags
D=<dc3-decomp>
for t in src/xdk/nuispeech/xboxmem.cpp src/system/utl/Pool.cpp; do
  c2rs compile "$t" --cwd $D --flags-file work/dc3-workload/flags.txt \
       --keep-obj work/w-cfg/$(basename $t .cpp).obj
  c2rs census  "$t" --cwd $D --flags-file work/dc3-workload/flags.txt \
       --keep-il  work/w-cfg/il-$(basename $t .cpp)
done

# read the objs
python3 work/w-cfg/objdis.py work/w-cfg/xboxmem.obj

# c2's own narration (a decode aid; see §7 before trusting it).
# `listing` takes neither --cwd nor --flags-file, so includes go in as --flag.
c2rs listing work/w-cfg/p/pb.cpp --out work/w-cfg/pb.cod \
     --flag /nologo --flag /c --flag /GR --flag /O1 --flag /Oi --flag /EHsc
```

### 10.3 The probe grid

`work/w-cfg/p/` — gitignored scratch, per the project rule; every byte quoted
above is transcribed here so the document stands without it.

| TU | functions | what it separates |
|---|---|---|
| `pa.cpp` | `a_eq a_ne a_lt a_eqk a_else a_var a_store` | whether a `cflow-if-1` leaf folds — 7 × `cflow-if-1` |
| `pb.cpp` | `b_if b_ifelse b_ifval b_and b_or b_if2 b_ifn` | control flow **around calls**, which cannot fold — 3/3/1 `if-1`/`if-2`/`if-n` |
| `pc.cpp` | `c_while c_for c_do c_callloop c_forcall` | loop rotation, back-edge form, CTR — 5 × `cflow-loop` |
| `pd.cpp` | `d_nest d_join d_break d_cont d_switch d_early d_goto d_cold` | block order, joins, `break`/`continue`/`goto`, `switch`, a cold arm |
| `pe.cpp` | `e_long`, generated with N stores | the long-branch threshold; N swept 4000/4200/4400/6000 |
| `pf.cpp` | `f_eq59 f_gt59 f_ne59 f_eqcall f_eq3 f_eqvoid f_eqzk` | the fold boundary — constant pair and relation varied against `pa.cpp` |

`work/w-cfg/objdis.py` is this lane's scratch COFF reader and PPC disassembler
(sections, symbols by record index, relocations, `BO`/`BI` decoded to extended
mnemonics). It is scratch, not tooling: the disassembly was cross-checked against
c2's own `.cod` listing on `pb.cpp` and `xboxmem.cpp`, which is the only reason
its output is quoted.

**Held-out cells**, compiled after §3's rules were written: `pe.cpp`'s sweep
(answered §3.3.1, which no other probe reaches) and `pf.cpp` (refuted the fold
rule that `pa.cpp` alone would have supported — `a_eq` folds, `f_eq59` with the
same relation does not).
