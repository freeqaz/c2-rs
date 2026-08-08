# WB-MEMCPY — the intrinsic expansion decision function, read out of the binary

> **PROVENANCE — DISASSEMBLY-DERIVED.** Everything below was obtained by
> statically disassembling Microsoft's `c2.dll` — the exact image pinned in
> [`C2_MAP_METHOD.md`](C2_MAP_METHOD.md) §0, sha256 `c80981…6258`, verified at
> the head of this lane. It is **navigation** until it earns a
> [`DISCLOSURE.md`](DISCLOSURE.md) row. **The obj is the sole judge**: a reading
> that disagrees with what c2 emitted is wrong, however clean the code looked
> (method doc §7 — the `.bss` retraction). This lane adopts **nothing** into
> `crates/`.

Lane **WB-C** of [`CAMPAIGN_2026-08-08.md`](CAMPAIGN_2026-08-08.md). Ground
truth is [`docs/rungs/2026-08-08-w-memcpy.md`](../rungs/2026-08-08-w-memcpy.md)
— 1,155 frozen grid cells, already-paid-for obj checks. Where this document and
those cells disagree, the cells win.

---

## 0. PRE-REGISTRATION

Written and committed **before the first `cl.exe` of GRID-W**, the only new
measurement this lane makes. It is scored in §8. Board #770's streak stands at
~10 optimistic / 2 pessimistic / 1 hit; this lane registers **OPTIMISTIC**, on
the argument that the reading below already reproduces four measured boundaries
it was not fitted to, which is exactly the state in which a lane over-trusts
the next one.

| # | registered |
|---|---|
| **Q1** | The expansion decision is keyed on **element count = size / alignment-hint**, not on the intrinsic id, not on the size alone, and not on the C type. |
| **Q2** | The threshold is **5**, and the alternative **10** is selected by a single global that the option decoder derives from **one bit of the option word**. GRID-W part A will show a threshold of **10** at `/O2`, `/Ox` and `/O1 /Ot`, and **5** at `/O1` and `/O2 /Os` — i.e. **W-OT** beats both **W-T5** and **W-LEVEL**. |
| **Q3** | A **non-constant** size is a call unconditionally, decided before the division. |
| **Q4** | **Size 0 emits nothing**, and that arm is inside the lowering itself (not an earlier pass). |
| **Q5** | The **elimination** measured by GRID-M2 (44 cells, bare `blr`) is **NOT** in the memcpy lowering. At size 96 / align 8 the lowering reads `n = 12 > 5` and would emit a call, so something upstream removed the node. GRID-W part B will show **E-DEADDST** (dead non-escaping destination) beating **E-LOCALS** (both operands local). |
| **Q6** | `memcpy` is minted by a **name→symbol** call taking a C string literal, which is why it never appears in the `.gl` name stream. Two mint sites, one in the expansion and one later. |
| **Q7** | This reading explains **none** of R-DESC's 24 misses and **neither** of the two block-plan mismatches. They are a different subsystem (call-site move scheduling and block planning); the memcpy work is a tuple rewrite that happens before either. Registered as a **decline**, not as a gap to be closed by this lane. |
| **Q8** | GRID-W part A will find **≥ 1 flag set at which some rival that scored 100 % on w-memcpy's 408 cells is refuted**, i.e. the existing grids' agreement with "threshold 5" is a property of their single flag set. |

**Decline clauses, registered in advance.**

* If GRID-W part A shows the threshold is 5 at *every* flag set, **Q2 is
  retracted** — the global is then not the one being read, or it is not
  reachable from `cl.exe` flags, and the reading of `0x10bf65d6` is downgraded
  to "read correctly, not what c2 does" (method doc §7, case 1).
* If GRID-W part B shows `lu` (two locals, destination used afterwards)
  eliminated, **Q5's E-DEADDST is retracted** and the mechanism is broader than
  dead-destination.
* Nothing here is adopted into `crates/` under any outcome. The deliverable is
  a reading plus DISCLOSURE rows a later code lane may carry.

---

## 1. Where the lowering lives

