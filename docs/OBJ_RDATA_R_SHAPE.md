# `.rdata$r` — the RTTI record graph, measured

**Status: SPECIFICATION ONLY. Nothing in `crates/` emits `.rdata$r`, and lane
`w-rdata` deliberately did not add it to `PORT_WRITER_SECTIONS`** — see §9 for
why, and `rungs/2026-08-04-w-rdata.md` for the decision. This file stands to
`.rdata$r` as [`OBJ_DATA_BSS_SHAPE.md`](OBJ_DATA_BSS_SHAPE.md) stood to
`coff/data.rs`: measure first, then write the writer against a caller.

`.rdata$r` is the **top step of the greedy section ladder** and the single
largest one on the board: over the 871 graded workload TUs, adding it to the
writer's vocabulary would take factor **C** from **169 to 590, +421**. It
appears in **676 of 871** TUs as **24,163** sections (measured on
`work/w-bss/census/sections.jsonl`, `.XBLD$W:C1`/`:C2` normalized to
`.XBLD$W` — the census artefact `rungs/2026-08-04-w-gr.md` §2.1 labelled). The
other two names on the ladder block 243 (`.text$yd`) and 67 (`.xdata$x`) TUs;
the ladder is greedy and re-ranks after each step, so those two figures are
*blocked-TU counts*, not the ladder's own conditional step sizes.

## §0 How everything below was measured

* A **22-source hierarchy grid**, captured at the workload's own profile
  (`/GR /O1 /Oi /EHsc /GS- /c`) and at `/Od` and `/Ox`: **25 objs**.
* **38 real workload objs**, captured through `work/w-frame/refobj.sh` at
  `work/dc3-workload/flags.txt` verbatim — the 12 densest `.rdata$r` TUs in the
  census plus 18 sampled, spanning 4 to 156 `.rdata$r` sections per obj. **2,668
  `.rdata$r` sections and 902 `??_R0` `.data` COMDATs, 3,570 records total.**
