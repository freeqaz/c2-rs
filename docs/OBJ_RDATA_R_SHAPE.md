# `.rdata$r` — the RTTI record graph, measured

**Status: SPECIFICATION ONLY, and RE-VERIFIED (§0.1). Nothing in `crates/` emits
`.rdata$r`; lane `w-rdata` deliberately did not add it to
`PORT_WRITER_SECTIONS` and lane `w-rtti` — briefed to add it — re-derived the
price and declined again** (§8.1, `rungs/2026-08-07-w-rtti.md`). See §9 for why,
and §9.1 for the guard that now makes the decision enforceable rather than a
matter of lane discipline. This file stands to
`.rdata$r` as [`OBJ_DATA_BSS_SHAPE.md`](OBJ_DATA_BSS_SHAPE.md) stood to
`coff/data.rs`: measure first, then write the writer against a caller.

> ## ⚠ AND IT IS WORTH `+0` TU MATCH — measured, not argued. See §10.
>
> The `+421` below is **factor C**, one of four terms. Closing the *entire*
> section ladder (C `169 → 871`) moves `A∧B∧C` by **0**, `A∧B∧C∧(D∨E)` by
> **0**, the FRONTIER by **0** and TU match by **0**. `w-reach`'s joint `+91`
> (board #302) is **reach**, not match. **§10 is the correction and it is the
> first thing to read on this page**, because every number between here and
> there is a factor-C number and factor C is not the binding constraint.

`.rdata$r` is the **top step of the greedy section ladder** and the single
largest one on the board *as a factor-C step*: over the 871 graded workload
TUs, adding it to the
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

### §0.1 RE-VERIFIED 2026-08-06 on 72 fresh objs — lane `w-rtti`, board #926

**Everything below was re-checked on cells this file was NOT derived from**, by
`work/w-rtti/verify.py` (which reads the COFF through `scripts/gt_dump.py`'s
`Obj` and gets its relocation names from `crates/c2-obj/src/reloc.rs`, so no
table is copied). The grid is 18 sources — templates with one and two
parameters, a class nested in a *class*, a four-deep chain, virtual inheritance
with a **single** base, two virtual bases, a secondary base with its own
virtuals, an anonymous namespace, a long template name, `dllexport`, three
classes in one TU, a two-level diamond, abstract-with-concrete-derived, virtual
destructors on both levels, `novtable` on the base only, and a member-pointer
member — frozen in `work/w-rtti/grid_list.txt` **before** anything was captured,
plus two known-answer controls that ARE this file's own cells. Captured at four
profiles (`/O1`, `/Od`, `/Ox`, `/GR-`) = **72 objs**, and at `/O1`
**139 `.rdata$r` sections + 35 `??_R0` `.data` COMDATs = 174 records**.

**21 of 23 claims HELD. Two did not, and one rule needed re-cutting:**

| claim | result at `/O1` |
|---|---|
| `.rdata$r` Characteristics `0x40301040` | **HELD 139/139** |
| `.rdata$r` COMDAT Selection 2 | **HELD 139/139** |
| every relocation is `ADDR32` | **HELD 268/268** |
| the aux `CheckSum` is `coff_checksum` | **HELD 174/174** |
| `??_R0` in `.data`, `0xC0301040`, Selection 2 | **HELD 35/35** each |
| `??_R0` size `8+strlen+1`, unpadded, name, spare 0 | **HELD 35/35** each |
| `??_R4` 20 B · `??_R3` 16 B · `??_R1` 28 B | **HELD** 27 / 35 / 42 |
| `??_R2` NULL-terminated · `size == 4*(n+1)` | **HELD 35/35** each |
| `??_R1`'s four fields are spelled in its own name | **HELD 42/42** |
| **`??_7` vftable: Selection 6, symbol `Value` 4** | **HELD 27/27** |
| **`??_8` vbtable: Selection 6, symbol `Value` 4** | **REFUTED 0/4** — see §3.1 |
| the DFS pre-order (§5) | **HELD 21/21** — after re-cutting the group, §6.1 |

The two corrections are §3.1 and §6.1. Neither changes a record's bytes; both
change what a writer would have to *do*, which is the only kind of correction
that matters here.

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

### §3.1 CORRECTION — `??_8` is NOT `??_7`, and Selection 6 is a `/GR` property

Two things the paragraph above gets wrong, both found by `w-rtti`'s fresh grid
(§0.1) and both stated as counts:

**(a) A `??_8` VBTABLE is Selection 2 and `Value` 0 — 0 of 4.** §6 below lumps
`??_7` and `??_8` together as *"the vftable/vbtable block"* and this paragraph
states one rule for the block. The rule is the **vftable's**: `??_7` holds
**27/27**, and every one of the four `??_8` vbtables in the grid
(`??_8Vd@@7B@`, `??_8R@@7B@`, `??_8Dia@@7BLft@@@`, `??_8Dia@@7BRgt@@@` — the
virtual-inheritance cells) reads Selection **2** and `Value` **0**. Its
Characteristics are `0x4030_1040`, 4/4, so the two records differ in exactly the
two fields a writer would have to set. The reason is the same one that explains
the `??_7` values: there is no complete-object locator at `vbtable[-1]`, so
there is nothing for the symbol to be displaced past.

**(b) Selection 6 / `Value` 4 is conditional on `/GR`.** At `/GR-` the same 27
vftables read Selection **2** and `Value` **0** — **0/27** on both, over the same
18 sources. Without RTTI there is no COL to sit at `vftable[-1]`, so the vftable
loses both the displacement and the LARGEST selection. This file only ever
measured `/GR` cells, so it recorded a `/GR`-conditional fact as an
unconditional one. It is not a live hazard — the workload is 100 % `/GR` — but a
writer keyed on "a vftable is Selection 6" is keyed on the wrong thing.

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

> **⚠ 2026-08-06 — "not recoverable" is true of the NAME and false of the
> STREAM, and this sentence sent the price down the wrong road.** All three are
> spelled literally in `.in`, alongside every other field of every record — see
> **§8.1**, which has the bytes. §4 is a correct measurement of *what a
> name-only synthesizer can do* and was read downstream as *what a reader must
> derive*. A reader does not have to derive any of it; it has to decode one more
> `.in` element tag. Board **#931**.

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

### §6.1 CORRECTION — the group is cut at the VFTABLE BLOCK, not at the `.text`

`w-rtti` (§0.1). The paragraph above and §5 are individually right and compose
wrongly: **§5's DFS is a per-group rule, and the group boundary is not the
`.text` COMDAT.** Measured, as the two counts a reader has to choose between:

| DFS model | `/O1` | `/Od` | `/Ox` |
|---|---:|---:|---:|
| one global walk rooted at every vftable in reverse section order | 16/18 objs | — | — |
| a walk per group, group cut at each `.text` COMDAT | 21/21 groups | 17/19 | 16/18 |
| **a walk per group, group cut at each VFTABLE RUN** | **21/21** | **22/22** | **21/21** |

The `.text` cut fails because **the `.text` COMDAT is not always there.**
`f11_three_classes.cpp` defines three polymorphic classes and their three
constructors; at `/O1` the obj carries three `.text` COMDATs and three
vftable+RTTI blocks, and at **`/Od` and `/Ox` it carries ONE `.text` COMDAT and
still three blocks** — `??0C2@@QAA@XZ` is emitted and `??0C3`/`??0C1` are not,
while all three vftables and all twelve records are. The block order is
unchanged across all three modes and is the order the constructors are
**defined** in (C2, C3, C1 — deliberately not the declaration order), so the
ordering key is the *function*, not the section it did or did not get.

§2's *"a `.rdata$r` TU always has at least one `.text` COMDAT"* survives this —
it is still true on every one of the 72 objs, and the four probes in
`work/w-rtti/probe/` looked for a counterexample on purpose and did not find
one (a namespace-scope object of a polymorphic class is **statically**
initialized here, vfptr relocation and all, and *still* drags the constructor
COMDAT in). What does not survive is the stronger reading that the `.text`
COMDATs and the RTTI blocks are in bijection.

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

**CONFIRMED 2026-08-06 on a fresh cell** (`w-rtti`, §0.1). `f14_vdtor_derived.cpp`
is exactly that shape — virtual destructors on base *and* derived — and it reads
**4 records / 1 vftable at `/O1` and at `/Ox`, and 8 records / 2 vftables at
`/Od`**. It is the **only** one of the eighteen sources whose record *count*
moves with the optimization mode; `f08_anon_namespace` and `f11_three_classes`
move their section counts at `/Od` and `/Ox` (§6.1) with their record counts
fixed. So the sentence above is right, and the shape it needs is narrower than
"base and derived": it needs the virtual **destructor** on both levels.

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

### §8.1 RE-DERIVED 2026-08-06 — still seven, all seven still unpaid, and item 2 is a DIFFERENT problem than this page says

Lane `w-rtti` was briefed to ship the writer and re-priced it first, at master
`9827bcf` — 20+ commits and several `c2-il` widenings after `w-rdata`. **The
count is still seven and not one of them has been paid.** The two that were
re-measured directly:

**Item 1 — `expr-op-0x27` still refuses, measured.** `c2rs census` on §2's own
source reads **`0/1 functions in class`**, blocking key `expr-op-0x27`, and
`c2rs diff` reads `ReferenceReplay=ByteExact (ref=1033B replay=1033B)
Port=NotImplemented`. The reference half is byte-exact and the port half is an
honest refusal, which is the boundary working.

**Item 2 is not "a graph reader" and not a derivation problem — it is a `.gl`
record kind `c2-il` cannot see AT ALL.** Measured with a throwaway spike over
`gl_data_objects_ordered`, with a positive control so that a zero is a reading
and not a broken probe:

| capture | records the `.gl` data reader returns |
|---|---:|
| `wsect_data_two.cpp` — the positive control | **2 of 2** |
| §2's minimal RTTI TU — 11 sections, 6 of them data | **0** |
| a namespace-scope object of a polymorphic class | **1 of 12** — the `$initializer$`, not the object and not one record |

**Zero of eleven.** Not the four `??_R*`, not the `??_R0` in `.data`, not the
vftable — and in the third row not even the plain statically-initialized
`?g@@3UA@@A`, because its initializer carries a **relocation**. So the reader is
blind to (a) every COMDAT record and (b) every initializer with a symbol
reference, and a `.rdata$r` writer needs both.

**And the good news, which is the part that makes the next lane cheaper: the
three layout integers §4 calls "the reader's job" are LITERALLY IN `.in`.** §4
scored a synthesizer that rebuilds records from mangled names and left
`??_R4.offset`, `??_R4.cdOffset` and `??_R3.attributes` as the irreducible
residue. They do not have to be derived. The minimal TU's `.in` spells every
record's contents element by element, including the ones §4 could not reach:

```text
  ??_R1 …  07 00 f7 09 00 · 02 f4 09 00 04 · 01 02 04 00 · 01 01 04 00
           · 01 01 04 80 ffffffff · 01 01 04 00 · 01 02 04 40 · 02 f5 09 00 04
                                    ^^ pdisp = -1        ^^ attributes = 0x40
  ??_R3 …  07 00 f5 09 00 · 01 02 04 00 · 01 02 04 00 · 01 02 04 01 · 02 f6 09 00 04
                                                          ^^ numBaseClasses = 1
  ??_R0 …  07 00 f4 09 00 · 02 f8 09 00 04 · 01 03 04 00 · 03 08 ".?AUA@@" 00
                             ^^ reloc to ??_7type_info@@6B@   ^^ tag 03, the name
```

Element tags `01` (scalar) and `03` (byte string) are **already read** by
[`crates/c2-il/src/func/ininit.rs`] and [`inlit.rs`]. The one missing tag is
**`02` — `<token> 00 <n>`, the address of another symbol** — which that module
names `InInitResidue::SymbolAddress` and refuses by design, citing
`OBJ_DATA_BSS_SHAPE.md` §8.6's *"unexercised"*. **That single element tag is the
relocation, and it is the same tag the third row of the table above trips over.**

So item 2's price is not a derivation engine. It is: teach `.gl`'s data-record
reader the COMDAT/section-name attribute, and teach `.in` element tag `02`. Both
are ordinary reader work with a graded consumer already in the tree
(`emit_data_obj`). **The count of seven does not change** — a cheaper fact is
still a fact — but the *shape* of the two binding ones does, and this page said
the wrong thing about the harder of them.

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

### §9.1 2026-08-06 — THE REMAINING HOLE IS CLOSED (board #301, lane `w-rtti`)

The second bullet above — *"a `Section { name: … }` literal in an emitter with
**no caller** would satisfy that test and still inflate C"* — is no longer open.
`coff::tests::every_production_emitter_has_a_lib_rs_caller` asserts that every
`pub fn emit_*` which exists in a **non-test** build is named by `lib.rs`, and
`emit_mvp_obj` (whose only caller was a test, and which §10 of
`rungs/2026-08-04-w-rdata.md` called *"luck, not a guarantee"*) is `#[cfg(test)]`
rather than exempted, because an allow-list is the thing that goes stale.

**Measured, not asserted** — `work/w-rtti/counterfactual.sh` breaks the tree in
each of the two ways that inflate C and reports which guard catches which:

```text
### BASE
  the_writer_vocabulary_…  ok        every_production_emitter_…  ok
### BREAK 1 — the constant only, no Section literal
  the_writer_vocabulary_…  FAILED    every_production_emitter_…  ok
### BREAK 2 — the constant PLUS an uncalled emitter that builds the Section
  the_writer_vocabulary_…  ok        every_production_emitter_…  FAILED
### RESTORE
  git status --porcelain crates/ = 0 path(s)
  the_writer_vocabulary_…  ok        every_production_emitter_…  ok
```

BREAK 2's row is the whole point: the vocabulary test is **green** over a
`.rdata$r` literal nothing calls, exactly as this section predicted, and the new
test is the only thing that says no.

**The guard caught its own author first.** The test read `pub fn emit_*_obj` for
one commit, and BREAK 2's emitter was named `emit_rtti_obj_counterfactual` —
which does not *end* in `_obj` — so the population excluded it and BREAK 2 passed
both tests on the first run. The suffix was an allow-list wearing a different
hat. It reads every `pub fn emit_*` now.

---

## §10 THE CORRECTION — this section is worth `+0` TU match, and so is the whole ladder

Added by lane **`w-joint2`** (`docs/rungs/_2026-08-05-w-joint2.md`, board
**#360**), which was briefed to build this writer *jointly* with the tag-0x10
ALIAS channel on the strength of `w-reach`'s **`+91`**. It measured the joint
that actually bounds TU match first, and the brief's premise does not survive
it.

### §10.1 Close the whole ladder and nothing downstream moves

`PORT_WRITER_SECTIONS` was widened transiently through the **real** `c2rs gap`
binary and reverted (`work/w-joint2/counterfactual.sh`, which asserts
`git status --porcelain crates/` is empty afterwards):

| writer vocabulary | C | `B∧C` | **`A∧B∧C`** | **`A∧B∧C∧(D∨E)`** | **FRONTIER** | **match** |
|---|---:|---:|---:|---:|---:|---:|
| today, 10 names | 169 | 151 | **27** | **8** | **19** | **8** |
| `+ .rdata$r` | **590** | **315** | **27** | **8** | **19** | **8** |
| `+ .text$yd` | 804 | 324 | **27** | **8** | **19** | **8** |
| `+ .xdata$x` — C **closed** | **871** | 338 | **27** | **8** | **19** | **8** |

The `590` / `315` row is a **known-answer control**: it is `w-rdata` §5's
writer-edit figure and `w-reach` §0.2's independent key reconstruction, and this
lane reproduces both by a third route. The `27` / `8` / `19` columns are the new
measurement — nobody had read them across the ladder.

### §10.2 Why — one count, and it is not a coincidence of this workload

> **`A∧B` = 27, and all 27 are already inside C. `|{A∧B} \ C| = 0`.**

There is not one TU on this workload whose emit set the port can reach and whose
symbols all bind that is blocked by a section name. A section name is therefore
**nobody's binding constraint**, and `+421` of C is `+0` of every joint C
appears in.

### §10.3 And the cap is structural: `|D∨E| = 10`

A byte-exact obj requires `A∧B∧C∧(D∨E)`. `D∨E` is the port's *codegen* class
(D per-function, E the whole-TU recognizer registry) and **reads nothing from
`PORT_WRITER_SECTIONS`**. It is **10**. So:

* **TU match is capped at 10** at every C, under every emit-set model, until the
  codegen class widens. Match is **8**. Total non-codegen headroom: **2 TUs**.
* Those 2 are `src/system/decomp_pch.cpp` and `src/system/math/vec.cpp` —
  board #213's divergence pair, `w-reach` §4.1's whole zero-codegen population.
  Both are `-BCD-`: they carry **no** out-of-vocabulary section, are already
  inside C, and fail **only A**. Section work is irrelevant to both.

### §10.4 The pair intersects the reachable population in the empty set

| | count |
|---|---:|
| TUs carrying `.rdata$r` | **676** |
| …of those, in `D∨E` (the port has an accepted route to the contents) | **0** |
| …of those, satisfying **A** | 1 |
| …of those, satisfying **B** | 181 |
| …of those, satisfying **`A∧B`** | **0** |
| of the 10 TUs in `D∨E`, how many carry `.rdata$r` | **0** |
| of the 10, how many carry **any** out-of-vocabulary section | **0** |

**0 of 676.** This writer's entire population is the population the port cannot
codegen a single function of, and the alias channel — which by construction only
ever touches TUs with a vftable, i.e. TUs with `.rdata$r` — inherits the same
zero. That is why the pair is super-additive *in reach* and still `+0` *in
match*: both halves live strictly outside `D∨E`.

### §10.5 What this does NOT say

1. **It does not say `w-reach` was wrong.** `+91` is `|{model exact} ∩ B∧C|`,
   correctly computed and correctly labelled *reach*; §6.1 of that page says in
   its own words that TU match does not move. The error was in reading it as a
   match figure downstream.
2. **It does not say this specification is wasted.** §1–§9 are a byte-level
   measurement of a real section and they will be needed the day the codegen
   class covers a polymorphic ctor. They are just not needed *first*.
3. **It does not price the codegen.** The 19-TU FRONTIER is unchanged by all of
   this and `w-conv` prices it at ≥ 6 independent refusals each.
4. **It is a statement about this workload at `/O1 /Oi /EHsc /GR`**, on the 871
   graded TUs, at `master bed9894`. `A∧B = 27` is a measured count, not a law;
   if it ever exceeds C, the ladder starts paying and this section must be
   re-measured rather than re-quoted.
