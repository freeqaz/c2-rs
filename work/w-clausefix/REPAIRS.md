# `w-clausefix` — the ten address repairs, derivation by derivation

Evidence for `work/w-clausefix/PREREG.md` §3. Every repaired address is shown in
its disassembly context, from the independent objdump listing
(`objdump -d -M intel`, PE32 as `pei-i386` at true VAs — 424,232 instruction
starts, `docs/whitebox/C2_MAP_METHOD.md`), in
c2.dll sha256 `c80981c0..a66258`.

`>>` marks the instructions that carry the clause. The **repaired** address is
the first `>>` line named in each row's *repaired* field.

**Ten of 24 rows moved.** Eight were mid-instruction (found by the ALIGN check,
`#3721`); **two — C10 and C15 — were aligned, inside the correct function, and
pointed at a different instruction**, which no alignment check can see. The
other **fourteen** rows were verified correct and are unchanged: C1, C5, C6, C7,
C8, C9, C11, C12, C13, C20, C21, C22, C23, C24.

**Zero rows could not be located.** PREREG §5's not-located path did not fire.

**No `state`, `witness`, `exercised`, `owner`, `clause` or `note` cell was
touched**, and every repaired address stays inside the function its unchanged
`owner` cell already named — so PREREG §5/F3's flag-don't-move path did not fire
either. Split unchanged: `absent 17 · fitted 2 · R-derived 2 · unexercisable 3`
over 24 rows, reachable denominator 21.

---

## C2 — `caller instruction count seeded: DAT_10c3f5cc = (ushort)[fn+0x50]`

    was      0x10b626d8      MID-INSTRUCTION
    repaired 0x10b62703      +43 (0x2b)
    owner    FUN_10b62675  (unchanged; the repair stays inside it)

       10b626f5  mov eax,DWORD PTR [esi]
    >> 10b626f7  movzx eax,WORD PTR [eax+0x50]
       10b626fb  or DWORD PTR ds:0x10c3f5c8,0xffffffff
       10b62702  push ebx
    >> 10b62703  mov ds:0x10c3f5cc,eax
       10b62708  add eax,eax

The clause is an ASSIGNMENT and the repaired address is its STORE. The source load `movzx eax,WORD PTR [eax+0x50]` is at `0x10b626f7`, four instructions earlier; `esi` holds the function record and `[esi]` its symbol. The original `0x10b626d8` was +1 into the `jmp 0x10b626f1` at `0x10b626d7` -- the back-edge of a completely unrelated loop that clears `[ecx+0x40]` over a `0x60`-stride list.

---

## C3 — `growth budget B = clamp(2 x caller_instrs, 1000, 35000)`

    was      0x10b626f4      MID-INSTRUCTION
    repaired 0x10b62708      +20 (0x14)
    owner    FUN_10b62675  (unchanged; the repair stays inside it)

       10b62703  mov ds:0x10c3f5cc,eax
    >> 10b62708  add eax,eax
    >> 10b6270a  mov ebx,0x3e8
       10b6270f  cmp eax,ebx
       10b62711  jle 0x10b62715
       10b62713  mov ebx,eax
    >> 10b62715  mov eax,0x88b8
       10b6271a  cmp ebx,eax
       10b6271c  jl 0x10b62720
       10b6271e  mov ebx,eax
       10b62720  push edi

Both literals are present and the clause is exact: `add eax,eax` doubles the count just stored by C2, `0x3e8` = 1000 is the floor at `0x10b6270a`, `0x88b8` = 35000 is the ceiling at `0x10b62715`, and `ebx` carries B into C4's call. The repaired address is the FIRST instruction of that sequence, per PREREG SS3.1. The original `0x10b626f4` was +1 into the `jne 0x10b626d9` at `0x10b626f3` -- again the unrelated loop, one instruction before the clause's real start.

---

## C4 — `driver entry FUN_10b61ee1(fn, level=1, budget=B, 0, 100000000, 0)`

    was      0x10b6276a      MID-INSTRUCTION
    repaired 0x10b6276e      +4 (0x4)
    owner    FUN_10b62675  (unchanged; the repair stays inside it)

       10b62736  push ebp
    >> 10b62737  push 0x5f5e100
       10b6273c  shr eax,0x1b
       10b6273f  push ebp
       10b62740  and eax,edi
       10b62742  push ebx
       10b62743  mov edx,edi
       10b62745  mov ecx,esi
       10b62747  mov ds:0x10c2e334,eax
       10b6274c  mov DWORD PTR ds:0x10c3f50c,ebp
       10b62752  mov DWORD PTR ds:0x10c3f504,ebp
       10b62758  mov DWORD PTR ds:0x10c4632c,ebp
       10b6275e  mov DWORD PTR ds:0x10c46334,ebp
       10b62764  mov DWORD PTR ds:0x10c46330,0x10c46334
    >> 10b6276e  call 0x10b61ee1
       10b62773  mov ebx,eax

