# `.ex` statement layer — what `0x53` is, the statement grammar, and the control-flow production

**Status: characterization only (2026-07-30). No code was changed by this
document.**

This is an independent re-derivation of the statement layer from a fresh probe
set, driven by the fact that `body-0x53` blocks the single missing function in
four separate near-miss TUs of the real workload (§7). It was written with
`docs/IL_STMT_GRAMMAR.md` (P2d) in hand: **every claim of that document that
this probe set touched was confirmed, none contradicted**, and cross-references
to it are marked **[SG §n]**. What is new here: the ternary production (§3.9),
the proof that statement boundaries cannot be found by byte-scan (§5), the
annotated decode of the real blocked `Primes.cpp` function (§4), the
token-ownership matrix against the current parser (§6), and a ranking re-measured
on the 2026-07-30 whole-workload scan (§7).

Evidence labels:

* **MEASURED [Pn]** — a probe in this document's corpus (`/tmp/stmtprobe/*.cpp`,
  regenerable from §8; captured with `c2rs census <cpp> --keep-il <dir>` against
  the live 16.00.11886.00 toolchain under wibo, default `/Bd /d2nop /Ox /GS- /c`
  census flags).
* **MEASURED [PRIMES]** — the real `src/system/math/Primes.cpp` TU of the
  dc3-decomp workload, captured with the workload's own flags
  (`--flags-file work/dc3-workload/flags.txt --cwd ../dc3-decomp`).
* **MEASURED [SCAN]** — aggregated from `work/dc3-workload/scan-20260730.jsonl`
  (878 TUs, `c2rs gap` at HEAD).
* **HYPOTHESIS** — stated as such, with the evidence that would settle it.

Byte spellings below elide the fixed 10-byte void CALL header
`bd 82 07 03 00 80 01 10 00 00` as `CALL` and the int type `86 41 74` as `<int>`
where the neighbourhood is not in question.

---

## 1. What `0x53` is

> **`53` opens one lexical scope. It takes no operand. `54 <k>` closes the
> innermost open scope, where `k` = the number of scopes still open after the
> pop.** It is not a statement separator, not a line marker, and it is emitted
> zero times per statement.

The rule is "one `53` per **scope entered**", where a scope is: the two
module/function scopes already open before the body **[SG §1]**, the function
body itself, every `{ }` block, every `if`/`while`/`for`/`switch` statement
(one for the statement itself), and every `if`/`else` **clause** (even
unbraced). Loop *bodies* get one only when braced (§3.4).

### 1.1 The probe ladder that pins it — MEASURED [P1 = p_count.cpp]

Every function on one source line (so no line markers intrude). Count of `53`
bytes from the body-start `4F 11` on, and the observed closes:

| source | `53`s | closes before fn-tail | witness (from `4f 11`) |
|---|---:|---|---|
| `void a0() {}` | 1 | — | `53 3a e5 09 54 02 29 e5 09` |
| `void a1() { gv(); }` | 1 | — | `53 26 e3 09 CALL 4c 4b 3a e7 09 54 02 …` |
| `void a2() { gv(); gv(); }` | 1 | — | `53 26 e3 09 CALL 4c 4b 26 e3 09 CALL 4c 4b 3a e9 09 …` |
| `void a3() { gv(); gv(); gv(); }` | 1 | — | three concatenated call statements, still one `53` |
| `void semi3() { ;;; gv(); }` | 1 | — | **byte-identical shape to `a1`** — `;` emits nothing |
| `void b1() { { gv(); } }` | 2 | `54 03` | `53 53 26 e3 09 CALL 4c 4b 54 03 3a ef 09 54 02 …` |
| `void b2() { { { gv(); } } }` | 3 | `54 04 54 03` | `53 53 53 … 4c 4b 54 04 54 03 …` |
| `void seq_blocks() { { gv(); } { gv(); } }` | 3 | `54 03 … 54 03` | `53 53 …4c 4b 54 03 53 …4c 4b 54 03 …` |
| `void deep5() { {{{{ gv(); }}}} }` [P3] | 5 | `54 06 54 05 54 04 54 03` | `53 53 53 53 53 … 4c 4b 54 06 54 05 54 04 54 03 …` |

### 1.2 The wrong rules, and the probe that kills each

