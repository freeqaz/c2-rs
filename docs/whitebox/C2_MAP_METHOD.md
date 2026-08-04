# C2_MAP_METHOD — how to reproduce the map from a clean checkout

> **PROVENANCE — DISASSEMBLY-DERIVED.** Everything in `docs/whitebox/` was
> obtained by statically disassembling Microsoft's `c2.dll`, not by observing its
> I/O. It is **navigation only**. No value, constant, table, or algorithm from
> this directory may be copied into `crates/` without first adding a row to
> [`DISCLOSURE.md`](DISCLOSURE.md) naming the address it came from. The project's
> clean-room claim in `README.md` is a *blanket* claim today; adopting anything
> from here weakens it to per-finding disclosure. See
> [`README_DELTA.md`](README_DELTA.md).

## 0. The bytes this map is about

| file | sha256 | size |
|---|---|---|
| `compilers/X360/16.00.11886.00/c2.dll` | `c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258` | 1 347 072 |
| `compilers/X360/16.00.11886.00/msobjXX.dll` | `09ecf372ce3424b0c1947706296153a83c453c2716d5fad6e7b28e13936000bd` | 57 856 |
| `compilers/X360/16.00.11886.00/msdisXXX.dll` | `03dc209765761fbf1a9a4781a54d2936b8f4b12f37e557d78e71c5b865f42299` | 87 040 |
| `compilers/X360/16.00.11886.00/c1.dll` | `68949991f05c5a26c43c113f8742cae201aedc90b659c949098cdb73c6ec8ef5` | 496 640 |
| `compilers/X360/16.00.11886.00/c1xx.dll` | `e3057880d38354078459adc44dc4374e9a3429eaad1ecb1da9830ef79057f366` | 1 714 176 |
| `compilers/X360/16.00.11886.00/cl.exe` | `e35465b562f14a7f8eed5ca446a6f4f128477e86a676ea48dfeee38f1803bc9f` | 99 328 |

`compilers/` is gitignored (Microsoft binaries are never committed); populate it
with `scripts/fetch_compilers.sh`. **Verify the sha256 before trusting any
address in this directory** — every address here is an absolute VA in this exact
image.

## 1. Environment

* Ghidra **12.1.2**, headless analyzer at `/opt/ghidra/support/analyzeHeadless`,
  JDK 26. No Ghidra extensions, no PDB, default analyzers.
* `objdump` (GNU binutils) — reads PE32 as `pei-i386` and is used as an
  **independent** disassembly source, so no claim rests on Ghidra alone.
* `python3` for the two generator scripts. (Tooling only — outside the workspace's
  std-only Rust constraint, same status as `scripts/plot_perf.py`.)

## 2. The dot-path trap

**Ghidra headless refuses any path containing a `.`** (recorded in
`docs/ROADMAP.md` §9.4). This bites twice, and the second one is not in §9.4:

1. it kills `.claude/worktrees/…`, so the analysis cannot run from a worktree; and
2. **it kills the binary's own directory**, `compilers/X360/16.00.11886.00/` —
   the XDK build id itself contains dots.

Workaround: copy the DLLs to dot-free names outside the repo, and keep the
project there too. Nothing under `~/ghidra-projects/` is committed.

```sh
mkdir -p ~/ghidra-projects/{bin,scripts,export,log,c2map,small}
B=compilers/X360/16.00.11886.00
cp $B/c2.dll       ~/ghidra-projects/bin/c2dll
cp $B/msobjXX.dll  ~/ghidra-projects/bin/msobjdll
cp $B/msdisXXX.dll ~/ghidra-projects/bin/msdisdll
sha256sum ~/ghidra-projects/bin/c2dll     # must equal the table above
```

## 3. Analysis

```sh
/opt/ghidra/support/analyzeHeadless ~/ghidra-projects/c2map c2map \
    -import ~/ghidra-projects/bin/c2dll \
    -processor x86:LE:32:default -loader PeLoader \
    -analysisTimeoutPerFile 5400
```

Auto-analysis of the 1.35 MB image converged in a few minutes and found **4916
functions**. `docs/ROADMAP.md` §9.4 warns that this "takes a long time"; on this
box it did not. The sibling binaries go into a separate project (`small`) so they
can be analyzed concurrently — **two headless runs must never share one project
directory.**

