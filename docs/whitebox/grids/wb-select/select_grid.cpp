// select_grid.cpp — lane wb-select (WB-I), campaign 2026-08-08.  THE FROZEN GRID.
//
// Grades the instruction-selection reading in docs/whitebox/WB_SELECT_FINDINGS.md:
// the per-type opcode tables installed by FUN_10c04cb9 @ 0x10c04cb9, the
// per-operator dispatch FUN_10c0f882 @ 0x10c0f882, and above all the
// value-producing-relational machinery — the profitability predicate
// FUN_10c1b315 @ 0x10c1b315, the driver FUN_10c1b517 @ 0x10c1b517 and its two
// costed expanders FUN_10c1ac5c @ 0x10c1ac5c (the CARRY idiom) and
// FUN_10c1af2d @ 0x10c1af2d (the CNTLZW idiom).
//
// FROZEN BEFORE THE FIRST cl.exe OF THIS FILE.  Per-cell predictions, with the
// rivals, are in frozen.tsv.  A calibration pass (work/wb-select/calib.cpp,
// UNSCORED, six cells that share no relation and no constant with this file)
// was compiled first; what it showed and what it changed is disclosed in
// WB_SELECT_FINDINGS.md §6.1.
//
// One COMDAT per cell (/Gy), so every cell reads out of one obj by name.
//
// Compile: work/wb-select/run.sh <this> <out.obj>
//          == wibo cl.exe /nologo /c /GR /O1 /Oi /EHsc /Gy
// Read:    scripts/gt_dump.py <out.obj>

extern "C" {

// =====================================================================
// BLOCK S — value-producing relationals whose two results are NOT {0,1},
//           so the select tuple keeps its integer type and goes through
//           FUN_10c1b517's two-strategy cost race.
// =====================================================================

// S1: WB-D's flagship, verbatim.  Reading: relation `<u` normalises to `>u`
//     by swapping the COMPARE operands, so the constant becomes the left
//     operand and is materialised with `li`; delta = 1-2 = -1 kills the mask;
//     base 2 emits the addi.  Cost 4.
int wbs_s1(unsigned x) { return x < 10u ? 1 : 2; }

// S2: same relation, both operands in registers -> no `li`.  Cost 3.
int wbs_s2(unsigned a, unsigned b) { return a < b ? 1 : 2; }

// S3: canonical `>u` against a constant -> subfic, and a delta that is NOT -1
//     so the rlandi mask survives.  Cost 4.
int wbs_s3(unsigned x) { return x > 7u ? 12 : 4; }

// S4: equality against zero — the cell where the two strategies race.
int wbs_s4(unsigned x) { return x == 0 ? 5 : 6; }

// S5: SIGNED relation against a non-zero constant.  Both expanders refuse
//     (cost 500), so FUN_10c1b315's 2*cost < 20 test fails and there is no
//     if-conversion at all.  Predicts a real compare-and-branch.
int wbs_s5(int x) { return x < 10 ? 1 : 2; }

// S6: SIGNED relation against zero — strategy A still refuses, strategy B
//     accepts with its extra cntlzw (bVar5).
int wbs_s6(int x) { return x < 0 ? 1 : 2; }

// =====================================================================
// BLOCK B — the same relations as a bool-valued expression (results {0,1}).
// =====================================================================

int wbs_b1(unsigned x) { return (int)(x < 10u); }
int wbs_b2(unsigned a, unsigned b) { return (int)(a > b); }
int wbs_b3(int x) { return (int)(x < 0); }

// =====================================================================
// BLOCK K — operators outside the relational family, and one combination.
// =====================================================================

// K1: signed divide by a power of two.  One C operator, two words, and the
//     second one consumes XER[CA] from the first.  The port emits neither.
int wbs_k1(int x) { return x / 8; }

// K2: AND with a contiguous 8-bit mask — the rlandi pseudo-op's expansion
//     (FUN_10c0a2e2 @ 0x10c0a2e2) choosing between rlwinm and andi.
int wbs_k2(int x) { return x & 0xFF; }

// K3: COMBINATION — a value-producing relational feeding an add.
unsigned wbs_k3(unsigned a, unsigned b, unsigned c) { return (a < b) + c; }

}
