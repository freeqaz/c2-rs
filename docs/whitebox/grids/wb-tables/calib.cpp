// calib.cpp — lane wb-tables (WB-J), 2026-08-09.  UNSCORED CALIBRATION.
//
// Compiled FIRST, before the graded grid (mask_grid.cpp) is frozen.  Nothing
// here is a prediction and nothing here is scored.  It exists because
// wb-inline's v1 grid was refuted by its own cells: a grid whose cells fold,
// or whose cells all land in one arm of the mechanism, measures nothing.
//
// What it is calibrating: the `rlandi` (AND-with-constant) expansion, the pass
// that beat BOTH prior WB-I runs.  The disassembly (FUN_10c0a2e2 @ 0x10c0a2e2,
// reached from the expansion switch FUN_10c0d57e @ 0x10c0dabc) says the
// decision is MASK SHAPE: a mask that is a valid PowerPC rotate-mask (a
// contiguous run of 1s, wrapping allowed — FUN_10c04daf @ 0x10c04daf returns
// 0xffffffff exactly for those) becomes `rlwinm`; otherwise the 16-bit cases
// go to `andi.`/`andis.` and the rest to a materialised constant + `and`.
// wb-select2's diagnostics say something else entirely (the SOURCE and
// DESTINATION registers coinciding).  These cells span both axes so the
// graded grid can be aimed at whichever one survives.
//
// Mode: /nologo /c /GR /O1 /Oi /EHsc

extern "C" {

// ---- axis 1: mask shape, in a plain expression (dst and src both r3) -------
unsigned c_m_1     (unsigned x){ return x & 1u; }
unsigned c_m_8     (unsigned x){ return x & 8u; }
unsigned c_m_ff    (unsigned x){ return x & 0xffu; }
unsigned c_m_f0    (unsigned x){ return x & 0xf0u; }
unsigned c_m_ffff0 (unsigned x){ return x & 0xffff0000u; }
unsigned c_m_wrap  (unsigned x){ return x & 0xf000000fu; }   // contiguous, WRAPPING
unsigned c_m_101   (unsigned x){ return x & 0x101u; }        // NOT contiguous, fits u16
unsigned c_m_10001 (unsigned x){ return x & 0x10001u; }      // NOT contiguous, wide
unsigned c_m_f0f0  (unsigned x){ return x & 0xf0f0u; }       // NOT contiguous, fits u16

// ---- axis 2: the same masks where the AND's result feeds something else,
//              so the destination register need not equal the source ---------
unsigned c_two_8   (unsigned a, unsigned b){ return (a & 8u) + (b & 8u); }
unsigned c_two_ff  (unsigned a, unsigned b){ return (a & 0xffu) | (b & 0xff00u); }
unsigned c_and_add (unsigned a, unsigned b){ return (a & 0xf0u) + b; }

// ---- axis 3: the wb-select2 anomaly, both sides of it ---------------------
int c_rel_bias0_8  (unsigned x){ return x < 10u ? 8 : 0; }    // mask 8, NO bias
int c_rel_bias3_8  (unsigned x){ return x < 10u ? 11 : 3; }   // mask 8, bias 3
int c_rel_bias0_ff (unsigned x){ return x < 10u ? 255 : 0; }  // mask 0xff, NO bias
int c_rel_bias1_ff (unsigned x){ return x < 10u ? 256 : 1; }  // mask 0xff, bias 1
int c_rel_bias0_101(unsigned x){ return x < 10u ? 257 : 0; }  // NON-contiguous, no bias
int c_rel_bias1_101(unsigned x){ return x < 10u ? 258 : 1; }  // NON-contiguous, bias 1

}
