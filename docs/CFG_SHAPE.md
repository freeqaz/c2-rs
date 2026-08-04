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

## 4. The minimal instance — `cflow-if-1`, in full

## 5. The widening order, ranked by TUs

## 6. What the block/instruction IR must carry

## 7. The `/FAsc` listing as a decode aid

## 8. What an implementer still cannot build from this document

## 9. Proposed board rows

## 10. Reproducing this
