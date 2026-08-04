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

**Next free number: `#206`.** The sequence is sparse — 1–4, 6–13, 16–18, 20–34,
36–42, 45, 49–51, 54–61, 63–91, 93–102, 104–109, 111–117, 123–126, 129–130, 147
and 148 were never used. **Gaps are not lost items.**

> **`#196`–`#205` are CLAIMED, and the rows below carry only 201–205.** Two
> landed rungs number their own findings in frozen text: `w-cfgimpl` names
> **196–200** (`rungs/2026-08-04-w-cfgimpl.md` §8) and `w-repro` names
> **201–205** (`rungs/_2026-08-04-w-repro-findings.md` §10, which chose 201
> precisely to avoid colliding with a lane it could not see). Lane `w-prov`
> landed w-repro's rows and left w-cfgimpl's to their owner rather than
> transcribing another lane's text — **do not re-mint 196–200**. Lane
> `w-pair`'s five proposals are deliberately unnumbered (`§6` of its rung) and
> are not holding any number. `w-prov` proposes **206–210**.

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
(see the script header). Coverage at 2026-08-04, after the w-book merges:
**86 board numbers, 81 cited, 0 uncovered**, 0 unresolved anchors, 0 raw
line-number anchors, 0 rows behind the prose. (It read **50 / 55 / 0** at
`a091e37`.)

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

### The 2026-08-04 collision — FOUR lanes all started at `#183`, and it cost nothing

`w-front`, `w-fifth`, `w-bss2` and `w-cfg` each finished the same session with a
proposed row list beginning at **`#183`**, the number this file advertised as
free. None of them wrote to this file, which is the rule working: the collision
was **visible at the funnel and resolved once**, instead of four lanes minting
four `#183`s the way 2026-08-01 minted two `#143`–`#146`s. Resolved by lane
`w-book` (R:§10.23), in this order:

1. **Strike duplicates before ordering anything.** Three of the ten proposals
   already had rows, so they were never candidates for a number: w-fifth's
   `??__F` row is **#180**; w-bss2's "writer scoped to ≤ 2 objects" is a *scope*
   for **#174**, which its own text says it supersedes; w-front's
   "`fn_gate_refusals` == 0" is a re-measurement of **#47**/**#44**. Two more of
   w-front's are not units of work at all. That deflated ten proposals to five
   new numbers and most of the contention with them.
2. **Where two landed records name numbers, prefer the assignment that needs no
   edit to either document.** `OBJ_DATA_BSS_SHAPE.md` §9 (w-bss2) named
   183–186; `CFG_SHAPE.md` §9 (w-cfg) named 186–194 and reasoned explicitly that
   "w-front proposed 183–185, so the next free number is 186". Striking w-bss2's
   writer row freed **185**, so w-bss2's fourth item moved 186 → **185** and
   w-cfg's nine landed **exactly** as its document writes them. One number moved,
   in the lane whose duplicate caused the gap; **no lane's frozen text was
   rewritten**.
3. **A number is not the only way to record a finding.** w-front's headline is a
   *property of the frontier*, not a unit of work, so it took the un-numbered
   `—` row form this board already uses for "the wall does not decompose into
   items".

**One redirect, so the frozen records stay readable**: `w-bss2`'s rung and
`OBJ_DATA_BSS_SHAPE.md` §9 both say *"#186 — census `.tls$`"*. It is **#185**.
Its other two numbers (183, 184) are as written.

**A false alarm worth keeping, because acting on it would have destroyed an
unrelated row.** w-bss2's rung reports that *"#166/#178 should be re-scoped or
struck"*. **#166 is not implicated.** The lane's §9 keeps its pre-mint bullet
list under the old numbers with a `#162 → #174 … #166 → #178` redirect at its
head, so *"#166"* there is **#178's own former number** — one item cited under
both spellings, not two items. Board **#166** is w-map's "retarget the *split*
concept to stripped x86 PE" and has nothing to do with `.bss`. See #178.

---

## Open

