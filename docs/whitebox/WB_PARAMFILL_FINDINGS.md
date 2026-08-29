# `WB_PARAMFILL` — GATE 1 of c2's inline-parameter fill, read

**Lane `w-paramfill`, 2026-08-29.** `docs/ADOPTION_BRIEF_2026-08-29.md` §L3,
board **#3802**–**#3807**. Characterization lane, `docs/rungs/README.md` kind 3.

Prereg: [`../../work/w-paramfill/PREREG.md`](../../work/w-paramfill/PREREG.md),
committed at `959281309` **before the image was opened**.
**Predicted reach 0, delivered 0** — zero `crates/` bytes, no `DISCLOSURE.md`
row, no `scripts/gate.sh` row (`#3691`), no clause row added, removed or
renumbered.

Evidence tiers, per `P_INLINE.md`'s convention: **`[R]`** read from
disassembly · **`[O]`** confirmed against a real obj or a real toolchain
invocation with the witness named · **`[I]`** interpretive.

Instrument: [`scripts/dump_paramfill.py`](scripts/dump_paramfill.py).
Outputs: [`../../work/w-paramfill/paramfill.out`](../../work/w-paramfill/paramfill.out) ·
[`cl_argv_flgate.out`](../../work/w-paramfill/cl_argv_flgate.out) ·
[`defaults_rederived.out`](../../work/w-paramfill/defaults_rederived.out) ·
[`clause_addr_recheck.out`](../../work/w-paramfill/clause_addr_recheck.out).
Page amendment: `ref/P_INLINE.md` §6.9, plus §6.1's re-sync and §6.6.3's
correction block.

---

## 0. The headline, and the four things it refutes

> **GATE 1 is `0` on every compilation this project runs.** `DAT_10c462c4` is
> a **one-way latch** — BSS-zero at load, two writers, both storing `1`, no
> writer of `0` anywhere in the image — and the only one that can fire on the
> path that reaches the fill is guarded by **`-Fl<file>`**, which **0 of 27
> `cl /Bd` mode rows** pass. `[R]` + `[O]`
>
> **And it does NOT follow that the live record stays zero.** `P_INLINE`
> §6.8.2 calls `FUN_10b5e4cc` *"the whole parameter initialisation"*; there is
> a **second copier**, `FUN_10b5b86d` at `0x10b5b88a`, which is **not** behind
> GATE 1 and runs **once per code-generated function**. Table A reaches
> `0x10c3f510` anyway. `[R]`
>
> **So what GATE 1 actually costs here is one thing: the module-size trim
> `FUN_10b5b9de`.** That refutes `WB_INLSWITCH_FINDINGS.md` §9 item 2 —
> `-inlT#` is **exactly 104** on this workload, not the range 80–136, and
> `-inlfcsw#` exactly 32. `[R]` + `[O]`
>
> **And one published default is wrong.** Table A stores `+0x40` (`-inlfcsa#`)
> **twice** in a straight-line sweep, `20` then `5`; the first wins. §3's
> `defA = 5` is **false**, and its "13 fields differ / 8 switch-fed" is **14
> and 9**. `[R]`

Refuted, in the order the claims appear:

| what | said | is |
|---|---|---|
| `P_INLINE` §6.8.2 | `FUN_10b5e4cc` is *"also the whole parameter initialisation"* | **there are two copiers**; the operative one is `FUN_10b5b86d`, ungated, per function |
| `WB_INLSWITCH_FINDINGS` §3.2 | *"the second gate is the whole story"* | the **first** gate is decided first and is 0 — GATE 2 at `0x10b5e50f` is never evaluated on that path. GATE 2 still decides the *live* copy, at `0x10b5b86d` |
| `WB_INLSWITCH_FINDINGS` §9 item 2 | `-inlT#`'s effective default is a **range, 80–136** | **104.** The six-band trim is behind GATE 1 |
| `WB_INLSWITCH_FINDINGS` §3 | `-inlfcsa#` `defA = 5`; A and B differ on **13** of 46 fields, **8** switch-fed | **20**; **14** and **9** |
| `ADOPTION_BRIEF_2026-08-29` §L3 / `BOARD.md` ledger | *"bounds every statement in **`P_INLINE` §3**"* | `P_INLINE` §3 is GRID-I's 264 obj-measured cells and is **downstream of nothing here**. The §3 meant is `WB_INLSWITCH_FINDINGS.md` §3, which is what `w-inlswitch`'s rung actually wrote |
| this lane's own **P1** | the gate is **taken**, and §3 survives | **REFUTED** — the gate is 0 |
| this lane's own **P3** | **≥ 3** writers | **REFUTED — exactly 2**, by three instruments including a decode-independent byte scan |
| this lane's own **P4** (specific half) | the gate is the **`-Og`** optimisations-on flag | **REFUTED** — it is `-Fl#`, kind `0x2601`. P4's weak half **held**: 4 of 112 reads are in the inliner band |

