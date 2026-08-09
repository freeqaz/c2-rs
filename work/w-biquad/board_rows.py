#!/usr/bin/env python3
"""w-biquad — append this lane's board rows to `docs/BOARD.md`.

Kept as a script rather than done by hand because the rows are long and the
column order is load-bearing (`# | item | verdict | number | where | notes`),
and because two lanes this week had master renumber their rows underneath them.
Run once; idempotent by refusing when `#2530` is already present.
"""
import sys

ROWS = [
    (
        "2530",
        "**`Biquad.cpp` CONVERTS — TU MATCH 20 → 21 — AND `w-park` DECLINED IT AT FIFTEEN**",
        "**PAID.** Two body classes ship: `fp_store_diamond` (35 words — a null-guarded `if`/`else` whose arms are float member stores, with a CSE'd division run) and `ctor_forward_call` (9 words). Both bodies byte-exact and the whole obj byte-exact: 9 sections, 29 symbols, 10 relocations. `w-band` (#2242) had independently confirmed this TU strictly deeper than `mmio.cpp`, inverting the byte-fraction ranking",
        "**20 → 21** · mismatch 0 · `fnbyte-exact` **+2** · FRONTIER 7 → 6",
        "rungs/2026-08-09-w-biquad.md §1 · #1923 · #2242",
        "**Exactly ONE per-TU verdict moved over 878 TUs, toward acceptance.** Both classes are TRANSCRIPTIONS of one TU's two functions, on `ptr_walk_loop`'s precedent — accepting them is not a claim about `cflow-if-1` as a class",
    ),
    (
        "2531",
        "**`expr-op-0x27` IS A DISPATCH FACT, NOT A GRAMMAR HOLE — AND CONVERTING ITS HEADLINE TU MOVED IT BY ZERO**",
        "**MEASURED, to the unit.** The key is `parse_expr`'s FALL-THROUGH arm: it says *\"no recognizer claimed this body\"*, which is a statement about the shape ladder and not about the operand stream. `designator::walk_offset_adds` has consumed `27` and `28` since `w-34`; both classes shipped here read every designator through it and **added no grammar**. Base and tip both read **22,409 emitted · 844 TUs · 403,879 bodies**",
        "22,409 → **22,409** · 844 → **844** · 403,879 → **403,879**",
        "rungs/2026-08-09-w-biquad.md §3 · whitebox/WB_READER_FINDINGS.md §3.3 · #2282",
        "**Three prior readings confirmed from the other side.** `w-readpx` priced the grammar cost NONE and the TU delta 0; `w-dclass` §6.1 measured six functions and zero TUs; `w-band` read `NoSignal`. A lane that CONVERTED the TU the key was supposed to be about is the strongest form of that evidence — and `Biquad.cpp` was never in the population, its first blocker being `expr-cmp-eq`",
    ),
    (
        "2532",
        "**THE LAST THING IN THE WAY WAS A LABEL SURCHARGE THAT HAS BEEN UNOBSERVABLE SINCE THE PORT FIRST POOLED A CONSTANT**",
        "**FOUND BY THE DIFFERENTIAL**, with both emitters finished and both bodies byte-exact. `??0Biquad`'s triple came out `$M2570/$M2571/$T2572` against c2's `$M2574/$M2575/$T2576` — **four low, which is 2 + 2**: `LABEL_COUNTER` §1.1's *\"a newly pooled FP constant … +2\"*, absent from `plan_labels`. Absent HARMLESSLY — only a framed function has labels, so a leaf's surcharge is invisible unless a framed function follows it, and every pool-bearing obj the port had emitted (`w13b_fconst`, `w13b_fdedup`, `w13b_fpool`) is leaves alone",
        "4 slots · 2 + 2 · 3 prior objs blind",
        "rungs/2026-08-09-w-biquad.md §5 · docs/LABEL_COUNTER.md §1.1 · #764 · #2301",
        "**CEILING §11.4 paying a fourth time** — the last blocker was a TU-level fact and not an instruction, after `w-hash`'s `.sy`, half of `w-bdnz`'s, and `w-blockir`'s `_fltused`. The charge is confirmed THREE ways: the obj, §7.6's in-the-middle stride (predicted 6, measured 6), and a must-fail mutation",
    ),
    (
        "2533",
        "**THIS LANE INTRODUCED A LIVE WRONG-BYTES EMIT, AND EXACTLY ONE INSTRUMENT WAS LOOKING**",
        "**CAUGHT AND FIXED.** Making `FpConstRef::lo_off` a field left `PortC2::build`'s packed rebasing at `hi_off: r.hi_off + off, ..r` — COMPLETE while `lo_off` did not exist, silently wrong the moment it did. `w13b_fdedup.cpp` at `/Ox`: **`Port=Mismatch @ offset 760`**, the fifth relocation record of the shared `.text`. It reaches every packed obj with a pooled constant in a function that is not FIRST",
        "1 fixture · 1 mode · offset 760",
        "rungs/2026-08-09-w-biquad.md §1 · #2475",
        "**The 878-TU scan is `/O1`-only and never reaches the packed writer; the `/O1` fixture lane was clean; 1,430 workspace tests passed.** The fixture-level neutrality scan at BOTH modes, compared by name, was the only thing that could see it — and the full gate's four `/Ox` lanes then reproduced it independently. `w-fence2` #2475's shape one field along: admitting a new field is two changes and the second is invisible from the first",
    ),
    (
        "2534",
        "**FOUR MUST-FAIL MUTATIONS, FOUR `mismatch`ES — AND B-RULE'S RIVAL IS NOT A STRAWMAN**",
        "**RUN, not reasoned.** Deleting the pool surcharge, dropping B′-RULE's flip, hoisting BOTH `lis` into the entry block, and parking `this` in r11 each turn `wbiquad_fp_store_diamond.cpp` from `match` into a live `mismatch` against real `c2.dll` through `scripts/mode_lane.sh /O1`. Each is a `mismatch` and not a refusal, so each proves the fence is carrying a BYTE",
        "4 of 4 · `match=163 → 162` each",
        "rungs/2026-08-09-w-biquad.md §6 · work/w-biquad/MUTATIONS.md · #2305",
        "**M3 is the one to read twice.** Its rival is exactly what `Biquad.cpp`'s own obj invites — both readings put a `lis` at word 0 and disagree only at word 4 — and `WB_CHOOSER_FINDINGS`' cell **B1** is the only thing separating them. This lane depended on a cell it did not compile, and the mutation is what makes that dependence visible",
    ),
    (
        "2535",
        "**SEVEN OF ELEVEN `_neg` CELLS REFUSED ON SOURCE FORMATTING, AND THE PROBE IS WHAT SAID SO**",
        "**CAUGHT.** The first draft read seven cells at `fpdiamond-then-close-4` — a `4F` LINE MARKER — because the recognizer skipped a marker once before a scope-close pair, which fits `Biquad.cpp`'s brace-per-line formatting exactly and refuses a semantically identical body written on one line. Post-fix: **ten cells, ten distinct clauses**",
        "7 of 11 confounded → **10 distinct**",
        "rungs/2026-08-09-w-biquad.md §7 · work/w-biquad/NEG_CLAUSES.md · #1704",
        "**The confound six of the last nine lanes paid for, caught by RUNNING the probe rather than by reading the cells.** The fix moved the marker skip inside `eat_close`/`eat_label`/`eat_transfer` — where `eat_return_head` has always had it — so the class is strictly WIDER for it and no accepted body moved",
    ),
    (
        "2536",
        "**THE FORWARDING CONSTRUCTOR'S PARK IS A FACT ABOUT ITS CALLEE, AND THE PORT NOW SAYS SO OUT LOUD**",
        "**MEASURED, both sides compiled.** `mr r10,r3` with NO restore is M-RULE's volatile branch. `work/w-biquad/probe/park_extern.cpp` — the SAME constructor over an undefined external — is **48 bytes**: `std r31`, `mr r31,r3`, `mr r3,r31`, `ld r31`. `park_local.cpp`, with a SMALL same-TU callee, is **12 bytes**: c2 inlines it entirely. So the class is not \"a constructor forwarding a call\"; it is that AND a statement about the callee",
        "36 B vs 48 B vs 12 B",
        "rungs/2026-08-09-w-biquad.md §4.2 · whitebox/WB_CHOOSER_FINDINGS.md §2.3",
        "**The gate therefore lives in `comdat`, not in the parser** — the only layer where the callee's own lowering exists — and it admits exactly the classes whose GPR footprint the port has STATED (today one, `fp_store_diamond::GPR_FOOTPRINT`). A port that guessed would be right about eight of the nine words",
    ),
    (
        "2537",
        "**`emit_comdat_obj` EMITS `.rdata` CONSTANT POOLS — INTERLEAVED, LIFO, AND DEFINED**",
        "**SHIPPED**, under `OBJ_GY_SHAPES` §2.4's three rules: a pool's COMDAT and symbol pair go immediately after the `.text` of the function that FIRST references it; one section per distinct `(bits, width)` TU-wide; and several constants from ONE function are appended in **reverse** first-reference order. Plus two repairs — `FpConstRef::lo_off` becomes a field (B-RULE separates one pool's halves by five words) and the `__real@…` record is emitted **in its own section**, where it was section 0",
        "2 sections · 6 symbols · 8 relocations",
        "rungs/2026-08-09-w-biquad.md §4.3 · docs/OBJ_GY_SHAPES.md §2.4",
        "**The section-0 bug is the quiet kind**: an undefined `__real@…` links perfectly well against another TU's copy of the same constant, so nothing but a byte compare can see it. The refusal it replaces had stood since W13b with a comment naming the exact rule it could not express",
    ),
    (
        "2538",
        "**THE READER IS DELIBERATELY WIDER THAN THE EMITTER, AND IT COSTS TEN NAMED DECLINES**",
        "**MEASURED.** `fnbyte-partial` **0 → 10** and `fnbyte-decline-gy-shape` **0 → 10**: ten workload constructors have `ctor_forward_call`'s shape and a callee whose GPR footprint the port cannot state, so they decline BY NAME in `comdat` rather than vanishing into a parse failure that names the wrong construct",
        "10 functions · 0 TUs · 0 parser-expressible",
        "rungs/2026-08-09-w-biquad.md §4.3 · #139",
        "**The census/gate PARSER-EXPRESSIBLE count stays at 0**, which is board #139's target and not an accident: the fact the gate needs is a property of a DIFFERENT function, so no parser clause could ever reach it. `store_run_call`'s stated discipline, applied to a cross-function fact",
    ),
    (
        "2539",
        "**`LABEL_COUNTER` §7.6's PROCEDURE, RUN AS WRITTEN, AND IT PREDICTED THE STRIDE BEFORE THE COMPILE**",
        "**HIT, 6 predicted and 6 measured.** Subject in the middle (`a0 · P · a1 · a2`), never the counterfactual form; `base = first(a2) − first(a1) = 5` under `/Gy`, so step 3's validity check holds. `first(a1) − first(a0) = 11 = 5 + 6`, and `6 = 1 (leaf) + 1 (_fltused) + 2 + 2 (two new pools)`",
        "base **5** · stride **11** · P **6**",
        "rungs/2026-08-09-w-biquad.md §5 · docs/LABEL_COUNTER.md §7.6 · #2430",
        "**The first lane to use §7.6 and have the table be RIGHT.** Four consecutive lanes measured `LABEL_COUNTER` wrong; the difference is that this charge is a SURCHARGE row — §7.5's *\"the minting population is computable\"* — and not a control-flow lead. `label_slots` gains no arm and neither class gains a `label_lead`",
    ),
    (
        "2540",
        "**THE `.rdata` LIFO ORDER WAS ALREADY MEASURED, WITH A BETTER CELL THAN THIS LANE'S**",
        "**RE-CONFIRMED, and DOWNGRADED in the same breath.** The PREREG registered reverse-first-reference order as this lane's own finding on two compiled cells. `OBJ_GY_SHAPES` §2.3 — found afterwards, in a comment inside the very refusal being lifted — already had it, and with a **three-constant** cell that separates LIFO from descending bit-pattern order where no two-constant cell can",
        "2 cells added · 1 prior cell stronger",
        "rungs/2026-08-09-w-biquad.md §4.3 · docs/OBJ_GY_SHAPES.md §2.3 · #2360",
        "**Recorded as a demotion of the lane's own prediction rather than banked as a hit.** The lane read every doc the commission named and still re-derived a measurement sitting in the file it was about to edit — which is what a characterization in a `.md` and a refusal in a code comment cost between them",
    ),
    (
        "2541",
        "**B-RULE-2 SHIPS IN NOTHING, BY DECISION, AND THE DECISION IS THE ROW**",
        "**DECLINED.** The compare/branch separation slot is `medium` at exactly THREE witnesses (`WB_CHOOSER_FINDINGS` §3.3, counted by script after a hand count got both numbers wrong). The entry block's word order is transcribed from this class's own obj and no clause in the reader, the emitter or the writer asks a separation question",
        "3 witnesses · 0 clauses · 1 decline",
        "rungs/2026-08-09-w-biquad.md §4.1 · whitebox/WB_CHOOSER_FINDINGS.md §3.3 · #260",
        "**PREREG clause D6 honoured rather than tested.** #260's warning is about a clause with exactly this history, and the cheapest way to raise the rule — cells that hoist two or more words into a block with a compare in it — is named in `WB_CHOOSER` and was not taken here",
    ),
    (
        "2542",
        "**OF `w-park`'s FIFTEEN, THIRTEEN ARE PAID, ONE IS DECLINED, AND TWO WERE A MISREADING**",
        "**RE-DERIVED, group by group.** Nine reader rungs are paid inside the two recognizers' own grammar; B-RULE, B′-RULE and M-RULE are paid; B-RULE-2 is declined by name (#2541); and **the two designator opcodes were never owed** — implemented since `w-34` (#2531). `w-park`'s six-cell ladder `lad_bq.cpp` was 0 of 6 in class and every cell's construct is in class at this tip",
        "9 + 3 paid · 1 declined · 2 never owed",
        "rungs/2026-08-09-w-biquad.md §11 · #1923",
        "**The misreading is the finding, not the arithmetic.** A price of fifteen containing two items the port already had is a price read off a census key rather than off the port — the failure mode #1760 / #1782 / #2360 keep recording in a different register",
    ),
    (
        "2543",
        "**`mmio.cpp`'s ELEVEN ARE UNTOUCHED, AND THE FRONTIER'S HEAD IS A TU THIS LANE DID NOT LOOK AT**",
        "**STATED.** `mmio.cpp` stays at 256/380 accepted bytes (67.4 %) with one blocked emitted function, and is the frontier's head by byte fraction at both ends. Nothing this lane ships reaches it: it has no pooled constant, no float and no CSE run, and `w-blockir` §1 priced it at eleven distinct unbuilt mechanisms",
        "6 frontier TUs · 1 blocked fn · 124 B remain",
        "rungs/2026-08-09-w-biquad.md §10 · rungs/2026-08-09-w-blockir.md §1",
        "**Named so a later lane reading `FRONTIER 7 → 6` does not have to work out which one left.** The TU that left is `Biquad.cpp`, and it left by MATCHING",
    ),
    (
        "2544",
        "**THE `#[test]` DELTA WAS UNDER-ESTIMATED, AFTER FIVE CONSECUTIVE LANES OVER-ESTIMATED IT**",
        "**SCORED.** #2481 told the next lane to register **+4** and treat `±3` as the whole claim. This lane registered exactly that and landed **+12** — `#[test]` 1,428 at `111b6357`, 1,440 at tip, targets 38 → 38",
        "+4 registered · **+12** actual · 6 lanes, 2 directions",
        "rungs/2026-08-09-w-biquad.md §9 · #2481 · #770",
        "**The correction over-corrected, which is the more useful failure**: the estimator is not biased, it is unanchored. A lane shipping two emitters, a writer path and a label rule is not the same subject as a fence narrowing, and what predicts the count is the number of *facts each needing its own cell*, not a per-lane constant",
    ),
    (
        "2545",
        "**`hatch-red` IS REFUSED AT MASTER FOR A DIFFERENT REASON THAN THE COMMISSION NAMED**",
        "**COUNTERFACTUAL, run here.** The commission expected `HATCH-DRIFT id=call-arg-lit-permuted` (#1406). This tree reads **`HATCH-STALE` — `hatch.py apply` cannot hatch this tree** (#1389), a different refusal one stage earlier, and it reproduces at `111b6357` with `crates/` reverted",
        "1 arm · 0 of 14 run · reproduced at base",
        "rungs/2026-08-09-w-biquad.md §8 · #1389 · #1406",
        "**Board #1406 applies either way: this gate run does NOT establish what a full hatch run establishes.** Declined rather than repaired — #1322 makes the disposition a judgement about the change that stalled it, and this lane did not make that change",
    ),
]

FREE = """
> **`#2500`–`#2529` are allocated to lane `w-vec` and are NOT this lane's to
> mint.** `w-biquad` took `#2530`–`#2559` rather than the next free number,
> because that block was already spoken for.

> **`#2546`–`#2559` are minted by nobody and are FREE.** Lane `w-biquad` was
> allocated `#2530`–`#2559` and used sixteen (`#2530`–`#2545`). The unused
> fourteen are recorded as explicitly unminted rather than left to be inferred
> from a gap.
"""


def main():
    path = "docs/BOARD.md"
    s = open(path).read()
    if "**2530**" in s:
        print("already appended; refusing")
        return 1
    lines = [
        "| **{n}**<sub>w-biquad</sub> | {item} | {verdict} | {number} | {where} | {notes} |".format(
            n=n, item=i, verdict=v, number=num, where=w, notes=no
        )
        for (n, i, v, num, w, no) in ROWS
    ]
    open(path, "w").write(s.rstrip("\n") + "\n" + "\n".join(lines) + "\n" + FREE)
    print(f"appended {len(lines)} rows")
    return 0


sys.exit(main())
