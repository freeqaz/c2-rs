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

## 3. Subsystem clusters

*(populated from the analysis children; see §5 for the confidence rules)*

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

## 6. Known-answer controls

*(scored results and the hit rate; misses are reported as misses)*

<!-- CONTROLS-END -->

<!-- NOTKNOWN-START -->
---

## 7. What is NOT known

*(the honest boundary — filled in with the rest)*

<!-- NOTKNOWN-END -->
