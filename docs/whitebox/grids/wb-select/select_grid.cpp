// select_grid.cpp — lane wb-select (WB-I), campaign 2 (2026-08-09).
//
// THE GRID.  Frozen together with frozen.tsv and committed BEFORE its first
// cl.exe.  Every cell's predicted word sequence is in frozen.tsv, derived from
// the reading of c2's selection tables and idiom expanders in
// WB_SELECT_FINDINGS.md §1-§4 — NOT from any disassembly of this obj.
//
// Calibration (grids/wb-select/calib.cpp) ran first and was read for SECTION
// SIZES ONLY.  It is unscored; it exists so a folding compiler cannot refute
// the grid with its own cells (wb-inline §3.1, wb-loop §6.1).
//
// Mode: /nologo /c /GR /O1 /Oi /EHsc  (WB-D's workload mode).
// One COMDAT per cell.
//
// SEPARATION ASSERTION, made before the run: 11 of the 12 cells predict a word
// sequence `c2-core` does not emit today (it ships straight-line int
// add-chains, tail calls, one framed non-leaf call, and four transcribed body
// shapes).  Four cells (S1, S3, S5, S11) turn on a COMBINATION of operators
// rather than one.  No cell is a loop, a call, or a transcribed shape.

extern "C" {

// --- S1: value-producing unsigned relational, ARBITRARY result constants.
//         The flagship: the carry expander with a mask AND a bias.
int sel_ltu_ab(unsigned x){ return x < 10u ? 7 : 3; }

// --- S2: the same shape, SIGNED.  The carry expander must refuse.
int sel_lts_ab(int x){ return x < 10 ? 7 : 3; }

// --- S3: x == 0 -> the cntlzw rival.
int sel_eqz(int x){ return x == 0; }

// --- S4: x != 0 -> which rival wins?
int sel_nez(int x){ return x != 0; }

// --- S5: signed divide by a power of two -> the two-word sign-bias idiom.
int sel_divs8(int x){ return x / 8; }

// --- S6: signed divide by a non-power-of-two constant.
int sel_divs3(int x){ return x / 3; }

// --- S7: AND with a byte mask.
unsigned sel_and_ff(unsigned x){ return x & 0xffu; }

// --- S8: OR with a constant wider than 16 bits.
unsigned sel_or_big(unsigned x){ return x | 0x12345u; }

// --- S9: signed char load + widen.
int sel_schar(signed char *p){ return p[0] + 1; }

// --- S10: signed short load + widen.
int sel_short(short *p){ return p[0] + 1; }

// --- S11: unsigned relational whose result constants differ by a power of two
//          and whose false-value is 0 -> mask, no bias.
int sel_ltu_pow2(unsigned x){ return x < 10u ? 8 : 0; }

// --- S12: the same relational consumed by a BRANCH, not producing a value.
int sel_br_u(unsigned x){ if (x < 10u) return 1; return 2; }

}
