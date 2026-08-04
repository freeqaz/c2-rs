# BOARD_PROPOSED — rows the `w-map` lane asks the coordinator to mint

> **PROVENANCE — DISASSEMBLY-DERIVED.** These rows summarize static-analysis
> findings. Adopting any of them into `crates/` requires a
> [`DISCLOSURE.md`](DISCLOSURE.md) entry naming the address.

`docs/BOARD.md` is the coordinator's file and this lane does not edit it. Next
free number at the time of writing was **#162**; the coordinator assigns the
actual numbers. Rows follow `BOARD.md`'s format:
`| N | title | payoff | anchors | notes |`.

Statuses used below follow `BOARD.md`'s conventions — in particular **DECLINED is
not failure** (the measurement is the deliverable) and **REFUTED rows are the
most valuable ones**.

---

## Proposed rows

| # | status | title | payoff | anchors | notes |
|---|---|---|---|---|---|
| **162** | DONE | `docs/whitebox/` — a navigational map of `c2.dll` from static analysis | 4916 functions enumerated; 2791 literals with cross-references; a reproducible flat-export pipeline | WB:`C2_MAP.md`, WB:`C2_MAP_METHOD.md` | The lane's container item. Ghidra 12.1.2 headless converged on the 1.35 MB image in minutes and decompiled all 4916 functions with **0 failures**. Two committed reference tables (`c2_functions.tsv`, `c2_strings.tsv`) plus the scripts that generate them. The map is **navigation, not adoption**: `DISCLOSURE.md` is empty. |
| **163** | REFUTED | `ROADMAP.md` §9's "`c2.dll` … is **not a stripped build**" | — | R:§9 (line ~4074), WB:`C2_MAP.md` §Is it stripped? | **The claim is false.** No COFF symbol table (`NumberOfSymbols` = 0), 4 exports total, no RTTI type descriptors, and the CodeView entry is an `RSDS` reference to an *absent* `c2.pdb`. The evidence originally cited for the claim — the `/FAsc` listing — is real but supports a different proposition: c2 is unusually **talkative**, not unusually **symbol-rich**. Correcting this matters because "not stripped" makes white-box work look cheaper than it is. |
| **164** | DONE | The COFF writer lives in `c2.dll`, not in `msobjXX.dll` | Keeps the whole obj-emission map inside one binary | WB:`C2_MAP.md` §msobj | c2 imports **exactly one** symbol from `msobjXX.dll`: `objf::ObjectCode::FCreateFromBytesW` — a *reader*, constructing an `ObjectCode` **from** a byte buffer. Every other msobj export is a reader too (`Image::FCreate`, `Library::FCreate`, `IDebugSSectionReader`, `FunctionSymbols::get`). This was an explicit decline-clause in the lane's brief ("if msobj owns the writer, say so plainly — it relocates the map"); it does not. |
| **165** | ~~OPEN~~ **CORRECTED** | The pointer table at `.data 0x10c37c40` is **not** a section table | Prevents a wrong entry point for factor **C** | WB:`C2_MAP.md` §4A | Proposed as "a 9-slot array of section-name pointers". **That reading is wrong.** It is a table of *default name strings used as identity tokens*: `FUN_10b982d6` compares `sect->name == g_defaultName[k]` to decide whether to substitute the canonical literal for a possibly-`$`-suffixed name. Slot `0x10c37c60` → `0x10b18db4` = `"$$TYPES"`, the DEBTYP section's default name, not a section name. The real entry point for factor C is **`FUN_10b982d6`** — the single place a kind becomes (name, Characteristics). Row kept rather than deleted: the mis-reading is the useful record. |
| **166** | OPEN | Retarget the "split" concept to stripped x86 PE | Turns "which of 4916 functions" into "which of ~200 pseudo-TUs" | WB:`RESEARCH.md` (W-CARVE) | `jeff`'s `xex split` and `dtk` carve **PowerPC** images; `c2.dll` is x86 PE, so neither applies directly. The *concept* — partition a stripped image into per-object units using layout order, literal pooling and symbol bounds — is portable and is the highest-leverage reusable technique found. Priced in the research report. **Superseded in practice by #167**, which achieves the same partition from c2's own ICE strings without needing the technique. |
| **167** | DONE | c2's 52 original translation units, recovered from its own ICE strings | Converts "cluster and hope" into reading module boundaries off the binary; **the map's spine** | WB:`C2_MAP.md` §3, WB:`c2_tus.tsv` | c2's C1001 path prints `compiler file '%s', line %d`, so the binary names its own sources. Link order makes each file's xrefs a contiguous range. **Confidence metric: overlap 1/51 adjacent pairs = 1.96%** (the one overlap is `dbgcpp.h` inside `dbg.cpp`; between distinct TUs it is **0/50**), **gap coverage 28.1% of bytes / 29.2% of functions in-range**. Validated against a null the partition never used: 7 ascending runs vs 26.5 expected (**P = 1.5 × 10⁻²⁵**), longest run 33 files (**1/33!**), every run directory-pure. **Two provenance tiers** — the file-name list is `strings` output and costs the clean-room claim nothing; only the addresses are white-box. |
| **168** | DONE | The emit predicate is `(sym->flags4c & 0x20) && !(flags4c & 0x02)` — and **c2 does not compute it** | The project's most valuable unknown, relocated | WB:`C2_MAP.md` §4D | Walk loop `0x10b7f15f` in `FUN_10b7f1ff` (**`p2/main.c`**, not `coffemit.c`). The flag word is stored to `sym+0x4c` **verbatim from the IL** at `0x10b9bf78` — the base decision is transmitted by `c1xx` in the `.gl` stream. Outside the pruner c2's closure is purely **additive**; the only two sites in the image that clear the bit are `10b27cde` and `10b8a6c6`. **Refutation:** a body in the obj whose symbol never had bit `0x20`, or a non-pruning TU where a marked body is absent. Black-box corroboration: functions provably folded into their caller are **still** emitted out-of-line. |
| **169** | REFUTED | The `.bss` object-address permutation is a hash | Removes the stated premise for the `OBJ_DYNINIT_SHAPE.md` §7.1 decline | WB:`C2_MAP.md` §4C | Brute force over 9 name decorations × every modulus 2..8191 × every `(h>>s)&mask` (`s≤28`, `mask≤16` bits) × {asc,desc} × {FIFO,LIFO} = **0 hits**. The measured rule needs no hash at all: **`.bss` ascending = reverse of `.gl` record order** for dyninit objects, **= `.gl` order** for plain ones, groups never interleaving. Verified 5/5 plus a held-out control. The residual permutation is **c1xx's**, and it is already in the `.gl` the port is handed — so **the port never has to reproduce any hash**. §7.1 should be revisited. |
| **170** | DONE | `DAT_10c45f9c` is the `-optref` flag, not LTCG | Corrects a medium-confidence guess everything else hung off | WB:`C2_MAP.md` §4D | The child that found the emit predicate could not determine what selects prune-vs-additive: **`0x10c45f9c` has no WRITE xref anywhere in the export**, because c2's 147-entry flag table is built at run time by a 4250-byte store sequence at `0x10c2932e..0x10c2a3b5` and is invisible to a `.data` scan. A second child on a different seam reconstructed that table; the join answers it outright. **Two seams closed by intersection what neither could close alone** — the argument for keeping a map rather than isolated findings. |
| **171** | DONE | JamCRC is **absent** from `c2.dll`; the aux `CheckSum` is computed outside it | Prevents a confident wrong label | WB:`C2_MAP.md` §6 P1, WB:`PREREG.md` | Pre-registered control, scored a **MISS**, and the miss is the valuable result. No `0xEDB88320` table at any 4-aligned offset in either bit order; the polynomial immediate occurs nowhere in the image; the `A..P` renderer is absent (the only `ABCDEFGHIJKLMNOP` run is the base64 alphabet). The table lives in `mspdbXX.dll` @ `0x4898`. **Search method itself controlled**: two constants the port hardcodes and a fresh obj demonstrably carries are *also* absent as immediates. c2 reaches the checksum through the callback table at `DAT_10c44bf4…0x10c44c0c`. |

