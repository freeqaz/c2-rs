// GRID-L cell l1_n2 — 2 formals, a literal inserted at slot 1, so
// 1 formal(s) move up one register. ONE move — vsprintf_s's own family.
// C++ linkage and long names: the 8-byte inline-name fence (GRID-T) cannot
// decide this cell.
int grid_l_callee_l1_n2(int, int, int);
int grid_l_forward_l1_n2(int formal_number_0, int formal_number_1) { return grid_l_callee_l1_n2(formal_number_0, 0, formal_number_1); }
