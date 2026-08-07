#!/usr/bin/env python3
"""rows.py — insert w-carrier's board rows #1207..#1216 at their NUMERIC
position, immediately after #1206, and fail hard if the anchor is not unique or
if any of the numbers is already taken.

Board rows are inserted by position, never appended: `docs/BOARD.md` is read by
number and a row filed out of order is a row nobody finds.
"""
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
BOARD = os.path.join(ROOT, "docs", "BOARD.md")

ROWS = []


def row(n, title, verdict, worth, where, notes):
    ROWS.append(
        (n, f"| **{n}**<sub>w-carrier</sub> | **{title}** | {verdict} | {worth} "
            f"| {where} | {notes} |\n")
    )


row(1207,
    "BOARD #1199 IS CLOSED — THE BIND CARRIER IS AN `IlOp`, NOT A FIELD, and the "
    "bad state is UNSPELLABLE rather than merely unreached",
    "**DONE**, and the shape was registered in the prereg before the first line "
    "of `crates/` was written. `IlOp::BoundAddr { tok, base, off }` — *\"the "
    "token `tok`, which denotes `base + off`\"* — stands in the op stream exactly "
    "where `Load(<bound local>)` stood; `shape_to_function` discharges the "
    "reader's `RefBind` list into it and then builds **the carriers that already "
    "exist**. **No field was added to `IlFunction`, `CallSeq` or "
    "`StoreRunPrefix`.** The rejected rival is a `binds:` list beside the ops — a "
    "SECOND CONTAINER, and `ops` and `CallSeq::store_run` are already two homes "
    "for a run, so a consumer can hold the run and drop the bindings, which is "
    "board #232's mechanism and #844's own. Inside the op stream there is nothing "
    "beside the ops to drop",
    "1 variant · 0 new fields · **4** forced match arms, all one-line refusals in "
    "the two EXPRESSION emitters — the prereg's own failure condition was "
    "*\"three or more modules that have nothing to do with store runs\"*",
    "rungs/2026-08-08-w-carrier.md §2; `c2-il/src/func/mod.rs::IlOp::BoundAddr`; "
    "`bundle.rs::shape_to_function`",
    "The property that makes it unspellable is not the variant, it is that **the "
    "base SYMBOL and the base ADDRESS are two derivations of ONE value**: `tok` "
    "is what `schedule::Stmt::base` keys aliasing on, `base + off + <store off>` "
    "is the instruction's, and both are destructured in the same match arm. To "
    "emit the DIRECT spelling's words the stream would have to hold `Load(base)` "
    "where `BoundAddr` stands, and the reader never substitutes. The offset is "
    "summed at **one** site, so the binding cannot be discharged twice")

row(1208,
    "#868/#836 IS MEASURABLE FOR THE FIRST TIME, AND THE NUMBER IS **1** — the "
    "frontier's last refusal is now a named, countable row",
    "**MEASURED on the 878-TU scan at both ends**, with a base binary built from "
    "`7a52aa2b` in the same tree. `w-bind` §8 recorded that #868/#836 *\"is "
    "unmeasurable until the carrier is paid — nothing reaches `alloc::allocate` "
    "on this body\"*. It is paid: `store-run-bind-no-emitter-carrier:eof` **1 → "
    "0** and `store-run-bind-mixed-kind-alloc:eof` **0 → 1**, with `fn_blockers` "
    "total **1,751,958 at both ends** and `emit_blockers` total **130,576 at both "
    "ends**. Exactly two rows move and nothing else does",
    "**1** function, and it is `src/xdk/nuispeech/xboxheap.cpp` — the frontier's "
    "cheapest TU",
    "rungs/2026-08-08-w-carrier.md §6; `work/w-carrier/blockers.py`; "
    "`fixtures/cpp/w1199_bind_run_neg.cpp::nf_mixed`",
    "The row this lane was sent to unblock, and it is unblocked as a "
    "**measurement, not a conversion**. What it names is unchanged and unlifted: "
    "#836 (clause 1 wrong on 29 of 81, the refusal wrong on 0), #868 (the narrow "
    "lift 12 MISS of 36, `slwi` 0/12), #1134 (clause 1 refuted on this very mix "
    "by `j1_lit2`). `nf_mixed` is the shape as a graded fixture so the row cannot "
    "go quiet")