`push 0x5f5e100` at `0x10b62737` is the literal 100,000,000 the clause names; `ebx` is B from C3; `edi` is 1 (`xor edi,edi` / `inc edi` at `0x10b62728`), which is both `level=1` and, as `edx`, the fastcall second argument. The original `0x10b6276a` was +6 into the 10-byte `mov DWORD PTR ds:0x10c46330,0x10c46334` at `0x10b62764`, whose tail objdump prints on a continuation line at `0x10b6276b` -- the exact shape that makes this class of error survive a naive boundary scan.

---

## C10 — `__forceinline: test [sym+0x4c], 0x2000 bypasses every size and budget test`

    was      0x10b609d3      ALIGNED, WRONG INSTRUCTION
    repaired 0x10b60a28      +85 (0x55)
    owner    FUN_10b60930  (unchanged; the repair stays inside it)

    >> 10b60a25  mov eax,DWORD PTR [edi+0x4c]
    >> 10b60a28  and eax,0x2000
       10b60a2d  jne 0x10b60a3c
       10b60a2f  cmp edx,0xff
       10b60a35  je 0x10b60a3c
       10b60a37  cmp DWORD PTR [ebp+0x8],edx
       10b60a3a  jg 0x10b609f3
    >> 10b60a3c  cmp eax,ebx
    >> 10b60a3e  jne 0x10b60a59
       10b60a40  cmp DWORD PTR [ebp-0x4],ebx
       10b60a43  je 0x10b60a63

**This is the row `w-inlfit` filed and did not pursue, and it is not a harder class -- it is the same class with a luckier address.** `mov eax,[edi+0x4c]` / `and eax,0x2000` is the test; the clause's word BYPASSES is the `jne 0x10b60a3c` at `0x10b60a2d` (skips C15's maxlevel test) followed by `cmp eax,ebx` / `jne 0x10b60a59` at `0x10b60a3c`-`0x10b60a3e`, which returns 1 -- ACCEPT -- before the POGO branch, the caller-huge test (C16) and the budget test (C17) are reached. The original `0x10b609d3` is a real instruction (`call 0x10b5e64d`) inside the right function on a diagnostic path, which is why ADDRESS and ALIGN are both green on it and only DECODE can fail.

---

## C14 — `depth cap: 0x10 < level - DAT_10c3f50c => decline (16 levels)`

    was      0x10b609ae      MID-INSTRUCTION
    repaired 0x10b60a1c      +110 (0x6e)
    owner    FUN_10b60930  (unchanged; the repair stays inside it)

    >> 10b60a0b  mov eax,ds:0x10c3f50c
       10b60a10  mov edx,DWORD PTR [ebp+0xc]
       10b60a13  cmp eax,ebx
       10b60a15  je 0x10b60a25
       10b60a17  mov ecx,DWORD PTR [ebp+0x8]
    >> 10b60a1a  sub ecx,eax
    >> 10b60a1c  cmp ecx,0x10
    >> 10b60a1f  jg 0x10b609f3
       10b60a21  cmp ecx,edx
       10b60a23  jg 0x10b609f3

`eax` = `DAT_10c3f50c`, `ecx` = `[ebp+0x8]` = level, `sub ecx,eax`, `cmp ecx,0x10`, `jg 0x10b609f3` -- and `0x10b609f3` is `xor eax,eax` then the epilogue, i.e. DECLINE. The whole cap is skipped when `DAT_10c3f50c` is zero (`je 0x10b60a25` at `0x10b60a15`). The original `0x10b609ae` was +1 into `and eax,0x10` at `0x10b609ad` -- an unrelated bit-merge into `[esi+0x1c]` that happens to carry the same constant `0x10`, which is a plausible way to mis-hit when searching by constant.

---

## C15 — `maxlevel != 0xff && maxlevel < level => decline`

    was      0x10b609bd      ALIGNED, WRONG INSTRUCTION
    repaired 0x10b60a2f      +114 (0x72)
    owner    FUN_10b60930  (unchanged; the repair stays inside it)

       10b60a2d  jne 0x10b60a3c
    >> 10b60a2f  cmp edx,0xff
    >> 10b60a35  je 0x10b60a3c
    >> 10b60a37  cmp DWORD PTR [ebp+0x8],edx
    >> 10b60a3a  jg 0x10b609f3

