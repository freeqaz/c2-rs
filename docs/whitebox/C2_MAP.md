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
'%s'` is there, not here). So hunt diagnostics by the **immediate**, never by
the string.

**But not by the immediate you would expect — an earlier revision of this file
was wrong here, and it cost a child time.** The numbers are **not** stored as
`0x3EF`/`0x43B`. `FUN_10c1ee7d` @ `0x10c1ee7d` adds a **base of 1000**
(`lea ecx,[esi+0x3e8]` at `0x10c1ee84`), so every raise site pushes
`number − 1000`:

| diagnostic | immediate to grep | site |
|---|---|---|
| C1001 (the ICE that yields §3) | `0x001` | — |
| C1007 unrecognized flag | **`0x007`** | `0x10b84a93` |
| C1047 | `0x02F` | — |
| C1081 file name too long | `0x051` | `0x10c1eee8` |
| C1083 cannot open file | **`0x053`** | `0x10c1fd40` |
| C1310 | `0x136` | — |
| C1900 / C1905 (IL magic / format) | `0x384` / `0x389` | `0x10b97a22` |

Grepping for `0x43b` finds nothing, which reads as "the diagnostic is not in
this binary" when in fact it is. Verified live: `cl /c /Bd /d2nop add3.cpp` →
`fatal error C1007: unrecognized flag '-nop' in 'p2'`, and `/d2ilzz` →
`C1083 … 'zzgl'`.

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

## 3B. `coffemit.c` — the densest file, and the one on the critical path

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
   The name/kind/class/override all arrive **in the IL**, in the tag-`0x09`
   record decoded in **§3F** — which closes this question.
2. **COMDAT and symbol emission order (R6)** — the writer is
   `FUN_10b2a936`, driven by `FUN_10b8303c(g_symList@0x10c2e234, …)`, so the
   *iteration order of that list* is the whole question. **Unresolved**; see §7.
3. **The `.bss` object-address permutation** — **settled, and it was never
   c2's.** See §4C.

### 3C. The `.bss`/`.data` decision — factor C's +402-TU item

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

### 3D. The `.bss` permutation — REFUTED as a hash, and it is c1xx's

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
(`10b9b161`/`10b9b6a4` → `10c27b56` → the bump allocator at `10c2757d`). Has one
→ **deferred**, head-inserted onto `DAT_10c2f064` and drained head-first by
`10b99093`. Head insert + head-first drain = reversal. The first-touch flag
`0x800` in `10c27b56` is why the two groups never interleave.

> **Independently confirmed by lane w-bss, black-box, from the IL alone.**
> Reaching the same mechanism by a route that never opened a disassembler —
> across 6 cells, 4 declaration-order permutations, and N = 1…10 in three
> families — makes the reversal rule the best-supported claim in this document.
> w-bss adds that the ordering keys on the **source identifier** (static and
> extern give identical orders), not on declaration order, linkage, type or
> position; and that `.data` is declaration order and does not permute. Its own
> 7 452-configuration hash search also returned nothing (best score 0.08 against
> a 0.03 baseline), with the right diagnosis: it was fitting a **c2** hash to a
> **c1xx** artefact. **Two independent routes to the same mechanism is the
> strongest evidence this project produces.**

### 3D-bis. The bump rule — **RETRACTED**, and it is the lane's best calibration datum

An earlier revision of this file stated the `.bss` offset rule as *"align 8,
then `+(8−size)` for sizes 1/2/4"*, read straight out of `FUN_10c2757d`:

```
cur = (cur + 7) & ~7;
if (size - 1 < 7 && (size & (size - 1)) == 0) cur += 8 - size;
cur += size;
```

**That rule does not reproduce the real objs.** Lane w-bss §5.5 records the
counterexample. The resolution on this project is not negotiable and is applied
here without argument: **the obj is the sole judge.** A rule read off the
disassembly that disagrees with what c2 actually emitted is wrong, however clean
the code looked — and this one looked very clean, which is exactly the problem.

The claim is withdrawn. What replaces it is **`unknown`**, pending an obj-checked
derivation.

**Why it failed is itself worth determining**, and there are three candidate
explanations, none yet distinguished:

1. **The path is not the one these inputs take.** `10c27b56` has seven callers;
   the read assumed the streaming one.
2. **A guard was not modelled.** The `0x800` first-touch flag and `sym[0xC]`
   short-circuit both gate entry, and neither was varied.
3. **A later pass rewrites the result.** Section-relative offsets are assigned
   before `FUN_10b287b8` assigns section indices; something downstream may
   re-lay-out.

Determining which would be genuinely valuable, because it says how much of this
binary's *apparent* logic is actually reachable on real inputs — and that
generalises well beyond `.bss`.

**The residual permutation is the front end's.** The `.gl` order for N=6 is
`s2 s1 | s5 s3 | s4 | s6` — stable groups `{s1,s2}`, `{s3,s5}` each emitted
later-first: a hash table walked bucket-ascending with head-inserted chains, in
**c1xx**. c2's hash cannot produce it (there `h(sN) = const + N`, and no modulus
collides `s1` with `s2` while separating `s3` from `s4`).

**Consequence: the port never has to reproduce any hash.** The order is already
in the `.gl` it is handed. `docs/OBJ_DYNINIT_SHAPE.md` §7.1 declined this
permutation on the grounds that it "would need the front end's hash reproduced";
that premise is false and #158's owner should revisit it.

### 3E. The emit predicate — found by file name, and it is `main.c`

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

> **`(sym->flags4c & 0x20) && !(sym->flags4c & 0x02)`** — but this selects the
> **seed set of c2's work queue, not the emitted set.** See the correction
> immediately below; an earlier revision of this file stated it as the emit
> predicate outright and that was an over-claim.

> ### ⛔ 2026-08-18 — **"inside `FUN_10b7f1ff`" is WRONG, and the sentence above stands as written.**
>
> `0x10b7f15f` is **below** `FUN_10b7f1ff`'s entry address, so it cannot be
> inside it, and **`decomp_all.c` has no body containing the loop** — anyone
> grepping the export for `FUN_10b7f1ff` to read the emit predicate finds the
> wrong function. Found by lane `w-c2map2` when 17 addresses in this very
> section failed to resolve to any function in the flat export.
>
> The loop is inside **`0x10b7f022`**, which is a real function entry Ghidra's
> auto-analysis never created: `push esi` immediately after the `ret` at
> `0x10b7f021`, and the target of a **tail jump** from `FUN_10b7f1ff`
> (`jmp 0x10b7f022` at `0x10b7f362`) — exactly the shape auto-analysis misses.
> Carried in `scripts/build_ref.py`'s `GHIDRA_MISSED` table; see
> [`ref/README.md`](ref/README.md) §6.2.
>
> **Everything else in §3E — the predicate, the bits, the cascade, the tag-`0x0e`
> decode — is unaffected. Only the location was wrong**, and "Ghidra found 4 916
> functions" is a statement about Ghidra.

Bit `0x02` is set by the loop itself, so the load-bearing bit is **`0x20` at
symbol offset `0x4c`**. `coffemit.c` only *consumes* the same bit later
(`10b28548`, deciding which `.debug$S` records to write), which is why looking
for the decision in the writer would have failed.

#### The correction: seed set, then closure

The predicate was verified by clearing bit `0x20` in a real `.gl` and replaying.
On a bundle of **six mutually independent leaf functions** it was a clean hit —
the function's COFF symbol vanished, `.text` shrank by exactly its 16 bytes, and
the remaining bytes were identical (`base.text[16:] == mut.text[0:]`). Both
halves confirmed separately: setting `0x02` yields a **bit-identical obj** to
clearing `0x20`.

**On a bundle with a real call graph, clearing `0x20` on 17 of 20 functions
changed nothing at all.** Only the three functions nothing else calls vanished
singly. A cascade test pins the actual rule:

```
cleared 3 -> lost 3   (the roots)          cleared 6 -> lost 6  + supershuffle
cleared 4 -> lost 4   + getKeyImpl         cleared 7 -> lost 7  + shuffle3
cleared 5 -> lost 5   + revealKey          cleared 8 -> lost 8  + parseHex16
```

Each function falls only once its **caller** has also been cleared. Clearing all
20 deletes `.text`, `.pdata` *and* `.data` entirely.

> **The emitted set is the `0x20`-seeded set closed under "referenced by an
> already-emitted function."**

**This reconciles the two halves of the lane's own work rather than upsetting
them.** The static read had already found the closure — `FUN_10b2773f`, the
non-`-optref` path, described as *"a fixpoint propagating `|= 0x20` to callees;
never clears"*. The black-box cascade is that fixpoint, measured. **The
disassembly predicted the mechanism and the oracle confirmed it**, which is the
strongest form this document has.

**The practical warning:** the six-leaf fixture read as a clean per-function
predicate *only because its fixtures were mutually independent leaves*. Anyone
porting the emit rule from the seed test alone **will over-delete on real TUs**.

#### The tag-`0x0e` record, decoded — and where the flag word actually sits

Handler `0x10b9bdcf`, on `?zero_test@@YAII@Z` in a real `.gl`, offsets
`0x0a9..0x0d8`:

```
0x0a9  0e                 GetByte   tag = 0x0e
0x0aa  e4 09              varU      -> +0x28  symbol id 2532
0x0ac  00                 GetByte   -> +0x31
0x0ad  "?zero_test@@YAII@Z\0"  GetCStr   name
0x0c0  86                 GetByte   -> +0x37  storage class
0x0c1  02                 i32c      -> +0x40
0x0c2  05 04              varU      -> +0x20  flags 0x0405
0x0c4  00 00              varU      -> +0x0c  owner idx   [gate unresolved]
0x0c6  00                 i32c      -> optword
0x0c7  80 01 10 00 00     i32c      -> +0x2c  type index 0x1001
0x0cc  00 / 0x0cd  00     i32c x2   -> debug fields
--- tag==0x0e payload ---
0x0ce  80 54 0a 00 00     i32c      -> +0x54 = 2644   (.ex offset)
0x0d3  00                 i32c      -> +0x58 = 0      (.sy offset)
0x0d4  12                 i16c      -> +0x50 = 18
0x0d5  68 10              varU      -> +0x4c = 0x1068   <<<< THE FLAG WORD
0x0d7  01                 i16c      -> +0x52 = 1
0x0d8  00                 i32c      inline count   [only if +0x4c & 0x1000]
```

**The flag word is at file offset `0x0d5..0x0d6` for this record and is at no
fixed offset in general** — the name is variable-length and three preceding
scalars use the `0x80` escape. Exactly the trap §3F warns about.

The decode is confirmed by an independent check that could not have come out
right by accident: across six records `+0x58` = 0, 30, 60, 90, 120, 150 over a
`.sy` of **exactly 180 = 6 × 30** bytes, and `+0x54` steps by 110 over a
3312-byte `.ex`. Every record ends precisely where the next tag begins — 6/6,
then 20/20 and 24/24 on two further bundles.

**Two corrections to this document's own earlier description of the dispatcher.**
Tags `0x04`, `0x0E` and `0x10` **share one handler** (`0x10b9bdcf`); and
`0x10b9c5ca` is **not a no-op arm** — it is `mov edx,0x7ba; jmp 0x10b9bd1a`, the
**fatal-error path**, confirmed live by a deliberate one-byte desync producing
`C1001 … p2symtab.c, line 1978` (= `0x7ba`). Tag `0x0E` is not in that set at
all. The "shared no-op arm" reading was wrong.

Bit semantics settled by single-bit mutation: `0x04` is force-cleared on read
and is **dead in the IL** (setting it gives a byte-identical obj); `0x1000`
marks *presence of the trailing inline list* — clearing it alone desyncs into
C1001, clearing it *and* dropping the count byte gives a byte-identical obj.
`0x08`, `0x40`, `0x80`, `0x2000`, `0x2080` and the high bits were codegen-inert
on these fixtures, which is a **coverage-bounded negative, not a proof** — the
fixtures are simple leaves. `0x0001` makes c2 SIGSEGV; real coupling, cause
unidentified.

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

### 3F. The IL section record — `.gl` tag `0x09`, and factor C's last link

This was §7's highest-value open item ("the last link between *c2 emits the
right shape* and *we can predict the shape from IL*"). **It is now closed**, and
closed the right way: decoded statically, then proven by mutating real `.gl`
bytes and replaying them through the real compiler.

`FUN_10b9b8e9` case 9 (jump-table slot 7 → `0x10b9c212`) is the
section-definition record. Node size `0x68`.

| # | primitive | → | field |
|---|---|---|---|
| 0 | GetByte | — | tag `0x09` |
| 1 | **varU** `10c1f91b` | `+0x28` | **section index** — what symbols reference |
| 2 | **GetCStr** `10c1fc5b` | `+0x04` (interned) | **name** |
| 3 | **GetByte** `10c1f8fc` | `+0x4d` | **kind** |
| 4 | **GetCStr** `10c1fc5b` | `+0x3b` | **class/group** (`"CODE"`, `"DATA"`, `""`) |
| 5 | **GetU32** `10c1fb8b` | `+0x53` | **Characteristics override** |

Then `FUN_10b982d6` computes `(name, chars)` and writes back to `+0x53`.

A real record, hand-decoded — `_CL_9d4ae740.gl` @ `0x00E1`:

```
09 | 06 0a | 2e 43 52 54 24 58 43 55 00 | 1d | 44 41 54 41 00 | 00 00 00 00
^tag ^varU  ^ ".CRT$XCU\0"                ^kind ^ "DATA\0"      ^chars = 0
```

22 bytes; the next tag lands exactly at `0x00F7`. **25 records chain cleanly to
EOF** in that TU.

#### `.CRT$XCU`, finally

| property | value | comes from |
|---|---|---|
| name | `.CRT$XCU` | **IL** |
| **kind** | **`0x1D`** | **IL** |
| class | `"DATA"` | **IL** |
| chars override | `0` (absent) | **IL** |
| **Characteristics** | **`0xC0000040`** | **computed by c2** — `FUN_10b982d6`; `0x1D` survives `FUN_10be7727` because `.CRT` matches no prefix |
| alignment | 4 → `0x300000` | **computed by c2**, accumulated from the *symbols* |
| COMDAT selection | **none** | not in the record at all |
| **emitted** | **`0xC0300040`**, non-COMDAT, last in the section table | |

Kind `0x1D` means *"named data section, keep my name"*: for kind `1`,
`FUN_10b982d6` resolves through `FUN_10be76d4` and substitutes `".data"`; for
`0x1D` it takes `sect+4`, the section's own name. Both yield `0xC0000040`.

#### The mutation matrix — this is what makes it a fact rather than a reading

Every field was patched in the bytes and replayed through real c2:

| mutation | emitted chars | reads |
|---|---|---|
| baseline | `0xC0300040` | |
| name `.`→`Z` | `ZCRT$XCU`, chars unchanged | name is IL-borne |
| kind `1D`→`00` | `0x60400020` | `.text`, align forced to 8 |
| →`03` / `04` / `13` | `0xC0300080` / `0x40300040` / `0x42300040` | `.bss` / `.rdata` / `.debug$S` |
| override → `0x40000040` | `0x40300040` | **override beats kind** |
| override → `0xC0500040` | `0xC0500040` **verbatim** | c2 skips its align OR when the nibble is already set |
| class `DATA`→`CODE` | no change | |
| **swap the ids of `.CRT$XCU` and `.CRT$XCL`** | initializer pointer lands in **`.CRT$XCL`** | the varU **is** the section index |

Source-side corroboration: `#pragma section(".mysec", read, write, discard)`
produces a tag-9 record with kind `01` and **`chars = 0xC2000040`** — **the u32
is exactly the `#pragma section` attribute set.** `__declspec(align(64))` on the
symbol gave `0xC2700040`, i.e. alignment comes from the **symbol**, not the
record.

