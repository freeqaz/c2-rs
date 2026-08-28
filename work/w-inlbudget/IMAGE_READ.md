# `w-inlbudget` — the image read, V1–V7 re-derived, and what §6.6.2 did not have

Image `compilers/X360/16.00.11886.00/c2.dll`,
`sha256 c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`
(verified on this tree). Disassembly: the independent objdump listing
(`objdump -d -M intel`, PE32 as `pei-i386` at true VAs), regenerated and never
committed. Extracted with `work/w-inlfit/grab.py`.

Every line below is quoted from that listing. **Prereg `work/w-inlbudget/PREREG.md`
was committed at `fa4e059cf`, before the first `grab.py` invocation.**

---

## 1. The V-table — all seven confirm

### V1 · the recursion edge, `0x10b62402` — **CONFIRMED**

```
10b623e6:  8b 5d 0c        mov    ebx,DWORD PTR [ebp+0xc]     <- the budget POINTER
10b623e9:  8b 03           mov    eax,DWORD PTR [ebx]         <- *budget
10b623eb:  99              cdq
10b623ec:  f7 7d 14        idiv   DWORD PTR [ebp+0x14]        <- V4: / remaining_sites
10b623ef:  ff 77 14        push   DWORD PTR [edi+0x14]        <- stack4 = site+0x14
10b623f2:  0f b6 57 18     movzx  edx,BYTE PTR [edi+0x18]     <- V3
10b623f6:  ff 77 10        push   DWORD PTR [edi+0x10]        <- stack3 = site+0x10
10b623f9:  03 55 10        add    edx,DWORD PTR [ebp+0x10]    <- V3: + level
10b623fc:  ff 75 18        push   DWORD PTR [ebp+0x18]        <- stack2, unchanged
10b623ff:  8b ce           mov    ecx,esi                     <- the function
10b62401:  50              push   eax                         <- stack1 = the divided budget
10b62402:  e8 da fa ff ff  call   0x10b61ee1
```

The push order fixes the four stack arguments unambiguously (last push = first
stack argument). **§6.6.2's six-parameter table is exactly right.**

### V2 · `FUN_10b61ee1` has exactly two callers — **CONFIRMED**

`grep 'call   0x10b61ee1'` over the whole 22 MB listing returns **two** lines:
`0x10b62402` and `0x10b6276e`. The pass entry's own call, with all six
arguments, and it confirms `level = 1`, `budget = B`, `stack3 = 1e8`:

```
10b62736:  55                    push   ebp             <- ebp is 0 here; stack4 = 0
10b62737:  68 00 e1 f5 05        push   0x5f5e100       <- stack3 = 100,000,000
10b6273f:  55                    push   ebp             <- stack2 = 0
10b62742:  53                    push   ebx             <- stack1 = B
10b62743:  8b d7                 mov    edx,edi         <- edi = 1: LEVEL = 1
10b62745:  8b ce                 mov    ecx,esi         <- the function
10b6276e:  e8 6e f7 ff ff        call   0x10b61ee1
```

### V3 · `level' = BYTE [site+0x18] + level` — **CONFIRMED, and the `+ level` operand is traced**

`FUN_10b620fc`'s `[ebp+0x10]` (the addend at `0x10b623f9`) traces back through
`FUN_10b6242a`'s `[ebp+0xc]` and `FUN_10b61d2c`'s `[ebp+0x8]` to the driver's
local `[ebp-0x20]`, and the driver's prologue is:

```
10b61ef2:  8b f2        mov    esi,edx            <- edx IS the level parameter
10b61ef6:  89 75 e0     mov    DWORD PTR [ebp-0x20],esi
```

So the addend is the driver's own `level`. The claim holds end to end.

### V4 · the budget argument is `*budget / remaining_sites` — **CONFIRMED**

`idiv DWORD PTR [ebp+0x14]` at `0x10b623ec`, dividend `*[ebp+0xc]`. The same
`[ebp+0xc]` is the cell the charge writes back through at `0x10b62418`
(`sub DWORD PTR [ebx],eax`), so the dividend is the live remaining budget.

### V5 · the divisor is the site collector's out-parameter — **CONFIRMED, all four frames**