| VA | what |
|---|---|
| **`0x10bf7c59`** | the intrinsic/tuple dispatcher (`cgintrin.c` — the file string at `0x10b19698` is referenced from this function at `0x10bf7d8b`). Switches on `WORD [node+0x34]`, the tuple opcode. |
| **`0x10bf7dd6`** | the `0xa4…0xb2` arm: `sub ecx,0xa4; cmp ecx,0xe; ja …` then a byte index table at **`0x10bf8e59`** into a jump table at **`0x10bf8e35`**. |
| **`0x10bf7e3b`** | the **`0xac` = 172** target: `push [ebp+8]; push 1; call 0x10bf6555` — **memcpy**. |
| **`0x10bf7e28`** | the **`0xad` = 173** target: `push [ebp+8]; push 1; call 0x10bf5d2b` — **memset**. |
| **`0x10bf7d0b`** | `cmp ecx,0x96` — opcode **`0x96` = 150**, the block assignment; it enters the *same* lowering at `0x10bf7e40` but passes `movsx eax, WORD [node+0x38]` where memcpy passes `1`. |
| **`0x10bf6555`** | **the memcpy/blkmov expansion**, 1477 bytes, one caller. |
| **`0x10bf5d2b`** | the memset expansion, 2090 bytes, one caller — same decision, own code. |
| **`0x10c083e7`** | a *second*, later `memcpy` materializer (MD level: `0x2a6` symbol ref, `0x2b` call), which mints the name again and sets `[sym+0x20] \|= 0x10800`. Not on the expansion path. |

**`0xac` = 172 is the selector w-memcpy's rung named** ("the rule a reader keyed
on selector 172 would implement"). It is the tuple opcode, decoded here.

## 2. The decision function

Read off `0x10bf6555`. `param_2` = the tuple node, `param_3` = 1 for memcpy /
memset and the element width for a `0x96` block assign.

```
  align  = BYTE [node+0x38]          if WORD [node+0x34] == 0xac   (0x10bf657f)
         = (byte)param_3             otherwise                     (0x10bf6584)
  align  = max(align, 1)                                           (0x10bf658b)

  size   = the third operand, reached by  arg[0] -> arg[1] -> arg[2]
           through 0x10c2574b                                      (0x10bf65b1)

  if kind(size) != 7   ->  CALL                                    (0x10bf65b8)
                           (7 = 64-bit integer constant; a variable
                            size never reaches the division)

  n      = size / align          64-bit signed, __alldiv           (0x10bf65d1)

  T      = 5   if  DWORD [0x10c2e310] == 0                         (0x10bf65e3)
         = 10  otherwise                                           (0x10bf65de)

  if n <= T   ->  INLINE   (fall through to 0x10bf6649)            (0x10bf65e6)
  else        ->  CALL     (0x10bf65e8)
```

The call path then:

```
  0x10bf65f9   cmp WORD [node+0x34], 0x96
  0x10bf6614   ecx = 0x10b19724  "_blkmov"      (block assign)
  0x10bf661b   ecx = 0x10b1971c  "memcpy"       (everything else)
  0x10bf6620   call 0x10b9ae7e                  <- THE MINT: name -> symbol
  0x10bf6627   call 0x10bd4361                  symbol -> operand node
  0x10bf662e   edx = 0x2dc                      the call tuple opcode
  0x10bf6635   call 0x10bd754d                  build the call
```

memset is the same shape at `0x10bf5e30` (division), **`0x10bf5e35`** (the flag),
`0x10bf5e3e` (10), `0x10bf5e43` (5), `0x10bf5e46` (`jle` → inline), `0x10bf5e48`
(`cmp WORD [node+0x34],0xad`), **`0x10bf5e5d`** (`ecx = 0x10b19714` `"memset"`),
`0x10bf5e62` (the same mint at `0x10b9ae7e`).

### 2.1 `0x10c2e310` — one bit of the option word

`0x10c2e310` is a 4-byte global in `.data`, image value `1`, 77 readers. Its
live writer is the option decoder:

```
  0x10b8238d   shr ecx,0x17          <- BIT 23 of the option word in eax
  0x10b82390   and ecx,esi           esi = 1
  0x10b82392   mov ds:0x10c2e310,ecx
```

(The other write, at `0x10b624dc`, is behind `xor eax,eax; test eax,eax; je` at
`0x10b624c0` — dead. It masks the same `0x800000`, which corroborates the bit.)

The measured boundaries in §3 were all taken at the dc3 workload's `/O1 /Oi`,
where this reads **0**. **Which `cl.exe` flag sets bit 23 is not read off the
disassembly here** — it is measured, in §4.

### 2.2 The shapes it selects among (inline path)

