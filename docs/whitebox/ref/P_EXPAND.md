# `P_EXPAND` — the final-expansion switch, and the pseudo-op word-count table

> **PROVENANCE — DISASSEMBLY-DERIVED.** See [`../DISCLOSURE.md`](../DISCLOSURE.md).
> Nothing here may enter `crates/` without a `DISCLOSURE.md` row naming the
> address it came from.

**Read R6** ([`../READ_PLAN_2026-08-21.md`](../READ_PLAN_2026-08-21.md) §3),
lane `w-read-r6`, board **#3429**–**#3432**. Prereg:
[`../WB_EXPAND_PREREG.md`](../WB_EXPAND_PREREG.md). Grade:
[`../WB_EXPAND_FINDINGS.md`](../WB_EXPAND_FINDINGS.md). Tooling:
[`../scripts/dump_expansion.py`](../scripts/dump_expansion.py),
[`../scripts/probe_prolog_words.py`](../scripts/probe_prolog_words.py).

**Image.** `compilers/X360/16.00.11886.00/c2.dll`, sha256
`c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258` — verified
before any address here was read. Every address below is reproducible from the
image alone: the tooling disassembles the PE directly and does **not** depend on
the Ghidra flat export (which is 19 days older than HEAD).

**Provenance legend** ([`README.md`](README.md) §2): `[R]` read, not confirmed
against any obj; `[O]` obj-confirmed; `[I]` inferred. **`[R]` means "the
instructions were read correctly", never "this is what c2 does."**

---

## 0. The answer, in one screen

The read plan asked for *"the pseudo-op expansion table — which opcodes expand
to how many words."* The shape of the answer is not the shape the row expected,
and that is the finding:

> **The final-expansion switch `FUN_10c0d57e` almost never changes the word
> count itself.** Of the 29 arm bodies this lane recovered, **24 emit at most
> one instruction** and **5 are unbounded** (`retaddr`, `nopalign` ×3, `0x2e5`),
> and the three that produce a whole prologue or epilogue
> emit **zero words directly — they delegate** to a driver outside the switch.
> The count-changing work is not spread across the switch; it is concentrated in
> **four** delegate helpers, and the prologue's own count is **recorded by c2 in
> the object it emits**, so it never has to be predicted at all.

| question | answer | tier |
|---|---|---|
| how many opcodes get a non-default arm? | **69** discriminated opcodes over **29** arm bodies; **767** more reach the dispatch tail | `[R]` |
| how many words does an arm emit? | **0 or 1** for **24 of 29** bodies; the other **5** are unbounded (§3) | `[R]` |
| which arms expand 1→many? | **none directly.** `0x2f0`/`0x2f4`/`0x2f6` delegate (§4) | `[R]` |
| how many words is a prologue? | **1–7**, seven distinct values, **100 % ≤ 8** over 12,610 framed functions | **`[O]`** |
| is the count a constant? | **no** | **`[O]`** |
| can a port predict it without simulating the pass? | **yes** — `.pdata`'s `prolog_words` (§4.4) | **`[O]`** |

---

## 1. The dispatch — `FUN_10c0d57e`, 3,899 B  `[R]`

`0x10c0d57e`, 3 callers, 41 callees, 1,255 x86 instructions. Size **verified
exactly** against the export's function table.

The opcode is `param_2[1]`, loaded into `eax` at **`0x10c0d58f`**
(`mov eax,DWORD PTR [esi+0x4]`). Dispatch is a **binary search tree**, not a
jump table — `cmp eax,0x270 / ja / je`, `cmp eax,0x7b / ja / je`, … — which
reproduces `WB_SELECT_FINDINGS.md:668`'s PARTIAL **from the bytes**.

**Recovering the arm set therefore requires interval propagation, not pattern
matching.** There is no table to read, and two of the most important arms
contain no equality test on the opcode at all:

```
10c0d5c7:  cmp eax,0xb        10c0daab:  lea ecx,[eax-0x26e]
10c0d5ca:  jb  <default>      10c0dab1:  cmp ecx,0x1
10c0d5d0:  cmp eax,0xd        10c0dab4:  ja  <default>
10c0d5d3:  jbe 0x10c0d6fd     10c0dabc:  call 0x10c0a2e2   <- the rlandi expander
```

`dump_expansion.py --arms` walks the CFG carrying the surviving opcode interval
along each path; when a path reaches an arm body the interval **is** that arm's
opcode set.

### 1.1 Denominators, measured — not assumed

```
opcode bound the tree actually discriminates      0x2ff
distinct opcodes with a non-default arm              69
distinct arm bodies                                  29
opcodes reaching the dispatch TAIL 0x10c0e30b       767
shared bodies reached with an un-narrowed interval   10
```

**`ADDR.tsv:1124`'s "41 arms" for this function is not a measurement of it** —
41 is its *callee* count. No document in this repo had ever counted its arms.

