# PRIOR_ART — what exists outside this repo, and what it is worth

Lane **w-prior**, 2026-08-04. Research only; no `crates/` change, no gate run.

The question was: *what existing work can we lean on to reduce scope, reduce
complexity, or serve as a reference to port from?* The answer, stated once:

> **Almost nothing reduces scope. One thing reduces cost by ~an order of
> magnitude and nobody here has considered it. And the strategic alternative we
> were asked to price — static recompilation of `c2.dll` — is not merely
> expensive, it is a category error, because there is no emulation layer to
> remove.**

> ### ⚠ 2026-08-21 — **READ THIS PAGE AS A SURVEY, NOT AS A RANKING. ITS MIDDLE CLAUSE RANKS BY THROUGHPUT, AND THROUGHPUT NO LONGER RANKS ANYTHING HERE.**
> *Owner's goal decision,
> [`GOAL_DECISION_2026-08-21.md`](GOAL_DECISION_2026-08-21.md); `CLAUDE.md`
> § "The goal". Annotated by lane `w-goaldocs` — **not one finding, licence,
> URL or measurement below is edited or withdrawn.***
>
> The goal is **perfect reproduction**, for (1) understanding MSVC's internals
> in service of decomp and (2) parity — a 100 % open-source implementation.
> **Ranked the same day** (`GOAL_DECISION` § "AMENDED"; propagated 2026-08-22
> by lane `w-readdocs`): **(1) is primary**, and (2) is also **instrumental to
> (1)**. **The axis this page should be re-read on, then, is what a piece of
> prior art teaches about c2's internals or supplies as an open replacement** —
> a survey ranked by speed is ranked on the one axis that now ranks nothing.
> Applied to this page:
>
> * **"One thing reduces cost by ~an order of magnitude"** — the wibo
>   fork-server (§1.3, §4 row 1) — buys **~10–20× on novel-input throughput at
>   100 % coverage** and, as §4 row 1 says in its own words, *"does not move TU
>   match by one."* Under the retired thesis that made it the headline. Under
>   goal (2) it moves the scoreboard by **zero**, and it is **vendor-backed**,
>   so it cannot serve goal (2) even in principle. It is still worth building
>   if someone wants the speed — it just cannot be recommended *by this page*
>   over work that reproduces c2.
> * **§4's "two caps" framing is UNCHANGED and is the durable half.** Codegen
>   coverage and the emit predicate are the caps under *any* goal, and the
>   page's most load-bearing sentence — *"Nothing in this review touches the
>   emit predicate"* — is a negative result about **reproduction**, not about
>   speed.
> * **§1's category-error finding stands entirely** (static recompilation
>   removes no interpretation layer; there is no ISA gap). It is a fact about
>   `c2.dll`, not an economic argument.
> * **Everything that serves goal (1) is PROMOTED, not demoted**: §3's
>   candidate table, §5's confident negatives (nobody has published a word
>   about the `.ex`/`.gl`/`.sy` grammar), and every reference-quality reading
>   of the format. Under goal (1) that record **is** product.

Everything below is either measured on this box (marked **[m]**) or fetched with
a URL. Provenance and license are given for every candidate because a GPL
reference is a different decision from an MIT one and a leaked source dump is a
hard no.

---

## 1. Static recompilation: closed, and the measurement that closes it

### 1.1 The premise does not hold

`ido-static-recomp` exists because IDO is a **MIPS/IRIX** binary that otherwise
runs under `qemu-irix`. Static recompilation removes an *interpretation* layer.
`XenonRecomp` and `N64Recomp` are the same shape: PPC→x86 and MIPS→x86, crossing
an ISA boundary.

**[m]** `compilers/X360/16.00.11886.00/c2.dll` is `PE32 executable for MS Windows
5.00 (DLL), Intel i386`. wibo is a **PE loader, not an emulator** — every
instruction of `c2.dll` is executed by this CPU natively, in 32-bit mode. There
is no ISA gap. Static recompilation of `c2.dll` would translate x86 to x86 and
buy **zero** instruction-level throughput; the published expectation for
lift-and-recompile in the same ISA family is *performance-neutral to a
regression*, because indirect calls that the original resolved with `call [eax]`
become hash-table lookups the original never paid for.

Neither tool accepts x86 input in any case. `XenonRecomp`'s decoder is a
PPC table; `N64Recomp`'s is MIPS.

### 1.2 Somebody already tried it, for exactly this purpose