```
  0x10bf66f1   if param_3 == 1 and size > 1:                (memcpy/memset only)
  0x10bf6714       unit = min(align, 8)
  0x10bf6723       while size % unit != 0:  unit >>= 1
  0x10bf673b       count = size / unit
  0x10bf6758       if unit > 1: the trip-count operand becomes a NEW constant
               else:                                        (0x96 block assign)
                   unit = param_3, count = size              [see §6.1]

  0x10bf6835   count_hi > 0            -> LOOP
  0x10bf6841   count_lo > 4 (unsigned) -> LOOP               (0x10bf6909)
  otherwise    fully UNROLLED straight line                  (0x10bf684b)
```

The unrolled arm (`0x10bf688a`–`0x10bf6902`) emits, per iteration, a `0x29f`
load and a `0x29f` store joined by `0x2af`, with the displacement advanced by
`unit` (`0x10bf68ee`) and the count decremented (`0x10bf68f4`). The loop arm
(`0x10bf6909`–) emits `0x2c6` (base − unit) on both pointers, opens a block
(`0x10c0f12e`), materialises the trip count into a temp, allocates a label with
`0x10bd42c2(0x54, 0x2004)`, emits `0x2c5` (advance) on both pointers and closes
with `0x288`.

### 2.3 Size zero

`0x10bf669d`: `or`-ing the two halves of the 64-bit constant and `jne` past the
whole emit. **Size 0 emits no call and no copy, and the arm is inside the
lowering.** (Q4.)

## 3. The check against w-memcpy's measured grid

Every boundary below was measured by w-memcpy at `/O1 /Oi` (so T = 5) and is
recomputed here from §2 with **no** parameter fitted to it.

| measured cell (w-memcpy §6.1/§6.2) | `align` | `n = size/align` | reading | measured | ✓ |
|---|---:|---:|---|---|---|
| `char*` size 5 | 1 | 5 | inline | inline | ✓ |
| `char*` size 7 | 1 | 7 | call | call | ✓ |
| `int*` size 20 | 4 | 5 | inline | inline | ✓ |
| `int*` size 24 | 4 | 6 | call | call | ✓ |
| `double*` size 44 | 8 | **5** (44/8 truncates) | inline | inline | ✓ |
| `double*` size 48 | 8 | 6 | call | call | ✓ |
| 16-byte struct, every size | 8 | same as `double*` | identical | identical | ✓ |
| size 0, both callees | any | — | nothing | nothing | ✓ |
| `void*` sizes 44, 46, 47 | 1 | 44, 46, 47 | call | call | ✓ |
| 4-byte struct sizes 44, 46, 47 | 4 | 11 | call | call | ✓ |
| `long long*` sizes 44, 46, 47 | 8 | 5 | inline | inline | ✓ |
| 32-byte struct sizes 44, 46, 47 | 8 | 5 | inline | inline | ✓ |
| variable size | — | — | call | call (`M-VARCALL`'s direction) | ✓ |

**The truncating division is the whole of the `double*` 44/48 boundary**, and
it is the one thing no size-only or type-only rule can produce: 44 and 47 are
`inline` while 48 is a `call`, at the *same* alignment, because `44/8 = 47/8 =
5` and `48/8 = 6`. That is why the four separately-frozen thresholds in GRID-M
(8/16/32/64) all missed — none of them is a threshold on size at all.

It also retro-explains **why `M-ALWAYSCALL` scored 114/232 and `M-THRESH-32`
scored 182**: a size threshold at 32 agrees with `n ≤ 5` exactly on the cells
where `align` happens to be 4–8, and disagrees everywhere `char*` or `void*` is
in play.

### 3.1 What the reading does NOT cover, stated so absence does not read as coverage

* **R-DESC's 24 misses** (w-memcpy §4.2/§4.3 — the two-call and live-across-call
  drivers). Not covered, and registered as a decline (Q7). Those cells contain
  no intrinsic at all; they are about the order in which call-site `li`/`mr`
  ops are placed, which is a later, register-level concern than a tuple
  rewrite. Nothing at `0x10bf6555` touches argument slots.
* **The two block-plan `Port=Mismatch`** (w-memcpy §5.1 — one `li r3,5` shared
  by an early return and a call argument, with the branch inverted). Not
  covered, same reason: block planning, not intrinsic expansion. This lane did
  not read the block planner and does not speculate about it.
* **Which `cl.exe` flags set bit 23.** Read as a bit position, not as a flag
  name. §4 measures it.

## 4. GRID-W — the new frozen grid

216 cells, generated by `work/wb-memcpy/gridw.py`, frozen with every rival's
per-cell prediction and the separation assertions **before the first `cl.exe`**
(commit named in §7). sha256 of the cell sources
`7cea5fdcbe9d63358e9d9307c852e3b8b78c587e561eeffb50d16c03be8913e5`.

