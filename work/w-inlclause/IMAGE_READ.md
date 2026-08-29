# `w-inlclause` — the image read

Image `compilers/X360/16.00.11886.00/c2.dll`,
`sha256 c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`.
Listings from the **independent** `objdump -d -M intel` disassembly
(`docs/whitebox/C2_MAP_METHOD.md`), never the Ghidra database the clause
addresses were transcribed out of. Prereg `work/w-inlclause/PREREG.md` §3
requires this: a citation I only relayed is not a read.

**Provenance marks**: `[R]` read off the listing · `[I]` inferred from what was
read, and labelled so nobody promotes it later.

---

## 0. What was already recorded, and is RE-DERIVED here rather than found

Registered up front because the honest half of this section is the larger half.
`work/w-clausefix/REPAIRS.md` §"14 rows, all verified" and §C10–§C17 already
carry an address-cited read of **thirteen** of the fifteen `absent` clauses,
including the two things I set out believing were open:

* **C10's `bypasses`** — that `and eax,0x2000` at `0x10b60a28` reaches
  `jne 0x10b60a3c` and then `cmp eax,ebx` / `jne 0x10b60a59` = ACCEPT, *before*
  C16, C17 and the POGO branch. REPAIRS.md §C10 states this, at these
  addresses. **Re-derived, not found.**
* **C11/C12/C13's shared `addr`** — that all three cite the block head
  `0x10b5c06b` (`mov eax,[ecx+0x20]`) and that the clause-pinning addresses are
  `0x10b5c06e`/`0x10b5c080`/`0x10b5c08f`/`0x10b5c093`, `0x10b5c078`/`0x10b5c087`
  and `0x10b5c09a`–`0x10b5c0a0`. REPAIRS.md's table records all nine, and
  **deliberately did not move the rows**. So this is a documented state, not a
  defect, and this lane does **not** repair those three `addr` cells. Had I not
  checked, I would have re-litigated another lane's recorded decision and
  called it a finding.

I re-derived all of it against the listing anyway, because the prereg requires
it before a row is placed. Every claim below marked **NEW** was checked against
the frozen corpus with `work/w-inlclause/read_scan.py`'s method before being
called new.

---

## 1. The accept/decline polarity, which every clause in `FUN_10b60930` depends on `[R]`

Needed first: three of the fifteen rows are decline arms, and a decline arm read
with the polarity backwards says the opposite of what it says.

```
10b609f3:  33 c0        xor    eax,eax        <- 0 : the DECLINE sink
10b609f5:  eb 65        jmp    0x10b60a5c        (epilogue)
...
10b60a59:  33 c0        xor    eax,eax
10b60a5b:  40           inc    eax            <- 1 : the ACCEPT sink
10b60a5c:  5f           pop    edi
```

Every `jg 0x10b609f3` / `ja 0x10b609f3` in the function is therefore a decline,
and C14 / C15 / C16 / C17 all target it. `FUN_10b60930` returns **1 = inline**.

**And the whole chain is entered only when `DAT_10c3de20 == 0`** (`[R]`,
`0x10b60975`–`0x10b6097c`: `mov eax,ds:0x10c3de20` / `cmp eax,ebx` /
`je 0x10b609f7`, `ebx = 0`). At `DAT_10c3de20 == 1` the function jumps straight
to `0x10b60a59` and **accepts unconditionally** (`0x10b6097e`–`0x10b60981`).
`P_INLINE.md` §6.8.4 reads `DAT_10c3de20` as the effective POGO mode; this is
where it decides whether the depth/maxlevel/budget chain runs at all.

---

## 2. **NEW** — an arm of the decline chain that none of the 24 clauses names `[R]`

```
10b60a0b:  a1 0c f5 c3 10   mov    eax,ds:0x10c3f50c     <- the base
10b60a10:  8b 55 0c         mov    edx,DWORD PTR [ebp+0xc]   <- maxlevel
10b60a13:  3b c3            cmp    eax,ebx               <- base == 0 ?
10b60a15:  74 0e            je     0x10b60a25            <- yes: skip BOTH arms
10b60a17:  8b 4d 08         mov    ecx,DWORD PTR [ebp+0x8]   <- level
10b60a1a:  2b c8            sub    ecx,eax               <- level - base
10b60a1c:  83 f9 10         cmp    ecx,0x10              <- C14
10b60a1f:  7f d2            jg     0x10b609f3            <- decline
10b60a21:  3b ca            cmp    ecx,edx               <- **NOT ANY CLAUSE**
10b60a23:  7f ce            jg     0x10b609f3            <- decline
```

