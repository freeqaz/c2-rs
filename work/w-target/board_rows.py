#!/usr/bin/env python3
"""board_rows.py — insert lane w-target's rows (#1037-#1046) into docs/BOARD.md.

Lane w-target merge tooling. **Fails hard rather than skipping** — the brief
records that board ranges collided twice this wave, and a renumber tool that
silently does nothing is how the second collision happened.

    board_rows.py [--check]

`--check` verifies the range is free and the anchor exists, and writes nothing.
"""

import re
import sys

BOARD = "docs/BOARD.md"
# **The anchor moved once, during the rebase onto master, and that is the point
# of having a tool.** The lane branched off `217d4a85`, whose last row was
# #1026; by the time it rebased, `w-quar` had landed #1027-#1036. Inserting
# after #1026 would have put this lane's #1037-#1046 *above* them — numerically
# out of order in a file whose ordering is its index. Re-run against the new
# anchor instead of hand-merging the conflict. Numbers themselves did NOT need
# to move: the ranges were disjoint, which `--check` re-verified.
ANCHOR = "| **1036**<sub>w-quar</sub> |"
FIRST, LAST = 1037, 1046

ROWS = [
    (
        1037,
        "**`__declspec(noinline)` IS A CHAIN c2 DOES NOT CLOSE — #1013's closure "
        "hypothesis has a COMPILED COUNTEREXAMPLE**",
        "**DECLINED, and the decline is the deliverable.** On the 878-TU workload "
        "the closure converts **158** of the 861 and fires on **zero** of the "
        "3,803 relocating bodies the judge credits today — `wtarget-fix-convert` "
        "158 · `-regress` **0**. GRID-W cell `w04a` stops it: c2 obeys the "
        "attribute, `?f` branches to `?g` on **both** sides and is `Exact` today, "
        "and the rule would rename its target to `?ext` and take it out of the "
        "credited count. `regress 1` on that cell, against real c2's own obj",
        "rungs/2026-08-08-w-target.md §1, §4; "
        "`crates/c2-harness/tests/noinline_boundary.rs`; "
        "`work/w-target/cells/w04a_noinline.cpp`",
        "**A rule whose counterexample is compiled is not a rule with an "
        "unmeasured error rate — it is a rule that is wrong, and the workload's "
        "`regress 0` is a property of the workload.** `src/lazer/game/"
        "BustAMovePanel.cpp` is TU **#4 of the 878** and carries three "
        "`__declspec(noinline)` functions; none is a body either mechanism "
        "reaches, which is why the corpus is silent. `w-drop3` §6 declined this "
        "rung on the *absence* of a discriminator; this lane declines it on the "
        "*presence* of a counterexample, which is a stronger reason and a cheaper "
        "one to re-check",
    ),
    (
        1038,
        "**THE SHIPPED `SPLICE-0-PORT` ALREADY EMITS THE WRONG BODY THROUGH A "
        "`noinline` CALLEE — latent on this corpus, demonstrated on a cell**",
        "**OPEN, in landed code (`crates/c2-core/src/splice.rs`).** GRID-W2 `w10` "
        "puts the attribute on a leaf the splice reaches: the port emits `?g`'s "
        "**two-word** body where c2 emits **one** branch word, **0** words equal, "
        "and **no** relocation where c2 emits a `REL24` against `?g`. `w12` is the "
        "same source without the attribute and splices **byte-exactly**, so the "
        "red verdict is the attribute's doing and not the splice's",
        "rungs/2026-08-08-w-target.md §5; "
        "`crates/c2-harness/tests/noinline_boundary.rs::"
        "the_shipped_splice_emits_the_wrong_body_through_a_noinline_callee`",
        "**LATENT, NOT LIVE, and the distinction is the whole row.** The workload "
        "reads `fnbyte-spliced` **723** / `-spliced-exact` **723** / **0** differ, "
        "so nothing is wrong today; and no obj ships wrong either way, because "
        "`IlBundle::functions()` refuses every TU where a callee is also defined "
        "— which is every TU either mechanism can fire in. It is pinned as a "
        "**characterization test** rather than left in prose, because a defect "
        "the corpus does not exercise is a defect a scan cannot remember. "
        "**#878's loaded gun, one mechanism over**",
    ),
    (
        1039,
        "**THE DISCRIMINATOR EXISTS AND IT IS TWO BYTES OF `.gl` — everything else "
        "in the IL is byte-identical**",
        "**MEASURED, on two shapes.** A matched pair differing only by the "
        "attribute, compiled under filenames of the **same length** (the `.gl` "
        "embeds the source path), comes back with `.ex`, `.sy`, `.in` and `.db` "
        "**byte-identical** and `.gl` longer by exactly **2 bytes** — 2842/2842 "
        "and 2862/2862 on `.ex`, 381→383 and 377→379 on `.gl`",
        "rungs/2026-08-08-w-target.md §6; `work/w-target/nicmp2.sh`, "
        "`work/w-target/nicmp2.txt`; OPT_MODE.md §2",
        "**This is the priced next rung and it is NOT this lane's to take**: "
        "`crates/c2-il/` decode is another lane's ownership this wave, and the "
        "clause needed is fail-closed — a function whose `.gl` record carries the "
        "undecoded field must be **refused**, not parsed and silently treated as "
        "inlinable. `OPT_MODE.md` §2 already records that the opt word does *not* "
        "move for `noinline`, so the opt-mode path cannot carry it. GRID-W "
        "`w04d` shows the shape that is safe today **by accident**: "
        "`#pragma optimize(\"\",off)` makes the parser refuse the callee outright, "
        "so the rule cannot fire",
    ),
    (
        1040,
        "**THE COUNTERFACTUAL REACH — a rule graded WITHOUT moving one emitted byte**",
        "**MEASURED and shipped as an instrument.** R-CLOSE is applied to a *copy* "
        "of the port's relocation plan and the copy is graded by the same "
        "`compare_relocs` the partition uses, over all **4,664** relocating "
        "graded functions. One step: convert **85** · wrong **244** · null-differ "
        "**532** · **regress 0**. Fixpoint: convert **158** · wrong **171** · "
        "null-differ **532** · **regress 0**",
        "rungs/2026-08-08-w-target.md §3; keys `wtarget-close-*`, `wtarget-fix-*`, "
        "`wtarget-fn|…`; `gap::fnbytes::close_target`",
        "**Every pre-existing `gap-metric` is byte-identical across the change** — "
        "`diff` of two sorted 878-TU scans is empty. That is what makes it a "
        "counterfactual and not a rule: `w-drop3` §10.1 named *\"a lane that builds "
        "a number and then acts on it in the same commit has no before\"*, and this "
        "one deliberately builds the number and stops. **There is no new reader**: "
        "*what does the port emit for `g`* is `complete_comdat`, which is "
        "`PortC2::build`'s own composition and already carries `w-splice`'s chain "
        "closure",
    ),
    (
        1041,
        "**THE FIXPOINT BEATS ONE STEP, AND THE CYCLE CELL IS WHY**",
        "**MEASURED.** R-CLOSE\\* converts **158** where R-CLOSE converts **85** — "
        "the extra **73** are exactly the `chain2` family, and the cross-tab says "
        "so per function (`wtarget-fix-rel|chain2|convert` **73**). GRID-W `w03` "
        "adds the depth-3 answer the workload has no witness for: **c2 closes at "
        "depth 3 too**, and only the fixpoint gets it. On `w06` (a two-cycle) the "
        "one-step rule **regresses a credited body** and the fixpoint refuses with "
        "`chain-cycle`",
        "rungs/2026-08-08-w-target.md §4.2; `work/w-target/cells/w03_chain3.cpp`, "
        "`w06_cycle.cpp`",
        "Termination is structural — a step either repeats a name or admits a new "
        "one and the TU has finitely many — with a ceiling behind it so an edit "
        "breaking that argument refuses instead of walking forever, which is "
        "`elide.rs`'s and `splice.rs`'s round-ceiling discipline reached from a "
        "third direction. **Recorded even though the rule is declined**: if #1039 "
        "is ever paid, this is the variant to ship and the one-step version is "
        "already known to be worse",
    ),
    (
        1042,
        "**`w-drop3` §6.1's NUMBER, BUILT — and it is POSITIVE, so R-REFUSE is "
        "forbidden as written**",
        "**MEASURED: `wtarget-refuse-regress` = 1,065.** `w-drop3` asked for "
        "exactly one count — *how many `fnbyte-exact` bodies name a same-bundle "
        "callee the port refuses* — and registered the consequence in advance: "
        "*\"If that number is 0 the rule is free … if it is positive the rule is "
        "forbidden as written.\"* It is **1,065**, against **531** it would "
        "convert",
        "rungs/2026-08-08-w-target.md §3.2; keys `wtarget-refuse-convert`, "
        "`wtarget-refuse-regress`; BOARD #988",
        "**A refusal rule that removes 531 wrong claims by removing 1,065 right "
        "ones is strictly worse than the incumbent**, and the incumbent is doing "
        "nothing. Counted in the same pass and over the same denominator as "
        "R-CLOSE, so the two rules are priced against one population rather than "
        "two — which is the arithmetic `w-splice` proved subtraction cannot do",
    ),
    (
        1043,
        "**THE 861'S CLOSURE PARTITION, RE-DERIVED: 158 REACHABLE, 532 NOT "
        "ANSWERABLE AT ALL**",
        "**REPRODUCED to the digit** from this lane's own baseline scan against "
        "`w-relo` §4.1: `blocked` **529** + 3 · `unrelated` **169** + 2 · "
        "`chain2` **73** · `chain1` **69** · `seq→extern chain1` **16**. "
        "**Closure-reachable = 158 of 861 = 18.4 %**",
        "rungs/2026-08-08-w-target.md §2; `work/w-target/PREREG.md` §2; "
        "keys `fnbyte-reloc-fam|…`",
        "**Registered in the prereg as PRIOR state, not as this lane's "
        "prediction** — `w-relo` published it and this lane only re-measured it. "
        "Carried as its own row because the brief's framing (*\"a chain-closing "
        "rule fixes a large fraction of the 861\"*) is true of **under a fifth** of "
        "the queue, and 61.8 % are `blocked` — the port's own target is a "
        "parse-refused body, so the closure cannot be evaluated at all. A later "
        "summary that reads *\"the closure rule converts 158\"* as *\"the 861 are "
        "solved\"* is the failure this row exists to prevent",
    ),
    (
        1044,
        "**PREREG P1 LOST IN THE SAFE DIRECTION, AND THE LOSS IS RECORDED RATHER "
        "THAN RESTATED**",
        "**LOST.** `PREREG.md` §2 P1 predicted the closure rule's reach on the "
        "already-`exact` population at **> 158**, point estimate **1,200**, "
        "interval **200 … 4,000**. Measured: **0**. §5 registered this exact "
        "outcome as the *most-expected loss* and said *\"I do not believe it; I am "
        "registering it so that if it happens the surprise is on the record\"*",
        "rungs/2026-08-08-w-target.md §7; `work/w-target/PREREG.md` §2, §5",
        "The reason the reach is 0 is structural and worth more than the "
        "prediction: **every one of the 3,803 credited relocating functions fails "
        "the rule's precondition** — 2,738 name a callee no census row binds and "
        "1,065 name one the parser refuses. A callee whose own emitted body is a "
        "single call is 4 bytes, and `INLINE_PREDICATE.md`'s `s ≤ 64` region is "
        "where the inline decision is **categorical** rather than 0.9716. **That "
        "argument is what makes `w04a` decisive**: `noinline` is the one thing "
        "that overrides a categorical region, and it is unreadable",
    ),
    (
        1045,
        "**A TEST THAT RAN FOUR CAPTURES IN ONE TEMP DIRECTORY PRODUCED A FALSE "
        "FINDING, AND IT REVERSED THE LANE'S CONCLUSION**",
        "**FOUND AND FIXED.** The first `noinline_boundary.rs` keyed its work "
        "directory on the process id; `cargo test` runs a target's tests in "
        "parallel **threads of one process**, so four captures raced through one "
        "directory and `?f@@YAXXZ` was graded against another cell's obj. The "
        "failure presented as *\"the attribute IS visible in `.ex`\"* — which would "
        "have made #1039 payable inside `crates/c2-core/` and reversed the "
        "decline",
        "rungs/2026-08-08-w-target.md §6.1; `noinline_boundary.rs::work`; "
        "`work/w-target/nicmp2.sh`",
        "Re-measured **serially**, outside the test, on **both** shapes: `.ex` is "
        "byte-identical in each. **Recorded as a defect of this file's first "
        "version, not as a design** — the same discipline `w-relo` §4.1 applied to "
        "its own `blocked`/`unrelated` merge. The general form: *a parallel test "
        "that shares a filesystem path with its siblings can fabricate evidence, "
        "and the fabrication looks exactly like a finding*",
    ),
    (
        1046,
        "**NO EMITTER CHANGE, NO NARROWING, AND `fnbyte-exact` DID NOT MOVE BY ONE**",
        "**HELD.** `git diff master..HEAD -- crates/c2-core/ crates/c2-il/` is "
        "**empty**. The 878-TU partition is identical at both ends: `exact` "
        "**35,986** · `reloc-differs` **861** · whole-TU **2** · `differs` "
        "**2,334** · `partial` **0** · `refused` **130,579** · `unbound` **9,217** "
        "of **178,977**; TU match **10**, `mismatch` **0**",
        "rungs/2026-08-08-w-target.md §8; `work/w-target/base_metrics.txt` vs "
        "`tip_metrics.txt`",
        "The lane's acceptance condition was `reloc-differs` **861 → lower**. It "
        "is **861**, because the only rule that would have lowered it is one a "
        "compiled cell says is wrong. **A registered decline that leaves every "
        "number where it found it is the honest reading of `PREREG.md` §6**, and "
        "the deliverable is the four numbers (#1040, #1042), the twelve cells and "
        "the two live findings (#1038, #1039) rather than a metric that moved",
    ),
]


