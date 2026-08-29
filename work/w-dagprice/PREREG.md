# `w-dagprice` — PREREGISTRATION

    Lane:    w-dagprice (wave 20, L4)
    Kind:    characterization
    Date:    2026-08-29
    Base:    c5bfe89d9 (master)
    Board:   #3838-#3844 (reserved for this lane)
    Brief:   docs/WAVE20_BRIEF_2026-08-29.md §2 "L4"

**Written and committed BEFORE the image was opened for this lane.** The only
things consulted first were repo documents (`P_DAG.md`, `SUBSYS.md`,
`P_REGALLOC.md` §7, `READ_PLAN_2026-08-21.md`, `WAVE20_BRIEF`, the rung docs)
and the output of `c2rs subsys`, none of which is a read of `c2.dll`.

---

## 0. What this lane is and is NOT

**IS:** settle-or-refute the `[dag]` band attribution; a ranked, addressed read
plan for `[dag]`; a price with its derivation printed beside it.

**IS NOT:** a scheduler, a register allocator, any `crates/` change, or a
rewrite of `P_DAG.md`'s findings. **Writes no `crates/` code** (brief §2 L4,
decision 20 §2). Predicted reach **0**; required byte delta **0**.

---

## 1. The claim under test, split into three

The brief and `SUBSYS.md`'s blind-spot box both say *"even attributing that band
to the scheduler is a hypothesis rather than a fact."* I register in advance
that I believe **that sentence conflates three separable claims**, and that they
have three different evidence standings. Registering the split before measuring,
so that agreeing with it later is not hindsight:

* **A — FUNCTIONAL.** The functions at `0x10be5cce`…`0x10be663f` implement a
  cycle-driven dependence-DAG list scheduler.
* **B — EXTENT.** The band's boundaries are `0x10be5cce` and `0x10be663f`, and
  the set of functions inside it is the set of scheduler functions (no scheduler
  function outside, no non-scheduler function inside).
* **C — TRANSLATION UNIT.** The band is *one* translation unit, distinct from
  `except.c` and `emit.cpp`, i.e. there exists a source file (a "`sched.c`")
  whose compiland is exactly this band.

**Registered prediction P1:** claim **A survives** — it is not a gap hypothesis
at all, because it rests on read function bodies and on measurements that never
consult `c2_tus.tsv`. Claim **C does not survive** as a fact and cannot be made
one by any read short of a positive TU-identifying artifact. Claim **B** is the
one that is actually load-bearing for a published number and is the one I expect
to be able to move.

**P2 (numeric).** The band contains exactly **13** Ghidra function entries
(`P_DAG.md:10-13` asserts `61 = 48 + 13`). I predict the re-derived count from
`functions.tsv` is 13. *If it is not 13, `P_DAG.md`'s coverage denominator 61 is
wrong and that is a finding in itself.*

**P3 (structure).** The band is call-closed from outside except at a small
number of entry points. I predict **at most 3 of the band's functions have any
caller outside the band**, and that `0x10be6382` (the scheduler driver) is one of
them.

**P4 (the number that matters).** `[dag]`'s published **`[O]` 7 of 50 (14.0 %)**
is a **mark census of `P_DAG.md`'s prose**, not a site census over the band. I
therefore predict it is **invariant under any re-attribution of the band**: if
the band were reassigned tomorrow the 7/50 would not move by one. If that holds,
then the brief's *"the load-bearing assumption under every `[dag]` number"* is
**false for the headline number** and true only for the `read` row's denominator
61 and the second denominator 83.

**P5 (edges).** The functions immediately below `0x10be5cce` and immediately
above `0x10be663f` are attributed by `c2_tus.tsv` to `except.c` and `emit.cpp`
respectively, and I predict **neither edge is witnessed by an ICE site adjacent
to the boundary** — i.e. the boundary addresses themselves are interpolated, not
observed. If an ICE site does sit within, say, 0x40 bytes of either edge, the
extent claim is much better supported than I expect and I will say so.

## 2. What would make my ranking an ARTIFACT — registered before it is built

`#3505` is **six for six**: every lane dispatched off a constructed ranking or
denominator found the ranking was an artifact. MEMORY's *"ranking instruments
measure themselves"* is four for four on the same shape. So:

**The artifact hypothesis for THIS lane, stated so it can fire:** *my ranking of
`[dag]` read targets is an artifact if the rank order is predicted by a property
of the binary (function byte size, band span, arm count, callee count, hop
distance) rather than by a named, cited downstream consumer.*

**The check I commit to running, and to publishing whichever way it goes:**