## 4. Flat export — the thing that makes this reproducible and parallel-safe

A Ghidra project is a single-writer database: concurrent access corrupts it. So
the project is exported **once** to flat text, and all downstream analysis (by
humans or by agents) greps those files. Nothing downstream opens Ghidra.

```sh
cp docs/whitebox/scripts/ExportFlat.java ~/ghidra-projects/scripts/
/opt/ghidra/support/analyzeHeadless ~/ghidra-projects/c2map c2map \
    -process c2dll -noanalysis \
    -scriptPath ~/ghidra-projects/scripts \
    -postScript ExportFlat.java ~/ghidra-projects/export/c2

objdump -d -M intel ~/ghidra-projects/bin/c2dll > ~/ghidra-projects/export/c2/objdump_intel.asm
```

Produces, under `~/ghidra-projects/export/c2/`:

| file | size | contents |
|---|---:|---|
| `functions.tsv` | 188 K | `addr size name nparams ncallers ncallees nrefs thunk framesize` |
| `strings.tsv` | 82 K | defined strings + the functions referencing each |
| `data.tsv` | 857 K | defined non-string data + xrefs — this is where the tables live |
| `xrefs.tsv` | 6.0 M | 146 818 references, `from to type from_func` |
| `calls.tsv` | 833 K | call-graph edges, including calls to imports |
| `symbols.tsv` | 389 K | all symbols incl. `EXTERNAL:` import entries |
| `decomp_all.c` | 7.2 M | all 4916 functions decompiled, **0 failures** |
| `objdump_intel.asm` | 22 M | independent disassembly at correct VAs |

`decomp_all.c` and `objdump_intel.asm` are **deliberately not committed** —
bulk decompiled third-party C is exactly the artifact the clean-room posture
should not carry in-tree, and both are regenerated by the commands above in
minutes. The distilled tables (`c2_functions.tsv`, `c2_strings.tsv`) are
committed because they are stable references.

Address arithmetic for byte-level probing of `~/ghidra-projects/bin/c2dll`:

```
imagebase 0x10b00000
.text   VA 0x10b01000  vsize 0x12cc7c   file offset 0x000400   (VA = off + 0x10b00c00)
.data   VA 0x10c2e000  vsize 0x042750   file offset 0x12d200
.rsrc   VA 0x10c71000                   file offset 0x13be00
.reloc  VA 0x10c72000                   file offset 0x13c200
```

## 5. Generated tables

```sh
python3 docs/whitebox/scripts/build_map.py \
        ~/ghidra-projects/export/c2 <labels-dir> docs/whitebox/c2_functions.tsv
python3 docs/whitebox/scripts/build_strings.py \
        ~/ghidra-projects/export/c2 ~/ghidra-projects/bin/c2dll docs/whitebox/c2_strings.tsv
```

`build_map.py` assigns a cluster only from facts that need no judgement —
thunk-ness, which imported DLL a function calls, and an exclusive-caller
propagation pass — then overlays hand-verified labels from `<labels-dir>`.
Everything else is emitted as `unknown`/`unknown`. **`unknown` is a required,
respectable value in this table.** A row you cannot defend must stay `unknown`.

`build_strings.py` merges Ghidra's typed strings (which carry xrefs) with a raw
image scan (which misses nothing). The merge is load-bearing rather than
cosmetic: Ghidra swallowed `.bss` at `0x10b165f8` into a neighbouring string and
only the raw scan recovered it — and `.bss` is worth +402 TUs on the section-shape
ladder.

## 6. Known-answer controls — the gate on publishing labels

A map with 4000 confidently-wrong labels is worse than no map. The project's
standing culture rule is *"absence must never read as success"*; the corollary
here is that **a labeling method you have not tested is not evidence.** Before
any label was published at scale, the method was run against facts the project
already knew completely from black-box work. Each control is a routine whose
behaviour is fully specified in advance, so finding it is pass/fail rather than
interpretive.

<!-- CONTROLS-TABLE-START -->
*(filled in below once every control has been run; see §6.1)*
<!-- CONTROLS-TABLE-END -->

### 6.1 Control results

See the table in [`C2_MAP.md`](C2_MAP.md) §Controls for the scored results and
the hit rate. Misses are reported as misses.
