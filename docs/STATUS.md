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

> ### ✔ RESOLVED 2026-08-05 — the block below was regenerated and is CURRENT
>
> The banner this replaces read *"THE BLOCK BELOW IS STALE — 7 merges and 38
> commits behind master"*, collected at tree `26306ba` against master `33cbdbe`,
> and it was raised by lane `w-book4` which **could not regenerate it**:
> `status.sh` needs `../dc3-decomp`, which does not resolve from a worktree.
> **It does resolve from one now** — `C2RS_DC3` is the documented override and
> the collector honours it, so a lane in a worktree can regenerate this block
> without going back to the main repo. Lane `w-fuzzy` did, and every one of the
> now-23 registered metrics produced a value.
>
> **The table below is kept as HISTORY, not as a live discrepancy.** Each row is
> the figure a landed rung measured while the block was stale; every one of them
> is now *in* the block above and can be read there instead. It stays because a
> record of which numbers went stale, by how much, and how long it took anyone to
> notice is worth more than the tidiness of deleting it — three of these went
> stale because a *dependency* moved, which is the failure mode `status.sh`'s
> registry exists to close.
>
> **What had moved, and where each figure was measured** — every one from a
> landed rung's §1 result table:
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
Collected 2026-08-10 · tree `94e119a0` · binary `85fb38b197be` · workload `104e7df9c`

