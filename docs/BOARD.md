# BOARD — the numbered items, enumerated

`ROADMAP.md` references numbered work items everywhere (`board #121`,
`roadmap #43`, `#152`) and **has never listed them**. Reconstructing the board
meant grepping 8,000 lines and reading each hit's surrounding paragraph to find
out whether the number was a work item at all. This file is that list.

It is **hand-maintained** — unlike `rungs/INDEX.md`, there is no header block to
generate it from. Adding a board item means adding a row here **in the same
commit** that mints the number.

---

## Conventions

**The status field and the payoff field are separate, on purpose.** §9.16.1
records what happens when a board's payoff field and its outcome field are the
same field, and #149 is the live instance: its *conversion* was declined while
the *item* stayed open at 356. A board with one column gets that wrong. So:

| status | means |
|---|---|
| **OPEN** | live work. May carry a re-priced number from a lane that measured it and declined to build it. |
| **DONE** | shipped, with the differential behind it. |
| **DECLINED** | measured, deliberately **not** built, with a number attached. The measurement is the deliverable. A declined item is not a failed one. |
| **REFUTED** | the claim was tested and is **false**. The most valuable rows here. |
| **PARTIAL** | some of it shipped; the rest is named. |
| **UNCLEAR** | the prose does not support a status. Not a guess — a flag that someone must read the bytes. |

**Numbers are never reused and never renumbered.** Every `#N` in `ROADMAP.md` is
a permanent reference into 8,000 lines of prose that will not be rewritten.

**Next free number: `#159`.** The sequence is sparse — 1–4, 6–13, 16–18, 20–34,
36–42, 45, 49–51, 54–61, 63–91, 93–102, 104–109, 111–117, 123–126, 129–130, 147
and 148 were never used. **Gaps are not lost items.**

> **`GAPS.md` §6 runs a SEPARATE series.** Its mis-emit instances #1–#15 are a
> defect *taxonomy*, not units of work, and they collide numerically with board
> #5, #14 and #15. "§6 #15" (a `volatile` stored value) and "board #15" (the
> capture cache) are different things. **Never absorb one series into the other.**

### The 2026-08-01 collision — six numbers carry two meanings each

Two lanes running concurrently (`w-eh` and `w-rerank`) both allocated **143, 144,
145, 146** on the same day, for eight different items; `w-arms` and
`w-emitset`/`w-vgl` did the same to **151** and **152**. `w-eh` records *why*:
its items were "kept out of `ROADMAP.md` on purpose" while the coordinator landed
§9.14 serially — so neither lane could see the other's allocation.

Downstream prose now cites **both senses without disambiguating**. Both are kept
below, suffixed by their minting lane. Renumbering them would silently break
whichever citations meant the other sense.

**The rule that prevents the next one: mint the number by adding the row here
first.** A number that exists only inside a worktree is a number two lanes can
allocate at once.

---

## Open

