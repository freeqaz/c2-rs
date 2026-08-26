# WB_OPCLASS — limb 2 CLOSED, and the read it needed had been taken four times

**Lane `w-opclass`, wave 12, 2026-08-26. Characterization lane, docs-only.**
Prereg: [`../rungs/_2026-08-26-w-opclass-prereg.md`](../rungs/_2026-08-26-w-opclass-prereg.md),
committed first (`c0c2e7c21`), before the first image byte was decoded for this
lane. Board **#3585**–**#3590**. Funded by
[`../DECISIONS_2026-08-22.md`](../DECISIONS_2026-08-22.md) **decision 14**.

> **PROVENANCE — TWO HALVES, KEPT APART, `WB_ILARMS_MAP.md`'s convention.**
>
> * **`[R]`** — read from `compilers/X360/16.00.11886.00/c2.dll`, sha256
>   `c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`,
>   verified by every instrument before the first address is quoted. `[R]` says
>   *"the instructions were read correctly"*; it does **not** say *"this is what
>   c2 does"*.
> * **`[src]`** — read from **this repository** (the port's readers, the capture
>   cache's IL, prior lanes' pages).
>
> **This lane adopts NOTHING into `crates/` and owes ZERO `DISCLOSURE.md`
> rows.** §9 states the count, checked rather than asserted, and what a future
> adopter would owe.

**Instruments, all three re-runnable:**

```
python3 docs/whitebox/scripts/dump_opclass.py <c2.dll> --verify|--classmap|--arms|--prims|--cross
python3 docs/whitebox/scripts/cross_opclass_port.py <c2.dll> [--counts|--controls]
python3 docs/whitebox/scripts/scan_esc43.py <cache-index.tsv> [--walk N|--control <key>]
```

Committed output: [`labels/opclass_arms.txt`](labels/opclass_arms.txt),
[`labels/opclass_limb2.txt`](labels/opclass_limb2.txt),
[`labels/opclass_esc43_scan.txt`](labels/opclass_esc43_scan.txt).

---

## 0. The one-paragraph answer

Limb 2 is **closed on all 68 `MATCHED*` rows**, mechanically and re-derivably:
**35 `MATCHED` · 30 `NARROW(fields)` · 2 `WIDE(fields)` · 1 `UNRESOLVED`**, so
**33 of 68 change verdict** and, on the 65 `w-ilarms` called genuinely
unchecked, **30 of 65** — inside the registered `[22, 34]`. The count is
dominated by **one shared primitive**: 26 of the 33 are `readers::read_type` vs
c2's TYPE word, which is four separate narrownesses in one function, and the
whole set has **8 distinct root causes** over 68 opcodes. `0x28` is
**CONFIRMED** `NARROW(fields)`. `0x43` is **confirmed not an escape** and is one
of the two `WIDE(fields)` rows; the other is `0x2c`, which `w-ilarms` left
`UNRESOLVED` and which over-reads by four bytes on a legal payload. And the
`0x43` **hazard is not witnessed**: a token walk of 867 workload sources found
**2,404 real `43 42` sites and zero wide tokens** — every top-level `0x42` in
the workload is preceded by a `0x43`, so the port's pairing model is
empirically exact and its `+4` is right on every site that exists. **The read
this lane was funded to take had already been taken four times**, and §1 is
about that.

---

## 1. THE READ WAS ALREADY ON THE SHELF — FOUR TIMES, AND THE TREE'S OWN INDEX SAYS SO

Disclosed in the prereg §0 before any measurement, because it contaminates every
prediction about *what the arms say*.

Decision 14 funds this lane as *"the cheapest large closure on the board: 65
unchecked rows, **one budgeted read**, and a live hazard"*. The read was not
outstanding. Five surfaces in this tree already carried it, on four separate
occasions:

| where | what it already had | dated |
|---|---|---|
| [`WB_READER_FINDINGS.md`](WB_READER_FINDINGS.md) §3 | **all 29 class arms with their VAs and a one-line grammar each** | 2026-08-08 |
| `BOARD.md` **#1591**/**#1592** | the same, plus **nine** port/c2 width disagreements — including `0x28` and *"`0x43` is **not an escape**"*, both of this lane's named hazards | 2026-08-08 |
| [`READ_PLAN_2026-08-21.md:73`](READ_PLAN_2026-08-21.md) | *"29-entry jump table `0x10b3d954`, **all 29 arms read**"* — in the **already-read** section | 2026-08-21 |
| `work/wb-eh/extok.py` (committed) | a **working Python tokenizer** that reads the class table out of the image and applies all 29 arms plus the primitives | 2026-08-08 |
| [`WB_EH_FINDINGS.md`](WB_EH_FINDINGS.md) §4.2 | that tokenizer's output — a 41-token walk of `Main.cpp`'s body with no desync | 2026-08-08 |