[`riptl/mwcc-native`](https://github.com/riptl/mwcc-native) — **GPL-3.0**, 37
commits, last touched 2023-01-30 — is PE→ELF static conversion of the
Metrowerks CodeWarrior *compiler* binaries so GameCube/Wii decomp does not need
Wine. Same motivation, same class of artifact, one ISA over.

Its README ends by recommending `wibo` instead: *"same goals, but uses a runtime
loader vs binary conversion […] is also more stable and actively maintained."*
What it hit, in the author's own words: the `fs`-segment TIB has no Linux
equivalent, so it binary-search-and-replaces the `0x64` prefix with `0x90` — *"no
disassemblers used"* — giving a **process-wide** fake TIB that *"breaks any
multi-threaded applications"*; unwinding across the Win32/SysV boundary breaks
in both directions; **every** `mwasmeppc` build tested crashes with SIGSEGV or
SIGILL. There are no performance numbers in the repo, because performance was
never the point.

`c2.dll` under wibo pulls in `msobjXX`, `mspdbXX`, `msdisXXX`, `pgodb100`,
`msvcr100`, `msvcp100` and is threaded enough to want TLS. It is a strictly worse
candidate than `mwcceppc.exe`, which is the one that failed.

**Verdict: IRRELEVANT-AND-WHY, definitively.** Do not spend a lane on it. Read
`mwcc-native`'s README once for the failure list; do not read its Go, which is
GPL-3.0.

### 1.3 The measurement that closes it also opens the real alternative

If there is no emulation overhead, where does the oracle's ~4.2 ms/obj go?
(`docs/perf/perf_scale.csv`: 236.4 obj/s at concurrency 1.)

**[m]** Timed on this box (`ulimit -c 0`, warm page cache, 20–50 reps each):

| what | per invocation |
|---|---|
| `wibo --version` — bare loader floor | **1.5 ms** wall, 99 % CPU |
| `wibo c2host.exe` + a *failed* `LoadLibrary` | **3.0 ms** wall, 100 % CPU |
| same, + `LoadLibrary(c2.dll)` and its 6 dependent DLLs | **5.8 ms CPU** |
| committed reference replay, whole obj, 1 thread | **4.2 ms** |

The load path alone costs the same order as the entire measured per-obj replay.
**The reference oracle's cost is process spawn plus PE loading, essentially in
full.** `c2.dll`'s actual code generation for a fixture-sized TU is the small
remainder.

Two corollaries, both worth acting on:

* **The README's "it doesn't parallelize well" is not what the committed data
  says.** 236 → 1636 obj/s from 1 → 8 threads is 86 % of linear; the flattening
  at 16 (2253) and regression at 32 (2088) is exactly a 16-core / 32-thread
  Ryzen 7950X **[m]** running out of physical cores. It parallelizes fine. The
  wall is per-obj *cost*, not scaling.
* **The available win is a fork-server, not a port.** Load `c2.dll` and its
  dependents once in a wibo process, then `fork()` per TU. The user already
  maintains a wibo fork (`/home/free/code/milohax/wibo`, `1.0.1-23-g4a9dd6f`),
  so this is implementable where it belongs. Estimated ceiling: most of the
  3.0–5.8 ms fixed cost, at **100 % coverage**, in weeks. That is ~10–20×, not
  1000× — but it applies to every TU, today, including the 863 the port refuses.

**Counterargument, and it is strong:** `work/capture-cache` already amortizes
this for *repeat* workloads — a warm 871-TU scan is ~0.9 s. The fork-server buys
nothing there. It buys only on **novel** inputs, where the cache always misses:
the 14,484-case `expr_sweep`, cold corpus generation, and — the one that matters
— IL-space / permuter-style search, which is the thesis's actual use case. And
it retires **neither** of the two named caps: not codegen coverage, not the emit
predicate. TU match stays at 8.

Unverified risk to settle before committing to it: `c2.dll` may bake per-TU state
(TMP paths, `argv`, `MSC_CMD_FLAGS`) into DLL init, in which case the fork point
has to sit before that and the win shrinks. One probe answers it.

---

## 2. The uncomfortable second finding: Amdahl on the source-level loop

**[m]** Six real dc3 TUs, real workload flags, `cl.exe` under wibo, full compile
vs `/Bd /d2nop` (which aborts `c2` immediately after it loads, so the difference
is `c2`'s share):

| TU | total | front end | **c2** | c2 share |
|---|---:|---:|---:|---:|
| `src/App.cpp` | 2629 ms | 2363 ms | 266 ms | **10 %** |
| `src/lazer/meta_ham/CampaignPerformer.cpp` | 3033 | 1901 | 1132 | **37 %** |
| `src/lazer/meta_ham/SkeletonChooser.cpp` | 1566 | 1334 | 232 | **14 %** |
| `src/system/hamobj/DanceRemixer.cpp` | 1709 | 1283 | 426 | **24 %** |
| `src/system/rndobj/Shader.cpp` | 372 | 260 | 112 | **30 %** |
| `src/system/utl/Licenses.cpp` | 71 | 75 | ~0 | ~0 % |

Also **[m]**: full pipeline marginal cost is **~90–100 µs per trivial function**
(fitted over synthetic TUs of 1 / 100 / 1000 / 4000 functions), on ~40 ms of
fixed cost.

**An infinitely fast `c2` speeds a source→obj compile by 1.1×–1.6×.** For any
workflow that starts from C++ — human matching iteration, `decomp-permuter`,
decomp.me — the c2 port alone is Amdahl-capped there, and the front-end port
(`c1host`, P-F0.x) is not optional but load-bearing.

This does **not** refute the thesis, and the README already implies why: the
target loop is **IL-space search**, which bypasses `c1xx` entirely and where
`c2` is 100 % of the work (`docs/EDIT_MODEL_MVP.md`). But that makes the thesis
conditional on a research bet that has not yet paid, and the conditionality
should be stated where the 568× is quoted.

> **⚠ 2026-08-21 — THE THESIS THIS §2 IS ARGUING WITH IS RETIRED, SO THE
> CONDITIONALITY NO LONGER MATTERS AND THE AMDAHL MEASUREMENT STILL DOES.**
> *Owner, [`GOAL_DECISION_2026-08-21.md`](GOAL_DECISION_2026-08-21.md);
> annotated by lane `w-goaldocs`. **The table above and the ~90–100 µs figure
> are untouched and remain the repo's Amdahl ground truth.***
>
> The goal is **perfect reproduction**, for (1) understanding MSVC's internals
> in service of decomp and (2) parity — a 100 % open-source implementation.
> Throughput is a property. So:
>
> * *"the c2 port alone is Amdahl-capped there"* is still **true** and still
>   the right thing to say beside any speedup claim; it is no longer a
>   statement about whether the port is **worth building**.
> * *"the thesis is conditional on a research bet that has not yet paid"*
>   is **moot** — there is no longer a thesis for the bet to be conditional
>   on. The IL-space regime was subsequently stood down in both repos
>   (`ARCH_REVIEW_2026-08-21.md` §7), which under the old framing would have
>   been bad news for the project and is now simply a fact about a consumer.
> * The recommendation in the last sentence — **state the measurement
>   population wherever a ratio is quoted** — **survives in full** and was
>   discharged in `README.md` (`STRATEGY_REVIEW_2026-08-13.md` §7 item 5). A
>   demoted metric is not an unpoliced one.

---

## 3. Candidate table

Verdicts: **A** = ADOPT-AS-REFERENCE, **R** = READ-ONCE, **X** =
IRRELEVANT-AND-WHY.

### 3.1 Static recompilation / rehosting

| name | URL | license | gives | does NOT give | |
|---|---|---|---|---|:-:|
| ido-static-recomp | decompals/ido-static-recomp | **NO LICENSE FILE** (= all rights reserved) | the genre's design; the *motivation* is removing QEMU | any x86 path; any speed here — no emulation to remove. Unlicensed, so "port from it" has no grant | **X** |
| N64Recomp / XenonRecomp | N64Recomp/N64Recomp, hedge-dev/XenonRecomp | MIT | MIPS/PPC→C++ lifters; XenonRecomp's `xbox.h` + README are the best public prose on the **Xenon ABI and millicode** (`__savegprlr_14` etc.) | an x86 front end. XenonRecomp vendors **GPL-2+ binutils** in `thirdparty/disasm/` — do not read that subtree | **R** (`xbox.h` only: **A**) |
| riptl/mwcc-native | riptl/mwcc-native | **GPL-3.0** | the abandoned experiment §1.2; read the README's failure list | working software; its author points at wibo | **X** |
| mcsema / remill | lifting-bits/* | AGPL-3.0 (archived) / Apache-2.0 | x86 lifting exists | anything usable; AGPL, archived 2022 | **X** |
| **wibo** | decompals/wibo | **MIT** | already the host. Upstream is **1.2.0**; this fork is **1.0.1-23**. 1.1.0 = per-ABI msvcrt, 1.2.0 = "improve kernel32 compatibility for MSVC tools", DLL forwarding, module-TLS-init race fix. 1.0.1 itself was cut *because `msvc_ppc_16.00.11886.00` segfaulted* | it does not fork-serve today — that is ours to write | **A** |
| evmar/retrowin32 | evmar/retrowin32 | Apache-2.0 | a second, well-documented Win32-on-Linux/mac design | nothing wibo lacks for this workload | **R** |

### 3.2 clang / LLVM

| name | license | gives | does NOT give | |
|---|---|---|---|:-:|
| **`llvm-readobj` / `llvm-objdump` via a 1-byte machine patch** | Apache-2.0 w/ LLVM-exc | LLVM rejects `0x01F2` outright (`identify_magic` has no `0xF2` case). Patch a **scratch copy** to `0x01F0` and everything structural parses: aux `AuxSectionDef {Length/RelocationCount/Checksum/Number/Selection}`, and **`--codeview` fully decodes `.debug$S`** — `S_OBJNAME`, `S_COMPILE2 {Machine: PPC604, FrontendVersion: 16.0.11886}`. A free `.debug$S`/`.debug$T` decoder and aux cross-check | reloc *type names* (no `Triple::ppc` arm — prints `Unknown`); CodeView register names come out **x86** for a PPC CU | **A** |
| `llvm-mc -triple=powerpc` | same | verified encoder/decoder oracle for the scalar/FP PPC c2-rs emits today: `lis`→`3d 60 00 00`, `stwu 1,-80(1)`→`94 21 ff b0`, `mflr 12`→`7d 88 02 a6`, BE correct | **VMX128 — and it fails silently.** Opcode 6 collides with Power10 `lxvp`: `vrlimi128` (`18000710`) decodes as `lxvp 0, 1808(0)`, extended opcode read as a displacement, no diagnostic. Opcode 4 collides with AltiVec (`lvx128`→`vucmprlb`). Only opcode 5 hard-errors. Confirms `docs/ABI_EDGES.md:300` and sharpens it | **A** (scalar) / **X** (VMX128) |
| LLVM COFF **writer** | same | — | no PPC path at all: `COFF.h` has `0x1F0`/`0x1F1` and **no `RelocationTypesPPC` enum**; no `PPCWinCOFFObjectWriter.cpp`. LLVM cannot write PPC COFF | **X** |
| `MicrosoftMangle.cpp` | same | `mangleStringLiteral` reproduces `??_C@_03FIKCJHKP@abc?$AA@` **byte-identically** (verified), and `__real@3f000000` in `.rdata` COMDAT-`discard` matches c2's constant-pool naming | general MSVC mangling — c2 copies names verbatim from `.gl`; mangling is c1xx's job, so it is out of scope for this port | **A**, narrowly |
| LLVM PowerPC backend | same | — | 20 years of Power7–Power10 tuning. It will not tell you what a 2007 MSVC `/Ox` does | **X** |
| LLVM Win EH | same | — | `MCWin64EH` is x86_64/ARM/ARM64 only. **No PPC `.pdata`/`.xdata` anywhere in LLVM.** Windows-on-PPC predates Win64EH | **X** |
| Xbox 360 / VMX128 LLVM target | — | — | **does not exist.** No `+vmx128`, no Xenon subtarget, no out-of-tree fork. The 2007 binutils VMX128 patch never landed (FSF assignment + a copyright objection to RE'd ISA info) | **X** |

### 3.3 Xbox 360 / MSVC RE community

| name | URL | license | gives | does NOT give | |
|---|---|---|---|---|:-:|
| **Geoff Chappell, CL option pages** | geoffchappell.com/studies/msvc/cl/ | personal site, all rights reserved (documentation, freely citable) | `/Bk[pathname]` keeps intermediates and *becomes* the `-il` argument; `/BK <path>` **resumes where the front end left off** — Microsoft's own documented front-end/back-end split, i.e. our replay seam, sanctioned. Also names **seven** suffixes: `db ex gl in lk md sy` — **`lk` and `md` are two we do not use** | any format detail | **A** |
| assarbad/msvc-undoc | github.com/assarbad/msvc-undoc | **Unlicense** | a mechanical switch-table extractor + the `c2switch_t` struct layout in `NOTES.md` | **nothing we don't have.** `docs/whitebox/labels/W-FLAG.tsv` and `c2_strings.tsv` (2795 rows) already carry this; `docs/whitebox/c2_tus.tsv` already carries the 52 `be\p2\…` module names incl. `reader.c`, `coffemit.c`, `ppc/lower.c`, `ppc/mdlist.c`. **Orient before you measure — this thread rediscovered our own file** | **R** |
| Quarkslab XFG writeup | blog.quarkslab.com | article | the only public IL-internals identifiers anywhere: `XfgIlVisitor::visit_I_XFG_HASH(tagILMAP*)` — implies an `I_*` opcode enum and visitor walkers | four identifiers, on a 2020 x64 `c2.dll` | **R** |
| MS public symbol server | msdl.microsoft.com | — | **nothing. Verified empirically**: `c2.pdb`, `c1xx.pdb`, `c1.pdb`, `cl.pdb`, `link.pdb` all **404** on their real RSDS GUID+age, while `ntdll.pdb` and `kernel32.pdb` returned 200 with payload in the same session. Microsoft does not publish toolchain symbols | — | **X** (but a clean negative worth recording) |
| Biallas `vmx128.txt` | biallas.net/doc/vmx128/vmx128.txt | none stated; RE'd from `dumpbin` | **every** public VMX128 table descends from this one 2006 file — byte-identical to xenia's copy. 75 real instructions; split-register encoding `VD = (VDh<<5 \| VD128)`, `VA = (A<<6 \| a<<5 \| VA128)`, `VC` only 3 bits | correctness guarantees — 3 known errata in its headers | **A** when VMX128 lands |
| xenia `ppc-instructions.xml` + `ppc-table-gen` | xenia-project/xenia | **BSD-3** | 456 instructions with base opcode words and per-form bit ranges — an encoder, permissively licensed, strictly preferable to binutils | Xbox 360 obj/XEX *format* docs — xenia has none, the knowledge is code-only | **A** |
| `0dinD/ghidra` branch `vmx128` | fork of Ghidra | **Apache-2.0** | the only real VMX128 SLEIGH (`vmx128.sinc`, 536 lines); documents a genuine `stvepx`/`lvrxl` encoding collision | upstream Ghidra still has none (issue #2094 open since 2020) | **A** |
| emoose/idaxex | github.com/emoose/idaxex | **BSD-3** | best XEX structs; and the one 360-specific `.pdata` refinement not in any MS doc — the two flag bits merged into `FunctionType {SaveMillicode=0, NoHandler=1, RestoreMillicode=2, Handler=3}`, i.e. `ThirtyTwoBit=0` repurposed to tag millicode | `.xdata` content | **A** |
| encounter/powerpc-rs | github.com/encounter/powerpc-rs | **Apache-2.0** | Rust PPC disassembler **and assembler** with a Xenon/VMX128 extension set, fuzzed over all 2^32 encodings | a dependency (std-only) — read/cross-check only | **A** |
| **MS PE spec, `.pdata` section** | learn.microsoft.com/…/pe-format | MS docs | the ARM/PowerPC/SH3/SH4 CE-era `RUNTIME_FUNCTION`: `BeginAddress` + packed `PrologLength:8 \| FunctionLength:22 \| ThirtyTwoBit:1 \| ExceptionFlag:1`; handler record `{pHandler, pHandlerData}` precedes the function in `.text` | bit *ordering* within the BE dword (infer + verify); and it says nothing about `.xdata` | **A** |
| binutils `bfd/coff-ppc.c`, and the 2007 VMX128 gas patch | — | **GPL** | the only complete PPC-COFF reloc implementation; the only public VMX128 assembler | usable code for us. **Quarantine.** Use `gas -mvmx128` as a *black-box oracle* if ever needed; never read-and-transcribe | **X** |
| leaked Xbox 360 XDK | archive.org, BetaArchive | **NDA/tainted** | context only: it circulates as **binaries**, not source. (The May 2020 *source* leak was the original Xbox, not the 360.) The CHM docs inside are copyrighted NDA material | anything adoptable. **Do not read, do not cite** | **X** |
| Ghidra upstream, IDA stock | — | — | — | neither decodes VMX128; IDA decodes the `VA` split field **wrong** (which is why `Goatman13/ida_vmx128_helper` exists — unlicensed) | **X** |

### 3.4 The matching-decompilation ecosystem

| name | URL | license | gives | does NOT give | |
|---|---|---|---|---|:-:|
| **encounter/xbox360-binutils** | github.com/encounter/xbox360-binutils | **NO LICENSE** on the repo; new files carry **FSF GPL headers** | `bfd/doc/xenon-ppc-coff.txt` states in prose what no spec does: *"PE/COFF with little-endian headers, big-endian data"*, a 19-entry `IMAGE_REL_PPC_*` table incl. `SECREL16 0x0F` / `SECRELHI 0x14` / `GPREL 0x15`, and 89 VMX128 instructions. Builds `powerpc-xenon-pe-objdump` — **an independent decoder to cross-check `crates/c2-obj` against** | permission. **Read the `.txt`; quarantine `coff-ppc-be.c` / `pe-ppc-be.c` / `tc-ppc.c`.** Use the built binary as a black box | **A** (doc + binary) / **X** (source) |
| **decomp.me `xbox360` platform** | decompme/decomp.me PR #1701 | MIT | `msvc_ppc_16.00.11886.00` is a first-class decomp.me compiler as of 2026-02. Confirms the oracle's acquisition path is public and stable — and **we already fetch from it**: `scripts/fetch_compilers.sh` lists `files.decomp.dev/compilers_${tag}.zip` as its fallback | a port; there is none, anywhere. Their pipeline is the vendor binary under wibo | **A** (context) |
| encounter/objdiff | github.com/encounter/objdiff | **MIT OR Apache-2.0** | the only ecosystem tool that reads COFF — via `object`. `arch/ppc/` already selects `powerpc::Extensions::xenon()` for COFF and handles REL24/REL14/REFHI/REFLO/PAIR, with the comment that *PAIR carries the REF{HI,LO} displacement instead of a symbol index* — **independent corroboration of `crates/c2-core/src/coff/reloc.rs`**. `objdiff.json` is a ready-made frontier-reporting schema | a verifier. Its "match" is normalized-instruction, not byte-exact. Never the gate | **R** |
| rjkiv/jeff | github.com/rjkiv/jeff | **Apache-2.0** | an Xbox 360 decomp toolkit forked from decomp-toolkit, **originally built for a Dance Central 3 decomp**; its `xex split` *emits* 360 PPC COFF objs. Closest sibling project on this planet | a compiler, or an emit predicate | **A** |
| decomp-permuter | simonlindholm/decomp-permuter | MIT | one blocker to COFF: `src/objdump.py:285` parses an ELF header in `get_arch()`. Sniff `Machine == 0x01F2`, point `objdump_command` at `powerpc-xenon-pe-objdump`, done | a verifier — it scores *fuzzy* similarity with fixed penalties (`PENALTY_REGALLOC=5`, `PENALTY_REORDERING=60`) and ignores stack offsets. Structurally the opposite of our rule | **R** |
| decomp-toolkit, splat, ppcdis, mapfile_parser, asm-differ | — | Apache-2.0 / MIT / MIT / MIT / Unlicense | — | **no COFF anywhere.** dtk is DOL/REL/RSO/ELF; splat is MIPS-only; `mapfile_parser` has no MSVC `/MAP` | **X** |
| m2c | matt-kempster/m2c | **GPL-3.0** | — | wrong direction (asm→C), text-only, and GPL | **X** |
| openblack/bw1-decomp | github.com/openblack/bw1-decomp | CC0-1.0 | proof the dtk+objdiff stack **forks cleanly to MSVC PE/COFF** (MSVC 6.0 SP4/SP5, SHA-1 verified) | PPC, or byte-exact obj as the gate | **R** |
| i686.me `csplit` | i686.me/blog/csplit/ | article | recovering TU boundaries from PDB **section contributions** — the linker records each input COFF object's sections. Also: MSVC folds arithmetic into the relocation *addend* (a DIR32 whose applied value points *before* the symbol) | anything about c2's emit decision | **R** |

### 3.5 COFF references to read

**The settled question first.** COFF *structures* are **little-endian** even in a
big-endian PowerPC object; only section *contents* are BE. **[m]** A real dc3
obj parses correctly with `struct.unpack('<HHIIIHH')`: `machine=0x01f2
nsect=494 nsym=2159`. A corpus sweep found 543,594 objects at `0x01F2`, every one
with `Characteristics == 0x0180` (`32BIT_MACHINE | BYTES_REVERSED_LO`); zero set
`BYTES_REVERSED_HI`. **No revision of the PE/COFF spec ever states this**, which
is why implementations get it wrong.

| name | license | gives | does NOT give | |
|---|---|---|---|:-:|
| **gimli-rs/object**, `src/pe.rs:1414-1477` | **Apache-2.0 OR MIT** | the most complete `IMAGE_REL_PPC_*` table in existence — all 23, including four MS never documented (`TOCREL16 0x08`, `TOCREL14 0x09`, `IFGLUE 0x0D`, `IMGLUE 0x0E`), `SECRELHI 0x14`, and the modifier flags (`TYPEMASK 0x00FF`, `NEG/BRTAKEN/BRNTAKEN/TOCDEFN`) — i.e. `Type` is a **packed word, not an enum**. Knows `0x01F2` (PR #783, "Used in Xbox 360 COFF"). Also correct on the **archive** trap: first linker member BE, second LE, in the same `.lib` | a dependency (std-only). **Trap:** `is_little_endian()` returns `false` for POWERPCBE — that describes *payload*, while every struct field is a hardcoded `U16<LE>`. Read both together or you conclude the opposite. Its `write::coff` panics `"unimplemented architecture"` on PPC relocs | **A** |
| MS PE/COFF **rev 6.0** (Feb 1999) | MS | the authoritative text for `SECRELHI 0x0014` and for PAIR's load-bearing semantics — *SymbolTableIndex contains a displacement, not an index* (our objs: every PAIR has `sym=0`) | `0x01F2`; bigobj | **A** |
| MS PE Format, current | MS | 17–18 live `IMAGE_REL_PPC_*`; the `.pdata` CE format (§3.3). The PowerPC section was **not** deleted, contrary to a common claim — verified by direct fetch | `0x01F2` (**no revision has it**); the LE-header rule; bigobj; COMDAT *ordering*; and `SECRELHI` is still dangling-referenced under PAIR with no row — a live spec bug | **A** |
| microsoft/microsoft-pdb `cvdump.cpp` | MS-PL-ish, published by MS | the **only** official Microsoft artifact anywhere defining `IMAGE_FILE_MACHINE_POWERPCBE 0x01F2`, and it routes it to the standard 20-byte `IMAGE_FILE_HEADER` with no byte-swapping — a second confirmation of the LE-header rule | more | **A** |
| LLVM `BinaryFormat/COFF.h` | Apache-2.0 w/ LLVM-exc | the de-facto **bigobj** spec (`BigObjMagic`, `Header32Size=56`, `Symbol32Size=20`, `MaxNumberOfSections16=65279`) — Microsoft documents bigobj **nowhere** | PPC anything | **R** |
| goblin | MIT | a second opinion on COMDAT aux records | `0x01F2`; no PPC reloc constants. Use `object` | **R** |
| pe-parse | MIT | a cautionary tale: it *defines* `0x1F2` and then, 12 lines later, byte-swaps all structure fields on `BYTES_REVERSED_HI` while testing `0x1F0` — an active bug that happens never to fire, because real 360 objs set `_LO` | anything correct here | **R** |
| pelite | MIT | — | **no `.obj` support at all** (PE images only, host-endian transmutes) | **X** |

### 3.6 The "reimplement a proprietary compiler byte-exactly" genre

**Nobody has done this.** Every prior effort in the decomp world chose
*preserve-and-rehost* (wibo, wine, dosemu2, qemu-irix, static recompilation) over
*reimplement*. There is no reimplementation of `mwcc`, of IDO's `ugen`/`uopt`, of
ASPSX's codegen, of armcc, of any MSVC component, by anyone.

| name | class | why it is not what we are doing | |
|---|---|---|:-:|
| agbcc, decompals/old-gcc, SN64-gcc, ee-gcc, armcc-under-wine | **patched original / archival** | the compiler *is* the original binary or source; byte-exactness is trivial. Validation is a downstream whole-ROM SHA-1, not a compiler-level oracle | **X** |
| ido-matching-decomp | **decompilation of** the compiler | targets IDO's `uopt`/`ugen` — the components analogous to c2 — which are **written in Pascal**. Every ELF column, both versions, still unmatched. No LICENSE | **R** |
| **mkst/maspsx** | **MIT** — the closest analogue | a post-processor between `gcc -S` and GNU `as` that reimplements ASPSX's *macro expansion and `$at`/`$gp` policy* — explicitly **not** codegen, regalloc or scheduling; its premise is that *"ASPSX does not appear to do very much in terms of code optimisation."* Its `aspsx/` dir is the structural match to our harness (13 ASPSX versions under dosemu2/wine, a hand-rolled `LNK\x02` parser). **But `aspsx/` is not in CI, and expectations are hand-transcribed instead of captured from the oracle** — precisely the failure mode our memory records as *`gate.sh` cannot see `expr_sweep`*, and it is why maspsx's divergences arrive as user bug reports | **A** as a cautionary design |
| **Camlboot** | **right criterion, wrong mechanism** | an independent OCaml implementation whose success criterion is *bit-for-bit identical* object files, in two human-months. Its key observation transfers exactly: bytecode objects carry **side-data — constant tables, debug info — serialized by the runtime's marshaller and embedded as-is**, which the reference gets free and an independent implementation must replicate bit-for-bit. That is our COFF-metadata problem restated. Caveat: it reaches bit-identity by *interpreting the reference compiler's own source*, so it never solves instruction selection | **A** |
| GCC `make bootstrap`, `bootstrap-debug` | the canonical byte-exact compiler compare | `contrib/compare-debug` compares **stripped** objects, to verify the same code is generated with and without debug info. That is a disciplined second tier below our strict compare, and it separates *codegen* divergence from *metadata* divergence — directly useful to the A/B/C/D factorization | **A** |
| US 5,754,860 / US 8,825,689 | patents | the first is the formal statement of "the compiler is the sole judge" (*no preliminary determination that either output is correct is required*). The second adds **classifying** each disparity by type — which our A/B/C/D factorization already is an instance of, and which nobody has published applying to *parity* rather than bug-finding | **R** |
| chibicc, tcc, cproc, qbe, 8cc | — | none targets reference parity; `qbe` names it an anti-goal | **X** |
| csmith, EMI, yarpgen, Alive2 | — | all compare *behavior* or formal semantics, never object bytes | **X** |
| "Systematic Impact Study for Fuzzer-Found Compiler Bugs" (arXiv 1902.09334) | paper | the citation that defends our rule: across Debian, triggered miscompile paths produced **318 bitwise diffs and 0 test failures**. The field rejects byte-equality as *too strong for bug-finding* — which is exactly why it is right for a *parity* problem | **R** |

---

## 4. Scope reduction, priced against the two caps

The caps are **codegen coverage** and **the emit predicate**. Be suspicious of
any candidate that claims to touch either.

| # | candidate | codegen coverage | emit predicate | what it actually retires |
|---|---|---|---|---|
| 1 | **wibo fork-server** (build on MIT wibo, in the existing fork) | **nothing** | **nothing** | ~10–20× on novel-input throughput at **100 % coverage**. It does not move TU match by one. Its value is that it is the *only* candidate that helps the 863 refused TUs at all — and it helps them by not needing the port |
| 2 | **LLVM tooling via the `0x1F2→0x1F0` scratch patch** + `object`'s `pe.rs` reloc table + `powerpc-xenon-pe-objdump` | **indirectly, materially** — a free `.debug$S`/`.debug$T` decoder, an aux-record cross-check, a verified PPC encoder oracle, and an *independent second implementation* of COFF reloc decoding to disagree with `crates/c2-obj`. This shortens the diagnose step of every codegen and section-shape rung, which is where factor **C** (114 → 871) is won | **nothing** | no rung, but a constant-factor cut on every rung's debugging. Cost: hours |
| 3 | **Chappell `/Bk` + `/BK`** and the Xenon `.pdata`/ABI reference set (idaxex `FunctionType`, XenonRecomp `xbox.h`, MS PE `.pdata`, Biallas/xenia VMX128) | **yes, for two specific future rungs** — `.pdata` shape (factor C's `.xdata$x`, worth 871 in C's ladder) and, much later, VMX128 encoding | **nothing** | `/BK` may simplify the capture seam (Microsoft's own resume-from-IL switch, vs. our `/Bd`-scrape + `c2host` reconstruction). The `.pdata` bitfield is documented and does not need RE. **`.xdata` content is not documented and does need RE** |

**Nothing in this review touches the emit predicate.** That is the report's
most load-bearing negative result: the emit decision lives in `c2.dll`'s
`be\p2\{fg,inline,ltcg,reader}.c`, nobody outside this repo has published a word
about MSVC's IL or its emission rules, and the +82 TUs it is worth remain
entirely ours to earn.

---

## 5. Searched for, found nothing

Each of these was searched deliberately. A confident negative is a result.

1. **The `.ex` / `.gl` / `.sy` / `.in` / `.db` opcode grammar or record layout.**
   Not documented by anyone — not Chappell (who names the files but not their
   contents), not the VB6-era literature (whose author says outright he has no
   idea what they contain), not lectem, not assarbad, not Quarkslab, not any
   GitHub repo, gist, 010 template, IDA loader or Ghidra plugin. The `/GL` LTCG
   variant of the same IL (`.cil$` / `.cil$fg` in `c2.dll`'s strings) is equally
   undocumented. **The RE work is unavoidable.**
2. **Any reimplementation of any MSVC component**, by anyone, ever.
3. **Any byte-exact differential harness for a compiler**, in decomp,
   reproducible-builds, or the academic literature. maspsx's `aspsx/` is the
   nearest and it is out of CI with hand-written expectations.
4. **PDBs for `c2.dll` / `c1xx.dll` / `c1.dll` / `cl.exe` / `link.exe`** — all
   404 on `msdl.microsoft.com` against their real RSDS GUID+age, with two
   known-good public PDBs returning 200 in the same session as the control.
5. **`IMAGE_FILE_MACHINE_POWERPCBE = 0x01F2` in any Microsoft specification** —
   absent from every PE/COFF revision, from Win7 SDK and Win10 1607 `winnt.h`,
   and from LLVM. `cvdump.cpp` is the sole official citation.
6. **Any spec statement that COFF structures stay little-endian in a BE object.**
   The one thing implementers need is the one thing never written down.
7. **COMDAT section *ordering* semantics.** Silent in the spec, LLVM
   (`assignSectionNumbers()` is naive insertion order), `object`, goblin,
   pelite, pe-parse and objdiff. The spec's `$`-lexical rule governs the
   *linker's image layout*, not the *compiler's object layout*. **Board #259
   (packed `.text` order ≠ `.ex` segment order) has no external answer and never
   will.**
8. **bigobj**, documented by Microsoft nowhere; LLVM's `COFF.h` is the de-facto
   spec.
9. **Xbox 360 `.xdata` / `.xdata$x` content.** The MS `.pdata` record is
   documented; whatever `pHandlerData` points at is not, by anyone. (Note: the
   claim "PowerPC has no `.xdata`" is **wrong at the obj level** — `c2.dll`'s
   section-name table contains `.xdata` and `.xdata$x`, and real objs carry
   `.xdata$x` with ADDR32 relocations.)
10. **Xbox 360 XDK linker/compiler switch documentation**, `/MACHINE:PPCBE`,
    LTCG for 360, the `imagexex` post-link step — nothing public. The only
    "documentation" is the CHM inside the XDK, which is NDA material and out of
    bounds.
11. **A fork-server / process-reuse design in decomp.me or decomp-permuter.**
    Both scale horizontally instead (`-j`, permuter@home, more instances). So the
    fork-server in §1.3 has no prior art to copy either — but also nobody has
    found a reason it doesn't work.
12. **Any Xbox 360 *matching* decomp with a byte-exact obj gate** besides this
    project's own. decomp.me's `xbox360` platform and `rjkiv/jeff` are the only
    other 360 matching-decomp presences, and neither has a compiler port.

---

## 6. Corrections to things this lane was told or assumed

* **`docs/whitebox/` already contains what two external threads "found"** — the
  52 `be\p2\…` module names (`c2_tus.tsv`), the switch table (`labels/W-FLAG.tsv`),
  2795 strings (`c2_strings.tsv`), 4920 functions. Orienting first saved a lane
  from re-importing its own output.
* **`scripts/fetch_compilers.sh` already fetches from `files.decomp.dev`** — the
  "the oracle binary is publicly distributed" finding is ours already.
* **The PowerPC section of the PE/COFF spec was not deleted.** It is on the
  current MS Learn page. What was dropped after rev 6.0 is `SECRELHI 0x14` alone,
  and its cross-reference under PAIR survived the deletion.
* **`llvm-mc -triple=powerpc` does not merely "mis-decode" VMX128** — for
  opcode-6 forms it emits a *plausible legal* Power10 `lxvp` with the extended
  opcode read as a displacement, with no diagnostic. `docs/ABI_EDGES.md:300`
  should say "silently".
