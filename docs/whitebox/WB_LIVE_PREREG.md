# WB-LIVE `wb-live` — PREREG

> **PROVENANCE — DISASSEMBLY-DERIVED.** See
> [`C2_MAP_METHOD.md`](C2_MAP_METHOD.md) §0 for the exact bytes and
> [`DISCLOSURE.md`](DISCLOSURE.md) for what adoption costs. This lane expects to
> adopt nothing into `crates/` and is expected to be docs-only.

Registered **before the first grep of `~/ghidra-projects/export/c2/`** and
before the first `cl.exe` this lane authored, per board #770's standing rule.
Scored in [`WB_LIVE_FINDINGS.md`](WB_LIVE_FINDINGS.md) §8.

Lane: `wb-live` / branch `wt-wb-live`, branched at master `31a83377`.

**What was read before freezing** (in-repo only, no export, no disassembly):
`CLAUDE.md`, `docs/STATUS.md`, `docs/CFG_SHAPE.md` §6, `C2_MAP_METHOD.md`,
`WB_REGALLOC_FINDINGS.md` (all of it), `WB_REGALLOC_PREREG.md`, `BOARD.md` rows
#1820–#1830, `docs/rungs/_TEMPLATE.md`. Nothing in
`~/ghidra-projects/export/c2/` has been opened by this lane.

**Image sha256 to be verified at the top of the lane** —
`c80981c015166effecc71ad8112d5577a065b2300891dfdb02b9c13787a66258`.

---

## The question, and why it is not the one `wb-regalloc` answered