| frame | function | slot | the instruction that passes it on |
|---|---|---|---|
| driver | `FUN_10b61ee1` | local `[ebp-0xc]` | `lea edx,[ebp-0xc]` `0x10b61f99`, then `call 0x10b600e6` `0x10b61f9f` |
| ↓ | | | `push DWORD PTR [ebp-0xc]` **`0x10b62068`** (3rd of 8 pushes → callee `[ebp+0x1c]`) |
| per-site | `FUN_10b61d2c` | `[ebp+0x1c]` | `push DWORD PTR [ebp+0x1c]` **`0x10b61e04`** (3rd of 5 → callee `[ebp+0x10]`) |
| charge | `FUN_10b6242a` | `[ebp+0x10]` | `push DWORD PTR [ebp+0x10]` **`0x10b625d3`** (4th of 6 → callee `[ebp+0x14]`) |
| expansion | `FUN_10b620fc` | `[ebp+0x14]` | `idiv DWORD PTR [ebp+0x14]` `0x10b623ec` |

`FUN_10b61d2c` has exactly one caller (`0x10b62081`) and `FUN_10b6242a` exactly
one (`0x10b61e12`), so the chain is not merely *a* path — it is **the only**
path.

**And the counter's semantics are now closed, which §6.6.2 asserted but did not
show.** The collector zeroes the out-parameter on entry and increments it once
per site record it appends:

```
10b600f7:  89 1a        mov    DWORD PTR [edx],ebx     <- ebx = 0: ZEROED at entry
10b60102:  89 55 c8     mov    DWORD PTR [ebp-0x38],edx
...
10b60371:  8b 45 c8     mov    eax,DWORD PTR [ebp-0x38]
10b60374:  ff 00        inc    DWORD PTR [eax]         <- once per site appended
```

and the driver decrements it once per loop iteration at the bottom
(`dec DWORD PTR [ebp-0xc]`, `0x10b620c8`, immediately before
`jne 0x10b61fae`). So at site *i* of *n*, 1-based, the divisor is exactly
`n − i + 1`. **`remaining_budget / (n − i + 1)` is right as published.**

### V6 · `__forceinline` is charged nothing — **CONFIRMED, and there are TWO such skips, not one**

The one §6.6.2 cites, on the **nested expansion's consumed budget**:

```
10b62407:  8b 4d f4              mov    ecx,DWORD PTR [ebp-0xc]     <- the callee sym
10b6240a:  a3 d0 f5 c3 10        mov    ds:0x10c3f5d0,eax
10b6240f:  f7 41 4c 00 20 00 00  test   DWORD PTR [ecx+0x4c],0x2000
10b62416:  75 08                 jne    0x10b62420
10b62418:  29 03                 sub    DWORD PTR [ebx],eax          <- local budget
10b6241a:  01 05 cc f5 c3 10     add    DWORD PTR ds:0x10c3f5cc,eax  <- global growth
```

and a second, in the charge function, on the **callee's own instruction count**:

```
10b625a6:  f7 46 4c 00 20 00 00  test   DWORD PTR [esi+0x4c],0x2000
10b625ad:  8b 7d 08              mov    edi,DWORD PTR [ebp+0x8]      <- budget pointer
10b625b0:  75 15                 jne    0x10b625c7                   <- __forceinline: skip BOTH
10b625b2:  0f b7 46 50           movzx  eax,WORD PTR [esi+0x50]
10b625b6:  83 f8 28              cmp    eax,0x28                     <- C18, 40 instrs
10b625b9:  76 02                 jbe    0x10b625bd                   <- <=40: skip the LOCAL only
10b625bb:  29 07                 sub    DWORD PTR [edi],eax          <- C19a, local budget
10b625bd:  0f b7 46 50           movzx  eax,WORD PTR [esi+0x50]
10b625c1:  01 05 cc f5 c3 10     add    DWORD PTR ds:0x10c3f5cc,eax  <- C19b, global growth
```

**This is the sharpest form of §6.6.2's orthogonality claim and it is now
visible in one listing**: the `jbe` at `0x10b625b9` skips **only** the local
budget, while the `jne` at `0x10b625b0` skips **both**. The two exemptions are
different in *extent*, not only in *condition*.