| # | item | worth (measured, not estimated) | defined | notes |
|---|---|---|---|---|
| **160** | **The COFF writer's section vocabulary** — the port emits **9** section names; the workload uses **13** | **C = 114 of 871 today** (was 84); the greedy ladder is now **four** steps: `.data` 169 → **`.rdata$r` 590** → `.text$yd` 804 → `.xdata$x` **871** | R:§10.19, R:§10.20, R:§10.21 | **The tightest of Phase 7's four factors, and the only one that is BOUNDED.** **2.96×** tighter than the binding ceiling (**338** — §10.19's 324 is stale). **§10.21: `.bss`, `.CRT$XCU` and `.text$yc` are LANDED** — w-r1c's `??__E` emitter added all three, which is where C's +30 came from; the three-step head of the old ladder is spent and the step sizes below it re-ranked, because the ladder is greedy. **Necessary, not sufficient** — C = 871 converts nothing alone. **§10.20 corrects two claims this row used to carry.** (1) **`.rdata$r` is RTTI, not EH** — 24,163 content symbols, all `??_R1..R4`, zero `__ehfuncinfo$`; it dies at `/GR-` and survives dropping `/EHsc`. Rung three is therefore an **RTTI** rung (+63: four fixed-layout COMDATs and a `??_R1` mangling), **Phase 5 moves C by zero** (EH records land in plain `.rdata`, a name the writer already has), and EH blocks by factor **D** over **740** objs, not 676. (2) The 13 names are **closed over this workload as measured, not closed by the language** — `#pragma init_seg("name")` mints a user-chosen name (w-emitpred `a7c7` → `.mycrt$a`). Measured: **0** occurrences of `init_seg`/`code_seg` in 78,746 workload source files (grep calibrated against `#pragma once` 1,009 / `warning` 208 / `pack` 47), so 13 holds empirically and the ladder is unaffected — but re-run that grep before any new corpus inherits the number. |
| **161** | The **emit predicate**, fitted black-box — least-fixpoint reachability from roots, ODR-use over kept definitions, vtable-forced virtuals, **no transitivity through dead code** | **0 violations on 172 designed cells**; unvalidated out of sample | R:§10.19 | Replaces the refuted separator-as-linkage reading (registered ≥95 %, measured **12.1 %**). **Must not ship without an out-of-sample gate with predictions committed before the held-out set is compiled** — that protocol has caught four rules that were perfect in-sample. Skip-naming channel exists (`/Wall` C4505/C4514, precision 1.00 / recall 0.928). |
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
| **180** | **`??__F`** — the atexit destructor thunk, the other half of #158's title | **PRICED EXACTLY, not estimated**: +2 sections (`.pdata`, `.text$yd`), +10 symbol records, and the `??__E` becomes framed — 0x40 bytes, 14 relocations, a `bl atexit` | `OBJ_DYNINIT_SHAPE.md` §4.4, rungs/2026-08-04-w-r1c.md, rungs/2026-08-04-w-fifth.md | The **decode already handles it** (w-r1 widened the parser for the bare `4C` both thunks share); the refusal is purely emit-side. A sized rung, not an unknown. **It is now also the whole-TU registry's first degradation test** (#179): `??__F` would arrive as a second acceptance arm in `PortC2::build`, which does **not** enter `WHOLE_TU_RECOGNIZERS` by itself, so the moment it converts a TU the `D∨E` control goes **red** and the printed ALARM names it — until someone deliberately registers it. **That red is the design working, not a regression**, and whoever lands `??__F` should expect it and register the recognizer in the same commit. **w-fifth proposed this as a new row at `#183`; it is this row** — same shape, same price, same §4.4 source. |
| **174** | **`.data`/`.bss` writer** — **RE-SCOPED 2026-08-04 by the measurement, not by preference** | the two names are worth **+55** to factor C on their own (`.data` 169), and `.bss` is already landed by #158 | `OBJ_DATA_BSS_SHAPE.md` §2–§6, §5.7, §8.11 | Build the writer from the spec. **The scope is now a measured bound, and it is far wider than "singular objects".** w-bss2 graded the layout on every real section where it can be wrong and found: a section with **one** object is trivially right and that is **23,253 of the workload's 24,055** `.data`/`.bss` sections; a `.bss` with **exactly two** is **47 of 48**; anything larger is **38 of 62**. So the gate is *emit at ≤ 2 objects per non-COMDAT section, `NotImplemented` above it* — which is nearly all of the workload — and the old "blocked-by #175 for mixed-size TUs" framing was **wrong about the axis**: it is not a mixed-size problem, because **10 of the 64 sections needing no alignment padding at all — where every candidate allocator coincides — are still wrong**. The residual is walk order (**#184**), not size. w-bss2 proposed this scope as a separate row and it is deliberately **not** one: its own text says it "supersedes the scope of #174", and a superseding scope is an edit to a row, not a second row for one writer. |
| **175** | **The `.data`/`.bss` address ALLOCATOR** (was titled *"the skip-and-retry walk"*) | **PARTIAL — the allocator is SETTLED and the row's own title is refuted.** It is a **plain bump with no free list**: exact on **110/117** real `.bss`, **68/68** real `.data`, **38/38** probe cells | `OBJ_DATA_BSS_SHAPE.md` §5.5, §5.7, §8.1 | The row asked for a prereg with predictions on a held-out mixed-size set; w-bss2 ran exactly that (R4, 19/20 held out). The answer is that **the question was in the wrong place**: "hole reuse", "pass-over" and "best-fit" are **not three rival allocators** — each is a different story about the order the objects were *visited* in, and every one of them emits a layout that is a bump in *some* order. So "skip-and-retry" names a walk, not an allocation policy, and there is no allocator left to determine. **What remains is spun out as #184**, on the precedent of #149 out of #143<sub>w-rerank</sub>. **§8.1 stayed OPEN on purpose and that is the row's other result**: both §5.5 counterexamples reproduce byte-for-byte and need *different* mechanisms — cell 10 by hole reuse and not pass-over, cell 11 the reverse — and no member of a 13-model zoo gets both, so the prereg's registered "state the boundary instead" clause fired rather than a fitted 14th model shipping. |
| **183** | **A FORWARD parser for the `.gl` record stream** | the error bar on every `.data` number w-bss2 published — including R2's *"67.6 %, wrong by 2.4 points"* | `OBJ_DATA_BSS_SHAPE.md` §8.10, §5.6 | §5.6 reads size, alignment, linkage and the deferral markers reliably (**12,207/12,207** on the R0 control), but reads the declaration-order **id** by scanning **backwards**, which is right on small TUs and **demonstrably wrong on some large ones**. Declaration order is what Rule A2 walks, so `.data` inherits the error and `.bss` — which depends only on file order — does not. **This is the lane's own self-reported defect, not a reviewer's**, and until it is fixed `#184`'s `.data` half cannot be graded. Gate: on all 871 workload TUs, every data record's id is distinct within its scope and a TU's namespace-scope objects form a contiguous ascending run. |
| **184** | **Close the `.bss`/`.data` WALK ORDER** — spun out of #175, which settled the allocator | Rule A1 reproduces **85 of 110** real `.bss`, Rule A2 **45 of 68** real `.data`; the residual is **entirely** this | `OBJ_DATA_BSS_SHAPE.md` §8.1, §8.11, §5.7.1 | **The only thing between here and the writer (#174).** Held-out set already exists: the 62 real `.bss` sections with more than two objects and the 27 `.data` ones. §5.7.1 characterises the 25 failing `.bss` precisely — the deferred clause is right in **24 of 25**, the eager block is a *near*-`.gl` order (median 0.17 inversions, 9 of 25 a single adjacent transposition), and **the transposed pairs share no size, alignment or linkage property**, which is why the obvious axes are spent. **Eliminated candidates are listed in §8.11 — do not re-run them**, and in particular not the `w-map` bucket walk (`id & 0x3ff`, bucket-ascending), which would have explained the probe grid and the real-TU deviation at once and scores **1 of 12, chance**. |
| **185** | **Census `.tls$`** | Rule T1 is fitted on **ten probe cells and has never been seen on a real TU** — `.tls$` is absent from the workload census entirely | `OBJ_DATA_BSS_SHAPE.md` §8.9, §5.8 | Rule T1: one section, **two blocks, uninitialized first**, each walked **backwards** — reverse `.gl` for the uninitialized block, reverse declaration order for the initialized one, ascending object size within each. **The mirror image of `.bss`/`.data`**, which is exactly why it should not be assumed to generalize. Multiplicity and COMDAT behaviour are both unmeasured. **w-bss2's rung and `OBJ_DATA_BSS_SHAPE.md` §9 call this `#186`; it is 185** — see the 2026-08-04 collision above. |
| **186** | **A `cflow-if-1` body often emits NO BRANCH AT ALL** | **6 of 7** `if-1` leaf probes in `pa.cpp`, and **both** real `if-1` functions in the frontier TU `Pool.cpp`, fold to a branchless arithmetic select or a `bclr` conditional return | `CFG_SHAPE.md` §0, §3.5 | **The shape the census names `cflow-if-1` is not the shape "a conditional branch" in the obj.** Three fold bands (§3.5). Consequence for scheduling, and it is the row's whole point: **grade a branch lowering on `xboxmem.cpp`, never on `Pool.cpp`** — an implementer who grades on `Pool.cpp` grades nothing. Found only because the prereg's registered rival at D2 was *also* branch-shaped, so only the bytes could produce the third reading. |
| **187** | The band-1 ↔ band-2 fold decision is a **c2 cost model** | **OPEN, and DECLINED here** — §3.5's eighteen-row table is **fitted by every cell and tested by none** | `CFG_SHAPE.md` §3.5, §8.1 B1 | The decline is the deliverable: shipping the fitted table as a specification is what this row exists to prevent. Settling it needs a ~30-cell probe varying **only the constant pair** over a fixed relation — named as the highest-value follow-up in `CFG_SHAPE.md`. |
| **188** | Condition registers are **two-valued**: cr6 for an explicit compare, **cr0** for a record-form | measured across three sequential compares in one body (cr6 **reused**, never allocated) and on every decrement-and-test loop (`addic.` → cr0) | `CFG_SHAPE.md` §3.2 | Hard-coding `BI = 4*6+bit` emits `409a…` where the obj has `4082…` — **a plausible-looking wrong branch**, which is the failure mode that survives review. |
| **189** | `if-2` and `if-n` add **no production** over `if-1` — admit all three in one rung | `if-1` alone unblocks **1** frontier TU; `if-1`+`if-2`+`if-n` unblocks **8** | `CFG_SHAPE.md` §4.4, §5.2 | **Splitting them across rungs costs seven TUs and buys nothing.** Read the numbers as **unblocked, not converted**: 32 of the 35 blocked frontier functions read `complete-none`, so every count in that ladder is a **ceiling** (§5.0 trap 1). |
| **190** | Leaf counted loops become **CTR loops** (`mtctr`/`bdnz`, `BO=16 BI=0`) | an instruction family absent from the port **and from `docs/` entirely** | `CFG_SHAPE.md` §3.7 | It drags in trip-count computation, which is **not a CFG problem**. A loop rung must decide up front whether its class includes them; the prereg predicted these would *not* appear (A2) and was refuted in scope. |
| **191** | Intra-section and external branches use **the same opcode with different encodings** | `48000008` (true displacement, no relocation) vs `4bffffec` (section-start placeholder + `REL24`) | `CFG_SHAPE.md` §3.3 | **A fixup pass that treats every `b` alike corrupts one of the two.** Discriminated only by whether the target is inside this section. |
| **192** | The epilogue block is emitted **even when unreachable** | `b_if`, `b_and`, `b_or` each end in a dead `4e800020` | `CFG_SHAPE.md` §3.6 | Every edge into it folded to a `bclr`, so no path reaches it — and it is still **four real bytes in the section size**. An emitter that drops it is short four bytes. |
| **193** | Block order is IL statement order — **with one measured refutation** | holds in **10 of 11** cells; `d_join` inverts | `CFG_SHAPE.md` §3.4, §3.4.1 | `d_join` tail-merges two identical `bl` sites, empties the then-block and inverts the layout. **Block order is downstream of code motion**, so a body whose arms end in the same call is **outside any class specified in `CFG_SHAPE.md`**. |
| **194** | **`c2rs capture` hardcoded `/Ox` and silently ignored flags** | **CODE FIXED at `6a33b4d` and pinned by `crates/c2-harness/tests/cli_flags.rs`. The DOCUMENT AUDIT is OPEN** | `CFG_SHAPE.md` §10.1, R:§10.23 | Every pre-`6a33b4d` document that read a capture bundle against `/O1`-compiled objs still needs **its own owner** to check it. **The mechanism, because it decides what to look for:** w-cfg's control measured that `.gl` and `.sy` are **byte-identical** either way and only the **7 per-function optimization words** differ (`0x00a00005` → `0x00200005`). So a document that read `.gl` or `.sy` is unaffected and one that read opt words is not — and **the failure is invisible without the comparison**, which means *"the numbers looked fine"* is evidence of nothing. Two lanes are already cleared **by their own controls, not by the fix**: w-cfg (used `census --flags-file` throughout; §10.1 shows the on-disk bundle reproduces byte-for-byte from that path) and w-bss2 (`work/w-bss2/cap.py` mirrors `capture_il_with` rather than going through the CLI). |
| **195** | **`cmd_compile` still parses by `position()` scan** — the same class, third instance, **not fixed** | `--flag` (which belongs to `listing`) is still **accepted and dropped** by `c2rs compile` | `crates/c2-harness/src/main.rs:599–615`, R:§10.23 | The fix at `6a33b4d` converted **`cmd_capture`** to a loop that *refuses* an option it does not know, and left `cmd_compile`'s three `position()` scans exactly as they were — verified on the merged tree. This is the **second** bug of the class recurring after the first was fixed: bug 1 was `c2rs compile` ignoring `--flag`, which made a `/GR` vs `/GR-` probe run **two literally identical command lines** and read the identity as a finding about RTTI. **The test table is already written for it** — `cli_flags.rs`'s `an_unknown_option_is_refused_not_ignored` covers `("capture", …)` and `("census", …)`; adding `("compile", "--flag")` is one line and currently fails. |
| **201** | **REFUTED: *“the IL capture is nondeterministic”*** | The front end is byte-exact | rungs/_2026-08-04-w-repro-findings.md §2, §6 | 24 serial and 48 concurrent captures of `Anim.cpp` → **one** distinct `.gl` sha256; six whole-871-TU censuses → exactly **two**, one per corpus; four censuses of a frozen tree at jobs 1/14/32 → byte-identical. The 0850/0920 disagreement is **68 TUs of moved corpus** (dc3 `dd9a4bdc` → `940d07dc`), proven causally by recompiling the pre-window blob and by censusing a `git archive` of it (0/871 differences in name-set, `ngl`, `gid`, size, align, linkage). |
| **202** | **Census provenance** — no artefact recorded which dc3 commit, which path or which `sections.jsonl` it was built from | **FIX LANDED by w-prov (see #207); the AUDIT of pre-stamp documents is OPEN** | rungs/_2026-08-04-w-repro-findings.md §9, rungs/_2026-08-04-w-prov-findings.md §4 | Cost of not having it: one whole lane. Every number published before 2026-08-04 was graded against a corpus nobody wrote down, and `sections.jsonl` itself was **18 TU records stale** when it was finally checked. |
| **203** | **The census join is path-bound** | **OPEN — PINNED, deliberately not repaired** | rungs/_2026-08-04-w-repro-findings.md §5, rungs/_2026-08-04-w-prov-findings.md §5 | MSVC's `?A0x<hash>` anonymous-namespace mangling is path-derived. Moving the corpus drops 48 TUs' anon symbols and **20 % of the graded population** while the printed rates hold (`.bss` 117→93 at 94.0 %→93.5 %, `.data` 68→53 at 100 %→100 %). New instance of trap 5. **w-prov measured that a path-free join fully restores the population and moves the winning `.bss` walk score 85/110 → 81/110**, so the pin is the fix and normalisation is the trap. |
| **204** | **`gid` framing is not established** | OPEN | rungs/_2026-08-04-w-repro-findings.md §4.1 | 46.04 % of kept records share a `gid` in-TU, 44.63 % of adjacent `.gl` pairs *decrease*, and the decimal-digit histogram is **bimodal** at 3 and 5 — which a monotone per-TU counter cannot be. Deterministic, so the landed numbers stand; but *“ascending id”* is the winning `.data` walk model and is fitted on this field. |
| **205** | **`globals_in_order()` admits non-symbols** | **OPEN — REVISED UPWARD, see #209** | rungs/_2026-08-04-w-repro-findings.md §5.1, rungs/_2026-08-04-w-prov-findings.md §7 | w-repro sampled 40 TUs and found 6.5 %. w-prov measured all 871 and found **18.39 %** (15,613 of 84,898), in 94.8 % of TUs, with **90.09 %** of real records sitting after at least one. Scores are unaffected — `i` is only ever a sort key and noise between two real records preserves their relative order — but `ngl` is inflated by 18.4 %. |
| — | **The FRONTIER is a control-flow frontier, and the breadth is missing in the wrong crate** | **15 of 17** frontier TUs sit behind `cflow-if-*` / `cflow-loop` / `cf-expr-0x05`; only **2** are straight-line-only | `rungs/2026-08-04-w-front.md` §2, `CFG_SHAPE.md` §5.1, R:§10.23 | Not an item — a **property of the frontier**, on the precedent of "the wall does not decompose into items" above. All 17 report one TU-level reason (`functions()` and `dyninit_tu()` both `None`), so `select_function` is **never reached** and `function_gate` refuses **zero** functions across all 878 TUs: **widening `crates/c2-core/src/codegen/` cannot move the frontier by one TU.** Two corrections travel with it: **`cf-expr-0x05` is NOT control flow** — it is a **DIV** width refusal (3 frontier TUs, unreachable by any CFG work) — and **`Pool.cpp` is the wrong TU to grade the CFG step on twice over** (its `if-1`s emit no branch per #186, and its constructor is `cf-expr-0x05`). |
| **176** | **Revise `OBJ_DYNINIT_SHAPE.md` §7.1, §4.1 and §2.3** against `OBJ_DATA_BSS_SHAPE.md` | three claims refuted; one of them was a **decline** that a later rung then reversed | `OBJ_DATA_BSS_SHAPE.md` §2.2, §4.2, §5.2; #169 | §7.1's *"would need the front end's hash reproduced"* is superseded (the permutation is an IL input — #169); §2.3's *"non-COMDAT sections carry CheckSum 0"* is refuted for `.data`; §4.1's *"`.bss` and `.CRT$XCU` always last"* is refuted — it is class-local, and w-r1c turned it into a gate rather than an assumption. |
| **177** | **The `c1xx` name-hash lane** | data already in hand: **1024 buckets**, an 11,000-name partition, and a refuted-family list (7,452 configurations, nothing above 0.08 against a 0.03 baseline) | `OBJ_DATA_BSS_SHAPE.md` §7.3, §7.4 | Only worth opening if something needs the hash itself. **The port does not** — the permutation is already in the `.gl` it is handed (#169). Filed so the negative result is not re-derived. |
| **166** | Retarget the *"split"* concept to stripped x86 PE | turns *"which of 4916 functions"* into *"which of ~200 pseudo-TUs"* | WB:`C2_MAP.md`, R:§10.21 | `jeff`'s `xex split` and `dtk` carve **PowerPC** images; `c2.dll` is x86 PE, so neither applies directly. The *concept* is portable and was the highest-leverage reusable technique found. **Superseded in practice by #167**, which achieves the same partition from c2's own ICE strings without needing the technique — kept open because the technique outlives this binary.  **NOT implicated by w-bss2's "#166/#178 should be re-scoped or struck"** — that citation is **#178 under its pre-mint number**, and this row was read and deliberately left alone; see the 2026-08-04 collision note above. |
| 35 | General non-leaf lowering | frame model **DONE**; Class A many-calls rung 1 **byte-exact** | R:§6e; frame model R:§6g, rung 1 R:§6j, merged R:§6k | **PARTIAL.** Still blocked on >1 call per body (call sequencing, r3–r10 + spill, live ranges across calls) and the label stride of every class it will admit (`__savegprlr_N` stride is **7, not 5**). |

## Declined and refuted — the rows that saved work

| # | item | verdict | number | where |
|---|---|---|---|---|
| 122 | "TU match 6 → up to 15" | **REFUTED** | TU match **6 → 6**. The "15" was the item's own ceiling restated as an outcome — *"the string has never existed in this repository."* Real ceiling **25 of 871** until Phase 7. | R:§9.16.1 |
| 134 | `/QXSTALLS` scheduling-demand axis (was to answer #119) | **REFUTED** | blocked emitted 91.17 % vs in-class control 14.93 % — **does not survive its own control** once stratified by exact instruction count. | R:§9.9.2 |
| **182**<sub>w-gc</sub> | "Deleting the capture caches reclaims **~266 GB**" | **REFUTED** | Deleting **98.7 % of 4.94 M entries** returned **~17 GiB** — off by more than an order of magnitude. The estimate came from `du -s` (**blocks × 512**, so every file rounds up to 4 KB) against files of **~850 B**; and btrfs **inlines** files under `max_inline` straight into metadata, so they never occupy a data extent at all. **The caches were an inode/metadata problem, not a data problem** — the cost was millions of metadata records and the walk time over them, which is why the standing rule is "never recursively walk `work/capture-cache`" and not "watch the disk". The suggested verification could not have shown it either: **`df -i` reports nothing useful on btrfs**, which has no fixed inode table. A recurrence of §6s's `/tmp` misreading in the opposite direction — there a metadata *limit* was read as a space limit, here a metadata *cost* was priced as a space cost. **Do not requote the 266**; it lived only in a cleanup lane's working notes, and `docs/CAPTURE_CACHE_DESIGN.md`, where the brief directed an in-place correction, **does not exist**. **Update (w-land4, same day):** it does now — the document was an unmerged lane branch, landed as `5e278f0`, and it **carries the in-place correction at its head** (markers **[C1]** bytes, **[C2]** 47→44 sibling caches, **[C3]** the age GC is unsafe as stated), with the original estimate left visible in the body. | R:§10.22 |
| 127 | `expr-intrinsic-this-adjust` (the receiver `this`-adjust row) | **DECLINED** | **+472** any offset, **+434** at offset 0 = **5.4 % of the row**, against a row listed at 8,790. | R:§9.13 |
| 143<sub>w-rerank</sub> | `…recv-load-then-off-add-more` | **DECLINED** | 1,038 emitted / 851 clean → realizable **6 here, 356 elsewhere**. 1,008 of 1,038 bail at one key. The 356 was spun out as **#149**. | R:§9.17.5 |
| 150 | `expr-op-0x27` — the #1 row on the emitted board | **CLOSED at 6** | 22,759 emitted / 407,016 bodies; granting its named token converts **6 emitted functions**. *"The board should carry 6, not 22,759."* | R:§9.19.8 |
| 151<sub>w-vgl</sub> | Read the virtual member's `.gl` record shape | **DONE — and the described defect was REFUTED** | Priced +88 TUs. The actual defect: the `.gl` name separator is `00` **or** `26`. MODEL ceiling **111 → 324**; wall **755 → 451**; unbound-with-no-record **13,646 → 4,591**. **TU match 6 → 6 — all four numbers are ceilings.** The repair the original reading invited is worth **exactly zero, measured**. | R:§9.20 |
| 163<sub>w-map</sub> | `ROADMAP.md` §9's *"`c2.dll` … is **not a stripped build**"* | **REFUTED** | No COFF symbol table (`NumberOfSymbols` = **0**), **4** exports, **0** RTTI type descriptors, and the CodeView entry is an `RSDS` reference to an *absent* `c2.pdb`. The `/FAsc` evidence originally cited is real but supports a different proposition: c2 is unusually **talkative**, not unusually **symbol-rich**. | WB:`C2_MAP.md` §0, R:§9 (~line 4074) |
| 165<sub>w-map</sub> | *"The pointer table at `.data 0x10c37c40` is a 9-slot array of section-name pointers"* | **CORRECTED — the lane's own proposal, wrong** | It is a table of *default name strings used as identity tokens*: `FUN_10b982d6` compares `sect->name == g_defaultName[k]` to decide whether to substitute the canonical literal for a possibly-`$`-suffixed name. The real entry point for factor **C** is **`FUN_10b982d6`**. Row kept rather than deleted — **the mis-reading is the useful record**. | WB:`C2_MAP.md` §3B |
| 169<sub>w-map</sub> | *"The `.bss` object-address permutation is a hash"* | **REFUTED** | Brute force over 9 name decorations × every modulus 2..8191 × every `(h>>s)&mask` × {asc,desc} × {FIFO,LIFO} = **0 hits**. No hash is needed: `.bss` ascending = **reverse** of `.gl` record order for dyninit objects, **=** `.gl` order for plain ones, groups never interleaving. **Reproduced independently and black-box by lane w-bss** — two routes to one mechanism. The residual permutation is **`c1xx`'s** and is already in the `.gl` the port is handed, so **the port never has to reproduce any hash**. Removes the stated premise for `OBJ_DYNINIT_SHAPE.md` §7.1's decline (#176). | WB:`C2_MAP.md` §3D, `OBJ_DATA_BSS_SHAPE.md` §5.2 |
| 173<sub>w-map</sub> | The `.bss` **offset** rule (align 8, right-justify sizes 1/2/4) | **RETRACTED — and the lane's best calibration datum** | Read at instruction level from a small, fully decompiled function and published at confidence `high`. **It does not reproduce real objs** (`OBJ_DATA_BSS_SHAPE.md` §5.5). Withdrawn to `unknown`; the obj is the sole judge. The value is the calibration: this was the **good** kind of static finding and it still failed, so **`high` in that map means "the instructions were read correctly", not "this is what c2 does"** — two propositions that come apart, and static analysis cannot tell you when. | WB:`C2_MAP.md` §3D-bis, WB:`C2_MAP_METHOD.md` §7 |

## Done

| # | item | number | where settled |
|---|---|---|---|
| **179** | **The factorization's FIFTH TERM, `E`** — whole-TU acceptance, disjoined onto D | **DONE.** `E` = *"at least one **registered** whole-TU recognizer accepts this bundle"*. The model is **`A ∧ B ∧ C ∧ (D ∨ E)` = 8, exactly the match set by name**; the control is green on its necessary terms (`A 0 B 0 C 0 D-or-E 0`) with `D 2` and `E 6` still **printed as diagnostics**, and §10.19's refuted `A∧B∧C∧D: 6` is still measured beside it — *a refutation whose quantity stops being measured is a claim nobody can re-check*. **0 of 878 TUs changed class**, census `706402/2463318` and `census/gate disagreement: 0` on both sides, **FRONTIER 17 → 17 with 0 in and 0 out** (both E-true TUs were already matches). **D was not widened** — its definition is byte-for-byte what it was, and nothing in `census.rs` was touched. **The registry is closed, explicit and named** (`WHOLE_TU_RECOGNIZERS` in `gap.rs`, one entry): a new acceptance arm in `PortC2::build` does not enter it, so an unregistered path makes `D∨E` go **red** and names itself. **The rejected one-liner is written into the code's doc comment so it cannot be quietly adopted**: `E := decodes() && functions().is_none()` needs no registry and is the anti-pattern — `decodes()`'s own doc says *"adding a third path means adding it here"*, so it would **silently absorb** a third recognizer and be green by construction. That is the whole difference between an instrument and a rubber stamp. | R:§10.21, R:§10.23, rungs/2026-08-04-w-fifth.md |
| **178** | Re-census with raw section bytes retained, to grade §5's allocator on all **24,055** real `.data`/`.bss` sections | **DONE — by a route this row did not propose, and the route it DID propose is REFUTED as unnecessary.** The grading happened (#175): **110/117** real `.bss`, **68/68** real `.data`. It needed **no raw section bytes at all**. The allocator's *output* was already in `sections.jsonl` — every defined symbol's `Value`. What was missing was its **input** — size, alignment, linkage and declaration order — **none of which is in the obj**; they are in the IL `.gl` c2 is handed, and locating them is what made the grading possible. **102 MB of retained bytes would have answered a different question.** **Kept rather than struck**, deliberately: striking it would delete the record of why the expensive re-census must not be retried, which is exactly what this board is for. w-bss2 proposed *"re-scope or strike"*; re-scoped, and the reason is the row. | R:§10.23, `OBJ_DATA_BSS_SHAPE.md` §8.8, §5.7 |
| **151**<sub>w-vgl</sub> **→ realise it** | Teach the **gate** the `26` name separator (`gl_defined_names`) | **DONE — and the "unrealisable ceiling" framing was WRONG.** The gate now names records on **270 of 871** TUs it previously refused to name at all (**8,583 records**), and **0** bound names changed. But **TU match 6 → 6**, and the MODEL ceiling did not move (324 / 420 / 451 at both ends) — it never was a gate number. What this bought is a *precondition*, not a payoff: the binding is no longer the blocker on 270 TUs, and it was silently the blocker on all of them. | rungs/w-adopt |
| **158** | The `??__E`/`??__F` thunk body — the decode, the obj shape, and the whole-TU recognizer that joins them | **DONE — TU match 6 → 8**, the first movement in this metric's history. `TomCryptLicense.cpp` and `ZlibLicense.cpp` are byte-exact against real `c2`; `vocab-gap` 865 → 863; census **+0**; `codegen-gap` still **0**; mismatch **0** everywhere. Landed in two rungs: **w-r1** built both halves and they converted **nothing** (`emit_dyninit_obj` had no caller — the halves were never the whole item), and **w-r1c** built the caller. `Bindings::resolve_data` was **never widened** — the join is a whole-TU recognizer an ordinary function cannot reach, which is why the census moved by zero. The `/Ox` fence is structural, not a flag test: `/GF` is implied by `/O1`/`/O2` but not `/Ox`, and at `/Ox` the literal is still in `.in` while no `??_C@` record exists in `.gl`, so a reader trusting `.in` alone emits the wrong obj by doing the obvious thing. **`??__F` is spun out as #180**; the factorization it broke is **#179**. | R:§10.11, R:§10.12, R:§10.21; rungs/2026-08-04-w-r1.md, rungs/2026-08-04-w-r1c.md |
| 162<sub>w-map</sub> | `docs/whitebox/` — a navigational map of `c2.dll` from static analysis | **4916** functions enumerated and decompiled with **0** failures (Ghidra 12.1.2 headless on the 1.35 MB image); 2791 literals cross-referenced; four generated reference tables plus the flat-export pipeline that builds them. **Navigation, not adoption** — `DISCLOSURE.md`'s adoption table has **zero rows** and nothing has been copied into `crates/`. | WB:`C2_MAP.md`, WB:`C2_MAP_METHOD.md`, WB:`DISCLOSURE.md` |
| 164<sub>w-map</sub> | The COFF writer lives in `c2.dll`, not in `msobjXX.dll` | c2 imports **exactly one** msobj symbol — `objf::ObjectCode::FCreateFromBytesW`, a **reader**. Every other msobj export it could reach is a reader too. This was an explicit decline-clause in the lane's brief (*"if msobj owns the writer, say so plainly — it relocates the map"*); it does not. | WB:`C2_MAP.md` §msobj |
| 167<sub>w-map</sub> | c2's **52 original translation units**, recovered from its own ICE strings — the map's spine | c2's C1001 path prints `compiler file '%s', line %d`, so the binary names its own sources; link order makes each file's xrefs a contiguous range. **Confidence metric, not an assertion**: adjacent-pair overlap **1/51 = 1.96 %** (0/50 between distinct TUs), gap coverage 28.1 % of bytes. Validated against a null the partition never used: 7 ascending runs vs 26.5 expected (**P = 1.5 × 10⁻²⁵**), longest run 33 files, every run directory-pure. **Two provenance tiers** — the file-name list is `strings` output and costs the clean-room claim nothing; only the addresses are white-box. | WB:`C2_MAP.md` §3, WB:`c2_tus.tsv`, WB:`DISCLOSURE.md` |
| 168<sub>w-map</sub> | **The emit decision is NOT c2's** — `0x20` is a SEED bit transmitted by `c1xx`, and the emitted set is its reachability closure | Walk loop `0x10b7f15f` in **`p2/main.c`**, not `coffemit.c`; the flag word is stored to `sym+0x4c` **verbatim from the IL** at `0x10b9bf78`. **Amended by obj-level mutation rather than left as a reading**: clearing `0x20` removes the function on a bundle of six independent leaves (`.text` shrinks by exactly its 16 bytes, rest byte-identical), but on a bundle with a real call graph **17 of 20 single clears change nothing**, and a 6-step cascade shows each function falls only once its caller is also cleared. **Consequence: the ODR-use decision behind #161's false-positive class is made in `c1xx`, and probing c2 will never find it.** Porting the seed test alone will over-delete on real TUs. **Refutation condition:** a called function vanishing when only it is cleared. | WB:`C2_MAP.md` §3E, R:§10.21 |
| 170<sub>w-map</sub> | `DAT_10c45f9c` is the `-optref` flag, not LTCG | Corrects a medium-confidence guess everything else hung off. It has **no WRITE xref anywhere in the export**, because c2's 147-entry flag table is built at run time by a 4250-byte store sequence and is invisible to a `.data` scan; a second child on a different seam reconstructed that table and the join answers it outright. **Two seams closed by intersection what neither could close alone** — the argument for keeping a map rather than isolated findings. | WB:`C2_MAP.md` §3E |
| 171<sub>w-map</sub> | **JamCRC is ABSENT from `c2.dll`** — the aux `CheckSum` is computed outside it | Pre-registered control, scored a **MISS**, and the miss is the valuable result. No `0xEDB88320` table at any 4-aligned offset in either bit order; the polynomial occurs nowhere as an immediate; the `A..P` renderer is absent. It lives in `mspdbXX.dll`. **The search method was itself controlled**: two constants the port hardcodes and a fresh obj demonstrably carries are *also* absent as immediates. | WB:`C2_MAP.md` §6 P1, WB:`PREREG.md` |
| 172<sub>w-map</sub> | The IL section record is `.gl` tag `0x09` — **factor C is predictable from IL** | Six fields in read order (`varU` section index, name, kind byte, class, Characteristics override). **Proven by mutation, not by reading**: patching the kind byte walks the section through `.text`/`.bss`/`.rdata`/`.debug$S` exactly as predicted, the override beats the kind, and swapping two records' ids moves the initializer pointer. COMDAT-ness is **not** in tag 9. Closes the last link between *"c2 emits the right shape"* and *"the port can predict it"* for the tightest of the four factors. **Refutation condition:** a section in a real obj whose Characteristics match neither its tag-9 override nor `FUN_10b982d6`'s kind switch. | WB:`C2_MAP.md` §3F, #160 |
| 15 | The capture cache | **36.5 s → 0.9 s**; capture was 98.7 % of a scan; the 1.5–3 s estimate was wrong by ~2× | R:§6h |
| 43 | `eat_int_like`'s four-triple whitelist (W22) | estimate 5,684, realized **+15,924** — **wrong by 2.8×** | R:§6d |
| 44 | The census/gate disagreement | closed at disagreement **0**; never sized before. **RE-MEASURED 2026-08-04 (w-front): still 0, and now from the other direction too** — `function_gate`, which runs `select_function` itself rather than a copy, records **`fn_gate_refusals` NONE across all 878 TUs** and none across the 17 frontier TUs. w-front proposed this as a new row; it is this one plus #47. | R:§6c, R:§10.23 |
| 46, 48 | Provenance and loader reporting | records workload + HEAD + resolved toolchain paths + wibo version | R:§6h |
| 47 | The census/gate invariant in **both** linkage modes | `/Ox` 1 and `/Gy` 9 refusals, **0 functions** on the workload. **Still 0 at 2026-08-04**, and w-front drew the consequence this row never stated: if the gate refuses zero functions anywhere, then **there is no live refusal in `crates/c2-core/src/codegen/` to widen** — the shapes never arrive. See the un-numbered frontier row under Open. | R:§6h, R:§10.23 |
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
| **181**<sub>w-gc</sub> | **The capture seam, hardened** — an `O_EXCL` lockfile for concurrent capture, a canonical `cwd` in the key, and one cache root per *repo* | **DONE — and the race was LIVE, not latent.** The per-key lock was an in-process `HashMap` guarding a *filesystem* resource, while `scripts/gate.sh --jobs N` already ran N separate `c2rs` **processes** against one root — and `capture_reference_with` **deletes every `_CL_*` in its work dir on entry**, so a colliding process destroyed the first's live IL bundle. Surfaces downstream as a **false `mismatch`**: an alarm pointing at the port while the port is fine. Also fixed: `cwd` was keyed by its raw spelling, so `../dc3-decomp` **aliased a different directory in every worktree onto one key**; and `repo_root()` is `CARGO_MANIFEST_DIR` (compile-time), which minted **50 caches / 3,996,458 entries**, three being independent copies of one 530k sweep. **Fail-open on every error path**, so the degraded case is *exactly* the old behaviour and never a wedged scan. `LOCK_DIR` is `pub` because it breaks the "every child of the root is 32-hex" invariant — **any age GC over the root must skip it**, those files are live cross-process locks. The cwd precondition (spelling must not reach the bytes) is held to the **real toolchain**, not asserted in a comment. `main_repo_root()` is resolved in code and **deliberately not** exported as an env var: a shared root is what first makes concurrent same-key captures possible, and a lockfile only guards binaries that *have* it — tying both to one build makes the rollout monotone. | R:§10.22 |

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

*Anchors: `R:§x` = a section of `ROADMAP.md`, `WB:<file>` = a file under
`docs/whitebox/` (**disassembly-derived — navigation only; adopting anything from
there into `crates/` needs a `DISCLOSURE.md` row naming the address**),
`EH:§x` = `EH_RECORDS.md`,
`LC:§x` = `LABEL_COUNTER.md`, `SEAMS:§x` = `ARCHITECTURE_SEAMS.md`. These are
**section numbers, not line numbers** — the anchors were line numbers until
2026-08-02, and one mid-file insertion (§9.21, landed before §10) shifted every
one past it; section numbers survive insertion. `scripts/board_audit.sh` checks
that every `PREFIX:§x` resolves to a real heading in its file and lists any raw
`PREFIX:<line-number>` anchor as drift waiting to happen. `rungs/<lane>:<line>`
and `vgl-prereg:<line>` are line numbers into frozen one-shot rung records under
`docs/rungs/`, which do not grow after landing.*
