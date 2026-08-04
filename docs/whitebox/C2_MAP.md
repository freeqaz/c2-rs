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

*(the honest boundary — filled in with the rest)*

<!-- NOTKNOWN-END -->
