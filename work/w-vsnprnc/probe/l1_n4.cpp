// GRID-L cell l1_n4 — 4 formals, a literal inserted at slot 3, so
// 1 formal(s) move up one register. ONE move — vsprintf_s's own family.
// C++ linkage and long names: the 8-byte inline-name fence (GRID-T) cannot
// decide this cell.
int grid_l_callee_l1_n4(int, int, int, int, int);
int grid_l_forward_l1_n4(int formal_number_0, int formal_number_1, int formal_number_2, int formal_number_3) { return grid_l_callee_l1_n4(formal_number_0, formal_number_1, formal_number_2, 0, formal_number_3); }