**Four of the seven registered halves were refuted by this lane's own
measurements** (P1, P3, P4(a), P6; P2, P4(b) and P5 held), and the one that
matters most — P1 — was wrong in *both* directions inside one session: first "the gate is open" (wrong), then "so the record is zero"
(wrong for a different reason). §7 records the near-miss, because the second
error is the one that would have been published.

---

## 1. Controls — watched before any verdict here was written (`#3336`)

`dump_paramfill.py --controls`, output in
[`paramfill.out`](../../work/w-paramfill/paramfill.out).

| control | population | required | observed |
|---|---|---|---|
| **C1 GREEN** — the enumerator recovers `DAT_10c46318`'s independently-established set (`P_INLINE` §6.6.1) | 424,232 decoded instruction starts | writers `0x10b5e4d7` `0x10b5e4e8`, reader `0x10b5fc8a` | exactly that, and nothing else — **GREEN** |
| **C2 RED** — planted `0xdeadbe00` | same | **0** refs | 0 — **no false positives** |
| **C3 RED** — the byte scan must find nothing for a pattern that cannot occur and something for one that must | 1,232,384 `.text` bytes | `de ad be 00` → 0; `c4 62 c4 10` → ≥ the listing's count | **0** and **114 vs 114** — the scan is neither blind nor vacuous |
| **C4 CROSS** — Ghidra `xrefs.tsv`, control-flow-driven | 146,818 references | agreement to the address | **114 = 114**, `L\G` empty, `G\L` empty |