* Everything read with `scripts/gt_dump.py` (whose `IMAGE_REL_PPC_*` names come
  from `crates/c2-obj/src/reloc.rs`, so the relocation types below are the
  hand-ported table's and not a second copy).

Every count in this file is from one of those two sets or from the census.
Nothing is extrapolated.

---

## §1 What mints it — the trigger, and two things that do not

**A vftable mints the records, and a vftable is emitted by whichever TU defines a
constructor or destructor body of a polymorphic class** — that body is what
writes the vfptr. `rungs/2026-08-04-w-gr.md` §4 established this and it
re-derives here, with the controls:

| probe | `.rdata$r` sections | note |
|---|---:|---|
| `struct A{A();virtual void f();int a;}; A::A(){}` | **4** | 11 sections total — the minimal shape (§2) |
| `struct A{A();virtual ~A();int a;}; A::~A(){}` | **4** | 13 sections: a *virtual* destructor drags in `??_G`, a frame and `.pdata` |
| `struct A{A();virtual void f()=0;}; A::A(){}` | **4** | abstract: the vftable slot relocates to `_purecall` |
| `struct __declspec(novtable) A{…}; A::A(){}` | **0** | **5 sections.** The ctor does not write the vfptr, so there is no vftable and no RTTI at all |
| `D* c(B* p){return dynamic_cast<D*>(p);}` | **0** | 7 sections: two `??_R0` in **`.data`**, nothing in `.rdata$r` |
| `const type_info* t(B* p){return &typeid(*p);}` | **0** | **5 sections** — not even a `??_R0` |

The last two are worth stating twice because the obvious model of RTTI is
exactly wrong: **`dynamic_cast` and `typeid` mint zero `.rdata$r`.** A writer
sized off them would emit nothing and read as success.

`__declspec(novtable)` is the sharpest control in the set — it is an XDK idiom,
it is *in* this workload, and it takes a TU that has every other RTTI ingredient
down to five sections.

---

## §2 The minimal shape — 11 sections

```cpp
struct A { A(); virtual void f(); int a; };
A::A(){}
```

```
   1 .drectve                                    (shell)
   2 .debug$S                                    (shell)
   3 .XBLD$W    __C2_11886                       (shell)
   4 .XBLD$W    __C1_11886                       (shell)
   5 .text      ??0A@@QAA@XZ            16 B  4 rel
   6 .rdata     ??_7A@@6B@               8 B  2 rel   the VFTABLE
   7 .rdata$r   ??_R4A@@6B@             20 B  2 rel   CompleteObjectLocator
   8 .data      ??_R0?AUA@@@8           16 B  1 rel   TypeDescriptor
   9 .rdata$r   ??_R3A@@8               16 B  1 rel   ClassHierarchyDescriptor
  10 .rdata$r   ??_R2A@@8                8 B  1 rel   BaseClassArray
  11 .rdata$r   ??_R1A@?0A@EA@A@@8      28 B  2 rel   BaseClassDescriptor
```

The constructor body is four instructions —
`lis r11,??_7A@@6B@@ha · addi r11,r11,…@l · stw r11,0(r3) · blr` — and the four
`.text` relocations are the REFHI/PAIR/REFLO/PAIR quad the port already models
for [`coff::DataRef`](../crates/c2-core/src/coff/function.rs), except that the
low half feeds a **store** rather than an argument register.

**A `.rdata$r` TU always has at least one `.text` COMDAT.** There is no
function-free `.rdata$r` obj, because the trigger *is* a function body. That is
the fact that puts this section out of reach of `emit_data_obj`, whose whole TU
class is "defines no functions".

---

## §3 The five record kinds, byte for byte

All fields **big-endian**. Offsets are within the record. Every relocation is
`IMAGE_REL_PPC_ADDR32` against the named symbol.

### `??_R0<type>@8` — TypeDescriptor — in **`.data`**, not `.rdata$r`

```
  +0x00  u32   pVFTable        -> ??_7type_info@@6B@   (reloc; undefined external)
  +0x04  u32   spare = 0
  +0x08  char[] decorated name, NUL-terminated
  size = 8 + strlen(name) + 1,  NOT padded
```

`name` is **`"."` followed by the record's own mangled middle**: strip the
`??_R0` prefix and the `@8` suffix from the symbol and prepend a dot.
`??_R0?AUA@@@8` -> `.?AUA@@`; `??_R0?AVA@@@8` (a `class`, not a `struct`) ->
`.?AVA@@`; `??_R0?AUA@N@@@8` -> `.?AUA@N@@`, 18 bytes; the 47-character
`??_R0?AUAVeryLong…@@@8` -> 55 bytes. Sizes 16, 18 and 55 are all in the grid,
which is what says the record is unpadded.

Characteristics `0xC030_1040` (CNT_INITIALIZED_DATA | ALIGN_4 | LNK_COMDAT |
READ | **WRITE**), Selection **2** (ANY). **It is writable**, which is why it
lands in `.data` and not `.rdata` — the name buffer is mutable.

### `??_R4<class>@@6B…@` — CompleteObjectLocator — 20 B

```
  +0x00  u32   signature = 0
  +0x04  u32   offset                     vftable's offset in the complete object
  +0x08  u32   cdOffset                   constructor-displacement offset
  +0x0c  u32   pTypeDescriptor  -> ??_R0…   (reloc)
  +0x10  u32   pClassDescriptor -> ??_R3…   (reloc)
```

`offset` is 0 for the primary vftable and the base's offset otherwise — `8` for
the second base of `struct D:B1,B2`, `0x10` for the third of three, `0x3c` and
`0x70` in real workload objs. `cdOffset` is 0 except under **virtual**
inheritance, where it is 4.

### `??_R3<class>@@8` — ClassHierarchyDescriptor — 16 B

```
  +0x00  u32   signature  = 0
  +0x04  u32   attributes                 1 = multiple inheritance, |2 = virtual
  +0x08  u32   numBaseClasses             INCLUDING the class itself
  +0x0c  u32   pBaseClassArray -> ??_R2…   (reloc)
```

`numBaseClasses` counts self: 1 for a standalone class, 2 for `D:B`, 3 for
`D:M:B`, 5 for the diamond. `attributes` is 0 for single inheritance, 1 for MI,
**3** for the diamond (MI | VI).

### `??_R2<class>@@8` — BaseClassArray — `4 * (n + 1)` B

```
  +0x00  u32[n]  -> ??_R1…                (n relocs, one per entry)
  +4n    u32     = 0                      NULL terminator
```

`n == ??_R3.numBaseClasses`, so the array size and the descriptor's count are two
readings of one number and a writer that got them from two places could disagree.
Measured 8 B (n=1), 12 B (n=2), 16 B (n=3).

### `??_R1<pmd><class>@8` — BaseClassDescriptor — 28 B

```
  +0x00  u32   pTypeDescriptor -> ??_R0…   (reloc)
  +0x04  u32   numContainedBases          bases BELOW this entry in the array
  +0x08  i32   PMD.mdisp
  +0x0c  i32   PMD.pdisp                  -1 when not virtual
  +0x10  i32   PMD.vdisp
  +0x14  u32   attributes                 0x40 = BCD_HASPCHD (always set here)
  +0x18  u32   pClassDescriptor -> ??_R3…  (reloc)
```

**`mdisp`, `pdisp`, `vdisp` and `attributes` are all spelled in the symbol
name**, in that order, in MSVC's number mangling (`A@` = 0, `0`..`9` = 1..10,
`?` = negate, `A`..`P`+`@` = hex nibbles):