def main():
    check = "--check" in sys.argv
    text = open(BOARD).read()

    existing = set(int(m) for m in re.findall(r"^\| \*\*(\d+)\*\*", text, re.M))
    clash = sorted(n for n, *_ in ROWS if n in existing)
    if clash:
        sys.exit("FAIL: board numbers already present: %s" % clash)
    for n, *_ in ROWS:
        if not FIRST <= n <= LAST:
            sys.exit("FAIL: %d outside the lane's allocated range %d-%d" % (n, FIRST, LAST))
    if text.count(ANCHOR) != 1:
        sys.exit("FAIL: anchor %r appears %d times, expected 1"
                 % (ANCHOR, text.count(ANCHOR)))
    print("range %d-%d free; max existing = %d; anchor OK"
          % (FIRST, LAST, max(existing)))
    if check:
        return

    at = text.index(ANCHOR)
    end = text.index("\n", at) + 1
    block = "".join(
        "| **%d**<sub>w-target</sub> | %s | %s | %s | %s |\n" % r for r in ROWS
    )
    open(BOARD, "w").write(text[:end] + block + text[end:])
    # Named from ANCHOR rather than hardcoded: the first version printed
    # "after #1026" while inserting after #1036, which is a report that would
    # have been wrong in exactly the place a reader checks.
    print("inserted %d rows after %s" % (len(ROWS), ANCHOR.split("**")[1]))


if __name__ == "__main__":
    main()