COMDAT-ness never appears in tag 9: `FUN_10b283b0` spins a COMDAT child off the
tag-9 base via `FUN_10be74cf`. `.text$yc` has an *identically shaped* record yet
emits `0x60401020`. `.CRT$XCU` is non-COMDAT simply because no symbol asked for
one.

#### Correction to the `.gl` header size, and it is the variable-width trap again

This document previously said the `.gl` header is **26 bytes**. **It is 26 only
when all four `i16c` version fields fit in one byte.** In a real capture the
build number `11886` escapes (`80 6e 2e`), making the header **28** and moving
the first record from `0x1A` to `0x1C`. Exactly the failure mode §3F's own
primitives table warns about — and it was in this file.

---

## 4. How to look something up

> ### 2026-08-18 — **there is now an address-indexed reference: [`ref/`](ref/).**
>
> This section still works and is unchanged. What it did not have is a way to
> start from an **address** and find out what the record already says about it,
> or from a **subsystem** and find the functions in it —
> [`ref/ADDR.tsv`](ref/ADDR.tsv) joins the 313 hand labels, `c2_functions.tsv`,
> `c2_tus.tsv`, the flat export and the **1 126 addresses cited across
> `docs/`** into one row per address, and [`ref/SUBSYS.md`](ref/SUBSYS.md)
> indexes the subsystem pages. Read [`ref/README.md`](ref/README.md) first for
> the `[R]`/`[O]`/`[I]` provenance legend, which this file predates.

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

