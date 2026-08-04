# C2_MAP — a navigational map of `c2.dll`

> **PROVENANCE — DISASSEMBLY-DERIVED.** This file and everything else under
> `docs/whitebox/` was obtained by statically disassembling Microsoft's
> `c2.dll`. It is a **navigation aid**: it answers *"where in the binary is the
> code that decides X?"* so you can go look. It is **not** a source of values for
> the port. Nothing here may be copied into `crates/` without first adding a row
> to [`DISCLOSURE.md`](DISCLOSURE.md) naming the address it came from.
>
> The one correctness rule is unchanged: **the real `c2` under wibo plus a
> byte-exact obj compare is the sole judge of the port.** A white-box reading is
> a hypothesis; only the oracle settles anything.

Reproduce it from a clean checkout with [`C2_MAP_METHOD.md`](C2_MAP_METHOD.md).

---

## 0. The binary in one screen

| | |
|---|---|
| file | `compilers/X360/16.00.11886.00/c2.dll` |
| sha256 | `c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258` |
| size | 1 347 072 bytes |
| format | **PE32, x86** — the *target* is PowerPC, the compiler itself is a Win32 x86 DLL |
| imagebase | `0x10b00000` |
| link timestamp | `0x4C7F5BC4` = **2010-09-02 10:29:24 UTC** (the file's *mtime* is 2012-06-19 — the XDK repack date, not the build date) |
| sections | 4: `.text` `.data` `.rsrc` `.reloc` |
| functions found | **4916** (Ghidra 12.1.2 auto-analysis; all 4916 decompiled, **0 failures**) |

### Address-space layout

```
0x10b01000  .text   vsize 0x12cc7c  chars 0x60000020  CODE|EXEC|READ
            ├─ 0x10b01000 .. 0x10b0147c   import address table (3 runs: 55 + 103 + 104 entries)
            ├─ 0x10b01480 .. 0x10b266d0   POOLED READ-ONLY DATA — ~153 KB, no functions
            │                             (.rdata was merged into .text at link time;
            │                              this is where the string pool and const tables live)
            └─ 0x10b266d0 .. 0x10c2dc7c   CODE — 4916 functions, 1 033 556 bytes covered
                                          (84% of .text; the remainder is interleaved
                                           jump tables and const data)
0x10c2e000  .data   vsize 0x042750  chars 0xc0000040  INITIALIZED|READ|WRITE
0x10c71000  .rsrc   vsize 0x0003e8  — version info only; no message table
0x10c72000  .reloc  vsize 0x00ca8a
```

The two facts worth carrying away: **`.rdata` is merged into `.text`**, so a
string at a `.text` address is data, not code; and **code starts at
`0x10b266d0`**, so any address below that is data by construction.

### Is it stripped? — **yes**, and `ROADMAP.md` §9 is wrong about this

`docs/ROADMAP.md` §9 (~line 4074) asserts `c2.dll` "is **not a stripped build**".
The lane was told to verify rather than inherit that claim. **It is false.**

| probe | result |
|---|---|
| COFF symbol table | `PointerToSymbolTable` = 0, `NumberOfSymbols` = **0** |
| exports | **4**, and they are the pass ABI, not internals — see §2 |
| RTTI type descriptors | **0** strings matching `.?AV`/`.?AU` — built `/GR-` |
| CodeView debug directory | one `RSDS` entry, GUID `37d5710da3100542a380297384c16f8d`, age 19, path **`c2.pdb`** — and no `c2.pdb` ships in the XDK |
| POGO / VC_FEATURE debug entries | none |

What §9 actually established is that c2 is unusually **talkative** — `/FAsc`
makes it narrate its own output — which is a different and still-valuable
property. It is not evidence about symbols. The distinction matters because
"not stripped" makes white-box work look far cheaper than it is: every name in
this map had to be *earned*, and 4916 minus a few dozen of them are still
`unknown`.

---

## 1. What links to what — the subsystem partition from imports

The import table partitions the binary before a single instruction is read. c2
links against eight DLLs; four of them are load-bearing and name a subsystem
outright.

| DLL | imports | what it tells you |
|---|---:|---|
| `MSVCR100.dll` | 103 | CRT. Note `qsort`, `__unDName` (name **undecoration**), `_CIlog`/`_CIpow`. |
| `KERNEL32.dll` | 55 | Includes `CreateFileW`, **`CreateFileMappingW`**, `MapViewOfFile`, `UnmapViewOfFile` — **the IL bundle is memory-mapped, not streamed.** Also `CreateSemaphoreW`/`GetSystemInfo` (c2 is internally parallel). |
| `MSDISXXX.DLL` | 6 | Microsoft's disassembler: `DIS::PdisNew`, **`DIS::CchFormatInstr`**, `PfncchfixupSet`, `PfncchaddrSet`, `PvClient`, `PvClientSet`. This is how the `/FAsc` `.cod` listing gets its instruction text — c2 assembles bytes, then disassembles them back to print them. |
| `msobjXX.dll` | **1** | `objf::ObjectCode::FCreateFromBytesW` — see §3. |
| `MSPDBXX.DLL` | 7 | `PDBOpen2W`, `PDBOpenStream`, `StreamRead`, `SigForPbCb` — c2 *reads* a PDB (for `/Zi` type info), it does not write one. |
| `pgodb100.dll` | 104 | Profile-guided optimization database. **104 imports is more than any other non-CRT dependency** — a large, self-contained subsystem that is almost certainly *dead* for this project's workload (no PGO in the DC3 build). Carving it out is pure profit: see the `pgo-client` cluster. |
| `USER32.dll` | 1 | `wsprintfW`. |
| `OLEAUT32.dll` | 2 | by ordinal (#2, #6) — `SysAllocString`/`SysFreeString` family. |

**Diagnostic text is not in `c2.dll`.** c2 holds only error *numbers*; the
strings live in `1033/clui.dll`'s message table (`unrecognized flag '%s' in
'%s'` is there, not here). So hunt diagnostics by the **immediate** — `C1007` is
`0x3EF`, `C1083` is `0x43B` — never by the string.

---

## 2. Entry points

`c2.dll` exports exactly four symbols. Everything the driver can ask of the back
end goes through them.

| ordinal | RVA | VA | symbol |
|---:|---|---|---|
| 1 | `0x000ec40a` | `0x10bec40a` | `DllGetObjHandler` |
| 2 | `0x000ec2ac` | `0x10bec2ac` | `_AbortCompilerPass@4` |
| 3 | `0x000ebffd` | `0x10bebffd` | `_InvokeCompilerPass@12` |
| 4 | `0x000ec133` | `0x10bec133` | `_InvokeCompilerPassW@16` |

`cl.exe` drives the back end as pass **`p2`** with an argv like:

```
c2.dll -il <base> -typedil -Fo<out>.obj -Bd -Og -Ob2 -FAasc -Fa <file>.cod
```

<!-- SUBSYSTEMS-START -->
---

## 3. The translation-unit partition — the spine of this map

Everything below §3 is subordinate to this section. Every other labeling method
in this directory is a heuristic that needs a control. **This one is c2's own
bookkeeping.**

c2's internal-compiler-error path prints `compiler file '%s', line %d` (C1001),
so the binary carries **its own original source file names** as literals: 53
paths under `e:\bt\278379\vctools\compiler\be\…`, of which 52 are source files
and one is `c2.pdb`. Because the linker laid the object files down in order, the
addresses of the code referencing each file's string partition the address space
into **contiguous ranges owned by known original source files**. That converts
"cluster by call-graph shape and hope" into reading module boundaries off the
binary.

The table is [`c2_tus.tsv`](c2_tus.tsv), generated by
[`scripts/build_tus.py`](scripts/build_tus.py).

### 3.1 The confidence metric for the whole map

Two numbers. Quote them whenever you quote a range attribution; they tell you
how far to trust it.

| metric | value |
|---|---|
| **overlap rate** | **1 of 51 adjacent pairs = 1.96%** — and the sole overlap is `dbgcpp.h` inside `dbg.cpp`, a header nested in its own including TU. Between *distinct translation units* the overlap is **0 of 50 = 0.0%**. All-pairs: 1 of 1326. |
| **gap coverage** | **28.1% of bytes** (294 679 / 1 046 846) and **29.2% of functions** (1435 / 4916) lie inside an anchor range. The other **72.8%** is in gaps. |

**Read the second number honestly: it is the weak one.** `anchor_start` /
`anchor_end` bracket the *first and last function containing an ICE site* for
that file — so **22 of the 52 files have a zero-width anchor** (a single
function; 20 files have exactly one ICE site). Their "range" is a point, not an
interval.

The distinction that matters downstream:

* **Certain attribution — 28.1%.** Inside an anchor range, both endpoints assert
  the same file. This is as good as a symbol.
* **Modelled attribution — 96.7% of all functions** fall inside the anchored
  span `0x10b28586..0x10c27ec4`, so under the link-order tiling model nearly the
  whole binary is attributable. But **72.8% of those bytes rest on the ordering
  assumption alone**, and *a file with no ICE site at all is invisible* — it is
  silently absorbed into its predecessor's gap.

A worked example of exactly that failure, found by a control: c2's string hash
`0x10b8a01b` sits in the gap after `misc.c`, in a file that has no ICE site. Any
tool that reports "`10b8a01b` is in `misc.c`" is over-claiming. **Gap
attributions are hypotheses; in-range attributions are facts.**

### 3.2 The link-order model, tested against a null — this is the strong result

The partition was built from ICE-site addresses. The *file names* and their
*directory prefixes* were never used to build it, so they are an independent
test. Sort the 52 files by `anchor_start` and read off the names:

| run | len | directory | files |
|---:|---:|---|---|
| 1 | 33 | `be\p2` | `coff.c coffemit.c color.c dag.c factor.c fg.c fpmodel.c getattr.c globdf.c globlopt.c globopt.c globregs.c hash.c inline.c list.c ltcg.c lur.c main.c misc.c mod.c optimize.c p2pragma.c p2symtab.c pogocg.c pogoinline.c pogoopt.c ptinl.c reader.c regasg.c sizeopt.c ssa_seh.c stack.c tuple.c` |
| 2 | 2 | `be\p2` | `ehexcept.c except.c` |
| 3 | 1 | `be\p2` | `emit.cpp` |
| 4 | 3 | `be\p2` | `dbg.cpp dbgcpp.h dll.cpp` |
| 5 | **6** | `be\p2\ppc` | `cgintrin.c code.c inlnasm.c lower.c mdlist.c mdmisc.c` |
| 6 | **5** | `be\common` | `error.c get_err.c getflags.c ioin.c vlines.c` |
| 7 | **2** | `be\p2\smd` | `lowersmd.c smdmisc.c` |

Seven maximal ascending runs, and **every run is directory-pure**. Better: the
string pool contains 39 `be\p2` files, 6 `be\p2\ppc`, 5 `be\common`, 2
`be\p2\smd` — and **each of the three non-`p2` directories appears as one
complete, contiguous, fully alphabetical block** (6/6, 5/5, 2/2), while runs 1–4
are all and only the 39 `p2` files. Runs 3 and 4 are one transposition from
being a single alphabetical run of the four C++ files.

Against a null of random ordering:

| test | observed | null | probability |
|---|---|---|---|
| number of ascending runs | 7 | 26.5 expected | **P(runs ≤ 7) = 1.5 × 10⁻²⁵** (exact, Eulerian) |
| longest run in sorted order | 33 files | — | **1/33! = 1.2 × 10⁻³⁷** |

The link-order hypothesis is not marginally supported; it is supported to the
point where the interesting question is what the *four* `p2` runs mean (four
link inputs — most likely a C group, a second C group, and the C++ objects),
not whether ordering holds.

### 3.3 Line numbers — within-file ordering, and it is the shakier half

The ICE macro pushes a line-number immediate alongside the file pointer, so
`(file, line)` pairs order sites *within* a file. Two macro shapes exist and
they put the line in **different places** — the `stdcall` shape pushes the line
*first*, so a naive "nearest immediate" rule returns the diagnostic *code*
instead. That was got wrong once; the fix is documented in `build_tus.py`.

**Globally: 93 in order, 46 inversions = 66.9% monotone.** That is positive but
far weaker than the ordering result above, and it is *not uniform*:

| file | monotone | inversions | sites |
|---|---:|---:|---:|
| `coffemit.c` | 21 | 2 | 27 |
| `p2symtab.c` | 11 | 4 | 16 |
| `reader.c` | 5 | 5 | 12 |
| `dbg.cpp` | 5 | 5 | 11 |

So line order is usable for sub-file navigation in dense, well-behaved files
(`coffemit.c` at 91% monotone) and **should not be trusted in `reader.c` or
`dbg.cpp`** (50%). Function reordering by the optimizer is the obvious cause and
we have not tried to separate it from mis-recovered lines. `hash.c`, `main.c`,
`p2pragma.c`, `except.c`, `emit.cpp` and `dll.cpp` have ≥3 sites and **zero**
inversions.

<!-- SUBSYSTEMS-END -->

---

## 3A. The 53 file names — the tier-1 artifact

**These names come from `strings`.** They are an observable output of the black
box, on the same footing as the obj, the `/FAsc` listing and the diagnostic
text, and `docs/ROADMAP.md` §9.8 already blesses that class. **This list on its
own incurs no white-box debt** — see [`DISCLOSURE.md`](DISCLOSURE.md) for why
the *addresses* are a different tier.

Build root `e:\bt\278379\vctools\compiler\be\`. 52 sources + one `.pdb` path.

| directory | n | files |
|---|---:|---|
| `p2\` | 39 | `coff.c` `coffemit.c` `color.c` `dag.c` `dbg.cpp` `dbgcpp.h` `dll.cpp` `ehexcept.c` `emit.cpp` `except.c` `factor.c` `fg.c` `fpmodel.c` `getattr.c` `globdf.c` `globlopt.c` `globopt.c` `globregs.c` `hash.c` `inline.c` `list.c` `ltcg.c` `lur.c` `main.c` `misc.c` `mod.c` `optimize.c` `p2pragma.c` `p2symtab.c` `pogocg.c` `pogoinline.c` `pogoopt.c` `ptinl.c` `reader.c` `regasg.c` `sizeopt.c` `ssa_seh.c` `stack.c` `tuple.c` |
| `p2\ppc\` | 6 | `cgintrin.c` `code.c` `inlnasm.c` `lower.c` `mdlist.c` `mdmisc.c` |
| `common\` | 5 | `error.c` `get_err.c` `getflags.c` `ioin.c` `vlines.c` |
| `p2\smd\` | 2 | `lowersmd.c` `smdmisc.c` |
| — | 1 | `…\p2\c2\obj\i386\c2.pdb` (build output path, not a source) |

Reading the names alone already answers questions the project had open: there is
a `p2\smd\` directory (a second machine-description layer beside `ppc\`); EH is
split `ehexcept.c` / `except.c`; the debug writer is C++ (`dbg.cpp`) while
almost everything else is C; and **`coff.c` and `coffemit.c` are separate
files**, which is §4A.

---

## 4A. `coffemit.c` — the densest file, and the one on the critical path

27 ICE xrefs, the most of any file, and 91% line-monotone — the best-behaved
region in the binary. The name points at the project's tightest constraint, and
three open questions land in or beside it.

**That `coff.c` and `coffemit.c` are separate files is itself the finding.** It
predicted a model/reader layer distinct from the writer, and the split is clean:

| | `coff.c` `0x10b28586..` | `coffemit.c` `0x10b290dc..0x10b2b0dd` |
|---|---|---|
| role | opens the obj, owns the section model | **every `fwrite`** |
| routines | `10b281af` `10b281f7` `10b28261` `10b28304` `10b28586` `10b287b8` `10b2888e` `10b289fd` | `10b291b1` `10b291de` `10b2921c` `10b29268` `10b2948b` `10b2a265` `10b2a936` `10b2ad50` `10b2ae0e` `10b2b02d` `10b2b0dd` |

### The writer, field by field

* **`FUN_10b2b0dd`** — `IMAGE_FILE_HEADER` (and the 56-byte BIGOBJ variant,
  identified by the documented ClassID GUID `{D1BAA1C7-BAEE-4BA9-AF20-FAF66AA4DCB8}`
  at `.data 0x10b01be4`). `SizeOfOptionalHeader = 0`, `Characteristics = 0x0180`.
  `Machine` is **not** an immediate here: `FUN_10b28586` sets `0x1F2`
  (`POWERPCFP`), or `0x0C13` under LTCG.
* **`FUN_10b2b02d`** — the 40-byte `IMAGE_SECTION_HEADER`.
  `PointerToLinenumbers` and `NumberOfLinenumbers` are hard `0`;
  `NumberOfRelocations = min(n, 0xFFFF)` with `IMAGE_SCN_LNK_NRELOC_OVFL`
  (`0x01000000`) OR'd in above `0xFFFE`.
* **`FUN_10b2a936`** — the 18-byte `IMAGE_SYMBOL` writer, including the `.file`
  record (`SectionNumber = 0xFFFE`, `StorageClass = 0x67`).
* **`FUN_10b2948b`** — the 18-byte auxiliary section-definition record.
  **`Number` and `Selection` are written only when `Characteristics & 0x1000`**,
  i.e. only for COMDATs.
* **`FUN_10b2ad50`** — the COFF long-name encoder: `≤8` bytes inline
  zero-padded, else `'/'`+decimal string-table offset, else `"//"`+6 base64
  characters above `9999999`.
* **`FUN_10b281af`** — the sole `_time64` caller, installed as the
  `TimeDateStamp` callback, and it **returns 0 when `DAT_10c2ead8` is set**.
  That global is the `-Brepro` flag: c2 already implements the determinism the
  harness gets by zeroing file offsets 4..8.

### The three open questions, and where each actually lives

1. **Section selection and naming (factor C)** — **NOT in `coffemit.c`.**
   `FUN_10b982d6` is the single place a section kind becomes (name,
   Characteristics), and it is in **`p2symtab.c`**; the section *constructors*
   (`FUN_10be7473` non-COMDAT, `FUN_10be74cf` COMDAT) are in **`emit.cpp`**.
   The 13-name vocabulary and the exact characteristic words are in §4B.
2. **COMDAT and symbol emission order (R6)** — the writer is
   `FUN_10b2a936`, driven by `FUN_10b8303c(g_symList@0x10c2e234, …)`, so the
   *iteration order of that list* is the whole question. **Unresolved**; see §7.
3. **The `.bss` object-address permutation** — **settled, and it was never
   c2's.** See §4C.

### 4B. The `.bss`/`.data` decision — factor C's +402-TU item

`FUN_10b9a143` (`p2symtab.c`) assigns the section, after `FUN_10b98457`
normalises the storage class. The decisive instruction is:

```
10b9849f:  f7 41 20 80 04 00 00    test  DWORD PTR [ecx+0x20],0x480
10b984a6:  74 0c                   je    0x10b984b4          ; -> .bss
```

**The predicate is "does this symbol carry initializer bytes as it reaches
c2", not a zero-scan.** Bit `0x80` at `sym+0x20` is set by `FUN_10b805b3`
(`SetInitialData`: `flags |= 0x180; size → +0x1c;` then `memcpy` the bytes).
**No code anywhere in `.text` clears it** — there is no `and […+0x20], ~0x80`.
So if `static int x = 0;` lands in `.bss`, the zero-folding happened in **c1xx**
and c2 simply never received bytes. High confidence on the c2-side predicate;
**medium** on attributing the folding to c1xx, which was not verified there.

An explicit IL section (`#pragma data_seg`, `__declspec(allocate)`) sets
`sym[0xC]` and **short-circuits all of the above**.

### 4C. The `.bss` permutation — REFUTED as a hash, and it is c1xx's

Lane w-bss is bounding this from the outside; this lane was told it might be the
faster route, and it was, but **not by finding a hash — by proving there isn't
one.**

For N=6 the observed order `s6 s4 s3 s5 s1 s2` was brute-forced against 9 name
decorations × every modulus 2..8191 × every `(h>>s)&mask` for `s≤28`,
`mask≤16` bits × {ascending, descending} × {FIFO, LIFO}: **0 hits.** Under c2's
own hash the six names are affinely related, so any mask/modulus key yields an
ordering of the form `N XOR k`, and none of the eight is the observed one.

The measured rule instead, 5/5 plus a held-out control with different names and
lengths:

> **`.bss` ascending address = exact REVERSE of the IL `.gl` record order** for
> objects **with** a dynamic initializer; **= `.gl` record order** for plain
> zero-init statics; and in a mixed TU every non-dyninit object precedes every
> dyninit one.

Mechanism: no initializer → **eager**, allocated as the record streams past
(`10b9b161`/`10b9b6a4` → `10c27b56` → bump allocator `10c2757d`, align 8 with a
`+(8−size)` fixup for sizes 1/2/4). Has one → **deferred**, head-inserted onto
`DAT_10c2f064` and drained head-first by `10b99093`. Head insert + head-first
drain = reversal. The first-touch flag `0x800` in `10c27b56` is why the two
groups never interleave.

**The residual permutation is the front end's.** The `.gl` order for N=6 is
`s2 s1 | s5 s3 | s4 | s6` — stable groups `{s1,s2}`, `{s3,s5}` each emitted
later-first: a hash table walked bucket-ascending with head-inserted chains, in
**c1xx**. c2's hash cannot produce it (there `h(sN) = const + N`, and no modulus
collides `s1` with `s2` while separating `s3` from `s4`).

**Consequence: the port never has to reproduce any hash.** The order is already
in the `.gl` it is handed. `docs/OBJ_DYNINIT_SHAPE.md` §7.1 declined this
permutation on the grounds that it "would need the front end's hash reproduced";
that premise is false and #158's owner should revisit it.

### 4D. The emit predicate — found by file name, and it is `main.c`

The project's most valuable unknown is *"should this function body be written
out at all?"*. The lane was told a file name might find it faster than any
call-graph work. It did — but the answer was not in any of the files anyone
expected, and the mechanism inverts the question.

**It is `p2/main.c`.** The walk loop is at `0x10b7f15f`, inside `FUN_10b7f1ff`,
reached from the export `_InvokeCompilerPass@12` (`10bebffd`, `dll.cpp`) via
`10b7f3e7` → `10b7f3b6`. It iterates the global function list at
`.data 0x10c4630c` (next `+0x78`, prev-link `+0x7c`):

```
10b7f16b: mov  edx, DWORD PTR [eax+0x4c]  ; the flag word
10b7f16e: test dl, 0x20                   ; the EMIT bit
10b7f171: je   0x10b7f178                 ; not marked -> skip
10b7f173: test dl, 0x2                    ; "already dequeued"
10b7f176: je   0x10b7f199                 ; marked & not done -> COMPILE IT
```

> **`emit(sym) == (sym->flags4c & 0x20) && !(sym->flags4c & 0x02)`**

Bit `0x02` is set by the loop itself, so the load-bearing bit is **`0x20` at
symbol offset `0x4c`**. Confidence **high** — instruction-level. `coffemit.c`
only *consumes* the same bit later (`10b28548`, deciding which `.debug$S`
records to write), which is why looking for the decision in the writer would
have failed.

### The finding that changes the question: c2 does not compute it

In `FUN_10b9b8e9` (`p2symtab.c`), the `.gl` reader, at the function-with-body
record:

```
10b9bf70: call 0x10c1f91b                 ; varU
10b9bf75: and  eax, 0xfffffffb            ; force-clear bit 0x4
10b9bf78: mov  DWORD PTR [esi+0x4c], eax  ; <<<< THE FLAG WORD, VERBATIM FROM THE IL
```

**The base emit decision is transmitted by `c1xx` in the `.gl` stream and read
wholesale.** Emission order is an IL field too: insertion into `10c4630c` is
sorted ascending by `sym+0x54`, the first `i32c` of the record, then
topologically sorted callee-before-caller by `FUN_10b2778e`.

### What c2 adds — and that outside `-optref` it never subtracts

`FUN_10b27f3c` selects between two closures on `DAT_10c45f9c`:

- **`== 0` → `FUN_10b2773f`, purely ADDITIVE.** A fixpoint propagating `|= 0x20`
  to callees. **Never clears.**
- **`!= 0` → `FUN_10b27b7f`, PRUNES.** A reachability fixpoint; unreached records
  get `flags4c &= ~0x20` and log `"INF:\t%s not allowed to be inlined (globally
  unreferenced)"`.

The **only two** sites in the whole image that clear the bit are `10b27cde`
(inside that pruner) and `10b8a6c6` (per-TU teardown).

**Correction, and it is a case of the map paying for itself.** The child that
found this could not determine what sets `DAT_10c45f9c` — there is **no WRITE
xref to it anywhere in the export** — and inferred "it is LTCG" at *medium*
confidence, flagging that everything above hung off the guess. A *different*
child, working a different seam, reconstructed c2's complete 147-entry flag
table (built at run time by a 4250-byte store sequence at
`0x10c2932e..0x10c2a3b5`, which is why a `.data` scan cannot see it). Joining
the two tables answers it outright:

> **`DAT_10c45f9c` is the target of the `-optref` flag** (`kind` set-1, 7
> readers). Not LTCG.

That is what a map is for: two seams that could not each close a question closed
it by intersection. The pruner is gated on `-optref`.

### Black-box confirmation, and what it refutes

With `/O1 /Oi /Ob2`, an `inline` and a `static` function each called once and
provably folded into the caller are **still emitted out-of-line** — exactly what
the additive path predicts, and it **refutes** the intuition that c2 drops
inlined-away or internal-linkage bodies. It does not, outside `-optref`.

Separately: unreferenced `static`s, unreferenced `inline`s, uninstantiated
templates, unused member functions and unused member templates are **absent from
the `.gl` entirely**. On small TUs **c1xx does the filtering and c2 emits 100% of
what it is handed.**

### The open edge, and how this bears on lane w-emitpred

That constrains any story about the workload's 7.23% emit rate: if c2 never
prunes outside `-optref`, the unemitted bodies must arrive with `0x20` already
clear, or be counted from a stream other than the `.gl` function records.
`p2/main.c` reads the bundle by suffix — `gl`, `in`, `ex`, `sy` — and **`.in` is
the inline-body stream**, whose members reach the candidate list `0x10c3cf68`
via the `0x1000` bit at `10b9bf99`. That is a **hypothesis, confidence low, not
measured**, handed to `w-emitpred` rather than guessed at.

Lane w-emitpred's own intelligence sharpens the target from the other side: its
fitted predicate has a demonstrated **false-positive** class — a virtual call
through a pointer where no constructor of the class is kept in the TU. c2 emits
nothing; the fitted rule says it should. The corrected mechanism is that a
virtual call **ODR-uses the vtable slot, not the definition**. The white-box
reading says that decision cannot be c2's at all: if the emit bit arrives from
the front end and c2's only subtractive path is `-optref`-gated, then **the
ODR-use decision is made in `c1xx.dll`, and no amount of probing `c2` will find
it.** That is a falsifiable claim and it redirects the search.

---

## 4. How to look something up

The map is deliberately two flat tables plus a regenerable export. There is no
database and no tool to learn.

**"Which function mentions X?"** — `docs/whitebox/c2_strings.tsv` has every
literal in the image with the functions that reference it:

```sh
grep -P '\t[^\t]*bss' docs/whitebox/c2_strings.tsv     # -> 10b165f8, and its xrefs
awk -F'\t' '$7 ~ /Pogo/ {print $1, $5, $7}' docs/whitebox/c2_strings.tsv
```

**"What is at address A, and what do we think it does?"** —
`docs/whitebox/c2_functions.tsv`:

```sh
grep -P '^10b982d6\t' docs/whitebox/c2_functions.tsv
awk -F'\t' '$4 != "unknown" && $5 == "high"' docs/whitebox/c2_functions.tsv
```

**"Who calls A / what does A call / what is A's code?"** — regenerate the flat
export (a few minutes, see `C2_MAP_METHOD.md` §3–4) and grep it:

```sh
grep -P '\t10b982d6\t' ~/ghidra-projects/export/c2/calls.tsv        # callers
grep -P '^10b982d6\t'  ~/ghidra-projects/export/c2/calls.tsv        # callees
awk '/^\/\/ ===== FUNC 10b982d6 /{p=1} p; /^\/\/ ===== FUNC /&&p&&!/10b982d6/{exit}' \
    ~/ghidra-projects/export/c2/decomp_all.c
grep -n '^10b982d6' ~/ghidra-projects/export/c2/objdump_intel.asm   # raw bytes
```

The export is **not committed** — bulk decompiled third-party C is the artifact
the clean-room posture should not carry in-tree, and it regenerates in minutes.

**"Is this finding safe to use?"** — read [`DISCLOSURE.md`](DISCLOSURE.md) first.
Navigation is free. Adoption is not.

---

## 5. Confidence, and what `unknown` means here

`c2_functions.tsv` carries a `confidence` column and **`unknown` is a required,
respectable value**. It is used, not avoided:

| confidence | means |
|---|---|
| `high` | mechanical — a fact about the image (thunk-ness, an import edge), or a hand finding that passed a stated refutation check |
| `medium` | a mechanical graph fact with an interpretive step (e.g. "every caller of this function is in cluster C") |
| `low` | a hypothesis with evidence, explicitly not yet checked against the oracle |
| `unknown` | **we do not know.** No guess has been substituted. |

The lane's rule, inherited from the project's *"absence must never read as
success"*: **a labeling method that has not been tested is not evidence.** Every
method used here was first run against facts already known from the black box —
the controls in §6 — and its hit rate reported before any label it produced was
published. A map with 4000 confidently-wrong labels is strictly worse than no
map, because every agent downstream builds on it.

<!-- CONTROLS-START -->
---

## 6. Known-answer controls — scored

Four routines whose behaviour the project already knows **completely** from
black-box work were located *before* being hunted, by predicting which original
source file should contain them from the §3 partition. Grading rule (fixed in
advance, [`PREREG.md`](PREREG.md)): a HIT requires the entry point to fall in
the named file's recovered range **or in the ambiguous gap immediately following
it**; naming the wrong file is a MISS even if the address is close.

**Hits and misses are tallied separately, and the two registration tiers are
never pooled** — a scheme that quietly promotes post-hoc reasoning converts an
honest hit rate into an inflated one.

| # | control | tier | predicted | outcome |
|---|---|---|---|---|
| P1 | **JamCRC** (the string-COMDAT name hash) | **PREREG** | `hash.c`, alt `coffemit.c` | **MISS** |
| P2 | **flag/argv parser** | **PREREG** | `getflags.c` | **HIT** |
| P3 | **`/FAsc` listing writer** | IN-FLIGHT | `list.c` + an unnamed PPC printer | **HIT** (graded half) |
| P4 | **COFF writer** | IN-FLIGHT | `coffemit.c` + model layer in `coff.c` | **HIT** (both halves) |

**PREREG tier: 1 hit, 1 miss. IN-FLIGHT tier: 2 hits, 0 misses.**

### P1 — MISS, and it is the most valuable of the four

The prediction is wrong three times over, and the control did exactly the job a
control exists to do.

1. **JamCRC is not in `c2.dll` at all.** No 256-entry `0xEDB88320` table at any
   4-aligned offset (searched for both bit orders, verifying
   `t[0]==0 ∧ t[128]==poly ∧ t[i]==crc8(i)`); the immediate `20 83 b8 ed` and
   three equivalents occur **nowhere** in the 1 347 072-byte image; the `A..P`
   renderer is absent — the only `ABCDEFGHIJKLMNOP` run continues `…+/`, i.e. it
   is the standard **base64** alphabet. The canonical table lives at file offset
   `0x4898` of `mspdbXX.dll` and `0xb70` of `link.exe`.
2. **`hash.c` is the wrong file even for c2's own string hash.** That region
   (`0x10b5a1fc..0x10b5b1a0`) is the CSE/value-number hash (`% 0x65`, 101
   buckets, `10b5a3cc`/`10b5a399`). c2's actual string hash is `0x10b8a01b`
   (`h = (h<<9) ^ ((h>>14)&0x3FE00) ^ (int8_t)c`) — which sits in **an
   unanchored gap** after `misc.c`, i.e. in a file with no ICE site.
3. The alternate `coffemit.c` is wrong too.

The search method was itself controlled, so the absence means something: the two
`.XBLD$W` aux checksums the port hardcodes (`0x92F87AA0`, `0x838510D9`) are
**also** absent as immediates, yet a freshly compiled obj carries them — and a
fresh obj's `.rdata` CheckSum `948a3c63` reproduces as CRC-32(`0xEDB88320`,
init 0) exactly. **The checksum is computed outside c2's own bytes**, through
the eight-entry callback table `FUN_10b2ae0e` latches into
`DAT_10c44bf4…0x10c44c0c`.

