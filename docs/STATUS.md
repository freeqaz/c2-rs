# STATUS — where the port is, and how to check for yourself

**This doc is a cache, not a source.** Every number in the generated block below
comes from `scripts/status.sh`; regenerate it with `scripts/status.sh --write`
rather than editing it by hand. If the block and the tree disagree, the tree is
right — rerun the script.

The rest of the doc is the part a script cannot produce: what each number
**means**, what it **does not** mean, and which of them is actually the target.
That distinction is the whole content of ROADMAP §8, and getting it wrong has
cost this project real work more than once.

---

## The numbers

> ### ⚠ THE BLOCK BELOW IS STALE — **7 merges and 38 commits** behind master
>
> Collected at tree `26306ba`; master is `33cbdbe`. Flagged 2026-08-04 by lane
> `w-book4`, which **cannot regenerate it** — `status.sh` needs `../dc3-decomp`,
> which does not resolve from a worktree, and ten of fifteen metrics come back
> `NO-RESULT` (see *Reproducing any of it* below). **Run
> `scripts/status.sh --write` from the main repo.** Nothing here is hand-edited
> into the block; this banner is outside it on purpose.
>
> **What moved, and where each figure was measured** — every one is from a landed
> rung's §1 result table, not from this lane:
>
> | metric | block reads | measured at `33cbdbe` | source |
> |---|---|---|---|
> | Workspace tests | 706 | **763 passed, 0 failed, 25 targets** | `rungs/2026-08-04-w-label.md` §1 |
> | Oracle self-test | 225 PASS | **245 PASS / 0 FAIL** | same |
> | Fixture port gate | 106 Match / 119 n-i of 225 | **118 Match / 0 mismatch / 127 n-i of 245** | same |
> | Perf geomean | 568× over 106 | **565× over 118** — a change of *population*, not a regression (GAPS §1) | same |
> | **878-TU scan** | match 8, mismatch 0, codegen-gap 0, vocab-gap 863, capture-fail 7 | **every digit unchanged** | same |
> | Per-function / emitted census | 706555/2463393 · 38458/178975 | **both unchanged** | same |
> | Factor **C** | — (prose said 114) | **169** | `rungs/2026-08-04-w-sect.md` §10, re-read at w-label §1 |
> | `A∧B∧C` · FRONTIER | — (prose said 25 · 17) | **27 · 19** | same |
> | **`B∧C`** | — (prose said 107, taken at `C = 114`) | **151** at `c303ad0` | `rungs/2026-08-04-w-bc.md` §2 |
> | **frontier-if-A** · the `+82` projection | — (prose/board said 99 · +82) | **141** · **+124 reach / +122 frontier — and those are two different numbers now** | same, §3 |
> | workload stamp | `940d07dc` | **`fe1b5b39`** — a stamp change only: **0 of 878 workload source blobs differ**, checked | same, §6.1 |
>
> **Seven merges moved TU match by zero, and that is the expected result rather
> than a bad day** — the same reading the paragraph below already gives for the
> twelve before them. What they moved instead is the *warranty*: **three
> independent live wrong-bytes families were found and closed** — board **#259**
> (six mismatching cases, not the one that was filed), **#263** (the `/EHsc`
> `eh-bare` slot, at the workload's own `/O1 /EHsc`) and **#276** (a TU that
> defines data and no functions, older than #232) — and the sweep grew from
> 14,484 cases to **14,817 reached / 14,721 graded**, with
> `scripts/mode_cross.sh` added as a second gate row at **63,723 selected /
> 63,335 graded**.

<!-- BEGIN GENERATED: scripts/status.sh — do not hand-edit -->
Collected 2026-08-05 · tree `218cee1` · binary `7567cf518e62` · workload `20a48363`