**And `WB_ILARMS_MAP.md` cites the first of them, twice, in the section that
declares the read outstanding.** Its §6.2 quotes `WB_READER_FINDINGS.md` for
`0x2c` (class `05`) and `0x54` (class `0d`) as *"prior art, NOT adopted as a
premise"* — the correct instinct, applied to two rows — while §6 and §7 publish
*"reading `0x10b3d954`'s 29 class arms would close all of them at once and is
the single cheapest follow-up this map exposes"*.

**The sharpest instance is inside one file.** `w-ilarms`'s consumer sweep
amended `READ_PLAN_2026-08-21.md` at **`:99`** and **`:174`**.
**`:73` of the same file says the arms are read.** A lane opened the tree's own
read-index, edited two rows in it, and did not read the row 26 lines above the
first edit.

### 1.1 What this does and does not change

* **It does not make the lane's work redundant.** Nobody had ever crossed the
  class grammar against the port's per-opcode readers over the handled set.
  `WB_READER_FINDINGS.md` §3.4 did **nine positions** and says so in its own
  heading. The cross is 68, and §4 is it.
* **It does change the pricing sentence.** *"One budgeted read"* was a price for
  something already owned; the real cost of this lane was the **cross** and the
  **workload walk**, neither of which decision 14 names.
* **The correct generalisation is not "grep harder".** `#3546` established that
  a topic grep does not reach `docs/whitebox/ref/`; this is one shelf over and
  a topic grep *would* have reached it — `WB_READER_FINDINGS.md` sits in the
  same directory as `WB_ILARMS_MAP.md` and was **cited by it**. What failed is
  narrower and more fixable: **a page that says "X is unread" is a claim, and
  the tree has an index of what is read.** `READ_PLAN_2026-08-21.md` §1 exists
  to answer exactly that question and was not consulted as an *answer*, only
  edited as a *consumer*. Board **#3585**.

---

## 2. The dispatch, re-derived from ONE address `[R]`

`dump_opclass.py` hard-codes `0x10b3d610` — the operand decoder's entry — and
derives everything else from the operand bytes of the instructions it decodes
there. `w-ilarms`'s rule, and the reason is unchanged: a script that hard-codes
the table addresses can only test that the bytes at them have not moved.

### 2.1 The head, decoded byte by byte `[R]`

```
10b3d610  55                        push ebp
10b3d611  8b ec                     mov ebp,esp
10b3d613  83 ec 28                  sub esp,0x28
10b3d616  a1 04 e0 c2 10            mov eax,ds:0x10c2e004
10b3d61b  33 c5                     xor eax,ebp
10b3d61d  89 45 fc                  mov DWORD PTR [ebp-0x4],eax
10b3d620  53                        push ebx
10b3d621  8b 5d 08                  mov ebx,DWORD PTR [ebp+0x8]
10b3d624  89 0b                     mov DWORD PTR [ebx],ecx      ; *node = opcode
10b3d626  0f b6 81 48 5e b2 10      movzx eax,BYTE PTR [ecx+0x10b25e48]
10b3d62d  56                        push esi
10b3d62e  57                        push edi
10b3d62f  8b f2                     mov esi,edx
10b3d631  83 f8 1c                  cmp eax,0x1c
10b3d634  0f 87 07 03 00 00         ja 0x10b3d941
10b3d63a  ff 24 85 54 d9 b3 10      jmp DWORD PTR [eax*4+0x10b3d954]
```

| derived | value | from |
|---|---|---|
| class table | `0x10b25e48` | the `movzx` displacement |
| class bound | `0x1c` → **29** classes `0x00`…`0x1c` | the `cmp` + unsigned `ja` |
| out-of-range arm | `0x10b3d941` | the `ja` rel32 |
| class jump table | `0x10b3d954`…`0x10b3d9c7`, stride 4, 29 entries | the `jmp` displacement |
| epilogue | `0x10b3d92e` | derived: the block ending `leave; ret` |

### 2.2 The checks, and what each excludes `[R]`

