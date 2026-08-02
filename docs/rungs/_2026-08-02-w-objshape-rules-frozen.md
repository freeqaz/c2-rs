# w-objshape — rules frozen, held-out predictions (2026-08-02)

Committed **before** the held-out cells were compiled. The rules below were
derived from the pre-registered grid only (`base`, `big`, `ar0`, `ar1i`,
`ar1p`, `ar1f`, `ar3`, `ar4`, `dtor`, `two`, `extlink`, `str2`, `sz1..sz256`,
`mint`/`mshort`/`mdbl`, plus one ordinary-function control `call`). The
held-out cells were chosen **after** this text was written, from axes the rules
did not see.

Reference flags for every rule: `/O1 /Oi /EHsc /GS- /c` (the workload's), unless
a rule explicitly names `/Ox`.

## The rules, as derived

**R1 — section order.** Fixed prefix `.drectve`, `.debug$S`, `.XBLD$W`(C2),
`.XBLD$W`(C1). Then one *group* per emitted function, in emission order:
`.text$y?` COMDAT, then each `.rdata` COMDAT it is the first to reference (in
first-reference order), then its `.pdata` if it is framed. Then a single `.bss`,
then a single `.CRT$XCU`. Both trailing sections are non-COMDAT.

**R2 — characteristics.** `.text$y?` `0x60401020`; `.rdata` COMDAT
`0x40<a>01040`; `.pdata` `0x40401040`; `.bss` `0xc0<a>00080`; `.CRT$XCU`
`0xc0300040`. `<a>` is the alignment nibble from R4.

**R3 — COMDAT selection.** `.text$y?` for a `??__E`/`??__F` thunk and its
`.rdata` COMDATs use **Selection 2 (ANY)**; an *ordinary* function's `.text`
uses **Selection 1 (NODUPLICATES)**. `.pdata` uses **Selection 5
(ASSOCIATIVE)** with `Number` = the 1-based index of the `.text` section it
belongs to.

**R4 — alignment.** For an object or `.rdata` blob of `n` bytes with natural
alignment `t`: `align = max(t, 1 if n < 2 else 4 if n < 64 else 8)`.

**R5 — `.bss`.** One section for all namespace-scope objects. `SizeOfRawData`
carries the total size; `PointerToRawData = 0`; `VirtualSize = 0`. Object
symbols are listed in **reverse address order**. A `static` object's symbol is
STATIC and unmangled (`sL`); a non-`static` one is EXTERNAL and mangled
(`?gL@@3UL@@A`).

**R6 — `.CRT$XCU`.** One section, `SizeOfRawData = 4 × (number of dynamic
initializers)`, raw data all zero, one `ADDR32` relocation at offset `4i`
targeting the *i*-th `??__E…@@YAXXZ`. Symbol `<name>$initializer$` is STATIC,
`Value = 4i`, listed in **forward** order.

**R7 — the thunk symbols.** `??__E<name>@@YAXXZ` and `??__F<name>@@YAXXZ` are
**STATIC** (storage class 3), `Type = 0x0020`, `Value = 0`, even when the object
has external linkage. `??__E` lives in `.text$yc`, `??__F` in `.text$yd`.

**R8 — relocations on `.text$y?`.** Ascending offset. Each address operand
contributes a `REFHI` (0x0010) at its `lis` and a `REFLO` (0x0011) at the
consuming instruction (`addi`, or a load's displacement field). **Every** REFHI
and **every** REFLO is immediately followed by a `PAIR` (0x0012) record whose
`VirtualAddress` equals the primary's and whose `SymbolTableIndex` is 0. The
call is `REL24` (0x0006) with no PAIR.

**R9 — REFHI/REFLO adjacency.** Not adjacent in general: all `lis` are hoisted
into a block, so the HI records come first and the LO records after. With one
address operand and no interleaved literal they *are* adjacent (`ar0`); with an
interleaved `li` they are not (`ar1i`, offsets 0 and 8). **The symbol order
inside the HI block and inside the LO block need not agree** (`ar1f`: HI order
`__real`, `sL`; LO order `sL`, `__real`).

**R10 — branch encoding.** The word stored at a `REL24` site at section offset
`k` is `0x48000000 | ((-k) & 0x03FFFFFC) | LK`. The obj never stores
`0x48000001`; it stores a self-relative displacement back to the section start,
i.e. an addend of 0 with the PC bias pre-applied. Verified on an ordinary
function too (`?h` at `k=0` → `48000000`; `?f` `bl` at `k=0xc` → `4bfffff5`).

**R11 — COMDAT aux `CheckSum`.** CRC-32, polynomial `0xEDB88320`, **init 0, no
final XOR**, over the section's raw data — for `.rdata`, `.pdata` and both
`.XBLD$W`. It is **0** for `.text$y?` and for every non-COMDAT section.

**R12 — string-literal COMDAT name.** `??_C@_0<L>@<H>@<text>@` where `<L>` is
the MSVC-mangled byte length **including the NUL**, and `<H>` is JamCRC of those
same bytes (CRC-32, poly `0xEDB88320`, **init 0xFFFFFFFF, no final XOR**)
rendered as 8 nibbles **most-significant first** over the alphabet `A`..`P`.

**R13 — `/GF` is the discriminator.** Without `/GF` (e.g. plain `/Ox`, `/Od`,
`/Ox /Gy`) string literals are pooled into one **non-COMDAT** `.rdata` named
`$SG<n>` that sits **before** the `.text` COMDATs. `/O1` and `/O2` imply `/GF`;
`/Ox` does **not**. The fixture-gate default flags therefore do not reproduce
the workload's shape for this class.

---

## Held-out predictions (falsifiable, none of these cells compiled yet)

| # | cell | prediction |
|---|---|---|
| H1 | `int f(int a){return a+1;}` **then** `static L sL("abc",0);` | sections after the shell: `.text`(f, SEL=1, `?f@@YAHH@Z` **EXTERNAL**), `.text$yc`, `.rdata`, `.bss`, `.CRT$XCU` — i.e. **ordinary function first, source order** |
| H1b | the same TU with the object **declared first** | `.text$yc`, `.rdata`, `.text`, `.bss`, `.CRT$XCU` — order tracks source |
| H2 | non-zero relocation addend (`static char buf[64]; static L sL(buf+32,0);`) | `PAIR.VirtualAddress` still equals the primary's; `PAIR.SymbolTableIndex` becomes **32** (the addend). **Registered alternative:** PAIR stays `0` and the addend rides in the `addi` immediate |
| H3 | three static objects sharing the literal `"abc"` | 3 × `.text$yc`, exactly **1** `.rdata` COMDAT (in the first group), `.bss` size **3**, `.CRT$XCU` size **0xc** with 3 relocs; `.bss` symbols reverse address order, `.CRT$XCU` symbols forward |
| H4 | `struct L{L(double);}; static L sL(1.5);` | `.rdata` COMDAT `__real@3ff8000000000000`, size **8**, characteristics **`0x40401040`** (ALIGN_8 by R4), plus an undefined `_fltused` |
| H5 | `char pad[64]` **and** a destructor | 10 sections; `.bss` size **0x40**, ALIGN_8 (`0xc0400080`); `.pdata` SEL=5 `Number=5`; `.text$yd` present |
| H6 | `static L sL("xyzzy",0);` | string COMDAT named exactly **`??_C@_05POJHDMIP@xyzzy?$AA@`**, `.rdata` size **6**, aux `CheckSum` **`0xb0aa62d3`**, ALIGN_4 |
| H7 | a 100-character literal (`"qqq…"`, 101 B with NUL) | name begins **`??_C@_0GF@ALHLJLME@`**, `.rdata` size **101**, aux `CheckSum` **`0x5dae28de`**, characteristics **`0x40401040`** (ALIGN_8, R4) |
| H8 | `struct L{L(const char*,int,int,int,int,int);}` with 5 ints | `.text$yc` size **0x28** (2 `lis` + 2 `addi` + 5 `li` + `b`), still **9** relocations |
| H9 | any held-out cell | every COMDAT aux `CheckSum` equals R11 applied to that section's raw bytes; every `.text$y?` CheckSum is 0 |

**Decline floor (restated).** If H1/H1b, H3 or H2 come out in a way that makes
the section/symbol order depend on something not present in the IL, this lane
declines the emit half and says so.

The `$SG<n>` / `$M<n>` / `$T<n>` counter values are **explicitly not predicted** —
they are a front-end name counter this lane has not characterized.