**Found by this lane. `#3721` does not name it and no alignment check can.** `edx` = `[ebp+0xc]` = maxlevel (loaded at `0x10b60a10`), `[ebp+0x8]` = level. `cmp edx,0xff` / `je 0x10b60a3c` is the `!= 0xff` guard; `cmp [ebp+0x8],edx` / `jg 0x10b609f3` is `level > maxlevel => decline`, which is the clause's `maxlevel < level` written the other way round. The original `0x10b609bd` is `cmp ecx,ebx` -- the test for whether a POGO profile record exists, an entirely different clause (it is C21's guard), aligned and inside FUN_10b60930.

---

## C16 — `caller-huge decline: 35000 < DAT_10c3f5cc`

    was      0x10b609ee      MID-INSTRUCTION
    repaired 0x10b60a63      +117 (0x75)
    owner    FUN_10b60930  (unchanged; the repair stays inside it)

    >> 10b60a63  cmp DWORD PTR ds:0x10c3f5cc,0x88b8
    >> 10b60a6d  jg 0x10b609f3

`cmp DWORD PTR ds:0x10c3f5cc,0x88b8` / `jg 0x10b609f3` -- the global C2 seeds and C19 charges, against 35000 = 0x88b8, declining when it is exceeded. The original `0x10b609ee` was +3 into `call 0x10b9ded7` at `0x10b609eb`, the diagnostic call on the decline path.

---

## C17 — `budget accept/decline: budget < instrs && instrs > 0x28 => DECLINE`

    was      0x10b60a04      MID-INSTRUCTION
    repaired 0x10b60a73      +111 (0x6f)
    owner    FUN_10b60930  (unchanged; the repair stays inside it)

    >> 10b60a6f  movzx eax,WORD PTR [edi+0x50]
    >> 10b60a73  cmp DWORD PTR [ebp+0x10],eax
       10b60a76  jge 0x10b60a81
    >> 10b60a78  cmp eax,0x28
    >> 10b60a7b  ja 0x10b609f3

`movzx eax,WORD PTR [edi+0x50]` is instrs; `cmp DWORD PTR [ebp+0x10],eax` / `jge 0x10b60a81` leaves the decline path only when budget >= instrs; the fall-through `cmp eax,0x28` / `ja 0x10b609f3` is the second conjunct. Both of the clause's conditions are here and the repaired address is the first. The original `0x10b60a04` was +2 into `call 0x10b9c95d` at `0x10b60a02`.

---

## C18 — `the 40-instruction test, SECOND copy: cmp WORD [callee+0x50], 0x28`

    was      0x10b6249b      MID-INSTRUCTION
    repaired 0x10b625b6      +283 (0x11b)
    owner    FUN_10b6242a  (unchanged; the repair stays inside it)

       10b625a6  test DWORD PTR [esi+0x4c],0x2000
       10b625ad  mov edi,DWORD PTR [ebp+0x8]
       10b625b0  jne 0x10b625c7
    >> 10b625b2  movzx eax,WORD PTR [esi+0x50]
    >> 10b625b6  cmp eax,0x28
       10b625b9  jbe 0x10b625bd
    >> 10b625bb  sub DWORD PTR [edi],eax
    >> 10b625bd  movzx eax,WORD PTR [esi+0x50]
    >> 10b625c1  add DWORD PTR ds:0x10c3f5cc,eax

The clause says SECOND COPY and this is it: `0x10b625b6` is the other `cmp ...,0x28` in the image, C17's being the first. It sits under a `test DWORD PTR [esi+0x4c],0x2000` at `0x10b625a6` whose `jne` skips the whole block -- the __forceinline charge exemption. The original `0x10b6249b` was +1 into `mov ecx,[eax+0x4]` at `0x10b6249a`.

---

## C19 — `the charge: *budget -= WORD[callee+0x50]; DAT_10c3f5cc += same`

    was      0x10b624a2      MID-INSTRUCTION
    repaired 0x10b625bb      +281 (0x119)
    owner    FUN_10b6242a  (unchanged; the repair stays inside it)

       10b625a6  test DWORD PTR [esi+0x4c],0x2000
       10b625ad  mov edi,DWORD PTR [ebp+0x8]
       10b625b0  jne 0x10b625c7
    >> 10b625b2  movzx eax,WORD PTR [esi+0x50]
    >> 10b625b6  cmp eax,0x28
       10b625b9  jbe 0x10b625bd
    >> 10b625bb  sub DWORD PTR [edi],eax
    >> 10b625bd  movzx eax,WORD PTR [esi+0x50]
    >> 10b625c1  add DWORD PTR ds:0x10c3f5cc,eax