| metric | value |
|---|---|
| Workspace tests (cargo test --workspace --release) | 799 passed, 0 failed, 27 targets |
| Oracle self-test (c2rs selftest) | 245 PASS, 0 FAIL |
| Fixture port gate (c2rs perf) | 118 port Match, 0 mismatch, 127 not-implemented (of 245) |
| Port speedup, geomean over matched fixtures | 481x geomean over matched fixtures |
| 878-TU dc3 workload scan (c2rs gap) | match 8, mismatch 0, codegen-gap 0, vocab-gap 863, capture-fail 7 |
| Per-function census (driver, not target) | 706555/2463393 functions in class (28.68%) |
| Emitted-function census | 38458/178975 emitted functions in class (21.49%) |
| Emitted-census residue | residue 9225: 1962 compiler-generated (no IL body), 7263 unexplained  (5.15% of the denominator) |
| TU distance to match, blocked functions | ≤0: 1, ≤1: 12, ≤10: 27, ≤100: 34, ≤1000: 212 |
| TU distance to match, blocked emitted functions | ≤0: 2, ≤1: 19, ≤10: 82, ≤100: 403, ≤1000: 858 |
| Emit-set ceiling, LO-anchored (segments == COMDATs) | 27 of 871 graded TUs |
| Emit-set ceiling, GATE-anchored (4F 1F — what the port consumes) | 28 of 871 graded TUs |
| Emit-set MODEL ceiling (today / repaired / wall) | 338 today / 420 repaired / 451 wall |
| .gl binding invariants (records / arity / conflicts) | 1515167 records, 420 nameless, 0 before the first row, 39296 row-conflicts, 731 name-conflicts, 0 accounting breaks, 0 unreadable objs |
| Phase-7 factors over the graded TUs (A / B / C / D / E) | A 28 (LO 27) · B 338 · C 169 · D 8 · E 2, of 871 graded |
| Joint ceilings (B∧C, A∧B∧C) | B∧C 151 · A∧B∧C 27 · A∧B∧C∧D 6 |
| Pre-Phase-7 FRONTIER (codegen breadth alone / if A were free) | 19 reachable by codegen breadth alone; 141 if factor A were free |
| Emit-predicate worth, B∧C − A∧B∧C (board #213) | +124 TUs (B∧C − A∧B∧C) |
| Factor-C section ladder (writer names / workload names / next step) | 10 writer names of 13 workload names; 3 steps left, next +.rdata$r → C = 590 |

<!-- END GENERATED -->

---

## The one-paragraph answer

The **foundation is proven and fast**: standalone replay of the real `c2.dll` is
byte-exact on all 871 capturable TUs of a real Xbox 360 game, and the port is
byte-exact on every shape a standing instrument grades. The **payoff metric has
moved for the first time**:
TU match is **8/878**, up from a 6 that had held across a per-function census run
from 4.45 % to 28.69 %. The two new TUs are
`src/system/synth/tomcrypt/TomCryptLicense.cpp` and
`src/system/zlib/ZlibLicense.cpp`, converted by a **whole-TU `??__E`
dynamic-initializer recognizer** (`IlBundle::dyninit_tu`), not by widening the
per-function class — the census is **+0** across that change and `vocab-gap` fell
865 → **863**. That is worth reading precisely: the number that had been flat for
the project's entire history moved by a path the census cannot see, which is also
why §10.21 has to add a fifth term to §10.19's factorization. §8.1 still measures
why the *ordinary* path is stuck, and the emit-set ceilings below still bound how
far widening alone can ever take it.

**Two sentences that stood in this paragraph until 2026-08-04 are RETRACTED, and
the retraction is the most important thing on this page.** It read *"the port is
byte-exact everywhere it accepts, it refuses everywhere else, and **no run has
ever recorded a mismatch**"*. Both halves are false.

* Board **#232** was a **live `Port=Mismatch` on master for 255 commits**
  (`d0d8a98..be86f9d`, two days) — the `26`-separator widening turned a clean
  refusal into a wrong emit, which is the one direction the correctness rule
  exists to forbid. It was found by `scripts/expr_sweep.sh`, which **the merge
  gate did not run**, so every lane gate and every coordinator re-gate in those
  255 commits came back green over a defect none of them could see. Closed at
  `be86f9d` by restoring the refusal — **not** by teaching the writer the shape,
  which is Phase 7 work.
* Board **#259** was **live when this paragraph was written and is CLOSED now**
  — corrected 2026-08-04, and *this page said "live on this tip right now" for
  two merges after the fix landed*, which is the same class of error as the two
  sentences retracted above, pointing the other way. `struct Bd{Bd();~Bd();int
  b0;}; struct M:Bd{M();~M();}; struct D:M{D();}; D::D(){} M::~M(){}` reproduced
  `Port=Mismatch @ offset 8`: the packed `.text` function order is not the `.ex`
  segment order and the port assumed it was. That TU's `.gl` contains no `0x26`
  byte, so it was older than #232 and unrelated to it. Lane `w-order` closed it
  at `bbef4bb` — **and found the filed row understated it by 6×**: a 47-TU grid
  gave **six** live mismatches, the smallest of them *strictly smaller* than the
  filed reproducer (`struct B{B();~B();int x;}; struct D:B{D();}; D::D(){}
  B::~B(){}` — four lines, no `0x26`, no implicit anything). The rule is a
  **dependency order** — a function is emitted only once every function it
  references *and defines* has been — established against three rivals each
  killed by its own probe, and it holds at `/O1` too, where it orders the COMDAT
  sections and therefore the section table, the section indices, the symbol
  indices and the `.pdata` association numbers.
* **Two more live wrong emits were found and closed the same day**, by lanes
  looking for something else: board **#263**, the `/EHsc` `eh-bare` label slot,
  **at the workload's own `/O1 /EHsc`**; and board **#276**, a TU that defines
  data and no functions emitted as the bare four-section shell — **older than
  both #232 and #263**, with *three* standing instruments green on it because
  none could represent the class.

`scripts/expr_sweep.sh` is now a **row of `scripts/gate.sh`**, re-derived and
counted like a lane, so the *class* cannot go unwatched again — but #259 said
plainly what that does and does not buy, and **it did not find #259, #263 or
#276**. Each of those three was found by a lane that built a probe grid for its
own rung. **The honest statement is that the port is byte-exact on every shape a
standing instrument grades, and the set of standing instruments is the whole
warranty.** Widening the instruments is the work; a green gate is a statement
about them, not about the port. **Board #283 is the standing measurement of how
wide they are not: 16 of 56 enumerated shape markers have ZERO cases in the
generated corpus**, `try`/`throw` among them — so the `/EHsc` axis is graded
entirely through implicit destructor unwind and never through a written `throw`.

**And read the generated block against the previous one before believing the day
was productive.** **Twelve merges** landed between tree `88e5ff6` and `26306ba`,
the tree the block below was collected at
(counted: `git rev-list --merges --count 88e5ff6..HEAD`). **Seven more have
landed since, and the same reading holds for them** — see the staleness banner
above the block, which gives their deltas and their sources. `c2rs gap` reads
**match 8, mismatch 0, codegen-gap 0, vocab-gap 863, capture-fail 7** — every
digit unchanged. Per-function census `706555/2463393`, emitted census
`38458/178975`, the emit-set ceilings `27` / `28` and `338 / 420 / 451`, the
`.gl` invariants — all unchanged. What moved: workspace tests 687 → **706**,
self-test 219 → **225**, the fixture gate 102/219 → **106/225**, geomean 567× →
568×. **Twelve merges moved the payoff metric by exactly zero, and that is the
expected result rather than a bad day** — eight of the twelve were commissioned
to *measure* (`w-emit`, `w-roots`, `w-refs`, `w-mark`, `w-skip`, `w-joint` and
`w-db` twice) and shipped no `crates/` behaviour at all. **#250 is the day's
sharpest finding precisely because it says a large move in a *leading* indicator
bought nothing**: the numbers to watch for over-reading are theirs, not this
block's.

---

## What each number is for

| number | it is | it is NOT |
|---|---|---|
| **TU match** (of 878) | **the payoff metric** — whole objs byte-exact at the workload's real flags | ~~a coverage percentage~~ |
| TU distance ≤1 / ≤10 / ≤100 | the leading indicator for TU match | a promise that the near ones are cheap |
| **emitted-function census** | in-class ∩ *code c2 actually emits* | gradeable by the differential on its own |
| per-function census | **a driver** — it ranks rungs, and does that superbly | the target. "census → 100 %" is **retired** (§8.1) |
| emit-set ceiling (28/871 gate-anchored) | TUs where `.ex` segments == obj COMDATs — the most TU match can reach **before** Phase 7 exists | reachable by widening |
| emit-set MODEL ceiling (338/871) | TUs where a segment-driven model binds every emitted symbol | the same thing as the line above (see below) |
| mismatch count | an **alarm** — and on **2026-08-04 it FIRED, four times over**: board **#232**, **#259** (a family of six), **#263** and **#276**. **All four are closed on `33cbdbe`.** Before that day it had never fired, and that record was doing more reassuring than it had earned | ~~"it has never fired"~~; and never evidence of correctness, before or after (see the coverage bound). **Nor is "four found and closed" a completeness claim** — three of the four were found by lanes building probe grids for unrelated rungs, so the rate says more about how many grids were built that day than about how many defects remain |
| **generated sweep** (`reached=N graded=G mismatches=M`) | enumerated small TUs graded against real `c2`, **part of the merge gate since 2026-08-04**. It read **14,484** when it found #232 and **14,817 reached / 14,721 graded** at `33cbdbe`; **quote it from the run, never from this page**. The instrument that found #232 and the only one that grades shapes nobody chose | a substitute for the workload — it enumerates axes somebody thought of, at one fixed profile (`/Ox /GS- /c`), and it generated **none of #259, #263 or #276**. **`reached` and `graded` differ by 96** — cases the *reference* rejects (board #281), which the driver counted as passes until board #280 separated the two counters |
| fixture gate | the port's accepted class, graded per fixture | representative of the workload's shape |
| perf geomean | the project's actual thesis — verifier throughput | comparable across versions. **Always quote it with its fixture count** (GAPS §1): the geomean is taken over the *matched* set, which grows as the port widens, so two geomeans are a change of population, not a regression. It is *also* wall-clock — 623×/653×/689× on three consecutive runs of one binary over the same 100 fixtures. Quote the order of magnitude with the count, never the digits alone. |

### The two ceilings are different things, and only one bounds TU match

* **Emit-set ceiling — 27 `LO`-anchored, 28 GATE-anchored** — TUs where the
  number of `.ex` function segments already equals the number of `.text` COMDATs
  in the real obj. For these, a port that lowered every body correctly would emit
  the right *set* of functions without modelling anything. **This is the hard
  bound on TU match until Phase 7 (the emit-set model) exists** — and **8 are
  already taken**, so every widening rung in the plan, summed, can move the
  payoff metric by at most **20 more TUs, ever** — and **19 of those 20 are
  reachable by codegen breadth alone** (`A∧B∧C` = **27** less the 8 matched; the
  other 1 of A's 28 fails B or C and needs section or binding work first). On the
  rest, the port emits one `.text` COMDAT per `.ex` segment and is wrong about
  the *set* regardless of how correctly it lowers each body.

  These were `6 / 22 / 16` until 2026-08-04, then `8 / 25 / 17`, and read
  **`8 / 27 / 19`** at `33cbdbe`. Two of A's 28 converted (§10.21) and the
  writer's section vocabulary grew twice — first w-r1c's three names, then
  w-sect's `.data`/`.bss` writer — which moved five more TUs inside C in total
  **while A itself never moved at all**. The bound's *structure* is unchanged;
  only its counts are, and they have now changed three times in one day, which is
  the argument for reading them off a scan rather than off this page.

  **Two numbers, because there are two splitters and only one is the port's**
  (§10.11, §10.15, §10.18). `LO`-anchored counts `.ex` segments on the `4C 4F 11`
  marker, which is what the *census* uses; the port consumes the `4F 1F` split.
  They disagree on **634 of 871 TUs**, unanimously in one direction — the gate
  sees more segments, never fewer, exactly the `??__E`/`??__F` population §10.12
  named. The ceiling still moved by only **1**, because it wants `segments ==
  COMDATs` *exactly* and extra segments mostly move a TU further from equality.
  **Quote the gate-anchored number**: it is the one `PortC2::build` has to
  satisfy. This bound was `25` and `"at most 19, ever"` for most of the project's
  life; both were an `LO`-anchored count of a `4F 1F`-anchored property.
* **Emit-set MODEL ceiling, 338 today / 420 repaired / 451 wall** — TUs where the
  `.gl` binding can account for every emitted symbol. This bounds *a model*, not
  today's port. It went 111 → 324 in §9.20 from a one-byte reader repair (both
  figures as recorded there; the key reads **338** today).
  §9.20 then claimed that gain was "unrealisable until the gate learns the same
  rule"; **W-ADOPT taught the gate that rule and the ceiling did not move**
  (§9.21). It is computed on `EmitBinding`, which already had the widened
  reader, so the gate was never the dependency. Realising it needs Phase 7 — an
  emit-set model — and nothing short of that.

Quoting 338 as "where we are" is the most likely misreading of this page. (It
was quoted as **324** across the front page until ROADMAP §10.20; the generated
block above has read 338 throughout, and the hand-written copies were stale.)

### And neither ceiling is the tightest constraint — the SECTION SHAPE is

§10.19 factored Phase 7 into four predicates over the 871 graded TUs and claimed
**A∧B∧C∧D was exactly the observed match set**, the same six files by name. **That
claim is REFUTED (§10.21): the conjunction is 6 and the differential grades 8.**
All four are printed by every `c2rs gap` run:

| factor | predicate | TUs |
|---|---|---:|
| A | `.ex` segments == `.text` COMDATs | 28 |
| B | every emitted symbol binds | **338** |
| **C** | **obj section set ⊆ what the port's COFF writer can emit** | **169** |
| D | every emitted COMDAT in the port's codegen class | 8 |
| E | at least one **registered** whole-TU recognizer accepts this bundle | 2 |

**D is no longer necessary for a match.** Factor D's proxy is the *per-function*
census verdict, which structurally cannot model a *whole-TU* emitter — so the two
`??__E` TUs are byte-exact in the obj and out of class in the census at the same
time, and the scan's known-answer control prints `A 0 B 0 C 0 **D 2**` with that
explanation next to the number. **It is left red on purpose**: the factorization
needs a **fifth term** for whole-TU emitters (board #179), and teaching the
per-function census a whole-TU fact would break the census/gate symmetry the
`census/gate disagreement: 0` line tracks. A red control that is understood and
documented is worth more than a green one that was adjusted to go green.

**C = 169 is still 2.00× tighter than B = 338.** The good news is that C is the
one factor that is **bounded**: this workload uses **13** section names, the
writer now emits **10** (`PORT_WRITER_SECTIONS`,
`crates/c2-core/src/coff/function.rs:32`), and **three** additions close it —
**`.rdata$r` 590**, `.text$yd` 804, `.xdata$x` **871**.

**`B∧C` = 151, re-measured over 871 graded TUs at tree `c303ad0`** (lane `w-bc`,
`rungs/2026-08-04-w-bc.md` §2, from a run that printed `capture-fail 7` /
`match 8`). It had been published as **107**, taken at `C = 114`, and no scan
had re-quoted it since the writer's vocabulary grew — flagged UNVERIFIED by
`w-book4` and **now verified at a different number**. `107` could not have been
extrapolated: `C` grows monotonically with the vocabulary, so the true answer
was forced into `[107, 169]` and any figure in that range would have been
consistent. The marginal is the readable part — **C gained 55 TUs and `B∧C`
gained 44 of them**, so 80 % of the vocabulary's new TUs were already
binding-complete and the inclusion rate `B∧C / C` fell `93.9 % → 89.3 %`.

> **⚠ And the projection built on it is TWO numbers, not one.** Board **#213**
> states *"what a perfect emit predicate is worth"* both as `B∧C − A∧B∧C` and as
> `frontier-if-A − FRONTIER`, and published a single `+82` because the two
> coincided at the time. **They do not coincide now: `151 − 27 = +124` of
> reachability, but only `141 − 19 = +122` of codegen frontier.** The difference
> is exactly the two TUs inside `B∧C` that fail A and that the port *already*
> accepts (`src/system/decomp_pch.cpp`, `src/system/math/vec.cpp`) — modelling
> the emit set would reach them without any codegen. Both are **reachability,
> not conversions**: a perfect factor A converts zero TUs by itself. `c2rs gap`
> now prints both, derives the subtraction itself, and names the divergence.

(C was 84 with a six-name writer and a seven-step ladder, 114 with nine names and
four steps; the step sizes below the top are not comparable across those changes,
because the ladder is greedy and re-ranks.)

**Two corrections you must not re-derive from §10.19** (ROADMAP **§10.20**):

* **`.rdata$r` is RTTI, not EH** — 24,163 content symbols, every one
  `??_R1..R4`, zero `__ehfuncinfo$`; it dies at `/GR-` and survives dropping
  `/EHsc`. EH's records land in **plain `.rdata`**, which the writer already
  has, so **Phase 5 moves C by zero**; rung three is an **RTTI** rung. EH blocks
  by factor **D**, over **740** objs, not 676.
* **13 is closed over this workload as measured, not closed by the language** —
  `#pragma init_seg("name")` mints a user-chosen name. Measured **0**
  occurrences in the workload's 78,746 source files (grep calibrated first), so
  13 holds empirically; re-run that grep before any new corpus inherits it.

**C is necessary, not sufficient** — reaching C = 871 converts nothing on its
own; only codegen converts. And **the pre-Phase-7 frontier is 19**: `A∧B∧C` = 27
with 8 already matched, so 19 graded TUs are reachable by codegen breadth alone
and the other 1 of A's 28 needs section or binding work first. `gap.rs` prints
those 19 by name each scan as the **FRONTIER**. Board **#160**.

**Every figure in this section is now also printed as a `gap-metric <key>
<value>` line at the end of each scan's factorization block** — `factor-c`,
`b-and-c`, `a-and-b-and-c`, `frontier`, `frontier-if-a`,
`emit-predicate-worth`, `ladder-head`, and the rest. That block exists because
C, `A∧B∧C` and the FRONTIER live only in hand-written prose here and **all three
went stale twice in one day**, and `B∧C` went stale by a *dependency* moving
under it with nothing able to notice. **`scripts/status.sh` does not consume
those keys yet** — the collector change is specified in
`rungs/2026-08-04-w-bc.md` §5.1 and is not made there. Until it lands, these
paragraphs are still hand-copied and still able to go stale; **quote them from a
scan, not from this page**.

**And the frontier is PRICED, which is the number to read next to it.** Lane
`w-conv` compiled all 17 (as it then stood) at the workload's own flags,
disassembled every code section and hand-counted the independent refusals per TU:
**the minimum over the seventeen is 6**, the cheapest *framed-and-branching* one
is **9**, and the standing decline clause — *a frontier TU at ≥ 4 independent
refusals is not a target* — **fires on all seventeen**. `negate_test.cpp`
re-derives at 9 by a different partition than w-cross's, which is the cross-check.
So `8 → 27` is real headroom **and there is no cheap TU left in it**: every step
costs ≥ 6 facts, and the counts are *lower bounds* because w-conv stopped counting
each row once the clause had fired. Board **#269**. **⚠ The pricing is
UNVERIFIED on the two newest frontier members** — it was hand-counted when the
frontier was 17 and the frontier is **19**; the same caveat `CFG_SHAPE.md`
already carries. "All seventeen" is not "all nineteen", and w-bc did not
re-derive it (it needs a disassembly pass per TU). (The dump that row cites,
`work/w-conv/frontier_dis.txt`, **was never committed** — the hand-count is in
`work/w-conv/PREREG.md` §1.1–§1.2 prose and reproduces via `work/w-frame/refobj.sh`
plus `scripts/gt_dump.py` per TU.)

---

## The traps

Each of these is a mistake the project has already made and paid for. They are
recorded here because the numbers above are individually true and jointly
misleading without them.

1. **`mismatch 0` is not evidence of correctness.** 863 of 878 TUs refuse before
   the emitter is consulted, so the scan *cannot see* a codegen or binding defect
   in them. Zero mismatches means "nothing the scan could grade came out wrong",
   over a population the scan mostly cannot grade. Verification here is
   coverage-bounded differential testing, and a green run is sound only on the IL
   it ran against. (ROADMAP **§7 / §10.8** — the bound has been restated
   independently fourteen times and is now an invariant. Do **not** cite this as
   "#149"; that number denotes the off-add argument slot.) **2026-08-04 supplied
   the demonstration this trap had only ever had in the abstract**: the workload
   scan read `mismatch 0` on every run for 255 commits while board #232 was a
   live wrong emit, because the scan cannot generate that shape. The number was
   true and the reassurance it carried was not.

2. **A per-function census claim for a never-emitted body can never be graded.**
   The differential compares whole objs, and an unemitted body is not in the obj.
   For those, "in class" is a *parser-only* claim with no byte behind it. The
   recorded precedent that this direction can be green-and-wrong is the `.sy`
   positional relaxation: census +2,981, mismatch 0, **wrong on 62 % of
   bindings**.

3. **A residue shrinking is not the thing the residue is a proxy for.** §9.20.3
   raised the `.gl` name-distance bound and watched `records_nameless` fall
   monotonically from 70 → 4 while **not one additional emitted symbol was
   covered**, and past a point it started handing one name to two records. A lane
   grading itself on the residue would have reported steady progress while
   covering nothing and corrupting the binding. (ROADMAP §9.20.3, §9.16.5. The
   prose calls this "#144's shape" as an **echo**; neither registered #144 is
   this rule — see [`BOARD.md`](BOARD.md) on bare-`#N` ambiguity.)

4. **Totality residue 0 is not a control.** `records == bound + residue` is
   satisfied exactly by moving a record from one bucket to another, so it cannot
   distinguish "we found a record" from "we found a name". The arity axis exists
   because of this: record *count* and record *offsets* are published and
   compared, and they were byte-identical (1,515,160) across a change that moved
   152,521 records between buckets.

5. **Absence reads as success unless something forbids it.** ROADMAP §9.18.8
   records this failure mode **twelve times**, and the newest instance was the
   *test runner itself* — a run reporting `ok` for every target with **169 tests
   silently not run**. Two others: a sweep that `sed`-ed a number out of a report
   and read the missing number as `0`, passing a run that graded literally
   nothing (§6s); and a lane registry whose four recorded lanes contained **no
   `/EH` at all** on a workload that is 100 % `/EHsc` (GAPS §7). This is why
   `gate.sh` renders from a registry, why `lanes.txt` is data, and why
   `status.sh` prints `NO-RESULT`. **The mitigation generalizes: compare a count,
   never a status.**

6. **The census names the callee, not the function, for any call-bearing body.**
   In the near-match tables, any row whose body makes a call is labelled with the
   *callee's* name. Known, unfixed. (GAPS §9.6.) **The blocker keys, the counts
   and both class axes are unaffected** — it is a labelling defect, so the
   rankings built on it stand.

7. **The mode caveat is resolved, but know that it existed.** Fixture numbers are
   captured at `/Ox`; the 878-TU workload compiles `/O1`. The port now reads the
   per-function optimization word and refuses anything unmodeled, `/O1` is a
   supported target, and `scripts/gate.sh` runs 12 enumerated lanes crossing the
   optimization and `/EHsc` axes. Numerator and denominator now speak the same
   modes.
8. **micro-F1 and per-TU exact are DECOUPLED — the leading indicator does not
   lead.** Six lanes in one day optimized the emit-set model's micro-F1. `w-db`
   moved code micro-F1 **0.85260 → 0.92655** — **+7.395 pp**, closing 47.4 % of
   the gap to the oracle ceiling — and **per-TU exact stayed at 132 of 850, name
   for name: zero gained and zero lost**, with TU match **8 → 8**. The mechanism
   is not subtle and is worth stating so it is not rediscovered: a whole-obj
   verdict is a **conjunction** over every symbol in that obj, and a micro-average
   is not, so a model can get much better on average without closing any single
   TU's *last* error. **Board #250.** Any rung sized off micro-F1 owes an argument
   for why it is not this case. It is trap 3's shape one level up — there a
   residue was not the thing it proxied for; here the proxy is a genuinely better
   model of the wrong quantity, which is harder to notice and was noticed only
   because the lane printed the per-TU set by name instead of by count.

   **UPDATE 2026-08-04, lane `w-emitp` — per-TU exact HAS now moved, and the
   trap's second half needs a correction.** The `.gl` stream carries a record
   class no model's node universe contained: **tag-0x10 ALIAS records**
   (`0x10b9c024` / `0x10b9c030`) — no `.ex` body, and a **token naming another
   symbol** in the word a tag-0x0E record uses for `flags4c`. A vftable's
   initializer names the **alias**; c2 emits the alias's **target**. Resolving
   `in`-stream nodes through it moves w-joint's ORACLE ceiling from per-TU exact
   **151 → 472 of 850** (micro-F1 0.97888 → 0.99243) and w-db's `JFP`, a model
   conditioning on no truth, from **132 → 308** — **0 TUs lost in either case**,
   and **4 592 of 4 592** added ORACLE predictions are emitted. Real `c2.dll`
   confirms it **15/15** with a **0/15** parity control. **TU match is still 8**:
   nothing shipped, `PortC2` has no emit-set model to put this in, and the spec
   is `rungs/_2026-08-04-w-emitp-findings.md` §6.

   **And the sizing method the project used for classes is wrong in the
   optimistic direction's opposite.** Removing `#152` from both sides of the
   ORACLE — the subtraction w-joint's U-i publishes — gives per-TU exact
   **287**. *Modelling* the same class gives **472**. **Class removal is a LOWER
   BOUND on the worth of modelling a class**, because a modelled name is also an
   **edge source**: the ORACLE's non-`#152` false negatives fall 2 750 → 1 768
   once `??_G` is live and its own reference list is followed. Do not price a
   class by subtracting it.

---

## Reproducing any of it

```sh
scripts/status.sh                 # everything below, in one pass
scripts/status.sh --check         # prove the collector, no toolchain needed
```

| what | command |
|---|---|
| workspace tests (portable) | `cargo test --workspace --release` |
| oracle self-test | `cargo run --release -p c2-harness --bin c2rs -- selftest` |
| fixture gate + speedup | `cargo run --release -p c2-harness --bin c2rs -- perf` |
| the 878-TU workload scan | `c2rs gap --list work/dc3-workload/files.txt --flags-file work/dc3-workload/flags.txt --cwd ../dc3-decomp --jobs 16` |
| regenerate the workload inputs | `scripts/gen_dc3_workload.sh <dc3-tree>` |
| **the merge gate** (12 mode lanes **+ the generated sweep + the mode cross**) | `scripts/gate.sh --jobs 8` — `12/12 PASS, 2,940 verdicts` at `33cbdbe` |
| the sweep alone | `scripts/expr_sweep.sh` (`C2RS_SWEEP_JOBS=8`; ~1 min 26 s, or 9 min 51 s serial). Reads `14817/14817 reached, 14721 graded, 0 mismatch` at `33cbdbe` — **`reached` and `graded` are different numbers and the gap is board #281** |
| the mode cross alone | `scripts/mode_cross.sh` — the generated corpus × the lane registry, `63,723 selected, 63,335 graded, 0 mismatch`; ~5 m 45 s cold, **13.8 s warm** on the capture cache (board #279) |
| cross-product lane | `scripts/cross_sweep.sh` |
| **board coverage** (no toolchain) | `scripts/board_audit.sh` — every `#N` `ROADMAP.md` cites that [`BOARD.md`](BOARD.md) has no row for |
| throughput vs concurrency | `c2rs perf-scale --csv docs/perf/perf_scale.csv` |

`status.sh` deliberately does **not** run the merge gate or the cross-product:
those answer *"is this tree safe to land"*, which is a different question from
*"where is this project"*. **Neither the sweep nor the mode cross is a separate
thing to remember any more — both are rows of `gate.sh`** (board #232, #279),
which is why the gate went from ~7 s to minutes and why that is the right price.
Run the gate before landing; run `status.sh` to report.

**Two open items about the gate's own cost and coverage, so they are not
rediscovered:** it grades the same `/Ox` case corpus **twice**, once uncached
through `c2rs diff` and once cached through `c2rs gap` — not the same check, since
only `diff` asserts the reference *replay* is byte-exact (board **#282**) — and
**16 of 56 enumerated shape markers have zero cases in the generated corpus at
all** (board **#283**, closure in flight).

**Run `status.sh` from the main repo, or set `C2RS_DC3`.** A worktree sits three
directories down, so the default `<repo>/../dc3-decomp` does not resolve from
one and **ten of the fifteen metrics come back `NO-RESULT`** — including TU
match, both censuses and every ceiling. The script says `STATUS: INCOMPLETE` and
refuses to call it a measurement, which is the mitigation working; but a block
regenerated that way and committed would look like a page whose numbers had
collapsed. It has never been committed in that state — checked, not assumed.

Everything except `cargo test` needs the toolchain (wibo + `compilers/`); all of
it degrades to `SKIP: toolchain absent` rather than failing.

---

## Where the code is

Two files hold the accept/refuse boundary, and a third exists only to stop them
from diverging:

| | |
|---|---|
| parse-time acceptance | `crates/c2-il/src/func/bundle.rs:699` — `IlBundle::functions()` |
| emit-time dispatch | `crates/c2-core/src/codegen/select.rs:127` — `select_function` (ordered match; **order is load-bearing**) |
| the anti-divergence check | `crates/c2-core/src/codegen/select.rs:210` — `function_gate`, why `census/gate disagreement` is 0 |
| gap-key rendering | `crates/c2-il/src/func/body/mod.rs:784` — `Block::feature()` |
| TU-level classes | `crates/c2-harness/src/gap.rs:74` — `TuClass` (`vocab-gap` = IL decode, `codegen-gap` = port refusal) |

Decode is **3.4× the emitter** by line count (`c2-il` 35.5k vs `c2-core` 10.4k)
and holds ~4× the tests. That is the physical signature of `vocab-gap 863`: the
port is not blocked on generating PowerPC, it is blocked on reading IL. The
largest single file in the project is the member-call decode
(`crates/c2-il/src/func/body/mcall.rs`, 4,643 lines), which is exactly the
`tail-recv-not-a-plain-b9-load/*` family at the top of the blocked histogram.

---

## Where to go from here

| question | doc |
|---|---|
| what is open, what was declined, what was refuted | [`BOARD.md`](BOARD.md) |
| the phase plan and why it is ordered that way | [`ROADMAP.md`](ROADMAP.md) §8 |
| what each blocker holds hostage, per rung | [`GAPS.md`](GAPS.md) |
| what landed, when, and for how much census | [`rungs/INDEX.md`](rungs/INDEX.md) |
| the correctness rule and the invariants | [`ROADMAP.md`](ROADMAP.md) §7, `../CLAUDE.md` |
| **what the CFG step must emit** — and why **15 of the 17** FRONTIER TUs need it (measured by `w-front` when the frontier was 17; it is **19** now and the two new members are ungraded on this axis) | [`CFG_SHAPE.md`](CFG_SHAPE.md) |
| **what the label counter charges, and the two channels it is NOT in** — `#286`/`#287` close "derive it from the blocks" | [`LABEL_COUNTER.md`](LABEL_COUNTER.md) §4.1 |
| **why `/Ox` and `/O1` differ in more than a register field** — the refutation, and the three reasons the `else` arm is out of reach | [`OPT_MODE.md`](OPT_MODE.md) §3.0 |
| the `.data`/`.bss` layout spec — allocator settled, walk order open | [`OBJ_DATA_BSS_SHAPE.md`](OBJ_DATA_BSS_SHAPE.md) |
