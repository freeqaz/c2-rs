# WB-D `wb-regalloc` — PREREG

> **PROVENANCE — DISASSEMBLY-DERIVED.** See
> [`C2_MAP_METHOD.md`](C2_MAP_METHOD.md) §0 for the exact bytes and
> [`DISCLOSURE.md`](DISCLOSURE.md) for what adoption costs. Nothing here is
> adopted into `crates/`.

Registered **before the first grep of `~/ghidra-projects/export/c2/`** and
before the first `cl.exe` this lane authored, per board #770's standing rule
(running streak ~10 optimistic / 2 pessimistic / 1 hit). Scored in
[`WB_REGALLOC_FINDINGS.md`](WB_REGALLOC_FINDINGS.md) §8.

Lane: `wb-regalloc` / branch `worktree-agent-a4ebb1b635c41c11c`, branched at
master `cfd972c`. Image sha256 verified at the top of this lane:
`c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`.

**What was read before freezing** (in-repo only, no export, no disassembly):
`CAMPAIGN_2026-08-08.md`, `CAMPAIGN_2026-08-08_GENERATORS.md`,
`C2_MAP_METHOD.md`, `WB_FRAME_FINDINGS.md`, `labels/W-EMIT.tsv`,
`labels/W-FRAME.tsv`, the `w-osfinfo` and `w-xlr` rungs (§1–§4 of each),
`WB_READER_PREREG.md` for format. Nothing in
`~/ghidra-projects/export/c2/` has been opened.

---

## P0 — the success floor

| # | prediction | registered direction if wrong |
|---|---|---|
| P0.1 | This lane clears its floor: **at least one of {register policy, ordering policy} survives a frozen obj-check on ≥1 function outside every shipped class.** | optimistic |
| P0.2 | Of the two, **ordering** survives and **register assignment** does not. Order is a property of one traversal; register identity is a property of a whole-function liveness computation the port does not have. | — |

## P1 — where the stages are

| # | prediction | direction if wrong |
|---|---|---|
| P1.1 | c2 has a **separately-named TU band for register allocation** in `c2_tus.tsv` — a `p2\…` file whose name contains `reg`/`alloc`/`color`/`live` — distinct from `code.c`, `lower.c`, `dag.c`, `mdlist.c`. | optimistic |
| P1.2 | c2 has a **separate instruction-scheduler band** (`sched`/`sch`/`stall`), and it is real rather than vestigial — the `/QXSTALLS` listing writer at `10b71d8f` is its narrator. | optimistic |
| P1.3 | Instruction selection is **not** a separate band: it is fused into the lowering walk (`lower.c`) that builds the machine instruction list `FUN_10bff507` later scans. | pessimistic |
| P1.4 | The register allocator is a **global** allocator over the whole function (a liveness/interference computation), **not** a per-expression-tree local assignment. | optimistic |

## P2 — the register-choice policy (stated cold)