| # | item | worth (measured, not estimated) | defined | notes |
|---|---|---|---|---|
| **158** | The `??__E`/`??__F` thunk body — **`LO_MARKER` is `4C` plus an OPTIONAL `4F 11`**, modelled as one atomic 3-byte token | **2 TUs** in the 4-TU `segments < COMDATs` bucket; 145 B of `.ex`, **byte-identical** across both | R:§10.11, §10.12 | **CHARACTERIZED, not built.** Nine functions in five captures: every `??__E`/`??__F` carries bare `4C`, everything else `4C 4F 11` — **including `??_G`**, which refutes "generated bodies differ". Fixture `il_dyninit_static.cpp` (2 lines). Two halves owed: **(a)** the decode — `4C` is also `IntCallEnd`'s tail and `VoidCallEnd`'s head, so re-tokenizing touches every function behind the byte-exact K1 gate; **(b)** the obj shape — `.rdata` + `.bss` + `.CRT$XCU` beside `.text`, against a fixed four-section shell. **The 2 TUs need both.** |
| 152<sub>w-emitset</sub> | Synthesize the `??_` COMDAT family (no `.ex` body exists) | priced +27 TUs — **must be re-measured against 4,591, not 13,646** | R:7734 | Two thirds of the "synthesis wall" was #151's reader defect. `special-generated` residue fell 90 → 6. First probe named: `TomCryptLicense`/`ZlibLicense` — **and §10.11 removed them from this item**: they have a `4F 1F` start and a bound `.gl` record, so they are #158 (a decode), not a synthesis. Whatever remains of #152 must be re-scoped without them. |
| 131 | The receiver designator — the whole site | 37,060 emitted blocked / 9,111 clean / **honest worth ≈2,600 (1.4 pp)** | rungs/w-adjust:52 | Largest single site on the emitted board (29.3 % of blocked emitted). Raw stock **overstates by ~14×**. Arms convert 19× apart, so **no rate transfers between them**. Needs a refactor returning an operand form richer than a token. |
| 142 | The other clean-not-whole receiver arms | site 25,654 emitted / 7,842 clean; decomposed into **27 named constructs** | R:5070 | Registered 35,700 → **MISS below the floor**. Ranking blocked by **#154**. Sizing off 25,654 is over by ≥8.0 % per #152<sub>w-arms</sub>. |
| 154 | `Block::completeness` returns `NoSignal` for every refusal minted outside `CALL_IN_EXPR` | **98.7 % of #142's clean stock** | R:6755 | Blocks ranking-by-completeness on the largest site on the board. Either the walker reaches these positions or the board needs a second completeness producer. |
| 149 | The off-add ARGUMENT slot — a `SlotArg` variant and its permutation position | **356 emitted** (conversion DECLINED; item open) | R:6717 | The fitted ordering rule **agreed on 360 of 360 cells and died on the 361st**; 98 witnesses refuted it. **"Do not re-derive the rule from grids 1–2 alone."** Entangled with #155, #157. |
| 153 | The three receiver arms worth ranking | `no-b9-this-adjust` 3,063 clean; `then-off-add` 2,856 clean (**unmeasured**); `then-dynamic-cast` 542 over 115 names | R:6745 | Two clean→realized ratios measured: **6.5×** and **142×**. `clean` is an optimistic ceiling, and the spread is why it cannot be scaled. |
| 140 | `expr-intrinsic-this-adjust` at adjust offset 0 | **434 emitted** (472 any offset) | R:5060 | Sink is 30 lines, already in `db812f7`. **"Schedule it at 434, not at 8,790."** Blocked on the same refactor as #131. |
| 141 | `call-arg-sym-permuted` — the data address beside a formal that must move | sized off **one probe** | R:5064 | Blocks **every free-function caller of the #128 shape**. R:7558 says it is **the same object as #155**; measure both on one grid. |
| 155 | The r11 pre-save is a rule of its own, under-scoped everywhere | `sym_slots_text` refuses at "two shifting formals"; **grid 3 fires it at one**; 98 of 394 cells | R:7555 | Same object as #141. Grid over (base register position) × (walk length) × (wide/narrow offset). |
| 157 | A computed address whose base formal is passed on the stack | grid 3 arity-8 cells; **8 witnesses captured** | R:7569 | Out of the modeled domain. Named so a later grid does not rediscover it as an anomaly. |
| 53 | EH step 1 — the 8-byte personality prefix; function symbol moves to `Value = 8` | **20 objs, 21 EH functions, `Value = 0x8` on every one** | EH:461 | Transcribed, not implemented. First step of a phase behind **233,526 functions (13.1 % of all blocked)**. |
| 143<sub>w-eh</sub> | `__catchsym$F$k` — the per-function symbol ordinal | the one piece of the EH record set §11 could not model | R:5389 | **Blocks a byte-exact Phase-5 emitter on any function with a try block.** Reaches the obj string table. |
| 144<sub>w-eh</sub> | `nIPMapEntries` for try/catch shapes | `h_try1` 1, `h_try2seq` 4, `h_try3seq` 7, `h_nest3` 3 | R:5394 | **"Not a function of any count in `FuncInfo`."** §9.7 refuted the no-try rule; this lane declined to guess rather than fit. |
| 146<sub>w-eh</sub> | Repair `EH_RECORDS.md` §9.8's `G = 4 + Σmint` | base is **`2 + 2`**; `Σmint` should range over **all** §1.1-style surcharges | R:5402 | Instrument correction, not a rung. Its `qLOOP` miss was already this defect. |
| 118 | The numerator's binding residue — census row ↔ obj symbol | **9,275 emitted symbols (5.18 %)** no census row claims *(was 17,706 / 9.89 %)* | R:4390 | The denominator is exact and watched; the numerator's residue is not. **#136 was never able to reach it** — needs a different instrument. |
| 119 | An instrument for general allocator/scheduler demand | **none — "the answer to #119 is a refusal, not a number"** | R:4136 | "The largest unbounded unknown, and it has no instrument." #134 was meant to answer it and was itself refuted. |
| 14 | The symbol-binding seam — positional vs per-record 1:1; a wrong resolution is **silent** | "moves the census numerator by ~2 %" | SEAMS:52 | Pinned apart by a standing test. `gl_defined_names` is the correct locator. |
| 151<sub>w-arms</sub> | The completeness walker cannot COUNT, and reads its own inability as a bug | every `-then-<x>-more` key with no `-and-` is a candidate | R:6731 | **Same family as #110/#139: one measure, wrong about what it is measuring.** An `Admit` carrying multiplicity would name these `-whole2`. |
| 152<sub>w-arms</sub> | The receiver-designator site is at least 8.0 % assignments | **2,060 of 25,654** emitted rows are `calls-0` — no CALL token at all | R:6739 | Any sizing of #131/#142 off 25,654 is over by at least that. |
| 146<sub>w-rerank</sub> | Extend the correspondence guard beyond the call-argument region | "each new pair costs one enumerated test and, on this evidence, finds something" | R:5827 | Generalizes the #139 repair to every place a census measure shadows a shipping production. |
| 145<sub>w-rerank</sub> | `configure_existing_worktree.sh` should say why it does not link the capture cache; `--validate-cache` should report a path-length mismatch, not self-heal it | — | R:5824 | Source of the standing rule: **"a validator that cannot see its own defect is worse than none."** |
| 156 | `prefilter` and `differential` disagree about function-level linking | `.text` characteristics **`0x60401020`** vs **`0x60400020`** on the same source | vgl-prereg:118 | A body the differential grades `Port=Match` reads `bytes-diverge at 217` through `prefilter`. Nothing shipped depends on it, but `prefilter` is the reject-only seam callers are meant to trust — **one of the two is wrong about the workload's real flags.** |
| 35 | General non-leaf lowering | frame model **DONE**; Class A many-calls rung 1 **byte-exact** | R:1648 | **PARTIAL.** Still blocked on >1 call per body (call sequencing, r3–r10 + spill, live ranges across calls) and the label stride of every class it will admit (`__savegprlr_N` stride is **7, not 5**). |