* **"one `53` per statement"** — would give `a3` two more `53`s than `a1`.
  MEASURED: identical count (1). Statements are concatenated with no separator
  (§2).
* **"one `53` per empty statement / null statement marker"** — `semi3`'s three
  `;` emit **zero bytes**. MEASURED.
* **"`54 <k>`'s operand is a lexical block index"** — `seq_blocks`' two
  sequential blocks would close `54 03` then `54 04`. MEASURED: both close
  `54 03` (same depth). The nested `b2` is the contrast: `54 04 54 03`.
* **"`53` takes an operand"** — `if (a);` emits `53 54 04` (open, immediately
  close: the empty clause scope, MEASURED [P3 `if_semi`]), and `deep5` runs five
  `53`s back to back. Nothing between them. Across every probe the byte after
  `53` is the first token of a statement, another `53`, or a `54` — a field of
  width 0. INDISTINGUISHABLE from "no operand"; treated as none.
* **"count `0x53` bytes to count scopes"** — the byte is not position-free:
  `int x = a + 83;` embeds `53` as a *literal payload* —
  `26 f2 09 b9 ef 09 <int> 33 86 41 74 >53< 02 32 <int> 4b` (MEASURED [P4
  `s83`], 83 = 0x53). Only a width-correct tokenizer can count scopes.

### 1.3 Why the census byte-window shows `53` in already-parsed bodies

The current parser owns **exactly one** `53` — the body's own, at
`crates/c2-il/src/func/body/mod.rs:257` — and blocks on any second one
(`mod.rs:350` → `body-0x53`). So `53` is "unowned in some positions" precisely:
position 1 owned, every deeper scope unowned. In the task's `Primes.cpp` window
the leading `08` is the tail of the line marker `4f 01 08` (line 8) cut by the
window — MEASURED [PRIMES]: the capture reads
`… 4f 01 07 4f 01 08 4f 01 09 … 4f 01 0e 53 26 eb 09 …`.

---

## 2. The statement grammar as a byte layout

Confirms [SG §0] wholesale; every production below has a probe witness in this
corpus. The layer is a **flat token stream**, not a tree — one loop over:

```text
body        := 4F 11  53  item*  4F 12 47 54 01 54 00      (fn-tail; the last
item        := 4F 01 <varint>       source line number       two closes pop the
             | 53                   open scope                pre-body scopes)
             | 54 <k>               close scope; k = depth remaining after pop
             | 29 <tok>             define label <tok> here
             | 38 <tok>             pop condition, branch to <tok> if FALSE
             | 39 <tok>             pop condition, branch to <tok> if TRUE
             | 3A <tok>             unconditional branch to <tok>
             | 4B                   end expression statement, DISCARD the value
             | 3B <tok>             switch dispatch        (§3.8)
             | 3C <TYPE> <tok>      switch table header
             | 3D <tok>             switch case entry
             | <operand-stream token>                      (expression layer)

expr-stmt   := <postfix operand stream>  4B
assign      := <lvalue-push> <value>  32 <TYPE>  4B         (32 yields the value)
return-stmt := [ <value> 41 <TYPE> ]  3A <epilogue-label>   — NO 4B
declaration := (nothing)             bare `int x;` emits zero bytes
null-stmt   := (nothing)             `;` emits zero bytes   (MEASURED [P1])
```

Key statement-level facts, each with this corpus' witness:

* **Concatenation, no separator.** `a2` vs `a1` [P1]: the delta is exactly one
  16-byte call statement, nothing between.
* **`4B` discards.** `int f(int a){ int x = a+1; int y = x+2; return y; }`
  [P2 seg 12] is two `26 <dst> … 32 <int> 4b` statements then a return with no
  `4b` — the value-yielding `32` (assignment as expression, [SG §4.2]) is capped
  by `4B` at each statement end.
* **`x++` as a statement keeps its own opcode.** The real Primes loop increment
  is `26 eb 09 33 <int> 01 35 <int> 4b` [PRIMES] — postfix `35` yields a value
  that the `4B` then discards; it is not normalized to `+= 1` (`0F`) [SG §5].