| # | check | result | the alternative it kills |
|---|---|---|---|
| 1 | **are the 29 targets distinct?** | **27 of 29** — `0x10b3d922` is shared by classes `0D`/`11`, `0x10b3d941` by `10`/`16` | *the check `w-ilarms` ran on the record table and nobody ran on this one.* "29 entries" is not "29 arms"; here it is **27** |
| 2 | containment | **29 of 29** targets inside `[0x10b3d610, 0x10b3d954)` | a target escaping the function |
| 3 | the refusal | **2** classes (`10`, `16`) route to the `ja` destination — **27 real arms** | a refusal reachable only from out of range |
| 4 | packing | body ends `0x10b3d954`, table starts `0x10b3d954` — a **zero-byte** gap | a table somewhere else that happens to decode |
| 5 | **the class table's own extent, from its bytes** | the first byte exceeding the bound is at opcode **`0xCC`** (class `0xAC`) — so the bytes make it a table over **`0x00`…`0xCB`, 204 entries** | see below |

**Check 5 is a correction to `#1591`, and a small one.** That row publishes the
class table as *"192 entries, `0x00`–`0xBF`"*. The **bytes** stay legal to
`0xCB`; `0xBF` is a *choice* (it is one past the dispatch domain's last opcode
`0xBD`) and not a boundary the table forces. Nothing downstream depends on the
difference — every opcode this dispatch can reach is `≤ 0xBD` — but a published
extent narrower than the bytes' own needs its citation, and `#1591`'s did not
carry one. Board **#3586**.

### 2.3 The class table agrees with `dump_ilarms.py` on all 95 handled opcodes

`dump_opclass.py --cross` re-runs `w-ilarms`'s independent reader: the class
table VA, the class byte for every one of the 95 handled opcodes, and the count
of handled opcodes whose class exceeds the bound (**0**). `ALL AGREE`. **22**
distinct classes over the handled set, which is `w-ilarms` §6's number
re-derived by a different path.

---

## 3. The 29 arms and the 8 primitives, DERIVED `[R]`

Full listing: [`labels/opclass_arms.txt`](labels/opclass_arms.txt). The walker
records a conditional as its **own row** with both successors, so a gated read
is visible rather than silently taken or silently dropped, and it derives
*"this callee does not return"* from the arm table itself — the return address
of class `0B`'s `call 0x10b33526` is **class `0C`'s own entry**, which the
compiler can only lay out that way for a noreturn call.

### 3.1 The stream primitives, and their width functions `[R]`

Each derived by decoding the primitive's own body and counting cursor bumps
(`inc ecx` on `[0x10c46310]`) per path:

| VA | name | width | derivation |
|---|---|---|---|
| `0x10c1f8fc` | GetByte | **{1}** | one bump, no branch |
| `0x10c1f90a` | skip | **{1,2,3,…}** | `do { b = *p++ } while (b & 0x80)` — unbounded |
| `0x10c1f91b` | varU | **{2,4}** | two bumps, then `jns` past two more iff byte 1 has bit 7 |
| `0x10c1f9a6` | i16c | **{1,3}** | signed byte, or **exactly** `0x80` + 2 LE |
| `0x10c1f9e9` | i32c | **{1,5}** | signed byte, or **exactly** `0x80` + 4 LE |
| `0x10c1fae7` | i64c | **{1,9}** | signed byte, or **exactly** `0x80` + 8 LE |
| `0x10c1fc5b` | str | bounded NUL run | `0x10b33526` on overrun |
| `0x10c1feef` | dec10 | **{10}** | `0x10c1fc8c(8)` then `0x10c1fc8c(2)` — raw copies |
| `0x10b3d546` | TYPE | word{1,2,3} + [i32c] + [skip] | §3.2 |

**Two of these are the port's, exactly.** `readers::read_token_var` is
`varU`'s width function bit for bit, and `readers::read_varint` is `i32c`'s —
signed short byte, `0x80` escape, 4 LE bytes. Both were reached from captures
alone, years before anyone opened the image, and both are right. That is worth
as much as the disagreements.

### 3.2 The TYPE field is three reads, not one `[R]`

`FUN_10b3d546` touches the stream in exactly three places:

1. `0x10b3d550` `call 0x10c1fe40` — **the type word**: `b1 < 0x80` → **one
   byte**; `b1 & 0x40` → **three** (`((b2&0x7f)<<16) | ((b1&0x7f)<<8) | b3`);
   else **two**.
2. `0x10b3d59f` `call 0x10c1f9e9` — an **i32c**, iff `(v & 0xf) == 6` (aggregate)
   **and** `((v >> 4) & 0x1f) == 0` (the inline 5-bit size escaped).
3. `0x10b3d5b4` `call 0x10c1f90a` — a **skip run**, iff
   `[DAT_10c472e8 + 0xcac] != 0`.

Everything after `0x10b3d5b9` is classification and writes no stream. This
reproduces `WB_READER_FINDINGS.md` §3.2 instruction for instruction, at a second
independent decode.