Consequence, in the hunting child's words: *"I would have shipped a wrong
address had I pattern-matched hash-looking code near an emit site."* **No `crc`
label is published.**

### P2 — HIT

The applier `FUN_10c1f572` is **in range** — and is *exactly* `getflags.c`'s
`anchor_end` address. The matcher `FUN_10c1f746` falls in the gap immediately
following `getflags.c` (next file `ioin.c`), a HIT under the stated rule.

Reported honestly: two ancillary helpers land **one file early**, in the gap
after `get_err.c` — the wildcard compare `FUN_10c1f3c9` and the wide `atol`
`FUN_10c1f34c`. The named routine landed; two of its four sub-components sit in
the preceding gap.

Independently black-box confirmed at full strength: the reconstructed 147-entry
table was replayed against the real compiler — **156/156 flags accepted, 0 false
positives, 0 false negatives**, with the negative controls (`/d2nop`,
`/d2dumpil`, `/d2Bx`, …) all raising C1007 and `/d2GL`, `/d2W9` raising C1048 as
predicted.

### P3 — HIT on the graded half; the other half was too vague to grade

`list.c` was named and landed: `10b70e57` and `10b71324` are **in range**,
`10b71d8f` in the gap immediately after. The machine-dependent half was
predicted only as "a separate PPC instruction printer late in the image" —
**no file was named, so it cannot be scored either way.** Observed: the `.cod`
printer cluster `10c10c7d`/`10c10d73`/`10c10cf1`/`10c11219` straddles
`mdlist.c`'s anchor, and `mdlist.c` is indeed late in the image. Recorded as
corroboration, **not** as a hit.