row(1209,
    "A BASE-ONLY BIND MATERIALISES **NOTHING** — c2 folds the bound object's "
    "offset into the store's displacement, and that is the whole reason an accept "
    "surface exists",
    "**MEASURED, and it is the lane's REGISTERED LOSS (prereg P0) landing on the "
    "right side.** `work/w-carrier/grid/k_base1` — `h->mSize=2; BE& "
    "l=h->mListHead; l.mNext=p;` — is `li 11,2 ; stw 11,16(3) ; stw 4,8(3)`: "
    "**no `addi`**, base r3, displacement `8 + 0`. `k_both1` — the same bind used "
    "as a VALUE — is `addi 11,3,8 ; stw 11,8(3)`. So the bound name is a "
    "**producer only in the value position**, which is exactly where #836's "
    "mixed-kind refusal lives: the two facts are one fact",
    "**20** bind bodies emit byte-exact across 73 graded cells; the registered "
    "alternative (*\"the emittable sub-class is empty\"*) is refuted",
    "rungs/2026-08-08-w-carrier.md §4.2; `work/w-carrier/twins.out`",
    "Registered as the clause most likely to lose, with the consequence of losing "
    "it written down: *\"this lane ships a carrier with no accept surface at "
    "all\"*. It held, and the same measurement explains why `xboxheap` does "
    "**not** convert — its bind is used as a value, twice")

row(1210,
    "#1128 HOLDS IN THE **PORT'S OWN BYTES** — six BIND/DIRECT pairs, both halves "
    "emitted, differing exactly where real `c2`'s differ",
    "**MEASURED on this lane's own captures and its own emitter.** Every accept "
    "candidate in GRID K carries a `_c` control that is the same body with the "
    "bind removed. `k_base1`/`k_base1_c`, `k_base2`, `k_off24`, `k_gap1`, "
    "`k_gap2`, `k_pos_last` — real `c2`'s `.text` DIFFERS on all six (`li 11,2 ; "
    "stw 11,16 ; stw 4,8` against `li 11,2 ; stw 4,8 ; stw 11,16`), and **both "
    "halves of all six grade `Port=Match`**. The mirror is equally load-bearing: "
    "`k_off0`/`k_off0_c` and `k_dead`/`k_dead_c` are TEXT IDENTICAL and both stay "
    "refused",
    "6 pairs × 2 halves byte-exact · 2 identical pairs refused · prereg **P4**",
    "rungs/2026-08-08-w-carrier.md §4.4; `work/w-carrier/twins.sh`; "
    "`fixtures/cpp/w1199_bind_run.cpp`",
    "Boards **#1200**/#1128 promoted from *\"the reader returns different "
    "shapes\"* to *\"the emitter writes different bytes\"*. A carrier that "
    "collapsed the spellings would emit the other body's words, which is #232's "
    "direction; this is the positive proof rather than the argument")

row(1211,
    "THE `88-store-run-call` SWEEP REFUTED THIS LANE'S FIRST EMITTER — 4 cases "
    "and 56 cross cells, and the 53-cell FROZEN GRID was green through all of "
    "them",
    "**FOUND BY THE GENERATED CORPUS, not by the grid.** "
    "`work/w-carrier/bisect/s1427.cpp`: `H::H(a,b){ BE& lh=mListHead; mCount=0; "
    "lh.mNext=(BE*)this; Reset(); }`. Real `c2`: `li 11,0 ; mr 31,3 ; stw "
    "11,20(3) ; stw 3,8(3) ; bl`. The port: `li 11,0 ; stw 11,20(3) ; mr 31,3 ; "
    "stw 3,8(3) ; bl`. **The copy lands after ZERO stores and board #867's rule "
    "says one** — two right words in the wrong order, an obj that links",
    "4 sweep cases · 56 cross cells · **0** of them visible to 53 frozen cells",
    "rungs/2026-08-08-w-carrier.md §5, §5.2; `work/w-carrier/bisect/`",
    "**Why the grid could not see it**: GRID K has exactly ONE cell with a call "
    "tail and a producer, and its unproduced store is FIRST in source order, so "
    "its leading run and its count agree. The axis the grid did not vary is "
    "**which kind of store leads the run** — named here because four earlier "
    "grids on this row did not vary it either. Third time in two days a corpus or "
    "a cross-product caught what a frozen grid did not (#1174, #1189)")