### 3.3 The replication against `WB_READER_FINDINGS.md` §3 — 29 of 29, with one imprecision

Prereg **R3** was registered at p = 0.60 precisely because `#3547` had found a
prior page's cell wrong in both clauses. **It holds on all 29 rows in
substance.** One row is imprecise in a way that matters, and it is the row that
makes `0x33` `UNRESOLVED` below:

> §3's class `06` reads *"`TYPE`; then payload **by decoded type**: class `5`
> (real) → the 12-byte decimal path; else if **the type's** low 12 bits are `8`
> → `i64c`; else → `i32c`"*.
>
> **The predicate is not on the type.** `0x10b3d6c2` loads
> `node[+4]` — the **lowered** word written by `FUN_10b3d40a` at the end of the
> TYPE reader — and tests `& 0xf000 == 0x5000` (`0x10b3d6d3`) and then
> `& 0x0fff == 8` (`0x10b3d794`). The type word and the lowered word are
> different fields, and the difference is exactly why the port's own `0x33`
> discriminator cannot be compared to c2's without reading `FUN_10b3d40a`.

Board **#3587**.

---

## 4. LIMB 2, CLOSED — the cross, and its four controls

`cross_opclass_port.py`. c2's side is derived at run time by importing the arm
walker; the port's side is an `[src]` table of field sequences. Full output:
[`labels/opclass_limb2.txt`](labels/opclass_limb2.txt).

**Four controls run before any verdict is printed, and all four pass:**

| control | result |
|---|---|
| the port table covers **exactly** the 68 `MATCHED*` opcodes of `labels/ilarms_portmap.txt` | PASS (68/68, 0 missing, 0 extra) |
| every cited `control_flow.rs` line **equals the line re-derived** from the `match` arm that names the opcode | PASS (0 drifted) — `#3367`'s failure mode closed by derivation, not by care |
| every callee the arm walk produced is named | PASS |
| every named VA is one the arm walk produced | PASS (no stale address) |

### 4.1 The counts, with their denominators

