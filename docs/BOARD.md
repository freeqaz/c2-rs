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
| **PARTIAL** | some of it shipped; the rest is named. A PARTIAL row lives in the section its dominant half earns — #35 sits under Open (the unshipped rest is the live work), #135 under Done (the measurement is finished; what it failed to ship is named in the row). |
| **UNCLEAR** | the prose does not support a status. Not a guess — a flag that someone must read the bytes. |

**Numbers are never reused and never renumbered.** Every `#N` in `ROADMAP.md` is
a permanent reference into 8,000 lines of prose that will not be rewritten.

**Next free number: `#160`.** The sequence is sparse — 1–4, 6–13, 16–18, 20–34,
36–42, 45, 49–51, 54–61, 63–91, 93–102, 104–109, 111–117, 123–126, 129–130, 147
and 148 were never used. **Gaps are not lost items.**

> **`GAPS.md` §6 runs a SEPARATE series.** Its mis-emit instances #1–#15 are a
> defect *taxonomy*, not units of work, and they collide numerically with board
> #5, #14 and #15. "§6 #15" (a `volatile` stored value) and "board #15" (the
> capture cache) are different things. **Never absorb one series into the other.**

> **A bare `#1`–`#4` in `ROADMAP.md` is almost always a RANKING, not an item** —
> "the #1 census blocker", "#2 blocker at 141,800", "the row that was #4 by
> bodies". `#1` alone occurs 15 times that way and not once as a work item. Those
> numbers are effectively unusable for new items in that document.

**Checked, not asserted**: `scripts/board_audit.sh` lists every `#N` in
`ROADMAP.md` with no row here, suppressing the two known non-item series by an
explicit printed list rather than silently. It also resolves every section
anchor in this file (`R:§x` etc.) against a real heading in its target document,
lists any raw line-number anchor as drift waiting to happen, and lists every
board number that a `ROADMAP.md` **heading** names in a section no row of that
number cites — the mechanized form of the 2026-08-02 staleness defect, where
§10.11/§10.13 re-measured #152 and nothing forced the row to notice. `--check`
self-tests all of it without a toolchain, and every check is mutation-tested
(see the script header). Coverage at `a091e37`: **50 board numbers, 55 cited,
0 uncovered.**

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

**Which sense a bare `#N` means by default** — measured over every `ROADMAP.md`
citation, 2026-08-02. For five of the six numbers one sense dominates
completely: outside the minority sense's own minting list, downstream prose uses
the dominant sense every single time. A reader hitting a bare citation should
assume the default below; the minority sense never escapes its minting section.

| bare | default sense | citations | the other sense appears ONLY at |
|---|---|---|---|
| `#143` | *w-rerank*: `…recv-load-then-off-add-more` | 9 of 10 (§9.17 throughout, §10.1, §10.6) | R:§9.15.6 (`__catchsym$F$k`) |
| `#144` | *w-rerank*: "residue 0 is not a control" — cited as that principle | 7 of 9 (§9.17.1, §9.17.8, §9.18.1, §9.20.3, §9.20.6, §9.20.12, §10.11) | R:§9.15.6 (`nIPMapEntries`) |
| `#145` | *w-rerank*: the validator rule | §9.16.3, §9.18.2, two preregs | R:§9.15.6 (`bind.rs:84`; R:§9.20.4 settles that item without using the number) |
| `#146` | **no default** — each sense cited only at its own minting | — | R:§9.15.6 vs R:§9.14.10 |
| `#151` | *w-vgl*: the `.gl` record shape / `26` separator | §9.20 preamble, §9.20.5, §9.21, §10.9 | R:§9.17.10 (the completeness walker) |
| `#152` | *w-emitset*: `??_` synthesis | every citation from §9.20.5 through §10.13 | R:§9.17.10 (8.0 % assignments) |

The board's `<sub>lane</sub>` suffixes remain the *unambiguous* form; this table
is for reading prose that did not use them.

---

## Open