row(1212,
    "THE `mr r31,r3` SLOT IS FED THE **WRONG `u`** ON A MULTI-SYMBOL RUN — "
    "`store_run_call`'s own identity argument is false there, and the carrier is "
    "what opened the region",
    "**MEASURED — the mechanism behind #1211.** `store_run_call.rs` feeds "
    "`save_slot` the **COUNT** of unproduced stores and its doc argues that "
    "equals #584's **LEADING RUN**: *\"they cannot be [separated] … the leading "
    "run is always at least `min(2, total)`\"*. That is `store_order`'s floor and "
    "it holds on a **SINGLE-symbol** run — every cell #867 was fitted on and "
    "every cell of its 18/18 fresh holdout. "
    "`order::tests::the_two_readings_of_u_agree_on_every_single_symbol_run` "
    "enumerates 5,000+ to say so and `the_layout_u_is_the_leading_run_not_the_count` "
    "exhibits the multi-symbol cell where they differ. **A bind IS a second base "
    "symbol (#1128)**",
    "`save_slot(1,1) = 1` ships · `save_slot(1,0) = 0` is c2's answer on all four "
    "cells · the pre-existing class cannot reach it (its only multi-symbol "
    "admission is the all-one-literal run, where the count is 0)",
    "rungs/2026-08-08-w-carrier.md §5.1, §5.3; "
    "`codegen/store_run_call.rs::a_bind_carrying_run_is_refused_in_the_composition_with_its_counterexample`",
    "**REFUSED, not corrected** — `store-run-bind-call-tail-mr-slot` in the "
    "reader and `BIND_IN_A_COMPOSITION` as the emitter backstop, and `k_call` "
    "went `match → vocab-gap` to buy it. The correction changes a rule governing "
    "**every** #844 body and would rest on the four cells that refuted this lane, "
    "which is how all six refuted allocation keys got written. Both numbers are "
    "pinned beside the refusal so a lifting lane meets the target and not just "
    "permission")

row(1213,
    "THE CARRIER CONVERTS **ZERO** FUNCTIONS ON THE 878-TU WORKLOAD, AND THAT WAS "
    "REGISTERED BEFORE THE FIRST PROBE",
    "**MEASURED at 0, not inferred.** `fn_in_class` **711,485** and `fn_total` "
    "**2,463,443** at both ends; `fn_blockers` total **1,751,958** at both ends; "
    "`emit_blockers` total **130,576** at both ends; the whole `gap-metric` block "
    "is **byte-identical** under `diff`. TU match **10 → 10**, mismatch **0 → "
    "0**, `codegen-gap` **0 → 0**, `fnbyte-exact` **36,212** unshrunk, "
    "`fnbyte-differs` **2,111** ungrown. `peerkeys.py`: **FAMILIES THAT VANISHED: "
    "0**",
    "**0** functions · **0** TUs · 20 bind bodies on grids and 14 in the sweep "
    "fragment",
    "rungs/2026-08-08-w-carrier.md §1, §7; `work/w-carrier/metrics_{base,tip}.txt`",
    "Prereg **P3**, registered so that no representation change could be reported "
    "as breadth. `w-bind` measured #839's whole residue at **ONE** body over 878 "
    "TUs and this lane's own headline refusal takes it, so a carrier that "
    "converted anything on the workload would have been the surprise. The accept "
    "surface is real and graded — 20 cells, 14 sweep cases, a 15/15 fixture — and "
    "its **workload** population is zero, and both sentences are true")

