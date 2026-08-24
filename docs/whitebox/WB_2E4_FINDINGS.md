# WB_2E4 — `0x2e4` is an unnamed, one-operand conditional-branch pseudo-opcode whose operand is a label, and **the name is not in the binary**

> **PROVENANCE — DISASSEMBLY-DERIVED.** See [`DISCLOSURE.md`](DISCLOSURE.md).
> Nothing here may enter `crates/` without a `DISCLOSURE.md` row naming the
> address it came from. **This lane adopts nothing.**

**Lane:** `w-2e4` · characterization · **Fixtures:** none · **Census:** +0 ·
**reach: 0** · **prereg:** [`docs/rungs/_2026-08-24-w-2e4-prereg.md`](../rungs/_2026-08-24-w-2e4-prereg.md)
· **rung:** [`docs/rungs/2026-08-24-w-2e4.md`](../rungs/2026-08-24-w-2e4.md)
· **board:** `#3501`–`#3504`
· **image:** `c2.dll` sha256 `c80981c015166eff…a66258`, 1 347 072 B.

**Subject.** `w-r8idiom`'s **#1-ranked follow-up**
([`2026-08-24-w-r8idiom.md`](../rungs/2026-08-24-w-r8idiom.md) § "Found and not
taken" 1). [`WB_R8IDIOM_FINDINGS.md`](WB_R8IDIOM_FINDINGS.md) §6 is a **written
refusal**: it read that `0x2e4` has no mnemonic, that it is tested with `je`,
and that three published predicates group it with `bc`/`bca`, and it declined
to name its identity, operands or contract. **That refusal was right and this
lane closes two of its three parts and converts the third from an omission into
a measurement.**

---

## 0. The answer, in seven lines

| # | claim | class |
|---|---|---|
| **A** | **`0x2e4` has no name in this image, and that is now a MEASURED absence rather than an unchecked one.** Exactly **one** of the **34** pointer-to-string arrays in `.text`/`.data` passes a four-row control (`0x21`→`bc`, `0x22`→`bca`, `0x276`→`nop`, `0x290`→`emit`) — the mnemonic table `0x10b1b260` — and it is **75 rows too short**: it ends at `0x296` (`_first` … `_last`(`0x295`) … `illegal`(`0x296`)). | `[R]` |
| **B** | **Its kind is `0x12` by construction, not by inference.** All seven mints go through **`0x10bd76e6`**, which begins `mov cl,0x12 / call 0x10bd3750` — the node allocator writes `cl` to `node[+8]`. A `0x2e4` **cannot** be any other kind. | `[R]` |
| **C** | **It carries exactly one operand and that operand is a LABEL.** The constructor takes the target as its first stack argument, tests `BYTE PTR [target+8] == 0x1b` (label), registers a **kind-`0x1d` predecessor record** in the label's list at `label+0x28` (`0x10bd3f62`) and stores a label-address operand (`0x2a7`) in `tuple+0x28`. All seven mints pass **`0`** for the second operand. | `[R]` |
| **D** | **It is minted at 7 sites in 4 TUs** — `ehexcept.c` ×4, `except.c` ×1, `fg.c` ×1, `lower.c` ×1 — out of **171** callers of that constructor. The `ehexcept.c` pair splices **BEFORE** (`0x10bd3824`) and **AFTER** (`0x10bd3815`) the same anchor: that is the bracket `w-r8idiom` measured on the obj side. The `fg.c` mint is **PGO-only** (`DAT_10c3de20 == 2`). | `[R]` |
| **E** | **The contract is one predicate, and it is written out verbatim in at least two TUs.** `PLAIN_CONDITIONAL(t) := t.kind == 0x12 && t[+0x34] == 0 && t.opcode ∉ {0x2e4, 0x21 (bc), 0x22 (bca)}`. **146 of 185** `cmp …,0x2e4` sites co-test `0x21` **and** `0x22` in the same window. | `[R]` |
| **F** | **In the flow graph a `0x2e4` does NOT end a basic block, and a walk steps over it.** `fg.c`'s block builder `0x10b372ea` reaches its block-terminator case for kinds `0x12`/`0x14`/`0x18` and then **exempts `0x2e4`** (`0x10b374c3`), and separately skips runs of `0x2e4` when looking for the significant neighbour of a call (`0x10b373c1`). | `[R]` |
| **G** | **`0x2e4` is one of a contiguous branch-pseudo family `0x2df`…`0x2e6`**, which the constructor flags with `tuple[+9] \|= 8` at `0x10bd7754` — a bit **52 of 64** test sites branch *away* on. | `[R]` |

**What is still refused: the NAME.** §1 gives the search, its control, and the
trap it was built to defeat. **A role sentence is offered in §6 and it is
labelled a story, not a claim.**

---

## 1. The NAME — refused, and the refusal is now a measurement

### 1.1 The mnemonic table ends at `0x296`, and this is read, not inherited `[R]`

`ref/P_ENCODE.md` §2.1 bounds the **base-word / encode-form** tables at
`_last = 0x295`. `WB_R8IDIOM_FINDINGS.md` claim B carried that bound over to
the **mnemonic** table without measuring it. Measured here:

| opcode | row VA | string |
|---|---|---|
| `0x294` | `0x10b1d150` | `vmr128` |
| `0x295` | `0x10b1d15c` | **`_last`** |
| `0x296` | `0x10b1d168` | **`illegal`** |
| `0x297` | `0x10b1d174` | `_last` (the terminator word of the table that follows) |
| `0x298` | `0x10b1d180` | `_first` — **a different table** (§1.4) |

**So `w-r8idiom` claim B is right about the conclusion and one row off about
the boundary**: the mnemonic table's own opcode space is `0x000`…`0x296`, not
`0x000`…`0x295`. Amended beside; nothing that depended on it moves.

### 1.2 The trap: row `0x2e4` of the mnemonic table reads `twle` `[R]`

`0x10b1b260 + 0x2e4 × 12 = 0x10b1d510`, which is **`0x39c` past the end of the
table**, inside the extended-mnemonic table of §1.4, and it holds a pointer to
the real string **`twle`**.

> **A reader who indexes the mnemonic table by `0x2e4` gets a plausible PPC
> trap mnemonic and no warning at all.** This is prereg P2.5's registered
> failure mode, and it fired on the first command of the lane.
>
> **And this trap was already documented, by name, before this lane ran.**
> [`WB_MIDDLE_INTERFACES.md`](WB_MIDDLE_INTERFACES.md) §2.2 records walking
> into the identical thing at a different index (*"read as a continuation of
> the first table at index `0x298`, it decodes tuple opcode `0x30f` as
> `tdlngi` — a trap instruction — in a function that is `return a+b+c`"*), and
> [`ref/P_ENCODE.md`](ref/P_ENCODE.md) §2.1 tabulates three more of its
> phantoms. **I hit it anyway**, because a trap recorded in prose two pages
> away does not stop a `u32` read. That is the argument for putting the control
> **in the tool**: `dump_pseudoop.py --names` is a control-checked search
> rather than one array lookup, and it prints `covers? no` where a lookup would
> print `twle`.

### 1.3 The search, and the control that makes it a search `[R]`

`dump_pseudoop.py --names 0x2e4` enumerates **every** maximal run of ≥ 20
consecutive entries whose first dword points at a printable C string, in
`.text` and `.data`, at strides 4/8/12/16 and **at every 4-byte phase**. It
finds **34** such arrays (the first version of it found 15 — §8). For each it runs a four-row control —
`0x21`→`bc`, `0x22`→`bca`, `0x276`→`nop`, `0x290`→`emit`.

```
0x10b1b260   12     665    covers? no    control 4/4    _first, add, add.
   ... 33 others, all control 0/4 ...
NOT NAMED: no table in this image both covers index 0x2e4 and passes the control.
```

**Exactly one table names opcodes, and it stops 75 rows short of `0x2e4`.**
The other 33 are the register-name table (`0x10b181c0`), the
`__savegprlr_*`/`__restfpr_*` helper-name tables, the `/Qfast_transcendentals`
and loop-classifier enum names, `__PogoProbeVector*`, the extended-mnemonic
table `0x10b1d180` (§1.4), and phase-shifted aliases of those. **None passes a
single control row.**

**The control was watched failing before any of this was quoted**
(`--names-selftest`): it scores 4/4 at the true base `0x10b1b260` and **0/4**
at base ± one stride, at base + 4, and at base + `0x1000`. A classifier that
returns the same verdict everywhere is measuring itself — `w-r8idiom` defect 1,
four-for-four being the tell.

### 1.4 The table the coincidence lives in — ALREADY IN THE RECORD `[R]`

**`0x10b1d180` is not a new address and this lane very nearly published it as
one.** It is c2's **extended / simplified-mnemonic table**, stride 16,
`{char *name, u32 real_opcode, u32 BO, u32 BI}`, and the record already holds:

* [`WB_MIDDLE_INTERFACES.md`](WB_MIDDLE_INTERFACES.md) §2.2 — *"a SECOND table
  nobody had named"*, with ten rows decoded and the trap it sets;
* [`ref/P_EXPAND.md`](ref/P_EXPAND.md) §6 — *"a table that LOOKS like the
  answer and is not"*, with the refusal to publish an index mapping;
* [`ref/P_OPATTR.md`](ref/P_OPATTR.md) §7 (`w-tailread`, board `#3463`) —
  **settled**: nothing indexes it by an opcode; 13 references in 3 functions;
  the row index becomes an opcode only through field `+4` at `0x10c0299c`.

> **This is the standing "check the record before claiming novelty" rule, and
> it caught me between the read and the commit.** The first draft of this
> section was headed *"a table this record did not have"*. It had it three
> times over. Recorded rather than quietly deleted, because a re-discovery
> published as a discovery is how a record starts disagreeing with itself.

**Two small things this re-read does add**, and they are the only claims here:

1. **The complete row grouping**, which no page carries (the three above give
   ten rows between them):

| rows | names | base opcode | field `+8` (BO) | field `+0xc` (BI) |
|---|---|---|---|---|
| 0 | `_first` | — | — | — |
| 1–6 | `subi`, `subis`, `subic`, `subic.`, `sub`, `subc` | `0x0b`,`0x0e`,`0x0c`,`0x0d`,`0x181`,`0x183` | 0 | 0 |
| 7–14 | `blt`,`ble`,`beq`,`bge`,`bgt`,`bnl`,`bne`,`bng` | **`0x21` (`bc`)** | `0xc`/`0x4` | `0`/`1`/`2` |
| 15–20 | `b{lt,ne,gt,le,ge,eq}ctr` | `0x23` | `0xc`/`0x4` | 0–2 |
| 21–26 | `b{lt,ne,gt,le,ge,eq}lr` | `0x27` | `0xc`/`0x4` | 0–2 |
| 27–34 | `cmpd`,`cmpw`,`cmpdi`,`cmpwi`,`cmpld`,`cmplw`,`cmpldi`,`cmplwi` | `0x2d`–`0x30` | 0 | the `L` bit |
| 35–45 | `mftbu`,`mftbl`,`mtxer`,`mtlr`,`mtctr`,`mtsrr0`,`mtsrr1`,`mtdar`,`mttbl`,`mttbu`,`mtdabr` | `0xe9` / `0xf8` (`mtspr`) | 0 | SPR number |
| 46–54 | `mfxer`…`mfdabr` | `0xe6` (`mfspr`) | 0 | SPR number |
| 55–71 | `trap`, `twlt`…`twlng` | `0x19c` | 0 | the `TO` field |
| 72–87 | `tdlt`…`tdlng` | `0x197` | 0 | `TO` |
| 88–103 | `twlti`…`twlngi` | `0x19d` | 0 | `TO` |
| 104–119 | `tdlti`…`tdlngi` | `0x198` | 0 | `TO` |
| **120** | **`_last`** | `0x295` | 0 | 0 |

2. **A correction to `P_OPATTR.md` §7.1.** That table calls `FUN_10c01f23`'s
   reference to `0x10b1d180 + 0x790` (row **121**, VA `0x10b1d910`) *"a
   table-end pointer"*. **Row 121 is not past the end** — it is the first row
   of a **second alias block with 32-byte entries**: `extldi`(`0x12d`),
   `extrdi`(`0x12b`), `insrdi`(`0x12f`), `rotldi`, `rotrdi`, `rotld`(`0x125`),
   `sldi`, `srdi`, `clrldi`, `clrrdi` — the rotate/mask simplified mnemonics,
   which need more than `(BO, BI)` to expand. `FUN_10c0174b` terminates on the
   `_last` **string** and so never reaches them with its stride-16 walk, which
   is consistent with §7.1's read that this is a *different* consumer.
   **What that consumer does is not read here.**

### 1.5 What is therefore refused, in one sentence

> **The identifier c2's own source used for `0x2e4` is not recoverable from
> this binary.** [`WB_MIDDLE_INTERFACES.md`](WB_MIDDLE_INTERFACES.md) §2.3
> already said *"tuple opcodes above `0x297` are structural pseudo-ops with no
> mnemonic at all"* — as an `[O]` observation over four fixtures. **What this
> section adds is that it is also true as an `[R]` property of the whole
> image**, established by an enumeration with a control rather than by
> sampling, so it now covers every opcode and not only the five that appeared. It is a compile-time enumerator with no runtime string: the
> release image contains no tuple-opcode dumper (there is no `dmp.c` in the
> 53-TU list — see [`c2_tus.tsv`](c2_tus.tsv)), and the only two opcode strings
> in the diagnostics path, `"Unknown opcode"` and
> `"Opcode not supported by backend"` (`0x10b1e9dc` / `0x10b1e940`, one xref
> each, both `0x10c027d3`), are on the **encoder** path and print no number.
> **Do not fit a name to it.** §2–§5 give what the binary *does* say, and that
> is a contract, which is what a port needs.

---

## 2. The OPERANDS — settled

### 2.1 `0x10bd76e6`, the branch-tuple constructor, read in full `[R]`

`__fastcall(ecx = opcode, dl = condition-code) ` + **five** stack arguments,
`ret 0x14`. The **last** push is argument 1.

```
10bd76e6  push ebp / mov ebp,esp / push ecx / push ebx / push esi / push edi
10bd76ed  mov  edi,ecx                 ; edi = OPCODE
10bd76ef  mov  cl,0x12                 ; <== KIND 0x12, hard-coded
10bd76f1  mov  bl,dl                   ; cc
10bd76f6  call 0x10bd3750              ; node allocator: node[+8] = cl
10bd76fb  mov  esi,eax
10bd7700  or   BYTE PTR [esi+0x9],0x1  ; "real tuple" bit (R2)
10bd7706  mov  DWORD PTR [esi+0x4],edi ; tuple[+4] = OPCODE
10bd7709  mov  edi,DWORD PTR [ebp+0x8] ; arg1 = the TARGET
10bd770e  xor  BYTE PTR [esi+0xa],al   ; cc into the low 5 bits of +0xa
10bd7711  cmp  BYTE PTR [edi+0x8],0x1b ; is the target a LABEL?
10bd7715  jne  0x10bd772b
10bd7717    call 0x10bd3f62            ;   yes: register a PREDECESSOR record
10bd7724    call 0x10bd41b8            ;   and turn the label into an operand
10bd772b  call 0x10bd6e89 / mov [esi+0x28],eax     ; operand 0  <- arg1
10bd7738  test ecx,ecx (arg2)          ; operand 1 <- arg2, chained at [op0]
10bd7754  cmp  DWORD PTR [ebp-0x4],0x2df / jb
10bd775d  cmp  DWORD PTR [ebp-0x4],0x2e6 / ja
10bd7766    or BYTE PTR [esi+0x9],0x8  ; <== THE FAMILY BOUND, 0x2df..0x2e6
10bd776a  push [ebp+0x14] (splice cb) / mov ecx,[ebp+0x18] (anchor)
10bd7772  call 0x10bd5732              ; link it in, then tuple[+0x14] = line no.
```

Two things fall straight out and neither needed a probe:

* **`0x2e4` tuples are kind `0x12`.** Not "usually", not "in the sites read" —
  **by construction**, because the only constructor that mints them writes
  `cl = 0x12`. Prereg P3.4 predicted this at 85 %; it is stronger than
  predicted, because it is a property of the mint rather than of the sites.
* **`0x2e4` is inside a contiguous opcode family `0x2df`…`0x2e6`** which c2
  itself brackets with two compares. §5.3.

### 2.2 The splices, and which side is which `[R]`

| VA | body | meaning |
|---|---|---|
| `0x10bd3824` | `new->[0x10] = a->[0x10]; a->[0x10] = new; new->[0] = a; new->[0x10]->[0] = new` | **splice BEFORE** |
| `0x10bd3815` | `new->[0] = a->[0]; a->[0] = new; new->[0x10] = a; new->[0]->[0x10] = new` | **splice AFTER** |

`+0x00` is **next** and `+0x10` is **prev**. Established two ways rather than
guessed: `0x10bd417d` uses `0x10bd3815`'s body verbatim to insert a
**fall-through** label *after* a tuple, and
[`WB_MERGER4_FINDINGS.md`](WB_MERGER4_FINDINGS.md) independently reads *"walks
both predecessors **backwards** in lockstep through `tuple+0x10`"*.

### 2.3 The predecessor record — the CFG edge `[R]`

`0x10bd3f62(ecx = label, edx = branch)`:

```
mov cl,0x1d / call 0x10bd3750        ; a KIND-0x1d node
mov [eax+0xc],edi                    ; ->[0xc] = the branch tuple
... append to label->[0x28], tail kept at (head)->[0x14]
inc DWORD PTR [label->[0x24] + 0x3b] ; bump the label symbol's reference count
```

`label+0x28` is the **predecessor list** `WB_MERGER4_FINDINGS.md` walks. So
**minting a `0x2e4` adds a real edge to the flow graph**, visible to every pass
that walks predecessors, and bumps the label's reference count so nothing
deletes it.

`0x10bd41b8(ecx = label, edx = 0)` then builds a **kind-4** operand node with
`node[+4] = 0x2a7` and `node[+0x18] = label->[0x24]` — a label-address operand.
That is what lands in `tuple+0x28`.

### 2.4 The canonical target chase `[R]`

Three unrelated TUs read a `0x2e4`'s target with the *same* three hops:

```
target_label_symbol(t) = t[+0x28] -> [+0x18] -> [+0x33]
```

— `inline.c` `0x10b6e9f6`, `globopt.c` `0x10b4d7f9`, `lower.c` `0x10c0e240`,
and `ehexcept.c` produces the argument from the mirror of it (`[eax+0x33]`).
`+0x33` is an **unaligned** field, so the enclosing structure is packed.

---

## 3. WHO MINTS IT — seven sites, and the published table needs a correction

### 3.1 The correction `[R]`

`WB_R8IDIOM_FINDINGS.md` §4 carries a table captioned **"Who mints `0x2e4`"**
with eight functions across six TUs, built from `mov ecx,imm32` sites. Measured
mechanically here — a site counts as a mint only if the value **reaches `ecx`
of a known constructor with no intervening write** — the 224 immediate uses of
`0x2e4` split:

```
224 sites | MINT 7 | TEST 208 | SELECT 9 | 160 functions | 33 TUs
```

and the **9 SELECTs were each read by hand**: all nine are the same idiom, a
loop-invariant hoist of the comparison constant into a callee-saved register
(`mov ebp,0x2e4` … later `cmp eax,ebp`). **None of the nine is a mint**, so the
honest split is **7 mints and 217 tests**.

| `w-r8idiom` row | verdict here |
|---|---|
| `0x10be3e4c` `ehexcept.c` ×4 | ✅ **4 real mints** |
| `0x10be4f28` `except.c` ×1 | ✅ **1 real mint** |
| `0x10b39937` `fg.c` | ✅ **1 real mint** (PGO-only — §3.3) |
| `0x10c0d57e` `lower.c` | ✅ **1 real mint** |
| **`0x10b372ea` `fg.c`** | ❌ **not a mint** — a *consumer*, and the most informative one in the image (§4.2) |
| **`0x10b6e99b` `inline.c`** | ❌ not a mint — the canonical predicate (§4.1) |
| **`0x10b9f04e` `p2symtab.c`** | ❌ not a mint — the canonical predicate again |
| **`0x10b9fb3f` `p2symtab.c`** | ❌ not a mint |

> **The brief's #1 pointer was half right.** It sent this lane to
> `0x10b372ea` **and** `0x10b39937` for "`fg.c`'s edge construction". Only
> `0x10b39937` constructs anything. `0x10b372ea` turned out to be worth the
> visit anyway — it is where the *contract* is, not the construction — but
> **a lane that had gone looking only for a mint would have found nothing
> there and reported the address wrong.**

### 3.2 The denominator `[R]`

**171** call sites of `0x10bd76e6` in the image, with the opcode recovered from
the preceding `mov ecx,imm32`:

| opcode | mints | | opcode | mints |
|---|---:|---|---|---:|
| `0x2dd` | **60** | | `0x2e4` | **7** |
| `0x2de` | **50** | | `0x2e5` | 4 |
| `0x288` | 3 | | `0x2e6` | 4 |
| `0x289` | 2 | | `0x2df`,`0x2e0`,`0x2e1` | 1 each |
| *opcode in a register* | 20 | | *not recovered* | 18 |

`0x2e4` is a **rare** member of a family whose two common members are `0x2dd`
(60) and `0x2de` (50). The 20 register-carried sites are a **stated blind
spot**: a mint whose opcode arrives in a variable is invisible to an
immediate scan, so **7 is a floor, not a total.**

### 3.3 `ehexcept.c` — the bracket, and it is literally a bracket `[R]`

`FUN_10be3e4c` mints four times. Two of the four sit in a list walk over
`region->[0x20]`, one element at a time (`edi = edi->[0]`), each contributing
`push [edi->[0x10] + 0x33]` — a label — and they differ **only** in the splice:

| VA | splice | anchor |
|---|---|---|
| `0x10be3fdf` | `0x10bd3824` — **BEFORE** | `esi` |
| `0x10be40d8` | `0x10bd3815` — **AFTER** | `esi` |
| `0x10be41ba` | `0x10bd3815` — AFTER | `esi` |
| `0x10be42a4` | `0x10bd3824` — BEFORE | `ebp` |

> **This is the mechanism behind `w-r8idiom` §1.3–§1.4 exactly.** *"Runs
> bracketing a call"* is one loop that splices before the anchor and one that
> splices after it; *"run length == `__catch$` count on 95.19 %"* is *one
> `0x2e4` per element of the region's list*. The `[R]` side and the `[O]` side
> were measured by different lanes with different instruments and they agree.
> **The `[R]` side does NOT establish that the walked list is the catch-handler
> list** — that identification comes from the `[O]` side and is quoted as such.

### 3.4 `fg.c`'s mint is PGO-ONLY, and that is a live constraint `[R]`

`0x10b39937` is a branch folder. When it folds a two-way branch it rewrites the
surviving tuple's opcode to **`0x2de`** (`0x10b39add`) — and *first*, gated by

```
10b39aa9  cmp DWORD PTR ds:0x10c3de20,0x2   ; PGO level == 2 (/LTCG:PGO optimize)
10b39ab2  cmp DWORD PTR ds:0x10c3dd78,ebp   ; and a second global non-zero
```

it mints a `0x2e4` to the tuple's **fall-through label** (`0x10bd417d`) and
then **clears** the family bit the constructor had just set
(`and BYTE PTR [eax+0x9],0xf7`, `0x10b39ad6`).

`DAT_10c3de20` is [`WB_DAGCLIENTS_FINDINGS.md`](WB_DAGCLIENTS_FINDINGS.md) §2's
PGO level (`1` = `/LTCG:PGI`, `2` = `/LTCG:PGO`, `0` = ordinary).

> **So the `fg.c` producer cannot fire on this project's workload at all**, and
> the corpus's *"100 % of bearing objs are EH"* (`w-r8idiom` §1.5, its
> registered failure mode) is now partly explained from the image rather than
> apologised for: two of the four producers are EH files, one is PGO-gated, and
> the fourth is `lower.c`.

### 3.5 `lower.c`'s mint `[R]`

`0x10c0e236`, inside the **same** final-expansion switch `FUN_10c0d57e` whose
arm `0x10c0e194` emits the word. A different arm mints a **fresh** label
(`0x10b9a455` → `0x10bd415e`), builds a `0x2e4` pointing at it, and then
splices that label in before the current tuple. So expansion both **produces**
and **consumes** `0x2e4`.

---

## 4. THE CONTRACT — what a pass is required to do

### 4.1 The predicate, read verbatim in two TUs `[R]`

`p2symtab.c` `0x10b9f04e` and `inline.c` `0x10b6e99b` contain the identical
sequence:

```
cmp BYTE PTR [t+0x8],0x12      ; kind must be a branch
jne  no
cmp DWORD PTR [t+0x34],0x0
mov  ecx,0x2e4
jne  yes                       ; +0x34 != 0            -> NOT a plain branch
cmp  DWORD PTR [t+0x4],ecx
je   yes                       ; opcode == 0x2e4       -> NOT a plain branch
cmp  eax,0x21 / je yes         ; opcode == bc          -> NOT a plain branch
cmp  eax,0x22 / jne no         ; opcode == bca         -> NOT a plain branch
```

```
PLAIN_CONDITIONAL(t)  :=  t.kind == 0x12
                       && t[+0x34] == 0
                       && t.opcode ∉ { 0x2e4, 0x21 (bc), 0x22 (bca) }
```

This is the classification
[`WB_DAGCLIENTS_FINDINGS.md`](WB_DAGCLIENTS_FINDINGS.md) §2 and
[`WB_MERGER4_FINDINGS.md`](WB_MERGER4_FINDINGS.md) §2 both attribute to
`0x10b3c2cc` and both list under "grey zone — *those opcode numbers were not
decoded*". They are decoded now.

**The two halves are not redundant.** `0x10bd76e6` **never writes `+0x34`**,
and the recycled-node path of the allocator `memset`s the node to zero
(`0x10bd3788 → 0x10c28862`, which is `MSVCR100.dll!memset` — resolved through
the import table, not guessed). So a `0x2e4` looks *exactly* like a plain
conditional branch in `+0x34`, and **the opcode test is the only thing that
distinguishes it.**

### 4.2 The flow graph: a `0x2e4` does **not** end a block `[R]`

`fg.c`'s block builder **`FUN_10b372ea`** (559 B) walks the tuple list and
computes, per tuple, "does a new block start before this one". Its
kind-dispatch chain is

```
10b373fd  eax = kind - 0x10
          je 0x10b374e1   (0x10)
          eax -= 2; je 0x10b374ba   (0x12)   <-- conditional branch
          eax -= 2; je 0x10b374ba   (0x14)
          eax -= 1; je 0x10b3744d   (0x15)
          eax -= 2; je 0x10b37438   (0x17)
          eax -= 1; je 0x10b374ba   (0x18)
          eax -= 3; jne next        (0x1b = label -> t[+0x20] = current block)
```

and the branch case is

```
10b374ba  cmp DWORD PTR ds:0x10c2e2b8,0x0
10b374c1  je  0x10b374cc
10b374c3  cmp DWORD PTR [esi+0x4],0x2e4      <== HERE
10b374ca  je  next                            ; <-- do NOT set "block ends"
10b374cc  ebp = 1                             ; every other branch DOES
10b374cf  cmp DWORD PTR [esi+0x4],0x2df
10b374d6  jne next
10b374d8  or  DWORD PTR [edi+0x18],0x1000000  ; 0x2df marks the BLOCK
```

> **A `0x2e4` is a branch tuple that the flow graph refuses to split a block
> on.** It contributes an *edge* (§2.3) without contributing a *block
> boundary*. That is the whole design, and it is the one sentence a port needs.

### 4.3 A walk steps over runs of `0x2e4` `[R]`

Earlier in the same function:

```
10b373a2  eax = t[+4]
          if (eax < 0x2ee) skip
          if (eax <= 0x2f4 || eax == 0x2f6 || eax == 0x2fe) {
10b373c1      p = t[+0x10]                    ; the PREVIOUS tuple
              ecx = 0x2e4
10b373cb      while (p[+4] == ecx) p = p[+0x10]   ; SKIP EVERY 0x2e4
              if (p.kind != 0x1b) ebp = 1     ; not a label -> start a block
          }
```

For the opcode class `{0x2ee…0x2f4, 0x2f6, 0x2fe}` — the class whose members
sit next to the runs on the obj side — the significant neighbour is found by
**skipping the markers**. `WB_MERGER4_FINDINGS.md` §2 reads the same idiom in
merger M4 ("skipping labels (`0x1b`) and `0x317` pseudo-tuples"); `0x2e4` is a
third member of that walk-transparent set.

### 4.4 How widely the contract is enforced, with a denominator `[O]`-free `[R]`

`dump_pseudoop.py --family 0x2e4`, over **185** `cmp …,0x2e4` sites:

| measurement | value | stability |
|---|---|---|
| the test feeds `je` | **177 of 185** | exact |
| the test feeds `jne` | 8 of 185 | exact |
| co-tested with **both** `0x21` and `0x22` | **146 of 185 (78.9 %)** | **143…147 (77.3–79.5 %)** across windows ±6, ±14, ±24, ±40 — **a 6.7× change in the window moves it 2.2 points** |
| a `cmp [reg+0x34],0x0` in the same window | 123 of 185 (66.5 %) | **111…127 (60.0–68.6 %)** over the same windows |

> **Instrument-defect discipline, applied and reported both ways.** The
> `{bc, bca}` co-test rate is a property of **c2**: it barely moves when the
> window changes. The `+0x34` rate is **partly a property of my window** — it
> moves 8.6 points over the same sweep — and it is quoted as a range and must
> not be quoted as a single number. Three lanes have now been bitten by a
> number that was a property of its traversal; this one says which of its two
> numbers is which.

The site enumeration itself is invariant: `--sites 0x2e4` with objdump run in
**1, 3 and 7 chunks with different boundaries** produces a **byte-identical**
224-row listing. A raw byte scan of `.text` for `e4 02 00 00` at any alignment
finds **229**, so the 224 disassembled sites are bounded above by a number the
disassembler cannot bias.

### 4.5 The expander and the peephole — `w-r8idiom`'s half, unchanged `[R]`

Nothing here disturbs `w-r8idiom`: arm `0x10c0e194` is still the only consumer
in the final-expansion switch, it still emits `emit 0x7d084378`, and the
peephole `FUN_10c182b4` still bounds its opcode at `0x295` three times so arm
14 never sees a `0x2e4`. **What is added is why there is a word at all**: the
tuple is an *edge*, edges must survive to the EH tables, and an edge with no
instruction has no address.

---

## 5. Corrections and amendments to the published record

### 5.1 `w-r8idiom` §4's mint table over-counts `[R]`

Four of its eight rows are not mints (§3.1). The caption, not the addresses, is
what is wrong — every address it gives does reference `0x2e4`.

### 5.2 `w-r8idiom` claim B is one row off `[R]`

The mnemonic table ends at `0x296` (`illegal`), not `0x295` (`_last`). §1.1.

### 5.3 "tested with `je` in **18** TUs" is low `[R]`

**185 `cmp` sites**, 224 immediate uses across **160 functions and 32 named
TUs** (plus **1** site, `0x10c0ee91`, that falls outside every `FUNCS.tsv`
extent — recorded, not swept): `dag.c`, `ehexcept.c`, `except.c`, `factor.c`, `fg.c`, `globdf.c`,
`globlopt.c`, `globopt.c`, `hash.c`, `inline.c`, `list.c`, `lower.c`,
`lowersmd.c`, `ltcg.c`, `lur.c`, `mdlist.c`, `mdmisc.c`, `misc.c`,
`optimize.c`, `p2symtab.c`, `pogocg.c`, `pogoinline.c`, `pogoopt.c`, `ptinl.c`,
`reader.c`, `regasg.c`, `sizeopt.c`, `smdmisc.c`, `ssa_seh.c`, `stack.c`,
`tuple.c`, `vlines.c`. The three heaviest are **`fg.c` 59**, **`pogocg.c`
33**, **`p2symtab.c` 24**.

**`0x2e4` is one of the most widely tested constants in this backend.** A port
that models the tuple space at all has to carry it from the first pass onward.

### 5.4 `+0x9` bit 0 is NOT an "opcode ≤ `0x297`" predicate before expansion `[R]`

[`WB_MIDDLE_INTERFACES.md`](WB_MIDDLE_INTERFACES.md) §2.3 measures, over 4
fixtures and 288 tuples with zero counterexamples: *"`+0x9` bit 0 clear ⇒ the
opcode is above `0x297`"*, and its converse at `sched0`/`after0`.

**Every one of c2's 16 tuple constructors sets `+0x9 |= 1` unconditionally**,
regardless of opcode — checked at `0x10bd59c3`, `0x10bd7243`, `0x10bd7282` and
`0x10bd7700`. `0x10bd76e6` sets it on `0x2dd`, `0x2de` and `0x2e4` alike, all
of which are **above** `0x297`.

> **There is no contradiction, and the reason is the STAGE.** §2.3's snapshots
> are `sched0`/`after0` — *after* final expansion, by which point every `0x2e4`
> has already become an `emit` (`0x290`). The rule is sound where it was
> measured and **must not be carried back to the pre-expansion tuple list.**
> Anyone using bit 0 as a "is this a machine opcode" test on IL as `fg.c` or
> `ehexcept.c` sees it will classify every branch tuple wrongly.

(Incidentally: the other three constructors read here allocate kind **`0x0d`**,
not `0x12` — so `0x0d` is the ordinary tuple kind and `0x12` the branch kind,
which is why `0x10bd76e6` is the only constructor a `0x2e4` can come from.)

### 5.5 `WB_DAGCLIENTS_FINDINGS.md` grey-zone item 5 is closed `[R]`

*"those opcode numbers were not decoded"* — `0x21` is `bc`, `0x22` is `bca`,
`0x2e4` is the subject of this page, and §4.1 gives the predicate they form.

---

## 6. What is NOT settled, and is deliberately not guessed

* **The NAME.** §1. Not in the binary; the search and its control are given so
  the absence can be re-derived rather than re-assumed.
* **`tuple[+9]` bit 3.** Read: the constructor sets it for **every** opcode in
  `0x2df`…`0x2e6` (`0x10bd7754`); `fg.c`'s PGO mint clears it immediately
  (`0x10b39ad6`) and three other sites clear it (`0x10bc6fef` `regasg.c`,
  `0x10bcb2cc` `sizeopt.c`, `0x10bff942` `lower.c`); six sites set it;
  **64** sites test it and **52** of those branch *away* when it is set.
  **Not read: what it means.** The polarity is consistent with a
  "leave this branch alone" flag and **that sentence is an inference from a
  branch-direction count, not a read**, and it is not in the claims.
* **`tuple[+0x34]`.** Read: the constructor never writes it; the classifier
  pairs it with the opcode test; it is non-null on *some* branch class.
  **Not read: which.**
* **What the `ehexcept.c` list at `region->[0x20]` is.** The `[O]` side says
  its length tracks `__catch$` count on 95.19 %; the `[R]` side says only that
  it is a list whose elements carry labels.
* **`0x2dd`, `0x2de`, `0x2df`, `0x2e5`, `0x2e6`.** Named here only by their
  mint counts and their family membership. `0x2de` is what `fg.c` rewrites a
  folded branch to; `0x2df` sets `block[+0x18] |= 0x1000000`. Nothing else.
* **20 mints whose opcode arrives in a register** are outside an immediate
  scan. §3.2 says so rather than presenting 7 as a total.

**A story that fits every fact above** — *`0x2e4` is the compiler's
representation of a non-control-flow-transferring successor edge: an extra
outgoing edge from a call to a handler label, which the CFG must see so that
liveness and the EH tables are right, but which must not split a block and must
not be scheduled or peepholed like a real branch; at expansion it becomes one
inert word so the edge has an address* — **is a story.** It is offered because
a reader will construct one anyway and it is better to have it labelled. It is
**not** in §0's claims and no `[R]` above depends on it.

---

## 7. Addresses this lane adds to the record

| address | what | first recorded |
|---|---|---|
| **`0x10bd76e6`** | the **kind-`0x12` branch-tuple constructor** — `ecx` = opcode, `dl` = cc, 5 stack args, `ret 0x14`; hard-codes `cl = 0x12`; flags `0x2df`…`0x2e6` | listed as a "constructor" by `dump_expansion.py`; **signature and body read here** |
| **`0x10bd7754`** | the family bound `0x2df ≤ op ≤ 0x2e6` → `tuple[+9] \|= 8` | here |
| **`0x10bd3750`** | the node allocator — `cl` = KIND → `node[+8]`; free lists at `0x10c6f848`, size classes at `0x10b18910`; recycled nodes `memset` to 0 | here |
| **`0x10bd3f62`** | registers a **kind-`0x1d` predecessor record** in `label[+0x28]`, bumps `label[+0x24][+0x3b]` | here |
| **`0x10bd41b8`** | wraps a label as a **kind-4, opcode-`0x2a7`** label-address operand | here |
| **`0x10bd417d`** | get-or-create the **fall-through label** after a tuple | here |
| `0x10bd3824` / `0x10bd3815` | splice **BEFORE** / **AFTER**; `+0x00` = next, `+0x10` = prev | named by `w-r8idiom` as "a callback"; **decoded here** |
| `0x10bd5732` | the link-in tail: `0x10bd3da7`, then the splice callback, then `tuple[+0x14] = ds:0x10c2e2ec` (the current line) | here |
| **`0x10b372ea`** | `fg.c`'s **block builder** — the `0x2e4` block-exemption at `0x10b374c3` and the skip-loop at `0x10b373c1`…`0x10b373d1` | named by `w-r8idiom` as a minter; **read here, and it is not one** |
| **`0x10b39937`** | `fg.c`'s branch folder — the **PGO-only** mint at `0x10b39acc`, gated on `0x10c3de20 == 2` and `0x10c3dd78 != 0`, rewriting the folded tuple to `0x2de` | here |
| **`0x10b6e99b`, `0x10b9f04e`** | the **canonical `PLAIN_CONDITIONAL` predicate**, written out verbatim | here |
| `0x10b1d180` | the extended-mnemonic table — **NOT new**: `WB_MIDDLE_INTERFACES.md` §2.2, `P_EXPAND.md` §6, `P_OPATTR.md` §7. Complete 121-row grouping added here | already recorded |
| **`0x10b1d910`** | **row 121 — a SECOND alias block, 32-byte entries** (`extldi`…`clrrdi`), which `P_OPATTR.md` §7.1 calls "a table-end pointer" | here |
| `0x10b1d15c` / `0x10b1d168` | the mnemonic table's `_last` (`0x295`) and `illegal` (`0x296`) rows — **the true end of the table** | here |
| `0x10be5c5a` / `0x10be5c92` | `except.c`'s opcode jump table + byte index, covering `0x2dd`…`0x308`; `0x2e4` takes the **default** arm `0x10be5976` | here |
| `0x10c28862` | the `memset` import thunk (`MSVCR100.dll`), resolved through the import directory | here |
| `0x10b1e940` / `0x10b1e9dc` | `"Opcode not supported by backend"` / `"Unknown opcode"`, one xref each, both `0x10c027d3` — the **encoder** path, printing no number | strings table; **cited here as the absence-of-name evidence** |

`ref/ADDR.tsv` is **generated** (`build_ref.py`, from prose citations plus a
machine-local Ghidra export). These enter it at the next regeneration on a
machine that has the export; the citations above are the source of record
either way.

---

## 8. The instrument, and what running it caught

[`scripts/dump_pseudoop.py`](scripts/dump_pseudoop.py) — sha256-fenced against
the pinned image; `--names`, `--names-selftest`, `--sites`, `--mint`,
`--family`, `--denom`, `--split`, `--window`.

**Fences, each watched refusing deliberately broken input before any number
above was quoted:**

| fence | input | result |
|---|---|---|
| image digest | `c2.dll` truncated to 400 kB | `REFUSE`, exit 1 |
| image digest | one flipped byte at `0x10000` | `REFUSE`, exit 1 |
| mode | `--frobnicate` | `REFUSE: unknown mode`, exit 1 |
| name control | mnemonic base ± one stride, +4, +`0x1000` | **0/4, FAIL** at all four; 4/4 only at the true base |
| traversal | objdump run in 1 / 3 / 7 chunks | site listing **byte-identical** |
| traversal | family window ±6 / ±14 / ±24 / ±40 | one metric stable (2.2 pts), one not (8.6 pts) — **both reported** |

**One defect was found by running it, and it was mine.**

> `string_arrays()` started each stride at the **section base**, so a stride-12
> table whose offset from that base is not a multiple of 12 was invisible — and
> the table it hid was **the mnemonic table itself**, `0x10b1b260`, the one
> known-good name table in the image. The first `--names` run therefore
> reported "15 arrays, none passes the control", which is the *right answer for
> the wrong reason*.
>
> It surfaced because a control that **must** pass never appeared in the
> enumeration at all. **A search for "is X named anywhere" that cannot see the
> one table that names things is not a search**, and a lane that had shipped
> the 15-array run would have published a correct conclusion resting on a
> broken instrument. Every 4-byte phase is scanned now, the array count went
> 15 → 34, and the mnemonic table appears with control 4/4 and the words
> `covers? no`.

---

## 9. Consequences for the port

**None adopted, and none required.** For a future lane that models the tuple
space:

* `0x2e4` is a **kind-`0x12`, one-operand branch tuple whose operand is a
  label**, and minting one must **also** add a predecessor record to that
  label and bump its reference count — a port that mints the tuple without the
  edge will lose the label to dead-label elimination.
* It must **not** split a basic block, and any "find the significant
  neighbour" walk must **skip** it, together with labels (`0x1b`) and `0x317`.
* Every `PLAIN_CONDITIONAL` test in the port must exclude it alongside `bc` and
  `bca`; that predicate appears in ~30 of c2's 53 TUs.
* It is **rare**: 7 of 171 branch-tuple mints, and one of those four producers
  is PGO-gated and cannot fire on this workload.
* The word it becomes is a constant, `0x7d084378`
  ([`WB_R8IDIOM_FINDINGS.md`](WB_R8IDIOM_FINDINGS.md) §4), and adopting that
  constant needs a `DISCLOSURE.md` row naming `0x10c0e1a1`.
* **Do not name it.** If the port needs an identifier, it needs one that says
  in its own spelling that it is c2's `0x2e4` and not a recovered name.
