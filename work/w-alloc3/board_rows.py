#!/usr/bin/env python3
"""board_rows.py — insert lane w-alloc3's board rows #1067-#1076.

Lane w-alloc3 tooling. Writes `docs/BOARD.md` in place, once, and **FAILS HARD**
rather than skipping: the brief records that board ranges collided twice this
wave, and a renumbering tool that silently skips a taken number is how a
collision becomes two rows with one meaning. So this asserts, before writing,
that every number it mints is free and that the anchor line is unique.
"""

import os
import re
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
BOARD = os.path.join(ROOT, "docs/BOARD.md")

ROWS = [
    # (number, item, verdict, number-column, where)
    (
        1067,
        "**RULE BIND — *the inlined callee's body with its formal registers rebound and its result register chosen by one bit*. The SEVENTH allocation key to die, and the first that is not a producer-priority key**",
        "**REFUTED ON A FROZEN HOLDOUT — 5 WRONG of 38 in-domain graded cells.** It reproduces, from published bytes and with no toolchain at all, **all five** recorded rename witnesses — `w-seq`'s 123 (`?back@?$vector` against `?end@`), its 286 (`?Release@Object@Hmx@@` against `?Release@ObjRef@@`) and its hand cells `s01`, `s03`, `s11` — and it is **33 of 33** on GRID-A. GRID-H's sources and their `sha256` were committed at `22e30ad1` before a cell was compiled and the rule was frozen at `ccf208e3`; the grader re-checks all 46 and reads the frozen column. **It dies because c2 does not rename a body — it RECOMPILES an expression.** Four misses are one three-formal callee at the permutations that put its commutative pair the other way round, and the fifth is `int g(int a){return -a;}` at a site `g(x1)+4`, which c2 emits as **`subfic r3,r4,4`** — one word, an opcode absent from the callee",
        "GRID-A **38 cells / 33 graded / 33 HIT / 0 WRONG / 5 refused**; GRID-H **46 cells / 38 in domain / 33 HIT / 5 WRONG / 8 out of domain**, 46 of 46 `sha256` re-checked. The shipped refusal is **wrong on 0** of the same **71** cells",
        "rungs/2026-08-08-w-alloc3.md §4; `work/w-alloc3/gridH.tsv`, `gridH.frozen.tsv`, `misses.txt`; `crates/c2-core/src/codegen/alloc.rs` module doc — **`w-seq` §10.2's caution is now measured rather than argued**: the field diff says WHAT changed, not WHAT DECIDES IT, and a rule stated as a field edit is a description of the output. A successor may not restate BIND; what it owes first is a decision procedure for c2's operand **canonicalisation** (#1070), on a fresh frozen grid",
    ),
    (
        1068,
        "**RULE BIND's TEMP HALF SURVIVES — an inlined callee's result takes `POOL_TOP` = `r11`, measured in a regime `codegen::alloc` has never been exercised in**",
        "**MEASURED, and it is the surviving half of #1067.** Every grid behind ALLOC to date is a *store run in a leaf*. GRID-H's `H-wide` family is a different shape entirely — the single value an inlined callee returns, consumed by one more instruction — and it lands in `r11` at **every** caller formal count from **1 to 8**, at the first and the last bound position, with the caller's `r3` provably dead, and even when the callee already holds a temp in `r11` (`H-temp` 4 of 4). At five caller formals the **lowest** free volatile is `r8` and c2 emits `lwz r11,4(r7)`, so *\"the pool is walked highest-first\"* is separated from *\"lowest-first\"* on a population that is not a store run",
        "`H-wide` **16 of 16**, `H-temp` **4 of 4**, 0 wrong",
        "rungs/2026-08-08-w-alloc3.md §5.1; `crates/c2-core/src/codegen/alloc.rs::one_producer_takes_pool_top_at_every_floor` — the new test varies the **pool floor**, which no existing test does, so a future edit cannot invert the walk and stay green. Agrees with **#543** (`r12` never used) and **#605** (the descent reaches `r7`) from a third direction",
    ),
    (
        1069,
        "**THE 123 AND THE 286 ARE TWO IDIOMS, NOT 409 — the brief's *\"most constrained allocation evidence this project has ever collected\"* is `n = 2` when counted the way #925 and #952 require**",
        "**MEASURED on this lane's own 878-TU baseline.** The **286** source renames are **one symbol**, `?Release@Object@Hmx@@QAAXPAVObjRef@@@Z`, in 286 TUs. The **123** destination renames are **83 distinct symbols** in 76 TUs and **one template root**, `?back@?$vector<T>` — 83 instantiations of one accessor. The whole `tail differs` residue is 380 pairs over **44 symbols / 30 roots**, so the concentration is a property of the two largest signatures and not of the population",
        "`framed` 123 pairs · 83 symbols · 76 TUs · **1 root**; `tail` 380 pairs · 44 symbols · 332 TUs · 30 roots",
        "rungs/2026-08-08-w-alloc3.md §3; `work/w-alloc3/pop.py`, `pop.txt` — **#925/#952 is the standing caution and this is its sharpest instance yet.** Any rule fitted on those two signatures is fitted on two cells, whatever its pair count says, and that is why this lane's whole argument rests on a holdout of its own manufacture rather than on the workload",
    ),
    (
        1070,
        "**c2 CANONICALISES A COMMUTATIVE OPERAND PAIR BY REGISTER NUMBER — the description that fits all six permutation cells, DELIBERATELY NOT PROMOTED**",
        "**A POST-HOC DESCRIPTION, 6 of 6, and it is not a rule.** `int g(int a,int b,int c){return a-b+c;}` is symmetric in `a` and `c`. Across all six bindings of a three-formal caller, c2 orders that pair so the operand in the **lower-numbered register** comes first — which predicts the three HITs and the three MISSes of #1067 exactly, and also `A-two-SUM-10`, where `add r3,r3,r4` survives a swapped binding unchanged",
        "6 of 6 permutations + 2 `A-two` cells, **0 cells compiled to test it**",
        "rungs/2026-08-08-w-alloc3.md §4.3 — **NOT PROMOTED, and #912 is why in as many words**: this is a description fitted on the cells that refuted its predecessor, which is precisely the standing `RULE W2` had at 388 of 388 before it died on 14 of 106. The grid that would decide it is named in the rung and was not built by the lane that wrote it",
    ),
    (
        1071,
        "**THE CONSTANT FOLD IS MECHANICALLY PREDICTABLE AND IS PRICED HERE RATHER THAN TAKEN**",
        "**MEASURED on one cell and REFUSED (clause D11).** `int g(int a){return a+1;}` at a site `g(x0)+5` is not two instructions with a temp — c2 emits **`addi r3,r3,6`**, and the inlined value never takes a register at all. The fold `addi r3,rA,c1` at `± K` → `addi r3,rA,c1+K` reproduces `38630006` exactly",
        "1 GRID-A cell; the clause refuses **1** of 38 there and **0** of 46 on GRID-H",
        "rungs/2026-08-08-w-alloc3.md §4.2, `work/w-alloc3/ADDENDUM-2.md` §A2.3 — not taken for two reasons, both registered before the re-grade: a constant fold is not an allocation, and it would be a branch of the rule **fitted on the single cell that produced it**, which is the standing every one of the seven dead keys had. It is `w-seq` §4.2's third field family (~92 displacement folds + the constant folds) and it needs its own lane",
    ),
    (
        1072,
        "**THREE NARROWINGS, EACH REGISTERED BEFORE THE POPULATION IT SHRANK WAS GRADED, AND EACH COSTED IN CELLS**",
        "**REGISTERED AND PRICED.** `D9` (clobber) was written after the grids existed and before the first `cl.exe`; `D10` (commutative) and `D11` (const fold) after GRID-A and before GRID-H's column was frozen. `D10` is stated with **no free parameter** — it refuses the identity binding too, costing two GRID-A hits, rather than the cheaper *\"refuse only when the substitution REORDERS the pair\"* which would have been fitted on the direction c2 happened to pick in one cell. Its **indexed-form** members (`lwzx`/`stwx`/…) went in a priori, on the ground that `RA+RB` is a sum, before any cell containing one was compiled",
        "13 of 84 cells refused: `D10` **6** · `D7` 2 (registered) · `D2` 2 (registered) · `D4` 2 · `D11` **1**",
        "rungs/2026-08-08-w-alloc3.md §4.2; `work/w-alloc3/ADDENDUM-1.md`, `ADDENDUM-2.md` — **a narrowing can turn a miss into a refusal and can never turn a refusal into a wrong emit**, which is `codegen::alloc`'s own discipline applied one level up. Every refused cell is counted and printed, so the shrinkage reads as *\"RULE BIND is narrow\"* rather than as *\"RULE BIND is general\"*",
    ),
    (
        1073,
        "**A FREEZER THAT IMPORTED ITS OWN GRADER RAN IT AT IMPORT AND GRADED THE HOLDOUT ONE STEP EARLY**",
        "**FOUND, REPORTED, AND FIXED.** `freeze_h.py` imports `run_grid.py` for `compile_cell` / `text_comdats` — the alternative was a second COFF reader, which `docs/GAPS.md` §6 forbids. `run_grid.py` called `main()` at module level with no `__name__` guard, so the import **executed the grader over `sys.argv`**, which was GRID-H's, and printed the holdout's verdict before the frozen column had been written. **The verdict is unaffected** — it came from the program frozen at `ccf208e3`, with no refinement between the freeze and the grade — and it **reproduces exactly** under `--frozen`, 46 of 46 `sha256` re-checked, same 33/5/8",
        "1 accidental grade; re-graded under the frozen column, **identical**",
        "rungs/2026-08-08-w-alloc3.md §6.1; `work/w-alloc3/run_grid.py`'s guard carries the note — **the general form**: a module whose `main()` is not guarded is a *side effect* that any `import` fires, and the thing it fired here was the one step the whole protocol exists to order. Reported rather than smoothed, because a holdout protocol that is described and not audited is not a protocol",
    ),
    (
        1074,
        "**D4 COUNTS `r3`-AS-RETURN-REGISTER AS A TEMP, AND IT COST TWO CELLS TO A CONSERVATIVE REFUSAL**",
        "**A DEFECT OF THIS LANE'S DOMAIN CLAUSE, in the safe direction.** `D4` requires every register the callee touches that is not one of *its own formals* to sit strictly above the caller's formal high-water mark. A callee with **no** formals — `int g(){return 42;}` — writes its result into `r3`, which `D4` then reads as a temp colliding with the caller's first formal, and both `H-noarg` cells refuse. The clause should exempt the ABI return register",
        "**2** of 46 GRID-H cells refused for a reason that is not a real collision. 0 wrong either way",
        "rungs/2026-08-08-w-alloc3.md §5.3 — a refusal is never wrong, so this changes no verdict; it is recorded because the *next* lane's holdout is 2 cells smaller than it needs to be and would otherwise re-discover this by re-writing the clause",
    ),
    (
        1075,
        "**THE HYPOTHESIS WAS CHECKED AGAINST THE RECORD BEFORE IT WAS CHECKED AGAINST A COMPILER — five published witnesses, reproduced from bytes alone, with no toolchain**",
        "**MEASURED, and it is a `python3 bind.py` with no obj in sight.** `work/w-alloc3/bind.py`'s self-test reproduces `w-seq`'s 123-witness (`80630004` → `81630004 386bfffc`), its 286-witness (all four `r3` source fields → `r4`, both `r11`/`r10` destinations untouched), and its hand cells `s01` (no trailing `mr`), `s03` and `s11` (`7c641850` → `7c632050`) — each from the bytes those rungs published",
        "**5 of 5** published witnesses, 0 objs",
        "rungs/2026-08-08-w-alloc3.md §2 — **this is what made the refutation worth running.** A hypothesis that cannot reproduce the record's own witnesses is refuted before it costs a compile; one that reproduces all five and *still* dies on fresh structure is the RULE W2 shape, and saying so needs both halves measured",
    ),
    (
        1076,
        "**CONTROL — NOTHING MOVED, AND THE `crates/` DIFF IS PROVED INERT BY THE BINARY RATHER THAN ARGUED**",
        "**HELD.** `git diff master..HEAD -- crates/` is **one file**, `codegen/alloc.rs`, doc block + one `#[cfg(test)]` test. The release `c2rs` built with and without that diff is **byte-identical** (`md5 8413ed165089a7671e1da14608d4ac1c`), so this lane's 878-TU scan *is* master's, measured rather than subtracted. Partition at the tip: `exact` **35,986** · `reloc-differs` **861** · `differs` **2,334** · `partial` **0** · `refused` **130,579** · `unbound` **9,217** of **178,977**, `exact-bytes` **36,847**, `framed-differs` **123**; TU match **10**, `mismatch` **0**. `peerkeys.py`: **0 families vanished**",
        "gate **18/18 PASS, 0 mismatch**; `cargo test --workspace --release` **35 targets / 1,087 passed / 0 failed**, master's own being 35 / **1,086** / 0 — this lane adds exactly **one** test",
        "rungs/2026-08-08-w-alloc3.md §7; `work/w-alloc3/tip_metrics.txt`, `peerkeys.txt`, `gate_tip.txt` — the two `gap-metric` families that differ from this lane's `f0d24e46` baseline (`fnbyte-noeffect-*`, `fnbyte-blr-stop3-*`) are **`w-memset`'s**, which landed mid-lane, and the identical binary is what attributes them",
    ),
]


def main():
    txt = open(BOARD).read()
    for n, *_ in ROWS:
        pat = "| **%d**" % n
        if pat in txt:
            raise SystemExit("FAIL HARD: board #%d is already taken" % n)
    anchor = "## Declined and refuted — the rows that saved work\n\n| # | item | verdict | number | where |\n|---|---|---|---|---|\n"
    if txt.count(anchor) != 1:
        raise SystemExit("FAIL HARD: anchor is not unique (%d)" % txt.count(anchor))
    block = ""
    for n, item, verdict, num, where in ROWS:
        block += "| **%d**<sub>w-alloc3</sub> | %s | %s | %s | %s |\n" % (
            n, item, verdict, num, where)
    open(BOARD, "w").write(txt.replace(anchor, anchor + block, 1))

    txt2 = open(BOARD).read()
    nums = re.findall(r"^\| \*\*(\d+)\*\*", txt2, re.M)
    if len(nums) != len(set(nums)):
        raise SystemExit("FAIL HARD: duplicate board numbers after the write")
    print("board: %d rows, %d distinct, 0 duplicates; minted %s"
          % (len(nums), len(set(nums)), [r[0] for r in ROWS]))


main()