> **⛔ AMENDED — lane `w-tailread`, 2026-08-23, board #3460/#3461.** The block
> above stands as written except for two rows, and the originals are left
> intact so this can be graded.
>
> **1. `opcodes reaching the dispatch TAIL = 767` is not a measurement.**
> `0x2ff` **is** 767, and it is exactly this walk's domain (`1..OPMAX`). Re-run
> `opcode_tree` with the bound raised and the number follows it:
> `OPMAX 0x2ff → 767`, `0x400 → 1024`, `0x600 → 1536`. It says the tail is
> reachable carrying an un-narrowed interval — a property of the abstract
> interpretation, not of c2. **Six of the ten shared fall-through bodies below
> report `767` for the same reason.** §7's warning that *"reaches the tail is
> not is unchanged"* is right and is not what this amends.
>
> **2. `opcode bound … 0x2ff` is short, and so is the arm map: `0x302` exists.**
> `opcode_bound()` takes the largest **literal** in a `cmp`/`sub`/`add`/`lea`,
> giving `0x2fe + 1`. But the dispatch continues past that literal by a
> subtract chain — `sub ecx,0x2fe / je (0x2fe) / dec / je (0x2ff) / sub ecx,3 /
> je 0x10c0e479` — so **opcode `0x302` has a discriminated arm at
> `0x10c0e479`** that no literal names and this map therefore misses. Corrected
> count: **70** discriminated opcodes, still over 29 arm bodies. Raising the
> bound to `0x600` gains nothing further, so 70 is stable.
>
> See [`P_OPATTR.md`](P_OPATTR.md) §3 and §3.1.

### 1.2 The dispatch tail is a **table**, and it is new  `[R]`

Everything the tree does not discriminate falls to **`0x10c0e30b`**, which is
not a default at all:

```
10c0e30b:  mov cl,BYTE PTR [eax+0x10c3afd8]
10c0e311:  and cl,0x7
10c0e314:  cmp cl,0x2
10c0e317:  je  0x10c0e40f
10c0e31d:  cmp eax,0x281            <- `lea`, the same body
```

**`0x10c3afd8` is a per-opcode attribute BYTE table whose low 3 bits are an
opcode class.** Class 2 routes to `0x10c0e40f`, shared with `lea` (`0x281`) and
`retaddr` (`0x28f`). No document in this repo records this table. It is the
natural next read for anyone extending this page, and it is why "767 opcodes
reach the tail" is **not** the same statement as "767 opcodes are unchanged".