| symbol | mdisp | pdisp | vdisp | attrs |
|---|---:|---:|---:|---:|
| `??_R1A@?0A@EA@A@@8` | 0 | −1 | 0 | 0x40 |
| `??_R17?0A@EA@B2@@8` | **8** | −1 | 0 | 0x40 |
| `??_R1BA@?0A@EA@B3@@8` | **0x10** | −1 | 0 | 0x40 |
| `??_R1A@A@3FA@B@@8` | 0 | **0** | **4** | **0x50** |

The last row is the virtual base in the diamond. `numContainedBases` is the only
one of the seven fields **not** in the name.

### The aux `CheckSum`

**Every one of the 3,570 records reproduces under the port's existing
[`coff_checksum`]** — reflected CRC-32, polynomial `0xEDB88320`, init **0**, no
final inversion (`crates/c2-core/src/coff/checksum.rs`). 3,570 of 3,570, over
both the grid and the workload objs. `??_R4` and `??_R2` read `0x00000000` not
because the field is suppressed but because their non-relocated bytes are all
zero and that CRC of zeros is zero — a distinction that matters, because a writer
that special-cased "zero for a fully relocated record" would be right for the
wrong reason and wrong on `??_R3`.

### Characteristics and Selection

Every `.rdata$r` section: `0x4030_1040` — CNT_INITIALIZED_DATA | ALIGN_4 |
LNK_COMDAT | READ. Selection **2** (ANY). Uniform across all 2,668 workload
sections and all grid objs; no alignment or selection variation was observed.

The **vftable**'s own `.rdata` is `0x4030_1040` too but Selection **6**
(LARGEST), and its defining symbol's `Value` is **4**, not 0: the COL pointer
occupies `vftable[-1]` and the symbol names the first virtual slot.

---

## §4 The bytes are DERIVABLE — 3,337 of 3,570 from names alone

Scored by `work/w-rdata/synth.py`, which builds each record from nothing but the
mangled symbol names and the reference's relocation target list, then compares
bytes and relocation offsets:

```
  grid (25 objs, 203 records)      194 exact   9 wrong
  workload (38 objs, 3,570)      3,337 exact 233 wrong
```

**Every one of the 242 misses is one of three fields** — `??_R4.offset`,
`??_R4.cdOffset`, `??_R3.attributes`. Supplying those three from the reference:

```
  grid        203 of   203 exact
  workload  3,570 of 3,570 exact
```

That is the price of the record graph, stated as a number: **three class-layout
integers per class**, and everything else follows from the names. Those three are
what c1xx computes and hands to c2; they are not recoverable from a mangled
symbol, which is why they are the reader's job and not the writer's.

---

## §5 The emission order is ONE rule — a DFS, and it explains the symbol table too

**Sections appear in DFS pre-order over the relocation graph**, rooted at the
vftables in forward base order, with each node emitted on first visit.

```
  place the vftable/vbtable block first  (§6)
  then, for each vftable in forward base order:
      visit(v):  emit v if unemitted; then visit each reloc target of v,
                 in ascending relocation offset
```

Worked on `struct D:B` — `??_R4D` refs `??_R0D` then `??_R3D`; `??_R3D` refs
`??_R2D`; `??_R2D` refs `??_R1D`, `??_R1B`; `??_R1D`'s targets are both already
out; `??_R1B` opens `??_R0B` and `??_R3B`, which opens `??_R2B`:

```
  R4D  R0D  R3D  R2D  R1D  R1B  R0B  R3B  R2B      <- observed, exactly
```

**Measured: exact on 25 of 25 grid objs and 38 of 38 real workload objs.** No
other rule was needed for any obj — not for multiple inheritance, not for the
diamond, not for a 156-section TU.

**And the same walk orders the undefined externals in the symbol table.** In the
minimal obj, `??_7type_info@@6B@` sits immediately after `??_R0`'s group while
`?f@A@@UAAXXZ` — referenced by the *vftable*, the DFS root — is the **last**
symbol in the obj. That is not two rules: the vftable's children are
`[??_R4, ?f]`, the DFS drains the whole `??_R4` subtree first, and `?f` is
therefore visited last. One walk, both orders.

The symbol table otherwise follows section order, section symbol + aux then the
defining EXTERNAL, exactly as `coff/data.rs` already does it.

---

## §6 The vftable block, and where `.pdata` goes

The vftable/vbtable sections are **not** in the DFS positions their roots occupy.
They come out as a block, immediately after the emitting function's `.text`, and
**within the block they are reversed** relative to the DFS root order:

| probe | vftable block, in section order | DFS root order |
|---|---|---|
| `D : B1, B2` | `??_7D@@6BB2@@@`, `??_7D@@6BB1@@@` | B1 then B2 |
| `D : B1, B2, B3` | `??_7…B3`, `??_7…B2`, `??_7…B1` | B1, B2, B3 |
| diamond `D : L, R` (virtual `B`) | `??_7D@@6B@`, `??_8D@@7BR@@@`, `??_8D@@7BL@@@` | — |