`wb-regalloc` read the **selector** (#1821): minimum cost over the
*interference-allowed candidates*. It did not read what makes a candidate
interference-allowed. `docs/CFG_SHAPE.md` §6.2 item **F** — values live across
block boundaries — is the one item of the new IR that has no characterized
mechanism behind it, and §6.2 says so by name. This lane reads the liveness /
interference construction that produces the candidate set.

---

## P0 — the success floor and the DECLINE FLOOR

| # | prediction | p | direction if wrong |
|---|---|---:|---|
| P0.1 | **Floor.** This lane names the interference representation and its granularity with an address, and at least one claim about it survives a frozen obj-check on ≥1 function outside every shipped port class. | 0.75 | optimistic |
| P0.2 | **DECLINE FLOOR.** If fewer than **3 of the 5 mission questions** get an answer that is *both* address-backed *and* carries at least one obj cell that could have gone red, this lane reports the reading as **UNSETTLED** and publishes no rule — the findings doc says "not characterized" and item F stays blocked. | — | — |
| P0.3 | **DECLINE FLOOR, second arm.** On the black-box grid, if the liveness model and the **INCUMBENT** (below) are not separated by ≥3 cells that actually discriminate, the grid is declared insufficient in the findings and no rule is published off it. | — | — |
| P0.4 | This lane ships **no `crates/` change** and adds **no `DISCLOSURE.md` row**. | 0.85 | — |

### The INCUMBENT, which is the control

The port's shipped model, as `CFG_SHAPE.md` §6.2 F and `CODEGEN_W6_COMPARE.md`
§6 state it:

> **I0 — positional and local.** Formals occupy `ARG_REGS` by declaration
> order; the result is `r3`; temps descend from `r11` **in emission order**,
> one register per temp, with no notion of a value's live range ending.

Every register claim below is graded against **I0**, not against a bare
threshold. I0 is falsifiable on this grid by construction: it must predict
`r11, r10, r9` for three temps whose live ranges do **not** overlap, where a
liveness model predicts `r11, r11, r11`.

---

## P1 — the interference REPRESENTATION (mission question 1)

| # | prediction | p | direction if wrong |
|---|---|---:|---|
| P1.1 | The interference relation is stored as a **bit vector / bitset per node** (a candidate's neighbour set), not as an edge list and not as a triangular bit matrix alone. | 0.70 | — |
| P1.2 | There is **also** a per-node **degree count** kept alongside the bitset, because #1821's `100 × degree` term needs a degree without a popcount. | 0.80 | — |
| P1.3 | The node granularity is a **live range / web** that c2 synthesizes — one node may cover several defs and uses of the same symbol, and one symbol may produce several nodes. It is **not** one node per IL token and **not** one node per `.sy` local. | 0.65 | optimistic |
| P1.4 | The bitsets are **word-array bitsets sized per function** (allocated from an arena at function entry), not a fixed-size image-resident array like the `0x594` cost array. | 0.85 | — |
| P1.5 | **Machine registers are nodes in the same graph** — precoloured — which is how an argument in `r3` and a call's volatile clobber both reach the candidate set through one mechanism. | 0.75 | optimistic |

## P2 — how liveness is COMPUTED (mission question 2)

| # | prediction | p | direction if wrong |
|---|---|---:|---|
| P2.1 | Liveness is a **dataflow fixpoint over basic blocks**, with per-block `live-in`/`live-out` bitsets — **not** a linear scan over a linearized instruction order. | 0.80 | optimistic |
| P2.2 | The transfer is the textbook backward one: `live_in = use ∪ (live_out ∖ def)`, `live_out = ∪ live_in(succ)`, with `use`/`def` (a.k.a. `gen`/`kill`) precomputed per block in one forward pass over the block's tuple list. | 0.70 | — |
| P2.3 | The iteration is a **worklist** (a queue/stack of blocks to revisit) rather than a round-robin "iterate all blocks until nothing changes". | 0.45 | pessimistic — registered *against* my own instinct, because a 1990s UTC pass is as likely to be a simple repeat-until-stable loop |
| P2.4 | The iteration order is **reverse of the block-construction order** (an approximation of reverse postorder), i.e. the same linear block array `wb-regalloc` §4 found, walked backwards. | 0.55 | — |
| P2.5 | Interference edges are added by a **second walk** over each block, backwards, maintaining a running live set from `live_out` — an edge from each def to every member of the current live set. Liveness and interference are two passes, not one. | 0.75 | — |
| P2.6 | The whole construction lives inside `color.c` (`0x10b2c21d`…`0x10b30517`), **not** in `fg.c` and not in a separate `live.c`; `c2_tus.tsv` has **no** TU whose name contains `live`. | 0.60 | — |
| P2.7 | The allocator is **iterative with a spill/rebuild outer loop** (build → simplify/colour → on failure insert spills → rebuild), which is what makes `color.c` 7 100 lines rather than 700. | 0.70 | optimistic |

## P3 — what a VALUE is (mission question 3)

| # | prediction | p | direction if wrong |
|---|---|---:|---|
| P3.1 | The allocator's node is keyed on a **symbol-table entry** (the same `sym` pointer `wb-regalloc` §7.5 saw assigned literally for `cr6`), not on a tuple/instruction index. A register is chosen *for a symbol*, and the instruction operand points at that symbol. | 0.70 | — |
| P3.2 | Those symbols are a **superset** of the `.sy` locals: c2 mints symbols for expression temporaries the IL never named, and those temporaries are the majority of the nodes on a straight-line body. | 0.85 | — |
| P3.3 | Therefore the port-side IR must be able to name **its own temporaries as first-class values with a live range**, not merely reproduce the IL's tokens — i.e. an IL-token-keyed IR is *insufficient* for item F. | 0.80 | — |
| P3.4 | A single `.sy` local can be split into more than one node (live-range splitting is present in the image), but is **not** split at `/O1` on any body this lane grades. | 0.50 | — |

## P4 — what feeds the COST (mission question 4)

| # | prediction | p | direction if wrong |
|---|---|---:|---|
| P4.1 | The interference "weight" #1821 adds per already-coloured neighbour is **not the constant 1** — it is a per-neighbour number read out of the neighbour's node. | 0.65 | optimistic |
| P4.2 | At least one term is an **execution-frequency / loop-depth estimate** (a block weight, likely `10^depth` or a shift by depth), so a value used in a loop outbids one used once. | 0.70 | optimistic |
| P4.3 | The **copy-preference** list (`param_1 + 0x38`) is populated from **register-to-register move tuples**, including the ABI's argument moves and the return move to `r3` — this is the single mechanism behind #1821's "an argument stays in the argument register". | 0.85 | — |
| P4.4 | The calling convention enters as **precoloured nodes plus interference**, not as a separate constraint list: the call tuple defines every volatile, so a range live across a call interferes with `r3`…`r12` and is excluded from those candidates outright rather than merely penalised. | 0.80 | — |
| P4.5 | There is a **spill-cost** field per node (uses/defs weighted by frequency, divided by degree in the Chaitin tradition) used by the *spill chooser*, and it is a **different** number from the selector's cost array. | 0.60 | — |

## P5 — callee-saved, and §6.2 item F's two measured cases (mission question 5)

| # | prediction | p | direction if wrong |
|---|---|---:|---|
| P5.1 | **The `r31`/`r30` case is entirely explained by P4.4 and the list order.** A formal live across a call interferes with all volatiles, so the candidate set is exactly the callee-saved tail of `0x10c37de0`, whose head is `r31`, then `r30`. **No preference for callee-saved registers is needed and none exists** — it falls out of "volatiles are excluded" plus "ties go to the earliest surviving entry". | 0.85 | — |
| P5.2 | **The framing is a CONSEQUENCE, not a cause.** The body is framed *because* the allocator took `r31`/`r30` and they must be saved, not the other way round; the prologue builder (`0x10bff507`'s flag scan, wb-frame) reads the used-register set **after** colouring. So a port that decides framing before allocation has the dependency backwards. | 0.80 | — |
| P5.3 | **The `MemFree` r4→r11 case needs no new mechanism either.** `v2`'s range spans the point where `r4` is redefined, so it interferes with the precoloured `r4` node, `r4` leaves the candidate set, and the head of the list `r11` wins at cost 0. The **copy** exists because the value arrives precoloured in `r4` and its range cannot stay there. | 0.70 | — |
| P5.4 | The copy is emitted by the **lowering/ABI code**, not by the allocator: there is no coalescing pass that *inserts* moves. c2's coalescer only *removes* them (or declines to), via the preference list. | 0.55 | — |
| P5.5 | Nothing in the mechanism is specific to **entry blocks**. If P5.3 is right, the same rule places the copy wherever the interference forces it, and §6.2's "in the entry block" is a property of that body, not of the rule. | 0.70 | — |

## P6 — the obj grid (frozen separately, before the first `cl.exe`)

| # | prediction | p | direction if wrong |
|---|---|---:|---|
| P6.1 | **The sharpest discriminator against I0 is REUSE.** Three temporaries whose live ranges do not overlap take **`r11` three times**; I0 predicts `r11, r10, r9`. | 0.85 | — |
| P6.2 | A value live across a call takes **`r31`**; I0 predicts `r11`. | 0.90 | — |
| P6.3 | Live-range **overlap count**, not temp count, sets the high-water mark: a body with 6 temps of which at most 2 overlap uses **2** registers, not 6. | 0.80 | — |
| P6.4 | On ≥3 of the grid's cells I0 is **refuted** — i.e. I0 is a control that goes red. If it does not go red anywhere, the grid is declared insufficient per P0.3 and no rule is published. | 0.85 | optimistic |
| P6.5 | The grid finds **at least one fact the disassembly reading did not predict**, and it is reported as a miss rather than absorbed. #770's streak plus the method doc §7 makes this the modal outcome. | 0.70 | — |

## P7 — the judgment: is item F buildable after this lane?

| # | prediction | p | direction if wrong |
|---|---|---:|---|
| P7.1 | **Yes for the mechanism, no for the schedule.** After this lane, item F is specifiable — a port can state what it must compute — but building it still requires items A–E (blocks, terminators, labels, fixups, condition codes), which do not exist, so no TU converts and the workload reach of this lane is **exactly 0**. | 0.80 | — |
| P7.2 | The port-side IR requirement this lane lands on is **"every value needs a live interval over an ordered instruction list, and machine registers must be nameable as values"** — strictly more than "blocks with terminators". | 0.75 | — |
| P7.3 | The **binding constraint remains the IL reader**, not the allocator — the same P5.4 that scored a hit for `wb-regalloc`. A liveness model converts nothing until the reader accepts the constructs. | 0.85 | — |
| P7.4 | This lane does **not** need to read the spiller to specify item F: at `/O1` on the frontier's shapes nothing spills (`wb-regalloc` P2.7 hit), so the spiller is out of scope and is declared so rather than skipped silently. | 0.65 | — |

---

## Scoring rule

`H` hit · `M` miss · `U` unscoreable (premise did not occur). A prediction
registered at `p ≤ 0.5` that misses is **not** a free pass — it is scored as a
miss and counted in the calibration line, per `wb-regalloc` §8. The calibration
line reports hits split by registered `p` band so that a lane that hedges
everything at 0.5 is visible as such.