| metric | value |
|---|---|
| Workspace tests (cargo test --workspace --release) | 1493 passed, 0 failed, 41 targets |
| Oracle self-test (c2rs selftest) | 369 PASS, 0 FAIL |
| Fixture port gate (c2rs perf) | 150 port Match, 0 mismatch, 219 not-implemented (of 369) |
| Port speedup, geomean over matched fixtures | 480x geomean over matched fixtures |
| 878-TU dc3 workload scan (c2rs gap) | match 23, mismatch 0, codegen-gap 0, vocab-gap 848, capture-fail 7 |
| Per-function census (driver, not target) | 714555/2463470 functions in class (29.01%) |
| Emitted-function census | 39253/162092 emitted functions in class (24.22%) |
| Emitted-census residue | residue 9220: 1961 compiler-generated (no IL body), 7259 unexplained  (5.69% of the denominator) |
| TU distance to match, blocked functions | ≤0: 16, ≤1: 20, ≤10: 27, ≤100: 34, ≤1000: 221 |
| TU distance to match, blocked emitted functions | ≤0: 17, ≤1: 27, ≤10: 91, ≤100: 460, ≤1000: 859 |
| Emit-set ceiling, LO-anchored (segments == COMDATs) | 27 of 871 graded TUs |
| Emit-set ceiling, GATE-anchored (4F 1F — what the port consumes) | 28 of 871 graded TUs |
| Emit-set MODEL ceiling (today / repaired / wall) | 338 today / 420 repaired / 451 wall |
| .gl binding invariants (records / arity / conflicts) | 1507159 records, 420 nameless, 0 before the first row, 39273 row-conflicts, 703 name-conflicts, 0 accounting breaks, 0 unreadable objs |
| Phase-7 factors over the graded TUs (A / B / C / D / E) | A 28 (LO 27) · B 338 · C 169 · D 23 · E 2, of 871 graded |
| Joint ceilings (B∧C, A∧B∧C) | B∧C 151 · A∧B∧C 27 · A∧B∧C∧D 21 |
| Pre-Phase-7 FRONTIER (codegen breadth alone / if A were free) | 4 reachable by codegen breadth alone; 126 if factor A were free |
| Emit-predicate worth, B∧C − A∧B∧C (board #213) | +124 TUs (B∧C − A∧B∧C) |
| Factor-C section ladder (writer names / workload names / next step) | 10 writer names of 13 workload names; 3 steps left, next +.rdata$r → C = 590 |
| PROGRESS MASS (driver, not target — docs/PROGRESS_METRIC.md) | P = 0.21410 · emitted in class 39253/162092 · mismatch-zeroed TUs 0 |
| FUNCTION BYTE MATCH (driver, not target — docs/FUNCTION_BYTE_MATCH.md) | FBM = 0.22094 · 35810 exact + 2 whole-TU of 162092 emitted functions, over 865 TUs (19 at 100%); 36342 are byte-exact before relocations are graded |
| FBM partition (the under-report, and the controls) | partial 10 (FBM under-reports by this) · differs 1898 · reloc-differs 532 · reloc-unknown 0 (UNGRADED residue) · refused 114622 · unbound 9220 · 3827 credited fns relocate, every record graded · controls: partition-broken 0, reloc-reach-broken 0, match-TU differs 0, match-TU reloc-differs 0, census disagree 1003 |
| Per-TU FBM (how close is the other 870) | 19 of 865 TUs with emitted functions are 100% byte-exact per function |

<!-- END GENERATED -->

---

## The one-paragraph answer

The **foundation is proven and fast**: standalone replay of the real `c2.dll` is
byte-exact on all 871 capturable TUs of a real Xbox 360 game, and every obj the
port has ever emitted matches — `mismatch` is 0 and has been through every gate
in this document.

> **⚠ 2026-08-06 — the clause that used to close that sentence, *"and the port is
> byte-exact on every shape a standing instrument grades"*, is RETRACTED.** Lane
> `w-fnbyte` widened FUNCTION BYTE MATCH to the four `/Gy` call shapes it had
> been declining (board #322) and **`fnbyte-differs` went 0 → 4,711**: of the
> 9,375 emitted functions the instrument could not previously see, **4,664 are
> byte-exact and 4,711 are not**, and **`framed` is 0 of 123**. A standing
> instrument grades those shapes now and the port is **not** byte-exact on them.
>
> **`mismatch` is still 0 and this is not a live wrong emit.**
> `IlBundle::functions()` refuses every TU carrying one of the 4,711, so none has
> ever reached an obj. What is wrong is the **emitted census's claim** — the
> PROGRESS MASS's `f` numerator — and the hazard is the *next* `functions()`
> widening, because every one of the 4,711 is already accepted by the
> per-function gate. Boards **#876**–**#879**;
> [`rungs/2026-08-06-w-fnbyte.md`](rungs/2026-08-06-w-fnbyte.md).
>
> This is the *third* time a sentence in this paragraph has been retracted by an
> instrument widening (see the two below), and the pattern is now the point:
> **every such retraction has come from widening an instrument, never from a
> gate going red.** A green gate is a statement about the instruments.
>
> > **2026-08-07 — 1,373 of the 4,711 are CLOSED, by the port and not by the
> > instrument.** Lane `w-empty` shipped **mechanism E** — c2 emits no branch,
> > no relocation and no external symbol for a tail call whose callee is defined
> > in the same TU with an empty body — as `crates/c2-core/src/elide.rs`.
> > `fnbyte-differs` **4,711 → 3,338**, `fnbyte-exact` **34,466 → 35,839**, and
> > **zero functions moved the other way** (checked per symbol, not by
> > subtracting totals). `fnbyte-elided 1373 / fnbyte-elided-exact 1373`: every
> > body the elision produced is byte-identical to real c2's.
> >
> > **`mismatch` is still 0 and `functions()` is untouched**, so the hazard the
> > paragraph above names is unchanged in kind and smaller by 1,373. Two things
> > worth carrying off that lane: **all 1,373 are one STLport template**
> > (`??1?$_STLP_alloc_proxy`, 545 instantiations — board #925, a coverage-bound
> > caution in its most concrete form yet), and the same rule keyed on the
> > *other* of a census row's two name bindings turned **14 byte-exact bodies
> > wrong and converted nothing** — `fnbyte-name-disagree` is **74,955** and is
> > printed on every scan now (board #918).
> > [`rungs/2026-08-07-w-empty.md`](rungs/2026-08-07-w-empty.md).
> >
> > > **2026-08-07 — 143 MORE ARE CLOSED, by taking the same mechanism to its
> > > FIXPOINT.** `w-empty` shipped the **one-step** rule and measured, on one
> > > cell, that c2 closes E under itself. Lane `w-fix` gridded that boundary —
> > > **34 cells, 94 call edges, 94 graded**, each compiled at the workload's
> > > flags and again at `/Ob0` and scored per *edge* — and shipped the closure:
> > > `fnbyte-differs` **3,338 → 3,195**, `fnbyte-exact` **35,839 → 35,982**,
> > > `fnbyte-elided` **1,373 → 1,516** with `-elided-exact` equal, **0**
> > > functions moved the other way, **72 of 80 `gap-metric` lines
> > > byte-identical**. `mismatch` 0 and `functions()` still untouched.
> > >
> > > **All 143 are `??1?$_Rb_tree_base@…`** — one template again, the STL tier
> > > directly above `w-empty`'s `_STLP_alloc_proxy` (board **#952**; #925's
> > > caution repeating rather than being retired). **The three things the grid
> > > found that one cell could not**: mechanism I mid-chain emits a bare `blr`
> > > at *every* level and is separated from E only by `/Ob0` (#954); a mid-node
> > > that keeps bytes drops its own call and does not let its caller drop one;
> > > a cycle is not E, and `void r(){r();}`'s self-branch takes no relocation
> > > at all, so the relocation observable reads `E` on a body that is plainly
> > > not nothing (#950). Boards **#946**–**#955**;
> > > [`rungs/2026-08-07-w-fix.md`](rungs/2026-08-07-w-fix.md).

The **payoff metric has moved for the first time**:
TU match is **10/878** (this paragraph read **8** until 2026-08-05 and the
generated block above is the source), up from a 6 that had held across a per-function census run
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
| **PROGRESS MASS** (`P = mean(a,b,c,f)`) | **a driver** — the *ranking* metric, and the only one that can say which of two lanes moved more on a day TU match read 8 before and after ([`PROGRESS_METRIC.md`](PROGRESS_METRIC.md)) | a completion percentage. `P = 0.21` does **not** mean 21 % done — the four terms are necessary, not sufficient. Its `f` term inherits trap 2 whole |
| **FUNCTION BYTE MATCH** (`FBM`) | **a driver** — the byte-exact differential asked *per emitted function* instead of per TU, so partial progress inside a TU is visible ([`FUNCTION_BYTE_MATCH.md`](FUNCTION_BYTE_MATCH.md)). The **only** continuous number on this page graded by the oracle's own bytes. **Quote it with `fnbyte-differs`, which was 0 until 2026-08-06, then 4,711, then 3,338, and is 3,195 since 2026-08-07 — quote it from a scan** | sufficient, and not a floor-free reading. A `.text` body is a *subset* of the obj, so `FBM = 1.0` would still not mean a matching TU. **The under-report it used to carry is CLOSED** — `fnbyte-partial` was 9,375 and is **0** (board #322, lane `w-fnbyte`); of that population **4,664 turned out byte-exact and 4,711 turned out WRONG**, so the widening bought +0.026 of ratio and one standing alarm that is no longer green by construction. `fnbyte-partial` is still printed, and prints `NONE` rather than vanishing. **And `exact` is not a clean credit: 861 of the 35,982 relocate against a symbol c2 does not name** (board #986 — a `/Gy` branch word cannot carry its callee, so FBM's byte test scores the word equal). `gap-metric fnbyte-calltarget-disagree-exact`, on every scan |
| emit-set ceiling (28/871 gate-anchored) | TUs where `.ex` segments == obj COMDATs — the most TU match can reach **before** Phase 7 exists | reachable by widening |
| emit-set MODEL ceiling (338/871) | TUs where a segment-driven model binds every emitted symbol | the same thing as the line above (see below) |
| mismatch count | an **alarm** — and on **2026-08-04 it FIRED, four times over**: board **#232**, **#259** (a family of six), **#263** and **#276**. **All four are closed on `33cbdbe`.** Before that day it had never fired, and that record was doing more reassuring than it had earned. **It fired a FIFTH time on 2026-08-08 — board #1148**, and that one is the sharpest of the five: lane `w-align16` was reading an alignment tag, cut two diagnostic cells for an unrelated reason, and **both graded `mismatch` on an unmodified master tree**. c2 places `.bss` *before* both `.XBLD$W` watermarks when the TU holds an internal-linkage object; Rule S1 puts it between them. **Closed fail-closed at zero match cost** (`8fa6b119`) — turned into an honest refusal, *not* into a guessed ordering, which is #174's three-cell rung | ~~"it has never fired"~~; and never evidence of correctness, before or after (see the coverage bound). **Nor is "five found and closed" a completeness claim** — four of the five were found by lanes building probe grids for *unrelated* rungs, so the rate says more about how many grids were built than about how many defects remain. **#1148 adds the sharpest version of that caution**: the shape was invisible because a fixture (`wsect_drop_static.cpp`) had **recorded it as unreachable**, and the route around that recording is *one line of C++* (`A* p = &g;`) nobody had written. A recorded unreachability is a statement about the cells someone thought of. The same scope error hits Rule Y1, whose static cells are TUs *with* functions while `emit_data_obj` serves *functionless* ones. **#1148's cells are byte-exact as of 2026-08-08** — lane `w-order3` derived the rule and #1152 is closed (board #1177); the count stays **five** because that lane fired no new one. **Two corrections to the sentence above, both from `w-order3`'s grid, and they cut in opposite directions.** (a) *"c2 places `.bss` before both watermarks when the TU holds an internal-linkage object"* is **too broad**: it holds when the static's first referrer is a `.data` initializer. When the first referrer is a **function body** the same section goes *after the code groups* — a **third** slot Rule S1 has no insertion point for, used by **109 of 871** workload objs (board #1179), and invisible for exactly the same reason: nobody had written `void f(){ g.a = 1; }` either. **The one-line-of-C++ lesson paid twice in the same file.** (b) It is also **not narrow enough about the trigger**: the rival it could not exclude — *"a `.data` relocation into `.bss` moves it"* — is dead, killed by `A g; A* p = &g;`, which has the relocation without the linkage and does not move. Rule Y1's static clause **is** wrong on functionless TUs (it is `.gl` order, not declaration order — board #1180) and was **not live**, because #1148's refusal had already fenced it |
| **generated sweep** (`reached=N graded=G mismatches=M`) | enumerated small TUs graded against real `c2`, **part of the merge gate since 2026-08-04**. It read **14,484** when it found #232 and **14,817 reached / 14,721 graded** at `33cbdbe`; **quote it from the run, never from this page**. The instrument that found #232 and the only one that grades shapes nobody chose | a substitute for the workload — it enumerates axes somebody thought of, at one fixed profile (`/Ox /GS- /c`), and it generated **none of #259, #263 or #276**. **`reached` and `graded` differ by 96** — cases the *reference* rejects (board #281), which the driver counted as passes until board #280 separated the two counters |
| fixture gate | the port's accepted class, graded per fixture | representative of the workload's shape |
| perf geomean | the project's actual thesis — verifier throughput | comparable across versions. **Always quote it with its fixture count** (GAPS §1): the geomean is taken over the *matched* set, which grows as the port widens, so two geomeans are a change of population, not a regression. It is *also* wall-clock — 623×/653×/689× on three consecutive runs of one binary over the same 100 fixtures. Quote the order of magnitude with the count, never the digits alone. **And it is LOAD-SENSITIVE, which is a third way to misread it** (2026-08-05): it read **674× → 481×** across two collections that changed no code it measures, because the second ran while three gates were saturating the box. A `status.sh` run is *itself* a heavy job — it runs `cargo test --workspace --release` and an 878-TU scan before it gets to `perf` — so this number is routinely taken under load. **Retake it on a quiet box before treating any move in it as signal**, and never rank two lanes by it. The *ratio* is the claim; the digits are a measurement of the box that day. |

### The two ceilings are different things, and only one bounds TU match

* **Emit-set ceiling — 27 `LO`-anchored, 28 GATE-anchored** — TUs where the
  number of `.ex` function segments already equals the number of `.text` COMDATs
  in the real obj. For these, a port that lowered every body correctly would emit
  the right *set* of functions without modelling anything. **This is the hard
  bound on TU match until Phase 7 (the emit-set model) exists** — and **8 are
  already taken**, so every widening rung in the plan, summed, can move the
  payoff metric by at most **18 more TUs, ever** (it read 20 when 8 were taken;
  **10** are taken now) — and **17 of those 18 are
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

  > **2026-08-07 — the ladder's top step has now been declined TWICE, by two
  > lanes, at two masters.** `w-rdata` priced the minimal `.rdata$r` obj at
  > **seven** independent refusals; **`w-rtti` was briefed to ship it anyway,
  > re-derived the price at `9827bcf`, and found all seven still unpaid** —
  > `c2rs census` reads `0/1 functions in class` at `expr-op-0x27`, and
  > `c2-il`'s `.gl` data-record reader returns **0 of the 6 data records** in
  > that obj against **2 of 2** on a `.data` control. `factor-c` is **169
  > before and after**, and all 77 `gap-metric` lines were byte-identical
  > across the lane. Two things did move: `OBJ_RDATA_R_SHAPE.md` is
  > **re-verified on 72 fresh objs (21 of 23 claims held, two did not)**, and
  > **board #301 is closed** — a `Section` literal in an emitter nothing calls
  > can no longer inflate C, measured by a counterfactual in which the *older*
  > guard stays green. Boards **#926**–**#933**;
  > [`rungs/2026-08-07-w-rtti.md`](rungs/2026-08-07-w-rtti.md).
* **13 is closed over this workload as measured, not closed by the language** —
  `#pragma init_seg("name")` mints a user-chosen name. Measured **0**
  occurrences in the workload's 78,746 source files (grep calibrated first), so
  13 holds empirically; re-run that grep before any new corpus inherits it.

**C is necessary, not sufficient** — reaching C = 871 converts nothing on its
own; only codegen converts. And **the pre-Phase-7 frontier is 8** (2026-08-09):
`A∧B∧C` = 27 with **19 already matched**, so 8 graded TUs are reachable by
codegen breadth alone and the other 1 of A's 28 needs section or binding work
first. `gap.rs` prints those 8 by name each scan as the **FRONTIER**. Board
**#160**. (This paragraph has now read 19-with-8, 17-with-10 and 16-with-11 as
conversions landed; it is corrected in place rather than annotated each time —
**quote it from a scan, not from this page**, which is the standing instruction
two sections up. `xboxheap.cpp`, discussed at length below, is in **neither**
end's frontier list as of board **#1792** — that block describes a TU the
instrument no longer ranks.)

> ### 2026-08-05 — **the frontier has had its first CFG conversion.** `src/system/math/Sort.cpp` matched at lane `w-hash` (board **#760**), and it is the **first TU ever converted that needed a control-flow class**: `?HashString@@YAHPBDH@Z` is an 80-byte pointer-walk loop with a back edge. What shipped is a **twenty-word transcription of one function class at `/O1`** — two immediate fields, no scheduler, no register allocator, no CFG builder — and **not** a loop lowering; `codegen::ptr_walk_loop`'s own module doc leads with that sentence, and `PORT_CFG_CLASSES` deliberately still does **not** list `cflow-loop` (board **#761**). The last blocker was not codegen at all: `.sy` admitted plain `int` automatics only, so the induction variable had no positive automatic-local test (board **#764**).

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

> ### ✔ 2026-08-08 — RE-PRICED, and *"there is no cheap TU left in it"* is REFUTED on one row. **The minimum over the seventeen is THREE.**
>
> Lane `w-front2` compiled all seventeen at the workload's own flags, applied
> board **#401**'s construct-ladder method rather than a hand-partition, and
> graded every cell against real `c2.dll` under wibo.
> **`src/xdk/nuispeech/xboxheap.cpp` prices at 3** — down from #401's 5 and from
> the `unpriceable` #270 gave it — because **two of #401's five are PAID and are
> shown paid by a byte-exact obj**: F1 (a literal store mixed into a run) and
> F4a/F4b (**the schedule**, reproduced at that TU's own six-store width). What
> is left is F2, F3 and board **#844**'s composition seam, and **#871's stated
> prerequisite for building that seam — #322 — has been closed** since it was
> written. The second-cheapest row is ≥ 7, so the clause holds everywhere else.
>
> **And the paragraph's frame is wrong, not just its number.** `codegen-gap` is
> **0 over all 878 TUs** and **all seventeen frontier TUs are `vocab-gap`** —
> not one has ever reached the emitter, though the FRONTIER is defined as
> *"reachable by codegen breadth alone"*. What paid the difference is six lanes
> from 2026-08-04/05 (`w-label`'s label map, #191's `encode_b_intra`,
> `w-hash`/`w-varloop`/`w-sched2`/`w-rotate`, `w-dclass`/`w-alloc`/`w-order2`,
> `div_mod_leaf`) and **zero** of the seven that landed since.
>
> `Primes.cpp` and `keygen_xbox.cpp` are priced for the first time, at **≥ 15**
> and **≥ 21**. The dump `#269` cites is still missing and
> **`work/w-front2/ref/*/dis.txt` replaces it, committed**. Boards
> **#1097**–**#1106**;
> [`rungs/2026-08-08-w-front2.md`](rungs/2026-08-08-w-front2.md).

> ### ✔ 2026-08-08 — RE-PRICED AGAIN, upward. **The minimum is FIVE, not three**, and the sub-target the re-price handed on is in the WRONG REGIME.
>
> Lane `w-heap` was sent to convert `xboxheap.cpp` on the strength of the block
> above and **declined at 5**, over a 27-cell grid frozen before a cell compiled
> and graded by real `c2.dll` under wibo at the workload's own flags.
> #1097's three stand; two more are **facts the re-price did not look at**:
>
> * **`codegen::alloc`'s mixed-kind refusal is LIVE on this exact body** — an
>   interior address at 2 uses beside a literal at 1. Board **#836** measured
>   that refusal wrong-on-0 over 81 cells and **#868** measured the narrow lift
>   that would open it and refused that too (`addi-interior` 12/12,
>   **`slwi` 0/12**, and `ProducerKind` cannot tell them apart). `alloc.rs`'s
>   own module doc ends *"a lane that widens the parser to admit an interior
>   address as a store value … inherits every paragraph above"*, and the block
>   above prices the codegen side at **1** without citing it.
> * **The reference bind is load-bearing at this width.** `xboxheap`'s ctor
>   written **without** `BE& listHead = mListHead;` is a **different body** —
>   both producers swap emission order and one store moves. `order::schedule`
>   predicts *both*, from the base symbol alone, so ORDER is genuinely paid; the
>   cost lands on the **reader**, which must carry the bound reference's own
>   token as the store's base symbol. Board **#1128**.
>
> **Two handover claims above do not hold.** `work/w-front2/ref/*/dis.txt` is
> **not** committed — that directory holds five `.cpp` copies and no
> disassembly — and neither is `work/w-front2/probe/x6/ref.obj`, which #1106
> cites. `work/w-heap/ref/xboxheap/dis.txt` is the target's own obj, committed.
> And **`x6` is not "a strictly smaller sub-target"**: its callee is a free
> function, so the argument setup writes `r3`, so the store base switches
> mid-run and the setup interleaves — board **#870**'s regime, which
> `xboxheap`'s own member call on `this` **avoids entirely** (empty setup, one
> base throughout). **Do not start at `x6`.** Boards **#1127**–**#1133**;
> [`rungs/2026-08-08-w-heap.md`](rungs/2026-08-08-w-heap.md).

---

## The traps

Each of these is a mistake the project has already made and paid for. They are
recorded here because the numbers above are individually true and jointly
misleading without them.

0. **A GREEN CONTROL IS A STATEMENT ABOUT THE POPULATION IT RAN OVER.** Numbered
   0 because it is trap 1 one level down, and 2026-08-07 supplied the cleanest
   instance the project has. The `.in` initializer reader's totality identity —
   `values + residue + conflicts == records` — counted `values` in **TOKENS** (a
   map key) and `records` in **RECORDS**, so two records carrying one token and
   the same bytes read `1 == 2`. It was **0 on every scan for the entire life of
   the file**, not because it was right but because the accepted population was
   43,113 scalar tokens and did not contain enough of the shape. Lane `w-tag02`
   read element tag `02`, the accepted population went to 496,135, and
   `in-init-accounting-broken` fired at **826 of 878 TUs** on the next scan. The
   identity was **repaired** (`records == accepted + residue`, both counts of
   records) rather than the control adjusted, and the duplicate population —
   **9,914** — is published beside it. Board **#937**;
   [`rungs/2026-08-07-w-tag02.md`](rungs/2026-08-07-w-tag02.md) §4. **The
   generalization: every green instrument on this page is green over the
   population it can reach, and widening the reach is how you find out which.**

   > **⚠ 2026-08-08 — THE REPAIRED IDENTITY WAS STILL SILENT ABOUT 43.7 % OF
   > THE STREAM, AND IT IS NOT ANY MORE.** `records == accepted + residue` is
   > *correct*, and it is a statement about the population the `00 01`/`00 02`
   > anchor scan reaches. A sequential parse of the same 850 streams framed
   > **879,377** records where the anchor scan counted **518,098**: **144,850**
   > were never anchored (their first element is a tag-`03` blob or a tag-`08`
   > fill) and **239,279** were dropped by the fail-closed `00 02` arm, and
   > **none of the 384,129 was in `records` OR in the residue** — invisible to
   > the totality control, the arity control and the residue histogram at the
   > same time. Board **#961**;
   > [`rungs/2026-08-08-w-emitp2.md`](rungs/2026-08-08-w-emitp2.md) §2.1.
   >
   > Lane `w-inread` **published the denominator rather than widening the
   > identity**: `in-init-unanchored`, `in-init-fail-closed` and
   > `in-init-no-token` now print on every scan beside `records`, counted
   > fail-closed and never folded in. Two things that only the printing could
   > have shown:
   >
   > * the fail-closed population fell **239,279 → 3,340** as a side effect of
   >   the same lane's reader widening (#960), so the number #961 was filed
   >   over was mostly a *symptom* of a different gap;
   > * the counter shipped **double-counting** — `no_token` read 3,340 beside
   >   `fail_closed` 3,340 on the first 878-TU run, one population wearing two
   >   labels — and it was caught only because the two happened to be equal
   >   (**#1002**).
   >
   > **The generalization one level further: a denominator is not published
   > until it has been printed on both sides of a change.** #996–#1005,
   > [`rungs/2026-08-08-w-inread.md`](rungs/2026-08-08-w-inread.md) §3.

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

   **2026-08-05 narrowed this trap without closing it.** FUNCTION BYTE MATCH
   ([`FUNCTION_BYTE_MATCH.md`](FUNCTION_BYTE_MATCH.md)) grades the port's
   *per-function* output against the reference obj's own COMDAT bytes, so the
   part of the emitted census that lands in an obj is no longer a parser-only
   claim: **29,084 of the 38,458 emitted in-class claims (75.6 %) are now graded
   by the oracle, and `fnbyte-differs` is 0.** The remaining 9,374 are shapes
   whose bodies the COFF emitter finishes and which the instrument declines to
   reconstruct (§3.1 there). The trap survives in full for the **2.28 M
   never-emitted bodies** — those are not in any obj and nothing can grade them —
   which is why the per-function census stays a driver and the emitted census
   stays the one the goal is written in.

   > **⚠ 2026-08-06 — THE UNGRADED REMAINDER WAS GRADED AND HALF OF IT IS
   > WRONG.** Lane `w-fnbyte` closed board #322: **100 % of the emitted in-class
   > population is now graded by the oracle**, and the split is **34,466 exact ·
   > 4,711 `fnbyte-differs` · 0 unexamined**. `framed` is **0 of 123**. The
   > paragraph above is kept as written because the sentence it licensed —
   > *"`fnbyte-differs` is 0"* — was quoted as evidence for a day and the
   > correction is the record. **`mismatch` is still 0 and has never moved**:
   > `IlBundle::functions()` refuses every TU carrying one of the 4,711, so what
   > is wrong is the *census's claim*, not an obj. The hazard is the next
   > `functions()` widening — every one of the 4,711 is already accepted by the
   > **per-function** gate. Boards **#876**–**#879**;
   > `rungs/2026-08-06-w-fnbyte.md`.
   >
   > **⚠ 2026-08-07 — the split is now `35,982 exact · 3,195 differs · 0
   > unexamined`.** Lane `w-empty` closed 1,373 of the 4,711 by shipping
   > **mechanism E** (`crates/c2-core/src/elide.rs`), with `functions()`
   > untouched and `mismatch` still 0 — boards **#916**–**#925**,
   > `rungs/2026-08-07-w-empty.md`. Lane `w-fix` then closed **143** more by
   > taking the same mechanism to its **fixpoint**, on 94 graded call edges —
   > boards **#946**–**#955**, `rungs/2026-08-07-w-fix.md`. **Quote
   > `fnbyte-differs` from a scan and not from this page — it has moved THREE
   > times in two days.**
   >
   > > **⚠ 2026-08-06 — `fnbyte-exact` IS NOT A CLEAN CREDIT EITHER: 861 of the
   > > 35,982 relocate against a symbol c2 does not name.** Lane `w-drop3` read
   > > the reference obj's **relocation targets** rather than its bytes
   > > (`c2_obj::ObjImage::text_comdat_call_targets`, board **#984**) and
   > > compared them with the port's own `REL24` list. Of 39,177 graded
   > > functions, **4,056 disagree: all 3,195 `differs`, and 861 `exact`.**
   > >
   > > **The cause is that a `/Gy` branch word cannot carry its callee** — c2
   > > writes the placeholder displacement `-(offset of the branch word)` for
   > > every target alike, so `??1?$list@H…`'s `48000000 → ?clear@…` and the
   > > port's `48000000 → ??1?$_List_base@H…` are the same four bytes. Board
   > > **#882** ("4,664 credited functions carry a relocation FBM does not
   > > check") was that gap as a caveat; **861** is the part of it that is
   > > wrong.
   > >
   > > **`mismatch` is still 0 and `functions()` is untouched** — all 861 sit in
   > > refused TUs, so what is wrong is the credit, not an obj, and the hazard is
   > > the next `functions()` widening. **This is the FOURTH time an instrument
   > > widening has retracted a claim on this page and the first time none of the
   > > four came from a gate**; it also **refuted a published board row**, #979's
   > > "the port omits a call c2 makes", which was a byte test misreading a
   > > substitution as a deletion. Boards **#984**–**#989**;
   > > [`rungs/2026-08-06-w-drop3.md`](rungs/2026-08-06-w-drop3.md).

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

   > **⚠ 2026-08-08 — THAT SPEC'S §6 IS THREE-FIFTHS LANDED AND TWO-FIFTHS
   > REFUTED, and the refuted part is the part a lane was about to build.**
   > Steps 1, 2 and 5 shipped at `d2bdadc` (lane `w-alias`:
   > `crates/c2-il/src/func/glalias.rs`, `DISCLOSURE.md` rows W-ALIAS-1 /
   > W-ALIAS-2). Steps 3 and 4 say *resolve the alias at the `in` `02`-node
   > site* and *never emit a name in `dom(alias)`* — and the only such site in
   > `crates/` is `IlBundle::data_tu`'s **relocation naming**, where **both
   > read false**. Real `c2.dll` leaves **4,248 relocations naming `??_E<X>`
   > unresolved over 675 of 871 objs** and realises the alias as a COFF
   > **`WEAK_EXTERNAL`** symbol record instead. So `dom(alias) ∩ E = 0` is a
   > statement about COMDAT **leaders**, and c2 *does* write a symbol record
   > carrying the alias's name.
   >
   > **The channel's real observable grades the DECODE PER RECORD, and it is
   > exact**: `alias-weak-predicted` **4,013 / 4,013**, `-default-disagree`
   > **0**, `-unpredicted` **0**. And the realisation rule — *c2 writes
   > `??_E<X> → ??_G<X>` iff `??_G<X>` is a `.text` COMDAT leader of the same
   > obj* — reads `alias-rule-miss 0` / `alias-rule-extra 0`, **exact on 871 of
   > 871**. That is a statement about the decode; the per-TU-exact figures in
   > this trap's paragraph are statements about a *model*, and the two are not
   > substitutes.
   >
   > **What it is worth is measured, not extrapolated**: `alias-weak-needed-tus`
   > **675 of 871**, `alias-weak-needed-in-b-and-c` **0**,
   > `alias-weak-needed-in-frontier` **0** — the port's COFF writer has no
   > weak-external record, so 675 TUs carry a symbol-table requirement it
   > cannot meet, **no factor in §10.19 represents it**, and it costs the
   > payoff metric nothing on this population. TU match **11 → 11**, mismatch
   > **0**, and all 199 pre-existing `gap-metric` keys byte-identical.
   > Boards **#1500**–**#1509**;
   > [`rungs/2026-08-08-w-phase7.md`](rungs/2026-08-08-w-phase7.md).

   **And the sizing method the project used for classes is wrong in the
   optimistic direction's opposite.** Removing `#152` from both sides of the
   ORACLE — the subtraction w-joint's U-i publishes — gives per-TU exact
   **287**. *Modelling* the same class gives **472**. **Class removal is a LOWER
   BOUND on the worth of modelling a class**, because a modelled name is also an
   **edge source**: the ORACLE's non-`#152` false negatives fall 2 750 → 1 768
   once `??_G` is live and its own reference list is followed. Do not price a
   class by subtracting it.


**A CODEGEN PRICE ON THIS PAGE IS A HAND-COUNT UNLESS IT NAMES A KEY.** Lane
`w-ladders` proved (board **#1464**) that the frontier ladder instrument has no
codegen column and never had one: `fn_blockers` and `emit_blockers` are the same
reader column at two populations, and `fn_gate_refusals` is an invariant defined
to be **0**. Every codegen number this project has published — #1105's `>= 15`,
#1418's 776 bytes, #770's eleven — was therefore produced by a person reading an
obj beside an IL body, and none of them moves when the tree does.

There **is** a codegen column now (boards **#1473**/**#1474**) and it is small on
purpose. On the 16 frontier TUs it reads, over **59** emitted functions:

| bucket | reads | what it means |
|---|---:|---|
| `frontier-codegen-exact` | **10** | c2's bytes, produced |
| `frontier-codegen-wrong` | **1** | the reader accepted, the port lowered it, the judge says the bytes differ — **the only instrument-read codegen price on the frontier** |
| `frontier-codegen-refused` | **0** | the reader accepted, the emitter declined. Structurally near-empty: three of its four stages are zero *by construction* while acceptance lives in the IL parser (#1475) |
| `frontier-codegen-reader` | **48** | **the IL parser refused, so no codegen question was asked and none CAN be.** 81 % of the frontier |

**Read the last row first.** `frontier-codegen-measured` is a **lower bound of
unknown tightness**, not a price: the frontier's true codegen distance is
`wrong + refused` plus an unknown amount hiding behind those 48. A reader who
takes `1` as the frontier's codegen cost has made `cflow-emitted-modeled`'s "718"
mistake (#1343/#1344) with a smaller number. Equally, **`0` refusals is not
"codegen is done"** — it is an alarm that did not fire.

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
| **the merge gate** (18 mode lanes **+ the generated sweep + the mode cross**) | `scripts/gate.sh --require-graded` — the default `--jobs` is **16** since 2026-08-08 (it was 4, unchanged since the file was written; lane `w-throughput`, board #1323), and `--jobs` still overrides. Reads **18/18 PASS, 0 FAIL, 0 SKIP, 0 NO-RESULT**, 5,184 fixture-verdicts, sweep `19556/19556 reached, 19460 graded, 0 mismatch`, cross `90812/90812 reached, 90424 graded, 0 mismatch` at `f49fe5e1`+`wt-w-throughput` — **quote it from the run, not from this page** |
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
| **what the CFG step must emit** — and why **15 of the 17** FRONTIER TUs need it (measured by `w-front` when the frontier was 17; it went to **19**, and is **17** again since `Sort.cpp` converted — the members are not the same 17 and the newer ones are ungraded on this axis) | [`CFG_SHAPE.md`](CFG_SHAPE.md) |
| **what the label counter charges, and the two channels it is NOT in** — `#286`/`#287` close "derive it from the blocks" | [`LABEL_COUNTER.md`](LABEL_COUNTER.md) §4.1 |
| **why `/Ox` and `/O1` differ in more than a register field** — the refutation, and the three reasons the `else` arm is out of reach | [`OPT_MODE.md`](OPT_MODE.md) §3.0 |
| the `.data`/`.bss` layout spec — allocator settled, walk order open | [`OBJ_DATA_BSS_SHAPE.md`](OBJ_DATA_BSS_SHAPE.md) |
| **what is INSIDE the 3,195 `fnbyte-differs` bodies** — the cluster table, and the answer to "is this a register/schedule problem". It is not: **100 % of the port's differing bodies make a call and 78.9 % of c2's counterparts make none**, 5,173 of 5,189 substituted words differ in their *opcode*, **0** are pure reorderings, and **0** fail to decode. Two smaller targets fall out (370 bodies and 140) | [`DIFF_STRUCTURE.md`](DIFF_STRUCTURE.md) |
| **why c2 does not emit a call the IL contains — and why that is TWO mechanisms, only one of them a cost model.** **Mechanism E is SHIPPED, and so is its FIXPOINT** (`crates/c2-core/src/elide.rs`, 2026-08-07): 1,373 + **143** of the 4,711 closed, `fnbyte-differs 4,711 → 3,338 → 3,195`, zero regressions at either step. Mechanism I is not, and holds at **0.9716** on a 100-TU frozen workload hold-out. Read §1.2 before reusing E's rule — the page's own §1 is refuted there, and §1.2 now carries the **six places the chain stops**, three of which one cell could not have shown | [`INLINE_PREDICATE.md`](INLINE_PREDICATE.md) |