### 5.1 Calibration — what `high` is actually worth

The confidence column is only meaningful if we know its error rate. We now have
one hard measurement of it, and it is sobering.

**The `.bss` bump rule (§3D-bis) was marked `high`, was read at instruction
level from a clean and complete-looking function, and is WRONG** — it fails
against real objs. It was not a guess, not a pattern match, and not a shaky
inference. It was the *good* kind of white-box finding, and it still did not
survive contact with the compiler's actual output.

That gives the following calibration, which readers should apply to every row in
this document:

| claim class | track record | how far to trust it |
|---|---|---|
| **white-box read, obj-checked by mutating the input** | tag-`0x09` section record (§3F), emit flag word (§3E) — every field's effect observed | as good as a fact |
| **white-box read, obj-checked as a value** | COFF header immediates (§7.3) — 3/3 held | as good as a fact |
| **white-box read, oracle-checked another way** | `.bss` reversal — two independent routes | strong |
| **white-box read, NOT obj-checked** | `.bss` bump rule — **1 known failure** | **hypothesis only, regardless of the `high` label** |
| **obj-checked, but on an unrepresentative fixture** | emit predicate — right on 6 independent leaves, wrong on 17/20 of a real call graph | **the fixture is part of the claim** |
| **absence claims with a controlled search** | JamCRC (§6 P1) — method verified against known-present constants | strong |
| **range attribution, in-anchor** | controls 3/4, two landing on anchor addresses | strong |
| **range attribution, in-gap** | c2's string hash mis-attributable to `misc.c` | hypothesis only |