So `??_7` (vftables) precede `??_8` (vbtables) and each group is internally
reversed. The trailing `??_R4` of a secondary base is consequently the **last**
RTTI section in the obj.

`.pdata` sits between the vftable block and the first `??_R4` when the emitting
function is framed (`struct D:B` with a defined ctor, `struct A` with a virtual
destructor). It belongs to the `.text` COMDAT, not to the RTTI graph.

**Two independent classes in one TU do not interleave**: each function's
`.text`, vftable block and complete RTTI graph are emitted before the next
function's, and the order follows the function emission order — reversing the
two definitions in the source reverses the two blocks in the obj, with an
unrelated plain function keeping its own position on either side.

---

## §7 `/Od` emits TWO vftables where every optimized mode emits ONE

Board **#295** / `rungs/2026-08-04-w-gr.md` §2.2. `/Ox`, `/O1`, `/O2` and
`/Ox /Gy` all emit one vftable and 4 records for

```cpp
struct A{A();virtual ~A();virtual int f();int a;};
struct S:A{S();virtual ~S();virtual int f();int s;};
A::~A(){}
S::~S(){}
```

and `/Od` emits **two** and **8**. Reproducer committed as sweep cases
`91-rtti-vftable-0144/0145`. It did **not** reproduce on this lane's simpler
grid — `g01`/`g02` are byte-stable across `/Od`, `/O1` and `/Ox` in record count
— so the divergence needs the base-and-derived shape specifically, and a writer
keyed on "one vftable per class defined here" is right at every optimized mode
and wrong at `/Od`.

---

## §8 What a writer needs that does not exist yet

Priced against the **minimal 11-section obj of §2**, which is the cheapest case
in the class. Independent facts, **no discount applied** — the standing
methodology of `rungs/2026-08-04-w-conv.md`:

| # | fact | crate |
|---:|---|---|
| 1 | the vfptr-store leaf body class — `c2rs census` calls it `expr-op-0x27` and refuses the parse | **`c2-il`** |
| 2 | a reader for the `??_R*` record graph: the symbol set, the three layout integers of §4, and the base array's order | **`c2-il`** |
| 3 | codegen for `lis/addi/stw rD,0(r3)/blr` — a `DataRef` whose low half feeds a **store**, which `data_refs_of` currently refuses (it searches for `addi rD,r11,0` into an *argument* register) | `c2-core` |
| 4 | the `.rdata$r` / `.data`-COMDAT `Section` emitter and its relocations | `c2-core` |
| 5 | the DFS emission order of §5, over sections **and** undefined externals | `c2-core` |
| 6 | the vftable `.rdata` COMDAT: Selection 6, symbol `Value` 4, two relocations | `c2-core` |
| 7 | the `??_7type_info@@6B@` undefined external | `c2-core` |

**Seven, and the first two are in another crate.** The standing decline clause —
*a frontier TU at ≥ 4 independent refusals is not a target* (board #269) — fires
on this at 7, and the count is a **lower bound**: it stops at the minimal case
and does not price the virtual-destructor shape, which adds the compiler-generated
`??_G` scalar deleting destructor (a framed body with a conditional and a call to
`??3@YAXPAX@Z`), the `??_E` alias emitted as a `SECTION`-class symbol with an aux
record, and a `.pdata`.

**Items 1 and 2 are the binding ones**, and they are not in this lane's seam.
Reported to lane `w-vocab` rather than implemented.

---

## §9 Why `PORT_WRITER_SECTIONS` was NOT extended

Factor **C** is `obj section set ⊆ PORT_WRITER_SECTIONS`. The constant is
therefore a claim about the writer, and adding a name to it moves C by 421
*whether or not anything emits that name*. `gap.rs`'s own control says so:
`factor_control_on_match_tus` catches a name that is **missing** (a matching
obj's section falls outside C and the control goes red) and its doc states
plainly that it **cannot** catch the opposite.

Two things stop that hole from being open:

* `crates/c2-core/src/coff/tests.rs`'s
  `the_writer_vocabulary_is_every_section_name_this_file_emits` reconciles the
  constant against the `Section { name: … }` literals in every `coff/*.rs` that
  can build one. **A name with no `Section` literal turns it red** — verified by
  this lane's counterfactual (`rungs/2026-08-04-w-rdata.md` §5).
* The remaining hole is one level up: a `Section` literal in an emitter **with
  no caller** would satisfy that test and still inflate C. That is exactly what
  `container::bss_deferred_layout` was, and board **#278** deleted it. This lane
  declined to create a second one.

So the honest order is: this document, then the `c2-il` reader, then the writer
**with a caller**, then the constant. Not the constant first.
