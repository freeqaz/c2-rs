# c2-rs documentation — start here

**This page routes; it does not restate.** Every row below says what question a
document answers and how to read it, so you can decide whether to spend the
hop. If a row does not tell you that, it is a defect, not brevity.

**Read the freshness class before you quote anything.** This tree keeps its
history on purpose — the failed attempt, the estimate that missed 5:1, the rule
refuted by the next cell. That makes a lot of it *evidence about a day*, not
advice about today, and the two look identical if nobody labels them.

| class | how to read it |
|---|---|
| **live** | maintained. If it is wrong, fix it in place |
| **generated** | produced by a script. **Never hand-edit** — regenerate |
| **dated record** | a measurement or decision made on a day, kept as written. Its numbers are true *of its tree*, not of yours |
| **superseded** | a dated record something later overturned, kept under a banner. The banner is the live part |

The conventions themselves — strike-in-place amendment, the prereg/findings
pairing, the citation chain, the naming patterns — are
[`DOC_CONVENTIONS.md`](DOC_CONVENTIONS.md) *(live)*.

---

## Start here

| If you are asking… | Go to |
|---|---|
| **where is this project right now?** | [`STATUS.md`](STATUS.md) *(live + generated block)* — the one-page answer: the headline metrics, what each one is *for*, the traps that make them individually true and jointly misleading, and the command that reproduces each. **Read it before quoting a number from anywhere else in this directory** |
| **what is this project for?** | [`GOAL_DECISION_2026-08-21.md`](GOAL_DECISION_2026-08-21.md) *(dated record — the owner's decision, still the authority)*. **Read its § "AMENDED", not just its opening**: the owner returned the same day and ranked the two goals. Its own line 18 still reads "ranked equally" and is superseded by a section 18 lines below it |
| **what was decided most recently?** | [`DECISIONS_2026-08-22.md`](DECISIONS_2026-08-22.md) *(dated record)* — the owner's four decisions of 2026-08-22, quoted verbatim: the reads are funded, the shipping roadmap is rewritten against the goals, push is authorized, and a docs-structure pass. Each one also states **what it does not decide** |
| **what work is funded right now?** | [`whitebox/READ_PLAN_2026-08-21.md`](whitebox/READ_PLAN_2026-08-21.md) *(live)* §3–§4 — the nine ranked reads R1–R9, priced in days against the black-box lanes they displace. **R1→R3 are funded** as of 2026-08-22; the 4a/Phase-0 branch choice is deliberately still open |
| **how is correctness judged?** | `../CLAUDE.md` § "The one correctness rule" — the real `c2` under wibo plus a byte-exact obj compare, and nothing else. Then [`PROGRESS_METRIC.md`](PROGRESS_METRIC.md) and [`FUNCTION_BYTE_MATCH.md`](FUNCTION_BYTE_MATCH.md) for the two continuous instruments that live *beside* the judge and never stand in for it |
| **how far is it from done, in arithmetic?** | [`CEILING.md`](CEILING.md) *(live)* — the distance between the process and TU match 871, regenerated from the instruments rather than quoted from history, and published as arithmetic even where it reads badly |
| **what is blocking, and what would each rung unlock?** | [`GAPS.md`](GAPS.md) *(live)* — the measured gap ledger: what each blocker holds hostage in the real 878-TU workload, per-rung acceptance gates, and the exact verification commands |
| **how is work dispatched and recorded?** | [`rungs/README.md`](rungs/README.md) *(live)* — the three lane kinds, the one-word `Outcome:`, the rules a probe must satisfy, and the standing facts every brief must carry. Then [`BOARD.md`](BOARD.md) *(live)* for the numbered items `ROADMAP.md` cites everywhere and lists nowhere else |
| **what does `c2.dll` actually do?** | [`whitebox/README.md`](whitebox/README.md) *(live)* — the index over the binary record: the map, the address reference, the disclosure ledger, and eighteen findings documents, most with the prereg they were scored against |
| **I am about to budget a probe grid** | [`WHITEBOX_LEVERAGE_2026-08-21.md`](WHITEBOX_LEVERAGE_2026-08-21.md) *(live doctrine, one section struck)* — **read before probe**: price the whitebox read that answers the same question and prefer it. Its §5(c) carries a dated ⚠ correction, board #3369 |
| **how do I write or move a doc here?** | [`DOC_CONVENTIONS.md`](DOC_CONVENTIONS.md) *(live)* — and the short version is: **do not move one.** Citations are load-bearing in five places and `board_audit.sh` cannot see them break |

## The plan of record, and the history behind it

| Document | What it answers | Class |
|---|---|---|
| [`ROADMAP.md`](ROADMAP.md) | the path from the MVP port to a full stack: the gaps G1–G6, the widening ladder W5–W14, the front-end track P-F0.2→P-F2, the census tool that drives ordering, the composition milestone. **8,000+ lines and largely session history** — it contains many superseded snapshots, so orient off `STATUS.md` and come here for the ordering | live, but read as an archive |
| [`CEILING.md`](CEILING.md) | the arithmetic to TU match 871, in one place with denominators. §6.1 enumerates the seven phases; §5 is the 5:1 optimism calibration every forward estimate is multiplied by; §11.4 is the check three consecutive conversion lanes wish they had run first | live |
| [`ROADMAP_SLICING_2026-08-21.md`](ROADMAP_SLICING_2026-08-21.md) | the owner asked how to chop row 4a into deliverables shorter than 45 months. Four research lenses answered; every ✅ figure was recomputed by the coordinator from a fresh 878-TU scan rather than quoted | dated record |
| [`SHIPPING_ROADMAP_2026-08-22.md`](SHIPPING_ROADMAP_2026-08-22.md) | **the roadmap of record.** What this program is, what is funded, what is open — written on the owner's order (`DECISIONS_2026-08-22.md` decision 2) and aligned with the goal statement, so it reads the project as what its predecessor's own escape clause said it would be: a **compiler-backend reconstruction program**. Milestones are **evidence-gated, not calendar-gated** (no dates anywhere — `CEILING.md` §5). Carries the three meanings of 100 %, the reads-first funding, the still-open branch choice, the operating model, and the vendor-DLL service as an explicitly **subordinate** option with its two disqualifications. Every judgement the owner has not made is posed as a proposal with alternatives | live |
| [`SHIPPING_ROADMAP_2026-08-19.md`](SHIPPING_ROADMAP_2026-08-19.md) | a route to a shippable product and then native parity: three meanings of 100 %, the options ranked by cost and return, phase exit gates. **Its headline tripped its own §1 escape clause** when the owner stated goal (2), and `DECISIONS_2026-08-22.md` decision 2 ordered it rewritten — **superseded by the row above**. What survives on its merits and is carried forward: §4's three meanings of 100 %, §6's operating-model items 3–6, and every measurement | superseded |
| [`PARKED_LANES.md`](PARKED_LANES.md) | branches that are built, reviewed, and deliberately **not** merged, so a later session finds them instead of re-doing the work or re-minting their board numbers. A parked branch is not a failed one — each of the three produced shippable machinery *and* found a defect in its own grading | live |
| [`rungs/INDEX.md`](rungs/INDEX.md) | one row per rung: date, tag, slug, fixture count, census delta | **generated** — `scripts/gen_rung_index.sh` |
| [`rungs/`](rungs/README.md) | 389 lane write-ups, one file per rung, plus the `_`-prefixed preregs. The filename is the claim | dated records |

## How the port is measured — and the wall between measuring and judging

The judge is the compiler. Everything in this section is an instrument beside
it, and the separation is load-bearing: a wrong emit scores strictly **below**
the refusal it replaced, and no continuous score may license one.

| Document | What it answers | Class |
|---|---|---|
| [`PROGRESS_METRIC.md`](PROGRESS_METRIC.md) | the **progress mass** — the first continuous instrument, adopted after a day that moved factor C by 55 TUs, closed three wrong-emit families and left the headline metric reading 8/878 before and after. Defines the wall between it and correctness *first*, because that is the load-bearing part | live |
| [`FUNCTION_BYTE_MATCH.md`](FUNCTION_BYTE_MATCH.md) | **FBM** — the judge's own question asked per function. Also records the premise of `PROGRESS_METRIC.md` that this lane *refuted*, and the defect FBM's own known-answer control found on its first corpus run | live |
| [`DIFF_STRUCTURE.md`](DIFF_STRUCTURE.md) | what is *inside* a `fnbyte-differs` body. The answer is one mechanism, not a near-miss: 0 pure reorderings, 99.7 % of substituted words differ in **opcode**, 94.3 % of bodies wrong at word 0 — c2 inlined a callee where the port emitted a call. **Its numbers are from tree `0c8a185` and want a rescan, not an edit** (`DECISIONS_2026-08-22.md`, last section) | dated record, §3.2 refuted in place |
| [`CROSS_PRODUCT.md`](CROSS_PRODUCT.md) | why a merge of two independently-green branches is a **new corpus** — the label counter is per-TU and was being read from a per-function method, and neither branch's corpus could contain the case. The four tiers `scripts/cross_sweep.sh` grades, and, stated explicitly, what it leaves ungraded | live |
| [`CORPUS_MVP.md`](CORPUS_MVP.md) | the P1.2 `(source, IL, obj)` triple corpus: generator, manifest schema, deterministic source generation, and why the generated corpus is gitignored while a synthetic sample is committed | live |
| [`EDIT_MODEL_MVP.md`](EDIT_MODEL_MVP.md) | the K3a length-consistent `.ex` edit primitive — what turns a lossless *read* codec into a verified *edit* substrate, and the differential gate that proves each edit byte-exact **as an edit** | live |
| [`plan/CONTROL_TUS.txt`](plan/CONTROL_TUS.txt) | the ObjPlan manifest lane's control, **pinned by name and not by count**, with the file's own header explaining why a count-pinned control passes in the wrong world. `include_str!`-ed into `crates/c2-harness/src/gap/plan.rs`, so it is data, not documentation | live |
| [`perf/perf_scale.csv`](perf/perf_scale.csv), `perf/perf_scale.png` | throughput vs concurrency, port against real c2 under wibo. **Reported, never gated** — throughput is a property of the port, not a reason to fund a lane | generated — `c2rs perf-scale --csv`, plotted by `scripts/plot_perf.py` |

## The input side — the c1xx→c2 IL bundle

The `_CL_*` five-file bundle is what c2 consumes. These pages are how the port
reads it; every one of them fails closed on what it has not characterized.

| Document | What it answers | Class |
|---|---|---|
| [`IL_BUNDLE_MVP.md`](IL_BUNDLE_MVP.md) | the capture recipe (`/Bd /d2nop`), the five suffixes, the per-file parse, and the surprisingly short list of bundle facts the emitter actually consumes | live |
| [`IL_STMT_GRAMMAR.md`](IL_STMT_GRAMMAR.md) | the `.ex` statement grammar, every claim byte-cited and marked `[CF]` / `[DIR]` / `[P]` by evidence kind | live |
| [`IL_STMT_LAYER.md`](IL_STMT_LAYER.md) | an **independent re-derivation** of the statement layer from a fresh probe set: what `0x53` is, the ternary production, and the proof that statement boundaries cannot be found by byte-scan. Every claim of `IL_STMT_GRAMMAR.md` it touched was confirmed, none contradicted | live |
| [`IL_EXPR_LAYER.md`](IL_EXPR_LAYER.md) | the operand stream: designators, loads, and what still refuses | live |
| [`IL_CALL_GRAMMAR.md`](IL_CALL_GRAMMAR.md) | the `.ex` CALL token and body-statement grammar — the #1 measured blocker of the real workload when it was written | live |
| [`IL_CALL_IN_EXPR.md`](IL_CALL_IN_EXPR.md) | the `expr-call-in-expr` bucket decomposed into 23 named sub-buckets over all 878 TUs. **Read §14.2 first if you are picking a rung**; §14 supersedes §2's estimated shares (two of them wrong by 6× and 400×) and §11's ranking | live, §§2/11 superseded in place |
| [`IL_CAST_CONVERT.md`](IL_CAST_CONVERT.md) | `.ex` casts and conversions — `0x2C`, and what `0x40` actually is | live |
| [`IL_INTRINSIC_CALL.md`](IL_INTRINSIC_CALL.md) | opcode `0x40`, the INTRINSIC CALL. Decode only — **nothing was lowered**, and the measured in-class count is unchanged to the function | live |
| [`IL_TYPE_TAGS.md`](IL_TYPE_TAGS.md) | the `.ex` TYPE encoding and what each scalar type costs in codegen — written because `expr-load-type-*` is a *family* of buckets and it was not clear how many were one mechanism | live |
| [`IL_LOAD_TYPES.md`](IL_LOAD_TYPES.md) | the full type-word field grammar including aggregates, what each bucket name means, the measured PPC lowering per type, and a ranked order of work with its estimation basis stated | live |
| [`IL_TYPE_WIDE_TAG.md`](IL_TYPE_WIDE_TAG.md) | the type width that was two bytes short and **invented 376 blocker rows**. Census delta 0 — a measurement, not a rung: decode reach 94.2 % → 97.2 %, undecoded distinct keys 384 → 8 | dated record |
| [`IL_DECODE_REACH.md`](IL_DECODE_REACH.md) | the two opcodes that were holding the statement-layer decode reach. Census delta 0: bodies decoded end to end 86.5 % → 94.2 %, `eh-unknown` down 52.4 %. **It admits nothing and lowers nothing** | dated record |
| [`IL_STORE_LEAF.md`](IL_STORE_LEAF.md) | the store leaf and the one-byte-unsigned value class (W25 + W26) — and the lane where the ranking suggested by row sizes was **not** the ranking the counterfactual produced | live |
| [`IL_SY_LOCALS.md`](IL_SY_LOCALS.md) | whether `.sy` names a function's locals — the positive signal `assign-dst-not-formal` needs and `.ex` does not carry. **It does**: token, name, scope depth, type, size, flags, with a decidable binding to the `.ex` body | live |
| [`OPT_MODE.md`](OPT_MODE.md) | the per-function optimization word, and which mode the port targets. The headline when written: the port's byte-exactness was a claim about `/Ox` and **the entire real workload is `/O1`**. Both are supported now, and the mode is read rather than assumed | live |

## The codegen side — IL → PPC

| Document | What it answers | Class |
|---|---|---|
| [`CODEGEN_PPC_MVP.md`](CODEGEN_PPC_MVP.md) | instruction encoding, the X360 integer ABI, the COLOR allocator's observed scratch order, the frame model the port implements (sizing, callee-saved slots, the two save-helper thresholds, the stack-probe ladder), and the **non-commutative hazard list — what NOT to generalize** | live |
| [`CODEGEN_FRAMED_CALLS.md`](CODEGEN_FRAMED_CALLS.md) | the byte-level encyclopedia for multi-call framed bodies: the frame-size rule and its 480-case refutation sweep, the five prologue/epilogue classes, `__savegprlr_` / `__savefpr_`, argument marshalling, and three symbol-order rules `OBJ_GY_SHAPES.md` §3.3 did not have | live |
| [`CODEGEN_ARG_PERM.md`](CODEGEN_ARG_PERM.md) | argument marshalling over the **complete** permutation grid at each arity — 304 objects, so a candidate model is scored on every cell instead of the three that were lying around | live |
| [`CODEGEN_FP_ARGS.md`](CODEGEN_FP_ARGS.md) | the two register numberings that run over one parameter list, neither of which is the formal's index, disagreeing in **opposite directions**. Four of the project's twelve live wrong-bytes emits came out of this one fact | live |
| [`ABI_EDGES.md`](ABI_EDGES.md) | what the ABI does for the types the MVP line does not cover: `long long`, FP and varargs, structs by value (register-chunked, no by-reference threshold up to 64 bytes), struct returns | live |
| [`CODEGEN_W5_SCRATCH.md`](CODEGEN_W5_SCRATCH.md) | expression trees deeper than a serial accumulator chain — why the canonical `(a+b)*(c+d)` was `NotImplemented`. Characterization only | live |
| [`CODEGEN_W6_COMPARE.md`](CODEGEN_W6_COMPARE.md) | the `.ex` vocabulary for the six relational operators and the branchless boolean-materialization idioms c2 lowers them to, every word **re-encoded from its fields and compared against the observed word** | live |
| [`CODEGEN_W6_O1.md`](CODEGEN_W6_O1.md) | the same matrix at `/O1` — the complete comparison-leaf byte table, so the port can be re-targeted from it without re-capturing | live |
| [`CODEGEN_W13_FLOAT.md`](CODEGEN_W13_FLOAT.md) | float/double codegen and the ported boundary, against the two operand-type buckets worth 6.5 % of blocked functions together | live |
| [`CFG_SHAPE.md`](CFG_SHAPE.md) | the control-flow step, byte-level: how `.ex` encodes control flow, what c2 emits per shape, the minimal instance to build first, and what a block/instruction IR must carry to serve it. The emission half (§3–§4) is new — nothing else in `docs/` states it | live |
| [`VMX128_DECODE.md`](VMX128_DECODE.md) | the VMX128 decode specification and the list of ways stock tooling lies about it. **Nothing here is a gate**, and nothing is derived from disassembling c2 — so no `DISCLOSURE.md` row is implied | live |

## The obj side — what c2 writes

| Document | What it answers | Class |
|---|---|---|
| [`OBJ_FORMAT_MVP.md`](OBJ_FORMAT_MVP.md) | the COFF byte map for the MVP class: header, section headers, `.drectve` / `.debug$S` / `.XBLD$W`, symbol and string tables, the COMDAT checksum algorithm, and every field classified **CONST** or **DERIVED** | live |
| [`OBJ_GY_SHAPES.md`](OBJ_GY_SHAPES.md) | the three COMDAT shapes that refuse under `/Gy`: `_fltused` placement, pooled `.rdata` FP constants, and the framed non-leaf call | live |
| [`OBJ_DATA_BSS_SHAPE.md`](OBJ_DATA_BSS_SHAPE.md) | `.data` and `.bss` — the section-shape specification behind the greedy ladder's single largest step, worth +402 TUs. Every rule names the cells it was fitted on **and the cells that refute it** | live |
| [`OBJ_DYNINIT_SHAPE.md`](OBJ_DYNINIT_SHAPE.md) | the `??__E` dynamic-initializer obj: the sections a namespace-scope object with a non-trivial constructor adds, and nothing the other two obj docs already cover | live |
| [`OBJ_RDATA_R_SHAPE.md`](OBJ_RDATA_R_SHAPE.md) | `.rdata$r`, the RTTI record graph. **Specification only, three times over**: three lanes were briefed to add it and all three re-derived the price and declined. §8.2 corrects §8.1's stated cause, and §9.1 is the guard that makes the decision enforceable rather than a matter of lane discipline | live, spec-only by decision |
| [`EH_RECORDS.md`](EH_RECORDS.md) | what `/EHsc` adds to an obj — read-only sizing so the EH rung can be ordered honestly against the others. **Nothing here should be implemented from this document alone** | live |
| [`EH_CRITICAL_PATH.md`](EH_CRITICAL_PATH.md) | `.rdata$r` is **RTTI, not EH** — a pure function of `/GR` that survives removing `/EHsc` — so rung three of the section ladder is not Phase 5, and the EH records land in plain `.rdata` | dated record |
| [`SYMBOL.md`](SYMBOL.md) | a store run through more than one base symbol: what the multi-symbol regime *is*, after `w-parse` proved the axis is the reference BIND and not the machine register | live |

## The hard axes — where rules go to be refuted

These carry the project's highest rule-mortality. Read the refutation tables
before proposing a rule; most of them have already been proposed and killed.

| Document | What it answers | Class |
|---|---|---|
| [`STORE_SCHEDULE.md`](STORE_SCHEDULE.md) | where c2 puts a value-producing instruction, and why the stores move. **The project's single most-refuted axis** — twelve candidate rules from four lanes, each killed by the next cell, tabulated with its killer | live |
| [`ALLOC.md`](ALLOC.md) | which register c2 gives each producer of a store run. All four previously refuted rules turn out to be derived consequences of one rule | live |
| [`ORDER.md`](ORDER.md) | the store order when the head slots are contested — the structured-but-not-single-family residual `ALLOC.md` §6 left open as board #544 | live |
| [`LABEL_COUNTER.md`](LABEL_COUNTER.md) | the compiler-label counter, measured seed-free. **Read the ⚠ banner first**: four consecutive lanes reported this document's surcharges as wrong, every one of their numbers is a real reading, and none of them is the charge this document tabulates — they measured `Δseed + Δcharge`, and the seed is a function of the source text because c1xx and c2 share one symbol-id space | live, with a standing banner |
| [`INLINE_PREDICATE.md`](INLINE_PREDICATE.md) | when c2 does not emit the call the IL contains — the mechanism `DIFF_STRUCTURE.md` says dominates the wrong-body population. Its status block records that mechanism E's seed set changed and §1.2's cycle stop had to be re-derived rather than inherited | live, amended in place |
| [`CMP_PRODUCES_A_VALUE.md`](CMP_PRODUCES_A_VALUE.md) | the comparison that produces a value — **DECLINED on measurement**. Built, graded byte-exact over 552 generated cases in four mode lanes, measured at **+0 census functions**, reverted. The four byte-level readings survive for the next rung into the family | dated record |
| [`OPERATOR_GRANTS.md`](OPERATOR_GRANTS.md) | the relational family is bare, and it is the first operator worth ranking. Ships no acceptance change: it replaces an *inference* with a capture, and the inference was wrong | dated record |

## The binary record — `docs/whitebox/`

Whitebox analysis is **authorized, encouraged, and not a legal risk** (owner,
2026-08-17 — `../CLAUDE.md`). Under the 2026-08-21 goal ranking it is
**product**, not overhead: goal (1) is a clear understanding of MSVC's
internals, and that directory is where the understanding is kept.

Go to [`whitebox/README.md`](whitebox/README.md) — it indexes the navigational
map (`C2_MAP.md`), the reproduction method and pinned image sha256
(`C2_MAP_METHOD.md`), the address-indexed reference (`ref/`), the
engineering-provenance ledger (`DISCLOSURE.md`), and eighteen findings
documents — most paired with the prereg frozen before the lane looked —
covering the scheduler, the allocator, instruction selection, the inliner, the
label counter, EH, the `.ex` reader and the opaque middle's two edges.

Two pages at *this* level govern how that record is used:

| Document | What it answers | Class |
|---|---|---|
| [`WHITEBOX_LEVERAGE_2026-08-21.md`](WHITEBOX_LEVERAGE_2026-08-21.md) | **read before probe**: price the read that answers the question before budgeting a probe grid. Also the design rule for general layers (expose the decision surface, do not bake the constant) and why the gate stays **binary** while gradients live beside it. §5(c) carries a dated ⚠ correction — it proposed an instrument that had shipped sixteen days earlier, and that instrument's own output refutes the table §5(c) proposed | live, §5(c) struck |
| [`whitebox/READ_PLAN_2026-08-21.md`](whitebox/READ_PLAN_2026-08-21.md) | the enumerated target list the doctrine needs: an inventory **with denominators**, an index of every fitted constant in `crates/` against the read that would replace it, and nine ranked reads priced in days. A probe-grid lane on any of the nine must first say why it is not the read | live |

## Reviews, proposals and pricings — dated records, all of them

**None of these is live advice.** Each was written against a named tree,
several have been overturned in part by their own successors, and the banners
are the part to read first. They are here because the reasoning is worth more
than the conclusion — and because a lane that re-derives one of them and
reaches the same answer has spent a lane learning what was already written
down.

| Document | What it established, on the day | Class |
|---|---|---|
| [`STRATEGY_REVIEW_2026-08-13.md`](STRATEGY_REVIEW_2026-08-13.md) | five named hypotheses tested against the tree with intent to refute; which survived, which did not, every load-bearing claim at file:line, and both sides quoted where two sources disagreed. **The review that recorded the goal question as "owned by nobody"** — since answered | dated record |
| [`ARCH_REVIEW_2026-08-21.md`](ARCH_REVIEW_2026-08-21.md) | the seven-lens architecture review that found the goal question was the hinge step 5's justification turned on. **Its finding 1 was overturned in both directions by the lane it dispatched**: COLOR *is* gradeable against real c2, and 85.6 % of its visible footprint was invisible to every prior measurement — including this review's | dated record, amended in place |
| [`ARCHITECTURE_PROPOSAL_2026-08-20.md`](ARCHITECTURE_PROPOSAL_2026-08-20.md) | the staged pipeline, and how to get there without losing the 26. §8 decision 0 is the branch point the 2026-08-22 funding sits *inside* — and that branch choice is **still open** | dated record, amended |
| [`REFACTOR_REVIEW_2026-08-20.md`](REFACTOR_REVIEW_2026-08-20.md) | the second architecture review, scoped deliberately to what the proposal did not examine: harness factoring, test architecture, `scripts/`, code quality, crate boundaries — warranty vs mass | dated record |
| [`STEP5_PRICING_2026-08-21.md`](STEP5_PRICING_2026-08-21.md) | step 5 re-priced per stage, with predictions frozen before any probe ran. **Every forward cost figure is a lower bound** and `CEILING.md` §5's ×5 optimism calibration is applied, not merely cited | dated record |
| [`PRIOR_ART.md`](PRIOR_ART.md) | what exists outside this repo and what it is worth. Almost nothing reduces scope; one thing reduces cost by ~an order of magnitude and nobody had considered it; and static recompilation of `c2.dll` is a **category error**, not merely expensive | dated record |
| [`PHASE6_RANKING.md`](PHASE6_RANKING.md) | "17 of 19 block on control flow" is **true and converts nothing**. Measurement only, prereg committed before the first measurement | dated record |
| [`PHASE7_PLAN.md`](PHASE7_PLAN.md) | the emit set, the obj shape, and the route from 6 to 871. A **plan** — nothing in it is built, and every number is labelled measured / ceiling / estimate with its predicate named | dated record |
| [`PHASE7_VALIDATION.md`](PHASE7_VALIDATION.md) | the out-of-sample gate for the fitted emit predicate (#161). It is the clearest worked application of `ROADMAP.md` §9.18.8 — *absence reads as success unless forbidden* — so its own header forbids it: a PENDING section is explicitly not a result | dated record |
| [`ARCHITECTURE_SEAMS.md`](ARCHITECTURE_SEAMS.md) | restructuring c2-rs for concurrent agents — and the origin of the one-file-per-rung convention, after nine parallel rungs all conflicted on the same three files. **Steps 0–3 landed**; §0 records what was executed and what the plan got wrong. Steps 4 and 5 are still plan | part landed, part plan |
| [`CAPTURE_CACHE_DESIGN.md`](CAPTURE_CACHE_DESIGN.md) | does `work/capture-cache` need a new storage structure, or just a delete? **Just a delete**, plus a retention policy and a one-line environment change; the database is a clear no. The alternatives are costed so the decision is reviewable rather than asserted | dated record |

## Keeping this page true

* **Adding a document?** Add its row here in the **same commit**, with its
  freshness class and a line saying what question it answers.
* **Thinking of moving one?** Measure first —
  `grep -rI --exclude-dir=.git -F '<basename>' .` — and read
  [`DOC_CONVENTIONS.md`](DOC_CONVENTIONS.md) §3. Of the 70 top-level documents
  present at `0636051e9` (2026-08-22, the tree the survey ran on — two peer
  lanes have since added one each), exactly **two** had zero inbound references.
* **Just edited a doc?** `scripts/doc_cite_audit.sh --self-test` (watch it go
  red on planted defects), then `scripts/doc_cite_audit.sh` — ~2 s over the
  whole `docs/` tree.
* **Scratch artifacts** referenced throughout these docs — `work/…` grids,
  captured bundles, `/tmp` fixture objs — are session-local and
  **regenerable**: `c2rs capture <fixture.cpp>` re-captures a bundle, and
  compiling any fixture under the reference toolchain regenerates the objs.
  Nothing binary is committed.