> **`0x10b60a21`–`0x10b60a23` declines when `level − base > maxlevel`, and no row
> of the 24-clause table covers it.** C14 is the `0x10` arm and C15 is the
> *absolute* maxlevel arm at `0x10b60a2f`–`0x10b60a3a`. This is a **third**
> arm: maxlevel applied to the **relative** depth, and — unlike C15 — it is
> **not** guarded by `maxlevel != 0xff` and **not** bypassed by
> `__forceinline`, because both of those sit downstream at `0x10b60a25`.

Novelty checked, not assumed: `0x10b60a21` and `0x10b60a23` appear in the frozen
corpus **only** inside REPAIRS.md's C14 context window, as two unannotated
listing lines below the `>>`-marked ones. No prose names them and no clause
covers them.

**Practical reach `[I]`**: at c2's default `maxlevel = 0xff` the arm needs
`level − base > 255`, so it is dead wherever C14's `> 16` already binds. It
becomes live only when `maxlevel` is set **below 16** by `#pragma inline_depth`
— i.e. exactly where C15 is live. It is a real clause on a nearly-dead path,
which is why the port models it and refuses nothing new for it (§6).

---

## 3. **NEW** — C6's flag word decoded: five bits, three counters, and a `0x2000` term `[R]`

C6's clause reads *"site collector: EH-region nesting + conditional/EH flag into
bit 1"*, and `WB_INLINE_FINDINGS` §1 adds *"tracks EH-region nesting through
opcodes `0x2ee/0x2f0/0x2f1/0x2f4/0x2f6/0x2ff/0x300`"*. Both are correct and
both are considerably coarser than the image.

### 3.1 The opcode dispatch is a 19-entry dense switch, and §1's list is short by one

```
10b603e4:  3c 15                    cmp    al,0x15
10b603e6:  0f 85 c5 00 00 00        jne    0x10b604b1
10b603ec:  8b 46 04                 mov    eax,DWORD PTR [esi+0x4]
10b603ef:  8d 88 12 fd ff ff        lea    ecx,[eax-0x2ee]
10b603f5:  83 f9 12                 cmp    ecx,0x12          <- 0x12 = 18, so 19 values
10b603f8:  0f 87 b3 00 00 00        ja     0x10b604b1
10b603fe:  0f b6 89 22 05 b6 10     movzx  ecx,BYTE PTR [ecx+0x10b60522]   <- index table
10b60405:  ff 24 8d 0e 05 b6 10     jmp    DWORD PTR [ecx*4+0x10b6050e]    <- jump table
```

Both tables are in the raw image and quotable. Decoded by
`work/w-inlclause/jumptable.py` (std only, reads `c2.dll` directly):

```
index bytes at 0x10b60522, for opcodes 0x2ee..0x300:
  [0, 4, 1, 2, 4, 4, 1, 4, 2, 4, 4, 4, 4, 4, 4, 4, 0, 3, 2]
jump table at 0x10b6050e:
  arm 0 -> 0x10b6040c   arm 1 -> 0x10b60425   arm 2 -> 0x10b60437
  arm 3 -> 0x10b60414   arm 4 -> 0x10b604b1  (the default: no-op)
```

| arm | body | opcodes |
|---|---|---|
| 0 | `inc [ebp-0x8]` | `0x2ee`, **`0x2fe`** |
| 1 | `inc [ebp-0xc]` · `dec [ebp-0x8]` · if opcode `== 0x2f0`, `inc [ebp-0x10]` | `0x2f0`, `0x2f4` |
| 2 | `dec [ebp-0xc]` · if opcode `== 0x2f1`, `dec [ebp-0x10]` | `0x2f1`, `0x2f6`, `0x300` |
| 3 | the `[esi+0x34]` / `[ecx+0x20]` identity guard, then falls into arm 1 | `0x2ff` |
| 4 | nothing | the other eleven |

> **`WB_INLINE_FINDINGS` §1 lists seven opcodes. There are EIGHT with a
> non-default arm — `0x2fe` is missing.** It shares arm 0 with `0x2ee`, so a
> reader working from the seven-opcode list has one of the two region-openers
> and does not know it. `0x2fe` appears **nowhere** in the frozen corpus.
>
> And it is not seven-versus-eight so much as **three counters versus "nesting"**:
> `[ebp-0x8]`, `[ebp-0xc]` and `[ebp-0x10]` are three independent depths, and
> `0x2f0`/`0x2f1` are secondary tests *inside* arms 1 and 2 rather than
> top-level cases. §1's flat list reads as one counter over seven opcodes.

### 3.2 `[site+0x1c]` is a five-bit flag word, and bit 1 has a `__forceinline` term

The record's flag word is assembled at `0x10b602bc`–`0x10b60300`:

```
10b602be:  cmp DWORD PTR [ebp-0x10],ecx  / setne cl        <- counter 3
10b602c9:  and edi,0x1 / add edi,edi                        <- an incoming bit, to b1 of ecx
10b602d5:  or ecx,edi / add ecx,ecx
10b602d9:  cmp DWORD PTR [ebp-0x30],edx  / setne dl / or ecx,edx
10b602e6:  shl ecx,0x2
10b602e9:  cmp DWORD PTR [ebp-0xc],edx   / setne dl / or ecx,edx   <- counter 2
10b602f4:  mov edx,DWORD PTR [eax+0x1c] / and edx,0xffffffe2       <- keep b1, b5+
10b602fa:  or ecx,edx
10b60300:  mov DWORD PTR [eax+0x1c],ecx
```

so, at bits 0–4 of `[site+0x1c]`:

| bit | source |
|---|---|
| 0 | `[ebp-0xc] != 0` — EH counter 2 |
| 1 | **preserved by the `0xffffffe2` mask here**, and written separately below |
| 2 | `[ebp-0x30] != 0` |
| 3 | `[ebp-0x10] != 0` — EH counter 3 |
| 4 | the incoming `edi & 1` |

Bit 1 is then written by the idiom at `0x10b60339`–`0x10b60347`
(`xor edx,edx` / `add edx,edx` / `xor edx,ecx` / `and edx,0x2` / `xor edx,ecx` /
`mov [eax+0x1c],edx` — set-bit-1-to-`edx`), and the value of `edx` is chosen by
**three** tests:

```
10b602fc:  cmp DWORD PTR [ebp-0x8],0x0          <- EH counter 1 (arm 0's)
10b60300:  (the store above)
10b60303:  je  0x10b60339                       <- counter 1 == 0  -> bit1 := 0
10b60305:  mov edx,DWORD PTR [ebp-0x14]
10b60308:  test BYTE PTR [edx+0x94],0x8
10b6030f:  je  0x10b60317
10b60311:  cmp DWORD PTR [ebp+0x8],0x0
10b60315:  je  0x10b60339                       ->  bit1 := 0
10b60317:  test DWORD PTR [ebx+0x4c],0x2000     <- __forceinline
10b6031e:  jne 0x10b60339                       ->  bit1 := 0
10b60320:  xor edx,edx / inc edx                ->  bit1 := 1
```

> **So bit 1 is `inside an open EH region AND the callee is NOT
> `__forceinline`` (and two further conditions).** This is a **third** site at
> which `[sym+0x4c] & 0x2000` is tested in the inliner — `0x10b60317` here,
> `0x10b60a28` (C10), and `0x10b625a6`/`0x10b6240f` (the charge exemptions
> `w-inlbudget` read). The candidacy function's near-miss at `0x10b5fcc1` tests
> `0x2080`, a **different** mask, which `WB_INLINE_FINDINGS` §2.1's amendment
> already flags. `__forceinline` is not one clause in this subsystem; it is at
> least four tests in four functions, and C10 names one of them.

## 4. **NEW** — C5's instruction-kind test, at an address `[R]`

```
10b60208:  8a 46 08             mov    al,BYTE PTR [esi+0x8]
10b6020b:  3c 0f                cmp    al,0xf
10b6020d:  0f 85 b7 01 00 00    jne    0x10b603ca
```

C5's clause — *"instruction kind `0x0f` is a call site"* — is correct, the field
is `BYTE [instr+0x8]`, and the address is `0x10b6020b`. The corpus states the
clause and cites only the function's **entry** (`0x10b600e6`, `asm` `push ebp`),
which REPAIRS.md's own table marks as *"pins the address and not the clause"*.

---

## 5. C1's gate, re-derived `[R]`

```
10b62675:  55                   push   ebp
10b62676:  33 ed                xor    ebp,ebp          <- ebp := 0, the comparand
10b6267b:  39 2d c4 0e c4 10    cmp    DWORD PTR ds:0x10c40ec4,ebp
10b62681:  75 3e                jne    0x10b626c1       <- non-zero: do the work
```

Agrees with REPAIRS.md's C1 row exactly. **What is still unread is the other
end**: `DAT_10c40ec4` has no enumerated writer anywhere in the corpus, so
nothing says which switch turns the inline pass off. That is C1's blocker and it
is a ten-minute grep for whoever wants the row.

---

## 6. The rows this lane's own read does **not** move

C7, C9, C14, C15, C16, C17, C18 were each re-derived at their repaired
addresses against the listing (`0x10b5e4d1`/`0x10b5e4d7`, `0x10b5fc7e`/
`0x10b5fc84`, `0x10b60a1c`/`0x10b60a1f`, `0x10b60a2f`/`0x10b60a37`,
`0x10b60a63`/`0x10b60a6d`, `0x10b60a73`/`0x10b60a78`, `0x10b625b6`/
`0x10b625b9`) and every one decodes as the corpus records it. **Seven for
seven**, and the value of saying so is that the `read` column's `R1`/`R2`
verdicts below rest on a check rather than on a citation.
