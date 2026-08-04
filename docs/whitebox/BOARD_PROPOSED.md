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
| **165** | OPEN | The section-name pointer table at `.data 0x10c37c40` | Entry point for factor **C** (section shape), the tightest factor at 84/871 | WB:`C2_MAP.md` §Sections | A 9-slot array of section-name pointers — `.pdata .xdata .data .bss .rdata .text .debug$S .tls$` + one unidentified. Slots are written around `0x10be75de..0x10be7c60` and read around `0x10b98318..0x10b98442`. `.bss` alone is worth **+402 TUs** on the greedy section ladder. |
| **166** | OPEN | Retarget the "split" concept to stripped x86 PE | Turns "which of 4916 functions" into "which of ~200 pseudo-TUs" | WB:`RESEARCH.md` (W-CARVE) | `jeff`'s `xex split` and `dtk` carve **PowerPC** images; `c2.dll` is x86 PE, so neither applies directly. The *concept* — partition a stripped image into per-object units using layout order, literal pooling and symbol bounds — is portable and is the highest-leverage reusable technique found. Priced in the research report. |

---

## Rows deliberately **not** proposed

* **No row for "the emit-set predicate is at 0x…"** unless the child that hunted
  it returns a finding that survives a stated refutation condition. A board row
  is a durable claim and the project's failure mode here would be a confident
  address that later turns out wrong — every agent downstream would build on it.
* **No row asking to weaken the clean-room claim.** Nothing has been adopted, so
  nothing needs weakening; [`README_DELTA.md`](README_DELTA.md) proposes a scope
  *note*, not a retraction. If and when `DISCLOSURE.md` gains its first row, that
  is the moment to mint a board item, not before.

*Anchors: `WB:<file>` = a file under `docs/whitebox/`. `R:§x` = a section of
`ROADMAP.md`.*