---

## Rows deliberately **not** proposed

* **No row for the COFF symbol-table order.** Three probes gave three different
  answers (all-dyninit → IL order; no-dyninit → *source* order; mixed →
  ascending address). Three points fit any rule. It stays `unknown` in
  `C2_MAP.md` §7 until a probe is built that *discriminates* between the
  candidates rather than accumulating agreements.
* **No row for `.CRT$XCU`'s kind/Characteristics.** The name is proven absent
  from `c2.dll` and present in `c1xx.dll`, so the kind must arrive in the IL —
  but no IL record carrying a section name was traced into
  `FUN_10be7473`/`FUN_10be74cf`. Naming a value here would be a guess, and it is
  the highest-value open item for factor C precisely because it is not settled.
* **No row asking to weaken the clean-room claim.** Nothing has been adopted, so
  nothing needs weakening; [`README_DELTA.md`](README_DELTA.md) proposes a scope
  *note*, not a retraction — and now argues the note can be **narrower** than
  first thought, because the file-name list is `strings` output and incurs no
  debt. If and when `DISCLOSURE.md` gains its first row, that is the moment to
  mint a board item, not before.
* **No row for `color.c` / the register allocator.** Deliberately unmapped: the
  doctrine is I/O-behavioral and register allocation is not on the critical
  path. The Ghidra+LLM first-draft Rust under `crates/c2-core/src/paint/` is
  gitignored scaffolding and explicitly **not truth**; nothing in this lane
  lends it any support.

## A note on #168 and the earlier decision not to propose it

The previous revision of this file said it would propose **no** row for the
emit predicate "unless the child that hunted it returns a finding that survives
a stated refutation condition." It did: an instruction-level read, a stated
refutation, and independent black-box corroboration that *refuted* a competing
intuition. The bar was set in advance and then met — which is the only reason
#168 appears. Recorded here so a reader can see the row was not simply willed
into existence.

*Anchors: `WB:<file>` = a file under `docs/whitebox/`. `R:§x` = a section of
`ROADMAP.md`.*
