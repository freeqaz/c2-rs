// calib.cpp — lane wb-select (WB-I).  CALIBRATION, UNSCORED.
//
// Deliberately shares NO cell with the graded grid: every relation and every
// constant here differs from select_grid.cpp.  Only the per-COMDAT .text SIZE
// is read (scripts/gt_dump.py --no-disasm); no word sequence from this file is
// looked at before the graded predictions are frozen.
//
// It answers three input questions, per the wb-inline v1 lesson:
//   (a) does the toolchain produce an obj at all in this worktree;
//   (b) does a `?:` over two constants survive /O1 as a value (not a branch);
//   (c) does the `if`-spelling of the same thing differ from the `?:` spelling.

extern "C" {

// (b) value-producing relational, unsigned, both operands live
int wbk_1(unsigned x) { return x <= 3u ? 7 : 9; }

// (c) the if-spelling of the same predicate
int wbk_2(unsigned x) { if (x <= 3u) return 7; return 9; }

// signed relational producing a value
int wbk_3(int x) { return x > 2 ? 1 : 0; }

// two-register unsigned relational
int wbk_4(unsigned a, unsigned b) { return a >= b ? 1 : 0; }

// arithmetic folding control (the wb-inline v1 failure mode)
int wbk_5(int x) { return x * 3; }

// constant divide (sibling of the graded /8 cell, different divisor)
int wbk_6(int x) { return x / 5; }

}