**The operative lesson: `high` in this table means "high confidence in the
reading of the instructions", NOT "high confidence that this is what c2 does."**
Those are different propositions and the `.bss` rule is the proof that they come
apart. A function can be present, correct-looking, fully decompiled, and simply
**not on the path your inputs take** — or guarded by a condition you did not
vary, or overwritten downstream.

**And a second, different failure mode, from the emit predicate:** a claim can
be obj-checked, reproduce perfectly, and still be **wrong as stated**, because
the fixture lacked the structure that would have exposed the gap. Six
mutually-independent leaf functions cannot distinguish "this bit decides
emission" from "this bit seeds a queue that is then closed under reachability".
Only a fixture with a call graph can. So: **an obj check is only as good as the
fixture's structural coverage, and the fixture belongs in the claim.** State
what you tested on, not just that you tested.

Anything from this directory that is about to influence code should be
obj-checked first. That is cheap — the oracle is `wibo cl.exe` and a byte
compare — and this document now contains a worked example of what it costs to
skip it.

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
* **The call-graph cross-validation was run and it did NOT confirm the
  partition.** This is reported as a negative result rather than dropped.

  | formulation | observed intra-file edge fraction | null | verdict |
  |---|---|---|---|
  | tiled model (file *i* owns `[start_i, start_{i+1})`), 16 094 edges | **0.3184** | 0.2945 ± 0.0092, size-matched shuffle | +2.6 sd, ratio 1.08× — but **the null's max (0.3240) exceeded the observed value** |
  | anchor ranges only (certain attribution), 2 911 edges | **0.5060** | 0.6041 ± 0.1216, same lengths at random positions | **−0.8 sd — observed is *below* the null mean** |

  Neither formulation separates the real partition from a random one. **The
  honest reading is that the test lacks discriminating power, not that the
  partition fails**, and three reasons are visible in the numbers:

  1. Anchors are ICE-site *brackets*, not module boundaries — they cover only the
     ICE-bearing middle of a file, so intra-anchor edges are a biased subsample.
  2. A compiler's call graph is heavily cross-module **by design** — everything
     calls the symbol table, the allocator and the error reporter — so the true
     intra-module fraction is genuinely low, and there is little signal to find.
  3. The positional null is a *bad* null: randomly placed short intervals land in
     dense code and capture tightly-coupled local neighbourhoods, which is why it
     scores 0.60.

  Per-file locality varies enormously under the tiled model (`tuple.c` 0.71,
  `p2symtab.c` 0.45, `cgintrin.c` 0.13), which is itself consistent with the
  tiled regions being well-aligned for some files and badly for others.

  **What this does *not* do is weaken §3.2 or §6.** The partition's evidence is
  the ordering statistics (7 runs vs 26.5 expected, P = 1.5 × 10⁻²⁵; every run
  directory-pure) and the control hits (3 of 4, two landing *exactly on* anchor
  addresses). Call locality was an additional axis that turned out to be
  uninformative here. Anyone repeating it should not read the numbers above as
  contradicting the partition — they should read them as a test that cannot tell
  the two hypotheses apart.