`sub DWORD PTR [edi],eax` at `0x10b625bb` (`edi` = `[ebp+0x8]`, the budget out-pointer) and `add DWORD PTR ds:0x10c3f5cc,eax` at `0x10b625c1` -- and the clause's word SAME is exact, because `0x10b625bd` re-loads `movzx eax,WORD PTR [esi+0x50]` rather than reusing the register. Note the asymmetry the clause does not mention and this lane does not add: the subtraction is guarded by `jbe 0x10b625bd` at `0x10b625b9`, so the LOCAL budget is charged only when instrs > 40 while the GLOBAL total is charged always. The repaired address is the `sub`, the clause's first named operation. The original `0x10b624a2` was +1 into `cmp DWORD PTR ds:0x10c6f1c8,0x1` at `0x10b624a1`.

---

## The two structural predictions, graded

### P-A — a uniform `−0x11b` transcription shift explains C18 **and** C19. HOLDS.

    C18   0x10b6249b + 0x11b = 0x10b625b6   cmp eax,0x28                  <- the clause's test
    C19   0x10b624a2 + 0x11b = 0x10b625bd   movzx eax,WORD PTR [esi+0x50] <- feeds the clause's add

The **same** shift carries both original addresses onto real instruction
boundaries inside the block the clause describes. Two independent errors landing
on real instructions of the right block under one constant offset is not a
coincidence worth preferring over a shift.

(C19's repaired `addr` is `0x10b625bb`, the `sub`, because PREREG §3.1 takes the
clause's *first named operation* — `*budget -= …`. The shift lands on
`0x10b625bd`, the re-load two bytes later that feeds `DAT_10c3f5cc +=`. Both
facts are stated; neither is bent to make the constant tidier.)

### P-B — the "instruction-for-instruction duplicate of the wrong function" is two-thirds wrong. HOLDS.

`w-inlfit` §4 and `docs/ADOPTION_BRIEF_2026-08-28.md` §L5 both state that C18/C19
are early *because* `0x10b62488`–`0x10b624be` is an **instruction-for-instruction
duplicate of the wrong function**, copying `0x10b5fb85`–`0x10b5fbbb`.
Re-derived here by decoding both ranges:

| claim | verdict |
|---|---|
| the two ranges hold the same idiom, instruction for instruction | **true** — same nine operations in the same order |
| …*instruction-for-instruction* identical | **false** — the register differs throughout: `0x10b5fb85` uses `edi` as the zero (`39 3d`, `3b c7`), `0x10b62488` uses `ecx` (`39 0d`, `3b c1`). Not a byte copy |
| …of the **wrong function** | **false** — `0x10b62488` is inside `FUN_10b6242a`, which is the function C18/C19's `owner` cell **already names correctly** |
| there are two copies | **false** — there are **three**. A third sits at `0x10b62519`–`0x10b6254f`, also inside `FUN_10b6242a` |

And the duplicate does not explain the defect in any case: the block at
`0x10b62488` contains no `0x28` and no `+0x50` access, so no search for C18's
`cmp WORD [callee+0x50], 0x28` could have landed there by matching content.
P-A explains both rows with one number; the duplicate explains neither.

**What survives of `#3721`:** the eight, exactly (CTL-0), and the diagnosis that
`check_table.py` could not see them. What does not survive: the causal story for
C18/C19, the byte-identity claim, the "wrong function" claim, the count of
copies, and the completeness of "eight".


---

## Appendix — the FOURTEEN rows that were NOT repaired, and why each is correct

The claim *"the other fourteen are correct"* is worth as little as the claim
*"eight are wrong"* if nobody shows it. Each row below was checked the same
way as the ten: aligned per the objdump boundary set, inside its `owner`
function per `FUNCS.tsv`, **and** decoding to something its `clause` names.

**Five of the fourteen — C1, C5, C6, C20, C21, at four distinct addresses,
because C5 and C6 share one — cite function ENTRIES**, so their `asm` witness is
`push ebp`, and DECODE-green on them is a statement about the address, not about
the clause. That weakness is real, is recorded in the table header so nobody
reads it as clause-confirmation, and is §8 item 4 of the rung.

