#!/usr/bin/env python3
"""boardrows.py — insert lane w-seed's board rows after #1056.

A one-shot editor. `docs/BOARD.md` is one row per line and the rows are long
enough that a hand edit is a transcription risk; this keeps the insertion
reproducible and the numbers in one place.
"""

ROWS = [
    # (number, title, verdict, where, note)
    (
        "1087",
        "**A REFUSED BODY CAN NOW *SEED* MECHANISM E — board #1053 closed, and it is a CAPABILITY and not a production**",
        "**DONE. `fnbyte-differs` 2,334 → 2,111, `fnbyte-exact` 35,986 → 36,209, `fnbyte-elided` 1,654 → 1,877 = `-elided-exact`; 223 converted, 0 regressed, 0 with different port bytes**, per `(TU, emit_name)` from two `--fnbyte-diff-jsonl` files and never by subtracting totals. `c2_il::…::no_effect::no_effect_nothing` is a third **decode-only** reader and the first that returns a `bool` rather than a callee token, because the body it reads has **no callee**: `p->~T()` on a class with a trivial destructor is `33 <int> v · 33 <void> v · 44 · 4B` and a return. `c2_core::elide::Reduction::NoEffectNothing` contributes `(seeds, link) = (true, None)`",
        "rungs/2026-08-08-w-seed.md §1, §3; GRID-N — **11 frozen cells**, `sha256` committed before the first `cl.exe`, graded per **call edge** at the workload's flags **and** `/Ob0` with the caller's bytes printed; 11 integration tests, 18 unit tests",
        "**`parse_segment` is byte-for-byte unchanged**, the row stays `FnVerdict::Blocked` and `fnbyte-refused` **130,579** at both ends, and `IlBundle::functions` still refuses its whole TU — #971 condition 4 satisfied by construction. The literal **TYPES** are pinned and the literal **VALUES** are not: a literal is pure whatever its value and the statement is discarded, so the value cannot change what is emitted (#644), while a `float`/`double` would drag `_fltused` in and the obj would grow a symbol",
    ),
    (
        "1088",
        "**THE CYCLE ARGUMENT IS RE-DERIVED — and the mutation that should have proved it came back GREEN, because the guard is not where the doc said**",
        "**MEASURED.** `w-fix` #950's *\"a cycle is never **seeded**, so it is never admitted\"* was true only because `empty_body` was the only seed, so it does not survive a widening on its own authority. Re-derived in four steps on `Reduction`'s doc: termination reads the **step** and not the seed set; a seeded name has **no outgoing link**; admission propagates only backwards along links from a seed; a cycle member always has a link and so cannot seed. **Mutation M3a** gives `NoEffectNothing` a link anyway and **nothing goes red** — the iteration skips a name already in `in_r`, so a seeded name is never asked for its link and that arm is **inert as the loop is written**",
        "rungs/2026-08-08-w-seed.md §5; `work/w-seed/mutate.sh` — **M1 RED (2), M2 RED (6), M3a GREEN, M3b RED (2)**; GRID-N `n06`",
        "**M3a is reported green rather than rewritten until it went red.** A mutation edited to fail proves nothing, which is the whole reason the registration is written down first. What it establishes is *where* the property lives: in the **reader's vocabulary** (M2 — open it so a body with a call in it can seed, 6 tests red) and in `NoEffectCall` **not** seeding (M3b — 2 tests red including the cycle one). Both doc sites are corrected in place, because the old wording is a licence a later lane would cite to weaken the vocabulary on the grounds that *\"`elide.rs` sets the link to `None` anyway\"*. It cannot save you",
    ),
    (
        "1089",
        "**THE POINT PREDICTION WAS 227 AND THE MEASUREMENT IS 223 — and the four are named down to the byte**",
        "**223.** `fnbyte-blr-stop3-expr-lit-type-8207` **227 → 4**, and the differs whose whole reference body is one `4e800020` went **232 → 9**. The four that did not convert carry the same census key and a **different production**: where a class element type folds the pointer away to an int literal, an **enum** element type keeps it, so `??$__destroy_aux@W4CubeFace@RndCubeTex@@…` on `src/system/rndobj/CubeTex.cpp` reads `b9 <formal> 86 43 c9 50` where the graded body reads `33 86 41 74 00`",
        "rungs/2026-08-08-w-seed.md §6; `work/w-seed/blrresidue.py` (232 → 9, by name); `a_formal_load_in_place_of_the_int_literal_is_declined`",
        "**A residue with no name is a residue nobody can price.** w-memset's 227 was a chain-stop count, and a chain-stop count is an upper bound on what one production closes — it says where chains stop, not that one reader reaches all of them. The gap is exactly the second production and it is #1090",
    ),
    (
        "1090",
        "**THE FORMAL-LOAD VARIANT — the same census key, a second production, worth 4 differs, DECLINED**",
        "**OPEN, priced at 4.** `??$__destroy_aux@W4Something@…` for an **enum** element type loads the pointer instead of folding it: `B9 <formal> <TYPE> · 33 82 07 <id> <v> · 44 · 4B`. It is very probably as pure as the graded shape — a discarded formal load emits nothing — and it is **not admitted**",
        "rungs/2026-08-08-w-seed.md §10.1; the declined shape is pinned as a byte-level test so a widening turns it red",
        "**Declined because GRID-N has no cell for it**, and adding the arm in the lane that discovered it needed one is fitting a reader so that four more functions convert. It also has a real extra obligation the literal form does not: a `B9` operand must be checked to name one of *this function's own formals*, for the reason `no_effect_loop`'s induction step already has that test — `26` is the data-symbol push and a body that materialized a global while reading as pure is `elide.rs` condition 3 one level down",
    ),
    (
        "1091",
        "**THE CENSUS KEY IS THE FIRST BLOCKING FEATURE, NOT A DESCRIPTION OF THE BODY — 10 seeded rows arrive under a second key**",
        "**MEASURED, and it converted nothing.** `fnbyte-nothing-key-expr-lit-type-8207` **4,156** and `fnbyte-nothing-key-param-width-undetermined:mid` **10**. The ten are the *same* `??$__destroy_aux@…` production at the same 115-byte segment length; `parse_segment` blocked in the **formals** region before it ever reached the body, so the key describes where the parser stopped and not what the function does. They extended **7** no-effect chains (`fnbyte-noeffect-stop-param-width-undetermined:mid` 80 → 73) and produced **zero** elisions",
        "rungs/2026-08-08-w-seed.md §7; `work/w-seed/findkey.sh`; the conversion arithmetic closes without them — 227 − 4 = 223 = 232 − 9",
        "**Registered in the prereg as a finding and not a bonus**: *\"a second key appearing here is a body the grid never saw\"*. The consequence for the next lane is a warning about instruments, not about this rule: **a stop histogram keyed on the census verdict does not partition the same way a body-keyed reader does**, so sizing a body rung off a key histogram is sizing it off a different question. Zero elisions came from these ten, and `fnbyte-elided` = `-elided-exact` = **1,877** is what says so",
    ),
    (
        "1092",
        "**THE FAMILY SPREAD IS ONE TEMPLATE, AGAIN — 223 of 223 are `??$_Destroy_Range@…`, across 150 TUs**",
        "**MEASURED. 1 distinct outermost template, 150 distinct TUs.** The fourth consecutive conversion family to be one or two templates: #925 (`??1?$_Rb_tree_base@…`), #952 (the same, one template up), `w-inl0`'s 138 (`??$_Destroy_Range@…` over scalars), and now the same name over class element types",
        "rungs/2026-08-08-w-seed.md §4; `work/w-seed/family.py`",
        "**Said out loud because a count invites the next lane to read it as breadth.** 223 functions is 150 instantiations of one STL algorithm, and the mechanism it exercises — a refused body seeding a least fixpoint — is general while the population is not. The next rung sized off *this* number, rather than off a scan, will be sized wrong in exactly the way #1047 records",
    ),
    (
        "1093",
        "**TWO PLANTED RESIDUE ASSERTIONS WENT RED ON PURPOSE, AND BOTH WERE INVERTED IN THE COMMIT THAT BROKE THEM**",
        "**DONE.** `destroy_loop_elision.rs::the_pseudo_destructor_leaf_is_the_residue_and_needs_a_seed` (w-memset, `l09`) failed **alone** — 10 passed, 1 failed — with the message it was given: *\"the wrapper came back Exact. THE SEED EXISTS NOW.\"* `dead_temp_elision.rs::the_loop_overload_is_the_residue_and_is_not_converted` (w-inl0, `m06`) failed for the same reason one target later. Both now assert the conversion **level by level** — the loop admitted as a LINK, the leaf as a SEED, both still `parse-refused` — because *\"the wrapper is Exact\"* alone would pass if the port had elided it for some other reason",
        "rungs/2026-08-08-w-seed.md §8; `work/w-seed/l09_red.txt`",
        "**This is the technique working exactly as designed, twice.** w-inl0 wrote *\"a lane that later converts them will turn this test red and should — with the rung that explains why\"*, and w-memset wrote its assertion *\"precisely so that widening the seed set turns it red in the same commit\"*. Neither is a stale expectation and neither was deleted: a decline that is asserted is a decline the next lane cannot cross silently, and the cost of the technique is one deliberate red per crossing",
    ),
    (
        "1094",
        "**THREE INTEGRATION TESTS HAD GROWN THREE COPIES OF `grade_cell`; #1053 WOULD HAVE MADE IT FOUR**",
        "**DONE for two of four.** `crates/c2-harness/tests/cellgrade/mod.rs` now owns the ANCHOR, the TAIL PAD, the flag profile, `grade_cell`, `row`, `row_opt` and the per-cell scratch directory; `nothing_seed.rs` and `destroy_loop_elision.rs` use it. **OPEN:** `empty_elision.rs` and `dead_temp_elision.rs` still carry their own",
        "rungs/2026-08-08-w-seed.md §9; `crates/c2-harness/tests/cellgrade/mod.rs`",
        "**`w-relo`'s merge is the price of the fourth copy**: two lanes wrote the same reader in different files, auto-merged with **no conflict marker**, and the duplicate walks were caught only by a compile error. Independent invention never conflicts, so the count only goes up. The two that were **not** migrated were left deliberately — `w-empty`'s cells **append** the ANCHOR where the template cells **prepend** it (`w-inl0` §4), so migrating them is a behaviour change to a peer lane's pinned test to prove a point this rung does not need",
    ),
    (
        "1095",
        "**A SEED'S KNOWN ANSWER IS STRONGER THAN A LINK'S, AND IT READS 0 OVER 398 WITNESSES**",
        "**MEASURED. `fnbyte-nothing-ref-other` 0**, `-ref-blr` **398**, `-ref-absent` **3,767**, `-not-admitted` **0**, `-unnamed` **1**, over `fnbyte-nothing-rows` **4,166**. For a **link** the known answer only exists where the callee happened to close; a **seed** asserts unconditionally that c2 emits nothing for the body, so every row with a `.text` COMDAT has one, and every one of the 398 is the single word `4e800020`",
        "rungs/2026-08-08-w-seed.md §7; `gap-metric fnbyte-nothing-*`",
        "**It is a positive count and not a subtraction**, for the reason every control on this page is: a key that appears only when non-zero is a key whose absence reads as success. It is also **not sufficient on its own** — `w-inl0`'s M2 is the standing proof that a control over the admitted row's *own* bytes is green while the rule is wrong about its **caller**, which is why the movers are measured per symbol and GRID-N grades per edge",
    ),
    (
        "1096",
        "**WHAT IS LEFT OF BOARD #980's 370 — nine differs, priced by production**",
        "**9**, from 370 across four lanes (`w-inl0` 138, `w-memset` 0, `w-seed` 223). By stop: `fnbyte-blr-stop3-expr-lit-type-8207` **4** (#1090, the formal-load variant), `-blr-stop3-expr-call-in-expr-recv-load-then-type-void-and-op-more` **1**, `-blr-stop2-module-end-0x4D` **2** (`w-inl0` §9.4's one-byte reader gap — the last segment of a bundle has no module trailer), `-blr-stop-callee-unbound` **1**, `-blr-stop2-callee-unbound` **1**",
        "rungs/2026-08-08-w-seed.md §10; `work/w-seed/blrresidue.py`, by name",
        "**Every one of the nine is named, and none of them is worth a lane on its own.** Recorded so the family is closed as a family rather than left as a number that drifts: the next lane to quote *\"board #980's 370\"* should quote **9**, and should read #1047 before sizing anything off either",
    ),
]


def row(n, title, verdict, where, note):
    return (
        f"| **{n}**<sub>w-seed</sub> | {title} | {verdict} | {where} | {note} |"
    )


def main():
    p = "docs/BOARD.md"
    lines = open(p).read().split("\n")
    anchor = next(i for i, l in enumerate(lines) if l.startswith("| **1056**<sub>w-memset</sub>"))
    new = [row(*r) for r in ROWS]
    lines[anchor + 1 : anchor + 1] = new
    open(p, "w").write("\n".join(lines))
    print(f"inserted {len(new)} rows after line {anchor + 1}")


if __name__ == "__main__":
    main()