**Over the 68 `MATCHED*` opcodes** (`WB_ILARMS_MAP.md` §4's denominator):

| limb-2 verdict | count | of |
|---|--:|--:|
| **MATCHED** | **35** | 68 |
| **NARROW(fields)** | **30** | 68 |
| **WIDE(fields)** | **2** | 68 |
| **UNRESOLVED** | **1** | 68 |
| → change verdict away from `MATCHED*` | **33** | 68 |

**Over the 65 `w-ilarms` called genuinely unchecked** (the 68 less `0x28`,
`0x2c`, `0x54`):

| limb-2 verdict | count | of |
|---|--:|--:|
| **MATCHED** | **35** | 65 |
| **NARROW(fields)** | **28** | 65 |
| **WIDE(fields)** | **1** | 65 |
| **UNRESOLVED** | **1** | 65 |
| → **change verdict** | **30** | **65** |

**The strict field-COUNT reading, published as a second denominator so both are
available:** the port's field count equals the class's on **57 of 68**. The
width-function reading is primary, and that is `w-ilarms`'s own precedent rather
than this lane's choice — its §6.1 calls `0x28` `NARROW(fields)` on a row where
the counts are equal and only the width function differs.

### 4.2 The rows that changed, by ROOT CAUSE — 8 causes over 33 rows

Ordered by cause, not by mass (the count is the count; it ranks nothing).

| rows | cause | what it is |
|--:|---|---|
| **26** | `ty` vs `TYPE` | **one primitive**, `readers::read_type`, four narrownesses — §4.3 |
| 1 | `vint` vs `GetByte` | `0x2c`, class `05` — **`WIDE(fields)`**, §4.4 |
| 1 | `esc43` vs (nothing) | `0x43`, class `00` — **`WIDE(fields)`**, §6 |
| 1 | `fix00` vs `varU` | `0x28`, class `02` — §5 |
| 1 | `b` vs `i32c` | `0x54`, class `0D`: the port reads a fixed byte where c2 reads an i32c, so at a depth of `0x80` c2 takes 5 and the port takes 1. `IL_STMT_GRAMMAR.md` §12.1's *"byte-vs-varint UNKNOWN"* is **resolved: it is an i32c** |
| 1 | `desc` vs `i32c` | `0x66`, class `1A`: `eat_class_descriptor` reads the arity as **one unsigned byte**; c2 reads an **i32c**, whose short form is **signed**. At `0x80` c2 takes 5 bytes and the port takes 1; at `0xFF` c2 reads `−1` (zero elements) and the port reads 255 and eats 255 LEBs |
| 1 | `end4F` vs `i16c` | `0x4f`, class `0C`: the port has **no general `0x4F` reader** — `step` ends the statement list and `codec.rs` recognises four fixed `4F` shapes, against c2's `i16c` sub-record code plus a format-string interpreter over 64 field codes (`ref/P_SUB4F.md`). Fail-closed |
| 1 | `lit` vs `dec10` | `0x33`, class `06` — **`UNRESOLVED`**, §4.5 |

**33 rows, 8 causes, and one of them is 26 of the 33.** That is the number to
carry: limb 2 is not 33 independent facts about 33 opcodes, it is **one shared
TYPE reader** and **seven singletons**.

*(Registered **B5** predicted ≤ 6 causes. It is **8** — a **MISS**, recorded
plainly. The cause of the miss is that the bracket counted the port's primitive
inventory (5 kinds) and not the *comparison* inventory: a one-off port arm
standing opposite a distinct c2 primitive is its own cause, and there are four
of those.)*

### 4.3 The 26 rows are ONE function with FOUR narrownesses `[R]` vs `[src]`

`readers::read_type` against `FUN_10b3d546`, all four fail **closed** (the walk
blocks; nothing is emitted):

1. **The one-byte type word.** c2: `b1 < 0x80` → the word is `b1`
   (`0x10c1fe98`). Port: `if tag & 0x80 == 0 { return None }`. `WB_READER_FINDINGS.md`
   §5.3(3) already **obj-confirmed** the one-byte form against real `c2.dll`
   (`26 C3 8B 20` byte-identical), so this is a narrowness with a cell behind it.
2. **The three-byte word's middle byte.** c2 masks `b2` with `0x7f`
   (`0x10c1fe72`) and never tests its bit 7. The port **requires** it set
   (`TYPE_WIDE_MARK_BIT`) and refuses otherwise.
3. **The aggregate escape.** c2 reads a plain `i32c` (`0x10b3d59f`). The port
   refuses any value `< 32`.
4. **The trailing run.** c2's is an unbounded `skip`. The port's LEB loop refuses
   past `shift > 28`, i.e. past 5 bytes.

**And a fifth difference that is NOT a narrowness — it is a BAKED
ENVIRONMENT**, §7.

### 4.4 `0x2c` is `WIDE(fields)` — `w-ilarms` left it `UNRESOLVED` and the arms decide it

Class `05` is `TYPE` then **one raw `GetByte`** (`0x10b3d6b4`, `0x10b3d694`).
The port reads `Scan::vint`, which takes **five** bytes when the payload byte is
`0x80`. So on a legal class-`05` record with payload `0x80` the port advances
four bytes further than c2 and **continues** — a desync, not a refusal.

This is `#1592`'s *"latent desync at any payload ≥ `0x80`"* re-derived
independently and **sharpened**: the trigger is not `≥ 0x80`, it is **exactly
`0x80`**, because `i32c`'s short form is a *signed* byte and only the value
`0x80` escapes. `0x81`–`0xFF` are negative payloads on both sides and agree.
`WB_READER_FINDINGS.md` §5.4 designed a probe for this and did not run it; it is
still unrun, and it is now the cheapest cell on this page. Board **#3588**.

### 4.5 `0x33` is the one `UNRESOLVED`, and the read that closes it is named

Class `06` branches on `node[+4]`, the **lowered** word (§3.3). The port's
`lit_payload` branches on the **raw** bytes: `kind & 0x0f == 0x0a` selects the
10-byte real path, and `tag == 0x88` selects the 8-byte escape.

* the **widths agree** on every branch — 10 for a real, 1-or-9 for an 8-byte
  scalar, 1-or-5 otherwise;
* the **discriminators are different fields**, and whether they select the same
  branch on every input needs **`FUN_10b3d40a` (`0x10b3d40a`)** and
  **`FUN_10c1fe9d` (`0x10c1fe9d`)**, which this lane did not read.

Registered **B4** predicted at least one row would stay `UNRESOLVED` and named
`0x4f` as the candidate. `0x4f` **resolved** (`NARROW`, §4.2) and `0x33` is the
one. The prediction hits; the candidate was wrong, and that is recorded rather
than quietly re-pointed.

---

## 5. `0x28` — CONFIRMED `NARROW(fields)`, and one caveat under the argument

**Confirmed.** Class `02`'s arm (`0x10b3d64d`) calls `varU` (`0x10b3d65f`) and
resolves it through the TU symbol table (`0x10b99977`). The port hard-codes
`28 00 00` (`control_flow.rs:1033`) and refuses anything else. `00 00` **is** a
`varU` of 0, so the port accepts exactly one of the values c2 accepts. It
**refuses**, which is the correct direction — the same shape as the four TYPE
narrownesses and not the shape of `0x2c` or `0x43`.

**The caveat is about the class, not the row.** `w-ilarms` §6.1 argues from
*"all six of its class-`02` siblings take a variable-width token"*. The six do,
and the sentence stands. But **class `02` is not uniform**: at `0x10b3d64d` the
arm tests the global `DAT_10c67fc0` and, if it is zero, tests the opcode against
`0x42` and takes **no operand at all** (`0x10b3d659` → the epilogue). So the
class has a seventh handled member whose grammar is *conditionally empty*, and
the method — *"opcodes sharing a class share a payload grammar"* — is true of
this class only in one environment. It does not change `0x28`'s verdict; it
changes what the class-uniformity cross-check is worth in general. Registered
**S2** at p = 0.50: **HALF-HIT** — the sentence needs no correction, its premise
needs a caveat. Board **#3589**.

---

## 6. `0x43` — no escape, and the hazard is NOT WITNESSED

### 6.1 The escape does not exist `[R]`

`0x43` is class **`00`**: the arm target is the epilogue itself
(`0x10b3d92e`) — the payload-free class *is* the fall-through to the exit, which
is a fact the arm table states rather than a reading of it. There is no
sub-opcode table anywhere in this dispatch.

`w-ilarms` §7.2's arithmetic reproduces exactly, and so does `#1592`'s:

| the port's "escape" | c2's two tokens | width |
|---|---|---|
| `43 42 XX XX` → `+4` | `43` (class `00`) then `42` (class `02`, a `varU`) | `1 + (1 + 2) = 4` ✔ |
| `43 37` → `+2` | `43` (class `00`) then `37` (class `00`) | `1 + 1 = 2` ✔ |

**The port is right by coincidence, and it is right in two directions at once.**
Registered **H2** predicted a second way the fixed `+4` is wrong, one that makes
the true width *narrower* than 4. **HIT**: in the environment where
`DAT_10c67fc0 == 0`, opcode `0x42` takes **no operand**, so `43 42` is **two**
bytes and the port's `+4` over-reads by 2 — the opposite end of the same fixed
constant that over-reads by 2 on a wide token.

### 6.2 Is the wide-token hazard reachable? — measured, and the answer is NO

Registered **H3** at p = 0.40. The method was fixed in the prereg before the
scan: an over-counting raw scan first, then a walk, and the raw count reported
as an upper bound beside it.

**The upper bound** ([`labels/opclass_esc43_scan.txt`](labels/opclass_esc43_scan.txt)),
over **3,036 distinct `.ex` streams** from **870** workload sources, **2.98 GB**:

| | count |
|---|--:|
| raw `43 42` byte occurrences (an **upper bound**) | **762,191** |
| … byte at `+4` opens one of the 95 handled opcodes (narrow-consistent) | 757,248 |
| … byte at `+3` has bit 7 set (would be a **wide** `varU`) | **238** |
| … and the byte at `+6` opens a handled opcode | 185 |
| … and `+4` does **not** — readable **only** as wide | **2** |

**Both of the 2 were decoded by hand and both are false positives.** Their
context is identical:

```
4f 02 20 00 4f 01 08 53 53 26 | 43 42 b9 ad 49 a6 43 be 33 99 …
```

— `53 53 26`, so the `43 42` is the **two-byte payload of a `26` symbol-push
token** (value `0x4243`) and not a `0x43` opcode at all. The raw scan found what
a raw scan finds.

**The walk.** The grammar this lane derived is turned into a walker
(`scan_esc43.py --walk`) that runs from each `4C 4F 11` body marker to the
function tail or the `4D` end of stream, one token at a time, every width taken
from the class arms:

| | count |
|---|--:|
| workload `.ex` streams walked (one per source) | **867** |
| bodies walked clean to a tail | **567,367** (23.5 % of candidate markers) |
| **top-level `0x42` tokens** | **2,404** |
| … whose `varU` is **wide** | **0** |
| **`43 42` sites** (a `0x43` immediately before) | **2,404** |
| … whose `varU` is **wide** | **0** |
| `43 42` sites in the in-sync **prefix** of a body the walk could not finish | 10 |
| … whose `varU` is **wide** | **0** |

**Two results, and the second is the stronger one.**

1. **Zero wide tokens at 2,414 real sites.** **H3 MISSES** (registered p = 0.40
   that it is reachable). The hazard is real in the language and **not witnessed
   in this workload**. Registered **H5** — ≥ 100 sites — **HITS** at 2,404, so
   this is a live production and not a curiosity.
2. **Every top-level `0x42` in the workload is immediately preceded by a
   `0x43`** — 2,404 of 2,404. The port's *pairing* model is empirically exact
   even though its *escape* model is fictitious. That is why the fiction has
   never cost anything.

**The walk's own limit, stated rather than left to be found:** it finishes 23.5 %
of candidate bodies and stops on **exactly one** cause — class `06`, the
`UNRESOLVED` row of §4.5, 1,848,038 stops and no other reason. Closing `0x33`
closes the walker too.

### 6.3 Is it CONSTRUCTIBLE? — yes, and that is a different claim

Registered **H4** at p = 0.75. **HIT, with the claim stated narrowly.**

* c2 **accepts** it: class `02` calls `varU` unconditionally (`0x10b3d65f`) and
  `varU`'s wide form is `0x10c1f91b`'s own second branch. A `43 42` over a token
  whose id is `≥ 0x8000` is six bytes and c2 reads all six.
* the **container accepts** it: `IlBundle` stores `.ex` bytes verbatim and
  `readers::read_token_var` — the port's *own* reader — decodes both widths.
  Nothing between the two bounds a token below `0x8000`.
* so the port's fixed `+4` walks **two bytes into the payload** on such a
  stream, and continues.

**This is NOT a claim that `c1xx` emits one.** Whether the front end can produce
a `0x42` operand with a symbol id ≥ 32,768 is a question about symbol-table size
that this lane did not measure, and it is the only remaining gap between
"constructible" and "will happen".

**No fix is made here and none should be read into this page.** An emit change
is outside a characterization lane's fence and outside this lane's file fence.
What is on the table for a future lane is a two-line width fix whose *cost* is
that it turns a coincidence into a rule — priced two-sided, like every fence.
Board **#3590**.

---

## 7. THREE ENVIRONMENT GLOBALS THE PORT HAS BAKED — the goal-(1) deliverable

Folded into **no** verdict, because they are not per-input. Each is a
**decision point** in the sense `CLAUDE.md` § "The goal" asks general layers to
expose: a named, settable parameter whose default reproduces c2.

| global | where c2 reads it `[R]` | what the port did `[src]` |
|---|---|---|
| **`[DAT_10c472e8 + 0xcac]`** | `0x10b3d5ab` — gates the TYPE's trailing `skip`; **and** `0x10b3d919` — gates class `1C`'s (`0x99`'s) trailing `i32c` | reads **both** unconditionally (`read_type`'s id loop; `control_flow.rs:1164`'s `vint`) — **baked NON-ZERO, in two places** |
| **`DAT_10c2edc4`** | `0x10b3d8ce` — class `19` (`0xBD`) reads an `i32c` when set and an `i16c` when clear | reads `Scan::vint` (== `i32c`) unconditionally — **baked NON-ZERO** |
| **`DAT_10c67fc0`** | `0x10b3d64d` — when **zero**, opcode `0x42` alone takes no operand | the port has no `0x42` reader, so it baked nothing; but the `0x43` escape that consumes `0x42`'s bytes assumes the **non-zero** branch |

**The first row is the interesting one, and it is self-consistent rather than
merely unnoticed.** One global gates two different reads in two different
functions, and the port hard-coded the same answer in both — so the port is a
model of c2 *at one setting of one flag*, and it is internally coherent at that
setting. That is exactly the shape decision-surface work wants: **one named
parameter, two call sites, one default**. `WB_READER_FINDINGS.md` §5.3's
obj-checks were taken at that same setting (the trailing skip *was* read), which
is why nothing has ever disagreed.

Registered **R5** — at least one arm carries a conditional the prior one-line
grammars do not surface — **HITS three times**.

---

## 8. THE PREREG, GRADED — 17 registered, 13 HIT, 3 MISS, 1 HALF

### 8.1 The replication

| # | p | prediction | result |
|---|--:|---|---|
| **R1** | 0.90 | the class table, the bound `0x1c` and the jump table all derive from one hard-coded head | **HIT** |
| **R2** | 0.80 | the 29 targets are **27** distinct | **HIT**, exactly 27, with both shared pairs named |
| **R3** | 0.60 | my decode agrees with `WB_READER_FINDINGS.md` §3 on all 29 | **HIT in substance on 29 of 29**, with one imprecision corrected (class `06`'s predicate reads the LOWERED word) |
| **R4** | 0.85 | the class byte reproduces `w-ilarms`'s column on all 95 handled opcodes | **HIT** |
| **R5** | 0.55 | ≥ 1 arm carries a conditional the one-line grammars do not surface | **HIT — three** (§7) |

### 8.2 Limb 2

| # | p | prediction | result |
|---|--:|---|---|
| **B1** | 0.55 | **[22, 34]** of the 65 change verdict | **HIT — 30** |
| **B1a** | 0.80 | the bimodal upper arm is taken (`Scan::ty` diverges) | **HIT** — and it is 26 of the 33 |
| **B2** | 0.90 | `NARROW` outnumbers `WIDE` | **HIT — 30 vs 2** |
| **B3** | 0.70 | ≥ 1 row other than `0x43` is `WIDE(fields)` | **HIT — `0x2c`** |
| **B4** | 0.55 | ≥ 1 row stays `UNRESOLVED`, with its residual read named | **HIT — `0x33`**, and the *candidate* named in the prereg (`0x4f`) resolved instead |
| **B5** | 0.80 | ≤ 6 distinct root causes | **MISS — 8.** §4.2 records why: the bracket counted port primitives, and a cause is a *pair* |
| **B6** | 0.50 | `0x2c` and `0x54` both resolve, ≥ 1 `WIDE` | **HIT** — `0x2c` `WIDE`, `0x54` `NARROW` |

### 8.3 The two hazards

| # | p | prediction | result |
|---|--:|---|---|
| **S1** | 0.90 | `0x28` confirmed `NARROW(fields)` | **HIT** |
| **S2** | 0.50 | the class-`02` sibling sentence needs an amendment | **HALF** — the sentence stands; its class-uniformity premise needs a caveat (§5) |
| **H1** | 0.95 | `0x43` is class `00`, no escape | **HIT** |
| **H2** | 0.60 | the fixed `+4` is wrong a second way, in the narrow direction | **HIT** — `DAT_10c67fc0 == 0` makes `43 42` two bytes |
| **H3** | 0.40 | the wide-token hazard is **reachable in the workload** | **MISS — 0 of 2,414 real sites.** Reported as the headline of §6.2, not buried |
| **H4** | 0.75 | it is **constructible** even if H3 misses | **HIT**, stated narrowly (§6.3) |
| **H5** | 0.65 | ≥ 100 real `43 42` sites | **HIT — 2,404** |

### 8.4 The record

| # | p | prediction | result |
|---|--:|---|---|
| **X1** | 0.70 | ≥ 2 live surfaces say the arms are unread while ≥ 1 says they are read | **HIT — five surfaces had the read; four say it is outstanding** (§1) |
| **X2** | 0.60 | the same shape holds for a **second** target this lane touches | **MISS.** Class `0C`'s sub-record reader is also already read (`ref/P_SUB4F.md`), but that is the *same* instance one level down, not a second one. Recorded as not established rather than stretched to fit |

**13 HIT · 3 MISS · 1 HALF over 17.** The three misses are **B5** (a bracket
built on the wrong inventory), **H3** (an optimistic reachability guess), and
**X2** (a pattern that did not generalise). None is smoothed.

---

## 9. Disclosure, checked

**This lane adopts nothing into `crates/` and owes 0 `DISCLOSURE.md` rows.**
`git diff --numstat f202268f6..HEAD -- crates/ scripts/ fixtures/` returns
**0 files** — the fence is checked, not asserted; the exact output is quoted in
[`../rungs/2026-08-26-w-opclass.md`](../rungs/2026-08-26-w-opclass.md) §1.

**What a future adopter would owe**, as a number rather than a gesture. A lane
that fixed any of §4.2's eight causes would adopt, at minimum:

* **1** for the class dispatch head `0x10b3d610` plus **1** for the class table
  `0x10b25e48` and **1** for the jump table `0x10b3d954` — **3**;
* **1 per class arm implemented** — **27** if all real arms are taken;
* **1** for the refusal `0x10b3d941` and its `reader.c` line 491, and **1** for
  class `0B`'s `0x10b3d7c8` / line 299 — **2**;
* **1 per scalar primitive whose width function enters a reader** — **9**
  (§3.1);
* **5** for the TYPE reader's five stream-relevant sites (`0x10b3d550`,
  `0x10b3d59f`, `0x10b3d5b4`, `0x10b3d5b9`, `0x10c1fe40`);
* **3** for the environment globals of §7, if any becomes a named parameter.

So **≥ 49 rows for the whole grammar**, or **≥ 8** for the narrowest useful
slice (the TYPE reader's four narrownesses plus `0x2c`, `0x54`, `0x66`, `0x43`)
— and both are floors.

**The narrowest slice is also the one with obj evidence already behind it.**
`WB_READER_FINDINGS.md` §5.3 obj-confirmed the one-byte TYPE word and the
`varU` bit-15 continuation against real `c2.dll`; `W-EXT-1` and `W-EXT-3` are
**pre-drafted** there and still unadopted.