> **⛔ AMENDED — lane `w-tailread`, 2026-08-23, board #3460/#3462.** Read in
> full at [`P_OPATTR.md`](P_OPATTR.md). Four corrections; the original is left
> as written.
>
> **1. *"No document in this repo records this table"* is FALSE**, and the same
> sentence appears in `WB_EXPAND_FINDINGS.md:79` (*"an unrecorded table"*) and
> in board **#3432** (*"recorded in no document here"*). The table is board
> **#2040**, **#2044**, **#2106** and **#2206**, lane `wb-select`, 2026-08-09,
> and `rungs/2026-08-09-wb-select2.md:67` states it in one line: *"The same
> byte is exposed as an array at `0x10c3afd8`, indexed by machine opcode."*
> #2044 already decodes bits `0x08`/`0x10`/`0x20`/`0x40`. **What was genuinely
> unread is the low 3 bits.** Verified: `attr[op]` equals the `0x10b1b260`
> mnemonic table's `+8` flags byte for **664 of 664** entries.
>
> **2. It is not "the dispatch tail's table".** The tail is **one of 38**
> consumers image-wide, and one of ten that read the class field.
>
> **3. The class field is an operand-shape partition, not an expansion one:**
> 1 = move/SPR-transfer (35), 2 = load (55), 3 = store (52), 4 = sign-extend
> (6), 0 = other (516). Classes 5–7 and bit `0x80` are unused.
>
> **4. The tail is a THREE-way classifier and `retaddr` is not part of it.**
> `class 2 → 0x10c0e40f`; `opcode 0x281 → 0x10c0e40f`; `class 3 → 0x10c0e331`;
> everything else → the exit join. **`lea` is class 0** — which is *why* it is
> named explicitly — and **`retaddr` (`0x28f`) is class 0 too**, reaching
> `0x10c0e40f` from neither route; its arm is `0x10c0e006`, as §3 says. The
> predicate the tail computes is *load-or-lea*.
>
> **And the tail expands nothing.** Its five callees are a form predicate
> (`0x10c123b9`, which indexes `P_ENCODE.md` §3's encode-form table), a set
> allocator, a set-insert, an **operand**-node allocator and an **operand**-list
> append. None is one of §2's 16 instruction constructors. Word delta **zero**
> for every opcode reaching it.

---

## 2. The emit alphabet — one call is one word  `[R]`

An instruction is created by exactly one family of functions: those that call
the list-insert wrapper **`FUN_10bd5732`** (`0x10bd5732`, 43 B), which calls
`FUN_10bd3824` (`0x10bd3824`, 17 B, **147 callers**, a doubly-linked insert-
after) and stamps `+0x14` with `DAT_10c2e2ec`.

> **⛔ AMENDED BESIDE — lane `w-2e4`, 2026-08-24, board #3503. TWO THINGS IN
> THE PARAGRAPH ABOVE ARE WRONG AS READ, AND THE SECOND IS A DECISION POINT
> THIS PAGE HIDES.** `[R]`, `0x10bd5732` read in full (7 instructions of body):
>
> ```
> 10bd5732  esi = edx (the new tuple) ; edi = ecx (the anchor)
> 10bd573a  call 0x10bd3da7                       ; operand bookkeeping
> 10bd573f  cmp DWORD PTR [esp+0xc],0x0 / je      ; <== the splice is a PARAMETER
> 10bd574a  call DWORD PTR [esp+0xc]              ; <== and it may be NULL
> 10bd574e  WORD PTR [esi+0x14] = ds:0x10c2e2ec   ; the line stamp, as stated
> ```
>
> 1. **`0x10bd5732` does not call `0x10bd3824`.** It calls an **indirect
>    callback supplied by the constructor's caller**, and that callback may be
>    **null**, in which case the tuple is built and *not linked at all*.
>    `0x10bd3824` is the common argument, not the callee. `ehexcept.c`
>    `0x10be40ca` and `0x10be41ac` pass **`0x10bd3815`** instead, and
>    `cgintrin.c` `0x10bf80cc` passes it too (`WB_R8IDIOM_FINDINGS.md` §4.1
>    noticed the difference without naming it).
> 2. **`0x10bd3824` is INSERT-BEFORE, not insert-after.** `+0x00` is *next*
>    and `+0x10` is *prev*.
>    [`P_BLOCKORDER.md`](P_BLOCKORDER.md) §"list primitives" already reads it
>    that way (`0x10bd3815` INSERT AFTER · `0x10bd3824` INSERT BEFORE, with
>    bodies); this page and [`P_OPATTR.md`](P_OPATTR.md) §5 say "insert-after".
>    **A third, independent read agrees with `P_BLOCKORDER.md`**, on three
>    pieces of evidence: `fg.c`'s block builder `0x10b372ea` walks the tuple
>    list head→tail via `+0x00`; `0x10bd417d` inserts a *fall-through* label
>    after a tuple using `0x10bd3815`'s body verbatim; and
>    [`WB_MERGER4_FINDINGS.md`](../WB_MERGER4_FINDINGS.md) §2 reads M4 walking
>    predecessors **backwards** through `tuple+0x10`.
>
> **Nothing in this page's word counts moves** — a count of constructor calls
> does not depend on which side the callback links. What moves is any future
> port that reproduces tuple *order*: the direction is a caller-chosen
> parameter, and one EH producer uses both directions on the same anchor to
> bracket a call.
>
> Full record: [`WB_2E4_FINDINGS.md`](../WB_2E4_FINDINGS.md) §2.1–§2.2.

Inverting the call graph on `0x10bd5732` gives **16 constructors**:

```
10bd59aa  10bd722e  10bd726d  10bd72b0  10bd72fb  10bd7354  10bd73ac  10bd7413
10bd748b  10bd74f8  10bd75ff  10bd7652  10bd76e6  10bd7780  10bd77db  10bd7814
```

Every one allocates via `FUN_10bd3750(kind)`, writes `node[1] = opcode`, and
**ORs bit 0 into `node+9`**. Read `FUN_10bd72b0` (75 B, 230 callers — the
commonest) or `FUN_10bd76e6` (154 B) to see it.

> **This independently reproduces R2's invariant.** `P_ENCODE.md` states *"real
> instruction iff `tuple+0x9` bit 0"* from the **encoder** end; this lane
> arrived at the same bit from the **constructor** end without looking. Two
> derivations, one bit.

**Consequence, and it is what makes the deliverable tractable:** "how many words
does this arm emit" is a *countable* property — the number of constructor calls
on the arm's paths — not an interpretive one. `dump_expansion.py --words`
computes it as a (min, max) over CFG paths, reporting **`unbounded`** when a
back edge makes it data-dependent.

---

## 3. The word-count table  `[R]`

`dump_expansion.py --words`, verbatim. `DELEGATES` names a helper that is itself
a multi-word emitter, so the arm's own count is not the whole story.

| arm VA | words | opcodes | note |
|---|---|---|---|
| `0x10c0d5ec` | 0..0 | `cmpi`, `cmpli` | rewrite in situ |
| `0x10c0d5f8` | **0..1** | `bc` | **machine-band arm that can ADD a word** |
| `0x10c0d6a1` | 0..1 | `bc` | second `bc` body |
| `0x10c0d718` / `0x10c0d72b` | 0..0 | `0x2aa`, `addi`, `addic`, `addic.`, `0x2ab` | the `0x0b..0x0d` range arm |
| `0x10c0d957` / `0x10c0d967` | 0..1 | `divd`,`divdu` / `divw`,`divwu` | |
| `0x10c0d9f3` | **1..1** | all four divides | always one extra word |
| `0x10c0dac6` / `0x10c0db6b` | 0..0 | `xori` | |
| `0x10c0dfdc` / `0x10c0e103` | 0..0 | `fmr`, `mr` | |
| `0x10c0e006` | 0..**unbounded** | `retaddr` | delegates to all three prologue helpers |
| `0x10c0e065/06c/06e` | 0..**unbounded** | `nopalign` | alignment padding — a loop |
| `0x10c0e146` | 0..**unbounded** | `0x2e5` | |
| `0x10c0e185`/`0x10c0e1bf`/`0x10c0e1cb`/`0x10c0e1e6` | 0..0 | `0x2e5`,`0x2e1`,`0x2ba` | |
| `0x10c0e194` | 1..1 | `0x2e4` | **the one word is `emit 0x7d084378` = `mr r8,r8`** — see the beside-amendment under this table |
| **`0x10c0e283`** | **0..0** | **`0x2f6`** | **DELEGATES `0x10bffb72`** (restore) |
| **`0x10c0e28f`** | **0..0** | **`0x2f4`** | **DELEGATES `0x10c216f5`** |
| **`0x10c0e29b`** | **0..0** | **`0x2f0`** | **DELEGATES `0x10c21719`** |
| `0x10c0e487`/`0x10c0e494` | 0..0 | `0x2ff`, `0x2fe` | |
| `0x10c0e4a4` | 0..0 | `fmr`,`mr`,`0x2e5`,`0x2f7` | the no-op join |
| `0x10c0e4ab` | 0..0 | 52 opcodes | the exit join |

**Three readings of this table that matter:**

1. **The switch is overwhelmingly count-preserving.** 24 of 29 bodies emit 0 or
   1 words. The read plan's mental model — a switch that fans pseudo-ops out
   into many words — is true of the *system* and false of *this function*.
2. **`bc` at `0x21` can add a word, and it is a MACHINE-band opcode.** This is
   the long-branch expansion (`CFG_SHAPE.md:477` — invert the condition and
   branch over an unconditional `b`). It refutes the tidy claim that all
   count-increasing arms live above the machine space; see the prereg's P1.3,
   registered as a prediction and scored a MISS, with `bc` named in advance as
   the likely falsifier (P1.4).
3. **`nopalign` (`0x27b`) is genuinely unbounded** — the alignment-padding arm
   contains a loop, so no constant describes it. Any instrument asserting a
   word count must special-case it.

> **⛔ AMENDED BESIDE — lane `w-r8idiom`, 2026-08-24, board #3481. THE ROW
> `0x10c0e194 | 1..1 | 0x2e4` IS THE `mr r8,r8` IDIOM, AND THE WORD WAS ONE
> LINE AWAY THE WHOLE TIME.** `[R]`. The arm is eleven instructions: it builds
> a literal operand from the **baked constant `0x7d084378`** (`push` at
> `0x10c0e1a1`, builder `0x10bd575d`) and an instruction of opcode **`0x290`**
> — which this image's own mnemonic table calls **`emit`**, the raw-word
> emitter — via `0x10bd726d` at `0x10c0e1b5`. `0x7d084378` is `or r8,r8,r8`,
> i.e. **`mr r8,r8`**, the 3,792-instance population `P_OPATTR.md` §6 recorded
> as unexplained and `dump_tailclass.py:496` uses as a mint CONTROL.
>
> Two consequences for *this* page rather than that one:
>
> * **This row's `1..1` was right and is now also meaningful.** The arm is
>   count-preserving in the sense of reading 3 above — one pseudo-op in, one
>   inert word out — and it is the clearest case in the table of the switch
>   giving a marker an *address* rather than an instruction.
> * **`0x2e4` reaches this arm and nothing else does**, checked against the
>   recovered tree (`0x2e1` → `0x10c0e1bf`, `0x2e5` → `0x10c0e185`). It is
>   minted at, among others, `0x10be3fdf`/`0x10be40d8`/`0x10be41ba`/`0x10be42a4`
>   in `FUN_10be3e4c` (`ehexcept.c`), inside a list walk. **What `0x2e4` IS is
>   NOT read** — see `WB_R8IDIOM_FINDINGS.md` §6, which refuses to name it.
>
> Full record: [`WB_R8IDIOM_FINDINGS.md`](../WB_R8IDIOM_FINDINGS.md).

> **⛔ AMENDED BESIDE — lane `w-2e4`, 2026-08-24, board #3501/#3502. THE
> "NOT READ" ABOVE IS NOW READ, EXCEPT FOR THE NAME.** `[R]`. `0x2e4` is a
> **kind-`0x12` branch tuple with exactly one operand, and that operand is a
> LABEL**: the constructor `0x10bd76e6` hard-codes `mov cl,0x12`, tests
> `target.kind == 0x1b`, registers a **kind-`0x1d` predecessor record** in
> `label[+0x28]` (`0x10bd3f62`), bumps the label's reference count, and stores
> a `0x2a7` label-address operand in `tuple[+0x28]`. **Minting one adds a real
> CFG edge.**
>
> It is minted at **7** sites in 4 TUs — not the 8 functions in 6 TUs
> `WB_R8IDIOM_FINDINGS.md` §4 tabulates, four of whose rows are consumers —
> out of **171** callers of that constructor, a family in which `0x2dd` (60)
> and `0x2de` (50) are the common members. The `fg.c` producer `0x10b39937` is
> **PGO-gated** (`DAT_10c3de20 == 2`) and cannot fire on this workload.
>
> The contract, written out verbatim in `inline.c` `0x10b6e99b` and
> `p2symtab.c` `0x10b9f04e`:
> `PLAIN_CONDITIONAL(t) := t.kind == 0x12 && t[+0x34] == 0 && t.opcode ∉ {0x2e4, 0x21, 0x22}`,
> and `fg.c`'s block builder **does not split a block on a `0x2e4`**
> (`0x10b374c3`).
>
> **The NAME is still refused, and now as a measurement**: of the 34 char*
> arrays in `.text`/`.data` exactly one passes a four-row control, and it stops
> 75 rows short. Row `0x2e4` of the mnemonic table reads `twle` and that is a
> **coincidence `0x39c` past the table's end**.
>
> Full record: [`WB_2E4_FINDINGS.md`](../WB_2E4_FINDINGS.md).

> **⛔ AMENDED — lane `w-tailread`, 2026-08-23, board #3463. THIS TABLE IS
> ONE-SIDED: IT COUNTS ADDITIONS AND CANNOT SEE DELETIONS.** The counts above
> are correct as counts of constructor calls; they are not word deltas.
>
> §2 names the **mint** primitive (`0x10bd3824`, doubly-linked insert-after,
> under the wrapper `0x10bd5732`). It has an exact inverse that no document
> here names: **`0x10bd5516`**, the **unlink**, whose body is `0x10bd3824`
> reversed. And it is the commoner operation —
> **`0x10bd5516`: 401 direct callers · `0x10bd3824`: 207.**
>
> Inside `FUN_10c0d57e` there is exactly one call to it, at **`0x10c0e4a6`**,
> in the body at **`0x10c0e4a4`** — the row above labelled *"the no-op join"*,
> `0..0`, `fmr, mr, 0x2e5, 0x2f7`. **It is the DELETE join; its word delta is
> −1.** Three further arms fall into it: `0x10c0e494` (`0x2fe`, after setting
> the opcode to `0x297`), `0x10c0e479` (`0x302`, the arm §1.1's amendment
> recovers) and `0x10c0dfdc` (via `call 0x10bd2d83 / jmp 0x10c0e4a4`) — which
> this table also scores `0..0`.
>
> **§2's closure argument is sound, and this supplies the premise it omits.**
> Inverting the call graph on `0x10bd5732` is closed only if that address is
> never taken. Measured: **`0x10bd5732` address-taken 0 times** — so the
> inversion holds. One level down it would not: **`0x10bd3824` is address-taken
> at 506 sites**, passed as a callback to the emitter helpers, two of them
> inside this switch (`0x10c0db6b`, `0x10c0e20e`). Spot-checked: that path's
> node builder `0x10bd575d` does not set `+9` bit 0, so it is not minting
> instructions — one function checked, not the population.
>
> See [`P_OPATTR.md`](P_OPATTR.md) §5.

---

## 4. The prologue family — five arms, not two  `[R]`/`[O]`

### 4.1 The arms, read from the bytes at `0x10c0e266`

```
10c0e255:  mov ecx,0x2f8 / cmp eax,ecx / ja 0x10c0e2ed / je 0x10c0e2a7
10c0e266:  sub ecx,0x2f0 / je 0x10c0e29b   ->  call 0x10c21719
10c0e26e:  sub ecx,0x4   / je 0x10c0e28f   ->  call 0x10c216f5
10c0e273:  dec / dec     / je 0x10c0e283   ->  call 0x10bffb72
10c0e277:  dec           / je 0x10c0e4a4   ->  the no-op join
```

So the family is **`0x2f0`, `0x2f4`, `0x2f6`, `0x2f7`, `0x2f8`** — five arms.
`FUN_10bfebf7` additionally scans until it meets **`0x2f1` or `0x2f6`**, so
those two are the region terminators. **`0x2f5` is not in the switch at all**,
although `P_ILRECORD.md:254` records IL arm 48 minting it beside `0x2f4`.

### 4.2 The two thunks, and the single argument that separates them

`0x10c216f5` (19 B) and `0x10c21719` (25 B) are the **only two callers** of the
prologue driver `FUN_10bff95c` (327 B) — a coherence check that closes that
population. They differ in one argument:

```
10c216f5:  ... push 1 / push 0            / push [eax+0x33] / call 0x10bff95c
10c21719:  ... push 0 / push [esi+0x33]   / push [eax+0x33] / call 0x10bff95c
```

In `FUN_10bff95c` that argument is `param_4`, and `bVar6 = param_4 != 0` gates a
**second** `FUN_10bfec72` call — i.e. a prologue laid down at a **second entry
point**, preceded by a minted label and a `b` (opcode `0x1f`) emitted through
`FUN_10bd76e6`.

> **THIS ARBITRATES A LIVE CONTRADICTION, AND IT SETTLES IT AGAINST BOTH SIDES.**
> `WB_SELECT_FINDINGS.md:177` says `0x2f0` = prologue / `0x2f4` = epilogue;
> `WB_SELECT_FINDINGS_R2.md:217` says the reverse; `WB_SELECT_RECONCILED.md`
> settled which *function* the arms belong to and never touched *which is
> which*. **Neither is right as stated.** Both `0x2f0` and `0x2f4` reach the
> **prologue** driver, differing only in whether a second entry point is
> supplied; the **restore** side is `0x2f6`, which reaches `FUN_10bffb72` →
> `FUN_10bffaa3`. Do not key an instrument on the prologue/epilogue reading.

### 4.3 What actually emits the words

```
FUN_10bff95c  (prologue driver, 327 B)
  ├─ FUN_10bfebf7  0x10bfebf7   saved-GPR mask; scans until opcode 0x2f1 or 0x2f6;
  │                             counts register numbers n in [0x0f..0x20]
  │                             (r14..r31 under R2's n = r+1; 0x0c with DAT_10c2e980)
  ├─ FUN_10bff507  0x10bff507   the flag word: bit0 LR spill, bit2 frame
  └─ FUN_10bfec72  0x10bfec72   THE WORD EMITTER, 211 B:
        flags&1  ->  FUN_10c07910(r12, -8, …)      LR spill
        loop     ->  FUN_10c07910(reg, off, …)     ONE CALL PER SET MASK BIT
        flags&4  ->  FUN_10c07910(0x53, …)         frame-establish pseudo-register
                     FUN_10c0b6fa(ip, fn[0x1a] + 8 + 8*nsaved)   frame allocator
```

`FUN_10c07910` (446 B) splits on the register number: `{0x4c,0x4d} ∪
[0x51..0x54] ∪ {0x6a}` take a special path emitting `0xe6` (`mfspr`) plus a
store; everything else emits **one** store whose opcode is looked up **by
register class** from the table at `DAT_10c6fdd0`, indexed by
`FUN_10bd7c10(class)` where the class word is `*(u16*)(&DAT_10c2f098 + reg*0x60)`.

So the prologue's word count is, structurally:

```
prolog_words  =  2·[LR spill]  +  popcount(saved mask)  +  frame_words(size)
size          =  fn[0x1a] + 8 + 8·popcount(saved mask)
```

with `frame_words` from `FUN_10c0b6fa` (`WB_FRAME_FINDINGS.md` §3.1): 0 when
size ≤ 0; one `stwu` (`0x17e`) normally; `stwux` (`0x17f`) plus
`_RtlCheckStack12` plus **one probe word per page** when `F ≥ 5 × 0x1000`.

**Every input is upstream of the pass** — the frame size and the saved mask are
decided before the expansion runs. Prereg P3.5 answered: **yes**, an instrument
can predict the count without simulating the expansion.

### 4.4 …but it does not have to, because c2 writes the number down  `[O]`

Every `.pdata` record's unwind word carries `prolog_words` in its low 8 bits
(`WB_EH_FINDINGS.md` §5 row W-EH-1; emitter side `crates/c2-core/src/coff/pdata.rs:71`).
**The expansion's output size is a directly observable field of the object.**

`probe_prolog_words.py` over the already-captured corpus, **12,610 framed
functions from 6,000 objs**:

```
1 word    28   0.22 %        5 words  1874  14.86 %
2 words  196   1.55 %        6 words    58   0.46 %
3 words 7468  59.22 %        7 words    48   0.38 %
4 words 2938  23.30 %        ≤ 8 words: 12610/12610 = 100.00 %
```

Shape sub-population (282 records — only single-`.text` objs whose length
matches the record, so the words can be attributed without resolving the
`BeginAddress` relocation):

```
176  mfspr | stw | stwu                      <- mflr r12; stw r12,-8(r1); stwu
 32  std ×7                                  <- FRAMELESS, saves only, no stwu
 28  mfspr | stw | std | std | stwu
 22  mfspr | bl                              <- the __savegprlr_N helper path
 16  mfspr | stw | std | stwu
  8  mfspr | bl | stwu
```

**The saves are inline stores, and the count is linear in the save count.** Only
**30 of 282** prologues contain a `bl` at all. The helper path exists (board
#1783, #1805) but does **not** dominate this corpus, and the flat 3-word
common case is "few registers saved", not "a helper absorbs them".

---

## 5. `FUN_10c182b4` is the peephole, and its arm 6 is `fmr`  `[R]`

426 B, one caller `0x10b7dd2c`, gated on `DAT_10c2e2fc`, list walked twice.
**It is not an expansion pass** — `c2_functions.tsv:4499` (W-TABLES) is right
and `WB_SELECT_FINDINGS_R2.md` §4 is the superseded reading. Dispatch is the
opposite shape from §1: an opcode-indexed **byte table** at `0x10c184a8`
(`0x293` entries, opcodes `0x001..0x293`) into a jump table at `0x10c18460`.

`dump_expansion.py --peephole` reproduces prior art exactly — **659 opcodes over
18 arms, 445 on the do-nothing arm 17** — and closes the one gap in the only
published table (`WB_SELECT_FINDINGS_R2.md:436-459`, which omits arm 6):

| arm | target | n | opcodes |
|---|---|---|---|
| 0 | `0x10c1841a` | 38 | three-operand ALU |
| 1 | `0x10c183fb` | 1 | `cmpi` |
| 2 | `0x10c18407` | 11 | `cmpli`,`neg`,`ori`,`sradi`,`srawi`,`xori`,… |
| 3/4/5 | `0x10c183b2`/`0x10c183ca`/`0x10c183d8` | 2/2/2 | `extsb`/`extsh`/`extsw` (± record) |
| **6** | **`0x10c1838b`** | **1** | **`fmr` — the row no prior document has** |
| 7/8/9 | `0x10c183dc`/`0x10c183ee`/`0x10c183f6` | 4/5/5 | `stb`/`sth`/`stw` families |
| 10/11/12 | `0x10c18432`/`0x10c18426`/`0x10c1843e` | 108/26/4 | VMX |
| 13 | `0x10c183a3` | 2 | `rlandi`,`rlandi.` → `FUN_10c1772b` |
| 14/15/16 | `0x10c18373`/`0x10c18397`/`0x10c1837f` | 1/1/1 | `mr`,`mr.`,`vmr` |
| 17 | `0x10c18448` | **445** | do nothing |

**`vmr128` (`0x294`) is outside the index**: the bound is `(u32)(op-1) > 0x292`,
so reading one entry too many invents a bogus "arm 51" whose jump-table word is
`0x11111111`. That is a trap for the next reader, and it is why the count is
659 and not 660.

**No arm thunk reaches an instruction constructor directly** (`--peephole`'s
`no-mint` column, all 18). This bounds but does **not** settle prereg P4.2: the
check reads the 24-byte thunk, not the handler it tail-calls transitively.

> **⛔ EXTENDED — lane `w-tailread`, 2026-08-23, board #3462.** Arm 6's handler
> read, and then obj-checked, with a result that goes against the read.
>
> **`[R]`** Arm 6 is a 12-byte thunk `mov ecx,esi / call 0x10c16fbd /
> jmp 0x10c18448`. **`FUN_10c16fbd`** (191 B, 1 caller, 7 callees) is a
> **redundant-move eliminator with a copy-propagation fallback**: if the `+0x28`
> and `+0x2c` operands' `[+0x1c]` fields name the **same register** it clears
> bit `0x40` on both descriptors and tail-calls `0x10c16cde`, which unlinks the
> instruction; otherwise it attempts propagation through `0x10c16a46` /
> `0x10bfc132` / `0x10c16ba5` / `0x10c16c66`. Its class-1 siblings each have
> their own handler: arm 14 `mr` → `0x10c16d83`, arm 15 `mr.` → `0x10c1707c`,
> arm 16 `vmr` → `0x10c16e59`.
>
> **`[O]` AND HERE IS WHY `[R]` IS NOT A LICENCE.** `probe_selfmove.py` over
> **120,000 objs** (176,969 `.text` sections, 1,726,709 words, **135,218**
> move-form instructions decoded — the liveness half):
>
> | form | non-self | self-move |
> |---|---:|---:|
> | `fmr` | 32,569 | **0** |
> | `mr.` | 150 | **0** |
> | `mr` | 102,499 | **3,792** |
>
> **c2 emits `mr r8,r8`** — 3,792 times, in 1,206 objs (1.00 %), at `/Ox`, with
> no relocation covering those offsets. **3,792 of 3,792 name `r8`** and none
> any other register, and they sit adjacent to branches. That is an idiom, and
> this lane does not settle which one.
>
> **Arm 6 itself is clean**: 0 self-moves across 32,569 `fmr`. What the corpus
> refutes is the *generalisation* to all class-1 opcodes, and the violator is
> arm 14, whose handler was **not** read. Do not quote the `[R]` reading as
> *"c2 emits no self-move"*.

---

## 6. A table that LOOKS like the answer and is not  `[R]`

At `0x10b1d180`, immediately past `_first` (`0x298`) in the mnemonic array,
there is a **stride-16 table** of rows `{char *name, u32 machine_opcode, u32 BO,
u32 BI}` — a genuine simplified-mnemonic expansion table, and it decodes
perfectly: `beq → (bc, BO=12, BI=2)`, `bnl → (bc, 4, 0)`, `bltlr → (0x27, 12,
0)`, `twlti → (twi, 0, 16)`.

**It is not the codegen expansion table, and a lane that adopted it would be
wrong.** Its only two references in the whole image are inside `FUN_10c00900`
and `FUN_10c0174b` (99 B each, one caller each), and both are **string-compare
loops** — name → encoding. Both are called only from `FUN_10c027d3` (8,796 B),
itself called only from `FUN_10bbe561`. That is a **name-lookup / assembler**
path, not the instruction-list rewrite.

The tell that caught it: under the obvious index hypothesis (`op = 0x298 + j`)
`0x2f0` decodes to the trap mnemonic `twlti`, while `0x2f0`'s arm in §4
demonstrably calls the prologue driver. **Both cannot be true, so the index
hypothesis is wrong** — and the mapping from this table's row index to an
`instr[1]` opcode is **unresolved and deliberately not published here.**

This is the `.bss`-bump failure mode (`C2_MAP_METHOD.md` §7) caught before it
shipped: a small, clean, correctly-read table that is simply not on the path the
inputs take.

> **⛔ SETTLED — lane `w-tailread`, 2026-08-23, board #3463. The refusal above
> was right, and the reason is that the question has no answer: THERE IS NO
> INDEX MAPPING, because nothing indexes this table by an opcode.** Full read at
> [`P_OPATTR.md`](P_OPATTR.md) §7.
>
> * **13 references in 3 functions, and `FUN_10c00900` is not one of them** —
>   it references **`0x10b1b260`**, the *first* table. The split by **field** is
>   the answer: `FUN_10c0174b` names only the base and row 1 (3 refs, the name
>   search); `FUN_10c027d3` reads **`+0x4`** ×1, **`+0x8`** ×1 and **`+0xc`** ×7
>   — opcode, `BO`, `BI`; `FUN_10c01f23` holds one end-pointer.
>   *(Counting this needs care: both tables live inside `.text`, so `objdump -d`
>   disassembles their bytes as code and invents branch operands that land in
>   the table's own range. Seven such phantoms must be filtered out by function
>   membership.)*
> * **It is a name search.** `FUN_10c0174b` starts at row 1
>   (`xor ebp,ebp / inc ebp`), strides `shl eax,4`, string-compares each row's
>   name and terminates on the `_last` string `0x10b19ce4` — the exact twin of
>   `FUN_10c00900`'s `imul eax,eax,0xc` walk of the first table. **`ebp` is a
>   search cursor and is never an opcode.**
> * **The row becomes an opcode by reading field `+4`, at one address.** Its
>   sole caller, `0x10c0298d`, does exactly that four instructions later:
>   `shl eax,0x4 / mov ecx,[eax+0x10b1d184]` at **`0x10c0299c`**. No index
>   arithmetic is involved and none is recoverable, because none exists.
> * **The table can never name a pseudo-op.** Over its 122 rows the `+4` field
>   has min `0x0`, max `0x295`, 23 distinct values, and **0 rows ≥ `0x298`**.
>   So no row of it denotes `0x2f0`, and the contradiction's premise is empty.
> * **Where `twlti` came from, reproduced exactly.** Row **88** is `twlti`
>   (real opcode `0x19d` = `twi`, BO 0, BI 16) and `0x2f0 − 0x298 = 88`. The
>   arithmetic was right; the hypothesis `op = 0x298 + j` is computed **nowhere
>   in the image**. (Indexing the *first*, stride-12 table past its extent —
>   board #3357's trap — gives a different garbage mnemonic again,
>   `0x2f0 → twige`.)

---

## 7. What this page does NOT claim

* **§3's counts are `[R]`.** They count constructor *calls*, which is a
  count of nodes, not a proof that each becomes a `.text` word. An opcode with
  no encoding (`P_ENCODE.md`'s form-0 rows) is a node that is not a word.
* **Word count is a scalar projection.** The table says *how many*; it does not
  say *which* words. An arm marked `1..1` whose single word is the wrong
  instruction scores identically here.
* **`0x10c3afd8`'s class table is located, not read** (§1.2). 767 opcodes reach
  it; "reaches the tail" is not "is unchanged".
* **The 10 shared fall-through bodies are excluded from the 69/29 count** by a
  width rule, not by reading them. The true arm map is a superset.
* **No `crates/` byte was changed** and no `DISCLOSURE.md` row is owed — nothing
  here was adopted.
* **§4.4's corpus is post-everything.** It cannot separate what the expansion
  emitted from what the peephole then deleted; only a live tap could, and this
  lane built none.

## 8. The three follow-ups this read hands over, ranked

1. **Read `0x10c3afd8`** (§1.2) — one byte table, 767 opcodes, closes the
   largest remaining hole in §1's denominator.
2. **Resolve the `0x10b1d180` index** (§6) — cheap, and it converts a landmine
   into a second expansion table.
3. **`0x2f5`** — minted by IL arm 48 (`P_ILRECORD.md:254`) and handled by no arm
   of this switch. Either another pass consumes it or it is dead.