### P4 — HIT on both halves, and the most exact result in the set

Five of five COFF writer routines are **in range** in `coffemit.c`
(`10b2b0dd`, `10b2b02d`, `10b2a936`, `10b2948b`, `10b2ad50`), and the predicted
"model/reader layer in `coff.c`" landed too (`10b28586`).

Two of them are not merely in range — **they are the anchor addresses
themselves**: `FUN_10b2b0dd`, the COFF/BIGOBJ file-header writer, *is*
`coffemit.c`'s `anchor_end`; `FUN_10b28586`, the obj opener that selects the
`Machine` word and creates `.drectve`, *is* `coff.c`'s anchor. The prediction
that `coff.c` and `coffemit.c` are a reader/model layer and a writer layer was
made from the file names alone and is borne out: `coff.c` opens the file and
holds the section model, `coffemit.c` does every `fwrite`.

<!-- CONTROLS-END -->

<!-- NOTKNOWN-START -->
---

## 7. What is NOT known

The honest boundary. Read this before building on anything above.

### 7.1 Limits of the partition itself

* **72.8% of the anchored span is gap.** In-range attributions are facts;
  **gap attributions are hypotheses.** 22 of 52 files have a zero-width anchor.
* **A file with no ICE site is invisible** and is silently absorbed into its
  predecessor's gap. This is not hypothetical: c2's string hash `0x10b8a01b`
  lives in exactly such a file, and the naive reading "it is in `misc.c`" is
  wrong. **There is no way to count how many invisible files exist** — the
  method cannot see its own misses. Treat 52 as a lower bound on the back end's
  translation units, never as the count.