* **Line markers `4F 01 <varint>` appear between any two items**, including
  runs (one per source line that emits nothing): the Primes body opens with
  eleven consecutive markers for the initializer lines 4–14, then `53`
  [PRIMES]. Payload past 127 is `80` + 4-byte LE i32: a function at line 132
  emits `4f 01 80 84 00 00 00` (MEASURED [P5 `p_late.cpp`]). The current
  `eat_opt_stmt_marker` (`crates/c2-il/src/func/readers.rs:225`) already loops
  and reads the varint — this is owned.

---

## 3. The control-flow production

**The port has no control-flow decode; this section names the production
precisely.**

> **A branch target is a label TOKEN — the same 2-or-4-byte token namespace as
> variables and functions (`readers.rs:67`), allocated by c1xx per function.
> There is no offset, no basic-block index, and no direction bit. `29 <tok>`
> defines the label at the current stream position; `38/39/3A <tok>` branch to
> it. Forward and backward references use identical bytes, and a label may be
> defined before or after its uses.**

### 3.1 The discriminator (token vs. relative offset vs. block index)

A *relative-offset* operand would differ when the same target is jumped to from
two different distances. MEASURED [P2 `c_break`]:

```
53 53  29 05 0a                      TOP:
       b9 01 0a <int>  38 06 0a      brFALSE -> 0x060A   (EXIT, 24 bytes ahead)
       53 26 e3 09 CALL 4c 4b
       3a 06 0a                      break   -> 0x060A   (11 bytes ahead)
       54 04  3a 05 0a  29 06 0a     backjump TOP; EXIT defined here
54 03 …
```

Two jumps to `0x060A` from different positions: **identical operand bytes**.
Same in `c_continue`: the `continue` and the loop's own back-jump are both
`3a 0b 0a` from different offsets. A *block-index* reading dies on `do/while`
[P2 `c_dowhile`], where the target `29 fe 09` is defined at a *shallower scope
depth* than the branch that targets it, and on the fact that label tokens
interleave with variable tokens in one ascending per-TU sequence (`0x09E3`,
`0x09E4`, … across functions) — they are symbols, not structure.

Backward jump = same opcode: `c_while` [P2] ends its body with `3a f2 09` to
the `29 f2 09` defined *before* the condition. A decoder therefore needs a
label→position map built in a first pass (or a fixup list); direction is not
in the bytes.

### 3.2 `if` — MEASURED [P2 `c_if`]

```
if (a) gv();
53                        if-statement scope
b9 e5 09 <int>  38 e8 09  cond; brFALSE -> Lend
53 26 e3 09 CALL 4c 4b    then-clause scope (opened even UNBRACED)
54 04  29 e8 09  54 03    close clause; Lend:; close if-scope
```

### 3.3 `if`/`else` — MEASURED [P2 `c_ifelse`]

```
b9 e9 09 <int>  38 ec 09      brFALSE -> Lelse
53 …then… 54 04
3a ed 09                      jump -> Ljoin  (emitted AFTER the then-close)
29 ec 09  53 …else… 54 04
29 ed 09  54 03               Ljoin:; close if-scope
```

### 3.4 `while` — MEASURED [P2 `c_while`, P3 `w_nobrace`]

```
53  29 TOP:  <cond>  38 EXIT   53? <body> 54?   3a TOP   29 EXIT:  54 <k>
```

Braced body gets its own `53 … 54`; **unbraced loop bodies do not** (contrast
with §3.2's unbraced-clause scope — the asymmetry [SG §7] reproduces here;
unexplained, and a decoder must not predict scope counts, only track them).

### 3.5 `for` — the increment is rotated ABOVE the condition — MEASURED [P2 `c_for`]

```
53                                        for-statement scope
26 f7 09 33 <int> 00 32 <int> 4b          i = 0            (init)
3a f8 09                                  jump COND
29 f9 09  26 f7 09 …33 <int> 01 02 32 …4b INCR: i = i + 1
29 f8 09  b9 f7 09 <int> b9 f4 09 <int> 22  COND: i < n    (22 = cmp-lt)
38 fa 09                                  brFALSE -> EXIT
53 … body … 54 04
3a f9 09  29 fa 09  54 03                 jump INCR; EXIT:
```

Byte order is init · INCR · COND · body — **not source order**. The real
Primes loop has the identical rotation [PRIMES] (§4).

### 3.6 `do`/`while` — MEASURED [P2 `c_dowhile`]

```
53  29 TOP:   53 53 …body… 54 04 54 03   29 CONT:  <cond> 39 TOP   29 EXIT:
```