| row | `addr` | decode | why it is correct |
|---|---|---|---|
| **C1** | `0x10b62675` | `push ebp` | **Entry**, and the clause says *pass entry*. The test it names is four instructions in, at `0x10b6267b cmp DWORD PTR ds:0x10c40ec4,ebp` with `ebp` zeroed at `0x10b62676`; `jne 0x10b626c1` takes the work path when it is non-zero. Weak `asm` witness (`push ebp`) by construction. |
| **C5** | `0x10b600e6` | `push ebp` | **Entry** of the collector. Correct as a citation of the function; the `asm` cell pins the address and not the clause. |
| **C6** | `0x10b600e6` | `push ebp` | Same **entry** as C5 — two clauses citing one function, which the table already allows. |
| **C7** | `0x10b5e4d7` | `mov DWORD PTR ds:0x10c46318,0x3e8` | `0x3e8` = **1000**, the clause's second arm, stored on the `k > 6` branch taken from `0x10b5e4d2 cmp ecx,0x6` / `jle`. The first arm `0x10 << k` is at `0x10b5e4e3`–`0x10b5e4e8`. The address cites the arm the clause states last; both are real and in `FUN_10b5e4cc`. |
| **C8** | `0x10b5fc8a` | `cmp eax,DWORD PTR ds:0x10c46318` | Exact. `eax` is `movzx eax,WORD PTR [esi+0x50]` at `0x10b5fc86`, and the `jl 0x10b5fcb9` at `0x10b5fc90` is the clause's `jl = candidate`. Verified clean by `w-inlfit` too. |
| **C9** | `0x10b5fc7e` | `cmp DWORD PTR ds:0x10c2e310,ebx` | Exact. `ebx` is zero and the `jne 0x10b5fcb9` at `0x10b5fc84` jumps **past** C8's size test — the clause's SKIPPED, at an address. |
| **C11** | `0x10b5c06b` | `mov eax,DWORD PTR [ecx+0x20]` | Head of the legality block, and all four masks are within eleven instructions: `0x400` at `0x10b5c06e`, `0x1000` at `0x10b5c080`, `0x40` at `0x10b5c08f`, `0x100` at `0x10b5c093`, every one `jne 0x10b5c0a4` = `xor eax,eax; ret` = refuse. |
| **C12** | `0x10b5c06b` | `mov eax,DWORD PTR [ecx+0x20]` | Same block head. `0x80000` at `0x10b5c078` and `0x200` at `0x10b5c087`, on `ecx = [ecx+0x4c]` loaded at `0x10b5c075`. Both refuse to the same `0x10b5c0a4`. |
| **C13** | `0x10b5c06b` | `mov eax,DWORD PTR [ecx+0x20]` | Same block head. The fall-through at `0x10b5c09a`–`0x10b5c0a0` is `movzx eax,cl` / `shr eax,0x6` / `and eax,0x1` — bit 6 returned as the verdict, which is REQUIRE rather than refuse, exactly as the clause distinguishes. |
| **C20** | `0x10b620fc` | `push ebp` | **Entry** of the expander. The recursion edge `w-inlfit` §3 reads is at `0x10b62402`, inside `FUN_10b61ee1`, not here — the clause cites the expander, and does so correctly. |
| **C21** | `0x10b5fcd8` | `push ebp` | **Entry** of the POGO model. Its `entered only when` guard is the caller's, at `0x10b60a40 cmp DWORD PTR [ebp-0x4],ebx` — which is the instruction C15 was wrongly citing at `0x10b609bd`, an earlier test of the same value. |
| **C22** | `0x10b600c8` | `mov eax,ds:0x10c3f550` | Exact, and every term is present: `K` = `DAT_10c3f550`, `add eax,esi` makes `K + cost`, `cdq` / `idiv ecx` divides by the site count, `sub esi,eax` at `0x10b600d2` is the `-=`. |
| **C23** | `0x10b5b86d` | `cmp DWORD PTR ds:0x10c6f1c8,0x0` | Exact: the `cmp` is the selector, `mov esi,0x10c45ed0` at `0x10b5b876` is the default and `mov esi,0x10c45e18` at `0x10b5b87d` the alternative, copied `0x2e` dwords to `0x10c3f510`. |
| **C24** | `0x10b9bf6c` | `mov WORD PTR [esi+0x50],ax` | Exact, and it is a **store** — the write of the field C8/C17/C18/C19 later read. `[esi+0x4c]` is written three instructions later at `0x10b9bf78`, which is the other field this table depends on. |

**14 rows, all verified, none moved.** With the ten repaired above that
is 24 of 24 accounted for, which is the only form in which "the table is
correct" means anything.