### 7.2 Named open questions, with where to start

| question | status | first move |
|---|---|---|
| ~~**`.CRT$XCU`'s kind and Characteristics**~~ | **CLOSED — see §3F.** The `.gl` tag-`0x09` record carries name, kind (`0x1D`), class and a Characteristics override; c2 computes `0xC0000040` and ORs alignment from the *symbols*. Proven by mutating real `.gl` bytes and replaying through real c2. | — |
| **COFF symbol-table order** (R6) | **open.** Three probes gave three answers: all-dyninit → IL order; no-dyninit → *source* order, matching neither IL nor address order; mixed → ascending address. The doc's "strictly descending address" holds only in the first case and is coincidence there. | `FUN_10b8303c(g_symList@0x10c2e234, FUN_10b2a936)` — the list's iteration order *is* the question |
| **Section emission order** | **open, `medium`.** Kind-ordered, not name-ordered. Mutation shows kind `0x1D` defers to the end while all six other kinds tested landed at the *same* index — so the key is `0x1D`-vs-not, not the kind value. Candidate sorters `FUN_10b98b00`/`FUN_10b9aaa8`/`FUN_10b9acfa` inspected; none clearly it. | `FUN_10b287b8` |
| ~~**The byte offset of the emit flag in a `.gl` record**~~ | **CLOSED — §3E.** Full tag-`0x0e` decode; the word is a `varU` at **no fixed offset** (variable-length name + three `0x80`-escaped scalars before it). Chain verified exactly on 6+20+24 records across three bundles. | — |
| **The enqueue mechanism for the reachability closure** | **NEW, open.** The closure is *proven* by cascade and its fixpoint (`FUN_10b2773f`) is located, but the code that adds a callee to the queue was not pinned. | `FUN_10b276e4`, list `0x10c4630c` |
| **The gate on the owner-index `varU` at the tag-`0x0e` gap offset 0** | **NEW, open, `low`.** Ghidra and raw asm both say it is read only when `+0x20 & 0x200`, yet `+0x20` decoded to `0x005`/`0x105`/`0x405` in **every** record across three bundles — bit `0x200` never set — while the two bytes are unambiguously consumed. One record in 61 (`??0P@@QAA@ABU0@@Z`, a constructor) breaks the chain, almost certainly the same gate. | `10b9be72` |
| **Whether the census's unemitted bodies are `.gl` records at all** | **open, and it matters.** Small-TU probes show `.gl` ≡ obj, so the census is measuring something else. `.in` is the obvious candidate. Not measured. | `10b9bf99`, list `0x10c3cf68` |
| **c1xx's zero-initializer folding** | asserted, **not verified** — out of scope, and it is a c1xx fact |
| **`.ex` opcode semantics** | operand *formats* known for all 200 opcodes, attributes for ~194; **naming each opcode** needs reading ~~189 arms~~ **61 real arms over 95 opcodes, plus one refusal over 94 ([`WB_ILARMS_MAP.md`](WB_ILARMS_MAP.md) §1)** of `FUN_10bc2d7a` | ~~mechanical, recipe is exact~~ **REFUTED — #3419: 17 of 62 arms read a `.data` global, six frame slots carry state across tokens, and arm 20 re-enters the dispatch** |
| **Selection code 8** | mapped by `FUN_10b281f7` and special-cased at emit, but it is **not** a documented `IMAGE_COMDAT_SELECT_*`. **Do not assume it is a valid on-the-wire value.** |
| **Kind 9** | `FUN_10b982d6` handles it; **no creator found**. `unknown`. |