| # | prediction | direction if wrong |
|---|---|---|
| P2.1 | Temporaries are allocated from the **volatile GPRs in DESCENDING order starting at r11**, with **r12 reserved** (the LR shuttle: `mflr r12` / `mtlr r12` in every prologue the project has seen). So the first scratch is r11, the second simultaneously-live scratch is r10, then r9… | — |
| P2.2 | **That descent is exactly the w-osfinfo r11→r10 fact**: r10 appeared where a shipped walk expected r11 because a *previously materialised value was still live in r11* at that point (`lis r11` / `lwz r11,0(r11)` for `<limit>`, still live when `<table>`'s high half is formed). Not a per-form or per-relocation rule. | optimistic |
| P2.3 | Argument registers r3..r10 are assigned **positionally by the ABI at the call-lowering site**, before the allocator runs, and appear to the allocator as pre-coloured fixed defs/uses. | — |
| P2.4 | Callee-saved GPRs are allocated **ASCENDING from r31 downward** — i.e. the first callee-saved value taken is **r31**, the second r30 (this is what `10bfebf7`'s `0x20 → 0x0f` descending store loop implies, and what every shipped transcription shows: a single saved register is always r31). | — |
| P2.5 | CR field choice: **CR0 by default**; a non-zero CR field is used only when two comparisons are simultaneously live, or when the compare feeds something other than the immediately following branch. `cr6` is *not* preferentially used for integer compares. | — |
| P2.6 | Signedness (#1788, `cmpwi`/`cmplwi`) is decided at **instruction selection**, from the IL TYPE byte, and is **not** revisited by the allocator or the scheduler. The consumer is one table/switch in the lowering walk. | — |
| P2.7 | Spilling exists but is **not reachable at `/O1` on the reach-pool's function shapes** — no function in the reach-pool spills a GPR to a stack slot the allocator invented (as opposed to an address-taken local). | optimistic |

## P3 — the ordering policy (stated cold)

| # | prediction | direction if wrong |
|---|---|---|
| P3.1 | The emitted word order is a **linear walk of one per-function doubly-linked instruction list**, in block order — the same list `FUN_10bff507` linearly scans for the prologue flag word. There is no second ordering data structure at emit time. | — |
| P3.2 | The **block order** is the order the blocks were CREATED by the flow-graph builder — i.e. source order, reverse-postorder-equivalent on reducible CFGs — and is not recomputed. This is why the label counter is a stable ladder (`LABEL_COUNTER.md`). | — |
| P3.3 | **Within a block, the order is NOT simply IR construction order**: a list scheduler reorders for the in-order PPC pipeline (load-use latency, no dual-issue hazards). Registered as the *harder* prediction on purpose. | optimistic |
| P3.4 | If P3.3 is right, the scheduler is **off or trivial at `/O1`** and only turns on with `/O2`, so the reach-pool (compiled `/O1`) sees construction order anyway. | optimistic |
| P3.5 | Nothing in c2's ordering depends on register identity — **selection → order → registers**, in that order — so a port can reproduce order without reproducing allocation. | optimistic |

## P4 — the obj-check (deliverable 4)

| # | prediction | direction if wrong |
|---|---|---|
| P4.1 | ≥3 functions outside every shipped class are graded. | — |
| P4.2 | On **instruction order** (word-kind sequence, registers ignored): **≥2 of 3 hit**. | optimistic |
| P4.3 | On **register assignment** (exact register numbers): **≤1 of 3 hit**. | pessimistic |
| P4.4 | At least one prediction is a **miss that forces a retraction** of something written in §2–§3 of the findings. This lane registers in advance that it expects to retract. | — |

## P5 — the judgment (deliverable 5)

| # | prediction | direction if wrong |
|---|---|---|
| P5.1 | The answer is **"yes, but not for the classes named"**: a general lowering IS derivable for a *straight-line-with-one-diamond* class, and is **NOT** derivable for loops without also porting the loop-strength/induction machinery. | — |
| P5.2 | The first class to attempt is **the counted `for` loop over one array with one accumulator** (the `?shuffle2` shape). | — |
| P5.3 | Predicted reach of the first class over the 124-TU reach-pool: **≤ 6 TUs**. Registered pessimistically on purpose — #770's streak is optimistic, and every prior "class" estimate on this project has been a per-TU transcription in disguise. | pessimistic |
| P5.4 | The binding constraint on class conversion is **not** register allocation and **not** ordering: it is that the port's IL reader refuses the constructs before any emitter question is reachable (the 48 reader refusals of WB-A). Registered so that a "the policy is readable!" result cannot be narrated as a conversion route. | — |

---

## Scoring rule

Every row above is scored HIT / MISS / UNSCOREABLE in
`WB_REGALLOC_FINDINGS.md` §8, with `UNSCOREABLE` reserved for rows whose
premise did not occur (the wb-frame C7 `assumption unmet` rule). A MISS in the
registered direction is reported as such; a MISS with no registered direction
is reported as a plain miss. No row is rewritten after the first probe.