| # | item | worth (measured, not estimated) | defined | notes |
|---|---|---|---|---|
| **158** | The `??__E`/`??__F` thunk body — **`LO_MARKER` is `4C` plus an OPTIONAL `4F 11`**, modelled as one atomic 3-byte token | **2 TUs** in the 4-TU `segments < COMDATs` bucket; 145 B of `.ex`, **byte-identical** across both | R:§10.11, R:§10.12 | **CHARACTERIZED, not built.** Nine functions in five captures: every `??__E`/`??__F` carries bare `4C`, everything else `4C 4F 11` — **including `??_G`**, which refutes "generated bodies differ". Fixture `il_dyninit_static.cpp` (2 lines). Two halves owed: **(a)** the decode — `4C` is also `IntCallEnd`'s tail and `VoidCallEnd`'s head, so re-tokenizing touches every function behind the byte-exact K1 gate; **(b)** the obj shape — `.rdata` + `.bss` + `.CRT$XCU` beside `.text`, against a fixed four-section shell. **The 2 TUs need both.** |
| 152<sub>w-emitset</sub> | Synthesize the `??_` COMDAT family (no `.ex` body exists) | **RE-MEASURED: +69 TUs off the wall** (was +27), of which **only +4 reach the `today` ceiling** and **0 are a TU match** | R:§9.20.5, R:§10.13 | The 13,646 → 4,591 correction made the TU payoff go **up** 2.6× — a symbol denominator and a TU payoff are different quantities. 365 wall TUs carry a `??_` symbol; 296 have another blocker too. Probe TUs `TomCryptLicense`/`ZlibLicense` **moved to #158** (§10.11) — they have a bound `.gl` record, so they are a decode, not a synthesis. |
| **159** | `emit-unbound-no-record\|ordinary` — header-declared **base-class virtuals** the obj emits with no `.gl` body **record** | **RE-MEASURED: +65 TUs off the wall, +9 reaching `today`, 0 a TU match** (190 symbols; 56 of the 65 still carry has-record). Denominator **2,809 symbols / 341 TUs** | R:§10.13, R:§10.17 | **Worth MORE than #152 where it counts** — +9 at the `today` ceiling against #152's +4. §10.17 re-framed it: **98.2 % are virtual member functions**, and **`.gl` NAMES 2,809 of 2,809** through `gl_symbol_index`. What is missing is the framed body-start record, **not the name** — so this is a `.gl` framing question adjacent to #121/#151, **not synthesis**. §10.13's "6,271 symbols" spliced the count from `emit-residue-unbound|ordinary` (6,271 / 452 TUs); the TU price was always right. Step one is done: `C2RS_WALL_DUMP` already printed these names before §10.14 went looking for a COFF reader. Cheapest next probe: `?CanSelect@UIListProvider@@UBA_NH@Z` **binds in 50 TUs and is no-record in 3** — one symbol on both sides of the framing rule. |
| — | **The wall does not decompose into items** | **305 of 451** wall TUs carry ≥2 live categories; every single-item price summed is 146 | R:§10.13 | Not an item — a **property of the board** for this phase. `dtor` is the sharpest case: 261 wall TUs carry one, **2** are unblocked by fixing dtors alone. |
| 131 | The receiver designator — the whole site | 37,060 emitted blocked / 9,111 clean / **honest worth ≈2,600 (1.4 pp)** | rungs/w-adjust:52 | Largest single site on the emitted board (29.3 % of blocked emitted). Raw stock **overstates by ~14×**. Arms convert 19× apart, so **no rate transfers between them**. Needs a refactor returning an operand form richer than a token. |
| 142 | The other clean-not-whole receiver arms | site 25,654 emitted / 7,842 clean; decomposed into **27 named constructs** | R:§9.13.4; decomposed R:§9.17.3 | Registered 35,700 → **MISS below the floor**. Ranking blocked by **#154**. Sizing off 25,654 is over by ≥8.0 % per #152<sub>w-arms</sub>. |
| 154 | `Block::completeness` returns `NoSignal` for every refusal minted outside `CALL_IN_EXPR` | **98.7 % of #142's clean stock** | R:§9.17.10 | Blocks ranking-by-completeness on the largest site on the board. Either the walker reaches these positions or the board needs a second completeness producer. |
| 149 | The off-add ARGUMENT slot — a `SlotArg` variant and its permutation position | **356 emitted** (conversion DECLINED; item open) | R:§9.17.10 | The fitted ordering rule **agreed on 360 of 360 cells and died on the 361st**; 98 witnesses refuted it. **"Do not re-derive the rule from grids 1–2 alone."** Entangled with #155, #157. |
| 153 | The three receiver arms worth ranking | `no-b9-this-adjust` 3,063 clean; `then-off-add` 2,856 clean (**unmeasured**); `then-dynamic-cast` 542 over 115 names | R:§9.17.10 | Two clean→realized ratios measured: **6.5×** and **142×**. `clean` is an optimistic ceiling, and the spread is why it cannot be scaled. |
| 140 | `expr-intrinsic-this-adjust` at adjust offset 0 | **434 emitted** (472 any offset) | R:§9.13.4 | Sink is 30 lines, already in `db812f7`. **"Schedule it at 434, not at 8,790."** Blocked on the same refactor as #131. |
| 141 | `call-arg-sym-permuted` — the data address beside a formal that must move | sized off **one probe** | R:§9.13.4 | Blocks **every free-function caller of the #128 shape**. R:§9.19.8 says it is **the same object as #155**; measure both on one grid. |
| 155 | The r11 pre-save is a rule of its own, under-scoped everywhere | `sym_slots_text` refuses at "two shifting formals"; **grid 3 fires it at one**; 98 of 394 cells | R:§9.19.8 | Same object as #141. Grid over (base register position) × (walk length) × (wide/narrow offset). |
| 157 | A computed address whose base formal is passed on the stack | grid 3 arity-8 cells; **8 witnesses captured** | R:§9.19.8 | Out of the modeled domain. Named so a later grid does not rediscover it as an anomaly. |
| 53 | EH step 1 — the 8-byte personality prefix; function symbol moves to `Value = 8` | **20 objs, 21 EH functions, `Value = 0x8` on every one** | EH:§7.5 | Transcribed, not implemented. First step of a phase behind **233,526 functions (13.1 % of all blocked)**. |
| 143<sub>w-eh</sub> | `__catchsym$F$k` — the per-function symbol ordinal | the one piece of the EH record set §11 could not model | R:§9.15.6 | **Blocks a byte-exact Phase-5 emitter on any function with a try block.** Reaches the obj string table. |
| 144<sub>w-eh</sub> | `nIPMapEntries` for try/catch shapes | `h_try1` 1, `h_try2seq` 4, `h_try3seq` 7, `h_nest3` 3 | R:§9.15.6 | **"Not a function of any count in `FuncInfo`."** §9.7 refuted the no-try rule; this lane declined to guess rather than fit. |
| 146<sub>w-eh</sub> | Repair `EH_RECORDS.md` §9.8's `G = 4 + Σmint` | base is **`2 + 2`**; `Σmint` should range over **all** §1.1-style surcharges | R:§9.15.6 | Instrument correction, not a rung. Its `qLOOP` miss was already this defect. |
| 118 | The numerator's binding residue — census row ↔ obj symbol | **9,275 emitted symbols (5.18 %)** no census row claims *(was 17,706 / 9.89 %)* | R:§9.9.3 | The denominator is exact and watched; the numerator's residue is not. **#136 was never able to reach it** — needs a different instrument. |
| 119 | An instrument for general allocator/scheduler demand | **none — "the answer to #119 is a refusal, not a number"** | R:§9.4 | "The largest unbounded unknown, and it has no instrument." #134 was meant to answer it and was itself refuted (R:§9.9.2 — "#119 still has no instrument"). |
| 14 | The symbol-binding seam — positional vs per-record 1:1; a wrong resolution is **silent** | "moves the census numerator by ~2 %" | SEAMS:§0 | Pinned apart by a standing test. `gl_defined_names` is the correct locator. |
| 151<sub>w-arms</sub> | The completeness walker cannot COUNT, and reads its own inability as a bug | every `-then-<x>-more` key with no `-and-` is a candidate | R:§9.17.10 | **Same family as #110/#139: one measure, wrong about what it is measuring.** An `Admit` carrying multiplicity would name these `-whole2`. |
| 152<sub>w-arms</sub> | The receiver-designator site is at least 8.0 % assignments | **2,060 of 25,654** emitted rows are `calls-0` — no CALL token at all | R:§9.17.10 | Any sizing of #131/#142 off 25,654 is over by at least that. |
| 146<sub>w-rerank</sub> | Extend the correspondence guard beyond the call-argument region | "each new pair costs one enumerated test and, on this evidence, finds something" | R:§9.14.10 | Generalizes the #139 repair to every place a census measure shadows a shipping production. |
| 145<sub>w-rerank</sub> | `configure_existing_worktree.sh` should say why it does not link the capture cache; `--validate-cache` should report a path-length mismatch, not self-heal it | — | R:§9.14.10 | Source of the standing rule: **"a validator that cannot see its own defect is worse than none."** |
| 156 | `prefilter` and `differential` disagree about function-level linking | `.text` characteristics **`0x60401020`** vs **`0x60400020`** on the same source | vgl-prereg:118 | A body the differential grades `Port=Match` reads `bytes-diverge at 217` through `prefilter`. Nothing shipped depends on it, but `prefilter` is the reject-only seam callers are meant to trust — **one of the two is wrong about the workload's real flags.** |
| 35 | General non-leaf lowering | frame model **DONE**; Class A many-calls rung 1 **byte-exact** | R:§6e; frame model R:§6g, rung 1 R:§6j, merged R:§6k | **PARTIAL.** Still blocked on >1 call per body (call sequencing, r3–r10 + spill, live ranges across calls) and the label stride of every class it will admit (`__savegprlr_N` stride is **7, not 5**). |