> **One correction to §6.6.2, and it is small.** *"`0x10b6240f` … skips **both**
> `sub DWORD [ebx],eax` and `add ds:0x10c3f5cc,eax`"* is true of the two stores
> it names, but there is a **third** global write on that path and it is
> **not** skipped: `mov ds:0x10c3f5d0,eax` at `0x10b6240a` stores the nested
> pass's consumed budget unconditionally, *before* the test. A reader taking
> "charged nothing" as "leaves no trace in c2's global state" would be wrong by
> exactly one datum.

### V7 · stack 3/4 are one 64-bit quota that halves — **CONFIRMED**

```
10b61fff:  39 4d 14           cmp    DWORD PTR [ebp+0x14],ecx        <- 64-bit compare of the
10b62002:  77 11              ja     0x10b62015                         driver's own (stack3,stack4)
10b62004:  72 09              jb     0x10b6200f                         pair against 0x5f5e100
10b62006:  81 7d 10 00 e1 f5 05  cmp DWORD PTR [ebp+0x10],0x5f5e100
...
10b62015:  8b 4e 14           mov    ecx,DWORD PTR [esi+0x14]        <- the SITE's pair
10b62018:  8b 46 10           mov    eax,DWORD PTR [esi+0x10]
10b6202f:  e8 78 9e ff ff     call   0x10b5beac                      <- 3 × 64-bit in
10b62034:  89 46 10           mov    DWORD PTR [esi+0x10],eax
10b62037:  89 56 14           mov    DWORD PTR [esi+0x14],edx
10b6203a:  f6 43 4c 10        test   BYTE PTR [ebx+0x4c],0x10        <- gated on the callee's 0x10
10b6204e:  0f ac f8 01        shrd   eax,edi,0x1                     <- the HALVING
10b62052:  d1 ef              shr    edi,1
10b62054:  89 46 10           mov    DWORD PTR [esi+0x10],eax
10b62057:  89 7e 14           mov    DWORD PTR [esi+0x14],edi
```

The pair is `(low, high)` and the seed is `0x5f5e100` = 100,000,000, matching
the pass entry's `push 0x5f5e100`. **Not adopted** — the port has no consumer
for it, the `0x10b5beac` helper's arithmetic is unread, and the halving is
gated on a flag bit the port cannot see.

---

## 2. THE FINDING §6.6.2 DOES NOT HAVE — what `BYTE [site+0x18]` actually holds

§6.6.2 publishes `level' = BYTE [site+0x18] + level` and leaves the field
unexplained. It has exactly **two** writers in the site collector `FUN_10b600e6`,
and between them they close it:

**Writer 1 — every site is born with `+0x18 = 1`.**

```
10b602ce:  c6 40 18 01     mov    BYTE PTR [eax+0x18],0x1
```

immediately after `call 0x10c2022a` (the arena allocation, `ecx = 7`) that
creates the record and `mov [eax+0x4],esi` that plants the site.

**Writer 2 — a fixup pass overrides it, for a callee already being expanded.**

```
10b604d0:  8b 55 e8        mov    edx,DWORD PTR [ebp-0x18]     <- the site list head
10b604d7:  8b 4a 04        mov    ecx,DWORD PTR [edx+0x4]
10b604da:  e8 cf ba ff ff  call   0x10b5bfae                   <- eax = the callee sym
10b604df:  f6 40 4c 10     test   BYTE PTR [eax+0x4c],0x10     <- only for THIS bit
10b604e3:  74 18           je     0x10b604fd
...
10b604ea:  39 41 04        cmp    DWORD PTR [ecx+0x4],eax      <- find the callee's record
10b604f7:  8a 41 08        mov    al,BYTE PTR [ecx+0x8]
10b604fa:  88 42 18        mov    BYTE PTR [edx+0x18],al       <- the OVERRIDE
```

and the record it reads is a **per-callee multiplicity counter**, a 12-byte
`{next, sym, count}` node built in the same scan:

```
10b60398:  8a 48 08        mov    cl,BYTE PTR [eax+0x8]
10b6039b:  80 f9 ff        cmp    cl,0xff
10b6039e:  0f 83 …         jae    0x10b604b1                   <- saturates at 255
10b603a4:  fe c1           inc    cl
10b603a6:  88 48 08        mov    BYTE PTR [eax+0x8],cl
...
10b603b9:  89 58 04        mov    DWORD PTR [eax+0x4],ebx      <- keyed on the callee sym
10b603bc:  c6 40 08 01     mov    BYTE PTR [eax+0x8],0x1       <- first occurrence = 1
```