### 7.3 Claims deliberately *not* made

* **No `crc` label.** JamCRC is absent from `c2.dll`; the aux `CheckSum` is
  computed outside c2's bytes through the callback table at
  `DAT_10c44bf4…0x10c44c0c`. Pattern-matching "hash-shaped code near an emit
  site" would have produced a confident wrong address. See §6 P1.
* ~~**`Characteristics = 0x0180`** was read from the immediate at `0x10b2b270`
  and never cross-checked.~~ **Now checked — and it holds.** Two real objs
  produced by the live toolchain:

  ```
  probe.obj  f201 0b00 8f5c 716a c003 0000 2200 0000  0000 8001
  n6.obj     f201 0d00 945c 716a 7c06 0000 3600 0000  0000 8001
             ^^^^                                     ^^^^ ^^^^
             Machine 0x01F2                           SizeOfOptionalHeader 0
             = POWERPCFP                                   Characteristics 0x0180
  ```

  All three static reads confirmed against real output: `Machine = 0x01F2`
  (`IMAGE_FILE_MACHINE_POWERPCFP`, so the non-LTCG branch of `FUN_10b28586` is
  the one taken), `SizeOfOptionalHeader = 0`, and
  `Characteristics = 0x0180` = `IMAGE_FILE_32BIT_MACHINE | 0x0080`. A
  disassembly hypothesis promoted to an observation of the compiler's actual
  I/O, which is the only kind of evidence this project accepts.
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
