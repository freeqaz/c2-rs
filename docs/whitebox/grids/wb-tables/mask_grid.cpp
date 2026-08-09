// mask_grid.cpp — lane wb-tables (WB-J), 2026-08-09.  THE FROZEN GRID.
//
// FROZEN BEFORE ITS FIRST cl.exe, together with frozen.tsv.  Grades the
// reading of the AND-with-constant (`rlandi`, machine opcode 622) expansion in
// docs/whitebox/WB_TABLES_FINDINGS.md §3 — the one pass BOTH prior WB-I runs
// named as undecidable:
//
//   * WB_SELECT_FINDINGS.md   §2.4 named FUN_10c0a2e2 @ 0x10c0a2e2 and froze
//     "a contiguous mask is always rlwinm, never andi." (rival R-M1, 5/5);
//   * WB_SELECT_FINDINGS_R2.md §6.3b named FUN_10c1772b @ 0x10c1772b instead,
//     retracted the form as unpredictable, and bounded it black-box as
//     "rlwinm when rlandi's src and dst coincide, li+and otherwise".
//
// This lane read both.  FUN_10c0a2e2 IS the expander (run 1 is right about the
// site); FUN_10c1772b is a peephole COMBINER, not an expander (run 2's §4 is
// corrected).  The rule this grid tests is in frozen.tsv.
//
// Calibration: grids/wb-tables/calib.cpp (19 cells) plus a scratch second
// round in work/wb-tables/calib2.cpp, both UNSCORED, both compiled before this
// file was written.  What they changed is disclosed in the findings doc §3.6.
//
// Mode: /nologo /c /GR /O1 /Oi /EHsc   (WB-D's workload mode, run 2's flags).
// Read: scripts/gt_dump.py <out.obj> --text-only

extern "C" {

// ===== BLOCK M — the mask SHAPE rule, in a plain expression ================
unsigned m1_run     (unsigned x){ return x & 0x7f8u; }        // contiguous, interior
unsigned m2_low16   (unsigned x){ return x & 0xffffu; }       // contiguous, low half
unsigned m3_split16 (unsigned x){ return x & 0x8001u; }       // NOT contiguous, fits u16
unsigned m4_wrap    (unsigned x){ return x & 0x80000001u; }   // contiguous, WRAPPING
unsigned m5_two     (unsigned x){ return x & 0x00ff00ffu; }   // 2 runs, residue contiguous
unsigned m6_four    (unsigned x){ return x & 0xf0f0f0f0u; }   // 4 runs, residue NOT contiguous

// ===== BLOCK D — the discriminator between this lane's rule and run 2's ====
// The rlandi here has base == 0 (no `addi` bias) but its source and its
// destination are the SAME register (both the r11 temp), because the result is
// consumed by the add rather than returned.  Run 2's bound says `rlwinm`;
// this lane's rule says `li` + `and`.  Exactly one can survive.
unsigned d1_consumed(unsigned x, unsigned y){ return (x < 10u ? 8u : 0u) + y; }

// ===== BLOCK R — the relational idiom's own mask step =====================
int r1_bias16 (unsigned x){ return x < 10u ? 0x18 : 0x10; }   // delta 8, base 16
int r2_pow_nb (unsigned x){ return x < 10u ? 0x100 : 0; }     // delta contiguous, base 0
int r3_spl_nb (unsigned x){ return x < 10u ? 0x101 : 0; }     // delta split,      base 0
int r4_regbnd (unsigned a, unsigned b){ return a < b ? 6 : 2; }  // delta 4, base 2
int r5_bias1  (unsigned x){ return x < 10u ? 0x8001 : 1; }    // delta 0x8000, base 1

}
