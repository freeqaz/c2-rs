// calib.cpp — lane wb-select (WB-I), campaign 2 (2026-08-09).  CALIBRATION ONLY.
//
// NOT the grid.  Per the lane brief and wb-loop §6.1, this pass exists so the
// frozen grid is not refuted by its own cells the way wb-inline's v1 grid was
// (a folding compiler collapsed the ladder).  It is read for SECTION SIZES
// ONLY — word counts, never word sequences — and nothing here is scored.
//
// What it must establish before select_grid.cpp is written:
//   * that a value-producing relational on a parameter survives /O1 as a
//     branchless sequence, and how many words it costs;
//   * whether the signed/unsigned split is visible at all in word count;
//   * that the div/mul-by-constant cells are not folded away;
//   * how many words a narrowing (signed char / short) parameter costs.
//
// Mode: /nologo /c /GR /O1 /Oi /EHsc  (WB-D's workload mode, for comparability)

extern "C" {

// --- C-A: value-producing relationals, unsigned vs signed, same shape.
int cal_ltu (unsigned x){ return x <  10u ? 1 : 2; }
int cal_lts (int      x){ return x <  10  ? 1 : 2; }
int cal_ltu_ab(unsigned x){ return x < 10u ? 7 : 3; }
int cal_geu (unsigned x){ return x >= 10u ? 1 : 2; }

// --- C-B: relational-to-bool (the plain `return cmp;` form).
int cal_ltu_b(unsigned x){ return x <  10u; }
int cal_lts_b(int      x){ return x <  10;  }
int cal_eqz  (int      x){ return x == 0;   }
int cal_nez  (int      x){ return x != 0;   }
int cal_ltz  (int      x){ return x <  0;   }

// --- C-C: relational consumed by a BRANCH (the context bit).
int cal_br_u (unsigned x){ if (x < 10u) return 1; return 2; }
int cal_br_s (int      x){ if (x < 10)  return 1; return 2; }

// --- C-D: logical / mask, constant operand.
unsigned cal_and_ff (unsigned x){ return x & 0xffu; }
unsigned cal_and_lo (unsigned x){ return x & 0x3fffu; }
unsigned cal_or_imm (unsigned x){ return x | 0x1234u; }
unsigned cal_or_big (unsigned x){ return x | 0x12345u; }
unsigned cal_xor_imm(unsigned x){ return x ^ 0x1234u; }

// --- C-E: shifts.
unsigned cal_shl (unsigned x){ return x << 3; }
unsigned cal_shru(unsigned x){ return x >> 3; }
int      cal_shrs(int      x){ return x >> 3; }

// --- C-F: div / mul by a constant.
int      cal_divs8 (int      x){ return x / 8; }
unsigned cal_divu8 (unsigned x){ return x / 8u; }
int      cal_divs3 (int      x){ return x / 3; }
int      cal_mul6  (int      x){ return x * 6; }
int      cal_mul8  (int      x){ return x * 8; }
int      cal_mulbig(int      x){ return x * 100000; }

// --- C-G: narrowing loads / sign extension (mixed signedness).
int cal_schar(signed char *p){ return p[0] + 1; }
int cal_uchar(unsigned char *p){ return p[0] + 1; }
int cal_short(short *p){ return p[0] + 1; }
int cal_ushort(unsigned short *p){ return p[0] + 1; }

// --- C-H: comparison immediates that do NOT fit 16 bits.
int cal_gtu_big(unsigned x){ if (x > 100000u) return 1; return 2; }
int cal_gts_big(int      x){ if (x > 100000)  return 1; return 2; }

// --- C-I: a plain arithmetic control, to confirm nothing exotic happens.
int cal_add (int a, int b){ return a + b; }
int cal_sub (int a, int b){ return a - b; }
int cal_subk(int a){ return a - 7; }

}