> ### `BYTE [site+0x18]` IS THE LEVEL INCREMENT, AND ITS ORDINARY VALUE IS **1**.
>
> It is `1` for every site by construction (`0x10b602ce`), and is replaced by
> the callee's occurrence count **only** when that callee carries `[sym+0x4c] &
> 0x10` — the very bit the driver *sets on the function it is expanding* at
> `0x10b61f56` (`or eax,0x10` into `[fn+0x4c]`) and clears at `0x10b620dc`.
> That bit means **"already on the inline stack"**, so the override is c2's
> handling of a **recursive** callee: a callee re-entered `m` times advances the
> level by `m` at once rather than by 1.
>
> Consequence, and it is the one that matters here: **on any chain with no
> recursion, `level` advances by exactly 1 per expansion**, so `level` is a
> true nesting depth and C14's `0x10` is a **16-level** cap on it.

**C14, re-derived, because the model needs the cap and §6.1's address for it is
mid-instruction:**

```
10b60a0b:  a1 0c f5 c3 10  mov    eax,ds:0x10c3f50c            <- the base
10b60a13:  3b c3           cmp    eax,ebx                      <- ebx = 0
10b60a15:  74 0e           je     0x10b60a25                   <- base 0: no cap
10b60a17:  8b 4d 08        mov    ecx,DWORD PTR [ebp+0x8]      <- the level
10b60a1a:  2b c8           sub    ecx,eax
10b60a1c:  83 f9 10        cmp    ecx,0x10
10b60a1f:  7f d2           jg     0x10b609f3                   <- decline
```

and the base is seeded at the pass entry (`mov ds:0x10c3f50c,ebp`, `0x10b6274c`,
`ebp = 0`) and at `0x10b61f77` (`mov ds:0x10c3f50c,esi`, `esi` = the level) for
the first function carrying `[fn+0x4c] & 0x10`.

**C3, re-derived for the same reason:**

```
10b626f7:  0f b7 40 50     movzx  eax,WORD PTR [eax+0x50]      <- caller instrs (C2)
10b62703:  a3 cc f5 c3 10  mov    ds:0x10c3f5cc,eax
10b62708:  03 c0           add    eax,eax                      <- × 2
10b6270a:  bb e8 03 00 00  mov    ebx,0x3e8                    <- 1000
10b6270f:  3b c3           cmp    eax,ebx
10b62711:  7e 02           jle    0x10b62715
10b62713:  8b d8           mov    ebx,eax
10b62715:  b8 b8 88 00 00  mov    eax,0x88b8                   <- 35000
10b6271a:  3b d8           cmp    ebx,eax
10b6271c:  7c 02           jl     0x10b62720
10b6271e:  8b d8           mov    ebx,eax
```

`B = min(max(2 × caller_instrs, 1000), 35000)`. §2.2's clamp, unchanged.

---

## 3. Addresses this lane verified, offered to `w-clausefix` and NOT edited here

`work/w-inlmetric/CLAUSES.tsv` is `w-clausefix`'s under decision 22 and this
lane's prereg forbids touching it. Independently re-derived, for that lane to
take or reject:

| row | cited in §6.1 | what this lane decodes there | the address the clause describes |
|---|---|---|---|
| C2 | `0x10b626d8` | mid-instruction | **`0x10b626f7`** (`movzx eax,WORD [fn+0x50]`) / `0x10b62703` (the store) |
| C3 | `0x10b626f4` | mid-instruction | **`0x10b62708`**, clamp through `0x10b6271e` |
| C14 | `0x10b609ae` | mid-instruction (inside `and eax,0x10` at `0x10b609ad`) | **`0x10b60a0b`**–`0x10b60a1f` |
| C18 | `0x10b6249b` | mid-instruction | **`0x10b625b6`** (`cmp eax,0x28`) — agrees with §6.6.3 |
| C19 | `0x10b624a2` | mid-instruction | **`0x10b625bb`** and **`0x10b625c1`** — agrees with §6.6.3 |
| C10 | `0x10b609d3` | `call 0x10b5e64d`, aligned but a different instruction | unresolved; §6.6.3's finding is reproduced |

C4's `0x10b6276a` → `0x10b6276e` is §6.6.3's and is reproduced here as V2.