## Declined and refuted — the rows that saved work

| # | item | verdict | number | where |
|---|---|---|---|---|
| 122 | "TU match 6 → up to 15" | **REFUTED** | TU match **6 → 6**. The "15" was the item's own ceiling restated as an outcome — *"the string has never existed in this repository."* Real ceiling **25 of 871** until Phase 7. | R:5857 |
| 134 | `/QXSTALLS` scheduling-demand axis (was to answer #119) | **REFUTED** | blocked emitted 91.17 % vs in-class control 14.93 % — **does not survive its own control** once stratified by exact instruction count. | R:4305 |
| 127 | `expr-intrinsic-this-adjust` (the receiver `this`-adjust row) | **DECLINED** | **+472** any offset, **+434** at offset 0 = **5.4 % of the row**, against a row listed at 8,790. | R:4861 |
| 143<sub>w-rerank</sub> | `…recv-load-then-off-add-more` | **DECLINED** | 1,038 emitted / 851 clean → realizable **6 here, 356 elsewhere**. 1,008 of 1,038 bail at one key. The 356 was spun out as **#149**. | R:6518 |
| 150 | `expr-op-0x27` — the #1 row on the emitted board | **CLOSED at 6** | 22,759 emitted / 407,016 bodies; granting its named token converts **6 emitted functions**. *"The board should carry 6, not 22,759."* | R:7573 |
| 151<sub>w-vgl</sub> | Read the virtual member's `.gl` record shape | **DONE — and the described defect was REFUTED** | Priced +88 TUs. The actual defect: the `.gl` name separator is `00` **or** `26`. MODEL ceiling **111 → 324**; wall **755 → 451**; unbound-with-no-record **13,646 → 4,591**. **TU match 6 → 6 — all four numbers are ceilings.** The repair the original reading invited is worth **exactly zero, measured**. | R:7584 |

## Done

| # | item | number | where settled |
|---|---|---|---|
| **151**<sub>w-vgl</sub> **→ realise it** | Teach the **gate** the `26` name separator (`gl_defined_names`) | **DONE — and the "unrealisable ceiling" framing was WRONG.** The gate now names records on **270 of 871** TUs it previously refused to name at all (**8,583 records**), and **0** bound names changed. But **TU match 6 → 6**, and the MODEL ceiling did not move (324 / 420 / 451 at both ends) — it never was a gate number. What this bought is a *precondition*, not a payoff: the binding is no longer the blocker on 270 TUs, and it was silently the blocker on all of them. | rungs/w-adopt |
| 15 | The capture cache | **36.5 s → 0.9 s**; capture was 98.7 % of a scan; the 1.5–3 s estimate was wrong by ~2× | R:2034 |
| 43 | `eat_int_like`'s four-triple whitelist (W22) | estimate 5,684, realized **+15,924** — **wrong by 2.8×** | R:1581 |
| 44 | The census/gate disagreement | closed at disagreement **0**; never sized before | R:1459 |
| 46, 48 | Provenance and loader reporting | records workload + HEAD + resolved toolchain paths + wibo version | R:2063 |
| 47 | The census/gate invariant in **both** linkage modes | `/Ox` 1 and `/Gy` 9 refusals, **0 functions** on the workload | R:2074 |
| 52 | `opt_word_at` — the word is a **varint** and the reader was not | **0 functions**; `opt-mode` keys 570 → 568 | R:2474 |
| 92 | The range audit — every negative re-run to the ceiling | census unchanged; **"two negatives turned out not to have been measured"**; range control existed for only 2 of 8 probe modes | LC:4532 |
| 103 | The second census over-claim | disagreement **4 → 0**; verdict was **BOTH** — port under-implemented *and* census over-claiming | R:3530 |
| 110 | The `-whole{k}` over-count | closed with #139 — **but its own registered claim scored MISS at −3,761** (see Contradictions) | R:5812 |
| 121 | `codec::gl_offset_framed`'s over-fit | **38** framed records on `App.cpp`, not the 34 the doc claimed; the gate binds **0 of 9,033 bodies** there | R:7699 |
| 128 | The named data object as a member call's receiver | **+1,385 emitted**, converts **100.4 %**, only 4 distinct mangled names | R:4911 |
| 132 | The `c2rs listing` seam | listing does **not** perturb the obj; "byte-faithful" overstated by exactly one class (7 relocated branches of 44 rows). **A decode aid, never a gate.** | R:4207 |
| 133 | Transcribe the EH record layout | census moves **0 by construction**; `maxState` = destructibles + 2 × lexical try blocks, **10/10 on unfitted shapes** | R:5101 |
| 135 | Model the label counter from `.cod` order | **PARTIAL** — ordinal rule 40/40 held out; cardinal prediction B5 **REFUTED**; ships no `plan_labels` change | R:4700 |
| 136 | Reconcile `.cod` `PUBLIC`/`PROC` against the obj COMDAT scan | error term **0.0000 pp** over 178,968 — **on the denominator only** | R:4365 |
| 137 | Portable pins for WR1's ordering rules | tests 571 → 579; WR1 had landed ~1,500 lines with the test count **unmoved** | R:4532 |
| 138 | What governs the label-number gaps | `G = 2 + 2×[first emitted in TU] + Σ(own surcharges)`; hypothesis C1 refuted — **0 %** of gap slots are named in the listing | R:5220 |
| 139 | Repair `eat_int_operands`'s type gate | claimed 7,983, measured **13,321** — **under-sized 1.69×** | R:5812 |
| 144<sub>w-rerank</sub> | The `volatile` operand class — admitted by the measure, refused by the emitter | had no board item at all until this found it | R:5819 |
| 145<sub>w-eh</sub> | Fix `bind.rs:84`'s doc comment | **38, not 34** | R:7699 |

## Unclear — someone must read the bytes

| # | item | why unclear | where |
|---|---|---|---|
| 5 | The `coff.rs` label-counter mis-modelling — the class behind **both** historical six-wrong-byte defects | #135 claims to retire it but **ships no `plan_labels` change** | R:4125 |
| 19 | The per-function optimization word (`4F 1F 80 <LE32>`) | listing prints it symbolically; never formally closed | R:4103 |
| 62, 120 | Section/COMDAT emission order with `SEGMENT` directives | co-cited as one site; the `.cod` now prints it; never closed | R:4100 |

---

## Contradictions carried deliberately

These are **not** bookkeeping errors to tidy away. Each is a place where the
prose disagrees with itself, and the disagreement is evidence about how the
project works. Resolving them silently would destroy that.

1. **#143–#146 minted twice, same day, two lanes** (R:5387 vs R:5810). Prose
   cites both senses undisambiguated: R:5978 and two preregs use #145 in the
   *w-rerank* sense; R:7699 resolves #145 in the *w-eh* sense. **Cause: numbers
   minted inside worktrees.** The fix is this file.
2. **#151/#152 likewise** (`w-arms` vs `w-emitset`/`w-vgl`). All of §9.20 is
   written in the second sense with no acknowledgement the first exists.
3. **#121 reads NOT settled → stands open → still open → settled** (R:5174,
   5215, 5400, 7699). Chronologically consistent; the earlier text was never
   amended. Compounded by R:5096: §9.2 and the lane brief attach #121 to **two
   unrelated artifacts**, and both were addressed.
4. **#110's closure is asserted but its registered claim scored MISS** (−3,761,
   R:5431), with R:5446 conceding "a ≥90 % drop was never what #110 claimed".
   #110 is never independently declared closed, and §9.11's separate
   `-whole`-suffix corruption (**18,931**) is still live.
5. **#149 is both "DECLINED" (R:7232) and "stays open at 356" (R:7550).**
   Reconcilable — conversion vs item — and it is exactly why this board has two
   fields.
6. **#136's 0.0000 pp is a denominator error term, not a numerator guarantee**
   (R:4385). Carried here so it is not read as the latter.

---

*Anchors: `R:` = `ROADMAP.md`, `EH:` = `EH_RECORDS.md`, `LC:` = `LABEL_COUNTER.md`,
`SEAMS:` = `ARCHITECTURE_SEAMS.md`. Line numbers are as of `a03b8c1` and will
drift; the section numbers in the notes are stable.*