TOP is defined at body depth **before any scope opens** — this is the
`body-0x29` census bucket (37,671 fns [SCAN]): the first item after the body's
`53` is a label definition, which `parse_segment_detail`'s dispatch
(`mod.rs:262`) has no arm for.

### 3.7 `break`, `continue`, `goto`, early `return` — zero new opcodes

All four are `3A <tok>` (§3.1 witnesses; `goto` in [P3 `g8`], reproducing
[SG §11]'s redundant `3A L; 29 L` fall-through pair). Early `return` targets the
**epilogue label** — one per function, allocated right after the function's own
token; `int c_early(int a){ if (a) return 1; return 2; }` [P2] has two
`33 <int> k 41 <int> 3a 0f 0a` returns and one `29 0f 0a` after the body
closes. A value-carrying return is `<value> 41 <TYPE> 3A <lbl>`; a void one is
bare `3A <lbl>`.

### 3.8 `switch` — MEASURED [P2 `c_switch`], confirms [SG §11]

```
b9 <a> <int>  3b 14 0a          dispatch on table symbol
53  29 <Lc1> …case1… 3a <Lx>  29 <Lc2> …case2… 3a <Lx>  29 <Ldef> …default…  54 04
3a <Lx>
26 14 0a  3c <int> <Ldef>       push table symbol; header: TYPE, default label
33 <int> 01 3d <Lc1>  33 <int> 02 3d <Lc2>  4c        (case value, entry)*, end
29 <Lx>  54 03                  EXIT
```

Table after the bodies, terminated by the same `4C` an argument list uses.

### 3.9 `&&`/`||`/`!` vs. the ternary — two different fates

**Short-circuit and `!` are pre-lowered to branches by c1xx** — opcodes
`1B`/`1C`/`1A` never appear in a condition. MEASURED [P2 `c_oror`,
`c_andand`]: `if (a || b)` is `b9 <a> 39 Lthen  b9 <b> 38 Lskip  29 Lthen: …`;
`if (a && b)` is two `38 Lskip` to the same label; `if (!a)` [SG §7] flips the
branch sense.

**The ternary is NOT lowered — it stays an expression node**: postfix
`<cond> <then-subtree> <else-subtree> 43 42 00 00`. MEASURED [P2 `c_ternary`,
P3 `t3`/`tnest`, P4 `tcall`/`tf`]:

```
a ? 3 : 4                 b9 <a> <int>  33 <int> 03  33 <int> 04   43 42 00 00
a ? b : c   (int, float)  three loads                              43 42 00 00
a ? (b?1:2) : 3           nested: inner 43 42 00 00 inside the then-subtree
a ? gi(1) : gi(2)         BOTH call subtrees serialized, then      43 42 00 00
```

The trailing `42 00 00` is byte-identical across int/float/nested/side-effecting
arms — **INDISTINGUISHABLE from a constant** on this corpus; whether `42` is a
second opcode and `00 00` a (never-used) token field is UNKNOWN. Evidence that
would settle it: a TU large enough that a genuine token would exceed `00 00`, or
a `43` in a context where the selected value's type must be spelled. Note the
serialized arm subtrees mean the stream is a **tree serialization, not an
execution trace** — the backend, not the front end, owes the ternary its
branch/`isel` lowering and its evaluation-order semantics.

---

## 4. The real blocked function, decoded end to end — MEASURED [PRIMES]

`NextHashPrime` (the whole of `Primes.cpp`, 294-byte segment), annotated under
§2/§3. The census blocks it at the `53` marked `>>`:

```
53                                        body scope
4f 01 04 … 4f 01 0e                       11 line markers (initializer lines 4..14)
>>53                                      for-statement scope        [§3.5]
26 eb 09 33 <int> 00 32 <int> 4b          i2 = 0
3a ec 09                                  jump COND
29 ed 09  26 eb 09 33 <int> 01 35 <int> 4b   INCR: i2++  (postfix 35)
29 ec 09                                  COND:
26 ea 09                                  push symbol `primes` (static array)
b9 eb 09 <int>                            load i2
33 86 41 12 04  04                        lit 4 (long-typed) ; MUL     — scale
28 00 00                                  byte-offset add (subscript)  — IL_EXPR_LAYER §4
30 <int>                                  indirect load  -> primes[i2]
33 <int> 00  20                           lit 0 ; cmp-ne
38 ee 09                                  brFALSE -> EXIT
53 4f 01 0f 53                            loop-body scope; line 15; if-scope
26 ea 09 …28 00 00 30 <int>               primes[i2] again
b9 e6 09 <int>  23                        load i ; cmp-ge
38 ef 09                                  brFALSE -> Lskip
53 4f 01 10  26 ea 09 …30 <int>           then-clause; primes[i2]
41 <int>  3a e8 09                        return primes[i2]  (epilogue jump)
4f 01 11  54 06  29 ef 09  54 05 54 04    closes; Lskip:; closes
3a ed 09  29 ee 09                        jump INCR; EXIT:
4f 01 13  b9 e6 09 <int> 41 <int> 3a e8 09   return i
54 03 54 02  4f 01 14  29 e8 09           closes; epilogue label
4f 12 47 54 01 54 00                      fn-tail
```

Every statement-layer byte is covered by §2/§3. The expression-layer residue
this one function needs beyond the current `parse_expr`: the `26 <sym>` data
push in operand position, the subscript `33 86 41 12 <size> 04 28 00 00`
(elem size moves the literal: 4/2/1 for int/short/char arrays, MEASURED [P6
`p_idx.cpp`]; the `28`'s trailing `00 00` is UNKNOWN, per `IL_EXPR_LAYER.md`
§4), the indirect load `30 <TYPE>`, comparisons feeding a branch (`20`, `23` —
opcode bytes already census-named at `mod.rs:143`), and postfix `35`. So
`body-0x53` names this function's *first* blocker, not its only one — the
bucket is an upper bound (§7).

---

## 5. Statement boundaries: can a body be split without understanding expressions?

**No.** The statement layer has no self-synchronizing separator:

* `4B` (statement end) occurs as a literal payload: `return 75;` is
  `33 86 41 74 4b 41 …` — MEASURED [P4 `k75`].
* `53` (scope open) occurs as a literal payload: `x = a + 83` — §1.2,
  MEASURED [P4 `s83`]. Both bytes also occur freely in tokens (`26 53 0a` would
  be a symbol push), type ids, and line numbers (line 75's marker is
  `4f 01 4b`).
* There is no length field anywhere in the layer, and `return` statements have
  no terminator at all (§2).

So a byte-scan split is impossible; **a deterministic split exists exactly when
every token's width is known**: statements end at `4B`, at a value-carrying
`3A` (return), or at a statement-layer item boundary (`53`/`54`/`29`/`38`/…),
*as encountered by a left-to-right tokenizer*. That tokenizer needs per-opcode
operand widths (`read_token_var`, `read_type`, `read_varint`) but **not**
semantics. The falsification standard is [SG §13]'s: a width-complete
skip-tokenizer landed exactly on the 7-byte fn-tail for 2792/5239 real bodies
with **zero off-tail landings** — every failure an honest stop at an unmodeled
expression opcode — while a fixed-3-byte-TYPE variant produced 34 silent
off-tail landings. Practical consequence for the port: parses are bounded by
*vocabulary coverage*, not by statement structure; the `54 <k> == depth` check
comes free and should be enforced as an integrity gate.

---

## 6. Token matrix

Owned = consumed by the current parser (citation). Characterized = pinned in a
characterization doc + this corpus, not yet in the parser. The task's window
bytes all appear here.

| bytes | meaning | status |
|---|---|---|
| `4C 4F 11` | `4C` closes the formals region, `4F 11` body-start | owned as an atomic anchor, `mod.rs:249` |
| `53` (position 1) | body scope open | owned, `mod.rs:257` |
| `53` (any deeper) | scope open | **unowned** → `body-0x53`; characterized §1 |
| `54 02 29 <tok>` | body-close + epilogue label, fixed | owned, `expr.rs:150` — but it is two productions and a line marker can sit between them [SG §9]; the fixed match under-accepts |
| `54 <k>` general | close innermost scope, `k` = remaining | **unowned**; characterized §1 (`k` plain-byte vs varint UNKNOWN, all observed `< 0x80`; track-and-compare sidesteps it) |
| `29 <tok>` free-standing | label definition | **unowned** → `body-0x29`; characterized §3 |
| `38 <tok>` / `39 <tok>` | branch if false / if true (pops condition) | **unowned**; characterized §3 |
| `3A <tok>` | unconditional branch (return/break/continue/goto/join) | owned in one position only (the return jump, `expr.rs:143`); general form unowned |
| `4F 01 <varint>` | source line number, loops | owned, `readers.rs:225` |
| `4F 12 47 54 01 54 00` | fn-tail (the `47` is a fixed byte, unexplained [SG §12.6]) | owned, `expr.rs:156` |
| `46 (2D <tok>)*` | formals, reverse declaration order | owned, `expr.rs:295` |
| `26 <tok>` + `BD` | callee push + call | owned, `mod.rs:284`, `shapes.rs:886` |
| `26 <tok>` + value + `32` | assignment destination push | owned for **formal** destinations only, `shapes.rs:52,117` |
| `26 <tok>` in operand position | **data-symbol push** (static/global/array base), not only a call | **unowned**; census names the byte `expr-call-in-expr` (`mod.rs:173`), which is a partial misnomer — MEASURED [PRIMES, P6] it also opens subscripted array reads; the call/data split of that 275k bucket is UNMEASURED |
| `B9 <tok> <TYPE>` | load named value | owned int-like only, `expr.rs:222`; other TYPEs → `expr-load-type-*` |
| `33 <TYPE> <varint>` | typed literal | owned int-like only, `expr.rs:237` |
| `86 41 74` | the `int` TYPE triple (general form `<tag> <kind> <LEB128>`, 3–5 bytes) | owned, `readers.rs:2,161,292` |
| `32 <TYPE>` | store; yields the stored value | owned inside assign bodies, `shapes.rs:120` |
| `4B` | end expression statement, discard value | owned only glued into `4C 4B` / after `32` (`shapes.rs:125,950,1011`); as a general item terminator unowned |
| `41 <TYPE>` | result-value annotation before a return jump (annotate-vs-convert UNKNOWN [SG §12.3]) | owned, `expr.rs:137` |
| `55 <TYPE>` / `4C` | call-argument push / argument-list end | owned, `shapes.rs:974-977` |
| `0F–13, 15–19` | compound assignment (`+=` … `\|=`); `0x14` unobserved — do not fill | **unowned**; table with per-operator witnesses [SG §5] |
| `35` / `36` | postfix `++` / `--` (yield a value) | **unowned**; witnessed here in the real workload [PRIMES] |
| `43 42 00 00` | ternary select node, postfix over 3 subtrees | **unowned**; characterized §3.9; census-named `mod.rs:172` |
| `3B / 3C <TYPE> <tok> / 3D <tok>` | switch dispatch / table header / case entry | **unowned**; characterized §3.8 |
| `30 <TYPE>` | indirect load | owned only inside the IndirectLoad leaf (`shapes.rs:593`) |
| `28 00 00` | byte-offset add (subscript); trailing 2 bytes UNKNOWN | **unowned**; `IL_EXPR_LAYER.md` §4, elem-size witness [P6] |
| `08` in the task's window | not a token — payload of the truncated line marker `4f 01 08` | resolved §1.3 |

---

## 7. Order of work

Corpus for the ranking: **[SCAN]** `work/dc3-workload/scan-20260730.jsonl` —
878 real TUs, 2,353,102 blocked functions, first-blocker histogram. Two
honesty caveats, both load-bearing:

1. **A bucket counts functions whose FIRST blocker is that feature** — its size
   is an **upper bound** on the win. §4 is the worked example: the `body-0x53`
   poster child also needs subscripts, comparisons and postfix `35` before it
   emits.
2. **Decode ≠ emit.** Everything below is decode work; emission stays gated on
   whitelisted shapes, and a decoded CFG in particular is not a lowered one
   (loops need regalloc across a back edge). [SG §14.1] measured the trap:
   statement layer alone moved Dir.cpp by **+7 bodies**; jointly with the
   expression-type gate it was worth +812.

The ranked order:

1. **Scope stack + labels + branches** (`53`, `54 <k>` with the depth check,
   `29`, `38`, `39`, general `3A`) — one coherent production family, §1+§3.
   Upper bound: `body-0x53` **170,401 fns / 845 TUs** (#2 bucket overall) +
   `body-0x29` **37,671** + `fn-tail-0xB9` **29,552** (statements after the
   epilogue label — [SG §9.1]'s 7.7%-of-bodies tail work) ≈ **238k functions,
   10.1% of all blocked**. Of the nine TUs on the scan that are exactly ONE function short of full
   coverage, **four block on `body-0x53`** (`Primes.cpp`, `osfinfo.cpp`,
   `undname.cpp`, `vswprnc.cpp`) and three more on `assign-dst-not-formal`
   (step 2) [SCAN]. This is the single production the port cannot
   currently even *census past*, so it also unblocks honest measurement of
   everything beneath it.
2. **The statement list generalized** — `4B` as a first-class terminator,
   assignment to non-formals (needs a positive local-vs-global signal or a
   fail-closed refusal that names itself; `assign-dst-not-formal` is 5,534 fns
   and 3 near-miss TUs [SCAN]), statements after `54 02`.
3. **Comparisons in branch position** (`1F`–`24` feeding `38`/`39`) — opcodes
   already census-named and pinned (`mod.rs:143`); small standalone buckets
   (cmp-eq 3,762) but required by both #1's loops and the W6-style leaves.
4. **Data-symbol operand + subscript + `30 <TYPE>`** (`26`-as-value,
   `28 00 00`) — bites into the **#1 bucket** `expr-call-in-expr` (275,829),
   an unknown fraction of which is data pushes, not calls (§6); finishes
   `Primes.cpp` together with 1–3 and postfix `35`.
5. **Compound assignment family** (`0F`–`19`, `35`, `36`) — twelve witnessed
   opcodes, nearly free to decode, needed by real loop bodies (Primes' `i2++`);
   near-zero standalone unlock ([SG §14.2] step 4 measured +2 bodies).
6. **Ternary `43 42 00 00`** — zero first-blocker sites on the current scan
   (`expr-ternary`: 0 [SCAN]; it was 93 on Dir.cpp before other buckets moved
   in front of it). Decode-only recognition is cheap; lowering is backend work
   (§3.9).
7. **`switch`** — `body-0x3B` first-blocker count is **0** [SCAN]; grammar
   recorded (§3.8) so it can be refused precisely, implemented last.

Not statement-layer, but ranked above all of it by prior measurement: the
operand-TYPE widening ([SG §14.1]: 3.2× on Dir.cpp) — steps 1–2 are what make
its census attribution trustworthy.

---

## 8. Reproduction

```sh
cargo build --release -p c2-harness
mkdir -p /tmp/stmtprobe && cd /tmp/stmtprobe
# P1 p_count.cpp   — a0..a3 statement ladder, semi3, b1, b2, seq_blocks   (§1)
# P2 p_ctl.cpp     — c_if, c_ifelse, c_while, c_for, c_dowhile, c_break,
#                    c_continue, c_early, c_switch, c_oror, c_andand,
#                    c_ternary, f (the task's 2-assign body)              (§2,3)
# P3 p_extra.cpp   — w_nobrace, t3, tnest, g8 (goto), deep5, if_semi      (§1,3)
# P4 p_tern2.cpp   — tcall, tf, k75, s83                                  (§3.9,5)
# P5 p_late.cpp    — a function at source line 132 (wide line marker)     (§2)
# P6 p_idx.cpp     — static int/short/char array reads                    (§4)
./target/release/c2rs census /tmp/stmtprobe/pN.cpp --keep-il /tmp/stmtprobe/il_N
python3 work/exp/tools/dump.py /tmp/stmtprobe/il_N        # raw segment hex

# the real near-miss TU
./target/release/c2rs census src/system/math/Primes.cpp \
    --flags-file work/dc3-workload/flags.txt --cwd ../dc3-decomp \
    --keep-il /tmp/stmtprobe/il_primes

# the ranking (first-blocker histogram + near-miss TUs)
python3 - <<'EOF'
import json, collections
tot = collections.Counter(); near = []
for line in open('work/dc3-workload/scan-20260730.jsonl'):
    j = json.loads(line); b = j.get('fn_blockers') or {}
    for k, v in b.items(): tot[k] += v
    if j['fn_total'] - (j['fn_in_class'] or 0) == 1 and b:
        near.append((j['src'], list(b)))
print(*tot.most_common(25), sep='\n'); print(near)
EOF
```