**`424,232`** is `#3784`'s corrected boundary count, reproduced here by an
independently written matcher (three tab-separated fields required, so
objdump's byte-continuation lines are excluded). Its agreement with
`check_table.py` is a cross-check, not a shared implementation.

> **A control this lane could not turn green, and therefore quotes nothing
> from.** §8.

---

## 2. The word `[R]`

`DAT_10c462c4` lies above the raw `.data` end `0x10c3cc00` → **BSS, zero at
load** (`read_dword` returns no raw bytes).

```
  L  objdump linear: 114 refs  (  2 WRITE, 112 READ)   of 424232 decoded instruction starts
  G  Ghidra xrefs  : 114 refs  (  2 WRITE, 0 READ_WRITE, 112 READ)
  B  raw byte scan : 114 occurrences of c4 62 c4 10  of 1232384 .text bytes scanned
  B-hits with no decoded instruction start within 7 bytes: 0 of 114
```

**Two writers, and both store `1`:**

```
10bec3dd:  33 c0              xor  eax,eax
10bec3df:  40                 inc  eax
10bec3e4:  a3 c4 62 c4 10     mov  ds:0x10c462c4,eax      ; FUN_10bec3d3, UNCONDITIONAL

10b848f4:  33 f6              xor  esi,esi                ; esi = 0 for all of FUN_10b848dc
10b84bb2:  39 35 a0 5f c4 10  cmp  DWORD PTR ds:0x10c45fa0,esi
10b84bb8:  74 06              je   0x10b84bc0
10b84bba:  89 3d c4 62 c4 10  mov  DWORD PTR ds:0x10c462c4,edi   ; edi = 1
```

**No instruction anywhere in the image stores `0` to it.** So it is a one-way
latch and its whole state space is `{0 at load, 1 once set}`.

**112 reads in 78 distinct owner functions; 4 of the 112 are inside the
inliner band** `0x10b5b86d`–`0x10b62b00` (`0x10b5e4f7` GATE 1, `0x10b5fe35`
and `0x10b6005d` inside the POGO cost model, `0x10b62583` in `FUN_10b6242a`).
109 of the 112 are `cmp <word>,<zero>`. **`0x10c462c4` is a global compiler
condition that the inliner consults; it is not an inliner flag** — which is
this lane's P4 weak half, held.

> **Instrument caveat, found here and worth carrying.** `functions.tsv`'s
> `addr .. addr+size` containment is **not** Ghidra's notion of function
> membership. Six of the 112 reads have no owner under that test; Ghidra's own
> `from_func` assigns **five** of them to `FUN_10b7f1ff`, whose body is
> non-contiguous (it ends `jmp 0x10b7f022`, a block 477 bytes *below* its
> entry). A census that reports "6 orphans" is reporting a containment test,
> not the image. Only `0x10b28184` is unowned by both.

---

## 3. What sets it: `-Fl#`, and kind `0x26` read `[R]`

`0x10c45fa0` is `-Fl#`'s value word — descriptor `0x10c46ec0`, kind
**`0x2601`**, the **only** row of that kind in c2's **148**-row option table
(`optmap.py` re-run unmodified, byte-identical to `w-inlfit`'s committed
output). `0x10c45fa0` has **no direct writer**: its only two references are the
`cmp` at `0x10b84bb4` and the descriptor plant at `0x10c29a40`.

Kind `0x26` is one of the four `FUN_10c1f572` arms `w-inlswitch` §9 item 6
named as unread. Read here:

```
10c1f6a9:  sub  esi,0x24        ; esi = BYTE[record+9], the kind
10c1f6ac:  je   0x10c1f734      ;   0x24 numeric
10c1f6b2:  dec  esi / dec esi
10c1f6b4:  je   0x10c1f703      ;   0x26  <-- -Fl#
...
10c1f703:  mov  esi,DWORD PTR [edi+0x4]        ; value_ptr = 0x10c45fa0
10c1f706:  cmp  DWORD PTR [esi],0xc8           ; 200 entries
10c1f70c:  jge  0x10c1f72a                     ; -> diagnostic 0x5e
10c1f715:  call 0x10c20107                     ; strdup the argument
10c1f720:  mov  ecx,DWORD PTR [esi]
10c1f722:  mov  DWORD PTR [esi+ecx*4+0x4],eax
10c1f726:  inc  DWORD PTR [esi]
```

**`-Fl` is a repeatable string-list option, capacity 200.** `0x10c45fa0` is the
count; the array is `0x10c45fa4`–`0x10c462c0`; and `0x10c462c4` is the dword
**immediately past its end**. That adjacency is not an overflow — index 199
writes `0x10c462c0` and the `jge` refuses at 200 — but it is worth stating,
because a reader who sees a flag one dword past a bounded array will wonder.

**The array is write-only inside `c2.dll`.** Decode-independent byte scan over
the whole `0x328`-byte block: **2** raw hits (both on the count) and **0** on
any of the 200 element addresses, over 1,232,384 `.text` bytes. c2 accumulates
the list and never reads it back; the only behavioural effect of `-Fl` is
"at least one was given".

---

## 4. The chain: the unconditional writer never reaches the fill `[R]`

Every route to `FUN_10b5e4cc` is one spine (`--chain`):

```
_InvokeCompilerPass@12   (0x10bebffd, export)  ─┐
_InvokeCompilerPassW@16  (0x10bec133, export)  ─┼─> FUN_10b7f3e7 -> FUN_10b7f3b6
FUN_10bec3d3             (0x10bec3d3)          ─┘        |
   ^-- DllGetObjHandler (0x10bec40a), FUN_10b73634        |
                                                          v
                            FUN_10b7f369 -> FUN_10b7f1ff -> FUN_10b5e4cc
```

and `FUN_10b7f3b6` is three tests deep:

```
10b7f3b6:  call 0x10b7e4f0        ; -> FUN_10b848dc, the option walk (writer #2)
10b7f3c0:  cmp  ds:0x10c2eb38,0x0
10b7f3c7:  jne  0x10b7f3e1        ; skip the whole init
10b7f3c9:  cmp  ds:0x10c46308,0x0 ; -ltcg
10b7f3d0:  jne  0x10b7f3d7
10b7f3d2:  call 0x10b7f369        ; the only route to the fill
```

`DAT_10c2eb38` is in raw `.data`, **load-time value 0**, and its **only** writer
is `0x10bec3e9` — the instruction five bytes after the unconditional gate
store. **So `FUN_10bec3d3` sets the gate to 1 and then takes the branch that
skips the fill.** The two writers are, on this axis, mutually exclusive with
the fill in exactly one direction.

`DAT_10c46308` is `-ltcg` (descriptor `0x10c46ecc`, boolean), BSS-zero, no
direct writer.

> **Therefore, on the path that runs the fill, GATE 1 is 1 iff `-Fl<file>` is
> on c2's argv.** Everything below rests on that sentence and nothing else.

---

## 5. `-Fl` is passed by no mode `[O]`

`cl /Bd` prints each pass's own command line. **27 mode rows** — every row of
`scripts/lanes.txt` plus `/Os` `/Ot` `/Ox /Ob0` `/GL` `/GL /O2` `/O2 /GL /Gy`
`/FAsc` `/O2 /FAsc` `/Ox /GL /EHsc` — against
`compilers/X360/16.00.11886.00` under wibo, on a two-function TU with one
inlinable callee. Witness
[`cl_argv_flgate.out`](../../work/w-paramfill/cl_argv_flgate.out); the runner
is [`argv/modes.zsh`](../../work/w-paramfill/argv/modes.zsh) and it uses zsh's
explicit `${=m}` word-split, carrying `w-inlswitch` §8.1's defect in its own
header.

| token | hits | of |
|---|---:|---:|
| `-Fl` | **0** | 27 mode rows |
| `-optref` | **0** | 27 |
| `-ltcg` | **4** | 27 — every `/GL` row, and only those |
| `-FA` / `-Fa` | 2 | 27 — the two `/FAsc` rows |

Two incidental readings, both relevant:

* **`/FAsc` passes `-FAasc -Fa <file>`, not `-Fl`.** The listing seam
  (`c2rs listing`, board #132) does **not** disturb this gate. That was a live
  worry — a narration seam that silently switches the compiler into another
  mode would invalidate every listing-derived finding in this repo — and it is
  now measured, not assumed.
* **`/GL` does reach c2, as `-ltcg`.** `0x10b7f3c9` turns that into "skip
  `FUN_10b7f369`", so at `/GL` the fill is not called at all. A *second*,
  independent reason the gated code is dead — and, unlike GATE 1, one that
  cl can actually produce.

**`DAT_10c462c4 = 0` on every compilation this project runs.** `[R]` + `[O]`

---

## 6. What GATE 1 costs — and what it does NOT `[R]`

```
10b5e4cc:  mov ecx,ds:0x10c2ea98            ; k
10b5e4d2:  ...  DAT_10c46318 = (k<=6) ? 0x10<<k : 1000     ; BEFORE the gate
10b5e4ed:  call 0x10b5ba71                  ; fill table B  ) BEFORE
10b5e4f2:  call 0x10b5bc6e                  ; fill table A  ) the gate
10b5e4f7:  cmp DWORD PTR ds:0x10c462c4,0x0
10b5e4fe:  je  0x10b5e52e                   ; -> ret 0x8
10b5e50a:  call 0x10b5b9de                  ; module-size trim of table A   <-- DEAD
10b5e50f:  cmp ds:0x10c6f1c8,0x0 ...
10b5e52a:  rep movs                          ; 46 dwords -> 0x10c3f510      <-- DEAD
```

Exactly **two** things are behind the gate.

### 6.1 The module-size trim is dead — and that refutes a published range

`FUN_10b5b9de` adjusts table A's `+0x04` (`-inlT#`) and `+0x08` (`-inlfcsw#`)
in six bands by module size (`w-inlswitch` §9 item 2). It is called **only** at
`0x10b5e50a`, behind the gate.

> **`WB_INLSWITCH_FINDINGS.md` §9 item 2's *"`-inlT#`'s effective default is a
> range, 80–136, not the 104 the sweep installs"* is FALSE on this workload.**
> It is exactly **104**, and `-inlfcsw#` exactly **32**. The range is real code
> and it never runs here.

### 6.2 The gated copy is dead — and it is redundant

§7.

### 6.3 What is NOT gated

`DAT_10c46318 = 0x10 << k` is computed at `0x10b5e4d2`, and both fillers run at
`0x10b5e4ed`/`0x10b5e4f2` — all three **before** the `cmp`. So:

* **`P_INLINE` §6.6.1's ceiling and `#3732`/`#3734` are untouched** by this
  lane in either direction. `128` is neither adopted nor restated as settled.
* **`WB_INLSWITCH_FINDINGS.md` §3's `defA`/`defB` columns need no condition
  from GATE 1** — the sweeps that install them are on the near side. This was
  the lane's **P2**, registered at 0.8, and it **held**.

### 6.4 `[I]` What the gate means

Four independent pointers, none contradicted:

1. its setter is a 200-entry **file list**;
2. its `== 0` arm in the driver (`0x10b7f026`, `0x10b7f2e0`) is the code that
   derives the per-module IL file names `.gl` / `.sy` / `.ex` / `.in` from a
   single `-il` base — extension pairs (ASCII + UTF-16) at `0x10b1339c` "gl",
   `0x10b13368` "sy", `0x10b13374` "ex", `0x10b13380` "in", which are exactly
   the five-file bundle `c2rs capture` keeps;
3. its two in-cost-model readers pair it with `-optref` — `/OPT:REF`, a
   **linker** option — at `0x10b5fe3e` and `0x10b60066`
   (`if (gate && !optref) skip`);
4. its descriptor neighbours in the option table are `-ltcg` and `-optref`.

Reading: **whole-program / link-time back-end mode**. Marked `[I]` and left
there: no obj this project can build exhibits it, and expanding `-Fl` to a
word would be `[I]` dressed as `[R]`.

---

## 7. THE SECOND COPIER — and the absence claim this lane nearly published `[R]`

`P_INLINE` §6.8.2 calls `FUN_10b5e4cc` *"also the whole parameter
initialisation"*. **It is not.**

```
FUN_10b5b86d   (0x10b5b86d, 34 B, exactly one caller)
10b5b86d:  cmp  DWORD PTR ds:0x10c6f1c8,0x0    ; GATE 2 only
10b5b876:  mov  esi,0x10c45ed0                 ; table B
10b5b87b:  jne  0x10b5b882
10b5b87d:  mov  esi,0x10c45e18                 ; table A
10b5b885:  mov  edi,0x10c3f510
10b5b88a:  rep movs DWORD PTR es:[edi],DWORD PTR ds:[esi]
```

Reached once per code-generated function, on every path:

| site | what |
|---|---|
| `0x10b7f15f`–`0x10b7f198` | the driver's function-list walk over `0x10c4630c`; a function with `[fn+0x4c] & 0x20` and `!( & 0x2)` goes to `0x10b7f199` |
| `0x10b7f1b1` | `call FUN_10b7ef55` — the per-function compile |
| `0x10b7ef5d` | `call FUN_10b7e113` — **third instruction, unconditional** |
| `0x10b7e1b0` | `call FUN_10b5b86d` — at the tail, past both of that function's two branches |

and `0x10b7f0e4` (`call FUN_10b5e4cc`) is **earlier in the same block** than
`0x10b7f1b1`, so the fillers have already installed table A's defaults by the
time the ungated copy runs.

> **So the live 46-dword record at `0x10c3f510` IS populated with table A on
> every compilation** (`DAT_10c6f1c8 = 0`, `P_INLINE` §6.8.6), by
> `0x10b5b88a`. `WB_INLSWITCH_FINDINGS.md` §3's per-switch `live` values are
> operative — **via an instruction §3.2 does not name.** §3.2's conclusion
> survives; its mechanism does not.

### 7.1 The near-miss, in the terms `#3505` is counted in

A per-field reference census over all 46 live fields returns:

```
  per-field xref census : 60 refs over the 46 fields, 0 of them WRITEs
```

**That is correct and it is useless.** A `rep movsd` writes its destination
through `EDI`; the destination address appears once, as an immediate, at
`0x10b5b885` — 5 bytes and one instruction away from the store. Read as an
absence, "0 writers over 46 fields" says *nothing else writes the record*, and
the confident wrong headline follows in one step: **"GATE 1 is 0, therefore
the live record stays all-zero, therefore the 24 switches are inert for a
second reason."** That sentence was drafted.

**What refuted it was an index of the same fact already in this repo** —
`P_INLINE` §6.1's **C23** row, `parameter-table selection … 0x10b5b86d`,
checked against the read instead of assumed consistent with it. C23 has named
the second copier since `w-inlmetric` froze the table, and
`dump_inlswitch.py`'s own docstring says *"FUN_10b5b86d then `rep movsd`s 46
dwords"* — two standing records, neither of which reached §6.8.2's prose.

`#3505` is now **six for six** on *"no writer / no reader / no cell exists"*
turning out to be a claim about an instrument's index. This one is the sharpest
of the six, because the index was not merely incomplete — **it was right**, and
the defect was in reading a correct zero as an absence.
`dump_paramfill.py --copiers` exists so the next reader gets the copiers and
not the census: it prints the field census, labels it invisible-by-
construction, and then finds the copiers by the immediate load of the base
address, cross-checked by byte scan (**5 hits, 0 unexplained**).

---

## 8. A control this lane could NOT turn green, and quotes nothing from `[O]`

The decisive obj measurement for §6 would be: run standalone c2 twice on one
captured bundle, once with `-Fl` appended, and byte-compare. It was attempted
and **its baseline failed**.

`c2rs replay fixtures/cpp/il_intra_tu_call.cpp` succeeds
(`ref=878B replay=878B normalized_identical=true`) and `c2rs selftest` is all
`PASS`, so the toolchain is present and the replay path works. A hand-built
invocation of the same stub — same `wibo`, same `c2host.exe`, same
`compilers/X360/16.00.11886.00/c2.dll`, the bundle from
`c2rs capture --keep-il`, and the argv template transcribed from
`crates/c2-reference/src/lib.rs:1709` — aborts inside c2 with
`wibo: call reached missing import lstrcatW from kernel32`, at **both** the
`ReferenceC2` argv order and the captured `/Bd` order, from three different
working directories, with and without `TMP`/`TEMP` set.

**The control is RED and no obj-level claim is made from the probe.** The
discrepancy is itself a finding and is filed as not-reached (§10 item 1): a
hand invocation of `c2host` that differs from `Toolchain::build_replay_command`
in no way this lane could find takes a different path inside c2. Until that is
explained, *"the standalone replay is reproducible by hand"* is not a claim
this repo can make.

---

## 9. P5 re-derived, and the one §3 cell that is false `[R]`

### 9.1 The sweeps are straight-line, which is what makes "first store wins" a fact

| filler | instructions | non-guard control transfers | store instructions | distinct fields |
|---|---:|---:|---:|---:|
| `FUN_10b5bc6e` (table A) | 120 | **1** (`call FUN_10b5b88f`) | **34** | **33** |
| `FUN_10b5ba71` (table B) | 119 | **1** (`call FUN_10b5b88f`) | **33** | **33** |

Every other branch in both is a `cmp ds:F,eax / jne` skipping exactly one
store, and `eax` is defined **once** in each (`xor eax,eax` at `0x10b5bc78` /
`0x10b5ba7d`) and never redefined. There is no arm, no loop, no alternative
path.

`FUN_10b5b88f` scatters **37** value words from `0x10c45d80`–`0x10c45e10` —
re-derived, matching `w-inlswitch`. `46 − 33 = 13` fields defaulted by neither
sweep — matching.

### 9.2 `-inlfcsa#` (`+0x40`) is stored twice in table A

```
10b5bcf5:  cmp ds:0x10c45e58,eax   / jne 10b5bd07 / 10b5bcfd:  mov ds:0x10c45e58,0x14   ; 20
10b5be3f:  cmp ds:0x10c45e58,eax   / jne 10b5be51 / 10b5be47:  mov ds:0x10c45e58,0x5    ; 5
```

The first store leaves the field at 20; the second guard therefore cannot pass.
**Table A's `-inlfcsa#` default is `20`, and `0x10b5be47` is dead code.**

Consequences for `WB_INLSWITCH_FINDINGS.md` §3, all mechanical:

| §3 statement | verdict |
|---|---|
| `+0x40` `-inlfcsa#` `defA = 5` | **FALSE** — 20 |
| "Table A vs table B differ on **13** of the 46 fields" | **FALSE** — **14** |
| "— **8** of the 24 switch-fed ones" | **FALSE** — **9** |
| "`\|A\|/\|B\|` spans 2.1× to 30.0×" | **survives** — 20/5 = 4.0× is inside it |
| "and A is the larger in all 13" | **survives as a shape, count wrong** — A is larger in all **14** |
| every other `defA` cell (32 of 33) and every `defB` cell (33 of 33) | **re-derived identical** |

Witness [`defaults_rederived.out`](../../work/w-paramfill/defaults_rederived.out),
which prints both tables field by field with the installing instruction's
address beside each value, and lists dead stores explicitly (**A: 1, B: 0**).

> **The matcher had to be widened twice to get this right, and both misses are
> in the record.** A first pass accepted only `mov ds:F,<imm>` and reported
> **10** default stores for A and **5** for B — half the sweep stores its
> constant through a register (`push 0x20 / pop edx / mov ds:F,edx` is 7 bytes
> against a 10-byte immediate form). A second pass added the register pool but
> not `inc`/`dec`, and reported `-inlniln#` as **0** where it is **1**
> (`xor edx,edx / inc edx`), and missed table B's `+0x44 = -1` entirely because
> it is an **`or ds:F,0xffffffff`**, not a `mov`. **Each wrong number was
> caught by disagreeing with `w-inlswitch`'s published cell**, which is the
> only reason the third pass's *agreement* on 32 of 33 is worth anything: a
> re-derivation that had matched on the first try would have proved less.

---

## 10. Found and not taken

Ranked, sized, with what stopped each.

1. **The hand `c2host` invocation that does not reproduce `c2rs replay`** (§8).
   Highest value here, because it bounds what any future by-hand standalone
   probe can claim. The next step is to make `Toolchain::build_replay_command`
   able to print its argv (or accept extra tokens), which is a `crates/` change
   this lane may not make. **One hour, and it unlocks the obj-level test of
   every gate in `P_INLINE` §6.8–§6.9.**
2. **`FUN_10b7f1ff`'s seven reads of the gate** (`0x10b7f026`, `0x10b7f0e9`,
   `0x10b7f121`, `0x10b7f12e`, `0x10b7f1df`, `0x10b7f20a`, `0x10b7f2e0`) select
   between two whole driver bodies. Only the `.gl`/`.sy`/`.ex`/`.in` name
   derivation was read; `call 0x10b72f0a` / `0x10b72f21` (gate ≠ 0) and
   `0x10b734f7` are unopened. **This is c2's whole-program driver and nobody
   has read it.**
3. **`FUN_10b5b9de`'s six bands are dead here but real** — the trim exists and
   a `-Fl` compilation would take it. Its interaction with `-inlT#`'s reader at
   `0x10b5ff90` is unexamined because the field is POGO-dead anyway.
4. **The other 74 owner functions of the 112 reads.** This lane opened 6.
   Anything that says "c2 behaves differently under LTCG" is in that set.
5. **`FUN_10b73634`**, the second caller of `FUN_10bec3d3`, has no callers of
   its own in `calls.tsv` — a vtable or callback target that was not resolved.
   `DllGetObjHandler`'s own use of `FUN_10bec3d3` was likewise not traced to a
   call site in the listing.
6. **Kinds `0x08`, `0x23`, `0x27`** of `FUN_10c1f572` are still unread; this
   lane closed `0x26` because `-Fl` needed it.

---

## 11. What this lane did not reach

* **Nothing was graded against an obj.** §8's control failed, so every `[O]`
  row here is a toolchain-invocation measurement (`cl /Bd`), never an obj
  comparison. The `[R]` claims about the second copier are reads, and the
  sentence *"the live record holds table A on every compilation"* is `[R]` — it
  has **not** been confirmed at an obj and this page does not claim it has.
* **`-Fl`'s expansion is not named.** §6.4's whole-program reading is `[I]`.
* **The `-Fl` list's consumer, if any, is outside `c2.dll`.** This page
  establishes only that nothing in this image reads the array.