**Part A (180 cells)** — `char*`/`int*`/`double*` × `n ∈ {4,5,6,9,10,11}`
elements × {`memcpy`,`memset`} × five flag sets. Rivals `W-T5` (threshold always
5), `W-LEVEL` (by `/O<n>` level) and `W-OT` (by favor-speed — the reading).
Separations asserted: 54 / 36 / 54 cells, ≥ 4 on every pointer type.

**Part B (36 cells)** — six operand shapes × sizes 16 and 96 × three pointer
types, at `/O1`. Rivals `E-LOCALS` (w-memcpy's stated finding) and `E-DEADDST`
(dead non-escaping destination). Separated on 12 cells, by construction on the
`lu` shape (two locals, destination used afterwards) and the `ld` shape (dead
local destination, formal source).

**The verdict function has three arms** — `call` / `none` / `inline`, with
`none` decided by the byte count and not by the absence of a relocation. That
is w-memcpy §6.2's own control: its first verdict function had no `none` arm
and reported a fence refuted by an inline expansion that did not exist.

<!-- GRIDW-RESULTS-START -->
*(filled in by §5 once measured)*
<!-- GRIDW-RESULTS-END -->

## 5. Results

*(see §5 below, written after the run)*

## 6. Two things the disassembly says that are worth carrying separately

### 6.1 An arm that the one caller cannot reach

`0x10bf6555`'s `0xac` test at `0x10bf6579` and its `param_3 == 1` test at
`0x10bf66f1` are *both* satisfied only on the memcpy/memset entries
(`0x10bf7e3b` / `0x10bf7e28`, which push a literal `1`). On the `0x96` entry
(`0x10bf7d1f`) `param_3` is the element width, so the unit-shrink block is
skipped and the trip count stays at `size` — which would be wrong arithmetic if
that path also took the inline arm. The two facts that make it consistent are
that a `0x96` node's `align` is then `(byte)param_3`, so `n = size/param_3` is
already the element count, and `unit = param_3`. **The `else if` arm the
decompiler shows at `0x10bf6769` is unreachable from the single call site**
(it is guarded by "size is not a constant", which `0x10bf65bf` has already sent
to the call path). Recorded because method doc §7 case 1 — *the path read is not
the one real inputs take* — is exactly this shape, and a reader who quoted that
arm would be quoting dead code.

### 6.2 The mint, and why `memcpy` is not in the `.gl` stream

w-memcpy §2 recorded, on no board row, that `memcpy` does not occur in the
`.gl` name stream at all while every other callee resolves from a token. The
reason is `0x10bf6620` / `0x10bf5e62` / `0x10c08483`: the name is a **C string
literal in `c2.dll`** (`0x10b1971c` `"memcpy"`, `0x10b19714` `"memset"`,
`0x10b19724` `"_blkmov"`) handed to `0x10b9ae7e`, the by-name symbol
lookup/insert. The IL never carries the name because c2 owns it. A port that
resolves callees only through `bundle::resolve` therefore cannot produce this
symbol at all, whatever it does with the arguments — which is the fifth
independent refusal w-memcpy §6.3 priced.

## 7. Reproducing

```sh
sha256sum ~/ghidra-projects/bin/c2dll   # must equal c80981…6258

# the readings — flat export only, never the Ghidra project (method doc §4)
grep -n memcpy ~/ghidra-projects/export/c2/strings.tsv
awk '/^10bf6555:/,/^10bf6b1a:/' ~/ghidra-projects/export/c2/objdump_intel.asm

# the jump table that names opcode 0xac
python3 - <<'PY'
import struct
f = open('<c2dll>', 'rb').read()
off = lambda va: va - 0x10b00c00
t = [struct.unpack('<I', f[off(0x10bf8e35)+4*i:off(0x10bf8e35)+4*i+4])[0]
     for i in range(9)]
for i, b in enumerate(f[off(0x10bf8e59):off(0x10bf8e59)+15]):
    print(hex(0xa4+i), hex(t[b]))
PY

# GRID-W
python3 work/wb-memcpy/gridw.py gen   work/wb-memcpy/probeW
python3 work/wb-memcpy/gridw.py run   work/wb-memcpy/probeW "$PWD"
python3 work/wb-memcpy/gridw.py score work/wb-memcpy/probeW
```

## 8. Pre-registration, scored

*(filled in after §5)*

## 9. Pre-drafted DISCLOSURE rows

*(filled in after §5 — only what the obj confirmed)*