* **Line numbers order sites within a file only 66.9% of the time**, and very
  unevenly (`coffemit.c` 91%, `reader.c` and `dbg.cpp` 50%). We did **not**
  separate optimizer reordering from mis-recovered line immediates.
* **The call-graph cross-validation was not run.** An independent test — do
  intra-range call edges exceed a null? — was planned and lost to a box-wide
  OOM. The link-order model is supported by the ordering statistics in §3.2 and
  by the control hits in §6, but **not** by call-graph locality, and that check
  remains outstanding.

### 7.2 Named open questions, with where to start

| question | status | first move |
|---|---|---|
| **`.CRT$XCU`'s kind and Characteristics** | **open — highest value for factor C.** The name is absent from `c2.dll`, present in `c1xx.dll` (`0x300d4`), and `.CRT` matches none of `FUN_10be7727`'s prefixes, so the kind must arrive *in the IL*. No IL record carrying a section name was traced into `FUN_10be7473`/`FUN_10be74cf`. | `FUN_10b9b8e9` |
| **COFF symbol-table order** (R6) | **open.** Three probes gave three answers: all-dyninit → IL order; no-dyninit → *source* order, matching neither IL nor address order; mixed → ascending address. The doc's "strictly descending address" holds only in the first case and is coincidence there. | `FUN_10b8303c(g_symList@0x10c2e234, FUN_10b2a936)` — the list's iteration order *is* the question |
| **Section emission order** | observed, mechanism untraced. Kind-ordered, not name-ordered. | `FUN_10b287b8` |
| **The byte offset of the emit flag in a `.gl` record** | **open.** Decode *order* is known; the offset is not, because preceding fields are variable-length. | decode the tag header at `FUN_10b9b8e9` |
| **Whether the census's unemitted bodies are `.gl` records at all** | **open, and it matters.** Small-TU probes show `.gl` ≡ obj, so the census is measuring something else. `.in` is the obvious candidate. Not measured. | `10b9bf99`, list `0x10c3cf68` |
| **c1xx's zero-initializer folding** | asserted, **not verified** — out of scope, and it is a c1xx fact |
| **`.ex` opcode semantics** | operand *formats* known for all 200 opcodes, attributes for ~194; **naming each opcode** needs reading 189 arms of `FUN_10bc2d7a` | mechanical, recipe is exact |
| **Selection code 8** | mapped by `FUN_10b281f7` and special-cased at emit, but it is **not** a documented `IMAGE_COMDAT_SELECT_*`. **Do not assume it is a valid on-the-wire value.** |
| **Kind 9** | `FUN_10b982d6` handles it; **no creator found**. `unknown`. |