## Declined and refuted — the rows that saved work

| # | item | verdict | number | where |
|---|---|---|---|---|
| 122 | "TU match 6 → up to 15" | **REFUTED** | TU match **6 → 6**. The "15" was the item's own ceiling restated as an outcome — *"the string has never existed in this repository."* Real ceiling **25 of 871** until Phase 7. | R:§9.16.1 |
| 134 | `/QXSTALLS` scheduling-demand axis (was to answer #119) | **REFUTED** | blocked emitted 91.17 % vs in-class control 14.93 % — **does not survive its own control** once stratified by exact instruction count. | R:§9.9.2 |
| 127 | `expr-intrinsic-this-adjust` (the receiver `this`-adjust row) | **DECLINED** | **+472** any offset, **+434** at offset 0 = **5.4 % of the row**, against a row listed at 8,790. | R:§9.13 |
| 143<sub>w-rerank</sub> | `…recv-load-then-off-add-more` | **DECLINED** | 1,038 emitted / 851 clean → realizable **6 here, 356 elsewhere**. 1,008 of 1,038 bail at one key. The 356 was spun out as **#149**. | R:§9.17.5 |
| 150 | `expr-op-0x27` — the #1 row on the emitted board | **CLOSED at 6** | 22,759 emitted / 407,016 bodies; granting its named token converts **6 emitted functions**. *"The board should carry 6, not 22,759."* | R:§9.19.8 |
| 151<sub>w-vgl</sub> | Read the virtual member's `.gl` record shape | **DONE — and the described defect was REFUTED** | Priced +88 TUs. The actual defect: the `.gl` name separator is `00` **or** `26`. MODEL ceiling **111 → 324**; wall **755 → 451**; unbound-with-no-record **13,646 → 4,591**. **TU match 6 → 6 — all four numbers are ceilings.** The repair the original reading invited is worth **exactly zero, measured**. | R:§9.20 |

## Done

| # | item | number | where settled |
|---|---|---|---|
| **151**<sub>w-vgl</sub> **→ realise it** | Teach the **gate** the `26` name separator (`gl_defined_names`) | **DONE — and the "unrealisable ceiling" framing was WRONG.** The gate now names records on **270 of 871** TUs it previously refused to name at all (**8,583 records**), and **0** bound names changed. But **TU match 6 → 6**, and the MODEL ceiling did not move (324 / 420 / 451 at both ends) — it never was a gate number. What this bought is a *precondition*, not a payoff: the binding is no longer the blocker on 270 TUs, and it was silently the blocker on all of them. | rungs/w-adopt |
| 15 | The capture cache | **36.5 s → 0.9 s**; capture was 98.7 % of a scan; the 1.5–3 s estimate was wrong by ~2× | R:§6h |
| 43 | `eat_int_like`'s four-triple whitelist (W22) | estimate 5,684, realized **+15,924** — **wrong by 2.8×** | R:§6d |
| 44 | The census/gate disagreement | closed at disagreement **0**; never sized before | R:§6c |
| 46, 48 | Provenance and loader reporting | records workload + HEAD + resolved toolchain paths + wibo version | R:§6h |
| 47 | The census/gate invariant in **both** linkage modes | `/Ox` 1 and `/Gy` 9 refusals, **0 functions** on the workload | R:§6h |
| 52 | `opt_word_at` — the word is a **varint** and the reader was not | **0 functions**; `opt-mode` keys 570 → 568 | R:§6k |
| 92 | The range audit — every negative re-run to the ceiling | census unchanged; **"two negatives turned out not to have been measured"**; range control existed for only 2 of 8 probe modes | LC:§6.20 |
| 103 | The second census over-claim | disagreement **4 → 0**; verdict was **BOTH** — port under-implemented *and* census over-claiming | R:§6t |
| 110 | The `-whole{k}` over-count | closed with #139 — **but its own registered claim scored MISS at −3,761** (see Contradictions) | R:§9.14.2, R:§9.14.10 |
| 121 | `codec::gl_offset_framed`'s over-fit | **38** framed records on `App.cpp`, not the 34 the doc claimed; the gate binds **0 of 9,033 bodies** there | R:§9.20.4 |
| 128 | The named data object as a member call's receiver | **+1,385 emitted**, converts **100.4 %**, only 4 distinct mangled names | R:§9.13 |
| 132 | The `c2rs listing` seam | listing does **not** perturb the obj; "byte-faithful" overstated by exactly one class (7 relocated branches of 44 rows). **A decode aid, never a gate.** | R:§9.9 |
| 133 | Transcribe the EH record layout | census moves **0 by construction**; `maxState` = destructibles + 2 × lexical try blocks, **10/10 on unfitted shapes** | R:§9.15.1 |
| 135 | Model the label counter from `.cod` order | **PARTIAL** — ordinal rule 40/40 held out; cardinal prediction B5 **REFUTED**; ships no `plan_labels` change | R:§9.12.2 |
| 136 | Reconcile `.cod` `PUBLIC`/`PROC` against the obj COMDAT scan | error term **0.0000 pp** over 178,968 — **on the denominator only** | R:§9.9.3 |
| 137 | Portable pins for WR1's ordering rules | tests 571 → 579; WR1 had landed ~1,500 lines with the test count **unmoved** | R:§9.12.1 |
| 138 | What governs the label-number gaps | `G = 2 + 2×[first emitted in TU] + Σ(own surcharges)`; hypothesis C1 refuted — **0 %** of gap slots are named in the listing | R:§9.15.3 |
| 139 | Repair `eat_int_operands`'s type gate | claimed 7,983, measured **13,321** — **under-sized 1.69×** | R:§9.14.2, R:§9.14.10 |
| 144<sub>w-rerank</sub> | The `volatile` operand class — admitted by the measure, refused by the emitter | had no board item at all until this found it | R:§9.14.10 |
| 145<sub>w-eh</sub> | Fix `bind.rs:84`'s doc comment | **38, not 34** | R:§9.20.4 |

## Unclear — someone must read the bytes

| # | item | why unclear | where |
|---|---|---|---|
| 5 | The `coff.rs` label-counter mis-modelling — the class behind **both** historical six-wrong-byte defects | #135 claims to retire it but **ships no `plan_labels` change** | R:§9.3 |
| 19 | The per-function optimization word (`4F 1F 80 <LE32>`) | listing prints it symbolically; never formally closed | R:§9.2 |
| 62, 120 | Section/COMDAT emission order with `SEGMENT` directives | co-cited as one site; the `.cod` now prints it; never closed | R:§9.2 |

---

## Contradictions carried deliberately

These are **not** bookkeeping errors to tidy away. Each is a place where the
prose disagrees with itself, and the disagreement is evidence about how the
project works. Resolving them silently would destroy that.

1. **#143–#146 minted twice, same day, two lanes** (R:§9.15.6 vs R:§9.14.10). Prose
   cites both senses undisambiguated: R:§9.16.3 and two preregs use #145 in the
   *w-rerank* sense; R:§9.20.4 resolves #145 in the *w-eh* sense. **Cause: numbers
   minted inside worktrees.** The fix is this file.
2. **#151/#152 likewise** (`w-arms` vs `w-emitset`/`w-vgl`). All of §9.20 is
   written in the second sense with no acknowledgement the first exists.
3. **#121 reads NOT settled → stands open → still open → settled** (R:§9.15.2
   twice — its heading and its close — then R:§9.15.6, then R:§9.20.4).
   Chronologically consistent; the earlier text was never
   amended. Compounded by R:§9.15: §9.2 and the lane brief attach #121 to **two
   unrelated artifacts**, and both were addressed.