row(1214,
    "`bind_run_ops` IS ONE DECISION PROCEDURE WITH TWO CALLERS AND SIX KEYS — and "
    "the gate is drawn STRICTER than the model's own domain, on purpose",
    "**DONE.** `shape_to_function` calls it for acceptance and "
    "`census::bind_refusal_key` calls it for the reason, so the key the census "
    "prints and the answer the model gives cannot drift. The one-producer bound "
    "is **not** a restatement of `order`'s domain: a test written as the opposite "
    "claim FAILED — `store_order` answers `k_2const` (two producers, two symbols) "
    "and answers it right, `P0 P1 S0 S1 S2`, real `c2`'s own. It is declined "
    "anyway, because at ONE producer the walk provably cannot fail and the "
    "reader's class is inside the emitter's **by construction**, and at two it "
    "would rest on domain gates this crate cannot see",
    "6 keys · 1 procedure · 2 callers · `k_2const` answered-and-declined · "
    "`k_3const` genuinely out of domain",
    "rungs/2026-08-08-w-carrier.md §3, §4.3; "
    "`codegen/order.rs::the_bind_runs_the_model_answers_and_the_one_it_answers_that_is_declined`",
    "`w-seam2` §6 had to move two gates out of the emitter after "
    "`census_gate.rs` named three functions the census counted and `PortC2` "
    "refused; this lane put all six in the reader from the start and kept two "
    "emitter backstops. The negative fixture is **0 of 5 in class with a DISTINCT "
    "key per function**, so no clause can go quiet")

row(1215,
    "TWO GATES THAT REFUSED NOTHING — one DELETED, one reported as exercised only "
    "by a unit test",
    "**FOUND by asking, which is #1175's whole lesson.** An earlier revision "
    "carried a **live-argument-base** clause (a bind hanging off a formal the "
    "call keeps alive); the call-tail refusal above it takes every body it could "
    "have caught, so it is **deleted rather than left dead**. The **pool** clause "
    "(`3 + params > 11`) fires on **ZERO** graded cells — "
    "`work/w-carrier/grid2/g_pool`, a nine-formal body, refuses one layer earlier "
    "at `expr-op-0x27` — and is exercised only by "
    "`every_bind_gate_fires_on_a_named_input`",
    "1 clause deleted · 1 clause with **0** graded witnesses, stated as such · 5 "
    "of 6 keys graded against real `c2`",
    "rungs/2026-08-08-w-carrier.md §8.1 D7; "
    "`leaf_store.rs::every_bind_gate_fires_on_a_named_input`",
    "`w-seam2`'s live-argument gate keyed on `value_is_load`, matched nothing, "
    "and was found only by a cross-check comparing two independent answers. The "
    "mitigation here is a test that FIRES every clause on a named witness with a "
    "printed count — and the two clauses it could not ground in a graded obj are "
    "named rather than counted as measured")

row(1216,
    "THE MULTI-SYMBOL SINGLE-PRODUCER RUN IS EMITTABLE, AND THE NON-BIND PATH "
    "STILL REFUSES IT — found and not taken",
    "**MEASURED as an asymmetry, on a matched pair.** "
    "`work/w-carrier/grid/k_nonthis` (`a->mSize=2; BE& l=b->mListHead; "
    "l.mNext=q;`) is **`Port=Match`**, byte-exact. Its DIRECT twin `k_nonthis_c` "
    "is **`vocab-gap`**, refused by `collect_store_run`'s clause 1 (*\"ONE base "
    "symbol\"*). The bind production skips clause 1 — `w-bind` wrote it that way "
    "because a bind IS a second symbol — and this lane replaced it with a "
    "one-producer plus two-crossing bound that is provably inside `order`'s exact "
    "region",
    "1 matched pair, opposite verdicts; the population behind clause 1 is **not "
    "sized by this lane**",
    "rungs/2026-08-08-w-carrier.md §9 item 1; `work/w-carrier/table_grid.txt`",
    "Prereg **P6** registered *\"I expect to have to restore clause 1\"* and that "
    "is the row's HALF: it was not restored and did not need to be. A lane that "
    "replaces clause 1 with the same two clauses inherits a population **nobody "
    "has sized**, and sizing it is the first thing it owes — the whole point of "
    "#621 is that a 99 % layout clause was measured and refused")


def main():
    text = open(BOARD).read()
    for n, _ in ROWS:
        if re.search(rf"^\| \*\*{n}\*\*", text, re.M):
            sys.exit(f"FAIL: board row #{n} already exists — renumber")
    anchor = "| **1206**<sub>w-bind</sub> |"
    if text.count(anchor) != 1:
        sys.exit(f"FAIL: anchor {anchor!r} occurs {text.count(anchor)} times, want 1")
    i = text.index(anchor)
    j = text.index("\n", i) + 1
    out = text[:j] + "".join(r for _, r in ROWS) + text[j:]
    open(BOARD, "w").write(out)
    print(f"inserted {len(ROWS)} rows after #1206")


if __name__ == "__main__":
    main()