### 7.3 Claims deliberately *not* made

* **No `crc` label.** JamCRC is absent from `c2.dll`; the aux `CheckSum` is
  computed outside c2's bytes through the callback table at
  `DAT_10c44bf4…0x10c44c0c`. Pattern-matching "hash-shaped code near an emit
  site" would have produced a confident wrong address. See §6 P1.
* **`Characteristics = 0x0180`** is read from the immediate at `0x10b2b270` and
  was **not** cross-checked against a real corpus obj. One `xxd -l 20` settles
  it; do that before anything depends on it.
* **`color.c` is the register allocator** and is deliberately **not mapped**.
  There is Ghidra+LLM first-draft Rust of a COLOR allocator under
  `crates/c2-core/src/paint/` — scaffolding, gitignored, explicitly **not
  truth**. The doctrine is I/O-behavioral and register allocation is not on the
  critical path. Noted and skipped on purpose.
* **`msobjXX.dll` does not write the obj.** Its single import
  `FCreateFromBytesW` has exactly one call site (`0x10be83f2`), which opens an
  *existing* file `GENERIC_READ` and only ever reads through vtable slots. **No
  write path touches msobj**; c2 `fwrite`s the obj itself. The lane's earlier
  worry that the COFF writer might live in msobj is **refuted**.
* **Nothing here is evidence about the port's correctness.** A white-box reading
  is a hypothesis. The real `c2` under wibo plus a byte-exact obj compare
  remains the sole judge, and every claim above is stated with a refutation
  condition so it can be killed by one.

<!-- NOTKNOWN-END -->