1. Every ranked row must name **a specific blocked claim with its citation** —
   a repo line that today says a thing is unknown, fitted, `[R]`-only, or
   ungraded. A row that cannot name one is deleted, not demoted.
2. After the ranking is fixed, compute **Spearman ρ between my rank and the
   candidate's byte size**, and between my rank and its citation count in
   `ADDR.tsv`. **I register `|ρ| ≥ 0.7` against size as the artifact
   threshold.** If it fires I publish the ranking *and* the finding that it is
   size-shaped, and I do not let a later wave dispatch off it unqualified.
3. I state the *denominator* of the ranking — how many candidate reads were
   considered and rejected — because a top-N with an unstated N is the shape
   `#3505` keeps catching.

## 3. The pricing rule I bind myself to — before any number exists

`#3603`: **R2, R5 and C1 each executed, each missed *pessimistic* by 30×–1,200×
on span read from git, and none of the three converted anything.** Two
directions, and a price that reports only one is not a price.

Registered in advance:

* Every published figure is a **PAIR**: *(construction span, conversion
  outcome)* — never a single scalar. `#3603`'s content is precisely that these
  two decoupled.
* Every figure prints its **derivation inline**, per `ROADMAP.md` §11.8: a
  figure whose inputs cannot be corrected cannot be re-priced, only withdrawn.
* **I will not restate `STEP5_PRICING_2026-08-21.md` §2.1's figures** (`#3370`'s
  mitigation — that block is canonical and is cited, never copied).
* The unit is **lane-days of read span**, calibrated against the *observed* span
  of already-executed reads in this repo (measured from git, not asserted), and
  the calibration's own sample size is printed.
* **I predict the calibrated multiplier will come out < 1** — i.e. that the
  repo's published read prices have been systematically *pessimistic*, in the
  direction `#3603` measured — and that the correct headline is therefore **not
  "the reads are cheap"** but *"read span is cheap and conversion is the
  unpriced term."*

## 4. What I refuse to conclude, whatever the reads say

1. **I will not upgrade claim C (one TU) to a fact on the strength of an
   absence.** "No ICE site" is a property of the instrument (`SUBSYS.md`'s own
   blind-spot box). Symmetrically, I will not conclude the band is *not* a TU
   from the same absence. Only a positive TU-identifying artifact settles it,
   and if none exists I say the claim is **unsettleable at this price** and name
   what would settle it.
2. **I will not claim any ranked read "unblocks F0/F5."** `P_REGALLOC.md` §7 as
   amended says F0 is **≥ 10 raw sub-lanes plus two UNPRICED terms** and that
   both published figures are floors. A read that removes one sub-item's
   unknown does not unblock the item, and saying so would repeat exactly the
   defect `w-f0price` found.
3. **I will not report a scheduler-model grade from this repo's corpus.**
   `#3435`/`#3728`: the final-schedule order channel is **8 tuple positions of
   3,015**, and a simulator returning its input scores ~99 %. Any read I rank
   must be priced *including* the fact that this corpus cannot grade its result.
4. **I will not report `built` if I produce no priced plan.** Per
   `CLAUDE.md` § "Units of work", a lane that produced none of its deliverable
   says **FAILED** in those words. `declined` is reserved for a lane that
   declined to convert a fixture, so a decline here rides in the rung's prose
   (brief §2 L4).
5. **I will not add a `gate.sh` row** (`#3691` — a 22nd makes
   `gate_identity_diff.sh` exit 2 for every lane).

## 5. Seam

Owns: `docs/whitebox/ref/P_DAG.md`, `docs/whitebox/WB_DAGPRICE_FINDINGS.md`,
`work/w-dagprice/**`, `docs/rungs/2026-08-29-w-dagprice.md`, board rows
**#3838**–**#3844** only.

Must not touch: `crates/**`, `docs/whitebox/ref/P_INLINE.md`,
`work/w-inlmetric/**`, `docs/whitebox/ref/P_GLOBREGS.md`,
`docs/whitebox/ref/P_REGALLOC.md`, `docs/STATUS.md`, `docs/rungs/INDEX.md`, any
other lane's board block.

## 6. Gate evidence owed

`scripts/gate.sh --jobs 16 --require-graded` (unqualified `GATE: PASS` expected —
the verdict LINE is read, never the exit code) and
`C2RS_REQUIRE_TOOLCHAIN=1 cargo test --workspace --release --no-fail-fast` with
**both the target count and the pass count**. Reach 0 and byte delta 0 are stated
here up front and shown to have held.