4. **#110's closure is asserted but its registered claim scored MISS** (−3,761,
   R:§9.14.1), with R:§9.14.1 conceding "a ≥90 % drop was never what #110 claimed".
   #110 is never independently declared closed, and §9.11's separate
   `-whole`-suffix corruption (**18,931**) is still live.
5. **#149 is both "DECLINED" (R:§9.19) and "stays open at 356" (R:§9.19.8).**
   Reconcilable — conversion vs item — and it is exactly why this board has two
   fields.
6. **#136's 0.0000 pp is a denominator error term, not a numerator guarantee**
   (R:§9.9.3). Carried here so it is not read as the latter.

---

*Anchors: `R:§x` = a section of `ROADMAP.md`, `EH:§x` = `EH_RECORDS.md`,
`LC:§x` = `LABEL_COUNTER.md`, `SEAMS:§x` = `ARCHITECTURE_SEAMS.md`. These are
**section numbers, not line numbers** — the anchors were line numbers until
2026-08-02, and one mid-file insertion (§9.21, landed before §10) shifted every
one past it; section numbers survive insertion. `scripts/board_audit.sh` checks
that every `PREFIX:§x` resolves to a real heading in its file and lists any raw
`PREFIX:<line-number>` anchor as drift waiting to happen. `rungs/<lane>:<line>`
and `vgl-prereg:<line>` are line numbers into frozen one-shot rung records under
`docs/rungs/`, which do not grow after landing.*
